//! Input-only substrate checks for the D-009 solver-trust decision suite.
//!
//! These checks bind the draft-unfrozen packet and eight-row missing-input
//! index, then enumerate 24 candidate-case identities in memory. They launch
//! no solver or process, authorize no physical run order, freeze no evidence
//! epoch, and create no result, certificate, proof, selection, or trust claim.

#[path = "d009_support/domain.rs"]
mod domain;
#[path = "d009_support/packet.rs"]
mod packet;
#[path = "d009_support/runner.rs"]
mod runner;
#[path = "d005_support/sha256.rs"]
mod sha256;
#[path = "d005_support/strict_json.rs"]
mod strict_json;

use std::collections::BTreeSet;

use domain::{
    ATOMIC_OUTCOME_MEANINGS, ATOMIC_OUTCOMES, BUDGETS, CANDIDATE_STATES, CANDIDATES,
    CASE_INPUT_NONCLAIMS, CASES, COMPARATIVE_LABELS, CONCLUSIONS, HARD_GATE_COUNT,
    HARD_GATE_STATE_PRECEDENCE, HARD_GATE_STATES, INPUT_BINDINGS, InputBindingId, METRICS,
    NONCLAIMS, OWNER_SCOPES, PROTOCOL_COUNTS, PROTOCOL_GAPS, REQUIRED_CANDIDATE_CASES,
    SEMANTIC_BINDINGS, SEMANTIC_NORMALIZATION,
};
use packet::{
    CASE_INPUT_INDEX_CANONICAL_SHA256, PacketErrorKind, canonical_case_input_index_bytes,
    canonical_case_input_index_file_bytes, canonical_draft_packet_bytes,
    canonical_draft_packet_file_bytes, case_input_index_digest_hex, parse_case_input_index,
    parse_draft_packet,
};
use runner::{ReplayError, ReplayInputs};
use strict_json::{JsonErrorKind, JsonValue};

const CHECKED_IN_PACKET: &[u8] =
    include_bytes!("../../../../research/decisions/D-009/d009-v0.1-draft-packet.json");
const CASE_INPUT_INDEX: &[u8] =
    include_bytes!("../../../../research/decisions/D-009/d009-v0.1-case-input-index.json");
const SOLVER_TRUST_SUITE: &[u8] = include_bytes!("../../../../docs/SOLVER_TRUST_DECISION_SUITE.md");

const PACKET_CANONICAL_SHA256: &str =
    "fa1411c83fdb6b57b8100f296ca904d88155ad6aa57bd7f48af62ae90c9ead31";
const PACKET_RAW_SHA256: &str = "c0ad0227f1f374da8796c6db4866188213be53bd904bbf22b141ee1de6e57171";
const INDEX_RAW_SHA256: &str = "c5298d625f5392de2774ffb861fe1dc1701b379ebd385cde0584a8cbcd249859";

fn checked_in_replay_inputs() -> ReplayInputs<'static> {
    ReplayInputs::new([CASE_INPUT_INDEX, SOLVER_TRUST_SUITE])
}

fn canonical_file_bytes(mut canonical: Vec<u8>) -> Vec<u8> {
    canonical.push(b'\n');
    canonical
}

#[test]
fn d009_pre_epoch_domain_is_exact_symmetric_and_non_executing() {
    assert_eq!(
        CANDIDATES.map(|candidate| (candidate.as_str(), candidate.name())),
        [
            ("SP-01", "Checked-artifact portfolio"),
            ("SP-02", "Kernel-only reconstruction"),
            ("SP-03", "Direct trusted-solver authority"),
        ]
    );
    assert_eq!(
        CASES.map(|case| case.as_str()),
        [
            "TC-01", "TC-02", "TC-03", "TC-04", "TC-05", "TC-06", "TC-07", "TC-08",
        ]
    );
    assert_eq!(CANDIDATES.len() * CASES.len(), REQUIRED_CANDIDATE_CASES);
    assert_eq!(REQUIRED_CANDIDATE_CASES, 24);
    assert_eq!(METRICS.len(), 16);
    assert_eq!(METRICS[0], "M-01");
    assert_eq!(METRICS[15], "M-16");
    assert_eq!(HARD_GATE_COUNT, 8);
    assert_eq!(
        ATOMIC_OUTCOMES,
        ["satisfied", "not_satisfied", "unresolved", "unsupported"]
    );
    assert_eq!(
        ATOMIC_OUTCOME_MEANINGS,
        [
            (
                "satisfied",
                "the exact proposition has its complete permitted mandatory closure and no valid decisive negative result",
            ),
            (
                "not_satisfied",
                "permitted, identity-bound negative evidence establishes that the exact proposition is false or violated within its scope; absence or incompleteness alone is not not_satisfied",
            ),
            (
                "unresolved",
                "the claim is well-formed and within the declared support model, but a required decision remains unknown, incomplete, conflicting, or exhausted",
            ),
            (
                "unsupported",
                "the declared policy or support envelope offers no permitted evaluation or authority path for that exact claim and scope",
            ),
        ]
    );
    assert_eq!(
        HARD_GATE_STATES,
        ["pass", "fail", "unresolved", "unsupported"]
    );
    assert_eq!(
        HARD_GATE_STATE_PRECEDENCE,
        ["unsupported", "fail", "unresolved", "pass"]
    );
    assert_eq!(OWNER_SCOPES.len(), 8);
    assert_eq!(OWNER_SCOPES[0], "SR-01");
    assert_eq!(OWNER_SCOPES[7], "SR-08");
    assert_eq!(
        COMPARATIVE_LABELS,
        [
            "checked_artifact_better",
            "kernel_only_better",
            "trusted_solver_better",
            "practically_equivalent",
            "inconclusive",
        ]
    );
    assert_eq!(
        CONCLUSIONS,
        [
            "recommend_checked_artifact",
            "recommend_kernel_only",
            "recommend_trusted_solver",
            "tie",
            "inconclusive",
        ]
    );
    assert_eq!(PROTOCOL_GAPS.len(), 11);
    assert_eq!(NONCLAIMS.len(), 13);
    assert_eq!(CASE_INPUT_NONCLAIMS.len(), 6);
    assert_eq!(INPUT_BINDINGS.len(), 2);
    assert_eq!(SEMANTIC_BINDINGS.len(), 5);
    assert_eq!(SEMANTIC_NORMALIZATION, "markdown-prose-lines-exact-v1");
    assert_eq!(CANDIDATE_STATES.len(), CANDIDATES.len());
    for (state, candidate) in CANDIDATE_STATES.into_iter().zip(CANDIDATES) {
        assert_eq!(state.candidate, candidate);
    }

    assert_eq!(BUDGETS.max_packet_bytes, 262_144);
    assert_eq!(BUDGETS.max_json_depth, 32);
    assert_eq!(BUDGETS.max_json_nodes, 16_384);
    assert_eq!(BUDGETS.max_string_bytes, 16_384);
    assert_eq!(PROTOCOL_COUNTS.cold_bootstrap_runs, 5);
    assert_eq!(PROTOCOL_COUNTS.deterministic_profile_runs, 3);
    assert_eq!(PROTOCOL_COUNTS.maximum_same_owner_reproducibility_level, 2);
    assert_eq!(PROTOCOL_COUNTS.owner_workspaces, 2);
    assert_eq!(PROTOCOL_COUNTS.timed_replays_per_case, 30);
    assert_eq!(PROTOCOL_COUNTS.unmeasured_warmups, 1);
}

#[test]
fn case_input_index_is_canonical_digest_bound_and_zero_fixture() {
    assert_eq!(CASE_INPUT_INDEX, canonical_case_input_index_file_bytes());
    let index = parse_case_input_index(CASE_INPUT_INDEX).expect("checked-in case-input index");
    assert_eq!(index.canonical_bytes(), canonical_case_input_index_bytes());
    assert_eq!(index.digest(), &sha256::digest(index.canonical_bytes()));
    assert_eq!(index.digest_hex(), CASE_INPUT_INDEX_CANONICAL_SHA256);
    assert_eq!(
        case_input_index_digest_hex(),
        CASE_INPUT_INDEX_CANONICAL_SHA256
    );
    assert_eq!(
        sha256::hex(&sha256::digest(CASE_INPUT_INDEX)),
        INDEX_RAW_SHA256
    );

    let root = index.value().as_object().expect("index root");
    assert_eq!(
        root.get("executable_inputs_status")
            .and_then(JsonValue::as_str),
        Some("absent")
    );
    assert_eq!(
        root.get("evidence_status").and_then(JsonValue::as_str),
        Some("none")
    );
    assert_eq!(
        root.get("owner_protocol_review")
            .and_then(JsonValue::as_str),
        Some("none")
    );
    let rows = root
        .get("case_inputs")
        .and_then(JsonValue::as_array)
        .expect("case rows");
    assert_eq!(rows.len(), CASES.len());
    for (row, case) in rows.iter().zip(CASES) {
        let row = row.as_object().expect("case row");
        assert_eq!(
            row.get("case").and_then(JsonValue::as_str),
            Some(case.as_str())
        );
        assert_eq!(
            row.get("shared_inputs_status").and_then(JsonValue::as_str),
            Some("absent")
        );
        assert_eq!(
            row.get("candidate_mapping_status")
                .and_then(JsonValue::as_str),
            Some("absent")
        );
        assert_eq!(
            row.get("coverage_status").and_then(JsonValue::as_str),
            Some("unresolved")
        );
        assert_eq!(
            row.get("executable_fixture_count")
                .and_then(JsonValue::as_integer),
            Some(0)
        );
        assert_eq!(row.get("freeze_blocker"), Some(&JsonValue::Bool(true)));
    }
}

#[test]
fn draft_packet_is_exact_canonical_self_bound_and_zero_baseline() {
    assert_eq!(CHECKED_IN_PACKET, canonical_draft_packet_file_bytes());
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in D-009 packet");
    assert_eq!(packet.canonical_bytes(), canonical_draft_packet_bytes());
    assert_eq!(packet.digest(), &sha256::digest(packet.canonical_bytes()));
    assert_eq!(packet.digest_hex(), PACKET_CANONICAL_SHA256);
    assert_eq!(
        sha256::hex(&sha256::digest(CHECKED_IN_PACKET)),
        PACKET_RAW_SHA256
    );
    assert_eq!(
        packet.case_input_index_sha256(),
        CASE_INPUT_INDEX_CANONICAL_SHA256
    );
    for binding in INPUT_BINDINGS {
        assert_eq!(packet.input_binding(binding.id), binding);
    }
    for binding in SEMANTIC_BINDINGS {
        assert_eq!(packet.semantic_binding(binding.id), binding);
    }

    let root = packet.value().as_object().expect("packet root");
    assert_eq!(
        root.get("schema_version").and_then(JsonValue::as_str),
        Some("d009-pre-epoch-packet-v0.3")
    );
    let atomic_outcomes = root
        .get("atomic_outcomes")
        .and_then(JsonValue::as_array)
        .expect("atomic outcomes");
    assert_eq!(
        atomic_outcomes
            .iter()
            .map(JsonValue::as_str)
            .collect::<Vec<_>>(),
        ATOMIC_OUTCOMES.into_iter().map(Some).collect::<Vec<_>>()
    );
    let atomic_outcome_meanings = root
        .get("atomic_outcome_meanings")
        .and_then(JsonValue::as_object)
        .expect("atomic outcome meanings");
    assert_eq!(atomic_outcome_meanings.len(), ATOMIC_OUTCOME_MEANINGS.len());
    for (outcome, meaning) in ATOMIC_OUTCOME_MEANINGS {
        assert_eq!(
            atomic_outcome_meanings
                .get(outcome)
                .and_then(JsonValue::as_str),
            Some(meaning)
        );
    }
    let hard_gate_state_precedence = root
        .get("hard_gate_state_precedence")
        .and_then(JsonValue::as_array)
        .expect("hard-gate state precedence");
    assert_eq!(
        hard_gate_state_precedence
            .iter()
            .map(JsonValue::as_str)
            .collect::<Vec<_>>(),
        HARD_GATE_STATE_PRECEDENCE
            .into_iter()
            .map(Some)
            .collect::<Vec<_>>()
    );
    assert_eq!(root.get("epoch"), Some(&JsonValue::Null));
    assert_eq!(root.get("physical_execution_order"), Some(&JsonValue::Null));
    assert_eq!(root.get("selection"), Some(&JsonValue::Null));
    assert_eq!(root.get("conclusion"), Some(&JsonValue::Null));
    let dependencies = root
        .get("dependency_acceptance")
        .and_then(JsonValue::as_object)
        .expect("dependency acceptance");
    for dependency in ["D-004", "D-005"] {
        assert_eq!(dependencies.get(dependency), Some(&JsonValue::Bool(false)));
    }
    let resources = root
        .get("execution_resource_state")
        .and_then(JsonValue::as_object)
        .expect("resource state");
    for field in [
        "case_output_bytes",
        "case_peak_memory_bytes",
        "case_temp_storage_bytes",
        "case_wall_seconds",
    ] {
        assert_eq!(resources.get(field), Some(&JsonValue::Null));
    }
    let states = root
        .get("candidate_states")
        .and_then(JsonValue::as_array)
        .expect("candidate states");
    assert_eq!(states.len(), CANDIDATES.len());
    for (state, candidate) in states.iter().zip(CANDIDATES) {
        let state = state.as_object().expect("candidate state");
        assert_eq!(
            state.get("candidate").and_then(JsonValue::as_str),
            Some(candidate.as_str())
        );
        assert_eq!(
            state
                .get("implementation_status")
                .and_then(JsonValue::as_str),
            Some("absent")
        );
        assert_eq!(
            state
                .get("dependency_admission_status")
                .and_then(JsonValue::as_str),
            Some("absent")
        );
        assert_eq!(
            state.get("adapter_status").and_then(JsonValue::as_str),
            Some("absent")
        );
        assert_eq!(
            state.get("execution_status").and_then(JsonValue::as_str),
            Some("not_performed")
        );
    }
}

#[test]
fn packet_and_index_reject_malformed_open_or_noncanonical_json() {
    let packet = String::from_utf8(CHECKED_IN_PACKET.to_vec()).expect("UTF-8 packet");
    let index = String::from_utf8(CASE_INPUT_INDEX.to_vec()).expect("UTF-8 index");

    let duplicate = packet.replacen('{', "{\"status\":\"draft_unfrozen\",", 1);
    assert_eq!(
        parse_draft_packet(duplicate.as_bytes())
            .expect_err("duplicate packet field")
            .kind,
        PacketErrorKind::Json(JsonErrorKind::DuplicateKey)
    );
    let floating = packet.replace(
        "\"max_packet_bytes\":262144",
        "\"max_packet_bytes\":262144.0",
    );
    assert_eq!(
        parse_draft_packet(floating.as_bytes())
            .expect_err("floating packet integer")
            .kind,
        PacketErrorKind::Json(JsonErrorKind::FloatingPoint)
    );
    let unknown = packet.replacen('{', "{\"unknown\":null,", 1);
    let error = parse_draft_packet(unknown.as_bytes()).expect_err("open packet root");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/unknown");
    let nested_unknown = packet.replace(
        "\"adapter_status\":\"absent\"",
        "\"adapter_status\":\"absent\",\"authority\":null",
    );
    let error = parse_draft_packet(nested_unknown.as_bytes()).expect_err("open candidate state");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/candidate_states/0/authority");
    let missing = packet.replace(",\"selection\":null", "");
    let error = parse_draft_packet(missing.as_bytes()).expect_err("missing selection");
    assert_eq!(error.kind, PacketErrorKind::MissingField);
    assert_eq!(error.path, "$/selection");

    for noncanonical in [
        canonical_draft_packet_bytes(),
        [b" ".as_slice(), CHECKED_IN_PACKET].concat(),
    ] {
        assert_eq!(
            parse_draft_packet(&noncanonical)
                .expect_err("noncanonical packet")
                .kind,
            PacketErrorKind::NonCanonicalEncoding
        );
    }

    let index_unknown = index.replace(
        "\"candidate_mapping_status\":\"absent\"",
        "\"authority\":null,\"candidate_mapping_status\":\"absent\"",
    );
    let error = parse_case_input_index(index_unknown.as_bytes()).expect_err("open index row");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/case_inputs/0/authority");
    assert_eq!(
        parse_case_input_index(&canonical_case_input_index_bytes())
            .expect_err("index without terminal LF")
            .kind,
        PacketErrorKind::NonCanonicalEncoding
    );
}

#[test]
fn draft_packet_rejects_every_authority_or_zero_baseline_substitution() {
    let canonical = String::from_utf8(CHECKED_IN_PACKET.to_vec()).expect("UTF-8 packet");
    let drifts = [
        canonical.replacen("\"D-004\":false", "\"D-004\":true", 1),
        canonical.replacen("\"epoch\":null", "\"epoch\":\"0001\"", 1),
        canonical.replacen(
            "\"epoch_status\":\"unfrozen\"",
            "\"epoch_status\":\"frozen\"",
            1,
        ),
        canonical.replacen(
            "\"completed_candidate_cases\":0",
            "\"completed_candidate_cases\":1",
            1,
        ),
        canonical.replacen("\"complete_candidates\":0", "\"complete_candidates\":1", 1),
        canonical.replacen(
            "\"complete_cross_candidate_cases\":0",
            "\"complete_cross_candidate_cases\":1",
            1,
        ),
        canonical.replacen(
            "\"evidence_status\":\"none\"",
            "\"evidence_status\":\"complete\"",
            1,
        ),
        canonical.replacen("\"case_wall_seconds\":null", "\"case_wall_seconds\":900", 1),
        canonical.replacen(
            "\"implementation_status\":\"absent\"",
            "\"implementation_status\":\"present\"",
            1,
        ),
        canonical.replacen(
            "\"dependency_admission_status\":\"absent\"",
            "\"dependency_admission_status\":\"admitted\"",
            1,
        ),
        canonical.replacen(
            "\"physical_execution_order\":null",
            "\"physical_execution_order\":[]",
            1,
        ),
        canonical.replacen("\"selection\":null", "\"selection\":\"SP-01\"", 1),
        canonical.replacen(
            "\"conclusion\":null",
            "\"conclusion\":\"recommend_checked_artifact\"",
            1,
        ),
        canonical.replacen(CASE_INPUT_INDEX_CANONICAL_SHA256, &"0".repeat(64), 1),
        canonical.replace(",\"no roadmap gate or readiness movement\"", ""),
    ];
    for drift in drifts {
        assert_eq!(
            parse_draft_packet(drift.as_bytes())
                .expect_err("packet authority drift")
                .kind,
            PacketErrorKind::InvalidValue
        );
    }
}

#[test]
fn draft_packet_rejects_atomic_outcome_or_gate_precedence_drift() {
    let canonical = String::from_utf8(CHECKED_IN_PACKET.to_vec()).expect("UTF-8 packet");
    let semantic_drifts = [
        canonical.replace(
            "\"atomic_outcomes\":[\"satisfied\",\"not_satisfied\",\"unresolved\",\"unsupported\"]",
            "\"atomic_outcomes\":[\"not_satisfied\",\"satisfied\",\"unresolved\",\"unsupported\"]",
        ),
        canonical.replace(
            "\"atomic_outcomes\":[\"satisfied\",\"not_satisfied\",\"unresolved\",\"unsupported\"]",
            "\"atomic_outcomes\":[\"satisfied\",\"not_satisfied\",\"unresolved\",\"unresolved\"]",
        ),
        canonical.replace(
            "the exact proposition has its complete permitted mandatory closure and no valid decisive negative result",
            "the exact proposition has favorable evidence",
        ),
        canonical.replace(
            "permitted, identity-bound negative evidence establishes that the exact proposition is false or violated within its scope; absence or incompleteness alone is not not_satisfied",
            "missing evidence establishes that the exact proposition is false",
        ),
        canonical.replace(
            "the claim is well-formed and within the declared support model, but a required decision remains unknown, incomplete, conflicting, or exhausted",
            "the claim is malformed or outside the declared support model",
        ),
        canonical.replace(
            "the declared policy or support envelope offers no permitted evaluation or authority path for that exact claim and scope",
            "the declared policy offers an inconclusive evaluation path",
        ),
        canonical.replace(
            "\"hard_gate_state_precedence\":[\"unsupported\",\"fail\",\"unresolved\",\"pass\"]",
            "\"hard_gate_state_precedence\":[\"fail\",\"unsupported\",\"unresolved\",\"pass\"]",
        ),
        canonical.replace(
            "\"hard_gate_state_precedence\":[\"unsupported\",\"fail\",\"unresolved\",\"pass\"]",
            "\"hard_gate_state_precedence\":[\"unsupported\",\"fail\",\"unresolved\",\"unresolved\"]",
        ),
        canonical.replacen(
            "bc4bf5fd534a61efdd62e16b57633bea1f5ad8f3224555f310911a3ab26bb41a",
            &"0".repeat(64),
            1,
        ),
        canonical.replacen(
            "markdown-prose-lines-exact-v1",
            "markdown-prose-lines-exact-v2",
            1,
        ),
        canonical.replacen(
            "\"scope\":\"whole_document\"",
            "\"scope\":\"markdown_exact_heading_range\"",
            1,
        ),
        canonical.replacen(
            "\"section_start_heading\":null",
            "\"section_start_heading\":\"## D-009 — Solver trust\"",
            1,
        ),
    ];
    for drift in semantic_drifts {
        assert_eq!(
            parse_draft_packet(drift.as_bytes())
                .expect_err("atomic outcome or precedence drift")
                .kind,
            PacketErrorKind::InvalidValue
        );
    }

    let missing_outcomes = canonical.replace(
        ",\"atomic_outcomes\":[\"satisfied\",\"not_satisfied\",\"unresolved\",\"unsupported\"]",
        "",
    );
    let error =
        parse_draft_packet(missing_outcomes.as_bytes()).expect_err("missing atomic outcomes");
    assert_eq!(error.kind, PacketErrorKind::MissingField);
    assert_eq!(error.path, "$/atomic_outcomes");

    let unsupported_meaning = "\"unsupported\":\"the declared policy or support envelope offers no permitted evaluation or authority path for that exact claim and scope\"";
    let missing_meaning = canonical.replace(&format!(",{unsupported_meaning}"), "");
    let error =
        parse_draft_packet(missing_meaning.as_bytes()).expect_err("missing outcome meaning");
    assert_eq!(error.kind, PacketErrorKind::MissingField);
    assert_eq!(error.path, "$/atomic_outcome_meanings/unsupported");

    let unknown_meaning = canonical.replace(
        unsupported_meaning,
        "\"other\":\"the declared policy or support envelope offers no permitted evaluation or authority path for that exact claim and scope\"",
    );
    let error = parse_draft_packet(unknown_meaning.as_bytes()).expect_err("open outcome meanings");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/atomic_outcome_meanings/other");
}

#[test]
fn case_input_index_rejects_missing_reordered_or_weakened_rows() {
    let canonical = String::from_utf8(CASE_INPUT_INDEX.to_vec()).expect("UTF-8 index");
    let reordered = canonical
        .replace("TC-01", "TC-TEMP")
        .replace("TC-02", "TC-01")
        .replace("TC-TEMP", "TC-02");
    let drifts = [
        (
            canonical.replacen(",\"freeze_blocker\":true", "", 1),
            PacketErrorKind::MissingField,
        ),
        (
            canonical.replacen("\"freeze_blocker\":true", "\"freeze_blocker\":false", 1),
            PacketErrorKind::InvalidValue,
        ),
        (
            canonical.replacen(
                "\"executable_fixture_count\":0",
                "\"executable_fixture_count\":1",
                1,
            ),
            PacketErrorKind::InvalidValue,
        ),
        (
            canonical.replacen(
                "\"shared_inputs_status\":\"absent\"",
                "\"shared_inputs_status\":\"present\"",
                1,
            ),
            PacketErrorKind::InvalidValue,
        ),
        (
            canonical.replacen(
                "\"candidate_mapping_status\":\"absent\"",
                "\"candidate_mapping_status\":\"present\"",
                1,
            ),
            PacketErrorKind::InvalidValue,
        ),
        (
            canonical.replacen(
                "\"coverage_status\":\"unresolved\"",
                "\"coverage_status\":\"complete\"",
                1,
            ),
            PacketErrorKind::InvalidValue,
        ),
        (
            canonical.replacen(
                "\"evidence_status\":\"none\"",
                "\"evidence_status\":\"complete\"",
                1,
            ),
            PacketErrorKind::InvalidValue,
        ),
        (
            canonical.replace(",\"no capability or readiness credit\"", ""),
            PacketErrorKind::InvalidValue,
        ),
        (reordered, PacketErrorKind::InvalidValue),
    ];
    for (drift, expected_kind) in drifts {
        assert_eq!(
            parse_case_input_index(drift.as_bytes())
                .expect_err("index semantic drift")
                .kind,
            expected_kind
        );
    }
}

#[test]
fn replay_refuses_every_raw_binding_drift_before_identity_enumeration() {
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in packet");
    let inputs = checked_in_replay_inputs();
    for binding in INPUT_BINDINGS {
        assert_eq!(
            sha256::hex(&sha256::digest(inputs.get(binding.id))),
            binding.sha256
        );
    }
    runner::prepare_identity_plan(&packet, &inputs).expect("exact bound inputs");

    for binding in INPUT_BINDINGS {
        let drifted = inputs.with_replacement(binding.id, b"corrupted");
        let error =
            runner::prepare_identity_plan(&packet, &drifted).expect_err("raw input binding drift");
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
}

#[test]
fn packet_rejects_attempted_case_index_self_rebinding() {
    let original_packet = String::from_utf8(CHECKED_IN_PACKET.to_vec()).expect("UTF-8 packet");
    let drifted_index = String::from_utf8(CASE_INPUT_INDEX.to_vec())
        .expect("UTF-8 index")
        .replacen(
            "\"status\":\"draft_unreviewed\"",
            "\"status\":\"reviewed\"",
            1,
        );
    let drifted_value = strict_json::parse(drifted_index.as_bytes()).expect("well-formed drift");
    let drifted_canonical = strict_json::canonical_bytes(&drifted_value);
    let drifted_file = canonical_file_bytes(drifted_canonical.clone());
    let drifted_canonical_sha256 = sha256::hex(&sha256::digest(&drifted_canonical));
    let drifted_raw_sha256 = sha256::hex(&sha256::digest(&drifted_file));

    let rebound_packet = original_packet
        .replace(CASE_INPUT_INDEX_CANONICAL_SHA256, &drifted_canonical_sha256)
        .replace(
            INPUT_BINDINGS[InputBindingId::CaseInputIndex.index()].sha256,
            &drifted_raw_sha256,
        );
    assert_eq!(
        parse_draft_packet(rebound_packet.as_bytes())
            .expect_err("self-rebound packet")
            .kind,
        PacketErrorKind::InvalidValue
    );
    assert_eq!(
        parse_case_input_index(&drifted_file)
            .expect_err("self-rebound index")
            .kind,
        PacketErrorKind::InvalidValue
    );
}

#[test]
fn identity_inventory_is_case_major_candidate_minor_unique_and_still_zero_of_24() {
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in packet");
    let inputs = checked_in_replay_inputs();
    let first = runner::prepare_identity_plan(&packet, &inputs).expect("identity plan");
    let second = runner::prepare_identity_plan(&packet, &inputs).expect("identity plan");

    assert_eq!(first, second);
    assert_eq!(first.packet_sha256(), packet.digest_hex());
    assert_eq!(
        first.case_input_index_sha256(),
        CASE_INPUT_INDEX_CANONICAL_SHA256
    );
    assert_eq!(first.identities().len(), REQUIRED_CANDIDATE_CASES);
    let pair_counts = runner::identity_pair_counts(&first);
    assert_eq!(pair_counts.len(), REQUIRED_CANDIDATE_CASES);
    assert!(pair_counts.values().all(|count| *count == 1));

    for (case_index, identities) in first
        .identities()
        .chunks_exact(CANDIDATES.len())
        .enumerate()
    {
        assert_eq!(identities.len(), CANDIDATES.len());
        assert!(
            identities
                .iter()
                .all(|identity| identity.case == CASES[case_index])
        );
        assert_eq!(
            identities
                .iter()
                .map(|identity| identity.candidate)
                .collect::<BTreeSet<_>>(),
            CANDIDATES.into_iter().collect()
        );
        for (candidate_index, identity) in identities.iter().enumerate() {
            assert_eq!(identity.candidate, CANDIDATES[candidate_index]);
            assert_eq!(
                identity.ordinal,
                case_index * CANDIDATES.len() + candidate_index + 1
            );
        }
    }

    assert_eq!(first.completed_candidate_cases(), 0);
    assert_eq!(first.complete_candidates(), 0);
    assert_eq!(first.complete_cross_candidate_cases(), 0);
    assert_eq!(first.evidence_status(), "none");
    assert_eq!(first.physical_execution_order(), None);
    assert_eq!(first.selection(), None);
    assert_eq!(first.conclusion(), None);
    assert_eq!(first.canonical_bytes(), second.canonical_bytes());
    assert_eq!(first.digest_hex(), second.digest_hex());
    let plan_value = strict_json::parse(&first.canonical_bytes()).expect("canonical plan");
    assert_eq!(
        strict_json::canonical_bytes(&plan_value),
        first.canonical_bytes()
    );
    let plan_text = String::from_utf8(first.canonical_bytes()).expect("UTF-8 plan");
    assert!(plan_text.contains("\"physical_execution_order\":null"));
    assert!(plan_text.contains("\"completed_candidate_cases\":0"));
    assert!(plan_text.contains("\"evidence_status\":\"none\""));
    assert!(plan_text.contains("\"selection\":null"));
    assert!(plan_text.contains("\"conclusion\":null"));
    for nonclaim in NONCLAIMS {
        assert!(plan_text.contains(nonclaim));
    }
}
