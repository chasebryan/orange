use std::collections::BTreeMap;

use super::domain::{
    BUDGETS, CANDIDATES, CASES, CandidateId, CaseId, InputBindingId, NONCLAIMS,
    REQUIRED_CANDIDATE_CASES,
};
use super::packet::DraftPacket;
use super::sha256;
use super::strict_json::{self, JsonValue};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PlannedExecution {
    pub(crate) ordinal: usize,
    pub(crate) candidate: CandidateId,
    pub(crate) case: CaseId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReplayInputs<'a> {
    pub(crate) decision_suite: &'a [u8],
    pub(crate) legacy_v01_manifest: &'a [u8],
    pub(crate) claim_record_v01_schema: &'a [u8],
}

impl ReplayInputs<'_> {
    fn get(&self, id: InputBindingId) -> &[u8] {
        match id {
            InputBindingId::DecisionSuite => self.decision_suite,
            InputBindingId::LegacyV01Manifest => self.legacy_v01_manifest,
            InputBindingId::ClaimRecordV01Schema => self.claim_record_v01_schema,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReplayError {
    pub(crate) input: InputBindingId,
    pub(crate) path: &'static str,
    pub(crate) expected_sha256: &'static str,
    pub(crate) observed_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReplayPlan {
    packet_sha256: String,
    schedule: Vec<PlannedExecution>,
    render_repetitions: usize,
    workspace_replays: usize,
}

impl ReplayPlan {
    pub(crate) fn packet_sha256(&self) -> &str {
        &self.packet_sha256
    }

    pub(crate) fn schedule(&self) -> &[PlannedExecution] {
        &self.schedule
    }

    pub(crate) const fn completed_candidate_cases(&self) -> usize {
        0
    }

    pub(crate) const fn evidence_status(&self) -> &'static str {
        "none"
    }

    pub(crate) const fn selection(&self) -> Option<CandidateId> {
        None
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        strict_json::canonical_bytes(&self.value())
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical_bytes()))
    }

    fn value(&self) -> JsonValue {
        let schedule = self
            .schedule
            .iter()
            .map(|execution| {
                strict_json::object([
                    integer_entry("ordinal", execution.ordinal),
                    string_entry("candidate", execution.candidate.as_str()),
                    string_entry("case", execution.case.as_str()),
                ])
            })
            .collect();
        strict_json::object([
            string_entry("packet_sha256", &self.packet_sha256),
            (
                "required_candidate_cases".to_owned(),
                JsonValue::Integer(i64::try_from(REQUIRED_CANDIDATE_CASES).unwrap_or(i64::MAX)),
            ),
            (
                "completed_candidate_cases".to_owned(),
                JsonValue::Integer(0),
            ),
            string_entry("evidence_status", self.evidence_status()),
            ("selection".to_owned(), JsonValue::Null),
            (
                "render_repetitions".to_owned(),
                JsonValue::Integer(i64::try_from(self.render_repetitions).unwrap_or(i64::MAX)),
            ),
            (
                "workspace_replays".to_owned(),
                JsonValue::Integer(i64::try_from(self.workspace_replays).unwrap_or(i64::MAX)),
            ),
            ("schedule".to_owned(), JsonValue::Array(schedule)),
            ("nonclaims".to_owned(), strict_json::strings(NONCLAIMS)),
        ])
    }
}

pub(crate) fn prepare_replay(
    packet: &DraftPacket,
    inputs: &ReplayInputs<'_>,
) -> Result<ReplayPlan, ReplayError> {
    for input in [
        InputBindingId::DecisionSuite,
        InputBindingId::LegacyV01Manifest,
        InputBindingId::ClaimRecordV01Schema,
    ] {
        let binding = packet.input_binding(input);
        let observed_sha256 = sha256::hex(&sha256::digest(inputs.get(input)));
        if observed_sha256 != binding.sha256 {
            return Err(ReplayError {
                input,
                path: binding.path,
                expected_sha256: binding.sha256,
                observed_sha256,
            });
        }
    }
    let mut schedule = Vec::with_capacity(REQUIRED_CANDIDATE_CASES);
    for (case_index, case) in CASES.into_iter().enumerate() {
        for candidate_offset in 0..CANDIDATES.len() {
            let candidate_index = (case_index + candidate_offset) % CANDIDATES.len();
            schedule.push(PlannedExecution {
                ordinal: schedule.len() + 1,
                candidate: CANDIDATES[candidate_index],
                case,
            });
        }
    }
    Ok(ReplayPlan {
        packet_sha256: packet.digest_hex(),
        schedule,
        render_repetitions: BUDGETS.render_repetitions,
        workspace_replays: BUDGETS.workspace_replays,
    })
}

pub(crate) fn schedule_pair_counts(plan: &ReplayPlan) -> BTreeMap<(CandidateId, CaseId), usize> {
    let mut counts = BTreeMap::new();
    for execution in plan.schedule() {
        let count = counts
            .entry((execution.candidate, execution.case))
            .or_insert(0);
        *count += 1;
    }
    counts
}

fn string_entry(key: &str, value: &str) -> (String, JsonValue) {
    (key.to_owned(), JsonValue::String(value.to_owned()))
}

fn integer_entry(key: &str, value: usize) -> (String, JsonValue) {
    (
        key.to_owned(),
        JsonValue::Integer(i64::try_from(value).unwrap_or(i64::MAX)),
    )
}
