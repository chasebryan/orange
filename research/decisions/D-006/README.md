# D-006 proof-foundation laboratory

Status: non-product research; `d006-v0.3` epoch
`d006-e-c7b6648ae3988234297f` has run both candidates under the full plan.
Its results are contributor-produced and unreviewed, and its conclusion is
`inconclusive` until the owner's DS-07 tasks and reviews exist. Nothing is
selected.

This directory holds the inputs for the owner-executable comparison of Rocq
and Lean 4 defined by
[`PROOF_FOUNDATION_DECISION_SUITE.md`](../../../docs/PROOF_FOUNDATION_DECISION_SUITE.md).

## d006-v0.3 prerequisites

[`d006-v0.3/`](d006-v0.3/) closes the v0.2 packet's input and contract gaps
so that both candidates can be built against one frozen packet:

- [`shared-inputs/`](d006-v0.3/shared-inputs/) holds the foundation-neutral
  packet for DS-01 to DS-07.
  [`semantics.md`](d006-v0.3/shared-inputs/semantics.md) fixes what every
  symbol and fixture means. The JSON files carry the statements,
  observations, negative cases and mutations as candidate-neutral terms.
  - DS-01 is a small Core fragment: exact words, length-indexed sequences,
    endian conversion, one canonical decoder with typed failure, and a
    parameterized quarter-round module checked against the RFC 8439 vector.
  - DS-02 is Sieve, a small constant-time language with public branches,
    bounded loops and one release construct. Its theorems are progress,
    preservation, and lock-step and two-run noninterference up to release,
    with a checked witness program and a control-flow leak counterexample.
  - DS-03 is OCR1, a canonical record format with 30 byte fixtures that
    between them exercise every error code.
  - DS-04 is the 32-bit carry-save identity
    `x + y = (x xor y) + ((x and y) << 1)`, with a canonical bit-blast, a
    golden certificate from a second producer, and eleven certificate
    mutations with their expected verdicts.
  - DS-05 to DS-07 fix the standalone checker's interface and corpus, the
    measurement profiles and faults, and the owner tasks with sealed fault
    seeds.
- [`manifest.json`](d006-v0.3/shared-inputs/manifest.json) lists every
  shared file's size and SHA-256. Its input-manifest digest is
  `06d3e2dce96f473b3bc4dfd54bdca86332dc57921804ab14699e1ccfd203916c`; an
  epoch freezes the packet by binding it.
- [`protocol/suite-overlay.json`](d006-v0.3/protocol/suite-overlay.json)
  assigns the host matrix, ceilings and timeout semantics, network,
  environment and cache contract, execution order, correction window and
  materiality bands. It lists which v0.2 gaps it closes and which remain
  open.
- [`protocol/toolchains.json`](d006-v0.3/protocol/toolchains.json) records
  the candidate and shared tools, with versions, commits, archive digests,
  build recipes and terms: Lean 4.34.1, Rocq 9.2.0 and its Stdlib built from
  their tags, CaDiCaL 3.0.1 as the pinned untrusted solver, drat-trim for
  the golden certificate, and the AArch64 runtime closures both standalone
  checkers need on the emulated H-02 row (an OCaml 4.14.1 bytecode runtime
  cross-built from its tag, and Lean 4.34.1's AArch64 release archive).

[`tools/d006_shared.py`](../../../tools/d006_shared.py) is a plain-Python
reference that computes every expected observation and regenerates every
file above byte for byte. It is an oracle for observations, not a
candidate. `python3 tools/d006_shared.py check` fails on any drift, and
[`tools/tests/test_d006_shared.py`](../../../tools/tests/test_d006_shared.py)
also checks that the DS-02 theorems hold on thousands of generated programs
in the reference model.

## d006-v0.3 candidates, runner and epoch

- [`rocq/`](d006-v0.3/rocq/) is C-01 on Rocq 9.2.0 and
  [`lean4/`](d006-v0.3/lean4/) is C-02 on Lean 4.34.1. Each formalizes DS-01
  to DS-04, ships a DS-05 standalone checker for both hosts (Rocq: extracted
  OCaml, native on H-01 and bytecode on H-02; Lean: compiled C), and declares
  its DS-06 re-checks, artifacts and fault hooks. Each candidate's
  `adapter.json` and `adapter.d/` map the shared symbols, methods, trust
  inventory and diagnostics. Contributor agents built them against
  [`CONTRACT.md`](d006-v0.3/CONTRACT.md). For each case the Rocq and Lean
  builders started in the preregistered order and ran at the same time;
  neither saw the other's candidate.
- Lean's DS-04 observations and its golden-certificate step use
  `native_decide`, which trusts Lean's compiler and runtime for those terms.
  `adapter.d/ds04.json` declares it, and the undeclared-trust check is scoped
  so this widening covers DS-04 only.
- [`tools/d006_run.py`](../../../tools/d006_run.py) is the epoch runner,
  with [`d006_check.py`](../../../tools/d006_check.py) and
  [`d006_render.py`](../../../tools/d006_render.py) as its harness.
  [`RUNNER.md`](d006-v0.3/RUNNER.md) describes the isolation (an unprivileged
  lab user in fresh namespaces with no network, inside the unchanged
  `tools/fs_sandbox.c` caps), the DS-04 solver runs, the archive, the export
  and the summary.
  [`HARNESS_ISSUES.md`](d006-v0.3/HARNESS_ISSUES.md) records H-01 to H-09 and
  [`FINDINGS.md`](d006-v0.3/FINDINGS.md) what the builders and the epoch
  found about the candidates.

Epoch `d006-e-c7b6648ae3988234297f` binds the shared-input manifest, the
overlay, the toolchain record, the runner, the harness and the sandbox by
digest. Its export is [`run/d006-e-c7b6648ae3988234297f/`](d006-v0.3/run/),
and `python3 tools/d006_run.py verify` on that directory checks every file,
record, object and log against its digest and regenerates the summary byte
for byte.

- Attempt 1, at revision `b7cb24b0a1eb3dc501bc6d550448f6352e0e2d5d`, ran the
  full plan: five cold bootstraps, three serial and three declared-parallel
  replays, 30 timed pairs, the nine DS-06 faults and a second workspace per
  candidate. Rocq met every gate a contributor can close. Lean's
  second-workspace parallel build aborted with "failed to create thread", so
  its HG-5 reproducibility gate and DS-06 failed: `lean --threads=4` peaks
  just over the 4 GiB address-space cap because glibc reserves a malloc arena
  for most threads.
- Attempt 2, at revision `754e61a3cc8b6f765e02d9b038a7bb76e15d0be5`, is
  Lean's one correction round (AM-09): `MALLOC_ARENA_MAX=2` for every Lean
  step, which keeps its builds at or under 3.30 GiB. Both candidates re-ran
  the full plan. Both archives' records stay in the export, and the summary
  reads the latest attempt.

In attempt 2 both candidates pass HG-2 to HG-7: 147 of 147 positive
observations agree, 20 of 20 negatives reject in their expected category,
no assumption goes undeclared, all builds and replays give one projection
digest in both workspaces, the standalone checkers match all 912 expected
corpus verdicts and run on both H-01 and H-02, and every component is
inventoried. HG-1 and
HG-8 stay unresolved for both because DS-07 is the owner's.

| Measure (H-01 medians) | Rocq | Lean 4 | Label |
| --- | --- | --- | --- |
| Cold bootstrap wall time | 12.4 s | 51.3 s | Rocq better |
| Replay wall time, DS-01 / DS-02 / DS-03 | 0.55 / 0.60 / 1.43 s | 0.93 / 0.87 / 1.98 s | Rocq better |
| Replay wall time, DS-04 | 5.75 s | 2.34 s | Lean better |
| Replay wall time, DS-05 / DS-06 | 1.76 / 5.26 s | 2.07 / 5.45 s | Rocq better / equivalent |
| Peak memory, DS-01 to DS-04 | 120 to 255 MB | 470 to 484 MB | Rocq better |
| Checker bytes on H-01, unstripped | 1.46 MB | 4.59 MB | Rocq better |
| Dependency closure | 917 MB | 1,162 MB | Rocq better |

Lean's DS-04 lead comes from `native_decide` above. The labels are the
overlay's materiality bands (replay times from 30 interleaved pairs, cold
bootstraps from five runs); H-02 timings are emulated and not compared. Owner items remain: DS-07's audit and
maintenance tasks (M-16, M-18), the R-01 to R-09 reviews, D-018 admission of
every tool, and D-004 and D-005 acceptance. M-17 independent review is
`unavailable`. A recommendation needs the owner's per-axis rationale.

## d006-v0.2 pre-epoch inputs

The candidate-neutral v0.2 pre-epoch packet and its seven-row case-input
index stay unchanged. Every DS-01 through DS-07 index row records absent
shared inputs, absent candidate mappings, zero executable fixtures,
unresolved coverage, and an active freeze blocker; v0.3 is the tranche that
supplies them. The Rust laboratory strictly parses these bytes, verifies the
raw bindings to the index and the unchanged suite document, and enumerates
the exact 14 candidate-case identities in memory. That enumeration is an
identity inventory, not a run order.

The packet's strict canonical JSON SHA-256, excluding its final line feed, is
`b56ad768c4584bdd00da4d4e85af642757b877dd5dc5ae438560ba4a486d9d21`.
The case-input index's strict canonical JSON SHA-256 is
`1118fe42a6d7111f50e40a88f0fe7b7fe4b9248b9335e0643b200fa983294ca0`;
its raw file SHA-256, including the final line feed, is
`1aec6a731bef0620c8500120ec8385d584f99a528b4a03c014e8516c55cc8136`.

## Nonclaims

The epoch above is contributor-produced, unreviewed, same-workspace and
same-owner evidence, never independent reproduction. Neither
Rocq nor Lean 4 is selected, preferred, recommended or admitted under D-018,
and neither is authorized for proof-bearing product work. The laboratory's
use of both toolchains is a recorded contributor disposition outside the
product lineage. Nothing here accepts D-004, D-005, D-006 or D-011, closes
S4, authorizes a release, or changes Orange's 3/10 (30%) binary gate-closure
score; that score is not release readiness. Independent review stays
`unavailable`.
