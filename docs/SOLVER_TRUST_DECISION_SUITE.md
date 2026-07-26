# D-009 solver-trust decision suite

Status: owner-executable draft under D-023; no solver-trust policy selected

Suite version: `d009-v0.1-draft`

Snapshot: 2026-07-25

## Solo-mode disposition

This suite compares three solver-authority policies without inventing candidate
teams, independent reviewers, or external audit. The Orange Project Owner may
implement and review all candidates under D-023, but every such record is
`solo-reviewed`. A second owner pass, workspace, tool, or implementation remains
same-owner evidence and never becomes independent review or reproduction.

The checked-in D-009 laboratory is input-only. It admits, acquires, installs,
or executes no solver, proof assistant, certificate checker, adapter, runner,
observer, or isolation backend. It freezes no evidence epoch, validates no
proof or counterexample, and records no candidate result. The exact execution
baseline is 0/24 candidate-case runs.

## 1. Decision boundary

This protocol supplies the candidate-neutral comparison required by
[D-009](DECISIONS.md#d-009--solver-trust). It decides which authority boundary
may turn automation output into an Orange claim outcome. It does not select a
solver executable, proof foundation, Proof IR, permanent certificate format,
claim schema, target, leakage model, package format, or release envelope.

D-004 and D-005 must be Accepted before D-009 can be Accepted: the solver
policy needs stable semantic identities and atomic claim outcomes. D-006 and
D-007 are downstream consumers, not D-009 acceptance prerequisites. Requiring
them first would create a cycle because D-006 case DS-04 itself needs the
D-009 policy. A D-009 disposition constrains that later case but supplies none
of its proof-foundation or checker evidence.

No third-party tool may be acquired for execution until the owner records the
applicable D-018 admission with exact source, version, digest, dependency graph,
terms, capabilities, offline availability, recovery plan, and TCB effect.

## 2. Candidate parity and frozen inputs

| ID | Candidate | Required treatment | Current execution evidence |
| --- | --- | --- | --- |
| SP-01 | Checked-artifact portfolio | Solvers search, but claim success requires an accepted certificate or Orange proof term; counterexamples are independently validated | 0/8 cases |
| SP-02 | Kernel-only reconstruction | Solver output may guide reconstruction, but only an Orange proof term accepted by the kernel can satisfy a claim | 0/8 cases |
| SP-03 | Direct trusted-solver authority | An exact admitted solver/version/fragment may decide directly and is explicitly included in the logical TCB | 0/8 cases |

The frozen matrix contains exactly 24 candidate-case runs per evidence epoch:
each of the 3 candidates runs each of the 8 cases. Similar outputs do not merge
candidates. A candidate that converges to another policy records that
convergence and its migration cost rather than silently changing identity.

Every candidate receives the same candidate-neutral packet containing:

- exact obligation, theorem, semantics-edition, target, leakage, and context
  identities;
- the four atomic outcomes `satisfied`, `not_satisfied`, `unresolved`, and
  `unsupported`, without numeric ordering or aggregate upgrade;
- claim-closing and developer-profile observations kept distinct;
- valid, satisfiable, malformed, substituted, unsupported, missing-output,
  timeout, crash, and resource-exhaustion cases;
- suite-only LRAT-family, reflective-procedure, proof-term, SMT-proof, model,
  trust-inventory, cache, and diagnostic records;
- exact resource ceilings, timeout semantics, network and cache policy, and an
  evaluation-host matrix frozen before measured work;
- an input-manifest digest covering every byte, mode, path, and case ID; and
- the metrics and decision rules in this document.

The suite-only records are research inputs, not Orange Proof IR or product
claims. Candidate adapters may implement their declared authority policy but
may not change an obligation, weaken a negative case, relabel a trust role, use
an unrecorded oracle, or treat a developer observation as claim-closing.

The atomic outcomes inherit D-005 exactly. `satisfied` requires complete
permitted mandatory closure for the exact proposition; `not_satisfied` requires
permitted identity-bound negative evidence establishing that proposition false
or violated in scope; absence or incompleteness alone is not `not_satisfied`.
`unresolved` applies when a well-formed claim within the candidate's support
still has an unknown, incomplete, conflicting, or exhausted required decision;
and `unsupported` applies only when the candidate offers no permitted
evaluation or authority path for the exact claim and scope. A malformed,
failed, or unsupported supplied artifact, format, rule, or step is `unresolved`
when another permitted claim-authority path still exists. No candidate may
redefine these meanings.

One evidence epoch freezes all inputs, expected observations, authority rules,
resource policy, candidate order, correction window, and materiality bands
before execution. A shared ambiguity or rule change creates a new symmetric
epoch. Candidate corrections retain the original failed record.

Two separately provisioned owner workspaces use distinct checkouts, caches,
output roots, and acquisition records. Matching deterministic results may reach
same-owner reproducibility level 2 only. Neither workspace populates an
independent-reproduction field.

## 3. Required decision cases

### TC-01 — Preserve outcomes across developer and claim-closing profiles

**Question:** Does the candidate preserve the exact four-outcome algebra while
keeping solver-only developer observations from becoming claim authority?

**Dependencies:** The accepted D-005 outcome and basis policy, exact obligation
identity, and frozen profile projection rules.

**Shared inputs:** Valid checked evidence, a valid counterexample, `unknown`,
timeout, unsupported fragment, missing proof output, malformed output, and one
developer-only favorable solver observation.

**Candidate outputs:** One atomic outcome, basis and trust closure, profile
projection, and exact diagnostic for every input.

**Positive checks:** Every claim-closing success names its permitted authority;
developer views retain favorable observations without satisfying a claim.

**Mutation and negative checks:** Relabel developer output, missing evidence,
an assumption, or an unsupported result as `satisfied`; map incompleteness to
`not_satisfied`; or hide an adverse basis.

**Hard acceptance:** Outcomes and profiles agree with the frozen policy for
100% of inputs, and no projection upgrades an atomic non-success.

### TC-02 — Replay an LRAT-family checked-artifact result

**Question:** Can a Boolean-search result close only the exact bit-vector
obligation through its candidate-declared authority path?

**Dependencies:** Suite-only word semantics, canonical bit-blast/CNF rules,
obligation identity, certificate grammar, and bounded checker relation.

**Shared inputs:** One unsatisfiable equivalence obligation, canonical CNF and
LRAT-family certificate records, plus truncated, reordered, forged-step,
wrong-CNF, wrong-obligation, oversized, and missing-certificate variants.

**Candidate outputs:** The declared proof, certificate, or direct-solver basis;
bound obligation/CNF/evidence identities; trust inventory; and diagnostics.

**Positive checks:** The exact positive closes only through the authority
declared by that candidate, and any checked-artifact replay does not execute the
search solver.

**Mutation and negative checks:** Corrupt or substitute every binding, omit the
evidence, introduce an unsupported step, or make the CNF satisfiable.

**Hard acceptance:** Every accepted result is mutually identity-bound and all
adversarial variants remain non-successful in their exact categories.

### TC-03 — Validate satisfiable counterexamples

**Question:** Can a solver-produced model establish `not_satisfied` only after
the exact witness is decoded and evaluated against the exact obligation?

**Dependencies:** Canonical value decoding, obligation semantics, and the
candidate's declared negative-evidence authority.

**Shared inputs:** One valid model and wrong-width, partial, noncanonical,
out-of-domain, substituted-obligation, and falsely favorable models.

**Candidate outputs:** Canonical witness, independent evaluation record,
obligation/model digests, atomic outcome, and exact failure reason.

**Positive checks:** The valid witness deterministically falsifies the bound
obligation without trusting a solver's textual `sat` label.

**Mutation and negative checks:** Change a value, width, subject, context,
obligation, or model digest; omit independent evaluation; or claim that an
invalid model proves the proposition.

**Hard acceptance:** Only a complete, valid, identity-bound witness may produce
`not_satisfied`; all other cases are `unresolved` or `unsupported` as frozen.

### TC-04 — Bound reflective algebra and range procedures

**Question:** Does the policy distinguish kernel-checked reflection from
external simplification and reject reasoning outside the supported fragment?

**Dependencies:** Suite-only ring, modular, range, word, and proof-term rules.

**Shared inputs:** Valid normalization/range cases plus overflow confusion,
wrong modulus, omitted side condition, forged reflection result, unsupported
quantifier, and resource-bound cases.

**Candidate outputs:** Declared authority path, normalized proposition,
checked reconstruction or direct-authority record, trust closure, and result.

**Positive checks:** Supported cases replay through the candidate's declared
boundary and bind every side condition and numeric domain.

**Mutation and negative checks:** Substitute a modulus or bound, remove a side
condition, admit host arithmetic, or relabel an unsupported fragment.

**Hard acceptance:** Supported results are exact and replayable; every
out-of-fragment or malformed case remains non-successful.

### TC-05 — Constrain SMT proof formats and external evidence

**Question:** Can the candidate admit only a ratified SMT fragment and retain
the external status of EasyCrypt/SSProve-style evidence?

**Dependencies:** A suite-only Alethe-like supported fragment, proof-term
mapping, external-evidence vocabulary, and bounded parser rules.

**Shared inputs:** One valid EUF/linear-arithmetic proof record; unknown rule,
trusted step, proof hole, unsupported theory, malformed scope/type, oversized
term, and external proof records.

**Candidate outputs:** Fragment/version identity, declared authority, proof or
external-evidence status, trust closure, outcome, and diagnostics.

**Positive checks:** Supported proof records use only the candidate's permitted
authority. External evidence stays external unless separately reconstructed.

**Mutation and negative checks:** Add an oracle step, unknown rule, proof hole,
unsupported theory, identity substitution, or relabel external evidence as a
kernel proof or checked certificate.

**Hard acceptance:** The supported fragment is closed and versioned; every
trusted, unknown, malformed, or external-only path is represented honestly.

### TC-06 — Fail closed for incomplete or failed automation

**Question:** Do all operational and semantic failure modes retain precise
non-success outcomes without stale-success reuse?

**Dependencies:** Frozen timeout, resource, process, output, and diagnostic
taxonomy plus claim/cache policy.

**Shared inputs:** `unknown`, timeout, CPU/memory/output exhaustion, crash,
signal, missing proof, truncated output, malformed certificate, checker failure,
unsupported step, unavailable authority, and poisoned-cache cases.

**Candidate outputs:** Exact outcome and diagnostic, termination/resource
record, cache disposition, and retained raw observation identity.

**Positive checks:** Each failure remains attributable and no valid prior result
is silently reused for a different attempt or identity.

**Mutation and negative checks:** Map any failure to success, omit its reason,
reuse a stale success, relabel as `unresolved` an exact claim and scope for which
the candidate offers no permitted evaluation or authority path, or infer
falsity from absence of proof.

**Hard acceptance:** Every preregistered failure maps exactly and no failure,
absence, or resource event satisfies or disproves a claim by itself.

### TC-07 — Close identities, caches, and the logical TCB

**Question:** Are proof-result and search-cache identities complete, and is
every direct or indirect authority visible in the TCB?

**Dependencies:** Canonical obligation/import/semantics/checker identities,
target and leakage context, candidate tool identity, and trust inventory.

**Shared inputs:** Valid result/search cache entries and mutations to obligation,
imports, semantics edition, checker/procedure, target, leakage policy, solver
digest, argv, seed, resource limits, proof/certificate, and trust role.

**Candidate outputs:** Separate result and search keys, invalidation trace,
complete trust inventory, basis classification, and cache decision.

**Positive checks:** Proof-result keys include every claim-affecting identity;
search keys additionally include solver and execution controls. A direct trusted
solver, if declared, is unmistakably in the logical TCB.

**Mutation and negative checks:** Omit or substitute any key component, place a
certificate digest in the lookup key for the search meant to find it, hide a
solver/runtime, or relabel direct solver authority as checked evidence.

**Hard acceptance:** All substitutions invalidate the dependent result; there
is no undeclared trust and no misleading cache status.

### TC-08 — Replay, inspect, and maintain the policy in solo mode

**Question:** Can the owner reproduce, challenge, and modify each candidate
under matched controls from exact published bytes?

**Dependencies:** TC-01 through TC-07 digests, frozen host/resource/cache rules,
owner review scopes, task order, seed, correction window, and D-018 records.

**Shared inputs:** Two clean owner workspaces, three deterministic replay
profiles, five cold bootstrap records, thirty timed replays after one warmup,
and matched hidden-trust, stale-cache, ambiguous-diagnostic, and dependency
substitution tasks.

**Candidate outputs:** Canonical manifests, raw resource observations,
same-owner replay records, seeded-fault results, repair trace, assistance and
prior-familiarity disclosures, and unresolved risks.

**Positive checks:** Exact published bytes suffice for offline replay and the
same maintenance task; deterministic outputs agree across required profiles.

**Mutation and negative checks:** Use undeclared network/cache state, substitute
a dependency or authority, miss a seeded fault, omit assistance, or rewrite a
failed run after correction.

**Hard acceptance:** Both workspaces reproduce the deterministic records at no
higher than level 2, every seeded fault is retained and classified, and all
owner-role overlap is disclosed. This supplies no independent-review evidence.

## 4. Comparable metrics

| ID | Metric | Unit and method | Decision use |
| --- | --- | --- | --- |
| M-01 | Required case completion | Passed cases out of 8 | Hard gate: 8/8 per candidate |
| M-02 | Atomic outcome conformance | Exact expected outcomes / total | Hard gate: 100% |
| M-03 | Permitted success authority | Successful results with complete permitted basis and trust closure / total successes | Hard gate: 100% |
| M-04 | Counterexample validation | Independently evaluated bound witnesses / negative claims | Hard gate: 100% |
| M-05 | Failure and mutation rejection | Expected non-successes / total, by category | Hard gate: 100% |
| M-06 | Checked-artifact replay | Solver-free accepted replays / applicable accepted artifacts | Hard gate where the candidate declares checked artifacts |
| M-07 | Supported-fragment closure | Correct supported/unsupported classifications / total | Hard gate: 100% |
| M-08 | Identity and cache invalidation | Affecting substitutions that invalidate dependent state / total | Hard gate: 100% |
| M-09 | Undeclared logical trust | Count of omitted solvers, checkers, runtimes, axioms, plugins, and direct authorities | Hard gate: 0 |
| M-10 | Deterministic replay | Matching canonical manifests across 3 clean profiles | Hard gate: 3/3 |
| M-11 | Diagnostic conformance | Cases with exact ID, category, bounded message, and location / total | Hard gate: 100% |
| M-12 | Resource behavior | Raw wall/CPU, peak memory, output, and temporary bytes | Comparative plus frozen ceilings |
| M-13 | Replay time | Thirty paired/interleaved runs after one warmup | Comparative; raw data and uncertainty retained |
| M-14 | Dependency and provenance closure | Identified and admitted components / total | Hard gate: 100% before execution |
| M-15 | Owner audit and maintenance | Completed common TC-08 tasks and detected seeded faults | Hard gate: all tasks and faults |
| M-16 | Independent-review status | Exact logic, checker, and comparative-decision review status | Disclosure only: `unavailable` |

There is no weighted aggregate score. Speed, convenience, or smaller artifacts
cannot compensate for an incorrect outcome, hidden authority, failed negative
case, or incomplete identity. M-16 is a mandatory disclosure, not a technical
selection score.

## 5. Hard gates and anti-gaming rules

A candidate is eligible only when all eight gates pass:

1. All 24 matrix runs are complete and all three candidates received identical
   frozen inputs and correction opportunity.
2. Atomic outcomes, developer projections, and basis classifications conform
   exactly across the complete corpus.
3. Every success and negative result follows only the candidate's declared,
   complete, identity-bound authority path.
4. Every malformed, substituted, missing, failed, exhausted, unknown, trusted-
   step, and unsupported case remains non-successful in its exact category.
5. Every affecting identity and cache mutation invalidates all dependent state;
   no stale success or cross-context result survives.
6. All solvers, checkers, runtimes, proof rules, dependencies, and direct
   authorities are inventoried with exact provenance and trust roles.
7. Offline deterministic replay and resource behavior conform in two separately
   provisioned owner workspaces, capped at reproducibility level 2.
8. SR-01 through SR-08 are complete, `solo-reviewed`, exact-revision records;
   independent review remains `unavailable` rather than manufactured.

A declared direct-solver TCB is not hidden checked evidence, and a smaller TCB
is not proof of soundness. Candidate-specific shortcuts, hardware, caches,
timeouts, help, or dependencies are variances, never silent advantages.

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
This gate vocabulary does not replace the four atomic claim outcomes.

Every comparative axis receives exactly one candidate-neutral label after raw
observations and uncertainty are published. Consider only candidates that pass
all eight hard gates, then apply this total rule:

- `checked_artifact_better`, `kernel_only_better`, or
  `trusted_solver_better` means at least two candidates are eligible and the
  named candidate alone has an advantage beyond the axis's preregistered
  materiality band over every other eligible candidate;
- `practically_equivalent` means at least two candidates are eligible and every
  pair of eligible candidates remains within that axis's preregistered
  materiality band; and
- `inconclusive` applies in every other state: fewer than two eligible
  candidates, incomplete or indeterminate evidence, overlapping uncertainty,
  a tied leading tier above another candidate, a mixed or intransitive ordering,
  or any comparison the frozen materiality rule does not uniquely classify.

Materiality bands define practical equivalence per comparative metric before
execution; they never weaken a hard gate, turn uncertainty into an advantage,
or combine axes into a score. After all 24 runs and every gate state and
decision-relevant axis label are complete, apply the following conclusion rules
in order:

1. With no eligible candidate, conclude `inconclusive`.
2. With exactly one eligible candidate, conclude its corresponding
   `recommend_checked_artifact`, `recommend_kernel_only`, or
   `recommend_trusted_solver` value.
3. With at least two eligible candidates, recommend one only if every
   decision-relevant axis is either `practically_equivalent` or the label naming
   that same candidate, and at least one axis names that candidate.
4. Otherwise, conclude `tie` only if at least two candidates are eligible and
   every decision-relevant axis is `practically_equivalent` across the complete
   eligible set.
5. All remaining states conclude `inconclusive`, including split advantages,
   a tied leading subset, or any `inconclusive` decision-relevant axis.

These first-match rules are total and mutually exclusive. A recommendation
always names a candidate that passes every hard gate; it never compensates for
a failed, unresolved, or unsupported gate. `tie` and `inconclusive` select no
policy.

## 6. Evidence packet and archive layout

The eventual packet should use this logical layout; this draft creates none of
these execution directories:

```text
d009-v0.1/
  epochs/0001/
    protocol/
    shared-inputs/
    candidates/checked-artifact/
    candidates/kernel-only/
    candidates/trusted-solver/
    cross-candidate/
    same-owner-replays/
    owner-reviews/
    decision/
```

Every future record binds the suite version, repository revision, candidate and
case, input manifest, exact tools and dependencies, argv, environment, resource
and network policy, ordered input/output manifests, obligation and evidence
identities, atomic outcome, diagnostics, cache effects, trust closure, raw
metrics, failures, corrections, owner role overlap, and nonclaims. Original,
failed, and superseded records remain content-addressed and immutable.

The provisional Gate 0 schemas cannot silently serve as the D-009 result
format. Before execution, a versioned result/replay schema must represent every
field above and ship positive, negative, and migration cases.

## 7. Owner review scopes

| ID | Accountable owner scope | Required record |
| --- | --- | --- |
| SR-01 | Suite custody and parity | Frozen packet, epoch, order, correction window, and proof every shared change reached all candidates |
| SR-02 | Checked-artifact candidate | Complete SP-01 mapping, results, trust boundary, and limitations |
| SR-03 | Kernel-only candidate | Complete SP-02 mapping, results, trust boundary, and limitations |
| SR-04 | Trusted-solver candidate | Complete SP-03 mapping, direct TCB inventory, results, and limitations |
| SR-05 | Outcome and authority semantics | Claim/profile decisions, negative authority, supported fragments, and adverse cases |
| SR-06 | Identity, cache, and trust closure | Obligation/evidence bindings, invalidation, TCB, axioms, runtimes, and external status |
| SR-07 | Replay, dependency, and isolation | D-018 records, offline workspaces, resources, diagnostics, and unsupported hosts |
| SR-08 | Comparative disposition | Hard gates, trade-offs, adverse evidence, M-16 status, nonclaims, and proposed OEP action |

Each record identifies exact bytes and revision, methods, findings, tools and
assistance, prior familiarity, role overlap, unresolved risk, date, and
disposition. Its review label is `solo-reviewed`.

## 8. Decision procedure

1. Confirm Accepted D-004 and D-005 records. D-006 and D-007 remain downstream
   and cannot be treated as prerequisites that create a dependency cycle.
2. Record every required D-018 admission, then freeze the suite, inputs,
   resources, host/cache/network policy, candidate order, correction window,
   materiality bands, and owner scopes before measured work.
3. Execute all candidates symmetrically, preserve failed runs, perform three
   deterministic profiles, five cold bootstraps, thirty timed replays after one
   warmup, and recreate deterministic records in two owner workspaces.
4. Complete SR-01 through SR-08 and all hard gates before examining comparative
   convenience or performance. A failed candidate does not prove another one
   acceptable.
5. Assign the frozen comparative label to each axis and publish per-axis
   trade-offs without a weighted score. Apply the frozen decision vocabulary;
   the suite conclusion is exactly `recommend_checked_artifact`,
   `recommend_kernel_only`, `recommend_trusted_solver`, `tie`, or
   `inconclusive`.
6. `tie` and `inconclusive` leave D-009 open. A recommendation advances an OEP
   but selects nothing by itself.
7. Acceptance first requires one deterministic `recommend_*` conclusion naming
   a candidate whose eight hard gates all `pass`; `tie` and `inconclusive` leave
   D-009 open. It then requires an Accepted Orange Enhancement Proposal whose
   `decision-revision` is exactly 40 lowercase hexadecimal characters naming
   the fully validated revision. It names `Orange Project Owner` as the review
   authority, includes an approval record containing literal `solo-reviewed`,
   preserves adverse evidence and reopen triggers, and claims no independent
   review.

An Accepted D-009 policy still does not implement D-006, D-007, a checker, or a
claim-bearing automation path. Any selected direct solver authority must be
described explicitly as logical TCB; any selected checked-artifact or
kernel-only path must still supply its later proof/checker evidence.

## 9. Structural completion criteria and current nonclaims

This suite remains structurally complete only while candidates are exactly
SP-01 through SP-03, cases are exactly TC-01 through TC-08, the matrix is 24
runs, metrics are M-01 through M-16, hard gates are 1 through 8, owner scopes
are SR-01 through SR-08, and every input and result is symmetric and
content-addressed.

Current execution evidence remains 0/24. The draft makes these nonclaims:

- no D-004, D-005, D-006, D-007, or D-009 acceptance is inferred;
- no solver, proof assistant, certificate checker, adapter, runner, observer,
  or isolation dependency is admitted, acquired, installed, or executed;
- no executable shared input, candidate mapping, or result schema exists;
- no evidence epoch, physical execution order, candidate result, selection, or
  conclusion exists;
- no proof, certificate, counterexample, theorem, claim, or cache result is
  validated;
- no solver is added to or removed from the logical TCB;
- no proof-bearing product implementation or solver-backed search is
  authorized;
- no independent review, reproduction, audit, or external validation is
  claimed; and
- no roadmap gate, S4 closure, release authority, or readiness credit follows.
