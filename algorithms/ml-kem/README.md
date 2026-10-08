# ML-KEM-512

ML-KEM is the module-lattice-based key-encapsulation mechanism that NIST
standardized in
[FIPS 203, Module-Lattice-Based Key-Encapsulation Mechanism Standard](https://doi.org/10.6028/NIST.FIPS.203)
(August 2024). It is CRYSTALS-Kyber (Bos, Ducas, Kiltz, Lepoint,
Lyubashevsky, Schanck, Schwabe, Seiler and Stehle, IEEE EuroS&P 2018), the
key-encapsulation mechanism NIST selected in July 2022 from the
post-quantum standardization process it opened in 2016, with the changes
FIPS 203 made in standardizing it. Its security rests on the module
learning-with-errors problem, for which no efficient quantum algorithm is
known. FIPS 203 defines three parameter sets, ML-KEM-512, ML-KEM-768 and
ML-KEM-1024, for NIST security categories 1, 3 and 5. ML-KEM-768 is the
lattice half of the hybrid key exchange `X25519MLKEM768` that TLS 1.3
clients and servers deploy, and of OpenSSH's default key exchange
`mlkem768x25519-sha256` since OpenSSH 10.0 (2025). This entry writes
ML-KEM-512 in Orange, from the hash functions of FIPS 202 that it uses up to
the three internal algorithms of FIPS 203 section 6, and reproduces twelve
published vectors.

## Analysis

### Structure

ML-KEM is two layers (FIPS 203 sections 5 and 6). K-PKE is a public-key
encryption scheme over the ring R_q = Z_q[X]/(X^256 + 1) with q = 3329.
Its key generation (Algorithm 13) expands a 32-byte seed rho into a k x k
matrix A-hat of polynomials, samples a secret vector s and an error vector
e with small coefficients, and publishes t = A s + e; encryption
(Algorithm 14) samples y, e1 and e2 from a 32-byte seed r, and sends
u = A^T y + e1 and v = t^T y + e2 plus the message, each coefficient of the
message a 0 or a rounded q/2; decryption (Algorithm 15) computes
v - s^T u, which is the message plus a small error, and rounds. The KEM
(Algorithms 16 to 18) is a Fujisaki-Okamoto transform of K-PKE: the
encapsulator draws a 32-byte message m, derives the key K and the
encryption coins r together as G(m || H(ek)), and encrypts m under r;
the decapsulator decrypts, derives K' and r' the same way, re-encrypts, and
returns K' only if the ciphertext it obtains equals the one it received.
Otherwise it returns J(z || c), a key derived from the secret z that the
decapsulation key carries: implicit rejection, so that a modified
ciphertext yields a pseudorandom key rather than an error.

Three devices make this efficient and compact, and each is a section of
the standard. Polynomials are multiplied in the number-theoretic transform
domain (section 4.3): 17 is a primitive 256th root of unity modulo q, so
X^256 + 1 splits into 128 quadratic factors X^2 - zeta^(2 BitRev7(i) + 1),
the NTT (Algorithm 9) maps a polynomial to its 128 residues, and
MultiplyNTTs (Algorithms 11 and 12) multiplies residue by residue. The
matrix A-hat is sampled directly in the NTT domain by rejection from a
SHAKE128 stream (SampleNTT, Algorithm 7), and the small polynomials come
from a centered binomial distribution over SHAKE256 output (SamplePolyCBD,
Algorithm 8). The ciphertext is compressed (section 4.2.1): each
coefficient of u keeps its top d_u = 10 bits and each of v its top
d_v = 4 bits, and the encodings ByteEncode_d and ByteDecode_d pack d-bit
integers into bytes. The hash functions are those of section 4.1: H is
SHA3-256, J is SHAKE256 with 32 bytes of output, G is SHA3-512 split into
two halves, PRF_eta is SHAKE256 with 64 eta bytes of output, and the XOF is
SHAKE128. ML-KEM-512 has k = 2, eta_1 = 3 and eta_2 = 2, so an
encapsulation key is 800 bytes, a decapsulation key 1632 and a ciphertext
768.

The entry is two files. `keccak.or` is the module of FIPS 202 the KEM
needs: Keccak-p[1600, 24] with its state as the standard indexes it, a
5 x 5 array of 64-bit lanes `A[x][y]` (`type State = Sheet^5`, a sheet
being the five lanes of one x), the sponge, and the four functions.
`ml-kem.or` uses it and follows FIPS 203 section by section, with the
standard's types: `type Zq = Mod[3329]`, a polynomial `Poly = Zq^256`, a
vector `Vector = Poly^2` and the matrix `Matrix = Vector^2`, so that
`a[i][j]` is A-hat[i, j] and `a[i][j][k]` its coefficient k. A compressed
coefficient is an element of Z_(2^d), `Mod[(1 << d)]`, as the standard
types it.

| Standard section | Orange spec |
| --- | --- |
| FIPS 202 3.2.1, Algorithm 1, theta | `parity`, `add_lane`, `theta` (`keccak.or`) |
| FIPS 202 3.2.2, Algorithm 2 and Table 2, rho | `rho` |
| FIPS 202 3.2.3, Algorithm 3, pi | `pi` |
| FIPS 202 3.2.4, Algorithm 4, chi | `chi_sheet`, `chi` |
| FIPS 202 3.2.5, Algorithms 5 and 6, iota and its constants | `round_constants`, the last line of `keccak_p` |
| FIPS 202 3.3, Algorithm 7, Keccak-p[1600, 24] | `keccak_p` |
| FIPS 202 4, Algorithm 8, the sponge | `absorb`, `squeeze` |
| FIPS 202 5.1, 6.1, 6.2 and B.2, padding, SHA3-256, SHA3-512, SHAKE128, SHAKE256 | `sha3_256`, `sha3_512`, `shake128`, `shake256` |
| FIPS 203 4.1, PRF, H, J, G | `prf`, `hash_h`, `hash_j`, `hash_g` (`ml-kem.or`) |
| FIPS 203 4.1, XOF | `keccak::shake128` and `keccak::squeeze` in `sample_ntt` |
| FIPS 203 4.2.1, Algorithms 3, 5 and 6, ByteEncode and ByteDecode | `byte_encode`, `byte_encode_12`, `byte_decode`, `byte_decode_12` |
| FIPS 203 4.2.1, equations 4.7 and 4.8, Compress and Decompress | `compress`, `decompress` |
| FIPS 203 4.2.2, Algorithm 7, SampleNTT | `sample_ntt` |
| FIPS 203 4.2.2, Algorithms 4 and 8, BytesToBits and SamplePolyCBD | `bytes_to_bits`, `sample_poly_cbd` |
| FIPS 203 4.3 and Appendix A, zeta^BitRev7(i) | `zetas` |
| FIPS 203 4.3, Algorithms 9 and 10, NTT and NTT^-1 | `ntt`, `inverse_ntt` |
| FIPS 203 4.3.1, Algorithms 11 and 12, MultiplyNTTs and BaseCaseMultiply | `multiply_ntts`, `base_case_multiply` |
| FIPS 203 2.4, sums and products of polynomials, vectors and matrices | `add`, `subtract`, `dot` |
| FIPS 203 5.1, Algorithm 13, K-PKE.KeyGen | `sample_matrix`, `k_pke_keygen` |
| FIPS 203 5.2, Algorithm 14, K-PKE.Encrypt | `k_pke_encrypt` |
| FIPS 203 5.3, Algorithm 15, K-PKE.Decrypt | `k_pke_decrypt` |
| FIPS 203 6.1, Algorithm 16, ML-KEM.KeyGen_internal | `keygen_internal` |
| FIPS 203 6.2, Algorithm 17, ML-KEM.Encaps_internal | `encaps_internal` |
| FIPS 203 6.3, Algorithm 18, ML-KEM.Decaps_internal | `decaps_internal` |

The internal algorithms are deterministic: key generation takes the seeds
d and z, and encapsulation the message m, which ML-KEM.KeyGen and
ML-KEM.Encaps (section 7) draw from a random bit generator. Those outer
algorithms, and the input checks of sections 7.2 and 7.3, are not written
(see Gaps).

### Security status

Record as written in October 2026.

K-PKE is IND-CPA secure if the module learning-with-errors problem is hard
for the module rank and noise of the parameter set, and the transform of
section 6 makes ML-KEM IND-CCA2 secure in the random-oracle model, with
proofs in the quantum random-oracle model that are not tight; the
transform with implicit rejection is the one Hofheinz, Hovelmanns and Kiltz
analysed in 2017. The best known attacks on the lattice problem are lattice
reduction, primal and dual, with sieving as the cost of the shortest-vector
subroutine; no attack on ML-KEM or Kyber below the claimed security
categories is known. The margin of the smallest parameter set has been the
debated point: NIST's third-round report (NIST IR 8413, 2022) placed
Kyber-512 in category 1, at least as hard to break as AES-128 by key search,
with an estimate that depends on how the cost of memory access in lattice
sieving is counted, and that placement was publicly contested. FIPS 203
recommends ML-KEM-768 as the default parameter set, and it is the one the
deployed hybrids use.

Decryption can fail, because the rounding in decryption can be overwhelmed
by the error terms; FIPS 203 gives the failure probability of ML-KEM-512 as
2^-138.8 (Table 1), and the implicit rejection of Algorithm 18 hides a
failure from the sender. The changes FIPS 203 made to Kyber's third-round
specification are small and visible in this file: the message m is used
as drawn rather than hashed first, K is the first half of G(m || H(ek))
rather than a further key derivation, the implicit-rejection key is
J(z || c), and the final standard added k as a domain-separation byte to
G(d || k) in key generation. Sections 7.2 and 7.3 add input checks before
encapsulation (every coefficient of ek below q) and decapsulation (the
hash of ek inside dk).

The attacks on deployed ML-KEM and Kyber have been on implementations.
KyberSlash (2024) measured secret-dependent timing in the division by q
that the reference code's compression used, and recovered keys from
libraries that copied it; a compiler-introduced branch on the message bit
in the reference code was shown exploitable the same year. Decapsulation
must compare the two ciphertexts and select the key in constant time, which
is what Wycheproof's `Strcmp` vector below probes from the other side: a
comparison that stopped at a zero byte would accept a ciphertext it must
reject. Masking and fault countermeasures against power and fault analysis
of the NTT and of the Fujisaki-Okamoto re-encryption are an active
engineering topic. FIPS 203 is the current standard.

### What the Orange rendering shows

Four steps depend on data. SampleNTT keeps a candidate only when it is
below q, so the position at which a coefficient is written depends on the
SHAKE128 stream: `sample_ntt` carries the count j as a `Word[16]`, guards
each write with `if (d1 < 3329) && (j < 256)`, and writes at
`a with [j as Word[8]]`, an index the checker proves in range because any
`Word[8]` indexes a polynomial of 256 coefficients. SamplePolyCBD reads
the secret bits of the PRF output, here by first writing them out one to an
element (`bytes_to_bits`, Algorithm 4) and then summing the two halves of
each coefficient's 2 eta bits from a slice. Compression divides a secret
value by q, written as the `Int` division `(2^(d + 1) x + q) / 2q`; it is
the computation that KyberSlash timed in C. And decapsulation compares the
re-encryption with the received ciphertext as whole arrays, `c == c_prime`,
whose cost does not depend on where they differ, and chooses between K' and
J(z || c) with an `if`; both keys are computed before the choice, as
Algorithm 18 computes them. The zetas of the NTT are read from the table
of Appendix A at indices built from the loop counters, never from data.

The NTT is written as a loop of 128 butterflies per layer rather than the
standard's three nested loops, because a loop's bounds are fixed: in layer
l, with len = 128 / 2^l, butterfly m belongs to block m div len, joins
coefficients m + (m div len) len and that plus len, and takes zeta number
2^l plus its block, the i that Algorithm 9 counts up. The index arithmetic
is in `Word[8]`, so every index into a polynomial is proved in range. The
quadratic factors' constants zeta^(2 BitRev7(i) + 1) of MultiplyNTTs are
not a second table: they are 17 (zeta^BitRev7(i))^2, computed from the
first.

Byte orders are those of the two standards and are written once each. The
sponge reads a block as little-endian lanes (`as little Word[64]^lanes`)
into `A[i mod 5][i div 5]` and squeezes the lanes back the same way.
ByteEncode_d is the standard's bit string read in groups: eight
coefficients of d bits are exactly d bytes, the little-endian form of
F[8g] + F[8g + 1] 2^d + ... + F[8g + 7] 2^(7d), so `byte_encode` builds
that integer and writes it with `v as little Word[8]^d` into the slice
`[d g .. d g + d]`, and `byte_decode` reads it back with `as little Int`.
ByteEncode_12 and ByteDecode_12 are separate specs because their integers
are residues modulo q rather than modulo 2^12, and ByteDecode_12 reduces a
12-bit value of q or more, as the standard does.

Lengths are sizes. ByteEncode_d, Compress_d and Decompress_d are one spec
each over d, and PRF_eta and SamplePolyCBD_eta one spec each over eta. The
hash functions of `keccak.or` take messages of 32w + t bytes, w from 1
through 25 and t from 0 through 2, because a function has at most 256
instances and every string ML-KEM-512 hashes is 32-byte seeds and
encodings followed by at most two index bytes; the padding computes the
number of blocks from w and t. G takes its two input lengths, 33 and 64
bytes, as a type parameter over `Word[8]^33` and `Word[8]^64`.

One departure from the standard's control flow is forced and stated where
it happens: SampleNTT squeezes three bytes at a time for as long as it
needs, and this file squeezes a fixed four blocks of SHAKE128, 224 triples,
where 256 coefficients take about 158 on average. Every matrix in the
eleven vectors completes within 168 triples, and every ML-KEM-512 case in
the two ACVP files within 172; a matrix that ran out would keep zeros and
fail its test.

Measured with `orangec test --stats` and `orangec eval --stats`: one
Keccak-p[1600, 24] costs about 21,600 steps (about 900 a round), SHA3-512
of one block 22,100, SHA3-256 or J of 800 bytes (six blocks) 133,000;
SampleNTT 108,000, of which four permutations are 87,000; PRF_3 44,000 and
SamplePolyCBD_3 a further 37,000; an NTT 51,000, an NTT^-1 57,000 and
MultiplyNTTs 13,000; ByteEncode_12 6,400. One key generation costs about
1,220,000 steps, one encapsulation 1,356,000 and one decapsulation
1,628,000; the permutation accounts for more than half of each (31
permutations in key generation). The eleven tests use 17,091,100 steps,
about one second of evaluation.

## Dissemination

### Files

- `keccak.or`: the module `keccak`, Keccak-p[1600, 24], the sponge with
  pad10*1, SHA3-256, SHA3-512, SHAKE128 and SHAKE256 of FIPS 202. It has no
  tests of its own; the ML-KEM vectors check it.
- `ml-kem.or`: ML-KEM-512 of FIPS 203, from the hash functions of section
  4.1 through the internal algorithms of section 6, and the eleven tests.

### Running

    orangec test --steps 1073741824 algorithms/ml-kem/ml-kem.or
    python3 algorithms/verify.py algorithms/ml-kem

The tests need more than the default budget of 1,048,576 steps;
`verify.py` passes the largest budget.

### Vectors

"ACVP" is NIST's Automated Cryptographic Validation Protocol test data for
ML-KEM, revision FIPS203, vector set 42, the files
`ML-KEM-keyGen-FIPS203/internalProjection.json` and
`ML-KEM-encapDecap-FIPS203/internalProjection.json`, which carry each
case's inputs and results together. "Wycheproof" is Project Wycheproof's
`mlkem_512_test.json`. All cases are ML-KEM-512.

| Test | Source | Case |
| --- | --- | --- |
| `ACVP keyGen tcId 1: KeyGen_internal(d, z)` | ACVP keyGen, tgId 1 (AFT), tcId 1 | `keygen_internal(d, z)` gives the case's ek and dk |
| `ACVP keyGen tcId 2: KeyGen_internal(d, z)` | ACVP keyGen, tgId 1, tcId 2 | the same, second case |
| `ACVP keyGen tcId 3: KeyGen_internal(d, z)` | ACVP keyGen, tgId 1, tcId 3 | the same, third case |
| `ACVP encapDecap tcId 1: Encaps_internal(ek, m)` | ACVP encapDecap, tgId 1 (encapsulation, AFT), tcId 1 | `encaps_internal(ek, m)` gives the case's K and c |
| `ACVP encapDecap tcId 2: Encaps_internal(ek, m)` | ACVP encapDecap, tgId 1, tcId 2 | the same, second case |
| `ACVP encapDecap tcId 3: Encaps_internal(ek, m)` | ACVP encapDecap, tgId 1, tcId 3 | the same, third case |
| `ACVP encapDecap tcId 76: implicit rejection of a modified ciphertext` | ACVP encapDecap, tgId 4 (decapsulation, VAL), tcId 76, reason "modified ciphertext" | `decaps_internal(dk, c)` gives J(z \|\| c) |
| `ACVP encapDecap tcId 77: implicit rejection of a modified ciphertext` | ACVP encapDecap, tgId 4, tcId 77, reason "modified ciphertext" | the same, second case |
| `ACVP encapDecap tcId 79: Decaps_internal of a valid ciphertext` | ACVP encapDecap, tgId 4, tcId 79, reason "valid decapsulation" | `decaps_internal(dk, c)` gives the encapsulated K |
| `ACVP encapDecap tcId 80: Decaps_internal of a valid ciphertext` | ACVP encapDecap, tgId 4, tcId 80, reason "valid decapsulation" | the same, second case |
| `Wycheproof tcId 1: key pair from the seed, implicit rejection` | Wycheproof `mlkem_512_test.json`, tcId 1 (source CCTV/strcmp, flag `Strcmp`, result valid) | the key pair of the 64-byte seed d \|\| z has the case's ek, and decapsulating c gives the case's K by implicit rejection |

The Wycheproof case is a different shape from the ACVP ones: it gives a
seed rather than a key pair, so the test runs key generation and
decapsulation in one. Its ciphertext and the re-encryption that
decapsulation computes both begin with a zero byte and differ from the
second byte on, so an implementation that compared them with `strcmp()`
would stop at once, find them equal, and return K' instead of rejecting.

### Provenance and claims

Every expected value in `ml-kem.or` is copied from the named JSON files,
which the build machine held as data, by a script that printed the
`hex"..."` lines of each test; none was typed. Before they were written
down, a second script recomputed every case of the two ACVP files for
ML-KEM-512 (the 25 keyGen cases, the 25 encapsulation cases and the 10
decapsulation cases) and Wycheproof's tcId 1 with kyber-py 1.2.0
(`kyber_py.ml_kem.ML_KEM_512`, its `_keygen_internal`, `_encaps_internal`
and `_decaps_internal`, an implementation independent of this file), and
all agreed with the files; for the Wycheproof case the seed was split as
d = its first 32 bytes and z = its last 32. The same script counted the
triples SampleNTT consumes for each case's matrix, which fixed the four
SHAKE128 blocks of `sample_ntt`. The table of `zetas` was printed by a
script from 17^BitRev7(i) mod 3329 and agrees with kyber-py's table and
with Appendix A; the 24 round constants of `keccak.or` were printed from
the register of FIPS 202 Algorithm 5 and agree with the table of the SHA-3
entry, which was checked against the Keccak team's `CompactFIPS202.py`.
While the module was written, SHA3-256, SHA3-512, SHAKE128 (four blocks of
output) and SHAKE256 of `keccak.or` were compared with Python's `hashlib`
on messages of 33, 34, 64 and 800 bytes; those comparisons are development
checks, not tests in the file. As a check that the rejection tests can
fail, a copy of the file whose decapsulation always returned K' failed the
three implicit-rejection tests and passed the other eight.

This entry is a new entry, written directly in the current language; it
has no earlier form whose values it carries over.

This entry is a reference evaluation of FIPS 203 and FIPS 202 under
`orangec test`. It makes no constant-time, side-channel, performance or
certification claim, and it is not a corpus entry in the sense of The
Orange Book chapter 12.

## Gaps

None that prevented a planned vector. What the language still shapes:

- A loop runs a fixed number of steps, so SampleNTT squeezes a fixed four
  blocks of SHAKE128 instead of squeezing until 256 coefficients are
  found; a matrix that needed more than 224 triples (none in the ACVP files
  needs more than 172) would be computed wrongly, and the test would fail.
- A function has at most 256 instances, so the hash functions take message
  lengths of the form 32w + t with t at most 2 and at most 800 bytes, the
  lengths ML-KEM-512 hashes, rather than every length; H and J are written
  for their 800-byte inputs and G for its 33- and 64-byte inputs.
- Only ML-KEM-512 is written. ML-KEM-768 and ML-KEM-1024 differ in k,
  eta_1, d_u and d_v; `Vector` and `Matrix` fix k = 2, and the key and
  ciphertext lengths are written for it. A size parameter for k would make
  one source of all three.
- The outer algorithms ML-KEM.KeyGen, ML-KEM.Encaps and ML-KEM.Decaps of
  section 7, with their random bit generator and the input checks of
  sections 7.2 and 7.3, are not written; the ACVP `encapsulationKeyCheck`
  and `decapsulationKeyCheck` groups are therefore not reproduced.
- Cost: one key generation takes about 1.2 million steps and one
  decapsulation about 1.6 million, so the eleven vectors use 17 million of
  the 25 million steps an entry should stay under; the remaining ACVP cases
  would not fit in one file.
