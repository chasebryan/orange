# Orange 2026 position parameters

Status: proposed — in owner review under OEP-0027

This document specifies S3w, the slice after S3u. It does not depend on S3v.
A position parameter names every integer in a static range, so one function
can act on several positions of one state. A call chooses the integers. The
function is checked once, for every integer in each range, and it has one
instance however long the ranges are.

The terms **must**, **must not**, and **may** are normative. This proposal
accepts no semantic stratum, proof foundation, backend, or target, and it
does not decide D-004 or any Gate 0 question.

## 1. Why a position is not a size

A size parameter `n in 0..16` is one instance for each integer. Four such
sizes are 65,536 instances, and a function has at most 256, so that spelling
cannot name the four positions of one ChaCha20 quarter round. A position
parameter does not multiply instances. `quarter[a at 0..16, b at 0..16, c at
0..16, d at 0..16]` is one instance, checked for every integer each name can
take, and a call writes the four integers: `quarter[0, 4, 8, 12](s)`.

OEP-0022's P1 acceptance criterion asks for one ChaCha20 quarter-round
definition that works at several static positions of the state, each index
checked against the state shape. S3e and S3g deferred that. S3w is that
capability. It does not add lists of types shared by several functions,
slices or words at positions computed from data, or tests that claim a call
stops or a source is rejected.

## 2. Syntax

Inside a function's brackets, `a at lo..hi` is a position parameter. `at` is
that word only between the parameter's name and an integer bound. Elsewhere
`at` is an ordinary name, as in `spec spell(at: Int) -> Int { at }`. No token
is added and no word is reserved.

`lo` and `hi` are integer tokens, the same bounds a size uses. A function has
at most four parameters in brackets, counting sizes, types, and positions
together. A fifth is `ORC0101`, "a function has at most 4 parameters in
brackets", labeled "one position parameter too many" when the extra parameter
is a position. A position may sit beside sizes and types:
`row[n in 2..4, a at 0..2]`.

A call that names any position parameter must write one entry in brackets for
every bracket parameter, in order. An entry for a position is an expression.
There is no new call syntax.

## 3. Ranges and instances

A position `a at lo..hi` names each integer from `lo` up to, but not
including, `hi`. The range must be nonempty and each bound at most 65536:
`0 <= lo < hi <= 65536`. An empty range or a bound past 65536 is `ORC0243`
at the bound. The range does not add an instance. A function whose bracket
parameters are only positions has one instance, and its Core record carries
no sizes. Its displayed name is the function's name, with no brackets.

A size or a type parameter still contributes its own instances. Only the
first instance of a function in error is reported, as for sizes. A position
does not appear in that instance's name.

## 4. Names and what a position is not

A position's name shares the namespace of size parameters, value parameters,
and bindings. A repeated name is `ORC0218` or `ORC0219`, as a repeated size
is. The name has type `Int`.

A position is not a size. It must not appear in an array length, a fill
length, a loop bound, or a modulus. Those uses are `ORC0237`, the same
rejection a value that is not a size already receives, because a position is
not one integer of the instance.

A slice's bounds stay what S3l admits: integer literals, loop indices, and
size constants, with `+`, `-`, and `*` by a constant. A position in a slice
bound is `ORC0226` with the slice's existing message. An `Int` parameter, an
`Int` binding, and a call that gives an `Int` still cannot index an array.
Their `ORC0226` message is unchanged. The note of that diagnostic now also
names position parameters, because a position is an `Int` index that does
have a bound.

## 5. Checking an index

Where a position is used as an index, or as an index of an update, the
checker proves every integer in its range. The range is the closed interval
from `lo` through `hi - 1`. It combines with literals and with `+`, `-`,
`*`, `/`, `%`, and conditionals by the same interval arithmetic S3e and S3g
already use, so `(a + 1) % 4` is in range for an array of length 4 when `a`
is `at 0..4`. An index that can leave the array is `ORC0223`, as any other
index is.

The body is proved for every combination of the position ranges, including
combinations that name the same element twice. A later update wins. A call
of ChaCha20 uses four distinct positions; distinctness is not required.

## 6. Calls

Each position entry must be an integer built from integer literals and the
caller's size parameters, with `+`, `-`, `*`, `/`, `%`, prefix `-`, and
parentheses: the same expressions that compute a size. The caller's position
parameters are not single integers, so they are not positions of the call.
Anything else is `ORC0243`, "a call's position may use only integer literals
and size parameters". A value outside the declared range, or not a `u32` in
that range, is `ORC0243`, "`name` is defined for `a` at lo..hi". A part whose
magnitude exceeds the significant-bit limit of `Int` is `ORC0205`, as a size's
part is.

A call that writes no brackets, or the wrong number of entries, for a
function that has a position parameter is `ORC0239`. When the function has
only positions, the message is "`name` takes N positions, but this call gives
none" or "gives M". A function with no position parameter keeps the size and
type messages it had before this slice.

The concrete integers are not part of the instance. In the Typed Reference
Core they are `Int` parameters after the source parameters, one per position
parameter in declaration order, and the call passes them as `Int` literals
after the source arguments. One Core function therefore serves every call.
Evaluation reads the call's integers. `--spec` still evaluates only functions
with no source parameters; a quarter round is reached through a function that
calls it.

## 7. ChaCha20

`algorithms/chacha20/chacha20.or` writes one `quarter_at` over a 16-word
state and calls it at the eight positions of RFC 8439 sections 2.2 and 2.3:
the columns `(0, 4, 8, 12)` through `(3, 7, 11, 15)` and the diagonals
`(0, 5, 10, 15)`, `(1, 6, 11, 12)`, `(2, 7, 8, 13)`, and `(3, 4, 9, 14)`.
The quarter round itself remains the four-word function of section 2.1.
`compiler/fixtures/s3w/valid-quarter.or` places the section 2.1.1 inputs at
positions 0, 4, 8, and 12 and checks the published outputs
`0xea2a92f4`, `0xcb1cf8ce`, `0x4581472e`, and `0x5881c4bb`. The vectors are
cited from <https://www.rfc-editor.org/rfc/rfc8439>; they are not vendored.

## 8. Diagnostics

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0101` | parse | a fifth parameter in brackets; a position's bounds missing |
| `ORC0226` | semantic | a position used as a slice bound, or an `Int` value that still has no bound; the index message is unchanged and its note names position parameters |
| `ORC0237` | semantic | a position used as a length, a loop bound, or any other size |
| `ORC0239` | semantic | a call that does not write one entry for each position |
| `ORC0243` | semantic | an empty or oversized position range, or a call position that is not an integer in that range |

`ORC0243` is new. No other code changes its meaning. A program this slice
rejects was rejected before, or it uses the new syntax.

## 9. Resources

A position adds no instance, so it does not multiply semantic events or Core
functions. The body is checked once. Each position parameter is one extra
`Int` parameter and each call position is one `Int` literal node, inside the
unchanged budgets of 1,048,576 semantic events, 262,144 Core nodes, and
1,048,576 evaluation steps. An update of a 16-word state costs ceil(16 / 64)
steps, as any other update of 16 elements does.

## 10. Compatibility

Every source S3u accepts is accepted by S3w with the same values and the same
costs. `at` was already an identifier, and it remains one except in the new
bracket position. No existing diagnostic message changes except the note of
`ORC0226`, which gains the words "position parameters".

## 11. Non-claims

This slice does not add a position computed from data, a slice whose bounds
use a position, a distinctness check on the four positions of a call, a
universal word-width parameter, or a test that expects rejection. Nothing
here is a proof, a leakage claim, or a statement about time on a machine.
The reference evaluator is not constant-time. A fixture that matches RFC 8439
tests this implementation; it is not a verified transcription of the RFC.

### S3w conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3W-01` | Section 2 | `a at lo..hi` is a position parameter, `at` remains a name elsewhere, and a fifth bracket parameter is `ORC0101`. | CLI |
| `S3W-02` | Section 3 | A position does not multiply instances, including four ranges of 16, and a size beside a position still makes one instance per size. | CLI |
| `S3W-03` | Sections 4 and 5 | An index and an update are proved for every integer in the range, including `(a + 1) % n`, and a position is not a length, a loop bound, or a slice bound. | CLI |
| `S3W-04` | Section 6 | A call writes one static integer in each range; omitting the brackets is `ORC0239`, and a position outside the range or not static is `ORC0243`. | CLI |
| `S3W-05` | Section 4 | An `Int` parameter still cannot index an array, and that `ORC0226` message is unchanged. | CLI |
| `S3W-06` | Section 7 | One quarter round at positions 0, 4, 8, and 12 reproduces RFC 8439 section 2.1.1. | CLI |
