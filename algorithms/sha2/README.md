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

The file `sha2.or` follows the standard section by section. The 64-bit
computation is a copy of the 32-bit one over `Word[64]`, with the suffix
`_512` on every name the standard shares between the two, except Ch and Maj:
sections 4.1.2 and 4.1.3 define them by the same formulas at both widths, so
one spec with a type parameter `W in {Word[32], Word[64]}` serves both.

| Standard section | Orange spec |
| --- | --- |
| 4.1.2 and 4.1.3, Ch and Maj | `ch`, `maj` (over `Word[32]` and `Word[64]`) |
| 4.1.2, SHA-256 functions | `big_sigma0`, `big_sigma1`, `small_sigma0`, `small_sigma1` |
| 4.1.3, SHA-512 functions | the same names with `_512` |
| 4.2.2, K{256} | `round_constants` |
| 4.2.3, K{512} | `round_constants_512` |
| 5.1.1 and 5.1.2, padding | `padding[len, s]` for `len` in 0 through 112, `s` = 1 (5.1.1) or 2 (5.1.2) |
| 5.2.1 and 5.2.2, parsing into words | `as big Word[32]^16` and `as big Word[64]^16` on each block, in `hash` and `hash_512` |
| 5.3.2 through 5.3.6.2, initial hash values | `initial_hash_224`, `initial_hash_256`, `initial_hash_384`, `initial_hash_512`, `initial_hash_512_256` |
| 6.2.2, SHA-256 hash computation | `schedule`, `round`, `compress`, `hash[n]`, `sha256[n]` |
| 6.3, SHA-224 | `sha224[n]` |
| 6.4.2, SHA-512 hash computation | `schedule_512`, `round_512`, `compress_512`, `hash_512[n]`, `sha512[n]` |
| 6.5, SHA-384 | `sha384[n]` |
| 6.7, SHA-512/256 | `sha512_256[n]` |

Padding is one spec with two size parameters: the message length `len` in
bytes, and `s`, which picks the subsection. `s` is w / 32 = m / 512, the
standard's word size w and block size m in units of SHA-256's, so `s = 1` is
section 5.1.1 (SHA-224 and SHA-256) and `s = 2` is section 5.1.2 (the
64-bit variants); a block is `64 s` bytes and the length field, two words,
`8 s` bytes. `padding[len, s]()` is the string the standard appends: the
byte `0x80` (the 1 bit and the first seven zero bits, since every message
here is whole bytes) opening a fill of zeros, then `l = 8 len` as a
big-endian integer of `8 s` bytes, `64 s ((len + 72 s) / (64 s)) - len`
bytes in all, so that the message followed by it is a whole number of
blocks. It returns the appended string rather than the padded message
because an Orange array has at least one element: the empty message is no
array, and its padded form is `padding[0, s]()` alone. The 56-byte message
shows the case the standard's example was chosen for: the 1 bit fits in the
first block but the length does not, so the padding spills into a second
block; the 64-byte message fills its block exactly and the whole padding is
a second block. The 112-byte message does the same for the 64-bit variants
as the 56-byte one does for SHA-256.

The hash computations take the padded message as `Word[8]^(64 * n)` (or
`128 * n`) for `n` of one or two blocks, and run the standard's "for i = 1
to N" as a loop over the blocks, each block sliced out and parsed
(section 5.2) by `as big Word[32]^16`. The digest is the final hash value
read back to bytes with `as big`, whole for SHA-256 and SHA-512 and its
leftmost seven, six or four words for SHA-224, SHA-384 and SHA-512/256.

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
branches. In the Orange file every index is a literal or a loop index,
every slice bound is a literal or a loop index times the block length, plus
the block length, and every shift and rotation amount is a literal, so the
checker proves every access in range before evaluation and the source
contains no conditional at all. The `Word[32]` and `Word[64]` rings give the
modular additions their meaning directly; nothing is masked or cast.

Byte order is written once, where the standard fixes it: `as big` parses a
64-byte or 128-byte block into its sixteen words (section 5.2), writes the
bit length into the padding, reads the round constants from `hex"..."` rows
printed as section 4.2 prints them, and turns the final hash value into the
digest's bytes. Lengths are sizes: `padding[len, s]` has one instance per
message length from 0 through 112 bytes at each of the two sizes, and the
checker computes each instance's length from the formula in its signature,
so a test that names the wrong length for its message finds no instance of
`sha256` that takes the result, unless the two lengths differ by a whole
block (a 64-byte message with `padding[0, 1]()`). A test reads as the
standard's example: `sha256("abc" ++ padding[3, 1]())` against the digest
in hex. The messages are string literals, and the 64-byte Botan message is
hex.

The two computations are visibly the same text at two widths, which is how
the standard presents them, and the truncated variants are visibly the same
computation from a different constant: SHA-224 and SHA-384 differ from
SHA-256 and SHA-512 only in `initial_hash_224` and `initial_hash_384` and in
which words are kept, and SHA-512/256 only in an initial value that is itself
a SHA-512 digest (section 5.3.6), which the generator script recomputes.

Measured costs under `orangec test --stats` and `orangec eval --stats`:
each block adds 8,935 steps to `hash` and 11,543 to `hash_512`, of which a
call to `compress` is 8,912 and a call to `compress_512` 11,511, almost all
of it the message schedule and the rounds; the rest is slicing out the
block and parsing it. Within a compression, the call to `round_constants`
costs 60 steps and the call to `round_constants_512` 210. A padding call
costs 12 to 14 steps at `s = 1` and 14 to 18 at `s = 2`. A one-block
SHA-256 or SHA-224 test costs 8,973 to 8,979 steps, a two-block one 17,914
to 17,917; for the 64-bit variants the figures are 11,591 to 11,596 and
23,142 to 23,146. The thirteen tests together use 196,481 steps.

Not expressed: a message whose length is not a whole number of bytes (the
standard's l may be any number of bits), and a message longer than the
size ranges. `padding` covers the lengths listed above and each hash one
or two blocks; widening a range is a change to one number, up to the
limits under Gaps.

## Dissemination

### Files

- `sha2.or`: SHA-224, SHA-256, SHA-384, SHA-512 and SHA-512/256 of FIPS
  180-4, with padding for messages of 0 through 112 bytes at both block
  sizes, and the thirteen tests below.

### Running

    orangec test algorithms/sha2/sha2.or
    python3 algorithms/verify.py algorithms/sha2

`orangec eval` prints every parameterless spec, and each instance of
`padding` is one: 226 padding strings (113 lengths at two sizes) before the
constants.

### Vectors

Each row is a `test` block in `sha2.or`, comparing a digest with the
published value. "NIST examples" are the digests of NIST's "Examples with
Intermediate Values" for SHA-256 and SHA-512, the messages FIPS 180-2
carried in its appendices; NIST's site is unreachable from the build
machine, so their values were read from Botan's and OpenSSL's vector files,
which reproduce them, and every value was recomputed with `hashlib`.

| Test | Source | Case |
| --- | --- | --- |
| `Botan sha2_32.vec: SHA-256 of the empty message` | Botan `src/tests/data/hash/sha2_32.vec`, `[SHA-256]`, `In =` (empty); `hashlib` | SHA-256 of the empty message |
| `NIST example: SHA-256 of abc` | NIST examples; Botan `sha2_32.vec`, `In = 616263`; OpenSSL `evpmd_sha.txt`, `Digest = SHA256`, `Input = "abc"`; `hashlib` | SHA-256 of "abc", one block |
| `NIST example: SHA-256 of the two-block message` | NIST examples; Botan `sha2_32.vec`; OpenSSL `evpmd_sha.txt`; `hashlib` | SHA-256 of the 56-byte "abcdbcde...nopq", two blocks |
| `Botan sha2_32.vec: SHA-256 of a 64-byte message` | Botan `sha2_32.vec`, `[SHA-256]`, `In = 3b47876f...ba43524d`; `hashlib` | SHA-256 of 64 bytes, padding a whole second block |
| `OpenSSL evpmd_sha.txt: SHA-224 of abc` | OpenSSL `test/recipes/30-test_evp_data/evpmd_sha.txt`, `Digest = SHA224`, `Input = "abc"`; `hashlib` | SHA-224 of "abc" |
| `OpenSSL evpmd_sha.txt: SHA-224 of the two-block message` | OpenSSL `evpmd_sha.txt`, `Digest = SHA224`, the 56-byte message; `hashlib` | SHA-224 of "abcdbcde...nopq" |
| `hashlib: SHA-512 of the empty message` | `hashlib.sha512(b"")` (no fetched file carries it) | SHA-512 of the empty message |
| `NIST example: SHA-512 of abc` | NIST examples; Botan `sha2_64.vec`, `[SHA-512]`, `In = 616263`; OpenSSL `evpmd_sha.txt`, `Digest = SHA512`; `hashlib` | SHA-512 of "abc", one block |
| `NIST example: SHA-512 of the two-block message` | NIST examples; Botan `sha2_64.vec`; OpenSSL `evpmd_sha.txt`; `hashlib` | SHA-512 of the 112-byte "abcdefgh...nopqrstu", two blocks |
| `Botan sha2_64.vec: SHA-384 of abc` | Botan `sha2_64.vec`, `[SHA-384]`, `In = 616263`; `hashlib` | SHA-384 of "abc" |
| `Botan sha2_64.vec: SHA-384 of the two-block message` | Botan `sha2_64.vec`, `[SHA-384]`, the 112-byte message; `hashlib` | SHA-384 of "abcdefgh...nopqrstu" |
| `OpenSSL evpmd_sha.txt: SHA-512/256 of abc` | OpenSSL `evpmd_sha.txt`, `Digest = SHA512-256`, `Input = "abc"`; `hashlib` | SHA-512/256 of "abc" |
| `OpenSSL evpmd_sha.txt: SHA-512/256 of the two-block message` | OpenSSL `evpmd_sha.txt`, `Digest = SHA512-256`, the 112-byte message; `hashlib` | SHA-512/256 of "abcdefgh...nopqrstu" |

The Botan files are `randombit/botan`, `src/tests/data/hash/sha2_32.vec` and
`sha2_64.vec`; the OpenSSL file is `openssl/openssl`,
`test/recipes/30-test_evp_data/evpmd_sha.txt`; both at `master` on the day
of writing.

### Provenance and claims

Every constant in `sha2.or` was derived from its definition in FIPS 180-4 by a
generator script, kept with the worker's notes outside the repository, using
exact integer roots: K{256} and K{512} as the first 32 or 64 bits of the
fractional parts of the cube roots of the first 64 or 80 primes, the SHA-256,
SHA-384 and SHA-512 initial values from the square roots of the first sixteen
primes, the SHA-224 initial value as the second 32 bits of the square roots of
the ninth through sixteenth primes, and the SHA-512/256 initial value by the
generation function of section 5.3.6 (SHA-512 with the a5...a5 mask over the
string "SHA-512/256"). Each table was then matched, as a contiguous sequence,
against the tables in OpenSSL's `crypto/sha/sha256.c` and
`crypto/sha/sha512.c` and in B-Con's `sha256.c`, and the Orange literals were
printed by the same script, never typed. The script also holds an independent
Python SHA-256 and SHA-512, with the padding of section 5.1, whose output was
checked against `hashlib` and against every published value above before the
`_expected` literals were emitted; every expected value in the file (formerly
an `_expected` spec) is copied from a named vector file or from `hashlib`, and
none was adjusted to match the Orange computation.

The entry was then rewritten in the current language. Every expected value
is carried over byte for byte from the first form, where each was a
`<name>_expected` spec of words: the new tests state the same digests as
the big-endian bytes FIPS 180-4 defines the digest to be, printed from the
old values by a script and compared with them, and no vector was added or
dropped. The three message arrays of the first form (56, 112 and 64
bytes) became string and hex literals, compared byte for byte with the
first form. The round-constant tables, now `hex"..."` rows read with
`as big`, were printed by script from the first form's word literals, and
the initial hash values are unchanged.

This entry is a reference evaluation of a specification under
`orangec test`. It makes no constant-time, side-channel, performance or
certification claim, it has not been independently reviewed, and it is not a
corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- No array has zero elements, so the empty message is not a value: the
  padding specs return what is appended, the hash specs take the padded
  message, and each test joins its message to the padding itself.
- A sized spec with no array parameter needs its size written at the call,
  so each test names its message length twice, in the literal and in
  `padding[3, 1]()`; the checker rejects a mismatch unless the two lengths
  differ by a whole block, but cannot infer the length.
- A function has at most 256 instances, counting every combination of its
  sizes, so `padding`, at two block sizes, covers at most 128 message
  lengths; a hash over arbitrary lengths would pad whole blocks and the
  final partial block separately. No vector here needs more than 112 bytes.
- Messages are whole bytes; a message of a bit length that is not a
  multiple of 8 (which FIPS 180-4 allows) would need its last byte padded
  by hand.
