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

The Orange file follows the RFC section by section. A 128-bit value is a
`Word[64]^2` with the high word first; a block is `Word[8]^16`; the subkeys
of one key are one array, laid out as kw1, kw2, k1 through k18 (or k24), ke1
through ke4 (or ke6), kw3, kw4.

| Standard section | Orange spec |
| --- | --- |
| 2.2, KL and KR from the key | `key_schedule_128`, `key_schedule_192`, `key_schedule_256` |
| 2.2, KA and KB with Sigma1 to Sigma6 | `sigma`, `derive_ka`, `derive_kb` |
| 2.2, the subkey table and its 128-bit rotations | `subkeys_128`, `subkeys_192_256`, `rotl128_15` to `rotl128_111` |
| 2.3, 128-bit key: 18 rounds, FL/FLINV after rounds 6 and 12 | `data_randomizing_128`, `feistel_round` |
| 2.3, 192- and 256-bit keys: 24 rounds, FL/FLINV after rounds 6, 12 and 18 | `data_randomizing_192_256` |
| 2.3, decryption with the subkeys reversed | `reverse_subkeys_128`, `reverse_subkeys_192_256`, `decrypt_128`, `decrypt_192`, `decrypt_256` |
| 2.4.1, F-function | `f_function` |
| 2.4.2, FL-function | `fl_function` |
| 2.4.3, FLINV-function | `flinv_function` |
| 2.4.4, S-function, SBOX1 and the derived s2, s3, s4 | `s_function`, `sbox1`, `lookup`, `byte_at`, `s1`, `s2`, `s3`, `s4` |
| 2.4.5, P-function | `p_function` |

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

The only data-dependent operation in Camellia is the s1 lookup. A byte may
index a table of 256 entries; this rendering's `lookup` still scans the 32
packed words with a static index, keeps the one whose position matches the
top five bits of the byte, and selects one of its eight bytes with a
conditional: about 290 steps per lookup, where a table-driven implementation
spends one memory access. The
other three S-boxes cost nothing beyond s1, since the RFC defines them as
rotations of it. Everything else in the cipher is 64-bit xor, and, or, shift
and rotate on `Word[64]` and `Word[32]`, one to one with the RFC's text; the
FL and FLINV layers, in particular, read exactly as section 2.4.2 and 2.4.3
write them.

The key schedule's rotations of a 128-bit value are seven specs,
`rotl128_15` through `rotl128_111`. A shift amount may be computed; this
rendering keeps one spec per amount. For the amounts above 64 the two words
change places and the rotation by the remainder is written out. Decryption
reuses the encryption path on a reversed subkey array rather than repeating
the rounds with the indices reversed, which is what RFC 3713 says
decryption is.

Measured under `orangec eval`, one 128-bit-key block costs between 55,000 and
58,000 steps to encrypt (176 lookups: 18 rounds of 8 and 4 F applications of
8 in the key schedule) and between 58,000 and 62,000 to decrypt; a 192- or
256-bit-key block costs between 75,000 and 81,000 to encrypt (240 lookups)
and between 81,000 and 87,000 to decrypt, the difference being the subkey
reversal. The eight vectors of `camellia.or` come to about 590,000 of the
1,048,576 steps that one file may use, leaving room for eight more 128-bit
blocks; nothing was split or dropped. Not expressed: constant-time behaviour
(the selection idiom is a specification of a lookup, not a claim about
leakage), any mode of operation, and any key size other than the three of
the RFC.

## Dissemination

### Files

- `camellia.or`: the key schedule, the data randomizing part for 18 and 24
  rounds, the F, FL, FLINV, S and P functions, encryption and decryption for
  128-, 192- and 256-bit keys, and eight vector pairs.

### Running

```console
orangec eval algorithms/camellia/camellia.or
python3 algorithms/verify.py algorithms/camellia
```

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `rfc3713_a_128` | RFC 3713, Appendix A | 128-bit key 0123456789abcdeffedcba9876543210, plaintext 0123456789abcdeffedcba9876543210, ciphertext 67673138549669730857065648eabe43 |
| `rfc3713_a_128_decrypt` | RFC 3713, Appendix A | the same ciphertext decrypted under the 128-bit key gives the plaintext |
| `rfc3713_a_192` | RFC 3713, Appendix A | 192-bit key 0123...3210 0011223344556677, same plaintext, ciphertext b4993401b3e996f84ee5cee7d79b09b9 |
| `rfc3713_a_192_decrypt` | RFC 3713, Appendix A | the same ciphertext decrypted under the 192-bit key gives the plaintext |
| `rfc3713_a_256` | RFC 3713, Appendix A | 256-bit key 0123...3210 00112233445566778899aabbccddeeff, same plaintext, ciphertext 9acc237dff16d76c20ef7c919e3a7509 |
| `rfc3713_a_256_decrypt` | RFC 3713, Appendix A | the same ciphertext decrypted under the 256-bit key gives the plaintext |
| `botan_camellia_128_case_2` | Botan `src/tests/data/block/camellia.vec`, section `[Camellia-128]`, second case | key 80000000000000000000000000000000, zero plaintext, ciphertext 6c227f749319a3aa7da235a9bba05a2c |
| `botan_camellia_256_case_3` | Botan `camellia.vec`, section `[Camellia-256]`, third case | key 0000000000000200 followed by 24 zero bytes, zero plaintext, ciphertext e18b0cb1980124504b46a46a6f4273f3 |

The RFC 3713 Appendix A values are stated in the RFC itself; they also open
each section of Botan's `camellia.vec`, from which they were copied here, and
the `cryptography` package (Camellia in ECB mode, over OpenSSL) confirms all
eight ciphertexts. The three decryption pairs state the RFC's plaintext as
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
  which is why the Orange file carries only s1. The packed `Word[64]^32`
  literal was generated by script and unpacked again to confirm it.
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

This entry is a reference evaluation of RFC 3713 under `orangec eval`. It
makes no constant-time, side-channel, performance or certification claim,
and it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

None that prevented anything. This rendering keeps seven rotation specs and a
32-word selection for each s1 lookup (about 290 steps, so a block costs about
55,000 to 87,000 steps and one file holds roughly twelve blocks). A shift
amount may be computed, and a byte may index a table of 256 entries; neither
shape is a limit of the current language. A spec returns one value, so the
26 or 34 subkeys travel in one flat array whose layout the comments state.
