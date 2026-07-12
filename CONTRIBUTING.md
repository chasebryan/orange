# Contributing to Orange

Orange is a solo, pre-alpha project with active production-lineage compiler
development. OEP-0001 and D-023 replace the former all-project Gate 0
implementation freeze with incremental capability gates. Work must still
advance a permanent end-product boundary rather than a disposable prototype.

## Current contribution and legal boundary

[Decision D-018](docs/DECISIONS.md#d-018--licenses) deliberately leaves source,
documentation, generated-output, specification, vector, patent, and inbound
contribution terms unresolved. Until that owner/legal decision closes, the
repository does **not** accept third-party pull requests for merge. This avoids
silently imposing terms on contributors or the future project.

External researchers and prospective contributors are welcome to open issues
containing facts, primary-source links, reproducible methods, and proposed
scope. Do not contribute original code or prose in an issue. The bootstrap
steward may author repository work during solo pre-alpha development.
Issue-form answers are intake summaries for evaluation, not text licensed for
incorporation; until D-018 closes, a maintainer must author any repository
change afresh from the cited facts and sources without incorporating the
issue's original expression.

If D-018 later selects contribution terms, this file must name the repository
licenses, generated-
output policy, patent and provenance terms, and either DCO 1.1 sign-off or a
counsel-selected contributor agreement. A DCO sign-off is a legal provenance
statement; it is distinct from cryptographically signing a commit.

Required CI supplies a blocking defense-in-depth signal by failing ordinary
pull requests whose opener is not `chasebryan`; it does not prove commit or
content authorship and cannot replace legal/provenance review. Dependabot pull
requests are surveillance and update suggestions only during solo mode; they
cannot be merged. The steward must review the upstream change and author any
admitted pin update afresh with its provenance and validation.

## Solo-development scope

Owner-authored work may include:

- factual corrections supported by primary sources;
- planning, governance, security, assurance, and threat-model improvements;
- standards, dependency, license, and provenance inventories;
- reproducible permanent decision evidence under `research/decisions/` once a
  decision case has a substantive artifact;
- permanent compiler, diagnostics, parser, semantic, and code-generation
  components admitted through incremental decisions; and
- tests, fixtures, documentation, and developer tooling for implemented
  behavior.

Do not add:

- a disposable prototype, demo-only scaffold, or parallel MVP;
- syntax or architecture selected merely by implementing it first;
- packages, releases, unratified replacement/final branding, or public
  namespaces (the steward-designated working assets are admitted only through
  the exact repository policy);
- empty directories for a future architecture;
- opaque binaries or generated executables; or
- dependencies that silently decide licensing, proof-foundation, target, or
  trust-boundary questions.

## Workflow

1. Open the matching issue form. Security vulnerabilities use the private path
   in [SECURITY.md](SECURITY.md), not an issue.
2. Classify the work as a defect, research evidence, an OEP, an ADR, or an
   emergency containment action.
3. Link the affected D-ID, OEP, or normative document before implementation.
4. Work on a branch and open a pull request using the repository template.
5. Record exact validation commands, evidence, provenance, and limitations.
6. Record owner review as `solo-reviewed`. Never label it independent review.
7. Update the decision register and coupled documents in the same change.

## Definition of done

A change belongs at its intended permanent boundary; is deterministic and
documented; includes tests or replayable evidence where applicable; cites
primary-source provenance; introduces no hidden capability decision; and declares
all effects on the TCB, threats, claims, dependencies, licenses, compatibility,
and support. Local links, formatting, whitespace, schemas, and policy checks
must pass.

## Generated and AI-assisted material

Disclose material generated or AI-assisted work, including the tool and model
or version when known and which portions it affected. The human author remains
accountable for correctness, security, provenance, and review. Never put
secrets, unreleased vulnerabilities, or proprietary inputs into an external
tool. Do not publish private prompts merely to satisfy disclosure.

Generated text is not proof, a primary source, or license provenance. Verify
citations and third-party rights independently. A proof-required claim closes
only with replayable evidence accepted by its authoritative checker.
