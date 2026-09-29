---
number: OEP-0007
title: Orange 2026 fixed-length arrays
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-28
updated: 2026-09-28
discussion: owner-direction-2026-09-28-s3d
related-decisions:
  - D-002
  - D-004
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0007: Orange 2026 fixed-length arrays

## Abstract

A parameter, result, or binding may have a fixed-length array type `T^n`,
where `T` is `Int` or a word type and n is a decimal length from 1 through 256.
An array literal `[e0, e1, ...]` lists every element, and `x[k]` selects the
element at a literal index k, which the analyzer proves in range. Operators
and conversions apply to elements, never to whole arrays.

With this slice, the whole ChaCha20 block function reads as RFC 8439 describes
it, with the state as one `Word[32]^16`:

```orange
spec double_round(x: Word[32]^16) -> Word[32]^16 {
  let q0: Word[32]^4 = quarter_round(x[0], x[4], x[8], x[12]);
  let q1: Word[32]^4 = quarter_round(x[1], x[5], x[9], x[13]);
  let q2: Word[32]^4 = quarter_round(x[2], x[6], x[10], x[14]);
  let q3: Word[32]^4 = quarter_round(x[3], x[7], x[11], x[15]);
  let d0: Word[32]^4 = quarter_round(q0[0], q1[1], q2[2], q3[3]);
  let d1: Word[32]^4 = quarter_round(q1[0], q2[1], q3[2], q0[3]);
  let d2: Word[32]^4 = quarter_round(q2[0], q3[1], q0[2], q1[3]);
  let d3: Word[32]^4 = quarter_round(q3[0], q0[1], q1[2], q2[3]);
  [
    d0[0], d1[0], d2[0], d3[0],
    d3[1], d0[1], d1[1], d2[1],
    d2[2], d3[2], d0[2], d1[2],
    d1[3], d2[3], d3[3], d0[3],
  ]
}
```

The normative text is [`docs/ARRAYS_2026.md`](../../ARRAYS_2026.md). An
implementation, 8 fixtures, and a 17-rule conformance runner accompany it so
that the proposal can be reviewed against running code. This proposal is in
**Review** and requires OEP-0006, which is also in review. It accepts no D-004
candidate and gives the Typed Reference Core no canonical or proof role.

## Motivation

S3c can name every step of a quarter round, but it cannot return one. Its
ChaCha20 fixture needs four functions to produce the four output words, each
recomputing part of the round, and no S3c program can hold a sixteen-word
state. Every flagship algorithm works on a state: ChaCha20 on a 4 by 4 matrix,
SHA-256 on eight working variables and a sixteen-word block, AES on a 4 by 4
byte matrix, Keccak on twenty-five lanes. A specification language for
cryptography needs a value that is the whole state.

Fixed length is the point, not a limitation. A standard's state has a length
the standard states, and a reader checking a transcription wants that length in
the type. Literal indices keep every read visible and make an out-of-range
access a compile-time error rather than a run-time one.

## Scope and non-goals

This proposal defines array types `T^n` for scalar `T` and 1 ≤ n ≤ 256, array
literals, literal indices of names and calls, their Core form, their
evaluation and display, diagnostics `ORC0221` through `ORC0224`, and a
256-element literal limit.

It does not define arrays of arrays, empty arrays, variable or computed
indices, slices, concatenation, comparison, elementwise operators, loops,
comprehensions, length polymorphism, tuples or records of mixed types,
byte-string literals, mutation or in-place update, or any exclusion of
OEP-0006.

### Strata assumption

As for S3b and S3c, S3d assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A fixed-length array is
a finite product of value domains, and a literal index is a projection, so
under `ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST` alike the
source surface needs no change. Placement of the Core remains D-004's decision.

## Specification

[`docs/ARRAYS_2026.md`](../../ARRAYS_2026.md) is the complete normative text.
In summary:

- **Grammar.** `declared_type = parsed_type ("^" INTEGER)?` for parameters,
  results, and bindings. `primary` gains `IDENTIFIER index?`, `call index?`,
  and `array = "[" expression ("," expression)* ","? "]"`, with
  `index = "[" INTEGER "]"`. A conversion target stays a scalar `parsed_type`,
  so `^` after it is still exclusive or.
- **Types.** `T^n` is admitted for the five scalar types and a decimal n from
  1 through 256 with no leading zero (`ORC0221` otherwise). Arrays of arrays
  and empty arrays are syntax errors.
- **Typing.** An array literal lists exactly n elements of type `T`
  (`ORC0222`, then each element). An index applies only to an array
  (`ORC0224`), is below its length (`ORC0223`), and has the element type
  (`ORC0214`). No operator or conversion is defined on an array (`ORC0215`).
- **Meaning.** Elements are evaluated left to right; an index selects one.
  Arrays are values with no aliasing. `orangec eval` prints `T^n = [e0, ...]`.
- **Core.** Core types gain `Array`, and Core gains `array` and `index` nodes.
  A function without arrays has exactly its S3c Core.
- **Limits.** At most 256 elements per literal (`ORC0106`). An array literal
  opens a nesting level. One semantic event per length token, per array
  literal, and per index, plus the index literal's decoding events. An array of
  n elements costs n steps, an index one. The deepest admitted sources still fit
  in 1 MiB of stack.

## Alternatives

Writing array types as `[Word[32]; 16]`, as Rust does, or `Word[32][16]`, as C
does, was considered. `T^n` is the notation of the mathematics a cryptographer
already reads: (Z/2^32 Z)^16 is a state of sixteen words. It also keeps `[`
for the one thing brackets already mean in Orange types, a word width.

Variable indices were rejected for this slice. A variable index needs either a
run-time range check, which introduces a failure value Orange does not have,
or a type of indices below n, which is a larger design. Every S3d index is
checked when the program is analyzed, and every position a specification reads
is visible in its text.

Elementwise operators, such as `x ^ y` on two arrays, were rejected. They would
make an operator's meaning depend on whether its operands are words or arrays,
and a reader could no longer tell from an operator alone that it acts on one
ring element.

Arrays of arrays were deferred. AES and Keccak are naturally two-dimensional,
but both standards also index their state linearly, and a one-dimensional array
with written positions covers them. Nesting belongs with a decision about
slicing and update.

Unbounded lengths were rejected. The 256 limit covers every flagship state and
key schedule in its natural form, matches the argument limit of a call, and
keeps every array literal inside the parser's existing budgets.

## Compatibility and migration

Every source that S3c accepts is accepted by S3d with the same Core values and
output bytes: `^` after a declared type and `[` after a name, a call, or at the
start of an operand were never valid, and `^` after a conversion target keeps
its meaning. A source that S3c rejects gets the same diagnostics, except that
the expression-nesting message now names arrays. The public Rust API gains
`ArrayType`, `CoreArray`, `ArrayExpression`, `IndexExpression`,
`MAX_ARRAY_LENGTH`, `MAX_ARRAY_ELEMENTS`, an `Array` variant of `CoreType` and
`CoreValue`, and two `ExpressionKind` and `CoreNodeKind` variants.
`CoreType` gains a `Display` implementation in place of `as_str`, because an
array type's spelling is not a fixed string.

Rollback reverts the parser, semantics, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to fixed-length arrays and literal indices.
The supported claim remains deterministic, bounded analysis and evaluation of
the documented fragment at a recorded implementation revision. It establishes
no soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, and evaluator remain engineering trust
dependencies. Array construction and selection are new trusted code; the
evaluator carries each array's type and fails closed on any Core whose array
counts, element types, positions, or result types disagree. No axiom, theorem,
proof rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

The hostile-frontend surface grows by array types, literals of up to 256
elements, and indices. The element limit, the shared nesting and height limits,
per-element step costs, and a 1 MiB stack bound tested with 64 levels of nested
arrays and indexed calls constrain them. An index literal is decoded under the
existing significant-bit limit before its range is checked. No secrecy label or
leakage property is defined.

## Target and ABI effects

None. An array is a mathematical tuple, not a memory layout.

## Standards, errata, and provenance

RFC 8439 sections 2.1, 2.2, 2.3, and 2.3.2 and FIPS 180-4 sections 4.2.2,
5.3.3, 6.2.2, and the NIST "abc" worked example motivate the forms. The
fixtures check the whole ChaCha20 block against RFC 8439 section 2.3.2 and the
SHA-256 message schedule and first two rounds against the "abc" example. No
standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3d_conformance.rs` binds the 17 rules of the
specification's index to evidence and fails on any drift. Eight fixtures and
one generated boundary case run through `orangec check` and `eval` twice each.
Unit tests cover spans, grammar errors, length resolution, literal and index
typing, operators and conversions on arrays, evaluation order, display, exact
event, node, and step accounting, allocation and foreign-input failure,
inconsistent Core, and the 1 MiB stack bound. The S2, S3a, S3b, and S3c
runners continue to pass, with the S3b nesting message updated as described
above.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether bounded loops, or a fold over a literal range, should come next, so
  that ten double rounds are one expression.
- Whether a functional update, such as `x with [3] = v`, belongs in the
  specification stratum.
- Whether a later edition should admit arrays of arrays for AES and Keccak, and
  how their indices would be written.
- Whether byte-string literals should be a separate form for keys and nonces.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals,
including the development steps after S3b, and that development not freeze
unless the owner asks. This proposal records the S3d surface built under that
direction and is presented for the owner's review. It is not accepted.
Acceptance is the owner's decision alone; until it is recorded here with a
decision date, reviewed revision, and `solo-reviewed` approval record, this
proposal authorizes nothing by itself.
