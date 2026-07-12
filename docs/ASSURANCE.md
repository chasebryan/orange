# Assurance and security model

Status: proposed technical assurance model with directed solo evidence boundary

Research snapshot: 2026-07-11

Solo amendment: 2026-07-12

Under D-023, outside human and organizational participation is
unavailable for current planning. Independent review, external audit,
laboratory validation, separate release roles, and independent rebuilds are
therefore optional evidence classes, not development prerequisites. Their
absence must remain explicit and limits the claims Orange can make. It never
turns owner review into independent evidence or weakens a machine-checkable
technical obligation for a claim that Orange actually advertises.

## 1. Assurance promise

Orange will say exactly what was checked, for which artifact, with which model,
and under which assumptions. It will not promise that “formal verification” is
a universal security property.

Every public claim resolves to a machine-readable record. Claim-bearing releases
include a thick bundle with every proof- and build-critical byte needed for the
offline replay they advertise. Kernel proofs and certificates are independently
machine-checked; tests are rerun; external audits and validations are recorded
with verifiable identity, scope, digest, and validity metadata but are not
misrepresented as kernel-checkable judgments.

If Orange cannot establish a requested claim, its outcome is
`not_satisfied`, `unresolved`, or `unsupported`; it is never silently downgraded
to a test pass or inherited from a neighboring implementation.

The assurance program covers the language and its development process. A sound
logic shipped through a compromised release pipeline is not a trustworthy
product, and a hardened pipeline cannot compensate for a wrong semantic model.

## 2. Threat model

### 2.1 Adversaries

Assume all of the following:

- A remote attacker chooses cryptographic inputs and observes outputs, error
  behavior, timing, allocation, protocol effects, and any leakage exposed by
  the deployment.
- A local co-resident attacker has stronger timing and cache observation.
- A physical attacker exists only for a profile that explicitly models and
  claims physical resistance.
- A malicious Orange source, proof, package, certificate, or object-file author
  attempts proof forgery, parser differentials, miscompilation, resource
  exhaustion, or evidence confusion.
- A dependency, maintainer account, package publisher, CI runner, registry,
  mirror, signing identity, or release workstation may be compromised.
- A privileged insider may attempt to bypass review, replace source or
  artifacts, weaken a target profile, or conceal an assumption.
- A standards ambiguity, erratum, unsafe composition, weak primitive, or stale
  cryptographic profile may make internally consistent software unsafe.
- A well-intentioned integrator may misuse a low-level API, pass overlapping or
  short buffers, reuse state/nonces, mishandle authentication failure, or choose
  an unsupported target.
- A compiler, linker, loader, OS, firmware, CPU, accelerator, or entropy source
  may behave outside Orange’s model.

### 2.2 Assets and controls

| Asset | Principal threats | Technical controls and solo evidence boundary |
| --- | --- | --- |
| Semantic truth | Ambiguous rules, unsound axiom, kernel bug | Complete normative semantics, small checker, axiom ledger, and implementation-diverse checking; independent logic review is recorded only when available, and its absence limits dependent claims |
| Source intent | Wrong transcription of a standard, ignored errata | Clause-linked standards provenance, vectors, explicit transcription-review status, and separately exercised owner cross-checks; external cryptographer review is unavailable and not claimed |
| Compiler correctness | Pass bug, backend drift, printer/assembler mismatch | Semantic IRs, verified passes or checked certificates, differential fuzzing, final-byte validation |
| Secret confidentiality | Branch/address/timing leakage, diagnostics, stale copies | Secrecy types, named leakage model, erasure obligations, binary analysis, and owner-executable measurements; laboratory evidence is required only for claims whose profile calls for it, and those profiles remain unsupported while it is unavailable |
| Key and entropy lifecycle | Weak entropy, reuse, cloning, false zeroization | Explicit provider contracts, affine capabilities, misuse-resistant APIs, target-scoped erasure claims |
| Claim integrity | Evidence substitution, hidden assumption, target confusion | Content addressing, claim closure, canonical formats, fail-closed checking |
| Build and release | Dependency/CI compromise, forged provenance, rollback | Hermetic inputs, SLSA, reproducible builds, signatures, transparency, TUF-style updates |
| Registry | Typosquatting, account takeover, malicious package, downgrade | MFA, namespace policy, trust tiers, quarantine/revocation, immutable lockfiles |
| Availability | Proof bombs, pathological parser input, solver divergence | Streaming formats, deterministic resource limits, cancellation, adversarial corpus |
| Governance | Capture, unilateral critical changes, sponsor pressure | Public decisions, conflict disclosure, and explicit solo-review status; two-person review and threshold authority are unavailable, remain disclosed conformance gaps, and are not claimed as current controls |

### 2.3 Security boundaries

The boundaries are:

- human standards intent to formal Orange specification;
- surface syntax to canonical Core;
- Core to checked claim/proof;
- proof search to proof checking;
- each compiler IR transformation;
- Machine IR to encoded object and linked artifact;
- generated C ABI to foreign caller;
- Orange artifact to OS/CPU/entropy provider;
- source repository to release builders, registry, and update client.

Each boundary must appear in the threat model, claim closure, or release
provenance. Treating a boundary as “just serialization” is prohibited.

## 3. Claim model

### 3.1 Independent claim dimensions

The following are independent:

1. well-formedness and totality;
2. memory, initialization, arithmetic, panic, and trap safety;
3. functional refinement to a named specification;
4. standard/profile conformance;
5. termination and optionally resource bounds;
6. source-level leakage noninterference;
7. target-level leakage preservation;
8. ABI, layout, serialization, and error-contract correctness;
9. architectural erasure/zeroization;
10. game-based cryptographic security and concrete advantage bound;
11. entropy, foreign-code, OS, CPU, and hardware assumptions;
12. empirical vector, differential, fuzz, timing, and interoperability evidence;
13. external validation or certification of a concrete module.

No dimension has a number that implies the others.

### 3.2 Claim record

Every record contains:

- stable claim ID and exact human wording;
- subject source definition, export, and artifact digest;
- language edition, toolchain, crypto-profile snapshot, target profile, and
  leakage model;
- theorem and proof-object fingerprints;
- test, audit, lab, and external-certificate evidence where applicable;
- every explicit axiom, foreign contract, external checker, and platform
  assumption in the closure;
- explicit exclusions;
- review/expiry policy for target- and standards-sensitive claims;
- one claim outcome: `satisfied`, `not_satisfied`, `unresolved`, or
  `unsupported`;
- typed basis entries: `kernel_proof`, `checked_certificate`,
  `external_proof`, `test_run`, `audit`, `external_validation`, or `assumption`;
- for each basis entry, a verification state such as `checked`, `recorded`,
  `failed`, `expired`, or `unavailable`.

Outcome, trust basis, and evidence are orthogonal. A satisfied claim may have a
kernel proof, tests, and an external audit at the same time. An assumption is a
visible dependency, not a successful proof status. `satisfied` requires at
least one valid, unexpired basis allowed by that claim kind’s policy; assumptions
alone cannot satisfy a proof-required claim. Other outcomes may have no basis
when, for example, the claim is unsupported before evidence can be produced.

The CLI and generated documentation display the complete matrix for every
exported symbol and implementation/target variant.

### 3.3 Trust budget

Every release reports:

- executable and source size of the authoritative checker;
- accepted axioms and why each is necessary;
- modeled ISA, ABI, object, and leakage components;
- external contracts and proof systems;
- changes to the TCB since the prior release;
- which claims are invalidated if a component is compromised.

The goal is not an arbitrary line-count threshold. The goal is a small,
reviewable, slowly changing closure with no undocumented expansion.

## 4. Leakage and side-channel posture

### 4.1 Baseline model

The first stable policy uses two-run noninterference over an architectural trace
containing at least:

- branch decisions and targets;
- memory addresses, widths, and access classes;
- indirect call/return targets;
- traps, exceptions, and termination;
- target-classified variable-latency instruction use.

The claim names the public-input relation and permitted declassification.

### 4.2 Stronger profiles

Speculative execution, data-independent-timing architectural modes, masked
implementations, power/EM, and fault resistance require distinct profiles with
their own semantics, target assumptions, tests, and review. They do not extend
the baseline by implication.

### 4.3 Evidence stack

A target leakage claim requires:

1. source/CT IR noninterference evidence;
2. pass-by-pass leakage preservation or checked translation validation;
3. Machine IR and final-object correspondence;
4. target instruction classification and ABI assumptions;
5. binary static inspection;
6. empirical timing testing on named hardware as defense in depth;
7. specialist laboratory work for release profiles whose stronger claims
   require it; while that work is unavailable, those profiles remain
   `unsupported` rather than blocking unrelated development.

Statistical tools can reveal a model or implementation failure; failure to
detect leakage is not a proof.

## 5. Verification and validation strategy

### 5.1 Proof checker

Required gates:

- formal soundness statement for the checked relation;
- explicit axiom inventory;
- deterministic and resource-bounded checking;
- malformed, cyclic, oversized, and adversarial proof corpus;
- structure-aware fuzzing and mutation of accepted proof objects;
- differential agreement between authoritative and implementation-diverse
  checkers where two checkers exist;
- proof-format compatibility and rejection tests;
- explicit review status showing that external logic and implementation audit
  are unavailable in solo mode.

Any soundness flaw is a stop-ship issue and triggers an analysis of every claim
that depended on the affected checker version.

### 5.2 Frontend and semantics

- Every normative grammar/static/dynamic rule maps to a conformance case or a
  mechanized theorem reference.
- Before a stable edition, ambiguity and differential behavior must be tested
  with a second implementation, generated-parser oracle, or equivalent
  owner-executable method. Same-owner diversity is never called independent.
- Parser, formatter, and elaborator fuzzing cover invalid UTF-8 policy,
  confusables, nesting, error recovery, namespace resolution, and resource use.
- The executable reference semantics is differentially compared with Impl Core,
  optimized IRs, and native results.
- Diagnostics for invalid programs are tested; a crash or hang is not an
  acceptable rejection mode.

### 5.3 Compiler

Every stable IR has a syntax/encoding, validator, interpreter or executable
semantics, and mechanized relation to adjacent IRs.

Compiler validation includes:

- property-based generation of well-typed programs;
- reference-versus-optimized differential execution;
- metamorphic transformations;
- pass-order and optimization-level variance;
- aliasing, alignment, endianness, boundary widths, and overlap cases;
- sanitizer and interpreter lanes for the compiler implementation;
- target emulation and real-hardware execution;
- object/relocation/ABI inspection;
- translation-certificate mutation and rejection;
- CPU dispatch and fallback testing;
- performance, code-size, proof-time, and memory regression budgets.

There is no silent fallback from a claimed target-specific implementation to an
unprofiled implementation.

### 5.4 Cryptography package admission

Every stable algorithm/construction/profile has:

- exact normative publication, edition, errata snapshot and source digest;
- clause-to-definition traceability, explicit transcription-review status, and
  separately exercised owner cross-checks; external cryptographer review is
  unavailable, and any claim that requires it remains `unsupported`;
- intellectual-property and transition/deprecation status;
- mathematical spec and implementation-refinement evidence;
- an honest statement separating implementation correctness from assumed
  hardness and game-based theorems;
- known-answer and intermediate-value vectors where available;
- negative, malformed, boundary, overlap, cross-endian, and authentication-
  failure cases;
- differential tests against mature independent implementations;
- Wycheproof or comparable adversarial corpora where applicable;
- source and target leakage evidence for every promised profile;
- ACVP Demo coverage when supported, without claiming a certificate;
- approved performance, memory, stack, and proof-replay budgets.

Cryptographic profiles can deprecate or withdraw algorithms without changing
language semantics.

## 6. Flagship corpus plan

The final corpus must exercise different permanent capabilities, not collect
demo algorithms.

| Family | Architectural purpose | Candidate claim coverage |
| --- | --- | --- |
| SHA-256/SHA-512 | Bit/word semantics, streaming state, vectors | Conformance, refinement, safety, native compilation |
| ChaCha20-Poly1305 | ARX, field arithmetic, AEAD API/failure behavior | Refinement, leakage, ABI, composition, RFC vectors |
| AES-GCM | Hardware intrinsics, tables forbidden in CT profile, dispatch | Multiple implementations, target features, leakage, conformance |
| HKDF/HMAC | Generic modules and composition | Refinement, security-game linkage, state/API behavior |
| X25519/Ed25519 | Finite fields, encodings, scalar handling | Field synthesis, canonical decode, leakage, ABI |
| ML-KEM | Polynomials, matrices, rejection/failure behavior, PQC standards | Standards+errata provenance, safety, leakage, ACVP |
| ML-DSA or SLH-DSA | Larger state/performance and randomized signatures | Parameterized modules, entropy effects, PQC profile evolution |

Final 1.0 membership requires an incremental corpus decision. Starting order
should maximize semantic coverage and independent vectors, not marketing
breadth.

## 7. CI policy

### Per pull request

- formatting, lint, unit tests, and affected conformance cases;
- mechanization/spec cross-reference checks;
- affected proof replay and certificate checks;
- changed-component fuzz/regression corpus;
- TCB, assumptions, public API, and public-claim delta;
- dependency, license, secret, and policy checks;
- documentation examples touched by the change;
- no privileged release secret exposed to untrusted PR execution.

### Protected-branch merge

- full proof replay;
- all supported host and target builds;
- reference/compiler differential suite;
- sanitizers and ABI/object tests;
- complete stable algorithm vector and negative suite;
- documentation build and examples;
- SBOM/CBOM and unsigned provenance generation as validation.

### Nightly

- coverage-guided and grammar-aware fuzzing;
- compiler differential/metamorphic campaigns;
- cross-architecture emulator and hardware runs;
- binary leakage analysis and statistical timing tests;
- performance, proof-time, code-size, stack, and memory regression;
- solver-version diversity and certificate replay;
- standards/errata/dependency surveillance.

### Weekly

- clean bootstrap from published stage-0 inputs;
- diverse compilation or equivalent bootstrap check;
- build-environment variance and network-disabled build;
- registry, dependency, and update compromise exercises;
- clean install, offline proof replay, and recovery checks.

### Release candidate

- frozen content-addressed dependency graph;
- network-disabled build from declared inputs;
- complete formal, target, vector, conformance, docs, and audit suite;
- two clean, separately provisioned owner rebuilds with identical artifacts and
  an explicit `not independently rebuilt` status;
- key, registry, update, rollback, and disaster-recovery drills;
- recorded solo release ceremony and owner sign-off.

## 8. Stop-ship conditions

Release is blocked by:

- unresolved proof-soundness flaw;
- incorrect cryptographic output;
- secret-dependent behavior within a promised target/leakage profile;
- undocumented axiom, TCB expansion, foreign boundary, or claim downgrade;
- semantics ambiguity that changes a valid program’s meaning;
- failed reproducibility, signature, provenance, update, or rollback protection;
- unresolved critical or high security, soundness, cryptographic, compiler, or
  promised-profile finding;
- unreviewed standards erratum relevant to a stable package;
- audit finding whose impact is not understood.

Security, soundness, and public-assurance gates cannot be waived. An exception
to a non-assurance operational gate requires a named owner, rationale,
compensating control, expiry, approval, and release-note disclosure.

## 9. Release and supply-chain policy

### 9.1 Framework targets

- NIST SSDF 1.1 is the secure-development framework baseline until a newer final
  edition is ratified.
- Target SLSA 1.2 Source Level 4 for protected release branches and Build Level
  3 for release artifacts.
- Target the current OpenSSF OSPS Baseline Level 3 for every release-bearing
  repository; pin the exact baseline version in evidence.
- Require hermetic/network-disabled and reproducible builds in addition to
  SLSA, because the SLSA level alone does not promise all of those properties.

### 9.2 Release contents

Every release ships:

- source archive and binaries;
- exact bootstrap and build-environment inputs;
- proof and claim bundle;
- standards/errata/vector snapshot;
- conformance results;
- SLSA/in-toto provenance;
- Sigstore bundle or equivalent identity/signature/transparency evidence;
- TUF-style update metadata;
- SPDX SBOM;
- CycloneDX SBOM/CBOM;
- TCB and axiom inventory;
- audit status and any available reports and finding dispositions;
- changelog, security changes, known limitations, and support dates.

The logical evidence remains verifiable without trusting a transparency service
to stay online forever.

### 9.3 Keys and updates

- Offline root and recovery keys with separately stored recovery material.
- Narrowly scoped online signing/update credentials; the sole owner controls
  them in solo mode and the missing role separation remains disclosed.
- Recorded owner release approval.
- Regular rotation and documented expiry.
- Rollback and freeze protection.
- Revocation and compromise recovery that has been rehearsed.
- The owner necessarily controls source acceptance, build, signing, registry,
  and recovery in solo mode; credentials and procedures are separated where
  possible and the missing separation of duties is disclosed.

## 10. Vulnerability response

The owner performs the security-response function before public packages.

Required public material:

- `SECURITY.md` and supported-version matrix;
- encrypted/private reporting path and an asset-scoped good-faith-research
  boundary; counsel review is unavailable and must not be claimed;
- disclosure and remediation policy;
- downstream notification and advisory channels.

Operational targets:

- acknowledge a report within one business day;
- provide an initial technical assessment within three business days;
- enter immediate incident mode for active exploitation, proof unsoundness,
  widespread silent miscompilation/leakage, registry compromise, or release-key
  compromise.

Orange-specific classes include:

- proof-system unsoundness;
- compiler miscompilation;
- target leakage-profile failure;
- standard nonconformance;
- unsafe API or misleading claim;
- build/update compromise;
- malicious or taken-over package;
- documentation that predictably induces cryptographic misuse.

An incident identifies the affected tuple: language edition, toolchain, crypto
profile, target/leakage profile, package/artifact, and operating environment.
The fix updates code, semantics, proofs, vectors, documentation, claims, and
attestations together. Advisories state which old claims are invalid.

Align the process with NIST SSDF and the current ISO/IEC 29147 disclosure and
ISO/IEC 30111 handling standards, tracking their revisions.

## 11. External validation posture

NIST ACVP/CAVP vector testing, Orange proofs, and FIPS 140 module validation are
different evidence classes.

- Orange imports/exports ACVP-compatible data and records Demo/production/lab
  status.
- A local test run is not an algorithm certificate.
- An algorithm certificate does not validate the complete module.
- The Orange language cannot be FIPS 140 validated in the abstract.
- A concrete generated/runtime module, exact build, boundary, approved modes,
  entropy strategy, self-tests, version, platform, and operational environment
  can be assessed externally.

Accredited-laboratory work is unavailable in solo mode. Certificate-bearing
profiles therefore remain `unsupported`. Module, runtime, entropy, update, and
package boundaries should still avoid choices that make later assessment
unnecessarily difficult.

## 12. Governance controls

The project owner is the sole governance, review, release-policy, and
security-response authority under D-019 and D-023. No committee, board,
independent adviser, external auditor, or separate operational role is assumed.
This is a disclosed assurance limitation rather than a substitute control.

Normative or security-relevant changes use public Orange Enhancement Proposals
containing motivation, non-goals, semantic changes, threat/TCB impact, proof
obligations, compatibility, conformance changes, implementation evidence,
standards and IP provenance, and alternatives.

The owner may approve an owner-authored TCB, cryptography, or release-system
change, but the record must say `solo-reviewed`, include adversarial evidence,
and preserve all unresolved risk. Protected history and required checks provide
defense in depth; they do not create a second trusted person.

## 13. Explicit non-claims

Orange does not, by default:

- prove the hardness of an underlying mathematical problem;
- turn functional correctness into construction or protocol security;
- turn source constant-time into binary constant-time;
- turn conventional constant-time into speculative, power, EM, or fault
  resistance;
- prove entropy quality beyond an explicit provider model;
- prove foreign callers satisfy an ABI precondition;
- make unmodeled OS, firmware, CPU, accelerator, or linker behavior trustworthy;
- grant FIPS or any other external certification;
- establish that a human transcription captures the intended standards prose.

These limitations are part of the product, not footnotes to remove from visible
reports.

## 14. Current external baselines

- [NIST SSDF 1.1](https://csrc.nist.gov/pubs/sp/800/218/final)
- [SLSA 1.2](https://slsa.dev/spec/v1.2/)
- [SLSA Source requirements](https://slsa.dev/spec/v1.2/source-requirements)
- [OpenSSF OSPS Baseline](https://baseline.openssf.org/)
- [Reproducible Builds](https://reproducible-builds.org/docs/definition/)
- [Sigstore](https://docs.sigstore.dev/cosign/signing/overview/)
- [TUF specification](https://theupdateframework.github.io/specification/latest/)
- [SPDX specifications](https://spdx.dev/use/specifications/)
- [CycloneDX specification](https://cyclonedx.org/specification/overview/)
- [CycloneDX CBOM](https://cyclonedx.org/capabilities/cbom/)
- [NIST CAVP](https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program)
- [NIST ACVP](https://pages.nist.gov/ACVP/)
- [FIPS 140-3](https://csrc.nist.gov/pubs/fips/140-3/final)
- [ISO/IEC 29147:2018](https://www.iso.org/standard/72311.html)
- [ISO/IEC 30111:2019](https://www.iso.org/standard/69725.html)

Versions in this section are a research snapshot. Release policy pins and
re-evaluates the then-current final versions rather than following `latest`
silently.
