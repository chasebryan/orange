# Gate 0 support-envelope options

Status: proposed Gate 0 comparison; no option, target, corpus, schedule, or
resource commitment is selected

Snapshot: 2026-07-11

## 1. Decision boundary

Orange 1.0 needs a finite envelope whose complete assurance obligations can be
staffed, reviewed, operated, and supported. This document compares three
candidate envelopes against one fixed set of axes:

- `E-REF`: the current reference dual-target and full recommended-corpus plan;
- `E-FOCUS`: one native assurance target and a reduced, capability-complete
  corpus; and
- `E-SCHEDULE`: exactly the `E-REF` scope delivered with lower peak concurrency
  and a longer calendar.

The options are decision inputs, not accepted architecture. Specific targets and
corpus entries inside a row are candidate assumptions whose selection still
requires Gate 0 evidence and an accepted Orange Enhancement Proposal. This
document does not resolve D-006, D-011, D-012, D-015, D-017, D-018, D-019, or
D-022; authorize product implementation; commit funding or procurement; engage
a laboratory; create an account; or accept a license.

The resource bands are deliberately low-confidence capacity estimates. They are
not budgets, prices, hiring plans, delivery promises, or evidence that qualified
people and external capacity exist. Exact cost cannot be credible before the
proof foundation, target/leakage models, corpus, lab scope, governance, support
window, and named staffing plan are frozen.

Current decision state is `inconclusive`: Gate 0 has closed 0/7 exit criteria,
feature ownership and independent mapping review remain 0/14, and none of the
three option evidence packets exists.

## 2. Non-waivable invariants

Scope and time may change. Assurance depth does not silently change with them.

| ID | Invariant required in every eligible option |
| --- | --- |
| G-01 | The complete Gate 0 closes before product implementation; D-001's mission and D-002's permanent-product lineage remain intact. |
| G-02 | Public claims are independent, artifact-scoped, assumption/TCB-complete, and fail closed; tests, assumptions, neighboring implementations, or portable C never inherit stronger assurance. |
| G-03 | A small authoritative checker, an independently implemented checker, certificate-only solver success, malformed/mutation testing, and independent logic review remain required. |
| G-04 | Every advertised native functional and leakage claim reaches exact final object bytes for its named target, ABI, feature, and leakage tuple; no assembler, linker, dispatch, wrapper, or fallback gap is hidden. |
| G-05 | Canonical content addressing, immutable locks, complete thick bundles, network-denied offline replay, explicit trust inventories, and independently checkable evidence remain required. |
| G-06 | Every admitted cryptography package retains exact standards/errata/rights provenance, a complete claim matrix, negative and interoperability tests, target leakage evidence, and independent cryptography review. |
| G-07 | Product releases remain immutable, signed, provenance- and SBOM/CBOM-bearing, bit-identically rebuilt by two independent builders, rollback/freeze protected, and recoverable through rehearsed multi-role procedures. |
| G-08 | Independent authorship/review, critical-subsystem bus factor of at least three, separate release and PSIRT rotations, two independent assurance organizations, and role separation remain release gates. |
| G-09 | A staffed and exercised PSIRT, update/withdrawal/revocation paths, downstream notification, disaster recovery, and funded support for the ratified window remain mandatory. |
| G-10 | Every stop-ship condition in `ASSURANCE.md` remains non-compensable. Only non-assurance operational gates may use a named, approved, expiring, disclosed exception with a compensating control. |

An option that cannot preserve all ten invariants is ineligible. It must narrow
the advertised product further, extend the schedule, add capacity, or remain
`inconclusive`; a weighted score cannot compensate for a failed invariant.

## 3. Fixed-axis comparison

`PQ-SIG-1` below means exactly one post-quantum signature family selected under
D-015. The reference options reserve its capacity but are not exact or eligible
until Gate 0 fills the slot, for example by selecting ML-DSA or SLH-DSA on
evidence. The placeholder selects neither.

| Axis | `E-REF` — reference dual-target | `E-FOCUS` — focused single-target | `E-SCHEDULE` — full scope, longer calendar |
| --- | --- | --- | --- |
| A-01 Language and semantics | Full F-01 through F-04 surface, including Spec/Impl/Game/Proof strata | Identical language surface; fewer target/corpus fixtures do not remove semantics | Identical to `E-REF` |
| A-02 Proof and claim system | Orange Proof IR, authoritative and independent checkers, checked certificates, all claim families | Identical; no reduction in proof, checker, claim, or independent-review obligations | Identical to `E-REF` |
| A-03 Host-tool releases | Linux x86-64/AArch64, macOS AArch64, and Windows x86-64 candidate rows | Same host-tool rows; host availability never implies native assurance on that host | Identical to `E-REF` |
| A-04 Native assurance targets | x86-64 Linux/SysV and AArch64 Linux/AAPCS64 | x86-64 Linux/SysV only; AArch64 native claims are `unsupported` for 1.0 | Identical to `E-REF` |
| A-05 Target feature profiles | Baseline plus selected crypto/SIMD profiles on both native targets | Baseline plus the AES-NI/PCLMULQDQ profile needed by the retained accelerated AES-GCM implementation on x86-64 | Identical to `E-REF` |
| A-06 Leakage scope | D-012 architectural two-run model for every admitted implementation/target profile; stronger speculative/physical profiles excluded unless separately ratified | Same model and exit tests over one target; fewer claim-matrix cells, not weaker evidence | Identical to `E-REF` |
| A-07 ABI and interoperability | Stable generated C ABI/Rust wrapper on both native tuples; deterministic lower-assurance portable C | Same boundary and adversarial tests on x86-64; portable C remains lower assurance | Identical to `E-REF` |
| A-08 Cryptography corpus | SHA-256/512; ChaCha20-Poly1305; HMAC/HKDF; AES-GCM portable plus selected acceleration on both targets; X25519 and Ed25519; ML-KEM; `PQ-SIG-1` | SHA-256; ChaCha20-Poly1305; HMAC/HKDF; AES-GCM portable plus one x86-64 accelerated profile; X25519; ML-KEM | Identical to `E-REF` |
| A-09 Corpus capability coverage | Hash, symmetric/AEAD, KDF/MAC composition/game linkage, hardware dispatch, field/curve, KEM, and PQ signature | Retains hash, symmetric/AEAD, KDF/MAC/game linkage, accelerated dispatch, field/curve, and post-quantum KEM; defers SHA-512, Ed25519, and PQ signatures | Identical to `E-REF` |
| A-10 Package and registry | Content-addressed packages, immutable lock, thick bundle/local store, public TUF-style registry, quarantine/revocation/recovery | Identical; registry and compromise/recovery exercises are not traded away | Identical to `E-REF` |
| A-11 Standards and validation | Exact provenance and ACVP-compatible exchange where applicable; no certification overclaim; lab pre-assessment only if a certificate-bearing module is separately required | Same rigor for every retained family; no certificate-bearing module assumed | Identical to `E-REF` |
| A-12 Release and supply chain | Network-disabled reproducible builds, two rebuilders, signatures/transparency, SBOM/CBOM, provenance, updates and recovery | Identical over one native target and the reduced corpus | Identical to `E-REF` |
| A-13 User product | All J-01 through J-08 journeys, full developer tools/docs, external pilots, conformance program | All journeys and tools remain; corpus and native-target inventories are narrower and visible | Identical to `E-REF`, completed later |
| A-14 Governance, PSIRT, and support | Release-capable governance, independent assurance, funded five-year full LTS plus proposed two-year critical-only tail | Identical; smaller scope does not permit solo release, weaker PSIRT, or unfunded support | Identical to `E-REF` |

The matrix fixes fourteen axes, exceeding the nine-axis minimum. A future change
adds an axis or versions the comparison; it cannot bury a scope difference in
prose.

## 4. Option specifications

### E-REF — Reference dual-target envelope

**Scope:** Both proposed D-011 native assurance tuples, all host-tool rows, the
full D-015 recommended coverage set including one unresolved `PQ-SIG-1` slot,
selected acceleration on both targets, the public registry, and all fourteen
feature/eight journey surfaces.

**Resource band:** 35–45 peak internal FTE, 78–90 months from Gate 0 start through
Gate 7, and 135–245 internal build FTE-years. The roadmap center remains 42 peak
FTE and approximately 84 months. Phase 0 still starts at 14–18 FTE for about nine
months. External audits, labs, independent organizations, and the Phase 8/LTS
tail are outside the FTE-year band.

**Dependencies:** Accepted D-006 proof-foundation evidence; exact D-011 target and
host tuples; D-012 leakage semantics and instruction-classification processes for
both targets; D-013 ABI; D-015 selection of every corpus member including
`PQ-SIG-1`; D-016 validation scope; D-017/D-018 legal gates; D-019 governance,
staffing, and funding; D-020 supply-chain plan; and D-022 support capacity.

**External capacity:** Logic/kernel, compiler/object-path, applied-crypto/API,
side-channel, and supply-chain reviews; hardware and lab access for both target
families and selected feature profiles; two independent assurance organizations;
an independent checker/frontend effort; two rebuild witnesses; external pilots;
and separate release/PSIRT operations.

**Principal risks:** The target × feature × implementation × claim × corpus
cross-product multiplies compiler, binary, lab, regression, and review work.
Target or standards drift can invalidate parallel evidence. Simultaneous compiler,
corpus, registry, and product work creates the highest peak hiring and integration
risk.

**Removal or deferral consequences:** None are planned inside the option. An
unfilled `PQ-SIG-1`, unavailable target/lab capacity, incomplete claim-matrix
cell, or missing independent role makes `E-REF` ineligible; the slot or target
must be explicitly removed through the focused option or another versioned OEP.

**Falsifiable selection evidence:** A resource-loaded dependency plan at the
named scale; one final-byte functional/leakage vertical slice on each target;
per-family standards/rights/workload/maintainer records; complete D-006 results;
hardware/lab and independent-review availability; registry and recovery design
exercises; named role/separation coverage; and an independent feasibility review
of the exact envelope manifest.

### E-FOCUS — Focused single-target envelope

**Scope:** x86-64 Linux/SysV is the sole 1.0 native assurance target candidate.
Host tools remain available on the same rows as `E-REF` but carry no native
assurance claim for macOS, Windows, or AArch64. The corpus is exactly SHA-256,
ChaCha20-Poly1305, HMAC/HKDF, AES-GCM portable plus one x86-64
AES-NI/PCLMULQDQ profile, X25519, and ML-KEM. The language, checker, claims,
registry, release, PSIRT, support, tools, and journey gates remain complete.

**Resource band:** 31–37 peak internal FTE, 84–102 months, and 130–220 internal
build FTE-years. Phase 0 remains 14–16 FTE for about nine months. The range does
not promise an earlier release: core semantics/checker and independent-assurance
critical paths do not shrink with corpus count, and host tools plus the public
registry remain. External work and the support tail are excluded.

**Dependencies:** The same proof, semantics, governance, legal, release, and
support decisions as `E-REF`; accepted D-011/D-015 scope changes naming x86-64
and the exact reduced corpus; D-012 coverage for baseline plus the selected x86
feature profile; evidence that AES-GCM supplies the permanent intrinsic/dispatch
fixture; and proof that the retained set still exercises every charter workload
and required claim family.

**External capacity:** All review and operational classes required by `E-REF`,
but native binary and side-channel hardware/lab matrices cover one target family.
Independent checkers, two assurance organizations, two rebuild witnesses,
release/PSIRT separation, external pilots, and the funded support organization
do not shrink away.

**Principal risks:** The option may optimize for one platform and under-test
portability, defer important deployment algorithms, or create expensive 1.1
target/corpus re-entry. A smaller corpus can miss proof/compiler ergonomics that
the removed SHA-512, Ed25519, or signature cases expose. Cross-platform host
tools may be mistaken for cross-platform native assurance unless claims and UI
remain explicit.

**Removal or deferral consequences:** AArch64 native assurance, SHA-512,
Ed25519, and every post-quantum signature package/claim are outside 1.0 and must
report `unsupported`, not disappear. Their first possible re-entry is a dated
post-1.0 profile/OEP after target, corpus, claim-matrix, lab, maintenance, and
support evidence passes. The charter's generic workload categories remain
covered, but its current dual-target proposal, D-011/D-015, roadmap examples,
traceability, journeys, architecture, assurance, threat model, and support
matrices require a coupled accepted update.

**Falsifiable selection evidence:** One x86-64 final-byte functional/leakage
vertical slice including AES dispatch; complete estimates and provenance for all
six retained corpus groups; a coverage argument showing every F-ID/J-ID and
charter workload remains testable; explicit `unsupported` records and user-impact
review for each deferral; named staffing/external capacity at the focused band;
and independent confirmation that removed matrix cells, not assurance rigor,
produce the claimed reduction.

### E-SCHEDULE — Full scope with extended calendar

**Scope:** Byte-for-byte the same envelope manifest, targets, feature profiles,
corpus including `PQ-SIG-1`, registry, claims, releases, journeys, audits, PSIRT,
and support obligations as `E-REF`. Only phase concurrency and dates differ.

**Resource band:** 28–34 peak internal FTE, 108–126 months, and 195–320 internal
build FTE-years. Phase 0 still requires 14–18 FTE for about nine months. The
lower peak uses a higher sustained-load/carrying-cost factor, so total FTE-years
may exceed `E-REF`; elapsed time is not free capacity. External work and the
support tail remain separate.

**Dependencies:** Every `E-REF` dependency plus a revised dependency-based phase
calendar, quarterly role-loading and hiring assumptions, independence/separation
matrix, external-review/lab reservation plan, standards/target/toolchain drift
policy, retention/succession plan, and estimate-to-complete recalibration method.

**External capacity:** The same reviews, two-target hardware/lab coverage,
rebuilders, pilots, PSIRT/release roles, and assurance organizations as `E-REF`,
held or reacquired across a longer period. Longer duration adds likely repeated
standards, target, dependency, and audit review.

**Principal risks:** Attrition and knowledge loss, proof/toolchain version churn,
standards and CPU errata, repeated audits/lab work, integration rework, delayed
external feedback, and a long pre-LTS period can raise total cost despite lower
peak staffing. Too little concurrency can starve compiler/corpus co-design and
turn permanent integration into a late big bang.

**Removal or deferral consequences:** No scope or claim may move. If capacity
cannot sustain independent authorship, three-person critical-subsystem coverage,
release/PSIRT separation, target/corpus co-development, or external review, the
calendar pauses or capacity rises; the option does not silently become
`E-FOCUS`. The current 84-month roadmap and every phase range must be replaced by
an accepted process/scope OEP before this option is selectable.

**Falsifiable selection evidence:** A phase-by-phase critical-path schedule that
fits every `E-REF` artifact and gate into 108–126 months; role loading that never
drops below independence/separation minima; explicit drift/retest/rework and
attrition contingency; external capacity across the longer window; and an
independent cost/schedule challenge showing the lower peak does not create an
unfunded or technically impossible dependency sequence.

## 5. Resource model

### 5.1 Derivation

The roadmap supplies three anchors: Gate 0 is about nine months at 14–18 FTE;
the reference program is approximately 84 months; and the reference peak is
35–45 people with a role-plan center of 42. For each option:

```text
internal_build_FTE_years =
    phase0_average_FTE × 0.75
  + peak_FTE × post_phase0_years × average_to_peak_load_factor
```

| Option | Phase 0 FTE | Peak FTE | Calendar | Post-Gate-0 load factor | Resulting internal FTE-years |
| --- | ---: | ---: | ---: | ---: | ---: |
| `E-REF` | 14–18 | 35–45 | 78–90 months | 0.62–0.75 | 135–245 |
| `E-FOCUS` | 14–16 | 31–37 | 84–102 months | 0.60–0.72 | 130–220 |
| `E-SCHEDULE` | 14–18 | 28–34 | 108–126 months | 0.80–0.92 | 195–320 |

The endpoints combine low assumptions with low and high with high, then round
outward. The intervals overlap because current uncertainty is larger than the
apparent difference between options. They must not be used to claim that one
option is cheaper.

### 5.2 Illustrative peak role loading

These centers are concurrent FTE roles, not unique people or appointments.

| Function | `E-REF` center | `E-FOCUS` center | `E-SCHEDULE` center |
| --- | ---: | ---: | ---: |
| Program, governance, standards | 3 | 3 | 3 |
| Language and semantics | 5 | 5 | 4 |
| Proof, checker, metatheory | 6 | 5 | 5 |
| Compiler, targets, object path | 8 | 6 | 6 |
| Applied cryptography and corpus | 6 | 4 | 4 |
| Conformance, assurance, product security | 5 | 4 | 4 |
| Build, registry, release, supply chain | 4 | 4 | 3 |
| Developer tooling, docs, adoption | 5 | 3 | 3 |
| **Illustrative center** | **42** | **34** | **32** |

The authoritative/independent checkers cannot be one nominal person; target and
final-object work needs compiler, ISA/ABI, and leakage expertise; applied
cryptography and standards review are distinct from general engineering; and
release engineering and PSIRT need separate rotations. Every critical subsystem
still needs at least three capable maintainers before 1.0, even when its average
FTE load is below three.

### 5.3 Exclusions and uncertainty

Internal build FTE-years exclude external audit/lab effort, independent
organizations not embedded as staff, hardware/procurement, legal/naming/IP work,
and the Phase 8 support tail. No credible currency estimate exists without
location, compensation, contracting, lab, hardware, insurance, contingency, and
funding assumptions that this document is not authorized to make.

D-022's proposed five full-support years plus two critical-only years require a
separate measured model:

```text
LTS_FTE_years =
    5 × measured_full_support_average_FTE
  + 2 × measured_critical_only_average_FTE
  + audit_and_revalidation_bursts
```

There is no evidence for those averages. Confidence in all build bands is low,
with planning error plausibly at least ±30%, because no accepted work breakdown,
measured proof throughput, frozen foundation, exact target/leakage model, final
corpus, lab scope, registry design, or named team exists.

## 6. External and operational capacity

| Capacity | `E-REF` | `E-FOCUS` | `E-SCHEDULE` |
| --- | --- | --- | --- |
| Logic/kernel and checker audit | Required, independent | Same | Same, potentially repeated for drift |
| Compiler/object-path audit | Two target families | One target family | Same as `E-REF`, later and potentially repeated |
| Applied cryptography/API review | Full corpus | Six retained corpus groups | Full corpus, with re-review for standards drift |
| Side-channel lab/hardware | x86-64 and AArch64 plus selected feature profiles | x86-64 plus AES-NI/PCLMULQDQ profile | Same as `E-REF` over a longer retention window |
| Supply-chain/registry review | Full public registry and release system | Same | Same, potentially repeated |
| Independent implementations | Independent checker and frontend | Same | Same |
| Rebuild witnesses | Two independently administered builders | Same | Same |
| Operations | Separate release and PSIRT rotations; update/revocation/recovery drills | Same | Same across a longer program |
| External pilots | Full target/corpus envelope | Reduced native/corpus envelope, all journeys | Full envelope, later |
| Sustaining capacity | Funded D-022 window after 1.0 | Same promised window | Same promised window, starting later |

Written statements of work, availability, conflicts, methods, hardware, review
scope, retest triggers, and scheduling are selection evidence. This table makes
no procurement or funding commitment.

## 7. Feature coverage

Coverage states describe option scope, not implementation status. Every feature
is currently unimplemented and unreviewed.

| Feature | `E-REF` | `E-FOCUS` | `E-SCHEDULE` |
| --- | --- | --- | --- |
| F-01 | Full language reference and mechanized semantics | Unchanged | Same scope, later |
| F-02 | Full frontend and developer tools | Unchanged | Same scope, later |
| F-03 | Full arithmetic/type/module contract surface | Unchanged; reduced corpus still exercises word, field, curve, and PQ domains | Same scope, later |
| F-04 | Memory, secrecy, erasure, target features, and intrinsics across two targets | Same semantics and rigor; one target and one accelerated AES profile | Same scope, later |
| F-05 | All named claim families and fail-closed closure | Unchanged; fewer implementation-target claim cells | Same scope, later |
| F-06 | Integrated games/reductions across selected claims | Retains complete HMAC/HKDF-linked and other selected reductions | Same scope, later |
| F-07 | Proof IR, two checkers, certificates, mutation/fuzzing, external audit | Unchanged | Same scope, later |
| F-08 | Verified native/final-object path for two tuples | Identical final-byte obligations for x86-64 only | Same scope, later |
| F-09 | Deterministic explicitly lower-assurance C backend | Unchanged | Same scope, later |
| F-10 | Generated C/Rust ABI evidence for two tuples | Same contract and adversarial tests on x86-64 | Same scope, later |
| F-11 | Packages, locks, signed bundles, offline replay, registry operations | Unchanged | Same scope, later |
| F-12 | Full named corpus and complete cross-product claim matrices | Reduced named membership; complete matrices remain mandatory for retained members/variants | Same scope, later |
| F-13 | Provenance and ACVP-compatible exchange for full corpus | Same rigor over retained corpus | Same scope, later |
| F-14 | Reproducible signed host releases carrying both target profiles | Same host-release rigor carrying one native assurance profile | Same scope, later |

Feature coverage is 14/14 for all three options. `E-FOCUS` narrows F-04, F-08,
F-10, F-12, F-13, and F-14 scope but does not weaken their acceptance rules.

## 8. User-journey coverage

| Journey | `E-REF` | `E-FOCUS` | `E-SCHEDULE` |
| --- | --- | --- | --- |
| J-01 | Install every host tool with both target-data profiles | Same host installations with one target-data profile | Same scope, later |
| J-02 | Import/specify every full-corpus member | Exact workflow over the retained corpus | Same scope, later |
| J-03 | Implement/prove/classify all packages and variants | Same claim rigor over fewer packages/variants | Same scope, later |
| J-04 | Build/inspect every artifact on both assurance targets | Same completion test on x86-64 only | Same scope, later |
| J-05 | C/Rust integration on both tuples | Same adversarial integration on x86-64 | Same scope, later |
| J-06 | Offline replay of the complete full-scope release | Same replay completeness over the smaller graph | Same scope, later |
| J-07 | Update/withdraw all target and crypto profiles | Same operations over one target and reduced crypto profile | Same scope, later |
| J-08 | Incident response across both targets and full corpus | Same PSIRT/invalidation/recovery requirements over the reduced inventory | Same scope, later |

Journey coverage is 8/8 for all options. Selection still requires representative
external users to complete the exact chosen journeys without private help.

## 9. Deferral and coupled-change ledger

| Change | Option | Required disposition if selected |
| --- | --- | --- |
| Select two native tuples | `E-REF`, `E-SCHEDULE` | Accept D-011 with exact ISA/ABI/feature models, hardware/lab evidence, costs, and final-byte cases |
| Select x86-64 only | `E-FOCUS` | Accept D-011 replacement; mark AArch64 native assurance `unsupported` for 1.0 and name re-entry evidence |
| Fill `PQ-SIG-1` | `E-REF`, `E-SCHEDULE` | Accept exact D-015 family after standards/rights/workload/maintainer evidence; placeholder cannot survive ratification |
| Reduce corpus | `E-FOCUS` | Accept exact D-015 set; mark SHA-512, Ed25519, and PQ-signature packages/claims `unsupported`; update public examples and profile metadata |
| Retain one acceleration fixture | `E-FOCUS` | Accept x86 AES-GCM profile and prove it still exercises vector/intrinsic/dispatch/final-byte claims |
| Extend calendar only | `E-SCHEDULE` | Replace the 84-month horizon and every phase range with a resource-loaded dependency schedule; scope manifest must equal `E-REF` |
| Preserve public registry | All | Keep D-014/Gate 5 registry, compromise, quarantine, revocation, and recovery obligations |
| Preserve support window | All | D-022 must be funded and staffed; no option silently shortens a published promise |

Any accepted option must update `PROJECT_CHARTER.md`, `ROADMAP.md`, D-011,
D-012, D-015, D-019, D-022, `ARCHITECTURE.md`, `ASSURANCE.md`,
`GATE0_TRACEABILITY.md`, `USER_JOURNEYS.md`, threat/control records, and
support/release material as one digest-bound change. A section 5 charter edit
changes the traceability source digest and triggers all fourteen mapping reviews.

## 10. Falsifiable selection evidence

Each option needs an immutable evidence epoch containing:

1. an exact envelope manifest with every feature, journey, claim family,
   algorithm/variant, host, native target/ABI/feature tuple, leakage profile,
   package/registry boundary, validation posture, and support window—no wildcard
   or inherited claim;
2. a 14/14 feature and 8/8 journey disposition in which every current charter
   obligation is included, explicitly unsupported, or deferred with user impact,
   first eligible future version, and re-entry evidence;
3. named accountable people, dependencies, evidence classes, exit tests, phase
   loading, independence/separation matrix, and supported capacity for every
   included item; a workstream name is not an owner;
4. complete D-006 results and proof/checker staffing evidence;
5. one final-byte functional/leakage vertical slice, ISA/ABI model assessment,
   hardware inventory, lab plan, and workload/resource estimate per proposed
   native target;
6. per-family standards/errata/rights sources, reference implementation,
   complete draft claim matrix, maintainer ownership, target benchmark, and
   proof/compiler estimate;
7. a dependency DAG, critical path, peak/FTE-year range, estimate assumptions,
   external review/lab plan, sustaining/support model, and contingency;
8. independent claim/TCB consistency, program-feasibility, external-capacity,
   and representative-user reviews bound to the exact manifest digest;
9. closed naming/license gates and ratified governance with authority to decide,
   without treating this document as legal, funding, or procurement approval;
   and
10. raw evidence and dissent sufficient to record every eligibility gate as
    `pass`, `fail`, `unresolved`, or `unsupported`.

An option is eligible only if every non-waivable invariant and required evidence
item passes. Eligible options are compared per axis for mission fit, assurance
completeness, user coverage, resource demand, calendar, external capacity, and
residual risk. No weighted total lets speed or price offset a soundness,
final-byte, independence, release, PSIRT, or support failure.

The comparison conclusion is `recommend_E_REF`, `recommend_E_FOCUS`,
`recommend_E_SCHEDULE`, `tie`, or `inconclusive`. A recommendation selects
nothing by itself. `tie`, `inconclusive`, missing qualified review, stale or
asymmetric evidence, or inability to staff/fund the exact envelope leaves Gate 0
open.

Only an authorized accepted Standards OEP, citing the exact evidence epoch, can
ratify one envelope and its coupled D-011/D-012/D-015/D-019/D-022 changes. The
current recommendation, document order, sunk implementation, solo-steward
preference, or Codex review is not a tie-breaker or independent approval.

## 11. Structural completion and current state

This comparison is structurally complete only while:

- options are 3/3 and none is labeled selected;
- fixed comparison axes are at least 9; this version defines 14;
- every option contains scope, resource band, dependencies, external capacity,
  risks, removal/deferral consequences, and falsifiable selection evidence;
- F-01 through F-14 and J-01 through J-08 are mapped for all three options;
- peak FTE, calendar, internal FTE-years, exclusions, uncertainty, and derivation
  are explicit;
- every non-waivable gate, coupled document change, deferral, re-entry condition,
  evidence requirement, tie, and inconclusive path remains visible; and
- repository validation passes without implying an option is feasible or chosen.

Current implementation, option review, and selection evidence are all 0/3. The
comparison defines what must be decided and measured; it supplies no architecture
or resource commitment.
