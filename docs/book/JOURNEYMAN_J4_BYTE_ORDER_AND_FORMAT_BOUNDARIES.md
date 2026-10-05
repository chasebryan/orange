# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J4: Byte Order and Format Boundaries. Draft 2026-10-05.

Continue from
[Ready for Standards](NOVICE_N14_READY_FOR_STANDARDS.md#n14-ready-for-standards).
N14 listed endianness as an assumption a Block A reader still carries.
This lesson is that assumption, stated as functions and checked on the
compiler. The reading habit is
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
The separation of a universal claim from one Match is
[Say What You Mean](NOVICE_LOGIC.md#n9-say-what-you-mean).
N12 already loaded ChaCha20 words little-endian, by hand, in
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study).
N13 already wrote SHA-256 words big-endian on the wire, in
[Modules and Provenance](NOVICE_N13_MODULES_AND_PROVENANCE.md#n13-modules-and-provenance).
Those two lessons named the orders. They did not prove that each
order is a function with an inverse, and they did not put the
conversion in one expression the compiler checks.

This lesson is **J4**. The locked label is J4. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. The
manuscript chapter titled *From Core to Native Bytes* keeps that
title and that job. When the text says “Chapter 2” or “§5.1.1”, the
first is a novice chapter and the second is a section of a cited
standard.

J4 is Block A of the Journeyman. J5, J9, J10, J12, and J20 use the
conventions this lesson fixes. J5 derives the SHA-256 compression
function and its message schedule from FIPS 180-4 §6.2.2. This
lesson does not transcribe those functions and does not derive
them. The section number is a pointer. The functions are not on
this page.

## J4: Byte Order and Format Boundaries

> “We agree that the difference between sending eggs with the little- or the big-end first is trivial, but we insist that everyone must do it in the same way, to avoid anarchy. Since the difference is trivial we may choose either way, but a decision must be made.”
>
> — Danny Cohen, *On Holy Wars and a Plea for Peace*, IEN 137 (1 April 1980), SWIFT's POINT. [J4S1]

Cohen's difference is trivial inside one machine that never sends
the word anywhere. The moment two parties share a byte string, the
difference is the whole meaning of the integer. A standard is the
decision he asks for. It names one order, for one field, in one
edition. A later chapter that compares a vector without that
decision has not yet said which integer the bytes are.

### J4.1 The only-this-stack test

A prose book can define little-endian and big-endian. It can quote
a standard. It cannot refuse a conversion whose two sides have
different widths, and it cannot run the conversion you wrote and
print the value. This lesson does both, on the `orangec` binary
built from this tree.

The binary identifies itself as follows. The line was copied from
`orangec --version` on 2026-10-05.

```text
orangec 0.0.1 (Orange edition 2026; implemented slice S3t)
```

Slice S3t is the implemented slice this tree prints. The byte-order
forms it evaluates, `as big` and `as little`, are the S3n surface
proposed in `docs/ORDER_2026.md` under OEP-0017. That proposal is in
Review. It is not an accepted language meaning. The listings in
this lesson are checked against the binary. A listing the binary
accepts is compiler-checked. A form the binary does not implement
is labeled Proposed and is not written as a listing. This lesson
invents no syntax.

The test, which a reader of prose alone cannot perform, is this.
You take one sequence of bytes. You convert it with `as little` and
again with `as big`. You ask `orangec eval` for both values. You
write a `test` that claims the two values are equal, and you ask
`orangec test` to run it. The test fails, because the two functions
disagree on that sequence. You then repair the claim so that it
states the relation the two functions actually have, and the test
passes. A silent `orangec check` means only that the source was
well-formed. The failure, and the repair, are the content.

Every worked example in the later sections is a listing in that
sense. The expected output is the output the binary printed. Where
a listing is meant to fail, the diagnostic or the test report is
the one the binary printed, and the repair is the smallest change
that makes the stated claim true.

### J4.2 The finish line

Five outcomes finish the lesson. None of them is a certificate.
Reaching the last page does not confer them. A reader who cannot
do them on a byte string this lesson did not walk is not ready
for J5.

1. You can number the bits of a byte and the bytes of a word, and
   you can say which numbering a given sentence is using. Bit 0
   inside a byte is one convention. The first byte of a word is
   another. Silence on one is not a choice of the other.
2. You can define little-endian and big-endian as functions from
   byte strings of a fixed length to integers, prove that each is
   a bijection onto its range, give the inverse, and prove that
   each order is the other composed with reversal of the bytes.
3. You can quote the convention a named standard states, in the
   words of that edition, and check one conversion of it with
   `as little` or `as big` before any later chapter compares a
   full construction. The standards are FIPS 180-4 for big-endian
   words and the length field, RFC 8439 for ChaCha20's
   little-endian 32-bit words and for Poly1305's little-endian
   number, RFC 7748 for the little-endian u-coordinate with the
   masked top bit, and FIPS 197 for byte and State ordering,
   including the column-major copy of the input into the State.
4. You can separate bit order inside a byte from byte order
   inside a word. You can name a length field, and a padding
   rule, as the boundary of a format: the place where one field
   ends and the next begins, fixed by the standard and not by
   the length of the example in front of you.
5. You can load a ChaCha20 key big-endian, and you can compare a
   SHA-256 word in the wrong order. You can read the failing
   test, and you can make the minimal repair. You do not
   transcribe the SHA-256 compression function or its message
   schedule. Those belong to J5.

A Match on one vector is outcome 3 or outcome 5 for the inputs
the test wrote. It is not a proof that every message encodes
uniquely, and it is not called verified.

### J4.3 Assumptions that bound the lesson

Seven assumptions bound every later section. A sentence that
needs a further fact names it there.

**Assumption J4.1 — The edition token is the pin.** Every Orange
listing begins with `edition 2026;`. That token is the edition
this compiler requires. It is not a date inside FIPS 180-4, RFC
8439, RFC 7748, or FIPS 197. Each of those documents is pinned,
when this lesson quotes it, by its own edition, and the pin is
written beside the quotation. [J4T1]

**Assumption J4.2 — A word has no byte order.** A value of type
`Word[n]`, for `n` one of 8, 16, 32, or 64, is an integer from 0
through 2^n − 1. An array is a sequence of values in index order,
index 0 first. Nothing in that pair of facts says which byte is
the low byte of an integer. A byte order is a way a sequence of
words spells one integer. The compiler writes it as a conversion,
`x as big T` or `x as little T`. In `big`, the first word is the
most significant. In `little`, the first word is the least
significant. The order belongs to the conversion. It does not
belong to the word. [J4T2]

**Assumption J4.3 — The conversion keeps the bits, or it is
rejected.** Words convert to words only when both sides have the
same number of bits. Words convert to `Int` and to `Mod[m]`, and
`Int` and `Mod[m]` convert to words. A pair of word types whose
widths differ is rejected. The code this compiler prints for that
rejection is `ORC0240`. A conversion between `Int` and `Mod[m]`
does not take a byte order. These are the pairs `docs/ORDER_2026.md`
states for the proposed S3n slice, and they are the pairs this
binary checks. OEP-0017 is in Review. Reporting the binary's
behavior is not an acceptance of the proposal. [J4T2]

**Assumption J4.4 — Serialize, then compare.** A test vector is
a printed string of bytes, or a printed word, in a document.
The Orange comparison is between values a conversion produced.
Reading the hex digits of a word from the left and calling those
digits the bytes on the wire is a second function. For a
big-endian word the two functions agree. For a little-endian
word they disagree, which is the fact N12 already used when it
read `0x61707865` as the bytes `65 78 70 61`. This lesson keeps
the two functions apart, and it compares them only after the
conversion has run.

**Assumption J4.5 — A Match is not a verification.** A silent
check means the source was well-formed under the checks this
compiler runs. An evaluation means the parameterless specs that
ran produced the printed values. A passing test means the `Bool`
in that test was true on the inputs the test wrote. None of
those is a proof for every byte string, none is a claim that an
algorithm is correctly implemented beyond the listing, and none
is called verified. [J4C1]

**Assumption J4.6 — This lesson stops at the boundary.** J5
derives the SHA-256 compression function and the message
schedule. J9 studies ChaCha20 past the block function N12
stopped at. J10 studies Poly1305. J12 studies AES. J20 studies
X25519. Each of those lessons may rely on the function this
lesson proves for the order its standard names. None of them is
derived here. A listing in this file that contained a message
schedule, a quarter-round cascade, an AES round, a Poly1305
accumulator, or an X25519 ladder would have left the lesson.

**Assumption J4.7 — Bit order and byte order are different
conventions.** Numbering the bits inside one byte, and deciding
which end is bit 0, is a claim about that byte. Numbering the
bytes inside a word, and deciding which end is the least
significant byte, is a claim about that word. A standard may
state both. It may state one and be silent on the other.
Silence is not little-endian, and it is not big-endian. Cohen's
point, recorded in [J4S1], is that an inconsistent mix of the
two orders is a third convention, and that a reader who has not
separated them cannot tell which convention a diagram is using.

The forms the listings use are the ones N7, N8, N12, and N13
already ran, plus the byte-order conversions of Assumption J4.2:
`edition`, `module`, `spec`, `let`, `Word[8]`, `Word[32]`,
`Word[64]`, `Int`, hexadecimal literals, `hex"..."`, a byte
string, `++`, a slice `a[i..j]`, `as`, `as big`, `as little`,
`^`, `&`, `==`, `Bool`, `&&`, a bounded `for`, and `test`. A
listing that needs a form outside that list says so, and a form
the binary rejects is shown as a rejection.

### J4.4 What a bit, a byte, and a word are

Chapter 2 already fixed the small objects. This section names them
again, only so the functions below have a domain, and it adds the
one distinction Chapter 2 did not need: a sequence of bytes is not
yet an integer.

A bit is 0 or 1. A byte is Definition 2.2: eight bits, and as an
unsigned number an integer from 0 through 255. The reading of those
eight bits is the place-value reading of §2.4 and §2.5. Written as
bits from the left, the leftmost bit is the coefficient of 2^7 and
the rightmost bit is the coefficient of 2^0. Written as two hex
digits, the left digit is the high nibble. The byte `0xa3` is the
bit string `10100011`, because `a` is `1010` and `3` is `0011`, and
its integer value is 163. That convention is inside one byte. It
will not be reopened. Assumption J4.7 keeps it separate from the
order of several bytes.

**Definition J4.1 — Byte string.** A byte string of length k, for
an integer k ≥ 1, is a sequence b_0, b_1, …, b_{k−1} in which every
b_i is a byte. Index 0 is the first byte in the order the string is
written down, or the first byte in the order a standard prints the
string. In Orange, a value of type `Word[8]^k` is a byte string of
length k. The expression `hex"01020304"` denotes the byte string
whose four bytes are `0x01`, `0x02`, `0x03`, `0x04`, in that index
order.

**Definition J4.2 — Word.** A word of width w, where w is 8, 16,
32, or 64, is an integer from 0 through 2^w − 1. In Orange it is a
value of type `Word[w]`. A word is one integer. It does not contain
a sequence of bytes. The hex numeral `0x01020304` is a way of
writing the integer whose place values are powers of sixteen, high
digit on the left, the same rule as §2.5. That numeral is not a
byte string. Turning the integer into a byte string, or a byte
string into the integer, is a function, and the function has to be
named.

The two objects in the last paragraph are the usual place a reader
slips. The digits of `0x01020304`, read from the left, are the
bytes `01`, `02`, `03`, `04`. Those digits are the big-endian
spelling of that integer, as Definition J4.4 will say. They are not
the little-endian spelling. A listing that stores the word
little-endian will not print `[0x01, 0x02, 0x03, 0x04]`. It will
print the low byte first. The calculation is below, after the
functions have been defined, so the printed array is a consequence
and not a surprise.

Index 0 is a position in a sequence. It is not a claim that the
byte in that position is the least significant byte of an integer.
Cohen's note, in the section MEMORY ORDER of [J4S1], records the
opposite habit as well: some writers assign B0 to the least
significant bit, and some assign B0 to the most significant bit.
This lesson does not use B0. It uses index 0 for the first element
of a written sequence, and it uses a named function to say whether
that element is low or high. A sentence that says “byte 0” without
saying which of those two it means has not yet chosen.

### J4.5 Little-endian and big-endian as functions

Fix k ≥ 1. Write 256^i for the product of i factors of 256, with
256^0 = 1. Every byte is an integer from 0 through 255, so the sums
below are ordinary integers.

**Definition J4.3 — Little-endian.** The little-endian reading of
length k is the function L_k that sends a byte string to an integer:

L_k(b_0, b_1, …, b_{k−1}) = b_0 + b_1·256 + b_2·256^2 + … + b_{k−1}·256^{k−1}.

The first byte is the coefficient of 256^0. It is the least
significant byte. Each step to the right multiplies the place by
256.

**Definition J4.4 — Big-endian.** The big-endian reading of length
k is the function B_k:

B_k(b_0, b_1, …, b_{k−1}) = b_0·256^{k−1} + b_1·256^{k−2} + … + b_{k−2}·256 + b_{k−1}.

The first byte is the coefficient of the highest power. It is the
most significant byte. This is the same shape as the hex numeral in
§2.5, with base 256 instead of base 16, and with bytes instead of
digits.

Both functions take values in the set of integers N with
0 ≤ N < 256^k. A byte is at most 255, so each term b_i·256^i is at
most 255·256^i, and the little-endian sum is at most
255·(1 + 256 + … + 256^{k−1}) = 255·(256^k − 1)/255 = 256^k − 1.
The big-endian sum is the same set of terms in another order, so it
has the same bound. The value 256^k itself is not reached.

For k = 4 the powers are 1, 256, 65536, and 16777216. Take the
byte string `(0x01, 0x02, 0x03, 0x04)`.

L_4 = 1 + 2·256 + 3·65536 + 4·16777216
    = 1 + 512 + 196608 + 67108864
    = 67305985.

In hex, that integer is `0x04030201`. The bytes of the string, read
from the left, have become the bytes of the hex numeral read from
the right. That is what “first byte least significant” does to a
left-to-right hex numeral. It is not a second, informal reversal
applied afterwards. It is the sum.

B_4 = 1·16777216 + 2·65536 + 3·256 + 4
    = 16777216 + 131072 + 768 + 4
    = 16909060,

which is `0x01020304`. Here the hex numeral shows the bytes in the
order the string wrote them. The two integers differ. The string
was not a palindrome, so the two sums were not rearranging equal
terms into the same total in a way that cancels the rearrangement.
The next section proves they differ in general. This one string is
enough to show they are not the same function.

The string whose first byte is `0x01` and whose other three bytes
are `0x00` makes the place values visible with no extra terms.
L_4 of that string is 1. B_4 of that string is 16777216, which is
256^3. One function says the first byte is the units place. The
other says the first byte is the 256^3 place.

**Listing J4.1 — `spell.or`**

```orange
edition 2026;
module spell {
  spec little() -> Word[32] { hex"01020304" as little Word[32] }
  spec big() -> Word[32] { hex"01020304" as big Word[32] }
  spec little_n() -> Int { hex"01020304" as little Int }
  spec big_n() -> Int { hex"01020304" as big Int }
  spec low_first() -> Int { hex"01000000" as little Int }
  spec low_first_big() -> Int { hex"01000000" as big Int }
  spec store_little() -> Word[8]^4 {
    let w: Word[32] = 0x01020304;
    w as little Word[8]^4
  }
  spec store_big() -> Word[8]^4 {
    let w: Word[32] = 0x01020304;
    w as big Word[8]^4
  }
}
```

```sh
./compiler/target/debug/orangec check spell.or
./compiler/target/debug/orangec eval spell.or
```

Check is silent. The status is 0.

**Expected evaluation output:**

```text
spell::little: Word[32] = 0x04030201
spell::big: Word[32] = 0x01020304
spell::little_n: Int = 67305985
spell::big_n: Int = 16909060
spell::low_first: Int = 1
spell::low_first_big: Int = 16777216
spell::store_little: Word[8]^4 = [0x04, 0x03, 0x02, 0x01]
spell::store_big: Word[8]^4 = [0x01, 0x02, 0x03, 0x04]
```

`little` and `little_n` are L_4 of `(0x01, 0x02, 0x03, 0x04)`, once
as a `Word[32]` and once as an `Int`. They denote the same integer.
`big` and `big_n` are B_4. `low_first` and `low_first_big` are L_4
and B_4 of `(0x01, 0x00, 0x00, 0x00)`. The integers match the hand
sums. The Match is those six values. It is not a proof that the
compiler implements L_k for every k. That claim is the meaning
recorded for the proposed S3n slice, checked here on these inputs.
[J4T2]

`store_little` and `store_big` run the functions in the other
direction. The integer written `0x01020304` is B_4 of
`(0x01, 0x02, 0x03, 0x04)`, from the sum above. Storing it
big-endian returns those four bytes. Storing it little-endian
returns `(0x04, 0x03, 0x02, 0x01)`, because the least significant
byte of `0x01020304` is `0x04`, and little-endian writes that byte
first. The hex numeral and the little-endian byte string are
different writings of one integer. N12 already used this fact when
it read the constant `0x61707865` as the bytes `65 78 70 61`. The
functions here are the general form of that reading.

The compiler writes the same two functions for words wider than a
byte. `docs/ORDER_2026.md` states them, for the proposed slice, as
follows. Words x_0 through x_{k−1}, each of n bits, spell

N = x_0·2^{n(k−1)} + x_1·2^{n(k−2)} + … + x_{k−1}

in `big` order, and

N = x_0 + x_1·2^n + … + x_{k−1}·2^{n(k−1)}

in `little` order. For n = 8 the powers of 2^n are the powers of
256, and the two formulas are B_k and L_k. A conversion between
word types of equal width spells N from the operand and writes the
words of the target that spell the same N in the same order. A
conversion to `Int` yields N. A conversion from `Int` writes the
words that spell the residue of the integer modulo 2^{nk}. The
document is proposed. The binary evaluates these forms. Listings
in this lesson report the binary. [J4T2]

### J4.6 The representation is unique

The sums in Definitions J4.3 and J4.4 are useful only if each
integer has one byte string. Otherwise “the little-endian bytes of
N” would name a set, and a test vector could not pick one element
of it. The fact is the ordinary uniqueness of base-256 numerals.
The proof is induction on the length, using Euclidean division by
256, which Chapter 6 introduced.

**Proposition J4.1.** Let k ≥ 1. Every integer N with
0 ≤ N < 256^k can be written in exactly one way as

N = b_0 + b_1·256 + … + b_{k−1}·256^{k−1}

where each b_i is an integer and 0 ≤ b_i ≤ 255.

*Proof.* Proceed by induction on k.

Base case. If k = 1, then 0 ≤ N < 256, and the sum is the single
term b_0. The equation N = b_0 with 0 ≤ b_0 ≤ 255 forces
b_0 = N. There is one such writing.

Inductive step. Fix k ≥ 2, and assume the claim is true for length
k − 1. That assumption is the induction hypothesis. Take N with
0 ≤ N < 256^k. Euclidean division by 256 produces integers Q and R
such that N = 256·Q + R and 0 ≤ R < 256, and Q and R are the only
such integers. Then Q ≥ 0. Also Q < 256^{k−1}: if Q were at least
256^{k−1}, then N = 256·Q + R would be at least 256^k, which
contradicts N < 256^k. The induction hypothesis therefore gives
unique integers c_0, …, c_{k−2} in the range 0 through 255 with

Q = c_0 + c_1·256 + … + c_{k−2}·256^{k−2}.

Set b_0 = R and b_{i+1} = c_i for each i from 0 through k − 2.
Substitution yields

N = R + 256·Q = b_0 + b_1·256 + … + b_{k−1}·256^{k−1},

and every coefficient lies in 0 through 255. That is one writing.

It is the only one. Suppose N = d_0 + d_1·256 + … + d_{k−1}·256^{k−1}
is any writing with 0 ≤ d_i ≤ 255. Then
N = d_0 + 256·(d_1 + d_2·256 + … + d_{k−1}·256^{k−2}), and the
quantity in parentheses is an integer. Euclidean division says the
remainder on division by 256 is unique, and 0 ≤ d_0 ≤ 255, so
d_0 = R. The quotient is then Q, and the induction hypothesis says
the writing of Q is unique, so d_{i+1} = c_i = b_{i+1} for each i.
The two writings have the same coefficients.

By induction the claim holds for every k ≥ 1. □

The base case is one byte, which Chapter 2 already counted: 256
values, one each. The step does not inspect 256^k strings. It uses
one division and the hypothesis at length k − 1. That is why the
argument covers a 32-byte coordinate, whose set has 256^32
elements, without listing them.

**Proposition J4.2.** L_k is a bijection from the set of byte
strings of length k to the set of integers N with 0 ≤ N < 256^k.
The inverse sends N to the unique coefficients (b_0, …, b_{k−1})
given by Proposition J4.1.

*Proof.* For any byte string, L_k of that string is a sum of the
shape in Proposition J4.1, so it lies in the set of integers. If
L_k(b) = L_k(d), the two strings are two writings of one integer,
so Proposition J4.1 says b_i = d_i for every i. The function is
injective. Every integer in the set has at least one writing, so
every integer is L_k of that writing's bytes. The function is
surjective. A function that is injective and surjective onto its
stated codomain is a bijection. The inverse is the function that
returns, for each N, the one string Proposition J4.1 assigns to N.
Applying L_k and then that function returns the string, because the
string was a writing of L_k of itself and the writing is unique.
Applying the function and then L_k returns N, because L_k of those
coefficients is the sum, which is N. □

The inverse has a direct formula, which is the same division
written as a loop. For each i from 0 through k − 1,

b_i = floor(N / 256^i) mod 256,

where “mod 256” means the remainder on Euclidean division by 256.
In the proof, b_0 is the remainder of N, b_1 is the remainder of
the quotient, and so on. Uniqueness says no other formula, with
coefficients in range, can give a different string for the same N.

**Listing J4.2 — `roundtrip.or`**

```orange
edition 2026;
module roundtrip {
  spec back_little() -> Word[8]^4 {
    let n: Int = hex"01020304" as little Int;
    n as little Word[8]^4
  }
  spec back_big() -> Word[8]^4 {
    let n: Int = hex"01020304" as big Int;
    n as big Word[8]^4
  }
  spec same() -> Word[32] {
    let w: Word[32] = 0x01020304;
    w as big Word[32]
  }
  spec pair_big() -> Word[64] {
    let hi: Word[32] = 0x01234567;
    let lo: Word[32] = 0x89abcdef;
    [hi, lo] as big Word[64]
  }
  spec pair_little() -> Word[64] {
    let hi: Word[32] = 0x01234567;
    let lo: Word[32] = 0x89abcdef;
    [hi, lo] as little Word[64]
  }
  test "little-endian store then load returns the bytes" {
    back_little() == hex"01020304"
  }
  test "big-endian store then load returns the bytes" {
    back_big() == hex"01020304"
  }
}
```

**Expected evaluation output:**

```text
roundtrip::back_little: Word[8]^4 = [0x01, 0x02, 0x03, 0x04]
roundtrip::back_big: Word[8]^4 = [0x01, 0x02, 0x03, 0x04]
roundtrip::same: Word[32] = 0x01020304
roundtrip::pair_big: Word[64] = 0x0123456789abcdef
roundtrip::pair_little: Word[64] = 0x89abcdef01234567
```

**Test report:**

```text
test "little-endian store then load returns the bytes" ... ok
test "big-endian store then load returns the bytes" ... ok
2 tests: 2 passed, 0 failed
```

`back_little` computes L_4 of the string, then the inverse of L_4.
Proposition J4.2 says the result is the original string, for every
string of length 4. The test checks the one string
`(0x01, 0x02, 0x03, 0x04)`. The passing line is a Match on that
string. The proposition is the reason the Match is not an accident
of those four bytes. The test does not range over the other
256^4 − 1 strings. Do not call the Match verified. [J4C1]

`same` is the case k = 1 of either function, applied to a 32-bit
word rather than a byte. A single word is a sequence of one term.
Both formulas reduce to N = x_0. Converting a `Word[32]` to a
`Word[32]` in either order denotes the same word. The order is
still written, because the conversion is the place an order is
allowed to appear, but on a one-word sequence it has nothing to
rearrange.

`pair_big` and `pair_little` are the same formulas with n = 32 and
k = 2, the width-preserving case from the proposed slice. The two
words `0x01234567` and `0x89abcdef` spell

0x01234567·2^32 + 0x89abcdef

when the first word is most significant, which is the integer
written `0x0123456789abcdef`, and they spell

0x01234567 + 0x89abcdef·2^32

when the first word is least significant, which is
`0x89abcdef01234567`. The bits are the same 64 bits. The order of
the two halves is the whole difference. A later chapter that loads
a 64-bit length, or a 64-bit nonce, is choosing between these two
integers.

### J4.7 Big-endian is little-endian after reversal

**Definition J4.5 — Reversal.** The reversal of a byte string of
length k is

rev_k(b_0, b_1, …, b_{k−1}) = (b_{k−1}, …, b_1, b_0).

Index i in the result holds the byte that was at index k − 1 − i.

**Proposition J4.3.** rev_k(rev_k(s)) = s for every byte string s
of length k. In particular, rev_k is a bijection on that set.

*Proof.* The byte at index i in rev_k(s) is the byte at index
k − 1 − i in s. The byte at index i in rev_k(rev_k(s)) is the byte
at index k − 1 − i in rev_k(s), which is the byte at index
k − 1 − (k − 1 − i) = i in s. Every index is unchanged, so the
strings are equal. A function that is its own inverse is a
bijection: it is injective because equal images give equal inputs
after one more application, and it is surjective because every
string s is rev_k of rev_k(s). □

**Proposition J4.4.** For every byte string s of length k,

B_k(s) = L_k(rev_k(s)) and L_k(s) = B_k(rev_k(s)).

*Proof.* Write s = (b_0, …, b_{k−1}). Then rev_k(s) has b_{k−1−i}
at index i. Definition J4.3 gives

L_k(rev_k(s)) = Σ_{i=0}^{k−1} b_{k−1−i} · 256^i.

Set j = k − 1 − i. As i runs from 0 through k − 1, j runs from
k − 1 down through 0, and 256^i = 256^{k−1−j}. The sum is

Σ_{j=0}^{k−1} b_j · 256^{k−1−j},

which is B_k(s). That is the first equation. For the second,
replace s by rev_k(s) in the first equation:
B_k(rev_k(s)) = L_k(rev_k(rev_k(s))) = L_k(s), where the last step
is Proposition J4.3. □

**Proposition J4.5.** B_k is a bijection from byte strings of
length k to the same set of integers as L_k. The inverse of B_k
is rev_k composed with the inverse of L_k: the big-endian bytes of
N are the little-endian bytes of N, reversed.

*Proof.* B_k = L_k ∘ rev_k by Proposition J4.4. Both factors are
bijections, by Propositions J4.2 and J4.3, and a composition of
bijections is a bijection. The inverse of a composition g ∘ h is
h^{-1} ∘ g^{-1}, so the inverse of B_k is rev_k^{-1} ∘ L_k^{-1}.
Proposition J4.3 says rev_k^{-1} = rev_k. □

This is the relation the two orders have. They are not opposites
in the sense that one undoes the other. L_k does not undo B_k.
Each undoes itself, by Proposition J4.2 and Proposition J4.5.
Each is the other one, applied after the bytes are reversed.
Undoing a big-endian store with a little-endian load does not
return the original integer, unless the byte string was a
palindrome. It returns the integer of the reversed bytes.

**Proposition J4.6.** L_k and B_k are the same function exactly
when k = 1. When k > 1 they differ on the string e whose first
byte is 1 and whose remaining bytes are 0: L_k(e) = 1 and
B_k(e) = 256^{k−1}.

*Proof.* If k = 1, both definitions reduce to the identity
function on bytes, so they agree on every string. If k > 1, then
L_k(e) = 1, and B_k(e) = 1·256^{k−1} = 256^{k−1}. These integers
differ because 256^{k−1} ≥ 256 > 1. Two functions that differ on
one input are not the same function. □

Listing J4.1 already computed the k = 4 case: `low_first` printed
1 and `low_first_big` printed 16777216. The proposition says the
disagreement is not special to the bytes `01 02 03 04`. Any string
that is not equal to its reversal will do, and this one-byte spike
is the simplest.

**Listing J4.3 — `disagree.or`, intentionally failing**

```orange
edition 2026;
module disagree {
  spec little() -> Word[32] { hex"01020304" as little Word[32] }
  spec big() -> Word[32] { hex"01020304" as big Word[32] }
  test "the two orders are the same function" {
    little() == big()
  }
}
```

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec test -
```

Check is silent. The false claim is well-formed. Assumption J4.5:
silence is not truth.

**Test report:**

```text
test "the two orders are the same function" ... FAILED
    left:  0x04030201
    right: 0x01020304
1 test: 0 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. There is no
`ORC` code. Left is L_4 of the string, as a `Word[32]`. Right is
B_4. Proposition J4.6 says the equality is false for this string.
The report shows the two integers. The minimal repair is not a
change to either conversion. The claim is the false part. The
repair states the relation Proposition J4.4 actually gives.

**Listing J4.4 — `reversal.or`**

```orange
edition 2026;
module reversal {
  spec little() -> Word[32] { hex"01020304" as little Word[32] }
  spec big_of_reversed() -> Word[32] { hex"04030201" as big Word[32] }
  spec swapped() -> Word[32] {
    let w: Word[32] = 0x01020304;
    (w as little Word[8]^4) as big Word[32]
  }
  test "little of the bytes is big of the reversed bytes" {
    little() == big_of_reversed()
  }
  test "storing little and reading big reverses the bytes" {
    swapped() == 0x04030201
  }
}
```

**Expected evaluation output:**

```text
reversal::little: Word[32] = 0x04030201
reversal::big_of_reversed: Word[32] = 0x04030201
reversal::swapped: Word[32] = 0x04030201
```

**Test report:**

```text
test "little of the bytes is big of the reversed bytes" ... ok
test "storing little and reading big reverses the bytes" ... ok
2 tests: 2 passed, 0 failed
```

`big_of_reversed` is B_4 of rev_4 of `(0x01, 0x02, 0x03, 0x04)`.
Proposition J4.4 says that equals L_4 of the original string. The
first test is that equation on one string. `swapped` stores the
integer `0x01020304` little-endian, which by the inverse of L_4 is
the byte string `(0x04, 0x03, 0x02, 0x01)`, and then reads those
bytes big-endian. B_4 of that string is `0x04030201`. Reading one
order and then the other reverses the bytes of a word. It does not
return the word. A program that “converts endianness” by applying
`as little` and then `as big` has reversed the bytes. That is
sometimes the edit a standard requires, and sometimes it is the
bug. The test title says which one this listing means.

The only-this-stack test of §J4.1 is this pair of listings. A prose
account can define L_k and B_k. It cannot print the failure of
`little() == big()` from the source that states the claim, and it
cannot print the passing report after the claim is changed to the
reversal equation. Both reports were copied from the binary named
in §J4.1.

### J4.8 What the conversion refuses

Proposition J4.1 needs every coefficient, and it needs the range
0 ≤ N < 256^k. A conversion that threw away a byte, or invented
one, would not be L_k or B_k. The compiler rejects a word-to-word
conversion whose two sides have different widths. The code is
`ORC0240`. Assumption J4.3.

**Listing J4.5 — `short.or`, intentionally rejected**

```orange
edition 2026;
module short {
  spec load(b: Word[8]^3) -> Word[32] {
    b as big Word[32]
  }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0240]: `Word[8]^3` and `Word[32]` have different widths
 --> <stdin>:4:14
  |
4 |     b as big Word[32]
  |              ^^^^^^^^ `Word[32]` has 32 bits
 ::: <stdin>:4:5
  |
4 |     b as big Word[32]
  |     - `Word[8]^3` has 24 bits
  = note: a byte order keeps every bit of the words it converts, so words convert only to words of the same number of bits
```

The status is 1. Standard output is empty. The code is `ORC0240`.
The message names both types. The locus of the target is column 14
of line 4, the type `Word[32]`, and the label says that type has
32 bits. The secondary locus is the operand `b`, labeled 24 bits.
Three bytes are 24 bits. A `Word[32]` is 32 bits. The note states
the rule the propositions used: the conversion keeps every bit.

The repair is not a cast that silently pads. Padding is a decision
about which bits to add and where, and §J4.17 treats it as a format
boundary. If the string is three bytes long, the function in scope
is L_3 or B_3, whose codomain is the integers below 256^3, not the
integers below 256^4. Writing `as big Word[32]` asks for L's
cousin on the wrong set. The minimal repair, when the missing byte
is a real byte of the format, is to put that byte into the string
and then convert four bytes. The minimal repair, when the value is
only three bytes, is to convert to a type of 24 bits, and `Word[32]`
is not such a type. Inventing a zero on the left or on the right
chooses an integer. `B_3` of three bytes, placed in the low 24 bits
of a 32-bit word, is a different function from `B_4` of those bytes
with a zero in front. The diagnostic does not choose. The standard
that named the field has to choose.

One more case belongs to the conversion and not yet to a standard.
An `Int` may be negative. The proposed slice writes it as its
residue modulo 2^{nk}, the same residue a conversion to one word
already uses. For width 32, the residue of −1 is 2^32 − 1, which
is the integer whose 32 bits are all 1. Every byte of that integer
is 255, in both orders, because the unique writing of 2^{32} − 1
in base 256 is four coefficients of 255. The orders agree on the
all-ones string. They disagree on strings that are not equal to
their reversals. All-ones is a palindrome.

**Listing J4.6 — `residue.or`**

```orange
edition 2026;
module residue {
  spec ones() -> Word[8]^4 {
    let n: Int = -1;
    n as big Word[8]^4
  }
  spec ones_little() -> Word[8]^4 {
    let n: Int = -1;
    n as little Word[8]^4
  }
}
```

**Expected evaluation output:**

```text
residue::ones: Word[8]^4 = [0xff, 0xff, 0xff, 0xff]
residue::ones_little: Word[8]^4 = [0xff, 0xff, 0xff, 0xff]
```

The two arrays are equal. That equality does not retract
Proposition J4.6. It is the palindrome case. A reader who tries
`-1` as the example that separates the orders will conclude that
the orders do not matter. The string `(0x01, 0x00, 0x00, 0x00)`
separates them. The string of four `0xff` bytes does not.

The integers used in the hand sums are recorded so a later edit
can be checked without re-deriving the arithmetic from the prose.

```text
j4-ledger
little-01020304 = 67305985
big-01020304 = 16909060
low-first-little = 1
low-first-big = 16777216
place-256 = 256
place-65536 = 65536
place-16777216 = 16777216
sum-little-parts = 67305985
sum-big-parts = 16909060
word32-modulus = 4294967296
all-ones-32 = 4294967295
```

`sum-little-parts` is 1 + 2·256 + 3·65536 + 4·16777216.
`sum-big-parts` is 1·16777216 + 2·65536 + 3·256 + 4.
`all-ones-32` is the residue of −1 modulo 2^32.

### J4.9 What the later sections are allowed to use

The propositions above are the tools. A standard does not replace
them. It chooses, for each field, which function applies, and it
chooses the length k. The choice is a sentence in a named edition.
The check is a conversion whose order is the function the sentence
named, compared with a value the edition prints.

Five restrictions keep the later sections inside the lesson.

A quoted sentence is copied from the edition named beside it. A
paraphrase is marked as a paraphrase and is not the convention.
The convention is the sentence.

A listing converts bytes before it compares. It does not compare
the left-to-right digits of a hex numeral with a byte string until
it has said which function those digits are. For a big-endian field
the digits and the bytes agree. For a little-endian field they do
not. Assumption J4.4.

A passing test is a Match on the inputs written in the test.
Where a proposition has a quantifier over every string of length
k, the proof is the proposition. The test is one string.

No listing in the rest of the file transcribes a compression
function, a message schedule, a quarter-round cascade, a Poly1305
accumulator loop, an AES round, or an X25519 ladder. Each of those
is named, when it is named, as the chapter that owns it.

Where the binary has no form for a step, the step is labeled
Proposed and is not given a listing. The forms used below are the
forms Listing J4.1 through Listing J4.6 already ran, plus `&`, `|`,
array update `with`, and `Mod[m]`, each on an input the binary
accepts.


## Sources and epigraph record

The quotation is the borrowed sentence. The definitions in the
later sections are the lesson's, checked against the cited
edition of each standard. This record grows as those sections
name their documents. The epigraph is fixed here.

**[J4S1] Danny Cohen.** *On Holy Wars and a Plea for Peace.*
IEN 137, USC/Information Sciences Institute, 1 April 1980. The
epigraph is the closing paragraph of the section headed
SWIFT's POINT. In the plain-text copy the paragraph is the last
paragraph of the note. The wording, with the line breaks of that
copy joined by single spaces, is: “We agree that the difference
between sending eggs with the little- or the big-end first is
trivial, but we insist that everyone must do it in the same way,
to avoid anarchy. Since the difference is trivial we may choose
either way, but a decision must be made.” Wording was checked
on 2026-10-05 against the IETF HTML copy of IEN 137 and against
the plain-text copy at the GWDG RFC mirror. The two copies agree
on this paragraph. No translation is involved. The IEEE Computer
reprint of 1981 rewrites the opening sentence of the note; that
reprint is not the source of this epigraph. The paragraph is
not an endorsement of Orange, and it does not choose
little-endian or big-endian. It requires that a choice be made
and kept.
This record's tag is [J4S1].

Source: <https://www.ietf.org/rfc/ien/ien137.html>

Plain-text copy consulted the same day:
<https://ftp3.gwdg.de/pub/rfc/ien/ien137.txt>

**[J4T1] Orange edition.** The declaration `edition 2026;` is the
edition token required at the start of a source.
`docs/LANGUAGE_2026.md` specifies that token. The listings use
it. The token is not a publication date of any standard this
lesson cites.
This record's tag is [J4T1].

**[J4T2] Orange byte order.** `x as big T` and `x as little T`
are the conversions specified in `docs/ORDER_2026.md` for the
proposed S3n slice, under OEP-0017, which is in Review. The
document's own status line says the text is proposed and not
accepted. The binary identified in §J4.1 implements the forms
and rejects a word-to-word conversion whose widths differ with
`ORC0240`. A listing in this lesson reports that behavior. It
does not accept OEP-0017, and it does not give the Typed
Reference Core a proof role.
This record's tag is [J4T2].

**[J4C1] Match surface.** `orangec check` checks a source and
does not run its tests. `orangec eval` evaluates the
parameterless specs of the root. `orangec test` runs the tests
after the checks. A passing test is a Match on the inputs it
writes. It is not a quantifier over every byte string, and it
is not a security claim. No constant-time claim, no production
deployment claim, and no independent review is made.
This record's tag is [J4C1].

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

Sections J4.1 through J4.3 state the only-this-stack test, the
five outcomes, and the seven assumptions. Sections J4.4 through
J4.9 define the two functions, prove the bijections and the
reversal relation, and discharge the test on one byte string.
The standard listings are added with the sections that quote the
standards.
`orangec --version` was run on 2026-10-05 and printed the line
in §J4.1. That run does not establish a cryptographic security
claim, and it does not accept OEP-0017.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at
the owner's direction. Owner review is pending. No deployment
recommendation is made.
