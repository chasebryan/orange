# Dependency-ordered roadmap to Orange 1.0

Status: proposed program plan

Planning horizon: approximately 84 months to the first LTS release

Snapshot: 2026-07-11

## 1. How to read this roadmap

This is a route to a production language and assurance ecosystem, not an MVP
sequence. The dates are staffing-based ranges and the gates are evidence-based.
When research takes longer, the gate moves; the project does not waive the gate
and preserve the date.

“No prototype” means:

- no partial system is marketed as production-ready;
- no merged product code is exempt from final error handling, documentation,
  determinism, tests, or assurance expectations;
- every merged product semantics, IR, checker, format, test, and build component
  is intended to survive in the production lineage;
- an early algorithm is a permanent conformance and regression artifact;
- design investigations produce reviewable evidence and are not allowed to
  become an accidental second architecture;
- competing Gate 0 decision cases live under archived research evidence, not
  the product implementation. The selected case graduates into the production
  conformance/metatheory suite; rejected cases remain reproducible history;
- development is continuously integrated. Avoiding a throwaway prototype does
  not justify a big-bang integration at the end.

The target is a bounded, complete 1.0 support envelope. “All cryptography on all
platforms” is not finishable. Gate 0 chooses exact algorithms, targets, ABIs,
leakage models, and claims that 1.0 will support completely.

## 2. Program outcome

Orange 1.0 is complete when it has:

- an editioned normative language specification and mechanized semantics;
- an authoritative small proof checker and independent implementation;
- a production frontend, reference evaluator, compiler, verified/validating
  optimization path, direct supported native targets, and stable generated C
  ABI;
- an explicit claim format, thin evidence manifest, and self-contained thick
  release bundle for offline proof/build replay;
- a standards-sourced cryptographic corpus with portable and accelerated
  implementations and a complete claim matrix;
- package/build tools, immutable resolution, registry security, formatter,
  language server, proof/trust explorer, documentation, and conformance suite;
- reproducible and independently rebuilt signed releases with provenance,
  SBOM/CBOM, proof bundle, and recovery infrastructure;
- independent formal, compiler, cryptographic, side-channel, and supply-chain
  review;
- an exercised PSIRT, LTS policy, governance process, and funded sustaining
  organization.

See [PROJECT_CHARTER.md](PROJECT_CHARTER.md) for the exact end condition and
[ASSURANCE.md](ASSURANCE.md) for stop-ship policy.

## 3. Version axes

Do not overload one version number with unrelated security meaning. A release
manifest binds:

1. language edition;
2. toolchain version;
3. cryptographic profile/registry snapshot;
4. target and leakage-assurance profile;
5. conformance-suite version.

This allows an unsafe algorithm to be withdrawn without changing language
semantics and a new target model to be added without rewriting historical proof
claims.

## 4. Workstreams

| ID | Workstream | Permanent responsibility |
| --- | --- | --- |
| W0 | Product, governance, and standards | Scope envelope, OEP process, naming/license/IP, standards surveillance, external partners |
| W1 | Language and semantics | Human reference, Spec/Impl/Game Core, types, effects, memory, erasure, leakage semantics |
| W2 | Proof and metatheory | Orange Proof IR, checker, decision procedures, certificate formats, metatheory, trust reporting |
| W3 | Frontend and developer tools | Parser, elaborator, diagnostics, formatter, incremental database, LSP, proof explorer |
| W4 | Compiler and targets | IRs, optimizers, validators, Machine IR, object path, ABIs, dispatch, bootstrap |
| W5 | Cryptography corpus | Standards provenance, specs, games, implementations, proofs, vectors, benchmarks, bindings |
| W6 | Package, build, and release | Manifest/lock, evidence bundle, registry, hermetic builds, signatures, provenance, updates |
| W7 | Assurance and conformance | Threat model, adversarial QA, fuzzing, independent checker, audits, labs, certification work |
| W8 | Documentation and adoption | Language book, references, operations, migrations, pilots, conformance program |

Documentation, assurance, standards tracking, and cryptography are concurrent
workstreams from the beginning. They are not final polish.

## 5. Dependency graph

```text
bounded product claims and support envelope
                 |
                 v
    normative cores + leakage/memory model
          |             |              |
          v             v              v
 proof kernel       frontend      target models
          \             |              /
           \            v             /
            +------ semantic IRs -----+
                          |
                          v
              verified/validating lowering
                          |
                          v
                  final-object evidence
                          |
                          v
      algorithm refinement + target leakage claims
                          |
                          v
        conformance + external assurance + release
```

Critical ordering rules:

- Compiler architecture cannot freeze before core semantics and the TCB model.
- A constant-time optimization cannot be approved before the leakage trace and
  target instruction classification exist.
- ABI and accelerated implementations depend on the memory, aliasing, integer,
  failure, and FFI models.
- Proof automation cannot close a claim before the proof/certificate trust rule
  is defined.
- Registry design depends on package claim composition and theorem fingerprints.
- Certification boundaries must be discussed with a lab before runtime and
  packaging boundaries become expensive to change.
- Every stable crypto package depends on an exact standards/errata import, not a
  citation added after implementation.

## 6. Phases and hard gates

No product implementation begins before Gate 0 closes. After that gate,
workstreams may overlap in calendar time, but a dependent component cannot pass
its gate before all prerequisite gates and preservation obligations close.

### Phase 0 — Constitution and evidence architecture

Indicative months: 0–9; indicative team: 14–18

Permanent outcomes:

- ratified charter, users, [user journeys](USER_JOURNEYS.md), non-goals, and a
  selected [1.0 support envelope](GATE0_SUPPORT_ENVELOPES.md);
- claim taxonomy, evidence graph/schema, TCB inventory format, and non-claims;
- exact proposed hosts, targets, ABIs, leakage profiles, and cryptographic
  corpus;
- Orange Enhancement Proposal and decision-record process;
- project name/namespace decision and migration plan from the codename;
- source/docs/generated-code/spec/vector license policy and contribution terms;
- governance, conflicts, release authority, PSIRT authority, and funding model;
- standards, patents, export, certification, and external-lab strategy;
- reproducible, archived
  [decision suite](PROOF_FOUNDATION_DECISION_SUITE.md) comparing Rocq, Lean, and
  any other proof foundation still under consideration;
- foundation-neutral human Core sketch and competing mechanized cases sufficient
  to test the decisions, stored as research evidence outside the product
  implementation;
- permanent repository policy, pinned development environment, CI trust
  boundaries, and documentation/conformance schema;
- independent feasibility review by cryptography, compiler, formal-methods,
  side-channel, product, and likely-consumer experts.

Gate 0 exit criteria:

- no unresolved contradiction between advertised claims and the proposed TCB;
- every 1.0 feature maps to an owner, dependency, evidence type, and exit test
  in the [Gate 0 feature traceability matrix](GATE0_TRACEABILITY.md);
- exact scope and non-goals are signed off;
- proof foundation, canonical formats, and leakage baseline are selected;
- naming and license gates are closed;
- staffing/funding can support at least the next two phases;
- external reviewers agree the plan is feasible within the stated envelope.

No language implementation begins merely to create visible syntax before this
gate. Mechanized decision cases are architecture evidence and must be retained
as permanent conformance/metatheory tests if their design is selected. Rejected
cases remain archived and reproducible but never become a second product path.

### Phase 1 — Normative language and semantics

Indicative months: 9–30, beginning only after Gate 0

Permanent outcomes:

- editioned grammar and name/module resolution;
- Spec Core and totality rules;
- Impl Core, memory/ownership/borrow model, effects, arithmetic, failure, and
  erasure;
- secrecy labels, declassification, leakage trace, and baseline
  noninterference statement;
- Game Core scope and semantic connection to shared pure definitions;
- target-independent ABI and FFI contract model;
- human-readable semantics cross-linked to the mechanization;
- executable reference evaluator;
- stable conformance-case schema and a case for every normative rule;
- first permanent standards-sourced spec fixtures: SHA-256 and ChaCha20 are
  strong candidates, subject to Gate 0.

Gate 1 exit criteria:

- no normative TODO or undefined behavior in the stable core;
- progress/preservation and relevant soundness, erasure, and noninterference
  theorems are checked;
- ambiguity review passes with independent parser work underway;
- every normative rule has a theorem cross-reference or conformance case;
- the flagship fixtures are executable and traceable to exact standards, but
  are not yet claimed as production implementations.

### Phase 2 — Assurance kernel and production frontend

Indicative months: 18–42

Permanent outcomes:

- canonical Core and Proof IR schemas;
- authoritative checker and independent checker;
- parser, elaborator, type/effect checker, termination checker, and VC
  generator;
- certificate interfaces for SAT, SMT fragments, algebra, and ranges;
- proof language, deterministic replay, proof cache, and source counterexamples;
- production diagnostics, formatter, incremental compiler database, and initial
  language server;
- axiom ledger, trust command, claim builder, and evidence inspector;
- adversarial proof/parser corpora and resource controls;
- local `Orange.toml`, `Orange.lock`, package-object, and thick/thin evidence
  formats, with content-addressed offline proof/build replay from a thick bundle
  or populated local store, but no public registry yet.

Gate 2 exit criteria:

- checker soundness argument and kernel logic audit complete;
- every accepted axiom inventoried and visible in dependent claims;
- malformed-proof fuzzing and mutation cannot produce an unexplained acceptance;
- authoritative and independent checkers agree on the frozen corpus;
- claim-closing mode rejects all solver trust steps, unknowns, and missing
  certificates;
- frontend behavior matches the normative suite and its failures are bounded.

### Phase 3 — Compiler, object path, and supported targets

Indicative months: 30–54

Permanent outcomes:

- CT IR and Machine IR semantics;
- ghost erasure, monomorphization, region lowering, instruction selection, and
  other stable verified passes;
- certificate-checking interfaces for search-based optimization, scheduling,
  vectorization, and allocation;
- x86-64/Linux and AArch64/Linux target/ABI/feature models or the exact Gate 0
  replacements;
- direct object emission plus byte/section/relocation/symbol validation;
- stable generated C ABI, headers, machine-readable contract, and Rust wrapper;
- checked CPU-feature dispatch and portable fallback;
- published stage-0 bootstrap and clean rebuild path;
- compiler fuzzing, differential, metamorphic, ABI, object, and hardware labs.

Gate 3 exit criteria:

- every stable IR has semantics and a validator;
- every supported pass is proved or produces an accepted artifact certificate;
- end-to-end functional preservation reaches final bytes for each supported
  tuple;
- target leakage preservation reaches final bytes for the claimed profiles;
- no printer, assembler, linker, dispatch, or wrapper gap is hidden from the TCB;
- zero unexplained differential mismatch and no silent assurance fallback;
- performance meets the approved per-profile floor without bypassing evidence.

### Phase 4 — Cryptography corpus and composition

Indicative months: 30–66

Permanent outcomes:

- standards/errata registry and clause-linked imports;
- approved flagship algorithms across hash, symmetric/AEAD, KDF/MAC, finite
  field/curve, and post-quantum domains;
- portable and selected accelerated implementations;
- implementation refinement, safety, leakage, ABI, conformance, and where
  selected game-based security claims;
- generated defensive APIs, vectors, ACVP adapters, docs, and benchmark suites;
- checked multi-implementation dispatch;
- algorithm admission, deprecation, withdrawal, and emergency profile process.

Recommended build order, subject to Gate 0:

1. SHA-256/512: core words, streaming, vectors, compiler baseline.
2. ChaCha20-Poly1305: ARX, field arithmetic, AEAD failure semantics, leakage.
3. HKDF/HMAC: parameterized composition and security-game connection.
4. AES-GCM: intrinsics, dispatch, multi-implementation proof.
5. X25519/Ed25519: field synthesis, encodings, scalar and ABI behavior.
6. ML-KEM: polynomials, matrices, failure behavior, errata and ACVP.
7. A selected signature family such as ML-DSA or SLH-DSA if included in the 1.0
   envelope.

Gate 4 exit criteria:

- every stable package has exact standards, errata, IP, and transition status;
- every advertised implementation/target has a complete claim matrix;
- all official, negative, boundary, interoperability, and applicable adversarial
  vectors pass;
- authentication failures and misuse cases have explicit API behavior;
- target leakage evidence exists for each promised profile;
- ACVP status is accurately labeled and no local run is called certification;
- no unapproved or lower-assurance fallback is reachable.

### Phase 5 — Developer product and ecosystem

Indicative months: 42–66

Permanent outcomes:

- production package resolver/client and policy built on the Gate 2 manifest,
  lock, object, and evidence formats;
- registry with TUF-style metadata, namespace governance, MFA, recovery,
  quarantine, and revocation;
- complete language server and proof/trust/IR explorer;
- documentation generator and versioned offline manuals;
- language book, proof guide, secure API guide, operations/reproducibility guide,
  migration material, and end-to-end corpus walkthroughs;
- public conformance runner and scoped conformance labels;
- external-design-partner workflows and migration/integration support.

Gate 5 exit criteria:

- install, specify, implement, prove, build, inspect, integrate, update, revoke,
  and offline-replay journeys are complete and documented;
- documentation examples compile, prove, and execute in CI;
- representative external users complete usability tasks without private help;
- registry/dependency compromise and recovery exercises pass;
- no arbitrary package script can affect a claim-bearing graph;
- at least one independently developed frontend/checker passes its applicable
  stable conformance suite before final 1.0.

### Phase 6 — Independent assurance and qualification

Indicative months: 60–78

Permanent outcomes:

- independent logic/metatheory and checker review;
- compiler and binary-path audit;
- applied cryptography and API audit;
- target side-channel laboratory work;
- supply-chain, registry, update, and PSIRT exercises;
- two independent reproducible-build witnesses;
- certification pre-assessment and any in-scope external validation work;
- real external production pilots within the supported envelope;
- public findings and dispositions.

Gate 6 exit criteria:

- every stop-ship finding is fixed and retested;
- residual risks are understood, owned, and publicly disclosed;
- release, revocation, key compromise, proof invalidation, algorithm withdrawal,
  and disaster recovery drills pass;
- production pilots complete without expanding scope by exception;
- support/funding covers the promised LTS window.

### Phase 7 — Orange 1.0 LTS

Indicative months: 75–84

Permanent outcomes:

- frozen Language Edition 1 and compatibility policy;
- signed, reproducible supported-host distributions;
- proof, claim, standards, conformance, SBOM/CBOM, provenance, audit, and support
  material for every artifact;
- published conformance program and stable crypto/target profiles;
- release and PSIRT on-call coverage;
- migration and upgrade material;
- archival and historical replay plan.

Gate 7 exit criteria:

- unanimous sign-off from language/semantics, assurance/TCB, compiler, applied
  cryptography, release engineering, and PSIRT authorities;
- every criterion in the charter’s definition of “end” is evidenced;
- no release depends on an undocumented manual step or unavailable private
  artifact.

### Phase 8 — Sustaining standard

Ongoing outcomes:

- compatible 1.x security and tooling updates;
- dated cryptographic-profile additions, deprecations, and withdrawals;
- certification maintenance for named modules;
- periodic target-model and side-channel review;
- new language semantics only through a future edition/OEP process;
- preserved historical sources, tools, proofs, and release attestations;
- five-year full-support and proposed two-year critical-only tail, subject to
  Gate 0 funding commitments.

## 7. First 90 days

The first 90 days close ambiguity; they do not rush to a parser screenshot.

### Days 1–30

- Ratify the mission, “no prototype” doctrine, target users, and complete user
  journeys.
- Open decisions for name, licenses, governance, proof foundation, target
  matrix, leakage baseline, corpus, and support policy.
- Recruit named external advisors across cryptography, compilers, formal
  methods, side channels, usability, and likely integrations.
- Write the first threat model and claim/assumption vocabulary.
- Inventory standards, errata sources, vector sources, validation paths,
  licenses, and name collisions.
- Define the decision-suite requirements and prohibited hidden assumptions.

### Days 31–60

- Build canonical draft schemas for claim records, standards provenance, TCB
  inventory, and evidence bundle; review them with auditors and integrators.
- Write representative permanent semantic cases: bitvector versus word
  arithmetic, decoding, regions/aliasing, secret branch/address rejection,
  leakage trace, and one probabilistic game relation.
- Implement those cases independently in the shortlisted proof foundations only
  as reproducible, archived architecture decision evidence. They are not
  production Orange implementations.
- Draft and independently review the exact
  [1.0 support-envelope options and resource bands](GATE0_SUPPORT_ENVELOPES.md).
- Select secure repository, CI trust-boundary, dependency, and review controls.
- Begin accredited-lab and standards-body conversations if validation is in
  scope.

### Days 61–90

- Run and publish the proof-foundation/format decision results.
- Select the proposed core calculus, certificate strategy, leakage baseline,
  initial target tuple, and first corpus fixture.
- Publish the first OEPs and decision records.
- Close naming and licensing due diligence or explicitly hold public branding.
- Produce the Phase 0 staffing/funding and procurement plan, including external
  reviews and lab capacity.
- Commission the independent challenge review for the full constitution.

After 90 days, Phase 0 continues through the independent constitution review and
all Gate 0 exit criteria. Product implementation begins only after the complete
Gate 0 closes; until then, code is limited to reproducible decision evidence
outside the product lineage.

## 8. Staffing and duration

A credible reference program peaks around 35–45 people and takes approximately
seven years. Formal verification, applied cryptography, target compiler work,
independent assurance, product tooling, and operations are different jobs.

| Function | Peak FTE |
| --- | ---: |
| Program/product/governance/standards | 3 |
| Language design and semantics | 5 |
| Formal methods, checker, metatheory | 6 |
| Compiler, targets, object path, bootstrap | 8 |
| Applied cryptography and corpus | 6 |
| Conformance, adversarial QA, product security | 5 |
| Build, registry, release, supply chain | 4 |
| Developer tooling, docs, education | 5 |
| **Reference peak** | **42** |

Phase 0 can begin with 14–18 unusually strong cross-disciplinary contributors.
Peak staffing arrives during the compiler/corpus/tooling overlap.

A smaller team should extend the calendar or reduce the ratified 1.0 envelope.
It must not quietly delete independent checking, target preservation, release
integrity, or incident response while keeping the same public claim. A solo
maintainer can lead Phase 0 and build permanent research assets, but the stated
1.0 exit criteria intentionally require independent authorship, review, a bus
factor greater than one, and operational separation of duties.

Staffing controls:

- at least three capable maintainers for every critical subsystem before 1.0;
- separate release and PSIRT rotations;
- two independent external assurance organizations funded;
- academic replication and independent checker/frontend work funded;
- specialist target hardware and side-channel lab capacity reserved early;
- multi-year sustaining reserve rather than funding only the build phase.

## 9. Program metrics

### Specification and proof

- 100% of public claims map to named evidence and assumptions.
- Zero undocumented axioms or trusted solver steps.
- Every normative stable rule has a conformance case or mechanized reference.
- Authoritative/independent checker agreement is 100% on the stable suite.
- Proof replay succeeds offline from clean release inputs.
- TCB size and changes are published per release.

### Compiler and corpus

- Zero unresolved differential mismatch.
- Zero stable algorithm-vector failure.
- 100% of supported tuples pass the target profile; unsupported tuples cannot
  accidentally claim assurance.
- Fuzzing, mutation, and adversarial-corpus results have no unexplained accepts,
  crashes, or hangs.
- Performance, stack, memory, code-size, and proof-time stay within approved
  per-profile budgets.

### Supply chain and operations

- 100% of release artifacts are bit-identically rebuilt by two independent
  builders.
- 100% carry verifiable provenance, signature, SBOM/CBOM, claims, TCB, and proof
  bundle.
- Protected release source and builders continuously meet the pinned SLSA/OSPS
  policies.
- Key/update/revocation/disaster drills meet their recovery objectives.
- Security acknowledgment, assessment, fixed-release, and downstream-notice
  objectives are measured.

### Usability and governance

- 100% of documentation examples compile, prove, and run.
- External users complete the full workflow and understand failure categories.
- Critical subsystem bus factor is at least three.
- Two-person review compliance is 100% for protected critical changes.
- Decision latency, participation, conflicts, funding, and expired exceptions
  are visible.

The project does not use theorem count, proof line count, code coverage, GitHub
stars, or benchmark wins alone as evidence of readiness.

## 10. Principal risks

| Risk | Early signal | Required response |
| --- | --- | --- |
| Scope is effectively “all crypto” | New algorithms/targets arrive without removing work | Freeze a finite envelope and versioned profiles at Gate 0 |
| Custom checker consumes the whole program | Kernel/format churn blocks semantics | Keep the logic deliberately small; compare proof foundations before commitment |
| Formalization and implementation share one mistake | Same team/model produces all evidence | Independent checker/frontend, vectors, oracles, and external review |
| Source-to-binary leakage proof stalls | Compiler work proves only functional behavior | Treat leakage refinement as a pass acceptance condition from CT IR onward |
| Object/ABI last mile stays informal | Claims stop at generated C/assembly | Final-byte decoder/validator and ABI conformance are Gate 3 requirements |
| Solver instability breaks replay | Proofs depend on unpinned heuristics | Certificates, exact digests/flags, bounded checking, solver diversity |
| Proof ergonomics remain academic | Only core authors can close corpus obligations | LSP goals, source counterexamples, libraries, user studies, external pilots |
| Performance pressure bypasses assurance | Fast variant lacks full matrix | Keep a verified fallback; no variant/dispatch admission without its evidence |
| Target behavior changes | New CPU errata or speculative attacks | Versioned target profiles, review dates, lab tests, emergency withdrawal |
| PQC/standards churn | Errata or drafts change behavior | Exact standards snapshots, surveillance, separate crypto profiles |
| Unsafe FFI defeats verified core | Caller contracts are vague or untestable | Generated defensive wrappers and machine-readable ABI contracts |
| Registry compromise | Namespace/account anomaly | TUF-style metadata, MFA, quarantine, revocation, immutable locks |
| Bootstrap compromise | Release cannot be rebuilt independently | Published stage 0, diverse compilation, independent rebuilders |
| Certification discovered too late | Module boundary conflicts with runtime | Lab pre-assessment during Phase 0 |
| Naming or licensing blocks distribution | Collision or incompatible dependency found late | Close name/license/IP gate before public branding and core dependency freeze |
| Governance capture or burnout | One person controls critical decisions/releases | Neutral governance, bus factor, separation of duties, funding transparency |
| Funding pressure asks to waive gates | Schedule becomes the only success measure | Gates cannot be waived for public assurance; narrow scope or extend schedule |
| Product remains a research artifact | No external workflow completes | Design partners, migrations, integration APIs, usability evidence from Phase 2 |

## 11. What happens if the plan is under-resourced

Use this order of response:

1. Remove a 1.0 algorithm family or accelerated variant.
2. Remove a 1.0 host/target tuple.
3. Reduce the number of native leakage profiles.
4. Defer the public registry while preserving local content-addressed packages.
5. Extend the schedule.

Do not respond by:

- trusting opaque solver success;
- stopping the proof at generated C while claiming binary assurance;
- removing independent review/checking;
- hiding a foreign or platform assumption;
- calling tests proofs;
- calling ACVP certification;
- shipping without vulnerability/update/recovery operations.

This ordering preserves a smaller complete product instead of a broad,
misleading one.

## 12. End-of-build handoff

At Gate 7, the build program becomes a language standards and maintenance
program. Historical artifacts remain replayable. Actual end-of-life requires a
successor or migration path, a published final support date, preserved source,
spec, proof, toolchain and release archives, and a way to verify historical
claims after online services disappear.
