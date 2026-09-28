"""D-004 v0.8 run harness (research-only).

This is the v0.7 harness (``tools/d004_run.py``, frozen with the v0.7 run)
extended to the v0.8 suite:

* ``generate`` and ``check`` keep the v0.8 case-subject catalog, adapter bundle
  and suite overlay equal to their mechanical derivation;
* ``prepare`` refuses until the overlay records the owner's distinguishing
  rule, then captures the host and derives a content-addressed epoch (packet,
  replay plan, 105 scheduled identities for 5 candidates, 7 cases and 3
  repetitions) and stages one request per logical slot;
* ``execute EPOCH`` runs the 105 executions once each, in physical order,
  under the isolated launcher, and writes one case record per execution plus
  the repetition closures and a summary that applies the preregistered rule to
  the SC-06 and SC-07 measures;
* ``verify EPOCH`` re-derives every identity, record, closure and the archive
  manifest from the archive bytes.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import selectors
import shutil
import signal
import stat
import subprocess
import sys
import tempfile
import time
import types
from pathlib import Path, PurePosixPath
from typing import Any

SUITE_VERSION = "d004-v0.8-draft"
D004 = "research/decisions/D-004"
SUITE_PATH = "docs/SEMANTIC_STRATA_DECISION_SUITE.md"
CATALOG_PATH = f"{D004}/d004-v0.8/case-subjects.json"
BUNDLE_PATH = f"{D004}/d004-v0.8/adapter-bundle.json"
OVERLAY_PATH = f"{D004}/d004-v0.8/protocol/suite-overlay.json"
V07_BUNDLE_PATH = f"{D004}/d004-v0.7/adapter-bundle.json"
V07_OVERLAY_PATH = f"{D004}/d004-v0.7/protocol/prerequisites-overlay.json"
ADAPTER_PATH = "tools/d004_v08_adapter.py"
RUNNER_PATH = "tools/d004_v08_run.py"
SANDBOX_SOURCE_PATH = "tools/fs_sandbox.c"
INPUT_PATHS = {
    "named_mutations": f"{D004}/d004-v0.2-named-mutations.json",
    "fixture_proposals": f"{D004}/d004-v0.2-cross-cutting-fixture-proposals.json",
    "fixtures": f"{D004}/d004-v0.3-cross-cutting-executable-fixtures.json",
    "case_subjects": f"{D004}/d004-v0.4-case-subjects.json",
    "candidate_mappings": f"{D004}/d004-v0.5-candidate-mappings.json",
    "draft_packet": f"{D004}/d004-v0.5-draft-packet.json",
    "owner_record": f"{D004}/d004-v0.6/protocol/d004-pre-01-owner-record.json",
    "reviewed_protocol": f"{D004}/d004-v0.6/protocol/reviewed-protocol.json",
    "reviewed_replay_plan": f"{D004}/d004-v0.6/protocol/reviewed-replay-plan.json",
    "v08_case_subjects": CATALOG_PATH,
}
CANDIDATES = ("ST-REL", "ST-UNI", "ST-DUAL", "ST-MIRROR", "ST-HOST")
V07_CASES = ("SC-01", "SC-02", "SC-03", "SC-04", "SC-05")
V08_CASES = ("SC-06", "SC-07")
CASES = (*V07_CASES, *V08_CASES)
SLOTS = len(CANDIDATES) * len(CASES)
RELATIONSHIPS = tuple(f"SR-{index:02d}" for index in range(1, 15))
DOMAIN_STATES = ("succeeded", "rejected", "unknown", "timeout", "unsupported", "exhausted")
REPETITIONS = 3
CEILINGS = {
    "wall_seconds": 900,
    "peak_memory_bytes": 4 * 1024 * 1024 * 1024,
    "temp_storage_bytes": 2 * 1024 * 1024 * 1024,
    "output_bytes": 256 * 1024 * 1024,
}
PROCESS_LIMIT = 256
ENVIRONMENT = (("LANG", "C"), ("LC_ALL", "C"), ("PATH", "/usr/bin:/bin"), ("TZ", "UTC"))
ADAPTER_FLAGS = ("-I", "-S", "-B", "-X", "utf8")
STAGED_ADAPTER = "tool/d004_v08_adapter.py"
ADAPTER_ARGS = (STAGED_ADAPTER, "in/request.json")
ARCHIVE_ROOT = Path(tempfile.gettempdir()) / "orange-d004"
NETWORK = "denied"
CACHE = "fresh_empty_candidate_specific_per_execution"
NAMESPACE = (
    "/usr/bin/unshare", "--user", "--map-current-user", "--mount", "--ipc", "--uts",
    "--pid", "--fork", "--kill-child=KILL", "--mount-proc", "--net",
)
PRIVILEGES = (
    "/usr/bin/setpriv", "--bounding-set=-all", "--inh-caps=-all",
    "--ambient-caps=-all", "--no-new-privs",
)
# A shell is PID 1 so the adapter never runs as a namespace init that ignores
# default-action signals; a signal death reaches the launcher as 128 + signal.
INIT = ("/bin/sh", "-c", '"$@"; exit $?', "d004-init")
SANDBOX_COMPILER = "/usr/bin/cc"
SANDBOX_FLAGS = (
    "-std=c17", "-O2", "-D_FORTIFY_SOURCE=3", "-fPIE", "-pie", "-Wall", "-Wextra",
    "-Werror", "-pedantic", "-Wl,-z,relro,-z,now",
)
ADAPTER_EXIT = {0: "completed", 3: "missing_input", 4: "digest_mismatch", 5: "unsupported_behavior", 6: "resource_exhaustion"}
EXHAUSTION_SIGNALS = frozenset({signal.SIGXCPU, signal.SIGXFSZ})
INABILITY = frozenset(
    {"delegated_host_unselected", "crossing_missing_native_edge", "crossing_unresolved_endpoint", "crossing_unbound_parameter"}
)
SLOT_SCHEMA = "d004-scheduled-slot-identity-v0.1"
EXECUTION_SCHEMA = "d004-scheduled-execution-identity-v0.1"
SLOT_PREIMAGE = (
    "schema_version", "suite_version", "epoch", "packet_sha256", "replay_plan_sha256",
    "ordinal", "round", "position", "candidate", "case",
)
EXECUTION_PREIMAGE = (
    "schema_version", "suite_version", "epoch", "packet_sha256", "replay_plan_sha256",
    "execution_ordinal", "repetition", "logical_slot_ordinal", "round", "position",
    "candidate", "case", "input_manifest_sha256", "model_sha256", "tool_sha256",
    "dependency_manifest_sha256", "environment_sha256", "candidate_graph_sha256",
    "sr_map_sha256", "semantic_endpoint_sha256", "parameter_model_sha256",
)
RECORD_SCHEMA = "d004-case-record-v0.3"
RECORD_FIELDS = (
    "schema_version", "suite_version", "epoch", "packet_sha256", "replay_plan_sha256",
    "execution_ordinal", "logical_slot_ordinal", "round", "position", "candidate", "case",
    "repetition", "positive_subject", "mutation_subjects", "identities", "argv",
    "environment", "resource_ceilings", "measured_resources", "execution_state",
    "observations", "log_manifest", "premises", "assumptions", "trusted_components",
    "unsupported_features", "candidate_graph", "sr_conformance_map", "case_verdict",
    "verdict_conditions", "byte_manifest", "replay", "measures", "owner_labels",
)
CONTEXT_FIELDS = ("premises", "assumptions", "trusted_components")
IDENTITY_FIELDS = (
    "scheduled_slot_sha256", "scheduled_execution_sha256", "input_manifest_sha256",
    "model_sha256", "tool_sha256", "dependency_manifest_sha256", "environment_sha256",
    "candidate_graph_sha256", "sr_map_sha256", "semantic_endpoint_sha256",
    "parameter_model_sha256", "positive_subject_sha256",
)
MEASURED_FIELDS = ("wall_milliseconds", "peak_memory_bytes", "temp_storage_bytes", "stdout_bytes", "stderr_bytes")
EXECUTION_STATE_FIELDS = ("kind", "exit_code", "signal", "stdout_truncated", "stderr_truncated", "adapter_status")
EXECUTION_KINDS = (
    "completed", "missing_input", "timeout", "resource_exhaustion", "crash",
    "digest_mismatch", "unsupported_behavior", "oversized_output",
)
OBSERVATION_FIELDS = (
    "id", "subject_id", "subject_sha256", "observation_level", "allowed_domain_states",
    "observed_state", "comparison", "required_invalidation", "observed_invalidation",
    "capability_credit", "normalized_observation_sha256", "raw_log_refs",
)
SR_ROW_FIELDS = (
    "relationship", "native_edges", "direction", "domain", "codomain", "definedness",
    "obligations", "identity_inputs", "trust_role", "failure_behavior",
    "prohibited_reverse_inferences", "observation", "applicability", "conformance_state",
    "dependent_observation_ids",
)
VERDICT_CONDITIONS = (
    "execution_completed", "resource_bounds_hold", "inventory_complete",
    "observations_matched", "invalidations_satisfied", "required_relationships_satisfied",
    "digest_joins", "replay_contract_holds",
)
# Excluded from the repetition projection: coordinates that differ by design
# and the three resource measurements the protocol lets vary within bounds.
REPETITION_VARIANT = ("execution_ordinal", "repetition")
OWNER_LABELS = {
    "producer_label": "contributor_produced",
    "review_authority": "none",
    "review_label": "unreviewed",
    "independent_review": "unavailable",
}
ARCHIVE_FILE_MODE = "0644"


class RunError(Exception):
    """A shared runner or environment defect: the epoch is invalid."""


class PayloadError(Exception):
    """The adapter wrote a response outside its closed contract."""


# ---------------------------------------------------------------------------
# Canonical JSON and identities


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("ascii")


def canonical_file(value: Any) -> bytes:
    return canonical(value) + b"\n"


def digest(value: Any) -> str:
    return hashlib.sha256(canonical(value)).hexdigest()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key}")
        result[key] = value
    return result


def _no_number(literal: str) -> Any:
    raise ValueError(f"forbidden JSON number {literal}")


def strict_json(data: bytes) -> Any:
    return json.loads(
        data.decode("utf-8"),
        object_pairs_hook=_no_duplicates,
        parse_float=_no_number,
        parse_constant=_no_number,
    )


def canonical_document(data: bytes) -> Any:
    value = strict_json(data)
    if canonical_file(value) != data:
        raise ValueError("document is not canonical JSON plus one terminal line feed")
    return value


def binding(path: str, raw: bytes) -> dict[str, str]:
    return {"path": path, "raw_sha256": sha256(raw), "canonical_sha256": sha256(canonical(strict_json(raw)))}


def load_adapter(raw: bytes, origin: str = ADAPTER_PATH) -> types.ModuleType:
    module = types.ModuleType("d004_v08_adapter")
    module.__file__ = origin
    exec(compile(raw, origin, "exec"), module.__dict__)  # noqa: S102 - content-bound research tool
    return module


class Repository:
    def __init__(self, root: Path) -> None:
        self.root = root

    def raw(self, relative: str) -> bytes:
        return (self.root / relative).read_bytes()

    def json(self, relative: str) -> Any:
        return strict_json(self.raw(relative))


# ---------------------------------------------------------------------------
# Suite inputs: oracle rows, input manifests, candidate models


def case_inventory(case: str, case_catalog: dict[str, Any], fixture_catalog: dict[str, Any]) -> list[dict[str, Any]]:
    """Ordered subject inventory of one case with the runner-held oracle rows."""
    rows = []
    for entry in [*case_catalog["positive_subjects"], *case_catalog["mutation_subjects"]]:
        if entry["case"] != case:
            continue
        subject = entry["subject"]
        expectation = entry["declared_expectation"]
        mutation = subject["kind"] == "suite-only-named-mutation"
        rows.append(
            {
                "source_catalog": "case_subject_catalog",
                "subject": subject,
                "oracle": {
                    "subject_id": entry["id"],
                    "source_catalog": "case_subject_catalog",
                    "case_scope": [case],
                    "subject_kind": subject["kind"],
                    "mutation_id": entry["mutation_id"] if mutation else None,
                    "subject_sha256": entry["subject_sha256"],
                    "relationship_scope": list(subject["relationship_scope"]),
                    "allowed_domain_states": list(expectation["allowed_domain_states"]),
                    "required_invalidation": expectation.get("required_invalidation"),
                    "observation_level": expectation["observation_level"],
                    "capability_credit": "none",
                },
            }
        )
    for entry in fixture_catalog["fixtures"]:
        subject = entry["fixture_subject"]
        if case not in subject["case_scope"]:
            continue
        expectation = entry["expected_observation"]
        rows.append(
            {
                "source_catalog": "cross_cutting_fixture_catalog",
                "subject": subject,
                "oracle": {
                    "subject_id": entry["proposal_id"],
                    "source_catalog": "cross_cutting_fixture_catalog",
                    "case_scope": list(subject["case_scope"]),
                    "subject_kind": subject["mutation_kind"],
                    "mutation_id": None,
                    "subject_sha256": entry["fixture_subject_sha256"],
                    "relationship_scope": list(subject["relationship_scope"]),
                    "allowed_domain_states": [expectation["state"]],
                    "required_invalidation": expectation["required_invalidation"],
                    "observation_level": expectation["observation_level"],
                    "capability_credit": expectation["capability_credit"],
                },
            }
        )
    if not rows or rows[0]["subject"]["kind"] != "suite-only-positive-case":
        raise RunError(f"{case} inventory does not start with its positive subject")
    for row in rows:
        if digest(row["subject"]) != row["oracle"]["subject_sha256"]:
            raise RunError(f"{row['oracle']['subject_id']} subject digest drifted")
    return rows


def required_relationships(rows: list[dict[str, Any]]) -> list[str]:
    scope = {item for row in rows for item in row["oracle"]["relationship_scope"]}
    return [item for item in RELATIONSHIPS if item in scope]


def input_manifest(case: str, rows: list[dict[str, Any]], bindings: dict[str, dict[str, str]]) -> dict[str, Any]:
    return {
        "schema_version": "d004-input-manifest-v0.1",
        "suite_version": SUITE_VERSION,
        "case": case,
        "catalogs": {
            "case_subject_catalog": bindings["v08_case_subjects" if case in V08_CASES else "case_subjects"],
            "cross_cutting_fixture_catalog": bindings["fixtures"],
        },
        "subjects": [
            {
                "subject_id": row["oracle"]["subject_id"],
                "source_catalog": row["source_catalog"],
                "subject_sha256": row["oracle"]["subject_sha256"],
                "relationship_scope": row["oracle"]["relationship_scope"],
            }
            for row in rows
        ],
        "oracle_sha256": digest([row["oracle"] for row in rows]),
        "required_relationships": required_relationships(rows),
    }


MODEL_DIGEST_FIELDS = ("graph_sha256", "sr_map_sha256", "model_sha256", "semantic_endpoint_sha256", "parameter_model_sha256")


class Suite:
    """Every repository input the epoch binds, parsed once."""

    def __init__(self, repository: Repository, adapter_raw: bytes | None = None) -> None:
        self.raw = {name: repository.raw(path) for name, path in INPUT_PATHS.items()}
        self.bindings = {name: binding(INPUT_PATHS[name], raw) for name, raw in self.raw.items()}
        documents = {name: strict_json(raw) for name, raw in self.raw.items()}
        self.protocol = documents["reviewed_protocol"]
        self.replay_plan = documents["reviewed_replay_plan"]
        self.required = {row["id"]: row for row in documents["candidate_mappings"]["catalog_subject"]["required_relationships"]}
        self.adapter_raw = repository.raw(ADAPTER_PATH) if adapter_raw is None else adapter_raw
        self.adapter = load_adapter(self.adapter_raw)
        entries = documents["candidate_mappings"]["catalog_subject"]["candidate_graphs"]
        if tuple(entry["candidate"] for entry in entries) != CANDIDATES:
            raise RunError("candidate catalog order drifted")
        self.models = {entry["candidate"]: self.adapter.derive_candidate_model(entry) for entry in entries}
        self.rows = {
            case: case_inventory(case, documents["v08_case_subjects" if case in V08_CASES else "case_subjects"], documents["fixtures"])
            for case in CASES
        }
        self.manifests = {case: input_manifest(case, self.rows[case], self.bindings) for case in CASES}


# ---------------------------------------------------------------------------
# v0.8 case subjects, measures and distinguishing rules

SUBJECT_CLASSES = (
    "pure_subject", "shared_pure_subject", "effectful_subject", "embedded_pure_subject",
    "probabilistic_experiment", "embedded_deterministic_subject", "checked_low_level_subject",
    "target_indexed_subject", "evidence_interface",
)
EVOLUTION_CHANGES = (
    ("E1", "target_operation_class", "target_indexed_subject", "add a target operation class, such as a new vector intrinsic family"),
    ("E2", "memory_model", "effectful_subject", "revise the memory model, such as a new region kind"),
    ("E3", "sampling_model", "probabilistic_experiment", "revise the sampling model, such as a new finite distribution form"),
    ("E4", "proof_evidence_interface", "evidence_interface", "revise the proof-evidence interface, such as a new judgment form"),
)
PROTECTED_CLASSES = (
    ("spec_core", ("E1", "E2", "E3"), ("pure_subject", "shared_pure_subject"), "suite 4: state, memory, target, and ambient effects cannot enter Spec Core; sampling cannot enter Specification; CT or Machine observations never silently become specification meaning"),
    ("runtime", ("E4",), ("effectful_subject", "embedded_pure_subject", "checked_low_level_subject", "target_indexed_subject"), "suite 4: proof or ghost data cannot affect runtime behavior"),
)
RELABEL_PROBES = (
    ("P-SR07", "SR-07", "shared_pure_subject", "embedded_pure_subject", "a Shared Pure subject presented as an Impl subject without the explicit embedding"),
    ("P-SR08", "SR-08", "shared_pure_subject", "embedded_deterministic_subject", "a Shared Pure subject presented as a Game subject without the explicit embedding"),
    ("P-SR09", "SR-09", "effectful_subject", "pure_subject", "an Impl subject presented as a Spec subject without a named refinement"),
    ("P-SR10", "SR-10", "effectful_subject", "checked_low_level_subject", "an Impl subject presented as a CT subject without erasure and lowering"),
    ("P-SR11", "SR-11", "checked_low_level_subject", "target_indexed_subject", "a CT subject presented as a target subject without target-parameterized preservation"),
)
V08_POSITIVES = {
    "SC-06": (
        ["SR-01", "SR-02", "SR-03", "SR-04", "SR-05", "SR-11"],
        {
            "authority": "suite-only-semantic-evolution-model-v0.1",
            "changes": [
                {"id": change, "construct_class": construct, "facet": facet, "description": text}
                for change, construct, facet, text in EVOLUTION_CHANGES
            ],
            "subject_classes": list(SUBJECT_CLASSES),
            "protected_classes": [
                {"id": group, "changes": list(changes), "classes": list(classes), "invariant": text}
                for group, changes, classes, text in PROTECTED_CLASSES
            ],
            "reidentification_policy": "authority_root_closure",
            "invariant_protection": "suite_fixed_invariants",
            "obligation_status": "open",
            "prior_identity": "fresh_identity_after_change",
        },
    ),
    "SC-07": (
        ["SR-07", "SR-08", "SR-09", "SR-10", "SR-11"],
        {
            "authority": "suite-only-relabel-probe-model-v0.1",
            "probes": [
                {"id": probe, "relationship": relationship, "from_facet": source, "to_facet": target, "description": text}
                for probe, relationship, source, target, text in RELABEL_PROBES
            ],
            "relabel_policy": "crossing_required",
            "judgment_identity": "named",
            "annotation_waiver": "none",
        },
    ),
}
V08_MUTATIONS = (
    ("SC-06", "M01", "reidentification_policy", "authority_root_closure", "none", "replace"),
    ("SC-06", "M02", "invariant_protection", "suite_fixed_invariants", "waived", "remove"),
    ("SC-06", "M03", "obligation_status", "open", "discharged_without_evidence", "replace"),
    ("SC-06", "M04", "prior_identity", "fresh_identity_after_change", "prior_identity_reused", "replace"),
    ("SC-07", "M01", "relabel_policy", "crossing_required", "silent_relabel", "replace"),
    ("SC-07", "M02", "judgment_identity", "named", "anonymous", "remove"),
    ("SC-07", "M03", "annotation_waiver", "none", "trusted_annotation", "insert"),
)


def case_catalog_document(suite_raw: bytes) -> dict[str, Any]:
    """The v0.8 case-subject catalog: SC-06 and SC-07 positives and mutations."""
    positives, mutations = [], []
    for case, (scope, model) in V08_POSITIVES.items():
        identifier = f"D004-CS-POS-{case.replace('-', '')}"
        subject = {"case": case, "id": identifier, "kind": "suite-only-positive-case", "model": model, "relationship_scope": scope, "schema_version": "d004-case-subject-v0.1"}
        positives.append(
            {
                "case": case,
                "declared_expectation": {"allowed_domain_states": ["succeeded"], "forbidden_domain_states": ["exhausted", "timeout"], "observation_level": "domain"},
                "id": identifier,
                "subject": subject,
                "subject_sha256": digest(subject),
            }
        )
    positive_digests = {entry["case"]: entry["subject_sha256"] for entry in positives}
    for case, number, target, baseline, mutated, operator in V08_MUTATIONS:
        mutation_id = f"{case}-{number}"
        identifier = f"D004-CS-MUT-{case.replace('-', '')}-{number}"
        subject = {
            "case": case,
            "id": identifier,
            "kind": "suite-only-named-mutation",
            "model": {
                "baseline_value": baseline,
                "dependent_result": {"id": "dependent_result", "required_target": target, "required_value": baseline},
                "kind": "suite-only-single-invariant-mutation",
                "mutated_value": mutated,
                "operator": operator,
                "target": target,
            },
            "mutation_id": mutation_id,
            "positive_subject_sha256": positive_digests[case],
            "relationship_scope": V08_POSITIVES[case][0],
            "schema_version": "d004-case-subject-v0.1",
        }
        mutations.append(
            {
                "case": case,
                "declared_expectation": {"allowed_domain_states": ["rejected"], "forbidden_domain_states": ["exhausted", "succeeded", "timeout"], "observation_level": "domain", "required_invalidation": "dependent_result"},
                "id": identifier,
                "mutation_id": mutation_id,
                "subject": subject,
                "subject_sha256": digest(subject),
            }
        )
    return {
        "schema_version": "d004-case-subject-catalog-v0.2",
        "suite_version": SUITE_VERSION,
        "status": "draft_unreviewed_input_only",
        "canonicalization": "RFC8785_ASCII_INTEGER_SUBSET",
        "evidence_status": "none",
        "owner_protocol_review": "none",
        "source_bindings": {"suite": {"path": SUITE_PATH, "raw_sha256": sha256(suite_raw)}},
        "positive_subject_count": len(positives),
        "mutation_subject_count": len(mutations),
        "subject_count": len(positives) + len(mutations),
        "positive_subjects": positives,
        "mutation_subjects": mutations,
        "nonclaims": [
            "subjects are suite-only inputs, not accepted Orange semantics",
            "no observed state, match, result, or verdict produced by this catalog",
            "no semantic-strata candidate selected",
            "no S3b implementation authorized",
        ],
    }


# Every measure is a count, and fewer is better. None is ever weighted.
MEASURES = (
    ("isolation_obligations", "SC-06", "open isolation obligations needed so the four suite-fixed changes leave every protected subject class unaffected"),
    ("spec_isolation_obligations", "SC-06", "the isolation obligations that protect Spec Core subject classes"),
    ("reidentified_classes", "SC-06", "subject classes re-identified across the four suite-fixed changes"),
    ("discrimination_judgments", "SC-07", "required crossings inside one authority root that need a named discrimination judgment"),
    ("semantic_definitions", "SC-06", "independent authoritative semantic definitions"),
)
DISTINGUISHING_RULES = {
    "isolation_first": ("lexicographic", ("isolation_obligations", "reidentified_classes", "discrimination_judgments", "semantic_definitions")),
    "fewest_definitions_first": ("lexicographic", ("semantic_definitions", "discrimination_judgments", "isolation_obligations", "reidentified_classes")),
    "spec_isolation_first": ("lexicographic", ("spec_isolation_obligations", "semantic_definitions", "isolation_obligations", "reidentified_classes")),
    "dominance_only": ("dominance", tuple(name for name, _, _ in MEASURES)),
}
# The owner's choice, recorded before any v0.8 epoch exists. prepare refuses
# while it is None; changing it starts a new epoch. The context says what the
# owner could see when choosing, since the measures are deterministic.
OWNER_RULE_CHOICE: dict[str, str] | None = {
    "rule": "isolation_first",
    "date": "2026-09-28",
    "source": "project thread decision card",
    "text": "Isolation first",
    "context": "The card showed the candidate each rule was predicted to select, and a local dry run with this rule had already reproduced those measures and recommend_st_rel. The choice was made before this suite's first epoch was prepared, not before its outcome could be known.",
}


def apply_rule(rule_id: str, measures: dict[str, dict[str, int]]) -> dict[str, Any]:
    """Apply one preregistered, non-compensable rule to candidates with measures."""
    kind, order = DISTINGUISHING_RULES[rule_id]
    candidates = sorted(measures)
    if kind == "lexicographic":
        key = {candidate: [measures[candidate][name] for name in order] for candidate in candidates}
        best = min(key.values()) if key else None
        remaining = [candidate for candidate in candidates if key[candidate] == best]
    else:
        def dominated(candidate: str) -> bool:
            return any(
                all(measures[other][name] <= measures[candidate][name] for name in order)
                and any(measures[other][name] < measures[candidate][name] for name in order)
                for other in candidates
                if other != candidate
            )
        remaining = [candidate for candidate in candidates if not dominated(candidate)]
    result = f"recommend_{remaining[0].lower().replace('-', '_')}" if len(remaining) == 1 else "inconclusive"
    return {"rule": rule_id, "kind": kind, "order": list(order), "compared": candidates, "remaining": remaining, "result": result}


# ---------------------------------------------------------------------------
# Committed bundle and suite overlay

RECORD_CONTRACT = {
    "record_schema": RECORD_SCHEMA,
    "record_fields": list(RECORD_FIELDS),
    "identity_fields": list(IDENTITY_FIELDS),
    "measured_resource_fields": list(MEASURED_FIELDS),
    "execution_state_fields": list(EXECUTION_STATE_FIELDS),
    "execution_kinds": list(EXECUTION_KINDS),
    "observation_fields": list(OBSERVATION_FIELDS),
    "sr_row_fields": list(SR_ROW_FIELDS),
    "verdict_conditions": list(VERDICT_CONDITIONS),
    "byte_manifest_entry_fields": ["byte_length", "mode", "path", "raw_sha256"],
    "log_manifest_entry_fields": ["id", "stderr_bytes", "stderr_sha256", "stdout_bytes", "stdout_sha256"],
    "owner_labels": OWNER_LABELS,
    "slot_identity_preimage": list(SLOT_PREIMAGE),
    "execution_identity_preimage": list(EXECUTION_PREIMAGE),
    "repetition_projection": "every record field except execution_ordinal, repetition, identities.scheduled_execution_sha256, and measured wall_milliseconds, peak_memory_bytes and temp_storage_bytes",
    "comparison_rule": "matched iff observed_state is allowed by the oracle row and an unsupported observation does not come from candidate inability",
    "candidate_inability_categories": sorted(INABILITY),
    "verdict_rule": "pass iff every verdict condition holds",
}
ISOLATION = {
    "namespaces": ["user", "mount", "ipc", "uts", "pid", "net"],
    "namespace_command": list(NAMESPACE),
    "privilege_command": list(PRIVILEGES),
    "init_command": list(INIT[:3]),
    "filesystem_sandbox": {
        "source": SANDBOX_SOURCE_PATH,
        "compiler": SANDBOX_COMPILER,
        "flags": list(SANDBOX_FLAGS),
        "rules": ["--dir /", "--ro /usr", "--ro interpreter roots outside /usr", "--ro STAGE/tool", "--ro STAGE/in", "--rw STAGE/tmp", "--rw /dev/null"],
        "limits": "fs_sandbox.c rlimits: 4 GiB address space, 600 CPU seconds, 512 MiB per file, 1024 open files, 256 processes, no core files",
    },
    "meter": "per-execution cgroup-v1 memory and pids groups hold the whole process tree; memory.limit_in_bytes and pids.max enforce the ceilings; memory.max_usage_in_bytes is the peak; a missing, unreadable or zero measurement fails the epoch",
    "wall_clock": "launcher kills the process group at 900 seconds",
    "output": "launcher stops reading and kills the process group once stdout plus stderr exceed 256 MiB",
    "temp_storage": "total bytes under STAGE/tmp after exit, the only writable tree",
    "cleanup": "read-only stage inputs are re-hashed after exit, the stage is removed, and removal is verified before the next execution",
    "fail_closed": "any namespace, sandbox, meter, staging or cleanup failure stops the run and invalidates the epoch",
}
ARCHIVE_LAYOUT = {
    "epoch": "packet.json, replay-plan.json, schedule.json, tool-manifest.json, dependency-manifest.json, environment-manifest.json, models/CANDIDATE.json, inputs/CASE.json, requests/slot-NN.json for NN = 01 to 35",
    "tool": "d004_v08_adapter.py exactly as staged",
    "executions": "NNN/stdout, NNN/stderr, NNN/record.json, NNN/diagnostics.json for NNN = execution ordinal",
    "results": "closures.json and summary.json",
    "manifest": "manifest.json lists every other archive file with mode, byte_length and raw_sha256",
    "execution_relative_paths": "byte_manifest entries named execution/stdout and execution/stderr resolve inside executions/NNN",
}


def bundle_document(suite: Suite) -> dict[str, Any]:
    adapter = suite.adapter
    contract = adapter.adapter_contract()
    return {
        "schema_version": "d004-adapter-bundle-v0.2",
        "suite_version": SUITE_VERSION,
        "status": "built_not_executed",
        "canonicalization": "RFC8785_ASCII_INTEGER_SUBSET",
        "inputs": suite.bindings,
        "adapter": {
            "path": ADAPTER_PATH,
            "raw_sha256": sha256(suite.adapter_raw),
            "argv": ["INTERPRETER", *ADAPTER_FLAGS, *ADAPTER_ARGS],
            "contract": contract,
            "contract_sha256": digest(contract),
        },
        "candidate_models": [
            {
                "candidate": candidate,
                **{field: suite.models[candidate][field] for field in MODEL_DIGEST_FIELDS},
                "crossings": {
                    kind: sum(1 for item in suite.models[candidate]["model"]["crossings"] if item["execution"] == kind)
                    for kind in ("local", "boundary", "delegated")
                },
                "discrimination_judgments": [item["relationship"] for item in suite.models[candidate]["model"]["discrimination_judgments"]],
                "semantic_definitions": suite.models[candidate]["model"]["semantic_definitions"],
            }
            for candidate in CANDIDATES
        ],
        "case_inputs": [
            {
                "case": case,
                "subject_count": len(suite.rows[case]),
                "input_manifest_sha256": digest(suite.manifests[case]),
                "oracle_sha256": suite.manifests[case]["oracle_sha256"],
                "required_relationships": suite.manifests[case]["required_relationships"],
            }
            for case in CASES
        ],
        "environment": [{"name": name, "value": value} for name, value in ENVIRONMENT],
        "network": NETWORK,
        "cache": CACHE,
        "resource_ceilings": CEILINGS,
        "isolation": ISOLATION,
        "record_contract": RECORD_CONTRACT,
        "archive_layout": ARCHIVE_LAYOUT,
        "correction": {
            "record_id_pattern": "D004-COR-<candidate>-01",
            "candidate_local_paths": {candidate: [] for candidate in CANDIDATES},
            "shared_change": "new_epoch_for_all_candidates",
        },
        "measures": [{"id": name, "case": case, "meaning": text, "better": "fewer"} for name, case, text in MEASURES],
        "distinguishing_rules": [
            {"id": rule, "kind": kind, "order": list(order)} for rule, (kind, order) in DISTINGUISHING_RULES.items()
        ],
        "nonclaims": [
            "no candidate adapter executed by this bundle",
            "no D-004 evidence epoch exists until prepare derives one",
            "no candidate graph or SR mapping accepted as Orange semantics",
            "no semantic-strata candidate selected preferred or accepted",
            "no S3b implementation authorized",
        ],
    }


AMENDMENTS = (
    ("AM-01 to AM-08", "Carry over from the v0.7 prerequisites overlay, which this overlay binds, with two replacements. AM-09 replaces AM-02's 75 scheduled identities with 105. AM-13 replaces AM-07's first sentence, since this suite binds one distinguishing rule; AM-07's second sentence still holds, so a rule written after an epoch's results exist applies only to a later epoch."),
    ("AM-09", "Two cases join SC-01 to SC-05: SC-06, semantic evolution, and SC-07, within-authority relabeling. The epoch runs 5 candidates, 7 cases and 3 repetitions, 105 executions, in a rotation schedule where every candidate meets every case once per repetition."),
    ("AM-10", "A construct class belongs to the authority root of the member that holds its facet: a view or an interface never owns a construct class apart from its parent. This follows each candidate's section 2 architecture statement."),
    ("AM-11", "A required crossing, other than SR-06 and SR-12, whose two sides share an authority root needs a named discrimination judgment. The judgments are derived from the graph and counted; a row that only asserts inspectability does not satisfy SS-G03."),
    ("AM-12", "SC-06 and SC-07 record five measures. Each is a count, fewer is better, and none is weighted against another."),
    ("AM-13", "The owner records one non-compensable distinguishing rule over the measures before prepare. The epoch packet binds it, and the summary reports what it selects among candidates that close all seven cases; if any of them lacks a complete measure set, the result is inconclusive. That result is not a D-004 recommendation under suite section 8 until the owner disposes the hard gates this run does not evaluate."),
)


def overlay_document(suite: Suite, bundle_raw: bytes, v07_bundle_raw: bytes, v07_overlay_raw: bytes) -> dict[str, Any]:
    rule = None
    if OWNER_RULE_CHOICE is not None:
        kind, order = DISTINGUISHING_RULES[OWNER_RULE_CHOICE["rule"]]
        rule = {
            "id": OWNER_RULE_CHOICE["rule"],
            "kind": kind,
            "order": list(order),
            "chosen": OWNER_RULE_CHOICE["date"],
            "source": OWNER_RULE_CHOICE["source"],
            "text": OWNER_RULE_CHOICE["text"],
            "context": OWNER_RULE_CHOICE["context"],
        }
    return {
        "schema_version": "d004-suite-overlay-v0.1",
        "protocol_version": "d004-v0.8-suite",
        "status": "suite_built_run_pending" if rule else "suite_built_rule_pending",
        "producer": "contributor_produced_not_an_owner_record",
        "base": {
            "reviewed_protocol": suite.bindings["reviewed_protocol"],
            "reviewed_replay_plan": suite.bindings["reviewed_replay_plan"],
            "v07_bundle": binding(V07_BUNDLE_PATH, v07_bundle_raw),
            "v07_overlay": binding(V07_OVERLAY_PATH, v07_overlay_raw),
        },
        "bundle": binding(BUNDLE_PATH, bundle_raw),
        "owner_directions": [
            {"date": "2026-09-28", "source": "project chat", "text": "we should have little need to freeze the development at this point or at any future point unless i specifically desire or request it."},
            {"date": "2026-09-28", "source": "project chat", "text": "old rules should not be standards for the overhaul and current modern development of Orange"},
            {"date": "2026-09-28", "source": "project thread decision card", "text": "Build v0.8 suite"},
        ],
        "amendments": [{"id": identifier, "rule": text} for identifier, text in AMENDMENTS],
        "distinguishing_rule": rule,
        "remaining": [
            *(() if rule else ("the owner's distinguishing rule",)),
            "prepare and execute one epoch and publish the archive with its manifest",
            "owner disposition of every candidate and hard gate under suite section 8",
        ],
        "nonclaims": [
            "this overlay is not an owner record and records no owner review",
            "no execution evidence exists until an archive verifies",
            "no semantic-strata candidate selected preferred or accepted",
            "no D-004 disposition accepted",
            "no S3b implementation authorized",
        ],
    }


def generated_documents_catalog(repository: Repository) -> bytes:
    return canonical_file(case_catalog_document(repository.raw(SUITE_PATH)))


def generated_documents(repository: Repository) -> tuple[bytes, bytes, bytes]:
    """Derive the catalog, bundle and overlay; the bundle binds the catalog bytes on disk."""
    catalog_raw = generated_documents_catalog(repository)
    suite = Suite(repository)
    bundle_raw = canonical_file(bundle_document(suite))
    overlay_raw = canonical_file(overlay_document(suite, bundle_raw, repository.raw(V07_BUNDLE_PATH), repository.raw(V07_OVERLAY_PATH)))
    return catalog_raw, bundle_raw, overlay_raw


# ---------------------------------------------------------------------------
# Host capture, sandbox build and isolation probe


def _file_identity(path: str) -> dict[str, Any]:
    real = os.path.realpath(path)
    data = Path(real).read_bytes()
    return {"executable": path, "realpath": real, "raw_sha256": sha256(data), "byte_length": len(data)}


def _interpreter_roots(interpreter: str) -> list[str]:
    roots = {os.path.realpath(sys.base_prefix), os.path.dirname(interpreter)}
    roots |= {item for item in ("/lib", "/lib64") if os.path.isdir(item) and not os.path.islink(item)}
    return sorted(root for root in roots if root != "/usr" and not root.startswith("/usr/"))


def build_sandbox(repository: Repository, work: Path) -> tuple[Path, dict[str, Any]]:
    source = repository.root / SANDBOX_SOURCE_PATH
    binary = work / "fs-sandbox"
    command = [SANDBOX_COMPILER, *SANDBOX_FLAGS, str(source), "-o", str(binary)]
    completed = subprocess.run(command, env=dict(ENVIRONMENT), capture_output=True, check=False)
    if completed.returncode != 0 or not binary.is_file():
        raise RunError(f"sandbox build failed: {completed.stderr.decode('utf-8', 'replace')}")
    version = subprocess.run([SANDBOX_COMPILER, "--version"], env=dict(ENVIRONMENT), capture_output=True, check=True)
    return binary, {
        "source": {"path": SANDBOX_SOURCE_PATH, "raw_sha256": sha256(source.read_bytes())},
        "compiler": {**_file_identity(SANDBOX_COMPILER), "version": version.stdout.decode("utf-8", "replace").splitlines()[0]},
        "flags": list(SANDBOX_FLAGS),
        "binary_raw_sha256": sha256(binary.read_bytes()),
    }


class Host:
    """The interpreter, sandbox binary and meter one epoch runs on."""

    def __init__(self, repository: Repository, work: Path, meter_factory: type[Meter]) -> None:
        self.repository = repository
        self.interpreter = os.path.realpath(sys.executable)
        self.sandbox, self.sandbox_identity = build_sandbox(repository, work)
        self.roots = _interpreter_roots(self.interpreter)
        self.meter_factory = meter_factory

    def adapter_argv(self) -> list[str]:
        return [self.interpreter, *ADAPTER_FLAGS, *ADAPTER_ARGS]

    def launch_argv(self, stage: Path, command: list[str], extra_ro: tuple[str, ...] = ()) -> list[str]:
        rules = ["--dir", "/", "--ro", "/usr"]
        for root in [*self.roots, *extra_ro]:
            rules += ["--ro", root]
        rules += ["--ro", str(stage / "tool"), "--ro", str(stage / "in"), "--rw", str(stage / "tmp"), "--rw", "/dev/null"]
        environment = [f"{name}={value}" for name, value in ENVIRONMENT]
        return [*NAMESPACE, *PRIVILEGES, *INIT, str(self.sandbox), *rules, "--", "/usr/bin/env", "-i", *environment, *command]

    def run_staged(self, ordinal: int, base: Path, adapter_raw: bytes, request_raw: bytes, command: list[str], extra_ro: tuple[str, ...] = ()) -> dict[str, Any]:
        stage = stage_execution(base, ordinal, adapter_raw, request_raw)
        launched = run_bounded(self.launch_argv(stage, command, extra_ro), stage, self.meter_factory())
        launched["temp_storage_bytes"] = measure_temp(stage)
        launched["cleanup"] = teardown(stage, adapter_raw, request_raw)
        stderr = launched["stderr"]
        if stderr.startswith((b"orange filesystem sandbox failed", b"unshare:", b"setpriv:", b"/usr/bin/env:")):
            raise RunError(f"launcher failure: {stderr[:200].decode('utf-8', 'replace')}")
        return launched

    def tool_manifest(self, adapter_raw: bytes) -> dict[str, Any]:
        runner_raw = self.repository.raw(RUNNER_PATH)
        return {
            "schema_version": "d004-tool-manifest-v0.1",
            "adapter": {"path": ADAPTER_PATH, "raw_sha256": sha256(adapter_raw)},
            "runner": {"path": RUNNER_PATH, "raw_sha256": sha256(runner_raw)},
            "interpreter": {
                **_file_identity(self.interpreter),
                "implementation": sys.implementation.name,
                "version": ".".join(str(part) for part in sys.version_info[:3]),
                "flags": list(ADAPTER_FLAGS),
            },
            "sandbox": self.sandbox_identity,
            "launch_tools": [_file_identity(path) for path in ("/usr/bin/unshare", "/usr/bin/setpriv", "/bin/sh", "/usr/bin/env")],
            "meter": self.meter_factory.kind,
        }

    def dependency_manifest(self, base: Path, adapter_raw: bytes) -> dict[str, Any]:
        launched = self.run_staged(0, base, adapter_raw, b"{}\n", [self.interpreter, *ADAPTER_FLAGS, "-c", DEPENDENCY_PROBE], ("/proc/self",))
        if launched["exit_status"] != 0:
            raise RunError(f"dependency probe failed: {launched['stderr'][:400].decode('utf-8', 'replace')}")
        report = json.loads(launched["stdout"])
        modules = []
        for name, origin in report["modules"]:
            entry: dict[str, Any] = {"name": name, "file": None, "raw_sha256": None}
            if origin and not origin.startswith("tool/"):
                real = os.path.realpath(origin)
                if os.path.isfile(real):
                    entry.update(file=real, raw_sha256=sha256(Path(real).read_bytes()))
            modules.append(entry)
        shared = [
            {"file": item, "raw_sha256": sha256(Path(item).read_bytes())}
            for item in report["maps"]
            if item.startswith("/") and os.path.isfile(item) and not item.startswith("/proc/")
        ]
        return {"schema_version": "d004-dependency-manifest-v0.1", "python_modules": modules, "mapped_files": shared}

    def isolation_probe(self, base: Path, adapter_raw: bytes) -> dict[str, Any]:
        launched = self.run_staged(0, base, adapter_raw, b"{}\n", [self.interpreter, *ADAPTER_FLAGS, "-c", ISOLATION_PROBE])
        if launched["exit_status"] != 0:
            raise RunError(f"isolation probe failed: {launched['stderr'][:400].decode('utf-8', 'replace')}")
        observed = json.loads(launched["stdout"])
        if observed != EXPECTED_ISOLATION:
            raise RunError(f"isolation is not enforced: {observed}")
        return observed

    def environment_manifest(self, probe: dict[str, Any]) -> dict[str, Any]:
        return {
            "schema_version": "d004-environment-manifest-v0.1",
            "entries": [{"name": name, "value": value} for name, value in ENVIRONMENT],
            "argv": self.adapter_argv(),
            "working_directory": "fresh per-execution stage root; its absolute path is diagnostic",
            "network": NETWORK,
            "cache": CACHE,
            "isolation": ISOLATION,
            "read_only_roots": ["/usr", *self.roots],
            "platform": {"system": os.uname().sysname, "machine": os.uname().machine},
            "isolation_probe": probe,
        }


DEPENDENCY_PROBE = (
    "import sys\n"
    "exec(compile(open('tool/d004_v08_adapter.py','rb').read(),'tool/d004_v08_adapter.py','exec'),"
    "{'__name__':'d004_adapter_probe','__file__':'tool/d004_v08_adapter.py'})\n"
    "import json\n"
    "maps=sorted({l.split(None,5)[5].strip() for l in open('/proc/self/maps') if len(l.split(None,5))==6})\n"
    "mods=sorted([n,getattr(m,'__file__',None)] for n,m in sys.modules.items())\n"
    "sys.stdout.write(json.dumps({'maps':maps,'modules':mods}))\n"
)
ISOLATION_PROBE = (
    "import json,os,socket\n"
    "def denied(f):\n"
    "    try:\n"
    "        f()\n"
    "    except OSError:\n"
    "        return True\n"
    "    return False\n"
    "def net():\n"
    "    s=socket.socket(); s.settimeout(2)\n"
    "    try: s.connect(('192.0.2.1',9))\n"
    "    finally: s.close()\n"
    "def write(p):\n"
    "    with open(p,'wb') as h: h.write(b'x')\n"
    "r={'network_denied':denied(net),'etc_read_denied':denied(lambda:open('/etc/passwd','rb').read()),"
    "'usr_write_denied':denied(lambda:write('/usr/d004-probe')),'tool_write_denied':denied(lambda:write('tool/probe')),"
    "'input_write_denied':denied(lambda:write('in/probe')),'tmp_write_allowed':not denied(lambda:write('tmp/probe')),"
    "'pid_namespace':os.getpid()<16,'environment':sorted(os.environ)}\n"
    "os.remove('tmp/probe')\n"
    "print(json.dumps(r,sort_keys=True))\n"
)
EXPECTED_ISOLATION = {
    "environment": sorted(name for name, _ in ENVIRONMENT),
    "etc_read_denied": True,
    "input_write_denied": True,
    "network_denied": True,
    "pid_namespace": True,
    "tmp_write_allowed": True,
    "tool_write_denied": True,
    "usr_write_denied": True,
}


# ---------------------------------------------------------------------------
# Launcher: staging, bounded execution, metering, cleanup


class Meter:
    kind = "abstract"

    def preexec(self) -> None:  # runs in the child between fork and exec
        return None

    def close(self, rusage: Any) -> dict[str, Any]:
        raise NotImplementedError


class RusageMeter(Meter):
    """Largest single-process RSS from wait4; for unit tests only, never for an epoch."""

    kind = "rusage-max-rss-test-only"

    def close(self, rusage: Any) -> dict[str, Any]:
        peak = int(rusage.ru_maxrss) * 1024
        if peak <= 0:
            raise RunError("resource meter reported no memory use")
        return {"peak_memory_bytes": peak, "memory_limit_hit": False, "process_peak": None}


class CgroupMeter(Meter):
    """Per-execution cgroup-v1 memory and pids groups covering the process tree."""

    kind = "cgroup-v1-memory-pids"

    def __init__(self) -> None:
        self.groups: dict[str, Path] = {}
        own = {}
        for line in Path("/proc/self/cgroup").read_text().splitlines():
            _, controllers, path = line.split(":", 2)
            for controller in controllers.split(","):
                own[controller] = path
        label = f"d004-{os.getpid()}-{time.monotonic_ns()}"
        for controller in ("memory", "pids"):
            if controller not in own:
                raise RunError(f"cgroup-v1 {controller} controller is unavailable")
            group = Path("/sys/fs/cgroup") / controller / own[controller].lstrip("/") / label
            try:
                group.mkdir()
            except OSError as exc:
                raise RunError(f"cannot create the {controller} cgroup: {exc}") from None
            self.groups[controller] = group
        try:
            (self.groups["memory"] / "memory.limit_in_bytes").write_text(str(CEILINGS["peak_memory_bytes"]))
            (self.groups["pids"] / "pids.max").write_text(str(PROCESS_LIMIT))
        except OSError as exc:
            self._remove()
            raise RunError(f"cannot set cgroup limits: {exc}") from None
        self._procs = [str(group / "cgroup.procs") for group in self.groups.values()]

    def preexec(self) -> None:
        for path in self._procs:
            with open(path, "w") as handle:
                handle.write(str(os.getpid()))

    def _read(self, controller: str, name: str) -> int:
        return int((self.groups[controller] / name).read_text().strip())

    def _remove(self) -> None:
        for group in self.groups.values():
            if group.exists():
                group.rmdir()

    def close(self, rusage: Any) -> dict[str, Any]:
        try:
            peak = self._read("memory", "memory.max_usage_in_bytes")
            failures = self._read("memory", "memory.failcnt")
            peak_path = self.groups["pids"] / "pids.peak"
            processes = int(peak_path.read_text().strip()) if peak_path.exists() else None
            leftovers = [True]
            for _ in range(100):
                leftovers = [(group / "cgroup.procs").read_text().strip() for group in self.groups.values()]
                if not any(leftovers):
                    break
                time.sleep(0.01)
        except (OSError, ValueError) as exc:
            raise RunError(f"resource meter could not be read: {exc}") from None
        if any(leftovers):
            raise RunError("processes outlived the execution")
        self._remove()
        if any(group.exists() for group in self.groups.values()):
            raise RunError("cgroup cleanup failed")
        if peak <= 0:
            raise RunError("resource meter reported no memory use")
        return {"peak_memory_bytes": peak, "memory_limit_hit": failures > 0, "process_peak": processes}


def run_bounded(
    argv: list[str],
    cwd: Path,
    meter: Meter,
    timeout_seconds: float = CEILINGS["wall_seconds"],
    output_cap: int = CEILINGS["output_bytes"],
) -> dict[str, Any]:
    """Run one process tree with wall-clock, output and metering bounds."""
    started = time.monotonic_ns()
    try:
        process = subprocess.Popen(
            argv,
            cwd=cwd,
            env=dict(ENVIRONMENT),
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
            preexec_fn=meter.preexec,
        )
    except OSError as exc:
        raise RunError(f"launcher could not start: {exc}") from None
    buffers = {"stdout": bytearray(), "stderr": bytearray()}
    truncated = {"stdout": False, "stderr": False}
    timed_out = oversized = False
    total = 0
    deadline = time.monotonic() + timeout_seconds
    with selectors.DefaultSelector() as selector:
        selector.register(process.stdout, selectors.EVENT_READ, "stdout")
        selector.register(process.stderr, selectors.EVENT_READ, "stderr")
        while selector.get_map() and not (timed_out or oversized):
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                timed_out = True
                break
            for key, _ in selector.select(remaining):
                chunk = os.read(key.fd, 65536)
                if not chunk:
                    selector.unregister(key.fileobj)
                    continue
                room = output_cap - total
                if len(chunk) > room:
                    buffers[key.data] += chunk[:room]
                    truncated[key.data] = oversized = True
                    break
                buffers[key.data] += chunk
                total += len(chunk)
    if timed_out or oversized:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
    _, status, rusage = os.wait4(process.pid, 0)
    process.returncode = os.waitstatus_to_exitcode(status)
    process.stdout.close()
    process.stderr.close()
    wall = (time.monotonic_ns() - started) // 1_000_000
    metered = meter.close(rusage)
    return {
        "exit_status": process.returncode,
        "stdout": bytes(buffers["stdout"]),
        "stderr": bytes(buffers["stderr"]),
        "stdout_truncated": truncated["stdout"],
        "stderr_truncated": truncated["stderr"],
        "timed_out": timed_out,
        "oversized": oversized,
        "wall_milliseconds": wall,
        **metered,
    }


def classify(launched: dict[str, Any]) -> dict[str, Any]:
    """Map one launched process tree to the closed execution state."""
    status = launched["exit_status"]
    exit_code: int | None = None
    signal_name: str | None = None
    if status < 0:
        number = -status
    elif status > 128 and status - 128 < signal.NSIG:
        number = status - 128
    else:
        number = 0
        exit_code = status
    if number:
        try:
            signal_name = signal.Signals(number).name
        except ValueError:
            signal_name = f"SIG{number}"
    if launched["timed_out"]:
        kind = "timeout"
    elif launched["oversized"]:
        kind = "oversized_output"
    elif exit_code is not None:
        kind = ADAPTER_EXIT.get(exit_code, "crash")
    elif number in EXHAUSTION_SIGNALS or launched["memory_limit_hit"]:
        kind = "resource_exhaustion"
    else:
        kind = "crash"
    finished = exit_code is not None and not (launched["timed_out"] or launched["oversized"])
    return {
        "kind": kind,
        "exit_code": exit_code,
        "signal": signal_name,
        "stdout_truncated": launched["stdout_truncated"],
        "stderr_truncated": launched["stderr_truncated"],
        "adapter_status": "executed" if finished and exit_code == 0 else "failed",
    }


def stage_execution(base: Path, ordinal: int, adapter_raw: bytes, request_raw: bytes) -> Path:
    root = base / f"x{ordinal:03d}"
    if root.exists():
        raise RunError(f"stage {root.name} already exists")
    root.mkdir(mode=0o700)
    for name, data in ((STAGED_ADAPTER, adapter_raw), ("in/request.json", request_raw)):
        target = root / name
        target.parent.mkdir(mode=0o700)
        target.write_bytes(data)
        target.chmod(0o444)
        target.parent.chmod(0o555)
    (root / "tmp").mkdir(mode=0o700)
    return root


def measure_temp(root: Path) -> int:
    tmp = root / "tmp"
    if not tmp.is_dir() or tmp.is_symlink():
        raise RunError("temporary storage tree is missing")
    total = 0
    for directory, _, files in os.walk(tmp):
        for name in files:
            total += os.lstat(os.path.join(directory, name)).st_size
    return total


def teardown(root: Path, adapter_raw: bytes, request_raw: bytes) -> dict[str, bool]:
    expected = {"tool": {PurePosixPath(STAGED_ADAPTER).name: adapter_raw}, "in": {"request.json": request_raw}}
    for directory, files in expected.items():
        observed = {entry.name: entry for entry in (root / directory).iterdir()}
        if set(observed) != set(files) or any(
            observed[name].is_symlink() or observed[name].read_bytes() != data for name, data in files.items()
        ):
            raise RunError(f"read-only stage inputs changed under {directory}")
        (root / directory).chmod(0o700)
    try:
        shutil.rmtree(root)
    except OSError as exc:
        raise RunError(f"stage cleanup failed: {exc}") from None
    if root.exists():
        raise RunError("stage cleanup failed")
    return {"inputs_unchanged": True, "stage_removed": True}


# ---------------------------------------------------------------------------
# Epoch derivation


def epoch_packet(
    bundle: dict[str, str],
    overlay: dict[str, str],
    runner_raw: bytes,
    tool: dict[str, Any],
    dependency: dict[str, Any],
    environment: dict[str, Any],
    suite: Suite,
    distinguishing_rule: dict[str, Any],
) -> dict[str, Any]:
    core = {
        "schema_version": "d004-epoch-packet-v0.2",
        "suite_version": SUITE_VERSION,
        "derivation": "mechanical_content_addressed",
        "bundle": bundle,
        "overlay": overlay,
        "runner": {"path": RUNNER_PATH, "raw_sha256": sha256(runner_raw)},
        "tool_manifest_sha256": digest(tool),
        "dependency_manifest_sha256": digest(dependency),
        "environment_sha256": digest(environment),
        "candidate_models": [
            {"candidate": candidate, **{field: suite.models[candidate][field] for field in MODEL_DIGEST_FIELDS}}
            for candidate in CANDIDATES
        ],
        "input_manifests": [{"case": case, "input_manifest_sha256": digest(suite.manifests[case])} for case in CASES],
        "resource_ceilings": CEILINGS,
        "logical_schedule": "rotation_5x7_v0.8",
        "physical_order": "repetition_major_then_rotation_slot_ordinal",
        "repetitions_per_slot": REPETITIONS,
        "selection": {"rule": None, "distinguishing_rule": distinguishing_rule, "tie_break": "forbidden_after_results_exist"},
    }
    return {**core, "epoch": f"d004-e-{digest(core)[:20]}"}


def epoch_replay_plan(packet: dict[str, Any], suite: Suite) -> dict[str, Any]:
    return {
        "schema_version": "d004-epoch-replay-plan-v0.2",
        "suite_version": SUITE_VERSION,
        "epoch": packet["epoch"],
        "packet_sha256": digest(packet),
        "base_replay_plan": suite.bindings["reviewed_replay_plan"],
        "schedule": rotation_schedule(),
        "slot_identity_schema": SLOT_SCHEMA,
        "slot_identity_preimage_fields": list(SLOT_PREIMAGE),
        "execution_identity_schema": EXECUTION_SCHEMA,
        "execution_identity_preimage_fields": list(EXECUTION_PREIMAGE),
        "network": NETWORK,
        "cache": CACHE,
        "forbidden_aggregation": suite.protocol["replay_contract"]["forbidden_aggregation"],
    }


def rotation_schedule() -> list[dict[str, Any]]:
    """Seven rounds of five positions: every round holds every candidate once,
    and every candidate meets every case exactly once per repetition."""
    logical = []
    for round_index in range(len(CASES)):
        for position in range(len(CANDIDATES)):
            candidate_index = (round_index + position) % len(CANDIDATES)
            logical.append(
                {
                    "candidate": CANDIDATES[candidate_index],
                    "case": CASES[(round_index + 3 * candidate_index) % len(CASES)],
                    "logical_slot_ordinal": len(CANDIDATES) * round_index + position + 1,
                    "position": position + 1,
                    "round": round_index + 1,
                }
            )
    return [
        {**slot, "execution_ordinal": SLOTS * (repetition - 1) + slot["logical_slot_ordinal"], "repetition": repetition}
        for repetition in range(1, REPETITIONS + 1)
        for slot in logical
    ]


def schedule_identities(
    packet: dict[str, Any], plan: dict[str, Any], suite: Suite, tool: dict[str, Any], dependency: dict[str, Any], environment: dict[str, Any]
) -> list[dict[str, Any]]:
    if plan["schedule"] != rotation_schedule():
        raise RunError("epoch schedule is not the v0.8 rotation schedule")
    packet_sha, plan_sha = digest(packet), digest(plan)
    rows = []
    for row in plan["schedule"]:
        candidate, case = row["candidate"], row["case"]
        model = suite.models[candidate]
        slot = {
            "schema_version": SLOT_SCHEMA, "suite_version": SUITE_VERSION, "epoch": packet["epoch"],
            "packet_sha256": packet_sha, "replay_plan_sha256": plan_sha, "ordinal": row["logical_slot_ordinal"],
            "round": row["round"], "position": row["position"], "candidate": candidate, "case": case,
        }
        execution = {
            "schema_version": EXECUTION_SCHEMA, "suite_version": SUITE_VERSION, "epoch": packet["epoch"],
            "packet_sha256": packet_sha, "replay_plan_sha256": plan_sha,
            "execution_ordinal": row["execution_ordinal"], "repetition": row["repetition"],
            "logical_slot_ordinal": row["logical_slot_ordinal"], "round": row["round"], "position": row["position"],
            "candidate": candidate, "case": case, "input_manifest_sha256": digest(suite.manifests[case]),
            "model_sha256": model["model_sha256"], "tool_sha256": digest(tool),
            "dependency_manifest_sha256": digest(dependency), "environment_sha256": digest(environment),
            "candidate_graph_sha256": model["graph_sha256"], "sr_map_sha256": model["sr_map_sha256"],
            "semantic_endpoint_sha256": model["semantic_endpoint_sha256"],
            "parameter_model_sha256": model["parameter_model_sha256"],
        }
        assert tuple(slot) == SLOT_PREIMAGE and tuple(execution) == EXECUTION_PREIMAGE
        rows.append({**row, "scheduled_slot_sha256": digest(slot), "scheduled_execution_sha256": digest(execution)})
    return rows


def request_document(suite: Suite, packet: dict[str, Any], plan: dict[str, Any], row: dict[str, Any]) -> dict[str, Any]:
    case = row["case"]
    return {
        "schema_version": suite.adapter.REQUEST_SCHEMA,
        "suite_version": SUITE_VERSION,
        "execution": {
            "epoch": packet["epoch"], "packet_sha256": digest(packet), "replay_plan_sha256": digest(plan),
            "scheduled_slot_sha256": row["scheduled_slot_sha256"], "logical_slot_ordinal": row["logical_slot_ordinal"],
            "round": row["round"], "position": row["position"], "candidate": row["candidate"], "case": case,
        },
        "candidate_model": suite.models[row["candidate"]],
        "subjects": [
            {
                "subject_id": item["oracle"]["subject_id"],
                "subject_sha256": item["oracle"]["subject_sha256"],
                "source_catalog": item["source_catalog"],
                "subject": item["subject"],
            }
            for item in suite.rows[case]
        ],
    }


# ---------------------------------------------------------------------------
# Payload validation and record construction


def _closed(value: Any, fields: Any, where: str) -> dict[str, Any]:
    if not isinstance(value, dict) or set(value) != set(fields):
        raise PayloadError(f"{where} is not the closed object")
    return value


def _check(condition: bool, detail: str) -> None:
    if not condition:
        raise PayloadError(detail)


def validate_response(contract: dict[str, Any], stdout: bytes, stderr: bytes, request_raw: bytes, rows: list[dict[str, Any]], model: dict[str, Any]) -> dict[str, Any]:
    """Validate one adapter response against the closed contract and the inventory."""
    _check(stderr == b"", "stderr is not empty on success")
    try:
        response = canonical_document(stdout)
    except (UnicodeError, ValueError) as exc:
        raise PayloadError(f"stdout is not canonical JSON: {exc}") from None
    request = strict_json(request_raw)
    execution = request["execution"]
    _closed(response, contract["response_fields"], "response")
    _check(response["schema_version"] == contract["response_schema"], "response schema drift")
    _check(response["request_sha256"] == sha256(request_raw[:-1]), "response does not bind the request")
    _check((response["candidate"], response["case"]) == (execution["candidate"], execution["case"]), "response coordinate drift")
    observations = response["observations"]
    _check(isinstance(observations, list) and len(observations) == len(rows), "observation inventory is incomplete")
    for observation, row in zip(observations, rows):
        oracle = row["oracle"]
        _closed(observation, contract["response_observation_fields"], "observation")
        normalized = _closed(observation["normalized_observation"], contract["normalized_observation_fields"], "normalized observation")
        _check(observation["id"] == f"obs-{oracle['subject_id']}", "observation order or identifier drift")
        _check(observation["subject_id"] == oracle["subject_id"] and observation["subject_sha256"] == oracle["subject_sha256"], "observation subject drift")
        _check(observation["observed_state"] in contract["domain_states"], "unknown domain state")
        _check(observation["observed_invalidation"] in contract["observed_invalidation_states"], "unknown invalidation state")
        _check(observation["normalized_observation_sha256"] == digest(normalized), "normalized observation digest mismatch")
        for field in ("subject_id", "subject_sha256", "observed_state", "observed_invalidation"):
            _check(normalized[field] == observation[field], f"normalized {field} disagrees")
        _check(normalized["observation_level"] == "domain", "observation level drift")
        _check(isinstance(normalized["decision"], dict) and "category" in normalized["decision"], "normalized decision is malformed")
        _check(isinstance(normalized["values"], dict), "normalized values is not an object")
    identifiers = [observation["id"] for observation in observations]
    rows_by_relationship = response["sr_conformance"]
    _check(isinstance(rows_by_relationship, list) and [item.get("relationship") for item in rows_by_relationship] == list(RELATIONSHIPS), "SR rows are not SR-01 through SR-14")
    crossings = {item["relationship"]: item for item in model["model"]["crossings"]}
    for item in rows_by_relationship:
        _closed(item, contract["response_sr_fields"], "SR row")
        relationship = item["relationship"]
        _check(item["native_edges"] == crossings[relationship]["native_edges"], "SR native edges drift")
        _check(item["conformance_state"] in contract["sr_conformance_states"], "unknown conformance state")
        expected = [f"obs-{row['oracle']['subject_id']}" for row in rows if relationship in row["oracle"]["relationship_scope"]]
        _check(item["dependent_observation_ids"] == expected, "SR dependencies are not derived from subject scopes")
    for name in CONTEXT_FIELDS:
        entries = response[name]
        _check(isinstance(entries, list), f"{name} is not a list")
        seen = set()
        for entry in entries:
            _closed(entry, contract["context_entry_fields"], name)
            _check(entry["id"] not in seen, f"duplicate {name} identifier")
            seen.add(entry["id"])
            _check(entry["sha256"] == digest({"description": entry["description"], "id": entry["id"], "status": entry["status"]}), f"{name} digest mismatch")
    unsupported = response["unsupported_features"]
    _check(isinstance(unsupported, list), "unsupported_features is not a list")
    for entry in unsupported:
        _closed(entry, contract["unsupported_feature_fields"], "unsupported feature")
        _check(entry["state"] == "unsupported", "unsupported feature state drift")
        _check(set(entry["dependent_result"]) <= set(identifiers), "unsupported feature names an unknown observation")
    return response


class EpochContext:
    """Everything a record needs, loaded from (and checked against) one archive."""

    def __init__(self, archive: Path, repository: Repository) -> None:
        self.archive = archive
        epoch = archive / "epoch"
        read = lambda name: canonical_document((epoch / name).read_bytes())  # noqa: E731
        self.packet = read("packet.json")
        # Execute and verify both judge an epoch with the repository's harness and bundle, so both must be the bound ones.
        for field, bound in (
            ("runner", {"path": RUNNER_PATH, "raw_sha256": sha256(repository.raw(RUNNER_PATH))}),
            ("bundle", binding(BUNDLE_PATH, repository.raw(BUNDLE_PATH))),
        ):
            if self.packet[field] != bound:
                raise RunError(f"repository {field} differs from the one the packet binds")
        self.plan = read("replay-plan.json")
        self.schedule = read("schedule.json")
        self.tool = read("tool-manifest.json")
        self.dependency = read("dependency-manifest.json")
        self.environment = read("environment-manifest.json")
        self.adapter_raw = (archive / STAGED_ADAPTER).read_bytes()
        self.suite = Suite(repository, self.adapter_raw)
        self.contract = self.suite.adapter.adapter_contract()
        core = {key: value for key, value in self.packet.items() if key != "epoch"}
        if self.packet["epoch"] != f"d004-e-{digest(core)[:20]}":
            raise RunError("epoch identifier is not derived from the packet")
        for field, value in (("tool_manifest_sha256", self.tool), ("dependency_manifest_sha256", self.dependency), ("environment_sha256", self.environment)):
            if self.packet[field] != digest(value):
                raise RunError(f"packet {field} does not match the archived manifest")
        if self.packet["candidate_models"] != [
            {"candidate": candidate, **{field: self.suite.models[candidate][field] for field in MODEL_DIGEST_FIELDS}} for candidate in CANDIDATES
        ]:
            raise RunError("candidate models differ from the packet")
        if self.packet["input_manifests"] != [{"case": case, "input_manifest_sha256": digest(self.suite.manifests[case])} for case in CASES]:
            raise RunError("input manifests differ from the packet")
        if self.tool["adapter"]["raw_sha256"] != sha256(self.adapter_raw) or self.suite.bindings != canonical_document(repository.raw(BUNDLE_PATH))["inputs"]:
            raise RunError("archived adapter or inputs differ from the bundle")
        overlay_raw = repository.raw(OVERLAY_PATH)
        if self.packet["overlay"] != binding(OVERLAY_PATH, overlay_raw) or self.packet["selection"]["distinguishing_rule"] != canonical_document(overlay_raw)["distinguishing_rule"]:
            raise RunError("the packet does not bind the overlay's distinguishing rule")
        if self.plan != epoch_replay_plan(self.packet, self.suite):
            raise RunError("epoch replay plan is not derived from the packet")
        if self.schedule != schedule_identities(self.packet, self.plan, self.suite, self.tool, self.dependency, self.environment):
            raise RunError("scheduled identities are not derived from the epoch")
        self.packet_sha256 = digest(self.packet)
        self.plan_sha256 = digest(self.plan)

    def request_raw(self, row: dict[str, Any]) -> bytes:
        return canonical_file(request_document(self.suite, self.packet, self.plan, row))

    def shared_files(self, row: dict[str, Any]) -> list[str]:
        return [
            "epoch/packet.json", "epoch/replay-plan.json", "epoch/schedule.json", "epoch/tool-manifest.json",
            "epoch/dependency-manifest.json", "epoch/environment-manifest.json",
            f"epoch/models/{row['candidate']}.json", f"epoch/inputs/{row['case']}.json",
            f"epoch/requests/slot-{row['logical_slot_ordinal']:02d}.json", STAGED_ADAPTER,
        ]


def _entry(path: str, data: bytes) -> dict[str, Any]:
    return {"path": path, "mode": ARCHIVE_FILE_MODE, "byte_length": len(data), "raw_sha256": sha256(data)}


def build_record(ctx: EpochContext, row: dict[str, Any], state: dict[str, Any], measured: dict[str, int], stdout: bytes, stderr: bytes) -> dict[str, Any]:
    """Derive the closed case record of one execution from its archived outputs."""
    candidate, case = row["candidate"], row["case"]
    rows = ctx.suite.rows[case]
    model = ctx.suite.models[candidate]
    request_raw = ctx.request_raw(row)
    response = None
    if state["kind"] == "completed":
        try:
            response = validate_response(ctx.contract, stdout, stderr, request_raw, rows, model)
        except PayloadError:
            state = {**state, "kind": "unsupported_behavior", "adapter_status": "failed"}
    observations = []
    if response is not None:
        for observation, item in zip(response["observations"], rows):
            oracle = item["oracle"]
            inability = observation["normalized_observation"]["decision"]["category"] in INABILITY
            matched = observation["observed_state"] in oracle["allowed_domain_states"] and not (
                observation["observed_state"] == "unsupported" and inability
            )
            observations.append(
                {
                    "id": observation["id"],
                    "subject_id": oracle["subject_id"],
                    "subject_sha256": oracle["subject_sha256"],
                    "observation_level": oracle["observation_level"],
                    "allowed_domain_states": oracle["allowed_domain_states"],
                    "observed_state": observation["observed_state"],
                    "comparison": "matched" if matched else "mismatched",
                    "required_invalidation": oracle["required_invalidation"],
                    "observed_invalidation": observation["observed_invalidation"],
                    "capability_credit": oracle["capability_credit"],
                    "normalized_observation_sha256": observation["normalized_observation_sha256"],
                    "raw_log_refs": ["adapter"],
                }
            )
    required = set(ctx.suite.manifests[case]["required_relationships"])
    conformance = {item["relationship"]: item for item in response["sr_conformance"]} if response else {}
    sr_rows = []
    for relationship in RELATIONSHIPS:
        requirement = ctx.suite.required[relationship]
        reported = conformance.get(relationship)
        sr_rows.append(
            {
                "relationship": relationship,
                "native_edges": [item for item in model["model"]["crossings"] if item["relationship"] == relationship][0]["native_edges"],
                **{field: requirement[field] for field in SR_ROW_FIELDS[2:12]},
                "applicability": "required" if relationship in required else "not_required",
                "conformance_state": reported["conformance_state"] if reported else "unresolved",
                "dependent_observation_ids": reported["dependent_observation_ids"] if reported else [],
            }
        )
    graph = model["graph"]
    candidate_graph = {
        "graph_sha256": model["graph_sha256"],
        "nodes": [{"id": node["id"], "role": node["role"], "semantic_subject_sha256": digest(node)} for node in graph["nodes"]],
        "edges": [
            {
                "id": edge["edge_subject"]["id"],
                "relationship": edge["edge_subject"]["relationship"],
                "direction": edge["edge_subject"]["direction"],
                "domain": f"{edge['edge_subject']['domain_endpoint']['node']}.{edge['edge_subject']['domain_endpoint']['facet']}",
                "codomain": [f"{item['node']}.{item['facet']}" for item in edge["edge_subject"]["codomain_endpoints"]],
                **{field: edge["edge_subject"][field] for field in ("definedness", "obligations", "identity_inputs", "trust_role", "failure_behavior", "prohibited_reverse_inferences")},
                "observation": edge["edge_subject"]["observation_requirement"],
                "edge_sha256": edge["edge_sha256"],
            }
            for edge in graph["edges"]
        ],
    }
    log_entries = [{"id": "adapter", "stdout_sha256": sha256(stdout), "stderr_sha256": sha256(stderr), "stdout_bytes": len(stdout), "stderr_bytes": len(stderr)}]
    byte_entries = [_entry(path, (ctx.archive / path).read_bytes()) for path in ctx.shared_files(row)]
    byte_entries += [_entry("execution/stderr", stderr), _entry("execution/stdout", stdout)]
    byte_entries.sort(key=lambda item: item["path"])
    output_manifest = {"entries": [{"id": "stderr", "raw_sha256": sha256(stderr), "byte_length": len(stderr)}, {"id": "stdout", "raw_sha256": sha256(stdout), "byte_length": len(stdout)}]}
    argv = ctx.environment["argv"]
    environment = ctx.environment["entries"]
    completed = state["kind"] == "completed" and state["exit_code"] == 0 and state["signal"] is None and state["adapter_status"] == "executed" and not (state["stdout_truncated"] or state["stderr_truncated"])
    joins = [ctx.request_raw(row) == (ctx.archive / f"epoch/requests/slot-{row['logical_slot_ordinal']:02d}.json").read_bytes()]
    joins.append(canonical_document((ctx.archive / f"epoch/models/{candidate}.json").read_bytes()) == model)
    joins.append(canonical_document((ctx.archive / f"epoch/inputs/{case}.json").read_bytes()) == ctx.suite.manifests[case])
    conditions = {
        "execution_completed": completed,
        "resource_bounds_hold": resource_bounds_hold(measured),
        "inventory_complete": [item["subject_id"] for item in observations] == [item["oracle"]["subject_id"] for item in rows],
        "observations_matched": bool(observations) and all(item["comparison"] == "matched" for item in observations),
        "invalidations_satisfied": bool(observations) and all(item["observed_invalidation"] == "satisfied" for item in observations if item["required_invalidation"] == "dependent_result"),
        "required_relationships_satisfied": all(item["conformance_state"] == "satisfied" and item["dependent_observation_ids"] and set(item["dependent_observation_ids"]) <= {entry["id"] for entry in observations} for item in sr_rows if item["applicability"] == "required"),
        "digest_joins": all(joins),
        "replay_contract_holds": replay_contract_holds(ctx),
    }
    record = {
        "schema_version": RECORD_SCHEMA,
        "suite_version": SUITE_VERSION,
        "epoch": ctx.packet["epoch"],
        "packet_sha256": ctx.packet_sha256,
        "replay_plan_sha256": ctx.plan_sha256,
        "execution_ordinal": row["execution_ordinal"],
        "logical_slot_ordinal": row["logical_slot_ordinal"],
        "round": row["round"],
        "position": row["position"],
        "candidate": candidate,
        "case": case,
        "repetition": row["repetition"],
        "positive_subject": {"subject_id": rows[0]["oracle"]["subject_id"], "subject_sha256": rows[0]["oracle"]["subject_sha256"]},
        "mutation_subjects": [
            {"mutation_id": item["oracle"]["mutation_id"], "subject_id": item["oracle"]["subject_id"], "subject_sha256": item["oracle"]["subject_sha256"]}
            for item in rows
            if item["oracle"]["mutation_id"] is not None
        ],
        "identities": {
            "scheduled_slot_sha256": row["scheduled_slot_sha256"],
            "scheduled_execution_sha256": row["scheduled_execution_sha256"],
            "input_manifest_sha256": digest(ctx.suite.manifests[case]),
            "model_sha256": model["model_sha256"],
            "tool_sha256": digest(ctx.tool),
            "dependency_manifest_sha256": digest(ctx.dependency),
            "environment_sha256": digest(ctx.environment),
            "candidate_graph_sha256": model["graph_sha256"],
            "sr_map_sha256": model["sr_map_sha256"],
            "semantic_endpoint_sha256": model["semantic_endpoint_sha256"],
            "parameter_model_sha256": model["parameter_model_sha256"],
            "positive_subject_sha256": rows[0]["oracle"]["subject_sha256"],
        },
        "argv": argv,
        "environment": environment,
        "resource_ceilings": CEILINGS,
        "measured_resources": measured,
        "execution_state": state,
        "observations": observations,
        "log_manifest": {"manifest_sha256": digest(log_entries), "entries": log_entries},
        **{name: response[name] if response else [] for name in CONTEXT_FIELDS},
        "unsupported_features": response["unsupported_features"] if response else [],
        "candidate_graph": candidate_graph,
        "sr_conformance_map": sr_rows,
        "case_verdict": "pass" if all(conditions.values()) else "fail",
        "verdict_conditions": conditions,
        "byte_manifest": {"manifest_sha256": digest(byte_entries), "entries": byte_entries},
        "replay": {
            "argv": argv,
            "environment_sha256": digest(ctx.environment),
            "network": NETWORK,
            "cache": CACHE,
            "input_manifest_sha256": digest(ctx.suite.manifests[case]),
            "expected_output_manifest_sha256": digest(output_manifest),
        },
        "measures": positive_measures(response, observations, case) if case in V08_CASES else None,
        "owner_labels": OWNER_LABELS,
    }
    assert tuple(record) == RECORD_FIELDS
    return record


def positive_measures(response: dict[str, Any] | None, observations: list[dict[str, Any]], case: str) -> dict[str, int] | None:
    """The measures an SC-06 or SC-07 positive observation reports, when it matched and names exactly its own case's measures."""
    if response is None or not observations or observations[0]["comparison"] != "matched":
        return None
    values = response["observations"][0]["normalized_observation"]["values"]
    measures = values.get("measures") if isinstance(values, dict) else None
    names = {name for name, measured_in, _ in MEASURES if measured_in == case}
    if not isinstance(measures, dict) or set(measures) != names or not all(type(value) is int and value >= 0 for value in measures.values()):
        return None
    return measures


def rule_result(rule_id: str, measures: dict[str, dict[str, int]], unmeasured: list[str]) -> dict[str, Any]:
    """Apply the rule; a complete candidate without a full measure set makes the result inconclusive."""
    result = apply_rule(rule_id, measures)
    if unmeasured:
        result.update(remaining=[], result="inconclusive")
    return {**result, "unmeasured": unmeasured}


def replay_contract_holds(ctx: EpochContext) -> bool:
    environment = ctx.environment
    return (
        environment["argv"] == [ctx.tool["interpreter"]["realpath"], *ADAPTER_FLAGS, *ADAPTER_ARGS]
        and environment["entries"] == [{"name": name, "value": value} for name, value in ENVIRONMENT]
        and environment["network"] == NETWORK
        and environment["cache"] == CACHE
        and environment["isolation_probe"] == EXPECTED_ISOLATION
    )


def resource_bounds_hold(measured: dict[str, int]) -> bool:
    return (
        measured["wall_milliseconds"] <= CEILINGS["wall_seconds"] * 1000
        and measured["peak_memory_bytes"] <= CEILINGS["peak_memory_bytes"]
        and measured["temp_storage_bytes"] <= CEILINGS["temp_storage_bytes"]
        and measured["stdout_bytes"] + measured["stderr_bytes"] <= CEILINGS["output_bytes"]
    )


def parse_record(ctx: EpochContext, row: dict[str, Any], record: Any, stdout: bytes, stderr: bytes) -> list[str]:
    """Return every way an archived record differs from its re-derivation."""
    if not isinstance(record, dict) or set(record) != set(RECORD_FIELDS):
        return ["record is not the closed field set"]
    state, measured = record["execution_state"], record["measured_resources"]
    errors = []
    if not isinstance(state, dict) or set(state) != set(EXECUTION_STATE_FIELDS) or state.get("kind") not in EXECUTION_KINDS:
        return ["execution_state is not closed"]
    if not isinstance(measured, dict) or set(measured) != set(MEASURED_FIELDS) or not all(isinstance(value, int) and value >= 0 for value in measured.values()):
        return ["measured_resources is not closed"]
    if measured["stdout_bytes"] != len(stdout) or measured["stderr_bytes"] != len(stderr):
        errors.append("measured output sizes disagree with the archived logs")
    if measured["peak_memory_bytes"] <= 0 or measured["wall_milliseconds"] <= 0:
        errors.append("resource measurement is missing")
    raw_state = dict(state)
    if raw_state["kind"] == "unsupported_behavior" and raw_state["exit_code"] == 0 and raw_state["signal"] is None:
        raw_state.update(kind="completed", adapter_status="executed")
    rebuilt = build_record(ctx, row, raw_state, measured, stdout, stderr)
    for field in RECORD_FIELDS:
        if record.get(field) != rebuilt[field]:
            errors.append(f"{field} differs from its re-derivation")
    return errors


# ---------------------------------------------------------------------------
# Repetition closure, correction records, archive manifest


def deterministic_projection(record: dict[str, Any]) -> dict[str, Any]:
    projected = {field: value for field, value in record.items() if field not in REPETITION_VARIANT}
    projected["identities"] = {key: value for key, value in record["identities"].items() if key != "scheduled_execution_sha256"}
    projected["measured_resources"] = {key: record["measured_resources"][key] for key in ("stdout_bytes", "stderr_bytes")}
    return projected


def repetition_closure(records: list[dict[str, Any]]) -> dict[str, Any]:
    """Python mirror of ``validate_repetition_closure`` in reviewed_result_contract.rs."""

    def result(error: str | None, path: str | None) -> dict[str, Any]:
        return {"closure": "failed" if error else "closed", "error": error, "path": path}

    if len(records) != REPETITIONS:
        return result("Cardinality", "$/repetitions")
    first = records[0]
    schedule = {item["logical_slot_ordinal"]: item for item in rotation_schedule()}
    if first["logical_slot_ordinal"] not in schedule:
        return result("CoordinateMismatch", "$/repetitions/0/logical_slot_ordinal")
    slot = schedule[first["logical_slot_ordinal"]]
    if (first["candidate"], first["case"]) != (slot["candidate"], slot["case"]):
        return result("CoordinateMismatch", "$/repetitions/0/scheduled_coordinate")
    anchor = digest(deterministic_projection(first))
    for index, record in enumerate(records):
        path = f"$/repetitions/{index}"
        measured = record["measured_resources"]
        if record["repetition"] != index + 1:
            return result("RepetitionOrder", f"{path}/repetition")
        if (record["logical_slot_ordinal"], record["candidate"], record["case"]) != (first["logical_slot_ordinal"], first["candidate"], first["case"]):
            return result("CoordinateMismatch", path)
        if record["case_verdict"] != "pass":
            return result("IndependentFailure", f"{path}/independently_passed")
        if digest(deterministic_projection(record)) != anchor:
            return result("DeterministicMismatch", path)
        if not resource_bounds_hold(measured):
            return result("ResourceLimit", f"{path}/measured_resources")
    return result(None, None)


CORRECTION_FIELDS = ("schema_version", "record_id", "candidate", "epoch", "first_packet_record_sha256s", "changed_paths", "reason")


def parse_corrections(documents: list[Any], bundle: dict[str, Any], epoch: str, records: dict[str, list[str]]) -> list[str]:
    """Check correction records against the reviewed policy; ``records`` maps candidate to record digests."""
    errors, seen = [], set()
    local = bundle["correction"]["candidate_local_paths"]
    for index, document in enumerate(documents):
        where = f"correction {index}"
        if not isinstance(document, dict) or set(document) != set(CORRECTION_FIELDS):
            errors.append(f"{where} is not the closed correction record")
            continue
        candidate = document["candidate"]
        if candidate not in CANDIDATES or document["record_id"] != f"D004-COR-{candidate}-01":
            errors.append(f"{where} identifier does not follow D004-COR-<candidate>-01")
        if candidate in seen:
            errors.append(f"{where} is a second correction window for {candidate}")
        seen.add(candidate)
        if document["schema_version"] != "d004-correction-record-v0.1" or document["epoch"] != epoch:
            errors.append(f"{where} does not bind this epoch")
        prior = document["first_packet_record_sha256s"]
        if sorted(prior) != sorted(records.get(candidate, [])) or len(prior) != len(CASES) * REPETITIONS:
            errors.append(f"{where} does not retain the complete first candidate packet")
        changed = document["changed_paths"]
        if not changed:
            errors.append(f"{where} changes nothing")
        for item in changed:
            if not isinstance(item, dict) or item.get("path") not in local.get(candidate, []):
                errors.append(f"{where} changes a shared path; a shared change starts a new epoch for all candidates")
    return errors


def archive_manifest(archive: Path) -> dict[str, Any]:
    entries = []
    for path in sorted(archive.rglob("*")):
        relative = path.relative_to(archive).as_posix()
        if relative == "manifest.json" or path.is_dir():
            continue
        if path.is_symlink() or not path.is_file():
            raise RunError(f"archive entry {relative} is not a regular file")
        entries.append(_entry(relative, path.read_bytes()))
    return {"schema_version": "d004-archive-manifest-v0.1", "entries": entries}


def parse_archive_manifest(archive: Path) -> list[str]:
    try:
        manifest = canonical_document((archive / "manifest.json").read_bytes())
    except (OSError, UnicodeError, ValueError) as exc:
        return [f"manifest.json is unreadable: {exc}"]
    errors = []
    entries = manifest.get("entries") if isinstance(manifest, dict) else None
    if not isinstance(entries, list) or set(manifest) != {"schema_version", "entries"}:
        return ["manifest is not the closed archive manifest"]
    paths = [item.get("path") if isinstance(item, dict) else None for item in entries]
    if paths != sorted(set(paths), key=str):
        errors.append("manifest entries are not path-ascending and unique")
    for item in entries:
        if not isinstance(item, dict) or set(item) != {"path", "mode", "byte_length", "raw_sha256"}:
            errors.append("manifest entry is not closed")
            continue
        relative = PurePosixPath(item["path"])
        if relative.is_absolute() or ".." in relative.parts or item["path"] != relative.as_posix() or item["mode"] != ARCHIVE_FILE_MODE:
            errors.append(f"{item['path']} escapes the archive or has the wrong mode")
            continue
        target = archive / item["path"]
        if target.is_symlink() or not target.is_file():
            errors.append(f"{item['path']} is missing")
            continue
        data = target.read_bytes()
        if len(data) != item["byte_length"] or sha256(data) != item["raw_sha256"]:
            errors.append(f"{item['path']} bytes changed")
    try:
        observed = {item["path"] for item in archive_manifest(archive)["entries"]}
    except RunError as exc:
        return [*errors, str(exc)]
    extra = sorted(observed - set(filter(None, paths)))
    if extra:
        errors.append(f"unlisted archive files: {extra}")
    return errors


def summarize(ctx: EpochContext, records: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], dict[str, Any]]:
    closures, slots = [], []
    for slot in range(1, SLOTS + 1):
        group = sorted((item for item in records if item["logical_slot_ordinal"] == slot), key=lambda item: item["repetition"])
        closure = repetition_closure(group)
        closures.append({"logical_slot_ordinal": slot, **closure})
        first = group[0] if group else None
        slots.append(
            {
                "logical_slot_ordinal": slot,
                "candidate": first["candidate"] if first else None,
                "case": first["case"] if first else None,
                "verdicts": [item["case_verdict"] for item in group],
                "closure": closure["closure"],
                "closure_error": closure["error"],
                "repetitions_identical": len({digest(deterministic_projection(item)) for item in group}) == 1 if group else False,
                "failed_conditions": sorted({name for item in group for name, value in item["verdict_conditions"].items() if not value}),
                "mismatched_observations": [
                    {"id": item["id"], "observed_state": item["observed_state"], "allowed_domain_states": item["allowed_domain_states"]}
                    for item in (first["observations"] if first else [])
                    if item["comparison"] == "mismatched"
                ],
                "unsatisfied_required_relationships": [
                    item["relationship"] for item in (first["sr_conformance_map"] if first else [])
                    if item["applicability"] == "required" and item["conformance_state"] != "satisfied"
                ],
            }
        )
    closed = {(item["candidate"], item["case"]) for item in slots if item["closure"] == "closed"}
    complete = [candidate for candidate in CANDIDATES if all((candidate, case) in closed for case in CASES)]
    measures: dict[str, dict[str, int]] = {}
    unmeasured: list[str] = []
    for candidate in complete:
        merged: dict[str, int] = {}
        for item in records:
            if item["candidate"] == candidate and item["case"] in V08_CASES and item["repetition"] == 1 and item["measures"]:
                merged.update(item["measures"])
        if set(merged) == {name for name, _, _ in MEASURES}:
            measures[candidate] = dict(sorted(merged.items()))
        else:
            unmeasured.append(candidate)
    rule = ctx.packet["selection"]["distinguishing_rule"]
    summary = {
        "schema_version": "d004-run-summary-v0.3",
        "epoch": ctx.packet["epoch"],
        "packet_sha256": ctx.packet_sha256,
        "execution": {
            "required_execution_records": SLOTS * REPETITIONS,
            "result_record_count": len(records),
            "required_candidate_cases": SLOTS,
            "completed_candidate_cases": len(closed),
            "complete_candidates": len(complete),
            "complete_cross_candidate_cases": sum(1 for case in CASES if all((candidate, case) in closed for candidate in CANDIDATES)),
        },
        "slots": slots,
        "measures": measures,
        "distinguishing_rule_result": {
            **rule_result(rule["id"], measures, unmeasured),
            "scope": "candidates that close all seven cases; SS-G05 and the SS-G03 structure only",
            "standing": "not a D-004 recommendation under suite section 8 until the owner disposes every candidate and every hard gate",
        } if rule else None,
        "selection": None,
        "nonclaims": [
            "results are contributor-produced and unreviewed",
            "no semantic-strata candidate selected preferred or accepted",
            "no D-004 disposition accepted",
            "no S3b implementation authorized",
        ],
    }
    return closures, summary


# ---------------------------------------------------------------------------
# Commands


def _write(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    path.chmod(0o644)


def existing_archive(root: Path, epoch: str) -> Path:
    """Return the prepared archive named ``epoch`` under ``root``.

    The name is only compared with directory entries that already exist, so a
    command-line argument never becomes part of a filesystem path.
    """
    for child in sorted(root.iterdir()) if root.is_dir() else ():
        if child.name == epoch and child.is_dir() and not child.is_symlink():
            return child
    raise RunError(f"no prepared archive named {epoch!r}")


def command_prepare(repository: Repository, root: Path, source_revision: str) -> Path:
    catalog_raw, bundle_raw, overlay_raw = repository.raw(CATALOG_PATH), repository.raw(BUNDLE_PATH), repository.raw(OVERLAY_PATH)
    if (catalog_raw, bundle_raw, overlay_raw) != generated_documents(repository):
        raise RunError("committed catalog, bundle or overlay is stale; run generate")
    rule = canonical_document(overlay_raw)["distinguishing_rule"]
    if rule is None:
        raise RunError("the owner's distinguishing rule is not recorded; no v0.8 epoch may be prepared")
    suite = Suite(repository)
    work = Path(tempfile.mkdtemp(prefix="d004-host-"))
    try:
        host = Host(repository, work, CgroupMeter)
        probe = host.isolation_probe(work, suite.adapter_raw)
        tool = host.tool_manifest(suite.adapter_raw)
        dependency = host.dependency_manifest(work, suite.adapter_raw)
        environment = host.environment_manifest(probe)
    finally:
        shutil.rmtree(work)
    packet = epoch_packet(binding(BUNDLE_PATH, bundle_raw), binding(OVERLAY_PATH, overlay_raw), repository.raw(RUNNER_PATH), tool, dependency, environment, suite, rule)
    plan = epoch_replay_plan(packet, suite)
    schedule = schedule_identities(packet, plan, suite, tool, dependency, environment)
    archive = root / packet["epoch"]
    if archive.exists():
        raise RunError("archive directory already exists")
    epoch = archive / "epoch"
    for name, value in (("packet.json", packet), ("replay-plan.json", plan), ("schedule.json", schedule), ("tool-manifest.json", tool), ("dependency-manifest.json", dependency), ("environment-manifest.json", environment)):
        _write(epoch / name, canonical_file(value))
    for candidate in CANDIDATES:
        _write(epoch / "models" / f"{candidate}.json", canonical_file(suite.models[candidate]))
    for case in CASES:
        _write(epoch / "inputs" / f"{case}.json", canonical_file(suite.manifests[case]))
    for row in schedule[:SLOTS]:
        _write(epoch / "requests" / f"slot-{row['logical_slot_ordinal']:02d}.json", canonical_file(request_document(suite, packet, plan, row)))
    _write(archive / STAGED_ADAPTER, suite.adapter_raw)
    _write(archive / "provenance.json", canonical_file({"source_revision": source_revision, "note": "diagnostic only; the epoch identity is content-addressed"}))
    return archive


def host_context_changes(host: Any, ctx: Any, work: Path) -> list[str]:
    """Re-capture everything the packet binds about the host and name each part that changed since prepare."""
    probe = host.isolation_probe(work, ctx.adapter_raw)
    captured = (
        ("tool manifest", host.tool_manifest(ctx.adapter_raw), ctx.tool),
        ("dependency manifest", host.dependency_manifest(work, ctx.adapter_raw), ctx.dependency),
        ("environment manifest", host.environment_manifest(probe), ctx.environment),
    )
    return [name for name, current, prepared in captured if current != prepared]


def command_execute(repository: Repository, archive: Path) -> dict[str, Any]:
    ctx = EpochContext(archive, repository)
    if (archive / "executions").exists():
        raise RunError("executions already exist; an epoch runs once")
    work = Path(tempfile.mkdtemp(prefix="d004-host-"))
    records = []
    try:
        host = Host(repository, work, CgroupMeter)
        changed = host_context_changes(host, ctx, work)
        if changed:
            raise RunError(f"the host differs from the prepared epoch: {', '.join(changed)}")
        for row in ctx.schedule:
            request_raw = (archive / "epoch" / "requests" / f"slot-{row['logical_slot_ordinal']:02d}.json").read_bytes()
            wall_start = time.time()
            launched = host.run_staged(row["execution_ordinal"], work, ctx.adapter_raw, request_raw, host.adapter_argv())
            state = classify(launched)
            measured = {
                "wall_milliseconds": max(1, launched["wall_milliseconds"]),
                "peak_memory_bytes": launched["peak_memory_bytes"],
                "temp_storage_bytes": launched["temp_storage_bytes"],
                "stdout_bytes": len(launched["stdout"]),
                "stderr_bytes": len(launched["stderr"]),
            }
            directory = archive / "executions" / f"{row['execution_ordinal']:03d}"
            _write(directory / "stdout", launched["stdout"])
            _write(directory / "stderr", launched["stderr"])
            record = build_record(ctx, row, state, measured, launched["stdout"], launched["stderr"])
            _write(directory / "record.json", canonical_file(record))
            _write(directory / "diagnostics.json", canonical_file({
                "started_unix_seconds": int(wall_start),
                "exit_status": launched["exit_status"],
                "process_peak": launched["process_peak"],
                "cleanup": launched["cleanup"],
                "meter": CgroupMeter.kind,
            }))
            records.append(record)
    finally:
        shutil.rmtree(work, ignore_errors=True)
    closures, summary = summarize(ctx, records)
    _write(archive / "closures.json", canonical_file(closures))
    _write(archive / "summary.json", canonical_file(summary))
    _write(archive / "manifest.json", canonical_file(archive_manifest(archive)))
    return summary


def command_verify(repository: Repository, archive: Path) -> list[str]:
    errors = parse_archive_manifest(archive)
    ctx = EpochContext(archive, repository)
    records = []
    for row in ctx.schedule:
        directory = archive / "executions" / f"{row['execution_ordinal']:03d}"
        try:
            record = canonical_document((directory / "record.json").read_bytes())
            stdout, stderr = (directory / "stdout").read_bytes(), (directory / "stderr").read_bytes()
        except (OSError, UnicodeError, ValueError) as exc:
            errors.append(f"execution {row['execution_ordinal']}: {exc}")
            continue
        errors += [f"execution {row['execution_ordinal']}: {item}" for item in parse_record(ctx, row, record, stdout, stderr)]
        records.append(record)
    closures, summary = summarize(ctx, records)
    for name, value in (("closures.json", closures), ("summary.json", summary)):
        try:
            if canonical_document((archive / name).read_bytes()) != value:
                errors.append(f"{name} differs from its re-derivation")
        except (OSError, UnicodeError, ValueError) as exc:
            errors.append(f"{name}: {exc}")
    return errors


def main(argv: list[str]) -> int:
    repository = Repository(Path(__file__).resolve().parents[1])
    command = argv[1] if len(argv) > 1 else ""
    try:
        if command == "generate" and len(argv) == 2:
            # The bundle binds the catalog bytes on disk, so the catalog lands first.
            catalog_raw = generated_documents_catalog(repository)
            _write(repository.root / CATALOG_PATH, catalog_raw)
            _, bundle_raw, overlay_raw = generated_documents(repository)
            _write(repository.root / BUNDLE_PATH, bundle_raw)
            _write(repository.root / OVERLAY_PATH, overlay_raw)
            return 0
        if command == "check" and len(argv) == 2:
            current = (repository.raw(CATALOG_PATH), repository.raw(BUNDLE_PATH), repository.raw(OVERLAY_PATH))
            if current != generated_documents(repository):
                sys.stderr.write("D-004 v0.8 catalog, bundle or overlay is stale; run generate\n")
                return 1
            return 0
        if command == "prepare" and len(argv) == 4 and argv[2] == "--source-revision":
            archive = command_prepare(repository, ARCHIVE_ROOT, argv[3])
            sys.stdout.write(f"{archive.name}\t{archive}\n")
            return 0
        if command == "execute" and len(argv) == 3:
            summary = command_execute(repository, existing_archive(ARCHIVE_ROOT, argv[2]))
            sys.stdout.write(json.dumps(summary["execution"], sort_keys=True) + "\n")
            return 0
        if command == "verify" and len(argv) == 3:
            errors = command_verify(repository, existing_archive(ARCHIVE_ROOT, argv[2]))
            for error in errors:
                sys.stderr.write(error + "\n")
            return 1 if errors else 0
    except RunError as exc:
        sys.stderr.write(f"d004 run invalid: {exc}\n")
        return 2
    sys.stderr.write("usage: d004_v08_run.py generate | check | prepare --source-revision REV | execute EPOCH | verify EPOCH\n")
    return 64


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
