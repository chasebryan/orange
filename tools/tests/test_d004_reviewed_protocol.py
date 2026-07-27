from __future__ import annotations

import copy
import json
import shutil
import tempfile
import unittest
from pathlib import Path

from tools.d004_reviewed_protocol import (
    OWNER_RECORD_PATH,
    REVIEWED_PROTOCOL_PATH,
    REVIEWED_REPLAY_PLAN_PATH,
    validate_d004_reviewed_protocol,
)
from tools.validate_foundation import FoundationValidator, canonical_json_bytes, load_json


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
D004_ROOT = Path("research/decisions/D-004")


class D004ReviewedProtocolTests(unittest.TestCase):
    @staticmethod
    def _write_reviewed(path: Path, value: object) -> None:
        path.write_bytes(canonical_json_bytes(value) + b"\n")

    def _copy_protocol(self, root: Path) -> None:
        shutil.copytree(REPOSITORY_ROOT / D004_ROOT, root / D004_ROOT)
        for relative_path in (
            Path("docs/SEMANTIC_STRATA_DECISION_SUITE.md"),
            Path("compiler/crates/orange-compiler/tests/d004_support/result_contract.rs"),
        ):
            target = root / relative_path
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(REPOSITORY_ROOT / relative_path, target)

    @staticmethod
    def _codes(root: Path) -> set[str]:
        validator = FoundationValidator(root)
        validate_d004_reviewed_protocol(validator)
        return {
            finding.code
            for finding in validator.findings
            if finding.code.startswith("d004_reviewed_protocol.")
        }

    def _assert_json_mutation(
        self,
        relative_path: Path,
        mutate,
        expected_code: str,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_protocol(root)
            target = root / relative_path
            value = load_json(target)
            mutate(value)
            self._write_reviewed(target, value)
            self.assertIn(expected_code, self._codes(root))

    def test_exact_reviewed_protocol_is_fully_bound(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_protocol(root)
            self.assertEqual(self._codes(root), set())

    def test_reviewed_json_requires_duplicate_free_canonical_bytes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_protocol(root)
            target = root / OWNER_RECORD_PATH
            source = target.read_bytes()
            target.write_bytes(b'{"schema_version":"duplicate",' + source[1:])
            self.assertIn("d004_reviewed_protocol.owner_parse", self._codes(root))

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_protocol(root)
            target = root / REVIEWED_PROTOCOL_PATH
            target.write_bytes(target.read_bytes() + b"\n")
            self.assertIn("d004_reviewed_protocol.protocol_canonical", self._codes(root))

    def test_owner_and_base_bindings_fail_closed(self) -> None:
        self._assert_json_mutation(
            OWNER_RECORD_PATH,
            lambda value: value["review_subjects"]["draft_packet"].__setitem__(
                "raw_sha256", "0" * 64
            ),
            "d004_reviewed_protocol.owner_bindings",
        )
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["bindings"]["owner_record"].__setitem__(
                "canonical_sha256", "0" * 64
            ),
            "d004_reviewed_protocol.protocol_bindings",
        )
        self._assert_json_mutation(
            REVIEWED_REPLAY_PLAN_PATH,
            lambda value: value["base_packet"].__setitem__("raw_sha256", "0" * 64),
            "d004_reviewed_protocol.replay_boundary",
        )

    def test_five_fixture_dispositions_and_counts_fail_closed(self) -> None:
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["fixture_class_dispositions"][1].__setitem__(
                "subject_count", 13
            ),
            "d004_reviewed_protocol.fixture_dispositions",
        )
        self._assert_json_mutation(
            OWNER_RECORD_PATH,
            lambda value: value["structured_disposition"]["fixture_class_reviews"].pop(),
            "d004_reviewed_protocol.owner_disposition",
        )

    def test_mapping_review_stays_hypothesis_only_and_zero_credit(self) -> None:
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["mapping_disposition"].__setitem__(
                "semantic_status", "accepted"
            ),
            "d004_reviewed_protocol.mapping_limits",
        )
        self._assert_json_mutation(
            OWNER_RECORD_PATH,
            lambda value: value["structured_disposition"]["mapping_review"].__setitem__(
                "selection", "ST-REL"
            ),
            "d004_reviewed_protocol.owner_disposition",
        )

    def test_protocol_gaps_close_but_epoch_blockers_remain(self) -> None:
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["protocol_gaps"].append("adapter missing"),
            "d004_reviewed_protocol.protocol_gaps",
        )
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value.__setitem__("epoch_freeze_blockers", []),
            "d004_reviewed_protocol.epoch_freeze_blockers",
        )

    def test_repetition_major_latin_schedule_is_exact(self) -> None:
        def duplicate_slot(value: dict[str, object]) -> None:
            schedule = value["schedule"]
            assert isinstance(schedule, list)
            schedule[25] = copy.deepcopy(schedule[0])

        self._assert_json_mutation(
            REVIEWED_REPLAY_PLAN_PATH,
            duplicate_slot,
            "d004_reviewed_protocol.schedule",
        )
        self._assert_json_mutation(
            REVIEWED_REPLAY_PLAN_PATH,
            lambda value: value.__setitem__("repetitions_per_slot", 2),
            "d004_reviewed_protocol.replay_boundary",
        )
        self._assert_json_mutation(
            REVIEWED_REPLAY_PLAN_PATH,
            lambda value: value.__setitem__("physical_execution_count", 74),
            "d004_reviewed_protocol.replay_boundary",
        )

    def test_execution_preimage_fields_are_exact_and_joined(self) -> None:
        self._assert_json_mutation(
            REVIEWED_REPLAY_PLAN_PATH,
            lambda value: value["execution_identity_preimage_fields"].remove(
                "repetition"
            ),
            "d004_reviewed_protocol.execution_preimage",
        )
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["replay_contract"][
                "execution_identity_preimage_fields"
            ].reverse(),
            "d004_reviewed_protocol.replay_contract",
        )

    def test_equality_variance_and_correction_rules_fail_closed(self) -> None:
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["replay_contract"][
                "deterministic_equality_fields"
            ].remove("execution_state"),
            "d004_reviewed_protocol.replay_contract",
        )
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["replay_contract"][
                "variable_fields_within_frozen_bounds"
            ].append("case_verdict"),
            "d004_reviewed_protocol.replay_contract",
        )
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value["correction_policy"].__setitem__(
                "max_candidate_corrections", 2
            ),
            "d004_reviewed_protocol.correction_policy",
        )

    def test_all_three_nonclaim_sets_are_closed(self) -> None:
        for relative_path, key, code in (
            (
                OWNER_RECORD_PATH,
                "nonclaims",
                "d004_reviewed_protocol.owner_nonclaims",
            ),
            (
                REVIEWED_PROTOCOL_PATH,
                "nonclaims",
                "d004_reviewed_protocol.protocol_nonclaims",
            ),
            (
                REVIEWED_REPLAY_PLAN_PATH,
                "nonclaims",
                "d004_reviewed_protocol.replay_nonclaims",
            ),
        ):
            with self.subTest(path=relative_path):
                self._assert_json_mutation(
                    relative_path,
                    lambda value, key=key: value[key].pop(),
                    code,
                )

    def test_owner_governance_and_provisional_closure_fail_closed(self) -> None:
        self._assert_json_mutation(
            OWNER_RECORD_PATH,
            lambda value: value.__setitem__("decision_status", "accepted"),
            "d004_reviewed_protocol.owner_boundary",
        )
        self._assert_json_mutation(
            OWNER_RECORD_PATH,
            lambda value: value["known_risks"].pop(),
            "d004_reviewed_protocol.owner_known_risks",
        )
        self._assert_json_mutation(
            OWNER_RECORD_PATH,
            lambda value: value["revisit_triggers"].pop(),
            "d004_reviewed_protocol.owner_revisit_triggers",
        )
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value.__setitem__(
                "implementation_closure_status", "closed"
            ),
            "d004_reviewed_protocol.protocol_boundary",
        )

    def test_old_v02_through_v05_bytes_are_immutable(self) -> None:
        immutable_inputs = (
            "d004-v0.2-named-mutations.json",
            "d004-v0.2-cross-cutting-fixture-proposals.json",
            "d004-v0.3-cross-cutting-executable-fixtures.json",
            "d004-v0.4-case-subjects.json",
            "d004-v0.5-candidate-mappings.json",
            "d004-v0.5-draft-packet.json",
        )
        for name in immutable_inputs:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self._copy_protocol(root)
                target = root / D004_ROOT / name
                target.write_bytes(target.read_bytes().rstrip(b"\n") + b" \n")
                codes = self._codes(root)
                self.assertIn("d004_reviewed_protocol.old_input_canonical", codes)
                self.assertIn("d004_reviewed_protocol.old_input_identity", codes)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._copy_protocol(root)
            target = root / "docs/SEMANTIC_STRATA_DECISION_SUITE.md"
            target.write_text(
                target.read_text(encoding="utf-8") + "\n",
                encoding="utf-8",
            )
            self.assertIn(
                "d004_reviewed_protocol.base_source_identity", self._codes(root)
            )

    def test_unknown_and_premature_result_fields_fail_closed(self) -> None:
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value.__setitem__("unknown", True),
            "d004_reviewed_protocol.protocol_schema",
        )
        self._assert_json_mutation(
            REVIEWED_PROTOCOL_PATH,
            lambda value: value.__setitem__("case_verdict", "pass"),
            "d004_reviewed_protocol.premature_result",
        )


if __name__ == "__main__":
    unittest.main()
