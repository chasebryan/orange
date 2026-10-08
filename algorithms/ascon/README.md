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

The permutation is the module `permutation` (`permutation.or`), which both
algorithms read with `use permutation;`. The state is `Word[64]^5` in all
three files, named `State` in each, since a type name does not cross
modules. Ascon-p[rnd] is one spec with the round number as a size,
`ascon_p[rnd]`, called as `ascon_p[12]` and `ascon_p[8]`. Byte strings
(key, nonce, associated data, plaintext, ciphertext, tag, message, digest)
are `Word[8]^n` whose length is their type.

| Standard section | Orange spec |
| --- | --- |
| 2, bytes to words (little-endian) | `as little Word[64]^2` and `as little Word[8]^16` in `add_to_rate`, `rate`, `initialization` and `finalization`; `as little` in `hash` |
| 2, padding with a 1 bit and zeros | `padding[len]` and `pad[len]` (`ascon.or`, 16-byte rate); `padding[len]` (`ascon-hash256.or`, 8-byte rate) |
| 3, the state | `State` (`Word[64]^5`) |
| 3.1, p_C and the round constants | `constant_addition`, `round_constants` (`permutation.or`) |
| 3.2, p_S, the bitsliced S-box | `substitution_layer` (`permutation.or`) |
| 3.3, p_L, the rotation pairs | `linear_diffusion_layer` (`permutation.or`) |
| 3, Ascon-p[rnd] | `round`, `ascon_p[rnd]` (`permutation.or`) |
| 4, the initial value | `iv` |
| 4, the rate S_r | `add_to_rate`, `rate` |
| 4, initialization | `initialization` |
| 4, associated data and domain separation | `associated_data[b]`, `domain_separation` |
| 4, plaintext | `plaintext[b]` |
| 4, finalization and the tag | `finalization` |
| 4, Algorithm 1, encryption | `encrypt[A, len]`, `encrypt_empty_a[len]`, `encrypt_empty_p[A]`, `encrypt_empty` |
| 4, Algorithm 2, decryption | `ciphertext[b]`, `plaintext_bytes[A, len]`, `verify[A, len]`, `decrypt[A, len]` |
| 5, Algorithm 3, Ascon-Hash256 | `iv`, `hash[b]`, `hash256[len]` (`ascon-hash256.or`) |

Two conventions of SP 800-232 differ from the submission and decide every
byte of the vectors. Bytes are loaded into a word little-endian, so the
first byte of the key is the least significant byte of S1, the padding
byte 0x01 sits at byte position n of the block, and the domain-separation
bit is the most significant bit of S4 (the last bit of the state); the
original Ascon loaded big-endian. And the initial value is
0x00001000808C0001 for Ascon-AEAD128 and 0x0000080100CC0002 for
Ascon-Hash256, encodings of the algorithm identifier, the round numbers,
the tag or digest length and the rate that replace the submission's
0x80800c0800000000 and 0x00400c0000000100. The reference implementation's
README (ascon-c, the designers' repository) states both changes.

### Security status

Ascon was selected in February 2019 as the first choice of the CAESAR
committee's final portfolio for the lightweight use case, and on 7 February
2023 NIST announced it as the winner of its lightweight cryptography
standardization process, which had run since 2018 with 57 submissions
and 10 finalists. The initial public draft of SP 800-232
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
practical cube attack in the nonce-misuse setting on the 6-round
permutation that Ascon-128 applies between data blocks (Ascon-AEAD128
applies 8); the designers make no claim under nonce misuse. Second, decryption
produces plaintext blocks before the tag can be checked, and Ascon makes no
claim when unverified plaintext is released; the standard requires an
implementation to withhold it, which the Orange `decrypt` does by
construction.

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
than a 32-entry table; the linear layer is the five Sigma functions with
their ten rotation amounts written out as the standard lists them. The
round constants are one `hex"..."` row, and `ascon_p[rnd]` runs the loop
`(12 - rnd)..12` over it, so Ascon-p[12] and Ascon-p[8] are visibly the same
rounds with different first constants.

Byte order is written where the standard fixes it: `as little Word[64]^2`
turns a 16-byte block, the key or the nonce into two words, the first byte
least significant; `[iv()] ++ ((k ++ n) as little Word[64]^4)` is
IV || K || N; and `as little Word[8]^16` turns the rate or the tag words
back into bytes. The domain separation is the literal
`0x8000000000000000` on S4, the last bit of the state in that order.

Lengths are sizes. `padding[len]()` is what section 2 appends to a string
of `len` bytes, the byte 0x01 opening a fill of zeros, and the padded
string is `x ++ padding[len]()`; the empty string has no array, and its
padded form is `padding[0]()` alone. The block loops take the padded string
as `Word[8]^(16 * b)` and slice block i out as `x[16 * i..16 * i + 16]`;
`i < (b - 1)` decides whether Ascon-p[8] follows. Encryption returns
C || T as `Word[8]^(len + 16)`: the ciphertext blocks are produced whole,
and `[..len]` cuts the last one to C~. The standard's |A| > 0 and |P| > 0
cases become four specs, `encrypt`, `encrypt_empty_a`, `encrypt_empty_p`
and `encrypt_empty`, because an empty A or P is not a value; each test
calls the one its lengths need, and the KAT file's PT, AD and CT appear in
it as hex. The associated data is a type parameter `A` listed at the
vectors' lengths (1, 7, 8, 16 and 32 bytes) and the plaintext a size from 1
to 32 bytes, so that `encrypt` has 160 instances rather than 1,024.

Decryption is written as the standard describes it, S_r replaced by the
ciphertext block, and `ciphertext[b]` yields P from the padded C. The last
block is cut to |C| mod 16 bytes, a length the slice bounds cannot compute,
so the state after it is rebuilt through the observation the standard's
Algorithm 2 rests on: S_r <- C_i is S_r <- S_r xor P_i, and after P~ the
state is S_r xor pad(P~), the state encryption reaches; `verify` encrypts
the recovered P and compares the last sixteen bytes with T, whole, with
`==`. `decrypt` returns P only when `verify` holds.

Ascon-Hash256 reads the same permutation module. Its padding is at the
8-byte rate, `hash[b]` absorbs b blocks with `as little Word[64]` and
squeezes four words with a tuple accumulator (the state and the digest
words), and `hash256[len]` pads a message of 1 to 64 bytes; the empty
message is `hash(padding[0]())`.

Measured with `orangec test --stats`: one round costs about 140 steps,
Ascon-p[12] about 1,690 and Ascon-p[8] about 1,130 (including the call and
the constants). Initialization and finalization cost about 1,720 steps each,
and each further block of associated data or plaintext about 1,170 to
1,195. The encryption of the empty message with no associated data costs
3,532 steps, and of 32 bytes of each (Count 1089) 9,427. A verification
costs 17,130, since it recovers P and then encrypts it, and a decryption
24,824, the verification and the recovery of P once more. Ascon-Hash256
costs 8,547 steps for the empty message and about 1,707 for each further
8-byte block (22,211 for 64 bytes). The twelve tests of `ascon.or` use
110,888 steps and the five of `ascon-hash256.or` 64,947.

## Dissemination

### Files

- `permutation.or`: the module `permutation`, Ascon-p[rnd] of section 3,
  read by both other files. It carries no vectors of its own.
- `ascon.or`: Ascon-AEAD128 encryption, verification and decryption, and
  twelve tests (nine encryptions, one decryption, one verdict and one
  rejected tag).
- `ascon-hash256.or`: Ascon-Hash256, with five tests. It is a second file
  because it is a second algorithm of the standard.

### Running

```console
orangec test algorithms/ascon/ascon.or
orangec test algorithms/ascon/ascon-hash256.or
python3 algorithms/verify.py algorithms/ascon
```

`orangec eval` prints every parameterless spec, and each instance of
`padding` is one: 33 padding strings in `ascon.or` and 65 in
`ascon-hash256.or`.

### Vectors

Each row is a `test` block. The AEAD tests are in `ascon.or`, the hash tests
in `ascon-hash256.or`.

| Test | Source | Case |
| --- | --- | --- |
| `LWC_AEAD_KAT_128_128 Count 1: empty P and A` | ascon-c `LWC_AEAD_KAT_128_128.txt`, Count 1 | empty plaintext, empty associated data: the tag alone |
| `LWC_AEAD_KAT_128_128 Count 2: empty P, 1-byte A` | same file, Count 2 | empty plaintext, 1 byte of associated data |
| `LWC_AEAD_KAT_128_128 Count 17: empty P, 16-byte A` | same file, Count 17 | empty plaintext, 16 bytes of associated data, one whole block |
| `LWC_AEAD_KAT_128_128 Count 33: empty P, 32-byte A` | same file, Count 33 | empty plaintext, 32 bytes of associated data, two blocks and a padding block |
| `LWC_AEAD_KAT_128_128 Count 34: 1-byte P, empty A` | same file, Count 34 | 1-byte plaintext, empty associated data |
| `LWC_AEAD_KAT_128_128 Count 273: 8-byte P, 8-byte A` | same file, Count 273 | 8 bytes of each, the half-block boundary of the reference code |
| `LWC_AEAD_KAT_128_128 Count 545: 16-byte P, 16-byte A` | same file, Count 545 | 16 bytes of each, whole blocks |
| `LWC_AEAD_KAT_128_128 Count 767: 23-byte P, 7-byte A` | same file, Count 767 | 23-byte plaintext, 7 bytes of associated data, partial blocks |
| `LWC_AEAD_KAT_128_128 Count 1089: 32-byte P, 32-byte A` | same file, Count 1089 | 32 bytes of each, the largest entry: C (32 bytes) and T |
| `LWC_AEAD_KAT_128_128 Count 1089: decryption returns P` | same file, Count 1089, PT field | decryption of its CT under its Key, Nonce and AD |
| `LWC_AEAD_KAT_128_128 Count 1089: the tag verifies` | same file, Count 1089 | the verdict of Algorithm 2: true |
| `LWC_AEAD_KAT_128_128 Count 1089 with the last tag byte changed: rejected` | by definition of Algorithm 2; the Python reference agrees | Count 1089 with the last tag byte 0xaa changed to 0xab: false |
| `LWC_HASH_KAT_128_256 Count 1: the empty message` | ascon-c `LWC_HASH_KAT_128_256.txt`, Count 1 | Ascon-Hash256 of the empty message |
| `LWC_HASH_KAT_128_256 Count 2: the one-byte message 00` | same file, Count 2 | one byte, 00 |
| `LWC_HASH_KAT_128_256 Count 9: 8 bytes, one whole block` | same file, Count 9 | 8 bytes, one whole block |
| `LWC_HASH_KAT_128_256 Count 33: 32 bytes, four blocks` | same file, Count 33 | 32 bytes, four blocks |
| `LWC_HASH_KAT_128_256 Count 65: 64 bytes, eight blocks` | same file, Count 65 | 64 bytes, eight blocks |

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

The entry was then rewritten in the current language. Every expected value
is carried over byte for byte from the first form, where each was a
`<name>_expected` spec of byte or Bool literals: the new tests state the
same bytes as `hex"..."` literals, compared with the old values by a
script, and the two verdicts as `verify(...)` and `!verify(...)`; no
vector was added or dropped. The inputs that the first form passed as
32-byte and 64-byte arrays with a length beside them (the generator's
plaintext, associated data and hash message) are now each test's own
`hex"..."` PT, AD or message, the prefix of the generator's pattern of the
test's length. The permutation, which the first form carried twice, is now
one module read by both files, and its constants are unchanged.

This entry is a reference evaluation of a specification under `orangec
test`. It makes no constant-time, side-channel, performance or certification
claim, and it is not a corpus entry in the sense of The Orange Book
chapter 12.

## Gaps

- No array has zero elements, so the empty string is not a value: an empty
  associated data or plaintext is a spec of its own (`encrypt_empty_a`,
  `encrypt_empty_p`, `encrypt_empty`), and the hash of the empty message is
  `hash(padding[0]())`, the padded form alone. Decryption with an empty
  ciphertext is not written; no vector here needs it.
- A spec has at most 256 instances, counting every combination of its sizes
  and types, so `encrypt`, `verify` and `decrypt` cannot take every length
  of both inputs (32 times 32 would be 1,024): the plaintext and ciphertext
  are any of 1 to 32 bytes, and the associated data is one of the listed
  lengths 1, 7, 8, 16 and 32. Another length is one more entry in the list;
  longer inputs need the size ranges widened. Ascon-Hash256 takes messages
  of 1 to 64 bytes.
- Slice bounds cannot use `/` or `%`, so the last partial block, |P| mod 16
  bytes, cannot be sliced out of an input by its length: the block loops run
  over the whole padded string and the result is cut with `[..len]`, and
  decryption rebuilds the state after the last block by encrypting the
  recovered plaintext, which doubles its cost.
- A spec has one result type and Orange has no value for fail, so `decrypt`
  returns all zeros when the tag does not verify, and `verify` gives the
  verdict.
- Ascon-XOF128 and Ascon-CXOF128 are not written; each is the sponge of
  `ascon-hash256.or` with another initial value, a variable-length squeeze
  and, for CXOF, a customization string absorbed first.
