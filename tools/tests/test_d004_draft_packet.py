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
PACKET_PATH = RESEARCH_ROOT / "d004-v0.4-draft-packet.json"
MANIFEST_PATH = RESEARCH_ROOT / "d004-v0.2-named-mutations.json"
PROPOSAL_MANIFEST_PATH = (
    RESEARCH_ROOT / "d004-v0.2-cross-cutting-fixture-proposals.json"
)
PACKET_CANONICAL_SHA256 = (
    "b298cbf0d1c6af2ca9a4af7bb6b020595ffd00c1ab6896d5f097b3ebff13127d"
)
PACKET_RAW_SHA256 = (
    "0da96c89f62125f915152fb0ab30f41e608502bbe3a571a928c91e9d3812bc7a"
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
CASE_SUBJECT_CATALOG_PATH = RESEARCH_ROOT / "d004-v0.4-case-subjects.json"
CASE_SUBJECT_CATALOG_CANONICAL_SHA256 = (
    "b3a8bcf4f0f084740e92cbff6fd57273df0a078af9c6b974f68d95ba333c6dc1"
)
CASE_SUBJECT_CATALOG_RAW_SHA256 = (
    "c94100598aaf39954fe683a44f6a4d34837304eb361a1b478ca26884892d8ed6"
)
RESULT_CONTRACT_SOURCE_PATH = Path(
    "compiler/crates/orange-compiler/tests/d004_support/result_contract.rs"
)
DECISION_SUITE_SOURCE_PATH = Path(
    "compiler/crates/orange-compiler/tests/d004_decision_suite.rs"
)
RESULT_CONTRACT_SOURCE_SHA256 = (
    "8ef2abfd63d711907e911c415e4abbb903244aa9b44211f59e9b1f963c884292"
)
RESULT_CONTRACT_DESCRIPTOR_SHA256 = (
    "e3afc61c7127ca0b59dd010e90ae03a92c3354e3eee490c0667482c9218e8789"
)


class D004DraftPacketTests(unittest.TestCase):
    @staticmethod
    def _write_canonical(path: Path, value: object) -> None:
        path.write_bytes(canonical_json_bytes(value) + b"\n")

    def _copy_lab(self, root: Path) -> Path:
        shutil.copytree(REPOSITORY_ROOT / RESEARCH_ROOT, root / RESEARCH_ROOT)
        result_contract_target = root / RESULT_CONTRACT_SOURCE_PATH
        result_contract_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(
            REPOSITORY_ROOT / RESULT_CONTRACT_SOURCE_PATH,
            result_contract_target,
        )
        suite_target = root / DECISION_SUITE_SOURCE_PATH
        suite_target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPOSITORY_ROOT / DECISION_SUITE_SOURCE_PATH, suite_target)
        packet = load_json(root / PACKET_PATH)
        for binding in packet["input_bindings"].values():
            source = REPOSITORY_ROOT / binding["path"]
            target = root / binding["path"]
            if target.exists():
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(source, target)
        return root / PACKET_PATH

    def _assert_result_contract_mutation(self, mutate, expected_code: str) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / RESULT_CONTRACT_SOURCE_PATH
            source = target.read_text(encoding="utf-8")
            target.write_text(mutate(source), encoding="utf-8")
            self.assertIn(expected_code, self._codes(root))

    @staticmethod
    def _codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_d004_draft_packet()
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d004_packet.")
        }

    def _assert_case_catalog_mutation(self, mutate, expected_code: str) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / CASE_SUBJECT_CATALOG_PATH
            catalog = load_json(target)
            mutate(catalog)
            self._write_canonical(target, catalog)
            self.assertIn(expected_code, self._codes(root))

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
        self.assertEqual(packet["schema_version"], "d004-pre-epoch-packet-v0.4")
        self.assertEqual(packet["suite_version"], "d004-v0.4-draft")
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
            "case_and_cross_cutting_materialized_unreviewed_freeze_blocker",
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

    def test_packet_binds_all_twenty_two_exact_existing_inputs_by_raw_bytes(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(len(packet["input_bindings"]), 22)
        self.assertEqual(
            set(packet["input_bindings"]),
            {
                "accepted_s3a_oep",
                "accepted_s3a_semantics",
                "accepted_s2_language",
                "cross_cutting_executable_fixtures",
                "cross_cutting_fixture_proposals",
                "case_subjects",
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

    def test_future_result_contract_source_is_exact_closed_and_non_executing(
        self,
    ) -> None:
        source_path = REPOSITORY_ROOT / RESULT_CONTRACT_SOURCE_PATH
        source = source_path.read_text(encoding="utf-8")
        self.assertEqual(
            hashlib.sha256(source_path.read_bytes()).hexdigest(),
            RESULT_CONTRACT_SOURCE_SHA256,
        )
        self.assertIn(
            '"d004-result-contract-descriptor-v0.1-draft"', source
        )
        self.assertIn(
            "pub(crate) fn parse_draft_result_contract_descriptor(", source
        )
        self.assertNotIn("std::process", source)
        self.assertNotIn("std::fs", source)
        suite_source = (REPOSITORY_ROOT / DECISION_SUITE_SOURCE_PATH).read_text(
            encoding="utf-8"
        )
        self.assertIn(RESULT_CONTRACT_DESCRIPTOR_SHA256, suite_source)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            self.assertFalse(
                {
                    code
                    for code in self._codes(root)
                    if code.startswith("d004_packet.result_contract_")
                }
            )

    def test_future_result_contract_source_drift_and_live_apis_fail_closed(
        self,
    ) -> None:
        mutations = (
            (
                lambda source: source.replace(
                    '"d004-result-contract-descriptor-v0.1-draft"',
                    '"d004-result-contract-descriptor-v0.2-draft"',
                    1,
                ),
                "d004_packet.result_contract_schema",
            ),
            (
                lambda source: source.replace(
                    'string_entry("readiness_credit", "none")',
                    'string_entry("readiness_credit", "granted")',
                    1,
                ),
                "d004_packet.result_contract_zero_state",
            ),
            (
                lambda source: source
                + '\nfn readiness_percent() -> usize { 30 }\n',
                "d004_packet.result_contract_readiness_scope",
            ),
            (
                lambda source: source.replace(
                    '"no candidate-case execution completed"',
                    '"candidate-case execution completed"',
                    1,
                ),
                "d004_packet.result_contract_nonclaims",
            ),
            (
                lambda source: source.replace(
                    "observed_state is a member of the resolved oracle "
                    "allowed_domain_states",
                    "allowed_domain_states is an ordered preference list",
                    1,
                ),
                "d004_packet.result_contract_semantics",
            ),
            (
                lambda source: source.replace(
                    'string_entry("capability_credit", "none")',
                    'string_entry("capability_credit", "granted")',
                    1,
                ),
                "d004_packet.result_contract_semantics",
            ),
            (
                lambda source: source.replace(
                    '"execution kind completed requires exit_code zero, signal null, '
                    'adapter_status executed, and no truncated output"',
                    '"execution kind completed has no closed invariants"',
                    1,
                ),
                "d004_packet.result_contract_schema",
            ),
            (
                lambda source: source.replace(
                    '("mutation_inventory".to_owned(), mutation_inventory_value())',
                    '("mutation_inventory".to_owned(), JsonValue::Array(Vec::new()))',
                    1,
                ),
                "d004_packet.result_contract_schema",
            ),
            (
                lambda source: source.replace(
                    '"subject_oracle_inventory".to_owned()',
                    '"unbound_subject_inventory".to_owned()',
                    1,
                ),
                "d004_packet.result_contract_schema",
            ),
            (
                lambda source: source.replace(
                    "pub(crate) const SCHEDULED_SLOT_PREIMAGE_FIELDS: [&str; 10]",
                    "pub(crate) const SCHEDULED_SLOT_PREIMAGE_FIELDS: [&str; 9]",
                    1,
                ),
                "d004_packet.result_contract_schema",
            ),
            (
                lambda source: source.replace(
                    "pub(crate) const SR_CONFORMANCE_STATES: [&str; 4]",
                    "pub(crate) const SR_CONFORMANCE_STATES: [&str; 3]",
                    1,
                ),
                "d004_packet.result_contract_schema",
            ),
            (
                lambda source: source.replace(
                    "CANDIDATES[(round + position) % CANDIDATES.len()]",
                    "CANDIDATES[position]",
                    1,
                ),
                "d004_packet.result_contract_schema",
            ),
            (
                lambda source: source
                + "\npub(crate) fn launch_candidate() {}\n",
                "d004_packet.result_contract_execution_api",
            ),
            (
                lambda source: source
                + "\nuse std::fs;\npub(crate) fn persist_result() {}\n",
                "d004_packet.result_contract_persistence_api",
            ),
            (
                lambda source: source
                + "\npub(crate) fn parse_case_result() {}\n",
                "d004_packet.result_contract_parser_api",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code):
                self._assert_result_contract_mutation(mutate, expected_code)

        self._assert_result_contract_mutation(
            lambda source: source + "\n// unreviewed source drift\n",
            "d004_packet.result_contract_identity",
        )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            (root / RESULT_CONTRACT_SOURCE_PATH).unlink()
            self.assertIn(
                "d004_packet.result_contract_missing",
                self._codes(root),
            )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            suite_path = root / DECISION_SUITE_SOURCE_PATH
            suite_path.write_text(
                suite_path.read_text(encoding="utf-8").replace(
                    RESULT_CONTRACT_DESCRIPTOR_SHA256,
                    "0" * 64,
                    1,
                ),
                encoding="utf-8",
            )
            self.assertIn(
                "d004_packet.result_contract_suite_binding",
                self._codes(root),
            )

    def test_v04_case_subject_catalog_has_exact_input_only_identity_and_joins(
        self,
    ) -> None:
        path = REPOSITORY_ROOT / CASE_SUBJECT_CATALOG_PATH
        catalog = load_json(path)
        manifest = load_json(REPOSITORY_ROOT / MANIFEST_PATH)
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        canonical = canonical_json_bytes(catalog)
        raw = path.read_bytes()
        self.assertEqual(raw, canonical + b"\n")
        self.assertEqual(len(raw), 35_099)
        self.assertEqual(
            hashlib.sha256(canonical).hexdigest(),
            CASE_SUBJECT_CATALOG_CANONICAL_SHA256,
        )
        self.assertEqual(
            hashlib.sha256(raw).hexdigest(), CASE_SUBJECT_CATALOG_RAW_SHA256
        )
        self.assertEqual(
            packet["case_subject_catalog_sha256"],
            CASE_SUBJECT_CATALOG_CANONICAL_SHA256,
        )
        self.assertEqual(
            packet["input_bindings"]["case_subjects"],
            {
                "path": str(CASE_SUBJECT_CATALOG_PATH),
                "sha256": CASE_SUBJECT_CATALOG_RAW_SHA256,
            },
        )
        self.assertEqual(
            set(catalog),
            {
                "canonicalization",
                "evidence_status",
                "execution_boundary",
                "mutation_subject_count",
                "mutation_subjects",
                "nonclaims",
                "owner_protocol_review",
                "positive_subject_count",
                "positive_subjects",
                "schema_version",
                "source_bindings",
                "status",
                "subject_count",
                "suite_version",
            },
        )
        self.assertEqual(catalog["schema_version"], "d004-case-subject-catalog-v0.1")
        self.assertEqual(catalog["suite_version"], "d004-v0.4-draft")
        self.assertEqual(catalog["status"], "draft_unreviewed_input_only")
        self.assertEqual(catalog["owner_protocol_review"], "none")
        self.assertEqual(catalog["evidence_status"], "none")
        self.assertEqual(catalog["subject_count"], 31)
        self.assertEqual(catalog["positive_subject_count"], 5)
        self.assertEqual(catalog["mutation_subject_count"], 26)
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
            catalog["source_bindings"],
            {
                "named_mutation_manifest": {
                    "canonical_sha256": MANIFEST_CANONICAL_SHA256,
                    "path": str(MANIFEST_PATH),
                    "raw_sha256": hashlib.sha256(
                        (REPOSITORY_ROOT / MANIFEST_PATH).read_bytes()
                    ).hexdigest(),
                },
                "suite": {
                    "path": "docs/SEMANTIC_STRATA_DECISION_SUITE.md",
                    "raw_sha256": packet["input_bindings"]["decision_suite"][
                        "sha256"
                    ],
                },
            },
        )

        cases = [f"SC-{index:02d}" for index in range(1, 6)]
        relationships = {
            "SC-01": ["SR-01"],
            "SC-02": ["SR-01", "SR-02", "SR-09"],
            "SC-03": ["SR-02", "SR-10"],
            "SC-04": ["SR-01", "SR-03", "SR-11"],
            "SC-05": ["SR-04", "SR-08", "SR-12"],
        }
        positives = catalog["positive_subjects"]
        self.assertEqual([record["case"] for record in positives], cases)
        self.assertEqual(
            [record["id"] for record in positives],
            [f"D004-CS-POS-SC{index:02d}" for index in range(1, 6)],
        )
        positive_digests = {}
        for record in positives:
            case = record["case"]
            subject = record["subject"]
            self.assertEqual(
                set(record),
                {"case", "declared_expectation", "id", "subject", "subject_sha256"},
            )
            self.assertEqual(
                record["declared_expectation"],
                {
                    "allowed_domain_states": ["succeeded"],
                    "forbidden_domain_states": ["exhausted", "timeout"],
                    "observation_level": "domain",
                },
            )
            self.assertEqual(
                set(subject),
                {"case", "id", "kind", "model", "relationship_scope", "schema_version"},
            )
            self.assertEqual(subject["case"], case)
            self.assertEqual(subject["id"], record["id"])
            self.assertEqual(subject["kind"], "suite-only-positive-case")
            self.assertEqual(subject["relationship_scope"], relationships[case])
            self.assertEqual(subject["schema_version"], "d004-case-subject-v0.1")
            digest = hashlib.sha256(canonical_json_bytes(subject)).hexdigest()
            self.assertEqual(record["subject_sha256"], digest)
            positive_digests[case] = digest

        mutations = catalog["mutation_subjects"]
        self.assertEqual(
            [record["mutation_id"] for record in mutations],
            [record["id"] for record in manifest],
        )
        self.assertEqual(len({record["subject_sha256"] for record in positives + mutations}), 31)
        for record, manifest_record in zip(mutations, manifest):
            mutation_id = manifest_record["id"]
            case = manifest_record["case"]
            subject = record["subject"]
            model = subject["model"]
            self.assertEqual(
                set(record),
                {
                    "case", "declared_expectation", "id", "manifest_record_sha256",
                    "mutation_id", "subject", "subject_sha256",
                },
            )
            self.assertEqual(record["case"], case)
            self.assertEqual(record["mutation_id"], mutation_id)
            self.assertEqual(
                record["manifest_record_sha256"],
                hashlib.sha256(canonical_json_bytes(manifest_record)).hexdigest(),
            )
            self.assertEqual(
                set(subject),
                {
                    "case", "id", "kind", "model", "mutation_id",
                    "positive_subject_sha256", "relationship_scope", "schema_version",
                },
            )
            self.assertEqual(subject["case"], case)
            self.assertEqual(subject["id"], record["id"])
            self.assertEqual(subject["kind"], "suite-only-named-mutation")
            self.assertEqual(subject["mutation_id"], mutation_id)
            self.assertEqual(subject["positive_subject_sha256"], positive_digests[case])
            self.assertEqual(subject["relationship_scope"], relationships[case])
            self.assertEqual(subject["schema_version"], "d004-case-subject-v0.1")
            self.assertEqual(
                set(model),
                {"baseline_value", "dependent_result", "kind", "mutated_value", "operator", "target"},
            )
            self.assertEqual(model["kind"], "suite-only-single-invariant-mutation")
            self.assertNotEqual(model["baseline_value"], model["mutated_value"])
            self.assertEqual(
                model["dependent_result"],
                {
                    "id": "dependent_result",
                    "required_target": model["target"],
                    "required_value": model["baseline_value"],
                },
            )
            self.assertEqual(
                record["subject_sha256"],
                hashlib.sha256(canonical_json_bytes(subject)).hexdigest(),
            )

        forbidden = {
            "adapter_output", "candidate_execution", "candidate_observation",
            "capability_credit", "case_result", "case_verdict", "loader_status",
            "matched", "observed_invalidation", "observed_state", "verdict",
        }
        pending = [catalog]
        while pending:
            value = pending.pop()
            if isinstance(value, dict):
                self.assertTrue(forbidden.isdisjoint(value))
                pending.extend(value.values())
            elif isinstance(value, list):
                pending.extend(value)

    def test_case_subject_catalog_root_and_record_fields_fail_closed(self) -> None:
        mutations = (
            (lambda value: value.update({"unknown": True}), "d004_packet.case_subject_schema"),
            (lambda value: value.pop("nonclaims"), "d004_packet.case_subject_schema"),
            (
                lambda value: value.update({"status": "reviewed"}),
                "d004_packet.case_subject_boundary",
            ),
            (
                lambda value: value.update({"owner_protocol_review": "solo-reviewed"}),
                "d004_packet.case_subject_boundary",
            ),
            (
                lambda value: value.update({"evidence_status": "partial"}),
                "d004_packet.case_subject_boundary",
            ),
            (
                lambda value: value.update({"positive_subject_count": 6}),
                "d004_packet.case_subject_boundary",
            ),
            (
                lambda value: value["execution_boundary"].update(
                    {"candidate_adapter": "invoked"}
                ),
                "d004_packet.case_subject_boundary",
            ),
            (
                lambda value: value["source_bindings"]["suite"].update(
                    {"raw_sha256": "0" * 64}
                ),
                "d004_packet.case_subject_boundary",
            ),
            (
                lambda value: value["positive_subjects"][0].update(
                    {"unknown": True}
                ),
                "d004_packet.case_subject_positive_schema",
            ),
            (
                lambda value: value["positive_subjects"][0].pop("subject_sha256"),
                "d004_packet.case_subject_positive_schema",
            ),
            (
                lambda value: value["positive_subjects"][0]["subject"].update(
                    {"observed_state": "succeeded"}
                ),
                "d004_packet.case_subject_positive_subject_schema",
            ),
            (
                lambda value: value["mutation_subjects"][0].update(
                    {"unknown": True}
                ),
                "d004_packet.case_subject_mutation_schema",
            ),
            (
                lambda value: value["mutation_subjects"][0]["subject"].pop(
                    "positive_subject_sha256"
                ),
                "d004_packet.case_subject_mutation_subject_schema",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code):
                self._assert_case_catalog_mutation(mutate, expected_code)

    def test_case_subject_catalog_order_joins_digests_and_expectations_fail_closed(
        self,
    ) -> None:
        def duplicate_positive(value: dict[str, object]) -> None:
            value["positive_subjects"][1] = copy.deepcopy(value["positive_subjects"][0])

        def duplicate_mutation(value: dict[str, object]) -> None:
            value["mutation_subjects"][1] = copy.deepcopy(value["mutation_subjects"][0])

        def substitute_positive_baseline(value: dict[str, object]) -> None:
            record = value["mutation_subjects"][0]
            subject = record["subject"]
            subject["positive_subject_sha256"] = value["positive_subjects"][1][
                "subject_sha256"
            ]
            record["subject_sha256"] = hashlib.sha256(
                canonical_json_bytes(subject)
            ).hexdigest()

        mutations = (
            (duplicate_positive, "d004_packet.case_subject_positive_join"),
            (duplicate_mutation, "d004_packet.case_subject_mutation_join"),
            (
                lambda value: value["positive_subjects"][0].update(
                    {"subject_sha256": "0" * 64}
                ),
                "d004_packet.case_subject_positive_digest",
            ),
            (
                lambda value: value["mutation_subjects"][0].update(
                    {"subject_sha256": "0" * 64}
                ),
                "d004_packet.case_subject_mutation_digest",
            ),
            (
                lambda value: value["mutation_subjects"][0].update(
                    {"manifest_record_sha256": "0" * 64}
                ),
                "d004_packet.case_subject_mutation_join",
            ),
            (substitute_positive_baseline, "d004_packet.case_subject_mutation_subject"),
            (
                lambda value: value["positive_subjects"][0][
                    "declared_expectation"
                ].update({"allowed_domain_states": ["rejected"]}),
                "d004_packet.case_subject_positive_expectation",
            ),
            (
                lambda value: value["mutation_subjects"][0][
                    "declared_expectation"
                ].update({"allowed_domain_states": ["succeeded"]}),
                "d004_packet.case_subject_mutation_expectation",
            ),
            (
                lambda value: value["mutation_subjects"][0][
                    "declared_expectation"
                ].update({"required_invalidation": "none"}),
                "d004_packet.case_subject_mutation_expectation",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code):
                self._assert_case_catalog_mutation(mutate, expected_code)

    def test_case_subject_catalog_model_and_nonclaim_drift_fail_closed(self) -> None:
        def coherently_substitute_positive_model(value: dict[str, object]) -> None:
            record = value["positive_subjects"][0]
            subject = record["subject"]
            subject["model"]["authority"] = "substituted-positive-model"
            record["subject_sha256"] = hashlib.sha256(
                canonical_json_bytes(subject)
            ).hexdigest()

        def coherently_substitute_mutation_target(value: dict[str, object]) -> None:
            record = value["mutation_subjects"][0]
            subject = record["subject"]
            model = subject["model"]
            model["target"] = "substituted_target"
            model["dependent_result"]["required_target"] = "substituted_target"
            record["subject_sha256"] = hashlib.sha256(
                canonical_json_bytes(subject)
            ).hexdigest()

        mutations = (
            (
                coherently_substitute_positive_model,
                "d004_packet.case_subject_positive_subject",
            ),
            (
                coherently_substitute_mutation_target,
                "d004_packet.case_subject_mutation_model",
            ),
            (
                lambda value: value["mutation_subjects"][0]["subject"][
                    "model"
                ].update({"target": "different_target"}),
                "d004_packet.case_subject_mutation_model",
            ),
            (
                lambda value: value["mutation_subjects"][0]["subject"][
                    "model"
                ].update({"baseline_value": "different_baseline"}),
                "d004_packet.case_subject_mutation_model",
            ),
            (
                lambda value: value["mutation_subjects"][0]["subject"][
                    "model"
                ].update(
                    {
                        "mutated_value": value["mutation_subjects"][0]["subject"][
                            "model"
                        ]["baseline_value"]
                    }
                ),
                "d004_packet.case_subject_mutation_model",
            ),
            (
                lambda value: value["mutation_subjects"][0]["subject"][
                    "model"
                ]["dependent_result"].update({"required_value": "substituted"}),
                "d004_packet.case_subject_mutation_model",
            ),
            (
                lambda value: value["positive_subjects"][0].update(
                    {"evidence": {"verdict": "pass"}}
                ),
                "d004_packet.case_subject_positive_schema",
            ),
            (
                lambda value: value["positive_subjects"][0]["subject"][
                    "model"
                ].update({"candidate_execution": "performed"}),
                "d004_packet.case_subject_nonclaim",
            ),
        )
        for mutate, expected_code in mutations:
            with self.subTest(code=expected_code):
                self._assert_case_catalog_mutation(mutate, expected_code)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / CASE_SUBJECT_CATALOG_PATH
            catalog = load_json(target)
            target.write_text(
                json.dumps(catalog, ensure_ascii=False) + "\n",
                encoding="utf-8",
            )
            self.assertIn(
                "d004_packet.case_subject_catalog_canonical", self._codes(root)
            )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            target = root / CASE_SUBJECT_CATALOG_PATH
            raw = target.read_bytes()
            target.write_bytes(
                raw.replace(
                    b"{",
                    b'{"status":"draft_unreviewed_input_only",',
                    1,
                )
            )
            self.assertIn(
                "d004_packet.case_subject_catalog_parse", self._codes(root)
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
            (
                CASE_SUBJECT_CATALOG_PATH,
                "d004_packet.case_subject_catalog_missing",
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

    def test_result_and_evidence_artifacts_cannot_enter_the_pre_epoch_lab(self) -> None:
        unexpected_paths = (
            RESEARCH_ROOT / "results" / "ST-REL-SC-01.json",
            RESEARCH_ROOT / "evidence" / "ST-REL-SC-01.json",
            RESEARCH_ROOT / "candidate-execution.json",
        )
        for unexpected_path in unexpected_paths:
            with (
                self.subTest(path=unexpected_path),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory)
                self._copy_lab(root)
                target = root / unexpected_path
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("{}\n", encoding="utf-8")
                codes = self._codes(root)
                self.assertIn("d004_packet.research_inventory", codes)


if __name__ == "__main__":
    unittest.main()
