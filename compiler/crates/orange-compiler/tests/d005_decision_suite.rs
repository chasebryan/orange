//! Draft-only substrate checks for the D-005 public-assurance-model decision suite.
//!
//! These checks plan 32 symmetric candidate-case slots but execute none of them.
//! They select no candidate and create no D-005 execution or product evidence.

#[path = "d005_support/adapter.rs"]
mod adapter;
#[path = "d005_support/capture.rs"]
mod capture;
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

use adapter::{CapturedProcess, CapturedTermination, DraftRequestErrorKind, TransportErrorKind};
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
const SYNTHETIC_INPUT_MANIFEST_SHA256: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";
const SYNTHETIC_PAYLOAD_SCHEMA_SHA256: &str =
    "2222222222222222222222222222222222222222222222222222222222222222";

fn checked_in_replay_inputs() -> ReplayInputs<'static> {
    ReplayInputs {
        decision_suite: DECISION_SUITE_INPUT,
        legacy_v01_manifest: LEGACY_V01_MANIFEST_INPUT,
        claim_record_v01_schema: CLAIM_RECORD_V01_SCHEMA_INPUT,
    }
}

fn prepared_draft_adapter_request() -> adapter::DraftAdapterRequest {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    adapter::prepare_draft_request(
        &plan,
        plan.schedule()[0],
        1,
        1,
        SYNTHETIC_INPUT_MANIFEST_SHA256,
        SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
    )
    .expect("draft transport request")
}

fn draft_base_slot_bindings(plan: &runner::ReplayPlan) -> Vec<adapter::DraftBaseSlotBinding> {
    plan.schedule()
        .iter()
        .map(|execution| adapter::DraftBaseSlotBinding {
            execution: *execution,
            input_manifest_sha256: format!("{:064x}", execution.ordinal),
            payload_schema_sha256: format!("{:064x}", 4_096 + execution.ordinal),
        })
        .collect()
}

fn equal_draft_base_slot_bindings(plan: &runner::ReplayPlan) -> Vec<adapter::DraftBaseSlotBinding> {
    plan.schedule()
        .iter()
        .map(|execution| adapter::DraftBaseSlotBinding {
            execution: *execution,
            input_manifest_sha256: SYNTHETIC_INPUT_MANIFEST_SHA256.to_owned(),
            payload_schema_sha256: SYNTHETIC_PAYLOAD_SCHEMA_SHA256.to_owned(),
        })
        .collect()
}

fn prepared_draft_transport_identities()
-> (runner::ReplayPlan, Vec<adapter::DraftTransportIdentity>) {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    let bindings = draft_base_slot_bindings(&plan);
    let identities = adapter::enumerate_draft_transport_identities(&plan, &bindings)
        .expect("exact draft transport identities");
    (plan, identities)
}

fn bounded_integrity_capture(identity: &adapter::DraftTransportIdentity) -> CapturedProcess {
    CapturedProcess {
        termination: CapturedTermination::Exited(0),
        stdout: format!(
            "synthetic-capture-slot:{}\n",
            identity.capture_slot_ordinal()
        )
        .into_bytes(),
        stderr: Vec::new(),
        stdout_truncated: false,
        stderr_truncated: false,
    }
}

fn raw_sha256(bytes: &[u8]) -> String {
    sha256::hex(&sha256::digest(bytes))
}

fn substituted_sha256(value: &str) -> String {
    let mut substituted = value.as_bytes().to_vec();
    substituted[0] = if substituted[0] == b'0' { b'1' } else { b'0' };
    String::from_utf8(substituted).expect("lowercase SHA-256")
}

fn synthetic_payload() -> JsonValue {
    strict_json::object([("synthetic".to_owned(), JsonValue::Bool(true))])
}

fn draft_response_for_payload(
    request: &adapter::DraftAdapterRequest,
    payload: &JsonValue,
) -> Result<Vec<u8>, adapter::TransportError> {
    let payload_bytes = strict_json::canonical_bytes(payload);
    adapter::canonical_draft_response_bytes(request, &payload_bytes)
}

fn chunk_payload(total_string_bytes: usize, item_count: usize) -> JsonValue {
    let mut remaining = total_string_bytes;
    let chunks = (0..item_count)
        .map(|_| {
            let length = remaining.min(BUDGETS.max_string_bytes);
            remaining -= length;
            JsonValue::String("x".repeat(length))
        })
        .collect();
    assert_eq!(remaining, 0, "chunk capacity must cover requested bytes");
    strict_json::object([("chunks".to_owned(), JsonValue::Array(chunks))])
}

fn successful_capture(stdout: Vec<u8>) -> CapturedProcess {
    CapturedProcess {
        termination: CapturedTermination::Exited(0),
        stdout,
        stderr: Vec::new(),
        stdout_truncated: false,
        stderr_truncated: false,
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
        "731428229b4f77cd7e684e2a5cae51bdfd277898aaab60852b843d3183dbc194"
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

#[test]
fn draft_adapter_requests_are_deterministic_schedule_bound_and_non_executing() {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    let execution = plan.schedule()[0];
    let first = adapter::prepare_draft_request(
        &plan,
        execution,
        1,
        1,
        SYNTHETIC_INPUT_MANIFEST_SHA256,
        SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
    )
    .expect("draft transport request");
    let second = adapter::prepare_draft_request(
        &plan,
        execution,
        1,
        1,
        SYNTHETIC_INPUT_MANIFEST_SHA256,
        SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
    )
    .expect("draft transport request");
    assert_eq!(first, second);
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.digest_hex(), second.digest_hex());

    let value = strict_json::parse(&first.canonical_bytes()).expect("canonical draft request");
    assert_eq!(
        strict_json::canonical_bytes(&value),
        first.canonical_bytes()
    );
    let text = String::from_utf8(first.canonical_bytes()).expect("UTF-8 request");
    assert!(text.contains("\"schema_version\":\"d005-adapter-request-v0.1-draft\""));
    assert!(text.contains("\"candidate\":\"AM-01\""));
    assert!(text.contains("\"case\":\"AC-01\""));
    assert!(!text.contains("verdict"));
    assert!(!text.contains("outcome"));
    assert!(!text.contains("evidence"));
    assert_eq!(plan.completed_candidate_cases(), 0);
    assert_eq!(plan.evidence_status(), "none");
    assert_eq!(plan.selection(), None);

    let wrong_execution = runner::PlannedExecution {
        ordinal: execution.ordinal,
        candidate: CANDIDATES[1],
        case: execution.case,
    };
    let error = adapter::prepare_draft_request(
        &plan,
        wrong_execution,
        1,
        1,
        SYNTHETIC_INPUT_MANIFEST_SHA256,
        SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
    )
    .expect_err("schedule mismatch");
    assert_eq!(error.kind, DraftRequestErrorKind::ScheduleMismatch);
    assert_eq!(error.path, "$/ordinal");

    for (workspace_replay, render_repetition, expected_kind, expected_path) in [
        (
            0,
            1,
            DraftRequestErrorKind::WorkspaceReplay,
            "$/workspace_replay",
        ),
        (
            BUDGETS.workspace_replays + 1,
            1,
            DraftRequestErrorKind::WorkspaceReplay,
            "$/workspace_replay",
        ),
        (
            1,
            0,
            DraftRequestErrorKind::RenderRepetition,
            "$/render_repetition",
        ),
        (
            1,
            BUDGETS.render_repetitions + 1,
            DraftRequestErrorKind::RenderRepetition,
            "$/render_repetition",
        ),
    ] {
        let error = adapter::prepare_draft_request(
            &plan,
            execution,
            workspace_replay,
            render_repetition,
            SYNTHETIC_INPUT_MANIFEST_SHA256,
            SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
        )
        .expect_err("out-of-range repetition");
        assert_eq!(error.kind, expected_kind);
        assert_eq!(error.path, expected_path);
    }

    for (input_digest, payload_digest, expected_path) in [
        (
            "short",
            SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
            "$/input_manifest_sha256",
        ),
        (
            SYNTHETIC_INPUT_MANIFEST_SHA256,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "$/payload_schema_sha256",
        ),
    ] {
        let error =
            adapter::prepare_draft_request(&plan, execution, 1, 1, input_digest, payload_digest)
                .expect_err("invalid digest");
        assert_eq!(error.kind, DraftRequestErrorKind::InvalidDigest);
        assert_eq!(error.path, expected_path);
    }
}

#[test]
fn draft_transport_matrix_is_exact_ordered_unique_deterministic_and_non_executing() {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    let bindings = draft_base_slot_bindings(&plan);
    let first = adapter::enumerate_draft_transport_identities(&plan, &bindings)
        .expect("exact draft transport matrix");
    let second = adapter::enumerate_draft_transport_identities(&plan, &bindings)
        .expect("deterministic draft transport matrix");

    assert_eq!(first, second);
    assert_eq!(first.len(), adapter::REQUIRED_DRAFT_TRANSPORT_IDENTITIES);
    assert_eq!(first.len(), 192);
    assert_eq!(bindings.len(), REQUIRED_CANDIDATE_CASES);

    let coordinates = first
        .iter()
        .map(|identity| {
            let request = identity.request();
            (
                request.execution(),
                request.workspace_replay(),
                request.render_repetition(),
            )
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(coordinates.len(), 192);
    let request_digests = first
        .iter()
        .map(|identity| identity.request().digest_hex())
        .collect::<BTreeSet<_>>();
    assert_eq!(request_digests.len(), 192);

    for (capture_index, identity) in first.iter().enumerate() {
        let base_index = capture_index / 6;
        let coordinate_index = capture_index % 6;
        let expected_workspace_replay = coordinate_index / 3 + 1;
        let expected_render_repetition = coordinate_index % 3 + 1;
        let request = identity.request();

        assert_eq!(identity.capture_slot_ordinal(), capture_index + 1);
        assert_eq!(request.execution(), plan.schedule()[base_index]);
        assert_eq!(request.workspace_replay(), expected_workspace_replay);
        assert_eq!(request.render_repetition(), expected_render_repetition);
        assert_eq!(
            request.input_manifest_sha256(),
            bindings[base_index].input_manifest_sha256
        );
        assert_eq!(
            request.payload_schema_sha256(),
            bindings[base_index].payload_schema_sha256
        );
    }
    for (base_index, identities) in first.chunks_exact(6).enumerate() {
        assert_eq!(
            identities
                .iter()
                .map(|identity| {
                    (
                        identity.request().workspace_replay(),
                        identity.request().render_repetition(),
                    )
                })
                .collect::<Vec<_>>(),
            [(1, 1), (1, 2), (1, 3), (2, 1), (2, 2), (2, 3)]
        );
        assert!(
            identities
                .iter()
                .all(|identity| identity.request().execution() == plan.schedule()[base_index])
        );
    }

    assert_eq!(plan.completed_candidate_cases(), 0);
    assert_eq!(plan.evidence_status(), "none");
    assert_eq!(plan.selection(), None);
}

#[test]
fn draft_transport_matrix_binds_digests_and_keeps_captures_non_interchangeable() {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    let bindings = draft_base_slot_bindings(&plan);
    let identities = adapter::enumerate_draft_transport_identities(&plan, &bindings)
        .expect("slot-specific digest bindings");

    let first_request = identities[0].request();
    let second_base_request = identities[6].request();
    assert_eq!(
        first_request.input_manifest_sha256(),
        bindings[0].input_manifest_sha256
    );
    assert_eq!(
        second_base_request.input_manifest_sha256(),
        bindings[1].input_manifest_sha256
    );
    assert_ne!(
        first_request.input_manifest_sha256(),
        second_base_request.input_manifest_sha256()
    );
    assert_ne!(first_request.digest_hex(), second_base_request.digest_hex());

    let response = draft_response_for_payload(first_request, &synthetic_payload())
        .expect("canonical synthetic capture");
    let unvalidated =
        adapter::validate_capture(first_request, &successful_capture(response.clone()))
            .expect("identity-bound unvalidated payload");
    assert_eq!(
        unvalidated.binding().input_manifest_sha256,
        bindings[0].input_manifest_sha256
    );
    let error = adapter::validate_capture(second_base_request, &successful_capture(response))
        .expect_err("capture from another digest-bound base slot");
    assert_eq!(error.kind, TransportErrorKind::InvalidValue);
    assert_eq!(error.path, "$/request_sha256");

    let equal_bindings = equal_draft_base_slot_bindings(&plan);
    let equal_digest_identities =
        adapter::enumerate_draft_transport_identities(&plan, &equal_bindings)
            .expect("equal digests are allowed across exact base-slot keys");
    assert_eq!(equal_digest_identities.len(), 192);
    assert!(equal_digest_identities.iter().all(|identity| {
        identity.request().input_manifest_sha256() == SYNTHETIC_INPUT_MANIFEST_SHA256
            && identity.request().payload_schema_sha256() == SYNTHETIC_PAYLOAD_SCHEMA_SHA256
    }));
    assert_eq!(
        equal_digest_identities
            .iter()
            .map(|identity| identity.request().digest_hex())
            .collect::<BTreeSet<_>>()
            .len(),
        192
    );
}

#[test]
fn draft_transport_matrix_rejects_open_reordered_or_substituted_binding_tables() {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    let canonical = draft_base_slot_bindings(&plan);

    let mut missing = canonical.clone();
    missing.pop();
    let error = adapter::enumerate_draft_transport_identities(&plan, &missing)
        .expect_err("missing binding");
    assert_eq!(
        error.kind,
        adapter::DraftTransportMatrixErrorKind::BindingCardinality
    );
    assert_eq!(error.path, "$/bindings");

    let mut extra = canonical.clone();
    extra.push(canonical[0].clone());
    let error =
        adapter::enumerate_draft_transport_identities(&plan, &extra).expect_err("extra binding");
    assert_eq!(
        error.kind,
        adapter::DraftTransportMatrixErrorKind::BindingCardinality
    );
    assert_eq!(error.path, "$/bindings");

    let mut reordered = canonical.clone();
    reordered.swap(0, 1);
    let error = adapter::enumerate_draft_transport_identities(&plan, &reordered)
        .expect_err("reordered bindings");
    assert_eq!(
        error.kind,
        adapter::DraftTransportMatrixErrorKind::BindingMismatch
    );
    assert_eq!(error.path, "$/bindings/0/execution");

    for substituted_execution in [
        runner::PlannedExecution {
            ordinal: canonical[0].execution.ordinal + 1,
            ..canonical[0].execution
        },
        runner::PlannedExecution {
            candidate: CANDIDATES[1],
            ..canonical[0].execution
        },
        runner::PlannedExecution {
            case: CASES[1],
            ..canonical[0].execution
        },
    ] {
        let mut substituted = canonical.clone();
        substituted[0].execution = substituted_execution;
        let error = adapter::enumerate_draft_transport_identities(&plan, &substituted)
            .expect_err("full planned-execution key substitution");
        assert_eq!(
            error.kind,
            adapter::DraftTransportMatrixErrorKind::BindingMismatch
        );
        assert_eq!(error.path, "$/bindings/0/execution");
    }

    for (field, invalid_digest, expected_path) in [
        (
            "input_manifest_sha256",
            "short",
            "$/bindings/7/input_manifest_sha256",
        ),
        (
            "payload_schema_sha256",
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "$/bindings/7/payload_schema_sha256",
        ),
    ] {
        let mut invalid = canonical.clone();
        match field {
            "input_manifest_sha256" => {
                invalid[7].input_manifest_sha256 = invalid_digest.to_owned();
            }
            "payload_schema_sha256" => {
                invalid[7].payload_schema_sha256 = invalid_digest.to_owned();
            }
            _ => unreachable!("closed test field inventory"),
        }
        let error = adapter::enumerate_draft_transport_identities(&plan, &invalid)
            .expect_err("invalid binding digest");
        assert_eq!(
            error.kind,
            adapter::DraftTransportMatrixErrorKind::InvalidDigest
        );
        assert_eq!(error.path, expected_path);
    }

    assert_eq!(plan.completed_candidate_cases(), 0);
    assert_eq!(plan.evidence_status(), "none");
    assert_eq!(plan.selection(), None);
}

#[test]
fn synthetic_capture_integrity_receipt_is_canonical_exact_and_non_evidentiary() {
    let (plan, identities) = prepared_draft_transport_identities();
    let identity = &identities[0];
    let capture = bounded_integrity_capture(identity);
    let first = capture::create_synthetic_capture_integrity_receipt(identity, &capture)
        .expect("bounded synthetic capture receipt");
    let second = capture::create_synthetic_capture_integrity_receipt(identity, &capture)
        .expect("deterministic synthetic capture receipt");

    assert_eq!(first, second);
    assert_eq!(first.capture_slot_ordinal(), 1);
    assert_eq!(first.request_sha256(), identity.request().digest_hex());
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.digest_hex(), second.digest_hex());
    let value = strict_json::parse(&first.canonical_bytes()).expect("canonical receipt");
    assert_eq!(
        strict_json::canonical_bytes(&value),
        first.canonical_bytes()
    );
    let root = value.as_object().expect("closed receipt object");
    assert_eq!(root.len(), 14);
    assert_eq!(
        root.get("schema_version").and_then(JsonValue::as_str),
        Some("d005-capture-integrity-receipt-v0.1-draft")
    );
    assert_eq!(
        root.get("isolation_status").and_then(JsonValue::as_str),
        Some("not_evaluated")
    );
    assert_eq!(
        root.get("payload_status").and_then(JsonValue::as_str),
        Some("unvalidated")
    );
    assert_eq!(
        root.get("evidence_status").and_then(JsonValue::as_str),
        Some("none")
    );
    assert_eq!(root.get("exit_code"), Some(&JsonValue::Integer(0)));
    assert_eq!(
        root.get("stdout_bytes").and_then(JsonValue::as_integer),
        Some(i64::try_from(capture.stdout.len()).expect("bounded stdout length"))
    );
    assert_eq!(
        root.get("stdout_sha256").and_then(JsonValue::as_str),
        Some(sha256::hex(&sha256::digest(&capture.stdout)).as_str())
    );
    let parsed = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        &first.canonical_bytes(),
        &first.digest_hex(),
        identity,
        &capture,
    )
    .expect("strict receipt verification");
    assert_eq!(parsed, first);

    assert_eq!(plan.completed_candidate_cases(), 0);
    assert_eq!(plan.evidence_status(), "none");
    assert_eq!(plan.selection(), None);
}

#[test]
fn integrity_receipts_allow_bounded_failures_while_adapter_validation_still_rejects_them() {
    let (_, identities) = prepared_draft_transport_identities();
    let identity = &identities[0];
    let failures = [
        CapturedTermination::Exited(1),
        CapturedTermination::Signaled,
        CapturedTermination::TimedOut,
        CapturedTermination::StdoutLimit,
        CapturedTermination::StderrLimit,
        CapturedTermination::SpawnFailed,
        CapturedTermination::IoFailed,
        CapturedTermination::UnsupportedSandbox,
    ];
    for termination in failures {
        let capture = CapturedProcess {
            termination,
            stdout: b"bounded synthetic failure".to_vec(),
            stderr: b"bounded diagnostic".to_vec(),
            stdout_truncated: matches!(termination, CapturedTermination::StdoutLimit),
            stderr_truncated: matches!(termination, CapturedTermination::StderrLimit),
        };
        let receipt = capture::create_synthetic_capture_integrity_receipt(identity, &capture)
            .expect("failure capture still has integrity metadata");
        capture::parse_and_verify_synthetic_capture_integrity_receipt(
            &receipt.canonical_bytes(),
            &receipt.digest_hex(),
            identity,
            &capture,
        )
        .expect("failure receipt round trip");
        adapter::validate_capture(identity.request(), &capture)
            .expect_err("integrity receipt cannot upgrade a failed adapter capture");

        let value = strict_json::parse(&receipt.canonical_bytes()).expect("failure receipt");
        let root = value.as_object().expect("receipt object");
        if matches!(termination, CapturedTermination::Exited(_)) {
            assert_eq!(root.get("exit_code"), Some(&JsonValue::Integer(1)));
        } else {
            assert_eq!(root.get("exit_code"), Some(&JsonValue::Null));
        }
        assert_eq!(
            root.get("evidence_status").and_then(JsonValue::as_str),
            Some("none")
        );
    }

    let truncated_success = CapturedProcess {
        termination: CapturedTermination::Exited(0),
        stdout: b"bounded truncated bytes".to_vec(),
        stderr: Vec::new(),
        stdout_truncated: true,
        stderr_truncated: false,
    };
    capture::create_synthetic_capture_integrity_receipt(identity, &truncated_success)
        .expect("bounded truncation can be integrity-recorded");
    assert_eq!(
        adapter::validate_capture(identity.request(), &truncated_success)
            .expect_err("truncation remains an adapter failure")
            .kind,
        TransportErrorKind::StdoutTruncated
    );

    let oversized = CapturedProcess {
        termination: CapturedTermination::StdoutLimit,
        stdout: vec![0; BUDGETS.max_output_bytes + 1],
        stderr: Vec::new(),
        stdout_truncated: true,
        stderr_truncated: false,
    };
    let error = capture::create_synthetic_capture_integrity_receipt(identity, &oversized)
        .expect_err("receipt inputs remain bounded");
    assert_eq!(error.kind, capture::CaptureReceiptErrorKind::OutputTooLarge);
    assert_eq!(error.path, "$/capture");
}

#[test]
fn capture_integrity_receipt_rejects_malformed_open_noncanonical_or_substituted_data() {
    let (_, identities) = prepared_draft_transport_identities();
    let identity = &identities[0];
    let capture = bounded_integrity_capture(identity);
    let receipt = capture::create_synthetic_capture_integrity_receipt(identity, &capture)
        .expect("canonical integrity receipt");
    let canonical = receipt.canonical_bytes();
    let expected_receipt_sha256 = receipt.digest_hex();
    let text = String::from_utf8(canonical.clone()).expect("UTF-8 receipt");

    for invalid_digest in ["short".to_owned(), "A".repeat(64)] {
        let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
            &canonical,
            &invalid_digest,
            identity,
            &capture,
        )
        .expect_err("malformed expected receipt digest");
        assert_eq!(error.kind, capture::CaptureReceiptErrorKind::InvalidDigest);
        assert_eq!(error.path, "$/expected_receipt_sha256");
    }

    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        &canonical,
        &"0".repeat(64),
        identity,
        &capture,
    )
    .expect_err("wrong expected receipt digest");
    assert_eq!(
        error.kind,
        capture::CaptureReceiptErrorKind::ReceiptDigestMismatch
    );
    assert_eq!(error.path, "$/receipt_sha256");

    let oversized_receipt = vec![b' '; BUDGETS.max_packet_bytes + 1];
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        &oversized_receipt,
        &raw_sha256(&oversized_receipt),
        identity,
        &capture,
    )
    .expect_err("receipt source hashing remains input-bounded");
    assert_eq!(
        error.kind,
        capture::CaptureReceiptErrorKind::Json(JsonErrorKind::InputTooLarge)
    );
    assert_eq!(error.path, "$/receipt@0");

    let malformed = b"{";
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        malformed,
        &raw_sha256(malformed),
        identity,
        &capture,
    )
    .expect_err("malformed receipt");
    assert!(matches!(
        error.kind,
        capture::CaptureReceiptErrorKind::Json(_)
    ));

    let unknown = text.replacen('}', ",\"unknown\":true}", 1);
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        unknown.as_bytes(),
        &raw_sha256(unknown.as_bytes()),
        identity,
        &capture,
    )
    .expect_err("unknown receipt field");
    assert_eq!(error.kind, capture::CaptureReceiptErrorKind::UnknownField);
    assert_eq!(error.path, "$/unknown");

    let missing = text.replace("\"evidence_status\":\"none\",", "");
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        missing.as_bytes(),
        &raw_sha256(missing.as_bytes()),
        identity,
        &capture,
    )
    .expect_err("missing receipt field");
    assert_eq!(error.kind, capture::CaptureReceiptErrorKind::MissingField);
    assert_eq!(error.path, "$/evidence_status");

    let duplicate = text.replacen('{', "{\"capture_slot_ordinal\":1,", 1);
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        duplicate.as_bytes(),
        &raw_sha256(duplicate.as_bytes()),
        identity,
        &capture,
    )
    .expect_err("duplicate receipt key");
    assert_eq!(
        error.kind,
        capture::CaptureReceiptErrorKind::Json(JsonErrorKind::DuplicateKey)
    );

    let noncanonical = format!(" {text}");
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        noncanonical.as_bytes(),
        &raw_sha256(noncanonical.as_bytes()),
        identity,
        &capture,
    )
    .expect_err("noncanonical receipt transport");
    assert_eq!(error.kind, capture::CaptureReceiptErrorKind::NonCanonical);

    let request_digest = identity.request().digest_hex();
    let altered_request_digest = substituted_sha256(&request_digest);
    let stdout_digest = raw_sha256(&capture.stdout);
    let altered_stdout_digest = substituted_sha256(&stdout_digest);
    let stderr_digest = raw_sha256(&capture.stderr);
    let altered_stderr_digest = substituted_sha256(&stderr_digest);
    let semantic_mutations = [
        (
            "\"capture_slot_ordinal\":1".to_owned(),
            "\"capture_slot_ordinal\":2".to_owned(),
            "$/capture_slot_ordinal",
        ),
        (request_digest, altered_request_digest, "$/request_sha256"),
        (
            "\"termination_kind\":\"exited\"".to_owned(),
            "\"termination_kind\":\"timed_out\"".to_owned(),
            "$/termination_kind",
        ),
        (
            "\"exit_code\":0".to_owned(),
            "\"exit_code\":1".to_owned(),
            "$/exit_code",
        ),
        (
            format!("\"stdout_bytes\":{}", capture.stdout.len()),
            format!("\"stdout_bytes\":{}", capture.stdout.len() + 1),
            "$/stdout_bytes",
        ),
        (stdout_digest, altered_stdout_digest, "$/stdout_sha256"),
        (
            format!("\"stderr_bytes\":{}", capture.stderr.len()),
            format!("\"stderr_bytes\":{}", capture.stderr.len() + 1),
            "$/stderr_bytes",
        ),
        (stderr_digest, altered_stderr_digest, "$/stderr_sha256"),
        (
            "\"stdout_truncated\":false".to_owned(),
            "\"stdout_truncated\":true".to_owned(),
            "$/stdout_truncated",
        ),
        (
            "\"stderr_truncated\":false".to_owned(),
            "\"stderr_truncated\":true".to_owned(),
            "$/stderr_truncated",
        ),
        (
            "\"isolation_status\":\"not_evaluated\"".to_owned(),
            "\"isolation_status\":\"passed\"".to_owned(),
            "$/isolation_status",
        ),
        (
            "\"payload_status\":\"unvalidated\"".to_owned(),
            "\"payload_status\":\"validated\"".to_owned(),
            "$/payload_status",
        ),
        (
            "\"evidence_status\":\"none\"".to_owned(),
            "\"evidence_status\":\"complete\"".to_owned(),
            "$/evidence_status",
        ),
    ];
    for (before, after, expected_path) in semantic_mutations {
        let mutated = text.replacen(&before, &after, 1);
        assert_ne!(mutated, text, "receipt mutation must alter bytes");
        let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
            mutated.as_bytes(),
            &raw_sha256(mutated.as_bytes()),
            identity,
            &capture,
        )
        .expect_err("receipt identity or status substitution");
        assert_eq!(error.kind, capture::CaptureReceiptErrorKind::InvalidValue);
        assert_eq!(error.path, expected_path);
    }

    let timeout_capture = CapturedProcess {
        termination: CapturedTermination::TimedOut,
        stdout: b"bounded timeout".to_vec(),
        stderr: Vec::new(),
        stdout_truncated: false,
        stderr_truncated: false,
    };
    let timeout_receipt =
        capture::create_synthetic_capture_integrity_receipt(identity, &timeout_capture)
            .expect("bounded timeout receipt");
    let timeout_text = String::from_utf8(timeout_receipt.canonical_bytes())
        .expect("UTF-8 timeout receipt")
        .replace("\"exit_code\":null", "\"exit_code\":0");
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        timeout_text.as_bytes(),
        &raw_sha256(timeout_text.as_bytes()),
        identity,
        &timeout_capture,
    )
    .expect_err("non-exit termination cannot carry an exit code");
    assert_eq!(error.kind, capture::CaptureReceiptErrorKind::InvalidValue);
    assert_eq!(error.path, "$/exit_code");

    let mut altered_capture = capture.clone();
    altered_capture.stdout[0] ^= 1;
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        &canonical,
        &expected_receipt_sha256,
        identity,
        &altered_capture,
    )
    .expect_err("same-length capture byte substitution");
    assert_eq!(error.kind, capture::CaptureReceiptErrorKind::InvalidValue);
    assert_eq!(error.path, "$/stdout_sha256");

    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        &canonical,
        &expected_receipt_sha256,
        &identities[1],
        &capture,
    )
    .expect_err("cross-slot request substitution");
    assert_eq!(error.kind, capture::CaptureReceiptErrorKind::InvalidValue);
    assert_eq!(error.path, "$/capture_slot_ordinal");

    let coordinated_receipt =
        capture::create_synthetic_capture_integrity_receipt(identity, &altered_capture)
            .expect("coordinated altered receipt");
    let error = capture::parse_and_verify_synthetic_capture_integrity_receipt(
        &coordinated_receipt.canonical_bytes(),
        &expected_receipt_sha256,
        identity,
        &altered_capture,
    )
    .expect_err("retained external digest rejects coordinated receipt and capture mutation");
    assert_eq!(
        error.kind,
        capture::CaptureReceiptErrorKind::ReceiptDigestMismatch
    );
    assert_eq!(error.path, "$/receipt_sha256");
}

#[test]
fn capture_observation_inventory_is_exact_deterministic_and_still_zero_of_32() {
    let (plan, identities) = prepared_draft_transport_identities();
    let observations = identities
        .iter()
        .map(|identity| {
            capture::create_synthetic_capture_integrity_receipt(
                identity,
                &bounded_integrity_capture(identity),
            )
            .expect("bounded in-memory observation")
        })
        .collect::<Vec<_>>();
    let first =
        capture::bind_draft_capture_observation_inventory(&identities, observations.clone())
            .expect("exact observation inventory");
    let second = capture::bind_draft_capture_observation_inventory(&identities, observations)
        .expect("deterministic observation inventory");

    assert_eq!(first, second);
    assert_eq!(first.observations().len(), 192);
    for (index, observation) in first.observations().iter().enumerate() {
        assert_eq!(observation.capture_slot_ordinal(), index + 1);
        assert_eq!(
            observation.request_sha256(),
            identities[index].request().digest_hex()
        );
    }
    assert_eq!(first.completed_candidate_cases(), 0);
    assert_eq!(first.evidence_status(), "none");
    assert_eq!(first.selection(), None);
    assert_eq!(plan.completed_candidate_cases(), 0);
    assert_eq!(plan.evidence_status(), "none");
    assert_eq!(plan.selection(), None);
}

#[test]
fn capture_observation_inventory_rejects_missing_extra_duplicate_reordered_and_cross_slot_data() {
    let (_, identities) = prepared_draft_transport_identities();
    let canonical = identities
        .iter()
        .map(|identity| {
            capture::create_synthetic_capture_integrity_receipt(
                identity,
                &bounded_integrity_capture(identity),
            )
            .expect("bounded in-memory observation")
        })
        .collect::<Vec<_>>();

    let mut missing = canonical.clone();
    missing.pop();
    let error = capture::bind_draft_capture_observation_inventory(&identities, missing)
        .expect_err("missing observation");
    assert_eq!(
        error.kind,
        capture::CaptureInventoryErrorKind::ObservationCardinality
    );
    assert_eq!(error.path, "$/observations");

    let mut extra = canonical.clone();
    extra.push(canonical[0].clone());
    let error = capture::bind_draft_capture_observation_inventory(&identities, extra)
        .expect_err("extra observation");
    assert_eq!(
        error.kind,
        capture::CaptureInventoryErrorKind::ObservationCardinality
    );
    assert_eq!(error.path, "$/observations");

    let mut duplicate = canonical.clone();
    duplicate[1] = duplicate[0].clone();
    let error = capture::bind_draft_capture_observation_inventory(&identities, duplicate)
        .expect_err("duplicate observation");
    assert_eq!(
        error.kind,
        capture::CaptureInventoryErrorKind::DuplicateObservation
    );
    assert_eq!(error.path, "$/observations/1");

    let mut reordered = canonical.clone();
    reordered.swap(0, 1);
    let error = capture::bind_draft_capture_observation_inventory(&identities, reordered)
        .expect_err("reordered cross-slot observations");
    assert_eq!(
        error.kind,
        capture::CaptureInventoryErrorKind::CrossSlotObservation
    );
    assert_eq!(error.path, "$/observations/0");

    let mut missing_identity = identities.clone();
    missing_identity.pop();
    let error =
        capture::bind_draft_capture_observation_inventory(&missing_identity, canonical.clone())
            .expect_err("missing expected identity");
    assert_eq!(
        error.kind,
        capture::CaptureInventoryErrorKind::IdentityCardinality
    );
    assert_eq!(error.path, "$/identities");

    let mut reordered_identities = identities.clone();
    reordered_identities.swap(0, 1);
    let error = capture::bind_draft_capture_observation_inventory(&reordered_identities, canonical)
        .expect_err("reordered expected identities");
    assert_eq!(
        error.kind,
        capture::CaptureInventoryErrorKind::IdentityOrder
    );
    assert_eq!(error.path, "$/identities/0");
}

#[test]
fn adapter_transport_accepts_only_an_identity_bound_unvalidated_payload() {
    let request = prepared_draft_adapter_request();
    let payload = synthetic_payload();
    let response =
        draft_response_for_payload(&request, &payload).expect("canonical synthetic response");
    assert_eq!(response.last(), Some(&b'\n'));

    let captured = successful_capture(response.clone());
    let unvalidated =
        adapter::validate_capture(&request, &captured).expect("valid transport capture");
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    let binding = unvalidated.binding();
    assert_eq!(binding.request_sha256, request.digest_hex());
    assert_eq!(binding.packet_sha256, packet.digest_hex());
    assert_eq!(binding.plan_sha256, plan.digest_hex());
    assert_eq!(binding.ordinal, 1);
    assert_eq!(binding.candidate, CANDIDATES[0]);
    assert_eq!(binding.case, CASES[0]);
    assert_eq!(binding.workspace_replay, 1);
    assert_eq!(binding.render_repetition, 1);
    assert_eq!(
        binding.input_manifest_sha256,
        SYNTHETIC_INPUT_MANIFEST_SHA256
    );
    assert_eq!(
        binding.payload_schema_sha256,
        SYNTHETIC_PAYLOAD_SCHEMA_SHA256
    );
    assert_eq!(unvalidated.value(), &payload);
    assert_eq!(
        unvalidated.canonical_bytes(),
        strict_json::canonical_bytes(&payload)
    );
    assert_eq!(
        unvalidated.digest_hex(),
        sha256::hex(&sha256::digest(unvalidated.canonical_bytes()))
    );

    let text = String::from_utf8(response).expect("UTF-8 response");
    assert!(text.contains("\"schema_version\":\"d005-adapter-response-v0.1-draft\""));
    assert!(text.contains(&format!("\"request_sha256\":\"{}\"", request.digest_hex())));
    assert!(!text.contains("verdict"));
    assert!(!text.contains("outcome"));
    assert!(!text.contains("evidence"));

    let non_object = adapter::canonical_draft_response_bytes(&request, b"null")
        .expect_err("payload must remain an unvalidated object");
    assert_eq!(non_object.kind, TransportErrorKind::InvalidValue);
    assert_eq!(non_object.path, "$/payload");

    let noncanonical =
        adapter::canonical_draft_response_bytes(&request, b"{ \"synthetic\": true }")
            .expect_err("payload input must be canonical");
    assert_eq!(noncanonical.kind, TransportErrorKind::NonCanonical);
    assert_eq!(noncanonical.path, "$/payload");
}

#[test]
fn identical_payload_bodies_from_different_slots_are_not_interchangeable() {
    let packet = parse_draft_packet(CHECKED_IN_DRAFT_PACKET).expect("checked-in draft packet");
    let plan =
        runner::prepare_replay(&packet, &checked_in_replay_inputs()).expect("bound replay plan");
    let first_request = adapter::prepare_draft_request(
        &plan,
        plan.schedule()[0],
        1,
        1,
        SYNTHETIC_INPUT_MANIFEST_SHA256,
        SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
    )
    .expect("first draft request");
    let second_request = adapter::prepare_draft_request(
        &plan,
        plan.schedule()[1],
        1,
        1,
        SYNTHETIC_INPUT_MANIFEST_SHA256,
        SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
    )
    .expect("second draft request");
    let payload = synthetic_payload();
    let first_response =
        draft_response_for_payload(&first_request, &payload).expect("first synthetic response");
    let second_response =
        draft_response_for_payload(&second_request, &payload).expect("second synthetic response");
    let first =
        adapter::validate_capture(&first_request, &successful_capture(first_response.clone()))
            .expect("first captured payload");
    let second = adapter::validate_capture(
        &second_request,
        &successful_capture(second_response.clone()),
    )
    .expect("second captured payload");

    assert_eq!(first.value(), second.value());
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.digest_hex(), second.digest_hex());
    assert_ne!(first.binding(), second.binding());
    assert_ne!(first, second);
    assert_ne!(
        first.binding().request_sha256,
        second.binding().request_sha256
    );
    assert_ne!(first.binding().ordinal, second.binding().ordinal);
    assert_ne!(first.binding().candidate, second.binding().candidate);
    assert_eq!(first.binding().case, second.binding().case);

    let error = adapter::validate_capture(&first_request, &successful_capture(second_response))
        .expect_err("second-slot response cannot satisfy first slot");
    assert_eq!(error.kind, TransportErrorKind::InvalidValue);
    assert_eq!(error.path, "$/request_sha256");
    let error = adapter::validate_capture(&second_request, &successful_capture(first_response))
        .expect_err("first-slot response cannot satisfy second slot");
    assert_eq!(error.kind, TransportErrorKind::InvalidValue);
    assert_eq!(error.path, "$/request_sha256");
}

#[test]
fn adapter_transport_rejects_every_identity_substitution() {
    let request = prepared_draft_adapter_request();
    let response = String::from_utf8(
        draft_response_for_payload(&request, &synthetic_payload())
            .expect("canonical synthetic response"),
    )
    .expect("UTF-8 response");
    let mutations = [
        (
            "d005-adapter-response-v0.1-draft",
            "d005-adapter-response-v0.2-draft",
            "$/schema_version",
        ),
        (
            SYNTHETIC_INPUT_MANIFEST_SHA256,
            &"3".repeat(64),
            "$/input_manifest_sha256",
        ),
        (
            SYNTHETIC_PAYLOAD_SCHEMA_SHA256,
            &"4".repeat(64),
            "$/payload_schema_sha256",
        ),
        (
            "\"candidate\":\"AM-01\"",
            "\"candidate\":\"AM-02\"",
            "$/candidate",
        ),
        ("\"case\":\"AC-01\"", "\"case\":\"AC-02\"", "$/case"),
        ("\"ordinal\":1", "\"ordinal\":2", "$/ordinal"),
        (
            "\"workspace_replay\":1",
            "\"workspace_replay\":2",
            "$/workspace_replay",
        ),
        (
            "\"render_repetition\":1",
            "\"render_repetition\":2",
            "$/render_repetition",
        ),
    ];
    for (before, after, expected_path) in mutations {
        let mutated = response.replacen(before, after, 1);
        assert_ne!(mutated, response, "mutation must alter response");
        let error = adapter::validate_capture(&request, &successful_capture(mutated.into_bytes()))
            .expect_err("identity substitution");
        assert_eq!(error.kind, TransportErrorKind::InvalidValue);
        assert_eq!(error.path, expected_path);
    }

    for (field, expected_path) in [
        ("packet_sha256", "$/packet_sha256"),
        ("plan_sha256", "$/plan_sha256"),
        ("request_sha256", "$/request_sha256"),
    ] {
        let marker = format!("\"{field}\":\"");
        let start = response.find(&marker).expect("digest field") + marker.len();
        let mut mutated = response.clone().into_bytes();
        mutated[start] = if mutated[start] == b'0' { b'1' } else { b'0' };
        let error = adapter::validate_capture(&request, &successful_capture(mutated))
            .expect_err("digest substitution");
        assert_eq!(error.kind, TransportErrorKind::InvalidValue);
        assert_eq!(error.path, expected_path);
    }
}

#[test]
fn adapter_transport_rejects_malformed_open_or_noncanonical_envelopes() {
    let request = prepared_draft_adapter_request();
    let canonical = draft_response_for_payload(&request, &synthetic_payload())
        .expect("canonical synthetic response");
    let text = String::from_utf8(canonical.clone()).expect("UTF-8 response");

    for (injected, expected_path) in [
        ("{\"unknown\":true,", "$/unknown"),
        ("{\"slash/tilde~\":true,", "$/slash~1tilde~0"),
        ("{\"line\\nbreak\":true,", "$/line\\nbreak"),
        ("{\"é\":true,", "$/\\u{e9}"),
        ("{\"literal\\\\escape\":true,", "$/literal\\\\escape"),
    ] {
        let unknown = text.replacen('{', injected, 1);
        let error = adapter::validate_capture(&request, &successful_capture(unknown.into_bytes()))
            .expect_err("unknown field");
        assert_eq!(error.kind, TransportErrorKind::UnknownField);
        assert_eq!(error.path, expected_path);
        assert!(!error.path.chars().any(char::is_control));
    }

    let missing = text.replace(",\"payload_schema_sha256\":\"2222222222222222222222222222222222222222222222222222222222222222\"", "");
    let error = adapter::validate_capture(&request, &successful_capture(missing.into_bytes()))
        .expect_err("missing field");
    assert_eq!(error.kind, TransportErrorKind::MissingField);
    assert_eq!(error.path, "$/payload_schema_sha256");

    let duplicate = text.replacen('{', "{\"ordinal\":1,", 1);
    let error = adapter::validate_capture(&request, &successful_capture(duplicate.into_bytes()))
        .expect_err("duplicate field");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::DuplicateKey)
    );

    let floating = text.replace("\"ordinal\":1", "\"ordinal\":1.0");
    let error = adapter::validate_capture(&request, &successful_capture(floating.into_bytes()))
        .expect_err("floating point");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::FloatingPoint)
    );

    let mut invalid_utf8 = canonical.clone();
    invalid_utf8.insert(1, 0xff);
    let error = adapter::validate_capture(&request, &successful_capture(invalid_utf8))
        .expect_err("invalid UTF-8");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::InvalidUtf8)
    );

    let noncanonical = format!(" {}", text);
    let error = adapter::validate_capture(&request, &successful_capture(noncanonical.into_bytes()))
        .expect_err("noncanonical whitespace");
    assert_eq!(error.kind, TransportErrorKind::NonCanonical);

    let mut extra_line_feed = canonical.clone();
    extra_line_feed.push(b'\n');
    let error = adapter::validate_capture(&request, &successful_capture(extra_line_feed))
        .expect_err("extra line feed");
    assert_eq!(error.kind, TransportErrorKind::NonCanonical);

    let no_line_feed = canonical
        .strip_suffix(b"\n")
        .expect("one line feed")
        .to_vec();
    let error = adapter::validate_capture(&request, &successful_capture(no_line_feed))
        .expect_err("missing line feed");
    assert_eq!(error.kind, TransportErrorKind::MissingLineFeed);

    let non_object_payload = text.replace("\"payload\":{\"synthetic\":true}", "\"payload\":[]");
    let error = adapter::validate_capture(
        &request,
        &successful_capture(non_object_payload.into_bytes()),
    )
    .expect_err("non-object payload");
    assert_eq!(error.kind, TransportErrorKind::InvalidValue);
    assert_eq!(error.path, "$/payload");
}

#[test]
fn adapter_capture_fails_closed_for_every_process_or_resource_failure() {
    let request = prepared_draft_adapter_request();
    let response = draft_response_for_payload(&request, &synthetic_payload())
        .expect("canonical synthetic response");

    for termination in [
        CapturedTermination::Exited(1),
        CapturedTermination::Signaled,
        CapturedTermination::TimedOut,
        CapturedTermination::StdoutLimit,
        CapturedTermination::StderrLimit,
        CapturedTermination::SpawnFailed,
        CapturedTermination::IoFailed,
        CapturedTermination::UnsupportedSandbox,
    ] {
        let mut capture = successful_capture(response.clone());
        capture.termination = termination;
        let error = adapter::validate_capture(&request, &capture).expect_err("process failure");
        assert_eq!(error.kind, TransportErrorKind::Termination(termination));
        assert_eq!(error.path, "$/termination");
    }

    let mut stdout_truncated = successful_capture(response.clone());
    stdout_truncated.stdout_truncated = true;
    let error = adapter::validate_capture(&request, &stdout_truncated)
        .expect_err("truncated standard output");
    assert_eq!(error.kind, TransportErrorKind::StdoutTruncated);

    let mut stderr_truncated = successful_capture(response.clone());
    stderr_truncated.stderr_truncated = true;
    let error = adapter::validate_capture(&request, &stderr_truncated)
        .expect_err("truncated standard error");
    assert_eq!(error.kind, TransportErrorKind::StderrTruncated);

    let mut noisy_success = successful_capture(response);
    noisy_success.stderr = b"warning\n".to_vec();
    let error = adapter::validate_capture(&request, &noisy_success).expect_err("unexpected stderr");
    assert_eq!(error.kind, TransportErrorKind::UnexpectedStderr);

    assert_eq!(
        adapter::validate_output_lengths(BUDGETS.max_output_bytes, 0),
        Ok(BUDGETS.max_output_bytes)
    );
    assert_eq!(
        adapter::validate_output_lengths(BUDGETS.max_output_bytes - 1, 1),
        Ok(BUDGETS.max_output_bytes)
    );
    assert_eq!(
        adapter::validate_output_lengths(BUDGETS.max_output_bytes, 1)
            .expect_err("combined output limit")
            .kind,
        TransportErrorKind::OutputTooLarge
    );
    assert_eq!(
        adapter::validate_output_lengths(usize::MAX, 1)
            .expect_err("output size overflow")
            .kind,
        TransportErrorKind::OutputSizeOverflow
    );

    let oversized = CapturedProcess {
        termination: CapturedTermination::Exited(0),
        stdout: vec![b'x'; BUDGETS.max_output_bytes + 1],
        stderr: Vec::new(),
        stdout_truncated: false,
        stderr_truncated: false,
    };
    let error = adapter::validate_capture(&request, &oversized).expect_err("oversized output");
    assert_eq!(error.kind, TransportErrorKind::OutputTooLarge);
}

#[test]
fn draft_response_construction_preflights_every_structural_and_byte_boundary() {
    let request = prepared_draft_adapter_request();

    let escaped_payload = strict_json::object([
        (
            "escaped".to_owned(),
            JsonValue::String("\"\\\u{0008}\u{000c}\n\r\t\u{0001}é".to_owned()),
        ),
        (
            "items".to_owned(),
            JsonValue::Array(vec![
                JsonValue::Null,
                JsonValue::Bool(true),
                JsonValue::Bool(false),
                JsonValue::Integer(-12),
            ]),
        ),
    ]);
    let escaped_response = draft_response_for_payload(&request, &escaped_payload)
        .expect("escaped response length matches canonical serialization");
    let escaped_capture =
        adapter::validate_capture(&request, &successful_capture(escaped_response))
            .expect("escaped response round trip");
    assert_eq!(escaped_capture.value(), &escaped_payload);

    let exact_nested_arrays = BUDGETS.max_json_depth - 2;
    let exact_depth_payload = format!(
        "{{\"nested\":{}0{}}}",
        "[".repeat(exact_nested_arrays),
        "]".repeat(exact_nested_arrays)
    );
    adapter::canonical_draft_response_bytes(&request, exact_depth_payload.as_bytes())
        .expect("exact response depth");
    let too_deep_payload = format!(
        "{{\"nested\":{}0{}}}",
        "[".repeat(exact_nested_arrays + 1),
        "]".repeat(exact_nested_arrays + 1)
    );
    let error = adapter::canonical_draft_response_bytes(&request, too_deep_payload.as_bytes())
        .expect_err("response depth plus one");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::DepthLimit)
    );
    assert_eq!(error.path, "$/response@0");

    let adversarial_depth = 100_000;
    let deeply_nested_payload = format!(
        "{{\"nested\":{}0{}}}",
        "[".repeat(adversarial_depth),
        "]".repeat(adversarial_depth)
    );
    let error = adapter::canonical_draft_response_bytes(&request, deeply_nested_payload.as_bytes())
        .expect_err("untrusted payload bytes cannot create an over-deep owned tree");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::DepthLimit)
    );
    assert!(error.path.starts_with("$/payload@"));

    let exact_scalar_nodes = BUDGETS.max_json_nodes - 14;
    let exact_node_payload = strict_json::object([(
        "nodes".to_owned(),
        JsonValue::Array(vec![JsonValue::Null; exact_scalar_nodes]),
    )]);
    draft_response_for_payload(&request, &exact_node_payload).expect("exact response node count");
    let too_many_node_payload = strict_json::object([(
        "nodes".to_owned(),
        JsonValue::Array(vec![JsonValue::Null; exact_scalar_nodes + 1]),
    )]);
    let error = draft_response_for_payload(&request, &too_many_node_payload)
        .expect_err("response node count plus one");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::NodeLimit)
    );

    let exact_string_payload = strict_json::object([(
        "text".to_owned(),
        JsonValue::String("x".repeat(BUDGETS.max_string_bytes)),
    )]);
    draft_response_for_payload(&request, &exact_string_payload)
        .expect("exact response string length");
    let too_long_string_payload = strict_json::object([(
        "text".to_owned(),
        JsonValue::String("x".repeat(BUDGETS.max_string_bytes + 1)),
    )]);
    let error = draft_response_for_payload(&request, &too_long_string_payload)
        .expect_err("response string length plus one");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::StringLimit)
    );

    let exact_integer_payload = strict_json::object([(
        "integer".to_owned(),
        JsonValue::Integer(9_007_199_254_740_991),
    )]);
    draft_response_for_payload(&request, &exact_integer_payload).expect("exact I-JSON integer");
    let out_of_range_integer_payload = strict_json::object([(
        "integer".to_owned(),
        JsonValue::Integer(9_007_199_254_740_992),
    )]);
    let error = draft_response_for_payload(&request, &out_of_range_integer_payload)
        .expect_err("I-JSON integer plus one");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::IntegerRange)
    );

    let empty_chunks = chunk_payload(0, 0);
    let base_length = draft_response_for_payload(&request, &empty_chunks)
        .expect("empty chunk response")
        .len();
    let remaining = BUDGETS.max_output_bytes - base_length;
    let maximum_item_cost = BUDGETS.max_string_bytes + 3;
    let item_count = (remaining + 1).div_ceil(maximum_item_cost) + 1;
    let syntax_cost = 3 * item_count - 1;
    let exact_string_bytes = remaining - syntax_cost;
    assert!(exact_string_bytes < item_count * BUDGETS.max_string_bytes);

    let exact_payload = chunk_payload(exact_string_bytes, item_count);
    let exact_output = draft_response_for_payload(&request, &exact_payload)
        .expect("exact combined output ceiling");
    assert_eq!(exact_output.len(), BUDGETS.max_output_bytes);
    let oversized_payload = chunk_payload(exact_string_bytes + 1, item_count);
    let error = draft_response_for_payload(&request, &oversized_payload)
        .expect_err("combined output ceiling plus one");
    assert_eq!(
        error.kind,
        TransportErrorKind::Json(JsonErrorKind::InputTooLarge)
    );
    assert_eq!(error.path, "$/response@0");
}

#[test]
fn strict_json_transport_parsing_can_use_the_distinct_output_ceiling() {
    let expanded = vec![b' '; BUDGETS.max_packet_bytes + 1];
    assert_eq!(
        strict_json::parse(&expanded)
            .expect_err("packet ceiling")
            .kind,
        JsonErrorKind::InputTooLarge
    );
    strict_json::parse_with_max_input(&expanded, BUDGETS.max_output_bytes)
        .expect_err("whitespace has no JSON value");

    let mut padded = Vec::with_capacity(BUDGETS.max_packet_bytes + 1);
    padded.push(b'0');
    padded.resize(BUDGETS.max_packet_bytes + 1, b' ');
    assert_eq!(
        strict_json::parse(&padded)
            .expect_err("packet ceiling")
            .kind,
        JsonErrorKind::InputTooLarge
    );
    assert_eq!(
        strict_json::parse_with_max_input(&padded, BUDGETS.max_output_bytes)
            .expect("result ceiling"),
        JsonValue::Integer(0)
    );

    let mut exact_output = vec![b' '; BUDGETS.max_output_bytes];
    exact_output[0] = b'0';
    assert_eq!(
        strict_json::parse_with_max_input(&exact_output, BUDGETS.max_output_bytes)
            .expect("exact transport ceiling"),
        JsonValue::Integer(0)
    );
    exact_output.push(b' ');
    assert_eq!(
        strict_json::parse_with_max_input(&exact_output, BUDGETS.max_output_bytes)
            .expect_err("transport ceiling plus one")
            .kind,
        JsonErrorKind::InputTooLarge
    );
}
