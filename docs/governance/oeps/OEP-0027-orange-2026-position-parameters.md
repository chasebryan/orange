---
number: OEP-0027
title: Orange 2026 position parameters
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-10-05
updated: 2026-10-05
discussion: owner-direction-2026-10-05-s3w
related-decisions:
  - D-002
  - D-004
  - D-011
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
  - OEP-0003
  - OEP-0004
  - OEP-0005
  - OEP-0006
  - OEP-0007
  - OEP-0008
  - OEP-0009
  - OEP-0010
  - OEP-0011
  - OEP-0012
  - OEP-0013
  - OEP-0014
  - OEP-0015
  - OEP-0016
  - OEP-0017
  - OEP-0018
  - OEP-0019
  - OEP-0020
  - OEP-0021
  - OEP-0023
  - OEP-0024
  - OEP-0025
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0027: Orange 2026 position parameters

## Abstract

S3w admits a position parameter `a at lo..hi` in a function's brackets. The
name ranges over every integer from `lo` up to, but not including, `hi`, the
function is checked once for that whole range, and a call chooses one integer
in the range. Four such parameters let one ChaCha20 quarter round act on four
positions of a 16-word state.

[`docs/POSITIONS_2026.md`](../../POSITIONS_2026.md) is the complete normative
proposal, accompanied by a provisional implementation and a 6-rule
conformance runner. This proposal is in **Review**. It follows S3u and does
not depend on S3v. Implementation and merge do not constitute semantic
acceptance. It selects no semantic stratum, proof foundation, backend, or
target, and it does not decide D-004, a Gate 0 question, or a license.

## Motivation

RFC 8439 writes one quarter round and applies it at eight positions of a
16-word state. Through S3u, Orange can write the quarter round on four words
and can index a state with literals, but it cannot give the positions to one
definition. A size parameter `n in 0..16` would make one instance per
integer, and four of them exceed the 256-instance limit, so a size cannot
stand for a position.

OEP-0022 records this as a P1 acceptance criterion and S3e and S3g deferred
it. S3w is the smallest addition that meets it: the positions are parameters,
each index is checked against the state shape, and the definition is written
once.

## Scope and non-goals

The slice admits position parameters and the literal call arguments that choose
one integer from each range. It does not admit:

- Lists of types named once for several functions.
- Slices or words at positions computed from data.
- Tests that claim a call stops or a source is rejected.
- A universal word-width parameter (S3v, OEP-0026). This slice does not
  depend on that work.
- Distinctness of the positions in one call. Overlapping positions are
  well typed; a later update wins.
- Any proof, leakage, timing, backend, or target claim.

## Specification

- A bracket entry `a at lo..hi` is a position parameter. `at` is that word
  only there. Elsewhere it remains an identifier. No token or reserved word
  is added.
- Bounds match sizes: `0 <= lo < hi <= 65536`. An empty or oversized range is
  `ORC0243`. The range does not multiply instances.
- The name has type `Int` and shares the namespace of sizes, parameters, and
  bindings. It is not a size, so it cannot be a length, a loop bound, or a
  modulus (`ORC0237`).
- An index or an update that uses the name is proved for every integer in the
  range, by the interval arithmetic already used for literals and loop
  indices. A slice bound does not admit it (`ORC0226`).
- A call writes one static integer per position, in brackets, built as a size
  is built. A missing entry is `ORC0239`. An entry that is not an integer in
  range is `ORC0243`.
- The integers are trailing `Int` parameters of the one Core function, passed
  as literals by the call. They are not part of the instance identity.
- ChaCha20's `inner_block` calls one `quarter_at` at the eight positions RFC
  8439 writes. The section 2.1.1 vector is a conformance fixture. The vectors
  are cited from the RFC and are not vendored.

## Alternatives

Copying the quarter round once per column and diagonal keeps the checker S3u
already has and multiplies the text a reviewer must compare. Hiding the eight
index tuples inside one body also typechecks, and it removes the positions
from the signature. Using a size parameter would multiply instances and exceed
the 256-instance limit. Data-dependent positions would need a proof this
checker cannot enumerate. This slice keeps the positions in the signature and
the proof finite.

## Compatibility and migration

Every source S3u accepts is accepted with the same values and costs. The only
change to an existing diagnostic is the note of `ORC0226`, which now names
position parameters. The message is unchanged. No source migration is required
for programs that do not use the new form.

## Semantic and claim effects

A position parameter is a static integer interval, not a new numeric width and
not a new proof domain. The function still has one instance. The checker
proves the index obligations at every integer of each declared interval, and a
call proves its literal lies in that interval. No verified, constant-time, or
Gate 0 claim is created or extended.

## TCB, axiom, and proof effects

The trusted checker gains one static obligation: a position used as an index
stands for every integer in its interval, and a call position is a literal in
that interval. No axiom is added. No external solver is consulted. The Core
form is an ordinary trailing `Int` parameter with a literal argument, which the
existing evaluator already understands.

## Threat, abuse, and leakage effects

An out-of-range or computed position is rejected before evaluation, so a caller
cannot move a quarter round onto an index the definition did not cover. The
slice does not change secret-independent control flow, table-free code
generation, or the leakage claims recorded for earlier slices. Position names
are source locations only and are not a channel.

## Conformance, tests, and evidence

`compiler/fixtures/s3w/` holds the accepted and rejected programs for rules
S3W-01 through S3W-06. `compiler/crates/orangec/tests/s3w_conformance.rs`
executes them. `algorithms/chacha20/chacha20.or` writes one quarter round and
calls it at the RFC 8439 column and diagonal positions. The RFC's test vectors
are cited by URL and are not vendored.

## Unresolved questions

Whether a later slice should allow a position to compute a slice bound, or a
position drawn from data, is left open. Both are named in the roadmap and are
outside this record. The owner has not accepted this proposal.

## Decision record

This record is in Review. It is not an owner decision, it does not change any
D-record, and it claims no independent review. Acceptance would require the
owner's explicit decision and the repository's normal governance steps. This
text does not claim that acceptance.
