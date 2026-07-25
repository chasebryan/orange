//! Draft-only substrate checks for the D-005 public-assurance-model decision suite.
//!
//! These checks plan 32 symmetric candidate-case slots but execute none of them.
//! They select no candidate and create no D-005 execution or product evidence.

#[path = "d005_support/cases.rs"]
mod cases;
#[path = "d005_support/domain.rs"]
mod domain;
#[path = "d005_support/packet.rs"]
mod packet;
#[path = "d005_support/runner.rs"]
mod runner;
#[path = "d005_support/sha256.rs"]
mod sha256;
#[path = "d005_support/strict_json.rs"]
mod strict_json;

use std::collections::{BTreeMap, BTreeSet};

use cases::MUTATIONS;
use domain::{
    BUDGETS, CANDIDATES, CASES, CLAIM_FAMILIES, HARD_GATES, INPUT_BINDINGS, InputBindingId,
    LEGACY_V01_MUTATIONS, METRICS, NONCLAIMS, OWNER_SCOPES, REQUIRED_CANDIDATE_CASES,
};
use packet::{
    MUTATION_MANIFEST_SHA256, PacketErrorKind, canonical_draft_packet_bytes,
    canonical_mutation_manifest_bytes, mutation_manifest_digest_hex, parse_draft_packet,
};
use runner::ReplayInputs;
use strict_json::{JsonErrorKind, JsonValue};

const CHECKED_IN_DRAFT_PACKET: &[u8] = include_bytes!(
    "../../../../research/decisions/D-005/d005-v0.1/epochs/0001/protocol/epoch.json"
);
const DECISION_SUITE_INPUT: &[u8] =
    include_bytes!("../../../../docs/PUBLIC_ASSURANCE_MODEL_DECISION_SUITE.md");
const LEGACY_V01_MANIFEST_INPUT: &[u8] = include_bytes!(
    "../../../../research/decisions/D-005/d005-v0.1/epochs/0001/shared-inputs/legacy-v0.1-mutations.json"
);
const CLAIM_RECORD_V01_SCHEMA_INPUT: &[u8] =
    include_bytes!("../../../../schemas/gate0/claim-record-v0.1.schema.json");

fn checked_in_replay_inputs() -> ReplayInputs<'static> {
    ReplayInputs {
        decision_suite: DECISION_SUITE_INPUT,
        legacy_v01_manifest: LEGACY_V01_MANIFEST_INPUT,
        claim_record_v01_schema: CLAIM_RECORD_V01_SCHEMA_INPUT,
    }
}

#[test]
fn d005_draft_contract_has_exact_cardinalities_and_mutation_inventory() {
    assert_eq!(CANDIDATES.len(), 4);
    assert_eq!(CASES.len(), 8);
    assert_eq!(CANDIDATES.len() * CASES.len(), REQUIRED_CANDIDATE_CASES);
    assert_eq!(CLAIM_FAMILIES.len(), 10);
    assert_eq!(METRICS.len(), 18);
    assert_eq!(HARD_GATES.len(), 8);
    assert_eq!(OWNER_SCOPES.len(), 8);
    assert_eq!(LEGACY_V01_MUTATIONS.len(), 5);
    assert_eq!(NONCLAIMS.len(), 4);
    assert_eq!(MUTATIONS.len(), 50);

    let mutation_ids = MUTATIONS
        .iter()
        .map(|mutation| mutation.id)
        .collect::<BTreeSet<_>>();
    assert_eq!(mutation_ids.len(), MUTATIONS.len());
    assert!(
        MUTATIONS
            .iter()
            .all(|mutation| !mutation.description.is_empty())
    );

    let mut per_case = BTreeMap::new();
    for mutation in MUTATIONS {
        assert!(mutation.id.starts_with(mutation.case.as_str()));
        *per_case.entry(mutation.case).or_insert(0_usize) += 1;
    }
    assert_eq!(
        CASES.map(|case| per_case.get(&case).copied().unwrap_or(0)),
        [6, 6, 6, 6, 7, 5, 6, 8]
    );
}

#[test]
fn d005_draft_contract_has_exact_non_compensable_budgets() {
    assert_eq!(BUDGETS.max_packet_bytes, 262_144);
    assert_eq!(BUDGETS.max_json_depth, 32);
    assert_eq!(BUDGETS.max_json_nodes, 16_384);
    assert_eq!(BUDGETS.max_string_bytes, 16_384);
    assert_eq!(BUDGETS.max_diagnostics, 256);
    assert_eq!(BUDGETS.max_claims, 4_096);
    assert_eq!(BUDGETS.max_edges, 16_384);
    assert_eq!(BUDGETS.max_output_bytes, 4_194_304);
    assert_eq!(BUDGETS.render_repetitions, 3);
    assert_eq!(BUDGETS.workspace_replays, 2);
}

#[test]
fn strict_json_rejects_duplicate_keys_and_floating_point() {
    let duplicate = strict_json::parse(br#"{"a":1,"a":2}"#).expect_err("duplicate key");
    assert_eq!(duplicate.kind, JsonErrorKind::DuplicateKey);
    assert!(duplicate.offset > 0);

    for source in [br#"{"a":1.0}"#.as_slice(), br#"{"a":1e0}"#.as_slice()] {
        let error = strict_json::parse(source).expect_err("floating point");
        assert_eq!(error.kind, JsonErrorKind::FloatingPoint);
    }
}

#[test]
fn strict_json_rejects_malformed_numbers_unicode_and_utf8() {
    let leading_zero = strict_json::parse(b"[01]").expect_err("leading zero");
    assert_eq!(leading_zero.kind, JsonErrorKind::InvalidNumber);

    let too_large = strict_json::parse(b"9007199254740992").expect_err("I-JSON range");
    assert_eq!(too_large.kind, JsonErrorKind::IntegerRange);

    for source in [br#""\ud800""#.as_slice(), br#""\udc00""#.as_slice()] {
        let error = strict_json::parse(source).expect_err("unpaired surrogate");
        assert_eq!(error.kind, JsonErrorKind::InvalidUnicode);
    }

    let invalid_utf8 = strict_json::parse(&[b'"', 0xff, b'"']).expect_err("UTF-8");
    assert_eq!(invalid_utf8.kind, JsonErrorKind::InvalidUtf8);
}

#[test]
fn strict_json_fails_closed_at_depth_node_string_and_input_limits() {
    let mut too_deep = vec![b'['; BUDGETS.max_json_depth + 1];
    too_deep.push(b'0');
    too_deep.extend(std::iter::repeat_n(b']', BUDGETS.max_json_depth + 1));
    assert_eq!(
        strict_json::parse(&too_deep).expect_err("depth").kind,
        JsonErrorKind::DepthLimit
    );

    let mut too_many_nodes = String::from("[");
    for index in 0..BUDGETS.max_json_nodes {
        if index != 0 {
            too_many_nodes.push(',');
        }
        too_many_nodes.push('0');
    }
    too_many_nodes.push(']');
    assert_eq!(
        strict_json::parse(too_many_nodes.as_bytes())
            .expect_err("node limit")
            .kind,
        JsonErrorKind::NodeLimit
    );

    let too_long_string = format!("\"{}\"", "x".repeat(BUDGETS.max_string_bytes + 1));
    assert_eq!(
        strict_json::parse(too_long_string.as_bytes())
            .expect_err("string limit")
            .kind,
        JsonErrorKind::StringLimit
    );

    let oversized = vec![b' '; BUDGETS.max_packet_bytes + 1];
    assert_eq!(
        strict_json::parse(&oversized)
            .expect_err("input limit")
            .kind,
        JsonErrorKind::InputTooLarge
    );
}

#[test]
fn strict_json_accepts_each_exact_frozen_limit() {
    let mut exact_depth = vec![b'['; BUDGETS.max_json_depth];
    exact_depth.push(b'0');
    exact_depth.extend(std::iter::repeat_n(b']', BUDGETS.max_json_depth));
    strict_json::parse(&exact_depth).expect("exact depth limit");

    let mut exact_nodes = String::from("[");
    for index in 0..(BUDGETS.max_json_nodes - 1) {
        if index != 0 {
            exact_nodes.push(',');
        }
        exact_nodes.push('0');
    }
    exact_nodes.push(']');
    strict_json::parse(exact_nodes.as_bytes()).expect("exact node limit");

    let exact_string = format!("\"{}\"", "x".repeat(BUDGETS.max_string_bytes));
    strict_json::parse(exact_string.as_bytes()).expect("exact string limit");

    let mut exact_input = Vec::with_capacity(BUDGETS.max_packet_bytes);
    exact_input.push(b'0');
    exact_input.resize(BUDGETS.max_packet_bytes, b' ');
    strict_json::parse(&exact_input).expect("exact input limit");
}

#[test]
fn strict_json_canonicalizes_transport_without_changing_meaning() {
    let value = strict_json::parse("{\"z\":-0,\"é\":\"é\",\"𐀀\":1,\"\":2}".as_bytes())
        .expect("strict JSON");
    assert_eq!(
        String::from_utf8(strict_json::canonical_bytes(&value)).expect("canonical UTF-8"),
        "{\"z\":0,\"é\":\"é\",\"𐀀\":1,\"\":2}"
    );
}

#[test]
fn sha256_matches_known_answers_and_padding_boundaries() {
    let cases = [
        (
            b"".as_slice(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc".as_slice(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(sha256::hex(&sha256::digest(input)), expected);
    }

    let boundaries = [
        (
            55,
            "9f4390f8d30c2dd92ec9f095b65e2b9ae9b0a925a5258e241c9f1e910f734318",
        ),
        (
            56,
            "b35439a4ac6f0948b6d6f9e3c6af0f5f590ce20f1bde7090ef7970686ec6738a",
        ),
        (
            63,
            "7d3e74a05d7db15bce4ad9ec0658ea98e3f06eeecf16b4c6fff2da457ddc2f34",
        ),
        (
            64,
            "ffe054fe7ae0cb6dc65c3af9b61d5209f439851db43d0ba5997337df154668eb",
        ),
        (
            65,
            "635361c48bb9eab14198e76ea8ab7f1a41685d6ad62aa9146d301d4f17eb0ae0",
        ),
    ];
    for (length, expected) in boundaries {
        assert_eq!(sha256::hex(&sha256::digest(&vec![b'a'; length])), expected);
    }

    assert_eq!(
        sha256::hex(&sha256::digest(&vec![b'a'; 1_000_000])),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn mutation_manifest_binds_every_id_case_and_description() {
    let manifest = canonical_mutation_manifest_bytes();
    let parsed = strict_json::parse(&manifest).expect("canonical mutation manifest");
    assert_eq!(parsed.as_array().expect("manifest array").len(), 50);
    assert_eq!(mutation_manifest_digest_hex(), MUTATION_MANIFEST_SHA256);
    assert_eq!(
        sha256::hex(&sha256::digest(&manifest)),
        "8d069daf4a9443cf9df2d127f86d834e1aefed149324503f980c43f29c356082"
    );
}

#[test]
fn draft_packet_round_trips_to_one_canonical_digest() {
    let canonical = canonical_draft_packet_bytes();
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    assert_eq!(packet.canonical_bytes(), canonical);
    assert_eq!(packet.digest(), &sha256::digest(&canonical));
    assert_eq!(packet.digest_hex().len(), 64);
    assert_eq!(
        packet.digest_hex(),
        "2a56537bfa61fe1e4f015047b7c49b11fa926bd4cb688c6e7d4a0da07e21b633"
    );

    let reordered = format!(" \n{} \r\n", String::from_utf8(canonical).expect("UTF-8"));
    let replayed = parse_draft_packet(reordered.as_bytes()).expect("transport whitespace");
    assert_eq!(replayed.canonical_bytes(), packet.canonical_bytes());
    assert_eq!(replayed.digest(), packet.digest());
}

#[test]
fn draft_packet_rejects_unknown_missing_or_weakened_fields() {
    let canonical = String::from_utf8(canonical_draft_packet_bytes()).expect("UTF-8");
    let unknown = canonical.replacen('{', "{\"unknown\":true,", 1);
    let error = parse_draft_packet(unknown.as_bytes()).expect_err("unknown field");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/unknown");

    let missing = canonical.replace(",\"selection\":null", "");
    let error = parse_draft_packet(missing.as_bytes()).expect_err("missing selection");
    assert_eq!(error.kind, PacketErrorKind::MissingField);
    assert_eq!(error.path, "$/selection");

    let weakened = canonical.replace("\"max_claims\":4096", "\"max_claims\":4097");
    let error = parse_draft_packet(weakened.as_bytes()).expect_err("weakened budget");
    assert_eq!(error.kind, PacketErrorKind::InvalidValue);
    assert_eq!(error.path, "$/budgets/max_claims");

    let nested_unknown =
        canonical.replace("\"max_claims\":4096", "\"extra\":0,\"max_claims\":4096");
    let error = parse_draft_packet(nested_unknown.as_bytes()).expect_err("nested unknown");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/budgets/extra");

    let mutation_manifest = canonical.replace(
        MUTATION_MANIFEST_SHA256,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    let error = parse_draft_packet(mutation_manifest.as_bytes()).expect_err("mutation semantics");
    assert_eq!(error.kind, PacketErrorKind::InvalidValue);
    assert_eq!(error.path, "$/mutation_manifest_sha256");

    let binding_digest = canonical.replace(INPUT_BINDINGS[0].sha256, &"0".repeat(64));
    let error = parse_draft_packet(binding_digest.as_bytes()).expect_err("binding digest");
    assert_eq!(error.kind, PacketErrorKind::InvalidValue);
    assert_eq!(error.path, "$/input_bindings/decision_suite/sha256");

    let binding_unknown = canonical.replace(
        "\"decision_suite\":{",
        "\"decision_suite\":{\"authority\":\"owner\",",
    );
    let error = parse_draft_packet(binding_unknown.as_bytes()).expect_err("binding unknown");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/input_bindings/decision_suite/authority");
}

#[test]
fn draft_packet_cannot_claim_execution_evidence_or_selection() {
    let canonical = String::from_utf8(canonical_draft_packet_bytes()).expect("UTF-8");
    let false_evidence = canonical.replace(
        "\"evidence_status\":\"none\"",
        "\"evidence_status\":\"complete\"",
    );
    let error = parse_draft_packet(false_evidence.as_bytes()).expect_err("false evidence status");
    assert_eq!(error.kind, PacketErrorKind::InvalidValue);
    assert_eq!(error.path, "$/execution/evidence_status");

    for mutation in [
        canonical.replace(
            "\"completed_candidate_cases\":0",
            "\"completed_candidate_cases\":1",
        ),
        canonical.replace("\"selection\":null", "\"selection\":\"AM-01\""),
        canonical.replace(",\"AC-08-M08\"", ""),
    ] {
        let error = parse_draft_packet(mutation.as_bytes()).expect_err("false completion");
        assert_eq!(error.kind, PacketErrorKind::InvalidValue);
    }

    let packet = parse_draft_packet(canonical.as_bytes()).expect("draft packet");
    let root = packet.value().as_object().expect("root object");
    assert_eq!(root.get("selection"), Some(&JsonValue::Null));
}

#[test]
fn replay_refuses_any_input_whose_raw_bytes_do_not_match_the_packet() {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in packet");
    let inputs = checked_in_replay_inputs();
    let bound_bytes = [
        inputs.decision_suite,
        inputs.legacy_v01_manifest,
        inputs.claim_record_v01_schema,
    ];
    for (binding, bytes) in INPUT_BINDINGS.into_iter().zip(bound_bytes) {
        assert_eq!(sha256::hex(&sha256::digest(bytes)), binding.sha256);
        assert_eq!(packet.input_binding(binding.id), binding);
    }
    runner::prepare_replay(&packet, &inputs).expect("all exact input bindings");

    let corrupted = b"corrupted";
    let corruptions = [
        (
            InputBindingId::DecisionSuite,
            ReplayInputs {
                decision_suite: corrupted,
                ..inputs
            },
        ),
        (
            InputBindingId::LegacyV01Manifest,
            ReplayInputs {
                legacy_v01_manifest: corrupted,
                ..inputs
            },
        ),
        (
            InputBindingId::ClaimRecordV01Schema,
            ReplayInputs {
                claim_record_v01_schema: corrupted,
                ..inputs
            },
        ),
    ];
    for (expected_input, corrupted_inputs) in corruptions {
        let error =
            runner::prepare_replay(&packet, &corrupted_inputs).expect_err("corrupted bound input");
        let binding = packet.input_binding(expected_input);
        assert_eq!(error.input, expected_input);
        assert_eq!(error.path, binding.path);
        assert_eq!(error.expected_sha256, binding.sha256);
        assert_ne!(error.observed_sha256, binding.sha256);
    }
}

#[test]
fn replay_plan_is_symmetric_deterministic_and_still_zero_of_32() {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let inputs = checked_in_replay_inputs();
    let first = runner::prepare_replay(&packet, &inputs).expect("bound replay plan");
    let second = runner::prepare_replay(&packet, &inputs).expect("bound replay plan");

    assert_eq!(first, second);
    assert_eq!(first.packet_sha256(), packet.digest_hex());
    assert_eq!(first.schedule().len(), REQUIRED_CANDIDATE_CASES);
    assert_eq!(
        runner::schedule_pair_counts(&first).len(),
        REQUIRED_CANDIDATE_CASES
    );
    assert!(
        runner::schedule_pair_counts(&first)
            .values()
            .all(|count| *count == 1)
    );
    assert_eq!(first.completed_candidate_cases(), 0);
    assert_eq!(first.evidence_status(), "none");
    assert_eq!(first.selection(), None);

    for (case_index, case) in CASES.into_iter().enumerate() {
        let first_for_case = &first.schedule()[case_index * CANDIDATES.len()];
        assert_eq!(first_for_case.case, case);
        assert_eq!(
            first_for_case.candidate,
            CANDIDATES[case_index % CANDIDATES.len()]
        );
    }
    for (index, execution) in first.schedule().iter().enumerate() {
        assert_eq!(execution.ordinal, index + 1);
    }

    let mut positional_counts = BTreeMap::new();
    for case_executions in first.schedule().chunks_exact(CANDIDATES.len()) {
        let candidates = case_executions
            .iter()
            .map(|execution| execution.candidate)
            .collect::<BTreeSet<_>>();
        assert_eq!(candidates.len(), CANDIDATES.len());
        for (position, execution) in case_executions.iter().enumerate() {
            *positional_counts
                .entry((execution.candidate, position))
                .or_insert(0_usize) += 1;
        }
    }
    for candidate in CANDIDATES {
        for position in 0..CANDIDATES.len() {
            assert_eq!(positional_counts.get(&(candidate, position)), Some(&2));
        }
    }

    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.digest_hex(), second.digest_hex());
    let manifest = strict_json::parse(&first.canonical_bytes()).expect("replay manifest");
    assert_eq!(
        strict_json::canonical_bytes(&manifest),
        first.canonical_bytes()
    );
    let text = String::from_utf8(first.canonical_bytes()).expect("manifest UTF-8");
    assert!(text.contains("\"completed_candidate_cases\":0"));
    assert!(text.contains("\"evidence_status\":\"none\""));
    assert!(text.contains("\"selection\":null"));
    assert!(text.contains("no D-005 execution evidence"));
}
