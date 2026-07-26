use std::collections::{BTreeMap, BTreeSet};

use super::domain::{
    CROSS_CUTTING_PROPOSAL_CLASS_STATUSES, CROSS_CUTTING_PROPOSAL_COUNT,
    IDENTITY_SUBSTITUTION_PROPOSALS, RELATIONSHIPS,
};
use super::sha256;
use super::strict_json::{self, JsonErrorKind, JsonValue};

pub(crate) const FIXTURE_CATALOG_PATH: &str =
    "research/decisions/D-004/d004-v0.3-cross-cutting-executable-fixtures.json";
pub(crate) const FIXTURE_CATALOG_SCHEMA: &str = "d004-cross-cutting-executable-fixtures-v0.1";
pub(crate) const FIXTURE_SUBJECT_SCHEMA: &str = "d004-cross-cutting-fixture-subject-v0.1";
pub(crate) const FIXTURE_CATALOG_CANONICAL_SHA256: &str =
    "0516a84260bcc4d8ebb64e0cd3416deb5c43a86b7f5cd882ca757c924e575767";
pub(crate) const FIXTURE_CATALOG_RAW_SHA256: &str =
    "5fea65960c47818243d41076dd96a6cab2dbd6d4038fd354a3f5ba30a12622ae";

const ROOT_FIELDS: [&str; 12] = [
    "schema_version",
    "suite_version",
    "status",
    "owner_protocol_review",
    "proposal_manifest",
    "canonicalization",
    "fixture_count",
    "class_counts",
    "fixtures",
    "execution_boundary",
    "evidence_status",
    "nonclaims",
];
const PROPOSAL_MANIFEST_FIELDS: [&str; 3] = ["path", "canonical_sha256", "raw_sha256"];
const CLASS_COUNT_FIELDS: [&str; 2] = ["class", "fixture_count"];
const FIXTURE_FIELDS: [&str; 5] = [
    "proposal_id",
    "proposal_record_sha256",
    "fixture_subject",
    "fixture_subject_sha256",
    "expected_observation",
];
const SUBJECT_FIELDS: [&str; 9] = [
    "schema_version",
    "proposal_id",
    "class",
    "case_scope",
    "relationship_scope",
    "layer",
    "mutation_kind",
    "target",
    "model",
];
const EXPECTATION_FIELDS: [&str; 5] = [
    "observation_level",
    "state",
    "required_invalidation",
    "match_rule",
    "capability_credit",
];
const EXECUTION_BOUNDARY_FIELDS: [&str; 5] = [
    "candidate_adapter",
    "candidate_process",
    "candidate_tool",
    "network",
    "preflight_output_persistence",
];
const MISSING_EDGE_MODEL_FIELDS: [&str; 4] = [
    "kind",
    "baseline_relationships",
    "mutated_relationships",
    "dependent_result",
];
const IDENTITY_MODEL_FIELDS: [&str; 5] = [
    "kind",
    "identity_namespace",
    "baseline_bindings",
    "mutated_bindings",
    "dependent_result",
];
const AMBIGUITY_MODEL_FIELDS: [&str; 4] = [
    "kind",
    "authority_key",
    "interpretations",
    "dependent_result",
];
const UNSUPPORTED_MODEL_FIELDS: [&str; 4] =
    ["kind", "support_domain", "request", "dependent_result"];
const RESOURCE_MODEL_FIELDS: [&str; 4] = ["kind", "resource_domain", "request", "dependent_result"];

pub(crate) const FIXTURE_NONCLAIMS: [&str; 11] = [
    "no candidate adapter executed",
    "no candidate process or tool invoked",
    "no candidate observation or case verdict produced",
    "no candidate capability or capability absence established",
    "fixture preflight is not D-004 execution evidence",
    "fixture-domain unsupported is not candidate-adapter inability",
    "fixture-domain exhaustion is not replay-level timeout or exhaustion",
    "no D-004 evidence epoch frozen",
    "no semantic-strata candidate selected",
    "no D-003 disposition inferred",
    "no S3b implementation or roadmap readiness movement authorized",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FixtureState {
    Rejected,
    Unsupported,
    Exhausted,
}

impl FixtureState {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Rejected => "rejected",
            Self::Unsupported => "unsupported",
            Self::Exhausted => "exhausted",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FixturePreflight {
    pub(crate) proposal_id: String,
    pub(crate) fixture_subject_sha256: String,
    pub(crate) loader_status: &'static str,
    pub(crate) observed_state: FixtureState,
    pub(crate) observed_invalidation: &'static str,
    pub(crate) matched: bool,
    pub(crate) candidate_execution: &'static str,
    pub(crate) capability_credit: &'static str,
    pub(crate) evidence_status: &'static str,
    pub(crate) replay_ceiling_exercised: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum FixtureErrorKind {
    Json(JsonErrorKind),
    MissingField,
    UnknownField,
    InvalidValue,
    NonCanonicalEncoding,
    ProposalJoin,
    ProposalDigest,
    SubjectDigest,
    StructuralMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FixtureError {
    pub(crate) kind: FixtureErrorKind,
    pub(crate) path: String,
}

impl FixtureError {
    fn new(kind: FixtureErrorKind, path: impl Into<String>) -> Self {
        Self {
            kind,
            path: path.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FixtureCatalog {
    value: JsonValue,
    canonical: Vec<u8>,
    digest: [u8; 32],
    preflights: Vec<FixturePreflight>,
}

impl FixtureCatalog {
    pub(crate) fn value(&self) -> &JsonValue {
        &self.value
    }

    pub(crate) fn canonical_bytes(&self) -> &[u8] {
        &self.canonical
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&self.digest)
    }

    pub(crate) fn preflights(&self) -> &[FixturePreflight] {
        &self.preflights
    }
}

pub(crate) fn parse_fixture_catalog(
    input: &[u8],
    proposal_manifest: &JsonValue,
) -> Result<FixtureCatalog, FixtureError> {
    let value = strict_json::parse(input).map_err(|error| {
        FixtureError::new(
            FixtureErrorKind::Json(error.kind),
            format!("$@{}", error.offset),
        )
    })?;
    let canonical = strict_json::canonical_bytes(&value);
    if input != canonical_file_bytes(canonical.clone()) {
        return Err(FixtureError::new(
            FixtureErrorKind::NonCanonicalEncoding,
            "$",
        ));
    }
    let preflights = validate_catalog(&value, proposal_manifest)?;
    let digest = sha256::digest(&canonical);
    Ok(FixtureCatalog {
        value,
        canonical,
        digest,
        preflights,
    })
}

fn validate_catalog(
    value: &JsonValue,
    proposal_manifest: &JsonValue,
) -> Result<Vec<FixturePreflight>, FixtureError> {
    let root = require_object(value, "$")?;
    exact_fields(root, &ROOT_FIELDS, "$")?;
    require_exact_string(root, "schema_version", FIXTURE_CATALOG_SCHEMA, "$")?;
    require_exact_string(root, "suite_version", "d004-v0.3-draft", "$")?;
    require_exact_string(root, "status", "draft_unreviewed_input_only", "$")?;
    require_exact_string(root, "owner_protocol_review", "none", "$")?;
    require_exact_string(
        root,
        "canonicalization",
        "RFC8785_ASCII_INTEGER_SUBSET",
        "$",
    )?;
    require_exact_usize(root, "fixture_count", CROSS_CUTTING_PROPOSAL_COUNT, "$")?;
    require_exact_string(root, "evidence_status", "none", "$")?;
    require_exact_strings(root, "nonclaims", &FIXTURE_NONCLAIMS, "$")?;
    validate_proposal_manifest_binding(root)?;
    validate_class_counts(root)?;
    validate_execution_boundary(root)?;

    let proposal_root = require_object(proposal_manifest, "$proposal_manifest")?;
    let proposals = require_field(proposal_root, "proposals", "$proposal_manifest")?
        .as_array()
        .ok_or_else(|| {
            FixtureError::new(
                FixtureErrorKind::InvalidValue,
                "$proposal_manifest/proposals",
            )
        })?;
    let fixtures = require_field(root, "fixtures", "$")?
        .as_array()
        .ok_or_else(|| FixtureError::new(FixtureErrorKind::InvalidValue, "$/fixtures"))?;
    if proposals.len() != CROSS_CUTTING_PROPOSAL_COUNT
        || fixtures.len() != CROSS_CUTTING_PROPOSAL_COUNT
    {
        return Err(FixtureError::new(
            FixtureErrorKind::ProposalJoin,
            "$/fixtures",
        ));
    }

    let mut ids = BTreeSet::new();
    let mut class_counts = BTreeMap::new();
    let mut preflights = Vec::with_capacity(CROSS_CUTTING_PROPOSAL_COUNT);
    for (index, (fixture, proposal)) in fixtures.iter().zip(proposals).enumerate() {
        let path = format!("$/fixtures/{index}");
        let record = require_object(fixture, &path)?;
        exact_fields(record, &FIXTURE_FIELDS, &path)?;
        let proposal_record = require_object(proposal, "$proposal_manifest/proposals")?;
        let proposal_id = require_string(proposal_record, "id", "$proposal_manifest/proposals")?;
        require_exact_string(record, "proposal_id", proposal_id, &path).map_err(|_| {
            FixtureError::new(
                FixtureErrorKind::ProposalJoin,
                format!("{path}/proposal_id"),
            )
        })?;
        if !ids.insert(proposal_id) {
            return Err(FixtureError::new(
                FixtureErrorKind::ProposalJoin,
                format!("{path}/proposal_id"),
            ));
        }

        let proposal_digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(proposal)));
        require_exact_string(record, "proposal_record_sha256", &proposal_digest, &path).map_err(
            |_| {
                FixtureError::new(
                    FixtureErrorKind::ProposalDigest,
                    format!("{path}/proposal_record_sha256"),
                )
            },
        )?;

        let subject = require_field(record, "fixture_subject", &path)?;
        validate_subject_common(subject, proposal_record, proposal_id, &path)?;
        let subject_digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(subject)));
        require_exact_string(record, "fixture_subject_sha256", &subject_digest, &path).map_err(
            |_| {
                FixtureError::new(
                    FixtureErrorKind::SubjectDigest,
                    format!("{path}/fixture_subject_sha256"),
                )
            },
        )?;

        let class = require_string(proposal_record, "class", "$proposal_manifest/proposals")?;
        *class_counts.entry(class).or_insert(0_usize) += 1;
        let target = require_string(proposal_record, "target", "$proposal_manifest/proposals")?;
        let model = require_field(
            require_object(subject, &format!("{path}/fixture_subject"))?,
            "model",
            &format!("{path}/fixture_subject"),
        )?;
        let observed_state = evaluate_model(
            class,
            target,
            model,
            &format!("{path}/fixture_subject/model"),
        )?;
        validate_expectation(record, proposal_record, observed_state, &path)?;
        preflights.push(FixturePreflight {
            proposal_id: proposal_id.to_owned(),
            fixture_subject_sha256: subject_digest,
            loader_status: "accepted",
            observed_state,
            observed_invalidation: "dependent_result",
            matched: true,
            candidate_execution: "not_performed",
            capability_credit: "none",
            evidence_status: "none",
            replay_ceiling_exercised: false,
        });
    }
    for status in CROSS_CUTTING_PROPOSAL_CLASS_STATUSES {
        if class_counts.get(status.class).copied() != Some(status.proposal_count) {
            return Err(FixtureError::new(
                FixtureErrorKind::ProposalJoin,
                "$/fixtures",
            ));
        }
    }
    Ok(preflights)
}

fn validate_proposal_manifest_binding(
    root: &BTreeMap<String, JsonValue>,
) -> Result<(), FixtureError> {
    let path = "$/proposal_manifest";
    let binding = require_object(require_field(root, "proposal_manifest", "$")?, path)?;
    exact_fields(binding, &PROPOSAL_MANIFEST_FIELDS, path)?;
    require_exact_string(
        binding,
        "path",
        "research/decisions/D-004/d004-v0.2-cross-cutting-fixture-proposals.json",
        path,
    )?;
    require_exact_string(
        binding,
        "canonical_sha256",
        "85407a4a43b5a6bf450ea905fe858482f2f79abb4cbe8ee8690bddc1753d0912",
        path,
    )?;
    require_exact_string(
        binding,
        "raw_sha256",
        "d3d58cbeb0d2a90987680cd00bc70caf53518be730a71d0d55ba2a7b50544481",
        path,
    )
}

fn validate_class_counts(root: &BTreeMap<String, JsonValue>) -> Result<(), FixtureError> {
    let statuses = require_field(root, "class_counts", "$")?
        .as_array()
        .ok_or_else(|| FixtureError::new(FixtureErrorKind::InvalidValue, "$/class_counts"))?;
    if statuses.len() != CROSS_CUTTING_PROPOSAL_CLASS_STATUSES.len() {
        return Err(FixtureError::new(
            FixtureErrorKind::InvalidValue,
            "$/class_counts",
        ));
    }
    for (index, (value, expected)) in statuses
        .iter()
        .zip(CROSS_CUTTING_PROPOSAL_CLASS_STATUSES)
        .enumerate()
    {
        let path = format!("$/class_counts/{index}");
        let status = require_object(value, &path)?;
        exact_fields(status, &CLASS_COUNT_FIELDS, &path)?;
        require_exact_string(status, "class", expected.class, &path)?;
        require_exact_usize(status, "fixture_count", expected.proposal_count, &path)?;
    }
    Ok(())
}

fn validate_execution_boundary(root: &BTreeMap<String, JsonValue>) -> Result<(), FixtureError> {
    let path = "$/execution_boundary";
    let boundary = require_object(require_field(root, "execution_boundary", "$")?, path)?;
    exact_fields(boundary, &EXECUTION_BOUNDARY_FIELDS, path)?;
    for field in ["candidate_adapter", "candidate_process", "candidate_tool"] {
        require_exact_string(boundary, field, "not_invoked", path)?;
    }
    require_exact_string(boundary, "network", "not_used", path)?;
    require_exact_string(boundary, "preflight_output_persistence", "none", path)
}

fn validate_subject_common(
    subject: &JsonValue,
    proposal: &BTreeMap<String, JsonValue>,
    proposal_id: &str,
    fixture_path: &str,
) -> Result<(), FixtureError> {
    let path = format!("{fixture_path}/fixture_subject");
    let subject = require_object(subject, &path)?;
    exact_fields(subject, &SUBJECT_FIELDS, &path)?;
    require_exact_string(subject, "schema_version", FIXTURE_SUBJECT_SCHEMA, &path)?;
    require_exact_string(subject, "proposal_id", proposal_id, &path)?;
    for field in [
        "class",
        "case_scope",
        "relationship_scope",
        "layer",
        "mutation_kind",
        "target",
    ] {
        if subject.get(field) != proposal.get(field) {
            return Err(FixtureError::new(
                FixtureErrorKind::ProposalJoin,
                format!("{path}/{field}"),
            ));
        }
    }
    Ok(())
}

fn validate_expectation(
    fixture: &BTreeMap<String, JsonValue>,
    proposal: &BTreeMap<String, JsonValue>,
    observed_state: FixtureState,
    fixture_path: &str,
) -> Result<(), FixtureError> {
    let path = format!("{fixture_path}/expected_observation");
    let expected = require_object(
        require_field(fixture, "expected_observation", fixture_path)?,
        &path,
    )?;
    exact_fields(expected, &EXPECTATION_FIELDS, &path)?;
    for (expected_field, proposal_field) in [
        ("observation_level", "observation_level"),
        ("state", "expected_state"),
        ("required_invalidation", "required_invalidation"),
        ("match_rule", "match_rule"),
        ("capability_credit", "capability_credit"),
    ] {
        if expected.get(expected_field) != proposal.get(proposal_field) {
            return Err(FixtureError::new(
                FixtureErrorKind::ProposalJoin,
                format!("{path}/{expected_field}"),
            ));
        }
    }
    require_exact_string(expected, "observation_level", "domain", &path)?;
    require_exact_string(expected, "state", observed_state.as_str(), &path).map_err(|_| {
        FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            format!("{path}/state"),
        )
    })?;
    require_exact_string(expected, "required_invalidation", "dependent_result", &path)?;
    require_exact_string(expected, "match_rule", "required_not_sufficient", &path)?;
    require_exact_string(expected, "capability_credit", "none", &path)
}

fn evaluate_model(
    class: &str,
    target: &str,
    model: &JsonValue,
    path: &str,
) -> Result<FixtureState, FixtureError> {
    match class {
        "missing-edge" => evaluate_missing_edge(target, model, path),
        "identity-substitution" => evaluate_identity_substitution(target, model, path),
        "ambiguity" => evaluate_ambiguity(target, model, path),
        "unsupported" => evaluate_unsupported(target, model, path),
        "resource-exhaustion" => evaluate_resource_exhaustion(model, path),
        _ => Err(FixtureError::new(FixtureErrorKind::InvalidValue, path)),
    }
}

fn evaluate_missing_edge(
    target: &str,
    model: &JsonValue,
    path: &str,
) -> Result<FixtureState, FixtureError> {
    let model = require_object(model, path)?;
    exact_fields(model, &MISSING_EDGE_MODEL_FIELDS, path)?;
    require_exact_string(model, "kind", "missing-edge", path)?;
    require_exact_strings(model, "baseline_relationships", &RELATIONSHIPS, path)?;
    let expected_mutated = RELATIONSHIPS
        .iter()
        .copied()
        .filter(|relationship| *relationship != target)
        .collect::<Vec<_>>();
    if expected_mutated.len() + 1 != RELATIONSHIPS.len() {
        return Err(FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            format!("{path}/mutated_relationships"),
        ));
    }
    require_exact_strings(model, "mutated_relationships", &expected_mutated, path).map_err(
        |_| {
            FixtureError::new(
                FixtureErrorKind::StructuralMismatch,
                format!("{path}/mutated_relationships"),
            )
        },
    )?;
    let dependent_path = format!("{path}/dependent_result");
    let dependent = require_object(
        require_field(model, "dependent_result", path)?,
        &dependent_path,
    )?;
    exact_fields(
        dependent,
        &["id", "required_relationships"],
        &dependent_path,
    )?;
    require_exact_string(dependent, "id", "dependent_result", &dependent_path)?;
    require_exact_strings(
        dependent,
        "required_relationships",
        &[target],
        &dependent_path,
    )
    .map_err(|_| {
        FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            format!("{dependent_path}/required_relationships"),
        )
    })?;
    Ok(FixtureState::Rejected)
}

fn evaluate_identity_substitution(
    target: &str,
    model: &JsonValue,
    path: &str,
) -> Result<FixtureState, FixtureError> {
    let model = require_object(model, path)?;
    exact_fields(model, &IDENTITY_MODEL_FIELDS, path)?;
    require_exact_string(model, "kind", "identity-substitution", path)?;
    require_exact_string(
        model,
        "identity_namespace",
        "d004-suite-only-identity-v0.1",
        path,
    )?;
    let baseline = require_field(model, "baseline_bindings", path)?
        .as_array()
        .ok_or_else(|| {
            FixtureError::new(
                FixtureErrorKind::InvalidValue,
                format!("{path}/baseline_bindings"),
            )
        })?;
    let mutated = require_field(model, "mutated_bindings", path)?
        .as_array()
        .ok_or_else(|| {
            FixtureError::new(
                FixtureErrorKind::InvalidValue,
                format!("{path}/mutated_bindings"),
            )
        })?;
    if baseline.len() != IDENTITY_SUBSTITUTION_PROPOSALS.len()
        || mutated.len() != IDENTITY_SUBSTITUTION_PROPOSALS.len()
    {
        return Err(FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            path,
        ));
    }
    let mut changed = 0_usize;
    for (index, proposal) in IDENTITY_SUBSTITUTION_PROPOSALS.iter().enumerate() {
        let original = suite_identity_digest(proposal.target, "original");
        let substitute = suite_identity_digest(proposal.target, "substitute");
        validate_binding(
            &baseline[index],
            proposal.target,
            &original,
            &format!("{path}/baseline_bindings/{index}"),
        )?;
        let expected_mutated = if proposal.target == target {
            changed += 1;
            &substitute
        } else {
            &original
        };
        validate_binding(
            &mutated[index],
            proposal.target,
            expected_mutated,
            &format!("{path}/mutated_bindings/{index}"),
        )?;
    }
    if changed != 1 {
        return Err(FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            format!("{path}/mutated_bindings"),
        ));
    }
    let dependent_path = format!("{path}/dependent_result");
    let dependent = require_object(
        require_field(model, "dependent_result", path)?,
        &dependent_path,
    )?;
    exact_fields(dependent, &["id", "required_binding"], &dependent_path)?;
    require_exact_string(dependent, "id", "dependent_result", &dependent_path)?;
    let required_path = format!("{dependent_path}/required_binding");
    validate_binding(
        require_field(dependent, "required_binding", &dependent_path)?,
        target,
        &suite_identity_digest(target, "original"),
        &required_path,
    )?;
    Ok(FixtureState::Rejected)
}

fn validate_binding(
    value: &JsonValue,
    slot: &str,
    identity: &str,
    path: &str,
) -> Result<(), FixtureError> {
    let binding = require_object(value, path)?;
    exact_fields(binding, &["slot", "identity_sha256"], path)?;
    require_exact_string(binding, "slot", slot, path)?;
    require_exact_string(binding, "identity_sha256", identity, path).map_err(|_| {
        FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            format!("{path}/identity_sha256"),
        )
    })
}

fn suite_identity_digest(slot: &str, variant: &str) -> String {
    let mut bytes = b"d004-fixture-identity-v0.1\0".to_vec();
    bytes.extend_from_slice(slot.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(variant.as_bytes());
    sha256::hex(&sha256::digest(&bytes))
}

fn evaluate_ambiguity(
    target: &str,
    model: &JsonValue,
    path: &str,
) -> Result<FixtureState, FixtureError> {
    let model = require_object(model, path)?;
    exact_fields(model, &AMBIGUITY_MODEL_FIELDS, path)?;
    require_exact_string(model, "kind", "ambiguity", path)?;
    require_exact_string(model, "authority_key", target, path)?;
    let interpretations = require_field(model, "interpretations", path)?
        .as_array()
        .ok_or_else(|| {
            FixtureError::new(
                FixtureErrorKind::InvalidValue,
                format!("{path}/interpretations"),
            )
        })?;
    if interpretations.len() != 2 {
        return Err(FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            format!("{path}/interpretations"),
        ));
    }
    for (index, (id, value)) in [
        ("interpretation-a", "d004-suite-only-interpretation-a"),
        ("interpretation-b", "d004-suite-only-interpretation-b"),
    ]
    .into_iter()
    .enumerate()
    {
        let interpretation_path = format!("{path}/interpretations/{index}");
        let interpretation = require_object(&interpretations[index], &interpretation_path)?;
        exact_fields(interpretation, &["id", "value"], &interpretation_path)?;
        require_exact_string(interpretation, "id", id, &interpretation_path)?;
        require_exact_string(interpretation, "value", value, &interpretation_path).map_err(
            |_| {
                FixtureError::new(
                    FixtureErrorKind::StructuralMismatch,
                    format!("{interpretation_path}/value"),
                )
            },
        )?;
    }
    let dependent_path = format!("{path}/dependent_result");
    let dependent = require_object(
        require_field(model, "dependent_result", path)?,
        &dependent_path,
    )?;
    exact_fields(
        dependent,
        &["id", "requires_unique_authority"],
        &dependent_path,
    )?;
    require_exact_string(dependent, "id", "dependent_result", &dependent_path)?;
    require_exact_bool(
        dependent,
        "requires_unique_authority",
        true,
        &dependent_path,
    )?;
    Ok(FixtureState::Rejected)
}

fn evaluate_unsupported(
    target: &str,
    model: &JsonValue,
    path: &str,
) -> Result<FixtureState, FixtureError> {
    let model = require_object(model, path)?;
    exact_fields(model, &UNSUPPORTED_MODEL_FIELDS, path)?;
    require_exact_string(model, "kind", "unsupported", path)?;
    let domain_path = format!("{path}/support_domain");
    let domain = require_object(require_field(model, "support_domain", path)?, &domain_path)?;
    exact_fields(
        domain,
        &["id", "supported_operations", "unsupported_operations"],
        &domain_path,
    )?;
    require_exact_string(domain, "id", "d004-suite-only-support-v0.1", &domain_path)?;
    require_exact_strings(
        domain,
        "supported_operations",
        &["baseline_operation"],
        &domain_path,
    )?;
    require_exact_strings(domain, "unsupported_operations", &[target], &domain_path).map_err(
        |_| {
            FixtureError::new(
                FixtureErrorKind::StructuralMismatch,
                format!("{domain_path}/unsupported_operations"),
            )
        },
    )?;
    let request_path = format!("{path}/request");
    let request = require_object(require_field(model, "request", path)?, &request_path)?;
    exact_fields(request, &["operation"], &request_path)?;
    require_exact_string(request, "operation", target, &request_path)?;
    validate_dependent_result(model, path)?;
    Ok(FixtureState::Unsupported)
}

fn evaluate_resource_exhaustion(
    model: &JsonValue,
    path: &str,
) -> Result<FixtureState, FixtureError> {
    let model = require_object(model, path)?;
    exact_fields(model, &RESOURCE_MODEL_FIELDS, path)?;
    require_exact_string(model, "kind", "resource-exhaustion", path)?;
    let domain_path = format!("{path}/resource_domain");
    let domain = require_object(require_field(model, "resource_domain", path)?, &domain_path)?;
    exact_fields(domain, &["id", "unit", "limit"], &domain_path)?;
    require_exact_string(domain, "id", "d004-suite-only-resource-v0.1", &domain_path)?;
    require_exact_string(domain, "unit", "abstract_work_item", &domain_path)?;
    require_exact_usize(domain, "limit", 8, &domain_path)?;
    let request_path = format!("{path}/request");
    let request = require_object(require_field(model, "request", path)?, &request_path)?;
    exact_fields(request, &["work_items"], &request_path)?;
    let work_items = require_field(request, "work_items", &request_path)?
        .as_array()
        .ok_or_else(|| {
            FixtureError::new(
                FixtureErrorKind::InvalidValue,
                format!("{request_path}/work_items"),
            )
        })?;
    if work_items.len() != 9 {
        return Err(FixtureError::new(
            FixtureErrorKind::StructuralMismatch,
            format!("{request_path}/work_items"),
        ));
    }
    for (index, item) in work_items.iter().enumerate() {
        let expected = format!("work-{:02}", index + 1);
        if item.as_str() != Some(expected.as_str()) {
            return Err(FixtureError::new(
                FixtureErrorKind::StructuralMismatch,
                format!("{request_path}/work_items/{index}"),
            ));
        }
    }
    validate_dependent_result(model, path)?;
    Ok(FixtureState::Exhausted)
}

fn validate_dependent_result(
    model: &BTreeMap<String, JsonValue>,
    path: &str,
) -> Result<(), FixtureError> {
    let dependent_path = format!("{path}/dependent_result");
    let dependent = require_object(
        require_field(model, "dependent_result", path)?,
        &dependent_path,
    )?;
    exact_fields(dependent, &["id"], &dependent_path)?;
    require_exact_string(dependent, "id", "dependent_result", &dependent_path)
}

fn canonical_file_bytes(mut canonical: Vec<u8>) -> Vec<u8> {
    canonical.push(b'\n');
    canonical
}

fn require_object<'a>(
    value: &'a JsonValue,
    path: &str,
) -> Result<&'a BTreeMap<String, JsonValue>, FixtureError> {
    value
        .as_object()
        .ok_or_else(|| FixtureError::new(FixtureErrorKind::InvalidValue, path))
}

fn require_field<'a>(
    object: &'a BTreeMap<String, JsonValue>,
    field: &str,
    path: &str,
) -> Result<&'a JsonValue, FixtureError> {
    object
        .get(field)
        .ok_or_else(|| FixtureError::new(FixtureErrorKind::MissingField, format!("{path}/{field}")))
}

fn exact_fields(
    object: &BTreeMap<String, JsonValue>,
    expected: &[&str],
    path: &str,
) -> Result<(), FixtureError> {
    if let Some(field) = object
        .keys()
        .find(|field| !expected.contains(&field.as_str()))
    {
        return Err(FixtureError::new(
            FixtureErrorKind::UnknownField,
            format!("{path}/{field}"),
        ));
    }
    for field in expected {
        if !object.contains_key(*field) {
            return Err(FixtureError::new(
                FixtureErrorKind::MissingField,
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
) -> Result<&'a str, FixtureError> {
    let path = format!("{parent}/{field}");
    require_field(object, field, parent)?
        .as_str()
        .ok_or_else(|| FixtureError::new(FixtureErrorKind::InvalidValue, path))
}

fn require_exact_string(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &str,
    parent: &str,
) -> Result<(), FixtureError> {
    let path = format!("{parent}/{field}");
    if require_string(object, field, parent)? != expected {
        return Err(FixtureError::new(FixtureErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn require_exact_strings(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: &[&str],
    parent: &str,
) -> Result<(), FixtureError> {
    let path = format!("{parent}/{field}");
    let observed = require_field(object, field, parent)?
        .as_array()
        .ok_or_else(|| FixtureError::new(FixtureErrorKind::InvalidValue, &path))?;
    if observed.len() != expected.len()
        || observed
            .iter()
            .zip(expected)
            .any(|(actual, wanted)| actual.as_str() != Some(*wanted))
    {
        return Err(FixtureError::new(FixtureErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn require_exact_bool(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: bool,
    parent: &str,
) -> Result<(), FixtureError> {
    let path = format!("{parent}/{field}");
    if require_field(object, field, parent)? != &JsonValue::Bool(expected) {
        return Err(FixtureError::new(FixtureErrorKind::InvalidValue, path));
    }
    Ok(())
}

fn require_exact_usize(
    object: &BTreeMap<String, JsonValue>,
    field: &str,
    expected: usize,
    parent: &str,
) -> Result<(), FixtureError> {
    let path = format!("{parent}/{field}");
    let integer = require_field(object, field, parent)?
        .as_integer()
        .ok_or_else(|| FixtureError::new(FixtureErrorKind::InvalidValue, &path))?;
    if usize::try_from(integer).ok() != Some(expected) {
        return Err(FixtureError::new(FixtureErrorKind::InvalidValue, path));
    }
    Ok(())
}
