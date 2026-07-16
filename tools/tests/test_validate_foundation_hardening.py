from __future__ import annotations

import datetime as dt
import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import ExitStack, redirect_stderr, redirect_stdout
from pathlib import Path
from unittest import mock

from tools.validate_foundation import (
    _read_git_records,
    FoundationValidator,
    Finding,
    GATE0_GIT_EXECUTABLE,
    GATE0_MAXIMUM_FINDING_MESSAGE_CHARACTERS,
    GATE0_MAXIMUM_FINDINGS,
    SCHEMA_DIALECT,
    audit_schema_vocabulary,
    canonical_json_bytes,
    duplicate_yaml_mapping_key,
    load_json,
    main,
    parse_arguments,
    parse_front_matter,
    parse_rust_usize_product,
    relative,
    rust_code_without_comments_and_literals,
    safe_manifest_path,
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


def protected_file_policy() -> dict[str, object]:
    source = Path(__file__).resolve().parents[2] / "policy/gate0-repository-policy.json"
    return {"protected_file_digests": load_json(source)["protected_file_digests"]}


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

    def test_git_inventory_ignores_a_hostile_path(self) -> None:
        self.assertEqual(GATE0_GIT_EXECUTABLE, "/usr/bin/git")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            marker = root / "hostile-git-ran"
            hostile_bin = root / "bin"
            hostile_bin.mkdir()
            fake_git = hostile_bin / "git"
            fake_git.write_text(
                "#!/bin/sh\n"
                f"/usr/bin/touch -- {marker}\n",
                encoding="utf-8",
            )
            fake_git.chmod(0o755)

            with mock.patch.dict("os.environ", {"PATH": str(hostile_bin)}):
                _read_git_records(
                    root,
                    ["ls-files"],
                    maximum_record_bytes=128,
                )

            self.assertFalse(marker.exists())

    def test_schema_equality_distinguishes_booleans_from_integers(self) -> None:
        schema_path = Path("/virtual/equality.schema.json")
        for keyword, schema in (("const", {"const": True}), ("enum", {"enum": [True]})):
            with self.subTest(keyword=keyword):
                issues = validate_schema_instance(1, schema, schema_path, {schema_path: schema}, {})
                self.assertIn(keyword, {issue.keyword for issue in issues})


class WorkflowHardeningTests(unittest.TestCase):
    def test_public_project_status_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "README.md").read_text(encoding="utf-8")
        mutations = (
            (
                "Orange is now in solo, pre-alpha compiler development.",
                "Orange is now a production-ready compiler and toolchain.",
            ),
            (
                "Implemented behavior is solo-authored and solo-reviewed. It is not independently\nreviewed, formally verified, production-ready, or a cryptographic assurance\nclaim.",
                "Implemented behavior is independently reviewed, formally verified, production-ready, and a cryptographic assurance claim.",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "README.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "project.public_status_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_dependency_admission_contract_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "DEPENDENCY_POLICY.md").read_text(encoding="utf-8")
        mutations = (
            (
                "It admits no third-party\nRust crates.",
                "It admits third-party Rust crates without a separate admission record.",
            ),
            (
                "GitHub Actions and reusable workflows use a full 40-character commit SHA",
                "GitHub Actions and reusable workflows may use mutable tags",
            ),
            (
                "No\nexception may waive an assurance stop-ship condition.",
                "An exception may waive an assurance stop-ship condition.",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "DEPENDENCY_POLICY.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "dependency.admission_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_conduct_reporting_boundary_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "CODE_OF_CONDUCT.md").read_text(encoding="utf-8")
        mutations = (
            (
                "The\nrequest itself is public: do not describe the incident or identify affected\npeople there.",
                "The request itself is private: describe the incident and identify affected people there.",
            ),
            (
                "There is no independent private\nintake or appeal yet",
                "There is an independent private intake and appeal",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "CODE_OF_CONDUCT.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "conduct.reporting_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_solo_governance_boundary_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "GOVERNANCE.md").read_text(encoding="utf-8")
        mutations = (
            (
                "The single-person model cannot supply independent review, separation of duties,\nmulti-party key custody, external validation, or organizational bus-factor\nassurance.",
                "The single-person model supplies independent review, separation of duties, multi-party key custody, external validation, and organizational bus-factor assurance.",
            ),
            (
                "is labeled `owner-approved` or `solo-reviewed`, never `independently reviewed`.",
                "is labeled `owner-approved`, `solo-reviewed`, or `independently reviewed`.",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "GOVERNANCE.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "governance.solo_authority_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_support_claim_boundary_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "SUPPORT.md").read_text(encoding="utf-8")
        mutations = (
            (
                "It is not a supported product and provides no\nproduction, compatibility, cryptographic, or software-security guarantee.",
                "It is a supported product with production, compatibility, cryptographic, and software-security guarantees.",
            ),
            (
                "No response-time SLA is currently offered.",
                "A response-time SLA is currently offered.",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "SUPPORT.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "support.claim_boundary_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_release_boundary_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "RELEASE_POLICY.md").read_text(encoding="utf-8")
        mutations = (
            (
                "A merge,\narchive, CI artifact, Cargo build, or planning snapshot is not an Orange product\nrelease",
                "A merge, archive, CI artifact, Cargo build, or planning snapshot is an Orange product release",
            ),
            (
                "prohibit crate,\npackage-registry, and binary distribution until their exact release boundary is\nrecorded",
                "permit crate, package-registry, and binary distribution before an exact release boundary is recorded",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "RELEASE_POLICY.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "release.boundary_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_contribution_legal_boundary_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "CONTRIBUTING.md").read_text(encoding="utf-8")
        mutations = (
            (
                "does **not** accept third-party pull requests for merge",
                "accepts third-party pull requests for merge",
            ),
            (
                "Do not contribute original code or prose in an issue.",
                "Original code and prose may be contributed in an issue.",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "CONTRIBUTING.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "contribution.legal_boundary_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_security_reporting_contract_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "SECURITY.md").read_text(encoding="utf-8")
        mutations = (
            (
                "Never disclose an unpatched",
                "You may disclose an unpatched",
            ),
            (
                "Give the project\na reasonable opportunity to remediate before disclosure.",
                "Immediate public disclosure is encouraged.",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.assertIn(old, source)
                (root / "SECURITY.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "security.reporting_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_editorconfig_contract_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        target = source_root / ".editorconfig"
        source = target.read_bytes()
        mutations = (
            (b"end_of_line = lf", b"end_of_line = crlf"),
            (b"trim_trailing_whitespace = true", b"trim_trailing_whitespace = false"),
        )
        for old, new in mutations:
            with self.subTest(setting=old):
                self.assertIn(old, source)
                validator = FoundationValidator(source_root)
                read_repository_bytes = validator._read_repository_bytes

                def substituted_read(path: Path) -> bytes | None:
                    if path == target:
                        return source.replace(old, new, 1)
                    return read_repository_bytes(path)

                with mock.patch.object(
                    validator,
                    "_read_repository_bytes",
                    side_effect=substituted_read,
                ):
                    findings = validator.run()
                self.assertIn(
                    "editorconfig.contract",
                    {finding.code for finding in findings},
                )

    def test_gitignore_contract_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        mutations = (
            (".gitignore", b"*.pem", b"*.crt"),
            ("compiler/.gitignore", b"/target/", b"/build/"),
        )
        for value, old, new in mutations:
            with self.subTest(path=value):
                target = source_root / value
                source = target.read_bytes()
                self.assertIn(old, source)
                validator = FoundationValidator(source_root)
                read_repository_bytes = validator._read_repository_bytes

                def substituted_read(path: Path) -> bytes | None:
                    if path == target:
                        return source.replace(old, new, 1)
                    return read_repository_bytes(path)

                with mock.patch.object(
                    validator,
                    "_read_repository_bytes",
                    side_effect=substituted_read,
                ):
                    findings = validator.run()
                self.assertIn(
                    "gitignore.contract",
                    {finding.code for finding in findings},
                )

    def test_issue_form_safety_contract_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        mutations = (
            (
                "conduct-contact.yml",
                "This issue is public. Do not describe the incident",
                "This issue is public. Describe the incident",
            ),
            (
                "oep-proposal.yml",
                "Acceptance occurs through a maintainer-authored, reviewed, numbered OEP",
                "Acceptance occurs when this intake issue is submitted",
            ),
        )
        for name, old, new in mutations:
            with self.subTest(template=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                template_dir = root / ".github/ISSUE_TEMPLATE"
                template_dir.mkdir(parents=True)
                source = (source_root / ".github/ISSUE_TEMPLATE" / name).read_text(
                    encoding="utf-8"
                )
                self.assertIn(old, source)
                (template_dir / name).write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "template.issue_form_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_pull_request_safety_contract_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        mutations = (
            (
                "Every row must say `Changed` or `No change` and explain why.",
                "Every row should explain the impact when practical.",
            ),
            (
                "I included no secret, private key, embargoed vulnerability, or private cryptographic material.",
                "I included no secret, private key, or private cryptographic material.",
            ),
        )
        for old, new in mutations:
            with self.subTest(safeguard=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                github_dir = root / ".github"
                github_dir.mkdir()
                source = (source_root / ".github/pull_request_template.md").read_text(
                    encoding="utf-8"
                )
                self.assertIn(old, source)
                (github_dir / "pull_request_template.md").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "template.pr_safety_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_issue_routing_configuration_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        mutations = (
            ("blank_issues_enabled: false", "blank_issues_enabled: true"),
            (
                "https://github.com/chasebryan/orange/security/advisories/new",
                "https://github.com/chasebryan/orange/issues/new",
            ),
        )
        for old, new in mutations:
            with self.subTest(setting=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                template_dir = root / ".github/ISSUE_TEMPLATE"
                template_dir.mkdir(parents=True)
                source = (source_root / ".github/ISSUE_TEMPLATE/config.yml").read_text(
                    encoding="utf-8"
                )
                self.assertIn(old, source)
                (template_dir / "config.yml").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_repository_templates()
                self.assertIn(
                    "template.issue_routing_contract",
                    {finding.code for finding in validator.findings},
                )

    def test_markdownlint_configuration_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = (source_root / ".markdownlint-cli2.jsonc").read_text(encoding="utf-8")
            old = '    "compiler/target/**"'
            self.assertIn(old, source)
            (root / ".markdownlint-cli2.jsonc").write_text(
                source.replace(old, '    "**"', 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = load_json(source_root / "policy/gate0-repository-policy.json")
            validator._validate_workflows()
            self.assertIn(
                "markdownlint.configuration",
                {finding.code for finding in validator.findings},
            )

    def test_codeowners_coverage_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            github = root / ".github"
            github.mkdir()
            source = (source_root / ".github/CODEOWNERS").read_text(encoding="utf-8")
            rule = "/RELEASE_POLICY.md @chasebryan\n"
            self.assertIn(rule, source)
            (github / "CODEOWNERS").write_text(
                source.replace(rule, "", 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = load_json(source_root / "policy/gate0-repository-policy.json")
            validator._validate_codeowners()
            self.assertIn(
                "codeowners.contract",
                {finding.code for finding in validator.findings},
            )

    def test_codeowners_order_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            github = root / ".github"
            github.mkdir()
            source = (source_root / ".github/CODEOWNERS").read_text(encoding="utf-8")
            first = "/SECURITY.md @chasebryan\n/GOVERNANCE.md @chasebryan\n"
            second = "/GOVERNANCE.md @chasebryan\n/SECURITY.md @chasebryan\n"
            self.assertIn(first, source)
            (github / "CODEOWNERS").write_text(
                source.replace(first, second, 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = load_json(source_root / "policy/gate0-repository-policy.json")
            validator._validate_codeowners()
            self.assertIn(
                "codeowners.contract",
                {finding.code for finding in validator.findings},
            )

    def test_dependabot_configuration_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            github = root / ".github"
            github.mkdir()
            source = (source_root / ".github/dependabot.yml").read_text(encoding="utf-8")
            old = "      default-days: 7"
            self.assertIn(old, source)
            (github / "dependabot.yml").write_text(
                source.replace(old, "      default-days: 8", 1),
                encoding="utf-8",
            )
            shutil.copyfile(
                source_root / ".github/dependency-review-config.yml",
                github / "dependency-review-config.yml",
            )
            validator = FoundationValidator(root)
            validator._validate_dependabot()
            self.assertIn(
                "dependabot.configuration",
                {finding.code for finding in validator.findings},
            )

    def test_dependency_review_configuration_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            github = root / ".github"
            github.mkdir()
            shutil.copyfile(source_root / ".github/dependabot.yml", github / "dependabot.yml")
            source = (source_root / ".github/dependency-review-config.yml").read_text(
                encoding="utf-8"
            )
            old = "retry_on_snapshot_warnings_timeout: 120"
            self.assertIn(old, source)
            (github / "dependency-review-config.yml").write_text(
                source.replace(old, "retry_on_snapshot_warnings_timeout: 121", 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_dependabot()
            self.assertIn(
                "dependency_review.configuration",
                {finding.code for finding in validator.findings},
            )

    def test_unreviewed_workflow_is_reported_without_crashing(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflow_dir = root / ".github/workflows"
            workflow_dir.mkdir(parents=True)
            (workflow_dir / "unreviewed.yml").write_text(
                "name: Unreviewed\non:\n  workflow_dispatch:\npermissions: {}\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = load_json(source_root / "policy/gate0-repository-policy.json")

            validator._validate_workflows()

            self.assertIn("workflow.inventory", {finding.code for finding in validator.findings})

    def test_critical_workflow_step_contracts_are_exact(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        mutations = (
            (
                "ci.yml",
                "          persist-credentials: false\n",
                "          persist-credentials: false\n          ref: main\n",
                "workflow.checkout_contract",
            ),
            (
                "dependency-review.yml",
                "          config-file: ./.github/dependency-review-config.yml\n",
                "          config-file: ./.github/dependency-review-config.yml\n          warn-only: true\n",
                "workflow.dependency_review_contract",
            ),
            (
                "dependency-review.yml",
                "    runs-on: ubuntu-24.04\n",
                "    if: false\n    runs-on: ubuntu-24.04\n",
                "workflow.job_condition_contract",
            ),
            (
                "dependency-review.yml",
                "    runs-on: ubuntu-24.04\n",
                "    env:\n      NODE_OPTIONS: --require=./bootstrap.js\n    runs-on: ubuntu-24.04\n",
                "workflow.ambient_env",
            ),
            (
                "dependency-review.yml",
                "permissions: {}\n",
                "permissions: {}\nenv:\n  NODE_OPTIONS: --require=./bootstrap.js\n",
                "workflow.ambient_env",
            ),
            (
                "ci.yml",
                "    shell: /bin/bash -p -e -o pipefail {0}\n",
                "    shell: bash {0}\n",
                "workflow.defaults_contract",
            ),
            (
                "dependency-review.yml",
                "permissions: {}\n",
                "permissions: {}\ndefaults:\n  run:\n    shell: /bin/bash -p -e -o pipefail {0}\n",
                "workflow.defaults_contract",
            ),
            (
                "dependency-review.yml",
                "    branches:\n      - main\n",
                "    branches:\n      - main\n    paths-ignore:\n      - compiler/**\n",
                "workflow.path_filter",
            ),
            (
                "dependency-review.yml",
                "  merge_group:\n",
                "  push:\n    branches:\n      - main\n  merge_group:\n",
                "workflow.event_contract",
            ),
            (
                "ci.yml",
                "  pull_request:\n    branches:\n      - main\n",
                "  pull_request:\n    branches:\n      - main\n    types:\n      - closed\n",
                "workflow.event_contract",
            ),
            (
                "dependency-review.yml",
                "      - checks_requested\n",
                "      - destroyed\n",
                "workflow.event_contract",
            ),
            (
                "external-links.yml",
                "    - cron: \"23 4 * * 1\"\n",
                "    - cron: \"23 4 * * 0\"\n",
                "workflow.event_contract",
            ),
            (
                "workflow-online-audit.yml",
                "  workflow_dispatch:\n",
                "  workflow_dispatch:\n    inputs:\n      ref:\n        required: false\n",
                "workflow.event_contract",
            ),
            (
                "ci.yml",
                "  group: required-ci-${{ github.event.pull_request.number || github.ref }}\n",
                "  group: required-ci\n",
                "workflow.concurrency",
            ),
            (
                "scorecard.yml",
                "  group: openssf-scorecard-${{ github.ref }}\n",
                "  group: required-ci-${{ github.ref }}\n",
                "workflow.concurrency",
            ),
            (
                "dependency-review.yml",
                "  cancel-in-progress: true\n",
                "  cancel-in-progress: false\n",
                "workflow.concurrency",
            ),
            (
                "ci.yml",
                "    timeout-minutes: 15\n",
                "    timeout-minutes: 1\n",
                "workflow.timeout",
            ),
            (
                "scorecard.yml",
                "    timeout-minutes: 20\n",
                "    timeout-minutes: 200\n",
                "workflow.timeout",
            ),
            (
                "ci.yml",
                "      contents: read\n",
                "      contents: none\n",
                "workflow.job_permissions",
            ),
            (
                "dependency-review.yml",
                "      contents: read\n",
                "      contents: read\n      issues: read\n",
                "workflow.job_permissions",
            ),
            (
                "scorecard.yml",
                "      security-events: write # Required to upload SARIF to code scanning.\n",
                "      security-events: read # Cannot upload SARIF.\n",
                "workflow.job_permissions",
            ),
            (
                "ci.yml",
                "permissions: {}\n",
                "permissions:\n",
                "workflow.permissions",
            ),
            (
                "ci.yml",
                "    name: Required CI / docs-policy-workflows\n",
                "    name: Alternate check\n    # name: Required CI / docs-policy-workflows\n",
                "workflow.required_content",
            ),
            (
                "workflow-online-audit.yml",
                "    name: Workflow Online Audit / upstream metadata\n",
                "    name: Unreviewed metadata job\n",
                "workflow.required_content",
            ),
            (
                "scorecard.yml",
                "name: OpenSSF Scorecard\n",
                "name: Scorecard fork\n# name: OpenSSF Scorecard\n",
                "workflow.name_contract",
            ),
            (
                "dependency-review.yml",
                "      - main\n",
                "      - release\n",
                "workflow.event_contract",
            ),
            (
                "dependency-review.yml",
                "      - main\n",
                "      - main\n      - release\n",
                "workflow.event_contract",
            ),
            (
                "dependency-review.yml",
                "    branches:\n",
                "    branches-ignore:\n",
                "workflow.event_contract",
            ),
            (
                "dependency-review.yml",
                "    runs-on: ubuntu-24.04\n",
                "    defaults:\n      run:\n        shell: bash {0}\n    runs-on: ubuntu-24.04\n",
                "workflow.defaults_contract",
            ),
            (
                "dependency-review.yml",
                "    runs-on: ubuntu-24.04\n",
                "    strategy:\n      matrix:\n        mode: [review, bypass]\n    runs-on: ubuntu-24.04\n",
                "workflow.job_extension",
            ),
            (
                "dependency-review.yml",
                "    runs-on: ubuntu-24.04\n",
                "    needs: missing-gate\n    runs-on: ubuntu-24.04\n",
                "workflow.job_extension",
            ),
            (
                "dependency-review.yml",
                "    runs-on: ubuntu-24.04\n",
                "    runs-on: ubuntu-22.04\n",
                "workflow.runner",
            ),
            (
                "external-links.yml",
                "    runs-on: ubuntu-24.04\n",
                "",
                "workflow.runner",
            ),
            (
                "scorecard.yml",
                "    if: ${{ github.ref == 'refs/heads/main' }}\n",
                "    if: false\n",
                "workflow.job_condition_contract",
            ),
            (
                "ci.yml",
                '          echo "Solo mode does not accept third-party pull requests until D-018 selects contribution terms." >&2\n          exit 1\n',
                '          if false; then\n            echo "Solo mode does not accept third-party pull requests until D-018 selects contribution terms." >&2\n            exit 1\n          fi\n',
                "workflow.solo_boundary_contract",
            ),
            (
                "ci.yml",
                "TZ=UTC python3 -S -P -B -X utf8 -W error::ResourceWarning tools/validate_foundation.py\n",
                "TZ=UTC python3 -S -P -B -X utf8 -W error::ResourceWarning tools/validate_foundation.py || :\n",
                "workflow.ci_gate_contract",
            ),
            (
                "ci.yml",
                " -W error::ResourceWarning tools/validate_foundation.py",
                " tools/validate_foundation.py",
                "workflow.ci_gate_contract",
            ),
            (
                "ci.yml",
                " -W error::ResourceWarning -c",
                " -c",
                "workflow.ci_gate_contract",
            ),
            (
                "ci.yml",
                "        run: ./scripts/ci/install-actionlint \"$RUNNER_TEMP/actionlint\"\n",
                "        run: ./scripts/ci/install-actionlint \"$RUNNER_TEMP/actionlint\" || :\n",
                "workflow.ci_tool_contract",
            ),
            (
                "ci.yml",
                "            **/*.md\n            .github/**/*.md\n",
                "            **/*.md\n",
                "workflow.ci_tool_contract",
            ),
            (
                "external-links.yml",
                '        run: ./scripts/ci/check-external-links "$RUNNER_TEMP/lychee/bin/lychee"\n',
                '        run: ./scripts/ci/check-external-links "$RUNNER_TEMP/lychee/bin/lychee" || :\n',
                "workflow.external_links_contract",
            ),
            (
                "workflow-online-audit.yml",
                "          online-audits: true\n",
                "          online-audits: false\n",
                "workflow.online_audit_contract",
            ),
        )
        for name, original, replacement, expected_code in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                workflow_dir = root / ".github/workflows"
                workflow_dir.mkdir(parents=True)
                source = (source_root / ".github/workflows" / name).read_text(encoding="utf-8")
                self.assertEqual(source.count(original), 1)
                (workflow_dir / name).write_text(
                    source.replace(original, replacement),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator.policy = load_json(source_root / "policy/gate0-repository-policy.json")

                validator._validate_workflows()

                self.assertIn(expected_code, {finding.code for finding in validator.findings})

    def test_duplicate_yaml_mapping_keys_are_rejected_outside_scripts(self) -> None:
        self.assertIsNone(
            duplicate_yaml_mapping_key("run: |2-\n  label: script text\n  label: still script text\n")
        )
        self.assertEqual(
            duplicate_yaml_mapping_key(
                "steps:\n  - name: Checkout\n    with:\n      persist-credentials: false\n      persist-credentials: true\n"
            ),
            "persist-credentials",
        )
        self.assertIsNone(
            duplicate_yaml_mapping_key(
                "steps:\n  - name: First\n    env:\n      VALUE: one\n  - name: Second\n    env:\n      VALUE: two\n"
            )
        )
        source = """name: CI
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
      - name: Script
        run: |
          label: allowed inside the script
          label: still script text
jobs:
  replacement:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    permissions: {}
    steps: []
"""
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflow_dir = root / ".github/workflows"
            workflow_dir.mkdir(parents=True)
            (workflow_dir / "ci.yml").write_text(source, encoding="utf-8")
            validator = FoundationValidator(root)
            validator.policy = workflow_policy()

            validator._validate_workflows()

            self.assertIn(
                "workflow.duplicate_key",
                {finding.code for finding in validator.findings},
            )

    def test_yaml_indirection_syntax_is_rejected(self) -> None:
        base = """name: CI
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
    steps: []
"""
        for syntax in (
            "anchor: &shared value\n",
            "alias: *shared\n",
            "merge: {<<: *shared}\n",
            "tagged: !custom value\n",
        ):
            with self.subTest(syntax=syntax), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                workflow_dir = root / ".github/workflows"
                workflow_dir.mkdir(parents=True)
                (workflow_dir / "ci.yml").write_text(base + syntax, encoding="utf-8")
                validator = FoundationValidator(root)
                validator.policy = workflow_policy()

                validator._validate_workflows()

                self.assertIn(
                    "workflow.indirection",
                    {finding.code for finding in validator.findings},
                )

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
            '/usr/bin/rm -f -- "$GITHUB_WORKSPACE/results.sarif"',
            "/usr/bin/env -i",
            "DOCKER_HOST=unix:///var/run/docker.sock",
            'HOME="$RUNNER_TEMP"',
            "/usr/bin/docker run --rm",
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
            (
                "DOCKER_HOST=unix:///var/run/docker.sock",
                "DOCKER_HOST=tcp://hostile.invalid:2375",
            ),
            ("--cap-add=DAC_OVERRIDE", "--cap-add=ALL"),
            (",readonly\"", "\""),
            ("--env INPUT_PUBLISH_RESULTS=false", "--env INPUT_PUBLISH_RESULTS=true"),
            ("--pids-limit=256", "--privileged"),
            (
                "2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941",
                "0" * 64,
            ),
        )
        for old, new in mutations:
            with self.subTest(mutation=old):
                steps = {name: list(lines) for name, lines in source_steps.items()}
                steps["Run OpenSSF Scorecard"] = [line.replace(old, new) for line in steps["Run OpenSSF Scorecard"]]
                validator = FoundationValidator(Path("/virtual"))
                validator._validate_step_details(Path("scorecard.yml"), "analysis", steps)
                codes = {finding.code for finding in validator.findings}
                self.assertIn("workflow.scorecard_contract", codes)

        steps = {name: list(lines) for name, lines in source_steps.items()}
        steps["Upload result to code scanning"] = [
            "        if: false" if "        if:" in line else line
            for line in steps["Upload result to code scanning"]
        ]
        validator = FoundationValidator(Path("/virtual"))
        validator._validate_step_details(Path("scorecard.yml"), "analysis", steps)
        self.assertIn(
            "workflow.scorecard_upload_contract",
            {finding.code for finding in validator.findings},
        )

    def test_digest_pinned_scorecard_does_not_request_publication_identity(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        workflow = (source_root / ".github/workflows/scorecard.yml").read_text(encoding="utf-8")
        self.assertIn("--env INPUT_PUBLISH_RESULTS=false", workflow)
        self.assertNotIn("INPUT_PUBLISH_RESULTS=true", workflow)
        self.assertNotIn("id-token: write", workflow)
        self.assertNotIn("INPUT_INTERNAL_PUBLISH_BASE_URL", workflow)
        self.assertNotIn("INPUT_INTERNAL_DEFAULT_TOKEN", workflow)

    def test_repository_launcher_canonicalizes_scope_and_fixes_the_build_epoch(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        launcher = (source_root / "scripts/ci/check-repository").read_text(
            encoding="utf-8"
        )
        self.assertIn('if [ -L "$SCRIPT_PATH" ]; then', launcher)
        self.assertIn('find "$SCRIPT_PATH" -prune -links 1 -print', launcher)
        self.assertIn(
            'readonly SCRIPT_DIRECTORY="$(CDPATH= cd -- "${SCRIPT_PATH%/*}" && pwd -P)"',
            launcher,
        )
        self.assertIn(
            'readonly REPOSITORY_ROOT="$(cd -- "$SCRIPT_DIRECTORY/../.." && pwd -P)"',
            launcher,
        )
        self.assertIn("  SOURCE_DATE_EPOCH=0 \\\n", launcher)
        self.assertIn("  -i \\\n", launcher)
        self.assertIn('  HOME="$ACCOUNT_HOME" \\\n', launcher)
        self.assertIn('  PATH="$SAFE_PATH" \\\n', launcher)
        self.assertIn('/usr/bin/getent passwd "$(/usr/bin/id -u)"', launcher)
        self.assertNotIn("${SOURCE_DATE_EPOCH", launcher)
        self.assertIn('if [ "$#" -ne 0 ]; then\n', launcher)
        self.assertIn('/usr/bin/find "$SCRIPT_PATH" -prune -links 1 -print', launcher)
        self.assertIn("exec /usr/bin/env \\\n", launcher)
        self.assertIn(
            "/usr/bin/make --no-builtin-rules --no-builtin-variables check\n",
            launcher,
        )

    def test_repository_launcher_uses_absolute_control_commands(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        launcher = source_root / "scripts/ci/check-repository"
        with tempfile.TemporaryDirectory() as directory:
            test_root = Path(directory)
            script_directory = test_root / "scripts/ci"
            script_directory.mkdir(parents=True)
            copied_launcher = script_directory / "check-repository"
            shutil.copy2(launcher, copied_launcher)
            account_home = test_root / "account-home"
            account_home.mkdir()
            launcher_source = copied_launcher.read_text(encoding="utf-8")
            account_lookup = 'ACCOUNT_RECORD="$(/usr/bin/getent passwd "$(/usr/bin/id -u)")" || {'
            self.assertEqual(launcher_source.count(account_lookup), 1)
            fixture_record = (
                f'ACCOUNT_RECORD="sandbox:x:{os.getuid()}:{os.getgid()}:Sandbox:'
                f'{account_home}:/bin/sh" || {{'
            )
            copied_launcher.write_text(
                launcher_source.replace(account_lookup, fixture_record, 1),
                encoding="utf-8",
            )

            hostile_path = test_root / "hostile-path"
            hostile_path.mkdir()
            marker = test_root / "hostile-command-ran"
            for command in ("env", "find", "make"):
                replacement = hostile_path / command
                replacement.write_text(
                    "#!/bin/sh\n"
                    f"/usr/bin/touch -- {marker}\n"
                    "exit 97\n",
                    encoding="utf-8",
                )
                replacement.chmod(0o755)

            observed = test_root / "environment.txt"
            (test_root / "Makefile").write_text(
                "check:\n"
                f"\t@/usr/bin/env > {observed}\n",
                encoding="utf-8",
            )
            rejected = subprocess.run(
                [copied_launcher, "--unexpected"],
                cwd=test_root,
                env={"PATH": str(hostile_path)},
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(rejected.returncode, 2)
            self.assertEqual(rejected.stdout, "")
            self.assertEqual(rejected.stderr, "usage: check-repository\n")
            self.assertFalse(marker.exists())
            self.assertFalse(observed.exists())

            result = subprocess.run(
                [copied_launcher],
                cwd=test_root,
                env={
                    "BASH_ENV": str(test_root / "hostile-bash-startup"),
                    "ENV": str(test_root / "hostile-shell-startup"),
                    "GNUMAKEFLAGS": "--eval=hostile",
                    "HOME": str(hostile_path),
                    "MAKEFLAGS": "--invalid-hostile-flag",
                    "PATH": str(hostile_path),
                    "TMPDIR": str(hostile_path),
                },
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(marker.exists())
            environment = dict(
                line.split("=", 1)
                for line in observed.read_text(encoding="utf-8").splitlines()
                if "=" in line
            )
            account_home = account_home.resolve()
            self.assertEqual(environment["HOME"], str(account_home))
            self.assertEqual(
                environment["PATH"],
                f"{account_home}/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
            )
            self.assertEqual(environment["LANG"], "C")
            self.assertEqual(environment["LC_ALL"], "C")
            self.assertEqual(environment["SOURCE_DATE_EPOCH"], "0")
            self.assertEqual(environment["TZ"], "UTC")
            self.assertNotEqual(environment.get("MAKEFLAGS"), "--invalid-hostile-flag")
            self.assertNotIn("BASH_ENV", environment)
            self.assertNotIn("ENV", environment)
            self.assertNotIn("GNUMAKEFLAGS", environment)
            self.assertNotIn("TMPDIR", environment)

    def test_repository_launcher_rejects_a_direct_script_symlink(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        launcher = source_root / "scripts/ci/check-repository"
        with tempfile.TemporaryDirectory() as directory:
            alias_root = Path(directory)
            alias_directory = alias_root / "scripts/ci"
            alias_directory.mkdir(parents=True)
            alias = alias_directory / "check-repository"
            alias.symlink_to(launcher)
            marker = alias_root / "wrong-root-make-ran"
            (alias_root / "Makefile").write_text(
                f"check:\n\t@touch -- {marker}\n",
                encoding="utf-8",
            )

            result = subprocess.run(
                [str(alias)],
                cwd=alias_root,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            marker_exists = marker.exists()

        self.assertEqual(result.returncode, 1)
        self.assertIn("must not be invoked through a symbolic link", result.stderr)
        self.assertFalse(marker_exists)

    def test_repository_launcher_rejects_a_direct_script_hardlink(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        launcher = source_root / "scripts/ci/check-repository"
        with tempfile.TemporaryDirectory(
            prefix=".launcher-hardlink-",
            dir=source_root,
        ) as directory:
            alias_root = Path(directory)
            alias_directory = alias_root / "scripts/ci"
            alias_directory.mkdir(parents=True)
            alias = alias_directory / "check-repository"
            alias.hardlink_to(launcher)
            marker = alias_root / "wrong-root-make-ran"
            (alias_root / "Makefile").write_text(
                f"check:\n\t@touch -- {marker}\n",
                encoding="utf-8",
            )

            result = subprocess.run(
                [str(alias)],
                cwd=alias_root,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            marker_exists = marker.exists()

        self.assertEqual(result.returncode, 1)
        self.assertIn("must not be invoked through a hard link", result.stderr)
        self.assertFalse(marker_exists)

    def test_ci_bash_helpers_use_hardened_startup_and_bounded_installers(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        for name in ("check-external-links", "install-actionlint", "install-lychee"):
            with self.subTest(helper=name, contract="interpreter"):
                script = (source_root / "scripts/ci" / name).read_text(encoding="utf-8")
                self.assertTrue(script.startswith("#!/bin/bash -p\n"))
                self.assertIn('[[ $# -ne 1 || -z "${1-}" ]]', script)
                if name == "check-external-links":
                    self.assertIn(
                        '[[ ! -f "$1" || ! -s "$1" || ! -x "$1" || -L "$1" ]]',
                        script,
                    )
                    self.assertIn(
                        '$(/usr/bin/stat --format=\'%a:%h\' -- "$1")',
                        script,
                    )
                    self.assertIn(
                        "exec -- /usr/bin/env \\\n"
                        "  --ignore-environment \\\n"
                        "  -- \\\n"
                        "  LANG=C \\\n"
                        "  LC_ALL=C \\\n"
                        "  PATH=/usr/bin:/bin \\\n"
                        "  TZ=UTC \\\n"
                        '  "$LYCHEE" \\\n',
                        script,
                    )
        contracts = (
            (
                "install-actionlint",
                '"${TMPDIR:-/tmp}/orange-actionlint.XXXXXXXX"',
                'readonly MAXIMUM_ARCHIVE_BYTES="33554432"',
                'readonly MAXIMUM_ARCHIVE_KIB="32768"',
                'readonly MAXIMUM_EXTRACTED_FILE_KIB="65536"',
                "-- \\\n    actionlint\n",
                'readonly EXTRACTED_FILE="$TEMPORARY_DIRECTORY/actionlint"',
                'readonly INSTALLED_FILE="$DESTINATION_DIRECTORY/actionlint"',
            ),
            (
                "install-lychee",
                '"${TMPDIR:-/tmp}/orange-lychee.XXXXXXXX"',
                'readonly MAXIMUM_ARCHIVE_BYTES="67108864"',
                'readonly MAXIMUM_ARCHIVE_KIB="65536"',
                'readonly MAXIMUM_EXTRACTED_FILE_KIB="131072"',
                '-- \\\n    "$ARCHIVE_DIRECTORY/lychee"\n',
                'readonly EXTRACTED_FILE="$TEMPORARY_DIRECTORY/$ARCHIVE_DIRECTORY/lychee"',
                'readonly INSTALLED_FILE="$DESTINATION_DIRECTORY/bin/lychee"',
            ),
        )
        for (
            name,
            temporary_template,
            maximum_archive_size,
            maximum_archive_kib,
            maximum_file_size,
            selected_member,
            extracted_file,
            installed_file,
        ) in contracts:
            with self.subTest(installer=name):
                script = (source_root / "scripts/ci" / name).read_text(encoding="utf-8")
                for required in (
                    'readonly PATH="/usr/bin:/bin"\nexport PATH\n',
                    "unset GZIP TAR_OPTIONS",
                    'if [[ -e "$1" || -L "$1" ]]; then',
                    'echo "DESTINATION_DIRECTORY must not already exist" >&2',
                    f"/usr/bin/mktemp -d -- {temporary_template}",
                    'TEMPORARY_DIRECTORY="$(CDPATH= cd -- "$TEMPORARY_DIRECTORY" && pwd -P)"',
                    "trap '/usr/bin/rm -rf -- \"$TEMPORARY_DIRECTORY\"' EXIT",
                    maximum_archive_size,
                    maximum_archive_kib,
                    'readonly MAXIMUM_CONNECTION_SECONDS="20"',
                    'readonly MAXIMUM_DOWNLOAD_SECONDS="300"',
                    maximum_file_size,
                    '(\n  ulimit -c 0\n  ulimit -f "$MAXIMUM_ARCHIVE_KIB"\n  curl \\\n'
                    "    --disable \\\n",
                    '--connect-timeout "$MAXIMUM_CONNECTION_SECONDS"',
                    '--max-filesize "$MAXIMUM_ARCHIVE_BYTES"',
                    '--max-time "$MAXIMUM_DOWNLOAD_SECONDS"',
                    '(\n  ulimit -c 0\n  ulimit -f "$MAXIMUM_EXTRACTED_FILE_KIB"\n  tar \\\n',
                    "--no-same-owner",
                    "--no-same-permissions",
                    selected_member,
                    extracted_file,
                    installed_file,
                    '[[ ! -f "$EXTRACTED_FILE" || ! -s "$EXTRACTED_FILE" || '
                    '-L "$EXTRACTED_FILE" ]]',
                    "stat --format='%h' --",
                    '/usr/bin/mkdir --mode=0700 -- "$DESTINATION_DIRECTORY"',
                    "install \\\n  --no-target-directory \\\n  -m 0755 \\\n  -- \\\n",
                    '[[ ! -f "$INSTALLED_FILE" || ! -s "$INSTALLED_FILE" || '
                    '! -x "$INSTALLED_FILE" || -L "$INSTALLED_FILE" ]]',
                    "stat --format='%a:%h' --",
                    '/usr/bin/cmp --silent -- "$EXTRACTED_FILE" "$INSTALLED_FILE"',
                ):
                    self.assertIn(required, script)
                self.assertNotIn("install \\\n  -D \\\n", script)
                self.assertEqual(
                    script.count('/usr/bin/mkdir --mode=0700 -- "$DESTINATION_DIRECTORY'),
                    2 if name == "install-lychee" else 1,
                )
                if name == "install-lychee":
                    self.assertIn(
                        '/usr/bin/mkdir --mode=0700 -- "$DESTINATION_DIRECTORY/bin"',
                        script,
                    )
                missing = subprocess.run(
                    [source_root / "scripts/ci" / name],
                    cwd=source_root,
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
                self.assertEqual(missing.returncode, 2)
                self.assertEqual(missing.stdout, "")
                self.assertEqual(
                    missing.stderr,
                    f"usage: {name} DESTINATION_DIRECTORY\n",
                )
                rejected = subprocess.run(
                    [source_root / "scripts/ci" / name, "relative-destination"],
                    cwd=source_root,
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
                self.assertEqual(rejected.returncode, 2)
                self.assertEqual(rejected.stdout, "")
                self.assertIn("DESTINATION_DIRECTORY must be absolute", rejected.stderr)
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    existing = root / "existing"
                    existing.mkdir()
                    occupied = subprocess.run(
                        [source_root / "scripts/ci" / name, existing],
                        cwd=source_root,
                        check=False,
                        capture_output=True,
                        text=True,
                        timeout=5,
                    )
                    dangling = root / "dangling"
                    dangling.symlink_to(root / "missing", target_is_directory=True)
                    redirected = subprocess.run(
                        [source_root / "scripts/ci" / name, dangling],
                        cwd=source_root,
                        check=False,
                        capture_output=True,
                        text=True,
                        timeout=5,
                    )
                for result in (occupied, redirected):
                    self.assertEqual(result.returncode, 2)
                    self.assertEqual(result.stdout, "")
                    self.assertEqual(
                        result.stderr,
                        "DESTINATION_DIRECTORY must not already exist\n",
                    )

    def test_external_link_helper_clears_ambient_environment(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        helper = source_root / "scripts/ci/check-external-links"
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            probe = temporary_root / "probe"
            observed = temporary_root / "environment.json"
            probe.write_text(
                "#!/usr/bin/python3\n"
                "import json\n"
                "import os\n"
                "import sys\n"
                "from pathlib import Path\n"
                "Path(__file__).with_name('environment.json').write_text(\n"
                "    json.dumps({\n"
                "        'arguments': sys.argv[1:],\n"
                "        'environment': dict(os.environ),\n"
                "    }),\n"
                "    encoding='utf-8',\n"
                ")\n",
                encoding="utf-8",
            )
            probe.chmod(0o755)
            missing = subprocess.run(
                [helper],
                cwd=source_root,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(missing.returncode, 2)
            self.assertEqual(missing.stdout, "")
            self.assertEqual(
                missing.stderr,
                "usage: check-external-links PATH_TO_LYCHEE\n",
            )
            rejected = subprocess.run(
                [helper, f"PATH={temporary_root}"],
                cwd=source_root,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(rejected.returncode, 2)
            self.assertEqual(rejected.stdout, "")
            self.assertIn("PATH_TO_LYCHEE must be absolute", rejected.stderr)
            self.assertFalse(observed.exists())

            empty = temporary_root / "empty"
            empty.touch(mode=0o755)
            nonexecutable = temporary_root / "nonexecutable"
            nonexecutable.write_text("probe\n", encoding="utf-8")
            redirected = temporary_root / "redirected"
            redirected.symlink_to(probe)
            for invalid_path in (temporary_root / "missing", empty, nonexecutable, redirected):
                invalid = subprocess.run(
                    [helper, invalid_path],
                    cwd=source_root,
                    capture_output=True,
                    text=True,
                    check=False,
                )
                self.assertEqual(invalid.returncode, 2)
                self.assertEqual(invalid.stdout, "")
                self.assertEqual(
                    invalid.stderr,
                    "PATH_TO_LYCHEE must be a nonempty executable regular file\n",
                )

            wrong_mode = temporary_root / "wrong-mode"
            wrong_mode.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            wrong_mode.chmod(0o700)
            mode_rejected = subprocess.run(
                [helper, wrong_mode],
                cwd=source_root,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(mode_rejected.returncode, 2)
            self.assertEqual(mode_rejected.stdout, "")
            self.assertEqual(
                mode_rejected.stderr,
                "PATH_TO_LYCHEE must have mode 0755 and one link\n",
            )

            linked_source = temporary_root / "linked-source"
            linked_source.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
            linked_source.chmod(0o755)
            linked_alias = temporary_root / "linked-alias"
            linked_alias.hardlink_to(linked_source)
            linked = subprocess.run(
                [helper, linked_alias],
                cwd=source_root,
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(linked.returncode, 2)
            self.assertEqual(linked.stdout, "")
            self.assertEqual(
                linked.stderr,
                "PATH_TO_LYCHEE must have mode 0755 and one link\n",
            )

            result = subprocess.run(
                [helper, probe],
                cwd=source_root,
                env={
                    "BASH_ENV": str(temporary_root / "hostile-startup"),
                    "HOME": str(temporary_root / "hostile-home"),
                    "HTTPS_PROXY": "http://hostile.invalid",
                    "LYCHEE_CONFIG": str(temporary_root / "hostile.toml"),
                    "PATH": str(temporary_root),
                    "RUST_LOG": "trace",
                },
                capture_output=True,
                text=True,
                check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            observation = json.loads(observed.read_text(encoding="utf-8"))
            self.assertEqual(
                observation["environment"],
                {
                    "LANG": "C",
                    "LC_ALL": "C",
                    "PATH": "/usr/bin:/bin",
                    "TZ": "UTC",
                },
            )
            self.assertEqual(
                observation["arguments"],
                [
                    "--exclude",
                    r"^https://eprint\.iacr\.org/",
                    "--exclude-all-private",
                    "--extensions",
                    "md,yml",
                    "--host-concurrency",
                    "2",
                    "--include-fragments",
                    "--max-concurrency",
                    "16",
                    "--max-retries",
                    "3",
                    "--no-progress",
                    "--require-https",
                    "--timeout",
                    "20",
                    "--",
                    ".",
                    ".github/**/*.md",
                    ".github/**/*.yml",
                ],
            )

    def test_hosted_run_steps_use_fixed_privileged_bash(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        shell = "shell: /bin/bash -p -e -o pipefail {0}"
        for name in ("ci.yml", "external-links.yml", "scorecard.yml"):
            with self.subTest(workflow=name):
                workflow = (source_root / ".github/workflows" / name).read_text(
                    encoding="utf-8"
                )
                self.assertEqual(workflow.count(shell), 1)
                self.assertNotIn("shell: bash\n", workflow)
        ci = (source_root / ".github/workflows/ci.yml").read_text(encoding="utf-8")
        self.assertIn(
            'run: /usr/bin/env -i HOME="$HOME" LANG=C LC_ALL=C PATH="$PATH" TZ=UTC '
            "rustup toolchain install 1.96.1 --profile minimal "
            "--component clippy,rustfmt --no-self-update",
            ci,
        )
        self.assertIn('pycache="$(/usr/bin/mktemp -d -- ', ci)
        self.assertIn('pycache="$(CDPATH= cd -- "$pycache" && pwd -P)"', ci)
        self.assertIn("trap '/usr/bin/rm -rf -- \"$pycache\"' EXIT", ci)
        self.assertIn(
            "-u MFLAGS /usr/bin/make --no-builtin-rules --no-builtin-variables",
            ci,
        )


class PolicyShapeHardeningTests(unittest.TestCase):
    def test_malformed_nested_policy_values_fail_closed_without_crashing(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        baseline = load_json(source_root / "policy/gate0-repository-policy.json")
        mutations = (
            (
                "allowed top-level item",
                lambda policy: policy["allowed_top_level_paths"].__setitem__(0, []),
            ),
            (
                "required path item",
                lambda policy: policy["required_paths"].__setitem__(0, []),
            ),
            (
                "Action repository item",
                lambda policy: policy["github_actions"]["allowed_action_repositories"].__setitem__(0, []),
            ),
            (
                "write-permission mapping",
                lambda policy: policy["github_actions"].__setitem__("allowed_write_permissions", []),
            ),
            (
                "forbidden-event list",
                lambda policy: policy["github_actions"].__setitem__("forbidden_events", None),
            ),
            (
                "NUL path",
                lambda policy: policy["required_paths"].__setitem__(0, "unsafe\0path"),
            ),
            (
                "NUL binary path",
                lambda policy: policy["allowed_binary_artifacts"][0].__setitem__("path", "unsafe\0path"),
            ),
            (
                "extra NUL forbidden path",
                lambda policy: policy["forbidden_paths"].append("unsafe\0path"),
            ),
            (
                "duplicate top-level path",
                lambda policy: policy["allowed_top_level_paths"].append(
                    policy["allowed_top_level_paths"][0]
                ),
            ),
            (
                "duplicate Action repository",
                lambda policy: policy["github_actions"]["allowed_action_repositories"].append(
                    policy["github_actions"]["allowed_action_repositories"][0]
                ),
            ),
            (
                "duplicate write permission",
                lambda policy: policy["github_actions"]["allowed_write_permissions"][
                    "scorecard.yml"
                ].append("security-events"),
            ),
        )
        for name, mutate in mutations:
            with self.subTest(name=name):
                policy = json.loads(json.dumps(baseline))
                mutate(policy)
                validator = FoundationValidator(source_root)
                with mock.patch.object(validator, "_load_repository_json", return_value=policy):
                    findings = validator.run()
                self.assertTrue(any(finding.code.startswith("policy.") for finding in findings))
                self.assertEqual(validator.policy, {})


class RepositoryInventoryHardeningTests(unittest.TestCase):
    def test_cli_pipe_closure_fails_quietly_for_reports_help_and_errors(self) -> None:
        repository_root = Path(__file__).resolve().parents[2]
        cases = (
            ((), "stdout"),
            (("--format", "json"), "stdout"),
            (("--help",), "stdout"),
            (("--invalid",), "stderr"),
        )
        for arguments, closed_name in cases:
            with self.subTest(arguments=arguments, closed_name=closed_name):
                process = subprocess.Popen(
                    [
                        sys.executable,
                        "-I",
                        "-S",
                        "-B",
                        "-X",
                        "utf8",
                        str(repository_root / "tools/validate_foundation.py"),
                        *arguments,
                    ],
                    cwd=repository_root,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                closed = getattr(process, closed_name)
                captured = process.stderr if closed_name == "stdout" else process.stdout
                self.assertIsNotNone(closed)
                self.assertIsNotNone(captured)
                closed.close()
                output = captured.read()
                captured.close()

                self.assertEqual(process.wait(timeout=10), 1)
                self.assertEqual(output, b"")

    def test_json_report_streams_without_materializing_the_serialization(self) -> None:
        validator = mock.Mock()
        validator.policy = {"repository": "owner/repository", "policy_version": "test"}
        validator.run.return_value = [Finding("path.test", "record.or", "m\u00e9ssage\n")]
        output = io.StringIO()
        with (
            mock.patch(
                "tools.validate_foundation.FoundationValidator",
                return_value=validator,
            ),
            mock.patch(
                "tools.validate_foundation.json.dumps",
                side_effect=AssertionError("JSON report was materialized"),
            ),
            redirect_stdout(output),
        ):
            status = main(("--format", "json"))

        self.assertEqual(status, 1)
        self.assertEqual(
            output.getvalue(),
            '{"findings":[{"code":"path.test","message":"m\\u00e9ssage\\n",'
            '"path":"record.or"}],"policy_version":"test","repository":'
            '"owner/repository","schema_version":"0.1.0","valid":false}\n',
        )

    def test_text_report_escapes_untrusted_fields_injectively_on_one_line(self) -> None:
        validator = mock.Mock()
        validator.policy = {}
        validator.run.return_value = [
            Finding("path.test", "safe:path\\name\nforged\x1bé", "message\rsecond\\line")
        ]
        output = io.StringIO()
        with (
            mock.patch(
                "tools.validate_foundation.FoundationValidator",
                return_value=validator,
            ),
            redirect_stdout(output),
        ):
            status = main(())

        self.assertEqual(status, 1)
        self.assertEqual(
            output.getvalue().splitlines(),
            [
                "safe\\U0000003apath\\U0000005cname\\U0000000aforged"
                "\\U0000001b\\U000000e9: "
                "path.test: message\\U0000000dsecond\\U0000005cline",
                "Solo-bootstrap repository policy failed with 1 finding(s).",
            ],
        )

    def test_finding_messages_are_bounded_with_a_truncation_marker(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            validator = FoundationValidator(Path(directory))
            validator.add(
                "synthetic.finding",
                ".",
                "x" * (GATE0_MAXIMUM_FINDING_MESSAGE_CHARACTERS + 100),
            )

        message = validator.findings[0].message
        self.assertEqual(len(message), GATE0_MAXIMUM_FINDING_MESSAGE_CHARACTERS)
        self.assertTrue(message.endswith("... [truncated]"))

    def test_finding_retention_is_bounded_with_one_suppression_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            validator = FoundationValidator(Path(directory))
            for index in range(GATE0_MAXIMUM_FINDINGS + 100):
                validator.add("synthetic.finding", f"path-{index}", "synthetic")

        self.assertEqual(len(validator.findings), GATE0_MAXIMUM_FINDINGS + 1)
        self.assertEqual(validator.findings[0].path, "path-0")
        self.assertEqual(validator.findings[-1].code, "resource.finding_count")
        self.assertIn(str(GATE0_MAXIMUM_FINDINGS), validator.findings[-1].message)

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
                "-I",
                "-S",
                "-B",
                "-X",
                "utf8",
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

    def test_manifest_path_validation_is_lexical_and_does_not_follow_symlinks(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            parent = Path(directory)
            root = parent / "root"
            outside = parent / "outside"
            root.mkdir()
            outside.mkdir()
            (root / "link").symlink_to(outside, target_is_directory=True)

            self.assertEqual(
                safe_manifest_path(root, "link/record.json"),
                root / "link/record.json",
            )
            self.assertIsNone(safe_manifest_path(root, "../outside/record.json"))
            self.assertIsNone(safe_manifest_path(root, str(outside / "record.json")))
            for value in (
                ".",
                "record.json/",
                "nested//record.json",
                "nested/./record.json",
                "C:record.json",
                "nested\\record.json",
                "nested/record\n.json",
            ):
                with self.subTest(value=value):
                    self.assertIsNone(safe_manifest_path(root, value))

    def test_validator_directory_queries_reuse_the_bounded_inventory(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "tools/validate_foundation.py").read_text(encoding="utf-8")

        self.assertNotRegex(source, r"\.(?:rglob|glob|iterdir)\(")

    def test_validator_presence_queries_do_not_stat_worktree_paths(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = (source_root / "tools/validate_foundation.py").read_text(encoding="utf-8")
        validator_source = source.split("class FoundationValidator:", 1)[1]

        self.assertNotRegex(
            validator_source,
            r"\.(?:exists|is_file|is_dir|is_symlink|lstat)\(",
        )

    def test_required_and_forbidden_paths_use_inventory_without_stat(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            existing = root / "docs/existing.md"
            existing.parent.mkdir()
            existing.write_text("# Existing\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator.policy = {
                "allowed_top_level_paths": ["docs"],
                "required_paths": ["docs/existing.md", "docs/missing.md"],
                "forbidden_paths": ["docs", "absent"],
            }

            with mock.patch.object(
                Path, "exists", side_effect=AssertionError("worktree path was statted")
            ), mock.patch.object(
                Path, "is_file", side_effect=AssertionError("worktree path was statted")
            ), mock.patch.object(
                Path, "is_dir", side_effect=AssertionError("worktree path was statted")
            ), mock.patch.object(
                Path, "is_symlink", side_effect=AssertionError("worktree path was statted")
            ):
                validator._validate_required_and_forbidden_paths()

        path_findings = {
            (finding.code, finding.path)
            for finding in validator.findings
            if finding.code in {"path.required", "path.forbidden"}
        }
        self.assertEqual(
            path_findings,
            {
                ("path.required", "docs/missing.md"),
                ("path.forbidden", "docs"),
            },
        )

    def test_inventory_directory_selection_is_lexical_and_depth_bounded(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            validator = object.__new__(FoundationValidator)
            validator.root = root
            validator.repository_files = [
                root / "schemas/gate0/z.txt",
                root / "schemas/gate00/outside.schema.json",
                root / "schemas/gate0/nested/hidden.schema.json",
                root / "schemas/gate0/accepted.schema.json",
            ]

            self.assertEqual(
                [
                    relative(path, root)
                    for path in validator._inventory_files_in("schemas/gate0")
                ],
                ["schemas/gate0/accepted.schema.json", "schemas/gate0/z.txt"],
            )
            self.assertEqual(
                [
                    relative(path, root)
                    for path in validator._inventory_files_in(
                        "schemas/gate0", recursive=True
                    )
                ],
                [
                    "schemas/gate0/accepted.schema.json",
                    "schemas/gate0/nested/hidden.schema.json",
                    "schemas/gate0/z.txt",
                ],
            )

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
missing_docs = "deny"
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "deny"

[profile.release]
debug-assertions = true
overflow-checks = true
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
            lambda source: source.replace('missing_docs = "deny"', 'missing_docs = "allow"'),
            lambda source: source.replace('unsafe_code = "forbid"', 'unsafe_code = "allow"'),
            lambda source: source.replace('all = "deny"', 'all = "warn"'),
            lambda source: source.replace("debug-assertions = true", "debug-assertions = false"),
            lambda source: source.replace("overflow-checks = true", "overflow-checks = false"),
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
        temporary_root = tempfile.TemporaryDirectory()
        self.addCleanup(temporary_root.cleanup)
        validator = FoundationValidator(Path(temporary_root.name))
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
        "compiler/crates/orangec/src/main.rs",
        "compiler/README.md",
        "docs/LANGUAGE_2026.md",
        "docs/SEMANTICS_2026.md",
        "docs/operations/CI_DEPENDENCIES.md",
        "policy/README.md",
        "scripts/ci/check-external-links",
        "scripts/ci/install-actionlint",
        "scripts/ci/install-lychee",
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

    def test_installer_shell_budget_drift_is_rejected(self) -> None:
        mutations = (
            (
                "scripts/ci/install-actionlint",
                'readonly MAXIMUM_ARCHIVE_BYTES="33554432"',
                'readonly MAXIMUM_ARCHIVE_BYTES="33554431"',
            ),
            (
                "scripts/ci/install-lychee",
                'readonly MAXIMUM_DOWNLOAD_SECONDS="300"',
                'readonly MAXIMUM_DOWNLOAD_SECONDS="299"',
            ),
        )
        for value, old, new in mutations:
            with self.subTest(path=value), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_boundary(root)
                path = root / value
                source = path.read_text(encoding="utf-8")
                self.assertIn(old, source)
                path.write_text(source.replace(old, new, 1), encoding="utf-8")
                self.assertIn("ci.installer_budget", self._codes(root))

    def test_installer_budget_documentation_drift_is_rejected(self) -> None:
        mutations = (
            ("the 32 MiB archive cap", "the 31 MiB archive cap"),
            ("caps the extracted member at 128 MiB", "caps the extracted member at 127 MiB"),
        )
        for old, new in mutations:
            with self.subTest(marker=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_boundary(root)
                path = root / "docs/operations/CI_DEPENDENCIES.md"
                source = path.read_text(encoding="utf-8")
                self.assertIn(old, source)
                path.write_text(source.replace(old, new, 1), encoding="utf-8")
                self.assertIn("ci.installer_spec_budget", self._codes(root))

    def test_installer_budget_use_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "scripts/ci/install-actionlint"
            source = path.read_text(encoding="utf-8")
            old = '--max-time "$MAXIMUM_DOWNLOAD_SECONDS"'
            self.assertIn(old, source)
            path.write_text(source.replace(old, "--max-time 300", 1), encoding="utf-8")
            self.assertIn("ci.installer_budget", self._codes(root))

    def test_external_link_runner_budget_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "scripts/ci/check-external-links"
            source = path.read_text(encoding="utf-8")
            old = 'readonly MAXIMUM_RETRIES="3"'
            self.assertIn(old, source)
            path.write_text(
                source.replace(old, 'readonly MAXIMUM_RETRIES="4"', 1),
                encoding="utf-8",
            )
            self.assertIn("ci.external_link_budget", self._codes(root))

    def test_external_link_runner_use_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "scripts/ci/check-external-links"
            source = path.read_text(encoding="utf-8")
            old = '--timeout "$REQUEST_TIMEOUT_SECONDS"'
            self.assertIn(old, source)
            path.write_text(source.replace(old, "--timeout 20", 1), encoding="utf-8")
            self.assertIn("ci.external_link_budget", self._codes(root))

    def test_external_link_runner_documentation_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "docs/operations/CI_DEPENDENCIES.md"
            source = path.read_text(encoding="utf-8")
            old = "per-host concurrency at 2"
            self.assertIn(old, source)
            path.write_text(source.replace(old, "per-host concurrency at 3", 1), encoding="utf-8")
            self.assertIn("ci.external_link_spec_budget", self._codes(root))

    def test_workflow_timeout_documentation_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "docs/operations/CI_DEPENDENCIES.md"
            source = path.read_text(encoding="utf-8")
            old = "`dependency-review.yml` permits\n10 minutes"
            self.assertIn(old, source)
            path.write_text(source.replace(old, "`dependency-review.yml` permits\n11 minutes", 1), encoding="utf-8")
            self.assertIn("ci.workflow_timeout_spec", self._codes(root))

    def test_dependency_review_documentation_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "docs/operations/CI_DEPENDENCIES.md"
            source = path.read_text(encoding="utf-8")
            old = "snapshot warnings for at most 120 seconds"
            self.assertIn(old, source)
            path.write_text(source.replace(old, "snapshot warnings for at most 121 seconds", 1), encoding="utf-8")
            self.assertIn("ci.dependency_review_spec", self._codes(root))

    def test_dependabot_documentation_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "docs/operations/CI_DEPENDENCIES.md"
            source = path.read_text(encoding="utf-8")
            old = "permits at most five open\nupdate pull requests"
            self.assertIn(old, source)
            path.write_text(
                source.replace(old, "permits at most six open\nupdate pull requests", 1),
                encoding="utf-8",
            )
            self.assertIn("ci.dependabot_spec", self._codes(root))

    def test_markdownlint_documentation_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "docs/operations/CI_DEPENDENCIES.md"
            source = path.read_text(encoding="utf-8")
            old = "Markdown lint ignores only `compiler/target/**`"
            self.assertIn(old, source)
            path.write_text(
                source.replace(old, "Markdown lint ignores only `**`", 1),
                encoding="utf-8",
            )
            self.assertIn("ci.markdownlint_spec", self._codes(root))

    def test_rust_source_stripping_never_copies_remaining_suffixes(self) -> None:
        class SliceRejectingString(str):
            def __getitem__(self, key: object) -> str:
                if isinstance(key, slice):
                    raise AssertionError("source suffix was copied")
                return super().__getitem__(key)

        source = SliceRejectingString(
            ("a" * 262_144)
            + ' r###"hidden\nraw"### /* nested /* block */ comment */ "string" // line\ncode'
        )
        stripped = rust_code_without_comments_and_literals(source)
        self.assertEqual(len(stripped), len(source))
        self.assertEqual(stripped.count("\n"), source.count("\n"))
        self.assertNotIn("hidden", stripped)
        self.assertTrue(stripped.endswith("\ncode"))

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
            ("compiler/crates/orangec/src/main.rs", "usize = 256;", "usize = 255;"),
            (
                "compiler/crates/orangec/src/main.rs",
                "MAX_ARGUMENT_BYTES_PER_INVOCATION: usize = 4 * 1024 * 1024",
                "MAX_ARGUMENT_BYTES_PER_INVOCATION: usize = 3 * 1024 * 1024",
            ),
            (
                "compiler/crates/orangec/src/main.rs",
                "MAX_SOURCE_BYTES_PER_INVOCATION: usize = 64 * 1024 * 1024",
                "MAX_SOURCE_BYTES_PER_INVOCATION: usize = 63 * 1024 * 1024",
            ),
            (
                "compiler/crates/orangec/src/main.rs",
                "MAX_STANDARD_OUTPUT_BYTES: usize = 64 * 1024 * 1024",
                "MAX_STANDARD_OUTPUT_BYTES: usize = 63 * 1024 * 1024",
            ),
            (
                "compiler/crates/orangec/src/main.rs",
                "MAX_STANDARD_ERROR_BYTES: usize = 64 * 1024 * 1024",
                "MAX_STANDARD_ERROR_BYTES: usize = 63 * 1024 * 1024",
            ),
        )
        for value, old, new in mutations:
            with self.subTest(path=value, old=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_boundary(root)
                path = root / value
                source = path.read_text(encoding="utf-8")
                self.assertIn(old, source)
                path.write_text(source.replace(old, new, 1), encoding="utf-8")
                expected = "compiler.cli_budget" if value.endswith("orangec/src/main.rs") else "compiler.language_budget"
                self.assertIn(expected, self._codes(root))

    def test_cli_operational_limit_documentation_drift_is_rejected(self) -> None:
        for old, new in (
            ("up to 256 source inputs", "up to 255 source inputs"),
            (
                "inspects at most 4 MiB (`4 * 1024 * 1024` bytes) of encoded command-line",
                "inspects at most 3 MiB (`3 * 1024 * 1024` bytes) of encoded command-line",
            ),
            (
                "buffers at most\n64 MiB (`64 * 1024 * 1024` bytes)",
                "buffers at most\n63 MiB (`63 * 1024 * 1024` bytes)",
            ),
            (
                "caps standard output at 64 MiB (`64 * 1024 * 1024` bytes)",
                "caps standard output at 63 MiB (`63 * 1024 * 1024` bytes)",
            ),
            (
                "caps standard error at 64 MiB (`64 * 1024 * 1024` bytes)",
                "caps standard error at 63 MiB (`63 * 1024 * 1024` bytes)",
            ),
        ):
            with self.subTest(marker=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_boundary(root)
                path = root / "compiler/README.md"
                source = path.read_text(encoding="utf-8")
                path.write_text(source.replace(old, new, 1), encoding="utf-8")
                self.assertIn("compiler.cli_spec_budget", self._codes(root))

    def test_repository_resource_documentation_drift_is_rejected(self) -> None:
        for old, new in (
            ("256 KiB (`256 * 1024` bytes)", "255 KiB (`255 * 1024` bytes)"),
            ("384 KiB\n(`384 * 1024` bytes)", "383 KiB\n(`383 * 1024` bytes)"),
            ("2 MiB (`2 * 1024 * 1024` bytes)", "1 MiB (`1 * 1024 * 1024` bytes)"),
            ("12 MiB (`12 * 1024 * 1024` bytes)", "11 MiB (`11 * 1024 * 1024` bytes)"),
            ("at most 512 files", "at most 511 files"),
            ("at most 1,024 bytes per raw path", "at most 1,023 bytes per raw path"),
            (
                "at most 1 MiB\n(`1024 * 1024` bytes) of raw path metadata",
                "at most 2 MiB\n(`2048 * 1024` bytes) of raw path metadata",
            ),
            (
                "at most 4,096 entries in one\nfallback directory",
                "at most 4,095 entries in one\nfallback directory",
            ),
            ("One\n30-second deadline", "One\n29-second deadline"),
            (
                "each retain at most 4,096 detailed findings",
                "each retain at most 4,095 detailed findings",
            ),
            (
                "Final finding messages retain at\nmost 4,096 characters",
                "Final finding messages retain at\nmost 4,095 characters",
            ),
            (
                "Each stage-zero metadata prefix is\ncapped at 128 bytes",
                "Each stage-zero metadata prefix is\ncapped at 127 bytes",
            ),
            (
                "Structural JSON nesting is capped at 64 levels",
                "Structural JSON nesting is capped at 63 levels",
            ),
        ):
            with self.subTest(marker=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_boundary(root)
                path = root / "policy/README.md"
                source = path.read_text(encoding="utf-8")
                self.assertIn(old, source)
                path.write_text(source.replace(old, new, 1), encoding="utf-8")
                self.assertIn("policy.resource_budget", self._codes(root))

    def test_oversized_compiled_budget_is_rejected_without_crashing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_boundary(root)
            path = root / "compiler/crates/orange-compiler/src/source.rs"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("16 * 1024 * 1024", "9" * 4301, 1),
                encoding="utf-8",
            )
            self.assertIn("compiler.language_budget", self._codes(root))

    def test_rust_usize_product_is_bounded_to_64_bits(self) -> None:
        maximum = (1 << 64) - 1
        self.assertEqual(parse_rust_usize_product("18_446_744_073_709_551_615"), maximum)
        self.assertEqual(parse_rust_usize_product("000000000000000000001"), 1)
        self.assertIsNone(parse_rust_usize_product("18_446_744_073_709_551_616"))
        self.assertIsNone(parse_rust_usize_product("9_223_372_036_854_775_808 * 2"))
        self.assertIsNone(parse_rust_usize_product("9" * 4301))

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
            validator.policy = protected_file_policy()
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
    def test_gitattributes_contract_drift_is_rejected(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        mutations = (
            ("*.wasm binary !eol", "*.wasm text eol=lf"),
            ("* text=auto eol=lf", "* text=auto eol=crlf"),
        )
        for old, new in mutations:
            with self.subTest(rule=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                source = (source_root / ".gitattributes").read_text(encoding="utf-8")
                self.assertIn(old, source)
                (root / ".gitattributes").write_text(
                    source.replace(old, new, 1),
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_brand_assets()
                self.assertIn(
                    "gitattributes.contract",
                    {finding.code for finding in validator.findings},
                )

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

    def test_unhashable_brand_manifest_path_is_rejected_without_crashing(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            shutil.copytree(source_root / "assets/brand", root / "assets/brand")
            manifest_path = root / "assets/brand/manifest.json"
            manifest = load_json(manifest_path)
            manifest["assets"][0]["path"] = []
            manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_brand_assets()
            self.assertIn("brand.manifest_item", {finding.code for finding in validator.findings})

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
        if not (source_root / ".git").exists():
            attributes = set(
                (source_root / ".gitattributes").read_text(encoding="utf-8").splitlines()
            )
            binary_rules = {
                ".jpeg": "*.[jJ][pP][eE][gG] binary !eol",
                ".jpg": "*.[jJ][pP][gG] binary !eol",
                ".png": "*.[pP][nN][gG] binary !eol",
            }
            self.assertTrue(paths)
            for path in paths:
                rule = binary_rules.get(Path(path).suffix.lower())
                self.assertIsNotNone(rule, path)
                self.assertIn(rule, attributes, path)
            return
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

    def test_reader_entrypoints_use_only_the_hand_drawn_banner(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        readme = (source_root / "README.md").read_text(encoding="utf-8")
        orange_book = (source_root / "docs/THE_ORANGE_BOOK.md").read_text(encoding="utf-8")
        readme_banner = (
            "![Hand-drawn Orange carton emblem and wordmark]"
            "(assets/brand/orange-handdrawn-marker-banner.png)"
        )
        book_banner = (
            "![Hand-drawn Orange carton emblem and wordmark]"
            "(../assets/brand/orange-handdrawn-marker-banner.png)"
        )
        self.assertEqual(readme.count(readme_banner), 1)
        self.assertEqual(orange_book.count(book_banner), 1)
        self.assertNotIn("user-attachments/assets", readme)
        self.assertNotIn("user-attachments/assets", orange_book)

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
    def test_make_check_must_be_policy_first_serialized_and_environment_isolated(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        canonical = (source_root / "Makefile").read_text(encoding="utf-8")
        contract = (source_root / "policy/makefile-entrypoint-contract-v0.1.json").read_bytes()
        mutations = (
            (".NOTPARALLEL: check\n", "", "make.entrypoint_contract"),
            ("override .SHELLFLAGS := -p -c\n", "", "make.entrypoint_contract"),
            ("unexport BASH_ENV ENV\n", "", "make.entrypoint_contract"),
            ("umask 077; \\\n", "umask 022; \\\n", "make.compiler_environment_contract"),
            (
                "check: check-policy check-compiler",
                "check: check-compiler check-policy",
                "make.entrypoint_contract",
            ),
            ("env -i", "env", "make.compiler_environment_contract"),
            (
                'cargo_home="$$(CDPATH= cd -- "$$cargo_home" && pwd -P)"',
                'cargo_home="$$cargo_home"',
                "make.compiler_environment_contract",
            ),
            (
                '/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-repro-home.XXXXXXXX"',
                '/usr/bin/mktemp -d -- "$${TMPDIR:-/tmp}/orange-cargo-home.XXXXXXXX"',
                "make.compiler_environment_contract",
            ),
            (
                'trap \'/usr/bin/rm -rf -- "$$cargo_home" "$$repro_home_b"\' EXIT;',
                'trap \'/usr/bin/rm -rf -- "$$cargo_home"\' EXIT;',
                "make.compiler_environment_contract",
            ),
            (
                'repro_home_b="$$(CDPATH= cd -- "$$repro_home_b" && pwd -P)"',
                'repro_home_b="$$cargo_home"',
                "make.compiler_environment_contract",
            ),
            (
                'trap \'/usr/bin/rm -rf -- "$$cargo_home" "$$repro_home_b" '
                '"$$gate_tools"\' EXIT;',
                'trap \'/usr/bin/rm -rf -- "$$cargo_home" "$$repro_home_b"\' EXIT;',
                "make.compiler_environment_contract",
            ),
            (
                'toolchain_root="$$(CDPATH= cd -- "$$toolchain_root" && pwd -P)"',
                'toolchain_root="$$toolchain_root"',
                "make.compiler_environment_contract",
            ),
            (
                "exec 8<&- 9<&-;",
                "/usr/bin/true;",
                "make.compiler_environment_contract",
            ),
            (
                "/usr/bin/unshare \\\n",
                "/usr/bin/env \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--map-current-user \\\n",
                "\t\t--map-auto \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--keep-caps \\\n",
                "",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--mount-proc \\\n",
                "\t\t--mount-proc=/host/proc \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--kill-child=KILL \\\n",
                "\t\t--kill-child=TERM \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--net \\\n",
                "",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--ipc \\\n",
                "",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--uts \\\n",
                "",
                "make.compiler_environment_contract",
            ),
            (
                "/usr/bin/hostname orange-gate;",
                "/usr/bin/true;",
                "make.compiler_environment_contract",
            ),
            (
                'gate_ipc_namespace="$$(/usr/bin/readlink -- /proc/self/ns/ipc)"',
                'gate_ipc_namespace="ipc:[0]"',
                "make.compiler_environment_contract",
            ),
            (
                'gate_uts_namespace="$$(/usr/bin/readlink -- /proc/self/ns/uts)"',
                'gate_uts_namespace="uts:[0]"',
                "make.compiler_environment_contract",
            ),
            (
                "remount,bind,ro,nosuid,nodev",
                "remount,bind,rw,nosuid,nodev",
                "make.compiler_environment_contract",
            ),
            (
                "mode=755,nosuid,nodev,noexec tmpfs /home",
                "mode=755,nosuid,nodev,noexec tmpfs /mnt",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t\t/usr/bin/sudo \\\n",
                "\t\t\t/usr/bin/true \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t\t--clear-groups \\\n",
                "\t\t\t--keep-groups \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "--bounding-set=-all \\\n",
                "--bounding-set=+all \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "--inh-caps=-all \\\n",
                "--inh-caps=+all \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "--ambient-caps=-all \\\n",
                "--ambient-caps=+all \\\n",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t--no-new-privs \\\n",
                "",
                "make.compiler_environment_contract",
            ),
            (
                '"$$gate_tools/fs-sandbox" \\\n',
                '/bin/true \\\n',
                "make.compiler_environment_contract",
            ),
            (
                "\t\t\t\t--ro /usr \\\n",
                "",
                "make.compiler_environment_contract",
            ),
            (
                "\t\t\t\t--rw \"$$cargo_home\" \\\n",
                "",
                "make.compiler_environment_contract",
            ),
            (
                'HOME="$$cargo_home/home"',
                'HOME="$$HOME"',
                "make.compiler_environment_contract",
            ),
            (
                'PATH="$$gate_tools/toolchain/bin:/usr/bin:/bin"',
                'PATH="$$PATH"',
                "make.compiler_environment_contract",
            ),
            (
                'TMPDIR="$$cargo_home/tmp"',
                'TMPDIR="/tmp"',
                "make.compiler_environment_contract",
            ),
            (
                '/usr/bin/cc -std=c17 -O2 -D_FORTIFY_SOURCE=3 -fPIE -pie '
                '-Wall -Wextra -Werror -pedantic -Wl,-z,relro,-z,now',
                "/usr/bin/cc -std=c17",
                "make.compiler_environment_contract",
            ),
            (
                "run_cargo /bin/bash -p -c 'for capability_set in CapInh CapPrm CapEff CapBnd CapAmb; do [[ \"$$(/usr/bin/sed -n \"s/^$${capability_set}:[[:space:]]*//p\" /proc/self/status)\" == 0000000000000000 ]] || exit 1; done; for descriptor in /proc/self/fd/*; do [[ ! -e \"$$descriptor\" || \"$${descriptor##*/}\" =~ ^[012]$$ ]] || exit 1; done; for ipc_table in msg sem shm; do [[ -z \"$$(/usr/bin/sed -n \"2p\" \"/proc/sysvipc/$$ipc_table\")\" ]] || exit 1; done; ! /usr/bin/head -c 1 -- \"$$3/Makefile\" >/dev/null 2>&1 || exit 1; [[ $$$$ == 1 && $$PPID == 0 && \"$$(/usr/bin/id -u)\" == \"$$1\" && \"$$(/usr/bin/id -g)\" == \"$$2\" && \"$$(/usr/bin/hostname)\" == orange-gate && \"$$(/usr/bin/readlink -- /proc/self/ns/ipc)\" != \"$$6\" && \"$$(/usr/bin/readlink -- /proc/self/ns/uts)\" != \"$$7\" && \"$$HOME\" == \"$$4\" && \"$$PATH\" == \"$$5/toolchain/bin:/usr/bin:/bin\" && \"$$(/usr/bin/sed -n \"s/^NoNewPrivs:[[:space:]]*//p\" /proc/self/status)\" == 1 && -z \"$$(/usr/bin/sed -n \"2p\" /proc/net/route)\" ]]' gate-isolation \"$$gate_uid\" \"$$gate_gid\" \"$$repository_root\" \"$$cargo_home/home\" \"$$gate_tools\" \"$$gate_ipc_namespace\" \"$$gate_uts_namespace\"",
                "/usr/bin/true",
                "make.compiler_environment_contract",
            ),
            (
                "RUSTUP_TOOLCHAIN=1.96.1",
                "RUSTUP_TOOLCHAIN=stable",
                "make.compiler_environment_contract",
            ),
            (
                "--all-targets --release --locked --offline",
                "--all-targets --locked --offline",
                "make.compiler_environment_contract",
            ),
            (
                "-D clippy::arithmetic_side_effects",
                "-W clippy::arithmetic_side_effects",
                "make.compiler_environment_contract",
            ),
            (
                'CARGO_TARGET_DIR="$$repro_home_b/deep/target"',
                'CARGO_TARGET_DIR="$$cargo_home/target-a"',
                "make.compiler_environment_contract",
            ),
            (
                'CARGO_HOME="$$repro_home_b/cargo"',
                'CARGO_HOME="$$cargo_home"',
                "make.compiler_environment_contract",
            ),
            (
                'copy_compiler_source "$$repro_home_b/deep/src"',
                'copy_compiler_source "$$cargo_home/repro-a"',
                "make.compiler_environment_contract",
            ),
            (
                '/usr/bin/mkdir -- "$$repro_home_b/deep"; \\\n',
                "",
                "make.compiler_environment_contract",
            ),
            (
                'artifact_b="$$repro_home_b/deep/target/release/orangec"',
                'artifact_b="$$cargo_home/target-a/release/orangec"',
                "make.compiler_environment_contract",
            ),
            (
                '[[ -f "$$artifact" && ! -L "$$artifact" ]]',
                '[[ -e "$$artifact" ]]',
                "make.compiler_environment_contract",
            ),
            (
                'artifact_b_mode="$$(/usr/bin/stat --format=%a -- "$$artifact_b")"',
                'artifact_b_mode="$$(/usr/bin/stat --format=%a -- "$$artifact_a")"',
                "make.compiler_environment_contract",
            ),
            (
                '[[ "$$artifact_a_mode" == "$$artifact_b_mode" ]]',
                "[[ 0 == 0 ]]",
                "make.compiler_environment_contract",
            ),
            (
                "filecmp.cmp(sys.argv[1], sys.argv[2], shallow=False)",
                "filecmp.cmp(sys.argv[1], sys.argv[2], shallow=True)",
                "make.compiler_environment_contract",
            ),
            (
                'tested_roots=("$$cargo_home/check-src" "$$cargo_home/repro-a" "$$repro_home_b/deep/src")',
                'tested_roots=("$$cargo_home/check-src" "$$cargo_home/repro-a")',
                "make.compiler_environment_contract",
            ),
            (
                'copy_compiler_source "$$cargo_home/check-src"',
                'copy_compiler_source "$$cargo_home/repro-a"',
                "make.compiler_environment_contract",
            ),
            (
                "run_cargo /usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 "
                "-S -P -B -X utf8 -W error::ResourceWarning "
                '"$$cargo_home/check-src/tools/validate_foundation.py"',
                "/usr/bin/true",
                "make.python_environment_contract",
            ),
            (
                "PYTHONPYCACHEPREFIX=\"$$cargo_home/snapshot-python-cache\" "
                "/usr/bin/python3 -S -P -B -X utf8 -W error::ResourceWarning "
                "-c 'import sys, unittest; sys.path.insert(0, sys.argv.pop(1)); "
                "unittest.main(module=None)' \"$$cargo_home/check-src\" discover",
                "/usr/bin/true",
                "make.python_environment_contract",
            ),
            (
                'ls-files --cached -z > "$$capture_paths_path"',
                'ls-files --others -z > "$$capture_paths_path"',
                "make.compiler_environment_contract",
            ),
            (
                'ls-files --cached -z > "$$repro_source_paths_after"',
                'ls-files --others -z > "$$repro_source_paths_after"',
                "make.compiler_environment_contract",
            ),
            (
                "--sort=name --mtime=@0",
                "--sort=none --mtime=@1",
                "make.compiler_environment_contract",
            ),
            (
                "--mode='u+rwX,go+rX,go-w,u-s,g-s,o-t'",
                "--mode='a+rwx'",
                "make.compiler_environment_contract",
            ),
            (
                "--hard-dereference --null",
                "--null",
                "make.compiler_environment_contract",
            ),
            (
                "--null --verbatim-files-from --no-recursion",
                "--null --files-from",
                "make.compiler_environment_contract",
            ),
            (
                'exec 8<"$$capture_paths_path"; \\\n',
                "exec 8</dev/null; \\\n",
                "make.compiler_environment_contract",
            ),
            (
                'exec 9<"$$capture_archive_path"; \\\n',
                "exec 9</dev/null; \\\n",
                "make.compiler_environment_contract",
            ),
            (
                '/usr/bin/rm -- "$$capture_paths_path" "$$capture_archive_path"',
                "/usr/bin/true --",
                "make.compiler_environment_contract",
            ),
            (
                '--extract --file="$$repro_source_archive"',
                "--extract --file=-",
                "make.compiler_environment_contract",
            ),
            (
                '/usr/bin/cmp --silent -- "$$repository_root/$$relative_path"',
                '/usr/bin/true -- "$$repository_root/$$relative_path"',
                "make.compiler_environment_contract",
            ),
            (
                '! -L "$$cargo_home/check-src/$$relative_path" ]]',
                '-e "$$cargo_home/check-src/$$relative_path" ]]',
                "make.compiler_environment_contract",
            ),
            (
                'live_executable="$$(( (8#$$live_mode & 0111) != 0 ))"',
                'live_executable="$$((8#$$live_mode & 0111))"',
                "make.compiler_environment_contract",
            ),
            (
                '[[ "$$live_executable" == "$$snapshot_executable" ]]',
                "[[ 0 == 0 ]]",
                "make.compiler_environment_contract",
            ),
            (
                '/usr/bin/cmp --silent -- "$$repro_source_paths"',
                '/usr/bin/true -- "$$repro_source_paths"',
                "make.compiler_environment_contract",
            ),
            (
                '[[ "$$(/usr/bin/sha256sum --binary -- "$$repro_source_archive")" '
                '== "$$repro_source_archive_identity" ]]',
                '[[ "$$repro_source_archive_identity" == "$$repro_source_archive_identity" ]]',
                "make.compiler_environment_contract",
            ),
            (
                '[[ "$$(/usr/bin/sha256sum --binary -- "$$repro_source_paths")" '
                '== "$$repro_source_paths_identity" ]]',
                '[[ "$$repro_source_paths_identity" == "$$repro_source_paths_identity" ]]',
                "make.compiler_environment_contract",
            ),
            (
                'verify_capture_identity; \\\n\tcopy_compiler_source "$$cargo_home/check-reference"',
                'copy_compiler_source "$$cargo_home/check-reference"',
                "make.compiler_environment_contract",
            ),
            (
                'done < "$$repro_source_paths"; \\\n\tverify_capture_identity',
                'done < "$$repro_source_paths"',
                "make.compiler_environment_contract",
            ),
            (
                'CARGO_TARGET_DIR="$$cargo_home/target"',
                'CARGO_TARGET_DIR="compiler/target"',
                "make.compiler_environment_contract",
            ),
            ("PYTHONHASHSEED=0", "PYTHONHASHSEED=random", "make.python_environment_contract"),
            (
                "python3 -S -P -B -X utf8",
                "python3 -S",
                "make.python_environment_contract",
            ),
            (
                "/usr/bin/python3 -S -P -B -X utf8",
                "python3 -S -P -B -X utf8",
                "make.python_environment_contract",
            ),
            (
                'PYTHONPYCACHEPREFIX="$$pycache"',
                "",
                "make.python_cache_contract",
            ),
            (
                "-W error::ResourceWarning",
                "",
                "make.python_environment_contract",
            ),
            (
                "-W error::ResourceWarning tools/validate_foundation.py",
                "tools/validate_foundation.py",
                "make.python_environment_contract",
            ),
            (
                "-W error::ResourceWarning -c 'import filecmp, sys",
                "-c 'import filecmp, sys",
                "make.python_environment_contract",
            ),
            (
                "/usr/bin/env PYTHONHASHSEED=0 /usr/bin/python3 -S",
                "/usr/bin/python3 -S",
                "make.python_environment_contract",
            ),
            (
                'pycache="$$(CDPATH= cd -- "$$pycache" && pwd -P)"',
                'pycache="$$pycache"',
                "make.python_cache_contract",
            ),
        )
        for old, new, expected_code in mutations:
            with self.subTest(mutation=old), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "Makefile").write_text(canonical.replace(old, new), encoding="utf-8")
                contract_path = root / "policy/makefile-entrypoint-contract-v0.1.json"
                contract_path.parent.mkdir()
                contract_path.write_bytes(contract)
                validator = FoundationValidator(root)
                validator._validate_makefile_entrypoint()
                codes = {finding.code for finding in validator.findings}
                self.assertIn(expected_code, codes)
                self.assertNotIn("make.contract", codes)

    def test_filesystem_sandbox_denies_unadmitted_files_and_closes_descriptors(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        source = source_root / "tools/fs_sandbox.c"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            binary = root / "fs-sandbox"
            allowed = root / "allowed"
            denied = root / "denied"
            allowed.mkdir()
            denied.mkdir()
            (allowed / "input").write_text("allowed\n", encoding="utf-8")
            (denied / "secret").write_text("denied\n", encoding="utf-8")
            (allowed / "escape").symlink_to(denied / "secret")
            subprocess.run(
                [
                    "/usr/bin/cc",
                    "-std=c17",
                    "-O2",
                    "-D_FORTIFY_SOURCE=3",
                    "-fPIE",
                    "-pie",
                    "-Wall",
                    "-Wextra",
                    "-Werror",
                    "-pedantic",
                    "-Wl,-z,relro,-z,now",
                    os.fspath(source),
                    "-o",
                    os.fspath(binary),
                ],
                check=True,
                capture_output=True,
                text=True,
            )
            inherited = os.open(denied / "secret", os.O_RDONLY)
            try:
                result = subprocess.run(
                    [
                        os.fspath(binary),
                        "--dir",
                        "/",
                        "--ro",
                        "/usr",
                        "--ro",
                        "/etc",
                        "--ro",
                        "/proc",
                        "--rw",
                        "/dev/null",
                        "--rw",
                        os.fspath(allowed),
                        "--",
                        "/bin/bash",
                        "-p",
                        "-c",
                        (
                            'set -euo pipefail; [[ "$(cat input)" == allowed ]]; '
                            "printf sandboxed > output; "
                            "! cat ../denied/secret >/dev/null 2>&1; "
                            "! cat escape >/dev/null 2>&1; "
                            "! printf escaped > ../denied/output 2>/dev/null; "
                            '[[ ! -e "/proc/self/fd/$1" ]]'
                        ),
                        "sandbox",
                        str(inherited),
                    ],
                    cwd=allowed,
                    env={"LANG": "C", "LC_ALL": "C", "PATH": "/usr/bin:/bin"},
                    pass_fds=(inherited,),
                    capture_output=True,
                    text=True,
                    check=False,
                )
            finally:
                os.close(inherited)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((allowed / "output").read_text(encoding="utf-8"), "sandboxed")
            self.assertFalse((denied / "output").exists())

    def test_make_contract_rejects_an_invalid_root(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Makefile").write_bytes((source_root / "Makefile").read_bytes())
            contract_path = root / "policy/makefile-entrypoint-contract-v0.1.json"
            contract_path.parent.mkdir()
            contract_path.write_text("{}\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_makefile_entrypoint()
            self.assertIn("make.contract", {finding.code for finding in validator.findings})

    def test_make_contract_enforces_compiler_phase_order(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        canonical = (source_root / "Makefile").read_text(encoding="utf-8")
        contract = (source_root / "policy/makefile-entrypoint-contract-v0.1.json").read_bytes()
        earlier = 'run_cargo cargo fmt --manifest-path "$$manifest" --all -- --check; \\\n'
        later = (
            'verify_capture_identity; \\\n'
            '\tcopy_compiler_source "$$cargo_home/check-reference"; \\\n'
        )
        marker = "__ORANGE_ORDERED_CONTRACT_MARKER__\n"
        reordered = canonical.replace(earlier, marker, 1).replace(later, earlier, 1).replace(marker, later, 1)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Makefile").write_text(reordered, encoding="utf-8")
            contract_path = root / "policy/makefile-entrypoint-contract-v0.1.json"
            contract_path.parent.mkdir()
            contract_path.write_bytes(contract)
            validator = FoundationValidator(root)
            validator._validate_makefile_entrypoint()
            self.assertIn(
                "make.compiler_environment_contract",
                {finding.code for finding in validator.findings},
            )

    def test_make_contract_rejects_repeated_ordered_fragments(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        canonical = load_json(source_root / "policy/makefile-entrypoint-contract-v0.1.json")
        ordered = next(check for check in canonical["checks"] if check["match"] == "ordered")
        ordered["expected_count"] = 2
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Makefile").write_bytes((source_root / "Makefile").read_bytes())
            contract_path = root / "policy/makefile-entrypoint-contract-v0.1.json"
            contract_path.parent.mkdir()
            contract_path.write_text(json.dumps(canonical), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_makefile_entrypoint()
            self.assertIn("make.contract", {finding.code for finding in validator.findings})

    def test_make_contract_rejects_duplicate_keys(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        canonical = (source_root / "policy/makefile-entrypoint-contract-v0.1.json").read_text(
            encoding="utf-8"
        )
        duplicate = canonical.replace(
            '  "schema_version": "0.1.0",',
            '  "schema_version": "0.1.0",\n  "schema_version": "0.1.0",',
            1,
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "Makefile").write_bytes((source_root / "Makefile").read_bytes())
            contract_path = root / "policy/makefile-entrypoint-contract-v0.1.json"
            contract_path.parent.mkdir()
            contract_path.write_text(duplicate, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_makefile_entrypoint()
            self.assertIn("make.contract", {finding.code for finding in validator.findings})

    def test_make_contract_rejects_unhashable_check_fields(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        canonical = load_json(source_root / "policy/makefile-entrypoint-contract-v0.1.json")
        for field, value in (("finding_code", []), ("match", {})):
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                (root / "Makefile").write_bytes((source_root / "Makefile").read_bytes())
                contract = json.loads(json.dumps(canonical))
                contract["checks"][0][field] = value
                contract_path = root / "policy/makefile-entrypoint-contract-v0.1.json"
                contract_path.parent.mkdir()
                contract_path.write_text(json.dumps(contract), encoding="utf-8")
                validator = FoundationValidator(root)
                validator._validate_makefile_entrypoint()
                self.assertIn("make.contract", {finding.code for finding in validator.findings})

    def test_codeowners_and_fixture_mutations_are_digest_protected(self) -> None:
        paths = (
            ".github/CODEOWNERS",
            "compiler/crates/orangec/tests/s3a_conformance.rs",
            "compiler/fixtures/s3a/invalid-word-range.or",
            "conformance/foundation/valid/claim-record.json",
            "policy/makefile-entrypoint-contract-v0.1.json",
        )
        for value in paths:
            with self.subTest(path=value), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                path = root / value
                path.parent.mkdir(parents=True)
                path.write_text("tampered\n", encoding="utf-8")
                validator = FoundationValidator(root)
                validator.policy = protected_file_policy()
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

    def test_policy_cannot_change_the_ordered_codeowners_contract(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        policy = load_json(source_root / "policy/gate0-repository-policy.json")
        policy["required_codeowners"].remove("/RELEASE_POLICY.md @chasebryan")
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "policy/gate0-repository-policy.json"
            path.parent.mkdir(parents=True)
            path.write_text(json.dumps(policy), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._load_and_validate_policy()
            codes = {finding.code for finding in validator.findings}
            self.assertIn("policy.codeowners_contract", codes)

    def test_policy_cannot_change_the_protected_digest_mapping(self) -> None:
        source_root = Path(__file__).resolve().parents[2]
        policy = load_json(source_root / "policy/gate0-repository-policy.json")
        policy["protected_file_digests"]["SECURITY.md"] = "0" * 64
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "policy/gate0-repository-policy.json"
            path.parent.mkdir(parents=True)
            path.write_text(json.dumps(policy), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._load_and_validate_policy()

        self.assertIn(
            "policy.protected_file_digests", {finding.code for finding in validator.findings}
        )


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

    def test_front_matter_error_retention_is_bounded(self) -> None:
        parsed = parse_front_matter("---\n" + "invalid syntax\n" * (GATE0_MAXIMUM_FINDINGS + 100))

        self.assertIsNotNone(parsed)
        assert parsed is not None
        _, errors = parsed
        self.assertEqual(len(errors), GATE0_MAXIMUM_FINDINGS)
        self.assertIn("line 2", errors[0])
        self.assertIn(f"line {GATE0_MAXIMUM_FINDINGS + 1}", errors[-1])

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

    def _copy_proof_suite(self, root: Path) -> Path:
        docs = root / "docs"
        docs.mkdir(exist_ok=True)
        source = Path(__file__).resolve().parents[2] / "docs/PROOF_FOUNDATION_DECISION_SUITE.md"
        target = docs / source.name
        shutil.copyfile(source, target)
        return target

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

    def test_oversized_journey_flow_ordinal_is_rejected_without_crashing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_planning_docs(root)
            path = root / "docs/USER_JOURNEYS.md"
            source = path.read_text(encoding="utf-8")
            path.write_text(
                source.replace("1. Select", f"{'9' * 4301}. Select", 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_user_journeys()
            self.assertIn("journey.flow_order", {finding.code for finding in validator.findings})

    def test_missing_proof_suite_case_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_proof_suite(root)
            text = target.read_text(encoding="utf-8")
            target.write_text(text.replace("### DS-07", "### DS-06", 1), encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_proof_foundation_suite()
            self.assertIn("proof_suite.case_ids", {finding.code for finding in validator.findings})

    def test_short_proof_candidate_rows_are_rejected_without_crashing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_proof_suite(root)
            text = target.read_text(encoding="utf-8")
            text = text.replace(
                "| C-01 | Rocq | Run the complete frozen suite with idiomatic, fully inventoried candidate artifacts | 0/7 cases |",
                "| C-01 |",
                1,
            ).replace(
                "| C-02 | Lean 4 | Run the complete frozen suite with idiomatic, fully inventoried candidate artifacts | 0/7 cases |",
                "| C-02 |",
                1,
            )
            target.write_text(text, encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_proof_foundation_suite()
            self.assertIn("proof_suite.candidates", {finding.code for finding in validator.findings})

    def test_oversized_proof_gate_ordinal_is_rejected_without_crashing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_proof_suite(root)
            text = target.read_text(encoding="utf-8")
            target.write_text(
                text.replace("1. DS-01", f"{'9' * 4301}. DS-01", 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_proof_foundation_suite()
            self.assertIn("proof_suite.hard_gates", {finding.code for finding in validator.findings})

    def test_zero_padded_planning_ordinals_remain_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_planning_docs(root)
            journey_path = root / "docs/USER_JOURNEYS.md"
            journeys = journey_path.read_text(encoding="utf-8")
            journey_path.write_text(
                journeys.replace("1. Select", "0001. Select", 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_user_journeys()
            self.assertIn("journey.flow_order", {finding.code for finding in validator.findings})

            proof_target = self._copy_proof_suite(root)
            proof_text = proof_target.read_text(encoding="utf-8")
            proof_target.write_text(
                proof_text.replace("1. DS-01", "0001. DS-01", 1),
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_proof_foundation_suite()
            self.assertIn("proof_suite.hard_gates", {finding.code for finding in validator.findings})

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
    def test_schema_issue_retention_is_bounded(self) -> None:
        schema_path = Path("/virtual/record.schema.json")
        schema = {
            "type": "object",
            "required": [f"missing-{index}" for index in range(GATE0_MAXIMUM_FINDINGS + 100)],
        }
        issues = validate_schema_instance({}, schema, schema_path, {schema_path: schema}, {})

        self.assertEqual(len(issues), GATE0_MAXIMUM_FINDINGS)
        self.assertIn("missing-0", issues[0].message)
        self.assertIn(f"missing-{GATE0_MAXIMUM_FINDINGS - 1}", issues[-1].message)

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
                "format": "hostname",
                "$id": "urn:orange:gate0:test#fragment",
                "properties": [],
            }
        )
        for fragment in ("type", "required", "enum", "minLength", "uniqueItems", "format", "$id", "properties"):
            self.assertTrue(any(fragment in finding for finding in findings), fragment)

        nested_id = audit_schema_vocabulary({"properties": {"x": {"$id": "urn:orange:gate0:x"}}})
        self.assertTrue(any("$id" in finding for finding in nested_id))
        nested_dialect = audit_schema_vocabulary({"properties": {"x": {"$schema": SCHEMA_DIALECT}}})
        self.assertTrue(any("$schema" in finding for finding in nested_dialect))

    def test_schema_profile_rejects_invalid_regular_expression(self) -> None:
        findings = audit_schema_vocabulary({"patternProperties": {"[": {"type": "string"}}})
        self.assertTrue(any("invalid patternProperties" in finding for finding in findings))


if __name__ == "__main__":
    unittest.main()
