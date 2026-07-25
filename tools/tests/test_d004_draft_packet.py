from __future__ import annotations

import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.validate_foundation import FoundationValidator, canonical_json_bytes, load_json


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RESEARCH_ROOT = Path("research/decisions/D-004")
PACKET_PATH = RESEARCH_ROOT / "d004-v0.2-draft-packet.json"
MANIFEST_PATH = RESEARCH_ROOT / "d004-v0.2-named-mutations.json"
PACKET_CANONICAL_SHA256 = (
    "6ab790957af9ff6b7dc1dc637800542280a229647367c5312d9b5ac4fd38fb87"
)
MANIFEST_CANONICAL_SHA256 = (
    "970999d998cdc202a6caa4e2f798017416c88211a5b6b8508132a07cc9080c0c"
)


class D004DraftPacketTests(unittest.TestCase):
    @staticmethod
    def _write_canonical(path: Path, value: object) -> None:
        path.write_bytes(canonical_json_bytes(value) + b"\n")

    def _copy_lab(self, root: Path) -> Path:
        shutil.copytree(REPOSITORY_ROOT / RESEARCH_ROOT, root / RESEARCH_ROOT)
        packet = load_json(root / PACKET_PATH)
        for binding in packet["input_bindings"].values():
            source = REPOSITORY_ROOT / binding["path"]
            target = root / binding["path"]
            if target.exists():
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        return root / PACKET_PATH

    @staticmethod
    def _codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_d004_draft_packet()
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d004_packet.")
        }

    def test_canonical_pre_epoch_lab_is_valid_and_records_no_execution(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        manifest = load_json(REPOSITORY_ROOT / MANIFEST_PATH)
        self.assertEqual(
            (REPOSITORY_ROOT / PACKET_PATH).read_bytes(),
            canonical_json_bytes(packet) + b"\n",
        )
        self.assertEqual(
            (REPOSITORY_ROOT / MANIFEST_PATH).read_bytes(),
            canonical_json_bytes(manifest) + b"\n",
        )
        self.assertEqual(
            packet["mutation_manifest_sha256"],
            hashlib.sha256(canonical_json_bytes(manifest)).hexdigest(),
        )
        self.assertEqual(
            hashlib.sha256(canonical_json_bytes(packet)).hexdigest(),
            PACKET_CANONICAL_SHA256,
        )
        self.assertEqual(
            hashlib.sha256(canonical_json_bytes(manifest)).hexdigest(),
            MANIFEST_CANONICAL_SHA256,
        )
        self.assertEqual(packet["status"], "draft_unfrozen")
        self.assertIsNone(packet["epoch"])
        self.assertEqual(packet["epoch_status"], "unfrozen")
        self.assertEqual(packet["d003_disposition"], "pending")
        self.assertEqual(packet["owner_protocol_review"], "none")
        self.assertEqual(
            packet["execution"],
            {
                "completed_candidate_cases": 0,
                "complete_candidates": 0,
                "complete_cross_candidate_cases": 0,
                "evidence_status": "none",
                "required_candidate_cases": 25,
            },
        )
        self.assertIsNone(packet["conclusion"])
        self.assertIsNone(packet["selection"])

        validator = FoundationValidator(REPOSITORY_ROOT)
        validator._validate_d004_draft_packet()
        self.assertEqual(
            [
                finding
                for finding in validator.findings
                if finding.code.startswith("d004_packet.")
            ],
            [],
        )

    def test_packet_binds_all_nineteen_exact_existing_inputs_by_raw_bytes(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(len(packet["input_bindings"]), 19)
        self.assertEqual(
            set(packet["input_bindings"]),
            {
                "accepted_s3a_oep",
                "accepted_s3a_semantics",
                "accepted_s2_language",
                "decision_suite",
                "fixture_invalid_duplicate_spec",
                "fixture_invalid_int_magnitude",
                "fixture_invalid_negative_word",
                "fixture_invalid_typed_impl",
                "fixture_invalid_unsupported_type",
                "fixture_invalid_word_range",
                "fixture_invalid_word_width",
                "fixture_valid_empty_mixed",
                "fixture_valid_int_radices",
                "fixture_valid_word8_boundaries",
                "named_mutations_manifest",
                "permanent_s3a_fixture",
                "product_form_decision_packet",
                "s3a_conformance_runner",
                "user_journeys",
            },
        )
        for name, binding in packet["input_bindings"].items():
            with self.subTest(binding=name):
                self.assertEqual(
                    hashlib.sha256(
                        (REPOSITORY_ROOT / binding["path"]).read_bytes()
                    ).hexdigest(),
                    binding["sha256"],
                )

    def test_lifecycle_owner_and_epoch_freeze_drift_are_rejected(self) -> None:
        mutations = (
            ("status", "frozen", "d004_packet.lifecycle"),
            ("epoch", "0001", "d004_packet.lifecycle"),
            ("epoch_status", "frozen", "d004_packet.lifecycle"),
            ("d003_disposition", "accepted", "d004_packet.lifecycle"),
            ("owner_protocol_review", "solo-reviewed", "d004_packet.lifecycle"),
            (
                "fixture_inventory_status",
                "complete",
                "d004_packet.protocol_gaps",
            ),
        )
        for field, replacement, expected_code in mutations:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                target = self._copy_lab(root)
                packet = load_json(target)
                packet[field] = replacement
                self._write_canonical(target, packet)
                self.assertIn(expected_code, self._codes(root))

    def test_exact_protocol_inventories_and_named_mutation_ids_are_closed(self) -> None:
        mutations = (
            ("candidates", ["ST-REL"], "d004_packet.inventory"),
            ("cases", ["SC-01"], "d004_packet.inventory"),
            ("relationships", ["SR-01"], "d004_packet.inventory"),
            ("hard_gates", ["SS-G01"], "d004_packet.inventory"),
            ("source_roles", ["Specification"], "d004_packet.inventory"),
            (
                "domain_observation_states",
                ["succeeded", "rejected"],
                "d004_packet.inventory",
            ),
            ("case_verdicts", ["pass"], "d004_packet.inventory"),
            ("mutations", ["SC-01-M01"], "d004_packet.mutations"),
        )
        for field, replacement, expected_code in mutations:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                target = self._copy_lab(root)
                packet = load_json(target)
                packet[field] = replacement
                self._write_canonical(target, packet)
                self.assertIn(expected_code, self._codes(root))

    def test_protocol_gap_deletion_nonzero_execution_and_disposition_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_lab(root)
            packet = load_json(target)
            packet["protocol_gaps"].pop()
            self._write_canonical(target, packet)
            self.assertIn("d004_packet.protocol_gaps", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_lab(root)
            packet = load_json(target)
            packet["unresolved_cross_cutting_fixture_classes"].pop()
            self._write_canonical(target, packet)
            self.assertIn("d004_packet.protocol_gaps", self._codes(root))

        mutations = (
            (
                lambda packet: packet["execution"].update(
                    {"completed_candidate_cases": 1, "evidence_status": "partial"}
                ),
                "d004_packet.execution",
            ),
            (
                lambda packet: packet["execution"].update({"complete_candidates": 1}),
                "d004_packet.execution",
            ),
            (
                lambda packet: packet["execution"].update(
                    {"complete_cross_candidate_cases": 1}
                ),
                "d004_packet.execution",
            ),
            (
                lambda packet: packet.update({"conclusion": "recommend_st_rel"}),
                "d004_packet.conclusion",
            ),
            (
                lambda packet: packet.update({"selection": "ST-REL"}),
                "d004_packet.selection",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                target = self._copy_lab(root)
                packet = load_json(target)
                mutate(packet)
                self._write_canonical(target, packet)
                self.assertIn(expected_code, self._codes(root))

    def test_research_allowlist_forbids_results_reviews_decisions_and_extra_paths(self) -> None:
        extras = (
            RESEARCH_ROOT / "results/ST-REL.json",
            RESEARCH_ROOT / "owner-reviews/review.json",
            RESEARCH_ROOT / "decision.json",
            RESEARCH_ROOT / "unregistered-input.json",
        )
        for extra in extras:
            with self.subTest(path=extra), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / extra
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("{}\n", encoding="utf-8")
                codes = self._codes(root)
                self.assertIn("d004_packet.research_inventory", codes)
                if any(
                    segment in {"results", "owner-reviews", "decision.json"}
                    for segment in extra.parts
                ):
                    self.assertIn("d004_packet.premature_artifact", codes)

    def test_missing_packet_or_manifest_fails_closed(self) -> None:
        for removed, expected_code in (
            (PACKET_PATH, "d004_packet.missing"),
            (MANIFEST_PATH, "d004_packet.manifest_missing"),
        ):
            with self.subTest(path=removed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                (root / removed).unlink()
                self.assertIn(expected_code, self._codes(root))

    def test_named_manifest_missing_unknown_duplicate_weakened_and_digest_drift_fail(self) -> None:
        mutations = (
            (
                lambda manifest: manifest[0].update({"unexpected": True}),
                "d004_packet.manifest_shape",
            ),
            (
                lambda manifest: manifest.append(copy.deepcopy(manifest[0])),
                "d004_packet.manifest_inventory",
            ),
            (
                lambda manifest: manifest[0].update(
                    {"description": "accept implicit conversion"}
                ),
                "d004_packet.manifest_digest",
            ),
            (
                lambda manifest: manifest[0].update({"case": "SC-05"}),
                "d004_packet.manifest_inventory",
            ),
            (
                lambda manifest: manifest[0].update({"case": []}),
                "d004_packet.manifest_inventory",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / MANIFEST_PATH
                manifest = load_json(target)
                mutate(manifest)
                self._write_canonical(target, manifest)
                self.assertIn(expected_code, self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_lab(root)
            packet = load_json(target)
            packet["mutation_manifest_sha256"] = "0" * 64
            self._write_canonical(target, packet)
            self.assertIn("d004_packet.manifest_digest", self._codes(root))

    def test_noncanonical_json_duplicate_keys_and_floats_are_rejected_not_crashed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / MANIFEST_PATH
            manifest = load_json(target)
            target.write_text(
                json.dumps(manifest, ensure_ascii=False) + "\n",
                encoding="utf-8",
            )
            self.assertIn("d004_packet.manifest_canonical", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_lab(root)
            raw = target.read_bytes()
            needle = b'"schema_version":'
            target.write_bytes(
                raw.replace(
                    needle,
                    b'"schema_version":"duplicate","schema_version":',
                    1,
                )
            )
            self.assertIn("d004_packet.parse", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_lab(root)
            packet = load_json(target)
            packet["budgets"]["case_wall_seconds"] = 1.5
            target.write_text(
                json.dumps(packet, ensure_ascii=False, separators=(",", ":")) + "\n",
                encoding="utf-8",
            )
            self.assertTrue(
                {"d004_packet.parse", "d004_packet.canonical"}
                & self._codes(root)
            )

    def test_bound_input_raw_byte_drift_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_lab(root)
            packet = load_json(target)
            binding = packet["input_bindings"]["accepted_s3a_semantics"]
            bound_path = root / binding["path"]
            bound_path.write_bytes(bound_path.read_bytes() + b"\n")
            self.assertIn("d004_packet.input_digest", self._codes(root))


if __name__ == "__main__":
    unittest.main()
