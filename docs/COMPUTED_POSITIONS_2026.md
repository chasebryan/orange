# Orange 2026 computed positions specification

Status: proposed S3y semantics under OEP-0029, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-10-06

This document defines S3y: positions computed from data. A remainder by a
divisor that is never zero bounds any `Int`; a `let` gives its name the
range of its value; and a slice's bounds may name such bindings, so a
window of fixed length slides to a position computed from data. It is a
delta over proposed S3u in [`DIMENSIONS_2026.md`](DIMENSIONS_2026.md), the
index rules of S3e, S3f, and S3g in [`LOOPS_2026.md`](LOOPS_2026.md),
[`CONDITIONS_2026.md`](CONDITIONS_2026.md), and
[`LOOKUPS_2026.md`](LOOKUPS_2026.md), and the slice rules of S3l in
[`BYTES_2026.md`](BYTES_2026.md). Clauses not changed here retain their
earlier meaning. The implementation is provisional evidence for review.
This text becomes normative only when the owner accepts
[OEP-0029](governance/oeps/OEP-0029-orange-2026-computed-positions.md); it accepts no
D-004 candidate, proof foundation, backend, or target.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Remainders

A remainder `e % d` whose divisor `d` has a range that does not hold zero
lies from 0 through one less than the greatest magnitude of `d`, whatever
`e` is. The dividend may be a parameter, an accumulator, a call, or any
other `Int`, with or without a range of its own:

```orange
spec at(r: Word[8]^8, n: Int) -> Word[8] { r[n % 8] }
```

`n % 8` and `n % -8` both lie from 0 through 7, because S3f's remainder is
Euclidean: it is never negative, and it is less than the divisor's
magnitude. When the divisor's range holds zero, as for `n % 0`, for
`n % (b as Int)` with `b` a word, or for a divisor from -4 through 4, the
rule does not apply: S3f defines `x % 0` as `x`, so the remainder is bounded
only when `e` is. When `e` has a
range of its own, the exact range of S3f section 9 is used as before.

## 2. Ranged bindings

A `let` that binds one name of type `Int` or a word type gives that name the
range of its value, computed when the binding is checked:

```orange
let at: Int = (m[0] as Int) % 12;   // at lies from 0 through 11
let low: Word[8] = x & 15;          // low lies from 0 through 15
```

The range is the value's range over every value of the loop indices, words,
and ranged names in it. A binding in a loop step takes the range of its
value over every step of the loop; a binding in a branch takes its value's
range in that branch. A ranged name may appear wherever S3e, S3f, and S3g
admit an index operand, and in further bindings, whose ranges follow it
through `+`, `-`, `*`, `/`, `%`, and conditionals by S3f section 9:

```orange
let here: Int = n % 7;   // 0 through 6
let next: Int = here + 1;   // 1 through 7
```

A word binding keeps the range S3g section 5 gives its value, narrowed by
its operators, so `low` above selects in a table of 16. A word binding
whose value ranges over its whole type has its type's range, as before.
An index built from a ranged name must still select an element for every
value it can take; a possibly out-of-range index is `ORC0223`, with the
note "every value an index can take, over every loop index, word, and
ranged binding in it, must select an element".

## 3. Names without a range

These have no range:

- a parameter, of any type;
- a loop accumulator, including each name of a tuple of accumulators;
- each name of a tuple pattern, as in `let (k: Int, j: Int) = (1, 2);`; and
- a binding whose value has no range, such as `let k: Int = n * 2;` for a
  parameter `n`.

A condition does not narrow a range: in `if n < 16 { t[n] } else { 0 }`,
`n` keeps the range of a parameter, which is none. An `Int` index that uses
a name without a range is `ORC0226`: "an `Int` index may use only integer
literals, loop indices, words converted with `as Int`, and ranged
bindings", labeled "this `Int` has no bound". When the name is a binding,
a secondary label at the binding's name says why it has none: "this
binding's value has no range", or "a name of a tuple pattern has no range".
The note becomes "every index is proved in range when the program is
checked: a word index ranges over its type, and an `Int` index is built
from integer literals, loop indices, words converted with `as Int`, and
ranged bindings, using `+`, `-`, `*`, `/`, `%`, and conditionals; a `let`
gives its name its value's range, and `x % 16` lies from 0 through 15
whatever x is".

## 4. Windows

A slice's bounds may be built from integer literals, loop indices, sizes,
and ranged `Int` bindings, with `+`, `-`, and `*` by a constant. A window
of fixed length then slides to a position computed from data:

```orange
let at: Int = (m[0] as Int) % 12;
m[at + 1..at + 5] as big Word[32]
```

A ranged binding whose range is one value is that value, as a literal is:
`let from: Int = 2; r[from..6]` holds four elements. A product of two
variables, loop indices or ranged bindings, is `ORC0226`: "a slice's bound
may multiply a loop index or a ranged binding only by a constant", at its
`*`, labeled "both operands of this `*` vary". Any other part is
`ORC0226`: "a slice's bounds may use only integer literals, loop indices,
and ranged bindings", labeled "this has no range", with the secondary
label of section 3 when the part names a binding without a range.

A part that has a range but is not a name, such as `n % 4` written in a
bound, is labeled "this has a range, but is not a name" and gains the note
"a computed position enters a slice's bounds through a name: bind it with
`let`, as in `let at: Int = n % 4;`, and write both bounds with `at`".
A window's bounds stay affine over named positions, so that its length can
be computed exactly before the program runs. Every such diagnostic carries
the note "a slice's length never depends on data: its bounds are built from
integer literals, loop indices, and ranged bindings with `+`, `-`, and `*`
by a constant, and differ by the same number for every value they can
take".

## 5. Window length

A window's end minus its start must be the same positive number for every
value of its loop indices and ranged bindings, as in `x[at..at + 4]`; that
number is its length, which never depends on data. Ranged bindings are
compared by name: `x[k..j]` with `let j: Int = k + 2` is rejected, though
the two differ by 2, because its bounds name different bindings. A length
that changes is `ORC0236`:

| Its bounds' difference names | Message | Label |
| --- | --- | --- |
| loop indices only | "the length of this slice changes from step to step" | "its bounds must differ by the same number at every step" |
| ranged bindings only | "the length of this slice changes with its ranged bindings" | "its bounds must differ by the same number for every value" |
| both | "the length of this slice changes from step to step and with its ranged bindings" | "its bounds must differ by the same number at every step and for every value" |

The first row's note is S3l's; the others note "a slice `x[a..b]` holds the
b - a elements from index a, and b - a must be the same positive number for
every value its ranged bindings can take, as in `x[at..at + 4]`".

## 6. Window range

Both ends of a window must lie in the array for every combination of values
its loop indices and ranged bindings can take, each over its own range. A
window that can leave the array is `ORC0223`, as in S3l: "this slice
reaches elements a through b, out of range for `T^n`", with the note
"every element a slice can take, over every loop index and ranged binding in
its bounds, must be an element of the array". A rotation by an amount from
data is a window of a row joined to itself:

```orange
let by: Int = (k as Int) % 8;
let twice: Word[8]^16 = r ++ r;
twice[by..by + 8]
```

## 7. Window updates

A slice update `x with [a..b] = v` follows sections 4 through 6 for its
bounds, and `v` must have the window's exact length and element type, as in
S3l.

## 8. Meaning and cost

S3y changes what the checker can prove, not what a program means. A ranged
name in an index or bound evaluates to its binding's value, and the index
or window selects exactly what the same expression written in place would
select. Core gains no node, the evaluator is unchanged, and every
operation costs what it cost in S3u; a `let` costs what it cost in S3c. The
evaluator still checks every index it uses and rejects inconsistent Core
with no partial output.

## 9. Positions that depend on data

S3g's lookups could be recognized by their syntax: an `Int` index built only
from literals and loop indices could not depend on data. Since S3y, a name
can carry data into an index or a window's start, as `at` does in section
4. Whether a position may depend on data is decided by following each name
to its value, not by an index's spelling. A window's length still never
depends on data; its position may.

A selection at a position computed from a secret is the access pattern that
cache-timing attacks observe. Orange makes no timing, secrecy, or leakage
claim about any index or window. A later decision on the implementation
stratum (D-004) and the native target (D-011) must say what code generation
does with a position that may depend on a secret, as `LOOKUPS_2026.md`
section 11 requires of lookups.

## 10. Unchanged boundaries

S3y infers no range for a parameter or an accumulator, narrows no range
from a condition, and adds no refinement type. It bounds no quotient
`e / d` of an `Int` without a range, admits no remainder or quotient
written in place in a slice's bounds, and admits no window whose length
depends on data. A loop's bounds remain integer literals and sizes, as in
S3e and S3m. No command, option, token, reserved word, or diagnostic code
is added; `orangec lex` is unchanged.

## 11. Compatibility, determinism, and claims

Every S3u source that checks retains its types, values, output bytes, and
evaluation steps. S3y admits indices and windows that S3u rejected, and
changes these diagnostics: the message, label, and note of `ORC0226` for
an index and for a slice bound gain ranged bindings; the notes of
`ORC0223` for an index and a slice gain ranged bindings; and S3l's note "a
slice's position never depends on data" becomes "a slice's length never
depends on data". A rejected source may now also carry the secondary label
or the note of sections 3 and 4. The same source and evaluation controls
must produce the same status, diagnostics, Core, and output bytes, and
`check`, `eval`, and `test` report identical diagnostics for a rejected
source.

This implements a bounded indexing component of the [roadmap](ROADMAP.md)
and [OEP-0022 development plan](governance/oeps/OEP-0022-crypto-language-development-plan.md).
The fixtures run FIPS 203's SampleNTT on the first 504 bytes of two
SHAKE128 outputs and FIPS 204's SampleInBall on the first 136 bytes of a
SHAKE256 output. Both write to a position that counts how many candidates
were accepted, so the position is data and only a remainder bounds it. The
streams and expected results come from Python's `hashlib` and a
transcription of each algorithm, not from published test vectors. They test
positions and selection, not complete ML-KEM or ML-DSA conformance. The
slice makes no proof, soundness, compilation, ABI, timing, secrecy,
cryptographic-security, independent-review, or production-readiness claim.

Removing S3y requires reverting the analyzer's binding ranges, remainder
rule, and window forms, the fixtures, the conformance runner, and coupled
documents together; newly admitted sources then regain their earlier
rejection.

### S3y conformance rule index

The runner `compiler/crates/orangec/tests/s3y_conformance.rs` parses this
index and requires exact agreement with its executable evidence map.

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3Y-01` | Section 1 | A remainder by a divisor whose range excludes zero lies from 0 through one less than the divisor's greatest magnitude for every dividend, and a divisor that may be zero bounds nothing. | CLI and generated CLI |
| `S3Y-02` | Section 2 | A `let` of one `Int` name gives it its value's range, in bodies, loop steps, and branches, and through chains of bindings; an index built from it is proved in range for every value or rejected. | CLI and generated CLI |
| `S3Y-03` | Section 2 | A `let` of one word name keeps its value's narrowed range. | CLI and generated CLI |
| `S3Y-04` | Section 3 | Parameters, accumulators, tuple-pattern names, and bindings of values without a range have none, and a binding's diagnostic points at where it is made. | CLI and generated CLI |
| `S3Y-05` | Section 4 | Slice bounds are affine over literals, loop indices, sizes, and ranged bindings; a one-valued binding is its value; a product of two variables and an unnamed part are rejected with their labels and notes. | CLI and generated CLI |
| `S3Y-06` | Section 5 | A window's length is the same for every value of its loop indices and bindings, and each kind of changing length has its own message. | CLI and generated CLI |
| `S3Y-07` | Section 6 | Both ends of a window are in the array for every value of its loop indices and bindings, or the window is rejected. | CLI and generated CLI |
| `S3Y-08` | Section 7 | Window updates follow the window rules and replace exactly the window. | CLI and generated CLI |
| `S3Y-09` | Section 8 | A ranged name selects exactly what its value written in place selects. | Generated CLI |
| `S3Y-10` | Section 10 | Conditions, parameters, accumulators, quotients without a range, and positions computed in a slice's bounds remain unranged. | CLI and generated CLI |
| `S3Y-11` | Section 11 | Repeated identical valid or rejected inputs have identical status, diagnostics, and output bytes across commands. | CLI and generated CLI |
