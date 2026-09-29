---
number: OEP-0006
title: Orange 2026 bindings and conversions
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-28
updated: 2026-09-28
discussion: owner-direction-2026-09-28-s3c
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0006: Orange 2026 bindings and conversions

## Abstract

A typed `spec` body may begin with `let` bindings, each with a name, a stated
type, and a value, and ends with its result expression as before. Bindings are
immutable, are evaluated once in order, and never shadow a parameter or each
other. An explicit conversion `e as T` moves a value between any two of `Int`,
`Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`: it keeps the integer value
and, into a word type, takes the residue modulo 2^n. Nothing converts
implicitly.

With this slice, the ChaCha20 quarter round reads as RFC 8439 prints it, and a
word is assembled from bytes in the order a standard names:

```orange
edition 2026;
module chacha20 {
  spec quarter_a(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    a1 + b1
  }

  spec load_le32(b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8]) -> Word[32] {
    (b0 as Word[32]) | ((b1 as Word[32]) << 8) | ((b2 as Word[32]) << 16)
      | ((b3 as Word[32]) << 24)
  }
}
```

The normative text is [`docs/BINDINGS_2026.md`](../../BINDINGS_2026.md). An
implementation, 10 fixtures, and a 17-rule conformance runner accompany it so
that the proposal can be reviewed against running code. This proposal is in
**Review** and requires OEP-0005, which is also in review. It accepts no D-004
candidate and gives the Typed Reference Core no canonical or proof role.

## Motivation

S3b can compute every word operation of SHA-2 and ChaCha20, but it cannot name
an intermediate value. Its ChaCha20 fixture has to recompute `a + b` through a
chain of helper functions, because the only way to name a value is to make it a
function. A reader checking a transcription against RFC 8439 or FIPS 180-4
compares named steps; a specification should offer the same names.

S3b also cannot move a value between types. Every standard that works with
bytes and words needs that: ChaCha20 reads its key as little-endian 32-bit
words, SHA-256 reads its message as big-endian words and appends a 64-bit
length, and field arithmetic moves between machine words and integers. Without
conversions, none of these can be written.

## Scope and non-goals

This proposal defines `let` bindings in typed `spec` bodies, their scope and
typing, explicit `as` conversions among the five admitted types, their Core
form, their evaluation, diagnostics `ORC0219` and `ORC0220`, and a 256-binding
limit.

It does not define tuples, booleans, comparisons, conditionals, loops,
mutation or reassignment, shadowing, type inference, signed words, sign
extension, arbitrary word widths, division, remainder, variable shift or
rotation amounts, recursion, typed `impl` declarations, contracts, effects,
secrecy labels, failure values, imports, proofs, claims, canonical Core, code
generation, targets, or any cryptographic claim.

### Strata assumption

As for S3b, S3c assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A `let` binding is a
pure local definition and a conversion is a total function between value
domains, so under `ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST`
alike the source surface needs no change. Placement of the Core remains
D-004's decision.

## Specification

[`docs/BINDINGS_2026.md`](../../BINDINGS_2026.md) is the complete normative
text. In summary:

- **Grammar.** `typed_tail = "->" parsed_type "{" binding* expression "}"`,
  with `binding = "let" IDENTIFIER ":" parsed_type "=" expression ";"`, and a
  new expression form `conversion = prefixed "as" parsed_type`.
- **No new reserved words.** `let` starts a binding only at the start of a body
  item and before a name; `as` converts only after a complete operand.
  Elsewhere both are ordinary names, so every S3b program keeps its meaning.
- **Grouping.** A conversion applies to one prefixed operand and is a group of
  its own: `x + y as Word[32]` is `ORC0108`, and the reader chooses between
  `(x + y) as Word[32]` and `(x as Word[32]) + (y as Word[32])`, which differ.
- **Scope.** A binding is in scope after its own `;`. Parameters and bindings
  share one set of names with no shadowing (`ORC0219`). A name used before its
  binding is `ORC0211`, citing the binding.
- **Typing.** A binding's value is checked against its stated type. A
  conversion's target must be the expected type (`ORC0214`), and its operand
  has the type of its first name, call, or conversion from left to right, which
  every S3b operator shares with its result. An operand with no such leaf, such
  as `(1 + 2) as Word[8]`, is `ORC0220`.
- **Meaning.** Bindings are evaluated once, in order, before the result. `as`
  takes the operand's integer value (a word's canonical residue) and, into
  `Word[n]`, its residue modulo 2^n. The `Int` value -1 converts to `Word[8]`
  as `0xff`; widening keeps the value; narrowing keeps the low bits.
- **Core.** Core functions gain typed locals, and Core gains `local` and
  `convert` nodes. A function without bindings has exactly its S3b Core.
- **Limits.** At most 256 bindings per body (`ORC0106`). One semantic event per
  binding-name check and per conversion, plus parsed-type events. One step per
  binding read and per conversion. The deepest admitted sources still fit in
  1 MiB of stack.

## Alternatives

Inferred binding types, as in `let t = a + b;`, were rejected. They save a few
characters and hide the one fact a reader most needs when checking a
transcription. They would also need literal defaulting, which S3b deliberately
does not have.

Shadowing was rejected. Standards that update a variable in place, like the
quarter round's `a += b`, are clearer as `a1`, `a2` than as a reused `a`, and
unique names make every reference in a body unambiguous.

Reserving `let` and `as` as keywords was rejected for this edition. It would
invalidate S3b programs that use them as names, with no gain in clarity:
position already decides their meaning without lookahead beyond two tokens.

Implicit widening from a narrower word to a wider one was rejected. It is safe
for values but not for arithmetic: `x + y` then silently means a different ring
depending on context, which is exactly the ambiguity the grouping rule of
section 4 removes.

Distinct operations such as `widen`, `truncate`, and `to_int` were considered.
One operation with one rule, the integer value and then the residue, covers
every case and reads the way standards describe byte and word handling.

Signed interpretation and sign extension were deferred. No flagship algorithm
needs them in its specification, and they belong with a decision about signed
word types.

## Compatibility and migration

Every source that S3b accepts is accepted by S3c with the same Core values and
output bytes. A source that S3b rejects gets the same diagnostics unless it
contains `let` followed by a name at the start of a typed body, or `as`
directly after a complete operand; those spellings were never valid. The
`ORC0211` message differs only in bodies with bindings. The public Rust API
gains `Binding`, `ConversionExpression`, `CoreLocal`, `MAX_BINDINGS_PER_BODY`,
and two `ExpressionKind` and `CoreNodeKind` variants.

Rollback reverts the parser, semantics, Core, evaluator, tests, fixtures, and
normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to bindings and conversions. The supported
claim remains deterministic, bounded analysis and evaluation of the documented
fragment at a recorded implementation revision. It establishes no soundness,
proof, refinement, compilation, cryptographic correctness, constant-time
behavior, compatibility, independent review, or production readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, and evaluator remain engineering trust
dependencies. The conversion arithmetic is new trusted code, tested against an
independent 128-bit reference for every pair of types. No axiom, theorem,
proof rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

The hostile-frontend surface grows by up to 256 bindings per body and by
conversions. The binding limit, the unchanged nesting and height limits for
each binding's value, per-read step costs, and a 1 MiB stack bound tested with
nested conversions and a full set of deeply nested bindings constrain them. No
secrecy label or leakage property is defined.

## Target and ABI effects

None. A conversion is a function between mathematical value domains, not a
machine instruction.

## Standards, errata, and provenance

RFC 8439 sections 2.1, 2.1.1, 2.3, and 2.3.2 and FIPS 180-4 sections 3.1,
5.1.1, and 6.2.2 motivate the forms. The fixtures check the ChaCha20 quarter
round against RFC 8439 section 2.1.1, a little-endian key word against section
2.3.2, and SHA-256 round 0 and message words against the FIPS 180-4 "abc"
example. No standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3c_conformance.rs` binds the 17 rules of the
specification's index to evidence and fails on any drift. Ten fixtures and one
generated boundary case run through `orangec check` and `eval` twice each.
Unit tests cover spans, grouping, contextual names, scope, leaf typing, every
conversion pair against a 128-bit reference, evaluation order, exact event,
node, and step accounting, allocation and foreign-input failure, inconsistent
Core, and the 1 MiB stack bound. The S2, S3a, and S3b runners continue to pass
unchanged.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether tuples or fixed-size records should come next, so that one function
  can return a whole ChaCha20 quarter round or SHA-256 round.
- Whether a later edition should reserve `let` and `as` once more syntax
  depends on them.
- Whether signed word interpretation belongs in the specification stratum.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals,
including the development steps after S3b, and that development not freeze
unless the owner asks. This proposal records the S3c surface built under that
direction and is presented for the owner's review. It is not accepted.
Acceptance is the owner's decision alone; until it is recorded here with a
decision date, reviewed revision, and `solo-reviewed` approval record, this
proposal authorizes nothing by itself.
