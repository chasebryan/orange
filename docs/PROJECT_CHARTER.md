# Orange project charter

Status: proposed for ratification

Research snapshot: 2026-07-11

## 1. Mission

Orange will make the relationship between cryptographic intent, production
implementation, and assurance evidence explicit and reviewable.

An Orange package should be able to answer all of these questions without
requiring a reviewer to reconstruct an informal tool chain:

1. What mathematical function, construction, or protocol component is meant?
2. Which executable implementation realizes it?
3. Which properties are claimed, over precisely which inputs and targets?
4. Which assumptions and leakage model bound each claim?
5. Which proof object, certificate, test corpus, or external derivation supports
   the claim?
6. Which source, toolchain, dependencies, and build invocation produced the
   bytes being shipped?

The product is not merely a notation, theorem-prover front end, or code
generator. The end product is a coherent language, proof system, compiler,
package format, standard library, developer toolchain, and release discipline.

## 2. Problem

High-assurance cryptography is currently achievable, but usually through a
specialist, polyglot pipeline. A mathematical specification may live in one
language, a fast implementation in another, functional proofs in a third,
game-based security arguments in a fourth, and build or binary evidence in
shell scripts and CI logs. The seams become part of the trusted computing base,
yet are often implicit.

Orange addresses the seams. Its primary product is a claim-oriented build graph
that connects a source specification all the way to a shipped artifact. The
surface language exists to make that graph usable by cryptographic engineers;
the evidence format exists to make it independently auditable.

## 3. Target users

### Cryptographic implementers

They need fixed-width arithmetic, vector intrinsics, predictable memory and
control flow, benchmark visibility, and stable foreign-function interfaces.
They should not have to restate the algorithm in an unrelated prover language.

### Verification engineers

They need precise semantics, compositional lemmas, explicit invariants,
counterexamples, proof replay, and a small trusted checker. They need to know
where automation stops and assumptions begin.

### Cryptographers and standards authors

They need readable, executable specifications; probabilistic games and
assumptions; parameterization; canonical test-vector generation; and links from
standards clauses to source definitions.

### Library maintainers and integrators

They need stable C and Rust-facing APIs, deterministic builds, target feature
selection, misuse-resistant wrappers, machine-readable assurance manifests,
and an actionable security-update process.

### Auditors and downstream consumers

They need to replay evidence without network access, inspect the exact trusted
base, distinguish proved facts from tested observations, and trace a binary to
reviewed source.

## 4. Product thesis

Orange is a standalone, domain-specific language with several deliberately
separated semantic strata and one shared module system:

- **Specification** for total mathematical functions and relations.
- **Implementation** for terminating, memory-safe executable procedures.
- **Machine implementation** for explicit layout, vector operations, target
  intrinsics, and leakage-aware control flow.
- **Game** for probabilistic programs, adversary interfaces, and reduction-based
  cryptographic claims.
- **Proof** for refinements, invariants, equivalences, noninterference, and
  security reductions.

The strata share names and types where it is sound to do so, but they do not
silently identify mathematical integers with machine words, pure functions
with stateful procedures, or source-level constant-time style with a claim
about emitted machine code.

The unit of assurance is a **claim**, not a package-wide adjective. A build can
therefore say, for example, that one exported implementation:

- conforms to a named standard and vector set;
- refines a particular Orange specification;
- is memory-safe and free of specified arithmetic faults;
- is constant-time under an address-and-control-flow leakage model;
- preserves those properties through a named compiler and target;
- has a game-based security theorem under listed assumptions; and
- has passed empirical tests that are useful but are not proofs.

No one claim implies the others.

## 5. In scope for the 1.0 product

- A versioned language reference and mechanized semantics.
- A deterministic parser, formatter, type checker, interpreter, documentation
  generator, and language server.
- Fixed-size sequences, bit vectors, mathematical integers, finite fields,
  modular arithmetic, algebraic data types, parameterized modules, and
  refinement-friendly contracts.
- Explicit public/secret information-flow labels, regions, ownership, mutable
  buffers, loops with invariants, zeroization obligations, target features, and
  vector intrinsics.
- Functional correctness, safety, termination, equivalence, and a stated form
  of constant-time noninterference.
- Probabilistic games and an integrated path for machine-checked security
  reductions.
- Interactive proof terms plus certificate-producing automation for decidable
  fragments.
- A verified compilation path for the native targets ratified at Gate 0,
  including an auditable connection to final object code. The current proposed
  envelope is Linux x86-64 and AArch64, subject to staffing and model review.
- A reference/interoperability C backend whose weaker assurance is clearly
  labeled.
- Stable C ABI artifacts, generated C headers, and generated Rust bindings.
- Content-addressed packages, a lock file, offline proof replay, and a signed
  evidence bundle.
- A standard library and a flagship claim-complete cryptography corpus that
  exercise symmetric, hash, field, elliptic-curve, and post-quantum workloads.
- NIST ACVP-compatible vector import/export and standards provenance metadata.
- Reproducible, signed releases for supported hosts.

## 6. Explicit non-goals for 1.0

- A general-purpose application language.
- Automatic invention or validation of new cryptographic designs.
- A promise that functional correctness proves cryptographic security.
- A promise that a constant-time proof covers power, electromagnetic, fault,
  speculative-execution, or all microarchitectural leakage unless a particular
  claim explicitly models it.
- General concurrent, distributed, or network-protocol verification.
- Hardware-description-language or FPGA synthesis.
- General heap allocation, garbage collection, exceptions, or ambient I/O in
  verified cryptographic kernels.
- A built-in network dependency during verification or proof replay.
- Self-declared FIPS 140 validation. Orange can produce useful algorithm and
  evidence artifacts, but validation remains an external process.
- Compatibility with arbitrary unsafe C or Rust code without an explicit FFI
  boundary and assumptions.

These exclusions keep the first stable language honest. Later work can add a
new claim model; it must not silently widen an existing one.

## 7. Engineering doctrine: build the end product directly

Orange will be developed incrementally but not prototyped and rewritten.

1. The normative semantics precede convenience syntax and optimization.
2. Every committed implementation component must occupy its intended final
   boundary and have production error handling, deterministic output, tests,
   and versioned formats.
3. Early algorithms are conformance fixtures for the permanent language and
   compiler, not one-off demonstrations.
4. Automation may propose proofs; a proof-required claim closes only with
   replayable evidence accepted by the checker. Claims whose authority is an
   external audit or validation use a distinct recorded-external basis.
5. A backend cannot inherit a source property by assertion. The lowering or
   emitted artifact must carry a checked preservation argument.
6. Performance work starts with the IR and cost model, then remains subordinate
   to semantics and evidence.
7. Unimplemented claims fail closed. `Unsupported`, `unresolved`, and
   `not_satisfied` are distinct outcomes; an assumption or test run cannot
   satisfy a claim that requires a checked proof.
8. No phase is allowed to create a second informal language inside build
   scripts, macros, or backend annotations.

Small design-validation cases are allowed only in the permanent test and
semantics framework. They are not a parallel product, and passing them is not a
release claim.

## 8. Product principles

### Soundness before automation

An inconvenient proof obligation is preferable to an unsound success. Solvers
are search engines; their answer is accepted only through a checked certificate
or an explicitly disclosed external-trust claim.

### Claims are scoped and compositional

Every report names the definition, implementation, target, compiler, model,
assumptions, dependencies, and evidence digest to which it applies.

### Secrets are a semantic concern

Secrecy labels affect typing, allowable control flow and addresses, leakage
traces, diagnostics, and ABI review. They are not comments.

### No hidden target behavior

Integer widths, overflow, shifts, byte order, alignment, aliasing, target
features, and randomness are explicit. Unsupported behavior is rejected rather
than inherited from a host compiler.

### Independent replay is a feature

A reviewer can unpack a proof bundle, inspect its manifest, and run the checker
against pinned inputs without a registry, cloud service, or source checkout.

### Evidence survives optimization

Optimization passes either have a mechanized preservation theorem or emit a
translation-validation certificate. A pass that cannot do either is excluded
from an assurance-preserving build.

### The standard library proves the product

The flagship cryptography corpus is not a marketing sample. It is the end-to-end
acceptance suite for expressiveness, proof ergonomics, generated code,
interoperability, documentation, and maintenance.

## 9. What “end” means

Languages do not become permanently finished. For Orange, “end” means the
first stable, supportable 1.0 system, not the end of maintenance.

The 1.0 gate is closed only when:

- the language reference, core calculus, leakage model, ABI, package schema,
  evidence schema, and compatibility policy are versioned and published;
- the mechanized metatheory and verified compilation statements cover every
  assurance-preserving path advertised by the CLI;
- the authoritative checker can replay every machine-checkable release claim
  from a clean offline environment and validate the integrity, scope, identity,
  and validity metadata of recorded external evidence;
- supported native targets produce correct, ABI-conformant objects and the
  final artifact connection is checked;
- the flagship corpus meets its published claim matrix and standard vectors;
- negative tests demonstrate that invalid programs, forged evidence, invalid
  proofs, and unsupported claims fail closed;
- releases are deterministic, independently reproduced, signed, accompanied by
  SBOM and provenance, and recoverable through a documented bootstrap path;
- two independent security reviews have been completed, all critical and high
  findings are resolved, and the public threat model matches the shipped
  system;
- the language server, diagnostics, reference manual, proof guide, integration
  guide, and migration policy are usable by people outside the core team;
- maintainers have an LTS, vulnerability-response, key-management, and
  deprecation process they have exercised in a release drill.

After 1.0, new architectures, leakage models, proof automation, and algorithm
packages are normal evolution. They do not retroactively strengthen old claim
bundles.

## 10. Success measures

The project will track evidence, not vanity metrics:

- percentage of language rules represented in the mechanized semantics;
- proof-checker and compiler-pass theorem coverage;
- number of hidden assumptions per published claim and trend over time;
- proof replay success from clean environments and median replay time;
- differential and negative-test corpus size and mutation score;
- reproducibility rate across independent builders;
- conformance coverage by standards clause and vector family;
- generated-code performance and code size against named, versioned baselines;
- time from a source change to an updated, reviewable proof bundle;
- number and severity of unresolved audit findings;
- onboarding time for an external contributor to specify, implement, prove,
  and export one small primitive using only published documentation.

Adoption counts are useful, but they cannot substitute for these gates.

## 11. Decisions required before implementation

The following must be ratified at the first roadmap gate:

1. project name and package/command namespace;
2. source, documentation, generated-code, and specification licenses;
3. governance and security-response authority;
4. proof foundation and certificate formats;
5. normative 1.0 host and target matrix;
6. the exact leakage semantics attached to the first constant-time claim;
7. the flagship corpus and required claim matrix;
8. the compatibility and support window.

Recommended answers and alternatives are recorded in
[DECISIONS.md](DECISIONS.md). No implementation should smuggle in a choice that
this gate has not made explicit.
