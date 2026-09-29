# D-006 proof-foundation laboratory

Status: non-product research; `d006-v0.3` prerequisites written, no epoch
frozen, no candidate built

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

Still open: D-004 and D-005 acceptance, the owner's D-018 admission of every
candidate tool, the candidate adapters and the isolated runner and observer,
the result and same-owner-replay schema (both written with the runner), and
the owner's protocol review. The candidates' formalizations and the first
epoch come next.

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

Current D-006 execution evidence remains 0/14 candidate-case runs. Neither
Rocq nor Lean 4 is selected, preferred, recommended or admitted under D-018,
and neither is authorized for proof-bearing product work. The laboratory's
use of both toolchains is a recorded contributor disposition outside the
product lineage. Nothing here accepts D-004, D-005, D-006 or D-011, closes
S4, authorizes a release, or changes Orange's 3/10 (30%) binary gate-closure
score; that score is not release readiness. Independent review stays
`unavailable`.
