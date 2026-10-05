# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J2: Standards as Versioned Inputs. Draft 2026-10-05.

Continue from
[Ready for Standards](NOVICE_N14_READY_FOR_STANDARDS.md#n14-ready-for-standards).
The gate asked you to carry five assumptions into Block A: the edition
pin, the byte order, the padding, the module seam, and the non-claims.
This lesson takes the first of those and makes it a procedure. The
reading habit is still
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
A passing test is still the Match of
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study)
and of N14: one `Bool`, on the inputs the test wrote. J5 is still the
study that derives the SHA-256 compression function and its message
schedule from FIPS 180-4 §6.2.2. This lesson cites that section as an
address. It does not transcribe the function.

This lesson is **J2**. The locked label is J2. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. The
manuscript chapter titled *Standards as Versioned Inputs* keeps that
title and that number until a later renumber. When the text says
“§2.8” or “§5.3.3”, the number is a section of the edition named in
the same sentence.

## J2: Standards as Versioned Inputs

> “RFC 7539, the predecessor of this document, was meant to serve as a stable reference and an implementation guide.”
>
> — Yoav Nir and Adam Langley, RFC 8439, *ChaCha20 and Poly1305 for IETF Protocols* (June 2018), Abstract. [S13]

The sentence names a purpose and a predecessor. RFC 7539, dated May
2015, was written to be the stable reference. RFC 8439, dated June
2018, obsoletes it. The abstract of the later RFC says that the new
text merges the errata filed against the old one and adds a little
text to the Security Considerations section. A document that was
meant to hold still was replaced, and the replacement says so in its
first paragraph. The name “ChaCha20 and Poly1305 for IETF Protocols”
is the title of both. The title does not say which file you have.

**The only-this-stack test.** A prose account can place two printed
integers side by side and tell you which edition printed which. It
cannot reject a file. In this lesson you will pin one expected value
to one edition and one section, write that pin as an Orange `test`,
and run `orangec check`, `orangec eval`, and `orangec test` on the
compiler this tree builds. You will then substitute the other
edition's printed value and read the failing report. The report
names the test, prints the value the program computed, and prints
the value the test demanded. That rejection is the act a prose book
cannot perform. The compiler's version line is `orangec 0.0.1
(Orange edition 2026; implemented slice S3t)`. Every listing is
written for that slice. A form this slice does not implement is
marked Proposed, and this lesson does not use one.

### J2.1 What you will be able to do

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can name the exact document. For an RFC, the name is the
   number, the month and year on the cover, and a choice among three
   objects: that RFC, an erratum filed against it, or the RFC that
   obsoletes it. For a FIPS publication, the name is the publication
   number and the date the edition prints, including an update date
   when the cover prints one.
2. You can cite a clause by the section number of the edition you
   named. You can also tell when that same number denotes a different
   clause in another edition.
3. You can record where one expected value came from. The record
   names the issuer, the document, the edition and its date, the
   section, and the role of the value: an input, a constant, or an
   output of that section.
4. You can attach that value to an Orange `test` and run `orangec
   check`, `orangec eval`, and `orangec test`. You can say what a
   passing test matches.
5. You can say what that test does not establish. A constant that
   two editions print alike does not, by matching, choose the
   edition. A Match is not called verified.
6. You can diagnose one deliberate mismatch, a value copied from the
   wrong edition or a section number that moved between editions,
   from the failing test the compiler prints. You can apply the same
   diagnosis to a pin the walk did not repair. Missing that
   diagnosis means the pin is still a name.

The documents the lesson actually opens are these, and no others are
required to finish it.

RFC 7539, *ChaCha20 and Poly1305 for IETF Protocols*, Y. Nir and
A. Langley, May 2015. RFC 8439, the same title, Y. Nir and
A. Langley, June 2018, which obsoletes RFC 7539. The verified errata
filed against RFC 7539, as the RFC Editor records them. FIPS PUB
180-4, *Secure Hash Standard (SHS)*, August 2015, which states that
it supersedes FIPS 180-3. FIPS 197, *Advanced Encryption Standard
(AES)*, published November 26, 2001, and updated May 9, 2023 as
NIST FIPS 197-upd1.

The repository files the lesson reads beside those documents are
`algorithms/chacha20/chacha20.or`, `algorithms/aes/aes.or`,
`algorithms/sha2/sha2.or`, and the Gate 0 fixture
`conformance/foundation/valid/standards-provenance.json`. Each of
those files already cites a document. The lesson asks which edition
the citation pins, and what an Orange test of one value from it
fails to decide.

### J2.2 What you may take as given

Eight assumptions bound the lesson. A later sentence that needs a
further fact names it there.

**Assumption J2.1 — The edition token is not the standard's date.**
Every Orange listing begins with `edition 2026;`. That token is the
edition this compiler requires. It is not May 2015, not June 2018,
not August 2015, and not May 9, 2023. Changing the token does not
select a different RFC or a different FIPS update. [T8]

**Assumption J2.2 — A title is not a pin.** “ChaCha20 and Poly1305
for IETF Protocols” is the title of RFC 7539 and of RFC 8439. “Secure
Hash Standard” is the title of more than one FIPS publication in
that series. “Advanced Encryption Standard (AES)” is the name FIPS
197 prints on both the November 26, 2001 publication and the May 9,
2023 update. A pin names the identifier and the date. A title alone
does not.

**Assumption J2.3 — An erratum, an RFC, and an obsoleting RFC are
three objects.** An erratum is a record filed against one RFC. It
has its own identifier, status, and date. It is not the RFC, and it
is not the later RFC that obsoletes that RFC, even when the later
RFC says it merges errata. A citation that says “RFC 7539” without
saying whether the errata are applied has not named one text.

**Assumption J2.4 — A section number is an address inside one
edition.** The digits name a clause only together with the edition.
FIPS 180-4's contents list §5.3.3 as the SHA-256 initial hash value
and §6.2.2 as the SHA-256 hash computation. Those addresses belong
to the August 2015 publication. A different edition of the Secure
Hash Standard may use the same digits for a different clause. The
citation carries the edition, or it has not cited a clause.

**Assumption J2.5 — An expected value has a role.** The record says
whether the printed figure is an input to the clause, a constant
the clause defines, or an output the clause computes on a stated
input. Copying an output into the position of an input, or a
constant from another section into the position of an output, is a
different pin. The digits can be right and the role wrong.

**Assumption J2.6 — A Match is not a verification.** A silent check
means the source was well-formed under the checks this compiler
runs. An evaluation means the parameterless specs that ran produced
the printed values. A passing test means the `Bool` in that test
was true on the inputs the test wrote. None of those is a proof for
every input, none reads the standard on its own, and none is called
verified. [C2]

**Assumption J2.7 — This lesson does not transcribe SHA-256's
compression function or its message schedule.** Those operations are
FIPS 180-4 §6.2.2, and deriving them is J5. N13 transcribed them and
did not derive them. N14 withheld them. This lesson withholds them
again. A section number of FIPS 180-4 may appear as a pin. The
functions do not appear on this page. The same restraint applies to
the AES round and to the ChaCha20 block function: N12 already
derived the block function, and this lesson uses a constant or a
printed bound from the RFC, not a second copy of that derivation.

**Assumption J2.8 — Byte order is part of the pin when a word is
assembled from printed bytes.** Where this lesson turns a printed
byte string into a `Word[32]`, it states the order. J4 is the study
of byte order and format boundaries. Stating the order for one
constant is not that study.

The repository entries under `algorithms/` are reference
evaluations. They reproduce vectors the entry's README names. They
are not corpus entries in the manuscript's sense, they make no
constant-time claim, and a passing evaluation is not a
certification. The Gate 0 provenance fixture records a shape. Its
digest is synthetic. A synthetic digest is not the digest of the
document it names.

### J2.3 One title, three objects

RFC 7539 and RFC 8439 share a title. They do not share a date, and
they do not share a number. The plain-text file of RFC 7539 prints
`May 2015` on its cover and does not print an `Obsoletes` line. The
plain-text file of RFC 8439 prints `June 2018` and prints
`Obsoletes: 7539`. Assumption J2.2 says the shared title is not a
pin. The pin is the number together with the date.

The RFC Editor keeps a third object: the errata filed against RFC
7539. An erratum has its own number, a status, a reporter, and a
date. The status is a word in that record. It is not a property of
an Orange test. The statuses that matter here are three.

A **Verified** erratum is one the RFC Editor has marked Verified.
That mark is the editor's, on that record. It does not make an
Orange Match verified, and Assumption J2.6 still stands for every
test in this lesson.

A record **Held for Document Update** is not the same mark. It is
waiting. Merging it into a later RFC is a further act, and the
abstract's verb does not perform that act for you.

A **Rejected** erratum is a record the editor declined. It is not
applied. Reading it as if it had been applied changes the pin.

Four Verified errata are on the record retrieved on 2026-10-05.
Two of them change a printed integer. One changes the width of a
length field in pseudocode. One changes a description of an output
from two results into one concatenation. The integers are the ones
an Orange `test` can compute without transcribing ChaCha20.

**Erratum 4858**, Technical, reported by Timm Korte on 2016-11-10,
Verified by Lars Eggert on 2016-11-13. It cites §2.8 of RFC 7539.
The text it quotes prints a total of `247,877,906,880` bytes. The
corrected sentence prints `274,877,906,880` bytes. The note says
there is an error in the result of `P_MAX = ((2^32) - 1) * 64`.

**Erratum 4861**, Technical, the same reporter and the same
verification date. It cites the same section. RFC 7539 prints
`C_MAX = P_MAX + tag length = 247,877,906,896 octets.` The
corrected line prints `274,877,906,896`. The note says the line was
found while reviewing erratum 4858, and that this erratum was
created by duplicating 4858.

**Erratum 4371**, Technical, reported by Adam Eijdenberg on
2015-05-21, Verified by Lars Eggert on 2015-06-03. It cites §2.8.1.
RFC 7539's pseudocode appends the lengths with `num_to_4_le_bytes`.
The correction uses `num_to_8_le_bytes`. The note says §2.8 gives
the lengths as 64-bit quantities, so the width is 8 bytes, not 4.

**Erratum 4700**, Technical, reported by Martin Thomson on
2016-05-24, Verified by Lars Eggert on 2016-06-08. It cites §2.8.
RFC 7539 says the AEAD output is twofold, a ciphertext and a tag.
The correction says the output is the concatenation of those two.
The note points at RFC 5116 §2.1, whose AEAD interface produces one
output. This lesson does not transcribe that interface. The erratum
is here because it is a change of shape, not of an integer, and a
test of `P_MAX` does not see it.

RFC 8439's abstract says the document merges the errata filed
against RFC 7539. That sentence is the authors' summary. It is not
a substitute for putting each erratum next to the new text.
Assumption J2.3 requires the three objects to stay distinct. The
check that belongs to this lesson is a count of the disputed
integers in the two plain-text files.

In the RFC 7539 file, the digit string `247,877,906,880` occurs
twice, and `274,877,906,880` occurs zero times. In the RFC 8439
file the counts are reversed. The same swap holds for
`247,877,906,896` and `274,877,906,896`: one occurrence in RFC
7539, and the other integer in RFC 8439. `num_to_4_le_bytes` occurs
in RFC 7539 and does not occur in RFC 8439. `num_to_8_le_bytes`
occurs in RFC 8439 and does not occur in RFC 7539.

Those counts were taken on 2026-10-05 from the plain-text files at
the RFC Editor. They say what those files contain. They do not say
that every Held erratum was merged, because this section did not
place a Held erratum beside the new text. A later reader who needs
one of those still has to do that placement. The abstract does not
do it.

One Rejected erratum is on the same page: erratum 8274, against
§2.3.2, marked Rejected. A Rejected record is not a correction.
Citing RFC 7539 “with the errata” without saying which statuses
were applied has mixed the three objects Assumption J2.3 separates.

The two places RFC 8439 prints `274,877,906,880` are not the same
sentence. One is the note in §2.8 that computes `(2^32 - 1)` blocks
of 64 bytes. The other is the limit list in the same section:
`P_MAX` (maximum size of the plaintext) is that many bytes, in the
list the RFC says is defined in §4 of RFC 5116. This lesson uses
the arithmetic the note states. It does not open RFC 5116. A pin
that stops at “the RFC 5116 limit” has named a different document.

### J2.4 The byte limit, derived and pinned

The note in RFC 8439 §2.8 says the amount of encrypted data possible
in a single invocation is `2^32 - 1` blocks of 64 bytes each,
because of the size of the block counter, and that this gives a
total of `274,877,906,880` bytes. The same paragraph in RFC 7539
prints `247,877,906,880` bytes for the same description of the
product. The description agrees. The printed total does not. The
role of the integer, under Assumption J2.5, is an output of that
multiplication, not an input to the cipher.

**Proposition J2.1.** `(2^32 - 1) * 64 = 274877906880`.

*Proof.* `64 = 2^6`, and `2^32 - 1` is the integer one below `2^32`.
Multiplying by `2^6` shifts the binary point of that difference:

`(2^32 - 1) * 2^6 = 2^38 - 2^6`.

`2^10 = 1024`, so `2^20 = 1048576` and `2^30 = 1073741824`. Then
`2^38 = 2^30 * 2^8 = 1073741824 * 256`. Split `256` as
`200 + 50 + 6`:

`1073741824 * 200 = 214748364800`,

`1073741824 * 50 = 53687091200`,

`1073741824 * 6 = 6442450944`.

The first two products sum to `268435456000`. Adding the third
gives `274877906944`. That is `2^38`. Subtract `2^6`, which is
`64`:

`274877906944 - 64 = 274877906880`.

The commas in the RFC are separators, not part of the integer. The
integer the June 2018 text prints is this one. □

**Proposition J2.2.** The integer RFC 7539 prints in that sentence,
`247877906880`, is `274877906880 - 27000000000`. The two integers
agree in every digit except the leading group, where `274` stands
in one file and `247` in the other.

*Proof.* `274877906880 - 247877906880 = 27000000000`, by subtracting
digitwise: the trailing digits `877906880` cancel, and
`274 - 247 = 27`, in the billions place, so the difference is
`27 * 1000000000 = 27000000000`. □

Erratum 4858's note marks the same swap, writing the corrected
leading group and the printed leading group so the exchanged digits
are visible. Proposition J2.2 is that observation as an integer
identity. It does not depend on the note. The note is the record
that a reader filed the disagreement, and that the editor marked
the record Verified.

The tag length in the `C_MAX` line is not a second mystery. RFC
8439 prints both `P_MAX` as `274,877,906,880` bytes and
`C_MAX = P_MAX + tag length = 274,877,906,896` octets. Subtracting
the two printed integers leaves `16`. A 128-bit tag is `128 / 8 =
16` octets, which is the width erratum 4700's quoted sentence
already calls a 128-bit tag. The same subtraction on the May 2015
pair leaves `16` as well: both files add sixteen, and both files
are wrong or right together on that addend. The edition
disagreement is the base, not the tag width.

`274877906880 + 16 = 274877906896`. That is the June 2018 `C_MAX`.
`247877906880 + 16 = 247877906896`. That is the May 2015 `C_MAX`.

Listing J2.1 pins the June 2018 integers. The multiplication is the
one the note describes. The expected values are the integers that
file prints. The test names the section. The name is a string in
the source. The compiler does not read RFC 8439. You do.

**Listing J2.1 — `byte_limit.or`**

```orange
edition 2026;
module byte_limit {
  spec blocks() -> Int { 4294967296 - 1 }
  spec bytes() -> Int { blocks() * 64 }
  spec c_max() -> Int { bytes() + 16 }
  test "RFC 8439 2.8 P_MAX" { bytes() == 274877906880 }
  test "RFC 8439 2.8 C_MAX" { c_max() == 274877906896 }
}
```

`edition 2026;` is Assumption J2.1. It does not select June 2018.
`4294967296` is `2^32`, so `blocks` is `2^32 - 1`. `bytes` is
Proposition J2.1. `c_max` adds the sixteen octets just derived.
Both tests write the comparison in the test, so a failure can
print the two integers. A comparison buried inside a `Bool` that
the test only requires to be true does not print them. N8 already
showed that shape for a word array. The same shape is used here
for an `Int`.

```sh
./compiler/target/debug/orangec check byte_limit.or
./compiler/target/debug/orangec eval byte_limit.or
./compiler/target/debug/orangec test byte_limit.or
```

Check is silent. The status is 0. Silence means the source was
well-formed. It does not mean the expected integers were copied
from June 2018 rather than from May 2015.

**Expected evaluation output:**

```text
byte_limit::blocks: Int = 4294967295
byte_limit::bytes: Int = 274877906880
byte_limit::c_max: Int = 274877906896
```

`eval` runs the parameterless specs. It does not run the tests.
The three integers are `2^32 - 1`, Proposition J2.1, and that
result plus 16.

**Test report:**

```text
test "RFC 8439 2.8 P_MAX" ... ok
test "RFC 8439 2.8 C_MAX" ... ok
2 tests: 2 passed, 0 failed
```

The status of `test` is 0. Standard error is empty. Each `ok` means
the `Bool` in that test was true. Two tests passed. The report does
not say that the strings `RFC 8439` and `2.8` were looked up in a
document. A test title is not a locator until a reader checks it.
Assumption J2.4 is that check, and it lives in the prose around the
listing, not in the title.

What this passing pair establishes is narrow. On this compiler, the
function `bytes` denotes `274877906880`, and `c_max` denotes
`274877906896`. Those are the integers printed in RFC 8439 §2.8 for
`P_MAX` and for `C_MAX`. The tests do not establish that the AEAD
construction accepts a plaintext of that length, they do not run
ChaCha20, and they do not choose a nonce. They also do not establish
the width of the length field in §2.8.1. That width is 8 in the
June 2018 pseudocode and 4 in the May 2015 pseudocode, which is
erratum 4371, and neither test mentions it.

### J2.5 A constant both editions print

Section 2.3 of each RFC initializes the ChaCha20 state. Both files
print the same four constant words, in the same order:

```text
0x61707865, 0x3320646e, 0x79622d32, 0x6b206574
```

N12 already showed that these words are the little-endian readings
of the sixteen ASCII bytes of `expand 32-byte k`. This lesson does
not repeat that derivation as a new one. It uses the result.
Proposition N12.4 is available. The bytes, in the order a reader of
the sentence meets them, are

```text
65 78 70 61  6e 64 20 33  32 2d 62 79  74 65 20 6b
```

A little-endian word takes the first byte of each group as the
least significant byte. Listing J2.2 builds the four words that
way and compares them with the hex the RFCs print. The comparison
is the assembly. It is not a search of either RFC file.

**Listing J2.2 — `constants.or`**

```orange
edition 2026;
module constants {
  spec pack(b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8]) -> Word[32] {
    (b0 as Word[32]) | ((b1 as Word[32]) << 8) | ((b2 as Word[32]) << 16)
      | ((b3 as Word[32]) << 24)
  }
  spec word0() -> Word[32] { pack(0x65, 0x78, 0x70, 0x61) }
  spec word1() -> Word[32] { pack(0x6e, 0x64, 0x20, 0x33) }
  spec word2() -> Word[32] { pack(0x32, 0x2d, 0x62, 0x79) }
  spec word3() -> Word[32] { pack(0x74, 0x65, 0x20, 0x6b) }
  test "RFC 8439 2.3 and RFC 7539 2.3 constant words" {
    (word0() == 0x61707865) && (word1() == 0x3320646e)
      && (word2() == 0x79622d32) && (word3() == 0x6b206574)
  }
}
```

`pack` is the little-endian assembly N12 used for a ChaCha word:
the first argument is the low byte. `<<` moves a byte up by eight
bits, and `|` combines the four pieces. The four calls pass the
ASCII bytes of `expa`, `nd 3`, `2-by`, and `te k`. Assumption J2.8
is this order. The listing does not also try the high byte first.
N12 already recorded what that swap produces: the letters `apxe`,
which is not the sentence.

```sh
./compiler/target/debug/orangec check constants.or
./compiler/target/debug/orangec eval constants.or
./compiler/target/debug/orangec test constants.or
```

Check is silent. The status is 0.

**Expected evaluation output:**

```text
constants::word0: Word[32] = 0x61707865
constants::word1: Word[32] = 0x3320646e
constants::word2: Word[32] = 0x79622d32
constants::word3: Word[32] = 0x6b206574
```

**Test report:**

```text
test "RFC 8439 2.3 and RFC 7539 2.3 constant words" ... ok
1 test: 1 passed, 0 failed
```

The test passed. The title names both RFCs. The pass is compatible
with either citation, because both documents print those four
words. A passing test of an unchanged constant does not choose the
edition. That is outcome 5 of the finish line, on this example.

The same section of the two RFCs does not agree in every sentence.
RFC 7539 §2.3 says the nonce “should not be repeated for the same
key.” RFC 8439 §2.3 says the nonce “MUST not be repeated for the
same key.” The word `MUST` is in capitals. The word `not` is not.
Section 1.1 of RFC 8439 says the key words, including `MUST NOT`,
are interpreted as in BCP 14 “when, and only when, they appear in
all capitals, as shown here.” The sentence in §2.3 does not contain
the all-capitals phrase `MUST NOT`. It contains `MUST not`. Listing
J2.2 does not mention a nonce. Its passing test does not detect the
change of wording, and it does not decide whether `MUST not` is the
BCP 14 phrase. A reader who treats the passing constant test as a
reading of §2.3 has used outcome 4 and skipped outcome 5.

`algorithms/chacha20/chacha20.or` pins RFC 8439 in its header,
including the year 2018 and the URL of that RFC, and it places the
same four words in `initial_state`. The header is the pin. The four
words, taken alone, are not, for the reason Listing J2.2 just
showed. The file's tests are named by RFC 8439 section numbers.
That naming is a second part of the pin, and §J2.9 reads it. The
block function itself stays in N12. This lesson does not copy it.

### J2.6 A value copied from the other edition

Listing J2.3 computes the same product as Listing J2.1. The test
title names RFC 8439 §2.8. The expected integer is the one RFC 7539
prints. The title and the integer disagree. The compiler can see
the integer. It cannot see the title's claim about a document.

**Listing J2.3 — `wrong_edition.or`**

```orange
edition 2026;
module wrong_edition {
  spec blocks() -> Int { 4294967296 - 1 }
  spec bytes() -> Int { blocks() * 64 }
  test "RFC 8439 2.8 copied from RFC 7539" { bytes() == 247877906880 }
}
```

```sh
./compiler/target/debug/orangec check wrong_edition.or
./compiler/target/debug/orangec eval wrong_edition.or
./compiler/target/debug/orangec test wrong_edition.or
```

Check is silent. The false pin is still well-formed. That is the
same separation N8 recorded: a silent check is not a true test.

**Expected evaluation output:**

```text
wrong_edition::blocks: Int = 4294967295
wrong_edition::bytes: Int = 274877906880
```

`eval` does not apply the test. The product is Proposition J2.1
either way. Changing the expected integer does not change `bytes`.

**Test report:**

```text
test "RFC 8439 2.8 copied from RFC 7539" ... FAILED
    left:  274877906880
    right: 247877906880
1 test: 0 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. There is no
`ORC` code. `left` is the value `bytes()` denotes. `right` is the
integer written in the test. Left is the June 2018 total. Right is
the May 2015 total. The gap is Proposition J2.2.

**Proposition J2.3.** The `Bool` in Listing J2.3 is false because
the expected integer is `247877906880` and `bytes()` denotes
`274877906880`. Replacing the expected integer with `274877906880`
makes the `Bool` true. No change to `blocks` or `bytes` is required.

*Proof.* `blocks` denotes `4294967296 - 1`, and `bytes` denotes
that integer times `64`, which Proposition J2.1 puts at
`274877906880`. The test compares that value with `247877906880`.
Proposition J2.2 says those integers differ by `27000000000`, so
they are not equal, and the `Bool` is false. Substituting the June
2018 integer makes the two sides the same value, so `==` denotes
true. The substitution is in the test. The specs are untouched. □

The minimal repair is that one integer. Rewriting the
multiplication would answer a left value that disagreed with
Proposition J2.1. Left agrees. The failure report is how you know
which side disagreed. A prose book can tell you the two editions
differ. This report is the compiler rejecting the file that mixed
them.

The repair, once made, is Listing J2.1's first test. That repaired
test still does not establish the length-field width, the nonce
sentence, or the AEAD construction. Outcome 5 applies to the
repaired test exactly as it applied to the constant words. A pass
after a repair establishes the repaired `Bool`. It does not reach
back and establish the sentences the test never mentioned.

### J2.7 FIPS 180-4 is a date, and §6.2.2 is not this page

FIPS PUB 180-4, *Secure Hash Standard (SHS)*, prints `August 2015`
on its cover. The DOI on that cover is `10.6028/NIST.FIPS.180-4`.
The announcement in the same publication says the standard
supersedes FIPS 180-3. Appendix B of FIPS 180-4 dates that
predecessor as October 2008. The file of FIPS 180-3 retrieved for
this lesson carries an archive wrapper that prints a third date,
“superseded on March 6, 2012,” and points at the publications page
for FIPS 180-4. Three dates are now in play. The pin this lesson
uses for the standard itself is the date the standard prints on
its own cover, August 2015. The wrapper's date is a date on a
wrapper. It is not a sentence of the October 2008 text, and it is
not the cover date of the August 2015 text.

The contents of the two publications agree on two addresses this
book has already used. In the October 2008 contents, §5.3.3 is
SHA-256 and §6.2.2 is SHA-256 hash computation. In the August 2015
contents, those same digits name those same clauses. A reader who
expects every revision to renumber every section will “correct”
a citation that did not move. Assumption J2.4 still requires the
edition, because agreement of two addresses is not agreement of
the documents. The August 2015 contents add §5.3.6, §6.6, and
§6.7 for SHA-512/t, SHA-512/224, and SHA-512/256, which the
October 2008 contents do not list. Section 5.2 is titled “Parsing
the Padded Message” in October 2008 and “Parsing the Message” in
August 2015. The digits `5.2` survived. The title did not. A
citation that gives the digits and omits the edition has not said
which title it means.

Appendix C of the August 2015 publication is “Technical Changes
from FIPS 180-3.” Item 2 says FIPS 180-4 adds SHA-512/224 and
SHA-512/256, which matches the new sections. Item 1, on the same
page, reads:

> In FIPS 180-3, padding was inserted before hash computation begins. FIPS 140-4 removed this restriction.

The digits `140` are in the content stream of that page of the
DOI PDF, in the sentence whose neighbors are FIPS 180-3 and, in
the next item, FIPS 180-4. The sentence is about when padding may
be inserted. The identifier it prints is `140-4`. This lesson does
not replace those digits with `180`. A silent repair would be a
new pin, and the page would no longer be the page that was read.
A reader who goes and fetches FIPS 140-4 because this sentence
names it has followed the printed identifier. That is a different
document from the one the paragraph is discussing. The
disagreement is inside one edition, between the identifier and
the work the sentence is doing. Outcome 6 includes that kind of
disagreement. No Orange test in this lesson executes padding, so
no test here can catch the identifier. The catch is the reading.

The publication also carries an erratum page. The table has one
row. The date is `5/9/2014`. The type is Editorial. The change is
from `t < 79` to `t ≤ 79`, at page 10, §4.1.1, line 1. The row
says the change has been incorporated. Section 4.1.1 of the August
2015 text is the SHA-1 functions, and the bound it prints on `t`
is `0 ≤ t ≤ 79`. This lesson does not transcribe those functions.
The fact it needs is the status of the row: the erratum is already
in the August 2015 text. Applying it a second time, as if the
cover still printed `t < 79`, would edit a text that has already
been edited. The date on the erratum, May 9, 2014, is not the
cover date, August 2015, and it is not the Orange edition token.

Section 5.3.3 is the clause this lesson does open. It says that
for SHA-256, the initial hash value `H(0)` shall consist of eight
32-bit words, in hex. The first of them is printed `6a09e667`. The
sentence under the eight words says they were obtained by taking
the first thirty-two bits of the fractional parts of the square
roots of the first eight prime numbers. The first prime is 2. The
first word is therefore the first thirty-two bits of the
fractional part of `sqrt(2)`.

Section 6.2.2 is the SHA-256 hash computation. Its first
preprocessing sentence points back: set `H(0)` as specified in
§5.3.3. The computation itself is the message schedule and the
compression function. Those are J5. Assumption J2.7 stands. The
address §6.2.2 may be written down. The functions may not.

The same eight hex digits begin a different word in a different
subsection. Section 5.3.5 prints the SHA-512 initial hash value in
64-bit words. Its first word is printed `6a09e667f3bcc908`. The
first thirty-two bits match §5.3.3. The word is twice as wide.
A pin that copies `6a09e667` and does not say “32-bit” and does
not say “§5.3.3” has not said whether it stopped on purpose.

`algorithms/sha2/sha2.or` names `initial_hash_256` as §5.3.3 and
places `0x6a09e667` first in that array. The header of the file
dates the standard to August 2015 and gives the DOI. That header
is a pin of the edition. The single word `0x6a09e667`, in
isolation, is also the leading half of the §5.3.5 word. The file
distinguishes them by the section comment and by the width of the
array element, `Word[32]` against `Word[64]`. A test of the 32-bit
word does not test the 64-bit word. This lesson derives the
32-bit word and does not open the file's compression function.

The derivation uses integer arithmetic only. Let `n = 2^65`. The
integer square root of `n` is the unique non-negative integer `r`
such that `r^2 ≤ n < (r + 1)^2`. Then `floor(sqrt(2) * 2^32) = r`,
because `sqrt(2) * 2^32 = sqrt(2 * 2^64) = sqrt(2^65)`. The
fractional part's first thirty-two bits are `r - 2^32`, provided
`r` lies in the next binade, which the value below does.

One recurrence produces a candidate. Start at `x0 = 2^33 =
8589934592`. The step is

`x → (x + floor(n / x)) / 2`,

with division the Euclidean division of non-negative integers,
which on this compiler is `/` for positive `Int` values.

**Proposition J2.4.** The first step is exactly `6442450944`.

*Proof.* `floor(2^65 / 2^33) = 2^32 = 4294967296`, with no
remainder, because `2^65 = 2^33 * 2^32`. The sum is
`2^33 + 2^32 = 3 * 2^32 = 12884901888`. Half of that is
`3 * 2^31 = 6442450944`. □

The listing applies the step six times. The value before any
step, and the value after each step, are

```text
0  8589934592
1  6442450944
2  6084537002
3  6074010122
4  6074000999
5  6074000999
6  6074000999
```

Row 0 is the start. Row 1 is Proposition J2.4. Rows 4, 5, and 6
agree, so the step has stopped changing the value. The `for`
runs six steps and therefore denotes row 6. Call that value
`r = 6074000999`.

**Proposition J2.5.** If `r` is a positive integer and
`floor(n / r) = r + 1`, then `r^2 ≤ n < (r + 1)^2`.

*Proof.* `floor(n / r) = r + 1` means `r * (r + 1) ≤ n` and
`n < r * (r + 2)`. The first inequality is `r^2 + r ≤ n`, so
`r^2 ≤ n`. The second is `n ≤ r^2 + 2r - 1`, because the largest
integer strictly below `r * (r + 2) = r^2 + 2r` is
`r^2 + 2r - 1`. Then `n ≤ r^2 + 2r - 1 < r^2 + 2r + 1 = (r + 1)^2`.
So `n < (r + 1)^2`. □

The hypothesis of Proposition J2.5, for this `n` and this `r`, is
`floor(2^65 / 6074000999) = 6074001000`. Listing J2.4 computes
that quotient and computes `r - 2^32`.

**Proposition J2.6.** `6074000999 - 2^32 = 1779033703`, and
`1779033703` is the hex word `6a09e667`.

*Proof.* `2^32 = 4294967296`, and
`6074000999 - 4294967296 = 1779033703`. The hex digits of that
integer are computed by remainders on division by 16, from the
low digit: the remainders are `7, 6, 6, e, 9, 0, a, 6`, so the
word printed high digit first is `6a09e667`. □

The eight remainders are an exercise. The listing checks the
integer, not each remainder. A reader who wants the hex from the
decimal does the remainders. A reader who wants the decimal from
the standard's hex expands `6 * 16^7 + 10 * 16^6 + 0 * 16^5 +
9 * 16^4 + 14 * 16^3 + 6 * 16^2 + 6 * 16 + 7`.

**Listing J2.4 — `iv.or`**

```orange
edition 2026;
module iv {
  spec root() -> Int {
    for i in 0..6 with x: Int = 8589934592 {
      (x + (36893488147419103232 / x)) / 2
    }
  }
  spec word() -> Int { root() - 4294967296 }
  spec stable_quot() -> Int { 36893488147419103232 / 6074000999 }
  test "FIPS 180-4 5.3.3 first word" { word() == 0x6a09e667 }
  test "fixed point quotient is one more than the root" {
    stable_quot() == 6074001000
  }
}
```

`36893488147419103232` is `2^65`. The parentheses around the
division are required. `+` and `/` are in different operator
groups, and this compiler rejects the ungrouped spelling with
`ORC0108`. The `for` runs the integer literals `0` through `5`,
six steps, which is the list above. The index `i` is not read.
The bound is what the standard's “first thirty-two bits” becomes
once the square root has been replaced by this recurrence. The
recurrence is not in FIPS 180-4. The publication states the
mathematical description. The recurrence is a way to compute the
integer that description names. A different recurrence that
reached a different `r` would be a different computation, and
Proposition J2.5 would not apply to it unless its quotient
hypothesis held.

```sh
./compiler/target/debug/orangec check iv.or
./compiler/target/debug/orangec eval --spec word iv.or
./compiler/target/debug/orangec eval --spec stable_quot iv.or
./compiler/target/debug/orangec test iv.or
```

Check is silent. The status is 0.

**Expected evaluation output:**

```text
iv::word: Int = 1779033703
```

```text
iv::stable_quot: Int = 6074001000
```

**Test report:**

```text
test "FIPS 180-4 5.3.3 first word" ... ok
test "fixed point quotient is one more than the root" ... ok
2 tests: 2 passed, 0 failed
```

The first test uses the hex literal `0x6a09e667`, which is how
§5.3.3 prints the word. The report of a failure would print the
decimal, as the next listing shows. The pass means `word()`
denotes `1779033703` and that this equals the hex literal. Together
with Proposition J2.5 and the quotient test, it means that integer
is `r - 2^32` for an integer square root of `2^65`. It does not
mean the other seven words of §5.3.3 were computed. It does not
mean a message was hashed. It does not mean §6.2.2 was read. It
does not choose August 2015 over October 2008 by itself, because
this lesson did not show that the October 2008 printing of §5.3.3
differs in this word. The edition is pinned by the citation around
the test, not by a digit the two editions might share. That is the
constant-word lesson of §J2.5, applied to a FIPS word.

### J2.8 The neighboring section is the wrong pin

Section 5.3 of FIPS 180-4 is “Setting the Initial Hash Value.”
Section 5.3.1, the first algorithm in that section, is SHA-1. Its
first word is printed `67452301`. Section 5.3.3 is SHA-256. A
reader who takes “the first initial-hash word in §5.3” has cited
a section that contains several words, and the first one in the
section is the SHA-1 word. The role is the same kind of role, an
initial hash word. The section is not the same section.
Assumption J2.5 says a right role in the wrong section is a
different pin.

`0x67452301` is the decimal `1732584193`. Listing J2.5 computes
the SHA-256 word and demands the SHA-1 word.

**Listing J2.5 — `wrong_section.or`**

```orange
edition 2026;
module wrong_section {
  spec root() -> Int {
    for i in 0..6 with x: Int = 8589934592 {
      (x + (36893488147419103232 / x)) / 2
    }
  }
  spec word() -> Int { root() - 4294967296 }
  test "FIPS 180-4 5.3.3 copied from 5.3.1" { word() == 0x67452301 }
}
```

```sh
./compiler/target/debug/orangec check wrong_section.or
./compiler/target/debug/orangec eval --spec word wrong_section.or
./compiler/target/debug/orangec test wrong_section.or
```

Check is silent. Evaluation prints the SHA-256 word, because
`eval` does not run the test:

```text
wrong_section::word: Int = 1779033703
```

**Test report:**

```text
test "FIPS 180-4 5.3.3 copied from 5.3.1" ... FAILED
    left:  1779033703
    right: 1732584193
1 test: 0 passed, 1 failed
```

The status is 1. Standard error is empty. Left is `0x6a09e667`,
the word §5.3.3 prints and Listing J2.4 derives. Right is
`0x67452301`, the word §5.3.1 prints. The title already confesses
the copy. A title that said only `FIPS 180-4 5.3.3 first word`
would fail with the same left and right. The diagnosis would be
the same arithmetic, and the reader would still have to notice
that the right-hand hex is the other subsection. The compiler
prints integers. It does not print “you opened §5.3.1.”

**Proposition J2.7.** The `Bool` in Listing J2.5 is false, and the
repair is the expected hex `0x6a09e667`. The recurrence stays.

*Proof.* Listing J2.4 already shows that this recurrence and this
subtraction denote `1779033703`, equal to `0x6a09e667`. The test
demands `0x67452301`, which is `1732584193`. The report's two
integers differ, so the `Bool` is false. Substituting the §5.3.3
hex makes the sides equal. The body of `root` is not the site of
the disagreement. □

The repaired test is the first test of Listing J2.4. It still
does not hash a message, and it still does not transcribe §6.2.2.

### J2.9 FIPS 197, and a figure number that moved

FIPS 197, *Advanced Encryption Standard (AES)*, was published
November 26, 2001. The May 9, 2023 update is NIST FIPS 197-upd1.
Its cover prints both dates: “Published November 26, 2001; Updated
May 9, 2023.” The DOI of the update is
`10.6028/NIST.FIPS.197-upd1`. The archived PDF of the 2001 text
carries a withdrawal notice. The notice prints the withdrawal date
May 9, 2023, names NIST FIPS 197-upd1 as the superseding
publication, and says: “This update makes no technical changes to
the algorithm specified in the original (2001) release of this
standard. This update includes extensive editorial improvements
to the original version.”

“No technical changes to the algorithm” is a claim about the
algorithm. It is not a claim that every section number, figure
number, and sentence survived. Appendix D of the update is the
change log. Item 5 says the material in the previous §2.2,
“Algorithm Parameters, Symbols and Functions,” was split into two
new sections: §2.2, “List of Functions,” and §2.3, “Algorithm
Parameters and Symbols.” The 2001 contents have one section 2.2
with that older title, and they do not have a §2.3 under
“Definitions.” A citation of “FIPS 197 §2.2” that omits the date
names a different clause in the two editions. That is Assumption
J2.4 on a split rather than on a reuse of the same title.

The figure numbers move further. The 2001 list of figures says
Figure 7 is “S-box: substitution values for the byte xy (in
hexadecimal format).” The 2023 list of figures says Figure 7 is
the illustration of `KEYEXPANSION()` for AES-192. The S-box in the
2023 text is Table 4, “SBOX(): substitution values for the byte
xy (in hexadecimal format).” The first entry of that table, at
row `0` and column `0`, is `63`. The 2001 Figure 7 begins with the
same bytes `63 7c 77 7b`. The withdrawal notice and the matching
first bytes are consistent with each other. They do not make
Figure 7 a stable name. In 2001, Figure 7 is the S-box. In 2023,
Figure 7 is a picture of the AES-192 key expansion, and the S-box
is Table 4.

`algorithms/aes/aes.or` records both locators in one comment: the
S-box is “Section 5.1.1, Table 4 (Figure 7 of the 2001 text).”
That parenthetical is a pin of the old figure to the new table.
It is the right shape. A comment that said only “Figure 7” would
be the wrong shape, and a test that checked the byte `0x63` would
still pass, because the byte did not move when the figure number
did. This lesson does not add a listing whose body is the literal
`0x63` compared with the literal `0x63`. That comparison would
establish that a literal equals itself. It would not establish
which figure you meant. The constant-word test of §J2.5 already
taught the general fact. The S-box byte is the same fact with a
locator that did move.

Appendix D item 17 of the update says the description of
`INVSHIFTROWS()` “in Section 5.3.2” was improved, and that a
mistake in it was corrected. The contents of the same PDF list
`INVSHIFTROWS()` at §5.3.1 and `INVSUBBYTES()` at §5.3.2. The 2001
contents use those same two numbers for those same two names.
The changelog's digits `5.3.2`, attached to `INVSHIFTROWS()`, do
not match the contents of either edition. This lesson does not
re-derive the inverse shift, and it does not guess which formula
was the mistake. It records that a change log can cite a section
number the contents do not assign to the name in the same
sentence. A reader who opens §5.3.2 of the 2023 text because item
17 said to will open `INVSUBBYTES()`. The repair is to follow the
contents' number for the name, and to treat item 17's digits as a
locator that failed Assumption J2.4 inside a single PDF.

Item 23 of the same appendix says the examples in Appendix C were
removed in favor of a reference to example vectors maintained
elsewhere. A vector copied from Appendix C of the 2001 text into
a test titled as Appendix C of the 2023 update is a copy from an
appendix the update says it removed. `aes.or` says it reproduces
the cipher examples of Appendix C.1, C.2, and C.3 and the
round-by-round example of Appendix B, and its header cites the
2001 text and the 2023 DOI together. Whether those examples still
sit in the 2023 Appendix C is a question the header's pair of
dates forces you to ask. This lesson does not answer it by
transcribing a round. J5 is not AES either. The question is the
pin: which edition's appendix, and is the appendix still in that
edition.

### J2.10 What the repository records, and what it does not

Three files are enough to see the habit on the code that is
already in the tree.

`algorithms/chacha20/chacha20.or` opens by naming RFC 8439, the
year 2018, and the URL of that RFC. It says which sections it
follows: 2.1, 2.3, 2.4, and the sections of the separate
XChaCha20 draft it also implements. The draft is a fourth
document. A test that passes against an RFC 8439 vector does not
pass the draft, and a test that passes against the draft does not
become an RFC 8439 test by sharing the quarter round. The file's
`initial_state` uses the four constant words of Listing J2.2 and
a 32-bit counter. The comment above that spec says the counter is
the 32-bit word of RFC 8439, and that Bernstein's ChaCha of 2008
kept a 64-bit counter in words 12 and 13. That sentence is a pin
of layout. Listing J2.2 does not check it. N12 checked the RFC
8439 layout against §2.3.2. Neither check is a check of the 2008
widths.

The README table in `algorithms/chacha20/README.md` names each
Orange spec by an RFC 8439 section: `rfc8439_2_1_1` for §2.1.1,
`rfc8439_2_3_2` for the serialized block of §2.3.2, and the
appendix A rows for those vectors. The name is a locator only if
it matches the section the expected bytes were copied from. A
name `rfc8439_2_1_1` on a vector copied from §2.3.2 would be
Listing J2.5's mistake in miniature: right document, wrong
section, and a title that can lie. The compiler sees bytes. The
table is the human record.

`algorithms/aes/aes.or` opens on FIPS 197, “2001; update 1, 2023,”
with the 2023 DOI. The S-box comment is the dual locator of
§J2.9. The file is a reference evaluation of the examples it
names. It is not a certification, and the entry's own README says
the tables are the standard's tables read by a selection over all
of them, with no constant-time claim. This lesson adds none.

`algorithms/sha2/sha2.or` opens on FIPS 180-4, August 2015, with
the DOI, and it lists §5.3 among the sections it follows. It also
lists §6.2 and §6.4, which are the hash computations. Those
functions are in that file. They are not in this lesson. Pointing
at `initial_hash_256` is a pin of one array to §5.3.3. Opening
`schedule` or `compress` would be J5. The header also says some
digests were cross-checked with Python's `hashlib` and with other
libraries' vector files. A cross-check against another
implementation is not the standard. It is a second artifact. If
the other artifact was itself transcribed from a different
edition, the cross-check repeats the wrong pin twice. Listing
J2.4 does not call another library. Its expected hex is the hex
§5.3.3 prints, and its left value is the recurrence.

The Gate 0 fixture
`conformance/foundation/valid/standards-provenance.json` is a
different kind of record, and it says so. Its `record_status` is
`provisional_gate0`. Its `non_product` field is true. The one
standard inside it is RFC 8785, dated `2020-06-01`, and the digest
value is sixty-four repetitions of the character `a`. The
`archive_state` is `acquisition_required`. The limitations array
says the fixture demonstrates provenance shape, that its digest
and retrieval time are synthetic, and that they are not a verified
standards acquisition. A reader who copies `aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`
into a test titled as the digest of RFC 8785 has pinned the
fixture's placeholder. The placeholder is not a document.

The fields the fixture is exercising are the fields a real pin
needs: an issuer, a document identifier, an edition, a publication
date, a source URI, a retrieval time, a digest, an errata list, a
clause locator, and a statement of what the record does not claim.
For the integer in Listing J2.1, filled from the files this lesson
retrieved on 2026-10-05, the record is the following. It is a
record in this chapter. It is not a row of the Gate 0 fixture,
and it does not make that fixture's digest real.

The issuer is the Internet Research Task Force, and the plain
text is published by the RFC Editor. The document identifier is
RFC 8439. The edition is the June 2018 plain-text file, whose
header prints `Obsoletes: 7539`. The URI is
`https://www.rfc-editor.org/rfc/rfc8439.txt`. The SHA-256 digest
of the file retrieved that day is
`25bef70fbf7a07ff45c2fe4cb7c6ce954eac687413d8610603268b4e4415324c`.
The clause is §2.8, both the sentence that gives the total of
`(2^32 - 1)` blocks of 64 bytes and the `P_MAX` bullet that prints
the same integer. The role is the output of that multiplication.
The errata consulted are 4858 and 4861, both marked Verified on
the errata page, and the predecessor file of RFC 7539, whose
digest that day was
`546e12200dbbc7b08cf85c8f07ea8681be89954b7c7e0c56f0c51f775a0ba704`.
The August 2015 PDF of FIPS 180-4, DOI `10.6028/NIST.FIPS.180-4`,
had digest
`0455b406d89648d20cbde375561e19c245b9815e894164c2670772e3d54deb82`.
The May 9, 2023 PDF of NIST FIPS 197-upd1 had digest
`62c86eb567f13edb8f71826e985da870b04ef6381634f303cdb16e84d47becd1`.
The archived November 26, 2001 PDF, carrying the withdrawal
notice, had digest
`251dfe0b5dc283abaf364adf586f7ec6dc4e495335d48dd5ee0fee6c5961da8a`.

A digest is of a file, not of a sentence. The HTML and PDF
renderings of RFC 8439 are different byte strings. They can carry
the same §2.8 sentence and hash differently. The pin of the
sentence is the quotation and the section. The pin of the file is
the digest. Listing J2.1 tests the integer. It does not recompute
the file digest. A reader who changes one comma in a local copy
changes the digest and may leave the integer alone. Both facts
are then true, and they answer different questions.

### J2.11 What a passing test does not establish

Collect the refusals in one place. Each of them is an instance of
Assumption J2.6, named so it can be demanded again.

The test does not establish a document. The title string is not
looked up.

The test does not choose an edition when the expected integer is
one that two editions print. Listing J2.2 is that case. The S-box
byte `63` is that case across the 2001 text and the 2023 update.

The test does not see a sentence that changed while the integer
stood still. `should not` against `MUST not` in §2.3 is that
case. `MUST not` is not the all-capitals `MUST NOT` of §1.1.

The test does not apply an erratum the source never mentions.
Erratum 4371 changes a width from 4 to 8. Listings J2.1 and J2.3
never mention that width.

The test does not read a Held erratum or a Rejected one. Erratum
8274 is Rejected. It is not part of RFC 8439 merely because RFC
8439 says it merges errata.

The test does not transcribe FIPS 180-4 §6.2.2, and it does not
transcribe the AES round. The addresses are pins. The functions
are not on this page.

The test does not make `algorithms/sha2/sha2.or` or
`algorithms/aes/aes.or` a certification, a constant-time
implementation, or a corpus entry. Those files say so themselves.
This lesson does not withdraw what they say.

The test does not turn the Gate 0 fixture's synthetic digest into
an acquisition of RFC 8785.

The test does not verify the program. A passing report is a Match
on the inputs the test wrote. The word Verified, when it appears
in this lesson, is the RFC Editor's status on an erratum record.
It is not a name for the Match.

## Sources and epigraph record

The quotation is the borrowed sentence. The dates and identifiers
in the opening are the publication's own cover lines, checked
against the files named here. Later sections quote clauses. They
do not replace this record.

**[S13] Yoav Nir and Adam Langley.** RFC 8439, “ChaCha20 and
Poly1305 for IETF Protocols,” Internet Research Task Force,
Informational, June 2018. The header line is `Obsoletes: 7539`.
The epigraph is one sentence of the Abstract, the sentence that
begins “RFC 7539, the predecessor of this document,”. It is the
second paragraph of the Abstract. The following sentence, not used
as the epigraph, says that this document merges the errata filed
against RFC 7539 and adds a little text to the Security
Considerations section. Wording was checked on 2026-10-05 against
the plain-text RFC at the RFC Editor. The sentence contains no
inner quotation. No translation is involved. The sentence is not
an endorsement of Orange. It records that a document written to be
a stable reference has a predecessor, and that the predecessor was
replaced.
This record's tag is [S13].

Source: <https://www.rfc-editor.org/rfc/rfc8439.txt>

The authors' names are the names in the RFC's “Authors' Addresses”
section: Yoav Nir and Adam Langley. The cover also prints `Y. Nir`
and `A. Langley`.

## Evidence boundary

J2 is a Journeyman lesson. The six outcomes in §J2.1 are the finish
line. The assumptions in §J2.2 bound them. Listings J2.1 through
J2.3 pin RFC 8439 against RFC 7539. Listings J2.4 and J2.5 pin
FIPS 180-4 §5.3.3 and show the neighboring section failing.
Worked answers close the lesson. Those checks do not establish a
cryptographic security claim, they do not derive FIPS 180-4
§6.2.2, and they do not accept a proposal.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction. Owner review is pending. No deployment
recommendation is made.
