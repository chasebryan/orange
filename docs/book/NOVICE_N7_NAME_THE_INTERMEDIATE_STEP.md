# The Orange Book

By Chase Bryan

## Part 1, The Novice

N7: Name the Intermediate Step. Draft 2026-10-05.

Continue from [Words Have Edges](NOVICE_PROGRAMMING.md#chapter-6-words-have-edges).
This lesson is **N7**. It is not a renumbering of the original manuscript.
The manuscript chapter titled No Disposable Prototype keeps its own number
and its own job. N7 belongs to the novice sequence.

The listings use the Orange you already know from the first six novice
lessons, plus two forms this repository's compiler accepts in a typed `spec`
body: a `let` binding, and an `as` conversion. Both are specified for the
bindings slice and exercised by its quarter-round fixture. They are not a
new language proposal, and accepting a source file is not a security claim.
[B1]

## N7: Name the Intermediate Step

### N7.1 The value between two operations

Listing 6.6 applies three operations and returns only the last result:

```text
((x + 7) ^ 0x3c) <<< 1
```

The parentheses say which operation happens first. They do not give the
middle results names. If the final byte is wrong, you cannot point at the
program and say “this is the value after the addition.” You have to
recompute that value in your head and hope you and the expression agree.

A **binding** names one value for the rest of the body. Its form is:

```text
let name: Type = expression;
```

Three parts are written every time. `name` is the identifier you will use
later. `Type` is the type of the value, stated rather than inferred.
`expression` is computed once, and that result is what `name` denotes.
The semicolon ends the binding. The body's final expression still has no
semicolon; it is the result, as in Chapter 5.

A binding is not an assignment you may repeat. The name is fixed. The
standard you are about to read updates `a` in place. Orange will not do
that. It will give the next value of `a` a new name. That is a difference
of notation, not a difference of the number being computed, provided the
new name is defined to be exactly the updated value.

**Assumption A1.** In one typed `spec`, parameters and bindings share one
set of names. A binding may not reuse a parameter's name or an earlier
binding's name. A name is in scope after its own semicolon: in later
bindings and in the result. It is not in scope inside its own expression.

### N7.2 The same small round, with the steps visible

Take the educational byte construction from §6.10. It is still not a
cipher. Input `0xfa`, add seven, XOR `0x3c`, rotate left by one.

**Listing N7.1 — `named_round.or`**

```orange
edition 2026;
module named_round {
  spec forward(x: Word[8]) -> Word[8] {
    let added: Word[8] = x + 7;
    let masked: Word[8] = added ^ 0x3c;
    masked <<< 1
  }

  spec example() -> Word[8] {
    forward(0xfa)
  }
}
```

**Expected evaluation output:**

```text
named_round::example: Word[8] = 0x7a
```

Work the bindings before trusting the line above. `0xfa` is 250. Adding
seven on a byte wraps: 257 = 1 × 256 + 1, so `added` is `0x01`. XOR with
`0x3c` gives `0x3d`. Rotating that byte left by one gives `0x7a`. These
are the same three numbers as the trace in §6.10. The only change is that
the first two now have names in the source.

`example` calls `forward`. The bindings inside `forward` are not results
of the module. A nullary `spec` is what evaluation prints. A binding is
local to its body.

### N7.3 Say which type the bits move into

A binding can also hold a value of a different numeric type from the
expression that produced it, but only when you write the conversion.
`as` takes one operand and a target type:

```text
operand as Type
```

**Assumption A2.** `as` keeps the operand's integer value. If the target is
`Word[n]`, it then reduces that integer modulo 2ⁿ. Nothing else converts
implicitly. Chapter 6 used widths 8 and 16. `Word[32]` is that type with
width 32, so its values are the integers 0 through 2³² − 1. Widening does
not invent high bits. Narrowing keeps the low bits, which is the residue
rule you already used for wrapping.

On a byte, 255 has integer value 255. `255` as `Word[32]` is still 255,
written `0x000000ff`. The wider word has room; the value does not change.
The type does. A `Word[8]` and a `Word[32]` that happen to denote the same
integer are not the same type, just as §6.4 separated a byte sum from an
integer sum.

### N7.4 Two readings, and a form the compiler rejects

Let `x` and `y` be bytes. Consider the two expressions

```text
(x + y) as Word[32]
(x as Word[32]) + (y as Word[32])
```

**Proposition N7.1.** These expressions do not denote the same function.

*Proof.* Interpret `x` and `y` as integers in the range 0 through 255.
Byte addition denotes the sum modulo 256. So the first expression denotes
`(x + y) mod 256`, and that residue already lies in 0 through 255, which
fits in `Word[32]` without a further change. The second expression widens
first. Each widened value is `x` or `y` itself, and their sum is at most
510. Since 510 < 2³², addition on `Word[32]` does not wrap, and the second
expression denotes the integer `x + y`. These integers agree exactly when
`x + y < 256`. They disagree when `x = 255` and `y = 1`: the first result
is 0 and the second is 256. □

That is why Orange will not choose one of them for you. A conversion is
its own group. It applies to one operand. The ungrouped spelling sits
between the two meanings and is rejected.

**Listing N7.2 — `widenings.or`**

```orange
edition 2026;
module widenings {
  spec add_then_widen() -> Word[32] {
    let x: Word[8] = 0xff;
    let y: Word[8] = 0x01;
    (x + y) as Word[32]
  }

  spec widen_then_add() -> Word[32] {
    let x: Word[8] = 0xff;
    let y: Word[8] = 0x01;
    (x as Word[32]) + (y as Word[32])
  }
}
```

**Expected evaluation output:**

```text
widenings::add_then_widen: Word[32] = 0x00000000
widenings::widen_then_add: Word[32] = 0x00000100
```

`0x00000100` is 256. The successful run agrees with the two cases in the
proof. It does not replace the proof: the proof is about every pair of
bytes, and the run is one pair.

**Listing N7.3 — `mixed_conversion.or`, intentionally rejected**

```orange
edition 2026;
module mixed_conversion {
  spec mixed(x: Word[8], y: Word[8]) -> Word[32] {
    x + y as Word[32]
  }
}
```

The diagnostic is `ORC0108`: `` `as` follows `+` without grouping parentheses ``.
The note says `` `as` converts exactly one operand; parenthesize the conversion or the expression it converts ``.
No value is printed. The checker has not picked a winner between
Proposition N7.1's two functions.
[B1]

### N7.5 The quarter round, one name per update

RFC 8439 §2.1 defines the ChaCha quarter round on four 32-bit unsigned
integers `a`, `b`, `c`, and `d`. In the standard's C-like notation, `+` is
addition modulo 2³², `^` is XOR, and `<<< n` is left rotation by `n`:

```text
a += b; d ^= a; d <<<= 16;
c += d; b ^= c; b <<<= 12;
a += b; d ^= a; d <<<= 8;
c += d; b ^= c; b <<<= 7;
```

Read each statement as “the new value of this name is the old value
combined with the operation.” The standard then uses the same letter for
the new value. Under assumption A1, Orange cannot store the new value
back into `a`. Each update therefore needs a fresh name: `a1`, `d1`,
`c1`, `b1`, `a2`, `d2`, `c2`, and `b2`.

One function still returns one word, as in Chapter 5, so the transcription
uses four functions. Each function names the prefix its result depends on
and recomputes that prefix from the original inputs. The result of
`quarter_a` is `a2`, written `a1 + b1`, not a second binding of `a2`.
The result of `quarter_d` is `d2`. `quarter_c` adds `c1` to that `d2`.
`quarter_b` rotates `b1` XOR the resulting `c2`. Final `a` does not need
`d2`, `c2`, or `b2`. Final `d` needs `a2` and does not need `c2` or `b2`.
Repeating a prefix is the cost of one result. It is not a second
algorithm. Later in this lesson, one function binds all eight names and
returns all four words.

**Listing N7.4 — `quarter.or`**

```orange
edition 2026;
module quarter {
  spec quarter_a(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    a1 + b1
  }

  spec quarter_d(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    (d1 ^ a2) <<< 8
  }

  spec quarter_c(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    let d1: Word[32] = (d ^ (a + b)) <<< 16;
    let c1: Word[32] = c + d1;
    c1 + quarter_d(a, b, c, d)
  }

  spec quarter_b(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    let d1: Word[32] = (d ^ (a + b)) <<< 16;
    let b1: Word[32] = (b ^ (c + d1)) <<< 12;
    (b1 ^ quarter_c(a, b, c, d)) <<< 7
  }

  spec word_a() -> Word[32] { quarter_a(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }
  spec word_b() -> Word[32] { quarter_b(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }
  spec word_c() -> Word[32] { quarter_c(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }
  spec word_d() -> Word[32] { quarter_d(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }
}
```

The parenthesized XOR-then-rotate is the same grouping rule as §6.9.
`+` and `^` are different operator families. `^` and `<<<` are too. Each
pair is written with parentheses, so the source picks the standard's order
instead of leaving it to a precedence guess.

Section 2.1.1 of the RFC gives one test vector:

```text
a = 0x11111111
b = 0x01020304
c = 0x9b8d6f43
d = 0x01234567
```

and states that the quarter round sends it to:

```text
a = 0xea2a92f4
b = 0xcb1cf8ce
c = 0x4581472e
d = 0x5881c4bb
```

Compute the first two names by hand. The integer sum
`0x11111111 + 0x01020304` is `0x12131415`. That integer is less than 2³²,
so reduction modulo 2³² leaves it unchanged:

```text
a1 = 0x12131415
```

XOR with `d`:

```text
0x01234567 ^ 0x12131415 = 0x13305172
```

A left rotation by 16 on a 32-bit word exchanges the two 16-bit halves,
because each half moves exactly to the other half and the bits that leave
one end reenter the other. Therefore:

```text
d1 = 0x51721330
```

The remaining six names use the same three operations. Two of the additions
stay below 2³². The addition that produces `c2` does not.

`c + d1` produces no carry from one byte into the next. From the low
byte:

```text
0x43 + 0x30 = 0x73
0x6f + 0x13 = 0x82
0x8d + 0x72 = 0xff
0x9b + 0x51 = 0xec
```

The second of those sums is 111 + 19 = 130, which is `0x82` and still
below 256, so the carry inside that byte stops there. Thus `c1 = 0xecff8273`, and the
sum is less than 2³². XOR with `b`:

```text
0x01020304 ^ 0xecff8273 = 0xedfd8177
```

A left rotation by 12 moves the high 12 bits, `0xedf`, to the low end and
the low 20 bits, `0xd8177`, to the high end. Thus `b1 = 0xd8177edf`.

`a1 + b1` does involve carries, and the integer sum still fits:

```text
303240213 + 3625418463 = 3928658676 = 0xea2a92f4
```

`2³² = 4294967296`. Because 3928658676 < 4294967296, `a2 = 0xea2a92f4`.
This is already the RFC's final `a`. XOR with `d1`, then rotate left by 8.
On a 32-bit word that rotation moves each byte one place toward the high
end and brings the high byte around to the low end:

```text
0x51721330 ^ 0xea2a92f4 = 0xbb5881c4
d2 = 0x5881c4bb
```

`0x5881c4bb` is the RFC's final `d`. The next addition is the one that
wraps. In ordinary integers:

```text
0xecff8273 + 0x5881c4bb = 3976168051 + 1484899515 = 5461067566
5461067566 = 0x14581472e = 1 × 2³² + 0x4581472e
```

Addition on `Word[32]` denotes the residue modulo 2³², so the leading 1
is dropped and `c2 = 0x4581472e`. That is the RFC's final `c`. Stopping
at 5461067566 keeps the integer sum and skips that reduction. XOR with
`b1`:

```text
0xd8177edf ^ 0x4581472e = 0x9d9639f1
```

Left rotation by 7 moves the high seven bits, `1001110`, onto the low end.
The other 25 bits move up. These are the bits of `0x9d9639f1`, then the
bits after that rotation:

```text
1001 1101 1001 0110 0011 1001 1111 0001
1100 1011 0001 1100 1111 1000 1100 1110
```

The second line is `b2 = 0xcb1cf8ce`, the RFC's final `b`.

The four final names are the RFC's four results. `a2` and `d2` matched
them before the last addition. `c2` matches only after the residue is
taken, and `b2` is computed from that residue. The interesting point of
the program is still the names: each name is one update in the standard,
and each update uses only names that the preceding updates have defined.

**Proposition N7.2.** Suppose `+` on `Word[32]` denotes addition modulo
2³², `^` denotes bitwise XOR, and `<<< n` denotes left rotation by the
literal `n`. Then, on every four input words, `quarter_a`, `quarter_d`,
`quarter_c`, and `quarter_b` denote the standard's final `a`, `d`, `c`,
and `b`.

*Proof.* Define the eight mathematical values by the eight updates in
§2.1, using a fresh name wherever the standard updates a letter. Final `a`
is `a1 + b1` modulo 2³², which is the body of `quarter_a`. Final `d` is
the rotation of `d1 XOR a2` by 8, which is the body of `quarter_d`. Final
`c` is `c1` plus final `d`, and `quarter_c` writes that sum by calling
`quarter_d`. Final `b` is the rotation by 7 of `b1 XOR` final `c`, and
`quarter_b` writes that by calling `quarter_c`. Each call receives the
original four inputs, and each callee recomputes the prefix it needs from
those inputs. By the supposition, recomputing a prefix denotes the same
values as naming them once. Therefore each function denotes the
corresponding final word on every input, not merely on the test vector. □

The supposition is an assumption about what the Orange operators mean. It
is not proved by printing the test vector. The test vector is one point in
a domain of 2¹²⁸ four-word inputs. Agreement at that point is evidence
that this implementation, on that input, produced the RFC's stated output.
It is not, by itself, Proposition N7.2.

**Expected evaluation output:**

```text
quarter::word_a: Word[32] = 0xea2a92f4
quarter::word_b: Word[32] = 0xcb1cf8ce
quarter::word_c: Word[32] = 0x4581472e
quarter::word_d: Word[32] = 0x5881c4bb
```

Separate the three lines of work. The mathematics is Proposition N7.2,
under an assumption about the operators. The test is one RFC vector, which
checks one element of the domain. The implementation is whatever `orangec`
does with Listing N7.4. A match between the test and the implementation
supports the implementation at that input. It does not prove the
supposition, and it says nothing about whether ChaCha20 is a secure cipher,
whether the rest of RFC 8439 has been transcribed, or whether a different
compiler build implements the same operators.

### N7.6 Four words under one name

Listing N7.4 repeats the early updates because each function returns one
word. An **array** holds a fixed number of elements of one type as a single
value. `Word[32]^4` means four words of width 32. In a type, `^` followed
by a length is not the XOR operator from Chapter 5. XOR remains the
operator between two values, as in `d ^ a1`. A **literal index** selects
one element by an integer written in the source. Counting starts at zero.

**Assumption A3.** An array literal is checked against the length in its
type. A literal index `k` on an array of length `n` is accepted only when
`0 ≤ k < n`. That comparison happens before evaluation. An accepted
program does not find out at run time that a literal index missed the
array.

**Listing N7.5 — `lanes.or`**

```orange
edition 2026;
module lanes {
  spec words() -> Word[32]^4 {
    [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567]
  }

  spec first() -> Word[32] {
    let words: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    words[0]
  }

  spec last() -> Word[32] {
    let words: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    words[3]
  }
}
```

**Expected evaluation output:**

```text
lanes::words: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567]
lanes::first: Word[32] = 0x11111111
lanes::last: Word[32] = 0x01234567
```

`words[0]` is the first element because the positions are 0, 1, 2, and 3.
`words[3]` is the last because the length is 4 and the last position is
one less than the length. The same four numbers are the RFC's inputs.
Putting them in an array does not yet apply the quarter round.

**Proposition N7.3.** On an array of length 4, the literal indices that
select an element are 0, 1, 2, and 3, and 4 does not.

*Proof.* By assumption A3, index `k` is accepted exactly when `0 ≤ k < 4`.
The integers satisfying that inequality are 0, 1, 2, and 3. The integer 4
fails `4 < 4`. □

The next listing is the case the proof rejects. It must not print a value.

**Listing N7.6 — `past_end.or`, intentionally rejected**

```orange
edition 2026;
module past_end {
  spec missing() -> Word[32] {
    let words: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    words[4]
  }
}
```

The diagnostic is `ORC0223`: `` index `4` is out of range for `Word[32]^4` ``.
The label says the indices run from 0 through 3. The note says `` a literal index must be less than the array's length ``.
The failure is a check, not a wrapped position and not a value invented
past the end. [A1]

One function can now follow every update once and return all four results.
Position 0 holds final `a`, position 1 final `b`, position 2 final `c`,
and position 3 final `d`, because that is the order written in the array
literal.

**Listing N7.7 — `quarter_lane.or`**

```orange
edition 2026;
module quarter_lane {
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

**Expected evaluation output:**

```text
quarter_lane::vector: Word[32]^4 = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]
```

**Proposition N7.4.** Under the operator assumption of Proposition N7.2,
the four elements of `quarter_round(a, b, c, d)` are the standard's final
`a`, `b`, `c`, and `d`, in that order, on every input.

*Proof.* The eight bindings are the eight updates of RFC 8439 §2.1, in
the same order, and each binding's right-hand side uses only earlier
names. The result expression is the array whose positions are `a2`, `b2`,
`c2`, and `d2`. Those four names are the standard's final words, by the
same identification used in Proposition N7.2. A literal index selects the
element written at that position, so the array's positions 0 through 3 are
those four words. Each name is evaluated once. Recomputing a prefix in
another function is no longer required for the result to contain every
word. □

The printed vector is again one input. It matches §2.1.1. It does not
discharge the operator assumption, and the array type does not by itself
make the construction a cipher.

### N7.7 A condition chooses one value

An array holds several values at once. A condition chooses one of them.
`Bool` is the type of the two truth values `true` and `false`. A
comparison such as `x > 0x7f` or `x < 0` denotes a `Bool`. The form

```text
if condition { chosen } else { other }
```

denotes `chosen` when the condition is `true` and `other` when it is
`false`. Both branches are written with the same result type. The
condition is not a number, and a number is not a condition.

**Assumption A4.** Words compare as unsigned integers in the range
0 through 2ⁿ − 1. `Int` values compare as ordinary integers, so a
negative integer is less than zero. `&&` combines two `Bool` values by
the AND table of Chapter 3: the result is `true` only when both are
`true`. Only the branch that the condition selects is evaluated.

**Listing N7.8 — `choice.or`**

```orange
edition 2026;
module choice {
  spec both() -> Bool { true && false }

  spec high(x: Word[8]) -> Bool { x > 0x7f }

  spec sample_high() -> Bool { high(0x81) }

  spec sample_low() -> Bool { high(0x7f) }

  spec magnitude(x: Int) -> Int { if x < 0 { 0 - x } else { x } }

  spec sample_negative() -> Int { magnitude(0 - 12) }

  spec sample_positive() -> Int { magnitude(12) }
}
```

**Expected evaluation output:**

```text
choice::both: Bool = false
choice::sample_high: Bool = true
choice::sample_low: Bool = false
choice::sample_negative: Int = 12
choice::sample_positive: Int = 12
```

**Proposition N7.5.** For every byte `x`, `x > 0x7f` is `true` exactly
when the most significant bit of `x` is 1.

*Proof.* A byte is an integer from 0 through 255. `0x7f` is 127, so
`x > 0x7f` means `x ≥ 128`. Every such integer is `128 + r` with
`0 ≤ r ≤ 127`, and 128 is the place value of the leftmost bit. The seven
bits of `r` occupy the remaining positions, so the leftmost bit is 1.
Every integer from 0 through 127 is less than 128, so that bit is 0.
Assumption A4 says the comparison uses this unsigned reading. □

Thus `0x81`, which is 129, is above the threshold, and `0x7f` is not.
The two sample specs are those two cases. They do not replace the
proposition: the proposition is all 256 bytes, and the samples are two
of them.

`magnitude` is the ordinary absolute value on the integers used here.
If `x` is negative, the chosen branch is `0 - x`, which is the positive
integer of the same distance from zero. If `x` is not negative, the
chosen branch is `x` itself. For the input `0 - 12`, which is −12, the
condition `x < 0` is true, the other branch is not evaluated, and the
result is 12. For 12 the condition is false and the result is 12.

The quarter round in §2.1 does not branch. The condition is here because
later rounds and later algorithms do choose, and because a choice is a
different kind of intermediate step from a binding: the binding names a
value that is always computed, and the condition names which expression
is allowed to run.

### N7.8 The four results are one value

An array of `Word[32]` is the right container when every element has that
one type and the position is the name you mean. The standard's result is
not “element 0.” It is the four words `a`, `b`, `c`, and `d`. A **tuple**
is one value with a fixed sequence of elements, each with its own type,
selected by a position written `.0`, `.1`, `.2`, and so on, counting from
zero. `(a2, b2, c2, d2)` is that sequence. It is not a group: a group has
no comma. Two through sixteen elements are a tuple.

**Assumption A5.** A tuple's elements are evaluated from left to right.
`.k` selects the element at position `k`. The position is a decimal
integer with no sign. In this quarter round every element is `Word[32]`,
so a tuple and an array can both hold the result. The tuple still records
four positions with four types, which is what you will need when a later
result mixes a word with a `Bool` or an `Int`.

**Listing N7.9 — `paired.or`**

```orange
edition 2026;
module paired {
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

  spec vector() -> (Word[32], Word[32], Word[32], Word[32]) {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
  }

  spec final_b() -> Word[32] {
    let quad: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567);
    quad.1
  }
}
```

**Expected evaluation output:**

```text
paired::vector: (Word[32], Word[32], Word[32], Word[32]) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
paired::final_b: Word[32] = 0xcb1cf8ce
```

**Proposition N7.6.** Under the operator assumption of Proposition N7.2,
`quarter_round(a, b, c, d).0`, `.1`, `.2`, and `.3` are the standard's
final `a`, `b`, `c`, and `d`.

*Proof.* The eight bindings are the eight updates, as in Proposition N7.4.
The result is the tuple whose positions, from zero, are `a2`, `b2`, `c2`,
and `d2`. By assumption A5, `.k` is the element written in position `k`.
Those four names are the standard's final words. □

`final_b` selects position 1, which is `b2`, which the RFC's test vector
states as `0xcb1cf8ce`. The selection is not a second calculation of the
round. It reads one element of the value the round already returned.
Calling `quarter_round` again in `final_b` does recompute it, because each
call evaluates its body. The recomputation is the call, not the `.1`.

This is the quarter round as the standard writes it: four inputs, eight
named updates, four outputs. The standard updates a letter in place.
Orange names the new value. The tuple is what makes those four new values
one result.

### N7.9 Repeat only where the index is already safe

A tuple names four results. A loop names a step that is applied to each
position. Orange's loop in this lesson is bounded by two integer literals:

```text
for i in start..end with s: Type = start_value { step }
```

Read it as: `i` takes the integers `start`, `start + 1`, and so on, up to
but not including `end`. The accumulator `s` begins as `start_value`. Each
step replaces the mathematical value of `s` by the step's result. The
value of the loop is `s` after the last step. Nothing inside the array is
overwritten. `s with [i] = v` is a new array equal to `s` except at
position `i`, where it holds `v`.

**Assumption A6.** The bounds are integer literals with `0 ≤ start < end`.
The index `i` is an `Int` whose only values are `start` through `end - 1`.
An index expression built from integer literals and that loop index, using
addition, subtraction, and multiplication, is accepted only when every
value it can take selects an element. The checker computes that range
before any step runs. Evaluation of an accepted loop does not meet an
index outside the array.

`for i in 0..4` therefore gives `i` the values 0, 1, 2, and 3. It does
not give `i` the value 4.

**Proposition N7.7.** If `i` is an integer and `0 ≤ i ≤ 3`, then
`0 ≤ 3 - i ≤ 3`. If instead the index is `i + 1`, its values include 4,
which does not select an element of a length-4 array.

*Proof.* From `i ≥ 0`, subtracting `i` from 3 gives `3 - i ≤ 3`. From
`i ≤ 3`, `3 - i ≥ 0`. So `3 - i` lies in 0 through 3, which is every
legal position of a length-4 array and no others. For the second claim,
the four values of `i` produce `i + 1` equal to 1, 2, 3, and 4. The last
of those fails `4 < 4`. □

The first family is the reversal below. The second is the rejected
listing. The rejection is the proof's second claim, reported before the
body runs. There is no partial array and no wrapped index.

**Listing N7.10 — `bounded.or`**

```orange
edition 2026;
module bounded {
  spec reversed() -> Word[32]^4 {
    let x: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    for i in 0..4 with s: Word[32]^4 = x { s with [3 - i] = x[i] }
  }

  spec total() -> Int {
    let x: Word[32]^4 = [0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567];
    for i in 0..4 with s: Int = 0 { s + (x[i] as Int) }
  }
}
```

**Expected evaluation output:**

```text
bounded::reversed: Word[32]^4 = [0x01234567, 0x9b8d6f43, 0x01020304, 0x11111111]
bounded::total: Int = 2932066495
```

Follow `reversed` for each `i`. The accumulator starts as `x`.

| `i` | `3 - i` | value written |
| --- | --- | --- |
| 0 | 3 | `0x11111111` |
| 1 | 2 | `0x01020304` |
| 2 | 1 | `0x9b8d6f43` |
| 3 | 0 | `0x01234567` |

After the last step the array is the original four words in reverse
order. Each index in the table is inside 0 through 3, which is
Proposition N7.7's first claim. The checker accepts the loop because of
that range, not because a particular run happened to stay inside.

The parentheses in `s + (x[i] as Int)` are the rule from Listing N7.3.
`as` converts the selected word, and the addition sits outside that
conversion. `total` converts each selected word to `Int` and adds it to an
accumulator that starts at 0. Conversion to `Int` keeps the word's
integer value, so the four addends are 286331153, 16909060, 2609737539,
and 19088743. Their sum is 2932066495. That integer is less than 2³², so
reducing it modulo 2³² would leave it unchanged. `Int` addition and
`Word[32]` addition are still different functions: the word sum is the
residue modulo 2³², and the integer sum is not. They agree on these four
inputs because the integer fits. They disagree as soon as a sum reaches
2³². For example, `0xffffffff + 1` is 4294967296 as an `Int` and `0` as
a `Word[32]`.

**Listing N7.11 — `slipped.or`, intentionally rejected**

```orange
edition 2026;
module slipped {
  spec shift(x: Word[8]^4) -> Word[8]^4 {
    for i in 0..4 with s: Word[8]^4 = x { s with [i] = x[i + 1] }
  }
}
```

The diagnostic is `ORC0223`: `` this index runs from 1 through 4, out of range for `Word[8]^4` ``.
The label says the indices run from 0 through 3. The note says `` every value an index can take, over every loop index and word in it, must select an element ``.
No step runs. The parameter `x` is never read for a value, because the
program is rejected first.

An `Int` parameter is not built from the loop index and integer literals.
The checker has no range for it, so it does not guess that the parameter
stayed inside the array.

**Listing N7.12 — `no_range.or`, intentionally rejected**

```orange
edition 2026;
module no_range {
  spec at(k: Int, words: Word[32]^4) -> Word[32] {
    words[k]
  }
}
```

The diagnostic is `ORC0226`: `` an `Int` index may use only integer literals, loop indices, and words converted with `as Int` ``.
The label says `` this `Int` has no bound ``.
The note says `` every index is proved in range when the program is checked: a word index ranges over its type, and an `Int` index is built from integer literals, loop indices, and words converted with `as Int`, using `+`, `-`, `*`, `/`, `%`, and conditionals ``.
No step runs, and `words` is not read for a value. Listing N7.11 proved a
range and found that the range leaves the array. Listing N7.12 has no
range to prove. The division, remainder, and conditional forms in that
note are the checker's list. The indices in this lesson use addition and
subtraction of the loop index and integer literals. Those further forms
are not a new exercise. [E1]

### N7.10 The finish line

Four outcomes finish this lesson, in the order the listings introduced them.

1. **Name the step.** You can bind an intermediate value with a name, a
   stated type, and one evaluation, and you can write the ChaCha20 quarter
   round so that each update in RFC 8439 §2.1 has a name.
2. **Convert on purpose.** You can move a value between the numeric types
   with `as`, and you can say why `x + y as Word[32]` is rejected: the two
   parenthesizations are different functions.
3. **Keep several values.** You can store one type at literal indices in
   an array and reject an index that fails `0 ≤ k < n` before evaluation.
   You can choose with `Bool` and `if`, and hold a short fixed sequence
   of values in a tuple, including the four words of one quarter round.
4. **Repeat inside a proved bound.** You can write a bounded `for` whose
   index runs through a finite range known before the loop starts, and you
   can use that index only where it has already been shown to lie inside
   the array.

All four are now in the listings above, and each stops at a stated
boundary.

1. Listing N7.1 names the byte steps. Listing N7.4 names each prefix its
   result depends on. Listings N7.7 and N7.9 bind all eight updates.
   Proposition N7.2 still assumes what `+`, `^`, and `<<<` denote.
2. Listing N7.2 shows the two conversions, and Listing N7.3 is rejected
   with `ORC0108` rather than silently picking one.
3. Listing N7.5 selects positions 0 and 3. Listing N7.6 is rejected with
   `ORC0223` because 4 is not below the length. Listings N7.7, N7.8, and
   N7.9 then hold several values: the quarter round as an array, a `Bool`
   choice, and one tuple for the four words.
4. Listing N7.10 repeats inside `0..4`. Proposition N7.7 is why `3 - i`
   is in range. Listing N7.11 is rejected with `ORC0223` before any step
   runs, because the proved range of `i + 1` leaves the array. Listing
   N7.12 is rejected with `ORC0226` because an `Int` parameter has no
   range to prove.

Finishing the four outcomes does not prove ChaCha20 secure, does not prove
the compiler correct, and does not renumber the original manuscript.

### N7.11 Work at the desk

**Exercise N7.1 — Name the byte steps.** Starting from `0xfa`, compute
`added`, `masked`, and the final rotation in Listing N7.1. Which of those
three values does evaluation of the module print, and why?

**Exercise N7.2 — Separate the two sums.** For bytes `x = 200` and
`y = 100`, compute `(x + y) as Word[32]` and
`(x as Word[32]) + (y as Word[32])`. Do the same for `x = 255` and
`y = 1`. Why is `x + y as Word[32]` not a way to avoid choosing?

**Exercise N7.3 — Check the first quarter-round sum.** Show that
`0x11111111 + 0x01020304` needs no reduction modulo 2³². Give `a1`.

**Exercise N7.4 — Rotate by a half turn.** From `d XOR a1 = 0x13305172`,
obtain `d1` by a left rotation of 16. State the general fact about
16-bit halves that makes the arithmetic unnecessary.

**Exercise N7.5 — Locate the assumption.** Proposition N7.2 concludes that
the four functions match the standard on every input. Name the assumption
the proof does not discharge. What does Listing N7.4's expected output
add, and what does it still not add?

**Exercise N7.6 — Count from zero.** List every literal index that selects
an element of `Word[32]^4`. Explain why index 4 is rejected, and why a
reader who counts the elements as 1, 2, 3, 4 has named the wrong integer.

**Exercise N7.7 — One evaluation of `a1`.** Listing N7.4 computes `a + b`
in more than one function. Listing N7.7 binds `a1` once. Under Proposition
N7.4, which array position is final `a`, and which is final `d`? Why does
agreement of `vector` with the RFC line not by itself prove the proposition?

**Exercise N7.8 — Read the high bit.** For the bytes `0x81` and `0x7f`,
say whether `high` in Listing N7.8 returns `true`, and connect each answer
to the most significant bit. Then compute `magnitude` at −12 and at 12,
and say which branch runs in each case.

**Exercise N7.9 — AND is not addition.** Chapter 3's AND table says
`true` AND `false` is false. Listing N7.8 writes that as `true && false`.
Why is that result a `Bool`, not the byte 0? What would go wrong if a
later reader treated the condition of `if` as a number that can be added?

**Exercise N7.10 — Point at `b`.** In Listing N7.9, which projection
selects final `b`? Why is that `.1` rather than `.2`? Which element of
the RFC test vector must it equal, and which part of that equality is the
tuple's order rather than a new arithmetic step?

**Exercise N7.11 — Stay inside the length.** For each `i` in 0, 1, 2, and
3, compute `3 - i` and `i + 1`. Which of those two families is entirely
inside 0 through 3? What does the compiler do with the other family, and
does the loop body run? What does Listing N7.12 reject, and why is that
diagnostic not `ORC0223`?

**Exercise N7.12 — Add the words as integers.** Convert each of
`0x11111111`, `0x01020304`, `0x9b8d6f43`, and `0x01234567` to an integer
and add them. Compare the sum with `bounded::total`. Does this particular
sum change if it is reduced modulo 2³²? Give a different pair of
`Word[32]` values whose `Int` sum and `Word[32]` sum disagree.

## Worked answers

**N7.1.** `added` is `0x01`, because 250 + 7 = 257 = 1 × 256 + 1.
`masked` is `0x01 XOR 0x3c = 0x3d`. The rotation is `0x7a`. Evaluation
prints only `named_round::example`, the nullary spec. `added` and `masked`
are bindings inside `forward`. They are not module results. Calling
`forward` from `example` is what makes the final byte a printed result.

**N7.2.** 200 + 100 = 300. Modulo 256 that is 44, so the add-then-widen
result is `0x0000002c`. The widen-then-add result is 300, or `0x0000012c`.
For 255 and 1, add-then-widen is 0 and widen-then-add is 256, or
`0x00000100`, as in Listing N7.2. The ungrouped spelling is rejected with
`ORC0108` because `as` would have to attach either to `y` alone or to the
sum, and those attachments are Proposition N7.1's two functions. Leaving
out the parentheses does not select the mathematically nicer one. It
selects neither.

**N7.3.** `0x11111111 + 0x01020304 = 0x12131415 = 303240213`.
`2³² = 4294967296`. Because 303240213 < 4294967296, the residue modulo
2³² is the sum. Thus `a1 = 0x12131415`.

**N7.4.** `d1 = 0x51721330`. On a 32-bit word, left rotation by 16 sends
each 16-bit half onto the other half. The bits that leave either end are
exactly the other half, so the operation is an exchange:
`0x1330` and `0x5172` trade places. The same fact holds for every 32-bit
word, not only this one.

**N7.5.** The undischarged assumption is that Orange's `+`, `^`, and
`<<<` on `Word[32]` denote addition modulo 2³², XOR, and left rotation.
The expected output adds one checked input, the RFC 8439 §2.1.1 vector,
and only after a real run of that listing. One agreeing input does not
discharge the assumption for every operator on every word, does not cover
the other 2¹²⁸ − 1 inputs by itself, and does not establish confidentiality,
integrity, or any property of the ChaCha20 block function beyond this
quarter round.

**N7.6.** The legal literal indices are 0, 1, 2, and 3. Index 4 fails
`4 < 4`, so `ORC0223` rejects Listing N7.6 before evaluation. Counting the
elements as 1, 2, 3, 4 names four positions, but it starts at one. The
first element is position 0, so the fourth element is position 3, not 4.

**N7.7.** Final `a` is position 0, the value `a2`. Final `d` is position
3, the value `d2`. The printed vector shows those positions for one input.
Proposition N7.4 claims every input, and its proof uses the operator
assumption together with the order of the bindings. One matching line does
not discharge that assumption.

**N7.8.** `0x81` is 129, which is at least 128, so the most significant
bit is 1 and `high` is `true`. `0x7f` is 127, so that bit is 0 and `high`
is `false`. At −12 the condition `x < 0` is true, the branch `0 - x`
runs, and the result is 12. At 12 the condition is false, the branch `x`
runs, and the result is 12. The other branch is not evaluated.

**N7.9.** `true && false` has type `Bool` and value `false`. It is the
AND of two truth values, not an arithmetic sum, so it is not the byte 0
and not the integer 0 sitting in a word. An `if` condition is that `Bool`.
Adding it to a byte would mix a truth value with a residue. Orange does
not give that mixture a meaning: the condition stays a `Bool`, and the
branches stay values of the result type.

**N7.10.** Final `b` is position 1, written `.1`, because the tuple is
`(a2, b2, c2, d2)` and counting starts at zero. Position 2 is final `c`.
The RFC states final `b` as `0xcb1cf8ce`. That equality uses the order of
the tuple and the identification of `b2` with the standard's final `b`.
The `.1` does not add, XOR, or rotate. A second call of `quarter_round`
does recompute the body; the projection only reads the element.

**N7.11.** `3 - i` is 3, 2, 1, 0, all inside 0 through 3. `i + 1` is
1, 2, 3, 4. The value 4 is not a legal index of a length-4 array, so
Listing N7.11 is rejected with `ORC0223` before any step. The body does
not run, and no array is printed. Listing N7.12 is rejected with
`ORC0226`. The parameter `k` is an `Int` with no bound the checker can
compute, so there is no range to compare with the length. `words` is not
read. The two codes are different failures: a proved range that leaves
the array, and no proved range at all.

**N7.12.** The integers are 286331153, 16909060, 2609737539, and
19088743. Their sum is 2932066495, which is `bounded::total`, because
`as Int` keeps each word's value and the accumulator adds those integers.
`2³² = 4294967296`, and 2932066495 is smaller, so the residue modulo 2³²
is the same integer. The functions still differ. `0xffffffff + 1` is
4294967296 as an `Int` and 0 as a `Word[32]`.

## Sources

**[R1] RFC 8439.** Y. Nir and A. Langley, “ChaCha20 and Poly1305 for IETF
Protocols,” May 2015, §2.1 and §2.1.1. The four update lines and the
quarter-round test vector are those sections. `+`, `^`, and `<<<` in the
quotation have the meanings the RFC states there: addition modulo 2³²,
XOR, and left rotation. Consulted 2026-10-05. This lesson transcribes the
quarter round into named bindings. It does not transcribe the block
function, and it is not a recommendation to deploy an implementation.

<https://www.rfc-editor.org/rfc/rfc8439>

**[B1] Orange bindings.** `docs/BINDINGS_2026.md` and the S3c fixtures in
this repository, including the quarter round with named steps and the
rejection of an ungrouped conversion. `let` begins a binding only where a
body item starts with `let` and a name. `as` converts only after one
complete operand. Elsewhere both words can still be ordinary names. The
diagnostic quoted for Listing N7.3 is `ORC0108` with the message that
`` `as` follows `+` without grouping parentheses ``. Implementation of the
slice is not acceptance of the proposal, and it adds no cryptographic
claim.

**[A1] Orange arrays.** `docs/ARRAYS_2026.md` and the S3d fixtures in this
repository. A length is a decimal integer on the element type, and a
literal index is an integer token in brackets. The diagnostic quoted for
Listing N7.6 is `ORC0223`. An array in this lesson is a value, not a region
of memory that later text may overwrite.

**[F1] Orange conditions.** `docs/CONDITIONS_2026.md` and the S3f fixtures
in this repository. `Bool`, comparisons, `&&`, and `if` / `else` are that
slice. Word comparison is unsigned. Only the selected branch is evaluated.
Implementation of the slice is not acceptance of the proposal.

**[K1] Orange tuples.** `docs/TUPLES_2026.md` and the S3k fixtures in this
repository. A tuple lists two through sixteen elements, and `.k` selects
element `k` counting from zero. The quarter round in Listing N7.9 is the
shape of the S3k quarter round reduced to the RFC §2.1.1 vector. The tuple
slice is implemented here; that is not acceptance of the proposal.

**[E1] Orange loops.** `docs/LOOPS_2026.md` and the S3e fixtures in this
repository. A loop's bounds are integer literals, and an index built from
a loop index is proved in range before evaluation. The diagnostic quoted
for Listing N7.11 is `ORC0223`, including the computed range. Listing N7.12
is `ORC0226`: `` an `Int` index may use only integer literals, loop indices, and words converted with `as Int` ``,
with the label `` this `Int` has no bound ``. Bounded
iteration here is not a general `while`, and it adds no cryptographic
claim.
