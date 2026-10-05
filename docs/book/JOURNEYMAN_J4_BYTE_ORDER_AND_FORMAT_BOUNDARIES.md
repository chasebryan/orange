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

The opening states the only-this-stack test, the five outcomes,
and the seven assumptions. The Orange listings that discharge
the test are added with the sections that teach them.
`orangec --version` was run on 2026-10-05 and printed the line
in §J4.1. That run does not establish a cryptographic security
claim, and it does not accept OEP-0017.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at
the owner's direction. Owner review is pending. No deployment
recommendation is made.
