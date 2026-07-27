use super::domain::{CandidateId, CaseId, REQUIRED_CANDIDATE_CASES};
use super::reviewed_protocol::{
    DETERMINISTIC_EQUALITY_FIELDS, EPOCH_FREEZE_BLOCKERS, EXECUTION_IDENTITY_PREIMAGE_FIELDS,
    REVIEWED_PROTOCOL_CANONICAL_IDENTITY_SHA256, REVIEWED_PROTOCOL_RAW_SHA256,
    REVIEWED_REPLAY_PLAN_CANONICAL_IDENTITY_SHA256, REVIEWED_REPLAY_PLAN_RAW_SHA256,
    ReviewedProtocol, ReviewedReplayPlan, VARIABLE_RESOURCE_FIELDS, resource_ceilings,
};
use super::schedule::{
    REQUIRED_EXECUTION_RECORDS, REQUIRED_REPETITIONS_PER_SLOT, latin_base_schedule,
};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

pub(crate) const REVIEWED_RESULT_CONTRACT_SCHEMA_VERSION: &str =
    "d004-reviewed-result-contract-descriptor-v0.1";

pub(crate) const REVIEWED_RESULT_CONTRACT_NONCLAIMS: [&str; 8] = [
    "reviewed schema only; no populated result record exists",
    "no candidate adapter process or tool invoked",
    "no D-004 evidence epoch frozen or candidate execution authorized",
    "no candidate graph or SR mapping accepted as Orange semantics",
    "no semantic-strata candidate selected preferred or accepted",
    "no D-004 disposition accepted",
    "no S3b implementation authorized",
    "no roadmap gate or readiness movement",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepetitionRecordSummary {
    pub(crate) logical_slot_ordinal: usize,
    pub(crate) candidate: CandidateId,
    pub(crate) case: CaseId,
    pub(crate) repetition: usize,
    pub(crate) independently_passed: bool,
    /// Digest of the closed deterministic-equality projection enumerated by
    /// `DETERMINISTIC_EQUALITY_FIELDS`, including stdout/stderr identities.
    pub(crate) deterministic_fields_sha256: String,
    pub(crate) stdout_bytes: u64,
    pub(crate) stderr_bytes: u64,
    pub(crate) wall_milliseconds: u64,
    pub(crate) peak_memory_bytes: u64,
    pub(crate) temp_storage_bytes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RepetitionClosureErrorKind {
    Cardinality,
    RepetitionOrder,
    CoordinateMismatch,
    InvalidDigest,
    IndependentFailure,
    DeterministicMismatch,
    ResourceLimit,
    OutputSizeOverflow,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepetitionClosureError {
    pub(crate) kind: RepetitionClosureErrorKind,
    pub(crate) path: String,
}

impl RepetitionClosureError {
    fn new(kind: RepetitionClosureErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RepetitionClosure {
    logical_slot_ordinal: usize,
    candidate: CandidateId,
    case: CaseId,
    deterministic_fields_sha256: String,
}

impl RepetitionClosure {
    pub(crate) const fn logical_slot_ordinal(&self) -> usize {
        self.logical_slot_ordinal
    }

    pub(crate) const fn candidate(&self) -> CandidateId {
        self.candidate
    }

    pub(crate) const fn case(&self) -> CaseId {
        self.case
    }

    pub(crate) fn deterministic_fields_sha256(&self) -> &str {
        &self.deterministic_fields_sha256
    }
}

pub(crate) fn validate_repetition_closure(
    records: &[RepetitionRecordSummary],
) -> Result<RepetitionClosure, RepetitionClosureError> {
    if records.len() != REQUIRED_REPETITIONS_PER_SLOT {
        return Err(RepetitionClosureError::new(
            RepetitionClosureErrorKind::Cardinality,
            "$/repetitions",
        ));
    }
    let first = &records[0];
    if !(1..=REQUIRED_CANDIDATE_CASES).contains(&first.logical_slot_ordinal) {
        return Err(RepetitionClosureError::new(
            RepetitionClosureErrorKind::CoordinateMismatch,
            "$/repetitions/0/logical_slot_ordinal",
        ));
    }
    let expected_slot = &latin_base_schedule()[first.logical_slot_ordinal - 1];
    if first.candidate != expected_slot.candidate || first.case != expected_slot.case {
        return Err(RepetitionClosureError::new(
            RepetitionClosureErrorKind::CoordinateMismatch,
            "$/repetitions/0/scheduled_coordinate",
        ));
    }
    let (max_wall_milliseconds, max_peak_memory, max_temp_storage, max_output) =
        resource_ceilings();
    for (index, record) in records.iter().enumerate() {
        let path = format!("$/repetitions/{index}");
        if record.repetition != index + 1 {
            return Err(RepetitionClosureError::new(
                RepetitionClosureErrorKind::RepetitionOrder,
                format!("{path}/repetition"),
            ));
        }
        if record.logical_slot_ordinal != first.logical_slot_ordinal
            || record.candidate != first.candidate
            || record.case != first.case
        {
            return Err(RepetitionClosureError::new(
                RepetitionClosureErrorKind::CoordinateMismatch,
                path,
            ));
        }
        if !is_sha256_hex(&record.deterministic_fields_sha256) {
            return Err(RepetitionClosureError::new(
                RepetitionClosureErrorKind::InvalidDigest,
                format!("{path}/deterministic_fields_sha256"),
            ));
        }
        if !record.independently_passed {
            return Err(RepetitionClosureError::new(
                RepetitionClosureErrorKind::IndependentFailure,
                format!("{path}/independently_passed"),
            ));
        }
        if record.deterministic_fields_sha256 != first.deterministic_fields_sha256
            || record.stdout_bytes != first.stdout_bytes
            || record.stderr_bytes != first.stderr_bytes
        {
            return Err(RepetitionClosureError::new(
                RepetitionClosureErrorKind::DeterministicMismatch,
                path,
            ));
        }
        let output_bytes = record
            .stdout_bytes
            .checked_add(record.stderr_bytes)
            .ok_or_else(|| {
                RepetitionClosureError::new(
                    RepetitionClosureErrorKind::OutputSizeOverflow,
                    format!("{path}/measured_resources"),
                )
            })?;
        if record.wall_milliseconds > max_wall_milliseconds
            || record.peak_memory_bytes > max_peak_memory
            || record.temp_storage_bytes > max_temp_storage
            || output_bytes > max_output
        {
            return Err(RepetitionClosureError::new(
                RepetitionClosureErrorKind::ResourceLimit,
                format!("{path}/measured_resources"),
            ));
        }
    }
    Ok(RepetitionClosure {
        logical_slot_ordinal: first.logical_slot_ordinal,
        candidate: first.candidate,
        case: first.case,
        deterministic_fields_sha256: first.deterministic_fields_sha256.clone(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReviewedResultContractErrorKind {
    Json(JsonErrorKind),
    NonCanonical,
    SchemaMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewedResultContractError {
    pub(crate) kind: ReviewedResultContractErrorKind,
    pub(crate) path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewedResultContractDescriptor {
    canonical: Vec<u8>,
}

impl ReviewedResultContractDescriptor {
    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical))
    }

    pub(crate) const fn result_record_count(&self) -> usize {
        0
    }

    pub(crate) const fn execution_authorized(&self) -> bool {
        false
    }
}

pub(crate) fn canonical_reviewed_result_contract_descriptor_bytes(
    protocol: &ReviewedProtocol,
    plan: &ReviewedReplayPlan,
) -> Vec<u8> {
    strict_json::canonical_bytes(&descriptor_value(protocol, plan))
}

pub(crate) fn parse_reviewed_result_contract_descriptor(
    source: &[u8],
    protocol: &ReviewedProtocol,
    plan: &ReviewedReplayPlan,
) -> Result<ReviewedResultContractDescriptor, ReviewedResultContractError> {
    let value = strict_json::parse(source).map_err(|error| ReviewedResultContractError {
        kind: ReviewedResultContractErrorKind::Json(error.kind),
        path: format!("$/descriptor@{}", error.offset),
    })?;
    if strict_json::canonical_bytes(&value) != source {
        return Err(ReviewedResultContractError {
            kind: ReviewedResultContractErrorKind::NonCanonical,
            path: "$/descriptor".to_owned(),
        });
    }
    if value != descriptor_value(protocol, plan) {
        return Err(ReviewedResultContractError {
            kind: ReviewedResultContractErrorKind::SchemaMismatch,
            path: "$/descriptor".to_owned(),
        });
    }
    Ok(ReviewedResultContractDescriptor {
        canonical: source.to_vec(),
    })
}

fn descriptor_value(protocol: &ReviewedProtocol, plan: &ReviewedReplayPlan) -> JsonValue {
    assert_eq!(
        protocol.digest_hex(),
        REVIEWED_PROTOCOL_CANONICAL_IDENTITY_SHA256
    );
    assert_eq!(
        plan.digest_hex(),
        REVIEWED_REPLAY_PLAN_CANONICAL_IDENTITY_SHA256
    );
    let mut equality_fields = DETERMINISTIC_EQUALITY_FIELDS.to_vec();
    equality_fields.push("measured_resources_stdout_bytes_and_stderr_bytes");
    strict_json::object([
        string_entry("schema_version", REVIEWED_RESULT_CONTRACT_SCHEMA_VERSION),
        string_entry("status", "reviewed_schema_not_executable"),
        ("epoch".to_owned(), JsonValue::Null),
        string_entry("epoch_status", "unfrozen"),
        string_entry("owner_protocol_review", "solo-reviewed"),
        string_entry("protocol_raw_sha256", REVIEWED_PROTOCOL_RAW_SHA256),
        string_entry(
            "protocol_canonical_identity_sha256",
            REVIEWED_PROTOCOL_CANONICAL_IDENTITY_SHA256,
        ),
        string_entry("replay_plan_raw_sha256", REVIEWED_REPLAY_PLAN_RAW_SHA256),
        usize_entry("base_candidate_case_slots", REQUIRED_CANDIDATE_CASES),
        usize_entry("repetitions_per_slot", REQUIRED_REPETITIONS_PER_SLOT),
        usize_entry("required_execution_records", REQUIRED_EXECUTION_RECORDS),
        string_entry("physical_order", "repetition_major_then_latin_slot_ordinal"),
        strings_entry(
            "execution_identity_preimage_fields",
            &EXECUTION_IDENTITY_PREIMAGE_FIELDS,
        ),
        (
            "deterministic_equality_fields".to_owned(),
            strict_json::strings(equality_fields),
        ),
        strings_entry(
            "variable_fields_within_frozen_bounds",
            &VARIABLE_RESOURCE_FIELDS,
        ),
        string_entry(
            "slot_pass_rule",
            "all_three_repetitions_independently_pass_and_all_deterministic_fields_equal",
        ),
        string_entry(
            "failure_rule",
            "missing_invalid_non_successful_or_inconsistent_repetition_fails_candidate_case",
        ),
        strings_entry("epoch_freeze_blockers", &EPOCH_FREEZE_BLOCKERS),
        (
            "current_state".to_owned(),
            strict_json::object([
                usize_entry("result_record_count", 0),
                usize_entry("completed_candidate_cases", 0),
                string_entry("evidence_status", "none"),
                ("selection".to_owned(), JsonValue::Null),
                ("conclusion".to_owned(), JsonValue::Null),
                ("execution_authorized".to_owned(), JsonValue::Bool(false)),
            ]),
        ),
        ("result_records".to_owned(), JsonValue::Array(Vec::new())),
        strings_entry("nonclaims", &REVIEWED_RESULT_CONTRACT_NONCLAIMS),
    ])
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

fn string_entry(key: &str, value: &str) -> (String, JsonValue) {
    (key.to_owned(), JsonValue::String(value.to_owned()))
}

fn strings_entry<const N: usize>(
    key: &str,
    values: &'static [&'static str; N],
) -> (String, JsonValue) {
    (key.to_owned(), strict_json::strings(*values))
}

fn usize_entry(key: &str, value: usize) -> (String, JsonValue) {
    (
        key.to_owned(),
        JsonValue::Integer(i64::try_from(value).expect("bounded reviewed-protocol integer")),
    )
}
