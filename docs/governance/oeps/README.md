# Orange Enhancement Proposal process

Status: proposed Gate 0 process. The first governance OEP must ratify or amend
this process; this document does not make its own recommendations accepted.

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
5. Required authorities review within their scope. The author cannot provide
   the independent approval for their own assurance-critical change.
6. The decision records disposition of alternatives and minority or dissenting
   evidence, not just the winning conclusion.
7. Acceptance updates `docs/DECISIONS.md`, affected normative documents,
   conformance expectations, and threat/control records in the same change.
8. Implementation starts only after every prerequisite Gate 0 OEP is Accepted.

An Accepted record names the exact reviewed commit in `decision-revision` and
lists immutable review or authority references in `approval-records`. Its
decision date, related decisions, review authorities, and substantive decision
record must be complete. OEP-0001 may use the one-time bootstrap-owner path
described below, but its approval record must explicitly preserve the missing
independent-review gap; it cannot present owner ratification as independence.

The recommended minimum review periods are fourteen days for ordinary
normative proposals and thirty days for constitutional or security-model
changes. Those periods are targets until OEP-0001 ratifies exact durations and
an accountable exception process.

## Decision authority

Consensus is preferred. Voting membership, quorum, recusals, supermajority
subjects, appeal, and succession are unresolved D-019 details and must be
ratified rather than inferred from this file. No authority may waive an
assurance stop-ship condition; it can resolve the condition, reduce the claim
or supported scope, or delay work.

During Bootstrap only, the project owner may explicitly ratify OEP-0001 under
the top-level authority recorded in `GOVERNANCE.md`. The decision record must
identify that owner direction, the evidence reviewed, unresolved independence
gaps, and the exact process text accepted. Repository stewardship or admin
access alone cannot mark it Accepted. Once OEP-0001 and D-019 establish the
normal authority, this one-time bootstrap path expires and cannot be used for
later normative proposals.

An emergency OEP may only contain risk: withdraw an artifact or profile,
quarantine a package, revoke a key, or land a minimum safe correction. It cannot
permanently widen semantics, trust, privileges, or public claims. It requires a
public retrospective and permanent record within the period later ratified by
governance; thirty days is the current recommendation.

## Mechanical validation

The repository validator checks filename/number agreement, required metadata,
status and type vocabulary, decision links, and local references. Mechanical
success does not establish technical soundness or authority approval.
