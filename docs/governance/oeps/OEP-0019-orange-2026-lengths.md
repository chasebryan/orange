---
number: OEP-0019
title: Orange 2026 lengths and evaluation controls
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3p
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

# OEP-0019: Orange 2026 lengths and evaluation controls

## Abstract

An array, an array literal, and a byte string hold up to **65,536** elements,
where they held 256: as many as a loop visits, and exactly the values of a
16-bit word, so a `Word[16]` indexes the longest array with no check at run
time. Costs per element are unchanged, so a long array is built in rows
placed with slice updates. `orangec eval` gains three options: `--steps N`
sets the step budget of a run, `--spec NAME` evaluates only the functions it
names, and `--stats` reports the steps each used.

```orange
// RFC 8439 appendix A.2, test vector 2: 375 bytes, padded to six blocks
// and cut back.
encrypt(key, 1, nonce, ietf() ++ [0; 9])[..375]
```

```console
$ orangec eval --steps 2097152 --spec pepin --stats compiler/fixtures/s3p/valid-lengths.or
lengths::pepin: (Mod[65537], Mod[65537], Bool) = (65536, 21846, true)
lengths::pepin: 1452583 steps
total: 1452583 of 2097152 steps
```

The normative text is [`docs/LENGTHS_2026.md`](../../LENGTHS_2026.md). An
implementation, three programs, and an 11-rule conformance runner accompany
it so that the proposal can be reviewed against running code. This proposal
is in **Review** and requires OEP-0018, which is also in review. It accepts
no D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

The objects of cryptography are longer than 256 bytes. An ML-KEM-512
encapsulation key is 800 bytes and its ciphertext 768 (FIPS 203); an
ML-DSA-44 public key is 1,312 bytes and its signature 2,420 (FIPS 204); an
RSA-2048 modulus and every OAEP block under it are 256 bytes, and RSA-4096's
are 512. RFC 8439 prints test vectors of 375 and 265 bytes. Through S3o each
had to be cut into pieces of at most 256 and carried as a head and a tail,
and the standards' vectors could not be written as printed.

The limit of 65,536 is not arbitrary: it is the loop bound Orange has always
had, and the number of values of a 16-bit word, so the index proofs of S3g
extend to tables of 16-bit entries unchanged. Longer programs also need a
budget larger than the fixed 1,048,576 steps, and a reader who wants to run
one function of a long program and see its cost had no way to do either.

## Scope and non-goals

This proposal defines the new length limit for array types, lengths written
with sizes, array literals, byte strings, fills, joins, slices, and slice
updates; the index proofs at that limit; conversions in a byte order of words
wider than the evaluator's exact-integer limit; and the three evaluation
controls with their usage errors, their report, and the command-line
diagnostic `ORC1016`. It adds no syntax, token, reserved word, Core node, or
language diagnostic code.

It does not define arrays longer than 65,536 elements, lengths known only at
run time, arrays of arrays, more instances per function, a budget per
function or per call, a budget for checking, or a change to the cost of any
operation. It makes no timing, secrecy, or leakage claim, and the step report
is not a measure of time on any machine.

### Strata assumption

As for S3b through S3o, S3p assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A longer array means
what a shorter one means, element by element, and the evaluation controls
change which functions run and within what budget, never what a function
means. S3p therefore has the same meaning under `ST-REL`, `ST-UNI`,
`ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source surface needs no
change.

## Specification

[`docs/LENGTHS_2026.md`](../../LENGTHS_2026.md) is the complete normative
text. In summary:

- **Lengths.** Every array has 1 through 65,536 elements, written lengths are
  decimal with at most five digits and no leading zero, and every message
  that named 256 names 65536. Size parameters are unchanged: bounds up to
  65,536 and at most 256 instances.
- **Indices.** The index and slice proofs are unchanged; a `Word[16]` index
  is in range of an array of 65,536 elements and of no shorter one.
- **Conversions.** Words of any admitted length convert to words of every
  width, and to `Int` or `Mod[m]` while the number they spell has at most
  16,384 significant bits; a larger number stops at run time with
  `ORC0301`, as every other exact integer past the limit does.
- **Costs.** Unchanged at every length. Every operation that makes an array
  makes at most 64 elements per step, so memory stays proportional to steps.
- **Evaluation controls.** `--steps N` from 1 through 1,073,741,824, once;
  `--spec NAME` up to 64 distinct names of functions without parameters of
  the root module, `ORC1016` when one names none; `--stats` writes each
  function's steps and the total against the budget to standard error. A
  step-limit diagnostic gains a note naming `--steps`. The library gains
  `evaluate_selected` and `EvaluatedFunction::steps`.

## Alternatives

Keeping 256 and adding a separate type for long byte strings was rejected: it
would have split every function over bytes into two, and the loop bound
already admits 65,536 iterations.

A larger limit, such as 2^20, was considered. It would let a `Word[16]` index
no longer be proved in range by its type alone, gives no standard's object a
home that 65,536 does not (the longest above is 3,168 bytes), and multiplies
the memory a literal can ask for by 16.

Lowering the cost of updates on long arrays, so that one element costs one
step whatever the length, was rejected for this slice: it breaks the
invariant that an evaluation's memory is bounded by its steps. Rows placed
with slice updates keep that invariant and cost about as much per element.

A budget per function, written in the source, was left for later: a
command-line budget changes no program's meaning, while a source budget would
be part of it.

## Compatibility and migration

Every source that S3o accepts has arrays of at most 256 elements and is
accepted with the same Core values, output bytes, and steps. A source that S3o
rejects gets the same diagnostics, except that lengths, literals, byte
strings, fills, joins, and slices of 257 through 65,536 elements are now
accepted where their types agree; every message and note that named 256 now
names 65536; a `Word[16]` index into an array of 65,536 elements is now in
range; and conversions of words wider than 16,384 bits can now be written.

`orangec` gains the three options, the code `ORC1016`, the usage line
`orangec eval [--steps <N>] [--spec <NAME>]... [--stats] <FILE>`, and a note
on step-limit diagnostics. `orangec enc` reads a sealed file whose header
names a chunk of up to 65,536 sealed bytes, where it refused more than 256;
the format is unchanged. `orangec lex` is unchanged.

The public Rust API gains `evaluate_selected` and `EvaluatedFunction::steps`
in the `eval` module, and `MAX_ARRAY_LENGTH` and `MAX_ARRAY_ELEMENTS` change
from 256 to 65,536.

Rollback reverts the analyzer limits, evaluator, command-line interface,
tests, fixtures, and normative documents together.

## Semantic and claim effects

This proposal extends the lengths the earlier slices give meaning to and
adds evaluation controls. The supported claim remains deterministic, bounded
analysis and evaluation of the documented fragment at a recorded
implementation revision. It establishes no soundness, proof, refinement,
compilation, cryptographic correctness, constant-time behavior,
compatibility, independent review, or production readiness.

## TCB, axiom, and proof effects

The lexer, parser, analyzer, Core constructor, evaluator, and command-line
interface remain engineering trust dependencies. The option parser, the
selection of functions, the step report, and the width check on conversions
to numbers are new trusted code. Unit tests check exact messages at the new
limit, every index and slice boundary, conversions at and past the
significant-bit limit, selected evaluation and its step counts, every option
and usage error, and allocation failure. No axiom, theorem, proof rule,
certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

Costs are unchanged, so an evaluation still makes at most 64 elements for
each step it costs, and its memory is bounded by its budget. The largest
budget, 1,073,741,824 steps, may ask for more memory than a machine has;
every array and integer is reserved before it is written, and a failure is
`ORC0301` with no partial output. Literals of 65,536 elements are flat lists
bounded by the unchanged per-source token, node, event, and Core budgets.
Byte strings are decoded into storage no larger than their spelling and stop
at the 65,537th byte. The `--spec` list holds at most 64 names.

The reference evaluator is not constant-time, and no secrecy label or leakage
property is defined.

## Target and ABI effects

None.

## Standards, errata, and provenance

RFC 8439 appendices A.2 (test vector 2, 375 bytes), A.3 (test vectors 2 and
3), and A.5 (a 265-byte ciphertext with 12 bytes of associated data) are
reproduced byte for byte from the RFC's printed inputs; FIPS 203 and FIPS 204
object sizes motivate the slice. The fixture on lengths checks Pepin's test
for the Fermat prime 2^16 + 1 (3^32768 is −1) from a table of all 65,536
powers of 3. No standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3p_conformance.rs` binds the 11 rules of the
specification's index to evidence and fails on any drift. Three programs run
through `orangec check` and `eval` twice each, and generated tests exercise
every option, usage error, budget, selection, and report. Unit tests cover
lengths, literals, indices, slices, conversions, selected evaluation, option
parsing, and allocation. The S2 through S3o runners, the `orangec enc` tests,
and the algorithms corpus continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a function should cover more than 256 lengths, so that one
  `spec` serves every message up to 16 KiB byte by byte rather than block by
  block.
- Whether a source should be able to state its own budget, per function or
  per call.
- Whether updates of one element in a long array should cost less, with
  another way to bound memory.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. Arrays long enough for post-quantum keys,
RSA blocks, and the RFCs' own vectors, and the `orangec eval` controls that
other lanes asked for, were the next candidates after S3o's type parameters.
This proposal records the S3p surface built under that direction and is
presented for the owner's review. It is not accepted. Acceptance is the
owner's decision alone; until it is recorded here with a decision date,
reviewed revision, and `solo-reviewed` approval record, this proposal
authorizes nothing by itself.
