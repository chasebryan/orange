# D-006 proof-foundation decision suite

Status: institutional protocol suspended by D-023; no proof foundation selected

Suite version: `d006-v0.1-draft`

Snapshot: 2026-07-11

## Solo-mode disposition

This protocol was designed around separate candidate authors, independent
reviewers, external auditors, and practitioner cohorts. Those roles are
unavailable under D-023, so the protocol cannot be executed as written and is
not an active implementation gate.

DS-01 through DS-06 remain useful design inputs for a later owner-executable
comparison. DS-07 and every external-review metric are unavailable and
non-blocking. A revised decision may use reproducible solo-produced evidence,
but it must not describe a second owner run or implementation as independent.
D-006 gates proof-bearing semantics and checker work only; it does not gate the
proof-neutral compiler foundation authorized by D-024.

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
frozen bit-blast/CNF/LRAT specifications, a pinned untrusted solver, an
independent LRAT checker, and D-009 solver-trust policy.

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

**Question:** Can the candidate produce a small, independently runnable,
inspectable checker path for the shared relation on every proposed host without
requiring the interactive prover at replay time?

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
three clean serial/declared-parallel runs and may support reproducibility levels
2 or 3. Wall time, CPU time, RSS, and human observations remain empirical
measurements: independent runs create new raw records and variance summaries,
not a false byte-identical timing claim.

**Mutation and negative checks:** Empty cache, poisoned cache, unavailable
dependency, read-only home, path change, locale/time variation, one-core limit,
timeout, and forced process failure produce the declared bounded behavior and do
not reuse stale success.

**Hard acceptance:** Required deterministic outputs are identical; all measured
inputs and tools are content-identified; no network or undeclared cache is used;
diagnostic and resource failures remain non-successful and attributable; one
non-author witness reproduces the packet in a separately provisioned environment.

### DS-07 — Assess auditability and contributor availability

**Question:** Can independent people understand, review, reproduce, and maintain
the candidate path without relying on its original author or unavailable
specialists?

**Dependencies:** Exact DS-01 through DS-06 digests, preregistered reviewer
qualification/conflict/help/cohort rules, D-019 authority and independence
requirements, and protected retention/privacy rules for participant records.

**Shared inputs:** The same architecture brief, DS-01 through DS-06 artifacts,
two bounded repair/review tasks, conflict-disclosure form, review rubric, and
time/help recording protocol.

**Candidate outputs:** Independent logic/kernel review, extraction/distribution
review, two non-author practitioner task records, issue/diagnostic quality report,
identified maintenance and audit capacity, conflicts, and unresolved risks.

**Positive checks:** Reviewers can locate the exact theorem, assumption, trusted
component, failing case, and produced artifact from published material and can
replay the affected evidence without private help.

**Mutation and negative checks:** Give each reviewer one seeded hidden assumption,
one stale artifact, one ambiguous diagnostic, and one dependency substitution;
the rubric records detection, miss, time, assistance, and impact without changing
the shared task between candidates.

**Hard acceptance:** At least one independent logic/kernel reviewer and one
independent extraction/distribution reviewer assess each candidate, and at least
two non-author practitioners complete the common tasks under matched time and
help rules. The results are feasibility evidence, not a population estimate.
Missing qualified people or an underfilled preregistered cohort makes the
decision inconclusive; it does not automatically make the other candidate sound
or maintainable.

## 4. Comparable metrics

Metrics use the same frozen inputs, hardware allocation, operating-system image,
resource ceilings, and run protocol for both candidates. Candidate order is
paired and interleaved on the same host. Short performance cases use one
unmeasured warmup followed by thirty measured runs; clean bootstrap uses five
independent cold runs. Report every raw observation plus median, median absolute
deviation, p95, and a bootstrap 95% confidence interval where the sample supports
it. No post-hoc outlier trimming is allowed; an invalid machine run needs an
objective recorded cause and remains in the archive. Time and memory from
different hosts are never compared as if equivalent.

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
| M-13 | License/provenance closure | Resolved components / total with exact source and terms | Hard gate: 100%; D-018 approval still external |
| M-14 | Host distribution coverage | Passing required hosts / ratified required hosts | Hard gate: 100% for chosen envelope |
| M-15 | Diagnostic conformance | Cases with required shared ID, category, location, and bounded output / total | Hard gate: 100% |
| M-16 | Practitioner task completion | Completed common tasks / 2, with time and assistance recorded | Hard gate: 2/2; qualitative review retained |
| M-17 | Qualified independent review coverage | Filled required review roles / required roles | Hard gate: 100% with conflicts disclosed |
| M-18 | Maintenance variance | Files, proofs, dependencies, and elapsed reviewer time for the same seeded change | Comparative maintainability evidence |

There is no weighted aggregate score. Weighting milliseconds against soundness,
auditability, or contributor availability would hide non-substitutable risks and
allow a fast candidate to compensate numerically for a failed assurance gate.
Metrics inform the recorded rationale only after all hard gates pass.

## 5. Hard gates and anti-gaming rules

A candidate is eligible for selection only when all of these pass:

1. DS-01 through DS-07 are complete against the same frozen statements and cases.
2. Every required positive observation and cross-candidate canonical result
   agrees, or a reviewed variance proves that the shared input was underspecified.
3. Every required mutation/negative case rejects in its expected category.
4. No axiom, unsafe feature, plugin, native evaluator, solver, runtime, extraction
   step, or build tool is omitted from the applicable trust closure.
5. Clean bootstrap and replay are network-denied, resource-bounded, deterministic,
   and reach reproducibility level 2 plus level 3 aggregate reproduction by a
   distinct human principal in an independently provisioned environment.
6. The standalone checker path agrees with its in-prover relation on the complete
   corpus and supports every host in the envelope under decision.
7. All candidate bytes, sources, tools, dependencies, licenses, and provenance
   are inventoried; unresolved legal terms block selection but are not resolved
   by a technical score.
8. Required independent technical reviews and practitioner tasks are complete,
   conflict-disclosed, and attached to exact revisions.

Candidate teams may optimize only after the first conforming result is archived.
They must publish both pre- and post-optimization results and apply any generally
useful protocol correction to both candidates. A candidate-specific timeout,
hardware advantage, hidden cache, admitted library, hand-written shortcut, or
review assistance is a recorded variance, not a silent advantage.

Generated code is counted and archived separately from reviewed handwritten
source. Proof line count, theorem count, GitHub popularity, benchmark wins, or
one expert's familiarity cannot satisfy a gate. A missing feature cannot be
scored as zero cost.

## 6. Evidence packet and archive layout

The eventual research packet should use this logical layout; this document does
not create empty product or research directories:

```text
d006-v0.1/
  epochs/0001/
    protocol/
    shared-inputs/
    candidates/rocq/
    candidates/lean4/
    cross-candidate/
    reproductions/
    reviews/
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
- complete trust inventory and claim/non-claim statements; and
- author, reproducer, reviewer, conflict, date, revision, and attestation scope.

The provisional Gate 0 evidence, trust-inventory, and claim schemas may record
parts of an individual case, but the `v0.1` evidence manifest is not sufficient
as the suite record. It represents one replay and has no first-class candidate,
case, host, resource-budget, raw-metric, reproduction-level, reviewer, or
protocol-amendment fields. Its environment allowlist also cannot yet express
every affecting variable required by the reproducibility contract. Before a
measured run, a versioned D-006 suite-index/result/reproduction layer or a
general evidence-schema `v0.2` must model those fields, resolve that environment
contract, and ship positive, negative, and migration cases. Implementers must
not hide missing structure in free-form strings. Schema acceptance still proves
shape only, not review or decision truth.

Original inputs, rejected candidate results, failed runs, raw observations, and
superseded protocol versions remain content-addressed. A correction adds a new
record linked to the old digest. It never edits history to make the selected
candidate appear to have passed earlier.

## 7. Review roles

The same person may fill a technical role for both candidates when qualified and
conflict-disclosed, but candidate authors cannot approve their own work. The
proposed minimum roles are:

- neutral suite custodian, responsible for frozen shared inputs and parity;
- Rocq implementation author and separate candidate reviewer;
- Lean implementation author and separate candidate reviewer;
- Language and Semantics reviewer for statement equivalence;
- Assurance and TCB reviewer for logic, axioms, certificates, and trust closure;
- compiler/bootstrap reviewer for extraction and distribution;
- reproducibility witness for each independent environment;
- dependency and legal/IP reviewer for provenance and terms;
- external logic/kernel auditor and external distribution auditor; and
- two non-author practitioner participants per candidate.

Every review identifies the exact digest, scope, methods, findings, assistance,
conflicts, and disposition. A bot or a second pass by the author is useful
analysis but not independent approval.

## 8. Decision procedure

1. Ratify the protocol version, shared statements, host/resource envelope,
   dependency-admission method, and reviewer independence criteria before a
   measured candidate run.
2. Freeze and publish the shared input-manifest digest. Candidate teams implement
   independently, then cross-review statement mappings before seeing comparative
   performance summaries.
3. Run each deterministic profile three times in clean network-denied
   environments, complete the five-cold-bootstrap and thirty-timed-run protocol,
   and obtain independent reproductions and required reviews.
4. Validate hard gates and canonical cross-candidate agreement. A failed gate
   makes that candidate ineligible for the tested envelope; it does not erase its
   evidence or prove the other candidate acceptable.
5. If both candidates pass, the authorized Gate 0 decision body publishes a
   per-axis rationale covering assurance boundary, semantic fit, proof and
   checker maintainability, distribution/bootstrap, external capacity, and
   measured resources. It must explain material disadvantages and rejected
   alternatives without collapsing them into a weighted score.
6. If neither passes, both pass but reviewers cannot justify a choice, shared
   statements were materially underspecified, qualified review is unavailable,
   or D-018/D-019 remains blocking, D-006 stays `investigate`. Narrow the envelope,
   repair the protocol symmetrically, or collect more evidence.
7. An accepted decision records the selected candidate and exact version range,
   migration/compatibility boundary, rejected-candidate archive, dissent,
   review expiry, and triggers for reopening the choice.

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
through an accepted Orange Enhancement Proposal under ratified governance. An
ADR, implementation merge, candidate adapter, solo-steward declaration, or
Codex-only review cannot accept it or supply the required independence.

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
- metrics are at least 12 and preserve raw observations; this version defines 18;
- every hard gate is non-compensable and no weighted aggregate exists;
- evidence, review, archive, decision, tie, and inconclusive behavior are explicit;
  and
- local repository validation passes without implying either candidate or Gate 0
  has passed.

Execution evidence is currently 0/2 candidates and 0/7 cases. Independent review
is currently absent. This document defines the experiment; it does not supply
its results.
