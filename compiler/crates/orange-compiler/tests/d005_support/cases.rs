use super::domain::{CaseId, MutationSpec};

pub(crate) const MUTATIONS: [MutationSpec; 50] = [
    MutationSpec {
        id: "AC-01-M01",
        case: CaseId::Ac01,
        description: "remove compiler-preservation family CF-06",
    },
    MutationSpec {
        id: "AC-01-M02",
        case: CaseId::Ac01,
        description: "merge source and target leakage",
    },
    MutationSpec {
        id: "AC-01-M03",
        case: CaseId::Ac01,
        description: "rename game security as empirical confidence",
    },
    MutationSpec {
        id: "AC-01-M04",
        case: CaseId::Ac01,
        description: "inherit one target result across another target",
    },
    MutationSpec {
        id: "AC-01-M05",
        case: CaseId::Ac01,
        description: "replace mixed outcomes with one color",
    },
    MutationSpec {
        id: "AC-01-M06",
        case: CaseId::Ac01,
        description: "omit an unsupported claim from the public view",
    },
    MutationSpec {
        id: "AC-02-M01",
        case: CaseId::Ac02,
        description: "change basis type while retaining bytes",
    },
    MutationSpec {
        id: "AC-02-M02",
        case: CaseId::Ac02,
        description: "mark recorded evidence as checked",
    },
    MutationSpec {
        id: "AC-02-M03",
        case: CaseId::Ac02,
        description: "let one checked test mask a failed proof",
    },
    MutationSpec {
        id: "AC-02-M04",
        case: CaseId::Ac02,
        description: "use an assumption as its own evidence",
    },
    MutationSpec {
        id: "AC-02-M05",
        case: CaseId::Ac02,
        description: "omit a mandatory basis",
    },
    MutationSpec {
        id: "AC-02-M06",
        case: CaseId::Ac02,
        description: "relabel owner review as proof or external validation",
    },
    MutationSpec {
        id: "AC-03-M01",
        case: CaseId::Ac03,
        description: "substitute one identity coordinate",
    },
    MutationSpec {
        id: "AC-03-M02",
        case: CaseId::Ac03,
        description: "reuse evidence by friendly name",
    },
    MutationSpec {
        id: "AC-03-M03",
        case: CaseId::Ac03,
        description: "swap a target or leakage profile",
    },
    MutationSpec {
        id: "AC-03-M04",
        case: CaseId::Ac03,
        description: "alter one byte after hashing",
    },
    MutationSpec {
        id: "AC-03-M05",
        case: CaseId::Ac03,
        description: "redirect an evidence reference",
    },
    MutationSpec {
        id: "AC-03-M06",
        case: CaseId::Ac03,
        description: "attach a valid proof to the wrong theorem fingerprint",
    },
    MutationSpec {
        id: "AC-04-M01",
        case: CaseId::Ac04,
        description: "remove or substitute a required boundary edge",
    },
    MutationSpec {
        id: "AC-04-M02",
        case: CaseId::Ac04,
        description: "hide a tool as glue",
    },
    MutationSpec {
        id: "AC-04-M03",
        case: CaseId::Ac04,
        description: "break an assumption reference",
    },
    MutationSpec {
        id: "AC-04-M04",
        case: CaseId::Ac04,
        description: "create a trust or composition cycle",
    },
    MutationSpec {
        id: "AC-04-M05",
        case: CaseId::Ac04,
        description: "use owner approval as a preservation edge",
    },
    MutationSpec {
        id: "AC-04-M06",
        case: CaseId::Ac04,
        description: "lend a source claim directly to final bytes",
    },
    MutationSpec {
        id: "AC-05-M01",
        case: CaseId::Ac05,
        description: "delete a revocation",
    },
    MutationSpec {
        id: "AC-05-M02",
        case: CaseId::Ac05,
        description: "prefer cached success",
    },
    MutationSpec {
        id: "AC-05-M03",
        case: CaseId::Ac05,
        description: "accept an older policy after downgrade",
    },
    MutationSpec {
        id: "AC-05-M04",
        case: CaseId::Ac05,
        description: "create multiple current successors",
    },
    MutationSpec {
        id: "AC-05-M05",
        case: CaseId::Ac05,
        description: "move logical time implicitly",
    },
    MutationSpec {
        id: "AC-05-M06",
        case: CaseId::Ac05,
        description: "omit adverse history",
    },
    MutationSpec {
        id: "AC-05-M07",
        case: CaseId::Ac05,
        description: "revoke shared trust without invalidating dependents",
    },
    MutationSpec {
        id: "AC-06-M01",
        case: CaseId::Ac06,
        description: "rename owner review as independent audit",
    },
    MutationSpec {
        id: "AC-06-M02",
        case: CaseId::Ac06,
        description: "claim independent reproduction from a second owner run",
    },
    MutationSpec {
        id: "AC-06-M03",
        case: CaseId::Ac06,
        description: "treat a signature as proof of scope",
    },
    MutationSpec {
        id: "AC-06-M04",
        case: CaseId::Ac06,
        description: "turn a synthetic fixture into certification",
    },
    MutationSpec {
        id: "AC-06-M05",
        case: CaseId::Ac06,
        description: "let absent external evidence upgrade or erase failure",
    },
    MutationSpec {
        id: "AC-07-M01",
        case: CaseId::Ac07,
        description: "average mixed outcomes",
    },
    MutationSpec {
        id: "AC-07-M02",
        case: CaseId::Ac07,
        description: "use the best target as the package result",
    },
    MutationSpec {
        id: "AC-07-M03",
        case: CaseId::Ac07,
        description: "omit unsupported members",
    },
    MutationSpec {
        id: "AC-07-M04",
        case: CaseId::Ac07,
        description: "inherit claims across implementations",
    },
    MutationSpec {
        id: "AC-07-M05",
        case: CaseId::Ac07,
        description: "let optional evidence compensate for mandatory failure",
    },
    MutationSpec {
        id: "AC-07-M06",
        case: CaseId::Ac07,
        description: "display a green profile with a non-successful child",
    },
    MutationSpec {
        id: "AC-08-M01",
        case: CaseId::Ac08,
        description: "reorder graph input",
    },
    MutationSpec {
        id: "AC-08-M02",
        case: CaseId::Ac08,
        description: "inject unknown data",
    },
    MutationSpec {
        id: "AC-08-M03",
        case: CaseId::Ac08,
        description: "truncate a bundle",
    },
    MutationSpec {
        id: "AC-08-M04",
        case: CaseId::Ac08,
        description: "exploit a path or reference",
    },
    MutationSpec {
        id: "AC-08-M05",
        case: CaseId::Ac08,
        description: "exceed a frozen resource limit",
    },
    MutationSpec {
        id: "AC-08-M06",
        case: CaseId::Ac08,
        description: "coerce an unknown version",
    },
    MutationSpec {
        id: "AC-08-M07",
        case: CaseId::Ac08,
        description: "silently map missing compiler preservation",
    },
    MutationSpec {
        id: "AC-08-M08",
        case: CaseId::Ac08,
        description: "silently reinterpret historical v0.1 satisfaction",
    },
];
