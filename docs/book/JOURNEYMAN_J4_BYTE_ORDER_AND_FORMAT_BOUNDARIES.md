# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J4: Byte Order and Format Boundaries. Draft 2026-10-05. Revised 2026-10-09.

Continue from
[The Corpus as Acceptance Test](JOURNEYMAN_J3_THE_CORPUS_AS_ACCEPTANCE_TEST.md#j3-the-corpus-as-acceptance-test).
The corpus lesson runs pinned bytes as Orange tests. It does not pack
a byte into a word. The standards lesson keeps four ChaCha constant
words as a fixture and does not pack them either. N12 already showed,
as Proposition N12.4, that those words are the little-endian readings
of `expand 32-byte k`. This lesson takes the next question. Between
the hex a standard prints and the Orange value, where is the byte
order decided, and what does the other order do to the same bytes?
The reading habit is still
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
A passing test is still the Match of
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study).
J5 remains the study that derives the SHA-256 compression function
and its message schedule from FIPS 180-4 §6.2.2. This lesson cites
that section as an address. It does not transcribe the function.

This lesson is **J4**. The locked label is J4. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. The
manuscript chapter titled *From Core to Native Bytes* keeps that
title and that number. This lesson is not that chapter. When the
text says “§2.3” or “§5.1.1”, the number is a section of the edition
named in the same sentence.

## J4: Byte Order and Format Boundaries

> “We agree that the difference between sending eggs with the little- or the big-end first is trivial, but we insist that everyone must do it in the same way, to avoid anarchy. Since the difference is trivial we may choose either way, but a decision must be made.”
>
> — Danny Cohen, *On Holy Wars and a Plea for Peace*, IEN 137 (1 April 1980), SWIFT's POINT. [S15]

The paragraph does not choose an order. It says a choice has to be
made and kept. A standard is that choice for one field. The hex
digits of a word do not make the choice by themselves. The sentence
in a named edition does.

**The only-this-stack test.** You will trace one fixture the
standards lesson left unpacked, the sixteen ASCII bytes of
`expand 32-byte k`, from the hex RFC 8439 prints, through the bytes,
through the word, to an Orange value. You will read the same four
bytes both ways on the compiler this tree builds. You will predict
the integer a wrong-endianness load computes, then read the corpus
test that fails on it. You will check one format boundary that is
not byte order: the 64-bit length field of the padded message “abc”.
The compiler's version line is `orangec 0.0.1 (Orange edition 2026;
implemented slice S3t)`. Every listing is written for that compiler.
A form this compiler does not implement is marked Proposed, and this
lesson does not use one.

### J4.1 The decision, not the vibe

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can draw the boundary map for the ChaCha constant bytes the
   standards lesson keeps as a fixture. The stations are the hex the
   standard prints, the bytes, the word, and the Orange value. You
   can name where byte order is decided at each station. You can
   cite the convention by edition and section: FIPS PUB 180-4,
   August 2015, §3.1, big-endian; RFC 8439, June 2018, §2.3,
   little-endian; RFC 7748, January 2016, §5, little-endian.
2. You can read the same four bytes as a big-endian word and as a
   little-endian word in one Orange listing, with the conversion
   this compiler implements, and you can say which reading RFC 8439
   §2.3 prints.
3. You can predict the value a wrong-endianness load computes before
   you run it, and you can read the corpus `test` that fails on
   that value. The repair changes the order. It does not change the
   word the standard prints.
4. You can check a format boundary that is not byte order. FIPS
   180-4 §5.1.1 appends a 64-bit length field. For “abc” the field
   is the bit length 24, which the corpus lesson already pinned,
   not the byte length 3. An Orange test checks the field. A
   little-endian load of those same bytes fails a second corpus
   test. You predict `24·2^56` before you run it. The repair is
   `as big`.
5. You can compute the first message word and the eight length
   bytes that J5 will take for that padded block. The little-endian
   word is a wrong answer you can write down. This lesson does not
   build the message schedule and does not compress the block.
6. You can say what those tests do not establish.
   A Match is not called verified.
   The tests do not transcribe FIPS 180-4 §6.2.2.
   They are not a constant-time claim and not a performance claim.

Six assumptions bound the lesson. A later sentence that needs a
further fact names it there.

**Assumption J4.1 — The edition token is not the standard's date.**
Every Orange listing begins with `edition 2026;`. That token is the
edition this compiler requires. It is not June 2018, not August
2015, and not January 2016. Changing the token does not select a
different RFC or a different FIPS edition. [T8]

**Assumption J4.2 — A word has no byte order.** A value of type
`Word[32]` is an integer from 0 through 2^32 − 1. An array of
`Word[8]` is a sequence of bytes. Neither value stores a convention.
Byte order is a property of a sentence in an edition, or of a
conversion that names an order. The same bytes are two integers
until that sentence or that conversion is chosen.

**Assumption J4.3 — A test title is a string.** The compiler does
not open RFC 8439 and does not open FIPS 180-4. A title that names
a section becomes a pin only when a reader checks it against the
document. The standards lesson already used that gap for an edition.
This lesson uses it for an order.

**Assumption J4.4 — A Match is not a verification.** A silent check
means the source was well-formed under the checks this compiler
runs. An evaluation means the parameterless specs that ran produced
the printed values. A passing test means the `Bool` in that test
was true on the inputs the test wrote. None of those reads the
standard on its own, and none is called verified. [C2]

**Assumption J4.5 — This lesson does not transcribe SHA-256's
compression function or its message schedule, and it does not rerun
the ChaCha20 block function.** The compression function and the
schedule are FIPS 180-4 §6.2.2, and deriving them is J5. The block
function is the study N12 already ran. A section number may appear
as a pin. The functions do not appear on this page.

**Assumption J4.6 — The bit length and the byte length are different
integers.** FIPS 180-4 §5.1.1 names the length ℓ in bits. The corpus
lesson pins ℓ = 24 for the three-byte message “abc”. The byte length
of that message is 3. A field that holds 3 is not the field the
section appends.

### J4.2 From hex to an Orange value

The fixture is the one the standards lesson copied and did not pack.
RFC 8439, June 2018, §2.3, prints four constant words:

```text
0x61707865, 0x3320646e, 0x79622d32, 0x6b206574
```

Proposition N12.4 says those words are the little-endian 32-bit
readings of the sixteen ASCII bytes of `expand 32-byte k`, taken
four at a time from the left. This lesson does not re-prove the
four groups. It takes the first group, where a reader can still
swap the ends, and names the station at which the swap would be
legal.

The sixteen bytes, in the order a reader of the sentence meets the
letters, are the bytes N12 recorded:

```text
65 78 70 61  6e 64 20 33  32 2d 62 79  74 65 20 6b
```

The first four are `65 78 70 61`, the letters `e`, `x`, `p`, `a`.

The map has four stations. Byte order is not decided at all of them.

| Station | What is fixed | Where the byte order is decided |
| --- | --- | --- |
| Standard hex | The integer §2.3 prints, `0x61707865` | Not in the hex digits. The section chose which integer to print. |
| Bytes | The sequence `65 78 70 61` | Not yet. Character order is not a word convention. |
| Words | One integer from 0 through 2^32 − 1 | The edition's sentence, named below. |
| Orange value | The value of one conversion | The token `little` or `big`. The array has none. |

At the hex station the digits are a base-16 numeral. The left-most
digit is the most significant digit of that numeral. That is a rule
about how a numeral is written. It is not yet a rule about which
byte of `expa` is the low byte of the word. A reader who treats the
left-most pair `61` as the first byte of the string has left the
hex station and entered the word station without saying so. The
pair `61` is the letter `a`, the last of the four letters.

At the byte station the sequence is fixed by the characters. The
first byte is `0x65` because `e` is the first letter. Nothing at
this station says that `0x65` is the least significant byte of a
word, or the most significant. The station ends when the four bytes
have been named in order.

At the word station a sentence chooses. Three editions choose, and
they do not all choose the same function.

RFC 8439, June 2018, §2.3, says the next eight words of the ChaCha
state are taken from the 256-bit key by reading the bytes in
little-endian order, in 4-byte chunks. The same section prints the
four constant words as integers, and it says the result of the
block function is serialized by sequencing the words one-by-one in
little-endian order. The constants are not given a second rule.
Proposition N12.4 is the reading that connects the printed words to
the ASCII bytes under that order. This lesson uses that result for
the four words and computes the first word from the bytes, below,
so the place of the decision is visible.

FIPS PUB 180-4, August 2015, §3.1, item 2, says:

> Throughout this specification, the “big-endian” convention is used when expressing both 32- and 64-bit words, so that within each word, the most significant bit is stored in the left-most bit position.

The sentence is about bits inside a word. The left-most bit is the
most significant bit. Grouping those bits into bytes from the left,
eight bits at a time, puts the first byte in the most significant
place. Applied to `65 78 70 61`, that convention is the big-endian
integer, not the integer §2.3 of RFC 8439 prints. FIPS 180-4 is not
the document that defines the ChaCha constant. The citation is the
convention. It says what the other decision would do to these bytes.

RFC 7748, January 2016, §5, encodes a u-coordinate as an array of
bytes, u, in little-endian order, such that

```text
u[0] + 256·u[1] + 256^2·u[2] + ... + 256^(n-1)·u[n-1]
```

is congruent to the coordinate modulo the field prime, with the
last byte minimal. On four bytes that sum is the little-endian
reading. The section is about a coordinate, not about `expa`. The
same section requires implementations of X25519 to mask the most
significant bit of the final byte on receipt. This lesson does not
apply that mask, and it does not decode a coordinate. The sentence
used here is the sum.

Two of the three sentences choose the little-endian sum. FIPS 180-4
§3.1 does not. The disagreement is at the word station. It is not
in the hex digits, and it is not in the ASCII sequence.

At the Orange station the program chooses by a token. `hex"65787061"`
is the four bytes in character order. Assumption J4.2 says that
array has no byte order. `as little Word[32]` is the little-endian
decision. `as big Word[32]` is the big-endian decision. Until one
of those tokens is written, the program has not left the byte
station.

The powers used below are 256^1 = 256, 256^2 = 65536, and
256^3 = 16777216. Each byte is an integer from 0 through 255.
`0x65` = 101, `0x78` = 120, `0x70` = 112, and `0x61` = 97.

**Proposition J4.1.** The little-endian reading of the bytes
`65 78 70 61` is the integer `0x61707865`. The big-endian reading
of the same bytes is the integer `0x65787061`.

*Proof.* The little-endian sum takes the first byte as the
coefficient of 1:

`101 + 120·256 + 112·65536 + 97·16777216`.

`120·256 = 30720`. `112·65536 = 7340032`.
`97·16777216 = 1627389952`. Then
`101 + 30720 = 30821`,
`30821 + 7340032 = 7370853`, and
`7370853 + 1627389952 = 1634760805`.

The hex numeral `0x61707865` expands from the high digit. The
digits are `6`, `1`, `7`, `0`, `7`, `8`, `6`, `5`, and the places
are `16^7 = 268435456`, `16^6 = 16777216`, `16^5 = 1048576`,
`16^4 = 65536`, `16^3 = 4096`, `16^2 = 256`, `16^1 = 16`, and 1.

`6·268435456 = 1610612736`,

`1·16777216 = 16777216`,

`7·1048576 = 7340032`,

`0·65536 = 0`,

`7·4096 = 28672`,

`8·256 = 2048`,

`6·16 = 96`,

`5·1 = 5`.

The running total is `1610612736 + 16777216 = 1627389952`, then
`1627389952 + 7340032 = 1634729984`, then
`1634729984 + 28672 = 1634758656`, then
`1634758656 + 2048 = 1634760704`, then
`1634760704 + 96 = 1634760800`, then
`1634760800 + 5 = 1634760805`.

The byte sum and the hex numeral are the same integer. That is the
little-endian reading.

The big-endian sum takes the first byte as the coefficient of
16777216:

`101·16777216 + 120·65536 + 112·256 + 97`.

`101·16777216 = 1694498816`. `120·65536 = 7864320`.
`112·256 = 28672`. Then
`1694498816 + 7864320 = 1702363136`,
`1702363136 + 28672 = 1702391808`, and
`1702391808 + 97 = 1702391905`.

The hex numeral `0x65787061` has digits `6`, `5`, `7`, `8`, `7`,
`0`, `6`, `1`.

`6·268435456 = 1610612736`,

`5·16777216 = 83886080`,

`7·1048576 = 7340032`,

`8·65536 = 524288`,

`7·4096 = 28672`,

`0·256 = 0`,

`6·16 = 96`,

`1·1 = 1`.

The running total is `1610612736 + 83886080 = 1694498816`, then
`1694498816 + 7340032 = 1701838848`, then
`1701838848 + 524288 = 1702363136`, then
`1702363136 + 28672 = 1702391808`, then
`1702391808 + 96 = 1702391904`, then
`1702391904 + 1 = 1702391905`.

The byte sum and that hex numeral are the same integer. That is
the big-endian reading. □

RFC 8439 §2.3 prints the first of those two integers. It does not
print `0x65787061`. The hex display of the printed word, read from
the left, is the pairs `61 70 78 65`, the letters `a`, `p`, `x`,
`e`. That is the four letters in the other order. The word is not
the string `apxe`. N12 already recorded that warning. The map says
where it comes from: the hex station shows the high byte on the
left, and the little-endian decision put the last letter in the
high byte.

**Proposition J4.2.** The two integers in Proposition J4.1 are not
equal. Four bytes `0x0b` have one integer in both orders, and that
integer is `0x0b0b0b0b`.

*Proof.* `1634760805` and `1702391905` differ, so the two readings
of `65 78 70 61` differ. For four bytes `0x0b`, each coefficient
is 11. Swapping the ends does not change a sum in which every
coefficient is the same. The sum is

`11·(1 + 256 + 65536 + 16777216) = 11·16843009`.

`10·16843009 = 168430090`, and `168430090 + 16843009 = 185273099`.
The hex numeral `0x0b0b0b0b` is that integer: each byte `0x0b` is
the pair of digits `0b`, and four of them are the eight digits.
Both readings equal `185273099`. □

The corpus lesson's key byte from RFC 4231 §4.2 is `0x0b`. A word
made by repeating that byte is the palindrome case of Proposition
J4.2. A corpus test of `0x0b0b0b0b` passes under either order. It
cannot catch a swapped convention. The ChaCha bytes can, because
the two readings differ. That is why the witness in §J4.4 uses
this fixture and not the repeated key byte.

### J4.3 Both orders on this compiler

Listing J4.1 reads the bytes `65 78 70 61` with `as little` and
with `as big`. It also reads the whole constant string as four
little-endian words. The string conversion is the four groups
Proposition N12.4 names. The first spec is the little-endian half
of Proposition J4.1. The second spec is the big-endian half. The
two conversions are the Orange station of the map. No other token
in the listing chooses an order.

**Listing J4.1 — `orders.or`**

```orange
edition 2026;
module orders {
  spec first_little() -> Word[32] { hex"65787061" as little Word[32] }
  spec first_big() -> Word[32] { hex"65787061" as big Word[32] }
  spec constants() -> Word[32]^4 { "expand 32-byte k" as little Word[32]^4 }
  test "RFC 8439 2.3 first constant word" {
    first_little() == 0x61707865
  }
  test "same bytes read big-endian" {
    first_big() == 0x65787061
  }
  test "RFC 8439 2.3 four constant words" {
    constants() == [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
  }
}
```

`edition 2026;` is Assumption J4.1. It does not select June 2018.
`hex"65787061"` is the byte station: the characters of the hex
literal are the bytes in index order, not a word. `as little` and
`as big` are the word station written in the program. The third
test asks for the four words the RFC prints. The compiler does not
read the RFC. Assumption J4.3 is that check, and it lives in this
prose.

```sh
./compiler/target/debug/orangec check orders.or
./compiler/target/debug/orangec eval orders.or
./compiler/target/debug/orangec test orders.or
```

Check is silent. The status is 0. Standard error is empty. Silence
means the source was well-formed. It does not mean the little-endian
word was copied from June 2018 rather than computed from the bytes.

**Expected evaluation output:**

```text
orders::first_little: Word[32] = 0x61707865
orders::first_big: Word[32] = 0x65787061
orders::constants: Word[32]^4 = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574]
```

`eval` runs the parameterless specs. It does not run the tests.
`first_little` is the first integer of Proposition J4.1.
`first_big` is the second. `constants` is Proposition N12.4's four
words, checked as one array. The first entry of that array is
`first_little`. The listing does not also print the big-endian
reading of the other three groups. The same two sums do that work.
The lesson's new integer is the big-endian reading of the first
group, which the RFC does not print.

**Test report:**

```text
test "RFC 8439 2.3 first constant word" ... ok
test "same bytes read big-endian" ... ok
test "RFC 8439 2.3 four constant words" ... ok
3 tests: 3 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. Each `ok` means
the `Bool` in that test was true. The first test matches the word
§2.3 prints. The second matches the other reading of the same
bytes, which §2.3 does not print. A pass of the second test is not
a claim that FIPS 180-4 defines ChaCha. It is a Match of the
big-endian sum. The third test matches the four printed words under
the little-endian conversion. Passing it does not rerun the block
function. Assumption J4.5 stands.

The three tests are a corpus in the sense the corpus lesson uses:
each title names a vector, and `orangec test` counts the `Bool`s.
What they cover is the first word in both orders and the four
little-endian words. What they leave out is the key, the nonce, the
block count, the rounds, and the length field of a hash. A passing
report is a Match on the inputs the tests wrote. Do not call that
Match verified.

### J4.4 The wrong order fails

Listing J4.2 computes the big-endian reading and compares it with
the word RFC 8439 §2.3 prints. The title names that word. The
conversion does not. Before you run the test, write the two
integers the report will show.

The prediction is left `0x65787061` and right `0x61707865`.

Left is the value `first` denotes. The conversion is `as big`, so
left is the big-endian integer of Proposition J4.1. Right is the
integer written in the test, the word the section prints, which is
the little-endian integer of the same proposition. Proposition J4.2
says those integers differ, so the `Bool` is false. The wrong value
is `0x65787061`. It is the value the wrong order computes. It is
not the value the test should demand.

**Listing J4.2 — `wrong_order.or`, intentionally failing**

```orange
edition 2026;
module wrong_order {
  spec first() -> Word[32] { hex"65787061" as big Word[32] }
  test "RFC 8439 2.3 first constant word" {
    first() == 0x61707865
  }
}
```

The title is the first test of Listing J4.1. The bytes are the
same bytes. One token differs: `big` stands where `little` stood.

```sh
./compiler/target/debug/orangec check wrong_order.or
./compiler/target/debug/orangec eval wrong_order.or
./compiler/target/debug/orangec test wrong_order.or
```

Check is silent. The status is 0. Standard error is empty. The
false order is still well-formed. A silent check is not a true
test.

**Expected evaluation output:**

```text
wrong_order::first: Word[32] = 0x65787061
```

`eval` does not apply the test. The value is the prediction's left
integer. Changing the expected word in the test would not change
`first`.

**Test report:**

```text
test "RFC 8439 2.3 first constant word" ... FAILED
    left:  0x65787061
    right: 0x61707865
1 test: 0 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. There is no
`ORC` code. `left` is the value `first()` denotes. `right` is the
integer written in the test. Left is the prediction. Right is the
word §2.3 prints. The corpus test caught the swapped order because
the bytes are not the palindrome of Proposition J4.2.

**Proposition J4.3.** The `Bool` in Listing J4.2 is false because
`first()` denotes `0x65787061` and the test demands `0x61707865`.
Replacing `as big` with `as little` makes the `Bool` true. No
change to the expected word is required.

*Proof.* `hex"65787061" as big Word[32]` is the big-endian reading
in Proposition J4.1, the integer `0x65787061`. The test compares
that value with `0x61707865`. Proposition J4.2 says the two
integers differ, so `==` denotes false. Substituting `little` for
`big` selects the other reading in Proposition J4.1, which is the
integer the test writes, so `==` denotes true. The substitution is
one token. The expected word stays the word the RFC prints. □

The minimal repair is that token. Rewriting the right-hand side to
`0x65787061` would make the `Bool` true and would make the claim
false as a reading of §2.3. The report's right-hand side is the
standard's word. The left-hand side is the wrong function. N8
already separated a false expected word from a false function. The
same separation applies here, with the order as the function.

The repaired test is the first test of Listing J4.1. A pass after
that repair establishes the repaired `Bool`. It does not establish
the key schedule of ChaCha, the serialization of a block, or the
length field in the next section.

### J4.5 The length field

Byte order is one boundary. A length field is another. The field
says where the message bits end, and it says so by carrying an
integer the section defines. The integer is not “how the example
looks on the page.”

FIPS PUB 180-4, August 2015, §5.1.1, pads a message whose length is
ℓ bits. Append a 1 bit, then the smallest non-negative number k of
zero bits such that ℓ + 1 + k is congruent to 448 modulo 512, then
a 64-bit block equal to ℓ in the binary representation §3.1 fixes.
The publication's example is the 8-bit ASCII message “abc”. The
corpus lesson pins the bit length of that message at 24. This
lesson uses 24. It does not re-prove `8 × 3`.

**Proposition J4.4.** For ℓ = 24 the padding count is k = 423, so
the 64-bit length field occupies bits 448 through 511 of the first
512-bit block. Under the big-endian convention of §3.1 that field
is the eight bytes `00 00 00 00 00 00 00 18`, and the integer is
24. The little-endian reading of those eight bytes is
`24·2^56`, which is not 24. The byte length 3 is not the field.

*Proof.* `24 + 1 = 25`. `448 − 25 = 423`. Then
`25 + 423 = 448`, and 448 is congruent to 448 modulo 512. A
smaller non-negative k would leave a sum strictly below 448, which
is not congruent to 448 modulo 512. So k = 423. The bits that
follow begin at index 448 and there are 64 of them, through index
511.

Section 3.1 stores the most significant bit of a 64-bit word in
the left-most position. The unique base-256 writing of 24 with
eight digits puts 24 in the last digit and zero in the other
seven, because 24 is greater than or equal to 0 and strictly less
than 256. Those digits are the bytes
`00 00 00 00 00 00 00 18`. The big-endian reading of a string of
zero bytes followed by the byte 24 is 24.

The little-endian reading takes that last byte as the coefficient
of 256^7. `256^7 = (2^8)^7 = 2^56`. Compute the power in decades.
`2^10 = 1024`. `2^20 = 1048576`. `2^30 = 1073741824`.
`2^40 = 1099511627776`. `2^50 = 1125899906842624`.
`2^56 = 2^50·64 = 72057594037927936`. Then
`20·72057594037927936 = 1441151880758558720` and
`4·72057594037927936 = 288230376151711744`, so
`24·2^56 = 1729382256910270464`. That integer is greater than 24,
so it is not ℓ.

The byte length of “abc” is 3. `3` is not `24`. A 64-bit
big-endian field whose last byte is 3 denotes 3, not ℓ. □

The same section's example begins the padded message with the bits
of `a`, `b`, and `c`, then a 1 bit. The bytes of those letters are
`0x61`, `0x62`, and `0x63`. The next byte has its high bit set and
its other bits clear, which is `0x80`. The first four bytes of the
padded block are therefore `61 62 63 80`.

**Proposition J4.5.** The big-endian reading of the bytes
`61 62 63 80` is `0x61626380`. The little-endian reading of the
same bytes is `0x80636261`. The first message word J5 will take
from this block is the big-endian integer. The little-endian
integer is not that word.

*Proof.* `0x61` = 97, `0x62` = 98, `0x63` = 99, and `0x80` = 128.
The big-endian sum is

`97·16777216 + 98·65536 + 99·256 + 128`.

`97·16777216 = 1627389952`. `98·65536 = 6422528`.
`99·256 = 25344`. Then
`1627389952 + 6422528 = 1633812480`,
`1633812480 + 25344 = 1633837824`, and
`1633837824 + 128 = 1633837952`.

The hex numeral `0x61626380` expands as
`6·268435456 + 1·16777216 + 6·1048576 + 2·65536 + 6·4096 + 3·256 + 8·16`.
That is `1610612736 + 16777216 = 1627389952`, then
`1627389952 + 6291456 = 1633681408`, then
`1633681408 + 131072 = 1633812480`, then
`1633812480 + 24576 = 1633837056`, then
`1633837056 + 768 = 1633837824`, then
`1633837824 + 128 = 1633837952`. The two sums meet.

The little-endian sum puts 128 in the highest place:

`128·16777216 + 99·65536 + 98·256 + 97`.

`128·16777216 = 2147483648`. `99·65536 = 6488064`.
`98·256 = 25088`. Then
`2147483648 + 6488064 = 2153971712`,
`2153971712 + 25088 = 2153996800`, and
`2153996800 + 97 = 2153996897`.

The hex numeral `0x80636261` is that integer, because its high byte
is `0x80` and its low byte is `0x61`. `2153996897` is not
`1633837952`. Section 3.1's left-most bit is the most significant
bit, so the message word is the big-endian reading. The
little-endian reading is the other integer. J5 takes the
big-endian word. This lesson does not derive the schedule that
would consume it. □

Exercise J4.5 is the self-check. Write the first message word and
the eight length bytes before you read that answer. You can fail
it by writing `0x80636261`, which is the little-endian word
Proposition J4.5 just separated, or by writing a length field that
ends in the byte 3. Neither of those is the input J5 will take.

Listing J4.3 reads the eight length bytes little-endian and
compares that integer with 24. The title names the bit length the
corpus lesson pinned. The conversion does not. Before you run the
test, write the two integers the report will show.

The prediction is left `1729382256910270464` and right `24`.

Left is `24·2^56`, the little-endian reading Proposition J4.4
computed from `00 00 00 00 00 00 00 18`. Right is ℓ. The
proposition says those integers differ, so the `Bool` is false.
The wrong value is `1729382256910270464`. It is the value the
little-endian load computes. It is not the bit length.

**Listing J4.3 — `wrong_length.or`, intentionally failing**

```orange
edition 2026;
module wrong_length {
  spec length() -> Int { hex"0000000000000018" as little Int }
  test "FIPS 180-4 5.1.1 length field is the bit length" {
    length() == 24
  }
}
```

The bytes are the length field of Proposition J4.4. The expected
integer is ℓ. One token is the wrong order: `little` stands where
§3.1 requires `big`.

```sh
./compiler/target/debug/orangec check wrong_length.or
./compiler/target/debug/orangec eval wrong_length.or
./compiler/target/debug/orangec test wrong_length.or
```

Check is silent. The status is 0. Standard error is empty. The
false order is still well-formed. A silent check is not a true
test.

**Expected evaluation output:**

```text
wrong_length::length: Int = 1729382256910270464
```

`eval` does not apply the test. The value is the prediction's left
integer. Changing the expected integer in the test would not
change `length`.

**Test report:**

```text
test "FIPS 180-4 5.1.1 length field is the bit length" ... FAILED
    left:  1729382256910270464
    right: 24
1 test: 0 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. There is no
`ORC` code. `left` is the value `length()` denotes. `right` is the
integer written in the test. Left is the prediction. Right is ℓ.
The corpus test caught the swapped order of the length field.

Replacing `as little` with `as big` makes the `Bool` true. No
change to the expected integer is required. The repaired reading
is `length_be` in Listing J4.4. Rewriting the right-hand side to
`1729382256910270464` would make the `Bool` true and would make
the claim false as a reading of §5.1.1. The report's right-hand
side is the bit length. The left-hand side is the wrong function.

Listing J4.4 checks the integers on the compiler. `length_be` is
the field, the repair of Listing J4.3. `length_le` is the other
reading of the same eight bytes, the integer Listing J4.3's test
rejects. `word0` is the message word. `word0_le` is the wrong word
for J5. `byte_length_field` is the integer 3 written in the same
eight-byte shape. The tests demand 24 for the bit length and for
the field, and they demand the big-endian message word. They do
not demand that 3 equal 24.

**Listing J4.4 — `length_field.or`**

```orange
edition 2026;
module length_field {
  spec bits() -> Int { 3 * 8 }
  spec length_be() -> Int { hex"0000000000000018" as big Int }
  spec length_le() -> Int { hex"0000000000000018" as little Int }
  spec word0() -> Word[32] { hex"61626380" as big Word[32] }
  spec word0_le() -> Word[32] { hex"61626380" as little Word[32] }
  spec byte_length_field() -> Int { hex"0000000000000003" as big Int }
  test "FIPS 180-4 5.1.1 abc bit length" { bits() == 24 }
  test "FIPS 180-4 5.1.1 length field is the bit length" {
    length_be() == 24
  }
  test "FIPS 180-4 3.1 first message word" { word0() == 0x61626380 }
}
```

```sh
./compiler/target/debug/orangec check length_field.or
./compiler/target/debug/orangec eval length_field.or
./compiler/target/debug/orangec test length_field.or
```

Check is silent. The status is 0. Standard error is empty.

**Expected evaluation output:**

```text
length_field::bits: Int = 24
length_field::length_be: Int = 24
length_field::length_le: Int = 1729382256910270464
length_field::word0: Word[32] = 0x61626380
length_field::word0_le: Word[32] = 0x80636261
length_field::byte_length_field: Int = 3
```

`bits` is the corpus lesson's integer, computed as `3 * 8`.
`length_be` is Proposition J4.4's field. `length_le` is
`24·2^56`. `word0` and `word0_le` are the two integers of
Proposition J4.5. `byte_length_field` is 3. The evaluation prints
3 beside 24 so the two integers can be compared by eye. The
listing does not contain a round constant, a schedule, or a
compression step.

**Test report:**

```text
test "FIPS 180-4 5.1.1 abc bit length" ... ok
test "FIPS 180-4 5.1.1 length field is the bit length" ... ok
test "FIPS 180-4 3.1 first message word" ... ok
3 tests: 3 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. The second
test matches the length field to ℓ. It is the repair of Listing
J4.3: `as big` where that listing wrote `as little`. It does not
match the little-endian reading to ℓ, and it does not match the
byte length to ℓ. The little-endian failure is the fenced report
above, left `1729382256910270464` and right `24`, status 1.

What J5 may take from this page is narrow. It may take the first
message word to be `0x61626380`. It may take the length field to
be the eight bytes `00 00 00 00 00 00 00 18`, the integer 24, in
the big-endian convention of §3.1. It derives the schedule and the
compression function. This lesson did not.

The ledger keeps the integers this lesson prints, so a later edit
of a sum cannot leave a different total standing in the block.
The Python check recomputes each line from the arithmetic named
here. It does not fetch a file.

```text
j4-ledger
little-first = 1634760805
big-first = 1702391905
place-256 = 256
place-65536 = 65536
place-16777216 = 16777216
repeated-0b = 185273099
abc-bits = 24
abc-bytes = 3
length-le = 1729382256910270464
word0 = 1633837952
word0-le = 2153996897
zero-bits = 423
length-start = 448
```

`little-first` and `big-first` are Proposition J4.1.
`repeated-0b` is Proposition J4.2. `abc-bits` and `abc-bytes` are
Assumption J4.6. `length-le` is Proposition J4.4. `word0` and
`word0-le` are Proposition J4.5. `zero-bits` is k. `length-start`
is the bit index where the length field begins. The ledger answers
whether those printed integers are the integers the stated
arithmetic produces. It does not answer which file you retrieved.

### J4.6 Exercises

Eight exercises. Each one uses the map, a sum this lesson proved,
or the repair this lesson showed. Each answer is below. A guess
without the arithmetic is not an answer. Exercise J4.5 is the
self-check. Write it before you read its answer.

**Exercise J4.1 — Four stations.** For the first constant word,
name the four stations of the map. At which station is the byte
order decided by RFC 8439, June 2018, §2.3, and which integer does
that decision select? At which station would FIPS PUB 180-4,
August 2015, §3.1, select the other integer, if those bytes were
a word of that standard?

**Exercise J4.2 — The two sums.** Compute the little-endian and
big-endian readings of `65 78 70 61` from the powers 256, 65536,
and 16777216. Give both integers in decimal. Say which one RFC
8439 §2.3 prints.

**Exercise J4.3 — Predict the report.** Before using the report in
§J4.4, state left and right. Then state the one token that repairs
Listing J4.2, and state which side of the report must not be
edited.

**Exercise J4.4 — Eight bytes, not three.** Write the eight bytes
of the length field for “abc”. Write the eight bytes of the field
a reader builds from the byte length. Say which integer each field
denotes under §3.1, and which one is ℓ.

**Exercise J4.5 — The self-check J5 will need.** Write the first
message word of the padded “abc” block, as J5 will take it, and
write the eight length bytes. Then write the little-endian word a
reader produces by reusing the ChaCha order on `61 62 63 80`, and
say why that word is not the message word.
Do not write a message schedule.
Do not write a round constant. Do not compress the block.

**Exercise J4.6 — The third sentence, on these bytes.** Apply the
sum in RFC 7748, January 2016, §5, to the four bytes `65 78 70 61`,
with `u[0]` the first byte. Which integer of Proposition J4.1 do
you get? Why is that integer not, by this calculation, an X25519
coordinate?

**Exercise J4.7 — A witness the key byte is not.** The corpus
lesson uses the key byte `0x0b` from RFC 4231 §4.2. Form a 32-bit
word by repeating that byte four times. Compute both readings.
Say whether a corpus test of that word can catch a swapped order,
and why the bytes `65 78 70 61` can.

**Exercise J4.8 — What the passes leave out.** Listing J4.1 passes
three tests and Listing J4.4 passes three tests. Name two
functions those passes do not establish, and say why a Match is
not called verified.

#### Worked answers

**J4.1.** The stations are the standard hex, the bytes, the word,
and the Orange value. At the hex station, `0x61707865` is a
numeral. The left-most digit is the high digit of that numeral.
That rule does not say which letter of `expa` is the low byte. At
the byte station the sequence is `65 78 70 61`, character order,
and no word convention has been chosen. The byte order is decided
at the word station. RFC 8439 §2.3 chooses the little-endian
reading, the integer `0x61707865`. FIPS 180-4 §3.1, applied to the
same four bytes, chooses the big-endian reading, `0x65787061`,
because the left-most bit is the most significant bit. The Orange
station decides only when the program writes `little` or `big`.
RFC 7748 §5 chooses the little-endian sum, the same function as
RFC 8439 on these four bytes, for a coordinate this fixture is not.

**J4.2.** The little-endian sum is
`101 + 120·256 + 112·65536 + 97·16777216 = 1634760805`.
The big-endian sum is
`101·16777216 + 120·65536 + 112·256 + 97 = 1702391905`.
Proposition J4.1 is that pair. RFC 8439 §2.3 prints `1634760805`,
whose hex numeral is `0x61707865`. It does not print `1702391905`.

**J4.3.** The prediction is left `0x65787061` and right
`0x61707865`. The report in §J4.4 prints those two words, in that
order, and the status is 1. The repair is the token `big` becoming
`little`. The right-hand side stays `0x61707865`. Editing the
right-hand side to equal the left-hand side would pass a word the
section does not print. Proposition J4.3 is that repair.

**J4.4.** The length field is `00 00 00 00 00 00 00 18`. Under
§3.1 it denotes 24, which is ℓ. The byte-length field is
`00 00 00 00 00 00 00 03`. Under the same convention it denotes 3.
Assumption J4.6 says 3 is not 24. Proposition J4.4 is the same
split, and it adds the little-endian reading of the true field,
`1729382256910270464`, which is also not 24. Listing J4.3 loads
those bytes `as little` and demands 24. The report prints left
`1729382256910270464` and right `24`, and the status is 1.
Replacing `as little` with `as big` repairs it. The expected
integer stays 24.

**J4.5.** The first message word is `0x61626380`. The eight length
bytes are `00 00 00 00 00 00 00 18`. Proposition J4.5 is the word.
Proposition J4.4 is the field. The little-endian reading of
`61 62 63 80` is `0x80636261`. That integer is what the ChaCha
order would produce from these bytes. FIPS 180-4 §3.1 does not
choose it, so it is not the message word J5 will take. The
schedule and the compression function are §6.2.2. They are not
this answer. A reader who writes `0x80636261`, or who ends the
length field with the byte 3, has failed the self-check.

**J4.6.** With `u[0] = 0x65`, `u[1] = 0x78`, `u[2] = 0x70`, and
`u[3] = 0x61`, the sum in §5 is the little-endian sum, the integer
`0x61707865`. The section states that sum for a u-coordinate, and
it requires a mask on the top bit of a received X25519 coordinate.
These four bytes are the letters `expa`. The calculation did not
apply the mask, and it did not produce a coordinate. It shows that
the convention in §5 is the little-endian sum.

**J4.7.** Both readings of `0b 0b 0b 0b` equal `0x0b0b0b0b`, which
is `185273099`. Proposition J4.2 is that fact. A test that expects
`0x0b0b0b0b` passes if the load says `little` and passes if the
load says `big`. The swapped order is invisible. The bytes
`65 78 70 61` are not a palindrome. Proposition J4.2 says their
two readings differ, and Listing J4.2 is the corpus test that
fails when the load uses the wrong one.

**J4.8.** The passes do not establish the SHA-256 compression
function, and they do not establish its message schedule.
Assumption J4.5 withholds both. They also do not establish the
ChaCha20 block function. N12 already ran that function. This
lesson stopped at the words. A passing test is a Match of the
`Bool` it writes, on the inputs it writes. Assumption J4.4 says
that Match is not called verified. The passes are not a
constant-time claim and not a performance claim.

## Sources and epigraph record

The quotation is the borrowed sentence. The dates and identifiers
in the map are the publication's own cover lines, checked against
the files named here and against the pins the standards lesson
already recorded. Later sections quote clauses. They do not
replace this record.

**[S15] Danny Cohen.** *On Holy Wars and a Plea for Peace.*
IEN 137, USC/Information Sciences Institute, 1 April 1980. The
epigraph is the closing paragraph of the section headed
SWIFT's POINT. In the plain-text copy the paragraph is the last
paragraph of the note. The wording, with the line breaks of that
copy joined by single spaces, is the sentence pair quoted at the
opening. Wording was checked on 2026-10-05 against the IETF HTML
copy of IEN 137 and against the plain-text copy at the GWDG RFC
mirror. The two copies agree on this paragraph. No translation is
involved. The IEEE Computer reprint of 1981 rewrites the opening
sentence of the note. That reprint is not the source of this
epigraph. The paragraph is not an endorsement of Orange, and it
does not choose little-endian or big-endian. It requires that a
choice be made and kept.
This record's tag is [S15].

Source: <https://www.ietf.org/rfc/ien/ien137.html>

Plain-text copy consulted the same day:
<https://ftp3.gwdg.de/pub/rfc/ien/ien137.txt>

**[T11] Retrieved file.** On 2026-10-09 the lesson retrieved the
plain text of RFC 7748 from the RFC Editor. The SHA-256 digest of
that file is
`279ca0ecc5e92e2962e27b846986aeb74729d9dd34bd4a04a362f80dcb596ad3`.
The header prints January 2016, Informational, and the authors
A. Langley, M. Hamburg, and S. Turner. The “Authors' Addresses”
section prints Adam Langley, Mike Hamburg, and Sean Turner.
Section 5 is the little-endian sum quoted in §J4.2. The August
2015 pin of FIPS PUB 180-4, including the wording of §3.1 item 2
and of the “abc” padding example in §5.1.1, and the June 2018 pin
of RFC 8439 §2.3, are the pins the standards lesson recorded.
This lesson does not claim a new retrieval of those earlier files.
The standards lesson's copy of RFC 8439 is
<https://www.rfc-editor.org/rfc/rfc8439.txt>.
The August 2015 PDF of FIPS PUB 180-4 is
<https://doi.org/10.6028/NIST.FIPS.180-4>.
A digest is of the retrieved file. Another rendering can carry the
same sentence and a different digest. No translation is involved.
The file is not an endorsement of Orange, and the retrieval is
not a certification.
This record's tag is [T11].

Source: <https://www.rfc-editor.org/rfc/rfc7748.txt>

**[C5] Order surface.** The listings use `edition 2026`, `module`,
`spec`, `Int`, `Word[8]` bytes written as `hex"..."`, `Word[32]`,
`Word[32]^4`, `*`, `==`, `test`, `as little`, and `as big`. The
compiler this tree builds implements `as little` and `as big`.
No form in the listings is marked Proposed. On main, the OEP
index gives OEP-0017 the status Review, and
`docs/governance/oeps/OEP-0017-orange-2026-byte-order.md` records
`status: Review`. That file says the proposal is in Review. It
requires OEP-0016, which is also in review, and it accepts no
D-004 candidate. `docs/ORDER_2026.md` on main opens with the
status line “proposed S3n semantics under OEP-0017, in owner
review; not accepted.” The root README on main calls the same
conversions “Working; specification in review.” A listing reports
the binary. It does not accept OEP-0017. A passing test is a
Match on the inputs it writes. It is not a constant-time claim,
not a performance claim, and not a transcription of FIPS 180-4
§6.2.2.
This record's tag is [C5].

## Evidence boundary

J4 is a Journeyman lesson. The six outcomes in §J4.1 are the finish
line. The assumptions in that section bound them. The boundary map
in §J4.2 is the ChaCha constant bytes the standards lesson left
unpacked, read against FIPS 180-4 §3.1, RFC 8439 §2.3, and RFC 7748
§5. Listing J4.1 reads one group both ways. Listing J4.2 is the
wrong-endianness witness for the constant word. Listing J4.3 is
the wrong-endianness witness for the length field. Listing J4.4
checks the length field and the first message word. Eight
exercises have worked answers.
Exercise J4.5 is the self-check that sets up J5's message word and
length field. The integer ledger is recomputed by
`tools/test_book_foundations.py`. The Orange listings are the
fenced programs in this file.
`compiler/crates/orangec/tests/book_journeyman.rs` runs them.
Those checks do not establish a cryptographic security claim, they
do not derive FIPS 180-4 §6.2.2, they do not rerun the ChaCha20
block function, and they do not accept OEP-0017.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction, and revised into these six sections on
2026-10-09 at the owner's direction. Owner review is pending. No
deployment recommendation is made.
