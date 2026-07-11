# Governance

Status: provisional Gate 0 governance. Decision D-019 is not ratified.

## Current authority and limitations

`@chasebryan` is the Bootstrap Repository Steward and currently holds repository
administration, custody, and decision-recording authority. This one-person
state cannot satisfy independent review, threshold release, independent PSIRT,
or bus-factor requirements. Orange therefore cannot make a production,
certified, or mature-governance release in this stage.

The steward may advance Gate 0 planning, security controls, policy, and
reproducible decision evidence. Recommendations remain `proposed` or
`investigate` until the named decision gate is actually closed. Repository
administration is not authority to bypass an assurance stop-ship condition.

Authority descends in this order:

1. explicit direction from the project owner;
2. the ratified charter and assurance constitution;
3. accepted Orange Enhancement Proposals (OEPs);
4. accepted Architecture Decision Records (ADRs); and
5. implementation.

An ADR cannot override an OEP or normative document. Implementation cannot
silently ratify a proposed decision.

## Governance stages

- **Bootstrap:** one steward; Gate 0 policy and evidence only; no independent
  assurance claim or product release.
- **Collaborative:** at least two trusted maintainers; ordinary changes require
  at least one non-author approval; conflicts and ownership are documented.
- **Release-capable:** at least three capable maintainers per critical
  subsystem, separate PSIRT and release rotations, multi-role signing and root
  recovery, and independent external assurance.

Assurance requirements are never relaxed to advance a stage.

`CODEOWNERS` routes review requests; it cannot express threshold or multi-board
approval. While it names only the bootstrap steward, it is not evidence of
independent review. As trusted people and teams become real, critical ownership
must be split among the appropriate language, cryptography, assurance, target,
release, and PSIRT authorities.

## Proposed mature authorities

Gate 0 must define non-overlapping authority for:

- a Technical Steering Council for scope, governance, funding, and succession;
- a Language and Semantics Committee;
- a Cryptography Review Board;
- an Assurance and TCB Board with assurance-gate and stop-ship authority;
- Release Engineering with build and ceremony authority;
- a PSIRT with embargo, incident, and emergency-withdrawal authority;
- standards, legal/IP, and ecosystem working groups; and
- independent advisers and auditors.

No body may vote away a stop-ship condition in `docs/ASSURANCE.md`. It may
resolve the condition, reduce the claim or supported scope, or extend the
schedule. Exact seats, terms, quorum, voting thresholds, public-review windows,
and emergency-retrospective deadlines belong in the first accepted governance
OEP, not an implementation-side convention.

At minimum, two trusted persons means participation by two trusted principals
and no author self-approval. A trusted-maintainer-authored critical change needs
one independent authorized reviewer; an outside-authored critical change needs
two authorized reviewers. TCB, cryptography, and release-system changes also
need their responsible authorities. This interpretation remains proposed until
ratified by OEP.

## Conflicts, membership, and transparency

Maintainers must disclose material funding and conflicts annually and for an
affected proposal, recuse when independence is impaired, and publish decisions,
minutes, rejected proposals, and time-bounded exceptions. Embargoed incident
details may be delayed, followed by a public retrospective.

Role criteria and effective dates must be recorded in `MAINTAINERS.md` once a
second maintainer exists. Access follows least privilege: no shared accounts,
phishing-resistant MFA where available, time-bounded credentials, prompt
offboarding, and quarterly access review. Release or offline-root keys must not
be stored as ordinary repository secrets.

## Emergency and succession rules

The PSIRT emergency path may temporarily withdraw an artifact or profile,
quarantine a package, revoke a key, or land the minimum safe fix. It cannot
permanently widen semantics or public claims. The action requires a later
permanent record and retrospective within the period ratified by governance.

Succession planning must preserve source, archives, rebuild and verification
ability, and key-recovery continuity. No custodian may unilaterally destroy the
project record. Any repository transfer requires explicit project-owner
authorization; the current operating boundary is `chasebryan/orange` only.
