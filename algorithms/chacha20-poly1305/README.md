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
(Arciszewski, versions 00 to 03, 2018 to January 2020). ChaCha20-Poly1305
is a cipher suite of TLS 1.2 and 1.3, an OpenSSH transport cipher, an IPsec
ESP algorithm, a QUIC packet protection cipher, a Noise cipher and the only
cipher of WireGuard; RFC 8439 is a current IETF specification, and
XChaCha20-Poly1305 is an expired draft that libsodium, Botan, Go's x/crypto
and other libraries implement as written.

## Analysis

### Structure

The file follows RFC 8439 in the order of its section 2, then the draft.

ChaCha20 (RFC 8439 sections 2.1 to 2.4) is the ARX cipher of the
[chacha20 entry](../chacha20/README.md), carried again here because an
Orange file has no imports: the quarter round with its rotations 16, 12, 8
and 7, the double round of four column and four diagonal quarter rounds
(`inner_block`), the 4 by 4 state of the constants "expand 32-byte k", the
256-bit key, a 32-bit block counter and the 96-bit nonce, twenty rounds
followed by the addition of the initial state, and serialization into 64
little-endian bytes. Encryption XORs the keystream of blocks `counter`,
`counter + 1`, ... with the message.

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
| 2.3, bytes and words | `load_le32`, `le_bytes` |
| 2.3, the initial state, its serialization, `chacha20_block` | `initial_state`, `serialize`, `chacha20_block` |
| 2.4.1, `chacha20_encrypt` for 114, 47, 256 and 9 bytes | `chacha20_encrypt`, `chacha20_encrypt_47`, `chacha20_encrypt_256`, `chacha20_encrypt_9` |
| 2.5, `p = 2^130 - 5` and the clamping of `r` | `prime`, `clamp` |
| 2.5.1, `le_bytes_to_num`, `r` and `s` | `le_bytes_to_num`, `poly1305_r`, `poly1305_s` |
| 2.5.1, `a = ((a + n) * r) mod p` | `absorb` |
| 2.5.1, the number of a block with its `0x01` byte, the loop over the blocks | `block_byte`, `poly1305_blocks` |
| 2.5.1, `num_to_16_le_bytes` | `byte_weights`, `num_to_16_le_bytes` |
| 2.5.1, `poly1305_mac` for up to 256 bytes, and for 257 to 512 | `poly1305_mac`, `poly1305_mac_long` |
| 2.6.1, `poly1305_key_gen` | `poly1305_key_gen` |
| 2.8, the MAC input, block by block | `whole_block`, `absorb_aad_12`, `absorb_aad_8`, `absorb_ciphertext_114`, `absorb_ciphertext_47`, `absorb_ciphertext_265`, `num_to_8_le_bytes`, `absorb_lengths` |
| 2.8, the tag | `aead_tag`, `aead_tag_8_47`, `aead_tag_12_265` |
| 2.8.1, `chacha20_aead_encrypt` | `chacha20_aead_encrypt`, `chacha20_aead_encrypt_8_47` |
| 2.8, decryption: the tag comparison and the plaintext | `chacha20_aead_verify`, `chacha20_aead_decrypt`, `chacha20_aead_verify_265`, `chacha20_aead_decrypt_265_head`, `chacha20_aead_decrypt_265_tail` |
| draft, 2.2, HChaCha20 | `hchacha20` |
| draft, 2, steps 1 and 2: the nonce halves | `hchacha20_nonce`, `chacha20_nonce` |
| draft, 2, AEAD_XChaCha20_Poly1305 | `xchacha20_aead_encrypt`, `xchacha20_aead_encrypt_8_47` |

Poly1305 is written over `Int`, as the S3f fixture `valid-aead.or` writes
it, but for a message of any length: `poly1305_mac` takes a 256-byte buffer
and the message length as an `Int`, and `poly1305_blocks` loops over the
sixteen possible blocks, absorbing block `j` when `16j` is inside the
message. Each block's number is folded from byte 15 down to byte 0 by
`block_byte`, which contributes the byte when its position is inside the
message, the `0x01` when its position equals the length, and nothing
beyond; a whole block starts the fold from 1 so that the `0x01` lands above
byte 15, a partial one from 0. That is the RFC's `le_bytes_to_num(msg[...] | [0x01])`
for both the whole and the final block in one expression. The 375-byte text
of appendix A.3 exceeds an array, so `poly1305_mac_long` absorbs a 256-byte
head and a second segment starting at byte 256 with the same loop.

The AEAD's MAC input is never laid out as one array. Every block of it is
whole, so `aead_tag` absorbs it in parts with the same `absorb`: the padded
additional data (one block for 12 or 8 bytes), the ciphertext with its last
block zero-padded, and the block of the two lengths; the accumulator
carries between the parts exactly as it would over the concatenation, and
the comment on `whole_block` says so. Message sizes are types, so the
ciphertext parts and the tag exist once per size the vectors use: 12 bytes
of additional data with 114 (section 2.8.2 and the draft's example) and 265
bytes (appendix A.5), 8 with 47 (the two Wycheproof cases). Decryption is
two specs, because a spec has one result: `chacha20_aead_verify` returns the
verdict of the tag comparison as a `Bool`, and `chacha20_aead_decrypt`
returns the plaintext when the verdict is `true` and 114 zero bytes when it
is not. The 265-byte ciphertext of appendix A.5 is passed as a 256-byte
head and a 9-byte tail with the tag apart, and its plaintext comes back in
the same two parts, the tail decrypted under block counter 5, the counter
the RFC's loop reaches after the four blocks of the head.

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
times. In Poly1305 the arithmetic is exact: `absorb` is the RFC's
`Acc = ((Acc+Block)*r) % P` on `Int`, one 5-limb product and one reduction,
about 93 steps, and there are no limbs, no partial reduction and no
carries. Appendix A.3 test vectors 5 to 11 therefore test something
different here from what they test in a limb implementation: not carry
propagation but the reading of the specification itself, the `0x01` above
the last byte present, the reduction modulo `2^130 - 5` rather than
`2^130`, and the truncation of `a + s` to 128 bits, which `num_to_16_le_bytes`
performs by dividing by 256^i and converting to `Word[8]`. Vector 6 (the
sum passes 2^128), vector 8 (the polynomial part is exactly `p`, so the
accumulator is 0) and vectors 10 and 11 (a 130-bit `r` product) are the ones
that would expose a wrong reading.

The conditionals are on lengths and on the verdict. `block_byte` compares
each byte position with the message length, `poly1305_blocks` compares each
block's offset with it, and `chacha20_aead_decrypt` selects the plaintext or
zeros on the tag verdict; the message length is public, and the tag
comparison is a fold of `&&` over sixteen byte equalities, which Orange
evaluates in full (both sides of `&&` are always evaluated), so the spec
compares all sixteen bytes whatever the first says. That is the shape
section 4 asks for, and nothing more: `orangec eval` counts steps, it does
not measure time, and the entry makes no constant-time claim. `Int` has no
bitwise operators, so `clamp` is written on the sixteen bytes of `r` before
they become a number, with the RFC's mask as bytes.

Costs, measured with filler specs sharing a file's budget (`micro2.py`,
`measure.py` and `headroom.py` in the entry's scratch work, not in the
repository): a `quarter_round` is about 50 steps, an `inner_block` 434, and
a `chacha20_block` about 5,900, half the chacha20 entry's figure because
`serialize` builds the 64 bytes as one literal (about 500 steps) instead of
64 single-byte updates; `hchacha20` is about 7,300 and `poly1305_key_gen`
7,100. One Poly1305 block costs about 850 steps, of which the sixteen
`block_byte` calls are 580 and the `absorb` 93, so `poly1305_blocks` over
256 bytes is about 13,600 and `poly1305_mac` over 256 bytes about 15,900
with `r`, `s` and the tag bytes. What dominates the Poly1305 vectors is not
the arithmetic but the copy of the message into the 256-byte buffer: an
update of an n-element array costs n steps, so each message byte costs 256
steps to place, and test vector 1 (64 zero bytes) costs 22,800 of which
16,400 are the copy. Likewise `chacha20_encrypt_256` is about 87,400 steps,
of which the four blocks are 23,400 and the 256 updates of the 256-byte
ciphertext 65,500. A 114-byte seal (`chacha20_aead_encrypt`) is about
58,300 steps: three ChaCha20 blocks, the eleven `absorb`s of the tag, and
the placement of the 114 ciphertext bytes; the XChaCha20 seal adds
`hchacha20` for 65,500; the tag check on 130 bytes is 30,000 and the
decryption 43,700, the check being recomputed inside it. The vector specs
measure 14,400 (section 2.5.2), 7,200 (each one-time key), 7,700 to 55,200
(the A.3 vectors, the two over the 375-byte text the largest), 61,700 (the
2.8.2 seal), 31,800 and 74,900 (its tag check and its opening), 25,600,
131,100 and 31,800 (the A.5 check, head and tail), 69,900 (the draft's
example) and 26,200 and 33,800 (the Wycheproof cases): 832,000 steps in
all by that method, and 806,000 by measuring how large a filler the
committed file still admits, which leaves about 240,000 steps, room for
four more seals of the 2.8.2 size. No vector was moved to a second file or
dropped, and appendix A.5 is reproduced whole.

Not expressed: a message of arbitrary length in one spec (the Poly1305 MAC
takes any length up to 256 bytes through its buffer and length, and up to
512 in two segments; the AEAD's ciphertext parts and encryption exist per
size), the RFC's single `chacha20_aead_decrypt` returning either the
plaintext or a failure (two specs here), and the 64-bit block counter of
the original ChaCha20 that section 2.8 mentions for messages over 256 GB.

## Dissemination

### Files

- `chacha20-poly1305.or`: module `chacha20_poly1305`. ChaCha20 (quarter
  round, double round, block function, serialization, encryption for the
  four message sizes), Poly1305 (clamping, the block fold, the accumulator,
  the tag, for up to 256 and up to 512 bytes), the one-time key generation,
  the AEAD's tag, sealing, tag check and opening, HChaCha20,
  AEAD_XChaCha20_Poly1305, and every vector below.

### Running

    orangec eval algorithms/chacha20-poly1305/chacha20-poly1305.or
    python3 algorithms/verify.py algorithms/chacha20-poly1305

`eval` also prints `prime`, `byte_weights` and the input specs
`rfc8439_2_8_2_key`, `rfc8439_2_8_2_nonce`, `rfc8439_2_8_2_aad`,
`sunscreen`, `ietf_text_head`, `ietf_text_tail`, `rfc8439_a5_key`,
`rfc8439_a5_nonce`, `rfc8439_a5_aad`, `rfc8439_a5_ciphertext_head`,
`rfc8439_a5_ciphertext_tail` and `rfc8439_a5_tag`, which have no
`_expected` twin and are not vectors.

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `rfc8439_2_5_2` | RFC 8439, section 2.5.2 | Poly1305 of the 34-byte text "Cryptographic Forum Research Group" under the key 85:d6:be:78:...:f5:1b; the tag a8:06:1d:c1:...:27:a9 |
| `rfc8439_a3_1` | RFC 8439, appendix A.3, test vector 1 | zero key, 64 zero bytes; the zero tag |
| `rfc8439_a3_2` | RFC 8439, appendix A.3, test vector 2 | r = 0, s = 36e5f6b5..., the 375-byte IETF boilerplate text; the tag is s |
| `rfc8439_a3_3` | RFC 8439, appendix A.3, test vector 3 | r = 36e5f6b5... (before clamping), s = 0, the same text; the tag is the polynomial alone |
| `rfc8439_a3_4` | RFC 8439, appendix A.3, test vector 4 | key 1c:92:40:a5:..., the 127-byte "Jabberwocky" text: seven whole blocks and a 15-byte final block |
| `rfc8439_a3_5` | RFC 8439, appendix A.3, test vector 5 | r = 2, s = 0, one block of ff bytes: the partially reduced result 2^130 - 2 reduces to 3 |
| `rfc8439_a3_6` | RFC 8439, appendix A.3, test vector 6 | r = 2, s = 2^128 - 1, one block 02 00 ... 00: the addition of s overflows 2^128 |
| `rfc8439_a3_7` | RFC 8439, appendix A.3, test vector 7 | r = 1, s = 0, three blocks: a data limb of all ones with a carry from below; the tag 05 |
| `rfc8439_a3_8` | RFC 8439, appendix A.3, test vector 8 | r = 1, s = 0, three blocks whose polynomial part is exactly 2^130 - 5; the zero tag |
| `rfc8439_a3_9` | RFC 8439, appendix A.3, test vector 9 | r = 2, s = 0, one block fd ff ... ff: the polynomial part is exactly 2^130 - 6; the tag fa ff ... ff |
| `rfc8439_a3_10` | RFC 8439, appendix A.3, test vector 10 | r = 2^66 + 1, s = 0, four blocks: a 5*H+L reduction with a 131-bit intermediate result; the tag 14 00 ... 55 00 ... |
| `rfc8439_a3_11` | RFC 8439, appendix A.3, test vector 11 | the same r over the first three blocks: a 131-bit final result; the tag 13 00 ... 00 |
| `rfc8439_2_6_2` | RFC 8439, section 2.6.2 | the one-time key under the key 80:81:...:9f and the nonce 00 00 00 00 00 01 02 03 04 05 06 07: 8a d5 a0 8b ... |
| `rfc8439_a4_1` | RFC 8439, appendix A.4, test vector 1 | zero key, zero nonce: 76 b8 e0 ad ... (ChaCha20 block 0 of the zero state) |
| `rfc8439_a4_3` | RFC 8439, appendix A.4, test vector 3 | key 1c:92:40:a5:..., nonce 00 ... 00 02: 96 5e 3b c6 ... |
| `rfc8439_2_8_2` | RFC 8439, section 2.8.2 | AEAD_CHACHA20_POLY1305 of the 114-byte "sunscreen" plaintext, key 80:81:...:9f, nonce 07 00 00 00 40 41 ... 47, aad 50 51 52 53 c0 c1 ... c7; the 114-byte ciphertext d3 1a 8d 34 ... followed by the tag 1a:e1:0b:59:...:06:91 |
| `rfc8439_2_8_2_verify` | RFC 8439, section 2.8.2, decrypted | the tag of the published ciphertext recomputed and compared: `true` |
| `rfc8439_2_8_2_open` | RFC 8439, section 2.8.2, decrypted | the published ciphertext opened: the sunscreen plaintext |
| `rfc8439_2_8_2_tampered_verify` | not a published vector: the 2.8.2 ciphertext with the last bit of its tag flipped (0x91 to 0x90); oracle: `cryptography` `ChaCha20Poly1305.decrypt` raises `InvalidTag` | the verdict `false` |
| `rfc8439_a5_verify` | RFC 8439, appendix A.5 | the 265-byte ciphertext, key 1c:92:40:a5:..., nonce 00 00 00 00 01 02 ... 08, aad f3 33 88 86 00 00 00 00 00 00 4e 91, received tag ee:ad:9d:67:...:1f:38: the tag verifies, `true` |
| `rfc8439_a5_head` | RFC 8439, appendix A.5 | bytes 0 to 255 of its plaintext, the Internet-Drafts boilerplate, released by the decryption |
| `rfc8439_a5_tail` | RFC 8439, appendix A.5 | bytes 256 to 264 of the same plaintext, under block counter 5 |
| `xchacha_draft_a1` | draft-irtf-cfrg-xchacha-03, appendix A.1 (repeated as continuous hex in A.3.1) | AEAD_XCHACHA20_POLY1305 of the sunscreen plaintext under the 2.8.2 key and aad and the 24-byte nonce 40 41 ... 57; the ciphertext bd 6d 17 9d ... and the tag c0:87:59:24:...:cf:49 |
| `wycheproof_chacha20_poly1305_tc_71` | Wycheproof `testvectors_v1/chacha20_poly1305_test.json`, tcId 71 (`valid`, flag `Pseudorandom`) | a 47-byte message with 8 bytes of aad, 96-bit nonce: an independent source with a partial final ciphertext block |
| `wycheproof_xchacha20_poly1305_tc_71` | Wycheproof `testvectors_v1/xchacha20_poly1305_test.json`, tcId 71 (`valid`, flag `Pseudorandom`) | a 47-byte message with 8 bytes of aad, 192-bit nonce |

Every expected value except the tampered verdict is copied from the named
source; the tampered case is stated so that the tag comparison is seen to
reject as well as accept. The A.5 ciphertext and plaintext are longer than
an array, so each is two specs whose concatenation is the RFC's value. The
draft's example uses the RFC's key, plaintext and additional data with its
own nonce, and appears in the draft twice, as a hex dump in A.1 and as
continuous hex in A.3.1; both were parsed and agree.

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
The four ChaCha20 constants are "expand 32-byte k" by `struct.pack`,
`prime` is `2^130 - 5`, the clamping mask is the RFC's
`0x0ffffffc0ffffffc0ffffffc0fffffff` written as sixteen bytes, and
`byte_weights` is `256^i`, each asserted in the script.

This entry is a reference evaluation of RFC 8439 and
draft-irtf-cfrg-xchacha-03 under `orangec eval`: it shows that the Orange
text computes the standards' values on the cases listed. It makes no
constant-time, side-channel, performance or certification claim; the
remarks on constant time above are about the specification's requirements,
not about this evaluator, whose `Int` arithmetic is exactly the generic
big-number form section 4 warns against. It is not an implementation
anyone should deploy, and it is not a corpus entry in the sense of The
Orange Book chapter 12.

## Gaps

- The 375-byte text of A.3 vectors 2 and 3 is authenticated in two segments
  (`poly1305_mac_long`), and the 265-byte ciphertext and plaintext of A.5
  are a 256-byte head and a 9-byte tail, the tail's block counter (5) set
  by hand. An array holds 1 through 65,536 elements, so those lengths fit
  in one array; the sources keep the split written when the bound was 256,
  and the RFC's single call over the whole message is not one spec.
- No length polymorphism: the message length of the Poly1305 MAC is a
  value beside a fixed buffer, but the AEAD's encryption, ciphertext
  absorption, tag and seal are written once per size the vectors need
  (114, 47, 265 bytes of ciphertext; 12 and 8 of additional data), eleven
  specs that differ only in their bounds.
- Static indices only: the final partial block cannot be sliced at the
  message length, so `block_byte` decides per byte position with two
  comparisons, 580 of the 850 steps of a Poly1305 block.
- One result per spec: decryption is a `Bool` spec and a plaintext spec,
  and the plaintext spec recomputes the tag check, about 30,000 steps
  twice for the 2.8.2 opening.
- An update of an n-element array costs n steps: placing a message in the
  256-byte Poly1305 buffer costs 256 steps per byte, and placing the
  keystream in a 256-byte ciphertext costs three times the block
  functions; the file still uses about 810,000 of its 1,048,576 steps and
  nothing was split or dropped.
- `Int` has no bitwise operators, so `clamp` masks the bytes of `r` before
  `le_bytes_to_num` rather than the number after it.
