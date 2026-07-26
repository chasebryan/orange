//! Input-only substrate checks for the D-010 compiler-strategy decision suite.
//!
//! These checks bind the draft-unfrozen packet and eight-row missing-input
//! index, then enumerate 40 candidate-case identities in memory. They launch
//! no compiler or process, authorize no physical run order, freeze no evidence
//! epoch, and create no result, artifact, selection, claim, or compiler path.

#[path = "d010_support/domain.rs"]
mod domain;
#[path = "d010_support/packet.rs"]
mod packet;
#[path = "d010_support/runner.rs"]
mod runner;
#[path = "d005_support/sha256.rs"]
mod sha256;
#[path = "d005_support/strict_json.rs"]
mod strict_json;

use std::collections::BTreeSet;

use domain::{
    ATOMIC_OUTCOME_MEANINGS, ATOMIC_OUTCOMES, BUDGETS, CANDIDATES, CASES, COMPARATIVE_AXES,
    COMPARATIVE_LABELS, CONCLUSIONS, HARD_GATE_COUNT, HARD_GATE_STATE_PRECEDENCE, HARD_GATE_STATES,
    METRICS, OWNER_SCOPES, PROTOCOL_COUNTS, REQUIRED_CANDIDATE_CASES,
};
use packet::{
    CASE_INPUT_INDEX_CANONICAL_SHA256, PacketErrorKind, parse_case_input_index, parse_draft_packet,
};
use runner::{INPUT_BINDINGS, PLAN_NONCLAIMS, ReplayError, ReplayInputs};
use strict_json::{JsonErrorKind, JsonValue};

const CHECKED_IN_PACKET: &[u8] =
    include_bytes!("../../../../research/decisions/D-010/d010-v0.1-draft-packet.json");
const CASE_INPUT_INDEX: &[u8] =
    include_bytes!("../../../../research/decisions/D-010/d010-v0.1-case-input-index.json");
const COMPILER_STRATEGY_SUITE: &[u8] =
    include_bytes!("../../../../docs/COMPILER_STRATEGY_DECISION_SUITE.md");

const PACKET_CANONICAL_SHA256: &str =
    "076e911bb5f52ee048d7c854928becca4675c97a678c48a03ae5d40b71a67007";
const PACKET_RAW_SHA256: &str = "aec7514683746c4fdc3fb33f771e793146fcb258be3d4c9b8b9eadd507bb8d0e";
const INDEX_RAW_SHA256: &str = "e9f59e86dff6219474d244ff01a98c75b7b17c65f1f91506d483a57e95e33670";
const SUITE_RAW_SHA256: &str = "5d36f1faeda027b9784846af0aa742339c6b821f39b72a8ca067a90c41a46c73";

const PACKET_NONCLAIMS: [&str; 10] = [
    "D-003 acceptance grants no D-004, D-005, D-006, D-007, D-009, D-010, D-011, D-012, or D-013 acceptance",
    "no compiler, backend, proof assistant, solver, checker, assembler, linker, adapter, runner, observer, emulator, or isolation dependency admitted, acquired, installed, or executed",
    "no executable shared input, candidate mapping, result schema, or physical execution order exists",
    "no evidence epoch, candidate result, selection, or conclusion exists",
    "no source-to-IR, pass, certificate, theorem, leakage, target, ABI, object, wrapper, interoperability, or final-byte proposition validated",
    "no compiler strategy or output path selected, preferred, recommended by these inputs, implemented, or authorized for claim-bearing product work",
    "no claim frontier, logical TCB, target envelope, leakage model, or foreign boundary accepted or changed",
    "no C11 or LLVM artifact inherits a native assurance claim, and neither may borrow evidence from the other",
    "no independent review, reproduction, audit, certification, or external validation claimed",
    "no roadmap gate, S5 closure, release authority, compiler capability, or readiness credit",
];

const PROTOCOL_GAPS: [&str; 13] = [
    "D-004 acceptance absent",
    "D-005 acceptance absent",
    "D-006 acceptance absent",
    "D-009 acceptance absent",
    "candidate-neutral shared inputs and all eight executable case inventories absent",
    "input-manifest digest unfrozen",
    "execution resource, timeout, host, environment, cache, and network contract unassigned",
    "compiler, backend, toolchain, proof, checker, acquisition, and D-018 admissions absent",
    "candidate adapters, runner, observer, emulator, and isolation backend absent",
    "versioned D-010 result and same-owner-replay schema absent",
    "suite-only downstream model fixtures and reference endpoints absent",
    "physical execution order, correction window, and materiality bands unassigned",
    "owner protocol review absent",
];

const CASE_INPUT_NONCLAIMS: [&str; 6] = [
    "no shared candidate-neutral input packet present",
    "no candidate mapping present",
    "no executable fixture present",
    "no case coverage established",
    "no candidate observation or evidence recorded",
    "no capability or readiness credit",
];

fn checked_in_replay_inputs() -> ReplayInputs<'static> {
    ReplayInputs::new([CASE_INPUT_INDEX, COMPILER_STRATEGY_SUITE])
}

fn canonical_file_bytes(value: &JsonValue) -> Vec<u8> {
    let mut canonical = strict_json::canonical_bytes(value);
    canonical.push(b'\n');
    canonical
}

fn strings(value: &JsonValue) -> Vec<&str> {
    value
        .as_array()
        .expect("string array")
        .iter()
        .map(|entry| entry.as_str().expect("string entry"))
        .collect()
}

#[test]
fn d010_pre_epoch_domain_is_exact_symmetric_and_non_executing() {
    assert_eq!(
        CANDIDATES.map(|candidate| (candidate.as_str(), candidate.name())),
        [
            ("CP-01", "Theorem/certificate hybrid direct-native path"),
            ("CP-02", "Mechanized proof-per-pass direct-native path"),
            ("CP-03", "Versioned Jasmin backend boundary"),
            ("CP-04", "Portable C11 interoperability boundary"),
            ("CP-05", "Versioned LLVM IR interoperability boundary"),
        ]
    );
    assert_eq!(
        CASES.map(|case| case.as_str()),
        [
            "CC-01", "CC-02", "CC-03", "CC-04", "CC-05", "CC-06", "CC-07", "CC-08",
        ]
    );
    assert_eq!(CANDIDATES.len() * CASES.len(), REQUIRED_CANDIDATE_CASES);
    assert_eq!(REQUIRED_CANDIDATE_CASES, 40);
    assert_eq!(
        METRICS,
        [
            "M-01", "M-02", "M-03", "M-04", "M-05", "M-06", "M-07", "M-08", "M-09", "M-10", "M-11",
            "M-12", "M-13", "M-14", "M-15", "M-16", "M-17", "M-18", "M-19",
        ]
    );
    assert_eq!(
        COMPARATIVE_AXES,
        [
            "AX-01", "AX-02", "AX-03", "AX-04", "AX-05", "AX-06", "AX-07", "AX-08", "AX-09",
        ]
    );
    assert_eq!(
        OWNER_SCOPES,
        [
            "CR-01", "CR-02", "CR-03", "CR-04", "CR-05", "CR-06", "CR-07", "CR-08", "CR-09",
            "CR-10", "CR-11",
        ]
    );
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
    assert_eq!(
        COMPARATIVE_LABELS,
        [
            "hybrid_direct_native_better",
            "proof_per_pass_direct_native_better",
            "jasmin_backend_better",
            "portable_c11_better",
            "llvm_ir_better",
            "practically_equivalent",
            "inconclusive",
        ]
    );
    assert_eq!(
        CONCLUSIONS,
        [
            "recommend_hybrid_direct_native",
            "recommend_proof_per_pass_direct_native",
            "recommend_jasmin_backend",
            "recommend_portable_c11",
            "recommend_llvm_ir",
            "tie",
            "inconclusive",
        ]
    );
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
    assert_eq!(INPUT_BINDINGS.len(), 2);
    assert_eq!(PLAN_NONCLAIMS.len(), 8);
}

#[test]
fn case_input_index_is_canonical_digest_bound_and_zero_fixture() {
    let value = strict_json::parse(CASE_INPUT_INDEX).expect("checked-in index JSON");
    assert_eq!(CASE_INPUT_INDEX, canonical_file_bytes(&value));
    let index = parse_case_input_index(CASE_INPUT_INDEX).expect("checked-in case-input index");
    assert_eq!(
        index.canonical_bytes(),
        strict_json::canonical_bytes(&value)
    );
    assert_eq!(index.digest_hex(), CASE_INPUT_INDEX_CANONICAL_SHA256);
    assert_eq!(
        sha256::hex(&sha256::digest(CASE_INPUT_INDEX)),
        INDEX_RAW_SHA256
    );

    let root = index.value().as_object().expect("index root");
    assert_eq!(
        root.get("schema_version").and_then(JsonValue::as_str),
        Some("d010-pre-epoch-case-input-index-v0.1")
    );
    assert_eq!(
        root.get("suite_version").and_then(JsonValue::as_str),
        Some("d010-v0.1-draft")
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
    assert_eq!(
        root.get("evidence_status").and_then(JsonValue::as_str),
        Some("none")
    );
    assert_eq!(
        strings(root.get("nonclaims").expect("index nonclaims")),
        CASE_INPUT_NONCLAIMS
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
fn draft_packet_is_canonical_content_addressed_and_zero_of_40() {
    let value = strict_json::parse(CHECKED_IN_PACKET).expect("checked-in packet JSON");
    assert_eq!(CHECKED_IN_PACKET, canonical_file_bytes(&value));
    let packet = parse_draft_packet(CHECKED_IN_PACKET).expect("checked-in D-010 packet");
    assert_eq!(
        packet.canonical_bytes(),
        strict_json::canonical_bytes(&value)
    );
    assert_eq!(packet.digest_hex(), PACKET_CANONICAL_SHA256);
    assert_eq!(
        sha256::hex(&sha256::digest(CHECKED_IN_PACKET)),
        PACKET_RAW_SHA256
    );
    assert_eq!(
        sha256::hex(&sha256::digest(COMPILER_STRATEGY_SUITE)),
        SUITE_RAW_SHA256
    );

    let root = packet.value().as_object().expect("packet root");
    assert_eq!(
        root.get("schema_version").and_then(JsonValue::as_str),
        Some("d010-pre-epoch-packet-v0.1")
    );
    assert_eq!(
        root.get("suite_version").and_then(JsonValue::as_str),
        Some("d010-v0.1-draft")
    );
    assert_eq!(
        root.get("status").and_then(JsonValue::as_str),
        Some("draft_unfrozen")
    );
    assert_eq!(
        root.get("epoch_status").and_then(JsonValue::as_str),
        Some("unfrozen")
    );
    assert_eq!(root.get("epoch"), Some(&JsonValue::Null));
    assert_eq!(root.get("physical_execution_order"), Some(&JsonValue::Null));
    assert_eq!(root.get("selection"), Some(&JsonValue::Null));
    assert_eq!(root.get("conclusion"), Some(&JsonValue::Null));
    assert_eq!(
        root.get("owner_protocol_review")
            .and_then(JsonValue::as_str),
        Some("none")
    );
    assert_eq!(
        root.get("independent_review_status")
            .and_then(JsonValue::as_str),
        Some("unavailable")
    );
    assert_eq!(
        root.get("case_input_index_sha256")
            .and_then(JsonValue::as_str),
        Some(CASE_INPUT_INDEX_CANONICAL_SHA256)
    );

    assert_eq!(
        strings(root.get("atomic_outcomes").expect("atomic outcomes")),
        ATOMIC_OUTCOMES
    );
    let meanings = root
        .get("atomic_outcome_meanings")
        .and_then(JsonValue::as_object)
        .expect("atomic outcome meanings");
    for (outcome, meaning) in ATOMIC_OUTCOME_MEANINGS {
        assert_eq!(
            meanings.get(outcome).and_then(JsonValue::as_str),
            Some(meaning)
        );
    }
    assert_eq!(
        strings(root.get("hard_gate_states").expect("hard-gate states")),
        HARD_GATE_STATES
    );
    assert_eq!(
        strings(
            root.get("hard_gate_state_precedence")
                .expect("hard-gate precedence")
        ),
        HARD_GATE_STATE_PRECEDENCE
    );
    assert_eq!(
        strings(root.get("comparative_axes").expect("comparative axes")),
        COMPARATIVE_AXES
    );
    assert_eq!(
        strings(root.get("comparative_labels").expect("comparative labels")),
        COMPARATIVE_LABELS
    );
    assert_eq!(
        strings(root.get("conclusions").expect("conclusions")),
        CONCLUSIONS
    );
    assert_eq!(strings(root.get("metrics").expect("metrics")), METRICS);
    assert_eq!(
        strings(root.get("owner_scopes").expect("owner scopes")),
        OWNER_SCOPES
    );
    assert_eq!(
        strings(root.get("protocol_gaps").expect("protocol gaps")),
        PROTOCOL_GAPS
    );
    assert_eq!(
        strings(root.get("nonclaims").expect("packet nonclaims")),
        PACKET_NONCLAIMS
    );

    let dependencies = root
        .get("dependency_acceptance")
        .and_then(JsonValue::as_object)
        .expect("dependency acceptance");
    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies.get("D-003"), Some(&JsonValue::Bool(true)));
    for dependency in ["D-004", "D-005", "D-006", "D-009"] {
        assert_eq!(dependencies.get(dependency), Some(&JsonValue::Bool(false)));
    }

    let execution = root
        .get("execution")
        .and_then(JsonValue::as_object)
        .expect("execution state");
    assert_eq!(
        execution
            .get("required_candidate_cases")
            .and_then(JsonValue::as_integer),
        Some(40)
    );
    for field in [
        "completed_candidate_cases",
        "complete_candidates",
        "complete_cross_candidate_cases",
    ] {
        assert_eq!(
            execution.get(field).and_then(JsonValue::as_integer),
            Some(0)
        );
    }
    assert_eq!(
        execution.get("evidence_status").and_then(JsonValue::as_str),
        Some("none")
    );

    let resources = root
        .get("execution_resource_state")
        .and_then(JsonValue::as_object)
        .expect("execution resource state");
    for field in [
        "case_output_bytes",
        "case_peak_memory_bytes",
        "case_temp_storage_bytes",
        "case_wall_seconds",
    ] {
        assert_eq!(resources.get(field), Some(&JsonValue::Null));
    }
    for field in [
        "contract_status",
        "host_matrix_status",
        "timeout_semantics_status",
    ] {
        assert_eq!(
            resources.get(field).and_then(JsonValue::as_str),
            Some("unassigned_freeze_blocker")
        );
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
        for field in [
            "adapter_status",
            "dependency_admission_status",
            "implementation_status",
        ] {
            assert_eq!(state.get(field).and_then(JsonValue::as_str), Some("absent"));
        }
        assert_eq!(
            state.get("execution_status").and_then(JsonValue::as_str),
            Some("not_performed")
        );
    }

    let bindings = root
        .get("input_bindings")
        .and_then(JsonValue::as_object)
        .expect("input bindings");
    for binding in INPUT_BINDINGS {
        let actual = bindings
            .get(binding.id.as_str())
            .and_then(JsonValue::as_object)
            .expect("input binding");
        assert_eq!(
            actual.get("path").and_then(JsonValue::as_str),
            Some(binding.path)
        );
        assert_eq!(
            actual.get("sha256").and_then(JsonValue::as_str),
            Some(binding.sha256)
        );
    }
}

#[test]
fn packet_and_index_reject_malformed_mutated_or_noncanonical_json() {
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
    for mutation in [
        packet.replacen('{', "{\"unknown\":null,", 1),
        packet.replace(",\"selection\":null", ""),
        packet.replacen("\"D-003\":true", "\"D-003\":false", 1),
        packet.replacen("\"epoch\":null", "\"epoch\":\"0001\"", 1),
        packet.replacen(
            "\"completed_candidate_cases\":0",
            "\"completed_candidate_cases\":1",
            1,
        ),
        packet.replacen("\"selection\":null", "\"selection\":\"CP-01\"", 1),
        packet.replacen(
            "\"conclusion\":null",
            "\"conclusion\":\"recommend_hybrid_direct_native\"",
            1,
        ),
        packet.replacen(
            "\"hard_gate_state_precedence\":[\"unsupported\",\"fail\",\"unresolved\",\"pass\"]",
            "\"hard_gate_state_precedence\":[\"fail\",\"unsupported\",\"unresolved\",\"pass\"]",
            1,
        ),
    ] {
        assert!(
            parse_draft_packet(mutation.as_bytes()).is_err(),
            "packet mutation must be rejected"
        );
    }
    for noncanonical in [
        strict_json::canonical_bytes(&strict_json::parse(CHECKED_IN_PACKET).expect("packet value")),
        [b" ".as_slice(), CHECKED_IN_PACKET].concat(),
    ] {
        assert_eq!(
            parse_draft_packet(&noncanonical)
                .expect_err("noncanonical packet")
                .kind,
            PacketErrorKind::NonCanonicalEncoding
        );
    }

    let index_duplicate = index.replacen('{', "{\"status\":\"draft_unreviewed\",", 1);
    assert_eq!(
        parse_case_input_index(index_duplicate.as_bytes())
            .expect_err("duplicate index field")
            .kind,
        PacketErrorKind::Json(JsonErrorKind::DuplicateKey)
    );
    for mutation in [
        index.replacen("\"freeze_blocker\":true", "\"freeze_blocker\":false", 1),
        index.replacen(
            "\"executable_fixture_count\":0",
            "\"executable_fixture_count\":1",
            1,
        ),
        index.replacen(
            "\"shared_inputs_status\":\"absent\"",
            "\"shared_inputs_status\":\"present\"",
            1,
        ),
        index.replacen("\"case\":\"CC-01\"", "\"case\":\"CC-02\"", 1),
    ] {
        assert!(
            parse_case_input_index(mutation.as_bytes()).is_err(),
            "index mutation must be rejected"
        );
    }
    let index_value = strict_json::parse(CASE_INPUT_INDEX).expect("index value");
    assert_eq!(
        parse_case_input_index(&strict_json::canonical_bytes(&index_value))
            .expect_err("index without terminal LF")
            .kind,
        PacketErrorKind::NonCanonicalEncoding
    );
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
        let error = runner::prepare_identity_plan(&packet, &drifted).expect_err("raw input drift");
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
fn packet_and_index_reject_attempted_self_rebinding() {
    let packet = String::from_utf8(CHECKED_IN_PACKET.to_vec()).expect("UTF-8 packet");
    let index = String::from_utf8(CASE_INPUT_INDEX.to_vec()).expect("UTF-8 index");
    let drifted_index = index.replacen(
        "\"status\":\"draft_unreviewed\"",
        "\"status\":\"reviewed\"",
        1,
    );
    let drifted_value = strict_json::parse(drifted_index.as_bytes()).expect("well-formed drift");
    let drifted_canonical = strict_json::canonical_bytes(&drifted_value);
    let drifted_canonical_sha256 = sha256::hex(&sha256::digest(&drifted_canonical));
    let drifted_file = canonical_file_bytes(&drifted_value);
    let drifted_raw_sha256 = sha256::hex(&sha256::digest(&drifted_file));
    let rebound_packet = packet
        .replace(CASE_INPUT_INDEX_CANONICAL_SHA256, &drifted_canonical_sha256)
        .replace(INDEX_RAW_SHA256, &drifted_raw_sha256);

    assert!(
        parse_draft_packet(rebound_packet.as_bytes()).is_err(),
        "packet identity must reject self-rebinding"
    );
    assert!(
        parse_case_input_index(&drifted_file).is_err(),
        "index identity must reject self-rebinding"
    );
}

#[test]
fn identity_inventory_is_case_major_candidate_minor_unique_and_still_zero_of_40() {
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
    for nonclaim in PLAN_NONCLAIMS {
        assert!(plan_text.contains(nonclaim));
    }
}

#[test]
fn identity_planner_exposes_no_execution_or_evidence_api() {
    let source = include_str!("d010_support/runner.rs");
    for forbidden in [
        "std::process",
        "Command::new",
        "process::Command",
        "TcpStream",
        "UdpSocket",
        "File::create",
        "OpenOptions",
        "completed_candidate_cases: 1",
        "evidence_status: \"complete\"",
    ] {
        assert!(
            !source.contains(forbidden),
            "input-only runner must not contain {forbidden}"
        );
    }
}
