---
number: OEP-0028
title: Orange 2026 named type lists
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-10-05
updated: 2026-10-05
discussion: owner-direction-2026-09-29-s3x
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0028: Orange 2026 named type lists

## Abstract

S3x lets a module name a finite list of types once, `types Fields = {F, L, P};`,
and lets several functions be checked for that list, `spec pow[K in Fields]`.
Each function is still one instance per listed type. The list is not a type
and not a type variable.

[`docs/TYPE_LISTS_2026.md`](../../TYPE_LISTS_2026.md) is the complete normative
proposal, accompanied by a provisional implementation and an 8-rule
conformance runner. This proposal is in **Review**, follows S3u, and requires
OEP-0018. Implementation and merge do not constitute semantic acceptance. It
selects no semantic stratum, proof foundation, backend, or target.

## Motivation

OEP-0018 writes one function for a list of types, and leaves open whether
that list should be nameable once so several functions stay checked for the
same fields. Exponentiation, inversion, and Euler's criterion repeat
`K in {F, L, P, Q, D}`. SHA-256 and SHA-512 repeat `W in {Word[32], Word[64]}`
on Ch, Maj, and the round. A reader checks the copies by eye. Naming the
list once keeps those functions on the same types without a new runtime.

## Scope and non-goals

This proposal adds `types NAME = {TYPE, ...};` and `K in NAME`. It adds no
token and no reserved word: `types` is recognized by position. It adds
`ORC0244`. It does not add type variables, bounds over every type,
parameterized aliases, imported lists, or a second meaning for `K in {F, L}`.

### Strata assumption

Like the preceding pure specification slices, S3x assumes only total,
deterministic value semantics. A named list is finite specialization, the
same specialization S3o already checks. This definition does not depend on
choosing `ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, or `ST-HOST`.

## Specification

[`TYPE_LISTS_2026.md`](../../TYPE_LISTS_2026.md) defines the normative rules
and their bounded conformance index. In summary:

- A list is declared among the module's `type` aliases, before functions,
  with 1 through 256 distinct resolved types.
- `K in Fields` is that list. Calls and fitting are unchanged.
- A list name is not a type. A failed list is reported once.

## Alternatives

Leaving the list written on every function was considered and is what S3o
does. It stays available. Making `type Fields = {F, L}` overload `type`,
which already names one type, was declined so a list cannot be mistaken for
a type. A type variable `K` with no list would check one body for every
possible type; this checker does not do that, and the source would no
longer say which types were checked.

## Compatibility and migration

Every source S3u accepted keeps its Core, values, and output. `types` was
an identifier, and it remains one except where a declaration may begin.
`K in Name` was a syntax error unless `..` followed `Name`. Rollback reverts
the declaration, the parameter form, `ORC0244`, the fixtures, and the
runner together. No binary, ABI, package, or release compatibility promise
is introduced.

## Semantic and claim effects

The proposed semantic delta is a name for a finite list of types already
admitted by OEP-0018. The supported observation remains bounded,
deterministic analysis and reference evaluation at an identified
implementation revision. The work establishes no proof, soundness,
refinement, compilation, cryptographic correctness, constant-time behavior,
independent review, or production readiness.

## TCB, axiom, and proof effects

The analyzer remains an engineering trust dependency. Resolving a list name
to the types written in its declaration is a new trusted path. No axiom,
theorem, proof rule, certificate, checker, or solver is added.

## Threat, abuse, and leakage effects

The instance bound stays 256, and a list longer than that is rejected before
evaluation. Sharing a list does not change the cost of checking one
instance. The existing threat model continues to apply. No new control
number is taken.

## Target and ABI effects

None. This proposal selects no layout, calling convention, or target.

## Standards, errata, and provenance

This is a language-convenience proposal. Its fixtures check addition in
`Mod[65537]` and `Mod[3329]`, Ch and Maj on `Word[32]` and `Word[64]` as
FIPS 180-4 writes those functions, and doubling in `Mod[7]` and `Mod[11]`.
They make no complete SHA-2 or field claim. No external standard gains
normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency or license is added. Existing dependency and provenance
requirements remain in force.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3x_conformance.rs` binds all 8 normative
rule IDs to executable evidence and rejects drift. The CLI corpus runs
positive and negative cases repeatedly. Generated cases admit 256 moduli,
reject 257, reject a product of 512 instances, keep an inline list, reject
`use` after a list, and render the list in `orangec doc`.

The runner's exact evidence entry points are
`s3x_rule_index_is_exact_and_covered`,
`s3x_cli_conformance_corpus_is_exact_and_repeatable`,
`s3x_a_list_of_256_types_is_one_instance_each_and_257_is_rejected`,
`s3x_inline_lists_and_use_order_keep_their_meaning`, and
`s3x_documentation_names_the_list`.

Existing S2 through S3u conformance runners and the algorithm corpus remain
compatibility obligations. Passing these checks does not supply semantic
acceptance or universal proof.

## Operations, release, and recovery

No service, key, deployment, or release mechanism is added. A merge records
engineering integration; it does not change this proposal's Review status
or the acceptance gate.

## Support and deprecation

The fragment remains pre-alpha and best effort under D-022, with no new
maintenance period or compatibility commitment. Solo owner review under
D-023 is not independent review.

## Unresolved questions

- Whether a `type` declaration should take the same named list, so one
  alias is checked for each type.
- Whether a list should be visible to a module that uses its owner, and
  how a caller would write the name.
- Whether two lists may be concatenated, as `K in Fields` composed with
  another list, without writing the union out.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation
goals and that development not freeze unless the owner asks, and on
2026-09-29 that development of Orange continue. OEP-0018 left the shared
list as an open question. This proposal records the S3x surface built under
that direction and is presented for the owner's review. It is not accepted.
Acceptance is the owner's decision alone; until it is recorded here with a
decision date, reviewed revision, and `solo-reviewed` approval record, this
proposal authorizes nothing by itself.
