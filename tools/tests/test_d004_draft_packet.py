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
PACKET_PATH = RESEARCH_ROOT / "d004-v0.3-draft-packet.json"
MANIFEST_PATH = RESEARCH_ROOT / "d004-v0.2-named-mutations.json"
PROPOSAL_MANIFEST_PATH = (
    RESEARCH_ROOT / "d004-v0.2-cross-cutting-fixture-proposals.json"
)
PACKET_CANONICAL_SHA256 = (
    "7fb725d374e39eeae8a3a01ecf6033d53205f61d28ab94371e35ee0b59a07e58"
)
PACKET_RAW_SHA256 = (
    "0095a821d2a94b6163538965707b3ebadc554c9260b66bd45c943b8cefb9e739"
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
FIXTURE_CATALOG_PATH = (
    RESEARCH_ROOT / "d004-v0.3-cross-cutting-executable-fixtures.json"
)
FIXTURE_CATALOG_CANONICAL_SHA256 = (
    "ca08308161244e9541803aa8008dd1624a2101f77da8b656cf0c5deff8a60703"
)
FIXTURE_CATALOG_RAW_SHA256 = (
    "268b4065028f1af9c9ec912ae8884c150094189f5d782963f42ed6ed4cca6ce0"
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
        fixture_catalog = load_json(REPOSITORY_ROOT / FIXTURE_CATALOG_PATH)
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
            hashlib.sha256((REPOSITORY_ROOT / PACKET_PATH).read_bytes()).hexdigest(),
            PACKET_RAW_SHA256,
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
        self.assertEqual(
            packet["cross_cutting_executable_fixture_catalog_sha256"],
            hashlib.sha256(canonical_json_bytes(fixture_catalog)).hexdigest(),
        )
        self.assertEqual(
            packet["cross_cutting_executable_fixture_catalog_sha256"],
            FIXTURE_CATALOG_CANONICAL_SHA256,
        )
        fixture_binding = packet["input_bindings"][
            "cross_cutting_executable_fixtures"
        ]
        self.assertEqual(fixture_binding["path"], str(FIXTURE_CATALOG_PATH))
        self.assertEqual(fixture_binding["sha256"], FIXTURE_CATALOG_RAW_SHA256)
        self.assertEqual(
            hashlib.sha256(
                (REPOSITORY_ROOT / FIXTURE_CATALOG_PATH).read_bytes()
            ).hexdigest(),
            fixture_binding["sha256"],
        )
        self.assertEqual(packet["schema_version"], "d004-pre-epoch-packet-v0.3")
        self.assertEqual(packet["suite_version"], "d004-v0.3-draft")
        self.assertEqual(packet["status"], "draft_unfrozen")
        self.assertIsNone(packet["epoch"])
        self.assertEqual(packet["epoch_status"], "unfrozen")
        self.assertEqual(
            packet["d003_disposition"],
            "accepted_exact_revision_oep_closure",
        )
        self.assertEqual(packet["owner_protocol_review"], "none")
        self.assertEqual(
            packet["fixture_inventory_status"],
            "cross_cutting_materialized_unreviewed_freeze_blocker",
        )
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
        self.assertTrue(
            {
                "candidate_mappings",
                "candidate_adapters",
                "observed_states",
                "observation_matches",
                "case_records",
                "case_results",
            }.isdisjoint(packet)
        )
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
        self.assertEqual(
            packet["protocol_gaps"],
            [
                "five positive subjects absent",
                "26 named-mutation subjects absent",
                "ambiguity fixture sufficiency review unresolved",
                "missing-edge fixture sufficiency review unresolved",
                "identity-substitution fixture sufficiency review unresolved",
                "unsupported fixture sufficiency review unresolved",
                "resource-exhaustion fixture sufficiency review unresolved",
                "replay repetition count unresolved",
            ],
        )

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

    def test_packet_binds_all_twenty_one_exact_existing_inputs_by_raw_bytes(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(len(packet["input_bindings"]), 21)
        self.assertEqual(
            set(packet["input_bindings"]),
            {
                "accepted_s3a_oep",
                "accepted_s3a_semantics",
                "accepted_s2_language",
                "cross_cutting_executable_fixtures",
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

    def test_v03_fixture_catalog_has_exact_ordered_byte_addressed_input_only_coverage(
        self,
    ) -> None:
        catalog_path = REPOSITORY_ROOT / FIXTURE_CATALOG_PATH
        proposal_manifest = load_json(REPOSITORY_ROOT / PROPOSAL_MANIFEST_PATH)
        catalog = load_json(catalog_path)
        canonical = canonical_json_bytes(catalog)
        raw = catalog_path.read_bytes()
        self.assertEqual(raw, canonical + b"\n")
        self.assertEqual(len(raw), 67_329)
        self.assertEqual(
            hashlib.sha256(canonical).hexdigest(),
            FIXTURE_CATALOG_CANONICAL_SHA256,
        )
        self.assertEqual(
            hashlib.sha256(raw).hexdigest(), FIXTURE_CATALOG_RAW_SHA256
        )
        self.assertEqual(
            set(catalog),
            {
                "canonicalization",
                "class_counts",
                "evidence_status",
                "execution_boundary",
                "fixture_count",
                "fixtures",
                "nonclaims",
                "owner_protocol_review",
                "proposal_manifest",
                "schema_version",
                "status",
                "suite_version",
            },
        )
        self.assertEqual(
            catalog["schema_version"],
            "d004-cross-cutting-executable-fixtures-v0.1",
        )
        self.assertEqual(catalog["suite_version"], "d004-v0.3-draft")
        self.assertEqual(catalog["status"], "draft_unreviewed_input_only")
        self.assertEqual(catalog["owner_protocol_review"], "none")
        self.assertEqual(catalog["evidence_status"], "none")
        self.assertEqual(catalog["fixture_count"], 39)
        self.assertEqual(
            catalog["class_counts"],
            [
                {"class": "ambiguity", "fixture_count": 5},
                {"class": "missing-edge", "fixture_count": 14},
                {"class": "identity-substitution", "fixture_count": 10},
                {"class": "unsupported", "fixture_count": 5},
                {"class": "resource-exhaustion", "fixture_count": 5},
            ],
        )
        self.assertEqual(
            catalog["execution_boundary"],
            {
                "candidate_adapter": "not_invoked",
                "candidate_process": "not_invoked",
                "candidate_tool": "not_invoked",
                "network": "not_used",
                "preflight_output_persistence": "none",
            },
        )
        self.assertEqual(
            catalog["proposal_manifest"],
            {
                "path": str(PROPOSAL_MANIFEST_PATH),
                "canonical_sha256": PROPOSAL_MANIFEST_CANONICAL_SHA256,
                "raw_sha256": PROPOSAL_MANIFEST_RAW_SHA256,
            },
        )

        fixtures = catalog["fixtures"]
        proposals = proposal_manifest["proposals"]
        self.assertEqual(len(fixtures), len(proposals))
        self.assertEqual(
            [fixture["proposal_id"] for fixture in fixtures],
            [proposal["id"] for proposal in proposals],
        )
        self.assertEqual(
            len({fixture["fixture_subject_sha256"] for fixture in fixtures}), 39
        )
        relationships = [f"SR-{value:02d}" for value in range(1, 15)]
        identity_slots = [
            "packet_identity",
            "replay_plan_identity",
            "scheduled_slot_identity",
            "input_manifest_identity",
            "candidate_graph_identity",
            "sr_map_identity",
            "semantic_endpoint_identity",
            "parameter_model_identity",
            "tool_identity",
            "environment_identity",
        ]

        def identity(slot: str, variant: str) -> str:
            return hashlib.sha256(
                b"d004-fixture-identity-v0.1\0"
                + slot.encode("ascii")
                + b"\0"
                + variant.encode("ascii")
            ).hexdigest()

        for index, (fixture, proposal) in enumerate(zip(fixtures, proposals)):
            with self.subTest(proposal=proposal["id"]):
                self.assertEqual(
                    set(fixture),
                    {
                        "expected_observation",
                        "fixture_subject",
                        "fixture_subject_sha256",
                        "proposal_id",
                        "proposal_record_sha256",
                    },
                )
                self.assertEqual(
                    fixture["proposal_record_sha256"],
                    hashlib.sha256(canonical_json_bytes(proposal)).hexdigest(),
                )
                subject = fixture["fixture_subject"]
                self.assertEqual(
                    fixture["fixture_subject_sha256"],
                    hashlib.sha256(canonical_json_bytes(subject)).hexdigest(),
                )
                self.assertEqual(
                    set(subject),
                    {
                        "case_scope",
                        "class",
                        "layer",
                        "model",
                        "mutation_kind",
                        "proposal_id",
                        "relationship_scope",
                        "schema_version",
                        "target",
                    },
                )
                self.assertEqual(
                    subject["schema_version"],
                    "d004-cross-cutting-fixture-subject-v0.1",
                )
                for subject_field, proposal_field in (
                    ("proposal_id", "id"),
                    ("class", "class"),
                    ("case_scope", "case_scope"),
                    ("relationship_scope", "relationship_scope"),
                    ("layer", "layer"),
                    ("mutation_kind", "mutation_kind"),
                    ("target", "target"),
                ):
                    self.assertEqual(
                        subject[subject_field], proposal[proposal_field]
                    )
                self.assertEqual(
                    fixture["expected_observation"],
                    {
                        "observation_level": proposal["observation_level"],
                        "state": proposal["expected_state"],
                        "required_invalidation": proposal[
                            "required_invalidation"
                        ],
                        "match_rule": proposal["match_rule"],
                        "capability_credit": "none",
                    },
                )
                self.assertTrue(
                    {
                        "loader_status",
                        "observed_state",
                        "matched",
                        "result",
                    }.isdisjoint(fixture)
                )

                model = subject["model"]
                target = proposal["target"]
                fixture_class = proposal["class"]
                self.assertEqual(model["kind"], fixture_class)
                if fixture_class == "missing-edge":
                    self.assertEqual(model["baseline_relationships"], relationships)
                    self.assertEqual(
                        model["mutated_relationships"],
                        [value for value in relationships if value != target],
                    )
                    self.assertEqual(
                        model["dependent_result"]["required_relationships"],
                        [target],
                    )
                elif fixture_class == "identity-substitution":
                    self.assertEqual(
                        [binding["slot"] for binding in model["baseline_bindings"]],
                        identity_slots,
                    )
                    changed = []
                    for baseline, mutated in zip(
                        model["baseline_bindings"], model["mutated_bindings"]
                    ):
                        slot = baseline["slot"]
                        self.assertEqual(
                            baseline["identity_sha256"], identity(slot, "original")
                        )
                        if baseline != mutated:
                            changed.append(slot)
                            self.assertEqual(
                                mutated["identity_sha256"],
                                identity(slot, "substitute"),
                            )
                    self.assertEqual(changed, [target])
                    self.assertEqual(
                        model["dependent_result"]["required_binding"],
                        {"slot": target, "identity_sha256": identity(target, "original")},
                    )
                elif fixture_class == "ambiguity":
                    self.assertEqual(model["authority_key"], target)
                    self.assertEqual(len(model["interpretations"]), 2)
                    self.assertEqual(
                        len(
                            {
                                interpretation["value"]
                                for interpretation in model["interpretations"]
                            }
                        ),
                        2,
                    )
                    self.assertIs(
                        model["dependent_result"]["requires_unique_authority"],
                        True,
                    )
                elif fixture_class == "unsupported":
                    self.assertEqual(model["request"]["operation"], target)
                    self.assertEqual(
                        model["support_domain"]["unsupported_operations"],
                        [target],
                    )
                    self.assertNotIn(
                        target, model["support_domain"]["supported_operations"]
                    )
                else:
                    self.assertEqual(fixture_class, "resource-exhaustion")
                    self.assertEqual(model["resource_domain"]["limit"], 8)
                    self.assertEqual(len(model["request"]["work_items"]), 9)
                    self.assertGreater(
                        len(model["request"]["work_items"]),
                        model["resource_domain"]["limit"],
                    )

        for candidate in ("ST-REL", "ST-UNI", "ST-DUAL", "ST-MIRROR", "ST-HOST"):
            self.assertNotIn(candidate.encode("ascii"), raw)

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
            (
                FIXTURE_CATALOG_PATH,
                "d004_packet.fixture_catalog_missing",
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

    def test_cross_cutting_fixture_catalog_transport_and_duplicate_keys_fail_closed(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / FIXTURE_CATALOG_PATH
            fixture_catalog = load_json(target)
            target.write_text(
                json.dumps(fixture_catalog, ensure_ascii=False) + "\n",
                encoding="utf-8",
            )
            self.assertIn(
                "d004_packet.fixture_catalog_canonical",
                self._codes(root),
            )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / FIXTURE_CATALOG_PATH
            raw = target.read_bytes()
            target.write_bytes(
                raw.replace(
                    b"{",
                    b'{"status":"draft_unreviewed_input_only",',
                    1,
                )
            )
            self.assertIn(
                "d004_packet.fixture_catalog_parse",
                self._codes(root),
            )

    def test_cross_cutting_fixture_catalog_cannot_self_rebind(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path = self._copy_lab(root)
            catalog_path = root / FIXTURE_CATALOG_PATH
            fixture_catalog = load_json(catalog_path)
            subject = fixture_catalog["fixtures"][0]["fixture_subject"]
            subject["model"]["mutated_relationships"].append("SR-01")
            fixture_catalog["fixtures"][0]["fixture_subject_sha256"] = (
                hashlib.sha256(canonical_json_bytes(subject)).hexdigest()
            )
            self._write_canonical(catalog_path, fixture_catalog)

            packet = load_json(packet_path)
            packet["cross_cutting_executable_fixture_catalog_sha256"] = (
                hashlib.sha256(canonical_json_bytes(fixture_catalog)).hexdigest()
            )
            packet["input_bindings"]["cross_cutting_executable_fixtures"][
                "sha256"
            ] = hashlib.sha256(catalog_path.read_bytes()).hexdigest()
            self._write_canonical(packet_path, packet)

            codes = self._codes(root)
            self.assertIn("d004_packet.fixture_catalog_digest", codes)
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
