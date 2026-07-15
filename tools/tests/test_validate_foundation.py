from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest import mock

from tools.validate_foundation import (
    DuplicateKeyError,
    FoundationValidator,
    GATE0_GIT_EXECUTABLE,
    GATE0_MAXIMUM_BINARY_FILE_BYTES,
    GATE0_MAXIMUM_JSON_NESTING_DEPTH,
    GATE0_MAXIMUM_REPOSITORY_BYTES,
    GATE0_IGNORE_PATTERNS,
    GATE0_MAXIMUM_TEXT_FILE_BYTES,
    _fallback_repository_files,
    audit_schema_vocabulary,
    load_json,
    markdown_anchors,
    markdown_fence_error,
    markdown_html_comment_error,
    markdown_inline_link_targets,
    git_index_entries,
    iter_repository_files,
    unsafe_run_interpolations,
    valid_format,
    validate_schema_instance,
    workflow_jobs,
)


def protected_file_policy() -> dict[str, object]:
    source = Path(__file__).resolve().parents[2] / "policy/gate0-repository-policy.json"
    return {"protected_file_digests": load_json(source)["protected_file_digests"]}


class _FakePipe:
    def __init__(self, data: bytes) -> None:
        self.file = tempfile.TemporaryFile()
        self.file.write(data)
        self.file.seek(0)

    def read(self, size: int) -> bytes:
        return self.file.read(size)

    def fileno(self) -> int:
        return self.file.fileno()

    def close(self) -> None:
        self.file.close()

    def __del__(self) -> None:
        self.file.close()

    @property
    def closed(self) -> bool:
        return self.file.closed


class _FakePopen:
    def __init__(self, data: bytes, return_code: int = 0) -> None:
        self.stdout = _FakePipe(data)
        self.return_code = return_code
        self.kill_count = 0
        self.wait_count = 0

    def kill(self) -> None:
        self.kill_count += 1

    def wait(self, timeout: float | None = None) -> int:
        del timeout
        self.wait_count += 1
        return self.return_code


class _FailingFirstWaitPopen(_FakePopen):
    def wait(self, timeout: float | None = None) -> int:
        del timeout
        self.wait_count += 1
        if self.wait_count == 1:
            raise OSError("wait failed")
        return self.return_code


class _TimeoutWaitPopen(_FakePopen):
    def __init__(self, data: bytes) -> None:
        super().__init__(data)
        self.wait_timeouts: list[float | None] = []

    def wait(self, timeout: float | None = None) -> int:
        self.wait_count += 1
        self.wait_timeouts.append(timeout)
        raise subprocess.TimeoutExpired("git", timeout)


class JsonParsingTests(unittest.TestCase):
    def test_duplicate_keys_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "duplicate.json"
            path.write_text('{"scope":"first","scope":"second"}\n', encoding="utf-8")
            with self.assertRaises(DuplicateKeyError):
                load_json(path)

    def test_unambiguous_json_loads(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "record.json"
            path.write_text('{"scope":"one","status":true}\n', encoding="utf-8")
            self.assertEqual(load_json(path), {"scope": "one", "status": True})

    def test_i_json_integer_boundaries_and_negative_zero_are_accepted(self) -> None:
        cases = (
            ("9007199254740991", 9007199254740991),
            ("-9007199254740991", -9007199254740991),
            ("-0", 0),
        )
        for literal, expected in cases:
            with self.subTest(literal=literal), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "integer.json"
                path.write_text(literal, encoding="utf-8")
                self.assertEqual(load_json(path), expected)

    def test_integers_one_beyond_i_json_boundaries_are_rejected(self) -> None:
        for literal in ("9007199254740992", "-9007199254740992"):
            with self.subTest(literal=literal), tempfile.TemporaryDirectory() as directory:
                path = Path(directory) / "integer.json"
                path.write_text(literal, encoding="utf-8")
                with self.assertRaises(json.JSONDecodeError) as raised:
                    load_json(path)
                self.assertEqual(
                    raised.exception.msg,
                    "integer exceeds the I-JSON interoperable range",
                )

    def test_attacker_sized_integer_is_rejected_before_host_conversion(self) -> None:
        literal = "9" * 4_301
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "integer.json"
            path.write_text(literal, encoding="utf-8")
            with self.assertRaises(json.JSONDecodeError) as raised:
                load_json(path)
        self.assertEqual(
            raised.exception.msg,
            "integer exceeds the I-JSON interoperable range",
        )
        self.assertNotIn(literal, raised.exception.msg)

    def test_excessive_structural_nesting_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "deep.json"
            depth = GATE0_MAXIMUM_JSON_NESTING_DEPTH + 1
            path.write_text("[" * depth + "0" + "]" * depth, encoding="utf-8")
            with self.assertRaisesRegex(
                json.JSONDecodeError,
                rf"Gate 0 limit of {GATE0_MAXIMUM_JSON_NESTING_DEPTH}",
            ):
                load_json(path)

    def test_structural_brackets_and_escapes_inside_strings_do_not_count_as_nesting(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "brackets-in-string.json"
            value = ("[{" * (GATE0_MAXIMUM_JSON_NESTING_DEPTH + 1)) + '\\"quoted\\\\tail'
            document = {"value": value}
            path.write_text(json.dumps(document), encoding="utf-8")
            self.assertEqual(load_json(path), document)

    def test_fallback_recursion_errors_are_normalized_as_json_decode_errors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "record.json"
            path.write_text("{}", encoding="utf-8")
            targets = (
                "tools.validate_foundation.json.loads",
                "tools.validate_foundation._require_unicode_scalars",
            )
            for target in targets:
                with self.subTest(target=target), mock.patch(target, side_effect=RecursionError("deep")):
                    with self.assertRaises(json.JSONDecodeError) as raised:
                        load_json(path)
                    self.assertEqual(raised.exception.doc, "{}")
                    self.assertIn("structural nesting", raised.exception.msg)


class TomlParsingTests(unittest.TestCase):
    @staticmethod
    def _deep_array() -> str:
        depth = 2_000
        return "value = " + ("[" * depth) + "0" + ("]" * depth) + "\n"

    def test_repository_toml_recursion_is_normalized_as_a_decode_error(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "deep.toml"
            path.write_text(self._deep_array(), encoding="utf-8")
            validator = FoundationValidator(root)
            with mock.patch(
                "tools.validate_foundation.tomllib.loads",
                side_effect=RecursionError("deep parser stack"),
            ):
                with self.assertRaises(tomllib.TOMLDecodeError) as raised:
                    validator._load_repository_toml(path)
        self.assertEqual(
            str(raised.exception),
            "TOML structural nesting exceeds parser capacity",
        )
        self.assertIsInstance(raised.exception.__cause__, RecursionError)

    def test_full_run_reports_toml_recursion_without_a_traceback(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        validator = FoundationValidator(source_root)
        with mock.patch(
            "tools.validate_foundation.tomllib.loads",
            side_effect=RecursionError("deep parser stack"),
        ):
            findings = validator.run()
        compiler_findings = [
            finding
            for finding in findings
            if finding.code
            in {"compiler.toolchain_toml", "compiler.manifest_toml", "compiler.lock_toml"}
        ]
        self.assertTrue(compiler_findings)
        self.assertTrue(
            all("TOML structural nesting exceeds parser capacity" in finding.message for finding in compiler_findings)
        )
        self.assertTrue(all("deep parser stack" not in finding.message for finding in compiler_findings))


class RepositoryResourceBoundTests(unittest.TestCase):
    @staticmethod
    def _sized_file(path: Path, size: int) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"\0" * size)

    def test_text_and_binary_file_size_boundaries_are_inclusive(self) -> None:
        cases = (
            ("record.txt", GATE0_MAXIMUM_TEXT_FILE_BYTES),
            ("asset.png", GATE0_MAXIMUM_BINARY_FILE_BYTES),
        )
        for name, limit in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._sized_file(root / name, limit)
                validator = FoundationValidator(root)
                self.assertTrue(validator._preflight_repository_resources())
                snapshot = validator._read_repository_bytes(root / name)
                self.assertIsNotNone(snapshot)
                self.assertEqual(len(snapshot or b""), limit)
                self.assertEqual(validator.findings, [])

    def test_preflight_fails_closed_without_secure_open_capabilities(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "record.txt").write_text("record\n", encoding="utf-8")
            validator = FoundationValidator(root)
            with mock.patch(
                "tools.validate_foundation._secure_repository_reads_supported",
                return_value=False,
            ):
                self.assertFalse(validator._preflight_repository_resources())

        self.assertEqual(
            {finding.code for finding in validator.findings},
            {"resource.unsupported_host"},
        )

    def test_text_and_binary_files_one_byte_over_the_limit_are_rejected(self) -> None:
        cases = (
            ("record.txt", GATE0_MAXIMUM_TEXT_FILE_BYTES),
            ("asset.png", GATE0_MAXIMUM_BINARY_FILE_BYTES),
        )
        for name, limit in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._sized_file(root / name, limit + 1)
                validator = FoundationValidator(root)
                self.assertFalse(validator._preflight_repository_resources())
                self.assertIsNone(validator._read_repository_bytes(root / name))
                self.assertIn("resource.file_size", {finding.code for finding in validator.findings})

    def test_aggregate_boundary_is_inclusive_and_one_extra_byte_is_rejected(self) -> None:
        for extra_bytes, expected in ((0, True), (1, False)):
            with self.subTest(extra_bytes=extra_bytes), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                file_count = GATE0_MAXIMUM_REPOSITORY_BYTES // GATE0_MAXIMUM_BINARY_FILE_BYTES
                for index in range(file_count):
                    self._sized_file(
                        root / f"asset-{index}.png",
                        GATE0_MAXIMUM_BINARY_FILE_BYTES,
                    )
                if extra_bytes:
                    self._sized_file(root / "extra.txt", extra_bytes)
                validator = FoundationValidator(root)
                self.assertEqual(validator._preflight_repository_resources(), expected)
                aggregate_findings = [
                    finding for finding in validator.findings if finding.code == "resource.aggregate_size"
                ]
                self.assertEqual(bool(aggregate_findings), not expected)

    def test_oversized_policy_stops_before_json_parsing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            policy = root / "policy/gate0-repository-policy.json"
            policy.parent.mkdir(parents=True)
            policy.write_bytes(b"{" + b"x" * GATE0_MAXIMUM_TEXT_FILE_BYTES)
            validator = FoundationValidator(root)
            findings = validator.run()
            self.assertEqual({finding.code for finding in findings}, {"resource.file_size"})

    def test_repeated_reads_reuse_one_snapshot_and_one_aggregate_charge(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "record.txt"
            path.write_bytes(b"first")
            validator = FoundationValidator(root)
            self.assertEqual(validator._read_repository_bytes(path), b"first")
            path.write_bytes(b"later")
            self.assertEqual(validator._read_repository_bytes(path), b"first")
            self.assertEqual(validator._repository_read_bytes, len(b"first"))
            self.assertEqual(validator.findings, [])

    def test_first_read_rejects_a_post_preflight_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "record.txt"
            path.write_bytes(b"before")
            validator = FoundationValidator(root)
            self.assertTrue(validator._preflight_repository_resources())
            path.write_bytes(b"changed after preflight")
            self.assertIsNone(validator._read_repository_bytes(path))
            self.assertIn(
                "resource.post_preflight_change",
                {finding.code for finding in validator.findings},
            )

    def test_first_read_rejects_a_same_size_replacement_after_preflight(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "record.txt"
            path.write_bytes(b"first!")
            validator = FoundationValidator(root)
            self.assertTrue(validator._preflight_repository_resources())
            replacement = root / "replacement.txt"
            replacement.write_bytes(b"other!")
            os.replace(replacement, path)

            self.assertIsNone(validator._read_repository_bytes(path))

        self.assertIn(
            "resource.post_preflight_change",
            {finding.code for finding in validator.findings},
        )
        self.assertEqual(validator._repository_read_bytes, 0)

    @unittest.skipUnless(
        os.name == "posix" and hasattr(os, "symlink"),
        "component-relative no-follow opens require POSIX symlinks",
    )
    def test_secure_open_does_not_follow_a_parent_swapped_after_inspection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            nested = root / "nested"
            nested.mkdir(parents=True)
            path = nested / "record.txt"
            path.write_text("inside\n", encoding="utf-8")
            outside = parent / "outside"
            outside.mkdir()
            (outside / "record.txt").write_text("outside\n", encoding="utf-8")
            validator = FoundationValidator(root)
            self.assertTrue(validator._preflight_repository_resources())
            original_inspect = validator._inspect_repository_file
            saved = root / "saved"
            swapped = False

            def inspect_then_swap(candidate: Path):
                nonlocal swapped
                result = original_inspect(candidate)
                if result is not None and not swapped:
                    nested.rename(saved)
                    nested.symlink_to(outside, target_is_directory=True)
                    swapped = True
                return result

            with mock.patch.object(
                validator,
                "_inspect_repository_file",
                side_effect=inspect_then_swap,
            ):
                snapshot = validator._read_repository_bytes(path)

        self.assertIsNone(snapshot)
        self.assertIn("resource.unreadable", {finding.code for finding in validator.findings})
        self.assertEqual(validator._repository_read_bytes, 0)

    @unittest.skipUnless(
        os.name == "posix" and hasattr(os, "symlink"),
        "component-relative no-follow inspection requires POSIX symlinks",
    )
    def test_resource_inspection_does_not_follow_a_parent_swapped_after_metadata(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            nested = root / "nested"
            nested.mkdir(parents=True)
            path = nested / "record.txt"
            path.write_text("inside\n", encoding="utf-8")
            outside = parent / "outside"
            outside.mkdir()
            (outside / "record.txt").write_text("outside\n", encoding="utf-8")
            validator = FoundationValidator(root)
            original_stat = os.stat
            saved = root / "saved"
            swapped = False

            def stat_then_swap(
                target: str | bytes | os.PathLike[str] | os.PathLike[bytes],
                *,
                dir_fd: int | None = None,
                follow_symlinks: bool = True,
            ) -> os.stat_result:
                nonlocal swapped
                metadata = original_stat(
                    target,
                    dir_fd=dir_fd,
                    follow_symlinks=follow_symlinks,
                )
                if target == "nested" and dir_fd is not None and not swapped:
                    nested.rename(saved)
                    nested.symlink_to(outside, target_is_directory=True)
                    swapped = True
                return metadata

            with mock.patch(
                "tools.validate_foundation._secure_repository_reads_supported",
                return_value=True,
            ), mock.patch(
                "tools.validate_foundation.os.stat",
                side_effect=stat_then_swap,
            ):
                inspected = validator._inspect_repository_file(path)

        self.assertIsNone(inspected)
        self.assertIn("resource.unreadable", {finding.code for finding in validator.findings})

    def test_markdown_anchor_target_uses_the_preflight_snapshot_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text(
                "# Source\n\n[target](target.md#exact-heading)\n",
                encoding="utf-8",
            )
            target = root / "target.md"
            target.write_text("# Exact heading\n", encoding="utf-8")
            validator = FoundationValidator(root)
            self.assertTrue(validator._preflight_repository_resources())
            target.write_text("# Changed heading with different bytes\n", encoding="utf-8")
            validator._validate_markdown_links()
            codes = {finding.code for finding in validator.findings}
            self.assertIn("resource.post_preflight_change", codes)
            self.assertNotIn("markdown.anchor_missing", codes)

    @unittest.skipUnless(hasattr(os, "symlink"), "symlinks are unavailable")
    def test_preflight_rejects_symlinked_repository_content(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "target.txt").write_text("target\n", encoding="utf-8")
            os.symlink("target.txt", root / "link.txt")
            validator = FoundationValidator(root)
            self.assertFalse(validator._preflight_repository_resources())
            self.assertIn("resource.symlink", {finding.code for finding in validator.findings})

    @unittest.skipUnless(hasattr(os, "link"), "hard links are unavailable")
    def test_preflight_rejects_hardlinked_repository_content(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            first = root / "first.txt"
            first.write_text("shared\n", encoding="utf-8")
            os.link(first, root / "second.txt")
            validator = FoundationValidator(root)

            self.assertFalse(validator._preflight_repository_resources())
            self.assertEqual(
                {finding.code for finding in validator.findings},
                {"resource.hardlink"},
            )

    @unittest.skipUnless(hasattr(os, "SEEK_HOLE"), "sparse-file discovery is unavailable")
    def test_preflight_rejects_sparse_repository_content(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            sparse = root / "sparse.txt"
            with sparse.open("wb") as destination:
                destination.truncate(64 * 1024)
            validator = FoundationValidator(root)

            self.assertFalse(validator._preflight_repository_resources())
            self.assertEqual(
                {finding.code for finding in validator.findings},
                {"resource.sparse"},
            )

    @unittest.skipUnless(hasattr(os, "symlink"), "symlinks are unavailable")
    def test_tree_mode_validation_does_not_follow_a_post_preflight_symlink(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            root.mkdir()
            path = root / "checked.txt"
            path.write_text("checked\n", encoding="utf-8")
            outside = parent / "outside.txt"
            outside.write_text("outside\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator.index_entries = [("100644", "checked.txt")]
            validator.policy = {"executable_paths": [], "allowed_binary_artifacts": []}
            self.assertTrue(validator._preflight_repository_resources())

            path.unlink()
            path.symlink_to(outside)
            original_stat = Path.stat

            def reject_following_stat(candidate: Path, *, follow_symlinks: bool = True):
                if follow_symlinks:
                    raise AssertionError("post-preflight mode checks must not follow paths")
                return original_stat(candidate, follow_symlinks=False)

            with mock.patch.object(Path, "stat", autospec=True, side_effect=reject_following_stat):
                validator._validate_tree_encoding_and_format()

            self.assertIn("resource.symlink", {finding.code for finding in validator.findings})

    @unittest.skipUnless(hasattr(os, "mkfifo"), "FIFOs are unavailable")
    def test_preflight_rejects_a_fifo_without_opening_it(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fifo = root / "input.pipe"
            os.mkfifo(fifo)

            validator = FoundationValidator(root)

            self.assertFalse(validator._preflight_repository_resources())
            self.assertEqual(
                {finding.code for finding in validator.findings},
                {"resource.not_file"},
            )


class RepositoryInventoryBoundTests(unittest.TestCase):
    def test_inventory_rejects_an_unsupported_host_before_spawning_git(self) -> None:
        findings = []
        with tempfile.TemporaryDirectory() as directory, mock.patch(
            "tools.validate_foundation._secure_repository_reads_supported",
            return_value=False,
        ), mock.patch(
            "tools.validate_foundation.subprocess.Popen",
            side_effect=AssertionError("Git was spawned on an unsupported host"),
        ):
            paths = list(iter_repository_files(Path(directory), findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.unsupported_host"})

    def test_static_git_excludes_match_the_protected_gitignores(self) -> None:
        root = Path(__file__).resolve().parents[2]
        root_patterns = tuple(
            line
            for line in (root / ".gitignore").read_text(encoding="utf-8").splitlines()
            if line and not line.startswith("#")
        )
        compiler_patterns = tuple(
            "compiler/" + line.removeprefix("/")
            for line in (root / "compiler/.gitignore").read_text(encoding="utf-8").splitlines()
            if line and not line.startswith("#")
        )
        self.assertEqual(GATE0_IGNORE_PATTERNS, root_patterns + compiler_patterns)

    def test_git_inventory_has_one_deadline_for_output_and_exit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            root.mkdir()
            fake_git = parent / "git"
            fake_git.write_text("#!/bin/sh\nprintf x\n/bin/sleep 1\n", encoding="utf-8")
            fake_git.chmod(0o700)
            findings = []
            with (
                mock.patch.dict(
                    "tools.validate_foundation.os.environ",
                    {"PATH": str(parent)},
                    clear=True,
                ),
                mock.patch(
                    "tools.validate_foundation.GATE0_GIT_EXECUTABLE",
                    str(fake_git),
                ),
                mock.patch(
                    "tools.validate_foundation._GT",
                    0.05,
                ),
            ):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_timeout"},
        )

    def test_git_selector_range_failure_is_fail_closed_and_reaps_producer(self) -> None:
        process = _FakePopen(b"file.txt\0")
        process.stdout.fileno = lambda: 1025
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                mock.patch(
                    "tools.validate_foundation.subprocess.Popen",
                    return_value=process,
                ),
                mock.patch(
                    "tools.validate_foundation.select.select",
                    side_effect=ValueError("filedescriptor out of range"),
                ),
            ):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_read"},
        )
        self.assertEqual((process.kill_count, process.wait_count), (1, 1))

    def test_git_inventory_without_a_descriptor_fails_before_reading(self) -> None:
        process = _FakePopen(b"file.txt\0")
        process.stdout.fileno = mock.Mock(side_effect=OSError("no descriptor"))
        process.stdout.read = mock.Mock(side_effect=AssertionError("blocking read attempted"))
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                return_value=process,
            ):
                paths = list(iter_repository_files(Path(directory), findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_protocol"})
        process.stdout.read.assert_not_called()
        self.assertEqual((process.kill_count, process.wait_count), (1, 1))


    def test_git_path_limit_is_inclusive_and_one_extra_byte_kills_the_producer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "abc").write_bytes(b"")
            accepted = _FakePopen(b"H abc\0")
            findings = []
            with (
                mock.patch("tools.validate_foundation.GATE0_MAXIMUM_REPOSITORY_PATH_BYTES", 3),
                mock.patch("tools.validate_foundation.subprocess.Popen", return_value=accepted) as popen,
            ):
                paths = list(iter_repository_files(root, findings))
            self.assertEqual([path.name for path in paths], ["abc"])
            self.assertEqual(findings, [])
            self.assertTrue(accepted.stdout.closed)
            self.assertEqual((accepted.kill_count, accepted.wait_count), (0, 1))
            self.assertIs(popen.call_args.kwargs["stdin"], subprocess.DEVNULL)
            self.assertIs(popen.call_args.kwargs["stderr"], subprocess.DEVNULL)

            rejected = _FakePopen(b"H abcd\0")
            findings = []
            with (
                mock.patch("tools.validate_foundation.GATE0_MAXIMUM_REPOSITORY_PATH_BYTES", 3),
                mock.patch("tools.validate_foundation.subprocess.Popen", return_value=rejected),
            ):
                paths = list(iter_repository_files(root, findings))
            self.assertEqual(paths, [])
            self.assertEqual({finding.code for finding in findings}, {"resource.inventory_path"})
            self.assertTrue(rejected.stdout.closed)
            self.assertEqual((rejected.kill_count, rejected.wait_count), (1, 1))

    def test_git_process_environment_replaces_inherited_git_controls(self) -> None:
        process = _FakePopen(b"H file.txt\0")
        findings = []
        environment = {
            "GIT_DIR": "/redirected/git-dir",
            "git_index_file": "/redirected/index",
            "GIT_OBJECT_DIRECTORY": "/redirected/objects",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES": "/redirected/alternates",
            "GIT_CONFIG_COUNT": "1",
            "GIT_CONFIG_KEY_0": "core.worktree",
            "GIT_CONFIG_VALUE_0": "/redirected/worktree",
            "GIT_TRACE": "1",
            "HOME": "/redirected/home",
            "HTTPS_PROXY": "http://hostile.invalid",
            "LD_LIBRARY_PATH": "/redirected/libraries",
            "LD_PRELOAD": "/redirected/preload.so",
            "LC_ALL": "host-locale",
            "PATH": "/test/bin",
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "file.txt").write_bytes(b"")
            with (
                mock.patch.dict("tools.validate_foundation.os.environ", environment, clear=True),
                mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process) as popen,
            ):
                paths = list(iter_repository_files(root, findings))
        self.assertEqual([path.name for path in paths], ["file.txt"])
        self.assertEqual(
            popen.call_args.args[0][:9],
            [
                GATE0_GIT_EXECUTABLE,
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.ignoreCase=false",
                "-c",
                "core.precomposeUnicode=false",
                "-C",
                str(root),
            ],
        )
        child_environment = popen.call_args.kwargs["env"]
        self.assertEqual(
            child_environment,
            {
                "GIT_CONFIG_GLOBAL": os.devnull,
                "GIT_CONFIG_NOSYSTEM": "1",
                "GIT_DIR": str(root / ".git"),
                "GIT_NO_LAZY_FETCH": "1",
                "GIT_NO_REPLACE_OBJECTS": "1",
                "GIT_OPTIONAL_LOCKS": "0",
                "GIT_TERMINAL_PROMPT": "0",
                "GIT_WORK_TREE": str(root),
                "LC_ALL": "C",
                "PATH": "/usr/bin:/bin",
            },
        )

    def test_git_inventory_rejects_external_gitdir_indirection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "worktree"
            root.mkdir()
            external_git = parent / "external.git"
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "init", "--bare", "--quiet", external_git],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            (root / ".git").write_text(
                f"gitdir: {external_git}\n",
                encoding="utf-8",
            )
            findings = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_git"},
        )

    def test_git_inventory_rejects_local_metadata_redirects(self) -> None:
        for relative_path in ("commondir", "config.worktree", "objects/info/alternates"):
            with self.subTest(path=relative_path), tempfile.TemporaryDirectory() as directory:
                root = Path(directory) / "worktree"
                subprocess.run(
                    [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                    check=True,
                    env={"PATH": "/usr/bin:/bin"},
                )
                redirect = root / ".git" / relative_path
                redirect.parent.mkdir(parents=True, exist_ok=True)
                redirect.write_text("../../external\n", encoding="utf-8")
                findings = []

                paths = list(iter_repository_files(root, findings))

            self.assertEqual(paths, [])
            self.assertEqual(
                {finding.code for finding in findings},
                {"resource.inventory_git"},
            )

    def test_git_inventory_accepts_empty_checkout_worktree_config(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "worktree"
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            (root / ".git/config.worktree").touch()
            findings = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual(findings, [])

    def test_git_inventory_accepts_checkout_worktree_config(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "worktree"
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "sparse-checkout", "disable"],
                cwd=root,
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "config", "--local", "--unset-all", "extensions.worktreeConfig"],
                cwd=root,
                check=False,
                env={"PATH": "/usr/bin:/bin"},
            )
            findings = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual(findings, [])

    def test_git_inventory_rejects_linked_empty_worktree_config(self) -> None:
        for link_kind in ("hardlink", "symlink"):
            if link_kind == "symlink" and not hasattr(os, "symlink"):
                continue
            with self.subTest(link_kind=link_kind), tempfile.TemporaryDirectory() as directory:
                parent = Path(directory)
                root = parent / "worktree"
                subprocess.run(
                    [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                    check=True,
                    env={"PATH": "/usr/bin:/bin"},
                )
                target = parent / "empty-config"
                target.touch()
                worktree_config = root / ".git/config.worktree"
                if link_kind == "hardlink":
                    worktree_config.hardlink_to(target)
                else:
                    worktree_config.symlink_to(target)
                findings = []

                paths = list(iter_repository_files(root, findings))

            self.assertEqual(paths, [])
            self.assertEqual(
                {finding.code for finding in findings},
                {"resource.inventory_git"},
            )

    @unittest.skipUnless(hasattr(os, "symlink"), "symlinks are unavailable")
    def test_git_inventory_rejects_a_symlinked_object_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "worktree"
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            objects = root / ".git" / "objects"
            external = parent / "external-objects"
            objects.replace(external)
            objects.symlink_to(external, target_is_directory=True)
            findings = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_git"})

    def test_git_inventory_rejects_symlinked_config_and_index(self) -> None:
        for relative_path in ("config", "index"):
            with self.subTest(path=relative_path), tempfile.TemporaryDirectory() as directory:
                parent = Path(directory)
                root = parent / "worktree"
                subprocess.run(
                    [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                    check=True,
                    env={"PATH": "/usr/bin:/bin"},
                )
                external = parent / "external-metadata"
                external.write_text("external\n", encoding="utf-8")
                redirected = root / ".git" / relative_path
                if redirected.exists():
                    redirected.unlink()
                redirected.symlink_to(external)
                findings = []

                paths = list(iter_repository_files(root, findings))

            self.assertEqual(paths, [])
            self.assertEqual(
                {finding.code for finding in findings},
                {"resource.inventory_git"},
            )

    def test_git_inventory_rejects_hardlinked_config_and_index(self) -> None:
        for relative_path in ("config", "index"):
            with self.subTest(path=relative_path), tempfile.TemporaryDirectory() as directory:
                parent = Path(directory)
                root = parent / "worktree"
                subprocess.run(
                    [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                    check=True,
                    env={"PATH": "/usr/bin:/bin"},
                )
                (root / "tracked.txt").write_text("tracked\n", encoding="utf-8")
                subprocess.run(
                    [GATE0_GIT_EXECUTABLE, "-C", root, "add", "tracked.txt"],
                    check=True,
                    env={"PATH": "/usr/bin:/bin"},
                )
                redirected = root / ".git" / relative_path
                external = parent / f"external-{relative_path}"
                redirected.replace(external)
                redirected.hardlink_to(external)
                self.assertEqual(redirected.stat().st_nlink, 2)
                findings = []

                paths = list(iter_repository_files(root, findings))

            self.assertEqual(paths, [])
            self.assertEqual(
                {finding.code for finding in findings},
                {"resource.inventory_git"},
            )

    def test_git_inventory_rejects_split_indexes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "worktree"
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            (root / "tracked.txt").write_text("tracked\n", encoding="utf-8")
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "-C", root, "add", "tracked.txt"],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "-C", root, "update-index", "--split-index"],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            findings = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_git"})

    def test_git_inventory_rejects_external_config_includes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "worktree"
            subprocess.run(
                [GATE0_GIT_EXECUTABLE, "init", "--quiet", root],
                check=True,
                env={"PATH": "/usr/bin:/bin"},
            )
            external = parent / "external.config"
            external.write_text("[core]\n\tignoreCase = true\n", encoding="utf-8")
            with (root / ".git" / "config").open("a", encoding="utf-8") as config:
                config.write(f"[include]\n\tpath = {external}\n")
            findings = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_git"})

    def test_git_wait_failure_stops_and_reaps_the_producer(self) -> None:
        process = _FailingFirstWaitPopen(b"file.txt\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "file.txt").write_bytes(b"")
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                return_value=process,
            ):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_read"},
        )
        self.assertTrue(process.stdout.closed)
        self.assertEqual((process.kill_count, process.wait_count), (1, 2))

    def test_git_timeout_cleanup_never_uses_an_unbounded_wait(self) -> None:
        process = _TimeoutWaitPopen(b"file.txt\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "file.txt").write_bytes(b"")
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                return_value=process,
            ):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_timeout"})
        self.assertEqual((process.kill_count, process.wait_count), (1, 2))
        self.assertTrue(all(timeout is not None for timeout in process.wait_timeouts))

    def test_global_git_excludes_cannot_hide_repository_files(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            home = parent / "home"
            home.mkdir()
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            global_excludes = home / "global-excludes"
            global_excludes.write_text("globally-hidden.txt\n", encoding="utf-8")
            (home / ".gitconfig").write_text(
                f"[core]\n\texcludesFile = {global_excludes.as_posix()}\n",
                encoding="utf-8",
            )
            (root / ".gitignore").write_text("locally-hidden.txt\n", encoding="utf-8")
            (root / "globally-hidden.txt").write_text("must be inventoried\n", encoding="utf-8")
            (root / "locally-hidden.txt").write_text("repository-ignored\n", encoding="utf-8")
            findings = []
            with mock.patch.dict(
                "tools.validate_foundation.os.environ",
                {"HOME": str(home), "XDG_CONFIG_HOME": str(home / "xdg")},
                clear=False,
            ):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(
            [path.name for path in paths],
            [".gitignore", "globally-hidden.txt", "locally-hidden.txt"],
        )
        self.assertEqual(findings, [])

    def test_untracked_nested_gitignore_cannot_hide_repository_files(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "tracked.txt").write_text("tracked\n", encoding="utf-8")
            subprocess.run(
                ["git", "-C", str(root), "add", "tracked.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            hidden = root / "hidden"
            hidden.mkdir()
            (hidden / ".gitignore").write_text("*\n", encoding="utf-8")
            (hidden / "payload").write_text("must remain visible\n", encoding="utf-8")
            findings: list = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual(
            [path.relative_to(root).as_posix() for path in paths],
            ["hidden/.gitignore", "hidden/payload", "tracked.txt"],
        )
        self.assertEqual(findings, [])

    def test_local_core_worktree_cannot_hide_checkout_files(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            external = parent / "redirected-worktree"
            external.mkdir()
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "tracked.txt").write_text("tracked\n", encoding="utf-8")
            subprocess.run(
                ["git", "-C", str(root), "add", "tracked.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "-C", str(root), "config", "core.worktree", str(external)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "untracked.txt").write_text("must remain visible\n", encoding="utf-8")
            findings: list = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual([path.name for path in paths], ["tracked.txt", "untracked.txt"])
        self.assertEqual(findings, [])

    def test_local_ignore_case_cannot_hide_case_collisions(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "name.txt").write_text("tracked\n", encoding="utf-8")
            subprocess.run(
                ["git", "-C", str(root), "add", "name.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "-C", str(root), "config", "core.ignoreCase", "true"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "NAME.txt").write_text("must remain visible\n", encoding="utf-8")
            if (root / "NAME.txt").samefile(root / "name.txt"):
                self.skipTest("case-collision inventory requires a case-sensitive filesystem")
            findings: list = []

            paths = list(iter_repository_files(root, findings))

        self.assertEqual([path.name for path in paths], ["NAME.txt", "name.txt"])
        self.assertEqual(findings, [])

    def test_git_file_count_and_raw_metadata_limits_are_inclusive(self) -> None:
        cases = (
            ("file count", b"H a\0H b\0", {"GATE0_MAXIMUM_REPOSITORY_FILES": 2}, True),
            ("file count plus one", b"H a\0H b\0H c\0", {"GATE0_MAXIMUM_REPOSITORY_FILES": 2}, False),
            ("metadata bytes", b"H a\0H b\0", {"GATE0_MAXIMUM_RAW_PATH_METADATA_BYTES": 8}, True),
            ("metadata bytes plus one", b"H a\0H b\0", {"GATE0_MAXIMUM_RAW_PATH_METADATA_BYTES": 7}, False),
        )
        for name, output, limits, accepted in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                for path in ("a", "b", "c"):
                    (root / path).write_bytes(b"")
                process = _FakePopen(output)
                findings = []
                with (
                    mock.patch.multiple("tools.validate_foundation", **limits),
                    mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process),
                ):
                    paths = list(iter_repository_files(root, findings))
                self.assertEqual(bool(paths), accepted)
                self.assertEqual(bool(findings), not accepted)
                self.assertEqual(process.kill_count, 0 if accepted else 1)
                self.assertEqual(process.wait_count, 1)

    def test_nonzero_git_exit_uses_bounded_filesystem_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "fallback.txt").write_text("fallback\n", encoding="utf-8")
            process = _FakePopen(b"", return_code=1)
            findings = []
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(root, findings))
            self.assertEqual([path.name for path in paths], ["fallback.txt"])
            self.assertEqual(findings, [])
            self.assertEqual((process.kill_count, process.wait_count), (0, 1))

    def test_git_failure_is_fatal_when_repository_metadata_is_present(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / ".git").mkdir()
            (root / "must-not-fallback.txt").write_text("data\n", encoding="utf-8")
            process = _FakePopen(b"", return_code=1)
            findings = []
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_git"})

    def test_alternate_index_environment_cannot_redirect_either_git_inventory(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "tracked.txt").write_text("tracked\n", encoding="utf-8")
            subprocess.run(
                ["git", "-C", str(root), "add", "tracked.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            findings = []
            with mock.patch.dict(
                "tools.validate_foundation.os.environ",
                {"GIT_INDEX_FILE": str(parent / "alternate-index")},
                clear=False,
            ):
                paths = list(iter_repository_files(root, findings))
                entries = git_index_entries(root, findings, required=True)
        self.assertEqual([path.name for path in paths], ["tracked.txt"])
        self.assertEqual(entries, [("100644", "tracked.txt")])
        self.assertEqual(findings, [])

    def test_stage_inventory_rejects_a_missing_index_object(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "missing-object.txt").write_bytes(b"worktree content\n")
            subprocess.run(
                [
                    "git",
                    "-C",
                    str(root),
                    "update-index",
                    "--add",
                    "--info-only",
                    "--cacheinfo",
                    "100644,1111111111111111111111111111111111111111,missing-object.txt",
                ],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            findings = []
            entries = git_index_entries(root, findings, required=True)

        self.assertEqual(entries, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_stage"})

    def test_stage_inventory_rejects_a_replaced_object_with_the_wrong_type(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "tracked.txt").write_bytes(b"tracked\n")
            subprocess.run(
                ["git", "-C", str(root), "add", "tracked.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            blob = subprocess.run(
                ["git", "-C", str(root), "rev-parse", ":tracked.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
            ).stdout.strip()
            tree = subprocess.run(
                ["git", "-C", str(root), "write-tree"],
                check=True,
                env=clean_environment,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
            ).stdout.strip()
            subprocess.run(
                ["git", "-C", str(root), "update-index", "--cacheinfo", f"100644,{tree},tracked.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "-C", str(root), "update-ref", f"refs/replace/{tree}", blob],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            findings = []
            entries = git_index_entries(root, findings, required=True)

        self.assertEqual(entries, [])
        self.assertEqual({finding.code for finding in findings}, {"git.index_object_type"})

    def test_stage_inventory_failure_after_git_file_inventory_is_fatal(self) -> None:
        file_process = _FakePopen(b"H file.txt\0")
        stage_process = _FakePopen(b"", return_code=1)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "file.txt").write_bytes(b"")
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                side_effect=(file_process, stage_process),
            ):
                validator = FoundationValidator(root)
            with mock.patch.object(validator, "_load_and_validate_policy") as load_policy:
                findings = validator.run()
        load_policy.assert_not_called()
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_stage"})

    def test_stage_inventory_must_be_a_subset_of_the_file_inventory(self) -> None:
        file_process = _FakePopen(b"H file.txt\0")
        metadata = b"100644 " + (b"a" * 40) + b" 0 0"
        stage_process = _FakePopen(metadata + b"\tother.txt\0")
        type_process = _FakePopen((b"a" * 40) + b" blob\0")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "file.txt").write_bytes(b"")
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                side_effect=(file_process, stage_process, type_process),
            ):
                validator = FoundationValidator(root)

        self.assertEqual(validator.repository_files, [root / "file.txt"])
        self.assertEqual(
            [(finding.code, finding.path) for finding in validator.findings],
            [("resource.inventory_protocol", "other.txt")],
        )

    def test_git_file_inventory_rejects_untracked_admitted_paths(self) -> None:
        file_process = _FakePopen(b"? untracked.txt\0")
        stage_process = _FakePopen(b"")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "untracked.txt").write_bytes(b"")
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                side_effect=(file_process, stage_process),
            ):
                validator = FoundationValidator(root)

        self.assertEqual(
            [(finding.code, finding.path) for finding in validator.findings],
            [("git.untracked", "untracked.txt")],
        )

    def test_git_inventory_rejects_hidden_index_state(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "assumed.txt").write_bytes(b"tracked\n")
            (root / "skipped-intent.txt").write_bytes(b"intent\n")
            subprocess.run(
                ["git", "-C", str(root), "add", "assumed.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "-C", str(root), "add", "-N", "skipped-intent.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "-C", str(root), "update-index", "--assume-unchanged", "assumed.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            subprocess.run(
                ["git", "-C", str(root), "update-index", "--skip-worktree", "skipped-intent.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            validator = FoundationValidator(root)

        self.assertEqual({finding.code for finding in validator.findings}, {"git.index_flags"})

    def test_intent_to_add_entries_are_rejected_including_empty_files(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "empty-intent.txt").write_bytes(b"")
            (root / "nonempty-intent.txt").write_bytes(b"content\n")
            subprocess.run(
                ["git", "-C", str(root), "add", "-N", "empty-intent.txt", "nonempty-intent.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            validator = FoundationValidator(root)

        self.assertEqual(
            [(finding.code, finding.path) for finding in validator.findings],
            [("git.intent_to_add", "empty-intent.txt")],
        )

    def test_modified_staged_empty_file_is_not_intent_to_add(self) -> None:
        clean_environment = {
            key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(
                ["git", "init", "--quiet", str(root)],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            for name in ("staged-empty.txt", "modified-empty.txt"):
                (root / name).write_bytes(b"")
            subprocess.run(
                ["git", "-C", str(root), "add", "staged-empty.txt", "modified-empty.txt"],
                check=True,
                env=clean_environment,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            (root / "modified-empty.txt").write_bytes(b"worktree change\n")
            validator = FoundationValidator(root)

        self.assertEqual(validator.findings, [])

    def test_intent_to_add_inventory_failure_is_fatal(self) -> None:
        file_process = _FakePopen(b"H file.txt\0")
        metadata = b"100644 " + (b"a" * 40) + b" 0 0"
        stage_process = _FakePopen(metadata + b"\tfile.txt\0")
        type_process = _FakePopen((b"a" * 40) + b" blob\0")
        intent_process = _FakePopen(b"", return_code=1)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "file.txt").write_bytes(b"")
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                side_effect=(file_process, stage_process, type_process, intent_process),
            ):
                validator = FoundationValidator(root)

        self.assertEqual(
            {finding.code for finding in validator.findings},
            {"resource.inventory_intent"},
        )

    def test_fallback_counts_ignored_entry_but_prunes_its_contents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ignored = root / ".agents"
            ignored.mkdir()
            for index in range(8):
                (ignored / f"ignored-{index}").write_text("ignored\n", encoding="utf-8")
            (root / "kept.txt").write_text("kept\n", encoding="utf-8")
            findings = []
            with (
                mock.patch("tools.validate_foundation.GATE0_MAXIMUM_FALLBACK_DIRECTORY_ENTRIES", 2),
                mock.patch("tools.validate_foundation.subprocess.Popen", side_effect=OSError("no git")),
            ):
                paths = list(iter_repository_files(root, findings))
            self.assertEqual([path.name for path in paths], ["kept.txt"])
            self.assertEqual(findings, [])

            findings = []
            with (
                mock.patch("tools.validate_foundation.GATE0_MAXIMUM_FALLBACK_DIRECTORY_ENTRIES", 1),
                mock.patch("tools.validate_foundation.subprocess.Popen", side_effect=OSError("no git")),
            ):
                paths = list(iter_repository_files(root, findings))
            self.assertEqual(paths, [])
            self.assertEqual({finding.code for finding in findings}, {"resource.inventory_entries"})

    def test_fallback_only_prunes_service_directories_at_the_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            nested = root / "nested" / ".agents"
            nested.mkdir(parents=True)
            hidden = nested / "repository-content.txt"
            hidden.write_text("content\n", encoding="utf-8")
            findings = []
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                side_effect=OSError("no git"),
            ):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [hidden])
        self.assertEqual(findings, [])

    def test_fallback_rejects_a_host_without_descriptor_scandir(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "present.txt").write_text("present\n", encoding="utf-8")
            findings = []
            with mock.patch(
                "tools.validate_foundation._secure_repository_discovery_supported",
                return_value=False,
            ), mock.patch(
                "tools.validate_foundation.os.scandir",
                side_effect=AssertionError("unsupported descriptor scan was attempted"),
            ):
                paths = _fallback_repository_files(root, findings)

        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.unsupported_host"},
        )

    @unittest.skipUnless(hasattr(os, "symlink"), "symlinks are unavailable")
    def test_fallback_collects_but_does_not_descend_into_symlinked_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "root"
            outside = parent / "outside"
            root.mkdir()
            outside.mkdir()
            for index in range(8):
                (outside / f"outside-{index}").write_text("outside\n", encoding="utf-8")
            (root / "link").symlink_to(outside, target_is_directory=True)
            findings = []
            with (
                mock.patch("tools.validate_foundation.GATE0_MAXIMUM_FALLBACK_DIRECTORY_ENTRIES", 1),
                mock.patch("tools.validate_foundation.subprocess.Popen", side_effect=OSError("no git")),
            ):
                paths = list(iter_repository_files(root, findings))
        self.assertEqual([path.name for path in paths], ["link"])
        self.assertEqual(findings, [])

    @unittest.skipUnless(
        os.name == "posix" and hasattr(os, "symlink"),
        "component-relative no-follow discovery requires POSIX symlinks",
    )
    def test_fallback_reopen_does_not_follow_a_swapped_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "root"
            nested = root / "nested"
            nested.mkdir(parents=True)
            (nested / "inside.txt").write_text("inside\n", encoding="utf-8")
            outside = parent / "outside"
            outside.mkdir()
            (outside / "outside.txt").write_text("outside\n", encoding="utf-8")
            saved = root / "saved"
            raw_root = os.fsencode(root)
            original_open = os.open
            root_open_count = 0

            def open_then_swap(
                path: str | bytes | os.PathLike[str] | os.PathLike[bytes],
                flags: int,
                mode: int = 0o600,
                *,
                dir_fd: int | None = None,
            ) -> int:
                nonlocal root_open_count
                if dir_fd is None and os.fsencode(path) == raw_root:
                    root_open_count += 1
                    if root_open_count == 2:
                        nested.rename(saved)
                        nested.symlink_to(outside, target_is_directory=True)
                return original_open(path, flags, mode, dir_fd=dir_fd)

            findings = []
            with mock.patch(
                "tools.validate_foundation._secure_repository_reads_supported",
                return_value=True,
            ), mock.patch(
                "tools.validate_foundation.os.open",
                side_effect=open_then_swap,
            ):
                paths = _fallback_repository_files(root, findings)

        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_read"},
        )

    def test_unterminated_git_record_is_a_protocol_error_without_fallback(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "must-not-fallback.txt").write_text("data\n", encoding="utf-8")
            process = _FakePopen(b"unterminated")
            findings = []
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(root, findings))
            self.assertEqual(paths, [])
            self.assertEqual({finding.code for finding in findings}, {"resource.inventory_protocol"})
            self.assertTrue(process.stdout.closed)
            self.assertEqual(process.wait_count, 1)

    def test_git_file_inventory_rejects_duplicate_paths(self) -> None:
        process = _FakePopen(b"duplicate.txt\0duplicate.txt\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "duplicate.txt").write_text("data\n", encoding="utf-8")
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_protocol"})

    def test_stage_prefix_limit_is_inclusive_and_path_tabs_are_preserved(self) -> None:
        metadata = b"100644 " + (b"a" * 40) + b" 0 0"
        raw_path = b"tab\tname.txt"
        output = metadata + b"\t" + raw_path + b"\0"
        for adjustment, accepted in ((0, True), (-1, False)):
            with self.subTest(adjustment=adjustment), tempfile.TemporaryDirectory() as directory:
                process = _FakePopen(output)
                findings = []
                with (
                    mock.patch(
                        "tools.validate_foundation.GATE0_MAXIMUM_GIT_STAGE_PREFIX_BYTES",
                        len(metadata) + adjustment,
                    ),
                    mock.patch(
                        "tools.validate_foundation.subprocess.Popen",
                        side_effect=(process, _FakePopen((b"a" * 40) + b" blob\0"))
                        if accepted
                        else (process,),
                    ),
                ):
                    entries = git_index_entries(Path(directory), findings)
                if accepted:
                    self.assertEqual(entries, [("100644", os.fsdecode(raw_path))])
                    self.assertEqual(findings, [])
                else:
                    self.assertEqual(entries, [])
                    self.assertEqual(
                        {finding.code for finding in findings},
                        {"resource.inventory_protocol"},
                    )

    def test_stage_inventory_rejects_a_path_escape_before_decoding(self) -> None:
        metadata = b"100644 " + (b"a" * 40) + b" 0 0"
        process = _FakePopen(metadata + b"\t../outside\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                entries = git_index_entries(Path(directory), findings)
        self.assertEqual(entries, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_protocol"})

    def test_stage_inventory_rejects_unmerged_index_entries(self) -> None:
        metadata = b"100644 " + (b"a" * 40) + b" 2 0"
        process = _FakePopen(metadata + b"\tconflicted.txt\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                entries = git_index_entries(Path(directory), findings)

        self.assertEqual(entries, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_protocol"})

    def test_stage_inventory_rejects_malformed_object_ids_and_duplicate_paths(self) -> None:
        valid_metadata = b"100644 " + (b"a" * 40) + b" 0 0"
        cases = (
            b"100644 not-an-object-id 0\ttracked.txt\0",
            b"\xff00644 " + (b"a" * 40) + b" 0 0\ttracked.txt\0",
            b"100644 " + (b"a" * 40) + b" 0 -1\ttracked.txt\0",
            valid_metadata
            + b"\ttracked.txt\0"
            + valid_metadata
            + b"\ttracked.txt\0",
        )
        for output in cases:
            with self.subTest(output=output), tempfile.TemporaryDirectory() as directory:
                process = _FakePopen(output)
                findings = []
                with mock.patch(
                    "tools.validate_foundation.subprocess.Popen", return_value=process
                ):
                    entries = git_index_entries(Path(directory), findings)

                self.assertEqual(entries, [])
                self.assertEqual(
                    {finding.code for finding in findings},
                    {"resource.inventory_protocol"},
                )

    def test_stage_inventory_rejects_non_utf8_paths_after_bounding(self) -> None:
        metadata = b"100644 " + (b"a" * 40) + b" 0 0"
        process = _FakePopen(metadata + b"\tinvalid-\xff\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                entries = git_index_entries(Path(directory), findings)

        self.assertEqual(entries, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_encoding"},
        )

    @unittest.skipUnless(os.name == "posix", "non-UTF-8 paths require POSIX")
    def test_git_file_inventory_rejects_non_utf8_paths_after_bounding(self) -> None:
        raw_paths = (b"z\xff", b"a\xfe")
        process = _FakePopen(b"\0".join(b"H " + path for path in reversed(raw_paths)) + b"\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            raw_root = os.fsencode(directory)
            for raw_path in raw_paths:
                descriptor = os.open(raw_root + b"/" + raw_path, os.O_CREAT | os.O_WRONLY, 0o600)
                os.close(descriptor)
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(Path(directory), findings))
        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_encoding"},
        )

    @unittest.skipUnless(os.name == "posix", "non-UTF-8 paths require POSIX")
    def test_filesystem_fallback_rejects_non_utf8_paths_after_bounding(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            raw_path = os.fsencode(directory) + b"/invalid-\xff"
            descriptor = os.open(raw_path, os.O_CREAT | os.O_WRONLY, 0o600)
            os.close(descriptor)
            findings = []
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen",
                side_effect=OSError("no git"),
            ):
                paths = list(iter_repository_files(Path(directory), findings))

        self.assertEqual(paths, [])
        self.assertEqual(
            {finding.code for finding in findings},
            {"resource.inventory_encoding"},
        )

    @unittest.skipUnless(hasattr(os, "mkfifo"), "FIFOs are unavailable")
    def test_git_inventory_retains_nonregular_entries_for_preflight_rejection(self) -> None:
        process = _FakePopen(b"H input.pipe\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fifo = root / "input.pipe"
            os.mkfifo(fifo)
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(root, findings))

        self.assertEqual(paths, [fifo])
        self.assertEqual(findings, [])

    def test_successful_git_inventory_rejects_tracked_deletions_after_bounding(self) -> None:
        process = _FakePopen(b"H deleted.txt\0H present.txt\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "present.txt").write_text("present\n", encoding="utf-8")
            with mock.patch(
                "tools.validate_foundation.subprocess.Popen", return_value=process
            ), mock.patch.object(
                Path, "exists", side_effect=AssertionError("inventory path was statted")
            ), mock.patch.object(
                Path, "is_symlink", side_effect=AssertionError("inventory path was statted")
            ):
                paths = list(iter_repository_files(root, findings))
        self.assertEqual(paths, [])
        self.assertEqual(
            [(finding.code, finding.path) for finding in findings],
            [("resource.inventory_missing", "deleted.txt")],
        )

    def test_inventory_finding_stops_run_before_policy_parsing(self) -> None:
        process = _FakePopen(b"unterminated")
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                validator = FoundationValidator(Path(directory))
            with mock.patch.object(validator, "_load_and_validate_policy") as load_policy:
                findings = validator.run()
        load_policy.assert_not_called()
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_protocol"})


class MarkdownTests(unittest.TestCase):
    def test_inline_links_reuse_their_physical_line_boundary(self) -> None:
        class CountingText(str):
            line_break_searches = 0

            def find(
                self,
                substring: str,
                start: int | None = None,
                end: int | None = None,
            ) -> int:
                if substring in {"\r", "\n"}:
                    self.line_break_searches += 1
                begin = 0 if start is None else start
                if end is None:
                    return super().find(substring, begin)
                return super().find(substring, begin, end)

        text = CountingText(" ".join("[label](target.md)" for _ in range(64)))

        self.assertEqual(list(markdown_inline_link_targets(text)), ["target.md"] * 64)
        self.assertEqual(text.line_break_searches, 2)

    def test_inline_links_reuse_their_physical_line_delimiter_lookahead(self) -> None:
        class CountingText(str):
            delimiter_searches = 0

            def find(
                self,
                substring: str,
                start: int | None = None,
                end: int | None = None,
            ) -> int:
                if substring in {">", '"', "'"}:
                    self.delimiter_searches += 1
                begin = 0 if start is None else start
                if end is None:
                    return super().find(substring, begin)
                return super().find(substring, begin, end)

            def rfind(
                self,
                substring: str,
                start: int | None = None,
                end: int | None = None,
            ) -> int:
                if substring in {">", '"', "'"}:
                    self.delimiter_searches += 1
                begin = 0 if start is None else start
                if end is None:
                    return super().rfind(substring, begin)
                return super().rfind(substring, begin, end)

        text = CountingText(" ".join("[label](<target.md)" for _ in range(64)))

        self.assertEqual(list(markdown_inline_link_targets(text)), ["<target.md"] * 64)
        self.assertEqual(text.delimiter_searches, 3)

    def test_html_comment_scan_is_fence_aware_and_sentinel_free(self) -> None:
        self.assertIsNone(
            markdown_html_comment_error("OPEN_COMMENT_SENTINEL CLOSE_COMMENT_SENTINEL\n")
        )
        self.assertIsNone(markdown_html_comment_error("<!-- balanced -->\n"))
        self.assertIsNone(markdown_html_comment_error("```text\n<!-- fenced\n```\n"))
        self.assertIsNone(markdown_html_comment_error("`<!--` and ``-->``\n"))
        self.assertEqual(
            markdown_html_comment_error("`<!--` -->\n"),
            "HTML comment closer without opener",
        )
        self.assertEqual(
            markdown_html_comment_error("<!-- outer <!-- nested -->\n"),
            "nested HTML comment opener",
        )
        self.assertEqual(
            markdown_html_comment_error("stray -->\n"),
            "HTML comment closer without opener",
        )

    def test_heading_anchors_match_github_style_duplicates(self) -> None:
        anchors = markdown_anchors("# One heading\n\n## Repeated\n\n## Repeated\n\n## Use `orange`\n")
        self.assertEqual(anchors, {"one-heading", "repeated", "repeated-1", "use-orange"})

    def test_repeated_fragment_links_scan_their_target_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text(
                "[one](target.md#heading)\n[two](target.md#heading)\n",
                encoding="utf-8",
            )
            (root / "target.md").write_text("# Heading\n", encoding="utf-8")
            validator = FoundationValidator(root)

            with mock.patch(
                "tools.validate_foundation.markdown_anchors",
                wraps=markdown_anchors,
            ) as anchors:
                validator._validate_markdown_links()

            self.assertEqual(validator.findings, [])
            self.assertEqual(anchors.call_count, 1)

    def test_missing_relative_link_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text("# Source\n\n[missing](absent.md)\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertIn("markdown.link_missing", {finding.code for finding in validator.findings})

    def test_balanced_inline_link_destinations_are_scanned_completely(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "(guide).md").write_text("# Guide\n", encoding="utf-8")
            (root / "image.png").write_bytes(b"image")
            (root / "target.md").write_text("# Target\n", encoding="utf-8")
            (root / "source.md").write_text(
                "[balanced]((guide).md)\n"
                "[outer [inner]]((guide).md)\n"
                "[escaped \\] label]((guide).md)\n"
                "[multiline\nlabel]((guide).md)\n"
                "[blank\n\nlabel](blank-ignored.md)\n"
                "[![image](image.png)](outer-missing.md)\n"
                "[continued destination](\n  continued-inline-missing.md)\n"
                "[continued title](continued-title-missing.md\n  \"title\")\n"
                "[continued close](continued-close-missing.md\n)\n"
                "[split destination](split-ignored.md\npath)\n"
                "[angle](<(guide).md>)\n"
                "[escaped](\\(guide\\).md)\n"
                "[query](target.md?value=(nested))\n"
                "[quoted title](target.md \"see (this)\")\n"
                "[parenthesized title](target.md (see this))\n"
                "[missing](missing(part).md)\n"
                "[missing [nested]](nested-missing.md)\n"
                "[missing \\] label](escaped-missing.md)\n"
                "[missing\nmultiline](multiline-missing.md)\n"
                "[unterminated title](missing-title.md \"unterminated)\n"
                "[unterminated](ignored.md\r[after cr](cr-missing.md)\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)

            validator._validate_markdown_links()

        self.assertEqual(
            [(finding.code, finding.message) for finding in validator.findings],
            [
                (
                    "markdown.link_missing",
                    "local link target does not exist: outer-missing.md",
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: continued-inline-missing.md",
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: missing(part).md",
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: nested-missing.md",
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: escaped-missing.md",
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: multiline-missing.md",
                ),
                (
                    "markdown.link_missing",
                    'local link target does not exist: missing-title.md "unterminated',
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: cr-missing.md",
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: continued-title-missing.md",
                ),
                (
                    "markdown.link_missing",
                    "local link target does not exist: continued-close-missing.md",
                ),
            ],
        )

    def test_links_in_comments_and_fences_are_not_live_navigation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text(
                "<!-- [commented](commented.md) -->\n\n"
                "```markdown\n[fenced](fenced.md)\n```\n\n"
                "[visible](visible.md)\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)

            validator._validate_markdown_links()

            self.assertEqual(
                [(finding.code, finding.message) for finding in validator.findings],
                [("markdown.link_missing", "local link target does not exist: visible.md")],
            )

    def test_links_in_inline_code_are_not_live_navigation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text(
                "`[single](single.md)` and ``code ` [double](double.md) ` code``\n"
                "unmatched ` [visible](visible.md)\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertEqual(
                [(finding.code, finding.message) for finding in validator.findings],
                [("markdown.link_missing", "local link target does not exist: visible.md")],
            )

    def test_reference_style_link_destinations_are_validated(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text(
                "[guide][docs]\n\n"
                "[docs]: missing.md\n"
                "[escaped\\]]: escaped-missing.md\n"
                "[\nmultiline\n]: multiline-missing.md\n"
                "[continued]:\n  continued-missing.md\n\n"
                "[invalid[label]: unescaped-ignored.md\n"
                "[   ]: whitespace-ignored.md\n"
                f"[{'x' * 999}]: maximum-missing.md\n"
                f"[{'x' * 1_000}]: oversized-ignored.md\n"
                "`[ignored]: ignored.md`\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertEqual(
                [(finding.code, finding.message) for finding in validator.findings],
                [
                    ("markdown.link_missing", "local link target does not exist: missing.md"),
                    (
                        "markdown.link_missing",
                        "local link target does not exist: escaped-missing.md",
                    ),
                    (
                        "markdown.link_missing",
                        "local link target does not exist: multiline-missing.md",
                    ),
                    (
                        "markdown.link_missing",
                        "local link target does not exist: continued-missing.md",
                    ),
                    (
                        "markdown.link_missing",
                        "local link target does not exist: maximum-missing.md",
                    ),
                ],
            )

    def test_malformed_link_destinations_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "target.md").write_text("# Target\n", encoding="utf-8")
            (root / "with space.md").write_text("# Space\n", encoding="utf-8")
            (root / "source.md").write_text(
                "[percent](https://example.com/%GG)\n"
                "[port](https://example.com:port/source)\n"
                "[fragment](https://example.com/#first#second)\n"
                "[local query](target.md?value=%GG)\n"
                "[local fragment](target.md#first#second)\n"
                "[local brackets](target[1].md)\n"
                "[valid](https://example.com/a%20path)\n"
                "[valid local](target.md?value=%20)\n"
                "[valid angle](<with space.md>)\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_markdown_links()

        self.assertEqual(
            [finding.code for finding in validator.findings],
            ["markdown.link_invalid"] * 6,
        )

    def test_existing_cross_file_anchor_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text("# Source\n\n[target](target.md#exact-heading)\n", encoding="utf-8")
            (root / "target.md").write_text("# Exact heading\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertEqual(validator.findings, [])

    @unittest.skipUnless(hasattr(os, "symlink"), "symlinks are unavailable")
    def test_ignored_symlink_target_is_not_treated_as_checkout_content(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "repository"
            root.mkdir()
            source = root / "source.md"
            source.write_text("# Source\n\n[ignored](ignored.md)\n", encoding="utf-8")
            outside = parent / "outside.md"
            outside.write_text("# Outside\n", encoding="utf-8")
            (root / "ignored.md").symlink_to(outside)
            validator = FoundationValidator(root)
            validator.repository_files = [source]

            with mock.patch.object(
                Path, "exists", side_effect=AssertionError("link target was statted")
            ), mock.patch.object(
                Path, "is_file", side_effect=AssertionError("link target was statted")
            ):
                validator._validate_markdown_links()

            self.assertIn(
                "markdown.link_missing", {finding.code for finding in validator.findings}
            )

    def test_directory_link_is_derived_from_inventory_without_stat(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source.md"
            source.write_text("# Source\n\n[docs](docs/)\n", encoding="utf-8")
            docs = root / "docs"
            docs.mkdir()
            target = docs / "target.md"
            target.write_text("# Target\n", encoding="utf-8")
            validator = FoundationValidator(root)

            with mock.patch.object(
                Path, "exists", side_effect=AssertionError("link target was statted")
            ), mock.patch.object(
                Path, "is_file", side_effect=AssertionError("link target was statted")
            ):
                validator._validate_markdown_links()

            self.assertEqual(validator.findings, [])

    def test_malformed_uri_targets_are_reported_without_a_parser_crash(self) -> None:
        for target in ("https://[", "//[", "<missing"):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "source.md").write_text(
                    f"# Source\n\n[malformed]({target})\n",
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)

                validator._validate_markdown_links()

                self.assertEqual(
                    [(finding.code, finding.message) for finding in validator.findings],
                    [("markdown.link_invalid", "link target is not a valid URI reference")],
                )

    def test_local_links_reject_malformed_or_non_utf8_percent_encoding(self) -> None:
        for target in ("percent%.md", "percent%GG.md", "%FF.md"):
            with self.subTest(target=target), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "source.md").write_text(f"[target]({target})\n", encoding="utf-8")
                validator = FoundationValidator(root)

                validator._validate_markdown_links()

                self.assertEqual(
                    [(finding.code, finding.message) for finding in validator.findings],
                    [
                        (
                            "markdown.link_invalid",
                            "link target has invalid percent encoding"
                            if target == "%FF.md"
                            else "link target is not a valid URI reference",
                        )
                    ],
                )

    def test_local_links_accept_strict_utf8_percent_encoding(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text("[target](%C3%A9.md)\n", encoding="utf-8")
            (root / "é.md").write_text("# Target\n", encoding="utf-8")
            validator = FoundationValidator(root)

            validator._validate_markdown_links()

            self.assertEqual(validator.findings, [])

    def test_valid_network_uri_targets_still_pass(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text(
                "# Source\n\n[IPv6](https://[2001:db8::1]/path)\n\n"
                "[protocol relative](//example.test/path)\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)

            validator._validate_markdown_links()

            self.assertEqual(validator.findings, [])

    def test_unclosed_fence_is_rejected(self) -> None:
        self.assertEqual(
            markdown_fence_error("# Document\n\n```text\nnot closed\n"),
            "unclosed ``` fence opened on line 3",
        )
        self.assertIsNone(markdown_fence_error("```invalid`info\n"))

    def test_invalid_backtick_fence_cannot_hide_a_link(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text(
                "```invalid`info\n[visible](visible.md)\n```\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertIn("markdown.link_missing", {finding.code for finding in validator.findings})

    def test_longer_closing_fence_is_valid(self) -> None:
        self.assertIsNone(markdown_fence_error("~~~text\ncontent\n~~~~\n"))

    @staticmethod
    def _orange_book_text(
        *,
        byline: str = "By Chase Bryan",
        chapter_words: int = 1_200,
        chapter_two_words: int = 1_200,
    ) -> str:
        chapter = " ".join("evidence" for _ in range(chapter_words))
        chapter_two = " ".join("claims" for _ in range(chapter_two_words))
        return f"""# The Orange Book

{byline}

Status: living pre-alpha reader guide

Snapshot: 2026-07-12

Manuscript version: 0.2

This is not a normative language specification.

## Contents

- [Preface](#preface)
- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)
- [Chapter 2: Claims, Not Labels](#chapter-2-claims-not-labels)
- [Manuscript map](#manuscript-map)
- [Sources and drafting disclosure](#sources-and-drafting-disclosure)

## Preface

Reader context.

## Chapter 1: The Seams Are the System

{chapter}

## Chapter 2: Claims, Not Labels

{chapter_two}

## Manuscript map

Future chapters.

## Sources and drafting disclosure

Drafted with OpenAI Codex, based on GPT-5. Chase Bryan is the named author.

Manuscript version 0.2 added Chapter 2, drafted with OpenAI Codex, based on
GPT-5, under Chase Bryan's direction on 2026-07-14.
"""

    def test_orange_book_contract_accepts_v02_structure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            path.write_text(self._orange_book_text(), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual(validator.findings, [])

    def test_orange_book_contract_rejects_wrong_byline_and_short_chapter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            path.write_text(
                self._orange_book_text(byline="By Someone Else", chapter_words=20),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual(
                {finding.code for finding in validator.findings},
                {"book.chapter_length", "book.identity"},
            )

    def test_orange_book_contract_rejects_short_second_chapter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            path.write_text(self._orange_book_text(chapter_two_words=20), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.chapter_length"})

    def test_orange_book_contract_rejects_reordered_contents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            text = self._orange_book_text().replace(
                "- [Preface](#preface)\n"
                "- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)",
                "- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)\n"
                "- [Preface](#preface)",
            )
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.navigation"})

    def test_orange_book_contract_rejects_hidden_chapter_and_disclosure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            hidden_words = " ".join("evidence" for _ in range(1_200))
            text = self._orange_book_text(chapter_words=20).replace(
                "\n## Manuscript map",
                f"\n<!-- {hidden_words} -->\n\n## Manuscript map",
            ).replace(
                "Drafted with OpenAI Codex, based on GPT-5. Chase Bryan is the named author.",
                "<!-- OpenAI Codex GPT-5 Chase Bryan is the named author -->",
            )
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual(
                {finding.code for finding in validator.findings},
                {"book.chapter_length", "book.disclosure"},
            )

    def test_orange_book_contract_rejects_missing_v02_disclosure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            text = self._orange_book_text().replace(
                "\nManuscript version 0.2 added Chapter 2, drafted with OpenAI Codex, based on\n"
                "GPT-5, under Chase Bryan's direction on 2026-07-14.\n",
                "\n",
                1,
            )
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.disclosure"})

    def test_orange_book_contract_rejects_impossible_snapshot_date(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            text = self._orange_book_text().replace("Snapshot: 2026-07-12", "Snapshot: 2026-99-99")
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.snapshot"})

    def test_orange_book_contract_rejects_missing_wrong_or_duplicate_version(self) -> None:
        mutations = (
            lambda text: text.replace("Manuscript version: 0.2\n\n", "", 1),
            lambda text: text.replace("Manuscript version: 0.2", "Manuscript version: 0.3", 1),
            lambda text: text.replace(
                "Manuscript version: 0.2",
                "Manuscript version: 0.2\n\nManuscript version: 0.2",
                1,
            ),
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = root / "docs/THE_ORANGE_BOOK.md"
                path.parent.mkdir(parents=True)
                path.write_text(mutate(self._orange_book_text()), encoding="utf-8")
                validator = FoundationValidator(root)
                validator._validate_orange_book()
                self.assertEqual({finding.code for finding in validator.findings}, {"book.version"})

    def test_orange_book_contract_rejects_competing_byline(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "docs/THE_ORANGE_BOOK.md"
            path.parent.mkdir(parents=True)
            text = self._orange_book_text().replace(
                "By Chase Bryan\n",
                "By Chase Bryan\n\nBy Someone Else\n",
                1,
            )
            path.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_orange_book()
            self.assertEqual({finding.code for finding in validator.findings}, {"book.identity"})


class WorkflowTests(unittest.TestCase):
    def test_job_blocks_are_found_without_parsing_untrusted_yaml(self) -> None:
        jobs = workflow_jobs(
            [
                "name: Test",
                "jobs:",
                "  first:",
                "    timeout-minutes: 5",
                "  second:",
                "    timeout-minutes: 7",
            ]
        )
        self.assertEqual([name for name, _ in jobs], ["first", "second"])

    def test_untrusted_context_interpolation_in_run_is_rejected(self) -> None:
        for field in (
            "github.event.inputs.command",
            "github.event.issue.title",
            "github.base_ref",
            "github.head_ref",
            "github.ref",
            "github.ref_name",
            "inputs.command",
        ):
            lines = [
                "      - name: Unsafe",
                "        run: |",
                f"          printf '%s' '${{{{ {field} }}}}'",
            ]
            with self.subTest(field=field):
                self.assertEqual(unsafe_run_interpolations(lines), [2])

        safe = ["        run: printf '%s' '${{ github.sha }}'"]
        self.assertEqual(unsafe_run_interpolations(safe), [])

    def test_mutable_action_ref_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflow_dir = root / ".github/workflows"
            workflow_dir.mkdir(parents=True)
            (workflow_dir / "ci.yml").write_text(
                """name: CI
on:
  pull_request:
  push:
  merge_group:
permissions: {}
concurrency:
  group: ci
  cancel-in-progress: true
jobs:
  check:
    timeout-minutes: 5
    permissions: {}
    steps:
      - uses: actions/checkout@v7 # v7.0.0
        with:
          persist-credentials: false
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = {
                "required_workflows": ["ci.yml"],
                "github_actions": {
                    "allowed_action_repositories": ["actions/checkout"],
                    "allowed_write_permissions": {},
                    "forbidden_events": ["pull_request_target"],
                    "require_full_commit_sha": True,
                    "require_version_comment": True,
                },
            }
            validator._validate_workflows()
            self.assertIn("workflow.mutable_action", {finding.code for finding in validator.findings})


class ProvisionalSchemaTests(unittest.TestCase):
    def setUp(self) -> None:
        self.schema_path = Path("/virtual/record.schema.json")
        self.schema = {
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$id": "urn:orange:gate0:test:v0.1",
            "type": "object",
            "required": ["id", "status"],
            "properties": {
                "id": {"type": "string", "pattern": "^[a-z]+-[0-9]+$"},
                "status": {"enum": ["open", "closed"]},
            },
            "additionalProperties": False,
        }

    def validate(self, instance: object):
        return validate_schema_instance(
            instance,
            self.schema,
            self.schema_path,
            {self.schema_path: self.schema},
            {self.schema["$id"]: (self.schema_path, self.schema)},
        )

    def test_valid_instance_passes(self) -> None:
        self.assertEqual(self.validate({"id": "claim-1", "status": "open"}), [])

    def test_invalid_instance_fails_on_stable_keyword_and_path(self) -> None:
        issues = self.validate({"id": "INVALID", "status": "open"})
        self.assertIn(("pattern", "$/id"), {(issue.keyword, issue.instance_path) for issue in issues})

    def test_unknown_property_fails_closed(self) -> None:
        issues = self.validate({"id": "claim-1", "status": "open", "surprise": True})
        self.assertIn(
            ("additionalProperties", "$/surprise"),
            {(issue.keyword, issue.instance_path) for issue in issues},
        )

    def test_unsupported_schema_keyword_is_detected(self) -> None:
        schema = dict(self.schema)
        schema["if"] = {"required": ["x"]}
        findings = audit_schema_vocabulary(schema)
        self.assertTrue(any("unsupported keyword 'if'" in finding for finding in findings))

    def test_invalid_pattern_is_audited_and_direct_validation_fails_closed(self) -> None:
        schema = {"type": "string", "pattern": "["}
        self.assertEqual(
            audit_schema_vocabulary(schema),
            ["invalid pattern expression '[' at $"],
        )

        issues = validate_schema_instance(
            "value",
            schema,
            self.schema_path,
            {self.schema_path: schema},
            {},
        )
        self.assertEqual(
            [(issue.keyword, issue.instance_path, issue.message) for issue in issues],
            [("pattern", "$", "schema pattern expression is invalid")],
        )

    def test_relative_schema_reference_resolution_is_lexical(self) -> None:
        schema_path = Path("/virtual/root.schema.json")
        referenced_path = Path("/virtual/referenced.schema.json")
        schema = {"$ref": "referenced.schema.json"}
        referenced = {"type": "string"}

        with mock.patch.object(
            Path, "resolve", side_effect=AssertionError("schema path was resolved")
        ):
            issues = validate_schema_instance(
                "value",
                schema,
                schema_path,
                {schema_path: schema, referenced_path: referenced},
                {},
            )

        self.assertEqual(issues, [])

        absolute = {"$ref": referenced_path.as_posix()}
        issues = validate_schema_instance(
            "value",
            absolute,
            schema_path,
            {schema_path: absolute, referenced_path: referenced},
            {},
        )
        self.assertEqual([issue.keyword for issue in issues], ["$ref"])

    def test_schema_reference_fragments_decode_strict_json_pointers(self) -> None:
        schema_path = Path("/virtual/root.schema.json")
        valid = {"$defs": {"a/b": {"const": 1}}, "$ref": "#/$defs/a%7E1b"}
        self.assertEqual(validate_schema_instance(1, valid, schema_path, {schema_path: valid}, {}), [])

        for fragment in ("%GG", "~2"):
            schema = {"$defs": {fragment: True}, "$ref": f"#/$defs/{fragment}"}
            issues = validate_schema_instance(1, schema, schema_path, {schema_path: schema}, {})
            self.assertEqual([issue.keyword for issue in issues], ["$ref"])

    def test_fixture_validation_never_executes_unreviewed_schema_patterns(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        dangerous_pattern = "^(a+)+$"
        mutations = ("pattern", "patternProperties")
        real_compile = re.compile
        real_search = re.search

        def guarded_compile(pattern: object, flags: int = 0):
            if pattern == dangerous_pattern:
                raise AssertionError("unreviewed schema pattern was compiled")
            return real_compile(pattern, flags)

        def guarded_search(pattern: object, string: str, flags: int = 0):
            if pattern == dangerous_pattern:
                raise AssertionError("unreviewed schema pattern was searched")
            return real_search(pattern, string, flags)

        for mutation in mutations:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                shutil.copytree(source_root / "schemas/gate0", root / "schemas/gate0")
                shutil.copytree(
                    source_root / "conformance/foundation",
                    root / "conformance/foundation",
                )
                if mutation == "pattern":
                    schema_path = root / "schemas/gate0/claim-record-v0.1.schema.json"
                    schema = load_json(schema_path)
                    schema["properties"]["claim_id"]["pattern"] = dangerous_pattern
                else:
                    schema_path = root / "schemas/gate0/evidence-manifest-v0.1.schema.json"
                    schema = load_json(schema_path)
                    pattern_properties = schema["$defs"]["replay"]["properties"]["environment"]["properties"][
                        "additional_variables"
                    ]["patternProperties"]
                    child_schema = pattern_properties.pop(next(iter(pattern_properties)))
                    pattern_properties[dangerous_pattern] = child_schema
                schema_path.write_text(json.dumps(schema, indent=2) + "\n", encoding="utf-8")

                validator = FoundationValidator(root)
                validator.policy = protected_file_policy()
                with mock.patch(
                    "tools.validate_foundation.re.compile",
                    side_effect=guarded_compile,
                ), mock.patch(
                    "tools.validate_foundation.re.search",
                    side_effect=guarded_search,
                ):
                    validator._validate_schema_fixtures()

                digest_findings = [
                    finding
                    for finding in validator.findings
                    if finding.code == "protected_file.digest"
                    and finding.path == schema_path.relative_to(root).as_posix()
                ]
                self.assertEqual(len(digest_findings), 1)

    def test_schema_validation_uses_the_authenticated_single_read_buffer(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        dangerous_pattern = "^(a+)+$"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(source_root / "schemas/gate0", root / "schemas/gate0")
            shutil.copytree(
                source_root / "conformance/foundation",
                root / "conformance/foundation",
            )
            validator = FoundationValidator(root)
            validator.policy = protected_file_policy()
            validator._validate_protected_file_digests()

            schema_path = root / "schemas/gate0/claim-record-v0.1.schema.json"
            schema = load_json(schema_path)
            schema["properties"]["claim_id"]["pattern"] = dangerous_pattern
            schema_path.write_text(json.dumps(schema, indent=2) + "\n", encoding="utf-8")

            real_compile = re.compile

            def guarded_compile(pattern: object, flags: int = 0):
                if pattern == dangerous_pattern:
                    raise AssertionError("schema path was reopened after authentication")
                return real_compile(pattern, flags)

            with mock.patch(
                "tools.validate_foundation.re.compile",
                side_effect=guarded_compile,
            ):
                validator._validate_schema_fixtures()

            self.assertEqual(validator.findings, [])

    def test_uri_formats_enforce_rfc3986_lexical_form(self) -> None:
        valid_values = (
            ("https://example.com/a%20path?x=%2F#frag", "uri"),
            ("urn:orange:test", "uri"),
            ("", "uri-reference"),
            ("relative/%7e?q=%25", "uri-reference"),
        )
        for value, format_name in valid_values:
            with self.subTest(value=value, format_name=format_name):
                self.assertTrue(valid_format(value, format_name))

        invalid_characters = ("%", "%0", "%GG", "%0G", " ", "\n", "\\", "é", "{")
        for suffix in invalid_characters:
            with self.subTest(suffix=suffix, format_name="uri"):
                self.assertFalse(valid_format(f"https://example.com/{suffix}", "uri"))
            with self.subTest(suffix=suffix, format_name="uri-reference"):
                self.assertFalse(valid_format(f"relative/{suffix}", "uri-reference"))

        self.assertFalse(valid_format("relative/path", "uri"))
        self.assertTrue(valid_format("relative/path", "uri-reference"))
        self.assertTrue(valid_format("https://[2001:db8::1]/source", "uri"))
        self.assertTrue(valid_format("https://user:pass@example.com:443/source", "uri"))
        self.assertTrue(valid_format("https://example.com:/source", "uri"))
        for value in (
            "https://example.com/a[b]",
            "https://example.com/?q=[x]",
            "https://example.com/#a#b",
            "https://example.com:port/source",
            "https://[2001:db8::1]:port/source",
            "https://unbracketed:host:443/source",
            "https://first@second@example.com/source",
            "urn:orange:a[b]",
        ):
            self.assertFalse(valid_format(value, "uri"), value)

    def test_date_time_format_enforces_rfc3339_lexical_form(self) -> None:
        for value in (
            "2026-07-14T12:34:56Z",
            "2026-07-14t12:34:56.123z",
            "2026-07-14T12:34:56-05:30",
        ):
            self.assertTrue(valid_format(value, "date-time"), value)
        for value in (
            "2026-07-14 12:34:56+00:00",
            "2026-07-14T12:34:56+00:00:30",
            "2026-07-14T12:34:56",
            "2026-07-14T12:34:56+24:00",
            "2026-07-14T12:34:60Z",
            "2026-02-30T12:34:56Z",
        ):
            self.assertFalse(valid_format(value, "date-time"), value)

    def test_invalid_uri_is_reported_as_a_schema_format_issue(self) -> None:
        schema = {"type": "string", "format": "uri"}
        issues = validate_schema_instance(
            "https://example.com/%GG",
            schema,
            self.schema_path,
            {self.schema_path: schema},
            {},
        )
        self.assertEqual(
            [(issue.keyword, issue.instance_path) for issue in issues],
            [("format", "$")],
        )

    def test_schema_invalid_cross_record_fields_do_not_crash_fixture_validation(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        mutations = (
            ("non-array basis", "$/basis"),
            ("unhashable assumption reference", "$/basis/0/assumption_ref"),
        )
        for mutation, expected_path in mutations:
            with self.subTest(mutation=mutation), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                shutil.copytree(source_root / "schemas/gate0", root / "schemas/gate0")
                shutil.copytree(
                    source_root / "conformance/foundation",
                    root / "conformance/foundation",
                )
                fixture_path = root / "conformance/foundation/valid/claim-record.json"
                fixture = load_json(fixture_path)
                if mutation == "non-array basis":
                    fixture["basis"] = 7
                else:
                    fixture["basis"][0] = {
                        "basis_id": "G0-BASIS-FOUNDATION-VALIDATION",
                        "type": "assumption",
                        "verification_state": "recorded",
                        "assumption_ref": [],
                    }
                fixture_path.write_text(json.dumps(fixture, indent=2) + "\n", encoding="utf-8")

                validator = FoundationValidator(root)
                validator.policy = protected_file_policy()
                validator._validate_schema_fixtures()

                findings = [
                    finding
                    for finding in validator.findings
                    if finding.code == "fixture.unexpected_invalid"
                    and finding.path == "conformance/foundation/valid/claim-record.json"
                ]
                self.assertEqual(len(findings), 1)
                self.assertTrue(
                    findings[0].message.startswith(f"type at {expected_path}:"),
                    findings[0].message,
                )


if __name__ == "__main__":
    unittest.main()
