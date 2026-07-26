use std::collections::BTreeMap;

use super::domain::{
    BUDGETS, CandidateId, CaseId, REQUIRED_CANDIDATE_CASES, REQUIRED_RENDER_REPETITIONS,
    REQUIRED_WORKSPACE_REPLAYS,
};
use super::runner::{PlannedExecution, ReplayPlan};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

const REQUEST_SCHEMA_VERSION: &str = "d005-adapter-request-v0.1-draft";
const RESPONSE_SCHEMA_VERSION: &str = "d005-adapter-response-v0.1-draft";
const MAX_IJSON_INTEGER: i64 = 9_007_199_254_740_991;
pub(crate) const REQUIRED_DRAFT_TRANSPORT_IDENTITIES: usize =
    REQUIRED_CANDIDATE_CASES * REQUIRED_WORKSPACE_REPLAYS * REQUIRED_RENDER_REPETITIONS;
const RESPONSE_FIELDS: [&str; 12] = [
    "schema_version",
    "packet_sha256",
    "plan_sha256",
    "request_sha256",
    "ordinal",
    "candidate",
    "case",
    "workspace_replay",
    "render_repetition",
    "input_manifest_sha256",
    "payload_schema_sha256",
    "payload",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DraftRequestErrorKind {
    ScheduleMismatch,
    WorkspaceReplay,
    RenderRepetition,
    InvalidDigest,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftRequestError {
    pub(crate) kind: DraftRequestErrorKind,
    pub(crate) path: &'static str,
}

impl DraftRequestError {
    const fn new(kind: DraftRequestErrorKind, path: &'static str) -> Self {
        Self { kind, path }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftAdapterRequest {
    packet_sha256: String,
    plan_sha256: String,
    ordinal: usize,
    candidate: CandidateId,
    case: CaseId,
    workspace_replay: usize,
    render_repetition: usize,
    input_manifest_sha256: String,
    payload_schema_sha256: String,
}

impl DraftAdapterRequest {
    pub(crate) const fn execution(&self) -> PlannedExecution {
        PlannedExecution {
            ordinal: self.ordinal,
            candidate: self.candidate,
            case: self.case,
        }
    }

    pub(crate) const fn workspace_replay(&self) -> usize {
        self.workspace_replay
    }

    pub(crate) const fn render_repetition(&self) -> usize {
        self.render_repetition
    }

    pub(crate) fn input_manifest_sha256(&self) -> &str {
        &self.input_manifest_sha256
    }

    pub(crate) fn payload_schema_sha256(&self) -> &str {
        &self.payload_schema_sha256
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        strict_json::canonical_bytes(&self.value())
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical_bytes()))
    }

    fn value(&self) -> JsonValue {
        strict_json::object([
            string_entry("schema_version", REQUEST_SCHEMA_VERSION),
            string_entry("packet_sha256", &self.packet_sha256),
            string_entry("plan_sha256", &self.plan_sha256),
            integer_entry("ordinal", self.ordinal),
            string_entry("candidate", self.candidate.as_str()),
            string_entry("case", self.case.as_str()),
            integer_entry("workspace_replay", self.workspace_replay),
            integer_entry("render_repetition", self.render_repetition),
            string_entry("input_manifest_sha256", &self.input_manifest_sha256),
            string_entry("payload_schema_sha256", &self.payload_schema_sha256),
        ])
    }
}

pub(crate) fn prepare_draft_request(
    plan: &ReplayPlan,
    execution: PlannedExecution,
    workspace_replay: usize,
    render_repetition: usize,
    input_manifest_sha256: &str,
    payload_schema_sha256: &str,
) -> Result<DraftAdapterRequest, DraftRequestError> {
    let schedule_index = execution.ordinal.checked_sub(1).ok_or_else(|| {
        DraftRequestError::new(DraftRequestErrorKind::ScheduleMismatch, "$/ordinal")
    })?;
    if plan.schedule().get(schedule_index) != Some(&execution) {
        return Err(DraftRequestError::new(
            DraftRequestErrorKind::ScheduleMismatch,
            "$/ordinal",
        ));
    }
    if !(1..=BUDGETS.workspace_replays).contains(&workspace_replay) {
        return Err(DraftRequestError::new(
            DraftRequestErrorKind::WorkspaceReplay,
            "$/workspace_replay",
        ));
    }
    if !(1..=BUDGETS.render_repetitions).contains(&render_repetition) {
        return Err(DraftRequestError::new(
            DraftRequestErrorKind::RenderRepetition,
            "$/render_repetition",
        ));
    }
    for (digest, path) in [
        (input_manifest_sha256, "$/input_manifest_sha256"),
        (payload_schema_sha256, "$/payload_schema_sha256"),
    ] {
        if !is_sha256_hex(digest) {
            return Err(DraftRequestError::new(
                DraftRequestErrorKind::InvalidDigest,
                path,
            ));
        }
    }
    Ok(DraftAdapterRequest {
        packet_sha256: plan.packet_sha256().to_owned(),
        plan_sha256: plan.digest_hex(),
        ordinal: execution.ordinal,
        candidate: execution.candidate,
        case: execution.case,
        workspace_replay,
        render_repetition,
        input_manifest_sha256: input_manifest_sha256.to_owned(),
        payload_schema_sha256: payload_schema_sha256.to_owned(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftBaseSlotBinding {
    pub(crate) execution: PlannedExecution,
    pub(crate) input_manifest_sha256: String,
    pub(crate) payload_schema_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DraftTransportMatrixErrorKind {
    BaseScheduleCardinality,
    BindingCardinality,
    BindingMismatch,
    InvalidDigest,
    Request(DraftRequestErrorKind),
    TransportCardinality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftTransportMatrixError {
    pub(crate) kind: DraftTransportMatrixErrorKind,
    pub(crate) path: String,
}

impl DraftTransportMatrixError {
    fn new(kind: DraftTransportMatrixErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

/// One deterministic draft transport identity. Its ordinal identifies the
/// enumeration slot only; it does not authorize or prescribe physical execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftTransportIdentity {
    capture_slot_ordinal: usize,
    request: DraftAdapterRequest,
}

impl DraftTransportIdentity {
    pub(crate) const fn capture_slot_ordinal(&self) -> usize {
        self.capture_slot_ordinal
    }

    pub(crate) const fn request(&self) -> &DraftAdapterRequest {
        &self.request
    }
}

/// Expands the exact base-slot binding table into an in-memory identity matrix.
/// It performs no launch, capture, payload validation, comparison, or execution.
pub(crate) fn enumerate_draft_transport_identities(
    plan: &ReplayPlan,
    bindings: &[DraftBaseSlotBinding],
) -> Result<Vec<DraftTransportIdentity>, DraftTransportMatrixError> {
    if plan.schedule().len() != REQUIRED_CANDIDATE_CASES {
        return Err(DraftTransportMatrixError::new(
            DraftTransportMatrixErrorKind::BaseScheduleCardinality,
            "$/base_schedule",
        ));
    }
    if bindings.len() != REQUIRED_CANDIDATE_CASES {
        return Err(DraftTransportMatrixError::new(
            DraftTransportMatrixErrorKind::BindingCardinality,
            "$/bindings",
        ));
    }

    for (index, (execution, binding)) in plan.schedule().iter().zip(bindings).enumerate() {
        let binding_path = format!("$/bindings/{index}");
        if binding.execution != *execution {
            return Err(DraftTransportMatrixError::new(
                DraftTransportMatrixErrorKind::BindingMismatch,
                format!("{binding_path}/execution"),
            ));
        }
        for (digest, field) in [
            (&binding.input_manifest_sha256, "input_manifest_sha256"),
            (&binding.payload_schema_sha256, "payload_schema_sha256"),
        ] {
            if !is_sha256_hex(digest) {
                return Err(DraftTransportMatrixError::new(
                    DraftTransportMatrixErrorKind::InvalidDigest,
                    format!("{binding_path}/{field}"),
                ));
            }
        }
    }

    let mut identities = Vec::with_capacity(REQUIRED_DRAFT_TRANSPORT_IDENTITIES);
    for (base_index, binding) in bindings.iter().enumerate() {
        for workspace_replay in 1..=REQUIRED_WORKSPACE_REPLAYS {
            for render_repetition in 1..=REQUIRED_RENDER_REPETITIONS {
                let request = prepare_draft_request(
                    plan,
                    binding.execution,
                    workspace_replay,
                    render_repetition,
                    &binding.input_manifest_sha256,
                    &binding.payload_schema_sha256,
                )
                .map_err(|error| {
                    let suffix = error.path.strip_prefix('$').unwrap_or(error.path);
                    DraftTransportMatrixError::new(
                        DraftTransportMatrixErrorKind::Request(error.kind),
                        format!("$/bindings/{base_index}/request{suffix}"),
                    )
                })?;
                identities.push(DraftTransportIdentity {
                    capture_slot_ordinal: identities.len() + 1,
                    request,
                });
            }
        }
    }
    if identities.len() != REQUIRED_DRAFT_TRANSPORT_IDENTITIES {
        return Err(DraftTransportMatrixError::new(
            DraftTransportMatrixErrorKind::TransportCardinality,
            "$/capture_slots",
        ));
    }
    Ok(identities)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapturedTermination {
    Exited(i32),
    Signaled,
    TimedOut,
    StdoutLimit,
    StderrLimit,
    SpawnFailed,
    IoFailed,
    UnsupportedSandbox,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CapturedProcess {
    pub(crate) termination: CapturedTermination,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr_truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransportErrorKind {
    OutputSizeOverflow,
    OutputTooLarge,
    StdoutTruncated,
    StderrTruncated,
    Termination(CapturedTermination),
    UnexpectedStderr,
    MissingLineFeed,
    Json(JsonErrorKind),
    CanonicalLengthMismatch,
    NonCanonical,
    MissingField,
    UnknownField,
    InvalidValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransportError {
    pub(crate) kind: TransportErrorKind,
    pub(crate) path: String,
}

impl TransportError {
    fn new(kind: TransportErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PayloadBinding {
    pub(crate) request_sha256: String,
    pub(crate) packet_sha256: String,
    pub(crate) plan_sha256: String,
    pub(crate) ordinal: usize,
    pub(crate) candidate: CandidateId,
    pub(crate) case: CaseId,
    pub(crate) workspace_replay: usize,
    pub(crate) render_repetition: usize,
    pub(crate) input_manifest_sha256: String,
    pub(crate) payload_schema_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UnvalidatedPayload {
    binding: PayloadBinding,
    value: JsonValue,
    canonical: Vec<u8>,
    digest: [u8; 32],
}

impl UnvalidatedPayload {
    pub(crate) const fn binding(&self) -> &PayloadBinding {
        &self.binding
    }

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

pub(crate) fn canonical_draft_response_bytes(
    request: &DraftAdapterRequest,
    payload_source: &[u8],
) -> Result<Vec<u8>, TransportError> {
    let payload = strict_json::parse_with_max_input(payload_source, BUDGETS.max_output_bytes)
        .map_err(|error| {
            TransportError::new(
                TransportErrorKind::Json(error.kind),
                format!("$/payload@{}", error.offset),
            )
        })?;
    if payload.as_object().is_none() {
        return Err(TransportError::new(
            TransportErrorKind::InvalidValue,
            "$/payload",
        ));
    }
    if strict_json::canonical_bytes(&payload) != payload_source {
        return Err(TransportError::new(
            TransportErrorKind::NonCanonical,
            "$/payload",
        ));
    }
    let value = response_value(request, payload);
    let json_ceiling = BUDGETS.max_output_bytes.saturating_sub(1);
    let mut bytes = canonical_response_bytes_with_limit(&value, json_ceiling)?;
    bytes.push(b'\n');
    validate_output_lengths(bytes.len(), 0)?;
    Ok(bytes)
}

pub(crate) fn validate_capture(
    request: &DraftAdapterRequest,
    capture: &CapturedProcess,
) -> Result<UnvalidatedPayload, TransportError> {
    validate_output_lengths(capture.stdout.len(), capture.stderr.len())?;
    if capture.stdout_truncated {
        return Err(TransportError::new(
            TransportErrorKind::StdoutTruncated,
            "$/stdout",
        ));
    }
    if capture.stderr_truncated {
        return Err(TransportError::new(
            TransportErrorKind::StderrTruncated,
            "$/stderr",
        ));
    }
    if capture.termination != CapturedTermination::Exited(0) {
        return Err(TransportError::new(
            TransportErrorKind::Termination(capture.termination),
            "$/termination",
        ));
    }
    if !capture.stderr.is_empty() {
        return Err(TransportError::new(
            TransportErrorKind::UnexpectedStderr,
            "$/stderr",
        ));
    }
    let Some(source) = capture.stdout.strip_suffix(b"\n") else {
        return Err(TransportError::new(
            TransportErrorKind::MissingLineFeed,
            "$/stdout",
        ));
    };
    let value =
        strict_json::parse_with_max_input(source, BUDGETS.max_output_bytes).map_err(|error| {
            TransportError::new(
                TransportErrorKind::Json(error.kind),
                format!("$/stdout@{}", error.offset),
            )
        })?;
    validate_response_value(request, &value)?;
    if strict_json::canonical_bytes(&value) != source {
        return Err(TransportError::new(
            TransportErrorKind::NonCanonical,
            "$/stdout",
        ));
    }
    let root = require_object(&value, "$")?;
    let payload = require_field(root, "payload", "$")?.clone();
    let canonical = strict_json::canonical_bytes(&payload);
    let digest = sha256::digest(&canonical);
    Ok(UnvalidatedPayload {
        binding: PayloadBinding {
            request_sha256: request.digest_hex(),
            packet_sha256: request.packet_sha256.clone(),
            plan_sha256: request.plan_sha256.clone(),
            ordinal: request.ordinal,
            candidate: request.candidate,
            case: request.case,
            workspace_replay: request.workspace_replay,
            render_repetition: request.render_repetition,
            input_manifest_sha256: request.input_manifest_sha256.clone(),
            payload_schema_sha256: request.payload_schema_sha256.clone(),
        },
        value: payload,
        canonical,
        digest,
    })
}

pub(crate) fn validate_output_lengths(
    stdout_bytes: usize,
    stderr_bytes: usize,
) -> Result<usize, TransportError> {
    let total = stdout_bytes
        .checked_add(stderr_bytes)
        .ok_or_else(|| TransportError::new(TransportErrorKind::OutputSizeOverflow, "$/output"))?;
    if total > BUDGETS.max_output_bytes {
        return Err(TransportError::new(
            TransportErrorKind::OutputTooLarge,
            "$/output",
        ));
    }
    Ok(total)
}

fn validate_response_value(
    request: &DraftAdapterRequest,
    value: &JsonValue,
) -> Result<(), TransportError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &RESPONSE_FIELDS, "$")?;
    require_exact_string(root, "schema_version", RESPONSE_SCHEMA_VERSION, "$")?;
    require_exact_string(root, "packet_sha256", &request.packet_sha256, "$")?;
    require_exact_string(root, "plan_sha256", &request.plan_sha256, "$")?;
    require_exact_string(root, "request_sha256", &request.digest_hex(), "$")?;
    require_exact_usize(root, "ordinal", request.ordinal, "$")?;
    require_exact_string(root, "candidate", request.candidate.as_str(), "$")?;
    require_exact_string(root, "case", request.case.as_str(), "$")?;
    require_exact_usize(root, "workspace_replay", request.workspace_replay, "$")?;
    require_exact_usize(root, "render_repetition", request.render_repetition, "$")?;
    require_exact_string(
        root,
        "input_manifest_sha256",
        &request.input_manifest_sha256,
        "$",
    )?;
    require_exact_string(
        root,
        "payload_schema_sha256",
        &request.payload_schema_sha256,
        "$",
    )?;
    require_object(require_field(root, "payload", "$")?, "$/payload")?;
    Ok(())
}

fn response_value(request: &DraftAdapterRequest, payload: JsonValue) -> JsonValue {
    strict_json::object([
        string_entry("schema_version", RESPONSE_SCHEMA_VERSION),
        string_entry("packet_sha256", &request.packet_sha256),
        string_entry("plan_sha256", &request.plan_sha256),
        string_entry("request_sha256", &request.digest_hex()),
        integer_entry("ordinal", request.ordinal),
        string_entry("candidate", request.candidate.as_str()),
        string_entry("case", request.case.as_str()),
        integer_entry("workspace_replay", request.workspace_replay),
        integer_entry("render_repetition", request.render_repetition),
        string_entry("input_manifest_sha256", &request.input_manifest_sha256),
        string_entry("payload_schema_sha256", &request.payload_schema_sha256),
        ("payload".to_owned(), payload),
    ])
}

fn canonical_response_bytes_with_limit(
    value: &JsonValue,
    max_bytes: usize,
) -> Result<Vec<u8>, TransportError> {
    let expected_length = preflight_canonical_value(value, max_bytes)
        .map_err(|kind| TransportError::new(TransportErrorKind::Json(kind), "$/response@0"))?;
    let canonical = strict_json::canonical_bytes(value);
    if canonical.len() != expected_length || canonical.len() > max_bytes {
        return Err(TransportError::new(
            TransportErrorKind::CanonicalLengthMismatch,
            "$/response@0",
        ));
    }
    Ok(canonical)
}

fn preflight_canonical_value(root: &JsonValue, max_bytes: usize) -> Result<usize, JsonErrorKind> {
    if BUDGETS.max_json_nodes == 0 {
        return Err(JsonErrorKind::NodeLimit);
    }

    let mut stack = Vec::new();
    stack.try_reserve(1).map_err(|_| JsonErrorKind::NodeLimit)?;
    stack.push((root, 0_usize));
    let mut discovered_nodes = 1_usize;
    let mut canonical_length = 0_usize;

    while let Some((value, depth)) = stack.pop() {
        match value {
            JsonValue::Null | JsonValue::Bool(true) => {
                add_canonical_length(&mut canonical_length, 4, max_bytes)?;
            }
            JsonValue::Bool(false) => {
                add_canonical_length(&mut canonical_length, 5, max_bytes)?;
            }
            JsonValue::Integer(integer) => {
                if !(-MAX_IJSON_INTEGER..=MAX_IJSON_INTEGER).contains(integer) {
                    return Err(JsonErrorKind::IntegerRange);
                }
                add_canonical_length(&mut canonical_length, integer.to_string().len(), max_bytes)?;
            }
            JsonValue::String(string) => {
                let string_length = canonical_string_length(string)?;
                add_canonical_length(&mut canonical_length, string_length, max_bytes)?;
            }
            JsonValue::Array(values) => {
                require_canonical_container_depth(depth)?;
                discover_canonical_children(&mut stack, &mut discovered_nodes, values.len())?;
                add_canonical_length(
                    &mut canonical_length,
                    2_usize
                        .checked_add(values.len().saturating_sub(1))
                        .ok_or(JsonErrorKind::InputTooLarge)?,
                    max_bytes,
                )?;
                let child_depth = depth.checked_add(1).ok_or(JsonErrorKind::DepthLimit)?;
                for child in values.iter().rev() {
                    stack.push((child, child_depth));
                }
            }
            JsonValue::Object(values) => {
                require_canonical_container_depth(depth)?;
                discover_canonical_children(&mut stack, &mut discovered_nodes, values.len())?;
                let punctuation = values
                    .len()
                    .checked_mul(2)
                    .and_then(|length| length.checked_sub(usize::from(!values.is_empty())))
                    .and_then(|length| length.checked_add(2))
                    .ok_or(JsonErrorKind::InputTooLarge)?;
                add_canonical_length(&mut canonical_length, punctuation, max_bytes)?;
                let child_depth = depth.checked_add(1).ok_or(JsonErrorKind::DepthLimit)?;
                for (key, child) in values.iter().rev() {
                    let key_length = canonical_string_length(key)?;
                    add_canonical_length(&mut canonical_length, key_length, max_bytes)?;
                    stack.push((child, child_depth));
                }
            }
        }
    }

    Ok(canonical_length)
}

fn discover_canonical_children(
    stack: &mut Vec<(&JsonValue, usize)>,
    discovered_nodes: &mut usize,
    child_count: usize,
) -> Result<(), JsonErrorKind> {
    *discovered_nodes = discovered_nodes
        .checked_add(child_count)
        .ok_or(JsonErrorKind::NodeLimit)?;
    if *discovered_nodes > BUDGETS.max_json_nodes {
        return Err(JsonErrorKind::NodeLimit);
    }
    stack
        .try_reserve(child_count)
        .map_err(|_| JsonErrorKind::NodeLimit)
}

fn require_canonical_container_depth(depth: usize) -> Result<(), JsonErrorKind> {
    if depth >= BUDGETS.max_json_depth {
        Err(JsonErrorKind::DepthLimit)
    } else {
        Ok(())
    }
}

fn canonical_string_length(value: &str) -> Result<usize, JsonErrorKind> {
    if value.len() > BUDGETS.max_string_bytes {
        return Err(JsonErrorKind::StringLimit);
    }
    value.chars().try_fold(2_usize, |length, character| {
        let encoded_length = match character {
            '"' | '\\' | '\u{0008}' | '\u{000c}' | '\n' | '\r' | '\t' => 2,
            '\u{0000}'..='\u{001f}' => 6,
            _ => character.len_utf8(),
        };
        length
            .checked_add(encoded_length)
            .ok_or(JsonErrorKind::InputTooLarge)
    })
}

fn add_canonical_length(
    total: &mut usize,
    amount: usize,
    max_bytes: usize,
) -> Result<(), JsonErrorKind> {
    *total = total
        .checked_add(amount)
        .ok_or(JsonErrorKind::InputTooLarge)?;
    if *total > max_bytes {
        Err(JsonErrorKind::InputTooLarge)
    } else {
        Ok(())
    }
}

fn require_object<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, TransportError> {
    value
        .as_object()
        .ok_or_else(|| TransportError::new(TransportErrorKind::InvalidValue, path))
}

fn require_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<&'a JsonValue, TransportError> {
    object.get(field).ok_or_else(|| {
        TransportError::new(
            TransportErrorKind::MissingField,
            format!("{parent}/{field}"),
        )
    })
}

fn exact_fields(
    object: &BTreeMap<String, JsonValue>,
    expected: &[&str],
    path: &str,
) -> Result<(), TransportError> {
    if let Some(field) = object
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        return Err(TransportError::new(
            TransportErrorKind::UnknownField,
            format!("{path}/{}", render_pointer_token(field)),
        ));
    }
    for field in expected {
        if !object.contains_key(*field) {
            return Err(TransportError::new(
                TransportErrorKind::MissingField,
                format!("{path}/{field}"),
            ));
        }
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

fn require_exact_string(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &str,
    parent: &str,
) -> Result<(), TransportError> {
    let path = format!("{parent}/{field}");
    if require_field(object, field, parent)?.as_str() != Some(expected) {
        return Err(TransportError::new(TransportErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn require_exact_usize(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: usize,
    parent: &str,
) -> Result<(), TransportError> {
    let path = format!("{parent}/{field}");
    let observed = require_field(object, field, parent)?
        .as_integer()
        .and_then(|integer| usize::try_from(integer).ok());
    if observed != Some(expected) {
        return Err(TransportError::new(TransportErrorKind::InvalidValue, path));
    }
    Ok(())
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

fn integer_entry(key: &str, value: usize) -> (String, JsonValue) {
    (
        key.to_owned(),
        JsonValue::Integer(i64::try_from(value).unwrap_or(i64::MAX)),
    )
}
