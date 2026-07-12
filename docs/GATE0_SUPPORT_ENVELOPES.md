# Solo development envelope

Status: directed active envelope under D-023; supersedes the institutional Gate
0 comparison for current planning

Snapshot: 2026-07-12

## 1. Decision boundary

The former `E-REF`, `E-FOCUS`, and `E-SCHEDULE` options all assumed teams of
28–45 people, outside auditors, laboratories, independent implementations,
external pilots, multiple release roles, and funded long-term support. Those
options are ineligible under the owner's direction that Orange must be planned
and built as a solo project without external participation.

`E-SOLO` is the active development envelope. It authorizes incremental
production-lineage work but does not promise that the complete former 1.0 scope
will be delivered on a particular date.

## 2. E-SOLO invariants

| ID | Invariant |
| --- | --- |
| S-01 | One owner can execute every active milestone without waiting for another person or organization. |
| S-02 | Owner approval is valid governance but is never labeled independent review. |
| S-03 | Missing proof, review, audit, lab, certification, or rebuild evidence limits the corresponding claim rather than unrelated implementation. |
| S-04 | Every merged compiler component belongs to the intended production lineage and includes deterministic behavior, diagnostics, tests, documentation, and non-claims. |
| S-05 | The project admits one bounded semantic, target, ABI, or cryptographic slice at a time instead of promising an unstaffed cross-product. |
| S-06 | Tests, assumptions, neighboring implementations, and generated output never inherit a stronger claim than their evidence supports. |
| S-07 | Source, dependency, evidence, and artifact identities remain explicit and replayable from declared inputs. |
| S-08 | No release claims independent rebuild, separation of duties, external validation, certification, or multi-person support while those events are unavailable. |
| S-09 | The absence of a repository license continues to block third-party contributions and distribution grants, not owner-authored local development. |
| S-10 | A future collaborative transition is explicit and does not retroactively relabel solo-produced evidence. |

## 3. Active scope

| Axis | E-SOLO scope |
| --- | --- |
| Governance | `@chasebryan` is sole owner, implementer, reviewer, merger, security contact, and decision authority. |
| Schedule | Dependency ordered, best effort, with no date or staffing promise. |
| Implementation language | Pinned Rust edition 2024 toolchain; standard library only for the initial compiler slice. |
| Initial product slice | Source files and byte spans, deterministic lexer, structured diagnostics, and `orangec` CLI. |
| Syntax and semantics | Pre-alpha and added in recorded slices; no complete grammar or normative Core exists yet. |
| Proof system | D-006 remains investigative and gates proof-bearing work only. |
| Targets and ABI | None selected or supported yet. |
| Leakage claims | Unsupported until D-012 and the affected target model are implemented. |
| Cryptography corpus | None implemented; candidates remain research inputs rather than promises. |
| Dependencies | Exact owner admission; no third-party Rust crates in the initial compiler. |
| Releases | No product release currently authorized. A later solo preview requires an explicit release record and solo-produced provenance. |
| Support | Best effort with no SLA, LTS, compatibility, or migration promise. |
| External evidence | Unavailable and not assumed. Independently checkable artifacts remain a design goal distinct from independent human review. |

## 4. Claim boundary

The active envelope can produce implementation and test evidence. It cannot
currently produce organizational independence, external validation,
certification, multi-party key custody, separate PSIRT/release roles, or
independent rebuild evidence.

Machine-checkable proof may later establish a technical theorem even when the
proof and checker were selected by the owner. Its claim record must separately
show that independent human review is unavailable. Conversely, a second owner
implementation can support differential testing but is not an independently
authored implementation.

Every unsupported or unavailable dimension remains visible. It cannot be
omitted merely because no external participant is expected.

## 5. Resource model

The available internal capacity is one owner. There is no credible FTE-year,
currency, or completion-date estimate. Work is sequenced to reduce simultaneous
maintenance:

1. compiler source model and lexer;
2. parser and grammar;
3. typed semantic core and evaluator;
4. proof and claim boundary;
5. one compiler output path;
6. one target and ABI profile;
7. one standards-sourced cryptographic package; and
8. package, tooling, and preview-release capabilities.

The owner may narrow a slice when it cannot be maintained, but cannot keep a
broader public claim after removing its evidence.

## 6. Exit and revisit rules

`E-SOLO` remains active until explicit owner direction supersedes it. It does
not end because an issue, message, or unsolicited contribution appears.

A collaborative envelope may be considered only after participants and their
actual roles exist. At that point the owner records access, authority, review
scope, conflicts, and the effective date. External evidence collected later
applies only to the exact artifacts and revisions it covers.

The former institutional options remain historical planning context in Git.
They are not current requirements, schedules, staffing targets, or reasons to
delay solo compiler development.
