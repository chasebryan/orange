# D-004 semantic-strata decision suite

Status: draft pre-freeze decision protocol; no semantic-strata candidate
selected and no evidence epoch authorized

Suite version: `d004-v0.2-draft`

Snapshot: 2026-07-13

## 1. Authority and decision boundary

This suite supplies the comparison protocol required by
[decision D-004](DECISIONS.md#d-004--semantic-strata). It turns the proposed
semantic roles and Core relationships into symmetric cases, typed crossings,
hard gates, resource rules, and an inconclusive outcome. It is decision
research, not a normative language specification.

D-003 remains proposed and its product-form decision packet remains unapproved.
D-004 therefore may collect conditional research, but it cannot be accepted
before the owner accepts or otherwise disposes of D-003. This suite does not
accept D-003 or authorize S3b implementation.

The currently accepted semantic boundary remains
[`SEMANTICS_2026.md`](SEMANTICS_2026.md). Its Typed Reference Core is an
internal, noncanonical S3a representation. It is not retroactively Spec Core,
Impl Core, a shared universal Core, or a proof language. Empty declarations
remain semantically empty, and equal `spec` and `impl` names create no
relationship.

This protocol does not select:

- the D-005 claim model;
- the D-006 proof foundation or calculus;
- the D-007 proof format or checker;
- the D-009 solver trust policy;
- the D-010 compiler-pass strategy;
- the D-011 host, target, ISA, or object-format envelope;
- the D-012 leakage observations or declassification policy;
- the D-013 ABI;
- the D-015 cryptography corpus;
- an S3b syntax, operator, call, binding, or evaluator boundary; or
- a canonical encoding, theorem fingerprint, package, release, license, or
  public product name.

Proof, compiler, target, leakage, ABI, and claim choices remain parameterized
where their own decisions are open. A candidate that can pass only by silently
choosing one of them fails this suite.

## 2. Candidate architectures

Every candidate receives the same frozen case packet, observation vocabulary,
resource limits, mutation set, and evidence schema. A candidate-specific
demonstration cannot replace a required case.

| ID | Candidate | Architecture under comparison | Current execution evidence |
| --- | --- | --- | --- |
| ST-REL | Role-oriented related family | Five source roles; Spec, Impl, and Game program Cores; CT and Machine compilation IRs; a parameterized proof-evidence interface | 0/5 cases |
| ST-UNI | Universal Core | One effect-parameterized calculus represents pure mathematics, state, probability, targets, and proofs | 0/5 cases |
| ST-DUAL | Pure/effect pair | One pure Core plus one general effect Core encodes implementation, games, machine behavior, and proof interaction | 0/5 cases |
| ST-MIRROR | Five mirrored Cores | One semantic Core mirrors each source declaration role and crossings connect the five Cores | 0/5 cases |
| ST-HOST | Host-delegated strata | Deterministic Orange semantics remain local while games, proofs, or machine meaning are delegated to external systems | 0/5 cases |

`ST-REL` is the current research recommendation, not a selection. It should be
falsified if its additional relations cannot be made smaller, clearer, and
more reviewable than the hidden effect conventions required by fewer formats.
`ST-UNI` is not rejected by assertion; it must fail a gate or mutation in the
same recorded packet as every other candidate.

The candidate packet freezes:

- exact candidate and case identifiers;
- the accepted S3a source, diagnostic, Core, and evaluation observations;
- a foundation-neutral notation for judgments, effects, traces, probability,
  memory, refinement, lowering, and evidence references;
- positive, forbidden, ambiguity, missing-relation, identity-substitution,
  resource-exhaustion, and unsupported cases;
- an input manifest covering every byte, path, mode, and expected observation;
- a correction window that applies equally to all candidates; and
- a variance log in which any changed premise creates a new evidence epoch.

### 2.1 Freeze-readiness review

The preceding list is a protocol requirement, not a statement that a frozen
packet already exists. The five prose cases are sufficient to review the
experiment's intended scope, but they are not executable fixtures. An evidence
epoch cannot start until the following closure artifacts exist at one exact
repository revision.

| ID | Blocking pre-freeze item | Required closure artifact |
| --- | --- | --- |
| FR-01 | The relationship table uses `ST-REL` names even though four candidates may organize the same obligations differently. | One neutral adapter contract plus a complete mapping from every candidate's internal members and routes to the required external observations. |
| FR-02 | Case inputs and expected observations are prose, not exact bytes. | A byte manifest with paths, modes, SHA-256 digests, model identities, and expected observation records for every positive and negative run. |
| FR-03 | Mutations have stable names below but no fixture identities or provenance. | One immutable fixture per mutation ID, linked to its base fixture and exact changed byte or structured field; generated fixtures also bind the generator and seed. |
| FR-04 | Section 7 enumerates result fields but no versioned suite record schema exists. | A versioned suite-index, run-result, and reproduction schema, or an admitted general evidence schema that expresses every required field without free-form substitutes, with positive, negative, and migration fixtures. |
| FR-05 | Replay principles do not yet identify an executable runner or resource observer. | Content-identified runner and observer artifacts specifying process-tree accounting, timeout and kill behavior, output counting, cache cleanup, temporary-storage accounting, and unsupported hosts. |
| FR-06 | The packet does not yet bind exact argument vectors, working directories, or the complete affecting environment. | A replay manifest per run and an environment allowlist whose omissions fail closed. |
| FR-07 | Candidate work order and correction events could create asymmetric learning or repair. | A preregistered authoring and replay order, equal candidate-local correction rules, retained failed runs, and a rule that any shared repair starts a new epoch for all candidates. |
| FR-08 | The conclusion rule permits a later distinguishing rule but does not make post-hoc within-epoch selection explicitly invalid. | A frozen decision record template stating that a new distinguishing rule changes the suite version, creates a new epoch, and reruns the complete common packet. |
| FR-09 | D-003, dependency admission, and top-level research inventory are not yet disposed for execution. | Owner-recorded conditional-research authority, the current D-003 dependency state, admitted tool/dependency terms, and the policy change that admits the real evidence tree. |

Freeze readiness is not currently established. These nine items measure
protocol closure, not candidate quality, and no candidate may gain execution
evidence by implementing against an unfrozen draft. Closing an item requires
the named artifact; restating its requirement in prose is not closure.

## 3. Proposed role map

The role map below is a hypothesis to test, not accepted semantics.

### 3.1 Source declaration roles

- **Specification** expresses pure, total mathematical meaning.
- **Implementation** expresses effectful procedures, contracts, memory, and
  typed failure.
- **Machine Implementation** exposes low-level operations and target-indexed
  obligations without selecting a target.
- **Game** expresses finite probabilistic experiments, adversaries, oracles,
  reductions, and advantage relations.
- **Proof** supplies evidence for a named judgment over exact semantic subjects;
  its calculus and durable representation remain D-006 and D-007 questions.

A `claim` is a later D-005 record binding a subject, relation, assumptions, and
evidence. It is not a sixth semantic stratum. Foreign and declassification
boundaries are cross-cutting declarations, not annotations that waive a
stratum's rules.

### 3.2 Candidate `ST-REL` semantic members

- **Shared Pure** is a versioned executable subset of Spec Core, not a universal
  Core and not a separately authoritative semantics.
- **Spec Core** gives authoritative pure and total program meaning.
- **Impl Core** gives authoritative stateful implementation meaning.
- **Game Core** gives authoritative probabilistic and adversarial meaning.
- **CT IR** is a proof-neutral compilation semantic boundary that retains
  memory, control-flow, effect, and later leakage-relevant observations.
- **Machine IR** is a target-parameterized compilation semantic boundary; D-011
  and D-013 still select concrete targets and ABIs.
- **Proof-evidence interface** names a judgment, exact subjects, assumptions,
  and evidence slot without choosing a proof calculus or wire format.

The three Cores are normative program-semantic domains. CT IR and Machine IR
are compilation-semantic domains. The proof-evidence interface reserves no
executable proof semantics and chooses no Proof IR.

## 4. Required relationship graph

Every candidate must express the following crossings or a demonstrably
equivalent graph. The crossing names use the `ST-REL` hypothesis as readable
shorthand; they do not require another candidate to copy its node count,
internal names, or representation boundaries. Each edge has a versioned name,
domain, codomain, definedness conditions, obligations, identity inputs, trust
role, failure behavior, and prohibited reverse inferences.

Before a packet freezes, every candidate must provide an adapter map for each
`SR-*` row. The map records the source role, authoritative meaning, required
external observations, candidate-internal route, failure point, and dependent
results invalidated by failure. Equivalent graphs are judged on those frozen
boundary behaviors. A missing `ST-REL`-named internal node is not itself a
failure, and a same-named node is not evidence that the behavior exists.

| ID | Required crossing | Mandatory boundary behavior |
| --- | --- | --- |
| SR-01 | Specification source to Spec Core | Elaboration either emits one checked pure subject or fails without creating an identity |
| SR-02 | Implementation source to Impl Core | Elaboration preserves contracts, effects, memory operations, and typed failure |
| SR-03 | Machine source to CT IR | Elaboration exposes low-level operations and unsupported features before target lowering |
| SR-04 | Game source to Game Core | Elaboration preserves sampling, oracle, adversary, and bound structure |
| SR-05 | Proof source to proof-evidence interface | Elaboration names an exact judgment and exact subject identities without choosing a proof calculus |
| SR-06 | Shared Pure into Spec Core | Inclusion is versioned, total on the subset, and cannot admit effects or sampling |
| SR-07 | Shared Pure into Impl Core | Explicit embedding cannot import state or infer refinement from a shared name |
| SR-08 | Shared Pure into Game Core | Explicit embedding cannot import ambient randomness or change deterministic meaning |
| SR-09 | Impl Core to Spec Core | A named refinement obligation relates explicit subjects and never follows from name equality |
| SR-10 | Impl Core to CT IR | Ghost erasure and lowering preserve runtime meaning or invalidate the dependent result |
| SR-11 | CT IR to Machine IR | Target-parameterized preservation records unsupported operations and assumptions without fallback claims |
| SR-12 | Game Core to Game Core | A named reduction or equivalence relates experiments and preserves its exact bound expression |
| SR-13 | Proof evidence to judgment | Checking binds the evidence to exact Core, IR, relation, model, and version identities |
| SR-14 | Claim record to subject and evidence | Later D-005 binding cannot upgrade a failed, missing, unknown, or unsupported relation |

These invariants apply to every graph:

- a shared source name never creates a refinement relation;
- sampling cannot enter Specification or Implementation through a pure
  embedding;
- state, memory, target, and ambient effects cannot enter Spec Core;
- proof or ghost data cannot affect runtime behavior;
- Machine source cannot bypass the checked low-level boundary;
- CT or Machine observations never silently become specification meaning;
- byte or format conversion is not semantic preservation; and
- a failed crossing invalidates its dependent result rather than producing a
  generic lower assurance level.

## 5. Required decision cases

Each candidate must run all five cases from the same frozen packet. Each case
records inputs, expected observations, positive and negative outcomes, exact
dependencies, resource use, and a falsification condition. A prose-only claim
that a case is representable is not execution evidence.

The variant IDs below are stable suite identities. `P00` is the positive run;
each `M*` ID is one independently replayed mutation of that case's positive
fixture. They name intended changes but do not become frozen inputs until
FR-02 and FR-03 close.

| Case | Positive variant | Mutation variants |
| --- | --- | --- |
| SC-01 | `SC-01-P00` exact word operations | `SC-01-M01` implicit integer-to-word conversion; `SC-01-M02` implicit endian conversion; `SC-01-M03` width mismatch; `SC-01-M04` unbounded shift |
| SC-02 | `SC-02-P00` valid in-place transformation | `SC-02-M01` illegal alias; `SC-02-M02` out-of-range access; `SC-02-M03` missing loop invariant; `SC-02-M04` uninitialized read; `SC-02-M05` wrong refinement subject |
| SC-03 | `SC-03-P00` public-control implementation | `SC-03-M01` secret-dependent branch; `SC-03-M02` secret-dependent address; `SC-03-M03` secret-dependent loop bound; `SC-03-M04` secret-dependent failure path; `SC-03-M05` secret-dependent debug observation |
| SC-04 | `SC-04-P00` supported vector operation | `SC-04-M01` missing feature; `SC-04-M02` unsupported intrinsic; `SC-04-M03` lane-order mismatch; `SC-04-M04` width mismatch; `SC-04-M05` target-identity substitution; `SC-04-M06` undeclared fallback |
| SC-05 | `SC-05-P00` explicit finite experiments and reduction | `SC-05-M01` sampling in Specification; `SC-05-M02` ambient randomness; `SC-05-M03` hidden oracle; `SC-05-M04` unbounded sample; `SC-05-M05` subject substitution; `SC-05-M06` altered bound |

A case passes only when its `P00` variant and every listed mutation complete
with their frozen expected observations. Resource-exhaustion, missing-input,
digest-mismatch, crash, and unsupported-path fixtures receive additional IDs
when the executable packet closes; they must not be hidden inside a listed
semantic mutation.

### SC-01 — SHA-like word code

**Question:** Can the candidate express pure SHA-like word operations without
confusing mathematical integers, words, bytes, byte order, or signedness?

**Dependencies:** Accepted S3a `Int` and `Word[8]` meaning; foundation-neutral
definitions of `Word[32]`, rotate, XOR, choice, modular addition, and endian
conversion. No standard algorithm or cryptographic claim is selected.

**Inputs:** One total round-like function, fixed observations for boundary word
values, and mutations introducing an implicit integer-to-word conversion, an
implicit endian conversion, a width mismatch, and an unbounded shift.

**Required boundary observations:** The authoritative pure stratum, every
conversion, normalized word result, rejection category and location, semantic
subject identity, and every crossing used by evaluation.

**Positive case:** Exact word operations elaborate and evaluate
deterministically; repeated runs produce the same observations and preserve the
accepted meaning of S3a literals.

**Mutation and negative case:** Each implicit conversion, width mismatch, and
invalid shift rejects at the authoritative boundary. No candidate may recover
by silently treating a word as an integer or byte sequence.

**Resource bounds:** One case replay has at most 15 minutes wall time, 4 GiB
peak resident memory, 2 GiB temporary storage, and 256 MiB captured output.
Timeout, exhaustion, or oversized output is non-success.

**Non-claims:** The case establishes no SHA conformance, cryptographic security,
proof, constant-time behavior, code generation, or canonical Core encoding.

**Falsification:** The candidate fails this case if two incompatible numeric
meanings share one unchecked term or if a forbidden mutation remains accepted.

### SC-02 — Mutable-buffer refinement

**Question:** Can the candidate keep pure meaning separate from mutable memory
while stating one explicit implementation-to-specification obligation?

**Dependencies:** A small pure buffer transformation, a foundation-neutral
owned and borrowed buffer model, bounds, typed failure, a loop invariant, and a
named but unproved refinement relation. D-005 claim composition is not needed.

**Inputs:** One in-place transformation over an owned mutable slice plus alias,
out-of-range, missing-invariant, uninitialized-read, and wrong-refinement-subject
mutations.

**Required boundary observations:** Authoritative Spec and Impl subjects,
ownership and region state, effects, failure paths, loop obligations, the exact
refinement pair, and every erased or lowered value.

**Positive case:** Pure and mutable meanings remain distinct; the valid program
produces a deterministic implementation observation and one explicit open or
discharged refinement obligation.

**Mutation and negative case:** Illegal aliasing, range failure,
uninitialized access, missing invariant, and subject substitution reject or
leave the exact obligation unsatisfied. Equal declaration names prove nothing.

**Resource bounds:** One case replay has at most 15 minutes wall time, 4 GiB
peak resident memory, 2 GiB temporary storage, and 256 MiB captured output.
Timeout, exhaustion, or oversized output is non-success.

**Non-claims:** The case establishes no accepted memory model, solver result,
proof, ABI, native safety, leakage property, or public refinement claim.

**Falsification:** The candidate fails this case if mutable operations enter the
pure meaning, invalid memory remains accepted, or refinement follows from a
name or format conversion.

### SC-03 — Secret-dependent rejection

**Question:** Does the architecture preserve enough boundary information for a
later leakage policy to reject secret-controlled behavior without pretending
that D-004 selects that policy?

**Dependencies:** A suite-only public/secret parameter, control-flow and address
observations, and one policy hook. These fixtures exercise but do not ratify
D-012, declassification, target timing, or a constant-time claim.

**Inputs:** One public-control implementation and mutations containing a
secret-dependent branch, address, loop bound, failure path, and debug
observation.

**Required boundary observations:** The authoritative implementation subject,
the crossing at which each control or memory observation remains visible, the
parameterized policy identity, rejection or unknown state, and dependent-result
invalidation.

**Positive case:** The public-control fixture crosses the boundary without
erasing the observations a later selected leakage model needs. Its result is
labeled only as a suite observation, not leakage evidence.

**Mutation and negative case:** Every secret-dependent mutation remains visible
and is rejected or reported as unsupported or unknown by the suite policy hook.
It never becomes a successful security claim.

**Resource bounds:** One case replay has at most 15 minutes wall time, 4 GiB
peak resident memory, 2 GiB temporary storage, and 256 MiB captured output.
Timeout, exhaustion, or oversized output is non-success.

**Non-claims:** The case selects no leakage trace, declassification rule,
target profile, timing model, speculative model, side-channel scope, or
constant-time claim.

**Falsification:** The candidate fails this case if a low-level observation is
irreversibly erased before the policy boundary or any unknown becomes success.

### SC-04 — Vector intrinsic

**Question:** Can pure lane meaning and target-specific machine behavior remain
distinct while an intrinsic is related to its exact abstract operation?

**Dependencies:** A foundation-neutral fixed-lane vector operation, one abstract
feature identifier, explicit lane and word order, and a parameterized target
model. No concrete ISA, ABI, host, or target tuple is selected.

**Inputs:** One vector operation with a scalar pure meaning plus missing-feature,
unsupported-intrinsic, lane-order, width, target-identity, and fallback
mutations.

**Required boundary observations:** Pure lane result, low-level operation,
feature and target-model identity, lowering relation, unsupported state,
preservation obligation, and any fallback selected by the input.

**Positive case:** The abstract intrinsic remains distinct from its pure meaning
and produces an exact target-parameterized obligation. A declared fallback is
checked as a separate path rather than inferred.

**Mutation and negative case:** Missing features, unsupported operations,
identity substitution, lane or width mismatch, and undeclared fallback reject
or remain unsupported without inheriting the pure result as machine evidence.

**Resource bounds:** One case replay has at most 15 minutes wall time, 4 GiB
peak resident memory, 2 GiB temporary storage, and 256 MiB captured output.
Timeout, exhaustion, or oversized output is non-success.

**Non-claims:** The case selects no target, instruction encoding, ABI, compiler
pass, performance property, code-generation path, or native preservation proof.

**Falsification:** The candidate fails this case if target behavior is hidden in
the pure meaning or unsupported lowering silently becomes a portable success.

### SC-05 — Game and reduction relation

**Question:** Can a probabilistic game reuse deterministic pure definitions
without importing sampling into Spec Core or reducing a security statement to
ordinary runtime randomness?

**Dependencies:** One Shared Pure primitive, finite explicit sampling, an
adversary and oracle boundary, symbolic probability expressions, and a named
reduction relation. No proof foundation or cryptographic theorem is selected.

**Inputs:** Two small experiments and a symbolic advantage relation plus
sampling-in-Spec, ambient-randomness, hidden-oracle, unbounded-sample,
subject-substitution, and altered-bound mutations.

**Required boundary observations:** Exact Game subjects, imported pure subject,
sample space, adversary and oracle interface, failure behavior, probability and
advantage expressions, reduction direction, bound, and evidence state.

**Positive case:** Shared Pure meaning is imported through an explicit
embedding; sampling remains authoritative only in Game semantics; the named
reduction records exact endpoints and a symbolic bound.

**Mutation and negative case:** Sampling in Spec Core, ambient randomness,
hidden oracle effects, unbounded sampling, endpoint substitution, and bound
changes reject or invalidate the exact relation.

**Resource bounds:** One case replay has at most 15 minutes wall time, 4 GiB
peak resident memory, 2 GiB temporary storage, and 256 MiB captured output.
Timeout, exhaustion, or oversized output is non-success.

**Non-claims:** The case establishes no cryptographic reduction, probability
bound, proof, theorem, solver result, corpus membership, or public security
claim.

**Falsification:** The candidate fails this case if probabilistic meaning leaks
into the pure stratum, the relation can change endpoints without failure, or an
unchecked symbolic bound is described as proved.

## 6. Hard gates and anti-gaming rules

The gates are non-compensable. `Unproven`, missing, timeout, unsupported, and
resource exhaustion are failures for candidate selection, not partial credit.

1. **SS-G01 — Product meaning:** all five source roles and the semantic needs of
   J-01 through J-08 remain expressible without changing journey identities or
   claiming the journeys are complete.
2. **SS-G02 — One authority:** every construct has one authoritative semantic
   member; conflicting judgments are not hidden as annotations or modes.
3. **SS-G03 — Complete crossings:** SR-01 through SR-14 have exact domains,
   codomains, obligations, identity rules, trust roles, failure behavior, and
   prohibited reverse inferences.
4. **SS-G04 — S3a compatibility:** all accepted S3a observations and non-claims
   survive a bounded migration; private IDs stay noncanonical, empty
   declarations stay meaningless, and equal names imply no relation.
5. **SS-G05 — Permanent cases:** SC-01 through SC-05 pass with every mutation,
   ambiguity, missing-edge, identity-substitution, unsupported, and resource
   case recorded.
6. **SS-G06 — No preemption:** proof, claim, solver, compiler, target, leakage,
   ABI, corpus, canonical-format, package, release, and S3b choices remain
   parameterized where their decisions are open.
7. **SS-G07 — Exact identity:** every public subject and relation is versionable
   and binds all semantic inputs without selecting a canonical encoding.
8. **SS-G08 — Solo execution:** one owner can author, replay, inspect, and
   archive the full comparison; no unavailable person or organization is an
   entry condition.
9. **SS-G09 — Permanent lineage:** the selected structure gives S3b one bounded
   production-lineage destination and a migration path without implementing or
   authorizing S3b.
10. **SS-G10 — Acceptance closure:** the owner disposes every candidate and gate
    at one exact revision; affected normative documents and change records
    agree; and all required repository checks pass at that revision.

There is no weighted aggregate score. Fewer formats, fewer relations, smaller
documents, or faster prototypes cannot compensate for one failed hard gate.
Likewise, additional layers cannot compensate for an unnamed or unjustified
crossing.

## 7. Evidence, resource, and replay contract

Substantive execution evidence belongs under `research/decisions/D-004/` after
an evidence epoch intentionally admits that top-level research inventory. This
draft creates no empty evidence directory and does not widen repository policy
for results that do not yet exist.

Each candidate receives at most 24 owner-hours for its first complete case
packet and one four-owner-hour correction window. Time accounting is elapsed
focused work recorded by the owner; automation runs are separately bounded by
each case. Exceeding a budget records non-success. Changing these budgets after
candidate work starts creates a new epoch and restarts every candidate.

For resource accounting, one replay is one candidate, one case, and one
variant ID. The stated 15-minute wall-time, 4-GiB peak-resident-memory, 2-GiB
temporary-storage, and 256-MiB captured-output ceilings apply independently to
every replay, including resource and unsupported fixtures. Captured output is
the combined uncompressed stdout, stderr, and runner-owned raw log bytes before
normalization. Peak memory and timeout cover the complete descendant process
tree; temporary storage covers every candidate-specific writable path. The
frozen observer defines platform-specific enforcement, termination, descendant
cleanup, and what happens when a measurement cannot be made reliably.

Candidate-specific authoring, debugging, and inspection consume that
candidate's owner-hour budget. Work on the common protocol consumes neither
candidate budget. Waiting for unattended automation does not consume focused
owner-hours, but inspecting or repairing its result does. A correction may
change only candidate-local artifacts under the frozen contract and retains
the failed record. A shared fixture, observer, schema, rubric, resource rule,
or decision-rule change creates a new epoch and resets all candidate evidence.

Automated replay uses argument vectors rather than shell strings, a declared
allowlisted environment, pinned tool and input digests, network denied, an
empty candidate-specific cache, deterministic output manifests, and explicit
non-success for missing input, timeout, resource exhaustion, crash, digest
mismatch, or unsupported behavior. Capture may use a network only before the
frozen replay epoch and only under the dependency policy.

Each case record contains:

- suite, epoch, candidate, case, and mutation identifiers;
- input, model, tool, dependency, and environment digests;
- exact arguments, resource ceilings, measured resource use, and exit state;
- normalized observations plus raw bounded logs;
- every premise, assumption, trusted component, and unsupported feature;
- pass, fail, unknown, timeout, unsupported, or exhausted state;
- a byte manifest and replay instructions; and
- owner-produced and owner-reviewed labels, never an independent-review label.

A candidate adapter may use research-only models, but it cannot enter the
product lineage by accident. After selection, the five accepted cases must be
rewritten or graduated deliberately as permanent conformance fixtures with
reviewed provenance. Rejected candidate artifacts remain replayable research
evidence and do not become a parallel Orange implementation.

## 8. Candidate disposition

A candidate is eligible only if all five cases and all ten hard gates pass in
one evidence epoch. A failed relation invalidates only the dependent result; it
does not create a generic numeric assurance downgrade.

The suite conclusion is exactly `recommend_st_rel`, `recommend_st_uni`,
`recommend_st_dual`, `recommend_st_mirror`, `recommend_st_host`, or
`inconclusive`. A recommendation identifies the complete evidence epoch and
does not accept D-004. If zero or multiple candidates pass, the result is
`inconclusive` until the owner records a non-compensable distinguishing rule or
revises and reruns the common suite.

A distinguishing rule cannot select a candidate from the epoch in which it was
invented. It changes the suite version, starts a new evidence epoch, and applies
to the complete common packet for all candidates. Candidate-specific evidence
cannot be carried forward unless its bytes and every affecting premise remain
identical and the new rule explicitly admits that reuse for every candidate.

Acceptance requires:

- accepted disposition of D-003 and its product-form record;
- a complete D-004 evidence epoch;
- explicit owner disposition of every candidate, case, relation, variance, and
  hard gate;
- a D-004 standards OEP created only after real intake and steward numbering;
- a `solo-reviewed` owner approval bound to an exact 40-hex revision;
- synchronized D-004, architecture, roadmap, traceability, reader, and OEP
  records; and
- the repository's required local and hosted evidence at that exact revision.

No semantic stratum is selected by this draft suite. Independent review is
currently absent. That absence limits any independent or external feasibility
claim, but under D-023 it is not replaced with a fictional reviewer and does
not prevent the owner from executing the comparison.

Execution evidence is currently 0/5 candidates and 0/5 cases.

## 9. Current handoff

The next authorized actions are to obtain owner intake and disposition for
D-003, review this conditional D-004 protocol, and close or revise FR-01 through
FR-09 before freezing an evidence epoch. Only the resulting frozen suite may
produce decision evidence; it still does not implement S3b.

Until those actions occur, D-004 remains proposed, the architecture role map
remains a recommendation, the S3a Typed Reference Core remains the only
implemented semantic boundary, and no proof, native-code, leakage,
cryptographic, compatibility, release, or production-readiness claim follows.
