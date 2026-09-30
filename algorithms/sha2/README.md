# SHA-2: SHA-256 and SHA-512

SHA-2 is the family of hash functions that NIST published in 2001 (SHA-256,
SHA-384, SHA-512, designed at the NSA) and extended in 2004 (SHA-224) and 2012
(SHA-512/224, SHA-512/256). The current definition is
[FIPS 180-4, Secure Hash Standard](https://doi.org/10.6028/NIST.FIPS.180-4)
(August 2015). SHA-256 and SHA-512 are the hashes of TLS, X.509 certificates,
code signing, Git's newer object format, HMAC and HKDF, most password-based
key derivation, and Bitcoin's proof of work; they remain approved and in
active use, with SHA-3 standardized alongside them rather than in their
place.

## Analysis

### Structure

SHA-256 and SHA-512 are the same construction at two word sizes. A message is
padded (section 5.1) with a single 1 bit, zeros, and its length in bits as a
big-endian integer, 64 bits wide for SHA-256 and 128 for SHA-512, so that the
result is a whole number of 512-bit or 1024-bit blocks (section 5.2). A
compression function then folds the blocks, one at a time, into an eight-word
chaining value that starts from a fixed initial value (section 5.3). Each
block is first expanded into a message schedule, 64 words for SHA-256 and 80
for SHA-512, by a recurrence over the previous sixteen words using the two
"small sigma" functions; then the working variables a through h go through
one round per schedule word, each round adding a constant K_t and the word W_t
into two temporaries T1 and T2 built from Ch, Maj and the two "big sigma"
functions; finally the working variables are added back into the chaining
value. The digest is the final chaining value, or its leftmost words for the
truncated variants: SHA-224 (section 6.3) runs SHA-256 from its own initial
value and keeps seven words, SHA-384 (6.5) and SHA-512/256 (6.7) run SHA-512
from theirs and keep six and four.

The file `sha2.or` follows the standard section by section, and the 64-bit
computation is a copy of the 32-bit one over `Word[64]` with the suffix
`_512` on every name the standard shares between the two.

| Standard section | Orange spec |
| --- | --- |
| 3.1, bytes to words, big-endian | `load_be32`, `load_be64`, `be_bytes64` |
| 4.1.2, SHA-256 functions | `ch`, `maj`, `big_sigma0`, `big_sigma1`, `small_sigma0`, `small_sigma1` |
| 4.1.3, SHA-512 functions | the same names with `_512` |
| 4.2.2, K{256} | `round_constants` |
| 4.2.3, K{512} | `round_constants_512` |
| 5.1.1, padding, 64-bit length | `pad_empty`, `pad_3`, `pad_56`, `pad_64`, with `with_length_64` and `with_length_128` (the suffix is the buffer's size in bytes) |
| 5.1.2, padding, 128-bit length | `pad_512_empty`, `pad_512_3`, `pad_512_112`, with `with_length_128` and `with_length_256` |
| 5.2.1, parsing into 32-bit words | `parse_1`, `parse_2` |
| 5.2.2, parsing into 64-bit words | `parse_512_1`, `parse_512_2` |
| 5.3.2 through 5.3.6.2, initial hash values | `initial_hash_224`, `initial_hash_256`, `initial_hash_384`, `initial_hash_512`, `initial_hash_512_256` |
| 6.2.2, SHA-256 hash computation | `schedule`, `round`, `compress`, `hash_two_blocks` |
| 6.3, SHA-224 | `truncate_224` |
| 6.4.2, SHA-512 hash computation | `schedule_512`, `round_512`, `compress_512`, `hash_two_blocks_512` |
| 6.5, SHA-384 | `truncate_384` |
| 6.7, SHA-512/256 | `truncate_512_256` |

Padding is written for each message length the vectors need, because Orange
has no length-generic arrays: `pad_3` takes a `Word[8]^3`, copies it into a
64-byte block with a loop, sets the byte after it to `0x80` (the 1 bit,
since every message here is whole bytes), writes the bit length into the
last eight bytes, and parses the result into words. The 56-byte message
shows the case the standard's example was chosen for: the 1 bit fits in the
first block but the length does not, so the padding spills into a second
block; the 64-byte message fills its block exactly and the whole padding is
a second block. A message of 0 bytes has no array, so `pad_empty` builds its
block from the fill `[0; 64]` directly. `hash_two_blocks` is the standard's
"for i = 1 to N" with N = 2, each block taken out of the parsed message by a
loop with a static index `16 * n + t`.

### Security status

Record as written in September 2026; the newest results cited are from 2024.

SHA-2 is a Merkle-Damgard construction: the digest is the whole final
chaining value. Anyone who knows H(m) and the length of m can therefore
continue the computation and produce H(m || pad || m') for a suffix m' of
their choice without knowing m. This length-extension property is why a
secret-prefix MAC `H(k || m)` is insecure, why HMAC (Bellare, Canetti,
Krawczyk 1996; FIPS 198-1) wraps the hash in two keyed passes, and one of the
reasons the SHA-3 competition asked for designs, such as the Keccak sponge
(FIPS 202, 2015), that do not output their full state. Among the SHA-2
family, the truncated variants leak less: SHA-512/256 withholds 256 bits of
the state and is not subject to length extension in practice, while SHA-224
withholds only 32 bits, a barrier of 2^32 guesses, so it should not be relied
on for that purpose. Merkle-Damgard hashes also admit generic structural
attacks that fall short of breaking the compression function: Joux's
multicollisions (2004), Kelsey and Schneier's second preimages on very long
messages (2005), and the herding attack of Kelsey and Kohno (2006).

The compression function itself has held up. For SHA-256, the best collision
attacks on reduced-step versions reach 31 of the 64 steps (Mendel, Nad and
Schlaffer, Eurocrypt 2013, at about 2^65.5, made practical by Li, Liu and
Wang, Eurocrypt 2024), with semi-free-start collisions, where the attacker
also chooses the chaining value, on 38 steps (Mendel, Nad and Schlaffer 2013)
and 39 steps (Li, Liu and Wang 2024); the 2024 work also gives the first
collision for 31 steps of SHA-512 and semi-free-start collisions for 28
steps. Preimage attacks with bicliques reach 45 steps of SHA-256 and 50 of the
80 steps of SHA-512, and pseudo-preimages 52 and 57 steps (Khovratovich,
Rechberger and Savelieva, FSE 2012), at complexities only marginally below
brute force. No attack on the full
functions is known, and the margin, half of the rounds, has changed little in
a decade of study. The generic security is the design strength: 2^128
collision and 2^256 preimage work for SHA-256, 2^256 and 2^512 for SHA-512,
with Grover's algorithm halving the preimage exponent on a quantum computer
and no practical quantum speedup for collisions.

The context that matters is SHA-1. Its collision resistance was broken in
theory by Wang, Yin and Yu (2005), in practice by the SHAttered collision
(Stevens, Bursztein, Karpman, Albertini and Markov, 2017, about 2^63.1
SHA-1 computations), and turned into chosen-prefix collisions by Leurent and
Peyrin (2020). SHA-2 shares SHA-1's overall shape but not its weak message
expansion, and none of the SHA-1 techniques has carried over to more than
half of SHA-2's rounds. NIST disallowed SHA-1 for digital signatures in 2013
(SP 800-131A) and announced in December 2022 that SHA-1 is to be retired from
all federal use by the end of 2030; SHA-2 and SHA-3 are both approved under
FIPS 180-4 and FIPS 202, and the stateful hash-based signatures of SP 800-208
and the SHA-2 parameter sets of SLH-DSA (FIPS 205, 2024) continue to build on
SHA-256.

Pitfalls that are not attacks on the hash: using SHA-256 as a password hash
(it is fast by design; use a memory-hard function), using `H(k || m)` as a
MAC, and comparing digests with data-dependent early exit.

### What the Orange rendering shows

Nothing in SHA-2 is data-dependent except the values themselves: there are
no tables indexed by data, no rotations by data-dependent amounts, and no
branches. In the Orange file every index is a literal or a loop index and
every shift and rotation amount is a literal, so the checker proves every
access in range before evaluation and the source contains no conditional at
all. The `Word[32]` and `Word[64]` rings give the modular additions their
meaning directly; nothing is masked or cast except the bytes at the
boundaries.

The two computations are visibly the same text at two widths, which is how
the standard presents them, and the truncated variants are visibly the same
computation from a different constant: SHA-224 and SHA-384 differ from
SHA-256 and SHA-512 only in `initial_hash_224` and `initial_hash_384` and in
which words are kept, and SHA-512/256 only in an initial value that is itself
a SHA-512 digest (section 5.3.6), which the generator script recomputes.

Measured costs under `orangec eval` (each spec wrapped in a loop, and the
largest iteration count that still evaluates found by bisection): one
SHA-256 block, padding and parsing of a one-block message included, costs
about 15,000 steps and one SHA-512 block about 21,000; padding a 112-byte
message into two SHA-512 blocks costs about 37,000, most of it the 112
single-byte updates of a 256-byte array at 256 steps each. The thirteen
vectors together use about 486,000 of the 1,048,576-step budget, so every
planned vector sits in one file with room for roughly the same again.

Not expressed: a message of arbitrary length. Each padding spec takes an
array of one fixed length, so a new length needs a new `pad_n`. A single
`Word[8]` array holds at most 256 bytes, so a message longer than 239 bytes
(256 minus the 17 bytes of SHA-512 padding) would have to arrive as several
arrays or as `Word[64]` words; none of the standard's examples needs that.

## Dissemination

### Files

- `sha2.or`: SHA-224, SHA-256, SHA-384, SHA-512 and SHA-512/256 of FIPS
  180-4 with padding for messages of 0, 3, 56, 64 and 112 bytes, and the
  thirteen vector pairs below. One file; the budget did not force a split.

### Running

    orangec eval algorithms/sha2/sha2.or
    python3 algorithms/verify.py algorithms/sha2

### Vectors

Each row is a pair `<spec>` and `<spec>_expected`; `verify.py` requires
them equal. "NIST examples" are the digests of NIST's "Examples with
Intermediate Values" for SHA-256 and SHA-512, the messages FIPS 180-2
carried in its appendices; NIST's site is unreachable from the build
machine, so their values were read from Botan's and OpenSSL's vector files,
which reproduce them, and every value was recomputed with `hashlib`.

| Spec | Source | Case |
| --- | --- | --- |
| `botan_sha256_empty` | Botan `src/tests/data/hash/sha2_32.vec`, `[SHA-256]`, `In =` (empty); `hashlib` | SHA-256 of the empty message |
| `nist_sha256_abc` | NIST examples; Botan `sha2_32.vec`, `In = 616263`; OpenSSL `evpmd_sha.txt`, `Digest = SHA256`, `Input = "abc"`; `hashlib` | SHA-256 of "abc", one block |
| `nist_sha256_two_block` | NIST examples; Botan `sha2_32.vec`; OpenSSL `evpmd_sha.txt`; `hashlib` | SHA-256 of the 56-byte "abcdbcde...nopq", two blocks |
| `botan_sha256_64_bytes` | Botan `sha2_32.vec`, `[SHA-256]`, `In = 3b47876f...ba43524d`; `hashlib` | SHA-256 of 64 bytes, padding a whole second block |
| `openssl_sha224_abc` | OpenSSL `test/recipes/30-test_evp_data/evpmd_sha.txt`, `Digest = SHA224`, `Input = "abc"`; `hashlib` | SHA-224 of "abc" |
| `openssl_sha224_two_block` | OpenSSL `evpmd_sha.txt`, `Digest = SHA224`, the 56-byte message; `hashlib` | SHA-224 of "abcdbcde...nopq" |
| `hashlib_sha512_empty` | `hashlib.sha512(b"")` (no fetched file carries it) | SHA-512 of the empty message |
| `nist_sha512_abc` | NIST examples; Botan `sha2_64.vec`, `[SHA-512]`, `In = 616263`; OpenSSL `evpmd_sha.txt`, `Digest = SHA512`; `hashlib` | SHA-512 of "abc", one block |
| `nist_sha512_two_block` | NIST examples; Botan `sha2_64.vec`; OpenSSL `evpmd_sha.txt`; `hashlib` | SHA-512 of the 112-byte "abcdefgh...nopqrstu", two blocks |
| `botan_sha384_abc` | Botan `sha2_64.vec`, `[SHA-384]`, `In = 616263`; `hashlib` | SHA-384 of "abc" |
| `botan_sha384_two_block` | Botan `sha2_64.vec`, `[SHA-384]`, the 112-byte message; `hashlib` | SHA-384 of "abcdefgh...nopqrstu" |
| `openssl_sha512_256_abc` | OpenSSL `evpmd_sha.txt`, `Digest = SHA512-256`, `Input = "abc"`; `hashlib` | SHA-512/256 of "abc" |
| `openssl_sha512_256_two_block` | OpenSSL `evpmd_sha.txt`, `Digest = SHA512-256`, the 112-byte message; `hashlib` | SHA-512/256 of "abcdefgh...nopqrstu" |

The Botan files are `randombit/botan`, `src/tests/data/hash/sha2_32.vec` and
`sha2_64.vec`; the OpenSSL file is `openssl/openssl`,
`test/recipes/30-test_evp_data/evpmd_sha.txt`; both at `master` on the day
of writing.

### Provenance and claims

Every constant in `sha2.or` was derived from its definition in FIPS 180-4
by a generator script, kept with the worker's notes outside the repository,
using exact integer roots: K{256} and K{512} as the first 32 or 64 bits of the fractional parts
of the cube roots of the first 64 or 80 primes, the SHA-256, SHA-384 and
SHA-512 initial values from the square roots of the first sixteen primes,
the SHA-224 initial value as the second 32 bits of the square roots of the
ninth through sixteenth primes, and the SHA-512/256 initial value by the
generation function of section 5.3.6 (SHA-512 with the a5...a5 mask over
the string "SHA-512/256"). Each table was then matched, as a contiguous
sequence, against the tables in OpenSSL's `crypto/sha/sha256.c` and
`crypto/sha/sha512.c` and in B-Con's `sha256.c`, and the Orange literals were
printed by the same script, never typed. The script also holds an
independent Python SHA-256 and SHA-512, with the padding of section 5.1,
whose output was checked against `hashlib` and against every published
value above before the `_expected` literals were emitted; every `_expected`
value in the file is copied from a named vector file or from `hashlib`, and
none was adjusted to match the Orange computation.

This entry is a reference evaluation of a specification under
`orangec eval`. It makes no constant-time, side-channel, performance or
certification claim, it has not been independently reviewed, and it is not a
corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- No length-generic arrays or specs, so padding is written once per message
  length (`pad_3`, `pad_56`, `pad_64`, `pad_512_3`, `pad_512_112`) instead of
  once. Each is five lines; the cost is repetition, not expressiveness.
- Arrays hold at most 256 elements, so a `Word[8]` message is at most 239
  bytes for SHA-512 (256 less the 17 bytes of padding) and 247 for SHA-256
  in this style. No vector here needs more.
- Single-element updates cost the array's length in steps, so padding a
  112-byte message into a 256-byte array (about 37,000 steps) costs more
  than the two SHA-512 compressions it feeds (about 19,500 each). It did not
  threaten the budget.
