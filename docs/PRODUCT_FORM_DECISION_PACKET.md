# D-003 product-form decision packet

Status: draft owner-review packet; no product form selected

Packet version: `d003-v0.1-draft`

Snapshot: 2026-07-13

## Abstract

This proposal makes Orange a standalone, editioned domain-specific language
whose source semantics and canonical Core boundary are owned by the Orange
project. External proof assistants, solvers, compilers, cryptography systems,
and host languages remain valuable tools and interoperability endpoints, but
none is the semantic authority for an Orange program.

The product boundary includes Orange source, normative semantics, versioned
Orange-owned Core artifacts, deterministic developer tools, and explicit
interfaces to proof, claim, compiler, package, and evidence systems. The exact
number and relationship of semantic strata remain a D-004 decision. This
proposal fixes the ownership boundary without preempting that structure or the
later proof-format and claim-model decisions.

The current Typed Reference Core remains a permanent internal compiler
boundary for accepted S3a behavior. It is not retroactively made canonical,
serialized, proof-bearing, or stable across revisions. A later accepted Core
proposal must define its migration into the first versioned canonical boundary.

This packet is draft decision research. It recommends the standalone form but
records no project-owner acceptance, no implementation authority for S3b, and
no product release or assurance claim.

## Motivation

[D-003](DECISIONS.md#d-003--product-form) is the first unresolved
dependency in the active roadmap after S3a. It must constrain the language and
evaluator before S3b adds pure expressions or calls. Leaving the product form
implicit would let each new construct choose a host language, proof assistant,
or orchestration seam accidentally.

The proposed 1.0 boundary has 8/8 structurally specified design journeys and
0/8 completed journeys in [`USER_JOURNEYS.md`](USER_JOURNEYS.md):
installation, specification, implementation and proof, native builds, foreign
integration, offline replay, updates, and incident response. Those journeys
require identities and claims to remain connected across source, Core, proof,
target, artifact, and evidence boundaries. An orchestration manifest can
coordinate those systems, but it cannot by itself supply one language meaning
or one canonical claim subject.

S1 through S3a provide permanent-lineage progression evidence, not proof of a
future migration. Orange already has an exact edition marker, its own grammar,
source-mapped syntax, name checking, typed values, an internal Typed Reference
Core, and deterministic evaluation. A standalone product continues those
boundaries directly. A host-backed alternative would need to preserve the
accepted behavior through an explicit bounded migration; discarding it would
conflict with D-002's permanent-lineage rule, while D-002 does not categorically
forbid every host-backed migration.

The solo envelope in
[`GATE0_SUPPORT_ENVELOPES.md`](GATE0_SUPPORT_ENVELOPES.md) requires one
owner-executable sequence and honest claim limits. Owning the semantic boundary
does add language and Core maintenance, but it avoids making several external
systems mandatory co-authorities whose version skew and translation relations
would all need permanent support.

## Scope and non-goals

This proposal defines:

- Orange as a standalone source language with explicit editions;
- Orange specifications, not host-language behavior, as semantic authority;
- an Orange-owned, versioned canonical Core boundary for future accepted
  product artifacts;
- the role of external systems as explicit adapters, evidence producers,
  oracles, implementation dependencies, or foreign boundaries;
- migration requirements from the current noncanonical Typed Reference Core;
- non-compensable product-form decision criteria; and
- reconsideration triggers for the selected boundary.

This proposal does not select:

- the D-004 semantic strata or the number of Core languages;
- the D-005 claim model or D-007 proof artifact and checker architecture;
- a D-006 proof foundation, proof calculus, checker implementation, or solver;
- S3b syntax, operators, calls, bindings, control flow, or failure semantics;
- a canonical encoding, version number, theorem fingerprint, or package format;
- a compiler IR strategy, target, ABI, memory model, or leakage model;
- a cryptographic primitive, standard, corpus, release, or support profile; or
- a repository-wide license or final public product name.

This packet changes no accepted language behavior. S3b remains blocked on an
accepted D-003 disposition, an accepted D-004 disposition, and its own bounded
OEP.

## Specification

### Product identity

An Orange source is a first-class program identified by its bytes and declared
edition. Accepted Orange specifications define its syntax and meaning. A tool
may be implemented in Rust, extracted from a proof assistant, or call an
external engine, but changing that implementation dependency cannot silently
change the meaning of an accepted Orange program.

The eventual product presents one coherent Orange workflow:

- `orangec` and frontend services consume editioned Orange source;
- the reference evaluator observes accepted source semantics;
- canonical Orange Core artifacts identify elaborated meanings;
- `orange-check` consumes proof, claim, and model artifacts selected by later
  decisions while binding them to canonical Orange Core subjects;
- `orange-compile` relates checked Core to target artifacts; and
- package and evidence tools bind every result to exact Orange and external
  identities.

These names describe component responsibilities, not current implementation or
release promises. The implementation need not self-host, and not every
component must use one implementation language.

### Orange-owned Core boundary

The stable product-form boundary has these invariants:

1. Every accepted source meaning belongs to exactly one declared Orange
   edition and normative semantic revision.
2. Public elaborated meanings use versioned Orange-owned Core formats. A host
   AST, compiler heap, prover environment, or foreign IR is never the canonical
   identity merely because one implementation uses it.
3. Canonical Core identities bind exact bytes, schema or grammar version,
   semantic stratum, and all referenced definitions. Unknown versions and
   noncanonical encodings fail closed.
4. A transformation between Core members, proof artifacts, foreign models, or
   target IRs names the relation and evidence that justify it. Format
   conversion alone never implies semantic preservation.
5. A later proof and checker boundary binds its subject to canonical Core rather
   than trusting only an in-memory frontend object.
6. D-004 decides how many semantic Core members exist, what they share, and
   which embeddings, erasures, or refinement relations connect them.

The S3a Typed Reference Core satisfies an internal engineering boundary only.
Its source-ordered IDs, Rust representation, and in-memory values have no
canonical byte identity. Acceptance of this proposal would require later Core
work to migrate the accepted typed-literal meaning without treating current
private representation details as public compatibility commitments.

### Interoperability boundary

External systems integrate through named, versioned adapters. Each adapter
records the source and destination identities, supported fragment, translation
relation, trust role, assumptions, unsupported cases, resource limits, and
failure behavior.

- A proof assistant may host metatheory or produce an authoritative extracted
  checker after D-006, but its project file is not Orange source or canonical
  Orange Core. D-007 separately decides permanent proof interchange.
- A solver may search for evidence, but its success is non-authoritative until
  an accepted checker policy validates a supported certificate or proof term.
- Cryptol, hacspec, Jasmin, EasyCrypt, SSProve, and similar systems may provide
  import, export, differential, or external-evidence paths. Their results retain
  exact provenance and do not become native Orange claims by relabeling.
- Rust, C, LLVM, native objects, and foreign consumers may be implementation or
  target boundaries. Their semantics and ABI assumptions remain explicit.

Adapter failure affects only the translated artifact or dependent claim. It
does not silently select a second Orange language meaning.

### Decision gates

The four D-003 candidates are evaluated against eight non-compensable gates.
There is no weighted score; one failed gate makes a candidate ineligible for
the product boundary described by D-001 and D-002.

| ID | Hard gate |
| --- | --- |
| PF-G01 | The result is recognizably a language for specifying, implementing, and verifying cryptography, not only a task manifest or library convention. |
| PF-G02 | Every semantic authority is pinned, versioned, and explicit; a dependency upgrade cannot silently redefine accepted program meaning. |
| PF-G03 | Source, Core, proof, target, artifact, and evidence identities can be bound in an artifact-scoped claim graph that replays offline. |
| PF-G04 | J-01 through J-08 remain expressible, with every cross-language or cross-tool transition identified rather than hidden. |
| PF-G05 | Accepted S1 through S3a source and semantic behavior can be preserved through a bounded documented migration without a disposable rewrite. |
| PF-G06 | One owner can advance the system incrementally; mandatory software can be pinned and replayed without requiring unavailable people or organizations. |
| PF-G07 | External proof, compiler, standards, and consumer ecosystems remain usable through explicit bounded adapters. |
| PF-G08 | A failed adapter, unsupported host feature, or missing external tool fails at its boundary without changing unrelated Orange meaning or claims. |

The proposal's assessment is:

| Candidate | G01 | G02 | G03 | G04 | G05 | G06 | G07 | G08 | Disposition |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| PF-01: standalone editioned Orange DSL | Pass | Pass | Pass | Pass | Pass | Pass | Pass | Pass | Recommend |
| PF-02: manifest-only orchestration | Fail | Pass | Pass | Pass | Unproven | Pass | Pass | Unproven | Reject as product form; retain orchestration techniques |
| PF-03: DSL embedded in F\*, Lean, or Rocq | Pass | Pass | Pass | Unproven | Unproven | Pass | Pass | Unproven | Reject as product form; retain proof adapters |
| PF-04: Rust subset with proof annotations | Pass | Pass | Pass | Unproven | Unproven | Pass | Pass | Unproven | Reject as product form; retain Rust implementation and integration paths |

`Unproven` is non-success for a hard gate. It means the current record does not
show that the candidate can satisfy the gate with a complete declared bridge,
migration, dependency, and failure boundary. It is not evidence that the
candidate is impossible. `Pass` is design-level evidence against the current
repository record, not an implemented end-to-end journey result.

### Journey coverage

The standalone boundary supports each proposed journey for a distinct reason:

| Journey | Product-form obligation |
| --- | --- |
| J-01 | Installation binds one Orange edition, Core/checker format set, toolchain identity, and support state rather than a compatible-looking set of host tools. |
| J-02 | Standards authors write total Orange specifications whose integer, word, decoding, and failure meaning does not inherit from a host language. |
| J-03 | Implementations, proofs, and independently scoped claims meet through canonical Orange Core subjects and a later accepted claim graph while external search remains labeled. |
| J-04 | Native artifacts retain a traceable relation from Orange meaning through every IR, certificate, target model, and final byte identity. |
| J-05 | Generated foreign interfaces expose one explicit contract without turning C or Rust caller behavior into Orange semantics. |
| J-06 | Canonical Orange artifacts can be inspected and replayed offline without reconstructing a live multi-tool workspace as semantic authority. |
| J-07 | An edition, profile, adapter, or tool can be replaced without silently rewriting the meaning of preserved Orange artifacts. |
| J-08 | A vulnerability or invalidated bridge identifies the exact dependent Orange claims instead of invalidating an opaque polyglot success label. |

This is design coverage, not journey completion or user validation.

### Owner-executable resource analysis

The proposal compares permanent authority and bridge burden rather than
assigning an unsupported calendar estimate.

| Candidate | Mandatory semantic authorities | Permanent bridge burden | Solo effect |
| --- | --- | --- | --- |
| PF-01 | One Orange editioned language and its accepted Core family | Explicit adapters only for used external systems | Highest direct language/Core work; dependencies and adapters can be admitted one at a time |
| PF-02 | Every orchestrated source language, prover, compiler, and their version combinations | Pairwise identity and semantic relations across the selected tool graph | Low initial syntax work, but an expanding mandatory integration and claim-closure surface |
| PF-03 | Orange embedding plus the selected prover language, kernel, libraries, extraction, and project format | Bridges for non-host proof, compilation, packages, targets, and long-lived artifact interchange | Fast host-native proof experiments but permanent ecosystem lock and migration burden |
| PF-04 | Rust subset semantics, compiler interpretation, proof annotations, and proof backend semantics | Rust-to-proof, Rust-to-target, source-version, and claim-identity bridges | Familiar implementation path but specification and implementation roles remain coupled |

The standalone choice is viable only with incremental scope discipline: one
edition, one semantic slice, one accepted Core boundary, one adapter, and one
target path at a time. Under-resourcing removes a claim, adapter, target, or
package; it does not delegate the language definition implicitly.

## Alternatives

### Manifest-only orchestration

Orange could be a manifest and evidence layer over Cryptol, hacspec, Jasmin,
EasyCrypt, or similar systems. This preserves mature tools and minimizes new
surface syntax. It was rejected as the product form because each source system
would retain its own semantics, versioning, errors, trust model, and artifact
identity. The manifest could report those seams honestly, but it would not
meet the language mission directed by D-001 through the existing standalone
S1-S3a lineage.

The orchestration techniques remain useful for differential tests, standards
imports, external evidence, and evidence-bundle assembly.

### Embedded DSL in a proof assistant

Embedding in F\*, Lean, or Rocq could provide binding, notation, automation,
and proof infrastructure quickly. It was rejected as the product form because
host parsing, elaboration, universe behavior, libraries, compiled environments,
and upgrades would become part of Orange source meaning or require a permanent
translation layer to deny that fact.

The selected D-006 foundation may still host normative metatheory and produce
an authoritative checker. That implementation role does not make its source
language or compiled environment the Orange interchange boundary.

### Rust subset with proof annotations

A Rust subset could reduce implementation friction and improve immediate
interop. It was rejected as the product form because Rust's evolving language,
compiler, unsafe boundary, machine-oriented types, panic and allocation
behavior, and implementation-first structure do not directly express the
mathematical specification and proof roles Orange needs. Defining an exact
subset plus a stable semantic translation would itself create a language, while
retaining Rust syntax and version coupling.

Safe Rust remains the directed implementation language for the permanent
frontend and tools. Generated Rust wrappers remain an intended integration
path.

### One universal Orange IR

One surface language could lower immediately to one universal IR. This is not a
separate D-003 product form and is deferred to D-004. The current evidence warns
that mathematical totality, stateful implementation, probabilistic games,
proofs, and target execution have different judgments. This proposal requires
Orange ownership of the boundary but does not assume one or several Core
members.

## Compatibility and migration

This packet changes no source, compiler, Core, proof, package, or artifact
behavior. If accepted, the product-form decision preserves every source
accepted by OEP-0002 and OEP-0003 until an edition-aware semantic proposal
records a change.

The current Typed Reference Core remains internal. Its function order, IDs,
Rust types, and evaluator structures may evolve. Before any public canonical
Core claim, a later accepted proposal must:

1. map every accepted S3a typed literal and type into the selected D-004 Core
   member;
2. prove by conformance cases that reference observations and failures remain
   consistent;
3. define versioned canonical encoding and rejection behavior;
4. state whether any internal identity is intentionally preserved; and
5. provide a rollback path that restores the last accepted internal boundary
   without accepting two competing meanings.

Rejecting this recommendation or retiring the packet removes no implemented
feature. After acceptance, changing the standalone product form requires a
superseding OEP, complete journey and migration analysis, and explicit
disposition of every published Orange-owned identity.

## Semantic and claim effects

This packet defines no new Orange expression, type, evaluation rule, or
accepted Core artifact. If accepted, it would make the ownership boundary
normative: Orange specifications define Orange meaning, and public semantic
identities use Orange-owned formats.

The decision would not establish semantic soundness, completeness, refinement,
proof soundness, compilation correctness, cryptographic correctness, leakage
resistance, ABI stability, external interoperability, certification, release
reproducibility, independent review, or production readiness.

External evidence remains external until an accepted relation imports it with
its exact status and assumptions. A standalone product form does not make an
Orange-authored result proved or independently validated.

## TCB, axiom, and proof effects

Drafting adds no trusted component, axiom, theorem, proof rule, checker, solver,
certificate, or extraction path. The current Rust frontend, standard library,
toolchain, host, and sole owner remain engineering trust dependencies for
implemented behavior.

If accepted, later proof and compiler components must expose their external
trust through boundaries bound to canonical Orange Core identities. A proof
assistant kernel may become part of the logical construction or extraction TCB
after D-006; D-007 still determines the public proof identity. No
implementation-diverse checker written by the same owner becomes independent
review.

## Threat, abuse, and leakage effects

The main product-form threats are semantic substitution, version confusion,
adapter mistranslation, authority laundering, and claim over-composition.

- A malicious or stale adapter could claim that host artifacts represent a
  different Orange meaning. Exact endpoint identities, supported fragments,
  checked relations, and fail-closed version handling constrain that path.
- A tool upgrade could substitute a host AST, prover heap, or foreign IR for a
  canonical semantic artifact. Orange-owned Core formats and explicit
  conversion records prevent implementation convenience from changing
  authority.
- A successful external proof, vector, compile, or test could be presented as a
  native Orange claim. Typed evidence status and artifact-scoped claim records
  preserve the external boundary.
- A universal success label could hide one failed bridge. Independent claim
  families and dependency closure keep failures local and visible.

These controls refine the evidence-confusion risks identified by TM-005 and
TM-010 and the hostile frontend boundary in TB-008. They establish no secrecy
label, leakage trace, constant-time behavior, physical protection, or resistance
to a defect shared by the solo-authored specification and implementation.

## Target and ABI effects

This proposal selects no host, target, object format, calling convention, CPU
feature, memory layout, foreign ABI, portable C path, or native assurance
profile. It requires only that a later target boundary be explicit and related
to an Orange-owned semantic identity.

Generated C and Rust interfaces remain intended interoperability artifacts.
Their contracts and target assumptions require later decisions and do not
become Orange source semantics.

## Standards, errata, and provenance

No external language or cryptographic standard is incorporated normatively by
this product-form decision. The comparison uses repository-defined missions,
journeys, architecture, and accepted implementation boundaries.

Future imports from standards and external cryptography systems retain exact
edition, clause, errata, digest, acquisition, interpretation, and rights
provenance. Standalone ownership must not be used to erase the authority or
ambiguity of an external standard.

## Dependencies, licenses, and IP

Drafting adds no crate, proof assistant, solver, compiler, package, build script,
network fetch, or generated source. It does not change the admitted Rust
standard-library-only compiler graph.

The repository-wide outbound license, contribution terms, generated-output
policy, dependency redistribution terms, final product name, and trademark
clearance remain unresolved under D-017 and D-018. Product-form acceptance
would not grant copying, redistribution, package publication, or release
authority.

## Conformance, tests, and evidence

The current owner-executable decision evidence is:

- four named D-003 candidates compared against PF-G01 through PF-G08;
- 8/8 structurally specified proposed journeys, with 0/8 complete, mapped to a
  standalone product-form obligation;
- the accepted S1-S3a permanent-lineage progression from source identity through
  deterministic typed reference evaluation;
- explicit requirements for a future private-to-canonical Core migration;
- the solo resource comparison and incremental scope rule; and
- falsifiable reconsideration triggers below.

Acceptance requires all of the following to be true at an exact reviewed
revision, followed by an acceptance record that binds that revision:

1. The Orange Project Owner explicitly accepts, rejects, or modifies every hard
   gate and candidate disposition.
2. D-003, the roadmap, architecture status, and affected reference prose agree
   on the product form without accepting D-004 implicitly.
3. The current Typed Reference Core migration and noncanonical status remain
   explicit.
4. Every unresolved question is either assigned to a later decision or blocks
   acceptance.
5. `./scripts/ci/check-repository`, `make check`, and
   `python3 tools/validate_foundation.py --root . --format json` pass at the
   reviewed bytes.
6. A follow-up acceptance record binds the exact reviewed commit and a literal
   `solo-reviewed` owner approval record without implying independence.

Acceptance of D-003 does not authorize S3b. D-004 and a bounded S3b OEP remain
separate gates.

Reconsider the recommendation if reproducible evidence shows any of these:

- a permanent representative J-02 or J-03 case cannot be expressed without an
  undocumented host-language semantic escape;
- the selected D-004 Core boundary cannot bind all eight journey identities
  without an unbounded or cyclic trust closure;
- preserving accepted S1-S3a behavior requires a disposable frontend rewrite;
- one mandatory adapter makes unrelated Orange meanings depend on its
  availability or upgrade schedule; or
- another candidate passes all eight hard gates with a strictly smaller
  complete dependency, trust, migration, and support burden.

## Operations, release, and recovery

This proposal adds no service, registry, deployment, key, build publication,
update channel, package, or release operation. Drafting, withdrawal, and
rejection are repository-document changes only.

If accepted product-form assumptions later prove false, the owner may stop the
affected capability, retain the last known-good edition and artifact readers,
withdraw unsupported claims, and propose a replacement OEP. History and
canonical artifacts must remain inspectable; recovery cannot silently reinterpret
old bytes under a new host system.

## Support and deprecation

Orange remains pre-alpha, best effort, and without an SLA, LTS period,
compatibility promise, migration service, or production-support claim. The
standalone form would be a durable architecture boundary, not a promise that
current syntax or internal Rust types are stable.

Each accepted edition, Core format, adapter, target, and release must state its
own support and deprecation terms. Withdrawing an adapter does not redefine the
Orange artifacts it once translated.

## Unresolved questions

The Orange Project Owner has not yet disposed this proposal or its hard gates.
That authority question blocks Accepted status.

D-004 must still decide the semantic strata, Core membership, shared pure
fragment, embeddings, erasures, and refinement relations. A later proposal must
select the first canonical Core encoding and version. D-006, S3b semantics,
targets, ABI, leakage, packages, releases, licensing, and the final public name
remain separate decisions.

No unresolved implementation detail permits the draft packet to claim owner
approval or to authorize dependent code.

## Current disposition

On 2026-07-13, OpenAI Codex co-authored and prepared this draft owner-review
packet under the active repository goal to advance the first unblocked roadmap
dependency. Chase Bryan is the named project author, champion, and sole
decision authority. The packet uses the existing D-003 recommendation,
accepted S1-S3a evidence, proposed user journeys, architecture, and solo support
envelope.

The comparison recommends PF-01, the standalone editioned Orange DSL with an
Orange-owned canonical Core boundary. It rejects manifest-only orchestration,
a proof-assistant embedding, and a Rust subset as the product form while
retaining each as a possible bounded interoperability or implementation
technique.

No owner review or approval is recorded. The packet has no OEP number, intake
or discussion reference, decision date, decision revision, approval record, or
change authority. Creating a numbered Draft OEP requires documented intake and
steward numbering. Acceptance requires explicit project-owner disposition and
exact reviewed evidence; a Codex review cannot supply that authority or
independence.

This packet does not accept D-003 or authorize S3b implementation.
