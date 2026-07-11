# Decision register

Status: active proposed-decision ledger

Snapshot: 2026-07-11

This file separates user direction, research recommendations, and ratified
architecture. A recommendation is not allowed to become a hidden decision by
being implemented first.

Statuses:

- `accepted`: ratified and change-controlled;
- `directed`: explicit project direction, with design details still open;
- `proposed`: recommended answer pending the named gate;
- `investigate`: alternatives need a reproducible decision suite;
- `blocked`: an external fact or authority is required;
- `superseded`: retained for history with a replacement link.

## D-001 — Mission

Status: directed

Source: original repository README

Directed decision: Orange is a language for specifying, implementing, and
verifying cryptography.

Working product interpretation, proposed for Gate 0: the deliverable includes
the language, proof system, compiler, developer tools, package/evidence format,
standard cryptography corpus, and operational release system needed to make
that sentence true.

Open detail: Gate 0 must freeze the finite 1.0 support envelope.

## D-002 — No disposable prototype

Status: directed

Source: explicit user direction

Decision: build the end product through permanent, production-lineage
components. There is no prototype-to-rewrite phase and no MVP that postpones
the core assurance claim.

Interpretation:

- incremental integration is required;
- early components have final boundaries, tests, determinism, diagnostics, and
  documentation;
- early algorithms become permanent conformance fixtures;
- research decision cases do not become an unreviewed parallel implementation;
- incomplete pre-1.0 work is not marketed as production-ready.

Change rule: only explicit user/project-governance direction can supersede this.

## D-003 — Product form

Status: proposed for Gate 0

Recommendation: a standalone domain-specific language with its own editioned
semantics and canonical Core formats.

Alternatives considered:

- manifest-only orchestration over Cryptol/hacspec/Jasmin/EasyCrypt;
- embedded DSL in F*, Lean, or Rocq;
- Rust subset with proof annotations.

Rationale: interoperability with those systems is valuable, but delegating the
surface language and semantics would preserve the polyglot seams Orange is meant
to make explicit.

Acceptance evidence: complete user journeys, stable core boundary, and an
independent review that the standalone scope is fundable.

## D-004 — Semantic strata

Status: proposed for Gate 0

Recommendation: one module system with separate Specification, Implementation,
Machine Implementation, Game, and Proof strata, lowering to several formally
related IRs.

Rejected default: one universal IR. Mathematical totality, probabilistic games,
stateful memory, target leakage, and concrete instructions have conflicting
requirements; hiding them in annotations would make the semantics less honest.

Acceptance evidence: representative permanent decision cases for SHA-like word
code, mutable buffers, a secret-dependent rejection case, one vector intrinsic,
and one game/reduction relation.

## D-005 — Public assurance model

Status: proposed for Gate 0

Recommendation: independent named claims with statuses and full assumption/TCB
closure. Never issue one package-wide `verified` Boolean or a numeric ladder
that implies unrelated properties.

Minimum claim families: conformance, refinement, safety, termination, leakage,
compiler preservation, ABI, erasure, game-based security, and empirical tests.

Acceptance evidence: schema review by implementers, auditors, cryptographers,
and downstream integrators; demonstrate mixed statuses on a representative
artifact without ambiguity.

## D-006 — Proof foundation

Status: investigate; decide at Gate 0

Current recommendation: Rocq for the normative metatheory, Orange kernel
soundness, and verified compiler transformations.

Strong alternative: Lean 4.

Why Rocq leads today:

- the closest relevant verified compiler, crypto synthesis, Jasmin, SSProve,
  and related end-to-end research already use the ecosystem;
- mature extraction paths can produce the authoritative checker/passes;
- it reduces semantic bridge work with important reference projects.

Why Lean remains serious:

- modern integrated programming/theorem-proving environment;
- small kernel and strong proof-term discipline;
- tooling and contributor growth;
- potentially better implementation ergonomics.

Required decision suite:

- define/check the proposed Core fragment;
- mechanize progress/preservation and a leakage lemma;
- implement canonical serialization validation;
- produce and replay an LRAT-backed bitvector proof;
- exercise extraction/distribution on all supported hosts;
- measure clean bootstrap, proof replay, diagnostics, binary size, and long-term
  dependency surface;
- assess external-audit and contributor availability.

The decision is evidence-based. No surface syntax should make it irreversible
before the suite is published.

## D-007 — Orange-owned proof format and checker

Status: proposed; depends on D-006

Recommendation: define a small Orange Proof IR and kernel. Formalize the checker
in the selected proof foundation and distribute an authoritative extracted
checker, plus an independent safe-Rust checker for differential validation.

Alternative: make the host prover’s compiled environment the permanent public
artifact.

Rationale: Orange needs stable theorem fingerprints, offline inspection, bounded
checking, and evidence bundles independent of a host prover’s internal file
format and release cadence.

Risk: a custom kernel is a major soundness and schedule risk. The logic must be
smaller than the surface language and explicitly exclude convenient features
that would enlarge the TCB.

Acceptance evidence: mechanized soundness, two checkers, malformed/mutation
fuzzing, external logic audit, stable canonical encoding.

## D-008 — Implementation languages

Status: proposed for Gate 0

Recommendation:

- safe Rust for the permanent driver, frontend services, package tooling, LSP,
  and independent checker;
- selected proof foundation for normative semantics, authoritative checker, and
  verified stable compiler passes;
- no mandatory self-hosting target.

Rationale: Rust is suitable for hostile-input tooling and distribution, while
the checker/passes need a direct mechanized relationship. Orange is too
specialized to gain from adding general application features merely to
self-host its package manager or LSP.

Acceptance evidence: bootstrap and distribution design, license/dependency
audit, and proof that cross-language canonical boundaries are checkable.

## D-009 — Solver trust

Status: proposed for Gate 0

Recommendation: solvers are untrusted search/counterexample engines in
certified mode. Successful automated claims require checked certificates or
Orange proof terms.

Initial portfolio:

- verified bit-blasting plus LRAT-family SAT certificates;
- reflective algebra/range procedures;
- an explicitly ratified proof format such as Alethe for supported SMT
  fragments only;
- external EasyCrypt/SSProve evidence labeled as external until reconstructed.

Timeout, `unknown`, missing proof output, resource exhaustion, or a trusted/
unsupported certificate step leaves the claim outcome `unresolved` and records
the exact diagnostic reason.

Acceptance evidence: negative tests prove every failure mode fails closed and
no solver executable is in the native logical TCB.

## D-010 — Compiler strategy

Status: proposed for Gate 0 architecture, final pass policy at Gate 1

Recommendation: hybrid verified compilation and translation validation.

- Prove stable structural passes once.
- Let optimizers, schedulers, vectorizers, and allocators search outside the TCB
  only when each accepted result carries a checked functional and leakage
  certificate.
- Build an Orange Machine IR and direct supported native-object path.
- Decode and validate final bytes, sections, relocations, constants, and symbols.

Alternatives:

- permanently target Jasmin;
- target C/LLVM and stop the claim there;
- prove every optimization implementation.

Rationale: the hybrid preserves a small assurance boundary without freezing
performance research. Jasmin/SAW/mature libraries remain valuable independent
oracles and interoperability targets.

Acceptance evidence: one end-to-end target case whose final object passes both
functional and leakage preservation, including a deliberately corrupted object
that is rejected.

## D-011 — Initial native target envelope

Status: proposed; decide at Gate 0

Recommendation:

- certified target tuples: x86-64 Linux/SysV and AArch64 Linux/AAPCS64;
- explicit baseline and selected crypto/SIMD feature profiles;
- host tools for current Linux, macOS, and Windows;
- stable generated C ABI and Rust wrapper;
- portable C output clearly labeled as an interoperability path.

Deferred unless Gate 0 substitutes them: RISC-V certified target, Windows and
macOS certified native outputs, general Wasm constant-time claims, GPUs, and
hardware synthesis.

Rationale: x86-64 and AArch64 cover the principal server and client CPU families
while keeping target verification finite. The team and lab capacity may require
Gate 0 to choose only one for 1.0 rather than weaken both.

Acceptance evidence: resource estimate per target, ISA/ABI model availability,
hardware/lab access, and flagship-corpus feasibility.

## D-012 — Baseline leakage claim

Status: investigate; decide at Gate 0

Recommendation: two-run architectural noninterference covering branches,
addresses/widths, indirect targets, traps, termination, and target-classified
variable-latency operations, with explicit declassification.

Separate future profiles cover speculative execution, architectural DIT modes,
power/EM, masking, and fault resistance.

Acceptance evidence: formal trace semantics, target instruction-classification
process, positive and negative examples, preservation plan through final bytes,
and independent side-channel review.

## D-013 — Stable foreign boundary

Status: proposed; finalize by Gate 1

Recommendation: generated C ABI plus a machine-readable contract, with safe
Rust wrappers above it.

The contract includes lengths, alignment, overlap, mutability, initialization,
layout, failure, target features, ownership, zeroization, and entropy behavior.
No hidden allocator, panic, exception, TLS state, or RNG exists in the kernel
boundary.

Acceptance evidence: ABI model and adversarial callers for each supported target
tuple; generated header/wrapper/object all derive from one definition.

## D-014 — Package and registry model

Status: proposed; local format by Gate 2, public registry by Gate 5

Recommendation:

- human manifest plus immutable generated lock;
- proofs bind to exact content and theorem fingerprints;
- published versions immutable, with yanking affecting new resolution only;
- certified graphs forbid arbitrary native build scripts/plugins;
- offline resolution/replay from a thick bundle or populated local
  content-addressed store is mandatory; a lockfile alone is insufficient;
- public registry uses TUF-style delegated/threshold metadata, MFA, recovery,
  namespace governance, quarantine, and revocation.

Acceptance evidence: dependency-confusion, takeover, rollback, freeze, yanking,
offline, and compromised-key exercises.

## D-015 — Flagship 1.0 corpus

Status: proposed set; decide exact membership at Gate 0

Recommended coverage set:

- SHA-256/512;
- ChaCha20-Poly1305;
- HMAC/HKDF;
- AES-GCM with portable and selected accelerated variants;
- X25519 and/or Ed25519;
- ML-KEM;
- one selected post-quantum signature family if resources allow.

Rationale: each family exercises a different permanent language/compiler/proof
capability. Breadth is subordinate to a complete claim matrix.

Scope rule: under-resourcing removes a family or target rather than removing the
proof, leakage, binary, interop, or response gates while retaining the claim.

Acceptance evidence: claim matrix, standards/errata/vector sources, formal and
compiler workload estimate, target benchmarks, independent reference library,
and maintainer ownership for each family.

## D-016 — Validation and certification posture

Status: proposed; external-lab requirement depends on product goals

Decision recommendation:

- support NIST ACVP-compatible input/output and record validation status;
- never call local vectors or proof replay an ACVP/CAVP certificate;
- never call Orange itself FIPS 140 validated;
- if a certificate-bearing module is a 1.0 requirement, engage an accredited
  laboratory during Phase 0 and design the module/entropy/runtime/self-test
  boundaries with it.

Acceptance evidence: lab scope and budget, module boundary, change/revalidation
strategy, and approved public wording.

## D-017 — Project and package name

Status: blocked on naming/trademark/namespace review; decide at Gate 0

Current state: **Orange** is the working codename and repository name.

Evidence of collision:

- long-running Orange data-mining/visual-programming software;
- earlier `orange-lang/orange` systems language;
- broad commercial use of “Orange.”

Required review:

- professional trademark/legal search in intended jurisdictions and classes;
- command, package, domain, organization, documentation, and social namespace
  availability;
- searchability and confusion analysis;
- codename-to-final-name migration cost.

Do not publish packages or invest in a final visual identity before this closes.

## D-018 — Licenses

Status: blocked on owner/legal decision; decide at Gate 0

Working recommendation for review:

- permissive compiler/toolchain license, likely Apache-2.0 with patent terms or
  dual Apache-2.0/MIT;
- an explicit generated-output exception/statement so Orange does not impose a
  license on user artifacts;
- documentation and language specification license that permits independent
  implementations and quotation;
- vector/standards provenance preserved according to source terms;
- contribution terms compatible with future neutral governance and patent
  defense.

Do not add a license based solely on this recommendation. Dependency and proof-
library license compatibility must be audited first.

## D-019 — Governance and release authority

Status: proposed; decide structure/funding at Gate 0

Recommendation: move toward neutral, transparent governance with language,
cryptography, assurance/TCB, release, and PSIRT authorities. Critical changes
require two trusted persons; authors do not self-approve TCB, cryptography, or
release-system work; releases use threshold multi-role approval.

Acceptance evidence: charter, conflict/funding disclosure, escalation and
security authority, maintainer succession, separation of duties, and funded LTS.

## D-020 — Supply-chain target

Status: proposed; versions pinned at each release

Recommendation:

- NIST SSDF 1.1 baseline until a newer final edition is ratified;
- SLSA 1.2 Source L4 and Build L3 for release source/artifacts;
- current OpenSSF OSPS Baseline Level 3;
- additional Orange requirement for network-disabled, fully declared,
  reproducible release builds;
- Sigstore or equivalent signature/transparency evidence plus TUF-style update
  recovery;
- SPDX SBOM and CycloneDX SBOM/CBOM.

Acceptance evidence: continuous policy checks and successful independent build,
rollback, freeze, compromise, revocation, and disaster-recovery drills.

## D-021 — Self-hosting

Status: proposed

Recommendation: self-host only components for which Orange is naturally suited.
Keep the safe-Rust bootstrap/frontend and authoritative formal checker path
published and supported. Do not add general-purpose language features solely to
self-host networking, registry, or editor code.

If a compiler core becomes self-hosted, require reproducible diverse double
compilation or equivalent evidence and retain the prior bootstrap until
independent audits/rebuilds pass.

## D-022 — Support policy

Status: proposed; funding decision at Gate 0, final by Gate 6

Recommendation: five years of full LTS support for Language Edition 1/toolchain
line plus two years critical-security-only. A security-driven algorithm or
target-profile withdrawal may override normal deprecation windows. Ordinary
deprecations receive at least twelve months when safety permits.

Acceptance evidence: funded maintainers, rotations, archival/rebuild capacity,
downstream notification, and end-of-life/migration plan.

## How decisions change

An accepted decision changes through an Orange Enhancement Proposal or the
equivalent governance process. The proposal must state semantic, TCB, threat,
compatibility, conformance, migration, standards, IP, and schedule effects. A
security emergency can use a fast path, but it receives a time-bounded public
retrospective and permanent decision record.
