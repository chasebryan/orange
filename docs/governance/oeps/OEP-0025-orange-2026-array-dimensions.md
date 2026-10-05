---
number: OEP-0025
title: Orange 2026 array dimensions and update paths
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-10-04
updated: 2026-10-04
discussion: owner-direction-2026-09-29-s3u
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0025: Orange 2026 array dimensions and update paths

## Abstract

S3u admits arrays of three and four dimensions through the same `type`
declarations S3s uses for two: `type Poly = Zq^256; type Vector = Poly^2;
type Matrix = Vector^2;`. Every axis is nonempty and the product of all axes
is at most 65,536. It also admits update paths, `x with [i][j][k] = v`, which
name one index per dimension and mean the nested updates they abbreviate.

[`docs/DIMENSIONS_2026.md`](../../DIMENSIONS_2026.md) is the complete
normative proposal, accompanied by a provisional implementation and a
12-rule conformance runner. This proposal is in **Review**, follows S3t,
and requires OEP-0023 and OEP-0024. Implementation and merge do not
constitute semantic acceptance. It selects no semantic stratum, proof
foundation, backend, or target.

## Motivation

The objects of lattice cryptography have more than two dimensions. FIPS 203
writes ML-KEM's public matrix as a k × k array of polynomials, each 256
coefficients modulo q: three dimensions, and a batch of matrices is four.
Through S3t, such a matrix had to be a flat array with index arithmetic
repeated in source, losing the polynomial type exactly where a function
needs one.

Updating one element of a matrix also took two nested updates, as in
`s with [r] = (s[r] with [c] = v)`. Standards write `s[r][c] ← v`, and AES's
state and Keccak's lanes are updated that way on every round. Update paths
let source read as the standard reads.

## Scope and non-goals

This proposal raises the rank limit from two to four, extends the bounded
Core array type to record up to three inner dimensions, and adds update
paths of two to four indices with one Core node. It adds no type grammar,
command, option, token, reserved word, or diagnostic code. Direct repeated
powers such as `Word[8]^2^2^2`, ragged arrays, arrays of tuples, conversions
of arrays of arrays, broadcasting, transposition, rectangular windows, and
slices inside an update path remain outside this slice.

### Strata assumption

Like the preceding pure specification slices, S3u assumes only total,
deterministic value semantics. An array of rank r + 1 is a finite sequence
of identically typed arrays of rank r. This definition does not depend on
choosing `ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, or `ST-HOST`. A later
implementation stratum must preserve the same values at its representation
boundary.

## Specification

[`DIMENSIONS_2026.md`](../../DIMENSIONS_2026.md) defines the normative rules
and their bounded conformance index. In summary:

- A named array type followed by `^n` adds a dimension, up to four. The axes
  and their product are checked before evaluation, including for every
  instance of a size or type parameter; types are structural.
- Literals, fills, selections, slices, joins, slice updates, tuples,
  parameters, witnesses, and equality apply at every rank by the S3s rules.
- An update path names one index per dimension it reaches. Each index is
  checked against its own axis, and the value must have the exact type at
  the path's end. A path means the nested updates it abbreviates.
- A path costs ceil(n / 64), at least one, for each array it copies. Core
  records a path as one `update_path` node with its number of indices.

## Alternatives

Direct `T^n^m` syntax was considered and declined. A repeated power reads as
a tower of exponents in mathematics, and the alias names the object the
standard names. S3s chose the same, and S3u keeps the diagnostic display
`((Mod[3329]^256)^2)^2` distinct from source syntax.

Unbounded rank would introduce recursive type storage and recursion
obligations without a cryptographic object that needs it; four covers the
batches of matrices that lattice schemes use. Slices inside a path, as
`x with [i][a..b] = v`, would combine two update forms whose costs and
errors differ; the nested form expresses them without new rules.

## Compatibility and migration

Existing S3t source retains its types, values, output bytes, and steps.
Previously rejected third and fourth dimensions become valid when they
satisfy the scalar limit, and the dimension diagnostic now names four.
Update paths, which the parser rejected, are parsed and checked. The Rust
`ArrayType` remains `Copy`; it gains `dimensions()`, and the exported
`MAX_ARRAY_DIMENSIONS` is 4. The CLI gains no interface.

Rollback must revert the constructor, parser, analyzer, evaluator,
argument decoder, fixtures, conformance runner, and coupled normative
documents together. Sources using the new forms then return to their S3t
rejection. No binary, ABI, wire-format, package, or release compatibility
promise is introduced.

## Semantic and claim effects

The proposed semantic delta is finite rectangular nesting to rank four and
a functional update several dimensions deep. The supported observation
remains bounded, deterministic analysis and reference evaluation at an
identified implementation revision. The work establishes no proof,
soundness, refinement, compilation, cryptographic correctness, full ML-KEM
conformance, constant-time behavior, independent review, or production
readiness.

## TCB, axiom, and proof effects

The analyzer, Core constructors, argument decoder, and evaluator remain
engineering trust dependencies. The rank and product checks, path typing,
and the evaluator's path reconstruction are new or extended trusted paths.
The Core type representation fixes rank at four and admits no unbounded
recursive collection. No axiom, theorem, proof rule, certificate, checker,
or solver is added.

## Threat, abuse, and leakage effects

The resource boundary covers every axis and all scalar leaves. Checked
shape arithmetic must reject overflow, and specialization must reject an
invalid concrete shape before evaluating it. The argument decoder bounds
its frames at six, the argument vector, a tuple, and four dimensions, and
counts the value nodes of every rank before reading a witness. A path
charges each copied level before it rebuilds, and the evaluator rejects
inconsistent Core with no partial output.

The existing [threat model](../../security/THREAT_MODEL.md) and
[assurance model](../../ASSURANCE.md) continue to apply. Nested secret
indices have the same unresolved leakage obligations as earlier array
indices. Deterministic step accounting does not make the reference
evaluator constant-time.

## Target and ABI effects

None. This proposal selects no native layout, contiguous representation,
calling convention, object format, or target instruction. A future backend
must define and preserve the dimension and scalar boundaries it lowers.

## Standards, errata, and provenance

This is a language-shape proposal. Its fixtures transcribe FIPS 197 (AES,
updated May 2023), FIPS 202 (SHA-3, August 2015), and FIPS 203 (ML-KEM,
August 2024) as their sections write them, with expected values from those
standards' examples and appendices, and identities checked against the
algebra the standards define. The fixtures make no complete KEM, sampling,
encoding, decapsulation, or verified-standard-transcription claim. No
external standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency or license is added. Existing dependency and provenance
requirements remain in force.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3u_conformance.rs` binds all 12 normative
rule IDs to executable evidence and rejects drift. The CLI corpus runs
positive and negative cases repeatedly, observing exact status,
diagnostics, values, and test reports. Generated cases exercise shapes at
the scalar limit at rank four, every path index against its own axis,
paths against their nested forms at every position of a rank-four array,
path costs across shapes with exact and one-short budgets, equality costs,
rank-three and rank-four replay witnesses, and the unchanged boundaries.

The runner's exact evidence entry points are
`s3u_rule_index_is_exact_and_covered`,
`s3u_cli_conformance_corpus_is_exact_and_repeatable`,
`s3u_rank_and_scalar_limits_are_checked_before_evaluation`,
`s3u_each_path_index_checks_its_own_axis`,
`s3u_paths_agree_with_nested_updates_at_every_position`,
`s3u_path_cost_is_the_sum_of_the_copied_levels`,
`s3u_equality_cost_is_independent_of_difference_position`,
`s3u_rank_three_and_four_witnesses_replay`, and
`s3u_unchanged_boundaries_stay_refused`.

Existing S2 through S3t conformance runners and the algorithm corpus remain
compatibility obligations. Passing these checks does not supply semantic
acceptance or universal proof.

## Operations, release, and recovery

No service, key, deployment, or release mechanism is added. Repository and
compiler validation include the new normative index and its runner. A merge
records engineering integration; it does not change this proposal's Review
status or the acceptance gate.

## Support and deprecation

The fragment remains pre-alpha and best effort under D-022, with no new
maintenance period or compatibility commitment. Solo owner review under
D-023 is not independent review.

## Unresolved questions

- Arrays of tuples and of other structured elements need separate storage,
  recursion, and operation-boundary evidence.
- General shape parameters, transposition, rectangular windows, and named
  polynomial algebra need separate semantics and conformance records.
- Backend layout, leakage profiles, refinement contracts, and proof-bearing
  polynomial operations retain their existing decision prerequisites.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation
goals and that development not freeze unless the owner asks, and on
2026-09-29 that development of Orange continue, with the vision of a
technical and beautiful language for cryptographers, cryptologists, and
cryptanalysts. ML-KEM's matrix of polynomials, the third dimension S3s left
out, and update paths for the states of AES and Keccak follow from that
direction and the shape work of OEP-0022. This proposal records the S3u
surface built under that direction and is presented for the owner's review.
It is not accepted. Acceptance is the owner's decision alone; until it is
recorded here with a decision date, reviewed revision, and `solo-reviewed`
approval record, this proposal authorizes nothing by itself.
