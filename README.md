# Orange

Orange is a language and toolchain for specifying, implementing, and verifying
cryptography.

The goal is a production system in which a cryptographic engineer can write a
mathematical specification, connect it to an efficient implementation, state
the exact assurance claims that matter, and ship native artifacts together with
machine-readable evidence, including independently checkable proofs and
certificates where the claim kind permits them.

Orange is at the research and architecture stage. There is no compiler,
standard library, or verified cryptographic implementation in this repository
yet. The documents below define the intended end product and the permanent path
to it; they do not describe features that already exist.

## Directed commitments

- Deliver the complete language and toolchain, not a disposable prototype.
- Build incrementally through tested components of the final production
  architecture; do not plan a prototype-to-rewrite phase.

## Proposed Gate 0 baseline

The current planning documents recommend the following. These become project
commitments only when the Gate 0 decisions are ratified.

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

- [Project charter](docs/PROJECT_CHARTER.md)
- [Research and landscape analysis](docs/RESEARCH.md)
- [End-state architecture](docs/ARCHITECTURE.md)
- [Assurance and security model](docs/ASSURANCE.md)
- [Dependency-ordered roadmap](docs/ROADMAP.md)
- [Gate 0 feature traceability](docs/GATE0_TRACEABILITY.md)
- [Proposed Orange 1.0 user journeys](docs/USER_JOURNEYS.md)
- [D-006 proof-foundation decision suite](docs/PROOF_FOUNDATION_DECISION_SUITE.md)
- [Decision register](docs/DECISIONS.md)

## Gate 0 repository foundation

The repository now carries the permanent policy and evidence architecture used
to complete Gate 0 without beginning a disposable product implementation:

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
  [GitHub control runbook](docs/operations/GITHUB_CONTROLS.md).

Run the deterministic local policy and adversarial suite with:

```sh
make check
```

Passing this check proves the scoped Gate 0 repository invariants and fixture
expectations only. It does not prove the future language, compiler,
cryptographic correctness, a security certification, OSPS conformance, or
release readiness.

The repository has no selected license while D-018 remains blocked and does
not accept third-party pull requests for merge yet. Security reports must use
the private path in [SECURITY.md](SECURITY.md), never a public issue.

The name **Orange** is a working project name until the naming and trademark
gate in the decision register is closed. Existing software and an earlier
systems language already use the name.
