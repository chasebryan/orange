# Orange

<img width="2172" height="724" alt="orange-banner" src="https://github.com/user-attachments/assets/ec07da49-f12a-4827-886c-dff4662b3e71" />

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
- [Decision register](docs/DECISIONS.md)

The name **Orange** is a working project name until the naming and trademark
gate in the decision register is closed. Existing software and an earlier
systems language already use the name.
