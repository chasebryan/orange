# End-state architecture

Status: proposed end-state architecture with active pre-alpha frontend

Audience: project owner and future language, compiler, formal-methods,
cryptography, and tooling readers

Research snapshot: 2026-07-11

Solo/compiler amendment: 2026-07-12

D-023 through D-025 authorize a proof-neutral Rust compiler foundation and the
minimal Orange 2026 parser. D-026 and provisional OEP-0003 add one bounded
semantic foothold: closed typed `spec` literals, separate declaration-kind
namespaces, exact contextual `Int` and `Word[8]`, a deterministic Typed Reference
Core, and reference evaluation. [`SEMANTICS_2026.md`](SEMANTICS_2026.md) is the
normative provisional boundary.

The Typed Reference Core has no canonical encoding, proof identity, refinement
relation, target, ABI, cryptographic, or leakage meaning. D-003 and D-004 remain
unratified, and unresolved architecture choices continue to gate only the
component or claim that depends on them.

## 1. Architecture objective

Orange is a standalone proof-carrying compiler and cryptographic engineering
environment. One coherent source language is presented to users, but several
formally related semantic layers are kept internally because no single IR can
honestly serve all of these needs:

- total mathematical computation;
- probabilistic games and adversaries;
- stateful, memory-safe implementation;
- leakage-aware low-level optimization;
- concrete ISA behavior and ABI layout; and
- compact, independently checkable evidence.

The architectural endpoint is not generated source code. It is a native or
interoperability artifact plus an evidence graph that states exactly how the
artifact relates to specifications and claims.

```text
 standards + errata + vectors
              |
              v
        Orange source modules
       /          |           \
      v           v            v
 Spec Core     Impl Core     Game Core
                   |
                   v
                 CT IR
                   |
          verified/validated passes
                   |
                   v
               Machine IR
                   |
        checked encoding and object validation
                   |
                   v
         object/library + generated C ABI

 Spec Core + Impl Core + Game Core + every transition relation
                   |
                   v
              Claim Graph
                   |
                   v
               obligations <---- certificate-producing automation
                   |
                   v
                Proof IR
                   |
                   v
            authoritative checker

 object + claims + checked evidence + recorded external evidence
                   |
                   v
        content-addressed evidence bundle
```

Every formal-preservation arrow is either:

1. part of the normative semantics;
2. justified by a kernel-checked theorem;
3. accompanied by a checked per-artifact certificate; or
4. recorded as an explicit external assumption.

There is no fifth formal category called “obvious glue.” Empirical tests,
audits, and external validations attach to a precisely scoped claim as separate
evidence bases; they do not close a formal preservation obligation.

## 2. Component boundary

### 2.1 `orange`

The main driver coordinates builds, package resolution, evaluation, proof
search, compilation, documentation, and evidence assembly. It is expected to be
implemented in safe Rust with narrow, audited platform bindings.

The driver is not logically trusted merely because it invokes trusted tools. It
can be wrong without forging an accepted proof if all durable boundaries use
canonical checked formats.

### 2.2 `orange-check`

The authoritative offline checker accepts canonical Orange Core, claims, proof
objects, transformation certificates, and target models. It has:

- no network access;
- no registry or package resolution;
- no tactics, plugins, or code generation;
- deterministic, resource-bounded behavior;
- one explicit set of supported format versions; and
- a command that prints every axiom and trusted model in a claim closure.

The recommended implementation is an Orange-owned kernel and checker specified
and proved sound in Rocq, with an extracted authoritative executable. An
implementation-diverse safe-Rust checker should be maintained for differential
testing and ecosystem resilience. The proof format must not be a serialized
compiler heap or a solver transcript.

This proof-foundation choice remains a ratification gate. Lean 4 is the strongest
alternative and must be compared with the same decision suite before product
code is committed to either ecosystem. Competing mechanized cases are archived
as reproducible Gate 0 research evidence; the selected case graduates into the
production metatheory suite, while rejected cases never become a second product
implementation.

### 2.3 `orange-compile`

The compiler elaborates checked source into the IR family, performs verified
passes or produces translation certificates for untrusted passes, and emits
Machine IR plus object-code evidence.

Rapidly changing search procedures—superoptimization, scheduling, vectorization,
register allocation—may remain untrusted if the accepted result is validated
for both functional and leakage refinement. Stable structural passes should be
proved once.

### 2.4 `orange-lsp`

The language server, CLI, and editor integrations use one incremental compiler
database. They must not reimplement typing or claim logic. The LSP is not in the
proof TCB, but it is a security-sensitive parser of untrusted projects and must
have normal production hardening and resource limits.

### 2.5 `orange-registry` and package client

The package system resolves content-addressed, immutable packages and theorem
fingerprints. Claim-bearing package graphs prohibit arbitrary native build scripts
and compiler plugins. The registry is not required for offline build or proof
replay when a thick bundle or populated local object store supplies every
addressed byte.

### 2.6 Orange cryptography corpus

The standard and flagship cryptography packages are first-party acceptance
artifacts. They contain standards provenance, specs, games, implementations,
proofs, vectors, bindings, benchmarks, and claim manifests. They are developed
with the compiler, not postponed until the compiler is “done.”

## 3. Surface-language model

Orange uses visibly distinct declaration kinds within one editioned module
system. D-025 defines the first pre-alpha Orange 2026 grammar, and D-026
additively preserves its empty declarations while permitting
`spec NAME() -> TYPE { SIGNED_INTEGER }`. The parser accepts a generic type shape;
semantic analysis accepts only exact contextual `Int` and `Word[8]` and rejects
same-kind duplicate names. A same-named `spec` and `impl` is permitted because
their namespaces are separate.

[`LANGUAGE_2026.md`](LANGUAGE_2026.md) is authoritative for current lexical and
parsing behavior. [`SEMANTICS_2026.md`](SEMANTICS_2026.md) defines the
provisional typed-literal meaning. Empty declarations retain no type or value,
and the narrow accepted form is not a definition of the complete semantic roles
proposed below.

### 3.1 Specification declarations

Specifications are pure and total. They support mathematical integers, exact
bit strings, bytes, length-indexed collections, residues, fields, polynomials,
matrices, finite maps, algebraic data types, and parameterized modules.

Key rules:

- mathematical integers and fixed-width words are different types;
- sequence-of-bits and modular-machine-word intent are different types;
- byte order conversions are explicit;
- signedness changes are explicit;
- decoders return a result and expose canonicality requirements;
- recursion is structurally or well-founded terminating;
- no ambient I/O, allocation, clock, entropy, or target feature exists;
- executable evaluation is definitionally connected to the spec semantics.

### 3.2 Implementation declarations

Implementations are first-order, effectful procedures with contracts. They have
structured control flow, regions, ownership, borrows, mutable buffers, and typed
failure. Loops require invariants and termination variants.

Claim-bearing cryptographic kernels have no:

- undefined behavior;
- implicit panic or exception;
- hidden heap allocation;
- ambient randomness;
- ambient I/O;
- secret-dependent debug formatting; or
- unchecked pointer arithmetic.

Those capabilities may appear at a declared foreign boundary whose assumptions
are visible in the claim graph.

### 3.3 Machine implementation declarations

This stratum exposes exact layout, stack slots, target intrinsics, SIMD types,
instruction constraints, alignment, and scheduling controls. It is designed for
crypto engineers who would otherwise use intrinsics or assembly.

It remains typed and has a formal semantics. “Low level” does not mean “outside
verification.” Unsupported instructions or target combinations are rejected.

### 3.4 Game declarations

Games model finite or otherwise explicitly supported discrete distributions,
oracles, adversary interfaces, assumptions, reductions, and concrete advantage
bounds. They are separate from ordinary executable randomness.

The native 1.0 logic should be deliberately bounded to cryptographic patterns
the team can formalize and support. EasyCrypt and SSProve export/import adapters
provide wider expert workflows. An external theorem remains labeled external
until its evidence is reconstructed into Orange Proof IR.

### 3.5 Proof and claim declarations

Proofs establish propositions. Claims attach proved propositions and other
evidence to concrete exported artifacts. A theorem that is never bound to a
target, implementation, or artifact does not silently describe them.

### 3.6 Declassification and foreign declarations

Every deliberate release of secret-derived information names a policy. Every
foreign import names its ABI, preconditions, effects, alias rules, failure
behavior, and assumed claims. Neither is an annotation that suppresses the type
checker.

## 4. Core semantic family

The current provisional Typed Reference Core contains only typed `spec`
functions, contiguous source-order IDs, exact module and function names, one
normalized `Int` or `Word8` type, and one literal value. It contains no
operators, calls, parameters, bindings, effects, implementation declarations,
proof terms, or target information. `orangec eval FILE` visits those functions
in order and prints deterministic typed values; an empty Core prints nothing.

This internal compiler boundary is not any canonical Core proposed below. It
has no serialized identity, cross-revision ID promise, proof-checking role,
refinement relation, or erasure relation. The provisional OEP-0003 boundary does
not accept D-003 or D-004.

### 4.1 Spec Core

Spec Core is a small predicative dependent calculus suitable for mathematical
definitions and proof propositions:

- stratified universes;
- inductive families;
- propositional equality;
- proof irrelevance for propositions;
- structural or well-founded recursion;
- explicit axioms and axiom-closure reporting;
- no unchecked general recursion;
- no native execution primitive inside proof checking.

Built-in domains are small and semantically fixed. Higher cryptographic
structures are library definitions so they can evolve without expanding the
kernel.

### 4.2 Impl Core

Impl Core is a typed imperative calculus with:

- values, places, regions, and initialized storage;
- owned, shared-read, and unique-mutable capabilities;
- structured loops and calls;
- preconditions, postconditions, invariants, and variants;
- typed result/failure values;
- explicit arithmetic modes;
- an effect row or equivalent effect summary;
- an erasure map for ghost and proof values.

The core must have an executable reference interpreter and a mechanized
operational semantics. Surface elaboration emits a checked derivation or enough
canonical information for the core checker to reject an invalid translation.

### 4.3 Game Core

Game Core models probabilistic packages, adversaries, oracle calls, relational
reasoning, and exact bounds. It shares pure definitions with Spec Core through
one formal embedding rather than source duplication.

### 4.4 Proof IR

Proof IR is the stable evidence language checked by `orange-check`. It contains
fully elaborated terms, explicit universe and type arguments, references by
theorem fingerprint, and no tactic syntax.

Required properties:

- deterministic canonical encoding;
- streaming and resource-bounded validation;
- stable rejection behavior for malformed inputs;
- no acceptance of unknown fields in a security-critical major version;
- structural sharing without cyclic or exponential expansion attacks;
- theorem fingerprints that include definitions, axioms, semantics edition,
  and relevant target model;
- an inspection form suitable for independent implementations.

### 4.5 CT IR

CT IR is the compilation security boundary. It is first-order, monomorphized,
fixed-width, and free of undefined behavior. It exposes:

- basic blocks and call graph;
- loads, stores, widths, addresses, and alignment;
- explicit stack and buffer regions;
- fixed-width scalar and vector operations;
- endian conversion;
- secret/public domains;
- traps and termination behavior;
- target-classified instruction effects;
- an executable leakage trace.

The baseline relational property compares two runs with the same public input
and potentially different secrets. Their permitted observations must match.

Versioned policies are used instead of one `constant_time` Boolean. Proposed
families include:

- `ct-architectural-v1`: branches, call targets, memory addresses and widths,
  traps, and termination are secret-independent;
- `ct-variable-latency-v1`: additionally constrains secret operands to
  instructions classified as variable-latency for the target profile;
- `ct-speculative-v1`: uses a named speculative model or a proved hardening
  transformation;
- later hardware profiles tied to documented architectural data-independent
  timing contracts.

Power, electromagnetic, fault, remanence, and unspecified microarchitectural
behavior are excluded unless a separate policy models them.

### 4.6 Machine IR

Machine IR is ISA- and ABI-specific. It models registers, flags, instructions,
stack frames, calls, relocations, constants, target features, and exported
symbols.

Normative 1.0 target families are proposed as:

- x86-64 Linux, SysV ABI, with explicit baseline and selected AES/SHA/SIMD
  feature profiles;
- AArch64 Linux, AAPCS64, with explicit baseline and selected crypto/NEON/SVE
  profiles where modeled;
- host tools on current Linux, macOS, and Windows, even where those hosts are
  not initially supported for claim-bearing native output.

RISC-V, Windows native ABI, macOS object targets, and additional Wasm profiles
are planned extensions, not implicit 1.0 promises. The exact matrix must be
ratified by an incremental target decision before target implementation or
native-code claims.

## 5. Type, memory, secrecy, and effect system

### 5.1 Ownership plus deductive verification

Routine aliasing rules should be handled by affine ownership and borrowing:

- owned buffers;
- shared read borrows;
- unique mutable borrows;
- region and lifetime indices;
- length, alignment, initialization, and disjointness refinements;
- proof-backed safe slicing;
- narrowly scoped raw foreign pointers.

The verification-condition system handles facts that a borrow checker should
not attempt to infer: arithmetic bounds, nontrivial overlap, loop invariants,
representation relations, and functional postconditions.

### 5.2 Arithmetic

Every operation selects a semantics:

- mathematical integer arithmetic;
- modular word arithmetic;
- checked arithmetic with a proof obligation;
- saturating arithmetic if deliberately requested;
- target intrinsic with exact defined results and preconditions.

Shift amounts, division by zero, widening, narrowing, signedness, and overflow
cannot inherit host-language behavior.

### 5.3 Secrecy

Public and secret are semantic labels, not lints. They affect:

- branch and address typing;
- permitted conversions;
- diagnostics and formatter/debug output;
- proof obligations and leakage traces;
- ABI review;
- value copying and erasure tracking.

Secret values are non-copy by default. Deliberate cloning is explicit and
tracked. Labels may use domains beyond a two-point lattice when protocols need
separate principals, but the 1.0 kernel should start with the simplest model
that satisfies the approved corpus.

### 5.4 Effects

Representative effects are:

```text
read(region)       write(region)      allocate(region)
erase(region)      random(provider)   extern(contract)
declassify(policy) leak(policy)       io(capability)
```

Local effects may be inferred, while exported summaries are explicit and part
of compatibility.

### 5.5 Randomness and state

Specifications and games use semantic sampling. Implementations consume a
linear entropy/RNG capability with an explicit provider contract. There is no
ambient random function.

Linear or affine capabilities can also enforce state and nonce-use protocols
when an API genuinely supports that guarantee. Orange must not claim that a
type proves the quality of external entropy.

### 5.6 Erasure

Ghost values, specifications, and proof terms cannot affect runtime behavior.
An erasure theorem relates the typed program to Impl Core.

An `erase` or zeroization claim concerns architecturally modeled storage and
compiler-created copies. It does not promise physical destruction, cache
clearing, or remanence resistance without a stronger target policy.

### 5.7 Concurrency

General shared-memory concurrency is outside the 1.0 claim-bearing kernel. Explicit
data-parallel vector operations are supported. Protocol orchestration and
threading live in a host language through generated interfaces until Orange has
a separately designed concurrent memory and leakage model.

## 6. Claims and evidence

Every exported symbol has a claim matrix. The minimum native claim kinds are:

| Claim | Question answered |
| --- | --- |
| `conforms` | Does this artifact match the exact named standard/profile and evidence set? |
| `refines` | Does the implementation realize the named specification for all inputs under its precondition? |
| `safe` | Is execution free of the named memory, initialization, arithmetic, panic, and trap faults? |
| `terminates` | Does it terminate under the stated variant and environment assumptions? |
| `leakage` | Does it satisfy the named noninterference/leakage policy at the named layer and target? |
| `compiled` | Do the accepted compiler steps and final bytes preserve the named source properties? |
| `abi` | Does the object and wrapper satisfy the named calling, layout, alias, and error contract? |
| `erases` | Are named secret storage locations overwritten under the stated machine model? |
| `security` | Does a named construction satisfy a game-based theorem with the stated assumptions and bound? |
| `test_result` | Which explicitly scoped empirical vector, fuzz, differential, timing, and interoperability proposition was observed? |

Each claim record includes:

- stable claim ID and exact wording;
- subject definition/export and artifact digest;
- language, toolchain, cryptographic profile, target, and leakage policy;
- proof/certificate/test/audit evidence digests;
- all axioms, imported contracts, and external proof checkers in the closure;
- explicit exclusions;
- one `outcome`: `satisfied`, `not_satisfied`, `unresolved`, or `unsupported`;
- typed basis entries: `kernel_proof`, `checked_certificate`,
  `external_proof`, `test_run`, `audit`, `external_validation`, or `assumption`;
- a verification state for each basis entry, such as `checked`, `recorded`,
  `failed`, `expired`, or `unavailable`;
- creation and review policy, without making wall-clock time affect the proof
  digest.

Claims compose only through explicit rules. No numeric “assurance level” makes
functional correctness imply side-channel resistance. An outcome and a basis
are separate: one claim can be supported simultaneously by a kernel proof,
external review, and tests. `satisfied` requires at least one valid, unexpired
basis permitted by the claim kind’s policy; assumptions alone cannot satisfy a
proof-required claim. Other outcomes may carry no basis.

## 7. Proof automation

Automation is a portfolio behind one obligation protocol.

### 7.1 Bit vectors and finite equivalence

Use verified bit-blasting to SAT and require a checkable LRAT-family certificate
for successful claim-closing proofs. Counterexamples are decoded back to source
values.

### 7.2 Equality, arithmetic, and algebra

- kernel-checked reflective normalization for rings and fields;
- kernel-checked modular and range decision procedures;
- proof-producing SMT for explicitly supported EUF/linear-arithmetic/bit-vector
  fragments;
- explicit induction and user lemmas for quantified properties.

Generated routine obligations should be engineered to stay in decidable,
certificate-friendly fragments wherever possible.

### 7.3 Solver policy

Solvers may search, simplify, and find counterexamples. Claim-closing success
requires a certificate with no trusted or unsupported steps. Timeout, unknown,
resource exhaustion, missing proof output, or certificate-check failure leaves
the claim outcome `unresolved`, with the precise diagnostic reason recorded.

Developer profiles may display solver-only results, but they cannot satisfy a
claim or be cached under a misleading status.

### 7.4 Proof cache

The proof-result lookup key includes:

- normalized obligation;
- imported theorem fingerprints;
- source and core-semantics editions;
- checker and decision-procedure versions;
- target and leakage policy where relevant.

The cached value contains the accepted Proof IR or certificate digest and its
verification result. A separate proof-search cache may additionally key on the
solver executable digest, exact arguments, deterministic seed, and resource
limits. The certificate digest cannot be part of the lookup key for the result
the search is meant to find.

The CLI can explain which component invalidated a cached proof.

## 8. Compiler and binary path

### 8.1 Pass policy

Stable canonical passes are mechanized:

- elaboration validation;
- ghost erasure;
- monomorphization;
- closure elimination where applicable;
- memory/region lowering;
- CT IR construction;
- instruction-semantic lowering.

Optimization passes choose one of two acceptable forms:

1. a reusable mechanized preservation theorem; or
2. an untrusted transformer whose output comes with a checked per-artifact
   functional and leakage translation certificate.

An unverified pass cannot run in an assurance-preserving pipeline and retain old
claims.

### 8.2 Direct native path

The primary 1.0 assurance path emits Machine IR and direct native objects for
the approved targets. It validates:

- instruction bytes against the Machine IR;
- section placement and permissions;
- constants and tables;
- relocations and symbol bindings;
- stack/call ABI behavior;
- dispatch and CPU-feature selection;
- final exported symbol digests.

If a system assembler or linker remains in the path, Orange decodes the result
and checks it rather than merely trusting textual assembly. The link boundary
and any unchecked loader behavior appear in the TCB report.

### 8.3 Interoperability targets

| Output | Intended status |
| --- | --- |
| Reference interpreter | Exact executable source semantics and test oracle |
| Portable C11 | Broad review/integration; no generic post-C-compiler leakage claim |
| Generated Rust crate | Safe wrapper over the stable C ABI, not the canonical crypto backend |
| Jasmin export | Independent high-assurance cross-check and selected backend research path |
| Standard Wasm/WASI | Functional portability only unless a named runtime profile adds stronger evidence |
| LLVM IR | Research/interoperability only unless one exact pipeline has a ratified preservation argument |

Generated output is never described more strongly than its checked path allows.

### 8.4 Multi-implementation dispatch

A package may provide portable and hardware-accelerated implementations of one
specification. Each implementation has its own claims. The dispatcher is also
an implementation with a proof that:

- target-feature detection is correct under its platform contract;
- it selects only an implementation whose preconditions hold;
- every selected implementation refines the same spec;
- fallback behavior is approved and never silently lowers assurance.

## 9. Trusted computing base

`orange trust <artifact-or-claim>` prints a closure, not a marketing summary.

The intended logical TCB contains only what a particular claim needs:

- normative core semantics;
- the authoritative proof checker and built-in kernel-checked decision
  procedures;
- selected ISA, ABI, object, and leakage models;
- final binary decoder/validator;
- explicit axioms and imported foreign contracts;
- for runtime claims, named OS/CPU/entropy assumptions.

Normally outside the logical TCB because their results are checked:

- parser and formatter;
- elaborator and tactics;
- SAT/SMT solvers;
- untrusted optimizers, schedulers, and register allocators;
- package resolver, registry, LSP, and documentation generator;
- test, fuzz, benchmark, and statistical timing tools.

There is still an intent boundary. The checker can prove a formal spec; humans,
standards provenance, independent implementations, and vectors establish that
the formal spec is the intended algorithm.

In the current provisional S3a slice, the Rust semantic analyzer, integer
decoder, Typed Reference Core constructor, evaluator, and output formatter are
engineering trust dependencies. No logical checker exists, so their results are
not proof evidence and are not outside a logical TCB by virtue of being checked.

## 10. Canonical artifacts and reproducibility

### 10.1 Manifest and lock

`Orange.toml` is human-authored. `Orange.lock` is generated and records:

- immutable package identity and digest;
- semantic version for discovery;
- language and core-format editions;
- exported theorem fingerprints and assumption summaries;
- enabled features and target/leakage requirements;
- license and provenance metadata.

Proofs bind to exact digests, not semantic-version ranges.

### 10.2 Evidence bundle

Orange distinguishes two canonical forms:

- a **thin evidence manifest** may content-address external objects and is useful
  for local development or online distribution; it does not claim offline
  replay by itself;
- a **thick evidence bundle** contains every source, package, model, proof,
  certificate, tool, and build-critical byte needed for the replay it claims.
  Claim-bearing releases require a thick bundle.

The proposed `.orange-evidence` thick bundle contains:

- a canonical manifest that maps every content digest to its role, media type,
  size, and replay requirement;
- a content-addressed blob store containing the actual normalized sources,
  package objects, Core modules, target/ABI/leakage models, proof objects,
  certificates, checker/compiler/build tools or bootstrap inputs, native
  objects/libraries, generated headers, wrappers, and other bytes required for
  the replay the bundle advertises;
- claims and theorem-to-assumption graph;
- pass and translation certificates;
- compiler, checker, solver, and target-model identities;
- standards/errata/vector provenance;
- object/library, C header, Rust wrapper, and ABI-contract digests linked to
  their blobs;
- test summaries and their machine-readable result artifacts;
- archival audit/validation material when redistribution is permitted, and
  otherwise its digest, issuer, scope, validity metadata, and stable archival
  location; externally authoritative evidence is recorded, not transformed
  into a kernel proof;
- SPDX SBOM and CycloneDX SBOM/CBOM;
- SLSA/in-toto build provenance;
- signature/transparency material.

The logical proof and build-critical chain must remain replayable even if the
registry, transparency service, or ancillary audit URLs vanish. A third-party
audit or certificate may still require its issuer as the authority; the bundle
preserves and validates metadata without pretending to machine-check the human
or institutional judgment.

### 10.3 Determinism

Builds use canonical ordering and serialization, normalized paths, a pinned
locale and timezone, declared seeds, immutable tool digests, and
`SOURCE_DATE_EPOCH`. Profile-guided or search data that affects output is a
checked-in, hashed input.

Any authorized release must satisfy the separately provisioned owner-rebuild and
byte-comparison requirements in [the release policy](../RELEASE_POLICY.md).
Those runs are repeatability evidence, not independent rebuilds.

## 11. Package and registry security

- Published versions are immutable; yanking changes new resolution only.
- A lockfile plus all addressed package objects in a local store or thick bundle
  remains buildable without the registry. A lockfile alone is not the package
  bytes.
- Registry metadata uses a TUF-style threshold and delegated-role design with
  rollback and freeze protection.
- Maintainer MFA, namespace reservation, anti-typosquatting review, recovery,
  quarantine, and revocation are required before public package publication.
- Claim-bearing graphs do not execute arbitrary package build scripts.
- Deterministic generators run with explicit capabilities; generated source is
  hashed and checked like handwritten source.
- Packages declare axioms, foreign contracts, declassifications, unsafe
  boundaries, licenses, and proof checker dependencies.
- Policy can reject a graph whose assumption or license closure exceeds an
  organization’s approved budget.

## 12. Developer experience

The product is not complete if only its authors can use it.

### CLI

The current pre-alpha CLI has `orangec check`, `orangec eval`, and `orangec lex`.
`check` performs lexical, syntactic, and bounded semantic validation. `eval`
accepts one source and prints each typed specification in source order as
`module::name: Type = value`, using decimal `Int` and two-digit lowercase
hexadecimal `Word[8]` values.

The intended command families additionally include formatting, testing,
proving, building, documentation, package operations, evidence replay, trust
inspection, target inspection, and conformance runs. Their exact names remain
later CLI design, and the current evaluator output is not a canonical Core or
evidence encoding.

### Language server

- Hover: normalized type, widths, effects, secrecy, contracts, and claim status.
- Inlay hints: inferred sizes, regions, and secrecy domains.
- Goal view: hypotheses, assumptions, and remaining proof obligations.
- Counterexamples: decoded from solver/Core values to source spans.
- Code lenses: run/replay a proof and inspect evidence.
- Trust view: theorem-to-assumption and target-model graph.
- IR explorer: source through Spec/Impl Core, CT IR, Machine IR, and bytes.
- Cache explanation: why a proof or build was invalidated.

Diagnostics distinguish parse/type failure, disproved goal, unknown, timeout,
unsupported reasoning, untrusted solver result, failed certificate, and unmet
target assumption.

### Secret-safe debugging

- Spec evaluation supports deterministic and symbolic replay.
- Differential testing compares spec, implementation, external libraries, and
  standards vectors.
- Instrumented native profiles are non-claim-bearing and redact secret values.
- Claim-bearing builds reject logging, tracing, coverage, and secret-dependent
  assertions in cryptographic kernels.
- Source maps connect every IR and final instruction to source obligations.

## 13. Foreign interface

The stable integration boundary is generated C ABI plus a machine-readable
contract:

- exact scalar and aggregate layout;
- buffer lengths, alignment, overlap, mutability, and initialization;
- typed error/failure behavior;
- no hidden allocator, exception, panic, TLS state, or RNG;
- stable symbol/version policy;
- explicit zeroization and ownership transfer rules;
- target-feature and dispatcher requirements.

Rust wrappers enforce what Rust’s type system can represent and check remaining
preconditions. Other language bindings sit above the same C contract.

Imported functions remain assumptions until separately proved. The assumption
is attached to every dependent claim.

## 14. Bootstrap

Orange will not create a disposable compiler.

1. Mechanize the normative cores, checking relation, erasure, and leakage
   semantics before broad surface-language growth.
2. Implement the permanent safe-Rust driver/frontend and preserve it as a
   supported bootstrap and implementation-diverse frontend.
3. Produce the authoritative extracted checker from the ratified metatheory and
   differential-test it against the Rust checker.
4. Add each permanent compiler pass with semantics, preservation obligations,
   interpreters, and conformance cases.
5. Use Jasmin, SAW, mature libraries, and independent evaluators as oracles while
   the direct path develops; do not let an oracle become an undocumented TCB.
6. Self-host only Orange components that naturally fit a crypto-focused
   language. Networking and editor tooling need not be self-hosted.
7. If any compiler core becomes self-hosted, retain the stage-0 path and require
   reproducible diverse double compilation or an equivalent bootstrap check.
8. Never retire a bootstrap stage until separately provisioned owner rebuilds,
   proof-checker agreement, and recovery drills pass; record independent build
   and audit evidence as unavailable unless it actually exists.

## 15. Proposed repository structure

This is the intended production layout, not a request to create empty folders:

```text
docs/                  charter, research, architecture, assurance, roadmap
spec/                  human-readable normative language specification
formal/                selected metatheory, kernel, semantics, verified passes
research/decisions/    reproducible, archived Gate 0 decision evidence
compiler/               Rust workspace: driver, frontend, diagnostics, later tools
schemas/                canonical Core, claim, package, and evidence schemas
stdlib/                 Orange language and proof standard library
crypto/                 flagship standards-sourced cryptography corpus
targets/                ISA, ABI, leakage, object, and feature profiles
conformance/            stable language/toolchain/claim conformance cases
tests/                  integration, adversarial, differential, and fuzz corpora
tools/                  deterministic repository and release tooling
release/                provenance policies, bootstrap inputs, ceremonies
```

Directories are created when their first permanent, solo-reviewed artifact
lands. `Solo-reviewed` is not independent review.

## 16. Architecture gates

Before the affected component or claim stabilizes, ratify:

1. Rocq versus Lean metatheory using the same representative decision suite;
2. Orange Proof IR logic and axiom policy;
3. exact semantic strata and crossings;
4. 1.0 target/host and object-format matrix;
5. baseline leakage trace and target-classification process;
6. direct object validation strategy;
7. flagship corpus and the claim each algorithm must exercise;
8. final name, licenses, release policy, and version axes.

D-024 deliberately precedes these gates because source identity, byte spans,
lexing, diagnostics, and a host CLI do not depend on their answers. Parsing must
record its grammar boundary; proof work must wait for items 1–3; target and code
generation claims must wait for items 4–6; cryptography packages must wait for
item 7; distribution must wait for item 8.

The custom proof kernel, source-to-binary leakage preservation, object-file last
mile, and probabilistic game logic are the hardest architectural work. They are
the spine of the product and therefore appear early in the roadmap rather than
as promises to bolt on after a pleasant syntax exists.
