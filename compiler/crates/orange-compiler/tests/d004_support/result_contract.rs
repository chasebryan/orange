use super::case_subjects::{
    CASE_SUBJECT_CATALOG_CANONICAL_SHA256, CASE_SUBJECT_CATALOG_PATH,
    CASE_SUBJECT_CATALOG_RAW_SHA256, CaseSubjectCatalog,
};
use super::cases::MUTATIONS;
use super::domain::{
    BUDGETS, CANDIDATES, CASE_VERDICTS, CASES, DOMAIN_OBSERVATION_STATES, HARD_GATES,
    RELATIONSHIPS, REQUIRED_CANDIDATE_CASES,
};
use super::fixtures::{
    FIXTURE_CATALOG_CANONICAL_SHA256, FIXTURE_CATALOG_PATH, FIXTURE_CATALOG_RAW_SHA256,
    FixtureCatalog,
};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

pub(crate) const RESULT_CONTRACT_SCHEMA_VERSION: &str =
    "d004-result-contract-descriptor-v0.1-draft";

pub(crate) const REQUIRED_CASE_RECORD_FIELDS: [&str; 31] = [
    "schema_version",
    "suite_version",
    "epoch",
    "packet_sha256",
    "replay_plan_sha256",
    "slot_ordinal",
    "round",
    "position",
    "candidate",
    "case",
    "repetition",
    "positive_subject",
    "mutation_subjects",
    "identities",
    "argv",
    "environment",
    "resource_ceilings",
    "measured_resources",
    "execution_state",
    "observations",
    "log_manifest",
    "premises",
    "assumptions",
    "trusted_components",
    "unsupported_features",
    "candidate_graph",
    "sr_conformance_map",
    "case_verdict",
    "byte_manifest",
    "replay",
    "owner_labels",
];

pub(crate) const REQUIRED_POSITIVE_SUBJECT_BINDING_FIELDS: [&str; 2] =
    ["subject_id", "subject_sha256"];

pub(crate) const REQUIRED_MUTATION_SUBJECT_BINDING_FIELDS: [&str; 3] =
    ["mutation_id", "subject_id", "subject_sha256"];

pub(crate) const REQUIRED_IDENTITY_FIELDS: [&str; 11] = [
    "scheduled_slot_sha256",
    "input_manifest_sha256",
    "model_sha256",
    "tool_sha256",
    "dependency_manifest_sha256",
    "environment_sha256",
    "candidate_graph_sha256",
    "sr_map_sha256",
    "semantic_endpoint_sha256",
    "parameter_model_sha256",
    "positive_subject_sha256",
];

pub(crate) const REQUIRED_RESOURCE_CEILING_FIELDS: [&str; 4] = [
    "wall_seconds",
    "peak_memory_bytes",
    "temp_storage_bytes",
    "output_bytes",
];

pub(crate) const REQUIRED_MEASURED_RESOURCE_FIELDS: [&str; 5] = [
    "wall_milliseconds",
    "peak_memory_bytes",
    "temp_storage_bytes",
    "stdout_bytes",
    "stderr_bytes",
];

pub(crate) const REQUIRED_EXECUTION_STATE_FIELDS: [&str; 6] = [
    "kind",
    "exit_code",
    "signal",
    "stdout_truncated",
    "stderr_truncated",
    "adapter_status",
];

pub(crate) const REQUIRED_OBSERVATION_FIELDS: [&str; 12] = [
    "id",
    "subject_id",
    "subject_sha256",
    "observation_level",
    "allowed_domain_states",
    "observed_state",
    "comparison",
    "required_invalidation",
    "observed_invalidation",
    "capability_credit",
    "normalized_observation_sha256",
    "raw_log_refs",
];

pub(crate) const REQUIRED_LOG_MANIFEST_FIELDS: [&str; 2] = ["manifest_sha256", "entries"];

pub(crate) const REQUIRED_LOG_MANIFEST_ENTRY_FIELDS: [&str; 5] = [
    "id",
    "stdout_sha256",
    "stderr_sha256",
    "stdout_bytes",
    "stderr_bytes",
];

pub(crate) const REQUIRED_CONTEXT_INVENTORIES: [&str; 4] = [
    "premises",
    "assumptions",
    "trusted_components",
    "unsupported_features",
];

pub(crate) const REQUIRED_CANDIDATE_GRAPH_FIELDS: [&str; 3] = ["graph_sha256", "nodes", "edges"];

pub(crate) const REQUIRED_GRAPH_NODE_FIELDS: [&str; 3] = ["id", "role", "semantic_subject_sha256"];

pub(crate) const REQUIRED_GRAPH_EDGE_FIELDS: [&str; 13] = [
    "id",
    "relationship",
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
    "edge_sha256",
];

pub(crate) const REQUIRED_SR_MAP_FIELDS: [&str; 15] = [
    "relationship",
    "native_edges",
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
    "applicability",
    "conformance_state",
    "dependent_observation_ids",
];

pub(crate) const REQUIRED_BYTE_MANIFEST_FIELDS: [&str; 2] = ["manifest_sha256", "entries"];

pub(crate) const REQUIRED_BYTE_MANIFEST_ENTRY_FIELDS: [&str; 4] =
    ["path", "mode", "byte_length", "sha256"];

pub(crate) const REQUIRED_ENVIRONMENT_ENTRY_FIELDS: [&str; 2] = ["name", "value"];

pub(crate) const REQUIRED_CONTEXT_ENTRY_FIELDS: [&str; 4] =
    ["id", "description", "sha256", "status"];

pub(crate) const REQUIRED_UNSUPPORTED_FEATURE_FIELDS: [&str; 4] =
    ["id", "description", "state", "dependent_result"];

pub(crate) const REQUIRED_REPLAY_FIELDS: [&str; 6] = [
    "argv",
    "environment_sha256",
    "network",
    "cache",
    "input_manifest_sha256",
    "expected_output_manifest_sha256",
];

pub(crate) const REQUIRED_OWNER_LABEL_FIELDS: [&str; 4] = [
    "producer_label",
    "review_authority",
    "review_label",
    "independent_review",
];

pub(crate) const REPLAY_NON_SUCCESS_STATES: [&str; 7] = [
    "missing_input",
    "timeout",
    "resource_exhaustion",
    "crash",
    "digest_mismatch",
    "unsupported_behavior",
    "oversized_output",
];

pub(crate) const EXECUTION_STATE_KINDS: [&str; 8] = [
    "completed",
    "missing_input",
    "timeout",
    "resource_exhaustion",
    "crash",
    "digest_mismatch",
    "unsupported_behavior",
    "oversized_output",
];

pub(crate) const ADAPTER_STATUS_STATES: [&str; 3] = ["executed", "not_executed", "failed"];

pub(crate) const OBSERVED_INVALIDATION_STATES: [&str; 3] =
    ["satisfied", "not_satisfied", "not_required"];

pub(crate) const SR_APPLICABILITY_STATES: [&str; 2] = ["required", "not_required"];

pub(crate) const SR_CONFORMANCE_STATES: [&str; 4] =
    ["satisfied", "not_satisfied", "unresolved", "unsupported"];

pub(crate) const SCHEDULE_SLOT_FIELDS: [&str; 5] =
    ["ordinal", "round", "position", "candidate", "case"];

pub(crate) const SCHEDULED_SLOT_PREIMAGE_FIELDS: [&str; 10] = [
    "schema_version",
    "suite_version",
    "epoch",
    "packet_sha256",
    "replay_plan_sha256",
    "ordinal",
    "round",
    "position",
    "candidate",
    "case",
];

pub(crate) const SUBJECT_ORACLE_FIELDS: [&str; 11] = [
    "subject_id",
    "source_catalog",
    "case_scope",
    "subject_kind",
    "mutation_id",
    "subject_sha256",
    "relationship_scope",
    "allowed_domain_states",
    "required_invalidation",
    "observation_level",
    "capability_credit",
];

pub(crate) const DIGEST_JOIN_FIELDS: [&str; 13] = [
    "packet_sha256",
    "replay_plan_sha256",
    "scheduled_slot_sha256",
    "input_manifest_sha256",
    "model_sha256",
    "tool_sha256",
    "dependency_manifest_sha256",
    "environment_sha256",
    "candidate_graph_sha256",
    "sr_map_sha256",
    "semantic_endpoint_sha256",
    "parameter_model_sha256",
    "positive_subject_sha256",
];

pub(crate) const ORDERING_RULES: [&str; 6] = [
    "schedule slots are ordinal ascending and contain each candidate-case pair exactly once",
    "mutation identifiers follow the exact registered same-case order",
    "environment entries are name-ascending and unique",
    "byte and log manifest entries are path-or-id ascending and unique",
    "observations follow preregistered fixture order and are unique",
    "SR conformance rows are exactly SR-01 through SR-14 in canonical order",
];

pub(crate) const OBSERVATION_COMPARISONS: [&str; 2] = ["matched", "mismatched"];

pub(crate) const DERIVATION_RULES: [&str; 30] = [
    "each observation subject_id and subject_sha256 must resolve exactly one authenticated subject-oracle row",
    "observation_level, allowed_domain_states, required_invalidation, and capability_credit must equal the resolved oracle and cannot be result-defined or broadened",
    "comparison is matched if and only if observed_state is a member of the resolved oracle allowed_domain_states",
    "comparison is mismatched otherwise",
    "the later frozen epoch defines the complete per-case subject inventory; every required subject appears exactly once and no extra observation is permitted",
    "case_verdict is derived and must be recomputed rather than trusted",
    "case_verdict is pass if and only if execution is validly completed, every resource bound holds, the frozen authenticated observation inventory is complete and matched, every required invalidation is satisfied, every digest and manifest join succeeds, all required SR rows are satisfied with valid dependencies, and the replay contract is satisfied",
    "any failed pass condition forces case_verdict fail",
    "a replay-level missing input, timeout, exhaustion, crash, digest mismatch, or oversized output forces case_verdict fail",
    "candidate-adapter inability cannot satisfy a domain-level unsupported expectation",
    "domain-level unknown, unsupported, or exhausted can match only when preregistered",
    "every SR-01 through SR-14 relationship appears exactly once in canonical order",
    "SR applicability is derived from authenticated subject relationship scopes rather than declared by a result",
    "every required SR row must be satisfied; not_satisfied, unresolved, or unsupported forces case_verdict fail",
    "every dependent_observation_id must join exactly one authenticated observation in the same candidate-case repetition",
    "candidate and case must match the digest-bound scheduled slot",
    "the positive subject and ordered mutation-subject objects must join exact authenticated subject-oracle rows for the scheduled case",
    "each mutation identifier, subject identifier, and subject sha256 must join in registered order without parallel-array substitution",
    "every required digest is exactly 64 lowercase hexadecimal characters",
    "packet, plan, slot, input, model, tool, dependency, environment, graph, and SR-map identities must join the byte manifest",
    "scheduled_slot_sha256 is sha256 of the no-line-feed canonical scheduled-slot preimage and is unavailable until epoch, packet, and replay-plan identities are frozen",
    "replay uses an argument vector, exact allowlisted environment, denied network, empty candidate cache, and deterministic output manifest",
    "execution kind completed requires exit_code zero, signal null, adapter_status executed, and no truncated output",
    "resource bounds hold only when wall_milliseconds is at most wall_seconds times 1000, peak_memory_bytes and temp_storage_bytes are at most their ceilings, and checked-add stdout_bytes plus stderr_bytes is at most output_bytes",
    "every replay non-success kind forces case_verdict fail and cannot be reclassified as a domain observation",
    "replay resource_exhaustion is distinct from preregistered domain-level exhausted",
    "owner labels are owner-produced, Orange Project Owner, solo-reviewed, and independent review unavailable only after an actual owner review record exists",
    "normalized observations must join their bounded raw-log manifest entries by digest",
    "any changed premise creates a new evidence epoch",
    "owner production or owner review never implies independent review",
];

pub(crate) const RESULT_CONTRACT_NONCLAIMS: [&str; 12] = [
    "descriptor only; no populated case record exists",
    "no candidate mapping or adapter exists",
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
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResultContractErrorKind {
    Json(JsonErrorKind),
    NonCanonical,
    SchemaMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResultContractError {
    pub(crate) kind: ResultContractErrorKind,
    pub(crate) path: String,
}

impl ResultContractError {
    fn new(kind: ResultContractErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

/// A closed descriptor for a possible later case-record schema. It contains no
/// case record and exposes no launch, capture, persistence, or evidence API.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftResultContractDescriptor {
    canonical: Vec<u8>,
}

impl DraftResultContractDescriptor {
    pub(crate) const fn epoch(&self) -> Option<&'static str> {
        None
    }

    pub(crate) const fn epoch_status(&self) -> &'static str {
        "unfrozen"
    }

    pub(crate) const fn owner_protocol_review(&self) -> &'static str {
        "none"
    }

    pub(crate) const fn replay_repetitions(&self) -> Option<usize> {
        None
    }

    pub(crate) const fn result_record_count(&self) -> usize {
        0
    }

    pub(crate) const fn completed_candidate_cases(&self) -> usize {
        0
    }

    pub(crate) const fn complete_candidates(&self) -> usize {
        0
    }

    pub(crate) const fn complete_cross_candidate_cases(&self) -> usize {
        0
    }

    pub(crate) const fn evidence_status(&self) -> &'static str {
        "none"
    }

    pub(crate) const fn selection(&self) -> Option<&'static str> {
        None
    }

    pub(crate) const fn conclusion(&self) -> Option<&'static str> {
        None
    }

    pub(crate) const fn roadmap_gate_credit(&self) -> &'static str {
        "none"
    }

    pub(crate) const fn readiness_credit(&self) -> &'static str {
        "none"
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        self.canonical.clone()
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical_bytes()))
    }
}

pub(crate) fn canonical_draft_result_contract_descriptor_bytes(
    case_subject_catalog: &CaseSubjectCatalog,
    fixture_catalog: &FixtureCatalog,
) -> Vec<u8> {
    strict_json::canonical_bytes(&descriptor_value(case_subject_catalog, fixture_catalog))
}

pub(crate) fn parse_draft_result_contract_descriptor(
    source: &[u8],
    case_subject_catalog: &CaseSubjectCatalog,
    fixture_catalog: &FixtureCatalog,
) -> Result<DraftResultContractDescriptor, ResultContractError> {
    let value = strict_json::parse(source).map_err(|error| {
        ResultContractError::new(
            ResultContractErrorKind::Json(error.kind),
            format!("$/descriptor@{}", error.offset),
        )
    })?;
    if strict_json::canonical_bytes(&value) != source {
        return Err(ResultContractError::new(
            ResultContractErrorKind::NonCanonical,
            "$/descriptor",
        ));
    }
    let expected = descriptor_value(case_subject_catalog, fixture_catalog);
    if value != expected {
        return Err(ResultContractError::new(
            ResultContractErrorKind::SchemaMismatch,
            first_mismatch_path(&expected, &value, "$"),
        ));
    }
    Ok(DraftResultContractDescriptor {
        canonical: source.to_vec(),
    })
}

fn first_mismatch_path(expected: &JsonValue, observed: &JsonValue, path: &str) -> String {
    match (expected, observed) {
        (JsonValue::Object(expected), JsonValue::Object(observed)) => {
            for key in expected.keys() {
                if !observed.contains_key(key) {
                    return format!("{path}/{}", pointer_token(key));
                }
            }
            for key in observed.keys() {
                if !expected.contains_key(key) {
                    return format!("{path}/{}", pointer_token(key));
                }
            }
            for (key, expected_value) in expected {
                let observed_value = observed
                    .get(key)
                    .expect("object membership checked before comparison");
                if expected_value != observed_value {
                    return first_mismatch_path(
                        expected_value,
                        observed_value,
                        &format!("{path}/{}", pointer_token(key)),
                    );
                }
            }
            path.to_owned()
        }
        (JsonValue::Array(expected), JsonValue::Array(observed)) => {
            if expected.len() != observed.len() {
                return path.to_owned();
            }
            for (index, (expected_value, observed_value)) in
                expected.iter().zip(observed).enumerate()
            {
                if expected_value != observed_value {
                    return first_mismatch_path(
                        expected_value,
                        observed_value,
                        &format!("{path}/{index}"),
                    );
                }
            }
            path.to_owned()
        }
        _ => path.to_owned(),
    }
}

fn pointer_token(value: &str) -> String {
    value.replace('~', "~0").replace('/', "~1")
}

fn descriptor_value(
    case_subject_catalog: &CaseSubjectCatalog,
    fixture_catalog: &FixtureCatalog,
) -> JsonValue {
    strict_json::object([
        string_entry("schema_version", RESULT_CONTRACT_SCHEMA_VERSION),
        string_entry("suite_version", "d004-v0.4-draft"),
        string_entry("status", "draft_schema_descriptor_only"),
        string_entry("canonicalization", "RFC8785_ASCII_INTEGER_SUBSET"),
        ("epoch".to_owned(), JsonValue::Null),
        string_entry("epoch_status", "unfrozen"),
        string_entry("owner_protocol_review", "none"),
        ("replay_repetitions".to_owned(), JsonValue::Null),
        ("candidate_ids".to_owned(), candidate_ids_value()),
        ("case_ids".to_owned(), case_ids_value()),
        strings_entry("hard_gates", &HARD_GATES),
        (
            "mutation_counts_by_case".to_owned(),
            mutation_counts_value(),
        ),
        ("mutation_inventory".to_owned(), mutation_inventory_value()),
        (
            "subject_catalog_bindings".to_owned(),
            subject_catalog_bindings_value(case_subject_catalog, fixture_catalog),
        ),
        (
            "subject_oracle_inventory".to_owned(),
            subject_oracle_inventory_value(case_subject_catalog, fixture_catalog),
        ),
        ("schedule_contract".to_owned(), schedule_contract_value()),
        ("schedule_slots".to_owned(), schedule_slots_value()),
        (
            "scheduled_slot_identity_contract".to_owned(),
            scheduled_slot_identity_contract_value(),
        ),
        strings_entry("required_case_record_fields", &REQUIRED_CASE_RECORD_FIELDS),
        strings_entry(
            "required_positive_subject_binding_fields",
            &REQUIRED_POSITIVE_SUBJECT_BINDING_FIELDS,
        ),
        strings_entry(
            "required_mutation_subject_binding_fields",
            &REQUIRED_MUTATION_SUBJECT_BINDING_FIELDS,
        ),
        strings_entry("required_identity_fields", &REQUIRED_IDENTITY_FIELDS),
        strings_entry(
            "required_resource_ceiling_fields",
            &REQUIRED_RESOURCE_CEILING_FIELDS,
        ),
        strings_entry(
            "required_measured_resource_fields",
            &REQUIRED_MEASURED_RESOURCE_FIELDS,
        ),
        strings_entry(
            "required_execution_state_fields",
            &REQUIRED_EXECUTION_STATE_FIELDS,
        ),
        strings_entry("required_observation_fields", &REQUIRED_OBSERVATION_FIELDS),
        strings_entry(
            "required_log_manifest_fields",
            &REQUIRED_LOG_MANIFEST_FIELDS,
        ),
        strings_entry(
            "required_log_manifest_entry_fields",
            &REQUIRED_LOG_MANIFEST_ENTRY_FIELDS,
        ),
        strings_entry(
            "required_context_inventories",
            &REQUIRED_CONTEXT_INVENTORIES,
        ),
        strings_entry(
            "required_candidate_graph_fields",
            &REQUIRED_CANDIDATE_GRAPH_FIELDS,
        ),
        strings_entry("required_graph_node_fields", &REQUIRED_GRAPH_NODE_FIELDS),
        strings_entry("required_graph_edge_fields", &REQUIRED_GRAPH_EDGE_FIELDS),
        strings_entry("required_sr_map_fields", &REQUIRED_SR_MAP_FIELDS),
        strings_entry(
            "required_byte_manifest_fields",
            &REQUIRED_BYTE_MANIFEST_FIELDS,
        ),
        strings_entry(
            "required_byte_manifest_entry_fields",
            &REQUIRED_BYTE_MANIFEST_ENTRY_FIELDS,
        ),
        strings_entry(
            "required_environment_entry_fields",
            &REQUIRED_ENVIRONMENT_ENTRY_FIELDS,
        ),
        strings_entry(
            "required_context_entry_fields",
            &REQUIRED_CONTEXT_ENTRY_FIELDS,
        ),
        strings_entry(
            "required_unsupported_feature_fields",
            &REQUIRED_UNSUPPORTED_FEATURE_FIELDS,
        ),
        strings_entry("required_replay_fields", &REQUIRED_REPLAY_FIELDS),
        strings_entry("required_owner_label_fields", &REQUIRED_OWNER_LABEL_FIELDS),
        strings_entry("relationships", &RELATIONSHIPS),
        strings_entry("domain_observation_states", &DOMAIN_OBSERVATION_STATES),
        strings_entry("observation_comparisons", &OBSERVATION_COMPARISONS),
        strings_entry("case_verdicts", &CASE_VERDICTS),
        strings_entry("execution_state_kinds", &EXECUTION_STATE_KINDS),
        strings_entry("adapter_status_states", &ADAPTER_STATUS_STATES),
        strings_entry(
            "observed_invalidation_states",
            &OBSERVED_INVALIDATION_STATES,
        ),
        strings_entry("sr_applicability_states", &SR_APPLICABILITY_STATES),
        strings_entry("sr_conformance_states", &SR_CONFORMANCE_STATES),
        strings_entry("replay_non_success_states", &REPLAY_NON_SUCCESS_STATES),
        ("resource_ceilings".to_owned(), resource_ceilings_value()),
        ("transport_limits".to_owned(), transport_limits_value()),
        ("digest_contract".to_owned(), digest_contract_value()),
        ("transport_contract".to_owned(), transport_contract_value()),
        (
            "owner_label_contract".to_owned(),
            owner_label_contract_value(),
        ),
        ("ordering_contract".to_owned(), ordering_contract_value()),
        strings_entry("derivation_rules", &DERIVATION_RULES),
        ("current_state".to_owned(), current_state_value()),
        ("result_records".to_owned(), JsonValue::Array(Vec::new())),
        (
            "observation_records".to_owned(),
            JsonValue::Array(Vec::new()),
        ),
        ("verdict_records".to_owned(), JsonValue::Array(Vec::new())),
        ("review_records".to_owned(), JsonValue::Array(Vec::new())),
        ("evidence_records".to_owned(), JsonValue::Array(Vec::new())),
        strings_entry("nonclaims", &RESULT_CONTRACT_NONCLAIMS),
    ])
}

fn candidate_ids_value() -> JsonValue {
    JsonValue::Array(
        CANDIDATES
            .into_iter()
            .map(|candidate| JsonValue::String(candidate.as_str().to_owned()))
            .collect(),
    )
}

fn case_ids_value() -> JsonValue {
    JsonValue::Array(
        CASES
            .into_iter()
            .map(|case| JsonValue::String(case.as_str().to_owned()))
            .collect(),
    )
}

fn mutation_counts_value() -> JsonValue {
    strict_json::object([
        usize_entry("SC-01", 4),
        usize_entry("SC-02", 5),
        usize_entry("SC-03", 5),
        usize_entry("SC-04", 6),
        usize_entry("SC-05", 6),
    ])
}

fn mutation_inventory_value() -> JsonValue {
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

fn subject_catalog_bindings_value(
    case_subject_catalog: &CaseSubjectCatalog,
    fixture_catalog: &FixtureCatalog,
) -> JsonValue {
    assert_eq!(
        case_subject_catalog.digest_hex(),
        CASE_SUBJECT_CATALOG_CANONICAL_SHA256,
        "result descriptor requires the authenticated case-subject catalog"
    );
    assert_eq!(
        fixture_catalog.digest_hex(),
        FIXTURE_CATALOG_CANONICAL_SHA256,
        "result descriptor requires the authenticated cross-cutting fixture catalog"
    );
    strict_json::object([
        (
            "case_subject_catalog".to_owned(),
            catalog_binding_value(
                CASE_SUBJECT_CATALOG_PATH,
                CASE_SUBJECT_CATALOG_RAW_SHA256,
                CASE_SUBJECT_CATALOG_CANONICAL_SHA256,
            ),
        ),
        (
            "cross_cutting_fixture_catalog".to_owned(),
            catalog_binding_value(
                FIXTURE_CATALOG_PATH,
                FIXTURE_CATALOG_RAW_SHA256,
                FIXTURE_CATALOG_CANONICAL_SHA256,
            ),
        ),
    ])
}

fn catalog_binding_value(path: &str, raw_sha256: &str, canonical_sha256: &str) -> JsonValue {
    strict_json::object([
        string_entry("path", path),
        string_entry("raw_sha256", raw_sha256),
        string_entry("canonical_sha256", canonical_sha256),
    ])
}

fn subject_oracle_inventory_value(
    case_subject_catalog: &CaseSubjectCatalog,
    fixture_catalog: &FixtureCatalog,
) -> JsonValue {
    let case_root = case_subject_catalog
        .value()
        .as_object()
        .expect("validated case-subject catalog root");
    let positives = case_root
        .get("positive_subjects")
        .and_then(JsonValue::as_array)
        .expect("validated positive-subject array");
    let mutations = case_root
        .get("mutation_subjects")
        .and_then(JsonValue::as_array)
        .expect("validated mutation-subject array");
    let fixture_root = fixture_catalog
        .value()
        .as_object()
        .expect("validated cross-cutting fixture catalog root");
    let fixtures = fixture_root
        .get("fixtures")
        .and_then(JsonValue::as_array)
        .expect("validated cross-cutting fixture array");

    let mut inventory = Vec::with_capacity(positives.len() + mutations.len() + fixtures.len());
    inventory.extend(
        positives
            .iter()
            .map(|record| case_subject_oracle_value(record, false)),
    );
    inventory.extend(
        mutations
            .iter()
            .map(|record| case_subject_oracle_value(record, true)),
    );
    inventory.extend(fixtures.iter().map(cross_cutting_oracle_value));
    JsonValue::Array(inventory)
}

fn case_subject_oracle_value(record: &JsonValue, mutation: bool) -> JsonValue {
    let record = record.as_object().expect("validated case-subject record");
    let subject = record
        .get("subject")
        .and_then(JsonValue::as_object)
        .expect("validated case-subject value");
    let expectation = record
        .get("declared_expectation")
        .and_then(JsonValue::as_object)
        .expect("validated case-subject expectation");
    strict_json::object([
        clone_entry(record, "id", "subject_id"),
        string_entry("source_catalog", "case_subject_catalog"),
        (
            "case_scope".to_owned(),
            JsonValue::Array(vec![required_clone(record, "case")]),
        ),
        clone_entry(subject, "kind", "subject_kind"),
        (
            "mutation_id".to_owned(),
            if mutation {
                required_clone(record, "mutation_id")
            } else {
                JsonValue::Null
            },
        ),
        clone_entry(record, "subject_sha256", "subject_sha256"),
        clone_entry(subject, "relationship_scope", "relationship_scope"),
        clone_entry(
            expectation,
            "allowed_domain_states",
            "allowed_domain_states",
        ),
        (
            "required_invalidation".to_owned(),
            expectation
                .get("required_invalidation")
                .cloned()
                .unwrap_or(JsonValue::Null),
        ),
        clone_entry(expectation, "observation_level", "observation_level"),
        string_entry("capability_credit", "none"),
    ])
}

fn cross_cutting_oracle_value(record: &JsonValue) -> JsonValue {
    let record = record
        .as_object()
        .expect("validated cross-cutting fixture record");
    let subject = record
        .get("fixture_subject")
        .and_then(JsonValue::as_object)
        .expect("validated cross-cutting fixture subject");
    let expectation = record
        .get("expected_observation")
        .and_then(JsonValue::as_object)
        .expect("validated cross-cutting fixture expectation");
    strict_json::object([
        clone_entry(record, "proposal_id", "subject_id"),
        string_entry("source_catalog", "cross_cutting_fixture_catalog"),
        clone_entry(subject, "case_scope", "case_scope"),
        clone_entry(subject, "mutation_kind", "subject_kind"),
        ("mutation_id".to_owned(), JsonValue::Null),
        clone_entry(record, "fixture_subject_sha256", "subject_sha256"),
        clone_entry(subject, "relationship_scope", "relationship_scope"),
        (
            "allowed_domain_states".to_owned(),
            JsonValue::Array(vec![required_clone(expectation, "state")]),
        ),
        clone_entry(
            expectation,
            "required_invalidation",
            "required_invalidation",
        ),
        clone_entry(expectation, "observation_level", "observation_level"),
        clone_entry(expectation, "capability_credit", "capability_credit"),
    ])
}

fn clone_entry(
    object: &std::collections::BTreeMap<String, JsonValue>,
    source_key: &str,
    target_key: &str,
) -> (String, JsonValue) {
    (target_key.to_owned(), required_clone(object, source_key))
}

fn required_clone(object: &std::collections::BTreeMap<String, JsonValue>, key: &str) -> JsonValue {
    object
        .get(key)
        .unwrap_or_else(|| panic!("validated catalog field {key}"))
        .clone()
}

fn schedule_contract_value() -> JsonValue {
    strict_json::object([
        string_entry("status", "identity_plan_only"),
        string_entry("physical_execution_order", "unassigned"),
        usize_entry("candidate_count", CANDIDATES.len()),
        usize_entry("case_count", CASES.len()),
        usize_entry("required_slot_count", REQUIRED_CANDIDATE_CASES),
        usize_entry("unique_candidate_case_pairs", REQUIRED_CANDIDATE_CASES),
        strings_entry("slot_fields", &SCHEDULE_SLOT_FIELDS),
    ])
}

fn scheduled_slot_identity_contract_value() -> JsonValue {
    strict_json::object([
        string_entry("schema_version", "d004-scheduled-slot-identity-v0.1"),
        string_entry("suite_version", "d004-v0.4-draft"),
        string_entry("canonicalization", "RFC8785_ASCII_INTEGER_SUBSET"),
        ("terminal_line_feed".to_owned(), JsonValue::Bool(false)),
        string_entry("hash", "sha256"),
        string_entry(
            "availability",
            "unavailable_before_frozen_epoch_packet_and_replay_plan",
        ),
        strings_entry("preimage_fields", &SCHEDULED_SLOT_PREIMAGE_FIELDS),
    ])
}

fn schedule_slots_value() -> JsonValue {
    let mut slots = Vec::with_capacity(REQUIRED_CANDIDATE_CASES);
    for round in 0..CANDIDATES.len() {
        for position in 0..CANDIDATES.len() {
            let candidate = CANDIDATES[(round + position) % CANDIDATES.len()];
            let case = CASES[(2 * round + position) % CASES.len()];
            slots.push(strict_json::object([
                usize_entry("ordinal", slots.len() + 1),
                usize_entry("round", round + 1),
                usize_entry("position", position + 1),
                string_entry("candidate", candidate.as_str()),
                string_entry("case", case.as_str()),
            ]));
        }
    }
    JsonValue::Array(slots)
}

fn digest_contract_value() -> JsonValue {
    strict_json::object([
        string_entry("algorithm", "sha256"),
        string_entry("encoding", "lowercase_hex"),
        usize_entry("hex_characters", 64),
        strings_entry("join_fields", &DIGEST_JOIN_FIELDS),
        string_entry(
            "byte_binding",
            "all referenced bytes must match their exact raw sha256 before interpretation",
        ),
        string_entry(
            "cross_slot_rule",
            "no digest-bound identity may be substituted across scheduled slots",
        ),
    ])
}

fn transport_contract_value() -> JsonValue {
    strict_json::object([
        string_entry("argv_form", "argument_vector"),
        string_entry("shell_string", "forbidden"),
        string_entry("environment", "exact_allowlist"),
        string_entry("network", "denied"),
        string_entry("cache", "empty_candidate_specific"),
        string_entry("output_manifest", "deterministic_digest_bound"),
        string_entry("persistence", "none_before_frozen_epoch"),
        strings_entry("non_success_states", &REPLAY_NON_SUCCESS_STATES),
    ])
}

fn owner_label_contract_value() -> JsonValue {
    strict_json::object([
        string_entry("producer_label", "owner-produced"),
        string_entry("review_authority", "Orange Project Owner"),
        string_entry("review_label", "solo-reviewed"),
        string_entry("independent_review", "unavailable"),
        string_entry("current_record_status", "absent"),
    ])
}

fn ordering_contract_value() -> JsonValue {
    strict_json::object([strings_entry("rules", &ORDERING_RULES)])
}

fn resource_ceilings_value() -> JsonValue {
    strict_json::object([
        usize_entry("wall_seconds", BUDGETS.case_wall_seconds),
        u64_entry("peak_memory_bytes", BUDGETS.case_peak_memory_bytes),
        u64_entry("temp_storage_bytes", BUDGETS.case_temp_storage_bytes),
        u64_entry("output_bytes", BUDGETS.case_output_bytes),
    ])
}

fn transport_limits_value() -> JsonValue {
    strict_json::object([
        usize_entry("max_packet_bytes", BUDGETS.max_packet_bytes),
        usize_entry("max_json_depth", BUDGETS.max_json_depth),
        usize_entry("max_json_nodes", BUDGETS.max_json_nodes),
        usize_entry("max_string_bytes", BUDGETS.max_string_bytes),
    ])
}

fn current_state_value() -> JsonValue {
    strict_json::object([
        ("epoch".to_owned(), JsonValue::Null),
        string_entry("epoch_status", "unfrozen"),
        string_entry("owner_protocol_review", "none"),
        ("replay_repetitions".to_owned(), JsonValue::Null),
        usize_entry("required_candidate_cases", REQUIRED_CANDIDATE_CASES),
        usize_entry("result_record_count", 0),
        usize_entry("completed_candidate_cases", 0),
        usize_entry("complete_candidates", 0),
        usize_entry("complete_cross_candidate_cases", 0),
        string_entry("evidence_status", "none"),
        ("selection".to_owned(), JsonValue::Null),
        ("conclusion".to_owned(), JsonValue::Null),
        string_entry("roadmap_gate_credit", "none"),
        string_entry("readiness_credit", "none"),
    ])
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
        JsonValue::Integer(i64::try_from(value).expect("bounded D-004 integer")),
    )
}

fn u64_entry(key: &str, value: u64) -> (String, JsonValue) {
    (
        key.to_owned(),
        JsonValue::Integer(i64::try_from(value).expect("bounded D-004 integer")),
    )
}
