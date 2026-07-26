from __future__ import annotations

import copy
import hashlib
import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest import mock

import tools.validate_foundation as validate_foundation
from tools.validate_foundation import (
    DECISION_LABORATORY_SPECS,
    FoundationValidator,
    canonical_json_bytes,
    decision_laboratory_spec_errors,
    load_json,
)


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RESEARCH_ROOT = Path("research/decisions/D-009")
PACKET_PATH = RESEARCH_ROOT / "d009-v0.1-draft-packet.json"
INDEX_PATH = RESEARCH_ROOT / "d009-v0.1-case-input-index.json"
SUITE_PATH = Path("docs/SOLVER_TRUST_DECISION_SUITE.md")
DECISIONS_PATH = Path("docs/DECISIONS.md")
ROADMAP_PATH = Path("docs/ROADMAP.md")
PACKET_CANONICAL_SHA256 = "6c1da1fb28069b3511ef77350b3dcd6ba2759f087837f04155f545319ad3a18b"
PACKET_RAW_SHA256 = "acaaa774d8a14b19a505a719e38b9793b41883748280be7b3b7115c6266bfcae"
INDEX_CANONICAL_SHA256 = "2e55c671771d5740b0346992c8b86b9cce0571a8fc3e5b745195b0956010470e"
INDEX_RAW_SHA256 = "c5298d625f5392de2774ffb861fe1dc1701b379ebd385cde0584a8cbcd249859"
SUITE_RAW_SHA256 = "a26073e6431fb401af4aac6e57dcdfa76b27fe9451c26fb42595d7de14c2a35b"


class D009DraftPacketTests(unittest.TestCase):
    @staticmethod
    def _write_canonical(path: Path, value: object) -> None:
        path.write_bytes(canonical_json_bytes(value) + b"\n")

    def _copy_lab(self, root: Path) -> tuple[Path, Path]:
        shutil.copytree(REPOSITORY_ROOT / RESEARCH_ROOT, root / RESEARCH_ROOT)
        for path in (SUITE_PATH, DECISIONS_PATH, ROADMAP_PATH):
            target = root / path
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPOSITORY_ROOT / path, target)
        return root / PACKET_PATH, root / INDEX_PATH

    @staticmethod
    def _codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_solver_trust_suite()
        validator._validate_d009_draft_packet()
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d009_")
        }

    @staticmethod
    def _semantic_codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validator._validate_solver_trust_suite()
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d009_suite.")
        }

    def _assert_text_mutations(
        self,
        path: Path,
        mutations: tuple[tuple[str, str, str], ...],
        expected_code: str,
    ) -> None:
        for name, original, replacement in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / path
                source = target.read_text(encoding="utf-8")
                self.assertIn(original, source)
                target.write_text(source.replace(original, replacement, 1), encoding="utf-8")
                self.assertIn(expected_code, self._semantic_codes(root))

    def _assert_mutations(
        self,
        target: str,
        mutations: tuple[tuple[str, object], ...],
        expected_code: str,
    ) -> None:
        for name, mutate in mutations:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                packet_path, index_path = self._copy_lab(root)
                path = packet_path if target == "packet" else index_path
                value = load_json(path)
                mutate(value)
                self._write_canonical(path, value)
                self.assertIn(expected_code, self._codes(root))

    def test_canonical_pre_epoch_lab_is_exact_and_records_no_execution(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        case_index = load_json(REPOSITORY_ROOT / INDEX_PATH)
        canonical_packet = canonical_json_bytes(packet)
        canonical_index = canonical_json_bytes(case_index)

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
            hashlib.sha256((REPOSITORY_ROOT / PACKET_PATH).read_bytes()).hexdigest(),
            PACKET_RAW_SHA256,
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
        self.assertEqual(packet["schema_version"], "d009-pre-epoch-packet-v0.3")
        self.assertEqual(packet["suite_version"], "d009-v0.1-draft")
        self.assertEqual(packet["status"], "draft_unfrozen")
        self.assertIsNone(packet["epoch"])
        self.assertEqual(packet["epoch_status"], "unfrozen")
        self.assertEqual(packet["owner_protocol_review"], "none")
        self.assertEqual(packet["independent_review_status"], "unavailable")
        self.assertEqual(
            packet["dependency_acceptance"],
            {"D-004": False, "D-005": False},
        )
        self.assertEqual(
            [candidate["id"] for candidate in packet["candidates"]],
            ["SP-01", "SP-02", "SP-03"],
        )
        self.assertEqual(packet["cases"], [f"TC-{index:02d}" for index in range(1, 9)])
        self.assertEqual(packet["hard_gate_count"], 8)
        self.assertEqual(
            packet["atomic_outcomes"],
            ["satisfied", "not_satisfied", "unresolved", "unsupported"],
        )
        self.assertEqual(
            packet["atomic_outcome_meanings"],
            {
                "satisfied": (
                    "the exact proposition has its complete permitted mandatory closure and no "
                    "valid decisive negative result"
                ),
                "not_satisfied": (
                    "permitted, identity-bound negative evidence establishes that the exact "
                    "proposition is false or violated within its scope; absence or "
                    "incompleteness alone is not not_satisfied"
                ),
                "unresolved": (
                    "the claim is well-formed and within the declared support model, but a "
                    "required decision remains unknown, incomplete, conflicting, or exhausted"
                ),
                "unsupported": (
                    "the declared policy or support envelope offers no permitted evaluation or "
                    "authority path for that exact claim and scope"
                ),
            },
        )
        self.assertEqual(
            packet["hard_gate_states"],
            ["pass", "fail", "unresolved", "unsupported"],
        )
        self.assertEqual(
            packet["hard_gate_state_precedence"],
            ["unsupported", "fail", "unresolved", "pass"],
        )
        self.assertEqual(
            packet["semantic_bindings"],
            {
                "decision_register_document": {
                    "normalization": "markdown-prose-lines-exact-v1",
                    "normalized_sha256": (
                        "e42c8d0a94b5cf4cd151bfc584740e2a846380cfce6394b37f261bf34650d648"
                    ),
                    "path": "docs/DECISIONS.md",
                    "scope": "whole_document",
                    "section_end_heading": None,
                    "section_start_heading": None,
                },
                "decision_register_d009": {
                    "normalization": "markdown-prose-lines-exact-v1",
                    "normalized_sha256": (
                        "69c61bb8e6cd7fd745be6d308074497916cd7ecb9b5ee1786461454bec363270"
                    ),
                    "path": "docs/DECISIONS.md",
                    "scope": "markdown_exact_heading_range",
                    "section_end_heading": "## D-010 — Compiler strategy",
                    "section_start_heading": "## D-009 — Solver trust",
                },
                "roadmap_document": {
                    "normalization": "markdown-prose-lines-exact-v1",
                    "normalized_sha256": (
                        "9e0e93db121115250f4f8312fd7d3e95d52f7ffd5aae38492130aeab05909d58"
                    ),
                    "path": "docs/ROADMAP.md",
                    "scope": "whole_document",
                    "section_end_heading": None,
                    "section_start_heading": None,
                },
                "roadmap_s4": {
                    "normalization": "markdown-prose-lines-exact-v1",
                    "normalized_sha256": (
                        "f8a3ee4beeee6c789a3b4c7b6b0177c32a573788a7f9779b1aea763297288d62"
                    ),
                    "path": "docs/ROADMAP.md",
                    "scope": "markdown_exact_heading_range",
                    "section_end_heading": "### S5 — Compiler IRs and one output path",
                    "section_start_heading": "### S4 — Proof and claim boundary",
                },
                "solver_trust_suite": {
                    "normalization": "markdown-prose-lines-exact-v1",
                    "normalized_sha256": (
                        "c2838efcc963de22141631d58fae4730c131d47d8ea2e79906cf66d0546032d0"
                    ),
                    "path": "docs/SOLVER_TRUST_DECISION_SUITE.md",
                    "scope": "whole_document",
                    "section_end_heading": None,
                    "section_start_heading": None,
                },
            },
        )
        self.assertEqual(len(packet["metrics"]), 16)
        self.assertEqual(len(packet["owner_scopes"]), 8)
        self.assertEqual(
            packet["execution"],
            {
                "complete_candidates": 0,
                "complete_cross_candidate_cases": 0,
                "completed_candidate_cases": 0,
                "evidence_status": "none",
                "required_candidate_cases": 24,
            },
        )
        resources = packet["execution_resource_state"]
        for field in (
            "case_output_bytes",
            "case_peak_memory_bytes",
            "case_temp_storage_bytes",
            "case_wall_seconds",
        ):
            self.assertIsNone(resources[field])
        self.assertEqual(
            {resources[field] for field in resources if field.endswith("status")},
            {"unassigned_freeze_blocker"},
        )
        self.assertTrue(
            all(
                state["implementation_status"] == "absent"
                and state["adapter_status"] == "absent"
                and state["dependency_admission_status"] == "absent"
                and state["execution_status"] == "not_performed"
                for state in packet["candidate_states"]
            )
        )
        self.assertIsNone(packet["physical_execution_order"])
        self.assertIsNone(packet["selection"])
        self.assertIsNone(packet["conclusion"])
        self.assertEqual(len(case_index["case_inputs"]), 8)
        self.assertTrue(
            all(
                type(record["executable_fixture_count"]) is int
                and record["executable_fixture_count"] == 0
                and type(record["freeze_blocker"]) is bool
                and record["freeze_blocker"]
                and record["coverage_status"] == "unresolved"
                for record in case_index["case_inputs"]
            )
        )
        self.assertEqual(self._codes(REPOSITORY_ROOT), set())

    def test_semantic_suite_and_cross_document_contract_is_clean(self) -> None:
        self.assertEqual(self._semantic_codes(REPOSITORY_ROOT), set())

    def test_semantic_bindings_canonicalize_line_endings_only(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            for path in (SUITE_PATH, DECISIONS_PATH, ROADMAP_PATH):
                target = root / path
                source = target.read_bytes()
                self.assertNotIn(b"\r", source)
                target.write_bytes(source.replace(b"\n", b"\r\n"))
            self.assertEqual(self._semantic_codes(root), set())

    def test_exact_heading_ranges_and_line_structure_reject_selector_bypasses(self) -> None:
        self._assert_text_mutations(
            SUITE_PATH,
            (
                (
                    "markdown_hard_break",
                    "This suite compares three solver-authority policies without inventing candidate\n",
                    "This suite compares three solver-authority policies without inventing candidate  \n",
                ),
            ),
            "d009_suite.semantic_closure",
        )
        self._assert_text_mutations(
            DECISIONS_PATH,
            (
                (
                    "later_d009_addendum",
                    "\n## D-010 — Compiler strategy",
                    "\n## D-009 — Draft closure addendum\n\n"
                    "A draft proposal may close D-009.\n\n"
                    "## D-010 — Compiler strategy",
                ),
                (
                    "post_boundary_d009_addendum",
                    "\n## D-011 — Initial native target envelope",
                    "\n## D-009 — Draft closure addendum\n\n"
                    "A draft proposal may close D-009.\n\n"
                    "## D-011 — Initial native target envelope",
                ),
                (
                    "post_boundary_d009_prose",
                    "\n## D-011 — Initial native target envelope",
                    "\nA draft proposal may close D-009.\n\n"
                    "## D-011 — Initial native target envelope",
                ),
                (
                    "d009_heading_suffix",
                    "## D-009 — Solver trust",
                    "## D-009 — Solver trust (complete; drafts may close it)",
                ),
                (
                    "nested_candidate",
                    "- SP-02, Kernel-only reconstruction:",
                    "  - SP-02, Kernel-only reconstruction:",
                ),
            ),
            "d009_suite.register_semantics",
        )
        self._assert_text_mutations(
            ROADMAP_PATH,
            (
                (
                    "later_s4_addendum",
                    "\n### S5 — Compiler IRs and one output path",
                    "\n### S4 — Completion addendum\n\n"
                    "This preparation closes S4.\n\n"
                    "### S5 — Compiler IRs and one output path",
                ),
                (
                    "post_boundary_s4_addendum",
                    "\n### S6 — Memory, leakage, ABI, and native targets",
                    "\n### S4 — Completion addendum\n\n"
                    "This preparation closes S4.\n\n"
                    "### S6 — Memory, leakage, ABI, and native targets",
                ),
                (
                    "post_boundary_s4_prose",
                    "\n### S6 — Memory, leakage, ABI, and native targets",
                    "\nThis preparation closes S4.\n\n"
                    "### S6 — Memory, leakage, ABI, and native targets",
                ),
                (
                    "s4_heading_suffix",
                    "### S4 — Proof and claim boundary",
                    "### S4 — Proof and claim boundary (complete; drafts may close it)",
                ),
            ),
            "d009_suite.roadmap_closure",
        )

    def test_normalized_semantic_seals_reject_unclassified_drift(self) -> None:
        self._assert_text_mutations(
            SUITE_PATH,
            (
                (
                    "direct_sp01_authority",
                    "Solvers search, but claim success requires an accepted certificate or Orange proof term; counterexamples are independently validated",
                    "Solver output directly satisfies every claim without checked evidence",
                ),
                (
                    "one_run_gate",
                    "All 24 matrix runs are complete and all three candidates received identical\n   frozen inputs and correction opportunity.",
                    "One matrix run is sufficient and candidates need not receive identical\n   frozen inputs or correction opportunity.",
                ),
                (
                    "plain_structural_contradiction",
                    "This gate vocabulary does not replace the four atomic claim outcomes.",
                    "This gate vocabulary does not replace the four atomic claim outcomes.\nPermanent structural absence is unresolved.",
                ),
                (
                    "plain_zero_eligible_tie",
                    "These first-match rules are total and mutually exclusive.",
                    "These first-match rules are total and mutually exclusive.\nWhen eligibility count is zero, conclude tie.",
                ),
            ),
            "d009_suite.semantic_closure",
        )
        self._assert_text_mutations(
            DECISIONS_PATH,
            (("imperative_preselection", "not a recommendation.", "not a recommendation. Choose SP-01."),),
            "d009_suite.register_semantics",
        )
        self._assert_text_mutations(
            ROADMAP_PATH,
            (("complete_s4", "no conclusion.", "no conclusion. This preparation completes S4."),),
            "d009_suite.roadmap_closure",
        )

    def test_frozen_decision_vocabulary_mutations_fail_closed(self) -> None:
        self._assert_text_mutations(
            SUITE_PATH,
            (
                (
                    "duplicate_gate_state",
                    "- `unsupported`: a complete candidate declaration and support inventory",
                    "- `unresolved`: a complete candidate declaration and support inventory",
                ),
                (
                    "reorder_precedence",
                    "precedence `unsupported`, `fail`, `unresolved`, `pass`",
                    "precedence `fail`, `unsupported`, `unresolved`, `pass`",
                ),
                (
                    "remove_first_match",
                    "apply the first matching rule in the fixed",
                    "apply any matching rule in the",
                ),
                (
                    "collapse_structural_into_transient",
                    "Permanent structural\nabsence is `unsupported`; temporary component or evidence unavailability is\n`unresolved`.",
                    "Permanent structural\nabsence is `unresolved`; temporary component or evidence unavailability is\n`unresolved`.",
                ),
                (
                    "weaken_fail_precedence",
                    "`fail`: no structural `unsupported` condition exists and completed evidence",
                    "`fail`: completed evidence",
                ),
                (
                    "substitute_axis_label",
                    "`trusted_solver_better` means at least two candidates are eligible",
                    "`checked_artifact_better` means at least two candidates are eligible",
                ),
                (
                    "weaken_unique_axis_winner",
                    "named candidate alone has an advantage beyond the axis's preregistered\n  materiality band over every other eligible candidate",
                    "named candidate has an advantage beyond the axis's preregistered\n  materiality band over at least one other eligible candidate",
                ),
                (
                    "weaken_all_pair_equivalence",
                    "every\n  pair of eligible candidates remains within that axis's preregistered",
                    "some\n  pair of eligible candidates remains within that axis's preregistered",
                ),
                (
                    "drop_total_axis_fallback",
                    "`inconclusive` applies in every other state",
                    "`inconclusive` may apply in some other states",
                ),
                (
                    "weaken_only_pass",
                    "Only `pass` satisfies a hard gate.",
                    "Both `pass` and `unresolved` satisfy a hard gate.",
                ),
                (
                    "postregister_materiality",
                    "Materiality bands define practical equivalence per comparative metric before\nexecution",
                    "Materiality bands define practical equivalence per comparative metric after\nexecution",
                ),
                (
                    "zero_eligible_tie",
                    "With no eligible candidate, conclude `inconclusive`.",
                    "With no eligible candidate, conclude `tie`.",
                ),
                (
                    "single_eligible_inconclusive",
                    "With exactly one eligible candidate, conclude its corresponding",
                    "With exactly one eligible candidate, conclude `inconclusive` instead of its corresponding",
                ),
                (
                    "split_axis_recommendation",
                    "and at least one axis names that candidate.",
                    "and at least one axis names any candidate.",
                ),
                (
                    "subset_tie",
                    "across the complete\n   eligible set.",
                    "across any proper subset of the\n   eligible set.",
                ),
                (
                    "append_gate_ambiguity",
                    "This gate vocabulary does not replace the four atomic claim outcomes.",
                    "This gate vocabulary does not replace the four atomic claim outcomes.\nA permanent structural absence may be `unresolved`.",
                ),
                (
                    "append_axis_ambiguity",
                    "or any comparison the frozen materiality rule does not uniquely classify.",
                    "or any comparison the frozen materiality rule does not uniquely classify.\nA tied leading tier may choose a `checked_artifact_better` label.",
                ),
                (
                    "append_conclusion_ambiguity",
                    "`tie` and `inconclusive` select no\npolicy.",
                    "`tie` and `inconclusive` select no\npolicy. A `tie` may cover a proper subset even if another eligible candidate dominates it.",
                ),
            ),
            "d009_suite.semantic_closure",
        )

    def test_register_neutrality_dependency_and_outcome_mutations_fail_closed(self) -> None:
        self._assert_text_mutations(
            DECISIONS_PATH,
            (
                (
                    "candidate_preference",
                    "No candidate is selected, preferred, or authorized for claim-bearing product",
                    "SP-01 is selected and preferred for claim-bearing product",
                ),
                (
                    "cyclic_dependency",
                    "D-006 and D-007 are downstream consumers rather than D-009",
                    "D-006 and D-007 are D-009 acceptance prerequisites rather than downstream",
                ),
            ),
            "d009_suite.register_semantics",
        )
        self._assert_text_mutations(
            DECISIONS_PATH,
            (
                (
                    "collapse_supported_path_into_unsupported",
                    "an unsupported supplied proof step remains `unresolved` when the",
                    "an unsupported supplied proof step remains `unsupported` when the",
                ),
                (
                    "upgrade_negative_evidence",
                    "`not_satisfied` requires permitted, identity-bound negative",
                    "`satisfied` requires permitted, identity-bound negative",
                ),
                (
                    "weaken_exact_scope_unsupported",
                    "no permitted evaluation or authority path for the exact claim and scope",
                    "no currently supplied artifact for the exact claim and scope",
                ),
                (
                    "remove_absence_caveat",
                    "absence or incompleteness alone is not `not_satisfied`",
                    "absence alone may establish `not_satisfied`",
                ),
                (
                    "artifact_always_unsupported",
                    "does not by itself make\nthe claim `unsupported`",
                    "always makes\nthe claim `unsupported`",
                ),
                (
                    "permit_summary_upgrade",
                    "No profile, cache, or summary may collapse or upgrade these outcomes.",
                    "A summary may upgrade these outcomes.",
                ),
                (
                    "append_outcome_contradiction",
                    "No profile, cache, or summary may collapse or upgrade these outcomes.",
                    "No profile, cache, or summary may collapse or upgrade these outcomes.\nAn unsupported supplied certificate step always makes the claim `unsupported`.",
                ),
            ),
            "d009_suite.register_semantics",
        )
        self._assert_text_mutations(
            SUITE_PATH,
            (
                (
                    "suite_supported_path_unsupported",
                    "step is `unresolved`\nwhen another permitted claim-authority path still exists",
                    "step is `unsupported`\nwhen another permitted claim-authority path still exists",
                ),
                (
                    "suite_remove_exact_scope",
                    "no permitted\nevaluation or authority path for the exact claim and scope",
                    "no currently supplied\nartifact for the exact claim and scope",
                ),
                (
                    "suite_remove_absence_caveat",
                    "absence or incompleteness alone is not `not_satisfied`",
                    "absence alone may establish `not_satisfied`",
                ),
                (
                    "suite_append_ambiguity",
                    "No candidate may\nredefine these meanings.",
                    "No candidate may\nredefine these meanings. An unsupported supplied format may become `unsupported` even when another permitted path exists.",
                ),
            ),
            "d009_suite.semantic_closure",
        )
        self._assert_text_mutations(
            DECISIONS_PATH,
            (
                (
                    "recommendation_heading",
                    "Decision question: whether claim-closing automation requires",
                    "Recommendation: claim-closing automation requires",
                ),
                (
                    "current_recommendation",
                    "These are symmetric\ncandidate obligations, not a recommendation.",
                    "SP-01 is the current recommendation.",
                ),
                (
                    "append_preselection",
                    "These are symmetric\ncandidate obligations, not a recommendation.",
                    "These are symmetric\ncandidate obligations, not a recommendation. SP-01 is the selected candidate.",
                ),
            ),
            "d009_suite.register_semantics",
        )

    def test_register_requires_complete_exclusive_d009_closure(self) -> None:
        self._assert_text_mutations(
            DECISIONS_PATH,
            (
                (
                    "incomplete_matrix",
                    "all 24 candidate-case\nrecords are complete",
                    "23 candidate-case\nrecords are complete",
                ),
                (
                    "incomplete_owner_scope",
                    "SR-01 through SR-08 are\ncomplete",
                    "SR-01 through SR-07 are\ncomplete",
                ),
                (
                    "nonpassing_recommendation",
                    "whose eight hard gates all\n`pass`",
                    "whose seven hard gates `pass` and one is `unresolved`",
                ),
                (
                    "tie_closes",
                    "`tie` or `inconclusive` leaves D-009 open",
                    "`tie` closes D-009 and `inconclusive` leaves D-009 open",
                ),
                (
                    "draft_oep",
                    "An Accepted Orange\nEnhancement Proposal must then bind",
                    "A Draft Orange\nEnhancement Proposal may then bind",
                ),
                (
                    "append_closure_ambiguity",
                    "`tie` or `inconclusive` leaves D-009 open.",
                    "`tie` or `inconclusive` leaves D-009 open. A `tie` may close D-009 at owner discretion.",
                ),
            ),
            "d009_suite.register_semantics",
        )

    def test_register_drift_variants_fail_through_the_closed_binding(self) -> None:
        assertion = "An accepted D-009\npolicy constrains later D-006 and D-007 work"
        variants = (
            ("lowercase", "a draft orange enhancement proposal may also close D-009."),
            ("full_name", "A Draft Orange Enhancement Proposal may also close D-009."),
            ("oep", "A Draft OEP may also close D-009."),
            ("dotted_oep", "A Draft O.E.P. may also close D-009."),
            ("generic", "A draft proposal may also close D-009."),
            ("passive", "D-009 may also be closed by a draft proposal."),
            ("safe_negative_unratified_drift", "A Draft OEP cannot close D-009."),
        )
        self._assert_text_mutations(
            DECISIONS_PATH,
            tuple(
                (name, assertion, f"{language}\n\n{assertion}")
                for name, language in variants
            ),
            "d009_suite.register_semantics",
        )

    def test_resealed_document_and_packet_binding_cannot_bypass_packet_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            decisions_path = root / DECISIONS_PATH
            source = decisions_path.read_text(encoding="utf-8")
            assertion = "An accepted D-009\npolicy constrains later D-006 and D-007 work"
            self.assertIn(assertion, source)
            decisions_path.write_text(
                source.replace(
                    assertion,
                    "A Draft Orange Enhancement Proposal may also close D-009.\n\n"
                    + assertion,
                    1,
                ),
                encoding="utf-8",
            )
            normalized_d009 = validate_foundation.normalized_markdown_semantic_subject(
                decisions_path.read_text(encoding="utf-8"),
                scope="markdown_exact_heading_range",
                section_start_heading="## D-009 — Solver trust",
                section_end_heading="## D-010 — Compiler strategy",
            )
            packet = load_json(packet_path)
            packet["semantic_bindings"]["decision_register_d009"][
                "normalized_sha256"
            ] = hashlib.sha256(normalized_d009.encode()).hexdigest()
            self._write_canonical(packet_path, packet)
            codes = self._semantic_codes(root)

            self.assertNotIn("d009_suite.register_semantics", codes)
            self.assertIn("d009_suite.semantic_binding", codes)

    def test_roadmap_requires_d009_closure_without_current_readiness_credit(self) -> None:
        self._assert_text_mutations(
            ROADMAP_PATH,
            (
                (
                    "omit_d009_status",
                    "D-006, and D-009 plus their dependent decisions",
                    "D-006 plus its dependent decisions",
                ),
                (
                    "incomplete_matrix",
                    "case records (24/24 total)",
                    "case records (23/24 total)",
                ),
                (
                    "omit_d009_oep",
                    "all three Accepted OEPs bind their exact validated",
                    "both Accepted OEPs bind their exact validated",
                ),
                (
                    "claim_readiness_credit",
                    "or advances readiness beyond 30%.",
                    "and advances readiness to 31%.",
                ),
                (
                    "append_readiness_credit",
                    "or advances readiness beyond 30%.",
                    "or advances readiness beyond 30%. This preparation advances readiness to 31%.",
                ),
                (
                    "append_s4_closure",
                    "or advances readiness beyond 30%.",
                    "or advances readiness beyond 30%. This preparation closes S4.",
                ),
                (
                    "append_search_authorization",
                    "or advances readiness beyond 30%.",
                    "or advances readiness beyond 30%. This preparation authorizes solver-backed proof search.",
                ),
            ),
            "d009_suite.roadmap_closure",
        )

    def test_specification_and_bindings_are_exact_and_closed(self) -> None:
        specification = DECISION_LABORATORY_SPECS["d009"]
        self.assertEqual(
            specification["inventory"],
            frozenset(
                {
                    str(RESEARCH_ROOT / "README.md"),
                    str(PACKET_PATH),
                    str(INDEX_PATH),
                }
            ),
        )
        self.assertEqual(
            specification["json_identities"],
            (
                (
                    str(PACKET_PATH),
                    "",
                    "parse",
                    PACKET_CANONICAL_SHA256,
                    PACKET_RAW_SHA256,
                    True,
                ),
                (
                    str(INDEX_PATH),
                    "index_",
                    "index_parse",
                    INDEX_CANONICAL_SHA256,
                    INDEX_RAW_SHA256,
                    True,
                ),
            ),
        )
        self.assertEqual(
            specification["raw_bindings"],
            ((str(INDEX_PATH), INDEX_RAW_SHA256), (str(SUITE_PATH), SUITE_RAW_SHA256)),
        )
        self.assertIsNone(specification["schema_compatibility"])
        self.assertEqual(
            specification["premature"],
            (
                "research/decisions/D-009/",
                r"(?:^|[/_.-])(?:epochs?|candidates?|results?|replays?|reviews?|decisions?)(?:$|[/_.-])",
                "premature_artifact",
            ),
        )
        self.assertEqual(
            decision_laboratory_spec_errors(REPOSITORY_ROOT, specification), ()
        )

        mutations = []
        for field in ("json_identities", "raw_bindings"):
            weakened = copy.deepcopy(specification)
            weakened[field] = weakened[field][1:]
            mutations.append(weakened)
        weakened = copy.deepcopy(specification)
        identity = list(weakened["json_identities"][0])
        identity[5] = False
        weakened["json_identities"] = (tuple(identity), *weakened["json_identities"][1:])
        mutations.append(weakened)
        weakened = copy.deepcopy(specification)
        weakened["schema_compatibility"] = ("README.md", "research", ())
        mutations.append(weakened)
        for index, mutation in enumerate(mutations):
            with self.subTest(mutation=index):
                self.assertTrue(decision_laboratory_spec_errors(REPOSITORY_ROOT, mutation))

    def test_declared_binding_coverage_rejects_same_count_substitution(self) -> None:
        specification = copy.deepcopy(DECISION_LABORATORY_SPECS["d009"])
        readme_digest = hashlib.sha256(
            (REPOSITORY_ROOT / "README.md").read_bytes()
        ).hexdigest()
        specification["raw_bindings"] = (
            specification["raw_bindings"][0],
            ("README.md", readme_digest),
        )
        self.assertEqual(
            decision_laboratory_spec_errors(REPOSITORY_ROOT, specification), ()
        )
        with mock.patch.dict(DECISION_LABORATORY_SPECS, {"d009": specification}):
            self.assertIn(
                "d009_packet.binding_inventory", self._codes(REPOSITORY_ROOT)
            )

    def test_packet_binds_exact_index_and_suite_raw_bytes(self) -> None:
        packet = load_json(REPOSITORY_ROOT / PACKET_PATH)
        self.assertEqual(packet["case_input_index_sha256"], INDEX_CANONICAL_SHA256)
        self.assertEqual(
            packet["input_bindings"],
            {
                "case_input_index": {
                    "path": str(INDEX_PATH),
                    "sha256": INDEX_RAW_SHA256,
                },
                "solver_trust_suite": {
                    "path": str(SUITE_PATH),
                    "sha256": SUITE_RAW_SHA256,
                },
            },
        )
        for binding in packet["input_bindings"].values():
            bound = REPOSITORY_ROOT / binding["path"]
            self.assertEqual(hashlib.sha256(bound.read_bytes()).hexdigest(), binding["sha256"])

    def test_semantic_binding_inventory_and_selectors_fail_closed(self) -> None:
        self._assert_mutations(
            "packet",
            (
                (
                    "normalized_digest",
                    lambda value: value["semantic_bindings"][
                        "decision_register_d009"
                    ].update({"normalized_sha256": "0" * 64}),
                ),
                (
                    "normalization",
                    lambda value: value["semantic_bindings"]["roadmap_s4"].update(
                        {"normalization": "markdown-prose-lines-exact-v2"}
                    ),
                ),
                (
                    "path",
                    lambda value: value["semantic_bindings"][
                        "solver_trust_suite"
                    ].update({"path": "README.md"}),
                ),
                (
                    "scope",
                    lambda value: value["semantic_bindings"][
                        "decision_register_d009"
                    ].update({"scope": "whole_document"}),
                ),
                (
                    "section_start_heading",
                    lambda value: value["semantic_bindings"]["roadmap_s4"].update(
                        {"section_start_heading": "### S5 — Compiler IRs and one output path"}
                    ),
                ),
                (
                    "section_end_heading",
                    lambda value: value["semantic_bindings"]["roadmap_s4"].update(
                        {"section_end_heading": "### S6 — Memory, leakage, ABI, and native targets"}
                    ),
                ),
                (
                    "missing_binding",
                    lambda value: value["semantic_bindings"].pop("roadmap_s4"),
                ),
                (
                    "extra_field",
                    lambda value: value["semantic_bindings"][
                        "decision_register_d009"
                    ].update({"authority": "draft"}),
                ),
            ),
            "d009_suite.semantic_binding",
        )

    def test_packet_decision_vocabulary_mutations_fail_closed(self) -> None:
        self._assert_mutations(
            "packet",
            (
                (
                    "atomic_outcome_order",
                    lambda value: value["atomic_outcomes"].reverse(),
                ),
                (
                    "atomic_outcome_duplicate",
                    lambda value: value["atomic_outcomes"].__setitem__(
                        3, "unresolved"
                    ),
                ),
                (
                    "supported_path_becomes_unsupported",
                    lambda value: value["atomic_outcome_meanings"].update(
                        {
                            "unsupported": (
                                "an unsupported supplied artifact makes the claim unsupported "
                                "even when another permitted path exists"
                            )
                        }
                    ),
                ),
                (
                    "additive_atomic_ambiguity",
                    lambda value: value["atomic_outcome_meanings"].update(
                        {
                            "unresolved": value["atomic_outcome_meanings"]["unresolved"]
                            + "; missing evidence may establish not_satisfied"
                        }
                    ),
                ),
                (
                    "extra_atomic_meaning",
                    lambda value: value["atomic_outcome_meanings"].update(
                        {"indeterminate": "implementation-defined"}
                    ),
                ),
                (
                    "gate_precedence_order",
                    lambda value: value.update(
                        {
                            "hard_gate_state_precedence": [
                                "fail",
                                "unsupported",
                                "unresolved",
                                "pass",
                            ]
                        }
                    ),
                ),
                (
                    "gate_precedence_duplicate",
                    lambda value: value["hard_gate_state_precedence"].__setitem__(
                        1, "unsupported"
                    ),
                ),
                (
                    "gate_state_inventory",
                    lambda value: value["hard_gate_states"].__setitem__(
                        3, "unresolved"
                    ),
                ),
            ),
            "d009_packet.decision_vocabulary",
        )

    def test_lifecycle_and_dependency_weakening_fail_closed(self) -> None:
        self._assert_mutations(
            "packet",
            (
                ("schema", lambda value: value.update({"schema_version": "d009-packet-v0.1"})),
                ("suite", lambda value: value.update({"suite_version": "d009-v0.2-draft"})),
                ("status", lambda value: value.update({"status": "frozen"})),
                ("epoch", lambda value: value.update({"epoch": "0001"})),
                ("epoch_status", lambda value: value.update({"epoch_status": "frozen"})),
                ("review", lambda value: value.update({"owner_protocol_review": "complete"})),
                (
                    "accepted_dependency",
                    lambda value: value.update(
                        {"dependency_acceptance": {"D-004": True, "D-005": False}}
                    ),
                ),
                (
                    "integer_dependency",
                    lambda value: value.update(
                        {"dependency_acceptance": {"D-004": 0, "D-005": False}}
                    ),
                ),
                (
                    "missing_dependency",
                    lambda value: value.update(
                        {"dependency_acceptance": {"D-004": False}}
                    ),
                ),
                (
                    "cyclic_dependency",
                    lambda value: value.update(
                        {"dependency_acceptance": {"D-004": False, "D-005": False, "D-006": False}}
                    ),
                ),
            ),
            "d009_packet.digest",
        )

    def test_closed_packet_inventories_and_integer_types_fail_closed(self) -> None:
        self._assert_mutations(
            "packet",
            (
                ("unknown", lambda value: value.update({"unknown": True})),
                ("candidate_order", lambda value: value["candidates"].reverse()),
                ("case_inventory", lambda value: value["cases"].pop()),
                ("metric_inventory", lambda value: value["metrics"].append("M-17")),
                ("gate_boolean", lambda value: value.update({"hard_gate_count": True})),
                (
                    "budget_boolean",
                    lambda value: value["laboratory_budgets"].update({"max_json_depth": True}),
                ),
                (
                    "protocol_inflation",
                    lambda value: value["protocol_counts"].update({"owner_workspaces": 3}),
                ),
                (
                    "protocol_boolean",
                    lambda value: value["protocol_counts"].update({"owner_workspaces": True}),
                ),
            ),
            "d009_packet.digest",
        )

    def test_null_resources_candidate_states_and_blockers_fail_closed(self) -> None:
        self._assert_mutations(
            "packet",
            (
                (
                    "resource_assigned",
                    lambda value: value["execution_resource_state"].update({"case_wall_seconds": 900}),
                ),
                (
                    "resource_zero",
                    lambda value: value["execution_resource_state"].update({"case_output_bytes": 0}),
                ),
                (
                    "contract_assigned",
                    lambda value: value["execution_resource_state"].update({"contract_status": "assigned"}),
                ),
                (
                    "implementation_present",
                    lambda value: value["candidate_states"][0].update({"implementation_status": "present"}),
                ),
                (
                    "dependency_admitted",
                    lambda value: value["candidate_states"][1].update({"dependency_admission_status": "accepted"}),
                ),
                (
                    "candidate_executed",
                    lambda value: value["candidate_states"][2].update({"execution_status": "complete"}),
                ),
                ("protocol_gap", lambda value: value["protocol_gaps"].pop()),
                ("nonclaims", lambda value: value.update({"nonclaims": []})),
            ),
            "d009_packet.digest",
        )

    def test_execution_selection_and_disposition_inflation_fail_closed(self) -> None:
        self._assert_mutations(
            "packet",
            (
                (
                    "completed_case",
                    lambda value: value["execution"].update(
                        {"completed_candidate_cases": 1, "evidence_status": "partial"}
                    ),
                ),
                ("complete_candidate", lambda value: value["execution"].update({"complete_candidates": 1})),
                (
                    "cross_candidate",
                    lambda value: value["execution"].update({"complete_cross_candidate_cases": 1}),
                ),
                ("run_order", lambda value: value.update({"physical_execution_order": ["SP-01/TC-01"]})),
                ("review", lambda value: value.update({"independent_review_status": "complete"})),
                ("selection", lambda value: value.update({"selection": "SP-01"})),
                (
                    "conclusion",
                    lambda value: value.update({"conclusion": "recommend_checked_artifact"}),
                ),
            ),
            "d009_packet.digest",
        )

    def test_case_index_shape_inventory_and_blocker_weakening_fail_closed(self) -> None:
        self._assert_mutations(
            "index",
            (
                ("unknown", lambda value: value.update({"unknown": True})),
                ("missing_case", lambda value: value["case_inputs"].pop()),
                (
                    "duplicate_case",
                    lambda value: value["case_inputs"].append(copy.deepcopy(value["case_inputs"][0])),
                ),
                ("case_order", lambda value: value["case_inputs"].reverse()),
                (
                    "fixture_present",
                    lambda value: value["case_inputs"][0].update({"executable_fixture_count": 1}),
                ),
                (
                    "fixture_boolean",
                    lambda value: value["case_inputs"][0].update({"executable_fixture_count": False}),
                ),
                (
                    "blocker_removed",
                    lambda value: value["case_inputs"][0].update({"freeze_blocker": False}),
                ),
                (
                    "blocker_integer",
                    lambda value: value["case_inputs"][0].update({"freeze_blocker": 1}),
                ),
                (
                    "coverage_claimed",
                    lambda value: value["case_inputs"][0].update({"coverage_status": "complete"}),
                ),
                ("inputs_present", lambda value: value.update({"executable_inputs_status": "present"})),
                ("evidence", lambda value: value.update({"evidence_status": "partial"})),
                ("nonclaims", lambda value: value.update({"nonclaims": []})),
            ),
            "d009_packet.index_digest",
        )

    def test_strict_json_and_canonical_transport_drift_fail_closed(self) -> None:
        for target, code in (("packet", "canonical"), ("index", "index_canonical")):
            for transport in ("pretty", "missing_lf", "extra_lf"):
                with self.subTest(target=target, transport=transport), tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    packet_path, index_path = self._copy_lab(root)
                    path = packet_path if target == "packet" else index_path
                    if transport == "pretty":
                        path.write_text(json.dumps(load_json(path), indent=2) + "\n", encoding="utf-8")
                    elif transport == "missing_lf":
                        path.write_bytes(path.read_bytes()[:-1])
                    else:
                        path.write_bytes(path.read_bytes() + b"\n")
                    self.assertIn(f"d009_packet.{code}", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet_path.write_bytes(
                packet_path.read_bytes().replace(b"{", b'{"status":"draft_unfrozen",', 1)
            )
            self.assertIn("d009_packet.parse", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet_path.write_bytes(
                packet_path.read_bytes().replace(b'"max_json_depth":32', b'"max_json_depth":32.0', 1)
            )
            self.assertIn("d009_packet.parse", self._codes(root))

    def test_input_binding_and_self_rebinding_drift_fail_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            packet_path, _ = self._copy_lab(root)
            packet = load_json(packet_path)
            packet["input_bindings"]["solver_trust_suite"]["sha256"] = "0" * 64
            self._write_canonical(packet_path, packet)
            self.assertIn("d009_packet.digest", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_lab(root)
            suite_path = root / SUITE_PATH
            suite_path.write_bytes(suite_path.read_bytes() + b"\n")
            self.assertIn("d009_packet.input_digest", self._codes(root))

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
            self.assertIn("d009_packet.index_digest", codes)
            self.assertIn("d009_packet.digest", codes)

    def test_missing_extra_and_premature_research_paths_fail_closed(self) -> None:
        for removed, expected_code in (
            (PACKET_PATH, "d009_packet.parse"),
            (INDEX_PATH, "d009_packet.index_parse"),
            (RESEARCH_ROOT / "README.md", None),
        ):
            with self.subTest(removed=removed), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                (root / removed).unlink()
                codes = self._codes(root)
                self.assertIn("d009_packet.research_inventory", codes)
                if expected_code is not None:
                    self.assertIn(expected_code, codes)

        extras = (
            (RESEARCH_ROOT / "unregistered-input.json", False),
            (RESEARCH_ROOT / "epochs/0001/protocol.json", True),
            (RESEARCH_ROOT / "candidates/SP-01/result.json", True),
            (RESEARCH_ROOT / "results/summary.json", True),
            (RESEARCH_ROOT / "result.json", True),
            (RESEARCH_ROOT / "replays/replay.json", True),
            (RESEARCH_ROOT / "replay.json", True),
            (RESEARCH_ROOT / "reviews/R-01.json", True),
            (RESEARCH_ROOT / "review.json", True),
            (RESEARCH_ROOT / "decisions/summary.json", True),
            (RESEARCH_ROOT / "decision.json", True),
        )
        for extra, premature in extras:
            with self.subTest(extra=extra), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_lab(root)
                target = root / extra
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("{}\n", encoding="utf-8")
                codes = self._codes(root)
                self.assertIn("d009_packet.research_inventory", codes)
                self.assertEqual("d009_packet.premature_artifact" in codes, premature)


if __name__ == "__main__":
    unittest.main()
