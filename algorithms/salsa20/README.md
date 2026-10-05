# Salsa20 and XSalsa20

Salsa20 is the stream cipher Daniel J. Bernstein designed in 2005 and
submitted to the eSTREAM project, defined in the
[Salsa20 specification](https://cr.yp.to/snuffle/spec.pdf) (sections 3 to 10)
and described in [The Salsa20 family of stream ciphers](https://cr.yp.to/snuffle/salsafamily-20071225.pdf)
(2007): a 256-bit or 128-bit key, a 64-bit nonce, a 64-bit block counter, and
64-byte keystream blocks produced by twenty rounds of additions, rotations and
XORs. Salsa20/12 and Salsa20/8 are the same cipher with twelve and eight
rounds; Salsa20/12 is one of the four software ciphers of the eSTREAM
portfolio (2008). XSalsa20 is the extended-nonce form of
[Extending the Salsa20 nonce](https://cr.yp.to/snuffle/xsalsa-20110204.pdf)
(Bernstein, 2011): a 192-bit nonce, with HSalsa20 deriving a subkey from the
key and the first 128 nonce bits. Salsa20 is not an IETF or NIST standard;
it is the design ChaCha20 (2008, RFC 8439) was derived from, and XSalsa20 is
the cipher of NaCl's and libsodium's `crypto_secretbox` and `crypto_box` and
of DNSCurve, in use since 2008 and unbroken.

## Analysis

### Structure

Salsa20 is an ARX design. Its state is sixteen 32-bit words, written as a
4 by 4 matrix, and its only operations are addition modulo 2^32, rotation by
a fixed amount and XOR. The quarterround (specification section 3) takes
four words and modifies each once, in the order z1, z2, z3, z0, by XORing it
with the rotated sum of the two words before it: rotations 7, 9, 13 and 18.
The rowround (section 4) applies the quarterround to each row of the matrix,
starting at the row's diagonal word, and the columnround (section 5) does
the same to each column, again starting at the diagonal; the specification
points out that the columnround is the transpose of the rowround. A
doubleround (section 6) is a columnround followed by a rowround. The Salsa20
hash function (section 8) reads sixteen little-endian words (section 7) from
a 64-byte input, applies ten doublerounds, and adds the input words to the
result before writing 64 little-endian bytes; the addition is what makes the
function one-way, since the doubleround alone is a permutation.

The expansion function (section 9) builds the 64-byte hash input from a key
and a 16-byte string n. For a 32-byte key k = (k0, k1) the input is
(sigma0, k0, sigma1, n, sigma2, k1, sigma3) with sigma the sixteen bytes of
"expand 32-byte k"; for a 16-byte key k it is (tau0, k, tau1, n, tau2, k,
tau3), the key standing in both positions, with tau the bytes of
"expand 16-byte k". In the matrix the four constants are the diagonal (words
0, 5, 10, 15), the key words 1 to 4 and 11 to 14, and n words 6 to 9. This
layout differs from ChaCha's, which places the constants, key, counter and
nonce by rows. The encryption function (section 10) takes n = (v, i), the
8-byte nonce v and the 8-byte little-endian block counter i, so that the
nonce is words 6 and 7 and the counter words 8 (low) and 9 (high); the
keystream is the blocks Salsa20_k(v, 0), Salsa20_k(v, 1), ..., and the
ciphertext is the message XORed with it, the surplus of the last block being
discarded. Salsa20/r replaces the ten doublerounds with r/2.

HSalsa20 (Extending the Salsa20 nonce) runs the same twenty rounds
on the expansion of a 32-byte key and a 16-byte n, omits the final addition,
and returns the words z0, z5, z10, z15, z6, z7, z8, z9, that is the diagonal
and the n positions, as a 32-byte subkey. XSalsa20 encrypts under the subkey
HSalsa20_k(n[0..15]) with Salsa20 and the 8-byte nonce n[16..23].

`salsa20.or` follows the two documents section by section:

| Standard section | Orange spec |
| --- | --- |
| Specification, 3, quarterround | `quarter_round` |
| 4, rowround | `row_round` |
| 5, columnround | `column_round` |
| 6, doubleround | `double_round` |
| 7, littleendian and its inverse | `load_le32`, `le_bytes`, `little_endian_16`, `little_endian_32` |
| 8, the Salsa20 hash function | `double_rounds`, `salsa20_hash`, `serialize` |
| 9, the expansion for 32-byte and 16-byte keys | `expand_32`, `expand_16` |
| 10, n = (v, i) and one keystream block | `nonce_counter`, `salsa20_block`, `keystream_block` |
| 10, encryption, for the message lengths of the vectors | `encrypt_32`, `encrypt_39`, `encrypt_64`, `encrypt_111`, `encrypt_128`, `encrypt_139`, `encrypt_192`, `encrypt_238` |
| Extending the Salsa20 nonce, HSalsa20 | `hsalsa20` |
| XSalsa20 | `xsalsa20_state` with the `encrypt_` specs |

`quarter_round` takes and returns its four words; `row_round` and
`column_round` place each result back into the state in one array literal,
with the specification's index pattern visible in the literal. The two
constant strings are written as their little-endian words with the ASCII in
the comment (`0x61707865` is "expa"). `double_rounds` takes the round count
as an `Int` and runs a loop of ten iterations in which those past r/2 leave
the state unchanged, because a loop's bounds must be literals; the same spec
serves Salsa20/20, Salsa20/12 and Salsa20/8. The 64-bit block counter is one
`Word[64]` assembled from words 8 and 9 in `keystream_block`, so the carry
from word 8 into word 9 at counter 2^32 is computed as the specification's
64-bit integer, and two vectors cross that boundary. HSalsa20 differs from
the block function in two lines: no final addition and a selection of eight
words. An Orange array has a fixed length, so the section 10 encryption is
written once per message length the vectors need, as the ChaCha20 entry does.

### Security status

Record as of September 2026.

Salsa20 was designed against the timing attacks on table-driven AES that
Bernstein published the same year: the quarterround is add-rotate-xor and
nothing else, there are no key-dependent tables, no data-dependent memory
addresses and no data-dependent branches, so a straightforward
implementation is constant-time. The hash core, ten doublerounds, is an
invertible permutation of the 64-byte state; the cipher is the core in
counter mode with the input added back, and the diagonal constants ensure
that a quarter of every input is fixed and public. Without the constants the
core alone is not collision-resistant: Hernandez-Castro, Tapiador and
Quisquater (FSE 2008) showed that adding 2^31 to every word of the input
leaves the core's output unchanged, which Bernstein's expansion excludes by
fixing the diagonal, and the specification itself says the hash function is
not meant as a cryptographic hash.

Every published key-recovery attack is on a reduced number of rounds, and
none reaches the twelve of Salsa20/12. Crowley (2005) attacked 5 rounds with
a truncated differential, Fischer, Meier, Berbain, Biasse and Robshaw
(Indocrypt 2006) 6 rounds, and Tsunoo, Saito, Kubo, Suzaki and Nakashima
(SASC 2007) 7 rounds, with a marginal 8-round attack at 2^255. The
differential attack with probabilistic neutral bits of Aumasson, Fischer,
Khazaei, Meier and Rechberger (FSE 2008) is the reference point: Salsa20/8
with a 256-bit key in 2^251 operations, Salsa20/7 in 2^151, and Salsa20/7
with a 128-bit key in 2^111. The 8-round complexity has since been lowered
by better differentials and differential-linear analysis, to about 2^250
(Shi, Zhang, Feng and Wu, ICISC 2012), 2^244.9 (Choudhuri and Maitra,
ToSC 2016), 2^243.7 (Dey and Sarkar, 2017) and, by Coutinho, Passos,
Vasconcelos, de Sousa and Borges (Asiacrypt 2022), to about 2^217 with a
first 8-round attack on the 128-bit key; the exact exponents of the 2017
and 2022 results are not checked from this machine. All of these are far
beyond any computation, and none extends to 9 rounds. Salsa20/20 therefore
has a margin of twelve rounds over the best attack, and Salsa20/12 a margin
of four, which is the reasoning behind eSTREAM's choice of Salsa20/12 for
its final portfolio (September 2008, Profile 1, alongside HC-128, Rabbit and
SOSEMANUK) and behind Bernstein's own recommendation of Salsa20/20 as the
default and Salsa20/12 and Salsa20/8 where speed matters; Salsa20/8's core
is also the mixing function of scrypt (Percival, 2009).

XSalsa20 has a proof rather than a cryptanalytic record. Bernstein (2011)
shows that if Salsa20 is a secure pseudorandom function of its 16-byte input
then so is HSalsa20, because the eight words HSalsa20 returns are exactly
the positions where the expansion's input is public (the constants and n),
so each of them can be recovered from a Salsa20 output by subtracting a
public word; XSalsa20, Salsa20 under an HSalsa20 subkey, is then a cascade
of two pseudorandom functions and inherits their security. The argument
depends on the diagonal layout of the constants, and the same argument is
what draft-irtf-cfrg-xchacha adapted for HChaCha20.

The practical hazards are those of every stream cipher. A repeated
(key, nonce) pair repeats the keystream, so the XOR of two ciphertexts is
the XOR of the plaintexts, and the cipher gives no integrity: a flipped
ciphertext bit flips the plaintext bit, which is why NaCl only exposes it
with Poly1305 in `crypto_secretbox`. The 64-bit nonce is too short to choose
at random under a long-lived key (a collision is expected after about 2^32
messages), and that is the reason XSalsa20 exists: its 192-bit nonce can be
random, and a repeat of only the first 128 or only the last 64 nonce bits
still changes the keystream. The 64-bit counter allows 2^64 blocks, 2^70
bytes, per (key, nonce) pair; this rendering wraps it modulo 2^64 as an
implementation with a 64-bit integer would, and the specification does not
define what happens beyond that. The 128-bit key variant has a 128-bit
security level only against exhaustive search; the reduced-round results
above are correspondingly cheaper for it.

Status: Salsa20 has no RFC and no NIST approval and is not a standard of
either body; the eSTREAM portfolio, last revised in 2012, still lists
Salsa20/12. Its design is the direct ancestor of ChaCha20 (Bernstein, 2008),
which the IETF standardized in RFC 8439 and which has displaced Salsa20 in
new protocols. XSalsa20-Poly1305 remains the `crypto_secretbox` and
`crypto_box` primitive of NaCl (2008) and libsodium, and the cipher of
DNSCurve (2009); libsodium also exposes Salsa20, Salsa20/12 and Salsa20/8 as
`crypto_stream` variants and the core functions used by this entry's
vectors.

### What the Orange rendering shows

Nothing in the cipher depends on data. The rotation amounts are the literals
7, 9, 13 and 18, every index in `row_round` and `column_round` is a
constant, the loop bounds are 10 (doublerounds) and the block and byte
counts, and the file's only `if` is in `double_rounds`, on the loop index
against the round count, so that one spec serves Salsa20/20, Salsa20/12 and
Salsa20/8; it is a consequence of literal loop bounds, not a property of the
cipher. The layout of the expansion is written out as one sixteen-element
literal in `expand_32` and `expand_16`, so the diagonal constants, the two
key halves and the (v, i) block are read directly, and the difference
between the 32-byte and 16-byte keys, sigma against tau and k1 against a
second copy of k, is two words and four positions. The 64-bit counter is a
`Word[64]` split into words 8 and 9 and reassembled, so the carry that the
counter-crossing vectors exercise is the ordinary addition of a 64-bit word.
HSalsa20's difference from the block function, no final addition and a
selection of eight words, is two lines, and the reader can see that the
selected positions are the ones the expansion fills with public words. What
the rendering does not express is the constant-time property discussed
above: `orangec eval` evaluates a specification, and its step count measures
the specification's size, not any implementation's timing.

The costs were measured with a loop of N calls sharing a file's budget
(`probe.py` in the scratch directory): one `quarter_round` is about 35
steps, one `double_round` about 430, and one `salsa20_block` with its
expansion about 11,900 (Salsa20/12: about 10,100), of which the 64
single-byte updates of `serialize` are about 4,100, since an update of an
n-element array costs n steps. `hsalsa20` is about 7,300. `encrypt_64` is
about 16,600 and `encrypt_238` about 105,000: four blocks plus 238 updates
of a 238-byte array, so placing the keystream costs more than computing it.
The whole file, 19 vector pairs with 27 block functions and 5 HSalsa20
calls, uses about 558,000 of the 1,048,576 steps, with headroom for about
40 more blocks; no vector had to be moved or dropped.

## Dissemination

### Files

- `salsa20.or`: the algorithm (quarterround, rowround, columnround,
  doubleround, the hash function, both expansions, the counter and
  encryption of section 10, HSalsa20, XSalsa20) and every vector below.

### Running

    orangec eval algorithms/salsa20/salsa20.or
    python3 algorithms/verify.py algorithms/salsa20

`eval` also prints the input specs `estream_set6_vector3_key`,
`estream_set1_vector0_key`, `nacl_first_key`, `botan_13_message_head` and
`botan_4_message`, which have no `_expected` twin and are not vectors.

### Vectors

Botan's `salsa20.vec` states no set or vector numbers; where a case is an
eSTREAM vector, the row says how it was identified. Crypto++'s `salsa.txt`
labels its eSTREAM cases and cites the eSTREAM `verified.test-vectors`
files it copied them from.

| Spec | Source | Case |
| --- | --- | --- |
| `botan_salsa20_1` | Botan `src/tests/data/stream/salsa20.vec`, first case | 128-bit key 00 01 ... 0f, zero nonce, first 39 bytes of keystream; by its key, eSTREAM set 3, vector 0 (128-bit) |
| `botan_salsa20_2` | Botan `salsa20.vec`, second case | 256-bit key 1b 1c ... 3a, zero nonce, first 111 bytes of keystream; by its key, eSTREAM set 3, vector 27 (256-bit) |
| `botan_salsa20_9` | Botan `salsa20.vec`, ninth case | key 0f 62 b5 08 ..., nonce 28 8f f6 5d c4 2b 92 f9, stream[0..63]; the key and IV Crypto++ labels eSTREAM set 6, vector 3 |
| `botan_salsa20_10` | Botan `salsa20.vec`, tenth case | the same key and nonce, Seek 65472 = block counter 1023, stream[65472..65535] |
| `botan_salsa20_13_head` | Botan `salsa20.vec`, thirteenth case ("Long random inputs/outputs") | key 00 01 ... 1f, nonce a0 a1 ... a7, the first 128 bytes of the 2600-byte message and ciphertext |
| `botan_salsa20_14` | Botan `salsa20.vec`, fourteenth case | 128-bit key b0 b1 ... bf, nonce c0 c1 ... c7, the first 64 of 1300 keystream bytes |
| `botan_salsa20_16_head` | Botan `salsa20.vec`, sixteenth case | key ff fe ... e0, nonce d0 d1 ... d7, Seek 274877906816 = counter 0xfffffffe, the first 192 of 2048 keystream bytes: counters 0xfffffffe, 0xffffffff, 0x100000000 |
| `botan_xsalsa20_3` | Botan `salsa20.vec`, third case | XSalsa20, key 1b 27 55 64 ..., 24-byte nonce 69 69 6e e9 ..., first 139 bytes of keystream (NaCl's stream test key and nonce) |
| `botan_xsalsa20_4` | Botan `salsa20.vec`, fourth case | XSalsa20, key a6 a7 25 1c ..., nonce 9e 64 5a 74 ..., a 238-byte message |
| `cryptopp_set1_vector0` | Crypto++ `TestVectors/salsa.txt`, "Set 1, vector# 0" (eSTREAM 128-bit key file) | key 80 00 ... 00, zero IV, stream[0..63] |
| `cryptopp_set1_vector0_seek448` | Crypto++ `salsa.txt`, the same case, Seek 448 | block counter 7, stream[448..511] |
| `cryptopp_set3_vector243` | Crypto++ `salsa.txt`, "Set 3, vector#243" (eSTREAM 256-bit key file) | key f3 f4 ... 12, zero IV, stream[0..63] |
| `cryptopp_salsa20_12_set1_vector0` | Crypto++ `salsa.txt`, Rounds 12, "Set 1, vector# 0" (eSTREAM reduced/12-rounds) | Salsa20/12, key 80 00 ... 00, zero IV, stream[0..63] |
| `cryptopp_salsa20_8_set1_vector0` | Crypto++ `salsa.txt`, Rounds 8, "Set 1, vector# 0" (eSTREAM reduced/8-rounds) | Salsa20/8, the same key and IV, stream[0..63] |
| `cryptopp_counter_crossing_head` | Crypto++ `salsa.txt`, "Counter crosses 32-bit boundary (0xffffffff*64)" | zero key and IV, Seek64 0x3fffffffc0 = counter 0xffffffff, the first 128 of 1024 bytes: counters 0xffffffff and 0x100000000 |
| `libsodium_core4` | libsodium `test/default/core4.c` and `core4.exp` | `crypto_core_salsa20`: the hash of the 32-byte-key expansion with key 1, 2, ..., 216 and input 101, ..., 116 |
| `libsodium_core1` | libsodium `test/default/core1.c` and `core1.exp` | `crypto_core_hsalsa20`: HSalsa20 of the shared key 4a 5d 9d 5b ... and a zero input |
| `libsodium_core2` | libsodium `test/default/core2.c` and `core2.exp` | HSalsa20 of core1's first key and the nonce prefix 69 69 6e e9 ... |
| `libsodium_stream3` | libsodium `test/default/stream3.c` and `stream3.exp` | `crypto_stream` (XSalsa20) under the first key and the 24-byte nonce 69 69 ... 0b 37, first 32 bytes |

Every expected value is copied from the named file; none was produced by an
oracle. Vectors longer than one Orange array, or longer than needed, are
reproduced as their first 64, 128 or 192 bytes, as the row says; the rest of
each file's value was checked with the Python reference. `botan_salsa20_9`,
`botan_salsa20_10` and Crypto++'s "Set 6, vector# 3" are one eSTREAM case:
Crypto++ gives only the XOR of its 2048 blocks, which is not a reproducible
spec here, and Botan gives blocks 0 and 1023 of the same stream.

### Provenance and claims

The specification, the XSalsa20 paper and the eSTREAM `verified.test-vectors`
files are unreachable from the build machine. The algorithm was written from
the specification's section structure and checked against three independent
implementations fetched from raw.githubusercontent.com: Botan's
`src/lib/stream/salsa20/salsa20.cpp` (which fixed the HSalsa20 output
positions), Go's `x/crypto/salsa20/salsa/salsa20_ref.go`, and libsodium's
tests. A Python reference of the specification (`ref.py` in the scratch
directory, sections 3 to 10 as functions, then HSalsa20 and XSalsa20) agrees
with pycryptodome's `Salsa20` on 200 random keys of both sizes, nonces and
lengths, and reproduces (`check.py`) all 17 cases of Botan's `salsa20.vec`,
all 110 Salsa20, Salsa20/12, Salsa20/8, counter-crossing and XSalsa20 cases
of Crypto++'s `salsa.txt` including the 131072-byte XOR digest of set 6,
vector 3, libsodium's `core1`, `core2`, `core4` and `stream3` expected
outputs, and the four set 6 XOR digests of Go's `salsa20_test.go`. The
constants `0x61707865, 0x3320646e, 0x79622d32, 0x6b206574` and
`0x61707865, 0x3120646e, 0x79622d36, 0x6b206574` are "expand 32-byte k" and
"expand 16-byte k" in little-endian words, checked by `struct.unpack`, and
libsodium's `core1.c` and `core4.c` carry the first string as bytes. The
Orange literals of every key, nonce, message and expected value were
rendered from the parsed files by `gen.py`, which asserts each case against
`ref.py`, and against pycryptodome's `Salsa20` where the library can reach
the case (8-byte nonce, counter below 2^14), before writing `salsa20.or`;
nothing was typed by hand. pycryptodome 3.23 rejects a 24-byte nonce for
`Salsa20`, so XSalsa20 has no library oracle here: its values are the files'
and `ref.py` reproduces them through both routes (HSalsa20 then Salsa20, and
Salsa20 under libsodium's published second key).

The identification of Botan's first and second cases as eSTREAM set 3,
vectors 0 and 27, rests on the set's construction (the key bytes are the
vector number and its successors), which Crypto++'s labelled "Set 3,
vector#243" case (key f3 f4 f5 ...) exhibits; Botan does not say so.

This entry is a reference evaluation of the Salsa20 specification and of
Extending the Salsa20 nonce under `orangec eval`. It makes no constant-time,
side-channel, performance or certification claim; the constant-time
discussion above is about the design, not about this evaluator. It is not a
corpus entry in the sense of The Orange Book chapter 12.

## Gaps

- The files' long cases (1024 to 2600 bytes) are reproduced as their first
  128 or 192 bytes. Those lengths fit in one array (the bound is 65,536);
  the sources were written when it was 256 and were not widened. Crypto++'s
  131,072-byte figure for set 6, vector 3 is the stream covered by the XOR
  of 2,048 blocks. The stored result is one 64-byte block, which a loop of
  2,048 iterations can accumulate under the current loop bound. The entry
  does not record that pair; the omission is not an array-length gap past
  65,536.
- Arrays have no length parameter, so the encryption of section 10 is
  written once per message length (`encrypt_32` to `encrypt_238`), eight
  copies of the same few lines.
- Loop bounds are literals, so Salsa20/r is a ten-iteration loop with an
  `if` on the iteration index rather than a loop of r/2 iterations; the
  unused iterations cost one comparison each.
- An update of an n-element array costs n steps, so placing four keystream
  blocks into a 238-byte ciphertext costs about 57,000 steps against 47,600
  for computing them; the file still uses about half its budget, so nothing
  was split or dropped.
- pycryptodome 3.23 does not implement XSalsa20 (a 24-byte nonce is
  rejected), so the XSalsa20 vectors have the files and the Python reference
  as their only checks.
