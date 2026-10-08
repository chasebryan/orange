# RC4

RC4 is a byte-oriented stream cipher designed by Ronald Rivest at RSA Data
Security in 1987 and kept as a trade secret until its source was posted
anonymously to the Cypherpunks mailing list in September 1994, where it
took the names "Arcfour" and "ARC4" that libraries still use. It has no
standard of its own: the algorithm is the two loops of that posting, a
key-scheduling algorithm (KSA) that turns a key of 1 to 256 bytes into a
permutation S of the 256 byte values, and a pseudo-random generation
algorithm (PRGA) that walks S with two indices, swapping as it goes, and
emits one byte of keystream per step; encryption is xor with the
keystream. Its test vectors are those of
[RFC 6229](https://www.rfc-editor.org/rfc/rfc6229) (Strombergson and
Josefsson, 2011), which gives keystream at eighteen offsets for keys of 40
to 256 bits. RC4 was the most widely deployed stream cipher of the 1990s and
2000s: WEP and WPA-TKIP, SSL and TLS, SSH-1, Kerberos, Microsoft Office and
PDF encryption. It is broken, by the keystream biases published from 2001
to 2015; [RFC 7465](https://www.rfc-editor.org/rfc/rfc7465) prohibits it in
TLS (2015), and it is kept here because it remains the canonical example of
a biased keystream that cryptanalysts study.

## Analysis

### Structure

The state is a permutation S of the values 0 through 255, held in a
256-byte array, and two byte indices i and j. The KSA starts from the
identity permutation and j = 0 and, for i from 0 to 255, sets
j := j + S[i] + K[i mod L] (all arithmetic modulo 256, L the key length in
bytes) and swaps S[i] and S[j]. The PRGA resets i and j to 0 and, for each
output byte, sets i := i + 1 and j := j + S[i], swaps S[i] and S[j], and
outputs S[S[i] + S[j]]. That is the whole cipher: every step of both loops
reads and writes S at an index that the key or the previous state chose.

The Orange file `rc4.or` follows the two loops as they are written, with
these correspondences:

| RC4 | Orange spec |
| --- | --- |
| S, a permutation of the 256 byte values | `Permutation`, a `Word[8]^256` indexed by a byte |
| S, i and j, the PRGA's state | `State`, the tuple `(Permutation, Word[8], Word[8])` |
| KSA: S = identity, then for i = 0 to 255, j := j + S[i] + K[i mod L], swap(S[i], S[j]) | `key_scheduling[L]` for `L` in 1 through 256 |
| PRGA start: i = j = 0 | `start` |
| PRGA step: i := i + 1, j := j + S[i], swap(S[i], S[j]) | `advance` |
| PRGA output: S[S[i] + S[j]] | `output` |
| n bytes generated and discarded (RFC 6229's offsets, "RC4-drop[n]") | `discard[n]` for `n` in 1 through 256 |
| n bytes of keystream | `keystream[n]` for `n` in 1 through 256 |
| ciphertext = message xor keystream | `encrypt[n]` for messages of 1 through 256 bytes |

S is an array of 256 bytes and i and j are bytes, so the arithmetic modulo
256 that the posting writes is the arithmetic of `Word[8]`, and every index
into S, whether a loop index, i, j or S[i] + S[j], is a byte that the
checker proves in range. There is one KSA for every key length: the key is a
`Word[8]^L` with the length a size parameter, and the key index is written
`key[i % L]` as the posting writes K[i mod L].

### Security status

RC4's keystream is not uniformly distributed, and the record of its
cryptanalysis is the record of finding, measuring and exploiting those
biases. Roos (1995) observed, from the Cypherpunks source, that the KSA
leaves the first entries of S correlated with the first key bytes
(S[i] equals i(i + 1)/2 plus the sum of the first i + 1 key bytes with
probability about 0.37 for small i), and exhibited classes of weak keys.
Golic (1997) and Fluhrer and McGrew (2000) gave distinguishers from
digraph statistics of the keystream, the latter needing about 2^30.6
bytes. Mantin and Shamir (2001) found that the second output byte is 0
with probability about 1/128, twice the uniform value, and used it in a
broadcast attack that recovers the second byte of a plaintext encrypted
under a few hundred different keys. Fluhrer, Mantin and Shamir (2001)
showed that when part of the key is known, as in WEP where a 24-bit IV is
prepended to the secret key, the first keystream byte leaks the next key
byte for a class of "resolved" IVs; Stubblefield, Ioannidis and Rubin
(2001) implemented it against WEP within weeks, and the later refinements
of Klein (2005) and Tews, Weinmann and Pyshkin (2007) recover a 104-bit
WEP key from about 40,000 frames. Mironov (2002) analyzed the KSA as an
incomplete shuffle and recommended discarding the initial keystream, the
"RC4-drop[n]" of later usage; the discarded prefix does not remove the
biases that follow.

The attacks that ended RC4's deployment target TLS, where it had become
the most used cipher after the CBC attacks of 2011 to 2013 (BEAST, Lucky
13). AlFardan, Bernstein, Paterson, Poettering and Schuldt (2013) measured
the single-byte distributions of the first 256 keystream bytes over 2^44
keys, found every position biased, and recovered the first 256 bytes of a
plaintext repeated under many keys from about 2^28 to 2^32 encryptions,
and later bytes from the Fluhrer-McGrew double-byte biases with about
13 x 2^30 encryptions. Isobe, Ohigashi, Watanabe and Morii (2013) gave a
full plaintext recovery in the broadcast setting from about 2^34
ciphertexts. Garman, Paterson and van der Merwe (2015) brought the
password-recovery cost against Basic authentication and IMAP down to
2^26 encryptions. Vanhoef and Piessens (2015), "RC4 NOMORE", combined the
Fluhrer-McGrew and Mantin ABSAB biases with a list of candidate plaintexts
and decrypted a TLS cookie in 75 hours from about 9 x 2^27 encryptions,
and broke WPA-TKIP within an hour. State recovery from keystream alone,
without any bias, remains expensive (Knudsen, Meier, Preneel, Rijmen and
Verdoolaege, 1998, about 2^779; Maximov and Khovratovich, 2008, about
2^241), which is why the practical attacks are the statistical ones. The
attack complexities above are as published; none of the papers was
fetched from this machine.

The consequences: RFC 7465 (February 2015) prohibits RC4 cipher suites in
every TLS version, requiring clients not to offer them and servers not to
select them; Chrome 48 and Firefox 44 removed RC4 in January 2016 and
Microsoft's Internet Explorer 11 and Edge followed in 2016; RFC 8429 (2018)
deprecates the RC4 Kerberos encryption types; NIST SP 800-52 Revision 2
(2019) does not allow RC4 in TLS for federal use; the Wi-Fi Alliance
deprecated TKIP with WPA3 (2018). Standing as of September 2026: RC4 is
broken in every setting where an attacker can see many encryptions of
related plaintexts, which is every network protocol, and prohibited in
TLS; it is not a design margin question, since the biases are properties of
the full cipher, not of a reduced round count. It is kept here as the
canonical example of a biased keystream and of a key schedule that leaks
its key.

### What the Orange rendering shows

Every access to S in RC4 is at an index the data chose except S[i], and in
Orange each one is written as the posting writes it: S[j] is `s[j1]`, the
output is `s[s[i] + s[j]]`, and the swap is the two updates
`(s with [i] = s[j1]) with [j1] = s[i]`, both reading the S from before
it. Each index is a `Word[8]` into an array of 256, so the checker proves
it in range and nothing is masked or reduced by hand. These lookups carry
no timing claim: an access to S keyed by secret data is the cache-timing
pattern, and the evaluator makes no attempt to hide it. The PRGA's i
depends only on the step count, so only j and the output index
S[i] + S[j] are chosen by the data; the file still carries i as a byte
from step to step, as the posting does, rather than deriving it from the
loop index.

Lengths are sizes. The key is a `Word[8]^L`, so one `key_scheduling` serves
the 40-, 64-, 128- and 256-bit keys of the vectors, the instance chosen by
the length of the key's `hex"..."` literal. `keystream[n]`, `discard[n]` and
`encrypt[n]` take their length the same way, `encrypt` from its message and
the other two written at the call (`keystream[32]`), since their argument
is the state and not an array of that length. The RFC's offsets are
`discard`: the rows at offsets 240 and 256 are
`keystream[32](discard[240](start(key_scheduling(key))))`. The PRGA's
state is a tuple, `State`, and a tuple cannot hold another tuple, so
`keystream`'s loop carries S, i and j beside its output bytes rather than a
`State` beside them. A step names the j it computes `j1`, and the PRGA's new
i `i1`, because a name is bound once.

Measured costs under `orangec test --stats`, and `orangec eval --stats` for
the parts: building the identity permutation costs 2,310 steps and the key
schedule 12,558 in all, about 40 steps for each of its 256 swaps, the same
for every key length. A keystream byte costs about 95 steps and a discarded
byte about 45; the keystream costs more because its loop takes the state
apart and puts it back together around each step to write the output byte.
A vector of 32 keystream bytes costs about 15,600 steps with its key
schedule, the offset-240 vector 26,428 and the encryption 15,983; the five
tests together use 89,290 steps.

The RFC's rows at offsets 496 and beyond, up to 4096, are within reach (a
row at offset 4096 would cost the key schedule and about 185,000 steps of
discarded keystream, in sixteen calls of `discard[256]`), but the first
form did not reproduce them and this rewrite adds no vector.

## Dissemination

### Files

- `rc4.or`: the KSA for keys of 1 to 256 bytes, the PRGA with `discard`
  and `keystream`, `encrypt` as xor, the RFC 6229 keystream at offsets 0
  and 16 for the 40-, 128- and 256-bit keys and at offsets 240 and 256 for
  the 40-bit key, and the encryption of a message from Botan's `rc4.vec`.

### Running

    orangec test algorithms/rc4/rc4.or
    python3 algorithms/verify.py algorithms/rc4

### Vectors

Each row is a `test` block in `rc4.or`, comparing 32 bytes of keystream or
ciphertext with the published value.

| Test | Source | Case |
| --- | --- | --- |
| `RFC 6229 section 2: 40-bit key, offsets 0 and 16` | RFC 6229, section 2, key length 40 bits, rows at offsets 0 and 16 | key 0102030405, keystream bytes 0 through 31, b2396305...7a0d0919 |
| `RFC 6229 section 2: 128-bit key, offsets 0 and 16` | RFC 6229, section 2, key length 128 bits, rows at offsets 0 and 16 | key 0102030405060708090a0b0c0d0e0f10, keystream bytes 0 through 31, 9ac7cc9a...1d1a9e1c |
| `RFC 6229 section 2: 256-bit key, offsets 0 and 16` | RFC 6229, section 2, key length 256 bits, rows at offsets 0 and 16 | key 01020304...1d1e1f20 (32 bytes), keystream bytes 0 through 31, eaa6bd25...7cb14380 |
| `RFC 6229 section 2: 40-bit key, offsets 240 and 256` | RFC 6229, section 2, key length 40 bits, rows at offsets 240 and 256 | key 0102030405, keystream bytes 240 through 271, 28cb1132...7f8d8c93 |
| `Botan rc4.vec [RC4] case 5: encryption of 0x01 bytes, first 32` | Botan `src/tests/data/stream/rc4.vec`, section `[RC4]`, fifth case (Key = 0123456789ABCDEF, In = 0101...) | the first 32 bytes of the message of 0x01 bytes encrypted under the 64-bit key of the 1994 posting, 7595c3e6...778dcad8 |

The four RFC 6229 rows were taken from the copy of the RFC's vectors that
the `cryptography` project keeps in its `vectors/` tree
(`ciphers/ARC4/rfc-6229-{40,128,256}.txt`, which reformat the RFC's tables
as KEY, OFFSET, CIPHERTEXT of an all-zero plaintext), cross-checked against
OpenSSL's `test/recipes/30-test_evp_data/evpciph_rc4.txt`, which carries
the offset-0 rows of the 40- and 128-bit keys, and against both oracles.
The Botan row is the fifth case of the `[RC4]` section of Botan's file, a
512-byte message truncated to its first 32 bytes, which is sound for a
stream cipher; the full case was checked against the oracle before
truncation. No `_expected` value was produced by an oracle alone; every one
is copied from a fetched vector file.

### Provenance and claims

RFC 6229 and RFC 7465 are not reachable from the build, so the vectors
came from the three mirrors above, all fetched from
`raw.githubusercontent.com` and kept in the scratch directory: the
`cryptography` project's `rfc-6229-*.txt` files (seven key lengths, 36
rows each), OpenSSL's `evpciph_rc4.txt`, and Botan's `rc4.vec`, with Go's
`crypto/rc4/rc4_test.go` for the 1994 posting's cases. A script
(`crosscheck.py`) ran all 252 RFC 6229 rows and all 69 `[RC4]` cases of
Botan's file through pycryptodome's `Crypto.Cipher.ARC4`, the
`cryptography` package's `ARC4` and a plain Python RC4 written from the
two loops (`rc4_ref.py`, which can also dump S, i and j after any step);
all agree. The Orange literals, the packed identity permutation, the keys
and the expected keystreams, were generated from the fetched files by
`gen_literals.py`, not typed. Section 2 of RFC 6229 is cited as the
section that holds the test vectors, as the document is known to the
author; the number was not checked from this machine. The Orange files
matched all five vectors on their first evaluation, so `rc4_ref.py`'s
step-by-step dump was not needed. Costs were measured by the loop-and-
binary-search method of the folder's brief (`measure.py`) and by filling
each file's remaining budget with a calibrated loop (`headroom.py`).

The entry was then rewritten in the current language, the two files
folded into one. Every expected value is carried over byte for byte from
the first form, where each was a `<name>_expected` spec of 32 byte
literals: the new tests state the same bytes as `hex"..."` in the RFC's
16-byte rows, compared by script with the first form's values, and no
vector was added or dropped. The packed identity permutation of the first
form is gone; the KSA builds S = identity with its first loop, as the
posting does. Costs are as `orangec test --stats` reports them.

This entry is a reference evaluation of RC4 as the 1994 posting and RFC
6229 describe it, under `orangec test`. It makes no constant-time,
side-channel, performance or certification claim, and it is not a corpus
entry in the sense of The Orange Book chapter 12. It is not a
recommendation to use RC4 for anything.

## Gaps

None that prevents a vector here. The language limits met, and what they
cost:

- A function has at most 256 instances, so `keystream`, `discard` and
  `encrypt` cover 1 to 256 bytes a call. A longer discard is a chain of
  calls (`discard[256]` then `discard[n]`), but `encrypt` starts the PRGA
  afresh, so a message longer than 256 bytes would need an `encrypt` that
  takes and returns the PRGA's state, which this file does not write. No
  vector here needs more than 32 bytes of keystream after the discard.
- No array has zero elements, so the empty message is not a value; every
  key RC4 admits, 1 to 256 bytes, is.
- `keystream[n]` and `discard[n]` have no array parameter of their length,
  so each call names the length (`keystream[32]`); the checker cannot infer
  it from the expected value.
- A tuple cannot hold a tuple, so `keystream`'s loop carries S, i and j as
  three elements beside the output bytes instead of one `State`, and takes
  the state apart and back together around each step.
