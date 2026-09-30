# Orange 2026 bytes specification

Status: proposed S3l semantics under OEP-0015, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3l of Orange 2026: **byte strings**, which
write an array of bytes as the text or the hex digits a standard prints;
**concatenation** `++`, which joins two arrays; and **slices** `x[a..b]` and
**slice updates** `x with [a..b] = v`, which read and replace a run of
consecutive elements whose bounds are proved in range before a program runs.
It is a delta over the proposed S3k rules in
[`TUPLES_2026.md`](TUPLES_2026.md), which are a delta over
[`BLOCKS_2026.md`](BLOCKS_2026.md) and the documents it extends. Everything
those documents define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0015](governance/oeps/OEP-0015-orange-2026-bytes.md), which requires
OEP-0014. At that point it replaces the S3k clauses listed in section 13.
Until then, the compiler behavior it describes exists so that the proposal can
be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`SIZES_2026.md`](SIZES_2026.md), proposed under OEP-0016, extends this
> document with size parameters: a size parameter's name counts as an integer
> literal in a slice's bounds, as `x[n - 2..]`, and a byte string, a join, or a
> slice may stand where a sized function's parameter of that length is required.
> [`ORDER_2026.md`](ORDER_2026.md), proposed under OEP-0017, lifts this
> document's exclusion of conversions between bytes and words: `hex"01020304" as
> big Word[32]` is `0x01020304`.
> [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md), proposed under
> OEP-0018, lists `Word[8]^n` among a function's types, so that a call may
> name it, as `first[Word[8]^4](hex"00010203")`. Every source this document
> accepts keeps its meaning under all three.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Standards print their inputs as bytes. RFC 4231 keys HMAC with the text "Jefe"
and the bytes `0b 0b ... 0b`; FIPS 180-4 section 5.1.1 pads a message by
appending the byte 80, zeros, and its length; RFC 8439 section 2.8.2 prints a
plaintext as a sentence and its key, nonce, and additional data in hex, takes
the Poly1305 key as the first 32 bytes of a ChaCha20 block, and feeds
Poly1305 sixteen bytes at a time. Through S3k an Orange program wrote each of
these as an array of numbers, one element at a time, and moved runs of bytes
with a loop that updated one element per step.

S3l writes them as the standards do. A **byte string** `"..."` is the array of
the ASCII bytes of its text, and a **hex string** `hex"..."` the array of its
hex digit pairs; `++` joins two arrays; `x[a..b]` is the run of elements of `x`
from index `a` up to, but not including, index `b`; and `x with [a..b] = v`
replaces that run:

```orange
// FIPS 180-4 section 5.1.1: "abc" is 24 bits, followed by the byte 80, 52
// zero bytes and the 64-bit length 0x18.
spec abc() -> Word[8]^32 { hash64("abc" ++ hex"80" ++ [0; 52] ++ hex"00000000 00000018") }

// RFC 8439 section 2.8: the one-time key is the first 32 bytes of block 0.
ciphertext ++ mac(block(key, 0, nonce)[..32], mac_data(aad, ciphertext))

// Section 2.5.1: ten sixteen-byte blocks, one per step.
for j in 0..10 with a: P = 0 { (a + block_number(m[16 * j..16 * j + 16])) * r }
```

A slice's bounds are integer literals and loop indices combined by `+`, `-`,
and `*` by a constant, so its length is the same at every step and every
element it takes is proved to exist before the program runs. A slice's
position never depends on data.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3k extends it, is
unchanged. Every S3k source keeps its meaning (section 13).

## 3. Lexical grammar

S3l adds two tokens and gives string tokens a grammatical role.

**Hex strings.** An identifier spelled exactly `hex` and followed directly by
`"`, with nothing between them, begins a `HEX_STRING` token, which ends at the
next `"` on the same logical line. The token spans `hex` through the closing
quote. Its contents are hex digits, of either case, and spaces (U+0020): each
byte is two adjacent hex digits, and spaces may stand before, between, and
after bytes but not between the two digits of one byte. A hex string has no
escapes. The first character that breaks these rules is a lexical error
(`ORC0009`, new) at that character, with the note "a hex string holds bytes
written as pairs of hex digits, as in `hex"00 1f a0"`; spaces may separate
bytes but not split one":

| Offense | Message | Label |
| --- | --- | --- |
| a character other than a hex digit or a space, such as `g`, `x`, `:`, or a tab | "'g' cannot appear in a hex string" (a character that is not printable ASCII is named by its code point, as "U+0009 cannot appear in a hex string") | "not a hex digit or a space" |
| `\` | "'\\' cannot appear in a hex string" | "a hex string has no escapes" |
| a digit followed by a space or the closing quote before its partner | "hex digit '2' has no partner" | "a byte is written as two hex digits" |

At most one offense is reported for one hex string. A hex string that reaches
a line ending or the end of the source is `ORC0003`, "unterminated hex
string", at its opening `hex"` alone, labeled "this hex string is never
closed", with the note "pre-alpha Orange strings cannot cross a line
boundary"; nothing inside it is reported. `hex` anywhere else, including
`hex "00"` with a space and `hexa"00"`, is an identifier, and `hex` is not
reserved: a function or binding may still be named `hex`.

**Concatenation.** `++` is a punctuation token, `PLUS_PLUS`. Punctuation is
lexed by longest match, so `+++` is `++` followed by `+`, and `+ +` is still
two `+` tokens.

**Strings.** The `STRING` token of `LANGUAGE_2026.md` section 2.4 is
unchanged: it ends on its logical line, and its escapes are `\"`, `\\`, `\n`,
`\r`, `\t`, `\0`, and `\xNN`. It is now a byte string where an expression may
stand (section 4), and its contents are given meaning by section 5.

## 4. Grammar

The primary, suffix, and update productions of the earlier documents become:

```text
primary      = IDENTIFIER suffix? | call suffix? | "(" expression ")"
             | byte_string | tuple | array | fill | loop | conditional | ... ;
byte_string  = STRING | HEX_STRING ;
suffix       = projection ( index | slice )? | index | slice ;
slice        = "[" range "]" ;
range        = expression ".." expression? | ".." expression ;
update       = operand "with" "[" ( expression | range ) "]" "=" expression ;
```

`++` is a binary operator of a group of its own:

| Group | Operators | Shape |
| --- | --- | --- |
| Concatenation | `++` | chains of `++` associate to the left |

The grouping rule of `EXPRESSIONS_2026.md` section 5 is unchanged: an
operator of another group, a comparison, `as`, or `with` after `++` at the
same level, or `++` after one of them, is `ORC0108`, as "`+` follows `++`
without grouping parentheses". `a ++ b ++ c` means `(a ++ b) ++ c`.

A **slice** follows a name, a call, or a projection, as an index does. Its
start or its end may be omitted, but not both: `x[a..]` runs to the end of
`x`, and `x[..b]` from its start. A slice is followed by no index, slice, or
projection, and a byte string by no index or slice; bind either with `let` to
select from it. A **slice update** writes a range where an update writes an
index; its value extends as far as an expression can, as an update's does.

The parse errors are `ORC0101`, each with a note that shows the form:

| Form | Message | Note |
| --- | --- | --- |
| `x[..]` | "expected a bound of the slice after `..`" | "a slice is written `x[a..b]`, the elements of `x` from index a up to but not including index b, or `x[a..]` or `x[..b]` to run to the end or from the start" |
| `x[0..4..2]` | "expected `]` after the slice" | the same |
| `x[1..3][0]`, `x[1..3][0..1]` | "expected an operator or the end of the expression", at the second `[` | "a slice is taken once, from a name, a call, or a tuple's element; bind it with `let` to select from it" |
| `x[1..3].0` | the same, at `.` | "a slice is an array, not a tuple, so it has no `.k`" |
| `"abc"[0]`, `hex"00 01"[0..1]` | the same, at `[` | "a byte string is not indexed or sliced where it is written; bind it with `let` to select from it" |
| `x with [..] = v` | "expected a bound of the slice after `..`" | "a slice update is written `x with [a..b] = values`, the array `x` with its elements from index a up to but not including index b replaced" |
| `x with [0..2 = v`, `x with [0..2] v` | "expected `]` after the slice" or "expected `=` after the updated slice" | the same |
| `hex "00"` | the message of whatever was expected at the string, such as "expected `}` after the body expression" | "a hex string's quote follows `hex` directly, with no space, as in `hex"00 1f a0"`" |

**Nesting and height.** A byte string is a leaf of height 1. `++` is a binary
operator: its height is one more than its taller operand's, so a chain of
joins has at most 255 operands. A slice's brackets open one nesting level of
the 64-level budget, shared by both bounds, and its height is one more than
the tallest of its base and its bounds. A slice update opens one level for its
bounds and value, as an update does for its index and value, and its height
is one more than the tallest of its base, bounds, and value.

The syntax tree gains a byte string, recording whether it is a hex string; the
`++` operator; a slice with its base and range; a slice update with its base,
the span of `with`, its range, and its value; and a range with its written
bounds, the span of `..`, and its own span, from the first bound, or `..`,
through the last bound, or `..`.

## 5. Byte strings

**Bytes.** A byte string's bytes are decoded from its spelling. In `"..."`,
each character from U+0020 through U+007E other than `"` and `\` is its ASCII
byte, and each escape is one byte: `\"` is 22, `\\` is 5c, `\n` is 0a, `\r`
is 0d, `\t` is 09, `\0` is 00, and `\xNN` is the byte NN, in either case. In
`hex"..."`, each pair of digits is one byte, the first digit high; spaces are
not bytes.

Decoding reads the string from its start and stops at the first of these:

- a character that is not printable ASCII, such as a tab, U+007F, or any
  character outside ASCII, is `ORC0235` (new), "U+00E9 is not a printable
  ASCII character", at that character, labeled with the character's bytes as
  a hex string ("its byte is written `hex"09"`", or for a character outside
  ASCII "its UTF-8 bytes are written `hex"c3 a9"`"), with the note "a byte
  string's characters are its bytes, so each is printable ASCII, from ` `
  through `~`; write any other byte as an escape, or in a hex string joined
  with `++`"; and
- a 257th byte is `ORC0221`, "a byte string holds at most 256 bytes", at the
  string, labeled "this string holds more than 256", and no later character
  is read.

A byte string with no bytes, such as `""`, `hex""`, or `hex"  "`, is
`ORC0221`, "a byte string holds at least one byte", labeled "this string is
empty". Both `ORC0221` diagnostics have the note "a byte string is an array
`Word[8]^n` of 1 through 256 bytes; join longer runs with `++`".

**Type.** A byte string of n bytes has type `Word[8]^n`. It is checked
against the type required where it stands, after it is decoded:

- an array of `Word[8]` of another length is `ORC0222`, "this byte string
  holds 3 bytes, but `Word[8]^4` has 4", labeled "expected 4 bytes"; and
- any other type is `ORC0214`, "this byte string has type `Word[8]^3`, but
  `Word[32]^3` is required here", labeled "expected `Word[32]^3`".

Both have the note "a byte string is the array `Word[8]^n` of its n bytes".
A byte string whose decoding fails is not checked against its type.

## 6. Joins, slices, and slice updates

**Found lengths.** The typing below uses the length of an array expression
**found without reporting**: an array literal's element count; a fill's
length; a byte string's number of bytes, when it decodes; for `a ++ b`, the
sum of the found lengths of both operands; for a group, its expression's; for
an update or a slice update, its base's; for a conditional, the length of its
first typed leaf's type, or else that of its first branch that binds no names
and has a found length; and for anything else, the length of the array type
of its first typed leaf, when it has one.

**Joins.** `a ++ b` is checked against the required type E as follows.

1. E must be an array type `T^n`. Any other type is `ORC0214`, "`++` joins
   arrays, but `Word[32]` is required here", at `++`, labeled "`Word[32]` is
   not an array type", and neither operand is checked.
2. If `a` has no found length and its first typed leaf has a type that is not
   an array, that is `ORC0224`, "only arrays can be joined, but this has type
   `Word[32]`", at `a`, labeled "`Word[32]` is not an array", and neither
   operand is checked. When the length of `a` is found, the same holds for
   `b`.
3. If both lengths l and r are found and l + r is not n, that is `ORC0222`,
   "`++` joins 3 and 2 elements, 5 in all, but `Word[8]^8` has 8", at `++`,
   labeled "expected 8 elements in all". If only l is found and l is at
   least n, that is `ORC0222`, "the left operand of `++` has 8 elements,
   leaving none of the 8 of `Word[8]^8` for the right", at `++`.
4. Otherwise `a` is checked against `T^l` and then `b` against `T^(n - l)`,
   so that an operand of another element type is reported inside it, as in
   "`x` has type `Word[32]^2`, but `Word[8]^2` is required here".
5. If l is not found, `a` is checked against all of E; if that succeeds, it
   leaves nothing for `b`, which is the second `ORC0222` of step 3, and `b` is
   not checked.

Every diagnostic of the joins has the note "`a ++ b` is the array of the
elements of a followed by those of b, of one element type". In a chain, each
join is validated by steps 1 through 3 before its left operand is checked,
the leftmost operand is checked first, and the right operands follow from
left to right, so a chain's diagnostics appear in source order. A rejected
join has no value, and the joins around it report nothing further about it.
An operand whose length is not found, such as a conditional every branch of
which binds names, may stand last in a chain, where it takes what the others
leave, or be bound with `let` first.

**Slices.** `x[a..b]` is checked against the required type E as follows.

1. The base's type is found without reporting, as an indexed array's is
   (`ARRAYS_2026.md` section 5). If it has none, the base is checked against E
   so that the reason is reported, and the slice has no value.
2. A base whose type is not an array is `ORC0224`, "only an array can be
   sliced, but this has type `(Int, Int)`", at the base, labeled "`(Int,
   Int)` is a tuple, not an array" or, for another type, "`Word[32]` has no
   elements", with the note "a slice `x[a..b]` is taken from a value of type
   `T^n`". The base is then checked against its own type, and the slice has
   no value.
3. E must be an array of the base's element type. An array of another
   element type is `ORC0214`, "this slice is an array of `Word[32]`, but
   `Word[8]^4` is required here", with the note "a slice is an array of the
   elements of the array it is taken from"; a type that is not an array is
   `ORC0214`, "a slice is an array, but `Word[8]` is required here", with the
   note "one element is selected by an index, such as `x[0]`". Both are at
   the slice and labeled "expected" and the type; the base and bounds are
   still checked.
4. The base is checked against its type, and the bounds are checked as `Int`
   expressions, the start before the end, so that "`w` has type `Word[8]`, but
   `Int` is required here" is reported in a bound. The bounds must then be
   static (section 7), which gives the slice's length L.
5. If L is not the length of E, that is `ORC0222`, "this slice has 4
   elements, but `Word[8]^3` has 3", at the slice, labeled "expected 3
   elements".

**Slice updates.** `x with [a..b] = v` is checked against the required type E
as follows.

1. If the base's first typed leaf has a type that is not an array, that is
   `ORC0224`, "only an array can be updated, but this has type `Word[32]`", at
   the base, labeled as in step 2 of slices, with the note "a tuple with
   elements replaced is written anew, such as `(v, p.1)`" for a tuple; the
   base is checked against its own type, and the update has no value.
2. E must be an array type. Any other type is `ORC0214`, "an update gives an
   array, but `Word[8]` is required here", at the update.
3. The base is checked against E, the bounds are checked as for a slice of an
   array of E's type, giving L, and `v` is checked against the array of E's
   element type and length L, as in "this byte string holds 3 bytes, but
   `Word[8]^2` has 2".

The notes of a slice update's `ORC0214` and non-tuple `ORC0224` are "`x with
[a..b] = v` is the array `x` with its elements from index a up to b replaced
by those of v".

**Typed leaves.** A byte string, a join, and a slice are typed leaves
(`BINDINGS_2026.md` section 6). A byte string's type is `Word[8]^n`; a join's
type is the array of its operands' found lengths, when both are found, and of
the element type of its first operand that has one; and a slice's type is the
array of its base's element type and its length, when its bounds are static.
A slice update is not a typed leaf: its first typed leaf is its base's, as
for an update. So `"ab" == x` is `ORC0215`, "`==` is not defined for
`Word[8]^2`", at `==`, and `"ab" as Word[8]` is `ORC0215`, "`as` is not
defined for `Word[8]^2`", as for any array.

## 7. Static bounds

A slice's bounds are **static**: each is built only from integer literals,
the indices of the loops that enclose it, parentheses, prefix `-`, and the
operators `+`, `-`, and `*`, where at least one operand of each `*` uses no
loop index. Such a bound is a constant plus a sum of loop indices, each with a
constant coefficient. A name of a parameter, binding, or accumulator, a call,
a projection, `/`, `%`, a conversion, and a conditional are not parts of a
static bound, even where their values are constant. An omitted start is 0,
and an omitted end is the length n of the array sliced.

- A bound with any other part is `ORC0226`, "a slice's bounds may use only
  integer literals and loop indices", at the first such part, labeled "this
  is neither".
- A product both of whose operands use a loop index is `ORC0226`, "a slice's
  bound may multiply a loop index only by a constant", at its `*`, labeled
  "both operands of this `*` use a loop index".
- A bound one of whose constants or coefficients has more significant bits
  than the limit of `Int` (16,384, `SEMANTICS_2026.md` section 9), or one of
  whose sums, differences, or products has a value with more at some step, is
  `ORC0223`, "a part of this bound exceeds the 16384-significant-bit limit of
  `Int`", at the bound, labeled "this bound has no representable range". The
  evaluator computes every part of a bound, so a part too large is an error
  even where it cancels, as `(a * a - a * a) + i` does for a literal `a` of
  more than 8,192 significant bits. A part that is not static is reported
  before a part that is too large.

Each of these has the note "a slice's position never depends on data: its
bounds are built from integer literals and loop indices with `+`, `-`, and `*`
by a constant".

**Length.** The slice's length L is its end minus its start. L must be the
same at every step, so the loop indices must cancel, and L must be at
least 1. Otherwise that is `ORC0236` (new), at the range, with the note "a slice
`x[a..b]` holds the b - a elements from index a, and b - a must be the same
positive number at every step, as in `x[16 * i..16 * i + 16]`":

| Difference | Message | Label |
| --- | --- | --- |
| depends on a loop index | "the length of this slice changes from step to step" | "its bounds must differ by the same number at every step" |
| 0 | "this slice is empty: its bounds are equal" | "a slice holds at least one element" |
| negative | "this slice ends 3 elements before it starts" | the same |
| beyond the range of a 64-bit signed integer | "this slice's bounds are too far apart" | the same |

**Range.** Over every value of every loop index in the bounds, within that
loop's range, the start must be at least 0 and the end at most n. The least
start and the greatest end are computed exactly from the bounds'
coefficients. Otherwise that is `ORC0223`, "this slice reaches elements 6
through 9, out of range for `Word[8]^8`", naming the least start and the
greatest end minus one, at the range, labeled "indices run from 0 through 7",
with the note "every element a slice can take, over every loop index in its
bounds, must be an element of the array".

The checks run in the order of this section: a bound's form, the length, and
the range, and the first that fails is the one reported. Because every bound
is static, no slice needs a run-time bounds check and none depends on data. A
slice at a position computed from data is not part of this slice
(section 14).

## 8. Meaning

A byte string is the array of its bytes, in order, as `Word[8]` values. `a ++
b` is the array of the elements of `a` followed by those of `b`. `x[a..b]` is
the array of elements a, a + 1, ..., b - 1 of `x`. `x with [a..b] = v` is `x`
with elements a through b - 1 replaced by the elements of `v`, in order; `x`
itself is a value and is not changed.

Evaluation is left to right: a join evaluates its left operand and then its
right; a slice its base, its start, and its end; a slice update its base, its
start, its end, and its value. `orangec eval` displays an array of bytes as
any array, each byte as two hex digits:

```text
bytes::greeting: Word[8]^8 = [0x48, 0x69, 0x20, 0x54, 0x68, 0x65, 0x72, 0x65]
bytes::iv: Word[8]^8 = [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47]
```

## 9. Typed Reference Core

The Core of the earlier documents gains three node kinds, and a literal node
may hold an array:

```text
node_kind  = ...
           | concat
           | slice
           | slice_update ;
```

- A byte string is one `literal` node whose value is the array of its bytes,
  of type `Word[8]^n`.
- A `concat` node consumes two preceding array subtrees, the left and then the
  right, whose element types are its own and whose lengths sum to its length.
- A `slice` node consumes an array subtree, an `Int` start subtree, and an
  `Int` end subtree, and has the array type of the base's element type and
  length L. Analysis proved that the start is at least 0, that the end is the
  start plus L, and that the end is at most the base's length.
- A `slice_update` node consumes an array subtree of its own type, an `Int`
  start subtree, an `Int` end subtree, and a value subtree of the same element
  type and of length end minus start, and has the base's type.
- An omitted bound is an `Int` literal node, 0 or the base's length, spanning
  `..`.

The Core is still internal and noncanonical, with no encoding, digest, or
proof role, and a function without the S3l forms has exactly its S3k Core.

## 10. Evaluation

| Operation | Steps |
| --- | --- |
| a byte string | 1 |
| `a ++ b` of n elements | the steps of its operands, and ⌈n/64⌉ |
| `x[a..b]` of L elements | the steps of its base and bounds, and ⌈L/64⌉ |
| an omitted bound | 1 |
| `x with [a..b] = v`, `x` of n elements | the steps of its base, bounds, and value, and ⌈n/64⌉ |

As for an update or a fill (`LOOKUPS_2026.md`), a step of a join, a slice, or
a slice update stands for up to 64 elements copied. The evaluator builds each
byte string's array once, before evaluation, and shares it wherever the
literal is evaluated, as it shares integer literals. The per-source budget of
1,048,576 steps is unchanged.

Inconsistent Core stops evaluation with no values: a `concat` node whose type
is not an array, or whose operands are not arrays of its element type whose
lengths sum to its length; a `slice` node whose type is not an array, whose
base is not an array of its element type, whose bounds are not `Int` values,
whose start is negative, whose end is not its start plus its length, or whose
run lies outside its base; a `slice_update` node whose base has another type
than its own, whose value is not an array of its element type, or whose
bounds, value length, and base disagree in the same ways; and too few
subtrees before any of them.

## 11. Resource limits and failure

The S3k budgets remain. S3l adds or refines the following.

- A byte string holds 1 through 256 bytes (section 5). Decoding reserves
  storage for 256 bytes and stops at the 257th, so a long spelling is never
  read to its end. A join, a slice, and a slice update are arrays of at most
  256 elements, as every array is.
- Nesting and height are as in section 4.
- A byte string of n bytes costs one semantic event and one more per byte,
  and counts one Core node. `++` costs one event and one node. A slice costs
  one event, the events of its base and written bounds, and one node for
  itself and one for each omitted bound. A slice update costs one event, the
  events of its base, written bounds, and value, and one node for itself and
  one for each omitted bound. As always, each Core node also counts one
  semantic event. With a parameter `x: Word[8]^4`, the body `x` takes 15
  events and 5 nodes, `"abc"` 18 and 5, `x ++ x` 19 and 7, `x[1..3]` 25 and
  8, `x[..2]` 22 and 8, and `x with [1..3] = x[..2]` 34 and 12.
- An allocation failure while decoding a byte string or building its value is
  `ORC0209`, "byte string storage allocation failed", at the string; while
  reading a slice's bounds, `ORC0209`, "slice bound storage allocation
  failed", at the range or the bound. While evaluating, it is `ORC0301`,
  "reference evaluation result allocation failed", labeled "evaluation array
  storage could not be reserved" for a join, a slice, or a slice update, and
  "evaluated exact integer storage could not be reserved" when the shared
  literals, byte strings included, cannot be built. None gives partial output.

Analysis checks a chain of joins in one frame, however long. The deepest sources the limits admit, including a join of
255 operands, a slice whose bound is nested in 62 groups, 31 slice updates
nested in the values of calls, and 63 slices of calls nested in calls, must
parse, analyze, and evaluate within 1 MiB of native stack, and a join of 255
operands with errors in it must be rejected within 1 MiB. A syntax tree whose
spans, including those of every byte string, join, slice, range, bound, `..`,
`with`, and value, do not all belong to the source it is supplied with is
`ORC0210` for that source, and nothing is checked.

## 12. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3l conformance runner
(`compiler/crates/orangec/tests/s3l_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3k runners.

### S3l conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3L-LEX-01` | Section 3 | `hex"..."` is one `HEX_STRING` token of hex digit pairs and spaces and `++` one `PLUS_PLUS` token, by longest match; a malformed hex string is `ORC0009` at its first offense and an unterminated one `ORC0003` at its opening. | CLI and unit |
| `S3L-SYNTAX-01` | Section 4 | Byte strings, joins, slices, and slice updates parse with exact spans, levels, and heights; `++` is a group of its own (`ORC0108`), and malformed forms are `ORC0101` with the specified messages and notes. | CLI and parser unit |
| `S3L-BYTES-01` | Section 5 | A byte string's bytes are its printable ASCII characters and escapes or its hex digit pairs, 1 through 256 of them (`ORC0235`, `ORC0221`), and it has type `Word[8]^n` (`ORC0214`, `ORC0222`). | CLI and unit |
| `S3L-TYPE-01` | Section 6 | Joins, slices, and slice updates are checked against the required array type in the specified order, with found lengths and typed leaves as specified. | CLI and unit |
| `S3L-STATIC-01` | Section 7 | A slice's bounds are static (`ORC0226`), a fixed positive distance apart (`ORC0236`), and in range at every step (`ORC0223`), with every part of each bound within the significant-bit limit of `Int` at every step. | CLI and unit |
| `S3L-CORE-01` | Section 9 | Core records a byte string as one array literal and joins, slices, and slice updates as `concat`, `slice`, and `slice_update` nodes after their operands, with omitted bounds as literals. | Unit and CLI observation |
| `S3L-EVAL-01` | Section 10 | Byte strings, joins, slices, and slice updates evaluate left to right at the specified step costs; HMAC-SHA-256 and ChaCha20-Poly1305 written with them match FIPS 180-4, RFC 4231, and RFC 8439. | CLI and unit |
| `S3L-RES-01` | Section 11 | Byte strings, joins, slices, and slice updates, their events, nodes, allocations, stack use, and inconsistent Core are bounded as specified, and foreign spans are `ORC0210`. | Generated CLI and unit |
| `S3L-COMPAT-01` | Section 13 | S3k sources keep their meaning, Core values, and output bytes, with only the specified lexical and diagnostic changes. | CLI and unit |
| `S3L-DETERMINISM-01` | Section 12 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 13. Relationship to S3k

When OEP-0015 is accepted, this document replaces these clauses of
`TUPLES_2026.md` and the documents it extends:

- the string and punctuation tokens of `LANGUAGE_2026.md` sections 2.4 and
  2.5, for hex strings, `++`, and the role of strings, by section 3;
- the primary, suffix, and update productions of `TUPLES_2026.md` section 3
  and `LOOPS_2026.md` section 3, and the operator groups of
  `EXPRESSIONS_2026.md` section 5 as `CONDITIONS_2026.md` section 3 extends
  them, by section 4;
- the typing of literals, typed leaves, and array operands, for byte strings,
  joins, slices, and slice updates, by sections 5 through 7; and
- the Core records and the step table, by sections 9 and 10.

Every source that S3k accepts has no string, hex string, `++`, or range in
brackets, so S3l accepts it with the same Core values and the same output
bytes. A source that S3k rejects gets the same diagnostics, with these
exceptions:

- `orangec lex` gives one `PLUS_PLUS` token where it gave two `PLUS` tokens,
  and one `HEX_STRING` token where it gave the identifier `hex` and a
  `STRING`; a hex string that breaks section 3 is now `ORC0009`, and an
  unterminated one is `ORC0003`, "unterminated hex string";
- a string where an expression was expected, a `+` after `+`, and `..` inside
  an index or an update's brackets were `ORC0101`; these forms are now
  accepted, or rejected by sections 4 through 7.

## 14. Explicit non-claims and future work

This slice defines no slice at a position computed from data, no slice with a
step or counted from the end, no slice of a slice or index of a slice without
a binding, no text beyond printable ASCII and no string encoding, no empty
array and so no empty string, no byte string or other array of more than 256
elements, no equality or order of whole arrays (compare them element by
element), no conversion between bytes and words (write `le32` or `be64` as a
function of four or eight bytes), no display of bytes as text, no `++` of
tuples or of values that are not arrays, no size parameters, and none of the
exclusions of `TUPLES_2026.md` section 12 that this document does not lift.

A byte string is a statement about mathematical values, not about memory:
nothing here fixes a layout, a byte order for words, an ABI, or a calling
convention for a backend, and the reference evaluator is not constant-time
(`CONDITIONS_2026.md` section 15).

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
