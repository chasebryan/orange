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
    GATE0_MAXIMUM_BINARY_FILE_BYTES,
    GATE0_MAXIMUM_JSON_NESTING_DEPTH,
    GATE0_MAXIMUM_REPOSITORY_BYTES,
    GATE0_MAXIMUM_TEXT_FILE_BYTES,
    audit_schema_vocabulary,
    checkout_disables_credentials,
    load_json,
    markdown_anchors,
    markdown_fence_error,
    git_index_entries,
    iter_repository_files,
    unsafe_run_interpolations,
    valid_format,
    validate_schema_instance,
    workflow_jobs,
)


class _FakePipe:
    def __init__(self, data: bytes) -> None:
        self.data = data
        self.offset = 0
        self.closed = False

    def read(self, size: int) -> bytes:
        chunk = self.data[self.offset : self.offset + size]
        self.offset += len(chunk)
        return chunk

    def close(self) -> None:
        self.closed = True


class _FakePopen:
    def __init__(self, data: bytes, return_code: int = 0) -> None:
        self.stdout = _FakePipe(data)
        self.return_code = return_code
        self.kill_count = 0
        self.wait_count = 0

    def kill(self) -> None:
        self.kill_count += 1

    def wait(self) -> int:
        self.wait_count += 1
        return self.return_code


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
    def _sparse_file(path: Path, size: int) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("wb") as destination:
            destination.truncate(size)

    def test_text_and_binary_file_size_boundaries_are_inclusive(self) -> None:
        cases = (
            ("record.txt", GATE0_MAXIMUM_TEXT_FILE_BYTES),
            ("asset.png", GATE0_MAXIMUM_BINARY_FILE_BYTES),
        )
        for name, limit in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._sparse_file(root / name, limit)
                validator = FoundationValidator(root)
                self.assertTrue(validator._preflight_repository_resources())
                snapshot = validator._read_repository_bytes(root / name)
                self.assertIsNotNone(snapshot)
                self.assertEqual(len(snapshot or b""), limit)
                self.assertEqual(validator.findings, [])

    def test_text_and_binary_files_one_byte_over_the_limit_are_rejected(self) -> None:
        cases = (
            ("record.txt", GATE0_MAXIMUM_TEXT_FILE_BYTES),
            ("asset.png", GATE0_MAXIMUM_BINARY_FILE_BYTES),
        )
        for name, limit in cases:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._sparse_file(root / name, limit + 1)
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
                    self._sparse_file(
                        root / f"asset-{index}.png",
                        GATE0_MAXIMUM_BINARY_FILE_BYTES,
                    )
                if extra_bytes:
                    self._sparse_file(root / "extra.txt", extra_bytes)
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


class RepositoryInventoryBoundTests(unittest.TestCase):
    def test_git_path_limit_is_inclusive_and_one_extra_byte_kills_the_producer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "abc").write_bytes(b"")
            accepted = _FakePopen(b"abc\0")
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

            rejected = _FakePopen(b"abcd\0")
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

    def test_git_process_environment_removes_inventory_redirections(self) -> None:
        process = _FakePopen(b"file.txt\0")
        findings = []
        environment = {
            "GIT_DIR": "/redirected/git-dir",
            "git_index_file": "/redirected/index",
            "GIT_OBJECT_DIRECTORY": "/redirected/objects",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES": "/redirected/alternates",
            "GIT_CONFIG_COUNT": "1",
            "GIT_CONFIG_KEY_0": "core.worktree",
            "GIT_CONFIG_VALUE_0": "/redirected/worktree",
            "LC_ALL": "C",
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
        child_environment = popen.call_args.kwargs["env"]
        self.assertEqual(child_environment, {"LC_ALL": "C", "PATH": "/test/bin"})
        self.assertFalse(any(key.upper().startswith("GIT_CONFIG_") for key in child_environment))

    def test_git_file_count_and_raw_metadata_limits_are_inclusive(self) -> None:
        cases = (
            ("file count", b"a\0b\0", {"GATE0_MAXIMUM_REPOSITORY_FILES": 2}, True),
            ("file count plus one", b"a\0b\0c\0", {"GATE0_MAXIMUM_REPOSITORY_FILES": 2}, False),
            ("metadata bytes", b"a\0b\0", {"GATE0_MAXIMUM_RAW_PATH_METADATA_BYTES": 4}, True),
            ("metadata bytes plus one", b"a\0b\0", {"GATE0_MAXIMUM_RAW_PATH_METADATA_BYTES": 3}, False),
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

    def test_stage_inventory_failure_after_git_file_inventory_is_fatal(self) -> None:
        file_process = _FakePopen(b"file.txt\0")
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

    def test_fallback_counts_ignored_entry_but_prunes_its_contents(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ignored = root / ".git"
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

    def test_stage_prefix_limit_is_inclusive_and_path_tabs_are_preserved(self) -> None:
        metadata = b"100644 " + (b"a" * 40) + b" 0"
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
                    mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process),
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
        metadata = b"100644 " + (b"a" * 40) + b" 0"
        process = _FakePopen(metadata + b"\t../outside\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                entries = git_index_entries(Path(directory), findings)
        self.assertEqual(entries, [])
        self.assertEqual({finding.code for finding in findings}, {"resource.inventory_protocol"})

    @unittest.skipUnless(os.name == "posix", "byte-preserving paths require POSIX")
    def test_git_paths_are_sorted_as_bytes_then_decoded_losslessly(self) -> None:
        raw_paths = (b"z\xff", b"a\xfe")
        process = _FakePopen(b"\0".join(reversed(raw_paths)) + b"\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            raw_root = os.fsencode(directory)
            for raw_path in raw_paths:
                descriptor = os.open(raw_root + b"/" + raw_path, os.O_CREAT | os.O_WRONLY, 0o600)
                os.close(descriptor)
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(Path(directory), findings))
        self.assertEqual([os.fsencode(path.name) for path in paths], sorted(raw_paths))
        self.assertEqual(findings, [])

    def test_successful_git_inventory_filters_tracked_deletions_after_bounding(self) -> None:
        process = _FakePopen(b"deleted.txt\0present.txt\0")
        findings = []
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "present.txt").write_text("present\n", encoding="utf-8")
            with mock.patch("tools.validate_foundation.subprocess.Popen", return_value=process):
                paths = list(iter_repository_files(root, findings))
        self.assertEqual([path.name for path in paths], ["present.txt"])
        self.assertEqual(findings, [])

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
    def test_heading_anchors_match_github_style_duplicates(self) -> None:
        anchors = markdown_anchors("# One heading\n\n## Repeated\n\n## Repeated\n")
        self.assertEqual(anchors, {"one-heading", "repeated", "repeated-1"})

    def test_missing_relative_link_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text("# Source\n\n[missing](absent.md)\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertIn("markdown.link_missing", {finding.code for finding in validator.findings})

    def test_existing_cross_file_anchor_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "source.md").write_text("# Source\n\n[target](target.md#exact-heading)\n", encoding="utf-8")
            (root / "target.md").write_text("# Exact heading\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_markdown_links()
            self.assertEqual(validator.findings, [])

    def test_malformed_uri_targets_are_reported_without_a_parser_crash(self) -> None:
        for target in ("https://[", "//["):
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

    def test_longer_closing_fence_is_valid(self) -> None:
        self.assertIsNone(markdown_fence_error("~~~text\ncontent\n~~~~\n"))

    @staticmethod
    def _orange_book_text(*, byline: str = "By Chase Bryan", chapter_words: int = 1_200) -> str:
        chapter = " ".join("evidence" for _ in range(chapter_words))
        return f"""# The Orange Book

{byline}

Status: living pre-alpha reader guide

Snapshot: 2026-07-12

This is not a normative language specification.

## Contents

- [Preface](#preface)
- [Chapter 1: The Seams Are the System](#chapter-1-the-seams-are-the-system)
- [Manuscript map](#manuscript-map)
- [Sources and drafting disclosure](#sources-and-drafting-disclosure)

## Preface

Reader context.

## Chapter 1: The Seams Are the System

{chapter}

## Manuscript map

Future chapters.

## Sources and drafting disclosure

Drafted with OpenAI Codex, based on GPT-5. Chase Bryan is the named author.
"""

    def test_orange_book_contract_accepts_v01_structure(self) -> None:
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
    def test_checkout_credentials_must_be_explicitly_disabled(self) -> None:
        safe = [
            "      - name: Checkout",
            "        uses: actions/checkout@" + "a" * 40 + " # v1.0.0",
            "        with:",
            "          persist-credentials: false",
            "      - name: Next",
        ]
        unsafe = safe[:2] + safe[4:]
        self.assertTrue(checkout_disables_credentials(safe, 1))
        self.assertFalse(checkout_disables_credentials(unsafe, 1))

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

    def test_untrusted_event_interpolation_in_run_is_rejected(self) -> None:
        lines = [
            "      - name: Unsafe",
            "        run: |",
            "          printf '%s' '${{ github.event.issue.title }}'",
        ]
        self.assertEqual(unsafe_run_interpolations(lines), [2])

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
