---
number: OEP-0026
title: Orange 2026 universal words
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-10-05
updated: 2026-10-05
discussion: owner-direction-2026-10-05-universal-words
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

# OEP-0026: Orange 2026 universal words

## Abstract

S3v lets a pure specification take one parameter `W: Word`, standing for
`Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`. The body is checked at each
width. Ordinary size parameters of that function are checked for every affine
value, at the corners of their ranges, rather than by listing a few widths or
by building every size instance. A call names the specialization it uses, and
only those specializations are lowered.

[`UNIVERSAL_2026.md`](../../UNIVERSAL_2026.md) is the complete normative
proposal with seven bounded conformance rules. This proposal is in **Review**.
It follows S3t and OEP-0024 and accompanies a provisional implementation.
Integration is not semantic acceptance. S3u and OEP-0025 are a separate
in-flight increment and are not a dependency of this proposal.

## Motivation

[OEP-0022](OEP-0022-crypto-language-development-plan.md) asks for one
permutation written over a lane width. A finite type list can name `Word[8]`
and `Word[64]`, and a finite size range can name each round count, but writing
theta, rho, pi, chi, and iota once for every lane width still required a listed
few. Checking every pair of width and size in advance would also spend the
existing instance budget on combinations no call uses.

Keccak-p[b, nr] from FIPS 202 is the target of this slice: one source of those
steps, the existing SHA3-256("abc") digest at lane width 64, and Keccak-f[200]
at lane width 8 checked against XKCP's published intermediate state. Typed
`impl` bodies and refinement stay for a later increment.

## Scope and non-goals

S3v admits `W: Word` as one bracket parameter, together with the existing
finite size parameters on the same function. It checks that body at the four
word widths and at the corners of its size ranges, rejects a non-affine size
use and a length that names a size, and lowers a specialization when a call in
the declaring module names it.

The following stay outside this slice: a second word parameter, a word
parameter combined with a listed type parameter, type classes, parameters of
`type` declarations, imports of names, typed `impl` bodies, arrays of rank
three or more, variable-length arrays, symbolic moduli, and any selection
among the D-004 candidates. The corner rule is the checker for the affine
expressions this slice admits. It is not offered as a proof.

### Strata assumption

The proposal assumes only total, deterministic checking at a finite set of
widths and corners, and the existing pure word and array operations. It
neither selects a D-004 candidate nor defines an implementation or proof
stratum.

## Specification

The companion specification defines the word-parameter vocabulary; checking
at every width, including literals that must fit `Word[8]`; affine sizes and
the corner rule; invariant lengths; call specializations; the Keccak-p known
answers; and compatibility. A literal that fits `Word[64]` and not `Word[8]`
is rejected. A computed shift or rotation keeps the S3r meaning, so rho's
offsets and iota's bit places stay generic. One module builds at most 256
specializations. A call from another module does not create a new
specialization in a module that has already been checked.

## Alternatives

Listing `Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]` in `K in { ... }`
remains available and is unchanged. It checks each listed type eagerly, which
is the right rule for a few unrelated types, and it does not by itself check
a size for every value in a range. Expanding every width and every size into
Core would reuse the existing instance machinery and would spend the instance
budget on uncalled combinations. A general type quantifier would need a
different proof and resource story. S3v checks the four widths and the affine
corners, and it lowers only the calls that occur.

## Compatibility and migration

Functions that use only finite sizes, or only a listed type parameter, keep
their types, values, diagnostics, and reference-evaluation costs. Sources that
S3t accepted keep those results under S3v. The new colon form is `W: Word`
only; `in` remains required for a size and for a type list. Rollback must
revert the word-parameter parser, the corner checker, specialization lowering,
the fixtures, and the coupled proposal documents together. No stable source,
binary, ABI, package, or release promise is introduced.

## Semantic and claim effects

The delta is a finite check of every word width and of every affine size
value, plus demand-driven specializations of the existing word and array
types. The supported observation is bounded typechecking and reference
evaluation at an identified implementation revision. It establishes no
universal theorem about a non-affine program, no constant-time behavior, no
native preservation, and no complete SHA-3 or Keccak conformance beyond the
two tested inputs.

## TCB, axiom, and proof effects

The analyzer and evaluator remain engineering trust dependencies. The corner
enumeration, the affine-expression restriction, specialization identity, and
the same-module lowering rule are extended trusted paths. No theorem, axiom,
solver, certificate, authoritative checker, or product proof dependency is
added.

## Threat, abuse, and leakage effects

An attacker-controlled source must not turn a non-affine size use into an
unchecked index, must not make an uncalled literal that overflows `Word[8]`
succeed, and must not build an unbounded number of specializations. Those
cases fail closed under the existing diagnostic and instance budgets. This
slice does not classify secrets or prove source or target leakage. The
existing threat and assurance models remain in force. No new control is added
to the threat-model register in this increment; S3s and S3t likewise added
none, and a later control number must not collide with an in-flight assignment.

## Target and ABI effects

None. Word and array values remain reference values, with no selected native
layout, machine instruction, serialization format, or foreign contract.

## Standards, errata, and provenance

FIPS 202 (August 2015) defines Keccak-p and the SHA3-256 example for `abc`.
The Keccak-f[200] known answer is the permutation of the all-zero state in
XKCP's `tests/TestVectors/KeccakF-200-IntermediateValues.txt` at commit
`4017707cade3c1fd42f3c6fa984609db87606700` (2018-03-16, "Reshaped the
directory structure"),
<https://github.com/XKCP/XKCP/blob/4017707cade3c1fd42f3c6fa984609db87606700/tests/TestVectors/KeccakF-200-IntermediateValues.txt>.
The fixture records that URL and commit. The file itself is not copied into
this repository. Matching those two published values does not certify the
implementation against the whole of either document.

## Dependencies, licenses, and IP

No dependency or license is added. The compiler remains free of third-party
crates. D-018 admissions and the publication rules remain unchanged. The XKCP
intermediate-values file is an external known answer cited by URL and commit;
it is not vendored.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3v_conformance.rs` binds the exact seven
`S3V-01` through `S3V-07` rules. Its repeated CLI corpus covers the generic
Keccak-p known answers, a program that keeps listed types and finite sizes
beside a word parameter, and rejected vocabulary, literals, non-affine
indices, corners, lengths, calls, and type arguments. Generated cases exercise
all four widths, a second word parameter, an uncalled overflowing literal,
retained S3m and S3o programs, affine and non-affine sizes, and call
diagnostics. Existing S2 through S3t conformance remains a compatibility
obligation. Test success is implementation evidence and does not accept this
specification or prove a generic theorem.

The exact runner entry points are `s3v_rule_index_is_exact_and_covered`,
`s3v_fixture_inventory_and_outputs_are_exact`,
`s3v_widths_and_vocabulary_are_exact`, and
`s3v_sizes_lengths_and_calls_are_exact`. Rule S3V-06 is covered by the CLI
corpus. The other rules also have generated CLI cases.

## Operations, release, and recovery

No service, deployment, key, or release mechanism is added. The
implementation, normative rule index, source inventory, and coupled documents
are validated together. Merge does not close S3, S4, a complete 1.0 journey,
or release gates.

## Support and deprecation

The fragment remains pre-alpha and best effort under D-022. It introduces no
support window or compatibility commitment. Owner review is `solo-reviewed`
and never independent review.

## Unresolved questions

- Typed `impl` bodies and refinement between `spec` and `impl` remain a later
  increment and must not decide D-004 by themselves.
- Lists of types named once for several functions, positions given as
  parameters, slices and words at positions computed from data, and tests that
  claim a call stops or a source is rejected remain separate candidate slices.
- Imports, rank of three or more, and a word parameter created from another
  module remain unspecified.

## Decision record

On 2026-10-05 the language-design direction for this increment was one
function over every word width, with sizes checked once for every affine
value, leaving typed `impl` bodies for a later increment. That direction
authorizes this development and integration. It does not record semantic or
foundational acceptance.

Acceptance requires the owner's exact reviewed revision, decision date, and
`solo-reviewed` approval record under the OEP process. Those fields remain
empty. Grok, as a Cursor cloud agent, assisted drafting and implementation
under Chase Bryan's direction. The owner remains decision authority. AI output
is not technical proof or independent review.
