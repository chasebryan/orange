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
the same sponge. This entry writes the permutation, the sponge, SHA3-256,
SHA3-512, SHAKE128 and SHAKE256 in Orange and reproduces eight digests.

## Analysis

### Structure

The state of Keccak-p[1600, 24] is 1600 bits arranged as a 5 by 5 array of
64-bit lanes A[x, y] (FIPS 202 section 3.1). The Orange file keeps it as
`Word[64]^25` with A[x, y] at index x + 5y and loads each lane little-endian
from the byte string, exactly the conversion of section 3.1.2. A round is the
five step mappings of section 3.2 applied in order, and the permutation is 24
rounds, ir = 0 through 23, with the round constants of Algorithm 5.

| Standard section | Orange spec |
| --- | --- |
| 3.1.2, string to state array | `lanes` |
| 3.2.1, theta | `theta` |
| 3.2.2, rho (Table 2 offsets) | `rho` |
| 3.2.3, pi | `pi` |
| 3.2.4, chi | `chi` |
| 3.2.5, iota and the constants RC[ir] | `iota`, `round_constants` |
| 3.3, Rnd and Keccak-p[1600, 24] | `rnd`, `keccak_p` |
| 4, Algorithm 8, absorbing | `absorb` |
| 4, Algorithm 8, squeezing (one block, d <= r) | `squeeze_32`, `squeeze_64`, `squeeze_168` |
| 5.1 and B.2, pad10*1 with the domain suffix | `pad` |
| 6.1, SHA3-256 and SHA3-512 | `sha3_256`, `sha3_256_two_blocks`, `sha3_512` |
| 6.2, SHAKE128 and SHAKE256 | `shake128_32`, `shake128_168`, `shake256_64` |

Theta computes the five column parities C[x], the five values
D[x] = C[x - 1] xor rot(C[x + 1], 1), and adds D[x] to every lane of column
x. Rho rotates each lane by its own offset, pi moves lane (x, y) to position
(y, 2x + 3y), chi is the row-wise non-linear map
A[x] xor (not A[x + 1] and A[x + 2]), and iota adds the round constant to
lane (0, 0). The sponge (section 4) absorbs r-bit blocks by xor into the
first r bits of the state followed by the permutation, and squeezes output
from the same r bits. Each SHA-3 function fixes the capacity c and the rate
r = 1600 - c; SHA3-256 and SHAKE256 have r = 1088 bits (136 bytes), SHA3-512
has r = 576 (72 bytes) and SHAKE128 has r = 1344 (168 bytes). The message is
followed by a domain suffix, 01 for the hash functions and 1111 for the
XOFs, and then by pad10*1 (section 5.1), which in bytes is the value 0x06 or
0x1f at the first free position and 0x80 at the last byte of the rate
(Appendix B.2).

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

Everything in SHA-3 is data-independent: no branch, no table, no rotation
depends on the message, and the Orange file has no conditional inside the
permutation. The state array with its (x, y) coordinates becomes a flat
`Word[64]^25` because Orange has no arrays of arrays; the index x + 5y is the
standard's own lane order (section 3.1.2), so the little-endian byte loading
is the one the standard describes. Rho is the one place the language shapes
the text: a rotation amount must be a literal, so Table 2 appears as the 25
written-out rotations `a[i] <<< offset` rather than as an offset table read
in a loop. Theta, pi and chi are 25-element array literals laid out one line
per plane y, so each line is the standard's formula for one row; the column
parities C and D of theta are loops with the indices (x + 4) mod 5 and
(x + 1) mod 5 of the standard.

The sponge needs a message length, and Orange has no data-dependent index, so
`pad` finds the suffix byte by comparing the loop index with n and the final
byte by comparing with r - 1 (about 3,800 steps for the two passes over 200
bytes). Every message travels as a 200-byte string, the width of the state,
with zeros beyond its length, which lets one `absorb` serve all three rates.
Squeezing more than one block is not needed for these outputs (d <= r in
every case) and is not written.

Measured under `orangec eval`: one Keccak-p[1600, 24] permutation costs about
23,300 steps, a one-block SHA3-256 about 35,000, SHAKE128 with 168 bytes of
output about 70,000 (the 168-byte squeeze is a third of it), and the
two-block SHA3-256 about 105,000. The eight vectors together use about
420,000 of the 1,048,576-step budget; six more two-block vectors would fit,
and nothing was dropped or moved to a second file.

## Dissemination

### Files

- `sha3.or`: Keccak-p[1600, 24], the sponge with pad10*1, SHA3-256, SHA3-512,
  SHAKE128 and SHAKE256, and the eight vector pairs.

### Running

```console
orangec eval algorithms/sha3/sha3.or
python3 algorithms/verify.py algorithms/sha3
```

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `botan_sha3_256_empty` | Botan `src/tests/data/hash/sha3.vec`, `[SHA-3(256)]`, empty `In`; `hashlib.sha3_256` agrees | SHA3-256 of the empty message |
| `botan_sha3_512_empty` | Botan `sha3.vec`, `[SHA-3(512)]`, empty `In`; `hashlib.sha3_512` agrees | SHA3-512 of the empty message |
| `hashlib_sha3_256_abc` | oracle `hashlib.sha3_256(b"abc")`; CompactFIPS202.py agrees | SHA3-256 of `abc`, `3a985da7...11431532` |
| `hashlib_sha3_512_abc` | oracle `hashlib.sha3_512(b"abc")`; CompactFIPS202.py agrees | SHA3-512 of `abc` |
| `hashlib_shake128_empty_32` | oracle `hashlib.shake_128(b"").digest(32)`; Botan `src/tests/data/xof/shake.vec` (selected from the NIST CAVS file), `[SHAKE-128]`, empty `In` gives the first 16 bytes | SHAKE128 of the empty message, 32 bytes |
| `hashlib_shake256_abc_64` | oracle `hashlib.shake_256(b"abc").digest(64)`; CompactFIPS202.py agrees | SHAKE256 of `abc`, 64 bytes |
| `hashlib_sha3_256_200_bytes` | oracle `hashlib.sha3_256(bytes(range(200)))`; CompactFIPS202.py agrees | SHA3-256 of the bytes 00 through c7: two blocks at rate 136 |
| `hashlib_shake128_abc_168` | oracle `hashlib.shake_128(b"abc").digest(168)`; CompactFIPS202.py agrees | SHAKE128 of `abc`, one full rate of 168 bytes |

### Provenance and claims

The rotation offsets of Table 2, the pi mapping and the 24 round constants
were not typed from memory. A script derived the offsets from the
(t + 1)(t + 2) / 2 walk of section 3.2.2, the pi positions from
A'[x, y] = A[(x + 3y) mod 5, x], and the constants from the LFSR rc(t) of
Algorithm 5 with bit 2^j - 1 of RC[ir] equal to rc(j + 7 ir); it then
printed the Orange literals that appear in `rho`, `pi`, `chi`, `theta` and
`round_constants`. The same script checked the constants against the
register of the Keccak team's reference `CompactFIPS202.py` (from the XKCP
repository) and ran the derived permutation against `KeccakF1600onLanes` on
20 random states. Every expected digest was produced by Python's `hashlib`
and compared with `CompactFIPS202.py`; the three empty-message cases were
also compared with the values in Botan's `sha3.vec` and `shake.vec`.
Botan's `shake.vec` states that its vectors are selected from the NIST CAVS
file; `sha3.vec` carries no such note, so its two values are cited as
Botan's. The scripts live in the worker's scratch directory, not in the
repository.

This entry is a reference evaluation of a specification under `orangec
eval`. It makes no constant-time, side-channel, performance or certification
claim, and it is not a corpus entry in the sense of The Orange Book
chapter 12.

## Gaps

- Rotation amounts must be literals, so rho cannot loop over an offset table;
  the 25 rotations are written out with their amounts. No cost, but the
  table of section 3.2.2 is read from the code rather than from a literal
  array.
- Indices must be static, so the padding positions n and r - 1 are found by
  comparison in a loop over the 200-byte string (about 3,800 steps per
  message instead of two updates), and a message length cannot parametrize an
  array type: every message is carried in a 200-byte string, and a message of
  more than one block has its split written for its length
  (`sha3_256_two_blocks` for 200 bytes at rate 136).
- FIPS 202 defines the functions on bit strings, and the NIST example values
  include messages of 5, 30, 1605 and 1630 bits; Orange has byte arrays, so
  only whole-byte messages are written and those examples are not reproduced.
- SHA3-224 and SHA3-384 are not included; each would be one further spec with
  r = 144 or 104 bytes and a 28- or 48-byte squeeze.
- The step budget was not reached: the file uses about 420,000 of 1,048,576
  steps.
