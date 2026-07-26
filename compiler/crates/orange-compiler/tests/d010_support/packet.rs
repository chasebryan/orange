use std::collections::BTreeMap;

use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

const PACKET_ROOT_FIELDS: [&str; 33] = [
    "atomic_outcome_meanings",
    "atomic_outcomes",
    "candidate_states",
    "candidates",
    "case_input_index_sha256",
    "cases",
    "comparative_axes",
    "comparative_labels",
    "conclusion",
    "conclusions",
    "dependency_acceptance",
    "epoch",
    "epoch_status",
    "execution",
    "execution_resource_state",
    "hard_gate_count",
    "hard_gate_state_precedence",
    "hard_gate_states",
    "independent_review_status",
    "input_bindings",
    "laboratory_budgets",
    "metrics",
    "nonclaims",
    "owner_protocol_review",
    "owner_scopes",
    "physical_execution_order",
    "protocol_counts",
    "protocol_gaps",
    "schema_version",
    "selection",
    "semantic_bindings",
    "status",
    "suite_version",
];

const INDEX_ROOT_FIELDS: [&str; 8] = [
    "case_inputs",
    "evidence_status",
    "executable_inputs_status",
    "nonclaims",
    "owner_protocol_review",
    "schema_version",
    "status",
    "suite_version",
];

const SEMANTIC_BINDING_NAMES: [&str; 10] = [
    "architecture_document",
    "assurance_document",
    "compiler_strategy_suite",
    "decision_register_d010",
    "decision_register_document",
    "research_document",
    "roadmap_document",
    "roadmap_s5",
    "threat_model_document",
    "traceability_document",
];

const PACKET_CANONICAL_SHA256: &str =
    "ae6c8fc6e433d3cbb895aebf95667711d442efa348e241c0059e72a489bcac9b";
pub(crate) const CASE_INPUT_INDEX_CANONICAL_SHA256: &str =
    "4c8b0547a8f3bd380f4569008c8728014bb1d8718a5bfe17402bd03866560209";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PacketErrorKind {
    Json(JsonErrorKind),
    MissingField,
    UnknownField,
    InvalidValue,
    NonCanonicalEncoding,
    IdentityMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PacketError {
    pub(crate) kind: PacketErrorKind,
    pub(crate) path: String,
}

impl PacketError {
    fn new(kind: PacketErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoundJson {
    value: JsonValue,
    canonical: Vec<u8>,
    digest: [u8; 32],
}

impl BoundJson {
    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&self.digest)
    }
}

pub(crate) fn parse_draft_packet(input: &[u8]) -> Result<BoundJson, PacketError> {
    let document = parse_canonical(input, PACKET_CANONICAL_SHA256)?;
    validate_packet_shape(document.value())?;
    Ok(document)
}

pub(crate) fn parse_case_input_index(input: &[u8]) -> Result<BoundJson, PacketError> {
    let document = parse_canonical(input, CASE_INPUT_INDEX_CANONICAL_SHA256)?;
    validate_index_shape(document.value())?;
    Ok(document)
}

fn parse_canonical(input: &[u8], expected_digest: &str) -> Result<BoundJson, PacketError> {
    let value = strict_json::parse(input).map_err(|error| {
        PacketError::new(
            PacketErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })?;
    let canonical = strict_json::canonical_bytes(&value);
    let mut canonical_file = canonical.clone();
    canonical_file.push(b'\n');
    if input != canonical_file {
        return Err(PacketError::new(PacketErrorKind::NonCanonicalEncoding, "$"));
    }
    let digest = sha256::digest(&canonical);
    if sha256::hex(&digest) != expected_digest {
        return Err(PacketError::new(PacketErrorKind::IdentityMismatch, "$"));
    }
    Ok(BoundJson {
        value,
        canonical,
        digest,
    })
}

fn validate_packet_shape(value: &JsonValue) -> Result<(), PacketError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &PACKET_ROOT_FIELDS, "$")?;
    closed_object(
        root,
        "atomic_outcome_meanings",
        &["not_satisfied", "satisfied", "unresolved", "unsupported"],
    )?;
    closed_object(
        root,
        "dependency_acceptance",
        &["D-003", "D-004", "D-005", "D-006", "D-009"],
    )?;
    closed_object(
        root,
        "input_bindings",
        &["case_input_index", "compiler_strategy_suite"],
    )?;
    closed_object(root, "semantic_bindings", &SEMANTIC_BINDING_NAMES)?;
    object_array(root, "candidates", &["id", "name"], 5)?;
    object_array(
        root,
        "candidate_states",
        &[
            "adapter_status",
            "candidate",
            "dependency_admission_status",
            "execution_status",
            "implementation_status",
        ],
        5,
    )?;
    closed_object(
        root,
        "execution",
        &[
            "complete_candidates",
            "complete_cross_candidate_cases",
            "completed_candidate_cases",
            "evidence_status",
            "required_candidate_cases",
        ],
    )?;
    Ok(())
}

fn validate_index_shape(value: &JsonValue) -> Result<(), PacketError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &INDEX_ROOT_FIELDS, "$")?;
    object_array(
        root,
        "case_inputs",
        &[
            "candidate_mapping_status",
            "case",
            "coverage_status",
            "executable_fixture_count",
            "freeze_blocker",
            "shared_inputs_status",
        ],
        8,
    )
}

fn closed_object(
    root: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &[&str],
) -> Result<(), PacketError> {
    let path = format!("$/{field}");
    let value = root
        .get(field)
        .ok_or_else(|| PacketError::new(PacketErrorKind::MissingField, &path))?;
    exact_fields(require_object(value, &path)?, expected, &path)
}

fn object_array(
    root: &BTreeMap<String, JsonValue>,
    field: &str,
    expected_fields: &[&str],
    expected_len: usize,
) -> Result<(), PacketError> {
    let path = format!("$/{field}");
    let values = root
        .get(field)
        .and_then(JsonValue::as_array)
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, &path))?;
    if values.len() != expected_len {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, path));
    }
    for (index, value) in values.iter().enumerate() {
        let item_path = format!("$/{field}/{index}");
        exact_fields(
            require_object(value, &item_path)?,
            expected_fields,
            &item_path,
        )?;
    }
    Ok(())
}

fn require_object<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, PacketError> {
    value
        .as_object()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, path))
}

fn exact_fields(
    object: &BTreeMap<String, JsonValue>,
    expected: &[&str],
    path: &str,
) -> Result<(), PacketError> {
    if let Some(field) = object
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        return Err(PacketError::new(
            PacketErrorKind::UnknownField,
            format!("{path}/{field}"),
        ));
    }
    if let Some(field) = expected.iter().find(|field| !object.contains_key(**field)) {
        return Err(PacketError::new(
            PacketErrorKind::MissingField,
            format!("{path}/{field}"),
        ));
    }
    Ok(())
}
