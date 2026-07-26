use std::collections::{BTreeMap, BTreeSet};

use super::domain::{CANDIDATES, RELATIONSHIPS};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

pub(crate) const CANDIDATE_MAPPING_CATALOG_PATH: &str =
    "research/decisions/D-004/d004-v0.5-candidate-mappings.json";
pub(crate) const CANDIDATE_MAPPING_CATALOG_CANONICAL_SHA256: &str =
    "c967d7db8ea5049da054129367ec61cd80d729b8ce8cd34c95a76e42c67c97b8";
pub(crate) const CANDIDATE_MAPPING_CATALOG_RAW_SHA256: &str =
    "70765c64936bbb8aafd6e101fbf20c85396eb722d70e55bb9311d14bfbb15156";
pub(crate) const CANDIDATE_MAPPING_CATALOG_SUBJECT_SHA256: &str =
    "e3b790857ee21a0c651995919aaadb9bf59b05367a8dce99bab6afb6e7d2543f";

pub(crate) const CANDIDATE_MAPPING_COUNT: usize = 5;
pub(crate) const CANDIDATE_MAPPING_RELATIONSHIP_COUNT: usize = 14;
pub(crate) const CANDIDATE_MAPPING_ROW_COUNT: usize = 70;

const DECISION_SUITE_RAW_SHA256: &str =
    "64abe8290955f889e28f8bb9ce7653a26ef71a624286aef900d4dbfc3b7eb117";

const ROOT_FIELDS: [&str; 14] = [
    "schema_version",
    "suite_version",
    "status",
    "epoch",
    "frozen",
    "owner_protocol_review",
    "canonicalization",
    "source_bindings",
    "open_decisions",
    "catalog_subject",
    "catalog_subject_sha256",
    "execution_boundary",
    "evidence_status",
    "nonclaims",
];
const SOURCE_BINDING_FIELDS: [&str; 1] = ["suite"];
const SUITE_BINDING_FIELDS: [&str; 2] = ["path", "raw_sha256"];
const OPEN_DECISION_FIELDS: [&str; 3] = ["id", "scope", "status"];
const CATALOG_SUBJECT_FIELDS: [&str; 5] = [
    "candidate_count",
    "relationship_count",
    "mapping_row_count",
    "required_relationships",
    "candidate_graphs",
];
const RELATIONSHIP_FIELDS: [&str; 12] = [
    "id",
    "crossing",
    "direction",
    "domain",
    "codomain",
    "definedness",
    "obligations",
    "identity_inputs",
    "trust_role",
    "failure_behavior",
    "prohibited_reverse_inferences",
    "observation",
];
const CANDIDATE_GRAPH_RECORD_FIELDS: [&str; 3] = ["candidate", "graph", "graph_sha256"];
const GRAPH_FIELDS: [&str; 5] = ["architecture", "candidate", "nodes", "edges", "sr_rows"];
const NODE_FIELDS: [&str; 6] = ["id", "role", "member_kind", "authority", "parent", "facets"];
const EDGE_RECORD_FIELDS: [&str; 2] = ["edge_subject", "edge_sha256"];
const EDGE_SUBJECT_FIELDS: [&str; 16] = [
    "id",
    "relationship",
    "draft_status",
    "conformance_status",
    "direction",
    "domain_endpoint",
    "codomain_endpoints",
    "definedness",
    "obligations",
    "identity_inputs",
    "trust_role",
    "failure_behavior",
    "prohibited_reverse_inferences",
    "observation_requirement",
    "parameter_slots",
    "delegation_boundary",
];
const ENDPOINT_FIELDS: [&str; 2] = ["node", "facet"];
const DELEGATION_FIELDS: [&str; 3] = [
    "selector_parameter",
    "host_identity_parameter",
    "non_success_behavior",
];
const ROW_FIELDS: [&str; 2] = ["mapping", "mapping_sha256"];
const MAPPING_FIELDS: [&str; 8] = [
    "candidate",
    "relationship",
    "required_relationship_sha256",
    "fused_authority",
    "native_edges",
    "mapping_form",
    "inspectability",
    "draft_hypothesis",
];
const EXECUTION_BOUNDARY_FIELDS: [&str; 5] = [
    "adapter",
    "command",
    "network",
    "output_persistence",
    "process",
];

pub(crate) const CANDIDATE_MAPPING_NONCLAIMS: [&str; 10] = [
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
];

const MAPPING_FORMS: [&str; 4] = ["direct", "fused", "split", "delegated"];
const NODE_MEMBER_KINDS: [&str; 9] = [
    "source_declaration",
    "semantic_domain",
    "semantic_view",
    "shared_subset",
    "evidence_interface",
    "claim_boundary",
    "judgment_boundary",
    "host_delegated_domain",
    "reference_set",
];
const NODE_AUTHORITIES: [&str; 5] = [
    "input_only",
    "candidate_local",
    "later_decision_boundary",
    "unresolved_checker_boundary",
    "host_delegated",
];
const HOST_LOCAL_RELATIONSHIP_INDICES: [usize; 8] = [0, 1, 2, 5, 6, 8, 9, 13];

const OPEN_DECISIONS: [(&str, &str); 8] = [
    ("D-005", "claim model"),
    ("D-006", "proof foundation"),
    ("D-007", "proof format and checker"),
    ("D-009", "solver trust policy"),
    ("D-010", "compiler-pass strategy"),
    ("D-011", "host target ISA and object-format envelope"),
    ("D-012", "leakage observations and declassification policy"),
    ("D-013", "ABI"),
];

const FORBIDDEN_PERSISTED_FIELDS: [&str; 18] = [
    "adapter_argv",
    "adapter_output",
    "adapter_path",
    "candidate_execution",
    "candidate_observation",
    "capability_credit",
    "case_result",
    "case_verdict",
    "environment",
    "evidence",
    "execution_result",
    "loader_status",
    "matched",
    "observed_invalidation",
    "observed_state",
    "readiness_credit",
    "result",
    "verdict",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateMappingIdentity {
    pub(crate) candidate: String,
    pub(crate) graph_sha256: String,
    pub(crate) sr_map_sha256: String,
    pub(crate) mapping_sha256: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NodeTopology {
    facets: BTreeSet<String>,
    parent: Option<String>,
    member_kind: String,
    authority: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NativeEdgeIdentity {
    relationship: String,
    edge_sha256: String,
    endpoint_nodes: BTreeSet<String>,
}

struct NativeEdgeValidationContext<'a> {
    candidate: &'a str,
    mapping_form: &'a str,
    expected_relationship: &'a str,
    mapping_path: &'a str,
    available_edges: &'a BTreeMap<String, NativeEdgeIdentity>,
    nodes: &'a BTreeMap<String, NodeTopology>,
    used_edges: &'a mut BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CandidateMappingErrorKind {
    Json(JsonErrorKind),
    MissingField,
    UnknownField,
    InvalidValue,
    NonCanonicalEncoding,
    RawDigest,
    CatalogDigest,
    CatalogSubjectDigest,
    GraphDigest,
    EdgeDigest,
    MappingDigest,
    SourceBinding,
    DuplicateIdentity,
    UnresolvedEdge,
    StructuralMismatch,
    PersistedResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateMappingError {
    pub(crate) kind: CandidateMappingErrorKind,
    pub(crate) path: String,
}

impl CandidateMappingError {
    fn new(kind: CandidateMappingErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CandidateMappingCatalog {
    value: JsonValue,
    canonical: Vec<u8>,
    digest: [u8; 32],
    subject_digest: [u8; 32],
    identities: Vec<CandidateMappingIdentity>,
}

impl CandidateMappingCatalog {
    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&self.digest)
    }

    pub(crate) fn subject_digest_hex(&self) -> String {
        sha256::hex(&self.subject_digest)
    }

    pub(crate) fn identities(&self) -> &[CandidateMappingIdentity] {
        &self.identities
    }
}

pub(crate) fn parse_candidate_mapping_catalog(
    input: &[u8],
) -> Result<CandidateMappingCatalog, CandidateMappingError> {
    let value = strict_json::parse(input).map_err(|error| {
        CandidateMappingError::new(
            CandidateMappingErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })?;
    let canonical = strict_json::canonical_bytes(&value);
    if input != canonical_file_bytes(canonical.clone()) {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::NonCanonicalEncoding,
            "$",
        ));
    }
    reject_persisted_fields(&value, "$")?;
    let (subject_digest, identities) = validate_catalog(&value)?;

    let digest = sha256::digest(&canonical);
    if sha256::hex(&digest) != CANDIDATE_MAPPING_CATALOG_CANONICAL_SHA256 {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::CatalogDigest,
            "$",
        ));
    }
    if sha256::hex(&sha256::digest(input)) != CANDIDATE_MAPPING_CATALOG_RAW_SHA256 {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::RawDigest,
            "$",
        ));
    }

    Ok(CandidateMappingCatalog {
        value,
        canonical,
        digest,
        subject_digest,
        identities,
    })
}

fn validate_catalog(
    value: &JsonValue,
) -> Result<([u8; 32], Vec<CandidateMappingIdentity>), CandidateMappingError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &ROOT_FIELDS, "$")?;
    require_exact_string(
        root,
        "schema_version",
        "d004-candidate-mapping-catalog-v0.1",
        "$",
    )?;
    require_exact_string(root, "suite_version", "d004-v0.5-draft", "$")?;
    require_exact_string(root, "status", "draft_unreviewed_input_only", "$")?;
    require_null(root, "epoch", "$")?;
    require_exact_bool(root, "frozen", false, "$")?;
    require_exact_string(root, "owner_protocol_review", "none", "$")?;
    require_exact_string(
        root,
        "canonicalization",
        "RFC8785_ASCII_INTEGER_SUBSET",
        "$",
    )?;
    require_exact_string(root, "evidence_status", "none", "$")?;
    require_exact_strings(root, "nonclaims", &CANDIDATE_MAPPING_NONCLAIMS, "$")?;
    validate_source_bindings(root)?;
    validate_open_decisions(root)?;
    validate_execution_boundary(root)?;

    let subject_path = "$/catalog_subject";
    let subject_value = require_field(root, "catalog_subject", "$")?;
    let subject = require_object(subject_value, subject_path)?;
    exact_fields(subject, &CATALOG_SUBJECT_FIELDS, subject_path)?;
    require_exact_usize(
        subject,
        "candidate_count",
        CANDIDATE_MAPPING_COUNT,
        subject_path,
    )?;
    require_exact_usize(
        subject,
        "relationship_count",
        CANDIDATE_MAPPING_RELATIONSHIP_COUNT,
        subject_path,
    )?;
    require_exact_usize(
        subject,
        "mapping_row_count",
        CANDIDATE_MAPPING_ROW_COUNT,
        subject_path,
    )?;

    let relationships = require_array(subject, "required_relationships", "$/catalog_subject")?;
    let relationship_digests = validate_required_relationships(relationships)?;
    let identities = validate_candidate_graphs(subject, relationships, &relationship_digests)?;

    let subject_digest = sha256::digest(&strict_json::canonical_bytes(subject_value));
    let observed_subject_digest = require_string(root, "catalog_subject_sha256", "$")?;
    if observed_subject_digest != sha256::hex(&subject_digest)
        || observed_subject_digest != CANDIDATE_MAPPING_CATALOG_SUBJECT_SHA256
    {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::CatalogSubjectDigest,
            "$/catalog_subject_sha256",
        ));
    }
    Ok((subject_digest, identities))
}

fn validate_source_bindings(
    root: &BTreeMap<String, JsonValue>,
) -> Result<(), CandidateMappingError> {
    let path = "$/source_bindings";
    let bindings = require_object(require_field(root, "source_bindings", "$")?, path)?;
    exact_fields(bindings, &SOURCE_BINDING_FIELDS, path)?;
    let suite_path = "$/source_bindings/suite";
    let suite = require_object(require_field(bindings, "suite", path)?, suite_path)?;
    exact_fields(suite, &SUITE_BINDING_FIELDS, suite_path)?;
    require_exact_string(
        suite,
        "path",
        "docs/SEMANTIC_STRATA_DECISION_SUITE.md",
        suite_path,
    )
    .map_err(|_| {
        CandidateMappingError::new(
            CandidateMappingErrorKind::SourceBinding,
            "$/source_bindings/suite/path",
        )
    })?;
    require_exact_string(suite, "raw_sha256", DECISION_SUITE_RAW_SHA256, suite_path).map_err(|_| {
        CandidateMappingError::new(
            CandidateMappingErrorKind::SourceBinding,
            "$/source_bindings/suite/raw_sha256",
        )
    })
}

fn validate_execution_boundary(
    root: &BTreeMap<String, JsonValue>,
) -> Result<(), CandidateMappingError> {
    let path = "$/execution_boundary";
    let boundary = require_object(require_field(root, "execution_boundary", "$")?, path)?;
    exact_fields(boundary, &EXECUTION_BOUNDARY_FIELDS, path)?;
    for (field, expected) in [
        ("adapter", "absent"),
        ("command", "absent"),
        ("network", "not_used"),
        ("output_persistence", "none"),
        ("process", "not_invoked"),
    ] {
        require_exact_string(boundary, field, expected, path)?;
    }
    Ok(())
}

fn validate_open_decisions(
    root: &BTreeMap<String, JsonValue>,
) -> Result<(), CandidateMappingError> {
    let path = "$/open_decisions";
    let decisions = require_array(root, "open_decisions", "$")?;
    if decisions.len() != OPEN_DECISIONS.len() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            path,
        ));
    }
    for (index, (value, (id, scope))) in decisions.iter().zip(OPEN_DECISIONS).enumerate() {
        let record_path = format!("{path}/{index}");
        let record = require_object(value, &record_path)?;
        exact_fields(record, &OPEN_DECISION_FIELDS, &record_path)?;
        require_exact_string(record, "id", id, &record_path)?;
        require_exact_string(record, "scope", scope, &record_path)?;
        require_exact_string(record, "status", "unselected", &record_path)?;
    }
    Ok(())
}

fn validate_required_relationships(
    relationships: &[JsonValue],
) -> Result<Vec<String>, CandidateMappingError> {
    let path = "$/catalog_subject/required_relationships";
    if relationships.len() != RELATIONSHIPS.len() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            path,
        ));
    }
    let mut ids = BTreeSet::new();
    let mut digests = Vec::with_capacity(CANDIDATE_MAPPING_RELATIONSHIP_COUNT);
    for (index, (value, expected_id)) in relationships.iter().zip(RELATIONSHIPS).enumerate() {
        let record_path = format!("{path}/{index}");
        let record = require_object(value, &record_path)?;
        exact_fields(record, &RELATIONSHIP_FIELDS, &record_path)?;
        require_exact_string(record, "id", expected_id, &record_path)?;
        insert_unique(&mut ids, expected_id, &format!("{record_path}/id"))?;
        for field in RELATIONSHIP_FIELDS
            .into_iter()
            .filter(|field| *field != "id")
        {
            require_nonempty_string(record, field, &record_path)?;
        }
        digests.push(sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
            value,
        ))));
    }
    Ok(digests)
}

fn validate_candidate_graphs(
    subject: &BTreeMap<String, JsonValue>,
    relationships: &[JsonValue],
    relationship_digests: &[String],
) -> Result<Vec<CandidateMappingIdentity>, CandidateMappingError> {
    let path = "$/catalog_subject/candidate_graphs";
    let graphs = require_array(subject, "candidate_graphs", "$/catalog_subject")?;
    if graphs.len() != CANDIDATES.len() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            path,
        ));
    }
    let mut candidates = BTreeSet::new();
    let mut graph_digests = BTreeSet::new();
    let mut mapping_digests = BTreeSet::new();
    let mut identities = Vec::with_capacity(CANDIDATE_MAPPING_COUNT);

    for (index, (value, expected_candidate)) in graphs.iter().zip(CANDIDATES).enumerate() {
        let record_path = format!("{path}/{index}");
        let record = require_object(value, &record_path)?;
        exact_fields(record, &CANDIDATE_GRAPH_RECORD_FIELDS, &record_path)?;
        let candidate = expected_candidate.as_str();
        require_exact_string(record, "candidate", candidate, &record_path)?;
        insert_unique(
            &mut candidates,
            candidate,
            &format!("{record_path}/candidate"),
        )?;

        let graph_path = format!("{record_path}/graph");
        let graph_value = require_field(record, "graph", &record_path)?;
        let graph = require_object(graph_value, &graph_path)?;
        exact_fields(graph, &GRAPH_FIELDS, &graph_path)?;
        require_exact_string(graph, "candidate", candidate, &graph_path)?;
        require_nonempty_string(graph, "architecture", &graph_path)?;
        let nodes = validate_nodes(graph, candidate, &graph_path)?;
        let available_edges = validate_edges(graph, candidate, &graph_path, &nodes, relationships)?;
        let (row_digests, used_edges) = validate_mapping_rows(
            graph,
            candidate,
            &graph_path,
            relationship_digests,
            &available_edges,
            &nodes,
            &mut mapping_digests,
        )?;
        let available_edge_ids = available_edges.keys().cloned().collect::<BTreeSet<_>>();
        if available_edge_ids != used_edges {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::UnresolvedEdge,
                format!("{graph_path}/edges"),
            ));
        }

        let graph_digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(graph_value)));
        let sr_map_sha256 = candidate_sr_map_digest_hex(graph_value, &graph_path)?;
        if require_string(record, "graph_sha256", &record_path)? != graph_digest {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::GraphDigest,
                format!("{record_path}/graph_sha256"),
            ));
        }
        insert_unique(
            &mut graph_digests,
            &graph_digest,
            &format!("{record_path}/graph_sha256"),
        )?;
        identities.push(CandidateMappingIdentity {
            candidate: candidate.to_owned(),
            graph_sha256: graph_digest,
            sr_map_sha256,
            mapping_sha256: row_digests,
        });
    }
    if mapping_digests.len() != CANDIDATE_MAPPING_ROW_COUNT {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            "$/catalog_subject/mapping_row_count",
        ));
    }
    Ok(identities)
}

fn candidate_sr_map_digest_hex(
    graph_value: &JsonValue,
    graph_path: &str,
) -> Result<String, CandidateMappingError> {
    let graph = require_object(graph_value, graph_path)?;
    let rows = require_field(graph, "sr_rows", graph_path)?;
    if rows.as_array().is_none() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            format!("{graph_path}/sr_rows"),
        ));
    }
    Ok(sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
        rows,
    ))))
}

fn validate_nodes(
    graph: &BTreeMap<String, JsonValue>,
    candidate: &str,
    graph_path: &str,
) -> Result<BTreeMap<String, NodeTopology>, CandidateMappingError> {
    let path = format!("{graph_path}/nodes");
    let nodes = require_array(graph, "nodes", graph_path)?;
    if nodes.is_empty() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            path,
        ));
    }
    let expected_prefix = candidate
        .strip_prefix("ST-")
        .unwrap_or(candidate)
        .to_ascii_lowercase();
    let mut topology = BTreeMap::new();
    for (index, value) in nodes.iter().enumerate() {
        let node_path = format!("{path}/{index}");
        let node = require_object(value, &node_path)?;
        exact_fields(node, &NODE_FIELDS, &node_path)?;
        let id = require_nonempty_string(node, "id", &node_path)?;
        require_nonempty_string(node, "role", &node_path)?;
        if !id.starts_with(&format!("{expected_prefix}-")) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::StructuralMismatch,
                format!("{node_path}/id"),
            ));
        }
        let member_kind = require_string(node, "member_kind", &node_path)?;
        if !NODE_MEMBER_KINDS.contains(&member_kind) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{node_path}/member_kind"),
            ));
        }
        let authority = require_string(node, "authority", &node_path)?;
        if !NODE_AUTHORITIES.contains(&authority) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{node_path}/authority"),
            ));
        }
        let facets = nonempty_unique_strings(node, "facets", &node_path)?;
        let parent = match require_field(node, "parent", &node_path)? {
            JsonValue::Null => None,
            JsonValue::String(parent) if !parent.is_empty() => Some(parent.clone()),
            _ => {
                return Err(CandidateMappingError::new(
                    CandidateMappingErrorKind::InvalidValue,
                    format!("{node_path}/parent"),
                ));
            }
        };
        if topology
            .insert(
                id.to_owned(),
                NodeTopology {
                    facets,
                    parent,
                    member_kind: member_kind.to_owned(),
                    authority: authority.to_owned(),
                },
            )
            .is_some()
        {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::DuplicateIdentity,
                format!("{node_path}/id"),
            ));
        }
    }
    for (id, node) in &topology {
        if let Some(parent) = &node.parent
            && (parent == id || !topology.contains_key(parent))
        {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::UnresolvedEdge,
                format!("{path}/{id}/parent"),
            ));
        }
    }
    validate_candidate_topology(candidate, &topology, &path)?;
    Ok(topology)
}

fn validate_candidate_topology(
    candidate: &str,
    topology: &BTreeMap<String, NodeTopology>,
    path: &str,
) -> Result<(), CandidateMappingError> {
    let required_parents: &[(&str, &str)] = match candidate {
        "ST-REL" => &[("rel-shared-pure", "rel-spec-core")],
        "ST-UNI" => &[
            ("uni-pure-view", "uni-calculus"),
            ("uni-impl-view", "uni-calculus"),
            ("uni-game-view", "uni-calculus"),
            ("uni-machine-view", "uni-calculus"),
            ("uni-proof-interface", "uni-calculus"),
        ],
        "ST-DUAL" => &[
            ("dual-game-view", "dual-effect-core"),
            ("dual-ct-view", "dual-effect-core"),
            ("dual-machine-view", "dual-effect-core"),
            ("dual-proof-interface", "dual-effect-core"),
        ],
        "ST-MIRROR" => &[("mirror-shared-pure", "mirror-spec-core")],
        "ST-HOST" => &[("host-judgment", "host-proof-domain")],
        _ => &[],
    };
    for (node, parent) in required_parents {
        if topology.get(*node).and_then(|node| node.parent.as_deref()) != Some(*parent) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::StructuralMismatch,
                format!("{path}/{node}/parent"),
            ));
        }
    }
    if candidate == "ST-MIRROR"
        && topology
            .get("mirror-proof-core")
            .map(|node| node.member_kind.as_str())
            != Some("semantic_domain")
    {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            format!("{path}/mirror-proof-core/member_kind"),
        ));
    }
    Ok(())
}

fn validate_edges(
    graph: &BTreeMap<String, JsonValue>,
    candidate: &str,
    graph_path: &str,
    nodes: &BTreeMap<String, NodeTopology>,
    relationships: &[JsonValue],
) -> Result<BTreeMap<String, NativeEdgeIdentity>, CandidateMappingError> {
    let path = format!("{graph_path}/edges");
    let edges = require_array(graph, "edges", graph_path)?;
    if edges.len() != RELATIONSHIPS.len() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            path,
        ));
    }
    let expected_prefix = candidate
        .strip_prefix("ST-")
        .unwrap_or(candidate)
        .to_ascii_lowercase();
    let mut identities = BTreeMap::new();
    let mut digests = BTreeSet::new();
    let mut referenced_nodes = BTreeSet::new();

    for (index, ((value, expected_relationship), relationship_value)) in edges
        .iter()
        .zip(RELATIONSHIPS)
        .zip(relationships)
        .enumerate()
    {
        let record_path = format!("{path}/{index}");
        let record = require_object(value, &record_path)?;
        exact_fields(record, &EDGE_RECORD_FIELDS, &record_path)?;
        let subject_path = format!("{record_path}/edge_subject");
        let subject_value = require_field(record, "edge_subject", &record_path)?;
        let subject = require_object(subject_value, &subject_path)?;
        exact_fields(subject, &EDGE_SUBJECT_FIELDS, &subject_path)?;
        let id = require_nonempty_string(subject, "id", &subject_path)?;
        if !id.starts_with(&format!("{expected_prefix}-")) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::UnresolvedEdge,
                format!("{subject_path}/id"),
            ));
        }
        if identities.contains_key(id) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::DuplicateIdentity,
                format!("{subject_path}/id"),
            ));
        }
        require_exact_string(
            subject,
            "relationship",
            expected_relationship,
            &subject_path,
        )?;
        require_exact_string(
            subject,
            "draft_status",
            "draft_unreviewed_unfrozen_hypothesis",
            &subject_path,
        )?;
        require_exact_string(subject, "conformance_status", "unresolved", &subject_path)?;

        let relationship = require_object(
            relationship_value,
            &format!("$/catalog_subject/required_relationships/{index}"),
        )?;
        for (edge_field, relationship_field) in [
            ("direction", "direction"),
            ("definedness", "definedness"),
            ("obligations", "obligations"),
            ("identity_inputs", "identity_inputs"),
            ("trust_role", "trust_role"),
            ("failure_behavior", "failure_behavior"),
            (
                "prohibited_reverse_inferences",
                "prohibited_reverse_inferences",
            ),
            ("observation_requirement", "observation"),
        ] {
            if require_field(subject, edge_field, &subject_path)?
                != require_field(relationship, relationship_field, "$required_relationship")?
            {
                return Err(CandidateMappingError::new(
                    CandidateMappingErrorKind::StructuralMismatch,
                    format!("{subject_path}/{edge_field}"),
                ));
            }
        }

        let mut edge_endpoint_nodes = BTreeSet::new();
        validate_endpoint(
            require_field(subject, "domain_endpoint", &subject_path)?,
            nodes,
            &format!("{subject_path}/domain_endpoint"),
            &mut edge_endpoint_nodes,
        )?;
        let codomain_path = format!("{subject_path}/codomain_endpoints");
        let codomains = require_array(subject, "codomain_endpoints", &subject_path)?;
        if codomains.is_empty() {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                codomain_path,
            ));
        }
        for (endpoint_index, endpoint) in codomains.iter().enumerate() {
            validate_endpoint(
                endpoint,
                nodes,
                &format!("{codomain_path}/{endpoint_index}"),
                &mut edge_endpoint_nodes,
            )?;
        }
        if index == 13 {
            let expected_node = format!("{expected_prefix}-eligible-reference-set");
            if codomains.len() != 1 {
                return Err(CandidateMappingError::new(
                    CandidateMappingErrorKind::StructuralMismatch,
                    &codomain_path,
                ));
            }
            let endpoint = require_object(&codomains[0], &codomain_path)?;
            require_exact_string(endpoint, "node", &expected_node, &codomain_path)?;
            require_exact_string(
                endpoint,
                "facet",
                "candidate_wide_subject_or_evidence_reference",
                &codomain_path,
            )?;
        }
        referenced_nodes.extend(edge_endpoint_nodes.iter().cloned());
        validate_nonempty_unique_string_array(subject, "parameter_slots", &subject_path)?;
        validate_delegation_boundary(subject, candidate, index, &subject_path)?;

        let digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
            subject_value,
        )));
        if require_string(record, "edge_sha256", &record_path)? != digest {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::EdgeDigest,
                format!("{record_path}/edge_sha256"),
            ));
        }
        insert_unique(&mut digests, &digest, &format!("{record_path}/edge_sha256"))?;
        identities.insert(
            id.to_owned(),
            NativeEdgeIdentity {
                relationship: expected_relationship.to_owned(),
                edge_sha256: digest,
                endpoint_nodes: edge_endpoint_nodes,
            },
        );
    }
    let parent_nodes = nodes
        .values()
        .filter_map(|node| node.parent.as_ref())
        .cloned()
        .collect::<BTreeSet<_>>();
    if let Some(orphan) = nodes
        .keys()
        .find(|id| !referenced_nodes.contains(*id) && !parent_nodes.contains(*id))
    {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::UnresolvedEdge,
            format!("{graph_path}/nodes/{orphan}"),
        ));
    }
    Ok(identities)
}

fn validate_endpoint(
    value: &JsonValue,
    nodes: &BTreeMap<String, NodeTopology>,
    path: &str,
    referenced_nodes: &mut BTreeSet<String>,
) -> Result<(), CandidateMappingError> {
    let endpoint = require_object(value, path)?;
    exact_fields(endpoint, &ENDPOINT_FIELDS, path)?;
    let node = require_nonempty_string(endpoint, "node", path)?;
    let topology = nodes.get(node).ok_or_else(|| {
        CandidateMappingError::new(
            CandidateMappingErrorKind::UnresolvedEdge,
            format!("{path}/node"),
        )
    })?;
    let facet = require_nonempty_string(endpoint, "facet", path)?;
    if !topology.facets.contains(facet) {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::UnresolvedEdge,
            format!("{path}/facet"),
        ));
    }
    referenced_nodes.insert(node.to_owned());
    Ok(())
}

fn validate_delegation_boundary(
    subject: &BTreeMap<String, JsonValue>,
    candidate: &str,
    relationship_index: usize,
    subject_path: &str,
) -> Result<(), CandidateMappingError> {
    let path = format!("{subject_path}/delegation_boundary");
    let value = require_field(subject, "delegation_boundary", subject_path)?;
    if candidate == "ST-HOST" && relationship_index == 2 {
        let slots = require_array(subject, "parameter_slots", subject_path)?;
        for forbidden in [
            "delegation_selector",
            "host_identity",
            "host_non_success_policy",
        ] {
            if slots.iter().any(|slot| slot.as_str() == Some(forbidden)) {
                return Err(CandidateMappingError::new(
                    CandidateMappingErrorKind::StructuralMismatch,
                    format!("{subject_path}/parameter_slots"),
                ));
            }
        }
    }
    let delegation_domain = if candidate == "ST-HOST" {
        host_delegation_domain(relationship_index)
    } else {
        None
    };
    let delegated = delegation_domain.is_some();
    if !delegated {
        if !matches!(value, JsonValue::Null) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                path,
            ));
        }
        return Ok(());
    }

    let boundary = require_object(value, &path)?;
    exact_fields(boundary, &DELEGATION_FIELDS, &path)?;
    let domain = delegation_domain.expect("delegated branch");
    let selector = format!("{domain}_delegation_selector");
    let identity = format!("{domain}_host_identity");
    let policy = format!("{domain}_non_success_policy");
    for (field, expected) in [
        ("selector_parameter", selector.as_str()),
        ("host_identity_parameter", identity.as_str()),
        (
            "non_success_behavior",
            "reject_or_unsupported_without_fallback_claim",
        ),
    ] {
        require_exact_string(boundary, field, expected, &path)?;
    }
    let slots = require_array(subject, "parameter_slots", subject_path)?;
    for required in [selector.as_str(), identity.as_str(), policy.as_str()] {
        if !slots.iter().any(|slot| slot.as_str() == Some(required)) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::StructuralMismatch,
                format!("{subject_path}/parameter_slots"),
            ));
        }
    }
    Ok(())
}

fn host_delegation_domain(relationship_index: usize) -> Option<&'static str> {
    match relationship_index {
        3 | 7 | 11 => Some("game"),
        4 | 12 => Some("proof"),
        10 => Some("machine"),
        _ => None,
    }
}

fn validate_mapping_rows(
    graph: &BTreeMap<String, JsonValue>,
    candidate: &str,
    graph_path: &str,
    relationship_digests: &[String],
    available_edges: &BTreeMap<String, NativeEdgeIdentity>,
    nodes: &BTreeMap<String, NodeTopology>,
    catalog_mapping_digests: &mut BTreeSet<String>,
) -> Result<(Vec<String>, BTreeSet<String>), CandidateMappingError> {
    let path = format!("{graph_path}/sr_rows");
    let rows = require_array(graph, "sr_rows", graph_path)?;
    if rows.len() != RELATIONSHIPS.len() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            path,
        ));
    }
    let mut relationship_ids = BTreeSet::new();
    let mut used_edges = BTreeSet::new();
    let mut row_digests = Vec::with_capacity(CANDIDATE_MAPPING_RELATIONSHIP_COUNT);

    for (index, (value, expected_relationship)) in rows.iter().zip(RELATIONSHIPS).enumerate() {
        let row_path = format!("{path}/{index}");
        let row = require_object(value, &row_path)?;
        exact_fields(row, &ROW_FIELDS, &row_path)?;
        let mapping_path = format!("{row_path}/mapping");
        let mapping_value = require_field(row, "mapping", &row_path)?;
        let mapping = require_object(mapping_value, &mapping_path)?;
        exact_fields(mapping, &MAPPING_FIELDS, &mapping_path)?;
        require_exact_string(mapping, "candidate", candidate, &mapping_path)?;
        require_exact_string(
            mapping,
            "relationship",
            expected_relationship,
            &mapping_path,
        )?;
        require_exact_string(
            mapping,
            "required_relationship_sha256",
            &relationship_digests[index],
            &mapping_path,
        )?;
        insert_unique(
            &mut relationship_ids,
            expected_relationship,
            &format!("{mapping_path}/relationship"),
        )?;
        require_exact_string(
            mapping,
            "inspectability",
            "separately_inspectable_and_falsifiable",
            &mapping_path,
        )?;
        let form = require_string(mapping, "mapping_form", &mapping_path)?;
        if !MAPPING_FORMS.contains(&form) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{mapping_path}/mapping_form"),
            ));
        }
        let expected_delegated =
            candidate == "ST-HOST" && !HOST_LOCAL_RELATIONSHIP_INDICES.contains(&index);
        if (form == "delegated") != expected_delegated {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::StructuralMismatch,
                format!("{mapping_path}/mapping_form"),
            ));
        }
        validate_native_edges(
            mapping,
            &mut NativeEdgeValidationContext {
                candidate,
                mapping_form: form,
                expected_relationship,
                mapping_path: &mapping_path,
                available_edges,
                nodes,
                used_edges: &mut used_edges,
            },
        )?;
        let hypothesis = require_nonempty_string(mapping, "draft_hypothesis", &mapping_path)?;
        let required_prefix = format!("Draft hypothesis: {candidate} ");
        if !hypothesis.starts_with(&required_prefix)
            || !hypothesis.contains(expected_relationship)
            || !hypothesis.contains("unreviewed")
            || !hypothesis.contains("unfrozen")
        {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{mapping_path}/draft_hypothesis"),
            ));
        }

        let digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
            mapping_value,
        )));
        if require_string(row, "mapping_sha256", &row_path)? != digest {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::MappingDigest,
                format!("{row_path}/mapping_sha256"),
            ));
        }
        insert_unique(
            catalog_mapping_digests,
            &digest,
            &format!("{row_path}/mapping_sha256"),
        )?;
        row_digests.push(digest);
    }
    Ok((row_digests, used_edges))
}

fn validate_native_edges(
    mapping: &BTreeMap<String, JsonValue>,
    context: &mut NativeEdgeValidationContext<'_>,
) -> Result<(), CandidateMappingError> {
    let candidate = context.candidate;
    let mapping_form = context.mapping_form;
    let expected_relationship = context.expected_relationship;
    let mapping_path = context.mapping_path;
    let available_edges = context.available_edges;
    let nodes = context.nodes;
    let path = format!("{mapping_path}/native_edges");
    let edges = require_array(mapping, "native_edges", mapping_path)?;
    if edges.is_empty() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            path,
        ));
    }
    let valid_cardinality = if mapping_form == "split" {
        edges.len() >= 2
    } else {
        edges.len() == 1
    };
    if !valid_cardinality {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::StructuralMismatch,
            path,
        ));
    }
    let fused_authority = match (
        mapping_form,
        require_field(mapping, "fused_authority", mapping_path)?,
    ) {
        ("fused", JsonValue::String(node)) if nodes.contains_key(node) => Some(node.as_str()),
        ("fused", _) => {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::StructuralMismatch,
                format!("{mapping_path}/fused_authority"),
            ));
        }
        (_, JsonValue::Null) => None,
        _ => {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::StructuralMismatch,
                format!("{mapping_path}/fused_authority"),
            ));
        }
    };
    let mut observed = BTreeSet::new();
    let expected_prefix = candidate
        .strip_prefix("ST-")
        .unwrap_or(candidate)
        .to_ascii_lowercase();
    for (index, edge) in edges.iter().enumerate() {
        let edge = edge.as_str().ok_or_else(|| {
            CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{path}/{index}"),
            )
        })?;
        if edge.is_empty() {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{path}/{index}"),
            ));
        }
        insert_unique(&mut observed, edge, &format!("{path}/{index}"))?;
        let identity = available_edges.get(edge).ok_or_else(|| {
            CandidateMappingError::new(
                CandidateMappingErrorKind::UnresolvedEdge,
                format!("{path}/{index}"),
            )
        })?;
        if !edge.starts_with(&format!("{expected_prefix}-"))
            || identity.relationship != expected_relationship
            || identity.edge_sha256.is_empty()
        {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::UnresolvedEdge,
                format!("{path}/{index}"),
            ));
        }
        if let Some(authority) = fused_authority {
            let authority_node = nodes.get(authority).ok_or_else(|| {
                CandidateMappingError::new(
                    CandidateMappingErrorKind::UnresolvedEdge,
                    format!("{mapping_path}/fused_authority"),
                )
            })?;
            if authority_node.member_kind != "semantic_domain" {
                return Err(CandidateMappingError::new(
                    CandidateMappingErrorKind::StructuralMismatch,
                    format!("{mapping_path}/fused_authority"),
                ));
            }
            for endpoint in &identity.endpoint_nodes {
                let endpoint_node = &nodes[endpoint];
                if endpoint_node.member_kind != "source_declaration"
                    && endpoint != authority
                    && endpoint_node.parent.as_deref() != Some(authority)
                {
                    return Err(CandidateMappingError::new(
                        CandidateMappingErrorKind::StructuralMismatch,
                        format!("{path}/{index}"),
                    ));
                }
            }
        }
        if mapping_form == "delegated"
            && !identity.endpoint_nodes.iter().any(|node| {
                let endpoint = &nodes[node];
                endpoint.authority == "host_delegated"
                    || endpoint
                        .parent
                        .as_ref()
                        .and_then(|parent| nodes.get(parent))
                        .is_some_and(|parent| parent.authority == "host_delegated")
            })
        {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::StructuralMismatch,
                format!("{path}/{index}"),
            ));
        }
        insert_unique(context.used_edges, edge, &format!("{path}/{index}"))?;
    }
    Ok(())
}

fn reject_persisted_fields(value: &JsonValue, path: &str) -> Result<(), CandidateMappingError> {
    match value {
        JsonValue::Object(object) => {
            for (field, child) in object {
                let child_path = format!("{path}/{field}");
                if FORBIDDEN_PERSISTED_FIELDS.contains(&field.as_str()) {
                    return Err(CandidateMappingError::new(
                        CandidateMappingErrorKind::PersistedResult,
                        child_path,
                    ));
                }
                reject_persisted_fields(child, &child_path)?;
            }
        }
        JsonValue::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                reject_persisted_fields(child, &format!("{path}/{index}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn canonical_file_bytes(mut canonical: Vec<u8>) -> Vec<u8> {
    canonical.push(b'\n');
    canonical
}

fn require_object<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, CandidateMappingError> {
    value
        .as_object()
        .ok_or_else(|| CandidateMappingError::new(CandidateMappingErrorKind::InvalidValue, path))
}

fn require_array<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<&'a [JsonValue], CandidateMappingError> {
    let path = format!("{parent}/{field}");
    require_field(object, field, parent)?
        .as_array()
        .ok_or_else(|| CandidateMappingError::new(CandidateMappingErrorKind::InvalidValue, path))
}

fn require_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<&'a JsonValue, CandidateMappingError> {
    object.get(field).ok_or_else(|| {
        CandidateMappingError::new(
            CandidateMappingErrorKind::MissingField,
            format!("{parent}/{field}"),
        )
    })
}

fn exact_fields(
    object: &BTreeMap<String, JsonValue>,
    expected: &[&str],
    path: &str,
) -> Result<(), CandidateMappingError> {
    if let Some(field) = object
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::UnknownField,
            format!("{path}/{field}"),
        ));
    }
    for field in expected {
        if !object.contains_key(*field) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::MissingField,
                format!("{path}/{field}"),
            ));
        }
    }
    Ok(())
}

fn require_string<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<&'a str, CandidateMappingError> {
    let path = format!("{parent}/{field}");
    require_field(object, field, parent)?
        .as_str()
        .ok_or_else(|| CandidateMappingError::new(CandidateMappingErrorKind::InvalidValue, path))
}

fn require_nonempty_string<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<&'a str, CandidateMappingError> {
    let value = require_string(object, field, parent)?;
    if value.is_empty() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            format!("{parent}/{field}"),
        ));
    }
    Ok(value)
}

fn require_exact_string(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &str,
    parent: &str,
) -> Result<(), CandidateMappingError> {
    if require_string(object, field, parent)? != expected {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            format!("{parent}/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_strings(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &[&str],
    parent: &str,
) -> Result<(), CandidateMappingError> {
    let path = format!("{parent}/{field}");
    let observed = require_field(object, field, parent)?
        .as_array()
        .ok_or_else(|| {
            CandidateMappingError::new(CandidateMappingErrorKind::InvalidValue, &path)
        })?;
    if observed.len() != expected.len() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            path,
        ));
    }
    for (index, (value, expected)) in observed.iter().zip(expected).enumerate() {
        if value.as_str() != Some(*expected) {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{path}/{index}"),
            ));
        }
    }
    Ok(())
}

fn validate_nonempty_unique_string_array(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<(), CandidateMappingError> {
    nonempty_unique_strings(object, field, parent).map(|_| ())
}

fn nonempty_unique_strings(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<BTreeSet<String>, CandidateMappingError> {
    let path = format!("{parent}/{field}");
    let values = require_field(object, field, parent)?
        .as_array()
        .ok_or_else(|| {
            CandidateMappingError::new(CandidateMappingErrorKind::InvalidValue, &path)
        })?;
    if values.is_empty() {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            path,
        ));
    }
    let mut observed = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        let value = value.as_str().ok_or_else(|| {
            CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{path}/{index}"),
            )
        })?;
        if value.is_empty() {
            return Err(CandidateMappingError::new(
                CandidateMappingErrorKind::InvalidValue,
                format!("{path}/{index}"),
            ));
        }
        insert_unique(&mut observed, value, &format!("{path}/{index}"))?;
    }
    Ok(observed)
}

fn require_exact_usize(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: usize,
    parent: &str,
) -> Result<(), CandidateMappingError> {
    let path = format!("{parent}/{field}");
    let observed = require_field(object, field, parent)?
        .as_integer()
        .ok_or_else(|| {
            CandidateMappingError::new(CandidateMappingErrorKind::InvalidValue, &path)
        })?;
    if usize::try_from(observed).ok() != Some(expected) {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            path,
        ));
    }
    Ok(())
}

fn require_null(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<(), CandidateMappingError> {
    if !matches!(require_field(object, field, parent)?, JsonValue::Null) {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            format!("{parent}/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_bool(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: bool,
    parent: &str,
) -> Result<(), CandidateMappingError> {
    if !matches!(require_field(object, field, parent)?, JsonValue::Bool(value) if *value == expected)
    {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::InvalidValue,
            format!("{parent}/{field}"),
        ));
    }
    Ok(())
}

fn insert_unique(
    values: &mut BTreeSet<String>,
    value: &str,
    path: &str,
) -> Result<(), CandidateMappingError> {
    if !values.insert(value.to_owned()) {
        return Err(CandidateMappingError::new(
            CandidateMappingErrorKind::DuplicateIdentity,
            path,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECKED_IN_CATALOG: &[u8] =
        include_bytes!("../../../../../research/decisions/D-004/d004-v0.5-candidate-mappings.json");

    #[test]
    fn aggregate_sr_map_identities_are_distinct_lower_hex_digests() {
        let catalog =
            parse_candidate_mapping_catalog(CHECKED_IN_CATALOG).expect("checked-in catalog");
        let mut observed = BTreeSet::new();
        for identity in catalog.identities() {
            assert_eq!(identity.sr_map_sha256.len(), 64);
            assert!(
                identity
                    .sr_map_sha256
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            );
            assert_ne!(identity.sr_map_sha256, identity.graph_sha256);
            assert!(observed.insert(identity.sr_map_sha256.clone()));
        }
        assert_eq!(observed.len(), CANDIDATE_MAPPING_COUNT);
    }

    #[test]
    fn aggregate_sr_map_identity_changes_on_row_substitution() {
        let mut value = strict_json::parse(CHECKED_IN_CATALOG).expect("checked-in JSON");
        let root = object_mut(&mut value);
        let subject = object_mut(
            root.get_mut("catalog_subject")
                .expect("catalog subject field"),
        );
        let graphs = array_mut(
            subject
                .get_mut("candidate_graphs")
                .expect("candidate graphs field"),
        );
        let first_record = object_mut(&mut graphs[0]);
        let graph = first_record.get_mut("graph").expect("graph field");
        let before = candidate_sr_map_digest_hex(graph, "$test/graph").expect("initial digest");
        let rows = array_mut(object_mut(graph).get_mut("sr_rows").expect("SR rows field"));
        rows.swap(0, 1);
        let after = candidate_sr_map_digest_hex(graph, "$test/graph").expect("mutated digest");
        assert_ne!(before, after);
    }

    fn object_mut(value: &mut JsonValue) -> &mut BTreeMap<String, JsonValue> {
        match value {
            JsonValue::Object(object) => object,
            _ => panic!("expected object"),
        }
    }

    fn array_mut(value: &mut JsonValue) -> &mut Vec<JsonValue> {
        match value {
            JsonValue::Array(array) => array,
            _ => panic!("expected array"),
        }
    }
}
