# SM4 (GB/T 32907-2016)

SM4 is a 128-bit block cipher with a 128-bit key, designed in China by a group
led by Lu Shuwang and published in January 2006 as SMS4, the cipher of the
WAPI wireless standard; it became the industry standard GM/T 0002-2012 and
then the national standard GB/T 32907-2016, "Information security technology:
SM4 block cipher algorithm" (2016). The IETF draft
[draft-ribose-cfrg-sm4](https://datatracker.ietf.org/doc/html/draft-ribose-cfrg-sm4)
describes it in English, and
[RFC 8998](https://www.rfc-editor.org/rfc/rfc8998) (2021) defines the TLS 1.3
cipher suites TLS_SM4_GCM_SM3 and TLS_SM4_CCM_SM3 that use it. SM4 is
the block cipher of China's commercial cryptography, is in ISO/IEC 18033-3
through its 2021 amendment, and ships in OpenSSL, Botan and the Linux kernel,
with dedicated instructions in Armv8.2 and in recent Intel processors. It is
a current standard: no attack on the
full 32 rounds is published, and the best reduced-round results stop at 23
rounds.

## Analysis

### Structure

SM4 is a 32-round unbalanced Feistel network on four 32-bit words. The
plaintext is (X0, X1, X2, X3), read big-endian; round i computes
X_{i+4} = F(X_i, X_{i+1}, X_{i+2}, X_{i+3}, rk_i) with the round function
F(X0, X1, X2, X3, rk) = X0 xor T(X1 xor X2 xor X3 xor rk); and the ciphertext
is the reverse transformation R(X32, X33, X34, X35) = (X35, X34, X33, X32).
The composite permutation T is the nonlinear transformation tau, which sends
each of the four bytes of its input through one 8-bit S-box, followed by the
linear transformation L(B) = B xor (B <<< 2) xor (B <<< 10) xor (B <<< 18)
xor (B <<< 24). One word is replaced per round and the other three shift
down; the reversal R at the end makes decryption the same procedure with the
round keys in the reverse order (clause 7.2), which is the only difference
between `encrypt` and `decrypt` in the file.

The key expansion (clause 7.3) is the same Feistel structure run on the key:
(K0, K1, K2, K3) is the key xored with the system parameter FK, then for i
from 0 to 31, rk_i = K_{i+4} = K_i xor T'(K_{i+1} xor K_{i+2} xor K_{i+3} xor
CK_i). T' is T with the linear map replaced by L'(B) = B xor (B <<< 13) xor
(B <<< 23), and the same S-box; the fixed parameters CK_i are the bytes
(4i + j) * 7 mod 256, j = 0 to 3, written out in the file as the standard
lists them and derived from that formula in a comment.

The Orange file follows the standard clause by clause. A block and a key are
`Word[8]^16`, read as four big-endian words with `as big Word[32]^4` and
written back with `as big Word[8]^16`; the round keys are a `Word[32]^32`;
the four words in flight through the rounds are a `Word[32]^4` accumulator;
the 36 words K0 through K35 of the key expansion are one `Word[32]^36` array
whose last 32 entries, the slice `k[4..36]`, are the round keys. The S-box is
the standard's table as a `Word[8]^256`, sixteen `hex"..."` rows of sixteen
bytes, and FK and CK are `hex"..."` words read with `as big`.

| Standard clause | Orange spec |
| --- | --- |
| 6.1, the round function F | `round_f` |
| 6.2, the nonlinear transformation tau and its S-box | `tau`, `sbox` |
| 6.2, the linear transformation L, and T = L o tau | `linear_l`, `composite_t` |
| 7.1, encryption: 32 rounds and the reverse transformation R | `rounds`, `encrypt` |
| 7.2, decryption with the round keys reversed | `decrypt` |
| 7.3, the key expansion with FK, CK, L' and T' | `key_expansion`, `system_parameter_fk`, `fixed_parameters_ck`, `linear_l_prime`, `composite_t_prime` |
| Appendix A, example 1 | the tests `GB/T 32907-2016 Appendix A, example 1: encrypts` and `GB/T 32907-2016 Appendix A, example 1: decrypts` |

### Security status

SM4 has one nonlinear component, an 8-bit S-box that is the same in the data
path and in the key schedule. The standard gives the S-box as a table; Liu,
Ji, Hu, Ding and Lv (with Pyshkin and Weinmann, ACISP 2007) showed that it is
an affine map, then inversion in GF(2^8) modulo x^8 + x^7 + x^6 + x^5 + x^4 +
x^2 + 1, then the same affine map again, with the affine map x -> A x + c for
A the circulant matrix with first row 11010011 and c = 0xd3. That is the
construction of the AES S-box with a different field polynomial and different
affine maps, so the two S-boxes are affine equivalent and share their
cryptographic parameters: differential uniformity 4, nonlinearity 112,
algebraic degree 7, and the low-degree implicit equations that Courtois and
Pieprzyk (2002) proposed to exploit. The same equivalence is why SM4 can be
computed with AES hardware instructions and with GF(2^8) affine instructions
(GFNI): Botan's test file for SM4 lists `aesni`, `armv8aes` and `gfni` among
the CPU features it exercises. Algebraic attacks based on the equations
(Erickson, Ding and Christensen, 2009) have not scaled past a handful of
rounds.

The cryptanalytic record, as of 2026, stops at 23 of the 32 rounds. The
first results, in 2007 and 2008, were rectangle and impossible-differential
attacks on 14 to 16 rounds (Lu, ICICS 2007; Toz and Dunkelman, ICICS 2008;
Zhang, Zhang and Wu, ACISP 2008). Linear and differential cryptanalysis then
reached 22 rounds in 2008 (Kim, Kim, Hong and Sung; Etrog and Robshaw, SAC
2008; Zhang, Wu, Feng and Su, ISPEC 2009), and 23 rounds by 2011: a
differential attack by Su, Wu and Zhang (Journal of Computer Science and
Technology, 2011) with about 2^118 chosen plaintexts and 2^126.7 encryptions,
and multidimensional linear attacks by Cho and Nyberg (2011) and by Liu and
Chen (2014) with data complexities near 2^122 to 2^127 known plaintexts.
Later work through the 2010s and early 2020s, much of it with automated
searches for differential and linear trails, has refined these complexities
and the bounds on the best trails; to our knowledge no published attack
reaches 24 rounds, and every attack on more than 22 rounds needs close to the
whole codebook and time close to exhaustive search. The margin is nine
rounds. No related-key, weak-key or structural attack on the full cipher is
known. With a 128-bit key and a
128-bit block, SM4 offers the same generic security as AES-128: Grover's
algorithm halves the effective key length against a quantum adversary, and
modes of operation meet the birthday bound after 2^64 blocks under one key.

Table implementations of SM4 leak through the cache exactly as table
implementations of AES do (Bernstein 2005; Osvik, Shamir and Tromer 2006);
deployed software uses the hardware instructions above or bitsliced or
GFNI-based code. The S-box in this entry is the standard's table, indexed
directly by the secret byte, which is that cache-timing pattern; the entry
makes no constant-time claim.

On standing: SMS4 was published in 2006 for WAPI (GB 15629.11), became
GM/T 0002-2012 and then GB/T 32907-2016 (issued August 2016, in force from
March 2017), and is the block cipher of the SM series that Chinese
regulation of commercial cryptography prescribes (the Cryptography Law in
force since 2020 and the GM/T standards under it). ISO/IEC
18033-3:2010/Amd 1:2021 added it beside the 128-bit block ciphers of that
standard, AES, Camellia and SEED. RFC 8998 (March 2021,
Informational) registers TLS_SM4_GCM_SM3 and TLS_SM4_CCM_SM3 for TLS 1.3 with
IANA's "Recommended" column set to N, so they are available to
implementations that opt in and are not negotiated by the major browsers.
NIST has not approved SM4 for United States federal use, and it appears in
no IETF Standards Track protocol; outside China it is implemented widely and
used rarely. It carries the pitfalls of any 128-bit block cipher: a single
block, as evaluated here, is not a mode of operation, and its security in
use depends on the mode and on nonce and IV discipline (the GCM and CCM of
RFC 8998 fail entirely under a repeated nonce), not on the cipher.

### What the Orange rendering shows

The only data-dependent operation in SM4 is the S-box lookup, of which there
are 128 in the 32 rounds and 128 in the key expansion, 256 per block. In the
file it is written as the standard writes tau: `a as big Word[8]^4` splits
the word into a0 through a3, each byte indexes the 256-entry table as
`s[bytes[0]]`, and `as big Word[32]` joins the four results. A byte indexes
a table of 256 entries, so the checker proves every lookup in range with no
mask. `tau` takes the one word the standard gives it and builds the table
itself from `sbox()`; T, T' and F take exactly the standard's arguments. Every other index is a literal or a loop index, every rotation amount
is a literal, and there is no conditional in the file. Everything else is xor
and rotation on `Word[32]`, one to one with the standard's formulas for F, L
and L'. The Feistel rounds and the key expansion are both loops over an
array accumulator; the four words X_i through X_{i+3} travel as a
`Word[32]^4` whose new last entry is F, and the key expansion writes K_{i+4}
into a 36-word array at an index computed from the loop index. Byte order
is written once per conversion, with `as big`, where the standard reads a
key or block as words (Appendix A prints them big-endian) and where it
writes the ciphertext back.

Measured under `orangec test --stats`: one application of tau costs about
85 steps, of which about 56 build the S-box from its sixteen rows (a
parameterless spec is evaluated at each call) and the rest are the four
lookups and the two conversions; one round costs about 111; the key
expansion about 4,250 and the 32 rounds with R about 4,270. An encryption
test costs 8,531 steps and a decryption test 8,854 (reversing the round keys
adds about 320), and the six tests together use 51,832. Example 2 of
Appendix A, which applies the cipher 1,000,000 times to the same block under
the same key, would cost one key expansion and 32,000,000 rounds, about
3.6 x 10^9 steps, more than three times the largest budget `orangec` accepts
(2^30 = 1,073,741,824 steps), and is not reproduced. Not expressed: constant-time behaviour (the lookup is a
specification of the S-box, not a claim about leakage), any mode of
operation, and the SM4-GCM and SM4-CCM examples of RFC 8998 Appendix A,
which need GHASH and CBC-MAC around the cipher and are outside this entry's
scope.

## Dissemination

### Files

- `sm4.or`: the S-box as a 256-byte table, tau, L, L', T and T', the round
  function F, the key expansion with FK and CK, encryption and decryption,
  and the six tests below.

### Running

```console
orangec test algorithms/sm4/sm4.or
python3 algorithms/verify.py algorithms/sm4
```

### Vectors

Each row is a `test` block in `sm4.or`, comparing one block's output with
the published value.

| Test | Source | Case |
| --- | --- | --- |
| `GB/T 32907-2016 Appendix A, example 1: encrypts` | GB/T 32907-2016, Appendix A, example 1 | key 0123456789abcdeffedcba9876543210, plaintext 0123456789abcdeffedcba9876543210, ciphertext 681edf34d206965e86b3e94f536e4246 |
| `GB/T 32907-2016 Appendix A, example 1: decrypts` | GB/T 32907-2016, Appendix A, example 1 | the same ciphertext decrypted under the same key gives the plaintext |
| `Botan sm4.vec case 2, block 1: encrypts` | Botan `src/tests/data/block/sm4.vec`, section `[SM4]`, second case (the first "Random tests generated by GmSSL") | key 681edf34d206965e86b3e94f536e4246, first block f42131b002425b6f5cf52a810682a09d, ciphertext ec4b7b1757fee9ce455197e5bf9c3a90 |
| `Botan sm4.vec case 4: encrypts` | Botan `sm4.vec`, `[SM4]`, fourth case (the first "Random tests generated by Botan") | key fd0c5fbdb30201222daea461486b2853, plaintext 000102030405060708090a0b0c0d0e0f, ciphertext 73f102977f15599c61b15d13d3da6064 |
| `Botan sm4.vec case 4: decrypts` | Botan `sm4.vec`, `[SM4]`, fourth case | the same ciphertext decrypted under the same key gives the plaintext |
| `Botan sm4.vec case 5, block 1: encrypts` | Botan `sm4.vec`, `[SM4]`, fifth case | key e5ab67c47b9be83f1f37627532d91ab7, first block 000102030405060708090a0b0c0d0e0f, ciphertext 03a595d9af32aa810aa0beb758462f2c |

The example 1 values are stated in the standard and in the IETF draft; the
same case opens Botan's `sm4.vec` and the `SM4-ECB` case of OpenSSL's
`test/recipes/30-test_evp_data/evpciph_sm4.txt`, from which they were copied
here. The Botan cases are the first block of a multi-block ECB case or a
single block. The `cryptography` package (SM4 in ECB mode, over OpenSSL)
confirms all four ciphertexts; the two decryption tests state the source's
plaintext as their expected value.

### Provenance and claims

Neither GB/T 32907-2016 nor the IETF draft could be read from the machine
this entry was written on, so every constant was taken from a fetched
reference implementation and cross-checked by script against a second one,
never transcribed by eye or from memory:

- The S-box came from Botan's `src/lib/block/sm4/sm4.cpp` (`SM4_SBOX`, 256
  bytes) and was checked equal to OpenSSL's `crypto/sm4/sm4.c` (`SM4_S`) and
  to be a permutation. Its algebraic structure was verified by the same
  script: a search over circulant matrices and constants found exactly the
  affine map (first row 11010011, c = 0xd3) and the field polynomial 0x1f5
  named above, and the table equals A(I(A(x) + c)) + c for all 256 inputs.
  The first form's packed `Word[64]^32` literal was generated by script from
  the extracted bytes and unpacked again to confirm it.
- FK and CK came from Botan's `key_schedule` and agree with OpenSSL's `FK`
  and `CK`; CK was also recomputed from the formula (4i + j) * 7 mod 256 and
  found equal.
- A Python reference in the standard's vocabulary (MK, FK, CK, K_i, rk_i,
  X_i, F, T, T', tau, L, L', R), built from those extracted constants,
  reproduces all 22 cases of Botan's `sm4.vec` in both directions and agrees
  with the `cryptography` package on 500 random key and block pairs. The
  Orange specs were written from that reference and the fetched sources; the
  six vectors matched on the first evaluation.
- The clause numbers in the file's comments (6.1 for F, 6.2 for T, tau, L
  and the S-box, 7.1 to 7.3 for encryption, decryption and the key expansion
  with FK and CK, Appendix A for the examples) were checked against the
  outline of draft-ribose-cfrg-sm4-10 (April 2018), which states that its
  sections 1 to 7 map directly to the section numbers of GB/T 32907-2016 and
  which has examples 1 and 2 of the standard in its Appendix A.1. The text of
  the standard itself could not be read, so these numbers rest on the draft.
  One number is not confirmed: the file and the table give clause 7.3 for L'
  and T', as draft-crypto-sm4-00 does (its section 7.3.1), but
  draft-ribose-cfrg-sm4-10 defines them in its section 6.2; a reader with the
  standard should check that one.

The extraction, generation and measurement scripts were kept with the work
record and are not part of the repository.

The entry was then rewritten in the current language. Every expected value
is carried over byte for byte from the first form, where each was a
`<name>_expected` spec of bytes: the six tests state the same blocks as
`hex"..."` literals, compared by script with the old specs' evaluated
values, and no vector was added or dropped. The sixteen `hex"..."` rows of
the S-box were printed by script from the first form's packed words (and
checked again to be a permutation and to equal A(I(A(x) + c)) + c), and the
CK rows were printed from its CK words and checked against the formula
(4i + j) * 7 mod 256; FK is unchanged.

This entry is a reference evaluation of GB/T 32907-2016 under `orangec test`.
It makes no constant-time, side-channel, performance or certification claim,
and it is not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- Example 2 of Appendix A (10^6 encryptions of one block under one key) is
  about 3.6 x 10^9 steps, more than the largest step budget the evaluator
  accepts (2^30), and is recorded above as not reproduced.
