use std::collections::{BTreeMap, BTreeSet};

use super::adapter::{
    CapturedProcess, CapturedTermination, DraftTransportIdentity,
    REQUIRED_DRAFT_TRANSPORT_IDENTITIES,
};
use super::domain::{BUDGETS, CandidateId};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

const RECEIPT_SCHEMA_VERSION: &str = "d005-capture-integrity-receipt-v0.1-draft";
const MAX_IJSON_INTEGER: u64 = 9_007_199_254_740_991;
const RECEIPT_FIELDS: [&str; 14] = [
    "schema_version",
    "capture_slot_ordinal",
    "request_sha256",
    "termination_kind",
    "exit_code",
    "stdout_bytes",
    "stdout_sha256",
    "stderr_bytes",
    "stderr_sha256",
    "stdout_truncated",
    "stderr_truncated",
    "isolation_status",
    "payload_status",
    "evidence_status",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaptureReceiptErrorKind {
    OutputSizeOverflow,
    OutputTooLarge,
    IntegerRange,
    InvalidDigest,
    ReceiptDigestMismatch,
    Json(JsonErrorKind),
    NonCanonical,
    MissingField,
    UnknownField,
    InvalidValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaptureReceiptError {
    pub(crate) kind: CaptureReceiptErrorKind,
    pub(crate) path: String,
}

impl CaptureReceiptError {
    fn new(kind: CaptureReceiptErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

/// Integrity metadata for already-captured synthetic bytes. This record neither
/// authorizes execution nor validates isolation, payload semantics, or evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SyntheticCaptureIntegrityReceipt {
    capture_slot_ordinal: usize,
    request_sha256: String,
    termination: CapturedTermination,
    stdout_bytes: usize,
    stdout_sha256: String,
    stderr_bytes: usize,
    stderr_sha256: String,
    stdout_truncated: bool,
    stderr_truncated: bool,
}

impl SyntheticCaptureIntegrityReceipt {
    pub(crate) const fn capture_slot_ordinal(&self) -> usize {
        self.capture_slot_ordinal
    }

    pub(crate) fn request_sha256(&self) -> &str {
        &self.request_sha256
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        strict_json::canonical_bytes(&self.value())
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical_bytes()))
    }

    fn value(&self) -> JsonValue {
        let (termination_kind, exit_code) = termination_parts(self.termination);
        strict_json::object([
            string_entry("schema_version", RECEIPT_SCHEMA_VERSION),
            usize_entry("capture_slot_ordinal", self.capture_slot_ordinal),
            string_entry("request_sha256", &self.request_sha256),
            string_entry("termination_kind", termination_kind),
            (
                "exit_code".to_owned(),
                exit_code.map_or(JsonValue::Null, |code| JsonValue::Integer(i64::from(code))),
            ),
            usize_entry("stdout_bytes", self.stdout_bytes),
            string_entry("stdout_sha256", &self.stdout_sha256),
            usize_entry("stderr_bytes", self.stderr_bytes),
            string_entry("stderr_sha256", &self.stderr_sha256),
            bool_entry("stdout_truncated", self.stdout_truncated),
            bool_entry("stderr_truncated", self.stderr_truncated),
            string_entry("isolation_status", "not_evaluated"),
            string_entry("payload_status", "unvalidated"),
            string_entry("evidence_status", "none"),
        ])
    }
}

pub(crate) fn create_synthetic_capture_integrity_receipt(
    identity: &DraftTransportIdentity,
    capture: &CapturedProcess,
) -> Result<SyntheticCaptureIntegrityReceipt, CaptureReceiptError> {
    let total_bytes = capture
        .stdout
        .len()
        .checked_add(capture.stderr.len())
        .ok_or_else(|| {
            CaptureReceiptError::new(CaptureReceiptErrorKind::OutputSizeOverflow, "$/capture")
        })?;
    if total_bytes > BUDGETS.max_output_bytes {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::OutputTooLarge,
            "$/capture",
        ));
    }
    for (value, path) in [
        (identity.capture_slot_ordinal(), "$/capture_slot_ordinal"),
        (capture.stdout.len(), "$/stdout_bytes"),
        (capture.stderr.len(), "$/stderr_bytes"),
    ] {
        if !u64::try_from(value).is_ok_and(|value| value <= MAX_IJSON_INTEGER) {
            return Err(CaptureReceiptError::new(
                CaptureReceiptErrorKind::IntegerRange,
                path,
            ));
        }
    }

    Ok(SyntheticCaptureIntegrityReceipt {
        capture_slot_ordinal: identity.capture_slot_ordinal(),
        request_sha256: identity.request().digest_hex(),
        termination: capture.termination,
        stdout_bytes: capture.stdout.len(),
        stdout_sha256: sha256::hex(&sha256::digest(&capture.stdout)),
        stderr_bytes: capture.stderr.len(),
        stderr_sha256: sha256::hex(&sha256::digest(&capture.stderr)),
        stdout_truncated: capture.stdout_truncated,
        stderr_truncated: capture.stderr_truncated,
    })
}

pub(crate) fn parse_and_verify_synthetic_capture_integrity_receipt(
    source: &[u8],
    expected_receipt_sha256: &str,
    identity: &DraftTransportIdentity,
    capture: &CapturedProcess,
) -> Result<SyntheticCaptureIntegrityReceipt, CaptureReceiptError> {
    if !is_sha256_hex(expected_receipt_sha256) {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::InvalidDigest,
            "$/expected_receipt_sha256",
        ));
    }
    if source.len() > BUDGETS.max_packet_bytes {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::Json(JsonErrorKind::InputTooLarge),
            "$/receipt@0",
        ));
    }
    if sha256::hex(&sha256::digest(source)) != expected_receipt_sha256 {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::ReceiptDigestMismatch,
            "$/receipt_sha256",
        ));
    }
    let value = strict_json::parse(source).map_err(|error| {
        CaptureReceiptError::new(
            CaptureReceiptErrorKind::Json(error.kind),
            format!("$/receipt@{}", error.offset),
        )
    })?;
    if strict_json::canonical_bytes(&value) != source {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::NonCanonical,
            "$/receipt",
        ));
    }
    let expected = create_synthetic_capture_integrity_receipt(identity, capture)?;
    validate_receipt_value(&value, &expected)?;
    Ok(expected)
}

fn validate_receipt_value(
    value: &JsonValue,
    expected: &SyntheticCaptureIntegrityReceipt,
) -> Result<(), CaptureReceiptError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &RECEIPT_FIELDS, "$")?;
    require_exact_string(root, "schema_version", RECEIPT_SCHEMA_VERSION)?;
    require_exact_usize(root, "capture_slot_ordinal", expected.capture_slot_ordinal)?;
    require_exact_string(root, "request_sha256", &expected.request_sha256)?;
    let (termination_kind, exit_code) = termination_parts(expected.termination);
    require_exact_string(root, "termination_kind", termination_kind)?;
    require_exact_exit_code(root, exit_code)?;
    require_exact_usize(root, "stdout_bytes", expected.stdout_bytes)?;
    require_exact_string(root, "stdout_sha256", &expected.stdout_sha256)?;
    require_exact_usize(root, "stderr_bytes", expected.stderr_bytes)?;
    require_exact_string(root, "stderr_sha256", &expected.stderr_sha256)?;
    require_exact_bool(root, "stdout_truncated", expected.stdout_truncated)?;
    require_exact_bool(root, "stderr_truncated", expected.stderr_truncated)?;
    require_exact_string(root, "isolation_status", "not_evaluated")?;
    require_exact_string(root, "payload_status", "unvalidated")?;
    require_exact_string(root, "evidence_status", "none")?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CaptureInventoryErrorKind {
    IdentityCardinality,
    IdentityOrder,
    ObservationCardinality,
    DuplicateObservation,
    CrossSlotObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaptureInventoryError {
    pub(crate) kind: CaptureInventoryErrorKind,
    pub(crate) path: String,
}

impl CaptureInventoryError {
    fn new(kind: CaptureInventoryErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

/// Exact in-memory receipt inventory. Its count is integrity bookkeeping only;
/// completed candidate cases and D-005 evidence remain zero.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftCaptureObservationInventory {
    observations: Vec<SyntheticCaptureIntegrityReceipt>,
}

impl DraftCaptureObservationInventory {
    pub(crate) fn observations(&self) -> &[SyntheticCaptureIntegrityReceipt] {
        &self.observations
    }

    pub(crate) const fn completed_candidate_cases(&self) -> usize {
        0
    }

    pub(crate) const fn evidence_status(&self) -> &'static str {
        "none"
    }

    pub(crate) const fn selection(&self) -> Option<CandidateId> {
        None
    }
}

pub(crate) fn bind_draft_capture_observation_inventory(
    identities: &[DraftTransportIdentity],
    observations: Vec<SyntheticCaptureIntegrityReceipt>,
) -> Result<DraftCaptureObservationInventory, CaptureInventoryError> {
    if identities.len() != REQUIRED_DRAFT_TRANSPORT_IDENTITIES {
        return Err(CaptureInventoryError::new(
            CaptureInventoryErrorKind::IdentityCardinality,
            "$/identities",
        ));
    }
    let mut identity_keys = BTreeSet::new();
    for (index, identity) in identities.iter().enumerate() {
        let key = (
            identity.capture_slot_ordinal(),
            identity.request().digest_hex(),
        );
        if identity.capture_slot_ordinal() != index + 1 || !identity_keys.insert(key) {
            return Err(CaptureInventoryError::new(
                CaptureInventoryErrorKind::IdentityOrder,
                format!("$/identities/{index}"),
            ));
        }
    }
    if observations.len() != REQUIRED_DRAFT_TRANSPORT_IDENTITIES {
        return Err(CaptureInventoryError::new(
            CaptureInventoryErrorKind::ObservationCardinality,
            "$/observations",
        ));
    }
    let mut observation_keys = BTreeSet::new();
    for (index, observation) in observations.iter().enumerate() {
        let key = (
            observation.capture_slot_ordinal(),
            observation.request_sha256().to_owned(),
        );
        if !observation_keys.insert(key) {
            return Err(CaptureInventoryError::new(
                CaptureInventoryErrorKind::DuplicateObservation,
                format!("$/observations/{index}"),
            ));
        }
    }
    for (index, (identity, observation)) in identities.iter().zip(&observations).enumerate() {
        if observation.capture_slot_ordinal() != identity.capture_slot_ordinal()
            || observation.request_sha256() != identity.request().digest_hex()
        {
            return Err(CaptureInventoryError::new(
                CaptureInventoryErrorKind::CrossSlotObservation,
                format!("$/observations/{index}"),
            ));
        }
    }
    Ok(DraftCaptureObservationInventory { observations })
}

fn termination_parts(termination: CapturedTermination) -> (&'static str, Option<i32>) {
    match termination {
        CapturedTermination::Exited(code) => ("exited", Some(code)),
        CapturedTermination::Signaled => ("signaled", None),
        CapturedTermination::TimedOut => ("timed_out", None),
        CapturedTermination::StdoutLimit => ("stdout_limit", None),
        CapturedTermination::StderrLimit => ("stderr_limit", None),
        CapturedTermination::SpawnFailed => ("spawn_failed", None),
        CapturedTermination::IoFailed => ("io_failed", None),
        CapturedTermination::UnsupportedSandbox => ("unsupported_sandbox", None),
    }
}

fn require_object<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, CaptureReceiptError> {
    value
        .as_object()
        .ok_or_else(|| CaptureReceiptError::new(CaptureReceiptErrorKind::InvalidValue, path))
}

fn exact_fields(
    object: &BTreeMap<String, JsonValue>,
    expected: &[&str],
    path: &str,
) -> Result<(), CaptureReceiptError> {
    if let Some(field) = object
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::UnknownField,
            format!("{path}/{}", render_pointer_token(field)),
        ));
    }
    for field in expected {
        if !object.contains_key(*field) {
            return Err(CaptureReceiptError::new(
                CaptureReceiptErrorKind::MissingField,
                format!("{path}/{field}"),
            ));
        }
    }
    Ok(())
}

fn require_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
) -> Result<&'a JsonValue, CaptureReceiptError> {
    object.get(field).ok_or_else(|| {
        CaptureReceiptError::new(CaptureReceiptErrorKind::MissingField, format!("$/{field}"))
    })
}

fn require_exact_string(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &str,
) -> Result<(), CaptureReceiptError> {
    if require_field(object, field)?.as_str() != Some(expected) {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::InvalidValue,
            format!("$/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_usize(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: usize,
) -> Result<(), CaptureReceiptError> {
    let observed = require_field(object, field)?
        .as_integer()
        .and_then(|integer| usize::try_from(integer).ok());
    if observed != Some(expected) {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::InvalidValue,
            format!("$/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_bool(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: bool,
) -> Result<(), CaptureReceiptError> {
    if require_field(object, field)? != &JsonValue::Bool(expected) {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::InvalidValue,
            format!("$/{field}"),
        ));
    }
    Ok(())
}

fn require_exact_exit_code(
    object: &BTreeMap<String, JsonValue>,
    expected: Option<i32>,
) -> Result<(), CaptureReceiptError> {
    let expected = expected.map_or(JsonValue::Null, |code| JsonValue::Integer(i64::from(code)));
    if require_field(object, "exit_code")? != &expected {
        return Err(CaptureReceiptError::new(
            CaptureReceiptErrorKind::InvalidValue,
            "$/exit_code",
        ));
    }
    Ok(())
}

fn render_pointer_token(value: &str) -> String {
    let mut rendered = String::new();
    for character in value.chars() {
        match character {
            '~' => rendered.push_str("~0"),
            '/' => rendered.push_str("~1"),
            '\\' => rendered.extend(character.escape_default()),
            _ if !character.is_ascii_graphic() && character != ' ' => {
                rendered.extend(character.escape_default());
            }
            _ => rendered.push(character),
        }
    }
    rendered
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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
