use std::fmt;

pub(crate) const REQUIRED_CANDIDATE_CASES: usize = 40;
pub(crate) const HARD_GATE_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CandidateId {
    HybridDirectNative,
    ProofPerPassDirectNative,
    JasminBackend,
    PortableC11,
    LlvmIr,
}

impl CandidateId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::HybridDirectNative => "CP-01",
            Self::ProofPerPassDirectNative => "CP-02",
            Self::JasminBackend => "CP-03",
            Self::PortableC11 => "CP-04",
            Self::LlvmIr => "CP-05",
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::HybridDirectNative => "Theorem/certificate hybrid direct-native path",
            Self::ProofPerPassDirectNative => "Mechanized proof-per-pass direct-native path",
            Self::JasminBackend => "Versioned Jasmin backend boundary",
            Self::PortableC11 => "Portable C11 interoperability boundary",
            Self::LlvmIr => "Versioned LLVM IR interoperability boundary",
        }
    }
}

impl fmt::Display for CandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CANDIDATES: [CandidateId; 5] = [
    CandidateId::HybridDirectNative,
    CandidateId::ProofPerPassDirectNative,
    CandidateId::JasminBackend,
    CandidateId::PortableC11,
    CandidateId::LlvmIr,
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CaseId {
    Cc01,
    Cc02,
    Cc03,
    Cc04,
    Cc05,
    Cc06,
    Cc07,
    Cc08,
}

impl CaseId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Cc01 => "CC-01",
            Self::Cc02 => "CC-02",
            Self::Cc03 => "CC-03",
            Self::Cc04 => "CC-04",
            Self::Cc05 => "CC-05",
            Self::Cc06 => "CC-06",
            Self::Cc07 => "CC-07",
            Self::Cc08 => "CC-08",
        }
    }
}

impl fmt::Display for CaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CASES: [CaseId; 8] = [
    CaseId::Cc01,
    CaseId::Cc02,
    CaseId::Cc03,
    CaseId::Cc04,
    CaseId::Cc05,
    CaseId::Cc06,
    CaseId::Cc07,
    CaseId::Cc08,
];

pub(crate) const METRICS: [&str; 19] = [
    "M-01", "M-02", "M-03", "M-04", "M-05", "M-06", "M-07", "M-08", "M-09", "M-10", "M-11", "M-12",
    "M-13", "M-14", "M-15", "M-16", "M-17", "M-18", "M-19",
];

pub(crate) const COMPARATIVE_AXES: [&str; 9] = [
    "AX-01", "AX-02", "AX-03", "AX-04", "AX-05", "AX-06", "AX-07", "AX-08", "AX-09",
];

pub(crate) const OWNER_SCOPES: [&str; 11] = [
    "CR-01", "CR-02", "CR-03", "CR-04", "CR-05", "CR-06", "CR-07", "CR-08", "CR-09", "CR-10",
    "CR-11",
];

pub(crate) const ATOMIC_OUTCOMES: [&str; 4] =
    ["satisfied", "not_satisfied", "unresolved", "unsupported"];

pub(crate) const ATOMIC_OUTCOME_MEANINGS: [(&str, &str); 4] = [
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
];

pub(crate) const HARD_GATE_STATES: [&str; 4] = ["pass", "fail", "unresolved", "unsupported"];
pub(crate) const HARD_GATE_STATE_PRECEDENCE: [&str; 4] =
    ["unsupported", "fail", "unresolved", "pass"];

pub(crate) const COMPARATIVE_LABELS: [&str; 7] = [
    "hybrid_direct_native_better",
    "proof_per_pass_direct_native_better",
    "jasmin_backend_better",
    "portable_c11_better",
    "llvm_ir_better",
    "practically_equivalent",
    "inconclusive",
];

pub(crate) const CONCLUSIONS: [&str; 7] = [
    "recommend_hybrid_direct_native",
    "recommend_proof_per_pass_direct_native",
    "recommend_jasmin_backend",
    "recommend_portable_c11",
    "recommend_llvm_ir",
    "tie",
    "inconclusive",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BudgetSpec {
    pub(crate) max_packet_bytes: usize,
    pub(crate) max_json_depth: usize,
    pub(crate) max_json_nodes: usize,
    pub(crate) max_string_bytes: usize,
}

pub(crate) const BUDGETS: BudgetSpec = BudgetSpec {
    max_packet_bytes: 256 * 1024,
    max_json_depth: 32,
    max_json_nodes: 16_384,
    max_string_bytes: 16_384,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProtocolCountSpec {
    pub(crate) cold_bootstrap_runs: usize,
    pub(crate) deterministic_profile_runs: usize,
    pub(crate) maximum_same_owner_reproducibility_level: usize,
    pub(crate) owner_workspaces: usize,
    pub(crate) timed_replays_per_case: usize,
    pub(crate) unmeasured_warmups: usize,
}

pub(crate) const PROTOCOL_COUNTS: ProtocolCountSpec = ProtocolCountSpec {
    cold_bootstrap_runs: 5,
    deterministic_profile_runs: 3,
    maximum_same_owner_reproducibility_level: 2,
    owner_workspaces: 2,
    timed_replays_per_case: 30,
    unmeasured_warmups: 1,
};
