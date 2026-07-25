# D-006 proof-foundation decision suite

Status: owner-executable draft under D-023; no proof foundation selected

Suite version: `d006-v0.2-draft`

Snapshot: 2026-07-25

## Solo-mode disposition

This revision replaces the suspended institutional protocol with a comparison
that the project owner can execute without fictional candidate teams, reviewers,
auditors, or practitioner cohorts. The owner implements and exercises both
candidates under the same frozen packet, performs separately logged review
passes, and records every role overlap, tool, correction, and unresolved risk.
Those controls improve repeatability and error detection; they do not create
independent human or organizational evidence.

DS-01 through DS-07, all deterministic and adversarial checks, and all
candidate-parity rules remain mandatory. External review is recorded as
`unavailable` and cannot distinguish the candidates or be traded against a
technical failure. It does not block an otherwise conforming owner selection
under D-023. A second owner run, workspace, tool, or implementation is always
labeled same-owner evidence, never independent reproduction or review.

D-006 gates proof-bearing semantics and checker work only; it does not gate the
proof-neutral compiler foundation authorized by D-024. This draft selects no
candidate, authorizes no dependency installation, and authorizes no
proof-bearing implementation. Selection requires the exact acceptance process
in section 8.

## 1. Decision boundary

This protocol supplies the reproducible comparison required by
[decision D-006](DECISIONS.md#d-006--proof-foundation). Rocq and Lean 4 are the
two mandatory candidates. A third candidate may enter only through a reviewed
scope change and must run the identical frozen suite; it cannot replace a weak
result with a candidate-specific demonstration.

The suite chooses no surface syntax, product implementation, package namespace,
license, or final Core. Its artifacts are Gate 0 architecture evidence outside
the product lineage. The selected candidate's cases may later graduate into the
permanent metatheory and conformance suites; rejected cases remain archived and
replayable without becoming a second Orange implementation.

No proof toolchain is installed, account created, license accepted, or package
downloaded by this document. Before execution, exact tool and dependency terms
must pass owner dependency admission under D-018 and D-023. This requirement
does not apply to the admitted dependency-free Rust compiler slice.

## 2. Candidate parity and frozen inputs

| ID | Candidate | Required treatment | Current execution evidence |
| --- | --- | --- | --- |
| C-01 | Rocq | Run the complete frozen suite with idiomatic, fully inventoried candidate artifacts | 0/7 cases |
| C-02 | Lean 4 | Run the complete frozen suite with idiomatic, fully inventoried candidate artifacts | 0/7 cases |

The frozen matrix contains exactly 14 candidate-case runs per evidence epoch:
each of the 2 candidates runs each of the 7 cases.

Both candidates receive one foundation-neutral input packet containing:

- the mathematical judgments and metatheory statements for the shared fragments;
- canonical UTF-8/JSON test inputs and expected observation schemas;
- positive, malformed, mutation, ambiguity, resource, and unsupported cases;
- exact resource ceilings and timeout semantics;
- a frozen Gate 0 evaluation-host matrix, with unsupported hosts reported rather
  than hidden; this test matrix does not itself accept D-011;
- a shared diagnostic taxonomy covering parse/type failure, disproved obligation,
  `unknown`, timeout, unsupported feature, untrusted solver step, failed
  certificate, unmet target assumption, and resource exhaustion;
- the metric and diagnostic rubric in this document; and
- an input-manifest digest covering every byte, mode, path, and case ID.

Candidate adapters may use idiomatic Rocq or Lean source, libraries, build tools,
and extraction mechanisms, but may not change a shared statement, weaken a
negative case, precompute a measured result, call an unrecorded oracle, or add a
trusted plugin. A necessary semantic deviation becomes a named variance with
its impact; it never silently changes the shared input.

The execution packet freezes exact candidate versions, package/dependency
graphs, build images or equivalent declared environments, resource limits,
locale, time source, randomness, and host identities. Capture may use the
network. Measured bootstrap and replay run with network access denied and an
empty candidate-specific cache unless a case explicitly measures a populated
cache separately.

One evidence epoch freezes the packet before either measured implementation
begins. Both candidates receive the same preregistered correction window. A
candidate defect may be corrected within it and retains its failed run; an
ambiguity or change in shared cases, rubric, resource policy, toolchain pinning,
or decision rule creates a new epoch and reruns both candidates.

All replay records follow [`REPRODUCIBILITY.md`](REPRODUCIBILITY.md): argument
vectors rather than shell strings, an allowlisted environment, pinned tool and
input digests, deterministic output manifests, and explicit non-success for
missing input, timeout, resource exhaustion, crash, or digest mismatch.

The two required owner workspaces use distinct clean checkouts, candidate
caches, output roots, and acquisition records, and reconstruct inputs from the
content-addressed packet. They may use the same declared physical host; any
shared host, tool store, or other trust root is disclosed. Their matching result
is reproducibility level 2. Neither workspace is entered into an
`independent_reproductions` field.

## 3. Required decision cases

### DS-01 — Define and check the proposed Core fragment

**Question:** Can the candidate represent, check, compute, and inspect the same
small foundation-neutral Core without hidden axioms or candidate-only semantics?

**Dependencies:** The frozen foundation-neutral Core packet, D-004 semantic
strata, D-005 trust/claim vocabulary, and the Gate 0 reproducibility profile.

**Shared inputs:** A typed total fragment with universes/sorts, inductive data,
functions, equality, mathematical naturals, exact bit vectors, length-indexed
sequences, explicit word/integer and endian conversion, a parameterized module,
and one canonical decoder with typed failure. The packet includes valid terms,
ill-typed terms, non-total recursion, width/endian confusion, invalid decoding,
and resource-bound cases.

**Candidate outputs:** Idiomatic formalization, checked definitions, evaluator or
normalization observations, complete axiom/extension inventory, stable mapping
from shared IDs to candidate declarations, and machine-readable diagnostics.

**Positive checks:** Both candidates accept every valid shared term, compute the
same canonical observations, and expose the same intended theorem statements.

**Mutation and negative checks:** Each candidate rejects or bounds every
ill-typed, non-total, ambiguous-width, malformed-decoder, oversized, and unknown-
extension case. A diagnostic must identify the shared case and failure category;
a crash, hang, or implicit axiom is not rejection.

**Hard acceptance:** Shared positive observations agree 100%; all required
negative cases fail in the expected category; every trusted primitive, axiom,
plugin, kernel extension, and host evaluation mechanism is inventoried.

### DS-02 — Mechanize progress, preservation, and leakage

**Question:** Can the candidate support the core metatheory and the security-
relevant relational reasoning Orange needs, with reviewable statements and
proof replay?

**Dependencies:** Accepted DS-01 statement mappings; a frozen suite-only trace,
public-equivalence, and declassification model; and the assurance requirements
for leakage and proof checking. The suite model exercises but does not ratify
D-012.

**Shared inputs:** A small typed operational semantics with values, explicit
failure, fixed words, public/secret labels, public branches, memory-address and
control-flow traces, and two representative programs: one trace-equivalent
public-control implementation and one deliberately secret-branching rejection
case.

**Candidate outputs:** Formal statements and proofs of progress and preservation,
a fragment-level two-run leakage noninterference lemma, the positive program as
a checked witness, complete assumptions, and an explicit counterexample or
failed precondition for the negative program.

**Positive checks:** Proofs replay from clean inputs; statements use the shared
judgment IDs and quantify over the same observations, public relation, and
declassification boundary.

**Mutation and negative checks:** Remove one typing premise, alter one transition,
mislabel one secret, expose one secret-dependent branch, and corrupt one proof
object. The affected theorem or case must not remain accepted.

**Hard acceptance:** All three required theorem families replay; the negative
program does not satisfy the leakage claim; no candidate hides an assumption in automation,
native evaluation, an unsafe declaration, or an unlisted classical axiom.

### DS-03 — Validate canonical serialization

**Question:** Can the candidate validate and reason about a stable external
format without treating its own compiled heap or parser behavior as canonical?

**Dependencies:** The DS-01 value model, a frozen suite-only wire specification
and resource limits, and the canonical-byte rules in the reproducibility
contract. This case deliberately does not depend on or ratify D-007's eventual
product proof format.

**Shared inputs:** A bounded canonical record format for definitions, theorem
fingerprints, claims, and references; exact byte fixtures for valid objects,
duplicate names, non-canonical ordering, invalid UTF-8, malformed numbers,
unknown fields, path/reference escape, cyclic reference, oversized input, and
semantically equivalent but non-canonical encodings.

**Candidate outputs:** A format model, decoder/validator relation, canonical
encoder where applicable, byte-to-value and value-to-byte properties, stable
error categories and paths, and output digests for valid canonical bytes.

**Positive checks:** Both candidates produce identical canonical bytes and
digests for every accepted value and prove or check the required round-trip and
uniqueness statements over the supported fragment.

**Mutation and negative checks:** Every malformed, duplicate, non-canonical,
escaping, cyclic, oversized, and unknown-version fixture is rejected before it
can become a theorem or claim identity.

**Hard acceptance:** Accepted byte sets and output digests agree 100%; the
candidate's internal serialization is never the public proof/evidence format;
decoding is deterministic and resource-bounded.

### DS-04 — Replay an LRAT-backed bit-vector proof

**Question:** Can untrusted Boolean search produce a compact certificate that the
candidate checks without adding the solver to the logical TCB?

**Dependencies:** DS-01 word semantics, DS-03 identity binding and diagnostics,
frozen bit-blast/CNF/LRAT specifications, a pinned untrusted solver, a
solver-independent LRAT checker path, and D-009 solver-trust policy. Here,
`solver-independent` means that certificate replay does not execute or trust the
search solver; it makes no claim about a separate author or organization.

**Shared inputs:** One nontrivial fixed-width equivalence obligation, canonical
bit-blast and CNF rules, pinned untrusted solver bytes/argv/seed, a supplied
golden CNF/LRAT interoperability fixture, a satisfiable counterexample case, and
truncated, reordered, forged-step, wrong-CNF, oversized, and resource-exhaustion
variants.

**Candidate outputs:** Verified or explicitly modeled translation connection,
candidate-emitted canonical CNF, freshly generated LRAT certificate and untrusted
search log, certificate parser/checker, solver-free replay artifact, theorem/CNF/
certificate fingerprints, resource report, and stable diagnostics for every
rejected variant.

**Positive checks:** First, the pinned solver produces a fresh LRAT certificate
from the candidate-emitted canonical CNF under frozen arguments and seed. Second,
that certificate replays without the solver. The supplied golden certificate
also replays as an interoperability fixture. Each closes only the exact shared
obligation.

**Mutation and negative checks:** Apply truncation, step forgery, reordering, CNF
substitution, and theorem-identity changes to both fresh and golden certificates.
Satisfiable input, missing solver output, timeout, and resource exhaustion must
leave the claim non-successful with the exact reason.

**Hard acceptance:** The solver executable and heuristics are absent from the
logical TCB; 100% of adversarial certificates are rejected; theorem, CNF, and
certificate digests are bound together; missing proof output never becomes a
successful automation result.

### DS-05 — Extract and distribute the authoritative checker case

**Question:** Can the candidate produce a small, standalone, inspectable checker
path for the shared relation on every proposed host without requiring the
interactive prover at replay time?

**Dependencies:** The DS-03/DS-04 relation and corpus, the frozen Gate 0
evaluation-host matrix, admitted and captured extraction/build toolchains, and
D-018 review of every relevant dependency and redistribution term. The matrix
informs but does not accept D-011; a later different target envelope reruns the
affected case.

**Shared inputs:** The DS-03/DS-04 validation relation and frozen accepted/rejected
corpus, evaluation-host matrix, distribution constraints, and a clean bootstrap
manifest.

**Candidate outputs:** Extracted/compiled checker artifact or equivalently narrow
standalone checked path, build recipe, bootstrap closure, host packages, source-
to-binary mapping, TCB inventory, and usage/diagnostic records.

**Positive checks:** A clean user can build or acquire, authenticate, invoke, and
replay the corpus without an interactive IDE, registry, network, tactic, plugin,
or solver at check time.

**Mutation and negative checks:** Unsupported host, missing runtime, altered
checker byte, incompatible format, malformed proof, absent dependency, and
bootstrap mismatch fail without substituting a different candidate version.

**Hard acceptance:** Every required evaluation-host row is supported or the
candidate is explicitly ineligible for that tested envelope; the standalone
path agrees with the in-prover result on 100% of cases; all runtime and extraction
trust is listed.

### DS-06 — Measure clean bootstrap, replay, diagnostics, and dependency surface

**Question:** What reproducible engineering and audit cost does each candidate
impose when completing the same cases?

**Dependencies:** Frozen DS-01 through DS-05 artifacts, exact runner and observer
identities, host/resource/cache protocol, shared diagnostic taxonomy, and a
dependency-snapshot cutoff.

**Shared inputs:** Frozen DS-01 through DS-05 packets, identical host/resource
allocation, cold-cache and declared warm-cache profiles, fault-injection cases,
and the metric protocol in section 4.

**Candidate outputs:** Five cold bootstrap records, three deterministic serial
and declared-parallel replay manifests, thirty paired/interleaved timed replays
after one unmeasured warmup, peak-resource observations, diagnostic rubric
results, binary/source/dependency inventories, and variance notes.

**Positive checks:** Deterministic artifacts and static sizes match across the
three clean serial/declared-parallel runs and may support reproducibility level
2. Wall time, CPU time, RSS, and owner observations remain empirical
measurements: separate runs create new raw records and variance summaries, not
a false byte-identical timing claim.

**Mutation and negative checks:** Empty cache, poisoned cache, unavailable
dependency, read-only home, path change, locale/time variation, one-core limit,
timeout, and forced process failure produce the declared bounded behavior and do
not reuse stale success.

**Hard acceptance:** Required deterministic outputs are identical; all measured
inputs and tools are content-identified; no network or undeclared cache is used;
diagnostic and resource failures remain non-successful and attributable; and a
second, separately provisioned owner workspace recreates the packet from its
archived inputs. Both workspaces are same-owner level-2 evidence. Neither is an
independent reproduction.

### DS-07 — Exercise solo auditability and maintenance

**Question:** Can the owner locate, challenge, replay, and modify each candidate
from its published packet under matched controls, without relying on unrecorded
session state or candidate-specific assistance?

**Dependencies:** Exact DS-01 through DS-06 digests; a preregistered seed,
task-order, clean-workspace, time, and help protocol; the owner review scopes in
section 7; D-019 and D-023 authority; and protected retention rules for raw
records. An elapsed cooling period may be recorded but is not treated as
independence.

**Shared inputs:** The same architecture brief and DS-01 through DS-06 artifacts;
one bounded published-material audit task; one bounded seeded maintenance task;
one hidden-assumption mutation, stale-artifact substitution, ambiguous diagnostic,
and dependency substitution per candidate; and identical recording forms. Seeds,
time ceilings, permitted tools, and help rules are frozen before either task.

**Candidate outputs:** Exact-digest `solo-reviewed` audit and maintenance records;
the located theorem, assumption, trusted component, failing case, and produced
artifact; a patch or repair trace for the common change; detection results for
all seeded faults; time and assistance logs; role-overlap and prior-familiarity
disclosures; issue/diagnostic quality notes; and unresolved risks.

**Positive checks:** Starting from a clean workspace and only the frozen packet,
the owner can locate and replay the requested evidence, explain its trust closure,
and complete the same maintenance task for each candidate. Candidate order is
counterbalanced across audit and maintenance tasks. Every command, consulted
source, tool-produced suggestion, and candidate-specific hint is recorded.

**Mutation and negative checks:** The owner attempts to detect the seeded hidden
assumption, stale artifact, ambiguous diagnostic, and dependency substitution
without changing the task between candidates. A miss, timeout, unrecorded input,
or inability to reconstruct the relevant result is retained as a failed task;
it is not repaired silently after comparative results are visible.

**Hard acceptance:** Both candidates complete 2/2 common owner tasks within the
frozen rules; all seeded faults are detected and correctly classified; exact
published bytes suffice for replay and repair; and all assistance, prior
familiarity, and role overlap are disclosed. This measures same-owner packet
auditability and maintenance cost only. It supplies no evidence about a new
maintainer, contributor availability, independent review, or external audit.

## 4. Comparable metrics

Metrics use the same frozen inputs, hardware allocation, operating-system image,
resource ceilings, and run protocol for both candidates. Candidate order is
paired and interleaved on the same host. Short performance cases use one
unmeasured warmup followed by thirty measured runs; clean bootstrap uses five
cold runs. Report every raw observation plus median, median absolute deviation,
p95, and a bootstrap 95% confidence interval where the sample supports it. No
post-hoc outlier trimming is allowed; an invalid machine run needs an objective
recorded cause and remains in the archive. Time and memory from different hosts
are never compared as if equivalent.

| ID | Metric | Unit and method | Decision use |
| --- | --- | --- | --- |
| M-01 | Required case completion | Passed cases out of 7 | Hard gate: 7/7 |
| M-02 | Positive observation agreement | Matching shared observations / total | Hard gate: 100% |
| M-03 | Mutation and negative rejection | Expected rejections / total, by category | Hard gate: 100% |
| M-04 | Clean deterministic replay | Matching serial and declared-parallel output-manifest digests across 3 runs | Hard gate: 3/3 per claimed profile |
| M-05 | Authoritative/standalone agreement | Matching decisions / complete corpus | Hard gate: 100% |
| M-06 | Undeclared trust | Count of unlisted axioms, plugins, unsafe steps, native evaluators, and tools | Hard gate: 0 |
| M-07 | Clean bootstrap time | Wall/CPU seconds, peak RSS, and temporary bytes for each of 5 cold runs | Comparative; raw values retained |
| M-08 | Proof/certificate replay time | Wall/CPU seconds for 30 paired/interleaved measured runs per case | Comparative; median/MAD/p95/CI plus raw values |
| M-09 | Peak resident memory | Peak bytes measured by one pinned observer | Comparative and resource-ceiling gate |
| M-10 | Standalone checker size | Exact bytes, stripped and unstripped where meaningful | Comparative, never a soundness proxy |
| M-11 | Formal source review surface | UTF-8 lines and bytes by trusted, proof, test, generated, and glue role | Audit planning; no smaller-is-better score |
| M-12 | Bootstrap/dependency closure | Component count and total archived bytes by trust role | Hard gate: 100% identified and retrievable |
| M-13 | License/provenance closure | Resolved components / total with exact source and terms | Hard gate: 100%; owner dependency admission remains a separate D-018 disposition |
| M-14 | Host distribution coverage | Passing required hosts / ratified required hosts | Hard gate: 100% for chosen envelope |
| M-15 | Diagnostic conformance | Cases with required shared ID, category, location, and bounded output / total | Hard gate: 100% |
| M-16 | Owner audit/maintenance task completion | Completed common DS-07 tasks / 2 per candidate, with time, assistance, and role overlap recorded | Hard gate: 2/2 per candidate and every seeded fault detected |
| M-17 | Independent-review status | Exact status for logic/kernel, extraction/distribution, and comparative-decision review | Disclosure only: `unavailable`; never a selection score or implied claim |
| M-18 | Same-owner maintenance variance | Files, proofs, dependencies, and elapsed owner time for the same seeded change | Comparative maintainability evidence; no external-usability inference |

There is no weighted aggregate score. Weighting milliseconds against soundness,
auditability, or contributor availability would hide non-substitutable risks and
allow a fast candidate to compensate numerically for a failed assurance gate.
Metrics inform the recorded rationale only after all hard gates pass. M-17 is a
mandatory disclosure, not a technical hard gate: under D-023 its value remains
`unavailable` unless qualifying participation actually occurs. No value in
M-01 through M-16 or M-18 compensates for that assurance limitation or turns it
into independent evidence.

## 5. Hard gates and anti-gaming rules

A candidate is eligible for selection only when all of these pass:

1. DS-01 through DS-07 are complete for both candidates against the same frozen
   statements and cases.
2. Every required positive observation and cross-candidate canonical result
   agrees, or a reviewed variance proves that the shared input was underspecified.
3. Every required mutation/negative case rejects in its expected category.
4. No axiom, unsafe feature, plugin, native evaluator, solver, runtime, extraction
   step, or build tool is omitted from the applicable trust closure.
5. Clean bootstrap and replay are network-denied, resource-bounded, deterministic,
   reach reproducibility level 2, and recreate the declared manifests in two
   separately provisioned owner workspaces. Level 3 is neither required nor
   claimed; same-owner workspaces never populate an independent-reproduction
   field.
6. The standalone checker path agrees with its in-prover relation on the complete
   corpus and supports every host in the envelope under decision.
7. All candidate bytes, sources, tools, dependencies, licenses, and provenance
   are inventoried; unresolved legal terms block selection but are not resolved
   by a technical score.
8. All owner review scopes and DS-07 tasks are complete, attached to exact
   revisions, labeled `solo-reviewed`, and disclose role overlap, prior
   familiarity, assistance, tools, findings, and unresolved risk. M-17 records
   independent review as `unavailable` rather than manufacturing approval.

The owner may optimize a candidate only after its first conforming result is
archived. Both pre- and post-optimization results are published, and any
generally useful protocol correction is applied to both candidates. A
candidate-specific timeout, hardware advantage, hidden cache, admitted library,
hand-written shortcut, or assistance is a recorded variance, not a silent
advantage.

Generated code is counted and archived separately from owner-reviewed
handwritten source. Proof line count, theorem count, GitHub popularity,
benchmark wins, or the owner's familiarity cannot satisfy a gate. A missing
feature cannot be scored as zero cost.

## 6. Evidence packet and archive layout

The eventual research packet should use this logical layout; this document does
not create empty product or research directories:

```text
d006-v0.2/
  epochs/0001/
    protocol/
    shared-inputs/
    candidates/rocq/
    candidates/lean4/
    cross-candidate/
    same-owner-replays/
    owner-reviews/
    decision/
```

Every candidate and cross-candidate run records:

- suite version, repository revision, candidate identity, and input-manifest
  digest;
- exact tool, library, runtime, operating-system, hardware, and observer
  identities and acquisition/provenance;
- license and redistribution status without asserting legal approval;
- argument vectors, working directory, allowlisted environment, resource limits,
  network policy, cache state, and expected exit category;
- ordered input/output manifests with paths, modes, sizes, and SHA-256 digests;
- theorem, proof, certificate, checker, extracted artifact, and diagnostic IDs;
- all raw metrics, not only summaries;
- failure, variance, unsupported, and nondeterminism records;
- complete trust inventory and claim/non-claim statements;
- owner-pass identity, role/scope, prior familiarity, assistance, conflicts,
  date, revision, and attestation scope; and
- the explicit `unavailable` status of independent human and organizational
  review.

The provisional Gate 0 evidence, trust-inventory, and claim schemas may record
parts of an individual case, but the `v0.1` evidence manifest is not sufficient
as the suite record. It represents one replay and has no first-class candidate,
case, host, resource-budget, raw-metric, reproduction-level, reviewer, or
protocol-amendment fields. Its environment allowlist also cannot yet express
every affecting variable required by the reproducibility contract. Before a
measured run, a versioned D-006 suite-index/result/same-owner-replay layer or a
general evidence-schema `v0.2` must model those fields, resolve that environment
contract, and ship positive, negative, and migration cases. Implementers must
not hide missing structure in free-form strings. Schema acceptance still proves
shape only, not review or decision truth.

Original inputs, rejected candidate results, failed runs, raw observations, and
superseded protocol versions remain content-addressed. A correction adds a new
record linked to the old digest. It never edits history to make the selected
candidate appear to have passed earlier.

## 7. Owner review scopes

D-023 assigns every scope to the project owner. The separation below is between
recorded questions and passes, not between people. Completion requires exactly
one current record for each scope; the owner may add superseding records but may
not omit an adverse or obsolete one.

| ID | Accountable owner scope | Required record |
| --- | --- | --- |
| R-01 | Suite custody and parity | Frozen inputs, epoch, correction window, execution order, and proof that every shared change reached both candidates |
| R-02 | Rocq construction and conformance | Exact candidate mapping, implementation log, DS-01 through DS-07 results, and candidate-specific limitations |
| R-03 | Lean 4 construction and conformance | Exact candidate mapping, implementation log, DS-01 through DS-07 results, and candidate-specific limitations |
| R-04 | Language and semantics equivalence | Judgment-by-judgment comparison, canonical-observation agreement, variances, and unresolved ambiguity |
| R-05 | Assurance and trust closure | Logic, axioms, unsafe features, automation, solvers, certificates, kernels, runtimes, and all claim/non-claim effects |
| R-06 | Bootstrap and distribution | Extraction or standalone path, host coverage, dependency closure, clean replay, resource behavior, and unsupported hosts |
| R-07 | Dependency and provenance disposition | Exact sources, versions, acquisition, terms, redistribution status, unresolved IP risk, and the separate D-018 admission reference |
| R-08 | Solo auditability and maintenance | DS-07 task inputs, clean-workspace records, assistance, seeded-fault results, repair trace, and residual same-author bias |
| R-09 | Comparative disposition | All hard-gate outcomes, per-axis trade-offs, adverse evidence, M-17 `unavailable` status, nonclaims, and proposed OEP disposition |

Every R-01 through R-09 record identifies the exact digest and revision, scope,
methods, findings, tools and generated suggestions, assistance, prior
familiarity, role overlap, unresolved risk, date, and disposition. Its review
label is `solo-reviewed`. A bot, a second owner pass, a second workspace, or an
implementation-diverse checker may strengthen the analysis but is never an
independent approval, reviewer, or organization.

## 8. Decision procedure

1. Before a measured candidate run, the owner records the protocol version,
   shared statements, host/resource envelope, dependency-admission method,
   owner review scopes, task order, correction window, and materiality bands.
   Every candidate tool and dependency must first receive its required D-018
   disposition; this draft grants no admission.
2. Freeze and publish the shared input-manifest digest. The owner implements
   both candidates under the preregistered alternating order, records all prior
   familiarity and assistance, and completes the statement mappings before
   viewing comparative performance summaries. A shared correction creates a new
   symmetric epoch as section 2 requires.
3. Run each deterministic profile three times in clean network-denied
   environments, complete the five-cold-bootstrap and thirty-timed-run protocol,
   and recreate the deterministic manifests in two separately provisioned owner
   workspaces. Record no result above reproducibility level 2.
4. Complete R-01 through R-09 and DS-07, then validate every hard gate and
   canonical cross-candidate agreement. A failed gate makes that candidate
   ineligible for the tested envelope; it does not erase its evidence or prove
   the other candidate acceptable. External review remains explicitly
   `unavailable` and is not a tie-breaker.
5. If one or both candidates pass, the owner publishes a per-axis rationale
   covering assurance boundary, semantic fit, proof and checker maintainability,
   distribution/bootstrap, dependency/provenance closure, solo auditability,
   and measured resources. It must explain material disadvantages and adverse
   evidence without collapsing them into a weighted score. One eligible
   candidate still requires an affirmative rationale; eligibility alone is not
   selection.
6. If neither passes, both pass but the evidence does not justify a choice,
   shared statements were materially underspecified, or a required D-018
   admission or D-019 authority is absent, D-006 stays `investigate`. Narrow the
   envelope, repair the protocol symmetrically, or collect more owner-executable
   evidence. Unavailable outside participation alone does not force
   `inconclusive` under D-023.
7. An accepted decision records the selected candidate and exact version range,
   migration/compatibility boundary, rejected-candidate archive, adverse and
   contrary evidence, review expiry, and triggers for reopening the choice.

D-006 acceptance requires D-004 and D-005 to be Accepted in their governing
records; proposed, investigate, or implementation-only states do not satisfy
this dependency gate.

Each hard gate records `pass`, `fail`, `unresolved`, or `unsupported`. Each
comparative metric uses preregistered materiality bands to report
`rocq_better`, `lean_better`, `practically_equivalent`, or `inconclusive`, with
raw data and uncertainty beside the label. These labels form a trade-off table,
not an aggregate score. If the evidence has no justified Pareto or governance
choice under the frozen envelope, D-006 remains `investigate`.

The suite conclusion is exactly `recommend_rocq`, `recommend_lean`, `tie`, or
`inconclusive`. A tie means complete eligible evidence does not distinguish the
candidates under the frozen rule. Inconclusive means required evidence is
missing, unreplayable, asymmetric, conflicted, legally blocked, or outside the
frozen envelope. A recommendation may advance an OEP but selects nothing by
itself; `tie` and `inconclusive` both leave D-006 as `investigate`.

Because D-006 fixes a normative proof and metatheory foundation, it closes only
through an Accepted Orange Enhancement Proposal under ratified governance. The
`decision-revision` value must be exactly 40 lowercase hexadecimal characters
and name the fully validated Git revision. The OEP must list `Orange Project Owner`
as its solo-mode review authority, contain an `approval-records` entry with the
literal `solo-reviewed`, and preserve the suite results, adverse evidence,
nonclaims, and reopen triggers. An ADR, implementation merge, candidate adapter,
owner-analysis record, or Codex-only review cannot accept D-006. No approval
record may claim that the owner supplied independent review.

No candidate becomes Orange's foundation because its adapter landed first, its
syntax appears in product docs, its package graph is convenient, or one benchmark
is faster. Proof-bearing implementation remains prohibited until its incremental
proof and canonical-format gates close. Unrelated compiler implementation is
authorized.

## 9. Structural completion criteria

This protocol is structurally complete only while:

- candidates are 2/2: Rocq and Lean 4 receive the same frozen suite;
- cases are 7/7: DS-01 through DS-07 contain shared inputs, candidate outputs,
  positive checks, mutation/negative checks, and hard acceptance;
- metrics are 18/18 and preserve raw observations;
- hard gates are 8/8, every gate is non-compensable, and no weighted aggregate
  exists;
- owner review scopes are 9/9, carry `solo-reviewed`, and never imply separate
  people;
- decision-case reproduction is capped at level 2 while only the owner executes
  the suite;
- D-006 acceptance requires an exact-revision Accepted OEP under section 8;
- evidence, review, archive, decision, tie, and inconclusive behavior are explicit;
  and
- local repository validation passes without implying either candidate or Gate 0
  has passed.

Execution evidence is currently 0/14 candidate-case runs (0/7 Rocq and 0/7 Lean
4). M-17 is `unavailable`. This document defines the experiment; it does not
supply its results or select a proof foundation.
