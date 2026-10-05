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

The Orange file follows the paper section by section. A block is
`Word[8]^16` read as four little-endian words; the result of the key
schedule is one `Word[32]^44`, the forty K_j followed by the S-box key
words in the order (S_(k-1), ..., S_0) that g takes them, padded to four; k
travels as an `Int` parameter and selects the stages of h.

| Paper section | Orange spec |
| --- | --- |
| 4, the little-endian words of a block | `load_le32`, `le_bytes`, `block_words`, `block_bytes` |
| 4, whitening and the sixteen rounds | `encrypt`, `round`; `decrypt`, `inverse_round` |
| 4.1, the function F with the PHT and the round subkeys | `f` |
| 4.2, the MDS matrix over GF(2^8) modulo v(x) | `v_polynomial`, `xtime`, `mds_products`, `mds` |
| 4.2 and 4.3.3, g(X) = h(X, S) | `g` |
| 4.3, the key words M_i, Me, Mo and the RS matrix modulo w(x) | `key_schedule_128`, `key_schedule_192`, `key_schedule_256`, `w_polynomial`, `gf_mul`, `rs` |
| 4.3.2, the function h | `h` |
| 4.3.4, the expanded key words K_j | `expanded_key_words`, `with_sbox_keys` |
| 4.3.5, q0 and q1 from the 4-bit tables t0 to t3 | `q0_tables`, `q1_tables`, `nibble_at`, `ror4`, `q`, `q0`, `q1` |

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

Two things in Twofish are data-dependent: the four nibble selections inside
each q permutation and the key-dependent xors around them. A 4-bit index may
select a table of 16 entries (`k & 15`). This rendering keeps each 4-bit
table as one `Word[64]` and `nibble_at` as a sixteen-arm conditional on the
nibble; a byte through q0 or q1 costs about 242 steps, where a table-driven
implementation spends one memory access. Nothing is precomputed: an implementation would expand the four
key-dependent S-boxes, once per key, into 1024 bytes or four 256-word
tables, whereas here g is written as the paper defines it, as h applied to
the S-box key words, and every S-box evaluation recomputes its two or
three fixed permutations and its key xors. That makes the rendering the
paper's definition rather than the paper's implementation notes, and it
makes the cost of a block scale with k, which the measurements below show.

The GF(2^8) arithmetic is written out: `xtime` takes the polynomial as a
parameter, `gf_mul` is an eight-step shift-and-add over `[a, b, product]`
and serves the RS matrix, whose twenty distinct coefficients make a general
product natural; the MDS matrix has two coefficients, EF and 5B, and their
products are written as the sums of a x^i over each coefficient's terms
from one chain of seven `xtime` calls, which is the same arithmetic at a
quarter of the cost. The two polynomials are named in `v_polynomial` and
`w_polynomial`. The whitening, the PHT, the 1-bit rotations and the
undoing of the last swap read exactly as section 4 writes them.

Measured under `orangec eval` by bisection with a filler spec: a byte
through q costs about 242 steps and a general GF(2^8) product 269; h costs
about 3,470 steps for k = 2, 4,400 for k = 3 and 5,340 for k = 4 (twelve,
sixteen or twenty q evaluations plus the MDS matrix), and an RS product of
eight key bytes about 8,600. One block, key schedule included, costs about
270,000 steps with a 128-bit key, 345,000 with a 192-bit key and 420,000
with a 256-bit key, encryption and decryption alike, varying by a few
percent with the data because the length of the sixteen-arm chain depends
on the nibble. A file's 1,048,576 steps therefore hold three 128-bit
blocks, two 192-bit blocks or two 256-bit blocks, which is how the vectors
are split: `twofish.or` uses about 815,000 steps, `twofish-192.or` about
697,000 and `twofish-256.or` about 849,000, and none has room for another
block of its size. Not expressed: constant-time behaviour (the selection
idiom is a specification of a lookup, not a claim about leakage), any mode
of operation, the paper's implementation options (full, partial, minimal
and zero keying), and any key size other than the three of the paper.

## Dissemination

### Files

- `twofish.or`: the complete cipher (q0 and q1 from their 4-bit tables, the
  GF(2^8) arithmetic, the MDS and RS matrices, h, g, the key schedule for
  128-, 192- and 256-bit keys, encryption and decryption) with three vector
  pairs for 128-bit keys.
- `twofish-192.or`: the same algorithm text with two vector pairs for
  192-bit keys.
- `twofish-256.or`: the same algorithm text with two vector pairs for
  256-bit keys.

The three files are one file split by the step budget; their algorithm part
is identical, and only the header's last line and the vector specs differ.

### Running

```console
orangec eval algorithms/twofish/twofish.or
orangec eval algorithms/twofish/twofish-192.or
orangec eval algorithms/twofish/twofish-256.or
python3 algorithms/verify.py algorithms/twofish
```

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `ecb_tbl_128_i1` | Botan `src/tests/data/block/twofish.vec`, record 1, first block; submission `ecb_tbl.txt`, KEYSIZE=128, I=1 | zero 128-bit key, zero plaintext, ciphertext 9f589f5cf6122c32b6bfec2f2ae8c35a |
| `ecb_tbl_128_i3` | Botan `twofish.vec`, record 2; `ecb_tbl.txt` KEYSIZE=128, I=3 | key 9f589f5cf6122c32b6bfec2f2ae8c35a, plaintext d491db16e7b1c39e86cb086b789f5419, ciphertext 019f9809de1711858faac3a3ba20fbc3 |
| `ecb_tbl_128_i3_decrypt` | the same record, decrypted | that ciphertext under that key gives the plaintext back |
| `ecb_tbl_192_i1` | Botan `twofish.vec`, record 49, first block; `ecb_tbl.txt` KEYSIZE=192, I=1 | zero 192-bit key, zero plaintext, ciphertext efa71f788965bd4453f860178fc19101 |
| `ecb_tbl_192_i3_decrypt` | Botan `twofish.vec`, record 50; `ecb_tbl.txt` KEYSIZE=192, I=3, decrypted | key efa71f788965bd4453f860178fc19101 followed by eight zero bytes, ciphertext 39da69d6ba4997d585b6dc073ca341b2, plaintext 88b2b2706b105e36b446bb6d731a1e88 |
| `ecb_tbl_256_i1` | Botan `twofish.vec`, record 97, first block; `ecb_tbl.txt` KEYSIZE=256, I=1 | zero 256-bit key, zero plaintext, ciphertext 57ff739d4dc92c1bd7fc01700cc8216f |
| `ecb_tbl_256_i3_decrypt` | Botan `twofish.vec`, record 98; `ecb_tbl.txt` KEYSIZE=256, I=3, decrypted | key 57ff739d4dc92c1bd7fc01700cc8216f followed by sixteen zero bytes, ciphertext 90afe91bb288544f2c32dc239b2635e6, plaintext d43bb7556ea32e46f2a282b7d45b4e0d |

Every expected value is copied from Botan's `twofish.vec`, which carries the
submission's `ecb_tbl` set (records 1 to 144, 48 per key size) followed by
its `ecb_vk` set and three long multi-block cases. Botan folds the first
two cases of each key size, which share the zero key, into one two-block
record, so record 1 is I=1 and I=2 of `ecb_tbl.txt` and record 2 is I=3;
the chained structure of the set (each plaintext is the previous
ciphertext, each key the ciphertext before that, padded with zeros) was
checked by script across the three key sizes. The decryption pairs state
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
generator, the build script that assembles the three `.or` files from one
algorithm part, and the step-measurement drivers were kept with the work
record and are not part of the repository.

This entry is a reference evaluation of the Twofish specification under
`orangec eval`. It makes no constant-time, side-channel, performance or
certification claim, and it is not a corpus entry in the sense of The
Orange Book chapter 12.

## Gaps

None that prevented anything. Three features of the language shaped the
files:

- The step budget of 1,048,576 steps per file holds three 128-bit blocks or
  two 192- or 256-bit blocks when the S-boxes are computed as the paper
  defines them, so the seven vectors are three files with an identical
  algorithm part instead of one; a third 192-bit block would have exceeded
  the budget by a margin smaller than the data-dependent variation, and
  was not attempted.
- A 4-bit index may select a table of 16 entries. Each nibble is still
  selected by a sixteen-arm conditional (about 35 steps) and a byte through
  q costs about 242 steps. An `Int` parameter is not an index, and a
  `Word[32]` runs past a table of 44, so `f` and `round` receive the two
  round subkeys as parameters and the loops in `encrypt` and `decrypt`
  index the expanded key with the loop variable.
- A spec may return a tuple. The key schedule's forty subkeys and the
  S-box key words still travel in one flat `Word[32]^44` whose layout the
  comments state, and h's list L is padded to four words with k passed
  beside it.
