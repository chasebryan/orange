from __future__ import annotations

import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.validate_foundation import (
    FoundationValidator,
    canonical_json_bytes,
    load_json,
    validate_cross_record_invariants,
    validate_schema_instance,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
PACKET_PATH = Path(
    "research/decisions/D-005/d005-v0.1/epochs/0001/protocol/epoch.json"
)
INPUT_ROOT = Path(
    "research/decisions/D-005/d005-v0.1/epochs/0001/shared-inputs"
)
MANIFEST_PATH = INPUT_ROOT / "legacy-v0.1-mutations.json"
SCHEMA_PATH = Path("schemas/gate0/claim-record-v0.1.schema.json")
PACKET_CANONICAL_SHA256 = (
    "731428229b4f77cd7e684e2a5cae51bdfd277898aaab60852b843d3183dbc194"
)


class D005DraftPacketTests(unittest.TestCase):
    def _validate_historical_record(self, path: Path) -> list[object]:
        schema_path = REPOSITORY_ROOT / SCHEMA_PATH
        schema = load_json(schema_path)
        instance = load_json(path)
        issues = validate_schema_instance(
            instance,
            schema,
            schema_path,
            {schema_path: schema},
            {schema["$id"]: (schema_path, schema)},
        )
        issues.extend(validate_cross_record_invariants(instance, schema_path.name))
        return issues

    def _copy_packet(self, root: Path) -> Path:
        target = root / PACKET_PATH
        target.parent.mkdir(parents=True)
        shutil.copyfile(REPOSITORY_ROOT / PACKET_PATH, target)
        return target

    def _copy_research_packet(self, root: Path) -> Path:
        source = REPOSITORY_ROOT / "research/decisions/D-005"
        target = root / "research/decisions/D-005"
        shutil.copytree(source, target)
        return root / MANIFEST_PATH

    def test_canonical_draft_packet_is_valid_and_records_no_execution(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(packet["status"], "draft")
        self.assertEqual(
            packet["execution"],
            {
                "required_candidate_cases": 32,
                "completed_candidate_cases": 0,
                "evidence_status": "none",
            },
        )
        self.assertIsNone(packet["selection"])
        canonical = canonical_json_bytes(packet)
        self.assertEqual(hashlib.sha256(canonical).hexdigest(), PACKET_CANONICAL_SHA256)

        validator = FoundationValidator(REPOSITORY_ROOT)
        validator._validate_d005_draft_packet()
        self.assertEqual(
            [
                finding
                for finding in validator.findings
                if finding.code.startswith("d005_packet.")
            ],
            [],
        )

    def test_packet_binds_exact_checked_in_shared_inputs(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(
            packet["mutation_manifest_sha256"],
            "8d069daf4a9443cf9df2d127f86d834e1aefed149324503f980c43f29c356082",
        )
        self.assertEqual(
            set(packet["input_bindings"]),
            {"decision_suite", "legacy_v01_manifest", "claim_record_v01_schema"},
        )
        for name, binding in packet["input_bindings"].items():
            with self.subTest(binding=name):
                bound_path = REPOSITORY_ROOT / binding["path"]
                self.assertEqual(
                    hashlib.sha256(bound_path.read_bytes()).hexdigest(),
                    binding["sha256"],
                )

    def test_packet_identity_inventory_and_zero_baseline_drift_are_rejected(self) -> None:
        mutations = (
            ("suite_version", "d005-v0.2", "d005_packet.digest"),
            ("candidates", ["AM-01", "AM-02", "AM-03"], "d005_packet.digest"),
            ("cases", [f"AC-{index:02d}" for index in range(1, 8)], "d005_packet.digest"),
            ("mutations", ["AC-01-M01"], "d005_packet.digest"),
            ("selection", "AM-01", "d005_packet.digest"),
        )
        for field, replacement, expected_code in mutations:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                target = self._copy_packet(root)
                packet = load_json(target)
                packet[field] = replacement
                target.write_text(
                    json.dumps(packet, indent=2, ensure_ascii=False) + "\n",
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_d005_draft_packet()
                self.assertIn(
                    expected_code,
                    {finding.code for finding in validator.findings},
                )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_packet(root)
            packet = load_json(target)
            packet["execution"]["completed_candidate_cases"] = 1
            packet["execution"]["evidence_status"] = "partial"
            target.write_text(json.dumps(packet, indent=2) + "\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_d005_draft_packet()
            self.assertIn(
                "d005_packet.digest",
                {finding.code for finding in validator.findings},
            )

    def test_packet_rejects_unknown_fields_and_premature_result_directories(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_packet(root)
            packet = load_json(target)
            packet["recommendation"] = "AM-01"
            target.write_text(json.dumps(packet, indent=2) + "\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_d005_draft_packet()
            self.assertIn(
                "d005_packet.digest",
                {finding.code for finding in validator.findings},
            )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_packet(root)
            result = (
                root
                / "research/decisions/D-005/d005-v0.1/epochs/0001"
                / "candidates/am-01/results.json"
            )
            result.parent.mkdir(parents=True)
            result.write_text("{}\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_d005_draft_packet()
            self.assertIn(
                "d005_packet.premature_results",
                {finding.code for finding in validator.findings},
            )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_research_packet(root)
            disguised_result = root / INPUT_ROOT / "am01-summary.json"
            disguised_result.write_text("{}\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_d005_draft_packet()
            self.assertIn(
                "d005_packet.research_inventory",
                {finding.code for finding in validator.findings},
            )

    def test_missing_packet_or_legacy_manifest_fails_closed(self) -> None:
        mutations = (
            (PACKET_PATH, "d005_packet.missing"),
            (MANIFEST_PATH, "d005_packet.legacy_missing"),
        )
        for removed_path, expected_code in mutations:
            with self.subTest(path=removed_path), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_research_packet(root)
                (root / removed_path).unlink()
                validator = FoundationValidator(root)
                validator._validate_d005_draft_packet()
                self.assertIn(
                    expected_code,
                    {finding.code for finding in validator.findings},
                )

    def test_all_five_historical_dangers_pass_v01_shape_and_second_pass(self) -> None:
        manifest = load_json(REPOSITORY_ROOT / MANIFEST_PATH)
        self.assertEqual(manifest["mutation_count"], 5)
        self.assertEqual(len(manifest["mutations"]), 5)
        self.assertEqual(
            [entry["mutation_id"] for entry in manifest["mutations"]],
            [
                "LV01-TEST-AS-REFINEMENT",
                "LV01-OPTIONAL-MASKS-FAILED-KERNEL",
                "LV01-UNRESOLVED-TARGET-LEAKAGE",
                "LV01-OWNER-AS-EXTERNAL",
                "LV01-SUBJECT-SUBSTITUTION",
            ],
        )
        for entry in manifest["mutations"]:
            with self.subTest(mutation=entry["mutation_id"]):
                fixture = REPOSITORY_ROOT / INPUT_ROOT / entry["fixture"]
                self.assertEqual(hashlib.sha256(fixture.read_bytes()).hexdigest(), entry["sha256"])
                self.assertEqual(self._validate_historical_record(fixture), [])

    def test_historical_schema_acceptance_is_rechecked_dynamically(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_research_packet(root)
            schema_path = root / SCHEMA_PATH
            schema_path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPOSITORY_ROOT / SCHEMA_PATH, schema_path)

            fixture_path = root / INPUT_ROOT / "checked-test-as-functional-refinement.json"
            fixture = load_json(fixture_path)
            fixture.pop("basis")
            fixture_path.write_text(
                json.dumps(fixture, indent=2) + "\n",
                encoding="utf-8",
            )

            validator = FoundationValidator(root)
            validator._validate_d005_draft_packet()
            codes = {finding.code for finding in validator.findings}
            self.assertIn("d005_packet.legacy_digest", codes)
            self.assertIn("d005_packet.legacy_acceptance", codes)

    def test_historical_manifest_drift_and_result_inflation_are_rejected(self) -> None:
        mutations = (
            (
                lambda manifest: manifest["mutations"][0].update({"sha256": "0" * 64}),
                "d005_packet.legacy_digest",
            ),
            (
                lambda manifest: manifest.update({"candidate_records": ["am-01.json"]}),
                "d005_packet.legacy_digest",
            ),
            (
                lambda manifest: manifest["mutations"][-1].update(
                    {"expected_changed_json_pointers": ["/subject/path"]}
                ),
                "d005_packet.legacy_digest",
            ),
            (
                lambda manifest: manifest.update({"recommendation": "AM-01"}),
                "d005_packet.legacy_digest",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(expected_code=expected_code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                manifest_path = self._copy_research_packet(root)
                manifest = load_json(manifest_path)
                mutate(manifest)
                manifest_path.write_text(
                    json.dumps(manifest, indent=2) + "\n",
                    encoding="utf-8",
                )
                validator = FoundationValidator(root)
                validator._validate_d005_draft_packet()
                self.assertIn(
                    expected_code,
                    {finding.code for finding in validator.findings},
                )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path = self._copy_research_packet(root)
            manifest_path.write_text("[]\n", encoding="utf-8")
            validator = FoundationValidator(root)
            validator._validate_d005_draft_packet()
            self.assertIn(
                "d005_packet.legacy_digest",
                {finding.code for finding in validator.findings},
            )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest_path = self._copy_research_packet(root)
            manifest = load_json(manifest_path)
            fixture_path = root / INPUT_ROOT / manifest["mutations"][0]["fixture"]
            fixture_path.write_bytes(fixture_path.read_bytes() + b"\n")
            manifest["mutations"][0]["sha256"] = hashlib.sha256(
                fixture_path.read_bytes()
            ).hexdigest()
            manifest_path.write_text(
                json.dumps(manifest, indent=2) + "\n",
                encoding="utf-8",
            )
            validator = FoundationValidator(root)
            validator._validate_d005_draft_packet()
            self.assertIn(
                "d005_packet.legacy_digest",
                {finding.code for finding in validator.findings},
            )

    def test_historical_records_exhibit_the_exact_authority_and_context_holes(self) -> None:
        input_root = REPOSITORY_ROOT / INPUT_ROOT
        test_only = load_json(input_root / "checked-test-as-functional-refinement.json")
        self.assertEqual(test_only["claim_kind"], "functional_refinement")
        self.assertEqual(test_only["outcome"], "satisfied")
        self.assertEqual(
            [(item["type"], item["verification_state"]) for item in test_only["basis"]],
            [("test_run", "checked")],
        )

        masked = load_json(input_root / "checked-test-masks-failed-kernel-proof.json")
        self.assertEqual(masked["outcome"], "satisfied")
        self.assertEqual(
            [(item["type"], item["verification_state"]) for item in masked["basis"]],
            [("kernel_proof", "failed"), ("test_run", "checked")],
        )

        leakage = load_json(
            input_root / "satisfied-target-leakage-with-unresolved-contexts.json"
        )
        self.assertEqual(leakage["claim_kind"], "target_leakage")
        self.assertEqual(leakage["outcome"], "satisfied")
        self.assertEqual(leakage["context"]["target_profile"]["state"], "unresolved")
        self.assertEqual(leakage["context"]["leakage_model"]["state"], "unresolved")

        owner_external = load_json(input_root / "owner-test-as-external-validation.json")
        self.assertEqual(owner_external["claim_kind"], "external_validation")
        self.assertEqual(
            [(item["type"], item["verification_state"]) for item in owner_external["basis"]],
            [("test_run", "checked")],
        )

    def test_subject_substitution_changes_only_subject_bytes_and_reuses_evidence(self) -> None:
        input_root = REPOSITORY_ROOT / INPUT_ROOT
        original = load_json(input_root / "subject-reuse-original.json")
        substituted = load_json(input_root / "substituted-subject-reuses-evidence.json")
        self.assertEqual(self._validate_historical_record(input_root / "subject-reuse-original.json"), [])
        self.assertEqual(
            self._validate_historical_record(
                input_root / "substituted-subject-reuses-evidence.json"
            ),
            [],
        )
        self.assertEqual(original["claim_id"], substituted["claim_id"])
        self.assertEqual(original["evidence_refs"], substituted["evidence_refs"])
        self.assertEqual(original["basis"], substituted["basis"])
        self.assertNotEqual(original["subject"]["path"], substituted["subject"]["path"])
        self.assertNotEqual(
            original["subject"]["digest"],
            substituted["subject"]["digest"],
        )

        normalized = copy.deepcopy(substituted)
        normalized["subject"] = copy.deepcopy(original["subject"])
        self.assertEqual(normalized, original)


if __name__ == "__main__":
    unittest.main()
