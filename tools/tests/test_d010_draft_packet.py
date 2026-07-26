from __future__ import annotations

import hashlib
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.validate_foundation import (
    DECISION_LABORATORY_SPECS,
    FoundationValidator,
    canonical_json_bytes,
    decision_laboratory_spec_errors,
    load_json,
    normalized_markdown_semantic_subject,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RESEARCH_ROOT = Path("research/decisions/D-010")
PACKET_PATH = RESEARCH_ROOT / "d010-v0.1-draft-packet.json"
INDEX_PATH = RESEARCH_ROOT / "d010-v0.1-case-input-index.json"
SUITE_PATH = Path("docs/COMPILER_STRATEGY_DECISION_SUITE.md")
PACKET_CANONICAL_SHA256 = (
    "1638d417a92c83cd6df984df1d832cb133aec839d170776ca27851e95232b00d"
)
PACKET_RAW_SHA256 = (
    "cdc0217c65468f05f6dc63ecbd743e2793a729988aad32f56004f0f124597f69"
)
INDEX_CANONICAL_SHA256 = (
    "4c8b0547a8f3bd380f4569008c8728014bb1d8718a5bfe17402bd03866560209"
)
INDEX_RAW_SHA256 = (
    "e9f59e86dff6219474d244ff01a98c75b7b17c65f1f91506d483a57e95e33670"
)
SUITE_RAW_SHA256 = (
    "5d36f1faeda027b9784846af0aa742339c6b821f39b72a8ca067a90c41a46c73"
)
BOUND_DOCUMENTS = (
    SUITE_PATH,
    Path("docs/DECISIONS.md"),
    Path("docs/ROADMAP.md"),
    Path("docs/ARCHITECTURE.md"),
    Path("docs/RESEARCH.md"),
    Path("docs/GATE0_TRACEABILITY.md"),
    Path("docs/ASSURANCE.md"),
    Path("docs/security/THREAT_MODEL.md"),
)


class D010DraftPacketTests(unittest.TestCase):
    @staticmethod
    def _write_canonical(path: Path, value: object) -> None:
        path.write_bytes(canonical_json_bytes(value) + b"\n")

    def _copy_lab(self, root: Path) -> tuple[Path, Path]:
        shutil.copytree(REPOSITORY_ROOT / RESEARCH_ROOT, root / RESEARCH_ROOT)
        for path in BOUND_DOCUMENTS:
            target = root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPOSITORY_ROOT / path, target)
        return root / PACKET_PATH, root / INDEX_PATH

    @staticmethod
    def _codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_compiler_strategy_suite()
        validator._validate_d010_draft_packet()
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d010_")
        }

    @staticmethod
    def _semantic_codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_compiler_strategy_suite()
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d010_suite.")
        }

    def test_canonical_pre_epoch_contract_is_exact_and_zero_baseline(self) -> None:
        packet_path = REPOSITORY_ROOT / PACKET_PATH
        index_path = REPOSITORY_ROOT / INDEX_PATH
        packet = load_json(packet_path)
        index = load_json(index_path)
        canonical_packet = canonical_json_bytes(packet)
        canonical_index = canonical_json_bytes(index)

        self.assertEqual(packet_path.read_bytes(), canonical_packet + b"\n")
        self.assertEqual(index_path.read_bytes(), canonical_index + b"\n")
        self.assertEqual(
            hashlib.sha256(canonical_packet).hexdigest(), PACKET_CANONICAL_SHA256
        )
        self.assertEqual(
            hashlib.sha256(packet_path.read_bytes()).hexdigest(), PACKET_RAW_SHA256
        )
        self.assertEqual(
            hashlib.sha256(canonical_index).hexdigest(), INDEX_CANONICAL_SHA256
        )
        self.assertEqual(
            hashlib.sha256(index_path.read_bytes()).hexdigest(), INDEX_RAW_SHA256
        )
        self.assertEqual(
            hashlib.sha256((REPOSITORY_ROOT / SUITE_PATH).read_bytes()).hexdigest(),
            SUITE_RAW_SHA256,
        )

        self.assertEqual(packet["schema_version"], "d010-pre-epoch-packet-v0.1")
        self.assertEqual(packet["suite_version"], "d010-v0.1-draft")
        self.assertEqual(packet["status"], "draft_unfrozen")
        self.assertIsNone(packet["epoch"])
        self.assertEqual(packet["epoch_status"], "unfrozen")
        self.assertEqual(packet["owner_protocol_review"], "none")
        self.assertEqual(packet["independent_review_status"], "unavailable")
        self.assertEqual(
            packet["dependency_acceptance"],
            {
                "D-003": True,
                "D-004": False,
                "D-005": False,
                "D-006": False,
                "D-009": False,
            },
        )
        self.assertEqual(
            [(row["id"], row["name"]) for row in packet["candidates"]],
            [
                ("CP-01", "Theorem/certificate hybrid direct-native path"),
                ("CP-02", "Mechanized proof-per-pass direct-native path"),
                ("CP-03", "Versioned Jasmin backend boundary"),
                ("CP-04", "Portable C11 interoperability boundary"),
                ("CP-05", "Versioned LLVM IR interoperability boundary"),
            ],
        )
        self.assertEqual(packet["cases"], [f"CC-{value:02d}" for value in range(1, 9)])
        self.assertEqual(packet["metrics"], [f"M-{value:02d}" for value in range(1, 20)])
        self.assertEqual(
            packet["comparative_axes"], [f"AX-{value:02d}" for value in range(1, 10)]
        )
        self.assertEqual(
            packet["owner_scopes"], [f"CR-{value:02d}" for value in range(1, 12)]
        )
        self.assertEqual(packet["hard_gate_count"], 8)
        self.assertEqual(
            packet["atomic_outcomes"],
            ["satisfied", "not_satisfied", "unresolved", "unsupported"],
        )
        self.assertEqual(
            packet["hard_gate_state_precedence"],
            ["unsupported", "fail", "unresolved", "pass"],
        )
        self.assertEqual(
            packet["comparative_labels"],
            [
                "hybrid_direct_native_better",
                "proof_per_pass_direct_native_better",
                "jasmin_backend_better",
                "portable_c11_better",
                "llvm_ir_better",
                "practically_equivalent",
                "inconclusive",
            ],
        )
        self.assertEqual(
            packet["conclusions"],
            [
                "recommend_hybrid_direct_native",
                "recommend_proof_per_pass_direct_native",
                "recommend_jasmin_backend",
                "recommend_portable_c11",
                "recommend_llvm_ir",
                "tie",
                "inconclusive",
            ],
        )
        self.assertEqual(
            packet["execution"],
            {
                "complete_candidates": 0,
                "complete_cross_candidate_cases": 0,
                "completed_candidate_cases": 0,
                "evidence_status": "none",
                "required_candidate_cases": 40,
            },
        )
        self.assertTrue(
            all(
                state["adapter_status"] == "absent"
                and state["dependency_admission_status"] == "absent"
                and state["implementation_status"] == "absent"
                and state["execution_status"] == "not_performed"
                for state in packet["candidate_states"]
            )
        )
        self.assertIsNone(packet["physical_execution_order"])
        self.assertIsNone(packet["selection"])
        self.assertIsNone(packet["conclusion"])
        self.assertEqual(len(index["case_inputs"]), 8)
        self.assertTrue(
            all(
                row["executable_fixture_count"] == 0
                and row["freeze_blocker"] is True
                and row["shared_inputs_status"] == "absent"
                and row["candidate_mapping_status"] == "absent"
                and row["coverage_status"] == "unresolved"
                for row in index["case_inputs"]
            )
        )
        self.assertEqual(self._codes(REPOSITORY_ROOT), set())

    def test_semantic_bindings_match_every_exact_subject(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        bindings = packet["semantic_bindings"]
        self.assertEqual(len(bindings), 10)
        for identifier, binding in bindings.items():
            with self.subTest(identifier=identifier):
                source = (REPOSITORY_ROOT / binding["path"]).read_text(encoding="utf-8")
                subject = normalized_markdown_semantic_subject(
                    source,
                    scope=binding["scope"],
                    section_start_heading=binding["section_start_heading"],
                    section_end_heading=binding["section_end_heading"],
                )
                self.assertEqual(
                    hashlib.sha256(subject.encode()).hexdigest(),
                    binding["normalized_sha256"],
                )
        self.assertEqual(self._semantic_codes(REPOSITORY_ROOT), set())

    def test_line_endings_are_the_only_semantic_normalization(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            for path in BOUND_DOCUMENTS:
                target = root / path
                source = target.read_bytes()
                self.assertNotIn(b"\r", source)
                target.write_bytes(source.replace(b"\n", b"\r\n"))
            self.assertEqual(self._semantic_codes(root), set())

    def test_cross_document_and_range_mutations_fail_closed(self) -> None:
        mutations = (
            (
                SUITE_PATH,
                "This suite compares five compiler and output-path strategies",
                "This suite prefers one compiler and compares five output-path strategies",
                "d010_suite.semantic_closure",
            ),
            (
                Path("docs/DECISIONS.md"),
                "## D-010 — Compiler strategy",
                "## D-010 — Compiler strategy (draft)",
                "d010_suite.register_semantics",
            ),
            (
                Path("docs/DECISIONS.md"),
                "## D-011 — Initial native target envelope",
                "## D-011 — Initial native target envelope\n\nD-010 is already selected.",
                "d010_suite.register_semantics",
            ),
            (
                Path("docs/ROADMAP.md"),
                "### S5 — Compiler IRs and one output path",
                "### S5 — Compiler IRs and one output path  ",
                "d010_suite.roadmap_closure",
            ),
            (
                Path("docs/ARCHITECTURE.md"),
                "# End-state architecture",
                "# End-state architecture\n\nCP-01 is selected.",
                "d010_suite.cross_document_semantics",
            ),
            (
                Path("docs/security/THREAT_MODEL.md"),
                "# Solo-bootstrap threat model",
                "# Solo-bootstrap threat model\n\nD-010 is accepted.",
                "d010_suite.cross_document_semantics",
            ),
        )
        for path, original, replacement, expected in mutations:
            with self.subTest(path=path), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / path
                source = target.read_text(encoding="utf-8")
                self.assertIn(original, source)
                target.write_text(source.replace(original, replacement, 1), encoding="utf-8")
                self.assertIn(expected, self._semantic_codes(root))

    def test_packet_and_index_mutations_cannot_reseal_the_contract(self) -> None:
        mutations = (
            (
                "packet_candidate",
                PACKET_PATH,
                lambda value: value["candidates"][0].update({"name": "Preferred path"}),
                "d010_packet.digest",
            ),
            (
                "packet_dependency",
                PACKET_PATH,
                lambda value: value["dependency_acceptance"].update({"D-003": False}),
                "d010_packet.digest",
            ),
            (
                "packet_semantic_reseal",
                PACKET_PATH,
                lambda value: value["semantic_bindings"]["roadmap_document"].update(
                    {"normalized_sha256": "0" * 64}
                ),
                "d010_packet.digest",
            ),
            (
                "index_fixture",
                INDEX_PATH,
                lambda value: value["case_inputs"][0].update(
                    {"executable_fixture_count": 1, "freeze_blocker": False}
                ),
                "d010_packet.index_digest",
            ),
        )
        for name, path, mutate, expected in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / path
                value = load_json(target)
                mutate(value)
                self._write_canonical(target, value)
                self.assertIn(expected, self._codes(root))

    def test_noncanonical_transport_and_raw_input_substitution_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet_path.write_bytes(packet_path.read_bytes() + b"\n")
            self.assertIn("d010_packet.canonical", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            suite = root / SUITE_PATH
            suite.write_bytes(suite.read_bytes() + b"\n")
            self.assertIn("d010_packet.input_digest", self._codes(root))

    def test_closed_lab_specification_is_self_consistent(self) -> None:
        specification = DECISION_LABORATORY_SPECS["d010"]
        self.assertEqual(decision_laboratory_spec_errors(REPOSITORY_ROOT, specification), ())
        self.assertEqual(
            specification["inventory"],
            {
                "research/decisions/D-010/README.md",
                "research/decisions/D-010/d010-v0.1-case-input-index.json",
                "research/decisions/D-010/d010-v0.1-draft-packet.json",
            },
        )


if __name__ == "__main__":
    unittest.main()
