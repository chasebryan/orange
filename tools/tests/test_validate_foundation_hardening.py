from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.validate_foundation import (
    FoundationValidator,
    audit_schema_vocabulary,
    canonical_json_bytes,
    checkout_disables_credentials,
    load_json,
    validate_schema_instance,
)


def workflow_policy() -> dict[str, object]:
    return {
        "required_workflows": ["ci.yml"],
        "github_actions": {
            "allowed_action_repositories": ["actions/checkout"],
            "allowed_container_actions": [
                "docker://ghcr.io/ossf/scorecard-action@sha256:"
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

    def test_tag_selected_container_action_is_rejected(self) -> None:
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
      - name: Mutable container
        uses: docker://ghcr.io/ossf/scorecard-action:v2.4.3
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = workflow_policy()
            validator._validate_workflows()
            self.assertIn("workflow.mutable_container", {finding.code for finding in validator.findings})

    def test_exact_admitted_container_digest_passes_identity_checks(self) -> None:
        digest_action = (
            "docker://ghcr.io/ossf/scorecard-action@sha256:"
            "2dd6a6d60100f78ef24e14a47941d0087a524b4d3642041558239b1c6097c941"
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            workflow_dir = root / ".github/workflows"
            workflow_dir.mkdir(parents=True)
            (workflow_dir / "ci.yml").write_text(
                f"""name: CI
on:
  pull_request:
  push:
  merge_group:
permissions: {{}}
concurrency:
  group: ci
  cancel-in-progress: true
jobs:
  check:
    runs-on: ubuntu-24.04
    timeout-minutes: 5
    permissions: {{}}
    steps:
      - name: Digest container
        uses: {digest_action} # v2.4.3
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = workflow_policy()
            validator._validate_workflows()
            identity_codes = {
                finding.code
                for finding in validator.findings
                if finding.code in {
                    "workflow.action_allowlist",
                    "workflow.mutable_action",
                    "workflow.mutable_container",
                    "workflow.container_allowlist",
                    "workflow.uses_syntax",
                }
            }
            self.assertEqual(identity_codes, set())

    def test_unadmitted_container_digest_is_rejected(self) -> None:
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
      - name: Unadmitted digest
        uses: docker://ghcr.io/ossf/scorecard-action@sha256:0000000000000000000000000000000000000000000000000000000000000000 # v2.4.3
""",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator.policy = workflow_policy()
            validator._validate_workflows()
            self.assertIn("workflow.container_allowlist", {finding.code for finding in validator.findings})


class RepositoryInventoryHardeningTests(unittest.TestCase):
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


class ProtectedControlHardeningTests(unittest.TestCase):
    def test_codeowners_and_fixture_mutations_are_digest_protected(self) -> None:
        paths = (
            ".github/CODEOWNERS",
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
