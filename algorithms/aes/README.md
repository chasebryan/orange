# AES

AES, the Advanced Encryption Standard, is the block cipher Rijndael of Joan
Daemen and Vincent Rijmen (1998), selected by NIST in 2000 and published as
[FIPS 197](https://doi.org/10.6028/NIST.FIPS.197-upd1) in 2001, with an
editorial update in 2023. It encrypts 128-bit blocks under keys of 128, 192 or
256 bits in 10, 12 or 14 rounds of a substitution-permutation network over the
field GF(2^8). [NIST SP 800-38A](https://doi.org/10.6028/NIST.SP.800-38A)
(2001) defines the five confidentiality modes that turn the block cipher into
an encryption scheme for messages: ECB, CBC, CFB, OFB and CTR. AES is the
block cipher of TLS, IPsec, SSH, disk encryption and most hardware, and it is
the current standard: no practical attack on the full cipher is known as of
2026.

## Analysis

### Structure

The cipher works on a state of sixteen bytes seen as a 4 x 4 array,
`s[r, c] = in[r + 4c]` (FIPS 197 section 3.4). In Orange the state is
`Word[8]^16` in that column-major order, so a block of input bytes is the
state and no transposition is written. A round is four transformations:
SubBytes (the S-box on every byte), ShiftRows (row `r` rotated left by `r`),
MixColumns (each column multiplied by the fixed polynomial
`{03}x^3 + {01}x^2 + {01}x + {02}` over GF(2^8)) and AddRoundKey (the round
key added). The last round omits MixColumns. The key schedule (section 5.2)
expands the key into `4(Nr + 1)` 32-bit words by a recurrence of period `Nk`
that applies RotWord, SubWord and a round constant every `Nk` words, and
SubWord alone at the half period when `Nk = 8`.

| Standard section | Orange spec |
| --- | --- |
| FIPS 197 section 4.2.1, multiplication by `x` | `xtime` |
| FIPS 197 section 5.1.1, SubBytes and the S-box (Table 4) | `sbox`, `lookup`, `byte_at`, `sub_bytes` |
| FIPS 197 section 5.1.2, ShiftRows | `shift_rows` |
| FIPS 197 section 5.1.3, MixColumns | `mix_column`, `mix_columns` |
| FIPS 197 section 5.1.4, AddRoundKey | `add_round_key` |
| FIPS 197 section 5.1, Algorithm 1, Cipher | `cipher` |
| FIPS 197 section 5.2, Algorithm 2, KeyExpansion (Table 5, Rcon) | `rot_word`, `sub_word`, `rcon`, `key_expansion_128`, `key_expansion_192`, `key_expansion_256` |
| FIPS 197 section 5.3.1 to 5.3.3, the inverse transformations (Table 6) | `inv_shift_rows`, `inv_sub_bytes`, `inv_sbox`, `inv_mix_column`, `inv_mix_columns` |
| FIPS 197 section 5.3, Algorithm 3, InvCipher | `inv_cipher` |
| FIPS 197 section 5, AES-128, AES-192, AES-256 and their inverses | `aes128`, `aes192`, `aes256`, `aes128_inverse`, `aes192_inverse`, `aes256_inverse` |
| SP 800-38A section 6.1, ECB | `ecb_encrypt` |
| SP 800-38A section 6.2, CBC | `cbc_encrypt` |
| SP 800-38A section 6.3, CFB with `s = 128` | `cfb128_encrypt` |
| SP 800-38A section 6.4, OFB | `ofb_encrypt` |
| SP 800-38A section 6.5, CTR, and Appendix B.1, the incrementing function | `ctr_encrypt`, `increment` |

`cipher(input, nr, w)` and `inv_cipher(input, nr, w)` take the number of
rounds `Nr` as the standard's algorithms do, and `aes128`, `aes192` and
`aes256` are `cipher` with `Nr = 10`, `12` and `14` under the schedule of
their key, which is how the 2023 update names the three ciphers. The modes
take the block cipher `CIPH_K` of SP 800-38A as the pair `(nr, w)`, so a
mode under AES-256 is the same spec as under AES-128 with a different
schedule; they are written for two-block messages as the equations of
section 6 with `n = 2`.

### Security status

AES has been the most analysed cipher in public cryptography for a quarter
century, and the full cipher stands. The best known key-recovery attacks in
the single-key setting are the biclique attacks of Bogdanov, Khovratovich and
Rechberger (2011), at about 2^126.1 operations for AES-128, 2^189.7 for
AES-192 and 2^254.4 for AES-256: a factor of three to five below exhaustive
search, with no practical consequence. Attacks that are materially faster
than brute force reach 7 rounds of AES-128, 8 of AES-192 and 9 of AES-256
(meet-in-the-middle and impossible-differential lines: Demirci and Selcuk
2008; Dunkelman, Keller and Shamir 2010; Derbez, Fouque and Jean 2013),
leaving 3, 4 and 5 rounds of margin.

In the related-key setting, Biryukov and Khovratovich (2009) gave key
recovery on the full AES-256 in about 2^99.5 time and data under four related
keys, and on the full AES-192 in about 2^176, after the related-key
distinguisher of Biryukov, Khovratovich and Nikolic (2009). These attacks
need encryptions under keys that differ from the target by differences the
attacker chooses. Keys drawn at random or derived through a key derivation
function have no such relation, so they do not affect AES as protocols use
it; they do show that the AES-256 key schedule is weaker than its key length
suggests, which matters where AES is used as a building block of a hash
function or with attacker-influenced keys. Against a quantum adversary,
Grover's algorithm halves the effective key length, and NIST's
post-quantum categories take AES-128 exhaustive search as their floor.

Implementations of AES from lookup tables leak through the cache. Bernstein
(2005) recovered an AES key across a network from the timing of OpenSSL's
table implementation, and Osvik, Shamir and Tromer (2006) recovered keys in
seconds to minutes from a process sharing a cache with the victim. That is
why deployed AES uses hardware instructions (AES-NI, ARMv8 Crypto) or
bitsliced software (Kasper and Schwabe 2009). The tables in this entry are
the standard's tables, written as packed words and read by a selection over
all of them; the entry is a reference evaluation under `orangec eval` and
makes no constant-time claim, and nothing about a compiled artifact follows
from it.

The modes have their own conditions. ECB encrypts equal blocks to equal
ciphertext blocks, so it hides neither repetition nor structure and is not
suitable for general use; NIST's review of the SP 800-38 series (NIST IR 8459,
initial public draft, 2023) took up whether it should stay approved for
general use at all. CBC needs an unpredictable IV
(Rogaway 2011; the BEAST attack of Duong and Rizzo 2011 exploited predictable
IVs in TLS 1.0), and a CBC decryptor that reveals whether padding was valid
gives a padding oracle that decrypts any ciphertext with about 128 queries per
block (Vaudenay 2002, followed by Lucky Thirteen, Al Fardan and Paterson
2013, and POODLE, Moller, Duong and Kotowicz 2014). This entry pads nothing:
its messages are whole blocks. OFB and CFB require the IV to be a nonce, and
CTR requires that no counter block be used twice under one key; a repeat in
any of the three leaks the exclusive-or of the two plaintexts. All five modes
are confidentiality only, with no integrity, which is why authenticated
modes (SP 800-38D, GCM) replaced them in protocols. SP 800-38A remains the
current NIST specification of these five modes, with the 2010 addendum that
adds ciphertext stealing to CBC.

### What the Orange rendering shows

Every data-dependent choice of the cipher is a table lookup: the S-box in
SubBytes and in SubWord of the key schedule, 200 lookups per AES-128 block
and 276 per AES-256 block. Nothing else depends on the data: ShiftRows is a
fixed reindexing, MixColumns and its inverse are `xtime` chains and
exclusive-ors, and the key schedule's branches are on the word index, which
is static. A byte may index a table of 256 entries. This rendering still
writes `sbox[x]` as a selection: the 256 entries are packed eight to a
`Word[64]` so that every literal reads like a row of the standard's table,
`lookup` walks the 32 words comparing `x >> 3` with each position, and
`byte_at` picks the byte `x & 7`.
One lookup costs 295 steps (measured), and the lookups are about 80 percent
of a block: with a given key schedule an AES-128 block costs about 58,000
steps and an AES-256 block about 81,000; `key_expansion_128` costs about
16,000 and `key_expansion_256` about 22,000; an AES-128 inverse block about
65,000, InvMixColumns being the costlier half.

An `Int` is not an index, and a slice bound is a literal or a loop index.
The round key of round `round` is `w[4 round .. 4 round + 3]`, and `nr` is
an `Int`, so the round index is the loop index: `cipher` loops over rounds
1 through 14 and lets the rounds above `Nr` pass the state through, and
`inv_cipher` counts a loop index up and takes `round = 14 - j`. The schedule
is `Word[32]^60` for all three key sizes, with AES-128 and AES-192 leaving
the tail at zero, and the recurrence `w[i] = w[i - Nk] ^ temp` is written
once per `Nk`, because `w[i - nk]` with `nk: Int` is rejected. A size is an
index, including `w[i - nk]` when `nk` is a size and `i` is a loop index;
this file does not use one.

The budget sized the vectors. The seven FIPS 197 cases of `aes.or` cost about
594,000 of the 1,048,576 steps a file has (measured with a filler spec). The
modes cannot share that file, so `aes-modes.or` carries its own copy of the
forward cipher and reproduces the first two blocks of each of six SP 800-38A
examples at about 817,000 steps; a third block of each would add about
371,000 and does not fit, and the four-block originals would need two more
files.

Not expressed: the modes' decryption direction (CBC and ECB decryption use
InvCipher; CFB, OFB and CTR decryption reuse the forward cipher), since a
spec without a vector would be dead code and its vectors do not fit the
budget; messages of other lengths, since an array's length is part of its
type (a size parameter covers a finite family of lengths; these modes do not
use one); and the Equivalent Inverse Cipher
of FIPS 197 section 5.3.5, which is an implementation arrangement rather than
a different function.

## Dissemination

### Files

- `aes.or`: module `aes`, the cipher of FIPS 197. The packed S-box and
  inverse S-box, the four transformations and their inverses, `xtime`,
  KeyExpansion for `Nk = 4, 6, 8`, `cipher` and `inv_cipher`, the six
  AES-128/192/256 entry points, and the vectors of Appendix B and C.
- `aes-modes.or`: module `aes_modes`, the five modes of SP 800-38A over
two-block messages, with the forward cipher of FIPS 197 repeated (a module
may `use` another; this file does not) and the first two blocks of six
Appendix F examples.

### Running

```console
orangec eval algorithms/aes/aes.or
orangec eval algorithms/aes/aes-modes.or
python3 algorithms/verify.py algorithms/aes
```

`eval` prints every parameterless spec, including the tables (`sbox`,
`inv_sbox` and `rcon` in `aes.or`, `sbox` and `rcon` in `aes-modes.or`); the pairs
below are the vectors.

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `fips197_c1_aes128` | FIPS 197, Appendix C.1, via the OpenSSL file | AES-128, plaintext 00112233...eeff, key 000102...0f |
| `fips197_c1_aes128_inverse` | FIPS 197, Appendix C.1 (INVERSE CIPHER), via the OpenSSL file | the C.1 ciphertext back to the plaintext |
| `fips197_c2_aes192` | FIPS 197, Appendix C.2, via the OpenSSL file | AES-192, the same plaintext, key 000102...17 |
| `fips197_c2_aes192_inverse` | FIPS 197, Appendix C.2 (INVERSE CIPHER), via the OpenSSL file | the C.2 ciphertext back to the plaintext |
| `fips197_c3_aes256` | FIPS 197, Appendix C.3, via the OpenSSL file | AES-256, the same plaintext, key 000102...1f |
| `fips197_c3_aes256_inverse` | FIPS 197, Appendix C.3 (INVERSE CIPHER), via the OpenSSL file | the C.3 ciphertext back to the plaintext |
| `fips197_b_aes128` | FIPS 197, Appendix B; expected value from the Python `cryptography` oracle | AES-128, input 3243f6a8..., key 2b7e1516... |
| `sp800_38a_f_1_1_ecb_aes128` | SP 800-38A, Appendix F.1.1, via the OpenSSL file | ECB-AES128.Encrypt, blocks 1 and 2 |
| `sp800_38a_f_2_1_cbc_aes128` | SP 800-38A, Appendix F.2.1, via the OpenSSL file | CBC-AES128.Encrypt, blocks 1 and 2 |
| `sp800_38a_f_3_13_cfb128_aes128` | SP 800-38A, Appendix F.3.13, via the OpenSSL file | CFB128-AES128.Encrypt, blocks 1 and 2 |
| `sp800_38a_f_4_1_ofb_aes128` | SP 800-38A, Appendix F.4.1, via the OpenSSL file | OFB-AES128.Encrypt, blocks 1 and 2 |
| `sp800_38a_f_5_1_ctr_aes128` | SP 800-38A, Appendix F.5.1, via the OpenSSL file | CTR-AES128.Encrypt, blocks 1 and 2 |
| `sp800_38a_f_5_5_ctr_aes256` | SP 800-38A, Appendix F.5.5, via the OpenSSL file | CTR-AES256.Encrypt, blocks 1 and 2 |

"The OpenSSL file" is `test/recipes/30-test_evp_data/evpciph_aes_common.txt`
of the OpenSSL repository, which transcribes these cases of the two standards
block by block; every one of its values used here was also checked against
the Python `cryptography` package (see Provenance below).

Blocks 3 and 4 of the six SP 800-38A examples are not evaluated, for the
budget reason given above; their plaintexts are the standard's blocks 3 and 4
and nothing else changes, so a reader who wants them can extend the message
type to `Word[8]^64` in a third file. The ECB examples for AES-192 and AES-256 (F.1.3,
F.1.5) are covered in substance by the FIPS 197 cases; the remaining
Appendix F examples are not reproduced.

### Provenance and claims

The standards' own sites are not reachable from the machine that wrote this
entry, so every constant and vector was taken from a fetched file or an
oracle and cross-checked by script, never transcribed by eye. The S-box was
computed from its definition in FIPS 197 section 5.1.1 (the inverse in
GF(2^8) modulo `x^8 + x^4 + x^3 + x + 1`, then the affine map with constant
`0x63`) and compared entry by entry with the table in the tiny-AES-c
reference implementation; the inverse S-box is its inverse permutation, also
compared; the round constants are the powers of `x` in the same field. The
packed `Word[64]` literals were generated by the script from those tables.
The FIPS 197 Appendix C and SP 800-38A Appendix F expected values are the
entries of OpenSSL's `test/recipes/30-test_evp_data/evpciph_aes_common.txt`,
which carries the FIPS 197 C.1 to C.3 cases and the SP 800-38A F.1 to F.5
examples block by block; the Appendix B output was produced by the Python
`cryptography` package (AES in ECB mode) because the vector file omits that
case. A Python reference of the cipher and the five modes written for this
entry agrees with `cryptography` and with all 84 whole-block AES entries of
the OpenSSL file in the five modes, and served as the oracle for
intermediate values while the Orange was written.

This entry is a reference evaluation of a specification under `orangec
eval`: it shows that the Orange text computes the standards' values on the
cases listed. It makes no constant-time, side-channel, performance or
certification claim, it is not an implementation anyone should deploy, and
it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- A module may `use` another. `aes-modes.or` still repeats about 200 lines
  of `aes.or` (the forward cipher and its tables).
- A byte may index a table of 256 entries. The lookup is still a 32-way
  selection at 295 steps, which makes a block cost 58,000 to 81,000 steps
  and limits the modes file to two blocks of each example. `nr` and `nk` are
  `Int` values, and an `Int` is not an index, so `cipher` and `inv_cipher`
  iterate over 14 rounds for every key size, the schedule is sized for
  AES-256, and the key expansion recurrence is written once per `Nk`.
- A size parameter covers a finite family of lengths. The modes are still
  fixed to two-block messages (`Word[8]^32`).
- The step budget per file (1,048,576) kept the modes' decryption direction
  and blocks 3 and 4 of the Appendix F examples out of the entry.
