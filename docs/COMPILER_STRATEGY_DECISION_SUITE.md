# D-010 compiler-strategy decision suite

Status: owner-executable draft under D-023; no compiler strategy selected

Suite version: `d010-v0.1-draft`

Snapshot: 2026-07-26

## Solo-mode disposition

This suite compares five compiler and output-path strategies without inventing
candidate teams, independent reviewers, or external audit. The Orange Project
Owner may implement and review every candidate under D-023, but each such
record is `solo-reviewed`. A second owner pass, workspace, tool, or
implementation remains same-owner evidence and never becomes independent
review or reproduction.

The checked-in pre-epoch inputs are limited to this candidate-neutral suite and
an eight-row missing-input index. They admit, acquire, install, or execute no
compiler, proof assistant, solver, checker, assembler, linker, adapter, runner,
observer, emulator, or isolation backend. They freeze no evidence epoch,
validate no transformation or artifact, and record no candidate result. The
exact execution baseline is 0/40 candidate-case runs.

## 1. Decision boundary

This protocol supplies the candidate-neutral comparison required by
[D-010](DECISIONS.md#d-010--compiler-strategy). It decides which compiler and
output-path policy Orange may adopt, where that policy's claim frontier ends,
and which authority may justify each transformation. It does not itself select
an implementation language, proof format, solver, target tuple, leakage model,
ABI, object format, package format, cryptographic primitive, or release
envelope.

D-003, D-004, D-005, D-006, and D-009 must be Accepted before D-010 can be
Accepted. They respectively fix the product boundary, semantic strata, atomic
claim model, proof foundation, and solver authority needed to interpret the
compiler comparison.

D-007, D-011, D-012, and D-013 are downstream decisions, not D-010 acceptance
prerequisites. Requiring them first would create cycles or allow a later target,
leakage, ABI, or Proof IR choice to predetermine the compiler strategy. Before
those decisions close, measured D-010 work uses bounded suite-only Proof IR,
target, leakage, ABI, and object models applied identically to every candidate.
Those research models accept none of the downstream decisions and authorize no
product claim.

No third-party tool or toolchain may be acquired for measured execution until
the owner records the applicable D-018 admission with exact source, version,
digest, dependency graph, terms, capabilities, offline availability, recovery
plan, and logical- and operational-TCB effect. A system compiler, Jasmin,
LLVM, assembler, linker, emulator, proof assistant, solver, and checker are all
subject to this rule when applicable.

## 2. Candidate parity and frozen inputs

| ID | Candidate | Required treatment | Current execution evidence |
| --- | --- | --- | --- |
| CP-01 | Theorem/certificate hybrid direct-native path | Mechanize stable structural passes; permit changing optimizers, schedulers, vectorizers, and allocators outside the logical TCB only when each accepted result carries checked functional and leakage evidence; emit and validate direct native objects | 0/8 cases |
| CP-02 | Mechanized proof-per-pass direct-native path | Require a reusable mechanized preservation theorem for every claim-bearing pass, including optimization and allocation; emit and validate direct native objects | 0/8 cases |
| CP-03 | Versioned Jasmin backend boundary | Translate to one exact Jasmin language/version boundary and stop there unless a checked relation through one exact D-018-admitted Jasmin compiler/toolchain is separately frozen and accepted to extend the frontier; disclose its theorem and toolchain scope and close every seam actually claimed | 0/8 cases |
| CP-04 | Portable C11 interoperability boundary | Emit deterministic portable C11 with an explicitly weaker frontier that ends before generic downstream C compilation unless a checked relation through one exact D-018-admitted C pipeline is separately frozen and accepted to extend it | 0/8 cases |
| CP-05 | Versioned LLVM IR interoperability boundary | Emit one exact LLVM IR version and data-layout contract with an explicitly weaker frontier that ends before generic LLVM optimization or code generation unless a checked relation through one exact D-018-admitted LLVM pipeline is separately frozen and accepted to extend it | 0/8 cases |

The frozen matrix contains exactly 40 candidate-case runs per evidence epoch:
each of the 5 candidates runs each of the 8 cases. Similar implementations or
outputs do not merge candidates. Portable C11 and LLVM IR remain distinct
candidates because their semantic boundaries, versioning, optimizer behavior,
undefined-behavior risks, target assumptions, and failure modes are different.
Evidence from one may not be cherry-picked to repair the other.

Every candidate receives the same candidate-neutral packet containing:

- exact source, semantic-layer, pass, claim, theorem, certificate, target,
  leakage, ABI, object, toolchain, environment, and endpoint identities;
- one frozen pipeline graph and authority classification for every edge;
- a preregistered candidate class and claim frontier, including every promised
  functional, leakage, target, ABI, and final-artifact proposition;
- the four atomic outcomes `satisfied`, `not_satisfied`, `unresolved`, and
  `unsupported`, without numeric ordering or aggregate upgrade;
- valid, malformed, corrupted, substituted, missing, unsupported, timeout,
  crash, resource-exhaustion, and fallback inputs;
- suite-only bounded downstream models used symmetrically without accepting
  D-007, D-011, D-012, or D-013;
- exact resource ceilings, timeout semantics, network and cache policy, and an
  evaluation-host and target matrix frozen before measured work;
- an input-manifest digest covering every byte, mode, path, case ID, candidate
  mapping, tool identity, and expected-observation rule; and
- M-01 through M-19, AX-01 through AX-09, the hard gates, materiality bands,
  and total decision rules in this document.

Candidate adapters may implement only their declared policy. They may not
change a source program, semantic endpoint, target model, observation rule,
claim wording, permitted authority, negative case, timeout, or resource limit.
Candidate-specific setup is recorded as variance and receives no silent credit.

The atomic outcomes inherit D-005 exactly:

- `satisfied`: the exact proposition has its complete permitted mandatory
  closure and no valid decisive negative result;
- `not_satisfied`: permitted, identity-bound negative evidence establishes that
  the exact proposition is false or violated within its scope; absence or
  incompleteness alone is not `not_satisfied`;
- `unresolved`: the claim is well-formed and within the declared support model,
  but a required decision remains unknown, incomplete, conflicting, or
  exhausted; and
- `unsupported`: the declared policy or support envelope offers no permitted
  evaluation or authority path for that exact claim and scope.

A claim-level `unsupported` outcome beyond an intentionally frozen candidate
frontier can be the required correct observation while the applicable hard gate
passes. It must not be converted to success or used to imply the stronger
claim. A candidate that preregisters a native-assurance frontier may not shrink
that frontier after observing adverse evidence. A candidate with an explicitly
weaker interoperability frontier is compared honestly at that frontier and may
not borrow a native claim from another candidate.

One evidence epoch freezes all inputs, expected observations, pipeline graphs,
claim frontiers, authority rules, resource policy, candidate order, correction
window, and materiality bands before execution. A shared ambiguity or rule
change creates a new symmetric epoch. Candidate corrections retain the
original failed record.

Two separately provisioned owner workspaces use distinct checkouts, caches,
output roots, acquisitions, and toolchain roots. Matching deterministic results
may reach same-owner reproducibility level 2 only. Neither workspace populates
an independent-reproduction field.

## 3. Required decision cases

### CC-01 — Freeze pipeline, authority, and claim frontier

**Question:** Can the candidate define one complete, unambiguous path from the
accepted semantic input to its declared endpoint, with an authority and claim
frontier fixed for every edge before execution?

**Dependencies:** Accepted D-003, D-004, D-005, D-006, and D-009 records;
suite-only downstream models; and the frozen candidate-class vocabulary.

**Shared inputs:** One canonical pipeline request, all semantic endpoints,
required pass classes, target/leakage/ABI/object placeholders, claim set,
authority roles, and competing, missing-edge, reordered-edge, and
frontier-shrink mutations.

**Candidate outputs:** A canonical pipeline graph, edge authority ledger,
logical and operational TCB, exact endpoint, supported/unsupported claim
frontier, assumptions, exclusions, and input/output identity rules.

**Positive checks:** Every required edge occurs exactly once in order; every
claim names a complete path to its declared endpoint; every theorem,
certificate, external tool, and assumption has one honest authority role.

**Mutation and negative checks:** Omit or reorder an edge, merge C11 and LLVM
boundaries, move an external compiler outside the TCB without checked output,
extend a claim past its endpoint, or shrink a native frontier after failure.

**Hard acceptance:** The graph, authority ledger, and frontier are complete,
closed, mutually identity-bound, and frozen before any measured candidate run.

### CC-02 — Preserve functional semantics through structural lowering

**Question:** Does the candidate preserve exact functional behavior through
elaboration validation, erasure, monomorphization, memory/region lowering,
control lowering, and construction of its low-level endpoint?

**Dependencies:** Accepted semantic strata and proof foundation, canonical
suite-only source/Core/IR models, executable reference semantics, and explicit
undefined-behavior and failure policy.

**Shared inputs:** Positive and negative scalar, word, memory, region, control,
alias, endianness, trap, termination, and boundary-width programs with exact
reference observations and structural-pass mutations.

**Candidate outputs:** Bound intermediate artifacts, pass evidence, functional
observations, assumptions, diagnostics, and an exact refinement chain through
the candidate's declared endpoint.

**Positive checks:** Each supported transformation agrees with the reference
semantics and uses only the candidate's declared theorem, checked certificate,
or explicitly weaker interoperability basis.

**Mutation and negative checks:** Change a width, sign, byte order, alias,
region, trap, termination result, pass order, semantic version, or artifact
binding; introduce undefined or host-dependent behavior.

**Hard acceptance:** All promised functional propositions close at the frozen
frontier, and every malformed, changed, or unsupported proposition retains its
exact non-successful outcome.

### CC-03 — Validate optimization, scheduling, vectorization, and allocation

**Question:** Can the candidate transform low-level programs for performance
without giving unverified search procedures undeclared claim authority?

**Dependencies:** Suite-only low-level semantics, pass contracts, register and
vector models, target-feature placeholders, D-009 solver policy, and the
candidate's frozen edge authorities.

**Shared inputs:** Equivalent scalar/vector kernels, schedules, allocation
constraints, spills, flags, aliasing cases, feature variants, invalid
certificates, stale proofs, and intentionally miscompiled results.

**Candidate outputs:** Optimized artifacts, theorem or certificate identities,
checker results, feature and allocation records, functional/leakage
observations, resource data, and diagnostics.

**Positive checks:** Every accepted result follows the candidate's declared
authority path and preserves all applicable functional and leakage
propositions for the exact inputs and target profile.

**Mutation and negative checks:** Substitute a source, schedule, register map,
target feature, certificate, theorem, checker, or output; omit a spill or flag;
reuse stale evidence; or accept a search tool's favorable assertion directly.

**Hard acceptance:** All promised optimized results are replayably justified,
and no convenience, performance gain, or aggregate score compensates for an
incorrect or unauthorized transformation.

### CC-04 — Preserve a named leakage model

**Question:** Does the candidate preserve the exact frozen leakage proposition
through every claimed transformation and endpoint, without treating ordinary
functional equivalence as leakage evidence?

**Dependencies:** Accepted D-005 outcome rules, suite-only D-012 architectural
trace model, target instruction classification, declassification policy, and
candidate frontier.

**Shared inputs:** Paired public-equal/secret-different executions covering
branches, addresses and widths, indirect targets, traps, termination,
variable-latency classifications, vector lowering, dispatch, and deliberately
leaking transformations.

**Candidate outputs:** Bound leakage traces or checked relations, target/profile
identity, declassification record, pass-by-pass disposition, endpoint claim,
assumptions, exclusions, and exact adverse diagnostics.

**Positive checks:** Every promised leakage claim uses a permitted authority
and reaches the declared endpoint for the exact target and profile.

**Mutation and negative checks:** Hide a branch or address, alter an instruction
classification, omit a trace event, change a target, reuse functional evidence,
silently select a fallback, or relabel an unsupported leakage claim.

**Hard acceptance:** All claims within the frozen frontier have exact complete
outcomes; a deliberately violating case is rejected, and every claim beyond an
honestly weaker frontier remains explicitly `unsupported`.

### CC-05 — Close the endpoint and final-object last mile

**Question:** Does each claim reach the candidate's exact declared endpoint,
and does every candidate promising native assurance close the path through
decoded final object bytes?

**Dependencies:** Suite-only D-011 target/object and D-013 ABI models, canonical
encoder/decoder relations, relocation and symbol policy, and frozen frontier.

**Shared inputs:** Machine artifacts, portable C11 and LLVM IR outputs, Jasmin
boundary records, sections, permissions, constants, relocations, symbols,
stack/call behavior, feature dispatch, wrappers, and deliberately corrupted or
substituted endpoint artifacts.

**Candidate outputs:** Exact endpoint bytes and digest, decoder/validator
records where applicable, external toolchain and link boundary, ABI and symbol
observations, final claim disposition, assumptions, and TCB closure.

**Positive checks:** Native-frontier candidates validate instruction bytes,
sections, relocations, constants, symbols, ABI behavior, dispatch, and exported
digests. CP-03 stops exactly at its versioned Jasmin boundary unless a checked
relation through one exact D-018-admitted Jasmin compiler/toolchain is
separately frozen and accepted to extend its frontier, in which case every
obligation through that relation's promised endpoint closes. CP-04 and CP-05
stop exactly at their declared C11 or LLVM IR boundary unless a checked relation
through one exact D-018-admitted downstream pipeline is separately frozen and
accepted to extend it.

**Mutation and negative checks:** Corrupt an instruction, section, relocation,
constant, symbol, wrapper, ABI field, target feature, compiler output, or link
result; substitute an endpoint digest; or describe an intermediate artifact as
final native evidence.

**Hard acceptance:** Every promised endpoint obligation closes, every
in-frontier corruption or substitution is detected, every beyond-frontier
mutation retains its exact expected `unsupported` outcome, and no Jasmin
compiler, assembler, linker, C compiler, LLVM pass, runtime, loader, or wrapper
seam is hidden from the trust report.

### CC-06 — Fail closed under corruption, substitution, failure, and fallback

**Question:** Does the path preserve exact non-successful outcomes across every
malformed artifact, unavailable authority, operational failure, and fallback?

**Dependencies:** D-005 atomic outcomes, D-009 authority policy, canonical
identity rules, timeout/resource contract, and frozen fallback policy.

**Shared inputs:** Missing, truncated, malformed, oversized, cyclic, corrupted,
wrong-version, wrong-context, wrong-target, stale, timed-out, crashed,
resource-exhausted, unavailable-tool, and unapproved-fallback cases at every
pipeline boundary.

**Candidate outputs:** One exact atomic outcome per affected claim, gate
observations kept separate, bounded diagnostic, attempted authority path,
fallback decision, retained failure record, and invalidation closure.

**Positive checks:** Permitted fallbacks retain or explicitly weaken claims only
as preregistered; failures never reuse prior success or manufacture negative
evidence from absence.

**Mutation and negative checks:** Convert unknown, timeout, missing proof,
checker failure, unsupported format, unavailable tool, or resource exhaustion
to success; silently fall back; suppress adverse evidence; or reuse a result
across identities.

**Hard acceptance:** Every adverse case fails closed in its exact atomic and
gate category, and no fallback increases the claim frontier or assurance.

### CC-07 — Replay exact identities and compare reference semantics

**Question:** Are all compiler results content-addressed, deterministically
replayable, and differentially checked against the same accepted reference
semantics without confusing test agreement with proof?

**Dependencies:** Canonical source/Core/IR/artifact encodings, reference
evaluator, input manifest, cache-key rules, environment contract, and exact
tool/dependency identities.

**Shared inputs:** Positive, negative, boundary, metamorphic, pass-order,
optimization-level, cache, clean-build, line-ending, path, environment, source,
context, target, tool, theorem, certificate, and endpoint substitutions.

**Candidate outputs:** Ordered input/output manifests, cache decisions,
reference and candidate observations, deterministic profile records,
invalidation graph, raw mismatches, and canonical replay identity.

**Positive checks:** Three clean deterministic profiles agree where promised;
every affecting identity change invalidates dependent state; reference
differential results are reported separately from formal evidence.

**Mutation and negative checks:** Omit an identity, normalize a meaningful
difference away, reuse a cache entry across context or target, ignore a
differential mismatch, or make a test result satisfy a proof-required claim.

**Hard acceptance:** Replays are canonical and identity-complete, all seeded
substitutions invalidate the correct closure, and no differential agreement is
misrepresented as a theorem or checked certificate.

### CC-08 — Audit, maintain, and resource the path in solo mode

**Question:** Can one owner inspect, reproduce, update, diagnose, and recover
the candidate within frozen resource and governance limits without claiming
independent review?

**Dependencies:** D-018 admissions, two owner workspaces, offline acquisition
records, host/target matrix, resource and timeout policy, review templates,
correction window, and maintenance scenarios.

**Shared inputs:** Clean bootstrap, offline replay, version update, dependency
replacement, theorem/certificate change, target/profile update, seeded fault,
cache loss, corrupted acquisition, rollback, unsupported-host, and recovery
tasks.

**Candidate outputs:** Raw time/memory/storage/output observations, dependency
and TCB inventory, update/recovery records, diagnostics, unsupported-host
status, owner review, role-overlap disclosure, and maintenance findings.

**Positive checks:** Both workspaces recreate deterministic records within
frozen ceilings; the owner locates every seeded fault, explains the claim
impact, and performs the prescribed update or recovery without hidden network
or state.

**Mutation and negative checks:** Omit a dependency, admission, tool digest,
resource breach, failed bootstrap, unsupported host, role overlap, adverse
finding, or recovery limitation; call same-owner repetition independent.

**Hard acceptance:** All common tasks and seeded faults are completed within
the frozen contract, every limitation remains visible, and review is labeled
only `solo-reviewed` with independent review `unavailable`.

## 4. Comparable metrics

| ID | Metric | Unit and method | Decision use |
| --- | --- | --- | --- |
| M-01 | Required case completion | Passed cases out of 8 | Hard gate: 8/8 per candidate |
| M-02 | Functional semantic preservation | Exact expected functional observations / total applicable observations | Hard gate: 100% within the frozen frontier |
| M-03 | Leakage-policy preservation | Exact expected named-leakage observations / total applicable observations | Hard gate: 100% within the frozen frontier |
| M-04 | Claim-frontier fidelity | Correct supported/unsupported and endpoint classifications / total claims | Hard gate: 100% |
| M-05 | Permitted transformation authority | Accepted transformations with complete permitted theorem, certificate, or declared authority closure / total accepted transformations | Hard gate: 100% |
| M-06 | Final-artifact closure | Required endpoint byte, section, relocation, symbol, ABI, dispatch, and provenance checks completed / total applicable checks | Hard gate: 100% of promised endpoint obligations |
| M-07 | Failure, mutation, and fallback rejection | Expected non-successes / total adverse cases, by category | Hard gate: 100% |
| M-08 | Identity and provenance invalidation | Affecting substitutions that invalidate every dependent result / total | Hard gate: 100% |
| M-09 | Reference differential agreement | Matching exact candidate/reference observations / total, with mismatches retained | Hard gate where agreement is promised; never proof credit |
| M-10 | Deterministic replay | Matching canonical manifests across 3 clean profiles | Hard gate: 3/3 per promised profile |
| M-11 | Dependency and logical-TCB closure | Exact admitted and correctly classified components / total components | Hard gate: 100% before execution |
| M-12 | Diagnostic conformance | Cases with exact ID, category, bounded message, and location / total | Hard gate: 100% |
| M-13 | Supported/unsupported classification | Correct candidate-frontier classifications / total boundary cases | Hard gate: 100% |
| M-14 | Clean bootstrap time | Wall/CPU seconds, peak memory, temporary bytes, and network observations for 5 cold runs | Comparative plus frozen ceilings |
| M-15 | Transformation and evidence replay time | Thirty paired/interleaved runs after one warmup | Comparative; raw data and uncertainty retained |
| M-16 | Output and artifact size | Exact bytes by source, IR, proof, certificate, object, debug, and evidence role | Comparative; no smaller-is-sounder score |
| M-17 | Peak resource behavior | Wall/CPU, resident memory, output, temporary storage, and process counts under one pinned observer | Comparative plus frozen ceilings |
| M-18 | Solo audit and maintenance | Completed common tasks and detected seeded faults | Hard gate: all tasks and faults |
| M-19 | Independent-review status | Exact compiler, proof, leakage, target, and comparative-decision review status | Disclosure only: `unavailable` |

There is no weighted aggregate score. Speed, size, familiarity,
interoperability, or broader unsupported output cannot compensate for semantic
error, leakage divergence, hidden authority, endpoint gaps, failed adverse
cases, or incomplete identity. M-19 is a mandatory disclosure, not a technical
selection score.

## 5. Comparative axes

| ID | Axis | Principal evidence | Candidate-neutral interpretation |
| --- | --- | --- | --- |
| AX-01 | Functional preservation | M-02, M-05, and CC-02/CC-03 | Compare only exact propositions within each frozen frontier; disclose frontier differences |
| AX-02 | Leakage preservation | M-03 and CC-04 | Compare the same named model and target assumptions; no functional-equals-leakage shortcut |
| AX-03 | Final-artifact closure | M-06 and CC-05 | Compare endpoint completeness while retaining native versus interoperability distinctions |
| AX-04 | Claim-frontier coverage | M-04 and M-13 | Broader honestly closed claims may matter; unsupported claims remain nonclaims |
| AX-05 | Logical TCB and dependency surface | M-05 and M-11 | Count and classify exact trust roles; smaller is not automatically sounder |
| AX-06 | Fail-closed robustness | M-07, M-12, and CC-06 | Compare exact adverse behavior without aggregating failures away |
| AX-07 | Reproducibility and provenance | M-08 through M-10 and CC-07 | Compare deterministic replay and invalidation; same-owner evidence stays level 2 |
| AX-08 | Performance and resource cost | M-14 through M-17 | Apply preregistered materiality bands only after all hard gates |
| AX-09 | Solo auditability, maintenance, and migration | M-18, M-19, and CC-08 | Compare common tasks, update/recovery burden, and honest review limitations |

Every axis is decision-relevant. Materiality bands are frozen independently for
each axis before execution and never weaken a hard gate.

## 6. Hard gates and anti-gaming rules

A candidate is eligible only when all eight gates pass:

1. All 40 matrix runs are complete and all five candidates received identical
   frozen inputs, observation rules, resources, and correction opportunity.
2. Functional preservation and structural-lowering observations conform
   exactly for every proposition within the frozen candidate frontier.
3. Named leakage and claim-frontier classifications conform exactly; expected
   claim-level `unsupported` outcomes remain honest, and no native candidate
   shrinks its promised frontier after execution.
4. Every accepted transformation follows only the candidate's declared,
   complete, identity-bound theorem, certificate, or direct-authority path, and
   the complete logical and operational TCB is disclosed.
5. Every malformed, corrupt, substituted, missing, failed, exhausted, stale,
   unavailable, or fallback case remains non-successful in its exact category.
6. Every promised endpoint closes through the applicable final bytes, sections,
   relocations, constants, symbols, ABI, dispatch, wrapper, toolchain, and
   provenance obligations; no last-mile seam is hidden.
7. Offline deterministic replay, identity invalidation, and reference
   differential checks conform in two separately provisioned owner workspaces,
   capped at same-owner reproducibility level 2.
8. CR-01 through CR-11 are complete, `solo-reviewed`, exact-revision records;
   all required D-018 admissions and resource contracts are complete, while
   independent review remains `unavailable` rather than manufactured.

A weaker declared interoperability endpoint is not hidden native assurance.
Likewise, a direct-native label is not evidence that the last mile is closed, a
mechanized pass count is not proof that its statements compose, and a checked
translation certificate is not an unbounded theorem. Candidate-specific
hardware, caches, timeouts, help, or dependencies are variances, never silent
advantages.

### Frozen decision vocabulary

Each candidate receives exactly one of four states for each hard gate. Retain
all underlying observations, then apply the first matching rule in the fixed
precedence `unsupported`, `fail`, `unresolved`, `pass`:

- `pass`: complete evidence satisfies every applicable requirement, including
  any explicitly permitted non-applicability condition stated by the gate;
- `fail`: no structural `unsupported` condition exists and completed evidence
  definitively contradicts at least one gate requirement;
- `unresolved`: neither `unsupported` nor `fail` is established, but absent,
  incomplete, conflicting, or indeterminate evidence, an operational failure,
  `unknown`, timeout, resource exhaustion, or transiently unavailable required
  authority prevents establishing `pass`; and
- `unsupported`: a complete candidate declaration and support inventory proves
  that its authority boundary or supported envelope structurally offers no way
  to supply a capability the gate requires.

Only `pass` satisfies a hard gate. `fail`, `unresolved`, and `unsupported` are
non-passing states and keep that candidate ineligible under the frozen epoch.
They are never silently converted into one another. Permanent structural
absence is `unsupported`; temporary component or evidence unavailability is
`unresolved`. The fixed precedence makes a demonstrated structural inability
`unsupported` even if other evidence also fails or is missing, and makes a
definite non-structural violation `fail` even if other evidence is unresolved.
This gate vocabulary does not replace the four atomic claim outcomes. In
particular, an expected atomic `unsupported` claim beyond a frozen frontier
does not by itself make a protocol hard gate `unsupported`.

Every comparative axis receives exactly one candidate-neutral label after raw
observations and uncertainty are published. Consider only candidates that pass
all eight hard gates, then apply this total rule:

- `hybrid_direct_native_better`, `proof_per_pass_direct_native_better`,
  `jasmin_backend_better`, `portable_c11_better`, or `llvm_ir_better` means at
  least two candidates are eligible and the named candidate alone has an
  advantage beyond the axis's preregistered materiality band over every other
  eligible candidate;
- `practically_equivalent` means at least two candidates are eligible and every
  pair of eligible candidates remains within that axis's preregistered
  materiality band; and
- `inconclusive` applies in every other state: fewer than two eligible
  candidates, incomplete or indeterminate evidence, overlapping uncertainty,
  a tied leading tier above another candidate, a mixed or intransitive ordering,
  or any comparison the frozen materiality rule does not uniquely classify.

Materiality bands define practical equivalence before execution. They never
turn uncertainty into an advantage, combine axes into a score, upgrade an
atomic outcome, or weaken a hard gate.

After all 40 runs and every gate state and axis label are complete, apply these
conclusion rules in order:

1. With no eligible candidate, conclude `inconclusive`.
2. With exactly one eligible candidate, conclude its corresponding
   `recommend_hybrid_direct_native`, `recommend_proof_per_pass_direct_native`,
   `recommend_jasmin_backend`, `recommend_portable_c11`, or
   `recommend_llvm_ir` value.
3. With at least two eligible candidates, recommend one only if every
   decision-relevant axis is either `practically_equivalent` or the label
   naming that same candidate, and at least one axis names that candidate.
4. Otherwise, conclude `tie` only if at least two candidates are eligible and
   every decision-relevant axis is `practically_equivalent` across the complete
   eligible set.
5. All remaining states conclude `inconclusive`, including split advantages,
   a tied leading subset, or any `inconclusive` decision-relevant axis.

These first-match rules are total and mutually exclusive. A recommendation
always names a candidate that passes every hard gate; it never compensates for
a failed, unresolved, or unsupported gate. `tie` and `inconclusive` select no
compiler strategy.

## 7. Evidence packet and archive layout

The eventual packet should use this logical layout; this draft creates none of
these execution directories:

```text
d010-v0.1/
  epochs/0001/
    protocol/
    shared-inputs/
    candidates/hybrid-direct-native/
    candidates/proof-per-pass-direct-native/
    candidates/jasmin-backend/
    candidates/portable-c11/
    candidates/llvm-ir/
    cross-candidate/
    same-owner-replays/
    owner-reviews/
    decision/
```

Every future record binds the suite version, repository revision, candidate and
case, input manifest, exact tools and dependencies, argv, environment, resource
and network policy, ordered input/output manifests, pipeline graph, semantic
endpoints, pass authorities, target/leakage/ABI/object models, claim frontier,
atomic outcomes, gate observations, diagnostics, cache effects, trust closure,
raw metrics, failures, corrections, owner role overlap, and nonclaims.
Original, failed, and superseded records remain content-addressed and
immutable.

Existing Gate 0 schemas cannot silently serve as the D-010 result format.
Before execution, a versioned result/replay schema must represent every field
above and ship positive, negative, resource, and migration cases.

## 8. Owner review scopes

| ID | Accountable owner scope | Required record |
| --- | --- | --- |
| CR-01 | Suite custody and candidate parity | Frozen packet, epoch, order, correction window, materiality bands, and proof every shared change reached all candidates |
| CR-02 | Hybrid direct-native candidate | Complete CP-01 graph, authorities, results, frontier, TCB, and limitations |
| CR-03 | Proof-per-pass direct-native candidate | Complete CP-02 graph, theorem inventory, results, frontier, TCB, and limitations |
| CR-04 | Jasmin backend candidate | Complete CP-03 translation boundary, exact Jasmin/toolchain identities, results, frontier, TCB, and limitations |
| CR-05 | Portable C11 candidate | Complete CP-04 C11 contract, downstream nonclaims, results, frontier, TCB, and limitations |
| CR-06 | LLVM IR candidate | Complete CP-05 IR/version/data-layout contract, downstream nonclaims, results, frontier, TCB, and limitations |
| CR-07 | Functional and structural-lowering semantics | CC-01 through CC-03 semantic endpoints, pass relations, mutations, and adverse evidence |
| CR-08 | Optimization, leakage, and claim frontier | Optimization authority, named leakage results, support classifications, and frontier-change audit |
| CR-09 | Endpoint, object, fallback, and trust closure | Last-mile checks, toolchain/link boundaries, corruption/fallback cases, assumptions, and TCB closure |
| CR-10 | Replay, dependencies, resources, and isolation | D-018 records, offline workspaces, identities, invalidation, raw resources, diagnostics, and unsupported hosts |
| CR-11 | Comparative disposition | Eight gates, nine axes, trade-offs, adverse evidence, M-19 status, nonclaims, and proposed OEP action |

Each record identifies exact bytes and revision, methods, findings, tools and
assistance, prior familiarity, role overlap, unresolved risk, date, and
disposition. Its review label is `solo-reviewed`.

## 9. Decision procedure

1. Confirm Accepted D-003, D-004, D-005, D-006, and D-009 records. Treat D-007,
   D-011, D-012, and D-013 as downstream and use only the frozen suite models
   needed to avoid predetermining them.
2. Record every required D-018 admission, then freeze the suite, inputs,
   pipeline graphs, claim frontiers, resources, host/target/cache/network
   policy, candidate order, correction window, materiality bands, and owner
   scopes before measured work.
3. Execute all candidates symmetrically, preserve failed runs, perform three
   deterministic profiles, five cold bootstraps, thirty timed replays after one
   warmup, and recreate deterministic records in two owner workspaces.
4. Complete CR-01 through CR-11 and all hard gates before examining comparative
   convenience or performance. A failed candidate does not prove another one
   acceptable, and an expected claim-level `unsupported` result is evaluated
   against the frozen frontier rather than hidden or upgraded.
5. Assign the frozen label to AX-01 through AX-09 and publish per-axis
   trade-offs without a weighted score. Apply the total conclusion procedure;
   the result is exactly one of the five `recommend_*` values, `tie`, or
   `inconclusive`.
6. `tie` and `inconclusive` leave D-010 open. A recommendation advances an OEP
   but selects or implements nothing by itself.
7. Acceptance requires one deterministic `recommend_*` conclusion naming a
   candidate whose eight hard gates all `pass`, followed by an Accepted Orange
   Enhancement Proposal. Its `decision-revision` is exactly 40 lowercase
   hexadecimal characters naming the fully validated revision, its review
   authority is `Orange Project Owner`, and its approval record contains the
   literal `solo-reviewed`. The OEP preserves adverse evidence, exact claim
   frontier, trust closure, nonclaims, and reopen triggers and claims no
   independent review.

An Accepted D-010 policy still does not implement a compiler, IR, pass,
certificate checker, backend, object validator, target, leakage model, ABI, or
product claim. D-011, D-012, and D-013 must still ratify the exact native
envelope before corresponding target claims can stabilize.

## 10. Structural completion criteria and current nonclaims

This suite remains structurally complete only while candidates are exactly
CP-01 through CP-05, cases are exactly CC-01 through CC-08, the matrix is 40
runs, metrics are M-01 through M-19, axes are AX-01 through AX-09, hard gates
are 1 through 8, owner scopes are CR-01 through CR-11, and every input and
result is symmetric and content-addressed.

The draft makes these nonclaims:

- no D-003, D-004, D-005, D-006, D-007, D-009, D-010, D-011, D-012, or D-013
  acceptance is inferred;
- no compiler, backend, proof assistant, solver, checker, assembler, linker,
  adapter, runner, observer, emulator, or isolation dependency is admitted,
  acquired, installed, or executed;
- no executable shared input, candidate mapping, result schema, or physical
  execution order exists;
- no evidence epoch, candidate result, selection, or conclusion exists;
- no source-to-IR, pass, certificate, theorem, leakage, target, ABI, object,
  wrapper, interoperability, or final-byte proposition is validated;
- no compiler strategy or output path is selected, preferred, recommended by
  these inputs, implemented, or authorized for claim-bearing product work;
- no claim frontier, logical TCB, target envelope, leakage model, or foreign
  boundary is accepted or changed;
- no C11 or LLVM artifact inherits a native assurance claim, and neither may
  borrow evidence from the other;
- no independent review, reproduction, audit, certification, or external
  validation is claimed; and
- no roadmap gate, S5 closure, release authority, compiler capability, or
  readiness credit follows.
