# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J7: Keccak-p and SHA-3. Draft 2026-10-17.

Continue from
[SHA-512 and the Rest of the SHA-2 Family](JOURNEYMAN_J6_SHA512_AND_THE_SHA2_FAMILY.md#j6-sha-512-and-the-rest-of-the-sha-2-family).
J6 derived SHA-512 at the width §6.4 of FIPS 180-4 uses, and the initial
values SHA-384 and SHA-512/t substitute for it. This lesson is the other
hash standard of August 2015. It writes Keccak-p as FIPS 202 writes it,
at the two lane widths this compiler can check from one text, and then
SHA3-256 on a whole-byte message.
[Standards as Versioned Inputs](JOURNEYMAN_J2_STANDARDS_AS_VERSIONED_INPUTS.md#j2-standards-as-versioned-inputs),
[The Corpus as Acceptance Test](JOURNEYMAN_J3_THE_CORPUS_AS_ACCEPTANCE_TEST.md#j3-the-corpus-as-acceptance-test),
and
[Byte Order and Format Boundaries](JOURNEYMAN_J4_BYTE_ORDER_AND_FORMAT_BOUNDARIES.md#j4-byte-order-and-format-boundaries)
are Block A. They are in this tree. This lesson cites their titles.
It does not repeat those proofs.

This lesson is **J7**. The locked label is J7. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. When
the text says “§3.2.1” or “§6.1”, the number is a section of the
edition named in the same sentence.

## J7: Keccak-p and SHA-3

> “This Standard specifies the Secure Hash Algorithm-3 (SHA-3) family of functions on binary data.”
>
> — National Institute of Standards and Technology, FIPS PUB 202, *SHA-3 Standard: Permutation-Based Hash and Extendable-Output Functions* (August 2015), Abstract, first sentence. [S18]

The sentence is the first sentence of the Abstract, on page iii. It
names the family and says the functions take binary data. The next
sentence names KECCAK. The Abstract's later sentences name the four
hash functions, the two extendable-output functions, and three
applications. This lesson does not adopt a sentence about collision
resistance, and it does not prove one. The announcement on page iv
opens with a different sentence, “This Standard (FIPS 202) specifies
the Secure Hash Algorithm-3 (SHA-3) family of functions on binary
data.” That sentence is not the epigraph. The object on this page is
the permutation §3 defines and the one hash §6.1 defines from it.

**The only-this-stack test.** You will derive the state, the five
step mappings, the round count of Keccak-f, the pad of a whole-byte
SHA3-256 message, and the domain suffix `01` from the August 2015
text, then run the transcription with `orangec check`, `orangec
eval`, and `orangec test` on the compiler this tree builds. You will
also feed the compiler programs it rejects, and one program it
accepts whose tests fail. The compiler's version line is
`orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`. Every
listing is written for that slice. Every fenced program is Current
on that slice. No listing is marked Proposed. A form this slice does
not implement is not given a syntax here. A program the compiler
rejects is still Current: the rejection is the result the listing
exists to show.

### J7.1 The sections, in the order the standard uses them

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can name the sections of FIPS PUB 202, August 2015, that
   define the state, θ, ρ, π, χ, ι, Keccak-p, Keccak-f, `pad10*1`,
   and SHA3-256, in the order §6.1 uses them, and you can separate
   that edition from the files that print the bytes.
2. You can say which columns of Table 1 are words on this slice, and
   what the compiler prints for `Word[w]`, for a repeated `^`, for a
   pair of type parameters, and for `Word[1]`.
3. You can compute θ on the state whose only 1-bit is lane `(0, 0)`,
   the offset `(t + 1)(t + 2) / 2`, and the reason the ρ walk meets
   every nonzero lane once.
4. You can show that π permutes the 25 lanes, and that χ permutes
   each 5-bit row.
5. You can say why a rotation of every lane by the same amount
   commutes with θ, ρ, π, and χ, and why ι at round 0 with round
   constant `0x01` does not. You can say why Keccak-f uses
   `nr = 12 + 2l` and indexes those rounds from `0` through `11 + 2l`.
6. You can transcribe the two listed widths so that Keccak-f[200] of
   the zero state, SHA3-256 of “abc”, and SHA3-256 of the empty
   message match the bytes this lesson pins, and you can predict the
   six failures before the report: a swapped lane index, a rotation
   offset one past the standard, a missing final pad bit, the domain
   bits omitted, the domain bits written in the other order, and
   big-endian lanes.

The edition is FIPS PUB 202, *SHA-3 Standard: Permutation-Based Hash
and Extendable-Output Functions*, August 2015, DOI
`10.6028/NIST.FIPS.202`. The file retrieved on 2026-10-17 from
<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf> has
SHA-256
`1592607831ff0908cc590632ce371c6c95e94025bb1a0c8ae90a4d0ec1ed025e`.
In Algorithm 6, step 3, the round-constant bit position is `2` with
`j` set as a superscript and then minus 1. A plain-text extraction
prints that superscript on the baseline. The position was read from
the PDF's own character boxes: the `j` sits above the line. The same
reading gives `RC = 0` with `w` as a superscript, a string of `w`
zeros.

**Assumption J7.1 — The pin is the edition, not the title.**
“SHA-3” names the competition, the submission, and this standard.
The algorithms, the section numbers, and the bit equations below are
the August 2015 file. Lesson J2 is the lesson that makes a pin a
procedure. This lesson does not depend on that text. The pin is
stated in this assumption.

**Assumption J7.2 — The state is a 5-by-5-by-w array, and the string
is lane order.** §3.1 sets `w = b / 25` and `l = log2(w)`. Table 1
gives seven columns. For `0 ≤ x < 5`, `0 ≤ y < 5`, and `0 ≤ z < w`,

```text
A[x, y, z] = S[w(5y + x) + z]
```

§3.1.3 builds the string back by lanes. Lane `(i, j)` is
`A[i, j, 0]` through `A[i, j, w − 1]`, plane `j` is those five lanes
in order of `i`, and `S` is the five planes in order of `j`. The
glossary calls a sheet the bits of one fixed `x`. In the listings a
sheet is `Word[w]^5` indexed by `y`, and the state is five sheets
indexed by `x`, so `a[x][y]` is lane `(x, y)`. That is the indexing
§3.1.2 writes.

**Assumption J7.3 — A word on this slice is 8, 16, 32, or 64 bits.**
Table 1 also has `w = 1`, `w = 2`, and `w = 4`. Those three widths
are not words in this compiler. `Word[16]` and `Word[32]` are words.
They are not in the list the transcription instantiates. The reason
is §J7.2.

**Assumption J7.4 — The step mappings are Algorithms 1 through 6, and
one round is their composition.** §3.2 defines θ, ρ, π, χ, and ι.
§3.3 sets `Rnd(A, ir) = ι(χ(π(ρ(θ(A)))), ir)`. Addition of
coordinates `x` and `y` is modulo 5. The bit equations use `⊕` for
exclusive or, `·` for the and inside χ, and a subtraction inside a
lane index modulo `w`. In Orange, `⊕` is `^`, the and is `&`,
negation of a bit is `^ 1` on that bit and `~` on a whole lane, and
a rotation of a lane toward its high bits is `<<<`. Parentheses
follow the equations. Orange gives operators from different groups
no relative precedence; the parentheses are written where the
compiler requires them.

**Assumption J7.5 — Keccak-f is the case `nr = 12 + 2l`.** §3.4
defines `KECCAK-f[b] = KECCAK-p[b, 12 + 2l]`. Algorithm 7 then
indexes `ir` from `12 + 2l − nr` through `12 + 2l − 1`. Substituting
that `nr` makes the index run from `0` through `11 + 2l`. The
constant `12` is the index origin Algorithm 7 and §3.4 write. This
lesson does not derive `12` from a smaller formula, and it does not
treat the constant as a margin that has been proved here.

**Assumption J7.6 — SHA3-256 is one sponge input, on whole bytes.**
§6.1 writes `SHA3-256(M) = KECCAK[512](M || 01, 256)`. §5.2 sets
`KECCAK[c] = SPONGE[KECCAK-p[1600, 24], pad10*1, 1600 − c]`, so the
rate is `1088` and the capacity is `512`. Algorithm 9 writes
`pad10*1`. Appendix B.2 writes the same pad as bytes when the
message length is a multiple of 8. The listings take that byte form
for `q > 2`. A message whose length in bits is not a multiple of 8
is not built on this page. The two NIST files that print a 5-bit
message and a 30-bit message are named in [T14] and are not run.

**Assumption J7.7 — The examples are not a section of the standard.**
FIPS 202 does not print `SHA3-256("abc")`. It does not print
Keccak-f[200] of the zero state. Appendix B.2 prints the shape of
the byte pad and does not print a digest. The bytes cited below are
the files [T14] names, or the bytes Python's `hashlib.sha3_256`
prints for the same messages, or both. Lesson J3 is the lesson that
treats a finite set of pinned vectors as an acceptance test. This
lesson does not depend on that text. The acceptance set is the tests
in Listing J7.7, and the coverage sentence is stated in §J7.11.

**Assumption J7.8 — A Match is one `Bool` on the inputs written in
that test.** Well-formed source is not a Match. A Match on these
inputs is not a Match on every input.
A Match is not called verified.
Nothing on this page is a security claim. A count of evaluator steps
is the number `orangec test --stats` prints for a test. It is not a
count of machine operations, and this lesson draws no conclusion
about speed from it.

**Assumption J7.9 — The hand calculation and the program denote the
same lane only where both perform the same operations under J7.2
through J7.5.** Agreement on one state is one point. The domain of
one 8-bit lane is `256` inputs. The domain of a 200-bit state is
`2^200` inputs. The domain of a 1600-bit state is `2^1600` inputs.

Figure 1 of the August 2015 file is the picture of sheets, planes,
slices, rows, columns, and lanes for `b = 200`. The algebraic
definitions are the glossary in §2.1. §3.1.2 and §3.1.3 are the
conversions this lesson uses. §3.2.1 through §3.2.5 are the five
steps. §3.3 is Keccak-p. §3.4 is the comparison with Keccak-f. §4
names the sponge and is not transcribed as a general loop. §5.1 is
the pad. §5.2 is `KECCAK[c]`. §6.1 is the four hash functions; this
lesson runs SHA3-256. §6.2 is SHAKE128 and SHAKE256. The curriculum
names that study as J8. This lesson does not append `1111` and does
not return a chosen length. HMAC is the construction the curriculum
names as J14. This lesson does not build a key, a key block, or an
outer hash.

### J7.2 Seven widths, and the word this slice has

Table 1 is seven columns.

```text
b    25   50   100   200   400   800   1600
w     1    2     4     8    16    32     64
l     0    1     2     3     4     5      6
```

`b = 25w` and `l = log2(w)`, so `w = 2^l` and `nr = 12 + 2l` is
`12, 14, 16, 18, 20, 22, 24` across the columns. The last two rows
are the arithmetic of §3.1 and §3.4. The ledger recomputes them.

The curriculum row for this lesson asks for the step mappings once,
over a lane type `Word[w]`. The compiler slice that would check one
word parameter at every width is pull request 248, “Check one word
parameter at every width”. That request was open on 2026-10-17 and
is not in this tree. This lesson does not promise it. What follows
is the diagnostic this compiler prints for the form the row names,
and then the form the compiler does check.

Listing J7.1 gives the width a size parameter and writes `Word[w]`.

```orange
edition 2026;
module wordw {
  spec lane[w in 8..9](x: Word[w]) -> Word[8] { 0 }
}
```

`orangec check` rejects it. The diagnostic is a parse error, twice.
It is `ORC0101`. It is not `ORC0204`. The width of `Word` is parsed
as an integer token before any size is in scope.

```text
error[ORC0101]: expected `]` after the type width
 --> <stdin>:3:32
  |
3 |   spec lane[w in 8..9](x: Word[w]) -> Word[8] { 0 }
  |                                ^ found IDENTIFIER
  = note: width-parameter syntax is written `Name[WIDTH]`

error[ORC0101]: expected an integer width after `[`
 --> <stdin>:3:32
  |
3 |   spec lane[w in 8..9](x: Word[w]) -> Word[8] { 0 }
  |                                ^ found IDENTIFIER
  = note: width-parameter syntax is written `Name[WIDTH]`
```

A state is a 5-by-5 array of lanes. Writing both lengths on the
parameter, `W^5^5`, is also a parse error. Listing J7.2 is that
line. The diagnostic truncates the source line with `...` because
of its length. The listing keeps the long line so the columns stay
the columns the compiler prints.

```orange
edition 2026;
module nested {
  spec id[W in {Word[8], Word[64]}](a: W^5^5) -> Word[8] { 0 }
}
```

```text
error[ORC0101]: expected the end of the type after its array length
 --> <stdin>:3:43
  |
3 | ... spec id[W in {Word[8], Word[64]}](a: W^5^5) -> Word[8] { 0 }
  |                                             ^ found CARET
  = note: name the row type with a `type` declaration, then write `Row^LENGTH`; repeated `^` dimensions are not type syntax
```

The note tells the repair this lesson uses. Name the row. A sheet is
five lanes, so the row of a sheet is a word and the sheet is
`Word[8]^5` or `Word[64]^5`. The state is then one more `^5`. A
`type` declaration cannot take a parameter, so the two sheets are
two names, `Sheet8` and `Sheet64`.

A second type parameter does not tie the sheet to the state.
Listing J7.3 lists a sheet parameter and a state parameter. The
checker builds the product. `lane[Sheet8, State64]` is one of the
four instances, and in that instance `a[0][0]` is a `Word[64]` while
the result element is a `Word[8]`.

```orange
edition 2026;
module paired {
  type Sheet8 = Word[8]^5;
  type State8 = Sheet8^5;
  type Sheet64 = Word[64]^5;
  type State64 = Sheet64^5;
  spec lane[S in {Sheet8, Sheet64}, A in {State8, State64}](a: A) -> S {
    let z: S = [0; 5];
    z with [0] = a[0][0]
  }
}
```

```text
error[ORC0214]: this element has type `Word[64]`, but `Word[8]` is required here
 --> <stdin>:9:18
  |
9 |     z with [0] = a[0][0]
  |                  ^^^^^^^ expected `Word[8]`
  = note: Orange has no implicit conversions between types
  = note: in the instance `lane[Sheet8, State64]`, the first of `lane` in error: a function is checked once for each type of its type parameters
```

The second note is the rule this lesson has to live with. A function
is checked once for each type of its type parameters. “Once for each
listed type” is the implemented rule. “Once for every width, including
widths that are not listed” is the rule pull request 248 is about.
They are different rules.

Table 1's first column is `w = 1`. Listing J7.4 asks for `Word[1]`.

```orange
edition 2026;
module word1 {
  spec lane() -> Word[1] { 0 }
}
```

```text
error[ORC0204]: `Word` width must be exactly 8, 16, 32, or 64
 --> <stdin>:3:23
  |
3 |   spec lane() -> Word[1] { 0 }
  |                       ^ unsupported word width
  = note: word widths do not coerce, truncate, or wrap
```

`Word[2]` and `Word[4]` fail the same check. The three narrow
columns of Table 1 are not transcribed. `Word[16]` and `Word[32]`
would pass this particular check. They are not listed in Listing
J7.7, so those two widths are not instances of the step mappings on
this page.

A literal rotation by the whole width is a separate rejection, used
when ρ reduces an offset. Listing J7.5 rotates a `Word[8]` by the
literal `8`.

```orange
edition 2026;
module rot8 {
  spec turn(x: Word[8]) -> Word[8] { x <<< 8 }
}
```

```text
error[ORC0216]: `<<<` on `Word[8]` needs an amount from 0 through 7
 --> <stdin>:3:44
  |
3 | ... pec turn(x: Word[8]) -> Word[8] { x <<< 8 }
  |                                             ^ a literal amount is from 0 through 7
  = note: an amount written as one integer literal is from 0 through n - 1; any other amount is computed, an `Int` or a word, such as `x <<< r` or `x >> (i % 8)`
```

The note distinguishes a literal amount from a computed amount. The
computed amount is specified by OEP-0021, which this lesson does not
accept. The binary evaluates it. Listing J7.6 uses that evaluation
for the offsets that exceed the lane, and it records what the
evaluation does on all 256 bytes of a `Word[8]`: a computed `8` is
the identity, and a computed `153` equals a rotation by `1`.

The form that does check is one type parameter whose values are the
two sheets:

```text
S in {Sheet8, Sheet64}
```

with the state written `S^5`. That is two instances. The checker
checks the body twice, once at `w = 8` and once at `w = 64`. It does
not check the body once and then conclude the unchecked widths. A
call that returns `Int` and takes no argument does not select an
instance; the listings write `lane_width[S]()` and `ell[S]()` so the
sheet is named at the call. Two hundred fifty-six is the instance
cap. Two is the instance count of each parameterized spec in Listing
J7.7.

`algorithms/sha3/README.md` says a type parameter does not give `w`
as a size for the `25w`-bit string, for `l`, or for the round
indices `12 + 2l − nr`. The sentence about the string length is
right on this tree: Listing J7.7 has a 25-byte conversion for
Keccak-f[200] and a 136-byte block for SHA3-256, not one length
`25w`. The sentence about a `for` bound is also right: the bound is
a literal, and `nr` is a computed `Int`, so the loop is written
`0..24` and the rounds with `ir ≥ nr` are the identity. The step
mappings themselves are one text. That is all this slice gives.

### J7.3 Theta

Algorithm 1. For each `x` and `z`,

```text
C[x, z] = A[x, 0, z] ⊕ A[x, 1, z] ⊕ A[x, 2, z] ⊕ A[x, 3, z] ⊕ A[x, 4, z]
D[x, z] = C[(x − 1) mod 5, z] ⊕ C[(x + 1) mod 5, (z − 1) mod w]
A′[x, y, z] = A[x, y, z] ⊕ D[x, z]
```

`(x − 1) mod 5` is `(x + 4) mod 5`. `(z − 1) mod w` is a rotation of
that parity lane by one toward the high bit, written `<<< 1`. `D`
does not depend on `y`. Every lane of sheet `x` is xored with the
same `D[x]`.

**Proposition J7.1.** Let every lane be `0` except lane `(0, 0) = 1`,
at `w = 8`. Then θ produces `A′[0, 0] = 1`, `A′[1, 0] = 1`,
`A′[2, 0] = 0`, and `A′[4, y] = 2` for every `y`, including `y = 0`
and `y = 1`.

The column parities are `C[0] = 1` and `C[1] = C[2] = C[3] = C[4] = 0`.
Then

```text
D[0] = C[4] ⊕ (C[1] <<< 1) = 0
D[1] = C[0] ⊕ (C[2] <<< 1) = 1
D[2] = C[1] ⊕ (C[3] <<< 1) = 0
D[3] = C[2] ⊕ (C[4] <<< 1) = 0
D[4] = C[3] ⊕ (C[0] <<< 1) = 2
```

`1 <<< 1 = 2`. Xoring `D` into the state changes lane `(0, 0)` by
`0`, so it stays `1`; it writes `1` into every lane of sheet `1`,
including lane `(1, 0)`; it writes `0` into sheet `2`; it writes `2`
into every lane of sheet `4`. Listing J7.6 evaluates those five
lanes. The proof is the six lines above. The evaluation is one state.

### J7.4 Rho

Algorithm 2 leaves lane `(0, 0)` unrotated. It sets `(x, y) = (1, 0)`
and, for `t` from `0` through `23`, rotates lane `(x, y)` by

```text
(t + 1)(t + 2) / 2
```

and then replaces `(x, y)` with `(y, (2x + 3y) mod 5)`.

The offset is the triangular number `T(t + 1) = (t + 1)(t + 2) / 2`.
`T(1) = 1` at `t = 0`, and `T(24) = 300` at `t = 23`. Table 2 prints
the twenty-four offsets in lane coordinates, and it prints `0` at
`(0, 0)`, which the algorithm never computes because that lane is
copied. The twenty-four values are

```text
1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 66, 78,
91, 105, 120, 136, 153, 171, 190, 210, 231, 253, 276, 300
```

§3.2.2 says the offsets may be reduced modulo the lane size, and it
gives the example `153 mod 8 = 1` for lane `A[3, 2]` when `w = 8`.
`153 = 19 · 8 + 1`. The same paragraph's picture is `b = 200`. At
`w = 64`, `300 = 4 · 64 + 44`, so the offset of the lane whose table
entry is `300` reduces to `44`. Listing J7.5 has already rejected the
literal `<<< 8`. A computed amount is reduced by the evaluator.
Listing J7.6 checks the reduction on every byte: `x <<< 8` computed
equals `x`, and `x <<< 153` computed equals `x <<< 1`.

**Proposition J7.2.** The twenty-four offsets are those triangular
numbers. The walk visits each of the twenty-four nonzero lanes once
and returns to `(1, 0)`. Reduced modulo 8, in the string order
`5y + x`, the twenty-five bytes are

```text
00 01 06 04 03 04 04 06 07 04 03 02 03 01 07 01 05 07 05 00 02 02 05 00 06
```

The leading `00` is lane `(0, 0)`, which the walk does not write.

Let `τ(x, y) = (y, (2x + 3y) mod 5)`. As a matrix on `(Z/5)^2`,

```text
τ = | 0  1 |
    | 2  3 |
```

The determinant is `0 · 3 − 1 · 2 = −2 ≡ 3 (mod 5)`. `3` and `5` are
coprime, so `τ` is invertible. An invertible linear map sends only
`(0, 0)` to `(0, 0)`, and the walk starts at `(1, 0)`, so the walk
never writes lane `(0, 0)`. The orbit of `(1, 0)` therefore lives in
the other twenty-four lanes. Listing J7.6 writes a `1` at each
visited index `x + 5y` and adds those bytes. The sum is `24` and
index `0` is `0`, so the twenty-four steps are twenty-four distinct
nonzero lanes. The same walk's final pair is `(1, 0)`. A shorter
period would have repeated a lane, and the sum would be smaller. The
orbit is all twenty-four nonzero lanes, each once.

The XKCP file in [T14] prints the same residues as `RhoOffset[x][y]`
for Keccak-f[200]. The hex string in the test is those residues in
string order. The comparison is the test. The triangular formula is
the proposition.

### J7.5 Pi

Algorithm 3, for every `x`, `y`, and `z`:

```text
A′[x, y, z] = A[(x + 3y) mod 5, x, z]
```

The `z` index is copied. π moves whole lanes.

**Proposition J7.3.** The map `ψ(x, y) = ((x + 3y) mod 5, x)` is a
bijection of the twenty-five lanes.

As a matrix,

```text
ψ = | 1  3 |
    | 1  0 |
```

The determinant is `1 · 0 − 3 · 1 = −3 ≡ 2 (mod 5)`. `2` and `5` are
coprime, so `ψ` is bijective. Equivalently, `3 · 2 = 6 ≡ 1 (mod 5)`,
so `2` is the inverse of `3`. Given an output lane `(x′, y′)`, the
input that lands there has `x = y′` and `3y ≡ x′ − y′`, hence
`y ≡ 2(x′ − y′)`. Each output lane reads exactly one input lane.

Listing J7.6 counts, at each lane, how many times the formula reads
it. Every count is `1`. The count is the same bijection, checked on
all twenty-five lanes. Twenty-five is the whole domain. The count is
a proof for this modulus, not a sample.

### J7.6 Chi

Algorithm 4, for every `x`, `y`, and `z`:

```text
A′[x, y, z] = A[x, y, z] ⊕ ((A[(x + 1) mod 5, y, z] ⊕ 1) · A[(x + 2) mod 5, y, z])
```

The product is the and of two bits. The formula uses one row: five
bits of fixed `y` and fixed `z`. Distinct rows do not meet. On a
lane, the same formula runs at every `z`, which is `^`, `~`, and `&`
on the word. Listing J7.7 writes that word form. Listing J7.6 checks
the five bits, because invertibility is a fact about one row.

**Proposition J7.4.** On a row of five bits, χ is a bijection. Its
inverse is

```text
a[x] = b[x] ⊕ ((b[x + 1] ⊕ 1) · (b[x + 2] ⊕ ((b[x + 3] ⊕ 1) · b[x + 4])))
```

with every index modulo 5. Therefore χ on the state is a bijection.

There are `2^5 = 32` rows. Listing J7.6 evaluates both compositions
on each of them: χ inverse after χ, and χ after χ inverse. Both
return the input row, for every row. Thirty-two is the whole domain
of a 5-bit row. A map that has a two-sided inverse on its whole
finite domain is a bijection. The state map applies that row map
independently at each `(y, z)`, so the state map is a bijection too.
The formula uses neighbors at distances `1, 2, 3, 4`. Those four
distances are four distinct positions precisely because the row has
length 5. The proof is the enumeration at this length. It is not a
claim about a row of another length.

Two rows are short enough to do by hand. On the zero row every
neighbor is `0`, and `(1 · 0) = 0`, so χ sends `0` to `0`. The
inverse formula does the same, so it also sends `0` to `0`. On the
row whose only `1` is bit `0`:

```text
b[0] = 1 ⊕ ((0 ⊕ 1) · 0) = 1
b[1] = 0 ⊕ ((0 ⊕ 1) · 0) = 0
b[2] = 0 ⊕ ((0 ⊕ 1) · 0) = 0
b[3] = 0 ⊕ ((0 ⊕ 1) · 1) = 1
b[4] = 0 ⊕ ((1 ⊕ 1) · 0) = 0
```

The bits are `b[0] = 1` and `b[3] = 1`, which is the byte `0x09`.
Listing J7.6 prints `chi_of_0 = 0x00`, `chi_of_1 = 0x09`, and
`inv_of_0 = 0x00`.

### J7.7 Iota, and the round count

Algorithm 5 builds one bit `rc(t)` from an eight-bit register.
`R = 10000000` means `R[0] = 1` and the other seven bits are `0`.
For `i` from `1` to `t mod 255`, the algorithm prepends a `0`, xors
the bit that shifted off into positions `0, 4, 5, 6`, and keeps
eight bits. Stored with `R[0]` as the low bit, the register starts
at `1`, prepending a `0` is `<< 1`, and the four tap positions are
the low byte of `0x171` when bit 8 is set. Listing J7.7 follows that
register for `t` from `0` through `167`, which covers `j + 7·ir` for
`ir` from `0` through `23` and `j` from `0` through `6`.

Algorithm 6 copies the state, sets `RC` to `w` zero bits, and for
`j` from `0` through `l` writes `rc(j + 7·ir)` into bit `2^j − 1` of
`RC`. It then xors `RC` into lane `(0, 0)`. The other twenty-four
lanes are unchanged.

The positions for `j = 0, 1, 2, 3, 6` are `0, 1, 3, 7, 63`. At
`w = 8`, `l = 3`, so the bits that exist are `j = 0, 1, 2, 3`. At
`w = 64`, `l = 6`, so `j` runs through `6`. Listing J7.7 loops `j`
through `0..7`, seven iterations, for both sheets. A computed shift
by an amount at least the width evaluates to `0`. On a `Word[8]`,
the positions `15, 31, 63` are such shifts, and or-ing `0` does not
change the lane. On a `Word[64]`, `j` through `6` is exactly the
range Algorithm 6 writes. The loop is that range, not a search that
stops at a computed `l`.

**Proposition J7.5.** Let `T` rotate every lane by one bit, the same
rotation `<<< 1` at every `(x, y)`. Then θ, ρ, π, and χ commute with
`T`. ι at round `0` for `w = 8` does not: its round constant is
`0x01`, and `0x01 <<< 1 = 0x02`.

θ builds each parity by xoring five lanes. Xor commutes with a
rotation, so the parities of `T(A)` are the rotated parities of `A`.
`D` xors one parity with a one-bit rotation of another. Rotation by
one commutes with rotation by one, so `D` commutes with `T`. The
output xors `D[x]` into every lane of sheet `x`, and that xor
commutes with `T` as well.

ρ rotates each lane by a fixed offset, and lane `(0, 0)` by zero.
Two rotations of the same lane commute.

π moves lanes and does not change `z`. Moving the lanes and then
rotating each of them is the same as rotating each and then moving
them, because every lane receives the same rotation.

χ's formula at bit `z` uses only bits at that same `z`. Rotating
every lane of the row rotates the inputs and the output together.

ι xors a constant into lane `(0, 0)` and leaves the other lanes
alone. `T(ι(A))` rotates the constant's bits. `ι(T(A))` xors the
unrotated constant into the already rotated lane. Those agree for
every state only when the constant is fixed by `T`. `0x01` is not
fixed by a one-bit rotation. Round `0` at `w = 8` is the round whose
constant the XKCP file prints as `01`, and Listing J7.7 prints the
same byte. One constant that moves is enough to show that this ι
does not commute with `T`.

The algebraic argument is the proof. Listing J7.6 also evaluates θ
and `T` on the one-lane state of Proposition J7.1, in both orders.
That is one state. It is not the proof of the identity for every
state.

**Proposition J7.6.** For Keccak-f, Algorithm 7 indexes `ir` from
`0` through `11 + 2l` inclusive. That is `12 + 2l` rounds. At
`w = 8`, `l = 3` and the index runs through `17`, eighteen rounds.
At `w = 64`, `l = 6` and the index runs through `23`, twenty-four
rounds.

Substitute `nr = 12 + 2l` into the start index
`12 + 2l − nr`. The start is `0`. The end index is
`12 + 2l − 1 = 11 + 2l`. The number of integers from `0` through
`11 + 2l` is `12 + 2l`. Table 1 gives `l`. The count is that
substitution and nothing further. §3.4's example
`KECCAK-p[1600, 30]` starts at `12 + 2·6 − 30 = −6`. Listing J7.7
has no negative index and no caller-chosen `nr`.

The eighteen Keccak-f[200] round constants, low byte only, are the
bytes the XKCP file prints:

```text
01 82 8A 00 8B 01 81 09 8A 88 09 0A 8B 8B 89 03 02 80
```

Listing J7.6 is the state arithmetic above, at `w = 8`, where a
5-bit row fits in a byte and the offsets can be reduced by hand.

```orange
edition 2026;
module facts {
  type Sheet = Word[8]^5;
  type State = Sheet^5;
  type Grid = Sheet^5;
  type Z5 = Mod[5];

  spec theta8(a: State) -> State {
    let c: Sheet = for x in 0..5 with c: Sheet = [0; 5] {
      c with [x] = a[x][0] ^ a[x][1] ^ a[x][2] ^ a[x][3] ^ a[x][4]
    };
    for x in 0..5 with b: State = a {
      for y in 0..5 with t: State = b {
        t with [x][y] = a[x][y] ^ c[(x + 4) % 5] ^ (c[(x + 1) % 5] <<< 1)
      }
    }
  }

  spec one_state() -> State { [[0; 5]; 5] with [0][0] = 1 }

  spec theta_a00() -> Word[8] { theta8(one_state())[0][0] }
  spec theta_a10() -> Word[8] { theta8(one_state())[1][0] }
  spec theta_a20() -> Word[8] { theta8(one_state())[2][0] }
  spec theta_a40() -> Word[8] { theta8(one_state())[4][0] }
  spec theta_a41() -> Word[8] { theta8(one_state())[4][1] }

  spec offsets() -> Int^24 {
    for t in 0..24 with a: Int^24 = [0; 24] {
      a with [t] = ((t + 1) * (t + 2)) / 2
    }
  }

  spec table() -> Int^24 {
    [1, 3, 6, 10, 15, 21, 28, 36, 45, 55, 66, 78, 91, 105, 120, 136, 153, 171, 190, 210, 231, 253, 276, 300]
  }

  spec reduced8() -> Word[8]^25 {
    let walk: (Word[8]^25, Z5, Z5) =
      for t in 0..24 with (a: Word[8]^25, x: Z5, y: Z5) = ([0; 25], 1, 0) {
        let off: Int = ((t + 1) * (t + 2)) / 2;
        (
          a with [(x as Int) + (5 * (y as Int))] = (off % 8) as Word[8],
          y,
          (2 * x) + (3 * y),
        )
      };
    walk.0
  }

  spec visits() -> Word[8]^25 {
    let walk: (Word[8]^25, Z5, Z5) =
      for t in 0..24 with (a: Word[8]^25, x: Z5, y: Z5) = ([0; 25], 1, 0) {
        (a with [(x as Int) + (5 * (y as Int))] = 1, y, (2 * x) + (3 * y))
      };
    walk.0
  }

  spec visit_cover() -> Bool {
    let a: Word[8]^25 = visits();
    let n: Int = for i in 0..25 with n: Int = 0 { n + (a[i] as Int) };
    (a[0] == 0) && (n == 24)
  }

  spec rho_back() -> (Int, Int) {
    let walk: (Z5, Z5) = for t in 0..24 with (x: Z5, y: Z5) = (1, 0) {
      (y, (2 * x) + (3 * y))
    };
    (walk.0 as Int, walk.1 as Int)
  }

  spec pi_hits() -> Grid {
    for x in 0..5 with g: Grid = [[0; 5]; 5] {
      for y in 0..5 with h: Grid = g {
        h with [(x + (3 * y)) % 5][x] = h[(x + (3 * y)) % 5][x] + 1
      }
    }
  }

  spec pi_bijective() -> Bool {
    let g: Grid = pi_hits();
    for x in 0..5 with ok: Bool = true {
      for y in 0..5 with inner: Bool = ok { inner && (g[x][y] == 1) }
    }
  }

  spec bit(a: Word[8], i: Int) -> Word[8] { (a >> (i % 5)) & 1 }

  spec chi_row(a: Word[8]) -> Word[8] {
    for x in 0..5 with b: Word[8] = 0 {
      let ax: Word[8] = bit(a, x);
      let n1: Word[8] = bit(a, x + 1);
      let n2: Word[8] = bit(a, x + 2);
      b | ((ax ^ ((n1 ^ 1) & n2)) << x)
    }
  }

  spec chi_inv(a: Word[8]) -> Word[8] {
    for x in 0..5 with b: Word[8] = 0 {
      let ax: Word[8] = bit(a, x);
      let n1: Word[8] = bit(a, x + 1);
      let n2: Word[8] = bit(a, x + 2);
      let n3: Word[8] = bit(a, x + 3);
      let n4: Word[8] = bit(a, x + 4);
      let mid: Word[8] = n2 ^ ((n3 ^ 1) & n4);
      b | ((ax ^ ((n1 ^ 1) & mid)) << x)
    }
  }

  spec chi_roundtrip() -> Bool {
    for i in 0..32 with ok: Bool = true {
      let a: Word[8] = i as Word[8];
      (ok && (chi_inv(chi_row(a)) == a)) && (chi_row(chi_inv(a)) == a)
    }
  }

  spec chi_of_0() -> Word[8] { chi_row(0) }
  spec chi_of_1() -> Word[8] { chi_row(1) }
  spec inv_of_0() -> Word[8] { chi_inv(0) }

  spec rot_cover() -> Bool {
    let eight: Int = 8;
    let off: Int = 153;
    for i in 0..256 with ok: Bool = true {
      let x: Word[8] = i as Word[8];
      (ok && ((x <<< eight) == x)) && ((x <<< off) == (x <<< 1))
    }
  }

  spec spin8(a: State) -> State {
    for x in 0..5 with b: State = a {
      for y in 0..5 with t: State = b { t with [x][y] = a[x][y] <<< 1 }
    }
  }

  spec theta_commutes() -> Bool {
    let a: State = one_state();
    theta8(spin8(a)) == spin8(theta8(a))
  }

  test "theta on the lane (0, 0) = 1 state" {
    (theta_a00() == 1) && (theta_a10() == 1) && (theta_a20() == 0) && (theta_a40() == 2)
      && (theta_a41() == 2)
  }

  test "rho offsets are the triangular numbers, reduced mod 8" {
    ((offsets() == table()) && (rho_back() == (1, 0))) && (visit_cover() == true)
      && (reduced8() == hex"00010604030404060704030203010701050705000202050006")
  }

  test "pi hits every lane once" { pi_bijective() == true }

  test "chi and its inverse are mutual inverses on every 5-bit row" {
    (((chi_roundtrip() == true) && (chi_of_0() == 0)) && (chi_of_1() == 0x09)) && (inv_of_0() == 0)
  }

  test "a computed rotation reduces mod 8" { rot_cover() == true }

  test "theta commutes with a lane translation" { theta_commutes() == true }
}
```

`orangec check` is silent. `orangec eval` prints the lanes, the
return of the walk, the two row values, and the two Bool results.

```text
facts::theta_a00: Word[8] = 0x01
facts::theta_a10: Word[8] = 0x01
facts::theta_a20: Word[8] = 0x00
facts::theta_a40: Word[8] = 0x02
facts::theta_a41: Word[8] = 0x02
facts::rho_back: (Int, Int) = (1, 0)
facts::visit_cover: Bool = true
facts::pi_bijective: Bool = true
facts::chi_of_0: Word[8] = 0x00
facts::chi_of_1: Word[8] = 0x09
facts::inv_of_0: Word[8] = 0x00
facts::chi_roundtrip: Bool = true
facts::rot_cover: Bool = true
facts::theta_commutes: Bool = true
```

`orangec test` runs the six claims. Each claim is one `Bool`.

```text
test "theta on the lane (0, 0) = 1 state" ... ok
test "rho offsets are the triangular numbers, reduced mod 8" ... ok
test "pi hits every lane once" ... ok
test "chi and its inverse are mutual inverses on every 5-bit row" ... ok
test "a computed rotation reduces mod 8" ... ok
test "theta commutes with a lane translation" ... ok
6 tests: 6 passed, 0 failed
```

The evaluator-step counts are the counts Assumption J7.8 names.

```text
test "theta on the lane (0, 0) = 1 state": 4879 steps
test "rho offsets are the triangular numbers, reduced mod 8": 3306 steps
test "pi hits every lane once": 1051 steps
test "chi and its inverse are mutual inverses on every 5-bit row": 45446 steps
test "a computed rotation reduces mod 8": 4359 steps
test "theta commutes with a lane translation": 2577 steps
total: 61618 of 1048576 steps
```

### J7.8 Keccak-f at the two widths this slice lists

Listing J7.7 is Algorithms 1 through 7 for `S` in
`{Sheet8, Sheet64}`, and then the one-block SHA3-256 pad of §J7.9.

The lane width is not a size the body can read. `lane_width` starts
from the one-bit and counts left shifts until the word is `0`. A
shift by the width or more evaluates to `0`, so the count is the
width. The loop bound `0..65` is a literal past `64`. `ell` doubles
`1` until the value equals that width. The bound `0..7` can count a
double up to `2^6`. The tests demand `8` and `3` at the narrow
sheet, and `6` at the wide sheet. A width that is not a power of two
would finish the double loop without a match. Both listed widths are
powers of two.

`keccak_f` computes `nr = 12 + 2·ell` and loops `ir` in `0..24`. The
round runs when `ir < nr`. At `w = 8`, iterations `18` through `23`
return the state unchanged. At `w = 64`, every iteration runs.
Proposition J7.6 says those are the Keccak-f rounds. The end bound
`24` is the larger of the two round counts, written as a literal
because a `for` bound cannot be the computed `nr`. The function is
Keccak-f at the two listed sheets. It is not Keccak-p at a
caller-chosen round count.

The round-constant lane is built by the shift onto bit `2^j − 1`
described in §J7.7. `rc8` reads the eighteen low bytes at `w = 8`.
The test demands the XKCP bytes. Matching those eighteen bytes is a
comparison of the constants. Matching the zero-state permutation is
a comparison of the composition. The lesson prints both, and it does
not treat one as a substitute for the other.

Keccak-f[200] of the all-zero state is serialized by §3.1.3. The
byte at string index `5y + x` is lane `(x, y)`, and at `w = 8` a
lane is a byte. The XKCP file's final line for the all-zero input is

```text
3C 28 26 84 1C B3 5C 17 1E AA E9 B8 11 13 4C EA A3 85 2C 69 D2 C5 AB AF EA
```

That line is the output of the permutation. The file continues with
a second example whose input is that output. The second example is
not the zero-state result.

Keccak-f[1600] of the zero state is a 200-byte string. Lane `(0, 0)`
is its first eight bytes, little-endian, so the integer value of
that lane is the first lane of the string. The listing demands

```text
0xf1258f7940e1dde7
```

which is the value `compiler/fixtures/s3u/valid-keccak-state.or`
already demands for that lane. The module's `orangec test --stats`
run uses `974370` of `1048576` evaluator steps. The `74206` steps
that remain do not cover a second evaluation that would print all
200 bytes. The listing reports the lane. It does not report the
other twenty-four lanes of the 1600-bit state.

### J7.9 The pad, and SHA3-256

**Proposition J7.7.** Let `M` be a whole-byte message, let
`r = 1088`, and let `q = (r / 8) − ((len(M) / 8) mod (r / 8))` with
`q > 2`. The byte pad of SHA3-256 is

```text
M || 0x06 || 0^(q − 2) || 0x80
```

For “abc”, `len(M) = 24` bits, `q = 133`, the zero bytes number
`131`, the first pad byte is `0x06`, and the last is `0x80`. The
bit length of `N = M || 01` is `26`. Algorithm 9 at `x = 1088` gives
`j = 1060` zero bits between two `1` bits, and `26 + 1062 = 1088`.
For the empty message, `q = 136` and the zero bytes number `134`.

§6.1 appends the bit string `01` before the pad. The first bit of
that string is `0` and the second is `1`. Algorithm 9 then appends
`1`, then `j` zeros, then `1`. Appendix B.1 packs eight bits into a
byte with bit `0` of the string in the low bit of the byte. The
first pad byte is therefore bit `0` equal to `0`, bit `1` equal to
`1`, bit `2` equal to `1`, and bits `3` through `7` equal to `0`.
That byte is `0x06`. The final `1` of `pad10*1` is the last bit of
the rate. In the last byte it is bit `7`, which is `0x80`. Table 6
writes this case as `M || 0x06 || 0x00 … || 0x80` when `q > 2`.

The bit count is the same pad. `m = 26`, `j = (−26 − 2) mod 1088`.
`−28` modulo `1088` is `1088 − 28 = 1060`. The pad string has length
`1 + 1060 + 1 = 1062`. The sum `26 + 1062 = 1088` is one rate block.
No second block is absorbed. The digest length is `256`, which is
less than one rate, so the squeeze is the first `256` bits of the
state after that one permutation. Four lanes are `256` bits. The
listing reads lanes `(0, 0)` through `(3, 0)` and writes them
little-endian. It does not write the general squeeze loop of
Algorithm 8.

For “abc”, `3` bytes leave `q = 136 − 3 = 133`. The appended bytes
are one `0x06`, `131` zeros, and one `0x80`. For the empty message,
`q = 136`: one `0x06`, `134` zeros, and one `0x80`. The NIST file
for the 0-bit message prints the absorbed block beginning `06` and a
byte `80` at the end of that 136-byte block, and it prints the hash

```text
A7 FF C6 F8 BF 1E D7 66 51 C1 47 56 A0 61 D6 62
F5 80 FF 4D E4 3B 49 FA 82 D8 0A 4B 80 F8 43 4A
```

`hashlib.sha3_256` of the empty byte string prints the same bytes.
`hashlib.sha3_256` of the three bytes `abc` prints

```text
3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532
```

FIPS 202 does not print that second digest. The URL
`sha3-256_msg24.pdf` in the NIST examples directory answered `404`
on 2026-10-17. The in-tree fixture
`compiler/fixtures/s3u/valid-keccak-state.or` titles a test “FIPS
202 example: SHA3-256 of abc”. The title attributes the digest to
the PDF. The PDF does not contain it. This lesson does not repeat
that attribution. The pin is the `hashlib` byte string, the same
bytes the fixture demands, and the test in Listing J7.7.

The rate occupies the first `1088` bits of the state string, which
is seventeen lanes. Lane number `i` in that string is
`5y + x = i`, so `x = i mod 5` and `y = i / 5`. The listing xors
those seventeen little-endian lanes into a zero state. The lanes
past the rate stay `0`, which is the capacity of a zero state. Both
pinned messages fit in one block. A message that needs a second
block is not built here.

Six neighbors of that pad are the failures in §J7.10. Each is named
before Listing J7.7 runs.

The missing final bit keeps `0x06` and fills the rest of the block
with zeros: three message bytes, one `0x06`, and `132` zero bytes.
There is no `0x80`. The domain omitted uses `pad10*1` on the message
alone, so the next byte starts with the pad's leading `1` and the
rest of that byte is `0`. The byte is `0x01`, and the block still
ends in `0x80`. The domain bits in the other order are the bit
string `10` and then the pad's leading `1`: low bit `1`, next bit
`0`, next bit `1`. The byte is `0x05`. Big-endian lanes parse the
correct `0x06` block with `as big`. The standard's lane puts `z = 0`
in the first bit of the lane string, and Appendix B.1 puts that bit
in the low bit of the byte. `as little` is that packing. `as big` is
the other packing. Swapping the lane index writes string index
`5x + y` instead of `5y + x`. Adding one to every ρ offset is a
different permutation. The predictions of what those six compute are
in §J7.10, and they are stated again immediately before the report.

Listing J7.7 is the transcription.

```orange
edition 2026;
module keccak {
  type Sheet8 = Word[8]^5;
  type State8 = Sheet8^5;
  type Sheet64 = Word[64]^5;
  type State64 = Sheet64^5;
  type Z5 = Mod[5];

  spec theta[S in {Sheet8, Sheet64}](a: S^5) -> S^5 {
    let c: S = for x in 0..5 with c: S = [0; 5] {
      c with [x] = a[x][0] ^ a[x][1] ^ a[x][2] ^ a[x][3] ^ a[x][4]
    };
    for x in 0..5 with b: S^5 = a {
      for y in 0..5 with t: S^5 = b {
        t with [x][y] = a[x][y] ^ c[(x + 4) % 5] ^ (c[(x + 1) % 5] <<< 1)
      }
    }
  }

  spec rho_walk[S in {Sheet8, Sheet64}](a: S^5) -> (S^5, Z5, Z5) {
    for t in 0..24 with (b: S^5, x: Z5, y: Z5) = (a, 1, 0) {
      (
        b with [x as Int][y as Int] = a[x as Int][y as Int] <<< (((t + 1) * (t + 2)) / 2),
        y,
        2 * x + 3 * y,
      )
    }
  }

  spec rho[S in {Sheet8, Sheet64}](a: S^5) -> S^5 { rho_walk(a).0 }

  spec pi[S in {Sheet8, Sheet64}](a: S^5) -> S^5 {
    for x in 0..5 with b: S^5 = a {
      for y in 0..5 with t: S^5 = b { t with [x][y] = a[(x + 3 * y) % 5][x] }
    }
  }

  spec chi[S in {Sheet8, Sheet64}](a: S^5) -> S^5 {
    for x in 0..5 with b: S^5 = a {
      for y in 0..5 with t: S^5 = b {
        t with [x][y] = a[x][y] ^ (~a[(x + 1) % 5][y] & a[(x + 2) % 5][y])
      }
    }
  }

  spec iota[S in {Sheet8, Sheet64}](a: S^5, rc: S) -> S^5 {
    a with [0][0] = a[0][0] ^ rc[0]
  }

  spec rc_bits() -> Word[8]^168 {
    let walk: (Word[8]^168, Word[16]) =
      for t in 0..168 with (bits: Word[8]^168, r: Word[16]) = ([0; 168], 1) {
        let shifted: Word[16] = r << 1;
        (
          bits with [t] = (r & 1) as Word[8],
          if (shifted & 0x100) == 0 { shifted } else { shifted ^ 0x171 },
        )
      };
    walk.0
  }

  spec round_constants[S in {Sheet8, Sheet64}]() -> S^24 {
    let bits: Word[8]^168 = rc_bits();
    for ir in 0..24 with table: S^24 = [[0; 5]; 24] {
      let lane: S = for j in 0..7 with s: S = [0; 5] {
        let bit: Word[8] = bits[j + (7 * ir)];
        let z: S = [0; 5];
        let one: S = z with [0] = ~((~z[0]) << 1);
        let placed: S = one with [0] = one[0] << ((one[0] << j) - one[0]);
        if bit == 0 { s } else { s with [0] = s[0] | placed[0] }
      };
      table with [ir] = lane
    }
  }

  spec lane_width[S in {Sheet8, Sheet64}]() -> Int {
    let z: S = [0; 5];
    let one: S = z with [0] = ~((~z[0]) << 1);
    let walk: (Int, S) = for i in 0..65 with (n: Int, p: S) = (0, one) {
      if p[0] == z[0] { (n, p) } else { (n + 1, p with [0] = p[0] << 1) }
    };
    walk.0
  }

  spec ell[S in {Sheet8, Sheet64}]() -> Int {
    let width: Int = lane_width[S]();
    let walk: (Int, Int) = for i in 0..7 with (e: Int, p: Int) = (0, 1) {
      if p == width { (e, p) } else { (e + 1, p * 2) }
    };
    walk.0
  }

  spec keccak_f[S in {Sheet8, Sheet64}](a: S^5) -> S^5 {
    let nr: Int = 12 + (2 * ell[S]());
    let rcs: S^24 = round_constants[S]();
    for ir in 0..24 with s: S^5 = a {
      if ir < nr { iota(chi(pi(rho(theta(s)))), rcs[ir]) } else { s }
    }
  }

  spec zero8() -> State8 { [[0; 5]; 5] }

  spec bytes8(a: State8) -> Word[8]^25 {
    for y in 0..5 with out: Word[8]^25 = [0; 25] {
      for x in 0..5 with row: Word[8]^25 = out { row with [(5 * y) + x] = a[x][y] }
    }
  }

  spec f200() -> Word[8]^25 { bytes8(keccak_f(zero8())) }

  spec zero64() -> State64 { [[0; 5]; 5] }

  spec f1600_lane00() -> Word[64] { keccak_f(zero64())[0][0] }

  spec rc8_0() -> Word[8] { round_constants[Sheet8]()[0][0] }

  spec rc8_1() -> Word[8] { round_constants[Sheet8]()[1][0] }

  spec rc8() -> Word[8]^18 {
    let table: Sheet8^24 = round_constants[Sheet8]();
    for i in 0..18 with a: Word[8]^18 = [0; 18] { a with [i] = table[i][0] }
  }

  spec width8() -> Int { lane_width[Sheet8]() }

  spec ell8() -> Int { ell[Sheet8]() }

  spec ell64() -> Int { ell[Sheet64]() }

  test "rc and width" {
    ((((rc8_0() == 0x01) && (rc8_1() == 0x82)) && (width8() == 8)) && (ell8() == 3))
      && (ell64() == 6) && (rc8() == hex"01828a008b0181098a88090a8b8b89030280")
  }

  test "Keccak-f[200] of the zero state" {
    f200() == hex"3c2826841cb35c171eaae9b811134ceaa3852c69d2c5abafea"
  }

  test "Keccak-f[1600] lane (0, 0) of the zero state" {
    f1600_lane00() == 0xf1258f7940e1dde7
  }

  spec absorb256(block: Word[8]^136) -> Word[8]^32 {
    let lanes: Word[64]^17 = block as little Word[64]^17;
    let absorbed: State64 = for i in 0..17 with s: State64 = [[0; 5]; 5] {
      s with [i % 5][i / 5] = lanes[i]
    };
    let out: State64 = keccak_f(absorbed);
    [out[0][0], out[1][0], out[2][0], out[3][0]] as little Word[8]^32
  }

  spec abc_block() -> Word[8]^136 { ("abc" ++ (hex"06" ++ [0; 131])) ++ hex"80" }

  spec empty_block() -> Word[8]^136 { (hex"06" ++ [0; 134]) ++ hex"80" }

  spec sha3_256_abc() -> Word[8]^32 { absorb256(abc_block()) }

  spec sha3_256_empty() -> Word[8]^32 { absorb256(empty_block()) }

  spec abc_no_final() -> Word[8]^136 { "abc" ++ (hex"06" ++ [0; 132]) }

  spec abc_domain_omitted() -> Word[8]^136 { ("abc" ++ (hex"01" ++ [0; 131])) ++ hex"80" }

  spec abc_domain_reversed() -> Word[8]^136 { ("abc" ++ (hex"05" ++ [0; 131])) ++ hex"80" }

  spec abc_big() -> Word[8]^32 {
    let lanes: Word[64]^17 = abc_block() as big Word[64]^17;
    let absorbed: State64 = for i in 0..17 with s: State64 = [[0; 5]; 5] {
      s with [i % 5][i / 5] = lanes[i]
    };
    let out: State64 = keccak_f(absorbed);
    [out[0][0], out[1][0], out[2][0], out[3][0]] as little Word[8]^32
  }

  spec bytes8_swapped(a: State8) -> Word[8]^25 {
    for x in 0..5 with out: Word[8]^25 = [0; 25] {
      for y in 0..5 with row: Word[8]^25 = out { row with [(5 * x) + y] = a[x][y] }
    }
  }

  spec f200_swapped() -> Word[8]^25 { bytes8_swapped(keccak_f(zero8())) }

  spec rho_bad[S in {Sheet8, Sheet64}](a: S^5) -> S^5 {
    let walked: (S^5, Z5, Z5) = for t in 0..24 with (b: S^5, x: Z5, y: Z5) = (a, 1, 0) {
      (
        b with [x as Int][y as Int] =
          a[x as Int][y as Int] <<< ((((t + 1) * (t + 2)) / 2) + 1),
        y,
        2 * x + 3 * y,
      )
    };
    walked.0
  }

  spec keccak_f_bad_rho[S in {Sheet8, Sheet64}](a: S^5) -> S^5 {
    let nr: Int = 12 + (2 * ell[S]());
    let rcs: S^24 = round_constants[S]();
    for ir in 0..24 with s: S^5 = a {
      if ir < nr { iota(chi(pi(rho_bad(s))), rcs[ir]) } else { s }
    }
  }

  spec f200_bad_rho() -> Word[8]^25 { bytes8(keccak_f_bad_rho(zero8())) }

  spec no_final_byte() -> Word[8] { absorb256(abc_no_final())[0] }

  spec omitted_byte() -> Word[8] { absorb256(abc_domain_omitted())[0] }

  spec reversed_byte() -> Word[8] { absorb256(abc_domain_reversed())[0] }

  spec big_byte() -> Word[8] { abc_big()[0] }

  spec swapped_byte() -> Word[8] { f200_swapped()[1] }

  spec correct_byte() -> Word[8] { f200()[1] }

  spec bad_rho_byte() -> Word[8] { f200_bad_rho()[0] }

  test "SHA3-256 of abc" {
    sha3_256_abc() == hex"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
  }

  test "SHA3-256 of the empty message" {
    sha3_256_empty() == hex"a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
  }

  test "missing final pad bit" {
    absorb256(abc_no_final()) == hex"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
  }

  test "domain bits omitted" {
    absorb256(abc_domain_omitted()) == hex"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
  }

  test "domain bits reversed" {
    absorb256(abc_domain_reversed()) == hex"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
  }

  test "big-endian lanes" {
    abc_big() == hex"3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
  }

  test "swapped lane order" {
    f200_swapped() == hex"3c2826841cb35c171eaae9b811134ceaa3852c69d2c5abafea"
  }

  test "rotation offset plus one" {
    f200_bad_rho() == hex"3c2826841cb35c171eaae9b811134ceaa3852c69d2c5abafea"
  }
}
```

`orangec check` is silent. The values below are `orangec eval` on
the specs the passing tests compare. They are printed here, before
the report in §J7.10, because that report is one run of every test
in the module, including the six that fail.

```text
keccak::rc8_0: Word[8] = 0x01
keccak::rc8_1: Word[8] = 0x82
keccak::rc8: Word[8]^18 = [0x01, 0x82, 0x8a, 0x00, 0x8b, 0x01, 0x81, 0x09, 0x8a, 0x88, 0x09, 0x0a, 0x8b, 0x8b, 0x89, 0x03, 0x02, 0x80]
keccak::width8: Int = 8
keccak::ell8: Int = 3
keccak::ell64: Int = 6
keccak::f1600_lane00: Word[64] = 0xf1258f7940e1dde7
keccak::f200: Word[8]^25 = [0x3c, 0x28, 0x26, 0x84, 0x1c, 0xb3, 0x5c, 0x17, 0x1e, 0xaa, 0xe9, 0xb8, 0x11, 0x13, 0x4c, 0xea, 0xa3, 0x85, 0x2c, 0x69, 0xd2, 0xc5, 0xab, 0xaf, 0xea]
keccak::sha3_256_abc: Word[8]^32 = [0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32]
keccak::sha3_256_empty: Word[8]^32 = [0xa7, 0xff, 0xc6, 0xf8, 0xbf, 0x1e, 0xd7, 0x66, 0x51, 0xc1, 0x47, 0x56, 0xa0, 0x61, 0xd6, 0x62, 0xf5, 0x80, 0xff, 0x4d, 0xe4, 0x3b, 0x49, 0xfa, 0x82, 0xd8, 0x0a, 0x4b, 0x80, 0xf8, 0x43, 0x4a]
```

### J7.10 Six witnesses, predicted before the report

Each prediction is the claim the failing test then compares with a
correct digest or the correct Keccak-f[200] string. The first five
digest bytes that are not the swapped-lane byte are results of the
reference evaluator. They are not a hand expansion of twenty-four
rounds. They are written here, and the eval lines repeat them,
before the report. The swapped-lane byte is a hand argument from the
correct string and the index formula. It does not need a second
permutation.

**Missing final pad bit.** The block is `"abc"`, `0x06`, and `132`
zero bytes. The first byte of the 32-byte result is `0xd9`. The
correct SHA3-256 first byte is `0x3a`. The first difference is at
index `0`.

**Domain bits omitted.** The first pad byte is `0x01`, and the block
still ends in `0x80`. The first result byte is `0x4e`. The first
difference is at index `0`.

**Domain bits reversed.** The first pad byte is `0x05`. The first
result byte is `0xd1`. The first difference is at index `0`.

**Big-endian lanes.** The correct byte pad is parsed with `as big`.
The first result byte is `0x36`. The first difference is at index
`0`.

**Swapped lane order.** Let the correct Keccak-f[200] string be
`S[0]` through `S[24]`, with `S[5y + x]` the lane `(x, y)`. The
swapped string writes `S′[5x + y] = lane (x, y)`. Index `0` is lane
`(0, 0)` under both formulas, so `S′[0] = S[0] = 0x3c`. Index `1` of
the swapped string is `5·0 + 1`, lane `(0, 1)`. That lane's correct
index is `5·1 + 0 = 5`, and `S[5] = 0xb3`. The correct string's
index `1` is lane `(1, 0)`, and `S[1] = 0x28`. The first difference
is at index `1`, with left byte `0xb3` and right byte `0x28`.

**Rotation offset plus one.** Every ρ offset is the triangular
number plus `1`, and lane `(0, 0)` stays unrotated. The first byte
of Keccak-f[200] on the zero state is `0x9b`. The correct first byte
is `0x3c`. The first difference is at index `0`.

`orangec eval` prints those bytes.

```text
keccak::no_final_byte: Word[8] = 0xd9
keccak::omitted_byte: Word[8] = 0x4e
keccak::reversed_byte: Word[8] = 0xd1
keccak::big_byte: Word[8] = 0x36
keccak::correct_byte: Word[8] = 0x28
keccak::swapped_byte: Word[8] = 0xb3
keccak::bad_rho_byte: Word[8] = 0x9b
```

`orangec test` then runs all eleven tests. Five pass. Six fail.
Standard error is empty. Status is `1`. The report is the whole
stdout of that one run.

```text
test "rc and width" ... ok
test "Keccak-f[200] of the zero state" ... ok
test "Keccak-f[1600] lane (0, 0) of the zero state" ... ok
test "SHA3-256 of abc" ... ok
test "SHA3-256 of the empty message" ... ok
test "missing final pad bit" ... FAILED
    left:  [0xd9, 0x2d, 0x2c, 0x3d, 0xb0, 0xa0, 0xf0, 0x02, 0x44, 0x1f, 0xed, 0x98, 0x5b, 0x59, 0x3b, 0x5d, 0xc8, 0x54, 0xb2, 0x50, 0x72, 0x65, 0x2e, 0xb4, 0x95, 0xd0, 0x11, 0x3b, 0x36, 0x86, 0x1d, 0x4a]
    right: [0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32]
    first difference at [0]
test "domain bits omitted" ... FAILED
    left:  [0x4e, 0x03, 0x65, 0x7a, 0xea, 0x45, 0xa9, 0x4f, 0xc7, 0xd4, 0x7b, 0xa8, 0x26, 0xc8, 0xd6, 0x67, 0xc0, 0xd1, 0xe6, 0xe3, 0x3a, 0x64, 0xa0, 0x36, 0xec, 0x44, 0xf5, 0x8f, 0xa1, 0x2d, 0x6c, 0x45]
    right: [0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32]
    first difference at [0]
test "domain bits reversed" ... FAILED
    left:  [0xd1, 0x4d, 0x00, 0x78, 0x7a, 0x4b, 0x40, 0x4f, 0x58, 0x2c, 0x2c, 0x3d, 0x2c, 0x21, 0x3b, 0x3a, 0x98, 0xe1, 0x53, 0x7d, 0x49, 0x4e, 0x05, 0xb2, 0x92, 0xc1, 0xa9, 0x36, 0x82, 0x54, 0x2f, 0x63]
    right: [0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32]
    first difference at [0]
test "big-endian lanes" ... FAILED
    left:  [0x36, 0x0a, 0xbc, 0xb8, 0x4c, 0x4b, 0x4d, 0x75, 0x95, 0xbe, 0x8e, 0xa7, 0x36, 0x78, 0xee, 0xc6, 0x02, 0x35, 0x1c, 0xf2, 0x7f, 0x40, 0x79, 0xce, 0x7b, 0x29, 0x63, 0x1a, 0x5e, 0x57, 0x2b, 0x77]
    right: [0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32]
    first difference at [0]
test "swapped lane order" ... FAILED
    left:  [0x3c, 0xb3, 0xe9, 0xea, 0xd2, 0x28, 0x5c, 0xb8, 0xa3, 0xc5, 0x26, 0x17, 0x11, 0x85, 0xab, 0x84, 0x1e, 0x13, 0x2c, 0xaf, 0x1c, 0xaa, 0x4c, 0x69, 0xea]
    right: [0x3c, 0x28, 0x26, 0x84, 0x1c, 0xb3, 0x5c, 0x17, 0x1e, 0xaa, 0xe9, 0xb8, 0x11, 0x13, 0x4c, 0xea, 0xa3, 0x85, 0x2c, 0x69, 0xd2, 0xc5, 0xab, 0xaf, 0xea]
    first difference at [1]
test "rotation offset plus one" ... FAILED
    left:  [0x9b, 0x85, 0x59, 0x75, 0x62, 0x35, 0xdd, 0x3f, 0xff, 0xb2, 0xd8, 0x1b, 0xde, 0xba, 0x28, 0x9c, 0x5f, 0x1b, 0x52, 0x8d, 0xe0, 0xdf, 0xf7, 0xe1, 0x8b]
    right: [0x3c, 0x28, 0x26, 0x84, 0x1c, 0xb3, 0x5c, 0x17, 0x1e, 0xaa, 0xe9, 0xb8, 0x11, 0x13, 0x4c, 0xea, 0xa3, 0x85, 0x2c, 0x69, 0xd2, 0xc5, 0xab, 0xaf, 0xea]
    first difference at [0]
11 tests: 5 passed, 6 failed
```

The step counts for the same run:

```text
test "rc and width": 38097 steps
test "Keccak-f[200] of the zero state": 79461 steps
test "Keccak-f[1600] lane (0, 0) of the zero state": 101725 steps
test "SHA3-256 of abc": 102033 steps
test "SHA3-256 of the empty message": 102029 steps
test "missing final pad bit": 102028 steps
test "domain bits omitted": 102032 steps
test "domain bits reversed": 102032 steps
test "big-endian lanes": 102031 steps
test "swapped lane order": 79461 steps
test "rotation offset plus one": 63441 steps
total: 974370 of 1048576 steps
```

A failing comparison is the result the test was written to produce.
Repairing a failure by editing the expected digest would make the
test pass and would leave the transcription wrong. The repair is the
formula in the proposition: the index `5y + x`, the offset without
the extra `1`, the final `1` of `pad10*1`, the suffix `01` in that
bit order, and `as little` for the lanes.

### J7.11 What this slice does not denote

The step mappings are checked twice, once for each sheet in the
list. They are not checked once for every width in Table 1. `w = 1`,
`w = 2`, and `w = 4` are not words here. `w = 16` and `w = 32` are
words and are not instances. Pull request 248 is the open compiler
request that would check one word parameter at every width. It is
not in this tree, and this lesson does not schedule it.

The round loop is Keccak-f at those two widths. It is not
Keccak-p for a caller-chosen `nr`. It does not implement the
negative round indices of §3.4's `KECCAK-p[1600, 30]` example.

SHA3-224, SHA3-384, and SHA3-512 are the other three lines of §6.1.
They are not run. SHAKE128 and SHAKE256 are §6.2. They are J8.
HMAC is J14. A message whose bit length is not a multiple of 8 is
not run. A message that pads to two rate blocks is not run. The
200-byte image of Keccak-f[1600] on the zero state is not printed;
lane `(0, 0)` is printed, because the remaining step budget of this
module does not hold another full 1600-bit evaluation.

`as little` and `as big` are implemented on the S3u binary and are
specified by OEP-0017, which this lesson does not accept. A computed
rotation amount is specified by OEP-0021, which this lesson does not
accept. A type parameter over a listed set of types is specified by
OEP-0018, which this lesson does not accept. A `test` block is
specified by OEP-0020, which this lesson does not accept. An array
of arrays is specified by OEP-0025, which this lesson does not
accept. The nested form `W^5^5` is not that array syntax; Listing
J7.2 is the rejection, and the named sheet is the form the binary
checks.

The passing tests are five Bool values in Listing J7.7 and six in
Listing J7.6, on the inputs those tests write. They do not establish
a cryptographic security claim.
Do not call that Match verified.
The step counts are evaluator steps. The `974370` steps of the
Keccak module and the `61618` steps of the facts module are inside
the default budget `1048576`. A larger budget is not evidence about
speed.

The in-tree file `algorithms/sha3/sha3.or` is a `w = 64` reference
for several SHA-3 functions, including lengths this lesson does not
run. Listing J7.7 does not `use` it. The book listing is the
standard written in one module so the harness can feed it on
standard input. The two texts were compared by the vectors in
§J7.8 and §J7.9, not by treating either file as a proof of the
other.

### J7.12 Exercises

**Exercise J7.1.** From Table 1, at `b = 200` and at `b = 1600`,
state `w`, `l`, and `nr = 12 + 2l`. State the first and last round
index of Keccak-f in each case. Which proposition is that
substitution?

**Exercise J7.2.** State the ρ offset at `t = 0` and at `t = 23`.
Reduce `153` modulo `8` and `300` modulo `64`. Why does Listing J7.5
reject `<<< 8` while Listing J7.6 can rotate by a computed `153`?

**Exercise J7.3.** Compute the determinant of the π matrix modulo
`5`, and the product `3 · 2` modulo `5`. Why do those two integers
show that every lane is read once? What does the hit count in
Listing J7.6 add?

**Exercise J7.4.** Compute χ on the row `0` and on the row whose
only `1` is bit `0`. How many rows does the inverse check cover, and
why is that a proof for this row length rather than a sample?

**Exercise J7.5.** For SHA3-256 of “abc”, state `q`, the number of
zero bytes, `j`, and the padded bit length. Do the same `q` for the
empty message. Which byte is the suffix `01` together with the
pad's leading `1`, and which byte is the final `1`?

**Exercise J7.6.** Before looking at the report, state the first
byte of each of these four results on “abc”: the pad with no final
`1`, the pad whose first byte is `0x01`, the pad whose first byte is
`0x05`, and the correct pad parsed `as big`. What does a Match on
the real SHA3-256 digest not establish about those four?

**Exercise J7.7.** From the correct Keccak-f[200] string, derive
byte `1` of the swapped-index string and byte `1` of the correct
string. At which index is the first difference? Separately, state
the first byte when every ρ offset is increased by `1`.

**Exercise J7.8.** Why is the round loop `0..24` with a test
`ir < nr`, rather than a loop whose end is `nr`? What does pull
request 248 ask for that Listing J7.7 does not do? Where are SHAKE
and HMAC in this curriculum, and what does this lesson not build?

## Worked answers

**J7.1.** At `b = 200`, `w = 8`, `l = 3`, `nr = 18`. The index runs
from `0` through `17`. At `b = 1600`, `w = 64`, `l = 6`, `nr = 24`.
The index runs from `0` through `23`. Proposition J7.6.

**J7.2.** The offset at `t = 0` is `1`. The offset at `t = 23` is
`300`. `153 = 19 · 8 + 1`, so the residue is `1`.
`300 = 4 · 64 + 44`, so the residue is `44`. Listing J7.5 uses a
literal amount, and a literal on `Word[8]` has to lie in `0`
through `7`. The diagnostic is `ORC0216`. Listing J7.6 uses a
computed amount. On every byte, a computed `8` equals the input and
a computed `153` equals a rotation by `1`. The computed form is
OEP-0021, which this lesson does not accept. The binary's behavior
is the Bool the test prints.
A Match is not called verified.

**J7.3.** The determinant is `−3 ≡ 2 (mod 5)`. `3 · 2 = 6 ≡ 1 (mod 5)`,
so `2` is the inverse of `3` and the matrix is invertible. An
invertible map on a finite set is a bijection, so each of the
twenty-five lanes is read once. The hit count repeats that fact by
enumerating the twenty-five lanes. It does not extend the modulus.

**J7.4.** χ sends the zero row to `0`. It sends the row `1` to
`0x09`, because bits `0` and `3` are set and the other three bits of
the row are clear. The inverse check covers `32` rows. That is every
5-bit row, so the two-sided inverse is a proof for this row length.
Proposition J7.4.

**J7.5.** For “abc”, `q = 133`, the zero bytes number `131`,
`j = 1060`, and the padded length is `1088` bits. For the empty
message, `q = 136` and the zero bytes number `134`. The suffix and
the pad's leading `1` are the byte `0x06`. The final `1` is the byte
`0x80`. Proposition J7.7.

**J7.6.** The first bytes are `0xd9`, `0x4e`, `0xd1`, and `0x36`.
Each differs from `0x3a` at index `0`. A Match on SHA3-256 of “abc”
is one Bool on that byte string. It is not a Match on the four
neighbors, and it is not a security claim.

**J7.7.** Swapped byte `1` is lane `(0, 1)`, which is byte `5` of
the correct string, `0xb3`. Correct byte `1` is lane `(1, 0)`,
`0x28`. The first difference is at index `1`. The rotation that adds
`1` to every offset produces first byte `0x9b`.

**J7.8.** `nr` is a computed `Int`. A `for` bound in these listings
is a literal. `24` is the larger round count. Rounds with
`ir ≥ nr` return the state, so at `w = 8` the last six iterations
are the identity and at `w = 64` all twenty-four run. That is
Keccak-f for the two listed sheets. Pull request 248 asks for one
word parameter checked at every width, including a width the
programmer does not list as a separate type. Listing J7.7 lists two
sheets and is checked once per sheet. SHAKE is J8. HMAC is J14.
This lesson builds neither, and it does not build a general `nr`.

```text
j7-ledger
b-200 = 200
w-8 = 8
l-3 = 3
nr-8 = 18
ir-last-8 = 17
b-1600 = 1600
w-64 = 64
l-6 = 6
nr-64 = 24
ir-last-64 = 23
b-25 = 25
w-1 = 1
l-0 = 0
nr-1 = 12
offset-t0 = 1
offset-t23 = 300
offset-153 = 153
mod8-153 = 1
mod8-300 = 4
mod64-300 = 44
rho-det = 3
pi-det = 2
inv-3 = 2
three-times-two-mod5 = 1
chi-rows = 32
chi-of-1 = 9
abc-bits = 24
suffix-bits = 2
n-bits = 26
rate = 1088
rate-bytes = 136
pad-j = 1060
pad-bits = 1062
block-bits = 1088
q-abc = 133
q-zeros = 131
q-empty = 136
q-empty-zeros = 134
domain-byte = 6
final-byte = 128
omitted-pad-byte = 1
reversed-pad-byte = 5
rc-bit-j0 = 0
rc-bit-j1 = 1
rc-bit-j2 = 3
rc-bit-j3 = 7
rc-bit-j6 = 63
p30-start = -6
instances = 2
instance-cap = 256
swapped-index = 1
correct-byte-1 = 40
swapped-byte-1 = 179
no-final-byte = 217
omitted-byte = 78
reversed-byte = 209
big-byte = 54
bad-rho-byte = 155
facts-steps = 61618
keccak-steps = 974370
step-budget = 1048576
steps-left = 74206
f200-steps = 79461
f1600-lane-steps = 101725
```

## Sources and epigraph record

The quotation is the borrowed wording. The proofs are the lesson's.
Table 1, Table 2, and Table 6, and Algorithms 1 through 9, are the
standard's. The Keccak-f[200] zero-state bytes and the eighteen
8-bit round constants are the XKCP file's. The empty-message digest
is the NIST examples file's, and it is also what `hashlib` prints.
The “abc” digest is what `hashlib` prints. It is not a line of the
August 2015 PDF.

**[S18] National Institute of Standards and Technology.** FIPS PUB
202, *SHA-3 Standard: Permutation-Based Hash and Extendable-Output
Functions*, August 2015, DOI `10.6028/NIST.FIPS.202`. The epigraph
is the first sentence of the Abstract, on page iii of the PDF. The
hyphen in “SHA-3” is an ASCII hyphen. The sentence contains no inner
quotation marks. Wording was checked on 2026-10-17 against the file
whose SHA-256 is recorded in §J7.1. No translation is involved. The
publication names no personal author of the Abstract. The foreword
is signed by Charles H. Romine, Director, Information Technology
Laboratory; the epigraph is not taken from the foreword. The
announcement on page iv is a different sentence and is not the
epigraph. J6's record of FIPS 180-4 is [S17]. This record is [S18]
so that the epigraph tag is not a second definition of [S17].
Sections used for the derivation, and not as the epigraph, are
§2.1, §3.1, §3.1.2, §3.1.3, §3.2.1, §3.2.2, §3.2.3, §3.2.4, §3.2.5,
§3.3, §3.4, §5.1, §5.2, §6.1, and Appendix B.1 and B.2. §4 is named
and not transcribed as a general loop. §6.2 is named and not run.
This record's tag is [S18].

Source: <https://doi.org/10.6028/NIST.FIPS.202>

**[T14] Retrieved files.** On 2026-10-17 the lesson retrieved the
August 2015 PDF from
<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf>.
Its SHA-256 is
`1592607831ff0908cc590632ce371c6c95e94025bb1a0c8ae90a4d0ec1ed025e`.
The same day the lesson retrieved the XKCP file
`KeccakF-200-IntermediateValues.txt` from
<https://raw.githubusercontent.com/XKCP/XKCP/master/tests/TestVectors/KeccakF-200-IntermediateValues.txt>.
Its SHA-256 is
`5420859c01dfd1e0556389537b214a68ac14c68c06012827f796e4158d49acf5`.
The all-zero example ends with the 25 bytes quoted in §J7.8. The
file's later block is a second example and is not that result. The
eighteen `RC` bytes and the `RhoOffset` table at the top of the file
are the bytes §J7.4 and §J7.7 use. The same day the lesson retrieved
the NIST SHA3-256 example for the 0-bit message from
<https://csrc.nist.gov/csrc/media/projects/cryptographic-standards-and-guidelines/documents/examples/sha3-256_msg0.pdf>.
Its SHA-256 is
`72c5a5d37ed639058421b0a292775605a6be1631cfc4c156a2fb4739e68f8e9c`.
That file prints the hash quoted in §J7.9 and shows the absorbed
block beginning `06` and ending the rate with `80`. It also
retrieved the 5-bit example, SHA-256
`06a1166ce3a0a77ebb7a0d360160daecc879a1a556eb020ebfde1ad9404d0829`,
and the 30-bit example, SHA-256
`d56697dfd40924726a078a67956c342d35acd4747df46ff584dff44e7e6db1a5`.
Those two messages are not whole bytes, and Listing J7.7 does not
run them. The request for `sha3-256_msg24.pdf` in that directory
returned HTTP 404 and an empty body. The XKCP short-message file
`ShortMsgKAT_SHA3-256.txt`, SHA-256
`bc2506ecea3d2ffc2ddfb53982327688ea429a668a98fec571ede949c913f0d0`,
has `Len = 0` equal to the empty digest and `Len = 24` equal to the
message bytes `1F877C`, which are not the ASCII bytes of “abc”.
`hashlib.sha3_256` on this machine printed the empty digest and the
“abc” digest quoted in §J7.9. This record's tag is [T14].

Source: <https://raw.githubusercontent.com/XKCP/XKCP/master/tests/TestVectors/KeccakF-200-IntermediateValues.txt>

**[C8] Permutation surface.** The listings use `edition 2026`,
`module`, `spec`, `type`, `let`, `Word[8]`, `Word[64]`,
`Int`, `Bool`, `Mod[5]`, `^`, `&`, `|`, `~`, `+`, `*`, `/`,
`%`, `<<`, `<<<`, `>>`, `++`, `&&`, `if`, `hex"06"`, a string
literal, `[0; n]`, `for`, `with`, `as little`, `as big`, `as Int`,
a type parameter over two named sheet types, and `test`. `as little`
and `as big` are implemented on the S3u binary and are specified by
OEP-0017, which this lesson does not accept. A computed rotation
amount is specified by OEP-0021, which this lesson does not accept.
A type parameter is specified by OEP-0018, which this lesson does
not accept. A test is specified by OEP-0020, which this lesson does
not accept. A nested array is specified by OEP-0025, which this
lesson does not accept. Identifiers are ASCII. `Word[1]` is rejected
by `ORC0204`. `Word[w]` with `w` a size name is rejected by
`ORC0101`. A repeated `^` is rejected by `ORC0101`. A product of two
type parameters is checked, and the mismatched pair is `ORC0214`. A
literal `<<< 8` on `Word[8]` is rejected by `ORC0216`. A `for` bound
in these listings is a literal, and the end is exclusive. The
version line these commands print is `orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`.
The policy file in this tree is `policy/gate0-repository-policy.json`,
policy version 0.2.170. That version is not part of the compiler's
version line.
This record's tag is [C8].

The test form is the one N12 recorded. `orangec check` checks a test
and does not run it. `orangec eval --spec` runs one parameterless
spec and does not run tests. `orangec test` runs the tests. A failed
check prints the diagnostic and no report. A failed test prints
`left` and `right` and no diagnostic. `orangec test --stats` prints
the evaluator-step count of each test on standard error. Those
counts are the counts Assumption J7.8 names.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

J7 adds eight exercises with worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`. The Orange listings
are the seven fenced programs in this file.
`compiler/crates/orangec/tests/book_j7.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not prove
Assumption J7.3 from a smaller theory of the compiler, and they do
not accept OEP-0017, OEP-0018, OEP-0020, OEP-0021, or OEP-0025. A
passing test in that file is a Match of the `Bool` the listing writes.
Do not call that Match verified.
Reaching the last page is not a certificate.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-17 at the
owner's direction. Owner review is pending.
