# Decision register

Status: active proposed-decision ledger

Snapshot: 2026-07-26

This file separates user direction, research recommendations, and ratified
architecture. A recommendation is not allowed to become a hidden decision by
being implemented first.

Statuses:

- `accepted`: ratified and change-controlled;
- `owner-accepted`: explicit owner disposition provisionally authorizes the
  boundary while its required exact-revision Accepted record remains pending;
- `directed`: explicit project direction, with design details still open;
- `proposed`: recommended answer pending the named gate;
- `investigate`: alternatives need a reproducible decision suite;
- `blocked`: an external fact or authority is required;
- `superseded`: retained for history with a replacement link.

Under D-023, remaining `Gate 0` phrases are legacy decision-stage labels, not
an aggregate implementation barrier. Each unresolved decision gates only the
component or claim that depends on it.

## D-001 — Mission

Status: directed

Source: original repository README

Directed decision: Orange is a language for specifying, implementing, and
verifying cryptography.

Working product interpretation for incremental capability decisions: the
deliverable includes the language, proof system, compiler, developer tools,
package/evidence format, standard cryptography corpus, and operational release
system needed to make that sentence true.

Open detail: incremental capability decisions must keep the support envelope
finite. D-023 supersedes the former requirement to freeze all of 1.0 before any
implementation begins.

## D-002 — No disposable prototype

Status: directed

Source: explicit user direction

Decision: build the end product through permanent, production-lineage
components. There is no prototype-to-rewrite phase and no MVP that postpones
the core assurance claim.

Interpretation:

- incremental integration is required;
- early components have final boundaries, tests, determinism, diagnostics, and
  documentation;
- early algorithms become permanent conformance fixtures;
- research decision cases do not become an unreviewed parallel implementation;
- incomplete pre-1.0 work is not marketed as production-ready.

Change rule: only explicit user/project-governance direction can supersede this.

## D-003 — Product form

Status: accepted on 2026-07-26 at exact decision revision
`a82a5cec2ee4359dc2fe66171f17c93146747333`

Accepted direction: a standalone domain-specific language with its own
editioned semantics and canonical Core formats.

Alternatives considered:

- manifest-only orchestration over Cryptol/hacspec/Jasmin/EasyCrypt;
- embedded DSL in F*, Lean, or Rocq;
- Rust subset with proof annotations.

Rationale: interoperability with those systems is valuable, but delegating the
surface language and semantics would preserve the polyglot seams Orange is meant
to make explicit.

Acceptance evidence: four candidates, eight non-compensable hard gates, 8/8
structurally specified journey mappings with 0/8 complete, an explicit
private-to-canonical Core migration boundary, and an owner-executable scope and
resource analysis. This decision evidence establishes neither completed
journeys nor a stable canonical Core. Independent feasibility review is
unavailable in solo mode; its absence limits any external feasibility claim but
does not block proof-neutral frontend work.

The four candidates, eight hard gates, journey coverage, migration boundary,
resource analysis, and reconsideration rules are specified in the
[D-003 product-form decision packet](PRODUCT_FORM_DECISION_PACKET.md). The owner
explicitly accepted candidate PF-01, the standalone Orange product form, on
2026-07-26. Under [`GOVERNANCE.md`](../GOVERNANCE.md), Accepted
[OEP-0004](governance/oeps/OEP-0004-standalone-orange-product-form.md)
formally ratifies that boundary and binds the fully validated merged decision
revision `a82a5cec2ee4359dc2fe66171f17c93146747333`. This acceptance does not
accept D-004 or authorize S3b.

## D-004 — Semantic strata

Status: proposed; decide before S3 stabilizes the semantic strata and Core

Recommendation: one module system with separate Specification, Implementation,
Machine Implementation, Game, and Proof strata, lowering to several formally
related IRs.

Alternative under comparison: one universal IR. The research concern is that
mathematical totality, probabilistic games, stateful memory, target leakage, and
concrete instructions have conflicting requirements and that hiding them in
annotations could make the semantics less honest. The alternative is not
rejected by this proposed register entry; it must run the same symmetric suite
as every other candidate.

Acceptance evidence: representative permanent decision cases for SHA-like word
code, mutable buffers, a secret-dependent rejection case, one vector intrinsic,
and one game/reduction relation.

The symmetric candidates, typed relationship graph, five cases,
non-compensable gates, resource contract, and inconclusive procedure are
specified in the
[D-004 semantic-strata decision suite](SEMANTIC_STRATA_DECISION_SUITE.md).
That Draft protocol records 0/25 required candidate-case executions: 0/5
candidates have complete five-case packets, and 0/5 cases have complete
cross-candidate execution. Its exact packet, catalogs, and bound suite-document
bytes are the historical D004-PRE-01 review subjects and remain unchanged.

The v0.5 laboratory byte-materializes exactly 73 candidate-neutral suite-only
subjects under its historical `draft_unreviewed_input_only` status: five
positive cases, all 26 named mutations, 14 missing-edge subjects, 13 identity-
substitution subjects, and five each for ambiguity, unsupported behavior, and
domain exhaustion. On 2026-07-26 the owner accepted D004-PRE-01 as
`solo-reviewed` at exact review-subject revision
`7d09a27369649855ce987c76315271b0d34a20ef`. The successor
`d004-v0.6-reviewed-protocol` finds the five fixture classes reviewed and
sufficient only for bounded suite coverage: 5 ambiguity, 14 missing-edge, 13
identity-substitution, 5 unsupported, and 5 resource-exhaustion. It reviews all
five candidate graphs and 70 SR mappings only as symmetric, falsifiable test
hypotheses; none is accepted Orange semantics or establishes capability.

That acceptance covers the immutable review subjects, not a revision that
already contained the successor overlay. The v0.6 implementation closure stays
`provisional_pending_exact_merged_revision` until the validated bytes are
available at an exact merged revision.

The reviewed replay plan assigns exactly three deterministic repetitions to
each of the 25 candidate-case units, for 75 planned executions. Every execution
requires a fresh empty candidate-specific cache and equality of the specified
deterministic fields. The plan remains uninstantiated: adapters, executable
manifests, enforcing isolation, result parsers, an exact execution-subject
revision, and a separate owner freeze record are absent. The D-004 epoch is null
and unfrozen, execution is unauthorized, evidence remains zero completed of 25
required units and 0 of 75 result records, and selection and conclusion remain
null. Integrity parsing and structural oracles ratify no Orange semantics.
D-004 remains proposed, S3b remains blocked, and Orange's 3-of-10 (30%) binary
gate-closure score remains unchanged; that mechanical score is not release
readiness.

## D-005 — Public assurance model

Status: proposed; decide before S4 stabilizes the claim model and public claim
records

Candidates: orthogonal typed claim records, named assurance profiles backed by
atomic claims, evidence-graph-derived claim views, and an aggregate package
assurance level or `verified` Boolean. No candidate is selected, preferred, or
authorized for product use by this register.

Every candidate must preserve the same ten separately queryable claim families:
conformance, functional refinement, safety, termination, leakage, compiler
preservation, ABI, erasure, game-based security, and empirical tests. Each
atomic claim has exactly one of `satisfied`, `not_satisfied`, `unresolved`, or
`unsupported`; no profile, graph view, package level, Boolean, neighboring
claim, or optional evidence may upgrade an atomic non-success.

The symmetric candidates, eight adversarial cases, authority and trust rules,
32-run matrix, historical-schema boundary, hard gates, archive, and
inconclusive procedure are specified in the
[D-005 public-assurance-model decision suite](PUBLIC_ASSURANCE_MODEL_DECISION_SUITE.md).
Current execution evidence is 0/32 candidate-case executions. The provisional
Gate 0 claim schema and synthetic fixtures are historical shape inputs, not a
selected candidate or public format.

The candidate-neutral laboratory also prepares canonical adapter requests and
strictly validates synthetic captured response envelopes. It launches no
process, validates no candidate payload semantics, creates no result, and
retains the exact 0/32 baseline. Real execution remains gated on an owner-frozen
epoch and an approved resource-isolation backend.

That draft boundary now enumerates exactly 192 in-memory transport identities
from the 32 candidate-case base slots, two workspace identities, and three
render identities. Its order is a canonical identity serialization only, not a
physical run order. It also binds synthetic capture receipts to the exact slot,
request digest, termination, truncation flags, and raw stream lengths and
digests. Every receipt keeps isolation `not_evaluated`, payload `unvalidated`,
and evidence `none`; exact inventory checks create no execution credit and do
not compare opaque payloads across repetitions.

D-005 closes only through an Accepted exact-revision OEP after the complete
owner-executable suite passes. Multidisciplinary external review remains
`unavailable` in solo mode; any claim whose policy requires it remains
unsupported or unresolved as specified, without blocking unrelated work. Owner
review is labeled `solo-reviewed` and never becomes independent or technical
evidence merely through repetition.

## D-006 — Proof foundation

Status: investigate; required before proof-bearing components, not the frontend

Dependency order: D-004 and D-005 must each be Accepted before D-006 can be
Accepted.

Candidates: Rocq and Lean 4. Neither candidate is selected, preferred, or
authorized for product use by this register. Rocq's verified-compiler,
cryptography, and extraction ecosystem and Lean 4's integrated implementation,
kernel, and tooling model are candidate-specific hypotheses to measure under the
same frozen suite, not reasons to preselect a winner.

Required decision suite:

- define/check the proposed Core fragment;
- mechanize progress/preservation and a leakage lemma;
- implement canonical serialization validation;
- produce and replay an LRAT-backed bitvector proof;
- exercise extraction/distribution on all supported hosts;
- measure clean bootstrap, proof replay, diagnostics, binary size, and long-term
  dependency surface;
- exercise published-packet auditability and the same seeded owner maintenance
  tasks for both candidates; and
- record independent review and external-audit evidence as `unavailable` in
  solo mode, disclosure that cannot distinguish the candidates.

The symmetric cases, measurements, hard gates, archive, and inconclusive
procedure are specified in the
[D-006 proof-foundation decision suite](PROOF_FOUNDATION_DECISION_SUITE.md).

The decision is evidence-based. The owner-executable `d006-v0.2-draft` defines
14 candidate-case runs, level-2 same-owner replay, symmetric hard gates, and
exact `solo-reviewed` OEP acceptance without pretending that the owner is an
independent reviewer. Current execution evidence is 0/14 candidate-case runs.
D-023 permits proof-neutral compiler work while this remains open. No source or
Core choice may make a proof foundation irreversible before the owner completes
the suite and accepts the selected foundation through an exact-revision OEP.
The draft admits no proof toolchain dependency and authorizes no proof-bearing
implementation.

The input-only pre-epoch laboratory binds the unchanged suite and a seven-row
case-input index. Each DS-01 through DS-07 row retains zero executable fixtures,
unresolved coverage, and an active freeze blocker. Its deterministic 14-pair
identity inventory is not a physical run order. The laboratory installs and
executes no prover, freezes no epoch, creates no result or evidence, performs no
owner review, and retains the exact 0/14 baseline and null selection.

## D-007 — Orange-owned proof format and checker

Status: proposed; depends on D-006

Recommendation: define a small Orange Proof IR and kernel. Formalize the checker
in the selected proof foundation and distribute an authoritative extracted
checker, plus an implementation-diverse safe-Rust checker for differential
validation.

Alternative: make the host prover’s compiled environment the permanent public
artifact.

Rationale: Orange needs stable theorem fingerprints, offline inspection, bounded
checking, and evidence bundles independent of a host prover’s internal file
format and release cadence.

Risk: a custom kernel is a major soundness and schedule risk. The logic must be
smaller than the surface language and explicitly exclude convenient features
that would enlarge the TCB.

Acceptance evidence: mechanized soundness, two checkers, malformed/mutation
fuzzing, external logic audit, stable canonical encoding.

## D-008 — Implementation languages

Status: directed for the solo compiler bootstrap

Directed decision:

- safe Rust, Rust edition 2024, for the permanent driver, frontend services,
  package tooling, LSP, and implementation-diverse checker;
- selected proof foundation for normative semantics, authoritative checker, and
  verified stable compiler passes;
- no mandatory self-hosting target.

The initial slice pins the Rust toolchain and uses only the standard library.
New crates require an explicit dependency admission record. The words
`independent checker` remain reserved for organizational independence; a second
checker written by the owner is an implementation-diverse checker.

Rationale: Rust is suitable for hostile-input tooling and distribution, while
the checker/passes need a direct mechanized relationship. Orange is too
specialized to gain from adding general application features merely to
self-host its package manager or LSP.

This decision authorizes the proof-neutral compiler foundation in D-024. The
proof foundation and cross-language canonical boundary remain open and gate
only components that depend on them.

## D-009 — Solver trust

Status: proposed; decide before S4 admits solver-backed proof search

Dependency order: D-004 and D-005 must each be Accepted before D-009 can be
Accepted. D-006 and D-007 are downstream consumers rather than D-009
acceptance prerequisites. Requiring either first would create a cycle because
D-006 case DS-04 itself depends on the D-009 solver-trust policy.

Decision question: whether claim-closing automation requires a checked
artifact, requires reconstruction as an Orange proof term, or may rely on an
exact admitted solver as direct logical authority.

Candidates:

- SP-01, Checked-artifact portfolio: untrusted search with claim-closing
  authority only after an accepted certificate or Orange proof term;
- SP-02, Kernel-only reconstruction: solver output may guide reconstruction,
  but only a kernel-accepted Orange proof term can satisfy a claim; and
- SP-03, Direct trusted-solver authority: an exact admitted
  solver/version/fragment may decide directly and is explicitly part of the
  logical TCB.

No candidate is selected, preferred, or authorized for claim-bearing product
work by this register. The owner-executable
[`d009-v0.1-draft`](SOLVER_TRUST_DECISION_SUITE.md) compares all three policies
across eight symmetric cases: profile/outcome separation, LRAT-family checked
artifacts, validated counterexamples, reflective procedures, supported SMT
proof formats and external evidence, fail-closed automation outcomes, complete
identity/cache/TCB closure, and solo-mode replay and maintenance. The frozen
matrix contains 24 candidate-case runs and no weighted aggregate score.

Candidate-neutral comparison inputs include:

- verified bit-blasting plus LRAT-family SAT certificates;
- reflective algebra/range procedures;
- an explicitly ratified proof format such as Alethe for supported SMT
  fragments only;
- external EasyCrypt/SSProve evidence labeled as external until reconstructed.

The four D-005 atomic claim outcomes retain their exact-claim-and-scope meanings
throughout the comparison. `satisfied` requires complete candidate-permitted
mandatory authority closure for the exact proposition and no valid decisive
negative result. `not_satisfied` requires permitted, identity-bound negative
evidence that establishes the exact proposition false or violated within its
scope; absence or incompleteness alone is not `not_satisfied`. `unresolved`
means a well-formed, supported claim still has an unknown,
incomplete, conflicting, or exhausted required decision; timeout, `unknown`,
missing proof output, resource exhaustion, crash, malformed output, checker
failure, or an unsupported supplied proof step remains `unresolved` when the
candidate still offers a permitted authority path for that claim and scope.
`unsupported` means the candidate's declared policy or support envelope offers
no permitted evaluation or authority path for the exact claim and scope. One
unsupported artifact format, rule, or certificate step does not by itself make
the claim `unsupported`. Every non-success records its exact diagnostic reason.
No profile, cache, or summary may collapse or upgrade these outcomes.

Acceptance evidence must prove every failure mode fails closed and every
claim-closing authority is represented honestly. SP-01 and SP-02 must show that
no solver executable has undeclared logical authority; SP-03 must expose the
exact direct-authority solver in the logical TCB and may never present its
direct result as a checked certificate or kernel proof. These are symmetric
candidate obligations, not a recommendation.

Current execution evidence is 0/24 candidate-case runs. The input-only
pre-epoch laboratory binds the unchanged suite and an eight-row case-input
index whose TC-01 through TC-08 rows retain absent shared inputs and candidate
mappings, zero executable fixtures, unresolved coverage, and active freeze
blockers. It enumerates the exact case-major, candidate-minor 24-slot identity
inventory in memory but assigns no physical execution order.

The laboratory admits, acquires, installs, or executes no solver, proof
assistant, certificate checker, adapter, runner, observer, or isolation
backend. It validates no proof, certificate, counterexample, theorem, claim, or
cache result; freezes no epoch; creates no result or evidence; and leaves
resources, selection, and conclusion null or unassigned. It adds no solver to
or removes one from the logical TCB, authorizes no proof-bearing implementation,
and changes no roadmap gate or the 3/10 (30%) binary gate-closure score.

D-009 closes only after D-004 and D-005 are Accepted, all 24 candidate-case
records are complete under one frozen symmetric epoch, SR-01 through SR-08 are
complete and `solo-reviewed`, and the deterministic decision procedure yields
one `recommend_*` conclusion naming a candidate whose eight hard gates all
`pass`. `tie` or `inconclusive` leaves D-009 open. An Accepted Orange
Enhancement Proposal must then bind that policy to the exact fully validated
Git revision. Its `decision-revision` is exactly 40 lowercase hexadecimal
characters, its review authority is `Orange Project Owner`, and an approval
record contains the literal `solo-reviewed`. Owner acceptance is a governance
disposition, never independent review or technical proof. An accepted D-009
policy constrains later D-006 and D-007 work but does not supply their
foundation, Proof IR, checker, or implementation evidence.

## D-010 — Compiler strategy

Status: investigate; decide through one frozen symmetric comparison before S5
selects an output path or any compiler boundary carries a preservation claim

Acceptance prerequisites: D-003, D-004, D-005, D-006, and D-009 must each be
Accepted before D-010 can be Accepted. D-007, D-011, D-012, and D-013 are
downstream decisions, not D-010 acceptance prerequisites. The suite must model
their proof-format, target, leakage, and ABI boundaries without accepting or
implementing them. Any external compiler, proof tool, target tool, or runtime
must receive the applicable D-018 admission before measured execution.

Decision question: which bounded compiler strategy gives Orange the strongest
owner-executable and accurately scoped preservation path without laundering an
external compiler, shrinking a promised native frontier, or treating an
unverified last mile as proved?

The five candidates are symmetric:

- CP-01 — theorem/certificate hybrid direct-native path;
- CP-02 — mechanized proof-per-pass direct-native path;
- CP-03 — versioned Jasmin backend boundary;
- CP-04 — portable C11 interoperability boundary; and
- CP-05 — versioned LLVM IR interoperability boundary.

CP-04 and CP-05 are separate candidates. A C source boundary and an LLVM IR
boundary have different semantics, toolchains, undefined-behavior surfaces,
target assumptions, replay identities, and downstream claim frontiers; results
from either may not be reused for the other.

The
[D-010 compiler-strategy decision suite](COMPILER_STRATEGY_DECISION_SUITE.md)
compares every candidate across the same eight cases:

- CC-01 — freeze the pipeline, authorities, and claim frontier;
- CC-02 — preserve functional semantics through structural lowering;
- CC-03 — handle optimization, scheduling, vectorization, and allocation;
- CC-04 — preserve one named leakage model;
- CC-05 — exercise the endpoint and, when claimed, the final-object last mile;
- CC-06 — fail closed under corruption, substitution, failure, and fallback;
- CC-07 — bind replay identities and compare against the reference semantics; and
- CC-08 — measure solo auditability, maintenance, and resource use.

The suite defines M-01 through M-19, AX-01 through AX-09, eight non-compensable
hard gates, CR-01 through CR-11, the four atomic claim outcomes within each
candidate-case record, and one
deterministic total conclusion procedure. A claim-level
`unsupported` result beyond an intentionally frozen claim frontier can be
correct when the candidate makes no claim beyond that boundary. It is not a gate
waiver. A direct-native candidate may not shrink its promised native frontier to
relabel missing final-byte evidence as unsupported.

Current execution evidence is 0/40 candidate-case runs. The input-only
pre-epoch laboratory binds its candidate-neutral draft packet, unchanged suite,
and eight zero-fixture case blockers, then enumerates the exact case-major,
candidate-minor 40-slot identity inventory in memory. It admits, acquires,
installs, or executes no compiler, proof tool, assembler, linker, target runtime,
adapter, runner, observer, or isolation backend; assigns no physical execution
order or resource contract; produces no IR, source output, object, certificate,
result, replay record, evidence, selection, or conclusion; and validates no
functional, leakage, target, ABI, or final-byte claim. It selects no output path,
closes no roadmap stage, and does not change the 3/10 (30%) binary gate-closure
score.

D-010 closes only after all five prerequisites are Accepted, all 40
candidate-case records are complete under one frozen symmetric epoch, CR-01
through CR-11 are complete and `solo-reviewed`, and the deterministic procedure
yields exactly one `recommend_*` conclusion naming a candidate whose eight hard
gates all `pass`. `tie` or `inconclusive` leaves D-010 open. An Accepted Orange
Enhancement Proposal must then bind the selected strategy and exact claim
frontier to the fully validated Git revision. Owner acceptance is a governance
disposition, never independent review, compiler correctness proof, leakage
proof, or implementation evidence.

## D-011 — Initial native target envelope

Status: proposed; decide before target implementation or native-code claims

Recommendation:

- initial assurance target tuples: x86-64 Linux/SysV and AArch64 Linux/AAPCS64;
- explicit baseline and selected crypto/SIMD feature profiles;
- host tools for current Linux, macOS, and Windows;
- stable generated C ABI and Rust wrapper;
- portable C output clearly labeled as an interoperability path.

Deferred unless an incremental target decision substitutes them: a RISC-V
assurance target, claim-bearing Windows and macOS native outputs, general Wasm
constant-time claims, GPUs, and hardware synthesis.

Rationale: x86-64 and AArch64 cover the principal server and client CPU families
while keeping target verification finite. Solo capacity and available target
model evidence may require the owner to choose only one for 1.0 rather than
weaken both.

Acceptance evidence: resource estimate per target, ISA/ABI model availability,
owner-accessible hardware evidence, and flagship-corpus feasibility.

The active solo capacity boundary that governs future target admission is
recorded in the [solo development envelope](GATE0_SUPPORT_ENVELOPES.md).

## D-012 — Baseline leakage claim

Status: investigate; decide before S6 stabilizes leakage semantics or makes any
constant-time claim

Recommendation: two-run architectural noninterference covering branches,
addresses/widths, indirect targets, traps, termination, and target-classified
variable-latency operations, with explicit declassification.

Separate future profiles cover speculative execution, architectural DIT modes,
power/EM, masking, and fault resistance.

Acceptance evidence: formal trace semantics, target instruction-classification
process, positive and negative examples, preservation plan through final bytes,
and an explicit review status. Independent side-channel review is unavailable
in solo mode, so claims that require it remain unsupported; this does not block
earlier proof-neutral compiler capabilities.

## D-013 — Stable foreign boundary

Status: proposed; finalize before S6 implements a foreign boundary or makes ABI
claims

Recommendation: generated C ABI plus a machine-readable contract, with safe
Rust wrappers above it.

The contract includes lengths, alignment, overlap, mutability, initialization,
layout, failure, target features, ownership, zeroization, and entropy behavior.
No hidden allocator, panic, exception, TLS state, or RNG exists in the kernel
boundary.

Acceptance evidence: ABI model and adversarial callers for each supported target
tuple; generated header/wrapper/object all derive from one definition.

## D-014 — Package and registry model

Status: proposed; decide the local format before S8 package tooling; decide any
public registry only through an explicit future release/distribution decision

Recommendation:

- human manifest plus immutable generated lock;
- proofs bind to exact content and theorem fingerprints;
- published versions immutable, with yanking affecting new resolution only;
- claim-bearing graphs forbid arbitrary native build scripts/plugins;
- offline resolution/replay from a thick bundle or populated local
  content-addressed store is mandatory; a lockfile alone is insufficient;
- public registry uses TUF-style delegated/threshold metadata, MFA, recovery,
  namespace governance, quarantine, and revocation.

Acceptance evidence: dependency-confusion, takeover, rollback, freeze, yanking,
offline, and compromised-key exercises.

## D-015 — Flagship 1.0 corpus

Status: proposed set; decide exact membership before S7 admits the corpus

Recommended coverage set:

- SHA-256/512;
- ChaCha20-Poly1305;
- HMAC/HKDF;
- AES-GCM with portable and selected accelerated variants;
- X25519 and/or Ed25519;
- ML-KEM;
- one selected post-quantum signature family if resources allow.

Rationale: each family exercises a different permanent language/compiler/proof
capability. Breadth is subordinate to a complete claim matrix.

Scope rule: under-resourcing removes a family or target rather than removing the
proof, leakage, binary, interop, or response gates while retaining the claim.

Acceptance evidence: claim matrix, standards/errata/vector sources, formal and
compiler workload estimate, target benchmarks, owner-executable comparisons
against mature reference implementations, and an explicit solo ownership and
resource record for each family. Independent human review and separate
maintainer ownership are unavailable in solo mode; their absence limits the
admissible claims rather than unrelated compiler development.

The active solo capacity boundary that governs future corpus admission is
recorded in the [solo development envelope](GATE0_SUPPORT_ENVELOPES.md).

## D-016 — Validation and certification posture

Status: proposed; external-lab requirement depends on product goals

Decision recommendation:

- support NIST ACVP-compatible input/output and record validation status;
- never call local vectors or proof replay an ACVP/CAVP certificate;
- never call Orange itself FIPS 140 validated;
- keep certificate-bearing profiles unsupported in the current solo operating
  model; only a future explicit operating-model change with an actually
  available accredited laboratory may open such a profile decision.

Acceptance evidence: lab scope and budget, module boundary, change/revalidation
strategy, and approved public wording.

Those acceptance items apply only to a future certificate-bearing profile.
Their current unavailability limits certification claims and does not block
development of non-certificate capabilities.

## D-017 — Project and package name

Status: directed working codename for solo development; public naming remains open

Current state: **Orange** is the working codename and repository name. The
Bootstrap Steward designated the byte-preserved images under
[`assets/brand/`](../assets/brand/) as the official working repository emblem,
wordmark, and lockup on 2026-07-11. That designation records current project
identity; it is not trademark clearance or ratification of the final name.

Evidence of collision:

- long-running Orange data-mining/visual-programming software;
- earlier `orange-lang/orange` systems language;
- broad commercial use of “Orange.”

Unavailable review inputs:

- professional trademark/legal search in intended jurisdictions and classes;
- command, package, domain, organization, documentation, and social namespace
  availability;
- searchability and confusion analysis;
- codename-to-final-name migration cost.

The owner directs use of **Orange** and `orangec` for repository-local solo
development. This is not trademark clearance and does not authorize package,
domain, or registry publication. Preserve the admitted originals and their
provenance so a later naming decision can migrate or retire them deliberately.

## D-018 — Licenses

Status: directed solo-development boundary; outbound license remains open

Working recommendation for review:

- permissive compiler/toolchain license, likely Apache-2.0 with patent terms or
  dual Apache-2.0/MIT;
- an explicit generated-output exception/statement so Orange does not impose a
  license on user artifacts;
- documentation and language specification license that permits independent
  implementations and quotation;
- vector/standards provenance preserved according to source terms;
- contribution terms compatible with future neutral governance and patent
  defense.

No repository-wide license or contribution grant is selected. The owner may
author and run Orange code in this repository, and that unresolved outbound
license does not block owner-authored implementation. Third-party contributions,
crate publication, binary distribution, and redistribution claims remain
blocked until the owner records appropriate terms. Dependencies require an
owner admission record; the initial compiler uses no third-party Rust crates.

## D-019 — Governance and release authority

Status: directed solo-project governance

Decision: `@chasebryan` is the sole project, implementation, review, merge,
security, and decision authority until explicit owner direction changes the
model. Plans must assume no contributors, independent reviewers, auditors,
laboratories, partner organizations, or separate operational roles.

Owner approval is valid governance disposition but is never independent
evidence. Missing separation of duties, bus factor, external review, and
multi-party custody are disclosed limitations rather than development blockers.
See D-023, OEP-0001, and `GOVERNANCE.md`.

## D-020 — Supply-chain target

Status: proposed; versions pinned at each release

Recommendation:

- NIST SSDF 1.1 baseline until a newer final edition is ratified;
- SLSA 1.2 Source L4 and Build L3 for release source/artifacts;
- current OpenSSF OSPS Baseline Level 3;
- additional Orange requirement for network-disabled, fully declared,
  reproducible release builds;
- Sigstore or equivalent signature/transparency evidence plus TUF-style update
  recovery;
- SPDX SBOM and CycloneDX SBOM/CBOM.

Acceptance evidence: continuous policy checks and successful independent build,
rollback, freeze, compromise, revocation, and disaster-recovery drills.

## D-021 — Self-hosting

Status: proposed

Recommendation: self-host only components for which Orange is naturally suited.
Keep the safe-Rust bootstrap/frontend and authoritative formal checker path
published and supported. Do not add general-purpose language features solely to
self-host networking, registry, or editor code.

If a compiler core becomes self-hosted, require reproducible diverse double
compilation or equivalent evidence and retain the prior bootstrap until
independent audits/rebuilds pass.

## D-022 — Support policy

Status: directed best-effort solo support until a release decision

Decision: pre-alpha solo development has no SLA, LTS window, compatibility
promise, or migration-service promise. Support is best effort by the owner.
A release-specific support window may be adopted only when the owner can
actually sustain it; the former five-plus-two-year institutional target is not
an active commitment.

Security-driven withdrawal may be immediate. Every release, if one is later
authorized, must state its actual support dates and single-maintainer risk.

## D-023 — Solo project operating model

Status: directed

Source: explicit project-owner direction on 2026-07-12

Decision: Orange is developed as a solo project until the owner explicitly
records otherwise. All current and future planning must treat outside human or
organizational participation as unavailable. No milestone may depend on
contributors, independent reviewers, auditors, laboratories, partner
organizations, or separate release and incident-response roles.

The former aggregate Gate 0 implementation embargo is superseded. Work proceeds
through incremental capability gates. An unresolved decision blocks only the
component or claim that relies on it. The absence of independent or external
evidence must be reported honestly, but it does not block unrelated work.

This decision does not convert owner review into independent review, waive a
technical proof obligation, grant certification, select a license, or authorize
a release. If participation later becomes real, the owner may amend the model;
earlier evidence remains labeled solo-produced.

## D-024 — Initial compiler foundation

Status: directed

Source: explicit project-owner direction to begin compiler work on 2026-07-12

Decision: begin the permanent Orange compiler lineage in Rust. The first bounded
slice contains source identity and byte spans, deterministic UTF-8 lexing,
structured stable diagnostics, and the `orangec` command-line boundary. It pins
the Rust toolchain, uses Rust edition 2024, and admits no third-party crates.

This slice may reserve clearly documented tokens but does not ratify the full
grammar, Core semantics, proof foundation, target model, ABI, or leakage model.
It performs no native code generation and carries no proof, cryptographic,
constant-time, compatibility, support, or production-readiness claim.

Acceptance evidence for the slice is deterministic formatting and linting,
unit and CLI tests covering positive and malformed input, stable diagnostics,
an exact source inventory, and green repository policy checks. Later slices add
their own decisions and do not inherit claims from this one.

## D-025 — Orange 2026 minimal grammar and bounded parser

Status: directed

Source: explicit project-owner direction for S2 on 2026-07-12; accepted
OEP-0002 at exact revision `52a3460853636f7cbaa27f3e27d86e032e3c82d4`

Decision: define the first Orange 2026 syntax as valid UTF-8 of at most 16 MiB,
with ASCII whitespace and identifiers, a mandatory exact `edition 2026;`
declaration, exactly one named module, and zero or more empty `spec` or `impl`
function declarations. The complete grammar is:

```text
source_file   = edition_decl module_decl EOF ;
edition_decl  = "edition" "2026" ";" ;
module_decl   = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl = function_kind IDENTIFIER "(" ")" empty_body ;
function_kind = "spec" | "impl" ;
empty_body    = "{" "}" ;
```

Line feed, carriage-return line feed, and bare carriage return each form one
logical line ending. `edition` is reserved with the existing Orange 2026
keywords. `game`, `proof`, and `claim` remain lexical reservations only.
Duplicate member names are syntactically valid because name resolution is not
part of parsing.

The parser is deterministic and bounded by exact token, syntax-node, event,
diagnostic, and recovery-depth limits in
[`LANGUAGE_2026.md`](LANGUAGE_2026.md). Lexically invalid input is not parsed;
recovery may improve diagnostics but never converts a malformed source into
success.

This slice explicitly does not define parameters, types, expressions, non-empty
bodies, imports, multiple modules, semantics, proofs, targets, ABI, leakage,
code generation, packages, or releases. Syntactic acceptance makes no claim
about any of them and does not settle D-003 through D-006 or D-009 through
D-016.

Acceptance evidence is the normative lexical and grammar document, exact source
inventory, positive and malformed parser tests, ambiguity and duplicate-name
cases, Unicode and line-ending cases, resource-limit tests, stable diagnostics,
repeatability, offline locked Rust checks, repository policy checks, and green
required hosted CI. Acceptance required the exact merged S2 revision; that
condition closed on 2026-07-12. OEP-0002 is Accepted at
exact merged revision `52a3460853636f7cbaa27f3e27d86e032e3c82d4` after its
required hosted checks passed.

## D-026 — Orange 2026 typed literal specifications

Status: directed

Source: explicit project-owner direction for S3a on 2026-07-12; accepted
OEP-0003 at exact merged revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`

Decision: preserve the accepted empty `spec` and `impl` function syntax and add
one typed form for specifications only:

```text
typed_spec     = "spec" IDENTIFIER "(" ")" "->" parsed_type
                 "{" signed_integer "}" ;
parsed_type    = IDENTIFIER ("[" INTEGER "]")? ;
signed_integer = "-"? INTEGER ;
```

Semantic acceptance recognizes exactly contextual `Int` without a width and
exact `Word[8]`. `Int` denotes mathematical signed integers. `Word[8]` denotes
unsigned values from 0 through 255; a minus sign or out-of-range literal is an
error, with no wrapping or coercion. Any other parsed type is a semantic error.
Integer magnitude input is limited to 16,384 significant bits without making
`Int` a finite-width type.

Function names are unique per `(spec | impl, exact name)` namespace. A `spec`
and `impl` may share a name. Empty functions participate in duplicate checking
but acquire no type, value, or execution meaning.

Successful analysis constructs a Typed Reference Core containing typed
specifications only, with contiguous source-order IDs, normalized types, and
literal values. The Core has no canonical encoding, proof identity, refinement
relation, implementation semantics, or cross-revision ID promise. `orangec eval
FILE` prints all Core values in source order as
`module::name: Type = value`, using decimal `Int` and two-digit lowercase
hexadecimal `Word[8]`; an empty Core prints nothing.

The semantic boundary is limited to 100 ordinary diagnostics plus one
suppression diagnostic, 262,144 Core nodes, 1,048,576 semantic events, and
1,048,576 evaluation steps. Exhaustion fails closed. Parameters, operators,
calls, bindings, control flow, dynamic failure, typed implementations, proofs,
code generation, ABI, leakage, packages, releases, and cryptographic claims are
outside this slice.

The normative grammar delta and semantic rules are in
[`SEMANTICS_2026.md`](SEMANTICS_2026.md). This bounded direction does not accept
D-003 or D-004, select the complete semantic strata, or authorize proof or
native-code claims.

Acceptance evidence includes exact positive, negative, boundary, resource, and
repeatability conformance coverage; stable diagnostics and output; 89 passing
Rust tests, including the documentation test; 95 passing Python policy tests;
and policy version 0.2.3 reporting zero findings. PR #9 head
`8c48a85997b756cf65d64110ebc869bb26e49079` passed Required CI run
`29215790064`, Dependency Review run `29215790110`, and CodeQL run
`29215789258` before the squash merge at `2026-07-13T00:42:10Z` as exact
revision `6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. On that exact merged revision,
Required CI run
`29215877872`, Workflow Online Audit run `29215877891`, External Links run
`29215877874`, OpenSSF Scorecard run `29215877875`, and dynamic CodeQL run
`29215877437` also completed successfully. OEP-0003 is Accepted at that
revision. This closure does not accept D-003 or D-004 or authorize any excluded
claim.

## How decisions change

An accepted decision changes through an Orange Enhancement Proposal or the
equivalent governance process. The proposal must state semantic, TCB, threat,
compatibility, conformance, migration, standards, IP, and schedule effects. A
security emergency can use a fast path, but it receives a time-bounded public
retrospective and permanent decision record.
