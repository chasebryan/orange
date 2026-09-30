# Ascon-AEAD128

Ascon is the family of lightweight authenticated ciphers and hash functions
designed by Christoph Dobraunig, Maria Eichlseder, Florian Mendel and Martin
Schläffer (Graz University of Technology, Infineon and Radboud University),
submitted to the CAESAR competition in 2014 and to the NIST lightweight
cryptography project in 2019. NIST standardized it in
[SP 800-232](https://doi.org/10.6028/NIST.SP.800-232), "Ascon-Based
Lightweight Cryptography Standards for Constrained Devices" (final, August
2025), as Ascon-AEAD128, Ascon-Hash256, Ascon-XOF128 and Ascon-CXOF128, all
built on one 320-bit permutation. Ascon-AEAD128 is the parameter set the
submission called Ascon-128a (a 128-bit rate, 12 rounds in initialization
and finalization, 8 between blocks), with two changes NIST made in the
standard: words are loaded little-endian and the initial value is new. It
is meant for microcontrollers, RFID tags, sensors and other devices where
AES-GCM is too heavy, and it is a current NIST standard: the first
lightweight authenticated cipher NIST has standardized. This entry writes
the permutation, Ascon-AEAD128 encryption and decryption, and Ascon-Hash256
in Orange, and reproduces fourteen entries of the designers' known-answer
files together with one decryption and one rejected tag.

## Analysis

### Structure

The state S is 320 bits held as five 64-bit words S0 through S4. SP 800-232
section 3 defines the permutation Ascon-p[rnd] as rnd rounds, each the
composition of three layers: the constant-addition layer p_C xors an 8-bit
round constant into S2; the substitution layer p_S applies one 5-bit S-box
to each of the 64 columns of the state, which the standard writes as a
short bitsliced sequence of xor, and and not on the five words; and the
linear diffusion layer p_L replaces each word S_i by
S_i xor (S_i >>> r1) xor (S_i >>> r2) with the rotation pairs (19, 28),
(61, 39), (1, 6), (10, 17) and (7, 41). The round constants are
c_i = ((0xf - i) << 4) | i; the standard lists sixteen of them so that up
to sixteen rounds are defined, and Ascon-p[12] and Ascon-p[8] use the last
twelve and the last eight.

Ascon-AEAD128 (section 4) is a duplex construction over that permutation
with a 128-bit rate (S0 and S1) and a 192-bit capacity. Initialization
loads IV || K || N, applies Ascon-p[12] and xors the key into S3 and S4.
Associated data, when there is any, is padded with a single 1 bit and zeros
to a multiple of the rate and absorbed a block at a time, each block
followed by Ascon-p[8]; then one bit of domain separation is xored into the
last bit of the state. The plaintext is padded the same way and each block
P_i gives C_i = S_r xor P_i, which also becomes the new S_r; Ascon-p[8]
follows every block but the last. Finalization xors the key into S2 and S3,
applies Ascon-p[12], and the tag is the last 128 bits of the state xored
with the key. Decryption runs the same duplex with C_i replacing S_r and
recomputes the tag, and the plaintext is released only when the tags agree.
Ascon-Hash256 (section 5) is the plain sponge: the initial value followed
by zeros, Ascon-p[12], 64-bit blocks of the padded message absorbed into
S0 with Ascon-p[12] after each, and four 64-bit words of S0 squeezed out
with Ascon-p[12] between them.

| Standard section | Orange spec |
| --- | --- |
| 2, bytes to words (little-endian) | `load_le64`, `le_bytes64` |
| 2, padding with a 1 bit and zeros | `pad`, `blocks` |
| 3.1, p_C and the round constants | `constant_addition`, `round_constants` |
| 3.2, p_S, the bitsliced S-box | `substitution_layer` |
| 3.3, p_L, the rotation pairs | `linear_diffusion_layer` |
| 3, Ascon-p[12] and Ascon-p[8] | `round`, `ascon_p12`, `ascon_p8` |
| 4, the initial value | `iv` |
| 4, initialization | `initialization` |
| 4, associated data and domain separation | `process_associated_data` |
| 4, plaintext | `process_plaintext`, `plaintext_block`, `ciphertext_bytes` |
| 4, finalization and the tag | `finalization` |
| 4, Algorithm 1, encryption | `ascon_aead128_encrypt`, `ciphertext_and_tag` |
| 4, Algorithm 2, decryption | `plaintext_bytes`, `keystream_block`, `ascon_aead128_verify`, `ascon_aead128_decrypt` |
| 5, Algorithm 3, Ascon-Hash256 | `ascon_hash256`, `squeeze` (in `ascon-hash256.or`) |

Two conventions of SP 800-232 differ from the submission and decide every
byte of the vectors. Bytes are loaded into a word little-endian, so the
first byte of the key is the least significant byte of S1, the padding
byte 0x01 sits at byte position n of the block, and the domain-separation
bit is the most significant bit of S4 (the last bit of the state); the
original Ascon loaded big-endian. And the initial value is
0x00001000808C0001 for Ascon-AEAD128 and 0x0000080100CC0002 for
Ascon-Hash256, encodings of the algorithm identifier, the round numbers,
the tag or digest length and the rate that replace the submission's
0x80400c0600000000 and 0x00400c0000000100. The reference implementation's
README (ascon-c, the designers' repository) states both changes.

### Security status

Ascon was selected in February 2019 as the first choice of the CAESAR
committee's final portfolio for the lightweight use case, and on 7 February
2023 NIST announced it as the winner of its lightweight cryptography
standardization process, which had run since 2018 with 57 first-round
candidates and 10 finalists. The initial public draft of SP 800-232
appeared in November 2024 and the final standard in August 2025.

The design is a duplex sponge with a keyed initialization and a keyed
finalization: the key enters the state at both ends, so that recovering the
inner state during processing does not by itself give the key or let an
adversary forge freely. The standard claims 128 bits of security for
confidentiality and integrity in the nonce-respecting setting, with a 128-bit
key, a 128-bit nonce and a 128-bit tag, under a bound on the data processed
under one key (the standard's stated limit, 2^54 bytes, was not checked from
this machine). The claim has two caveats a reader should keep. First, the
nonce must never repeat under one key: with a repeated nonce the keystream of
the first blocks repeats, and Baudrin, Canteaut and Perrin (ToSC 2022) gave a
practical cube attack on the 6-round initialization in the nonce-misuse
setting; the designers make no claim under nonce misuse. Second, decryption
produces plaintext blocks before the tag can be checked, and Ascon makes no
claim when unverified plaintext is released; the standard requires an
implementation to withhold it, which the Orange `ascon_aead128_decrypt`
does by construction.

The cryptanalytic record is long and stable. The designers' own
"Cryptanalysis of Ascon" (Dobraunig, Eichlseder, Mendel and Schläffer,
CT-RSA 2015) gave cube-like key recovery on 5 of the 12 initialization
rounds in 2^35 and 6 rounds in 2^66, and differential-linear and truncated
differential distinguishers on 4 and 5 rounds. Li, Dong and Wang (ToSC 2017)
extended conditional cube attacks to 7 initialization rounds at about
2^104 (2^77 for a class of weak keys), and Rohit, Hu, Sarkar and Sun
(ToSC 2021) gave misuse-free 7-round key recovery at 2^123, with further
weak-key results by Rohit and Sarkar (ToSC 2021). Forgery attacks on the
finalization reach 4 of its 12 rounds (Dobraunig et al. 2015; Gerault,
Peyrin and Tan, ToSC 2021, with differential-based forgeries). On the
permutation alone, zero-sum and integral distinguishers reach all 12 rounds
at complexities near 2^130, which is above the security claim and does not
translate into an attack on the modes. For the hash functions, collision
and preimage attacks stop at 2 to 4 rounds (Zong, Dong and Wang, 2019, on
Ascon-Hash and Ascon-Xof; Qin, Dong, Wang, Jia and Liu, EUROCRYPT 2023,
with meet-in-the-middle preimages on 4-round Ascon-XOF). Newer
differential-linear and cube results of 2022 to 2024 refine the
complexities at 5 to 7 rounds without adding a round; the years and venues
of those refinements were not checked from this machine. As of September
2026 the best key recovery reaches 7 of the 12 initialization rounds and no
attack touches the 8-round processing or the full permutation as used,
which leaves a margin of 5 rounds at each end.

The changes from the submission are the ones stated above: little-endian
words, new initial values, and the choice of the Ascon-128a parameters
(rate 128, 8 rounds between blocks) rather than the submission's primary
recommendation Ascon-128 (rate 64, 6 rounds), which is why a vector of the
2016 Ascon-128a or the original Ascon-128 does not match this file. Ascon-80pq
and Ascon-Hasha and Ascon-Xofa were not standardized. Status: NIST standard
since August 2025 for constrained devices, with SP 800-232 recommending it
where AES-GCM (SP 800-38D) or ChaCha20-Poly1305 are too costly; adoption in
protocols is beginning and no deprecation applies.

### What the Orange rendering shows

Nothing in the permutation depends on data: no branch, table or rotation
amount varies with the state, and the round is four short specs. The S-box
is the standard's bitsliced sequence, so a reader sees the five `~a & b`
terms that make it a chi-like map and the four xors around them rather
than a 32-entry table; the linear layer is ten rotations written out,
because a rotation amount must be a literal. `Word[64]^5` carries the state
through the loops of `ascon_p12` and `ascon_p8`, and the two permutations
differ only in the loop bounds `0..12` and `4..12` over the same constants.

The mode is where the language shows. Lengths are values and indices are
static, so a message of n bytes travels in a 32-byte array with n beside
it; `pad` places the 0x01 byte by comparing the loop index with n, and the
block loops run over all three possible blocks and absorb block i when
i <= n / 16, with `i < n / 16` deciding whether Ascon-p[8] follows. The
accumulator of the plaintext loop is `Word[64]^11`: the state in five words
and the three ciphertext blocks in six, because a loop carries one value.
Decryption is written as the standard describes it, S_r replaced by the
ciphertext block, and its last block is handled through the observation
the standard's Algorithm 2 rests on: after P~ is recovered, the state is
S_r xor pad(P~), the state encryption reaches, so `ascon_aead128_verify`
recovers P with `plaintext_bytes` and then recomputes the tag through the
encryption path. The C || T output puts the tag at byte position n, again
by comparison. Ascon-Hash256 needs the same devices at rate 8 and carries
its own copy of the permutation, since a module has no imports.

Measured under `orangec eval`, one round costs about 140 steps, Ascon-p[12]
about 1,760 and Ascon-p[8] about 1,180. One encryption of the empty message
with no associated data costs about 13,000 steps, of which the two
12-round permutations are 3,500 and the padding and output loops the rest;
32 bytes of each costs about 27,000; a verification about 28,000 and a
decryption about 31,000. Ascon-Hash256 of 0, 32 and 64 bytes costs about
16,000, 26,000 and 36,000. The twelve pairs of `ascon.or` use about
250,000 of the 1,048,576-step budget and the five of `ascon-hash256.or`
about 120,000; nothing was dropped or moved for budget reasons.

## Dissemination

### Files

- `ascon.or`: the permutation, Ascon-AEAD128 encryption, verification and
  decryption, and twelve vector pairs (nine encryptions, one decryption, one
  verdict and one rejected tag).
- `ascon-hash256.or`: the permutation again and Ascon-Hash256, with five
  vector pairs. It is a second file because it is a second algorithm of the
  standard, not because of the budget; both files would fit in one.

### Running

```console
orangec eval algorithms/ascon/ascon.or
orangec eval algorithms/ascon/ascon-hash256.or
python3 algorithms/verify.py algorithms/ascon
```

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `kat_count_1` | ascon-c `LWC_AEAD_KAT_128_128.txt`, Count 1 | empty plaintext, empty associated data: the tag alone |
| `kat_count_2` | same file, Count 2 | empty plaintext, 1 byte of associated data |
| `kat_count_17` | same file, Count 17 | empty plaintext, 16 bytes of associated data, one whole block |
| `kat_count_33` | same file, Count 33 | empty plaintext, 32 bytes of associated data, two blocks and a padding block |
| `kat_count_34` | same file, Count 34 | 1-byte plaintext, empty associated data |
| `kat_count_273` | same file, Count 273 | 8 bytes of each, the half-block boundary of the reference code |
| `kat_count_545` | same file, Count 545 | 16 bytes of each, whole blocks |
| `kat_count_767` | same file, Count 767 | 23-byte plaintext, 7 bytes of associated data, partial blocks |
| `kat_count_1089` | same file, Count 1089 | 32 bytes of each, the largest entry: C (32 bytes) and T |
| `kat_count_1089_decrypt` | same file, Count 1089, PT field | decryption of its CT under its Key, Nonce and AD |
| `kat_count_1089_verify` | same file, Count 1089 | the verdict of Algorithm 2: true |
| `kat_count_1089_altered_tag_verify` | by definition of Algorithm 2; the Python reference agrees | Count 1089 with the last tag byte 0xaa changed to 0xab: false |
| `kat_count_1` (hash) | ascon-c `LWC_HASH_KAT_128_256.txt`, Count 1 | Ascon-Hash256 of the empty message |
| `kat_count_2` (hash) | same file, Count 2 | one byte, 00 |
| `kat_count_9` (hash) | same file, Count 9 | 8 bytes, one whole block |
| `kat_count_33` (hash) | same file, Count 33 | 32 bytes, four blocks |
| `kat_count_65` (hash) | same file, Count 65 | 64 bytes, eight blocks |

The known-answer files are the designers' own, generated by
`tests/genkat_aead.c` and `tests/genkat_hash.c` of the ascon-c repository
for every plaintext and associated-data length from 0 to 32 bytes (AEAD,
Count = 33 plen + adlen + 1) and every message length from 0 to 1024 bytes
(hash, Count = n + 1), with the key 00 01 ... 0f, the nonce 10 11 ... 1f,
the plaintext 20 21 ..., the associated data 30 31 ... and the hash message
00 01 .... SP 800-232 publishes no example values in its text; NIST's
ACVP vectors were not reachable from this machine.

### Provenance and claims

The round constants, the S-box sequence, the rotation amounts, the two
initial values, the padding and the domain-separation bit were taken from
the reference implementation of SP 800-232 in the designers' ascon-c
repository (`crypto_aead/asconaead128/ref/aead.c`, `round.h`,
`permutations.h`, `word.h`, `constants.h` and
`crypto_hash/asconhash256/ref/hash.c`, fetched from
raw.githubusercontent.com), whose README states that it implements
NIST SP 800-232 and names the endianness and initial-value changes from the
submission. The initial values were computed with a script from the field
encoding of `constants.h`. A plain Python Ascon-AEAD128 and Ascon-Hash256
written for this entry from that C code (`ascon_ref.py` in the worker's
scratch directory, not in the repository) was checked against all 1,089
entries of `LWC_AEAD_KAT_128_128.txt` in both directions, against a
flipped tag bit for each entry, and against all 1,025 entries of
`LWC_HASH_KAT_128_256.txt` before it was used; a second script selected
the entries above, checked each entry's inputs against the generator's
pattern and its output against the Python reference, and printed the
`_expected` literals. No `_expected` value was typed by hand or adjusted.
The section numbers of SP 800-232 named in the source comments (2 for
notation and padding, 3.1 through 3.3 for the layers of the permutation,
4 for Ascon-AEAD128 with Algorithms 1 and 2, 5 for Ascon-Hash256 with
Algorithm 3) are from the worker's knowledge of the publication, whose
text was not reachable from this machine.

This entry is a reference evaluation of a specification under `orangec
eval`. It makes no constant-time, side-channel, performance or certification
claim, and it is not a corpus entry in the sense of The Orange Book
chapter 12.

## Gaps

- Indices must be static and an array's length is a type, so a message
  length cannot parametrize the array: every plaintext, ciphertext and
  associated-data input is a 32-byte array with its length beside it (64
  bytes for the hash), and the padding byte, the number of blocks and the
  position of the tag in C || T are found by comparing loop indices with the
  length. The cost is about 2,800 steps per padding and 6,600 per C || T
  assembly, against 3,500 for the two 12-round permutations. Inputs longer
  than 32 bytes need the array sizes and the block loops' bounds changed.
- A loop carries one accumulator, so the plaintext loop carries the state
  and the ciphertext blocks together in `Word[64]^11` and the hash squeeze
  the state and the digest words in `Word[64]^9`.
- A loop's step is one expression without `let`, so the per-block work is a
  helper spec (`plaintext_block`, `keystream_block`, `squeeze`) called from
  the loop with the recorded words already in place.
- Ascon-XOF128 and Ascon-CXOF128 are not written; each is the sponge of
  `ascon-hash256.or` with another initial value, a variable-length squeeze
  and, for CXOF, a customization string absorbed first.
- The step budget was not reached: about 250,000 and 120,000 steps of
  1,048,576 in the two files.
