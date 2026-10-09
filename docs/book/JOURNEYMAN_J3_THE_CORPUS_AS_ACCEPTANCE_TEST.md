# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J3: The Corpus as Acceptance Test. Draft 2026-10-09.

Continue from
[Standards as Versioned Inputs](JOURNEYMAN_J2_STANDARDS_AS_VERSIONED_INPUTS.md#j2-standards-as-versioned-inputs).
J2 pins an edition. Listing J2.1 computes one pinned integer.
Listing J2.2, in §J2.5, copies the expected value from the other
edition, and `orangec test` rejects that file. §J2.3 keeps N13's
pins of the NIST SHA-256 “abc” sample, under FIPS PUB 180-4, August
2015, and of RFC 4231 test case 1, §4.2, December 2005. This lesson
takes the next question. A file can hold several of those pins. You will call that
file a corpus, run it, and then meet two programs that pass a
smaller corpus and a third program that passes the corpus on this
page. The reading habit is still
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
A passing test is still the Match of
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study).
The pad byte and the short key are the ones
[Modules and Provenance](NOVICE_N13_MODULES_AND_PROVENANCE.md#n13-modules-and-provenance)
recorded. J5 remains the study that derives the SHA-256 compression
function and its message schedule from FIPS 180-4 §6.2.2. This
lesson cites that section as an address. It does not transcribe
the function. J4 remains the study of byte order. This lesson does
not pack bytes into a word.

This lesson is **J3**. The locked label is J3. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. The
manuscript chapter titled *The Corpus as Acceptance Test* keeps that
title and that number until a later renumber. When the text says
“§4.2” or “§5.1.1”, the number is a section of the edition named in
the same sentence.

## J3: The Corpus as Acceptance Test

> “An implementation that concurs with the results provided in this document should be interoperable with other similar implementations.”
>
> — M. Nystrom, RFC 4231, *Identifiers and Test Vectors for HMAC-SHA-224, HMAC-SHA-256, HMAC-SHA-384, and HMAC-SHA-512* (December 2005), §4.1. [S14]

The sentence says what concurrence is offered for. An implementation
that produces these results should interoperate with another
implementation that produces them. The sentence does not say that
the list of results is the definition of HMAC, and it does not say
that every input has been tried. The preceding sentence in §4.1
says the vectors were cross-verified by three independent
implementations. Three implementations can share one mistake on
every vector the list happens to contain. Agreement on the list is
the observation §4.1 records. The definition those implementations
were trying to meet is a different document.

**The only-this-stack test.** You will write a corpus as Orange
`test` declarations, run `orangec check`, `orangec eval`, and
`orangec test` on the compiler this tree builds, and read a report
that names each test. You will then change one expected byte to a
byte from a different vector and read the failure. You will then
run two other files that pass three of the tests and fail a fourth,
with two different computed bytes. The compiler's version line is
`orangec 0.0.1 (Orange edition 2026; implemented slice S3t)`. Every
listing is written for that compiler. A form this compiler does not
implement is marked Proposed, and this lesson does not use one.

### J3.1 Why a corpus is an acceptance test, not a spec

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can assemble a corpus of at least three vectors from at least
   two pinned sources. Each record names the source, the edition and
   its date, the section, the vector id, and what the vector covers.
2. You can run that corpus with `orangec test` on the compiler this
   tree builds, and you can read the report as a count of `Bool`s.
3. You can say what the corpus exercises and what it leaves out. A
   passing report is a Match on the inputs the tests wrote.
   A Match is not called verified.
4. You can diagnose one mislabelled vector, an expected byte copied
   from a different vector id, from the failing test, and you can
   repair that byte.
5. You can exhibit two wrong implementations that both pass one
   thin corpus, and one further pinned vector on which they fail
   with different computed bytes.
6. You can say why a file that passes the corpus on this page is
   still not the specification of the function. The witness is a
   third implementation that passes every test in the corpus and
   disagrees with the specification on a byte the corpus does not
   name.

The operation under test is one byte. RFC 2104, February 1997, §2,
defines ipad as the byte `0x36` repeated B times, for a hash whose
block is B bytes. N13 recorded that definition and proved, as
Proposition N13.1, that SHA-256's block is 64 bytes, so B is 64 for
that hash. This lesson does not re-prove the 512-bit sentence.
It uses the result. The function this corpus accepts is the inner
pad on a single byte:

```text
inner(k) = k XOR 0x36
```

The domain is one byte, so there are 256 possible inputs. The
corpus names a few of them. HMAC is the composition N13 wrote,
two calls to the hash. SHA-256 is the hash. `inner` is neither.
A test of `inner` does not become a test of the tag by sharing a
key byte with a vector that also prints a tag.

Eight assumptions bound the lesson. A later sentence that needs a
further fact names it there.

**Assumption J3.1 — The edition token is not the standard's date.**
Every Orange listing begins with `edition 2026;`. That token is the
edition this compiler requires. It is not February 1997, not
December 2005, and not August 2015. Changing the token does not
select a different RFC or a different FIPS edition. [T8]

**Assumption J3.2 — The operation is one byte.** `inner` takes one
`Word[8]` and returns `k XOR 0x36`. It does not take a message. It
does not call a hash. It does not place a key into a 64-byte block
except in the one listing that builds the RFC 4231 §4.2 key in
order to read two indices. XOR on a byte is the bitwise exclusive
or N13 recorded: one bit at a time, high bit on the left.

**Assumption J3.3 — A vector record has five fields.** The source
is the issuer and the document. The edition is that document's
date. For FIPS PUB 180-4, §J2.3 keeps the cover date August 2015.
The section is an address inside that edition. The vector id
is the case name the document prints, or the role of the byte when
the document is a definition rather than a numbered case. The fifth
field says what the comparison covers. A test title is a string.
It becomes those five fields only when a reader checks them against
the document. Listing J2.2 in §J2.5 already used that gap for an
edition. This lesson uses it for a vector id.

**Assumption J3.4 — A Match is not a verification.** A silent check
means the source was well-formed under the checks this compiler
runs. An evaluation means the parameterless specs that ran produced
the printed values. A passing test means the `Bool` in that test
was true on the inputs the test wrote. None of those is a proof for
every byte, none reads the standard on its own, and none is called
verified. [C2]

**Assumption J3.5 — This lesson does not transcribe SHA-256's
compression function or its message schedule, and it does not pack
bytes into a word.** The compression function and the schedule are
FIPS 180-4 §6.2.2, and deriving them is J5. Byte order and the
assembly of a word from printed bytes are J4. A section number of
FIPS 180-4 may appear as a pin. The functions do not appear on this
page. No listing shifts a byte into a `Word[32]`.

**Assumption J3.6 — The zero fill is not an empty message.** RFC
2104 §2 completes a key shorter than B with zero bytes, out to
length B. Those zeros are positions in the key block. An empty
message is a data string of length 0 passed to HMAC. The corpus
has a test for a zero key-byte. It has no test whose data string
is empty.

**Assumption J3.7 — A key longer than the block is a different
step.** RFC 2104 §2 hashes a key whose length exceeds B, and then
uses the hash output as the key. N13's keyed module does not own
that step. RFC 4231 §4.7, test case 6, prints a key of 131 bytes.
131 is greater than 64. This corpus does not hash that key, and a
test titled with §4.7 that still feeds a short byte has not
performed the step the section describes.

**Assumption J3.8 — Agreement on a finite set is not equality of
functions.** Two functions from bytes to bytes are the same
function when they agree on all 256 bytes. Agreement on three
bytes, or on four, leaves the other bytes unexamined. One byte
where they differ is enough to show they are not the same
function.

### J3.2 Building the corpus with provenance

The bytes below are computed from the definition, then pinned to
the document that prints the input. The expected result is not
copied from a tag. The tag, where the document prints one, is
named so you can see that this corpus does not compare it.

**Proposition J3.1.** `0x0b XOR 0x36 = 0x3d`, and
`0x00 XOR 0x36 = 0x36`.

*Proof.* Write the bytes with the high bit on the left. N14
already computed the first line on this key byte. The bits are
repeated here so the corpus does not depend on a page you have
not opened.

```text
0x0b = 00001011
0x36 = 00110110
XOR  = 00111101 = 0x3d
```

`0x00` has every bit 0, and `0 XOR b = b`, so
`0x00 XOR 0x36 = 0x36`. □

**Proposition J3.2.** `0x4a XOR 0x36 = 0x7c`.

*Proof.*

```text
0x4a = 01001010
0x36 = 00110110
XOR  = 01111100 = 0x7c
```

The same bytes as integers in 0 through 255 are 74 and 54.
74 + 54 = 128, which is `0x80`. Exclusive or is not addition.
`inner` does not add. □

**Proposition J3.3.** `0xaa XOR 0x36 = 0x9c`.

*Proof.*

```text
0xaa = 10101010
0x36 = 00110110
XOR  = 10011100 = 0x9c
```

The high bit of `0xaa` is 1. The high bit of `0x0b`, of `0x00`,
and of `0x4a` is 0. □

**Proposition J3.4.** `0x9c XOR 0x01 = 0x9d`, and `0x9d` differs
from `0x9c` in the low bit. Also `0x01 XOR 0x36 = 0x37`.

*Proof.*

```text
0x9c = 10011100
0x01 = 00000001
XOR  = 10011101 = 0x9d
```

```text
0x01 = 00000001
0x36 = 00110110
XOR  = 00110111 = 0x37
```

`0x37` differs from `0x00`. □

**Proposition J3.5.** The bit length of the three-byte message
“abc” is 24.

*Proof.* N13 records FIPS 180-4 §5.1.1 as padding the 8-bit ASCII
message “abc”, whose bit length is `8 × 3`. `8 × 3 = 24`. The
section prints a padding diagram. N13 records that it does not
print the digest. □

The corpus is six vectors. Three documents supply them. RFC 4231
is December 2005. RFC 2104 is February 1997. FIPS PUB 180-4 is
August 2015, the cover date §J2.3 keeps for that publication. The digest
of “abc” that N13 copied from NIST's examples file is not one of
the six. That file is not a section of FIPS 180-4, and this
operation does not produce a 32-byte digest.

| Id | Source | Edition | Section | Vector id | What it covers |
| --- | --- | --- | --- | --- | --- |
| C1 | RFC 4231 | December 2005 | §4.2 | HMAC-SHA-256 test case 1, the key byte | `0x0b`, the byte repeated in that 20-byte key, under the inner pad |
| C2 | RFC 4231 | December 2005 | §4.3 | HMAC-SHA-256 test case 2, the first key byte | `0x4a`, the first byte of the key printed `4a656665` |
| C3 | RFC 2104 | February 1997 | §2, step (1) | Zero fill past the RFC 4231 §4.2 key | `0x00`, a byte of the block the 20-byte key does not occupy |
| C4 | RFC 4231 | December 2005 | §4.2 | The same key, index 19 and index 20 | The last key byte and the first fill byte inside one 64-byte block |
| C5 | FIPS PUB 180-4 | August 2015 | §5.1.1 | The one-block sample “abc”, bit length | The integer 24 |
| C6 | RFC 4231 | December 2005 | §4.4 | HMAC-SHA-256 test case 3, the key byte | `0xaa`, a key byte whose high bit is 1 |

That is six vectors and three sources. The finish line asks for
three vectors and two sources. C1, C2, and C5 already meet it.
C3, C4, and C6 are here because the coverage argument needs the
zero fill, the index where the key ends, and a byte with the high
bit set. Dropping them would still be a corpus. It would be a
thinner one, which is what §J3.6 uses on purpose.

The inputs are read off the pages as follows. The pages are the
ones N13 named, plus §4.4 of the same RFC 4231 file, read again
for this lesson on 2026-10-09. [T10]

RFC 4231 §4.2 prints the test case 1 key as hex `0b` repeated, with
the parenthetical `(20 bytes)`. Sixteen bytes on the first hex line
and four on the second are 20. The data line is a different field.
C1 uses the key byte only. §J2.3 records this same case, test case
1 of §4.2, December 2005, and points at the HMAC-SHA-256 tag N13
compares. §J2.3 does not rebuild the pads. C1 does not compare the
tag either. It compares the inner pad of the key byte §J2.3 names
as twenty bytes `0b`.

RFC 4231 §4.3 prints the test case 2 key as `4a656665`, with the
note `("Jefe")`. Eight hex digits are four bytes. The first two
digits are `4a`, so the first byte is `0x4a`. The section heading
says the key is shorter than the HMAC output. The key is 4 bytes
and the SHA-256 tag is 32 bytes. C2 uses the first key byte only.

RFC 2104 §2, as N13 recorded it, completes a key shorter than B
with zero bytes. The §4.2 key has length 20 and B is 64, so the
bytes at indices 20 through 63 are `0x00` before the pad XOR.
C3 feeds `inner` the byte `0x00`. That is one of those positions,
not a scan of all 44 of them. `64 - 20 = 44`.

C4 builds the 20-byte key and reads index 19 and index 20. The
loop `0..20` writes indices 0 through 19. Index 19 receives
`0x0b`. Index 20 is outside that loop, so it stays the fill
`0x00`. Proposition J3.1 then gives `inner` at those two indices.

FIPS 180-4 §5.1.1, August 2015, is the padding section N13 cited
for “abc”. C5 compares `3 * 8` with 24. It does not compare a
digest. §J2.3 states the same split: §5.1.1 of the August 2015
text gives the bit length `8 × 3 = 24` and does not print the
digest, and a card that cites §5.1.1 as the source of the digest
has named the wrong section. C5 checks the bit length. It does not
check the digest. RFC 4231's normative reference for the hash is
FIPS 180-2, August 2002, with Change Notice 1 dated February 2004.
§J2.3 records that reference, and it records that a passing Match
of the digest does not choose August 2015 over August 2002. C5
does not make that choice either. The bit length is what §5.1.1
of the August 2015 text states. It is not a claim that RFC 4231
names the 2015 edition.

RFC 4231 §4.4 is test case 3. The heading says the case uses a
combined length of key and data larger than 64 bytes, and it names
64 as the block size of SHA-224 and SHA-256. The key hex is twenty
bytes of `aa`: 32 hex digits on the first line and 8 on the second.
`32 / 2 + 8 / 2 = 20`. The plain-text file prints `Key` and then
the hex, without an equals sign on that line. The other test cases
print `Key =`. This lesson does not insert the missing sign. The
parenthetical `(20 bytes)` and the hex digits are the record. The
data field is fifty bytes of `dd`: three lines of 16 bytes and a
final line of 2. `16 * 3 + 2 = 50`. The combined length is
`20 + 50 = 70`, and `70` is greater than `64`, which is what the
heading asserts. C6 uses the key byte `0xaa` only. It does not
use the data, and it does not check the sum 70. The HMAC-SHA-256
tag printed in §4.4 is not an expected value in this corpus.

The integers the proofs use are collected here so a later edit can
be checked against the arithmetic rather than against a memory of
it.

```text
j3-ledger
inner-00 = 54
inner-0b = 61
inner-4a = 124
inner-aa = 156
flip-aa = 157
spec-01 = 55
abc-bits = 24
key-len = 20
block = 64
fill-count = 44
combined = 70
long-key = 131
domain = 256
thin = 3
full-bytes = 4
high-half = 128
untested-thin = 253
untested-full = 252
```

`inner-00` is `0x36`. `inner-0b` is `0x3d`. `inner-4a` is `0x7c`.
`inner-aa` is `0x9c`. `flip-aa` is `0x9d`. `spec-01` is `0x37`.
`high-half` is the number of bytes whose high bit is 1, which is
128, half of 256. `full-bytes` is the number of distinct inputs
C1, C2, C3, and C6 pass to `inner`: `0x0b`, `0x4a`, `0x00`, and
`0xaa`. C4 repeats `0x0b` and `0x00`. C5 does not call `inner`.

### J3.3 Running it under orangec test

Listing J3.1 is the corpus. `inner` is the definition.
`case2_inner`, `case3_inner`, and `zero_inner` are parameterless
names for three of the bytes, so `eval` can print them.
`abc_bits` is Proposition J3.5. `case1_key` is the §4.2 key block,
the same shape N14 used for twenty bytes of `0x0b`. The six tests
are C1 through C6. Each expected byte is the proposition named
above, except C5, whose expected integer is 24.

**Listing J3.1 — `corpus.or`**

```orange
edition 2026;
module corpus {
  spec inner(k: Word[8]) -> Word[8] { k ^ 0x36 }
  spec case2_inner() -> Word[8] { inner(0x4a) }
  spec case3_inner() -> Word[8] { inner(0xaa) }
  spec zero_inner() -> Word[8] { inner(0x00) }
  spec abc_bits() -> Int { 3 * 8 }
  spec case1_key() -> Word[8]^64 {
    for i in 0..20 with b: Word[8]^64 = [0; 64] { b with [i] = 0x0b }
  }
  test "RFC 4231 4.2 test case 1 key byte" { inner(0x0b) == 0x3d }
  test "RFC 4231 4.3 test case 2 key byte J" { inner(0x4a) == 0x7c }
  test "RFC 2104 2 zero fill past the 4.2 key" { inner(0x00) == 0x36 }
  test "RFC 4231 4.2 key ends at index 19" {
    let key: Word[8]^64 = case1_key();
    (inner(key[19]) == 0x3d) && (inner(key[20]) == 0x36)
  }
  test "FIPS 180-4 5.1.1 abc bit length" { abc_bits() == 24 }
  test "RFC 4231 4.4 test case 3 key byte" { inner(0xaa) == 0x9c }
}
```

```sh
./compiler/target/debug/orangec check corpus.or
./compiler/target/debug/orangec eval corpus.or
./compiler/target/debug/orangec test corpus.or
```

Check is silent. The status is 0. Silence means the source was
well-formed. It does not mean the expected bytes were copied from
December 2005.

**Expected evaluation output:**

```text
corpus::case2_inner: Word[8] = 0x7c
corpus::case3_inner: Word[8] = 0x9c
corpus::zero_inner: Word[8] = 0x36
corpus::abc_bits: Int = 24
corpus::case1_key: Word[8]^64 = [0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

`eval` runs the parameterless specs. It does not run the tests.
The five lines are Proposition J3.2, Proposition J3.3, Proposition
J3.1's zero byte, Proposition J3.5, and the key block. Count the
`0x0b` entries in the block. There are twenty. Every later entry
is `0x00`.

**Test report:**

```text
test "RFC 4231 4.2 test case 1 key byte" ... ok
test "RFC 4231 4.3 test case 2 key byte J" ... ok
test "RFC 2104 2 zero fill past the 4.2 key" ... ok
test "RFC 4231 4.2 key ends at index 19" ... ok
test "FIPS 180-4 5.1.1 abc bit length" ... ok
test "RFC 4231 4.4 test case 3 key byte" ... ok
6 tests: 6 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. Each `ok`
means the `Bool` in that test was true. Six tests passed. The
report does not say that the titles were looked up in a document.
A title is not a vector record until you check the five fields.
Assumption J3.3 is that check, and it lives in §J3.2, not in the
title.

What this passing report establishes is narrow. On this compiler,
`inner(0x0b)` denotes `0x3d`, `inner(0x4a)` denotes `0x7c`,
`inner(0x00)` denotes `0x36`, `inner(0xaa)` denotes `0x9c`, the
two indices of this one key agree with Proposition J3.1, and
`abc_bits` denotes 24. Those are the values §J3.2 pinned. The
report does not establish HMAC, it does not establish SHA-256, and
it does not establish `inner` at any byte the six tests do not
write.

### J3.4 The coverage argument

Read the six tests against the five fields, and write down the
inputs that actually occur.

The lengths that occur are these. C2's key byte comes from a key
of length 4, which is shorter than the 32-byte SHA-256 tag, as the
§4.3 heading says, and shorter than the 64-byte block. C1's key
byte comes from a key of length 20, still shorter than the block.
C3 is the zero byte those 20 bytes leave behind. C5 is a message
length of 3 bytes, recorded as 24 bits. No test uses a key of
length 64, which would leave no zero fill. No test uses a key of
length 65 or of length 131.

The block boundary that occurs is C4. Index 19 is the last byte
the 20-byte key writes. Index 20 is the first byte it does not
write. That boundary sits inside one 64-byte block. It is not the
boundary FIPS 180-4 §5.1.1 draws when a message crosses from one
hash block into the next, and it is not the boundary at 55 bytes
that N13 recorded as the limit of its `hash`. C4 does not mention
either of those.

The zero fill that occurs is C3, one byte `0x00`, and the second
index of C4. Assumption J3.6 stands: this is empty key material
inside the block. It is not an empty HMAC message. RFC 4231's
seven test cases, as the December 2005 file prints them, do not
include a data field of length 0. This corpus does not add one.

The byte with the high bit set is C6, `0xaa`. Proposition J3.3
records that bit. §4.4's heading is about a combined length of 70,
which is larger than 64. C6 does not add 20 and 50. A reader who
treats the passing C6 line as a check of that heading has used the
title and skipped the `Bool`. The `Bool` is one XOR.

What the corpus leaves out, stated so it can be demanded later:

- The HMAC-SHA-256 tags printed in RFC 4231 §4.2, §4.3, and §4.4.
  N13 pinned the first two. This corpus does not recompute them
  and does not compare them.
- The SHA-256 digest of “abc”. N13 pinned it to NIST's examples
  file. C5 checks the bit length in FIPS 180-4 §5.1.1. The digest
  is a different value, from a different file, of a function this
  page does not contain.
- FIPS 180-4 §6.2.2, the compression function and the message
  schedule. That derivation is J5.
- A key longer than the block. RFC 4231 §4.7 prints 131 bytes of
  `aa`. The heading there compares 131 with 128, the block size it
  names for SHA-384 and SHA-512. For SHA-256 the block is 64, and
  131 exceeds 64 as well. Assumption J3.7 says the required step
  is to hash the key. No test hashes a key.
- An empty message, a key of length 64, and a key of length 65.
- The other bytes of the key “Jefe”, which are `0x65`, `0x66`, and
  `0x65`. C2 names the first byte only.
- Byte order. No listing builds a `Word[32]`.
- Constant time. Nothing in the report speaks about it.
- The 252 byte values `inner` can take that C1, C2, C3, and C6 do
  not name.

A report of six passes counts six `Bool`s. Assumption J3.4 still
applies to each of them, and it applies to the count. Do not call
the count verified.

The count also fails to determine the function. Listing J3.2
implements `inner` as a table of the four bytes the corpus feeds
it, and returns `0x00` for every other byte. The six tests are the
six tests of Listing J3.1, with the same titles and the same
expected values. `off` asks for `inner(0x01)`, which none of the
six tests asks for.

**Listing J3.2 — `wrong_rest.or`**

```orange
edition 2026;
module wrong_rest {
  spec inner(k: Word[8]) -> Word[8] {
    if k == 0x00 { 0x36 } else {
      if k == 0x0b { 0x3d } else {
        if k == 0x4a { 0x7c } else { if k == 0xaa { 0x9c } else { 0x00 } }
      }
    }
  }
  spec off() -> Word[8] { inner(0x01) }
  spec abc_bits() -> Int { 3 * 8 }
  spec case1_key() -> Word[8]^64 {
    for i in 0..20 with b: Word[8]^64 = [0; 64] { b with [i] = 0x0b }
  }
  test "RFC 4231 4.2 test case 1 key byte" { inner(0x0b) == 0x3d }
  test "RFC 4231 4.3 test case 2 key byte J" { inner(0x4a) == 0x7c }
  test "RFC 2104 2 zero fill past the 4.2 key" { inner(0x00) == 0x36 }
  test "RFC 4231 4.2 key ends at index 19" {
    let key: Word[8]^64 = case1_key();
    (inner(key[19]) == 0x3d) && (inner(key[20]) == 0x36)
  }
  test "FIPS 180-4 5.1.1 abc bit length" { abc_bits() == 24 }
  test "RFC 4231 4.4 test case 3 key byte" { inner(0xaa) == 0x9c }
}
```

```sh
./compiler/target/debug/orangec check wrong_rest.or
./compiler/target/debug/orangec eval wrong_rest.or
./compiler/target/debug/orangec test wrong_rest.or
```

Check is silent. The status is 0.

**Expected evaluation output:**

```text
wrong_rest::off: Word[8] = 0x00
wrong_rest::abc_bits: Int = 24
wrong_rest::case1_key: Word[8]^64 = [0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

`off` denotes `0x00`. Proposition J3.4 says the specification
`k XOR 0x36` denotes `0x37` at `0x01`. The evaluation printed the
table's value. It did not print `0x37`, because this file does not
compute that XOR for a byte outside the table.

**Test report:**

The report is the same six lines Listing J3.1 printed, including
the closing line `6 tests: 6 passed, 0 failed`. The status is 0.
Standard error is empty. The two files produce one report text.
The report has no field in which `off` could appear, because no
test calls `off`.

**Proposition J3.6.** Listing J3.2 passes every test in Listing
J3.1. At `k = 0x01` the two `inner` functions differ:
Listing J3.1 denotes `0x37` and Listing J3.2 denotes `0x00`.

*Proof.* The four branches of Listing J3.2 return `0x36`, `0x3d`,
`0x7c`, and `0x9c` at `0x00`, `0x0b`, `0x4a`, and `0xaa`. Those
are Propositions J3.1, J3.2, and J3.3, which are the expected
bytes in the tests that call `inner` on those inputs. The
boundary test reads index 19 and index 20 of the same key Listing
J3.1 builds, so it reads `0x0b` and `0x00`, which are two of those
four inputs. `abc_bits` is the same multiplication in both files.
Every test `Bool` is therefore true in both files.

`0x01` equals none of `0x00`, `0x0b`, `0x4a`, and `0xaa`, so
Listing J3.2 takes the final branch and denotes `0x00`. Listing
J3.1 denotes `0x01 XOR 0x36`, which Proposition J3.4 puts at
`0x37`. `0x00` and `0x37` differ, so the functions differ at
`0x01`. □

A passing corpus can be shared by two functions that are not the
same function. The extra byte was not a published vector. It did
not need to be. Assumption J3.8 only needs one input in the domain
where the functions differ. `0x01` is that input for this pair.
The specification's value there is a XOR, not a second copy of a
tag from a standard.

### J3.5 A corpus defect, diagnosed and repaired

Listing J3.3 computes `inner` as Listing J3.1 does. The test title
names RFC 4231 §4.3, test case 2, the key byte J. The input is
`0x4a`, which is that byte. The expected byte is `0x3d`.
Proposition J3.1 says `0x3d` is `0x0b XOR 0x36`, the result C1
pins for test case 1. The title and the expected byte name
different vectors of the same edition. The input names the vector
in the title. The expected byte was copied from the other vector.
This is the shape of Listing J2.2 in §J2.5, applied to a vector
id instead of to an edition: the function is the one the
propositions describe, and the copied expected value is the side
that does not match.

**Listing J3.3 — `mislabelled.or`**

```orange
edition 2026;
module mislabelled {
  spec inner(k: Word[8]) -> Word[8] { k ^ 0x36 }
  spec case2_inner() -> Word[8] { inner(0x4a) }
  test "RFC 4231 4.3 test case 2 key byte J" { case2_inner() == 0x3d }
}
```

```sh
./compiler/target/debug/orangec check mislabelled.or
./compiler/target/debug/orangec eval mislabelled.or
./compiler/target/debug/orangec test mislabelled.or
```

Check is silent. The false pin is still well-formed. A silent
check is not a true test.

**Expected evaluation output:**

```text
mislabelled::case2_inner: Word[8] = 0x7c
```

`eval` does not apply the test. `case2_inner` denotes `0x7c`,
which is Proposition J3.2. Changing the expected byte in the test
does not change `inner`.

**Test report:**

```text
test "RFC 4231 4.3 test case 2 key byte J" ... FAILED
    left:  0x7c
    right: 0x3d
1 test: 0 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. There is no
`ORC` code. `left` is the value `case2_inner()` denotes. `right`
is the byte written in the test. Left is the §4.3 result. Right
is the §4.2 result.

**Proposition J3.7.** The `Bool` in Listing J3.3 is false because
the expected byte is `0x3d` and `case2_inner()` denotes `0x7c`.
Replacing `0x3d` with `0x7c` makes the `Bool` true. No change to
`inner` is required.

*Proof.* `inner(0x4a)` denotes `0x4a XOR 0x36`, which Proposition
J3.2 puts at `0x7c`. The test compares that value with `0x3d`.
Proposition J3.1 puts `0x3d` at `0x0b XOR 0x36`. The bit strings
`01111100` and `00111101` differ, so the bytes differ, and the
`Bool` is false. Substituting `0x7c` makes the two sides the same
value, so `==` denotes true. The substitution is in the test.
The spec is untouched. □

The minimal repair is that one byte. The repaired test is the
second test of Listing J3.1. Rewriting `inner` would answer a left
value that disagreed with Proposition J3.2. Left agrees. The
failure report is how you know which side was copied from test
case 1.

The repair does not reach the claims the test never made. The
repaired `Bool` is still one byte of test case 2. It is not the
tag, it is not the other three key bytes, and it is not test
case 3. A pass after a repair establishes the repaired `Bool`.

A different defect would have passed. Suppose the file contained
Listing J3.1's C1 test twice, and the second copy carried the title
`RFC 4231 4.7 test case 6 key`. Both `Bool`s would be true, because
both compare `inner(0x0b)` with `0x3d`. The compiler would print
two `ok` lines. §4.7's key is 131 bytes of `aa`, and Assumption
J3.7 says that key is hashed before use. The duplicate does not
hash it. The title would be false while the `Bool` is true. The
compiler rejects a disagreement between two bytes. It does not
reject a disagreement between a title and a section you did not
make the `Bool` mention. The defect this section walked is the
one the report can show, because the copied byte and the computed
byte differ. The duplicate is the one the report cannot show.
Both are corpus defects. Only one of them fails `orangec test`.

### J3.6 Two wrong implementations that agree

The thin corpus is C1, C2, and C3: the bytes `0x0b`, `0x4a`, and
`0x00`. It omits C6. Proposition J3.3 says each of those three
bytes has high bit 0, and `0xaa` has high bit 1. Any function
that agrees with `k XOR 0x36` on the low half of the domain will
pass the thin corpus. The thin corpus cannot see the high half.

Listing J3.4 returns the three thin results by comparing the input
with `0x00`, `0x0b`, and `0x4a`, and returns `0x00` for every other
byte. Listing J3.5 computes `k XOR 0x36`, then XORs an extra
`0x01` when the high bit of `k` is set. On the thin corpus the
high bit is clear, so the extra XOR does not run.

Both files include a fourth test, C6, whose expected byte is
`0x9c`. That test is the distinguishing vector. Its provenance is
the C6 row of the table in §J3.2: RFC 4231, December 2005, §4.4,
HMAC-SHA-256 test case 3, the key byte `0xaa`, covered as one
byte under the inner pad and not as the combined length 70.

**Listing J3.4 — `wrong_table.or`**

```orange
edition 2026;
module wrong_table {
  spec inner(k: Word[8]) -> Word[8] {
    if k == 0x00 { 0x36 } else { if k == 0x0b { 0x3d } else { if k == 0x4a { 0x7c } else { 0x00 } } }
  }
  spec case3_inner() -> Word[8] { inner(0xaa) }
  test "RFC 2104 2 zero fill" { inner(0x00) == 0x36 }
  test "RFC 4231 4.2 test case 1 key byte" { inner(0x0b) == 0x3d }
  test "RFC 4231 4.3 test case 2 key byte J" { inner(0x4a) == 0x7c }
  test "RFC 4231 4.4 test case 3 key byte" { case3_inner() == 0x9c }
}
```

**Listing J3.5 — `wrong_flip.or`**

```orange
edition 2026;
module wrong_flip {
  spec inner(k: Word[8]) -> Word[8] {
    let x: Word[8] = k ^ 0x36;
    if (k & 0x80) == 0x00 { x } else { x ^ 0x01 }
  }
  spec case3_inner() -> Word[8] { inner(0xaa) }
  test "RFC 2104 2 zero fill" { inner(0x00) == 0x36 }
  test "RFC 4231 4.2 test case 1 key byte" { inner(0x0b) == 0x3d }
  test "RFC 4231 4.3 test case 2 key byte J" { inner(0x4a) == 0x7c }
  test "RFC 4231 4.4 test case 3 key byte" { case3_inner() == 0x9c }
}
```

```sh
./compiler/target/debug/orangec check wrong_table.or
./compiler/target/debug/orangec eval wrong_table.or
./compiler/target/debug/orangec test wrong_table.or
./compiler/target/debug/orangec check wrong_flip.or
./compiler/target/debug/orangec eval wrong_flip.or
./compiler/target/debug/orangec test wrong_flip.or
```

Check is silent for both files. The status of each check is 0.

**Expected evaluation output for Listing J3.4:**

```text
wrong_table::case3_inner: Word[8] = 0x00
```

**Expected evaluation output for Listing J3.5:**

```text
wrong_flip::case3_inner: Word[8] = 0x9d
```

`eval` prints `inner(0xaa)` and does not run the tests. The table
denotes `0x00` there. The flip denotes `0x9d`.

**Test report for Listing J3.4:**

```text
test "RFC 2104 2 zero fill" ... ok
test "RFC 4231 4.2 test case 1 key byte" ... ok
test "RFC 4231 4.3 test case 2 key byte J" ... ok
test "RFC 4231 4.4 test case 3 key byte" ... FAILED
    left:  0x00
    right: 0x9c
4 tests: 3 passed, 1 failed
```

**Test report for Listing J3.5:**

```text
test "RFC 2104 2 zero fill" ... ok
test "RFC 4231 4.2 test case 1 key byte" ... ok
test "RFC 4231 4.3 test case 2 key byte J" ... ok
test "RFC 4231 4.4 test case 3 key byte" ... FAILED
    left:  0x9d
    right: 0x9c
4 tests: 3 passed, 1 failed
```

The status of each `test` is 1. Standard error is empty. There is
no `ORC` code. In both reports the first three tests are `ok` and
the fourth is `FAILED`. The right-hand byte of the failure is
`0x9c` in both files, the byte Proposition J3.3 assigns to C6.
The left-hand bytes differ: `0x00` for the table, `0x9d` for the
flip.

**Proposition J3.8.** At `0x00`, `0x0b`, and `0x4a`, Listing J3.4
and Listing J3.5 both denote `k XOR 0x36`. At `0xaa`, Listing J3.4
denotes `0x00` and Listing J3.5 denotes `0x9d`. Neither equals
`0x9c`, and `0x00` differs from `0x9d`.

*Proof.* The high bit of `0x00`, `0x0b`, and `0x4a` is 0, by the
bit strings in Propositions J3.1 and J3.2. Listing J3.5 therefore
takes the branch that returns `x`, and `x` is `k XOR 0x36`.
Listing J3.4 returns `0x36`, `0x3d`, and `0x7c` on those three
inputs, which are the same three results. The three thin tests
compare those results with themselves, so those `Bool`s are true
in both files.

`0xaa` equals none of the three literals, so Listing J3.4 returns
`0x00`. The high bit of `0xaa` is 1, so Listing J3.5 returns
`(0xaa XOR 0x36) XOR 0x01`. Proposition J3.3 puts the inner XOR at
`0x9c`, and Proposition J3.4 puts the extra XOR at `0x9d`. The
fourth test compares each of those values with `0x9c`. `0x00` is
not `0x9c`. `0x9d` is not `0x9c`, because Proposition J3.4 flipped
the low bit. The two left values differ from each other because
`0x00` is not `0x9d`. □

The thin corpus is three passing tests. Both files pass it. The
files are not the specification, and they are not each other. C6
is what separates them, and what separates each of them from
`k XOR 0x36`. A prose sentence that such files could be written
is not this demonstration. The demonstration is the two listings,
the three `ok` lines, and the two `left` values.

One further agreement is accidental, and the thin corpus does not
see it either. Listing J3.4 returns `0x00` on the final branch.
`0x36 XOR 0x36 = 0x00`, so at input `0x36` the table and the
specification both denote `0x00`. `0x36` is not `0x00`, not
`0x0b`, and not `0x4a`, so the table reaches that branch. A test
that added `inner(0x36) == 0x00` would pass for the table and for
the specification, and it would still fail to mention `0xaa`.
Adding a point where a wrong function happens to be right does
not make the function right.

Listing J3.5 is right on every byte whose high bit is 0, all
128 of them, and wrong on every byte whose high bit is 1, the
other 128. The thin corpus is drawn entirely from the first
half. Any other corpus drawn entirely from that half would
accept Listing J3.5 as well. C6 is one byte of the second half.
It is enough, because one disagreement is enough.

Put §J3.4 beside this section. Listing J3.2 passes the six-vector
corpus and fails at `0x01`, a byte you chose because it was
absent. Listings J3.4 and J3.5 pass the three-vector corpus and
fail at `0xaa`, a byte a standard prints. The first pair shows
that the corpus you just watched go green is not the
specification. The second pair shows that two wrong programs can
agree on every vector a thin corpus contains and still be told
apart by one pinned vector the corpus left out. Both pairs are
finite. The domain has 256 elements. You do not need the other
elements once one of them has already disagreed.

### J3.7 Exercises

Twelve exercises. The first four use the corpus in §J3.2. The next
two use the defect in §J3.5. Two use the coverage argument in
§J3.4. Two use the reports in §J3.6. The last two use Listing
J3.2. Each answer is in the next section. A guess without the
arithmetic is not an answer.

**Exercise J3.1 — Five fields.** Write the five fields of vector
C2. Use Assumption J3.3 and the table in §J3.2.

**Exercise J3.2 — The byte J.** Compute `0x4a XOR 0x36` by the bit
table. Then add 74 and 54, and say which of the two results
`inner` denotes.

**Exercise J3.3 — Twenty bytes of aa.** The §4.4 key is printed as
32 hex digits, then 8 hex digits. How many bytes is that? Compute
`0xaa XOR 0x36`.

**Exercise J3.4 — The high bit.** Of `0x00`, `0x0b`, `0x4a`, and
`0xaa`, which have high bit 1? Which of them does the thin corpus
of §J3.6 contain?

**Exercise J3.5 — Read the report.** Listing J3.3 fails with left
`0x7c` and right `0x3d`. Which vector does each byte belong to,
and what one change repairs the test?

**Exercise J3.6 — A title the compiler accepts.** A second test,
titled `RFC 4231 4.7 test case 6 key`, has the body
`inner(0x0b) == 0x3d`. Does `orangec test` fail that file if the
first test is C1? What is the length of the key §4.7 prints, and
what step does that length require?

**Exercise J3.7 — Seventy.** Compute the combined length §4.4's
heading talks about, from a key of 20 bytes and data of 50 bytes.
Say what C6's `Bool` covers.

**Exercise J3.8 — Twenty-four.** What does the passing C5 test
establish? Where does the digest of “abc” live, relative to this
corpus?

**Exercise J3.9 — Two left values.** On C6, Listing J3.4 prints
left `0x00` and Listing J3.5 prints left `0x9d`. Show that
`0x9c XOR 0x01 = 0x9d`, and that neither left value equals the
right-hand `0x9c`.

**Exercise J3.10 — How many bytes are unnamed.** How many values
of a byte does the thin corpus not name? How many does the set
`{0x00, 0x0b, 0x4a, 0xaa}` not name?

**Exercise J3.11 — The zero byte.** Say what C3 covers, and say
why a passing C3 line is not a test of HMAC on an empty message.

**Exercise J3.12 — Six passes.** Listing J3.2's test report matches
Listing J3.1. `off` denotes `0x00`. Compute `0x01 XOR 0x36`. What
does the passing report fail to establish about `inner`?

## Worked answers

**J3.1.** The source is RFC 4231. The edition is December 2005.
The section is §4.3. The vector id is HMAC-SHA-256 test case 2,
the first key byte. The vector covers `0x4a`, the first byte of
the key the section prints as `4a656665`. It does not cover the
tag, and it does not cover the other three bytes of that key.

**J3.2.** The bits are

```text
0x4a = 01001010
0x36 = 00110110
XOR  = 01111100 = 0x7c
```

`0x4a` is 74 and `0x36` is 54. `74 + 54 = 128 = 0x80`. `inner`
denotes `0x7c`, the XOR. It does not denote `0x80`.

**J3.3.** `32 / 2 = 16` and `8 / 2 = 4`, so the key is
`16 + 4 = 20` bytes. The parenthetical in the file says the same
number. The XOR is

```text
0xaa = 10101010
0x36 = 00110110
XOR  = 10011100 = 0x9c
```

**J3.4.** `0xaa` is `10101010`, so its high bit is 1. `0x00`,
`0x0b`, and `0x4a` have high bit 0, by the strings in Propositions
J3.1 and J3.2. The thin corpus contains `0x00`, `0x0b`, and
`0x4a`. It does not contain `0xaa`.

**J3.5.** Left `0x7c` is Proposition J3.2, the inner pad of test
case 2's first key byte `0x4a`. Right `0x3d` is Proposition J3.1,
the inner pad of test case 1's key byte `0x0b`. The repair
replaces `0x3d` with `0x7c` in the test. `inner` stays
`k XOR 0x36`. The repaired test is C2 in Listing J3.1.

**J3.6.** The file would pass. Both tests compare `inner(0x0b)`
with `0x3d`, and Proposition J3.1 says that comparison is true.
The compiler prints `ok` for a true `Bool`. It does not compare
the title with §4.7. The key §4.7 prints is 131 bytes. That
length is greater than 64, so RFC 2104 §2 requires the key to be
hashed before it is used as the HMAC key. The duplicate test does
not hash a key.

**J3.7.** `20 + 50 = 70`, and `70` is greater than `64`. That is
the comparison the §4.4 heading states. C6's `Bool` is
`inner(0xaa) == 0x9c`. It covers one key byte under the inner pad.
It does not add the lengths, it does not read the data bytes, and
it does not compare the HMAC-SHA-256 tag §4.4 prints.

**J3.8.** C5 establishes that `abc_bits` denotes 24, which is
`3 * 8`, the bit length FIPS 180-4 §5.1.1 assigns to the message
“abc” in the August 2015 text §J2.3 cites for §5.1.1. The digest of that
message is the value N13 copied from NIST's examples file. That
file is not §5.1.1, and the digest is not an expected value in
this corpus. A passing C5 line does not compare it.

**J3.9.** The bits of the extra XOR are

```text
0x9c = 10011100
0x01 = 00000001
XOR  = 10011101 = 0x9d
```

The right-hand side of both failing tests is `0x9c`. Listing J3.4's
left value is `0x00`, and `0x00` differs from `0x9c`. Listing
J3.5's left value is `0x9d`, and the low bit just computed differs
from the low bit of `0x9c`, so `0x9d` differs from `0x9c`. The two
left values also differ from each other.

**J3.10.** A byte has 256 values. The thin corpus names 3, so it
leaves `256 - 3 = 253` unnamed. The four-byte set names 4, so it
leaves `256 - 4 = 252` unnamed. C4 does not add a fifth byte.
C5 does not name a byte of `inner` at all.

**J3.11.** C3 covers `inner(0x00)`, which is the inner pad of a
zero byte in the key block, the fill RFC 2104 §2 writes past a key
shorter than B. The §4.2 key occupies 20 bytes of a 64-byte block,
so 44 bytes of that block are this fill. C3 checks one of them.
An empty message is a data string of length 0. C3 does not pass a
data string to HMAC. HMAC is not the function under test.

**J3.12.** `0x01 XOR 0x36` is

```text
0x01 = 00000001
0x36 = 00110110
XOR  = 00110111 = 0x37
```

The passing report establishes the six `Bool`s. Those `Bool`s
constrain `inner` at `0x00`, `0x0b`, `0x4a`, and `0xaa`, and they
constrain `abc_bits`. They do not constrain `inner(0x01)`. Listing
J3.2 denotes `0x00` there, and the specification denotes `0x37`.
The report is green for both. Green does not establish that
`inner` is `k XOR 0x36` on the whole domain. Do not call the six
passes verified.

## Sources and epigraph record

The quotation is the borrowed sentence. The dates and identifiers
in the vector table are the publication's own cover lines, checked
against the file named here and against the pins N13 and J2
already recorded. Later sections quote clauses. They do not
replace this record.

**[S14] M. Nystrom.** RFC 4231, “Identifiers and Test Vectors for
HMAC-SHA-224, HMAC-SHA-256, HMAC-SHA-384, and HMAC-SHA-512,”
Internet Engineering Task Force, Standards Track, December 2005.
The epigraph is one sentence of §4.1, the sentence that begins
“An implementation that concurs with the results provided in this
document,”. It follows the sentence that says the test vectors
have been cross-verified by three independent implementations.
Section 5 says that no assertion of the security of these message
authentication code schemes for any particular use is intended.
Wording was checked on 2026-10-09 against the plain-text RFC at
the RFC Editor. The epigraph sentence contains no inner quotation.
No translation is involved. The sentence is not an endorsement of
Orange. It records what the RFC offers a matching implementation:
interoperability with other implementations that match the same
vectors.
This record's tag is [S14].

Source: <https://www.rfc-editor.org/rfc/rfc4231.txt>

The author's name in the “Author's Address” section is Magnus
Nystrom, RSA Security. The cover prints `M. Nystrom`.

**[T10] Retrieved file.** On 2026-10-09 the lesson retrieved the
plain text of RFC 4231 from the RFC Editor. The SHA-256 digest of
that file is
`72178527ce93500e730bc8eb182b857e583096d652b64ece0879c52ba1df973b`.
The August 2015 pin of FIPS PUB 180-4, and the December 2005 pin
of RFC 4231 test case 1 in §4.2, are the pins §J2.3 keeps from N13.
The February 1997 pin of RFC 2104, the test case 2 fields, and the
statement that §5.1.1 does not print the digest of “abc”, are the
pins N13 recorded and §J2.3 repeats for the digest. This lesson
does not claim a new retrieval of those earlier files. A digest
is of the retrieved file. Another rendering can carry the same
sentence and a different digest. No translation is involved.
The file is not an endorsement of Orange, and the retrieval is
not a certification.
This record's tag is [T10].

**[C4] Corpus surface.** The listings use `edition 2026`,
`module`, `spec`, `let`, `Word[8]`, `Word[8]^64`, `Int`, `^`,
`&`, `if`, `else`, `for`, a fill `[0; 64]`, `with`, an index,
`==`, `&&`, and `test`. Those forms are the ones N13 and N14
already ran on this compiler. No listing uses `Word[32]`, a
shift, a rotation, `as`, or `use`. No form in the listings is
Proposed. A passing test is a Match on the inputs it writes. It
is not a constant-time claim, not a certification, and not a
transcription of FIPS 180-4 §6.2.2.
This record's tag is [C4].

## Evidence boundary

J3 is a Journeyman lesson. The six outcomes in §J3.1 are the finish
line. The assumptions in §J3.1 bound them. Listings J3.1 through
J3.5 are the corpus, the implementation that passes the corpus and
fails off it, the mislabelled vector, and the two implementations
that agree on the thin corpus. Twelve exercises have worked
answers. The integer ledger is recomputed by
`tools/test_book_foundations.py`. The Orange listings are the
fenced programs in this file.
`compiler/crates/orangec/tests/book_j3.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not
derive FIPS 180-4 §6.2.2, and they do not accept a proposal.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-09 at the
owner's direction. Owner review is pending. No deployment
recommendation is made.
