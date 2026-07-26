use std::fmt;

pub(crate) const REQUIRED_CANDIDATE_CASES: usize = 14;
pub(crate) const INPUT_BINDING_COUNT: usize = 2;
pub(crate) const HARD_GATE_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CandidateId {
    Rocq,
    Lean4,
}

impl CandidateId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Rocq => "C-01",
            Self::Lean4 => "C-02",
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Rocq => "Rocq",
            Self::Lean4 => "Lean 4",
        }
    }
}

impl fmt::Display for CandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CANDIDATES: [CandidateId; 2] = [CandidateId::Rocq, CandidateId::Lean4];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CaseId {
    Ds01,
    Ds02,
    Ds03,
    Ds04,
    Ds05,
    Ds06,
    Ds07,
}

impl CaseId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ds01 => "DS-01",
            Self::Ds02 => "DS-02",
            Self::Ds03 => "DS-03",
            Self::Ds04 => "DS-04",
            Self::Ds05 => "DS-05",
            Self::Ds06 => "DS-06",
            Self::Ds07 => "DS-07",
        }
    }
}

impl fmt::Display for CaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CASES: [CaseId; 7] = [
    CaseId::Ds01,
    CaseId::Ds02,
    CaseId::Ds03,
    CaseId::Ds04,
    CaseId::Ds05,
    CaseId::Ds06,
    CaseId::Ds07,
];

pub(crate) const METRICS: [&str; 18] = [
    "M-01", "M-02", "M-03", "M-04", "M-05", "M-06", "M-07", "M-08", "M-09", "M-10", "M-11", "M-12",
    "M-13", "M-14", "M-15", "M-16", "M-17", "M-18",
];

pub(crate) const OWNER_SCOPES: [&str; 9] = [
    "R-01", "R-02", "R-03", "R-04", "R-05", "R-06", "R-07", "R-08", "R-09",
];

pub(crate) const HARD_GATE_STATES: [&str; 4] = ["pass", "fail", "unresolved", "unsupported"];

pub(crate) const COMPARATIVE_LABELS: [&str; 4] = [
    "rocq_better",
    "lean_better",
    "practically_equivalent",
    "inconclusive",
];

pub(crate) const CONCLUSIONS: [&str; 4] =
    ["recommend_rocq", "recommend_lean", "tie", "inconclusive"];

pub(crate) const PROTOCOL_GAPS: [&str; 10] = [
    "D-004 acceptance absent",
    "D-005 acceptance absent",
    "foundation-neutral shared inputs and all seven executable case inventories absent",
    "input-manifest digest unfrozen",
    "execution resource, timeout, host, environment, cache, and network contract unassigned",
    "candidate tool versions, dependency graphs, acquisitions, and D-018 admissions absent",
    "candidate adapters, runner, observer, and isolation backend absent",
    "versioned D-006 result and same-owner-replay schema absent",
    "physical execution order, correction window, and materiality bands unassigned",
    "owner protocol review absent",
];

pub(crate) const NONCLAIMS: [&str; 11] = [
    "no D-004 acceptance inferred",
    "no D-005 acceptance inferred",
    "no proof toolchain dependency admitted, acquired, or installed",
    "no foundation-neutral shared input or executable fixture present",
    "no candidate adapter, runner, observer, or isolation backend present",
    "no D-006 evidence epoch frozen",
    "no candidate executed and no candidate result recorded",
    "no proof foundation selected or recommended",
    "no proof-bearing implementation authorized",
    "no independent review claimed",
    "no roadmap gate or readiness movement",
];

pub(crate) const CASE_INPUT_NONCLAIMS: [&str; 6] = [
    "no shared foundation-neutral input packet present",
    "no candidate mapping present",
    "no executable fixture present",
    "no case coverage established",
    "no candidate observation or evidence recorded",
    "no capability or readiness credit",
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ToolStateSpec {
    pub(crate) candidate: CandidateId,
    pub(crate) name: &'static str,
}

pub(crate) const TOOL_STATES: [ToolStateSpec; 2] = [
    ToolStateSpec {
        candidate: CandidateId::Rocq,
        name: "Rocq",
    },
    ToolStateSpec {
        candidate: CandidateId::Lean4,
        name: "Lean 4",
    },
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InputBindingId {
    CaseInputIndex,
    ProofFoundationSuite,
}

impl InputBindingId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CaseInputIndex => "case_input_index",
            Self::ProofFoundationSuite => "proof_foundation_suite",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::CaseInputIndex => 0,
            Self::ProofFoundationSuite => 1,
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
        path: "research/decisions/D-006/d006-v0.2-case-input-index.json",
        sha256: "1aec6a731bef0620c8500120ec8385d584f99a528b4a03c014e8516c55cc8136",
    },
    InputBinding {
        id: InputBindingId::ProofFoundationSuite,
        path: "docs/PROOF_FOUNDATION_DECISION_SUITE.md",
        sha256: "6b1aa32784dd31d40bdaca4c6f3b62b8721a909ab3415051aa5a8e7994f0254b",
    },
];
