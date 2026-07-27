use std::collections::BTreeMap;

use super::candidate_mappings::{CandidateMappingErrorKind, parse_candidate_mapping_catalog};
use super::case_subjects::{CaseSubjectErrorKind, parse_case_subject_catalog};
use super::domain::{
    CandidateId, CaseId, INPUT_BINDING_COUNT, INPUT_BINDINGS, InputBindingId, NONCLAIMS,
    REQUIRED_CANDIDATE_CASES,
};
use super::fixtures::{FixtureErrorKind, parse_fixture_catalog};
use super::packet::{
    CANDIDATE_MAPPING_CATALOG_SHA256, CASE_SUBJECT_CATALOG_SHA256,
    CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256,
    CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256, DraftPacket, MUTATION_MANIFEST_SHA256,
    PacketErrorKind, canonical_cross_cutting_fixture_proposal_manifest_file_bytes,
    canonical_mutation_manifest_file_bytes, cross_cutting_fixture_proposal_manifest_digest_hex,
    mutation_manifest_digest_hex, parse_cross_cutting_fixture_proposal_manifest,
    parse_mutation_manifest,
};
pub(crate) use super::schedule::PlannedExecution;
use super::schedule::latin_base_schedule;
use super::sha256;
use super::strict_json::{self, JsonValue};

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
    MutationManifest(PacketErrorKind),
    NonCanonicalMutationManifest,
    MutationManifestDigest {
        expected_sha256: &'static str,
        observed_sha256: String,
    },
    CrossCuttingFixtureProposalManifest(PacketErrorKind),
    NonCanonicalCrossCuttingFixtureProposalManifest,
    CrossCuttingFixtureProposalManifestDigest {
        expected_sha256: &'static str,
        observed_sha256: String,
    },
    CrossCuttingExecutableFixtureCatalog(FixtureErrorKind),
    CrossCuttingExecutableFixtureCatalogDigest {
        expected_sha256: &'static str,
        observed_sha256: String,
    },
    CaseSubjectCatalog(CaseSubjectErrorKind),
    CaseSubjectCatalogDigest {
        expected_sha256: &'static str,
        observed_sha256: String,
    },
    CandidateMappingCatalog(CandidateMappingErrorKind),
    CandidateMappingCatalogDigest {
        expected_sha256: &'static str,
        observed_sha256: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReplayPlan {
    packet_sha256: String,
    schedule: Vec<PlannedExecution>,
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

    pub(crate) const fn complete_candidates(&self) -> usize {
        0
    }

    pub(crate) const fn complete_cross_candidate_cases(&self) -> usize {
        0
    }

    pub(crate) const fn evidence_status(&self) -> &'static str {
        "none"
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
        let schedule = self
            .schedule
            .iter()
            .map(|execution| {
                strict_json::object([
                    usize_entry("ordinal", execution.ordinal),
                    usize_entry("round", execution.round),
                    usize_entry("position", execution.position),
                    string_entry("candidate", execution.candidate.as_str()),
                    string_entry("case", execution.case.as_str()),
                ])
            })
            .collect();
        strict_json::object([
            string_entry("packet_sha256", &self.packet_sha256),
            usize_entry("required_candidate_cases", REQUIRED_CANDIDATE_CASES),
            usize_entry("completed_candidate_cases", 0),
            usize_entry("complete_candidates", 0),
            usize_entry("complete_cross_candidate_cases", 0),
            string_entry("evidence_status", self.evidence_status()),
            ("selection".to_owned(), JsonValue::Null),
            ("conclusion".to_owned(), JsonValue::Null),
            ("schedule".to_owned(), JsonValue::Array(schedule)),
            ("nonclaims".to_owned(), strict_json::strings(NONCLAIMS)),
        ])
    }
}

pub(crate) fn prepare_replay(
    packet: &DraftPacket,
    inputs: &ReplayInputs<'_>,
) -> Result<ReplayPlan, ReplayError> {
    for binding in INPUT_BINDINGS {
        let bytes = inputs.get(binding.id);
        let observed_sha256 = sha256::hex(&sha256::digest(bytes));
        if observed_sha256 != binding.sha256 {
            return Err(ReplayError::InputDigest {
                input: binding.id,
                path: binding.path,
                expected_sha256: binding.sha256,
                observed_sha256,
            });
        }
    }

    let manifest_bytes = inputs.get(InputBindingId::NamedMutationsManifest);
    let manifest = parse_mutation_manifest(manifest_bytes)
        .map_err(|error| ReplayError::MutationManifest(error.kind))?;
    if manifest_bytes != canonical_mutation_manifest_file_bytes() {
        return Err(ReplayError::NonCanonicalMutationManifest);
    }
    let observed_manifest_digest =
        sha256::hex(&sha256::digest(&strict_json::canonical_bytes(&manifest)));
    if observed_manifest_digest != MUTATION_MANIFEST_SHA256
        || mutation_manifest_digest_hex() != MUTATION_MANIFEST_SHA256
    {
        return Err(ReplayError::MutationManifestDigest {
            expected_sha256: MUTATION_MANIFEST_SHA256,
            observed_sha256: observed_manifest_digest,
        });
    }

    let proposal_manifest_bytes = inputs.get(InputBindingId::CrossCuttingFixtureProposals);
    let proposal_manifest = parse_cross_cutting_fixture_proposal_manifest(proposal_manifest_bytes)
        .map_err(|error| ReplayError::CrossCuttingFixtureProposalManifest(error.kind))?;
    if proposal_manifest_bytes != canonical_cross_cutting_fixture_proposal_manifest_file_bytes() {
        return Err(ReplayError::NonCanonicalCrossCuttingFixtureProposalManifest);
    }
    let observed_proposal_manifest_digest = sha256::hex(&sha256::digest(
        &strict_json::canonical_bytes(&proposal_manifest),
    ));
    if observed_proposal_manifest_digest != CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256
        || cross_cutting_fixture_proposal_manifest_digest_hex()
            != CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256
    {
        return Err(ReplayError::CrossCuttingFixtureProposalManifestDigest {
            expected_sha256: CROSS_CUTTING_FIXTURE_PROPOSAL_MANIFEST_SHA256,
            observed_sha256: observed_proposal_manifest_digest,
        });
    }

    let fixture_catalog_bytes = inputs.get(InputBindingId::CrossCuttingExecutableFixtures);
    let fixture_catalog = parse_fixture_catalog(fixture_catalog_bytes, &proposal_manifest)
        .map_err(|error| ReplayError::CrossCuttingExecutableFixtureCatalog(error.kind))?;
    let observed_fixture_catalog_digest = fixture_catalog.digest_hex();
    if observed_fixture_catalog_digest != CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256 {
        return Err(ReplayError::CrossCuttingExecutableFixtureCatalogDigest {
            expected_sha256: CROSS_CUTTING_EXECUTABLE_FIXTURE_CATALOG_SHA256,
            observed_sha256: observed_fixture_catalog_digest,
        });
    }

    let case_subject_bytes = inputs.get(InputBindingId::CaseSubjects);
    let case_subject_catalog = parse_case_subject_catalog(case_subject_bytes, &manifest)
        .map_err(|error| ReplayError::CaseSubjectCatalog(error.kind))?;
    let observed_case_subject_catalog_digest = case_subject_catalog.digest_hex();
    if observed_case_subject_catalog_digest != CASE_SUBJECT_CATALOG_SHA256 {
        return Err(ReplayError::CaseSubjectCatalogDigest {
            expected_sha256: CASE_SUBJECT_CATALOG_SHA256,
            observed_sha256: observed_case_subject_catalog_digest,
        });
    }

    let candidate_mapping_bytes = inputs.get(InputBindingId::CandidateMappings);
    let candidate_mapping_catalog = parse_candidate_mapping_catalog(candidate_mapping_bytes)
        .map_err(|error| ReplayError::CandidateMappingCatalog(error.kind))?;
    let observed_candidate_mapping_catalog_digest = candidate_mapping_catalog.digest_hex();
    if observed_candidate_mapping_catalog_digest != CANDIDATE_MAPPING_CATALOG_SHA256 {
        return Err(ReplayError::CandidateMappingCatalogDigest {
            expected_sha256: CANDIDATE_MAPPING_CATALOG_SHA256,
            observed_sha256: observed_candidate_mapping_catalog_digest,
        });
    }

    let schedule = latin_base_schedule();
    Ok(ReplayPlan {
        packet_sha256: packet.digest_hex(),
        schedule,
    })
}

pub(crate) fn schedule_pair_counts(plan: &ReplayPlan) -> BTreeMap<(CandidateId, CaseId), usize> {
    let mut counts = BTreeMap::new();
    for execution in plan.schedule() {
        *counts
            .entry((execution.candidate, execution.case))
            .or_insert(0) += 1;
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
