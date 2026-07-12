# Governance

Status: directed solo-project governance under D-023 and OEP-0001

## Current authority

`@chasebryan` is the Orange project owner, repository steward, sole maintainer,
implementation author, decision authority, and security contact. This operating
model is intentional and remains in force until the owner records a different
model.

Plans must assume that contributors, independent reviewers, advisers, auditors,
laboratories, partner organizations, and separate release or PSIRT personnel are
unavailable. Their participation is not a development prerequisite and no date
or deliverable may depend on them.

The single-person model cannot supply independent review, separation of duties,
multi-party key custody, external validation, or organizational bus-factor
assurance. Orange reports those properties as unavailable or not claimed. A
second pass, tool, environment, or implementation produced by the owner may
improve quality but does not become independent human evidence.

Authority descends in this order:

1. explicit direction from the project owner;
2. directed decisions in `docs/DECISIONS.md`;
3. Accepted or Provisional Orange Enhancement Proposals;
4. Accepted or Proposed Architecture Decision Records; and
5. implementation.

Implementation may explore a reversible local choice, but it cannot silently
stabilize semantics, widen a public claim, grant a license, or override a higher
record.

## Solo decision process

The owner may author, review, and approve the same change. Every such decision
is labeled `owner-approved` or `solo-reviewed`, never `independently reviewed`.
The record includes its scope, non-claims, validation, known risks, and revisit
triggers.

An unresolved decision blocks only the component or claim that depends on it.
It does not create an all-project implementation freeze. Permanent compiler
components may land incrementally under D-002 and D-024 as long as they do not
smuggle in unresolved proof, target, leakage, licensing, or release decisions.

OEP-0001 controls the solo process. OEPs remain the durable record for
project-wide behavior; ADRs capture implementation-local choices. An owner
direction may provisionally authorize bounded work before the final change has
a commit identity, but the resulting record must preserve that provisional
state.

## Claims and releases

Governance authority decides what the project will do; it does not prove that a
technical statement is true. Proof, test, audit, certification, and independent
review remain distinct evidence types. Missing external evidence is disclosed,
not waived or synthesized.

Orange is currently pre-alpha. No product release, compatibility promise,
cryptographic assurance claim, independent-review claim, or certification claim
is authorized. A later owner decision may authorize a solo release with an
explicit solo-produced evidence manifest; it still may not claim multi-party or
external controls that did not occur.

## Contributions and future participation

Third-party pull requests are not accepted while D-018 leaves contribution and
outbound license terms unresolved. Public issue reports may provide facts and
links, but the owner independently writes any repository change derived from
them.

If people later choose to participate, their involvement is welcome but not
assumed. The owner will record a governance transition before granting decision,
merge, release, PSIRT, or key-custody authority. Earlier solo work remains
historically labeled as solo work.

## Security and succession

The owner handles private vulnerability reports and repository recovery. This
is a disclosed key-person risk. Credentials must use least privilege and must
not be committed to the repository. History, source, tests, and decision records
are preserved so another maintainer could recover the project if circumstances
change.

No custodian may rewrite history to manufacture review or destroy an adverse
result. Repository transfer, archival, or succession requires explicit owner
direction while the owner remains available.
