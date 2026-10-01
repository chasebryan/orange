---
number: OEP-0017
title: Orange 2026 byte order
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3n
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
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0017: Orange 2026 byte order

## Abstract

A conversion may name a **byte order**, `x as big T` or `x as little T`. The
words on one side, a word or an array of words, spell one number, first word
most significant for `big` and least significant for `little`, and the
conversion goes through that number: words become the words of another width
that spell it, the number itself as an `Int`, or its residue as a `Mod[m]`,
and a number becomes the words that spell its residue modulo 2 to the power
of their width. Words convert only to words of the same number of bits:

```orange
let head: Word[32]^16 = block as big Word[32]^16;
let initial: Word[32]^16 =
  ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16;
hash as big Word[8]^32
```

The normative text is [`docs/ORDER_2026.md`](../../ORDER_2026.md). An
implementation, eight programs, and an 8-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0016, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

Standards print bytes, and algorithms compute on words. FIPS 180-4 reads each
64-byte block as sixteen big-endian 32-bit words (SHA-256) or 128-byte block
as sixteen 64-bit words (SHA-512) and writes the digest the same way. RFC 8439
reads ChaCha20's key, counter, and nonce as little-endian words, serializes
the state as little-endian bytes, and reads each Poly1305 block as a
little-endian number; RFC 7748 decodes and encodes X25519 coordinates as
little-endian numbers.

Through S3m each of these was a function of shifts and ors over single bytes,
such as `load_le32(b0, b1, b2, b3)`, called once per word inside a loop, and
its inverse four shifts, `[x as Word[8], (x >> 8) as Word[8], (x >> 16) as
Word[8], (x >> 24) as Word[8]]`, called in another. Poly1305 read its blocks
as `for i in 0..16 with n: P = 0 { n * 256 + (b[15 - i] as P) }`. The
standards say "read as little-endian" in three words; the programs said it in
loops that a reader had to check index by index, and the evaluator ran them
step by step. S3l's own non-claims pointed here: "no conversion between bytes
and words (write `le32` or `be64` as a function of four or eight bytes)", and
bytes-to-words conversions were the next language candidate on the roadmap
after S3m.

With S3n the SHA-256, SHA-512, ChaCha20, Poly1305, and X25519 fixtures read
and write their words where the standards say to, each in one conversion, and
reproduce FIPS 180-4's, RFC 8439's, and RFC 7748's published values. The
three schemes of `orangec enc` read and write their words the same way; each
gives the same bytes and passes the same known answers, and sealing a
megabyte takes about a third of the time it did.

## Scope and non-goals

This proposal defines the byte orders `big` and `little` after `as`, array
targets after them, which pairs of types a byte order converts, the numbers
that words spell in each order, the `pack` node of the Typed Reference Core,
and its cost. It adds no token and reserves no word, and it adds one
diagnostic code, `ORC0240`, for words converted to words of a different
number of bits.

It does not define a byte order of a single value, conversions between arrays
of types other than words, a bit order or bit strings, words narrower than 8
or wider than 64 bits, reading words at a position computed from data,
conversions of tuples, or mixed orders. It makes no timing, secrecy, or
leakage claim and fixes no layout, calling convention, or ABI.

### Strata assumption

As for S3b through S3m, S3n assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A conversion in a
byte order is a total function between finite sets of values, defined from
the types alone, with no failure. It therefore has the same meaning under
`ST-REL`, `ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source
surface needs no change. Its cost depends only on its types, never on the
values converted.

## Specification

[`docs/ORDER_2026.md`](../../ORDER_2026.md) is the complete normative text.
In summary:

- **Syntax.** `conversion = prefixed "as" ( parsed_type | tuple_type | order
  declared_type )` with `order = "big" | "little"`. `big` and `little` are
  byte orders only directly after `as` and before `(` or a name other than
  `as` and `with`; anywhere else they are names. After a byte order the
  target may be an array type; without one, an array type after `as` is still
  `ORC0108`, whose note now names the byte order.
- **Types.** The operand's type is its first typed leaf's, as for every
  conversion, with array literals and fills typed by their elements and
  length. Words convert to words of the same width (`ORC0240` otherwise), to
  `Int`, and to `Mod[m]`, and `Int` and `Mod[m]` convert to words; any other
  pair is `ORC0215`, an untyped operand `ORC0220`, and a target other than
  the expected type `ORC0214`.
- **Meaning.** Words spell N = Σ x_i · 2^{n(k−1−i)} in `big` order and
  Σ x_i · 2^{n·i} in `little`; a conversion gives the words of the target
  that spell N, N itself, or N modulo m, and a number gives the words that
  spell its residue modulo 2^{nk}, a negative `Int` its two's complement.
- **Core and evaluation.** A conversion in a byte order is one `pack` node
  recording its operand's type and its order. It costs one step per 64 bits
  of width, or part of 64, and a conversion to `Mod[m]` also what `as Mod[m]`
  costs.

## Alternatives

Library functions written in Orange, `le32(b)` and `be64(b)` with S3m sizes
for arrays, were considered. They need one function for each pair of widths
and each order, each a loop that the evaluator runs byte by byte, and they
cannot give an `Int` or a `Mod[m]` of any width without a size parameter for
each. A conversion states the type the standard names, in the standard's
words, and costs a step per 64 bits.

A separate operator or built-in call, such as `bytes_to_words(x, big)`, was
rejected: Orange already writes every change of type with `as`, and a byte
order is an adverb of that change, as the standards phrase it ("read as
big-endian 32-bit words").

A byte order attached to types, as `Word[32, little]`, was rejected: a word
is a number, and a number has no byte order. Only a sequence of words spells
a number in an order, so the order belongs to the conversion that reads the
sequence.

## Compatibility and migration

Every source that S3m accepts converts with `as` followed by a type without a
length, and a `big` or `little` there was a type's name only where S3n still
reads it as one, so S3n accepts it with the same Core values, messages, and
output bytes. A source that S3m rejects gets the same diagnostics, except
that `big` or `little` after `as` and before a type is now a byte order, the
`ORC0108` note for an array type after `as` gains a sentence naming the byte
order, and the `ORC0215` notes for a conversion of an array of words, and to
one, now name the byte order that converts them. `orangec lex` is unchanged.

The public Rust API gains, in the `parser` module, `ByteOrder` and
`ConversionExpression::order` and `order_span`; in the `core` module the
`CoreNodeKind::Pack` variant, `CoreType::words`,
`ExactInteger::magnitude_limbs`, and `ExactInteger::from_limbs`; and the
`DiagnosticCode` variant `PackedWidth`. Code that matches every variant of
these enums must follow them.

Rollback reverts the parser, analyzer, Core, evaluator, schemes, tests,
fixtures, and normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to conversions in a byte order. The
supported claim remains deterministic, bounded analysis and evaluation of the
documented fragment at a recorded implementation revision. It establishes no
soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness.

## TCB, axiom, and proof effects

The lexer, parser, analyzer, Core constructor, evaluator, and command-line
interface remain engineering trust dependencies. The parsing of byte orders,
the checking of conversions in a byte order, and the evaluator's packing and
unpacking of words are new trusted code. Unit tests check exact spans and
messages, the contextual words, every admitted and rejected pair of types,
events, a reference for every pair of widths in both orders, residues of
negative and oversized numbers and of residues, the widest array and integer,
the cost table, allocation failure, the deepest sources the limits admit on a
1 MiB stack, and spans that do not belong to their source. No axiom, theorem,
proof rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

A conversion in a byte order converts at most 16,384 bits, the width of
`Word[64]^256` and the significant-bit limit of `Int`, and costs at most 256
steps plus the residue cost of `Mod[m]`, so it cannot ask for unbounded work.
The evaluator reserves its storage before writing, and an allocation failure
is a resource diagnostic with no partial output. The byte order costs one
semantic event and the conversion one Core node, within the unchanged
per-source budgets.

The reference evaluator is not constant-time, and no secrecy label or leakage
property is defined. The cost of a conversion in a byte order depends only on
its types, never on the values it converts.

## Target and ABI effects

None. A byte order is a statement about mathematical values; how a backend
would load and store words, and in which order memory holds them, is left to
D-011 and later slices.

## Standards, errata, and provenance

FIPS 180-4 sections 3.1, 5.1.1, 5.1.2, and 6, RFC 8439 sections 2.3, 2.4,
2.5, and 2.6, and RFC 7748 section 5 motivate the slice. The fixtures check
FIPS 180-4's SHA-256 and SHA-512 digests of "abc" and of its two-block
messages, the SHA-256 and SHA-512 digests of a message of one block's
padding limit and of the longest message each admits, computed by an
independent implementation, the ChaCha20 block and encryption of RFC 8439
sections 2.3.2 and 2.4.2, the Poly1305 tag of section 2.5.2 and two of the
vectors of its Appendix A.3, and the X25519 vector of RFC 7748 section 5.2.
No standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3n_conformance.rs` binds the 8 rules of the
specification's index to evidence and fails on any drift. Eight programs run
through `orangec check` and `eval` twice each, and a generated program
converts words of every width in both orders to words of every width and to
`Int`, against a reference computed in the runner, converts the widest array
to `Int` and back, and shows that an array of one element more is no type.
Unit tests cover the parser, types, Core construction, evaluation, costs,
events, the deepest admitted sources, allocation failure, and foreign spans.
The S2 through S3m runners, the `orangec enc` tests, and the algorithms corpus
continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added. The encryption schemes of
`orangec enc` change their code, not their output: files sealed before open
after, and files sealed after open before.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a byte order should also apply between arrays of residues and
  words, as for field elements serialized one after another.
- Whether words should be read at a position computed from data, as
  `x[i..i + 4] as big Word[32]` with `i` a value.
- Whether bit strings and bit orders, as some hash functions and ciphers
  number their bits, belong in the language.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. Conversions between bytes and words were the
next language candidate on the roadmap after S3m. This proposal records the
S3n surface built under that direction and is presented for the owner's
review. It is not accepted. Acceptance is the owner's decision alone; until it
is recorded here with a decision date, reviewed revision, and `solo-reviewed`
approval record, this proposal authorizes nothing by itself.
