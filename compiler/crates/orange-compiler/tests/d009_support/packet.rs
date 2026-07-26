use std::collections::BTreeMap;

use super::domain::{
    ATOMIC_OUTCOME_MEANINGS, ATOMIC_OUTCOMES, BUDGETS, CANDIDATE_STATES, CANDIDATES,
    CASE_INPUT_NONCLAIMS, CASES, COMPARATIVE_LABELS, CONCLUSIONS, HARD_GATE_COUNT,
    HARD_GATE_STATE_PRECEDENCE, HARD_GATE_STATES, INPUT_BINDINGS, InputBinding, InputBindingId,
    METRICS, NONCLAIMS, OWNER_SCOPES, PROTOCOL_COUNTS, PROTOCOL_GAPS, REQUIRED_CANDIDATE_CASES,
    SEMANTIC_BINDING_COUNT, SEMANTIC_BINDINGS, SemanticBinding, SemanticBindingId,
};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

const PACKET_ROOT_FIELDS: [&str; 32] = [
    "atomic_outcome_meanings",
    "atomic_outcomes",
    "schema_version",
    "suite_version",
    "status",
    "epoch",
    "epoch_status",
    "owner_protocol_review",
    "independent_review_status",
    "candidates",
    "cases",
    "metrics",
    "hard_gate_count",
    "hard_gate_state_precedence",
    "hard_gate_states",
    "owner_scopes",
    "comparative_labels",
    "conclusions",
    "dependency_acceptance",
    "case_input_index_sha256",
    "input_bindings",
    "laboratory_budgets",
    "protocol_counts",
    "execution_resource_state",
    "candidate_states",
    "protocol_gaps",
    "execution",
    "physical_execution_order",
    "selection",
    "semantic_bindings",
    "conclusion",
    "nonclaims",
];
const CANDIDATE_FIELDS: [&str; 2] = ["id", "name"];
const DEPENDENCY_ACCEPTANCE_FIELDS: [&str; 2] = ["D-004", "D-005"];
const INPUT_BINDING_FIELDS: [&str; 2] = ["path", "sha256"];
const SEMANTIC_BINDING_FIELDS: [&str; 6] = [
    "normalization",
    "normalized_sha256",
    "path",
    "scope",
    "section_end_heading",
    "section_start_heading",
];
const LABORATORY_BUDGET_FIELDS: [&str; 4] = [
    "max_packet_bytes",
    "max_json_depth",
    "max_json_nodes",
    "max_string_bytes",
];
const PROTOCOL_COUNT_FIELDS: [&str; 6] = [
    "cold_bootstrap_runs",
    "deterministic_profile_runs",
    "maximum_same_owner_reproducibility_level",
    "owner_workspaces",
    "timed_replays_per_case",
    "unmeasured_warmups",
];
const EXECUTION_RESOURCE_FIELDS: [&str; 7] = [
    "case_output_bytes",
    "case_peak_memory_bytes",
    "case_temp_storage_bytes",
    "case_wall_seconds",
    "contract_status",
    "host_matrix_status",
    "timeout_semantics_status",
];
const CANDIDATE_STATE_FIELDS: [&str; 5] = [
    "adapter_status",
    "candidate",
    "dependency_admission_status",
    "execution_status",
    "implementation_status",
];
const EXECUTION_FIELDS: [&str; 5] = [
    "required_candidate_cases",
    "completed_candidate_cases",
    "complete_candidates",
    "complete_cross_candidate_cases",
    "evidence_status",
];

const INDEX_ROOT_FIELDS: [&str; 8] = [
    "schema_version",
    "suite_version",
    "status",
    "owner_protocol_review",
    "executable_inputs_status",
    "evidence_status",
    "case_inputs",
    "nonclaims",
];
const CASE_INPUT_FIELDS: [&str; 6] = [
    "candidate_mapping_status",
    "case",
    "coverage_status",
    "executable_fixture_count",
    "freeze_blocker",
    "shared_inputs_status",
];

pub(crate) const CASE_INPUT_INDEX_CANONICAL_SHA256: &str =
    "2e55c671771d5740b0346992c8b86b9cce0571a8fc3e5b745195b0956010470e";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PacketErrorKind {
    Json(JsonErrorKind),
    MissingField,
    UnknownField,
    InvalidValue,
    NonCanonicalEncoding,
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
pub(crate) struct DraftPacket {
    value: JsonValue,
    canonical: Vec<u8>,
    digest: [u8; 32],
}

impl DraftPacket {
    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&self.digest)
    }

    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn input_binding(&self, id: InputBindingId) -> InputBinding {
        INPUT_BINDINGS[id.index()]
    }

    pub(crate) fn semantic_binding(&self, id: SemanticBindingId) -> SemanticBinding {
        SEMANTIC_BINDINGS[id.index()]
    }

    pub(crate) const fn case_input_index_sha256(&self) -> &'static str {
        CASE_INPUT_INDEX_CANONICAL_SHA256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseInputIndex {
    value: JsonValue,
    canonical: Vec<u8>,
    digest: [u8; 32],
}

impl CaseInputIndex {
    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) const fn digest(&self) -> &[u8; 32] {
        &self.digest
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&self.digest)
    }

    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }
}

pub(crate) fn parse_draft_packet(input: &[u8]) -> Result<DraftPacket, PacketError> {
    let value = parse_json(input)?;
    validate_packet_shape(&value)?;
    if value != draft_packet_value() {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, "$"));
    }
    let canonical = strict_json::canonical_bytes(&value);
    if input != canonical_file_bytes(canonical.clone()) {
        return Err(PacketError::new(PacketErrorKind::NonCanonicalEncoding, "$"));
    }
    if case_input_index_digest_hex() != CASE_INPUT_INDEX_CANONICAL_SHA256 {
        return Err(PacketError::new(
            PacketErrorKind::InvalidValue,
            "$/case_input_index_sha256",
        ));
    }
    let digest = sha256::digest(&canonical);
    Ok(DraftPacket {
        value,
        canonical,
        digest,
    })
}

pub(crate) fn parse_case_input_index(input: &[u8]) -> Result<CaseInputIndex, PacketError> {
    let value = parse_json(input)?;
    validate_index_shape(&value)?;
    if value != case_input_index_value() {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, "$"));
    }
    let canonical = strict_json::canonical_bytes(&value);
    if input != canonical_file_bytes(canonical.clone()) {
        return Err(PacketError::new(PacketErrorKind::NonCanonicalEncoding, "$"));
    }
    let digest = sha256::digest(&canonical);
    if sha256::hex(&digest) != CASE_INPUT_INDEX_CANONICAL_SHA256 {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, "$"));
    }
    Ok(CaseInputIndex {
        value,
        canonical,
        digest,
    })
}

pub(crate) fn canonical_draft_packet_bytes() -> Vec<u8> {
    strict_json::canonical_bytes(&draft_packet_value())
}

pub(crate) fn canonical_draft_packet_file_bytes() -> Vec<u8> {
    canonical_file_bytes(canonical_draft_packet_bytes())
}

pub(crate) fn canonical_case_input_index_bytes() -> Vec<u8> {
    strict_json::canonical_bytes(&case_input_index_value())
}

pub(crate) fn canonical_case_input_index_file_bytes() -> Vec<u8> {
    canonical_file_bytes(canonical_case_input_index_bytes())
}

pub(crate) fn case_input_index_digest_hex() -> String {
    sha256::hex(&sha256::digest(&canonical_case_input_index_bytes()))
}

fn parse_json(input: &[u8]) -> Result<JsonValue, PacketError> {
    strict_json::parse(input).map_err(|error| {
        PacketError::new(
            PacketErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })
}

fn canonical_file_bytes(mut canonical: Vec<u8>) -> Vec<u8> {
    canonical.push(b'\n');
    canonical
}

fn validate_packet_shape(value: &JsonValue) -> Result<(), PacketError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &PACKET_ROOT_FIELDS, "$")?;

    validate_closed_object(root, "atomic_outcome_meanings", &ATOMIC_OUTCOMES, "$")?;
    validate_object_array(root, "candidates", &CANDIDATE_FIELDS, CANDIDATES.len())?;
    validate_closed_object(
        root,
        "dependency_acceptance",
        &DEPENDENCY_ACCEPTANCE_FIELDS,
        "$",
    )?;
    validate_closed_object(root, "input_bindings", &input_binding_names(), "$")?;
    let bindings = require_object(
        require_field(root, "input_bindings", "$")?,
        "$/input_bindings",
    )?;
    for binding in INPUT_BINDINGS {
        validate_closed_object(
            bindings,
            binding.id.as_str(),
            &INPUT_BINDING_FIELDS,
            "$/input_bindings",
        )?;
    }
    validate_closed_object(root, "semantic_bindings", &semantic_binding_names(), "$")?;
    let semantic_bindings = require_object(
        require_field(root, "semantic_bindings", "$")?,
        "$/semantic_bindings",
    )?;
    for binding in SEMANTIC_BINDINGS {
        validate_closed_object(
            semantic_bindings,
            binding.id.as_str(),
            &SEMANTIC_BINDING_FIELDS,
            "$/semantic_bindings",
        )?;
    }
    validate_closed_object(root, "laboratory_budgets", &LABORATORY_BUDGET_FIELDS, "$")?;
    validate_closed_object(root, "protocol_counts", &PROTOCOL_COUNT_FIELDS, "$")?;
    validate_closed_object(
        root,
        "execution_resource_state",
        &EXECUTION_RESOURCE_FIELDS,
        "$",
    )?;
    validate_object_array(
        root,
        "candidate_states",
        &CANDIDATE_STATE_FIELDS,
        CANDIDATE_STATES.len(),
    )?;
    validate_closed_object(root, "execution", &EXECUTION_FIELDS, "$")?;
    Ok(())
}

fn validate_index_shape(value: &JsonValue) -> Result<(), PacketError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &INDEX_ROOT_FIELDS, "$")?;
    validate_object_array(root, "case_inputs", &CASE_INPUT_FIELDS, CASES.len())?;
    Ok(())
}

fn validate_closed_object(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &[&str],
    parent: &str,
) -> Result<(), PacketError> {
    let path = format!("{parent}/{field}");
    let nested = require_object(require_field(object, field, parent)?, &path)?;
    exact_fields(nested, expected, &path)
}

fn validate_object_array(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected_fields: &[&str],
    expected_len: usize,
) -> Result<(), PacketError> {
    let path = format!("$/{field}");
    let values = require_field(object, field, "$")?
        .as_array()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, &path))?;
    if values.len() != expected_len {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, path));
    }
    for (index, value) in values.iter().enumerate() {
        let item_path = format!("$/{field}/{index}");
        let item = require_object(value, &item_path)?;
        exact_fields(item, expected_fields, &item_path)?;
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

fn require_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<&'a JsonValue, PacketError> {
    object
        .get(field)
        .ok_or_else(|| PacketError::new(PacketErrorKind::MissingField, format!("{path}/{field}")))
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
    for field in expected {
        if !object.contains_key(*field) {
            return Err(PacketError::new(
                PacketErrorKind::MissingField,
                format!("{path}/{field}"),
            ));
        }
    }
    Ok(())
}

fn input_binding_names() -> [&'static str; 2] {
    INPUT_BINDINGS.map(|binding| binding.id.as_str())
}

fn semantic_binding_names() -> [&'static str; SEMANTIC_BINDING_COUNT] {
    SEMANTIC_BINDINGS.map(|binding| binding.id.as_str())
}

fn draft_packet_value() -> JsonValue {
    strict_json::object([
        (
            "atomic_outcome_meanings".to_owned(),
            strict_json::object(
                ATOMIC_OUTCOME_MEANINGS.map(|(outcome, meaning)| string_entry(outcome, meaning)),
            ),
        ),
        (
            "atomic_outcomes".to_owned(),
            strict_json::strings(ATOMIC_OUTCOMES),
        ),
        ("candidate_states".to_owned(), candidate_states_value()),
        (
            "candidates".to_owned(),
            JsonValue::Array(
                CANDIDATES
                    .into_iter()
                    .map(|candidate| {
                        strict_json::object([
                            string_entry("id", candidate.as_str()),
                            string_entry("name", candidate.name()),
                        ])
                    })
                    .collect(),
            ),
        ),
        string_entry("case_input_index_sha256", CASE_INPUT_INDEX_CANONICAL_SHA256),
        (
            "cases".to_owned(),
            strict_json::strings(CASES.map(|case| case.as_str())),
        ),
        (
            "comparative_labels".to_owned(),
            strict_json::strings(COMPARATIVE_LABELS),
        ),
        ("conclusion".to_owned(), JsonValue::Null),
        ("conclusions".to_owned(), strict_json::strings(CONCLUSIONS)),
        (
            "dependency_acceptance".to_owned(),
            strict_json::object([bool_entry("D-004", false), bool_entry("D-005", false)]),
        ),
        ("epoch".to_owned(), JsonValue::Null),
        string_entry("epoch_status", "unfrozen"),
        ("execution".to_owned(), execution_value()),
        (
            "execution_resource_state".to_owned(),
            execution_resource_value(),
        ),
        usize_entry("hard_gate_count", HARD_GATE_COUNT),
        (
            "hard_gate_state_precedence".to_owned(),
            strict_json::strings(HARD_GATE_STATE_PRECEDENCE),
        ),
        (
            "hard_gate_states".to_owned(),
            strict_json::strings(HARD_GATE_STATES),
        ),
        string_entry("independent_review_status", "unavailable"),
        ("input_bindings".to_owned(), input_bindings_value()),
        ("laboratory_budgets".to_owned(), laboratory_budgets_value()),
        ("metrics".to_owned(), strict_json::strings(METRICS)),
        ("nonclaims".to_owned(), strict_json::strings(NONCLAIMS)),
        string_entry("owner_protocol_review", "none"),
        (
            "owner_scopes".to_owned(),
            strict_json::strings(OWNER_SCOPES),
        ),
        ("physical_execution_order".to_owned(), JsonValue::Null),
        ("protocol_counts".to_owned(), protocol_counts_value()),
        (
            "protocol_gaps".to_owned(),
            strict_json::strings(PROTOCOL_GAPS),
        ),
        string_entry("schema_version", "d009-pre-epoch-packet-v0.3"),
        ("selection".to_owned(), JsonValue::Null),
        ("semantic_bindings".to_owned(), semantic_bindings_value()),
        string_entry("status", "draft_unfrozen"),
        string_entry("suite_version", "d009-v0.1-draft"),
    ])
}

fn case_input_index_value() -> JsonValue {
    strict_json::object([
        (
            "case_inputs".to_owned(),
            JsonValue::Array(
                CASES
                    .into_iter()
                    .map(|case| {
                        strict_json::object([
                            string_entry("candidate_mapping_status", "absent"),
                            string_entry("case", case.as_str()),
                            string_entry("coverage_status", "unresolved"),
                            usize_entry("executable_fixture_count", 0),
                            bool_entry("freeze_blocker", true),
                            string_entry("shared_inputs_status", "absent"),
                        ])
                    })
                    .collect(),
            ),
        ),
        string_entry("evidence_status", "none"),
        string_entry("executable_inputs_status", "absent"),
        (
            "nonclaims".to_owned(),
            strict_json::strings(CASE_INPUT_NONCLAIMS),
        ),
        string_entry("owner_protocol_review", "none"),
        string_entry("schema_version", "d009-pre-epoch-case-input-index-v0.1"),
        string_entry("status", "draft_unreviewed"),
        string_entry("suite_version", "d009-v0.1-draft"),
    ])
}

fn input_bindings_value() -> JsonValue {
    strict_json::object(INPUT_BINDINGS.map(|binding| {
        (
            binding.id.as_str().to_owned(),
            strict_json::object([
                string_entry("path", binding.path),
                string_entry("sha256", binding.sha256),
            ]),
        )
    }))
}

fn semantic_bindings_value() -> JsonValue {
    strict_json::object(SEMANTIC_BINDINGS.map(|binding| {
        (
            binding.id.as_str().to_owned(),
            strict_json::object([
                string_entry("normalization", binding.normalization),
                string_entry("normalized_sha256", binding.normalized_sha256),
                string_entry("path", binding.path),
                string_entry("scope", binding.scope),
                (
                    "section_end_heading".to_owned(),
                    binding
                        .section_end_heading
                        .map_or(JsonValue::Null, |value| JsonValue::String(value.to_owned())),
                ),
                (
                    "section_start_heading".to_owned(),
                    binding
                        .section_start_heading
                        .map_or(JsonValue::Null, |value| JsonValue::String(value.to_owned())),
                ),
            ]),
        )
    }))
}

fn laboratory_budgets_value() -> JsonValue {
    strict_json::object([
        usize_entry("max_json_depth", BUDGETS.max_json_depth),
        usize_entry("max_json_nodes", BUDGETS.max_json_nodes),
        usize_entry("max_packet_bytes", BUDGETS.max_packet_bytes),
        usize_entry("max_string_bytes", BUDGETS.max_string_bytes),
    ])
}

fn protocol_counts_value() -> JsonValue {
    strict_json::object([
        usize_entry("cold_bootstrap_runs", PROTOCOL_COUNTS.cold_bootstrap_runs),
        usize_entry(
            "deterministic_profile_runs",
            PROTOCOL_COUNTS.deterministic_profile_runs,
        ),
        usize_entry(
            "maximum_same_owner_reproducibility_level",
            PROTOCOL_COUNTS.maximum_same_owner_reproducibility_level,
        ),
        usize_entry("owner_workspaces", PROTOCOL_COUNTS.owner_workspaces),
        usize_entry(
            "timed_replays_per_case",
            PROTOCOL_COUNTS.timed_replays_per_case,
        ),
        usize_entry("unmeasured_warmups", PROTOCOL_COUNTS.unmeasured_warmups),
    ])
}

fn execution_resource_value() -> JsonValue {
    strict_json::object([
        ("case_output_bytes".to_owned(), JsonValue::Null),
        ("case_peak_memory_bytes".to_owned(), JsonValue::Null),
        ("case_temp_storage_bytes".to_owned(), JsonValue::Null),
        ("case_wall_seconds".to_owned(), JsonValue::Null),
        string_entry("contract_status", "unassigned_freeze_blocker"),
        string_entry("host_matrix_status", "unassigned_freeze_blocker"),
        string_entry("timeout_semantics_status", "unassigned_freeze_blocker"),
    ])
}

fn candidate_states_value() -> JsonValue {
    JsonValue::Array(
        CANDIDATE_STATES
            .into_iter()
            .map(|state| {
                strict_json::object([
                    string_entry("adapter_status", "absent"),
                    string_entry("candidate", state.candidate.as_str()),
                    string_entry("dependency_admission_status", "absent"),
                    string_entry("execution_status", "not_performed"),
                    string_entry("implementation_status", "absent"),
                ])
            })
            .collect(),
    )
}

fn execution_value() -> JsonValue {
    strict_json::object([
        usize_entry("complete_candidates", 0),
        usize_entry("complete_cross_candidate_cases", 0),
        usize_entry("completed_candidate_cases", 0),
        string_entry("evidence_status", "none"),
        usize_entry("required_candidate_cases", REQUIRED_CANDIDATE_CASES),
    ])
}

fn string_entry(key: &str, value: &str) -> (String, JsonValue) {
    (key.to_owned(), JsonValue::String(value.to_owned()))
}

fn usize_entry(key: &str, value: usize) -> (String, JsonValue) {
    (
        key.to_owned(),
        JsonValue::Integer(i64::try_from(value).unwrap_or(i64::MAX)),
    )
}

fn bool_entry(key: &str, value: bool) -> (String, JsonValue) {
    (key.to_owned(), JsonValue::Bool(value))
}
