# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J6: SHA-512 and the Rest of the SHA-2 Family. Draft 2026-10-17.

Continue from
[SHA-256 as FIPS 180-4 Writes It](JOURNEYMAN_J5_SHA256_AS_FIPS_180_4_WRITES_IT.md#j5-sha-256-as-fips-180-4-writes-it).
J5 derived SHA-256: the padding of §5.1.1, the schedule and one round of
§6.2.2, and the two known-answer digests. This lesson is the same
derivation at the width §6.4 uses, and then the reason the standard
does not define SHA-384 and SHA-512/t as prefixes of a SHA-512 digest.
[Standards as Versioned Inputs](JOURNEYMAN_J2_STANDARDS_AS_VERSIONED_INPUTS.md#j2-standards-as-versioned-inputs),
[The Corpus as Acceptance Test](JOURNEYMAN_J3_THE_CORPUS_AS_ACCEPTANCE_TEST.md#j3-the-corpus-as-acceptance-test),
and
[Byte Order and Format Boundaries](JOURNEYMAN_J4_BYTE_ORDER_AND_FORMAT_BOUNDARIES.md#j4-byte-order-and-format-boundaries)
are Block A. They are in this tree. This lesson cites their titles.
It cites Proposition J5.1 for the shape of a padding argument. It
does not repeat those proofs, and it does not rebuild J4's
little-endian load.

This lesson is **J6**. The locked label is J6. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. When
the text says “§6.4” or “§5.1.2”, the number is a section of the
edition named in the same sentence.

## J6: SHA-512 and the Rest of the SHA-2 Family

> “This Standard specifies secure hash algorithms - SHA-1, SHA-224, SHA-256, SHA-384, SHA-512, SHA-512/224 and SHA-512/256 - for computing a condensed representation of electronic data (message).”
>
> — National Institute of Standards and Technology, FIPS PUB 180-4, *Secure Hash Standard (SHS)* (August 2015), Announcement, item 3, first sentence. [S17]

The sentence is the first sentence of item 3 on page iv. It names
seven algorithms and says what they compute. It does not say that a
changed message produces a different digest. The announcement's later
sentences assert computational infeasibility and a very high
probability. This lesson does not adopt those sentences and does not
prove them. The standard's own security discussion is Appendix A.1,
which points at SP 800-107 and is not transcribed here. The object
on this page is the function §6.4 defines, and the initial values
§6.5, §6.6, and §6.7 substitute for it.

**The only-this-stack test.** You will derive the 1024-bit padding,
the 64-bit functions, one schedule step, and one compression round
from the August 2015 text, then run the transcription with `orangec
check`, `orangec eval`, and `orangec test` on the compiler this tree
builds. You will also feed the compiler programs it rejects, and
programs it accepts whose tests fail. The compiler's version line is
`orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`. Every
listing is written for that slice. Every fenced program is Current
on that slice. No listing is marked Proposed. A form this slice does
not implement is not given a syntax here. A program the compiler
rejects is still Current: the rejection is the result the listing
exists to show.

### J6.1 The sections, in the order the standard uses them

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can name the sections of FIPS PUB 180-4, August 2015, that
   define SHA-512, SHA-384, and SHA-512/t, in the order §6.4 uses
   them, and you can separate that edition from the examples file
   that prints the SHA-512 rows.
2. You can compute the padding of “abc” and of the 112-byte message,
   including `k`, the number of zero bytes after `0x80`, and the
   reason the next byte is `0x80`. You can say why 111 bytes is the
   last whole-byte message that still pads to one block.
3. You can derive `H(0)_0` and `K_0` as the low 64 bits of an integer
   square root and an integer cube root, and check four further
   words of the same recipe in Orange.
4. You can compute `W16` and `W17` of the “abc” schedule, then `T1`,
   `T2`, and the eight working variables after round 0, including
   each sum that crosses `2^64`.
5. You can transcribe §6.4.2 and the IV construction of §5.3.6 so
   that the names and the order are the standard's, match the
   known-answer digests this lesson pins, and regenerate the IVs
   §5.3.6.1 and §5.3.6.2 print.
6. You can predict what fails, and how, when `Σ1` is written with the
   SHA-256 distances, when a 64-byte block is given to `sha512`, when
   the 112-byte message is given a 64-bit length field, when the pad
   omits `0x80`, and when SHA-384 or SHA-512/256 is formed by
   truncating a SHA-512 digest. You can say what a passing digest
   does not establish.

The edition is FIPS PUB 180-4, *Secure Hash Standard (SHS)*, August
2015, DOI `10.6028/NIST.FIPS.180-4`. The file retrieved on
2026-10-17 has SHA-256
`0455b406d89648d20cbde375561e19c245b9815e894164c2670772e3d54deb82`.
That is the hash J5 recorded for the file retrieved on 2026-10-09.
It is the same August 2015 file, not a later edition. The erratum
page records one editorial change, the SHA-1 edit J5 already
described: in §4.1.1, `t < 79` became `t ≤ 79`. That line is not an
equation of SHA-512. No further erratum is applied here.

**Assumption J6.1 — The pin is the edition, not the title.**
“Secure Hash Standard” names more than one publication. The
algorithms, the section numbers, and the hex words below are the
August 2015 file. Lesson J2 is the lesson that makes a pin a
procedure. This lesson does not depend on that text. The pin is
stated in this assumption.

**Assumption J6.2 — A word is 64 bits, and `+` is the residue.**
§3.2 fixes `w = 64` for SHA-384, SHA-512, SHA-512/224, and
SHA-512/256. Addition of two words is the sum of the integers they
represent, reduced modulo `2^64`. The same sentence is repeated at
the start of §6.4.2: addition is performed modulo `2^64`.
`2^64 = 18446744073709551616`. For a sum of integers that may exceed
one multiple of that modulus, the residue is the unique remainder in
`0` through `2^64 − 1`. On this compiler, `+` of two `Word[64]`
values is that residue, and so is `*`. The hand arithmetic below is
the mathematical claim. A Match says the program agreed on the
inputs the test wrote.

**Assumption J6.3 — The leftmost bit is the most significant bit.**
§3.1, item 2, states the big-endian convention for 32-bit and 64-bit
words: within each word, the most significant bit is stored in the
leftmost bit position. The item that represents an integer as a pair
of words is the 64-bit case in §3.1, raised by one word width: an
integer `Z` with `0 ≤ Z < 2^128` is the pair `(x, y)` where
`Z = 2^64·x + y`. §5.2.2 says a SHA-512 message block is 1024 bits,
sixteen 64-bit words, and the first word is the leftmost. Lesson J4
is the lesson that treats byte order. The order used by the correct
listings below is `as big`. The first byte of a word is the leftmost
eight bits.

**Assumption J6.4 — The six functions are equations (4.8) through
(4.13).** §4.1.3 defines them on 64-bit words. The names in Orange
are ASCII, for the reason J5's Listing J5.4 already showed: a name
that begins with U+03C3 is rejected. The mapping is:

```text
Ch(x, y, z)        ch(x, y, z)
Maj(x, y, z)       maj(x, y, z)
Σ0{512}(x)         big_sigma0_512(x)
Σ1{512}(x)         big_sigma1_512(x)
σ0{512}(x)         small_sigma0_512(x)
σ1{512}(x)         small_sigma1_512(x)
```

`∧` is `&`. `⊕` is `^`. `¬` is `~`. `ROTR n` is `>>> n`. `SHR n` is
`>> n`. Parentheses follow the equations. Orange gives operators
from different groups no relative precedence; the parentheses are
the equation's, written where the compiler requires them. `ch` and
`maj` are the same equations at `w = 32` and at `w = 64`. One spec
takes either width. The 32-bit test in Listing J6.2 is equation
(4.2), which J5 already computed, kept here so the shared spec is
not a silent change of width.

**Assumption J6.5 — The examples file is not a section of the
standard.** Appendix A.2 of the August 2015 file says examples are
available and prints a URL. It does not print a digest. The SHA-512
block contents, the round row `t = 0`, the intermediate hash after
the first of the two blocks, and the two SHA-512 digests cited below
are the NIST SHA-512 examples file recorded in §J6.14. The file
retrieved on 2026-10-17 has SHA-256
`84ce867c99bad5a7b05abc74bb1dfc9de8486f49a160653cc8b15272cc532115`.
That file does not contain a row labelled `W[16]` or `W[17]`, and it
does not contain a SHA-384 digest. Lesson J3 is the lesson that
treats a finite set of pinned vectors as an acceptance test. This
lesson does not depend on that text. The acceptance set is the tests
in §J6.6, and the coverage sentence is stated there.

**Assumption J6.6 — A Match is one `Bool` on the inputs written in
that test.** Well-formed source is not a Match. A Match on these
inputs is not a Match on every input.
A Match is not called verified.
Nothing on this page is a security claim. A count of
evaluator steps is the number `orangec test --stats` prints for a
test. It is not a count of machine operations, and this lesson draws
no conclusion about speed from it.

**Assumption J6.7 — The hand calculation and the program denote the
same word only where both perform the same operations under J6.2
through J6.4.** Agreement on one input is one point. The domain of
one 64-bit word is `2^64` inputs. The domain of a message block is
`2^1024` inputs. The domain of a message whose length in bits is
below `2^128` is larger than either.

**Assumption J6.8 — The byte form of the padding is for a whole-byte
message.** §5.1.2 defines `k` for any bit length `ℓ`. The Orange
function in the listings takes a byte length. A message whose length
in bits is not a multiple of 8 is not built on this page. The
identity between a 64-bit length field and a 128-bit length field,
in §J6.9, is stated for the whole-byte messages the function can
name.

Figure 1 of the August 2015 file gives SHA-512 a message shorter
than `2^128` bits, a block of 1024 bits, a word of 64 bits, and a
digest of 512 bits. SHA-384, SHA-512/224, and SHA-512/256 keep that
block and that word. Their digests are 384, 224, and 256 bits.
Figure 1 prints those bounds as powers of two.

§6 opens by splitting every algorithm into preprocessing and hash
computation. For SHA-512 the split is §6.4.1 and §6.4.2.

§6.4.1 has two steps. Step 1 sets `H(0)` from §5.3.5. Step 2 pads
and parses the message as specified in §5. §5 itself has three
steps: padding (§5.1.2), parsing into `N` blocks of 1024 bits
(§5.2.2), and the initial hash value (§5.3.5). §6.4.2 then uses the
functions of §4.1.3 and the constants of §4.2.3. The working
variables are `a` through `h`. The temporary words are `T1` and
`T2`. The schedule words are `W0` through `W79`.

SHA-384, §6.5, is SHA-512 from the `H(0)` of §5.3.4, truncated to
the leftmost 384 bits. SHA-512/224, §6.6, and SHA-512/256, §6.7,
are SHA-512 from the `H(0)` of §5.3.6, truncated to `t` bits. SHA-224,
§6.3, is the 32-bit algorithm with a different `H(0)`. It is named
in §J6.12 and not run. HMAC is the construction the curriculum names
as J14. This lesson does not build a key, a key block, or an outer
hash.

### J6.2 Padding

§5.1.2. Let `ℓ` be the length of the message `M` in bits. Append a
single `1` bit, then `k` zero bits, where `k` is the smallest
non-negative solution of

```text
ℓ + 1 + k ≡ 896 (mod 1024)
```

Then append the 128-bit block that is `ℓ` in binary. The padded
length is a multiple of 1024.

**Proposition J6.1.** There is exactly one such `k` in `0` through
`1023`. It is the remainder when `895 − ℓ` is divided by `1024`,
taking the remainder in that range.

*Proof.* `ℓ + 1 + k ≡ 896 (mod 1024)` rearranges to
`k ≡ 895 − ℓ (mod 1024)`. The integers `0` through `1023` are one
complete set of residues modulo `1024`, so exactly one of them
satisfies the congruence. That integer is the smallest non-negative
solution, because every smaller non-negative integer is a different
residue. The padded length is `ℓ + 1 + k + 128`, and
`ℓ + 1 + k = 896 + 1024·q` for some integer `q`, so the padded
length is `1024·(q + 1)`.

**Proposition J6.2.** Let `ℓ = 8·len` for an integer `len ≥ 0`.
Then `k ≡ 7 (mod 8)`. The first padding byte is `0x80`. The number
of zero bytes after that byte is `(k − 7) / 8`. The number of
appended bytes is

```text
1 + (k − 7) / 8 + 16
```

*Proof.* `895 = 8·111 + 7`, and `8·len ≡ 0 (mod 8)`, so
`895 − ℓ ≡ 7 (mod 8)`. By Proposition J6.1, `k` is that residue in
`0` through `1023`, so `k = 8·z + 7` for an integer `z` in `0`
through `127`. The appended bits begin with a `1` followed by seven
`0` bits, which is the byte `0x80`, and then `k − 7 = 8·z` zero
bits, which are `z` zero bytes. The length field is 128 bits, which
is 16 bytes. The sum is the displayed formula.

**Proposition J6.3.** For every `len` in `0` through `112`, that
appended-byte count equals

```text
128 · ((len + 144) / 128) − len
```

where the division is integer division, rounding toward zero. This
is the expression `padding[len, 2]` uses.

*Proof.* By Proposition J6.2 the appended count `A` is at least 17
(when `z = 0`) and at most 144 (when `z = 127`). The padded byte
length `len + A` is a multiple of 128, because the padded bit length
is a multiple of 1024. It is the least multiple of 128 that is at
least `len + 17`: a shorter pad would be a smaller non-negative `k`,
and Proposition J6.1 says there is only one. The least multiple of
128 that is at least `len + 17` is

```text
128 · ((len + 17 + 127) / 128) = 128 · ((len + 144) / 128)
```

Subtract `len`. The same algebra at a block of `64·s` bytes and a
length field of `8·s` bytes replaces 128 by `64·s` and 144 by
`72·s`, which is the expression in the listing. This lesson calls
it at `s = 2`. The declaration also admits `s = 1` and `s = 3`,
because that is the range in `algorithms/sha2/sha2.or`. `s = 1` is
the SHA-256 block, used in §J6.8 only to show a length error. `s = 3`
is not a hash this lesson runs.

The range `len in 0..113` is `0` through `112`. Together with
`s in 1..3` that is `113 · 2 = 226` instances. A function on this
slice has at most 256 instances. Listing J6.19 is the declaration
that asks for 400.

**Proposition J6.4.** For “abc”, `ℓ = 24`, `k = 871`, the zero
bytes after `0x80` number 108, the pad is 125 bytes, and the padded
length is 1024 bits. The first 64-bit word is `0x6162638000000000`.
The last 64-bit word is `0x0000000000000018`.

*Proof.* `ℓ = 8·3 = 24`. Proposition J6.1 gives
`k = (895 − 24) mod 1024 = 871`. Proposition J6.2 gives
`(871 − 7) / 8 = 108` zero bytes, and `1 + 108 + 16 = 125` appended
bytes. `24 + 1 + 871 + 128 = 1024`. The bytes are `0x61`, `0x62`,
`0x63`, then `0x80`, then 108 zero bytes, then the 16-byte length.
The first eight of those are `0x6162638000000000` under Assumption
J6.3. The length is 24, which is `0x18` in the last byte of the
16-byte field, so the last word is `0x18`.

**Proposition J6.5.** For the 112-byte message in the examples file,
`ℓ = 896`, `k = 1023`, the zero bytes after `0x80` number 127, and
the padded length is 2048 bits. The byte `0x80` is in the first
block. The length is in the second block, and its last word is
`0x0000000000000380`. A whole-byte message of 111 bytes is the last
length that still pads to one block: `k = 7`, there are no zero
bytes after `0x80`, and the length starts on the next byte.

*Proof.* `ℓ = 8·112 = 896`.
`k = (895 − 896) mod 1024 = 1023`.
`(1023 − 7) / 8 = 127`. The appended count is `1 + 127 + 16 = 144`.
`112 + 144 = 256` bytes, which is 2048 bits, two blocks. The first
block holds 112 message bytes, then `0x80`, then 15 zero bytes.
That `0x80` is the first byte of the fifteenth word, so the word is
`0x8000000000000000`, and the sixteenth word of the first block is
zero. The second block is 112 zero bytes and then the length 896,
which is `0x380`, in the last word. For 111 bytes,
`ℓ = 888` and `k = (895 − 888) mod 1024 = 7`. Then `z = 0`, the pad
is 17 bytes, and `111 + 17 = 128`, one block. The next whole-byte
length is Proposition J6.5's 112.

The empty message is the same arithmetic at `len = 0`:
`k = 895`, `z = 111`, and the padding alone is the one block. Its
first word is `0x8000000000000000`. Listing J6.1 builds that block
and does not hash it. The empty digest is a separate test, in
§J6.6.

**Listing J6.1 — `pad.or`**

```orange
edition 2026;
module pad {
  spec padding[len in 0..113, s in 1..3]()
    -> Word[8]^(64 * s * ((len + 72 * s) / (64 * s)) - len) {
    let l: Int = 8 * len;
    ([0; (64 * s * ((len + 72 * s) / (64 * s)) - len - 8 * s)] with [0] = 0x80)
      ++ (l as big Word[8]^(8 * s))
  }

  spec abc_block() -> Word[8]^128 { "abc" ++ padding[3, 2]() }

  spec abc_words() -> Word[64]^16 { abc_block() as big Word[64]^16 }

  spec abc_w0() -> Word[64] {
    let w: Word[64]^16 = abc_words();
    w[0]
  }

  spec abc_w15() -> Word[64] {
    let w: Word[64]^16 = abc_words();
    w[15]
  }

  spec two_padded() -> Word[8]^256 {
    let m: Word[8]^112 =
      "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn" ++
      "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
    m ++ padding[112, 2]()
  }

  spec two_block1() -> Word[64]^16 {
    let bytes: Word[8]^256 = two_padded();
    bytes[0..128] as big Word[64]^16
  }

  spec two_block2() -> Word[64]^16 {
    let bytes: Word[8]^256 = two_padded();
    bytes[128..256] as big Word[64]^16
  }

  spec two_w14() -> Word[64] {
    let w: Word[64]^16 = two_block1();
    w[14]
  }

  spec two_length() -> Word[64] {
    let w: Word[64]^16 = two_block2();
    w[15]
  }

  spec empty_words() -> Word[64]^16 { padding[0, 2]() as big Word[64]^16 }

  spec hand_abc() -> Word[8]^125 {
    let bits: Int = 24;
    let length: Word[8]^8 = bits as big Word[8]^8;
    hex"80" ++ [0; 116] ++ length
  }

  test "FIPS 180-4 5.1.2 abc W0" { abc_w0() == 0x6162638000000000 }
  test "FIPS 180-4 5.1.2 abc length word" { abc_w15() == 0x0000000000000018 }
  test "FIPS 180-4 5.1.2 two-block W14" { two_w14() == 0x8000000000000000 }
  test "FIPS 180-4 5.1.2 two-block length word" { two_length() == 0x0000000000000380 }
  test "pad64len abc matches a 64-bit length in one block" {
    padding[3, 2]() == hand_abc()
  }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec abc_w0 -
./compiler/target/debug/orangec eval --spec abc_w15 -
./compiler/target/debug/orangec eval --spec two_w14 -
./compiler/target/debug/orangec eval --spec two_length -
./compiler/target/debug/orangec test -
```

The commands read the listing on standard input, which is why the
locus of a later diagnostic is `<stdin>`. Check is silent.

**Expected evaluation output:**

```text
pad::abc_w0: Word[64] = 0x6162638000000000
pad::abc_w15: Word[64] = 0x0000000000000018
pad::two_w14: Word[64] = 0x8000000000000000
pad::two_length: Word[64] = 0x0000000000000380
```


**Test report:**

```text
test "FIPS 180-4 5.1.2 abc W0" ... ok
test "FIPS 180-4 5.1.2 abc length word" ... ok
test "FIPS 180-4 5.1.2 two-block W14" ... ok
test "FIPS 180-4 5.1.2 two-block length word" ... ok
test "pad64len abc matches a 64-bit length in one block" ... ok
5 tests: 5 passed, 0 failed
```


The status of `test` is 0. Standard error is empty when `--stats`
is absent. The five tests are the words Propositions J6.4 and J6.5
name, and the comparison of `padding[3, 2]()` with a 64-bit length
placed in the same 125 bytes. That comparison passes because 24 is
below `2^64`, so the high eight bytes of the 128-bit field are
zero, and a field of eight bytes holding 24 occupies the same
place. §J6.9 is the length at which this stops being true.

### J6.3 The functions, and the two constants the recipe fixes

§4.1.3 writes `Σ0{512}` as `ROTR 28` xor `ROTR 34` xor `ROTR 39`,
and `Σ1{512}` as `ROTR 14` xor `ROTR 18` xor `ROTR 41`. `σ0{512}`
is `ROTR 1` xor `ROTR 8` xor `SHR 7`. `σ1{512}` is `ROTR 19` xor
`ROTR 61` xor `SHR 6`. `Ch` and `Maj` are equations (4.8) and (4.9),
the same Boolean combinations J5 used at 32 bits.

§3.2 defines `ROTR n(x) = (x >> n) ∨ (x << (w − n))`, with `w = 64`.
J5 showed, for one 32-bit rotation, that the two shifted pieces
share no bit, so `∨` and `⊕` denote the same word. The same
disjointness is the argument here. The complementary shift is by
`64 − n`, not by `32 − n`.

**Proposition J6.6.** `ROTR 28` of `H(0)_0 = 0x6a09e667f3bcc908`
is `0x3bcc9086a09e667f`.

*Proof.* Write the word `x`. The right shift by 28 is
`x >> 28 = 0x00000006a09e667f`. The low 28 bits of `x` are
`0x3bcc908`. Shifted left by `64 − 28 = 36`, they are
`0x3bcc908000000000`. Those bits occupy positions 36 through 63.
The right shift occupies positions 0 through 35. No position is set
in both, so or and exclusive or agree:

```text
0x00000006a09e667f ∨ 0x3bcc908000000000 = 0x3bcc9086a09e667f
```

Listing J6.2 checks this word, and checks `Σ0`, `Σ1`, `Ch`, and
`Maj` on the initial words §5.3.5 prints. The other two rotations
inside `Σ0` are not expanded by hand. They are inside the `Σ0`
test. Repeating Proposition J6.6 at distances 34 and 39 is the same
proof with a different `n`.

§5.3.5 takes `H(0)` from the first 64 bits of the fractional part of
the square roots of the first eight primes. §4.2.3 takes `K_t` from
the first 64 bits of the fractional part of the cube roots of the
first eighty primes. The fractional part of a real number `r ≥ 1`
is the low 64 bits of `floor(r · 2^64)` after the integer bit, or
bits, have been removed. For a prime whose root lies between 1 and
2, `floor(root · 2^64)` is `2^64` plus those 64 bits.

**Proposition J6.7.** Let `s = 2^64 + 0x6a09e667f3bcc908`. Then
`s^2 ≤ 2^129 < (s + 1)^2`, so `s = floor(sqrt(2^129))`, and the low
64 bits of `s` are `H(0)_0`.

*Proof.* `0x6a09e667f3bcc908 = 7640891576956012808`, so
`s = 26087635650665564424`. Then

```text
s^2 = 680564733841876926890271207202494451776
2^129 = 680564733841876926926749214863536422912
```

The gap `2^129 − s^2` is `36478007661041971136`. The next square
exceeds this square by `2·s + 1 = 52175271301331128849`. The gap is
smaller than that difference, so

```text
(s + 1)^2 − 2^129 = 52175271301331128849 − 36478007661041971136
                  = 15697263640289157713
```

which is positive. Therefore `s^2 ≤ 2^129 < (s + 1)^2`. The integer
square root of `2^129` is `s`. And `sqrt(2) · 2^64 = sqrt(2^129)`,
so `floor(sqrt(2) · 2^64) = s`. Since `1 < sqrt(2) < 2`, this `s`
is `2^64` plus a 64-bit word, and that word is `H(0)_0`.

**Proposition J6.8.** Let `c = 2^64 + 0x428a2f98d728ae22`. Then
`c^3 ≤ 2^193 < (c + 1)^3`, so `c = floor(cbrt(2^193))`, and the low
64 bits of `c` are `K_0`.

*Proof.* `0x428a2f98d728ae22 = 4794697086780616226`, so
`c = 23241441160490167842`. Then

```text
c^3 = 12554203470773361527280037446887974176707433720323481911688
2^193 = 12554203470773361527671578846415332832204710888928069025792
```

The gap `2^193 − c^3` is
`391541399527358655497277168604587114104`. The next cube exceeds
this cube by `3·c^2 + 3·c + 1 = 1620493761649579679220648177341463314419`.
The gap is smaller, so

```text
(c + 1)^3 − 2^193 = 1228952362122221023723371008736876200315
```

which is positive. Therefore `c^3 ≤ 2^193 < (c + 1)^3`. And
`cbrt(2) · 2^64 = cbrt(2^193)`, so the low 64 bits of `c` are `K_0`.

The same subtraction proves the other words. This lesson writes it
out for the first square root and the first cube root, and for the
first word of the SHA-384 initial value, which is the square root
of 23, the ninth prime. The words for 3 and 5 are checked against
the constants §5.3.5 and §4.2.3 print. They are not given a second
copy of the subtraction in prose. The ledger in §J6.13 holds the
integers a reader needs to repeat it.

**Proposition J6.9.** Let `s = floor(sqrt(23) · 2^64)`. Then
`s = 88467476731178360536`, and `s mod 2^64 = 0xcbbb9d5dc1059ed8`,
which is the first word §5.3.4 prints for SHA-384.

*Proof.* `floor(sqrt(23) · 2^64) = floor(sqrt(23 · 2^128))`. Let
`T = 23 · 2^128 = 7826494439181584659657615970930668863488`, and let
`s = 88467476731178360536`. The gap `T − s^2` is
`159966782817866656192`. The step to the next square is
`2·s + 1 = 176934953462356721073`, which is larger than the gap, so
`s^2 ≤ T < (s + 1)^2`. The low 64 bits are `0xcbbb9d5dc1059ed8`.
The low 32 bits of that word are `0xc1059ed8`, which is the first
word §5.3.2 prints for SHA-224. That is a fact about two printed
words. This lesson does not run SHA-224.

**Listing J6.2 — `functions.or`**

```orange
edition 2026;
module functions {
  spec ch[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (~x & z) }
  spec maj[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0_512(x: Word[64]) -> Word[64] { (x >>> 28) ^ (x >>> 34) ^ (x >>> 39) }
  spec big_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 14) ^ (x >>> 18) ^ (x >>> 41) }
  spec small_sigma0_512(x: Word[64]) -> Word[64] { (x >>> 1) ^ (x >>> 8) ^ (x >> 7) }
  spec small_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 19) ^ (x >>> 61) ^ (x >> 6) }

  spec ch32() -> Word[32] { ch(0x510e527f, 0x9b05688c, 0x1f83d9ab) }
  spec ch_round0() -> Word[64] {
    ch(0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b)
  }
  spec maj_round0() -> Word[64] {
    maj(0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b)
  }
  spec sigma0_a() -> Word[64] { big_sigma0_512(0x6a09e667f3bcc908) }
  spec sigma1_e() -> Word[64] { big_sigma1_512(0x510e527fade682d1) }
  spec rotr28() -> Word[64] { 0x6a09e667f3bcc908 >>> 28 }

  test "FIPS 180-4 eq 4.2, Ch on the SHA-256 words" { ch32() == 0x1f85c98c }
  test "FIPS 180-4 eq 4.8, Ch on H4 H5 H6" { ch_round0() == 0x1f85c98c7b273d3b }
  test "FIPS 180-4 eq 4.9, Maj on H0 H1 H2" { maj_round0() == 0x3a6fe667f69ce92b }
  test "FIPS 180-4 eq 4.10, Sigma0 on H0" { sigma0_a() == 0x08c4db56aac80c2a }
  test "FIPS 180-4 eq 4.11, Sigma1 on H4" { sigma1_e() == 0x9427e33bb5c9dbca }
  test "FIPS 180-4 3.2 ROTR 28 of H0" { rotr28() == 0x3bcc9086a09e667f }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec ch_round0 -
./compiler/target/debug/orangec eval --spec rotr28 -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
functions::ch_round0: Word[64] = 0x1f85c98c7b273d3b
functions::rotr28: Word[64] = 0x3bcc9086a09e667f
```


**Test report:**

```text
test "FIPS 180-4 eq 4.2, Ch on the SHA-256 words" ... ok
test "FIPS 180-4 eq 4.8, Ch on H4 H5 H6" ... ok
test "FIPS 180-4 eq 4.9, Maj on H0 H1 H2" ... ok
test "FIPS 180-4 eq 4.10, Sigma0 on H0" ... ok
test "FIPS 180-4 eq 4.11, Sigma1 on H4" ... ok
test "FIPS 180-4 3.2 ROTR 28 of H0" ... ok
6 tests: 6 passed, 0 failed
```


The status is 0. The first test is the 32-bit `Ch` J5 computed,
`0x1f85c98c`, on the shared spec. The other five are the 64-bit
words named above.

### J6.4 The schedule words this block determines by hand

§6.4.2, step 1. For `t` from 16 through 79,

```text
W_t = σ1(W_{t−2}) + W_{t−7} + σ0(W_{t−15}) + W_{t−16}
```

with addition modulo `2^64`. For “abc”, Proposition J6.4 gives
`W0 = 0x6162638000000000` and `W15 = 0x18`, and `W1` through `W14`
are zero.

**Proposition J6.10.** `W16 = 0x6162638000000000`.

*Proof.* At `t = 16` the four terms are `σ1(W14)`, `W9`,
`σ0(W1)`, and `W0`. The first three inputs are zero. `ROTR` of zero
is zero and `SHR` of zero is zero, so `σ1(0) = 0` and `σ0(0) = 0`.
The sum is `W0`.

**Proposition J6.11.** `W17 = 0x00030000000000c0`.

*Proof.* At `t = 17` the four terms are `σ1(W15)`, `W10`,
`σ0(W2)`, and `W1`. The last three inputs are zero, so the sum is
`σ1(0x18)`. Equation (4.13) is `ROTR 19` xor `ROTR 61` xor `SHR 6`.
`0x18 >> 6 = 0`. `ROTR 19(0x18) = 0x18 << 45 = 0x0003000000000000`,
because the right shift of a value below `2^19` is zero and the
complementary distance is `64 − 19 = 45`.
`ROTR 61(0x18) = 0x18 << 3 = 0x00000000000000c0`, because
`64 − 61 = 3`. The exclusive or is `0x00030000000000c0`. The
examples file does not print this word. The derivation is the
proposition. Listing J6.3 checks it.

**Listing J6.3 — `schedule.or`**

```orange
edition 2026;
module schedule {
  spec small_sigma0_512(x: Word[64]) -> Word[64] { (x >>> 1) ^ (x >>> 8) ^ (x >> 7) }
  spec small_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 19) ^ (x >>> 61) ^ (x >> 6) }

  spec abc_words() -> Word[64]^16 {
    let bits: Int = 24;
    let length: Word[8]^16 = bits as big Word[8]^16;
    let block: Word[8]^128 = "abc" ++ hex"80" ++ [0; 108] ++ length;
    block as big Word[64]^16
  }

  spec message_schedule(block: Word[64]^16) -> Word[64]^80 {
    for t in 16..80 with w: Word[64]^80 = block ++ [0; 64] {
      w with [t] =
        small_sigma1_512(w[t - 2]) + w[t - 7] + small_sigma0_512(w[t - 15]) + w[t - 16]
    }
  }

  spec w16() -> Word[64] {
    let w: Word[64]^80 = message_schedule(abc_words());
    w[16]
  }

  spec w17() -> Word[64] {
    let w: Word[64]^80 = message_schedule(abc_words());
    w[17]
  }

  test "FIPS 180-4 6.4.2 abc W16" { w16() == 0x6162638000000000 }
  test "FIPS 180-4 6.4.2 abc W17" { w17() == 0x00030000000000c0 }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec w16 -
./compiler/target/debug/orangec eval --spec w17 -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
schedule::w16: Word[64] = 0x6162638000000000
schedule::w17: Word[64] = 0x00030000000000c0
```


**Test report:**

```text
test "FIPS 180-4 6.4.2 abc W16" ... ok
test "FIPS 180-4 6.4.2 abc W17" ... ok
2 tests: 2 passed, 0 failed
```


The status is 0. Each of those two tests evaluates the schedule
through `W17`. The step count on `--stats` is 3245 for each, which
is an evaluator-step count under Assumption J6.6.

### J6.5 Round 0

§6.4.2, step 3, at `t = 0`. The working variables start as `H(0)`.
`T1` is `h + Σ1(e) + Ch(e, f, g) + K_0 + W_0`. `T2` is
`Σ0(a) + Maj(a, b, c)`. The new `a` is `T1 + T2`. The new `e` is
`d + T1`. The other variables shift forward by one.

**Proposition J6.12.** On the “abc” block, the integer inside `T1`,
before reduction, is `31379689096978360480`. It crosses `2^64`
once, and the residue is `0xb37b0cfa1b97e8a0`. The integer inside
`T2` is `4842708523825821013`, which does not cross `2^64`, and the
word is `0x4334c1bea164f555`. The new `a` does not cross `2^64`.
The new `e` crosses once.

*Proof.* Listing J6.2 gives
`Ch = 0x1f85c98c7b273d3b`,
`Maj = 0x3a6fe667f69ce92b`,
`Σ0(a) = 0x08c4db56aac80c2a`, and
`Σ1(e) = 0x9427e33bb5c9dbca`.
§5.3.5 gives `h = 0x5be0cd19137e2179` and
`a = 0x6a09e667f3bcc908`. Proposition J6.8 gives `K_0`.
Proposition J6.4 gives `W0`. The five-term integer sum is

```text
6620516959819538809
+ 10675751287297530826
+ 2271443192277056827
+ 4794697086780616226
+ 7017280570803617792
= 31379689096978360480
```

`2^64 = 18446744073709551616`. Subtract once:
`31379689096978360480 − 18446744073709551616 = 12932945023268808864 = 0xb37b0cfa1b97e8a0`.
A second subtraction would pass zero, so the sum crosses once.
`T2` is `631871013017619498 + 4210837510808201515 = 4842708523825821013`,
which is less than `2^64`. The new `a` is the sum of the two
residues, `17775653547094629877 = 0xf6afceb8bcfcddf5`, also less
than `2^64`. The new `e` is `d + T1` with
`d = 0xa54ff53a5f1d36f1`:

```text
11912009170470909681 + 12932945023268808864 = 24844954193739718545
```

That equals `2^64 + 0x58cb02347ab51f91`, so it crosses once, and the
residue is `0x58cb02347ab51f91`. The examples file's row `t = 0`
prints `A = F6AFCEB8BCFCDDF5` and `E = 58CB02347AB51F91`. The other
six working variables at `t = 0` are the old `a` through `g`, which
are `H(0)_0` through `H(0)_6`.

**Listing J6.4 — `round0.or`**

```orange
edition 2026;
module round0 {
  spec ch[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (~x & z) }
  spec maj[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (x & z) ^ (y & z) }
  spec big_sigma0_512(x: Word[64]) -> Word[64] { (x >>> 28) ^ (x >>> 34) ^ (x >>> 39) }
  spec big_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 14) ^ (x >>> 18) ^ (x >>> 41) }

  spec t1() -> Word[64] {
    let h: Word[64] = 0x5be0cd19137e2179;
    let e: Word[64] = 0x510e527fade682d1;
    let f: Word[64] = 0x9b05688c2b3e6c1f;
    let g: Word[64] = 0x1f83d9abfb41bd6b;
    h + big_sigma1_512(e) + ch(e, f, g) + 0x428a2f98d728ae22 + 0x6162638000000000
  }

  spec t2() -> Word[64] {
    let a: Word[64] = 0x6a09e667f3bcc908;
    let b: Word[64] = 0xbb67ae8584caa73b;
    let c: Word[64] = 0x3c6ef372fe94f82b;
    big_sigma0_512(a) + maj(a, b, c)
  }

  spec after() -> Word[64]^8 {
    let t1: Word[64] = t1();
    let t2: Word[64] = t2();
    let a: Word[64] = 0x6a09e667f3bcc908;
    let b: Word[64] = 0xbb67ae8584caa73b;
    let c: Word[64] = 0x3c6ef372fe94f82b;
    let d: Word[64] = 0xa54ff53a5f1d36f1;
    let e: Word[64] = 0x510e527fade682d1;
    let f: Word[64] = 0x9b05688c2b3e6c1f;
    let g: Word[64] = 0x1f83d9abfb41bd6b;
    [t1 + t2, a, b, c, d + t1, e, f, g]
  }

  test "FIPS 180-4 6.4.2 abc T1" { t1() == 0xb37b0cfa1b97e8a0 }
  test "FIPS 180-4 6.4.2 abc T2" { t2() == 0x4334c1bea164f555 }
  test "examples file abc, t = 0" {
    after() == [
      0xf6afceb8bcfcddf5, 0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b,
      0x58cb02347ab51f91, 0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b,
    ]
  }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec t1 -
./compiler/target/debug/orangec eval --spec t2 -
./compiler/target/debug/orangec eval --spec after -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
round0::t1: Word[64] = 0xb37b0cfa1b97e8a0
round0::t2: Word[64] = 0x4334c1bea164f555
round0::after: Word[64]^8 = [0xf6afceb8bcfcddf5, 0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0x58cb02347ab51f91, 0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b]
```


**Test report:**

```text
test "FIPS 180-4 6.4.2 abc T1" ... ok
test "FIPS 180-4 6.4.2 abc T2" ... ok
test "examples file abc, t = 0" ... ok
3 tests: 3 passed, 0 failed
```


The status is 0. The eight words are the row the examples file
prints at `t = 0`, in the order `a` through `h`.

### J6.6 The transcription, the corpus, and the two generated IVs

§6.4.2, steps 1 through 4, is `schedule_512`, then 80 rounds, then
the wordwise sum of the incoming chaining value. §6.4.2's loop over
blocks is `hash_512`. The digest is those eight words as 64 bytes,
big-endian, which is `sha512`. §6.5 is the same compression from
§5.3.4, then the leftmost six words, which is `sha384`. The constants
are the eighty words §4.2.3 prints. This lesson copies them. It
derives the first, in Proposition J6.8, and checks the next two.

**Proposition J6.13.** SHA-384 and SHA-512/t are not defined as a
truncation of a SHA-512 digest. For a message that pads to one
block `B`, the chaining value those algorithms produce is
`C(H(0), B)`, where `C` is the compression of §6.4.2 and `H(0)` is
the initial value the algorithm names. SHA-512 names §5.3.5.
SHA-384 names §5.3.4. SHA-512/256 names §5.3.6.2. Those three words
of eight are not equal. Truncating the SHA-512 digest keeps the
§5.3.5 value and then drops bytes.

*Proof.* §6.5 says SHA-384 is the SHA-512 computation with the
initial value of §5.3.4, and the digest is the leftmost 384 bits of
the result. §6.7 says SHA-512/256 is the SHA-512 computation with
the initial value of §5.3.6.2, and the digest is the leftmost 256
bits. §6.4.2 sets `H(i)` from `H(i−1)` by one compression. For one
block, `H(1) = C(H(0), B)`. The procedures therefore differ at the
input of the first compression whenever their `H(0)` differs.
§5.3.4's first word is `0xcbbb9d5dc1059ed8`. §5.3.5's first word is
`0x6a09e667f3bcc908`. Those are not equal, so the inputs differ.
This argument does not say that `C` is injective in its first
argument. A function may send two different first arguments to the
same output. The specification does not claim that, and this lesson
does not prove it. What the specification does say is which value
is fed in. What one block shows is in Listing J6.11: for the “abc”
block, the first byte of the SHA-512 digest is `0xdd`, the first
byte of the SHA-384 digest is `0xcb`, and the first byte of the
SHA-512/256 digest is `0x53`. For this block, a different `H(0)`
produced a different digest. Keeping `H(0)` from §5.3.5 and dropping
bytes does not produce those digests.

§5.3.6 builds the SHA-512/t initial value in two steps. First, each
word of the SHA-512 `H(0)` is xored with `0xa5a5a5a5a5a5a5a5`.
Second, that modified value is the initial hash of one SHA-512
computation, whose message is the ASCII string `SHA-512/t` for the
decimal numeral `t`. The result is `H(0)` for SHA-512/t. For
`t = 224` and `t = 256` the string has 11 bytes either way, so both
messages use the padding of an 11-byte message, `padding[11, 2]`.
A general `t` can have a different number of digits. A different
length is a different padded block. This lesson does not run a
general `t`.

The words §5.3.6.1 and §5.3.6.2 print are the expected values of
those two computations. Listing J6.5 regenerates them. It does not
store a second copy of the IV as the thing being hashed. The
SHA-512/256 digest of “abc” is then the hash of the padded “abc”
block from the regenerated IV, truncated to 32 bytes. Those 32
bytes are the bytes `algorithms/sha2/sha2.or` demands in the test
it names `OpenSSL evpmd_sha.txt: SHA-512/256 of abc`. This lesson's
drafting also compared them with Python's `hashlib`, name
`sha512_256`, on the message `abc`. The examples file of §J6.5 does
not print them.

The SHA-512 digests of “abc” and of the 112-byte message are the
examples file. The intermediate eight words after the first block
of the 112-byte message are the row that file prints as the sum
`H(0) +` the working variables of that block. The empty SHA-512
digest, and both SHA-384 digests, are the bytes `sha2.or` demands
under the names `hashlib: SHA-512 of the empty message`,
`Botan sha2_64.vec: SHA-384 of abc`, and
`Botan sha2_64.vec: SHA-384 of the two-block message`. The same
drafting compared those four with `hashlib`. They are not rows of
the SHA-512 examples file.

A passing test on this set does not establish a message whose bit
length is not a multiple of 8. It does not establish SHA-224. It
does not establish SHA-512/224's digest of a message: the IV is
checked, and a message under that IV is not hashed here. It does
not establish the million-byte message of §J6.12. It does not
establish every 1024-bit block. It does not establish Appendix A.1.
Do not call that Match verified.

**Listing J6.5 — `sha512.or`**

```orange
edition 2026;
module sha512 {
  spec ch[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (~x & z) }
  spec maj[W in {Word[32], Word[64]}](x: W, y: W, z: W) -> W { (x & y) ^ (x & z) ^ (y & z) }

  spec big_sigma0_512(x: Word[64]) -> Word[64] { (x >>> 28) ^ (x >>> 34) ^ (x >>> 39) }
  spec big_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 14) ^ (x >>> 18) ^ (x >>> 41) }
  spec small_sigma0_512(x: Word[64]) -> Word[64] { (x >>> 1) ^ (x >>> 8) ^ (x >> 7) }
  spec small_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 19) ^ (x >>> 61) ^ (x >> 6) }

  spec round_constants_512() -> Word[64]^80 {
    (
      hex"428a2f98d728ae22 7137449123ef65cd b5c0fbcfec4d3b2f e9b5dba58189dbbc" ++
      hex"3956c25bf348b538 59f111f1b605d019 923f82a4af194f9b ab1c5ed5da6d8118" ++
      hex"d807aa98a3030242 12835b0145706fbe 243185be4ee4b28c 550c7dc3d5ffb4e2" ++
      hex"72be5d74f27b896f 80deb1fe3b1696b1 9bdc06a725c71235 c19bf174cf692694" ++
      hex"e49b69c19ef14ad2 efbe4786384f25e3 0fc19dc68b8cd5b5 240ca1cc77ac9c65" ++
      hex"2de92c6f592b0275 4a7484aa6ea6e483 5cb0a9dcbd41fbd4 76f988da831153b5" ++
      hex"983e5152ee66dfab a831c66d2db43210 b00327c898fb213f bf597fc7beef0ee4" ++
      hex"c6e00bf33da88fc2 d5a79147930aa725 06ca6351e003826f 142929670a0e6e70" ++
      hex"27b70a8546d22ffc 2e1b21385c26c926 4d2c6dfc5ac42aed 53380d139d95b3df" ++
      hex"650a73548baf63de 766a0abb3c77b2a8 81c2c92e47edaee6 92722c851482353b" ++
      hex"a2bfe8a14cf10364 a81a664bbc423001 c24b8b70d0f89791 c76c51a30654be30" ++
      hex"d192e819d6ef5218 d69906245565a910 f40e35855771202a 106aa07032bbd1b8" ++
      hex"19a4c116b8d2d0c8 1e376c085141ab53 2748774cdf8eeb99 34b0bcb5e19b48a8" ++
      hex"391c0cb3c5c95a63 4ed8aa4ae3418acb 5b9cca4f7763e373 682e6ff3d6b2b8a3" ++
      hex"748f82ee5defb2fc 78a5636f43172f60 84c87814a1f0ab72 8cc702081a6439ec" ++
      hex"90befffa23631e28 a4506cebde82bde9 bef9a3f7b2c67915 c67178f2e372532b" ++
      hex"ca273eceea26619c d186b8c721c0c207 eada7dd6cde0eb1e f57d4f7fee6ed178" ++
      hex"06f067aa72176fba 0a637dc5a2c898a6 113f9804bef90dae 1b710b35131c471b" ++
      hex"28db77f523047d84 32caab7b40c72493 3c9ebe0a15c9bebc 431d67c49c100d4c" ++
      hex"4cc5d4becb3e42b6 597f299cfc657e2a 5fcb6fab3ad6faec 6c44198c4a475817"
    ) as big Word[64]^80
  }

  spec initial_hash_384() -> Word[64]^8 {
    [
      0xcbbb9d5dc1059ed8, 0x629a292a367cd507, 0x9159015a3070dd17, 0x152fecd8f70e5939,
      0x67332667ffc00b31, 0x8eb44a8768581511, 0xdb0c2e0d64f98fa7, 0x47b5481dbefa4fa4,
    ]
  }

  spec initial_hash_512() -> Word[64]^8 {
    [
      0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
      0x510e527fade682d1, 0x9b05688c2b3e6c1f, 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
    ]
  }

  spec padding[len in 0..113, s in 1..3]()
    -> Word[8]^(64 * s * ((len + 72 * s) / (64 * s)) - len) {
    let l: Int = 8 * len;
    ([0; (64 * s * ((len + 72 * s) / (64 * s)) - len - 8 * s)] with [0] = 0x80)
      ++ (l as big Word[8]^(8 * s))
  }

  spec schedule_512(m: Word[64]^16) -> Word[64]^80 {
    for t in 16..80 with w: Word[64]^80 = m ++ [0; 64] {
      w with [t] =
        small_sigma1_512(w[t - 2]) + w[t - 7] + small_sigma0_512(w[t - 15]) + w[t - 16]
    }
  }

  spec round_512(v: Word[64]^8, k: Word[64], w: Word[64]) -> Word[64]^8 {
    let t1: Word[64] = v[7] + big_sigma1_512(v[4]) + ch(v[4], v[5], v[6]) + k + w;
    let t2: Word[64] = big_sigma0_512(v[0]) + maj(v[0], v[1], v[2]);
    [t1 + t2, v[0], v[1], v[2], v[3] + t1, v[4], v[5], v[6]]
  }

  spec compress_512(h: Word[64]^8, m: Word[64]^16) -> Word[64]^8 {
    let w: Word[64]^80 = schedule_512(m);
    let k: Word[64]^80 = round_constants_512();
    let v: Word[64]^8 = for t in 0..80 with v: Word[64]^8 = h { round_512(v, k[t], w[t]) };
    for i in 0..8 with out: Word[64]^8 = v { out with [i] = out[i] + h[i] }
  }

  spec hash_512[n in 1..3](h0: Word[64]^8, p: Word[8]^(128 * n)) -> Word[64]^8 {
    for i in 0..(n) with h: Word[64]^8 = h0 {
      compress_512(h, p[128 * i..128 * i + 128] as big Word[64]^16)
    }
  }

  spec sha512[n in 1..3](p: Word[8]^(128 * n)) -> Word[8]^64 {
    hash_512(initial_hash_512(), p) as big Word[8]^64
  }

  spec sha384[n in 1..3](p: Word[8]^(128 * n)) -> Word[8]^48 {
    hash_512(initial_hash_384(), p)[..6] as big Word[8]^48
  }

  spec masked_iv() -> Word[64]^8 {
    let h: Word[64]^8 = initial_hash_512();
    for i in 0..8 with out: Word[64]^8 = h {
      out with [i] = out[i] ^ 0xa5a5a5a5a5a5a5a5
    }
  }

  spec sha512_t_iv_256() -> Word[64]^8 {
    hash_512(masked_iv(), "SHA-512/256" ++ padding[11, 2]())
  }

  spec sha512_t_iv_224() -> Word[64]^8 {
    hash_512(masked_iv(), "SHA-512/224" ++ padding[11, 2]())
  }
  spec two_message() -> Word[8]^112 {
    "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn" ++
      "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu"
  }

  spec abc_digest() -> Word[8]^64 { sha512("abc" ++ padding[3, 2]()) }

  spec empty_digest() -> Word[8]^64 { sha512(padding[0, 2]()) }

  spec two_h1() -> Word[64]^8 {
    let padded: Word[8]^256 = two_message() ++ padding[112, 2]();
    compress_512(initial_hash_512(), padded[0..128] as big Word[64]^16)
  }

  spec two_digest() -> Word[8]^64 {
    let padded: Word[8]^256 = two_message() ++ padding[112, 2]();
    sha512(padded)
  }

  spec sha384_abc() -> Word[8]^48 { sha384("abc" ++ padding[3, 2]()) }

  spec sha384_two() -> Word[8]^48 {
    let padded: Word[8]^256 = two_message() ++ padding[112, 2]();
    sha384(padded)
  }

  spec sha512_256_abc() -> Word[8]^32 {
    hash_512(sha512_t_iv_256(), "abc" ++ padding[3, 2]())[..4] as big Word[8]^32
  }

  spec h0() -> Word[64] {
    let h: Word[64]^8 = initial_hash_512();
    h[0]
  }

  spec k0() -> Word[64] {
    let k: Word[64]^80 = round_constants_512();
    k[0]
  }

  spec h1_sqrt3() -> Word[64] {
    let h: Word[64]^8 = initial_hash_512();
    h[1]
  }

  spec h2_sqrt5() -> Word[64] {
    let h: Word[64]^8 = initial_hash_512();
    h[2]
  }

  spec k1_cbrt3() -> Word[64] {
    let k: Word[64]^80 = round_constants_512();
    k[1]
  }

  spec k2_cbrt5() -> Word[64] {
    let k: Word[64]^80 = round_constants_512();
    k[2]
  }

  spec h384_0() -> Word[64] {
    let h: Word[64]^8 = initial_hash_384();
    h[0]
  }

  test "FIPS 180-4 5.3.5 and 4.2.3 first words" {
    (h0() == 0x6a09e667f3bcc908) && (k0() == 0x428a2f98d728ae22)
  }
  test "square root of 3 and of 5, cube root of 3 and of 5" {
    (h1_sqrt3() == 0xbb67ae8584caa73b) && (h2_sqrt5() == 0x3c6ef372fe94f82b)
      && (k1_cbrt3() == 0x7137449123ef65cd) && (k2_cbrt5() == 0xb5c0fbcfec4d3b2f)
  }
  test "FIPS 180-4 5.3.4 first word, square root of 23" {
    h384_0() == 0xcbbb9d5dc1059ed8
  }
  test "sha512_empty_len_bits_0" {
    empty_digest() == (
      hex"cf83e1357eefb8bd f1542850d66d8007 d620e4050b5715dc 83f4a921d36ce9ce" ++
      hex"47d0d13c5d85f2b0 ff8318d2877eec2f 63b931bd47417a81 a538327af927da3e"
    )
  }
  test "sha512_abc_len_bits_24" {
    abc_digest() == (
      hex"ddaf35a193617aba cc417349ae204131 12e6fa4e89a97ea2 0a9eeee64b55d39a" ++
      hex"2192992a274fc1a8 36ba3c23a3feebbd 454d4423643ce80e 2a9ac94fa54ca49f"
    )
  }
  test "sha512_two_block_h1_len_bits_896" {
    two_h1() == [
      0x4319017a2b706e69, 0xcd4b05938bae5e89, 0x0186bf199f30aa95, 0x6ef8b71d2f810585,
      0xd787d6764b20bda2, 0xa260144709736920, 0x00ec057f37d14b8e, 0x06add5b50e671c72,
    ]
  }
  test "sha512_two_block_len_bits_896" {
    two_digest() == (
      hex"8e959b75dae313da 8cf4f72814fc143f 8f7779c6eb9f7fa1 7299aeadb6889018" ++
      hex"501d289e4900f7e4 331b99dec4b5433a c7d329eeb6dd2654 5e96e55b874be909"
    )
  }
  test "sha384_abc_len_bits_24" {
    sha384_abc() == (
      hex"cb00753f45a35e8b b5a03d699ac65007 272c32ab0eded163" ++
      hex"1a8b605a43ff5bed 8086072ba1e7cc23 58baeca134c825a7"
    )
  }
  test "sha384_two_block_len_bits_896" {
    sha384_two() == (
      hex"09330c33f71147e8 3d192fc782cd1b47 53111b173b3b05d2" ++
      hex"2fa08086e3b0f712 fcc7c71a557e2db9 66c3e9fa91746039"
    )
  }
  test "sha512_t_iv_256_section_5_3_6_2" {
    sha512_t_iv_256() == [
      0x22312194fc2bf72c, 0x9f555fa3c84c64c2, 0x2393b86b6f53b151, 0x963877195940eabd,
      0x96283ee2a88effe3, 0xbe5e1e2553863992, 0x2b0199fc2c85b8aa, 0x0eb72ddc81c52ca2,
    ]
  }
  test "sha512_t_iv_224_section_5_3_6_1" {
    sha512_t_iv_224() == [
      0x8c3d37c819544da2, 0x73e1996689dcd4d6, 0x1dfab7ae32ff9c82, 0x679dd514582f9fcf,
      0x0f6d2b697bd44da8, 0x77e36f7304c48942, 0x3f9d85a86a1d36c8, 0x1112e6ad91d692a1,
    ]
  }
  test "sha512_256_abc_from_generated_iv" {
    sha512_256_abc() ==
      hex"53048e2681941ef9 9b2e29b76b4c7dab e4c2d0c634fc6d46 e0e2f13107e7af23"
  }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec h0 -
./compiler/target/debug/orangec eval --spec empty_digest -
./compiler/target/debug/orangec eval --spec abc_digest -
./compiler/target/debug/orangec eval --spec two_h1 -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
sha512::h0: Word[64] = 0x6a09e667f3bcc908
sha512::empty_digest: Word[8]^64 = [0xcf, 0x83, 0xe1, 0x35, 0x7e, 0xef, 0xb8, 0xbd, 0xf1, 0x54, 0x28, 0x50, 0xd6, 0x6d, 0x80, 0x07, 0xd6, 0x20, 0xe4, 0x05, 0x0b, 0x57, 0x15, 0xdc, 0x83, 0xf4, 0xa9, 0x21, 0xd3, 0x6c, 0xe9, 0xce, 0x47, 0xd0, 0xd1, 0x3c, 0x5d, 0x85, 0xf2, 0xb0, 0xff, 0x83, 0x18, 0xd2, 0x87, 0x7e, 0xec, 0x2f, 0x63, 0xb9, 0x31, 0xbd, 0x47, 0x41, 0x7a, 0x81, 0xa5, 0x38, 0x32, 0x7a, 0xf9, 0x27, 0xda, 0x3e]
sha512::abc_digest: Word[8]^64 = [0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a, 0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd, 0x45, 0x4d, 0x44, 0x23, 0x64, 0x3c, 0xe8, 0x0e, 0x2a, 0x9a, 0xc9, 0x4f, 0xa5, 0x4c, 0xa4, 0x9f]
sha512::two_h1: Word[64]^8 = [0x4319017a2b706e69, 0xcd4b05938bae5e89, 0x0186bf199f30aa95, 0x6ef8b71d2f810585, 0xd787d6764b20bda2, 0xa260144709736920, 0x00ec057f37d14b8e, 0x06add5b50e671c72]
```


**Test report:**

```text
test "FIPS 180-4 5.3.5 and 4.2.3 first words" ... ok
test "square root of 3 and of 5, cube root of 3 and of 5" ... ok
test "FIPS 180-4 5.3.4 first word, square root of 23" ... ok
test "sha512_empty_len_bits_0" ... ok
test "sha512_abc_len_bits_24" ... ok
test "sha512_two_block_h1_len_bits_896" ... ok
test "sha512_two_block_len_bits_896" ... ok
test "sha384_abc_len_bits_24" ... ok
test "sha384_two_block_len_bits_896" ... ok
test "sha512_t_iv_256_section_5_3_6_2" ... ok
test "sha512_t_iv_224_section_5_3_6_1" ... ok
test "sha512_256_abc_from_generated_iv" ... ok
12 tests: 12 passed, 0 failed
```


The status is 0. The stderr of `orangec test --stats -` on this
listing, which is an evaluator-step count under Assumption J6.6, is:

```text
test "FIPS 180-4 5.3.5 and 4.2.3 first words": 238 steps
test "square root of 3 and of 5, cube root of 3 and of 5": 477 steps
test "FIPS 180-4 5.3.4 first word, square root of 23": 22 steps
test "sha512_empty_len_bits_0": 11592 steps
test "sha512_abc_len_bits_24": 11596 steps
test "sha512_two_block_h1_len_bits_896": 11594 steps
test "sha512_two_block_len_bits_896": 23147 steps
test "sha384_abc_len_bits_24": 11597 steps
test "sha384_two_block_len_bits_896": 23148 steps
test "sha512_t_iv_256_section_5_3_6_2": 11674 steps
test "sha512_t_iv_224_section_5_3_6_1": 11674 steps
test "sha512_256_abc_from_generated_iv": 23231 steps
total: 139990 of 1048576 steps
```


These counts are not the counts of `algorithms/sha2/sha2.or`. The
repository file's tests are different expressions, and they share
their file with the SHA-256 tests. The repository README states a
per-block figure for `hash_512` and for `compress_512`. This lesson
did not isolate those two calls, so it does not repeat the figures.
The thirteen tests of that file, run as
`orangec test --stats algorithms/sha2/sha2.or`, printed:

```text
test "Botan sha2_32.vec: SHA-256 of the empty message": 8973 steps
test "NIST example: SHA-256 of abc": 8976 steps
test "NIST example: SHA-256 of the two-block message": 17914 steps
test "Botan sha2_32.vec: SHA-256 of a 64-byte message": 17915 steps
test "OpenSSL evpmd_sha.txt: SHA-224 of abc": 8979 steps
test "OpenSSL evpmd_sha.txt: SHA-224 of the two-block message": 17917 steps
test "hashlib: SHA-512 of the empty message": 11591 steps
test "NIST example: SHA-512 of abc": 11595 steps
test "NIST example: SHA-512 of the two-block message": 23145 steps
test "Botan sha2_64.vec: SHA-384 of abc": 11596 steps
test "Botan sha2_64.vec: SHA-384 of the two-block message": 23146 steps
test "OpenSSL evpmd_sha.txt: SHA-512/256 of abc": 11592 steps
test "OpenSSL evpmd_sha.txt: SHA-512/256 of the two-block message": 23142 steps
total: 196481 of 1048576 steps
```


The one-block SHA-512 test in that file uses 11595 steps. The
two-block SHA-512 test uses 23145. The difference is 11550, which
is the evaluator-step cost of the second block in that pair of
tests, not a cost measured on any other message. Each generated IV
in Listing J6.5 uses 11674 steps. The repository file's SHA-512/256
test of “abc” uses 11592 steps. That test hashes from a stored IV.
It is not the IV generation.

### J6.7 The 32-bit distances on a 64-bit word

`Σ1{256}`, equation (4.6) in §4.1.2, is `ROTR 6` xor `ROTR 11` xor
`ROTR 25`. `Σ1{512}`, equation (4.11), is `ROTR 14` xor `ROTR 18`
xor `ROTR 41`. Both are rotations. On a `Word[64]` both sets of
distances are legal literal amounts. The width does not select the
equation.

The prediction, before the test runs, is that the two functions
disagree on `0x0123456789abcdef`. The word from the 32-bit distances
is `0xd4025efda375298a`. The word from equation (4.11) is
`0x7703112333475567`.

**Listing J6.6 — `bad_sigma.or`**

```orange
edition 2026;
module bad_sigma {
  spec big_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 14) ^ (x >>> 18) ^ (x >>> 41) }
  spec bad_sigma1_512(x: Word[64]) -> Word[64] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }

  spec left() -> Word[64] { bad_sigma1_512(0x0123456789abcdef) }
  spec right() -> Word[64] { big_sigma1_512(0x0123456789abcdef) }

  test "32-bit Sigma1 distances on a 64-bit word" { left() == right() }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec left -
./compiler/target/debug/orangec eval --spec right -
./compiler/target/debug/orangec test -
```

Check is silent. The swapped distances were well-formed.

**Expected evaluation output:**

```text
bad_sigma::left: Word[64] = 0xd4025efda375298a
bad_sigma::right: Word[64] = 0x7703112333475567
```


**Test report:**

```text
test "32-bit Sigma1 distances on a 64-bit word" ... FAILED
    left:  0xd4025efda375298a
    right: 0x7703112333475567
1 test: 0 passed, 1 failed
```


The status of `test` is 1. Standard error is empty. `left` is the
value the 32-bit distances computed. `right` is equation (4.11).
The repair is to put `>>> 14`, `>>> 18`, and `>>> 41` back.

### J6.8 A 64-byte block where the parameter wants 128

`padding[3, 1]()` is the pad at `s = 1`, a 64-byte block, the size
SHA-256 uses. `sha512` is defined for `n` in `1..3`, and its
argument has length `128·n`. The call
`sha512("abc" ++ padding[3, 1]())` passes an array of length 64.
The prediction is a size error, not a digest. The stub body that
returns 64 zero bytes is not a result, because the call is rejected
before a test runs.

**Listing J6.7 — `carried.or`, intentionally rejected**

```orange
edition 2026;
module carried {
  spec padding[len in 0..113, s in 1..3]()
    -> Word[8]^(64 * s * ((len + 72 * s) / (64 * s)) - len) {
    let l: Int = 8 * len;
    ([0; (64 * s * ((len + 72 * s) / (64 * s)) - len - 8 * s)] with [0] = 0x80)
      ++ (l as big Word[8]^(8 * s))
  }

  spec sha512[n in 1..3](p: Word[8]^(128 * n)) -> Word[8]^64 { [0; 64] }

  spec wrong() -> Word[8]^64 { sha512("abc" ++ padding[3, 1]()) }
}
```


```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0238]: no instance of `sha512` takes arguments of these lengths
 --> <stdin>:12:32
   |
12 |   spec wrong() -> Word[8]^64 { sha512("abc" ++ padding[3, 1]()) }
   |                                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ an array of length 64 is given
  = note: `sha512` is defined for `n` in 1..3
  = note: a call that writes no sizes calls the one instance of its function whose array parameters have the lengths of its arguments; any other call writes its sizes in brackets, as in `absorb[2](p)`
```


The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic and no value. The label says an array of length
64 is given. The note says `sha512` is defined for `n` in `1..3`.
No test report is printed.

### J6.9 A 64-bit length on the 112-byte message

Proposition J6.2 puts 16 length bytes at the end of the SHA-512
pad. For a message of `0` through `111` bytes the pad fits in one
128-byte block, and `ℓ < 2^64`, so the high eight bytes of the
length are zero. An eight-byte length at the end of that same
128-byte block is the same bytes. “abc” is in that range. Listing
J6.1 already compared the padding bytes. Listing J6.8 hashes them.

The 112-byte message is the first whole-byte length that does not
fit. Proposition J6.5 puts `0x80` in the first block and the length
in the second. A pad of `0x80`, seven zero bytes, and an eight-byte
length squeezes the message into one block of 128 bytes. That block
is not the padded message §5.1.2 defines. The prediction is that
the digest differs, and that the first byte is not the examples
file's `0x8e`.

**Listing J6.8 — `short_field.or`**

```orange
edition 2026;
module short_field {
  use sha2;

  spec abc_short_field() -> Word[8]^128 {
    let bits: Int = 24;
    let length: Word[8]^8 = bits as big Word[8]^8;
    "abc" ++ hex"80" ++ [0; 116] ++ length
  }

  spec two_short_field() -> Word[8]^128 {
    let bits: Int = 896;
    let length: Word[8]^8 = bits as big Word[8]^8;
    let m: Word[8]^112 =
      "abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmn" ++
      "hijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu";
    m ++ hex"80" ++ [0; 7] ++ length
  }

  spec first() -> Word[8] {
    let digest: Word[8]^64 = sha2::sha512(two_short_field());
    digest[0]
  }

  test "abc with a 64-bit length field still matches" {
    sha2::sha512(abc_short_field()) == (
      hex"ddaf35a193617aba cc417349ae204131 12e6fa4e89a97ea2 0a9eeee64b55d39a" ++
      hex"2192992a274fc1a8 36ba3c23a3feebbd 454d4423643ce80e 2a9ac94fa54ca49f"
    )
  }

  test "112-byte message with a 64-bit length field" {
    sha2::sha512(two_short_field()) == (
      hex"8e959b75dae313da 8cf4f72814fc143f 8f7779c6eb9f7fa1 7299aeadb6889018" ++
      hex"501d289e4900f7e4 331b99dec4b5433a c7d329eeb6dd2654 5e96e55b874be909"
    )
  }
}
```


The listing says `use sha2`. A listing read from standard input
resolves that name in the process's current directory. The commands
are run with current directory `algorithms/sha2`, where `sha2.or`
is the repository file. The locus stays `<stdin>`, because the
listing itself is the standard input.

```sh
cd algorithms/sha2
../../compiler/target/debug/orangec check -
../../compiler/target/debug/orangec eval --spec first -
../../compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
short_field::first: Word[8] = 0xfe
```


**Test report:**

```text
test "abc with a 64-bit length field still matches" ... ok
test "112-byte message with a 64-bit length field" ... FAILED
    left:  [0xfe, 0xb8, 0x85, 0xc0, 0xc5, 0x68, 0x38, 0x41, 0x36, 0x25, 0xba, 0x4e, 0x15, 0xf3, 0xe6, 0x11, 0x6f, 0xdb, 0x2e, 0xbe, 0x26, 0xf9, 0xc1, 0xd4, 0xb9, 0xea, 0x8b, 0xea, 0x9a, 0x98, 0x47, 0x02, 0xa6, 0x1b, 0xbd, 0xba, 0xd7, 0x84, 0xfa, 0x99, 0x5d, 0x9a, 0xb8, 0x87, 0x9b, 0xef, 0xab, 0x0d, 0xd7, 0xd8, 0x56, 0x6d, 0x26, 0xa5, 0xad, 0x95, 0xcd, 0xf4, 0x2f, 0xdd, 0xdf, 0x08, 0x81, 0x23]
    right: [0x8e, 0x95, 0x9b, 0x75, 0xda, 0xe3, 0x13, 0xda, 0x8c, 0xf4, 0xf7, 0x28, 0x14, 0xfc, 0x14, 0x3f, 0x8f, 0x77, 0x79, 0xc6, 0xeb, 0x9f, 0x7f, 0xa1, 0x72, 0x99, 0xae, 0xad, 0xb6, 0x88, 0x90, 0x18, 0x50, 0x1d, 0x28, 0x9e, 0x49, 0x00, 0xf7, 0xe4, 0x33, 0x1b, 0x99, 0xde, 0xc4, 0xb5, 0x43, 0x3a, 0xc7, 0xd3, 0x29, 0xee, 0xb6, 0xdd, 0x26, 0x54, 0x5e, 0x96, 0xe5, 0x5b, 0x87, 0x4b, 0xe9, 0x09]
    first difference at [0]
2 tests: 1 passed, 1 failed
```


The status of `test` is 1. Standard error is empty. The “abc” test
passed. The 112-byte test failed at byte 0: `left` is `0xfe`,
`right` is `0x8e`. The passing half is the one-block case. A suite
that stopped at “abc” would accept the short field.

### J6.10 The byte `0x80` omitted

Proposition J6.4 says the pad of “abc” is 125 bytes, beginning with
`0x80`, and ending with the length byte `0x18`. An array of 125
zero bytes with only the last byte set to 24 has the length and
does not have the `1` bit. The prediction is that the arrays differ
at index 0: `0x00` against `0x80`.

**Listing J6.9 — `missing80.or`**

```orange
edition 2026;
module missing80 {
  spec padding[len in 0..113, s in 1..3]()
    -> Word[8]^(64 * s * ((len + 72 * s) / (64 * s)) - len) {
    let l: Int = 8 * len;
    ([0; (64 * s * ((len + 72 * s) / (64 * s)) - len - 8 * s)] with [0] = 0x80)
      ++ (l as big Word[8]^(8 * s))
  }

  spec miss() -> Word[8]^125 { [0; 125] with [124] = 24 }

  spec first_bad() -> Word[8] {
    let bytes: Word[8]^125 = miss();
    bytes[0]
  }

  spec first_good() -> Word[8] {
    let bytes: Word[8]^125 = padding[3, 2]();
    bytes[0]
  }

  test "missing 0x80" { miss() == padding[3, 2]() }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec first_bad -
./compiler/target/debug/orangec eval --spec first_good -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
missing80::first_bad: Word[8] = 0x00
missing80::first_good: Word[8] = 0x80
```


**Test report:**

```text
test "missing 0x80" ... FAILED
    left:  [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18]
    right: [0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18]
    first difference at [0]
1 test: 0 passed, 1 failed
```


The status of `test` is 1. Standard error is empty. The first
difference is at index 0. The report prints both 125-byte arrays.
The repair is the pad of Listing J6.1.

### J6.11 Truncation without the variant IV

Proposition J6.13 says that dropping bytes of a SHA-512 digest keeps
the initial value of §5.3.5. SHA-384 and SHA-512/256 do not. The
prediction, for “abc”, is that the first byte of the truncated
SHA-512 digest is `0xdd`, against `0xcb` for SHA-384 and against
`0x53` for SHA-512/256.

**Listing J6.10 — `truncate.or`**

```orange
edition 2026;
module truncate {
  use sha2;

  spec sha384_abc() -> Word[8]^48 {
    hex"cb00753f45a35e8b b5a03d699ac65007 272c32ab0eded163" ++
    hex"1a8b605a43ff5bed 8086072ba1e7cc23 58baeca134c825a7"
  }

  spec sha512_256_abc() -> Word[8]^32 {
    hex"53048e2681941ef9 9b2e29b76b4c7dab e4c2d0c634fc6d46 e0e2f13107e7af23"
  }

  spec first384() -> Word[8] {
    let digest: Word[8]^48 = sha2::sha512("abc" ++ sha2::padding[3, 2]())[..48];
    digest[0]
  }

  spec first256() -> Word[8] {
    let digest: Word[8]^32 = sha2::sha512("abc" ++ sha2::padding[3, 2]())[..32];
    digest[0]
  }

  test "SHA-384 truncation without its own IV" {
    sha2::sha512("abc" ++ sha2::padding[3, 2]())[..48] == sha384_abc()
  }

  test "SHA-512/256 truncation without its own IV" {
    sha2::sha512("abc" ++ sha2::padding[3, 2]())[..32] == sha512_256_abc()
  }
}
```


The commands are run with current directory `algorithms/sha2`, for
the same reason as Listing J6.8.

```sh
cd algorithms/sha2
../../compiler/target/debug/orangec check -
../../compiler/target/debug/orangec eval --spec first384 -
../../compiler/target/debug/orangec eval --spec first256 -
../../compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
truncate::first384: Word[8] = 0xdd
truncate::first256: Word[8] = 0xdd
```


**Test report:**

```text
test "SHA-384 truncation without its own IV" ... FAILED
    left:  [0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a, 0x21, 0x92, 0x99, 0x2a, 0x27, 0x4f, 0xc1, 0xa8, 0x36, 0xba, 0x3c, 0x23, 0xa3, 0xfe, 0xeb, 0xbd]
    right: [0xcb, 0x00, 0x75, 0x3f, 0x45, 0xa3, 0x5e, 0x8b, 0xb5, 0xa0, 0x3d, 0x69, 0x9a, 0xc6, 0x50, 0x07, 0x27, 0x2c, 0x32, 0xab, 0x0e, 0xde, 0xd1, 0x63, 0x1a, 0x8b, 0x60, 0x5a, 0x43, 0xff, 0x5b, 0xed, 0x80, 0x86, 0x07, 0x2b, 0xa1, 0xe7, 0xcc, 0x23, 0x58, 0xba, 0xec, 0xa1, 0x34, 0xc8, 0x25, 0xa7]
    first difference at [0]
test "SHA-512/256 truncation without its own IV" ... FAILED
    left:  [0xdd, 0xaf, 0x35, 0xa1, 0x93, 0x61, 0x7a, 0xba, 0xcc, 0x41, 0x73, 0x49, 0xae, 0x20, 0x41, 0x31, 0x12, 0xe6, 0xfa, 0x4e, 0x89, 0xa9, 0x7e, 0xa2, 0x0a, 0x9e, 0xee, 0xe6, 0x4b, 0x55, 0xd3, 0x9a]
    right: [0x53, 0x04, 0x8e, 0x26, 0x81, 0x94, 0x1e, 0xf9, 0x9b, 0x2e, 0x29, 0xb7, 0x6b, 0x4c, 0x7d, 0xab, 0xe4, 0xc2, 0xd0, 0xc6, 0x34, 0xfc, 0x6d, 0x46, 0xe0, 0xe2, 0xf1, 0x31, 0x07, 0xe7, 0xaf, 0x23]
    first difference at [0]
2 tests: 0 passed, 2 failed
```


The status of `test` is 1. Standard error is empty. Both tests fail
at byte 0. `left` is `0xdd` in both. This is one block. It is not
a proof that every block separates the three initial values.

### J6.12 What this slice does not denote

The 128-bit length is a pair of 64-bit words, by Assumption J6.3.
It is not a `Word[128]`.

**Listing J6.11 — `word128.or`, intentionally rejected**

```orange
edition 2026;
module word128 {
  spec w() -> Word[128] { 0 }
}
```


```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0204]: `Word` width must be exactly 8, 16, 32, or 64
 --> <stdin>:3:20
  |
3 |   spec w() -> Word[128] { 0 }
  |                    ^^^ unsupported word width
  = note: word widths do not coerce, truncate, or wrap
```


The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic. The note says word widths do not coerce,
truncate, or wrap.

A literal rotation amount on `Word[64]` is from 0 through 63.
Equation (4.13)'s distances are inside that range. A literal 64 is
not an amount §4.1.3 writes.

**Listing J6.12 — `rot64.or`, intentionally rejected**

```orange
edition 2026;
module rot64 {
  spec w(x: Word[64]) -> Word[64] { x >>> 64 }
}
```


```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0216]: `>>>` on `Word[64]` needs an amount from 0 through 63
 --> <stdin>:3:43
  |
3 | ... spec w(x: Word[64]) -> Word[64] { x >>> 64 }
  |                                             ^^ a literal amount is from 0 through 63
  = note: an amount written as one integer literal is from 0 through n - 1; any other amount is computed, an `Int` or a word, such as `x <<< r` or `x >> (i % 8)`
```


The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic. The note distinguishes a literal amount from a
computed amount.

A computed amount is a different rule, and it is not a FIPS
equation. `docs/AMOUNTS_2026.md` states that a computed amount
rotates by the amount modulo the width. The proposal that specifies
it, OEP-0021, is not accepted by this lesson. The binary applies
that reduction. Listing J6.13 is what the binary did: a computed 64
returns the input word, and a computed 72 equals a literal
`>>> 8`.

**Listing J6.13 — `rot_computed.or`**

```orange
edition 2026;
module rot_computed {
  spec by(x: Word[64], n: Int) -> Word[64] { x >>> n }

  spec same() -> Word[64] { by(0x0123456789abcdef, 64) }
  spec eight() -> Word[64] { by(0x0123456789abcdef, 72) }
  spec literal8() -> Word[64] { 0x0123456789abcdef >>> 8 }

  test "computed amount 64 is a turn by 0" { same() == 0x0123456789abcdef }
  test "computed amount 72 is a turn by 8" { eight() == literal8() }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec same -
./compiler/target/debug/orangec eval --spec eight -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
rot_computed::same: Word[64] = 0x0123456789abcdef
rot_computed::eight: Word[64] = 0xef0123456789abcd
```


**Test report:**

```text
test "computed amount 64 is a turn by 0" ... ok
test "computed amount 72 is a turn by 8" ... ok
2 tests: 2 passed, 0 failed
```


The status is 0. These two tests are the amount rule. They are not
a round of SHA-512.

`+` and `*` on `Word[64]` are residues modulo `2^64`, by Assumption
J6.2. `(2^64 − 1) + 2 = 2^64 + 1`, whose residue is 1.
`(2^64 − 1)^2 = 2^128 − 2·2^64 + 1`, whose residue modulo `2^64` is
also 1. The round uses addition. The product is here so the modulus
is visible on both operations.

**Listing J6.14 — `wrap64.or`**

```orange
edition 2026;
module wrap64 {
  spec add() -> Word[64] { 0xffffffffffffffff + 2 }
  spec mul() -> Word[64] { 0xffffffffffffffff * 0xffffffffffffffff }

  test "sum past 2^64" { add() == 1 }
  test "product past 2^64" { mul() == 1 }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec add -
./compiler/target/debug/orangec eval --spec mul -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
wrap64::add: Word[64] = 0x0000000000000001
wrap64::mul: Word[64] = 0x0000000000000001
```


**Test report:**

```text
test "sum past 2^64" ... ok
test "product past 2^64" ... ok
2 tests: 2 passed, 0 failed
```


The status is 0. The printed words are `0x0000000000000001`. The
tests compare them with the integer `1`.

`Int` is not a word. `4611686018427387904 · 4 = 2^64`, and that
product is an `Int`, not a `Word[64]`. A shift of an `Int` is not
defined.

**Listing J6.15 — `no_shift.or`, intentionally rejected**

```orange
edition 2026;
module no_shift {
  spec p() -> Int { 1 << 310 }
}
```


```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0215]: `<<` is not defined for `Int`
 --> <stdin>:3:23
  |
3 |   spec p() -> Int { 1 << 310 }
  |                       ^^ `Int` is required here
  = note: shifts and rotations apply only to `Word[n]` values
```


The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic. `2^310` in the next listing is 310
multiplications by 2, starting from 1. The loop `for i in 0..310`
runs 310 times, because the end is exclusive.

`docs/ORDER_2026.md` §5 states that an `Int` converts to words by
its residue modulo `2^{nk}`, in the chosen order. The proposal that
specifies `as big`, OEP-0017, is not accepted by this lesson.
`2^310 = 2^182 · 2^128`, so the residue modulo `2^128` is 0. Sixteen
zero bytes are that residue. Listing J6.16 is what the binary
printed. Using the form is not acceptance of the proposal.

**Listing J6.16 — `wide_int.or`**

```orange
edition 2026;
module wide_int {
  spec product() -> Int { 4611686018427387904 * 4 }

  spec power() -> Int {
    let two: Int = 2;
    for i in 0..310 with p: Int = 1 { p * two }
  }

  spec bytes() -> Word[8]^16 {
    let n: Int = power();
    n as big Word[8]^16
  }

  test "2^62 times 4" { product() == 18446744073709551616 }
}
```


```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec product -
./compiler/target/debug/orangec eval --spec bytes -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
wide_int::product: Int = 18446744073709551616
wide_int::bytes: Word[8]^16 = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```


**Test report:**

```text
test "2^62 times 4" ... ok
1 test: 1 passed, 0 failed
```


The status is 0. The sixteen bytes are zero. The product is `2^64`,
printed as an `Int`.

`padding` is defined for `len` in `0..113`. The call at 128 asks
for an instance that does not exist. The result type on the spec is
`Word[8]^128`. That is the appended-byte count the formula of
Proposition J6.3 gives at `len = 128`:
`128 · ((128 + 144) / 128) − 128 = 128`. The diagnostic that is
printed is the size error. No value is produced.

**Listing J6.17 — `past_range.or`, intentionally rejected**

```orange
edition 2026;
module past_range {
  spec padding[len in 0..113, s in 1..3]()
    -> Word[8]^(64 * s * ((len + 72 * s) / (64 * s)) - len) {
    let l: Int = 8 * len;
    ([0; (64 * s * ((len + 72 * s) / (64 * s)) - len - 8 * s)] with [0] = 0x80)
      ++ (l as big Word[8]^(8 * s))
  }

  spec too_long() -> Word[8]^128 { padding[128, 2]() }
}
```


```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0238]: `padding` is defined for `len` in 0..113
 --> <stdin>:10:44
   |
10 | ... pec too_long() -> Word[8]^128 { padding[128, 2]() }
   |                                             ^^^ this size is 128
  = note: a size parameter `n in a..b` takes each value from a up to, but not including, b, with a < b <= 65536, and a function has at most 256 instances
```


The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic. The label says this size is 128. The note
states the range rule and the instance limit.

`sha512` is defined for `n` in `1..3`. The call `sha512[4]` passes
512 zero bytes, which is `128 · 4`, the length the instance would
take if `n = 4` were in range. The prediction is the same class of
size error, on `n` rather than on `len`.

**Listing J6.18 — `n_bound.or`, intentionally rejected**

```orange
edition 2026;
module n_bound {
  spec sha512[n in 1..3](p: Word[8]^(128 * n)) -> Word[8]^64 { [0; 64] }

  spec four() -> Word[8]^64 { sha512[4]([0; 512]) }
}
```


```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0238]: `sha512` is defined for `n` in 1..3
 --> <stdin>:5:38
  |
5 |   spec four() -> Word[8]^64 { sha512[4]([0; 512]) }
  |                                      ^ this size is 4
  = note: a size parameter `n in a..b` takes each value from a up to, but not including, b, with a < b <= 65536, and a function has at most 256 instances
```


The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic. The label says this size is 4.

A function has at most 256 instances. `len in 0..113` and
`s in 1..3` is 226, which is inside the limit. `len in 0..200` and
`s in 1..3` is 400. The code is `ORC0238` again, with a different
message: the function has 400 instances.

**Listing J6.19 — `too_many.or`, intentionally rejected**

```orange
edition 2026;
module too_many {
  spec padding[len in 0..200, s in 1..3]() -> Word[8]^1 { [0] }
}
```


```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0238]: `padding` has 400 instances, but a function has at most 256
 --> <stdin>:3:16
  |
3 |   spec padding[len in 0..200, s in 1..3]() -> Word[8]^1 { [0] }
  |                ^^^^^^^^^^^^^^^^^^^^^^^^ too many instances
  = note: a size parameter `n in a..b` takes each value from a up to, but not including, b, with a < b <= 65536, and a function has at most 256 instances
```


The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic. The body is not the point. The caret is on the
size parameters.

The tests of one file share one evaluator-step budget. The default
is 1048576. The option `--steps` admits a budget from 1 through
1073741824. The thirteen tests of `algorithms/sha2/sha2.or` use
196481 steps together, which is inside the default. One of them,
the SHA-512 test of “abc”, uses 11595. Listing J6.20 is that one
comparison, through `use sha2`. At `--steps 11595` the test passes
and the stats line says 11595 of 11595. At `--steps 11594` the run
stops with `ORC0301`. No test outcome is reported.

`orangec check --steps 10 algorithms/sha2/sha2.or` exits 2. Its
first line is `orangec: option `--steps` applies only to eval, test, replay, and analyze`.
`orangec test --steps 0 algorithms/sha2/sha2.or` exits 2. Its first
line is `orangec: option `--steps` takes a number of steps from 1 through 1073741824`.
The same first line is printed for 1073741825. Both messages
continue with the usage text. This lesson does not copy that text.

The NIST long message of 1000000 bytes of `0x61` has bit length
8000000. `1000000 = 128 · 7812 + 64`, so 7812 blocks are full and
64 message bytes remain. Those 64 bytes, the `0x80` byte, and the
16 length bytes total 81, which fits in the last block, so
`N = 7813`. This lesson does not run it. `sha512` is defined only
for `n` in `1..3`, and `padding` only for `len` in `0..112`, so the
message is not a call these listings can write. The estimate from
the two measured SHA-512 tests in `sha2.or` is
`11595 + 7812 · 11550 = 90240195` evaluator steps. It is an
estimate from that pair. It is not a run.

A message of 128 bytes is one full block, so the padding is a
second block by itself. `len = 128` is outside `0..113`. It is
named here and not run.

Messages in these listings are whole bytes, by Assumption J6.8.
SHA-512/t is run only at `t = 224` and `t = 256`. SHA-224 is the
pointer in Proposition J6.9. HMAC is J14. None of those is a
listing on this page.

**Listing J6.20 — `budget.or`**

```orange
edition 2026;
module budget {
  use sha2;

  test "sha512_abc_len_bits_24" {
    sha2::sha512("abc" ++ sha2::padding[3, 2]()) == (
      hex"ddaf35a193617aba cc417349ae204131 12e6fa4e89a97ea2 0a9eeee64b55d39a" ++
      hex"2192992a274fc1a8 36ba3c23a3feebbd 454d4423643ce80e 2a9ac94fa54ca49f"
    )
  }
}
```


The commands are run with current directory `algorithms/sha2`.

```sh
cd algorithms/sha2
../../compiler/target/debug/orangec check -
../../compiler/target/debug/orangec test --steps 11595 --stats -
../../compiler/target/debug/orangec test --steps 11594 --stats -
```

Check is silent. At 11595 steps the test passes. The stats on
standard error are:

```text
test "sha512_abc_len_bits_24": 11595 steps
total: 11595 of 11595 steps
```


**Test report:**

```text
test "sha512_abc_len_bits_24" ... ok
1 test: 1 passed, 0 failed
```


The status of that `test` is 0. Standard error is empty when
`--stats` is absent. At 11594 steps the status is 1, standard
output is empty, and standard error is:

```text
error[ORC0301]: reference evaluation step limit exceeded
 --> <stdin>:5:8
  |
5 |   test "sha512_abc_len_bits_24" {
  |        ^^^^^^^^^^^^^^^^^^^^^^^^ evaluation stopped while evaluating this test
  = note: at most 11594 evaluation steps are permitted
  = note: no test outcome is reported
  = note: `orangec test --steps N` sets the budget, up to 1073741824 steps
```


No test report is printed. The note names the upper bound
1073741824.

### J6.13 Exercises

**Exercise J6.1.** For “abc”, state `ℓ`, `k`, the number of zero
bytes after `0x80`, the appended-byte count, and the padded length
in bits. Do the same for the 112-byte message, and say which block
holds the length. Name the proposition each answer comes from.

**Exercise J6.2.** For `H(0)_0`, state `s`, `s^2`, `2^129`, the
gap, and `2·s + 1`. Why does the gap being smaller than `2·s + 1`
finish the proof that `s = floor(sqrt(2^129))`? State the
corresponding gap and step for `K_0`, and the one subtraction that
shows the next cube lies above `2^193`.

**Exercise J6.3.** Derive `W16` and `W17` of the “abc” schedule.
Show `ROTR 19(0x18)`, `ROTR 61(0x18)`, and `SHR 6(0x18)`. Does the
examples file print `W17`?

**Exercise J6.4.** Give the integer inside `T1` before reduction,
how many times it crosses `2^64`, and the residue. Do the same for
`T2`, for the new `a`, and for the new `e`.

**Exercise J6.5.** `Σ1{512}` is rewritten with the distances 6, 11,
and 25, each still a rotation, on the word `0x0123456789abcdef`.
What does check report? What are `left` and `right`?

**Exercise J6.6.** Why does a 64-bit length field match the SHA-512
padding of “abc”, and fail for the 112-byte message? What is the
first byte of the digest the short field produces on that message,
and what is the first byte the examples file prints?

**Exercise J6.7.** Truncating the SHA-512 digest of “abc” to 48
bytes fails against SHA-384, and truncating it to 32 bytes fails
against SHA-512/256. What does Proposition J6.13 prove about the
specification, and what does it not prove about the compression
function? What does the “abc” witness add?

**Exercise J6.8.** Name the result of `Word[128]`, of a literal
`>>> 64`, of `2^310` converted to sixteen bytes, and of the
million-byte message. Which of those is a diagnostic, which is a
value the binary printed, and which was not run? Where is HMAC in
this curriculum, and what does this lesson not build?

## Worked answers

**J6.1.** For “abc”, `ℓ = 24`, `k = 871`, the zero bytes number
108, the pad is 125 bytes, and the padded length is 1024 bits.
Propositions J6.1, J6.2, and J6.4. For the 112-byte message,
`ℓ = 896`, `k = 1023`, the zero bytes number 127, the pad is 144
bytes, and the padded length is 2048 bits. The byte `0x80` is in
the first block. The length is in the second block, and its last
word is `0x380`. Proposition J6.5. The last whole-byte one-block
message is 111 bytes, where `k = 7`.

**J6.2.** `s = 26087635650665564424`.
`s^2 = 680564733841876926890271207202494451776`.
`2^129 = 680564733841876926926749214863536422912`.
The gap is `36478007661041971136`.
`2·s + 1 = 52175271301331128849`. The gap is the amount by which
`2^129` exceeds `s^2`. The next square exceeds `s^2` by `2·s + 1`.
A smaller gap means `2^129` is still below `(s + 1)^2`, so `s` is
the integer square root. For `K_0`, the gap is
`391541399527358655497277168604587114104` and the step to the next
cube is `1620493761649579679220648177341463314419`. Their difference
is `1228952362122221023723371008736876200315`, which is positive, so
the next cube lies above `2^193`. Proposition J6.7 and Proposition
J6.8.

**J6.3.** `W16 = 0x6162638000000000`, because the other three terms
at `t = 16` are zero. Proposition J6.10.
`ROTR 19(0x18) = 0x0003000000000000`,
`ROTR 61(0x18) = 0x00000000000000c0`, and
`SHR 6(0x18) = 0`. Their exclusive or is `0x00030000000000c0`,
which is `W17`. Proposition J6.11. The examples file does not print
`W17`.

**J6.4.** The integer inside `T1` is `31379689096978360480`. It
crosses `2^64` once. The residue is `12932945023268808864`, which
is `0xb37b0cfa1b97e8a0`. The integer inside `T2` is
`4842708523825821013`. It does not cross, and the word is
`0x4334c1bea164f555`. The new `a` is `17775653547094629877`, which
does not cross. The new `e` is `24844954193739718545`. It crosses
once, and the residue is `6398210120030166929`, which is
`0x58cb02347ab51f91`. Proposition J6.12.

**J6.5.** Check is silent. The distances are legal amounts on
`Word[64]`. The test fails. `left` is `0xd4025efda375298a`.
`right` is `0x7703112333475567`. The failure is the report, not a
diagnostic.

**J6.6.** For “abc” the pad fits in one 128-byte block, and the
high eight bytes of a 128-bit length below `2^64` are zero, so an
eight-byte length at the end of that block is the same bytes. At
112 bytes, `k = 1023`, and the length does not fit in the first
block. The short field builds one block whose digest begins with
`0xfe`. The examples file's digest begins with `0x8e`. Proposition
J6.5 and §J6.9.

**J6.7.** Proposition J6.13 proves that §6.5 and §6.7 feed a
different `H(0)` into the first compression, so those algorithms
are not “compute SHA-512 and drop bytes.” It does not prove that
the compression function is injective in its first argument. The
“abc” witness shows that for this one block the three digests
differ at byte 0: `0xdd`, `0xcb`, and `0x53`.
A Match is not called verified.

**J6.8.** `Word[128]` is rejected with `ORC0204`. A literal
`>>> 64` is rejected with `ORC0216`. `2^310` converted with
`as big` to sixteen bytes is sixteen zero bytes; check is silent,
and the value is what the binary printed under the residue rule in
`docs/ORDER_2026.md` §5, which this lesson does not accept as
OEP-0017. The million-byte message was not run. The estimate is
`90240195` evaluator steps, from `11595 + 7812 · 11550`. HMAC is
J14. This lesson does not build a key, a key block, or an outer
hash.

```text
j6-ledger
abc-bits = 24
abc-k = 871
abc-zero-bytes = 108
abc-appended = 125
abc-padded-bits = 1024
empty-k = 895
empty-zero-bytes = 111
empty-padded-bits = 1024
bound-bits = 888
bound-k = 7
bound-zero-bytes = 0
bound-appended = 17
bound-padded-bits = 1024
two-bits = 896
two-k = 1023
two-zero-bytes = 127
two-appended = 144
two-padded-bits = 2048
range-instances = 226
too-many-instances = 400
h0 = 7640891576956012808
sqrt-s = 26087635650665564424
sqrt-square = 680564733841876926890271207202494451776
two129 = 680564733841876926926749214863536422912
sqrt-gap = 36478007661041971136
sqrt-step = 52175271301331128849
sqrt-next = 15697263640289157713
k0 = 4794697086780616226
cbrt-c = 23241441160490167842
cbrt-cube = 12554203470773361527280037446887974176707433720323481911688
two193 = 12554203470773361527671578846415332832204710888928069025792
cbrt-gap = 391541399527358655497277168604587114104
cbrt-step = 1620493761649579679220648177341463314419
cbrt-next = 1228952362122221023723371008736876200315
h384 = 14680500436340154072
sqrt23-s = 88467476731178360536
sqrt23-target = 7826494439181584659657615970930668863488
sqrt23-gap = 159966782817866656192
sqrt23-step = 176934953462356721073
sqrt23-next = 16968170644490064881
sha224-h0 = 3238371032
rotr28-shr = 28464539263
rotr28-low = 62703880
rotr28-shl = 4308977822916935680
rotr28 = 4308977851381474943
rotr19 = 844424930131968
rotr61 = 192
w17 = 844424930132160
ch-round0 = 2271443192277056827
maj-round0 = 4210837510808201515
sigma0-h0 = 631871013017619498
sigma1-h4 = 10675751287297530826
t1-raw = 31379689096978360480
t1 = 12932945023268808864
t2-raw = 4842708523825821013
t2 = 4842708523825821013
new-a = 17775653547094629877
new-e-raw = 24844954193739718545
new-e = 6398210120030166929
one-block-steps = 11595
two-block-steps = 23145
extra-block-steps = 11550
million-a-blocks = 7813
million-a-estimate = 90240195
iv-steps = 11674
```

## Sources and epigraph record

The quotation is the borrowed wording. The proofs are the lesson's.
The hex words copied from §4.2.3, §5.3.4, and §5.3.5, other than the
words Propositions J6.7, J6.8, and J6.9 derive, are the standard's.
The SHA-512 “abc” digest, the SHA-512 two-block digest, the block
words those messages determine, the row `t = 0`, and the
intermediate hash after the first of the two blocks are the
examples file's.

**[S17] National Institute of Standards and Technology.** FIPS PUB
180-4, *Secure Hash Standard (SHS)*, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. The epigraph is the first sentence of
item 3 of the Announcement, on page iv of the PDF. The hyphens in
that sentence are ASCII hyphens. The sentence contains no inner
quotation marks. Wording was checked on 2026-10-17 against the file
whose SHA-256 is recorded in §J6.1. That hash is the hash J5
recorded on 2026-10-09 for the same file. No translation is
involved. The publication names no personal author of the
Announcement. The foreword is signed by Charles H. Romine, Director,
Information Technology Laboratory; the epigraph is not taken from
the foreword. J5's record of the
same publication is [S16]. This record is [S17] so that the
epigraph tag is not a second definition of [S16].
Sections used for the derivation, and not as the epigraph, are
§3.1, §3.2, §4.1.3, §4.2.3, §5.1.2, §5.2.2, §5.3.4, §5.3.5, §5.3.6,
§5.3.6.1, §5.3.6.2, §6.4.1, §6.4.2, §6.5, §6.6, and §6.7. §5.3.2
and §6.3 are named and not run. Appendix A.1 is named and not
transcribed. Appendix A.2 is the sentence that points at examples
and does not print a digest. The erratum page records the SHA-1
edit described in §J6.1.
This record's tag is [S17].

Source: <https://doi.org/10.6028/NIST.FIPS.180-4>

**[T13] Retrieved files.** On 2026-10-17 the lesson retrieved the
August 2015 PDF from the DOI above. Its SHA-256 is
`0455b406d89648d20cbde375561e19c245b9815e894164c2670772e3d54deb82`.
The same day the lesson retrieved the NIST SHA-512 examples file.
Its SHA-256 is
`84ce867c99bad5a7b05abc74bb1dfc9de8486f49a160653cc8b15272cc532115`.
That file is not a section of [S17]. The one-block SHA-512 digest,
the two-block SHA-512 digest, the block contents, the row `t = 0`,
and the intermediate hash after the first of the two blocks are
copied from it. It does not contain `W17`, and it does not contain
a SHA-384 digest. The empty SHA-512 digest, the two SHA-384
digests, and the SHA-512/256 digest of “abc” are the bytes
`algorithms/sha2/sha2.or` already demands, compared during this
drafting with Python's `hashlib`. The SHA-512/224 and SHA-512/256
initial values are the words §5.3.6.1 and §5.3.6.2 print,
regenerated rather than copied into the hash that produces them.
This record's tag is [T13].

Source: <https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA512.pdf>

**[C7] Family surface.** The listings use `edition 2026`, `module`,
`use`, `spec`, `let`, `Word[8]`, `Word[32]`, `Word[64]`, `Int`,
`^`, `&`, `~`, `+`, `*`, `>>`, `>>>`, `++`, `&&`, `hex"80"`, a
string literal, `[0; n]`, `for`, `with`, `as big`, a call with
size arguments, a type parameter over `Word[32]` and `Word[64]`,
and `test`. `as big` is implemented on the S3u binary and is
specified by OEP-0017, which this lesson does not accept. A
computed rotation amount is specified by OEP-0021, which this
lesson does not accept. Identifiers are ASCII. An integer literal
is not an operand of `as` until a `let` types it. There is no
`Word[128]` and no shift of an `Int`. A `for` bound in these
listings is a literal, and the end is exclusive. A length
parameter in a static range is implemented on this slice. The
version line these commands print is `orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`.
The policy file in this tree is `policy/gate0-repository-policy.json`,
policy version 0.2.170. That version is not part of the compiler's
version line.
This record's tag is [C7].

The test form is the one N12 recorded as [T6]. `orangec check`
checks a test and does not run it. `orangec eval --spec` runs one
parameterless spec and does not run tests. `orangec test` runs the
tests. A failed check prints the diagnostic and no report. A failed
test prints `left` and `right` and no diagnostic. `orangec test
--stats` prints the evaluator-step count of each test on standard
error. Those counts are the counts Assumption J6.6 names.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

J6 adds eight exercises with worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`. The Orange listings
are the twenty fenced programs in this file.
`compiler/crates/orangec/tests/book_j6.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not prove
Assumption J6.2 from a smaller theory of the compiler, and they do
not accept OEP-0017 or OEP-0021. A passing test in that file is
a Match of the `Bool` the listing writes.
Do not call that Match verified.
Reaching the last page is not a certificate.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-17 at the
owner's direction. Owner review is pending.
