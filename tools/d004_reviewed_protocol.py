"""Closed validation for the reviewed, non-executable D-004 v0.6 protocol.

The helper deliberately has no dependency on :mod:`validate_foundation` so the
foundation validator can load it through its content-identified helper loader.
The supplied validator must expose ``root``, ``_read_repository_bytes``,
``_load_repository_json``, and ``add`` in the same way as ``FoundationValidator``.
"""

from __future__ import annotations

from collections import Counter
import hashlib
import json
import re
from pathlib import Path
from typing import Any


OWNER_RECORD_PATH = Path(
    "research/decisions/D-004/d004-v0.6/protocol/d004-pre-01-owner-record.json"
)
REVIEWED_PROTOCOL_PATH = Path(
    "research/decisions/D-004/d004-v0.6/protocol/reviewed-protocol.json"
)
REVIEWED_REPLAY_PLAN_PATH = Path(
    "research/decisions/D-004/d004-v0.6/protocol/reviewed-replay-plan.json"
)

OWNER_RECORD_CANONICAL_SHA256 = (
    "587c3ad11bf6e0d3dddc02cd7ba53896f54f8e08ec59ed1ece286e13fe9b0c9d"
)
OWNER_RECORD_RAW_SHA256 = (
    "cbdfa3e07245a6843100a6b17860ea5dcec39f7f341b194faeee91e2ae585f3c"
)
REVIEWED_PROTOCOL_CANONICAL_SHA256 = (
    "c67c17bdf68eb0619ec7698dd2807912251a0556931fa79b6838a9d5c6a9bd98"
)
REVIEWED_PROTOCOL_RAW_SHA256 = (
    "1111889b47edf24e88926bf8fa6770cf84ebcd1abb1d7dc2687dc42e0135fb53"
)
REVIEWED_REPLAY_PLAN_CANONICAL_SHA256 = (
    "a18084768cd05f6fcffdea724e330c0c81c5caac7816213156f4f9967fc5cb1b"
)
REVIEWED_REPLAY_PLAN_RAW_SHA256 = (
    "45632f796c7c08d26e668b277ccaff5679ccb82857732c3b8beead66198a3eb7"
)

_OLD_JSON_IDENTITIES = {
    "named_mutations": (
        "research/decisions/D-004/d004-v0.2-named-mutations.json",
        "970999d998cdc202a6caa4e2f798017416c88211a5b6b8508132a07cc9080c0c",
        "1d46d6d66c0704fcaa462c625dcac2e72150497bb075322c5e076ea42898be54",
    ),
    "cross_cutting_fixture_proposals": (
        "research/decisions/D-004/d004-v0.2-cross-cutting-fixture-proposals.json",
        "85407a4a43b5a6bf450ea905fe858482f2f79abb4cbe8ee8690bddc1753d0912",
        "d3d58cbeb0d2a90987680cd00bc70caf53518be730a71d0d55ba2a7b50544481",
    ),
    "cross_cutting_executable_fixtures": (
        "research/decisions/D-004/d004-v0.3-cross-cutting-executable-fixtures.json",
        "0516a84260bcc4d8ebb64e0cd3416deb5c43a86b7f5cd882ca757c924e575767",
        "5fea65960c47818243d41076dd96a6cab2dbd6d4038fd354a3f5ba30a12622ae",
    ),
    "case_subjects": (
        "research/decisions/D-004/d004-v0.4-case-subjects.json",
        "5b9e734b6bad7913072e87adb29c58547d67bdcb46af942eb6bbc79d0e68166e",
        "6266b8e38ad1a83fb777278fc0369844749b2915eabb40ba1ddfc9efa7c985f2",
    ),
    "candidate_mappings": (
        "research/decisions/D-004/d004-v0.5-candidate-mappings.json",
        "c967d7db8ea5049da054129367ec61cd80d729b8ce8cd34c95a76e42c67c97b8",
        "70765c64936bbb8aafd6e101fbf20c85396eb722d70e55bb9311d14bfbb15156",
    ),
    "draft_packet": (
        "research/decisions/D-004/d004-v0.5-draft-packet.json",
        "b6df1a38f8a1eb6a80a8864324c21a81cb292d4c48e1981b4547bad41933b340",
        "ec3a0a593d1dab7a6ace874dae4fd03c1ae0656cf301897ccabf51cb109c4009",
    ),
}

_DECISION_SUITE_PATH = "docs/SEMANTIC_STRATA_DECISION_SUITE.md"
_DECISION_SUITE_RAW_SHA256 = (
    "64abe8290955f889e28f8bb9ce7653a26ef71a624286aef900d4dbfc3b7eb117"
)
_RESULT_CONTRACT_PATH = (
    "compiler/crates/orange-compiler/tests/d004_support/result_contract.rs"
)
_RESULT_CONTRACT_RAW_SHA256 = (
    "f3fdb4187fa9dffb23e849208da20f112aba764cfa85df80ae578754f3c9c97a"
)
_RESULT_CONTRACT_DESCRIPTOR_SHA256 = (
    "58773e8ce29e8726a8a85203ff7e2a4b1a03f8c02bfbcd7f6056f34fe53a2f29"
)

_FIXTURE_COUNTS = (
    ("ambiguity", 5),
    ("missing-edge", 14),
    ("identity-substitution", 13),
    ("unsupported", 5),
    ("resource-exhaustion", 5),
)

_FIXTURE_DISPOSITIONS = [
    {
        "capability_credit": "none",
        "class": fixture_class,
        "scope": "bounded_suite_coverage_only",
        "status": "reviewed_sufficient",
        "subject_count": count,
    }
    for fixture_class, count in _FIXTURE_COUNTS
]

_OWNER_MAPPING_DISPOSITION = {
    "candidate_graph_count": 5,
    "capability_credit": "none",
    "evidence_credit": "none",
    "mapping_row_count": 70,
    "relationship_count": 14,
    "selection": None,
    "semantic_status": "unaccepted",
    "status": "reviewed_symmetric_falsifiable_test_hypotheses",
}

_PROTOCOL_MAPPING_DISPOSITION = {
    **_OWNER_MAPPING_DISPOSITION,
    "conformance_state": "unresolved_until_execution",
}

_FORBIDDEN_AGGREGATION = [
    "warmup",
    "retry_substitution",
    "voting",
    "averaging",
    "best_of",
    "statistical_confidence",
]

_OWNER_REPLAY_POLICY = {
    "base_candidate_case_slots": 25,
    "cache": "fresh_empty_candidate_specific_per_execution",
    "deterministic_equality_required": True,
    "forbidden_aggregation": _FORBIDDEN_AGGREGATION,
    "independent_pass_required": True,
    "measured_resource_variance": "permitted_only_within_frozen_bounds",
    "physical_order": "repetition_major_then_latin_slot_ordinal",
    "repetitions_per_slot": 3,
    "required_execution_records": 75,
}

_EXECUTION_PREIMAGE_FIELDS = [
    "schema_version",
    "suite_version",
    "epoch",
    "packet_sha256",
    "replay_plan_sha256",
    "execution_ordinal",
    "repetition",
    "logical_slot_ordinal",
    "round",
    "position",
    "candidate",
    "case",
    "input_manifest_sha256",
    "model_sha256",
    "tool_sha256",
    "dependency_manifest_sha256",
    "environment_sha256",
    "candidate_graph_sha256",
    "sr_map_sha256",
    "semantic_endpoint_sha256",
    "parameter_model_sha256",
]

_DETERMINISTIC_EQUALITY_FIELDS = [
    "digest_bound_inputs_graph_map_model_tool_dependencies_environment_and_argv",
    "execution_state",
    "log_manifest_stdout_and_stderr_bytes_lengths_and_digests",
    "normalized_observations",
    "premises_assumptions_trusted_components_and_unsupported_features",
    "candidate_graph",
    "sr_conformance_map",
    "case_verdict",
    "byte_manifest",
    "replay_expected_output_manifest_sha256",
    "measured_resources_stdout_bytes_and_stderr_bytes",
]

_VARIABLE_FIELDS = [
    "measured_resources.wall_milliseconds",
    "measured_resources.peak_memory_bytes",
    "measured_resources.temp_storage_bytes",
]

_DIAGNOSTIC_METADATA = [
    "pid",
    "timestamp",
    "absolute_temporary_path",
    "host_scheduling_data",
]

_REPLAY_CONTRACT = {
    "base_candidate_case_slots": 25,
    "cache": "fresh_empty_candidate_specific_per_execution",
    "deterministic_equality_fields": _DETERMINISTIC_EQUALITY_FIELDS,
    "diagnostic_metadata_excluded_from_semantic_payload": _DIAGNOSTIC_METADATA,
    "execution_identity_preimage_fields": _EXECUTION_PREIMAGE_FIELDS,
    "failure_rule": (
        "missing_invalid_non_successful_or_inconsistent_repetition_fails_candidate_case"
    ),
    "forbidden_aggregation": _FORBIDDEN_AGGREGATION,
    "logical_schedule": "balanced_latin_5x5_v0.5",
    "network": "denied",
    "physical_order": "repetition_major_then_latin_slot_ordinal",
    "repetitions_per_slot": 3,
    "required_execution_records": 75,
    "slot_pass_rule": (
        "all_three_repetitions_independently_pass_and_all_deterministic_fields_equal"
    ),
    "variable_fields_within_frozen_bounds": _VARIABLE_FIELDS,
}

_CORRECTION_POLICY = {
    "candidate_correction_window_owner_hours": 4,
    "candidate_local_change": (
        "one_linked_correction_revision_after_complete_first_candidate_packet"
    ),
    "correction_record_id_pattern": "D004-COR-<candidate>-01",
    "forbidden": [
        "broadening_expected_states",
        "reclassifying_replay_failure_as_domain_unsupported_or_exhausted",
        "changing_an_oracle",
        "hiding_or_replacing_prior_logs",
        "introducing_undeclared_dependencies",
        "second_correction_window",
    ],
    "max_candidate_corrections": 1,
    "prior_records": "retained_immutable",
    "rerun_scope": "all_five_candidate_cases_times_three_repetitions",
    "shared_change": "new_epoch_for_all_candidates",
    "shared_runner_or_environment_defect": "invalidate_entire_epoch",
}

_EPOCH_FREEZE_BLOCKERS = [
    "candidate adapter implementations and closed request and response schemas absent",
    "candidate semantic models endpoint inventories and parameter bindings absent",
    "exact executable tool and transitive dependency manifests absent",
    "exact deterministic environment and ordered input manifests absent",
    "enforcing isolation launcher cache reset and cleanup verification absent",
    "output normalizer payload validator and bounded resource meter absent",
    "populated result repetition-closure correction and evidence-archive parsers absent",
    "exact execution-subject revision and owner freeze record absent",
    "frozen epoch packet replay-plan identity and scheduled-execution digests absent",
]

_OWNER_NONCLAIMS = [
    "no D-004 evidence epoch frozen or candidate execution authorized",
    "no D-004 semantic-strata candidate selected or preferred",
    "no candidate graph or SR mapping accepted as Orange semantics",
    "no D-004 disposition accepted",
    "no S3b implementation authorized",
    "no execution evidence or candidate capability credit created",
    "no exact-revision implementation closure recorded",
    "no roadmap gate, release-readiness, or version-readiness movement",
]

_IMPLEMENTATION_CLOSURE_STATUS = "provisional_pending_exact_merged_revision"

_OWNER_KNOWN_RISKS = [
    "solo review may retain owner or delegated technical judgment error",
    "bounded fixture sufficiency may not survive executable adapter or semantic-model construction",
    "the content-addressed reviewed overlay is not yet bound to an exact merged implementation revision",
]

_OWNER_REVISIT_TRIGGERS = [
    "any bound review subject or protocol rule changes",
    "an executable D-004 epoch freeze is proposed",
    "independent review becomes available",
    "a fixture contradiction mapping error or replay-schedule defect is discovered",
    "the reviewed overlay is available at an exact merged revision for implementation closure",
]

_FIXTURE_REVIEW_BASES = [
    (
        "ambiguity",
        "one candidate-neutral ambiguity subject for each of five semantic cases",
        "does_not_establish_candidate_disambiguation_behavior",
    ),
    (
        "missing-edge",
        "one missing-edge subject for each of fourteen required relationships",
        "does_not_establish_adapter_edge_detection",
    ),
    (
        "identity-substitution",
        "thirteen candidate-neutral subjects spanning the catalogued identity-substitution inventory",
        "does_not_establish_identity_validation_behavior",
    ),
    (
        "unsupported",
        "one candidate-neutral unsupported subject for each of five semantic cases",
        "does_not_establish_adapter_support",
    ),
    (
        "resource-exhaustion",
        "one candidate-neutral structural resource-exhaustion subject for each of five semantic cases",
        "does_not_exercise_replay_resource_ceiling",
    ),
]

_OWNER_VALIDATION = {
    "fixture_adequacy_criteria": [
        {
            "basis": basis,
            "class": fixture_class,
            "residual_risk": residual_risk,
        }
        for fixture_class, basis, residual_risk in _FIXTURE_REVIEW_BASES
    ],
    "implementation_checks": {
        "required": [
            "strict_python_protocol_validator",
            "rust_d004_decision_suite",
            "foundation_policy_and_full_check",
            "exact_head_hosted_checks",
            "postmerge_hosted_checks",
        ],
        "status": "pending_exact_merged_revision_closure",
    },
    "mapping_hypothesis_checks": [
        "five distinct candidate graphs are present",
        "each graph contains exactly one SR row for each of fourteen required relationships",
        "all seventy rows remain unaccepted hypothesis-only inputs",
        "no candidate receives asymmetric capability or evidence credit",
    ],
    "replay_policy_checks": [
        "the reviewed plan statically contains three complete repetition-major traversals of the Latin schedule",
        "slot closure requires all three repetitions to independently pass with deterministic-field equality",
        "the policy permits only frozen bounded resource measurements to vary",
        "warmup retry substitution voting averaging best-of and statistical claims are forbidden",
    ],
    "review_subject_bindings": "exact_content_identified",
}

_PROTOCOL_NONCLAIMS = [
    "review closes protocol-review gaps only and creates no execution evidence",
    "no D-004 evidence epoch frozen or candidate execution authorized",
    "no candidate graph or SR mapping accepted as Orange semantics",
    "no semantic-strata candidate selected preferred or accepted",
    "no D-004 disposition accepted",
    "no S3b implementation authorized",
    "no independent review claimed",
    "reviewed overlay implementation closure remains provisional until an exact merged revision",
    "no roadmap gate or readiness movement",
]

_REPLAY_NONCLAIMS = [
    "reviewed schedule identities are not candidate executions or evidence",
    "no epoch packet replay identity or scheduled-execution digest exists",
    "no D-004 semantic-strata candidate selected preferred or accepted",
    "no D-004 disposition accepted",
    "no S3b implementation authorized",
    "reviewed overlay implementation closure remains provisional until an exact merged revision",
    "no roadmap gate or readiness movement",
]


def _canonical_json_bytes(value: Any) -> bytes:
    """Serialize the validator's bounded no-float I-JSON profile canonically."""

    def serialize(item: Any) -> str:
        if item is None:
            return "null"
        if item is True:
            return "true"
        if item is False:
            return "false"
        if isinstance(item, int):
            if not -(2**53) + 1 <= item <= 2**53 - 1:
                raise ValueError("integer exceeds the I-JSON interoperable range")
            return str(item)
        if isinstance(item, float):
            raise ValueError("floating-point values are forbidden")
        if isinstance(item, str):
            return json.dumps(item, ensure_ascii=False, separators=(",", ":"))
        if isinstance(item, list):
            return "[" + ",".join(serialize(child) for child in item) + "]"
        if isinstance(item, dict):
            if not all(isinstance(key, str) for key in item):
                raise TypeError("JSON object names must be strings")
            keys = sorted(item, key=lambda key: key.encode("utf-16-be"))
            return "{" + ",".join(
                f"{serialize(key)}:{serialize(item[key])}" for key in keys
            ) + "}"
        raise TypeError(f"unsupported JSON value {type(item).__name__}")

    return serialize(value).encode("utf-8")


def _json_binding(path: str, canonical_sha256: str, raw_sha256: str) -> dict[str, str]:
    return {
        "canonical_sha256": canonical_sha256,
        "path": path,
        "raw_sha256": raw_sha256,
    }


_OWNER_REVIEW_SUBJECTS = {
    name: _json_binding(path, canonical_sha256, raw_sha256)
    for name, (path, canonical_sha256, raw_sha256) in _OLD_JSON_IDENTITIES.items()
}
_OWNER_REVIEW_SUBJECTS.update(
    {
        "decision_suite": {
            "path": _DECISION_SUITE_PATH,
            "raw_sha256": _DECISION_SUITE_RAW_SHA256,
        },
        "result_contract_descriptor": {
            "generated_descriptor_sha256": _RESULT_CONTRACT_DESCRIPTOR_SHA256,
            "path": _RESULT_CONTRACT_PATH,
            "raw_sha256": _RESULT_CONTRACT_RAW_SHA256,
        },
    }
)

_PROTOCOL_BINDINGS = {
    name: _OWNER_REVIEW_SUBJECTS[name]
    for name in (
        "candidate_mappings",
        "case_subjects",
        "cross_cutting_executable_fixtures",
        "cross_cutting_fixture_proposals",
        "draft_packet",
    )
}
_PROTOCOL_BINDINGS["owner_record"] = _json_binding(
    OWNER_RECORD_PATH.as_posix(),
    OWNER_RECORD_CANONICAL_SHA256,
    OWNER_RECORD_RAW_SHA256,
)

_BASE_PACKET_BINDING = _OWNER_REVIEW_SUBJECTS["draft_packet"]
_OWNER_RECORD_BINDING = _PROTOCOL_BINDINGS["owner_record"]
_PROTOCOL_BINDING = _json_binding(
    REVIEWED_PROTOCOL_PATH.as_posix(),
    REVIEWED_PROTOCOL_CANONICAL_SHA256,
    REVIEWED_PROTOCOL_RAW_SHA256,
)


def _expected_schedule() -> list[dict[str, int | str]]:
    candidates = ["ST-REL", "ST-UNI", "ST-DUAL", "ST-MIRROR", "ST-HOST"]
    cases = ["SC-01", "SC-02", "SC-03", "SC-04", "SC-05"]
    logical: list[dict[str, int | str]] = []
    for round_index in range(5):
        for position_index in range(5):
            logical.append(
                {
                    "candidate": candidates[(round_index + position_index) % 5],
                    "case": cases[(2 * round_index + position_index) % 5],
                    "logical_slot_ordinal": len(logical) + 1,
                    "position": position_index + 1,
                    "round": round_index + 1,
                }
            )
    physical: list[dict[str, int | str]] = []
    for repetition in range(1, 4):
        for slot in logical:
            physical.append(
                {
                    **slot,
                    "execution_ordinal": len(physical) + 1,
                    "repetition": repetition,
                }
            )
    return physical


def validate_d004_reviewed_protocol(validator: Any) -> None:
    """Validate the exact reviewed D-004 protocol tranche and its immutable base.

    Findings use the ``d004_reviewed_protocol`` prefix. The function returns no
    value so it can be called directly from ``FoundationValidator`` later.
    """

    def fail(suffix: str, path: str | Path, message: str) -> None:
        validator.add(
            f"d004_reviewed_protocol.{suffix}", validator.root / path, message
        )

    def load_reviewed(
        relative_path: Path,
        label: str,
        canonical_sha256: str,
        raw_sha256: str,
    ) -> Any | None:
        path = validator.root / relative_path
        raw = validator._read_repository_bytes(path)
        if raw is None:
            fail(f"{label}_missing", relative_path, "reviewed D-004 JSON is missing")
            return None
        try:
            value = validator._load_repository_json(path)
        except (OSError, UnicodeError, ValueError, TypeError) as exc:
            fail(
                f"{label}_parse",
                relative_path,
                f"cannot strictly parse duplicate-free reviewed JSON: {exc}",
            )
            return None
        try:
            canonical = _canonical_json_bytes(value)
        except (TypeError, ValueError) as exc:
            fail(
                f"{label}_canonical",
                relative_path,
                f"reviewed JSON is outside the canonical I-JSON profile: {exc}",
            )
            return None
        if raw != canonical + b"\n":
            fail(
                f"{label}_canonical",
                relative_path,
                "reviewed JSON must be the exact canonical encoding plus one terminal LF",
            )
        if (
            hashlib.sha256(canonical).hexdigest() != canonical_sha256
            or hashlib.sha256(raw).hexdigest() != raw_sha256
        ):
            fail(
                f"{label}_identity",
                relative_path,
                "reviewed canonical or raw SHA-256 identity drifted",
            )
        return value

    owner = load_reviewed(
        OWNER_RECORD_PATH,
        "owner",
        OWNER_RECORD_CANONICAL_SHA256,
        OWNER_RECORD_RAW_SHA256,
    )
    protocol = load_reviewed(
        REVIEWED_PROTOCOL_PATH,
        "protocol",
        REVIEWED_PROTOCOL_CANONICAL_SHA256,
        REVIEWED_PROTOCOL_RAW_SHA256,
    )
    replay = load_reviewed(
        REVIEWED_REPLAY_PLAN_PATH,
        "replay",
        REVIEWED_REPLAY_PLAN_CANONICAL_SHA256,
        REVIEWED_REPLAY_PLAN_RAW_SHA256,
    )

    old_documents: dict[str, Any] = {}
    for name, (path_text, canonical_sha256, raw_sha256) in _OLD_JSON_IDENTITIES.items():
        relative_path = Path(path_text)
        path = validator.root / relative_path
        raw = validator._read_repository_bytes(path)
        if raw is None:
            fail("old_input_missing", relative_path, f"immutable {name} input is missing")
            continue
        try:
            value = validator._load_repository_json(path)
            canonical = _canonical_json_bytes(value)
        except (OSError, UnicodeError, ValueError, TypeError) as exc:
            fail(
                "old_input_parse",
                relative_path,
                f"cannot strictly parse immutable {name} input: {exc}",
            )
            continue
        if raw != canonical + b"\n":
            fail(
                "old_input_canonical",
                relative_path,
                f"immutable {name} input lost canonical JSON plus one terminal LF",
            )
        if (
            hashlib.sha256(canonical).hexdigest() != canonical_sha256
            or hashlib.sha256(raw).hexdigest() != raw_sha256
        ):
            fail(
                "old_input_identity",
                relative_path,
                f"immutable v0.2-v0.5 {name} bytes or digest drifted",
            )
        old_documents[name] = value

    for relative_path, expected_sha256, label in (
        (Path(_DECISION_SUITE_PATH), _DECISION_SUITE_RAW_SHA256, "decision suite"),
        (Path(_RESULT_CONTRACT_PATH), _RESULT_CONTRACT_RAW_SHA256, "result contract"),
    ):
        raw = validator._read_repository_bytes(validator.root / relative_path)
        if raw is None or hashlib.sha256(raw).hexdigest() != expected_sha256:
            fail(
                "base_source_identity",
                relative_path,
                f"exact owner-bound {label} source bytes drifted",
            )

    if not isinstance(owner, dict):
        if owner is not None:
            fail("owner_schema", OWNER_RECORD_PATH, "owner record root must be an object")
    else:
        owner_fields = {
            "accepted_on",
            "accepted_subject",
            "authority",
            "authorization_subject_revision",
            "decision_id",
            "decision_status",
            "delegated_technical_judgment",
            "implementation_authority",
            "implementation_closure_status",
            "known_risks",
            "nonclaims",
            "record_id",
            "record_disposition",
            "revisit_triggers",
            "review_kind",
            "review_subjects",
            "schema_version",
            "source_direction",
            "structured_disposition",
            "validation",
        }
        if set(owner) != owner_fields:
            fail("owner_schema", OWNER_RECORD_PATH, "owner-record fields are not closed")
        expected_owner_boundary = {
            "accepted_on": "2026-07-26",
            "accepted_subject": "D004-PRE-01",
            "authority": {
                "independent_review": "unavailable",
                "review_authority": "Orange Project Owner",
                "review_label": "solo-reviewed",
            },
            "authorization_subject_revision": (
                "7d09a27369649855ce987c76315271b0d34a20ef"
            ),
            "decision_id": "D-004",
            "decision_status": "proposed",
            "delegated_technical_judgment": True,
            "implementation_authority": (
                "reviewed_protocol_and_executable_freeze_prerequisite_work_only"
            ),
            "implementation_closure_status": _IMPLEMENTATION_CLOSURE_STATUS,
            "record_id": "D004-PRE-01",
            "record_disposition": "accepted",
            "review_kind": "pre_epoch_protocol",
            "schema_version": "d004-owner-protocol-record-v0.1",
            "source_direction": (
                "Accept D004-PRE-01. - you will need to make best decisions for this. "
                "this is beyond my range of knowledge."
            ),
        }
        if any(owner.get(key) != value for key, value in expected_owner_boundary.items()):
            fail(
                "owner_boundary",
                OWNER_RECORD_PATH,
                "owner authority, disposition, scope, or reviewed revision drifted",
            )
        if owner.get("review_subjects") != _OWNER_REVIEW_SUBJECTS:
            fail(
                "owner_bindings",
                OWNER_RECORD_PATH,
                "owner record must bind the exact immutable review subjects",
            )
        expected_structured_disposition = {
            "fixture_class_reviews": _FIXTURE_DISPOSITIONS,
            "mapping_review": _OWNER_MAPPING_DISPOSITION,
            "replay_policy": _OWNER_REPLAY_POLICY,
        }
        if owner.get("structured_disposition") != expected_structured_disposition:
            fail(
                "owner_disposition",
                OWNER_RECORD_PATH,
                "owner structured disposition must retain fixtures, mapping "
                "limits, and replay policy",
            )
        if owner.get("nonclaims") != _OWNER_NONCLAIMS:
            fail(
                "owner_nonclaims",
                OWNER_RECORD_PATH,
                "owner-record nonclaims were broadened, reordered, or duplicated",
            )
        if owner.get("known_risks") != _OWNER_KNOWN_RISKS:
            fail(
                "owner_known_risks",
                OWNER_RECORD_PATH,
                "solo-review known risks were weakened, reordered, or omitted",
            )
        if owner.get("revisit_triggers") != _OWNER_REVISIT_TRIGGERS:
            fail(
                "owner_revisit_triggers",
                OWNER_RECORD_PATH,
                "solo-review revisit triggers were weakened, reordered, or omitted",
            )
        if owner.get("validation") != _OWNER_VALIDATION:
            fail(
                "owner_validation",
                OWNER_RECORD_PATH,
                "fixture, mapping, replay, or implementation validation basis drifted",
            )

    if not isinstance(protocol, dict):
        if protocol is not None:
            fail("protocol_schema", REVIEWED_PROTOCOL_PATH, "protocol root must be an object")
    else:
        protocol_fields = {
            "base_suite_version",
            "bindings",
            "conclusion",
            "correction_policy",
            "epoch",
            "epoch_freeze_blockers",
            "epoch_status",
            "execution",
            "execution_authorized",
            "fixture_class_dispositions",
            "implementation_closure_status",
            "mapping_disposition",
            "nonclaims",
            "owner_protocol_review",
            "physical_execution_order",
            "protocol_gaps",
            "protocol_version",
            "replay_contract",
            "schema_version",
            "selection",
            "status",
        }
        if set(protocol) != protocol_fields:
            fail("protocol_schema", REVIEWED_PROTOCOL_PATH, "protocol fields are not closed")
        expected_protocol_boundary = {
            "base_suite_version": "d004-v0.5-draft",
            "conclusion": None,
            "epoch": None,
            "epoch_status": "unfrozen",
            "execution": {
                "complete_candidates": 0,
                "complete_cross_candidate_cases": 0,
                "completed_candidate_cases": 0,
                "evidence_status": "none",
                "required_candidate_cases": 25,
                "required_execution_records": 75,
                "result_record_count": 0,
            },
            "execution_authorized": False,
            "implementation_closure_status": _IMPLEMENTATION_CLOSURE_STATUS,
            "owner_protocol_review": "solo-reviewed",
            "physical_execution_order": "repetition_major_then_latin_slot_ordinal",
            "protocol_version": "d004-v0.6-reviewed-protocol",
            "schema_version": "d004-reviewed-protocol-tranche-v0.1",
            "selection": None,
            "status": "reviewed_not_executable",
        }
        if any(protocol.get(key) != value for key, value in expected_protocol_boundary.items()):
            fail(
                "protocol_boundary",
                REVIEWED_PROTOCOL_PATH,
                "reviewed protocol weakened its exact unfrozen, zero-evidence boundary",
            )
        if protocol.get("bindings") != _PROTOCOL_BINDINGS:
            fail(
                "protocol_bindings",
                REVIEWED_PROTOCOL_PATH,
                "reviewed protocol must bind the exact owner record and immutable base inputs",
            )
        if protocol.get("fixture_class_dispositions") != _FIXTURE_DISPOSITIONS:
            fail(
                "fixture_dispositions",
                REVIEWED_PROTOCOL_PATH,
                "the five ordered fixture dispositions or exact counts drifted",
            )
        if protocol.get("mapping_disposition") != _PROTOCOL_MAPPING_DISPOSITION:
            fail(
                "mapping_limits",
                REVIEWED_PROTOCOL_PATH,
                "mapping review must remain a 5-by-14, 70-row, zero-credit "
                "unresolved hypothesis boundary",
            )
        if protocol.get("protocol_gaps") != []:
            fail(
                "protocol_gaps",
                REVIEWED_PROTOCOL_PATH,
                "owner review closes protocol gaps exactly; the list must be empty",
            )
        blockers = protocol.get("epoch_freeze_blockers")
        if blockers != _EPOCH_FREEZE_BLOCKERS or len(set(blockers or [])) != len(
            _EPOCH_FREEZE_BLOCKERS
        ):
            fail(
                "epoch_freeze_blockers",
                REVIEWED_PROTOCOL_PATH,
                "all nonempty, unique executable-freeze blockers must remain exact",
            )
        if protocol.get("replay_contract") != _REPLAY_CONTRACT:
            fail(
                "replay_contract",
                REVIEWED_PROTOCOL_PATH,
                "repetition equality, variance, isolation, and fail-closed rules drifted",
            )
        if protocol.get("correction_policy") != _CORRECTION_POLICY:
            fail(
                "correction_policy",
                REVIEWED_PROTOCOL_PATH,
                "one-window correction and full-rerun rules drifted",
            )
        if protocol.get("nonclaims") != _PROTOCOL_NONCLAIMS:
            fail(
                "protocol_nonclaims",
                REVIEWED_PROTOCOL_PATH,
                "protocol nonclaims were broadened, reordered, or duplicated",
            )

    proposals = old_documents.get("cross_cutting_fixture_proposals")
    fixtures = old_documents.get("cross_cutting_executable_fixtures")
    if isinstance(proposals, dict) and isinstance(fixtures, dict):
        proposal_rows = proposals.get("proposals")
        fixture_rows = fixtures.get("fixtures")
        proposal_values = proposal_rows if isinstance(proposal_rows, list) else []
        fixture_values = fixture_rows if isinstance(fixture_rows, list) else []
        proposal_counts = Counter(
            row.get("class")
            for row in proposal_values
            if isinstance(row, dict)
        )
        fixture_counts = Counter(
            row.get("fixture_subject", {}).get("class")
            for row in fixture_values
            if isinstance(row, dict) and isinstance(row.get("fixture_subject"), dict)
        )
        expected_counts = Counter(dict(_FIXTURE_COUNTS))
        if (
            not isinstance(proposal_rows, list)
            or not isinstance(fixture_rows, list)
            or proposal_counts != expected_counts
            or fixture_counts != expected_counts
            or sum(fixture_counts.values()) != 42
        ):
            fail(
                "fixture_inventory",
                REVIEWED_PROTOCOL_PATH,
                "reviewed fixture counts do not match the exact 42 proposal "
                "and executable subjects",
            )

    mappings = old_documents.get("candidate_mappings")
    if isinstance(mappings, dict):
        subject = mappings.get("catalog_subject")
        graph_rows = subject.get("candidate_graphs") if isinstance(subject, dict) else None
        mapping_count = 0
        if isinstance(graph_rows, list):
            for graph_record in graph_rows:
                graph = graph_record.get("graph") if isinstance(graph_record, dict) else None
                rows = graph.get("sr_rows") if isinstance(graph, dict) else None
                if isinstance(rows, list):
                    mapping_count += len(rows)
        exact_input_boundary = {
            "epoch": None,
            "evidence_status": "none",
            "frozen": False,
            "owner_protocol_review": "none",
            "status": "draft_unreviewed_input_only",
        }
        if (
            not isinstance(subject, dict)
            or subject.get("candidate_count") != 5
            or subject.get("relationship_count") != 14
            or subject.get("mapping_row_count") != 70
            or not isinstance(graph_rows, list)
            or len(graph_rows) != 5
            or mapping_count != 70
            or any(mappings.get(key) != value for key, value in exact_input_boundary.items())
        ):
            fail(
                "mapping_input_boundary",
                REVIEWED_PROTOCOL_PATH,
                "reviewed mappings must remain exact unaccepted input-only hypotheses",
            )

    if not isinstance(replay, dict):
        if replay is not None:
            fail("replay_schema", REVIEWED_REPLAY_PLAN_PATH, "replay-plan root must be an object")
        return

    replay_fields = {
        "base_packet",
        "epoch",
        "execution_identity_preimage_fields",
        "execution_identity_schema",
        "execution_identity_status",
        "implementation_closure_status",
        "logical_schedule",
        "nonclaims",
        "owner_record",
        "physical_execution_count",
        "physical_order",
        "protocol",
        "repetitions_per_slot",
        "schedule",
        "schema_version",
        "status",
        "suite_version",
    }
    if set(replay) != replay_fields:
        fail("replay_schema", REVIEWED_REPLAY_PLAN_PATH, "replay-plan fields are not closed")
    expected_replay_boundary = {
        "base_packet": _BASE_PACKET_BINDING,
        "epoch": None,
        "execution_identity_schema": "d004-scheduled-execution-identity-v0.1",
        "execution_identity_status": (
            "unavailable_until_frozen_epoch_packet_plan_and_manifests"
        ),
        "logical_schedule": "balanced_latin_5x5_v0.5",
        "implementation_closure_status": _IMPLEMENTATION_CLOSURE_STATUS,
        "owner_record": _OWNER_RECORD_BINDING,
        "physical_execution_count": 75,
        "physical_order": "repetition_major_then_latin_slot_ordinal",
        "protocol": _PROTOCOL_BINDING,
        "repetitions_per_slot": 3,
        "schema_version": "d004-replay-plan-v0.1",
        "status": "reviewed_uninstantiated",
        "suite_version": "d004-v0.6-reviewed-protocol",
    }
    if any(replay.get(key) != value for key, value in expected_replay_boundary.items()):
        fail(
            "replay_boundary",
            REVIEWED_REPLAY_PLAN_PATH,
            "replay plan bindings or reviewed-uninstantiated boundary drifted",
        )
    if replay.get("execution_identity_preimage_fields") != _EXECUTION_PREIMAGE_FIELDS:
        fail(
            "execution_preimage",
            REVIEWED_REPLAY_PLAN_PATH,
            "scheduled-execution identity fields or order drifted",
        )
    if isinstance(protocol, dict):
        protocol_replay = protocol.get("replay_contract")
        protocol_preimage = (
            protocol_replay.get("execution_identity_preimage_fields")
            if isinstance(protocol_replay, dict)
            else None
        )
        if replay.get("execution_identity_preimage_fields") != protocol_preimage:
            fail(
                "execution_preimage_join",
                REVIEWED_REPLAY_PLAN_PATH,
                "protocol and replay plan disagree on the execution identity preimage",
            )
    expected_schedule = _expected_schedule()
    if replay.get("schedule") != expected_schedule:
        fail(
            "schedule",
            REVIEWED_REPLAY_PLAN_PATH,
            "schedule must be exactly 3 repetitions of the 25-slot balanced Latin order",
        )
    else:
        rows = replay["schedule"]
        row_keys = {
            "candidate",
            "case",
            "execution_ordinal",
            "logical_slot_ordinal",
            "position",
            "repetition",
            "round",
        }
        if (
            any(set(row) != row_keys for row in rows)
            or len({row["execution_ordinal"] for row in rows}) != 75
            or Counter((row["candidate"], row["case"]) for row in rows)
            != Counter(
                {
                    (candidate, case): 3
                    for candidate in (
                        "ST-REL",
                        "ST-UNI",
                        "ST-DUAL",
                        "ST-MIRROR",
                        "ST-HOST",
                    )
                    for case in ("SC-01", "SC-02", "SC-03", "SC-04", "SC-05")
                }
            )
        ):
            fail(
                "schedule_closure",
                REVIEWED_REPLAY_PLAN_PATH,
                "physical schedule is not unique, closed, and three-per-pair",
            )
    if replay.get("nonclaims") != _REPLAY_NONCLAIMS:
        fail(
            "replay_nonclaims",
            REVIEWED_REPLAY_PLAN_PATH,
            "replay-plan nonclaims were broadened, reordered, or duplicated",
        )

    for artifact, path in (
        (owner, OWNER_RECORD_PATH),
        (protocol, REVIEWED_PROTOCOL_PATH),
        (replay, REVIEWED_REPLAY_PLAN_PATH),
    ):
        if not isinstance(artifact, dict):
            continue
        serialized = _canonical_json_bytes(artifact).decode("utf-8")
        if re.search(
            r'"(?:observed_state|case_verdict|execution_result|evidence|readiness_credit)"\s*:',
            serialized,
        ):
            fail(
                "premature_result",
                path,
                "reviewed pre-epoch artifacts cannot persist result, evidence, or readiness fields",
            )
