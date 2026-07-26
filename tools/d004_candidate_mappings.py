"""Focused validation for D-004 candidate mappings and their result-contract binding."""

from __future__ import annotations

import hashlib
import re
from typing import Any, Callable


def validate_candidate_mapping_catalog(
    context: Any,
    *,
    catalog_path: str,
    canonical_sha256: str,
    raw_sha256: str,
    subject_sha256: str,
    suite_raw_sha256: str,
    canonical_json_bytes: Callable[[Any], bytes],
) -> None:
    path = context.root / catalog_path

    def fail(suffix: str, message: str) -> None:
        context.add(f"d004_packet.mapping_catalog_{suffix}", path, message)

    raw = context._read_repository_bytes(path)
    if raw is None:
        fail("missing", "closed D-004 candidate-mapping catalog is missing")
        return
    try:
        catalog = context._load_repository_json(path)
    except (OSError, UnicodeDecodeError, ValueError, TypeError) as exc:
        fail("parse", f"cannot strictly parse candidate-mapping catalog: {exc}")
        return
    try:
        canonical = canonical_json_bytes(catalog)
    except (TypeError, ValueError) as exc:
        fail("canonical", f"candidate-mapping catalog is outside the canonical I-JSON profile: {exc}")
        return
    if raw != canonical + b"\n":
        fail("canonical", "candidate-mapping catalog must use canonical JSON and one terminal LF")
    if (
        hashlib.sha256(canonical).hexdigest()
        != canonical_sha256
        or hashlib.sha256(raw).hexdigest()
        != raw_sha256
    ):
        fail("identity", "candidate-mapping canonical or raw SHA-256 identity drifted")
    if not isinstance(catalog, dict):
        fail("schema", "candidate-mapping catalog root must be an object")
        return

    root_fields = {
        "canonicalization", "catalog_subject", "catalog_subject_sha256", "epoch",
        "evidence_status", "execution_boundary", "frozen", "nonclaims",
        "open_decisions", "owner_protocol_review", "schema_version",
        "source_bindings", "status", "suite_version",
    }
    if set(catalog) != root_fields:
        fail("schema", "candidate-mapping catalog root fields are not closed")

    nonclaims = [
        "candidate mappings are draft hypotheses and are not accepted Orange semantics",
        "no candidate adapter executable command or process exists",
        "no candidate mapping was executed",
        "no observed state comparison result or verdict was produced",
        "no candidate capability or capability absence was established",
        "mapping authentication is not D-004 execution evidence",
        "no D-004 evidence epoch is frozen",
        "no semantic-strata candidate is selected",
        "no D-004 disposition is inferred",
        "no S3b implementation roadmap gate closure or release-readiness movement is authorized",
    ]
    exact_boundary = {
        "canonicalization": "RFC8785_ASCII_INTEGER_SUBSET",
        "epoch": None,
        "evidence_status": "none",
        "execution_boundary": {
            "adapter": "absent",
            "command": "absent",
            "network": "not_used",
            "output_persistence": "none",
            "process": "not_invoked",
        },
        "frozen": False,
        "nonclaims": nonclaims,
        "owner_protocol_review": "none",
        "schema_version": "d004-candidate-mapping-catalog-v0.1",
        "status": "draft_unreviewed_input_only",
        "suite_version": "d004-v0.5-draft",
    }
    if any(catalog.get(key) != value for key, value in exact_boundary.items()):
        fail("boundary", "catalog weakened its exact input-only, unreviewed, zero-evidence boundary")

    forbidden_fields = {
        "adapter_argv", "adapter_output", "adapter_path", "candidate_execution",
        "candidate_observation", "capability_credit", "case_result", "case_verdict",
        "environment", "evidence", "execution_result", "loader_status", "matched",
        "observed_invalidation", "observed_state", "readiness_credit", "result",
        "verdict",
    }
    pending: list[Any] = [catalog]
    while pending:
        value = pending.pop()
        if isinstance(value, dict):
            if forbidden_fields.intersection(value):
                fail("nonclaim", "catalog contains a forbidden execution, evidence, verdict, or readiness field")
                break
            pending.extend(value.values())
        elif isinstance(value, list):
            pending.extend(value)

    expected_source_bindings = {
        "suite": {
            "path": "docs/SEMANTIC_STRATA_DECISION_SUITE.md",
            "raw_sha256": suite_raw_sha256,
        }
    }
    if catalog.get("source_bindings") != expected_source_bindings:
        fail("source_binding", "catalog must bind the exact v0.5 suite source bytes")
    suite_path = context.root / "docs/SEMANTIC_STRATA_DECISION_SUITE.md"
    suite_raw = context._read_repository_bytes(suite_path)
    if suite_raw is None or hashlib.sha256(suite_raw).hexdigest() != suite_raw_sha256:
        fail("source_binding", "bound semantic-strata suite bytes are unavailable or drifted")

    expected_open_decisions = [
        {"id": identifier, "scope": scope, "status": "unselected"}
        for identifier, scope in (
            ("D-005", "claim model"),
            ("D-006", "proof foundation"),
            ("D-007", "proof format and checker"),
            ("D-009", "solver trust policy"),
            ("D-010", "compiler-pass strategy"),
            ("D-011", "host target ISA and object-format envelope"),
            ("D-012", "leakage observations and declassification policy"),
            ("D-013", "ABI"),
        )
    ]
    if catalog.get("open_decisions") != expected_open_decisions:
        fail("open_decisions", "all eight dependent decisions must remain ordered and unselected")

    subject = catalog.get("catalog_subject")
    subject_fields = {
        "candidate_count", "candidate_graphs", "mapping_row_count",
        "relationship_count", "required_relationships",
    }
    if not isinstance(subject, dict):
        fail("schema", "catalog_subject must be an object")
        return
    if set(subject) != subject_fields:
        fail("schema", "catalog_subject fields are not closed")
    subject_digest = hashlib.sha256(canonical_json_bytes(subject)).hexdigest()
    if (
        catalog.get("catalog_subject_sha256") != subject_digest
        or catalog.get("catalog_subject_sha256")
        != subject_sha256
    ):
        fail("subject_digest", "catalog-subject canonical identity drifted")
    if (
        subject.get("candidate_count") != 5
        or subject.get("relationship_count") != 14
        or subject.get("mapping_row_count") != 70
    ):
        fail("counts", "catalog must declare exactly 5 graphs, 14 relationships, and 70 mapping rows")

    relationship_fields = {
        "codomain", "crossing", "definedness", "direction", "domain",
        "failure_behavior", "id", "identity_inputs", "obligations", "observation",
        "prohibited_reverse_inferences", "trust_role",
    }
    expected_relationships = [f"SR-{index:02d}" for index in range(1, 15)]
    relationships = subject.get("required_relationships")
    relationship_digests: list[str] = []
    if not isinstance(relationships, list) or len(relationships) != 14:
        fail("relationship_inventory", "required relationships must contain exactly 14 ordered rows")
        relationships = []
    relationship_ids: list[str] = []
    for index, relationship in enumerate(relationships):
        if not isinstance(relationship, dict) or set(relationship) != relationship_fields:
            fail("relationship_schema", f"required relationship {index} fields are not closed")
            relationship_digests.append("")
            continue
        relationship_ids.append(str(relationship.get("id", "")))
        if any(
            not isinstance(relationship.get(field), str) or not relationship.get(field)
            for field in relationship_fields
        ):
            fail("relationship_value", f"required relationship {index} has an empty or non-string field")
        relationship_digests.append(
            hashlib.sha256(canonical_json_bytes(relationship)).hexdigest()
        )
    if relationship_ids != expected_relationships or len(set(relationship_ids)) != 14:
        fail("relationship_order", "required relationships must be unique and ordered SR-01 through SR-14")

    graph_record_fields = {"candidate", "graph", "graph_sha256"}
    graph_fields = {"architecture", "candidate", "edges", "nodes", "sr_rows"}
    node_fields = {"authority", "facets", "id", "member_kind", "parent", "role"}
    edge_record_fields = {"edge_sha256", "edge_subject"}
    edge_fields = {
        "codomain_endpoints", "conformance_status", "definedness",
        "delegation_boundary", "direction", "domain_endpoint", "draft_status",
        "failure_behavior", "id", "identity_inputs", "obligations",
        "observation_requirement", "parameter_slots", "prohibited_reverse_inferences",
        "relationship", "trust_role",
    }
    mapping_record_fields = {"mapping", "mapping_sha256"}
    mapping_fields = {
        "candidate", "draft_hypothesis", "fused_authority", "inspectability",
        "mapping_form", "native_edges", "relationship",
        "required_relationship_sha256",
    }
    member_kinds = {
        "source_declaration", "semantic_domain", "semantic_view", "shared_subset",
        "evidence_interface", "claim_boundary", "judgment_boundary",
        "host_delegated_domain", "reference_set",
    }
    authorities = {
        "input_only", "candidate_local", "later_decision_boundary",
        "unresolved_checker_boundary", "host_delegated",
    }
    candidates = ["ST-REL", "ST-UNI", "ST-DUAL", "ST-MIRROR", "ST-HOST"]
    candidate_graphs = subject.get("candidate_graphs")
    if not isinstance(candidate_graphs, list) or len(candidate_graphs) != 5:
        fail("candidate_inventory", "candidate_graphs must contain exactly five ordered graphs")
        return
    if [row.get("candidate") if isinstance(row, dict) else None for row in candidate_graphs] != candidates:
        fail("candidate_order", "candidate graphs must be uniquely ordered by the frozen candidate inventory")

    graph_digests: set[str] = set()
    mapping_digests: set[str] = set()
    for candidate_index, graph_record in enumerate(candidate_graphs):
        candidate = candidates[candidate_index]
        prefix = candidate.removeprefix("ST-").lower() + "-"
        if not isinstance(graph_record, dict) or set(graph_record) != graph_record_fields:
            fail("graph_schema", f"{candidate} graph record fields are not closed")
            continue
        graph = graph_record.get("graph")
        if not isinstance(graph, dict) or set(graph) != graph_fields:
            fail("graph_schema", f"{candidate} graph fields are not closed")
            continue
        if (
            graph_record.get("candidate") != candidate
            or graph.get("candidate") != candidate
            or not isinstance(graph.get("architecture"), str)
            or not graph.get("architecture")
        ):
            fail("candidate_join", f"{candidate} graph identity or architecture drifted")

        nodes = graph.get("nodes")
        node_map: dict[str, dict[str, Any]] = {}
        if not isinstance(nodes, list) or not nodes:
            fail("node_inventory", f"{candidate} must retain a nonempty node inventory")
            nodes = []
        for node_index, node in enumerate(nodes):
            if not isinstance(node, dict) or set(node) != node_fields:
                fail("node_schema", f"{candidate} node {node_index} fields are not closed")
                continue
            node_id = node.get("id")
            facets = node.get("facets")
            if (
                not isinstance(node_id, str)
                or not node_id.startswith(prefix)
                or not isinstance(node.get("role"), str)
                or not node.get("role")
                or node.get("member_kind") not in member_kinds
                or node.get("authority") not in authorities
                or not isinstance(facets, list)
                or not facets
                or any(not isinstance(facet, str) or not facet for facet in facets)
                or len(set(facets)) != len(facets)
                or not (node.get("parent") is None or isinstance(node.get("parent"), str) and node.get("parent"))
            ):
                fail("node_value", f"{candidate} node {node_index} violates its closed identity or topology vocabulary")
            if isinstance(node_id, str):
                if node_id in node_map:
                    fail("node_identity", f"{candidate} contains duplicate node identity {node_id}")
                node_map[node_id] = node
        for node_id, node in node_map.items():
            parent = node.get("parent")
            if parent is not None and (parent == node_id or parent not in node_map):
                fail("topology", f"{candidate} node {node_id} has an unresolved or self parent")
        required_parents = {
            "ST-REL": {"rel-shared-pure": "rel-spec-core"},
            "ST-UNI": {
                "uni-pure-view": "uni-calculus", "uni-impl-view": "uni-calculus",
                "uni-game-view": "uni-calculus", "uni-machine-view": "uni-calculus",
                "uni-proof-interface": "uni-calculus",
            },
            "ST-DUAL": {
                "dual-game-view": "dual-effect-core", "dual-ct-view": "dual-effect-core",
                "dual-machine-view": "dual-effect-core",
                "dual-proof-interface": "dual-effect-core",
            },
            "ST-MIRROR": {"mirror-shared-pure": "mirror-spec-core"},
            "ST-HOST": {"host-judgment": "host-proof-domain"},
        }[candidate]
        if any(node_map.get(node, {}).get("parent") != parent for node, parent in required_parents.items()):
            fail("topology", f"{candidate} required containment topology drifted")
        if candidate == "ST-MIRROR" and node_map.get("mirror-proof-core", {}).get("member_kind") != "semantic_domain":
            fail("topology", "ST-MIRROR proof core must remain an independent semantic domain")

        edges = graph.get("edges")
        edge_map: dict[str, dict[str, Any]] = {}
        edge_endpoint_nodes: dict[str, set[str]] = {}
        referenced_nodes: set[str] = set()
        edge_hashes: set[str] = set()
        if not isinstance(edges, list) or len(edges) != 14:
            fail("edge_inventory", f"{candidate} must retain exactly 14 ordered native edges")
            edges = []
        for edge_index, edge_record in enumerate(edges):
            relationship_id = expected_relationships[edge_index]
            relationship = relationships[edge_index] if edge_index < len(relationships) and isinstance(relationships[edge_index], dict) else {}
            if not isinstance(edge_record, dict) or set(edge_record) != edge_record_fields:
                fail("edge_schema", f"{candidate} edge {edge_index} record fields are not closed")
                continue
            edge = edge_record.get("edge_subject")
            if not isinstance(edge, dict) or set(edge) != edge_fields:
                fail("edge_schema", f"{candidate} edge {edge_index} subject fields are not closed")
                continue
            edge_id = edge.get("id")
            if not isinstance(edge_id, str) or not edge_id.startswith(prefix) or edge_id in edge_map:
                fail("edge_identity", f"{candidate} edge {edge_index} identity is invalid or duplicate")
            if edge.get("relationship") != relationship_id:
                fail("edge_relationship", f"{candidate} edge {edge_index} is not joined to {relationship_id}")
            if (
                edge.get("draft_status") != "draft_unreviewed_unfrozen_hypothesis"
                or edge.get("conformance_status") != "unresolved"
            ):
                fail("edge_boundary", f"{candidate} edge {edge_index} claims reviewed or resolved conformance")
            copied_fields = {
                "direction": "direction", "definedness": "definedness",
                "obligations": "obligations", "identity_inputs": "identity_inputs",
                "trust_role": "trust_role", "failure_behavior": "failure_behavior",
                "prohibited_reverse_inferences": "prohibited_reverse_inferences",
                "observation_requirement": "observation",
            }
            if any(edge.get(target) != relationship.get(source) for target, source in copied_fields.items()):
                fail("edge_relationship", f"{candidate} edge {edge_index} weakens its required SR semantics")

            endpoint_nodes: set[str] = set()

            def endpoint_valid(endpoint: Any) -> bool:
                if not isinstance(endpoint, dict) or set(endpoint) != {"facet", "node"}:
                    return False
                endpoint_node = endpoint.get("node")
                facet = endpoint.get("facet")
                if not isinstance(endpoint_node, str) or endpoint_node not in node_map:
                    return False
                node_facets = node_map[endpoint_node].get("facets")
                if not isinstance(facet, str) or not isinstance(node_facets, list) or facet not in node_facets:
                    return False
                endpoint_nodes.add(endpoint_node)
                return True

            codomains = edge.get("codomain_endpoints")
            endpoints_valid = endpoint_valid(edge.get("domain_endpoint"))
            if not isinstance(codomains, list) or not codomains:
                endpoints_valid = False
                codomains = []
            if any(not endpoint_valid(endpoint) for endpoint in codomains):
                endpoints_valid = False
            if not endpoints_valid:
                fail("edge_endpoint", f"{candidate} edge {edge_index} has an unresolved node or facet endpoint")
            if edge_index == 13 and codomains != [{
                "facet": "candidate_wide_subject_or_evidence_reference",
                "node": f"{prefix}eligible-reference-set",
            }]:
                fail("topology", f"{candidate} SR-14 must target its candidate-wide eligible reference set")
            referenced_nodes.update(endpoint_nodes)

            slots = edge.get("parameter_slots")
            if (
                not isinstance(slots, list)
                or not slots
                or any(not isinstance(slot, str) or not slot for slot in slots)
                or len(set(slots)) != len(slots)
            ):
                fail("edge_parameters", f"{candidate} edge {edge_index} parameter slots are invalid")
                slots = []
            if candidate == "ST-HOST" and edge_index == 2 and {
                "delegation_selector", "host_identity", "host_non_success_policy",
            }.intersection(slots):
                fail("delegation", "ST-HOST SR-03 local edge retains generic delegation slots")
            delegation_domains = {3: "game", 4: "proof", 7: "game", 10: "machine", 11: "game", 12: "proof"}
            delegation_domain = delegation_domains.get(edge_index) if candidate == "ST-HOST" else None
            delegation = edge.get("delegation_boundary")
            if delegation_domain is None:
                if delegation is not None:
                    fail("delegation", f"{candidate} edge {edge_index} has an unauthorized delegation boundary")
            else:
                expected_delegation = {
                    "selector_parameter": f"{delegation_domain}_delegation_selector",
                    "host_identity_parameter": f"{delegation_domain}_host_identity",
                    "non_success_behavior": "reject_or_unsupported_without_fallback_claim",
                }
                required_slots = {
                    f"{delegation_domain}_delegation_selector",
                    f"{delegation_domain}_host_identity",
                    f"{delegation_domain}_non_success_policy",
                }
                if delegation != expected_delegation or not required_slots.issubset(set(slots)):
                    fail("delegation", f"{candidate} edge {edge_index} delegation boundary drifted")

            edge_digest = hashlib.sha256(canonical_json_bytes(edge)).hexdigest()
            if edge_record.get("edge_sha256") != edge_digest:
                fail("edge_digest", f"{candidate} edge {edge_index} digest drifted")
            if edge_digest in edge_hashes:
                fail("edge_identity", f"{candidate} contains duplicate edge-subject identities")
            edge_hashes.add(edge_digest)
            if isinstance(edge_id, str):
                edge_map[edge_id] = edge
                edge_endpoint_nodes[edge_id] = endpoint_nodes

        parent_nodes = {
            parent for node in node_map.values()
            if isinstance((parent := node.get("parent")), str)
        }
        if any(node_id not in referenced_nodes and node_id not in parent_nodes for node_id in node_map):
            fail("topology", f"{candidate} contains an orphan node outside all edge and containment paths")

        rows = graph.get("sr_rows")
        used_edges: set[str] = set()
        row_relationships: list[str] = []
        if not isinstance(rows, list) or len(rows) != 14:
            fail("mapping_inventory", f"{candidate} must retain exactly 14 ordered SR mappings")
            rows = []
        for row_index, row_record in enumerate(rows):
            relationship_id = expected_relationships[row_index]
            if not isinstance(row_record, dict) or set(row_record) != mapping_record_fields:
                fail("mapping_schema", f"{candidate} mapping {row_index} record fields are not closed")
                continue
            mapping = row_record.get("mapping")
            if not isinstance(mapping, dict) or set(mapping) != mapping_fields:
                fail("mapping_schema", f"{candidate} mapping {row_index} fields are not closed")
                continue
            row_relationships.append(str(mapping.get("relationship", "")))
            expected_relationship_digest = relationship_digests[row_index] if row_index < len(relationship_digests) else ""
            if (
                mapping.get("candidate") != candidate
                or mapping.get("relationship") != relationship_id
                or mapping.get("required_relationship_sha256") != expected_relationship_digest
                or mapping.get("inspectability") != "separately_inspectable_and_falsifiable"
            ):
                fail("mapping_join", f"{candidate} mapping {row_index} does not preserve its candidate/SR identity join")
            form = mapping.get("mapping_form")
            expected_delegated = candidate == "ST-HOST" and row_index not in {0, 1, 2, 5, 6, 8, 9, 13}
            if form not in {"direct", "fused", "split", "delegated"} or (form == "delegated") != expected_delegated:
                fail("mapping_form", f"{candidate} mapping {row_index} uses an invalid form")
            native_edges = mapping.get("native_edges")
            if not isinstance(native_edges, list):
                native_edges = []
            valid_cardinality = len(native_edges) >= 2 if form == "split" else len(native_edges) == 1
            if (
                not valid_cardinality
                or any(not isinstance(edge_id, str) or not edge_id for edge_id in native_edges)
                or len(set(native_edges)) != len(native_edges)
            ):
                fail("mapping_edges", f"{candidate} mapping {row_index} has invalid native-edge cardinality or identity")
            fused_authority = mapping.get("fused_authority")
            if form == "fused":
                if not isinstance(fused_authority, str) or fused_authority not in node_map or node_map[fused_authority].get("member_kind") != "semantic_domain":
                    fail("topology", f"{candidate} fused mapping {row_index} lacks a semantic-domain authority")
            elif fused_authority is not None:
                fail("mapping_form", f"{candidate} non-fused mapping {row_index} retains fused authority")
            for edge_id in native_edges:
                edge = edge_map.get(edge_id) if isinstance(edge_id, str) else None
                if (
                    edge is None
                    or not edge_id.startswith(prefix)
                    or edge.get("relationship") != relationship_id
                    or edge_id in used_edges
                ):
                    fail("mapping_edges", f"{candidate} mapping {row_index} references an unresolved, reused, or wrong-SR edge")
                    continue
                used_edges.add(edge_id)
                endpoints = edge_endpoint_nodes.get(edge_id, set())
                if form == "fused" and isinstance(fused_authority, str):
                    if any(
                        node_map[endpoint].get("member_kind") != "source_declaration"
                        and endpoint != fused_authority
                        and node_map[endpoint].get("parent") != fused_authority
                        for endpoint in endpoints
                    ):
                        fail("topology", f"{candidate} fused mapping {row_index} crosses outside its authority containment")
                if form == "delegated" and not any(
                    node_map[endpoint].get("authority") == "host_delegated"
                    or node_map.get(str(node_map[endpoint].get("parent")), {}).get("authority") == "host_delegated"
                    for endpoint in endpoints
                ):
                    fail("delegation", f"{candidate} delegated mapping {row_index} does not cross a host-delegated endpoint")
            hypothesis = mapping.get("draft_hypothesis")
            if (
                not isinstance(hypothesis, str)
                or not hypothesis.startswith(f"Draft hypothesis: {candidate} ")
                or relationship_id not in hypothesis
                or "unreviewed" not in hypothesis
                or "unfrozen" not in hypothesis
            ):
                fail("mapping_boundary", f"{candidate} mapping {row_index} overstates its draft hypothesis")
            mapping_digest = hashlib.sha256(canonical_json_bytes(mapping)).hexdigest()
            if row_record.get("mapping_sha256") != mapping_digest:
                fail("mapping_digest", f"{candidate} mapping {row_index} digest drifted")
            if mapping_digest in mapping_digests:
                fail("mapping_identity", "mapping identities must be globally unique across all 70 rows")
            mapping_digests.add(mapping_digest)
        if row_relationships != expected_relationships or len(set(row_relationships)) != len(row_relationships):
            fail("mapping_order", f"{candidate} mappings must be unique and ordered SR-01 through SR-14")
        if set(edge_map) != used_edges:
            fail("mapping_edges", f"{candidate} mappings must consume every native edge exactly once")

        graph_digest = hashlib.sha256(canonical_json_bytes(graph)).hexdigest()
        if graph_record.get("graph_sha256") != graph_digest:
            fail("graph_digest", f"{candidate} graph digest drifted")
        if graph_digest in graph_digests:
            fail("graph_identity", "candidate graph identities must be unique")
        graph_digests.add(graph_digest)

    if len(mapping_digests) != 70:
        fail("counts", "catalog must retain 70 globally unique mapping identities")


def validate_d004_result_contract_source(
    context: Any,
    *,
    result_contract_source: str,
    result_contract_source_sha256: str,
    decision_suite_source: str,
    descriptor_sha256: str,
) -> None:
    path = context.root / result_contract_source
    source_bytes = context._read_repository_bytes(path)
    if source_bytes is None:
        context.add(
            "d004_packet.result_contract_missing",
            path,
            "closed D-004 future-result schema descriptor source is missing",
        )
        return
    if hashlib.sha256(source_bytes).hexdigest() != result_contract_source_sha256:
        context.add(
            "d004_packet.result_contract_identity",
            path,
            "future-result descriptor source identity drifted",
        )
    try:
        source = source_bytes.decode("utf-8")
    except UnicodeDecodeError:
        context.add(
            "d004_packet.result_contract_source",
            path,
            "future-result descriptor source is not UTF-8",
        )
        return

    suite_path = context.root / decision_suite_source
    suite_bytes = context._read_repository_bytes(suite_path)
    suite_markers = (
        '#[path = "d004_support/candidate_mappings.rs"]',
        "mod candidate_mappings;",
        '#[path = "d004_support/result_contract.rs"]',
        "mod result_contract;",
        'const RESULT_CONTRACT_DESCRIPTOR_SHA256: &str =',
        f'"{descriptor_sha256}"',
        "canonical_draft_result_contract_descriptor_bytes",
        "parse_draft_result_contract_descriptor",
        "assert_eq!(oracle_inventory.len(), 73);",
        "assert_eq!(subject_ids.len(), 73);",
        "assert_eq!(CANDIDATE_MAPPING_CATALOG_PATH, INPUT_BINDINGS[22].path);",
        "candidate_mapping_catalog_is_exact_complete_and_input_only",
        "assert_eq!(plan.schedule().len(), REQUIRED_CANDIDATE_CASES);",
        '.get("schedule_slots")',
    )
    if suite_bytes is None:
        context.add(
            "d004_packet.result_contract_suite_binding",
            suite_path,
            "D-004 suite source is unavailable for descriptor binding",
        )
    else:
        try:
            suite_source = suite_bytes.decode("utf-8")
        except UnicodeDecodeError:
            suite_source = ""
        if any(marker not in suite_source for marker in suite_markers):
            context.add(
                "d004_packet.result_contract_suite_binding",
                suite_path,
                "D-004 suite no longer binds the exact descriptor module and digest",
            )

    schema_markers = (
        '"d004-result-contract-descriptor-v0.1-draft"',
        "pub(crate) const REQUIRED_CASE_RECORD_FIELDS: [&str; 31]",
        "pub(crate) const REQUIRED_POSITIVE_SUBJECT_BINDING_FIELDS: [&str; 2]",
        "pub(crate) const REQUIRED_MUTATION_SUBJECT_BINDING_FIELDS: [&str; 3]",
        "pub(crate) const REQUIRED_IDENTITY_FIELDS: [&str; 11]",
        "pub(crate) const REQUIRED_RESOURCE_CEILING_FIELDS: [&str; 4]",
        "pub(crate) const REQUIRED_MEASURED_RESOURCE_FIELDS: [&str; 5]",
        "pub(crate) const REQUIRED_EXECUTION_STATE_FIELDS: [&str; 6]",
        "pub(crate) const REQUIRED_OBSERVATION_FIELDS: [&str; 12]",
        "pub(crate) const REQUIRED_LOG_MANIFEST_FIELDS: [&str; 2]",
        "pub(crate) const REQUIRED_LOG_MANIFEST_ENTRY_FIELDS: [&str; 5]",
        "pub(crate) const REQUIRED_CONTEXT_INVENTORIES: [&str; 4]",
        "pub(crate) const REQUIRED_CANDIDATE_GRAPH_FIELDS: [&str; 3]",
        "pub(crate) const REQUIRED_GRAPH_NODE_FIELDS: [&str; 3]",
        "pub(crate) const REQUIRED_GRAPH_EDGE_FIELDS: [&str; 13]",
        "pub(crate) const REQUIRED_SR_MAP_FIELDS: [&str; 15]",
        "pub(crate) const REQUIRED_BYTE_MANIFEST_FIELDS: [&str; 2]",
        "pub(crate) const REQUIRED_BYTE_MANIFEST_ENTRY_FIELDS: [&str; 4]",
        "pub(crate) const REQUIRED_ENVIRONMENT_ENTRY_FIELDS: [&str; 2]",
        "pub(crate) const REQUIRED_CONTEXT_ENTRY_FIELDS: [&str; 4]",
        "pub(crate) const REQUIRED_UNSUPPORTED_FEATURE_FIELDS: [&str; 4]",
        "pub(crate) const REQUIRED_REPLAY_FIELDS: [&str; 6]",
        "pub(crate) const REQUIRED_OWNER_LABEL_FIELDS: [&str; 4]",
        "pub(crate) const REPLAY_NON_SUCCESS_STATES: [&str; 7]",
        "pub(crate) const EXECUTION_STATE_KINDS: [&str; 8]",
        'pub(crate) const ADAPTER_STATUS_STATES: [&str; 3] = '
        '["executed", "not_executed", "failed"]',
        "pub(crate) const OBSERVED_INVALIDATION_STATES: [&str; 3]",
        "pub(crate) const SR_APPLICABILITY_STATES: [&str; 2]",
        "pub(crate) const SR_CONFORMANCE_STATES: [&str; 4]",
        "pub(crate) const SCHEDULE_SLOT_FIELDS: [&str; 5]",
        "pub(crate) const SCHEDULED_SLOT_PREIMAGE_FIELDS: [&str; 10]",
        "pub(crate) const SUBJECT_ORACLE_FIELDS: [&str; 11]",
        "pub(crate) const CANDIDATE_MAPPING_IDENTITY_FIELDS: [&str; 4]",
        "pub(crate) const DIGEST_JOIN_FIELDS: [&str; 13]",
        "pub(crate) const ORDERING_RULES: [&str; 6]",
        "pub(crate) const OBSERVATION_COMPARISONS: [&str; 2]",
        "pub(crate) const DERIVATION_RULES: [&str; 31]",
        "pub(crate) const RESULT_CONTRACT_NONCLAIMS: [&str; 12]",
        "pub(crate) fn canonical_draft_result_contract_descriptor_bytes(",
        "pub(crate) fn parse_draft_result_contract_descriptor(",
        "let expected = descriptor_value(",
        "if value != expected",
        "Ok(DraftResultContractDescriptor {",
        "case_subject_catalog: &CaseSubjectCatalog",
        "fixture_catalog: &FixtureCatalog",
        "candidate_mapping_catalog: &CandidateMappingCatalog",
        "use super::cases::MUTATIONS;",
        '("mutation_inventory".to_owned(), mutation_inventory_value())',
        '"subject_catalog_bindings".to_owned()',
        '"candidate_mapping_identity_inventory".to_owned()',
        "candidate_mapping_identity_inventory_value(candidate_mapping_catalog)",
        '"subject_oracle_inventory".to_owned()',
        "subject_oracle_inventory_value(case_subject_catalog, fixture_catalog)",
        '("schedule_slots".to_owned(), schedule_slots_value())',
        '"scheduled_slot_identity_contract".to_owned()',
        "MUTATIONS",
        "Vec::with_capacity(REQUIRED_CANDIDATE_CASES)",
        "Vec::with_capacity(positives.len() + mutations.len() + fixtures.len())",
        "CANDIDATES[(round + position) % CANDIDATES.len()]",
        "CASES[(2 * round + position) % CASES.len()]",
        "fn pointer_token(value: &str) -> String",
        "value.replace('~', \"~0\").replace('/', \"~1\")",
        '"execution kind completed requires exit_code zero, signal null, '
        'adapter_status executed, and no truncated output"',
        '"replay resource_exhaustion is distinct from preregistered '
        'domain-level exhausted"',
        '"case_verdict is pass if and only if execution is validly completed, '
        'every resource bound holds, the frozen authenticated observation inventory '
        'is complete and matched, every required invalidation is satisfied, every '
        'digest and manifest join succeeds, all required SR rows are satisfied with '
        'valid dependencies, and the replay contract is satisfied"',
    )
    if any(marker not in source for marker in schema_markers):
        context.add(
            "d004_packet.result_contract_schema",
            path,
            "future-result descriptor schema or exact-only parser surface drifted",
        )

    zero_state_markers = (
        '("epoch".to_owned(), JsonValue::Null)',
        'string_entry("epoch_status", "unfrozen")',
        'string_entry("owner_protocol_review", "none")',
        '("replay_repetitions".to_owned(), JsonValue::Null)',
        'usize_entry("required_candidate_cases", REQUIRED_CANDIDATE_CASES)',
        'usize_entry("result_record_count", 0)',
        'usize_entry("completed_candidate_cases", 0)',
        'usize_entry("complete_candidates", 0)',
        'usize_entry("complete_cross_candidate_cases", 0)',
        'string_entry("evidence_status", "none")',
        '("selection".to_owned(), JsonValue::Null)',
        '("conclusion".to_owned(), JsonValue::Null)',
        'string_entry("roadmap_gate_credit", "none")',
        'string_entry("readiness_credit", "none")',
        'string_entry("status", "identity_plan_only")',
        'string_entry("physical_execution_order", "unassigned")',
        'string_entry("persistence", "none_before_frozen_epoch")',
        'string_entry("current_record_status", "absent")',
    )
    empty_record_arrays = (
        "result_records",
        "observation_records",
        "verdict_records",
        "review_records",
        "evidence_records",
    )
    empty_records = all(
        re.search(
            rf'"{re.escape(name)}"\.to_owned\(\),\s*'
            r'JsonValue::Array\(Vec::new\(\)\)\s*,?\s*\)',
            source,
        )
        is not None
        for name in empty_record_arrays
    )
    if any(marker not in source for marker in zero_state_markers) or not empty_records:
        context.add(
            "d004_packet.result_contract_zero_state",
            path,
            "descriptor weakened the exact unfrozen 0/25, no-evidence, no-credit state",
        )
    if any(
        marker in source
        for marker in (
            'fn readiness_percent(',
            '"readiness_percent"',
            '"roadmap_gates_closed"',
            '"roadmap_gates_total"',
        )
    ):
        context.add(
            "d004_packet.result_contract_readiness_scope",
            path,
            "D-004 descriptor must not restate global gate count or readiness percentage",
        )

    nonclaims = (
        "descriptor only; no populated case record exists",
        "candidate mappings are unreviewed input-only hypotheses; no candidate adapter exists",
        "no candidate process or tool invoked",
        "no result, observation, verdict, review, or evidence record accepted",
        "no replay repetition count assigned",
        "no D-004 evidence epoch frozen",
        "no candidate-case execution completed",
        "no semantic-strata candidate selected",
        "no D-004 disposition accepted",
        "no S3b implementation authorized",
        "no independent review claimed",
        "no roadmap gate or readiness movement",
    )
    if any(f'"{value}"' not in source for value in nonclaims):
        context.add(
            "d004_packet.result_contract_nonclaims",
            path,
            "future-result descriptor weakened its exact nonclaims",
        )

    if (
        '"allowed_domain_states"' not in source
        or (
            "observed_state is a member of the resolved oracle "
            "allowed_domain_states"
        )
        not in source
        or '"comparison is mismatched otherwise"' not in source
        or '"no replay repetition count assigned"' not in source
        or 'string_entry("capability_credit", "none")' not in source
        or (
            "observation_level, allowed_domain_states, required_invalidation, "
            "and capability_credit must equal the resolved oracle and cannot be "
            "result-defined or broadened"
        )
        not in source
    ):
        context.add(
            "d004_packet.result_contract_semantics",
            path,
            "oracle-bound observation semantics or unresolved replay cardinality drifted",
        )

    function_names = set(
        re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", source)
    )
    execution_prefixes = ("execute", "launch", "invoke", "spawn", "run_candidate")
    if (
        any(name.startswith(execution_prefixes) for name in function_names)
        or re.search(
            r"\b(?:std::process|Command::new|process::Command)\b", source
        )
    ):
        context.add(
            "d004_packet.result_contract_execution_api",
            path,
            "future-result descriptor must expose no candidate launcher or execution API",
        )
    persistence_prefixes = ("persist", "save", "store", "write", "capture", "emit")
    if (
        any(name.startswith(persistence_prefixes) for name in function_names)
        or re.search(
            r"\b(?:std::fs|std::io::Write|OpenOptions|File::create|fs::write)\b",
            source,
        )
    ):
        context.add(
            "d004_packet.result_contract_persistence_api",
            path,
            "future-result descriptor must expose no capture or persistence API",
        )
    parser_names = {name for name in function_names if name.startswith("parse")}
    if parser_names != {"parse_draft_result_contract_descriptor"}:
        context.add(
            "d004_packet.result_contract_parser_api",
            path,
            "only the exact closed-descriptor parser is permitted",
        )
