# D-005 public-assurance-model decision suite

Status: owner-executable draft under D-023; no public assurance model selected

Suite version: `d005-v0.1-draft`

Snapshot: 2026-07-25

## Solo-mode disposition

This suite turns D-005 into a comparison that the Orange Project Owner can
execute without external implementers, auditors, cryptographers, laboratories,
or downstream integrators. The owner builds and challenges every candidate
under one frozen packet, repeats deterministic work in separately provisioned
workspaces, and records role overlap, prior familiarity, assistance, generated
suggestions, adverse evidence, and unresolved risk.

Those controls provide same-owner repeatability and maintenance evidence only.
They do not create independent review, external validation, certification, or a
population-level usability result. Independent review is `unavailable` while
D-023 remains the operating model. Its absence limits claims that require that
evidence, but it neither excuses a technical failure nor blocks an otherwise
conforming owner decision about the claim architecture.

This draft selects no candidate, ratifies no public schema or namespace,
upgrades no current compiler evidence into a product claim, and authorizes no
release. A second owner run, workspace, implementation, or rendering remains
same-owner evidence and records no reproducibility level above 2.

## 1. Decision boundary and historical inputs

This protocol supplies the symmetric comparison required by
[D-005](DECISIONS.md#d-005--public-assurance-model). It evaluates candidate
semantic architectures for public assurance claims: the unit of claim, the
outcome algebra, evidence authority, identity and trust closure, composition,
invalidation, and public projection. Only the Accepted exact-revision OEP in
section 8 chooses an architecture. This draft installs, adopts, or admits no
tool or dependency and grants no dependency-acquisition authority. It does not
choose D-003 product form, D-004 semantic strata, D-006 proof foundation, D-007
proof encoding, D-009 solver policy, D-012 leakage model, D-014 package format,
or a release envelope.

The proposed model in [`ASSURANCE.md`](ASSURANCE.md), the explanatory material
in [`THE_ORANGE_BOOK.md`](THE_ORANGE_BOOK.md), and the Gate 0 schemas and
fixtures are frozen research inputs. In particular,
`schemas/gate0/claim-record-v0.1.schema.json` is historical, provisional,
non-product shape material. It lacks a first-class compiler-preservation family,
claim-kind-to-basis policy, mandatory-basis completeness, evidence-reference
closure, subject-byte verification, theorem/proof binding, trust-closure links,
composition edges, and lifecycle invalidation. Passing that schema or its
current second-pass checks is not evidence that any D-005 candidate passes.

The frozen negative corpus must demonstrate, rather than merely assert, that a
shape-only record can otherwise admit each of these dangerous mutations:

- a checked test run presented as sufficient functional-refinement evidence;
- a checked test run masking a failed mandatory kernel proof;
- target leakage reported `satisfied` while target and leakage contexts remain
  unresolved;
- an owner-produced checked test presented as external validation; and
- a substituted subject path or digest reusing evidence bound to other bytes.

Every candidate receives those mutations. A future replacement schema may
reuse useful v0.1 fields, but it must receive a new identity and migration rule;
this decision cannot silently promote the historical URN into a public format.

## 2. Candidate parity and frozen invariants

| ID | Candidate | Required treatment | Current execution evidence |
| --- | --- | --- | --- |
| AM-01 | Orthogonal typed claim records | Run the complete suite with one canonical record per atomic proposition and explicit dependency edges | 0/8 cases |
| AM-02 | Named assurance profiles backed by atomic claims | Run the complete suite with profiles as deterministic projections over inspectable atomic claims | 0/8 cases |
| AM-03 | Evidence graph with derived claim views | Run the complete suite with claims derived from a canonical typed evidence and trust graph | 0/8 cases |
| AM-04 | Aggregate package assurance level or verified Boolean | Run the complete suite with its aggregate public result and complete drill-down semantics exposed to every mutation | 0/8 cases |

The frozen matrix contains exactly 32 candidate-case executions per evidence
epoch: each of the 4 candidates runs each of the 8 cases. Similar outputs do not
merge candidates silently. If a candidate can pass only by becoming
observationally equivalent to another candidate, that convergence is a named
result with migration and complexity consequences.

Every candidate represents these ten required claim families exactly and
separately:

| ID | Required claim family | Minimum distinction |
| --- | --- | --- |
| CF-01 | Conformance | Exact standard, edition, profile, clauses, errata, domain, implementation, and evidence authority |
| CF-02 | Functional refinement | Exact implementation-to-specification relation and proof or certificate policy |
| CF-03 | Safety | Named memory, initialization, arithmetic, panic, trap, and precondition scope |
| CF-04 | Termination | Named input domain, environment, resource qualification, and termination authority |
| CF-05 | Leakage | Separate source and target observations, public relation, declassification, and preservation boundary |
| CF-06 | Compiler preservation | Exact source relation, pass path, toolchain, target, and final artifact; never a generic refinement alias |
| CF-07 | ABI | Exact layout, symbol, ownership, aliasing, length, error, calling, and foreign-code contract |
| CF-08 | Erasure | Exact object, lifecycle point, architectural model, target, and residual exclusions |
| CF-09 | Game-based security | Exact game, construction, parameters, reduction, advantage bound, and assumptions |
| CF-10 | Empirical tests | Exact corpus, method, environment, observation, and explicitly limited inference |

External validation and repository-control observations remain separately typed
claims or evidence classes where applicable. Neither replaces a required family.
An external certificate does not become a theorem; a local theorem does not
become certification; and repository hygiene does not become product
correctness.

Each atomic claim has exactly one outcome: `satisfied`, `not_satisfied`,
`unresolved`, or `unsupported`. These outcomes are categorical, not ordered
grades. Every case also has one separate verdict, exactly `pass` or `fail`.
Matching an expected negative fixture may make the case verdict `pass`, but the
atomic claim remains its expected non-success outcome and gains no capability.

The four outcome meanings are frozen across candidates. `satisfied` means the
exact proposition has its complete permitted mandatory closure and no valid
decisive negative result. `not_satisfied` means permitted, identity-bound
negative evidence establishes that the exact proposition is false or violated
within its scope; absence or incompleteness alone is not `not_satisfied`.
`unresolved` means the claim is well-formed and within the declared support
model, but a required decision remains unknown, incomplete, conflicting, or
exhausted. `unsupported` means the declared policy or support envelope offers
no permitted evaluation or authority path for that exact claim and scope. No
candidate may reinterpret these categories or map any non-success to success.

A frozen claim policy names every permitted and mandatory basis type, context,
trust-closure component, composition edge, freshness rule, and review trigger
for each claim family and scope. `satisfied` requires all mandatory elements to
be present, valid, unexpired, unrevoked, mutually bound, and permitted. One
checked but optional basis cannot mask a failed, expired, unavailable, missing,
or mismatched mandatory element. An assumption is a dependency, never a basis
that proves itself.

Profiles, levels, summaries, queries, neighboring implementations, and package
rollups are read-only projections. They may preserve an atomic outcome or report
that no aggregate result exists. They may never upgrade, average, inherit, or
hide an atomic non-success. A public label is invalid if a reader cannot recover
the exact atomic claims, subjects, contexts, evidence, assumptions, exclusions,
trust closure, and invalidation state that produced it.

One evidence epoch freezes all inputs, expected outcomes, authority policies,
resource ceilings, output schemas, rendering rules, candidate order, correction
window, and decision rules before measured implementation begins. A shared
ambiguity or rule change creates a new epoch and reruns all four candidates. A
candidate defect retains its failed run and consumes the common correction
window. No candidate-specific repair may be applied after comparative results
are visible without an equivalent opportunity for every candidate.

Throughout this suite, separately provisioned owner workspaces are distinct
clean checkouts of the same exact revision with no shared mutable cache, output
directory, generated state, or dependency store. Each has a separate
dependency-acquisition and environment record. Only content-addressed,
read-only frozen inputs named by both manifests may be shared. Every replay
records those isolation facts and fails closed when shared or undeclared state
cannot be excluded.

## 3. Required decision cases

### AC-01 — Represent every claim family with mixed outcomes

**Question:** Can the candidate represent all ten required families over one
package without compressing their distinct subjects, authorities, or outcomes?

**Dependencies:** The frozen CF-01 through CF-10 vocabulary, exact synthetic
package/implementation/target identities, four-outcome algebra, and display
contract.

**Shared inputs:** One package with multiple implementations and targets; one
atomic claim in every required family; at least two `satisfied`, two
`not_satisfied`, two `unresolved`, and two `unsupported` outcomes; distinct
assumptions and exclusions; and separately scoped external-validation and
repository-control records.

**Candidate outputs:** Canonical atomic identities and records or derivations,
complete family coverage, exact outcomes, dependency edges, public machine and
human views, and a lossless path from every displayed statement to its closure.

**Positive checks:** All ten families remain separately queryable and retain
their exact subject, context, wording, outcome, bases, assumptions, exclusions,
and trust effects across three deterministic renders.

**Mutation and negative checks:** Remove CF-06, merge source and target leakage,
rename game security as empirical confidence, inherit one target result across
another target, replace mixed outcomes with a single color, and omit an
unsupported claim from the public view.

**Hard acceptance:** Coverage is 10/10; every atomic field and outcome matches;
no family, variant, or non-success disappears; and no aggregate output implies
that the package is generally verified.

### AC-02 — Enforce evidence authority and mandatory-basis completeness

**Question:** Does `satisfied` mean that the complete family-specific evidence
policy passed, rather than that some favorable evidence exists?

**Dependencies:** A frozen kind-to-basis authority matrix, mandatory and
optional basis sets, verification-state semantics, and evidence identities.

**Shared inputs:** Valid multi-basis claims plus assumption-only, test-only
refinement, audit-only leakage, owner-test external-validation, missing
certificate, wrong theorem, failed proof, expired audit, unavailable laboratory,
unknown solver, and one checked optional test beside each failed mandatory basis.

**Candidate outputs:** Per-basis decisions, complete mandatory-set evaluation,
atomic outcome with exact reason, retained adverse evidence, and a trace from
policy rule to every accepted or rejected authority.

**Positive checks:** Only complete, permitted, valid, current, identity-bound
basis sets can yield `satisfied`; optional defense-in-depth evidence remains
visible without changing its authority.

**Mutation and negative checks:** Change basis type while retaining bytes, mark
`recorded` as `checked`, let one checked test mask a failed proof, use an
assumption as its own evidence, omit a mandatory basis, and relabel owner review
as a technical proof or external validation.

**Hard acceptance:** Every authority substitution and incomplete mandatory set
is non-successful in the expected category. No quantity of tests, audits,
reviews, or assumptions silently satisfies a proof-, certificate-, laboratory-,
or external-authority requirement.

### AC-03 — Bind subject, context, theorem, proof, and evidence identities

**Question:** Does every successful claim close over the exact proposition and
bytes it purports to describe?

**Dependencies:** Canonical identities for repository revision, path, symbol,
artifact, language edition, toolchain, crypto profile, target, leakage model,
theorem, proof object, evidence object, and claim policy.

**Shared inputs:** One complete valid closure plus single-field substitutions
for every identity dimension, same-name/different-byte artifacts, cross-target
and cross-implementation evidence, unresolved required contexts, path traversal,
digest mismatch, and a theorem/proof pair bound to another proposition.

**Candidate outputs:** Canonical identity graph, verified byte and reference
closure, exact mismatch path and category, and no claim result before required
contexts resolve.

**Positive checks:** Reordering transport or presentation does not change the
canonical identity; replay binds the same exact subject, context, policy,
theorem, proof, evidence, and trust closure.

**Mutation and negative checks:** Substitute each coordinate independently,
reuse evidence by friendly name, swap a target or leakage profile, alter a byte
after hashing, redirect a reference, and attach a valid proof to the wrong
theorem fingerprint.

**Hard acceptance:** All substitutions fail closed; unresolved mandatory
contexts cannot yield `satisfied`; and diagnostics identify the exact mismatched
edge without accepting a neighboring valid record.

### AC-04 — Close assumptions, the TCB, and cross-boundary composition

**Question:** Can the candidate show every trusted dependency and justified
edge needed to carry a claim across semantic, compiler, ABI, and platform
boundaries?

**Dependencies:** Frozen trust inventory, assumption and compromise-effect
vocabulary, boundary graph, allowed edge authorities, and CF-06 compiler-
preservation distinction.

**Shared inputs:** A complete source-to-final-artifact path and variants with a
missing pass edge, unlisted plugin, dangling assumption, wrong model, cyclic
closure, omitted linker/ABI step, compromised component, duplicated trust ID,
and an unrelated valid source theorem.

**Candidate outputs:** Minimal complete trust and assumption closure, ordered or
canonically graph-identified edges, compromise propagation, exact final subject,
and preserved narrow source claims when a later boundary fails.

**Positive checks:** Every required edge has one permitted authority and exact
endpoints; every trusted component and assumption has identity, role, reason,
and compromise effect; final-artifact claims traverse the entire required path.

**Mutation and negative checks:** Remove or substitute each edge, hide a tool as
glue, break an assumption reference, create a cycle, use owner approval as a
technical preservation edge, and attempt to lend a source claim directly to
final bytes.

**Hard acceptance:** Incomplete or inconsistent closure cannot satisfy the
dependent claim. A later failure does not erase a valid narrower source claim,
but it never permits that claim to jump to a different subject.

### AC-05 — Apply expiry, revocation, supersession, and downgrade rules

**Question:** Does claim status change deterministically when evidence, policy,
identity, or trust state changes over time?

**Dependencies:** Frozen logical time, review triggers, validity intervals,
supersession graph, revocation records, compromise policy, and rollback rules.

**Shared inputs:** Current and expired bases, revoked evidence, superseded
claims, a compromised signer/tool, changed target erratum, policy version
upgrade, rollback bundle, stale cache, conflicting successors, and an unaffected
claim on another subject.

**Candidate outputs:** Current atomic outcome, retained historical state,
invalidation reason and scope, successor/predecessor links, deterministic query
at a declared time, and an explicit unaffected set.

**Positive checks:** The same time and input closure produce identical current
and historical views; invalidation propagates exactly through dependent edges
and no farther.

**Mutation and negative checks:** Delete a revocation, prefer cached success,
accept an older policy after downgrade, create multiple current successors,
move the clock implicitly, omit adverse history, and revoke one shared trust
root without invalidating its dependents.

**Hard acceptance:** Expired, revoked, superseded, compromised, conflicting, or
stale mandatory evidence cannot support current `satisfied`; rollback and
downgrade fail closed; history is append-only and attributable.

### AC-06 — Separate external authority from solo-produced evidence

**Question:** Can the model record useful owner evidence without manufacturing
external validation, certification, audit independence, or review separation?

**Dependencies:** D-023, the reproducibility-level definitions, frozen authority
and identity rules, and exact external-record requirements.

**Shared inputs:** Owner test, owner audit pass, second owner workspace, tool-
generated critique, missing external laboratory record, expired external record,
valid synthetic externally signed fixture for parser testing only, and claims
whose policies do and do not require external authority.

**Candidate outputs:** Exact producer and authority classification, `solo-reviewed`
owner records, independent-review status, expected atomic outcomes, external
record metadata where supplied, and explicit nonclaims.

**Positive checks:** Owner work can support only the evidence classes and claim
policies that permit it. A missing external basis leaves the dependent claim
`unsupported` or `unresolved` as frozen, without blocking unrelated claims.

**Mutation and negative checks:** Rename owner review as independent audit,
populate an independent-reproduction field from a second owner run, treat a
signature as proof of scope, turn a synthetic fixture into certification, and
let absent external evidence upgrade or erase a technical failure.

**Hard acceptance:** Every provenance and authority label is truthful; same-
owner replay is capped at level 2; independent review remains `unavailable`;
and no owner, tool, signature, or schema-valid record impersonates an external
institution or technical proof.

### AC-07 — Compose packages, profiles, implementations, and targets

**Question:** Can public summaries remain useful while preserving every atomic
variant and refusing unsafe inheritance or aggregation?

**Dependencies:** Exact package graph, implementation and target variants,
profile membership rules, claim-dependency graph, and projection semantics.

**Shared inputs:** Two packages, two implementations, and two targets with
deliberately mixed atomic outcomes; a profile whose mandatory member is
unresolved; optional evidence; one missing variant; one revoked child; and an
unrelated fully satisfied subgraph.

**Candidate outputs:** Atomic matrix, any profile or aggregate projection with
its derivation, missing-member representation, drill-down links, and stable
package/variant identities.

**Positive checks:** Every summary is a deterministic, lossless projection whose
wording and state cannot be read as stronger than its atomic members. A reader
can enumerate all covered and uncovered variants.

**Mutation and negative checks:** Average mixed outcomes, use the best target as
the package result, omit unsupported members, inherit across implementations,
let optional evidence compensate for a mandatory failure, and display a green
profile while a mandatory child is non-successful.

**Hard acceptance:** No aggregate, level, Boolean, profile, or neighboring claim
upgrades an atomic result. If safe aggregation is undefined, the candidate must
report no aggregate result rather than invent an ordering over the four outcomes.

### AC-08 — Render, inspect, replay, and migrate deterministically

**Question:** Can a user recover the complete claim meaning from stable public
artifacts and migrate without reinterpretation?

**Dependencies:** Frozen canonicalization and resource rules, machine schema,
human wording template, offline bundle, version negotiation, v0.1 historical
fixtures, and migration policy.

**Shared inputs:** Complete valid and mixed-status packets, reordered equivalent
inputs, unknown fields/versions, malformed UTF-8 and numbers, duplicate IDs,
oversized and cyclic graphs, missing offline bytes, v0.1 fixtures, and an
ambiguous legacy field with no safe mapping.

**Candidate outputs:** Canonical bytes and digest, deterministic human report,
bounded diagnostics, offline inspection/replay record, explicit migration map,
unmigratable result where needed, and round-trip observations.

**Positive checks:** Three clean serial and declared-parallel renders agree;
machine and human views carry the same atomic meaning; offline inspection uses
only inventoried bytes; safe migrations preserve identities and outcomes.

**Mutation and negative checks:** Reorder graph input, inject unknown data,
truncate a bundle, exploit a path/reference, exceed limits, coerce an unknown
version, silently map missing compiler preservation, and reinterpret a v0.1
`satisfied` record under stronger policy.

**Hard acceptance:** Canonical output and decisions are deterministic and
resource-bounded; malformed, cyclic, oversized, incomplete, unknown, or
ambiguous input fails closed; no historical record is silently promoted or
reinterpreted.

## 4. Comparable metrics

All candidates use the same frozen inputs, host allocation, resource ceilings,
logical time, cache states, run order, and observer. Raw records are retained.
Performance observations use preregistered paired runs and are never treated as
soundness evidence.

| ID | Metric | Unit and method | Decision use |
| --- | --- | --- | --- |
| M-01 | Required case completion | Passed cases out of 8 | Hard gate: 8/8 |
| M-02 | Required family coverage | Exact separately queryable families out of 10 | Hard gate: 10/10 |
| M-03 | Atomic outcome fidelity | Exact outcomes and reasons / total | Hard gate: 100% |
| M-04 | Mutation rejection | Expected fail-closed decisions / total, by category | Hard gate: 100% |
| M-05 | Basis-policy completeness | Claims with every permitted mandatory basis decision / total | Hard gate: 100% |
| M-06 | Identity binding | Verified subject/context/theorem/proof/evidence edges / total | Hard gate: 100% |
| M-07 | Trust and assumption closure | Complete resolved required closure edges / total | Hard gate: 100% |
| M-08 | Composition fidelity | Profile/package/target projections preserving atomic outcomes / total | Hard gate: 100% |
| M-09 | Lifecycle invalidation | Expected expiry/revocation/supersession/downgrade decisions / total | Hard gate: 100% |
| M-10 | Deterministic rendering | Matching machine and human output-manifest digests across 3 runs | Hard gate: 3/3 per claimed profile |
| M-11 | Offline closure | Required bytes resolved and replayed with network denied / total | Hard gate: 100% |
| M-12 | Migration fidelity | Safe exact migrations plus explicit unmigratable cases / total | Hard gate: 100% |
| M-13 | Bounded behavior | Peak bytes, events, depth, output, and time under frozen ceilings | Hard gate: every limit fails closed |
| M-14 | Implementation surface | Source/generated bytes and dependencies by parser, policy, graph, renderer, and glue role | Comparative only; no smaller-is-sounder inference |
| M-15 | Diagnostic conformance | Cases with exact ID, path/edge, category, reason, and bounded output / total | Hard gate: 100% |
| M-16 | Owner audit/maintenance tasks | Completed published-only locate-and-change tasks / 2 per candidate | Hard gate: 2/2 per candidate with seeded faults detected |
| M-17 | Independent-review status | Exact external assurance-model and public-wording review status | Disclosure only: `unavailable`; never a score, gate, or implied claim |
| M-18 | Same-owner maintenance variance | Files, rules, migrations, dependencies, and elapsed owner time for the same seeded change | Comparative maintainability evidence only |

There is no weighted aggregate score. Speed, compactness, familiarity, or a
pleasant summary cannot compensate for a false claim, missing authority,
unbound subject, incomplete trust closure, unsafe composition, stale evidence,
or nondeterministic public meaning. Metrics inform the recorded rationale only
after every hard gate passes. M-17 remains a mandatory limitation disclosure,
not a selection signal.

## 5. Hard gates and anti-gaming rules

A candidate is eligible for recommendation only when all eight gates pass:

1. AC-01 through AC-08 have complete records against the same frozen inputs,
   policies, expected outcomes, mutations, and resource ceilings.
2. CF-01 through CF-10 remain exact, separately queryable atomic families with
   one of the four exact outcomes and no missing implementation/target variant.
3. `satisfied` requires the complete permitted mandatory basis, identity,
   context, trust, composition, freshness, and validity closure; assumptions,
   optional evidence, or favorable neighbors cannot mask a required non-success.
4. Every required authority, substitution, omission, malformed, ambiguity,
   resource, lifecycle, composition, downgrade, and historical-schema mutation
   fails closed in its expected category.
5. Every profile, package view, level, Boolean, and human summary is a lossless
   non-upgrading projection over inspectable atomic claims, or is explicitly
   absent when safe aggregation is undefined.
6. Canonical machine and human outputs are deterministic, bounded, versioned,
   and inspectable offline; migration never silently reinterprets v0.1 or an
   unknown/ambiguous record.
7. Two separately provisioned owner workspaces recreate deterministic manifests
   from the published packet at reproducibility level 2; neither replay is
   relabeled as an audit, validation, independent review, or independent
   reproduction.
8. All owner review scopes and maintenance tasks are complete at exact revisions,
   labeled `solo-reviewed`, preserve adverse evidence, disclose assistance and
   role overlap, and record M-17 as `unavailable`.

The owner implements candidates under a preregistered alternating order and may
optimize only after the first conforming result is archived. Both pre- and post-
optimization records remain. A generally useful correction reaches all four
candidates in a new symmetric epoch. A candidate-specific shortcut, hidden
cache, stronger hardware, private hint, omitted claim, relaxed policy, or
post-comparison repair is a variance, not a silent advantage.

Case completion and claim outcome remain different layers. A negative or
unsupported claim can be the expected observation in a passing case; it never
becomes partial credit for the claim. Candidate similarity, schema validity,
signature validity, popularity, line count, and owner preference satisfy no
technical gate by themselves.

### Frozen owner-maintenance tasks

Every candidate performs these two tasks in order. Candidate order alternates
under the preregistered schedule, but task text, seeds, ceilings, permitted
assistance, and acceptance rules are identical.

| ID | Published-only change request | Seeded fault | Exact acceptance |
| --- | --- | --- | --- |
| MT-01 | Revise the frozen target-leakage policy so a named `binary_analysis` basis is mandatory for the affected CF-05 target claim; update enforcement, public projection, migration, and tests without changing unrelated claims | A checked optional test masks the missing mandatory basis and leaves the affected profile green | Detect the seed before repair; the affected atomic claim and every containing projection become the frozen non-success, the optional test remains visible without authority upgrade, unrelated claims remain byte-identical, and all new positive/negative fixtures pass |
| MT-02 | Add the frozen claim-policy digest as a mandatory identity edge, revoke the prior policy revision at the declared logical time, and update closure, lifecycle propagation, offline replay, migration, and rendering | Evidence with the old policy digest is accepted through a same-name alias and a cached package summary remains green | Detect the seed before repair; the alias substitution fails closed, exactly the old-policy dependents invalidate, the unaffected set remains byte-identical, history and migration remain attributable, and three offline renders agree |

Each task begins from a distinct clean checkout of the same exact revision with
empty candidate-specific caches and output directories. The record inventories
dependency acquisition, network state, environment, tools, inputs, outputs,
and elapsed owner time. The owner receives only the frozen public packet and
published task text: no private file map, repair note, or candidate-specific
hint. Any automated assistance uses the same preregistered allowance and
retains exact prompts and outputs; prior familiarity and role overlap remain
disclosed. A task passes only if the owner detects its seed before repair,
produces the required migration and adversarial evidence inside the common
ceiling, and recreates the repaired result from a second clean workspace. An
undetected seed, hidden aid, missing record, or incomplete effect set fails the
task and therefore M-16 and hard gate 8.

## 6. Evidence packet and archive layout

The eventual research packet uses this logical layout; this draft creates no
empty research or product directories:

```text
d005-v0.1/
  epochs/0001/
    protocol/
    shared-inputs/
    claim-policies/
    candidates/am-01/
    candidates/am-02/
    candidates/am-03/
    candidates/am-04/
    cross-candidate/
    same-owner-replays/
    owner-reviews/
    decision/
```

Every candidate and cross-candidate record includes:

- suite version, epoch, repository revision, candidate and case IDs, frozen
  input/policy manifest, expected observation, and overall case verdict;
- exact tool, dependency, runtime, operating-system, hardware, logical-time,
  locale, environment, cache, resource, and network identities;
- argument vectors, working directory, ordered input/output manifests, modes,
  sizes, SHA-256 digests, exit category, and bounded diagnostics;
- every atomic claim, family, subject, context, wording, outcome, basis decision,
  assumption, exclusion, trust edge, composition edge, lifecycle state, and
  public projection;
- failed, unsupported, unresolved, revoked, expired, superseded, ambiguous, and
  corrected records as first-class retained evidence;
- candidate variances, migrations, raw metrics, summaries, and nonclaims; and
- owner-pass identity, scope, prior familiarity, assistance, generated
  suggestions, role overlap, conflicts, date, exact revision, `solo-reviewed`
  label, and independent-review status `unavailable`.

Original and superseded records are append-only and content-addressed. A
correction links to prior bytes; it never edits history to make a candidate or
claim appear to have succeeded earlier. Schema validation proves shape only.
Cryptographic authentication proves byte and signer relationships only. Neither
proves the technical truth, authority, scope, or completeness of a claim.

## 7. Owner review scopes

D-023 assigns all scopes to the Orange Project Owner. The table separates
questions and review passes, not people. Each scope has exactly one current
record plus any retained superseded records.

| ID | Accountable owner scope | Required record |
| --- | --- | --- |
| AR-01 | Suite custody and candidate parity | Frozen inputs, policies, expected outcomes, epoch, order, correction window, and proof that shared changes reached all candidates |
| AR-02 | Claim-family and outcome semantics | CF-01 through CF-10 mappings, four-outcome rules, mixed-status results, ambiguities, and nonclaims |
| AR-03 | Evidence authority and identity | Kind-to-basis policy, mandatory-set decisions, subject/context/theorem/proof/evidence binding, and all substitution results |
| AR-04 | Trust and composition | Assumption and TCB closure, compromise propagation, boundary edges, aggregate/profile projections, and missing-edge results |
| AR-05 | Lifecycle and recovery | Expiry, revocation, supersession, downgrade, rollback, stale-cache, logical-time, and historical retention results |
| AR-06 | Public rendering and migration | Canonical machine bytes, human wording, offline inspection, version behavior, v0.1 boundary, and resource results |
| AR-07 | Solo auditability and maintenance | Published-only task inputs, clean-workspace records, seeded faults, repair trace, assistance, and residual same-author bias |
| AR-08 | Comparative disposition | All hard-gate outcomes, per-axis trade-offs, adverse evidence, M-17 disclosure, nonclaims, recommendation, and proposed OEP disposition |

Every AR-01 through AR-08 record names exact bytes and revision, methods,
findings, tools, assistance, prior familiarity, role overlap, unresolved risk,
date, and disposition. Its label is `solo-reviewed`. A bot, second owner pass,
second workspace, alternate implementation, or signature may strengthen the
analysis but is never an independent reviewer, organization, validation, or
technical authority beyond its declared evidence type.

## 8. Decision and acceptance procedure

1. The owner freezes the suite version, candidates, cases, claim families,
   policies, expected outcomes, mutations, identities, resource envelope,
   logical time, run order, correction window, materiality bands, owner scopes,
   and dependency-admission method before measured implementation.
2. The owner publishes the input-manifest digest, implements all candidates in
   the preregistered alternating order, records assistance and prior familiarity,
   and does not inspect comparative summaries until first-pass case records are
   immutable.
3. Each deterministic profile runs three times with network denied from the
   frozen packet. Two separately provisioned owner workspaces recreate the
   manifests. No decision-case result is recorded above reproducibility level 2.
4. The owner completes AC-01 through AC-08 and AR-01 through AR-08, validates
   every hard gate, publishes all failed and adverse records, and records
   independent review as `unavailable`.
5. If one or more candidates pass, the owner publishes a per-axis rationale
   covering semantic precision, authority safety, identity and trust closure,
   composition, lifecycle behavior, public usability, migration, implementation
   surface, and residual solo bias. Eligibility alone is not selection.
6. If no candidate passes, evidence is asymmetric or unreplayable, a shared rule
   was underspecified, legal/dependency admission is absent, or no rationale
   distinguishes eligible candidates, D-005 remains `proposed`. The owner narrows
   or repairs the protocol symmetrically; missing outside participation alone
   does not force an inconclusive result under D-023.
7. A recommendation records the candidate, exact architecture and version
   boundary, rejected-candidate archive, adverse and contrary evidence,
   migrations, nonclaims, expiry, and reopening triggers.

Each hard gate records `pass`, `fail`, `unresolved`, or `unsupported`. Each
comparative metric uses preregistered materiality bands to report
`am01_better`, `am02_better`, `am03_better`, `am04_better`,
`practically_equivalent`, or `inconclusive`, with raw observations and
uncertainty. These labels form a trade-off table, not a score.

The suite conclusion is exactly `recommend_am01`, `recommend_am02`,
`recommend_am03`, `recommend_am04`, `tie`, or `inconclusive`. A recommendation
selects nothing by itself. A tie or inconclusive result leaves D-005 `proposed`.

D-005 closes only through an Accepted Orange Enhancement Proposal under the
ratified governance process. Its `decision-revision` must be exactly 40
lowercase hexadecimal characters naming the fully validated Git revision. The
OEP must list `Orange Project Owner` as the solo-mode review authority, include
an `approval-records` entry containing the literal `solo-reviewed`, bind the
complete suite results, and preserve adverse evidence, nonclaims, migration and
reopening rules. This draft allocates no OEP number. An ADR, schema merge,
implementation, owner-analysis record, or Codex-only review cannot accept D-005,
and no approval record may claim that the owner supplied independent review.

Acceptance of D-005 selects the claim architecture and its semantic safety
rules. It does not by itself ratify a concrete public encoding, satisfy a
technical claim, accept D-006, close S4, or authorize a release. A product
schema and implementation still require their applicable specification,
conformance, migration, security, and milestone gates.

## 9. Structural completion criteria

This protocol is structurally complete only while:

- candidates are 4/4: AM-01 through AM-04 receive the same frozen suite;
- cases are 8/8: AC-01 through AC-08 contain shared inputs, candidate outputs,
  positive checks, mutation/negative checks, and hard acceptance;
- the frozen matrix is 32/32 candidate-case obligations with one honest zero
  execution baseline;
- required claim families are 10/10: CF-01 through CF-10 remain exact and
  include first-class compiler preservation;
- metrics are 18/18, hard gates are 8/8, and owner scopes are AR-01 through
  AR-08 exactly once;
- atomic outcomes remain the exact four-category non-ordered set, case verdicts
  remain binary, and no aggregate or weighted score can upgrade a claim;
- historical v0.1 shape validation remains explicitly non-ratifying;
- same-owner decision evidence is capped at reproducibility level 2 and all
  owner records remain `solo-reviewed`;
- D-005 acceptance requires the exact-revision Accepted OEP in section 8; and
- repository validation passes without implying that a candidate, claim,
  schema, milestone, or release passed.

Execution evidence is currently 0/32 candidate-case executions (0/8 AM-01,
0/8 AM-02, 0/8 AM-03, and 0/8 AM-04). M-17 is `unavailable`. This document
defines the experiment; it does not supply results or select a public assurance
model.
