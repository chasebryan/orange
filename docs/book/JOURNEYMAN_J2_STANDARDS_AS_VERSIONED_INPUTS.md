# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J2: Standards as Versioned Inputs. Draft 2026-10-05. Revised 2026-10-09.

Continue from
[Ready for Standards](NOVICE_N14_READY_FOR_STANDARDS.md#n14-ready-for-standards).
The gate asked you to carry five assumptions into Block A: the edition
pin, the byte order, the padding, the module seam, and the non-claims.
This lesson takes the first of those and makes it a procedure. The
reading habit is still
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
A passing test is still the Match of
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study)
and of N14: one `Bool`, on the inputs the test wrote. N13 already
pinned two expected values this lesson will name again: the NIST
SHA-256 digest of “abc”, and one HMAC-SHA-256 line of RFC 4231. J5
is still the study that derives the SHA-256 compression function and
its message schedule from FIPS 180-4 §6.2.2. This lesson cites that
section as an address. It does not transcribe the function, and it
does not re-derive HMAC. The HMAC program is N13.

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

The sentence is the first sentence of the Abstract's second paragraph.
It names a purpose and a predecessor. The next sentence
of that paragraph says the predecessor was a product of the Crypto
Forum Research Group (CFRG). The third sentence of that paragraph
says this document merges the errata filed against RFC 7539 and
adds a little text to the Security Considerations section. RFC 7539, dated May 2015, was
written to be the stable reference. RFC 8439, dated June 2018,
obsoletes it. A document that was meant to hold still was replaced,
and the replacement says so in the second paragraph of the Abstract,
not in the first. The first paragraph defines the algorithms. The
name “ChaCha20 and Poly1305 for IETF Protocols” is the title of
both files. The title does not say which file you have.

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

### J2.1 Why the version matters more than the vibe

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
   clause in another edition, or a different figure in an update of
   the same publication.
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
   wrong edition or a section number that does not denote the clause
   you meant, from the failing test the compiler prints when the
   integers differ, and from the citation when they do not. Missing
   that diagnosis means the pin is still a name.

The documents the lesson actually opens are these.

RFC 7539, *ChaCha20 and Poly1305 for IETF Protocols*, Y. Nir and
A. Langley, May 2015. RFC 8439, the same title, Y. Nir and
A. Langley, June 2018, which obsoletes RFC 7539. The errata filed
against RFC 7539. RFC 4231, *Identifiers and Test Vectors for
HMAC-SHA-224, HMAC-SHA-256, HMAC-SHA-384, and HMAC-SHA-512*,
M. Nystrom, December 2005. FIPS PUB 180-4, *Secure Hash Standard
(SHS)*, August 2015. FIPS 180-2, the hash standard RFC 4231 names.
FIPS 197, *Advanced Encryption Standard (AES)*, November 26, 2001,
and the May 9, 2023 update. NIST's SHA-256 examples, for the
one-block digest of “abc”. N13 already ran the programs that
compute that digest and one RFC 4231 tag. This lesson does not run
them again.

A title is a vibe. “ChaCha20 and Poly1305 for IETF Protocols” is
the title of RFC 7539 and of RFC 8439. “Secure Hash Standard” is
the title of more than one FIPS publication in that series.
“SHA-256” is the name of an algorithm those publications specify
and the name RFC 4231 uses while its normative reference points at
a different edition from the one N13 pinned. The version is the
identifier together with the date, and, when a clause is at issue,
the section number inside that edition.

RFC 7539 and RFC 8439 do not share a date, and they do not share a
number. The plain-text file of RFC 7539 prints `May 2015` on its
cover and does not print an `Obsoletes` line. The plain-text file
of RFC 8439 prints `June 2018` and prints `Obsoletes: 7539`. The
pin is the number together with the date.

The RFC Editor keeps a third object: the errata filed against RFC
7539. An erratum has its own number, a status, a reporter, and a
date. The status is a word in that record. It is not a property of
an Orange test.

A **Verified** erratum is one the RFC Editor has marked Verified.
That mark is the editor's, on that record. It does not make an
Orange Match verified.

A record **Held for Document Update** is not the same mark. It is
waiting. Merging it into a later RFC is a further act, and the
third sentence of the Abstract's second paragraph does not perform
that act for you.

A **Rejected** erratum is a record the editor declined. It is not
applied. Reading it as if it had been applied changes the pin.

Four Verified errata are on the record retrieved on 2026-10-05.
Two of them change a printed integer. One changes the width of a
length field in pseudocode. One changes a description of an output
from two results into one concatenation.

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
The note points at RFC 5116 §2.1. This lesson does not transcribe
that interface. The erratum is here because it is a change of
shape, not of an integer, and a test of `P_MAX` does not see it.

The third sentence of the Abstract's second paragraph is the
authors' summary that the document merges the errata filed against
RFC 7539. It is not a substitute for putting each erratum next to
the new text.
The check that belongs here is a count of the disputed integers in
the two plain-text files.

In the RFC 7539 file, the digit string `247,877,906,880` occurs
twice, and `274,877,906,880` occurs zero times. In the RFC 8439
file the counts are reversed. The same swap holds for
`247,877,906,896` and `274,877,906,896`: one occurrence in RFC
7539, and the other integer in RFC 8439. `num_to_4_le_bytes` occurs
in RFC 7539 and does not occur in RFC 8439. `num_to_8_le_bytes`
occurs in RFC 8439 and does not occur in RFC 7539.

Those counts were taken on 2026-10-05 from the plain-text files at
the RFC Editor. They say what those files contain. They do not say
that every Held erratum was merged, because this lesson did not
place a Held erratum beside the new text. A later reader who needs
one of those still has to do that placement.

One Rejected erratum is on the same page: erratum 8274, against
§2.3.2, marked Rejected. A Rejected record is not a correction.
Citing RFC 7539 “with the errata” without saying which statuses
were applied has mixed the three objects.

The two places RFC 8439 prints `274,877,906,880` are not the same
sentence. One is the note in §2.8 that computes `(2^32 - 1)` blocks
of 64 bytes. The other is the limit list in the same section:
`P_MAX` is that many bytes. This lesson uses the arithmetic the
note states. It does not open RFC 5116.

The note says the amount of encrypted data possible in a single
invocation is `2^32 - 1` blocks of 64 bytes each, because of the
size of the block counter, and that this gives a total of
`274,877,906,880` bytes. The same paragraph in RFC 7539 prints
`247,877,906,880` bytes for the same description of the product.
The description agrees. The printed total does not. The role of
the integer is an output of that multiplication, not an input to
the cipher.

**Proposition J2.1.** `(2^32 - 1) * 64 = 274877906880`.

*Proof.* `64 = 2^6`, and `2^32 - 1` is the integer one below `2^32`.
Multiplying by `2^6` shifts that difference:

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

Erratum 4858's note marks the same swap. Proposition J2.2 is that
observation as an integer identity. It does not depend on the note.
The note is the record that a reader filed the disagreement, and
that the editor marked the record Verified.

The same description, two printed totals: that is why the version
matters more than the vibe of the paragraph. A reader who remembers
“about 2^38 bytes, the block counter times 64” has the description
both files share and has not yet chosen a file.

A figure number can move while a byte stands still. FIPS 197 was
published November 26, 2001. The May 9, 2023 update, NIST FIPS
197-upd1, prints both dates on its cover and DOI
`10.6028/NIST.FIPS.197-upd1`. The archived 2001 PDF carries a
withdrawal notice dated May 9, 2023. The notice says the update
makes no technical changes to the algorithm specified in the 2001
release. That sentence is not a claim that every figure number
survived. In the 2001 text, Figure 7 is the S-box, and it begins
`63 7c 77 7b`. In the 2023 update, Figure 7 is the illustration of
`KEYEXPANSION()` for AES-192, and the S-box is Table 4, whose first
entry is `63`. Appendix D item 17 of the update says the
description of `INVSHIFTROWS()` “in Section 5.3.2” was improved.
The contents of that same PDF list `INVSHIFTROWS()` at §5.3.1 and
`INVSUBBYTES()` at §5.3.2. A test of the byte `0x63` would pass
under either figure. The byte did not move. The figure number did.
This lesson does not transcribe the AES round. The fact is the
address.

### J2.2 How to pin

Eight assumptions bound the lesson. A later sentence that needs a
further fact names it there.

**Assumption J2.1 — The edition token is not the standard's date.**
Every Orange listing begins with `edition 2026;`. That token is the
edition this compiler requires. It is not May 2015, not June 2018,
not August 2015, not December 2005, and not May 9, 2023. Changing
the token does not select a different RFC or a different FIPS
update. [T8]

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
The same digits can name a different clause in another edition, or,
inside one update, a different clause from the one a change log
names. The citation carries the edition, or it has not cited a
clause.

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
compression function, its message schedule, or HMAC.** The
compression function and the message schedule are FIPS 180-4
§6.2.2, and deriving them is J5. N13 transcribed the hash and the
HMAC composition and did not derive the round. This lesson
withholds the functions again. A section number may appear as a
pin. The functions do not appear on this page. HMAC is not
re-derived. The program that computes the tags is the one N13
already ran.

**Assumption J2.8 — The byte order of the ChaCha constants is N12,
not a second derivation.** N12 already showed that the four words
both RFCs print are the little-endian readings of the sixteen ASCII
bytes of `expand 32-byte k`. Proposition N12.4 is that result. This
lesson copies the four words. It does not pack them. J4 is the
study of byte order and format boundaries. Pointing at N12 for the
order is not that study.

The procedure is one card for one value.

The issuer is who published the file. The document identifier is
the RFC number or the FIPS number, not the title. The edition is
the date the file prints on its own cover, and the header lines
that say what it obsoletes or updates when it prints those lines.
A file that prints neither line is recorded as printing neither
line. The section is the clause that prints the value, or, when
the value is not in that publication, the other document that does
print it. The role is input, constant, or output. The last line of
the card says what a comparison of this value does not cover.

`edition 2026;` does not fill any of those fields. A test title
does not fill them either. The title is a string. The compiler
does not look it up. You do, in the prose around the listing.

The repository entries under `algorithms/` are reference
evaluations. They reproduce vectors the entry's README names. They
are not corpus entries in the manuscript's sense, they make no
constant-time claim, and a passing evaluation is not a
certification. `algorithms/chacha20/chacha20.or` names RFC 8439,
the year 2018, and the URL of that RFC, and it places the four
constant words in `initial_state`. The header is the pin. The four
words, taken alone, are not, because both RFCs print them. The
block function stays in N12. `algorithms/sha2/sha2.or` dates FIPS
180-4 to August 2015 and gives the DOI. That header is a pin of
the edition. This lesson does not open the file's compression
function. `algorithms/aes/aes.or` records the S-box as Section
5.1.1, Table 4, and notes that the table is Figure 7 of the 2001
text. That parenthetical is a pin of the old figure to the new
table.

The Gate 0 fixture
`conformance/foundation/valid/standards-provenance.json` records a
shape, and it says so. Its `record_status` is `provisional_gate0`.
Its `non_product` field is true. The one standard inside it is RFC
8785, dated `2020-06-01`, and the digest value is sixty-four
repetitions of the character `a`. The `archive_state` is
`acquisition_required`. The limitations array says the fixture
demonstrates provenance shape, that its digest and retrieval time
are synthetic, and that they are not a verified standards
acquisition. A synthetic digest is not the digest of the document
it names. The fields the fixture is exercising are the fields the
card above names: an issuer, a document identifier, an edition, a
publication date, a source URI, a retrieval time, a digest, an
errata list, a clause locator, and a statement of what the record
does not claim.

### J2.3 The fixture from the page

Four values are copied here. The listing in §J2.4 tests one of
them, the June 2018 total. The other three are the pins N13
already computed, or the constant both ChaCha RFCs print. Each
copy says what a passing comparison does not cover. None of these
copies is a transcription of HMAC or of §6.2.2.

**The NIST SHA-256 digest of “abc”.** N13's test
`FIPS 180-4 5.1.1 abc, NIST SHA-256 one-block sample` compares the
array `abc()` computes with the 32 bytes NIST prints for the
one-block message “abc”:

```text
BA7816BF 8F01CFEA 414140DE 5DAE2223 B00361A3 96177A9C B410FF61 F20015AD
```

The algorithm pin for that sample, as N13 records it and as this
lesson keeps it, is FIPS PUB 180-4, *Secure Hash Standard (SHS)*,
August 2015, DOI `10.6028/NIST.FIPS.180-4`. The announcement in
that publication says the standard supersedes FIPS 180-3. Appendix
B of FIPS 180-4 dates that predecessor as October 2008. The file of
FIPS 180-3 retrieved for this lesson carries an archive wrapper
that prints “superseded on March 6, 2012.” The pin of the standard
is the date on its own cover, August 2015. The wrapper's date is a
date on a wrapper.

Section 5.1.1 of the August 2015 text pads the 8-bit ASCII message
“abc”, whose bit length is `8 × 3 = 24`. The section prints the
padding diagram. It does not print the digest. The 32-byte line is
the one-block message sample of NIST's SHA-256 examples. That
examples file is not a section of FIPS 180-4. A card that cites
“FIPS 180-4 §5.1.1” as the source of `BA7816BF…` has named the
section that pads the message and has not named the file that
prints the digest. That is a wrong section. The repair is to split
the card: §5.1.1 for the padding, and the examples file for the
digest line. The bytes stay. The address changes.

The same leading hex digits begin a different word in a different
subsection, which is why the width belongs on the card.
Section 5.3.3 prints the SHA-256 initial hash value. Its first
32-bit word is `6a09e667`. Section 5.3.5 prints the SHA-512 initial
hash value. Its first word is `6a09e667f3bcc908`. Section 5.3.1
prints the SHA-1 initial hash value. Its first word is `67452301`.
The digest of “abc” is none of those words.

**Proposition J2.3.** The hex word `6a09e667` is the integer
`1779033703`.

*Proof.* Expand from the high digit. `16^2 = 256`, `16^3 = 4096`,
`16^4 = 65536`, `16^5 = 1048576`, `16^6 = 16777216`, and
`16^7 = 268435456`. The digits of `6a09e667` are `6`, `10`, `0`,
`9`, `14`, `6`, `6`, `7`:

`6 * 268435456 = 1610612736`,

`10 * 16777216 = 167772160`,

`0 * 1048576 = 0`,

`9 * 65536 = 589824`,

`14 * 4096 = 57344`,

`6 * 256 = 1536`,

`6 * 16 = 96`,

`7 * 1 = 7`.

The sum is `1610612736 + 167772160 = 1778384896`, then
`1778384896 + 589824 = 1778974720`, then
`1778974720 + 57344 = 1779032064`, then
`1779032064 + 1536 = 1779033600`, then
`1779033600 + 96 = 1779033696`, then
`1779033696 + 7 = 1779033703`. □

That integer is the first word of §5.3.3. It is not the digest of
“abc”, and it is not the word §5.3.1 prints. This lesson does not
recompute the square root the standard describes, and it does not
hash the message. N13's program hashes the message. J5 derives the
compression function. Neither derivation is copied here.

What N13's “abc” test does not cover is the edition RFC 4231
actually names. RFC 4231's normative reference [2] is FIPS 180-2,
August 2002, with Change Notice 1 dated February 2004, not FIPS
180-4. The digest bytes are not that choice. A passing Match of
`BA7816BF…` does not choose August 2015 over August 2002, because
this lesson does not show that the two editions print different
bytes for this sample. The edition is the citation on the card.
N13 already said the HMAC bytes are not a claim that RFC 4231 names
the 2015 edition. The deepening is the other half of the same fact:
the “abc” Match does not choose FIPS 180-2 either.

The test also does not cover the examples file's two-block message,
the empty message, or a message the N13 program does not denote. It
does not prove that every block agrees with §6.2.2. It does not say
the evaluator is constant-time. It does not say anything about
collisions. A passing report is a Match of one 32-byte array on one
message. Do not call that Match verified.

Section 6.2.2 is the SHA-256 hash computation. Its preprocessing
points back to §5.3.3. The computation itself is the message
schedule and the compression function. Those stay in J5.
Assumption J2.7 stands. In the October 2008 contents and in the
August 2015 contents, §5.3.3 is SHA-256 and §6.2.2 is SHA-256 hash
computation. Agreement of two addresses is not agreement of the
documents. The August 2015 contents add sections the October 2008
contents do not list, and §5.2's title changes while the digits
`5.2` survive. A citation that gives the digits and omits the
edition has not said which title it means.

Appendix C of the August 2015 publication is the list of technical
changes from FIPS 180-3. Item 1, in the content stream of the DOI
PDF, names `FIPS 140-4` in the sentence about when padding may be
inserted, between neighbors that discuss FIPS 180-3 and FIPS 180-4.
This lesson does not replace those digits with `180`. A silent
repair would be a new pin. The publication's erratum table has one
row, dated `5/9/2014`, editorial, changing `t < 79` to `t ≤ 79` at
§4.1.1. The row says the change has been incorporated. Applying it
again, as if the August 2015 cover still printed the old bound,
would edit a text that has already been edited. The erratum date
is not the cover date, and it is not the Orange edition token.

**One RFC 4231 test case.** The case N13 already runs is test case
1, §4.2, the HMAC-SHA-256 line. The header of the plain-text file,
read on 2026-10-09 from the RFC Editor, is:

```text
Request for Comments: 4231
Category: Standards Track
M. Nystrom
December 2005
```

That header prints no `Obsoletes` line and no `Updates` line. The
absence is part of the pin. A card that adds either line has named
a header the file does not print. The normative reference for the
hash, reference [2] in §7.1, is FIPS 180-2 as dated above, not the
August 2015 publication.

The key in §4.2 is twenty bytes `0b`. The data line is
`4869205468657265` with the note `("Hi There")`. The HMAC-SHA-256
digest printed under that data is

```text
b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
```

N13's test `RFC 4231 4.2 HMAC-SHA-256 test case 1` compares that
line. The definition of the tag is RFC 2104, February 1997, §2.
This lesson points at that test. It does not rebuild the pads, the
two calls, or the hash.

What that N13 test does not cover:

It does not cover the HMAC-SHA-224, HMAC-SHA-384, or HMAC-SHA-512
lines printed in the same section. A match against one of those
lines would be a match against a different scheme.

It does not cover §4.3 through §4.8. Section 4.6 truncates the tag
to 128 bits. Section 4.7's key is 131 bytes. N13's `mac` returns 32
bytes and does not denote that key length.

It does not establish the security discussion RFC 4231 §5 points at
in RFC 2104. It does not choose FIPS 180-4 over the FIPS 180-2
reference [2]. It does not run the “abc” test. Passing the HMAC
test is not a second run of the digest test. A passing report is a
Match of this one tag on this one key and this one eight-byte text.
Do not call that Match verified.

**The four constant words.** Section 2.3 of RFC 7539 and section 2.3
of RFC 8439 print the same four words, in the same order:

```text
0x61707865, 0x3320646e, 0x79622d32, 0x6b206574
```

Those words are the fixture both files print. N12 already showed
that they are the little-endian readings of `expand 32-byte k`.
Proposition N12.4 is the derivation. This lesson does not pack the
bytes, and it does not contain a second program that shifts them
into a word. A comparison that accepted these four words would pass
under either citation, because both documents print them. A passing
comparison does not choose the edition.

The same section of the two RFCs does not agree in every sentence.
RFC 7539 §2.3 says the nonce “should not be repeated for the same
key.” RFC 8439 §2.3 says the nonce “MUST not be repeated for the
same key.” The word `MUST` is in capitals. The word `not` is not.
Section 1.1 of RFC 8439 says the key words, including `MUST NOT`,
are interpreted as in BCP 14 when, and only when, they appear in
all capitals. The sentence in §2.3 contains `MUST not`. It does not
contain the all-capitals phrase `MUST NOT`. The four words do not
mention a nonce. A comparison of the words does not detect the
change of wording, and it does not decide whether `MUST not` is the
BCP 14 phrase.

**The June 2018 total, which the listing tests.** RFC 8439 §2.8
prints `274,877,906,880` for `P_MAX` and
`274,877,906,896` for `C_MAX`. RFC 7539 §2.8 prints
`247,877,906,880` and `247,877,906,896` for the same two roles.
Proposition J2.1 is the June product. Proposition J2.2 is the gap.

The tag length is not a second mystery. Subtracting the two June
integers leaves `16`. A 128-bit tag is `128 / 8 = 16` octets.
`274877906880 + 16 = 274877906896`. That is the June 2018 `C_MAX`.
`247877906880 + 16 = 247877906896`. That is the May 2015 `C_MAX`.
Both files add sixteen. The edition disagreement is the base, not
the tag width. Erratum 4371's width, 4 bytes in the May 2015
pseudocode and 8 bytes in the June 2018 pseudocode, is a different
integer. A test of `P_MAX` does not mention it. Erratum 4700's
change of shape is not an integer either.

The card for the integer the next section tests, filled from the
files retrieved on 2026-10-05:

The issuer is the Internet Research Task Force, and the plain text
is published by the RFC Editor. The document identifier is RFC 8439.
The edition is the June 2018 plain-text file, whose header prints
`Obsoletes: 7539`. The URI is
`https://www.rfc-editor.org/rfc/rfc8439.txt`. The SHA-256 digest of
the file retrieved that day is
`25bef70fbf7a07ff45c2fe4cb7c6ce954eac687413d8610603268b4e4415324c`.
The predecessor file of RFC 7539 had digest
`546e12200dbbc7b08cf85c8f07ea8681be89954b7c7e0c56f0c51f775a0ba704`.
The clause is §2.8, both the sentence that gives the total of
`(2^32 - 1)` blocks of 64 bytes and the `P_MAX` bullet that prints
the same integer. The role is the output of that multiplication.
The errata consulted for this integer are 4858 and 4861, both
marked Verified. The comparison does not cover the length-field
width, the nonce sentence, the AEAD construction, erratum 4700's
shape, any Held erratum this lesson did not place beside the text,
or the file digest. A digest is of a file, not of a sentence. The
HTML and PDF renderings of RFC 8439 are different byte strings.
They can carry the same §2.8 sentence and hash differently.

### J2.4 Orange against the pin

Listing J2.1 pins the June 2018 integers. The multiplication is the
one the note describes. The expected values are the integers that
file prints. The test names the section. The name is a string in
the source. The compiler does not read RFC 8439.

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
print the two integers. N8 already showed that shape for a word
array. The same shape is used here for an `Int`.

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
the `Bool` in that test was true. The report does not say that the
strings `RFC 8439` and `2.8` were looked up in a document.
Assumption J2.4 is that check, and it lives in the prose around the
listing, not in the title.

What this passing pair establishes is narrow. On this compiler, the
function `bytes` denotes `274877906880`, and `c_max` denotes
`274877906896`. Those are the integers printed in RFC 8439 §2.8 for
`P_MAX` and for `C_MAX`. The tests do not establish that the AEAD
construction accepts a plaintext of that length, they do not run
ChaCha20, and they do not choose a nonce. They do not establish the
width of the length field in §2.8.1. That width is 8 in the June
2018 pseudocode and 4 in the May 2015 pseudocode, which is erratum
4371, and neither test mentions it. They do not hash either RFC
file. They do not choose FIPS 180-4 over FIPS 180-2. They do not
run N13's “abc” test or N13's HMAC test.

The ledger keeps the integers this lesson prints, so a later edit
of the listing or of Proposition J2.1 cannot leave a different
total standing in the block. The Python check recomputes each line
from the arithmetic named here. It does not fetch a file.

```text
j2-ledger
blocks = 4294967295
block-bytes = 64
p-max = 274877906880
p-max-old = 247877906880
p-gap = 27000000000
tag-octets = 16
c-max = 274877906896
c-max-old = 247877906896
length-7539 = 4
length-8439 = 8
iv-word = 1779033703
```

`blocks` is `2^32 - 1`. `p-max` is Proposition J2.1. `p-gap` is
Proposition J2.2. `iv-word` is Proposition J2.3. `length-7539` and
`length-8439` are the widths erratum 4371 separates, which the
tests do not check. The ledger answers whether those printed
integers are the integers the stated arithmetic produces. It does
not answer which file you retrieved.

### J2.5 A wrong-pin repair

Listing J2.2 computes the same product as Listing J2.1. The test
title names RFC 8439 §2.8. The expected integer is the one RFC 7539
prints. The title and the integer disagree. The compiler can see
the integer. It cannot see the title's claim about a document.

**Listing J2.2 — `wrong_edition.or`**

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

**Proposition J2.4.** The `Bool` in Listing J2.2 is false because
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
which side disagreed.

The repair, once made, is Listing J2.1's first test. That repaired
test still does not establish the length-field width, the nonce
sentence, or the AEAD construction. A pass after a repair
establishes the repaired `Bool`. It does not reach back and
establish the sentences the test never mentioned.

The same shape of mistake, on the N13 pin, does not give the
compiler two different integers to print. A card that copies
`BA7816BF…` and then writes “FIPS 180-2, August 2002, with Change
Notice 1 dated February 2004” because that is reference [2] of RFC
4231 has named the edition the HMAC RFC cites. It has not named the
edition N13 pinned for the sample, which is FIPS 180-4, August
2015. The digest bytes do not fail, so the repair is not a new
array. The repair is the edition and the date on the card, plus the
sentence that those bytes do not choose FIPS 180-2. Listing J2.2
cannot see that repair. It can see a repair only when the printed
values differ, which is why the integer in §2.8 is the listing and
the hash edition is the card.

A wrong section of the same FIPS publication is the “abc” address
itself. Section 5.1.1 prints the padding of “abc” and does not print
`BA7816BF…`. Citing §5.1.1 as the source of the digest is the wrong
section. The repair, stated in §J2.3, keeps the bytes and names the
examples file. Citing §5.3.1's word `67452301` as if it were
§5.3.3's word `6a09e667` is a wrong section whose integers do
differ. Proposition J2.3 puts the SHA-256 word at `1779033703`.
The SHA-1 word is a different integer. The exercise expands it.
This lesson does not add a third listing to subtract them. One
failing report is enough to show the shape, and the card is enough
to show the repair the report cannot see.

### J2.6 Exercises with worked answers

Ten exercises. Each one uses a pin this lesson copied or a repair
this lesson showed. Each answer is below. A guess without the
arithmetic is not an answer.

**Exercise J2.1 — Name the object.** A colleague writes “implemented
against RFC 7539 with the errata.” List the objects that sentence
has not yet separated, using Assumption J2.3 and the statuses in
§J2.1. Say which two Verified errata change a printed integer, and
which Verified erratum changes a width rather than an integer.

**Exercise J2.2 — The swapped group.** Compute
`274877906880 - 247877906880`. Say which digits of the two
integers differ, and state the erratum that records the same swap.
Do not appeal to the compiler.

**Exercise J2.3 — Sixteen octets.** From the two integers RFC 8439
prints for `P_MAX` and `C_MAX`, compute the tag length the equation
uses. Then say, in one sentence, why a passing `C_MAX` test still
does not decide erratum 4371.

**Exercise J2.4 — Capitals.** Quote the nonce sentence of RFC 7539
§2.3 and the nonce sentence of RFC 8439 §2.3, as far as the words
that differ. Say what §1.1 of RFC 8439 requires before `MUST NOT`
is the BCP 14 phrase. Then say why a comparison of the four words
`0x61707865`, `0x3320646e`, `0x79622d32`, and `0x6b206574` does not
choose the edition, and where the byte order of those words was
derived.

**Exercise J2.5 — Read the report.** Listing J2.2 fails with left
`274877906880` and right `247877906880`. State the one edit that
repairs it, which listing that edit produces, and which sentences
the repaired test still does not establish.

**Exercise J2.6 — The edition the digest does not choose.** The
N13 test of “abc” passes on

```text
BA7816BF 8F01CFEA 414140DE 5DAE2223 B00361A3 96177A9C B410FF61 F20015AD
```

Name the publication and the cover date this lesson pins for that
sample. Name the publication RFC 4231's normative reference [2]
gives for the hash, including that reference's change notice. Say
what the passing N13 test does not choose, and write the repair for
a card that attributed the sample to FIPS 180-2. Do not recompute
the digest.

**Exercise J2.7 — The line beside the tag.** RFC 4231 §4.2 prints
four digests under one key and one data line. N13 compares the
HMAC-SHA-256 line. Give the RFC's date and say what its header
prints for `Obsoletes` and for `Updates`. Then name two things that
passing comparison does not cover, one of them inside §4.2 and one
of them outside it.

**Exercise J2.8 — The neighboring word.** Expand `67452301` from
hex into an integer by powers of 16. Say which subsection of FIPS
180-4 prints that word, which subsection prints `6a09e667`, and why
using one as the expected value of the other is a wrong section.
Use Proposition J2.3 for the SHA-256 word. Do not build a listing.

**Exercise J2.9 — Figure 7.** Say what Figure 7 denotes in the
November 26, 2001 text of FIPS 197, and what it denotes in the
May 9, 2023 update. Say where the 2023 text puts the S-box, and
what Appendix D item 17's section number fails to match. Explain
why a test of the byte `0x63` would not catch the move.

**Exercise J2.10 — Two digests, one integer.** The Gate 0 fixture's
digest is sixty-four `a` characters. The SHA-256 of the June 2018
RFC 8439 file retrieved on 2026-10-05 begins `25bef70f`. Listing
J2.1 compares `(2^32 - 1) * 64` with `274877906880`. Say which
question each of those three answers, and why a passing run of
Listing J2.1 can leave both digests unchecked.

#### Worked answers

**J2.1.** The sentence names RFC 7539 and then says “the errata”
without a status, a number, or a date. It has not separated the
May 2015 RFC, each erratum, and RFC 8439, the obsoleting RFC.
Assumption J2.3 is that separation. The statuses in §J2.1 are
Verified, Held for Document Update, and Rejected. A Verified mark
is the editor's. It does not make an Orange Match verified.
Errata 4858 and 4861 change a printed integer, `247,877,906,880`
to `274,877,906,880`, and `247,877,906,896` to `274,877,906,896`.
Erratum 4371 changes a width, `num_to_4_le_bytes` to
`num_to_8_le_bytes`, four bytes to eight. Erratum 4700 changes a
shape, two results into one concatenation, and erratum 8274 is
Rejected, so it is not a correction to apply.

**J2.2.** Subtract digitwise. The trailing nine digits
`877906880` occur in both integers and cancel. The leading group
is `274` in one integer and `247` in the other.
`274 - 247 = 27`, and that `27` sits in the billions place, so the
difference is `27 * 1000000000 = 27000000000`. The digits that
differ are the first and the third of the leading group. Erratum
4858 marks the exchanged pair. Proposition J2.2 is the same
identity.

**J2.3.** RFC 8439 prints `P_MAX` as `274,877,906,880` and prints
`C_MAX` as `274,877,906,896`. The difference is `16`. A 128-bit tag
is 16 octets, so the equation's tag length is 16. The May 2015
pair differs by 16 as well. Erratum 4371 is the width of the length
field in §2.8.1, 4 bytes against 8 bytes, which is
`num_to_4_le_bytes` against `num_to_8_le_bytes`. Listing J2.1's
`C_MAX` test compares `bytes() + 16` with `274877906896`. It never
mentions that field, so a pass does not decide the erratum.

**J2.4.** RFC 7539 §2.3 says the nonce “should not be repeated for
the same key.” RFC 8439 §2.3 says the nonce “MUST not be repeated
for the same key.” Section 1.1 of RFC 8439 says the key words are
interpreted as in BCP 14 when, and only when, they appear in all
capitals. `MUST not` is not the all-capitals phrase `MUST NOT`.
Both RFCs print `0x61707865`, `0x3320646e`, `0x79622d32`, and
`0x6b206574` in §2.3. A comparison of those words passes under
either citation, so it does not choose June 2018 over May 2015.
The byte order is Proposition N12.4: the little-endian reading of
`expand 32-byte k`. This lesson does not pack the bytes.

**J2.5.** Edit the right-hand side. Replace `247877906880` with
`274877906880`. Left is already the June total, which is the value
`bytes()` denotes, and Proposition J2.1 says that value is
`274877906880`. Proposition J2.4 says the specs stay. The repaired
test is the first test of Listing J2.1. It still does not establish
the length-field width of §2.8.1, the nonce sentence of §2.3, or
the AEAD construction. Outcome 5 applies to the repaired test.

**J2.6.** The sample is pinned to FIPS PUB 180-4, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. The digest line is the NIST examples
file, not a sentence of §5.1.1. RFC 4231's normative reference [2]
is FIPS 180-2, August 2002, with Change Notice 1 dated February
2004. The passing N13 test does not choose August 2015 over that
2002 edition, and it does not choose FIPS 180-2 over August 2015.
The repair of a card that attributed the sample to FIPS 180-2 is
the citation: write FIPS 180-4, August 2015, and write that the
bytes do not choose FIPS 180-2. The array is not replaced. The
test also does not cover the two-block sample, the empty message,
§6.2.2, or a constant-time claim. A Match is not called verified.

**J2.7.** RFC 4231 is dated December 2005. Its header prints no
`Obsoletes` line and no `Updates` line. Inside §4.2 the passing
comparison does not cover the HMAC-SHA-224, HMAC-SHA-384, or
HMAC-SHA-512 lines. Outside §4.2 it does not cover §4.3 through
§4.8, including the 128-bit truncation in §4.6 and the 131-byte key
in §4.7, and it does not choose FIPS 180-4 over reference [2].
This lesson does not re-derive the tag. The program is N13's.

**J2.8.** The digits of `67452301` are `6`, `7`, `4`, `5`, `2`,
`3`, `0`, `1`. With `16^7 = 268435456`, `16^6 = 16777216`,
`16^5 = 1048576`, `16^4 = 65536`, `16^3 = 4096`, `16^2 = 256`, and
`16^1 = 16`:

`6 * 268435456 = 1610612736`,

`7 * 16777216 = 117440512`,

`4 * 1048576 = 4194304`,

`5 * 65536 = 327680`,

`2 * 4096 = 8192`,

`3 * 256 = 768`,

`0 * 16 = 0`,

`1 * 1 = 1`.

The sum is `1610612736 + 117440512 = 1728053248`, then
`1728053248 + 4194304 = 1732247552`, then
`1732247552 + 327680 = 1732575232`, then
`1732575232 + 8192 = 1732583424`, then
`1732583424 + 768 = 1732584192`, then
`1732584192 + 1 = 1732584193`.

Section 5.3.1 prints `67452301`, the first word of the SHA-1
initial hash value. Section 5.3.3 prints `6a09e667`, which
Proposition J2.3 puts at `1779033703`, the first word of the
SHA-256 initial hash value. The role is the same kind of role, an
initial hash word, and the integers differ. Assumption J2.4 says a
section number is an address inside one edition. Using the §5.3.1
word as the expected value of the §5.3.3 word is a wrong section.
The repair is the expected hex `6a09e667`, not a new recurrence.
This lesson does not add a listing for that subtraction. Listing
J2.2 already showed the report shape on a pair of integers that
differ.

**J2.9.** In the November 26, 2001 text, Figure 7 is the S-box,
and it begins `63 7c 77 7b`. In the May 9, 2023 update, Figure 7
is the illustration of `KEYEXPANSION()` for AES-192, and the S-box
is Table 4, whose first entry is `63`. Appendix D item 17 attaches
`INVSHIFTROWS()` to section `5.3.2`, while the contents of that PDF
list `INVSHIFTROWS()` at §5.3.1 and `INVSUBBYTES()` at §5.3.2. The
byte `0x63` is the first S-box entry in both editions. A test of
that byte passes whether the card says Figure 7 or Table 4. The
byte did not move. The figure number did. `algorithms/aes/aes.or`
records both locators. This lesson does not transcribe the round.

**J2.10.** The sixty-four `a` characters answer the question “what
bytes does the Gate 0 fixture store as its digest?” The answer is
a placeholder. The fixture says the digest is synthetic and the
archive state is `acquisition_required`. The digest that begins
`25bef70f` answers the question “what was the SHA-256 of the June
2018 plain-text file retrieved on 2026-10-05?” Listing J2.1
compares `(2^32 - 1) * 64` with `274877906880`. That answers
whether `bytes()` denotes the June 2018 integer on this compiler.
The listing does not read the fixture, and it does not hash the
RFC file, so both digests can be wrong on a day when the test
still passes. The ledger line `p-max` is a fourth question, whether
the printed arithmetic still equals Proposition J2.1, and the
listing does not read the ledger either.

## Sources and epigraph record

The quotation is the borrowed sentence. The dates and identifiers
in the opening are the publication's own cover lines, checked
against the files named here. Later sections quote clauses. They
do not replace this record.

**[S13] Yoav Nir and Adam Langley.** RFC 8439, “ChaCha20 and
Poly1305 for IETF Protocols,” Internet Research Task Force,
Informational, June 2018. The header line is `Obsoletes: 7539`.
The epigraph is the first sentence of the Abstract's second paragraph,
the sentence that begins “RFC 7539, the predecessor of
this document,”. The next sentence of that paragraph, not used as
the epigraph, is “It was a product of the Crypto Forum Research Group (CFRG).”
The third sentence of that paragraph, not the
sentence after a different paragraph, says that this document
merges the errata filed against RFC 7539 and adds a little text to
the Security Considerations section. The Abstract's first paragraph
defines the cipher, the authenticator, and the combined mode. It
is not the epigraph. Wording was checked on 2026-10-05 against the
plain-text RFC at the RFC Editor. The sentence contains no inner
quotation. No translation is involved. The sentence is not an
endorsement of Orange. It records that a document written to be a
stable reference has a predecessor, and that the predecessor was
replaced.
This record's tag is [S13].

Source: <https://www.rfc-editor.org/rfc/rfc8439.txt>

The authors' names are the names in the RFC's “Authors' Addresses”
section: Yoav Nir and Adam Langley. The cover also prints `Y. Nir`
and `A. Langley`.

**[T9] Retrieved files.** On 2026-10-05 the lesson retrieved the
plain text of RFC 8439 and of RFC 7539 from the RFC Editor, the
errata page for RFC 7539, the August 2015 PDF of FIPS PUB 180-4
at `10.6028/NIST.FIPS.180-4`, the May 9, 2023 PDF of NIST FIPS
197-upd1 at `10.6028/NIST.FIPS.197-upd1`, the archived November
26, 2001 PDF of FIPS 197 carrying the withdrawal notice, and the
October 2008 PDF of FIPS 180-3 carrying its archive wrapper. The
SHA-256 digest of the RFC 8439 file is
`25bef70fbf7a07ff45c2fe4cb7c6ce954eac687413d8610603268b4e4415324c`.
The SHA-256 digest of the RFC 7539 file is
`546e12200dbbc7b08cf85c8f07ea8681be89954b7c7e0c56f0c51f775a0ba704`.
The SHA-256 digest of the August 2015 FIPS 180-4 PDF is
`0455b406d89648d20cbde375561e19c245b9815e894164c2670772e3d54deb82`.
The SHA-256 digest of the May 9, 2023 FIPS 197-upd1 PDF is
`62c86eb567f13edb8f71826e985da870b04ef6381634f303cdb16e84d47becd1`.
The SHA-256 digest of the archived November 26, 2001 FIPS 197 PDF
is
`251dfe0b5dc283abaf364adf586f7ec6dc4e495335d48dd5ee0fee6c5961da8a`.
The errata page and the FIPS 180-3 file were read for statuses,
contents, and the wrapper date and are not the pin of an expected
integer. On 2026-10-09 the lesson read the plain-text header of
RFC 4231 at the RFC Editor for the date, the category, and the
absence of an `Obsoletes` line and of an `Updates` line. That
reading is not given a file digest here. The expected HMAC line is
the line N13 already copied from §4.2, not a hash of the RFC file.
A digest is of the retrieved file. Another rendering can carry the
same sentence and a different digest. No translation is involved.
These files are not an endorsement of Orange, and the retrieval is
not a certification.
This record's tag is [T9].

**[C3] Pin surface.** The listings use `edition 2026`, `module`,
`spec`, `Int`, `*`, `+`, `-`, `==`, and `test`. Those forms are
ones N10, N12, and N14 already ran on this compiler. No form in
the listings is Proposed. The listings do not pack a byte string,
and they do not transcribe FIPS 180-4 §6.2.2 or HMAC. A passing
test is a Match on the inputs it writes. It is not a constant-time
claim, not a certification, and not a choice of FIPS 180-2 against
FIPS 180-4.
This record's tag is [C3].

## Evidence boundary

J2 is a Journeyman lesson. The six outcomes in §J2.1 are the finish
line. The assumptions in §J2.2 bound them. The fixtures copied in
§J2.3 are the NIST SHA-256 digest of “abc”, one RFC 4231 test case,
the four ChaCha constant words, and the June 2018 byte total.
Listing J2.1 tests that total. Listing J2.2 is the wrong-edition
repair. Ten exercises have worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`. The Orange listings
are the fenced programs in this file.
`compiler/crates/orangec/tests/book_j2.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not
derive FIPS 180-4 §6.2.2, they do not re-derive HMAC, and they do
not accept a proposal.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction, and revised into these six sections on
2026-10-09 at the owner's direction. Owner review is pending. No
deployment recommendation is made.
