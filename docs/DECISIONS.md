# Decision register

Status: active proposed-decision ledger

Snapshot: 2026-07-12

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

Under D-023, remaining `Gate 0` phrases are legacy decision-stage labels, not
an aggregate implementation barrier. Each unresolved decision gates only the
component or claim that depends on it.

## D-001 — Mission

Status: directed

Source: original repository README

Directed decision: Orange is a language for specifying, implementing, and
verifying cryptography.

Working product interpretation for incremental capability decisions: the
deliverable includes the language, proof system, compiler, developer tools,
package/evidence format, standard cryptography corpus, and operational release
system needed to make that sentence true.

Open detail: incremental capability decisions must keep the support envelope
finite. D-023 supersedes the former requirement to freeze all of 1.0 before any
implementation begins.

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

Status: proposed; decide before S2/S3 stabilizes the standalone language and
Core boundary

Recommendation: a standalone domain-specific language with its own editioned
semantics and canonical Core formats.

Alternatives considered:

- manifest-only orchestration over Cryptol/hacspec/Jasmin/EasyCrypt;
- embedded DSL in F*, Lean, or Rocq;
- Rust subset with proof annotations.

Rationale: interoperability with those systems is valuable, but delegating the
surface language and semantics would preserve the polyglot seams Orange is meant
to make explicit.

Acceptance evidence: complete user journeys, a stable Core boundary, and an
owner-executable scope and resource analysis. Independent feasibility review is
unavailable in solo mode; its absence limits any external feasibility claim but
does not block proof-neutral frontend work.

## D-004 — Semantic strata

Status: proposed; decide before S3 stabilizes the semantic strata and Core

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

Status: proposed; decide before S4 stabilizes the claim model and public claim
records

Recommendation: separately named claims with statuses and full assumption/TCB
closure. Never issue one package-wide `verified` Boolean or a numeric ladder
that implies unrelated properties.

Minimum claim families: conformance, refinement, safety, termination, leakage,
compiler preservation, ABI, erasure, game-based security, and empirical tests.

Acceptance evidence: owner-executable schema review and a representative
artifact with unambiguous mixed statuses. Multidisciplinary external review by
implementers, auditors, cryptographers, and downstream integrators is
unavailable in solo mode; any claim that depends on it remains unsupported,
without blocking unrelated S1-S3 development.

## D-006 — Proof foundation

Status: investigate; required before proof-bearing components, not the frontend

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
  dependency surface; and
- record that external-audit and contributor availability are unavailable in
  solo mode and therefore cannot distinguish the candidates.

The symmetric cases, measurements, hard gates, archive, and inconclusive
procedure are specified in the
[D-006 proof-foundation decision suite](PROOF_FOUNDATION_DECISION_SUITE.md).

The decision is evidence-based. D-023 permits proof-neutral compiler work while
this remains open. No source or Core choice may make a proof foundation
irreversible before the owner records a revised, solo-executable comparison.

## D-007 — Orange-owned proof format and checker

Status: proposed; depends on D-006

Recommendation: define a small Orange Proof IR and kernel. Formalize the checker
in the selected proof foundation and distribute an authoritative extracted
checker, plus an implementation-diverse safe-Rust checker for differential
validation.

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

Status: directed for the solo compiler bootstrap

Directed decision:

- safe Rust, Rust edition 2024, for the permanent driver, frontend services,
  package tooling, LSP, and implementation-diverse checker;
- selected proof foundation for normative semantics, authoritative checker, and
  verified stable compiler passes;
- no mandatory self-hosting target.

The initial slice pins the Rust toolchain and uses only the standard library.
New crates require an explicit dependency admission record. The words
`independent checker` remain reserved for organizational independence; a second
checker written by the owner is an implementation-diverse checker.

Rationale: Rust is suitable for hostile-input tooling and distribution, while
the checker/passes need a direct mechanized relationship. Orange is too
specialized to gain from adding general application features merely to
self-host its package manager or LSP.

This decision authorizes the proof-neutral compiler foundation in D-024. The
proof foundation and cross-language canonical boundary remain open and gate
only components that depend on them.

## D-009 — Solver trust

Status: proposed; decide before S4 admits solver-backed proof search

Recommendation: solvers are untrusted search/counterexample engines in
claim-closing mode. Successful automated claims require checked certificates or
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

Status: proposed; decide the IR strategy before S5 and each final pass policy
before that pass can carry a preservation claim

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

Status: proposed; decide before target implementation or native-code claims

Recommendation:

- initial assurance target tuples: x86-64 Linux/SysV and AArch64 Linux/AAPCS64;
- explicit baseline and selected crypto/SIMD feature profiles;
- host tools for current Linux, macOS, and Windows;
- stable generated C ABI and Rust wrapper;
- portable C output clearly labeled as an interoperability path.

Deferred unless an incremental target decision substitutes them: a RISC-V
assurance target, claim-bearing Windows and macOS native outputs, general Wasm
constant-time claims, GPUs, and hardware synthesis.

Rationale: x86-64 and AArch64 cover the principal server and client CPU families
while keeping target verification finite. Solo capacity and available target
model evidence may require the owner to choose only one for 1.0 rather than
weaken both.

Acceptance evidence: resource estimate per target, ISA/ABI model availability,
owner-accessible hardware evidence, and flagship-corpus feasibility.

The active solo capacity boundary that governs future target admission is
recorded in the [solo development envelope](GATE0_SUPPORT_ENVELOPES.md).

## D-012 — Baseline leakage claim

Status: investigate; decide before S6 stabilizes leakage semantics or makes any
constant-time claim

Recommendation: two-run architectural noninterference covering branches,
addresses/widths, indirect targets, traps, termination, and target-classified
variable-latency operations, with explicit declassification.

Separate future profiles cover speculative execution, architectural DIT modes,
power/EM, masking, and fault resistance.

Acceptance evidence: formal trace semantics, target instruction-classification
process, positive and negative examples, preservation plan through final bytes,
and an explicit review status. Independent side-channel review is unavailable
in solo mode, so claims that require it remain unsupported; this does not block
earlier proof-neutral compiler capabilities.

## D-013 — Stable foreign boundary

Status: proposed; finalize before S6 implements a foreign boundary or makes ABI
claims

Recommendation: generated C ABI plus a machine-readable contract, with safe
Rust wrappers above it.

The contract includes lengths, alignment, overlap, mutability, initialization,
layout, failure, target features, ownership, zeroization, and entropy behavior.
No hidden allocator, panic, exception, TLS state, or RNG exists in the kernel
boundary.

Acceptance evidence: ABI model and adversarial callers for each supported target
tuple; generated header/wrapper/object all derive from one definition.

## D-014 — Package and registry model

Status: proposed; decide the local format before S8 package tooling; decide any
public registry only through an explicit future release/distribution decision

Recommendation:

- human manifest plus immutable generated lock;
- proofs bind to exact content and theorem fingerprints;
- published versions immutable, with yanking affecting new resolution only;
- claim-bearing graphs forbid arbitrary native build scripts/plugins;
- offline resolution/replay from a thick bundle or populated local
  content-addressed store is mandatory; a lockfile alone is insufficient;
- public registry uses TUF-style delegated/threshold metadata, MFA, recovery,
  namespace governance, quarantine, and revocation.

Acceptance evidence: dependency-confusion, takeover, rollback, freeze, yanking,
offline, and compromised-key exercises.

## D-015 — Flagship 1.0 corpus

Status: proposed set; decide exact membership before S7 admits the corpus

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
compiler workload estimate, target benchmarks, owner-executable comparisons
against mature reference implementations, and an explicit solo ownership and
resource record for each family. Independent human review and separate
maintainer ownership are unavailable in solo mode; their absence limits the
admissible claims rather than unrelated compiler development.

The active solo capacity boundary that governs future corpus admission is
recorded in the [solo development envelope](GATE0_SUPPORT_ENVELOPES.md).

## D-016 — Validation and certification posture

Status: proposed; external-lab requirement depends on product goals

Decision recommendation:

- support NIST ACVP-compatible input/output and record validation status;
- never call local vectors or proof replay an ACVP/CAVP certificate;
- never call Orange itself FIPS 140 validated;
- keep certificate-bearing profiles unsupported in the current solo operating
  model; only a future explicit operating-model change with an actually
  available accredited laboratory may open such a profile decision.

Acceptance evidence: lab scope and budget, module boundary, change/revalidation
strategy, and approved public wording.

Those acceptance items apply only to a future certificate-bearing profile.
Their current unavailability limits certification claims and does not block
development of non-certificate capabilities.

## D-017 — Project and package name

Status: directed working codename for solo development; public naming remains open

Current state: **Orange** is the working codename and repository name. The
Bootstrap Steward designated the byte-preserved images under
[`assets/brand/`](../assets/brand/) as the official working repository emblem,
wordmark, and lockup on 2026-07-11. That designation records current project
identity; it is not trademark clearance or ratification of the final name.

Evidence of collision:

- long-running Orange data-mining/visual-programming software;
- earlier `orange-lang/orange` systems language;
- broad commercial use of “Orange.”

Unavailable review inputs:

- professional trademark/legal search in intended jurisdictions and classes;
- command, package, domain, organization, documentation, and social namespace
  availability;
- searchability and confusion analysis;
- codename-to-final-name migration cost.

The owner directs use of **Orange** and `orangec` for repository-local solo
development. This is not trademark clearance and does not authorize package,
domain, or registry publication. Preserve the admitted originals and their
provenance so a later naming decision can migrate or retire them deliberately.

## D-018 — Licenses

Status: directed solo-development boundary; outbound license remains open

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

No repository-wide license or contribution grant is selected. The owner may
author and run Orange code in this repository, and that unresolved outbound
license does not block owner-authored implementation. Third-party contributions,
crate publication, binary distribution, and redistribution claims remain
blocked until the owner records appropriate terms. Dependencies require an
owner admission record; the initial compiler uses no third-party Rust crates.

## D-019 — Governance and release authority

Status: directed solo-project governance

Decision: `@chasebryan` is the sole project, implementation, review, merge,
security, and decision authority until explicit owner direction changes the
model. Plans must assume no contributors, independent reviewers, auditors,
laboratories, partner organizations, or separate operational roles.

Owner approval is valid governance disposition but is never independent
evidence. Missing separation of duties, bus factor, external review, and
multi-party custody are disclosed limitations rather than development blockers.
See D-023, OEP-0001, and `GOVERNANCE.md`.

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

Status: directed best-effort solo support until a release decision

Decision: pre-alpha solo development has no SLA, LTS window, compatibility
promise, or migration-service promise. Support is best effort by the owner.
A release-specific support window may be adopted only when the owner can
actually sustain it; the former five-plus-two-year institutional target is not
an active commitment.

Security-driven withdrawal may be immediate. Every release, if one is later
authorized, must state its actual support dates and single-maintainer risk.

## D-023 — Solo project operating model

Status: directed

Source: explicit project-owner direction on 2026-07-12

Decision: Orange is developed as a solo project until the owner explicitly
records otherwise. All current and future planning must treat outside human or
organizational participation as unavailable. No milestone may depend on
contributors, independent reviewers, auditors, laboratories, partner
organizations, or separate release and incident-response roles.

The former aggregate Gate 0 implementation embargo is superseded. Work proceeds
through incremental capability gates. An unresolved decision blocks only the
component or claim that relies on it. The absence of independent or external
evidence must be reported honestly, but it does not block unrelated work.

This decision does not convert owner review into independent review, waive a
technical proof obligation, grant certification, select a license, or authorize
a release. If participation later becomes real, the owner may amend the model;
earlier evidence remains labeled solo-produced.

## D-024 — Initial compiler foundation

Status: directed

Source: explicit project-owner direction to begin compiler work on 2026-07-12

Decision: begin the permanent Orange compiler lineage in Rust. The first bounded
slice contains source identity and byte spans, deterministic UTF-8 lexing,
structured stable diagnostics, and the `orangec` command-line boundary. It pins
the Rust toolchain, uses Rust edition 2024, and admits no third-party crates.

This slice may reserve clearly documented tokens but does not ratify the full
grammar, Core semantics, proof foundation, target model, ABI, or leakage model.
It performs no native code generation and carries no proof, cryptographic,
constant-time, compatibility, support, or production-readiness claim.

Acceptance evidence for the slice is deterministic formatting and linting,
unit and CLI tests covering positive and malformed input, stable diagnostics,
an exact source inventory, and green repository policy checks. Later slices add
their own decisions and do not inherit claims from this one.

## D-025 — Orange 2026 minimal grammar and bounded parser

Status: directed

Source: explicit project-owner direction for S2 on 2026-07-12; accepted
OEP-0002 at exact revision `52a3460853636f7cbaa27f3e27d86e032e3c82d4`

Decision: define the first Orange 2026 syntax as valid UTF-8 of at most 16 MiB,
with ASCII whitespace and identifiers, a mandatory exact `edition 2026;`
declaration, exactly one named module, and zero or more empty `spec` or `impl`
function declarations. The complete grammar is:

```text
source_file   = edition_decl module_decl EOF ;
edition_decl  = "edition" "2026" ";" ;
module_decl   = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl = function_kind IDENTIFIER "(" ")" empty_body ;
function_kind = "spec" | "impl" ;
empty_body    = "{" "}" ;
```

Line feed, carriage-return line feed, and bare carriage return each form one
logical line ending. `edition` is reserved with the existing Orange 2026
keywords. `game`, `proof`, and `claim` remain lexical reservations only.
Duplicate member names are syntactically valid because name resolution is not
part of parsing.

The parser is deterministic and bounded by exact token, syntax-node, event,
diagnostic, and recovery-depth limits in
[`LANGUAGE_2026.md`](LANGUAGE_2026.md). Lexically invalid input is not parsed;
recovery may improve diagnostics but never converts a malformed source into
success.

This slice explicitly does not define parameters, types, expressions, non-empty
bodies, imports, multiple modules, semantics, proofs, targets, ABI, leakage,
code generation, packages, or releases. Syntactic acceptance makes no claim
about any of them and does not settle D-003 through D-006 or D-009 through
D-016.

Acceptance evidence is the normative lexical and grammar document, exact source
inventory, positive and malformed parser tests, ambiguity and duplicate-name
cases, Unicode and line-ending cases, resource-limit tests, stable diagnostics,
repeatability, offline locked Rust checks, repository policy checks, and green
required hosted CI. Acceptance required the exact merged S2 revision; that
condition closed on 2026-07-12. OEP-0002 is Accepted at
exact merged revision `52a3460853636f7cbaa27f3e27d86e032e3c82d4` after its
required hosted checks passed.

## How decisions change

An accepted decision changes through an Orange Enhancement Proposal or the
equivalent governance process. The proposal must state semantic, TCB, threat,
compatibility, conformance, migration, standards, IP, and schedule effects. A
security emergency can use a fast path, but it receives a time-bounded public
retrospective and permanent decision record.
