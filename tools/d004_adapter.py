"""Candidate-neutral D-004 semantic-strata adapter (research-only).

One engine serves all five candidates. A candidate differs only through its
reviewed graph from the v0.5 mapping catalog and the model, endpoint inventory,
and parameter bindings derived mechanically from that graph by
``derive_candidate_model``. The engine never receives an oracle row or a
declared expectation: it reads subject payloads, applies the candidate-neutral
``RULES`` and computations at the candidate's own crossings, and reports
observed domain states.

Run as ``python3 -I -S -B -X utf8 d004_adapter.py REQUEST`` inside the D-004
launcher. The request and the response are closed canonical JSON documents.
The module imports only the standard library and never touches the network.
"""

from __future__ import annotations

import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

REQUEST_SCHEMA = "d004-adapter-request-v0.1"
RESPONSE_SCHEMA = "d004-adapter-response-v0.1"
MODEL_SCHEMA = "d004-candidate-semantic-model-v0.1"
SUITE_VERSION = "d004-v0.5-draft"
CANDIDATES = ("ST-REL", "ST-UNI", "ST-DUAL", "ST-MIRROR", "ST-HOST")
CASES = ("SC-01", "SC-02", "SC-03", "SC-04", "SC-05")
RELATIONSHIPS = tuple(f"SR-{index:02d}" for index in range(1, 15))
DOMAIN_STATES = ("succeeded", "rejected", "unknown", "timeout", "unsupported", "exhausted")
SOURCE_CATALOGS = ("case_subject_catalog", "cross_cutting_fixture_catalog")
EXIT_CODES = {
    "completed": 0,
    "missing_input": 3,
    "digest_mismatch": 4,
    "unsupported_behavior": 5,
    "resource_exhaustion": 6,
}
MAX_REQUEST_BYTES = 4 * 1024 * 1024

# Parameter slots bound identically for every candidate. Slots whose meaning
# belongs to an open decision stay symbolic; a candidate that needs a concrete
# value for one of them cannot complete that crossing (suite section 1, SS-G06).
_SUITE = "suite_only"
_OPEN = "open_decision"
_DELEGATION = "delegation_policy"
SLOT_BINDINGS: dict[str, tuple[str, str | None, str | None]] = {
    "abi_model": (_OPEN, None, "D-013"),
    "bound_model": (_SUITE, "d004-suite-only-symbolic-bound-v0.1", None),
    "checker_policy": (_OPEN, None, "D-007"),
    "claim_model": (_OPEN, None, "D-005"),
    "ct_model": (_OPEN, None, "D-012"),
    "effect_model_identity": (_SUITE, "d004-suite-only-effect-model-v0.1", None),
    "elaborator_identity": (_SUITE, "d004-suite-only-elaborator-v0.1", None),
    "embedding_version": (_SUITE, "d004-suite-only-explicit-embedding-v0.1", None),
    "evidence_policy": (_OPEN, None, "D-005"),
    "game_delegation_selector": (_DELEGATION, "delegate_to_selected_host", None),
    "game_host_identity": (_OPEN, None, "D-006"),
    "game_non_success_policy": (
        _DELEGATION,
        "reject_or_unsupported_without_fallback_claim",
        None,
    ),
    "language_version": (_SUITE, "orange-edition-2026-s3a", None),
    "low_level_boundary_identity": (
        _SUITE,
        "d004-suite-only-low-level-boundary-v0.1",
        None,
    ),
    "lowering_version": (_SUITE, "d004-suite-only-lowering-v0.1", None),
    "machine_delegation_selector": (_DELEGATION, "delegate_to_selected_host", None),
    "machine_host_identity": (_OPEN, None, "D-011"),
    "machine_non_success_policy": (
        _DELEGATION,
        "reject_or_unsupported_without_fallback_claim",
        None,
    ),
    "probability_model": (_SUITE, "d004-suite-only-finite-probability-v0.1", None),
    "probability_model_identity": (
        _SUITE,
        "d004-suite-only-finite-probability-v0.1",
        None,
    ),
    "proof_calculus": (_OPEN, None, "D-006"),
    "proof_delegation_selector": (_DELEGATION, "delegate_to_selected_host", None),
    "proof_format": (_OPEN, None, "D-007"),
    "proof_host_identity": (_OPEN, None, "D-006"),
    "proof_non_success_policy": (
        _DELEGATION,
        "reject_or_unsupported_without_fallback_claim",
        None,
    ),
    "refinement_model": (_SUITE, "d004-suite-only-named-refinement-v0.1", None),
    "relation_version": (_SUITE, "d004-suite-only-relation-v0.1", None),
    "shared_pure_version": (_SUITE, "d004-suite-only-shared-pure-v0.1", None),
    "target_model": (_OPEN, None, "D-011"),
}

# Candidate-neutral judgments. Each rule is judged at the codomain member of the
# candidate's own native edge for ``relationship``. ``violation`` names the
# domain state a non-admissible value produces; ``policy_hook`` yields
# ``unknown`` while the subject's leakage policy hook is unselected.
RULES: tuple[dict[str, Any], ...] = tuple(
    {
        "id": rule_id,
        "case": case,
        "relationship": relationship,
        "fact": fact,
        "admissible": [admissible],
        "violation": violation,
        "source": source,
    }
    for rule_id, case, relationship, fact, admissible, violation, source in (
        ("R-SC01-INT-WORD", "SC-01", "SR-01", "integer_to_word_conversion_mode", "explicit_only", "rejected", "suite 5 SC-01 mutation and negative case"),
        ("R-SC01-ENDIAN", "SC-01", "SR-01", "byte_order_conversion_mode", "explicit_only", "rejected", "suite 5 SC-01 mutation and negative case"),
        ("R-SC01-WIDTH", "SC-01", "SR-01", "word_width", "declared_width", "rejected", "suite 5 SC-01 mutation and negative case"),
        ("R-SC01-SHIFT", "SC-01", "SR-01", "shift_bound", "bounded_to_word_width", "rejected", "suite 5 SC-01 mutation and negative case"),
        ("R-SC02-ALIAS", "SC-02", "SR-02", "mutable_aliasing", "exclusive_mutable_access", "rejected", "suite 5 SC-02 mutation and negative case"),
        ("R-SC02-RANGE", "SC-02", "SR-02", "memory_access_range", "within_declared_range", "rejected", "suite 5 SC-02 mutation and negative case"),
        ("R-SC02-INVARIANT", "SC-02", "SR-02", "loop_invariant", "required_present", "rejected", "suite 4 SR-02 definedness; suite 5 SC-02"),
        ("R-SC02-INIT", "SC-02", "SR-02", "read_initialization", "initialized", "rejected", "suite 5 SC-02 mutation and negative case"),
        ("R-SC02-REFINE", "SC-02", "SR-09", "refinement_subject_identity", "original", "rejected", "suite 4 SR-09; suite 5 SC-02"),
        ("R-SC03-BRANCH", "SC-03", "SR-10", "branch_condition", "public", "policy_hook", "suite 5 SC-03 mutation and negative case"),
        ("R-SC03-ADDRESS", "SC-03", "SR-10", "memory_address", "public", "policy_hook", "suite 5 SC-03 mutation and negative case"),
        ("R-SC03-LOOP", "SC-03", "SR-10", "loop_bound", "public", "policy_hook", "suite 5 SC-03 mutation and negative case"),
        ("R-SC03-FAILURE", "SC-03", "SR-10", "failure_path_selector", "public", "policy_hook", "suite 5 SC-03 mutation and negative case"),
        ("R-SC03-DEBUG", "SC-03", "SR-10", "debug_observation", "public_independent", "policy_hook", "suite 5 SC-03 mutation and negative case"),
        ("R-SC04-FEATURE", "SC-04", "SR-03", "required_target_feature", "declared_present", "unsupported", "suite 4 SR-03; suite 5 SC-04"),
        ("R-SC04-INTRINSIC", "SC-04", "SR-03", "intrinsic_identity", "declared_supported_intrinsic", "unsupported", "suite 4 SR-03; suite 5 SC-04"),
        ("R-SC04-LANE", "SC-04", "SR-03", "lane_order", "declared_order", "rejected", "suite 5 SC-04 mutation and negative case"),
        ("R-SC04-WIDTH", "SC-04", "SR-03", "vector_width", "declared_width", "rejected", "suite 5 SC-04 mutation and negative case"),
        ("R-SC04-TARGET", "SC-04", "SR-11", "target_model_identity", "original", "rejected", "suite 4 SR-11; suite 5 SC-04"),
        ("R-SC04-FALLBACK", "SC-04", "SR-11", "fallback_authorization", "declared", "rejected", "suite 4 SR-11; suite 5 SC-04"),
        ("R-SC05-RANDOM", "SC-05", "SR-04", "randomness_source", "explicit_sampling", "rejected", "suite 4 SR-04; suite 5 SC-05"),
        ("R-SC05-ORACLE", "SC-05", "SR-04", "oracle_effect_visibility", "explicit", "rejected", "suite 4 SR-04; suite 5 SC-05"),
        ("R-SC05-BOUNDED", "SC-05", "SR-04", "sample_space_bound", "finite", "rejected", "suite 4 SR-04; suite 5 SC-05"),
        ("R-SC05-AUTHORITY", "SC-05", "SR-08", "sampling_authority", "game_semantics", "rejected", "suite 4 SR-06 and SR-08 invariants"),
        ("R-SC05-ENDPOINTS", "SC-05", "SR-12", "reduction_endpoint_identity", "original", "rejected", "suite 4 SR-12; suite 5 SC-05"),
        ("R-SC05-BOUND", "SC-05", "SR-12", "symbolic_advantage_bound", "original", "rejected", "suite 4 SR-12; suite 5 SC-05"),
    )
)

COMPUTATIONS: tuple[dict[str, str], ...] = tuple(
    {"id": computation_id, "case": case, "relationship": relationship, "description": text}
    for computation_id, case, relationship, text in (
        ("C-SC01-EVAL", "SC-01", "SR-01", "evaluate the round-like word function on every boundary vector and require the declared normalized result"),
        ("C-SC02-PURE", "SC-02", "SR-01", "evaluate the pure byte map and require the declared output bytes"),
        ("C-SC02-IMPL", "SC-02", "SR-02", "execute the owned-region loop with bounds and initialization checks"),
        ("C-SC02-REFINE", "SC-02", "SR-09", "record the explicit refinement pair as an open obligation and require concrete agreement"),
        ("C-SC03-CHANNELS", "SC-03", "SR-02", "require every declared observation channel to carry a control label"),
        ("C-SC03-PRESERVE", "SC-03", "SR-10", "keep every observation channel visible at the unselected policy boundary"),
        ("C-SC04-PURE", "SC-04", "SR-01", "evaluate lane-wise addition modulo 2^32 and require the declared lanes"),
        ("C-SC04-LOWER", "SC-04", "SR-11", "record the exact target-parameterized lowering obligation and the separate fallback path"),
        ("C-SC05-GAME", "SC-05", "SR-04", "bind games to explicit oracles and the adversary interface"),
        ("C-SC05-EMBED", "SC-05", "SR-08", "require an explicit sampling-free Shared Pure embedding"),
        ("C-SC05-REDUCE", "SC-05", "SR-12", "record the exact reduction endpoints and symbolic bound as open"),
    )
)

_BOUNDARY_AUTHORITIES = ("later_decision_boundary", "unresolved_checker_boundary")
_SUPPORTED_INTRINSICS = {"vector_add_u32x4": "lane_wise_add_mod_2^32"}
_MAX_SAMPLE_SPACE = 1 << 16


class AdapterFailure(Exception):
    def __init__(self, kind: str, detail: str) -> None:
        super().__init__(detail)
        self.kind = kind


def canonical_bytes(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True).encode("ascii")


def digest(value: Any) -> str:
    return hashlib.sha256(canonical_bytes(value)).hexdigest()


def _reject_constant(token: str) -> Any:
    raise ValueError(f"non-finite JSON number {token}")


def _reject_float(token: str) -> Any:
    raise ValueError(f"floating-point JSON number {token}")


def _unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key}")
        result[key] = value
    return result


def strict_loads(data: bytes) -> Any:
    return json.loads(
        data.decode("utf-8"),
        object_pairs_hook=_unique_object,
        parse_float=_reject_float,
        parse_constant=_reject_constant,
    )


def binding_table() -> list[dict[str, Any]]:
    return [
        {"slot": slot, "binding": kind, "value": value, "decision": decision}
        for slot, (kind, value, decision) in sorted(SLOT_BINDINGS.items())
    ]


def rule_table() -> list[dict[str, Any]]:
    return [dict(rule) for rule in RULES]


def computation_table() -> list[dict[str, str]]:
    return [dict(computation) for computation in COMPUTATIONS]


def _root(nodes: dict[str, dict[str, Any]], node_id: str) -> str:
    seen = set()
    while nodes[node_id]["parent"] is not None:
        if node_id in seen:
            raise AdapterFailure("unsupported_behavior", "cyclic member parent chain")
        seen.add(node_id)
        node_id = nodes[node_id]["parent"]
    return node_id


def _endpoint(value: dict[str, str]) -> str:
    return f"{value['node']}.{value['facet']}"


def derive_candidate_model(entry: dict[str, Any]) -> dict[str, Any]:
    """Derive the candidate model, endpoints, and bindings from one catalog entry."""
    graph = entry["graph"]
    candidate = entry["candidate"]
    nodes = {node["id"]: node for node in graph["nodes"]}
    edges = {edge["edge_subject"]["id"]: edge["edge_subject"] for edge in graph["edges"]}
    rows = {row["mapping"]["relationship"]: row["mapping"] for row in graph["sr_rows"]}
    members = [
        {
            "id": node_id,
            "member_kind": node["member_kind"],
            "authority": node["authority"],
            "facets": list(node["facets"]),
            "parent": node["parent"],
            "authority_root": _root(nodes, node_id),
        }
        for node_id, node in sorted(nodes.items())
    ]
    crossings = []
    for relationship in RELATIONSHIPS:
        row = rows[relationship]
        native = [edges[edge_id] for edge_id in row["native_edges"]]
        codomains = [item for edge in native for item in edge["codomain_endpoints"]]
        delegations = [edge["delegation_boundary"] for edge in native if edge["delegation_boundary"]]
        if delegations:
            execution = "delegated"
        elif any(nodes[item["node"]]["authority"] in _BOUNDARY_AUTHORITIES for item in codomains):
            execution = "boundary"
        else:
            execution = "local"
        owner = codomains[0]["node"]
        crossings.append(
            {
                "relationship": relationship,
                "mapping_form": row["mapping_form"],
                "fused_authority": row["fused_authority"],
                "inspectability": row["inspectability"],
                "native_edges": list(row["native_edges"]),
                "domain_endpoints": [_endpoint(edge["domain_endpoint"]) for edge in native],
                "codomain_endpoints": [_endpoint(item) for item in codomains],
                "execution": execution,
                "owning_member": owner,
                "authority_root": _root(nodes, owner),
                "delegation": delegations[0] if delegations else None,
            }
        )
    by_relationship = {crossing["relationship"]: crossing for crossing in crossings}
    model = {
        "schema_version": MODEL_SCHEMA,
        "candidate": candidate,
        "graph_sha256": entry["graph_sha256"],
        "members": members,
        "crossings": crossings,
        "judgment_placement": [
            {
                "rule": rule["id"],
                "relationship": rule["relationship"],
                "member": by_relationship[rule["relationship"]]["owning_member"],
            }
            for rule in RULES
        ],
        "rule_table_sha256": digest(rule_table()),
        "computation_table_sha256": digest(computation_table()),
    }
    endpoint_roles: dict[str, dict[str, Any]] = {}
    for edge_id, edge in sorted(edges.items()):
        for role, items in (("domain", [edge["domain_endpoint"]]), ("codomain", edge["codomain_endpoints"])):
            for item in items:
                key = _endpoint(item)
                slot = endpoint_roles.setdefault(
                    key,
                    {"endpoint": key, "node": item["node"], "facet": item["facet"], "roles": [], "relationships": []},
                )
                if role not in slot["roles"]:
                    slot["roles"].append(role)
                if edge["relationship"] not in slot["relationships"]:
                    slot["relationships"].append(edge["relationship"])
    endpoints = []
    for key in sorted(endpoint_roles):
        slot = endpoint_roles[key]
        slot["roles"].sort()
        slot["relationships"].sort()
        endpoints.append(slot)
    parameters = []
    for edge in sorted(edges.values(), key=lambda item: (item["relationship"], item["id"])):
        for slot in edge["parameter_slots"]:
            if slot not in SLOT_BINDINGS:
                raise AdapterFailure("unsupported_behavior", f"unbound parameter slot {slot}")
            kind, value, decision = SLOT_BINDINGS[slot]
            parameters.append(
                {
                    "edge": edge["id"],
                    "relationship": edge["relationship"],
                    "slot": slot,
                    "binding": kind,
                    "value": value,
                    "decision": decision,
                }
            )
    return {
        "candidate": candidate,
        "graph": graph,
        "graph_sha256": entry["graph_sha256"],
        "sr_map_sha256": digest(graph["sr_rows"]),
        "model": model,
        "model_sha256": digest(model),
        "semantic_endpoints": endpoints,
        "semantic_endpoint_sha256": digest(endpoints),
        "parameter_bindings": parameters,
        "parameter_model_sha256": digest(parameters),
    }


def adapter_contract() -> dict[str, Any]:
    """Closed request/response contract published in the D-004 v0.7 bundle."""
    return {
        "request_schema": REQUEST_SCHEMA,
        "response_schema": RESPONSE_SCHEMA,
        "model_schema": MODEL_SCHEMA,
        "suite_version": SUITE_VERSION,
        "request_fields": ["candidate_model", "execution", "schema_version", "subjects", "suite_version"],
        "request_execution_fields": sorted(_EXECUTION_FIELDS),
        "request_candidate_model_fields": sorted(_MODEL_FIELDS),
        "request_subject_fields": sorted(_SUBJECT_FIELDS),
        "response_fields": sorted(_RESPONSE_FIELDS),
        "response_observation_fields": sorted(_OBSERVATION_FIELDS),
        "normalized_observation_fields": sorted(_NORMALIZED_FIELDS),
        "response_sr_fields": sorted(_SR_FIELDS),
        "context_entry_fields": ["description", "id", "sha256", "status"],
        "unsupported_feature_fields": ["dependent_result", "description", "id", "state"],
        "domain_states": list(DOMAIN_STATES),
        "observed_invalidation_states": ["not_required", "not_satisfied", "satisfied"],
        "sr_conformance_states": ["not_satisfied", "satisfied", "unsupported"],
        "exit_codes": dict(sorted(EXIT_CODES.items())),
        "other_exit_status": "crash",
        "declared_expectations_in_request": "forbidden",
        "network": "denied",
        "output": "canonical_json_with_terminal_line_feed_on_stdout; stderr empty on success",
        "slot_bindings": binding_table(),
        "rules": rule_table(),
        "computations": computation_table(),
    }


# Repetition-invariant coordinates only: the three repetitions of one logical
# slot receive byte-identical requests so their outputs can be compared.
_EXECUTION_FIELDS = {
    "epoch",
    "packet_sha256",
    "replay_plan_sha256",
    "scheduled_slot_sha256",
    "logical_slot_ordinal",
    "round",
    "position",
    "candidate",
    "case",
}
_MODEL_FIELDS = {
    "candidate",
    "graph",
    "graph_sha256",
    "sr_map_sha256",
    "model",
    "model_sha256",
    "semantic_endpoints",
    "semantic_endpoint_sha256",
    "parameter_bindings",
    "parameter_model_sha256",
}
_SUBJECT_FIELDS = {"subject_id", "subject_sha256", "source_catalog", "subject"}
_RESPONSE_FIELDS = {
    "schema_version",
    "request_sha256",
    "candidate",
    "case",
    "observations",
    "sr_conformance",
    "premises",
    "assumptions",
    "trusted_components",
    "unsupported_features",
}
_OBSERVATION_FIELDS = {
    "id",
    "subject_id",
    "subject_sha256",
    "observed_state",
    "observed_invalidation",
    "normalized_observation",
    "normalized_observation_sha256",
}
_NORMALIZED_FIELDS = {
    "subject_id",
    "subject_sha256",
    "observation_level",
    "layer",
    "observed_state",
    "observed_invalidation",
    "crossings",
    "decision",
    "values",
    "obligations",
    "open_parameters",
}
_SR_FIELDS = {"relationship", "native_edges", "conformance_state", "dependent_observation_ids"}
_HEX = re.compile(r"[0-9a-f]{64}")


def _require(condition: bool, detail: str, kind: str = "unsupported_behavior") -> None:
    if not condition:
        raise AdapterFailure(kind, detail)


def _closed(value: Any, fields: set[str], where: str) -> dict[str, Any]:
    _require(isinstance(value, dict) and set(value) == fields, f"{where} is not the closed object")
    return value


class Engine:
    def __init__(self, candidate_model: dict[str, Any], case: str) -> None:
        self.case = case
        self.graph = candidate_model["graph"]
        self.model = candidate_model["model"]
        self.nodes = {node["id"]: node for node in self.graph["nodes"]}
        self.edges = {edge["edge_subject"]["id"]: edge["edge_subject"] for edge in self.graph["edges"]}
        self.crossings = {crossing["relationship"]: crossing for crossing in self.model["crossings"]}
        self.bindings = {
            (item["edge"], item["slot"]): item for item in candidate_model["parameter_bindings"]
        }
        self.open_parameters: set[str] = set()

    # Structural resolution of the candidate's own native edges.
    def resolve(self, relationship: str) -> str | None:
        crossing = self.crossings.get(relationship)
        if crossing is None or not crossing["native_edges"]:
            return "missing_native_edge"
        for edge_id in crossing["native_edges"]:
            edge = self.edges.get(edge_id)
            if edge is None or edge["relationship"] != relationship:
                return "missing_native_edge"
            for item in [edge["domain_endpoint"], *edge["codomain_endpoints"]]:
                node = self.nodes.get(item["node"])
                if node is None or item["facet"] not in node["facets"]:
                    return "unresolved_endpoint"
            for slot in edge["parameter_slots"]:
                if (edge_id, slot) not in self.bindings:
                    return "unbound_parameter"
        return None

    def _crossing_record(self, relationship: str, state: str, category: str | None) -> dict[str, Any]:
        crossing = self.crossings.get(relationship, {})
        return {
            "relationship": relationship,
            "native_edges": list(crossing.get("native_edges", [])),
            "execution": crossing.get("execution", "unresolved"),
            "member": crossing.get("owning_member"),
            "authority_root": crossing.get("authority_root"),
            "state": state,
            "category": category,
        }

    def _note_open_parameters(self, relationship: str) -> list[str]:
        opened = []
        for edge_id in self.crossings[relationship]["native_edges"]:
            for slot in self.edges[edge_id]["parameter_slots"]:
                binding = self.bindings[(edge_id, slot)]
                if binding["binding"] == _OPEN:
                    opened.append(slot)
        self.open_parameters.update(opened)
        return sorted(set(opened))

    def host_unselected(self, relationship: str) -> bool:
        """True when the crossing is delegated to a host whose identity is open."""
        crossing = self.crossings[relationship]
        if crossing["execution"] != "delegated":
            return False
        slot = crossing["delegation"]["host_identity_parameter"]
        return self.bindings[(crossing["native_edges"][0], slot)]["binding"] == _OPEN

    def enter(self, relationship: str, semantic: bool) -> tuple[str | None, dict[str, Any]]:
        """Resolve and, for semantic subjects, enter one crossing."""
        failure = self.resolve(relationship)
        if failure is not None:
            return "unsupported", self._crossing_record(relationship, "unsupported", f"crossing_{failure}")
        if not semantic:
            return None, self._crossing_record(relationship, "resolved", None)
        opened = self._note_open_parameters(relationship)
        if self.host_unselected(relationship):
            record = self._crossing_record(relationship, "unsupported", "delegated_host_unselected")
            record["open_parameters"] = opened
            return "unsupported", record
        record = self._crossing_record(relationship, "succeeded", None)
        record["open_parameters"] = opened
        return None, record


def _conversion_mode(model: dict[str, Any], operation: str) -> str:
    matches = [item["mode"] for item in model["conversions"] if item["operation"] == operation]
    _require(len(matches) == 1, f"conversion {operation} is not unique")
    return matches[0]


def _word(text: str) -> int:
    _require(re.fullmatch(r"0x[0-9a-f]{8}", text) is not None, "malformed Word[32] literal")
    return int(text, 16)


def _hex_word(value: int) -> str:
    return f"0x{value & 0xFFFFFFFF:08x}"


_CALL = re.compile(r"\s*([A-Za-z_][A-Za-z0-9_^]*)\s*")


def _parse_expression(text: str) -> Any:
    position = 0

    def parse() -> Any:
        nonlocal position
        match = _CALL.match(text, position)
        if match is None:
            number = re.match(r"\s*([0-9]+)\s*", text[position:])
            _require(number is not None, "malformed word expression")
            position += number.end()
            return int(number.group(1))
        name = match.group(1)
        position = match.end()
        if position < len(text) and text[position] == "(":
            position += 1
            arguments = [parse()]
            while text[position] == ",":
                position += 1
                arguments.append(parse())
            _require(text[position] == ")", "unbalanced word expression")
            position += 1
            return (name, arguments)
        return name

    tree = parse()
    _require(position == len(text), "trailing word expression text")
    return tree


def _evaluate(tree: Any, inputs: dict[str, int]) -> int:
    if isinstance(tree, int):
        return tree
    if isinstance(tree, str):
        _require(tree in inputs, f"unbound word input {tree}")
        return inputs[tree]
    name, arguments = tree
    values = [_evaluate(argument, inputs) for argument in arguments]
    if name == "xor_32" and len(values) == 2:
        return (values[0] ^ values[1]) & 0xFFFFFFFF
    if name == "rotate_left_32" and len(values) == 2:
        amount = values[1]
        _require(0 <= amount < 32, "rotation exceeds the word width")
        word = values[0] & 0xFFFFFFFF
        return ((word << amount) | (word >> (32 - amount))) & 0xFFFFFFFF if amount else word
    if name == "choice_32" and len(values) == 3:
        return ((values[0] & values[1]) ^ (~values[0] & values[2])) & 0xFFFFFFFF
    if name == "add_mod_2^32" and len(values) == 2:
        return (values[0] + values[1]) & 0xFFFFFFFF
    raise AdapterFailure("unsupported_behavior", f"unknown word operator {name}")


def _operators(tree: Any) -> list[str]:
    if isinstance(tree, tuple):
        return [tree[0], *[name for argument in tree[1] for name in _operators(argument)]]
    return []


def _range(text: str) -> tuple[int, int]:
    match = re.fullmatch(r"(\d+) <= index < (\d+)", text)
    _require(match is not None, "malformed index range")
    return int(match.group(1)), int(match.group(2))


def _xor_constant(text: str) -> int:
    match = re.search(r"xor 0x([0-9a-f]{2})\)?$", text)
    _require(match is not None, "malformed byte operation")
    return int(match.group(1), 16)


class CaseSemantics:
    """Candidate-neutral fact extraction and computations for one positive model."""

    def __init__(self, case: str, model: dict[str, Any]) -> None:
        self.case = case
        self.model = model
        self.facts: dict[str, str] = {}
        self.values: dict[str, Any] = {}
        self.obligations: list[dict[str, Any]] = []
        getattr(self, "_facts_" + case.replace("-", "").lower())()

    # Fact extraction ----------------------------------------------------
    def _facts_sc01(self) -> None:
        model = self.model
        ast = model["round_like_ast"]
        tree = _parse_expression(ast["operation"])
        domains = {item["id"]: item["interpretation"] for item in model["domains"]}
        widths_declared = (
            domains.get("Word[32]") == "unsigned_modulo_2^32"
            and ast["output_type"] == "Word[32]"
            and all(item["type"] == "Word[32]" for item in ast["inputs"])
            and all(name.endswith("_32") or name.endswith("2^32") for name in _operators(tree))
        )
        self.tree = tree
        self.facts = {
            "integer_to_word_conversion_mode": _conversion_mode(model, "int_to_word_checked"),
            "byte_order_conversion_mode": _conversion_mode(model, "word_to_bytes_big_endian"),
            "word_width": "declared_width" if widths_declared else "mismatched_width",
            "shift_bound": model["shift_bound"],
        }

    def _facts_sc02(self) -> None:
        implementation = self.model["implementation_subject"]
        region = implementation["region"]
        pure = self.model["pure_subject"]
        obligation = self.model["refinement_obligation"]
        low, high = _range(implementation["loop"]["range"])
        bound_low, bound_high = _range(region["bounds"])
        within = bound_low <= low and high <= bound_high and bound_high == region["length"]
        self.facts = {
            "mutable_aliasing": region["aliasing"],
            "memory_access_range": "within_declared_range" if within else "outside_declared_range",
            "loop_invariant": "required_present" if implementation["loop"]["invariant"] else "absent",
            "read_initialization": "initialized"
            if region["initialization"] == "all_bytes_initialized"
            else "uninitialized",
            "refinement_subject_identity": "original"
            if obligation["implementation_subject"] == implementation["id"]
            and obligation["specification_subject"] == pure["id"]
            else "substitute",
        }

    def _facts_sc03(self) -> None:
        self.facts = {key: value for key, value in self.model["controls"].items()}

    def _facts_sc04(self) -> None:
        model = self.model
        intrinsic = model["abstract_intrinsic"]
        machine = model["machine_subject"]
        pure = model["pure_subject"]
        lanes = (pure["left"], pure["right"], pure["result"])
        self.facts = {
            "required_target_feature": "declared_present"
            if machine["required_feature"] == intrinsic["feature"]
            else "absent",
            "intrinsic_identity": "declared_supported_intrinsic"
            if _SUPPORTED_INTRINSICS.get(intrinsic["id"]) == intrinsic["operation"] == pure["operation"]
            else "unsupported_intrinsic",
            "lane_order": "declared_order" if model["lane_order"] == "lane_0_to_lane_3" else "reversed_order",
            "vector_width": "declared_width"
            if model["vector_width"] == "4xWord[32]"
            and pure["type"] == "Word[32][4]"
            and all(len(item) == 4 for item in lanes)
            else "mismatched_width",
            "target_model_identity": "original"
            if machine["target_model_identity"]
            and model["lowering_obligation"]["machine_subject"] == machine["id"]
            else "substitute",
            "fallback_authorization": model["declared_fallback"]["authorization"]
            if model["declared_fallback"]["path"] == "separate_checked_path"
            else "undeclared_selected",
        }

    def _facts_sc05(self) -> None:
        model = self.model
        space = model["sample_space"]
        games = model["games"]
        oracles = {item["id"]: item for item in model["oracle_interfaces"]}
        relation = model["reduction_relation"]
        endpoints = [game["id"] for game in games]
        bounded = (
            space["finite"] is True
            and isinstance(space["elements"], list)
            and 0 < len(space["elements"]) <= _MAX_SAMPLE_SPACE
        )
        bound = relation["bound"]
        self.facts = {
            "sampling_authority": space["authority"],
            "randomness_source": "explicit_sampling"
            if space["sampling"] == "explicit"
            and all(game["sampling"] == f"explicit_from_{space['id']}" for game in games)
            else "ambient_randomness",
            "oracle_effect_visibility": "explicit"
            if all(
                game["oracle"] in oracles and oracles[game["oracle"]]["effects"] == ["explicit_query"]
                for game in games
            )
            else "hidden",
            "sample_space_bound": "finite" if bounded else "unbounded",
            "reduction_endpoint_identity": "original"
            if relation["endpoints"] == endpoints
            and relation["direction"] == f"{endpoints[0]}_to_{endpoints[1]}"
            else "substitute",
            "symbolic_advantage_bound": "original"
            if relation["status"] == "open"
            and bound.startswith(f"Adv({endpoints[0]},")
            and f"Adv({endpoints[1]}," in bound
            else "altered",
        }

    # Computations -------------------------------------------------------
    def compute(self, relationship: str) -> str | None:
        """Run the computations owned by ``relationship``; return a failure category."""
        method = getattr(self, f"_compute_{self.case.replace('-', '').lower()}_{relationship.replace('-', '').lower()}", None)
        return None if method is None else method()

    def _compute_sc01_sr01(self) -> str | None:
        model = self.model
        if model["signedness"] != "unsigned" or model["byte_order"] != "big_endian" or model["round_like_ast"]["total"] is not True:
            return "boundary_interpretation_mismatch"
        names = [item["id"] for item in model["round_like_ast"]["inputs"]]
        results = []
        for vector in model["boundary_vectors"]:
            _require(sorted(vector["inputs"]) == sorted(names), "boundary vector inputs differ from the AST")
            inputs = {name: _word(vector["inputs"][name]) for name in names}
            results.append(_hex_word(_evaluate(self.tree, inputs)))
        self.values["word32_results"] = results
        if results != [vector["normalized_word32_result"] for vector in model["boundary_vectors"]]:
            return "evaluation_mismatch"
        return None

    def _compute_sc02_sr01(self) -> str | None:
        pure = self.model["pure_subject"]
        if pure["effects"] != [] or pure["total"] is not True:
            return "effect_in_pure_subject"
        key = _xor_constant(pure["operation"])
        output = [byte ^ key for byte in self.model["input_bytes"]]
        self.values["pure_output_bytes"] = output
        return None if output == pure["output_bytes"] else "evaluation_mismatch"

    def _compute_sc02_sr02(self) -> str | None:
        implementation = self.model["implementation_subject"]
        region = implementation["region"]
        if not set(implementation["effects"]) <= {"read_owned_region", "write_owned_region"}:
            return "undeclared_effect"
        if region["ownership"] != "owned_mutable" or not implementation["failure"].startswith("typed_"):
            return "ownership_or_failure_untyped"
        buffer = list(self.model["input_bytes"])
        _require(len(buffer) == region["length"], "input length differs from the owned region")
        key = _xor_constant(implementation["operation"])
        low, high = _range(implementation["loop"]["range"])
        for index in range(low, high):
            buffer[index] ^= key
        self.values["implementation_output_bytes"] = buffer
        return None

    def _compute_sc02_sr09(self) -> str | None:
        obligation = self.model["refinement_obligation"]
        if obligation["required_state"] != "open_or_discharged" or obligation["status"] not in ("open", "discharged"):
            return "refinement_state_invalid"
        agrees = self.values.get("implementation_output_bytes") == self.values.get("pure_output_bytes")
        self.obligations.append(
            {
                "id": "refinement",
                "left": obligation["implementation_subject"],
                "right": obligation["specification_subject"],
                "state": "open",
                "concrete_agreement": agrees,
            }
        )
        return None if agrees else "refinement_counterexample"

    def _compute_sc03_sr02(self) -> str | None:
        controls = self.model["controls"]
        if sorted(self.model["observation_channels"]) != sorted(controls):
            return "observation_channel_missing"
        return None

    def _compute_sc03_sr10(self) -> str | None:
        if self.model["preservation_requirement"] != "all_required_channels_visible_at_policy_boundary":
            return "observation_erased_before_policy_boundary"
        self.values["visible_channels"] = sorted(self.model["observation_channels"])
        self.values["policy_hook"] = self.model["policy_hook"]["identity"]
        return None

    def _compute_sc04_sr01(self) -> str | None:
        pure = self.model["pure_subject"]
        lanes = [
            _hex_word(_word(left) + _word(right)) for left, right in zip(pure["left"], pure["right"])
        ]
        self.values["lane_results"] = lanes
        return None if lanes == pure["result"] else "evaluation_mismatch"

    def _compute_sc04_sr11(self) -> str | None:
        obligation = self.model["lowering_obligation"]
        if obligation["abstract_subject"] != self.model["abstract_intrinsic"]["id"] or obligation["status"] != "open":
            return "lowering_obligation_mismatch"
        self.obligations.append(
            {
                "id": "lowering",
                "left": obligation["abstract_subject"],
                "right": obligation["machine_subject"],
                "state": "open",
                "fallback_path": self.model["declared_fallback"]["id"],
            }
        )
        return None

    def _compute_sc05_sr04(self) -> str | None:
        adversary = self.model["adversary_interface"]
        if not adversary.get("id") or len(self.model["games"]) != 2:
            return "adversary_or_game_boundary_missing"
        self.values["sample_space_size"] = len(self.model["sample_space"]["elements"])
        return None

    def _compute_sc05_sr08(self) -> str | None:
        shared = self.model["shared_pure_subject"]
        if shared["sampling"] is not False or shared["embedding"] != "explicit_into_game_core":
            return "implicit_or_sampling_embedding"
        return None

    def _compute_sc05_sr12(self) -> str | None:
        relation = self.model["reduction_relation"]
        self.obligations.append(
            {
                "id": "reduction",
                "left": relation["endpoints"][0],
                "right": relation["endpoints"][1],
                "state": "open",
                "bound": relation["bound"],
            }
        )
        return None


_MUTATION_FIELDS = {
    "baseline_value",
    "dependent_result",
    "kind",
    "mutated_value",
    "operator",
    "target",
}


def _violation_state(rule: dict[str, Any], positive: dict[str, Any]) -> str:
    if rule["violation"] == "policy_hook":
        return "unknown" if positive["policy_hook"]["status"] == "unselected" else "rejected"
    return rule["violation"]


def _semantic_observation(engine: Engine, subject: dict[str, Any], positive: dict[str, Any]) -> dict[str, Any]:
    case = engine.case
    semantics = CaseSemantics(case, positive["model"])
    facts = dict(semantics.facts)
    dependent = False
    if subject["kind"] == "suite-only-named-mutation":
        mutation = _closed(subject["model"], _MUTATION_FIELDS, "mutation model")
        _require(mutation["target"] in facts, "mutation target is not a registered case fact")
        _require(
            facts[mutation["target"]] == mutation["baseline_value"],
            "mutation baseline does not bind the positive subject",
            "digest_mismatch",
        )
        facts[mutation["target"]] = mutation["mutated_value"]
        dependent = True
    crossings = []
    decision = {"rule": None, "relationship": None, "member": None, "category": None}
    state = "succeeded"
    for relationship in subject["relationship_scope"]:
        failure, record = engine.enter(relationship, semantic=True)
        crossings.append(record)
        if failure is not None:
            state = failure
            decision.update(relationship=relationship, member=record["member"], category=record["category"])
            break
        for rule in RULES:
            if rule["case"] != case or rule["relationship"] != relationship:
                continue
            if facts[rule["fact"]] not in rule["admissible"]:
                state = _violation_state(rule, positive["model"])
                decision = {
                    "rule": rule["id"],
                    "relationship": relationship,
                    "member": record["member"],
                    "category": "admissibility_violation",
                }
                record["state"] = state
                break
        if state != "succeeded":
            break
        category = semantics.compute(relationship)
        if category is not None:
            state = "rejected"
            record["state"] = state
            decision = {"rule": None, "relationship": relationship, "member": record["member"], "category": category}
            break
    return {
        "layer": "semantic",
        "observed_state": state,
        "dependent": dependent,
        "crossings": crossings,
        "decision": decision,
        "values": semantics.values,
        "obligations": semantics.obligations,
    }


def _structural_observation(engine: Engine, subject: dict[str, Any]) -> dict[str, Any]:
    model = subject["model"]
    crossings = []
    decision = {"rule": None, "relationship": None, "member": None, "category": None}
    for relationship in subject["relationship_scope"]:
        failure, record = engine.enter(relationship, semantic=False)
        crossings.append(record)
        if failure is not None:
            decision.update(relationship=relationship, member=record["member"], category=record["category"])
            return {"layer": "structural", "observed_state": failure, "dependent": True, "crossings": crossings, "decision": decision, "values": {}, "obligations": []}
    kind = model["kind"]
    state = "succeeded"
    values: dict[str, Any] = {}
    if kind == "missing-edge":
        _require(model["baseline_relationships"] == list(RELATIONSHIPS), "missing-edge baseline is not SR-01..SR-14")
        active = set(model["mutated_relationships"])
        missing = [item for item in model["dependent_result"]["required_relationships"] if item not in active]
        if missing:
            state = "rejected"
            decision.update(relationship=missing[0], category="missing_relationship")
        values["active_relationship_count"] = len(active)
    elif kind == "identity-substitution":
        required = model["dependent_result"]["required_binding"]
        bound = [item for item in model["mutated_bindings"] if item["slot"] == required["slot"]]
        _require(len(bound) == 1, "identity slot is not bound exactly once")
        _require(_HEX.fullmatch(bound[0]["identity_sha256"]) is not None, "identity is not a SHA-256 digest")
        if bound[0]["identity_sha256"] != required["identity_sha256"]:
            state = "rejected"
            decision["category"] = "identity_substitution"
        values["slot"] = required["slot"]
    elif kind == "ambiguity":
        interpretations = model["interpretations"]
        if model["dependent_result"]["requires_unique_authority"] is True and len(interpretations) != 1:
            state = "rejected"
            decision["category"] = "competing_authority"
        values["interpretation_count"] = len(interpretations)
    elif kind == "unsupported":
        operation = model["request"]["operation"]
        domain = model["support_domain"]
        if operation in domain["unsupported_operations"]:
            state = "unsupported"
            decision["category"] = "domain_unsupported"
        elif operation not in domain["supported_operations"]:
            state = "rejected"
            decision["category"] = "unknown_operation"
    elif kind == "resource-exhaustion":
        limit = model["resource_domain"]["limit"]
        _require(isinstance(limit, int) and limit >= 0, "resource limit is not a natural number")
        consumed = 0
        for _ in model["request"]["work_items"]:
            if consumed == limit:
                state = "exhausted"
                decision["category"] = "domain_exhausted"
                break
            consumed += 1
        values["consumed_work_items"] = consumed
    else:
        raise AdapterFailure("unsupported_behavior", f"unknown structural subject kind {kind}")
    return {"layer": "structural", "observed_state": state, "dependent": True, "crossings": crossings, "decision": decision, "values": values, "obligations": []}


def _context_entry(entry_id: str, description: str, status: str) -> dict[str, str]:
    return {
        "id": entry_id,
        "description": description,
        "sha256": digest({"description": description, "id": entry_id, "status": status}),
        "status": status,
    }


def _verify_candidate_model(bundle: dict[str, Any], candidate: str) -> None:
    _closed(bundle, _MODEL_FIELDS, "candidate_model")
    _require(bundle["candidate"] == candidate, "candidate model is not the scheduled candidate", "digest_mismatch")
    for field, value in (
        ("graph_sha256", bundle["graph"]),
        ("sr_map_sha256", bundle["graph"]["sr_rows"]),
        ("model_sha256", bundle["model"]),
        ("semantic_endpoint_sha256", bundle["semantic_endpoints"]),
        ("parameter_model_sha256", bundle["parameter_bindings"]),
    ):
        _require(bundle[field] == digest(value), f"{field} does not match its bytes", "digest_mismatch")
    derived = derive_candidate_model(
        {"candidate": candidate, "graph": bundle["graph"], "graph_sha256": bundle["graph_sha256"]}
    )
    _require(derived == bundle, "candidate model is not derived from its graph", "digest_mismatch")


def run_request(request_bytes: bytes, source_sha256: str) -> bytes:
    """Evaluate one closed request and return canonical response bytes."""
    _require(len(request_bytes) <= MAX_REQUEST_BYTES, "request exceeds the transport bound")
    try:
        request = strict_loads(request_bytes)
    except (UnicodeDecodeError, ValueError) as error:
        raise AdapterFailure("unsupported_behavior", f"request is not strict JSON: {error}") from None
    _require(canonical_bytes(request) + b"\n" == request_bytes, "request is not canonical transport")
    _closed(request, {"schema_version", "suite_version", "execution", "candidate_model", "subjects"}, "request")
    _require(request["schema_version"] == REQUEST_SCHEMA and request["suite_version"] == SUITE_VERSION, "request schema drift")
    execution = _closed(request["execution"], _EXECUTION_FIELDS, "execution")
    candidate, case = execution["candidate"], execution["case"]
    _require(candidate in CANDIDATES and case in CASES, "unknown candidate or case")
    _verify_candidate_model(request["candidate_model"], candidate)
    subjects = request["subjects"]
    _require(isinstance(subjects, list) and subjects, "request has no subjects", "missing_input")
    for entry in subjects:
        _closed(entry, _SUBJECT_FIELDS, "subject entry")
        _require(entry["source_catalog"] in SOURCE_CATALOGS, "unknown source catalog")
        _require(entry["subject_sha256"] == digest(entry["subject"]), "subject digest mismatch", "digest_mismatch")
        _require(entry["subject"].get("id", entry["subject"].get("proposal_id")) == entry["subject_id"], "subject identifier mismatch", "digest_mismatch")
    positive = subjects[0]["subject"]
    _require(
        subjects[0]["source_catalog"] == "case_subject_catalog"
        and positive.get("kind") == "suite-only-positive-case"
        and positive.get("case") == case,
        "the first subject is not the scheduled positive subject",
        "missing_input",
    )
    engine = Engine(request["candidate_model"], case)
    observations = []
    seen = set()
    for entry in subjects:
        subject = entry["subject"]
        observation_id = f"obs-{entry['subject_id']}"
        _require(observation_id not in seen, "duplicate subject in request")
        seen.add(observation_id)
        if entry["source_catalog"] == "case_subject_catalog":
            _require(subject["case"] == case, "case subject is outside the scheduled case", "digest_mismatch")
            if subject["kind"] == "suite-only-named-mutation":
                _require(subject["positive_subject_sha256"] == subjects[0]["subject_sha256"], "mutation does not bind the positive subject", "digest_mismatch")
            else:
                _require(entry is subjects[0], "extra positive subject")
            outcome = _semantic_observation(engine, subject, positive)
        else:
            _require(case in subject["case_scope"], "fixture is outside the scheduled case", "digest_mismatch")
            outcome = _structural_observation(engine, subject)
        state = outcome["observed_state"]
        if not outcome["dependent"]:
            invalidation = "not_required"
        else:
            invalidation = "not_satisfied" if state == "succeeded" else "satisfied"
        crossing_parameters = sorted({slot for item in outcome["crossings"] for slot in item.pop("open_parameters", [])})
        normalized = {
            "subject_id": entry["subject_id"],
            "subject_sha256": entry["subject_sha256"],
            "observation_level": "domain",
            "layer": outcome["layer"],
            "observed_state": state,
            "observed_invalidation": invalidation,
            "crossings": outcome["crossings"],
            "decision": outcome["decision"],
            "values": outcome["values"],
            "obligations": outcome["obligations"],
            "open_parameters": crossing_parameters,
        }
        observations.append(
            {
                "id": observation_id,
                "subject_id": entry["subject_id"],
                "subject_sha256": entry["subject_sha256"],
                "observed_state": state,
                "observed_invalidation": invalidation,
                "normalized_observation": normalized,
                "normalized_observation_sha256": digest(normalized),
            }
        )
    sr_rows = []
    for relationship in RELATIONSHIPS:
        crossing = engine.crossings[relationship]
        if engine.resolve(relationship) is not None:
            state = "not_satisfied"
        elif engine.host_unselected(relationship):
            state = "unsupported"
        else:
            state = "satisfied"
        sr_rows.append(
            {
                "relationship": relationship,
                "native_edges": list(crossing["native_edges"]),
                "conformance_state": state,
                "dependent_observation_ids": [
                    f"obs-{entry['subject_id']}"
                    for entry in subjects
                    if relationship in entry["subject"]["relationship_scope"]
                ],
            }
        )
    bundle = request["candidate_model"]
    premises = [
        _context_entry("P-CANDIDATE-GRAPH", f"{candidate} graph {bundle['graph_sha256']} is a D004-PRE-01 reviewed falsifiable hypothesis, not accepted semantics", "reviewed_hypothesis"),
        _context_entry("P-S3A", "accepted S3a Int and Word[8] typed-literal meaning is preserved", "accepted_premise"),
        _context_entry("P-SUITE-MODELS", f"{SUITE_VERSION} suite-only subject models are the only semantic inputs", "suite_only"),
    ]
    assumptions = []
    for slot in sorted(engine.open_parameters):
        decision_id = SLOT_BINDINGS[slot][2]
        assumptions.append(
            _context_entry(f"A-OPEN-{slot}", f"{slot} stays an unselected {decision_id} parameter carried symbolically", "open")
        )
    trusted = [
        _context_entry("T-ADAPTER", f"candidate-neutral adapter source {source_sha256}", "trusted_research_only"),
        _context_entry("T-CANDIDATE-MODEL", f"derived candidate model {bundle['model_sha256']}", "trusted_research_only"),
        _context_entry("T-RULES", f"candidate-neutral rule table {digest(rule_table())}", "trusted_research_only"),
    ]
    unsupported = []
    for row in sr_rows:
        if row["conformance_state"] != "unsupported":
            continue
        crossing = engine.crossings[row["relationship"]]
        slot = crossing["delegation"]["host_identity_parameter"]
        unsupported.append(
            {
                "id": f"U-{row['relationship']}",
                "description": f"{row['relationship']} is delegated through {crossing['native_edges'][0]} and {slot} is an unselected {SLOT_BINDINGS[slot][2]} parameter",
                "state": "unsupported",
                "dependent_result": list(row["dependent_observation_ids"]),
            }
        )
    response = {
        "schema_version": RESPONSE_SCHEMA,
        "request_sha256": hashlib.sha256(request_bytes[:-1]).hexdigest(),
        "candidate": candidate,
        "case": case,
        "observations": observations,
        "sr_conformance": sr_rows,
        "premises": premises,
        "assumptions": assumptions,
        "trusted_components": trusted,
        "unsupported_features": unsupported,
    }
    return canonical_bytes(response) + b"\n"


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        sys.stderr.write("usage: d004_adapter.py REQUEST\n")
        return EXIT_CODES["unsupported_behavior"]
    try:
        source_sha256 = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        try:
            request_bytes = Path(argv[1]).read_bytes()
        except FileNotFoundError:
            raise AdapterFailure("missing_input", "request file is absent") from None
        response = run_request(request_bytes, source_sha256)
    except AdapterFailure as failure:
        sys.stderr.write(f"{failure.kind}: {failure}\n")
        return EXIT_CODES[failure.kind]
    except MemoryError:
        sys.stderr.write("resource_exhaustion: memory\n")
        return EXIT_CODES["resource_exhaustion"]
    except (KeyError, TypeError, IndexError, AttributeError) as error:
        sys.stderr.write(f"unsupported_behavior: subject shape {type(error).__name__}\n")
        return EXIT_CODES["unsupported_behavior"]
    sys.stdout.buffer.write(response)
    sys.stdout.buffer.flush()
    return EXIT_CODES["completed"]


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
