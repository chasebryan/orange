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

The entry is two files. `aes.or`, module `aes`, is the block cipher of
FIPS 197: the state is the standard's 4 x 4 array of bytes `s[r][c]`
(section 3.4), a word of the key schedule is an array of four bytes
`[a0, a1, a2, a3]` (section 5.2), and one key expansion and one cipher cover
every key length, with the size parameter `nk` (the key's length in words,
`Nk`) or `nr` (the number of rounds, `Nr = Nk + 6`). `aes-gcm.or`, module
`aes_gcm`, uses it and follows SP 800-38D section by section. A block is
`Word[8]^16`, a string of sixteen bytes, as the standard writes blocks: the
leftmost byte first and, within a byte, the most significant bit first.
Strings are their own lengths: `gctr`, `pad`, `bit_length` and
`pre_counter_block` take a string of `len` bytes for every `len` from 1
through 64, and `gcm_ae` and `gcm_ad` take the key, IV, plaintext or
ciphertext, and additional data as type parameters listing the lengths the
vectors use (keys of 16 and 32 bytes, IVs of 12 and 60, texts of 16 and 60,
additional data of 16 and 20).

| Standard section | Orange spec |
| --- | --- |
| FIPS 197 section 3.4, the state | `State`, `state`, `output` (`aes.or`) |
| FIPS 197 section 4.2, multiplication by `x` | `xtime` (`aes.or`) |
| FIPS 197 section 5.1.1, SubBytes and the S-box (Table 4) | `sbox`, `sub_bytes` (`aes.or`) |
| FIPS 197 section 5.1.2, ShiftRows | `shift_rows` (`aes.or`) |
| FIPS 197 section 5.1.3, MixColumns | `mix_columns` (`aes.or`) |
| FIPS 197 section 5.1.4, AddRoundKey | `add_round_key` (`aes.or`) |
| FIPS 197 section 5.1, Algorithm 1, Cipher (`Nr` = 10 or 14 here) | `cipher[nr]` (`aes.or`) |
| FIPS 197 section 5.2, Algorithm 2, KeyExpansion (`Nk` = 4 or 8 here), and Table 5 | `rot_word`, `sub_word`, `xor_word`, `rcon`, `key_expansion[nk]` (`aes.or`) |
| FIPS 197 section 5, AES-128 and AES-256 as Cipher over KeyExpansion | `encrypt[nk]` (`aes.or`) |
| SP 800-38D section 4.2.2, `X ^ Y` and `[len(X)]64` | `xor`, `bit_length[len]` |
| SP 800-38D section 6.2, the incrementing function | `inc32` |
| SP 800-38D section 6.3, Algorithm 1, the block product | `block_product` |
| SP 800-38D section 6.4, Algorithm 2, GHASH | `ghash[m]` |
| SP 800-38D section 6.5, Algorithm 3, GCTR | `keystream[K, n]`, `gctr[K, len]` |
| SP 800-38D section 7.1, the padding `0^s`, `0^u`, `0^v` | `pad[len]` |
| SP 800-38D section 7.1, Algorithm 4, step 2, `J0` | `pre_counter_block[len]` |
| SP 800-38D section 7.1, Algorithm 4, GCM-AE | `gcm_ae`; `gcm_ae_empty_a` (empty `A`), `gcm_ae_empty` (empty `P` and `A`) |
| SP 800-38D section 7.2, Algorithm 5, GCM-AD | `gcm_ad` |

GCM never inverts the block cipher, so `aes.or` carries the forward cipher
only. `gcm_ae` returns the pair `(C, T)` of step 7, `C` as long as `P`.
An Orange array has at least one element, so the empty string is not a
value: `gcm_ae_empty_a` is Algorithm 4 with `A` empty, whose step 5 hashes
`C || 0^u || 0^64 || [len(C)]64`, and `gcm_ae_empty` is Algorithm 4 with
`P` and `A` empty, whose `C` is empty (Algorithm 3, step 1) and whose result
is `T` alone. `gcm_ad` returns a pair too: the verdict `T = T'` of step 8,
and beside it the `P` of step 4, which a caller uses only when the verdict
is `true`; a `false` verdict is `FAIL`. The tag length is `t = 128`
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
with the key). NIST's review of the SP 800-38 series (NIST IR 8459, September
2024) describes the two-key ciphertext of the franking attack and recommends
reaffirming SP 800-38D with possible corrections; NIST announced in March 2024
that it will revise SP 800-38D, and the 2007 text remains the standard.

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
`orangec test` and makes no such claim.

**Status.** SP 800-38D (2007) is the current NIST specification and GCM is
the mandatory-to-implement AEAD of TLS 1.3 (RFC 8446 section 9.1). No
attack on the construction as specified, under distinct nonces, full tags
and the data limits, is known as of 2026.

### What the Orange rendering shows

Two things in GCM depend on data. In AES it is the S-box: 160 lookups per
AES-128 block and 40 more in its key schedule (224 and 52 for AES-256).
`sbox` is Table 4 as sixteen `hex"..."` rows, one row of the table per
line, and SubBytes is the index `box[s[r][c]]`, which the checker proves in
range because a byte indexes a table of 256 entries. In GHASH it is the
block product: `block_product` is the standard's Algorithm 1 as a
128-iteration loop over `(Z, V)`, each iteration testing one bit of `X` to
decide whether `V` joins `Z`, and the low bit of `V` to decide whether the
reduction `R` joins the shifted `V`. Both are data-dependent choices (the
hash key `H` is `Y`, and `X` is a function of the message, and the S-box
index is a function of the key and the data); the language allows them and
makes no timing claim, and a table-driven GHASH turns the product's
branches into the key-dependent memory reads of the security section above.

The bit order of SP 800-38D is made explicit rather than hidden in a
table. The standard numbers the bits of a block from the left,
`x_0 x_1 ... x_127`, and the leftmost bit is the coefficient of `x^0`: a
block is its polynomial read backwards. `block_product` reads each block as
two 64-bit words with `as big Word[64]^2`, so the standard's `x_i` is bit
`63 - (i mod 64)` of word `i / 64`, `V >> 1` (multiplication by `x`) is a
right shift of the pair, `LSB_1(V)` is bit 0 of the second word, and `R` is
written as the standard writes it, `hex"e1000000000000000000000000000000"`,
read the same way. The comment on `block_product` says this in the file, so
a reader who knows GF(2^128) in the usual order can see where the
reflection happens, which is where implementations of GCM most often go
wrong. The other byte orders are written where the standard fixes them:
`inc32` reads the last four bytes of a block with `as big Word[32]`, and
`bit_length` writes `[len(X)]64` with `as big Word[8]^8`.

Lengths are types. A 60-byte plaintext is a `Word[8]^60`, and every
function that a length changes has one instance per length: `gctr[K, len]`
encrypts `ceil(len / 16)` counter blocks with `keystream` and xors the
leading `len` bytes of the stream, which is the standard's
`MSB_len(X_n*)` on the last block; `pad[len]` places a string at the head of
a zero string of whole blocks; and `ghash[m]` takes the formatted input as
`m` blocks, which the checker computes from the join
`pad(a) ++ pad(c) ++ bit_length(a) ++ bit_length(c)` in `gcm_ae`, the step
5 of Algorithm 4 as the standard writes it. `gcm_ae` and `gcm_ad` take type
parameters rather than ranges of sizes because a function has at most 256
instances: ranges over every length from 1 through 64 for the IV, the
text and the additional data would have hundreds of thousands, while the
lists have sixteen. The checker picks every instance from the types of the
arguments, so no test writes a size or a type argument. The
96-bit case of step 2 is a branch on the IV's length, and since every
instance of `pre_counter_block` is checked, its first branch reads the
leading sixteen bytes of `IV || 0^31 || 1 || 0^96`, which for a 12-byte IV
are `IV || 0^31 || 1` exactly and which no other IV reaches.

`aes::encrypt[nk]` is the standard's `CIPHER(in, Nr, KEYEXPANSION(key))`,
so `CIPH_K` expands the key at every block. That keeps GCM's text the
standard's (the mode sees only `CIPH_K`) and costs the schedule once per
block, about a seventh of the block's steps.

Measured with `orangec test --stats`: an AES-128 block costs 22,936 steps,
3,322 of them the key expansion, and an AES-256 block 31,694, 4,104 of
them the expansion. A block product's cost depends on its data, since only
the chosen branch of each test runs: from about 6,300 steps (all-zero
blocks) to about 9,400 (all-one blocks), and about 7,900 on random blocks
and on Test Case 4's `H`. A GHASH block adds about 190 for the xor and the
slice. A GCM-AE with a 96-bit IV, `n` blocks of plaintext and `m` blocks of
formatted GHASH input costs `n + 2` block encryptions (`H`, the counter
blocks, the tag mask) and `m` GHASH blocks; a 60-byte IV adds five GHASH
blocks for `J0`. The tests cost 53,338 (Test Case 1), 85,464 (Test Case 2),
196,010 (Test Case 4), 235,904 (Test Case 6), 93,543 (Wycheproof tcId 2),
196,015 (GCM-AD on Test Case 4), 235,909 (GCM-AD on Test Case 6), 235,903
(the rejected tag), 70,623 (Test Case 13) and 248,544 (Test Case 16)
steps, 1,651,253 in all.

Not expressed: tag lengths below 128 bits (`MSB_t` for `t < 128`), the
length checks of Algorithm 5 step 1 and the limits of section 5.2.1 (the
inputs are the vectors' own), strings of more than 64 bytes, lengths that
the type lists of `gcm_ae` and `gcm_ad` do not name (each is one more entry
in a list), and GCM-AES-192: `aes.or` has the `Nk = 6` instance, but no
vector here exercises it and the GCM key lists name 16- and 32-byte keys.

## Dissemination

### Files

- `aes.or`: module `aes`, the forward cipher of FIPS 197. The S-box (Table
  4), `xtime`, the four transformations on the 4 x 4 state, `cipher[nr]`
  (Algorithm 1), `key_expansion[nk]` (Algorithm 2, Table 5) and
  `encrypt[nk]`. It has no tests of its own; `aes-gcm.or` exercises AES-128
  and AES-256 through it.
- `aes-gcm.or`: module `aes_gcm`, which uses `aes`. `inc32`,
  `block_product`, `ghash`, `keystream` and `gctr`, `pad`, `bit_length`,
  `pre_counter_block` for every IV length from 1 through 64 bytes,
  `gcm_ae` with its two empty-string forms, `gcm_ad`, and the ten tests
  below.

### Running

    orangec test --steps 1073741824 algorithms/aes-gcm/aes-gcm.or
    python3 algorithms/verify.py algorithms/aes-gcm

The tests need more than `orangec test`'s default budget of 1,048,576
evaluation steps (they use 1,651,253), so the command raises it, as
`verify.py` does.

`aes.or` has no tests; the gate checks it as the module the root uses.
`orangec eval` prints the parameterless specs, which are the two inputs that
Test Cases 4, 6 and 16 share.

### Vectors

Each row is a `test` block in `aes-gcm.or`. A GCM-AE test compares the pair
`(C, T)` with the published ciphertext and tag (the tag alone when `P` is
empty); a GCM-AD test compares the pair `(true, P)`, and the rejected tag
is the claim that the verdict is `false`.

| Test | Source | Case |
| --- | --- | --- |
| `GCM specification Test Case 1: AES-128, empty P and A` | GCM specification (McGrew and Viega 2005), Test Case 1; OpenSSL `evpciph_aes_common.txt`, GCM section | AES-128, zero key, zero 96-bit IV, empty P and A; the tag 58e2fcce...e7455a |
| `GCM specification Test Case 2: AES-128, one block of P, empty A` | GCM specification, Test Case 2; same file | AES-128, zero key and IV, one zero block of P, empty A |
| `GCM specification Test Case 4: AES-128, 60-byte P, 20-byte A` | GCM specification, Test Case 4; same file | AES-128, key feffe992..., IV cafebabe...f888, 60-byte P, 20-byte A |
| `GCM specification Test Case 6: AES-128, 60-byte IV` | GCM specification, Test Case 6; same file | as Test Case 4 with a 60-byte IV 9313225d...a637b39b, so `J0` is a GHASH |
| `Wycheproof aes_gcm_test.json tcId 2: AES-128` | Wycheproof `testvectors_v1/aes_gcm_test.json`, tcId 2 (flag `Ktv`) | AES-128, 96-bit IV, 16-byte msg, 16-byte aad, an independent source |
| `GCM specification Test Case 4: GCM-AD returns P` | GCM specification, Test Case 4, decrypted | GCM-AD on the Test Case 4 ciphertext and tag: `T = T'`, `true`, and the 60-byte plaintext d9313225... |
| `GCM specification Test Case 6: GCM-AD returns P` | GCM specification, Test Case 6, decrypted; OpenSSL file | GCM-AD with the 60-byte IV on the Test Case 6 ciphertext and genuine tag 619cc5ae...d050: `T = T'`, `true`, and the plaintext of Test Case 4; added in the rewrite |
| `OpenSSL evpciph_aes_common.txt: Test Case 6 with a changed tag fails` | OpenSSL `evpciph_aes_common.txt`, the `Operation = DECRYPT`, `Result = CIPHERFINAL_ERROR` row that follows Test Case 6 | Test Case 6's key, IV, A and C with the tag's last byte 0x50 changed to 0x51: `FAIL`, a `false` verdict |
| `GCM specification Test Case 13: AES-256, empty P and A` | GCM specification, Test Case 13; OpenSSL file | AES-256, zero key, zero 96-bit IV, empty P and A |
| `GCM specification Test Case 16: AES-256, 60-byte P, 20-byte A` | GCM specification, Test Case 16; OpenSSL file | AES-256, key feffe992...8308 twice, IV cafebabe...f888, 60-byte P, 20-byte A |

NIST's own GCM example file (`AES_GCM.pdf` on its page of example values) is a
different list, eighteen examples labelled Example #1 to #6 under each key
size, and it is not a source of any vector here. Every expected value above
was also produced by the Python `cryptography` package (`AESGCM.encrypt` and
`AESGCM.decrypt`, which raises `InvalidTag` for the tampered row) and agrees.

### Provenance and claims

The standards' own sites are not reachable from the machine that wrote this
entry, so every constant and vector came from a fetched file or an oracle
and was cross-checked by script, never transcribed by eye. The S-box was
parsed from the tiny-AES-c reference implementation (`aes.c`) and compared
entry by entry with its definition in FIPS 197 section 5.1.1 (the inverse
in GF(2^8) modulo `x^8 + x^4 + x^3 + x + 1`, then the affine map with
constant `0x63`); the round constants are the powers of `x` in that field;
the packed `Word[64]` literals and the byte arrays of the first form were
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

The entry was then rewritten in the current language, its four files, split
by the old step budget, folded into `aes-gcm.or` and the module `aes.or`.
Every expected value is carried over byte for byte from the first form,
and no vector was dropped. The first form stated each GCM-AE
result as one 80-byte buffer, the 64-byte ciphertext buffer, zero past the
plaintext's length, followed by the tag; the tests state `C` at its own
length and `T`, and a script checked that each old buffer is exactly the
new `C`, then zeros, then the new `T`. The two GCM-AD pairs on Test Case 4
(the verdict `true` and the plaintext buffer) became one test of
`(true, P)`, and the rejected tag's pair (`false`) the claim that the
verdict is false. Every input and expected byte string was printed by
script from the first form's arrays, and the S-box rows from its packed
words, checked again against the definition of section 5.1.1; the outputs
were recomputed once more with `cryptography`'s `AESGCM`. One test was
added: GCM-AD on Test Case 6 with its genuine tag, the accepting
counterpart of the rejected-tag row on the same path through the
GHASH-derived `J0`. Its inputs are Test Case 6's published key, IV, `A`,
`C` and `T`, and its expected plaintext, Test Case 6's `P`, was recomputed
with `cryptography`'s `AESGCM.decrypt` and with pycryptodome's
`decrypt_and_verify`, which agree.

This entry is a reference evaluation of a specification under
`orangec test`: it shows that the Orange text computes the standard's
values on the cases listed. It makes no constant-time, side-channel,
performance or certification claim, it is not an implementation anyone
should deploy, and it is not a corpus entry in the sense of The Orange Book
chapter 12.

## Gaps

- No array has zero elements, so the empty string is not a value. GCTR's
  step 1 is not written, and GCM-AE with an empty `A`, or with `P` and `A`
  both empty, is a spec of its own (`gcm_ae_empty_a`, `gcm_ae_empty`);
  GCM-AE with `P` empty and `A` not, and GCM-AD with either empty, would be
  further specs, which no vector here needs.
- A function has at most 256 instances and four size or type parameters,
  so `gcm_ae` and `gcm_ad` list the lengths of their vectors as types
  rather than ranging over every length; `gctr`, `pad`, `bit_length` and
  `pre_counter_block` cover 1 through 64 bytes, and `ghash` 1 through 7
  blocks. Longer strings would widen those ranges, up to that limit.
- Every instance of a sized spec is checked, including branches it never
  takes, so the 96-bit branch of `pre_counter_block` reads its IV through a
  padded copy, and a size range is contiguous, so `nk in 4..9` and
  `nr in 10..15` in `aes.or` also instantiate `Nk` = 5 and 7 and `Nr` = 11
  and 13, which name no AES variant and which nothing calls.
- A spec has one result type, with no sum of `P` and `FAIL`: `gcm_ad`
  returns the verdict and the plaintext together, and the caller enforces
  step 8.
