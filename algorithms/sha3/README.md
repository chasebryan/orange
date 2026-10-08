# SHA-3 and SHAKE

SHA-3 is the family of hash functions and extendable-output functions (XOFs)
that NIST standardized in
[FIPS 202](https://doi.org/10.6028/NIST.FIPS.202) (August 2015) from Keccak,
designed by Guido Bertoni, Joan Daemen, Michael Peeters and Gilles Van Assche
and selected in October 2012 as the winner of the SHA-3 competition that NIST
had opened in 2007. All six functions are one sponge construction over the
permutation Keccak-p[1600, 24]: SHA3-224, SHA3-256, SHA3-384 and SHA3-512
with fixed digests, and SHAKE128 and SHAKE256 with output of any length. They
are current standards; SHA3-256, SHA3-512, SHAKE128 and SHAKE256 are the hash
functions inside ML-KEM (FIPS 203) and ML-DSA (FIPS 204), and the derived
functions cSHAKE, KMAC, TupleHash and ParallelHash of SP 800-185 are built on
the same sponge. This entry writes the permutation, the sponge and all six
functions in Orange and reproduces ten digests.

## Analysis

### Structure

The state of Keccak-p[1600, 24] is 1600 bits arranged as a 5 by 5 array of
64-bit lanes A[x, y] (FIPS 202 section 3.1). The Orange file keeps it as the
standard indexes it: `type State = Sheet^5` with `type Sheet = Word[64]^5`,
so `a[x][y]` is lane (x, y), and every step mapping writes lane (x, y) with
the update path `with [x][y]`. A string of 1600 bits is 200 bytes; lane
(x, y) is its word 5y + x read little-endian (section 3.1.2 with the bit
order of Appendix B.1), and the string is read back plane by plane
(section 3.1.3). A round is the five step mappings of section 3.2 applied in
order, and the permutation is 24 rounds, i_r = 0 through 23, with the round
constants of Algorithms 5 and 6.

| Standard section | Orange spec |
| --- | --- |
| 3.1, the state array | `Sheet`, `State` (`a[x][y]`) |
| 3.1.2, string to state array | `state_array` |
| 3.1.3, state array to string | `state_string` |
| 3.2.1, Algorithm 1, theta | `theta` |
| 3.2.2, Algorithm 2, rho (the offsets of Table 2, computed) | `rho_walk`, `rho` |
| 3.2.3, Algorithm 3, pi | `pi` |
| 3.2.4, Algorithm 4, chi | `chi` |
| 3.2.5, Algorithm 5, rc(t) | `rc` |
| 3.2.5, Algorithm 6, RC[i_r] and iota | `round_constants`, `iota` |
| 3.3, Rnd and Algorithm 7, Keccak-p[1600, 24] | `rnd`, `keccak_p` |
| 4, Algorithm 8, step 6, the xor of a block | `xor_string` |
| 4 and 5.2, Algorithm 8, KECCAK[c] | `keccak[c, n]` (c in lanes, n blocks) |
| 5.1, 6 and B.2, pad10*1 after the domain suffix | `pad[q]` (q bytes) |
| 6.1, SHA3-224, SHA3-256, SHA3-384, SHA3-512 | `sha3_224[n]`, `sha3_256[n]`, `sha3_384[n]`, `sha3_512[n]` |
| 6.2, SHAKE128 and SHAKE256 | `shake128[n]`, `shake256[n]` |

Theta computes the five column parities C[x], the five values
D[x] = C[x - 1] xor rot(C[x + 1], 1), and adds D[x] to every lane of column
x. Rho rotates each lane by its own offset, pi moves lane (x, y) to position
(y, 2x + 3y), chi is the row-wise non-linear map
A[x] xor (not A[x + 1] and A[x + 2]), and iota adds the round constant to
lane (0, 0). The sponge (section 4) absorbs r-bit blocks by xor into the
first r bits of the state followed by the permutation, and squeezes output
from the same r bits. Each SHA-3 function fixes the capacity c and the rate
r = 1600 - c; SHA3-224 has r = 1152 bits (144 bytes), SHA3-256 and SHAKE256
have r = 1088 (136 bytes), SHA3-384 has r = 832 (104 bytes), SHA3-512 has
r = 576 (72 bytes) and SHAKE128 has r = 1344 (168 bytes). The message is
followed by a domain suffix, 01 for the hash functions and 1111 for the
XOFs, and then by pad10*1 (section 5.1), which in bytes is the value 0x06 or
0x1f at the first free position and 0x80 at the last byte of the rate, the
two sharing a byte (0x86 or 0x9f) when only one is free (Appendix B.2).

`keccak[c, n]` is KECCAK[c] of section 5.2 for a capacity of c lanes (4
through 16, so 256 through 1024 bits) over a padded string of n blocks of
8(25 - c) bytes, n from 1 through 19: SHA3-256, KECCAK[512], is
`keccak[8, n]`. It runs
Algorithm 8 on strings, as the standard writes it: each block is joined to
c lanes of zeros and xored into S, and S = Keccak-p(S); the first block of
output, Trunc_r(S), is returned, and each function keeps its first d bits
with a slice. `pad[q]` is Appendix B.2's padding in bytes: q is the number
of bytes from the end of the message to the end of its last block,
q = r/8 - (m mod r/8) for a message of m bytes, and the spec takes the
first byte (0x06 or 0x1f) as its argument; it is sized by q rather than
by m for the reason given under Gaps. The hash and XOF specs take the
padded message, `Word[8]^(136 * n)` for SHA3-256, and a test reads
`sha3_256("abc" ++ pad[136 - 3](0x06))`.

### Security status

The SHA-3 competition ran from November 2007 to October 2012 with 64
submissions, 14 second-round candidates and five finalists (BLAKE, Groestl,
JH, Keccak and Skein); NIST chose Keccak for its security margin, its
performance in hardware and its difference in structure from SHA-2. The
sponge construction comes with a proof: Bertoni, Daemen, Peeters and Van
Assche (EUROCRYPT 2008) showed that a sponge over a random permutation is
indifferentiable from a random oracle up to about 2^(c/2) queries, so the
capacity sets the generic security and the rate is what remains for
throughput. FIPS 202 Appendix A.1 states the resulting levels: for a digest
of d bits and capacity c, collision resistance min(d/2, c/2) and preimage
and second-preimage resistance min(d, c/2). With c = 2d for the hash
functions this gives 112, 128, 192 and 256 bits of collision resistance for
SHA3-224 through SHA3-512, and 224, 256, 384 and 512 bits against preimages.
SHAKE128 (c = 256) and SHAKE256 (c = 512) offer at most 128 and 256 bits of
security whatever the output length, and less when the output is short:
min(d/2, 128) or min(d/2, 256) against collisions. Unlike SHA-2, a sponge has
no length-extension property, which is why KMAC (SP 800-185) can be a keyed
prefix construction.

The cryptanalytic record on reduced rounds has advanced slowly against a
permutation of 24 rounds. Dinur, Dunkelman and Shamir (FSE 2012) gave
practical collisions for 4 rounds of Keccak-224 and Keccak-256. Qiao, Song,
Liu and Guo (EUROCRYPT 2017) and Song, Liao and Guo (CRYPTO 2017), collected
in Guo, Liao, Liu, Liu, Qiao and Song, "Practical Collision Attacks against
Round-Reduced SHA-3" (Journal of Cryptology, 2020), produced actual
collisions for 5 rounds of SHAKE128, SHA3-224 and SHA3-256 and for 6 rounds
of reduced-capacity Keccak challenge instances; Huang, Ben-Yehuda, Dunkelman
and Maximov (2022) reached 4 rounds of SHA3-384 in practical time, and Zhang,
Hou and Liu (EUROCRYPT 2023) extended collision attacks to the larger
instances with conditional internal differentials. Preimage attacks stop at
4 rounds and stay far from practical for the standardized parameters: the
linear-structure attacks of Guo, Liu and Song (ASIACRYPT 2016) and the
allocating approach of Li and Sun (EUROCRYPT 2019) cost about 2^207 for
4-round SHA3-224 and 2^239 for 4-round SHA3-256. Zero-sum distinguishers
reach the full 24-round permutation (Boura, Canteaut and De Canniere, 2011,
at complexity 2^1575), but they exploit the permutation's algebraic degree,
not the hash functions, and do not affect the sponge's security claims. As of
September 2026 no published attack on any SHA-3 or SHAKE instance reaches
beyond a quarter of the rounds; the margin is among the largest of the
standardized hash functions.

Two changes between the Keccak submission and the standard matter to a
reader of vectors. First, FIPS 202 appends the domain suffix 01 (SHA-3) or
1111 (SHAKE) before pad10*1, so SHA3-256 of a message differs from the
Keccak-256 of the 2012 submission, which Ethereum still uses; the two agree
on the permutation and the rate and differ only in the padding byte (0x06
against 0x01). Second, NIST's 2013 proposal to reduce the capacities of the
hash functions to 256 and 512 bits was withdrawn after public comment, and
the 2014 draft and the 2015 standard kept the submission's c = 2d.
Deployment pitfalls are few: a SHAKE output of d1 bytes is a prefix of the
output of d2 > d1 bytes, so applications that vary the length must separate
it themselves (cSHAKE's customization string exists for this); and Keccak-p
with fewer rounds (12 in KangarooTwelve and TurboSHAKE, from the Keccak team
in 2016 and 2023) is a distinct design choice, not a SHA-3 parameter.
Status: FIPS 202 (2015) is current, SP 800-185 (2016) builds cSHAKE, KMAC,
TupleHash and ParallelHash on it, and FIPS 203, 204 and 205 (2024) use
SHA3-256, SHA3-512, SHAKE128 and SHAKE256 in ML-KEM, ML-DSA and SLH-DSA.

### What the Orange rendering shows

Nothing in SHA-3 depends on the data except the values themselves: no
branch, no table lookup and no rotation amount depends on the message. The
only conditional in the file is the feedback of the shift register in `rc`,
which depends on the register alone. Every index is a literal, a loop index,
a sum or product of loop indices, a loop index reduced modulo 5, or a
coordinate of rho's walk, and every slice bound is an expression in the
sizes and loop indices; the checker proves each in range before evaluation.

The step mappings read as Algorithms 1 through 4. Theta, pi and chi are
loops over x and y that build the new state with `with [x][y]`, each lane
written with the standard's coordinates: `a[(x + 3 * y) % 5][x]` for pi,
`a[(x + 1) % 5][y]` and `a[(x + 2) % 5][y]` for chi, `c[(x - 1) % 5]` for
theta, where the language's `%` gives (x - 1) mod 5 = 4 at x = 0 as the
standard's mod does. Rho is Algorithm 2 itself: the coordinates (x, y) are
residues modulo 5, `Mod[5]`, stepped to (y, 2x + 3y), and each lane is
rotated by the computed amount (t + 1)(t + 2)/2, which the rotation reduces
modulo 64; the offsets of Table 2 are never typed. The round constants are
Algorithms 5 and 6: `rc` runs the shift register for t = 0 through 167, and
`round_constants` sets bit 2^j - 1 of RC[i_r] to rc(j + 7 i_r), so no
constant is typed either.

Byte order is written once, where Appendix B.1 fixes it: `as little
Word[64]^25` turns 200 bytes into the 25 lanes of section 3.1.2 and back.
Lengths are sizes. `keccak[c, n]` slices block i of the padded message as
`p[8 * (25 - c) * i..8 * (25 - c) * (i + 1)]` and joins it to `[0; (8 * c)]`,
the 0^c of Algorithm 8; `pad[q]` builds its q bytes as a fill with the first
byte set and the last xored with 0x80, so q = 1 gives 0x86 or 0x9f with no
special case. The checker computes each instance's lengths from its
signature, so a test that names the wrong q for its message finds no
instance of the hash spec that takes the result, unless the error is a
whole block. A digest of d bits is the slice `[..d/8]` of Trunc_r(S).

Measured costs under `orangec eval --stats` and `orangec test --stats`: one
Keccak-p[1600, 24] permutation costs 89,719 steps, of which 7,546 build the
round constants and 24 rounds of 3,388 the rest (theta 661, rho 1,286, pi
514, chi 921, iota and the call); the two conversions between string and
state array cost about 420 each. Each block absorbed adds 90,108 steps to
`keccak[c, n]`, and a call to `pad` costs 21 to 27 steps. A one-block test
costs 90,156 to 90,177 steps and the two-block SHA3-256 test 180,289. The
ten tests together use 991,744 steps.

Not expressed: a message whose length is not a whole number of bytes, or
whose padded form is more than 19 blocks (2,584 bytes at the SHA3-256
rate). Not written, by this entry's choice: SHAKE output longer than one
rate (Gaps).

## Dissemination

### Files

- `sha3.or`: Keccak-p[1600, 24], the sponge KECCAK[c] with pad10*1 after
  the domain suffix, SHA3-224, SHA3-256, SHA3-384, SHA3-512, SHAKE128 and
  SHAKE256, and the ten tests below.

### Running

```console
orangec test algorithms/sha3/sha3.or
python3 algorithms/verify.py algorithms/sha3
```

`orangec eval` prints the two parameterless specs, the bits rc(0) through
rc(167) and the 24 round constants.

### Vectors

Each row is a `test` block in `sha3.or`, comparing a digest or an output
with the published or recomputed value.

| Test | Source | Case |
| --- | --- | --- |
| `Botan sha3.vec: SHA3-256 of the empty message` | Botan `src/tests/data/hash/sha3.vec`, `[SHA-3(256)]`, empty `In`; `hashlib.sha3_256` agrees | SHA3-256 of the empty message |
| `Botan sha3.vec: SHA3-512 of the empty message` | Botan `sha3.vec`, `[SHA-3(512)]`, empty `In`; `hashlib.sha3_512` agrees | SHA3-512 of the empty message |
| `hashlib: SHA3-224 of abc` | oracle `hashlib.sha3_224(b"abc")`; pycryptodome `SHA3_224` agrees (added in the rewrite) | SHA3-224 of `abc` |
| `hashlib: SHA3-256 of abc` | oracle `hashlib.sha3_256(b"abc")`; CompactFIPS202.py agrees | SHA3-256 of `abc`, `3a985da7...11431532` |
| `hashlib: SHA3-384 of abc` | oracle `hashlib.sha3_384(b"abc")`; pycryptodome `SHA3_384` agrees (added in the rewrite) | SHA3-384 of `abc` |
| `hashlib: SHA3-512 of abc` | oracle `hashlib.sha3_512(b"abc")`; CompactFIPS202.py agrees | SHA3-512 of `abc` |
| `hashlib: SHAKE128 of the empty message, 32 bytes` | oracle `hashlib.shake_128(b"").digest(32)`; Botan `src/tests/data/xof/shake.vec` (selected from the NIST CAVS file), `[SHAKE-128]`, empty `In` gives the first 16 bytes | SHAKE128 of the empty message, 32 bytes |
| `hashlib: SHAKE256 of abc, 64 bytes` | oracle `hashlib.shake_256(b"abc").digest(64)`; CompactFIPS202.py agrees | SHAKE256 of `abc`, 64 bytes |
| `hashlib: SHA3-256 of the 200 bytes 00 through c7` | oracle `hashlib.sha3_256(bytes(range(200)))`; CompactFIPS202.py agrees | SHA3-256 of the bytes 00 through c7: two blocks at rate 136 |
| `hashlib: SHAKE128 of abc, 168 bytes` | oracle `hashlib.shake_128(b"abc").digest(168)`; CompactFIPS202.py agrees | SHAKE128 of `abc`, one full rate of 168 bytes |

### Provenance and claims

The rotation offsets of Table 2, the pi mapping and the 24 round constants
were not typed from memory. A script derived the offsets from the
(t + 1)(t + 2) / 2 walk of section 3.2.2, the pi positions from
A'[x, y] = A[(x + 3y) mod 5, x], and the constants from the LFSR rc(t) of
Algorithm 5 with bit 2^j - 1 of RC[ir] equal to rc(j + 7 ir); it then
printed the Orange literals that appeared in the first form's `rho`, `pi`,
`chi`, `theta` and `round_constants`. The same script checked the constants against the
register of the Keccak team's reference `CompactFIPS202.py` (from the XKCP
repository) and ran the derived permutation against `KeccakF1600onLanes` on
20 random states. Every expected digest was produced by Python's `hashlib`
and compared with `CompactFIPS202.py`; the three empty-message cases were
also compared with the values in Botan's `sha3.vec` and `shake.vec`.
Botan's `shake.vec` states that its vectors are selected from the NIST CAVS
file; `sha3.vec` carries no such note, so its two values are cited as
Botan's. The scripts live in the worker's scratch directory, not in the
repository.

The entry was then rewritten in the current language, with the state array
`A[x][y]` and the step mappings in the standard's coordinates. Every expected
value is carried over byte for byte from the first form, where each was a
`<name>_expected` spec of bytes: the new tests state the same values as
`hex"..."` literals, traced to the old values by a script that evaluates
the first form, and none of the eight was dropped. The offsets, the pi and
chi coordinates and the round constants are now computed in the file by
the standard's Algorithms 1 through 6 instead of printed as literals; the
computed offsets and the 24 computed constants were compared with the
first form's literals under `orangec eval`. The rewrite adds SHA3-224 and
SHA3-384, one spec each over the shared sponge, and one vector for each:
the digests of `abc` from `hashlib.sha3_224` and `hashlib.sha3_384`,
recomputed with pycryptodome's `SHA3_224` and `SHA3_384`, which agree.

This entry is a reference evaluation of a specification under
`orangec test`. It makes no constant-time, side-channel, performance or
certification claim, and it is not a corpus entry in the sense of The Orange
Book chapter 12.

## Gaps

- Keccak-p is written for 64-bit lanes only, b = 1600, w = 64, l = 6.
  Section 3 defines Keccak-p[b, n_r] for every lane width w = 2^l from 1
  through 64; one function for every width would need a word-width
  parameter the current language does not check: `Word[n]` takes only a
  literal width, and a type parameter `W in {Word[8], ..., Word[64]}` does
  not give w itself as a size for the 25w-bit string, l or the round
  indices 12 + 2l - n_r. Widths 1, 2 and 4 are not words at all.
- No array has zero elements, so the empty message is not a value: `pad`
  returns what is appended, the hash and XOF specs take the padded message,
  and each test joins its message to the padding itself.
- A sized spec with no array parameter needs its size written at the call,
  so each test computes q for its message, `pad[136 - 3]`; the checker
  rejects a wrong q unless it is off by a whole block, but cannot infer it.
  `pad` is sized by q, not by the message length m, because its result
  length depends on both m and the rate r, and array lengths come only from
  sizes, so r cannot be passed as a value; sizes m and r together (lengths
  0 through 200 at five rates) are far over the 256 instances a function
  may have.
- A function has at most 256 instances, counting every combination of its
  sizes. `keccak` takes 13 capacities, so it covers at most 19 blocks
  (13 times 19 is 247 instances): 2,584 bytes of message and padding for
  SHA3-256, 3,192 for SHAKE128. A longer message needs a spec with fewer
  capacities or a block count of its own.
- Squeezing stops after one block (Algorithm 8, step 10 is not written), so
  SHAKE output is at most one rate, 168 or 136 bytes. This is a choice of
  the entry, as the first form made it, not a limit of the language: a
  further size for the number of output blocks would write step 10. No
  vector here needs more.
- FIPS 202 defines the functions on bit strings, and the NIST example values
  include messages of 5, 30, 1605 and 1630 bits; Orange has byte arrays, so
  only whole-byte messages are written and those examples are not
  reproduced.
