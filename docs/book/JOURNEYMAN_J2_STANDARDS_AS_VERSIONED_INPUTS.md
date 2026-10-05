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
line. The assumptions in §J2.2 bound them. Listings, worked
answers, and the failing test are the rest of the lesson. Those
checks do not establish a cryptographic security claim, they do
not derive FIPS 180-4 §6.2.2, and they do not accept a proposal.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction. Owner review is pending. No deployment
recommendation is made.
