# Orange 2026 byte order specification

Status: proposed S3n semantics under OEP-0017, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3n of Orange 2026: **conversions in a byte
order**, `x as big T` and `x as little T`, which read words as the words of
another width, as a number, or as a residue, and write a number or a residue
as words, most significant word first or least significant word first. It is
a delta over the proposed S3m rules in [`SIZES_2026.md`](SIZES_2026.md), which
are a delta over [`BYTES_2026.md`](BYTES_2026.md) and the documents it
extends. Everything those documents define and this one does not mention is
unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0017](governance/oeps/OEP-0017-orange-2026-byte-order.md), which
requires OEP-0016. At that point it replaces the S3m clauses listed in
section 11. Until then, the compiler behavior it describes exists so that the
proposal can be reviewed against running code, and it establishes no accepted
language meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md), proposed under
> OEP-0018, extends this document with type parameters, which may stand for
> the target of a conversion in a byte order, each instance converting to its
> own type: `b as big W` with `W in {Word[32]^2, Word[64]}` reads eight bytes
> as two words in one instance and as one in the other. Every source this
> document accepts keeps its meaning under it.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Standards print bytes, and algorithms compute on words. FIPS 180-4 reads each
64-byte block of a message as sixteen 32-bit words, the first byte most
significant, and writes the digest the same way. RFC 8439 reads ChaCha20's
key, counter, and nonce as little-endian words and serializes the state as
little-endian bytes; Poly1305 reads each sixteen-byte block as a little-endian
number, and RFC 7748 decodes an X25519 coordinate the same way. Through S3m
every such reading was a function of shifts and ors over single bytes, such
as `load_le32(b0, b1, b2, b3)`, called once per word in a loop.

S3n writes it where the standard says it. A conversion names a **byte order**
after `as`:

```orange
// FIPS 180-4 section 6.2.2, step 1: a block's first sixteen message words.
let head: Word[32]^16 = block as big Word[32]^16;

// RFC 8439 section 2.3: ChaCha20's state is the constant, the key, the block
// counter, and the nonce, read as sixteen little-endian words.
let initial: Word[32]^16 =
  ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16;

// RFC 8439 section 2.5.1: a whole block is its sixteen bytes, little-endian,
// with the byte 0x01 above them, as an element of Poly1305's field.
(m[16 * j..16 * j + 16] ++ hex"01") as little P

// RFC 7748 section 5: the u-coordinate, its top bit masked, little-endian.
let x_1: F = (u with [31] = u[31] & 127) as little F;
```

Words are numbers, and nothing in Orange has a byte order of its own: a
`Word[32]` is an integer from 0 through 2^32 − 1, and an array is a sequence
of values. A byte order says only how a sequence of words spells one number,
first word most significant (`big`) or least significant (`little`), and a
conversion in that order goes through that number. So words of one width
become words of another, `Word[8]^64` sixteen `Word[32]` or eight `Word[64]`,
with every bit kept, and a number becomes words by its residue, exactly as
`as Word[32]` already gives an `Int`'s residue modulo 2^32.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3m extends it, is
unchanged: a conversion in a byte order is checked from the types of its
operand and target alone, before anything runs, and has no run-time failure.
Every S3m source keeps its meaning (section 11).

## 3. Grammar

S3n adds no token and no reserved word. The conversion production of
`BINDINGS_2026.md` section 3, as `TUPLES_2026.md` extends it, becomes, in the
names of the grammar of The Orange Book:

```text
conversion = prefixed "as" ( parsed_type | tuple_type | order declared_type ) ;
order      = "big" | "little" ;
```

**Contextual words.** `big` and `little` are names everywhere except directly
after `as`, where each is a byte order when the next token can begin a type:
`(` or a name other than `as` and `with`. Otherwise it is the target type
itself, so a type declared with the name `big` is still converted to with
`x as big`, and `x as big as Int` converts to the type `big` and is then
ungrouped. The byte order is one of those two spellings, exactly.

**Array targets.** After a byte order the target is any type, including an
array type whose length is an integer or a size, `Word[8]^4` or
`Word[8]^(4 * n)`. Without a byte order the target is a type without a
length, as before: `x as Word[8]^4` still reads as `(x as Word[8]) ^ 4`, the
S3c `ORC0108`, whose note now adds "For an array type, name a byte order
first, as in `as big Word[32]^16`".

**Nesting and height.** A byte order is one token of its conversion: it
opens no nesting level, and a conversion's height is one more than the larger
of its operand's and its target's, as before.

The syntax tree gains, on a conversion, its byte order, if any, with its
span.

## 4. Types

**Words.** A value is **words** when its type is a word type, `Word[n]` for n
of 8, 16, 32, or 64, or an array of one, `Word[n]^k`. Its **width** is n · k
bits, n for a single word. The widest words are `Word[64]^256`, 16,384 bits.

A conversion `e as order T` is checked against an expected type `E`:

1. The target `T` is resolved. If it resolves and differs from `E`, that is
   `ORC0214` at the target, as for every conversion.
2. The operand's own type is found as for every conversion
   (`BINDINGS_2026.md` section 6, `ARRAYS_2026.md` section 5, and
   `BYTES_2026.md` sections 5 and 6): the type of its first typed leaf, where a join,
   a slice, and a byte string are leaves with lengths of their own. An array
   literal or a fill has the type of an array of its elements' type, taken
   from the first element with a typed leaf, and of its length, so
   `[high, low] as big Word[64]` reads two `Word[32]` names. An operand with
   no typed leaf, including an array literal none of whose elements has one,
   is `ORC0220` at the operand.
3. One side must be words and the other words of the same width, `Int`, or
   `Mod[m]`, as the table below says.
4. The operand is checked against its own type, by the earlier rules.

| Operand | Target | Admitted when |
| --- | --- | --- |
| words | words | both have the same width; otherwise `ORC0240` |
| words | `Int` or `Mod[m]` | always |
| `Int` or `Mod[m]` | words | always |
| `Int` or `Mod[m]` | `Int` or `Mod[m]` | never: `ORC0215` at the byte order |
| anything else | any type | never: `ORC0215` at `as` |
| any type | anything else | never: `ORC0215` at the target |

"Anything else" is `Bool`, a tuple, and an array of `Int`, `Mod[m]`, or
`Bool`. A conversion between words of the same type is admitted, and is the
identity in either order (section 5).

**Without a byte order.** A conversion without a byte order is unchanged: its
operand and target are single values, and an array operand or array target is
`ORC0215` at `as`. When the array is words, the note now names the byte order
that converts it: "name a byte order to read the words as one number or as
words of another width, as in `x as big Int`, or convert each element, such
as `x[0] as Int`" for an operand, and "name a byte order to write words as
`Word[32]^2`, as in `as big Word[32]^2`, or build the array from its
elements" for a target.

**Diagnostics.**

| Code | Form | Message | Label or note |
| --- | --- | --- | --- |
| `ORC0240` | words of different widths | "`Word[8]^3` and `Word[32]` have different widths", at the target | label "`Word[32]` has 32 bits"; at the operand "`Word[8]^3` has 24 bits"; note "a byte order keeps every bit of the words it converts, so words convert only to words of the same number of bits" |
| `ORC0215` | a number to a number | "`big` orders words, but this converts `Int` to `Mod[7]`", at the byte order | label "neither side is a word or an array of words"; note "a number converts to another without a byte order, as `x as Mod[7]`" |
| `ORC0215` | an operand that is not words or a number | "`as big` does not convert `Bool`", at `as`; a tuple is "a tuple" | label "a byte order packs and unpacks words"; note "`as big` and `as little` convert a word or an array of words to words of another width, to `Int`, or to `Mod[m]`, and back" |
| `ORC0215` | a target that is not words or a number | "`as little` does not convert to `Bool`", at the target | the same label and note |
| `ORC0220` | an operand with no typed leaf | "the operand of `as` has no type of its own" | as in S3c |
| `ORC0214` | a target other than the expected type | "this conversion gives `Word[32]`, but `Word[64]` is required here" | as in S3c |

`ORC0240` is new. Like every undefined operator, a rejected conversion stops
there: its operand is checked only far enough to find its type.

## 5. Meaning

Let the words on one side be x_0 through x_{k−1}, each of n bits, in the
order of the array, a single word being x_0 with k = 1. They **spell** the
number

- big: N = x_0 · 2^{n(k−1)} + x_1 · 2^{n(k−2)} + … + x_{k−1}, the first word
  most significant;
- little: N = x_0 + x_1 · 2^n + … + x_{k−1} · 2^{n(k−1)}, the first word
  least significant;

an integer from 0 through 2^{nk} − 1. A conversion in a byte order means:

- **words to words**: the words of the target's type, in the same order, that
  spell N. Since both have the same width, every bit is kept, and the
  conversion is invertible by its reverse.
- **words to `Int`**: N.
- **words to `Mod[m]`**: N modulo m, as `N as Mod[m]` would give.
- **`Int` to words**: the words, in the order, that spell the value's residue
  modulo 2^{nk}. A negative value is written as its two's complement, as
  `as Word[n]` already writes one word.
- **`Mod[m]` to words**: the words that spell the least residue's residue
  modulo 2^{nk}, which is the least residue itself whenever m ≤ 2^{nk}.

So `hex"01020304" as big Word[32]` is `0x01020304` and `as little` gives
`0x04030201`; `[0x01234567, 0x89abcdef] as big Word[64]` is
`0x0123456789abcdef`; `-1 as big Word[8]^4`, written through a binding, is
four bytes `0xff`; and `hex"0100" as big Mod[251]` is 256 mod 251, 5. A byte
order changes nothing between words of the same type, since both sides spell
the same number in the same order; reversing the bytes of a word takes one
conversion in each order, `(w as big Word[8]^4) as little Word[32]`.

## 6. Typed Reference Core

A conversion in a byte order is one `pack` node, which records the operand's
type and the byte order; its type is the target's, and its one child is the
operand. A conversion without a byte order is a `convert` node, as before.
Core gains one node kind. The Core is still internal and noncanonical, with
no encoding, digest, or proof role.

## 7. Evaluation

A `pack` node costs one step for each 64 bits of its width, or part of 64:
ceil(n · k / 64), from 1 for a single word through 256 for `Word[64]^256`.
A conversion to `Mod[m]` also costs what `as Mod[m]` costs for N
(`MODULAR_2026.md` section 9): d times the 32-bit digits of N, for d the
32-bit digits of m. The per-source budget of 1,048,576 steps is unchanged.

`orangec eval` displays the result as it displays any value of its type.

## 8. Resource limits and failure

The S3m budgets remain. S3n adds or refines the following.

- A byte order costs one semantic event, as a token of its conversion does,
  and adds no Core node beyond its `pack`.
- Every width is at most 16,384 bits, since an array has at most 256
  elements of at most 64 bits, which is the most an `Int` holds; so a
  conversion to `Int` never exceeds the significant-bit limit, and no length
  outside 1 through 256 is a type (`ORC0221`).
- The evaluator reserves the 32-bit digits of N, ceil(n · k / 32), and then
  the target's array, before writing either. A failure to reserve either is
  `ORC0301`, "reference evaluation result allocation failed", labeled "exact
  integer storage could not be reserved" or "evaluation array storage could
  not be reserved", and gives no partial output.

The deepest sources the limits admit, including conversions in a byte order
nested in each other's operands, and calls whose argument converts an array
literal holding the next call, each nested to half the nesting limit, must
parse, analyze, and evaluate within 1 MiB of native stack. A syntax tree whose
byte-order spans do not all belong to the source it is supplied with is
`ORC0210` for that source, and nothing is checked.

## 9. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged: the same source gives the same diagnostics, Core, and output
bytes. The S3n conformance runner
(`compiler/crates/orangec/tests/s3n_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3m runners.

### S3n conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3N-SYNTAX-01` | Section 3 | `big` and `little` directly after `as` and before `(` or a name other than `as` and `with` are a byte order, recorded with its span, and any type, arrays included, may follow one; anywhere else they are names, and an array type after `as` without a byte order is the specified `ORC0108`. | CLI and parser unit |
| `S3N-TYPE-01` | Section 4 | A conversion in a byte order has an operand typed by its first typed leaf, array literals and fills included, and admits words to words of the same width, words to `Int` or `Mod[m]`, and `Int` or `Mod[m]` to words; words of different widths are `ORC0240`, other pairs `ORC0215`, an untyped operand `ORC0220`, and a target other than the expected type `ORC0214`, with the specified spans and notes. | CLI and unit |
| `S3N-VALUE-01` | Section 5 | Words spell the number of their order, first word most significant for `big` and least significant for `little`, and convert to the words, number, or residue of that number; a number converts to the words of its residue modulo 2 to the power of their width. | CLI and unit |
| `S3N-CORE-01` | Section 6 | A conversion in a byte order is one `pack` node recording its operand's type and order, and a conversion without one is unchanged. | Unit and CLI observation |
| `S3N-EVAL-01` | Section 7 | A `pack` node evaluates as specified at one step per 64 bits of width, plus the residue cost for `Mod[m]`; SHA-256, SHA-512, ChaCha20, Poly1305, and X25519 reading and writing their words in a byte order match FIPS 180-4, RFC 8439, and RFC 7748. | CLI and unit |
| `S3N-RES-01` | Section 8 | Byte orders cost one semantic event and one node, widths reach exactly 16,384 bits, allocation failures give no partial output, nesting and stack use are bounded as specified, and foreign spans are `ORC0210`. | Generated CLI and unit |
| `S3N-COMPAT-01` | Section 11 | S3m sources keep their meaning, Core values, and output bytes, with only the specified diagnostic changes. | CLI and unit |
| `S3N-DETERMINISM-01` | Section 9 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 10. The encryption schemes

The schemes of `orangec enc` (`compiler/schemes/`) read their bytes as words
and write words as bytes in little-endian order with S3n: the ChaCha20 state,
the keystream, the exclusive-or of a chunk eight bytes at a time, each
Poly1305 block as an element of its field, the tag, and Ascon's blocks. Each
gives the same bytes as before and passes the same known answers; the steps
to seal one chunk fell from 43,805 through 55,096 to 24,152 through 28,325,
and a megabyte seals in about a third of the time.

## 11. Relationship to S3m

When OEP-0017 is accepted, this document replaces these clauses of
`SIZES_2026.md` and the documents it extends:

- the conversion production of `BINDINGS_2026.md` section 3, and the rule of
  `ARRAYS_2026.md` section 3 that a conversion's target never has a length,
  by section 3;
- the conversion typing of `BINDINGS_2026.md` section 6 and the rule of
  `ARRAYS_2026.md` section 5 that a conversion of an array is `ORC0215`, for
  conversions in a byte order, by section 4;
- the meaning of conversions, by section 5; and
- the Core node inventory and the cost table, by sections 6 and 7.

Every source that S3m accepts converts with `as` followed by a type without a
length, and a `big` or `little` there was a type's name only where section 3
still reads it as one, so S3n accepts it with the same Core values and the
same output bytes. A source that S3m rejects gets the same diagnostics, with
these exceptions:

- `big` or `little` after `as` and before `(` or a name other than `as` and
  `with` was a type's name, reported as an unknown type or followed by a
  syntax error; it is now a byte order;
- an array type after `as` without a byte order is still `ORC0108`, and its
  note gains the sentence "For an array type, name a byte order first, as in
  `as big Word[32]^16`";
- a conversion of an array of words, and a conversion to one, are still
  `ORC0215`, and their note, which was "convert each element, such as
  `x[0] as Int`", now names the byte order that converts them (section 4).

`orangec lex` is unchanged.

## 12. Explicit non-claims and future work

This slice defines no byte order of a single value: words are integers, and
only a sequence of words spells a number in an order. It defines no
conversion between arrays of different element types other than words, no
bit order within a word and no bit strings, no words narrower than 8 bits or
wider than 64, no reading of words at a position computed from data, no
conversion of a tuple, no mixed or middle-endian orders, and none of the
exclusions of `SIZES_2026.md` section 14 that this document does not lift.

A byte order is a statement about mathematical values, not about memory:
nothing here fixes a layout, an ABI, or a calling convention for a backend,
and the reference evaluator is not constant-time (`CONDITIONS_2026.md`
section 15).

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
