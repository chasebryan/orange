# Orange

![Hand-drawn Orange carton emblem and wordmark](assets/brand/orange-handdrawn-marker-banner.png)

Orange is a language and toolchain for specifying, implementing, and verifying
cryptography.

The goal is a production system in which a cryptographic engineer can write a
mathematical specification, connect it to an efficient implementation, state
the exact assurance claims that matter, and ship native artifacts together with
machine-readable evidence, including independently checkable proofs and
certificates where the claim kind permits them.

Orange is now in solo, pre-alpha compiler development. The repository contains
the Rust compiler foundation, the accepted Orange 2026 parser slice, and an
accepted S3a typed-literal semantic slice. `orangec check` performs bounded lexical,
syntactic, and semantic validation; typed `spec` literals with exact `Int` or
`Word[8]` types lower to a deterministic, noncanonical Typed Reference Core;
and `orangec eval FILE` prints those closed values in source order.

The S3a slice has separate `spec` and `impl` name namespaces, but only
typed specifications acquire values. It defines no parameters, operators,
calls, typed implementations, refinement, proof system, canonical Core encoding,
code generation, ABI, standard library, package or release behavior, or verified
cryptographic implementation. D-003 and D-004 remain unresolved and unratified,
and later S3 semantic work is incomplete.

The accepted S3a implementation and normative documentation were merged by
[PR #9](https://github.com/chasebryan/orange/pull/9) as commit
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. That repository fact is
implementation evidence, not a stable compatibility or assurance claim.

Implemented behavior is solo-authored and solo-reviewed. It is not independently
reviewed, formally verified, production-ready, or a cryptographic assurance
claim.

## Directed commitments

- Deliver the complete language and toolchain, not a disposable prototype.
- Build incrementally through tested components of the final production
  architecture; do not plan a prototype-to-rewrite phase.
- Operate as a solo project without making development depend on unavailable
  contributors, reviewers, auditors, laboratories, or partner organizations.
- Separate implementation progress from assurance claims: missing external
  evidence is disclosed and limits claims, not unrelated development.

## Architecture direction

The current planning documents recommend the following. Individual choices are
ratified incrementally before the component or claim that depends on them.

- One language with distinct semantic strata for mathematical specifications,
  executable implementations, leakage-aware low-level code, probabilistic
  games, and proofs.
- Explicit claim reports instead of a generic `verified` label.
- Machine-readable, content-addressed evidence; proof and compilation evidence
  is independently checkable, and thick release bundles replay offline.
- A small, published trusted computing base for every kind of claim.
- Production native code, a stable C ABI, deterministic builds, and signed
  release provenance.

## Plan

- [The Orange Book](docs/THE_ORANGE_BOOK.md), the living reader guide by Chase
  Bryan
- [Project charter](docs/PROJECT_CHARTER.md)
- [Research and landscape analysis](docs/RESEARCH.md)
- [End-state architecture](docs/ARCHITECTURE.md)
- [Assurance and security model](docs/ASSURANCE.md)
- [Dependency-ordered roadmap](docs/ROADMAP.md)
- [Gate 0 feature traceability](docs/GATE0_TRACEABILITY.md)
- [Proposed Orange 1.0 user journeys](docs/USER_JOURNEYS.md)
- [D-006 proof-foundation decision suite](docs/PROOF_FOUNDATION_DECISION_SUITE.md)
- [Decision register](docs/DECISIONS.md)
- [Normative Orange 2026 lexical and grammar specification](docs/LANGUAGE_2026.md)
- [Normative Orange 2026 typed-literal semantics](docs/SEMANTICS_2026.md)
- [Solo-development process](docs/governance/oeps/OEP-0001-solo-development.md)
- [Edition 2026 parser proposal](docs/governance/oeps/OEP-0002-edition-2026-parser.md)
- [Accepted typed-literal semantics OEP](docs/governance/oeps/OEP-0003-orange-2026-typed-literals.md)
- [Compiler status and usage](compiler/README.md)

## Repository and compiler foundation

The repository carries the permanent policy and evidence architecture created
during Gate 0, the first two completed production-lineage compiler slices, and
the accepted S3a typed-literal slice. The larger S3 semantic milestone remains
incomplete:

- [governance](GOVERNANCE.md), [contribution boundary](CONTRIBUTING.md), and the
  [OEP](docs/governance/oeps/README.md) and
  [ADR](docs/governance/adrs/README.md) processes;
- [security reporting](SECURITY.md), [support](SUPPORT.md), the living
  [threat model](docs/security/THREAT_MODEL.md), and the honest
  [OSPS evidence matrix](docs/security/OSPS_BASELINE.md), backed by the
  [secrets and incident playbook](docs/security/SECRETS_AND_INCIDENTS.md);
- [dependency](DEPENDENCY_POLICY.md) and [release](RELEASE_POLICY.md) policy,
  with an honest [CI dependency inventory](docs/operations/CI_DEPENDENCIES.md);
- the [Gate 0 reproducibility contract](docs/REPRODUCIBILITY.md), provisional
  [evidence schemas](schemas/README.md), and positive/adversarial
  [conformance fixtures](conformance/foundation/README.md); and
- the machine-readable [repository policy](policy/README.md), pinned CI,
  dependency review, CodeQL default-setup record, and
  [GitHub control runbook](docs/operations/GITHUB_CONTROLS.md); and
- the owner-designated [official Orange emblem, wordmark, and lockup
  assets](assets/brand/README.md), preserved with a digest manifest and explicit
  rights boundary.

Run the deterministic repository and compiler checks with:

```sh
scripts/ci/check-repository
```

Passing this check demonstrates the scoped repository invariants, fixture
expectations, and tested compiler behavior only. It does not prove language or
compiler soundness, cryptographic correctness, a security certification, OSPS
conformance, independent review, or release readiness.

The repository has no selected outbound license under D-018 and does not accept
third-party pull requests for merge yet. Security reports must use
the private path in [SECURITY.md](SECURITY.md), never a public issue.

The name **Orange** is a working project name until the naming and trademark
gate in the decision register is closed. Existing software and an earlier
systems language already use the name.
