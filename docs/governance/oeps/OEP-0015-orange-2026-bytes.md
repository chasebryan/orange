---
number: OEP-0015
title: Orange 2026 bytes
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3l
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0015: Orange 2026 bytes

## Abstract

A byte string `"Hi There"` is the array `Word[8]^8` of the ASCII bytes of its
text, and a hex string `hex"07000000 40414243"` the array of its hex digit
pairs. `a ++ b` joins two arrays of one element type. `x[a..b]` is the run of
elements of `x` from index a up to, but not including, index b, and `x with
[a..b] = v` replaces that run. A slice's bounds are integer literals and
loop indices combined by `+`, `-`, and `*` by a constant, so its length is
fixed and every element it takes is proved to exist before the program runs:

```orange
spec abc() -> Word[8]^32 { hash64("abc" ++ hex"80" ++ [0; 52] ++ hex"00000000 00000018") }

spec schedule(block: Word[8]^64) -> Word[32]^64 {
  for t in 0..16 with w: Word[32]^64 = [0; 64] { w with [t] = word(block[4 * t..4 * t + 4]) }
}
```

The normative text is [`docs/BYTES_2026.md`](../../BYTES_2026.md). An
implementation, six programs, and a 10-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0014, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

Standards print their inputs as bytes, and their algorithms move runs of
bytes. RFC 4231 keys HMAC with the text "Jefe" and signs "what do ya want for
nothing?". FIPS 180-4 section 5.1.1 pads a message with the byte 80, zeros,
and its length, and reads each 64-byte block as sixteen big-endian words. RFC
8439 section 2.8.2 prints its plaintext as a sentence and its key, nonce, and
additional data in hex; section 2.6 takes the Poly1305 key as the first 32
bytes of a ChaCha20 block, and section 2.8 joins the additional data, the
ciphertext, their padding, and both lengths before Poly1305 reads them
sixteen bytes at a time.

Through S3k an Orange program wrote each of these as an array of numbers, one
element per byte, and moved a run of bytes with a loop that updated one
element per step. The text and the hex that a reader checks against the
standard were not in the program, and every join and every block was a loop
of index arithmetic. The encryption lane asked for array concatenation and
slicing, and the Daylight lane for byte-string literals, for this reason.

With S3l the HMAC fixture writes RFC 4231's keys and messages as the RFC
prints them, pads SHA-256's input with `++`, and reads each block through
slices. The ChaCha20-Poly1305 fixture writes RFC 8439's sunscreen sentence as
text and its key, nonce, and additional data in hex, reads key and nonce
words through four-byte slices, joins the key stream and Poly1305's input
with `++`, and takes the one-time key as `block(key, 0, nonce)[..32]`. Both
reproduce their published values, and the AEAD's tag verifies.

## Scope and non-goals

This proposal defines byte strings and hex strings, the concatenation of
arrays, slices, and slice updates: their lexical and syntactic forms and
limits, the decoding of byte strings, typing, the static rule for slice
bounds, the meaning of each form, their record in the Typed Reference Core,
and the evaluator's step costs. It adds two tokens, `HEX_STRING` and
`PLUS_PLUS`, and three diagnostic codes: `ORC0009` for a malformed hex
string, `ORC0235` for a character a byte string cannot hold, and `ORC0236`
for a slice whose bounds are not a fixed positive distance apart. It
reserves no word: `hex` is still a name wherever a quote does not follow it
directly.

It does not define slices at positions computed from data, slices with a
step or counted from the end, text beyond printable ASCII or any string
encoding, empty arrays or empty strings, arrays of more than 256 elements,
equality of whole arrays, conversions between bytes and words, the display of
bytes as text, `++` of tuples, or size parameters. It makes no timing,
secrecy, or leakage claim and fixes no layout, byte order, or ABI.

### Strata assumption

As for S3b through S3k, S3l assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A byte string is a
constant array, `++` is the concatenation of finite sequences, and a slice
and a slice update are a restriction and an override at positions fixed
before the program runs, so each has the same meaning under `ST-REL`,
`ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source surface needs
no change. Because no slice's position depends on data, none raises the
secret-index question that S3g settles for indices.

## Specification

[`docs/BYTES_2026.md`](../../BYTES_2026.md) is the complete normative text.
In summary:

- **Lexical.** `hex` followed directly by `"` begins a hex string of hex digit
  pairs and spaces; a malformed one is `ORC0009` at its first offense, and an
  unterminated one `ORC0003` at its opening. `++` is one token. Strings keep
  the escapes of S2.
- **Syntax.** A string or a hex string is an expression. `++` is a binary
  operator of its own group, associating to the left and mixing with no
  other operator without parentheses (`ORC0108`). A slice `x[a..b]`,
  `x[a..]`, or `x[..b]` follows a name, a call, or a projection and is
  followed by nothing; an update may replace a range as well as an index.
  Every malformed form is `ORC0101` with a note that shows the form.
- **Byte strings.** A byte string holds 1 through 256 bytes (`ORC0221`), each a
  printable ASCII character or an escape (`ORC0235`, with the character's
  bytes written as a hex string), and has type `Word[8]^n` (`ORC0214`,
  `ORC0222`).
- **Typing.** `a ++ b` is an array of the required type whose length is the
  sum of its operands' (`ORC0214`, `ORC0222`, `ORC0224`). A slice is an array
  of its base's element type and its own length, and a slice update has its
  base's type; the value of a slice update has the slice's length.
- **Static bounds.** A slice's bounds are built from integer literals and loop
  indices with `+`, `-`, and `*` by a constant (`ORC0226`), are the same
  positive distance apart at every step (`ORC0236`), and lie within the array
  over every value of the loop indices (`ORC0223`).
- **Core and evaluation.** A byte string is one array literal, built once and
  shared. Core gains `concat`, `slice`, and `slice_update` nodes; an omitted
  bound is a literal. A join, a slice, and a slice update cost one step per
  64 elements they copy, as an update and a fill do since S3g.

## Alternatives

Byte strings typed as a separate `Bytes` type were considered. Every byte
array in the fixtures is already `Word[8]^n`, and a second type would need
conversions at every call of a function that takes bytes. A byte string is
therefore only a way to write `Word[8]^n`.

Text in any Unicode character, encoded as UTF-8, was deferred. The standards
in hand print ASCII text and hex; admitting other characters would make a
program's bytes depend on an encoding its reader cannot see. A byte string
that holds such a character is rejected with its UTF-8 bytes written out as a
hex string, which the author can paste in their place.

Slices at positions computed from data, as `x[n..n + 4]` with `n` a
parameter, were deferred. Proving such a slice in range needs ranges of
parameters that Orange 2026 does not have, and a backend would have to
compile a possibly secret position to a scan of the whole array, as S3g
requires for indices. Every slice the fixtures need is at a position fixed by
literals and loop indices.

Concatenation written as a function, `concat(a, b)`, was rejected: a message
built from six parts reads as the standard writes it with an operator and as
nested calls without one. `++` was chosen over `||`, the concatenation of
many papers, because `||` is already Orange's logical or.

## Compatibility and migration

Every source that S3k accepts has no string, hex string, `++`, or range in
brackets, so S3l accepts it with the same Core values, messages, and output
bytes. A source that S3k rejects gets the same diagnostics, except that the
forms S3l defines, which were `ORC0101`, are now accepted or reported by the
S3l rules; a hex string that is not well formed, which was lexed as `hex` and
a string, is now `ORC0009` or `ORC0003`. `orangec lex` gives one `PLUS_PLUS`
token where it gave two `PLUS` tokens and one `HEX_STRING` token where it gave
an identifier and a string.

The public Rust API gains, in the `parser` module, `ByteString`,
`SliceRange`, `SliceExpression`, and `SliceUpdateExpression`, the
`ExpressionKind` variants `Bytes`, `Slice`, and `SliceUpdate`, and
`BinaryOperator::Concat` with `BinaryOperator::is_concatenation`; the
`TokenKind` variants `HexString` and `PlusPlus`; the `CoreNodeKind` variants
`Concat`, `Slice`, and `SliceUpdate`; and the `DiagnosticCode` variants
`MalformedHexString`, `UnprintableByteString`, and `SliceLength`. Code that
matches every variant of these enums must handle the new ones.

Rollback reverts the lexer, parser, analyzer, Core, evaluator, tests,
fixtures, and normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to byte strings, joins, slices, and slice
updates. The supported claim remains deterministic, bounded analysis and
evaluation of the documented fragment at a recorded implementation revision.
It establishes no soundness, proof, refinement, compilation, cryptographic
correctness, constant-time behavior, compatibility, independent review, or
production readiness.

## TCB, axiom, and proof effects

The lexer, parser, analyzer, Core constructor, evaluator, and command-line
interface remain engineering trust dependencies. The hex string lexer, the
parsing of byte strings, joins, slices, and slice updates, the decoding of
byte strings, the typing of joins and slices, the static analysis of slice
bounds, the Core nodes, and the evaluator's joins and slices are new trusted
code. The analyzer's unit tests moved into their own file so that no source
file exceeds the repository's size cap; the move changed no test. Unit tests
check exact tokens, spans, and messages, limits and heights, decoding at the
256-byte limit, the static rule and its checking order, node and event
accounting, evaluation and step costs, the deepest sources the limits admit
on a 1 MiB stack, inconsistent Core, allocation failures, and spans that do
not belong to their source. No axiom, theorem, proof rule, certificate,
checker, or solver is introduced.

## Threat, abuse, and leakage effects

A byte string holds at most 256 bytes, and decoding stops at the 257th, so a
long spelling is never read to its end. Joins, slices, and slice updates are
arrays of at most 256 elements, and each costs semantic events, Core nodes,
and steps, so the analyzer's and evaluator's memory stays bounded. A chain of
joins is checked in one frame, and the deepest admitted sources, accepted or
rejected, run within 1 MiB of stack. Every slice is proved in range before a
program runs, and the evaluator still checks each against its Core: Core
whose joins or slices are inconsistent with their types stops evaluation with
no values, and an allocation failure gives no Core or no values. The step
budget of 1,048,576 is unchanged.

The reference evaluator is not constant-time, and no secrecy label or leakage
property is defined. A slice's position never depends on data, so a slice
reveals nothing through its position that its program's text does not.

## Target and ABI effects

None. A byte string is a value of the specification; how a compiled program
lays it out, and the byte order of words, are left to D-011 and later
slices.

## Standards, errata, and provenance

FIPS 180-4 sections 5.1.1 and 6.2.2, RFC 2104, RFC 4231 section 4, and RFC
8439 sections 2.1 through 2.8 motivate the slice. The fixtures check the
SHA-256 digest of "abc" of FIPS 180-4's examples, the HMAC-SHA-256 values of
RFC 4231 test cases 1 and 2, and the ciphertext and tag of RFC 8439 section
2.8.2, and verify that tag. No standard gains normative authority through
this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3l_conformance.rs` binds the 10 rules of the
specification's index to evidence and fails on any drift. Six programs run
through `orangec check` and `eval` twice each, and generated programs check
byte strings and hex strings of 256 bytes and of 257 and byte strings holding
a raw tab and a raw delete. Unit tests cover the lexer, the parser, typing,
the static rule, Core construction, evaluation and its step costs, the
deepest admitted sources, inconsistent Core, allocation failure, and foreign
spans. The S2 through S3k runners, which now find the analyzer's unit tests
in `semantics/tests.rs`, and the algorithms corpus continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether slices at positions computed from data should be added, with a
  bound on the position and a whole-array scan for a secret one.
- Whether text beyond ASCII should be written in byte strings, and in which
  encoding.
- Whether conversions between bytes and words, such as `le32` and `be64`,
  should be built in or stay functions written in Orange.
- Whether arrays, and so byte strings, should hold more than 256 elements.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. Array concatenation and slicing were a
language request of the encryption lane and byte-string literals one of the
Daylight lane, pending after S3k. This proposal records the S3l surface built
under that direction and is presented for the owner's review. It is not
accepted. Acceptance is the owner's decision alone; until it is recorded here
with a decision date, reviewed revision, and `solo-reviewed` approval record,
this proposal authorizes nothing by itself.
