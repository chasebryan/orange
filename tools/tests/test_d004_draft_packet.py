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
PROPOSAL_MANIFEST_PATH = (
    RESEARCH_ROOT / "d004-v0.2-cross-cutting-fixture-proposals.json"
)
PACKET_CANONICAL_SHA256 = (
    "95b47374e65ddf88148ca5c5a4ff250288837edea4960bf80ae8009395aba14c"
)
MANIFEST_CANONICAL_SHA256 = (
    "970999d998cdc202a6caa4e2f798017416c88211a5b6b8508132a07cc9080c0c"
)
PROPOSAL_MANIFEST_CANONICAL_SHA256 = (
    "457c14e7d41f677b21af254af45e331b24e6c685a7d7aa8eae556ced5bd7be65"
)
PROPOSAL_MANIFEST_RAW_SHA256 = (
    "171c7b88d54fe2bd7ddb4c220adb63f006e07c35391018b914482ace17cf7e93"
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
        proposal_manifest = load_json(REPOSITORY_ROOT / PROPOSAL_MANIFEST_PATH)
        self.assertEqual(
            (REPOSITORY_ROOT / PACKET_PATH).read_bytes(),
            canonical_json_bytes(packet) + b"\n",
        )
        self.assertEqual(
            (REPOSITORY_ROOT / MANIFEST_PATH).read_bytes(),
            canonical_json_bytes(manifest) + b"\n",
        )
        self.assertEqual(
            (REPOSITORY_ROOT / PROPOSAL_MANIFEST_PATH).read_bytes(),
            canonical_json_bytes(proposal_manifest) + b"\n",
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
        self.assertEqual(
            packet["cross_cutting_fixture_proposal_manifest_sha256"],
            hashlib.sha256(canonical_json_bytes(proposal_manifest)).hexdigest(),
        )
        self.assertEqual(
            hashlib.sha256(canonical_json_bytes(proposal_manifest)).hexdigest(),
            PROPOSAL_MANIFEST_CANONICAL_SHA256,
        )
        proposal_binding = packet["input_bindings"][
            "cross_cutting_fixture_proposals"
        ]
        self.assertEqual(proposal_binding["path"], str(PROPOSAL_MANIFEST_PATH))
        self.assertEqual(proposal_binding["sha256"], PROPOSAL_MANIFEST_RAW_SHA256)
        self.assertEqual(
            hashlib.sha256(
                (REPOSITORY_ROOT / PROPOSAL_MANIFEST_PATH).read_bytes()
            ).hexdigest(),
            proposal_binding["sha256"],
        )
        self.assertEqual(packet["schema_version"], "d004-pre-epoch-packet-v0.2")
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
        self.assertEqual(proposal_manifest["status"], "draft_unreviewed")
        self.assertEqual(
            proposal_manifest["schema_version"],
            "d004-cross-cutting-fixture-proposals-v0.2",
        )
        self.assertEqual(proposal_manifest["owner_protocol_review"], "none")
        self.assertEqual(proposal_manifest["executable_inputs_status"], "absent")
        self.assertIsNone(proposal_manifest["replay_repetitions"])
        self.assertEqual(proposal_manifest["evidence_status"], "none")
        self.assertEqual(
            proposal_manifest["class_statuses"],
            [
                {
                    "class": fixture_class,
                    "coverage_status": "unresolved",
                    "executable_fixture_count": 0,
                    "freeze_blocker": True,
                    "proposal_count": proposal_count,
                    "proposal_status": "draft_unreviewed",
                }
                for fixture_class, proposal_count in (
                    ("ambiguity", 5),
                    ("missing-edge", 14),
                    ("identity-substitution", 10),
                    ("unsupported", 5),
                    ("resource-exhaustion", 5),
                )
            ],
        )
        self.assertEqual(len(proposal_manifest["nonclaims"]), 11)
        self.assertEqual(len(proposal_manifest["proposals"]), 39)
        self.assertTrue(
            all(
                proposal["observation_level"] == "domain"
                for proposal in proposal_manifest["proposals"]
            )
        )
        self.assertNotIn(
            "replay-level",
            {proposal["class"] for proposal in proposal_manifest["proposals"]},
        )
        self.assertEqual(
            packet["unresolved_cross_cutting_fixture_classes"],
            [
                "ambiguity",
                "missing-edge",
                "identity-substitution",
                "unsupported",
                "resource-exhaustion",
            ],
        )
        self.assertEqual(len(packet["protocol_gaps"]), 6)

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

    def test_packet_binds_all_twenty_exact_existing_inputs_by_raw_bytes(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(len(packet["input_bindings"]), 20)
        self.assertEqual(
            set(packet["input_bindings"]),
            {
                "accepted_s3a_oep",
                "accepted_s3a_semantics",
                "accepted_s2_language",
                "cross_cutting_fixture_proposals",
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
            ("schema_version", "d004-pre-epoch-packet-v0.1", "d004_packet.digest"),
            ("status", "frozen", "d004_packet.digest"),
            ("epoch", "0001", "d004_packet.digest"),
            ("epoch_status", "frozen", "d004_packet.digest"),
            ("d003_disposition", "accepted", "d004_packet.digest"),
            ("owner_protocol_review", "solo-reviewed", "d004_packet.digest"),
            (
                "fixture_inventory_status",
                "complete",
                "d004_packet.digest",
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
            ("candidates", ["ST-REL"], "d004_packet.digest"),
            ("cases", ["SC-01"], "d004_packet.digest"),
            ("relationships", ["SR-01"], "d004_packet.digest"),
            ("hard_gates", ["SS-G01"], "d004_packet.digest"),
            ("source_roles", ["Specification"], "d004_packet.digest"),
            (
                "domain_observation_states",
                ["succeeded", "rejected"],
                "d004_packet.digest",
            ),
            ("case_verdicts", ["pass"], "d004_packet.digest"),
            ("mutations", ["SC-01-M01"], "d004_packet.digest"),
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
            self.assertIn("d004_packet.digest", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = self._copy_lab(root)
            packet = load_json(target)
            packet["unresolved_cross_cutting_fixture_classes"].pop()
            self._write_canonical(target, packet)
            self.assertIn("d004_packet.digest", self._codes(root))

        mutations = (
            (
                lambda packet: packet["execution"].update(
                    {"completed_candidate_cases": 1, "evidence_status": "partial"}
                ),
                "d004_packet.digest",
            ),
            (
                lambda packet: packet["execution"].update({"complete_candidates": 1}),
                "d004_packet.digest",
            ),
            (
                lambda packet: packet["execution"].update(
                    {"complete_cross_candidate_cases": 1}
                ),
                "d004_packet.digest",
            ),
            (
                lambda packet: packet.update({"conclusion": "recommend_st_rel"}),
                "d004_packet.digest",
            ),
            (
                lambda packet: packet.update({"selection": "ST-REL"}),
                "d004_packet.digest",
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
            (
                PROPOSAL_MANIFEST_PATH,
                "d004_packet.proposal_manifest_missing",
            ),
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
                "d004_packet.manifest_digest",
            ),
            (
                lambda manifest: manifest.append(copy.deepcopy(manifest[0])),
                "d004_packet.manifest_digest",
            ),
            (
                lambda manifest: manifest[0].update(
                    {"description": "accept implicit conversion"}
                ),
                "d004_packet.manifest_digest",
            ),
            (
                lambda manifest: manifest[0].update({"case": "SC-05"}),
                "d004_packet.manifest_digest",
            ),
            (
                lambda manifest: manifest[0].update({"case": []}),
                "d004_packet.manifest_digest",
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
            self.assertIn("d004_packet.digest", self._codes(root))

    def test_cross_cutting_proposal_manifest_root_and_record_shapes_fail_closed(self) -> None:
        mutations = (
            (
                lambda value: value.update({"unknown": True}),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value.pop("nonclaims"),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["proposals"][0].pop("observation_level"),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["proposals"][0].update({"result": "pass"}),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].pop("freeze_blocker"),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update({"result": "pass"}),
                "d004_packet.proposal_manifest_digest",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / PROPOSAL_MANIFEST_PATH
                proposal_manifest = load_json(target)
                mutate(proposal_manifest)
                self._write_canonical(target, proposal_manifest)
                self.assertIn(expected_code, self._codes(root))

    def test_cross_cutting_class_statuses_preserve_zero_coverage_blockers(self) -> None:
        mutations = (
            (
                lambda value: value["class_statuses"].pop(),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"].append(
                    {
                        "class": "replay-level",
                        "coverage_status": "unresolved",
                        "executable_fixture_count": 0,
                        "freeze_blocker": True,
                        "proposal_count": 1,
                        "proposal_status": "draft_unreviewed",
                    }
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"class": "replay-level"}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"proposal_count": 4}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"proposal_status": "reviewed"}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"executable_fixture_count": 1}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"coverage_status": "complete"}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"freeze_blocker": False}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"proposal_count": True}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"executable_fixture_count": False}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"][0].update(
                    {"freeze_blocker": 1}
                ),
                "d004_packet.proposal_manifest_digest",
            ),
            (
                lambda value: value["class_statuses"].__setitem__(
                    slice(0, 2),
                    [value["class_statuses"][1], value["class_statuses"][0]],
                ),
                "d004_packet.proposal_manifest_digest",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / PROPOSAL_MANIFEST_PATH
                proposal_manifest = load_json(target)
                mutate(proposal_manifest)
                self._write_canonical(target, proposal_manifest)
                self.assertIn(expected_code, self._codes(root))

    def test_cross_cutting_proposal_inventory_and_domain_semantics_fail_closed(self) -> None:
        mutations = (
            lambda value: value["proposals"].append(
                copy.deepcopy(value["proposals"][0])
            ),
            lambda value: value["proposals"].pop(24),
            lambda value: value["proposals"].__setitem__(
                slice(0, 2),
                [value["proposals"][1], value["proposals"][0]],
            ),
            lambda value: value["proposals"].__setitem__(
                slice(24, 26),
                [value["proposals"][25], value["proposals"][24]],
            ),
            lambda value: value["proposals"][0].update(
                {"relationship_scope": ["SR-02"], "target": "SR-02"}
            ),
            lambda value: value["proposals"][15].update(
                {
                    "id": value["proposals"][14]["id"],
                    "target": value["proposals"][14]["target"],
                }
            ),
            lambda value: value["proposals"][24].update(
                {"case_scope": ["SC-02"]}
            ),
            lambda value: value["proposals"][29].update(
                {"relationship_scope": ["SR-01"]}
            ),
            lambda value: value["proposals"][24].update(
                {"class": "unsupported", "expected_state": "unsupported"}
            ),
            lambda value: value["proposals"][29].update(
                {"class": "ambiguity", "expected_state": "rejected"}
            ),
            lambda value: value["proposals"][0].update(
                {"expected_state": "succeeded"}
            ),
            lambda value: value["proposals"][0].update({"layer": "executable"}),
            lambda value: value["proposals"][0].update(
                {"required_invalidation": "none"}
            ),
            lambda value: value["proposals"][0].update(
                {"match_rule": "sufficient"}
            ),
            lambda value: value["proposals"][0].update(
                {"capability_credit": "granted"}
            ),
            lambda value: value["proposals"][0].update(
                {"observation_level": "adapter"}
            ),
            lambda value: value["proposals"].append(
                {
                    **copy.deepcopy(value["proposals"][-1]),
                    "class": "replay-level",
                    "id": "D004-XF-RL-WALL-TIME",
                    "observation_level": "replay",
                }
            ),
        )
        for index, mutate in enumerate(mutations):
            with self.subTest(index=index), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / PROPOSAL_MANIFEST_PATH
                proposal_manifest = load_json(target)
                mutate(proposal_manifest)
                self._write_canonical(target, proposal_manifest)
                self.assertIn(
                    "d004_packet.proposal_manifest_digest",
                    self._codes(root),
                )

    def test_cross_cutting_proposal_boundary_cannot_claim_review_execution_or_coverage(self) -> None:
        mutations = (
            (
                "schema_version",
                "d004-cross-cutting-fixture-proposals-v0.1",
                "d004_packet.proposal_manifest_digest",
            ),
            ("status", "reviewed", "d004_packet.proposal_manifest_digest"),
            (
                "owner_protocol_review",
                "solo-reviewed",
                "d004_packet.proposal_manifest_digest",
            ),
            (
                "executable_inputs_status",
                "present",
                "d004_packet.proposal_manifest_digest",
            ),
            (
                "replay_repetitions",
                1,
                "d004_packet.proposal_manifest_digest",
            ),
            (
                "evidence_status",
                "partial",
                "d004_packet.proposal_manifest_digest",
            ),
            (
                "nonclaims",
                ["proposal coverage is complete"],
                "d004_packet.proposal_manifest_digest",
            ),
        )
        for field, replacement, expected_code in mutations:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / PROPOSAL_MANIFEST_PATH
                proposal_manifest = load_json(target)
                proposal_manifest[field] = replacement
                self._write_canonical(target, proposal_manifest)
                self.assertIn(expected_code, self._codes(root))

    def test_cross_cutting_proposal_transport_and_duplicate_keys_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / PROPOSAL_MANIFEST_PATH
            proposal_manifest = load_json(target)
            target.write_text(
                json.dumps(proposal_manifest, ensure_ascii=False) + "\n",
                encoding="utf-8",
            )
            self.assertIn(
                "d004_packet.proposal_manifest_canonical",
                self._codes(root),
            )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / PROPOSAL_MANIFEST_PATH
            raw = target.read_bytes()
            target.write_bytes(
                raw.replace(
                    b"{",
                    b'{"status":"draft_unreviewed",',
                    1,
                )
            )
            self.assertIn(
                "d004_packet.proposal_manifest_parse",
                self._codes(root),
            )

    def test_cross_cutting_proposal_manifest_cannot_self_rebind(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path = self._copy_lab(root)
            proposal_path = root / PROPOSAL_MANIFEST_PATH
            proposal_manifest = load_json(proposal_path)
            proposal_manifest["proposals"][0]["target"] = "SR-02"
            self._write_canonical(proposal_path, proposal_manifest)

            packet = load_json(packet_path)
            packet["cross_cutting_fixture_proposal_manifest_sha256"] = hashlib.sha256(
                canonical_json_bytes(proposal_manifest)
            ).hexdigest()
            packet["input_bindings"]["cross_cutting_fixture_proposals"][
                "sha256"
            ] = hashlib.sha256(proposal_path.read_bytes()).hexdigest()
            self._write_canonical(packet_path, packet)

            codes = self._codes(root)
            self.assertIn("d004_packet.proposal_manifest_digest", codes)
            self.assertIn("d004_packet.digest", codes)

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
