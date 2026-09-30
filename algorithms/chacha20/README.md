# ChaCha20 and XChaCha20

ChaCha20 is the stream cipher Daniel J. Bernstein published in January 2008
as a variant of his Salsa20 (2005), and the IETF standardized in
[RFC 8439, ChaCha20 and Poly1305 for IETF Protocols](https://www.rfc-editor.org/rfc/rfc8439)
(Nir and Langley, June 2018, obsoleting RFC 7539 of 2015) with a 96-bit
nonce and a 32-bit block counter. XChaCha20 is the extended-nonce form, with
a 192-bit nonce, described in
[draft-irtf-cfrg-xchacha](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha)
(Arciszewski, versions 00 to 03, 2018 to January 2020), built on the
HChaCha20 subkey derivation the way Bernstein built XSalsa20 on HSalsa20.
ChaCha20, with Poly1305, is a cipher of TLS 1.2 and 1.3, SSH, IPsec, QUIC,
the Noise framework and WireGuard; RFC 8439 is a current IETF specification,
and XChaCha20 is an expired draft that libsodium, Botan, Go's x/crypto and
many other libraries implement as written.

## Analysis

### Structure

ChaCha20 is an ARX design: additions modulo 2^32, XORs and rotations by
fixed amounts, and nothing else. Its state is sixteen 32-bit words, written
as a 4 by 4 matrix (RFC 8439 section 2.3): the four constants that spell
"expand 32-byte k", the eight words of the 256-bit key, one word of block
counter and three words of nonce, all read little-endian. The quarter round
(section 2.1) mixes four words in eight steps, each of the form
`a += b; d ^= a; d <<<= 16`, with the rotation amounts 16, 12, 8 and 7. A
double round is a column round, the quarter round on each of the four
columns, followed by a diagonal round, the quarter round on each of the four
diagonals; the RFC writes the eight calls out as `inner_block`, and the block
function (section 2.3) runs ten of them, that is 20 rounds, then adds the
initial state word by word so that the rounds cannot be undone, and
serializes the sixteen words little-endian into 64 bytes of keystream. The
encryption function (section 2.4) is the standard stream-cipher form:
keystream block j is the block function at counter `counter + j`, and the
message is XORed with the keystream, the surplus of the last block being
discarded.

HChaCha20 (draft section 2.2) runs the same 20 rounds on a state whose last
four words hold a 128-bit nonce in place of the counter and nonce, omits the
final addition, and returns words 0 to 3 and 12 to 15 as a 256-bit subkey.
XChaCha20 (draft section 2.3) derives the subkey from the key and the first
16 bytes of its 24-byte nonce, then runs ChaCha20 under that subkey with a
12-byte nonce made of four zero bytes and the last 8 bytes of the 24-byte
nonce.

`chacha20.or` follows the two documents section by section:

| Standard section | Orange spec |
| --- | --- |
| RFC 8439, 2.1, the quarter round | `quarter_round` |
| 2.2 and 2.3, the eight quarter rounds of a double round | `inner_block` |
| 2.3, bytes to little-endian words and back | `load_le32`, `le_bytes`, `key_words` |
| 2.3, the initial state | `initial_state` |
| 2.3, the block function before serialization | `block` |
| 2.3, serialization of the state | `serialize` |
| 2.3.1, `chacha20_block` | `chacha20_block` |
| 2.4.1, `chacha20_encrypt` over whole blocks | `encrypt` (four blocks), `xor_block` |
| 2.4.1, the same for the message lengths of the vectors | `encrypt_128`, `encrypt_127`, `encrypt_119`, `encrypt_114`, `encrypt_64`, `encrypt_48` |
| draft, 2.2, HChaCha20 | `hchacha20` |
| draft, 2.3, steps 1 and 2 | `subkey`, `chacha20_nonce` |
| draft, 2.3.1, `xchacha20_encrypt` | `xchacha20_encrypt` |

`quarter_round` takes and returns the four words, and `inner_block` places
each result back into the state in one array literal, exactly as the S3f
fixture does. `block` keeps the RFC's split between the state after the
final addition (which section 2.3.2 prints as words) and its serialization
(which it prints as bytes), so both can be checked. `encrypt` is the RFC's
loop over whole blocks for a message of four blocks, with the block counter
`counter + j` as the RFC states it; `xor_block` places one keystream block at
byte offset 64j. The RFC's handling of a final partial block, one more block
function whose surplus keystream is dropped, is written once per message
length the vectors need, since an Orange array has a fixed length. The
comment on `initial_state` notes the counter that the RFC changed:
Bernstein's ChaCha (2008) keeps a 64-bit block counter in words 12 and 13 and
a 64-bit nonce in words 14 and 15; RFC 8439 gives word 12 to a 32-bit counter
and words 13 to 15 to a 96-bit nonce.

### Security status

Record as of September 2026.

ChaCha is Salsa20 with two changes in the quarter round and one in the
layout of the state. Salsa20's quarter round updates each of its four words
once, with rotations 7, 9, 13 and 18; ChaCha's updates each word twice, with
rotations 16, 12, 8 and 7 (two of them multiples of eight, cheap on every
processor), and alternates column rounds with diagonal rounds instead of
Salsa20's column and row rounds. Bernstein's stated reason (ChaCha, a variant
of Salsa20, 2008) is diffusion: each ChaCha round spreads every input bit to
more output bits than a Salsa20 round at the same cost, so that attacks that
reach a given number of Salsa20 rounds reach fewer ChaCha rounds. The
constants, key, counter and nonce are also placed by rows rather than along
Salsa20's diagonal, which is what makes the HChaCha20 argument of the draft's
section 3.1 go through: the public words of the state (constants and nonce)
are the words HChaCha20 returns.

Every published attack is on a reduced number of rounds, and all of them
descend from the differential attack with probabilistic neutral bits of
Aumasson, Fischer, Khazaei, Meier and Rechberger (FSE 2008), which recovered
the key of 7-round ChaCha in 2^248 operations and of 6-round ChaCha in
2^139. The 7-round complexity has been lowered steadily by differential-linear
cryptanalysis: 2^237.7 (Choudhuri and Maitra, 2016), 2^230.86 (Beierle,
Leander and Todo, Crypto 2020), 2^221.95 (Dey, Garai, Sarkar and Sharma,
Eurocrypt 2022), and about 2^190 with MILP-found linear approximations
(Bellini, Gerault, Grados, Makarim and Peyrin, ToSC 2023). Six rounds are far
cheaper, and attacks extended to 7.25 and 7.5 rounds cost within a few bits
of exhaustive key search. No attack reaches 8 of the 20 rounds. The margin,
more than twelve rounds of the twenty, is why the reduced variants ChaCha12
and ChaCha8 (Bernstein 2008) are also in use, XChaCha12 in Adiantum (Crowley
and Biggers, 2018) for disk encryption on Android, and why the full cipher is
considered secure with room to spare; there is no security proof for ChaCha
itself, and the draft's phrase is that ChaCha20 "is believed to be at least as
secure as Salsa20".

The nonce is where the practical hazards are. As a stream cipher, ChaCha20
turns any repeated (key, nonce) pair into a repeated keystream, so the XOR of
two ciphertexts is the XOR of their plaintexts, and it gives no integrity on
its own: a flipped ciphertext bit flips the plaintext bit, which is why every
protocol use pairs it with Poly1305. Bernstein's ChaCha has a 64-bit nonce
and a 64-bit block counter, so one (key, nonce) pair can encrypt 2^64 blocks.
RFC 8439 moved 32 bits from the counter to the nonce, following RFC 5116's
recommendation of a 96-bit nonce, and says so in section 2.3: a single
(key, nonce) pair is limited to 2^32 blocks, 256 GB, and the RFC does not
define what happens when the counter passes 2^32 - 1 (in this rendering, and
in most implementations, the `Word[32]` wraps and the keystream repeats from
block 0). RFC 8439 also says (section 2.6) that the nonce MUST be unique per key and
MUST NOT be random: 96 bits is too few for random nonces under a long-lived key,
since the collision probability reaches 2^-32 after about 2^32 messages.
XChaCha20's 192-bit nonce exists for exactly that case; the draft's own
arithmetic gives 2^80 random nonces per key at the same 2^-32 collision
threshold, and the HChaCha20 step means that a repeat of only the first 128
or only the last 64 nonce bits changes the keystream anyway. For messages
past 256 GB, RFC 8439 section 2.8 points back to the original 64-bit counter
rather than to any wraparound behaviour.

ChaCha20 is constant-time by construction: there are no tables, no
data-dependent memory addresses and no data-dependent branches, only
additions, XORs and rotations by fixed amounts, which was a design goal after
the cache-timing attacks on table-driven AES (Bernstein 2005) and is the
reason ChaCha20-Poly1305 became the cipher of choice for software without AES
instructions. An implementation still has to preserve the property; the
specification merely makes it easy.

Status: RFC 8439 (IRTF CFRG, Informational, June 2018) is the current
definition; ChaCha20-Poly1305 is a cipher suite of TLS 1.3 (RFC 8446, where
it is one of the suites every implementation should support) and of TLS 1.2
(RFC 7905), the `chacha20-poly1305@openssh.com` transport cipher of OpenSSH
since 2014, an IPsec ESP algorithm (RFC 7634), a QUIC packet protection and
header protection cipher (RFC 9001), a Noise cipher, and the only symmetric
cipher of WireGuard, which also uses XChaCha20-Poly1305 for its cookie
replies. XChaCha20 has no RFC: draft-irtf-cfrg-xchacha-03 expired in July
2020, and the construction is nonetheless a de facto standard through
libsodium (`crypto_stream_xchacha20` and `crypto_aead_xchacha20poly1305`),
where its vectors and those of the draft agree.

### What the Orange rendering shows

Nothing in the cipher depends on data. The rotation amounts are the
literals 16, 12, 8 and 7, every index in `inner_block` is a constant, the
loop bounds are 10 (double rounds) and 4 (blocks), and the file's only `if`
is in `xor_block`, on the loop index j, to place a keystream block at byte
offset 64j; it is a workaround for two language rules (a spec parameter is
not a static index, and a loop's step is one expression, so the block cannot
be bound with a `let` inside the loop) and not a property of ChaCha20. The
32-bit block counter is a `Word[32]`, so `counter + j` wraps exactly as the
RFC's 32-bit little-endian word does, and the 64-bit counter of Bernstein's
original is visible only as a comment. HChaCha20's difference from the block
function, no final addition and a different selection of words, is two
lines. What the rendering does not express is the constant-time property
discussed above: `orangec eval` evaluates a specification, and its step count
is a measure of the specification's size, not of any implementation's timing.

The costs were measured with filler specs sharing a file's budget
(`probe.py` and `probe2.py` in the scratch directory): one `quarter_round`
is about 46 steps, one `inner_block` about 411, and one `chacha20_block`
about 11,400, of which the 64 single-byte updates of `serialize` are about
4,100 (an update of an n-element array costs n steps); `hchacha20`, which
serializes only 32 bytes and skips the addition, is about 6,800. The
256-byte `encrypt` is about 116,500 steps, four blocks plus four times 64
updates of a 256-byte array, so placing the keystream costs more than
computing it. The whole file, 20 vector pairs with 35 block functions and 5
HChaCha20 calls, uses about 672,000 of the 1,048,576 steps, and 33 more
blocks would fit; no vector had to be moved or dropped.

## Dissemination

### Files

- `chacha20.or`: the algorithm (quarter round, double round, block function,
  serialization, encryption, HChaCha20, XChaCha20) and every vector below.

### Running

    orangec eval algorithms/chacha20/chacha20.or
    python3 algorithms/verify.py algorithms/chacha20

`eval` also prints the input specs `sequential_key`, `rfc8439_2_3_2_nonce`,
`sunscreen`, `ietf_text_head`, `ietf_text_tail`, `jabberwocky`,
`dhole_text_head`, `dhole_text_tail`, `xchacha_draft_key` and
`xchacha_draft_nonce`, which have no `_expected` twin and are not vectors.

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `rfc8439_2_1_1` | RFC 8439, section 2.1.1 | the quarter round on a = 11111111, b = 01020304, c = 9b8d6f43, d = 01234567 |
| `rfc8439_2_2_1` | RFC 8439, section 2.2.1 | QUARTERROUND(2, 7, 8, 13) on the sample state 879531e0 ... 91dbd320 |
| `rfc8439_2_3_2_state` | RFC 8439, section 2.3.2 | key 00:01:...:1f, nonce 00:00:00:09:00:00:00:4a:00:00:00:00, block count 1; the state after the final addition |
| `rfc8439_2_3_2` | RFC 8439, section 2.3.2 | the same block, serialized: 10 f1 e7 e4 ... |
| `rfc8439_2_4_2` | RFC 8439, section 2.4.2 | the 114-byte "sunscreen" plaintext, key 00:01:...:1f, nonce 00:00:00:00:00:00:00:4a:00:00:00:00, initial counter 1 |
| `rfc8439_a1_1` | RFC 8439, appendix A.1, test vector 1 | zero key, zero nonce, block counter 0 |
| `rfc8439_a1_2` | RFC 8439, appendix A.1, test vector 2 | zero key, zero nonce, block counter 1 |
| `rfc8439_a1_3` | RFC 8439, appendix A.1, test vector 3 | key 00...01, zero nonce, block counter 1 |
| `rfc8439_a1_4` | RFC 8439, appendix A.1, test vector 4 | key 00ff00...00, zero nonce, block counter 2 |
| `rfc8439_a1_5` | RFC 8439, appendix A.1, test vector 5 | zero key, nonce 00...02, block counter 0 |
| `rfc8439_a2_1` | RFC 8439, appendix A.2, test vector 1 | 64 zero bytes, zero key, zero nonce, initial counter 0 |
| `rfc8439_a2_2_head` | RFC 8439, appendix A.2, test vector 2 | bytes 0 to 255 of the 375-byte IETF text, key 00...01, nonce 00...02, initial counter 1 (blocks 1 to 4) |
| `rfc8439_a2_2_tail` | RFC 8439, appendix A.2, test vector 2 | bytes 256 to 374 of the same message, counter 5 (blocks 5 and 6) |
| `rfc8439_a2_3` | RFC 8439, appendix A.2, test vector 3 | the 127-byte "Jabberwocky" stanza, key 1c:92:40:a5:..., nonce 00...02, initial counter 42 |
| `xchacha_draft_2_2_1` | draft-irtf-cfrg-xchacha-03, section 2.2.1 | HChaCha20 of key 00:01:...:1f and nonce 00:00:00:09:00:00:00:4a:00:00:00:00:31:41:59:27 |
| `xchacha_draft_a2_1_head` | draft-irtf-cfrg-xchacha-03, appendix A.2.1 (A.3.2.1) | bytes 0 to 255 of the 304-byte "dhole" text, key 80:81:...:9f, nonce 40:41:...:57, block counter 0 |
| `xchacha_draft_a2_1_tail` | draft-irtf-cfrg-xchacha-03, appendix A.2.1 (A.3.2.1) | bytes 256 to 303 of the same message, under the same subkey and nonce, counter 4 |
| `xchacha_draft_a2_2_head` | draft-irtf-cfrg-xchacha-03, appendix A.2.2 (A.3.2.2) | the same message, key and nonce from block counter 1: bytes 0 to 255 (blocks 1 to 4) |
| `xchacha_draft_a2_2_tail` | draft-irtf-cfrg-xchacha-03, appendix A.2.2 (A.3.2.2) | bytes 256 to 303 of the same, counter 5 |
| `botan_xchacha_3` | Botan `src/tests/data/stream/chacha.vec`, third case under "XChaCha tests" | 128 bytes of XChaCha20 keystream, key 00:01:...:1f, nonce 00:01:...:17 |

Every expected value is copied from the named source; none was produced by
an oracle. A message longer than 256 bytes cannot be one Orange array, so
A.2 test vector 2 (375 bytes) and the draft's two examples (304 bytes each)
are each passed as two arrays, the second encrypted from the block counter
its first block has (5, 4 and 5), which is what the RFC's `chacha20_encrypt`
loop would have reached; the two halves together are the source's whole
ciphertext. The draft prints each example twice, as a hex dump in A.2 and as
continuous hex in A.3.2; `ref.py` parses both and asserts they agree.

### Provenance and claims

RFC 8439 and the draft are unreachable from the build machine, so their
text came from verbatim copies on raw.githubusercontent.com: `rfc8439.txt`
from `smuellerDD/leancrypto` (`aead/doc/`), byte-identical to the copies in
`mnot/rfc-refs` and `markkurossi/gotls`, and `draft-irtf-cfrg-xchacha-03.txt`
from `rust-stdx/stdx` (`crypto/docs/`). A Python reference of both documents
(`ref.py` in the scratch directory) parses every vector out of those texts
(the hex dumps of sections 2.1.1, 2.2.1, 2.3.2, 2.4.2 and appendices A.1 and
A.2, and the draft's section 2.2.1, A.2.1 and A.3.2), reproduces each of
them, and agrees with pycryptodome's `ChaCha20` (12-byte nonce, and 24-byte
nonce for XChaCha20) and with the `cryptography` package's `ChaCha20`
(16-byte nonce of counter || nonce) on fifty random keys, nonces, counters
and lengths. The same reference reproduces all 502 valid `ct` fields of
Wycheproof's `chacha20_poly1305_test.json` and `xchacha20_poly1305_test.json`
(`testvectors_v1`, 96-bit and 192-bit nonces), the 4 XChaCha20 cases of
Botan's `chacha.vec` (its other cases are not used), the RFC 7539 vectors as
OpenSSL's `evpciph_chacha.txt` copies them, and libsodium's ten HChaCha20 and
ten XChaCha20 vectors from `test/default/xchacha20.c` (`crosscheck.py`). The four constants
`0x61707865, 0x3320646e, 0x79622d32, 0x6b206574` are "expand 32-byte k" in
little-endian words, checked by `struct.unpack`. The Orange literals were
rendered from the parsed bytes by `gen.py` and assembled into `chacha20.or`
by `build_or.py`, never typed by hand, and the input specs the file states
inline (keys, nonces, counters) are asserted equal to the parsed inputs in
that script.

This entry is a reference evaluation of RFC 8439 and
draft-irtf-cfrg-xchacha-03 under `orangec eval`. It makes no constant-time,
side-channel, performance or certification claim; the constant-time
discussion above is about the design, not about this evaluator. It is not a
corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- Arrays hold at most 256 elements, so the two messages longer than that
  (375 and 304 bytes) are passed as two arrays with the second array's
  block counter set by hand; the RFC's single `chacha20_encrypt` call over
  the whole message is not one spec here.
- Arrays have no length parameter, so the partial-block case of section
  2.4.1 is written once per message length (`encrypt_48`, `encrypt_64`,
  `encrypt_114`, `encrypt_119`, `encrypt_127`, `encrypt_128`), six copies
  of the same six lines.
- A spec parameter is not a static index and a loop's step is a single
  expression (ORC0101 on a `let` inside it), so the keystream block of
  iteration j cannot be bound and written at `64 * j + i` in one loop;
  `xor_block` selects the offset with a four-arm conditional on j instead,
  costing four comparisons per block.
- An update of an n-element array costs n steps, so placing 64 bytes into a
  256-byte ciphertext costs about 16,400 steps, more than the 11,400 of the
  block function; the file still uses about two thirds of its budget, so nothing
  was split or dropped.
