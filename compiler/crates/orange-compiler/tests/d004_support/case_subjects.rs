use std::collections::{BTreeMap, BTreeSet};

use super::cases::MUTATIONS;
use super::domain::{CASES, CaseId};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

pub(crate) const CASE_SUBJECT_CATALOG_PATH: &str =
    "research/decisions/D-004/d004-v0.4-case-subjects.json";
pub(crate) const CASE_SUBJECT_CATALOG_CANONICAL_SHA256: &str =
    "b3a8bcf4f0f084740e92cbff6fd57273df0a078af9c6b974f68d95ba333c6dc1";
pub(crate) const CASE_SUBJECT_CATALOG_RAW_SHA256: &str =
    "c94100598aaf39954fe683a44f6a4d34837304eb361a1b478ca26884892d8ed6";

const CASE_SUBJECT_SCHEMA: &str = "d004-case-subject-v0.1";
const POSITIVE_SUBJECT_COUNT: usize = 5;
const MUTATION_SUBJECT_COUNT: usize = 26;
const SUBJECT_COUNT: usize = POSITIVE_SUBJECT_COUNT + MUTATION_SUBJECT_COUNT;
const DECISION_SUITE_RAW_SHA256: &str =
    "f44c556202d0e235fd42c03181134ab3047d3e9b6f19b3015dae15a86d00dc0b";
const MUTATION_MANIFEST_CANONICAL_SHA256: &str =
    "970999d998cdc202a6caa4e2f798017416c88211a5b6b8508132a07cc9080c0c";
const MUTATION_MANIFEST_RAW_SHA256: &str =
    "1d46d6d66c0704fcaa462c625dcac2e72150497bb075322c5e076ea42898be54";

const ROOT_FIELDS: [&str; 14] = [
    "schema_version",
    "suite_version",
    "status",
    "owner_protocol_review",
    "canonicalization",
    "source_bindings",
    "subject_count",
    "positive_subject_count",
    "mutation_subject_count",
    "positive_subjects",
    "mutation_subjects",
    "execution_boundary",
    "evidence_status",
    "nonclaims",
];
const SOURCE_BINDING_FIELDS: [&str; 2] = ["named_mutation_manifest", "suite"];
const SUITE_BINDING_FIELDS: [&str; 2] = ["path", "raw_sha256"];
const MANIFEST_BINDING_FIELDS: [&str; 3] = ["path", "canonical_sha256", "raw_sha256"];
const EXECUTION_BOUNDARY_FIELDS: [&str; 5] = [
    "candidate_adapter",
    "candidate_process",
    "candidate_tool",
    "network",
    "preflight_output_persistence",
];
const POSITIVE_RECORD_FIELDS: [&str; 5] = [
    "id",
    "case",
    "subject",
    "subject_sha256",
    "declared_expectation",
];
const POSITIVE_SUBJECT_FIELDS: [&str; 6] = [
    "schema_version",
    "id",
    "case",
    "kind",
    "relationship_scope",
    "model",
];
const POSITIVE_EXPECTATION_FIELDS: [&str; 3] = [
    "observation_level",
    "allowed_domain_states",
    "forbidden_domain_states",
];
const MUTATION_RECORD_FIELDS: [&str; 7] = [
    "id",
    "case",
    "mutation_id",
    "manifest_record_sha256",
    "subject",
    "subject_sha256",
    "declared_expectation",
];
const MUTATION_SUBJECT_FIELDS: [&str; 8] = [
    "schema_version",
    "id",
    "case",
    "mutation_id",
    "kind",
    "positive_subject_sha256",
    "relationship_scope",
    "model",
];
const MUTATION_EXPECTATION_FIELDS: [&str; 4] = [
    "observation_level",
    "allowed_domain_states",
    "forbidden_domain_states",
    "required_invalidation",
];
const MUTATION_MODEL_FIELDS: [&str; 6] = [
    "kind",
    "operator",
    "target",
    "baseline_value",
    "mutated_value",
    "dependent_result",
];
const DEPENDENT_RESULT_FIELDS: [&str; 3] = ["id", "required_target", "required_value"];
const MANIFEST_RECORD_FIELDS: [&str; 3] = ["id", "case", "description"];

pub(crate) const CASE_SUBJECT_NONCLAIMS: [&str; 9] = [
    "no candidate mapping or adapter exists",
    "no candidate process or tool invoked",
    "no observed state, match, result, or verdict produced",
    "no candidate capability or capability absence established",
    "case-subject integrity preflight is not D-004 execution evidence",
    "no D-004 evidence epoch frozen",
    "no semantic-strata candidate selected",
    "no D-004 disposition inferred",
    "no S3b implementation, roadmap gate closure, or readiness movement authorized",
];

const FORBIDDEN_PERSISTED_FIELDS: [&str; 11] = [
    "adapter_output",
    "candidate_execution",
    "candidate_observation",
    "capability_credit",
    "case_result",
    "case_verdict",
    "loader_status",
    "matched",
    "observed_invalidation",
    "observed_state",
    "verdict",
];

const SC01_SCOPE: [&str; 1] = ["SR-01"];
const SC02_SCOPE: [&str; 3] = ["SR-01", "SR-02", "SR-09"];
const SC03_SCOPE: [&str; 2] = ["SR-02", "SR-10"];
const SC04_SCOPE: [&str; 3] = ["SR-01", "SR-03", "SR-11"];
const SC05_SCOPE: [&str; 3] = ["SR-04", "SR-08", "SR-12"];

#[derive(Clone, Copy)]
struct PositiveSpec {
    id: &'static str,
    case: CaseId,
    subject_sha256: &'static str,
}

const POSITIVE_SPECS: [PositiveSpec; POSITIVE_SUBJECT_COUNT] = [
    PositiveSpec {
        id: "D004-CS-POS-SC01",
        case: CaseId::Sc01,
        subject_sha256: "6b1287b1c6e657e47bdb0e103e9d0810e11049c0b15dfffaf81f969ae7ad71e8",
    },
    PositiveSpec {
        id: "D004-CS-POS-SC02",
        case: CaseId::Sc02,
        subject_sha256: "e3a098f166118f93d9e9da76548c8a8db4e2bf53314f9aacb73981a66a4e1b6d",
    },
    PositiveSpec {
        id: "D004-CS-POS-SC03",
        case: CaseId::Sc03,
        subject_sha256: "c51ab70f1ce25442deb91eb29407772535c0ecdb43bc7de13021278f5a6cee9d",
    },
    PositiveSpec {
        id: "D004-CS-POS-SC04",
        case: CaseId::Sc04,
        subject_sha256: "ff4b439bc5716b4aee344cdc40196851e3e76c5d70577135d8d40cba7c90498d",
    },
    PositiveSpec {
        id: "D004-CS-POS-SC05",
        case: CaseId::Sc05,
        subject_sha256: "c9e527a57524a1a1dcf9cd8b88340b3083cdf1c3d0eac134d4d78461324dc45a",
    },
];

#[derive(Clone, Copy)]
struct MutationModelSpec {
    subject_id: &'static str,
    operator: &'static str,
    target: &'static str,
    baseline_value: &'static str,
    mutated_value: &'static str,
    allowed_states: &'static [&'static str],
}

const REJECTED: &[&str] = &["rejected"];
const REJECTED_UNKNOWN: &[&str] = &["rejected", "unknown"];
const REJECTED_UNKNOWN_UNSUPPORTED: &[&str] = &["rejected", "unknown", "unsupported"];
const REJECTED_UNSUPPORTED: &[&str] = &["rejected", "unsupported"];

const MUTATION_MODELS: [MutationModelSpec; MUTATION_SUBJECT_COUNT] = [
    mutation(
        "D004-CS-MUT-SC01-M01",
        "replace",
        "integer_to_word_conversion_mode",
        "explicit_only",
        "implicit_allowed",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC01-M02",
        "replace",
        "byte_order_conversion_mode",
        "explicit_only",
        "implicit_allowed",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC01-M03",
        "replace",
        "word_width",
        "declared_width",
        "mismatched_width",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC01-M04",
        "replace",
        "shift_bound",
        "bounded_to_word_width",
        "unbounded",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC02-M01",
        "replace",
        "mutable_aliasing",
        "exclusive_mutable_access",
        "illegal_alias",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC02-M02",
        "replace",
        "memory_access_range",
        "within_declared_range",
        "outside_declared_range",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC02-M03",
        "remove",
        "loop_invariant",
        "required_present",
        "absent",
        REJECTED_UNKNOWN,
    ),
    mutation(
        "D004-CS-MUT-SC02-M04",
        "replace",
        "read_initialization",
        "initialized",
        "uninitialized",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC02-M05",
        "substitute",
        "refinement_subject_identity",
        "original",
        "substitute",
        REJECTED_UNKNOWN,
    ),
    mutation(
        "D004-CS-MUT-SC03-M01",
        "replace",
        "branch_condition",
        "public",
        "secret",
        REJECTED_UNKNOWN_UNSUPPORTED,
    ),
    mutation(
        "D004-CS-MUT-SC03-M02",
        "replace",
        "memory_address",
        "public",
        "secret",
        REJECTED_UNKNOWN_UNSUPPORTED,
    ),
    mutation(
        "D004-CS-MUT-SC03-M03",
        "replace",
        "loop_bound",
        "public",
        "secret",
        REJECTED_UNKNOWN_UNSUPPORTED,
    ),
    mutation(
        "D004-CS-MUT-SC03-M04",
        "replace",
        "failure_path_selector",
        "public",
        "secret",
        REJECTED_UNKNOWN_UNSUPPORTED,
    ),
    mutation(
        "D004-CS-MUT-SC03-M05",
        "replace",
        "debug_observation",
        "public_independent",
        "secret_dependent",
        REJECTED_UNKNOWN_UNSUPPORTED,
    ),
    mutation(
        "D004-CS-MUT-SC04-M01",
        "remove",
        "required_target_feature",
        "declared_present",
        "absent",
        REJECTED_UNSUPPORTED,
    ),
    mutation(
        "D004-CS-MUT-SC04-M02",
        "substitute",
        "intrinsic_identity",
        "declared_supported_intrinsic",
        "unsupported_intrinsic",
        REJECTED_UNSUPPORTED,
    ),
    mutation(
        "D004-CS-MUT-SC04-M03",
        "reverse",
        "lane_order",
        "declared_order",
        "reversed_order",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC04-M04",
        "substitute",
        "vector_width",
        "declared_width",
        "mismatched_width",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC04-M05",
        "substitute",
        "target_model_identity",
        "original",
        "substitute",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC04-M06",
        "select",
        "fallback_authorization",
        "declared",
        "undeclared_selected",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC05-M01",
        "move",
        "sampling_authority",
        "game_semantics",
        "specification_semantics",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC05-M02",
        "replace",
        "randomness_source",
        "explicit_sampling",
        "ambient_randomness",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC05-M03",
        "hide",
        "oracle_effect_visibility",
        "explicit",
        "hidden",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC05-M04",
        "replace",
        "sample_space_bound",
        "finite",
        "unbounded",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC05-M05",
        "substitute",
        "reduction_endpoint_identity",
        "original",
        "substitute",
        REJECTED,
    ),
    mutation(
        "D004-CS-MUT-SC05-M06",
        "alter",
        "symbolic_advantage_bound",
        "original",
        "altered",
        REJECTED,
    ),
];

const fn mutation(
    subject_id: &'static str,
    operator: &'static str,
    target: &'static str,
    baseline_value: &'static str,
    mutated_value: &'static str,
    allowed_states: &'static [&'static str],
) -> MutationModelSpec {
    MutationModelSpec {
        subject_id,
        operator,
        target,
        baseline_value,
        mutated_value,
        allowed_states,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseSubjectPreflight {
    pub(crate) subject_id: String,
    pub(crate) case: String,
    pub(crate) subject_kind: &'static str,
    pub(crate) subject_sha256: String,
    pub(crate) integrity_status: &'static str,
    pub(crate) candidate_execution: &'static str,
    pub(crate) evidence_status: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CaseSubjectErrorKind {
    Json(JsonErrorKind),
    MissingField,
    UnknownField,
    InvalidValue,
    NonCanonicalEncoding,
    SourceBinding,
    ManifestJoin,
    ManifestDigest,
    SubjectDigest,
    StructuralMismatch,
    DuplicateSubject,
    PersistedResult,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseSubjectError {
    pub(crate) kind: CaseSubjectErrorKind,
    pub(crate) path: String,
}

impl CaseSubjectError {
    fn new(kind: CaseSubjectErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CaseSubjectCatalog {
    value: JsonValue,
    canonical: Vec<u8>,
    digest: [u8; 32],
    preflights: Vec<CaseSubjectPreflight>,
}

impl CaseSubjectCatalog {
    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&self.digest)
    }

    pub(crate) fn preflights(&self) -> &[CaseSubjectPreflight] {
        &self.preflights
    }
}

pub(crate) fn parse_case_subject_catalog(
    input: &[u8],
    mutation_manifest: &JsonValue,
) -> Result<CaseSubjectCatalog, CaseSubjectError> {
    let value = strict_json::parse(input).map_err(|error| {
        CaseSubjectError::new(
            CaseSubjectErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })?;
    let canonical = strict_json::canonical_bytes(&value);
    if input != canonical_file_bytes(canonical.clone()) {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::NonCanonicalEncoding,
            "$",
        ));
    }
    reject_persisted_fields(&value, "$")?;
    let preflights = validate_catalog(&value, mutation_manifest)?;
    let digest = sha256::digest(&canonical);
    Ok(CaseSubjectCatalog {
        value,
        canonical,
        digest,
        preflights,
    })
}

fn validate_catalog(
    value: &JsonValue,
    mutation_manifest: &JsonValue,
) -> Result<Vec<CaseSubjectPreflight>, CaseSubjectError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &ROOT_FIELDS, "$")?;
    require_exact_string(
        root,
        "schema_version",
        "d004-case-subject-catalog-v0.1",
        "$",
    )?;
    require_exact_string(root, "suite_version", "d004-v0.4-draft", "$")?;
    require_exact_string(root, "status", "draft_unreviewed_input_only", "$")?;
    require_exact_string(root, "owner_protocol_review", "none", "$")?;
    require_exact_string(
        root,
        "canonicalization",
        "RFC8785_ASCII_INTEGER_SUBSET",
        "$",
    )?;
    require_exact_usize(root, "subject_count", SUBJECT_COUNT, "$")?;
    require_exact_usize(root, "positive_subject_count", POSITIVE_SUBJECT_COUNT, "$")?;
    require_exact_usize(root, "mutation_subject_count", MUTATION_SUBJECT_COUNT, "$")?;
    require_exact_string(root, "evidence_status", "none", "$")?;
    require_exact_strings(root, "nonclaims", &CASE_SUBJECT_NONCLAIMS, "$")?;
    validate_source_bindings(root)?;
    validate_execution_boundary(root)?;

    let positives = require_array(root, "positive_subjects", "$")?;
    let mutations = require_array(root, "mutation_subjects", "$")?;
    if positives.len() != POSITIVE_SUBJECT_COUNT || mutations.len() != MUTATION_SUBJECT_COUNT {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::StructuralMismatch,
            "$/subject_count",
        ));
    }

    let manifest = mutation_manifest.as_array().ok_or_else(|| {
        CaseSubjectError::new(CaseSubjectErrorKind::ManifestJoin, "$mutation_manifest")
    })?;
    if manifest.len() != MUTATION_SUBJECT_COUNT || MUTATIONS.len() != MUTATION_SUBJECT_COUNT {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::ManifestJoin,
            "$mutation_manifest",
        ));
    }
    let manifest_digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
        mutation_manifest,
    )));
    if manifest_digest != MUTATION_MANIFEST_CANONICAL_SHA256 {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::ManifestDigest,
            "$mutation_manifest",
        ));
    }

    let mut ids = BTreeSet::new();
    let mut subject_digests = BTreeSet::new();
    let mut mutation_ids = BTreeSet::new();
    let mut preflights = Vec::with_capacity(SUBJECT_COUNT);

    for (index, (value, spec)) in positives.iter().zip(POSITIVE_SPECS).enumerate() {
        let path = format!("$/positive_subjects/{index}");
        let preflight = validate_positive(value, spec, &path)?;
        insert_unique(&mut ids, &preflight.subject_id, &format!("{path}/id"))?;
        insert_unique(
            &mut subject_digests,
            &preflight.subject_sha256,
            &format!("{path}/subject_sha256"),
        )?;
        preflights.push(preflight);
    }

    for (index, (((value, manifest_record), mutation_spec), model_spec)) in mutations
        .iter()
        .zip(manifest)
        .zip(MUTATIONS)
        .zip(MUTATION_MODELS)
        .enumerate()
    {
        let path = format!("$/mutation_subjects/{index}");
        let preflight =
            validate_mutation(value, manifest_record, mutation_spec, model_spec, &path)?;
        insert_unique(&mut ids, &preflight.subject_id, &format!("{path}/id"))?;
        insert_unique(
            &mut subject_digests,
            &preflight.subject_sha256,
            &format!("{path}/subject_sha256"),
        )?;
        insert_unique(
            &mut mutation_ids,
            mutation_spec.id,
            &format!("{path}/mutation_id"),
        )?;
        preflights.push(preflight);
    }
    Ok(preflights)
}

fn validate_source_bindings(root: &BTreeMap<String, JsonValue>) -> Result<(), CaseSubjectError> {
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
        CaseSubjectError::new(
            CaseSubjectErrorKind::SourceBinding,
            format!("{suite_path}/path"),
        )
    })?;
    require_exact_string(suite, "raw_sha256", DECISION_SUITE_RAW_SHA256, suite_path).map_err(
        |_| {
            CaseSubjectError::new(
                CaseSubjectErrorKind::SourceBinding,
                format!("{suite_path}/raw_sha256"),
            )
        },
    )?;

    let manifest_path = "$/source_bindings/named_mutation_manifest";
    let manifest = require_object(
        require_field(bindings, "named_mutation_manifest", path)?,
        manifest_path,
    )?;
    exact_fields(manifest, &MANIFEST_BINDING_FIELDS, manifest_path)?;
    for (field, expected) in [
        (
            "path",
            "research/decisions/D-004/d004-v0.2-named-mutations.json",
        ),
        ("canonical_sha256", MUTATION_MANIFEST_CANONICAL_SHA256),
        ("raw_sha256", MUTATION_MANIFEST_RAW_SHA256),
    ] {
        require_exact_string(manifest, field, expected, manifest_path).map_err(|_| {
            CaseSubjectError::new(
                CaseSubjectErrorKind::SourceBinding,
                format!("{manifest_path}/{field}"),
            )
        })?;
    }
    Ok(())
}

fn validate_execution_boundary(root: &BTreeMap<String, JsonValue>) -> Result<(), CaseSubjectError> {
    let path = "$/execution_boundary";
    let boundary = require_object(require_field(root, "execution_boundary", "$")?, path)?;
    exact_fields(boundary, &EXECUTION_BOUNDARY_FIELDS, path)?;
    for field in ["candidate_adapter", "candidate_process", "candidate_tool"] {
        require_exact_string(boundary, field, "not_invoked", path)?;
    }
    require_exact_string(boundary, "network", "not_used", path)?;
    require_exact_string(boundary, "preflight_output_persistence", "none", path)
}

fn validate_positive(
    value: &JsonValue,
    spec: PositiveSpec,
    path: &str,
) -> Result<CaseSubjectPreflight, CaseSubjectError> {
    let record = require_object(value, path)?;
    exact_fields(record, &POSITIVE_RECORD_FIELDS, path)?;
    require_exact_string(record, "id", spec.id, path)?;
    require_exact_string(record, "case", spec.case.as_str(), path)?;
    validate_positive_expectation(record, path)?;

    let subject_path = format!("{path}/subject");
    let subject_value = require_field(record, "subject", path)?;
    let subject = require_object(subject_value, &subject_path)?;
    exact_fields(subject, &POSITIVE_SUBJECT_FIELDS, &subject_path)?;
    require_exact_string(
        subject,
        "schema_version",
        CASE_SUBJECT_SCHEMA,
        &subject_path,
    )?;
    require_exact_string(subject, "id", spec.id, &subject_path)?;
    require_exact_string(subject, "case", spec.case.as_str(), &subject_path)?;
    require_exact_string(subject, "kind", "suite-only-positive-case", &subject_path)?;
    require_exact_strings(
        subject,
        "relationship_scope",
        relationship_scope(spec.case),
        &subject_path,
    )?;
    require_object(
        require_field(subject, "model", &subject_path)?,
        &format!("{subject_path}/model"),
    )?;

    let digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
        subject_value,
    )));
    require_exact_string(record, "subject_sha256", &digest, path).map_err(|_| {
        CaseSubjectError::new(
            CaseSubjectErrorKind::SubjectDigest,
            format!("{path}/subject_sha256"),
        )
    })?;
    if digest != spec.subject_sha256 {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::StructuralMismatch,
            format!("{subject_path}/model"),
        ));
    }

    Ok(preflight(spec.id, spec.case, "positive", digest))
}

fn validate_positive_expectation(
    record: &BTreeMap<String, JsonValue>,
    record_path: &str,
) -> Result<(), CaseSubjectError> {
    let path = format!("{record_path}/declared_expectation");
    let expectation = require_object(
        require_field(record, "declared_expectation", record_path)?,
        &path,
    )?;
    exact_fields(expectation, &POSITIVE_EXPECTATION_FIELDS, &path)?;
    require_exact_string(expectation, "observation_level", "domain", &path)?;
    require_exact_strings(expectation, "allowed_domain_states", &["succeeded"], &path)?;
    require_exact_strings(
        expectation,
        "forbidden_domain_states",
        &["exhausted", "timeout"],
        &path,
    )
}

fn validate_mutation(
    value: &JsonValue,
    manifest_value: &JsonValue,
    mutation_spec: super::domain::MutationSpec,
    model_spec: MutationModelSpec,
    path: &str,
) -> Result<CaseSubjectPreflight, CaseSubjectError> {
    let record = require_object(value, path)?;
    exact_fields(record, &MUTATION_RECORD_FIELDS, path)?;
    require_exact_string(record, "id", model_spec.subject_id, path)?;
    require_exact_string(record, "case", mutation_spec.case.as_str(), path)?;
    require_exact_string(record, "mutation_id", mutation_spec.id, path)?;
    validate_manifest_join(record, manifest_value, mutation_spec, path)?;
    validate_mutation_expectation(record, model_spec.allowed_states, path)?;

    let subject_path = format!("{path}/subject");
    let subject_value = require_field(record, "subject", path)?;
    let subject = require_object(subject_value, &subject_path)?;
    exact_fields(subject, &MUTATION_SUBJECT_FIELDS, &subject_path)?;
    require_exact_string(
        subject,
        "schema_version",
        CASE_SUBJECT_SCHEMA,
        &subject_path,
    )?;
    require_exact_string(subject, "id", model_spec.subject_id, &subject_path)?;
    require_exact_string(subject, "case", mutation_spec.case.as_str(), &subject_path)?;
    require_exact_string(subject, "mutation_id", mutation_spec.id, &subject_path)?;
    require_exact_string(subject, "kind", "suite-only-named-mutation", &subject_path)?;
    require_exact_string(
        subject,
        "positive_subject_sha256",
        positive_subject_digest(mutation_spec.case),
        &subject_path,
    )
    .map_err(|_| {
        CaseSubjectError::new(
            CaseSubjectErrorKind::StructuralMismatch,
            format!("{subject_path}/positive_subject_sha256"),
        )
    })?;
    require_exact_strings(
        subject,
        "relationship_scope",
        relationship_scope(mutation_spec.case),
        &subject_path,
    )?;
    validate_mutation_model(subject, model_spec, &subject_path)?;

    let digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
        subject_value,
    )));
    require_exact_string(record, "subject_sha256", &digest, path).map_err(|_| {
        CaseSubjectError::new(
            CaseSubjectErrorKind::SubjectDigest,
            format!("{path}/subject_sha256"),
        )
    })?;
    Ok(preflight(
        model_spec.subject_id,
        mutation_spec.case,
        "named-mutation",
        digest,
    ))
}

fn validate_manifest_join(
    record: &BTreeMap<String, JsonValue>,
    manifest_value: &JsonValue,
    mutation_spec: super::domain::MutationSpec,
    record_path: &str,
) -> Result<(), CaseSubjectError> {
    let manifest_path = format!("$mutation_manifest/{}", mutation_spec.id);
    let manifest = require_object(manifest_value, &manifest_path)
        .map_err(|_| CaseSubjectError::new(CaseSubjectErrorKind::ManifestJoin, &manifest_path))?;
    exact_fields(manifest, &MANIFEST_RECORD_FIELDS, &manifest_path)
        .map_err(|_| CaseSubjectError::new(CaseSubjectErrorKind::ManifestJoin, &manifest_path))?;
    for (field, expected) in [
        ("id", mutation_spec.id),
        ("case", mutation_spec.case.as_str()),
        ("description", mutation_spec.description),
    ] {
        require_exact_string(manifest, field, expected, &manifest_path).map_err(|_| {
            CaseSubjectError::new(
                CaseSubjectErrorKind::ManifestJoin,
                format!("{manifest_path}/{field}"),
            )
        })?;
    }
    let digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
        manifest_value,
    )));
    require_exact_string(record, "manifest_record_sha256", &digest, record_path).map_err(|_| {
        CaseSubjectError::new(
            CaseSubjectErrorKind::ManifestDigest,
            format!("{record_path}/manifest_record_sha256"),
        )
    })
}

fn validate_mutation_expectation(
    record: &BTreeMap<String, JsonValue>,
    allowed_states: &[&str],
    record_path: &str,
) -> Result<(), CaseSubjectError> {
    let path = format!("{record_path}/declared_expectation");
    let expectation = require_object(
        require_field(record, "declared_expectation", record_path)?,
        &path,
    )?;
    exact_fields(expectation, &MUTATION_EXPECTATION_FIELDS, &path)?;
    require_exact_string(expectation, "observation_level", "domain", &path)?;
    require_exact_strings(expectation, "allowed_domain_states", allowed_states, &path)?;
    require_exact_strings(
        expectation,
        "forbidden_domain_states",
        &["exhausted", "succeeded", "timeout"],
        &path,
    )?;
    require_exact_string(
        expectation,
        "required_invalidation",
        "dependent_result",
        &path,
    )
}

fn validate_mutation_model(
    subject: &BTreeMap<String, JsonValue>,
    spec: MutationModelSpec,
    subject_path: &str,
) -> Result<(), CaseSubjectError> {
    let path = format!("{subject_path}/model");
    let model = require_object(require_field(subject, "model", subject_path)?, &path)?;
    exact_fields(model, &MUTATION_MODEL_FIELDS, &path)?;
    for (field, expected) in [
        ("kind", "suite-only-single-invariant-mutation"),
        ("operator", spec.operator),
        ("target", spec.target),
        ("baseline_value", spec.baseline_value),
        ("mutated_value", spec.mutated_value),
    ] {
        require_exact_string(model, field, expected, &path).map_err(|_| {
            CaseSubjectError::new(
                CaseSubjectErrorKind::StructuralMismatch,
                format!("{path}/{field}"),
            )
        })?;
    }
    if spec.baseline_value == spec.mutated_value {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::StructuralMismatch,
            format!("{path}/mutated_value"),
        ));
    }
    let dependent_path = format!("{path}/dependent_result");
    let dependent = require_object(
        require_field(model, "dependent_result", &path)?,
        &dependent_path,
    )?;
    exact_fields(dependent, &DEPENDENT_RESULT_FIELDS, &dependent_path)?;
    for (field, expected) in [
        ("id", "dependent_result"),
        ("required_target", spec.target),
        ("required_value", spec.baseline_value),
    ] {
        require_exact_string(dependent, field, expected, &dependent_path).map_err(|_| {
            CaseSubjectError::new(
                CaseSubjectErrorKind::StructuralMismatch,
                format!("{dependent_path}/{field}"),
            )
        })?;
    }
    Ok(())
}

fn preflight(id: &str, case: CaseId, kind: &'static str, digest: String) -> CaseSubjectPreflight {
    CaseSubjectPreflight {
        subject_id: id.to_owned(),
        case: case.as_str().to_owned(),
        subject_kind: kind,
        subject_sha256: digest,
        integrity_status: "accepted",
        candidate_execution: "not_performed",
        evidence_status: "none",
    }
}

fn relationship_scope(case: CaseId) -> &'static [&'static str] {
    match case {
        CaseId::Sc01 => &SC01_SCOPE,
        CaseId::Sc02 => &SC02_SCOPE,
        CaseId::Sc03 => &SC03_SCOPE,
        CaseId::Sc04 => &SC04_SCOPE,
        CaseId::Sc05 => &SC05_SCOPE,
    }
}

fn positive_subject_digest(case: CaseId) -> &'static str {
    POSITIVE_SPECS[case_index(case)].subject_sha256
}

fn case_index(case: CaseId) -> usize {
    CASES
        .iter()
        .position(|candidate| *candidate == case)
        .unwrap_or(CASES.len())
}

fn insert_unique(
    values: &mut BTreeSet<String>,
    value: &str,
    path: &str,
) -> Result<(), CaseSubjectError> {
    if !values.insert(value.to_owned()) {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::DuplicateSubject,
            path,
        ));
    }
    Ok(())
}

fn reject_persisted_fields(value: &JsonValue, path: &str) -> Result<(), CaseSubjectError> {
    match value {
        JsonValue::Object(object) => {
            for (field, child) in object {
                let child_path = format!("{path}/{field}");
                if FORBIDDEN_PERSISTED_FIELDS.contains(&field.as_str()) {
                    return Err(CaseSubjectError::new(
                        CaseSubjectErrorKind::PersistedResult,
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
) -> Result<&'a BTreeMap<String, JsonValue>, CaseSubjectError> {
    value
        .as_object()
        .ok_or_else(|| CaseSubjectError::new(CaseSubjectErrorKind::InvalidValue, path))
}

fn require_array<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<&'a [JsonValue], CaseSubjectError> {
    let path = format!("{parent}/{field}");
    require_field(object, field, parent)?
        .as_array()
        .ok_or_else(|| CaseSubjectError::new(CaseSubjectErrorKind::InvalidValue, path))
}

fn require_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    parent: &str,
) -> Result<&'a JsonValue, CaseSubjectError> {
    object.get(field).ok_or_else(|| {
        CaseSubjectError::new(
            CaseSubjectErrorKind::MissingField,
            format!("{parent}/{field}"),
        )
    })
}

fn exact_fields(
    object: &BTreeMap<String, JsonValue>,
    expected: &[&str],
    path: &str,
) -> Result<(), CaseSubjectError> {
    if let Some(field) = object
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::UnknownField,
            format!("{path}/{field}"),
        ));
    }
    for field in expected {
        if !object.contains_key(*field) {
            return Err(CaseSubjectError::new(
                CaseSubjectErrorKind::MissingField,
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
) -> Result<&'a str, CaseSubjectError> {
    let path = format!("{parent}/{field}");
    require_field(object, field, parent)?
        .as_str()
        .ok_or_else(|| CaseSubjectError::new(CaseSubjectErrorKind::InvalidValue, path))
}

fn require_exact_string(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &str,
    parent: &str,
) -> Result<(), CaseSubjectError> {
    let path = format!("{parent}/{field}");
    if require_string(object, field, parent)? != expected {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::InvalidValue,
            path,
        ));
    }
    Ok(())
}

fn require_exact_strings(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &[&str],
    parent: &str,
) -> Result<(), CaseSubjectError> {
    let path = format!("{parent}/{field}");
    let observed = require_field(object, field, parent)?
        .as_array()
        .ok_or_else(|| CaseSubjectError::new(CaseSubjectErrorKind::InvalidValue, &path))?;
    if observed.len() != expected.len()
        || observed
            .iter()
            .zip(expected)
            .any(|(actual, wanted)| actual.as_str() != Some(*wanted))
    {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::InvalidValue,
            path,
        ));
    }
    Ok(())
}

fn require_exact_usize(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: usize,
    parent: &str,
) -> Result<(), CaseSubjectError> {
    let path = format!("{parent}/{field}");
    let integer = require_field(object, field, parent)?
        .as_integer()
        .ok_or_else(|| CaseSubjectError::new(CaseSubjectErrorKind::InvalidValue, &path))?;
    if usize::try_from(integer).ok() != Some(expected) {
        return Err(CaseSubjectError::new(
            CaseSubjectErrorKind::InvalidValue,
            path,
        ));
    }
    Ok(())
}
