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

The Orange file follows the two loops as they are written, with these
correspondences:

| RC4 | Orange spec |
| --- | --- |
| S, j, and the keystream bytes, as one state | the `Word[64]^38` layout described at the top of `rc4.or` |
| S = identity, j = 0 | `initial_state` |
| S[x] for a data byte x | `get_byte` (selection over `byte_at`) |
| S[x] := v for a data byte x | `set_byte` (selection over `put_byte`) |
| S[i] for the loop index i | `byte_at(t[i / 8], (i % 8) as Word[8])`, inline |
| swap(S[i], S[j]) | `swap_at_j` (S[j] := S[i], holding the old S[j]), then S[i] := held byte, inline |
| KSA for L = 5, 16, 32 (and 8 in the second file) | `key_scheduling_40`, `key_scheduling_128`, `key_scheduling_256`, `key_scheduling_64` |
| PRGA, 32 bytes from i = j = 0 | `keystream` |
| PRGA, 240 bytes discarded, then bytes 240 through 271 | `discard_240`, `keystream_240` (second file) |
| ciphertext = message xor keystream | `encrypt` (second file) |

The permutation is packed eight entries to a 64-bit word, big-endian, so
that `0x0001020304050607` is S[0] through S[7] of the identity, and the 32
words travel through the loops in one array together with j (word 32), a
byte held between the two halves of a swap (word 33), and 32 keystream
bytes (words 34 through 37). The KSA's key index i mod L has a literal
modulus, so each key length has its own copy of the six-line loop around
the shared `swap_at_j`, as the AES key expansion has one per key length.

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

Every access to S in RC4 is data-dependent except S[i], and Orange makes
each one a visible selection. An index in Orange is static, so S[x] for a
data byte x cannot be written `s[x]`: `get_byte` walks the 32 packed words
with a static index, keeps the one whose number equals x / 8, and selects
its byte x mod 8 with an eight-arm conditional, at a measured 291 steps;
`set_byte` does the same walk and replaces the byte in the one word at 344
steps. S[i], with i the loop index, is the ordinary static index `t[i / 8]`
and byte `i % 8` of that word, a few steps. The PRGA's i is (n + 1) mod 256
for output byte n, so it too is static; only j and the output index
S[i] + S[j] are data. The cost accounting makes the count of data-dependent
accesses plain: one KSA step is one `get_byte` (S[j]), one `set_byte`
(S[j] := S[i]) and one static write (S[i] := old S[j]), measured at about
880 steps, so the 256 steps of a key schedule cost about 225,000 steps for
every key length; one PRGA byte adds the output lookup, `get_byte` twice
(S[j] and then S[S[i] + S[j]]), for about 1,700 steps, and a discarded byte
about 900.

Two things are written otherwise than the standard's text, and both are
explained where they happen. The swap is two steps of an inner loop:
`swap_at_j` reads S[j], writes S[j] := S[i] and holds j and the old S[j] in
words 32 and 33; the next step writes S[i] from word 33 at the static index.
A loop step may begin with `let` bindings; this source keeps the two-step
form. And the key index K[i mod L] has a literal modulus, so the KSA loop
appears once per key length. The keystream bytes are packed into the same
array as S, and `keystream` unpacks them into a `Word[8]^32` at the end.

The budget sized the vectors. One vector, a key schedule and 32 keystream
bytes, costs about 280,000 steps; three of them, the three key lengths of
`rc4.or`, evaluate at about 840,000 of the 1,048,576 steps of a file, and
a fourth does not fit. The RFC's rows at offsets 240 and 256 need 240 bytes
generated and discarded before the 32 kept, about 500,000 steps with the
key schedule, so they are in `rc4-offset-240.or`, with the encryption
example, about 280,000 more; that file evaluates at about 780,000 steps.
The RFC's rows at offsets 496 and beyond, up to 4096, were not attempted:
each further 256-byte block of discarded keystream costs about 230,000
steps, so offset 496 would need a file of its own and offsets 1008 and
beyond exceed the budget of one file. Nothing else was dropped.

## Dissemination

### Files

- `rc4.or`: the KSA for 40-, 128- and 256-bit keys, the PRGA for 32 bytes,
  and the RFC 6229 keystream at offsets 0 and 16 for the three keys.
- `rc4-offset-240.or`: the same algorithm with the KSA for 40- and 64-bit
  keys, the PRGA discarding 240 bytes and then keeping bytes 240 through
  271, `encrypt` as xor, the RFC 6229 keystream at offsets 240 and 256 for
  the 40-bit key, and the encryption of a message from Botan's `rc4.vec`.
  It exists because one file's step budget holds three key schedules.

### Running

    orangec eval algorithms/rc4/rc4.or
    orangec eval algorithms/rc4/rc4-offset-240.or
    python3 algorithms/verify.py algorithms/rc4

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `rfc6229_key_40` | RFC 6229, section 2, key length 40 bits, rows at offsets 0 and 16 | key 0102030405, keystream bytes 0 through 31, b2396305...7a0d0919 |
| `rfc6229_key_128` | RFC 6229, section 2, key length 128 bits, rows at offsets 0 and 16 | key 0102030405060708090a0b0c0d0e0f10, keystream bytes 0 through 31, 9ac7cc9a...1d1a9e1c |
| `rfc6229_key_256` | RFC 6229, section 2, key length 256 bits, rows at offsets 0 and 16 | key 01020304...1d1e1f20 (32 bytes), keystream bytes 0 through 31, eaa6bd25...7cb14380 |
| `rfc6229_key_40_offset_240` | RFC 6229, section 2, key length 40 bits, rows at offsets 240 and 256 | key 0102030405, keystream bytes 240 through 271, 28cb1132...7f8d8c93 |
| `botan_rc4_vec_encrypt_ones` | Botan `src/tests/data/stream/rc4.vec`, section `[RC4]`, fifth case (Key = 0123456789ABCDEF, In = 0101...) | the first 32 bytes of the message of 0x01 bytes encrypted under the 64-bit key of the 1994 posting, 7595c3e6...778dcad8 |

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

This entry is a reference evaluation of RC4 as the 1994 posting and RFC
6229 describe it, under `orangec eval`. It makes no constant-time,
side-channel, performance or certification claim, and it is not a corpus
entry in the sense of The Orange Book chapter 12. It is not a
recommendation to use RC4 for anything.

## Gaps

None that prevented a planned vector. The language limits met, and what
they cost:

- Indices are static, so every access to S at a data-chosen index is a
  32-word walk plus an eight-arm byte selection (`get_byte`, 291 steps;
  `set_byte`, 344 steps) instead of one array read or write. This is the
  form in which Orange expresses the cipher's defining operation, and it
  puts the cost of a key schedule at about 225,000 steps and of a keystream
  byte at about 1,700, which is what forced the vectors into two files.
- A loop step may begin with `let` bindings, and a loop may carry more than
  one accumulator as a tuple. This source still splits the swap across two
  steps of an inner loop, with j and the byte in flight carried in spare
  words of the state array, and packs the keystream bytes into that array
  to unpack afterwards.
- Loop bounds and index moduli are literals, so the KSA is written once per
  key length and the PRGA once per range of output positions
  (`keystream` for bytes 0 through 31, `discard_240` and `keystream_240`
  for the second file).
- The step budget of one file, 1,048,576 steps, holds three key schedules
  with 32 bytes of keystream each; the RFC's rows at offsets 496 through
  4096 were not attempted, as described above.
