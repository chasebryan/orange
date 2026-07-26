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
RESEARCH_ROOT = Path("research/decisions/D-006")
PACKET_PATH = RESEARCH_ROOT / "d006-v0.2-draft-packet.json"
INDEX_PATH = RESEARCH_ROOT / "d006-v0.2-case-input-index.json"
SUITE_PATH = Path("docs/PROOF_FOUNDATION_DECISION_SUITE.md")
PACKET_CANONICAL_SHA256 = (
    "b56ad768c4584bdd00da4d4e85af642757b877dd5dc5ae438560ba4a486d9d21"
)
INDEX_CANONICAL_SHA256 = (
    "1118fe42a6d7111f50e40a88f0fe7b7fe4b9248b9335e0643b200fa983294ca0"
)
INDEX_RAW_SHA256 = "1aec6a731bef0620c8500120ec8385d584f99a528b4a03c014e8516c55cc8136"
SUITE_RAW_SHA256 = "6b1aa32784dd31d40bdaca4c6f3b62b8721a909ab3415051aa5a8e7994f0254b"


class D006DraftPacketTests(unittest.TestCase):
    @staticmethod
    def _write_canonical(path: Path, value: object) -> None:
        path.write_bytes(canonical_json_bytes(value) + b"\n")

    def _copy_lab(self, root: Path) -> tuple[Path, Path]:
        shutil.copytree(REPOSITORY_ROOT / RESEARCH_ROOT, root / RESEARCH_ROOT)
        suite_target = root / SUITE_PATH
        suite_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPOSITORY_ROOT / SUITE_PATH, suite_target)
        return root / PACKET_PATH, root / INDEX_PATH

    @staticmethod
    def _codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_d006_draft_packet()
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d006_packet.")
        }

    def test_canonical_pre_epoch_lab_is_valid_and_records_no_execution(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        index = load_json(REPOSITORY_ROOT / INDEX_PATH)
        canonical_packet = canonical_json_bytes(packet)
        canonical_index = canonical_json_bytes(index)

        self.assertEqual(
            (REPOSITORY_ROOT / PACKET_PATH).read_bytes(), canonical_packet + b"\n"
        )
        self.assertEqual(
            (REPOSITORY_ROOT / INDEX_PATH).read_bytes(), canonical_index + b"\n"
        )
        self.assertEqual(
            hashlib.sha256(canonical_packet).hexdigest(), PACKET_CANONICAL_SHA256
        )
        self.assertEqual(
            hashlib.sha256(canonical_index).hexdigest(), INDEX_CANONICAL_SHA256
        )
        self.assertEqual(
            hashlib.sha256((REPOSITORY_ROOT / INDEX_PATH).read_bytes()).hexdigest(),
            INDEX_RAW_SHA256,
        )
        self.assertEqual(
            hashlib.sha256((REPOSITORY_ROOT / SUITE_PATH).read_bytes()).hexdigest(),
            SUITE_RAW_SHA256,
        )
        self.assertEqual(packet["status"], "draft_unfrozen")
        self.assertIsNone(packet["epoch"])
        self.assertEqual(packet["epoch_status"], "unfrozen")
        self.assertEqual(packet["owner_protocol_review"], "none")
        self.assertEqual(packet["dependency_acceptance"], {"D-004": False, "D-005": False})
        self.assertEqual(
            packet["execution"],
            {
                "complete_candidates": 0,
                "complete_cross_candidate_cases": 0,
                "completed_candidate_cases": 0,
                "evidence_status": "none",
                "required_candidate_cases": 14,
            },
        )
        self.assertIsNone(packet["physical_execution_order"])
        self.assertIsNone(packet["selection"])
        self.assertIsNone(packet["conclusion"])
        self.assertEqual(packet["independent_review_status"], "unavailable")
        self.assertEqual(len(index["case_inputs"]), 7)
        self.assertTrue(
            all(
                type(record["executable_fixture_count"]) is int
                and record["executable_fixture_count"] == 0
                and type(record["freeze_blocker"]) is bool
                and record["freeze_blocker"]
                for record in index["case_inputs"]
            )
        )

        validator = FoundationValidator(REPOSITORY_ROOT)
        validator._validate_d006_draft_packet()
        self.assertEqual(
            [
                finding
                for finding in validator.findings
                if finding.code.startswith("d006_packet.")
            ],
            [],
        )

    def test_packet_binds_exact_index_and_suite_raw_bytes(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(
            packet["case_input_index_sha256"], INDEX_CANONICAL_SHA256
        )
        self.assertEqual(
            packet["input_bindings"],
            {
                "case_input_index": {
                    "path": str(INDEX_PATH),
                    "sha256": INDEX_RAW_SHA256,
                },
                "proof_foundation_suite": {
                    "path": str(SUITE_PATH),
                    "sha256": SUITE_RAW_SHA256,
                },
            },
        )
        for binding in packet["input_bindings"].values():
            bound = REPOSITORY_ROOT / binding["path"]
            self.assertEqual(hashlib.sha256(bound.read_bytes()).hexdigest(), binding["sha256"])

    def test_lifecycle_and_dependency_weakening_fail_closed(self) -> None:
        mutations = (
            ("schema_version", "d006-pre-epoch-packet-v0.2", "d006_packet.digest"),
            ("suite_version", "d006-v0.3-draft", "d006_packet.digest"),
            ("status", "frozen", "d006_packet.digest"),
            ("epoch", "0001", "d006_packet.digest"),
            ("epoch_status", "frozen", "d006_packet.digest"),
            ("owner_protocol_review", "solo-reviewed", "d006_packet.digest"),
        )
        for field, replacement, expected_code in mutations:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                packet_path, _ = self._copy_lab(root)
                packet = load_json(packet_path)
                packet[field] = replacement
                self._write_canonical(packet_path, packet)
                self.assertIn(expected_code, self._codes(root))

        dependency_mutations = (
            {"D-004": True, "D-005": False},
            {"D-004": 0, "D-005": False},
            {"D-004": False, "D-005": 0},
            {"D-004": False},
        )
        for replacement in dependency_mutations:
            with self.subTest(dependencies=replacement), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                packet_path, _ = self._copy_lab(root)
                packet = load_json(packet_path)
                packet["dependency_acceptance"] = replacement
                self._write_canonical(packet_path, packet)
                self.assertIn("d006_packet.digest", self._codes(root))

    def test_closed_packet_inventories_and_integer_types_fail_closed(self) -> None:
        mutations = (
            (
                lambda packet: packet.update({"unknown": True}),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["candidates"].reverse(),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["cases"].pop(),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["metrics"].append("M-19"),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet.update({"hard_gate_count": True}),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["laboratory_budgets"].update(
                    {"max_json_depth": True}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["protocol_counts"].update(
                    {"maximum_same_owner_reproducibility_level": 3}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["protocol_counts"].update(
                    {"owner_workspaces": True}
                ),
                "d006_packet.digest",
            ),
        )
        for index, (mutate, expected_code) in enumerate(mutations):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                packet_path, _ = self._copy_lab(root)
                packet = load_json(packet_path)
                mutate(packet)
                self._write_canonical(packet_path, packet)
                self.assertIn(expected_code, self._codes(root))

    def test_resource_tool_and_protocol_blocker_weakening_fail_closed(self) -> None:
        mutations = (
            (
                lambda packet: packet["execution_resource_state"].update(
                    {"case_wall_seconds": 900}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["execution_resource_state"].update(
                    {"contract_status": "assigned"}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["tool_states"][0].update(
                    {"version": "installed", "installation_status": "performed"}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["tool_states"][1].update(
                    {"admission_status": "accepted"}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["protocol_gaps"].pop(),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet.update({"nonclaims": []}),
                "d006_packet.digest",
            ),
        )
        for index, (mutate, expected_code) in enumerate(mutations):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                packet_path, _ = self._copy_lab(root)
                packet = load_json(packet_path)
                mutate(packet)
                self._write_canonical(packet_path, packet)
                self.assertIn(expected_code, self._codes(root))

    def test_execution_evidence_order_and_disposition_inflation_fail_closed(self) -> None:
        mutations = (
            (
                lambda packet: packet["execution"].update(
                    {"completed_candidate_cases": 1, "evidence_status": "partial"}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["execution"].update({"complete_candidates": True}),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet["execution"].update(
                    {"complete_cross_candidate_cases": 1}
                ),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet.update({"physical_execution_order": ["C-01"]}),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet.update({"independent_review_status": "complete"}),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet.update({"selection": "C-01"}),
                "d006_packet.digest",
            ),
            (
                lambda packet: packet.update({"conclusion": "recommend_rocq"}),
                "d006_packet.digest",
            ),
        )
        for index, (mutate, expected_code) in enumerate(mutations):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                packet_path, _ = self._copy_lab(root)
                packet = load_json(packet_path)
                mutate(packet)
                self._write_canonical(packet_path, packet)
                self.assertIn(expected_code, self._codes(root))

    def test_case_index_shape_inventory_and_blocker_weakening_fail_closed(self) -> None:
        mutations = (
            (
                lambda value: value.update({"unknown": True}),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"].pop(),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"].append(
                    copy.deepcopy(value["case_inputs"][0])
                ),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"].__setitem__(
                    slice(0, 2), [value["case_inputs"][1], value["case_inputs"][0]]
                ),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"][0].update(
                    {"executable_fixture_count": 1}
                ),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"][0].update(
                    {"executable_fixture_count": False}
                ),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"][0].update({"freeze_blocker": False}),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"][0].update({"freeze_blocker": 1}),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value["case_inputs"][0].update(
                    {"coverage_status": "complete"}
                ),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value.update({"executable_inputs_status": "present"}),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value.update({"evidence_status": "partial"}),
                "d006_packet.index_digest",
            ),
            (
                lambda value: value.update({"nonclaims": []}),
                "d006_packet.index_digest",
            ),
        )
        for index, (mutate, expected_code) in enumerate(mutations):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                _, index_path = self._copy_lab(root)
                case_index = load_json(index_path)
                mutate(case_index)
                self._write_canonical(index_path, case_index)
                self.assertIn(expected_code, self._codes(root))

    def test_raw_canonical_duplicate_key_and_float_drift_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet = load_json(packet_path)
            packet_path.write_text(json.dumps(packet, indent=2) + "\n", encoding="utf-8")
            self.assertIn("d006_packet.canonical", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            _, index_path = self._copy_lab(root)
            case_index = load_json(index_path)
            index_path.write_text(
                json.dumps(case_index, indent=2) + "\n", encoding="utf-8"
            )
            self.assertIn("d006_packet.index_canonical", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet_path.write_bytes(
                packet_path.read_bytes().replace(
                    b"{", b'{"status":"draft_unfrozen",', 1
                )
            )
            self.assertIn("d006_packet.parse", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet_path.write_bytes(
                packet_path.read_bytes().replace(
                    b'"max_json_depth":32', b'"max_json_depth":32.0', 1
                )
            )
            self.assertIn("d006_packet.parse", self._codes(root))

    def test_input_binding_and_self_rebinding_drift_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet = load_json(packet_path)
            packet["input_bindings"]["proof_foundation_suite"]["sha256"] = "0" * 64
            self._write_canonical(packet_path, packet)
            self.assertIn("d006_packet.digest", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            suite_path = root / SUITE_PATH
            suite_path.write_bytes(suite_path.read_bytes() + b"\n")
            self.assertIn("d006_packet.input_digest", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, index_path = self._copy_lab(root)
            case_index = load_json(index_path)
            case_index["case_inputs"][0]["coverage_status"] = "complete"
            self._write_canonical(index_path, case_index)

            packet = load_json(packet_path)
            packet["case_input_index_sha256"] = hashlib.sha256(
                canonical_json_bytes(case_index)
            ).hexdigest()
            packet["input_bindings"]["case_input_index"]["sha256"] = hashlib.sha256(
                index_path.read_bytes()
            ).hexdigest()
            self._write_canonical(packet_path, packet)

            codes = self._codes(root)
            self.assertIn("d006_packet.index_digest", codes)
            self.assertIn("d006_packet.digest", codes)

    def test_missing_extra_and_premature_research_paths_fail_closed(self) -> None:
        for removed, expected_code in (
            (PACKET_PATH, "d006_packet.parse"),
            (INDEX_PATH, "d006_packet.index_parse"),
        ):
            with self.subTest(removed=removed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                (root / removed).unlink()
                codes = self._codes(root)
                self.assertIn("d006_packet.research_inventory", codes)
                self.assertIn(expected_code, codes)

        extras = (
            RESEARCH_ROOT / "unregistered-input.json",
            RESEARCH_ROOT / "epochs/0001/protocol.json",
            RESEARCH_ROOT / "candidates/rocq/result.json",
            RESEARCH_ROOT / "results/summary.json",
            RESEARCH_ROOT / "result.json",
            RESEARCH_ROOT / "cross-candidate/comparison.json",
            RESEARCH_ROOT / "same-owner-replays/replay.json",
            RESEARCH_ROOT / "owner-reviews/R-01.json",
            RESEARCH_ROOT / "decision.json",
        )
        for extra in extras:
            with self.subTest(extra=extra), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / extra
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("{}\n", encoding="utf-8")
                codes = self._codes(root)
                self.assertIn("d006_packet.research_inventory", codes)
                if extra.name != "unregistered-input.json":
                    self.assertIn("d006_packet.premature_artifact", codes)


if __name__ == "__main__":
    unittest.main()
