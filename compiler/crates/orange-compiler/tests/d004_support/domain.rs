use std::fmt;

pub(crate) const REQUIRED_CANDIDATE_CASES: usize = 25;
pub(crate) const INPUT_BINDING_COUNT: usize = 19;

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

pub(crate) const PROTOCOL_GAPS: [&str; 6] = [
    "ambiguity fixture coverage unresolved",
    "missing-edge fixture coverage unresolved",
    "identity-substitution fixture coverage unresolved",
    "unsupported fixture coverage unresolved",
    "resource-exhaustion fixture coverage unresolved",
    "replay repetition count unresolved",
];

pub(crate) const NONCLAIMS: [&str; 6] = [
    "no candidate adapter executed",
    "no D-004 evidence epoch frozen",
    "no semantic-strata candidate selected",
    "no D-003 disposition inferred",
    "no roadmap gate or readiness movement",
    "no S3b implementation authorized",
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
        sha256: "a359302fcbd8ba81b5c616811bf29f7bd004d362b691450126d6e248cacc8dc2",
    },
    InputBinding {
        id: InputBindingId::ProductFormDecisionPacket,
        path: "docs/PRODUCT_FORM_DECISION_PACKET.md",
        sha256: "8fcd7cf378d488bed49ed3cdd3609475d681c7b1e414927c7c761989e67093e9",
    },
    InputBinding {
        id: InputBindingId::AcceptedS2Language,
        path: "docs/LANGUAGE_2026.md",
        sha256: "52b6ef45ff5ee5d9f3951b1d6bf0f2e40a1566de623fc01e809a2a6d84d7a082",
    },
    InputBinding {
        id: InputBindingId::UserJourneys,
        path: "docs/USER_JOURNEYS.md",
        sha256: "f26b179db777295b620731402962dc3092128f8f9a27049638f22883e0652bed",
    },
    InputBinding {
        id: InputBindingId::AcceptedS3aSemantics,
        path: "docs/SEMANTICS_2026.md",
        sha256: "bc429d9f1296aee9376d377f93c013f74ba8b6f7e3cb48eb6410498a0b8a00e7",
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
