---
number: OEP-0004
title: Standalone editioned Orange product form
authors:
  - Chase Bryan
champion: Chase Bryan
status: Accepted
type: Standards
created: 2026-07-26
updated: 2026-07-26
discussion: owner-direction-2026-07-26-d003-pf01
related-decisions:
  - D-001
  - D-002
  - D-003
  - D-004
  - D-005
  - D-006
  - D-023
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
  - OEP-0003
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: 2026-07-26
decision-revision: a82a5cec2ee4359dc2fe66171f17c93146747333
approval-records:
  - solo-reviewed owner direction Accept PF-01 accepted at merged revision a82a5cec2ee4359dc2fe66171f17c93146747333 on 2026-07-26
---

# OEP-0004: Standalone editioned Orange product form

## Abstract

Orange is a standalone, editioned domain-specific language for specifying,
implementing, and verifying cryptography. Orange owns its source semantics and
canonical Core family. Proof assistants, compilers, systems languages, and
other cryptographic tools may participate only through explicit, versioned
boundaries; none becomes the implicit definition of Orange program meaning.

On 2026-07-26 the Orange Project Owner supplied the exact direction
`Accept PF-01`. Under [`GOVERNANCE.md`](../../../GOVERNANCE.md), that direction
authorizes this product-form boundary. The reviewed candidate merged green as
exact revision `a82a5cec2ee4359dc2fe66171f17c93146747333`; this Accepted
record binds that revision. It does not accept D-004 or authorize S3b, a
release, or any technical assurance claim.

## Motivation

Orange exists to make the seams between specifications, implementations,
proofs, targets, artifacts, and evidence explicit. A manifest layered over
unrelated host languages would preserve those languages' competing semantics.
An embedding would make a proof assistant's parser and elaborator part of the
source authority. A Rust subset would couple specification meaning to one
implementation language. Each can remain useful behind a declared adapter, but
none supplies the single editioned semantic boundary required by D-001 and
D-002.

The complete owner-review basis is the
[D-003 product-form decision packet](../../PRODUCT_FORM_DECISION_PACKET.md).
It compares four candidates against eight non-compensable hard gates, maps all
eight proposed user journeys, preserves the accepted S1 through S3a lineage,
and records a solo-executable resource and migration analysis.

## Scope and non-goals

This OEP selects only the product form:

- one recognizably Orange source language with explicit editions;
- one Orange-owned, versioned family of Core boundaries;
- explicit adapters for every external semantic, proof, compilation, target,
  artifact, or evidence system; and
- fail-closed identity and provenance at every such boundary.

It does not select D-004 semantic strata, a canonical Core encoding, a proof
foundation or interchange, a solver policy, a compiler strategy, a target,
ABI, leakage model, package format, license, support term, or release name. It
does not make the eight proposed journeys complete. The current Typed Reference
Core remains private and noncanonical until a later accepted boundary replaces
or graduates it deliberately.

## Specification

PF-01 is the normative D-003 product form. Orange source bytes are interpreted
only under a named accepted Orange edition. Accepted Core subjects must name
their format/version and bind every semantic input needed to replay their
meaning. Same-looking names, files, or host-language values never create an
implicit relation.

Every external transition is a named, versioned adapter with exact source and
destination identities, a supported fragment, relation or observation type,
trust role, assumptions, unsupported cases, resource limits, and failure
behavior. Adapter failure invalidates only its dependent artifact or claim. It
cannot choose a second Orange meaning, silently fall back to another host
semantics, or upgrade external output into native Orange evidence.

The eight D-003 hard-gate dispositions are accepted as recorded in the
decision packet: PF-01 passes PF-G01 through PF-G08. PF-02, PF-03, and PF-04
remain ineligible as the product form because each has at least one failed or
unproven non-compensable gate. Their orchestration, proof-hosting, and systems
implementation techniques remain available behind separately admitted bounded
interfaces.

## Alternatives

PF-02, manifest-only orchestration, is rejected as the product form because it
does not itself supply a language semantic authority and leaves an expanding
polyglot trust and compatibility surface. Its orchestration techniques may
still assemble inputs and evidence with exact provenance.

PF-03, an Orange DSL embedded in F*, Lean, or Rocq, is rejected as the product
form because the host parser, elaborator, libraries, project format, and
upgrades would become semantic authority or require a permanent bridge to deny
that fact. A later accepted proof foundation may still host metatheory or an
authoritative checker.

PF-04, a Rust subset with proof annotations, is rejected as the product form
because it couples specification and implementation roles and inherits a
changing systems-language/compiler interpretation. Rust remains the directed
implementation language and may remain an interoperability boundary.

Reconsider PF-01 only if reproducible evidence triggers one of the packet's
falsification conditions or another candidate satisfies all eight gates with a
strictly smaller complete dependency, trust, migration, and support burden.

## Compatibility and migration

Accepted S1 through S3a source behavior remains valid. Existing Orange 2026
source continues through the permanent lexer, parser, Typed Reference Core,
and evaluator lineage. Before any private Core becomes canonical, a later OEP
must define its byte identity, versioning, migration, downgrade, and rejection
rules and provide a bounded migration from the private Rust structures.

External artifacts retain their original identities and provenance. Adopting
PF-01 does not relabel a Cryptol, hacspec, Jasmin, EasyCrypt, proof-assistant,
Rust, C, LLVM, or native artifact as Orange source or canonical Core. Rollback
preserves old bytes and readers; it never reinterprets history under a new host.

## Semantic and claim effects

The product-form choice fixes who may define Orange meaning, not the complete
meaning itself. It adds no construct, type, effect, proof judgment, target
behavior, ABI rule, leakage claim, or cryptographic theorem. It grants no
functional-correctness, safety, termination, constant-time, compiler,
interoperability, erasure, security, testing, audit, or certification claim.

Future claims must bind exact Orange source/Core identities and every explicit
external relation they depend on. A host tool's success, an owner review, or an
adapter label cannot substitute for the evidence type required by that claim.

## TCB, axiom, and proof effects

This decision prevents an external host from becoming an unnamed permanent
semantic authority. It does not yet reduce the implemented TCB, choose axioms,
select a proof assistant, admit a checker, or establish a proof. Each later
accepted Core, checker, adapter, compiler, and target boundary must publish its
own trust closure and fail closed when an identity or required relation is
missing.

The owner review recorded here is governance authority only. It is
`solo-reviewed`, never independent review, proof, audit, validation, or
certification.

## Threat, abuse, and leakage effects

PF-01 mitigates semantic-substitution and maturity-laundering risks by requiring
one named Orange authority and explicit bridges. An attacker or accidental
upgrade must not replace edition, Core, adapter, tool, environment, target, or
artifact identities while retaining a favorable result. Unsupported adapters
must fail at their boundary without changing unrelated Orange meaning.

These controls address the stable
[TM-005, TM-010, and TB-008](../../security/THREAT_MODEL.md) evidence
substitution, maturity-laundering, and hostile-frontend boundaries.

Residual risks include implementation defects in the Orange frontend and Core,
single-maintainer review, incomplete journeys, future adapter bugs, and open
semantic/target/leakage decisions. This OEP defines no leakage model and makes
no constant-time or side-channel claim.

## Target and ABI effects

No target, object format, CPU feature set, calling convention, foreign ABI, or
native output path is selected. Rust, C, LLVM, Jasmin, machine code, and foreign
consumers may become explicit implementation, interoperability, or target
boundaries only under their later accepted decisions. Until then, target and
ABI properties remain unsupported rather than inherited from a host tool.

## Standards, errata, and provenance

This product-form decision adopts no external technical standard or erratum as
Orange semantics. Later standards imports must cite exact editions and clauses,
retain source terms and transcription provenance, and distinguish intended
Orange meaning from external text and tool behavior. The D-003 packet and this
OEP are repository-native decision records under OEP-0001 and D-023.

## Dependencies, licenses, and IP

The decision requires only the already accepted solo-governance, parser, and
typed-literal OEPs. It installs no dependency and grants no acquisition,
copying, redistribution, patent, trademark, export, contribution, package, or
release authority. Every future external adapter or mandatory tool remains
subject to D-017 and D-018 before use in claim-bearing work.

## Conformance, tests, and evidence

The decision evidence consists of four named candidates, PF-G01 through
PF-G08, the 8/8 structurally specified but 0/8 complete journey mapping, the
accepted S1 through S3a permanent lineage, the private-to-canonical Core
migration boundary, the solo resource comparison, and the packet's explicit
reconsideration triggers.

The accepted decision revision keeps this OEP, D-003, the roadmap,
architecture, assurance, research, traceability, threat model, and policy
mutually consistent; preserves the noncanonical Typed Reference Core boundary;
and passed the full repository check and required hosted checks. This follow-up
record binds that merged revision. Mechanical success cannot manufacture the
owner authority already supplied or any technical evidence not present.

## Operations, release, and recovery

This Accepted decision creates no service, registry, deployment, signing
key, update channel, package, or release operation. If PF-01 assumptions later
fail, the owner may halt affected work, preserve the last readable edition and
artifacts, withdraw dependent claims, and propose a superseding OEP. Recovery
must preserve decision and artifact history.

## Support and deprecation

Orange remains pre-alpha, best effort, and without an SLA, LTS period,
compatibility promise, migration service, production-support claim, or release
authorization. PF-01 is a durable product-form boundary, not a promise that the
current syntax, private Core, CLI, or Rust implementation is stable. Each later
edition, Core format, adapter, target, and release must state its own support and
deprecation rules.

## Unresolved questions

The exact-revision D-003 closure is complete. D-004 must still select semantic
strata and Core membership. Later decisions must select canonical encodings,
proof and solver boundaries, compiler and target paths, ABI and leakage models,
packages, licenses, support terms, and release authority. None of those
questions changes the accepted disposition of PF-01.

## Decision record

On 2026-07-26 Chase Bryan, the Orange Project Owner and sole decision authority,
responded with the exact text `Accept PF-01` after being directed to use that
phrase to approve the standalone editioned Orange DSL and Orange-owned
canonical Core boundary. This accepts PF-01, its PF-G01-through-PF-G08 pass
disposition, and the recorded rejection of PF-02 through PF-04 as product forms
while retaining their bounded techniques.

The review is literally `solo-reviewed`. No independent person or organization
participated, and no such review is claimed. The fully validated candidate
merged green as exact revision
`a82a5cec2ee4359dc2fe66171f17c93146747333`. This Accepted OEP records that
`decision-revision` and binds it in the immutable approval record above, formally
closing D-003. D-004 and S3b remain unauthorized by this decision.
