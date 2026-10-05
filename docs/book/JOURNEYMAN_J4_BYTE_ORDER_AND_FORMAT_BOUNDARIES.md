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
./compiler/target/debug/orangec eval -
./compiler/target/debug/orangec test -
```

Check is silent. The status is 0. The false claim is well-formed.
Assumption J4.5: silence is not truth.

**Expected evaluation output:**

```text
disagree::little: Word[32] = 0x04030201
disagree::big: Word[32] = 0x01020304
```

`eval` prints the two functions on this string. It does not run
the test, so it does not say that the claim is false.

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

### J4.10 How a standard states the convention

A convention is a sentence that picks L_k or B_k, or that picks the
generalization of one of them to words of width n, for a field the
same sentence identifies. The sentence has a section number in a
named edition. A diagram, a hex dump, and a test vector are evidence
about that sentence. They are not a substitute for it. If the
sentence and the vector disagree, the edition's own rule about
which one wins applies, and this lesson quotes that rule where the
edition states one. RFC 8439 §2.3.1 says that if the pseudocode
conflicts with the textual explanation and the test vectors, the
textual explanation and the test vectors are normative. This lesson
therefore checks a vector only after the sentence that says how to
read it.

The four documents below are pinned as follows. FIPS PUB 180-4 is
the August 2015 publication, DOI `10.6028/NIST.FIPS.180-4`. [J4S2]
The SHA-256 one-block sample is the NIST examples file, which is
not a section of that publication. [J4S3] RFC 8439 is the June 2018
Informational RFC. [J4S4] RFC 7748 is the January 2016 Informational
RFC. [J4S5] FIPS 197 is the publication updated May 9, 2023, DOI
`10.6028/NIST.FIPS.197-upd1`, which states on its cover that it was
published November 26, 2001, and updated May 9, 2023. [J4S6] A
quotation is from the copy consulted on 2026-10-05. Ligatures that
the text extractor split, such as the letters of “first”, are
restored in the sentences quoted below. No sentence is otherwise
rewritten.

### J4.11 FIPS 180-4: the big-endian word and the length

Section 3.1 of FIPS 180-4, item 2, says:

> Throughout this specification, the “big-endian” convention is used when expressing both 32- and 64-bit words, so that within each word, the most significant bit is stored in the left-most bit position.

The sentence is about bits inside one word. The left-most bit is
the most significant bit. It does not, by itself, contain the word
“byte”. The byte order follows once a byte is eight consecutive
bits in that same left-to-right order, which is the grouping the
rest of the section uses when it turns a word into hex digits.

Item 3 of the same section gives the integer reading. An integer
between 0 and 2^32 − 1 inclusive may be represented as a 32-bit
word. The least significant four bits of the integer are
represented by the right-most hex digit of the word representation.
The example in the publication is the integer 291, written as the
sum 2^8 + 2^5 + 2^1 + 2^0, represented by the hex word `00000123`.
The same paragraph says an integer between 0 and 2^64 − 1 inclusive
may be represented as a 64-bit word. For SHA-256 it also records
the split: if 0 ≤ Z < 2^64, then Z = 2^32·X + Y with X and Y each
below 2^32, and Z is the pair of words (x, y). The high half is
the first word of the pair.

**Proposition J4.7.** Let a 32-bit word have its most significant
bit in the left-most position, and group the bits into four bytes
from the left, eight bits each. The integer value of the word is
B_4 of those four bytes.

*Proof.* Number the bit positions from the left as coefficients of
2^31, 2^30, …, 2^0. The first eight coefficients are those of
2^31 down through 2^24. Their contribution is a byte value, call
it b_0, multiplied by 2^24. The next eight contribute b_1·2^16,
then b_2·2^8, then b_3·2^0. Since 2^24 = 256^3, 2^16 = 256^2, and
2^8 = 256, the sum is B_4(b_0, b_1, b_2, b_3). □

The same grouping on a 64-bit word yields B_8. The pair-of-words
rule is the same function one level up: the first 32-bit word is
the coefficient of 2^32, which is big-endian order on two words of
32 bits. Listing J4.2 already computed that shape as `pair_big`.

Section 5.1.1 pads a SHA-256 message. Suppose the length of the
message M is ℓ bits. Append the bit `1`, then the smallest
non-negative number k of zero bits such that ℓ + 1 + k ≡ 448
(mod 512), then the 64-bit block that is equal to ℓ expressed using
a binary representation. The publication's example is the 8-bit
ASCII message “abc”, of length 8 × 3 = 24, padded with a one bit,
then 448 − (24 + 1) = 423 zero bits, then the length. The bit
diagram begins `01100001 01100010 01100011 1`. Those are the bytes
of `a`, `b`, and `c`, then a byte whose high bit is 1 and whose
other bits are the first of the zero pad. That byte is `0x80`.

The NIST one-block sample prints the sixteen words of that padded
block. The lines this lesson uses are only these two. [J4S3]

```text
W[0] = 61626380
W[15] = 00000018
```

W[0] is the first four bytes `61 62 63 80` read by Proposition
J4.7. W[15] is the low half of the length. The length is 24, which
is `0x18`, and the high half is zero, so the 64-bit big-endian
field is the eight bytes `00 00 00 00 00 00 00 18`. The sample does
not print those eight bytes as a hex dump. It prints the two words.
The conversion is what connects them.

This lesson does not compute W[16], and it does not apply the
compression function. Those steps are J5. The sample's later lines,
the round trace, are not copied here.

**Listing J4.7 — `sha_words.or`**

```orange
edition 2026;
module sha_words {
  spec w0() -> Word[32] { hex"61626380" as big Word[32] }
  spec w0_little() -> Word[32] { hex"61626380" as little Word[32] }
  spec length() -> Word[64] { hex"0000000000000018" as big Word[64] }
  spec length_words() -> Word[32]^2 { hex"0000000000000018" as big Word[32]^2 }
  spec length_little() -> Word[64] { hex"0000000000000018" as little Word[64] }
  test "FIPS 180-4 abc first word is big-endian" {
    w0() == 0x61626380
  }
  test "the length field is 24" {
    length() == 24
  }
}
```

**Expected evaluation output:**

```text
sha_words::w0: Word[32] = 0x61626380
sha_words::w0_little: Word[32] = 0x80636261
sha_words::length: Word[64] = 0x0000000000000018
sha_words::length_words: Word[32]^2 = [0x00000000, 0x00000018]
sha_words::length_little: Word[64] = 0x1800000000000000
```

**Test report:**

```text
test "FIPS 180-4 abc first word is big-endian" ... ok
test "the length field is 24" ... ok
2 tests: 2 passed, 0 failed
```

`w0` is B_4 of `(0x61, 0x62, 0x63, 0x80)`, and it equals the word
the sample prints. `w0_little` is L_4 of the same bytes, which is
`0x80636261`. That integer does not appear in the sample. A
comparison that uses it is not a comparison with W[0].

`length` is B_8 of the eight length bytes, and the value is 24,
which is ℓ. `length_words` is the pair (x, y) from §3.1: the first
word is 0 and the second is `0x18`, matching W[14] = 0 (not printed
above, and following from the seven high bytes being zero) and
W[15] = `00000018`. `length_little` is L_8 of those bytes. Its hex
numeral is `0x1800000000000000`. As an integer that is 24 · 2^56,
not 24. The length field is a big-endian integer. Reading it
little-endian does not produce ℓ.

**Proposition J4.8.** Under the big-endian convention of FIPS
180-4 §3.1, the 64-bit length block of the padded message “abc” is
the integer 24, and the little-endian reading of the same eight
bytes is not 24.

*Proof.* Section 5.1.1 sets ℓ = 24 for this message and appends ℓ
as a 64-bit binary block. Section 3.1 stores the most significant
bit of a 64-bit word in the left-most position, so the block is
B_8 of its eight bytes. The unique base-256 writing of 24 is seven
zero bytes followed by the byte 24, because 24 < 256. Those are
the bytes `00 00 00 00 00 00 00 18`. B_8 of them is 24. L_8 of them
is 24 · 256^7 = 24 · 2^56, which is greater than 24. □

The failing test is the claim a reader makes by reversing the
order out of habit from N12.

**Listing J4.8 — `length_wrong.or`, intentionally failing**

```orange
edition 2026;
module length_wrong {
  test "the abc length matches when the field is little-endian" {
    (hex"0000000000000018" as little Word[64]) == 24
  }
}
```

The module has no parameterless spec. `eval` prints nothing.
Check is silent.

**Test report:**

```text
test "the abc length matches when the field is little-endian" ... FAILED
    left:  0x1800000000000000
    right: 0x0000000000000018
1 test: 0 passed, 1 failed
```

Left is L_8 of the length bytes. Right is the word 24, which the
comparison prints at the width of the left operand. The two words
differ. The minimal repair is the order in the conversion: `little`
becomes `big`, which is the test already passed in Listing J4.7.
Changing 24 to `0x1800000000000000` would make the `Bool` true and
would make the claim false as a reading of §5.1.1. The report's
right-hand side is the standard's integer. The left-hand side is
the wrong function.

The same shape, on the first word, is the SHA-256 half of outcome
5. It is not a transcription of the schedule.

**Listing J4.9 — `sha_wrong.or`, intentionally failing**

```orange
edition 2026;
module sha_wrong {
  test "the abc word matches when read little-endian" {
    (hex"61626380" as little Word[32]) == 0x61626380
  }
}
```

**Test report:**

```text
test "the abc word matches when read little-endian" ... FAILED
    left:  0x80636261
    right: 0x61626380
1 test: 0 passed, 1 failed
```

Left is L_4. Right is the printed word, which Proposition J4.7
identifies with B_4. The minimal repair is again the order:
`as little` becomes `as big`. The expected word stays
`0x61626380`. J5 may compare message words with `as big`. It may
not compare them with `as little` and still be comparing the words
§3.1 defined.

### J4.12 RFC 8439: ChaCha20's little-endian words

Section 2.3 states the inputs:

> A 256-bit key, treated as a concatenation of eight 32-bit little-endian integers.
>
> A 96-bit nonce, treated as a concatenation of three 32-bit little-endian integers.
>
> A 32-bit block count parameter, treated as a 32-bit little-endian integer.

The bullets are split across lines in the plain text after
“little-”. The words above join those line breaks with a space.
“Little-endian integer”, for a 32-bit unit, is L_4 on each
four-byte group, taken in the order the bytes are written.

The same section then says the next eight words of the state are
taken from the key by reading the bytes in little-endian order, in
4-byte chunks. Word 12 is the block counter. The 13th word is the
first 32 bits of the nonce taken as a little-endian integer, and
the 15th word is the last 32 bits. After 20 rounds the result is
serialized by sequencing the words one-by-one in little-endian
order. That last sentence is the store direction: each word's
inverse of L_4, concatenated. N12 derived the block function and
the serialization of one vector. This section does not repeat the
rounds. It fixes the function the rounds' inputs and outputs use.

The four constant words are printed as `0x61707865`, `0x3320646e`,
`0x79622d32`, `0x6b206574`. N12, Proposition N12.4, already showed
they are L_4 applied to the sixteen ASCII bytes of
`expand 32-byte k`. The listing below asks the compiler for that
reading in one conversion, which N12 did by hand.

Section 2.3.2 prints the test key beginning
`00:01:02:03:04:05:06:07`, the block count 1, and the nonce
beginning `00:00:00:09`. N12 computed the first key word as
`0x03020100`, the counter word as `0x00000001`, and the first nonce
word as `0x09000000`. Those three integers are L_4 of
`(0x00, 0x01, 0x02, 0x03)`, L_4 of the four little-endian bytes of
the integer 1, and L_4 of `(0x00, 0x00, 0x00, 0x09)`.

**Listing J4.10 — `chacha_load.or`**

```orange
edition 2026;
module chacha_load {
  spec constants() -> Word[32]^4 { "expand 32-byte k" as little Word[32]^4 }
  spec key0() -> Word[32] { hex"00010203" as little Word[32] }
  spec counter() -> Word[32] {
    let n: Int = 1;
    n as little Word[32]
  }
  spec nonce0() -> Word[32] { hex"00000009" as little Word[32] }
  test "RFC 8439 constants are the little-endian words" {
    constants() == [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
  }
  test "section 2.3.2 first key word" {
    key0() == 0x03020100
  }
}
```

**Expected evaluation output:**

```text
chacha_load::constants: Word[32]^4 = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
chacha_load::key0: Word[32] = 0x03020100
chacha_load::counter: Word[32] = 0x00000001
chacha_load::nonce0: Word[32] = 0x09000000
```

**Test report:**

```text
test "RFC 8439 constants are the little-endian words" ... ok
test "section 2.3.2 first key word" ... ok
2 tests: 2 passed, 0 failed
```

The string conversion is sixteen bytes, read as four little-endian
words: L_4 on each group of four, which is the n = 8, target
`Word[32]^4` case of the proposed slice. The four results are the
constants the RFC prints. `counter` stores the integer 1 through
the inverse of L_4 and reads it back as a word; the only nonzero
byte is the first, so the word's low byte is 1 and the hex numeral
is `0x00000001`. `nonce0` is L_4 of `(0x00, 0x00, 0x00, 0x09)`,
which is `0x09000000`. A reader who copies the nonce digits into a
hex literal `0x00000009` has written B_4, not L_4. The state word
is `0x09000000`.

**Proposition J4.9.** The first key word of RFC 8439 §2.3.2 is
L_4(0x00, 0x01, 0x02, 0x03) = 0x03020100. B_4 of the same four
bytes is 0x00010203, which is not that word.

*Proof.* The key bytes are printed in order beginning `00:01:02:03`.
Section 2.3 reads each four-byte chunk as a little-endian integer,
which is L_4. Proposition J4.1 gives the unique coefficients, and
the sum is 0·1 + 1·256 + 2·65536 + 3·16777216 = 50462976 =
0x03020100. B_4 is the hex numeral of the same bytes, 0x00010203.
The two integers differ, so a test that equates them fails. □

**Listing J4.11 — `chacha_wrong.or`, intentionally failing**

```orange
edition 2026;
module chacha_wrong {
  test "the section 2.3.2 key word loaded big-endian" {
    (hex"00010203" as big Word[32]) == 0x03020100
  }
}
```

**Test report:**

```text
test "the section 2.3.2 key word loaded big-endian" ... FAILED
    left:  0x00010203
    right: 0x03020100
1 test: 0 passed, 1 failed
```

Left is what a big-endian load produced. Right is the word §2.3
requires. The minimal repair replaces `big` with `little`. The
expected word is left alone. Replacing the expected word with
`0x00010203` would silence the test and load a key the section
did not specify.

**Listing J4.12 — `chacha_repair.or`**

```orange
edition 2026;
module chacha_repair {
  test "the section 2.3.2 key word loaded little-endian" {
    (hex"00010203" as little Word[32]) == 0x03020100
  }
}
```

**Test report:**

```text
test "the section 2.3.2 key word loaded little-endian" ... ok
1 test: 1 passed, 0 failed
```

That passing line is a Match on four bytes. It is not the ChaCha20
block function, and it is not encryption. J9 may take L_4 on each
32-bit input word, and the inverse of L_4 on each output word, as
the convention §2.3 stated. J9 does not get to choose the other
order for those fields.

### J4.13 RFC 8439: Poly1305's little-endian number

Section 2.5 partitions the 32-byte key into r and s. It says r is
treated as a 16-octet little-endian number, and then it clears
bits: r[3], r[7], r[11], and r[15] have their top four bits clear,
and r[4], r[8], and r[12] have their bottom two bits clear. The
sample C in that section is `r[3] &= 15`, and the same for indices
7, 11, and 15, then `r[4] &= 252`, and the same for 8 and 12. The
pseudocode in §2.5.1 writes one mask,
`r &= 0x0ffffffc0ffffffc0ffffffc0fffffff`, after
`r = le_bytes_to_num(key[0..15])`. Clamping the bytes and clamping
the integer are the same operation when the integer is the
little-endian reading, because each masked bit sits in a known
byte. This lesson clamps the bytes, which is the sample's order,
and then applies L_16.

The number itself, for a message block, is the next sentence of
§2.5. Divide the message into 16-byte blocks. Read the block as a
little-endian number. Add one bit beyond the number of octets. For
a 16-byte block this is equivalent to adding 2^128. The pseudocode
writes that bit as a following byte `0x01`:

```text
n = le_bytes_to_num(msg[((i-1)*16)..(i*16)] | [0x01])
```

Appending the byte `0x01` at index 16 adds 1 · 256^16 = 2^128,
because L_17 puts index 16 at the coefficient of 256^16. That is
why the byte `0x01`, and not a bit stuffed into the high end of
the last message byte, is the marker. The marker is a format
boundary: it says where the block's octets ended. A short final
block uses a smaller power, 2^120 or below, as the section says.
This lesson checks a full 16-byte block and does not run the
accumulator. The multiply-and-reduce loop is J10.

Section 2.5.2 prints the example. The lines used here:

```text
s as an octet string:
   01:03:80:8a:fb:0d:b2:fd:4a:bf:f6:af:41:49:f5:1b
s as a 128-bit number: 1bf54941aff6bf4afdb20dfb8a800301
r before clamping: 85:d6:be:78:57:55:6d:33:7f:44:52:fe:42:d5:06:a8
Clamped r as a number: 806d5400e52447c036d555408bed685
```

And the first message line of the dump, together with the
calculation's first block:

```text
000  43 72 79 70 74 6f 67 72 61 70 68 69 63 20 46 6f  Cryptographic Fo
Block = 6f4620636968706172676f7470797243
Block with 0x01 byte = 016f4620636968706172676f7470797243
```

The block numeral is the hex spelling of L_16 of those sixteen
bytes: high byte on the left of the numeral, which is the last
byte of the string, `0x6f`. The dump's first byte is `0x43`, the
ASCII code of `C`, and it is the least significant byte of the
number. The numeral therefore ends in `43`.

**Listing J4.13 — `poly_read.or`**

```orange
edition 2026;
module poly_read {
  spec block() -> Int { hex"43727970746f6772617068696320466f" as little Int }
  spec shown() -> Word[8]^16 {
    let n: Int = hex"43727970746f6772617068696320466f" as little Int;
    n as big Word[8]^16
  }
  spec with_bit() -> Word[8]^17 {
    let n: Int = (hex"43727970746f6772617068696320466f" ++ hex"01") as little Int;
    n as big Word[8]^17
  }
  spec s() -> Int { hex"0103808afb0db2fd4abff6af4149f51b" as little Int }
  spec clamped() -> Int {
    let r: Word[8]^16 = hex"85d6be7857556d337f4452fe42d506a8";
    let a: Word[8]^16 = r with [3] = r[3] & 15;
    let b: Word[8]^16 = a with [7] = a[7] & 15;
    let c: Word[8]^16 = b with [11] = b[11] & 15;
    let d: Word[8]^16 = c with [15] = c[15] & 15;
    let e: Word[8]^16 = d with [4] = d[4] & 252;
    let f: Word[8]^16 = e with [8] = e[8] & 252;
    let g: Word[8]^16 = f with [12] = f[12] & 252;
    g as little Int
  }
  test "the first block displays as the RFC hex numeral" {
    shown() == hex"6f4620636968706172676f7470797243"
  }
}
```

**Expected evaluation output:**

```text
poly_read::block: Int = 147908425225540690611047796896236663363
poly_read::shown: Word[8]^16 = [0x6f, 0x46, 0x20, 0x63, 0x69, 0x68, 0x70, 0x61, 0x72, 0x67, 0x6f, 0x74, 0x70, 0x79, 0x72, 0x43]
poly_read::with_bit: Word[8]^17 = [0x01, 0x6f, 0x46, 0x20, 0x63, 0x69, 0x68, 0x70, 0x61, 0x72, 0x67, 0x6f, 0x74, 0x70, 0x79, 0x72, 0x43]
poly_read::s: Int = 37162754436723567162214218529947779841
poly_read::clamped: Int = 10669302975710760130990824415874176645
```

**Test report:**

```text
test "the first block displays as the RFC hex numeral" ... ok
1 test: 1 passed, 0 failed
```

`shown` writes L_16 of the message bytes back out with B_16, which
is the hex numeral of that integer, high byte first. The bytes are
`6f 46 20 63 69 68 70 61 72 67 6f 74 70 79 72 43`, the block the
RFC prints, including the final `43` that is the first byte of the
dump. `with_bit` is the same integer plus 2^128, displayed in 17
bytes. The leading byte is `0x01`, and the rest are the block
numeral. That is the line “Block with 0x01 byte”.

`s` is L_16 of the octet string. Its hex numeral is
`1bf54941aff6bf4afdb20dfb8a800301`, the number the RFC prints. The
low two bytes of that numeral are `03 01`, which are the first two
octets `01:03` in reverse, as Proposition J4.4 requires.

`clamped` is L_16 after the seven byte masks. The integer is
10669302975710760130990824415874176645. In hex it is
`806d5400e52447c036d555408bed685`, which is the clamped number the
section prints, a 31-digit numeral because the high nibble is
`8` and no leading zero was written. The test of `shown` does not
cover `clamped`. A reader who skips the masks and compares L_16 of
the raw r bytes with that numeral will not get a match. The masks
are part of the format of r. They are not an endianness. J10
inherits both: little-endian number, then this clamp, then the
field prime 2^130 − 5, which §2.5 prints as
`3fffffffffffffffffffffffffffffffb`. Reducing the 17-byte block
modulo that prime does not change it, because the block with the
`0x01` byte is less than 2^129 and the prime is greater. That
comparison is one integer. It is not the MAC.

### J4.14 RFC 7748: the u-coordinate and the masked top bit

Section 5 encodes a u-coordinate as a byte array in little-endian
order such that

u[0] + 256·u[1] + 256^2·u[2] + … + 256^{n−1}·u[n−1]

is congruent to the value modulo p, and u[n−1] is minimal. The sum
is L_n. The sentence then says: when receiving such an array,
implementations of X25519, but not X448, MUST mask the most
significant bit in the final byte. The Python in the same section
is the definition of that mask for a width that is not a multiple
of 8. For 255 bits, `bits % 8` is 7, and the last byte is combined
with `(1 << 7) − 1`, which is 127. Bit 7 of the last byte is
cleared. The other bits of that byte stay. The decoding used here
is L_32 of the 32 bytes after that mask.

The same section decodes a scalar by a different mask, and the
difference matters. For X25519, set the three least significant
bits of the first byte and the most significant bit of the last to
zero, set the second most significant bit of the last byte to 1,
and decode as little-endian. The resulting integer has the form
2^254 plus eight times a value between 0 and 2^251 − 1 inclusive.
The Python is `k_list[0] &= 248`, `k_list[31] &= 127`,
`k_list[31] |= 64`, then the little-endian sum. This lesson does
not perform the scalar multiplication. The ladder is J20. The
masks are format boundaries on the way into that function, and
they are not the same mask.

Section 5.2 prints an X25519 vector. The u-coordinate and the
number, with the line break of the plain text removed and no digit
added or deleted, are:

```text
e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c
34426434033919594451155107781188821651316167215306631574996226621102155684838
```

The last byte of the coordinate is `0x4c`. Its most significant
bit is 0, because `0x4c` = 76 < 128. The mask that clears that bit
does not change this array. The printed number is therefore L_32
of the printed bytes. A coordinate whose last byte had that bit
set would be a different integer, larger by 2^255, and the mask
would remove exactly that power. Section 6.1 encodes the base
point as a byte with value 9 followed by 31 zero bytes. L_32 of
that string is the integer 9.

The first scalar of §5.2 is printed as

```text
a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4
```

and its number, again joining the broken line, is

```text
31029842492115040904895560451863089656472772604678260265531221036453811406496
```

The last byte of the scalar is `0xc4`, whose most significant bit
is 1. L_32 of the raw bytes is not the printed number. The printed
number is L_32 after the scalar mask. Comparing the raw
little-endian integer with the printed number fails. The failure
is not an endianness bug. The endianness is already little-endian
on both sides. The missing operation is the mask.

**Listing J4.14 — `x_coord.or`**

```orange
edition 2026;
module x_coord {
  spec u() -> Int {
    hex"e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c" as little Int
  }
  spec masked() -> Int {
    let u: Word[8]^32 = hex"e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c";
    (u with [31] = u[31] & 127) as little Int
  }
  spec raised() -> Int {
    let u: Word[8]^32 = hex"e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c";
    (u with [31] = u[31] | 128) as little Int
  }
  spec base() -> Int { (hex"09" ++ [0x00; 31]) as little Int }
  test "masking this vector does not change the integer" {
    masked() == u()
  }
  test "the base point byte string is the integer 9" {
    base() == 9
  }
}
```

**Expected evaluation output:**

```text
x_coord::u: Int = 34426434033919594451155107781188821651316167215306631574996226621102155684838
x_coord::masked: Int = 34426434033919594451155107781188821651316167215306631574996226621102155684838
x_coord::raised: Int = 92322478652577692162940600285532775577951159548126913594725018625058720504806
x_coord::base: Int = 9
```

**Test report:**

```text
test "masking this vector does not change the integer" ... ok
test "the base point byte string is the integer 9" ... ok
2 tests: 2 passed, 0 failed
```

`u` equals the printed number. `masked` equals `u` on this vector
only because bit 7 of `0x4c` is already clear. `raised` sets that
bit. The difference `raised − u` is 2^255. The mask `u[31] & 127`
applied to the raised array clears the bit again and returns `u`.
J20, receiving a 32-byte coordinate, masks before the little-endian
sum. Skipping the mask accepts an integer the section told the
receiver to reduce by clearing one bit.

**Listing J4.15 — `x_scalar_wrong.or`, intentionally failing**

```orange
edition 2026;
module x_scalar_wrong {
  spec raw() -> Int {
    hex"a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4" as little Int
  }
  test "the raw little-endian reading is the printed scalar" {
    raw() == 31029842492115040904895560451863089656472772604678260265531221036453811406496
  }
}
```

**Expected evaluation output:**

```text
x_scalar_wrong::raw: Int = 88925887110773138616681052956207043583107764937498542285260013040410376226469
```

**Test report:**

```text
test "the raw little-endian reading is the printed scalar" ... FAILED
    left:  88925887110773138616681052956207043583107764937498542285260013040410376226469
    right: 31029842492115040904895560451863089656472772604678260265531221036453811406496
1 test: 0 passed, 1 failed
```

Left is L_32 of the printed scalar bytes. Right is the number §5.2
prints beside them. They differ. The order is not the defect.

**Listing J4.16 — `x_scalar.or`**

```orange
edition 2026;
module x_scalar {
  spec clamped() -> Int {
    let k: Word[8]^32 = hex"a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4";
    let a: Word[8]^32 = k with [0] = k[0] & 248;
    let b: Word[8]^32 = a with [31] = a[31] & 127;
    (b with [31] = b[31] | 64) as little Int
  }
  test "RFC 7748 5.2 scalar number is the clamped decoding" {
    clamped() == 31029842492115040904895560451863089656472772604678260265531221036453811406496
  }
}
```

**Expected evaluation output:**

```text
x_scalar::clamped: Int = 31029842492115040904895560451863089656472772604678260265531221036453811406496
```

**Test report:**

```text
test "RFC 7748 5.2 scalar number is the clamped decoding" ... ok
1 test: 1 passed, 0 failed
```

The repair is the three assignments the section lists, then the
same `as little` the failing test already used. 248 is 256 − 8, so
`& 248` clears the three low bits of the first byte. 127 clears
bit 7 of the last byte. 64 sets bit 6. The passing test is a Match
on this one scalar. It is not X25519. Section 5 also says the
`cswap` used by the ladder should be independent of the swap
argument in its timing. This lesson does not implement `cswap` and
makes no timing claim.

### J4.15 FIPS 197: bytes, the State, and the column

Section 3.1 says a bit is 0 or 1, and a block is a sequence of 128
bits. Section 3.2 says the basic processing unit is the byte, a
sequence of eight bits. When the bits of a byte are denoted by an
indexed variable, the indices decrease from left to right:
`{b7 b6 b5 b4 b3 b2 b1 b0}`. The left hex digit of a byte is
b7 through b4. The example `{10100011}` is the hex byte `{a3}`.
That is the inside-the-byte convention of Chapter 2, stated again
for this standard. Bit 0 of a byte is the least significant bit.
It is not byte 0 of a block.

Section 3.3 numbers a sequence of 8k bits as r0 through r_{8k−1}
from the left, and sets byte a_j to the eight bits r_{8j} through
r_{8j+7}. For a block, a0 is the first byte and a15 is the last.
Table 2 of that section puts the two numberings on one line. The
bit index in the sequence increases to the right. The bit index
inside the byte decreases to the right. Both statements are in the
table. Neither one is byte order inside a 32-bit integer. The
table does not say “little-endian” or “big-endian”.

Section 3.4 copies the input into the State. The State is a
four-by-four array of bytes. The copy is

s[r, c] = in[r + 4c] for 0 ≤ r < 4 and 0 ≤ c < 4.

The output copy is the same formula reversed,
out[r + 4c] = s[r, c]. Index c is the column. The formula takes
four consecutive input bytes, starting at a multiple of 4, and
places them down a column. That is column-major order of the input
bytes. It is not row-major, which would have been s[r, c] = in[4r + c].

Figure 1 prints the input bytes across the top of each column and
the State with s0,0 at the upper left. Appendix B gives the bytes.
The input line is

```text
Input = 32 43 f6 a8 88 5a 30 8d 31 31 98 a2 e0 37 07 34
```

and the State at the start of the round diagram is the columns

```text
32 88 31 e0
43 5a 31 37
f6 30 98 07
a8 8d a2 34
```

Read down the first column: 32, 43, f6, a8, which are in0 through
in3. Read across the first row: 32, 88, 31, e0, which are in0,
in4, in8, in12. Equation (3.6) produces both readings.

Section 3.5 says a word is a sequence of four bytes, and a block
is four words. The four columns are four words. Appendix A prints
the key

```text
Key = 2b 7e 15 16 28 ae d2 a6 ab f7 15 88 09 cf 4f 3c
```

and the words `w0 = 2b7e1516`, `w1 = 28aed2a6`, `w2 = abf71588`,
`w3 = 09cf4f3c`. The first word's hex numeral is the first four
key bytes in the order printed. That is B_4 of those bytes. The
section does not use the phrase “big-endian” for this numeral.
The numeral is the evidence. A little-endian reading of
`2b 7e 15 16` is `0x16157e2b`, which Appendix A does not print.
J12 may treat a column as a sequence of four bytes in row order,
index 0 at the top, and if it packs those bytes into a `Word[32]`
for the notation of Appendix A it uses `as big`. Packing them with
`as little` produces a word the appendix does not display. This
lesson does not expand the key and does not run a round.

**Listing J4.17 — `aes_state.or`**

```orange
edition 2026;
module aes_state {
  spec columns_match() -> Bool {
    let inn: Word[8]^16 = hex"3243f6a8885a308d313198a2e0370734";
    let picture: Word[8]^16 = hex"328831e0435a3137f6309807a88da234";
    for c in 0..4 with ok: Bool = true {
      (for r in 0..4 with row: Bool = true {
        row && (inn[r + (4 * c)] == picture[(4 * r) + c])
      }) && ok
    }
  }
  spec w0() -> Word[32] { hex"2b7e1516" as big Word[32] }
  spec w0_little() -> Word[32] { hex"2b7e1516" as little Word[32] }
  test "Appendix B input occupies the state by equation 3.6" {
    columns_match() == true
  }
  test "Appendix A prints the first key word big-endian" {
    w0() == 0x2b7e1516
  }
}
```

`picture` is the State written row by row, left to right, from the
diagram above: row 0 is `32 88 31 e0`, row 1 is `43 5a 31 37`, and
so on. The inner comparison says the input byte at index r + 4c
equals the picture byte at index 4r + c. That is equation (3.6)
together with row-major storage of the figure. The loops run r
through 0, 1, 2, 3 and c through the same range. Every index
r + 4c and 4r + c falls between 0 and 15. The `Bool` is the
conjunction of the sixteen equalities.

**Expected evaluation output:**

```text
aes_state::columns_match: Bool = true
aes_state::w0: Word[32] = 0x2b7e1516
aes_state::w0_little: Word[32] = 0x16157e2b
```

**Test report:**

```text
test "Appendix B input occupies the state by equation 3.6" ... ok
test "Appendix A prints the first key word big-endian" ... ok
2 tests: 2 passed, 0 failed
```

**Proposition J4.10.** For the Appendix B input, equation (3.6)
places `0x32` at s[0, 0], `0x43` at s[1, 0], and `0x88` at
s[0, 1]. The word Appendix A prints as `w0` is B_4 of the first
four key bytes, not L_4 of those bytes.

*Proof.* in[0] = `0x32`, in[1] = `0x43`, in[4] = `0x88`, from the
input line read left to right. Equation (3.6) at (r, c) = (0, 0)
selects in[0]. At (1, 0) it selects in[1]. At (0, 1) it selects
in[0 + 4·1] = in[4]. The first four key bytes are `2b 7e 15 16`.
B_4 of them is the integer whose hex numeral is `2b7e1516`, which
is the printed w0. L_4 of them is `0x16157e2b`, by the same sum as
Proposition J4.9. The printed word is the first integer. □

**Listing J4.18 — `aes_wrong.or`, intentionally failing**

```orange
edition 2026;
module aes_wrong {
  test "Appendix A first key word loaded little-endian" {
    (hex"2b7e1516" as little Word[32]) == 0x2b7e1516
  }
}
```

**Test report:**

```text
test "Appendix A first key word loaded little-endian" ... FAILED
    left:  0x16157e2b
    right: 0x2b7e1516
1 test: 0 passed, 1 failed
```

The minimal repair is `as big`, the conversion Listing J4.17
already tested. The right-hand side stays the word the appendix
prints.

### J4.16 Bit order inside a byte, byte order inside a word

The two orders are now both on the page, and they answer different
questions.

Take the byte `{10100011}`, which §3.2 of FIPS 197 writes as
`{a3}`. Number the bits b7 down to b0 from the left, as that
section does. Then b0 = 1, b1 = 1, b5 = 1, and the other bits are
0. The integer is 1·2^0 + 1·2^1 + 1·2^5 = 163 = 0xa3. Bit 0 is the
coefficient of 2^0. Changing bit 0 changes the integer by 1.
Changing bit 7 changes it by 128.

Take the four bytes of the ChaCha key prefix `(0x00, 0x01, 0x02, 0x03)`.
Byte 0 of that string is `0x00`. It is the first byte. Under L_4
it is also the least significant byte, and the integer changes by
1 if that byte changes by 1. Under B_4 the same byte is the most
significant, and changing it by 1 changes the integer by 2^24.
“The low end” is not the name of an index until the function is
chosen.

FIPS 197 Table 2 makes the inside-the-byte direction explicit.
Along the sequence, bit indices increase to the right, and byte
indices increase to the right. Inside a byte, bit indices decrease
to the right. A reader who exports those bytes onto a little-endian
wire, as ChaCha does, has used decreasing bit numbers inside the
byte and increasing significance across bytes. Cohen's note calls
a mix of those directions an inconsistent order, and says the
chunk size then has to be agreed, because the parties can no
longer regroup bits without knowing where the bytes were. [J4S1]
AES stays consistent with its own hex numerals: the first byte of
a word is the high byte of the printed word. ChaCha stays
consistent with its sentence: the first byte of a word is the low
byte. Each document picked one chunk size, 8 bits for the byte and
32 bits for the word, and stated the order at the word. A program
that uses AES key bytes as a ChaCha key, or the reverse, without
restating the order, has changed the integer.

Silence is the remaining case. A sentence that numbers bytes and
never says which end is significant has not chosen L_k or B_k.
Assumption J4.7. FIPS 197 §3.4 is nearly that sentence: it places
bytes into the State and does not, in that section, pack a column
into an integer. Appendix A is the later place where the integer
appears, and it appears as a hex numeral with the first byte on
the left. A J12 listing that packs the column must cite that
numeral, or cite §3.2's left-to-right bits, and not cite §3.4
alone.

### J4.17 Length fields and padding are format boundaries

A format boundary is a rule that says where one field ends. The
rule is not “where the example happens to be short”.

FIPS 180-4 §5.1.1 has three boundaries in one block. The message
bits end at bit ℓ. The next bit is 1, which for “abc” falls at the
start of the fourth byte and makes that byte `0x80`. Then zero bits
fill through bit 447 of the 512-bit block. Bits 448 through 511
are the length, and they are the integer ℓ in the big-endian
64-bit convention of §3.1, not the number of padding bytes and not
the number of blocks. Listing J4.7 put that integer at 24.
Listing J4.8 showed the other order produces a different integer.
Moving the length to the front of the block, or writing it in the
low eight bytes of a longer padding, is a different format. SHA-512
uses a 128-bit length and a different congruence, 896 mod 1024, in
§5.1.2. This lesson does not pad a SHA-512 message. The existence
of the second rule is why “the length goes at the end” is not yet
a specification.

Poly1305's boundary is the extra `0x01` byte at the first index
past the octets of the block, which adds 2^128 for a full block.
Without that byte, L_16 of the sixteen message bytes is a different
integer, smaller by 2^128. The RFC prints both. The clamp on r is
another boundary: bits that are required to be clear are not part
of the accepted key element, even though they occupy places in the
16-byte string. Clearing them changes the integer whenever one of
those bits arrived set.

RFC 7748's boundary on a received X25519 coordinate is one bit, the
most significant bit of the last byte. On the §5.2 vector that bit
is already 0, so a test of that vector alone cannot show that the
mask does anything. Listing J4.14 set the bit and the integer grew
by 2^255. A test suite that only round-trips vectors whose top bit
is clear does not exercise the boundary. The scalar mask is three
operations, not one, and Listing J4.15 showed that omitting them
fails against the printed number even though the endianness is
right.

The Orange rejection in Listing J4.5 is the same idea inside the
language. Three bytes are not a 32-bit word. The compiler will not
choose a pad byte in order to make the widths match. The program
names the pad, or it converts a type whose width is the width it
has.

### J4.18 How a test vector is printed

A printed vector has a layout, and the layout is not the integer.

RFC 8439 §2.5.2 prints the message as a dump of three columns. The
first column is an offset: `000`, then `016`, then `032`. Those
are decimal counts of bytes from the start of the message. Sixteen
bytes occupy the first line, so the next line begins at byte 16,
written `016`. The second column is the bytes in index order, hex
digits, high nibble on the left inside each byte. The third column
is those same bytes as ASCII, cut to the width of the line:
`Cryptographic Fo`, then `rum Research Gro`, then `up`. The cut is
the line width. It is not a field boundary of the message. The
message is the 34 bytes `Cryptographic Forum Research Group`.

The offset column is not a length field and not an endianness. A
reader who adds the offset bytes into the message, or who reads
`016` as the hex integer 0x16 = 22 and skips 22 bytes, has invented
a format the dump does not have. The ChaCha key in §2.3.2 is
printed with colons, `00:01:02:03`, and no offsets. The colons
separate bytes. They are not a 16-bit word `0x0001`.

FIPS 180-4 prints words as eight hex digits, high digit on the
left, which Proposition J4.7 already tied to B_4. The NIST sample
prints `W[0] = 61626380` in that style. It does not print a dump
of the padded block. Reconstructing the bytes is the conversion,
not a second copy of the line.

FIPS 197 Appendix B prints the input as sixteen hex bytes separated
by spaces, in index order, and prints the State as a grid whose
columns are the words. The grid is equation (3.6) drawn. Reading
the grid left to right along a row does not recover the input
order. Reading it down a column does.

RFC 7748 prints coordinates as unbroken hex, 64 digits for 32
bytes, in index order, so the left-most pair is u[0], the least
significant byte. The decimal on the next line is L_32 after the
mask the section requires, not B_32 of the hex digits. For the
u-coordinate in §5.2 the mask was idle and the decimal matched
L_32. For the scalar it was not idle. A reader who feeds the hex
digits to `as big Int` gets the integer whose first byte is the
high byte. That integer is B_32, and §5 did not print it.

The rule for every one of these layouts is the same. Find the
sentence that says what the marks mean. Apply L_k or B_k, or the
column formula, as that sentence says. Compare the result with the
number or the word the document prints. Do this before a later
chapter compares a full construction. A construction that is right
on the wrong integers will match nothing, and the failure will
look like a round error.

### J4.19 The wrong order, the failure, and the repair

Outcome 5 is one habit, applied twice.

For the ChaCha key, Listing J4.11 loaded `00 01 02 03` with
`as big` and compared it with `0x03020100`. The test failed. Left
was `0x00010203`. Right was the word §2.3 names. Listing J4.12
changed `big` to `little` and passed. No other token changed.

For the SHA-256 word, Listing J4.9 loaded `61 62 63 80` with
`as little` and compared it with `0x61626380`. The test failed.
Left was `0x80636261`. Right was the word the sample prints.
Listing J4.7 changed the order to `big` and passed.

The two repairs move in opposite directions because the two
standards chose opposite functions. A single project-wide
“endianness setting” cannot satisfy both. The setting belongs to
the field. ChaCha's key, counter, nonce, and serialized block are
little-endian 32-bit words. SHA-256's message words and length are
big-endian. Poly1305's blocks and its s are little-endian numbers,
with a clamp on r before the number is used. X25519's coordinates
are little-endian, with one bit cleared on receipt. AES's printed
words put the first byte on the left, and its State is filled down
the columns.

In each failing report the right-hand side was the document's
value and the left-hand side was the wrong function. The minimal
repair edited the function. Editing the right-hand side to equal
the left-hand side produces a passing test of a value the document
does not contain. That edit is available, and it is the wrong
repair. N8 already separated a false expected word from a false
function. The same separation applies here, with the order as the
function.

### J4.20 What later chapters may assume

J5 may take B_4 on each 32-bit message word of a SHA-256 block, and
B_8 on the 64-bit length, as FIPS 180-4 §3.1 and §5.1.1. It may
take the padded “abc” block's first word to be `0x61626380` and its
length word-pair to end in `0x00000018`. It derives the schedule
and the compression function. This lesson did not.

J9 may take L_4 on each ChaCha20 key word, nonce word, and counter
word, and the inverse of L_4 when it serializes a word, as RFC
8439 §2.3. The constants are the four words Listing J4.10
computed. J9 does not inherit a big-endian load of the §2.3.2 key.

J10 may take L_16 on a Poly1305 block, then add 2^128 for a full
block by the extra byte `0x01`, and it may clamp r by the masks in
§2.5 before that integer is used. It derives the accumulator. This
lesson stopped at the integer.

J12 may copy an AES input into the State by s[r, c] = in[r + 4c],
and if it packs four column bytes into the hex numeral of Appendix
A it uses B_4. It derives the rounds. This lesson did not.

J20 may decode an X25519 u-coordinate by clearing bit 7 of the last
byte and then applying L_32, and it may decode a scalar by the
three masks of §5 and then L_32. It derives the ladder. This
lesson did not, and it made no timing claim about `cswap`.

None of these permissions is a proof of the construction, a
security claim, or a statement that the Match on one vector extends
to every input. A passing test remains a Match on the inputs it
writes. [J4C1] OEP-0017 remains in Review. The listings report the
binary. They do not accept the proposal. [J4T2]

## Exercises

Twelve exercises. The answers follow, each one worked. Predicting
the integer before you read the answer is the point of the first
six. A prediction you do not write down is not a prediction.

**Exercise J4.1 — Two bytes, both functions.** Take the byte string
`(0xab, 0xcd)`. Compute L_2 and B_2 by the sums in Definitions J4.3
and J4.4. Give each integer in decimal and in hex. Say which one
equals the hex numeral `0xabcd`, and why that equality is not a
reason to prefer that function.

**Exercise J4.2 — The inverse, one division at a time.** Start from
the integer 52651. Recover its little-endian bytes by Euclidean
division by 256, twice. Then reverse those bytes and read them
big-endian. Which proposition says the result equals 52651, and
which proposition says the same steps with the other order would
not return 52651?

**Exercise J4.3 — The sentence, then the bytes.** Quote the
sentence of FIPS 180-4 §3.1 that names the big-endian convention.
The sentence mentions bits. Explain, in the steps of Proposition
J4.7, why the first byte of a 32-bit word is nevertheless the
coefficient of 2^24. Apply that to the four bytes `61 62 63 80`
and name the sample line you have matched.

**Exercise J4.4 — A nonce word N12 already used.** The third nonce
group in RFC 8439 §2.3.2 is the bytes `00 00 00 4a`, in the order
printed. Compute L_4 and B_4. Which integer is word 14 of the
ChaCha state, and which listing in this lesson is the same
function on a different group?

**Exercise J4.5 — The second Poly1305 block.** The dump's second
line is the sixteen bytes
`72 75 6d 20 52 65 73 65 61 72 63 68 20 47 72 6f`.
Section 2.5.2 prints `Block = 6f7247206863726165736552206d7572`
for block 2. Show that this numeral is B_16 of the reversal of
those bytes, by identifying the first byte of the numeral and the
last byte of the numeral. Do not multiply the block by r. That
product is J10.

**Exercise J4.6 — Where the two orders agree.** Let s be
`(0x01, 0x02, 0x02, 0x01)`. Compute L_4(s) and B_4(s). Explain,
from Proposition J4.4 and from the terms of the two sums, why they
are equal. Explain why this equality does not retract Proposition
J4.6.

**Exercise J4.7 — One cell of the State.** For the Appendix B
input, which index of `in` is s[2, 3] under equation (3.6), and
which byte sits there? Which index is s[3, 2]? Why are those two
cells not the same byte?

**Exercise J4.8 — Bit 0 is not byte 0.** In the byte `0xa3`, under
the numbering FIPS 197 §3.2 gives, what is bit 0, and by how much
does the integer change if that bit flips? In the ChaCha key
prefix `(0x00, 0x01, 0x02, 0x03)`, what is byte 0, and by how much
does L_4 change if that byte becomes `0x01`? Why is a sentence
that says only “the low end” not enough to pick one of these two
answers?

**Exercise J4.9 — The repair that keeps the standard.** Listing
J4.8 failed. State the minimal repair. Then state an edit that
makes the `Bool` true and abandons §5.1.1. Which of the two is the
repair outcome 5 asks for, and why is the other one available?

**Exercise J4.10 — Read the dump, not the offset.** In the RFC
8439 §2.5.2 message dump, what integer does the offset `016`
denote, and is that integer a byte of the message? What is the
first byte of the message, as an integer, and which character is
it? Why is `016` not the hex integer 22?

**Exercise J4.11 — Right order, wrong integer.** Listing J4.15
failed with both sides little-endian. Name the three masks that
Listing J4.16 applies, and say which bits of which bytes they
change. Why would replacing `little` with `big` not be the repair?

**Exercise J4.12 — A string this lesson did not walk.** Take
`(0xff, 0x00)`. Compute L_2 and B_2. State the reversal relation
on this string in one equation. A document prints the two bytes
and never says which end is significant. Which of the five
outcomes have you not yet met for this string, and what sentence
is still missing? A passing comparison of L_2 with 255 is a Match
on this string. What does that Match not establish?

## Worked answers

**J4.1.** L_2 = 0xab + 0xcd · 256 = 171 + 205 · 256 = 171 + 52480
= 52651. In hex that is `0xcdab`, because the first byte is the
low byte and the hex numeral writes the high byte on the left.
B_2 = 0xab · 256 + 0xcd = 43776 + 205 = 43981, which is `0xabcd`.
The hex numeral `0xabcd` equals B_2 because a hex numeral is
itself a big-endian spelling of its digits, the convention of
§2.5 lifted from base 16 to base 256. That is a fact about how we
write integers. It is not a reason for a protocol to choose B_k.
ChaCha chooses L_4 for its words and still prints those words in
hex. The printing convention and the wire convention are two
functions. They coincide only when the field is big-endian or the
string is a palindrome.

**J4.2.** Divide 52651 by 256. 256 · 205 = 52480, and
52651 − 52480 = 171, so the remainder is 171 = 0xab and the
quotient is 205 = 0xcd. The next division of 205 by 256 has
remainder 205 and quotient 0. The little-endian bytes are
`(0xab, 0xcd)`. That is the inverse of L_2 given by Proposition
J4.2. Reversing them yields `(0xcd, 0xab)`. B_2 of that string is
0xcd · 256 + 0xab = 52480 + 171 = 52651. Proposition J4.4 says
L_2(s) = B_2(rev_2(s)), so the result had to be 52651. The other
order does not return the same integer on the way back: B_2 of
`(0xab, 0xcd)` is 43981, and L_2 of those same bytes is 52651.
Proposition J4.6 is the general fact that the two functions differ
whenever the length is greater than 1 and the string is the one
named there; this pair is a second witness, since 52651 ≠ 43981.
Storing with one order and loading with the other reverses bytes.
It is not the inverse.

**J4.3.** The sentence is: “Throughout this specification, the
“big-endian” convention is used when expressing both 32- and
64-bit words, so that within each word, the most significant bit
is stored in the left-most bit position.” The left-most bit is
the coefficient of 2^31 in a 32-bit word. The next seven bits are
the coefficients of 2^30 down through 2^24. Together they form a
byte multiplied by 2^24. That byte is the first byte of the word
in left-to-right order, so the word's integer is B_4 of its four
bytes. This is Proposition J4.7. The bytes `61 62 63 80` therefore
denote the integer whose hex numeral is `61626380`. The NIST
one-block sample prints that integer as `W[0] = 61626380`. Listing
J4.7 evaluated `as big` to the same word. The little-endian
reading `0x80636261` is not W[0].

**J4.4.** L_4(0x00, 0x00, 0x00, 0x4a) = 0x4a · 256^3 = 0x4a ·
16777216 = 1241513984, written `0x4a000000`. B_4 of the same bytes
is `0x0000004a` = 74. Section 2.3 takes the nonce's 32-bit groups
as little-endian integers, so word 14 of the state, the middle
nonce word in the §2.3.2 layout N12 printed, is `0x4a000000`.
Listing J4.10 applied the same function to the first nonce group
`00 00 00 09` and obtained `0x09000000`. Copying the printed
digits into the literal `0x0000004a` stores B_4. The state does
not contain that word.

**J4.5.** The last byte of the string is `0x6f`, and the first
byte is `0x72`. Proposition J4.4 says the hex numeral of L_16, high
byte on the left, is the bytes of the string reversed. The numeral
therefore begins with `6f` and ends with `72`. The RFC's block
line is `6f7247206863726165736552206d7572`. It begins with `6f`
and ends with `72`. The bytes between are the interior of the
string read from the right: `6f` then `72` then `47` then `20`,
which are the last four bytes of the dump line in reverse, and so
on back to `72`. The integer is 148137671992688356791019219269509281138.
This answer does not multiply by r, and it does not claim the tag.

**J4.6.** Both sums contain the terms 1, 2 · 256, 2 · 65536, and
1 · 16777216. L_4 assigns them to bytes from the left in that
order. B_4 assigns 1 · 16777216 to the first byte, 2 · 65536 to
the second, 2 · 256 to the third, and 1 to the last. The string
reads the same from either end, so the two assignments are the
same four terms. The common value is 16908801, which is
`0x01020201`. Proposition J4.4 says B_4(s) = L_4(rev_4(s)). Here
rev_4(s) = s, so the two functions agree on s. Proposition J4.6
says they are not the same function for k = 4, because they differ
on `(1, 0, 0, 0)`. Agreement on a palindrome is the case the
proposition did not claim. Listing J4.6's four `0xff` bytes are
the same case. A test that uses only palindromes cannot see the
order.

**J4.7.** Equation (3.6) is s[r, c] = in[r + 4c]. For r = 2 and
c = 3, the index is 2 + 12 = 14. The Appendix B input, in order,
is `32 43 f6 a8 88 5a 30 8d 31 31 98 a2 e0 37 07 34`. Index 14 is
`0x07`. For r = 3 and c = 2, the index is 3 + 8 = 11, and the byte
is `0xa2`. The two cells differ because column-major order and
row-major order select different input bytes at those coordinates.
The picture in Listing J4.17 stores the State row by row, so
s[2, 3] is the last byte of the third row of that picture, which
is the byte at picture index 4 · 2 + 3 = 11 of the picture string
`328831e0435a3137f6309807a88da234`, namely `0x07`. Same byte, other
array. The input index and the picture index are not the same
number.

**J4.8.** Section 3.2 writes the byte `{b7 b6 b5 b4 b3 b2 b1 b0}`
with the indices decreasing to the right. The hex byte `0xa3` is
`10100011`, so b0 = 1, b1 = 1, and b5 = 1. Bit 0 is the coefficient
of 2^0. Flipping it changes the integer by 1, from 163 to 162.
Byte 0 of the ChaCha prefix is the first byte, `0x00`. Under L_4
that byte is the coefficient of 256^0, so replacing it with `0x01`
changes L_4 by 1, from `0x03020100` to `0x03020101`. Under B_4 the
same replacement changes the integer by 2^24. “The low end” names
bit 0 if the speaker means inside a byte, and names byte 0 only if
the speaker has already chosen L_k. Assumption J4.7: the two
conventions are not interchangeable, and a sentence that is silent
has chosen neither.

**J4.9.** The minimal repair replaces `as little` with `as big` in
Listing J4.8. The right-hand side stays 24, which is ℓ for “abc”.
Listing J4.7 is that repaired comparison, and it passed. The edit
that abandons the standard replaces 24 with the left-hand value
`0x1800000000000000`. The `Bool` becomes true. The claim becomes
“the little-endian reading equals the little-endian reading”,
which is not §5.1.1. Outcome 5 asks for the repair that keeps the
document's value and changes the function. The other edit is
available because a test only checks the `Bool` you wrote. It does
not check that the right-hand side is the integer the section
defined. That check is yours, before you change the expected word.

**J4.10.** The offsets `000`, `016`, and `032` increase by 16,
which is the number of bytes on a full line of that dump. `016` is
the decimal integer 16, the index of the first byte on the second
line. It is not a byte of the message, and it is not a length
field. The first byte of the message is `0x43`, which is 67, the
ASCII code of `C`. The line's ASCII column begins `Cryptographic`.
Reading `016` as hexadecimal would give 0x16 = 22, and skipping 22
bytes would land inside the second line rather than at its start.
The dump's own successor, `032`, confirms the base: 16 + 16 = 32,
written with a leading zero as `032`. Hexadecimal 0x16 + 0x16 is
not 0x32. The column is decimal.

**J4.11.** Listing J4.16 clears the three least significant bits of
byte 0 by `& 248`, clears the most significant bit of byte 31 by
`& 127`, and sets the second most significant bit of byte 31 by
`| 64`. Those are the three operations §5 states for an X25519
scalar, in the order the paragraph states them, and then L_32.
The failing test already used L_32. Its left value was the raw
integer 88925887110773138616681052956207043583107764937498542285260013040410376226469.
The printed number is the clamped integer. Replacing `little` with
`big` would apply B_32 to the raw bytes, which §5 does not print
and which is not the clamped value either. The order was the part
that was already right. The boundary that was missing is the mask.

**J4.12.** L_2(0xff, 0x00) = 255 + 0 · 256 = 255. B_2(0xff, 0x00)
= 255 · 256 + 0 = 65280. The reversal of the string is
`(0x00, 0xff)`, and B_2 of that reversal is 0 · 256 + 255 = 255,
so L_2(s) = B_2(rev_2(s)). Proposition J4.4 is that equation for
every string; this is one string. A document that prints `ff 00`
and does not say which end is significant has not chosen a
function. Outcomes 1 and 2 are met: the bits inside `0xff` are the
Chapter 2 reading, the bytes are indexed from the left, and both
functions and the reversal relation are on the page. Outcome 3 is
not met, because there is no sentence to quote. Outcome 4 is only
partly met: bit order and byte order are distinct here, and there
is no length field in a two-byte string until a format says there
is one. Outcome 5 is not met, because there is no document value
to fail against and therefore no repair. The missing sentence is
the convention. A Match of L_2 with 255 shows that this evaluator
agreed with the sum on this string. It does not show that every
string decodes uniquely — that is Proposition J4.1 — and it does
not show that a standard chose L_2. It is not called verified.

```text
j4-exercise-ledger
ab-cd-little = 52651
ab-cd-big = 43981
palindrome = 16908801
nonce-4a = 1241513984
block2 = 148137671992688356791019219269509281138
state-cell-14 = 14
ff00-little = 255
ff00-big = 65280
top-bit = 57896044618658097711785492504343953926634992332820282019728792003956564819968
```

`ab-cd-little` is 171 + 205 · 256. `ab-cd-big` is 171 · 256 + 205.
`palindrome` is 1 + 2 · 256 + 2 · 65536 + 1 · 16777216.
`nonce-4a` is 74 · 16777216. `block2` is the little-endian integer
of the sixteen bytes in Exercise J4.5. `top-bit` is 2^255, the
difference Listing J4.14 produced by setting bit 7 of the last
coordinate byte.

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

**[J4S2] FIPS PUB 180-4.** National Institute of Standards and
Technology, *Secure Hash Standard (SHS)*, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. Wording of §3.1 and §5.1.1 was checked
on 2026-10-05 against the PDF retrieved from that DOI. The
big-endian sentence is item 2 of §3.1. The padding rule and the
“abc” bit length are §5.1.1. This lesson does not quote §6.2.2
and does not transcribe the compression function. The publication
is the one N13 recorded as its hash standard. This record is the
pin for the sentences quoted here.
This record's tag is [J4S2].

Source: <https://doi.org/10.6028/NIST.FIPS.180-4>

**[J4S3] NIST SHA-256 examples.** The one-block message sample,
input message “abc”, in the file the NIST examples page serves as
the SHA-256 illustration. Consulted on 2026-10-05. The lines used
here are `W[0] = 61626380` and `W[15] = 00000018` under “Block
Contents”. The round trace in that file is not copied. The file
is not a section of [J4S2].
This record's tag is [J4S3].

Source: <https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA256.pdf>

**[J4S4] Y. Nir and A. Langley.** “ChaCha20 and Poly1305 for IETF
Protocols,” RFC 8439, June 2018, Informational. The plain text was
checked on 2026-10-05 against the RFC Editor copy. The ChaCha20
input bullets and the little-endian serialization sentence are
§2.3. The Poly1305 block rule, the clamp, and the example numbers
are §2.5 and §2.5.2. Section 2.3.1 says that if pseudocode conflicts
with the textual explanation and the test vectors, the textual
explanation and the test vectors are normative. No erratum was
applied. The document is not an endorsement of Orange.
This record's tag is [J4S4].

Source: <https://www.rfc-editor.org/rfc/rfc8439.txt>

**[J4S5] A. Langley, M. Hamburg, and S. Turner.** “Elliptic Curves
for Security,” RFC 7748, January 2016, Informational. The plain
text was checked on 2026-10-05 against the RFC Editor copy. The
u-coordinate sum and the X25519 top-bit mask are §5. The scalar
masks are the following paragraph of §5. The vectors are §5.2.
The base-point encoding is §6.1. This lesson does not transcribe
the ladder. No erratum was applied.
This record's tag is [J4S5].

Source: <https://www.rfc-editor.org/rfc/rfc7748.txt>

**[J4S6] FIPS 197.** National Institute of Standards and
Technology, *Advanced Encryption Standard (AES)*, published
November 26, 2001, updated May 9, 2023, DOI
`10.6028/NIST.FIPS.197-upd1`. Wording of §§3.1–3.5 and the
Appendix A and Appendix B values quoted here was checked on
2026-10-05 against the PDF retrieved from that DOI. Equation (3.6)
is the input copy. This lesson does not transcribe the cipher
rounds or the key expansion. The update's change log is not a
byte-order change used here.
This record's tag is [J4S6].

Source: <https://doi.org/10.6028/NIST.FIPS.197-upd1>

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
J4.9 define the two functions and prove the bijections and the
reversal relation. Sections J4.10 through J4.20 quote the four
standards, check one conversion of each, and show the failing
order and the minimal repair. Twelve exercises have worked
answers. The integer ledgers are recomputed by
`tools/test_book_foundations.py`. The Orange listings are the
fenced programs in this file. They do not establish a
cryptographic security claim, they do not derive SHA-256, ChaCha20
past the word, Poly1305 past the integer, AES past the State copy,
or X25519 past the decoding, and they do not accept OEP-0017.
`orangec --version` was run on 2026-10-05 and printed the line
in §J4.1. That run does not establish a cryptographic security
claim, and it does not accept OEP-0017.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at
the owner's direction. Owner review is pending. No deployment
recommendation is made.
