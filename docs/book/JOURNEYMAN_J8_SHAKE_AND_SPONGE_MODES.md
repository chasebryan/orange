# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J8: SHAKE and sponge modes. Draft 2026-10-17.

Continue from
[Keccak-p and SHA-3](JOURNEYMAN_J7_KECCAK_P_AND_SHA3.md#j7-keccak-p-and-sha-3).
J7 derived Keccak-f at the two lane widths this slice lists, and
SHA3-256 on a whole-byte message. §6.2 of the same standard defines
SHAKE128 and SHAKE256. This lesson writes those two functions, the
sponge that returns a caller-chosen length, and the suffix that
separates them from SHA-3.
[Standards as Versioned Inputs](JOURNEYMAN_J2_STANDARDS_AS_VERSIONED_INPUTS.md#j2-standards-as-versioned-inputs),
[The Corpus as Acceptance Test](JOURNEYMAN_J3_THE_CORPUS_AS_ACCEPTANCE_TEST.md#j3-the-corpus-as-acceptance-test),
and
[Byte Order and Format Boundaries](JOURNEYMAN_J4_BYTE_ORDER_AND_FORMAT_BOUNDARIES.md#j4-byte-order-and-format-boundaries)
are Block A. They are in this tree. This lesson cites their titles.
It does not repeat those proofs, and it does not repeat J7's proofs
of θ, ρ, π, χ, ι, or the round count.

This lesson is **J8**. The locked label is J8. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. When
the text says “§6.2” or “Algorithm 8”, the number is a section or an
algorithm of the edition named in the same sentence.

## J8: SHAKE and Sponge Modes

> “The SHA-3 family consists of four cryptographic hash functions, called SHA3-224, SHA3-256, SHA3-384, and SHA3-512, and two extendable-output functions (XOFs), called SHAKE128 and SHAKE256.”
>
> — National Institute of Standards and Technology, FIPS PUB 202, *SHA-3 Standard: Permutation-Based Hash and Extendable-Output Functions* (August 2015), Abstract, page iii. [S19]

The sentence is in the Abstract, on page iii. It is not the first
sentence. The first sentence is J7's epigraph, recorded as [S18].
This sentence names the four hash functions and then the two
extendable-output functions. The hyphen in “SHA-3” and the hyphen in
“extendable-output” are ASCII hyphens, read from that file. The
announcement on page iv names the same six functions in a different
sentence. That sentence is not the epigraph.

**The only-this-stack test.** You will derive the sponge's second
permutation, the SHAKE rates, and the suffix byte `0x1F` from the
August 2015 text, then run the transcription with `orangec check`,
`orangec eval`, and `orangec test` on the compiler this tree builds.
You will also feed the compiler three programs it rejects, and one
program it accepts whose tests fail. The compiler's version line is
`orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`. Every
listing is written for that slice. Every fenced program is Current
on that slice. No listing is marked Proposed. A form this slice does
not implement is not given a syntax here. A program the compiler
rejects is still Current: the rejection is the result the listing
exists to show.

### J8.1 The sections, in the order the standard uses them

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can name the sections of FIPS PUB 202, August 2015, that
   define the sponge, `pad10*1`, `KECCAK[c]`, SHAKE128, SHAKE256,
   the alternate RawSHAKE spelling, and the byte form of the pad,
   in the order §6.2 uses them, and you can separate that edition
   from the files that print the bytes.
2. You can compute the rate, the capacity, `q`, and the first and
   last pad bytes of SHAKE128 and of SHAKE256 for a whole-byte
   message with `q > 2`.
3. You can say how many times the permutation runs when the output
   is at most one rate, and how many times it runs when the output
   is longer than one rate and at most two.
4. You can show that SHA3-256 and SHAKE256 present different strings
   to one `KECCAK[512]`, and you can name the byte where the pinned
   outputs of “abc” differ.
5. You can say what Algorithm 8's integer `d` is, which finite
   ranges this slice accepts as a return length, and which three
   forms it rejects.
6. You can transcribe SHAKE128 of the empty message at 32 bytes and
   at 200 bytes, and SHAKE256 of the empty message and of “abc” at
   64 bytes, so that those outputs match the bytes this lesson pins,
   and you can predict the four failures before the report: SHA-3's
   pad byte `0x06` where SHAKE writes `0x1F`, SHAKE128's message
   absorbed at SHAKE256's rate, a truncation that starts one byte
   late, and a second block taken from the capacity instead of from
   a new permutation.

The edition is the file J7 pinned. It is FIPS PUB 202, August 2015,
DOI `10.6028/NIST.FIPS.202`. The file retrieved on 2026-10-17 from
<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf> has
SHA-256
`1592607831ff0908cc590632ce371c6c95e94025bb1a0c8ae90a4d0ec1ed025e`.
That is the digest [T14] records. This lesson retrieved the same
URL on the same day and computed the same digest. It does not
substitute another edition.

On the page of Algorithm 8, the note under the steps says that `d`
determines the number of bits returned. The next words are the
glyphs `the` and `ir` with no space between them, and a space before
`values`. Read as a word, that is `their`. The sentence is: the
input `d` determines how many bits Algorithm 8 returns, but it does
not affect their values. The following sentence says the output can
be regarded as an infinite string, whose computation is halted after
the desired number of bits is produced. This lesson uses that
reading. It does not treat `d` as an input to the permutation.

**Assumption J8.1 — The pin is the edition J7 recorded.** “SHA-3”,
“SHAKE”, and “XOF” name more than one document. The algorithms, the
section numbers, and the bit equations below are the August 2015
file whose digest is stated above. Lesson J2 is the lesson that
makes a pin a procedure. This lesson does not depend on that text.
The pin is stated in this assumption. J7's record of the same file
is [S18]. The epigraph tag of this lesson is [S19] so that the
sentence above is not a second definition of [S18].

**Assumption J8.2 — SHAKE128 and SHAKE256 are the two lines of §6.2.**
For a message `M` and an output length `d`,

```text
SHAKE128(M, d) = KECCAK[256](M || 1111, d)
SHAKE256(M, d) = KECCAK[512](M || 1111, d)
```

§6.2 says `d` is any output length. Algorithm 8's input line calls
`d` a nonnegative integer. The four lengths this lesson runs are
32 bytes, 64 bytes, 168 bytes inside the argument below, and 200
bytes. They are not every nonnegative integer.

**Assumption J8.3 — `KECCAK[c]` is the sponge of §5.2, on J7's
permutation.** §5.2 writes

```text
KECCAK[c] = SPONGE[KECCAK-p[1600, 24], pad10*1, 1600 − c]
```

§3.4 says `KECCAK-p[1600, 24]` is `KECCAK-f[1600]`. J7's Proposition
J7.6 is that substitution at `w = 64`: twenty-four rounds, indexed
from `0` through `23`. This lesson copies the `Sheet64` instance of
Listing J7.7 so that one module can call it. It does not re-prove
the five step mappings, the offsets, or the round constants. The
names are J7's names: `theta`, `rho`, `pi`, `chi`, `iota`,
`rc_bits`, `round_constants`, `lane_width`, `ell`, `keccak_f`. The
state is five sheets indexed by `x`, and `a[x][y]` is lane `(x, y)`,
which is Assumption J7.2. Bytes are packed low-bit-first, which is
Appendix B.1 and Assumption J7.4's mapping of the lane order.

**Assumption J8.4 — One absorb block, then a squeeze whose length is
the caller's.** Algorithm 8 is the sponge. For the messages in the
listings, the padded input is one rate block, so the absorb loop
runs once. The squeeze is steps 8 through 10. SHAKE128 has capacity
`256` and rate `1344`. SHAKE256 has capacity `512` and rate `1088`.
In both cases the width is `1600`, because `r + c = b` is the
equation §4 writes.

**Assumption J8.5 — The byte pad is Appendix B.2, for `q > 2`.**
A whole-byte message has `len(M) = 8m`. Appendix B.2 sets

```text
q = (r / 8) − (m mod (r / 8))
```

Table 6, for an XOF with `q > 2`, writes

```text
M || 0x1F || 0x00 … || 0x80
```

with `q − 2` zero bytes. The listings use that row. The rows
`q = 1` and `q = 2` are proved below and are not fed to the
permutation. A message whose length in bits is not a multiple of 8
is not built on this page.

**Assumption J8.6 — The examples are not a section of the standard.**
FIPS 202 does not print `SHAKE128` of the empty message, and it does
not print `SHAKE256` of “abc”. Appendix B names an examples page and
does not print a digest. The bytes cited below are the NIST 0-bit
files and the XKCP `Len = 0` lines that [T15] records, or the bytes
Python's `hashlib.shake_128` and `hashlib.shake_256` print for the
same messages, or both. “abc” is the `hashlib` string. It is not a
line of the August 2015 PDF, and it is not the `Len = 24` line of
either XKCP file. Lesson J3 is the lesson that treats a finite set
of pinned vectors as an acceptance test. This lesson does not depend
on that text. The acceptance set is the tests in Listing J8.7, and
the coverage sentence is stated in §J8.8.

**Assumption J8.7 — A Match is one `Bool` on the inputs written in
that test.** Well-formed source is not a Match. A Match on these
inputs is not a Match on every message, and it is not a Match on
every output length.
A Match is not called verified.
Nothing on this page is a security claim. A count of evaluator steps
is the number `orangec test --stats` prints for a test. It is not a
count of machine operations, and this lesson draws no conclusion
about speed from it.

**Assumption J8.8 — The hand calculation and the program denote the
same bytes only where both perform the same operations under J8.2
through J8.5.** Agreement on one message and one `d` is one point.
The domain of messages is every bit string. The domain of `d` is
the nonnegative integers. The domain of one 1600-bit state is
`2^1600` inputs.

**Assumption J8.9 — On this slice a return length is part of the
type.** Algorithm 8 takes `d` as an integer and returns a string of
that length. An Orange array length is not that integer. A size
parameter `n in a..b` takes each integer from `a` up to, but not
including, `b`, and one function has at most 256 instances. Those
are the diagnostics Listings J8.2, J8.3, and J8.4 print. This lesson
does not derive them from a smaller theory of the compiler.

Figure 7 of the August 2015 file is the picture of the sponge. The
algebraic definition this lesson uses is Algorithm 8. §4 is that
construction. §5.1 is `pad10*1`, Algorithm 9. §5.2 is `KECCAK[c]`.
§6.1 is the four hash functions; J7 ran SHA3-256, and this lesson
uses that definition only as the other suffix. §6.2 is SHAKE128 and
SHAKE256. §6.3 writes the same two functions as RawSHAKE on a
shorter suffix. Appendix B.1 is the byte packing. Appendix B.2 and
Table 6 are the hexadecimal pad. NIST SP 800-185 defines cSHAKE and
KMAC. This lesson does not retrieve that publication and does not
transcribe it. HMAC is the construction the curriculum names as
J14. This lesson does not build a key, a key block, or an outer
hash.

### J8.2 When the sponge calls the permutation again

Algorithm 8 is `SPONGE[f, pad, r](N, d)`. The steps, in the order
the standard numbers them, are these.

```text
1. Let P = N || pad(r, len(N)).
2. Let n = len(P)/r.
3. Let c = b − r.
4. Partition P into n strings P0, …, P(n−1), each of length r.
5. Let S = 0^b.
6. For i from 0 to n−1, let S = f(S ⊕ (Pi || 0^c)).
7. Let Z be the empty string.
8. Let Z = Z || Trunc_r(S).
9. If d ≤ |Z|, return Trunc_d(Z). Otherwise continue.
10. Let S = f(S), and continue with step 8.
```

The superscripts on the zeros are the widths `b` and `c`. Trunc_r
keeps the first `r` bits. Trunc_d keeps the first `d` bits. §4 says
`f` for the SHA-3 functions is a permutation. This lesson cites that
sentence. It does not give a new proof that Keccak-f[1600] is a
bijection.

**Proposition J8.1.** Suppose the padded input is one rate block, so
`n = 1`, and suppose `d > 0`. Step 6 runs once. If `d ≤ r`, Algorithm
8 returns from step 9 and does not reach step 10, so `f` runs once.
If `r < d ≤ 2r`, step 9 does not return after the first visit, step
10 runs once, and `f` runs twice. The output bits at positions `0`
through `r − 1` are Trunc_r of the state after the absorb. The
output bits at positions `r` through `d − 1` are the first `d − r`
bits of `f` applied to that state.

Step 6 with `n = 1` replaces `S` by `f(0^b ⊕ (P0 || 0^c))`. Call
that state `S1`. Step 8 sets `Z` to Trunc_r(`S1`), so `|Z| = r`.
The test in step 9 is `d ≤ |Z|`. When `d ≤ r` the test succeeds and
the algorithm returns Trunc_d(`Z`). Step 10 is the next numbered
step, and a return does not reach it. That is one call to `f`.

When `d > r` the test fails. The only continuation the algorithm
writes is step 10, which replaces `S` by `f(S1)`. Call that state
`S2`. Control returns to step 8, which appends Trunc_r(`S2`). Now
`|Z| = 2r`. If `d ≤ 2r`, step 9 returns Trunc_d of that longer
string. The first `r` bits were appended before step 10, so they are
bits of `S1`. The bits from position `r` onward were appended after
step 10, so they are bits of `S2`, which is `f(S1)`. That is two
calls to `f`.

The return lengths in Listing J8.7 are `32`, `64`, `64`, and `200`
bytes. Those four fall under this proposition. For SHAKE128 the
rate is `1344` bits, which is `168` bytes. An output of `32` bytes
has `d = 256 ≤ 1344`, so `f` runs once. An output of `168` bytes
has `d = r`, and the test `d ≤ |Z|` still succeeds, so `f` still
runs once. Listing J8.7 does not run `168` as its own spec. An
output of `200` bytes has `d = 1600`. Then
`1344 < 1600 ≤ 2688`, so `f` runs twice. For SHAKE256 the rate is
`1088` bits, which is `136` bytes. An output of `64` bytes has
`d = 512 ≤ 1088`, so `f` runs once. A length of `512` bytes on
SHAKE128 is `d = 4096`. The rate is `168` bytes, and
`4096 / 8 = 512 = 3 · 168 + 8`, so the squeeze needs four blocks and
`f` runs four times. Listing J8.7 does not run that length. §J8.8
says why.

The note under Algorithm 8 says `d` chooses how many of those bits
are returned and does not change the bits themselves. Trunc_d of a
longer prefix is the shorter prefix. SHAKE128 of one message at
`32` bytes is the first `32` bytes of SHAKE128 of that message at
`200` bytes. The listing checks both lengths. It does not check
every length between them.

### J8.3 The suffix, and the first pad byte

**Proposition J8.2.** Let `M` be a whole-byte message, let `r` be
`1344` or `1088`, and let `q > 2` be the byte count Appendix B.2
defines. The byte pad of SHAKE128 or SHAKE256 is

```text
M || 0x1F || 0^(q − 2) || 0x80
```

§6.2 appends the bit string `1111` before the pad. The four bits, in
order, are `1`, `1`, `1`, `1`. Algorithm 9 then appends `1`, then
`j` zeros, then `1`, where `j = (− len(N) − 2) mod r` and
`N = M || 1111`. Appendix B.1 packs eight bits into a byte with bit
`0` of the string in the low bit of the byte. The message ends on a
byte boundary, so the suffix starts at the low bit of the next byte.

That byte's low five bits are the four suffix bits and the pad's
leading `1`: bit `0` through bit `4` are each `1`. When `q > 2` the
final `1` is in a later byte, so this byte still has room for three
zeros. Bits `5`, `6`, and `7` are `0`. The byte is
`1 + 2 + 4 + 8 + 16 = 31`, which is `0x1F`. The final `1` of
`pad10*1` is the last bit of the rate. Both rates are multiples of
`8`, so that bit is bit `7` of the last rate byte, and the lower
seven bits of that byte are zeros from `j`. The byte is `0x80`.
Table 6 writes this case as `M || 0x1F || 0x00 … || 0x80`.

The same packing with the hash suffix `01` is J7's Proposition J7.7.
Bit `0` is `0`, bit `1` is `1`, bit `2` is `1`, and bits `3` through
`7` are `0`. That byte is `0x06`. The two suffixes do not produce
the same first pad byte.

For the empty message, `m = 0`, so `q = r / 8`. SHAKE128 has
`q = 168` and `166` zero bytes. SHAKE256 has `q = 136` and `134`
zero bytes. For “abc”, `m = 3`. SHAKE128 has `q = 165` and `163`
zero bytes. SHAKE256 has `q = 133` and `131` zero bytes. Each of
those `q` values is greater than `2`, so Proposition J8.2 applies.
The empty SHAKE128 block is the block the NIST 0-bit file absorbs:
it begins `1F`, and lane `(0, 4)` of the xored state is
`0x8000000000000000`, which is the byte `0x80` at string index
`167`.

The bit count meets the rate. For the empty SHAKE128 input,
`len(N) = 4`. Then `j = (−4 − 2) mod 1344 = 1338`, the pad has
`1340` bits, and `4 + 1340 = 1344`. For “abc” under SHAKE256,
`len(N) = 28`. Then `j = (−28 − 2) mod 1088 = 1058`, the pad has
`1060` bits, and `28 + 1060 = 1088`. One rate block is the whole
padded input. No second block is absorbed.

**Proposition J8.3.** Under the same packing, `q = 1` produces the
single byte `0x9F`, and `q = 2` produces the two bytes `0x1F` and
`0x80`.

If `q = 1`, the suffix and the pad occupy eight bits. Then
`4 + 1 + j + 1 = 8`, so `j = 2`. The byte is bit `0` through bit
`4` equal to `1`, bits `5` and `6` equal to `0`, and bit `7` equal
to `1`. That is `31 + 128 = 159`, which is `0x9F`. Table 6 writes
`M || 0x9F`. If `q = 2`, sixteen bits are available, so `j = 10`.
The first byte is the five `1` bits and three `0` bits, which is
`0x1F`. The second byte is seven `0` bits and a final `1`, which is
`0x80`. Table 6 writes `M || 0x1F80`. Listing J8.1 computes the three
bytes and the two `q` values the permutation listing uses. It does
not call Keccak-f.

```orange
edition 2026;
module pad {
  spec xof_byte() -> Word[8] { ((((0x01 | 0x02) | 0x04) | 0x08) | 0x10) }

  spec hash_byte() -> Word[8] { (0x02 | 0x04) }

  spec q1_byte() -> Word[8] { (0x1f | 0x80) }

  spec q_empty128() -> Int { 168 - (0 % 168) }

  spec q_abc256() -> Int { 136 - (3 % 136) }

  test "pad bytes" {
    ((xof_byte() == 0x1f) && (hash_byte() == 0x06)) && (q1_byte() == 0x9f)
  }

  test "q" { (q_empty128() == 168) && (q_abc256() == 133) }
}
```

`orangec check` is silent. `orangec eval` prints the five specs.

```text
pad::xof_byte: Word[8] = 0x1f
pad::hash_byte: Word[8] = 0x06
pad::q1_byte: Word[8] = 0x9f
pad::q_empty128: Int = 168
pad::q_abc256: Int = 133
```

`orangec test` runs both tests. Both pass. Standard error of that
run, without `--stats`, is empty. The report is the stdout.

```text
test "pad bytes" ... ok
test "q" ... ok
2 tests: 2 passed, 0 failed
```

The step counts of the same run with `--stats`:

```text
test "pad bytes": 26 steps
test "q": 22 steps
total: 48 of 1048576 steps
```

### J8.4 Two functions, one permutation

The glossary in §2.1 defines domain separation as a partitioning of
the inputs to different application domains so that no input is
assigned to more than one domain. §6.1 says the suffix `01` supports
that separation: it distinguishes inputs that arise from the SHA-3
hash functions from inputs that arise from the XOFs in §6.2. §6.2
says the purposes of the four bits `1111` are discussed in §6.3.

§6.3 defines

```text
RawSHAKE128(J, d) = KECCAK[256](J || 11, d)
RawSHAKE256(J, d) = KECCAK[512](J || 11, d)
```

and then

```text
SHAKE128(M, d) = RawSHAKE128(M || 11, d)
SHAKE256(M, d) = RawSHAKE256(M || 11, d)
```

Substituting gives `J || 11 = M || 11 || 11`, which is `M || 1111`.
That is the same string §6.2 appends. The first `11` is the
separator from the hash functions. The second `11` is the part §6.3
identifies with the Sakura coding scheme, for tree hashing that a
later document might specify. This lesson does not build a tree, and
it does not define RawSHAKE as a second Orange function. The bits
that reach `KECCAK[c]` are the four bits already in Proposition J8.2.

**Proposition J8.4.** Let `M` be any bit string. The strings
`M || 01` and `M || 1111` differ at position `len(M)`. SHA3-256 and
SHAKE256 are `KECCAK[512]` on those two strings: the same
permutation, the same rate `1088`, and different inputs. For the
three bytes `abc`, the first byte of SHA3-256 is `0x3a` and the
first byte of SHAKE256 at `64` bytes is `0x48`.

The bits of `M` occupy positions `0` through `len(M) − 1` in both
concatenations. Position `len(M)` is the first bit of the suffix.
The hash suffix begins with `0`. The SHAKE suffix begins with `1`.
Two strings that differ at one position are not equal. Algorithm 9
appends its pad after `N`, so the pad does not change bit `len(M)`.
The padded inputs still differ there.

SHA3-256 is `KECCAK[512](M || 01, 256)`. SHAKE256 is
`KECCAK[512](M || 1111, d)`. §5.2 gives both the underlying function
`KECCAK-p[1600, 24]` and the rate `1600 − 512 = 1088`. The
permutation is not what changed. The string `N` changed.

If `len(M) < 1088`, position `len(M)` lies in the first rate block.
For a whole-byte message of fewer than `136` bytes, that is this
case. The blocks xored into the zero state differ, so the inputs to
the first `f` differ. §4 says `f` is a permutation, so the full
1600-bit states after that call differ. Trunc_d can still agree on
two different states, because a difference can sit in the bits the
truncation drops. The proposition does not claim that every message
produces different truncated outputs. It claims the inputs differ
for every `M`, and it claims the truncated outputs differ for the
one message the listing runs.

That message is `abc`. J7 pinned SHA3-256 of those three bytes as

```text
3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532
```

`hashlib.shake_256` of the same three bytes, at `64` bytes, begins
`48336660`. The first bytes are `0x3a` and `0x48`. The difference is
at index `0`. Listing J8.7 runs the SHAKE side. The witness that
uses `0x06` at SHAKE256's rate produces the SHA-3 digest in its
first `32` bytes, which is the comparison §J8.7 writes before the
report.

cSHAKE and KMAC, in NIST SP 800-185, add further domain strings on
top of this suffix. They are not §6.2. This lesson does not define
them, does not choose a customization string, and does not build a
MAC.

### J8.5 A caller-chosen length, on this slice

Algorithm 8 takes `d` as an integer. The return type of an Orange
spec is fixed in each instance. The form a reader writes for the
integer in the standard is Listing J8.2.

```orange
edition 2026;
module intd {
  spec shake(d: Int) -> Word[8]^d { [0; d] }
}
```

`orangec check` rejects it. The diagnostic is `ORC0237`. `d` is an
`Int`. A size is built from integer literals and size parameters.
The same diagnostic is what `eval` and `test` print, and neither
prints a value.

```text
error[ORC0237]: a size may use only integer literals and size parameters
 --> <stdin>:3:33
  |
3 |   spec shake(d: Int) -> Word[8]^d { [0; d] }
  |                                 ^ this is neither
  = note: a size is fixed in each instance of its function: it is built from integer literals and the function's size parameters with `+`, `-`, `*`, `/`, `%`, and parentheses
```

A size parameter is a different form. `n in a..b` means each integer
from `a` up to, but not including, `b`. The return length `Word[8]^d`
is legal when every instance's length lies from `1` through `65536`,
and when the function has at most `256` instances. Listing J8.3
includes the length `0`. Algorithm 8 allows a nonnegative `d`, and
`0` is nonnegative. An array on this slice does not have length `0`.

```orange
edition 2026;
module zerolen {
  spec shake[d in 0..256]() -> Word[8]^d { [0; d] }
}
```

`orangec check` rejects it. The diagnostic is `ORC0221`. The note
names the instance `shake[0]`.

```text
error[ORC0221]: this array length is 0, but an array has 1 through 65536 elements
 --> <stdin>:3:40
  |
3 |   spec shake[d in 0..256]() -> Word[8]^d { [0; d] }
  |                                        ^ unsupported array length in this instance
  = note: a length written with sizes is computed in each instance of its function, and every instance's lengths are from 1 through 65536
  = note: in the instance `shake[0]`, the first of `shake` in error: a sized function is checked once for each value of its sizes
```

Listing J8.4 asks for `257` instances. The range `1..258` is the
integers `1` through `257`.

```orange
edition 2026;
module toomany {
  spec shake[d in 1..258]() -> Word[8]^d { [0; d] }
}
```

`orangec check` rejects it. The diagnostic is `ORC0238`.

```text
error[ORC0238]: `shake` has 257 instances, but a function has at most 256
 --> <stdin>:3:14
  |
3 |   spec shake[d in 1..258]() -> Word[8]^d { [0; d] }
  |              ^^^^^^^^^^^ too many instances
  = note: a size parameter `n in a..b` takes each value from a up to, but not including, b, with a < b <= 65536, and a function has at most 256 instances
```

The longest range that stays inside the cap and stays off the zero
length is `1..257`, which is the `256` lengths `1` through `256`.
Listing J8.5 checks that range on a function that returns `d` zero
bytes, and it evaluates the instance of length `1`. It does not run
SHAKE at each of those lengths. A function whose body is Keccak-f,
instantiated at every length, is not this listing.

```orange
edition 2026;
module span {
  spec unit[d in 1..257]() -> Word[8]^d { [0; d] }

  spec one() -> Word[8]^1 { unit[1]() }

  test "the longest size range has one instance at length 1" { one() == hex"00" }
}
```

`orangec check` is silent.

```text
span::one: Word[8]^1 = [0x00]
```

`orangec test` runs the one test. It passes. The report is the stdout.

```text
test "the longest size range has one instance at length 1" ... ok
1 test: 1 passed, 0 failed
```

The step count of the same run with `--stats`:

```text
test "the longest size range has one instance at length 1": 6 steps
total: 6 of 1048576 steps
```

Inside a range the compiler does accept, the loop bound may be the
size. Listing J8.6 keeps the first `d` bytes of a four-byte string,
for `d` from `1` through `4`. The test asks for `d = 3` on the first
four bytes of SHAKE128 of the empty message. Those four bytes are
`7f9c2ba4`. The three-byte prefix is `7f9c2b`. This is Trunc_d on a
string that is already in hand. It is not a call to the permutation,
and it is not Algorithm 8 at a length the caller chooses at runtime.

```orange
edition 2026;
module outlen {
  spec take[d in 1..5](s: Word[8]^4) -> Word[8]^d {
    for i in 0..d with z: Word[8]^d = [0; d] { z with [i] = s[i] }
  }

  spec three() -> Word[8]^3 { take[3](hex"7f9c2ba4") }

  test "three bytes of the empty SHAKE128 prefix" { three() == hex"7f9c2b" }
}
```

`orangec check` is silent. `orangec eval` prints the three-byte prefix.

```text
outlen::three: Word[8]^3 = [0x7f, 0x9c, 0x2b]
```

`orangec test` runs the one test. It passes. The report is the stdout.

```text
test "three bytes of the empty SHAKE128 prefix" ... ok
1 test: 1 passed, 0 failed
```

The step count of the same run with `--stats`:

```text
test "three bytes of the empty SHAKE128 prefix": 29 steps
total: 29 of 1048576 steps
```

The SHAKE specs in Listing J8.7 therefore name four fixed return
lengths: `Word[8]^32`, `Word[8]^64`, and `Word[8]^200`, plus the
`168`-byte rate used on the way to the `200`-byte string. Each
length is one spec. A single spec whose return length is the
standard's integer `d` is the program Listing J8.2 rejects.

### J8.6 The four lengths this module runs

Listing J8.7 is Listing J7.7's permutation, with the same sheet
parameter and the same names, followed by the sponge. The harness
feeds one module on standard input, so the permutation is copied
into this file. It is not re-derived. No test in this module calls
the `Sheet8` instance. The step total below does not include a
Keccak-f[200] evaluation. J7 already ran that width.

J7's Keccak module used `991668` of `1048576` evaluator steps and
had `56908` left. One Keccak-f[1600] evaluation in that lesson cost
`101725` steps for the zero-state lane alone. A SHAKE squeeze does
not fit in the steps J7 had left. This lesson uses its own module.
The eight tests below use `919661` of the same budget `1048576`.
The `128915` steps that remain are fewer than the `204158` steps of
the 200-byte test, so a second two-permutation output was not added
to this module. They are more than the `102093` steps of the
32-byte test. The 512-byte empty output printed by the NIST file
and by the XKCP file needs four permutations, because
`512 = 3 · 168 + 8`. Four permutations were not run.

The absorb follows J7's lane formula. String index `i` is lane
`5y + x`, so `x = i mod 5` and `y = i / 5`. SHAKE128's rate is
`168` bytes, which is `21` lanes. SHAKE256's rate is `136` bytes,
which is `17` lanes. The lanes past the rate stay `0` on an absorb
into a zero state. That is the capacity. `as little` is the packing
Appendix B.1 specifies. The repository file
`algorithms/sha3/sha3.or` is a different transcription, and its
SHAKE specs stop at one rate. Listing J8.7 does not `use` it.

`shake128_empty_32` is one permutation and the first `32` bytes.
`shake256_empty_64` and `shake256_abc_64` are one permutation and
the first `64` bytes. `shake128_empty_200` is Proposition J8.1 at
`d = 1600` bits: `rate168` of `S1`, then `prefix32` of `f(S1)`.
The `200` bytes are not one serialization of one state. The first
`168` are `S1`. The last `32` are `S2`.

The empty SHAKE128 string, through byte `199`, is the prefix of the
512-byte output in the NIST file `shake128_msg0.pdf` and of the
`Len = 0` line in the XKCP SHAKE128 file. Both of those outputs are
`512` bytes, and both match `hashlib.shake_128` of the empty message
at that length. The listing checks `200` of those bytes, including
the `32` that begin at index `168`. It does not check bytes `200`
through `511`. The empty SHAKE256 string, at `64` bytes, is the
prefix of the same two sources for SHAKE256. `hashlib.shake_256` of
the empty message prints the same `64` bytes.

`hashlib.shake_256` of the three bytes `abc`, at `64` bytes, prints
the third digest. FIPS 202 does not print it. On 2026-10-17 the
URLs `shake128_msg24.pdf`, `shake256_msg24.pdf`, `shake128_abc.pdf`,
and `shake256_abc.pdf` in the NIST examples directory each answered
`404` with an empty body. The XKCP line `Len = 24` has message bytes
`1F877C`, which are not the ASCII bytes of “abc”. This lesson does
not attribute the “abc” digest to those files.

The four failing tests are the witnesses in §J8.7. Each expected
string is the digest the corresponding passing test demands. The
predictions are stated before the report. The passing values are
printed here, before that section, because the report is one run of
every test in the module.

θ, ρ, π, χ, ι, `rc(t)`, and `RC` are J7's Assumption J7.4, mapped
there to `theta`, `rho`, `pi`, `chi`, `iota`, `rc_bits`, and
`round_constants`. This lesson does not re-prove them. The sponge
names in Listing J8.7 are ASCII. The mapping is:

```text
KECCAK-f[1600], f (§3.4, Algorithm 8)               keccak_f
Trunc_r, r = 1344 (Algorithm 8)                     rate168
Trunc_d, d = 256 (Algorithm 8)                      prefix32
Trunc_d, d = 512 (Algorithm 8)                      prefix64
0x1F || 0x00 … || 0x80, q = 168 (Table 6)           empty128
0x1F || 0x00 … || 0x80, q = 136 (Table 6)           empty136
"abc" || 0x1F || 0x00 … || 0x80, q = 133 (Table 6)  abc256
SHAKE128(M, 256), M empty (§6.2)                    shake128_empty_32
SHAKE256(M, 512), M empty (§6.2)                    shake256_empty_64
SHAKE256(M, 512), M = "abc" (§6.2)                  shake256_abc_64
SHAKE128(M, 1600), M empty (§6.2)                   shake128_empty_200
```

```orange
edition 2026;
module shake {
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

  spec absorb168(block: Word[8]^168) -> State64 {
    let lanes: Word[64]^21 = block as little Word[64]^21;
    for i in 0..21 with s: State64 = [[0; 5]; 5] {
      s with [i % 5][i / 5] = lanes[i]
    }
  }

  spec absorb136(block: Word[8]^136) -> State64 {
    let lanes: Word[64]^17 = block as little Word[64]^17;
    for i in 0..17 with s: State64 = [[0; 5]; 5] {
      s with [i % 5][i / 5] = lanes[i]
    }
  }

  spec prefix32(a: State64) -> Word[8]^32 {
    [a[0][0], a[1][0], a[2][0], a[3][0]] as little Word[8]^32
  }

  spec prefix40(a: State64) -> Word[8]^40 {
    let lanes: Word[64]^5 = for i in 0..5 with v: Word[64]^5 = [0; 5] {
      v with [i] = a[i % 5][i / 5]
    };
    lanes as little Word[8]^40
  }

  spec prefix64(a: State64) -> Word[8]^64 {
    let lanes: Word[64]^8 = for i in 0..8 with v: Word[64]^8 = [0; 8] {
      v with [i] = a[i % 5][i / 5]
    };
    lanes as little Word[8]^64
  }

  spec rate168(a: State64) -> Word[8]^168 {
    let lanes: Word[64]^21 = for i in 0..21 with v: Word[64]^21 = [0; 21] {
      v with [i] = a[i % 5][i / 5]
    };
    lanes as little Word[8]^168
  }

  spec state200(a: State64) -> Word[8]^200 {
    let lanes: Word[64]^25 = for i in 0..25 with v: Word[64]^25 = [0; 25] {
      v with [i] = a[i % 5][i / 5]
    };
    lanes as little Word[8]^200
  }

  spec empty128() -> Word[8]^168 { (hex"1f" ++ [0; 166]) ++ hex"80" }

  spec empty136() -> Word[8]^136 { (hex"1f" ++ [0; 134]) ++ hex"80" }

  spec abc256() -> Word[8]^136 { ("abc" ++ (hex"1f" ++ [0; 131])) ++ hex"80" }

  spec abc_domain06() -> Word[8]^136 { ("abc" ++ (hex"06" ++ [0; 131])) ++ hex"80" }

  spec shake128_empty_32() -> Word[8]^32 { prefix32(keccak_f(absorb168(empty128()))) }

  spec shake256_empty_64() -> Word[8]^64 { prefix64(keccak_f(absorb136(empty136()))) }

  spec shake256_abc_64() -> Word[8]^64 { prefix64(keccak_f(absorb136(abc256()))) }

  spec shake128_empty_200() -> Word[8]^200 {
    let s1: State64 = keccak_f(absorb168(empty128()));
    let s2: State64 = keccak_f(s1);
    rate168(s1) ++ prefix32(s2)
  }

  spec domain06() -> Word[8]^64 { prefix64(keccak_f(absorb136(abc_domain06()))) }

  spec wrong_rate() -> Word[8]^32 { prefix32(keccak_f(absorb136(empty136()))) }

  spec off_by_one() -> Word[8]^32 {
    let raw: Word[8]^40 = prefix40(keccak_f(absorb168(empty128())));
    for i in 0..32 with z: Word[8]^32 = [0; 32] { z with [i] = raw[i + 1] }
  }

  spec noreperm() -> Word[8]^200 { state200(keccak_f(absorb168(empty128()))) }

  spec domain06_byte() -> Word[8] { domain06()[0] }

  spec wrong_rate_byte() -> Word[8] { wrong_rate()[0] }

  spec off_byte() -> Word[8] { off_by_one()[0] }

  spec noreperm_byte() -> Word[8] { noreperm()[168] }

  spec second_byte() -> Word[8] { shake128_empty_200()[168] }

  spec correct128_byte() -> Word[8] { shake128_empty_32()[0] }

  spec correct256_byte() -> Word[8] { shake256_abc_64()[0] }

  test "SHAKE128 of the empty message, 32 bytes" {
    shake128_empty_32() == hex"7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26"
  }

  test "SHAKE256 of the empty message, 64 bytes" {
    shake256_empty_64() == hex"46b9dd2b0ba88d13233b3feb743eeb243fcd52ea62b81b82b50c27646ed5762fd75dc4ddd8c0f200cb05019d67b592f6fc821c49479ab48640292eacb3b7c4be"
  }

  test "SHAKE256 of abc, 64 bytes" {
    shake256_abc_64() == hex"483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739d5a15bef186a5386c75744c0527e1faa9f8726e462a12a4feb06bd8801e751e4"
  }

  test "SHAKE128 of the empty message, 200 bytes" {
    shake128_empty_200() == hex"7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef263cb1eea988004b93103cfb0aeefd2a686e01fa4a58e8a3639ca8a1e3f9ae57e235b8cc873c23dc62b8d260169afa2f75ab916a58d974918835d25e6a435085b2badfd6dfaac359a5efbb7bcc4b59d538df9a04302e10c8bc1cbf1a0b3a5120ea17cda7cfad765f5623474d368ccca8af0007cd9f5e4c849f167a580b14aabdefaee7eef47cb0fca9767be1fda69419dfb927e9df07348b196691abaeb580b32def58538b8d23f877"
  }

  test "domain byte 06" {
    domain06() == hex"483366601360a8771c6863080cc4114d8db44530f8f1e1ee4f94ea37e78b5739d5a15bef186a5386c75744c0527e1faa9f8726e462a12a4feb06bd8801e751e4"
  }

  test "SHAKE128 at rate 136" {
    wrong_rate() == hex"7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26"
  }

  test "truncation off by one" {
    off_by_one() == hex"7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef26"
  }

  test "squeeze without a second permutation" {
    noreperm() == hex"7f9c2ba4e88f827d616045507605853ed73b8093f6efbc88eb1a6eacfa66ef263cb1eea988004b93103cfb0aeefd2a686e01fa4a58e8a3639ca8a1e3f9ae57e235b8cc873c23dc62b8d260169afa2f75ab916a58d974918835d25e6a435085b2badfd6dfaac359a5efbb7bcc4b59d538df9a04302e10c8bc1cbf1a0b3a5120ea17cda7cfad765f5623474d368ccca8af0007cd9f5e4c849f167a580b14aabdefaee7eef47cb0fca9767be1fda69419dfb927e9df07348b196691abaeb580b32def58538b8d23f877"
  }
}
```

`orangec check` is silent.

```text
shake::shake128_empty_32: Word[8]^32 = [0x7f, 0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e, 0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26]
shake::shake256_empty_64: Word[8]^64 = [0x46, 0xb9, 0xdd, 0x2b, 0x0b, 0xa8, 0x8d, 0x13, 0x23, 0x3b, 0x3f, 0xeb, 0x74, 0x3e, 0xeb, 0x24, 0x3f, 0xcd, 0x52, 0xea, 0x62, 0xb8, 0x1b, 0x82, 0xb5, 0x0c, 0x27, 0x64, 0x6e, 0xd5, 0x76, 0x2f, 0xd7, 0x5d, 0xc4, 0xdd, 0xd8, 0xc0, 0xf2, 0x00, 0xcb, 0x05, 0x01, 0x9d, 0x67, 0xb5, 0x92, 0xf6, 0xfc, 0x82, 0x1c, 0x49, 0x47, 0x9a, 0xb4, 0x86, 0x40, 0x29, 0x2e, 0xac, 0xb3, 0xb7, 0xc4, 0xbe]
shake::shake256_abc_64: Word[8]^64 = [0x48, 0x33, 0x66, 0x60, 0x13, 0x60, 0xa8, 0x77, 0x1c, 0x68, 0x63, 0x08, 0x0c, 0xc4, 0x11, 0x4d, 0x8d, 0xb4, 0x45, 0x30, 0xf8, 0xf1, 0xe1, 0xee, 0x4f, 0x94, 0xea, 0x37, 0xe7, 0x8b, 0x57, 0x39, 0xd5, 0xa1, 0x5b, 0xef, 0x18, 0x6a, 0x53, 0x86, 0xc7, 0x57, 0x44, 0xc0, 0x52, 0x7e, 0x1f, 0xaa, 0x9f, 0x87, 0x26, 0xe4, 0x62, 0xa1, 0x2a, 0x4f, 0xeb, 0x06, 0xbd, 0x88, 0x01, 0xe7, 0x51, 0xe4]
shake::shake128_empty_200: Word[8]^200 = [0x7f, 0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e, 0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26, 0x3c, 0xb1, 0xee, 0xa9, 0x88, 0x00, 0x4b, 0x93, 0x10, 0x3c, 0xfb, 0x0a, 0xee, 0xfd, 0x2a, 0x68, 0x6e, 0x01, 0xfa, 0x4a, 0x58, 0xe8, 0xa3, 0x63, 0x9c, 0xa8, 0xa1, 0xe3, 0xf9, 0xae, 0x57, 0xe2, 0x35, 0xb8, 0xcc, 0x87, 0x3c, 0x23, 0xdc, 0x62, 0xb8, 0xd2, 0x60, 0x16, 0x9a, 0xfa, 0x2f, 0x75, 0xab, 0x91, 0x6a, 0x58, 0xd9, 0x74, 0x91, 0x88, 0x35, 0xd2, 0x5e, 0x6a, 0x43, 0x50, 0x85, 0xb2, 0xba, 0xdf, 0xd6, 0xdf, 0xaa, 0xc3, 0x59, 0xa5, 0xef, 0xbb, 0x7b, 0xcc, 0x4b, 0x59, 0xd5, 0x38, 0xdf, 0x9a, 0x04, 0x30, 0x2e, 0x10, 0xc8, 0xbc, 0x1c, 0xbf, 0x1a, 0x0b, 0x3a, 0x51, 0x20, 0xea, 0x17, 0xcd, 0xa7, 0xcf, 0xad, 0x76, 0x5f, 0x56, 0x23, 0x47, 0x4d, 0x36, 0x8c, 0xcc, 0xa8, 0xaf, 0x00, 0x07, 0xcd, 0x9f, 0x5e, 0x4c, 0x84, 0x9f, 0x16, 0x7a, 0x58, 0x0b, 0x14, 0xaa, 0xbd, 0xef, 0xae, 0xe7, 0xee, 0xf4, 0x7c, 0xb0, 0xfc, 0xa9, 0x76, 0x7b, 0xe1, 0xfd, 0xa6, 0x94, 0x19, 0xdf, 0xb9, 0x27, 0xe9, 0xdf, 0x07, 0x34, 0x8b, 0x19, 0x66, 0x91, 0xab, 0xae, 0xb5, 0x80, 0xb3, 0x2d, 0xef, 0x58, 0x53, 0x8b, 0x8d, 0x23, 0xf8, 0x77]
```

### J8.7 Four witnesses, predicted before the report

Each prediction is the claim the failing test then compares with a
correct SHAKE string. The differing bytes are results of the
reference evaluator. They are not a hand expansion of twenty-four
rounds. The index of the first difference, for the truncation and
for the missing permutation, is an argument from the formulas. The
bytes at that index are what `orangec eval` prints. They are written
here, and the eval lines repeat them, before the report.

**Domain byte `06`.** The block is “abc”, `0x06`, `131` zero bytes,
and `0x80`. That is SHA3-256's pad at rate `136`, which is also
SHAKE256's rate. Proposition J8.4 says the suffix is the difference.
The first `32` bytes of the 64-byte result are SHA3-256 of “abc”,
so the first byte is `0x3a`. The correct SHAKE256 first byte is
`0x48`. The first difference is at index `0`. The bytes after index
`31` are a further squeeze of that same state. SHA3-256 does not
define them. The test compares all `64` bytes with SHAKE256.

**SHAKE128 at rate `136`.** The empty message, the suffix byte
`0x1F`, and a rate of `136` bytes are SHAKE256's empty absorb, not
SHAKE128's. The rate decides where `0x80` sits: byte `135` at
SHAKE256's rate, byte `167` at SHAKE128's rate. The first byte of
this result is `0x46`, which is also the first byte of SHAKE256 of
the empty message. The correct SHAKE128 first byte is `0x7f`. The
first difference is at index `0`.

**Truncation off by one.** Trunc_d keeps indices `0` through `31`
of the SHAKE128 empty output. The wrong spec keeps indices `1`
through `32`. Index `0` of the correct string is `0x7f`. Index `1`
is `0x9c`. The strings differ at index `0`, and the left byte is
`0x9c`. The byte that falls off the end of the correct 32-byte
string, index `32`, is `0x3c`. It is the last byte of the wrong
string. One permutation still produces every one of those bytes,
because `33 ≤ 168`.

**Proposition J8.5.** Let `S1` be the state after the one absorb of
the empty SHAKE128 block, and let the correct 200-byte string be
Trunc_r(`S1`) followed by the first `32` bytes of `f(S1)`. A
serialization of the first `200` bytes of `S1` agrees with that
string at every index from `0` through `167`. At index `168` the
serialization is the first capacity byte of `S1`, and the correct
string is the first byte of `f(S1)`.

Both strings begin with Trunc_r(`S1`). The rate is `168` bytes, so
that common prefix is indices `0` through `167`. Proposition J8.1
puts index `168` of the correct string in `f(S1)`, because
`d = 1600` is past one rate and not past two. The serialization
does not call `f` again. Its next byte is the byte of `S1` at
string index `168`, which is the first byte of the capacity: the
rate ended at index `167`. Those are different expressions. The
evaluator prints `0x4c` for the capacity byte and `0x76` for the
first byte of `f(S1)`. The first difference is at index `168`.
This proposition does not expand the twenty-four rounds by hand.
The index is the argument. The two bytes are the evaluation.

`orangec eval` prints those bytes.

```text
shake::domain06_byte: Word[8] = 0x3a
shake::wrong_rate_byte: Word[8] = 0x46
shake::off_byte: Word[8] = 0x9c
shake::noreperm_byte: Word[8] = 0x4c
shake::second_byte: Word[8] = 0x76
shake::correct128_byte: Word[8] = 0x7f
shake::correct256_byte: Word[8] = 0x48
```

`orangec test` then runs all eight tests. Four pass. Four fail.
Standard error is empty. Status is `1`. The report is the whole
stdout of that one run.

```text
test "SHAKE128 of the empty message, 32 bytes" ... ok
test "SHAKE256 of the empty message, 64 bytes" ... ok
test "SHAKE256 of abc, 64 bytes" ... ok
test "SHAKE128 of the empty message, 200 bytes" ... ok
test "domain byte 06" ... FAILED
    left:  [0x3a, 0x98, 0x5d, 0xa7, 0x4f, 0xe2, 0x25, 0xb2, 0x04, 0x5c, 0x17, 0x2d, 0x6b, 0xd3, 0x90, 0xbd, 0x85, 0x5f, 0x08, 0x6e, 0x3e, 0x9d, 0x52, 0x5b, 0x46, 0xbf, 0xe2, 0x45, 0x11, 0x43, 0x15, 0x32, 0xd1, 0x36, 0xf6, 0x22, 0xfb, 0x92, 0x10, 0xf8, 0x27, 0x3e, 0x60, 0xd6, 0xaa, 0xc8, 0x97, 0x20, 0xf7, 0xb1, 0x3f, 0x62, 0x61, 0xc5, 0xf2, 0x31, 0x9c, 0xdf, 0x04, 0xf2, 0xf3, 0x74, 0xdf, 0x8f]
    right: [0x48, 0x33, 0x66, 0x60, 0x13, 0x60, 0xa8, 0x77, 0x1c, 0x68, 0x63, 0x08, 0x0c, 0xc4, 0x11, 0x4d, 0x8d, 0xb4, 0x45, 0x30, 0xf8, 0xf1, 0xe1, 0xee, 0x4f, 0x94, 0xea, 0x37, 0xe7, 0x8b, 0x57, 0x39, 0xd5, 0xa1, 0x5b, 0xef, 0x18, 0x6a, 0x53, 0x86, 0xc7, 0x57, 0x44, 0xc0, 0x52, 0x7e, 0x1f, 0xaa, 0x9f, 0x87, 0x26, 0xe4, 0x62, 0xa1, 0x2a, 0x4f, 0xeb, 0x06, 0xbd, 0x88, 0x01, 0xe7, 0x51, 0xe4]
    first difference at [0]
test "SHAKE128 at rate 136" ... FAILED
    left:  [0x46, 0xb9, 0xdd, 0x2b, 0x0b, 0xa8, 0x8d, 0x13, 0x23, 0x3b, 0x3f, 0xeb, 0x74, 0x3e, 0xeb, 0x24, 0x3f, 0xcd, 0x52, 0xea, 0x62, 0xb8, 0x1b, 0x82, 0xb5, 0x0c, 0x27, 0x64, 0x6e, 0xd5, 0x76, 0x2f]
    right: [0x7f, 0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e, 0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26]
    first difference at [0]
test "truncation off by one" ... FAILED
    left:  [0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e, 0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26, 0x3c]
    right: [0x7f, 0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e, 0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26]
    first difference at [0]
test "squeeze without a second permutation" ... FAILED
    left:  [0x7f, 0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e, 0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26, 0x3c, 0xb1, 0xee, 0xa9, 0x88, 0x00, 0x4b, 0x93, 0x10, 0x3c, 0xfb, 0x0a, 0xee, 0xfd, 0x2a, 0x68, 0x6e, 0x01, 0xfa, 0x4a, 0x58, 0xe8, 0xa3, 0x63, 0x9c, 0xa8, 0xa1, 0xe3, 0xf9, 0xae, 0x57, 0xe2, 0x35, 0xb8, 0xcc, 0x87, 0x3c, 0x23, 0xdc, 0x62, 0xb8, 0xd2, 0x60, 0x16, 0x9a, 0xfa, 0x2f, 0x75, 0xab, 0x91, 0x6a, 0x58, 0xd9, 0x74, 0x91, 0x88, 0x35, 0xd2, 0x5e, 0x6a, 0x43, 0x50, 0x85, 0xb2, 0xba, 0xdf, 0xd6, 0xdf, 0xaa, 0xc3, 0x59, 0xa5, 0xef, 0xbb, 0x7b, 0xcc, 0x4b, 0x59, 0xd5, 0x38, 0xdf, 0x9a, 0x04, 0x30, 0x2e, 0x10, 0xc8, 0xbc, 0x1c, 0xbf, 0x1a, 0x0b, 0x3a, 0x51, 0x20, 0xea, 0x17, 0xcd, 0xa7, 0xcf, 0xad, 0x76, 0x5f, 0x56, 0x23, 0x47, 0x4d, 0x36, 0x8c, 0xcc, 0xa8, 0xaf, 0x00, 0x07, 0xcd, 0x9f, 0x5e, 0x4c, 0x84, 0x9f, 0x16, 0x7a, 0x58, 0x0b, 0x14, 0xaa, 0xbd, 0xef, 0xae, 0xe7, 0xee, 0xf4, 0x7c, 0xb0, 0xfc, 0xa9, 0x4c, 0xca, 0xae, 0xba, 0x77, 0x4e, 0xc2, 0x0c, 0xff, 0x6a, 0x94, 0x85, 0xa9, 0x7b, 0xfc, 0x65, 0xaa, 0x93, 0xaa, 0x4f, 0xc9, 0x58, 0xd1, 0xed, 0xb5, 0x27, 0xc0, 0x2e, 0x3a, 0xe5, 0x7b, 0xcc]
    right: [0x7f, 0x9c, 0x2b, 0xa4, 0xe8, 0x8f, 0x82, 0x7d, 0x61, 0x60, 0x45, 0x50, 0x76, 0x05, 0x85, 0x3e, 0xd7, 0x3b, 0x80, 0x93, 0xf6, 0xef, 0xbc, 0x88, 0xeb, 0x1a, 0x6e, 0xac, 0xfa, 0x66, 0xef, 0x26, 0x3c, 0xb1, 0xee, 0xa9, 0x88, 0x00, 0x4b, 0x93, 0x10, 0x3c, 0xfb, 0x0a, 0xee, 0xfd, 0x2a, 0x68, 0x6e, 0x01, 0xfa, 0x4a, 0x58, 0xe8, 0xa3, 0x63, 0x9c, 0xa8, 0xa1, 0xe3, 0xf9, 0xae, 0x57, 0xe2, 0x35, 0xb8, 0xcc, 0x87, 0x3c, 0x23, 0xdc, 0x62, 0xb8, 0xd2, 0x60, 0x16, 0x9a, 0xfa, 0x2f, 0x75, 0xab, 0x91, 0x6a, 0x58, 0xd9, 0x74, 0x91, 0x88, 0x35, 0xd2, 0x5e, 0x6a, 0x43, 0x50, 0x85, 0xb2, 0xba, 0xdf, 0xd6, 0xdf, 0xaa, 0xc3, 0x59, 0xa5, 0xef, 0xbb, 0x7b, 0xcc, 0x4b, 0x59, 0xd5, 0x38, 0xdf, 0x9a, 0x04, 0x30, 0x2e, 0x10, 0xc8, 0xbc, 0x1c, 0xbf, 0x1a, 0x0b, 0x3a, 0x51, 0x20, 0xea, 0x17, 0xcd, 0xa7, 0xcf, 0xad, 0x76, 0x5f, 0x56, 0x23, 0x47, 0x4d, 0x36, 0x8c, 0xcc, 0xa8, 0xaf, 0x00, 0x07, 0xcd, 0x9f, 0x5e, 0x4c, 0x84, 0x9f, 0x16, 0x7a, 0x58, 0x0b, 0x14, 0xaa, 0xbd, 0xef, 0xae, 0xe7, 0xee, 0xf4, 0x7c, 0xb0, 0xfc, 0xa9, 0x76, 0x7b, 0xe1, 0xfd, 0xa6, 0x94, 0x19, 0xdf, 0xb9, 0x27, 0xe9, 0xdf, 0x07, 0x34, 0x8b, 0x19, 0x66, 0x91, 0xab, 0xae, 0xb5, 0x80, 0xb3, 0x2d, 0xef, 0x58, 0x53, 0x8b, 0x8d, 0x23, 0xf8, 0x77]
    first difference at [168]
8 tests: 4 passed, 4 failed
```

The step counts for the same run:

```text
test "SHAKE128 of the empty message, 32 bytes": 102093 steps
test "SHAKE256 of the empty message, 64 bytes": 102139 steps
test "SHAKE256 of abc, 64 bytes": 102143 steps
test "SHAKE128 of the empty message, 200 bytes": 204158 steps
test "domain byte 06": 102143 steps
test "SHAKE128 at rate 136": 102029 steps
test "truncation off by one": 102478 steps
test "squeeze without a second permutation": 102478 steps
total: 919661 of 1048576 steps
```

A failing comparison is the result the test was written to produce.
Repairing a failure by editing the expected digest would make the
test pass and would leave the transcription wrong. The repair is
the formula in the proposition: the suffix `1111` packed as `0x1F`,
the rate that belongs to that function, Trunc_d from index `0`, and
step 10 between output blocks.

### J8.8 What this slice does not denote

The permutation is checked here only by being called at `w = 64`.
The `Sheet8` parameter is present because the copied text is J7's.
No test instantiates it. `w = 1`, `w = 2`, and `w = 4` are still not
words. Pull request 248, the open compiler request that would check
one word parameter at every width, is not in this tree. This lesson
does not schedule it, and it does not repeat Listing J7.1.

The round loop is Keccak-f at `w = 64`. It is not Keccak-p at a
caller-chosen `nr`. SHA3-224, SHA3-384, and SHA3-512 are not run.
SHAKE128 of “abc” is not run. A message that absorbs two rate blocks
is not run. The rows `q = 1` and `q = 2` of Table 6 are not run as
permutations. A message whose bit length is not a multiple of 8 is
not run. An output of `512` bytes is not run. Bytes `200` through
`511` of the pinned 512-byte empty outputs are not compared.
Keccak-f[200] is not used as the sponge's `f`.

`as little` is implemented on the S3u binary and is specified by
OEP-0017, which this lesson does not accept. A computed rotation
amount is specified by OEP-0021, which this lesson does not accept.
A type parameter over a listed set of types is specified by
OEP-0018, which this lesson does not accept. A size parameter is
specified by OEP-0016, which this lesson does not accept. A `test`
block is specified by OEP-0020, which this lesson does not accept.
An array of arrays is specified by OEP-0025, which this lesson does
not accept.

The passing tests are two Bool values in Listing J8.1, one in
Listing J8.5, one in Listing J8.6, and four in Listing J8.7, on the
inputs those tests write. They do not establish a cryptographic
security claim.
Do not call that Match verified.
The step counts are evaluator steps. The `919661` steps of the
`shake` module, the `48` steps of the `pad` module, the `6` steps
of the `span` module, and the `29` steps of the `outlen` module are
each inside the default budget `1048576`. A larger budget is not
evidence about speed.

### J8.9 Exercises

**Exercise J8.1.** For SHAKE128 and SHAKE256, state `c`, `r` in
bits, and `r` in bytes. For the empty message and for “abc”, state
`q` and the number of zero bytes in the pad. Which bytes are the
suffix together with the pad's leading `1`, and the final `1`, when
`q > 2`?

**Exercise J8.2.** SHAKE128 of a message that absorbs as one block
is evaluated at `32` bytes, at `168` bytes, at `200` bytes, and at
`512` bytes. How many times does `f` run in each case? Which
proposition is that count? Which of those four lengths does Listing
J8.7 run?

**Exercise J8.3.** From the suffix bits and Algorithm 9, derive the
first pad byte of SHAKE, the first pad byte of SHA-3, and the single
byte Table 6 writes for an XOF with `q = 1`. Why does `q = 2` still
use `0x1F` and `0x80` as two bytes?

**Exercise J8.4.** Why is SHA3-256(“abc”) not the first `32` bytes
of SHAKE256(“abc”, `512`)? Name the byte index of the first
difference and both bytes. What does a Match on one of those strings
not establish about the other, and about every other message?

**Exercise J8.5.** Before looking at the report, state the first
differing index, the left byte, and the right byte for each of the
four witnesses: the pad byte `0x06`, the rate `136` used for
SHAKE128, the truncation that starts at index `1`, and the second
block read from the capacity.

**Exercise J8.6.** Why does Listing J8.2 reject `d: Int`? What does
Listing J8.3 reject, and what does Listing J8.4 reject? How many
instances does `d in 1..257` have, and which length does Listing
J8.5 actually evaluate?

**Exercise J8.7.** Why are the `200` bytes of SHAKE128 of the empty
message not lanes `(0, 0)` through `(4, 4)` of one state? At which
index must a capacity-byte reading first be allowed to differ, and
what are the two bytes the evaluator prints there?

**Exercise J8.8.** J7's Keccak module had `56908` steps left out of
`1048576`. Why was SHAKE not added to that module? This module has
`128915` steps left after `919661`. Why was a second 200-byte output
not added, and why were bytes `200` through `511` of the NIST empty
output not compared? Where are cSHAKE, KMAC, and HMAC, and what does
this lesson not build?

## Worked answers

**J8.1.** SHAKE128 has `c = 256`, `r = 1344` bits, and `168` bytes.
SHAKE256 has `c = 512`, `r = 1088` bits, and `136` bytes. The empty
message has `q = 168` and `166` zero bytes under SHAKE128, and
`q = 136` and `134` zero bytes under SHAKE256. “abc” has `q = 165`
and `163` zero bytes under SHAKE128, and `q = 133` and `131` zero
bytes under SHAKE256. The suffix and the pad's leading `1` are the
byte `0x1F`. The final `1` is the byte `0x80`. Proposition J8.2.

**J8.2.** At `32` bytes, `d ≤ r`, so `f` runs once. At `168` bytes,
`d = r`, so step 9 still returns and `f` runs once. At `200` bytes,
`r < d ≤ 2r`, so `f` runs twice. At `512` bytes,
`512 = 3 · 168 + 8`, so the squeeze has four blocks and `f` runs
four times. Proposition J8.1, extended by the same step-9 test once
for each additional block. Listing J8.7 runs `32` and `200`. It does
not run `168` as its own spec, and it does not run `512`.

**J8.3.** SHAKE's first five bits are `1`, so the byte is
`1 + 2 + 4 + 8 + 16 = 31 = 0x1F` when the next three bits are `0`.
SHA-3's first three bits are `0`, `1`, `1`, so the byte is
`2 + 4 = 6 = 0x06`. For `q = 1`, `j = 2`, and bit `7` is also `1`,
so the byte is `0x1F + 0x80 = 0x9F`. For `q = 2`, `j = 10`. The
final `1` still falls in the second byte, so the bytes stay `0x1F`
and `0x80`. Proposition J8.3.

**J8.4.** The strings `M || 01` and `M || 1111` differ at bit
`len(M)`. For “abc” that bit is inside the one rate block of
`KECCAK[512]`. The first output bytes are `0x3a` and `0x48`, so the
first difference is at index `0`. A Match on one of those strings
is one Bool on that string. It is not a Match on the other string,
it is not a Match on every message, and it is not a security claim.
Proposition J8.4.
A Match is not called verified.

**J8.5.** The pad byte `0x06` differs at index `0`, left `0x3a`,
right `0x48`. The rate `136` differs at index `0`, left `0x46`,
right `0x7f`. The late truncation differs at index `0`, left
`0x9c`, right `0x7f`. The capacity reading agrees through index
`167` and differs at index `168`, left `0x4c`, right `0x76`.

**J8.6.** `d: Int` is not a size. The diagnostic is `ORC0237`.
Listing J8.3 includes the instance of length `0`, which an array
rejects with `ORC0221`. Listing J8.4 has `257` instances and is
rejected with `ORC0238`. The range `1..257` has `256` instances,
which is the maximum. Listing J8.5 evaluates the instance of length
`1` and returns one zero byte. It does not evaluate the other `255`
lengths.

**J8.7.** The first `168` bytes are Trunc_r of `S1`. The last `32`
bytes are the first `32` bytes of `f(S1)`, which is a different
state. Lane `(0, 4)` of `S1` is not where those `32` bytes come
from. A reading that continues into the capacity of `S1` may differ
starting at index `168`. The two bytes are `0x4c` and `0x76`.
Proposition J8.5.

**J8.8.** `56908` is less than the `101725` steps J7 recorded for
one Keccak-f[1600] lane, so a SHAKE test did not fit in that
module. `128915` is less than the `204158` steps of the 200-byte
test, so a second output of that shape did not fit beside the eight
tests. Bytes `200` through `511` need two further permutations after
the two that produce byte `199`, and those calls were not added.
cSHAKE and KMAC are NIST SP 800-185. HMAC is J14. This lesson builds
none of them. It does not build RawSHAKE as a separate function, a
two-block absorb, or a general `d`.

```text
j8-ledger
b = 1600
cap-128 = 256
cap-256 = 512
rate-128 = 1344
rate-256 = 1088
rate-bytes-128 = 168
rate-bytes-256 = 136
cap-bytes-128 = 32
cap-bytes-256 = 64
suffix-bits = 4
hash-suffix-bits = 2
xof-byte = 31
hash-byte = 6
final-byte = 128
q1-byte = 159
q-empty-128 = 168
q-zeros-empty-128 = 166
q-empty-256 = 136
q-zeros-empty-256 = 134
q-abc-128 = 165
q-zeros-abc-128 = 163
q-abc-256 = 133
q-zeros-abc-256 = 131
abc-bits = 24
n-bits-empty = 4
n-bits-abc = 28
pad-j-empty-128 = 1338
pad-j-empty-256 = 1082
pad-j-abc-128 = 1314
pad-j-abc-256 = 1058
block-empty-128 = 1344
block-abc-256 = 1088
d-32 = 32
d-64 = 64
d-168 = 168
d-200 = 200
d-512 = 512
calls-32 = 1
calls-168 = 1
calls-200 = 2
calls-512 = 4
domain-index = 0
domain-left = 58
domain-right = 72
rate-index = 0
rate-left = 70
rate-right = 127
trunc-index = 0
trunc-left = 156
trunc-right = 127
noreperm-index = 168
noreperm-left = 76
noreperm-right = 118
instances-max = 256
instances-rejected = 257
pad-steps = 48
outlen-steps = 29
span-steps = 6
shake-steps = 919661
step-budget = 1048576
steps-left = 128915
s128-32-steps = 102093
s256-64-steps = 102139
s256-abc-steps = 102143
s128-200-steps = 204158
domain-steps = 102143
wrong-rate-steps = 102029
trunc-steps = 102478
noreperm-steps = 102478
```

## Sources and epigraph record

The quotation is the borrowed wording. The proofs are the lesson's.
Algorithm 8, Algorithm 9, Table 6, and the definitions in §6.2 and
§6.3 are the standard's. The empty-message bytes are the NIST files'
and the XKCP files', and they are also what `hashlib` prints. The
“abc” digest is what `hashlib` prints. It is not a line of the
August 2015 PDF.

**[S19] National Institute of Standards and Technology.** FIPS PUB
202, *SHA-3 Standard: Permutation-Based Hash and Extendable-Output
Functions*, August 2015, DOI `10.6028/NIST.FIPS.202`. The epigraph
is the sentence of the Abstract, on page iii of the PDF, that names
SHAKE128 and SHAKE256. The hyphen in “SHA-3” and the hyphen in
“extendable-output” are ASCII hyphens. The sentence contains no
inner quotation marks. Wording was checked on 2026-10-17 against the
file whose SHA-256 is recorded in §J8.1 and in [T14]. No translation
is involved. The publication names no personal author of the
Abstract. The first sentence of the Abstract is J7's epigraph and
is not this one. The announcement on page iv is a different sentence
and is not the epigraph. J7's record of this PDF is [S18]. This
record is [S19] so that the epigraph tag is not a second definition
of [S18]. Sections used for the derivation, and not as the epigraph,
are §2.1, §4, §5.1, §5.2, §6.1, §6.2, §6.3, and Appendix B.1 and
B.2, including Table 6. §3 is used by the copied permutation and is
not re-proved. NIST SP 800-185 is named and not retrieved.
This record's tag is [S19].

Source: <https://doi.org/10.6028/NIST.FIPS.202>

**[T15] Retrieved files.** On 2026-10-17 the lesson retrieved the
August 2015 PDF from
<https://nvlpubs.nist.gov/nistpubs/FIPS/NIST.FIPS.202.pdf>.
Its SHA-256 is
`1592607831ff0908cc590632ce371c6c95e94025bb1a0c8ae90a4d0ec1ed025e`,
the digest [T14] records. The same day the lesson retrieved the NIST
SHAKE128 example for the 0-bit message from
<https://csrc.nist.gov/csrc/media/projects/cryptographic-standards-and-guidelines/documents/examples/shake128_msg0.pdf>.
Its SHA-256 is
`3013d904e0f6cebc8dad6393cbf3eed4d1a537b3b3dc5e7b48552ff597942974`.
That file's output is `512` bytes beginning `7F9C2BA4`, and those
`512` bytes equal `hashlib.shake_128` of the empty message. The
absorbed block begins `1F`, and lane `(0, 4)` is
`8000000000000000`. The same day the lesson retrieved the NIST
SHAKE256 example for the 0-bit message from
<https://csrc.nist.gov/csrc/media/projects/cryptographic-standards-and-guidelines/documents/examples/shake256_msg0.pdf>.
Its SHA-256 is
`d736c1a93eb6440e1e6b640402e31ea5258281b2b9237f84ea0ba4186518fc59`.
That file's output is `512` bytes beginning `46B9DD2B`, and those
`512` bytes equal `hashlib.shake_256` of the empty message. The
requests for `shake128_msg24.pdf`, `shake256_msg24.pdf`,
`shake128_abc.pdf`, and `shake256_abc.pdf` in that directory each
returned HTTP 404 and an empty body. The same day the lesson
retrieved the XKCP file `ShortMsgKAT_SHAKE128.txt` from
<https://raw.githubusercontent.com/XKCP/XKCP/master/tests/TestVectors/ShortMsgKAT_SHAKE128.txt>.
Its SHA-256 is
`7a1bde3a3ad79ecd851a1f36a4ed5a87a793287d37426038e3326cdfb3f03e11`.
The header names SHAKE128 as in FIPS 202. The line `Len = 0` has a
512-byte `Squeezed` field equal to the NIST output. The line
`Len = 24` has message `1F877C`, which is not “abc”. The same day
the lesson retrieved `ShortMsgKAT_SHAKE256.txt` from
<https://raw.githubusercontent.com/XKCP/XKCP/master/tests/TestVectors/ShortMsgKAT_SHAKE256.txt>.
Its SHA-256 is
`965170bc74bd0c243adfad5014341471e5cc35269e4e5634eaeff526d3140c12`.
The line `Len = 0` matches the NIST SHAKE256 output. The line
`Len = 24` is again message `1F877C`. `hashlib.shake_128` and
`hashlib.shake_256` on this machine printed the empty outputs quoted
above and the “abc” digest quoted in §J8.6. This record's tag is
[T15].

Source: <https://csrc.nist.gov/csrc/media/projects/cryptographic-standards-and-guidelines/documents/examples/shake128_msg0.pdf>

**[C9] Sponge surface.** The listings use `edition 2026`,
`module`, `spec`, `type`, `let`, `Word[8]`, `Word[64]`, `Int`,
`Bool`, `Mod[5]`, `^`, `&`, `|`, `~`, `+`, `-`, `*`, `/`, `%`,
`<<`, `<<<`, `++`, `&&`, `if`, `hex"1f"`, a string literal,
`[0; n]`, `for`, `with`, `as little`, `as Int`, a type parameter
over two named sheet types, a size parameter, and `test`. `as little`
is implemented on the S3u binary and is specified by OEP-0017, which
this lesson does not accept. A computed rotation amount is specified
by OEP-0021, which this lesson does not accept. A type parameter is
specified by OEP-0018, which this lesson does not accept. A size
parameter is specified by OEP-0016, which this lesson does not
accept. A test is specified by OEP-0020, which this lesson does not
accept. A nested array is specified by OEP-0025, which this lesson
does not accept. Identifiers are ASCII. An `Int` used as an array
length is rejected by `ORC0237`. An instance of length `0` is
rejected by `ORC0221`. A function with `257` instances is rejected
by `ORC0238`. A `for` bound in the permutation is a literal. A `for`
bound in Listings J8.5 and J8.6 may be a size parameter. The end of
a range is exclusive. The version line these commands print is
`orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`.
The policy file in this tree is `policy/gate0-repository-policy.json`,
policy version 0.2.170. That version is not part of the compiler's
version line.
This record's tag is [C9].

The test form is the one N12 recorded. `orangec check` checks a test
and does not run it. `orangec eval --spec` runs one parameterless
spec and does not run tests. `orangec test` runs the tests. A failed
check prints the diagnostic and no report. A failed test prints
`left` and `right` and no diagnostic. `orangec test --stats` prints
the evaluator-step count of each test on standard error. Those
counts are the counts Assumption J8.7 names.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

J8 adds eight exercises with worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`. The Orange listings
are the seven fenced programs in this file.
`compiler/crates/orangec/tests/book_j8.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not prove
Assumption J8.9 from a smaller theory of the compiler, and they do
not accept OEP-0016, OEP-0017, OEP-0018, OEP-0020, OEP-0021, or
OEP-0025. A passing test in that file is a Match of the `Bool` the
listing writes.
Do not call that Match verified.
Reaching the last page is not a certificate.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-17 at the
owner's direction. Owner review is pending.
