# Orange complete 1.0 execution record

Status: development plan under explicit owner direction, 2026-10-01;
no release, semantic acceptance, or foundational decision is recorded here

The target is the complete 1.0 product in
[`PROJECT_CHARTER.md`](PROJECT_CHARTER.md), including every criterion in
section 9 and every owner-executable journey in
[`USER_JOURNEYS.md`](USER_JOURNEYS.md). The target includes the language,
mechanized semantics, proof and claim system, verified compiler, developer
tools, cryptographic corpus, package and evidence lifecycle, and release
operations. This plan does not substitute a source preview or a smaller
toolchain release for that product.

[`ROADMAP.md`](ROADMAP.md) governs dependencies and stage closure;
[`DECISIONS.md`](DECISIONS.md) records authority. Implementation may proceed
within explicit owner direction, while a component or claim that requires an
accepted decision retains that gate. Repository checks and merge demonstrate
engineering integration; they do not close those decisions or authorize
publication.

## Current engineering boundary

The permanent Rust frontend implements bounded parsing, stable diagnostics,
typed pure specifications, a reference evaluator, known-answer tests, and
acyclic multi-module programs. Its proposed S3 slices include exact integers,
fixed-width words, residues, arrays and rectangular scalar rows, tuples,
conditions, bounded folds, conversions, slices, and finite size and type
specialization. S3t extends those finite size instances to modulus expressions,
and S3u extends arrays to four dimensions with update paths. S3b through S3u
remain in owner review; the accepted semantic boundary is S3a.

The algorithm directories contain executable mathematical specifications and
tested vectors. The P2 five-limb field definitions supply representation
predicates, addition, carrying and canonicalization. Partial P4 mathematical
preparation adds exact `Int` product accumulators and three-pass normalization,
biased subtraction, dedicated squaring and a24 multiplication observations. A
predicate returning `true` is not a checked refinement proof; P4 remains
incomplete. The CLI also has file-sealing commands that run Orange
specifications through the reference evaluator. Neither these commands nor
their vectors establish verified native cryptography or a release claim.

Tabula is an implemented local editor and reference-evaluation workbench.
The syntax-only [formatter](FORMATTER_2026.md) is permanent W3 tooling. It
preserves token spellings and comments, validates its output by re-lexing and
re-parsing, and provides stdout formatting and check-only commands. It does
not load imports, check types or change language semantics. The
[documentation generator](DOCUMENTATION_2026.md) emits standalone offline HTML
for written declarations and escaped source, with the same syntax-only scope.
It supplies no resolved-interface, ABI or claim documentation. The permanent
[local witness replay](WITNESS_REPLAY_2026.md) adds exact typed argument decoding
and bounded reference evaluation of a selected Boolean specification. A false
or true result concerns one witness; it supplies no authoritative claim,
solver selection, D-009 execution credit or proof identity. The Orange Book
and manuals exist. Tabula is not a language server, and its
run records are not proof-bearing product evidence. The current Typed Reference
Core has no canonical product encoding or cross-revision proof identity.
Typed `impl` bodies, product proof checking, verified lowering, native object
generation, and the package/evidence release lifecycle are not implemented.

## Complete scope and exit evidence

Every row remains required for complete 1.0. Exact host, target, algorithm,
and support tuples require the applicable owner decisions; choosing those
tuples does not weaken any advertised claim.

| Work boundary | Existing foundation | Required engineering and exit evidence |
| --- | --- | --- |
| Language and semantics | Parser, diagnostics, pure reference evaluation, fixed sequences, words, integers, modular arithmetic, finite instances, syntax-only formatting and source documentation | Version the complete reference and Core calculus; complete the selected specification, implementation, machine, game, and proof strata, algebraic data types, parameterized modules, contracts and LSP; mechanize advertised semantics, maintain frontend tools across supported syntax, and extend documentation to actual resolved interfaces, ABI contracts and claim matrices. |
| Claims and proof | Non-product decision laboratories, proposed schemas, and typed local reference witness replay | Canonical Core and Proof IR, stable identities, authoritative checker and implementation-diverse checker, interactive proofs and certificate automation, typed atomic claims, complete assumption/axiom/TCB closure, bounded offline replay, malformed-proof and forged-evidence rejection. |
| Implementation and compilation | Mathematical algorithms and reference evaluator | Typed implementations with terminating control flow and invariants; selected stable IRs, checked functional and leakage preservation at every advertised transition, reference C output with explicit assurance scope, native code and checked final object connection. |
| Memory, secrecy, targets, and ABI | Proposal and research envelopes | Regions, ownership, mutable buffers, initialization and zeroization obligations, public/secret effects, declassification, target features and vector intrinsics; selected leakage semantics, native instructions, ABI objects, generated C headers and safe Rust interfaces, adversarial caller tests. |
| Cryptographic corpus | Reference hash, symmetric, field, curve, and other algorithm fixtures | Owner-selected claim-complete symmetric, hash, field, elliptic-curve, and post-quantum workloads; exact standards/errata/rights provenance and clause maps, vectors and negative cases, interoperability, performance budgets, checked implementation claims; ACVP-compatible vector import/export. |
| Probability and security reductions | Informational OEP-0022 P7 plan | Exact finite probability and bounded-game semantics, adversary interfaces, ideal entropy assumptions separated from implementation contracts, replayable security-reduction evidence for advertised game claims. |
| Packages and evidence lifecycle | Repository source, locks for development builds, proposed product formats | Content-addressed packages, immutable locks and theorem/proof dependencies, local store and thick offline bundles, signed evidence identities, cache invalidation, update/deprecation/withdrawal/revocation, schema and compatibility migrations, offline inspection. |
| Stable release and maintenance | Pinned bootstrap, repository CI, development rebuild checks, solo governance and reporting policy | Exact immutable release scope, support dates, names and license boundary, declared dependency admissions, clean network-disabled builds, separately provisioned owner rebuilds, source/artifact inventory and SBOM/provenance, signatures under the selected policy, immutable tags/assets, exercised recovery and security response. |

The final verification record must address all ten charter criteria separately:
published versioned contracts; mechanized metatheory and compiler coverage;
authoritative offline claim replay; ABI-correct native objects and checked
artifact connection; corpus claim matrices and vectors; fail-closed negatives;
reproducible release evidence; resolved critical/high findings and current
threat model; clean documented developer journeys; and exercised owner
vulnerability, key, support, and deprecation operations. No area inherits
completion from another area's tests.

## Dependency-ordered engineering sequence

1. Complete proof-neutral S3/P1/P2 work and mathematical preparation for P4
   in the permanent frontend and corpus. S3t supplies modulus expressions over
   finite size instances, and S3u matrices of polynomials. P2 supplies five-limb reconstruction, abstraction,
   tight/loose/canonical predicates, addition, carrying and canonicalization.
   Partial P4 preparation supplies exact `Int` multiplication with three-pass
   normalization, biased subtraction, dedicated squaring and a24 multiplication
   observations. Machine limb multiplication and checked representation
   contracts remain later work: wrapping `Word[64]` multiplication cannot stand
   in for these exact accumulators.
   Review the existing pure slices and define remaining shapes and parameter
   domains without claiming universal proof from finite specialization.
2. Close the semantic/assurance foundation with actual decision evidence.
   Review D-004's recorded epochs. Implement and execute the symmetric D-005
   public-claim candidates and D-009 solver-trust candidates; their current
   laboratories have no real candidate-case execution. Complete D-006's owner
   tasks and dependency admissions. Accepted exact-revision decisions, not
   inferred approval, unlock their dependent S4 components.
3. Implement S4 around one permanent mathematical/implementation contract,
   then grow the same claim-bearing path to every advertised family. Establish
   canonical identity and offline proof replay first, then representation
   contracts and reusable arithmetic obligations. P3/P4's tight/loose/canonical
   invariants and X25519 refinement are the first concrete arithmetic path;
   test algebraically correct operations with invalid output bounds as negatives.
4. Execute D-010's symmetric compiler-strategy suite, select the exact output
   path, and implement S5/S6 preservation, memory, secrecy, leakage, target and
   ABI boundaries. A small checked vertical path is the engineering order for
   building the complete product; it is not the final 1.0 scope. Expand its
   contracts and checked final-object connection across the selected corpus and
   targets rather than creating a second demonstration compiler.
5. Complete S7 and OEP-0022 P6/P7: the full selected cryptographic corpus,
   quotient-ring/ML-KEM transform and construction obligations, probability and
   security reductions, standards provenance, interoperation and claimed
   performance/leakage evidence. Scalar coefficient rows or a quadratic-pair
   example alone do not establish full ML-KEM, NTT, or KEM conformance.
6. Complete S8 and release operations alongside the later compiler/corpus
   work: package locks and offline bundles, LSP and resolved-interface/ABI/claim documentation,
   evidence inspector, migrations, release/recovery tooling, and clean scripted
   journeys. Maintain the implemented formatter and source documentation through the remaining language
   work; retain the local witness decoder/replayer when integrating the selected
   proof path. Close every final release criterion before authorizing 1.0.

## Immediate foundational owner work

These are review recommendations, not decisions. AI can prepare documents,
commands, comparisons, and contributor rehearsals. It cannot record that the
owner performed a task or reviewed evidence that the owner has not reviewed.

### D-004 semantic strata

Epoch `d004-e-633e0aa831615cda3e06` contains all 105 scheduled v0.8
executions. ST-REL, ST-UNI, ST-DUAL, and ST-MIRROR each close seven cases;
ST-HOST closes none because required host authorities are unresolved.
The owner-chosen isolation-first rule leaves ST-REL: it ties ST-MIRROR at
zero isolation obligations and re-identifies six subject classes to seven.
The rule was chosen with the predicted candidate outcomes known. The
[laboratory record](../research/decisions/D-004/README.md) discloses this
and retains both v0.7 and v0.8 results as contributor-produced and unreviewed.

The engineering proposal for owner review is ST-REL under that recorded rule.
The contributor summary covers SS-G05 and the SS-G03 structure; it is not the
suite's completed D-004 recommendation or accepted product semantics.
Before accepting it, the owner must review the exact epoch and bound subject
bytes, dispose every candidate and every hard gate, examine the semantic
evolution and relabeling results, explain the distinguishing rule and its
known-outcome limitation, and record alternatives, unresolved obligations and
scope at an exact revision. If that review finds the rule or results
insufficient, retain an inconclusive decision and define new symmetric work;
do not rewrite the frozen runs. This review cannot be inferred from an
instruction to continue development or from an all-green merge.

### D-006 proof foundation

Epoch `d006-e-c7b6648ae3988234297f` contains DS-01 through DS-06 for
Rocq and Lean 4. Both corrected candidates pass HG-2 through HG-7; HG-1
and HG-8 remain unresolved because DS-07 and owner review are absent.
The [laboratory record](../research/decisions/D-006/README.md) gives the
exact run, correction, metrics, and trust inventories. Rocq has lower measured
bootstrap, most replay, memory, checker-size, and dependency costs; Lean's
DS-04 replay advantage uses declared `native_decide` compiler/runtime trust.
The conditional engineering recommendation for owner review is Rocq if DS-07,
the trust/dependency dispositions and every remaining hard gate pass. Its
measured resource and extraction costs suit the proposed small authoritative
checker path. This is not a completed suite recommendation or a selection:
the owner's maintenance evidence and per-axis rationale can change it.
The [recorded findings](../research/decisions/D-006/d006-v0.3/FINDINGS.md)
also expose corpus gaps: Rocq's LRAT line processing has quadratic work on
long lines, while Lean's line recursion requires large bounded stack settings.
Product checkers must handle their complete hostile-input resource envelope;
passing the research corpus alone does not close that engineering obligation.

DS-07's frozen [owner task packet](../research/decisions/D-006/d006-v0.3/shared-inputs/ds07-owner-tasks.json)
requires, for each candidate, two tasks with the same 120-minute ceiling:

- Audit from a clean workspace and published packet: locate `D2-TH04`, list
  its assumptions and trusted components, identify a failing mutation and
  diagnostic, rebuild the standalone checker and match its digest.
- Maintain the model: add wrapping subtraction to Sieve with addition's
  typing rule, repair every DS-02 proof, and add an observation.

The order is audit Rocq then Lean, maintenance Lean then Rocq. The owner must
detect and classify the sealed hidden-assumption, stale-artifact, ambiguous
diagnostic, and dependency-substitution faults; retain misses and timeouts;
record every command, source, suggestion and hint; and disclose familiarity
and role overlap. Contributor rehearsals do not satisfy M-16 or an owner
review scope. Do not reveal the sealed seeds to the owner before the tasks.
R-01 through R-09 reviews and D-018 tool admissions also remain open.
D-004 and D-005 must each be Accepted before D-006 can be Accepted.

### Remaining choices at their dependent boundaries

| Decision | Exact action before the dependent component or claim |
| --- | --- |
| D-005 and D-009 | Execute all 32 public-assurance and 24 solver-trust candidate-case units under their accepted protocol/epoch requirements; review every hard gate and alternative and accept an exact-revision OEP. Synthetic transports and identity inventories supply no execution credit. |
| D-007 | Select the product proof/certificate format and authoritative checking boundary after D-006, with mechanized soundness, canonical encoding, bounded checking, and malformed-proof evidence. |
| D-010 | D-003, D-004, D-005, D-006 and D-009 must be Accepted. Execute all 40 compiler-strategy units, including each distinct direct-native, Jasmin, C11, and LLVM path; dispose the comparative and hard-gate results before selected-path implementation claims. |
| D-011 through D-013 | Record supported host/native feature tuples, the exact first leakage model, and C/Rust foreign contracts; satisfy each selected tuple's preservation and adversarial tests. Research C kernels and emulation are not Orange native output or physical hardware evidence. |
| D-014 and D-015 | Record local package/lock/evidence formats and exact claim-complete corpus membership. A public package registry remains a separate choice; its operating requirements apply if selected. |
| D-017, D-018, D-020, D-022 and release authority | Record public naming and namespace boundary, exact license and dependency terms, achievable supply-chain/signing/recovery policy, actual support dates, immutable publication rules, and the release decision. Local development authorization is not redistribution authorization. |

## Applicable release constraints

Complete 1.0 retains all charter and journey requirements and the applicable
[`RELEASE_POLICY.md`](../RELEASE_POLICY.md) identity, build, publication,
and recovery gates. Unresolved naming and license decisions currently prohibit
crate, package-registry, and binary distribution. Source integration and local
owner development may continue. The eventual record must distinguish the
sealing format, language/Core/proof/evidence versions, compiler, cryptographic
profiles, targets, ABI, leakage, support, and exact artifact bytes.

D-023 makes outside contributors, independent reviewers/rebuilders, separate
release principals, and laboratories unavailable. Their absence does not
freeze engineering; owner results remain `solo-reviewed` or `solo-produced`
and never become independent evidence. A claim whose authority specifically
requires external validation stays unsupported or unresolved. Technical
soundness, replay, preservation, negative tests, reproducibility and release
operations remain required and cannot be waived by relabeling evidence.

FIPS validation, physical side-channel models beyond a selected leakage
profile, a public registry, additional future targets, and general self-hosting
are not automatic full-1.0 prerequisites. They apply only where the selected
product scope or advertised claim requires them. Mandatory game semantics,
the selected post-quantum corpus, the reference C path, the developer tools,
and the eight journeys remain part of the complete target.

## Clean-environment journey ledger

All eight journeys are structurally specified and none is recorded complete.
Each exit must be executable from published exact inputs without private
session state; each retains its own positive and fail-closed mutation cases.

| Journey | Required result and current dependency |
| --- | --- |
| J-01 install | Verified immutable installation identity, receipt, supported host, roots and self-checks; product release and installation lifecycle absent. |
| J-02 specify | Exact standards provenance, clause/errata/rights disposition, reference definitions and vectors; pure evaluation exists, complete admission lifecycle absent. |
| J-03 implement/prove | Canonical implementation contracts, authoritative replay and atomic mixed claim matrix; typed implementations and S4 absent. |
| J-04 build/inspect | Locked native build, checked lowering/final object, exact target/ABI/leakage and artifact claims; S5/S6 absent. |
| J-05 integrate | Generated C/Rust interfaces with tested caller contracts and exact evidence; native ABI path absent. |
| J-06 offline replay | Thick bundle replay and complete claim/TCB inspection with hostile-input rejection; product package/checker path absent. |
| J-07 update/revoke | Authorized immutable replacement, compatibility/evidence checks and fail-closed active-state transitions; product lifecycle absent. |
| J-08 respond/recover | Exercise invalidated claims, key compromise, affected graph closure, rebuilt evidence, withdrawal and recovery; release-specific drills absent. |

The current tranche supplies permanent language and mathematical-representation
engineering for that path. Its tests, policy checks, push and merge do not
record a journey complete or declare Orange 1.0 finished.
