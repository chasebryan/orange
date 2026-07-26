# D-004 semantic-strata decision suite

Status: draft owner-executable decision protocol; no semantic-strata candidate
selected

Suite version: `d004-v0.3-draft`

Snapshot: 2026-07-26

## 1. Authority and decision boundary

This suite supplies the comparison protocol required by
[decision D-004](DECISIONS.md#d-004--semantic-strata). It turns the proposed
semantic roles and Core relationships into symmetric cases, typed crossings,
hard gates, resource rules, and an inconclusive outcome. It is decision
research, not a normative language specification.

On 2026-07-26 the owner explicitly accepted D-003 candidate PF-01, the
standalone Orange product form. That acceptance provisionally authorizes the
product-form boundary under [`GOVERNANCE.md`](../GOVERNANCE.md), but D-003
remains formally pending until an Accepted OEP binds the decision to an exact
revision. D-004 may collect conditional research, but it cannot be accepted
before that exact-revision D-003 closure. This suite does not accept D-004 or
authorize S3b implementation.

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

The current pre-epoch laboratory has byte-materialized exactly 39
candidate-neutral cross-cutting fixture subjects: 14 missing-edge subjects, 10
identity-substitution subjects, and five subjects each for ambiguity,
unsupported behavior, and domain exhaustion. Their status is
`materialized_unreviewed`. They are opaque suite-only models with fixed
identities and expected domain states, not accepted Orange syntax, semantics,
Core members, relations, or candidate observations. The five positive case
subjects and all 26 named mutation subjects remain definition-only gaps, and no
candidate mapping or adapter exists.

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
equivalent graph. Each edge has a versioned name, domain, codomain, definedness
conditions, obligations, identity inputs, trust role, failure behavior, and
prohibited reverse inferences.

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

An equivalent graph is not a waiver from this table. Every candidate supplies a
total SR conformance map with exactly one entry for each of SR-01 through SR-14.
Each entry names the candidate-native edge or edges, their direction, domain,
codomain, definedness conditions, obligations, identity inputs, trust role,
failure behavior, prohibited reverse inferences, and the observation that
demonstrates equivalence. A split edge records every constituent; a fused edge
records how each required crossing remains separately inspectable and
falsifiable. An omitted crossing, an unnamed extra crossing, or a many-to-one
mapping that hides a required authority or failure boundary fails the candidate.
The map and candidate-native graph are content-identified and frozen before
each recorded case run. A candidate-specific correction creates a linked new
run, retains the failed prior record, and consumes the common correction window.
Changing an SR requirement or the shared equivalence rule creates a new evidence
epoch for all candidates.

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

The frozen matrix therefore contains exactly 25 required candidate-case
executions per evidence epoch: each of the 5 candidates runs each of the 5
cases. A case has complete cross-candidate execution only when all five of its
candidate-specific records exist in the same epoch.

The prose below still defines, rather than materializes, the five positive case
subjects and 26 named mutation subjects. Byte materialization of the separate
39-subject cross-cutting corpus does not make any SC-01-through-SC-05 case
executable for a candidate, because candidate mappings and adapters are absent.

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

The gates are non-compensable. Only an overall case verdict of `pass` satisfies
SS-G05. Missing required execution, an unproven required positive capability,
an unexpected `unknown` or `unsupported` domain observation, or a replay-level
timeout or resource exhaustion forces the overall verdict to `fail` and makes
the candidate ineligible. As defined in section 7, a preregistered negative or
resource fixture may still pass when its domain observation is the expected
`unknown`, `unsupported`, or `exhausted`; that match demonstrates fail-closed
handling only and grants neither capability nor partial credit.

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

The repository may carry an exact input-only pre-epoch laboratory before that
freeze. Its `draft_unfrozen` packet may bind this suite, the accepted S3a
baseline, the 26 mutations named explicitly by SC-01 through SC-05, the
unreviewed structural proposal manifest, the exactly 39 corresponding
candidate-neutral byte-materialized subjects, and a deterministic 25-slot
candidate-case identity plan. The 39 subjects remain
`materialized_unreviewed`: 14 missing-edge, 10 identity-substitution, and five
each for ambiguity, unsupported behavior, and domain exhaustion. They carry
fixed bytes and expected domain states only. They contain no candidate mapping,
adapter output, observed state, comparison result, case verdict, evidence, or
capability credit.

The five positive case subjects and all 26 named mutation subjects remain
definition-only gaps. Sufficiency review for each of the five cross-cutting
classes remains unresolved, every class retains an active freeze blocker, and
replay repetitions remain unresolved and unassigned. Candidate-adapter
inability cannot satisfy an unsupported subject, and a domain-exhaustion
subject neither exercises a replay ceiling nor converts replay-level failure
into a matching observation.

Canonical parsing, digest verification, bounded structural checks, and
deterministic integrity-oracle evaluation of these opaque subjects are
repository-integrity operations, not candidate-case executions. They ratify no
Orange semantics and create no domain observation or decision evidence. The
laboratory executes no candidate adapter, creates no result or review record,
freezes no evidence epoch, and selects no candidate. D-003 is owner-accepted
pending exact-revision OEP closure; `owner_protocol_review` remains `none` for
D-004, and the D-004 epoch remains unfrozen.

Each candidate receives at most 24 owner-hours for its first complete case
packet and one four-owner-hour correction window. Time accounting is elapsed
focused work recorded by the owner; automation runs are separately bounded by
each case. Exceeding a budget records non-success. Changing these budgets after
candidate work starts creates a new epoch and restarts every candidate.

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
- the candidate-native relationship graph and total SR-01-through-SR-14
  conformance map used by the run;
- each normalized domain observation's separate expected state, observed state,
  and `matched` or `mismatched` comparison;
- one overall case verdict, exactly `pass` or `fail`;
- a byte manifest and replay instructions; and
- owner-produced and owner-reviewed labels, never an independent-review label.

Domain-observation states are `succeeded`, `rejected`, `unknown`, `timeout`,
`unsupported`, or `exhausted`. An expected `unknown`, `unsupported`, or
`exhausted` observation can match and contribute to a `pass`; it is never used
as the overall case verdict. Missing execution, an observation mismatch, an
unexpected non-success, or a replay-level timeout or exhaustion forces the
overall verdict to `fail`. A candidate adapter's inability to execute therefore
cannot masquerade as a correctly observed domain-level `unsupported` state.

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

Execution evidence is currently 0/25 required candidate-case executions: 0/5
candidates have complete five-case packets, and 0/5 cases have complete
cross-candidate execution.

The 39 byte-materialized cross-cutting subjects do not change that baseline.
No candidate mapping, adapter, observation, evidence, selection, or conclusion
exists, and materialization supplies no roadmap or readiness credit.

## 9. Current handoff

The next authorized actions are to close owner-accepted D-003 through an
Accepted exact-revision OEP, then review this conditional D-004 protocol and the
sufficiency of all five cross-cutting classes, materialize the five positive
and 26 named-mutation subjects, assign replay repetitions, and resolve the
remaining pre-epoch gaps symmetrically before freezing an evidence epoch.
Running the later frozen suite then produces decision evidence; it still does
not implement S3b.

Until those actions occur, D-004 remains proposed, the architecture role map
remains a recommendation, the S3a Typed Reference Core remains the only
implemented semantic boundary, and no proof, native-code, leakage,
cryptographic, compatibility, release, or production-readiness claim follows.
Orange remains 30% complete toward 1.0.0 by binary gate closure.
