use std::fmt;

pub(crate) const REQUIRED_CANDIDATE_CASES: usize = 25;
pub(crate) const INPUT_BINDING_COUNT: usize = 22;
pub(crate) const CROSS_CUTTING_PROPOSAL_COUNT: usize = 39;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CandidateId {
    Rel,
    Uni,
    Dual,
    Mirror,
    Host,
}

impl CandidateId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Rel => "ST-REL",
            Self::Uni => "ST-UNI",
            Self::Dual => "ST-DUAL",
            Self::Mirror => "ST-MIRROR",
            Self::Host => "ST-HOST",
        }
    }
}

impl fmt::Display for CandidateId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CANDIDATES: [CandidateId; 5] = [
    CandidateId::Rel,
    CandidateId::Uni,
    CandidateId::Dual,
    CandidateId::Mirror,
    CandidateId::Host,
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum CaseId {
    Sc01,
    Sc02,
    Sc03,
    Sc04,
    Sc05,
}

impl CaseId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Sc01 => "SC-01",
            Self::Sc02 => "SC-02",
            Self::Sc03 => "SC-03",
            Self::Sc04 => "SC-04",
            Self::Sc05 => "SC-05",
        }
    }
}

impl fmt::Display for CaseId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

pub(crate) const CASES: [CaseId; 5] = [
    CaseId::Sc01,
    CaseId::Sc02,
    CaseId::Sc03,
    CaseId::Sc04,
    CaseId::Sc05,
];

pub(crate) const RELATIONSHIPS: [&str; 14] = [
    "SR-01", "SR-02", "SR-03", "SR-04", "SR-05", "SR-06", "SR-07", "SR-08", "SR-09", "SR-10",
    "SR-11", "SR-12", "SR-13", "SR-14",
];

pub(crate) const HARD_GATES: [&str; 10] = [
    "SS-G01", "SS-G02", "SS-G03", "SS-G04", "SS-G05", "SS-G06", "SS-G07", "SS-G08", "SS-G09",
    "SS-G10",
];

pub(crate) const SOURCE_ROLES: [&str; 5] = [
    "Specification",
    "Implementation",
    "Machine Implementation",
    "Game",
    "Proof",
];

pub(crate) const DOMAIN_OBSERVATION_STATES: [&str; 6] = [
    "succeeded",
    "rejected",
    "unknown",
    "timeout",
    "unsupported",
    "exhausted",
];

pub(crate) const CASE_VERDICTS: [&str; 2] = ["pass", "fail"];

pub(crate) const UNRESOLVED_CROSS_CUTTING_FIXTURE_CLASSES: [&str; 5] = [
    "ambiguity",
    "missing-edge",
    "identity-substitution",
    "unsupported",
    "resource-exhaustion",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ProposalClassStatusSpec {
    pub(crate) class: &'static str,
    pub(crate) proposal_count: usize,
}

pub(crate) const CROSS_CUTTING_PROPOSAL_CLASS_STATUSES: [ProposalClassStatusSpec; 5] = [
    ProposalClassStatusSpec {
        class: "ambiguity",
        proposal_count: 5,
    },
    ProposalClassStatusSpec {
        class: "missing-edge",
        proposal_count: 14,
    },
    ProposalClassStatusSpec {
        class: "identity-substitution",
        proposal_count: 10,
    },
    ProposalClassStatusSpec {
        class: "unsupported",
        proposal_count: 5,
    },
    ProposalClassStatusSpec {
        class: "resource-exhaustion",
        proposal_count: 5,
    },
];

pub(crate) const MISSING_EDGE_PROPOSAL_IDS: [&str; 14] = [
    "D004-XF-ME-SR01",
    "D004-XF-ME-SR02",
    "D004-XF-ME-SR03",
    "D004-XF-ME-SR04",
    "D004-XF-ME-SR05",
    "D004-XF-ME-SR06",
    "D004-XF-ME-SR07",
    "D004-XF-ME-SR08",
    "D004-XF-ME-SR09",
    "D004-XF-ME-SR10",
    "D004-XF-ME-SR11",
    "D004-XF-ME-SR12",
    "D004-XF-ME-SR13",
    "D004-XF-ME-SR14",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IdentitySubstitutionProposalSpec {
    pub(crate) id: &'static str,
    pub(crate) target: &'static str,
}

pub(crate) const IDENTITY_SUBSTITUTION_PROPOSALS: [IdentitySubstitutionProposalSpec; 10] = [
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-PACKET",
        target: "packet_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-REPLAY-PLAN",
        target: "replay_plan_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-SCHEDULED-SLOT",
        target: "scheduled_slot_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-INPUT-MANIFEST",
        target: "input_manifest_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-CANDIDATE-GRAPH",
        target: "candidate_graph_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-SR-MAP",
        target: "sr_map_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-SEMANTIC-ENDPOINT",
        target: "semantic_endpoint_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-PARAMETER-MODEL",
        target: "parameter_model_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-TOOL",
        target: "tool_identity",
    },
    IdentitySubstitutionProposalSpec {
        id: "D004-XF-ID-ENVIRONMENT",
        target: "environment_identity",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CaseScopedProposalSpec {
    pub(crate) id: &'static str,
    pub(crate) class: &'static str,
    pub(crate) case: CaseId,
    pub(crate) relationship_scope: &'static [&'static str],
    pub(crate) mutation_kind: &'static str,
    pub(crate) target: &'static str,
    pub(crate) expected_state: &'static str,
}

pub(crate) const CASE_SCOPED_CROSS_CUTTING_PROPOSALS: [CaseScopedProposalSpec; 15] = [
    CaseScopedProposalSpec {
        id: "D004-XF-AMB-SC01",
        class: "ambiguity",
        case: CaseId::Sc01,
        relationship_scope: &["SR-01"],
        mutation_kind: "admit_competing_authoritative_interpretations",
        target: "numeric_word_byte_order_and_signedness_interpretation",
        expected_state: "rejected",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-AMB-SC02",
        class: "ambiguity",
        case: CaseId::Sc02,
        relationship_scope: &["SR-01", "SR-02", "SR-09"],
        mutation_kind: "admit_competing_authoritative_interpretations",
        target: "mutable_memory_and_refinement_interpretation",
        expected_state: "rejected",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-AMB-SC03",
        class: "ambiguity",
        case: CaseId::Sc03,
        relationship_scope: &["SR-02", "SR-10"],
        mutation_kind: "admit_competing_authoritative_interpretations",
        target: "suite_policy_observation_classification",
        expected_state: "rejected",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-AMB-SC04",
        class: "ambiguity",
        case: CaseId::Sc04,
        relationship_scope: &["SR-01", "SR-03", "SR-11"],
        mutation_kind: "admit_competing_authoritative_interpretations",
        target: "intrinsic_abstract_machine_mapping",
        expected_state: "rejected",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-AMB-SC05",
        class: "ambiguity",
        case: CaseId::Sc05,
        relationship_scope: &["SR-04", "SR-08", "SR-12"],
        mutation_kind: "admit_competing_authoritative_interpretations",
        target: "game_sampling_probability_and_reduction_interpretation",
        expected_state: "rejected",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-US-SC01",
        class: "unsupported",
        case: CaseId::Sc01,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_unsupported_behavior",
        target: "sc01_unsupported_fixture_slot",
        expected_state: "unsupported",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-US-SC02",
        class: "unsupported",
        case: CaseId::Sc02,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_unsupported_behavior",
        target: "sc02_unsupported_fixture_slot",
        expected_state: "unsupported",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-US-SC03",
        class: "unsupported",
        case: CaseId::Sc03,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_unsupported_behavior",
        target: "sc03_unsupported_fixture_slot",
        expected_state: "unsupported",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-US-SC04",
        class: "unsupported",
        case: CaseId::Sc04,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_unsupported_behavior",
        target: "sc04_unsupported_fixture_slot",
        expected_state: "unsupported",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-US-SC05",
        class: "unsupported",
        case: CaseId::Sc05,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_unsupported_behavior",
        target: "sc05_unsupported_fixture_slot",
        expected_state: "unsupported",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-RE-SC01",
        class: "resource-exhaustion",
        case: CaseId::Sc01,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_domain_exhaustion",
        target: "sc01_resource_fixture_slot",
        expected_state: "exhausted",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-RE-SC02",
        class: "resource-exhaustion",
        case: CaseId::Sc02,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_domain_exhaustion",
        target: "sc02_resource_fixture_slot",
        expected_state: "exhausted",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-RE-SC03",
        class: "resource-exhaustion",
        case: CaseId::Sc03,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_domain_exhaustion",
        target: "sc03_resource_fixture_slot",
        expected_state: "exhausted",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-RE-SC04",
        class: "resource-exhaustion",
        case: CaseId::Sc04,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_domain_exhaustion",
        target: "sc04_resource_fixture_slot",
        expected_state: "exhausted",
    },
    CaseScopedProposalSpec {
        id: "D004-XF-RE-SC05",
        class: "resource-exhaustion",
        case: CaseId::Sc05,
        relationship_scope: &[],
        mutation_kind: "exercise_preregistered_domain_exhaustion",
        target: "sc05_resource_fixture_slot",
        expected_state: "exhausted",
    },
];

pub(crate) const PROTOCOL_GAPS: [&str; 6] = [
    "ambiguity fixture sufficiency review unresolved",
    "missing-edge fixture sufficiency review unresolved",
    "identity-substitution fixture sufficiency review unresolved",
    "unsupported fixture sufficiency review unresolved",
    "resource-exhaustion fixture sufficiency review unresolved",
    "replay repetition count unresolved",
];

pub(crate) const NONCLAIMS: [&str; 6] = [
    "no candidate adapter executed",
    "no D-004 evidence epoch frozen",
    "no semantic-strata candidate selected",
    "no D-004 disposition inferred from D-003 acceptance",
    "no roadmap gate or readiness movement",
    "no S3b implementation authorized",
];

pub(crate) const CROSS_CUTTING_PROPOSAL_NONCLAIMS: [&str; 11] = [
    "no candidate adapter executed",
    "no D-004 evidence epoch frozen",
    "no semantic-strata candidate selected",
    "no D-003 disposition inferred",
    "no roadmap gate or readiness movement",
    "no S3b implementation authorized",
    "proposal definitions are not executable fixture coverage or evidence",
    "unsupported proposals do not establish candidate support or capability absence",
    "candidate adapter inability is not a preregistered unsupported observation",
    "resource-exhaustion proposals do not exercise or verify any resource ceiling",
    "replay repetitions remain unresolved and unassigned",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum InputBindingId {
    NamedMutationsManifest,
    DecisionSuite,
    ProductFormDecisionPacket,
    AcceptedS2Language,
    UserJourneys,
    AcceptedS3aSemantics,
    AcceptedS3aOep,
    S3aConformanceRunner,
    PermanentS3aFixture,
    InvalidDuplicateSpec,
    InvalidIntMagnitude,
    InvalidNegativeWord,
    InvalidTypedImpl,
    InvalidUnsupportedType,
    InvalidWordRange,
    InvalidWordWidth,
    ValidEmptyMixed,
    ValidIntRadices,
    ValidWord8Boundaries,
    CrossCuttingFixtureProposals,
    CrossCuttingExecutableFixtures,
    CaseSubjects,
}

impl InputBindingId {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::NamedMutationsManifest => "named_mutations_manifest",
            Self::DecisionSuite => "decision_suite",
            Self::ProductFormDecisionPacket => "product_form_decision_packet",
            Self::AcceptedS2Language => "accepted_s2_language",
            Self::UserJourneys => "user_journeys",
            Self::AcceptedS3aSemantics => "accepted_s3a_semantics",
            Self::AcceptedS3aOep => "accepted_s3a_oep",
            Self::S3aConformanceRunner => "s3a_conformance_runner",
            Self::PermanentS3aFixture => "permanent_s3a_fixture",
            Self::InvalidDuplicateSpec => "fixture_invalid_duplicate_spec",
            Self::InvalidIntMagnitude => "fixture_invalid_int_magnitude",
            Self::InvalidNegativeWord => "fixture_invalid_negative_word",
            Self::InvalidTypedImpl => "fixture_invalid_typed_impl",
            Self::InvalidUnsupportedType => "fixture_invalid_unsupported_type",
            Self::InvalidWordRange => "fixture_invalid_word_range",
            Self::InvalidWordWidth => "fixture_invalid_word_width",
            Self::ValidEmptyMixed => "fixture_valid_empty_mixed",
            Self::ValidIntRadices => "fixture_valid_int_radices",
            Self::ValidWord8Boundaries => "fixture_valid_word8_boundaries",
            Self::CrossCuttingFixtureProposals => "cross_cutting_fixture_proposals",
            Self::CrossCuttingExecutableFixtures => "cross_cutting_executable_fixtures",
            Self::CaseSubjects => "case_subjects",
        }
    }

    pub(crate) const fn index(self) -> usize {
        match self {
            Self::NamedMutationsManifest => 0,
            Self::DecisionSuite => 1,
            Self::ProductFormDecisionPacket => 2,
            Self::AcceptedS2Language => 3,
            Self::UserJourneys => 4,
            Self::AcceptedS3aSemantics => 5,
            Self::AcceptedS3aOep => 6,
            Self::S3aConformanceRunner => 7,
            Self::PermanentS3aFixture => 8,
            Self::InvalidDuplicateSpec => 9,
            Self::InvalidIntMagnitude => 10,
            Self::InvalidNegativeWord => 11,
            Self::InvalidTypedImpl => 12,
            Self::InvalidUnsupportedType => 13,
            Self::InvalidWordRange => 14,
            Self::InvalidWordWidth => 15,
            Self::ValidEmptyMixed => 16,
            Self::ValidIntRadices => 17,
            Self::ValidWord8Boundaries => 18,
            Self::CrossCuttingFixtureProposals => 19,
            Self::CrossCuttingExecutableFixtures => 20,
            Self::CaseSubjects => 21,
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
        id: InputBindingId::NamedMutationsManifest,
        path: "research/decisions/D-004/d004-v0.2-named-mutations.json",
        sha256: "1d46d6d66c0704fcaa462c625dcac2e72150497bb075322c5e076ea42898be54",
    },
    InputBinding {
        id: InputBindingId::DecisionSuite,
        path: "docs/SEMANTIC_STRATA_DECISION_SUITE.md",
        sha256: "f44c556202d0e235fd42c03181134ab3047d3e9b6f19b3015dae15a86d00dc0b",
    },
    InputBinding {
        id: InputBindingId::ProductFormDecisionPacket,
        path: "docs/PRODUCT_FORM_DECISION_PACKET.md",
        sha256: "1ef0be53344667993778d1abd9a83423fc92b358735ed7ad74cb766bb29d33fd",
    },
    InputBinding {
        id: InputBindingId::AcceptedS2Language,
        path: "docs/LANGUAGE_2026.md",
        sha256: "35981310cbe1e1ae61c889b4005b2610d0077e6a615a5e032b0ca9a5860b328a",
    },
    InputBinding {
        id: InputBindingId::UserJourneys,
        path: "docs/USER_JOURNEYS.md",
        sha256: "f26b179db777295b620731402962dc3092128f8f9a27049638f22883e0652bed",
    },
    InputBinding {
        id: InputBindingId::AcceptedS3aSemantics,
        path: "docs/SEMANTICS_2026.md",
        sha256: "63e14d674eb687f46aa600b36d6d13e3732090d658fb05fd805646b1d469dbdf",
    },
    InputBinding {
        id: InputBindingId::AcceptedS3aOep,
        path: "docs/governance/oeps/OEP-0003-orange-2026-typed-literals.md",
        sha256: "4ea34fc2499ba6b90eb930262f84f15a41ff0df0f526a4533a48e54ea4f9b4b8",
    },
    InputBinding {
        id: InputBindingId::S3aConformanceRunner,
        path: "compiler/crates/orangec/tests/s3a_conformance.rs",
        sha256: "7d25ea303fcb3c1603d60b6cb32d89ae15173cc043b8b695daedb737162b8116",
    },
    InputBinding {
        id: InputBindingId::PermanentS3aFixture,
        path: "compiler/fixtures/typed-answer.or",
        sha256: "22c71b6b8e09ff8dbb7393abfb6ce46597eed0b45f9a34660aa948071138ff6e",
    },
    InputBinding {
        id: InputBindingId::InvalidDuplicateSpec,
        path: "compiler/fixtures/s3a/invalid-duplicate-spec.or",
        sha256: "f3b870468c5f4a98c9dae6c94de74aacbabbf15e480296f696a87d5aebb209d6",
    },
    InputBinding {
        id: InputBindingId::InvalidIntMagnitude,
        path: "compiler/fixtures/s3a/invalid-int-magnitude.or",
        sha256: "11826c807240ac2fc4beddb26f25c3b14dd75008ed756f2afa3ee95668b05542",
    },
    InputBinding {
        id: InputBindingId::InvalidNegativeWord,
        path: "compiler/fixtures/s3a/invalid-negative-word.or",
        sha256: "4643e1247a017202f25a240ad72c83adbd7d2f436ec4de2dffbac1e292ce161b",
    },
    InputBinding {
        id: InputBindingId::InvalidTypedImpl,
        path: "compiler/fixtures/s3a/invalid-typed-impl.or",
        sha256: "4e457e50fbc3b8458c877c9a790e169ff643784b5b78f7a3a0f83a117cc7be07",
    },
    InputBinding {
        id: InputBindingId::InvalidUnsupportedType,
        path: "compiler/fixtures/s3a/invalid-unsupported-type.or",
        sha256: "14190eb262c79772b583c458500c777c54ef0c8913fc046a8809b5a146cfb9fc",
    },
    InputBinding {
        id: InputBindingId::InvalidWordRange,
        path: "compiler/fixtures/s3a/invalid-word-range.or",
        sha256: "4a7a4fd4bdfecdc21133f5f6ff24e212dde0bf357fe6d6807816930895300ddf",
    },
    InputBinding {
        id: InputBindingId::InvalidWordWidth,
        path: "compiler/fixtures/s3a/invalid-word-width.or",
        sha256: "d92ac896bd872f1aa4a3c8988d0b654a23c95ec10ec9183a7d2431cd12238be2",
    },
    InputBinding {
        id: InputBindingId::ValidEmptyMixed,
        path: "compiler/fixtures/s3a/valid-empty-mixed.or",
        sha256: "c30ab3cda5caa11d826dc38ea257d9c9413d6240c09b236a7f50f1cac9016b96",
    },
    InputBinding {
        id: InputBindingId::ValidIntRadices,
        path: "compiler/fixtures/s3a/valid-int-radices.or",
        sha256: "937f8f67b20794c9a887bcca15ea619276f921bc9bf884fdc35e7caab6ac11e4",
    },
    InputBinding {
        id: InputBindingId::ValidWord8Boundaries,
        path: "compiler/fixtures/s3a/valid-word8-boundaries.or",
        sha256: "db37bd00375daa1db43498c5f10b831fdaa5d43b3b886ef838ecbb8d0fbea2ee",
    },
    InputBinding {
        id: InputBindingId::CrossCuttingFixtureProposals,
        path: "research/decisions/D-004/d004-v0.2-cross-cutting-fixture-proposals.json",
        sha256: "171c7b88d54fe2bd7ddb4c220adb63f006e07c35391018b914482ace17cf7e93",
    },
    InputBinding {
        id: InputBindingId::CrossCuttingExecutableFixtures,
        path: "research/decisions/D-004/d004-v0.3-cross-cutting-executable-fixtures.json",
        sha256: "268b4065028f1af9c9ec912ae8884c150094189f5d782963f42ed6ed4cca6ce0",
    },
    InputBinding {
        id: InputBindingId::CaseSubjects,
        path: "research/decisions/D-004/d004-v0.4-case-subjects.json",
        sha256: "c94100598aaf39954fe683a44f6a4d34837304eb361a1b478ca26884892d8ed6",
    },
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BudgetSpec {
    pub(crate) max_packet_bytes: usize,
    pub(crate) max_json_depth: usize,
    pub(crate) max_json_nodes: usize,
    pub(crate) max_string_bytes: usize,
    pub(crate) case_wall_seconds: usize,
    pub(crate) case_peak_memory_bytes: u64,
    pub(crate) case_temp_storage_bytes: u64,
    pub(crate) case_output_bytes: u64,
    pub(crate) candidate_owner_hours: usize,
    pub(crate) correction_owner_hours: usize,
}

pub(crate) const BUDGETS: BudgetSpec = BudgetSpec {
    max_packet_bytes: 256 * 1024,
    max_json_depth: 32,
    max_json_nodes: 16_384,
    max_string_bytes: 16_384,
    case_wall_seconds: 15 * 60,
    case_peak_memory_bytes: 4_u64 * 1024 * 1024 * 1024,
    case_temp_storage_bytes: 2_u64 * 1024 * 1024 * 1024,
    case_output_bytes: 256_u64 * 1024 * 1024,
    candidate_owner_hours: 24,
    correction_owner_hours: 4,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MutationSpec {
    pub(crate) id: &'static str,
    pub(crate) case: CaseId,
    pub(crate) description: &'static str,
}
