use std::collections::BTreeMap;

use super::cases::MUTATIONS;
use super::domain::{
    BUDGETS, CANDIDATES, CASES, CLAIM_FAMILIES, HARD_GATES, INPUT_BINDINGS, InputBinding,
    InputBindingId, LEGACY_V01_MUTATIONS, METRICS, NONCLAIMS, OWNER_SCOPES,
    REQUIRED_CANDIDATE_CASES,
};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

const ROOT_FIELDS: [&str; 18] = [
    "schema_version",
    "suite_version",
    "epoch",
    "status",
    "candidates",
    "cases",
    "claim_families",
    "metrics",
    "hard_gates",
    "owner_scopes",
    "mutations",
    "mutation_manifest_sha256",
    "legacy_v01_mutations",
    "input_bindings",
    "budgets",
    "execution",
    "selection",
    "nonclaims",
];
const INPUT_BINDING_FIELDS: [&str; 2] = ["path", "sha256"];
const BUDGET_FIELDS: [&str; 10] = [
    "max_packet_bytes",
    "max_json_depth",
    "max_json_nodes",
    "max_string_bytes",
    "max_diagnostics",
    "max_claims",
    "max_edges",
    "max_output_bytes",
    "render_repetitions",
    "workspace_replays",
];
const EXECUTION_FIELDS: [&str; 3] = [
    "required_candidate_cases",
    "completed_candidate_cases",
    "evidence_status",
];
pub(crate) const MUTATION_MANIFEST_SHA256: &str =
    "8d069daf4a9443cf9df2d127f86d834e1aefed149324503f980c43f29c356082";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PacketErrorKind {
    Json(JsonErrorKind),
    MissingField,
    UnknownField,
    InvalidValue,
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
    input_bindings: [InputBinding; 3],
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
        match id {
            InputBindingId::DecisionSuite => self.input_bindings[0],
            InputBindingId::LegacyV01Manifest => self.input_bindings[1],
            InputBindingId::ClaimRecordV01Schema => self.input_bindings[2],
        }
    }
}

pub(crate) fn parse_draft_packet(input: &[u8]) -> Result<DraftPacket, PacketError> {
    let value = strict_json::parse(input).map_err(|error| {
        PacketError::new(
            PacketErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })?;
    validate_packet(&value)?;
    let canonical = strict_json::canonical_bytes(&value);
    let digest = sha256::digest(&canonical);
    Ok(DraftPacket {
        value,
        canonical,
        digest,
        input_bindings: INPUT_BINDINGS,
    })
}

pub(crate) fn canonical_draft_packet_bytes() -> Vec<u8> {
    strict_json::canonical_bytes(&draft_packet_value())
}

pub(crate) fn canonical_mutation_manifest_bytes() -> Vec<u8> {
    strict_json::canonical_bytes(&mutation_manifest_value())
}

pub(crate) fn mutation_manifest_digest_hex() -> String {
    sha256::hex(&sha256::digest(&canonical_mutation_manifest_bytes()))
}

fn validate_packet(value: &JsonValue) -> Result<(), PacketError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &ROOT_FIELDS, "$")?;

    require_exact_string(root, "schema_version", "d005-execution-packet-v0.1", "$")?;
    require_exact_string(root, "suite_version", "d005-v0.1-draft", "$")?;
    require_exact_string(root, "epoch", "0001", "$")?;
    require_exact_string(root, "status", "draft", "$")?;

    let candidate_ids = CANDIDATES.map(|candidate| candidate.as_str());
    let case_ids = CASES.map(|case| case.as_str());
    let mutation_ids = MUTATIONS.map(|mutation| mutation.id);
    require_exact_strings(root, "candidates", &candidate_ids)?;
    require_exact_strings(root, "cases", &case_ids)?;
    require_exact_strings(root, "claim_families", &CLAIM_FAMILIES)?;
    require_exact_strings(root, "metrics", &METRICS)?;
    require_exact_strings(root, "hard_gates", &HARD_GATES)?;
    require_exact_strings(root, "owner_scopes", &OWNER_SCOPES)?;
    require_exact_strings(root, "mutations", &mutation_ids)?;
    if mutation_manifest_digest_hex() != MUTATION_MANIFEST_SHA256 {
        return Err(PacketError::new(
            PacketErrorKind::InvalidValue,
            "$/mutation_manifest_sha256",
        ));
    }
    require_exact_string(
        root,
        "mutation_manifest_sha256",
        MUTATION_MANIFEST_SHA256,
        "$",
    )?;
    require_exact_strings(root, "legacy_v01_mutations", &LEGACY_V01_MUTATIONS)?;
    require_exact_strings(root, "nonclaims", &NONCLAIMS)?;
    validate_input_bindings(root)?;

    let budgets = require_object(require_field(root, "budgets", "$")?, "$/budgets")?;
    exact_fields(budgets, &BUDGET_FIELDS, "$/budgets")?;
    require_exact_usize(
        budgets,
        "max_packet_bytes",
        BUDGETS.max_packet_bytes,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "max_json_depth",
        BUDGETS.max_json_depth,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "max_json_nodes",
        BUDGETS.max_json_nodes,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "max_string_bytes",
        BUDGETS.max_string_bytes,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "max_diagnostics",
        BUDGETS.max_diagnostics,
        "$/budgets",
    )?;
    require_exact_usize(budgets, "max_claims", BUDGETS.max_claims, "$/budgets")?;
    require_exact_usize(budgets, "max_edges", BUDGETS.max_edges, "$/budgets")?;
    require_exact_usize(
        budgets,
        "max_output_bytes",
        BUDGETS.max_output_bytes,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "render_repetitions",
        BUDGETS.render_repetitions,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "workspace_replays",
        BUDGETS.workspace_replays,
        "$/budgets",
    )?;

    let execution = require_object(require_field(root, "execution", "$")?, "$/execution")?;
    exact_fields(execution, &EXECUTION_FIELDS, "$/execution")?;
    require_exact_usize(
        execution,
        "required_candidate_cases",
        REQUIRED_CANDIDATE_CASES,
        "$/execution",
    )?;
    require_exact_usize(execution, "completed_candidate_cases", 0, "$/execution")?;
    require_exact_string(execution, "evidence_status", "none", "$/execution")?;

    if require_field(root, "selection", "$")? != &JsonValue::Null {
        return Err(PacketError::new(
            PacketErrorKind::InvalidValue,
            "$/selection",
        ));
    }
    Ok(())
}

fn validate_input_bindings(root: &BTreeMap<String, JsonValue>) -> Result<(), PacketError> {
    let bindings = require_object(
        require_field(root, "input_bindings", "$")?,
        "$/input_bindings",
    )?;
    let binding_names = INPUT_BINDINGS.map(|binding| binding.id.as_str());
    exact_fields(bindings, &binding_names, "$/input_bindings")?;
    for binding in INPUT_BINDINGS {
        let parent = format!("$/input_bindings/{}", binding.id.as_str());
        let record = require_object(
            require_field(bindings, binding.id.as_str(), "$/input_bindings")?,
            &parent,
        )?;
        exact_fields(record, &INPUT_BINDING_FIELDS, &parent)?;
        require_exact_string(record, "path", binding.path, &parent)?;
        require_exact_string(record, "sha256", binding.sha256, &parent)?;
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

fn require_exact_string(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &str,
    parent: &str,
) -> Result<(), PacketError> {
    let path = format!("{parent}/{field}");
    let observed = require_field(object, field, parent)?
        .as_str()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, &path))?;
    if observed != expected {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn require_exact_strings(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &[&str],
) -> Result<(), PacketError> {
    let path = format!("$/{field}");
    let observed = require_field(object, field, "$")?
        .as_array()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, &path))?;
    if observed.len() != expected.len()
        || observed
            .iter()
            .zip(expected)
            .any(|(actual, wanted)| actual.as_str() != Some(*wanted))
    {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn require_exact_usize(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: usize,
    parent: &str,
) -> Result<(), PacketError> {
    let path = format!("{parent}/{field}");
    let integer = require_field(object, field, parent)?
        .as_integer()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, &path))?;
    if usize::try_from(integer).ok() != Some(expected) {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn draft_packet_value() -> JsonValue {
    let candidate_ids = CANDIDATES.map(|candidate| candidate.as_str());
    let case_ids = CASES.map(|case| case.as_str());
    let mutation_ids = MUTATIONS.map(|mutation| mutation.id);
    strict_json::object([
        string_entry("schema_version", "d005-execution-packet-v0.1"),
        string_entry("suite_version", "d005-v0.1-draft"),
        string_entry("epoch", "0001"),
        string_entry("status", "draft"),
        ("candidates".to_owned(), strict_json::strings(candidate_ids)),
        ("cases".to_owned(), strict_json::strings(case_ids)),
        (
            "claim_families".to_owned(),
            strict_json::strings(CLAIM_FAMILIES),
        ),
        ("metrics".to_owned(), strict_json::strings(METRICS)),
        ("hard_gates".to_owned(), strict_json::strings(HARD_GATES)),
        (
            "owner_scopes".to_owned(),
            strict_json::strings(OWNER_SCOPES),
        ),
        ("mutations".to_owned(), strict_json::strings(mutation_ids)),
        string_entry("mutation_manifest_sha256", MUTATION_MANIFEST_SHA256),
        (
            "legacy_v01_mutations".to_owned(),
            strict_json::strings(LEGACY_V01_MUTATIONS),
        ),
        ("input_bindings".to_owned(), input_bindings_value()),
        ("budgets".to_owned(), budget_value()),
        ("execution".to_owned(), execution_value()),
        ("selection".to_owned(), JsonValue::Null),
        ("nonclaims".to_owned(), strict_json::strings(NONCLAIMS)),
    ])
}

fn mutation_manifest_value() -> JsonValue {
    JsonValue::Array(
        MUTATIONS
            .iter()
            .map(|mutation| {
                strict_json::object([
                    string_entry("id", mutation.id),
                    string_entry("case", mutation.case.as_str()),
                    string_entry("description", mutation.description),
                ])
            })
            .collect(),
    )
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

fn budget_value() -> JsonValue {
    strict_json::object([
        usize_entry("max_packet_bytes", BUDGETS.max_packet_bytes),
        usize_entry("max_json_depth", BUDGETS.max_json_depth),
        usize_entry("max_json_nodes", BUDGETS.max_json_nodes),
        usize_entry("max_string_bytes", BUDGETS.max_string_bytes),
        usize_entry("max_diagnostics", BUDGETS.max_diagnostics),
        usize_entry("max_claims", BUDGETS.max_claims),
        usize_entry("max_edges", BUDGETS.max_edges),
        usize_entry("max_output_bytes", BUDGETS.max_output_bytes),
        usize_entry("render_repetitions", BUDGETS.render_repetitions),
        usize_entry("workspace_replays", BUDGETS.workspace_replays),
    ])
}

fn execution_value() -> JsonValue {
    strict_json::object([
        usize_entry("required_candidate_cases", REQUIRED_CANDIDATE_CASES),
        usize_entry("completed_candidate_cases", 0),
        string_entry("evidence_status", "none"),
    ])
}

fn string_entry(key: &str, value: &str) -> (String, JsonValue) {
    (key.to_owned(), JsonValue::String(value.to_owned()))
}

fn usize_entry(key: &str, value: usize) -> (String, JsonValue) {
    let integer = i64::try_from(value).unwrap_or(i64::MAX);
    (key.to_owned(), JsonValue::Integer(integer))
}
