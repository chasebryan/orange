# Orange 2026 computed amounts specification

Status: proposed S3r semantics under OEP-0021, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3r of Orange 2026: **computed amounts**, which
let a word be shifted or rotated by an amount that is an expression, an
`Int` or a word, computed from data, `x <<< r` or `x >> (i % 8)`, with a
meaning at every amount. It is a delta over the proposed S3q rules in
[`TESTS_2026.md`](TESTS_2026.md), which are a delta over
[`LENGTHS_2026.md`](LENGTHS_2026.md) and the documents it extends.
Everything those documents define and this one does not mention is
unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0021](governance/oeps/OEP-0021-orange-2026-computed-amounts.md), which
requires OEP-0020. At that point it replaces the S3q clauses listed in
section 11. Until then, the compiler behavior it describes exists so that the
proposal can be reviewed against running code, and it establishes no accepted
language meaning. It accepts no D-004 candidate.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Cryptography turns words by amounts it computes. RC5 and RC6 rotate each
word to the left by an amount their data choose, the least significant lg w
bits of another word. The rho step of SHA-3 turns lane after lane by
(t + 1)(t + 2)/2 bits, t counting the steps of a walk over the state, and its
round constants place the bits of a shift register at positions 2^j − 1. A Montgomery ladder reads bit t of its scalar, a reflected CRC or
GHASH reads the bits of a byte in the opposite order, and the number-theoretic
transform of ML-KEM orders its constants by reversing the bits of an index.
Through S3q an amount was a literal from 0 through n − 1, so each of these was
written as a table or a chain of conditionals, beside the one line the
standard prints.

S3r writes that line. An amount may be any expression of type `Int` or a
word:

```orange
// RC6 key schedule: B = L[j] = (L[j] + A + B) <<< (A + B).
let b1: Word[32] = (l[k % 4] + a1 + b) <<< (a1 + b);

// FIPS 202 Algorithm 2, rho: turn lane (x, y) by (t + 1)(t + 2)/2.
b with [(x as Int) + 5 * (y as Int)] =
  a[(x as Int) + 5 * (y as Int)] <<< (((t + 1) * (t + 2)) / 2)

// FIPS 203 section 4.3, BitRev7: bit i of r moves to bit 6 - i.
for i in 0..7 with b: Word[8] = 0 { b | (((r >> i) & 1) << (6 - i)) }
```

Every amount has a meaning. A shift is multiplication or division by a power
of two, kept to the word, so a shift by the width or more gives 0 and a
negative amount shifts the other way; a rotation is periodic, and turns by
its amount modulo the width. Nothing is undefined, masked, or left to the
machine: `x << 32` on a 32-bit word computed from data is 0 on every target,
which is what the mathematics says and what C, Java, and x86 do not.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3q extends it, is
unchanged: a computed amount is checked from types alone, before anything
runs, every value of its type is an amount, and a shift or rotation has no
run-time failure. Every S3q source keeps its meaning (section 11).

## 3. Literal and computed amounts

S3r adds no token, reserved word, or grammar production. A shift or rotation
is still `prefixed shift_operator prefixed`, the amount is its right operand,
and a shift takes exactly two operands and shares no level with another
operator group, so `x << i + 1` is `ORC0108` and is written
`x << (i + 1)`.

An amount written as **one integer literal**, with or without a sign, is a
**literal amount**. It keeps the rule of S3b: it must be written without a
sign, and its value must be from 0 through n − 1 for a word of n bits.
Otherwise it is `ORC0216` at the amount (section 4). A literal names a fixed
bit position, and a literal past the width, or with a sign, is far more often
a slip than a wish for 0 or for the other direction.

Any other amount is a **computed amount**: a name, a call, an element, a
projection, a conditional, an operation, or a group. A group is an
expression, so `x << (32)` on a 32-bit word is a computed amount, whose value
is 32, and gives 0; `x << (-1)` shifts right by one.

## 4. Types

The left operand of a shift or rotation is unchanged: it is checked against
the expected type, which must be `Word[n]`, and a shift or rotation of
anything else is `ORC0215` at the operator.

A computed amount has a type of its own, independent of the word it shifts.
Its type is the type of its **first typed leaf**, found as for an index
(`LOOKUPS_2026.md` section 4), when that type is a word type; otherwise it is
`Int`. The amount is then checked against that type by the earlier rules. So:

- `x <<< r` with `r: Word[8]` has a `Word[8]` amount, and a `Word[64]` may be
  turned by it;
- `x >> (i % 8)` with a loop index `i`, `x << (8 - n)` with a size `n`, and
  `x <<< (((t + 1) * (t + 2)) / 2)` have `Int` amounts;
- `rc << places[t % 7]` with `places: Word[8]^7` has a `Word[8]` amount;
- `x << (a1 + b)` with `a1, b: Word[32]` has a `Word[32]` amount, whose sum
  wraps modulo 2^32 as every word sum does.

An amount of any other type, `Bool`, `Mod[m]`, an array, or a tuple, has no
word as its first typed leaf, and so is checked against `Int` and reported as
that check reports it: "`b` has type `Bool`, but `Int` is required here",
`ORC0214`. A residue is an amount after `as Int`.

**Diagnostics.**

| Code | Form | Message | Label or note |
| --- | --- | --- | --- |
| `ORC0216` | a literal amount with a sign, or not below n | "`<<` on `Word[32]` needs an amount from 0 through 31", at the amount | label "a literal amount is from 0 through 31"; note "an amount written as one integer literal is from 0 through n - 1; any other amount is computed, an `Int` or a word, such as `x <<< r` or `x >> (i % 8)`" |
| `ORC0214` | a computed amount that is not of its type | as for every expression, such as "`b` has type `Bool`, but `Int` is required here" | as for every expression |
| `ORC0215` | a shift or rotation of a value that is not a word | "`<<` is not defined for `Int`", at the operator | as in S3b; the amount is not checked |

No diagnostic code is added. A shift whose left operand is not a word stops
there, as every undefined operator does, and its amount is not checked.

## 5. Meaning

Let a be a word of n bits, m = 2^n, and k the amount's value: an `Int`'s
integer, or a word's value from 0 through 2^j − 1 for a word of j bits.

| Expression | Value |
| --- | --- |
| `a << k` | the residue of floor(a · 2^k) modulo m |
| `a >> k` | the residue of floor(a · 2^−k) modulo m |
| `a <<< k` | `(a << r) \| (a >> (n − r))` for r = k mod n, from 0 through n − 1, and `a` when r = 0 |
| `a >>> k` | `a <<< (−k)`: `(a >> r) \| (a << (n − r))` for r = k mod n, and `a` when r = 0 |

So:

- for k from 0 through n − 1, each is the value S3b's table gives for the
  literal k, and a computed amount equal to a literal gives the same word;
- a shift by n or more, in either direction, gives 0, whatever the size of
  k: `0x96 << 8`, computed, is `0x00`, and so is `0x96 >> (2^200)`;
- a negative amount shifts the other way: `a << −1` is `a >> 1`, and
  `a >> −1` is `a << 1`;
- a rotation turns by k modulo n, so `a <<< 11` is `a <<< 3` on a byte, and
  `a <<< −1` is `a >>> 1`. RC6's "least significant lg w bits of b" is
  exactly b mod w, and FIPS 202 turns a lane by (t + 1)(t + 2)/2 modulo the
  lane size w.

The meaning is the mathematics of section 9 of `EXPRESSIONS_2026.md`
extended to every integer, not the behavior of a machine. C leaves a shift by
the width or more undefined, and Java and x86 reduce a 32-bit shift's amount
modulo 32, so there `x << 32` is `x`. In Orange it is 0, and a backend must
compute that value: a shift instruction that reduces its amount is correct
only for amounts it proves are below the width, and otherwise needs a
comparison and a choice as well.

## 6. Ranges of indices

A shift or rotation by a computed amount ranges over its whole type as an
index, as `LOOKUPS_2026.md` section 5 already states for "a shift by an
amount that is not a literal": its value depends on the amount, which the
range analysis does not follow. So `t[x >> k]` with a byte x and a table of
16 entries is `ORC0223`, "this index runs from 0 through 255, out of range
for `Word[8]^16`", and `t[(x >> k) & 15]` is admitted, since `&` bounds it.

## 7. Typed Reference Core

A shift or rotation by a computed amount is one `shift-by` node, which
records its operator and its amount's type, `Int` or a word type; its type is
the shifted word's, and its two children are the word and the amount, in
that order. A literal amount is still part of its `shift` node, as in S3b.
The Core grammar of `EXPRESSIONS_2026.md` section 11 gains:

```text
node_kind = ...
          | shift_by (shl | shr | rotl | rotr) amount_type ;
```

The Core is still internal and noncanonical, with no encoding, digest, or
proof role.

## 8. Evaluation

A `shift-by` node costs **one step**, whatever the amount's size or sign, as
a `shift` node does; its operands cost what they cost. The evaluator needs of
an `Int` amount only its sign, whether its magnitude has more than 64 bits,
and its low 64 bits: a shift by a magnitude of 64 or more gives 0 at every
width, and since every width divides 2^64, a rotation's k mod n follows from
the low bits and the sign. So `x <<< k` with k of 16,384 bits costs the same
step, and reads the same parts of k, as with k = 3.

`orangec eval` and `orangec test` display the result as they display any
word.

## 9. Resource limits and failure

The S3q budgets remain. S3r adds or refines the following.

- A computed amount costs the semantic events and Core nodes of its
  expression and no more; a literal amount keeps its literal event.
- A `shift-by` node allocates nothing when it runs.
- A `shift-by` node whose type is not a word, whose amount type is neither
  `Int` nor a word, whose operator is not a shift or rotation, or whose
  children do not have the types it records is inconsistent Core, which the
  evaluator refuses with `ORC0301`, "reference evaluation received
  inconsistent Core", giving no value.

## 10. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged: the same source gives the same diagnostics, Core, and output
bytes. The S3r conformance runner
(`compiler/crates/orangec/tests/s3r_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3q runners.

### S3r conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3R-FORM-01` | Section 3 | An amount written as one integer literal is a literal amount, admitted only unsigned and below the width and otherwise `ORC0216` at the amount with the specified label and note; any other amount, a group included, is computed; an ungrouped operator in an amount is `ORC0108`. | CLI, generated CLI, and unit |
| `S3R-TYPE-01` | Section 4 | A computed amount has the type of its first typed leaf when that is a word and `Int` otherwise, is checked as that type, and may differ in width from the word it shifts; an amount of another type is reported by that check, and a shift of a value that is not a word stays `ORC0215`. | CLI and unit |
| `S3R-SHIFT-01` | Section 5 | `a << k` and `a >> k` are floor(a · 2^k) and floor(a · 2^−k) modulo 2^n at every width, for `Int` amounts of every size and sign and word amounts of every width. | Generated CLI and unit |
| `S3R-ROTATE-01` | Section 5 | `a <<< k` turns a left by k mod n and `a >>> k` turns it right, at every width, for `Int` amounts of every size and sign and word amounts of every width; RC6 and SHA3-256 reproduce their published values. | Generated CLI and unit |
| `S3R-RANGE-01` | Section 6 | A shift by a computed amount ranges over its whole type as an index. | CLI and unit |
| `S3R-CORE-01` | Section 7 | A shift or rotation by a computed amount is one `shift-by` node recording its operator and amount type, and a literal amount keeps its `shift` node. | Unit |
| `S3R-COST-01` | Section 8 | A `shift-by` node costs one step whatever its amount's size, sign, or type. | Generated CLI and unit |
| `S3R-RES-01` | Section 9 | Amounts of up to 16,384 bits are read in bounded work, a `shift-by` node allocates nothing, and inconsistent `shift-by` Core is refused with no value. | Generated CLI and unit |
| `S3R-COMPAT-01` | Section 11 | S3q sources keep their meaning, Core values, output bytes, and steps, and a computed amount equal to a literal gives the literal's value; diagnostics change only as specified. | CLI and unit |
| `S3R-DETERMINISM-01` | Section 10 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 11. Relationship to S3q

When OEP-0021 is accepted, this document replaces these clauses of
`TESTS_2026.md` and the documents it extends:

- the rule of `EXPRESSIONS_2026.md` section 8 that the amount of a shift or
  rotation is not an expression but an unsigned integer literal from 0
  through n − 1, by sections 3 and 4;
- the shift and rotation rows of the word table of `EXPRESSIONS_2026.md`
  section 9, which S3r extends to every amount, by section 5;
- the Core node inventory of `EXPRESSIONS_2026.md` section 11, by section 7;
  and
- the step table of `EXPRESSIONS_2026.md` section 14, as later slices extend
  it, which S3r extends with the `shift-by` node, by section 8.

Every source that S3q accepts writes each amount as one unsigned integer
literal below the width, so S3r accepts it with the same Core values, the
same output bytes, and the same steps. A source that S3q rejects gets the
same diagnostics, with these exceptions:

- an amount that is not one integer literal was `ORC0216`, "`<<` on
  `Word[32]` needs an amount from 0 through 31", labeled "amount must be an
  unsigned integer literal", with the note "amounts are fixed literals;
  variable amounts are not part of Orange 2026", and the amount was not
  checked; it is now a computed amount, checked as section 4 says, and
  admitted when that check passes;
- a literal amount with a sign or not below the width is still `ORC0216` with
  the same message, and its label and note are those of section 4.

`orangec` gains no command or option. `orangec lex` is unchanged.

## 12. Leakage, non-claims, and future work

**Leakage.** A shift or rotation by a secret amount is a timing concern, as a
lookup at a secret index is (`LOOKUPS_2026.md` section 11). On a processor
with a barrel shifter a shift takes the same time at every amount, but on
processors without one, and on some microcontrollers and smart cards, it
takes time that grows with the amount, and the data-dependent rotations of
RC5 and RC6 were an early target of timing analysis for that reason. The
reference evaluator is not constant-time, and its step count does not depend
on the amount, which is not a timing claim. A backend must decide what a
shift by a possibly secret amount becomes: for example, a rotation built from
log2 n conditional rotations by 1, 2, 4, and so on, each done whether or not
its bit is set, or a refusal under a constant-time profile. Until then
Orange makes no timing, secrecy, or leakage claim about computed amounts.

**Non-claims.** This slice defines no shift of `Int`, `Mod[m]`, an array, or
a tuple; no arithmetic shift that copies the sign bit; no amount of type
`Mod[m]` without `as Int`; no shift across the words of an array, funnel
shift, bit-field extraction, or word narrower than 8 bits or wider than 64;
no change to the literal amount's rule; and none of the exclusions of
`TESTS_2026.md` section 12 that this document does not lift.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
