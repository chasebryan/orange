# Gate 0 feature traceability

Status: proposed Gate 0 planning map; no feature or decision is ratified

Snapshot: 2026-07-11

## 1. Scope and interpretation

This matrix maps the fourteen feature groups in
[`PROJECT_CHARTER.md` section 5](PROJECT_CHARTER.md#5-in-scope-for-the-10-product)
to proposed accountability, prerequisite decisions, evidence, and objective
exit tests. It is the traceability record required by the Gate 0 exit criteria;
it is not evidence that Gate 0 or any later gate has closed.

The rows follow the charter bullets in source order, one row per bullet. The
identifiers `F-01` through `F-14` are stable planning identifiers. A later
change that splits, merges, removes, or materially changes a feature must retain
the old identifier as a tombstone, add replacement identifiers, and update the
charter, roadmap, decision register, and this matrix together.

The source section currently contains fourteen top-level bullets. Its exact
UTF-8 byte selection, beginning with the section 5 heading and ending before
the section 6 heading, has SHA-256 digest
`2ed9492d19141935e5ba143b1166d7121cb5ed0be855e3c9568c9b7463679a3a`.
Recompute it with:

```sh
sed -n '/^## 5\. In scope for the 1\.0 product$/,/^## 6\. Explicit non-goals for 1\.0$/p' \
  docs/PROJECT_CHARTER.md | sed '$d' | sha256sum
```

A changed digest requires full row-by-row review even when the bullet count and
order remain unchanged.

Workstream IDs refer to the proposed workstreams in
[`ROADMAP.md` section 4](ROADMAP.md#4-workstreams). Named authorities refer to
the proposed mature bodies in [`GOVERNANCE.md`](../GOVERNANCE.md#proposed-mature-authorities).
They are accountability targets, not staffed teams. Decision D-019 is not
ratified, no accountable person has been appointed for these rows, and the
Bootstrap Repository Steward cannot supply independent approval.

Every row also depends on these common gates:

- D-001 and D-002 preserve the directed mission and permanent-product lineage.
- D-017 must settle the project and package namespace.
- D-018 must settle source, documentation, generated-output, specification,
  vector, and contribution terms.
- D-019 must ratify governance, name accountable people, fund the work, and
  establish independent review and release authority.
- Applicable assurance stop-ship conditions in
  [`ASSURANCE.md`](ASSURANCE.md#8-stop-ship-conditions) must be clear.

## 2. Evidence and state vocabulary

Evidence classes are cumulative. A row may not close on one class when its exit
test requires several.

| Code | Evidence class | Minimum content |
| --- | --- | --- |
| `N` | Normative and decision evidence | Accepted decisions, editioned specifications, exact scope, models, and non-claims |
| `M` | Machine-checked evidence | Proof terms, checked certificates, theorem fingerprints, checker results, and complete assumptions |
| `C` | Conformance and adversarial evidence | Positive, negative, differential, mutation, resource-bound, interoperability, and hardware cases |
| `A` | Artifact and provenance evidence | Exact source and artifact digests, dependency closure, build records, manifests, SBOM/CBOM, and signatures |
| `X` | Independent external evidence | Identified review, audit, laboratory, standards, usability, or rebuild evidence with scope and validity |
| `O` | Operational evidence | Exercised incident, release, rollback, revocation, recovery, support, and migration procedures |

Decision-state cells use only the register values `accepted`, `directed`,
`proposed`, `investigate`, `blocked`, and `superseded`. A cell may list more than
one value when its controlling decisions differ. These values describe decisions,
not feature completion.

Trace state is separate:

- `gap` means at least one required mapping field is absent;
- `mapped` means the row has a proposed role, dependencies, evidence classes,
  falsifiable exit test, target gate, and current decision state; and
- `reviewed` means the appointed accountable person and an independent authorized
  reviewer have accepted the mapping at an identified revision.

Target gate names when the complete evidence can first be required. It is not a
current pass. Assurance outcomes such as `satisfied` and `unresolved` remain
reserved for claim records and are not planning-row states.

## 3. Current coverage and Gate 0 closure

| Measure | Current count | Interpretation |
| --- | ---: | --- |
| Charter feature groups represented | 14/14 | Every section 5 bullet has one source-ordered row. |
| Structurally mapped rows | 14/14 | Every row has all planning fields; this is document coverage only. |
| Accountable people appointed and accepting | 0/14 | Workstreams and proposed authorities are not people or appointments. |
| Independently reviewed row mappings | 0/14 | No authorized non-author human has attested to a row. |
| Feature exit tests evidenced | 0/14 | Product implementation and later-gate evidence do not exist. |
| Gate 0 exit criteria closed | 0/7 | This matrix partially addresses only the traceability criterion. |

The complete Gate 0 closure ledger remains:

| Criterion | Closure state | Evidence still required |
| --- | --- | --- |
| Advertised claims and proposed TCB have no unresolved contradiction | `open` | Accepted claim/TCB boundaries and independent consistency review |
| Every 1.0 feature maps to ownership, dependencies, evidence, and an exit test | `partial` | This 14/14 structural map, 14 appointed owners, and 14 independent mapping reviews |
| Exact scope and non-goals are signed off | `open` | Ratified charter and finite support envelope |
| Proof foundation, canonical formats, and leakage baseline are selected | `open` | Accepted D-006, dependent format decisions, and D-012 |
| Naming and license gates are closed | `blocked` | Owner/legal resolution of D-017 and D-018 |
| Staffing and funding support at least Phases 1 and 2 | `blocked` | Funded staffing plan and accepted D-019/D-022 obligations |
| External reviewers agree the plan is feasible | `open` | Identified independent reviews with scope, findings, and disposition |

## 4. Feature matrix

| ID | Charter feature group | Proposed accountability | Feature-specific prerequisites | Required evidence | Objective exit test | Decision state | Target gate | Trace state |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F-01 | Versioned language reference and mechanized semantics | W1; Language and Semantics Committee, with Assurance and TCB Board approval | D-003 product form; D-004 semantic strata; D-006 proof foundation; D-008 implementation languages; D-012 leakage baseline | `N/M/C`: editioned grammar and semantics, mechanized Core, theorem cross-references, ambiguity cases, and one conformance case per normative rule | Gate 1 has no normative TODO or undefined behavior; progress, preservation, erasure, and applicable noninterference statements check; every normative rule has a theorem or conformance case; independent ambiguity review passes | `proposed`; `investigate` | Gate 1 | `mapped` |
| F-02 | Deterministic parser, formatter, type checker, interpreter, documentation generator, and language server | W3; developer-tools lead under the Language and Semantics Committee | F-01; D-003 product form; D-004 Core boundaries; D-006 proof foundation; D-008 implementation languages | `N/C/A/X`: versioned interfaces and diagnostics, parser differential corpus, golden formatting and evaluation cases, resource limits, documentation-example builds, and external usability results | Repeated clean runs produce identical formatter, evaluator, and generated-document outputs; at Gate 2 the frontend matches the complete normative suite with bounded failures; at Gate 5 every published example builds and representative external users complete the specify-to-inspect journey without private help | `proposed`; `investigate` | Gates 2 and 5 | `mapped` |
| F-03 | Fixed-size sequences, arithmetic domains, algebraic data types, modules, and refinement-friendly contracts | W1; Language and Semantics Committee | F-01; D-004 semantic strata; D-006 proof foundation; D-015 flagship corpus | `N/M/C`: complete type and operator rules, mathematical and executable models, proof obligations, boundary and malformed cases, and permanent SHA- and ChaCha-like decision fixtures | Every stable operation has unambiguous semantics and a theorem or conformance case; arithmetic and layout edge cases fail or evaluate as specified; forbidden implicit integer, word, signedness, and endian conversions are rejected; Gate 1 flagship fixtures execute from exact standards provenance | `proposed`; `investigate` | Gate 1 | `mapped` |
| F-04 | Secrecy labels, regions, ownership, buffers, loops, zeroization, target features, and vector intrinsics | W1 and W4; Language and Semantics Committee, compiler/target lead, and Assurance and TCB Board | F-03; D-004 strata; D-005 claim model; D-006 proof foundation; D-010 compiler strategy; D-011 target envelope; D-012 leakage baseline; D-013 foreign boundary | `N/M/C/X`: memory and leakage semantics, target instruction classifications, positive and negative secrecy/aliasing cases, erasure obligations, final-byte preservation plan, and independent side-channel review | Gate 1 safety, erasure, and baseline noninterference statements check; Gate 3 preserves each advertised leakage profile to final bytes for every supported tuple and rejects a deliberately violating case | `proposed`; `investigate` | Gates 1 and 3 | `mapped` |
| F-05 | Functional correctness, safety, termination, equivalence, and constant-time noninterference claims | W2 and W7; Assurance and TCB Board | F-01, F-03, and F-04; D-005 claim model; D-006 and D-007 proof/checker choices; D-009 solver trust; D-010 compiler strategy; D-012 leakage baseline | `N/M/C/X`: versioned claim policies, complete assumption and TCB closure, proofs or checked certificates, exact negative outcomes, differential checker results, and independent logic review | Every advertised claim closes only with a permitted, valid basis; assumption-only and missing-certificate cases fail closed; authoritative and independent checkers agree on the full frozen corpus; target claims reach exact final artifacts | `proposed`; `investigate` | Gates 2 and 3 | `mapped` |
| F-06 | Probabilistic games and machine-checked security reductions | W2 and W5; Cryptography Review Board with Assurance and TCB Board approval | F-01, F-05, and F-12; D-004 Game stratum; D-006 and D-007 proof foundation/checker; D-009 solver trust; D-015 corpus | `N/M/C/X`: Game Core semantics, relation to shared pure definitions, explicit adversary and advantage bounds, checked reductions or scoped external proof records, negative cases, and cryptographer review | A permanent game/reduction case checks end to end with every assumption visible; each corpus security claim names exact algorithms, parameters, targets, theorem evidence, and concrete bound; no external result is presented as kernel-checked | `proposed`; `investigate` | Gates 2 and 4 | `mapped` |
| F-07 | Interactive proof terms and certificate-producing automation | W2; Assurance and TCB Board | F-01 and F-05; D-006 proof foundation; D-007 Proof IR and checker; D-008 implementation languages; D-009 solver trust | `N/M/C/X`: canonical Proof IR, mechanized checker soundness, two checker implementations, certificate formats, proof replay records, mutation/fuzz corpus, and external logic audit | Gate 2 soundness review closes; both checkers agree on the frozen corpus; claim-closing mode rejects unknown, timeout, malformed, unsupported, trusted-solver, and missing-certificate cases without satisfying a claim | `proposed`; `investigate` | Gate 2 | `mapped` |
| F-08 | Verified native compilation path and auditable connection to final object code | W4; compiler and target lead with Assurance and TCB Board approval | F-01, F-03, F-04, F-05, and F-07; D-004 semantic strata; D-005 claims; D-006 and D-007 proof/checker choices; D-008 implementation languages; D-010 compiler strategy; D-011 targets; D-012 leakage; D-013 ABI | `N/M/C/A/X`: semantics and validators for every stable IR, pass theorems or certificates, ISA/ABI models, object-byte inspection, differential/hardware results, and final-artifact provenance | Gate 3 proves or certificate-checks every supported pass; functional and leakage preservation reach final bytes for every tuple; a corrupted object is rejected; no assembler, linker, dispatch, or wrapper gap is hidden from the TCB; the approved per-profile performance floor is met | `proposed`; `investigate` | Gate 3 | `mapped` |
| F-09 | Reference and interoperability C backend with explicitly weaker assurance | W4 and W7; compiler/target lead with Assurance and TCB Board claim review | F-01, F-03, and F-05; D-005 claim model; D-010 compiler strategy; D-011 target envelope; D-012 leakage baseline; D-013 foreign boundary | `N/C/A`: deterministic backend contract, pinned C-toolchain matrix, differential and interoperability corpus, compiler and host assumptions, generated-output provenance, and explicit non-claim records | Repeated clean runs produce byte-identical C; every declared C toolchain matches the reference semantics across the frozen corpus; every report labels the weaker boundary and assumptions; no claim-bearing path or release silently selects the C backend as assurance-preserving | `proposed`; `investigate` | Gate 3 | `mapped` |
| F-10 | Stable C ABI artifacts, generated headers, and generated Rust bindings | W4; compiler and target lead, reviewed by Language and Semantics and Assurance authorities | F-03, F-04, and F-08; D-004 semantic strata; D-005 claims; D-008 implementation languages; D-011 target tuples; D-013 stable foreign boundary | `N/C/A/X`: one machine-readable ABI contract, generated header/wrapper/object records, layout and symbol inspection, adversarial foreign callers, and per-target ABI review | At Gate 3 one definition generates the contract, header, wrapper, and object metadata; C and Rust consumers compile and run; all length, alignment, overlap, failure, ownership, feature, entropy, and zeroization cases pass for each tuple with zero unexplained mismatch | `proposed` | Gate 3 | `mapped` |
| F-11 | Content-addressed packages, lock file, offline proof replay, and signed evidence bundle | W6 and W2; Release Engineering with Assurance and TCB Board approval | F-05 and F-07; D-005 claim closure; D-006 proof foundation; D-007 proof format; D-009 solver trust; D-014 package model; D-020 supply-chain target | `N/M/C/A/O`: canonical manifest/lock/bundle formats, immutable dependency graph, network-denied replay, substitution and rollback cases, signatures, provenance, and key-recovery exercises | Gate 2 replays every machine-checkable claim from a thick bundle or populated store with network denied; Gate 5 compromise/recovery exercises pass; any missing byte, digest, proof, trust entry, or required signature fails closed | `proposed`; `investigate` | Gates 2 and 5 | `mapped` |
| F-12 | Standard library and claim-complete flagship cryptography corpus | W5; Cryptography Review Board | F-01 through F-11 as applicable; D-004 semantic strata; D-005 claims; D-006 and D-007 proof/checker choices; D-009 solver trust; D-010 compiler; D-011 targets; D-012 leakage; D-013 ABI; D-015 exact corpus; D-016 validation posture; D-022 support policy | `N/M/C/A/X`: exact standards, errata, rights, vectors, specs, implementations, proofs, claim matrices, benchmarks, interoperability results, and independent cryptography review | At Gate 4 every admitted package has exact provenance and a complete per-implementation/target claim matrix; all official, negative, boundary, interoperability, and adversarial vectors pass; target leakage evidence exists for every promised profile; no unapproved fallback is reachable | `proposed`; `investigate` | Gate 4 | `mapped` |
| F-13 | NIST ACVP-compatible vector exchange and standards provenance metadata | W5 and W7; standards working group and Cryptography Review Board | F-12; D-005 claim model; D-014 package/evidence model; D-015 corpus; D-016 validation/certification posture; source-rights decision under D-018 | `N/C/A/X`: clause- and errata-linked provenance records, exact source/vector digests and rights, ACVP adapters, round-trip cases, external status records, and laboratory evidence when claimed | Every admitted source and vector has stable identity, digest, locator, rights, and review; adapter round trips preserve the applicable schema and semantics; Gate 4 vectors pass; public wording never calls local replay an ACVP/CAVP certificate or Orange FIPS validated | `proposed`; `blocked` | Gate 4 | `mapped` |
| F-14 | Reproducible, signed releases for supported hosts | W6 and W7; Release Engineering, PSIRT, and Assurance and TCB Board | All preceding features; D-008 implementation languages; D-011 host/target envelope; D-014 package model; D-019 release authority; D-020 supply-chain target; D-021 bootstrap policy; D-022 funded support policy | `N/M/C/A/X/O`: hermetic build closure, two independent rebuild attestations, signatures and transparency records, SBOM/CBOM, provenance and proof bundles, audits, and release/revocation/recovery drills | At Gate 7 two independently administered builders reproduce each supported artifact; network-disabled build and offline verification bind source, claims, SBOM/CBOM, provenance, proofs, and signatures; all stop-ship findings are closed; rollback, freeze, compromise, revocation, and disaster-recovery drills pass with multi-role sign-off | `proposed`; `blocked` | Gate 7 | `mapped` |

## 5. Review attestations

A mapping becomes `reviewed` only when an appointed accountable person and an
independent authorized reviewer attest to the same repository revision. A bot,
the author's second account, or a second pass by the author is not independent
review. Empty assignments below are deliberate evidence of the current gap.

| ID | Accountable person | Independent reviewer | Reviewed revision | Review date | Outcome |
| --- | --- | --- | --- | --- | --- |
| F-01 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-02 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-03 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-04 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-05 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-06 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-07 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-08 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-09 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-10 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-11 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-12 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-13 | Unassigned | Unassigned | — | — | `not reviewed` |
| F-14 | Unassigned | Unassigned | — | — | `not reviewed` |

## 6. Coverage and change control

The matrix is complete only when all of the following remain true:

1. `F-01` through `F-14` occur exactly once as feature rows and remain in the
   same order as the charter bullets.
2. Every row names at least one workstream, accountable authority or role,
   feature-specific prerequisite, evidence class, objective exit test, and
   target gate, decision state, and trace state.
3. Every referenced decision remains present in `DECISIONS.md`; a status change
   updates the affected row without rewriting history.
4. The recorded charter-section digest matches; a source change updates this
   matrix in the same accepted proposal after full semantic review.
5. A row changes from `mapped` to `reviewed` only with a complete attestation,
   and never merely because a schema, plan, synthetic fixture, or local test
   exists.

Gate 0 can close this traceability criterion only after accountable people and
reviewers accept the finite feature envelope and its dependencies, evidence
obligations, and exit tests. Completing this document supplies the map; it does
not supply that acceptance or any future-gate evidence.
