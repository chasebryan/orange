# Camellia

Camellia is a 128-bit block cipher with 128-, 192- and 256-bit keys, designed
at NTT and Mitsubishi Electric by Aoki, Ichikawa, Kanda, Matsui, Moriai,
Nakajima and Tokita and published in 2000. The IETF describes it in
[RFC 3713](https://www.rfc-editor.org/rfc/rfc3713) (2004), which is the text
this entry follows. It is in the NESSIE portfolio (2003), on the CRYPTREC list
of recommended ciphers, in ISO/IEC 18033-3, and in TLS cipher suites
([RFC 5932](https://www.rfc-editor.org/rfc/rfc5932),
[RFC 6367](https://www.rfc-editor.org/rfc/rfc6367)), as well as in CMS, IPsec
and OpenPGP. It is a current, unbroken standard that most cryptographic
libraries implement and that is rarely negotiated in practice, where AES holds
the same place.

## Analysis

### Structure

Camellia is a Feistel network on two 64-bit halves, D1 and D2, with 18 rounds
for a 128-bit key and 24 rounds for 192- and 256-bit keys. Every six rounds
the halves pass through the key-dependent linear layers FL and FLINV, and the
block is whitened with two 64-bit subkeys before the first round and after the
last. The round function is F(x, k) = P(S(x xor k)): S applies eight 8-bit
S-boxes, one per byte, and P is a byte-wise linear map in which each output
byte is the xor of five or six input bytes. Only one S-box, s1, is a table;
RFC 3713 defines s2(x) = s1(x) <<< 1, s3(x) = s1(x) <<< 7 and s4(x) =
s1(x <<< 1). FL splits its 64-bit input into 32-bit halves and mixes them with
an and, a rotation by one, and an or; FLINV undoes it.

The key schedule (section 2.2) forms two 128-bit values KL and KR from the
key (for a 128-bit key KR is zero; for a 192-bit key KR is the last 64 bits
followed by their complement), then derives KA from KL and KR by four Feistel
rounds keyed with the constants Sigma1 through Sigma4, and KB from KA and KR
by two more rounds with Sigma5 and Sigma6. Every subkey is one 64-bit half of
KL, KR, KA or KB rotated left, as a 128-bit value, by 0, 15, 30, 45, 60, 77,
94 or 111 bits. Decryption is the encryption procedure with the subkeys taken
in the reverse order.

The Orange file follows the RFC section by section. A 128-bit value is the
type `U128`, a `Word[64]^2` holding `v >> 64` and `v & MASK64`, so the data
randomizing part carries [D1, D2] and a block is read and written with
`as big`. The subkeys of one key are a tuple of three arrays, kw1 through kw4,
k1 through k18 (or k24) and ke1 through ke4 (or ke6), of the type
`Subkeys18` or `Subkeys24`. The S-box section prints SBOX1 and defines the
other three from it, and the file does the same: `sbox1` is the RFC's table,
and `sbox2`, `sbox3` and `sbox4` are its three definitions.

| Standard section | Orange spec |
| --- | --- |
| 2.1, 128-bit values and their rotation `<<<` | `U128`, `rotl128` |
| 2.2, KL and KR from the key | `key_schedule_128`, `key_schedule_192`, `key_schedule_256` |
| 2.2, KA and KB with Sigma1 to Sigma6 | `sigma`, `derive_ka`, `derive_kb`, `two_rounds`, `xor128` |
| 2.2, the two subkey tables, `(KL <<< n) >> 64` and `(KL <<< n) & MASK64` | `key_schedule_128`, `subkeys_192_256`, `high`, `low`, `Subkeys18`, `Subkeys24` |
| 2.3.1, 128-bit keys: 18 rounds, FL/FLINV after rounds 6 and 12 | `data_randomizing_128`, `six_rounds`, `two_rounds` |
| 2.3.2, 192- and 256-bit keys: 24 rounds, FL/FLINV after rounds 6, 12 and 18 | `data_randomizing_192_256` |
| 2.3.3, decryption with the subkeys reversed | `decryption_subkeys`, `reversed`, `decrypt_128`, `decrypt_192`, `decrypt_256` |
| 2.4.1, F-function | `f` |
| 2.4.2, FL- and FLINV-functions | `halves`, `fl`, `flinv` |
| 2.4.3, S-boxes: SBOX1 and the derived SBOX2, SBOX3, SBOX4 | `sbox1`, `sbox2`, `sbox3`, `sbox4` |
| 2.2 and 2.3, one block, each key size and direction | `encrypt_128`, `decrypt_128`, `encrypt_192`, `decrypt_192`, `encrypt_256`, `decrypt_256` |

### Security status

Camellia and AES were designed in the same years for the same block and key
sizes, with different architectures. AES is a substitution-permutation
network of 10 to 14 rounds in which every byte passes through an S-box in
every round; Camellia is a Feistel network of 18 or 24 rounds in which eight
of the sixteen bytes do, and the FL and FLINV layers, inserted every six
rounds, break the regularity that attacks on pure Feistel ciphers exploit.
The S-box is inversion in GF(2^8) composed with affine maps on the input and
the output, so it is affine-equivalent to the AES S-box and shares its
properties: differential uniformity 4, nonlinearity 112, algebraic degree 7,
and the low-degree implicit equations behind the algebraic attacks proposed
in 2002 (Courtois and Pieprzyk), which have led to no attack on either
cipher.

The cryptanalytic record, as of 2026, stops well short of the full ciphers.
The designers' own evaluation (Aoki et al., 2000 and 2001) bounded
differential and linear characteristics and studied truncated and
higher-order differentials. Since then, the strongest published results with
the FL and FLINV layers and the whitening in place have come from impossible
differential cryptanalysis (Liu, Li, Gu, Liu, Li and Wang, FSE 2012; Boura,
Naya-Plasencia and Suder, ASIACRYPT 2014) and meet-in-the-middle attacks (Lu,
Wei, Pasalic and Fouque, CT-RSA 2012; Chen and Jia, 2014), which reach about
11 rounds of Camellia-128 and 12 to 13 rounds of Camellia-192 and
Camellia-256 at time and data complexities close to the exhaustive bound.
Variants without the FL layers fall one or two rounds further. The margin is
7 rounds of 18 for Camellia-128 and 11 or more of 24 for the longer keys.
No related-key, weak-key or structural attack on the full cipher is known.

On standing: ISO/IEC 18033-3 (2005, revised 2010) lists Camellia beside AES;
NESSIE selected it in 2003 and CRYPTREC has recommended it since 2003. In the
IETF it has cipher suites for TLS 1.2 (RFC 5932 with CBC, RFC 6367 with GCM),
CMS (RFC 3657), IPsec (RFC 4312, RFC 5529) and OpenPGP (RFC 5581). TLS 1.3
(RFC 8446, 2018) defines no Camellia suites, and the major browsers dropped
or never offered the TLS 1.2 ones, so Camellia is widely available and
seldom used. It carries the same pitfalls as any 128-bit block cipher: a single
block, as evaluated here, is not a mode of operation, and its security in use
depends on the mode and on nonce or IV discipline, not on the cipher.

### What the Orange rendering shows

The only data-dependent operation in Camellia is the S-box lookup. In the
Orange file it is written as the RFC writes it: `s1[x[0]]` for SBOX1[t1],
`s1[x] <<< 1` for SBOX2, `s1[x] <<< 7` for SBOX3 and `s1[x <<< 1]` for
SBOX4, a byte indexing the 256-entry table, which the checker proves in
range. The table is `sbox1`, a literal of 256 decimal entries laid out in the
RFC's sixteen rows of sixteen, so it can be read against section 2.4.3 row by
row. Everything else in the cipher is xor, and, or and rotation on
`Word[64]` and `Word[32]`, one to one with the RFC's text, and no step
branches on data: the one conditional, in `rotl128`, tests the rotation
amount, a constant of the subkey tables.

Byte orders are `as big`. The F-function splits `F_IN ^ KE` into its eight
bytes, t1 the most significant, with `as big Word[8]^8` and joins y1 through
y8 back with `as big Word[64]`; FL and FLINV split their input and subkey
into the 32-bit halves `x1`, `x2`, `k1`, `k2` with `as big` and a tuple, and
assign under the RFC's names, with `_out` on the values the RFC assigns a
second time. A key, a plaintext and a ciphertext are 128-bit values read
and written big-endian with `as big U128`; a 192-bit key is three 64-bit
words, KL the first two and KR the third followed by its complement, and a
256-bit key is two slices of 16 bytes.

The rounds are written as the RFC writes each pair, `D2 = D2 ^ F(D1, k1)`
and then `D1 = D1 ^ F(D2, k2)`, in `two_rounds`; the halves never change
places, six rounds are a loop of three pairs, and the key schedule's
derivation of KA and KB uses the same pair keyed with the Sigma constants.
The 128-bit rotation `<<<` is one spec, `rotl128(v, n)`, with the amount
as the subkey tables give it; from 64 bits on the halves change places and
the rotation by `n - 64` remains, and a shift by 64 gives 0, so the
rotations by 0 and 64 need no case of their own. The subkey tables then
read as the RFC prints them, one subkey per entry: `high(kl, 15)` is
`(KL <<< 15) >> 64` and `low(kl, 15)` is `(KL <<< 15) & MASK64`.
Decryption reuses the encryption path on reversed subkeys, which is what
section 2.3.3 says decryption is: `decryption_subkeys`, one spec with a type
parameter over both subkey layouts, exchanges kw1 and kw2 with kw3 and kw4
and reverses k and ke with `reversed`, one spec with a size parameter for
the arrays of 4, 6, 18 and 24 subkeys.

Measured under `orangec test --stats` and `orangec eval --stats`: an
F-function costs 681 steps, of which 513 build the decimal SBOX1 literal
(a 256-element array literal costs one step per element and one per
literal), which each F call builds once and reads eight times; FL and FLINV
cost 44 each and a 128-bit rotation 47. A round costs about 690 steps.
The key schedule costs 4,006 steps for a 128-bit key (four F calls and 26
rotations) and about 5,790 for a 192- or 256-bit key (six F calls and 34
rotations). The data randomizing part costs 12,761 steps for 18 rounds and
17,034 for 24, and reversing the subkeys 312 or 416. One 128-bit-key block
costs 16,772 to 16,775 steps to encrypt and 17,084 to decrypt; a 192- or
256-bit-key block 22,823 to 22,829 to encrypt and 23,239 to 23,244 to
decrypt. The eight tests together use 165,594 steps. About two thirds of
each block is the table literal; a `hex"..."` table would cost one step to
build, but would not read as the RFC prints it.

Not expressed: constant-time behaviour (a table indexed by a secret byte is
the classic cache-timing pattern, and the lookup is a specification of a
lookup, not a claim about leakage), any mode of operation, and any key size
other than the three of the RFC.

## Dissemination

### Files

- `camellia.or`: the key schedule, the data randomizing part for 18 and 24
  rounds, the F-, FL- and FLINV-functions and the S-boxes, encryption and
  decryption for 128-, 192- and 256-bit keys, and the eight tests below.

### Running

```console
orangec test algorithms/camellia/camellia.or
python3 algorithms/verify.py algorithms/camellia
```

### Vectors

Each row is a `test` block in `camellia.or`, comparing an encryption or a
decryption with the published value.

| Test | Source | Case |
| --- | --- | --- |
| `RFC 3713 Appendix A: 128-bit key encrypts` | RFC 3713, Appendix A | 128-bit key 0123456789abcdeffedcba9876543210, plaintext 0123456789abcdeffedcba9876543210, ciphertext 67673138549669730857065648eabe43 |
| `RFC 3713 Appendix A: 128-bit key decrypts` | RFC 3713, Appendix A | the same ciphertext decrypted under the 128-bit key gives the plaintext |
| `RFC 3713 Appendix A: 192-bit key encrypts` | RFC 3713, Appendix A | 192-bit key 0123...3210 0011223344556677, same plaintext, ciphertext b4993401b3e996f84ee5cee7d79b09b9 |
| `RFC 3713 Appendix A: 192-bit key decrypts` | RFC 3713, Appendix A | the same ciphertext decrypted under the 192-bit key gives the plaintext |
| `RFC 3713 Appendix A: 256-bit key encrypts` | RFC 3713, Appendix A | 256-bit key 0123...3210 00112233445566778899aabbccddeeff, same plaintext, ciphertext 9acc237dff16d76c20ef7c919e3a7509 |
| `RFC 3713 Appendix A: 256-bit key decrypts` | RFC 3713, Appendix A | the same ciphertext decrypted under the 256-bit key gives the plaintext |
| `Botan camellia.vec [Camellia-128] case 2` | Botan `src/tests/data/block/camellia.vec`, section `[Camellia-128]`, second case | key 80000000000000000000000000000000, zero plaintext, ciphertext 6c227f749319a3aa7da235a9bba05a2c |
| `Botan camellia.vec [Camellia-256] case 3` | Botan `camellia.vec`, section `[Camellia-256]`, third case | key 0000000000000200 followed by 24 zero bytes, zero plaintext, ciphertext e18b0cb1980124504b46a46a6f4273f3 |

The RFC 3713 Appendix A values are stated in the RFC itself; they also open
each section of Botan's `camellia.vec`, from which they were copied here, and
the `cryptography` package (Camellia in ECB mode, over OpenSSL) confirms all
eight ciphertexts. The three decryption tests state the RFC's plaintext as
their expected value.

### Provenance and claims

RFC 3713 could not be read from the machine this entry was written on, so
every constant was taken from a fetched reference implementation and
cross-checked by script against a second one, never transcribed by eye or
from memory:

- s1 came from Botan's `camellia.cpp` (`SBOX1`, 256 bytes) and was checked
  equal to the table OpenSSL's `camellia.c` holds as `Camellia_SBOX[0]` (each
  entry replicated in three bytes of a 32-bit word). Botan's `SBOX2`, `SBOX3`
  and `SBOX4` were checked equal to the rotations of s1 that RFC 3713 states,
  which is why the Orange file carries only s1. The first form held it as a
  packed `Word[64]^32` literal, generated by script and unpacked again to
  confirm it.
- Sigma1 through Sigma6 came from OpenSSL's `SIGMA[]` and agree with the
  constants in Botan's key schedule.
- The P-function's matrix was derived from the eight mask constants of
  Botan's `P()` rather than written from memory (a first draft written from
  memory was wrong, and the oracle caught it).
- A Python reference in the RFC's vocabulary (KL, KR, KA, KB, kw, k, ke, F,
  FL, FLINV, S, P), built from those extracted constants, reproduces all 17
  cases of Botan's `camellia.vec` in both directions and agrees with the
  `cryptography` package on 180 random key and block pairs across the three
  key sizes. The Orange specs were written from that reference and the
  fetched sources; the vectors above then matched on the first evaluation.

The extraction, generation and measurement scripts were kept with the work
record and are not part of the repository.

The entry was then rewritten in the current language. Every expected value
is carried over byte for byte from the first form, where each was a
`<name>_expected` spec of bytes, and every key and plaintext is the first
form's; a script compared each `hex"..."` literal of the new tests with the
value the first form's spec evaluates to, and no vector was added or
dropped. The decimal SBOX1 literal was printed by script from the first
form's packed words and checked equal to them, entry by entry, by evaluating
both forms; the Sigma constants are unchanged. The first form numbered the
P-function 2.4.5 and FLINV and the S-boxes 2.4.3 and 2.4.4; the rewrite
cites the P step inside the F-function of 2.4.1, FL and FLINV together in
2.4.2 and the S-boxes in 2.4.3, from the rewriter's knowledge of the RFC,
which could not be read from the build machine this time either.

This entry is a reference evaluation of RFC 3713 under `orangec test`. It
makes no constant-time, side-channel, performance or certification claim,
and it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- There are no module-level constants: a parameterless spec such as the
  SBOX1 table is evaluated again at every call, so each F call builds the
  256-entry literal, about two thirds of the cost of a block. Passing the
  table down from each block would avoid it at the price of a parameter on
  F that the RFC's F(F_IN, KE) does not have.
- There is no 128-bit word, so a 128-bit value is two 64-bit halves and its
  rotation `<<<` is the spec `rotl128`.
