# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J3: The Corpus as Acceptance Test. Draft 2026-10-09.

Continue from
[Modules and Provenance](NOVICE_N13_MODULES_AND_PROVENANCE.md#n13-modules-and-provenance)
and from J2, Standards as Versioned Inputs. N13.3 pinned an expected
digest to a document, an edition, a vector, and a list of non-claims.
J2 made the edition a procedure: one expected value, one section, one
edition, and a failing test when the edition is wrong. This lesson
takes the next object. A corpus is the set of those pins that a tree
actually runs, and an acceptance test is one pin the compiler can
fail. The reading habit is still
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
A passing test is still the Match of
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study).

J4 is the study of byte order as a function. This lesson writes eight
length bytes for one integer and does not define that function. J5 is
still the study that derives the SHA-256 compression function and its
message schedule from FIPS 180-4 §6.2.2. This lesson cites that
section as an address. It does not transcribe the function.

This lesson is **J3**. The locked label is J3. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. The
manuscript chapter titled *The Corpus as Acceptance Test* keeps that
title and that number until a later renumber. This lesson is the
Journeyman rewrite of that chapter. It does not replace the manuscript
file. When the text says “§5.1.1”, the number is a section of the
edition named in the same sentence.

## J3: The Corpus as Acceptance Test

> “But: program testing can be a very effective way to show the presence of bugs, but is hopelessly inadequate for showing their absence.”
>
> — Edsger W. Dijkstra, “The Humble Programmer” (EWD 340, 1972), argument three. [S14]

The sentence is about a program and a test. A test can catch a
program that is wrong on the input the test writes. The same test
cannot, by passing, show that the program is right on the inputs it
does not write. A corpus is a finite list of such tests. The length
of the list does not change the shape of the sentence. It changes
only how many inputs were written down.

**The only-this-stack test.** You will read the corpus this tree
actually runs, run `orangec test` on it, and keep the report. You
will then pin one expected block to one section of one edition,
derive the bytes before the compiler prints them, and run `orangec
check`, `orangec eval`, and `orangec test` on the compiler this tree
builds. You will substitute the length field of a different vector
and read the failing report. The compiler's version line, from the
binary built in this tree, is:

```text
orangec 0.0.1 (Orange edition 2026; implemented slice S3t)
```

Every listing is written for that slice. A form this slice does not
implement is marked Proposed, and this lesson does not use one.

### J3.1 What you will be able to do

Five outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can say what a test corpus is in this repository. You can
   name the directory, the pair convention, and the command that
   runs `test` declarations. You can read a report of zero tests,
   and you can read the one source whose report is not that line.
2. You can record the provenance of one expected value. The record
   names the standard, the edition and its date, the section, the
   vector's identity, and the role of the value.
3. You can attach that value to an Orange `test` and run `orangec
   check`, `orangec eval`, and `orangec test`. You can say what a
   passing test matches. A Match is not called verified.
4. You can diagnose one deliberate wrong vector from the failing
   report. The report names the test, prints both values, and, when
   the values are arrays, names the first index that differs. You
   can repair the expected value and read the new report.
5. You can prove, for finite sets, that k passing vectors do not
   separate implementations that agree on those k inputs. You can
   point at two Orange functions that witness the count.

The documents the lesson actually uses are these. FIPS PUB 180-4,
*Secure Hash Standard (SHS)*, August 2015, DOI
`10.6028/NIST.FIPS.180-4`, the edition N13 recorded. The message is
the 8-bit ASCII message “abc” of §5.1.1, the same message N13
pinned, and the value derived here is the padded block, not the
digest. The repository files read beside that pin are
`algorithms/README.md`, `algorithms/verify.py`,
`algorithms/sha2/sha2.or`, `algorithms/sha2/README.md`, and
`algorithms/x25519/field25519-limbs.or`.

### J3.2 What you may take as given

Eight assumptions bound the lesson. A later sentence that needs a
further fact names it there.

**Assumption J3.1 — The edition token is not the standard's date.**
Every Orange listing begins with `edition 2026;`. That token is the
edition this compiler requires. It is not August 2015. Changing the
token does not select a different FIPS publication. [T8]

**Assumption J3.2 — A title is not a pin.** A `test` title is a
string the report prints between quotes. It does not open a
document. The pin is the record in the prose: the standard, the
edition and its date, the section, the vector's identity, and the
role of the expected value. A title that contains those words is
still a string until the record is checked against the pages.

**Assumption J3.3 — The directory `algorithms/` is the reference
corpus this tree runs.** Each entry is a reference evaluation of a
specification. The entry's README is the place that names the
standard and the vectors. An entry does not carry the manuscript's
admission record, and the README of the directory says that an
entry makes no constant-time, side-channel, performance,
interoperability, or certification claim.

**Assumption J3.4 — `orangec test` runs tests, and it does not pair
names.** The command checks one source and then runs the root
module's `test` declarations. A parameterless spec whose name ends
in `_expected` is not a test. The command does not look for that
suffix. [T6]

**Assumption J3.5 — A Match is not a verification.** A silent check
means the source was well-formed under the checks this compiler
runs. An evaluation means the parameterless specs that ran produced
the printed values. A passing test means the `Bool` in that test
was true on the inputs the test wrote. None of those reads a
standard on its own, none is a constant-time claim, and none is
called verified. [C4]

**Assumption J3.6 — This lesson does not transcribe SHA-256's
compression function or its message schedule.** Those operations are
FIPS 180-4 §6.2.2, and deriving them is J5. N13 transcribed them
and did not derive them. J2 withheld them. This lesson withholds
them again. The padding clause §5.1.1 is the clause this lesson
derives. The digest is not on this page.

**Assumption J3.7 — For the integer 24, the length bytes are the
ones proved below.** Where this lesson writes the bit length 24 as
eight bytes, high byte first, it uses the uniqueness proof for that
one integer. J4 is the study of byte order as a function of every
integer. Stating the eight bytes of 24 is not that study.

**Assumption J3.8 — The count is a count of functions on finite
sets.** The sets are named in the proposition. The count is not a
count of source files, it is not a count of attacks, and it is not
a probability.

### J3.3 The corpus this repository runs

The directory `algorithms/` holds one folder per algorithm. On this
tree that is 20 folders and 39 files whose names end in `.or`. Each
folder holds a `README.md` and one or more sources. The directory
README gives every entry the same shape of record: an Analysis, a
Dissemination, and the vectors the entry reproduces. The script
`algorithms/verify.py` requires those two headings, requires every
source to pass `orangec check` with empty diagnostics, and requires
every source to contain at least one pair of parameterless specs.

A pair is two specs. The spec `<name>` computes a value. The spec
`<name>_expected` states a literal. The script runs `orangec eval`
and requires the two printed lines to carry the same type and the
same value. The Rust test `compiler/crates/orangec/tests/algorithms.rs`
reads the same pairs. This lesson did not run that script and did
not run that Rust test. What this lesson ran is recorded below.

The directory README's table of published cryptographic vectors
adds as follows. The block-cipher rows are 13, 8, 8, 6, 12, 7, 12,
and 13, which sum to 79. The authenticated-encryption rows are 10,
7, 25, and 17, which sum to 59. The stream-cipher rows are 20, 19,
and 5, which sum to 44. The hash and key-derivation rows are 13, 8,
and 16, which sum to 37. The public-key rows are 4 and 17, which
sum to 21. The five family totals sum to 240. The same README
states twelve mathematical answer pairs besides those 240. The sum
240 + 12 = 252 is the README's figure for recorded answer pairs.
This lesson checks the additions. It does not open every vector
file and recount the pairs.

The SHA-256 entry is the one this lesson uses as the example of a
cryptographic source. `algorithms/sha2/sha2.or` computes digests,
including the digest of “abc”, and it states the published digests
as `<name>_expected` specs. The file's comment on padding attributes
to §5.1.1 the rule derived in §J3.6: the message, the bit 1 written
as the byte `0x80` when the message is whole bytes, k zero bits so
that the length so far is 448 bits modulo 512, and the bit length as
64 bits. The spec `pad_3` is that rule for a 3-byte message. This
lesson does not copy `pad_3`, and it does not copy the compression
function that the file applies to the padded block. The digest
those specs compute is J5's object. The padding is this lesson's.

### J3.4 A file with no test

Assumption J3.4 says `orangec test` does not pair names. The
command on the SHA-256 source is:

```sh
./compiler/target/debug/orangec test algorithms/sha2/sha2.or
```

**Test report:**

```text
0 tests: 0 passed, 0 failed
```

The status is 0. Standard error is empty. The line says the root
module declared no test. It does not say that a computed digest
equaled an `_expected` digest. It does not say that `orangec eval`
was run. A reader who treats status 0 as “the corpus passed” has
read a report the command did not print.

The same command was run on each of the 39 `.or` sources under
`algorithms/`. Thirty-eight of them printed that same line, exited
0, and wrote nothing to standard error. One source did not. It is
`algorithms/x25519/field25519-limbs.or`.

```sh
./compiler/target/debug/orangec test algorithms/x25519/field25519-limbs.or
```

**Test report:**

```text
test "tight does not imply canonical" ... ok
test "distinct admitted representations can denote zero" ... ok
test "addition examples retain the loose bound" ... ok
test "carry examples retain their field value" ... ok
test "noncanonical input has canonical output" ... ok
test "canonicalization retains the p plus one field value" ... ok
test "maximum exact product satisfies its accumulator bound" ... ok
test "the third product carry preserves the tight output bound" ... ok
test "correct multiplication residue does not admit a nontight input" ... ok
test "correct carry residue does not establish the accumulator contract" ... ok
10 tests: 10 passed, 0 failed
```

The status is 0. Standard error is empty. Ten tests passed. The
file's header says the radix and the carry schedule are original
definitions, not an RFC 7748 storage requirement, and that the
Boolean examples are not refinement proofs. A passing line is a
Match of the `Bool` that test writes. It is not an RFC 7748 known
answer, and it is not a constant-time claim. Do not call that Match
verified.

The published cryptographic vectors live in the sources that printed
the zero line. Their acceptance check, in this repository, is the
pair convention under `orangec eval`, read by `algorithms/verify.py`
and by the Rust test named above. `orangec test` does not perform
that check. The limb file is the exception that shows the command
can run tests when a source declares them. It does not turn the
other 38 reports into pair checks.

Listing J3.1 is the pair convention on one byte, small enough to
read whole. It is not one of the 39 sources. It exists so the two
printed lines and the zero-test line can sit on the same page.

**Listing J3.1 — `pair.or`**

```orange
edition 2026;
module pair {
  spec marker() -> Word[8] { 0x80 }
  spec marker_expected() -> Word[8] { 0x80 }
}
```

```sh
./compiler/target/debug/orangec check pair.or
./compiler/target/debug/orangec eval pair.or
./compiler/target/debug/orangec test pair.or
```

Check is silent. The status is 0. Standard output and standard
error are empty. Silence means the source was well-formed. It does
not mean the two specs were compared.

**Expected evaluation output:**

```text
pair::marker: Word[8] = 0x80
pair::marker_expected: Word[8] = 0x80
```

`eval` runs the parameterless specs. It does not run tests. The two
lines carry the same type and the same value. That agreement is
what the pair convention requires. The command that prints the lines
does not itself fail when they differ. A reader, or the script that
parses the lines, has to compare them.

**Test report:**

```text
0 tests: 0 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. The report is
the same line `sha2.or` printed. The module declared no test, so
there was nothing to fail. Equal evaluation lines are not a passing
test. They are two lines a later comparison can read.

### J3.5 Provenance of an expected value

A known-answer test compares a value the program computes with a
value you were obliged to copy from a named place. The copy is part
of the claim. N13.3 required four parts for an expected digest: the
document, the edition, the vector's identity, and a statement of
what the comparison does not cover. J2 required the edition to be a
date, the section to be a section of that edition, and the role of
the value to be named: an input, a constant, or an output. This
lesson uses both. The pin for the block derived below is the
following record, and no shorter record is the pin.

**Provenance — the padded block of “abc”.** The standard is FIPS
PUB 180-4, *Secure Hash Standard (SHS)*, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. That is the edition N13 recorded. The
section is §5.1.1. The vector's identity is the 8-bit ASCII message
“abc”, the one-block message of that section, whose bytes N13
recorded as `0x61`, `0x62`, and `0x63`. The role of the expected
value is an output of the padding clause on that input. It is not
an input, and it is not the digest.

The bytes of the message, as integers, are 97, 98, and 99.

```text
6 * 16 + 1 = 97
6 * 16 + 2 = 98
6 * 16 + 3 = 99
```

Those are `0x61`, `0x62`, and `0x63`. The bit length is `3 * 8 = 24`,
which is the length N13 recorded for this message.

The test does not cover the empty message, except insofar as a later
exercise asks for it. It does not cover a 56-byte message. It does
not cover SHA-512's 128-bit length field. It does not cover the
digest. It does not prove that every message pads according to the
clause. It does not say the evaluator is constant-time. A passing
report is a Match of this one 64-byte block on this one message. Do
not call that Match verified.

Two editions that print the same padding bytes do not, by a passing
test, choose the edition. J2 already showed that shape for a
constant both editions print. The failure this lesson runs is a
wrong vector inside the edition already named, not a second date.

### J3.6 The padding of one message

The rule this lesson derives is the rule `algorithms/sha2/sha2.or`
attributes to §5.1.1, restricted to one message of whole bytes.
Append a 1 bit, then the smallest number k of 0 bits such that the
length so far is congruent to 448 modulo 512, then the bit length
as a 64-bit integer, high byte first. For a message whose length is
a multiple of 8, the 1 bit begins a byte, and that byte is `0x80`.
Assumption J3.7 is the high-byte-first writing of the single integer
24. The propositions fix every byte before a listing runs.

**Proposition J3.1.** Let l = 24. The smallest nonnegative integer
k such that l + 1 + k = 448 + 512t for some integer t ≥ 0 is
k = 423, and that solution has t = 0.

*Proof.* l + 1 = 25. Then 448 − 25 = 423, and 423 ≥ 0, so t = 0
gives k = 423. If t ≥ 1, then k = 423 + 512t ≥ 423 + 512, which is
greater than 423. Every later solution is larger. □

**Proposition J3.2.** The byte `0x80` is a 1 followed by seven 0
bits, high bit on the left. Of the 423 zero bits, 7 lie in that
byte and the other 416 are 52 zero bytes.

*Proof.* 8 × 16 = 128, so `0x80` = 128. And 128 = 2^7, so the only
bit set in that byte is the high bit. The bits are 10000000. The
message length 24 is a multiple of 8, so the next bit begins a new
byte. The rule puts 1 there and then 423 zeros. The first byte
therefore holds the 1 and 7 zeros, which is `0x80`, and those 7
zeros are 7 of the 423. The remainder is 423 − 7 = 416. Then
416 = 52 × 8, so the remainder is 52 zero bytes. □

**Proposition J3.3.** The unique 8-byte writing of the integer 24,
high byte first, is seven 0 bytes followed by the byte 24. That
last byte is `0x18`.

*Proof.* Such a writing is eight digits d7, d6, …, d0, each an
integer from 0 through 255, with value

```text
d7 * 256^7 + d6 * 256^6 + ... + d1 * 256 + d0
```

The high byte is d7. Suppose some di with i ≥ 1 is at least 1.
The value is then at least 256. But 24 < 256, so every such di is
0. The value that remains is d0, so d0 = 24. The byte 24 is
`0x18`, because 1 × 16 + 8 = 24. □

**Proposition J3.4.** Under the rule above, the padding of the
bytes `0x61`, `0x62`, `0x63` is one 64-byte string, and only that
string. It is those three bytes, then `0x80`, then 52 zero bytes,
then the eight bytes of Proposition J3.3.

*Proof.* Propositions J3.1 and J3.2 place the three message bytes,
the marker, and the 52 zero bytes. Proposition J3.3 places the
length. The sum of the lengths is 3 + 1 + 52 + 8 = 64, so the
result is one block and the length occupies the last eight bytes.
Indices 0, 1, and 2 are the message. Index 3 is `0x80`. Indices 4
through 55 are the 52 zeros, because 55 − 4 + 1 = 52. Indices 56
through 62 are the seven high zeros of the length, because
62 − 56 + 1 = 7. Index 63 is `0x18`. Each index was fixed. A
different 64-byte string differs at one of those indices, so it is
not this padding. □

The 52 zeros and the 7 high zeros of the length are adjacent. From
index 4 through index 62 the byte is 0, which is 59 zeros, and
index 63 is the only nonzero length byte. That is the same string.
The split matters because the 52 zeros are padding and the 7 zeros
are part of the length. A test that only looks at index 4 does not
by itself say which of those two roles the zero is playing.

### J3.7 The acceptance test

Listing J3.2 writes Proposition J3.4 as a computation and as tests.
The expected block in the last test is the string the proposition
names. It was not taken from the compiler's output. The compiler is
asked whether the computation denotes that string.

**Listing J3.2 — `abc_pad.or`**

```orange
edition 2026;
module abc_pad {
  spec message() -> Word[8]^3 { [0x61, 0x62, 0x63] }
  spec bit_length() -> Int { 3 * 8 }
  spec padded() -> Word[8]^64 {
    let copied: Word[8]^64 = for i in 0..3 with b: Word[8]^64 = [0; 64] {
      b with [i] = message()[i]
    };
    let marked: Word[8]^64 = copied with [3] = 0x80;
    marked with [63] = 0x18
  }
  spec marker() -> Word[8] { padded()[3] }
  spec length_byte() -> Word[8] { padded()[63] }
  spec head() -> Word[8]^4 {
    [padded()[0], padded()[1], padded()[2], padded()[3]]
  }
  test "FIPS 180-4 August 2015 5.1.1 abc bit length" { bit_length() == 24 }
  test "FIPS 180-4 August 2015 5.1.1 abc marker" { marker() == 0x80 }
  test "FIPS 180-4 August 2015 5.1.1 abc length byte" { length_byte() == 0x18 }
  test "FIPS 180-4 August 2015 5.1.1 abc bytes 0 through 3" {
    head() == [0x61, 0x62, 0x63, 0x80]
  }
  test "FIPS 180-4 August 2015 5.1.1 abc index 4" { padded()[4] == 0x00 }
  test "FIPS 180-4 August 2015 5.1.1 abc index 62" { padded()[62] == 0x00 }
  test "FIPS 180-4 August 2015 5.1.1 abc padded block" {
    padded() == [0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18]
  }
}
```

Each test in this listing is one comparison, so a failure can print
both values. Where a later listing joins two comparisons with `&&`,
each comparison is parenthesized. This compiler rejects the
ungrouped form.

```sh
./compiler/target/debug/orangec check abc_pad.or
./compiler/target/debug/orangec eval abc_pad.or
./compiler/target/debug/orangec test abc_pad.or
```

Check is silent. The status is 0. Standard output and standard
error are empty.

**Expected evaluation output:**

```text
abc_pad::message: Word[8]^3 = [0x61, 0x62, 0x63]
abc_pad::bit_length: Int = 24
abc_pad::padded: Word[8]^64 = [0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18]
abc_pad::marker: Word[8] = 0x80
abc_pad::length_byte: Word[8] = 0x18
abc_pad::head: Word[8]^4 = [0x61, 0x62, 0x63, 0x80]
```

`eval` does not run the tests. The printed block is the string
Proposition J3.4 describes: `0x61`, `0x62`, `0x63`, `0x80`, zeros
through the next-to-last byte, and `0x18` last. `bit_length` is 24.
`marker` is `0x80`. `length_byte` is `0x18`. `head` is the first
four bytes.

**Test report:**

```text
test "FIPS 180-4 August 2015 5.1.1 abc bit length" ... ok
test "FIPS 180-4 August 2015 5.1.1 abc marker" ... ok
test "FIPS 180-4 August 2015 5.1.1 abc length byte" ... ok
test "FIPS 180-4 August 2015 5.1.1 abc bytes 0 through 3" ... ok
test "FIPS 180-4 August 2015 5.1.1 abc index 4" ... ok
test "FIPS 180-4 August 2015 5.1.1 abc index 62" ... ok
test "FIPS 180-4 August 2015 5.1.1 abc padded block" ... ok
7 tests: 7 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. Seven tests
passed. Each `ok` means the `Bool` in that test was true. The last
test is the whole block. The six tests before it are parts of the
same block. Passing the marker test does not run the block test.
The report's last line counts them. It does not merge them into one
sentence about every message.

What this passing report establishes is narrow. On this compiler,
`padded` denotes the 64-byte string in the last test, for the
message the listing wrote. That string is the padding Proposition
J3.4 derived for “abc”. The tests do not establish the digest of
“abc”. They do not run the compression function. They do not
establish the padding of any other message. A function that returns
this block on this message and a different block on the empty
message passes every test in the listing. Section J3.10 counts how
many such functions there are. Do not call this Match verified.

### J3.8 The wrong vector

The 56-byte message is a different vector. `algorithms/sha2/sha2.or`
records its bit length as 448, which is the l of `pad_56`. This
lesson does not pad that message. It computes the two low length
bytes of 448, which are what a 64-bit high-byte-first writing of
448 ends with, and then it deliberately demands those bytes as the
tail of the “abc” block.

**Proposition J3.5.** 56 × 8 = 448, and 448 = 1 × 256 + 192, and
192 = `0xc0`.

*Proof.* 50 × 8 = 400 and 6 × 8 = 48, so 56 × 8 = 448. Then
448 − 256 = 192, so the quotient on division by 256 is 1 and the
remainder is 192. And 12 × 16 = 192, with remainder 0, so the hex
digits of 192 are `c` and `0`. □

The two low bytes are therefore `0x01` and `0xc0`. For the integer
24, Proposition J3.3 says the two low bytes are `0x00` and `0x18`.
The bytes are not the same.

**Listing J3.3 — `length_field.or`**

```orange
edition 2026;
module length_field {
  spec bits() -> Int { 56 * 8 }
  spec high() -> Int { bits() / 256 }
  spec low() -> Int { bits() % 256 }
  test "56-byte message, high length byte" { high() == 1 }
  test "56-byte message, low length byte" { low() == 0xc0 }
}
```

```sh
./compiler/target/debug/orangec check length_field.or
./compiler/target/debug/orangec eval length_field.or
./compiler/target/debug/orangec test length_field.or
```

Check is silent, status 0, with empty standard output and empty
standard error.

**Expected evaluation output:**

```text
length_field::bits: Int = 448
length_field::high: Int = 1
length_field::low: Int = 192
```

**Test report:**

```text
test "56-byte message, high length byte" ... ok
test "56-byte message, low length byte" ... ok
2 tests: 2 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. `low` denotes
192, and the test writes `0xc0`. The passing test says those two
writings are the same integer. It does not say that 192 is the
length byte of “abc”. The role is the low byte of a different
vector's length. Assumption J3.2 is why the title has to say
“56-byte message” and the prose has to say which message. The
digits 192 do not carry the role by themselves.

Listing J3.4 computes the “abc” block, the same way Listing J3.2
does, and then demands the 56-byte tail.

**Listing J3.4 — `wrong_vector.or`**

```orange
edition 2026;
module wrong_vector {
  spec message() -> Word[8]^3 { [0x61, 0x62, 0x63] }
  spec padded() -> Word[8]^64 {
    let copied: Word[8]^64 = for i in 0..3 with b: Word[8]^64 = [0; 64] {
      b with [i] = message()[i]
    };
    let marked: Word[8]^64 = copied with [3] = 0x80;
    marked with [63] = 0x18
  }
  spec marker() -> Word[8] { padded()[3] }
  spec tail() -> Word[8]^2 { [padded()[62], padded()[63]] }
  test "FIPS 180-4 August 2015 5.1.1 abc marker" { marker() == 0x80 }
  test "FIPS 180-4 August 2015 5.1.1 abc length copied from the 56-byte message" {
    tail() == [0x01, 0xc0]
  }
}
```

```sh
./compiler/target/debug/orangec check wrong_vector.or
./compiler/target/debug/orangec eval wrong_vector.or
./compiler/target/debug/orangec test wrong_vector.or
```

Check is silent, status 0. Evaluation status is 0. Standard error
is empty for both. `eval` still does not run the tests, so a wrong
expected value does not show up in the evaluation.

**Expected evaluation output:**

```text
wrong_vector::message: Word[8]^3 = [0x61, 0x62, 0x63]
wrong_vector::padded: Word[8]^64 = [0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18]
wrong_vector::marker: Word[8] = 0x80
wrong_vector::tail: Word[8]^2 = [0x00, 0x18]
```

The computed tail is `[0x00, 0x18]`. That is index 62 and index 63
of the block from Proposition J3.4. The evaluation did not fail.
The failure is in the test.

**Test report:**

```text
test "FIPS 180-4 August 2015 5.1.1 abc marker" ... ok
test "FIPS 180-4 August 2015 5.1.1 abc length copied from the 56-byte message" ... FAILED
    left:  [0x00, 0x18]
    right: [0x01, 0xc0]
    first difference at [0]
2 tests: 1 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. A failed test
is not a diagnostic. The first test passed. The second failed. The
passing test did not stop the run, and it did not make the second
test true.

The report prints `left` with two spaces after the colon and
`right` with one space after the colon. `left` is the value the
program computed, `tail()`. `right` is the value the test demanded.
The line `first difference at [0]` names an index in those two
arrays, not an index in the 64-byte block. Index 0 of `tail` is
`padded()[62]`. The computed byte there is `0x00`. The demanded
byte is `0x01`.

**Proposition J3.6.** The `Bool` in the second test of Listing J3.4
is false. The arrays differ at both indices. The first index that
differs is 0, and that index is byte 62 of the padded block.

*Proof.* `tail()` denotes `(padded()[62], padded()[63])`. By
Proposition J3.4 those bytes are `0x00` and `0x18`. The demanded
array is `0x01`, `0xc0`, the two bytes of Proposition J3.5. At
index 0, `0x00` ≠ `0x01`. At index 1, `0x18` ≠ `0xc0`. The least
index where the arrays differ is 0. The spec `tail` builds index 0
from `padded()[62]`. □

The report names the first index only. Both bytes differ, and the
printed arrays show the second difference to a reader who looks.
The padding function was not the defect. It denotes the block
Proposition J3.4 names, which is why the marker test passed and why
`eval` printed `0x18` at the end of the block. The defect is the
expected value: the length field of the 56-byte message, copied
onto “abc”. The repair replaces the expected array with
`[0x00, 0x18]`. It does not replace the padding. Exercise J3.10
asks for that repair, and the worked answer runs it.

A wrong edition whose padding bytes are the same as this edition's
would not fail this test. The failure here is a wrong vector, which
is a difference the bytes can show. Agreement of the bytes still
does not choose the edition. That is J2's result, and this report
does not reopen it.

### J3.9 What the k reports cannot separate

The whole-block test in Listing J3.2 accepts every implementation
that returns that block on the message “abc”. Let D be a set of
messages that contains “abc” and at least one other message, and
let C be the set of all 64-byte strings. A function from D to C is
determined by the block it returns on each message. The test fixes
the block on one message. It does not fix the block on any other.

The count is easier to see on a domain of three inputs and a
codomain a reader can compute. Let the inputs be the bytes 0, 1,
and 2. Let the corpus be the inputs 0 and 1. Let the codomain be
all 256 bytes. Two functions that agree on 0 and 1 and differ on 2
receive the same two corpus reports.

**Proposition J3.7.** Let D and C be finite sets with |D| = n and
|C| = m, and let m ≥ 1. Let E be a subset of D with |E| = k, and
let k ≤ n. Fix a function f from D to C. Let P be the set of all
functions g from D to C such that g(x) = f(x) for every x in E.
Then |P| = m^(n − k). If n = k, then |P| = 1. If n > k and m ≥ 2,
then |P| ≥ 2.

*Proof.* On E the value of a function in P is fixed: it copies f.
On D without E the value at each element may be any element of C,
and the choices do not depend on one another. The set D without E
has n − k elements. There are therefore m choices, n − k times,
which is the product of n − k factors of m. That product is
m^(n − k).

If n − k = 0, there is no element left to choose. The product of
no factors is 1, by the convention that an empty product equals the
multiplicative identity. P then contains only f, because every
input in D is an input in E.

If n − k ≥ 1 and m ≥ 2, then m^(n − k) is at least m, and m is at
least 2. So |P| ≥ 2. The function f is one element of P. At least
one other function agrees with f on E and differs from f on some
input outside E. □

The box picture is the same count. Label a box by the k pairs
(x, f(x)) for x in E. Every function in P goes in that box, because
each of them produces those k pairs. The reports of the k tests are
the label of the box. They are not a name of one function inside
the box. When the box holds two functions, the reports cannot say
which of the two ran.

**Proposition J3.8.** Take n = 3, k = 2, and m = 256. Then
|P| = 256, so |P| > 1.

*Proof.* n − k = 1. Proposition J3.7 gives 256^1 = 256. And
256 > 1. □

The 256 is 2^8, computed by eight multiplications by 2, starting
at 1. The products are 2, 4, 8, 16, 32, 64, 128, and 256. Listing
J3.6 writes that sequence as a loop. A byte has those 256 values,
so the codomain of a function that returns a byte is large enough
for Proposition J3.8.

Apply the same proposition to the block test, without computing the
large power. Here k = 1, the one message “abc”. The codomain is
the set of 64-byte strings, so m = 256^64. In particular m ≥ 2. If
the domain contains one message besides “abc”, then n − k ≥ 1, so
|P| ≥ 2 by Proposition J3.7. The passing block test does not
determine the padding of that other message. The empty message is
such a message. Its padding is Proposition J3.4 with length 0, and
the block test of “abc” does not mention it.

Listing J3.5 is two functions on the three-byte domain. `keep`
returns its input. `other` returns its input, except that it
returns 0 on input 2. The literal `2` in a `Word[8]` comparison is
the byte 2. Evaluation prints that byte as `0x02`, and it prints
the byte 0 as `0x00`.

**Listing J3.5 — `agree.or`**

```orange
edition 2026;
module agree {
  spec keep(x: Word[8]) -> Word[8] { x }
  spec other(x: Word[8]) -> Word[8] {
    if x == 2 { 0 } else { x }
  }
  spec keep0() -> Word[8] { keep(0) }
  spec keep1() -> Word[8] { keep(1) }
  spec keep2() -> Word[8] { keep(2) }
  spec other0() -> Word[8] { other(0) }
  spec other1() -> Word[8] { other(1) }
  spec other2() -> Word[8] { other(2) }
  test "corpus input 0" { (keep0() == 0) && (other0() == 0) }
  test "corpus input 1" { (keep1() == 1) && (other1() == 1) }
  test "input outside the corpus" { keep2() == other2() }
}
```

```sh
./compiler/target/debug/orangec check agree.or
./compiler/target/debug/orangec eval agree.or
./compiler/target/debug/orangec test agree.or
```

Check is silent, status 0, with empty standard output and empty
standard error.

**Expected evaluation output:**

```text
agree::keep0: Word[8] = 0x00
agree::keep1: Word[8] = 0x01
agree::keep2: Word[8] = 0x02
agree::other0: Word[8] = 0x00
agree::other1: Word[8] = 0x01
agree::other2: Word[8] = 0x00
```

`keep` and `other` agree at 0 and at 1. They differ at 2: `0x02`
against `0x00`. The evaluation printed both. It did not decide
which function a corpus should accept. The tests do that, and only
for the inputs they write.

**Test report:**

```text
test "corpus input 0" ... ok
test "corpus input 1" ... ok
test "input outside the corpus" ... FAILED
    left:  0x02
    right: 0x00
3 tests: 2 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. The first two
tests passed. They are the corpus of Proposition J3.8. Both
functions satisfy them. The third test is the input outside the
corpus, written as one equality so a failure prints `left` and
`right`. `left` is `keep(2)`, which is `0x02`. `right` is
`other(2)`, which is `0x00`. There is no `first difference` line.
That line is printed for arrays and tuples, and these two values
are bytes.

**Proposition J3.9.** `keep` and `other` agree at 0 and at 1, and
they differ at 2. A corpus that checks only the first two tests
accepts both functions.

*Proof.* By the bodies, `keep(0) = 0` and `other(0) = 0`, and
`keep(1) = 1` and `other(1) = 1`. Also `keep(2) = 2` and
`other(2) = 0`, and 2 ≠ 0. The first two tests mention inputs 0
and 1 only. Both functions make those `Bool`s true. The third
test mentions input 2, and that `Bool` is false. □

The two passing lines are Matches on the bytes the tests wrote.
They are not Proposition J3.9, and Proposition J3.9 did not become
a proof by the tests passing. The proof is the reading of the two
bodies. The failing line is the witness that the box holds two
functions. A reader who deletes the third test still has a module
that passes, and still has two functions.

### J3.10 The count, written as tests

Listing J3.6 computes the numbers Proposition J3.8 uses. The tests
are Matches of that arithmetic. They are not a search through the
256 functions.

**Listing J3.6 — `count.or`**

```orange
edition 2026;
module count {
  spec domain() -> Int { 3 }
  spec corpus() -> Int { 2 }
  spec free() -> Int { domain() - corpus() }
  spec codomain() -> Int {
    for i in 0..8 with p: Int = 1 { p * 2 }
  }
  spec accepted() -> Int { codomain() }
  test "one input lies outside a corpus of two" { free() == 1 }
  test "eight doublings make 256" { codomain() == 256 }
  test "the free input has 256 images" { accepted() == 256 }
}
```

```sh
./compiler/target/debug/orangec check count.or
./compiler/target/debug/orangec eval count.or
./compiler/target/debug/orangec test count.or
```

Check is silent, status 0, with empty standard output and empty
standard error.

**Expected evaluation output:**

```text
count::domain: Int = 3
count::corpus: Int = 2
count::free: Int = 1
count::codomain: Int = 256
count::accepted: Int = 256
```

**Test report:**

```text
test "one input lies outside a corpus of two" ... ok
test "eight doublings make 256" ... ok
test "the free input has 256 images" ... ok
3 tests: 3 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. `free` is
3 − 2 = 1. `codomain` is the eighth doubling, 256. `accepted` is
that same 256, because the exponent in Proposition J3.8 is 1, so
m^(n − k) = m. The listing does not compute a power with a varying
exponent. The proof does. When the exponent is 0, the proof gives
1, and this listing does not contain a test of that 1. The worked
answer of Exercise J3.7 is where that case is stated.

A passing report here means these integers are the ones the specs
denote. It does not mean the compiler counted functions. The count
of functions is Proposition J3.7. The listing is the arithmetic
the proposition uses for one triple (n, k, m).

### J3.11 What the report does not say

Collect the non-claims in one place, each attached to the report
that does not say it.

The line `0 tests: 0 passed, 0 failed` on `sha2.or` says the module
declared no test and the source checked. It does not say the
recorded digests were reproduced.

The ten `ok` lines on `field25519-limbs.or` say those ten `Bool`s
were true. They do not say the limb representation is the one RFC
7748 requires for a coordinate. The file's header says it is not
that requirement.

The seven `ok` lines on `abc_pad.or` say the padding specs denote
the bytes Proposition J3.4 names, on the message the listing wrote.
They do not say the digest of “abc” equals the NIST one-block
sample. They do not say the empty message pads correctly. They do
not say the evaluator is constant-time. They do not accept the
tests specification. That specification remains the proposed text
[T6] names. A run is not an acceptance of the proposal.

The failing report on `wrong_vector.or` says one `Bool` is false,
and it shows the bytes. It does not say the padding function is the
wrong function. The marker test passed in the same run.

The two `ok` lines on `agree.or` say `keep` and `other` agree at 0
and at 1. They do not say the two functions are the same function.
The next line says they are not.

None of these reports is called verified. None of them is a
constant-time claim. None of them is a certification. Assumption
J3.5 is that list.

### J3.12 The self-check

The finish line is the five outcomes in §J3.1, met on a vector the
walk did not derive in full. The empty message is that vector. Its
bit length is 0. The marker is still `0x80`, and it sits at index
0 because the message has no bytes. The length bytes are eight
zeros, because Proposition J3.3's argument with 24 replaced by 0
still forces every digit to be 0. The exercises ask for the pin,
the unread indices, the count, and the repair.

Close the listings and write the ten answers before you read the
worked answers. If the count you get for one free input is not
256, or the index you get for the first difference is not 62, the
miss is this lesson. J4 does not repair it. J4 is byte order. J5
does not repair it. J5 is the compression function. Beginning
either chapter with the count still wrong is leaving the corpus
unread.

```text
j3-ledger
entries = 20
sources = 39
zero-test-sources = 38
limb-tests = 10
block-vectors = 79
aead-vectors = 59
stream-vectors = 44
hash-vectors = 37
public-vectors = 21
published-vectors = 240
readme-mathematical-pairs = 12
recorded-pairs = 252
message-bytes = 3
bit-length = 24
one-plus-length = 25
k-zeros = 423
marker = 128
zero-bits-in-marker = 7
zero-bits-after = 416
zero-bytes-after-marker = 52
length-high-zeros = 7
length-byte = 24
block-bytes = 64
fifty-six-bytes = 56
fifty-six-bits = 448
length-high = 1
length-low = 192
domain = 3
corpus = 2
free = 1
codomain = 256
accepted = 256
covered = 1
empty-length = 0
unread-empty = 62
ascii-a = 97
ascii-b = 98
ascii-c = 99
```

The ledger is the arithmetic of the propositions, the family sums
of §J3.3, and the file counts of this tree. The foundation check
recomputes the arithmetic and recounts the files. It does not run
`orangec`. The Rust test runs the listings and the 39 sources.

## Exercises

**Exercise J3.1 — The zero line.** The report of `orangec test` on
`algorithms/sha2/sha2.or` is `0 tests: 0 passed, 0 failed`, status
0, standard error empty. State what that report establishes. State
two things it does not establish. One of the two must be the pair
convention.

**Exercise J3.2 — The limb file.** The report on
`algorithms/x25519/field25519-limbs.or` is ten passing tests.
State one claim that report does not establish. Use the file's own
header, not a guess about a coordinate format the header does not
mention.

**Exercise J3.3 — The pin.** For the 64-byte block of Listing J3.2,
name the standard, the edition and its date, the section, the
vector's identity, and the role of the expected value. Say which
of those parts a test title does not check.

**Exercise J3.4 — The zero bits.** From l = 24, compute k, the
number of zero bits inside the marker byte, and the number of zero
bytes after the marker. Show the subtractions.

**Exercise J3.5 — The length bytes.** Show that 24 < 256, and write
the eight length bytes in order. Name the index of `0x18`.

**Exercise J3.6 — Read the failure.** In Listing J3.4 the report
says `first difference at [0]`, with left `[0x00, 0x18]` and right
`[0x01, 0xc0]`. Which index of the 64-byte block is that `[0]`?
Which message's length field was copied? Does the second byte of
the tail also differ?

**Exercise J3.7 — The count.** For a domain of 3 inputs, a corpus
of 2, and a codomain of 256, how many functions agree with a fixed
function on the corpus? How many when the corpus is the whole
domain? Which proposition gives both numbers?

**Exercise J3.8 — A third function.** Listing J3.9, in the worked
answers, defines `third`, which returns its input except that it
returns 1 on input 2. Do the two corpus tests accept it beside
`keep`? What is `keep(2)` against `third(2)`? What does the
listing's own third test check instead?

**Exercise J3.9 — The empty message.** For the empty message under
the same padding rule, where does `0x80` sit, and what is the byte
at index 63? Listing J3.7 tests those two positions. Which indices
do those two tests leave unread, and how many indices is that?

**Exercise J3.10 — The repair.** Listing J3.4 fails because the
expected tail is `[0x01, 0xc0]`. What expected tail makes the
comparison true? After that change, what is the last line of a
report that contains only that one test, and what is the status?

## Worked answers

**J3.1.** The report establishes that the source checked and that
the root module declared no test. The status is 0 and standard
error is empty, which is what this compiler prints in that case.
It does not establish that any `<name>` spec equals the spec
`<name>_expected`. That comparison is the pair convention, and
`orangec test` does not perform it. It also does not establish the
digest of “abc”. The digest is not a `test` in that file, and this
lesson does not derive it.

**J3.2.** The report does not establish that the limb representation
is the coordinate encoding RFC 7748 requires. The header of
`field25519-limbs.or` says the radix and the carry schedule are
original definitions, not an RFC 7748 storage requirement, and that
the Boolean examples are not refinement proofs. Ten passing tests
are ten Matches. They are not that requirement.

**J3.3.** The standard is FIPS PUB 180-4. The edition is August
2015, DOI `10.6028/NIST.FIPS.180-4`. The section is §5.1.1. The
vector is the message “abc”. The role is an output of the padding
clause on that input, the 64-byte block, not the digest. A test
title is a string. Assumption J3.2 says the title does not open the
document and does not check the edition, the section, or the role.
The prose record is the check. The compiler checks the `Bool`.

**J3.4.** l + 1 = 25, and 448 − 25 = 423, so k = 423. The marker
byte holds 7 of those zero bits. 423 − 7 = 416, and 416 / 8 = 52,
so 52 zero bytes follow the marker. That is Proposition J3.1 and
Proposition J3.2.

**J3.5.** 24 < 256, so in the high-byte-first writing every digit
except the last is 0, and the last digit is 24, which is `0x18`.
The eight bytes are `0x00`, `0x00`, `0x00`, `0x00`, `0x00`, `0x00`,
`0x00`, `0x18`. In the 64-byte block they occupy indices 56 through
63, so `0x18` is index 63. That is Proposition J3.3.

**J3.6.** `tail` builds its array as `padded()[62]` and then
`padded()[63]`. Index 0 of that array is byte 62 of the block. The
copied field is the low two bytes of the bit length 448, which is
the length of the 56-byte message. The second bytes are `0x18` and
`0xc0`, which differ. The report names only the first difference.
Proposition J3.6 records both.

**J3.7.** Proposition J3.7 gives m^(n − k). Here n = 3, k = 2, and
m = 256, so the exponent is 1 and the count is 256. Proposition
J3.8 is that case. When the corpus is the whole domain, k = n, the
exponent is 0, and the count is 1. The one function is the fixed
function itself. The ledger names that 1 as `covered`.

**J3.8.** `third` agrees with `keep` at 0 and at 1, so the two
corpus tests accept it. `keep(2)` is `0x02` and `third(2)` is
`0x01`, so those bytes differ. The listing's third test checks
`third(2) == 1`, which is true, and does not ask whether `third`
equals `keep`. One more function in the box of 256 is still inside
the box.

**Listing J3.9 — `another.or`**

```orange
edition 2026;
module another {
  spec keep(x: Word[8]) -> Word[8] { x }
  spec third(x: Word[8]) -> Word[8] {
    if x == 2 { 1 } else { x }
  }
  spec at2() -> Word[8] { third(2) }
  test "corpus input 0" { (keep(0) == 0) && (third(0) == 0) }
  test "corpus input 1" { (keep(1) == 1) && (third(1) == 1) }
  test "input 2 is the byte 1" { at2() == 1 }
}
```

```sh
./compiler/target/debug/orangec check another.or
./compiler/target/debug/orangec eval another.or
./compiler/target/debug/orangec test another.or
```

Check is silent, status 0.

**Expected evaluation output:**

```text
another::at2: Word[8] = 0x01
```

**Test report:**

```text
test "corpus input 0" ... ok
test "corpus input 1" ... ok
test "input 2 is the byte 1" ... ok
3 tests: 3 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. `at2` is the
byte 1, printed `0x01`. The three tests passed. None of them is the
comparison `keep(2) == third(2)`, which would fail.

**J3.9.** The empty message has no bytes, so the marker is index 0.
The bit length is 0. Every digit of the high-byte-first writing of
0 is 0, so index 63 is `0x00`. The two tests mention index 0 and
index 63. They leave indices 1 through 62 unread. That is
62 − 1 + 1 = 62 indices. A block that holds `0x80` at index 0,
`0x00` at index 63, and `0x01` at index 1 passes both tests and is
not the padding. The padding's unread bytes are zeros. The tests
do not say so.

**Listing J3.7 — `empty_pad.or`**

```orange
edition 2026;
module empty_pad {
  spec padded() -> Word[8]^64 {
    [0; 64] with [0] = 0x80
  }
  test "FIPS 180-4 August 2015 5.1.1 empty marker" { padded()[0] == 0x80 }
  test "FIPS 180-4 August 2015 5.1.1 empty length byte" { padded()[63] == 0x00 }
}
```

```sh
./compiler/target/debug/orangec check empty_pad.or
./compiler/target/debug/orangec eval empty_pad.or
./compiler/target/debug/orangec test empty_pad.or
```

Check is silent, status 0.

**Expected evaluation output:**

```text
empty_pad::padded: Word[8]^64 = [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

The printed block is the padding: `0x80` and then 63 zeros. The
evaluation shows the zeros the two tests do not mention. The tests
still do not mention them. A reader who trusts the tests alone has
not read this line.

**Test report:**

```text
test "FIPS 180-4 August 2015 5.1.1 empty marker" ... ok
test "FIPS 180-4 August 2015 5.1.1 empty length byte" ... ok
2 tests: 2 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. Two Matches,
on index 0 and index 63. Do not call that Match verified.

**J3.10.** The expected tail that matches Proposition J3.4 is
`[0x00, 0x18]`. The padding function stays. The last line of a
report that contains only that test is `1 test: 1 passed, 0 failed`.
The status is 0. Standard error is empty.

**Listing J3.8 — `repaired.or`**

```orange
edition 2026;
module repaired {
  spec message() -> Word[8]^3 { [0x61, 0x62, 0x63] }
  spec padded() -> Word[8]^64 {
    let copied: Word[8]^64 = for i in 0..3 with b: Word[8]^64 = [0; 64] {
      b with [i] = message()[i]
    };
    let marked: Word[8]^64 = copied with [3] = 0x80;
    marked with [63] = 0x18
  }
  spec marker() -> Word[8] { padded()[3] }
  spec tail() -> Word[8]^2 { [padded()[62], padded()[63]] }
  test "FIPS 180-4 August 2015 5.1.1 abc length field" {
    tail() == [0x00, 0x18]
  }
}
```

```sh
./compiler/target/debug/orangec check repaired.or
./compiler/target/debug/orangec eval repaired.or
./compiler/target/debug/orangec test repaired.or
```

Check is silent, status 0. Evaluation status is 0.

**Expected evaluation output:**

```text
repaired::message: Word[8]^3 = [0x61, 0x62, 0x63]
repaired::padded: Word[8]^64 = [0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18]
repaired::marker: Word[8] = 0x80
repaired::tail: Word[8]^2 = [0x00, 0x18]
```

The tail is `[0x00, 0x18]` before the test runs. The repair did
not change the computation. It changed the expected array.

**Test report:**

```text
test "FIPS 180-4 August 2015 5.1.1 abc length field" ... ok
1 test: 1 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. One test
passed. The passing line is a Match of the tail of “abc” against
the bytes Proposition J3.4 assigned to indices 62 and 63. It is
not a Match of the whole block. Indices 0 through 61 are unread by
this test. The whole-block test is the one in Listing J3.2.

## Sources and epigraph record

The quotation is the borrowed sentence. The padding bytes are
derived in §J3.6 from the rule the SHA-256 entry attributes to
§5.1.1. The edition is the one N13 recorded. Later sections do not
replace this record.

**[S14] Edsger W. Dijkstra.** “The Humble Programmer,” EWD 340,
the ACM Turing Award lecture, 1972. The archive transcription
titles the page “The Humble Programmer by Edsger W. Dijkstra” and
heads it “ACM Turing Lecture 1972 | EWD340”. The epigraph is the
sentence of argument three that begins “But: program testing can
be a very effective way”. In the transcription it follows “Today a
usual technique is to make a program and then to test it.” Wording
was checked on 2026-10-09 against the University of Texas
E. W. Dijkstra archive transcription. The sentence contains no
inner quotation. No translation is involved. The lecture goes on
to argue for a proof of correctness grown with the program. This
lesson records the sentence about tests. It does not claim the
lecture's forecast. The sentence is not an endorsement of Orange.
This record's tag is [S14].

Source: <https://www.cs.utexas.edu/~EWD/transcriptions/EWD03xx/EWD340.html>

**[T10] Corpus commands on this tree.** The binary is
`compiler/target/debug/orangec`, version line `orangec 0.0.1
(Orange edition 2026; implemented slice S3t)`. On 2026-10-09
`orangec test` was run on each of the 39 `.or` sources under
`algorithms/`. Thirty-eight printed `0 tests: 0 passed, 0 failed`
and exited 0 with empty standard error. `algorithms/x25519/field25519-limbs.or`
printed the ten passing lines quoted in §J3.4 and exited 0 with
empty standard error. The listings in this lesson were run with
the same binary. A later tree can hold a different count. The
count is a fact about this tree, checked by the Rust test that
reads this file. This record's tag is [T10].

The test form itself remains the record N12 named [T6]. The
edition token remains the record N14 named [T8]. This lesson cites
those records and does not redefine them.

**[C4] Acceptance surface.** The listings use `edition 2026`,
`module`, `spec`, `let`, `Int`, `Word[8]`, hexadecimal byte
literals, a bounded `for`, a fill `[0; 64]`, an array update
`with`, an index, `if` with `else`, `&&`, `==`, and `test`. Those
forms are ones N12, N13, and N14 already ran, together with `if`,
which the conditions slice already runs on this compiler. No form
in the listings is a syntax this slice lacks. `Int` division in
these listings is of nonnegative integers. A passing test is a
Match on the inputs it writes. It is not a constant-time claim,
not a certification, and not a transcription of FIPS 180-4 §6.2.2.
A report of zero tests is the absence of `test` declarations. It
is not a pair check. This record's tag is [C4].

## Evidence boundary

J3 is a Journeyman lesson. The five outcomes in §J3.1 are the
finish line. The assumptions in §J3.2 bound them. Listings J3.1
through J3.6 are the pair, the padding, the foreign length, the
failing vector, the two functions, and the count. Listings J3.7
through J3.9 are the worked answers. Ten exercises have worked
answers. The integer ledger is recomputed by
`tools/test_book_foundations.py`. The Orange listings are the
fenced programs in this file.
`compiler/crates/orangec/tests/book_j3.rs` runs them, and it runs
`orangec test` on every `.or` source under `algorithms/`. Those
checks do not establish a cryptographic security claim, they do
not derive FIPS 180-4 §6.2.2, and they do not accept a proposal.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-09 at the
owner's direction. Owner review is pending. No deployment
recommendation is made.
