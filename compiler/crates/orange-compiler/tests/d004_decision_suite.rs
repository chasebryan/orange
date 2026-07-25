//! Draft-unfrozen substrate checks for the D-004 semantic-strata decision suite.
//!
//! These checks bind the pre-epoch packet and plan 25 symmetric candidate-case
//! slots. They execute no candidate adapter, freeze no epoch, select no
//! semantic architecture, and create no D-004 result or product evidence.

#[path = "d004_support/cases.rs"]
mod cases;
#[path = "d004_support/domain.rs"]
mod domain;
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
    BUDGETS, CANDIDATES, CASE_VERDICTS, CASES, DOMAIN_OBSERVATION_STATES, HARD_GATES,
    INPUT_BINDINGS, InputBindingId, NONCLAIMS, PROTOCOL_GAPS, RELATIONSHIPS,
    REQUIRED_CANDIDATE_CASES, SOURCE_ROLES, UNRESOLVED_CROSS_CUTTING_FIXTURE_CLASSES,
};
use packet::{
    MUTATION_MANIFEST_SHA256, PacketErrorKind, canonical_draft_packet_bytes,
    canonical_draft_packet_file_bytes, canonical_mutation_manifest_bytes,
    canonical_mutation_manifest_file_bytes, mutation_manifest_digest_hex, parse_draft_packet,
    parse_mutation_manifest,
};
use runner::{ReplayError, ReplayInputs};
use strict_json::{JsonErrorKind, JsonValue};

const CHECKED_IN_PACKET: &[u8] =
    include_bytes!("../../../../research/decisions/D-004/d004-v0.2-draft-packet.json");
const NAMED_MUTATIONS: &[u8] =
    include_bytes!("../../../../research/decisions/D-004/d004-v0.2-named-mutations.json");
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
    ])
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
            "ambiguity fixture coverage unresolved",
            "missing-edge fixture coverage unresolved",
            "identity-substitution fixture coverage unresolved",
            "unsupported fixture coverage unresolved",
            "resource-exhaustion fixture coverage unresolved",
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
fn draft_packet_has_exact_canonical_bytes_digest_and_zero_baseline() {
    assert_eq!(CHECKED_IN_PACKET, canonical_draft_packet_file_bytes());
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in packet");
    assert_eq!(packet.canonical_bytes(), canonical_draft_packet_bytes());
    assert_eq!(packet.digest(), &sha256::digest(packet.canonical_bytes()));
    assert_eq!(
        packet.digest_hex(),
        "6ab790957af9ff6b7dc1dc637800542280a229647367c5312d9b5ac4fd38fb87"
    );
    assert_eq!(
        sha256::hex(&sha256::digest(CHECKED_IN_PACKET)),
        "7a27e97692d60e24a680452179ef29a3095536a7cff03edc77688a85db2fad3d"
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
        Some("pending")
    );
    assert_eq!(
        root.get("owner_protocol_review")
            .and_then(JsonValue::as_str),
        Some("none")
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
            "\"d003_disposition\":\"pending\"",
            "\"d003_disposition\":\"accepted\"",
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
        canonical.replace(",\"resource-exhaustion\"", ""),
        canonical.replace(",\"SR-14\"", ""),
        canonical.replace(",\"SS-G10\"", ""),
        canonical.replace(",\"Proof\"", ""),
        canonical.replace(",\"exhausted\"", ""),
        canonical.replace(",\"fail\"", ""),
        canonical.replace(",\"SC-05-M06\"", ""),
        canonical.replace(MUTATION_MANIFEST_SHA256, &"0".repeat(64)),
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
    assert_eq!(INPUT_BINDINGS.len(), 19);
    for binding in INPUT_BINDINGS {
        let bytes = inputs.get(binding.id);
        assert_eq!(sha256::hex(&sha256::digest(bytes)), binding.sha256);
        assert_eq!(packet.input_binding(binding.id), binding);
    }
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
fn d004_research_tree_is_exactly_three_input_only_files() {
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
            "d004-v0.2-draft-packet.json".to_owned(),
            "d004-v0.2-named-mutations.json".to_owned(),
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
