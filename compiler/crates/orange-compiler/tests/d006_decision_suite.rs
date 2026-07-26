//! Input-only substrate checks for the D-006 proof-foundation decision suite.
//!
//! These checks bind the draft-unfrozen packet and seven-row missing-input
//! index, then enumerate 14 candidate-case identities in memory. They execute
//! no proof tool, authorize no physical run order, freeze no evidence epoch,
//! and create no candidate result, selection, or proof-bearing product claim.

#[path = "d006_support/domain.rs"]
mod domain;
#[path = "d006_support/packet.rs"]
mod packet;
#[path = "d006_support/runner.rs"]
mod runner;
#[path = "d005_support/sha256.rs"]
mod sha256;
#[path = "d005_support/strict_json.rs"]
mod strict_json;

use std::collections::BTreeSet;

use domain::{
    BUDGETS, CANDIDATES, CASE_INPUT_NONCLAIMS, CASES, COMPARATIVE_LABELS, CONCLUSIONS,
    HARD_GATE_COUNT, HARD_GATE_STATES, INPUT_BINDINGS, InputBindingId, METRICS, NONCLAIMS,
    OWNER_SCOPES, PROTOCOL_COUNTS, PROTOCOL_GAPS, REQUIRED_CANDIDATE_CASES, TOOL_STATES,
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
    include_bytes!("../../../../research/decisions/D-006/d006-v0.2-draft-packet.json");
const CASE_INPUT_INDEX: &[u8] =
    include_bytes!("../../../../research/decisions/D-006/d006-v0.2-case-input-index.json");
const PROOF_FOUNDATION_SUITE: &[u8] =
    include_bytes!("../../../../docs/PROOF_FOUNDATION_DECISION_SUITE.md");

fn checked_in_replay_inputs() -> ReplayInputs<'static> {
    ReplayInputs::new([CASE_INPUT_INDEX, PROOF_FOUNDATION_SUITE])
}

fn canonical_file_bytes(mut canonical: Vec<u8>) -> Vec<u8> {
    canonical.push(b'\n');
    canonical
}

#[test]
fn d006_pre_epoch_domain_is_exact_symmetric_and_non_executing() {
    assert_eq!(
        CANDIDATES.map(|candidate| (candidate.as_str(), candidate.name())),
        [("C-01", "Rocq"), ("C-02", "Lean 4")]
    );
    assert_eq!(
        CASES.map(|case| case.as_str()),
        [
            "DS-01", "DS-02", "DS-03", "DS-04", "DS-05", "DS-06", "DS-07"
        ]
    );
    assert_eq!(CANDIDATES.len() * CASES.len(), REQUIRED_CANDIDATE_CASES);
    assert_eq!(REQUIRED_CANDIDATE_CASES, 14);
    assert_eq!(METRICS.len(), 18);
    assert_eq!(METRICS[0], "M-01");
    assert_eq!(METRICS[17], "M-18");
    assert_eq!(HARD_GATE_COUNT, 8);
    assert_eq!(
        HARD_GATE_STATES,
        ["pass", "fail", "unresolved", "unsupported"]
    );
    assert_eq!(OWNER_SCOPES.len(), 9);
    assert_eq!(OWNER_SCOPES[0], "R-01");
    assert_eq!(OWNER_SCOPES[8], "R-09");
    assert_eq!(
        COMPARATIVE_LABELS,
        [
            "rocq_better",
            "lean_better",
            "practically_equivalent",
            "inconclusive"
        ]
    );
    assert_eq!(
        CONCLUSIONS,
        ["recommend_rocq", "recommend_lean", "tie", "inconclusive"]
    );
    assert_eq!(PROTOCOL_GAPS.len(), 10);
    assert_eq!(NONCLAIMS.len(), 11);
    assert_eq!(CASE_INPUT_NONCLAIMS.len(), 6);
    assert_eq!(INPUT_BINDINGS.len(), 2);
    assert_eq!(TOOL_STATES.len(), CANDIDATES.len());
    for (tool, candidate) in TOOL_STATES.into_iter().zip(CANDIDATES) {
        assert_eq!(tool.candidate, candidate);
        assert_eq!(tool.name, candidate.name());
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
        "1aec6a731bef0620c8500120ec8385d584f99a528b4a03c014e8516c55cc8136"
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
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in D-006 packet");
    assert_eq!(packet.canonical_bytes(), canonical_draft_packet_bytes());
    assert_eq!(packet.digest(), &sha256::digest(packet.canonical_bytes()));
    assert_eq!(
        packet.digest_hex(),
        "b56ad768c4584bdd00da4d4e85af642757b877dd5dc5ae438560ba4a486d9d21"
    );
    assert_eq!(
        sha256::hex(&sha256::digest(CHECKED_IN_PACKET)),
        "210eccad3a545927301d3cc147fdf918cc432fea65b8d71b79cbefc447e34bff"
    );
    assert_eq!(
        packet.case_input_index_sha256(),
        CASE_INPUT_INDEX_CANONICAL_SHA256
    );
    for binding in INPUT_BINDINGS {
        assert_eq!(packet.input_binding(binding.id), binding);
    }

    let root = packet.value().as_object().expect("packet root");
    assert_eq!(root.get("epoch"), Some(&JsonValue::Null));
    assert_eq!(root.get("physical_execution_order"), Some(&JsonValue::Null));
    assert_eq!(root.get("selection"), Some(&JsonValue::Null));
    assert_eq!(root.get("conclusion"), Some(&JsonValue::Null));
    let dependencies = root
        .get("dependency_acceptance")
        .and_then(JsonValue::as_object)
        .expect("dependency acceptance");
    assert_eq!(dependencies.get("D-004"), Some(&JsonValue::Bool(false)));
    assert_eq!(dependencies.get("D-005"), Some(&JsonValue::Bool(false)));
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
    let tools = root
        .get("tool_states")
        .and_then(JsonValue::as_array)
        .expect("tool states");
    for (tool, candidate) in tools.iter().zip(CANDIDATES) {
        let tool = tool.as_object().expect("tool state");
        assert_eq!(
            tool.get("candidate").and_then(JsonValue::as_str),
            Some(candidate.as_str())
        );
        assert_eq!(tool.get("version"), Some(&JsonValue::Null));
        assert_eq!(tool.get("content_sha256"), Some(&JsonValue::Null));
        assert_eq!(tool.get("dependency_graph_sha256"), Some(&JsonValue::Null));
        assert_eq!(
            tool.get("execution_status").and_then(JsonValue::as_str),
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
        "\"case_output_bytes\":null",
        "\"authority\":null,\"case_output_bytes\":null",
    );
    let error = parse_draft_packet(nested_unknown.as_bytes()).expect_err("open resource state");
    assert_eq!(error.kind, PacketErrorKind::UnknownField);
    assert_eq!(error.path, "$/execution_resource_state/authority");
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
        canonical.replacen("\"version\":null", "\"version\":\"unknown\"", 1),
        canonical.replacen(
            "\"physical_execution_order\":null",
            "\"physical_execution_order\":[]",
            1,
        ),
        canonical.replacen("\"selection\":null", "\"selection\":\"C-01\"", 1),
        canonical.replacen(
            "\"conclusion\":null",
            "\"conclusion\":\"recommend_rocq\"",
            1,
        ),
        canonical.replacen(CASE_INPUT_INDEX_CANONICAL_SHA256, &"0".repeat(64), 1),
        canonical.replace(",\"owner protocol review absent\"", ""),
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
fn case_input_index_rejects_missing_reordered_or_weakened_rows() {
    let canonical = String::from_utf8(CASE_INPUT_INDEX.to_vec()).expect("UTF-8 index");
    let reordered = canonical
        .replace("DS-01", "DS-TEMP")
        .replace("DS-02", "DS-01")
        .replace("DS-TEMP", "DS-02");
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
fn identity_inventory_is_case_major_candidate_minor_unique_and_still_zero_of_14() {
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
        assert_eq!(identities[0].case, CASES[case_index]);
        assert_eq!(identities[1].case, CASES[case_index]);
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
