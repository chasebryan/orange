use std::fmt;

pub(crate) const REQUIRED_CANDIDATE_CASES: usize = 32;
pub(crate) const REQUIRED_RENDER_REPETITIONS: usize = 3;
pub(crate) const REQUIRED_WORKSPACE_REPLAYS: usize = 2;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CandidateId {
    Am01,
    Am02,
    Am03,
    Am04,
}

impl CandidateId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Am01 => "AM-01",
            Self::Am02 => "AM-02",
            Self::Am03 => "AM-03",
            Self::Am04 => "AM-04",
        }
    }
}

impl fmt::Display for CandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CANDIDATES: [CandidateId; 4] = [
    CandidateId::Am01,
    CandidateId::Am02,
    CandidateId::Am03,
    CandidateId::Am04,
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CaseId {
    Ac01,
    Ac02,
    Ac03,
    Ac04,
    Ac05,
    Ac06,
    Ac07,
    Ac08,
}

impl CaseId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Ac01 => "AC-01",
            Self::Ac02 => "AC-02",
            Self::Ac03 => "AC-03",
            Self::Ac04 => "AC-04",
            Self::Ac05 => "AC-05",
            Self::Ac06 => "AC-06",
            Self::Ac07 => "AC-07",
            Self::Ac08 => "AC-08",
        }
    }
}

impl fmt::Display for CaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CASES: [CaseId; 8] = [
    CaseId::Ac01,
    CaseId::Ac02,
    CaseId::Ac03,
    CaseId::Ac04,
    CaseId::Ac05,
    CaseId::Ac06,
    CaseId::Ac07,
    CaseId::Ac08,
];

pub(crate) const CLAIM_FAMILIES: [&str; 10] = [
    "CF-01", "CF-02", "CF-03", "CF-04", "CF-05", "CF-06", "CF-07", "CF-08", "CF-09", "CF-10",
];

pub(crate) const METRICS: [&str; 18] = [
    "M-01", "M-02", "M-03", "M-04", "M-05", "M-06", "M-07", "M-08", "M-09", "M-10", "M-11", "M-12",
    "M-13", "M-14", "M-15", "M-16", "M-17", "M-18",
];

pub(crate) const HARD_GATES: [&str; 8] = [
    "HG-01", "HG-02", "HG-03", "HG-04", "HG-05", "HG-06", "HG-07", "HG-08",
];

pub(crate) const OWNER_SCOPES: [&str; 8] = [
    "AR-01", "AR-02", "AR-03", "AR-04", "AR-05", "AR-06", "AR-07", "AR-08",
];

pub(crate) const LEGACY_V01_MUTATIONS: [&str; 5] = [
    "LV01-TEST-AS-REFINEMENT",
    "LV01-OPTIONAL-MASKS-FAILED-KERNEL",
    "LV01-UNRESOLVED-TARGET-LEAKAGE",
    "LV01-OWNER-AS-EXTERNAL",
    "LV01-SUBJECT-SUBSTITUTION",
];

pub(crate) const NONCLAIMS: [&str; 4] = [
    "no candidate selected",
    "no D-005 execution evidence",
    "no public assurance schema ratified",
    "no milestone or release authorized",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InputBindingId {
    DecisionSuite,
    LegacyV01Manifest,
    ClaimRecordV01Schema,
}

impl InputBindingId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::DecisionSuite => "decision_suite",
            Self::LegacyV01Manifest => "legacy_v01_manifest",
            Self::ClaimRecordV01Schema => "claim_record_v01_schema",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InputBinding {
    pub(crate) id: InputBindingId,
    pub(crate) path: &'static str,
    pub(crate) sha256: &'static str,
}

pub(crate) const INPUT_BINDINGS: [InputBinding; 3] = [
    InputBinding {
        id: InputBindingId::DecisionSuite,
        path: "docs/PUBLIC_ASSURANCE_MODEL_DECISION_SUITE.md",
        sha256: "e23fc55a1b315f7ed040412ba5361ecba952b083db7983697bfdc2e6030a29c3",
    },
    InputBinding {
        id: InputBindingId::LegacyV01Manifest,
        path: "research/decisions/D-005/d005-v0.1/epochs/0001/shared-inputs/legacy-v0.1-mutations.json",
        sha256: "2bae9af1e102fe4a9233c78599a3b14a7ca1796f0c0fdfaa17539a998ff01b4d",
    },
    InputBinding {
        id: InputBindingId::ClaimRecordV01Schema,
        path: "schemas/gate0/claim-record-v0.1.schema.json",
        sha256: "a287dde9ddf114da30af61d050aa96406f23e480d62e0f796d66943489579131",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BudgetSpec {
    pub(crate) max_packet_bytes: usize,
    pub(crate) max_json_depth: usize,
    pub(crate) max_json_nodes: usize,
    pub(crate) max_string_bytes: usize,
    pub(crate) max_diagnostics: usize,
    pub(crate) max_claims: usize,
    pub(crate) max_edges: usize,
    pub(crate) max_output_bytes: usize,
    pub(crate) render_repetitions: usize,
    pub(crate) workspace_replays: usize,
}

pub(crate) const BUDGETS: BudgetSpec = BudgetSpec {
    max_packet_bytes: 256 * 1024,
    max_json_depth: 32,
    max_json_nodes: 16_384,
    max_string_bytes: 16_384,
    max_diagnostics: 256,
    max_claims: 4_096,
    max_edges: 16_384,
    max_output_bytes: 4 * 1024 * 1024,
    render_repetitions: REQUIRED_RENDER_REPETITIONS,
    workspace_replays: REQUIRED_WORKSPACE_REPLAYS,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MutationSpec {
    pub(crate) id: &'static str,
    pub(crate) case: CaseId,
    pub(crate) description: &'static str,
}
