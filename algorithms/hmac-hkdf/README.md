# HMAC-SHA-256 and HKDF

HMAC is the keyed-hash message authentication code of Bellare, Canetti and
Krawczyk (1996): the hash of a padded key and the message, hashed again under
a second padding of the same key. It was published as
[RFC 2104](https://www.rfc-editor.org/rfc/rfc2104) (February 1997) and
standardized by NIST as
[FIPS 198-1, The Keyed-Hash Message Authentication Code](https://doi.org/10.6028/NIST.FIPS.198-1)
(July 2008); its SHA-2 test cases are
[RFC 4231](https://www.rfc-editor.org/rfc/rfc4231) (December 2005). HKDF is
Krawczyk's extract-then-expand key derivation function built on HMAC,
published as [RFC 5869](https://www.rfc-editor.org/rfc/rfc5869) (May 2010),
with the analysis in his CRYPTO 2010 paper. HMAC is the MAC of the non-AEAD
cipher suites of TLS 1.2, of IPsec and of SSH, and the pseudorandom function
inside PBKDF2, HKDF, JWT's HS256 and TOTP; HKDF derives the keys of TLS 1.3,
the Signal protocol, HPKE and the Noise framework. Both are current standards with no known attack better than the
generic ones.

## Analysis

### Structure

HMAC (FIPS 198-1 section 4, RFC 2104 section 2) takes a hash H with block
size B bytes and output size L bytes, here SHA-256 with B = 64 and L = 32,
a key K and a text. The key is first brought to exactly B bytes as K0: a key
longer than B is hashed to L bytes, and a key shorter than B, including a
hashed one, has zeros appended. Then

    HMAC(K, text) = H((K0 xor opad) || H((K0 xor ipad) || text))

with ipad the byte 0x36 and opad the byte 0x5c, each repeated B times. The
inner hash absorbs the text after a key-dependent first block; the outer hash
absorbs the L-byte inner result after a different key-dependent first block.
The output may be truncated to its leftmost t bytes (RFC 2104 section 5).

HKDF (RFC 5869 section 2) has two stages. HKDF-Extract (section 2.2) takes a
salt and input keying material IKM and computes

    PRK = HMAC-Hash(salt, IKM)

with the salt as the HMAC key and, when no salt is given, a string of HashLen
zeros in its place. HKDF-Expand (section 2.3) takes PRK, an optional context
string info and a length L at most 255 HashLen, and computes

    T(1) = HMAC-Hash(PRK, info || 0x01)
    T(i) = HMAC-Hash(PRK, T(i-1) || info || i)   for 2 <= i <= N = ceil(L / HashLen)
    OKM  = the first L octets of T(1) || T(2) || ... || T(N)

where i is a single octet. The two files follow the standards section by
section, with SHA-256 carried along from FIPS 180-4 because an Orange module
has no imports.

| Standard section | Orange spec |
| --- | --- |
| FIPS 180-4, 3.1, a word to bytes | `be_bytes` |
| FIPS 180-4, 4.1.2, the SHA-256 functions | `ch`, `maj`, `big_sigma0`, `big_sigma1`, `small_sigma0`, `small_sigma1` |
| FIPS 180-4, 4.2.2, K{256} | `round_constants` |
| FIPS 180-4, 5.3.3, H(0) | `initial_hash` |
| FIPS 180-4, 5.1.1, padding | `put_byte`, `pad` |
| FIPS 180-4, 6.2.2, the hash computation | `schedule`, `round`, `compress`, `sha256` |
| FIPS 198-1, 4, ipad and opad | `ipad`, `opad` |
| FIPS 198-1, 4, steps 1 to 3, the key K0 | `k0` |
| FIPS 198-1, 4, steps 4 and 7 | `xor_pad` |
| FIPS 198-1, 4, steps 4 to 9, the MAC | `hmac_sha256` |
| FIPS 198-1 truncated output, RFC 2104, 5 | `leftmost_128` |
| RFC 5869, 2.2, HKDF-Extract | `hkdf_extract` |
| RFC 5869, 2.3, T(i) | `t_i` |
| RFC 5869, 2.3, HKDF-Expand with L = 42 and L = 82 | `hkdf_expand_42`, `hkdf_expand_82` |

Orange has no length-generic arrays, and HMAC hashes messages of many
lengths: the key itself when it is long, the 64-byte inner pad followed by
the text, the 96-byte outer message, and in HKDF-Expand the 32-byte T(i-1)
followed by info and a counter. Rather than one padding spec per length, as
the `sha2` entry writes, this entry carries every byte string as a pair: its
length in bytes and a `Word[32]^64` buffer of the big-endian words of FIPS
180-4 section 5.2.1, zero beyond the last byte. The buffer is four SHA-256
blocks, 256 bytes, enough for the 131-byte keys of RFC 4231, the 216-byte
inner message of its test case 7, and the 177-byte inner messages of T(2)
and T(3) in RFC 5869 A.2. `pad` writes the 0x80 byte at the length and the
bit length in the last word of the last block, both at positions computed
from the length, which is the one place a byte position depends on a value
rather than on a literal: `put_byte` puts it there by walking the 64 words
with a static index and matching the one whose index equals `p / 4`, and by
a four-way conditional on `p % 4` because a shift amount must be a literal.
`sha256` runs "for i = 1 to N" over the four possible blocks and compresses
only the first N. `k0` hashes the key only when its length exceeds 64,
inside the chosen branch of a conditional. `t_i` builds T(i-1) || info, with
T(i-1) as eight whole words so info starts at word 8, and appends the counter
octet with `put_byte`. Inputs are placed in the buffer by the `words_n`
helpers, one per word count in use.

### Security status

Record as written in September 2026. The standards themselves, FIPS 198-1,
SP 800-107 and the RFCs, are unreachable from the build machine; their
section numbers and the literature below are cited from the worker's
knowledge and were not re-read for this entry.

HMAC was designed with a proof. Bellare, Canetti and Krawczyk (CRYPTO 1996)
showed that NMAC, and HMAC as its single-key variant, is a secure MAC when
the compression function is a pseudorandom function and the iterated hash is
weakly collision resistant; the weak collision resistance requirement
concerned the hash's iteration under a secret initial value, not the public
hash. Bellare (CRYPTO 2006) removed the collision-resistance assumption
altogether: HMAC is a PRF if the compression function is a PRF under two
kinds of keying, which explains why the collision attacks on MD5 (Wang and
Yu, 2005) and SHA-1 (Stevens, Bursztein, Karpman, Albertini and Markov,
2017; Leurent and Peyrin, 2020) did not translate into practical forgeries
against HMAC-MD5 or HMAC-SHA-1. Koblitz and Menezes (2013) criticized the concrete tightness of
the 2006 bound, and Gazi, Pietrzak and Rybar (CRYPTO 2014) gave the exact
PRF security of NMAC and HMAC, with matching attacks. The generic limits are
those of any iterated MAC: an internal collision after about 2^(n/2) queries
gives forgeries (Preneel and van Oorschot, 1995), and the generic
state-recovery and universal-forgery attacks on hash-based MACs (Leurent,
Peyrin and Wang, Asiacrypt 2013; Peyrin and Wang, Eurocrypt 2014; Dinur and
Leurent, CRYPTO 2014) stay between 2^(n/2) and 2^n; for the 256-bit state
of SHA-256 none is a practical concern. No attack on HMAC-SHA-256 better
than these generic ones is known. For HMAC-MD5 the best known distinguishing
attacks (Wang, Yu, Wang, Zhang and Zhan, Eurocrypt 2009) need about 2^97
queries,
still impractical, and RFC 6151 (2011) advises against HMAC-MD5 in new
protocols on the strength of MD5's other failures; HMAC-SHA-1 has no
practical attack and NIST still lists SHA-1 as acceptable inside HMAC in SP
800-131A Rev. 2 (2019), while its December 2022 transition plan retires
SHA-1 from all federal use by the end of 2030.

The nested structure is what a secret-prefix MAC lacks. A Merkle-Damgard
hash such as SHA-256 outputs its whole chaining state, so anyone holding
H(k || m) and the length of m can continue the computation and produce
H(k || m || pad || m') without k: the length-extension forgery against
H(k || m). HMAC's outer hash processes the inner digest as data under a key
the attacker does not have, so the inner state is never exposed; and the
two distinct pads make the inner and outer keys K0 xor ipad and K0 xor opad
differ in every byte, so the two hashes are keyed by two related but
distinct blocks, which the 2006 proof relies on (the "dual PRF" keying by
the pads). A secret-suffix MAC H(m || k) fails in the other direction, from
an offline collision in m; HMAC is affected by neither. With SHA-3, whose
sponge hides part of its state, the plain keyed construction KMAC is secure,
but HMAC is defined for any approved hash, SHA-3 included.

Key length and truncation. RFC 2104 section 3 recommends keys of at least L
bytes; longer keys do not add strength, since a key longer than B is hashed
to L bytes anyway, and a key of less than L bytes gives at most its own
entropy. FIPS 198-1 section 3 requires a key of at least L / 2 bytes and
notes that keys above L bytes do not add strength; SP 800-107 Rev. 1 (2012)
ties HMAC's security strength to the key's and to the hash's state size.
Truncating the MAC (FIPS 198-1's truncated output, RFC 2104 section 5)
keeps the leftmost t bytes; RFC 2104 asks for t at least L / 2 and at least
80 bits, SP 800-107 states the corresponding conditions, and RFC 4231 test
case 5 is such a truncation to 128 bits. Two implementation pitfalls are not attacks on the
construction: comparing tags byte by byte with early exit leaks the first
differing position, and using a hash of a password as an HMAC key without a
slow derivation leaves the key guessable.

HKDF's rationale (Krawczyk, CRYPTO 2010) is that key derivation has two
jobs that should not be conflated. Extract turns input keying material of
unknown distribution, such as a Diffie-Hellman shared secret, into one
uniform PRK: with a random salt as the key, HMAC acts as a randomness
extractor (computationally, under assumptions on the compression function),
and the salt need not be secret. Expand then stretches PRK into as many
independent keys as the application needs, each bound to a context string
by the info parameter; this stage is a PRF and needs a uniform key, which is
why RFC 5869 warns that Expand alone is not a KDF for non-uniform material.
The chained form T(i) = HMAC(PRK, T(i-1) || info || i), rather than a plain
counter mode, is a design choice of the paper for the extract-then-expand
analysis and makes each block depend on the previous one. HKDF is the key
schedule of TLS 1.3 (RFC 8446, through HKDF-Expand-Label), of the Signal
protocol's X3DH and Double Ratchet, of HPKE (RFC 9180, as LabeledExtract and
LabeledExpand) and of the Noise framework, and NIST SP 800-56C Rev. 2 (2020)
approves it as a two-step derivation. Its pitfalls are those of any KDF: the
IKM must carry enough entropy (HKDF is fast and is not a password hash;
PBKDF2, scrypt or Argon2 are), the salt should be used when one is
available, info must separate every derived key, and L is bounded by 255
HashLen because the counter is one octet.

Status: FIPS 198-1 (2008) and RFC 2104 (1997) are the current definitions of
HMAC, and RFC 5869 (2010, Informational) of HKDF; neither has been revised.
NIST published a draft Special Publication 800-224 in 2024 that would carry
HMAC's specification and recommendations in place of FIPS 198-1; whether it
has been finalized was not checked from this machine.

### What the Orange rendering shows

Nothing in HMAC or HKDF is data-dependent except through lengths. The Orange
source has no table lookups and no data-dependent rotation; its conditionals
are on lengths and counters only: whether the key exceeds the block
(`k0`), how many blocks a message has (`pad`, `sha256`), where the 0x80 byte
and the counter octet fall (`put_byte`), whether the salt is empty
(`hkdf_extract`), and whether i = 1 (`t_i`). Every index is a literal or a
loop index, so the checker proves every access in range before evaluation.
The XOR of the pads, the two nested hashes and the chaining of T(i) are
visible as written in the standard; the word ring `Word[32]` gives SHA-256's
additions their meaning without masks.

Measured costs under `orangec eval` (a ballast loop added to the file and
its largest size found by bisection): one SHA-256 compression, with its
schedule, costs about 13,300 steps, of which the 48 updates of the 64-word
schedule are about 4,100; an HMAC of a short key and a short text is four
compressions and about 57,000 steps; RFC 4231 test cases 6 and 7, with the
131-byte key hashed first, are about 101,000 and 130,000; an HKDF with
L = 42 is about 176,000 and RFC 5869 A.2, with its 80-byte salt, IKM and
info and L = 82, about 325,000. All the planned vectors together are about
1,730,000 steps, so they are split between two files: `hmac-hkdf.or` uses
about 841,000 of the 1,048,576-step budget and `hmac-hkdf-rfc5869.or` about
893,000. No vector was dropped.

Not expressed: a message of arbitrary length, and an OKM of arbitrary L.
Every input travels as its length and a 256-byte buffer, so a key, text,
salt, IKM or info longer than 183 bytes (the buffer less the 64-byte pad and
the 9 bytes of SHA-256 padding), and an info longer than 150 bytes in
Expand, would need a larger buffer, which `Word[32]^64` cannot be. Expand
is written for L = 42 and L = 82 (N = 2 and N = 3), one spec per L, since
an output array's length is part of its type; the counter is written with
`put_byte` exactly as the standard's single octet.

## Dissemination

### Files

- `hmac-hkdf.or`: SHA-256, HMAC-SHA-256, HKDF-Extract and HKDF-Expand, with
  the seven HMAC-SHA-256 test cases of RFC 4231 (test case 5 both in full
  and truncated to 128 bits), Wycheproof's `hmac_sha256_test.json` tcId 171
  and Wycheproof's `hkdf_sha256_test.json` tcId 69.
- `hmac-hkdf-rfc5869.or`: the same algorithm text, with the PRK and OKM of
  the three SHA-256 test cases of RFC 5869 appendix A.

The two files are one file split by the step budget: their algorithm part,
from `edition 2026;` through `words_8`, is the same text, generated
from one source by a script kept with the worker's notes and compared byte
for byte; only the header's last paragraph, the `words_n` helpers each
file's vectors call, and the vector specs differ. `leftmost_128` is
exercised only by the first file and `hkdf_expand_82` only by the second.

### Running

    orangec eval algorithms/hmac-hkdf/hmac-hkdf.or
    orangec eval algorithms/hmac-hkdf/hmac-hkdf-rfc5869.or
    python3 algorithms/verify.py algorithms/hmac-hkdf

### Vectors

Each row is a pair `<spec>` and `<spec>_expected`; `verify.py` requires them
equal. The RFCs are unreachable from the build machine, so each value was
read from the fetched files named in the row, which reproduce the RFCs'
cases, and every value was recomputed with Python's `hmac` (HMAC and PRK) or
the `cryptography` package's `HKDF` (OKM). Where no fetched file carries a
case, the row says so and the oracle is the only source.

| Spec | Source | Case |
| --- | --- | --- |
| `rfc4231_case_1` | RFC 4231, 4.2; Go `src/crypto/hmac/hmac_test.go`; `hmac` | key 0x0b x 20, "Hi There" |
| `rfc4231_case_2` | RFC 4231, 4.3; Botan `src/tests/data/mac/hmac.vec`, `[HMAC(SHA-256)]`; Go `hmac_test.go`; `hmac` | key "Jefe", "what do ya want for nothing?" |
| `rfc4231_case_3` | RFC 4231, 4.4; Go `hmac_test.go`; `hmac` | key 0xaa x 20, data 0xdd x 50 |
| `rfc4231_case_4` | RFC 4231, 4.5; Go `hmac_test.go`; `hmac` | key 0x01..0x19, data 0xcd x 50 |
| `rfc4231_case_5` | RFC 4231, 4.6; `hmac` only (no fetched file carries the full MAC) | key 0x0c x 20, "Test With Truncation", full 256-bit MAC |
| `rfc4231_case_5_truncated` | RFC 4231, 4.6; `hmac` only (no fetched file carries this case) | the same MAC truncated to 128 bits, as the RFC prints it |
| `rfc4231_case_6` | RFC 4231, 4.7; Botan `hmac.vec`; Go `hmac_test.go`; `hmac` | key 0xaa x 131 (hashed first), 54-byte text |
| `rfc4231_case_7` | RFC 4231, 4.8; Botan `hmac.vec`; Go `hmac_test.go`; `hmac` | key 0xaa x 131, 152-byte text |
| `wycheproof_hmac_tc_171` | Wycheproof `testvectors_v1/hmac_sha256_test.json`, tcId 171 ("long key"); `hmac` | 65-byte key (one over the block), 32-byte message |
| `wycheproof_hkdf_tc_69` | Wycheproof `testvectors_v1/hkdf_sha256_test.json`, tcId 69; `cryptography` `HKDF` | 32-byte IKM, 64-byte salt (exactly the block), 20-byte info, L = 42 |
| `rfc5869_a_1_prk` | RFC 5869, A.1; Botan `src/tests/data/kdf/hkdf.vec`, `[HKDF-Extract(HMAC(SHA-256))]`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography `rfc-5869-HKDF-SHA256.txt`; `hmac` | IKM 0x0b x 22, salt 00..0c |
| `rfc5869_a_1_okm` | RFC 5869, A.1; Botan `hkdf.vec`, `[HKDF(HMAC(SHA-256))]`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Wycheproof `hkdf_sha256_test.json` tcId 1; `cryptography` `HKDF` | info f0..f9, L = 42 |
| `rfc5869_a_2_prk` | RFC 5869, A.2; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; `hmac` | IKM 00..4f, salt 60..af (80 bytes, hashed first) |
| `rfc5869_a_2_okm` | RFC 5869, A.2; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Wycheproof tcId 3; `cryptography` `HKDF` | info b0..ff, L = 82, N = 3 |
| `rfc5869_a_3_prk` | RFC 5869, A.3; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; `hmac` | IKM 0x0b x 22, empty salt (32 zero bytes) |
| `rfc5869_a_3_okm` | RFC 5869, A.3; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Wycheproof tcId 2; `cryptography` `HKDF` | empty info, L = 42 |

The fetched files are, all at `master` or `main` on the day of writing:
`randombit/botan` `src/tests/data/mac/hmac.vec` and
`src/tests/data/kdf/hkdf.vec`; `openssl/openssl`
`test/recipes/30-test_evp_data/evpkdf_hkdf.txt`; `C2SP/wycheproof`
`testvectors_v1/hmac_sha256_test.json` and `hkdf_sha256_test.json`;
`golang/go` `src/crypto/hmac/hmac_test.go` (which carries RFC 4231's SHA-256
cases 1, 2, 3, 4, 6 and 7); `pyca/cryptography`
`vectors/cryptography_vectors/KDF/rfc-5869-HKDF-SHA256.txt`.

### Provenance and claims

The SHA-256 constants were derived from their definition in FIPS 180-4 by
the reference script kept with the worker's notes outside the repository,
using exact integer roots (K{256} as the first 32 bits of the fractional
parts of the cube roots of the first 64 primes, H(0) from the square roots
of the first eight), and matched against the tables of the repository's
own `compiler/fixtures/s3e/valid-sha256.or`. The pads 0x36 and 0x5c and
HKDF's zero salt are the standards' own values, confirmed by every vector.
The same script holds an independent Python SHA-256, HMAC and HKDF, checked
against `hashlib`, `hmac` and `cryptography` over a grid of key and message
lengths, and then against every HMAC-SHA-256 entry of Botan's `hmac.vec`,
every valid case of both Wycheproof files, the RFC 5869 entries of Botan's
`hkdf.vec`, OpenSSL's `evpkdf_hkdf.txt` and pyca/cryptography's copy, and
the RFC 4231 cases in Go's test file. The Orange input and `_expected`
literals were printed by that script from the parsed bytes, never typed,
and the two `.or` files were generated from one master by a second script
that compares their algorithm parts byte for byte. No `_expected` value was
adjusted to match the Orange computation.

This entry is a reference evaluation of a specification under
`orangec eval`. It makes no constant-time, side-channel, performance or
certification claim, it has not been independently reviewed, and it is not a
corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- No length-generic arrays, so every byte string is carried as a length and
  a fixed `Word[32]^64` buffer, with one `words_n` placement helper per word
  count in use (fifteen distinct across the two files) and one HKDF-Expand
  spec per
  output length. Inputs above 183 bytes, or info above 150 bytes, would
  need a buffer larger than the 256-byte `Word[32]^64` used here. A
  `Word[32]` array may hold 65,536 elements; this entry has not been widened.
- A shift amount must be a literal, so placing one byte at a computed
  position (`put_byte`) is a four-way conditional over `p % 4` and a walk
  over the 64 words; about 830 steps each, twice per hash.
- The evaluation budget: a SHA-256 compression costs about 13,300 steps in
  this style, so one HMAC is about 57,000 and the sixteen planned pairs
  about 1,730,000 together, above the 1,048,576 of one file. The vectors
  are split between two files with identical algorithm text; none was
  dropped.
