from __future__ import annotations

import datetime as dt
import hashlib
import io
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import ExitStack, redirect_stderr
from pathlib import Path
from unittest import mock

from tools.validate_foundation import (
    FoundationValidator,
    audit_schema_vocabulary,
    canonical_json_bytes,
    checkout_disables_credentials,
    load_json,
    parse_arguments,
    relative,
    validate_schema_instance,
    workflow_jobs,
    workflow_steps,
)


def workflow_policy() -> dict[str, object]:
    return {
        "required_workflows": ["ci.yml"],
        "github_actions": {
            "allowed_action_repositories": ["actions/checkout"],
            "allowed_container_images": [
                "ghcr.io/ossf/scorecard-action@sha256:"
                + "2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941"
            ],
            "allowed_write_permissions": {},
            "forbidden_events": ["pull_request_target"],
            "require_full_commit_sha": True,
            "require_version_comment": True,
        },
    }


class JsonHardeningTests(unittest.TestCase):
    def test_non_finite_number_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "non-finite.json"
            path.write_text('{"value":NaN}\n', encoding="utf-8")
            with self.assertRaises(json.JSONDecodeError):
                load_json(path)

    def test_floating_point_and_unsafe_integer_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name, source in (
                ("float.json", '{"value":1.0}\n'),
                ("unsafe.json", '{"value":9007199254740992}\n'),
            ):
                path = root / name
                path.write_text(source, encoding="utf-8")
                with self.subTest(name=name), self.assertRaises(json.JSONDecodeError):
                    load_json(path)

    def test_lone_surrogate_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "surrogate.json"
            path.write_text('{"value":"\\ud800"}\n', encoding="utf-8")
            with self.assertRaises(json.JSONDecodeError):
                load_json(path)

    def test_canonical_json_uses_utf16_property_order(self) -> None:
        # RFC 8785 sorts object names by UTF-16 code units, not UTF-8 bytes.
        value = {"\ue000": 1, "\U0001f600": 2, "a": [True, None, "x"]}
        self.assertEqual(
            canonical_json_bytes(value),
            '{"a":[true,null,"x"],"😀":2,"":1}'.encode("utf-8"),
        )

    def test_schema_equality_distinguishes_booleans_from_integers(self) -> None:
        schema_path = Path("/virtual/equality.schema.json")
        for keyword, schema in (("const", {"const": True}), ("enum", {"enum": [True]})):
            with self.subTest(keyword=keyword):
                issues = validate_schema_instance(1, schema, schema_path, {schema_path: schema}, {})
                self.assertIn(keyword, {issue.keyword for issue in issues})


class WorkflowHardeningTests(unittest.TestCase):
    def test_legacy_container_action_policy_field_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        policy = load_json(source_root / "policy/gate0-repository-policy.json")
        policy["github_actions"]["allowed_container_actions"] = list(
            policy["github_actions"]["allowed_container_images"]
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            policy_dir = root / "policy"
            policy_dir.mkdir()
            (policy_dir / "gate0-repository-policy.json").write_text(
                json.dumps(policy, indent=2) + "\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._load_and_validate_policy()
            self.assertIn("policy.action_fields", {finding.code for finding in validator.findings})

    def test_descriptive_action_comment_cannot_bypass_policy(self) -> None:
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
      - uses: attacker/uncharted-action@main # v1.0.0 pinned for audit
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = workflow_policy()
            validator._validate_workflows()
            codes = {finding.code for finding in validator.findings}
            self.assertIn("workflow.action_allowlist", codes)
            self.assertIn("workflow.mutable_action", codes)

    def test_checkout_credentials_must_be_under_with(self) -> None:
        lines = [
            "      - name: Checkout",
            "        uses: actions/checkout@" + "a" * 40 + " # v1.0.0",
            "        env:",
            "          persist-credentials: false",
            "      - name: Next",
        ]
        self.assertFalse(checkout_disables_credentials(lines, 1))

    def test_direct_container_action_syntax_is_rejected(self) -> None:
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
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    permissions: {}
    steps:
      - name: Unsupported container Action
        uses: docker://ghcr.io/ossf/scorecard-action@sha256:2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941 # v2.4.3
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = workflow_policy()
            validator._validate_workflows()
            self.assertIn("workflow.container_action", {finding.code for finding in validator.findings})

    def test_scorecard_uses_hardened_exact_digest_docker_runtime(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        workflow = (source_root / ".github/workflows/scorecard.yml").read_text(encoding="utf-8")
        image = (
            "ghcr.io/ossf/scorecard-action@sha256:"
            "2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941"
        )
        self.assertEqual(workflow.count(image), 1)
        self.assertNotIn("uses: docker://", workflow)
        for required in (
            "set -euo pipefail",
            "printf '::add-mask::%s\\n'",
            "docker run --rm",
            "--read-only",
            "--tmpfs /tmp:rw,noexec,nosuid,nodev,size=1g,mode=1777",
            "--cap-drop=ALL",
            "--cap-add=DAC_OVERRIDE",
            "--security-opt=no-new-privileges=true",
            '--mount "type=bind,source=${GITHUB_EVENT_PATH},target=/github/workflow/event.json,readonly"',
            '--mount "type=bind,source=${GITHUB_WORKSPACE},target=/github/workspace"',
            "--workdir /github/workspace",
            "--env INPUT_PUBLISH_RESULTS=false",
            "--env INPUT_REPO_TOKEN",
            'test -s "$GITHUB_WORKSPACE/results.sarif"',
        ):
            self.assertIn(required, workflow)

    def test_scorecard_runtime_contract_rejects_bypass_mutations(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        workflow = (source_root / ".github/workflows/scorecard.yml").read_text(encoding="utf-8")
        jobs = dict(workflow_jobs(workflow.splitlines()))
        source_steps = dict(workflow_steps(jobs["analysis"]))
        mutations = (
            ("--cap-add=DAC_OVERRIDE", "--cap-add=ALL", "workflow.scorecard_runtime"),
            (",readonly\"", "\"", "workflow.scorecard_contract"),
            ("--env INPUT_PUBLISH_RESULTS=false", "--env INPUT_PUBLISH_RESULTS=true", "workflow.scorecard_publication"),
            ("--pids-limit=256", "--privileged", "workflow.scorecard_runtime"),
        )
        for old, new, expected_code in mutations:
            with self.subTest(mutation=old):
                steps = {name: list(lines) for name, lines in source_steps.items()}
                steps["Run OpenSSF Scorecard"] = [line.replace(old, new) for line in steps["Run OpenSSF Scorecard"]]
                validator = FoundationValidator(Path("/virtual"))
                validator._validate_step_details(Path("scorecard.yml"), "analysis", steps)
                codes = {finding.code for finding in validator.findings}
                self.assertIn("workflow.scorecard_contract", codes)
                self.assertIn(expected_code, codes)

    def test_unadmitted_scorecard_digest_is_rejected(self) -> None:
        digest = "0" * 64
        validator = FoundationValidator(Path("/virtual"))
        validator._validate_step_details(
            Path("scorecard.yml"),
            "analysis",
            {
                "Run OpenSSF Scorecard": [
                    "      - name: Run OpenSSF Scorecard",
                    "        env:",
                    "          INPUT_REPO_TOKEN: ${{ github.token }}",
                    "        run: |",
                    "          docker run --rm \\",
                    f"            ghcr.io/ossf/scorecard-action@sha256:{digest}",
                ],
                "Upload result to code scanning": [
                    "      - name: Upload result to code scanning",
                    "        uses: github/codeql-action/upload-sarif@" + "a" * 40 + " # v4.37.0",
                ],
            },
        )
        self.assertIn("workflow.scorecard_image", {finding.code for finding in validator.findings})

    def test_digest_pinned_scorecard_does_not_request_publication_identity(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        workflow = (source_root / ".github/workflows/scorecard.yml").read_text(encoding="utf-8")
        self.assertIn("--env INPUT_PUBLISH_RESULTS=false", workflow)
        self.assertNotIn("INPUT_PUBLISH_RESULTS=true", workflow)
        self.assertNotIn("id-token: write", workflow)
        self.assertNotIn("INPUT_INTERNAL_PUBLISH_BASE_URL", workflow)
        self.assertNotIn("INPUT_INTERNAL_DEFAULT_TOKEN", workflow)

    def test_scorecard_publication_settings_are_rejected(self) -> None:
        digest = "2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941"
        base_step = [
            "      - name: Run OpenSSF Scorecard",
            "        env:",
            "          INPUT_REPO_TOKEN: ${{ github.token }}",
            "        run: |",
            "          docker run --rm \\",
            "            --env INPUT_PUBLISH_RESULTS=false \\",
            f"            ghcr.io/ossf/scorecard-action@sha256:{digest} # v2.4.3",
        ]
        upload_step = [
            "      - name: Upload result to code scanning",
            "        uses: github/codeql-action/upload-sarif@" + "a" * 40 + " # v4.37.0",
        ]
        mutations = (
            ("            --env INPUT_PUBLISH_RESULTS=true \\", True),
            ("            --env INPUT_INTERNAL_PUBLISH_BASE_URL=https://api.scorecard.dev \\", False),
            ("            --env INPUT_INTERNAL_DEFAULT_TOKEN \\", False),
        )
        for forbidden, replaces_false in mutations:
            with self.subTest(forbidden=forbidden):
                run_step = [
                    forbidden if replaces_false and line == "            --env INPUT_PUBLISH_RESULTS=false \\" else line
                    for line in base_step
                ]
                if not replaces_false:
                    run_step.append(forbidden)
                validator = FoundationValidator(Path("/virtual"))
                validator._validate_step_details(
                    Path("scorecard.yml"),
                    "analysis",
                    {
                        "Run OpenSSF Scorecard": run_step,
                        "Upload result to code scanning": upload_step,
                    },
                )
                self.assertIn("workflow.scorecard_publication", {finding.code for finding in validator.findings})


class RepositoryInventoryHardeningTests(unittest.TestCase):
    def test_cli_root_assertion_cannot_redirect_repository_scope(self) -> None:
        repository_root = Path(__file__).resolve().parents[2]
        self.assertEqual(parse_arguments(()).root, repository_root)
        self.assertEqual(
            parse_arguments(("--root", str(repository_root))).root,
            repository_root,
        )

        with tempfile.TemporaryDirectory() as directory:
            temporary_root = Path(directory)
            root_link = temporary_root / "checkout-link"
            root_link.symlink_to(repository_root, target_is_directory=True)
            self.assertEqual(
                parse_arguments(("--root", str(root_link))).root,
                repository_root,
            )

            regular_file = temporary_root / "not-a-root"
            regular_file.write_text("not a repository\n", encoding="utf-8")
            rejected = (
                temporary_root,
                repository_root / "tools",
                regular_file,
                temporary_root / "missing",
            )
            for candidate in rejected:
                with self.subTest(candidate=candidate):
                    with redirect_stderr(io.StringIO()), self.assertRaises(
                        SystemExit
                    ) as raised:
                        parse_arguments(("--root", str(candidate)))
                    self.assertEqual(raised.exception.code, 2)

    def test_cli_cannot_redirect_policy_path(self) -> None:
        with redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as raised:
            parse_arguments(("--policy", "alternate.json"))
        self.assertEqual(raised.exception.code, 2)

    def test_cli_format_remains_independent_of_root_assertion(self) -> None:
        repository_root = Path(__file__).resolve().parents[2]
        arguments = parse_arguments(
            ("--root", str(repository_root), "--format", "json")
        )
        self.assertEqual(arguments.root, repository_root)
        self.assertEqual(arguments.format, "json")

    def test_goal_cli_command_accepts_root_assertion(self) -> None:
        repository_root = Path(__file__).resolve().parents[2]
        result = subprocess.run(
            [
                sys.executable,
                str(repository_root / "tools/validate_foundation.py"),
                "--root",
                ".",
                "--format",
                "json",
            ],
            cwd=repository_root,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr or result.stdout)
        self.assertTrue(json.loads(result.stdout)["valid"])

    def test_diagnostic_relative_path_does_not_follow_symlinks(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "root"
            outside = Path(directory) / "outside"
            root.mkdir()
            outside.mkdir()
            link = root / "outside-link"
            link.symlink_to(outside, target_is_directory=True)
            self.assertEqual(relative(link, root), "outside-link")

    @staticmethod
    def _path_policy() -> dict[str, object]:
        return {
            "allowed_top_level_paths": ["docs"],
            "required_paths": [],
            "forbidden_paths": [],
        }

    def test_nested_unadmitted_source_file_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            (docs / "stealth.py").write_text("raise SystemExit(0)\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator.policy = self._path_policy()
            validator._validate_required_and_forbidden_paths()
            self.assertIn("path.inventory", {finding.code for finding in validator.findings})

    def test_new_top_level_source_tree_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            src = root / "src"
            src.mkdir()
            (src / "main.py").write_text("raise SystemExit(0)\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator.policy = self._path_policy()
            validator._validate_required_and_forbidden_paths()
            self.assertIn("path.top_level", {finding.code for finding in validator.findings})
            self.assertIn("path.inventory", {finding.code for finding in validator.findings})

    def test_binary_admission_accepts_exact_bytes_and_rejects_mutation(self) -> None:
        data = b"\x89PNG\r\n\x1a\nfixture"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "asset.png"
            path.write_bytes(data)
            admission = {
                "path": "asset.png",
                "sha256": hashlib.sha256(data).hexdigest(),
                "role": "test fixture",
                "provenance": "unit test",
            }

            validator = FoundationValidator(root)
            validator.repository_files = [path]
            validator.index_entries = []
            validator.policy = {"executable_paths": [], "allowed_binary_artifacts": [admission]}
            validator._validate_tree_encoding_and_format()
            self.assertNotIn("file.binary_digest", {finding.code for finding in validator.findings})

            path.write_bytes(data + b"!")
            validator = FoundationValidator(root)
            validator.repository_files = [path]
            validator.index_entries = []
            validator.policy = {"executable_paths": [], "allowed_binary_artifacts": [admission]}
            validator._validate_tree_encoding_and_format()
            self.assertIn("file.binary_digest", {finding.code for finding in validator.findings})

    def test_numbered_change_record_path_remains_admitted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            records = root / "docs/governance/oeps"
            records.mkdir(parents=True)
            (records / "OEP-0001-example-change.md").write_text("# Example\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator.policy = self._path_policy()
            validator._validate_required_and_forbidden_paths()
            self.assertNotIn("path.inventory", {finding.code for finding in validator.findings})


class RustDependencyBoundaryHardeningTests(unittest.TestCase):
    @staticmethod
    def _write_workspace(root: Path) -> None:
        manifests = {
            "rust-toolchain.toml": """[toolchain]
channel = "1.96.1"
components = ["clippy", "rustfmt"]
profile = "minimal"
""",
            "compiler/Cargo.toml": """[workspace]
members = [
  "crates/orange-compiler",
  "crates/orangec",
]
resolver = "2"

[workspace.package]
version = "0.0.1"
edition = "2024"
rust-version = "1.96.1"
publish = false

[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "deny"
""",
            "compiler/crates/orange-compiler/Cargo.toml": """[package]
name = "orange-compiler"
description = "Permanent compiler foundations for the Orange language"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[lints]
workspace = true
""",
            "compiler/crates/orangec/Cargo.toml": """[package]
name = "orangec"
description = "Command-line frontend for the Orange compiler"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
orange-compiler = { path = "../orange-compiler" }

[lints]
workspace = true
""",
            "compiler/Cargo.lock": """version = 4

[[package]]
name = "orange-compiler"
version = "0.0.1"

[[package]]
name = "orangec"
version = "0.0.1"
dependencies = [
 "orange-compiler",
]
""",
        }
        for value, source in manifests.items():
            path = root / value
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding="utf-8")

    @staticmethod
    def _compiler_findings(root: Path):
        validator = FoundationValidator(root)
        validator._validate_compiler_dependency_boundary()
        return validator.findings

    def test_exact_first_party_workspace_passes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_workspace(root)
            self.assertEqual(self._compiler_findings(root), [])

    def test_external_dependencies_in_every_cargo_context_are_rejected(self) -> None:
        mutations = (
            ("compiler/crates/orange-compiler/Cargo.toml", '[dependencies]\nserde = "1"\n'),
            (
                "compiler/crates/orange-compiler/Cargo.toml",
                '[dev-dependencies]\nserde = { git = "https://example.invalid/serde" }\n',
            ),
            (
                "compiler/crates/orange-compiler/Cargo.toml",
                '[build-dependencies]\nserde = { path = "../../../../outside" }\n',
            ),
            ("compiler/Cargo.toml", '[workspace.dependencies]\nserde = "1"\n'),
            (
                "compiler/crates/orange-compiler/Cargo.toml",
                '[target.\'cfg(unix)\'.dependencies]\nserde = "1"\n',
            ),
            (
                "compiler/crates/orange-compiler/Cargo.toml",
                '[target.\'cfg(unix)\'.dev-dependencies]\nserde = "1"\n',
            ),
            (
                "compiler/crates/orange-compiler/Cargo.toml",
                '[target.\'cfg(unix)\'.build-dependencies]\nserde = "1"\n',
            ),
            (
                "compiler/Cargo.toml",
                '[patch.crates-io]\nserde = { git = "https://example.invalid/serde" }\n',
            ),
            (
                "compiler/Cargo.toml",
                '[replace]\n"serde:1.0.0" = { git = "https://example.invalid/serde" }\n',
            ),
        )
        for value, addition in mutations:
            with self.subTest(manifest=value, addition=addition), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / value
                path.write_text(path.read_text(encoding="utf-8") + "\n" + addition, encoding="utf-8")
                self.assertIn(
                    "compiler.dependencies",
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_admitted_path_dependency_cannot_escape_or_gain_source_fields(self) -> None:
        mutations = (
            '{ path = "../../../../outside" }',
            '{ path = "../orange-compiler", version = "0.0.1" }',
            '{ package = "orange-compiler", git = "https://example.invalid/orange" }',
        )
        for replacement in mutations:
            with self.subTest(replacement=replacement), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / "compiler/crates/orangec/Cargo.toml"
                source = path.read_text(encoding="utf-8")
                path.write_text(
                    source.replace('{ path = "../orange-compiler" }', replacement),
                    encoding="utf-8",
                )
                self.assertIn(
                    "compiler.dependencies",
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_malformed_manifest_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_workspace(root)
            path = root / "compiler/crates/orange-compiler/Cargo.toml"
            path.write_text(path.read_text(encoding="utf-8") + "\n[dependencies\n", encoding="utf-8")
            self.assertIn(
                "compiler.manifest_toml",
                {finding.code for finding in self._compiler_findings(root)},
            )

    def test_lockfile_rejects_registry_and_lock_only_path_packages(self) -> None:
        additions = (
            """[[package]]
name = "serde"
version = "1.0.0"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "0000000000000000000000000000000000000000000000000000000000000000"
""",
            """[[package]]
name = "vendored"
version = "0.1.0"
""",
        )
        for addition in additions:
            with self.subTest(addition=addition), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / "compiler/Cargo.lock"
                path.write_text(path.read_text(encoding="utf-8") + "\n" + addition, encoding="utf-8")
                self.assertIn(
                    "compiler.lock_graph",
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_extra_manifest_is_rejected_by_dependency_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_workspace(root)
            path = root / "compiler/crates/unadmitted/Cargo.toml"
            path.parent.mkdir(parents=True)
            path.write_text('[package]\nname = "unadmitted"\nversion = "0.0.1"\n', encoding="utf-8")
            self.assertIn(
                "compiler.manifest_inventory",
                {finding.code for finding in self._compiler_findings(root)},
            )

    def test_workspace_and_package_identity_cannot_drift(self) -> None:
        mutations = (
            (
                "compiler/Cargo.toml",
                lambda source: source + '\n[package]\nname = "unexpected-root"\nversion = "0.0.1"\n',
                "compiler.package_inventory",
            ),
            (
                "compiler/Cargo.toml",
                lambda source: source.replace('  "crates/orangec",\n', ''),
                "compiler.workspace_members",
            ),
            (
                "compiler/crates/orange-compiler/Cargo.toml",
                lambda source: source.replace('name = "orange-compiler"', 'name = "renamed"'),
                "compiler.package_inventory",
            ),
            (
                "compiler/crates/orange-compiler/Cargo.toml",
                lambda source: source.replace(
                    'name = "orange-compiler"',
                    'name = "orange-compiler"\nworkspace = "../../../../outside"',
                ),
                "compiler.package_inventory",
            ),
        )
        for value, mutate, expected_code in mutations:
            with self.subTest(manifest=value, code=expected_code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / value
                path.write_text(mutate(path.read_text(encoding="utf-8")), encoding="utf-8")
                self.assertIn(
                    expected_code,
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_toolchain_contract_rejects_every_pinned_dimension(self) -> None:
        mutations = (
            lambda source: source.replace('channel = "1.96.1"', 'channel = "stable"'),
            lambda source: source.replace(
                'components = ["clippy", "rustfmt"]',
                'components = ["clippy"]',
            ),
            lambda source: source.replace('profile = "minimal"', 'profile = "default"'),
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / "rust-toolchain.toml"
                path.write_text(mutate(path.read_text(encoding="utf-8")), encoding="utf-8")
                self.assertIn(
                    "compiler.toolchain_contract",
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_workspace_contract_rejects_every_foundation_dimension(self) -> None:
        mutations = (
            lambda source: source.replace('resolver = "2"', 'resolver = "3"'),
            lambda source: source.replace('version = "0.0.1"', 'version = "0.0.2"'),
            lambda source: source.replace('edition = "2024"', 'edition = "2021"'),
            lambda source: source.replace('rust-version = "1.96.1"', 'rust-version = "1.85"'),
            lambda source: source.replace("publish = false", "publish = true"),
            lambda source: source.replace('unsafe_code = "forbid"', 'unsafe_code = "allow"'),
            lambda source: source.replace('all = "deny"', 'all = "warn"'),
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / "compiler/Cargo.toml"
                path.write_text(mutate(path.read_text(encoding="utf-8")), encoding="utf-8")
                self.assertIn(
                    "compiler.manifest_contract",
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_member_contract_rejects_inheritance_and_lint_drift(self) -> None:
        mutations = (
            lambda source: source.replace("version.workspace = true", 'version = "0.0.1"'),
            lambda source: source.replace("edition.workspace = true", 'edition = "2024"'),
            lambda source: source.replace(
                "rust-version.workspace = true",
                'rust-version = "1.96.1"',
            ),
            lambda source: source.replace("publish.workspace = true", "publish = false"),
            lambda source: source.replace(
                "[lints]\nworkspace = true",
                "[lints]\nworkspace = false",
            ),
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / "compiler/crates/orange-compiler/Cargo.toml"
                path.write_text(mutate(path.read_text(encoding="utf-8")), encoding="utf-8")
                self.assertIn(
                    "compiler.manifest_contract",
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_build_scripts_and_legacy_dependency_tables_are_rejected(self) -> None:
        mutations = (
            lambda source: source.replace(
                'description = "Command-line frontend for the Orange compiler"',
                'description = "Command-line frontend for the Orange compiler"\n'
                'build = "src/main.rs"',
            ),
            lambda source: source.replace(
                'description = "Command-line frontend for the Orange compiler"',
                'description = "Command-line frontend for the Orange compiler"\n'
                'links = "orange-native"',
            ),
            lambda source: source
            + '\n[build_dependencies]\norange-compiler = { path = "../orange-compiler" }\n',
            lambda source: source
            + '\n[dev_dependencies]\norange-compiler = { path = "../orange-compiler" }\n',
        )
        for mutate in mutations:
            with self.subTest(mutation=mutate), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                path = root / "compiler/crates/orangec/Cargo.toml"
                path.write_text(mutate(path.read_text(encoding="utf-8")), encoding="utf-8")
                self.assertIn(
                    "compiler.manifest_contract",
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_malformed_toolchain_and_lockfile_fail_closed(self) -> None:
        mutations = (
            ("rust-toolchain.toml", '[toolchain\nchannel = "1.96.1"\n', "compiler.toolchain_toml"),
            ("compiler/Cargo.lock", "version = 4\n[[package]\n", "compiler.lock_toml"),
        )
        for value, source, expected_code in mutations:
            with self.subTest(path=value), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_workspace(root)
                (root / value).write_text(source, encoding="utf-8")
                self.assertIn(
                    expected_code,
                    {finding.code for finding in self._compiler_findings(root)},
                )

    def test_top_level_run_invokes_compiler_contract(self) -> None:
        validator = FoundationValidator(Path("/virtual"))
        other_validation_methods = (
            "_validate_required_and_forbidden_paths",
            "_validate_tree_encoding_and_format",
            "_validate_brand_assets",
            "_validate_protected_file_digests",
            "_validate_hosted_control_evidence",
            "_validate_markdown_links",
            "_validate_json_documents",
            "_validate_schema_fixtures",
            "_validate_workflows",
            "_validate_dependabot",
            "_validate_codeowners",
            "_validate_decision_gates",
            "_validate_traceability",
            "_validate_user_journeys",
            "_validate_proof_foundation_suite",
            "_validate_product_form_decision_packet",
            "_validate_semantic_strata_suite",
            "_validate_change_records",
            "_validate_repository_templates",
        )

        def load_policy() -> None:
            validator.policy = {"loaded": True}

        with ExitStack() as stack:
            stack.enter_context(mock.patch.object(validator, "_load_and_validate_policy", side_effect=load_policy))
            for method_name in other_validation_methods:
                stack.enter_context(mock.patch.object(validator, method_name))
            compiler_contract = stack.enter_context(
                mock.patch.object(validator, "_validate_compiler_dependency_boundary")
            )
            language_contract = stack.enter_context(
                mock.patch.object(validator, "_validate_compiler_language_boundary")
            )
            validator.run()
        compiler_contract.assert_called_once_with()
        language_contract.assert_called_once_with()


class CompilerLanguageBoundaryHardeningTests(unittest.TestCase):
    _BOUNDARY_PATHS = (
        "compiler/crates/orange-compiler/src/source.rs",
        "compiler/crates/orange-compiler/src/lexer.rs",
        "compiler/crates/orange-compiler/src/parser.rs",
        "compiler/crates/orange-compiler/src/semantics.rs",
        "compiler/crates/orange-compiler/src/eval.rs",
        "docs/LANGUAGE_2026.md",
        "docs/SEMANTICS_2026.md",
    )

    def _copy_boundary(self, root: Path) -> None:
        source_root = Path(__file__).resolve().parents[2]
        for value in self._BOUNDARY_PATHS:
            destination = root / value
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source_root / value, destination)

    def _codes(self, root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_compiler_language_boundary()
        return {finding.code for finding in validator.findings}

    def test_exact_normative_and_rust_budgets_pass(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            self.assertFalse(self._codes(root))

    def test_each_compiled_budget_drift_is_rejected(self) -> None:
        mutations = (
            ("compiler/crates/orange-compiler/src/source.rs", "16 * 1024 * 1024", "15 * 1024 * 1024"),
            ("compiler/crates/orange-compiler/src/lexer.rs", "262_144", "262_143"),
            ("compiler/crates/orange-compiler/src/lexer.rs", "usize = 100;", "usize = 99;"),
            ("compiler/crates/orange-compiler/src/parser.rs", "usize = 100;", "usize = 99;"),
            ("compiler/crates/orange-compiler/src/parser.rs", "262_144", "262_143"),
            ("compiler/crates/orange-compiler/src/parser.rs", "1_048_576", "1_048_575"),
            ("compiler/crates/orange-compiler/src/parser.rs", "usize = 64;", "usize = 63;"),
            ("compiler/crates/orange-compiler/src/semantics.rs", "usize = 100;", "usize = 99;"),
            ("compiler/crates/orange-compiler/src/semantics.rs", "262_144", "262_143"),
            ("compiler/crates/orange-compiler/src/semantics.rs", "1_048_576", "1_048_575"),
            ("compiler/crates/orange-compiler/src/semantics.rs", "16_384", "16_383"),
            ("compiler/crates/orange-compiler/src/eval.rs", "1_048_576", "1_048_575"),
        )
        for value, old, new in mutations:
            with self.subTest(path=value, old=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_boundary(root)
                path = root / value
                source = path.read_text(encoding="utf-8")
                self.assertIn(old, source)
                path.write_text(source.replace(old, new, 1), encoding="utf-8")
                self.assertIn("compiler.language_budget", self._codes(root))

    def test_commented_budget_decoy_cannot_hide_real_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "compiler/crates/orange-compiler/src/parser.rs"
            source = path.read_text(encoding="utf-8").replace("1_048_576", "1_048_575", 1)
            source += "\n/*\npub const MAX_PARSE_EVENTS_PER_SOURCE: usize = 1_048_576;\n*/\n"
            path.write_text(source, encoding="utf-8")
            self.assertIn("compiler.language_budget", self._codes(root))

    def test_normative_budget_drift_fails_boundary_and_protected_digest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "docs/LANGUAGE_2026.md"
            source = path.read_text(encoding="utf-8")
            path.write_text(source.replace("262,144 syntax nodes", "262,143 syntax nodes", 1), encoding="utf-8")
            self.assertIn("compiler.language_spec_budget", self._codes(root))
            validator = FoundationValidator(root)
            validator._validate_protected_file_digests()
            self.assertIn("protected_file.digest", {finding.code for finding in validator.findings})

    def test_each_normative_semantic_budget_drift_is_rejected(self) -> None:
        mutations = (
            (
                "100 ordinary semantic diagnostics followed by at most one suppression\n  diagnostic",
                "99 ordinary semantic diagnostics followed by at most one suppression\n  diagnostic",
            ),
            ("262,144 Typed Reference Core nodes", "262,143 Typed Reference Core nodes"),
            ("1,048,576 semantic events", "1,048,575 semantic events"),
            (
                "16,384 significant bits in any decoded integer magnitude",
                "16,383 significant bits in any decoded integer magnitude",
            ),
            ("1,048,576 reference-evaluation steps", "1,048,575 reference-evaluation steps"),
        )
        for old, new in mutations:
            with self.subTest(marker=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_boundary(root)
                path = root / "docs/SEMANTICS_2026.md"
                source = path.read_text(encoding="utf-8")
                self.assertIn(old, source)
                path.write_text(source.replace(old, new, 1), encoding="utf-8")
                self.assertIn("compiler.language_spec_budget", self._codes(root))


class BrandAssetHardeningTests(unittest.TestCase):
    def test_official_brand_manifest_matches_admitted_assets(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        validator = FoundationValidator(source_root)
        validator._validate_brand_assets()
        self.assertFalse([finding for finding in validator.findings if finding.code.startswith("brand.")])

    def test_brand_manifest_metadata_mutation_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(source_root / "assets/brand", root / "assets/brand")
            manifest_path = root / "assets/brand/manifest.json"
            manifest = load_json(manifest_path)
            manifest["assets"][0]["width"] += 1
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_brand_assets()
            self.assertIn("brand.manifest_metadata", {finding.code for finding in validator.findings})

    def test_brand_manifest_source_filename_mutation_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(source_root / "assets/brand", root / "assets/brand")
            manifest_path = root / "assets/brand/manifest.json"
            manifest = load_json(manifest_path)
            manifest["assets"][0]["source_filename"] = "unknown.PNG"
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_brand_assets()
            self.assertIn("brand.manifest_provenance", {finding.code for finding in validator.findings})

    def test_all_official_brand_assets_are_marked_binary_by_git(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        manifest = load_json(source_root / "assets/brand/manifest.json")
        paths = [f"assets/brand/{asset['path']}" for asset in manifest["assets"]]
        result = subprocess.run(
            [
                "git",
                "-C",
                str(source_root),
                "check-attr",
                "binary",
                "text",
                "eol",
                "diff",
                "merge",
                "--",
                *paths,
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        observed = set(result.stdout.splitlines())
        expected = {
            line
            for path in paths
            for line in (
                f"{path}: binary: set",
                f"{path}: text: unset",
                f"{path}: eol: unspecified",
                f"{path}: diff: unset",
                f"{path}: merge: unset",
            )
        }
        self.assertEqual(observed, expected)

    def test_root_readme_uses_only_the_repository_banner(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        readme = (source_root / "README.md").read_text(encoding="utf-8")
        banner = "![Official Orange emblem and wordmark](assets/brand/orange-banner.png)"
        self.assertEqual(readme.count(banner), 1)
        self.assertNotIn("user-attachments/assets", readme)

    def test_canonical_c2pa_container_removal_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(source_root / "assets/brand", root / "assets/brand")
            image_path = root / "assets/brand/orange.png"
            data = image_path.read_bytes()
            self.assertIn(b"caBX", data)
            image_path.write_bytes(data.replace(b"caBX", b"caBY", 1))
            validator = FoundationValidator(root)
            validator._validate_brand_assets()
            self.assertIn("brand.c2pa", {finding.code for finding in validator.findings})


class ProtectedControlHardeningTests(unittest.TestCase):
    def test_codeowners_and_fixture_mutations_are_digest_protected(self) -> None:
        paths = (
            ".github/CODEOWNERS",
            "compiler/crates/orangec/tests/s3a_conformance.rs",
            "compiler/fixtures/s3a/invalid-word-range.or",
            "conformance/foundation/valid/claim-record.json",
        )
        for value in paths:
            with self.subTest(path=value), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = root / value
                path.parent.mkdir(parents=True)
                path.write_text("tampered\n", encoding="utf-8")
                validator = FoundationValidator(root)
                validator._validate_protected_file_digests()
                self.assertIn("protected_file.digest", {finding.code for finding in validator.findings})

    def test_policy_cannot_remove_a_required_security_file(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        policy = json.loads((source_root / "policy/gate0-repository-policy.json").read_text(encoding="utf-8"))
        policy["required_paths"].remove("SECURITY.md")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "policy/gate0-repository-policy.json"
            path.parent.mkdir(parents=True)
            path.write_text(json.dumps(policy), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._load_and_validate_policy()
            codes = {finding.code for finding in validator.findings}
            self.assertTrue({"policy.minimum", "policy.required_inventory"} & codes)


class HostedControlEvidenceHardeningTests(unittest.TestCase):
    @staticmethod
    def _write_current_evidence(root: Path) -> None:
        common = (
            "Hosted-control snapshot: `snapshot_date=2026-07-11 "
            "review_due_date=2026-10-11 ruleset_id=18810248`\n"
            "Required-check binding: `context=\"Required CI / docs-policy-workflows\" "
            "integration_id=15368`\n"
            "Required-check binding: `context=\"Dependency Review / policy\" "
            "integration_id=15368`\n"
        )
        for value in (
            "docs/operations/GITHUB_CONTROLS.md",
            "docs/security/OSPS_BASELINE.md",
            "docs/security/THREAT_MODEL.md",
        ):
            path = root / value
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(common, encoding="utf-8")

    def test_current_hosted_control_snapshot_is_coherent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence()
            self.assertEqual(validator.findings, [])

    def test_rephrased_stale_unprotected_main_claim_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            path = root / "docs/security/THREAT_MODEL.md"
            path.write_text(
                path.read_text(encoding="utf-8")
                + "The default branch has no protection and no ruleset.\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence()
            self.assertIn(
                "hosted_control.contradiction",
                {finding.code for finding in validator.findings},
            )

    def test_check_context_must_bind_to_the_exact_producer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            path = root / "docs/security/OSPS_BASELINE.md"
            text = path.read_text(encoding="utf-8").replace(
                'context="Dependency Review / policy" integration_id=15368',
                'context="Dependency Review / policy" integration_id=99999',
            )
            path.write_text(text + "Historical integration ID: 15368.\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence()
            self.assertIn("hosted_control.markers", {finding.code for finding in validator.findings})

    def test_missing_evidence_document_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            (root / "docs/security/THREAT_MODEL.md").unlink()
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence()
            self.assertIn("hosted_control.missing", {finding.code for finding in validator.findings})

    def test_snapshot_expires_on_its_review_due_date(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence(today=dt.date(2026, 10, 11))
            self.assertIn("hosted_control.expired", {finding.code for finding in validator.findings})

    def test_extra_conflicting_binding_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            path = root / "docs/operations/GITHUB_CONTROLS.md"
            path.write_text(
                path.read_text(encoding="utf-8")
                + 'Required-check binding: `context="Dependency Review / policy" '
                + "integration_id=99999`\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence()
            self.assertIn("hosted_control.markers", {finding.code for finding in validator.findings})

    def test_markers_hidden_in_a_code_fence_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            path = root / "docs/security/OSPS_BASELINE.md"
            path.write_text(
                "```text\n" + path.read_text(encoding="utf-8") + "```\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence()
            self.assertIn("hosted_control.markers", {finding.code for finding in validator.findings})

    def test_future_dated_snapshot_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_current_evidence(root)
            validator = FoundationValidator(root)
            validator._validate_hosted_control_evidence(today=dt.date(2026, 7, 10))
            self.assertIn(
                "hosted_control.future_snapshot",
                {finding.code for finding in validator.findings},
            )


class DecisionGateHardeningTests(unittest.TestCase):
    def test_status_cannot_bleed_from_the_next_decision(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            (docs / "DECISIONS.md").write_text(
                """# Decisions

## D-017 — Project name

This decision has no status yet.

## D-018 — Licenses

Status: blocked
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = {
                "decision_gates": {
                    "project_name": {
                        "decision": "D-017",
                        "required_status": "blocked",
                    }
                }
            }
            validator._validate_decision_gates()
            self.assertIn("decision.missing", {finding.code for finding in validator.findings})

    def test_fenced_fake_status_cannot_override_real_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            (docs / "DECISIONS.md").write_text(
                """# Decisions

```text
## D-017 - decoy
Status: blocked
```

## D-017 - real

Status: accepted
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = {
                "decision_gates": {
                    "project_name": {"decision": "D-017", "required_status": "blocked"}
                }
            }
            validator._validate_decision_gates()
            self.assertIn("decision.gate_changed", {finding.code for finding in validator.findings})

    def test_duplicate_semantic_decision_sections_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            (docs / "DECISIONS.md").write_text(
                """## D-017 - first
Status: blocked

## D-017 - second
Status: blocked
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = {
                "decision_gates": {
                    "project_name": {"decision": "D-017", "required_status": "blocked"}
                }
            }
            validator._validate_decision_gates()
            self.assertIn("decision.duplicate", {finding.code for finding in validator.findings})

    def test_unclosed_html_comment_cannot_supply_gate_status(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            (docs / "DECISIONS.md").write_text(
                """# Decisions

<!-- unclosed comment
## D-017 - hidden decoy
Status: blocked

### D-017 - demoted real section
Status: accepted
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = {
                "decision_gates": {
                    "project_name": {"decision": "D-017", "required_status": "blocked"}
                }
            }
            validator._validate_decision_gates()
            self.assertIn("decision.missing", {finding.code for finding in validator.findings})


class ChangeRecordHardeningTests(unittest.TestCase):
    _ACCEPTED_REVISION = "0123456789abcdef0123456789abcdef01234567"

    def _write_accepted_oep(
        self,
        root: Path,
        *,
        review_authorities: list[str],
        approval_records: list[str],
    ) -> None:
        oeps = root / "docs/governance/oeps"
        oeps.mkdir(parents=True)
        authorities = "\n".join(f"  - {value}" for value in review_authorities)
        approvals = "\n".join(f"  - {value}" for value in approval_records)
        headings = (
            "Abstract",
            "Motivation",
            "Scope and non-goals",
            "Specification",
            "Alternatives",
            "Compatibility and migration",
            "Semantic and claim effects",
            "TCB, axiom, and proof effects",
            "Threat, abuse, and leakage effects",
            "Conformance, tests, and evidence",
            "Unresolved questions",
            "Decision record",
        )
        body = "\n".join(
            f"## {heading}\n\nSubstantive solo-mode decision evidence for this test record.\n"
            for heading in headings
        )
        (oeps / "OEP-0001-solo-test.md").write_text(
            f"""---
number: OEP-0001
title: Solo test
authors:
  - Chase Bryan
champion: Chase Bryan
status: Accepted
type: Process
created: 2026-07-12
updated: 2026-07-12
discussion: owner-test
related-decisions:
  - D-023
related-adrs: []
requires: []
supersedes: []
superseded-by: null
review-authorities:
{authorities}
decision-date: 2026-07-12
decision-revision: {self._ACCEPTED_REVISION}
approval-records:
{approvals}
---

# OEP-0001: Solo test

{body}""",
            encoding="utf-8",
        )

    def test_empty_and_invalid_oep_metadata_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            oeps = root / "docs/governance/oeps"
            oeps.mkdir(parents=True)
            (oeps / "OEP-0001-bad-record.md").write_text(
                """---
number: OEP-0001
title:
authors:
champion:
status: Accepted
type: Process
created: banana
updated: yesterday
review-authorities:
---

# Bad record
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_change_records()
            codes = {finding.code for finding in validator.findings}
            self.assertIn("record.value", codes)
            self.assertIn("record.date", codes)

    def test_duplicate_front_matter_key_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            adrs = root / "docs/governance/adrs"
            adrs.mkdir(parents=True)
            (adrs / "ADR-0001-duplicate.md").write_text(
                """---
number: ADR-0001
title: First title
title: Second title
status: Proposed
date: 2026-07-11
owners:
  - chasebryan
reviewers:
  - reviewer
related-oeps:
  - OEP-0001
---
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_change_records()
            self.assertIn("record.front_matter", {finding.code for finding in validator.findings})

    def test_accepted_oep_requires_review_bound_acceptance_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            oeps = root / "docs/governance/oeps"
            oeps.mkdir(parents=True)
            (oeps / "OEP-0001-superficial.md").write_text(
                """---
number: OEP-0001
title: Superficial acceptance
authors:
  - same-person
champion: same-person
status: Accepted
type: Process
created: 2026-07-11
updated: 2026-07-11
discussion: issue-1
related-decisions: []
related-adrs: []
requires: []
supersedes: []
superseded-by: null
review-authorities:
  - same-person
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0001: Superficial
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_change_records()
            codes = {finding.code for finding in validator.findings}
            self.assertIn("record.acceptance", codes)
            self.assertIn("record.independence", codes)
            self.assertIn("record.section", codes)

    def test_accepted_solo_oep_allows_owner_authorship_without_fake_independence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_accepted_oep(
                root,
                review_authorities=["Orange Project Owner"],
                approval_records=[
                    f"solo-reviewed owner acceptance at revision {self._ACCEPTED_REVISION}"
                ],
            )
            validator = FoundationValidator(root)
            validator._validate_change_records()
            codes = {finding.code for finding in validator.findings}
            self.assertNotIn("record.acceptance", codes)
            self.assertNotIn("record.independence", codes)

    def test_accepted_solo_oep_rejects_authority_alias_as_independence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_accepted_oep(
                root,
                review_authorities=["Chase Bryan"],
                approval_records=[
                    f"solo-reviewed owner acceptance at revision {self._ACCEPTED_REVISION}"
                ],
            )
            validator = FoundationValidator(root)
            validator._validate_change_records()
            self.assertIn("record.independence", {finding.code for finding in validator.findings})

    def test_accepted_solo_oep_requires_literal_solo_reviewed_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_accepted_oep(
                root,
                review_authorities=["Orange Project Owner"],
                approval_records=[f"owner approval at revision {self._ACCEPTED_REVISION}"],
            )
            validator = FoundationValidator(root)
            validator._validate_change_records()
            self.assertIn("record.acceptance", {finding.code for finding in validator.findings})

    def test_accepted_solo_oep_rejects_independent_claim_inside_approval_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_accepted_oep(
                root,
                review_authorities=["Orange Project Owner"],
                approval_records=[
                    f"solo-reviewed and independently reviewed at revision {self._ACCEPTED_REVISION}"
                ],
            )
            validator = FoundationValidator(root)
            validator._validate_change_records()
            self.assertIn("record.independence", {finding.code for finding in validator.findings})

    def test_accepted_solo_oep_may_disclose_absent_independent_review(self) -> None:
        records = (
            f"solo-reviewed; no independent review was available; revision {self._ACCEPTED_REVISION}",
            f"solo-reviewed without an independent review; revision {self._ACCEPTED_REVISION}",
            f"solo-reviewed; independent review was unavailable; revision {self._ACCEPTED_REVISION}",
        )
        for record in records:
            with self.subTest(record=record), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_accepted_oep(
                    root,
                    review_authorities=["Orange Project Owner"],
                    approval_records=[record],
                )
                validator = FoundationValidator(root)
                validator._validate_change_records()
                self.assertNotIn("record.independence", {finding.code for finding in validator.findings})

    def test_accepted_solo_oep_approval_must_bind_exact_decision_revision(self) -> None:
        wrong_revision = "1123456789abcdef0123456789abcdef01234567"
        records = (
            "solo-reviewed owner acceptance without a revision",
            f"solo-reviewed owner acceptance at revision {wrong_revision}",
            f"solo-reviewed owner acceptance at revision a{self._ACCEPTED_REVISION}",
        )
        for record in records:
            with self.subTest(record=record), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._write_accepted_oep(
                    root,
                    review_authorities=["Orange Project Owner"],
                    approval_records=[record],
                )
                validator = FoundationValidator(root)
                validator._validate_change_records()
                self.assertIn("record.acceptance", {finding.code for finding in validator.findings})


class PlanningTraceHardeningTests(unittest.TestCase):
    def _copy_planning_docs(self, root: Path) -> None:
        source = Path(__file__).resolve().parents[2] / "docs"
        docs = root / "docs"
        docs.mkdir()
        for name in ("DECISIONS.md", "GATE0_TRACEABILITY.md", "PROJECT_CHARTER.md", "USER_JOURNEYS.md"):
            shutil.copyfile(source / name, docs / name)

    def test_duplicate_feature_identifier_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_planning_docs(root)
            path = root / "docs/GATE0_TRACEABILITY.md"
            source = path.read_text(encoding="utf-8")
            path.write_text(source.replace("| F-14 | Reproducible", "| F-13 | Reproducible", 1), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_traceability()
            self.assertIn("traceability.feature_ids", {finding.code for finding in validator.findings})

    def test_charter_section_mutation_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_planning_docs(root)
            path = root / "docs/PROJECT_CHARTER.md"
            source = path.read_text(encoding="utf-8")
            path.write_text(source.replace("- A versioned language", "- An altered language", 1), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_traceability()
            self.assertIn("traceability.charter_digest", {finding.code for finding in validator.findings})

    def test_unknown_journey_operation_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_planning_docs(root)
            path = root / "docs/USER_JOURNEYS.md"
            source = path.read_text(encoding="utf-8")
            path.write_text(source.replace("`install` | F-01", "`invent` | F-01", 1), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_user_journeys()
            codes = {finding.code for finding in validator.findings}
            self.assertIn("journey.operation_ref", codes)
            self.assertIn("journey.operation_coverage", codes)

    def test_missing_proof_suite_case_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / "docs"
            docs.mkdir()
            source = Path(__file__).resolve().parents[2] / "docs/PROOF_FOUNDATION_DECISION_SUITE.md"
            target = docs / source.name
            shutil.copyfile(source, target)
            text = target.read_text(encoding="utf-8")
            target.write_text(text.replace("### DS-07", "### DS-06", 1), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_proof_foundation_suite()
            self.assertIn("proof_suite.case_ids", {finding.code for finding in validator.findings})

    def test_semantic_strata_suite_baseline_is_valid(self) -> None:
        root = Path(__file__).resolve().parents[2]
        validator = FoundationValidator(root)
        validator._validate_semantic_strata_suite()
        self.assertEqual(
            [
                finding
                for finding in validator.findings
                if finding.code.startswith("semantic_strata.")
            ],
            [],
        )

    def test_product_form_decision_packet_baseline_is_valid(self) -> None:
        root = Path(__file__).resolve().parents[2]
        validator = FoundationValidator(root)
        validator._validate_product_form_decision_packet()
        self.assertEqual(
            [
                finding
                for finding in validator.findings
                if finding.code.startswith("product_form.")
            ],
            [],
        )

    def test_product_form_decision_packet_mutations_are_rejected(self) -> None:
        mutations = (
            ("| PF-G08 |", "| PF-G07 |", "product_form.hard_gates"),
            ("| J-08 |", "| J-07 |", "product_form.journeys"),
            (
                "PF-04: Rust subset with proof annotations",
                "PF-03: Rust subset with proof annotations",
                "product_form.candidates",
            ),
            (
                "The packet has no OEP number, intake",
                "The packet has an assigned OEP number, intake",
                "product_form.assertion",
            ),
        )
        source = (
            Path(__file__).resolve().parents[2]
            / "docs/PRODUCT_FORM_DECISION_PACKET.md"
        )
        for old, new, expected_code in mutations:
            with self.subTest(old=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                docs = root / "docs"
                docs.mkdir()
                target = docs / source.name
                shutil.copyfile(source, target)
                text = target.read_text(encoding="utf-8")
                mutated = text.replace(old, new, 1)
                self.assertNotEqual(mutated, text)
                target.write_text(mutated, encoding="utf-8")
                validator = FoundationValidator(root)
                validator._validate_product_form_decision_packet()
                self.assertIn(
                    expected_code,
                    {finding.code for finding in validator.findings},
                )

    def test_semantic_strata_suite_identity_mutations_are_rejected(self) -> None:
        mutations = (
            ("### SC-05", "### SC-04", "semantic_strata.case_ids"),
            ("10. **SS-G10", "10. **SS-G09", "semantic_strata.hard_gates"),
            ("| SR-14 |", "| SR-13 |", "semantic_strata.relationships"),
            ("| ST-HOST |", "| ST-REL |", "semantic_strata.candidates"),
            (
                "| ST-HOST | Host-delegated strata |",
                "| ST-HOST |\n",
                "semantic_strata.candidates",
            ),
        )
        source = (
            Path(__file__).resolve().parents[2]
            / "docs/SEMANTIC_STRATA_DECISION_SUITE.md"
        )
        for old, new, expected_code in mutations:
            with self.subTest(old=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                docs = root / "docs"
                docs.mkdir()
                target = docs / source.name
                shutil.copyfile(source, target)
                text = target.read_text(encoding="utf-8")
                mutated = text.replace(old, new, 1)
                self.assertNotEqual(mutated, text)
                target.write_text(mutated, encoding="utf-8")
                validator = FoundationValidator(root)
                validator._validate_semantic_strata_suite()
                self.assertIn(
                    expected_code,
                    {finding.code for finding in validator.findings},
                )

    def test_semantic_strata_suite_protocol_mutations_are_rejected(self) -> None:
        mutations = (
            (
                "**Falsification:**",
                "**Unsupported conclusion:**",
                "semantic_strata.case_field",
            ),
            (
                "Execution evidence is currently 0/5 candidates and 0/5 cases.",
                "Execution evidence is currently complete.",
                "semantic_strata.assertion",
            ),
        )
        source = (
            Path(__file__).resolve().parents[2]
            / "docs/SEMANTIC_STRATA_DECISION_SUITE.md"
        )
        for old, new, expected_code in mutations:
            with self.subTest(old=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                docs = root / "docs"
                docs.mkdir()
                target = docs / source.name
                shutil.copyfile(source, target)
                text = target.read_text(encoding="utf-8")
                mutated = text.replace(old, new, 1)
                self.assertNotEqual(mutated, text)
                target.write_text(mutated, encoding="utf-8")
                validator = FoundationValidator(root)
                validator._validate_semantic_strata_suite()
                self.assertIn(
                    expected_code,
                    {finding.code for finding in validator.findings},
                )


class SchemaDeterminismTests(unittest.TestCase):
    def test_additional_property_issues_are_sorted(self) -> None:
        schema_path = Path("/virtual/record.schema.json")
        schema = {
            "$id": "urn:orange:gate0:test:determinism",
            "type": "object",
            "additionalProperties": False,
        }
        issues = validate_schema_instance(
            {"z": 1, "a": 2},
            schema,
            schema_path,
            {schema_path: schema},
            {schema["$id"]: (schema_path, schema)},
        )
        self.assertEqual([issue.instance_path for issue in issues], ["$/a", "$/z"])

    def test_schema_profile_rejects_malformed_keyword_values(self) -> None:
        findings = audit_schema_vocabulary(
            {
                "type": ["string", "string"],
                "required": ["x", "x"],
                "enum": ["same", "same"],
                "minLength": -1,
                "uniqueItems": "yes",
                "properties": [],
            }
        )
        for fragment in ("type", "required", "enum", "minLength", "uniqueItems", "properties"):
            self.assertTrue(any(fragment in finding for finding in findings), fragment)

    def test_schema_profile_rejects_invalid_regular_expression(self) -> None:
        findings = audit_schema_vocabulary({"patternProperties": {"[": {"type": "string"}}})
        self.assertTrue(any("invalid patternProperties" in finding for finding in findings))


if __name__ == "__main__":
    unittest.main()
