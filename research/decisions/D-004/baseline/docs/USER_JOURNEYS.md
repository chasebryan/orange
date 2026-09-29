# Proposed Orange 1.0 user journeys

Status: proposed product-boundary evidence; no end-to-end journey is complete

Snapshot: 2026-07-11

Solo amendment: D-023 treats external-user and independent-human validation as
unavailable rather than as a development prerequisite. Completion tests are
therefore executable by the owner in clean declared environments. Every result
records `solo-reviewed`; no owner run is relabeled as external validation.

## 1. Purpose and limits

This document turns the five target-user groups in
[`PROJECT_CHARTER.md`](PROJECT_CHARTER.md#3-target-users) into falsifiable,
end-to-end product journeys. It also covers the ten operations required by the
later capability stages in [`ROADMAP.md`](ROADMAP.md)
and connects each journey to the fourteen feature groups in
[`GATE0_TRACEABILITY.md`](GATE0_TRACEABILITY.md#4-feature-matrix).

The flows describe the intended 1.0 boundary. They do not ratify an exact CLI,
syntax, proof foundation, algorithm set, target set, registry, update system, or
release profile. Command families must eventually expose stable
machine-readable results, but command names and screen layouts remain later
interface decisions. A successful synthetic fixture or planning review does
not make a journey implemented, usable, secure, or independently validated.

These stable persona IDs are used below:

| ID | Charter target-user group |
| --- | --- |
| P-01 | Cryptographic implementers |
| P-02 | Verification engineers |
| P-03 | Cryptographers and standards authors |
| P-04 | Library maintainers and integrators |
| P-05 | Auditors and downstream consumers |

The required operation IDs are `install`, `specify`, `implement`, `prove`,
`build`, `inspect`, `integrate`, `update`, `revoke`, and `offline-replay`.
Operations may occur in more than one journey; coverage means that at least one
journey owns a complete acceptance path for each operation.

## 2. Journey index

| ID | Journey | Primary persona | Supporting personas | Operations | Feature groups | Target gate |
| --- | --- | --- | --- | --- | --- | --- |
| J-01 | Install and initialize an exact toolchain | P-04 | P-01, P-02 | `install` | F-01, F-02, F-07, F-11, F-14 | Gates 5 and 7 |
| J-02 | Import a standard and author a specification | P-03 | P-01, P-02 | `specify`, `inspect` | F-01, F-03, F-06, F-12, F-13 | Gates 1 and 4 |
| J-03 | Implement, prove, and classify claims | P-01, P-02 | P-03 | `implement`, `prove`, `inspect` | F-01, F-02, F-03, F-04, F-05, F-06, F-07, F-12 | Gates 2 and 4 |
| J-04 | Build and inspect native artifacts | P-01 | P-02, P-05 | `build`, `inspect` | F-04, F-05, F-07, F-08, F-09, F-10, F-11, F-12, F-14 | Gates 3 and 7 |
| J-05 | Integrate through generated foreign interfaces | P-04 | P-01, P-05 | `build`, `inspect`, `integrate` | F-04, F-05, F-08, F-09, F-10, F-12 | Gates 3 and 5 |
| J-06 | Audit and replay evidence offline | P-05 | P-02, P-04 | `inspect`, `offline-replay` | F-01, F-05, F-06, F-07, F-08, F-11, F-12, F-13, F-14 | Gates 2 and 7 |
| J-07 | Update, deprecate, withdraw, or replace a profile | P-04 | P-03, P-05 | `inspect`, `update`, `revoke`, `offline-replay` | F-02, F-05, F-11, F-12, F-13, F-14 | Gates 5 and 6 |
| J-08 | Respond to a vulnerability or invalidated claim | P-04, P-05 | P-01, P-02, P-03 | `inspect`, `update`, `revoke`, `offline-replay` | F-05, F-07, F-08, F-10, F-11, F-12, F-13, F-14 | Gates 6 and 7 |

## 3. Journey specifications

### J-01 — Install and initialize an exact toolchain

**Actors and intent:** P-04 installs a named Orange release for a supported host;
P-01 and P-02 need the same exact toolchain, checker, schemas, and target data.
The intent is a verified installation identity, not merely an executable on
`PATH`.

**Entry conditions and trusted inputs:** A release-capable governance stage,
ratified host and support profiles, a signed immutable release manifest,
published verification roots and procedure, exact asset digests, support dates,
and an otherwise empty supported host. Download transport and local caches are
untrusted until content and identity checks pass.

**Ordered flow:**

1. Select one immutable release identity and supported host tuple.
2. Acquire its manifest, signature/transparency evidence, toolchain archive,
   checker, schemas, target profiles, and bootstrap or installation metadata.
3. Verify authorized signer or threshold identity, manifest linkage, asset
   digest, platform tuple, version axes, and support state before execution.
4. Install without replacing a known-good installation until verification and
   self-checks complete.
5. Run deterministic installation conformance and print a machine-readable
   receipt containing every version axis, artifact digest, and trust-root ID.
6. Initialize a project manifest and immutable lock only from explicitly chosen
   inputs; a network lookup is not implicit proof or build input.

**Fail-closed outcomes:** An absent or invalid signature, digest mismatch,
unsupported host, expired/revoked identity, incomplete asset set, failed
self-check, or inconsistent version axis rejects the installation. A partial
install never becomes the active toolchain, and the tool must not substitute a
newer asset or host profile silently.

**Evidence outputs:** Installation receipt, verified release-manifest digest,
tool/checker/schema/target digests, host tuple, trust-root identity, conformance
results, and an exact diagnostic for any rejection.

**Non-goals:** Installation does not certify the host OS, firmware, CPU, entropy
source, or all installed packages. Possession of a signed Orange binary does
not satisfy any cryptographic claim.

**Completion test:** On every supported clean host, install from the published
asset set with an empty cache, reproduce the expected receipt, and pass the
installation conformance suite. Independently mutate the manifest, signature,
archive, host tuple, and one required asset; every mutation must reject before
the active installation changes.

### J-02 — Import a standard and author a specification

**Actors and intent:** P-03 records exact standards intent and authors a readable,
executable specification; P-01 and P-02 review implementability and formal
meaning. The result must distinguish source intent, Orange semantics, and later
implementation claims.

**Entry conditions and trusted inputs:** Ratified language and provenance
formats; exact publication, edition, errata, vector, rights, and archival
records; a selected corpus/profile scope; and a clean project locked to those
inputs. External publications are authoritative for their stated scope but are
not assumed to have been transcribed correctly.

**Ordered flow:**

1. Capture each publication, erratum, and vector source by stable identity,
   locator, date/version, exact digest, acquisition method, and rights status.
2. Record the clauses, ambiguities, interpretations, exclusions, and independent
   review required for the intended algorithm or construction.
3. Author total Orange specification definitions with explicit integer/word,
   byte-order, canonical-decoding, parameter, and failure meaning.
4. Link every normative definition and vector to its source clause or recorded
   interpretation; do not invent missing standards meaning in build scripts.
5. Execute specification and vector cases through the reference semantics and
   record results separately from proof, implementation, or certification.
6. Obtain independent transcription/cryptography review and publish unresolved
   ambiguity as a blocker or explicit non-claim.

**Fail-closed outcomes:** Missing rights or digest, an unreviewed relevant
erratum, ambiguous normative intent, invalid vector, unsupported construct, or
reference-evaluation mismatch leaves admission blocked. The tool never changes
the standard source, selects a convenient interpretation silently, or labels a
local vector run as external validation.

**Evidence outputs:** Standards-provenance records, archived/acquisition records,
clause-to-definition map, specification modules, interpretation log, vector
results, reviewer identity/scope, and unresolved-item list.

**Non-goals:** This journey does not establish implementation refinement,
memory safety, target leakage, game-based security, ACVP certification, or
FIPS 140 validation.

**Completion test:** For every admitted definition, trace 100% of normative
source obligations to a clause, erratum, or explicit reviewed interpretation;
replay all admitted vectors from exact bytes. Remove or mutate one source
digest, clause link, erratum disposition, or vector result and prove admission
fails.

### J-03 — Implement, prove, and classify claims

**Actors and intent:** P-01 implements a selected specification; P-02 proves and
checks named properties; P-03 reviews mathematical and cryptographic intent.
The user needs independent claims, not a package-wide `verified` label.

**Entry conditions and trusted inputs:** An admitted specification and provenance
record, ratified Core and claim formats, selected proof/checker and leakage
models, explicit target-independent contracts, locked dependencies, and declared
resource budgets. Source modules, tactics, solvers, certificates, and foreign
contracts are untrusted until checked under their named policy.

**Ordered flow:**

1. Implement the specification with explicit memory, arithmetic, failure,
   secrecy, declassification, randomness, and foreign-boundary contracts.
2. Elaborate to canonical Core and produce separate obligations for refinement,
   safety, termination, equivalence, leakage, erasure, game security, and other
   requested claim families.
3. Use interactive proof or untrusted proof search to produce Orange proof terms
   or supported certificates; record timeout, `unknown`, and unavailable output
   without upgrading them to success.
4. Replay every machine-checkable basis with the authoritative checker and the
   independent checker over exact canonical bytes.
5. Attach tests, audits, external derivations, and assumptions as typed bases
   with their own scope and verification state.
6. Emit a claim matrix showing `satisfied`, `not_satisfied`, `unresolved`, or
   `unsupported` independently for every implementation, target, and property.

**Fail-closed outcomes:** Type, effect, ownership, termination, proof, certificate,
checker-agreement, resource, or policy failure prevents the affected claim from
being satisfied. An assumption alone cannot satisfy a proof-required claim;
one claim never implies another; an external theorem never becomes
kernel-checked by labeling.

**Evidence outputs:** Canonical Core, proof/certificate objects, theorem and
checker fingerprints, complete axiom/assumption/TCB closure, conformance and
negative results, claim records, and exact failure diagnostics.

**Non-goals:** A successful functional proof does not imply cryptographic
security, leakage resistance, ABI correctness, or final-object preservation.
Implementing one part of this journey does not authorize or imply any stronger
claim elsewhere in the journey.

**Completion test:** A permanent representative package must produce a mixed
claim matrix whose permitted bases replay identically in both checkers. Mutate
one proof, certificate, invariant, assumption closure, secret-dependent branch,
and unsupported solver result; each affected claim must move to the exact
non-success outcome without changing unrelated claims.

### J-04 — Build and inspect native artifacts

**Actors and intent:** P-01 builds a supported native artifact; P-02 inspects the
preservation chain; P-05 needs to see exactly which bytes and target profile each
claim covers.

**Entry conditions and trusted inputs:** Checked source/Core and claim records,
a frozen lock graph, ratified compiler/target/ABI/leakage profiles, declared
build inputs, supported feature tuple, and accepted proof/certificate formats.
The driver, optimization search, assembler/linker where retained, and build host
are outside the logical TCB unless their exact role is explicitly justified.

**Ordered flow:**

1. Resolve every content-addressed input from the locked local store or thick
   bundle and deny undeclared network or package-script execution.
2. Validate source/Core and all prerequisite claims before entering a
   claim-bearing pipeline.
3. Lower through stable IRs; check a theorem or per-artifact functional and
   leakage certificate at every assurance-preserving transition.
4. Emit Machine IR and native object bytes, then decode and check instructions,
   sections, constants, relocations, symbols, ABI behavior, dispatch, and final
   export digests.
5. Run reference/native differential, conformance, adversarial, hardware, and
   approved performance-budget tests for the exact tuple.
6. Inspect the claim/TCB graph and assemble the artifact plus required evidence;
   optional C output remains explicitly lower-assurance.

**Fail-closed outcomes:** Missing input, unsupported target/feature, failed
certificate, object mismatch, unmodeled instruction, ABI mismatch, leakage
failure, unexplained differential result, budget failure, or unapproved fallback
blocks the affected artifact and claim. No old source claim silently survives a
broken transition.

**Evidence outputs:** Input and build manifests, IR and certificate digests,
object inspection record, target/ABI/leakage identities, differential and
hardware results, generated ABI material, claim graph, TCB report, and artifact
digest.

**Non-goals:** A native build does not cover physical or speculative leakage,
unmodeled loader/OS/CPU behavior, or another target tuple. Portable C output
does not inherit native final-byte assurance.

**Completion test:** For every supported tuple, reproduce the expected object,
claim closure, and inspection record from declared inputs; reference and native
results must agree with zero unexplained mismatch. Mutate one transition
certificate, instruction byte, relocation, dispatch choice, and ABI symbol;
each build must reject before emitting a satisfied final-artifact claim.

### J-05 — Integrate through generated foreign interfaces

**Actors and intent:** P-04 integrates an Orange cryptographic artifact into a C
or Rust consumer; P-01 supplies the implementation and P-05 reviews the exposed
contract. The integrator needs misuse-resistant behavior and precise assurance
limits at the foreign boundary.

**Entry conditions and trusted inputs:** A supported object/library, one
machine-readable ABI contract, generated C header and Rust wrapper, exact claim
matrix, target/dispatch requirements, support metadata, and integration guide.
Foreign callers and their allocation, entropy, concurrency, and error handling
are untrusted unless the contract states otherwise.

**Ordered flow:**

1. Select an implementation only when the consumer target and feature profile
   satisfy its dispatch and platform preconditions.
2. Generate the header, wrapper, documentation, and object metadata from the
   same ABI definition and verify their shared identity.
3. Compile representative C and Rust consumers using the supported toolchain and
   link mode; expose no hidden allocator, RNG, TLS state, panic, or exception.
4. Enforce length, alignment, overlap, initialization, ownership, mutability,
   failure, entropy, zeroization, and feature contracts at the boundary.
5. Run official, negative, malformed, overlap, authentication-failure,
   cross-endian, dispatch, and interoperability cases.
6. Publish the exact consumer-visible claims, assumptions, unsupported uses,
   update channel, and failure-handling requirements.

**Fail-closed outcomes:** Unsupported tuple, contract/version mismatch,
misalignment, invalid overlap, short buffer, missing entropy, failed
authentication, unavailable implementation, or dispatch inconsistency returns
the specified typed failure without partial output or silent lower-assurance
fallback.

**Evidence outputs:** ABI identity and conformance record, generated artifact
digests, C/Rust consumer results, adversarial-call results, dispatch trace,
foreign assumptions, integration receipt, and consumer-facing claim matrix.

**Non-goals:** The wrapper cannot make arbitrary unsafe foreign code safe,
validate the caller's complete application, or strengthen the underlying
artifact's claims. Error handling must not reveal secret-dependent detail.

**Completion test:** On every supported tuple, independently compile and run the
published C and Rust consumers and pass the full ABI/adversarial corpus. Mutate
the contract identity, alignment, length, overlap, entropy, target feature, and
authentication result; each case must produce the documented failure with no
unapproved output or fallback.

### J-06 — Audit and replay evidence offline

**Actors and intent:** P-05 independently determines what was actually checked;
P-02 reviews proof/TCB closure and P-04 decides whether the artifact is suitable
for integration. The journey must work without the registry, source checkout,
or a live transparency service.

**Entry conditions and trusted inputs:** A self-contained thick evidence bundle,
an independently obtained artifact/release identity and verification root,
supported checker binaries or reproducible checker inputs, declared resource
limits, and a clean network-denied environment. Bundle bytes are hostile until
canonical parsing, path containment, digest, and signature checks pass.

**Ordered flow:**

1. Verify bundle format/version, canonical paths, manifest identity, signatures,
   content digests, and absence of undeclared or escaping files.
2. Enumerate source, package, model, proof, certificate, tool, build, artifact,
   external-evidence, axiom, assumption, and TCB closure before execution.
3. Replay proofs and certificates with authoritative and independent checkers,
   applying deterministic resource bounds and network denial.
4. Rebuild or validate the advertised artifacts from declared inputs and compare
   paths, modes, sizes, digests, target/ABI/leakage identities, and claim subjects.
5. Re-run required tests and inspect audit/lab/external records for identity,
   scope, validity, expiry, and applicability without presenting them as
   machine proofs.
6. Produce a machine-readable replay report and human trust/claim matrix listing
   every success, failure, non-claim, unresolved item, and invalidated dependent.

**Fail-closed outcomes:** Missing or extra bytes, path escape, digest or signature
mismatch, unsupported version, checker disagreement, failed proof/test/build,
expired external evidence, undeclared trust, resource exhaustion, or attempted
network access prevents the affected claim from replaying successfully. The
auditor may inspect partial diagnostics but receives no generic green verdict.

**Evidence outputs:** Bundle inventory, verification and replay receipts,
checker/build/test logs, separately computed artifact digests, claim/TCB/
assumption matrix, external-evidence status, and exact non-success diagnostics.

**Non-goals:** Offline replay does not prove the original standard transcription,
auditor independence, hardware assumptions, or physical security. Signature
verification establishes an authorized identity, not correctness.

**Completion test:** The owner unpacks the same bundle in two clean,
network-denied environments and obtains the declared claim and artifact results;
the record remains `solo-reviewed` and external reproduction is unavailable.
Mutation cases for every manifest/object class, an escaping path, an expired
external record, checker disagreement, and a network request must fail at the
correct boundary without a satisfied affected claim.

### J-07 — Update, deprecate, withdraw, or replace a profile

**Actors and intent:** P-04 keeps deployed packages and toolchains within a
supported policy; P-03 evaluates standards and algorithm changes; P-05 needs a
durable explanation of changed and invalidated claims.

**Entry conditions and trusted inputs:** An installed immutable release/profile,
authenticated update metadata with rollback/freeze protection, current support
and deprecation policy, standards/errata/dependency surveillance, exact claim
dependency graph, and authorized governance/release roles. A registry response
or larger version number is not trusted by itself.

**Ordered flow:**

1. Detect a new release, profile, erratum, dependency event, deprecation, or
   emergency withdrawal through authenticated metadata or reviewed surveillance.
2. Resolve its exact affected tuple, source rationale, authority, urgency,
   compatibility, changed TCB/assumptions, and downstream claim impact.
3. Review and publish an immutable replacement, migration path, support window,
   and evidence delta; security withdrawal may shorten ordinary notice without
   rewriting history.
4. Verify update metadata thresholds, version/freeze rules, asset signatures,
   dependency closure, and complete evidence before changing the active state.
5. Re-run affected proof, build, conformance, ABI, offline-replay, and integration
   journeys; unchanged evidence is reused only when its exact dependency closure
   remains valid.
6. Mark old versions supported, deprecated, yanked, withdrawn, or revoked as
   authorized; notify downstreams and preserve historical replay material.

**Fail-closed outcomes:** Unsigned metadata, rollback/freeze attempt, missing
dependency, incompatible format, invalid migration, unavailable required
evidence, ambiguous impact, or unapproved claim downgrade rejects the update.
A withdrawn unsafe profile is not silently retained or replaced with a weaker
one while preserving its old claims.

**Evidence outputs:** Update/withdrawal decision, authenticated metadata,
old-to-new dependency and claim delta, migration and compatibility results,
rerun evidence, support dates, notices, revocation/yank state, and historical
replay references.

**Non-goals:** An algorithm/profile update does not silently change language
semantics or retroactively strengthen old bundles. Yanking does not mutate
already published immutable package bytes.

**Completion test:** Exercise normal compatible update, incompatible migration,
standards erratum, urgent withdrawal, rollback, freeze, missing-evidence, and
compromised-key scenarios. Only authorized complete updates may activate; every
affected old claim must remain traceable, correctly invalidated or scoped, and
offline replayable according to its preserved status.

### J-08 — Respond to a vulnerability or invalidated claim

**Actors and intent:** P-04 and P-05 need containment, corrected artifacts, and
accurate downstream guidance; P-01, P-02, and P-03 diagnose implementation,
proof, compiler, leakage, API, or standards failures. Mature Release Engineering
and PSIRT authorities coordinate the response.

**Entry conditions and trusted inputs:** A private reporting or monitoring path,
authorized PSIRT continuity, exact affected tuple inventory, preserved evidence,
release/update/revocation authority, and incident handling rules. Reports,
attachments, logs, and suspected exploit material are untrusted and embargoed
until safely handled.

**Ordered flow:**

1. Acknowledge and contain the report without exposing it in public issues,
   commits, CI logs, or ordinary collaboration channels.
2. Reproduce safely and identify affected language, toolchain, package, target,
   leakage, artifact, checker, proof, standard, credential, and claim identities.
3. Stop publication, quarantine packages, revoke credentials or profiles, and
   mark dependent claims invalid or unresolved whenever impact cannot be bounded.
   Create an immutable status event naming every affected claim, the authorized
   actor, effective time, reason, prior record, and replacement or supersession
   link; bind the event to exact content and authority digests.
4. Correct every coupled artifact—semantics, implementation, proof, certificate,
   compiler, vectors, documentation, claims, manifests, and attestations—that
   the root cause affects.
5. Re-run the complete affected proof, build, conformance, leakage, integration,
   update, and offline-replay evidence with recorded owner review.
6. Publish an immutable advisory, replacement/revocation metadata, downstream
   notification, recovery instructions, and time-bounded retrospective when
   disclosure is safe.

**Fail-closed outcomes:** Unknown scope, unreviewed root cause, checker or build
disagreement, failed fix evidence, compromised release authority, incomplete
downstream inventory, or an unresolved critical/high finding keeps publication
and affected claims blocked. Wording changes alone cannot repair missing or
false assurance.

**Evidence outputs:** Access-controlled incident record, affected-claim graph,
immutable invalidation/revocation status event, reproduction and root-cause
evidence, containment/revocation log, corrected artifact set, independent
retest results, public advisory, notices, recovery record, and retrospective.

**Non-goals:** This proposed journey does not claim Orange has a staffed PSIRT,
response SLA, real product vulnerability, or release today. The provisional
Gate 0 claim-record schema does not implement invalidation or revocation, and
this document does not extend it. The journey also does not disclose private
reports, secrets, recovery factors, or exploit details prematurely.

**Completion test:** Run a synthetic tabletop for proof unsoundness, silent
miscompilation, target leakage failure, standards erratum, unsafe ABI behavior,
malicious package, and signing-key compromise. For each case, the affected
claim closure must be deterministically identified, publication blocked,
revocation/update material produced, corrected evidence independently replayed,
and the old state prevented from re-entering through rollback or fallback.

## 4. Coverage matrices

### Persona coverage

| Persona | Primary journey coverage | Supporting journey coverage | Counted |
| --- | --- | --- | ---: |
| P-01 | J-03, J-04 | J-01, J-02, J-05, J-08 | 1/1 |
| P-02 | J-03 | J-01, J-02, J-04, J-06, J-08 | 1/1 |
| P-03 | J-02 | J-03, J-07, J-08 | 1/1 |
| P-04 | J-01, J-05, J-07, J-08 | J-06 | 1/1 |
| P-05 | J-06, J-08 | J-04, J-05, J-07 | 1/1 |

Persona coverage is 5/5. This is design coverage; no external user has validated
a journey.

### Operation coverage

| Operation | Owning journeys | Counted |
| --- | --- | ---: |
| `install` | J-01 | 1/1 |
| `specify` | J-02 | 1/1 |
| `implement` | J-03 | 1/1 |
| `prove` | J-03 | 1/1 |
| `build` | J-04, J-05 | 1/1 |
| `inspect` | J-02, J-03, J-04, J-05, J-06, J-07, J-08 | 1/1 |
| `integrate` | J-05 | 1/1 |
| `update` | J-07, J-08 | 1/1 |
| `revoke` | J-07, J-08 | 1/1 |
| `offline-replay` | J-06, J-07, J-08 | 1/1 |

Operation coverage is 10/10. An operation is not complete until its owning
journey passes with real product artifacts in a clean owner-run environment.
External usability validation is a separate, currently unavailable status.

### Feature coverage

| Feature | Journey coverage | Counted |
| --- | --- | ---: |
| F-01 | J-01, J-02, J-03, J-06 | 1/1 |
| F-02 | J-01, J-03, J-07 | 1/1 |
| F-03 | J-02, J-03 | 1/1 |
| F-04 | J-03, J-04, J-05 | 1/1 |
| F-05 | J-03, J-04, J-05, J-06, J-07, J-08 | 1/1 |
| F-06 | J-02, J-03, J-06 | 1/1 |
| F-07 | J-01, J-03, J-04, J-06, J-08 | 1/1 |
| F-08 | J-04, J-05, J-06, J-08 | 1/1 |
| F-09 | J-04, J-05 | 1/1 |
| F-10 | J-04, J-05, J-08 | 1/1 |
| F-11 | J-01, J-04, J-06, J-07, J-08 | 1/1 |
| F-12 | J-02, J-03, J-04, J-05, J-06, J-07, J-08 | 1/1 |
| F-13 | J-02, J-06, J-07, J-08 | 1/1 |
| F-14 | J-01, J-04, J-06, J-07, J-08 | 1/1 |

Feature coverage is 14/14. This means every proposed feature group has a user
reason and an acceptance surface; it does not mean any feature exists.

## 5. Validation and change control

The journey set is structurally complete only while all of these are true:

1. P-01 through P-05 each have at least one primary journey.
2. All ten named operation IDs have at least one owning journey.
3. F-01 through F-14 each appear in the journey index and feature matrix.
4. J-01 through J-08 each define actors and intent, entry conditions and trusted
   inputs, an ordered flow, fail-closed outcomes, evidence outputs, non-goals,
   target gate, and a falsifiable completion test.
5. Every exact CLI, syntax, target, algorithm, proof, packaging, update, support,
   and release choice remains proposed until its decision gate closes.
6. A material charter, architecture, assurance, roadmap, threat, claim, or
   feature-matrix change updates the affected journeys in the same proposal.

D-023 assigns accountability to the project owner and removes unavailable
external validation from the implementation gate. The current coverage is 8/8
structurally specified journeys, 0/8 complete journeys, and 0/8 externally
validated journeys. The last count remains visible as an honest non-claim.
