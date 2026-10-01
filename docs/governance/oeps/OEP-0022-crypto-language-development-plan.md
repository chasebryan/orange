---
number: OEP-0022
title: Crypto language development plan
authors:
  - Chase Bryan
champion: Chase Bryan
status: Draft
type: Informational
created: 2026-09-30
updated: 2026-10-01
discussion: owner-direction-2026-09-30-crypto-language-development-plan
related-decisions:
  - D-002
  - D-004
  - D-005
  - D-006
  - D-007
  - D-009
  - D-010
  - D-011
  - D-012
  - D-013
  - D-015
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0022: Crypto language development plan

## Abstract

Orange should let a cryptographer define an algorithm in its mathematical
domain, then express and check the representations used to implement it. This
plan orders the language work needed for that relationship: static parameters
and shapes, mathematical representation definitions, checked contracts and
proof automation, field implementations, leakage-aware lowering, polynomial
quotient rings, and finite probability and games.

The owner requested this development plan on 2026-09-30. This informational
draft records that planning request and supplies work items and acceptance
criteria within the existing [roadmap](../../ROADMAP.md). It accepts no language
semantics, proof foundation, compiler strategy, target, or public claim. Each
implementation slice still needs its own bounded authority and conformance
record under the [OEP process](README.md). All work belongs to the permanent
compiler lineage; no disposable prototype or parallel implementation is planned.

## Motivation

Cryptol already has executable specifications, modular integers, size
polymorphism, property testing, automated proofs, and implementation verification
through SAW. Those capabilities are a comparison baseline, not a claim that
Orange has already surpassed it. The proposed language distinction is to make
mathematical domains and implementation representations explicit, composable
objects with reusable reasoning inside Orange.

The first substantive case is field arithmetic for X25519: one mathematical
value can have tight, loose, and canonical limb representations with different
operation preconditions. The second case is ML-KEM: one polynomial quotient
ring can have coefficient and transform representations with different
multiplication operations. These cases exercise the language relationship, not
only convenient notation.

## Scope and non-goals

The current accepted foothold is S3a. S3b through S3s are implemented and in
owner review under OEP-0005 through OEP-0021 and OEP-0023; implementation is not semantic
acceptance. The current evaluator admits `Int`, `Bool`, `Word[8]`, `Word[16]`,
`Word[32]`, `Word[64]`, `Mod[m]`, scalar arrays and rank-two arrays with
at most 65,536 scalar elements,
tuples, transparent type aliases, array concatenation and slicing, and bounded
size and type parameters within those proposed slices.
[OEP-0023](OEP-0023-orange-2026-nested-arrays.md) supplies bounded scalar-row
arrays and chained indexing as one P1/P6 vocabulary slice; arrays of matrices
and tuples remain unsupported. It has no refined representation types,
typed implementation bodies, proof
checking, or native output. The
[compiler guide](../../../compiler/README.md) and proposed
[modular arithmetic specification](../../MODULAR_2026.md) describe those limits.

This plan sets a sequence for new language work. It introduces no implemented
syntax, compatibility promise, dependency, license, release, certification, or
assurance claim. Milestone IDs below are work labels, not additional capability
gates or allocated language-slice numbers. The existing S0 through S8 stages
and 3-of-10 gate count remain the governing completion model.

## Specification

### Development order and dependencies

| Milestone | Permanent outcome | Existing stages | Dependency |
| --- | --- | --- | --- |
| P1 | Static parameters, shapes, tuples, and slicing | S3 | Applicable pure specification slice |
| P2 | Executable algebra and representation definitions | S3 | P1 features used by each definition |
| P3 | Checked representation contracts and reusable proofs | S4 | P2 and accepted proof, claim, and solver boundaries |
| P4 | Compositional field arithmetic and X25519 refinement | S4 and S7 | P3 and defined wide arithmetic |
| P5 | Lowering, effects, secrecy, and scoped leakage | S5 and S6 | Applicable P3/P4 contracts and selected compiler/target boundaries |
| P6 | Quotient-ring definitions and ML-KEM transform refinement | S3, S4, S5, and S7 | P2 for pure definitions; P3/P4 methods for checked transforms |
| P7 | Exact finite probability and bounded games | S3, S4, and S7 | A separate probability semantics and applicable proof boundaries |

Finish one bounded semantic slice at a time. Prioritize P1, P2, and the first
checked field operations before expanding the proof-bearing corpus to ML-KEM
or general game proofs. Pure P6 or P7 definitions may proceed while their
proof-bearing dependencies are unresolved. Open proof, compiler, or target
decisions create no aggregate freeze on proof-neutral S3 work.

P3 retains the S4 prerequisites, including D-004, D-005, D-006, D-007, D-009,
and the canonical Core boundary as applicable to the claimed result. P5 retains
the D-010 selection procedure and its acceptance prerequisites. D-011, D-012,
and D-013 gate their target, leakage, and foreign-boundary claims. This plan
selects no candidate and replaces no exact-revision acceptance requirement.

### P1 Static parameters and shapes

S3k, S3l, S3m, and S3o already implement parts of P1: tuples and tuple loop
state, concatenation and slicing, bounded size parameters, and finite type
parameters. Their records are OEP-0014, OEP-0015, OEP-0016, and OEP-0018.
Review these implementations against the criteria below before proposing
remaining parameter domains or constraints; this plan does not accept them.

Add static modulus and size parameters, then tuples, multiple loop accumulators,
array concatenation, and slicing as separate bounded slices. Static values must
be distinguishable from runtime values. Specify their scope, constraints,
specialization rules, diagnostics, and resource budgets before stabilization.
Keep mathematical integers, wrapping words, and residues distinct, with explicit
conversions. Reuse the existing total, deterministic specification semantics.

Acceptance criteria:

- One generic arithmetic definition evaluates at more than one modulus without
  mixing the resulting domains. Invalid or unsatisfied modulus constraints are
  rejected at the source use that supplies them.
- One ChaCha20 quarter-round definition works at multiple static positions in
  the state. Each instantiated index is checked against the state shape.
- Tuple construction, projection, and loop state have explicit types and
  evaluation order; slice bounds and concatenated lengths are checked.
- Concrete existing specifications preserve their results. Invalid shapes,
  runtime values supplied as static arguments, excessive specialization, and
  mismatched domains have deterministic rejection cases.

### P2 Executable algebra and representation definitions

Define representation storage, reconstruction functions, abstraction maps, and
range predicates as pure mathematical definitions first. Evaluate them using
the reference semantics. A Boolean predicate evaluating to true on examples
does not create a refinement type or prove a universal contract.

Use `Mod[m]` as a ring. Composite moduli are valid, and current division returns
zero for a non-unit. Field cancellation and inverse laws therefore require
explicit prime or unit evidence when proofs arrive. Do not silently strengthen
the existing meaning of `Mod[m]`.

For the first field case, define p = 2^255 - 19, B = 2^51, and five unsigned
limbs. Their mathematical reconstruction and abstraction are:

```text
N(x) = sum over i = 0..4 of Int(x[i]) * B^i
alpha(x) = N(x) modulo p
```

| Proposed representation state | Predicate | Intended use |
| --- | --- | --- |
| Tight | Every limb is below B | Inputs with bounded limb arithmetic |
| Loose | Every limb is below 2B | Arithmetic before carry propagation |
| Canonical | Tight, and N(x) is below p | Unique output encoding |

Tight is not canonical: a tight value can reconstruct to p and represent field
zero. The abstraction map is many-to-one. Input acceptance and canonical output
encoding are separate contracts.

Acceptance criteria:

- The predicates and abstraction are executable with documented step costs.
- Fixtures distinguish zero, p, p - 1, and the maximal tight limb array, and
  exercise each bound immediately below, at, and above its limit.
- The same field value can have multiple admitted mathematical representations.
- The definitions introduce no machine layout, timing guarantee, accepted
  refinement rule, or implicit enforcement through today's transparent aliases.

### P3 Checked representations and proof automation

After the applicable S4 decisions, introduce representation or refinement
semantics that enforce P2 predicates. Define introduction, weakening,
normalization, and abstraction rules. Existing `as` conversions must not
silently acquire proof-discharge or representation-cast behavior.

Every operation contract separately establishes defined machine computation,
intermediate arithmetic bounds, the output representation invariant, and
agreement with the mathematical operation under its input preconditions. Use
compositional lemmas rather than one monolithic query for an entire primitive.
Separate modular identities, integer inequalities, and bit-vector reasoning;
each successful proof must meet the selected checker and solver-trust policy.

Acceptance criteria:

- Check componentwise `add: Tight x Tight -> Loose` with field-sum refinement:
  each limb sum is below 2^52 and fits `Word[64]` without wrapping.
- Check `carry: Loose -> Tight` for the actual selected carry schedule, and
  `canonicalize: Tight -> Canonical`, each preserving alpha.
- Reject an operation whose algebraic result is correct but whose output bounds
  do not satisfy the next operation's precondition.
- Reject false range lemmas, changed abstraction maps, missing assumptions, and
  incompatible imported representation contracts.
- Successful checked obligations compose. Counterexamples identify source
  values and the failed obligation; timeout, unknown, and unsupported reasoning
  remain distinct from a disproved goal.

### P4 Field arithmetic and X25519 refinement

Specify exact wide multiplication and accumulation before implementing limb
multiplication. A proposed wide word or checked pair-of-words primitive needs
its own semantics; wrapping `Word[64]` multiplication is not an exact wide
product. Derive intermediate and output bounds for the chosen schedule.

Add subtraction, multiplication, squaring, and multiplication by the ladder's
constants with explicit preconditions. A biased subtraction or lazy product
must not inherit the loose bound by convention. Compose those contracts into a
ladder-state invariant and a relation to the existing `Mod[p]` specification.

Acceptance criteria:

- Every product, accumulation, bias, carry, and reduction has checked bounds
  appropriate to its actual arithmetic and operation schedule.
- The complete ladder refines the mathematical function for every accepted
  scalar and coordinate byte string, with termination and encoding obligations.
- Preserve RFC 7748 scalar clamping, coordinate top-bit masking, little-endian
  decoding, and acceptance of noncanonical coordinates interpreted modulo p.
  Canonical output must not cause canonical-input rejection.
- Keep an all-zero shared-secret rejection policy in its caller or profile
  contract, separate from raw X25519 behavior.
- Published vectors and boundary cases supplement the checked relation; they
  are not substituted for universal refinement evidence.

### P5 Lowering effects secrecy and leakage

Carry one field implementation through the D-010-selected output path. Specify
memory, failure, layout, ghost erasure, and other implementation effects at the
applicable stage. Secrecy qualifiers are orthogonal to arithmetic representation
states: public and secret loose values share their arithmetic bounds but have
different permitted observations.

Pure specifications may express mathematical choices naturally. A conditional
swap can be functionally correct while its implementation leaks; its leakage
obligation is separate. Apply the named observation model to branches,
addresses, termination, declassification, and classified instruction behavior.

Acceptance criteria:

- Lowering preserves the checked relation through every boundary included in
  its exact claim frontier, with the applicable memory and erasure obligations.
- Constant-time selection, carry propagation, and canonicalization have a
  separate scoped leakage argument. A secret-dependent branch or address
  cannot pass merely because arithmetic refinement succeeds.
- Target leakage claims name the instruction and platform assumptions, compiler
  preservation, and final artifacts that their frontier actually covers.
- Jasmin, C11, or LLVM output does not inherit downstream native guarantees.
  A native claim requires the additional target, ABI, and final-byte closure.

### P6 Quotient rings and ML-KEM transformations

Extend the mathematical vocabulary to polynomials, quotient rings, vectors,
and matrices with static shape constraints. Define coefficient ordering and
reduction before choosing a representation. These constructs should elaborate
to small, explicit semantic boundaries with reusable algebraic lemmas.

Use the ML-KEM coefficient field modulo q = 3329, with prime evidence where
field laws are used, and the quotient ring R = F_q[X] / (X^256 + 1). The
quotient ring does not inherit field division laws from its coefficients.
The bounded scalar-row surface of S3s supports polynomial vectors and
scalar matrices. Matrices whose entries are polynomials need a further
collection slice; rank three and structured array elements remain unsupported.

Acceptance criteria:

- Reference ring multiplication and quotient reduction are executable, with
  dimension and domain mismatch rejection cases.
- Coefficient, transform, Montgomery, and packed representations have distinct
  checked meaning and explicit conversion contracts.
- Transform plans state root, invertibility, ordering, and scaling conditions.
  ML-KEM uses FIPS 203's incomplete transform into 128 quadratic factors;
  512 does not divide 3328, so a full transform requiring a primitive 512th root
  is inapplicable. Bind the specified root 17 and factor ordering to the plan.
- Prove both inverse round-trip and multiplication refinement. ML-KEM's
  transform multiplication operates on quadratic pairs; scalar pointwise
  multiplication is insufficient. Round-trip alone cannot close this milestone.
- Derive bounds for each butterfly, reduction, product, and lazy stage. Specify
  any proposed signed storage or its unsigned encoding explicitly. Bind
  twiddles, inverse normalization, and Montgomery scaling to the representation.
- Claim this ring/transform result at its actual scope. Complete KEM behavior,
  randomness, decapsulation, and cryptographic security need further work.

### P7 Finite probability and bounded games

Define finite distributions with exact weights, normalization, independent
sampling, bind, events, and bounded oracle interactions in a distinct semantic
layer. A deterministic function and a probabilistic experiment need different
relations and proof rules. Ideal sampling is separate from an implementation's
entropy provider or pseudorandom generator.

Acceptance criteria:

- Exact evaluation agrees with checked elementary distribution identities.
  Non-normalized weights and invalid supports are rejected.
- Establish one bounded game transformation, such as the uniform-mask identity
  underlying a fixed-width one-time pad, with explicit independence assumptions.
- Bounded rejection sampling exposes exhaustion and accounts for its
  probability. A fallback value cannot silently preserve a uniformity claim.
- Games declare oracle interfaces and query bounds. Wider computational
  reductions state their assumptions and concrete advantage bounds explicitly;
  no deterministic implementation proof supplies them by implication.

### Immediate work items

- [ ] Review the implemented P1 size and type parameter slices: grammar,
  parameter domains, scope, shape constraints, specialization limits,
  diagnostics, and conformance; identify remaining work.
- [ ] Check a modulus-generic arithmetic specification and a statically
  positioned ChaCha20 quarter round against the P1 criteria as permanent fixtures.
- [ ] Review implemented tuples, tuple accumulators, concatenation, and slicing
  against the P1 criteria; keep any remaining semantic choices in separate slices.
- [ ] Define P2 reconstruction, abstraction, and tight/loose/canonical predicates
  with boundary fixtures and deterministic evaluation costs.
- [ ] Prepare the P3 operation-obligation inventory while the existing S4
  decision work progresses; select no proof foundation or solver by doing so.
- [ ] Define wide arithmetic and operation schedules before scheduling P4
  multiplication or complete X25519 implementation refinement.

## Alternatives

Adding syntax and mathematical type names without checked relationships would
improve transcription but leave the representation boundary informal. Starting
with a generic game prover would defer the concrete algebra/implementation
case. Starting with many targets or optimized primitives would multiply open
boundaries before operation contracts compose.

The proposed order starts with reusable language components and one field case,
then transfers those methods to polynomial rings and games. Revisit the first
case if its obligation inventory exceeds the selected proof fragment or another
standards-sourced case exercises the same boundary more tractably. Record that
evidence and preserve any completed permanent components.

## Compatibility and migration

This document changes development planning only. Existing source, evaluator
results, evidence formats, and accepted semantics retain their present status.
Future slices must specify compatibility and migration explicitly, particularly
for `as`, aliases, modular division, parameter instantiation, and encoding.

## Semantic and claim effects

No new semantics or claims are accepted by this informational draft. Its
milestones distinguish evaluation, range invariants, functional refinement,
standard conformance, distribution relations, leakage, and compiler preservation.
Completion requires the evidence appropriate to the precise result, not a
generic verified label. Research snapshots and existing acceptance records
remain unchanged.

## TCB, axiom, and proof effects

No current TCB or axiom changes. P3 onward must identify their actual semantic
definitions, checker rules, certificates, assumptions, and imported contracts.
Algebraic vocabulary must not introduce unrecorded trusted field or transform
axioms. Proof search and checked evidence remain separate under D-009.

## Threat, abuse, and leakage effects

No current attack surface changes. Future slice reviews must cover unsound
representation conversions, wrong bounds, mistaken field assumptions,
specialization resource exhaustion, encoding changes, and leakage introduced
by lowering. Follow the existing [threat model](../../security/THREAT_MODEL.md)
and [assurance model](../../ASSURANCE.md); functionally correct arithmetic does
not discharge a leakage or cryptographic-security obligation.

## Target and ABI effects

No target, object format, feature set, or ABI is selected. P5 inherits the
applicable D-010 through D-013 decisions and their claim frontier. Mathematical
representation definitions can be developed without selecting a native layout.

## Standards, errata, and provenance

The first cases use [RFC 7748](https://www.rfc-editor.org/rfc/rfc7748), sections
4, 5, and 6, for X25519 arithmetic, decoding, vectors, and caller behavior;
[RFC 8439](https://www.rfc-editor.org/rfc/rfc8439), section 2.1, for the ChaCha20
quarter round; and [FIPS 203](https://csrc.nist.gov/pubs/fips/203/final), sections
4.1 and 4.3, for ML-KEM ring operations and transform arithmetic. Each future
implementation slice must retain the exact standard edition, clause, errata
snapshot, archival identity, and transcription status it actually implements.
Links here are planning provenance, not completed standard-conformance evidence.

## Dependencies, licenses, and IP

No dependency or license is introduced. D-018 remains unresolved. A future proof
tool, generator, arithmetic implementation, or imported library needs its
applicable admission and provenance record before product use. The
[landscape analysis](../../RESEARCH.md) supplies comparison sources, not a
license to copy implementations.

## Conformance, tests, and evidence

For each slice, document normative rules, positive cases, rejected cases,
resource bounds, differential checks where another path exists, and exact
validation commands. Checked milestones additionally retain replayable proofs
or certificates and their assumptions. Record actual results and failures;
unchecked examples and vector passes cannot satisfy a proof obligation.

Measure manual lemma effort, obligation coverage, proof/checking cost, and
reference evaluation cost on the first cases. Compare with Cryptol plus SAW
only at matching property and assumption scope. Record measurements as
observations; this plan asserts no measured advantage or superiority.

## Operations, release, and recovery

No CI, key, release, or recovery mechanism changes. Documentation validation
applies to this proposal. Keep bound roadmap and decision-laboratory inputs
intact; later changes to them need their documented identity updates. Removing
this draft and its proposal-index entry rolls back the planning addition.

## Support and deprecation

No calendar, staffing, support, or compatibility commitment is added. One owner
performs the work under D-023. Outside contributors, reviewers, laboratories,
and organizations are not milestone prerequisites; their unavailable evidence
is not manufactured by a second tool or owner pass.

## Unresolved questions

- Remaining static-parameter domains and constraints, nominal versus structural
  representations, and proof-erasure rules need bounded slice records.
- Wide arithmetic, operation schedules, and their derived bounds remain choices
  to specify and check; the illustrative field states do not choose a backend.
- The existing semantic-strata, assurance, proof, solver, compiler, leakage, and
  target decisions retain their own procedures and acceptance dependencies.
- Polynomial vocabulary, transform contracts, exact probability semantics, and
  the initial bounded game fragment need separate evidence and conformance.

## Decision record

The owner requested a development plan on 2026-09-30 following discussion of
Orange's mathematical domains, checked representations, secrecy, and games.
This records that request, not a review or acceptance of the resulting draft.
The decision record remains incomplete while status is Draft; no exact revision
has been accepted and no approval record is supplied.

AI-assisted drafting: Codex using GPT-6 prepared this proposal and its index
entry from the owner discussion and repository documentation. The owner remains
the decision authority. AI output is not technical proof or independent review.
