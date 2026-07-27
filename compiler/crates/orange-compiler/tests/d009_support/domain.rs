use std::fmt;

pub(crate) const REQUIRED_CANDIDATE_CASES: usize = 24;
pub(crate) const INPUT_BINDING_COUNT: usize = 2;
pub(crate) const SEMANTIC_BINDING_COUNT: usize = 5;
pub(crate) const HARD_GATE_COUNT: usize = 8;
pub(crate) const SEMANTIC_NORMALIZATION: &str = "markdown-prose-lines-exact-v1";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CandidateId {
    CheckedArtifact,
    KernelOnly,
    TrustedSolver,
}

impl CandidateId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CheckedArtifact => "SP-01",
            Self::KernelOnly => "SP-02",
            Self::TrustedSolver => "SP-03",
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::CheckedArtifact => "Checked-artifact portfolio",
            Self::KernelOnly => "Kernel-only reconstruction",
            Self::TrustedSolver => "Direct trusted-solver authority",
        }
    }
}

impl fmt::Display for CandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CANDIDATES: [CandidateId; 3] = [
    CandidateId::CheckedArtifact,
    CandidateId::KernelOnly,
    CandidateId::TrustedSolver,
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CaseId {
    Tc01,
    Tc02,
    Tc03,
    Tc04,
    Tc05,
    Tc06,
    Tc07,
    Tc08,
}

impl CaseId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Tc01 => "TC-01",
            Self::Tc02 => "TC-02",
            Self::Tc03 => "TC-03",
            Self::Tc04 => "TC-04",
            Self::Tc05 => "TC-05",
            Self::Tc06 => "TC-06",
            Self::Tc07 => "TC-07",
            Self::Tc08 => "TC-08",
        }
    }
}

impl fmt::Display for CaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CASES: [CaseId; 8] = [
    CaseId::Tc01,
    CaseId::Tc02,
    CaseId::Tc03,
    CaseId::Tc04,
    CaseId::Tc05,
    CaseId::Tc06,
    CaseId::Tc07,
    CaseId::Tc08,
];

pub(crate) const METRICS: [&str; 16] = [
    "M-01", "M-02", "M-03", "M-04", "M-05", "M-06", "M-07", "M-08", "M-09", "M-10", "M-11", "M-12",
    "M-13", "M-14", "M-15", "M-16",
];

pub(crate) const OWNER_SCOPES: [&str; 8] = [
    "SR-01", "SR-02", "SR-03", "SR-04", "SR-05", "SR-06", "SR-07", "SR-08",
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

pub(crate) const COMPARATIVE_LABELS: [&str; 5] = [
    "checked_artifact_better",
    "kernel_only_better",
    "trusted_solver_better",
    "practically_equivalent",
    "inconclusive",
];

pub(crate) const CONCLUSIONS: [&str; 5] = [
    "recommend_checked_artifact",
    "recommend_kernel_only",
    "recommend_trusted_solver",
    "tie",
    "inconclusive",
];

pub(crate) const PROTOCOL_GAPS: [&str; 11] = [
    "D-004 acceptance absent",
    "D-005 acceptance absent",
    "candidate-neutral shared inputs and all eight executable case inventories absent",
    "input-manifest digest unfrozen",
    "execution resource, timeout, host, environment, cache, and network contract unassigned",
    "solver, checker, proof-format, dependency-graph, acquisition, and D-018 admissions absent",
    "candidate adapters, runner, observer, and isolation backend absent",
    "versioned D-009 result and same-owner-replay schema absent",
    "D-006 and D-007 proof/checker interfaces unavailable for downstream conformance",
    "physical execution order, correction window, and materiality bands unassigned",
    "owner protocol review absent",
];

pub(crate) const NONCLAIMS: [&str; 13] = [
    "no D-004 acceptance inferred",
    "no D-005 acceptance inferred",
    "no D-006 or D-007 acceptance inferred",
    "no solver, proof assistant, certificate checker, adapter, runner, observer, or isolation dependency admitted, acquired, installed, or executed",
    "no candidate-neutral shared input or executable fixture present",
    "no D-009 evidence epoch frozen",
    "no candidate executed and no candidate result recorded",
    "no solver-trust candidate selected or recommended by the laboratory",
    "no proof, certificate, counterexample, theorem, claim, or cache result validated",
    "no solver added to or removed from the logical TCB",
    "no proof-bearing implementation or solver-backed search authorized",
    "no independent review claimed",
    "no roadmap gate or readiness movement",
];

pub(crate) const CASE_INPUT_NONCLAIMS: [&str; 6] = [
    "no shared candidate-neutral input packet present",
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
pub(crate) struct CandidateStateSpec {
    pub(crate) candidate: CandidateId,
}

pub(crate) const CANDIDATE_STATES: [CandidateStateSpec; 3] = [
    CandidateStateSpec {
        candidate: CandidateId::CheckedArtifact,
    },
    CandidateStateSpec {
        candidate: CandidateId::KernelOnly,
    },
    CandidateStateSpec {
        candidate: CandidateId::TrustedSolver,
    },
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InputBindingId {
    CaseInputIndex,
    SolverTrustSuite,
}

impl InputBindingId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::CaseInputIndex => "case_input_index",
            Self::SolverTrustSuite => "solver_trust_suite",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::CaseInputIndex => 0,
            Self::SolverTrustSuite => 1,
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
        path: "research/decisions/D-009/d009-v0.1-case-input-index.json",
        sha256: "c5298d625f5392de2774ffb861fe1dc1701b379ebd385cde0584a8cbcd249859",
    },
    InputBinding {
        id: InputBindingId::SolverTrustSuite,
        path: "docs/SOLVER_TRUST_DECISION_SUITE.md",
        sha256: "a26073e6431fb401af4aac6e57dcdfa76b27fe9451c26fb42595d7de14c2a35b",
    },
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum SemanticBindingId {
    DecisionRegisterDocument,
    DecisionRegisterD009,
    RoadmapDocument,
    RoadmapS4,
    SolverTrustSuite,
}

impl SemanticBindingId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DecisionRegisterDocument => "decision_register_document",
            Self::DecisionRegisterD009 => "decision_register_d009",
            Self::RoadmapDocument => "roadmap_document",
            Self::RoadmapS4 => "roadmap_s4",
            Self::SolverTrustSuite => "solver_trust_suite",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::DecisionRegisterDocument => 0,
            Self::DecisionRegisterD009 => 1,
            Self::RoadmapDocument => 2,
            Self::RoadmapS4 => 3,
            Self::SolverTrustSuite => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SemanticBinding {
    pub(crate) id: SemanticBindingId,
    pub(crate) path: &'static str,
    pub(crate) scope: &'static str,
    pub(crate) section_end_heading: Option<&'static str>,
    pub(crate) section_start_heading: Option<&'static str>,
    pub(crate) normalization: &'static str,
    pub(crate) normalized_sha256: &'static str,
}

pub(crate) const SEMANTIC_BINDINGS: [SemanticBinding; SEMANTIC_BINDING_COUNT] = [
    SemanticBinding {
        id: SemanticBindingId::DecisionRegisterDocument,
        path: "docs/DECISIONS.md",
        scope: "whole_document",
        section_end_heading: None,
        section_start_heading: None,
        normalization: SEMANTIC_NORMALIZATION,
        normalized_sha256: "c65d91e5566f72114a71739efa8ca300feea24f0f2312e902d06fd9172fa0628",
    },
    SemanticBinding {
        id: SemanticBindingId::DecisionRegisterD009,
        path: "docs/DECISIONS.md",
        scope: "markdown_exact_heading_range",
        section_end_heading: Some("## D-010 — Compiler strategy"),
        section_start_heading: Some("## D-009 — Solver trust"),
        normalization: SEMANTIC_NORMALIZATION,
        normalized_sha256: "bc4bf5fd534a61efdd62e16b57633bea1f5ad8f3224555f310911a3ab26bb41a",
    },
    SemanticBinding {
        id: SemanticBindingId::RoadmapDocument,
        path: "docs/ROADMAP.md",
        scope: "whole_document",
        section_end_heading: None,
        section_start_heading: None,
        normalization: SEMANTIC_NORMALIZATION,
        normalized_sha256: "85480fb2764ae7bb98acd08590bb49116f5bb8cf62bebdee509aadbf1e22e6ea",
    },
    SemanticBinding {
        id: SemanticBindingId::RoadmapS4,
        path: "docs/ROADMAP.md",
        scope: "markdown_exact_heading_range",
        section_end_heading: Some("### S5 — Compiler IRs and one output path"),
        section_start_heading: Some("### S4 — Proof and claim boundary"),
        normalization: SEMANTIC_NORMALIZATION,
        normalized_sha256: "ef9322f7c23467007a4c6d64411c79ba0f3dae856ed49602d9cecdfc9490b096",
    },
    SemanticBinding {
        id: SemanticBindingId::SolverTrustSuite,
        path: "docs/SOLVER_TRUST_DECISION_SUITE.md",
        scope: "whole_document",
        section_end_heading: None,
        section_start_heading: None,
        normalization: SEMANTIC_NORMALIZATION,
        normalized_sha256: "c2838efcc963de22141631d58fae4730c131d47d8ea2e79906cf66d0546032d0",
    },
];
