# Twofish

Twofish is a 128-bit block cipher with 128-, 192- and 256-bit keys, designed
by Bruce Schneier, John Kelsey, Doug Whiting, David Wagner, Chris Hall and
Niels Ferguson at Counterpane and submitted to the AES competition in June
1998. It is specified in the paper
["Twofish: A 128-Bit Block Cipher"](https://www.schneier.com/academic/twofish/)
(15 June 1998) and, at book length, in *The Twofish Encryption Algorithm*
(Wiley, 1999); it was one of the five finalists of the competition that
Rijndael won in October 2000. It is unpatented and was placed in the public
domain by its designers. It is used in disk encryption (TrueCrypt and its
successor VeraCrypt) and in OpenPGP (algorithm 10 of
[RFC 4880](https://www.rfc-editor.org/rfc/rfc4880), implemented by GnuPG),
and it is standardized neither by NIST nor by the IETF. As of 2026 it is
unbroken: no attack on the full sixteen rounds is known.

## Analysis

### Structure

Twofish is a sixteen-round Feistel network on four 32-bit words, with the
plaintext xored with four subkeys before the first round (input whitening)
and the result xored with four more after the last (output whitening). The
round function F takes the two left words R_0 and R_1, passes R_0 and
ROL(R_1, 8) through the key-dependent function g, mixes the two results with
a pseudo-Hadamard transform (T_0 + T_1 and T_0 + 2 T_1, modulo 2^32), adds
two round subkeys, and xors the two sums into R_2 and R_3, with a rotation
right by one bit after the xor on R_2 and a rotation left by one bit before
it on R_3; then the halves swap. The function g splits its word into four
bytes, passes each through a key-dependent 8-bit S-box, and multiplies the
four results as a column vector by a 4 x 4 maximum-distance-separable
matrix over GF(2^8) with the polynomial x^8 + x^6 + x^5 + x^3 + 1.

The S-boxes are where Twofish differs from the other finalists. Each is a
composition of two or three fixed 8-bit permutations, q0 and q1, with xors
of key bytes between them; the key bytes are the k = 2, 3 or 4 words S_i,
each the product of eight key bytes by a 4 x 8 Reed-Solomon matrix over
GF(2^8) with the polynomial x^8 + x^6 + x^3 + x^2 + 1. The permutations q0
and q1 are themselves built from four 4-bit permutations t0 to t3 each: the
byte splits into nibbles a and b, two stages form a xor b and
a xor ROR4(b, 1) xor 8a mod 16 and send the pair through two of the tables,
and the last pair is reassembled. The paper defines the general function
h(X, L) as this same construction with a list L of k words in place of S,
followed by the MDS matrix; g(X) = h(X, S). The key schedule uses h twice
more: with the even key words Me on the multiples 2i of rho = 0x01010101
and with the odd key words Mo on (2i + 1) rho, rotated, added and rotated
again to give the forty expanded key words K_0 to K_39. Decryption runs the
same rounds backwards with the same F.

The file `twofish.or` follows the paper section by section. A block is
`Word[8]^16`, read as four little-endian words with `as little Word[32]^4`
and written back the same way. One key schedule serves the three key
lengths through the size parameter `k` in 2 through 4, the paper's
k = N / 64: the key is `Word[8]^(8 * k)`, the S-box key words are
`Word[32]^k`, and h, g, F, the round and the cipher carry the same size,
so a test picks the key length by the length of its key. The key schedule
returns a tuple, the forty expanded key words K_0 to K_39 and the S-box
key words in the order (S_(k-1), ..., S_0) that g takes them. The tables
of q0 and q1 are a `QTables`, four rows of sixteen bytes, and the MDS and
RS matrices are 4 x 4 and 4 x 8 arrays of bytes indexed by row and column
as the paper prints them.

| Paper section | Orange spec |
| --- | --- |
| 4, the little-endian words of a block | `as little Word[32]^4` and `as little Word[8]^16` in `encrypt` and `decrypt` |
| 4, whitening and the sixteen rounds | `encrypt[k]`, `round[k]`; `decrypt[k]`, `inverse_round[k]` |
| 4.1, the function F with the PHT and the round subkeys | `f[k]` |
| 4.2, the MDS matrix over GF(2^8) modulo v(x) | `times_x`, `gf_walk`, `gf_mul`, `mds_matrix`, `mds` |
| 4.2 and 4.3.3, g(X) = h(X, S) | `g[k]` |
| 4.3, the key words M_i, Me, Mo and the RS matrix modulo w(x) | `key_schedule[k]`, `rs_matrix`, `rs` |
| 4.3.2, the function h | `stage`, `h[k]` |
| 4.3.4, the expanded key words K_j | `key_schedule[k]` |
| 4.3.5, q0 and q1 from the 4-bit tables t0 to t3 | `q0_tables`, `q1_tables`, `ror4`, `q`, `q0`, `q1` |

### Security status

Twofish's security argument rests on two features. The S-boxes are
key-dependent, so a differential or linear characteristic cannot be
computed once for all keys, and the RS code, a [12, 8, 5] code over
GF(2^8), makes the S-box key words of two keys differ whenever the keys
differ in at most four of the eight bytes feeding them; the MDS matrix gives
every g a branch number of 5 (a change in one input byte changes all four
output bytes), and the PHT, the 1-bit rotations and the whitening spread
that diffusion across the words and deny an attacker the first and last
round functions. The designers documented their own cryptanalysis in the
paper and in the Twofish Technical Reports of 1998 to 2000,
which include Ferguson's impossible differentials through six rounds
(1999), Kelsey's related-key attacks on reduced rounds (2000) and upper
bounds on differential characteristics.

The published attacks, as of 2026, all stop well short of sixteen rounds.
Knudsen's truncated differentials (2000, "Trawling Twofish") and Moriai and
Yin's truncated differentials (2000) distinguish or attack about six
rounds; Ferguson's impossible differential covers six rounds; Lucks'
saturation attack (FSE 2001) reaches seven rounds at near-codebook data.
Murphy and Robshaw (2000, published 2002) observed that key-dependent
S-boxes admit, for some keys, differential characteristics stronger than
the designers' key-averaged bounds; no key class exploitable against more
than a few rounds followed. Since 1998 no related-key, algebraic or
structural attack has reached the full cipher, no published attack goes
beyond eight of the sixteen rounds, and the margin is half the cipher.

In the AES competition, NIST's Round 2 report (Nechvatal et al., October
2000) rated Twofish, Serpent and MARS as having a high security margin and
Rijndael and RC6 an adequate one, then chose Rijndael for the combination of
security, performance across platforms, low memory, simplicity and
flexibility. Twofish lost on the last three: its key-dependent S-boxes
make the key setup expensive or the S-boxes large, its design is harder to
analyse than an SPN with fixed S-boxes, and its speed advantage on the
Pentium of 1998 was not general across platforms. Since then
Twofish has been neither standardized by NIST nor given IETF cipher suites;
it persists in disk encryption and in OpenPGP, where it is optional and
seldom negotiated, and in libraries such as Botan, Crypto++ and libgcrypt.
It carries the same pitfalls as any 128-bit block cipher: a single block,
as evaluated here, is not a mode of operation, and its security in use
depends on the mode and on nonce or IV discipline, not on the cipher.

### What the Orange rendering shows

Two things in Twofish are data-dependent: the four nibble lookups inside
each q permutation and the key-dependent xors around them. The 4-bit
tables are written as the paper prints them, one entry per byte of a
`hex"..."` row, and a nibble selects its entry with `t[0][a1 % 16]`: the
nibbles live in `Word[8]`, and the reduction mod 16, which the paper's
nibbles already satisfy, is what lets the checker prove each index below
16. The split of the byte into a0 and b0, ROR4 and 8a mod 16 read as the
paper writes them (`x / 16`, `x % 16`, `(8 * a0) % 16`), and the byte is
reassembled as `(16 * b4) + a4`. Nothing is precomputed: an
implementation would expand the four key-dependent S-boxes, once per key,
into 1024 bytes or four 256-word tables, whereas here g is written as the
paper defines it, as h applied to the S-box key words, and every S-box
evaluation recomputes its two or three fixed permutations and its key
xors. That makes the rendering the paper's definition rather than the
paper's implementation notes, and it makes the cost of a block scale with
k.

The function h runs its stages in a loop from L_(k-1) down to L_0, the
word `l[k - 1 - s]` at step s, and `stage` holds the four rows of the
paper's equations, choosing q0 or q1 per byte for the stage of L_3, L_2,
L_1 or L_0; the paper skips the stages of L_3 and L_2 for shorter keys,
and the loop's length k does the same. The key schedule writes Me and Mo
as the even and odd words of `key as little Word[32]^(2 * k)`, S_i as the
RS product of the slice `key[8 * i..8 * i + 8]`, and K_(2i) and K_(2i+1)
as section 4.3.4 writes them. Byte orders are stated once each, where the
paper fixes them: blocks, key words and the outputs of the MDS and RS
products are little-endian.

The GF(2^8) arithmetic is written out: `times_x` multiplies by x modulo a
polynomial passed as its full nine-bit value, 0x169 for v(x) and 0x14D
for w(x), and `gf_mul` adds a x^i for each bit i of b. Both matrices are
applied by the same row-by-column sum of `gf_mul` products over their
`hex"..."` rows. The whitening, the PHT, the 1-bit rotations and the
undoing of the last swap (C_i = R_(16,(i+2) mod 4) xor K_(i+4)) read as
section 4 writes them.

Measured with `orangec eval --stats` and `orangec test --stats`: a byte
through q0 or q1 costs about 90 steps and a GF(2^8) product about 264;
the MDS matrix costs 4,250 steps and an RS product of eight key bytes
8,588. h costs 5,637 steps for k = 2, 5,985 for k = 3 and 6,436 for
k = 4, three quarters of it the MDS matrix. The key schedule (forty h
evaluations and k RS products) costs 242,944, 267,544 and 292,818 steps
for 128-, 192- and 256-bit keys, and the sixteen rounds (thirty-two g
evaluations) about 180,000, 194,000 and 207,000 more. One block, key
schedule included, costs 423,369 to 423,727 steps with a 128-bit key,
461,199 to 462,185 with a 192-bit key and 499,725 to 500,555 with a
256-bit key, encryption and decryption alike, varying by a few hundred
steps with the data because each bit of a GF(2^8) multiplier takes one
branch or the other. The seven tests together use 3,194,487 steps. Not
expressed: constant-time behaviour (a lookup is a specification, not a
claim about leakage), any mode of operation, the paper's implementation
options (full, partial, minimal and zero keying), and any key length
other than the three of the paper (the paper pads shorter keys with
zeros to the next of them).

## Dissemination

### Files

- `twofish.or`: the complete cipher (q0 and q1 from their 4-bit tables,
  the GF(2^8) arithmetic, the MDS and RS matrices, h, g, the key schedule
  for 128-, 192- and 256-bit keys, encryption and decryption) and the
  seven tests below.

### Running

    orangec test algorithms/twofish/twofish.or
    python3 algorithms/verify.py algorithms/twofish

### Vectors

Each row is a `test` block in `twofish.or`, comparing a block with the
published value.

| Test | Source | Case |
| --- | --- | --- |
| `ecb_tbl KEYSIZE=128 I=1: encrypt` | Botan `src/tests/data/block/twofish.vec`, record 1, first block; submission `ecb_tbl.txt`, KEYSIZE=128, I=1 | zero 128-bit key, zero plaintext, ciphertext 9f589f5cf6122c32b6bfec2f2ae8c35a |
| `ecb_tbl KEYSIZE=128 I=3: encrypt` | Botan `twofish.vec`, record 2; `ecb_tbl.txt` KEYSIZE=128, I=3 | key 9f589f5cf6122c32b6bfec2f2ae8c35a, plaintext d491db16e7b1c39e86cb086b789f5419, ciphertext 019f9809de1711858faac3a3ba20fbc3 |
| `ecb_tbl KEYSIZE=128 I=3: decrypt` | the same record, decrypted | that ciphertext under that key gives the plaintext back |
| `ecb_tbl KEYSIZE=192 I=1: encrypt` | Botan `twofish.vec`, record 49, first block; `ecb_tbl.txt` KEYSIZE=192, I=1 | zero 192-bit key, zero plaintext, ciphertext efa71f788965bd4453f860178fc19101 |
| `ecb_tbl KEYSIZE=192 I=3: decrypt` | Botan `twofish.vec`, record 50; `ecb_tbl.txt` KEYSIZE=192, I=3, decrypted | key efa71f788965bd4453f860178fc19101 followed by eight zero bytes, ciphertext 39da69d6ba4997d585b6dc073ca341b2, plaintext 88b2b2706b105e36b446bb6d731a1e88 |
| `ecb_tbl KEYSIZE=256 I=1: encrypt` | Botan `twofish.vec`, record 97, first block; `ecb_tbl.txt` KEYSIZE=256, I=1 | zero 256-bit key, zero plaintext, ciphertext 57ff739d4dc92c1bd7fc01700cc8216f |
| `ecb_tbl KEYSIZE=256 I=3: decrypt` | Botan `twofish.vec`, record 98; `ecb_tbl.txt` KEYSIZE=256, I=3, decrypted | key 57ff739d4dc92c1bd7fc01700cc8216f followed by sixteen zero bytes, ciphertext 90afe91bb288544f2c32dc239b2635e6, plaintext d43bb7556ea32e46f2a282b7d45b4e0d |

Every expected value is copied from Botan's `twofish.vec`, which carries the
submission's `ecb_tbl` set (records 1 to 144, 48 per key size) followed by
its `ecb_vk` set and three long multi-block cases. Botan folds the first
two cases of each key size, which share the zero key, into one two-block
record, so record 1 is I=1 and I=2 of `ecb_tbl.txt` and record 2 is I=3;
the chained structure of the set (each plaintext is the previous
ciphertext, each key the ciphertext before that, padded with zeros) was
checked by script across the three key sizes. The decryption tests state
the record's plaintext as their expected value. A Python reference written
for this entry (see below) confirms every value; no library oracle exists
for Twofish among those available here.

### Provenance and claims

The paper could not be read from the machine this entry was written on, so
every constant was taken from fetched reference material and cross-checked
by script, never transcribed by eye or from memory:

- The 4-bit tables t0 to t3 of q0 and q1 came from Botan's `twofish.cpp`
  (current master, which builds Q0 and Q1 from them at compile time). A
  Python q written from the paper's construction (the a, b split, ROR4,
  8a mod 16, the two table stages) and those tables was checked equal, for
  all 256 inputs, to the expanded `Q0` and `Q1` tables of Botan 3.5.0's
  `twofish_tab.cpp`, an independent transcription. The eight packed
  `Word[64]` literals were generated by script from the tables and unpacked
  again to confirm them.
- The MDS matrix, its polynomial 0x169 and the RS matrix with its
  polynomial 0x14D came from Botan's comments and `RS32` constants; a Python
  GF(2^8) product with those polynomials was checked against Botan 3.5.0's
  expanded `MDS0` to `MDS3` tables (which fold the last fixed permutation
  of each column into the product) for all 256 inputs of each column, and
  the RS matrix against Botan's column-major `RS` bytes.
- A Python reference in the paper's vocabulary (q, h, g, F, RS, MDS, Me,
  Mo, S, K_j), built from those constants, reproduces all 723 records of
  `twofish.vec` on encryption, including the three 2048-byte multi-block
  cases, and 717 of them on decryption. The Orange specs were written from
  the paper's definitions and that reference; the seven vectors matched on
  the first evaluation. The identity of the two MDS coefficient products
  with the general GF(2^8) product was also checked for all 256 bytes
  before the shared `xtime` chain replaced `gf_mul` in `mds`.

The Python reference, the table and vector cross-checks, the literal
generator, the build script that assembled the three `.or` files of the
first form from one algorithm part, and the step-measurement drivers were
kept with the work record and are not part of the repository.

The entry was then rewritten in the current language, the three files of
the first form, which were split only by the step budget, folded into one.
Every expected value is carried over byte for byte from the first form,
where each was a `<name>_expected` spec of sixteen bytes: a script
evaluated the seven old specs and compared each with the `hex"..."`
literal of its test, and the keys, plaintexts and ciphertexts of the
inputs were compared the same way. The 4-bit tables, now one byte per
entry, were printed by script from the first form's packed `Word[64]`
literals and packed again to compare; the MDS and RS rows were compared
with the first form's matrices. The MDS matrix is again applied with the
general GF(2^8) product, whose identity with the first form's shared
`xtime` chain was checked when that chain was introduced. No vector was
added or dropped.

This entry is a reference evaluation of the Twofish specification under
`orangec test`. It makes no constant-time, side-channel, performance or
certification claim, and it is not a corpus entry in the sense of The
Orange Book chapter 12.

## Gaps

- There is no 4-bit word, and a nibble held in a byte ranges, for the
  checker, over all 256 values once it is bound to a name (even
  `let a0: Word[8] = x / 16`), so each lookup in a 4-bit table reduces its
  index mod 16 (`t[0][a1 % 16]`), a reduction that never changes a value
  here.
- Every index is checked in every instance of a sized spec, in branches
  that the instance never takes as well, so h cannot write the paper's
  "if k = 4" with `l[3]` in an instance where L has two words. The stages
  run in a loop over the words L_(k-1) to L_0 instead, and `stage` selects
  each stage's permutations by its index.
- GF(2^8) is a field of polynomials over GF(2), not of integers modulo a
  number, so `Mod[m]` does not express it; the products are written as
  shift and conditional xor over bytes.
