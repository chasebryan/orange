use std::collections::BTreeMap;

use super::cases::MUTATIONS;
use super::domain::{
    BUDGETS, CANDIDATES, CASE_SCOPED_CROSS_CUTTING_PROPOSALS, CASE_VERDICTS, CASES,
    CROSS_CUTTING_PROPOSAL_CLASS_STATUSES, CROSS_CUTTING_PROPOSAL_COUNT,
    CROSS_CUTTING_PROPOSAL_NONCLAIMS, DOMAIN_OBSERVATION_STATES, HARD_GATES,
    IDENTITY_SUBSTITUTION_PROPOSALS, INPUT_BINDINGS, InputBinding, InputBindingId,
    MISSING_EDGE_PROPOSAL_IDS, NONCLAIMS, PROTOCOL_GAPS, RELATIONSHIPS, REQUIRED_CANDIDATE_CASES,
    SOURCE_ROLES, UNRESOLVED_CROSS_CUTTING_FIXTURE_CLASSES,
};
use super::fixtures::FIXTURE_CATALOG_CANONICAL_SHA256;
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

const ROOT_FIELDS: [&str; 27] = [
    "schema_version",
    "suite_version",
    "status",
    "epoch",
    "epoch_status",
    "d003_disposition",
    "owner_protocol_review",
    "candidates",
    "cases",
    "source_roles",
    "relationships",
    "domain_observation_states",
    "case_verdicts",
    "hard_gates",
    "mutations",
    "mutation_manifest_sha256",
    "cross_cutting_fixture_proposal_manifest_sha256",
    "cross_cutting_executable_fixture_catalog_sha256",
    "fixture_inventory_status",
    "unresolved_cross_cutting_fixture_classes",
    "protocol_gaps",
    "input_bindings",
    "budgets",
    "execution",
    "selection",
    "conclusion",
    "nonclaims",
];
const CROSS_CUTTING_PROPOSAL_ROOT_FIELDS: [&str; 9] = [
    "schema_version",
    "status",
    "owner_protocol_review",
    "executable_inputs_status",
    "class_statuses",
    "replay_repetitions",
    "evidence_status",
    "proposals",
    "nonclaims",
];
const CROSS_CUTTING_PROPOSAL_CLASS_STATUS_FIELDS: [&str; 6] = [
    "class",
    "proposal_count",
    "proposal_status",
    "executable_fixture_count",
    "coverage_status",
    "freeze_blocker",
];
const CROSS_CUTTING_PROPOSAL_FIELDS: [&str; 12] = [
    "id",
    "class",
    "case_scope",
    "relationship_scope",
    "layer",
    "mutation_kind",
    "target",
    "expected_state",
    "required_invalidation",
    "match_rule",
    "capability_credit",
    "observation_level",
];
const INPUT_BINDING_FIELDS: [&str; 2] = ["path", "sha256"];
const BUDGET_FIELDS: [&str; 10] = [
    "max_packet_bytes",
    "max_json_depth",
    "max_json_nodes",
    "max_string_bytes",
    "case_wall_seconds",
    "case_peak_memory_bytes",
    "case_temp_storage_bytes",
    "case_output_bytes",
    "candidate_owner_hours",
    "correction_owner_hours",
];
const EXECUTION_FIELDS: [&str; 5] = [
    "required_candidate_cases",
    "completed_candidate_cases",
    "complete_candidates",
    "complete_cross_candidate_cases",
    "evidence_status",
];

pub(crate) const MUTATION_MANIFEST_SHA256: &str =
    "970999d998cdc202a6caa4e2f798017416c88211a5b6b8508132a07cc9080c0c";
pub(crate) const CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256: &str =
    "457c14e7d41f677b21af254af45e331b24e6c685a7d7aa8eae556ced5bd7be65";
pub(crate) const CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256: &str =
    FIXTURE_CATALOG_CANONICAL_SHA256;

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
    if input != canonical_file_bytes(canonical.clone()) {
        return Err(PacketError::new(PacketErrorKind::NonCanonicalEncoding, "$"));
    }
    let digest = sha256::digest(&canonical);
    Ok(DraftPacket {
        value,
        canonical,
        digest,
    })
}

pub(crate) fn parse_mutation_manifest(input: &[u8]) -> Result<JsonValue, PacketError> {
    let value = strict_json::parse(input).map_err(|error| {
        PacketError::new(
            PacketErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })?;
    if value != mutation_manifest_value() {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, "$"));
    }
    if input != canonical_file_bytes(strict_json::canonical_bytes(&value)) {
        return Err(PacketError::new(PacketErrorKind::NonCanonicalEncoding, "$"));
    }
    Ok(value)
}

pub(crate) fn parse_cross_cutting_fixture_proposal_manifest(
    input: &[u8],
) -> Result<JsonValue, PacketError> {
    let value = strict_json::parse(input).map_err(|error| {
        PacketError::new(
            PacketErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })?;
    validate_cross_cutting_fixture_proposal_manifest(&value)?;
    if input != canonical_file_bytes(strict_json::canonical_bytes(&value)) {
        return Err(PacketError::new(PacketErrorKind::NonCanonicalEncoding, "$"));
    }
    Ok(value)
}

pub(crate) fn canonical_draft_packet_bytes() -> Vec<u8> {
    strict_json::canonical_bytes(&draft_packet_value())
}

pub(crate) fn canonical_draft_packet_file_bytes() -> Vec<u8> {
    canonical_file_bytes(canonical_draft_packet_bytes())
}

pub(crate) fn canonical_mutation_manifest_bytes() -> Vec<u8> {
    strict_json::canonical_bytes(&mutation_manifest_value())
}

pub(crate) fn canonical_mutation_manifest_file_bytes() -> Vec<u8> {
    canonical_file_bytes(canonical_mutation_manifest_bytes())
}

pub(crate) fn mutation_manifest_digest_hex() -> String {
    sha256::hex(&sha256::digest(&canonical_mutation_manifest_bytes()))
}

pub(crate) fn canonical_cross_cutting_fixture_proposal_manifest_bytes() -> Vec<u8> {
    strict_json::canonical_bytes(&cross_cutting_fixture_proposal_manifest_value())
}

pub(crate) fn canonical_cross_cutting_fixture_proposal_manifest_file_bytes() -> Vec<u8> {
    canonical_file_bytes(canonical_cross_cutting_fixture_proposal_manifest_bytes())
}

pub(crate) fn cross_cutting_fixture_proposal_manifest_digest_hex() -> String {
    sha256::hex(&sha256::digest(
        &canonical_cross_cutting_fixture_proposal_manifest_bytes(),
    ))
}

fn canonical_file_bytes(mut canonical: Vec<u8>) -> Vec<u8> {
    canonical.push(b'\n');
    canonical
}

fn validate_packet(value: &JsonValue) -> Result<(), PacketError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &ROOT_FIELDS, "$")?;

    require_exact_string(root, "schema_version", "d004-pre-epoch-packet-v0.3", "$")?;
    require_exact_string(root, "suite_version", "d004-v0.3-draft", "$")?;
    require_exact_string(root, "status", "draft_unfrozen", "$")?;
    require_null(root, "epoch", "$")?;
    require_exact_string(root, "epoch_status", "unfrozen", "$")?;
    require_exact_string(
        root,
        "d003_disposition",
        "owner_accepted_pending_exact_revision_oep_closure",
        "$",
    )?;
    require_exact_string(root, "owner_protocol_review", "none", "$")?;

    let candidate_ids = CANDIDATES.map(|candidate| candidate.as_str());
    let case_ids = CASES.map(|case| case.as_str());
    let mutation_ids = MUTATIONS.map(|mutation| mutation.id);
    require_exact_strings(root, "candidates", &candidate_ids)?;
    require_exact_strings(root, "cases", &case_ids)?;
    require_exact_strings(root, "source_roles", &SOURCE_ROLES)?;
    require_exact_strings(root, "relationships", &RELATIONSHIPS)?;
    require_exact_strings(
        root,
        "domain_observation_states",
        &DOMAIN_OBSERVATION_STATES,
    )?;
    require_exact_strings(root, "case_verdicts", &CASE_VERDICTS)?;
    require_exact_strings(root, "hard_gates", &HARD_GATES)?;
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
    if cross_cutting_fixture_proposal_manifest_digest_hex()
        != CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256
    {
        return Err(PacketError::new(
            PacketErrorKind::InvalidValue,
            "$/cross_cutting_fixture_proposal_manifest_sha256",
        ));
    }
    require_exact_string(
        root,
        "cross_cutting_fixture_proposal_manifest_sha256",
        CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256,
        "$",
    )?;
    require_exact_string(
        root,
        "cross_cutting_executable_fixture_catalog_sha256",
        CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256,
        "$",
    )?;
    require_exact_string(
        root,
        "fixture_inventory_status",
        "cross_cutting_materialized_unreviewed_freeze_blocker",
        "$",
    )?;
    require_exact_strings(
        root,
        "unresolved_cross_cutting_fixture_classes",
        &UNRESOLVED_CROSS_CUTTING_FIXTURE_CLASSES,
    )?;
    require_exact_strings(root, "protocol_gaps", &PROTOCOL_GAPS)?;
    require_exact_strings(root, "nonclaims", &NONCLAIMS)?;
    validate_input_bindings(root)?;
    validate_budgets(root)?;
    validate_execution(root)?;
    require_null(root, "selection", "$")?;
    require_null(root, "conclusion", "$")?;

    if value != &draft_packet_value() {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, "$"));
    }
    Ok(())
}

fn validate_cross_cutting_fixture_proposal_manifest(value: &JsonValue) -> Result<(), PacketError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &CROSS_CUTTING_PROPOSAL_ROOT_FIELDS, "$")?;
    require_exact_string(
        root,
        "schema_version",
        "d004-cross-cutting-fixture-proposals-v0.2",
        "$",
    )?;
    require_exact_string(root, "status", "draft_unreviewed", "$")?;
    require_exact_string(root, "owner_protocol_review", "none", "$")?;
    require_exact_string(root, "executable_inputs_status", "absent", "$")?;
    let class_statuses = require_field(root, "class_statuses", "$")?
        .as_array()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, "$/class_statuses"))?;
    if class_statuses.len() != CROSS_CUTTING_PROPOSAL_CLASS_STATUSES.len() {
        return Err(PacketError::new(
            PacketErrorKind::InvalidValue,
            "$/class_statuses",
        ));
    }
    for (index, status) in class_statuses.iter().enumerate() {
        let path = format!("$/class_statuses/{index}");
        let record = require_object(status, &path)?;
        exact_fields(record, &CROSS_CUTTING_PROPOSAL_CLASS_STATUS_FIELDS, &path)?;
    }
    require_null(root, "replay_repetitions", "$")?;
    require_exact_string(root, "evidence_status", "none", "$")?;
    require_exact_strings(root, "nonclaims", &CROSS_CUTTING_PROPOSAL_NONCLAIMS)?;

    let proposals = require_field(root, "proposals", "$")?
        .as_array()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, "$/proposals"))?;
    if proposals.len() != CROSS_CUTTING_PROPOSAL_COUNT {
        return Err(PacketError::new(
            PacketErrorKind::InvalidValue,
            "$/proposals",
        ));
    }
    for (index, proposal) in proposals.iter().enumerate() {
        let path = format!("$/proposals/{index}");
        let record = require_object(proposal, &path)?;
        exact_fields(record, &CROSS_CUTTING_PROPOSAL_FIELDS, &path)?;
    }
    if value != &cross_cutting_fixture_proposal_manifest_value() {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, "$"));
    }
    Ok(())
}

fn validate_input_bindings(root: &BTreeMap<String, JsonValue>) -> Result<(), PacketError> {
    let bindings = require_object(
        require_field(root, "input_bindings", "$")?,
        "$/input_bindings",
    )?;
    let names = INPUT_BINDINGS.map(|binding| binding.id.as_str());
    exact_fields(bindings, &names, "$/input_bindings")?;
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

fn validate_budgets(root: &BTreeMap<String, JsonValue>) -> Result<(), PacketError> {
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
        "case_wall_seconds",
        BUDGETS.case_wall_seconds,
        "$/budgets",
    )?;
    require_exact_u64(
        budgets,
        "case_peak_memory_bytes",
        BUDGETS.case_peak_memory_bytes,
        "$/budgets",
    )?;
    require_exact_u64(
        budgets,
        "case_temp_storage_bytes",
        BUDGETS.case_temp_storage_bytes,
        "$/budgets",
    )?;
    require_exact_u64(
        budgets,
        "case_output_bytes",
        BUDGETS.case_output_bytes,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "candidate_owner_hours",
        BUDGETS.candidate_owner_hours,
        "$/budgets",
    )?;
    require_exact_usize(
        budgets,
        "correction_owner_hours",
        BUDGETS.correction_owner_hours,
        "$/budgets",
    )?;
    Ok(())
}

fn validate_execution(root: &BTreeMap<String, JsonValue>) -> Result<(), PacketError> {
    let execution = require_object(require_field(root, "execution", "$")?, "$/execution")?;
    exact_fields(execution, &EXECUTION_FIELDS, "$/execution")?;
    require_exact_usize(
        execution,
        "required_candidate_cases",
        REQUIRED_CANDIDATE_CASES,
        "$/execution",
    )?;
    require_exact_usize(execution, "completed_candidate_cases", 0, "$/execution")?;
    require_exact_usize(execution, "complete_candidates", 0, "$/execution")?;
    require_exact_usize(
        execution,
        "complete_cross_candidate_cases",
        0,
        "$/execution",
    )?;
    require_exact_string(execution, "evidence_status", "none", "$/execution")?;
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
    let expected_u64 = u64::try_from(expected).map_err(|_| {
        PacketError::new(PacketErrorKind::InvalidValue, format!("{parent}/{field}"))
    })?;
    require_exact_u64(object, field, expected_u64, parent)
}

fn require_exact_u64(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: u64,
    parent: &str,
) -> Result<(), PacketError> {
    let path = format!("{parent}/{field}");
    let integer = require_field(object, field, parent)?
        .as_integer()
        .ok_or_else(|| PacketError::new(PacketErrorKind::InvalidValue, &path))?;
    if u64::try_from(integer).ok() != Some(expected) {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn require_null(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<(), PacketError> {
    let path = format!("{parent}/{field}");
    if require_field(object, field, parent)? != &JsonValue::Null {
        return Err(PacketError::new(PacketErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn draft_packet_value() -> JsonValue {
    let candidate_ids = CANDIDATES.map(|candidate| candidate.as_str());
    let case_ids = CASES.map(|case| case.as_str());
    let mutation_ids = MUTATIONS.map(|mutation| mutation.id);
    strict_json::object([
        string_entry("schema_version", "d004-pre-epoch-packet-v0.3"),
        string_entry("suite_version", "d004-v0.3-draft"),
        string_entry("status", "draft_unfrozen"),
        ("epoch".to_owned(), JsonValue::Null),
        string_entry("epoch_status", "unfrozen"),
        string_entry(
            "d003_disposition",
            "owner_accepted_pending_exact_revision_oep_closure",
        ),
        string_entry("owner_protocol_review", "none"),
        ("candidates".to_owned(), strict_json::strings(candidate_ids)),
        ("cases".to_owned(), strict_json::strings(case_ids)),
        (
            "source_roles".to_owned(),
            strict_json::strings(SOURCE_ROLES),
        ),
        (
            "relationships".to_owned(),
            strict_json::strings(RELATIONSHIPS),
        ),
        (
            "domain_observation_states".to_owned(),
            strict_json::strings(DOMAIN_OBSERVATION_STATES),
        ),
        (
            "case_verdicts".to_owned(),
            strict_json::strings(CASE_VERDICTS),
        ),
        ("hard_gates".to_owned(), strict_json::strings(HARD_GATES)),
        ("mutations".to_owned(), strict_json::strings(mutation_ids)),
        string_entry("mutation_manifest_sha256", MUTATION_MANIFEST_SHA256),
        string_entry(
            "cross_cutting_fixture_proposal_manifest_sha256",
            CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256,
        ),
        string_entry(
            "cross_cutting_executable_fixture_catalog_sha256",
            CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256,
        ),
        string_entry(
            "fixture_inventory_status",
            "cross_cutting_materialized_unreviewed_freeze_blocker",
        ),
        (
            "unresolved_cross_cutting_fixture_classes".to_owned(),
            strict_json::strings(UNRESOLVED_CROSS_CUTTING_FIXTURE_CLASSES),
        ),
        (
            "protocol_gaps".to_owned(),
            strict_json::strings(PROTOCOL_GAPS),
        ),
        ("input_bindings".to_owned(), input_bindings_value()),
        ("budgets".to_owned(), budget_value()),
        ("execution".to_owned(), execution_value()),
        ("selection".to_owned(), JsonValue::Null),
        ("conclusion".to_owned(), JsonValue::Null),
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

fn cross_cutting_fixture_proposal_manifest_value() -> JsonValue {
    let mut proposals = Vec::with_capacity(CROSS_CUTTING_PROPOSAL_COUNT);
    let all_cases = CASES.map(|case| case.as_str());
    for (id, relationship) in MISSING_EDGE_PROPOSAL_IDS.into_iter().zip(RELATIONSHIPS) {
        proposals.push(cross_cutting_fixture_proposal_value(
            id,
            "missing-edge",
            &all_cases,
            &[relationship],
            "remove_required_relationship_descriptor",
            relationship,
            "rejected",
        ));
    }
    for proposal in IDENTITY_SUBSTITUTION_PROPOSALS {
        proposals.push(cross_cutting_fixture_proposal_value(
            proposal.id,
            "identity-substitution",
            &all_cases,
            &RELATIONSHIPS,
            "substitute_bound_identity",
            proposal.target,
            "rejected",
        ));
    }
    for proposal in CASE_SCOPED_CROSS_CUTTING_PROPOSALS {
        proposals.push(cross_cutting_fixture_proposal_value(
            proposal.id,
            proposal.class,
            &[proposal.case.as_str()],
            proposal.relationship_scope,
            proposal.mutation_kind,
            proposal.target,
            proposal.expected_state,
        ));
    }
    debug_assert_eq!(proposals.len(), CROSS_CUTTING_PROPOSAL_COUNT);
    strict_json::object([
        string_entry(
            "schema_version",
            "d004-cross-cutting-fixture-proposals-v0.2",
        ),
        string_entry("status", "draft_unreviewed"),
        string_entry("owner_protocol_review", "none"),
        string_entry("executable_inputs_status", "absent"),
        ("class_statuses".to_owned(), proposal_class_statuses_value()),
        ("replay_repetitions".to_owned(), JsonValue::Null),
        string_entry("evidence_status", "none"),
        ("proposals".to_owned(), JsonValue::Array(proposals)),
        (
            "nonclaims".to_owned(),
            strict_json::strings(CROSS_CUTTING_PROPOSAL_NONCLAIMS),
        ),
    ])
}

fn cross_cutting_fixture_proposal_value(
    id: &'static str,
    class: &'static str,
    case_scope: &[&'static str],
    relationship_scope: &[&'static str],
    mutation_kind: &'static str,
    target: &'static str,
    expected_state: &'static str,
) -> JsonValue {
    strict_json::object([
        string_entry("id", id),
        string_entry("class", class),
        (
            "case_scope".to_owned(),
            strict_json::strings(case_scope.iter().copied()),
        ),
        (
            "relationship_scope".to_owned(),
            strict_json::strings(relationship_scope.iter().copied()),
        ),
        string_entry("layer", "structural"),
        string_entry("mutation_kind", mutation_kind),
        string_entry("target", target),
        string_entry("expected_state", expected_state),
        string_entry("required_invalidation", "dependent_result"),
        string_entry("match_rule", "required_not_sufficient"),
        string_entry("capability_credit", "none"),
        string_entry("observation_level", "domain"),
    ])
}

fn proposal_class_statuses_value() -> JsonValue {
    JsonValue::Array(
        CROSS_CUTTING_PROPOSAL_CLASS_STATUSES
            .iter()
            .map(|status| {
                strict_json::object([
                    string_entry("class", status.class),
                    usize_entry("proposal_count", status.proposal_count),
                    string_entry("proposal_status", "draft_unreviewed"),
                    usize_entry("executable_fixture_count", 0),
                    string_entry("coverage_status", "unresolved"),
                    ("freeze_blocker".to_owned(), JsonValue::Bool(true)),
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
        usize_entry("case_wall_seconds", BUDGETS.case_wall_seconds),
        u64_entry("case_peak_memory_bytes", BUDGETS.case_peak_memory_bytes),
        u64_entry("case_temp_storage_bytes", BUDGETS.case_temp_storage_bytes),
        u64_entry("case_output_bytes", BUDGETS.case_output_bytes),
        usize_entry("candidate_owner_hours", BUDGETS.candidate_owner_hours),
        usize_entry("correction_owner_hours", BUDGETS.correction_owner_hours),
    ])
}

fn execution_value() -> JsonValue {
    strict_json::object([
        usize_entry("required_candidate_cases", REQUIRED_CANDIDATE_CASES),
        usize_entry("completed_candidate_cases", 0),
        usize_entry("complete_candidates", 0),
        usize_entry("complete_cross_candidate_cases", 0),
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

fn u64_entry(key: &str, value: u64) -> (String, JsonValue) {
    let integer = i64::try_from(value).unwrap_or(i64::MAX);
    (key.to_owned(), JsonValue::Integer(integer))
}
