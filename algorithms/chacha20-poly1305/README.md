# ChaCha20-Poly1305 and XChaCha20-Poly1305

AEAD_CHACHA20_POLY1305 is the authenticated encryption with associated data
that Adam Langley and Yoav Nir composed from two primitives of Daniel J.
Bernstein, the ChaCha20 stream cipher (2008) and the Poly1305 one-time
authenticator (2005), first for TLS in 2013 and then for the IRTF's Crypto
Forum Research Group as RFC 7539 (May 2015), revised as
[RFC 8439, ChaCha20 and Poly1305 for IETF Protocols](https://www.rfc-editor.org/rfc/rfc8439)
(June 2018). ChaCha20 under the key and a 96-bit nonce produces, from block
0, a one-time Poly1305 key, and from block 1 on, the keystream; Poly1305
under that key authenticates the additional data, the ciphertext and both
lengths in a 16-byte tag. XChaCha20-Poly1305 is the same construction with
a 192-bit nonce, the first 128 bits of which pass through HChaCha20 to make
a subkey, described in
[draft-irtf-cfrg-xchacha](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha)
(Arciszewski, versions 00 to 03, April 2019 to January 2020).
ChaCha20-Poly1305 is a cipher suite of TLS 1.2 and 1.3, an OpenSSH transport
cipher, an IPsec ESP algorithm, a QUIC packet protection cipher, a Noise
cipher and the only cipher of WireGuard; RFC 8439 is a current IETF
specification, and XChaCha20-Poly1305 is an expired draft that libsodium,
Botan, Go's x/crypto and other libraries implement as written.

## Analysis

### Structure

The file follows RFC 8439 in the order of its section 2, then the draft.

ChaCha20 (RFC 8439 sections 2.1 to 2.4) is the ARX cipher of the
[chacha20 entry](../chacha20/README.md), carried again here because a
module can be read only from its own folder: the quarter round with its
rotations 16, 12, 8 and 7, the double round of four column and four
diagonal quarter rounds (`inner_block`), the 4 by 4 state of the constants
"expand 32-byte k", the 256-bit key, a 32-bit block counter and the 96-bit
nonce, twenty rounds followed by the addition of the initial state, and
serialization into 64 little-endian bytes. Encryption XORs the keystream of
blocks `counter`, `counter + 1`, ... with the message.

Poly1305 (section 2.5) is a polynomial evaluation modulo the prime
`p = 2^130 - 5`. The 32-byte one-time key is two little-endian numbers: `r`,
the point of evaluation, clamped so that bytes 3, 7, 11 and 15 lose their
top four bits and bytes 4, 8 and 12 their bottom two (that is, `r` is ANDed
with `0ffffffc0ffffffc0ffffffc0fffffff`), and `s`, the mask. The message is
read in 16-byte blocks; each block is a little-endian number with the byte
`0x01` placed immediately above its last byte, so a whole block is its value
plus 2^128 and a final block of `k` bytes is its value plus 2^(8k). The
accumulator is updated as `a = ((a + n) * r) mod p` for each block, `s` is
added, and the low 128 bits of the sum, little-endian, are the tag. The key
generation of section 2.6 takes the first 32 bytes of ChaCha20 block 0
under the AEAD key and nonce as the one-time key, and discards the other 32.

The AEAD (section 2.8) generates the one-time key, encrypts the plaintext
from block counter 1, and computes Poly1305 over
`aad || pad16(aad) || ciphertext || pad16(ciphertext) || len(aad) || len(ciphertext)`,
where `pad16` adds up to fifteen zero bytes to a 16-byte boundary and the
two lengths are 64-bit little-endian integers, so that the MAC input is a
whole number of blocks. The output is the ciphertext followed by the tag.
Decryption recomputes the tag over the additional data and the received
ciphertext, compares it with the received tag, and releases the plaintext
only when they are equal. XChaCha20-Poly1305 (draft section 2) computes
the subkey `HChaCha20(key, nonce[0..16])` (draft section 2.2: the twenty
rounds on a state whose last four words are those 16 nonce bytes, without
the final addition, returning words 0 to 3 and 12 to 15) and runs
AEAD_CHACHA20_POLY1305 under the subkey with the 12-byte nonce
`00 00 00 00 || nonce[16..24]`.

| Standard section | Orange spec |
| --- | --- |
| RFC 8439, 2.1, the quarter round | `quarter_round` |
| 2.3, the eight quarter rounds of a double round | `inner_block` |
| 2.3, the initial state, the twenty rounds, the serialization | `chacha20_block` |
| 2.4.1, `chacha20_encrypt` | `chacha20_encrypt[n]` for `n` from 16 through 271 bytes |
| 2.5, `p = 2^130 - 5` | `type P = Mod[(1 << 130) - 5]` |
| 2.5.1, `clamp(r)` and `le_bytes_to_num` of `r` | `clamp` |
| 2.5.1, the loop body `a += n; a = (r * a) % p` | `absorb` |
| 2.5.1, `a += s` and `num_to_16_le_bytes` | `finish` |
| 2.5.1, `poly1305_mac` | `poly1305_mac[h, q, k]` for messages of `256 h + 16 q + k` bytes |
| 2.5.1, `poly1305_mac` on whole blocks | `poly1305_mac_blocks[b]` for 1 through 24 blocks |
| 2.6.1, `poly1305_key_gen` | `poly1305_key_gen` |
| 2.8.1, `x \|\| pad16(x)` and `num_to_8_le_bytes(x.length)` | `padded16[q, k]`, `length_bytes[q, k]` |
| 2.8.1, `mac_data` and the tag | `aead_tag` |
| 2.8.1, `chacha20_aead_encrypt` | `chacha20_aead_encrypt` |
| 2.8, decryption: the tag comparison and the plaintext | `chacha20_aead_decrypt`, `withheld[n]` |
| draft, 2.2, HChaCha20 | `hchacha20` |
| draft, 2, AEAD_XChaCha20_Poly1305, steps 1 and 2 (the same steps as 2.3 gives for XChaCha20) | `xchacha20_aead_encrypt` |

Byte strings are typed by their lengths, and each function over strings of
several lengths takes the length as a size parameter, within the limit of
256 instances a function may have. `chacha20_encrypt[n]` takes 16 through
271 bytes and writes the RFC's loop over blocks as one keystream of
`ceil(n / 64)` blocks, each laid in place by a slice update, exclusive-ored
with the message byte by byte. `poly1305_mac` takes a message of
`256 h + 16 q + k` bytes, `16 h + q` whole blocks and a last block of `k`
bytes with `1 <= k <= 16`: one size for every length from 1 through 375
would need 375 instances, so the length is split, `h` in 0 and 1 and `q` in
0 through 7, and the instances cover 1 through 128 and 257 through 384
bytes, which take in every message the RFC's Poly1305 vectors use (16, 34,
48, 64, 127 and 375 bytes). The loop over the blocks reads each whole
block followed by `hex"01"` as a 17-byte little-endian number of `P`, and
the last block from a copy of the message with its `0x01` already
appended, so the zeros above it add nothing; that is the RFC's
`le_bytes_to_num(msg[...] | [0x01])` for the whole and the final block.

The AEAD's tag follows the RFC's pseudocode: `mac_data` is
`padded16(aad) ++ padded16(ciphertext) ++ length_bytes(aad) ++
length_bytes(ciphertext)`, and Poly1305 runs over it. Its lengths (80,
160 and 304 bytes for the vectors) are whole blocks but not all among the
instances of `poly1305_mac`, so the AEAD calls `poly1305_mac_blocks`, the
same loop over whole blocks only. `padded16` and `length_bytes` take
`16 q + k` bytes with `k` from 1 through 15, every length up to 271 bytes
whose last block is partial (255 instances), which covers the vectors' 8
and 12 bytes of additional data and 47, 114 and 265 of ciphertext; RFC
8439's `pad16` of a string that fills its last block is empty, and no array
is empty. The AEAD functions take two strings of independent lengths, and
a size parameter for each would multiply far past 256 instances, so
`chacha20_aead_encrypt`, `chacha20_aead_decrypt`, `aead_tag` and
`xchacha20_aead_encrypt` take type parameters instead, `A` among
`Word[8]^8` and `Word[8]^12` for the additional data and `M` among
`Word[8]^47`, `Word[8]^114` and `Word[8]^265` for the message: six
instances, each checked as written out, and each call inside them picks the
instance of `chacha20_encrypt`, `padded16` or `length_bytes` that fits the
argument's length. Encryption returns the pair `(ciphertext, tag)`, as the
RFC's pseudocode does. Decryption returns the pair of the verdict and the
plaintext: the received tag is compared with the recomputed one as one
16-byte value, and the plaintext is released only when they are equal;
otherwise it is zeros of the message's length (`withheld`, a sized spec,
because a type parameter carries no length to write a fill with).

### Security status

Record as of September 2026.

**Poly1305 is a Wegman-Carter one-time authenticator.** Bernstein's
Poly1305-AES paper (FSE 2005) proves that the polynomial
`sum c_i r^(q-i+1) mod 2^130-5` over the padded blocks is almost-Delta-universal:
two distinct messages of at most `L` bytes collide, for a uniformly random
clamped `r`, with probability at most `8 * ceil(L/16) / 2^106`, and with `s`
uniform and used once, the tag reveals nothing about `r`, so the bound is
information-theoretic for one message per key and grows linearly with the
number of forgery attempts and the message length. RFC 8439 section 4 states
it as forgeries rejected with probability `1 - n/2^102` for a 16n-byte
message even after 2^64 legitimate messages, and asks for the whole 128-bit
tag; truncation is forbidden. The bound is what a 128-bit polynomial MAC can
give, not 2^-128: against a message of 2^20 blocks (16 MiB) one forgery
attempt succeeds with probability about 2^-82. When the one-time key is
`poly1305_key_gen(key, nonce)` instead of a random string, the advantage of
distinguishing the ChaCha20 block function from a random function is added.
`r = 0` is a weak key under which the tag is `s` whatever the text (appendix
A.3, test vector 2); the RFC notes that the key generation makes it as
unlikely as any other value.

**The composition.** RFC 8439 section 2.8 calls the AEAD "a novel
composition" and cites Procter's analysis (A Security Analysis of the
Composition of ChaCha20 and Poly1305, ePrint 2014/613) for its proof:
assuming the ChaCha20 block function is a pseudorandom function, the scheme
is secure against chosen-plaintext attacks and against ciphertext forgery,
with the Poly1305 bound above, for distinct nonces. The proof rests on two
features visible in the file: block 0 is used only for the one-time key and
never for keystream, so the Poly1305 key is independent of everything the
ciphertext reveals, and the MAC input ends with both lengths, so distinct
(aad, ciphertext) pairs never map to the same padded string.

**Nonce reuse breaks both halves.** Section 4 of the RFC says it plainly:
a repeated nonce repeats the keystream, so the XOR of two ciphertexts is
the XOR of the plaintexts, and repeats the one-time Poly1305 key. The second
consequence is the worse one. Two tags under the same `(r, s)` give an
attacker the difference of two known polynomials in `r` modulo the
2^128 truncation, that is a polynomial equation in `r` of degree the number
of blocks with a handful of candidate constant terms, whose roots modulo
`p` are found in seconds; with `r` and then `s` in hand, a tag for any
ciphertext under that nonce is computed directly. This is the Poly1305
analogue of Joux's forbidden attack on GCM, and the reason section 2.6
requires the nonce to be unique per key and says it MUST NOT be random.
Ninety-six bits are too few for random nonces under a long-lived key (a
collision at probability 2^-32 after about 2^32 messages); the draft's
section 2.1 gives the arithmetic for XChaCha20-Poly1305's 192-bit nonce,
2^80 random nonces per key at the same threshold, and notes that a repeat
of only the first 128 or only the last 64 bits of the nonce changes the
subkey or the ChaCha20 nonce and is therefore harmless. The construction
offers no misuse resistance beyond that; for a repeated nonce, the
misuse-resistant designs (AES-GCM-SIV, RFC 8452) are the alternative.

**The tag does not commit to the key.** Poly1305's tag is a linear
function of the ciphertext blocks for fixed `r`, plus a free mask `s`, so
one can construct a ciphertext and tag that decrypt validly under two
different keys (two different `(r, s)` pairs) by solving for one block.
Grubbs, Lu and Ristenpart (Message Franking via Committing Authenticated
Encryption, Crypto 2017) named the property and showed ChaCha20-Poly1305
and AES-GCM lack it; Len, Grubbs and Ristenpart (Partitioning Oracle
Attacks, USENIX Security 2021) extended the construction to ciphertexts
valid under thousands of keys at once, taking the Poly1305 carries into
account, and used them to recover passwords from Shadowsocks servers,
which derive ChaCha20-Poly1305 keys from passwords, by observing which
ciphertexts a server accepts; Albertini, Duong, Gueron, Kolbl, Luykx and
Schmieg (How to Abuse and Fix Authenticated Encryption Without Key
Commitment, USENIX Security 2022) catalogued the abuses across GCM,
GCM-SIV, ChaCha20-Poly1305 and OCB3 and the fixes, a padding block that
decryption checks or a commitment string derived from the key. The CFRG's
work on the properties of AEAD algorithms and on committing AEAD followed
from these papers; RFC 8439 itself makes no commitment claim, and a
protocol that lets an attacker try a ciphertext against many keys needs
one of the fixes.

**Data limits.** One (key, nonce) pair encrypts at most 2^32 - 1 blocks,
nearly 256 GB, before the 32-bit block counter wraps (section 2.8, note 1);
the RFC leaves the wrapped counter undefined and points to the original
64-bit counter for larger messages. The confidentiality bound is that of
ChaCha20 as a pseudorandom function and does not degrade with the volume
of data the way the AES-GCM birthday term does, which is why TLS 1.3 (RFC
8446 section 5.5) sets a record limit for AES-GCM and none for
ChaCha20-Poly1305, whose sequence number wraps before any limit is reached;
the integrity bound, per the paragraph above, grows with forgery attempts
times message length, and the CFRG's AEAD usage-limits draft derives its
ChaCha20-Poly1305 limits from it.

**Implementation record.** ChaCha20 has no data-dependent memory access
or branch. Poly1305's arithmetic is 130-bit and is implemented with 26- or
44-bit limbs and a lazy reduction (poly1305-donna, the NaCl reference,
OpenSSL, BoringSSL, Go), where carries are the classic bug: RFC 8439
appendix A.3 test vectors 5 to 11 exist for that, each headed by the
question it asks ("What happens if addition of s overflows modulo 2^128?",
"... if 5*H+L-type reduction produces 131-bit intermediate result?"), and
Botan's `poly1305.vec` carries further long inputs "chosen as more likely
to trigger carry bugs". Section 4 warns that a generic big-number library
is not constant time (a product is sometimes above 2^256 and sometimes
not) and section 3 recommends against one; the tag comparison must be
constant time, since a byte-by-byte early exit lets an attacker find the
tag one byte at a time. OpenSSH's `chacha20-poly1305@openssh.com` (2013)
is a different composition, two ChaCha20 instances with the packet length
encrypted separately, and the Terrapin attack on SSH (Baumer, Brinkmann
and Schwenk, 2023; USENIX Security 2024) truncated the handshake under
that cipher and under CBC with encrypt-then-MAC through a flaw of the SSH
transport, not of the AEAD; the fix is the strict key exchange extension.

**Status.** RFC 8439 (IRTF CFRG, Informational, June 2018, obsoleting RFC
7539) is the current definition. ChaCha20-Poly1305 is
`TLS_CHACHA20_POLY1305_SHA256` in TLS 1.3 (RFC 8446 section 9.1: a suite
every implementation SHOULD support, beside the mandatory AES-128-GCM), the
TLS 1.2 suites of RFC 7905, an ESP and IKEv2 algorithm (RFC 7634), a QUIC
packet protection and header protection cipher (RFC 9001), a Noise cipher,
and WireGuard's only symmetric cipher, which also uses XChaCha20-Poly1305
for its cookie replies. XChaCha20-Poly1305 has no RFC:
draft-irtf-cfrg-xchacha-03 is dated January 10, 2020 and expired on July
13, 2020, and the construction is a de facto standard through libsodium
(`crypto_aead_xchacha20poly1305_ietf`), Botan, Go's `x/crypto`, Tink,
LibreSSL and others, whose vectors and the draft's agree. No attack on
either construction as specified, under distinct nonces and full tags, is
known as of 2026.

### What the Orange rendering shows

Nothing in ChaCha20 depends on data: the rotation amounts are literals,
every index of `inner_block` is a constant, and the loops run 10 and 16
times. The byte orders are written where the RFC fixes them, with
`as little`: the state is `"expand 32-byte k" ++ key ++ counter ++ nonce`
read as sixteen little-endian words, the counter becomes its four bytes the
same way, and the sum of the rounds and the state is serialized back to 64
bytes in one conversion; HChaCha20 reads its sixteen nonce bytes in place
of the counter and the nonce, and writes its eight output words with the
same conversion. The one-time key is `chacha20_block(key, 0, nonce)[..32]`
and the XChaCha20 nonce `hex"00000000" ++ nonce[16..]`, slices and joins as
the standards write them.

In Poly1305 the arithmetic is exact: `P` is `Mod[(1 << 130) - 5]`, and
`absorb` is the RFC's `a += n; a = (r * a) % p` on residues, with no limbs,
no partial reduction and no carries. `clamp` applies the RFC's mask to the
two little-endian 64-bit halves of `r` and reads them as a residue; `finish`
adds `s` as an `Int` and keeps the low 128 bits by converting the sum to
sixteen little-endian bytes, which is the RFC's truncation. Appendix A.3
test vectors 5 to 11 therefore test something different here from what
they test in a limb implementation: not carry propagation but the reading
of the specification itself, the `0x01` above the last byte present, the
reduction modulo `2^130 - 5` rather than `2^130`, and the truncation of
`a + s` to 128 bits. Vector 6 (the sum passes 2^128), vector 8 (the
polynomial part is exactly `p`, so the accumulator is 0) and vectors 10 and
11 (a 130-bit `r` product) are the ones that would expose a wrong reading.

The conditionals are on lengths and on the verdict. In `poly1305_mac` the
loop index is compared with the number of whole blocks, a size, to choose
between a whole block and the last one; `chacha20_aead_decrypt` selects the
plaintext or zeros on the tag verdict. The tag comparison is one `==` on
two 16-byte arrays, which Orange evaluates in full: its cost does not
depend on where the arrays differ. That is the shape section 4 asks for,
and nothing more: `orangec test` counts steps, it does not measure time,
and the entry makes no constant-time claim. The checker proves every index
and slice in range for every instance before evaluation.

Measured costs, with `orangec test --stats`. The figures for one call come
from scratch tests that make that call alone, and hold to within a few
steps; the figures for tests are the ones `--stats` prints for the tests
of the table below. One call of `quarter_round` is about 41 steps,
`inner_block` about 412, `chacha20_block` about 4,320 and `hchacha20`
about 4,160; each one-time key test costs 4,333. One call of
`chacha20_encrypt` costs about 10,050 steps on 114 bytes (two blocks) and
25,690 on 265 (five blocks), the byte-by-byte exclusive-or being 12 to 15
steps a byte. One `absorb` is about 90 steps, a residue product of five
32-bit digits being 51 of them, so a Poly1305 block costs 110 to 120 steps
with its slice and `0x01`: one call of `poly1305_mac` over 64 bytes is
about 555 steps and over the 375-byte text about 2,920, and the 2.8.2 tag
over ten blocks of `mac_data` about 1,180. These depend a little on the
data (an all-zero key and message cost less). One sealing of the 114-byte
text is about 15,560 steps, three ChaCha20 blocks and the tag. As whole
tests, the seal of section 2.8.2 costs 15,598 steps and its opening
15,608, the XChaCha20 seal of the draft's A.1 adds `hchacha20` for 19,766,
and the opening of appendix A.5, six ChaCha20 blocks and a tag over
nineteen blocks, costs 32,271. The 22 tests use 135,961 steps in all.

Not expressed: a message of any length in one function (each function
takes the lengths its comment names, and the AEAD the lengths of its
vectors), empty additional data or an empty plaintext, which RFC 8439
allows, and the 64-bit block counter of the original ChaCha20 that section
2.8 mentions for messages over 256 GB.

## Dissemination

### Files

- `chacha20-poly1305.or`: module `chacha20_poly1305`. ChaCha20 (quarter
  round, double round, block function, encryption), Poly1305 (clamping, the
  accumulator, the tag, for messages of the lengths above and for whole
  blocks), the one-time key generation, the AEAD's padding, lengths, tag,
  sealing and opening, HChaCha20, AEAD_XChaCha20_Poly1305, and the 22 tests
  below.

### Running

    orangec test algorithms/chacha20-poly1305/chacha20-poly1305.or
    python3 algorithms/verify.py algorithms/chacha20-poly1305

`orangec eval` prints the parameterless specs that hold inputs shared by
several tests: `ietf_text` (the 375-byte text of A.3), `key_a5`,
`key_2_8_2`, `nonce_2_8_2`, `aad_2_8_2`, `sunscreen` (the 114-byte
plaintext of 2.8.2) and `sealed_2_8_2` (its published ciphertext and tag).

### Vectors

Each row is a `test` block in `chacha20-poly1305.or`.

| Test | Source | Case |
| --- | --- | --- |
| `RFC 8439 2.5.2: Poly1305 tag` | RFC 8439, section 2.5.2 | Poly1305 of the 34-byte text "Cryptographic Forum Research Group" under the key 85:d6:be:78:...:f5:1b; the tag a8:06:1d:c1:...:27:a9 |
| `RFC 8439 A.3 #1: Poly1305, zero key` | RFC 8439, appendix A.3, test vector 1 | zero key, 64 zero bytes; the zero tag |
| `RFC 8439 A.3 #2: Poly1305, r = 0` | RFC 8439, appendix A.3, test vector 2 | r = 0, s = 36e5f6b5..., the 375-byte IETF boilerplate text; the tag is s |
| `RFC 8439 A.3 #3: Poly1305, s = 0` | RFC 8439, appendix A.3, test vector 3 | r = 36e5f6b5... (before clamping), s = 0, the same text; the tag is the polynomial alone |
| `RFC 8439 A.3 #4: Poly1305, the Jabberwocky text` | RFC 8439, appendix A.3, test vector 4 | key 1c:92:40:a5:..., the 127-byte "Jabberwocky" text: seven whole blocks and a 15-byte final block |
| `RFC 8439 A.3 #5: Poly1305, 2^130 - 2 reduces to 3` | RFC 8439, appendix A.3, test vector 5 | r = 2, s = 0, one block of ff bytes: the partially reduced result 2^130 - 2 reduces to 3 |
| `RFC 8439 A.3 #6: Poly1305, a + s overflows 2^128` | RFC 8439, appendix A.3, test vector 6 | r = 2, s = 2^128 - 1, one block 02 00 ... 00: the addition of s overflows 2^128 |
| `RFC 8439 A.3 #7: Poly1305, the sum passes p` | RFC 8439, appendix A.3, test vector 7 | r = 1, s = 0, three blocks: a data limb of all ones with a carry from below; the tag 05 |
| `RFC 8439 A.3 #8: Poly1305, the sum is a multiple of p` | RFC 8439, appendix A.3, test vector 8 | r = 1, s = 0, three blocks whose polynomial part is exactly 2^130 - 5; the zero tag |
| `RFC 8439 A.3 #9: Poly1305, the sum is p - 1` | RFC 8439, appendix A.3, test vector 9 | r = 2, s = 0, one block fd ff ... ff: the polynomial part is exactly 2^130 - 6; the tag fa ff ... ff |
| `RFC 8439 A.3 #10: Poly1305, a 131-bit intermediate result` | RFC 8439, appendix A.3, test vector 10 | r = 2^66 + 1, s = 0, four blocks: a 5*H+L reduction with a 131-bit intermediate result; the tag 14 00 ... 55 00 ... |
| `RFC 8439 A.3 #11: Poly1305, a 131-bit final result` | RFC 8439, appendix A.3, test vector 11 | the same r over the first three blocks: a 131-bit final result; the tag 13 00 ... 00 |
| `RFC 8439 2.6.2: Poly1305 key generation` | RFC 8439, section 2.6.2 | the one-time key under the key 80:81:...:9f and the nonce 00 00 00 00 00 01 02 03 04 05 06 07: 8a d5 a0 8b ... |
| `RFC 8439 A.4 #1: Poly1305 key generation, zero key` | RFC 8439, appendix A.4, test vector 1 | zero key, zero nonce: 76 b8 e0 ad ... (ChaCha20 block 0 of the zero state) |
| `RFC 8439 A.4 #3: Poly1305 key generation` | RFC 8439, appendix A.4, test vector 3 | key 1c:92:40:a5:..., nonce 00 ... 00 02: 96 5e 3b c6 ... |
| `RFC 8439 2.8.2: AEAD seal` | RFC 8439, section 2.8.2 | AEAD_CHACHA20_POLY1305 of the 114-byte "sunscreen" plaintext, key 80:81:...:9f, nonce 07 00 00 00 40 41 ... 47, aad 50 51 52 53 c0 c1 ... c7; the 114-byte ciphertext d3 1a 8d 34 ... and the tag 1a:e1:0b:59:...:06:91 |
| `RFC 8439 2.8.2: AEAD open` | RFC 8439, section 2.8.2, decrypted | the published ciphertext and tag: the tag verifies (`true`) and the plaintext is the sunscreen text |
| `RFC 8439 2.8.2: a tag with its last bit flipped is rejected` | not a published vector: the 2.8.2 ciphertext with the last bit of its tag flipped (0x91 to 0x90); oracle: `cryptography` `ChaCha20Poly1305.decrypt` raises `InvalidTag` | the verdict `false` |
| `RFC 8439 A.5: AEAD open` | RFC 8439, appendix A.5 | the 265-byte ciphertext, key 1c:92:40:a5:..., nonce 00 00 00 00 01 02 ... 08, aad f3 33 88 86 00 00 00 00 00 00 4e 91, received tag ee:ad:9d:67:...:1f:38: the tag verifies (`true`) and the plaintext is the 265-byte Internet-Drafts boilerplate |
| `draft-irtf-cfrg-xchacha A.1: AEAD seal` | draft-irtf-cfrg-xchacha-03, appendix A.1 (repeated as continuous hex in A.3.1) | AEAD_XCHACHA20_POLY1305 of the sunscreen plaintext under the 2.8.2 key and aad and the 24-byte nonce 40 41 ... 57; the ciphertext bd 6d 17 9d ... and the tag c0:87:59:24:...:cf:49 |
| `Wycheproof chacha20_poly1305 tcId 71` | Wycheproof `testvectors_v1/chacha20_poly1305_test.json`, tcId 71 (`valid`, flag `Pseudorandom`) | a 47-byte message with 8 bytes of aad, 96-bit nonce: an independent source with a partial final ciphertext block |
| `Wycheproof xchacha20_poly1305 tcId 71` | Wycheproof `testvectors_v1/xchacha20_poly1305_test.json`, tcId 71 (`valid`, flag `Pseudorandom`) | a 47-byte message with 8 bytes of aad, 192-bit nonce |

Every expected value except the tampered verdict is copied from the named
source; the tampered case is stated so that the tag comparison is seen to
reject as well as accept. The draft's example uses the RFC's key,
plaintext and additional data with its own nonce, and appears in the draft
twice, as a hex dump in A.1 and as continuous hex in A.3.1; both were
parsed and agree.

### Provenance and claims

The standards' own sites are unreachable from the machine that wrote this
entry, so RFC 8439 and draft-irtf-cfrg-xchacha-03 were read from verbatim
text copies on raw.githubusercontent.com (`smuellerDD/leancrypto`,
`aead/doc/rfc8439.txt`, and `rust-stdx/stdx`,
`crypto/docs/draft-irtf-cfrg-xchacha-03.txt`), byte-identical to the copies
the chacha20 entry used. `rfc_text_check.py` (scratch work) parses the
labeled hex dumps of the RFC's sections 2.5.2, 2.6.2 and 2.8.2 and
appendices A.3 (all eleven vectors), A.4 (three) and A.5, and of the
draft's A.1 and A.3.1, and compares every one with the value the Orange
file states; it also checks the RFC's printed intermediate values, the
one-time keys of 2.8.2 and A.5, the 160- and 288-byte "AEAD Construction
for Poly1305" and "Poly1305 Input" buffers, and the decryption of A.5,
against the Python reference. The same vectors were taken independently
from BoringSSL's `crypto/poly1305/poly1305_tests.txt` (its first twelve
cases are 2.5.2 and A.3 in order), Botan's `src/tests/data/mac/poly1305.vec`
and `src/tests/data/aead/chacha20poly1305.vec` (the cases it labels "From
RFC 7539", "From draft-irtf-cfrg-chacha20-poly1305-03", the A.5 case, and
"XChaCha20Poly1305 from draft-irtf-cfrg-xchacha-00") and Wycheproof's
`testvectors_v1/chacha20_poly1305_test.json` and
`xchacha20_poly1305_test.json` (tcId 1 of each is the RFC's and the
draft's example; tcId 71 the cases above), and `vectors.py` asserted each
against a Python reference of RFC 8439 and the draft written for this
entry (`reference.py`, agreeing with the `cryptography` package's
`ChaCha20Poly1305` and `Poly1305` and pycryptodome's `ChaCha20`,
`ChaCha20_Poly1305` with 12- and 24-byte nonces and `_HChaCha20` on 200
random trials), against `cryptography`'s `Poly1305` and `ChaCha20Poly1305`,
pycryptodome's `ChaCha20` (block 0 for the one-time keys),
`ChaCha20_Poly1305` and `Crypto.Hash.Poly1305.Poly1305_MAC` on the raw
`r || s` keys. The Orange literals were emitted from those bytes by
`emit_vectors.py`, never typed by hand, and `recheck.py` re-parsed the
committed file's `eval` output and its inline input literals against them.
In that first form the four ChaCha20 constants were "expand 32-byte k" by
`struct.pack`, `prime` was `2^130 - 5`, the clamping mask was the RFC's
`0x0ffffffc0ffffffc0ffffffc0fffffff` written as sixteen bytes, and
`byte_weights` was `256^i`, each asserted in the script.

The entry was then rewritten in the current language. Every expected value
is carried over byte for byte from the first form, where each was a
`<name>_expected` spec: the generator of the new literals read the bytes
from the first form's `eval` output and source, and the texts now written
as string literals (the sunscreen plaintext, the A.3 texts and the A.5
plaintext, whose curly quotation marks are `\xe2\x80\x9c` and
`\xe2\x80\x9d`) were compared byte for byte with the old values by
evaluating both. The 2.8.2 and A.5 openings, which were a verdict and a
plaintext (in two parts for A.5) in the first form, are each one test of
the pair `(true, plaintext)`; the A.5 ciphertext and plaintext are whole.
No vector was added or dropped. The constants are now the string
"expand 32-byte k" read with `as little`, the modulus `(1 << 130) - 5` of
`P`, and the clamping mask as the two 64-bit words `0x0ffffffc0fffffff` and
`0x0ffffffc0ffffffc`, the RFC's mask read in halves.

This entry is a reference evaluation of RFC 8439 and
draft-irtf-cfrg-xchacha-03 under `orangec test`: it shows that the Orange
text computes the standards' values on the cases listed. It makes no
constant-time, side-channel, performance or certification claim; the
remarks on constant time above are about the specification's requirements,
not about this evaluator, whose arithmetic modulo `p` is exactly the
generic big-number form section 4 warns against. It is not an implementation
anyone should deploy, and it is not a corpus entry in the sense of The
Orange Book chapter 12.

## Gaps

- A function has at most 256 instances, and every length is part of a
  type, so no function here takes a byte string of any length:
  `chacha20_encrypt` and `withheld` take 16 through 271 bytes,
  `poly1305_mac` 1 through 128 and 257 through 384 (the length split as
  `256 h + 16 q + k` to reach the 375-byte text of A.3),
  `poly1305_mac_blocks` 1 through 24 whole blocks, and `padded16` and
  `length_bytes` the lengths up to 271 bytes that are not multiples of 16.
  Another length needs a range widened, within that limit.
- The AEAD functions take two strings of independent lengths, and a size
  parameter for each would exceed the limit, so they list the vectors'
  lengths as types (8 and 12 bytes of additional data; 47, 114 and 265 of
  message). A type parameter carries no length, so the lengths of
  `mac_data` come from sized helpers called by fit, and the zeros of a
  withheld plaintext from `withheld`.
- The AEAD's `mac_data` (80, 160 and 304 bytes for the vectors) does not
  fall entirely among the instances of `poly1305_mac`, so the tag uses
  `poly1305_mac_blocks`, the same loop over whole blocks, rather than the
  RFC's single `poly1305_mac`.
- No array has zero elements: RFC 8439's `pad16(x)` is empty when `x`
  fills its last block, so `padded16` writes `x || pad16(x)` and takes only
  lengths whose last block is partial, and empty additional data or an
  empty plaintext cannot be passed.
- The RFC's decryption fails without output; here it returns the verdict
  with zeros in place of the plaintext.
