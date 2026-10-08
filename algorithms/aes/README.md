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
`s[r, c] = in[r + 4c]` (FIPS 197 section 3.4). In Orange the state is that
array: `type State = Row^4` with `type Row = Word[8]^4`, indexed `s[r][c]`
with the row first, as the standard writes it; `state` fills it from the
input column by column and `output` reads it back the same way. A round is
four transformations: SubBytes (the S-box on every byte), ShiftRows (row `r`
rotated left by `r`), MixColumns (each column multiplied by the fixed
polynomial `{03}x^3 + {01}x^2 + {01}x + {02}` over GF(2^8)) and AddRoundKey
(the round key added). The last round omits MixColumns. The key schedule
(section 5.2) expands the key into `4(Nr + 1)` 32-bit words by a recurrence
of period `Nk` that applies RotWord, SubWord and a round constant every `Nk`
words, and SubWord alone at the half period when `Nk = 8`.

| Standard section | Orange spec |
| --- | --- |
| FIPS 197 section 3.4, the state | `Row`, `State`, `state`, `output` |
| FIPS 197 section 4.2, multiplication by `x` and multiplication in GF(2^8) | `xtime`, `mul` |
| FIPS 197 section 5.1.1, SubBytes and the S-box (Table 4) | `sbox`, `sub_bytes` |
| FIPS 197 section 5.1.2, ShiftRows | `shift_rows` |
| FIPS 197 section 5.1.3, MixColumns | `mix`, `mix_columns` |
| FIPS 197 section 5.1.4, AddRoundKey | `add_round_key` |
| FIPS 197 section 5.1, Algorithm 1, Cipher | `cipher[nr]` |
| FIPS 197 section 5.2, Algorithm 2, KeyExpansion (Table 5, Rcon) | `rot_word`, `sub_word`, `rcon`, `key_expansion[nk]` |
| FIPS 197 section 5.3.1 to 5.3.3, the inverse transformations (Table 6) | `inv_shift_rows`, `inv_sbox`, `inv_sub_bytes`, `mix`, `inv_mix_columns` |
| FIPS 197 section 5.3, Algorithm 3, InvCipher | `inv_cipher[nr]` |
| FIPS 197 section 5, AES-128, AES-192, AES-256 and their inverses | `aes128`, `aes192`, `aes256`, `aes128_inverse`, `aes192_inverse`, `aes256_inverse` |
| SP 800-38A section 4.2, exclusive-or | `xor[n]` |
| SP 800-38A section 6.1, ECB | `ecb_encrypt`, `ecb_decrypt` |
| SP 800-38A section 6.2, CBC | `cbc_encrypt`, `cbc_decrypt` |
| SP 800-38A section 6.3, CFB with `s = 128` | `cfb128_encrypt`, `cfb128_decrypt` |
| SP 800-38A section 6.4, OFB | `ofb_output_blocks`, `ofb_encrypt`, `ofb_decrypt` |
| SP 800-38A section 6.5, CTR, and Appendix B.1, the incrementing function | `increment`, `ctr_output_blocks`, `ctr_encrypt`, `ctr_decrypt` |
| SP 800-38A Appendix F, the shared inputs of the examples | `example_key_128`, `example_key_256`, `example_iv`, `example_counter`, `example_plaintext` |

`key_expansion[nk]` takes a key of `4 Nk` bytes and returns the schedule of
exactly `4 Nk + 28 = 4 Nr + 4` words; `cipher[nr]` and `inv_cipher[nr]` take
the number of rounds `Nr` as a size, as the standard's algorithms take it as
a parameter, and a schedule of `4 Nr + 4` words. `aes128`, `aes192` and
`aes256` are `cipher[10]`, `cipher[12]` and `cipher[14]` under the schedule
of their key, which is how the 2023 update names the three ciphers. MixColumns
and InvMixColumns are one spec, `mix`, the product of each column with a
matrix whose rows are rotations of its first row: `{02} {03} {01} {01}` for
MixColumns and `{0e} {0b} {0d} {09}` for InvMixColumns, with the products
those of section 4.2.

The modes are in their own module, `aes_modes`, which reads the cipher with
`use aes;`. Each takes the block cipher `CIPH_K` of SP 800-38A as an AES key
schedule `w` of `Nr` rounds and a message of `n` whole blocks as `16 n`
bytes, both sizes (`nr` and `n`), so a mode under AES-256 is the same spec as
under AES-128 with a longer schedule. The modes are written as the equations
of section 6: ECB and CBC call `aes::cipher` and, to decrypt,
`aes::inv_cipher`; CBC and CFB carry the previous ciphertext block (the IV at
first) through the loop over blocks; OFB and CTR build their output blocks
`O_1` through `O_n` once and exclusive-or them with the plaintext to encrypt
and with the ciphertext to decrypt.

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
the standard's tables, read by the secret byte itself as `sbox[x]`, the
pattern those attacks exploit; the entry is a reference evaluation under
`orangec test` and makes no constant-time claim, and nothing about a
compiled artifact follows from it.

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

The data choose two things in the cipher. The S-box lookups are written
`sbox[s[r][c]]` in SubBytes and `sbox[a[i]]` in SubWord, a byte indexing the
256 bytes of Table 4 (and `inv_sbox[...]` in InvSubBytes): 200 lookups per
AES-128 block with its key schedule and 276 per AES-256 block. A `Word[8]`
index ranges over 0 through 255, so the checker proves every lookup in range
from the types alone. Besides the lookups, `mul(a, b)` tests each bit `k` of
`b` and adds `x^k a` when it is set; in MixColumns and InvMixColumns `b` is a
state byte, so those tests follow the data. The `xtime` steps inside `mul`
run on `a`, which there is a matrix constant, so `xtime`'s test of the top
bit branches the same way for every state. Everything else is fixed by the
round and position: ShiftRows and InvShiftRows read `s[r][(c + r) % 4]` and
`s[r][(c - r) % 4]`, the standard's formulas, with loop indices that the
checker bounds; the key schedule's branches are on the word index `i`; and
the round key of round `round` is the slice `w[4 round .. 4 round + 4]`.

Byte order is written where FIPS 197 fixes it: the key is read into words
with `key as big Word[32]^nk` (the first byte most significant, as in
section 5.2), AddRoundKey spreads word `w[4 round + c]` down column `c` with
`k[c] as big Word[8]^4`, SubWord splits and rejoins a word the same way, and
the round constants are Table 5's words, printed as hex and read with
`as big`. RotWord on a big-endian word is `w <<< 8`. In the modes, the CTR
counter block is a 128-bit big-endian integer (`t as big Int`), and
`next as big Word[8]^16` keeps the incremented value modulo 2^128, which is
the standard incrementing function of Appendix B.1 with `m = 128`.

Lengths are sizes. `key_expansion[nk]` has one instance per key length, and
`cipher[nr]` and `inv_cipher[nr]` one per number of rounds, so a schedule
has exactly its `4 Nr + 4` words and the round loop runs `Nr - 1` times;
InvCipher's loop counts `j` up and takes round `Nr - j`, because a loop runs
upward. The modes have one instance per pair `(nr, n)`, and each test names
neither: the checker picks `nr = 10` from the 44 words of an AES-128 schedule
(60 for AES-256 gives 14) and `n = 4` from the 64-byte message, so a schedule
or message of a length no instance takes is rejected before evaluation.
The modes reach the cipher across modules as `aes::cipher[nr](...)` and
`aes::inv_cipher[nr](...)`; the gate runs the tests of `aes.or` on their own.

Measured costs under `orangec test --stats`: one AES-128 block costs about
149,000 steps given its schedule, one AES-256 block about 214,000, the
inverse cipher the same as the forward; `key_expansion` costs about 2,400 steps
for a 128-bit key, 2,500 for 192 and 3,100 for 256. MixColumns is the
expensive step: `mix` forms its 64 products with the general multiplication
of section 4.2, eight `xtime` steps each, at about 15,500 steps a call,
between about 14,700 and 16,300 as the state bytes have fewer or more bits
set; the nine calls in an AES-128 block make up about 94 percent of it.
SubBytes, ShiftRows and AddRoundKey cost under 300 steps each, the S-box
table about 60 to build and a lookup a few steps. A four-block mode example costs about 598,000 steps
under AES-128 and 862,000 under AES-256. The seven tests of `aes.or` use
1,259,062 steps and the twelve of `aes-modes.or` 7,714,374.

Not expressed: a final partial block, which SP 800-38A allows in CFB, OFB
and CTR (the last `u` bits of the output block); CFB with segments of 1, 8
or 64 bits; messages of more than four blocks, which are a change to the
size range up to the limits under Gaps; and the Equivalent Inverse Cipher
of FIPS 197 section 5.3.5, which is an implementation arrangement rather
than a different function.

## Dissemination

### Files

- `aes.or`: module `aes`, the cipher of FIPS 197. The state as a 4 x 4
  array, the S-box and inverse S-box as the standard's tables, the four
  transformations and their inverses, `xtime` and multiplication in
  GF(2^8), KeyExpansion for every `Nk`, `cipher` and `inv_cipher` for every
  `Nr`, the six AES-128/192/256 entry points, and the vectors of Appendix B
  and C.
- `aes-modes.or`: module `aes_modes`, the five modes of SP 800-38A in both
  directions over messages of one to four blocks, using module `aes`, and
  twelve four-block examples of Appendix F.

### Running

```console
orangec test algorithms/aes/aes.or
orangec test algorithms/aes/aes-modes.or
python3 algorithms/verify.py algorithms/aes
```

`orangec eval` prints every parameterless spec: the tables (`sbox`,
`inv_sbox` and `rcon`) for `aes.or`, and the shared inputs of Appendix F
for `aes-modes.or`.

### Vectors

Each row is a `test` block comparing an output with the published value.

| Test | Source | Case |
| --- | --- | --- |
| `FIPS 197 C.1: AES-128 encrypts` | FIPS 197, Appendix C.1, via the OpenSSL file | AES-128, plaintext 00112233...eeff, key 000102...0f |
| `FIPS 197 C.1: AES-128 inverse cipher` | FIPS 197, Appendix C.1 (INVERSE CIPHER), via the OpenSSL file | the C.1 ciphertext back to the plaintext |
| `FIPS 197 C.2: AES-192 encrypts` | FIPS 197, Appendix C.2, via the OpenSSL file | AES-192, the same plaintext, key 000102...17 |
| `FIPS 197 C.2: AES-192 inverse cipher` | FIPS 197, Appendix C.2 (INVERSE CIPHER), via the OpenSSL file | the C.2 ciphertext back to the plaintext |
| `FIPS 197 C.3: AES-256 encrypts` | FIPS 197, Appendix C.3, via the OpenSSL file | AES-256, the same plaintext, key 000102...1f |
| `FIPS 197 C.3: AES-256 inverse cipher` | FIPS 197, Appendix C.3 (INVERSE CIPHER), via the OpenSSL file | the C.3 ciphertext back to the plaintext |
| `FIPS 197 B: the cipher example` | FIPS 197, Appendix B; expected value from the Python `cryptography` oracle | AES-128, input 3243f6a8..., key 2b7e1516... |
| `SP 800-38A F.1.1: ECB-AES128.Encrypt` | SP 800-38A, Appendix F.1.1; blocks 1 and 2 via the OpenSSL file, 3 and 4 recomputed | ECB-AES128.Encrypt, blocks 1 to 4 |
| `SP 800-38A F.1.2: ECB-AES128.Decrypt` | SP 800-38A, Appendix F.1.2; recomputed | ECB-AES128.Decrypt, blocks 1 to 4 |
| `SP 800-38A F.2.1: CBC-AES128.Encrypt` | SP 800-38A, Appendix F.2.1; blocks 1 and 2 via the OpenSSL file, 3 and 4 recomputed | CBC-AES128.Encrypt, blocks 1 to 4 |
| `SP 800-38A F.2.2: CBC-AES128.Decrypt` | SP 800-38A, Appendix F.2.2; recomputed | CBC-AES128.Decrypt, blocks 1 to 4 |
| `SP 800-38A F.3.13: CFB128-AES128.Encrypt` | SP 800-38A, Appendix F.3.13; blocks 1 and 2 via the OpenSSL file, 3 and 4 recomputed | CFB128-AES128.Encrypt, blocks 1 to 4 |
| `SP 800-38A F.3.14: CFB128-AES128.Decrypt` | SP 800-38A, Appendix F.3.14; recomputed | CFB128-AES128.Decrypt, blocks 1 to 4 |
| `SP 800-38A F.4.1: OFB-AES128.Encrypt` | SP 800-38A, Appendix F.4.1; blocks 1 and 2 via the OpenSSL file, 3 and 4 recomputed | OFB-AES128.Encrypt, blocks 1 to 4 |
| `SP 800-38A F.4.2: OFB-AES128.Decrypt` | SP 800-38A, Appendix F.4.2; recomputed | OFB-AES128.Decrypt, blocks 1 to 4 |
| `SP 800-38A F.5.1: CTR-AES128.Encrypt` | SP 800-38A, Appendix F.5.1; blocks 1 and 2 via the OpenSSL file, 3 and 4 recomputed | CTR-AES128.Encrypt, blocks 1 to 4 |
| `SP 800-38A F.5.2: CTR-AES128.Decrypt` | SP 800-38A, Appendix F.5.2; recomputed | CTR-AES128.Decrypt, blocks 1 to 4 |
| `SP 800-38A F.5.5: CTR-AES256.Encrypt` | SP 800-38A, Appendix F.5.5; blocks 1 and 2 via the OpenSSL file, 3 and 4 recomputed | CTR-AES256.Encrypt, blocks 1 to 4 |
| `SP 800-38A F.5.6: CTR-AES256.Decrypt` | SP 800-38A, Appendix F.5.6; recomputed | CTR-AES256.Decrypt, blocks 1 to 4 |

"The OpenSSL file" is `test/recipes/30-test_evp_data/evpciph_aes_common.txt`
of the OpenSSL repository, which transcribes these cases of the two standards
block by block; every one of its values used here was also checked against
the Python `cryptography` package (see Provenance below). "Recomputed" means
recomputed with two independent libraries for this rewrite, as described
below. Each decryption example takes the ciphertext of the matching
encryption example back to the shared plaintext of Appendix F.

The ECB examples for AES-192 and AES-256 (F.1.3 to F.1.6) are covered in
substance by the FIPS 197 cases. The CFB1 and CFB8 examples, the AES-192
examples of CBC, CFB128, OFB and CTR, and the AES-256 examples of CBC,
CFB128 and OFB are not reproduced.

### Provenance and claims

The standards' own sites are not reachable from the machine that wrote this
entry, so every constant and vector was taken from a fetched file or an
oracle and cross-checked by script, never transcribed by eye. The S-box was
computed from its definition in FIPS 197 section 5.1.1 (the inverse in
GF(2^8) modulo `x^8 + x^4 + x^3 + x + 1`, then the affine map with constant
`0x63`) and compared entry by entry with the table in the tiny-AES-c
reference implementation; the inverse S-box is its inverse permutation, also
compared; the round constants are the powers of `x` in the same field. The
packed `Word[64]` literals of the first form were generated by the script
from those tables. The FIPS 197 Appendix C and SP 800-38A Appendix F
expected values are the entries of OpenSSL's
`test/recipes/30-test_evp_data/evpciph_aes_common.txt`, which carries the
FIPS 197 C.1 to C.3 cases and the SP 800-38A F.1 to F.5 examples block by
block; the Appendix B output was produced by the Python `cryptography`
package (AES in ECB mode) because the vector file omits that case. A Python
reference of the cipher and the five modes written for this entry agrees
with `cryptography` and with all 84 whole-block AES entries of the OpenSSL
file in the five modes, and served as the oracle for intermediate values
while the Orange was written.

The entry was then rewritten in the current language. Every expected value
is carried over byte for byte from the first form, where each was a
`<name>_expected` spec of bytes: the seven FIPS 197 tests state the same
sixteen bytes, and each SP 800-38A encryption test states the first form's
32 bytes as its first two blocks. The S-box and inverse S-box rows, now
`hex"..."` lines read as `sbox[x]`, were printed by script from the first
form's packed words, and the S-box was checked once more against its
definition in section 5.1.1. The first form evaluated only blocks 1 and 2
of the six SP 800-38A examples, to fit its step budget; the rewrite extends
each to the four blocks Appendix F prints and adds the six decryption
examples of the same cases (F.1.2, F.2.2, F.3.14, F.4.2, F.5.2 and F.5.6).
Blocks 3 and 4 of every example, and every decryption result, were
recomputed with pycryptodome 3.24.0 and with `cryptography` 50.0.2, in each
mode and both directions; the two agreed with each other and, on blocks 1
and 2, with the first form's values.

This entry is a reference evaluation of a specification under `orangec
test`: it shows that the Orange text computes the standards' values on the
cases listed. It makes no constant-time, side-channel, performance or
certification claim, it is not an implementation anyone should deploy, and
it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- A size ranges over an interval, so `key_expansion[nk]` is defined for
  `Nk` from 4 through 8 and `cipher[nr]` and `inv_cipher[nr]` for `Nr` from
  10 through 14, including the values 5, 7, 11 and 13 that AES does not
  use; a size cannot be restricted to `{4, 6, 8}`. The tie `Nr = Nk + 6`
  is written by the callers (`aes128` calls `cipher[10]`), not by a type.
- A function has at most 256 instances, and the modes have one per pair
  of `Nr` and block count: the five values of `nr` leave room for messages
  of up to 51 blocks, and an arbitrary message length is not a type.
- No array has zero elements, so a mode cannot take the empty message.
