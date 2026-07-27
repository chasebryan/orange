use super::domain::{CANDIDATES, CASES, CandidateId, CaseId, REQUIRED_CANDIDATE_CASES};

pub(crate) const REQUIRED_REPETITIONS_PER_SLOT: usize = 3;
pub(crate) const REQUIRED_EXECUTION_RECORDS: usize =
    REQUIRED_CANDIDATE_CASES * REQUIRED_REPETITIONS_PER_SLOT;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PlannedExecution {
    pub(crate) ordinal: usize,
    pub(crate) round: usize,
    pub(crate) position: usize,
    pub(crate) candidate: CandidateId,
    pub(crate) case: CaseId,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ReviewedExecution {
    pub(crate) execution_ordinal: usize,
    pub(crate) repetition: usize,
    pub(crate) logical_slot_ordinal: usize,
    pub(crate) round: usize,
    pub(crate) position: usize,
    pub(crate) candidate: CandidateId,
    pub(crate) case: CaseId,
}

impl ReviewedExecution {
    pub(crate) const fn logical_slot(self) -> PlannedExecution {
        PlannedExecution {
            ordinal: self.logical_slot_ordinal,
            round: self.round,
            position: self.position,
            candidate: self.candidate,
            case: self.case,
        }
    }
}

/// The reviewed 5-by-5 balanced Latin identity schedule inherited byte-for-byte
/// from the v0.5 draft laboratory.
pub(crate) fn latin_base_schedule() -> Vec<PlannedExecution> {
    let mut schedule = Vec::with_capacity(REQUIRED_CANDIDATE_CASES);
    for round in 0..CANDIDATES.len() {
        for position in 0..CANDIDATES.len() {
            schedule.push(PlannedExecution {
                ordinal: schedule.len() + 1,
                round: round + 1,
                position: position + 1,
                candidate: CANDIDATES[(round + position) % CANDIDATES.len()],
                case: CASES[(2 * round + position) % CASES.len()],
            });
        }
    }
    schedule
}

/// Expands the logical schedule in the reviewed physical order: one complete
/// Latin traversal for repetition one, then repetitions two and three.
pub(crate) fn repetition_major_execution_schedule() -> Vec<ReviewedExecution> {
    let logical = latin_base_schedule();
    let mut schedule = Vec::with_capacity(REQUIRED_EXECUTION_RECORDS);
    for repetition in 1..=REQUIRED_REPETITIONS_PER_SLOT {
        for slot in &logical {
            schedule.push(ReviewedExecution {
                execution_ordinal: schedule.len() + 1,
                repetition,
                logical_slot_ordinal: slot.ordinal,
                round: slot.round,
                position: slot.position,
                candidate: slot.candidate,
                case: slot.case,
            });
        }
    }
    schedule
}
