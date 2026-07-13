# Orange Enhancement Proposal process

Status: accepted solo-project process under OEP-0001 and D-023

Orange currently has one owner and no outside review authority. Accepted
OEP-0001 records the owner direction that external participation is unavailable
and must not block development. During solo mode, an owner-authored and
owner-approved record is valid project authority but is always labeled
`solo-reviewed`; it is never independent approval.

Orange Enhancement Proposals (OEPs) are the durable change-control record for
normative architecture, language semantics, assurance claims, governance,
security policy, compatibility, and ecosystem-wide behavior. An implementation
cannot substitute for an accepted OEP.

## Types and statuses

An OEP has one type:

- **Standards:** normative language, schema, ABI, target, claim, or protocol;
- **Process:** governance, security, release, or project operation;
- **Informational:** durable guidance without normative force; or
- **Emergency:** temporary containment, withdrawal, quarantine, or revocation.

Its status is exactly one of `Draft`, `Review`, `Provisional`, `Accepted`,
`Rejected`, `Withdrawn`, or `Superseded`. `Provisional` permits bounded decision
work; it is never release authority and cannot satisfy a gate that requires an
accepted decision.

## Numbering and immutability

The bootstrap steward assigns the next four-digit number after intake. Files
are named `OEP-NNNN-short-title.md` and remain at that path. Numbers are never
reused. Rejected, withdrawn, and superseded proposals remain in history with
their rationale and replacement link. Substantive decision evidence lives in
`research/decisions/D-NNN/`, not embedded as an opaque result in an OEP.

`OEP-0000-template.md` is a template, not a numbered proposal. The process
proposal should receive `OEP-0001` unless an earlier numbered intake exists.

## Required workflow

1. Open the OEP intake issue without sensitive material.
2. The steward checks scope, assigns a number, names the required review
   authorities, and opens or authorizes a Draft pull request.
3. The champion supplies reproducible evidence, primary-source provenance,
   negative cases, and the complete impact analysis in the template.
4. The proposal enters public Review only when its unresolved questions and
   decision criteria are explicit.
5. During solo mode, the project owner performs and records the review. The
   owner cannot label that review independent. If another authority actually
   exists in a later governance stage, that authority reviews within its scope.
6. The decision records disposition of alternatives and minority or dissenting
   evidence, not just the winning conclusion.
7. Acceptance updates `docs/DECISIONS.md`, affected normative documents,
   conformance expectations, and threat/control records in the same change.
8. Implementation starts when an Accepted OEP or explicit owner direction
   grants a bounded component boundary. Unresolved decisions gate only the
   components and claims that depend on them.

An Accepted record names the exact reviewed commit in `decision-revision` and
lists immutable review or authority references in `approval-records`. Its
decision date, related decisions, review authorities, and substantive decision
record must be complete. During solo mode, `review-authorities` is exactly
`Orange Project Owner`, and at least one approval record contains the literal
label `solo-reviewed`. An approval record that claims independence is invalid;
using a second spelling or role name for the same owner does not manufacture a
second principal.

The recommended minimum review periods are fourteen days for ordinary
normative proposals and thirty days for constitutional or security-model
changes. Those periods are targets until OEP-0001 ratifies exact durations and
an accountable exception process.

## Decision authority

The project owner is the decision authority during solo mode. Voting, quorum,
recusal, and committee rules do not apply while there is only one participant.
No owner decision can manufacture evidence: it may authorize work, reduce or
defer a claim, or accept a disclosed engineering risk, but it cannot turn
self-review into independent review, proof, audit, or certification.

During solo mode, the project owner may accept an OEP under the top-level
authority recorded in `GOVERNANCE.md`. The decision record must identify the
owner direction, evidence reviewed, unresolved questions, and exact text. The
approval record says `owner approval` or `solo-reviewed`; it must not imply a
second principal. If participants later join, a new governance OEP defines when
multi-person review begins and does not retroactively relabel prior decisions.

An emergency OEP may only contain risk: withdraw an artifact or profile,
quarantine a package, revoke a key, or land a minimum safe correction. It cannot
permanently widen semantics, trust, privileges, or public claims. It requires a
public retrospective and permanent record within the period later ratified by
governance; thirty days is the current recommendation.

## Mechanical validation

The repository validator checks filename/number agreement, required metadata,
status and type vocabulary, decision links, and local references. Mechanical
success does not establish technical soundness or authority approval.

## Current proposals

| OEP | Status | Scope |
| --- | --- | --- |
| [OEP-0001](OEP-0001-solo-development.md) | Accepted | Solo development and incremental capability gates |
| [OEP-0002](OEP-0002-edition-2026-parser.md) | Accepted | Orange 2026 lexical boundary, minimal grammar, and bounded parser |
| [OEP-0003](OEP-0003-orange-2026-typed-literals.md) | Provisional | Orange 2026 typed literal specifications and reference evaluation |
