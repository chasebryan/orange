use std::collections::BTreeMap;

use super::candidate_mappings::{
    CANDIDATE_MAPPING_CATALOG_CANONICAL_SHA256, CANDIDATE_MAPPING_CATALOG_PATH,
    CANDIDATE_MAPPING_CATALOG_RAW_SHA256,
};
use super::case_subjects::{
    CASE_SUBJECT_CATALOG_CANONICAL_SHA256, CASE_SUBJECT_CATALOG_PATH,
    CASE_SUBJECT_CATALOG_RAW_SHA256,
};
use super::domain::BUDGETS;
use super::fixtures::{
    FIXTURE_CATALOG_CANONICAL_SHA256, FIXTURE_CATALOG_PATH, FIXTURE_CATALOG_RAW_SHA256,
};
use super::packet::{CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256, MUTATION_MANIFEST_SHA256};
use super::schedule::{
    REQUIRED_EXECUTION_RECORDS, REQUIRED_REPETITIONS_PER_SLOT, ReviewedExecution,
    repetition_major_execution_schedule,
};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

pub(crate) const OWNER_RECORD_PATH: &str =
    "research/decisions/D-004/d004-v0.6/protocol/d004-pre-01-owner-record.json";
pub(crate) const OWNER_RECORD_RAW_SHA256: &str =
    "cbdfa3e07245a6843100a6b17860ea5dcec39f7f341b194faeee91e2ae585f3c";
pub(crate) const OWNER_RECORD_CANONICAL_IDENTITY_SHA256: &str =
    "587c3ad11bf6e0d3dddc02cd7ba53896f54f8e08ec59ed1ece286e13fe9b0c9d";
pub(crate) const REVIEWED_PROTOCOL_PATH: &str =
    "research/decisions/D-004/d004-v0.6/protocol/reviewed-protocol.json";
pub(crate) const REVIEWED_PROTOCOL_RAW_SHA256: &str =
    "1111889b47edf24e88926bf8fa6770cf84ebcd1abb1d7dc2687dc42e0135fb53";
pub(crate) const REVIEWED_PROTOCOL_CANONICAL_IDENTITY_SHA256: &str =
    "c67c17bdf68eb0619ec7698dd2807912251a0556931fa79b6838a9d5c6a9bd98";
pub(crate) const REVIEWED_REPLAY_PLAN_CANONICAL_IDENTITY_SHA256: &str =
    "a18084768cd05f6fcffdea724e330c0c81c5caac7816213156f4f9967fc5cb1b";
pub(crate) const REVIEWED_REPLAY_PLAN_RAW_SHA256: &str =
    "45632f796c7c08d26e668b277ccaff5679ccb82857732c3b8beead66198a3eb7";

pub(crate) const AUTHORIZATION_SUBJECT_REVISION: &str = "7d09a27369649855ce987c76315271b0d34a20ef";
pub(crate) const DRAFT_PACKET_PATH: &str = "research/decisions/D-004/d004-v0.5-draft-packet.json";
pub(crate) const DRAFT_PACKET_CANONICAL_SHA256: &str =
    "b6df1a38f8a1eb6a80a8864324c21a81cb292d4c48e1981b4547bad41933b340";
pub(crate) const DRAFT_PACKET_RAW_SHA256: &str =
    "ec3a0a593d1dab7a6ace874dae4fd03c1ae0656cf301897ccabf51cb109c4009";

pub(crate) const EXECUTION_IDENTITY_PREIMAGE_FIELDS: [&str; 21] = [
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
];

pub(crate) const DETERMINISTIC_EQUALITY_FIELDS: [&str; 10] = [
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
];

pub(crate) const VARIABLE_RESOURCE_FIELDS: [&str; 3] = [
    "measured_resources.wall_milliseconds",
    "measured_resources.peak_memory_bytes",
    "measured_resources.temp_storage_bytes",
];

pub(crate) const FORBIDDEN_AGGREGATION: [&str; 6] = [
    "warmup",
    "retry_substitution",
    "voting",
    "averaging",
    "best_of",
    "statistical_confidence",
];

pub(crate) const EPOCH_FREEZE_BLOCKERS: [&str; 9] = [
    "candidate adapter implementations and closed request and response schemas absent",
    "candidate semantic models endpoint inventories and parameter bindings absent",
    "exact executable tool and transitive dependency manifests absent",
    "exact deterministic environment and ordered input manifests absent",
    "enforcing isolation launcher cache reset and cleanup verification absent",
    "output normalizer payload validator and bounded resource meter absent",
    "populated result repetition-closure correction and evidence-archive parsers absent",
    "exact execution-subject revision and owner freeze record absent",
    "frozen epoch packet replay-plan identity and scheduled-execution digests absent",
];

const OWNER_ROOT_FIELDS: [&str; 20] = [
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
];
const PROTOCOL_ROOT_FIELDS: [&str; 21] = [
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
];
const PLAN_ROOT_FIELDS: [&str; 17] = [
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
];
const SCHEDULE_FIELDS: [&str; 7] = [
    "candidate",
    "case",
    "execution_ordinal",
    "logical_slot_ordinal",
    "position",
    "repetition",
    "round",
];
const BINDING_FIELDS: [&str; 3] = ["canonical_sha256", "path", "raw_sha256"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewedProtocolErrorKind {
    Json(JsonErrorKind),
    NonCanonical,
    DigestMismatch,
    MissingField,
    UnknownField,
    InvalidValue,
    BindingMismatch,
    ScheduleMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewedProtocolError {
    pub(crate) kind: ReviewedProtocolErrorKind,
    pub(crate) path: String,
}

impl ReviewedProtocolError {
    fn new(kind: ReviewedProtocolErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OwnerProtocolRecord {
    value: JsonValue,
    canonical: Vec<u8>,
}

impl OwnerProtocolRecord {
    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewedProtocol {
    value: JsonValue,
    canonical: Vec<u8>,
}

impl ReviewedProtocol {
    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical))
    }

    pub(crate) const fn execution_authorized(&self) -> bool {
        false
    }

    pub(crate) const fn replay_repetitions(&self) -> usize {
        REQUIRED_REPETITIONS_PER_SLOT
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewedReplayPlan {
    value: JsonValue,
    canonical: Vec<u8>,
    schedule: Vec<ReviewedExecution>,
}

impl ReviewedReplayPlan {
    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical))
    }

    pub(crate) fn schedule(&self) -> &[ReviewedExecution] {
        &self.schedule
    }
}

pub(crate) fn parse_owner_protocol_record(
    source: &[u8],
) -> Result<OwnerProtocolRecord, ReviewedProtocolError> {
    let (value, canonical) = parse_exact_document(
        source,
        OWNER_RECORD_CANONICAL_IDENTITY_SHA256,
        OWNER_RECORD_RAW_SHA256,
        "$/owner",
    )?;
    let root = require_object(&value, "$/owner")?;
    exact_fields(root, &OWNER_ROOT_FIELDS, "$/owner")?;
    require_exact_string(
        root,
        "schema_version",
        "d004-owner-protocol-record-v0.1",
        "$/owner",
    )?;
    require_exact_string(root, "record_id", "D004-PRE-01", "$/owner")?;
    require_exact_string(root, "accepted_subject", "D004-PRE-01", "$/owner")?;
    require_exact_string(root, "decision_id", "D-004", "$/owner")?;
    require_exact_string(root, "decision_status", "proposed", "$/owner")?;
    require_exact_string(root, "record_disposition", "accepted", "$/owner")?;
    require_exact_string(
        root,
        "implementation_closure_status",
        "provisional_pending_exact_merged_revision",
        "$/owner",
    )?;
    require_exact_string(
        root,
        "authorization_subject_revision",
        AUTHORIZATION_SUBJECT_REVISION,
        "$/owner",
    )?;
    require_exact_bool(root, "delegated_technical_judgment", true, "$/owner")?;

    let authority = require_object_field(root, "authority", "$/owner")?;
    exact_fields(
        authority,
        &["independent_review", "review_authority", "review_label"],
        "$/owner/authority",
    )?;
    require_exact_string(
        authority,
        "review_authority",
        "Orange Project Owner",
        "$/owner/authority",
    )?;
    require_exact_string(
        authority,
        "review_label",
        "solo-reviewed",
        "$/owner/authority",
    )?;
    require_exact_string(
        authority,
        "independent_review",
        "unavailable",
        "$/owner/authority",
    )?;

    let subjects = require_object_field(root, "review_subjects", "$/owner")?;
    exact_fields(
        subjects,
        &[
            "candidate_mappings",
            "case_subjects",
            "cross_cutting_executable_fixtures",
            "cross_cutting_fixture_proposals",
            "decision_suite",
            "draft_packet",
            "named_mutations",
            "result_contract_descriptor",
        ],
        "$/owner/review_subjects",
    )?;
    require_binding(
        subjects,
        "candidate_mappings",
        CANDIDATE_MAPPING_CATALOG_PATH,
        CANDIDATE_MAPPING_CATALOG_RAW_SHA256,
        CANDIDATE_MAPPING_CATALOG_CANONICAL_SHA256,
        "$/owner/review_subjects",
    )?;
    require_binding(
        subjects,
        "case_subjects",
        CASE_SUBJECT_CATALOG_PATH,
        CASE_SUBJECT_CATALOG_RAW_SHA256,
        CASE_SUBJECT_CATALOG_CANONICAL_SHA256,
        "$/owner/review_subjects",
    )?;
    require_binding(
        subjects,
        "cross_cutting_executable_fixtures",
        FIXTURE_CATALOG_PATH,
        FIXTURE_CATALOG_RAW_SHA256,
        FIXTURE_CATALOG_CANONICAL_SHA256,
        "$/owner/review_subjects",
    )?;
    require_binding(
        subjects,
        "cross_cutting_fixture_proposals",
        "research/decisions/D-004/d004-v0.2-cross-cutting-fixture-proposals.json",
        "d3d58cbeb0d2a90987680cd00bc70caf53518be730a71d0d55ba2a7b50544481",
        CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256,
        "$/owner/review_subjects",
    )?;
    require_binding(
        subjects,
        "draft_packet",
        DRAFT_PACKET_PATH,
        DRAFT_PACKET_RAW_SHA256,
        DRAFT_PACKET_CANONICAL_SHA256,
        "$/owner/review_subjects",
    )?;
    require_binding(
        subjects,
        "named_mutations",
        "research/decisions/D-004/d004-v0.2-named-mutations.json",
        "1d46d6d66c0704fcaa462c625dcac2e72150497bb075322c5e076ea42898be54",
        MUTATION_MANIFEST_SHA256,
        "$/owner/review_subjects",
    )?;

    let disposition = require_object_field(root, "structured_disposition", "$/owner")?;
    validate_mapping_disposition(
        disposition,
        "mapping_review",
        "$/owner/structured_disposition",
    )?;
    validate_fixture_dispositions(
        disposition,
        "fixture_class_reviews",
        "$/owner/structured_disposition",
    )?;
    let replay = require_object_field(
        disposition,
        "replay_policy",
        "$/owner/structured_disposition",
    )?;
    validate_replay_policy(
        replay,
        "$/owner/structured_disposition/replay_policy",
        false,
    )?;

    Ok(OwnerProtocolRecord { value, canonical })
}

pub(crate) fn parse_reviewed_protocol(
    source: &[u8],
    owner: &OwnerProtocolRecord,
) -> Result<ReviewedProtocol, ReviewedProtocolError> {
    if owner.digest_hex() != OWNER_RECORD_CANONICAL_IDENTITY_SHA256 {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::BindingMismatch,
            "$/bindings/owner_record/raw_sha256",
        ));
    }
    let (value, canonical) = parse_exact_document(
        source,
        REVIEWED_PROTOCOL_CANONICAL_IDENTITY_SHA256,
        REVIEWED_PROTOCOL_RAW_SHA256,
        "$/protocol",
    )?;
    let root = require_object(&value, "$/protocol")?;
    exact_fields(root, &PROTOCOL_ROOT_FIELDS, "$/protocol")?;
    require_exact_string(
        root,
        "schema_version",
        "d004-reviewed-protocol-tranche-v0.1",
        "$/protocol",
    )?;
    require_exact_string(root, "status", "reviewed_not_executable", "$/protocol")?;
    require_exact_string(
        root,
        "implementation_closure_status",
        "provisional_pending_exact_merged_revision",
        "$/protocol",
    )?;
    require_exact_string(root, "owner_protocol_review", "solo-reviewed", "$/protocol")?;
    require_null(root, "epoch", "$/protocol")?;
    require_exact_string(root, "epoch_status", "unfrozen", "$/protocol")?;
    require_exact_bool(root, "execution_authorized", false, "$/protocol")?;
    require_null(root, "selection", "$/protocol")?;
    require_null(root, "conclusion", "$/protocol")?;
    require_exact_empty_array(root, "protocol_gaps", "$/protocol")?;
    require_exact_strings(
        root,
        "epoch_freeze_blockers",
        &EPOCH_FREEZE_BLOCKERS,
        "$/protocol",
    )?;

    let bindings = require_object_field(root, "bindings", "$/protocol")?;
    require_binding(
        bindings,
        "owner_record",
        OWNER_RECORD_PATH,
        OWNER_RECORD_RAW_SHA256,
        OWNER_RECORD_CANONICAL_IDENTITY_SHA256,
        "$/protocol/bindings",
    )?;
    require_binding(
        bindings,
        "draft_packet",
        DRAFT_PACKET_PATH,
        DRAFT_PACKET_RAW_SHA256,
        DRAFT_PACKET_CANONICAL_SHA256,
        "$/protocol/bindings",
    )?;
    require_binding(
        bindings,
        "candidate_mappings",
        CANDIDATE_MAPPING_CATALOG_PATH,
        CANDIDATE_MAPPING_CATALOG_RAW_SHA256,
        CANDIDATE_MAPPING_CATALOG_CANONICAL_SHA256,
        "$/protocol/bindings",
    )?;
    require_binding(
        bindings,
        "case_subjects",
        CASE_SUBJECT_CATALOG_PATH,
        CASE_SUBJECT_CATALOG_RAW_SHA256,
        CASE_SUBJECT_CATALOG_CANONICAL_SHA256,
        "$/protocol/bindings",
    )?;
    require_binding(
        bindings,
        "cross_cutting_executable_fixtures",
        FIXTURE_CATALOG_PATH,
        FIXTURE_CATALOG_RAW_SHA256,
        FIXTURE_CATALOG_CANONICAL_SHA256,
        "$/protocol/bindings",
    )?;

    validate_mapping_disposition(root, "mapping_disposition", "$/protocol")?;
    validate_fixture_dispositions(root, "fixture_class_dispositions", "$/protocol")?;
    let replay = require_object_field(root, "replay_contract", "$/protocol")?;
    validate_replay_policy(replay, "$/protocol/replay_contract", true)?;

    let execution = require_object_field(root, "execution", "$/protocol")?;
    for (field, expected) in [
        ("required_candidate_cases", 25),
        ("required_execution_records", REQUIRED_EXECUTION_RECORDS),
        ("result_record_count", 0),
        ("completed_candidate_cases", 0),
        ("complete_candidates", 0),
        ("complete_cross_candidate_cases", 0),
    ] {
        require_exact_usize(execution, field, expected, "$/protocol/execution")?;
    }
    require_exact_string(execution, "evidence_status", "none", "$/protocol/execution")?;

    Ok(ReviewedProtocol { value, canonical })
}

pub(crate) fn parse_reviewed_replay_plan(
    source: &[u8],
    owner: &OwnerProtocolRecord,
    protocol: &ReviewedProtocol,
) -> Result<ReviewedReplayPlan, ReviewedProtocolError> {
    if owner.digest_hex() != OWNER_RECORD_CANONICAL_IDENTITY_SHA256
        || protocol.digest_hex() != REVIEWED_PROTOCOL_CANONICAL_IDENTITY_SHA256
    {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::BindingMismatch,
            "$/plan/bindings",
        ));
    }
    let (value, canonical) = parse_exact_document(
        source,
        REVIEWED_REPLAY_PLAN_CANONICAL_IDENTITY_SHA256,
        REVIEWED_REPLAY_PLAN_RAW_SHA256,
        "$/plan",
    )?;
    let root = require_object(&value, "$/plan")?;
    exact_fields(root, &PLAN_ROOT_FIELDS, "$/plan")?;
    require_exact_string(root, "schema_version", "d004-replay-plan-v0.1", "$/plan")?;
    require_exact_string(root, "status", "reviewed_uninstantiated", "$/plan")?;
    require_exact_string(
        root,
        "implementation_closure_status",
        "provisional_pending_exact_merged_revision",
        "$/plan",
    )?;
    require_null(root, "epoch", "$/plan")?;
    require_exact_usize(
        root,
        "physical_execution_count",
        REQUIRED_EXECUTION_RECORDS,
        "$/plan",
    )?;
    require_exact_usize(
        root,
        "repetitions_per_slot",
        REQUIRED_REPETITIONS_PER_SLOT,
        "$/plan",
    )?;
    require_exact_string(
        root,
        "physical_order",
        "repetition_major_then_latin_slot_ordinal",
        "$/plan",
    )?;
    require_exact_strings(
        root,
        "execution_identity_preimage_fields",
        &EXECUTION_IDENTITY_PREIMAGE_FIELDS,
        "$/plan",
    )?;
    require_binding(
        root,
        "owner_record",
        OWNER_RECORD_PATH,
        OWNER_RECORD_RAW_SHA256,
        OWNER_RECORD_CANONICAL_IDENTITY_SHA256,
        "$/plan",
    )?;
    require_binding(
        root,
        "protocol",
        REVIEWED_PROTOCOL_PATH,
        REVIEWED_PROTOCOL_RAW_SHA256,
        REVIEWED_PROTOCOL_CANONICAL_IDENTITY_SHA256,
        "$/plan",
    )?;
    require_binding(
        root,
        "base_packet",
        DRAFT_PACKET_PATH,
        DRAFT_PACKET_RAW_SHA256,
        DRAFT_PACKET_CANONICAL_SHA256,
        "$/plan",
    )?;

    let expected = repetition_major_execution_schedule();
    let observed = require_array_field(root, "schedule", "$/plan")?;
    if observed.len() != expected.len() {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::ScheduleMismatch,
            "$/plan/schedule",
        ));
    }
    for (index, (record, expected)) in observed.iter().zip(&expected).enumerate() {
        validate_schedule_record(record, *expected, index)?;
    }

    Ok(ReviewedReplayPlan {
        value,
        canonical,
        schedule: expected,
    })
}

fn validate_schedule_record(
    value: &JsonValue,
    expected: ReviewedExecution,
    index: usize,
) -> Result<(), ReviewedProtocolError> {
    let path = format!("$/plan/schedule/{index}");
    let record = require_object(value, &path)?;
    exact_fields(record, &SCHEDULE_FIELDS, &path)?;
    for (field, value) in [
        ("execution_ordinal", expected.execution_ordinal),
        ("repetition", expected.repetition),
        ("logical_slot_ordinal", expected.logical_slot_ordinal),
        ("round", expected.round),
        ("position", expected.position),
    ] {
        require_exact_usize(record, field, value, &path)?;
    }
    require_exact_string(record, "candidate", expected.candidate.as_str(), &path)?;
    require_exact_string(record, "case", expected.case.as_str(), &path)?;
    Ok(())
}

fn validate_mapping_disposition(
    parent: &BTreeMap<String, JsonValue>,
    field: &str,
    parent_path: &str,
) -> Result<(), ReviewedProtocolError> {
    let mapping = require_object_field(parent, field, parent_path)?;
    let path = format!("{parent_path}/{field}");
    for (name, expected) in [
        ("candidate_graph_count", 5),
        ("mapping_row_count", 70),
        ("relationship_count", 14),
    ] {
        require_exact_usize(mapping, name, expected, &path)?;
    }
    require_exact_string(
        mapping,
        "status",
        "reviewed_symmetric_falsifiable_test_hypotheses",
        &path,
    )?;
    require_exact_string(mapping, "semantic_status", "unaccepted", &path)?;
    require_exact_string(mapping, "capability_credit", "none", &path)?;
    require_exact_string(mapping, "evidence_credit", "none", &path)?;
    require_null(mapping, "selection", &path)?;
    Ok(())
}

fn validate_fixture_dispositions(
    parent: &BTreeMap<String, JsonValue>,
    field: &str,
    parent_path: &str,
) -> Result<(), ReviewedProtocolError> {
    let path = format!("{parent_path}/{field}");
    let rows = require_array_field(parent, field, parent_path)?;
    let expected = [
        ("ambiguity", 5),
        ("missing-edge", 14),
        ("identity-substitution", 13),
        ("unsupported", 5),
        ("resource-exhaustion", 5),
    ];
    if rows.len() != expected.len() {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::InvalidValue,
            path,
        ));
    }
    for (index, (row, (class, count))) in rows.iter().zip(expected).enumerate() {
        let row_path = format!("{path}/{index}");
        let row = require_object(row, &row_path)?;
        exact_fields(
            row,
            &[
                "capability_credit",
                "class",
                "scope",
                "status",
                "subject_count",
            ],
            &row_path,
        )?;
        require_exact_string(row, "class", class, &row_path)?;
        require_exact_usize(row, "subject_count", count, &row_path)?;
        require_exact_string(row, "status", "reviewed_sufficient", &row_path)?;
        require_exact_string(row, "scope", "bounded_suite_coverage_only", &row_path)?;
        require_exact_string(row, "capability_credit", "none", &row_path)?;
    }
    Ok(())
}

fn validate_replay_policy(
    replay: &BTreeMap<String, JsonValue>,
    path: &str,
    full_contract: bool,
) -> Result<(), ReviewedProtocolError> {
    for (field, expected) in [
        ("base_candidate_case_slots", 25),
        ("repetitions_per_slot", REQUIRED_REPETITIONS_PER_SLOT),
        ("required_execution_records", REQUIRED_EXECUTION_RECORDS),
    ] {
        require_exact_usize(replay, field, expected, path)?;
    }
    require_exact_string(
        replay,
        "physical_order",
        "repetition_major_then_latin_slot_ordinal",
        path,
    )?;
    require_exact_string(
        replay,
        "cache",
        "fresh_empty_candidate_specific_per_execution",
        path,
    )?;
    require_exact_strings(
        replay,
        "forbidden_aggregation",
        &FORBIDDEN_AGGREGATION,
        path,
    )?;
    if full_contract {
        require_exact_strings(
            replay,
            "execution_identity_preimage_fields",
            &EXECUTION_IDENTITY_PREIMAGE_FIELDS,
            path,
        )?;
        let mut equality = DETERMINISTIC_EQUALITY_FIELDS.to_vec();
        equality.push("measured_resources_stdout_bytes_and_stderr_bytes");
        require_exact_strings(replay, "deterministic_equality_fields", &equality, path)?;
        require_exact_strings(
            replay,
            "variable_fields_within_frozen_bounds",
            &VARIABLE_RESOURCE_FIELDS,
            path,
        )?;
    } else {
        require_exact_bool(replay, "deterministic_equality_required", true, path)?;
        require_exact_bool(replay, "independent_pass_required", true, path)?;
    }
    Ok(())
}

fn parse_exact_document(
    source: &[u8],
    expected_canonical_sha256: &str,
    expected_raw_sha256: &str,
    path: &str,
) -> Result<(JsonValue, Vec<u8>), ReviewedProtocolError> {
    let value = strict_json::parse(source).map_err(|error| {
        ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::Json(error.kind),
            format!("{path}@{}", error.offset),
        )
    })?;
    let canonical = strict_json::canonical_bytes(&value);
    let mut canonical_transport = canonical.clone();
    canonical_transport.push(b'\n');
    if canonical_transport != source {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::NonCanonical,
            path,
        ));
    }
    if sha256::hex(&sha256::digest(&canonical)) != expected_canonical_sha256
        || sha256::hex(&sha256::digest(source)) != expected_raw_sha256
    {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::DigestMismatch,
            format!("{path}_sha256"),
        ));
    }
    Ok((value, canonical))
}

fn require_binding(
    parent: &BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
    raw_sha256: &str,
    canonical_sha256: &str,
    parent_path: &str,
) -> Result<(), ReviewedProtocolError> {
    let binding_path = format!("{parent_path}/{field}");
    let binding = require_object_field(parent, field, parent_path)?;
    exact_fields(binding, &BINDING_FIELDS, &binding_path)?;
    require_exact_string(binding, "path", path, &binding_path)?;
    require_exact_string(binding, "raw_sha256", raw_sha256, &binding_path)?;
    require_exact_string(binding, "canonical_sha256", canonical_sha256, &binding_path)?;
    Ok(())
}

fn require_object<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, ReviewedProtocolError> {
    value
        .as_object()
        .ok_or_else(|| ReviewedProtocolError::new(ReviewedProtocolErrorKind::InvalidValue, path))
}

fn require_object_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, ReviewedProtocolError> {
    require_object(
        require_field(object, field, path)?,
        &format!("{path}/{field}"),
    )
}

fn require_array_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<&'a [JsonValue], ReviewedProtocolError> {
    require_field(object, field, path)?
        .as_array()
        .ok_or_else(|| {
            ReviewedProtocolError::new(
                ReviewedProtocolErrorKind::InvalidValue,
                format!("{path}/{field}"),
            )
        })
}

fn require_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<&'a JsonValue, ReviewedProtocolError> {
    object.get(field).ok_or_else(|| {
        ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::MissingField,
            format!("{path}/{field}"),
        )
    })
}

fn exact_fields(
    object: &BTreeMap<String, JsonValue>,
    expected: &[&str],
    path: &str,
) -> Result<(), ReviewedProtocolError> {
    if let Some(field) = object
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::UnknownField,
            format!("{path}/{field}"),
        ));
    }
    for field in expected {
        if !object.contains_key(*field) {
            return Err(ReviewedProtocolError::new(
                ReviewedProtocolErrorKind::MissingField,
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
    path: &str,
) -> Result<(), ReviewedProtocolError> {
    if require_field(object, field, path)?.as_str() != Some(expected) {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::InvalidValue,
            format!("{path}/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_usize(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: usize,
    path: &str,
) -> Result<(), ReviewedProtocolError> {
    let observed = require_field(object, field, path)?.as_integer();
    if observed.and_then(|value| usize::try_from(value).ok()) != Some(expected) {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::InvalidValue,
            format!("{path}/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_bool(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: bool,
    path: &str,
) -> Result<(), ReviewedProtocolError> {
    if require_field(object, field, path)? != &JsonValue::Bool(expected) {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::InvalidValue,
            format!("{path}/{field}"),
        ));
    }
    Ok(())
}

fn require_null(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<(), ReviewedProtocolError> {
    if require_field(object, field, path)? != &JsonValue::Null {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::InvalidValue,
            format!("{path}/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_empty_array(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<(), ReviewedProtocolError> {
    if !require_array_field(object, field, path)?.is_empty() {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::InvalidValue,
            format!("{path}/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_strings(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &[&str],
    path: &str,
) -> Result<(), ReviewedProtocolError> {
    let observed = require_array_field(object, field, path)?;
    if observed.len() != expected.len()
        || observed
            .iter()
            .zip(expected)
            .any(|(observed, expected)| observed.as_str() != Some(*expected))
    {
        return Err(ReviewedProtocolError::new(
            ReviewedProtocolErrorKind::InvalidValue,
            format!("{path}/{field}"),
        ));
    }
    Ok(())
}

pub(crate) fn resource_ceilings() -> (u64, u64, u64, u64) {
    (
        u64::try_from(BUDGETS.case_wall_seconds).expect("bounded seconds") * 1_000,
        BUDGETS.case_peak_memory_bytes,
        BUDGETS.case_temp_storage_bytes,
        BUDGETS.case_output_bytes,
    )
}
