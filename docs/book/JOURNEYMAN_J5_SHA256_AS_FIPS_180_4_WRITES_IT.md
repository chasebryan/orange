# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J5: SHA-256 as FIPS 180-4 Writes It. Draft 2026-10-09.

Continue from
[Ready for Standards](NOVICE_N14_READY_FOR_STANDARDS.md#n14-ready-for-standards).
N14 named this lesson as the study that derives the SHA-256 compression
function and its message schedule. The derivation is §6.2.2 of one
edition, together with the sections that §6.2.2 calls.
[Modules and Provenance](NOVICE_N13_MODULES_AND_PROVENANCE.md#n13-modules-and-provenance)
already ran a transcription of that computation on the message “abc”
and did not derive the round. A passing test is still the Match of
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study):
one `Bool`, on the inputs the test wrote. The reading habit is still
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).

Lesson J2 is titled *Standards as Versioned Inputs*. Lesson J3 is
titled *The Corpus as Acceptance Test*. Lesson J4 is titled *Byte
Order and Format Boundaries*. Those three are Block A. None of them
is in this tree. This lesson cites the titles. It does not cite a
section, a listing, or a proposition of any of them. A reader who
has those lessons and a reader who does not are reading the same
derivation.

This lesson is **J5**. The locked label is J5. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. When
the text says “§6.2.2” or “§5.1.1”, the number is a section of the
edition named in the same sentence.

## J5: SHA-256 as FIPS 180-4 Writes It

> “This standard specifies hash algorithms that can be used to generate digests of messages. The digests are used to detect whether messages have been changed since the digests were generated.”
>
> — National Institute of Standards and Technology, FIPS PUB 180-4, *Secure Hash Standard (SHS)* (August 2015), Abstract. [S16]

The two sentences are the whole Abstract. The first says what the
document specifies. The second says what the digests are used for.
Neither sentence is a proof that a changed message produces a
different digest, and this lesson does not supply that proof. The
standard's own security discussion is Appendix A.1, which points at
SP 800-107 and is not transcribed here. The object on this page is
the function §6.2 defines.

**The only-this-stack test.** You will derive the padding, the
message schedule, and one compression round from the August 2015
text, then run the transcription with `orangec check`, `orangec
eval`, and `orangec test` on the compiler this tree builds. You will
also feed the compiler three programs it rejects, and one program it
accepts whose test fails. The compiler's version line is `orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`. Every listing
is written for that slice. A form this slice does not implement is
shown by the diagnostic the compiler prints. This lesson does not
invent a syntax for it.

### J5.1 The sections, in the order the standard uses them

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can name the sections of FIPS PUB 180-4, August 2015, that
   define SHA-256, in the order §6.2 uses them, and you can separate
   that edition from the examples file that prints the digests.
2. You can compute the padding of “abc” and of the 56-byte message,
   including the number of zero bits and the reason the next byte is
   `0x80`.
3. You can compute `W16` and `W17` of the “abc” schedule from the
   recurrence in §6.2.2, step 1.
4. You can compute round 0 of that block: `T1`, `T2`, and the eight
   working variables, including each sum that crosses `2^32`.
5. You can transcribe §6.2.2 so that the names and the order are the
   standard's, and match both known-answer digests and the
   intermediate hash the examples file prints after the first block
   of the longer message.
6. You can predict the word computed when one rotation in `σ1` is
   replaced by a shift, read the failing test, and say what a
   passing digest does not establish.

The edition is FIPS PUB 180-4, *Secure Hash Standard (SHS)*, August
2015, DOI `10.6028/NIST.FIPS.180-4`. The file retrieved on
2026-10-09 has SHA-256
`0455b406d89648d20cbde375561e19c245b9815e894164c2670772e3d54deb82`.
N13 recorded this publication as [D1]. The erratum page in that file
records one editorial change, already incorporated: in §4.1.1, which
is SHA-1, `t < 79` became `t ≤ 79`. That line is not an equation of
SHA-256. No further erratum is applied here.

**Assumption J5.1 — The pin is the edition, not the title.**
“Secure Hash Standard” names more than one publication. The
algorithms, the section numbers, and the hex words below are the
August 2015 file. Lesson J2 is the lesson that makes a pin a
procedure. This lesson does not depend on that text. The pin is
stated in this assumption.

**Assumption J5.2 — A word is 32 bits, and `+` is the residue.**
§3.2 fixes `w = 32` for SHA-256. Addition of two words is the sum of
the integers they represent, reduced modulo `2^32`. The same
sentence is repeated at the start of §6.2.2: addition is performed
modulo `2^32`. `2^32 = 4294967296`. For a sum of integers that may
exceed one multiple of that modulus, the residue is the unique
remainder in `0` through `2^32 − 1`. On this compiler, `+` of two
`Word[32]` values is that residue. The hand arithmetic below is the
mathematical claim. A Match says the program agreed on the inputs
the test wrote.

**Assumption J5.3 — The leftmost bit is the most significant bit.**
§3.1, item 2, states the big-endian convention for 32-bit and 64-bit
words: within each word, the most significant bit is stored in the
leftmost bit position. Item 3 represents an integer `Z` with
`0 ≤ Z < 2^64` as the pair of words `(x, y)` where
`Z = 2^32·x + y`. Item 4 says a SHA-256 message block is 512 bits,
represented as sixteen 32-bit words. Lesson J4 is the lesson that
treats byte order. This lesson does not depend on that text. The
order used below is the order these three items state. The first
byte of a word is the leftmost eight bits, and those are the most
significant eight bits.

**Assumption J5.4 — The six functions are equations (4.2) through
(4.7).** §4.1.2 defines them on 32-bit words. The names in Orange
are ASCII, for the reason Listing J5.4 shows. The mapping is:

```text
Ch(x, y, z)        ch(x, y, z)
Maj(x, y, z)       maj(x, y, z)
Σ0{256}(x)         big_sigma0(x)
Σ1{256}(x)         big_sigma1(x)
σ0{256}(x)         small_sigma0(x)
σ1{256}(x)         small_sigma1(x)
```

`∧` is `&`. `⊕` is `^`. `¬` is `~`. `ROTR n` is `>>> n`. `SHR n` is
`>> n`. Parentheses follow the equations. Orange gives operators
from different groups no relative precedence; the parentheses are
the equation's, written where the compiler requires them.

**Assumption J5.5 — The examples file is not a section of the
standard.** Appendix A.2 of the August 2015 file says examples are
available and prints a URL. It does not print a digest. The digests,
the block contents, and the round rows cited below are the NIST
SHA-256 examples file N13 recorded as [N1]. The file retrieved on
2026-10-09 has SHA-256
`7006b6549dad2fc8c6f29417a921f2e48208157ef496a7e1e1d7d17c5cc1e7db`.
Lesson J3 is the lesson that treats a finite set of pinned vectors
as an acceptance test. This lesson does not depend on that text.
The acceptance set is the tests in §J5.6, and the coverage sentence
is stated there.

**Assumption J5.6 — A Match is one `Bool` on the inputs written in
that test.** Well-formed source is not a Match. A Match on these
inputs is not a Match on every input. A Match is not called
verified. Nothing on this page is a security claim.

**Assumption J5.7 — The hand calculation and the program denote the
same word only where both perform the same operations under J5.2
through J5.4.** Agreement on one input is one point. The domain of
one 32-bit word is `2^32` inputs. The domain of a message block is
`2^512` inputs. The domain of a message whose length in bits is
below `2^64` is larger than either.

§6 opens by splitting every algorithm in the standard into
preprocessing and hash computation. For SHA-256 the split is §6.2.1
and §6.2.2.

§6.2.1 has two steps. Step 1 sets `H(0)` from §5.3.3. Step 2 pads
and parses the message as specified in §5. §5 itself has three
steps: padding (§5.1.1 for SHA-256), parsing into `N` blocks of 512
bits (§5.2.1), and the initial hash value (§5.3.3). §6.2.2 then
uses the functions of §4.1.2 and the constants of §4.2.2. The
working variables are `a` through `h`. The temporary words are `T1`
and `T2`. The schedule words are `W0` through `W63`.

SHA-224, §6.3, is defined as SHA-256 with a different `H(0)` and a
truncated digest. SHA-512, §6.4, uses 64-bit words. Neither is
transcribed. The two messages below are the one-block sample and
the two-block sample in the examples file.

### J5.2 Padding

§5.1.1. Let `ℓ` be the length of the message `M` in bits. Append a
single `1` bit, then `k` zero bits, where `k` is the smallest
non-negative solution of

```text
ℓ + 1 + k ≡ 448 (mod 512)
```

Then append the 64-bit block that is `ℓ` in binary. The padded
length is a multiple of 512.

**Proposition J5.1.** There is exactly one such `k` in `0` through
`511`. It is the remainder when `447 − ℓ` is divided by `512`,
taking the remainder in that range.

*Proof.* `ℓ + 1 + k ≡ 448 (mod 512)` rearranges to
`k ≡ 447 − ℓ (mod 512)`. The integers `0` through `511` are one
complete set of residues modulo `512`, so exactly one of them
satisfies the congruence. The standard asks for the smallest
non-negative solution, and that unique residue is already
non-negative. □

**Proposition J5.2.** For the message “abc”, `ℓ = 24` and `k = 423`.
The padded message is one block of 512 bits.

*Proof.* §5.1.1 states that the message is 8-bit ASCII and that its
length is `8 × 3 = 24`. By Proposition J5.1,
`k ≡ 447 − 24 = 423 (mod 512)`. `423` is already in range, so
`k = 423`. Then

```text
24 + 1 + 423 = 448
```

and `448 ≡ 448 (mod 512)`. The length field adds 64 bits:

```text
448 + 64 = 512 = 1 × 512
```

So `N = 1`. The same arithmetic is the parenthetical the section
prints: `448 − (24 + 1) = 423`. □

**Proposition J5.3.** Let `ℓ = 8n` for an integer `n ≥ 0`. Then
`k ≡ 7 (mod 8)`, so `k ≥ 7`. The eight bits that follow the message
are `1` and then seven `0`s. Under Assumption J5.3 those eight bits
are the byte `0x80`. The remaining `k − 7` zero bits are
`(k − 7) / 8` zero bytes, and the length occupies the next eight
bytes, most significant byte first.

*Proof.* From Proposition J5.1, `k ≡ 447 − 8n (mod 512)`.
`447 = 55 × 8 + 7`, so `447 ≡ 7 (mod 8)`. And `8n ≡ 0 (mod 8)`.
Hence `k ≡ 7 (mod 8)`. Among `0` through `511`, every residue
congruent to `7` modulo `8` is at least `7`. The appended `1` and
the first seven of the `k` zeros are therefore eight bits, and no
length bit has started yet.

Assumption J5.3 stores the most significant bit on the left. The
leftmost of these eight bits is `1` and the other seven are `0`,
which is `10000000` in binary and `0x80` in hex. The remaining
zero-bit count is `k − 7`, and `k − 7` is divisible by `8` because
`k ≡ 7 (mod 8)`. Those bits are `(k − 7) / 8` bytes, each `0x00`.

The length is a 64-bit integer. Assumption J5.3 writes
`Z = 2^32·x + y` as the pair of words `(x, y)`, and the leftmost
word is the most significant. For every message in this lesson,
`ℓ < 2^32`, so `x = 0` and `y = ℓ`. The eight bytes are four `0x00`
bytes followed by `ℓ` as four bytes, most significant byte first. □

The hypothesis `ℓ = 8n` is the whole reach of the byte argument.
This compiler has `Word[8]` and does not have a bit-string type. A
message whose length in bits is not a multiple of `8` is not a
listing in this lesson. Both samples in the examples file are
byte-aligned, so both fall under the proposition.

For “abc”, `k − 7 = 416` and `416 / 8 = 52`. The block is the three
bytes `0x61`, `0x62`, `0x63`, then `0x80`, then 52 zero bytes, then
eight length bytes. `0x61`, `0x62`, and `0x63` are the ASCII bytes
of “a”, “b”, and “c”. The first word is those three bytes and the
`0x80` byte. Under Assumption J5.3 the word is `0x61626380`. The
length word is `24`, which is `0x00000018`, and it is the last of
the sixteen words. The fourteen words between them are zero. §5.1.1
draws this block and does not print the hex words. The examples
file's one-block “Block Contents” prints `W[0] = 61626380` and
`W[15] = 00000018`.

**Proposition J5.4.** The examples file's two-block message is 56
bytes, so `ℓ = 448` and `k = 511`. The padded message is two blocks.
The first block ends with the words `0x80000000` and `0x00000000`.
The second block's last word is `0x000001c0`.

*Proof.* `56 × 8 = 448`. By Proposition J5.1,
`k ≡ 447 − 448 = −1 ≡ 511 (mod 512)`, so `k = 511`. Then

```text
448 + 1 + 511 = 960
```

and `960 − 512 = 448`, so `960 ≡ 448 (mod 512)`. The length field
adds 64:

```text
960 + 64 = 1024 = 2 × 512
```

So `N = 2`. By Proposition J5.3 the zero bytes after `0x80` number
`(511 − 7) / 8 = 63`. The message is 56 bytes, which is eight bytes
short of one block. Those eight bytes are `0x80` and seven `0x00`
bytes: the word `0x80000000` and the word `0x00000000`. The
remaining zero bytes are `63 − 7 = 56`, which fill the second block
up to the length. `448 = 0x1c0`, so the last word is `0x000001c0`. □

The first word of that message is the ASCII bytes of “a”, “b”, “c”,
and “d”: `0x61`, `0x62`, `0x63`, `0x64`, hence `0x61626364`. The
examples file prints that word as `W[0]` of the first block, and it
prints `W[14] = 80000000`, `W[15] = 00000000`, and, in the second
block, `W[15] = 000001C0`.

The concatenation that builds these bytes is `++`. A string literal
is its ASCII bytes. `hex"80"` is the one byte `0x80`. `[0; 52]` is
fifty-two zero bytes. The length is not written as
`24 as big Word[8]^8`. An integer literal has no type of its own, and
`as` requires an operand that already has one.

**Listing J5.1 — `bare_length.or`, intentionally rejected**

```orange
edition 2026;
module bare_length {
  spec abc_block() -> Word[8]^64 {
    "abc" ++ hex"80" ++ [0; 52] ++ (24 as big Word[8]^8)
  }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0220]: the operand of `as` has no type of its own
 --> <stdin>:4:37
  |
4 |     "abc" ++ hex"80" ++ [0; 52] ++ (24 as big Word[8]^8)
  |                                     ^^ a literal takes its type from where it is used
  = note: write the literal where its type is required, or give it a type with a `let` binding
```

The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic and no value. The locus is the literal `24`.
The repair the note describes is a binding that gives the integer a
type before the conversion. `as big` is the form this binary
accepts for Assumption J5.3. The proposal that specifies it,
OEP-0017, is in review and is not accepted by this lesson. Using
the form is not acceptance of the proposal.

A conversion also cannot be indexed in the same expression. The
sixteen words are a binding. The index, when one is needed, is
applied to that name.

**Listing J5.2 — `indexed_as.or`, intentionally rejected**

```orange
edition 2026;
module indexed_as {
  spec word() -> Word[32] {
    let block: Word[8]^64 = [0; 64];
    (block[0..4] as big Word[32]^1)[0]
  }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0101]: expected `}` after the body expression
 --> <stdin>:5:36
  |
5 |     (block[0..4] as big Word[32]^1)[0]
  |                                    ^ found LEFT_BRACKET
  = note: a typed `spec` body holds `let` bindings, if any, and then one result expression
```

The status is 1. Standard output is empty. The note says the body
is bindings and then one result. The bracket after the conversion
is a second piece. Listing J5.3 binds the sixteen words in one
result and compares the whole array, so it never indexes that
conversion.

**Listing J5.3 — `pad.or`**

```orange
edition 2026;
module pad {
  spec abc_block() -> Word[8]^64 {
    let bits: Int = 24;
    let length: Word[8]^8 = bits as big Word[8]^8;
    "abc" ++ hex"80" ++ [0; 52] ++ length
  }

  spec abc_words() -> Word[32]^16 { abc_block() as big Word[32]^16 }

  spec two_padded() -> Word[8]^128 {
    let bits: Int = 448;
    let length: Word[8]^8 = bits as big Word[8]^8;
    "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
      ++ hex"80" ++ [0; 63] ++ length
  }

  spec two_block1() -> Word[32]^16 {
    let bytes: Word[8]^128 = two_padded();
    bytes[0..64] as big Word[32]^16
  }

  spec two_block2() -> Word[32]^16 {
    let bytes: Word[8]^128 = two_padded();
    bytes[64..128] as big Word[32]^16
  }

  test "FIPS 180-4 5.1.1 abc block" {
    abc_words() == [
      0x61626380, 0x00000000, 0x00000000, 0x00000000,
      0x00000000, 0x00000000, 0x00000000, 0x00000000,
      0x00000000, 0x00000000, 0x00000000, 0x00000000,
      0x00000000, 0x00000000, 0x00000000, 0x00000018,
    ]
  }

  test "examples file two-block, block 1" {
    two_block1() == [
      0x61626364, 0x62636465, 0x63646566, 0x64656667,
      0x65666768, 0x66676869, 0x6768696a, 0x68696a6b,
      0x696a6b6c, 0x6a6b6c6d, 0x6b6c6d6e, 0x6c6d6e6f,
      0x6d6e6f70, 0x6e6f7071, 0x80000000, 0x00000000,
    ]
  }

  test "examples file two-block, length word" {
    let w: Word[32]^16 = two_block2();
    w[15] == 0x000001c0
  }
}
```

`bits` is the `let` Listing J5.1 did not have. `abc_words` is §5.2.1:
the 512 bits of the block as sixteen 32-bit words, first byte most
significant. The first test is Proposition J5.2 and the word
`0x61626380` from the paragraph after Proposition J5.3, against all
sixteen positions. The second test is the examples file's first
block, including the two words Proposition J5.4 derived and the
fourteen message words that file prints. The third test is the
length word of the second block. The fourteen words of the second
block that this third test does not name are zero by Proposition
J5.4; the test does not repeat them.

```sh
./compiler/target/debug/orangec check pad.or
./compiler/target/debug/orangec eval --spec abc_words pad.or
./compiler/target/debug/orangec test pad.or
```

Check is silent. The status is 0.

**Expected evaluation output:**

```text
pad::abc_words: Word[32]^16 = [0x61626380, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000018]
```

**Test report:**

```text
test "FIPS 180-4 5.1.1 abc block" ... ok
test "examples file two-block, block 1" ... ok
test "examples file two-block, length word" ... ok
3 tests: 3 passed, 0 failed
```

Each `ok` is one `Bool`. The first `Bool` does not run the second.
Do not call that Match verified.

### J5.3 The six functions, and the first words of `H(0)` and `K`

§4.1.2 prints six equations. In the notation of Assumption J5.4:

```text
Ch(x, y, z)  = (x ∧ y) ⊕ (¬x ∧ z)
Maj(x, y, z) = (x ∧ y) ⊕ (x ∧ z) ⊕ (y ∧ z)
Σ0(x)        = ROTR 2(x) ⊕ ROTR 13(x) ⊕ ROTR 22(x)
Σ1(x)        = ROTR 6(x) ⊕ ROTR 11(x) ⊕ ROTR 25(x)
σ0(x)        = ROTR 7(x) ⊕ ROTR 18(x) ⊕ SHR 3(x)
σ1(x)        = ROTR 17(x) ⊕ ROTR 19(x) ⊕ SHR 10(x)
```

§3.2 defines `SHR n(x)` as the right shift `x >> n`, and
`ROTR n(x)` as `(x >> n) ∨ (x << (32 − n))`. The shift discards the
low `n` bits. The rotation carries them to the high end. The
standard writes `∨` in that definition. For two bits that are never
both `1`, `∨` and `⊕` agree. The low `n` bits of `x >> n` are the
bits that arrived as zeros, and the high `n` bits of
`x << (32 − n)` are the bits that were the low `n` bits of `x`.
They occupy different positions, so the `∨` in §3.2 is the same
word as the `⊕` of those two shifted values. `>>> n` is that word.

The standard's letters `σ` and `Σ` are not identifiers here.

**Listing J5.4 — `greek.or`, intentionally rejected**

```orange
edition 2026;
module greek {
  spec σ0(x: Word[32]) -> Word[32] { x }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0001]: unexpected character U+03C3
 --> <stdin>:3:8
  |
3 |   spec \u{3c3}0(x: Word[32]) -> Word[32] { x }
  |        ^^^^^^^ character is not part of Orange 2026
  = note: identifiers are ASCII in this pre-alpha edition
```

The status is 1. Standard output is empty. The character is
U+03C3, the small sigma in `σ0`. The diagnostic prints it as an
escape. The note states the limit: identifiers are ASCII. The names
in Assumption J5.4 are the repair. There is no accepted spelling of
`Σ0{256}` on this slice.

§5.3.3 sets `H(0)` to eight words and says they were obtained by
taking the first thirty-two bits of the fractional parts of the
square roots of the first eight primes. The first prime is `2`. The
first word is derived here. The other seven words are the hex the
section prints. The same extraction applies to them. This lesson
does not repeat it seven times.

**Proposition J5.5.** Let `h = 1779033703`, which is the hex word
`6a09e667`. Let `s = 2^32 + h = 6074000999`. Then
`s^2 ≤ 2^65 < (s + 1)^2`. The first 32 bits of the fractional part
of `√2` are the bits of `h`, and §5.3.3 prints that word as
`H0(0)`.

*Proof.* `1^2 = 1 < 2 < 4 = 2^2`, so the integer part of `√2` is
`1`. Multiplying by `2^32` gives
`√2 · 2^32 = √(2^65)`. The floor of `√(2^65)` is the greatest
integer whose square is at most `2^65`.

Expand `s^2 = (2^32 + h)^2 = 2^64 + h·2^33 + h^2`.

```text
h · 2^33 = 15281783145733554176
h^2      = 3164960916409892209
```

The second line is `h` multiplied by `h`. The first is `h` shifted
left by 33 bits, which is multiplication by `2^33`. Their sum is
`18446744062143446385`. Adding `2^64 = 18446744073709551616` gives

```text
s^2 = 36893488135852998001
```

`2^65 = 36893488147419103232`. The difference is

```text
2^65 − s^2 = 11566105231
```

which is positive, so `s^2 ≤ 2^65`. Also

```text
(s + 1)^2 − s^2 = 2s + 1 = 12148001999
```

and

```text
12148001999 − 11566105231 = 581896768
```

so `(s + 1)^2 − 2^65 = 581896768 > 0`. Therefore
`2^65 < (s + 1)^2`, the floor of `√(2^65)` is `s`, and removing the
integer part `1` shifted by 32 bits leaves `h`. □

The two products are finite multiplications. The ledger at the end
of the lesson recomputes them. They are not a citation of the hex
word in place of the arithmetic.

§4.2.2 says the sixty-four constants are the first thirty-two bits
of the fractional parts of the cube roots of the first sixty-four
primes, and then prints the words from left to right. The first
prime is again `2`. The first word is derived. The other sixty-three
are the hex the section prints, copied in that order into the
listing in §J5.6. This lesson does not repeat the cube-root
extraction sixty-three times.

**Proposition J5.6.** Let `k = 1116352408`, which is the hex word
`428a2f98`. Let `c = 2^32 + k = 5411319704`. Then
`c^3 ≤ 2^97 < (c + 1)^3`. The first 32 bits of the fractional part
of the cube root of `2` are the bits of `k`, and §4.2.2 prints that
word first.

*Proof.* `1^3 = 1 < 2 < 8 = 2^3`, so the integer part of the cube
root of `2` is `1`. Multiplying by `2^32` gives a quantity whose
cube is `2 · 2^96 = 2^97`. The floor is the greatest integer whose
cube is at most `2^97`.

```text
c^3   = 158456324954696271903413425664
2^97  = 158456325028528675187087900672
```

The difference `2^97 − c^3 = 73832403283674475008` is positive, so
`c^3 ≤ 2^97`. The next cube exceeds `2^97` by
`14014739549255426953`, so `2^97 < (c + 1)^3`. The floor is `c`,
and removing the integer part leaves `k`. □

Round 0 of “abc” applies the four large functions to words of
`H(0)`. Those words are §5.3.3:

```text
a = H0 = 6a09e667
b = H1 = bb67ae85
c = H2 = 3c6ef372
e = H4 = 510e527f
f = H5 = 9b05688c
g = H6 = 1f83d9ab
```

`ROTR 2` of `a` is the operation §3.2 defines, done once in full.
`a` ends in the two bits `11`. Shifting right by 2 discards them
and yields `0x1a827999`. Shifting left by `32 − 2 = 30` places
those two bits at the high end, which is `0xc0000000`. They occupy
different positions, so

```text
ROTR 2(a) = 0x1a827999 ⊕ 0xc0000000 = 0xda827999
```

`ROTR 13(a) = 0x333b504f` and `ROTR 22(a) = 0x27999da8` are the same
operation at the other two distances. Their exclusive or is

```text
0xda827999 ⊕ 0x333b504f ⊕ 0x27999da8 = 0xce20b47e
```

which is `Σ0(a)`. The same exclusive or of the three rotations of
`e` is `Σ1(e) = 0x3587272b`. The rotations are
`ROTR 6(e) = 0xfd443949`, `ROTR 11(e) = 0x4fea21ca`, and
`ROTR 25(e) = 0x87293fa8`.

`Ch` and `Maj` are the two bitwise equations, applied to these
words:

```text
e ∧ f       = 0x1104400c
(¬e) ∧ g    = 0x0e818980
Ch(e, f, g) = 0x1f85c98c

a ∧ b       = 0x2a01a605
a ∧ c       = 0x2808e262
b ∧ c       = 0x3866a200
Maj(a, b, c)= 0x3a6fe667
```

Each line is one application of `&` or `^` or `~` to the word on
its left. Listing J5.5 checks the four results.

**Listing J5.5 — `functions.or`**

```orange
edition 2026;
module functions {
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (~x & z) }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }

  spec ch_round0() -> Word[32] { ch(0x510e527f, 0x9b05688c, 0x1f83d9ab) }
  spec maj_round0() -> Word[32] { maj(0x6a09e667, 0xbb67ae85, 0x3c6ef372) }
  spec sigma0_a() -> Word[32] { big_sigma0(0x6a09e667) }
  spec sigma1_e() -> Word[32] { big_sigma1(0x510e527f) }

  test "FIPS 180-4 eq 4.2, Ch on H4 H5 H6" { ch_round0() == 0x1f85c98c }
  test "FIPS 180-4 eq 4.3, Maj on H0 H1 H2" { maj_round0() == 0x3a6fe667 }
  test "FIPS 180-4 eq 4.4, Sigma0 on H0" { sigma0_a() == 0xce20b47e }
  test "FIPS 180-4 eq 4.5, Sigma1 on H4" { sigma1_e() == 0x3587272b }
}
```

The bodies are equations (4.2) through (4.7), with the ASCII names.
`σ0` and `σ1` are defined here because the schedule uses them. This
listing does not apply them yet.

```sh
./compiler/target/debug/orangec check functions.or
./compiler/target/debug/orangec eval --spec ch_round0 functions.or
./compiler/target/debug/orangec test functions.or
```

Check is silent.

**Expected evaluation output:**

```text
functions::ch_round0: Word[32] = 0x1f85c98c
```

**Test report:**

```text
test "FIPS 180-4 eq 4.2, Ch on H4 H5 H6" ... ok
test "FIPS 180-4 eq 4.3, Maj on H0 H1 H2" ... ok
test "FIPS 180-4 eq 4.4, Sigma0 on H0" ... ok
test "FIPS 180-4 eq 4.5, Sigma1 on H4" ... ok
4 tests: 4 passed, 0 failed
```

Four `Bool`s. The command `eval --spec ch_round0` runs one of the
four parameterless specs. It does not run the tests.

### J5.4 The message schedule

§6.2.2, step 1. For each block `M(i)`, and for `t` from `0` through
`15`, `Wt` is the word `Mt(i)` from §5.2.1. For `t` from `16`
through `63`,

```text
Wt = σ1(W{t−2}) + W{t−7} + σ0(W{t−15}) + W{t−16}
```

The four `+` signs are Assumption J5.2. A sum of four words, each
reduced as it is added, equals the integer sum reduced once.
`(X + Y) mod m = ((X mod m) + Y) mod m` when `m` is positive. Apply
that identity from the left, three times. The order of the four
terms is the order the standard prints.

On the “abc” block, `W0 = 0x61626380`, `W1` through `W14` are `0`,
and `W15 = 0x00000018`.

**Proposition J5.7.** `W16 = 0x61626380`.

*Proof.* The recurrence at `t = 16` reads `W14`, `W9`, `W1`, and
`W0`. The first three are `0`. `σ1(0) = 0` and `σ0(0) = 0`, because
every rotation and every shift of the zero word is zero, and the
exclusive or of zeros is zero. The sum is `W0`. □

**Proposition J5.8.** `W17 = σ1(0x00000018) = 0x000f0000`.

*Proof.* At `t = 17` the recurrence reads `W15`, `W10`, `W2`, and
`W1`. The last three subscripts in that list, `W10`, `W2`, and
`W1`, are zero. `W15 = 0x18`. So `W17 = σ1(0x18)`.

`0x18` is the integer `24`. Its only set bits are bits 3 and 4,
counting from 0 at the low end. `SHR 10(0x18) = 0`, because a shift
of 10 discards both of those bits.

`ROTR 17(0x18)`: the right shift by 17 is `0`. The left shift by
`32 − 17 = 15` is `24 × 2^15`. `2^15 = 32768`, and
`24 × 32768 = 786432 = 0x000c0000`.

`ROTR 19(0x18)`: the right shift by 19 is `0`. The left shift by
`13` is `24 × 8192 = 196608 = 0x00030000`.

The exclusive or is

```text
0x000c0000 ⊕ 0x00030000 ⊕ 0x00000000 = 0x000f0000
```

because the set bits of the two rotations are disjoint: bits 18 and
19 from the first, bits 16 and 17 from the second. □

The index `t` in `16..64` takes the integers `16` through `63`. The
end is exclusive. Those sixty-four positions are inside an array of
64 words. The smallest subscript the body reads is `t − 16` at
`t = 16`, which is `0`. The largest is `t` at `t = 63`, which is
`63`. Every index the recurrence names is in range for that reason.

**Listing J5.6 — `schedule.or`**

```orange
edition 2026;
module schedule {
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }

  spec abc_words() -> Word[32]^16 {
    let bits: Int = 24;
    let length: Word[8]^8 = bits as big Word[8]^8;
    let block: Word[8]^64 = "abc" ++ hex"80" ++ [0; 52] ++ length;
    block as big Word[32]^16
  }

  spec message_schedule(block: Word[32]^16) -> Word[32]^64 {
    for t in 16..64 with w: Word[32]^64 = block ++ [0; 48] {
      w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
    }
  }

  spec w16() -> Word[32] {
    let w: Word[32]^64 = message_schedule(abc_words());
    w[16]
  }

  spec w17() -> Word[32] {
    let w: Word[32]^64 = message_schedule(abc_words());
    w[17]
  }

  test "FIPS 180-4 6.2.2 abc W16" { w16() == 0x61626380 }
  test "FIPS 180-4 6.2.2 abc W17" { w17() == 0x000f0000 }
}
```

The first sixteen words are the block, joined to forty-eight zeros
so the array has length 64 before `t` starts. Each iteration
replaces one zero. The replacement is the recurrence, in the
standard's order. `w with [t] = ...` is a new array. Orange has no
assignment to an existing word.

```sh
./compiler/target/debug/orangec check schedule.or
./compiler/target/debug/orangec eval --spec w16 schedule.or
./compiler/target/debug/orangec eval --spec w17 schedule.or
./compiler/target/debug/orangec test schedule.or
```

Check is silent.

**Expected evaluation output:**

```text
schedule::w16: Word[32] = 0x61626380
schedule::w17: Word[32] = 0x000f0000
```

**Test report:**

```text
test "FIPS 180-4 6.2.2 abc W16" ... ok
test "FIPS 180-4 6.2.2 abc W17" ... ok
2 tests: 2 passed, 0 failed
```

The two tests are Propositions J5.7 and J5.8. They do not constrain
`W18` through `W63`. Those words are computed by the same
recurrence when the digest in §J5.6 is computed. A Match on `W16`
and `W17` is not a Match on the other forty-six.

### J5.5 Round 0

§6.2.2, step 2, copies `H(i−1)` into `a` through `h`. For the first
block, `i = 1` and `H(i−1)` is `H(0)`.

Step 3, for `t` from `0` through `63`:

```text
T1 = h + Σ1(e) + Ch(e, f, g) + Kt + Wt
T2 = Σ0(a) + Maj(a, b, c)
h = g
g = f
f = e
e = d + T1
d = c
c = b
b = a
a = T1 + T2
```

The letters on the right of each assignment are the values those
letters have when the assignment is reached. `T1` and `T2` are both
computed from the values at the start of the round. The eight
assignments then move those old values. Orange does not assign to
`a`. The eight new values are one tuple, in the order
`(a, b, c, d, e, f, g, h)`:

```text
(T1 + T2, a, b, c, d + T1, e, f, g)
```

The names on the right are the old names. That is the same order as
the eight lines, read as new values rather than as assignments.

For `t = 0` on “abc”, `K0 = 0x428a2f98` by Proposition J5.6, and
`W0 = 0x61626380`. The four function values are the ones §J5.3
computed. `h = 0x5be0cd19` and `d = 0xa54ff53a`, the last and the
fourth words of §5.3.3.

**Proposition J5.9.** The integer sum inside `T1` is `5718561000`.
The residue modulo `2^32` is `0x54da50e8`. The integer sum inside
`T2` is `4438661861`. The residue is `0x08909ae5`. The new `a` is
`0x5d6aebcd` and does not cross `2^32`. The new `e` is `0xfa2a4622`
and does not cross `2^32`.

*Proof.* The five terms of `T1`, as integers, are

```text
h            1541459225
Σ1(e)         898049835
Ch(e, f, g)   528861580
K0           1116352408
W0           1633837952
```

Add from the top. `1541459225 + 898049835 = 2439509060`.
`2439509060 + 528861580 = 2968370640`.
`2968370640 + 1116352408 = 4084723048`.
`4084723048 + 1633837952 = 5718561000`.

`5718561000 − 4294967296 = 1423593704`, and
`1423593704 = 0x54da50e8`. One subtraction of `2^32` lands in
range, so the residue is that difference. That is `T1`.

The two terms of `T2` are `Σ0(a) = 3458249854` and
`Maj(a, b, c) = 980412007`. Their sum is `4438661861`.
`4438661861 − 4294967296 = 143694565 = 0x08909ae5`. That is `T2`.

The new `a` is `T1 + T2` as integers:
`1423593704 + 143694565 = 1567288269 = 0x5d6aebcd`. This sum is
below `2^32`, so the residue is the sum itself.

The new `e` is `d + T1`. `d = 2773480762`.
`2773480762 + 1423593704 = 4197074466 = 0xfa2a4622`, which is also
below `2^32`. □

The other six new words are the old `a`, `b`, `c`, `e`, `f`, and
`g`, unmoved. The examples file prints the row `t = 0` as

```text
5D6AEBCD 6A09E667 BB67AE85 3C6EF372 FA2A4622 510E527F 9B05688C 1F83D9AB
```

which is that tuple. The file's later rows are the same step on the
values the previous row produced. This lesson derives row `t = 0`
and does not derive row `t = 63` by repeating the arithmetic
sixty-three more times.

**Listing J5.7 — `round0.or`**

```orange
edition 2026;
module round0 {
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (~x & z) }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }

  spec t1() -> Word[32] {
    let h: Word[32] = 0x5be0cd19;
    let e: Word[32] = 0x510e527f;
    let f: Word[32] = 0x9b05688c;
    let g: Word[32] = 0x1f83d9ab;
    h + big_sigma1(e) + ch(e, f, g) + 0x428a2f98 + 0x61626380
  }

  spec t2() -> Word[32] {
    let a: Word[32] = 0x6a09e667;
    let b: Word[32] = 0xbb67ae85;
    let c: Word[32] = 0x3c6ef372;
    big_sigma0(a) + maj(a, b, c)
  }

  spec after() -> Word[32]^8 {
    let a: Word[32] = 0x6a09e667;
    let b: Word[32] = 0xbb67ae85;
    let c: Word[32] = 0x3c6ef372;
    let d: Word[32] = 0xa54ff53a;
    let e: Word[32] = 0x510e527f;
    let f: Word[32] = 0x9b05688c;
    let g: Word[32] = 0x1f83d9ab;
    let h: Word[32] = 0x5be0cd19;
    let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + 0x428a2f98 + 0x61626380;
    let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
    [t1 + t2, a, b, c, d + t1, e, f, g]
  }

  test "FIPS 180-4 6.2.2 abc T1" { t1() == 0x54da50e8 }
  test "FIPS 180-4 6.2.2 abc T2" { t2() == 0x08909ae5 }
  test "examples file abc, t = 0" {
    after() == [
      0x5d6aebcd, 0x6a09e667, 0xbb67ae85, 0x3c6ef372,
      0xfa2a4622, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
    ]
  }
}
```

`t1` and `t2` are the two residues in Proposition J5.9. `after` is
the eight-word row. The third test quotes the examples file. It
does not quote §6.2.2, because §6.2.2 does not print the row.

```sh
./compiler/target/debug/orangec check round0.or
./compiler/target/debug/orangec eval --spec t1 round0.or
./compiler/target/debug/orangec eval --spec t2 round0.or
./compiler/target/debug/orangec eval --spec after round0.or
./compiler/target/debug/orangec test round0.or
```

Check is silent.

**Expected evaluation output:**

```text
round0::t1: Word[32] = 0x54da50e8
round0::t2: Word[32] = 0x08909ae5
round0::after: Word[32]^8 = [0x5d6aebcd, 0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xfa2a4622, 0x510e527f, 0x9b05688c, 0x1f83d9ab]
```

**Test report:**

```text
test "FIPS 180-4 6.2.2 abc T1" ... ok
test "FIPS 180-4 6.2.2 abc T2" ... ok
test "examples file abc, t = 0" ... ok
3 tests: 3 passed, 0 failed
```

`T2` prints with eight hex digits, `0x08909ae5`. The leading zero
is part of the 32-bit word.

### J5.6 The two known answers

Step 4 adds each working variable to the corresponding word of
`H(i−1)`. After `N` blocks, the digest is the eight words of `H(N)`
concatenated, each word under Assumption J5.3. On this compiler
that concatenation is `hash as big Word[8]^32`: thirty-two bytes,
first byte most significant.

For “abc”, `N = 1`. One call of the round, sixty-four times, then
one step-4 addition, is the whole hash computation. For the
56-byte message, `N = 2`. The standard's loop is `i = 1` to `N`.
The listing's loop is `i` in `0..2`, which is the integers `0` and
`1`. Block `i + 1` of the standard is the slice
`padded[64 * i .. 64 * i + 64]`. Both slices lie inside the
128-byte padded message: the first starts at byte `0`, and the
second ends at byte `128`.

The bounds are literals. This slice can also give a `spec` a
length parameter drawn from a static range, as in `len in 1..2`.
These listings do not use that form. Each padded message has the
length Proposition J5.2 or J5.4 fixed. A message of any other
length, including the empty message and every length below `2^64`
that §6.2 allows, is not denoted by `abc` or by `two`.

The examples file, after `t = 63` of the second block, prints eight
sums and the digest

```text
248D6A61 D20638B8 E5C02693 0C3E6039 A33CE459 64FF2167 F6ECEDD4 19DB06C1
```

Before that second block it prints the eight words of the
intermediate hash, the value this lesson calls `two_h1`:

```text
85E655D6 417A1795 3363376A 624CDE5C 76E09589 CAC5F811 CC4B32C1 F20E533A
```

The one-block digest in the same file is

```text
BA7816BF 8F01CFEA 414140DE 5DAE2223 B00361A3 96177A9C B410FF61 F20015AD
```

N13 already matched the one-block line. The test here matches it
again, from a derivation rather than from a transcription that was
not derived, and it matches the two lines N13's test did not cover.

**Listing J5.8 — `sha256.or`**

```orange
edition 2026;
module sha256 {
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (~x & z) }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }

  spec round_constants() -> Word[32]^64 {
    [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ]
  }

  spec initial_hash() -> Word[32]^8 {
    [
      0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ]
  }

  spec schedule(block: Word[8]^64) -> Word[32]^64 {
    let head: Word[32]^16 = block as big Word[32]^16;
    for t in 16..64 with w: Word[32]^64 = head ++ [0; 48] {
      w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
    }
  }

  spec compress(hash: Word[32]^8, block: Word[8]^64) -> Word[32]^8 {
    let w: Word[32]^64 = schedule(block);
    let k: Word[32]^64 = round_constants();
    let (a: Word[32], b: Word[32], c: Word[32], d: Word[32],
         e: Word[32], f: Word[32], g: Word[32], h: Word[32]) =
      for t in 0..64 with (a: Word[32], b: Word[32], c: Word[32], d: Word[32],
                           e: Word[32], f: Word[32], g: Word[32], h: Word[32]) =
        (hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7]) {
        let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + k[t] + w[t];
        let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
        (t1 + t2, a, b, c, d + t1, e, f, g)
      };
    [
      a + hash[0], b + hash[1], c + hash[2], d + hash[3],
      e + hash[4], f + hash[5], g + hash[6], h + hash[7],
    ]
  }

  spec abc_block() -> Word[8]^64 {
    let bits: Int = 24;
    let length: Word[8]^8 = bits as big Word[8]^8;
    "abc" ++ hex"80" ++ [0; 52] ++ length
  }

  spec two_padded() -> Word[8]^128 {
    let bits: Int = 448;
    let length: Word[8]^8 = bits as big Word[8]^8;
    "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
      ++ hex"80" ++ [0; 63] ++ length
  }

  spec abc_hash() -> Word[32]^8 { compress(initial_hash(), abc_block()) }

  spec abc() -> Word[8]^32 {
    let hash: Word[32]^8 = abc_hash();
    hash as big Word[8]^32
  }

  spec two_h1() -> Word[32]^8 {
    let padded: Word[8]^128 = two_padded();
    compress(initial_hash(), padded[0..64])
  }

  spec two() -> Word[8]^32 {
    let padded: Word[8]^128 = two_padded();
    let hash: Word[32]^8 = for i in 0..2 with h: Word[32]^8 = initial_hash() {
      compress(h, padded[64 * i..64 * i + 64])
    };
    hash as big Word[8]^32
  }

  test "examples file abc digest" {
    abc() == [
      0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde,
      0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
      0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    ]
  }

  test "examples file two-block H1" {
    two_h1() == [
      0x85e655d6, 0x417a1795, 0x3363376a, 0x624cde5c,
      0x76e09589, 0xcac5f811, 0xcc4b32c1, 0xf20e533a,
    ]
  }

  test "examples file two-block digest" {
    two() == [
      0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8, 0xe5, 0xc0, 0x26, 0x93,
      0x0c, 0x3e, 0x60, 0x39, 0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67,
      0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1,
    ]
  }
}
```

`round_constants` is §4.2.2 in the order the section prints, left
to right, eight words to a row. `initial_hash` is §5.3.3.
`schedule` is step 1. The loop inside `compress` is step 3, and the
array after the loop is step 4. `t` in `0..64` is `t` from `0`
through `63`. The tuple is the eight lines of step 3. `K0` was
derived. `K1` through `K63` are copied. `H0` was derived. `H1`
through `H7` are copied.

```sh
./compiler/target/debug/orangec check sha256.or
./compiler/target/debug/orangec eval --spec abc sha256.or
./compiler/target/debug/orangec eval --spec two_h1 sha256.or
./compiler/target/debug/orangec eval --spec two sha256.or
./compiler/target/debug/orangec test sha256.or
```

Check is silent. A silent check means the source was well-formed.
It does not mean a digest matched.

**Expected evaluation output:**

```text
sha256::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]
sha256::two_h1: Word[32]^8 = [0x85e655d6, 0x417a1795, 0x3363376a, 0x624cde5c, 0x76e09589, 0xcac5f811, 0xcc4b32c1, 0xf20e533a]
sha256::two: Word[8]^32 = [0x24, 0x8d, 0x6a, 0x61, 0xd2, 0x06, 0x38, 0xb8, 0xe5, 0xc0, 0x26, 0x93, 0x0c, 0x3e, 0x60, 0x39, 0xa3, 0x3c, 0xe4, 0x59, 0x64, 0xff, 0x21, 0x67, 0xf6, 0xec, 0xed, 0xd4, 0x19, 0xdb, 0x06, 0xc1]
```

**Test report:**

```text
test "examples file abc digest" ... ok
test "examples file two-block H1" ... ok
test "examples file two-block digest" ... ok
3 tests: 3 passed, 0 failed
```

The first test is the one-block digest, thirty-two bytes, in the
order the examples file prints them. The second is the intermediate
hash after the first block of the two-block sample, eight words.
The third is the two-block digest. Passing the first does not run
the third. Passing all three does not run a fourth message.

What these three `Bool`s do not establish, under Assumption J5.6:

- They do not establish the empty message. `ℓ = 0` is allowed by
  §6.2 and is not an input of either spec.
- They do not establish SHA-224 or SHA-512.
- They do not establish a message whose bit length is not a
  multiple of 8.
- They do not establish `W18` through `W63` except as inputs to a
  digest that matched. A wrong word that the two digests happened
  not to notice would still be wrong. The `W16` and `W17` tests are
  a different pair of `Bool`s, on one block.
- They do not establish Appendix A.1.
- They do not establish that every block agrees with §6.2.2. The
  domain is the two messages written in the tests.
- They do not say anything about the time the evaluator takes.

Do not call that Match verified.

### J5.7 A rotation written as a shift

`σ1` uses `ROTR 17`, `ROTR 19`, and `SHR 10`. `>>>` and `>>` are
both accepted syntax. Replacing the first with a shift is a
different function, and check does not reject it.

**Proposition J5.10.** If `ROTR 17` in `σ1` is replaced by `SHR 17`,
then the “abc” schedule still has `W16 = 0x61626380`, and it has
`W17 = 0x00030000`.

*Proof.* `W16` depends on `σ1(W14)`, and `W14 = 0`. The shift of
zero is zero, as the rotation of zero was. Proposition J5.7 does
not use the difference between `ROTR` and `SHR`.

`W17 = σ1(0x18)` in Proposition J5.8, except that the first term
is now `SHR 17(0x18)`. A shift of 17 discards bits 3 and 4, so that
term is `0`. The other two terms are unchanged:
`ROTR 19(0x18) = 0x00030000` and `SHR 10(0x18) = 0`. The exclusive
or is `0x00030000`. □

The prediction, before the test runs, is that the program computes
`0x00030000` where the test demands `0x000f0000`.

**Listing J5.9 — `shr_for_rotr.or`**

```orange
edition 2026;
module shr_for_rotr {
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >> 17) ^ (x >>> 19) ^ (x >> 10) }

  spec abc_words() -> Word[32]^16 {
    let bits: Int = 24;
    let length: Word[8]^8 = bits as big Word[8]^8;
    let block: Word[8]^64 = "abc" ++ hex"80" ++ [0; 52] ++ length;
    block as big Word[32]^16
  }

  spec message_schedule(block: Word[32]^16) -> Word[32]^64 {
    for t in 16..64 with w: Word[32]^64 = block ++ [0; 48] {
      w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
    }
  }

  spec w16() -> Word[32] {
    let w: Word[32]^64 = message_schedule(abc_words());
    w[16]
  }

  spec w17() -> Word[32] {
    let w: Word[32]^64 = message_schedule(abc_words());
    w[17]
  }

  test "FIPS 180-4 6.2.2 abc W16" { w16() == 0x61626380 }
  test "FIPS 180-4 6.2.2 abc W17" { w17() == 0x000f0000 }
}
```

The only change from Listing J5.6 is `>> 17` in `small_sigma1`.
The expected word in the second test is still the word Proposition
J5.8 derived. The test is right. The function is not.

```sh
./compiler/target/debug/orangec check shr_for_rotr.or
./compiler/target/debug/orangec eval --spec w17 shr_for_rotr.or
./compiler/target/debug/orangec test shr_for_rotr.or
```

Check is silent. Silence here means the shifted function was
well-formed. The evaluation prints the word Proposition J5.10
predicted.

**Expected evaluation output:**

```text
shr_for_rotr::w17: Word[32] = 0x00030000
```

**Test report:**

```text
test "FIPS 180-4 6.2.2 abc W16" ... ok
test "FIPS 180-4 6.2.2 abc W17" ... FAILED
    left:  0x00030000
    right: 0x000f0000
2 tests: 1 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. `left` is the
value the program computed. `right` is the value the test demanded.
The first test passed, which is the half of Proposition J5.10 that
says `W16` does not see this fault. A suite that stopped at `W16`
would accept the shift. The repair is to put `>>> 17` back, which
is Listing J5.6. No other line needs to change.

### J5.8 Exercises

**Exercise J5.1.** For “abc”, state `ℓ`, `k`, the number of zero
bytes after `0x80`, and the padded length in bits. Name the
proposition each number comes from.

**Exercise J5.2.** For the 56-byte message, state `ℓ`, `k`, the
number of zero bytes after `0x80`, and which block holds the length
word. What is that word?

**Exercise J5.3.** Derive `W16` and `W17` of the “abc” schedule.
Show `ROTR 17(0x18)`, `ROTR 19(0x18)`, and `SHR 10(0x18)`.

**Exercise J5.4.** Give the integer sum inside `T1` before
reduction, the multiple of `2^32` that is subtracted, and the
residue. Do the same for `T2`. Does the new `e` cross `2^32`?

**Exercise J5.5.** `σ1` is rewritten so that its first operation is
`SHR 17` rather than `ROTR 17`. What is `W17`? Why does the test of
`W16` still pass? What does check report?

**Exercise J5.6.** The two-block digest test passed. Name six
things it does not establish. One of them is a message this lesson's
specs do not contain. One of them is SHA-224. One of them is
Appendix A.1.

**Exercise J5.7.** N14 listed five headings a reader still carries
out of that lesson: edition, endianness, padding, module seam, and
non-claims. For each heading, say what this lesson does with it.
The zero fill in an HMAC key block is not this padding.

**Exercise J5.8.** Listing J5.2 is rejected with `ORC0101`. What
does the note say a body may contain, and what binding repairs the
expression so that it still denotes a word? Separately, why is
`σ0` rejected, and what is the name this lesson uses instead?

## Worked answers

**J5.1.** `ℓ = 24` by `8 × 3`, as §5.1.1 states. `k = 423` by
Proposition J5.1, because `447 − 24 = 423`. The zero bytes after
`0x80` number `(423 − 7) / 8 = 52`, by Proposition J5.3. The padded
length is `512` bits, by Proposition J5.2.

**J5.2.** `ℓ = 448`. `k = 511`, because `447 − 448 ≡ 511 (mod 512)`.
The zero bytes number `(511 − 7) / 8 = 63`. The first block takes
the 56 message bytes, the byte `0x80`, and seven zero bytes, so the
length word is in the second block. That word is `0x000001c0`.
Proposition J5.4 is this answer.

**J5.3.** `W16 = 0x61626380`, because the other three terms at
`t = 16` are zero. Proposition J5.7.
`ROTR 17(0x18) = 0x000c0000`, `ROTR 19(0x18) = 0x00030000`, and
`SHR 10(0x18) = 0`. Their exclusive or is `0x000f0000`, which is
`W17`. Proposition J5.8.

**J5.4.** The integer sum inside `T1` is `5718561000`. Subtract
`2^32` once: `5718561000 − 4294967296 = 1423593704 = 0x54da50e8`.
The integer sum inside `T2` is `4438661861`. Subtract `2^32` once:
`4438661861 − 4294967296 = 143694565 = 0x08909ae5`. The new `e` is
`4197074466 = 0xfa2a4622`, which is less than `2^32`, so that sum
does not cross the modulus. Proposition J5.9.

**J5.5.** `W17 = 0x00030000`, by Proposition J5.10. `W16` still
passes because it applies `σ1` only to the zero word, and the shift
and the rotation agree on zero. Check is silent. The failure is the
test report, `left` `0x00030000` and `right` `0x000f0000`, not a
diagnostic.

**J5.6.** The two-block digest test does not establish the empty
message. It does not establish SHA-224. It does not establish
SHA-512. It does not establish a message whose length in bits is
not a multiple of 8. It does not establish Appendix A.1. It does
not establish every 512-bit block. It does not, by itself,
establish `W16`. Six of those are enough. The empty message is a
message the specs do not contain. SHA-224 is §6.3. Appendix A.1 is
the security discussion this lesson does not transcribe.

**J5.7.** Edition: Assumption J5.1 pins FIPS PUB 180-4, August
2015, and separates it from the examples file. The Orange line
`edition 2026;` is a different pin, the language edition, and it
does not choose the publication. Endianness: Assumption J5.3 is
§3.1, and the listings that read a block use `as big`. A listing
that never reads a byte has not chosen an order; these listings
do choose one. Padding: Propositions J5.2 through J5.4 are §5.1.1.
The zero bytes in an HMAC key block are a different padding, the
one N13 and N14 kept on the key. This lesson does not build that
block. Module seam: every listing is one module. Nothing here
calls another file. A comment is not a seam. Non-claims:
Assumption J5.6. The three digest `Bool`s are the list in §J5.6.
A Match is not called verified.

**J5.8.** The note on `ORC0101` says a typed `spec` body holds
`let` bindings, if any, and then one result expression. The repair
is to bind the conversion and index the name. Listing J5.3 does
the stronger thing and compares the whole sixteen-word array, so
the index is unnecessary for that test. `σ0` is rejected because
the name begins with U+03C3, which is not an ASCII identifier. The
diagnostic is `ORC0001`. The name this lesson uses is
`small_sigma0`.

```text
j5-ledger
abc-bits = 24
abc-k = 423
abc-zero-bytes = 52
abc-padded-bits = 512
two-bits = 448
two-k = 511
two-zero-bytes = 63
two-padded-bits = 1024
h0 = 1779033703
sqrt-shift = 15281783145733554176
sqrt-square = 3164960916409892209
sqrt-gap = 11566105231
sqrt-next = 581896768
k0 = 1116352408
cbrt-gap = 73832403283674475008
cbrt-next = 14014739549255426953
w0 = 1633837952
rotr17 = 786432
rotr19 = 196608
w17 = 983040
wrong-w17 = 196608
t1-raw = 5718561000
t1 = 1423593704
t2-raw = 4438661861
t2 = 143694565
new-a = 1567288269
new-e = 4197074466
```

## Sources and epigraph record

The quotation is the borrowed wording. The proofs are the lesson's.
The hex words copied from §4.2.2 and §5.3.3, other than the two
words Propositions J5.5 and J5.6 derive, are the standard's. The
digest bytes are the examples file's.

**[S16] National Institute of Standards and Technology.** FIPS PUB
180-4, *Secure Hash Standard (SHS)*, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. The epigraph is the Abstract, both
sentences, on page iii of the PDF. The sentences contain no inner
quotation marks. Wording was checked on 2026-10-09 against the file
whose SHA-256 is recorded in §J5.1. No translation is involved. The
publication names no personal author of the Abstract. The foreword
is signed by Charles H. Romine, Director, Information Technology
Laboratory; the epigraph is not taken from the foreword. The same
file is N13's [D1]. This record is [S16] so that the epigraph tag
is not a second definition of [D1].
Sections used for the derivation, and not as the epigraph, are
§3.1, §3.2, §4.1.2, §4.2.2, §5.1.1, §5.2.1, §5.3.3, §6.2.1, and
§6.2.2. §6.3 and §6.4 are named and not transcribed. Appendix A.1
is named and not transcribed. Appendix A.2 is the sentence that
points at examples and does not print a digest. The erratum page
records the SHA-1 edit described in §J5.1.
This record's tag is [S16].

Source: <https://doi.org/10.6028/NIST.FIPS.180-4>

**[T12] Retrieved files.** On 2026-10-09 the lesson retrieved the
August 2015 PDF from the DOI above. Its SHA-256 is
`0455b406d89648d20cbde375561e19c245b9815e894164c2670772e3d54deb82`.
The same day the lesson retrieved the NIST SHA-256 examples file
N13 recorded as [N1]. Its SHA-256 is
`7006b6549dad2fc8c6f29417a921f2e48208157ef496a7e1e1d7d17c5cc1e7db`.
That file is not a section of [S16]. The one-block digest, the
two-block digest, the block contents, the row `t = 0`, and the
intermediate hash after the first of the two blocks are copied
from it.
This record's tag is [T12].

Source: <https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA256.pdf>

**[C6] Compression surface.** The listings use `edition 2026`,
`module`, `spec`, `let`, `Word[8]`, `Word[32]`, `Int`, `^`, `&`,
`~`, `+`, `>>`, `>>>`, `++`, `hex"80"`, a string literal, `[0; n]`,
`for`, `with`, `as big`, a tuple accumulator, and `test`. `as big`
is implemented on the S3u binary and is specified by OEP-0017,
which this lesson does not accept. Identifiers are ASCII. An
integer literal is not an operand of `as` until a `let` types it.
A conversion is not indexed in the same expression. There is no
bit-string type and no assignment. A `for` bound in these listings
is a literal, and the end is exclusive. A length parameter in a
static range is implemented on this slice and is not used. The
version line these commands print is `orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`.
This record's tag is [C6].

The test form is the one N12 recorded as [T6]. `orangec check`
checks a test and does not run it. `orangec eval --spec` runs one
parameterless spec and does not run tests. `orangec test` runs the
tests. A failed check prints the diagnostic and no report. A failed
test prints `left` and `right` and no diagnostic.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

J5 adds eight exercises with worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`. The Orange listings
are the nine fenced programs in this file.
`compiler/crates/orangec/tests/book_j5.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not prove
Assumption J5.2 from a smaller theory of the compiler, and they do
not accept OEP-0017. Reaching the last page is not a certificate.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-09 at the
owner's direction. Owner review is pending.
