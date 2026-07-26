use std::collections::BTreeMap;

use super::domain::{CANDIDATES, CASES, CandidateId, CaseId, REQUIRED_CANDIDATE_CASES};
use super::packet::{
    BoundJson, CASE_INPUT_INDEX_CANONICAL_SHA256, PacketErrorKind, parse_case_input_index,
};
use super::sha256;
use super::strict_json::{self, JsonValue};

pub(crate) const INPUT_BINDING_COUNT: usize = 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InputBindingId {
    CaseInputIndex,
    CompilerStrategySuite,
}

impl InputBindingId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CaseInputIndex => "case_input_index",
            Self::CompilerStrategySuite => "compiler_strategy_suite",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::CaseInputIndex => 0,
            Self::CompilerStrategySuite => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InputBinding {
    pub(crate) id: InputBindingId,
    pub(crate) path: &'static str,
    pub(crate) sha256: &'static str,
}

pub(crate) const INPUT_BINDINGS: [InputBinding; INPUT_BINDING_COUNT] = [
    InputBinding {
        id: InputBindingId::CaseInputIndex,
        path: "research/decisions/D-010/d010-v0.1-case-input-index.json",
        sha256: "e9f59e86dff6219474d244ff01a98c75b7b17c65f1f91506d483a57e95e33670",
    },
    InputBinding {
        id: InputBindingId::CompilerStrategySuite,
        path: "docs/COMPILER_STRATEGY_DECISION_SUITE.md",
        sha256: "5d36f1faeda027b9784846af0aa742339c6b821f39b72a8ca067a90c41a46c73",
    },
];

pub(crate) const PLAN_NONCLAIMS: [&str; 8] = [
    "identity inventory is not a physical execution order",
    "no candidate adapter or subprocess invoked",
    "no compiler, backend, checker, assembler, linker, or tool executed",
    "no candidate-case execution recorded",
    "no evidence epoch or candidate result created",
    "no compiler strategy selected or recommended",
    "no product compiler or claim-bearing path authorized",
    "no roadmap gate or readiness movement",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PlannedIdentity {
    pub(crate) ordinal: usize,
    pub(crate) candidate: CandidateId,
    pub(crate) case: CaseId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ReplayInputs<'a> {
    bytes: [&'a [u8]; INPUT_BINDING_COUNT],
}

impl<'a> ReplayInputs<'a> {
    pub(crate) const fn new(bytes: [&'a [u8]; INPUT_BINDING_COUNT]) -> Self {
        Self { bytes }
    }

    pub(crate) fn get(&self, id: InputBindingId) -> &'a [u8] {
        self.bytes[id.index()]
    }

    pub(crate) fn with_replacement(self, id: InputBindingId, bytes: &'a [u8]) -> Self {
        let mut replaced = self;
        replaced.bytes[id.index()] = bytes;
        replaced
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ReplayError {
    InputDigest {
        input: InputBindingId,
        path: &'static str,
        expected_sha256: &'static str,
        observed_sha256: String,
    },
    CaseInputIndex(PacketErrorKind),
    PacketCaseInputIndexBinding {
        expected_sha256: String,
        observed_sha256: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IdentityPlan {
    packet_sha256: String,
    case_input_index_sha256: String,
    identities: Vec<PlannedIdentity>,
}

impl IdentityPlan {
    pub(crate) fn packet_sha256(&self) -> &str {
        &self.packet_sha256
    }

    pub(crate) fn case_input_index_sha256(&self) -> &str {
        &self.case_input_index_sha256
    }

    pub(crate) fn identities(&self) -> &[PlannedIdentity] {
        &self.identities
    }

    pub(crate) const fn completed_candidate_cases(&self) -> usize {
        0
    }

    pub(crate) const fn complete_candidates(&self) -> usize {
        0
    }

    pub(crate) const fn complete_cross_candidate_cases(&self) -> usize {
        0
    }

    pub(crate) const fn evidence_status(&self) -> &'static str {
        "none"
    }

    pub(crate) const fn physical_execution_order(&self) -> Option<()> {
        None
    }

    pub(crate) const fn selection(&self) -> Option<CandidateId> {
        None
    }

    pub(crate) const fn conclusion(&self) -> Option<&'static str> {
        None
    }

    pub(crate) fn canonical_bytes(&self) -> Vec<u8> {
        strict_json::canonical_bytes(&self.value())
    }

    pub(crate) fn digest_hex(&self) -> String {
        sha256::hex(&sha256::digest(&self.canonical_bytes()))
    }

    fn value(&self) -> JsonValue {
        let identities = self
            .identities
            .iter()
            .map(|identity| {
                strict_json::object([
                    usize_entry("ordinal", identity.ordinal),
                    string_entry("candidate", identity.candidate.as_str()),
                    string_entry("case", identity.case.as_str()),
                ])
            })
            .collect();
        strict_json::object([
            string_entry("packet_sha256", &self.packet_sha256),
            string_entry("case_input_index_sha256", &self.case_input_index_sha256),
            usize_entry("required_candidate_cases", REQUIRED_CANDIDATE_CASES),
            usize_entry("completed_candidate_cases", 0),
            usize_entry("complete_candidates", 0),
            usize_entry("complete_cross_candidate_cases", 0),
            string_entry("evidence_status", self.evidence_status()),
            ("physical_execution_order".to_owned(), JsonValue::Null),
            ("selection".to_owned(), JsonValue::Null),
            ("conclusion".to_owned(), JsonValue::Null),
            (
                "identity_inventory".to_owned(),
                JsonValue::Array(identities),
            ),
            ("nonclaims".to_owned(), strict_json::strings(PLAN_NONCLAIMS)),
        ])
    }
}

pub(crate) fn prepare_identity_plan(
    packet: &BoundJson,
    inputs: &ReplayInputs<'_>,
) -> Result<IdentityPlan, ReplayError> {
    for binding in INPUT_BINDINGS {
        let observed_sha256 = sha256::hex(&sha256::digest(inputs.get(binding.id)));
        if observed_sha256 != binding.sha256 {
            return Err(ReplayError::InputDigest {
                input: binding.id,
                path: binding.path,
                expected_sha256: binding.sha256,
                observed_sha256,
            });
        }
    }

    let index = parse_case_input_index(inputs.get(InputBindingId::CaseInputIndex))
        .map_err(|error| ReplayError::CaseInputIndex(error.kind))?;
    let observed_index_sha256 = index.digest_hex();
    let packet_index_sha256 = packet
        .value()
        .as_object()
        .and_then(|root| root.get("case_input_index_sha256"))
        .and_then(JsonValue::as_str)
        .unwrap_or("");
    if packet_index_sha256 != observed_index_sha256
        || observed_index_sha256 != CASE_INPUT_INDEX_CANONICAL_SHA256
    {
        return Err(ReplayError::PacketCaseInputIndexBinding {
            expected_sha256: packet_index_sha256.to_owned(),
            observed_sha256: observed_index_sha256,
        });
    }

    let mut identities = Vec::with_capacity(REQUIRED_CANDIDATE_CASES);
    for case in CASES {
        for candidate in CANDIDATES {
            identities.push(PlannedIdentity {
                ordinal: identities.len() + 1,
                candidate,
                case,
            });
        }
    }
    Ok(IdentityPlan {
        packet_sha256: packet.digest_hex(),
        case_input_index_sha256: index.digest_hex(),
        identities,
    })
}

pub(crate) fn identity_pair_counts(plan: &IdentityPlan) -> BTreeMap<(CandidateId, CaseId), usize> {
    let mut counts = BTreeMap::new();
    for identity in plan.identities() {
        let count = counts
            .entry((identity.candidate, identity.case))
            .or_insert(0);
        *count += 1;
    }
    counts
}

fn string_entry(key: &str, value: &str) -> (String, JsonValue) {
    (key.to_owned(), JsonValue::String(value.to_owned()))
}

fn usize_entry(key: &str, value: usize) -> (String, JsonValue) {
    (
        key.to_owned(),
        JsonValue::Integer(i64::try_from(value).unwrap_or(i64::MAX)),
    )
}
