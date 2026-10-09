# The Orange Book

By Chase Bryan

## Part 1, The Novice

N8: Read and Repair a Program. Draft 2026-10-05.

Continue from [Name the Intermediate Step](NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md#n7-name-the-intermediate-step).
This lesson is **N8**. It is not a renumbering of the original manuscript.
The manuscript chapter titled Orange 2026: The Smallest Honest Slice keeps
its own number and its own job. N8 belongs to the novice sequence.

You already have the forms from the first six novice lessons and from N7:
`let`, `as`, fixed arrays with literal indices, `Bool` and `if`, tuples,
and a bounded `for`. This lesson adds no form of that kind. It adds the
habit of reading what `orangec` prints, predicting the code and the locus
before you confirm them, and changing only the text that the reading
identifies. A `test` declaration is the claim form this compiler already
runs. It is specified for known-answer tests. Accepting a source file,
printing a value, or passing a test is not a security claim. [Q1]

The commands below assume the repository root and the debug binary built
for this checkout, the same binary Chapter 5 runs. A listing given to
`orangec` as `-` is read from standard input, and the locus names that
input `<stdin>`. The same listing saved as a file and passed by path
produces the same code, the same line, the same column, and the same note.
Only the path written after `-->` changes. Compare the code, the span, and
the note. Do not compare an unrelated path.

## N8: Read and Repair a Program

### N8.1 What silence means

Chapter 5.8 already separated a file the checker accepts from a result you
wanted. Twelve fits in a byte whenever thirteen does. The checker has no
access to a requirement that never appears in the source.

N7 made the requirement visible as names: each update of the ChaCha quarter
round is a binding, and one input from RFC 8439 §2.1.1 is a printed vector.
Silence is still available. A binding can be legal and still be the wrong
update.

**Assumption B1.** `orangec check` accepts a source when it reports no
diagnostic. Acceptance writes nothing to standard output and nothing to
standard error, and the process status is 0. That silence means the source
was well-formed under the checks this compiler runs. It does not mean the
source matches a standard, a comment, a name, or a result you computed on
paper.

RFC 8439 §2.1 rotates `d` left by 16 after the first XOR. The next listing
rotates by 8. Every other update is the quarter round from Listing N7.7.
The literal 8 is a legal rotation amount. The parentheses are the grouping
§6.9 requires. The array length and the indices in the result are inside
the rules of N7. Before you run the checker, the prediction is therefore
not a code. The prediction is silence.

**Listing N8.1 — `silent_wrong.or`**

```orange
edition 2026;
module silent_wrong {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32]^4 {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 8;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    [a2, b2, c2, d2]
  }

  spec vector() -> Word[32]^4 {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
  }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

The streams are empty and the status is 0. The prediction stands. Silence
did not mention the rotation amount, because a legal literal is not a
diagnostic.

The input is the RFC vector from N7. N7 already computed
`a1 = 0x12131415` and `d XOR a1 = 0x13305172`. A left rotation by 8 on a
32-bit word moves each byte one place toward the high end, and the byte
that leaves the high end returns at the low end. The bytes of `0x13305172`
are `13`, `30`, `51`, and `72`, so the rotation is `0x30517213`. N7's
rotation by 16 exchanged the 16-bit halves and produced `0x51721330`.
Those are different words. Call the rotation-by-8 result `d1`.

The remaining six updates are the same functions of their inputs as in
§2.1. Carrying them out on this input yields:

| name | value |
| --- | --- |
| `a1` | `0x12131415` |
| `d1` | `0x30517213` |
| `c1` | `0xcbdee156` |
| `b1` | `0xce252cad` |
| `a2` | `0xe03840c2` |
| `d2` | `0x6932d1d0` |
| `c2` | `0x3511b326` |
| `b2` | `0x9a4fc5fd` |

The result array is `[a2, b2, c2, d2]`.

**Proposition N8.1.** On the RFC 8439 §2.1.1 input, Listing N8.1 and the
quarter round of Listing N7.7 do not denote the same array.

*Proof.* Both programs bind `a1` to `a + b` and then bind `d1` to a left
rotation of `d XOR a1`. On this input that XOR is `0x13305172`. Rotation
by 8 sends it to `0x30517213`, by the byte shift above. Rotation by 16
sends it to `0x51721330`, by the half-word exchange in N7. The two values
of `d1` differ, so the two programs differ on this input. The result
arrays differ as well: the table's `[a2, b2, c2, d2]` is
`[0xe03840c2, 0x9a4fc5fd, 0x3511b326, 0x6932d1d0]`, and the RFC result
quoted in N7 is
`[0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]`. They differ at every
position. One input is enough. □

**Expected evaluation output:**

```text
silent_wrong::vector: Word[32]^4 = [0xe03840c2, 0x9a4fc5fd, 0x3511b326, 0x6932d1d0]
```

```sh
./compiler/target/debug/orangec eval -
```

The printed array agrees with the table on this input. It does not repair
the program. The checker had nothing to say, and evaluation reported the
function that was written.

The intention was the RFC quarter round. In Listing N8.1 the other seven
updates already follow §2.1. `d2`'s rotation by 8 is the standard's second
rotation, not a second copy of the mistake. The mistake is the single
literal on `d1`. The repair replaces that `8` with `16`:

```text
let d1: Word[32] = (d ^ a1) <<< 16;
```

**Listing N8.2 — `heard_round.or`**

```orange
edition 2026;
module heard_round {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32]^4 {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    [a2, b2, c2, d2]
  }

  spec vector() -> Word[32]^4 {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
  }
}
```

A rewrite that fused the eight names into one expression would still have
to contain the amount 16. It would also throw away the names N7 exists to
give you. The checker did not ask for that rewrite. The literal was the
whole difference from Listing N7.7.

**Expected evaluation output:**

```text
heard_round::vector: Word[32]^4 = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]
```

```sh
./compiler/target/debug/orangec eval -
```

Check is silent on this listing too. The silence is assumption B1 again.
The eval line matches the RFC vector on one input, which is the same
limited agreement N7 already recorded. Proposition N7.2 still depends on
what `+`, `^`, and `<<<` denote. Matching one vector does not discharge
that assumption, and this lesson does not call either listing verified.
Verified is not a result these commands report.

### N8.2 Anatomy of a diagnostic

When the checker does speak, it speaks in a fixed shape. Read the whole
shape before you change the file.

**Assumption B2.** A diagnostic this compiler prints for a source span has
five parts. The **code** is the `ORC` identifier in brackets. The message
on that same line says what failed. The **locus** is the `-->` line, the
source excerpt, the caret, and the label after the caret: file, line,
column, and the span those carets cover. A **note**, written
`= note:`, says how to read the failure. The caret marks the span the
compiler blames. It is not an instruction to delete the character under it.

Some later diagnostics have a code, a message, and a note, and no source
span, because the offending name was on the command line rather than in
the file. Those are still read whole. They are not loci you can repair by
editing a caret you were not shown.

The next listing is the ungrouped sum from §6.9, written on the quarter
round's word type. `+` and `^` are different operator families. Before you
run the checker, name the code and the token. The code is `ORC0108`, the
same class as Listing 6.5 and Listing N7.3. The locus token is `^`, the
operator that arrives without a group. The message will say that `^`
follows `+`.

**Listing N8.3 — `bare_mix.or`, intentionally rejected**

```orange
edition 2026;
module bare_mix {
  spec step(a: Word[32], b: Word[32]) -> Word[32] {
    a + b ^ a
  }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0108]: `^` follows `+` without grouping parentheses
 --> <stdin>:4:11
  |
4 |     a + b ^ a
  |           ^ ungrouped operator
  = note: operators from different groups have no relative precedence in Orange; parenthesize the part that applies first
```

The status is 1. Standard output is empty. No value was produced for you
to repair toward.

Read it in order. The code is `ORC0108`. The message names both operators
and the order they appeared. The locus is `<stdin>`, line 4, column 11,
which is the `^` on that line. One caret covers that operator. The label
is `ungrouped operator`. The note tells you the families have no relative
precedence, and that the repair is parentheses around the part that should
happen first. The note does not choose which part. Deleting the caret's
character would turn the line into `a + b a`, which is a different
program and not a reading of this one. You have not earned an edit yet
beyond what the note describes. The edit itself is the next section.

### N8.3 Grouping and operators

The intention for Listing N8.3 is the order used when a sum is combined
with `a` by XOR: add `a` and `b`, then XOR `a`. That is one function.
The other grouping is a different function. Both are legal once they are
parenthesized. The ungrouped spelling is neither.

**Proposition N8.2.** Let `a = 0x11111111` and `b = 0x01020304`. Then
`(a + b) XOR a` and `a + (b XOR a)` are different words, taking `+` as
addition modulo 2³² and `XOR` as bitwise exclusive or.

*Proof.* N7 computed `a + b = 0x12131415`, and that sum needs no further
reduction modulo 2³². XOR with `a` acts on each byte. The high nibble of
each byte is `1 XOR 1 = 0`. The low nibbles are 2, 3, 4, and 5, and each
XOR 1 is 3, 2, 5, and 4:

```text
0x12131415 XOR 0x11111111 = 0x03020504
```

The other grouping XORs first.
`0x01020304 XOR 0x11111111 = 0x10131215`. Adding `a` then gives
`0x11111111 + 0x10131215 = 0x21242326`, again with no carry out of any
byte and no reduction modulo 2³². The results `0x03020504` and
`0x21242326` differ. □

The minimal repair is the parentheses that select the first function.
The parameters, the result type, and both operators stay. Naming the sum
is N7's job. This diagnostic asked for a group.

**Listing N8.4 — `grouped_mix.or`**

```orange
edition 2026;
module grouped_mix {
  spec step(a: Word[32], b: Word[32]) -> Word[32] {
    (a + b) ^ a
  }

  spec sample() -> Word[32] {
    step(0x11111111, 0x01020304)
  }
}
```

**Expected evaluation output:**

```text
grouped_mix::sample: Word[32] = 0x03020504
```

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval -
```

Check is silent. Eval prints the word Proposition N8.2 assigned to the
add-then-XOR reading. The other parenthesization, `a + (b ^ a)`, denotes
`0x21242326` by that same proof. Parentheses record a choice between those
two functions. The silent check records that the choice was well-formed.
You bring the requirement that decides which function you meant.

### N8.4 Conversion mistakes

Listing N7.3 rejected `x + y as Word[32]` with `ORC0108`. The two
parenthesizations are Proposition N7.1's two functions: add the bytes and
then widen the residue, or widen each byte and then add. The ungrouped
spelling sits between them. N7 stopped at the rejection. The repair is the
parentheses for the function you mean.

Take the same bytes as the proof, `x = 0xff` and `y = 0x01`, and the
intention add-then-widen. Before you run the checker, the prediction is
`ORC0108` at `as`, with a note that `as` converts one operand. The code is
the grouping class again. The locus is the conversion, not the `+` and not
the target type.

**Listing N8.5 — `bare_widen.or`, intentionally rejected**

```orange
edition 2026;
module bare_widen {
  spec mixed(x: Word[8], y: Word[8]) -> Word[32] {
    x + y as Word[32]
  }
}
```

**Diagnostic:**

```text
error[ORC0108]: `as` follows `+` without grouping parentheses
 --> <stdin>:4:11
  |
4 |     x + y as Word[32]
  |           ^^ ungrouped operator
  = note: `as` converts exactly one operand; parenthesize the conversion or the expression it converts
```

Column 11 is the `a` of `as`. Two carets cover `as`. The label is the same
label as Listing N8.3, `ungrouped operator`, because the compiler's class
is grouping. The note is the conversion note, not the precedence note.
Read the note that was printed. A repair copied from the `+` / `^` note
would still be parentheses, and it would still leave you to choose which
group. The note that belongs to this locus tells you that `as` has one
operand, so the parentheses go around the conversion or around the
expression it converts.

The minimal repair for add-then-widen parenthesizes the sum. The
parameters stay bytes. The result type stays `Word[32]`. Changing the
parameters to `Word[32]` would make the addition a different operation,
addition modulo 2³², and would answer a question the diagnostic did not
ask. The diagnostic is not a type error on `+`. Both operands of `+` are
already `Word[8]`.

**Listing N8.6 — `added_widen.or`**

```orange
edition 2026;
module added_widen {
  spec mixed(x: Word[8], y: Word[8]) -> Word[32] {
    (x + y) as Word[32]
  }

  spec sample() -> Word[32] {
    mixed(0xff, 0x01)
  }
}
```

By Proposition N7.1 the sum of 255 and 1 on a byte is 0, and widening
keeps 0. The other repair, `(x as Word[32]) + (y as Word[32])`, is also
accepted and denotes 256. Writing neither pair of parentheses does not
select the result that fits in a byte, and it does not select the result
that fits in a wider word. It selects no function.

**Expected evaluation output:**

```text
added_widen::sample: Word[32] = 0x00000000
```

```sh
./compiler/target/debug/orangec eval -
```

### N8.5 Bounds and indices

A literal index is checked before evaluation. Assumption A3 says index `k`
on an array of length `n` is accepted only when `0 ≤ k < n`. Proposition
N7.3 says that on a length-4 array the legal literal indices are 0, 1, 2,
and 3. The next listing names the function `last` and then writes 4. The
name is your intention. The checker does not read intentions out of names.
Before you run it, the prediction is `ORC0223` at the literal `4`, with a
label that the indices run from 0 through 3.

**Listing N8.7 — `past_lane.or`, intentionally rejected**

```orange
edition 2026;
module past_lane {
  spec last() -> Word[32] {
    let words: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    words[4]
  }
}
```

**Diagnostic:**

```text
error[ORC0223]: index `4` is out of range for `Word[32]^4`
 --> <stdin>:5:11
  |
5 |     words[4]
  |           ^ indices run from 0 through 3
  = note: a literal index must be less than the array's length
```

The caret is the character `4`. The message quotes that spelling. The
label states the legal range. The note states the comparison with the
length. Nothing in the diagnostic says to shorten the array, and the four
words are the RFC input. The last position of a length-4 array is 3, which
holds `d`, the word `0x01234567`. Replacing `4` with `3` is the repair
that matches the name `last`. Rebuilding the literal, or dropping its last
element so that 3 becomes a new end, would keep a different array.

**Listing N8.8 — `last_lane.or`**

```orange
edition 2026;
module last_lane {
  spec last() -> Word[32] {
    let words: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    words[3]
  }
}
```

**Expected evaluation output:**

```text
last_lane::last: Word[32] = 0x01234567
```

```sh
./compiler/target/debug/orangec eval -
```

A loop index is the same family of check, done on a range rather than on
one literal. Proposition N7.7 says that if `i` runs through 0, 1, 2, and
3, then `3 - i` stays inside 0 through 3, and `i + 1` takes the value 4.
The next listing is the rejected half of that proposition. The intention
is the reversal Listing N7.10 performs, now on four bytes. The loop header
`for i in 0..4` is already that range. The update position `[i]` is inside
it. The read `x[i + 1]` is not.

Predict `ORC0223` on the index expression `i + 1`, with a message that the
index runs from 1 through 4. The body does not run.

**Listing N8.9 — `slipped_copy.or`, intentionally rejected**

```orange
edition 2026;
module slipped_copy {
  spec reversed(x: Word[8]^4) -> Word[8]^4 {
    for i in 0..4 with s: Word[8]^4 = x { s with [i] = x[i + 1] }
  }
}
```

**Diagnostic:**

```text
error[ORC0223]: this index runs from 1 through 4, out of range for `Word[8]^4`
 --> <stdin>:4:58
  |
4 | ...  with s: Word[8]^4 = x { s with [i] = x[i + 1] }
  |                                             ^^^^^ indices run from 0 through 3
  = note: every value an index can take, over every loop index and word in it, must select an element
```

Column 58 is the start of `i + 1`. Five carets cover that expression. The
excerpt begins with `...` because the line is longer than the window the
renderer shows. The ellipsis is not part of your source, and it is not a
token to delete. The column and the carets still name `i + 1`. The note
says every value of the index must select an element. No step runs, and no
array is printed.

The repair replaces that index with `3 - i`, which Proposition N7.7 already
put inside the array. The bounds `0..4`, the accumulator, and the update
position `[i]` stay. Unrolling the reversal into four bindings would also
move the bytes, and it would discard a loop whose header was already the
right bound. The header was not the failure.

The parameter `x` has no parameterless spec to print, so the repaired
module adds `sample` to supply one input. The diagnostic named `i + 1`.
Listing N8.10 replaces that expression with `3 - i` and keeps the loop
header Listing N8.9 already used.

**Listing N8.10 — `reversed_bytes.or`**

```orange
edition 2026;
module reversed_bytes {
  spec reversed(x: Word[8]^4) -> Word[8]^4 {
    for i in 0..4 with s: Word[8]^4 = x { s with [i] = x[3 - i] }
  }

  spec sample() -> Word[8]^4 {
    reversed([0x11, 0x22, 0x33, 0x44])
  }
}
```

Follow `sample` for each `i`. The accumulator starts as
`[0x11, 0x22, 0x33, 0x44]`. Each step writes `x[3 - i]` at `[i]`.

| `i` | `3 - i` | `x[3 - i]` at `[i]` |
| --- | --- | --- |
| 0 | 3 | `0x44` |
| 1 | 2 | `0x33` |
| 2 | 1 | `0x22` |
| 3 | 0 | `0x11` |

**Expected evaluation output:**

```text
reversed_bytes::sample: Word[8]^4 = [0x44, 0x33, 0x22, 0x11]
```

```sh
./compiler/target/debug/orangec eval -
```

### N8.6 Eval, test, and the limits of either

A test is a `Bool` written next to the functions it talks about:

```text
test "TITLE" { expression }
```

The title is a quoted string. The expression has type `Bool`. `orangec
check` checks tests and does not run them. `orangec eval` runs
parameterless specs and does not run tests. `orangec test` checks the
source and then runs the tests.

**Assumption B3.** If the checks fail, `orangec test` prints the
diagnostic and no test report. If the checks pass, each test is run.
The test passes when its `Bool` is `true` and fails when its `Bool` is
`false`. A failure is not an `ORC` diagnostic: standard error stays empty,
the report goes to standard output, and the status is 1. When the
expression is a single `left == right`, the report prints both values and,
for an array or a tuple, the first position where they differ. A failure
of any other `Bool` prints the title and `FAILED`, and does not print
values.

The next module is Listing N8.2's quarter round, the RFC one, plus a test
whose expected array is that result with the last word replaced by zero.
You can predict the outcome before you run anything. Check has nothing to
reject: the body is the repaired round, and `==` on two arrays of the same
type is the comparison this compiler runs. Eval of `vector` prints the RFC
array, because that is what the body denotes on this input under the same
reading as N7. The test's left operand is `vector()`. Its right operand
ends in `0x00000000`. They differ at index 3. The report should say so,
and standard error should stay empty.

**Listing N8.11 — `wrong_claim.or`**

```orange
edition 2026;
module wrong_claim {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32]^4 {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    [a2, b2, c2, d2]
  }

  spec vector() -> Word[32]^4 {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
  }

  test "wrong_claim RFC 8439 2.1.1" {
    vector() == [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x00000000]
  }
}
```

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval -
./compiler/target/debug/orangec test -
```

Check is silent. That silence is assumption B1: the false claim is still
well-formed.

**Expected evaluation output:**

```text
wrong_claim::vector: Word[32]^4 = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]
```

**Test report:**

```text
test "wrong_claim RFC 8439 2.1.1" ... FAILED
    left:  [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]
    right: [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x00000000]
    first difference at [3]
1 test: 0 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. There is no `ORC` code
to chase. The position `[3]` is a position in the value, not a column in
the source. Left is what `vector()` returned. Right is the array written
in the test. Left is the RFC result. Right is not.

**Proposition N8.3.** On this input, the `Bool` in Listing N8.11 is false
because the claim's fourth expected word differs from the array the
function denotes. Replacing that word with `0x5881c4bb` makes the `Bool`
true. No change to `quarter_round` is required for that `Bool`.

*Proof.* The eight bindings are the eight updates of §2.1, which is the
body of Listing N7.7. Under the operator assumption of Proposition N7.2,
`vector()` denotes
`[0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]` on this input. The
test compares that array with an array that is equal in the first three
positions and equal to 0 in the last. Whole-array equality fails at the
first unequal position, which is index 3. Substituting the RFC word at
that position makes the two arrays the same value, so `==` denotes
`true`. The substitution is in the test. The bindings are untouched. □

The minimal edit is that one word on the right-hand side. Rewriting
`quarter_round` would be a response to a left value that disagreed with
the paper calculation. Left agrees. A larger edit can preserve a true
`Bool` only by preserving left, which means it is not repairing the
failure the report showed.

**Listing N8.12 — `right_claim.or`**

```orange
edition 2026;
module right_claim {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32]^4 {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    [a2, b2, c2, d2]
  }

  spec vector() -> Word[32]^4 {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
  }

  test "right_claim RFC 8439 2.1.1" {
    vector() == [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]
  }
}
```

Eval prints the same array Listing N8.11 printed. The repair did not change
the function.

**Expected evaluation output:**

```text
right_claim::vector: Word[32]^4 = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]
```

**Test report:**

```text
test "right_claim RFC 8439 2.1.1" ... ok
1 test: 1 passed, 0 failed
```

```sh
./compiler/target/debug/orangec test -
```

The passing line means this `Bool` was `true` on this run of this
evaluator. It is one input. It is the same input N7 printed. A passing
test is not acceptance — acceptance already happened, silently, while the
claim was still false — and neither of those results is what this book
would call verified. The word does not name a status these commands
return.

A false `Bool` that is not a single `==` still fails, and it does not
show you two values. `true && false` is the AND row from Chapter 3. It is
`false`. A test whose body is that expression fails without a left, a
right, or a position.

**Listing N8.13 — `plain_false.or`**

```orange
edition 2026;
module plain_false {
  test "plain_false both" { true && false }
}
```

Check is silent. Eval prints nothing, because the module has no
parameterless spec. The test report is:

```text
test "plain_false both" ... FAILED
1 test: 0 passed, 1 failed
```

If you meant to record the AND row as a claim, the `Bool` you want is the
comparison of that row with `false`. `&&` and `==` are different groups,
so the comparison is parenthesized, for the same reason Listing N8.3 was
rejected. The parentheses are the grouping repair applied to the claim,
not a change to the AND.

**Listing N8.14 — `recorded_and.or`**

```orange
edition 2026;
module recorded_and {
  test "recorded_and both" { (true && false) == false }
}
```

**Test report:**

```text
test "recorded_and both" ... ok
1 test: 1 passed, 0 failed
```

Two tools sit beside these commands. They select or limit a run. They do
not change what the source denotes.

`--spec` applies to `eval` only. It names one or more parameterless
functions and prints those. The other parameterless functions in the file
are not printed. A function that has parameters is not a legal name for
`--spec`.

**Listing N8.15 — `two_specs.or`**

```orange
edition 2026;
module two_specs {
  spec first() -> Word[32] { 0x11111111 }
  spec last() -> Word[32] { 0x01234567 }
}
```

**Expected evaluation output:**

```text
two_specs::first: Word[32] = 0x11111111
two_specs::last: Word[32] = 0x01234567
```

```sh
./compiler/target/debug/orangec eval --spec first -
```

That command prints only the `first` line. `--spec quarter_round` on
Listing N8.11 is rejected, because `quarter_round` has parameters. There
is no caret in the file. The diagnostic is `ORC1016`, and the note says
`--spec` names a function of the evaluated module that takes no
parameters. You repair the command, or you add a parameterless spec that
calls the function. You do not have a span inside `quarter_round` to edit
in answer to this code.

`orangec test --spec first` is a usage error. The status is 2, standard
output is empty, and the first line of the error is
``orangec: option `--spec` applies only to eval``. The option was the
mistake. The tests were not run.

`--steps N` sets the step budget for one `eval` or one `test`. The count
`--stats` prints is how many steps this reference evaluator charged for
this source. On Listing N8.4, that report is:

```text
grouped_mix::sample: 8 steps
total: 8 of 1048576 steps
```

The 8 is an observation of this evaluator on `sample`, which calls `step`
once on one pair of words. It is not a bound derived for every input, and
the default shown after `of` is the budget you get when you do not pass
`--steps`, here 1048576. A budget below the charge stops the run. With
`--steps 7` the same listing produces no value:

```text
error[ORC0301]: reference evaluation step limit exceeded
 --> <stdin>:7:8
  |
7 |   spec sample() -> Word[32] {
  |        ^^^^^^ evaluation stopped while evaluating this function
  = note: at most 7 evaluation steps are permitted
  = note: no partial value set is returned
  = note: `orangec eval --steps N` sets the budget, up to 1073741824 steps
```

```sh
./compiler/target/debug/orangec eval --steps 7 -
./compiler/target/debug/orangec eval --steps 8 -
```

The locus is the function being evaluated, `sample`, not an operator
inside it. The carets mean the run stopped there. They do not mean the
name `sample` is misspelled. `--steps 8` prints the same word as an
ordinary eval, `0x03020504`. Raising the budget until a value appears
lets this run finish. It does not prove Proposition N8.2 for any other
pair of words. The note's upper figure, 1073741824, is the largest `N`
the option accepts on this compiler. It is not a property of XOR, and it
is not a property of ChaCha.

A test that exceeds its budget is the same code at the test's title. The
notes say that no test outcome is reported, and they name
`orangec test --steps N`. You do not receive a `FAILED` line to repair,
because the claim was not given a truth value. `--steps 0` is not a
budget. The option's range starts at 1, and the status is the usage
status 2.

### N8.7 One repair, then a green test

The pieces now have an order. A broken group never reaches a test report.
A silent check can still hide a false claim. A false claim can be the
expected value rather than the function. This section walks one
ChaCha-shaped fragment through that order.

The fragment is the quarter round of Listing N7.9: eight named updates
and one tuple. The first rotation is written without the parentheses the
standard's order needs. `^` and `<<<` are different families, the same
class as `+` and `^`. Predict `ORC0108` at `<<<` on the `d1` line, message
`` `<<<` follows `^` ``, before you run the tool. Predict also that
`orangec test` will not print `ok` or `FAILED`. The checks run first.

**Listing N8.16 — `torn_round.or`, intentionally rejected**

```orange
edition 2026;
module torn_round {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> (Word[32], Word[32], Word[32], Word[32]) {
    let a1: Word[32] = a + b;
    let d1: Word[32] = d ^ a1 <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  test "torn_round RFC 8439 2.1.1" {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
  }
}
```

**Diagnostic:**

```text
error[ORC0108]: `<<<` follows `^` without grouping parentheses
 --> <stdin>:5:31
  |
5 |     let d1: Word[32] = d ^ a1 <<< 16;
  |                               ^^^ ungrouped operator
  = note: operators from different groups have no relative precedence in Orange; parenthesize the part that applies first
```

`orangec test` prints that same diagnostic, writes nothing to standard
output, and exits with status 1. The expected tuple in the test is the
RFC tuple. You cannot confirm it until the source is accepted. Editing
the expected tuple in answer to `ORC0108` would leave the caret where it
is.

The standard's order on that line is XOR, then rotate by 16. The minimal
repair parenthesizes `(d ^ a1)` and leaves the amount 16. The other
grouping, `d ^ (a1 <<< 16)`, rotates `a1` and then XORs. That is a
different function, and it is not §2.1. The other seven bindings already
match the standard, including the parentheses they already have. Replacing
them would be a new transcription, not the repair of this locus.

**Listing N8.17 — `mended_round.or`**

```orange
edition 2026;
module mended_round {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> (Word[32], Word[32], Word[32], Word[32]) {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  test "mended_round RFC 8439 2.1.1" {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
  }
}
```

**Test report:**

```text
test "mended_round RFC 8439 2.1.1" ... ok
1 test: 1 passed, 0 failed
```

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval -
./compiler/target/debug/orangec test -
```

Check is silent. Eval is also silent, and its status is 0, because this
module has no parameterless spec. Eval's silence here means there was
nothing it runs. It does not mean the test passed. The test command is
the one that prints `ok`. Under the operator assumption of Proposition
N7.2, the repaired `d1` is the standard's `d1`, and Proposition N7.6
already identifies the tuple with the four final words. The passing test
is that identification on one input. It does not extend the proposition
to every input, it does not include the ChaCha20 block function, and it
is not a report that the transcription has been verified.

**The finish line.** Five outcomes, in the order you had to learn them.

1. You can predict silence, run `orangec check`, and refuse to treat the
   empty report as the RFC result. Listing N8.1 is the counterexample.
   Listing N8.2 is the one-literal repair.
2. You can read a code, a locus, and a note before you edit. Listing N8.3
   is the shape. The caret is the span, and the note is the instruction
   the caret does not replace.
3. You can repair a grouping, a conversion, and an index by the smallest
   edit that restores the function you meant. Listings N8.4, N8.6, N8.8,
   and N8.10 are those edits. Each section says which larger rewrite the
   diagnostic did not ask for.
4. You can tell a silent check from a passing test, and a failed test
   from an `ORC` diagnostic. Listing N8.11 fails in the claim. Listing
   N8.12 passes. Neither status is verified.
5. You can take one broken quarter-round fragment from `ORC0108` to a
   passing `orangec test` without rewriting the updates that were already
   the standard. Listings N8.16 and N8.17 are that story. `--spec` and
   `--steps` remain tools for selecting and limiting a run.

### N8.8 Work at the desk

**Exercise N8.1 — Hear the silence.** For Listing N8.1, what does
`orangec check` write, and what is its status? Which binding's rotation
amount differs from RFC 8439 §2.1? After Listing N8.2 matches the RFC
vector, which assumption of Proposition N7.2 is still open?

**Exercise N8.2 — Name the parts.** In the diagnostic for Listing N8.3,
give the code, the line, the column, the label, and the note. Which
token do the carets cover? Why is deleting that token a different change
from the parentheses in Listing N8.4?

**Exercise N8.3 — Choose a function.** On `a = 0x11111111` and
`b = 0x01020304`, compute `(a + b) ^ a` and `a + (b ^ a)`. Which one is
`grouped_mix::sample`? Why does a silent check of the other
parenthesization fail the intention stated in N8.3?

**Exercise N8.4 — Parenthesize `as`.** Using Proposition N7.1, give both
parenthesizations of Listing N8.5 at `x = 255` and `y = 1`. Which one is
Listing N8.6? Why does leaving the parentheses out fail to mean "the
reading that fits in the result type"?

**Exercise N8.5 — The last lane.** For `Word[32]^4`, prove that 3 is a
legal literal index and 4 is not. What does the label of Listing N8.7's
diagnostic say the range is? Why does the repair keep all four words of
the array literal?

**Exercise N8.6 — The index the loop can prove.** For each `i` in
0, 1, 2, and 3, compute `i + 1` and `3 - i`. Which family is Listing
N8.10? Does the body of Listing N8.9 run? The diagnostic's excerpt starts
with `...`. What do you repair instead of that ellipsis?

**Exercise N8.7 — The claim was the failure.** In Listing N8.11, check
succeeds and the test fails. Which side of `==` is the program's value?
At which index do the sides differ, and which word is wrong? Why is
editing `quarter_round` the wrong response to this report?

**Exercise N8.8 — A budget and a name.** Listing N8.4 charges 8 steps for
`sample` on this evaluator. What does `--steps 7` print, and where is the
locus? What does `--steps 8` print? Why is moving the budget from 7 to 8
not a proof about every input of `step`? What does `--spec first` omit
from Listing N8.15, and what code does `--spec quarter_round` produce on
Listing N8.11?

**Exercise N8.9 — The fragment.** Before running a tool on Listing N8.16,
name the code and the token. After Listing N8.17, which of check, eval,
and test prints the passing claim? What do the other two print, and why
does eval's silence differ from the silence in Listing N8.1?

**Exercise N8.10 — Break a program on purpose.** Author a complete module
for a partner, using only forms from N7, that you expect the checker to
reject. State the code, the token or index at the locus, and the reason,
before your partner runs the tool. A module that checks and computes the
wrong value does not meet the exercise: silence has no code to predict.
The worked answer is one specimen that meets the conditions, not the only
module that would.

## Worked answers

**N8.1.** Check writes nothing and returns status 0. The amount that
differs from §2.1 is the `8` on `d1`. The rotation on `d2` is 8 in the
standard as well. Listing N8.2 matches the RFC vector on one input.
Proposition N7.2 still assumes that Orange's `+`, `^`, and `<<<` on
`Word[32]` denote addition modulo 2³², XOR, and left rotation. One
matching vector does not discharge that assumption.

**N8.2.** The code is `ORC0108`. The locus is line 4, column 11. The
label is `ungrouped operator`. The note says that operators from different
groups have no relative precedence, and that you parenthesize the part
that applies first. The single caret covers `^`. Deleting `^` leaves
`a + b a`, which is not a grouping of the two operations. Listing N8.4
keeps both operations and parenthesizes the sum, which is the add-then-XOR
function in Proposition N8.2.

**N8.3.** `(a + b) ^ a = 0x03020504`. `a + (b ^ a) = 0x21242326`.
`grouped_mix::sample` is the first. The second value is the other function
in Proposition N8.2. The intention in N8.3 was add, then XOR with `a`.
The parentheses in Listing N8.4 record that function. A silent check
records that the parentheses were well-formed.

**N8.4.** Add-then-widen is `(255 + 1) mod 256 = 0`, then `0` as
`Word[32]`, which is `0x00000000`. Widen-then-add is `255 + 1 = 256`,
which is `0x00000100`. Listing N8.6 is the first. The result type
`Word[32]` has room for both 0 and 256, so "whichever fits" does not
choose. Proposition N7.1 is why the compiler refuses to choose either.
The ungrouped spelling is `ORC0108`.

**N8.5.** By assumption A3, index `k` is legal when `0 ≤ k < 4`. The
integer 3 satisfies that. The integer 4 fails `4 < 4`. The label says
the indices run from 0 through 3. The four words are the RFC input, and
position 3 is `d`. The repair changes the index. Shortening the literal
would make a different array, and the diagnostic did not say the literal
had the wrong length.

**N8.6.** `i + 1` is 1, 2, 3, 4. `3 - i` is 3, 2, 1, 0. Listing N8.10
keeps the update at `[i]` and reads `x[3 - i]`, the family that stays
inside 0 through 3. Listing N8.9 is rejected with `ORC0223` before any
step, so the body does not run. The `...` is the renderer's window, not a
source token. The carets at column 58 cover `i + 1`, and that expression
is what you replace.

**N8.7.** Left is `vector()`, the program. Right is the expected array.
They differ at `[3]`. The program's word there is `0x5881c4bb`. The
claim's word is `0x00000000`. Left already matches the RFC result, so the
false `Bool` is the claim. Proposition N8.3 says replacing the claim's
last word is enough, and that `quarter_round` need not change.

**N8.8.** `--steps 7` prints `ORC0301`, reference evaluation step limit
exceeded, at `<stdin>` line 7 column 8, on the name `sample`. Standard
output is empty. The notes say at most 7 steps are permitted, no partial
value set is returned, and `--steps N` sets the budget up to 1073741824.
`--steps 8` prints `grouped_mix::sample: Word[32] = 0x03020504`. The 8
counts this run of `sample` on one input. Another input is another run,
and Proposition N8.2 was a proof about these two particular words, not a
step count. `--spec first` omits `two_specs::last`. `--spec
quarter_round` on Listing N8.11 is `ORC1016`: the module has no function
`quarter_round` without parameters.

**N8.9.** The code is `ORC0108`. The token is `<<<` on the `d1` line,
column 31, following `^`. After the parentheses, `orangec test` prints
the passing claim. Check prints nothing and returns status 0. Eval prints
nothing and returns status 0, because Listing N8.17 has no parameterless
spec. That silence means eval had no entry to run. The silence in Listing
N8.1 means a parameterless spec was accepted and, when evaluated, denoted
an array other than the RFC result. The two empty reports are different
commands with different contents available to run.

**N8.10.** One specimen is a tuple projection past the last element. The
four RFC words are a tuple, and `.4` counts from zero past `.3`. The
prediction is `ORC0223` at the `4`, with a note that a tuple's elements
are counted from zero.

**Listing N8.18 — `bad_field.or`, intentionally rejected**

```orange
edition 2026;
module bad_field {
  spec lane() -> Word[32] {
    let quad: (Word[32], Word[32], Word[32], Word[32]) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb);
    quad.4
  }
}
```

**Diagnostic:**

```text
error[ORC0223]: `(Word[32], Word[32], Word[32], Word[32])` has no element 4
 --> <stdin>:5:10
  |
5 |     quad.4
  |          ^ its elements are numbered 0 through 3
  = note: a tuple's elements are counted from zero
```

The caret covers `4`. Elements `.0` through `.3` are the four words.
This module is rejected, so it meets the exercise. A partner who instead
submits Listing N8.1 has written a real mistake and has not given you a
code to predict. You answer that partner with N8.1: run check, observe
the silence, then compare eval with the value the name claimed.

## Sources

**[R1] RFC 8439.** Y. Nir and A. Langley, “ChaCha20 and Poly1305 for IETF
Protocols,” June 2018, §2.1 and §2.1.1. The rotation amounts and the
quarter-round vector are those sections. Consulted 2026-10-05. This lesson
repairs transcriptions of that quarter round. It does not transcribe the
block function.

<https://www.rfc-editor.org/rfc/rfc8439>

**[Q1] Orange tests.** `docs/TESTS_2026.md`, proposed under OEP-0020, in
particular the command behavior: `orangec check` checks tests and runs
none, `orangec eval` runs none, and `orangec test` runs the root module's
tests under one step budget. A failed `left == right` reports both values
and the first difference. A test that exceeds the budget is `ORC0301` and
reports no outcome. `--spec` applies to `eval` only. The proposal is not
accepted by being implemented here, and a passing test is one `Bool` on
one run.
