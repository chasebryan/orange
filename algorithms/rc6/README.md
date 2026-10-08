# RC6

RC6 is a 128-bit block cipher with 128-, 192- and 256-bit keys, designed by
Ronald Rivest, Matt Robshaw, Ray Sidney and Yiqun Lisa Yin at RSA
Laboratories and described in
[The RC6 Block Cipher](https://people.csail.mit.edu/rivest/pubs/RRSY98.pdf)
(version 1.1, August 1998), the submission that carried it into NIST's
Advanced Encryption Standard process, where it was one of the five
finalists chosen for the second round in August 1999, from which Rijndael
was selected in October 2000. It is the direct descendant of RC5 (Rivest,
1994): the same data-dependent rotations and the same key schedule, with
four 32-bit registers instead of two and an integer multiplication, the
quadratic function f(x) = x(2x + 1), driving the rotation amounts. The
cipher is parameterized as RC6-w/r/b; the AES version, and the one this
entry carries, is RC6-32/20/b with b = 16, 24 or 32. RC6 was never
standardized and is rarely deployed, though Crypto++, Bouncy Castle and
other libraries ship it; it is unbroken, with the best published attacks
stopping at about three quarters of its rounds, and it matters today as the
finalist that made the data-dependent rotation a design element of the AES
generation.

## Analysis

### Structure

The block is four 32-bit words A, B, C, D, read little-endian from the
sixteen bytes. Encryption adds S[0] to B and S[1] to D, runs r = 20 rounds,
and adds S[2r + 2] to A and S[2r + 3] to C. A round computes
t = f(B) and u = f(D), replaces A by ((A xor t) <<< u) + S[2i] and C by
((C xor u) <<< t) + S[2i + 1], and rotates the registers to (B, C, D, A).
The quadratic function f(x) = (x(2x + 1)) <<< 5 is a permutation of the
32-bit words whose high-order bits depend on all the bits of x; rotating it
left by lg w = 5 moves those bits into the low five positions, which are
what the data-dependent rotations read. Decryption runs the same
operations backwards with subtraction and right rotation, and the paper's
key schedule is RC5's: the key bytes become c = b/4 little-endian words
L[0..c-1], S[0..2r+3] is filled with the arithmetic progression
P32 = b7e15163, P32 + Q32, P32 + 2 Q32, ..., and three passes over the
longer of S and L (3 max(c, 2r + 4) = 132 iterations) mix them with
A = S[i] = (S[i] + A + B) <<< 3 and B = L[j] = (L[j] + A + B) <<< (A + B).

The Orange file follows the paper's text with these correspondences:

| RC6 paper | Orange spec |
| --- | --- |
| Details of RC6: little-endian words | `as little Word[32]^c` on the key, `as little Word[32]^4` and `as little Word[8]^16` on the block, in `key_schedule`, `encrypt` and `decrypt` |
| Details of RC6: a <<< b, a >>> b by the low lg w bits of b | `<<<` and `>>>` with a computed amount, which turns by the amount modulo 32 |
| Key schedule: P32 and Q32 | `p32`, `q32` |
| Key schedule: L[0..c-1], S[0..2r+3] and the mixing loop | `key_schedule[c]`, for c = 4 through 8 key words |
| Encryption and decryption: f(x) | `f` |
| Encryption and decryption: encryption with RC6-w/r/b | `encrypt` |
| Encryption and decryption: decryption with RC6-w/r/b | `decrypt` |

The round keys are a `Word[32]^44`, S[0] through S[43], and the key words a
`Word[32]^c` with the size c taken from the key's length, so one
`key_schedule` serves the 16-, 24- and 32-byte keys. It also accepts the
20- and 28-byte keys the paper allows, but no published vector in this
entry uses them. The four registers A, B, C, D travel through the rounds
as a tuple of four words, in the paper's order.

The entry's scope is the AES parameters, w = 32 and r = 20, with keys of
whole words from 16 through 32 bytes. The paper's RC6-w/r/b also allows
w = 16 or 64 and any key from 0 through 255 bytes, the last word padded
with zero bytes. Those cases need code this file does not carry: their own
P, Q and lg w per width, a padding of the key, and, for keys over 176
bytes (c > 44), a mixing loop of 3c iterations instead of the 132 the file
writes. No vector here needs them.

### Security status

RC6 inherits RC5's design rationale: the rotation amounts depend on the
data, so the differential and linear characteristics that carry through a
fixed rotation are broken up, and the quadratic function was added because
the attacks on RC5 (Kaliski and Yin, 1995; Knudsen and Meier, 1996;
Biryukov and Kushilevitz, 1998) exploited rounds in which the rotation
amounts were zero or otherwise predictable; in RC6 the amount is taken from
the top bits of x(2x + 1), which depend on every bit of x.

The published cryptanalysis of the AES parameters is statistical. Gilbert,
Handschuh, Joux and Vaudenay (circulated in 1999 during the second round,
published at FSE 2000) and Knudsen and Meier (FSE 2000) independently
observed that the low five bits of the registers, which choose the
rotations, are not uniformly distributed after several rounds, and built
chi-squared distinguishers and key-recovery attacks from it: key recovery
reaches 14 rounds for every key length and 15 rounds for 256-bit keys, with
on the order of 2^118 to 2^119 chosen plaintexts and time close to
exhaustive search, and Knudsen and Meier extend the correlation to 17
rounds for a small class of weak keys. Later refinements of the chi-squared
method (Miyaji and Nonaka, 2002; Isogai, Matsunaka and Miyaji, 2003) gain
about a round for the longer keys or for RC6 without its post-whitening.
Shimoyama, Takenaka and Kaneko (2002) reached 14 rounds with multiple
linear approximations. The round counts
and data complexities above are as the papers' abstracts and NIST's Round 2
report state them, from memory; they were not checked from this machine,
which cannot reach the papers. None of these attacks approaches 20 rounds,
and all need more plaintext than the 2^64 blocks a 128-bit block cipher can
safely encrypt under one key, so RC6-32/20/b is unbroken; its margin, about
five rounds beyond the best key recovery, is smaller than that of Serpent or
Twofish and was the reason NIST's Round 2 report (Nechvatal et al., 2000)
rated its security margin "adequate" rather than "high".

RC6 lost the AES selection on cost rather than on security. Its speed on
32-bit processors with a fast multiplier was among the best of the
finalists, but the 32-bit multiplication and the variable rotation are slow
on 8-bit smart-card processors and expensive in hardware area and latency,
where Rijndael's byte operations are cheap, and NIST's report also weighed
that RSA Security held patents on the construction: U.S. Patent 5,724,428
on RC5's data-dependent rotation (filed 1995) and U.S. Patent 6,269,163 on
RC6's enhancements (filed 1998). RSA had announced that RC6 would be
royalty-free if selected; as it was not, the patents stood until they
expired, twenty years after filing, in 2015 and 2018, and RC6 is now
unencumbered (the patent numbers and dates are as RSA's submission and the
patents' front pages give them, and were not checked from this machine).
RC6 was also submitted to the European NESSIE project (2000) and was not
retained in its 2003 portfolio. The same features are the cipher's
side-channel liability: on processors where a multiplication or a
variable-amount rotation takes a data-dependent number of cycles, RC6 leaks
through timing, as Handschuh and Heys showed for RC5's rotations (1998),
and constant-time implementations must emulate both.

Standing as of September 2026: no attack on the full 20 rounds is
published; RC6 has no NIST, ISO or IETF standard status and so no
deprecation to record; it has a 128-bit block, so the birthday bound of
2^64 blocks per key applies to its use in any mode. Its historical place is
as the AES finalist with the simplest description and as the cipher that
carried RC5's data-dependent rotation into the AES generation.

### What the Orange rendering shows

The one data-dependent choice in RC6 is the rotation amount, and the file
writes it as the paper does: `((a ^ t) <<< u) + s[2 * i]` in a round and
`(l[k % c] + a1 + b) <<< (a1 + b)` in the key schedule, with the amount a
`Word[32]` computed from the data. A computed rotation turns by its amount
modulo 32, which is the paper's "least significant lg w bits of b", so no
mask is written; only the rotations by the literal lg w = 5 in `f` and by
3 in the key schedule are fixed. Forty of a block's rotations, two per round,
are data-dependent, and so are the 132 rotations by A + B in the key
schedule. Nothing else depends on data: there are no tables, and every
index is a literal or computed from a loop index (`2 * i + 1`,
`41 - 2 * k`, `k % 44`, `k % c`).

Byte order is written where the paper fixes it: `as little` loads the key
into L[0..c-1] and the block into A, B, C, D, and stores the registers back
into sixteen bytes. The key length is a size: `key_schedule[c]` takes a key
of `4 * c` bytes, the checker picks the instance from the length of the
`hex"..."` literal, and `j = (j + 1) mod c` is `k % c` on the loop counter,
proved in range for each instance. The mixing loop carries S, L, A and B as
a tuple of accumulators, so each of the paper's 132 iterations is one step
that computes A and B and stores them into S[i] and L[j]. Encryption and
decryption each carry (A, B, C, D) as a tuple through their twenty rounds;
the register rotation (A, B, C, D) = (B, C, D, A) is the order of the
tuple a step returns, and decryption walks the round keys down with the
static indices `s[41 - 2 * k]` and `s[40 - 2 * k]`. Vectors, keys and
plaintexts are `hex"..."` as the vector files print them.

Measured costs under `orangec test --stats`: `f` costs about 10 steps and
a round about 60; a block costs about 1,280 steps to encrypt and 1,340 to
decrypt. The key schedule costs about 7,700 steps for every key length,
since its loop runs 132 times whatever c is, so a test (one key schedule
and one block) costs 8,993 to 9,054 steps. The twelve tests together use
108,106 steps.

## Dissemination

### Files

- `rc6.or`: RC6-32/20/16, /24 and /32, the key schedule, encryption and
  decryption, with the twelve tests below.

### Running

```console
orangec test algorithms/rc6/rc6.or
python3 algorithms/verify.py algorithms/rc6
```

`orangec eval` prints only the parameterless specs `p32` and `q32`.

### Vectors

Each row is a `test` block in `rc6.or`, comparing a ciphertext (or, for the
decryptions, a plaintext) with the published value.

| Test | Source | Case |
| --- | --- | --- |
| `RC6 paper, rc6val.dat entry 1: RC6-32/20/16 encrypts` | RC6 paper test vectors, as Crypto++ `TestData/rc6val.dat` entry 1 | RC6-32/20/16, key 00...00, plaintext 00...00, ciphertext 8fc3a536...9848a41e |
| `RC6 paper, rc6val.dat entry 2: RC6-32/20/16 encrypts` | RC6 paper test vectors, `rc6val.dat` entry 2 | RC6-32/20/16, key 0123456789abcdef0112233445566778, plaintext 02132435...cedfe0f1 |
| `RC6 paper, rc6val.dat entry 2: RC6-32/20/16 decrypts` | `rc6val.dat` entry 2, decryption | the entry 2 ciphertext 524e192f...7ea43f18 decrypted to its plaintext |
| `RC6 paper, rc6val.dat entry 3: RC6-32/20/24 encrypts` | RC6 paper test vectors, `rc6val.dat` entry 3; also Botan 1.11 `rc6.vec` | RC6-32/20/24, key 00...00 (24 bytes), plaintext 00...00 |
| `RC6 paper, rc6val.dat entry 4: RC6-32/20/24 encrypts` | RC6 paper test vectors, `rc6val.dat` entry 4; also Botan 1.11 `rc6.vec` | RC6-32/20/24, key 01234567...ccddeeff0 (24 bytes), plaintext 02132435...cedfe0f1 |
| `RC6 paper, rc6val.dat entry 4: RC6-32/20/24 decrypts` | `rc6val.dat` entry 4, decryption | the entry 4 ciphertext 688329d0...f95291d4 decrypted to its plaintext |
| `RC6 paper, rc6val.dat entry 5: RC6-32/20/32 encrypts` | RC6 paper test vectors, `rc6val.dat` entry 5; also Botan 1.11 `rc6.vec` | RC6-32/20/32, key 00...00 (32 bytes), plaintext 00...00 |
| `RC6 paper, rc6val.dat entry 6: RC6-32/20/32 encrypts` | RC6 paper test vectors, `rc6val.dat` entry 6; also Botan 1.11 `rc6.vec` | RC6-32/20/32, key 01234567...98badcfe (32 bytes), plaintext 02132435...cedfe0f1 |
| `RC6 paper, rc6val.dat entry 6: RC6-32/20/32 decrypts` | `rc6val.dat` entry 6, decryption | the entry 6 ciphertext c8241816...674e5d48 decrypted to its plaintext |
| `Bouncy Castle RC6Test.java test 0: RC6-32/20/16 encrypts` | AES-submission reference KATs, Bouncy Castle `RC6Test.java` test 0; also Botan 1.11 `rc6.vec` | RC6-32/20/16, key 00...00, plaintext 80 00...00 |
| `Bouncy Castle RC6Test.java test 1: RC6-32/20/24 encrypts` | Bouncy Castle `RC6Test.java` test 1; also Botan 1.11 `rc6.vec` | RC6-32/20/24, key with byte 16 = 80 and the rest zero, plaintext 00...00 |
| `Bouncy Castle RC6Test.java test 4: RC6-32/20/32 encrypts` | Bouncy Castle `RC6Test.java` test 4; also Botan 1.11 `rc6.vec` | RC6-32/20/32, key with byte 0 = 10 and the rest zero, plaintext 00...00 |

The six `RC6 paper` encryptions are the test vectors appended to the RC6
paper (two per key length), copied from Crypto++'s
`TestData/rc6val.dat`, which carries them verbatim; the four 192- and
256-bit ones are also among the 1,219 cases of Botan 1.11's
`src/tests/data/block/rc6.vec`, which does not carry the two 128-bit ones.
The three decryptions state the same entries' plaintexts as their expected
values and so add no value from an oracle. The three `Bouncy Castle` rows
are known-answer tests of the AES submission's reference implementation
(`rc6-unix-refc`) as Bouncy Castle's
`core/src/test/java/org/bouncycastle/crypto/test/RC6Test.java` carries
them; Botan's `rc6.vec` carries all six of that file's cases. No expected
value was produced by an oracle.

### Provenance and claims

Botan's `src/lib/block/rc6/rc6.cpp` and `src/tests/data/block/rc6.vec`,
named as reference material, answer 404 on `raw.githubusercontent.com` for
master and the release-2 branch: RC6 is absent from Botan's current tree.
They exist at the older tags, and the sources fetched and kept in the
scratch directory are Botan 1.10.17's `src/block/rc6/rc6.cpp` and
`checks/validate.dat` and Botan 1.11.29's `src/tests/data/block/rc6.vec`,
Crypto++'s `rc6.cpp`, `rc6.h` and `TestData/rc6val.dat`, and Bouncy
Castle's `RC6Engine.java` and `RC6Test.java`. The paper itself is not
reachable from the build, so the file's comments cite its sections by their
titles ("Details of RC6", "Key schedule", "Encryption and decryption")
rather than by number, and the test vectors by the file that carries them.

P32 and Q32 were recomputed by script from their definition,
P32 = Odd((e - 2) 2^32) and Q32 = Odd((phi - 1) 2^32) with Odd the nearest
odd integer, and matched against the two fetched implementations and the
Orange file (`constants.py`). The vector literals in the first form of
`rc6.or` were generated by script from the two fetched vector files, not
typed (`gen_rc6_or.py`), as were the two 32-arm rotation selections that
form used.

There is no RC6 in `hashlib`, `cryptography` or `pycryptodome`, so a plain
Python RC6-32/20/b written from the paper's structure (`rc6.py`) serves as
the oracle; it encrypts and decrypts all six `rc6val.dat` entries, all six
`RC6Test.java` entries and all 1,219 entries of Botan's `rc6.vec` and
`validate.dat` (511 with 128-bit keys, 322 with 192-bit, 386 with 256-bit)
to their published values (`check_botan.py`), and it can dump the key
schedule and the round-by-round registers. The Orange file matched all
twelve vectors on its first evaluation, so the round-by-round comparison
was not needed.

The entry was then rewritten in the current language: one sized
`key_schedule` in place of three, the computed rotations in place of the
32-arm selections, tuples for the registers and the mixing state, and
`as little` for the byte orders. Every expected value is carried over byte
for byte from the first form, where each was a `<name>_expected` spec of
sixteen byte literals; the new tests state the same bytes as `hex"..."`,
grouped in words as the vector files print them, and a script traced each
old value into the new file. The keys, plaintexts and ciphertext inputs
are the first form's bytes in the same notation. No vector was added or
dropped, and the rewritten file also passed all twelve tests on its first
run. The step costs above were measured with `orangec test --stats`.

This entry is a reference evaluation of the RC6 specification under
`orangec test`. It makes no constant-time, side-channel, performance or
certification claim, and it is not a corpus entry in the sense of The
Orange Book chapter 12.

## Gaps

None that prevented any planned vector. The one case of the paper the
language cannot state:

- No array has zero elements, so the empty key (b = 0, which the paper
  allows and loads as c = 1 word L[0] = 0) cannot be passed as a
  `Word[8]^b`. Every other key length the paper allows could be written
  with one sized spec: a loop copying the b bytes into a zero fill of
  `4 * ((b + 3) / 4)` bytes pads the last word, and a loop bound
  `0..(3 * (44 + (c / 45) * (c - 44)))` is 3 max(c, 44) for every c up to
  64; both were checked in scratch. The entry does not carry them (see
  Structure).
