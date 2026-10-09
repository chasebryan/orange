---
number: OEP-0023
title: Orange 2026 nested scalar arrays
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-10-01
updated: 2026-10-01
discussion: owner-direction-2026-10-01-s3s
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0023: Orange 2026 nested scalar arrays

## Abstract

S3s admits rectangular arrays of scalar rows through existing aliases:
`type Row = Word[32]^4; type Matrix = Row^4;`. Each axis is nonempty,
and rank in this slice is at most two. The scalar total is at most 65,536.
Existing literals, fills, indices, updates, slices, concatenation, tuples,
finite specialization, and equality apply structurally to rows. Byte-order
conversions retain their rank-one boundary.

[`docs/NESTED_ARRAYS_2026.md`](../../NESTED_ARRAYS_2026.md) is the complete
normative proposal, accompanied by a provisional implementation and a
12-rule conformance runner. This proposal is in **Review**, follows S3r,
and requires OEP-0021. Implementation and merge do not constitute semantic
acceptance. It selects no semantic stratum, proof foundation, backend, or
target.

## Motivation

The [roadmap](../../ROADMAP.md) and
[OEP-0022](OEP-0022-crypto-language-development-plan.md) call for checked
shapes and polynomial representations. Through S3r, an array's elements
were scalars, so matrices and collections of coefficient pairs had to use
one flat array and repeat the indexing arithmetic in source. That loses
the row type at the boundary where an operation needs exactly one row.

Two axes provide a bounded structural foothold: a matrix function can
require a particular row width and scalar domain, and each index is checked
against the dimension it selects. The same implementation supports word
states and modular coefficient-pair collections without introducing a
polynomial algebra, dependent shape language, or separate compiler lineage.

## Scope and non-goals

This proposal adds rank-two scalar arrays, their bounded Core type
representation, and structural use by the existing pure operations. It adds
no type grammar, command, option, token, reserved word, or diagnostic code.
The index-suffix grammar admits successive selections such as `m[i][j]`
within the existing expression-nesting budget.
Rank three, ragged arrays, empty axes, tuple elements, implicit flattening,
matrix byte-order conversions, broadcasting, column slicing, and multi-axis
update syntax remain outside this slice.

### Strata assumption

Like the preceding pure specification slices, S3s assumes only total,
deterministic value semantics. A matrix is a finite sequence of identically
typed finite scalar sequences. This definition does not depend on choosing
`ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, or `ST-HOST`. A later
implementation stratum must preserve the same values at its representation
boundary.

## Specification

[`NESTED_ARRAYS_2026.md`](../../NESTED_ARRAYS_2026.md) defines the normative
rules and their bounded conformance index. In summary:

- An existing scalar-row alias followed by `^n` constructs a matrix. Both
  axes and their product are checked before evaluation; types are structural.
- A matrix literal or fill holds exact row values. Each index selects one
  axis, and updates replace one element of that axis.
- Slices and concatenation operate on outer rows; selected rows retain the
  existing scalar-array operations. Matrices may be tuple parts and concrete
  size or type instances.
- Equality visits every scalar pair, with deterministic recursive costs.
  Core retains existing node kinds and bounded `Copy` array-type storage.
- Byte-order conversion rejects matrix sources and targets. A selected word
  row retains its prior conversion meaning.

## Alternatives

Unbounded recursive arrays would introduce arbitrary rank and recursive
type-storage obligations beyond the immediate row boundary. Arrays of tuples
would add structured element families not needed to establish that boundary.
Both require separate proposals and resource evidence.

New matrix-type syntax would expand the parser even though transparent aliases
already express both dimensions. S3s uses aliases and keeps the diagnostic
display `(Word[32]^4)^4` distinct from source syntax.

Flattening all matrices into a scalar sequence would erase row width from
indices, updates, and function boundaries. Implicitly flattening byte-order
conversions would also silently choose a serialization order. Both are
rejected; explicit source functions can define a flattening order when one
is wanted.

## Compatibility and migration

Existing S3r source retains its types, values, output bytes, and steps.
Previously rejected scalar-array aliases followed by a length become valid
when they satisfy the new rank and scalar limits. Tuples as elements remain
invalid, with diagnostics describing that boundary. S3u, implemented and in
owner review under
[OEP-0025](OEP-0025-orange-2026-array-dimensions.md), admits a third and a
fourth axis and update paths such as `x with [i][j] = v`. A fifth axis is
still `ORC0203`.
Successive index suffixes are parsed and then checked against each selected
type, where S3r rejected a repeated suffix as syntax. The CLI gains no
interface. The Rust `ArrayType` remains `Copy`; `element()`
returns the exact row or scalar, `length()` retains outer-length meaning,
and `scalar_length()` exposes the total scalar count.

Rollback must revert the constructor, analyzer, evaluator, fixtures,
conformance runner, and coupled normative documents together. Sources using
the new row boundary then return to their S3r rejection. No binary, ABI,
wire-format, package, or release compatibility promise is introduced.

## Semantic and claim effects

The proposed semantic delta is finite rectangular nesting of scalar arrays.
The supported observation remains bounded, deterministic analysis and
reference evaluation at an identified implementation revision. The work
establishes no proof, soundness, refinement, compilation, cryptographic
correctness, full ML-KEM or NTT conformance, constant-time behavior,
independent review, or production readiness.

## TCB, axiom, and proof effects

The analyzer, Core constructors, and evaluator remain engineering trust
dependencies. Array rank and product checks, row-type propagation, recursive
equality, and conversion rejection are new or extended trusted paths.
This slice's Core type representation recorded rank two. S3u widens that
same `Copy` representation to four dimensions under
[OEP-0025](OEP-0025-orange-2026-array-dimensions.md), and it still admits
no unbounded recursive collection. No axiom, theorem, proof rule,
certificate, checker, or solver is added.

## Threat, abuse, and leakage effects

The resource boundary covers both axes and all scalar leaves. Checked shape
arithmetic must reject overflow; specialization must reject an invalid
concrete shape before evaluating it. Exact row typing prevents domain or
dimension mismatch from being repaired by equal flattened lengths.
Functional updates preserve immutable row values even when fills share
storage. The evaluator rejects inconsistent Core with no partial output.

The existing [threat model](../../security/THREAT_MODEL.md) and
[assurance model](../../ASSURANCE.md) continue to apply. Nested secret
indices have the same unresolved leakage obligations as earlier array
indices. Deterministic step accounting and exhaustive equality do not make
the reference evaluator constant-time.

## Target and ABI effects

None. This proposal selects no native layout, contiguous representation,
calling convention, object format, or target instruction. A future backend
must define and preserve the row and scalar boundaries it lowers.

## Standards, errata, and provenance

This is a language-shape proposal. Its polynomial-pair fixture is a pure
mathematical model over `Mod[3329]`, with explicit quadratic factors and
expected values. OEP-0022 supplies the broader ML-KEM planning provenance.
The fixture makes no complete transform, packing, randomness, decapsulation,
KEM, or verified-standard-transcription claim. No external standard gains
normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency or license is added. Existing dependency and provenance
requirements remain in force.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3s_conformance.rs` binds all 12 normative
rule IDs to executable evidence and rejects drift. The CLI corpus runs
positive and negative cases repeatedly, observing exact status, diagnostics,
matrix types and values, and step reports. Generated cases exercise axis and
scalar limits, separate index ranges, exact row domains, specialization,
copy-operation costs, and equality differences at early and late positions.

The runner's exact evidence entry points are
`s3s_rule_index_is_exact_and_covered`,
`s3s_cli_conformance_corpus_is_exact_and_repeatable`,
`s3s_axis_and_scalar_limits_are_checked_before_evaluation`,
`s3s_each_axis_checks_static_and_computed_indices`,
`s3s_matrix_equality_cost_is_independent_of_difference_position`,
`s3s_nested_copy_operations_charge_by_outer_slots`, and
`s3s_nested_literals_keep_rank_and_domains`.

The mathematical coefficient-pair example provides a nontrivial use of rows
and residues with explicit expected results. It is evidence for those
operations at the tested inputs. Existing S2 through S3r conformance runners
and the algorithm corpus remain compatibility obligations. Passing these
checks does not supply semantic acceptance or universal proof.

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

- Arrays of structured elements still need their own storage, recursion,
  and operation-boundary evidence. Rank three and four, and update paths,
  are the S3u surface in OEP-0025 and are not accepted by this proposal.
- General shape parameters, rectangular windows, transposition, and named
  polynomial representations need separate semantics and conformance records.
- Backend layout, leakage profiles, refinement contracts, and proof-bearing
  polynomial operations retain their existing decision prerequisites.

## Decision record

On 2026-10-01 the owner directed Codex to select a complicated Orange design
area, implement it, push it to the repository, and merge when all checks are
green. This bounded S3s implementation follows the roadmap and OEP-0022's
shape work. The direction authorizes the implementation and integration,
not semantic acceptance of this text.

This proposal is presented for owner review. Acceptance requires the owner's
recorded decision date, exact reviewed revision, and `solo-reviewed` approval
record under the OEP process. Those fields remain empty.

AI-assisted drafting and implementation: Codex using GPT-6.1 prepared this
proposal and its companion specification under the owner's requested model
direction. The owner remains the decision authority. AI output is neither
technical proof nor independent review.
