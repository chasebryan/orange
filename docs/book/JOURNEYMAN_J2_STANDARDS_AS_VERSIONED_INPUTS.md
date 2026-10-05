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
J2.3 are the RFC pin, the unchanged constant, and the edition
mismatch. Later sections pin FIPS 180-4 and FIPS 197, and the
worked answers close the lesson. Those checks do not establish a
cryptographic security claim, they do not derive FIPS 180-4
§6.2.2, and they do not accept a proposal.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction. Owner review is pending. No deployment
recommendation is made.
