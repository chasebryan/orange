use super::domain::{CaseId, MutationSpec};

pub(crate) const MUTATIONS: [MutationSpec; 26] = [
    MutationSpec {
        id: "SC-01-M01",
        case: CaseId::Sc01,
        description: "introduce an implicit integer-to-word conversion",
    },
    MutationSpec {
        id: "SC-01-M02",
        case: CaseId::Sc01,
        description: "introduce an implicit endian conversion",
    },
    MutationSpec {
        id: "SC-01-M03",
        case: CaseId::Sc01,
        description: "substitute a mismatched word width",
    },
    MutationSpec {
        id: "SC-01-M04",
        case: CaseId::Sc01,
        description: "use an unbounded shift",
    },
    MutationSpec {
        id: "SC-02-M01",
        case: CaseId::Sc02,
        description: "introduce an illegal alias",
    },
    MutationSpec {
        id: "SC-02-M02",
        case: CaseId::Sc02,
        description: "access outside the declared range",
    },
    MutationSpec {
        id: "SC-02-M03",
        case: CaseId::Sc02,
        description: "remove the required loop invariant",
    },
    MutationSpec {
        id: "SC-02-M04",
        case: CaseId::Sc02,
        description: "introduce an uninitialized read",
    },
    MutationSpec {
        id: "SC-02-M05",
        case: CaseId::Sc02,
        description: "substitute the refinement subject",
    },
    MutationSpec {
        id: "SC-03-M01",
        case: CaseId::Sc03,
        description: "branch on a secret value",
    },
    MutationSpec {
        id: "SC-03-M02",
        case: CaseId::Sc03,
        description: "address memory with a secret value",
    },
    MutationSpec {
        id: "SC-03-M03",
        case: CaseId::Sc03,
        description: "bound a loop with a secret value",
    },
    MutationSpec {
        id: "SC-03-M04",
        case: CaseId::Sc03,
        description: "select a failure path with a secret value",
    },
    MutationSpec {
        id: "SC-03-M05",
        case: CaseId::Sc03,
        description: "emit a secret-dependent debug observation",
    },
    MutationSpec {
        id: "SC-04-M01",
        case: CaseId::Sc04,
        description: "remove the required target feature",
    },
    MutationSpec {
        id: "SC-04-M02",
        case: CaseId::Sc04,
        description: "substitute an unsupported intrinsic",
    },
    MutationSpec {
        id: "SC-04-M03",
        case: CaseId::Sc04,
        description: "reverse the declared lane order",
    },
    MutationSpec {
        id: "SC-04-M04",
        case: CaseId::Sc04,
        description: "substitute a mismatched vector width",
    },
    MutationSpec {
        id: "SC-04-M05",
        case: CaseId::Sc04,
        description: "substitute the target-model identity",
    },
    MutationSpec {
        id: "SC-04-M06",
        case: CaseId::Sc04,
        description: "select an undeclared fallback",
    },
    MutationSpec {
        id: "SC-05-M01",
        case: CaseId::Sc05,
        description: "place sampling in specification semantics",
    },
    MutationSpec {
        id: "SC-05-M02",
        case: CaseId::Sc05,
        description: "use ambient randomness instead of explicit sampling",
    },
    MutationSpec {
        id: "SC-05-M03",
        case: CaseId::Sc05,
        description: "hide an oracle effect",
    },
    MutationSpec {
        id: "SC-05-M04",
        case: CaseId::Sc05,
        description: "replace finite sampling with an unbounded sample",
    },
    MutationSpec {
        id: "SC-05-M05",
        case: CaseId::Sc05,
        description: "substitute a reduction endpoint subject",
    },
    MutationSpec {
        id: "SC-05-M06",
        case: CaseId::Sc05,
        description: "alter the symbolic advantage bound",
    },
];
