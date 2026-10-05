# AES-GCM

AES-GCM is the Galois/Counter Mode of David McGrew and John Viega (2004),
standardized by NIST in [SP 800-38D](https://doi.org/10.6028/NIST.SP.800-38D)
(2007), with AES of [FIPS 197](https://doi.org/10.6028/NIST.FIPS.197-upd1)
as the block cipher. It is authenticated encryption with associated data:
counter-mode encryption under a 32-bit block counter, and a Wegman-Carter
tag from GHASH, a polynomial hash over GF(2^128) keyed by the encryption of
the zero block. It is the bulk cipher of TLS 1.3 (`TLS_AES_128_GCM_SHA256`
is the suite every implementation must support), of IPsec ESP (RFC 4106),
SSH and QUIC, and it has hardware support on every mainstream processor.
As of 2026 it is the current NIST standard for authenticated encryption; its
known failures are conditions of use (a repeated nonce, a short tag, too
much data under one key, a key that must commit), not attacks on the
construction as specified.

## Analysis

### Structure

GCM is two functions of SP 800-38D section 7 over three primitives of
section 6. `GCTR_K(ICB, X)` (section 6.5) is counter mode: block `i` of `X`
is xored with `CIPH_K(CB_i)`, where `CB_1 = ICB` and each next counter block
is `inc32` of the last, the rightmost 32 bits incremented modulo 2^32 and
the other 96 unchanged (section 6.2). `GHASH_H(X)` (section 6.4) folds the
blocks of `X` as `Y_i = (Y_{i-1} ^ X_i) . H` with the product of section
6.3, the multiplication of GF(2^128) modulo `x^128 + x^7 + x^2 + x + 1`,
written bit by bit as the standard's Algorithm 1 with `R = 11100001 || 0^120`.
`GCM-AE_K(IV, P, A)` (section 7.1, Algorithm 4) computes the hash key
`H = CIPH_K(0^128)`, the pre-counter block `J0` (the IV followed by
`0^31 || 1` when the IV is 96 bits, and `GHASH_H(IV || 0^(s+64) || [len(IV)]64)`
otherwise), the ciphertext `C = GCTR_K(inc32(J0), P)`, the hash
`S = GHASH_H(A || 0^v || C || 0^u || [len(A)]64 || [len(C)]64)` and the tag
`T = MSB_t(GCTR_K(J0, S))`. `GCM-AD_K(IV, C, A, T)` (section 7.2,
Algorithm 5) recomputes `T'` from `C` and `A`, returns `P = GCTR_K(inc32(J0), C)`
when `T = T'` and `FAIL` otherwise.

The Orange files follow that order. A 128-bit block is `Word[64]^2`, the
first eight bytes in the first word, most significant byte first, so that
the standard's bit 0, the leftmost bit of the block, is the top bit of the
first word; `block_words` and `block_bytes` convert. Messages travel as a
64-byte buffer with the message length as an `Int` beside it, and the
additional data as a 32-byte buffer with its length. An array's length is
part of its type. A size parameter covers a finite family of lengths; these
files fix each buffer.

| Standard section | Orange spec |
| --- | --- |
| FIPS 197 section 4.2.1, multiplication by `x` | `xtime` |
| FIPS 197 section 5.1.1, SubBytes and the S-box (Table 4) | `s_box`, `lookup`, `byte_at`, `sub_bytes` |
| FIPS 197 section 5.1.2, ShiftRows | `shift_rows` |
| FIPS 197 section 5.1.3, MixColumns | `mix_column`, `mix_columns` |
| FIPS 197 section 5.1.4, AddRoundKey | `add_round_key` |
| FIPS 197 section 5.1, Algorithm 1, Cipher (`Nr = 10`; `Nr = 14` in `aes-gcm-256.or`) | `cipher` |
| FIPS 197 section 5.2, KeyExpansion (`Nk = 4`; `Nk = 8` in `aes-gcm-256.or`) | `rot_word`, `sub_word`, `rcon`, `key_expansion` |
| SP 800-38D section 6.2, the incrementing function | `inc32` |
| SP 800-38D section 6.3, Algorithm 1, the block product | `block_product` |
| SP 800-38D section 6.4, Algorithm 2, GHASH | `ghash` |
| SP 800-38D section 6.5, Algorithm 3, GCTR | `gctr`, `counter_block`, `xor_block` |
| SP 800-38D section 7.1, Algorithm 4, step 2, `J0` | `j0_96` (`aes-gcm.or`, `aes-gcm-256.or`); `pre_counter_block`, `j0_input` (`aes-gcm-iv.or`, `aes-gcm-ad.or`) |
| SP 800-38D section 7.1, Algorithm 4, steps 4 and 5, the input to GHASH | `ghash_input`, `ghash_blocks` |
| SP 800-38D section 7.1, Algorithm 4, GCM-AE | `gcm_ae` |
| SP 800-38D section 7.2, Algorithm 5, GCM-AD | `gcm_ad_verify`, `gcm_ad_plaintext` (`aes-gcm-ad.or`) |

GCM never inverts the block cipher, so the files carry the forward cipher
only. `gcm_ae` returns the 64-byte ciphertext buffer followed by the 16-byte
tag, `Word[8]^80`. A spec may return a tuple. GCM-AD is still two specs over
the same steps 2 and 3 of Algorithm 5: `gcm_ad_verify` computes `T'` (steps
5 to 7) and returns the verdict `T = T'` of step 8 as a `Bool`, and
`gcm_ad_plaintext` returns the `P` of step 4; a caller returns `P` when the
verdict is `true` and `FAIL` when it is `false`. The tag length is `t = 128`
throughout, so `MSB_t` is the identity and is not written.

### Security status

GCM's tag is a Wegman-Carter MAC whose universal hash is the polynomial
`GHASH_H` evaluated at the secret point `H`, encrypted with the one-time pad
`CIPH_K(J0)`. McGrew and Viega gave the proof in 2004; Iwata, Ohashi and
Minematsu (2012) found it wrong for IVs that are not 96 bits, where `J0`
itself comes from GHASH, and repaired it with a looser bound. That is one
reason SP 800-38D recommends 96-bit IVs (section 5.2.1.1) and the reason
the bounds below are stated for them. With a 128-bit tag, the probability
that one forgery attempt against a message of `n` blocks succeeds is about
`(n + 1) / 2^128`.

**Nonce reuse is catastrophic.** Encrypting two messages under one key and
one IV reuses the counter keystream, which exposes the xor of the two
plaintexts, and it reuses the tag mask `CIPH_K(J0)`, so the xor of the two
tags is a polynomial in `H` with known coefficients whose roots include
`H`. Joux (2006, the "forbidden attack" comment to NIST) recovers `H` from
that polynomial and then forges a valid tag for any ciphertext under that
key. GCM offers no nonce-misuse resistance at all; Bock, Zauner, Devlin,
Somorovsky and Jovanovic (2016) found servers on the public internet
repeating GCM nonces in TLS.

**Short tags.** Ferguson (2005, comment to NIST) showed that a tag of `t`
bits does not give `2^-t` forgery security: against messages of `2^n`
blocks a forgery succeeds with probability about `2^(n-t)`, and each
successful forgery leaks about `n` bits of `H`, so the second forgery is
easier than the first. Appendix C of SP 800-38D therefore restricts 32- and
64-bit tags to short messages and to a bounded number of invocations, and
128-bit tags are the norm. The weak-key classes of GHASH (Saarinen 2012,
cycling attacks; Handschuh and Preneel 2008; Procter and Cid 2013) belong to
the same family: they matter when many forgery attempts are affordable,
which a 128-bit tag denies.

**Data limits.** The 32-bit counter allows at most `2^32 - 2` blocks of
plaintext, about 64 GiB, in one invocation. When 96-bit IVs are chosen at
random, SP 800-38D section 8.3 limits a key to `2^32` invocations so that
the probability of a repeated IV stays below `2^-32`, and it recommends the
deterministic construction of section 8.2.1 (a device field and a counter)
where the invocation count would exceed that. Confidentiality degrades with
the total number of blocks encrypted under one key by the PRP-PRF switching
term, about `sigma^2 / 2^128` for `sigma` blocks, which is why TLS 1.3
limits AES-GCM to about `2^24.5` full-size records per key (RFC 8446 section
5.5, after Luykx and Paterson 2017). In the multi-user setting, an adversary
holding ciphertexts from many keys gains by the number of keys; Bellare and
Tackmann (2016) analysed GCM in TLS 1.3 and the per-connection secret IV
mask that TLS 1.3 xors into every nonce is the mitigation, with Hoang,
Tessaro and Thiruvengadam (2018) giving the tight bounds for that
nonce randomization.

**GCM does not commit to its key.** A ciphertext and tag that decrypt
validly under two different keys are easy to construct, because the tag is
a linear function of the ciphertext for a fixed `H` and the mask
`CIPH_K(J0)` is free. Grubbs, Lu and Ristenpart (2017) named the property
for message franking; Dodis, Grubbs, Ristenpart and Woodage (2018) turned
it into the "invisible salamanders" attack on Facebook's abuse reporting;
Len, Grubbs and Ristenpart (2021) built partitioning-oracle attacks on
password-based uses of GCM from ciphertexts valid under thousands of keys;
and Albertini, Duong, Gueron, Kolbl, Luykx and Schmieg (2022) collected the
abuses and the fixes (a padding check, or a key-commitment tag derived
with the key). NIST's review of the SP 800-38 series (NIST IR 8459, draft
2023) opened the question of revising the modes, key commitment among the
topics; the 2007 text remains the standard.

**The misuse-resistant alternative** is AES-GCM-SIV (Gueron, Langley and
Lindell; RFC 8452, 2019), which derives per-nonce keys, computes the tag
over the plaintext before encrypting, and uses it as the counter, so that a
repeated nonce reveals only whether two messages were equal. Its hash
POLYVAL is GHASH with the bit order of each block reversed, so that its
field arithmetic reads in the usual order and the reflection described
under the Orange rendering below disappears.

**Implementation.** GHASH computed from precomputed tables of multiples of
`H` (Shoup's 4-bit and 8-bit tables) indexes memory by the data, so the
cache leaks `H` the same way table-based AES leaks the key (Bernstein 2005;
Osvik, Shamir and Tromer 2006). Kasper and Schwabe (2009) gave the first
constant-time AES-GCM in software, bitsliced AES with a table-free GHASH;
deployed implementations use the carry-less multiply instruction
(Gueron and Kounavis 2010) and AES-NI. The bit-serial Algorithm 1 as this
entry writes it branches on every bit of the data and on every bit of `V`,
and is not constant time either; the entry is a reference evaluation under
`orangec eval` and makes no such claim.

**Status.** SP 800-38D (2007) is the current NIST specification and GCM is
the mandatory-to-implement AEAD of TLS 1.3 (RFC 8446 section 9.1). No
attack on the construction as specified, under distinct nonces, full tags
and the data limits, is known as of 2026.

### What the Orange rendering shows

Two things in GCM depend on data. In AES it is the S-box: 160 lookups per
AES-128 block and 40 more in its key schedule (224 and 52 for AES-256).
A byte may index a table of 256 entries. This rendering packs `s_box` eight
entries to a `Word[64]`, each literal a row of Table 4, and `lookup` walks
the 32 words comparing `x >> 3` with each position before `byte_at` picks
byte `x & 7`: 291 steps per lookup (measured). In GHASH it is the block
product: `block_product` is the standard's Algorithm 1 as a 128-iteration
loop over `[X, V, Z]`, each iteration testing one bit of `X` to decide
whether `V` joins `Z`, and the low bit of `V` to decide whether the
reduction `R` joins the shifted `V`. Both are branches on secret data (the
hash key `H` is `Y`, and `X` is a function of the message); a table-driven
GHASH turns them into the key-dependent memory reads of the security
section above.

The bit order of SP 800-38D is made explicit rather than hidden in a
table. The standard numbers the bits of a block from the left,
`x_0 x_1 ... x_127`, and the leftmost bit is the coefficient of `x^0`: a
block is its polynomial read backwards. With a block held as two words,
most significant byte first, the standard's `x_i` is the top bit of `X`
after `i` left shifts, the standard's `V >> 1` (multiplication by `x`) is a
right shift of the pair, `LSB_1(V)` is bit 0 of the second word, and
`R = 11100001 || 0^120` is `0xe100000000000000` in the first word. The
comment on `block_product` says this in the file, so a reader who knows
GF(2^128) in the usual order can see where the reflection happens, which
is where implementations of GCM most often go wrong.

Lengths are values. A 60-byte plaintext is a `Word[8]^64` buffer and the
`Int` 60, so `gctr` loops over the four possible blocks and a guard
`(16 * i) < len` skips the ones past the end, which also costs nothing; the
last, partial block keeps only its `len` leading bytes through the guard in
`xor_block`. `ghash_input` lays out `A || 0^v || C || 0^u || [len(A)]64 ||
[len(C)]64` in a seven-block buffer by comparing each block index with
`ceil(len(A) / 16)` and `ceil(len(C) / 16)`, and `word_at` selects a word
by value because the position of `C` in the buffer depends on `len(A)`;
`ghash` then folds the number of blocks that are in use.

The cost is the block cipher. Measured against the 1,048,576-step budget of
one file: an AES-128 block with the schedule hoisted costs about 70,000
steps and an AES-256 block about 95,000, the schedules about 15,000 and
21,000, and one block product about 7,300. A GCM-AE with a 96-bit IV, `n`
blocks of plaintext and `m` blocks of formatted GHASH input costs the
schedule plus `n + 2` block encryptions (`H`, the counter blocks, the tag
mask) plus `m` products; a 60-byte IV adds five products for `J0`. The
vectors were sized to that: Test Cases 1, 2 and 4 cost about 169,000,
246,000 and 491,000 steps, 907,000 together, which fills `aes-gcm.or`;
Test Case 6 (535,000) and Wycheproof case 2 (255,000) fill
`aes-gcm-iv.or` at 790,000; the GCM-AD verdict on Test Case 4 (209,000),
its plaintext (357,000) and the rejected tag (252,000) fill `aes-gcm-ad.or`
at 818,000; and Test Cases 13 and 16 under AES-256 (228,000 and 654,000)
fill `aes-gcm-256.or` at 882,000, where Test Case 14 (331,000) would exceed
the budget and was left out. Each figure was measured with a filler spec
(`measure.py` and `exact.py` in the entry's scratch work, not in the
repository).

Not expressed: tag lengths below 128 bits (`MSB_t` for `t < 128`), the
length checks of Algorithm 5 step 1 and the limits of section 5.2.1 (the
inputs are the vectors' own), plaintexts over 64 bytes, additional data
over 32 bytes and IVs over 64 bytes (the buffers are types, and other sizes
would be further specs), AES-192, and GCM-AD as one function returning
either `P` or `FAIL`, which a spec with one result type cannot; the two
specs recompute the schedule, `H` and `J0`, about 85,000 steps twice.

## Dissemination

### Files

- `aes-gcm.or`: module `aes_gcm`. AES-128 (the packed S-box, the four
  transformations, `xtime`, KeyExpansion for `Nk = 4`, `cipher`), `inc32`,
  `block_product`, `ghash`, `gctr`, `j0_96`, `ghash_input`, `gcm_ae`, and
  Test Cases 1, 2 and 4 of the GCM specification.
- `aes-gcm-iv.or`: the same AES-128 and primitives, with `J0` for an IV of
  any length up to 64 bytes (`pre_counter_block`, `j0_input`) so that an IV
  that is not 96 bits goes through GHASH, and `gcm_ae` over it. Test Case 6
  (a 60-byte IV) and Wycheproof `aes_gcm_test.json` tcId 2.
- `aes-gcm-ad.or`: the same, with GCM-AD (`gcm_ad_verify`,
  `gcm_ad_plaintext`) in place of GCM-AE. Test Case 4 decrypted and
  verified, and the OpenSSL decryption-error row that follows Test Case 6,
  rejected.
- `aes-gcm-256.or`: AES-256 (KeyExpansion for `Nk = 8`, `Nr = 14`) under
  the same primitives and `gcm_ae` with a 96-bit IV. Test Cases 13 and 16.

The three later files exist because the step budget of one file does not
hold their vectors beside the first file's; each is complete on its own,
and the shared text is identical across them.

### Running

```console
orangec eval algorithms/aes-gcm/aes-gcm.or
orangec eval algorithms/aes-gcm/aes-gcm-iv.or
orangec eval algorithms/aes-gcm/aes-gcm-ad.or
orangec eval algorithms/aes-gcm/aes-gcm-256.or
python3 algorithms/verify.py algorithms/aes-gcm
```

`eval` prints every parameterless spec, including `s_box` and `rcon`; the
pairs below are the vectors. A `gcm_ae` result is the 64-byte ciphertext
buffer, zero past the plaintext's length, followed by the 16-byte tag.

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `nist_gcm_test_case_1` | GCM specification (McGrew and Viega 2005), Test Case 1; OpenSSL `evpciph_aes_common.txt`, GCM section | AES-128, zero key, zero 96-bit IV, empty P and A; the tag 58e2fcce...e7455a (`aes-gcm.or`) |
| `nist_gcm_test_case_2` | GCM specification, Test Case 2; same file | AES-128, zero key and IV, one zero block of P, empty A (`aes-gcm.or`) |
| `nist_gcm_test_case_4` | GCM specification, Test Case 4; same file | AES-128, key feffe992..., IV cafebabe...f888, 60-byte P, 20-byte A (`aes-gcm.or`) |
| `nist_gcm_test_case_6` | GCM specification, Test Case 6; same file | as Test Case 4 with a 60-byte IV 9313225d...a637b39b, so `J0` is a GHASH (`aes-gcm-iv.or`) |
| `wycheproof_gcm_tc_2` | Wycheproof `testvectors_v1/aes_gcm_test.json`, tcId 2 (flag `Ktv`) | AES-128, 96-bit IV, 16-byte msg, 16-byte aad, an independent source (`aes-gcm-iv.or`) |
| `nist_gcm_test_case_4_verify` | GCM specification, Test Case 4, decrypted | GCM-AD on the Test Case 4 ciphertext and tag: `T = T'`, `true` (`aes-gcm-ad.or`) |
| `nist_gcm_test_case_4_plaintext` | GCM specification, Test Case 4, decrypted | GCM-AD on the Test Case 4 ciphertext: the 60-byte plaintext d9313225... (`aes-gcm-ad.or`) |
| `openssl_test_case_6_tampered_tag_verify` | OpenSSL `evpciph_aes_common.txt`, the `Operation = DECRYPT`, `Result = CIPHERFINAL_ERROR` row that follows Test Case 6 | Test Case 6's key, IV, A and C with the tag's last byte 0x50 changed to 0x51: `FAIL`, `false` (`aes-gcm-ad.or`) |
| `nist_gcm_test_case_13` | GCM specification, Test Case 13; OpenSSL file | AES-256, zero key, zero 96-bit IV, empty P and A (`aes-gcm-256.or`) |
| `nist_gcm_test_case_16` | GCM specification, Test Case 16; OpenSSL file | AES-256, key feffe992...8308 twice, IV cafebabe...f888, 60-byte P, 20-byte A (`aes-gcm-256.or`) |

The GCM specification's Test Cases 1 to 18 are the cases NIST's GCM example
file also carries. Every expected value above was also produced by the
Python `cryptography` package (`AESGCM.encrypt`, and `AESGCM.decrypt`
raising `InvalidTag` for the tampered row) and agrees.

### Provenance and claims

The standards' own sites are not reachable from the machine that wrote this
entry, so every constant and vector came from a fetched file or an oracle
and was cross-checked by script, never transcribed by eye. The S-box was
parsed from the tiny-AES-c reference implementation (`aes.c`) and compared
entry by entry with its definition in FIPS 197 section 5.1.1 (the inverse
in GF(2^8) modulo `x^8 + x^4 + x^3 + x + 1`, then the affine map with
constant `0x63`); the round constants are the powers of `x` in that field;
the packed `Word[64]` literals and every byte array in the files were
emitted by the script from those values. The GCM section of OpenSSL's
`test/recipes/30-test_evp_data/evpciph_aes_common.txt` carries the
specification's eighteen cases in order under a comment naming
`gcm-spec.pdf`, without individual labels and with three decryption-error
rows interleaved; the script numbered them by position after setting the
error rows aside and confirmed all eighteen, and the three rejections, with
`cryptography`. The Wycheproof case was read from
`testvectors_v1/aes_gcm_test.json`. A Python reference of AES-128, AES-256
and GCM written for this entry, mirroring the Orange specs step by step
(the same block product with the standard's bit order, the same
formatting), agrees with `cryptography` on all eighteen cases and on the 209
Wycheproof cases with 128- or 256-bit keys, a non-empty IV and a 128-bit
tag, including the invalid ones, and served as the oracle for intermediate
values (`H`, `J0`, each `Y_i`) while the Orange was written.

This entry is a reference evaluation of a specification under `orangec
eval`: it shows that the Orange text computes the standard's values on the
cases listed. It makes no constant-time, side-channel, performance or
certification claim, it is not an implementation anyone should deploy, and
it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- A byte may index a table of 256 entries. The S-box is still a 32-way
  selection at 291 steps per lookup, which is most of the 70,000 to 95,000
  steps of a block. An `Int` is not an index, so the number of GHASH blocks,
  the position of `C` in the formatted input and the counter block's index
  are handled by guards and by `word_at` and `counter_block` selections
  rather than by indexing.
- A size parameter covers a finite family of lengths. Plaintexts are still a
  64-byte buffer with a length, additional data a 32-byte buffer with a
  length, and IVs a 64-byte buffer with a length.
- The step budget per file (1,048,576): the ten vectors needed four files,
  Test Case 14 was left out of `aes-gcm-256.or`, and a file holds about a
  dozen AES-128 block encryptions in all, or eight AES-256.
- A spec may return a tuple. GCM-AD is still a `Bool` spec and a plaintext
  spec, each recomputing the schedule, `H` and `J0`.
- A module may `use` another. Each file still repeats about 230 lines of AES
  and the GCM primitives.
