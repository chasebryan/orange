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
the Signal protocol, HPKE and the Noise framework. Both are current
standards with no known attack better than the generic ones.

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

where i is a single octet. The root file `hmac-hkdf.or` follows FIPS 198-1
and RFC 5869 section by section, and takes SHA-256 from the module
`sha256.or`, which writes FIPS 180-4 as the `sha2` entry does, for messages
of 1 through 256 bytes.

| Standard section | Orange spec |
| --- | --- |
| FIPS 180-4, 4.1.2, the SHA-256 functions | `sha256::ch`, `maj`, `big_sigma0`, `big_sigma1`, `small_sigma0`, `small_sigma1` |
| FIPS 180-4, 4.2.2, K{256} | `sha256::round_constants` |
| FIPS 180-4, 5.3.3, H(0) | `sha256::initial_hash` |
| FIPS 180-4, 5.1.1, padding | `sha256::padding[len]` for `len` in 1 through 256 |
| FIPS 180-4, 5.2.1, parsing into words | `as big Word[32]^16` on each block, in `sha256::hash` |
| FIPS 180-4, 6.2.2, the hash computation | `sha256::schedule`, `round`, `compress`, `hash[n]`, `digest[len]` |
| FIPS 198-1, 4, ipad and opad | `ipad`, `opad` |
| FIPS 198-1, 4, steps 1 to 3, the key K0 | `k0[key_len]` for keys of 1 through 256 bytes |
| FIPS 198-1, 4, steps 4 and 7 | `xor` |
| FIPS 198-1, 4, steps 4 to 9, the MAC | `hmac[text_len]` for texts of 1 through 192 bytes |
| FIPS 198-1, 5, and RFC 2104, 5, truncated output | the slice `[..16]` in the test of RFC 4231 test case 5 |
| RFC 5869, 2.2, HKDF-Extract | `hkdf_extract[ikm_len]`, and `no_salt` for a salt not provided |
| RFC 5869, 2.3, HKDF-Expand | `hkdf_expand[nb, info_len]`, and `hkdf_expand_no_info[nb]` for a zero-length info |

Every byte string is a byte array, `Word[8]^n`, and its length is a size of
the spec that takes it: a key, text, salt, IKM or info is written in a test
as a string literal, `hex"..."` or a fill such as `[0x0b; 20]`, and the
checker picks the instance of `k0`, `hmac` or `hkdf_extract` whose size
matches. The messages HMAC hashes are joined with `++` as the standard
joins them, `xor(k0, ipad()) ++ text` and `xor(k0, opad()) ++ inner`, and
`sha256::digest` pads each with `padding[len]` and hashes its blocks.

A function has at most 256 instances, counting every combination of its
sizes, and that limit shapes three specs. HMAC of a key of `key_len` bytes
and a text of `text_len` bytes would need the product of both ranges, so
FIPS 198-1's steps 1 to 3 are `k0[key_len]`, with one size, and steps 4 to
9 are `hmac[text_len]`, whose parameter `k0` is K0; a test writes
HMAC(K, text) as `hmac(k0(key), text)`, and HKDF-Extract takes K0 of its
salt the same way, as `hkdf_extract(k0(salt), ikm)`. HKDF-Expand has
the info length and the block count N = ceil(L / HashLen) as its sizes,
240 instances for N of 1 through 3 and info of 1 through 80 bytes, and
returns all of T(1) || ... || T(N); each test keeps the first L octets with
a slice. And an array has at least one element, so the zero-length info of
RFC 5869 A.3 is `hkdf_expand_no_info`, the same loop with info left out,
and the empty salt of A.3 is the HashLen zeros that section 2.2 puts in its
place, `no_salt()`.

`k0` compares its size `key_len` with B: a key of more than 64 bytes is
hashed and 32 zeros appended, any other key is followed by 64 zeros and cut
to its first 64 bytes, which is step 1 for a key of exactly 64 bytes and
step 3 for a shorter one. `hkdf_expand` runs the standard's chain as a loop
over the N blocks, its index j being the standard's i - 1, that carries T
and the last T(i-1); T(0) is empty and has no array, so the first step
hashes info || 0x01 alone, and the 32 zeros that start the carried block
are never read. `hkdf_expand_no_info` repeats that loop without info, and
the two must stay identical apart from it.

### Security status

Record as written in September 2026. The standards themselves, FIPS 198-1,
SP 800-107 and the RFCs, were unreachable from the build machine when this
entry was written. Their section numbers were later checked against the
published texts: FIPS 198, FIPS 198-1 and FIPS 180-4 at nvlpubs.nist.gov and
csrc.nist.gov, RFC 2104, RFC 4231 and RFC 5869 at rfc-editor.org. The
literature below was cited from the worker's knowledge and was not re-read
for this entry.

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
entropy. The original FIPS 198 (2002) section 3 required a key of at least
L / 2 bytes and noted that keys above L bytes do not add strength; FIPS 198-1
section 3 asks only for a key of appropriate security strength, as SP 800-107
discusses, and SP 800-107 Rev. 1 (2012) ties HMAC's security strength to the
key's and to the hash's state size. Truncating the MAC (FIPS 198-1's
truncated output, RFC 2104 section 5) keeps the leftmost t bytes; RFC 2104
asks for t at least L / 2 and at least 80 bits, SP 800-107 states the
corresponding conditions, and RFC 4231 test case 5 is such a truncation to
128 bits. Two implementation pitfalls are not attacks on the construction:
comparing tags byte by byte with early exit leaks the first differing
position, and using a hash of a password as an HMAC key without a slow
derivation leaves the key guessable.

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

Nothing in HMAC or HKDF is data-dependent except through lengths, and the
lengths here are sizes, fixed for each instance by the checker. The only
table is K{256}, built once per block by `sha256::round_constants` and read
as `k[t]` with the round's loop index t, as the message schedule is read as
`w[t]`, `w[t - 2]` and the like; no index depends on the key or the
message, and every rotation amount is a literal. The two conditionals are
`key_len > 64` in `k0`, whose size makes it fixed in each instance, and
`j == 0` in the Expand loops, on the loop index; both choose between
branches of the standard's own text, and only the chosen branch is
evaluated, so a short key is never hashed and a long one never padded.
Every index is a literal or a loop index, and every slice bound a literal,
a loop index times 32 or 64, or a size, so the checker proves every access
in range before evaluation.
The XOR of the pads, the two nested hashes and the chaining of T(i) are
visible as written in the standards; the word ring `Word[32]` gives
SHA-256's additions their meaning without masks.

Byte order appears only inside `sha256`, where FIPS 180-4 fixes it: `as big`
parses each 64-byte block into sixteen words, writes the bit length into
the padding, reads the round constants from `hex"..."` rows and turns the
final hash value into the digest's 32 bytes. HMAC and HKDF are written over
bytes, as their standards are, and the vectors compare byte strings printed
as the sources print them.

Measured costs under `orangec test --stats`: each 64-byte block adds 8,938
steps to `sha256::digest` (8,978 for one block, 17,916 for two, 35,790 for
four); `xor` costs about 710 steps; `k0` costs about 14 steps for a key of
at most 64 bytes and 26,862 for the 131-byte key of RFC 4231, which pads to
three blocks. An HMAC of a text of up to 55 bytes is four blocks and two
`xor`s, 37,254 steps, and one of the 152-byte text of test case 7 is six
blocks, 55,132. `hkdf_expand` costs 74,596 steps for N = 2 with a 10-byte
info and 138,703 for N = 3 with the 80-byte info of A.2. The RFC 4231 tests
cost 37,266 to 37,270 steps each, except test cases 6 and 7 (64,109 and
81,992, with the long key hashed first), and the Wycheproof HMAC test,
whose 65-byte key is hashed too, 55,177; the PRK tests 37,270 to 64,122;
the OKM tests, which compute their PRK too, 111,862 to 202,832. The sixteen
tests together use 1,101,975 steps.

Not expressed: a key longer than 256 bytes, a text or IKM longer than 192
bytes (the inner message, 64 bytes more, is at most the 256 that
`sha256::digest` takes), an info longer than 80 bytes, an output of more
than three blocks (L above 96), and a message whose length is not a whole
number of bytes. Each range is one number in a signature, up to the
256-instance limit under Gaps.

## Dissemination

### Files

- `sha256.or`: the module `sha256`, SHA-256 of FIPS 180-4 for messages of
  1 through 256 bytes. It has no tests of its own; `hmac-hkdf.or` uses it.
- `hmac-hkdf.or`: HMAC-SHA-256, HKDF-Extract and HKDF-Expand, with the
  seven HMAC-SHA-256 test cases of RFC 4231 (test case 5 both in full and
  truncated to 128 bits), the PRK and OKM of the three SHA-256 test cases
  of RFC 5869 appendix A, Wycheproof's `hmac_sha256_test.json` tcId 171 and
  Wycheproof's `hkdf_sha256_test.json` tcId 69.

### Running

    orangec test --steps 1073741824 algorithms/hmac-hkdf/hmac-hkdf.or
    python3 algorithms/verify.py algorithms/hmac-hkdf

The tests need more than `orangec test`'s default budget of 1,048,576
evaluation steps (they use 1,101,975), so the command raises it, as
`verify.py` does.

`orangec test` reads `sha256.or` from the same folder through `use sha256;`.

### Vectors

Each row is a `test` block in `hmac-hkdf.or`, comparing a MAC, PRK or OKM
with the published value. The RFCs are unreachable from the build machine,
so each value was read from the fetched files named in the row, which
reproduce the RFCs' cases, and every value was recomputed with Python's
`hmac` (HMAC and PRK) or the `cryptography` package's `HKDF` (OKM). Where no
fetched file carries a case, the row says so and the oracle is the only
source.

| Test | Source | Case |
| --- | --- | --- |
| `RFC 4231 4.2: test case 1` | RFC 4231, 4.2; Go `src/crypto/hmac/hmac_test.go`; Python `hmac` | key 0x0b x 20, "Hi There" |
| `RFC 4231 4.3: test case 2` | RFC 4231, 4.3; Botan `src/tests/data/mac/hmac.vec`, `[HMAC(SHA-256)]`; Go `hmac_test.go`; Python `hmac` | key "Jefe", "what do ya want for nothing?" |
| `RFC 4231 4.4: test case 3` | RFC 4231, 4.4; Go `hmac_test.go`; Python `hmac` | key 0xaa x 20, data 0xdd x 50 |
| `RFC 4231 4.5: test case 4` | RFC 4231, 4.5; Go `hmac_test.go`; Python `hmac` | key 0x01..0x19, data 0xcd x 50 |
| `RFC 4231 4.6: test case 5, the full MAC` | RFC 4231, 4.6; Python `hmac` only (no fetched file carries the full MAC) | key 0x0c x 20, "Test With Truncation", full 256-bit MAC |
| `RFC 4231 4.6: test case 5, truncated to 128 bits` | RFC 4231, 4.6; Python `hmac` only (no fetched file carries this case) | the same MAC truncated to 128 bits, as the RFC prints it |
| `RFC 4231 4.7: test case 6` | RFC 4231, 4.7; Botan `hmac.vec`; Go `hmac_test.go`; Python `hmac` | key 0xaa x 131 (hashed first), 54-byte text |
| `RFC 4231 4.8: test case 7` | RFC 4231, 4.8; Botan `hmac.vec`; Go `hmac_test.go`; Python `hmac` | key 0xaa x 131, 152-byte text |
| `Wycheproof hmac_sha256_test tcId 171` | Wycheproof `testvectors_v1/hmac_sha256_test.json`, tcId 171 ("long key"); Python `hmac` | 65-byte key (one over the block), 32-byte message |
| `Wycheproof hkdf_sha256_test tcId 69` | Wycheproof `testvectors_v1/hkdf_sha256_test.json`, tcId 69; `cryptography` `HKDF` | 32-byte IKM, 64-byte salt (exactly the block), 20-byte info, L = 42 |
| `RFC 5869 A.1: PRK` | RFC 5869, A.1; Botan `src/tests/data/kdf/hkdf.vec`, `[HKDF-Extract(HMAC(SHA-256))]`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography `rfc-5869-HKDF-SHA256.txt`; Python `hmac` | IKM 0x0b x 22, salt 00..0c |
| `RFC 5869 A.1: OKM` | RFC 5869, A.1; Botan `hkdf.vec`, `[HKDF(HMAC(SHA-256))]`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Wycheproof `hkdf_sha256_test.json` tcId 1; `cryptography` `HKDF` | info f0..f9, L = 42 |
| `RFC 5869 A.2: PRK` | RFC 5869, A.2; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Python `hmac` | IKM 00..4f, salt 60..af (80 bytes, hashed first) |
| `RFC 5869 A.2: OKM` | RFC 5869, A.2; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Wycheproof tcId 3; `cryptography` `HKDF` | info b0..ff, L = 82, N = 3 |
| `RFC 5869 A.3: PRK` | RFC 5869, A.3; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Python `hmac` | IKM 0x0b x 22, empty salt (32 zero bytes) |
| `RFC 5869 A.3: OKM` | RFC 5869, A.3; Botan `hkdf.vec`; OpenSSL `evpkdf_hkdf.txt`; pyca/cryptography; Wycheproof tcId 2; `cryptography` `HKDF` | empty info, L = 42 |

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
literals of the first form were printed by that script from the parsed
bytes, never typed, and its two `.or` files were generated from one master
by a second script that compared their algorithm parts byte for byte. No
`_expected` value was adjusted to match the Orange computation.

The entry was then rewritten in the current language, as one root file
over a `sha256` module in place of the two files the old step budget
required. Every expected value is carried over byte for byte from the first
form, where each was a `<name>_expected` spec: the eight-word MACs and PRKs
are now the 32 big-endian bytes that FIPS 180-4 defines the digest to be,
printed from the old word literals by a script, and the 16-byte truncated
MAC and the 42- and 82-byte OKMs were byte arrays already and are the same
bytes in `hex"..."`. The inputs, which the first form carried as zero-padded
word buffers with a length, are now string literals, `hex"..."` and fills;
a script rebuilt each old input from its words and length and compared it
byte for byte with the new literal. No vector was added or dropped. The
round constants, now `hex"..."` rows read with `as big`, are the old word
literals in the same order, and the initial hash value is unchanged.

This entry is a reference evaluation of a specification under
`orangec test`. It makes no constant-time, side-channel, performance or
certification claim, it has not been independently reviewed, and it is not a
corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- A function has at most 256 instances, counting every combination of its
  sizes. HMAC of a key length and a text length is therefore two specs,
  `k0[key_len]` and `hmac[text_len]`, joined at each call as
  `hmac(k0(key), text)`, and
  HKDF-Extract takes its salt the same way; HKDF-Expand cannot take L as a
  size beside the info length, so it returns N whole blocks and each test
  slices off the first L octets.
- A sized spec with no array parameter of that size needs the size written
  at the call: a test names N and the info length in
  `hkdf_expand[2, 10](prk, info)`, since N cannot be inferred from the
  arguments.
- No array has zero elements, so a zero-length info (RFC 5869 A.3) is a
  second spec, `hkdf_expand_no_info`, and T(0) is never a value; a
  zero-length salt is written as the HashLen zeros RFC 5869 section 2.2
  puts in its place, which is the standard's own rule, and a zero-length
  key or text has no instance at all.
- `sha256::digest` takes 1 through 256 bytes, its 256 instances, so HMAC
  texts stop at 192 bytes; a hash over longer messages would pad whole
  blocks and the final partial block separately. No vector here needs more
  than 216 bytes.
- Messages are whole bytes; FIPS 180-4 allows any bit length, and a key or
  text of a bit length that is not a multiple of 8 would need its last byte
  padded by hand.
