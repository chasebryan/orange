//! Draft-unfrozen substrate checks for the D-004 semantic-strata decision suite.
//!
//! These checks bind the pre-epoch packet and plan 25 symmetric candidate-case
//! slots. They execute no candidate adapter, freeze no epoch, select no
//! semantic architecture, and create no D-004 result or product evidence.

#[path = "d004_support/cases.rs"]
mod cases;
#[path = "d004_support/domain.rs"]
mod domain;
#[path = "d004_support/fixtures.rs"]
mod fixtures;
#[path = "d004_support/packet.rs"]
mod packet;
#[path = "d004_support/runner.rs"]
mod runner;
#[path = "d005_support/sha256.rs"]
mod sha256;
#[path = "d005_support/strict_json.rs"]
mod strict_json;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use cases::MUTATIONS;
use domain::{
    BUDGETS, CANDIDATES, CASE_SCOPED_CROSS_CUTTING_PROPOSALS, CASE_VERDICTS, CASES,
    CROSS_CUTTING_PROPOSAL_CLASS_STATUSES, CROSS_CUTTING_PROPOSAL_COUNT,
    CROSS_CUTTING_PROPOSAL_NONCLAIMS, DOMAIN_OBSERVATION_STATES, HARD_GATES,
    IDENTITY_SUBSTITUTION_PROPOSALS, INPUT_BINDINGS, InputBindingId, MISSING_EDGE_PROPOSAL_IDS,
    NONCLAIMS, PROTOCOL_GAPS, RELATIONSHIPS, REQUIRED_CANDIDATE_CASES, SOURCE_ROLES,
    UNRESOLVED_CROSS_CUTTING_FIXTURE_CLASSES,
};
use fixtures::{
    FIXTURE_CATALOG_CANONICAL_SHA256, FIXTURE_CATALOG_PATH, FIXTURE_CATALOG_RAW_SHA256,
    FIXTURE_NONCLAIMS, FixtureErrorKind, FixtureState, parse_fixture_catalog,
};
use packet::{
    CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256,
    CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256, MUTATION_MANIFEST_SHA256, PacketErrorKind,
    canonical_cross_cutting_fixture_proposal_manifest_bytes,
    canonical_cross_cutting_fixture_proposal_manifest_file_bytes, canonical_draft_packet_bytes,
    canonical_draft_packet_file_bytes, canonical_mutation_manifest_bytes,
    canonical_mutation_manifest_file_bytes, cross_cutting_fixture_proposal_manifest_digest_hex,
    mutation_manifest_digest_hex, parse_cross_cutting_fixture_proposal_manifest,
    parse_draft_packet, parse_mutation_manifest,
};
use runner::{ReplayError, ReplayInputs};
use strict_json::{JsonErrorKind, JsonValue};

const CHECKED_IN_PACKET: &[u8] =
    include_bytes!("../../../../research/decisions/D-004/d004-v0.3-draft-packet.json");
const NAMED_MUTATIONS: &[u8] =
    include_bytes!("../../../../research/decisions/D-004/d004-v0.2-named-mutations.json");
const CROSS_CUTTING_FIXTURE_PROPOSALS: &[u8] = include_bytes!(
    "../../../../research/decisions/D-004/d004-v0.2-cross-cutting-fixture-proposals.json"
);
const CROSS_CUTTING_EXECUTABLE_FIXTURES: &[u8] = include_bytes!(
    "../../../../research/decisions/D-004/d004-v0.3-cross-cutting-executable-fixtures.json"
);
const DECISION_SUITE: &[u8] = include_bytes!("../../../../docs/SEMANTIC_STRATA_DECISION_SUITE.md");
const PRODUCT_FORM_DECISION_PACKET: &[u8] =
    include_bytes!("../../../../docs/PRODUCT_FORM_DECISION_PACKET.md");
const ACCEPTED_S2_LANGUAGE: &[u8] = include_bytes!("../../../../docs/LANGUAGE_2026.md");
const USER_JOURNEYS: &[u8] = include_bytes!("../../../../docs/USER_JOURNEYS.md");
const ACCEPTED_S3A_SEMANTICS: &[u8] = include_bytes!("../../../../docs/SEMANTICS_2026.md");
const ACCEPTED_S3A_OEP: &[u8] =
    include_bytes!("../../../../docs/governance/oeps/OEP-0003-orange-2026-typed-literals.md");
const S3A_CONFORMANCE_RUNNER: &[u8] = include_bytes!("../../orangec/tests/s3a_conformance.rs");
const PERMANENT_S3A_FIXTURE: &[u8] = include_bytes!("../../../fixtures/typed-answer.or");
const INVALID_DUPLICATE_SPEC: &[u8] =
    include_bytes!("../../../fixtures/s3a/invalid-duplicate-spec.or");
const INVALID_INT_MAGNITUDE: &[u8] =
    include_bytes!("../../../fixtures/s3a/invalid-int-magnitude.or");
const INVALID_NEGATIVE_WORD: &[u8] =
    include_bytes!("../../../fixtures/s3a/invalid-negative-word.or");
const INVALID_TYPED_IMPL: &[u8] = include_bytes!("../../../fixtures/s3a/invalid-typed-impl.or");
const INVALID_UNSUPPORTED_TYPE: &[u8] =
    include_bytes!("../../../fixtures/s3a/invalid-unsupported-type.or");
const INVALID_WORD_RANGE: &[u8] = include_bytes!("../../../fixtures/s3a/invalid-word-range.or");
const INVALID_WORD_WIDTH: &[u8] = include_bytes!("../../../fixtures/s3a/invalid-word-width.or");
const VALID_EMPTY_MIXED: &[u8] = include_bytes!("../../../fixtures/s3a/valid-empty-mixed.or");
const VALID_INT_RADICES: &[u8] = include_bytes!("../../../fixtures/s3a/valid-int-radices.or");
const VALID_WORD8_BOUNDARIES: &[u8] =
    include_bytes!("../../../fixtures/s3a/valid-word8-boundaries.or");

fn checked_in_replay_inputs() -> ReplayInputs<'static> {
    ReplayInputs::new([
        NAMED_MUTATIONS,
        DECISION_SUITE,
        PRODUCT_FORM_DECISION_PACKET,
        ACCEPTED_S2_LANGUAGE,
        USER_JOURNEYS,
        ACCEPTED_S3A_SEMANTICS,
        ACCEPTED_S3A_OEP,
        S3A_CONFORMANCE_RUNNER,
        PERMANENT_S3A_FIXTURE,
        INVALID_DUPLICATE_SPEC,
        INVALID_INT_MAGNITUDE,
        INVALID_NEGATIVE_WORD,
        INVALID_TYPED_IMPL,
        INVALID_UNSUPPORTED_TYPE,
        INVALID_WORD_RANGE,
        INVALID_WORD_WIDTH,
        VALID_EMPTY_MIXED,
        VALID_INT_RADICES,
        VALID_WORD8_BOUNDARIES,
        CROSS_CUTTING_FIXTURE_PROPOSALS,
        CROSS_CUTTING_EXECUTABLE_FIXTURES,
    ])
}

fn canonical_json_file_bytes(value: &JsonValue) -> Vec<u8> {
    let mut bytes = strict_json::canonical_bytes(value);
    bytes.push(b'\n');
    bytes
}

fn json_object_mut(value: &mut JsonValue) -> &mut BTreeMap<String, JsonValue> {
    match value {
        JsonValue::Object(object) => object,
        _ => panic!("expected JSON object"),
    }
}

fn json_array_mut(value: &mut JsonValue) -> &mut Vec<JsonValue> {
    match value {
        JsonValue::Array(array) => array,
        _ => panic!("expected JSON array"),
    }
}

fn fixture_record_mut(catalog: &mut JsonValue, index: usize) -> &mut BTreeMap<String, JsonValue> {
    let fixtures = json_array_mut(
        json_object_mut(catalog)
            .get_mut("fixtures")
            .expect("fixture catalog array"),
    );
    json_object_mut(&mut fixtures[index])
}

fn rehash_fixture_subject(catalog: &mut JsonValue, index: usize) {
    let record = fixture_record_mut(catalog, index);
    let digest = sha256::hex(&sha256::digest(&strict_json::canonical_bytes(
        record.get("fixture_subject").expect("fixture subject"),
    )));
    record.insert(
        "fixture_subject_sha256".to_owned(),
        JsonValue::String(digest),
    );
}

#[test]
fn d004_pre_epoch_contract_has_exact_cardinalities_and_semantics() {
    assert_eq!(
        CANDIDATES.map(|candidate| candidate.as_str()),
        ["ST-REL", "ST-UNI", "ST-DUAL", "ST-MIRROR", "ST-HOST",]
    );
    assert_eq!(
        CASES.map(|case| case.as_str()),
        ["SC-01", "SC-02", "SC-03", "SC-04", "SC-05",]
    );
    assert_eq!(CANDIDATES.len() * CASES.len(), REQUIRED_CANDIDATE_CASES);
    assert_eq!(
        RELATIONSHIPS,
        [
            "SR-01", "SR-02", "SR-03", "SR-04", "SR-05", "SR-06", "SR-07", "SR-08", "SR-09",
            "SR-10", "SR-11", "SR-12", "SR-13", "SR-14",
        ]
    );
    assert_eq!(
        HARD_GATES,
        [
            "SS-G01", "SS-G02", "SS-G03", "SS-G04", "SS-G05", "SS-G06", "SS-G07", "SS-G08",
            "SS-G09", "SS-G10",
        ]
    );
    assert_eq!(
        SOURCE_ROLES,
        [
            "Specification",
            "Implementation",
            "Machine Implementation",
            "Game",
            "Proof",
        ]
    );
    assert_eq!(
        DOMAIN_OBSERVATION_STATES,
        [
            "succeeded",
            "rejected",
            "unknown",
            "timeout",
            "unsupported",
            "exhausted",
        ]
    );
    assert_eq!(CASE_VERDICTS, ["pass", "fail"]);

    assert_eq!(MUTATIONS.len(), 26);
    let mut per_case = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for mutation in MUTATIONS {
        assert!(ids.insert(mutation.id));
        assert!(mutation.id.starts_with(mutation.case.as_str()));
        assert!(!mutation.description.is_empty());
        *per_case.entry(mutation.case).or_insert(0_usize) += 1;
    }
    assert_eq!(
        CASES.map(|case| per_case.get(&case).copied().unwrap_or(0)),
        [4, 5, 5, 6, 6]
    );
    assert_eq!(
        MUTATIONS.map(|mutation| mutation.id),
        [
            "SC-01-M01",
            "SC-01-M02",
            "SC-01-M03",
            "SC-01-M04",
            "SC-02-M01",
            "SC-02-M02",
            "SC-02-M03",
            "SC-02-M04",
            "SC-02-M05",
            "SC-03-M01",
            "SC-03-M02",
            "SC-03-M03",
            "SC-03-M04",
            "SC-03-M05",
            "SC-04-M01",
            "SC-04-M02",
            "SC-04-M03",
            "SC-04-M04",
            "SC-04-M05",
            "SC-04-M06",
            "SC-05-M01",
            "SC-05-M02",
            "SC-05-M03",
            "SC-05-M04",
            "SC-05-M05",
            "SC-05-M06",
        ]
    );
}

#[test]
fn d004_pre_epoch_contract_preserves_every_freeze_blocker_and_budget() {
    assert_eq!(
        UNRESOLVED_CROSS_CUTTING_FIXTURE_CLASSES,
        [
            "ambiguity",
            "missing-edge",
            "identity-substitution",
            "unsupported",
            "resource-exhaustion",
        ]
    );
    assert_eq!(
        PROTOCOL_GAPS,
        [
            "five positive subjects absent",
            "26 named-mutation subjects absent",
            "ambiguity fixture sufficiency review unresolved",
            "missing-edge fixture sufficiency review unresolved",
            "identity-substitution fixture sufficiency review unresolved",
            "unsupported fixture sufficiency review unresolved",
            "resource-exhaustion fixture sufficiency review unresolved",
            "replay repetition count unresolved",
        ]
    );
    assert_eq!(BUDGETS.max_packet_bytes, 262_144);
    assert_eq!(BUDGETS.max_json_depth, 32);
    assert_eq!(BUDGETS.max_json_nodes, 16_384);
    assert_eq!(BUDGETS.max_string_bytes, 16_384);
    assert_eq!(BUDGETS.case_wall_seconds, 900);
    assert_eq!(BUDGETS.case_peak_memory_bytes, 4_294_967_296);
    assert_eq!(BUDGETS.case_temp_storage_bytes, 2_147_483_648);
    assert_eq!(BUDGETS.case_output_bytes, 268_435_456);
    assert_eq!(BUDGETS.candidate_owner_hours, 24);
    assert_eq!(BUDGETS.correction_owner_hours, 4);
}

#[test]
fn reused_sha256_and_strict_json_primitives_fail_closed() {
    assert_eq!(
        sha256::hex(&sha256::digest(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        strict_json::parse(br#"{"a":1,"a":2}"#)
            .expect_err("duplicate key")
            .kind,
        JsonErrorKind::DuplicateKey
    );
    assert_eq!(
        strict_json::parse(br#"{"a":1.0}"#)
            .expect_err("floating point")
            .kind,
        JsonErrorKind::FloatingPoint
    );
    assert_eq!(
        strict_json::parse(&[b'"', 0xff, b'"'])
            .expect_err("invalid UTF-8")
            .kind,
        JsonErrorKind::InvalidUtf8
    );
    assert_eq!(
        strict_json::parse(&vec![b' '; BUDGETS.max_packet_bytes + 1])
            .expect_err("packet bound")
            .kind,
        JsonErrorKind::InputTooLarge
    );

    let mut too_deep = vec![b'['; BUDGETS.max_json_depth + 1];
    too_deep.push(b'0');
    too_deep.extend(std::iter::repeat_n(b']', BUDGETS.max_json_depth + 1));
    assert_eq!(
        strict_json::parse(&too_deep).expect_err("depth bound").kind,
        JsonErrorKind::DepthLimit
    );
}

#[test]
fn named_mutation_manifest_has_exact_canonical_bytes_and_digests() {
    assert_eq!(NAMED_MUTATIONS, canonical_mutation_manifest_file_bytes());
    let value = parse_mutation_manifest(NAMED_MUTATIONS).expect("checked-in manifest");
    assert_eq!(
        value.as_array().expect("manifest array").len(),
        MUTATIONS.len()
    );
    assert_eq!(
        strict_json::canonical_bytes(&value),
        canonical_mutation_manifest_bytes()
    );
    assert_eq!(mutation_manifest_digest_hex(), MUTATION_MANIFEST_SHA256);
    assert_eq!(
        mutation_manifest_digest_hex(),
        "970999d998cdc202a6caa4e2f798017416c88211a5b6b8508132a07cc9080c0c"
    );
    assert_eq!(
        sha256::hex(&sha256::digest(NAMED_MUTATIONS)),
        "1d46d6d66c0704fcaa462c625dcac2e72150497bb075322c5e076ea42898be54"
    );
    let mut noncanonical = Vec::with_capacity(NAMED_MUTATIONS.len() + 1);
    noncanonical.push(b' ');
    noncanonical.extend_from_slice(NAMED_MUTATIONS);
    assert_eq!(
        parse_mutation_manifest(&noncanonical)
            .expect_err("noncanonical manifest transport")
            .kind,
        PacketErrorKind::NonCanonicalEncoding
    );
}

#[test]
fn cross_cutting_fixture_proposals_are_closed_canonical_and_candidate_neutral() {
    assert_eq!(
        CROSS_CUTTING_FIXTURE_PROPOSALS,
        canonical_cross_cutting_fixture_proposal_manifest_file_bytes()
    );
    let value = parse_cross_cutting_fixture_proposal_manifest(CROSS_CUTTING_FIXTURE_PROPOSALS)
        .expect("checked-in proposal manifest");
    assert_eq!(
        strict_json::canonical_bytes(&value),
        canonical_cross_cutting_fixture_proposal_manifest_bytes()
    );
    assert_eq!(
        cross_cutting_fixture_proposal_manifest_digest_hex(),
        CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256
    );
    assert_eq!(
        CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256,
        "457c14e7d41f677b21af254af45e331b24e6c685a7d7aa8eae556ced5bd7be65"
    );
    assert_eq!(
        sha256::hex(&sha256::digest(CROSS_CUTTING_FIXTURE_PROPOSALS)),
        "171c7b88d54fe2bd7ddb4c220adb63f006e07c35391018b914482ace17cf7e93"
    );

    let root = value.as_object().expect("proposal root");
    assert_eq!(root.len(), 9);
    assert_eq!(
        root.get("schema_version").and_then(JsonValue::as_str),
        Some("d004-cross-cutting-fixture-proposals-v0.2")
    );
    assert_eq!(
        root.get("status").and_then(JsonValue::as_str),
        Some("draft_unreviewed")
    );
    assert_eq!(
        root.get("owner_protocol_review")
            .and_then(JsonValue::as_str),
        Some("none")
    );
    assert_eq!(
        root.get("executable_inputs_status")
            .and_then(JsonValue::as_str),
        Some("absent")
    );
    let class_statuses = root
        .get("class_statuses")
        .and_then(JsonValue::as_array)
        .expect("class statuses");
    assert_eq!(
        class_statuses.len(),
        CROSS_CUTTING_PROPOSAL_CLASS_STATUSES.len()
    );
    let expected_class_status_fields = BTreeSet::from([
        "class",
        "proposal_count",
        "proposal_status",
        "executable_fixture_count",
        "coverage_status",
        "freeze_blocker",
    ]);
    for (status, expected) in class_statuses
        .iter()
        .zip(CROSS_CUTTING_PROPOSAL_CLASS_STATUSES)
    {
        let status = status.as_object().expect("closed class status");
        assert_eq!(
            status.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected_class_status_fields
        );
        assert_eq!(
            status.get("class").and_then(JsonValue::as_str),
            Some(expected.class)
        );
        assert_eq!(
            status.get("proposal_count").and_then(JsonValue::as_integer),
            Some(i64::try_from(expected.proposal_count).expect("proposal count"))
        );
        assert_eq!(
            status.get("proposal_status").and_then(JsonValue::as_str),
            Some("draft_unreviewed")
        );
        assert_eq!(
            status
                .get("executable_fixture_count")
                .and_then(JsonValue::as_integer),
            Some(0)
        );
        assert_eq!(
            status.get("coverage_status").and_then(JsonValue::as_str),
            Some("unresolved")
        );
        assert_eq!(status.get("freeze_blocker"), Some(&JsonValue::Bool(true)));
    }
    assert_eq!(root.get("replay_repetitions"), Some(&JsonValue::Null));
    assert_eq!(
        root.get("evidence_status").and_then(JsonValue::as_str),
        Some("none")
    );
    assert_eq!(
        root.get("nonclaims"),
        Some(&strict_json::strings(CROSS_CUTTING_PROPOSAL_NONCLAIMS))
    );

    let proposals = root
        .get("proposals")
        .and_then(JsonValue::as_array)
        .expect("proposal records");
    assert_eq!(proposals.len(), CROSS_CUTTING_PROPOSAL_COUNT);
    let expected_fields = BTreeSet::from([
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
    ]);
    let expected_cases = strict_json::strings(CASES.map(|case| case.as_str()));
    let expected_relationships = strict_json::strings(RELATIONSHIPS);
    let mut ids = BTreeSet::new();
    let mut class_counts = BTreeMap::new();
    for (index, proposal) in proposals.iter().enumerate() {
        let record = proposal.as_object().expect("closed proposal record");
        assert_eq!(
            record.keys().map(String::as_str).collect::<BTreeSet<_>>(),
            expected_fields
        );
        assert!(
            ids.insert(
                record
                    .get("id")
                    .and_then(JsonValue::as_str)
                    .expect("proposal id")
            )
        );
        let class = record
            .get("class")
            .and_then(JsonValue::as_str)
            .expect("proposal class");
        *class_counts.entry(class).or_insert(0_usize) += 1;
        assert!(!matches!(class, "replay-ceiling" | "repetition"));
        assert!(
            !record
                .get("id")
                .and_then(JsonValue::as_str)
                .expect("proposal id")
                .starts_with("D004-XF-RP-")
        );
        assert_eq!(
            record.get("layer").and_then(JsonValue::as_str),
            Some("structural")
        );
        assert_eq!(
            record
                .get("required_invalidation")
                .and_then(JsonValue::as_str),
            Some("dependent_result")
        );
        assert_eq!(
            record.get("match_rule").and_then(JsonValue::as_str),
            Some("required_not_sufficient")
        );
        assert_eq!(
            record.get("capability_credit").and_then(JsonValue::as_str),
            Some("none")
        );
        assert_eq!(
            record.get("observation_level").and_then(JsonValue::as_str),
            Some("domain")
        );
        let mutation_kind = record
            .get("mutation_kind")
            .and_then(JsonValue::as_str)
            .expect("mutation kind");
        assert!(!mutation_kind.contains("repetition"));
        assert!(!mutation_kind.contains("replay_ceiling"));

        if index < RELATIONSHIPS.len() {
            let relationship = RELATIONSHIPS[index];
            assert_eq!(
                record.get("id").and_then(JsonValue::as_str),
                Some(MISSING_EDGE_PROPOSAL_IDS[index])
            );
            assert_eq!(record.get("case_scope"), Some(&expected_cases));
            assert_eq!(
                record.get("class").and_then(JsonValue::as_str),
                Some("missing-edge")
            );
            assert_eq!(
                record.get("relationship_scope"),
                Some(&strict_json::strings([relationship]))
            );
            assert_eq!(
                record.get("mutation_kind").and_then(JsonValue::as_str),
                Some("remove_required_relationship_descriptor")
            );
            assert_eq!(
                record.get("target").and_then(JsonValue::as_str),
                Some(relationship)
            );
            assert_eq!(
                record.get("expected_state").and_then(JsonValue::as_str),
                Some("rejected")
            );
        } else if index < RELATIONSHIPS.len() + IDENTITY_SUBSTITUTION_PROPOSALS.len() {
            let identity = IDENTITY_SUBSTITUTION_PROPOSALS[index - RELATIONSHIPS.len()];
            assert_eq!(
                record.get("id").and_then(JsonValue::as_str),
                Some(identity.id)
            );
            assert_eq!(record.get("case_scope"), Some(&expected_cases));
            assert_eq!(
                record.get("class").and_then(JsonValue::as_str),
                Some("identity-substitution")
            );
            assert_eq!(
                record.get("relationship_scope"),
                Some(&expected_relationships)
            );
            assert_eq!(
                record.get("mutation_kind").and_then(JsonValue::as_str),
                Some("substitute_bound_identity")
            );
            assert_eq!(
                record.get("target").and_then(JsonValue::as_str),
                Some(identity.target)
            );
            assert_eq!(
                record.get("expected_state").and_then(JsonValue::as_str),
                Some("rejected")
            );
        } else {
            let scoped = CASE_SCOPED_CROSS_CUTTING_PROPOSALS
                [index - RELATIONSHIPS.len() - IDENTITY_SUBSTITUTION_PROPOSALS.len()];
            assert_eq!(
                record.get("id").and_then(JsonValue::as_str),
                Some(scoped.id)
            );
            assert_eq!(
                record.get("class").and_then(JsonValue::as_str),
                Some(scoped.class)
            );
            assert_eq!(
                record.get("case_scope"),
                Some(&strict_json::strings([scoped.case.as_str()]))
            );
            assert_eq!(
                record.get("relationship_scope"),
                Some(&strict_json::strings(
                    scoped.relationship_scope.iter().copied()
                ))
            );
            assert_eq!(
                record.get("mutation_kind").and_then(JsonValue::as_str),
                Some(scoped.mutation_kind)
            );
            assert_eq!(
                record.get("target").and_then(JsonValue::as_str),
                Some(scoped.target)
            );
            assert_eq!(
                record.get("expected_state").and_then(JsonValue::as_str),
                Some(scoped.expected_state)
            );
        }
    }
    assert_eq!(ids.len(), CROSS_CUTTING_PROPOSAL_COUNT);
    assert_eq!(
        class_counts,
        BTreeMap::from([
            ("ambiguity", 5),
            ("identity-substitution", 10),
            ("missing-edge", 14),
            ("resource-exhaustion", 5),
            ("unsupported", 5),
        ])
    );

    let canonical = String::from_utf8(canonical_cross_cutting_fixture_proposal_manifest_bytes())
        .expect("UTF-8");
    let unknown = canonical.replacen('{', "{\"unknown\":true,", 1);
    assert_eq!(
        parse_cross_cutting_fixture_proposal_manifest(unknown.as_bytes())
            .expect_err("unknown proposal root field")
            .kind,
        PacketErrorKind::UnknownField
    );
    let missing = canonical.replace("\"evidence_status\":\"none\",", "");
    assert_eq!(
        parse_cross_cutting_fixture_proposal_manifest(missing.as_bytes())
            .expect_err("missing proposal root field")
            .kind,
        PacketErrorKind::MissingField
    );
    let weakened = canonical.replace("\"status\":\"draft_unreviewed\"", "\"status\":\"reviewed\"");
    assert_eq!(
        parse_cross_cutting_fixture_proposal_manifest(weakened.as_bytes())
            .expect_err("review was not performed")
            .kind,
        PacketErrorKind::InvalidValue
    );
    let semantic_drifts = [
        canonical.replace(
            "d004-cross-cutting-fixture-proposals-v0.2",
            "d004-cross-cutting-fixture-proposals-v0.1",
        ),
        canonical.replacen("\"proposal_count\":5", "\"proposal_count\":6", 1),
        canonical.replacen("\"freeze_blocker\":true", "\"freeze_blocker\":false", 1),
        canonical.replace(
            "\"case_scope\":[\"SC-01\"],\"class\":\"ambiguity\",\"expected_state\":\"rejected\",\"id\":\"D004-XF-AMB-SC01\"",
            "\"case_scope\":[\"SC-02\"],\"class\":\"ambiguity\",\"expected_state\":\"rejected\",\"id\":\"D004-XF-AMB-SC01\"",
        ),
        canonical.replacen(
            "\"mutation_kind\":\"exercise_preregistered_unsupported_behavior\",\"observation_level\":\"domain\",\"relationship_scope\":[]",
            "\"mutation_kind\":\"exercise_preregistered_unsupported_behavior\",\"observation_level\":\"domain\",\"relationship_scope\":[\"SR-01\"]",
            1,
        ),
        canonical.replace(
            "\"class\":\"ambiguity\",\"expected_state\":\"rejected\",\"id\":\"D004-XF-AMB-SC01\"",
            "\"class\":\"unsupported\",\"expected_state\":\"rejected\",\"id\":\"D004-XF-AMB-SC01\"",
        ),
        canonical.replace(
            "\"expected_state\":\"unsupported\",\"id\":\"D004-XF-US-SC01\"",
            "\"expected_state\":\"rejected\",\"id\":\"D004-XF-US-SC01\"",
        ),
        canonical.replace(
            "\"expected_state\":\"exhausted\",\"id\":\"D004-XF-RE-SC01\"",
            "\"expected_state\":\"timeout\",\"id\":\"D004-XF-RE-SC01\"",
        ),
        canonical.replacen(
            "\"observation_level\":\"domain\"",
            "\"observation_level\":\"adapter\"",
            1,
        ),
        canonical
            .replace("D004-XF-AMB-SC01", "D004-XF-AMB-TEMP")
            .replace("D004-XF-AMB-SC02", "D004-XF-AMB-SC01")
            .replace("D004-XF-AMB-TEMP", "D004-XF-AMB-SC02"),
    ];
    for drift in semantic_drifts {
        assert_eq!(
            parse_cross_cutting_fixture_proposal_manifest(drift.as_bytes())
                .expect_err("proposal catalog semantic drift")
                .kind,
            PacketErrorKind::InvalidValue
        );
    }
    assert_eq!(
        parse_cross_cutting_fixture_proposal_manifest(canonical.as_bytes())
            .expect_err("proposal manifest without canonical trailing LF")
            .kind,
        PacketErrorKind::NonCanonicalEncoding
    );
}

#[test]
fn cross_cutting_executable_fixture_catalog_is_exact_addressed_and_input_only() {
    let proposals = parse_cross_cutting_fixture_proposal_manifest(CROSS_CUTTING_FIXTURE_PROPOSALS)
        .expect("checked-in proposal manifest");
    let catalog = parse_fixture_catalog(CROSS_CUTTING_EXECUTABLE_FIXTURES, &proposals)
        .expect("checked-in executable fixture catalog");

    assert_eq!(
        FIXTURE_CATALOG_PATH,
        "research/decisions/D-004/d004-v0.3-cross-cutting-executable-fixtures.json"
    );
    assert_eq!(CROSS_CUTTING_EXECUTABLE_FIXTURES.len(), 67_329);
    assert_eq!(
        catalog.canonical_bytes(),
        &CROSS_CUTTING_EXECUTABLE_FIXTURES[..CROSS_CUTTING_EXECUTABLE_FIXTURES.len() - 1]
    );
    assert_eq!(catalog.digest_hex(), FIXTURE_CATALOG_CANONICAL_SHA256);
    assert_eq!(
        catalog.digest_hex(),
        "ca08308161244e9541803aa8008dd1624a2101f77da8b656cf0c5deff8a60703"
    );
    assert_eq!(
        sha256::hex(&sha256::digest(CROSS_CUTTING_EXECUTABLE_FIXTURES)),
        FIXTURE_CATALOG_RAW_SHA256
    );
    assert_eq!(
        FIXTURE_CATALOG_RAW_SHA256,
        "268b4065028f1af9c9ec912ae8884c150094189f5d782963f42ed6ed4cca6ce0"
    );
    assert_eq!(catalog.preflights().len(), CROSS_CUTTING_PROPOSAL_COUNT);

    let mut ids = BTreeSet::new();
    let mut subject_digests = BTreeSet::new();
    for preflight in catalog.preflights() {
        assert!(ids.insert(preflight.proposal_id.as_str()));
        assert!(subject_digests.insert(preflight.fixture_subject_sha256.as_str()));
        assert_eq!(preflight.loader_status, "accepted");
        assert_eq!(preflight.observed_invalidation, "dependent_result");
        assert!(preflight.matched);
        assert_eq!(preflight.candidate_execution, "not_performed");
        assert_eq!(preflight.capability_credit, "none");
        assert_eq!(preflight.evidence_status, "none");
        assert!(!preflight.replay_ceiling_exercised);
    }
    assert_eq!(
        catalog
            .preflights()
            .iter()
            .filter(|preflight| preflight.observed_state == FixtureState::Rejected)
            .count(),
        29
    );
    assert_eq!(
        catalog
            .preflights()
            .iter()
            .filter(|preflight| preflight.observed_state == FixtureState::Unsupported)
            .count(),
        5
    );
    assert_eq!(
        catalog
            .preflights()
            .iter()
            .filter(|preflight| preflight.observed_state == FixtureState::Exhausted)
            .count(),
        5
    );

    let root = catalog.value().as_object().expect("catalog root");
    assert_eq!(
        root.get("nonclaims"),
        Some(&strict_json::strings(FIXTURE_NONCLAIMS))
    );
    let fixtures = root
        .get("fixtures")
        .and_then(JsonValue::as_array)
        .expect("fixture entries");
    assert!(fixtures.iter().all(|fixture| {
        let fixture = fixture.as_object().expect("fixture record");
        !fixture.contains_key("loader_status")
            && !fixture.contains_key("observed_state")
            && !fixture.contains_key("matched")
            && !fixture.contains_key("result")
    }));
    for candidate in CANDIDATES {
        assert!(
            !CROSS_CUTTING_EXECUTABLE_FIXTURES
                .windows(candidate.as_str().len())
                .any(|window| window == candidate.as_str().as_bytes())
        );
    }
}

#[test]
fn cross_cutting_executable_fixture_loader_fails_closed_before_any_observation() {
    let proposals = parse_cross_cutting_fixture_proposal_manifest(CROSS_CUTTING_FIXTURE_PROPOSALS)
        .expect("checked-in proposal manifest");
    let baseline = strict_json::parse(CROSS_CUTTING_EXECUTABLE_FIXTURES)
        .expect("checked-in fixture catalog JSON");

    let mut noncanonical = Vec::with_capacity(CROSS_CUTTING_EXECUTABLE_FIXTURES.len() + 1);
    noncanonical.push(b' ');
    noncanonical.extend_from_slice(CROSS_CUTTING_EXECUTABLE_FIXTURES);
    assert_eq!(
        parse_fixture_catalog(&noncanonical, &proposals)
            .expect_err("noncanonical transport")
            .kind,
        FixtureErrorKind::NonCanonicalEncoding
    );

    let mut unknown = baseline.clone();
    json_object_mut(&mut unknown).insert("unknown".to_owned(), JsonValue::Bool(true));
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&unknown), &proposals)
            .expect_err("unknown root field")
            .kind,
        FixtureErrorKind::UnknownField
    );

    let mut missing = baseline.clone();
    json_object_mut(&mut missing).remove("nonclaims");
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&missing), &proposals)
            .expect_err("missing root field")
            .kind,
        FixtureErrorKind::MissingField
    );

    let mut reordered = baseline.clone();
    json_array_mut(
        json_object_mut(&mut reordered)
            .get_mut("fixtures")
            .expect("fixtures"),
    )
    .swap(0, 1);
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&reordered), &proposals)
            .expect_err("proposal order substitution")
            .kind,
        FixtureErrorKind::ProposalJoin
    );

    let mut proposal_digest = baseline.clone();
    fixture_record_mut(&mut proposal_digest, 0).insert(
        "proposal_record_sha256".to_owned(),
        JsonValue::String("0".repeat(64)),
    );
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&proposal_digest), &proposals)
            .expect_err("proposal record identity substitution")
            .kind,
        FixtureErrorKind::ProposalDigest
    );

    let mut missing_edge = baseline.clone();
    {
        let subject = json_object_mut(
            fixture_record_mut(&mut missing_edge, 0)
                .get_mut("fixture_subject")
                .expect("subject"),
        );
        let model = json_object_mut(subject.get_mut("model").expect("model"));
        json_array_mut(
            model
                .get_mut("mutated_relationships")
                .expect("mutated relationships"),
        )
        .push(JsonValue::String("SR-01".to_owned()));
    }
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&missing_edge), &proposals)
            .expect_err("stale subject digest")
            .kind,
        FixtureErrorKind::SubjectDigest
    );
    rehash_fixture_subject(&mut missing_edge, 0);
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&missing_edge), &proposals)
            .expect_err("rehashed but structurally invalid missing-edge fixture")
            .kind,
        FixtureErrorKind::StructuralMismatch
    );

    let mut identity = baseline.clone();
    {
        let subject = json_object_mut(
            fixture_record_mut(&mut identity, 14)
                .get_mut("fixture_subject")
                .expect("subject"),
        );
        let model = json_object_mut(subject.get_mut("model").expect("model"));
        let baseline_bindings = json_array_mut(
            model
                .get_mut("baseline_bindings")
                .expect("baseline bindings"),
        );
        let original = json_object_mut(&mut baseline_bindings[0])
            .get("identity_sha256")
            .expect("original identity")
            .clone();
        let mutated_bindings =
            json_array_mut(model.get_mut("mutated_bindings").expect("mutated bindings"));
        json_object_mut(&mut mutated_bindings[0]).insert("identity_sha256".to_owned(), original);
    }
    rehash_fixture_subject(&mut identity, 14);
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&identity), &proposals)
            .expect_err("identity fixture must contain exactly one substitution")
            .kind,
        FixtureErrorKind::StructuralMismatch
    );

    let mut ambiguity = baseline.clone();
    {
        let subject = json_object_mut(
            fixture_record_mut(&mut ambiguity, 24)
                .get_mut("fixture_subject")
                .expect("subject"),
        );
        let model = json_object_mut(subject.get_mut("model").expect("model"));
        let interpretations =
            json_array_mut(model.get_mut("interpretations").expect("interpretations"));
        let first = json_object_mut(&mut interpretations[0])
            .get("value")
            .expect("first interpretation")
            .clone();
        json_object_mut(&mut interpretations[1]).insert("value".to_owned(), first);
    }
    rehash_fixture_subject(&mut ambiguity, 24);
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&ambiguity), &proposals)
            .expect_err("ambiguity fixture requires distinct interpretations")
            .kind,
        FixtureErrorKind::StructuralMismatch
    );

    let mut unsupported = baseline.clone();
    {
        let subject = json_object_mut(
            fixture_record_mut(&mut unsupported, 29)
                .get_mut("fixture_subject")
                .expect("subject"),
        );
        let model = json_object_mut(subject.get_mut("model").expect("model"));
        let domain = json_object_mut(model.get_mut("support_domain").expect("support domain"));
        json_array_mut(
            domain
                .get_mut("unsupported_operations")
                .expect("unsupported operations"),
        )
        .clear();
    }
    rehash_fixture_subject(&mut unsupported, 29);
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&unsupported), &proposals)
            .expect_err("absence is not an explicit unsupported observation")
            .kind,
        FixtureErrorKind::StructuralMismatch
    );

    let mut resource = baseline.clone();
    {
        let subject = json_object_mut(
            fixture_record_mut(&mut resource, 34)
                .get_mut("fixture_subject")
                .expect("subject"),
        );
        let model = json_object_mut(subject.get_mut("model").expect("model"));
        let request = json_object_mut(model.get_mut("request").expect("request"));
        json_array_mut(request.get_mut("work_items").expect("work items")).pop();
    }
    rehash_fixture_subject(&mut resource, 34);
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&resource), &proposals)
            .expect_err("domain exhaustion must remain deterministic and explicit")
            .kind,
        FixtureErrorKind::StructuralMismatch
    );

    let mut capability = baseline;
    let expectation = json_object_mut(
        fixture_record_mut(&mut capability, 0)
            .get_mut("expected_observation")
            .expect("expectation"),
    );
    expectation.insert(
        "capability_credit".to_owned(),
        JsonValue::String("granted".to_owned()),
    );
    assert_eq!(
        parse_fixture_catalog(&canonical_json_file_bytes(&capability), &proposals)
            .expect_err("capability upgrade")
            .kind,
        FixtureErrorKind::ProposalJoin
    );
}

#[test]
fn draft_packet_has_exact_canonical_bytes_digest_and_zero_baseline() {
    assert_eq!(CHECKED_IN_PACKET, canonical_draft_packet_file_bytes());
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in packet");
    assert_eq!(packet.canonical_bytes(), canonical_draft_packet_bytes());
    assert_eq!(packet.digest(), &sha256::digest(packet.canonical_bytes()));
    assert_eq!(
        packet.digest_hex(),
        "7fb725d374e39eeae8a3a01ecf6033d53205f61d28ab94371e35ee0b59a07e58"
    );
    assert_eq!(
        sha256::hex(&sha256::digest(CHECKED_IN_PACKET)),
        "0095a821d2a94b6163538965707b3ebadc554c9260b66bd45c943b8cefb9e739"
    );
    assert_eq!(
        parse_draft_packet(packet.canonical_bytes())
            .expect_err("packet without canonical trailing LF")
            .kind,
        PacketErrorKind::NonCanonicalEncoding
    );

    let root = packet.value().as_object().expect("packet root");
    assert_eq!(root.get("epoch"), Some(&JsonValue::Null));
    assert_eq!(root.get("selection"), Some(&JsonValue::Null));
    assert_eq!(root.get("conclusion"), Some(&JsonValue::Null));
    assert_eq!(
        root.get("status").and_then(JsonValue::as_str),
        Some("draft_unfrozen")
    );
    assert_eq!(
        root.get("epoch_status").and_then(JsonValue::as_str),
        Some("unfrozen")
    );
    assert_eq!(
        root.get("d003_disposition").and_then(JsonValue::as_str),
        Some("accepted_exact_revision_oep_closure")
    );
    assert_eq!(
        root.get("owner_protocol_review")
            .and_then(JsonValue::as_str),
        Some("none")
    );
    assert_eq!(
        root.get("schema_version").and_then(JsonValue::as_str),
        Some("d004-pre-epoch-packet-v0.3")
    );
    assert_eq!(
        root.get("cross_cutting_fixture_proposal_manifest_sha256")
            .and_then(JsonValue::as_str),
        Some(CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256)
    );
    assert_eq!(
        root.get("cross_cutting_executable_fixture_catalog_sha256")
            .and_then(JsonValue::as_str),
        Some(CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256)
    );
    assert_eq!(
        root.get("fixture_inventory_status")
            .and_then(JsonValue::as_str),
        Some("cross_cutting_materialized_unreviewed_freeze_blocker")
    );
}

#[test]
fn draft_packet_rejects_unknown_missing_or_weakened_fields() {
    let canonical = String::from_utf8(canonical_draft_packet_bytes()).expect("UTF-8");

    let unknown = canonical.replacen('{', "{\"unknown\":true,", 1);
    let error = parse_draft_packet(unknown.as_bytes()).expect_err("unknown root field");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/unknown");

    let missing = canonical.replace("\"conclusion\":null,", "");
    let error = parse_draft_packet(missing.as_bytes()).expect_err("missing conclusion");
    assert_eq!(error.kind, PacketErrorKind::MissingField);
    assert_eq!(error.path, "$/conclusion");

    let duplicate = canonical.replacen('{', "{\"status\":\"draft_unfrozen\",", 1);
    assert_eq!(
        parse_draft_packet(duplicate.as_bytes())
            .expect_err("duplicate field")
            .kind,
        PacketErrorKind::Json(JsonErrorKind::DuplicateKey)
    );
    let floating = canonical.replace(
        "\"candidate_owner_hours\":24",
        "\"candidate_owner_hours\":24.0",
    );
    assert_eq!(
        parse_draft_packet(floating.as_bytes())
            .expect_err("floating budget")
            .kind,
        PacketErrorKind::Json(JsonErrorKind::FloatingPoint)
    );

    let mutations = [
        canonical.replace("\"status\":\"draft_unfrozen\"", "\"status\":\"frozen\""),
        canonical.replace("\"epoch\":null", "\"epoch\":\"0001\""),
        canonical.replace(
            "\"epoch_status\":\"unfrozen\"",
            "\"epoch_status\":\"frozen\"",
        ),
        canonical.replace(
            "\"d003_disposition\":\"accepted_exact_revision_oep_closure\"",
            "\"d003_disposition\":\"pending\"",
        ),
        canonical.replace(
            "\"owner_protocol_review\":\"none\"",
            "\"owner_protocol_review\":\"complete\"",
        ),
        canonical.replace(
            "\"completed_candidate_cases\":0",
            "\"completed_candidate_cases\":1",
        ),
        canonical.replace("\"complete_candidates\":0", "\"complete_candidates\":1"),
        canonical.replace(
            "\"complete_cross_candidate_cases\":0",
            "\"complete_cross_candidate_cases\":1",
        ),
        canonical.replace(
            "\"evidence_status\":\"none\"",
            "\"evidence_status\":\"partial\"",
        ),
        canonical.replace("\"selection\":null", "\"selection\":\"ST-REL\""),
        canonical.replace("\"conclusion\":null", "\"conclusion\":\"recommend_st_rel\""),
        canonical.replace(",\"replay repetition count unresolved\"", ""),
        canonical.replace("\"five positive subjects absent\",", ""),
        canonical.replace(",\"26 named-mutation subjects absent\"", ""),
        canonical.replace(
            "\"fixture_inventory_status\":\"cross_cutting_materialized_unreviewed_freeze_blocker\"",
            "\"fixture_inventory_status\":\"complete\"",
        ),
        canonical.replace(",\"resource-exhaustion\"", ""),
        canonical.replace(",\"SR-14\"", ""),
        canonical.replace(",\"SS-G10\"", ""),
        canonical.replace(",\"Proof\"", ""),
        canonical.replace(",\"exhausted\"", ""),
        canonical.replace(",\"fail\"", ""),
        canonical.replace(",\"SC-05-M06\"", ""),
        canonical.replace(MUTATION_MANIFEST_SHA256, &"0".repeat(64)),
        canonical.replace(
            CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256,
            &"0".repeat(64),
        ),
        canonical.replace(
            CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256,
            &"0".repeat(64),
        ),
        canonical.replace("\"case_wall_seconds\":900", "\"case_wall_seconds\":901"),
    ];
    for mutation in mutations {
        assert_eq!(
            parse_draft_packet(mutation.as_bytes())
                .expect_err("weakened packet")
                .kind,
            PacketErrorKind::InvalidValue
        );
    }
}

#[test]
fn replay_refuses_every_raw_input_drift_before_scheduling() {
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in packet");
    let inputs = checked_in_replay_inputs();
    assert_eq!(INPUT_BINDINGS.len(), 21);
    for binding in INPUT_BINDINGS {
        let bytes = inputs.get(binding.id);
        assert_eq!(sha256::hex(&sha256::digest(bytes)), binding.sha256);
        assert_eq!(packet.input_binding(binding.id), binding);
    }

    let mut one_byte_drift = CROSS_CUTTING_FIXTURE_PROPOSALS.to_vec();
    one_byte_drift[0] = b'[';
    let proposal_binding = INPUT_BINDINGS[InputBindingId::CrossCuttingFixtureProposals.index()];
    let error = runner::prepare_replay(
        &packet,
        &inputs.with_replacement(
            InputBindingId::CrossCuttingFixtureProposals,
            &one_byte_drift,
        ),
    )
    .expect_err("one-byte proposal manifest drift");
    assert_eq!(
        error,
        ReplayError::InputDigest {
            input: InputBindingId::CrossCuttingFixtureProposals,
            path: proposal_binding.path,
            expected_sha256: proposal_binding.sha256,
            observed_sha256: sha256::hex(&sha256::digest(&one_byte_drift)),
        }
    );
    runner::prepare_replay(&packet, &inputs).expect("all exact bindings");

    for binding in INPUT_BINDINGS {
        let corrupted = inputs.with_replacement(binding.id, b"corrupted");
        let error = runner::prepare_replay(&packet, &corrupted).expect_err("byte drift");
        assert_eq!(
            error,
            ReplayError::InputDigest {
                input: binding.id,
                path: binding.path,
                expected_sha256: binding.sha256,
                observed_sha256: sha256::hex(&sha256::digest(b"corrupted")),
            }
        );
    }

    let noncanonical_manifest = {
        let mut bytes = Vec::with_capacity(NAMED_MUTATIONS.len() + 1);
        bytes.push(b' ');
        bytes.extend_from_slice(NAMED_MUTATIONS);
        bytes
    };
    let error = runner::prepare_replay(
        &packet,
        &inputs.with_replacement(
            InputBindingId::NamedMutationsManifest,
            &noncanonical_manifest,
        ),
    )
    .expect_err("noncanonical manifest bytes");
    assert!(matches!(
        error,
        ReplayError::InputDigest {
            input: InputBindingId::NamedMutationsManifest,
            ..
        }
    ));
}

#[test]
fn replay_schedule_is_balanced_latin_deterministic_and_still_zero_of_25() {
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in packet");
    let inputs = checked_in_replay_inputs();
    let first = runner::prepare_replay(&packet, &inputs).expect("bound replay plan");
    let second = runner::prepare_replay(&packet, &inputs).expect("bound replay plan");

    assert_eq!(first, second);
    assert_eq!(first.packet_sha256(), packet.digest_hex());
    assert_eq!(first.schedule().len(), REQUIRED_CANDIDATE_CASES);
    let pair_counts = runner::schedule_pair_counts(&first);
    assert_eq!(pair_counts.len(), REQUIRED_CANDIDATE_CASES);
    assert!(pair_counts.values().all(|count| *count == 1));

    let mut candidate_position_counts = BTreeMap::new();
    let mut case_position_counts = BTreeMap::new();
    for (round_index, executions) in first.schedule().chunks_exact(CANDIDATES.len()).enumerate() {
        assert_eq!(executions.len(), 5);
        assert_eq!(
            executions
                .iter()
                .map(|execution| execution.candidate)
                .collect::<BTreeSet<_>>(),
            CANDIDATES.into_iter().collect()
        );
        assert_eq!(
            executions
                .iter()
                .map(|execution| execution.case)
                .collect::<BTreeSet<_>>(),
            CASES.into_iter().collect()
        );
        for (position_index, execution) in executions.iter().enumerate() {
            assert_eq!(execution.round, round_index + 1);
            assert_eq!(execution.position, position_index + 1);
            *candidate_position_counts
                .entry((execution.candidate, execution.position))
                .or_insert(0_usize) += 1;
            *case_position_counts
                .entry((execution.case, execution.position))
                .or_insert(0_usize) += 1;
        }
    }
    assert!(candidate_position_counts.values().all(|count| *count == 1));
    assert!(case_position_counts.values().all(|count| *count == 1));
    assert_eq!(candidate_position_counts.len(), 25);
    assert_eq!(case_position_counts.len(), 25);
    for (index, execution) in first.schedule().iter().enumerate() {
        assert_eq!(execution.ordinal, index + 1);
    }

    assert_eq!(first.completed_candidate_cases(), 0);
    assert_eq!(first.complete_candidates(), 0);
    assert_eq!(first.complete_cross_candidate_cases(), 0);
    assert_eq!(first.evidence_status(), "none");
    assert_eq!(first.selection(), None);
    assert_eq!(first.conclusion(), None);
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.digest_hex(), second.digest_hex());
    let replay_value = strict_json::parse(&first.canonical_bytes()).expect("replay manifest");
    assert_eq!(
        strict_json::canonical_bytes(&replay_value),
        first.canonical_bytes()
    );
    let replay_text = String::from_utf8(first.canonical_bytes()).expect("UTF-8");
    for nonclaim in NONCLAIMS {
        assert!(replay_text.contains(nonclaim));
    }
}

#[test]
fn d004_research_tree_is_exactly_five_input_only_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../research/decisions/D-004");
    let observed = fs::read_dir(&root)
        .expect("D-004 research directory")
        .map(|entry| {
            let entry = entry.expect("research entry");
            assert!(entry.file_type().expect("entry type").is_file());
            entry
                .file_name()
                .into_string()
                .expect("UTF-8 research filename")
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        observed,
        BTreeSet::from([
            "README.md".to_owned(),
            "d004-v0.2-cross-cutting-fixture-proposals.json".to_owned(),
            "d004-v0.2-named-mutations.json".to_owned(),
            "d004-v0.3-cross-cutting-executable-fixtures.json".to_owned(),
            "d004-v0.3-draft-packet.json".to_owned(),
        ])
    );
    for forbidden in [
        "results",
        "review",
        "reviews",
        "decision",
        "epoch",
        "candidates",
    ] {
        assert!(!observed.iter().any(|name| name.contains(forbidden)));
    }
}
