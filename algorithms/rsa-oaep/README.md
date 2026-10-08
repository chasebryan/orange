# RSAES-OAEP

RSAES-OAEP is RSA encryption with Optimal Asymmetric Encryption Padding, the
padding of Mihir Bellare and Phillip Rogaway (Eurocrypt 1994). RSA
Laboratories adopted it in PKCS #1 v2.0 (1998, published as RFC 2437), and
the current text is
[RFC 8017, PKCS #1: RSA Cryptography Specifications Version 2.2](https://www.rfc-editor.org/rfc/rfc8017)
(Moriarty, Kaliski, Jonsson and Rusch, November 2016), section 7.1. The
message is padded with the hash of a label and a run of zero octets, masked
twice with a mask generation function keyed by a random seed, and the
result is encrypted with the RSA permutation. RFC 8017 requires support
for RSAES-OAEP in new applications and keeps the older RSAES-PKCS1-v1_5
only for compatibility. It is the RSA key transport of the JOSE algorithms
`RSA-OAEP` and `RSA-OAEP-256` (RFC 7518), of X.509 and CMS (RFC 4055,
RFC 3560), and of the KTS-OAEP schemes of NIST SP 800-56B Rev. 2.

## Analysis

### Structure

RFC 8017 builds the scheme in layers. Section 4 converts between integers
and octet strings: I2OSP(x, xLen) writes x as xLen base-256 digits, most
significant first, and OS2IP reads them back. Section 5.1 gives the RSA
primitives: RSAEP((n, e), m) = m^e mod n, and RSADP(K, c) = c^d mod n, which
with the private key in its second form (section 3.2), the quintuple
(p, q, dP, dQ, qInv), is computed by the Chinese remainder theorem as
m_1 = c^dP mod p, m_2 = c^dQ mod q, h = (m_1 - m_2) qInv mod p and
m = m_2 + q h. Appendix B.2.1 defines MGF1(mgfSeed, maskLen), the
concatenation of Hash(mgfSeed || I2OSP(counter, 4)) for counter = 0, 1,
..., cut to maskLen octets.

Section 7.1.1 encrypts a message M of mLen octets under a key of k octets.
EME-OAEP encoding (step 2) forms the data block
DB = lHash || PS || 0x01 || M of k - hLen - 1 octets, where lHash is the
hash of the label L and PS is k - mLen - 2 hLen - 2 zero octets; it masks DB
with MGF(seed, k - hLen - 1) for a random seed of hLen octets, masks the
seed with MGF(maskedDB, hLen), and outputs EM = 0x00 || maskedSeed ||
maskedDB. Step 3 encrypts: C = I2OSP(RSAEP((n, e), OS2IP(EM)), k). Section
7.1.2 decrypts: EM = I2OSP(RSADP(K, OS2IP(C)), k), then the masks are
undone in the opposite order, and step 3.g outputs "decryption error"
unless the first octet Y of EM is zero, the first hLen octets of DB equal
lHash, and the first octet after them that is not zero is 0x01. M is what
follows that octet.

The Orange files follow the RFC section by section. The integers modulo n
are `Int` values, and the octet strings are `Word[8]` arrays. SHA-1 and
SHA-256 are the modules `sha1` and `sha256`, used by the root file
`rsa-oaep.or`. Everything that does not depend on the hash is written once;
MGF1 and the encoding and decoding are written once with SHA-1 for k = 128
octets (1024-bit moduli, the RSA Laboratories key) and once with SHA-256
for k = 256 (2048-bit moduli, the Wycheproof key). The label is the empty
string throughout.

| Standard section | Orange spec |
| --- | --- |
| RFC 8017 3.1, the public key (n, e) | `PublicKey` |
| RFC 8017 3.2, the private key, first form (n, d) and second form (p, q, dP, dQ, qInv) | `PrivatePair`, `PrivateQuintuple` |
| RFC 8017 4.1, I2OSP | `i2osp` |
| RFC 8017 4.2, OS2IP | `os2ip` |
| RFC 8017 5.1.1, RSAEP | `rsaep`, with `power` |
| RFC 8017 5.1.2, RSADP, step 2.a (the pair) and step 2.b (the quintuple) | `rsadp_pair`, `rsadp_quintuple`, with `power` |
| RFC 8017 appendix B.2.1, MGF1 | `mgf1_sha1`, `mgf1_sha256` |
| RFC 8017 7.1.1, step 2, EME-OAEP encoding | `eme_oaep_encode_sha1`, `eme_oaep_encode_sha256`, with `xor` |
| RFC 8017 7.1.1, RSAES-OAEP-ENCRYPT | `rsaes_oaep_encrypt_sha1`, `rsaes_oaep_encrypt_sha256` |
| RFC 8017 7.1.2, step 3, EME-OAEP decoding and the checks of step 3.g | `eme_oaep_decode_sha1`, `eme_oaep_decode_sha256`, with `xor` and `separator` |
| RFC 8017 7.1.2, RSAES-OAEP-DECRYPT, steps 1 through 3 | `rsaes_oaep_decrypt_sha1`, `rsaes_oaep_decrypt_sha256` |
| RFC 8017 7.1.2, step 4, the message M | `message_sha1`, `message_sha256` |
| FIPS 180-4 4.1.1, 4.2.1, 5.3.1, SHA-1 functions, constants and H(0) | `sha1::f`, `sha1::k`, `sha1::initial_hash` |
| FIPS 180-4 4.1.2, 4.2.2, 5.3.3, SHA-256 functions, constants and H(0) | `sha256::ch`, `maj`, `big_sigma0`, `big_sigma1`, `small_sigma0`, `small_sigma1`, `round_constants`, `initial_hash` |
| FIPS 180-4 5.1.1 and 5.2.1, padding and parsing | `sha1::pad`, `sha256::pad`, and `as big Word[32]^16` in `compress` |
| FIPS 180-4 6.1.2 and 6.2.2, the hash computations | `compress`, `hash_blocks`, `digest`, `digest_empty` in each module |

The scheme's errors are values. An evaluation cannot stop with an error, so
RSADP returns its result with a `Bool` that is false where step 1 outputs
"ciphertext representative out of range", the decoding returns DB with a
`Bool` that is false where step 3.g outputs "decryption error", and
`rsaes_oaep_decrypt_sha1` and `_sha256` return the conjunction of the two
with DB. The message M of step 4 has a length that the data decides, and an
array's length is part of its type, so `message_sha1[mLen]` takes the
length from the caller and returns M with a `Bool` that is true when
decryption succeeded and the separator stands where a message of that
length puts it.

### Security status

RSA-OAEP's security rests on two things: the hardness of inverting RSA, and
the padding's proof. Bellare and Rogaway's 1994 argument modelled the hash
and the mask generation function as random oracles. Shoup (Crypto 2001)
showed that the argument does not prove security against adaptive
chosen-ciphertext attack for a general trapdoor permutation; Fujisaki,
Okamoto, Pointcheval and Stern (Crypto 2001) proved RSA-OAEP secure against
that attack in the random oracle model under the RSA assumption alone, with
a reduction that is far from tight. No attack on the scheme as specified is
known when the key is large enough and the implementation does not leak.

The attacks that matter are on implementations. Manger (Crypto 2001)
showed that a decryptor which reveals whether the first octet Y of EM was
zero, by a distinct error or by its timing, lets an attacker decrypt any
ciphertext with a number of queries of the order of the modulus's length
in bits. RFC 8017 therefore says, in a note to step 3.g, that care must be
taken that an opponent cannot distinguish the failures of the decoding, and
Wycheproof's `InvalidOaepPadding` cases, of which tcId 12 is one, exist to
test exactly that; its notes cite CVE-2020-26939. The same family of attacks
on RSAES-PKCS1-v1_5 (Bleichenbacher, Crypto 1998) is the reason RFC 3560
gives for using OAEP for RSA key transport in CMS. A decryption by the
Chinese remainder theorem that suffers a fault in one half reveals a factor
of n from the faulty output (Boneh, DeMillo and Lipton, Eurocrypt 1997), so
implementations check the result before releasing it; and square-and-multiply
exponentiation that branches on the bits of the private exponent leaks them
through timing (Kocher, Crypto 1996).

The size of the key is the rest. The 1024-bit key of the RSA Laboratories
vectors is a test key: NIST SP 800-131A Rev. 2 disallows RSA key transport
with moduli below 2048 bits. Public factorizations of RSA challenge numbers
include RSA-768 (Kleinjung and others, 2009) and RSA-250, of 829 bits
(Boudot, Gaudry, Guillevic, Heninger, Thome and Zimmermann, 2020). Shor's
algorithm would factor any RSA modulus on a large enough quantum computer,
and NIST's draft transition plan, NIST IR 8547 (November 2024), proposes to
deprecate RSA at the 112-bit security level after 2030 and to disallow RSA
key establishment after 2035, with ML-KEM (FIPS 203) as the replacement.
The SHA-1 of the RSA Laboratories vectors is the hash of PKCS #1 v2.0's
time. The proof of Fujisaki and others assumes only the RSA problem in the
random oracle model, not collision resistance of the hash, and JOSE's
`RSA-OAEP-256` uses SHA-256.

### What the Orange rendering shows

What depends on data is the exponentiation and the decoding. `power` is
square-and-multiply from the low bit of the exponent up: each step tests
the low bit of the remaining exponent with `if (f % 2) == 1` and multiplies
only when it is set, so the work follows the bits of the private exponents
dP and dQ in decryption, which is the timing channel of the section above.
The loop is bounded at 2,048 steps, enough for every exponent of a key of
up to 2,048 bits; once the exponent is used up, the remaining steps carry
the state through unchanged at a few steps each. The modulus does not fit
a `Mod[m]` (m below 2^521), so every product is reduced with `%` on
exact integers. The decoding computes all three checks of step 3.g and
joins them with `&&`, which evaluates every operand, so no check is
skipped when another has failed; `separator` scans every octet of
PS || 0x01 || M whatever it finds. The hashes have no data-dependent index
or branch: their only branches are on the step number t.

Byte orders and lengths are written as the RFC writes them. I2OSP and OS2IP
are `x as big Word[8]^xLen` and `x as big Int`, and the hashes read a block
as `block as big Word[32]^16`. The encoding is
`l_hash ++ ps_one ++ message` and `hex"00" ++ masked_seed ++ masked_db`,
where `ps_one` is PS with its closing 0x01, written as a run of zeros with
the last octet set so that PS may be empty for the longest message, as it
is in tcId 11. The
decoding takes EM apart with slices, `em[1..21]` and `em[21..]`. The SHA
padding writes the 64-bit length and the zero bits before it as one
integer, `l as big Word[8]^(...)`, over the rest of the last block. The
message length mLen is a size parameter, from 1 to k - 2 hLen - 2 octets,
so one `rsaes_oaep_encrypt_sha1` serves the six messages of 7 to 55 octets
of Example 1. MGF1 takes the seed as a type parameter listing the two seed
lengths OAEP passes for each k, and the number of hash blocks as a size;
its caller takes the leading maskLen octets with a slice.

The cost is the exponentiation. Measured with `orangec test --stats`: a
SHA-1 block costs about 7,700 steps and a SHA-256 block about 8,700, so an
EME-OAEP encoding with SHA-1 and k = 128 (nine SHA-1 blocks) costs about
71,000; RSAEP with e = 65537 costs about 88,000 under a 1024-bit modulus
and 258,000 under a 2048-bit one, and each Example 1 encryption costs
159,755 to 159,758 in all. RSADP with the quintuple costs 1,291,886 steps
on Example 1.1's ciphertext; the first form, with the full 1,024-bit d, costs
4,949,233 there, nearly four times as much, so the tests decrypt with the
quintuple and `rsadp_pair` is checked but not run. A 2048-bit decryption
by the quintuple costs about 9.7 million steps (9,710,538 for tcId 7), a
2048-bit encryption about 369,000 (368,772 for tcId 11), and the eleven
tests take 22,479,970 steps together.

## Dissemination

### Files

- `rsa-oaep.or`: module `rsa_oaep`, the root. I2OSP, OS2IP, RSAEP, RSADP
  in both forms, MGF1 with SHA-1 and with SHA-256, RSAES-OAEP encryption
  and decryption with SHA-1 for k = 128 and with SHA-256 for k = 256, the
  two keys of the vectors, and the eleven tests.
- `sha1.or`: module `sha1`, SHA-1 of FIPS 180-4 for messages of 0 through
  255 octets. It holds no tests; the root's tests exercise it.
- `sha256.or`: module `sha256`, SHA-256 of FIPS 180-4 for messages of 0
  through 255 octets. It holds no tests; the root's tests exercise it.

### Running

```console
orangec test --steps 1073741824 algorithms/rsa-oaep/rsa-oaep.or
python3 algorithms/verify.py algorithms/rsa-oaep
```

The tests need about 22.5 million steps, more than the default budget of
1,048,576, so the command names the budget the gate uses; they run in about
a tenth of a second.

### Vectors

| Test | Source | Case |
| --- | --- | --- |
| `oaep-vect.txt Example 1.1: encryption` | RSA Laboratories, `oaep-vect.txt`, OAEP Example 1.1 | 1024-bit key of Example 1, SHA-1, MGF1-SHA-1, 28-octet message, the seed given: the encryption |
| `oaep-vect.txt Example 1.2: encryption` | `oaep-vect.txt`, OAEP Example 1.2 | the same key, 28-octet message: the encryption |
| `oaep-vect.txt Example 1.3: encryption` | `oaep-vect.txt`, OAEP Example 1.3 | the same key, 55-octet message: the encryption |
| `oaep-vect.txt Example 1.4: encryption` | `oaep-vect.txt`, OAEP Example 1.4 | the same key, 26-octet message: the encryption |
| `oaep-vect.txt Example 1.5: encryption` | `oaep-vect.txt`, OAEP Example 1.5 | the same key, 20-octet message: the encryption |
| `oaep-vect.txt Example 1.6: encryption` | `oaep-vect.txt`, OAEP Example 1.6 | the same key, 7-octet message: the encryption |
| `oaep-vect.txt Example 1.1: decryption` | `oaep-vect.txt`, OAEP Example 1.1, and Example 1's private key | the encryption decrypts by the quintuple to the 28-octet message |
| `Wycheproof tcId 7: decryption` | Wycheproof, `rsa_oaep_2048_sha256_mgf1sha256_test.json`, tcId 7 (valid, Normal) | 2048-bit key, SHA-256, MGF1-SHA-256, empty label: the ciphertext decrypts to the 32-octet message e0e1...feff |
| `Wycheproof tcId 7: encryption with its seed` | the same case; the seed recovered from it | the 32-octet message and the seed 008b6093...a4daf7 encrypt to the ciphertext of the case |
| `Wycheproof tcId 11: encryption with its seed` | the same file, tcId 11 (valid, Normal, "Longest valid message size"); the seed recovered from it | the 190-octet message of 0x78 octets, with PS empty, and the seed a4493a49...47e56b encrypt to the ciphertext of the case |
| `Wycheproof tcId 12: decryption error` | the same file, tcId 12 (invalid, InvalidOaepPadding, "first byte of l_hash modified") | Y is zero and the separator is in place, but lHash' differs from lHash: "decryption error" |

### Provenance and claims

The RSA Laboratories vectors are Example 1 of `oaep-vect.txt`, the RSA-OAEP
vectors distributed with PKCS #1 (the file's header says PKCS #1 v2.0; ten
keys, six messages each, SHA-1 and MGF1-SHA-1, a 20-octet seed per
message). The Wycheproof cases are from
`rsa_oaep_2048_sha256_mgf1sha256_test.json` (source google-wycheproof
version 0.9, schema `rsaes_oaep_decrypt_schema_v1.json`), whose one test
group carries the 2048-bit private key; the file prints the modulus and the
primes with a leading 00 octet where the top bit is set, and the Orange
writes the octets after it. Every key component, message, seed and
ciphertext in `rsa-oaep.or` was parsed from those two files and printed into
the source by a generator script that is not part of the repository; none
was typed. Before any expected value was written down it was recomputed
with independent implementations in Python: for each of Examples 1.1
through 1.6, pycryptodome's `PKCS1_OAEP` with SHA-1, given the
example's seed as its random source, encrypts the message to the published
encryption, and both pycryptodome and the `cryptography` package decrypt
the encryption to the message; the key components were checked to agree
(p q = n, dP = d mod (p - 1), dQ = d mod (q - 1), qInv q = 1 mod p) for both
keys. For Wycheproof tcId 7, `cryptography` with SHA-256 and MGF1-SHA-256
decrypts the ciphertext to the message. The file does not print the seed;
a short Python reference of RFC 8017 sections 5.1.2 and 7.1.2, steps 2 and
3.b through 3.d, using `hashlib`, recovered it from the ciphertext, and
pycryptodome's `PKCS1_OAEP` with SHA-256, given that seed, encrypts the
message to the case's ciphertext. For tcId 12 both `cryptography` and
pycryptodome refuse to decrypt, and the same reference showed that Y is
zero and the separator and a 6-octet message are in place while lHash'
differs from lHash in its first octet, so the test exercises the lHash
check alone. Wycheproof tcId 11 ("Longest valid message size", a 190-octet
message, so PS is empty) was added the same way as tcId 7's encryption:
`cryptography` decrypts its ciphertext to the message, the same reference
recovered its seed and showed DB = lHash || 0x01 || M, and pycryptodome's
`PKCS1_OAEP` with SHA-256, given that seed, encrypts the message to the
case's ciphertext. The SHA-1 and SHA-256 constants are FIPS 180-4's, and both
modules were checked against `hashlib` on every message length from 0
through 255 octets while the entry was written.

This entry was written new in the current language. The first draft of
`rsa-oaep.or`, on the branch `claude/orange-algorithms-oy5d27`, was written
for an older compiler with fixed-length helpers and could not evaluate a
decryption; it reproduced Examples 1.1 and 1.2, the EME-OAEP decoding of
Example 1.1's encoded message, and one SHA-256 encryption under a generated
key. The decryption test of Example 1.1 here covers that decoding, with the
separator after lHash and 58 octets of PS, through `message_sha1[28]`. That draft guided the structure, and no expected
value was carried over from it: the generated key and its reference value
were dropped in favour of the Wycheproof case.

This entry is a reference evaluation of a specification under
`orangec test`: it shows that the Orange text computes the published values
on the cases listed. It makes no constant-time, side-channel, performance
or certification claim, it has not been independently reviewed, and it is
not a corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- No function can take a hash as a parameter, and a type parameter cannot
  choose between two bodies, so MGF1, the encoding, the decoding and the
  scheme are written once for SHA-1 and once for SHA-256.
- A function has at most 256 instances, the product of its size and type
  parameters. The message length mLen is a size, so the key length k cannot
  be one as well over any useful range: each copy of the scheme fixes k
  (128 octets with SHA-1, 256 with SHA-256). For the same reason MGF1 lists
  the two seed lengths OAEP passes as types, and returns whole hash values
  for its caller to cut.
- An array has at least one element, so a message of 0 octets cannot be
  encrypted, and the label L is always the empty string: a label would be
  one more length. The hashes take 1 through 255 octets, and the empty
  string has its own spec, `digest_empty`.
- A function's result has one type, so decryption returns a `Bool` with DB
  and the errors of the RFC are `false`, not distinct outputs; the message
  of step 4 is returned for a length the caller names.
- The modulus is beyond `Mod[m]`, so the arithmetic is `Int` with `%`, at
  about 3,000 steps a modular product for a 1,024-bit modulus. A 2048-bit
  decryption costs about 9.7 million steps even by the Chinese remainder
  theorem, so the entry holds two of them; the first form of RSADP is
  checked but not run by the tests.
- `power` loops over at most 2,048 exponent bits, enough for keys of up to
  2,048 bits; a larger key needs a larger bound.
- RSADP's fault check, the blinding that implementations use against
  timing, and the random seed of step 2.d are not written: the seed is an
  input, as the vectors give it.
