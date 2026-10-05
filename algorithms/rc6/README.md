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
| Details of RC6: little-endian words | `load_le32`, `le_bytes`, `load_block`, `store_block` |
| Details of RC6: a <<< b, a >>> b by the low lg w bits of b | `rotate_left_by`, `rotate_right_by` |
| Key schedule: P32 and Q32 | `p32`, `q32` |
| Key schedule, step 1: L[0..c-1] | `key_words_128`, `key_words_192`, `key_words_256` |
| Key schedule, step 2: S[0..2r+3] | `schedule_state` |
| Key schedule, step 3: the mixing loop | `mix`, `key_schedule_128`, `key_schedule_192`, `key_schedule_256`, `round_keys` |
| Encryption and decryption: f(x) | `f` |
| Encryption and decryption: one round | `round`, `inverse_round` |
| Encryption and decryption: whitening and the r rounds | `encrypt`, `decrypt` |

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

The one data-dependent choice in RC6 is the rotation amount. A computed
amount checks. `x <<< amount` and `x >>> amount`, with the amount a
`Word[32]` or an `Int`, turn by that amount modulo 32. A literal amount is
still only 0 through 31 (`ORC0216`). This file still writes `rotate_left_by` and
`rotate_right_by` as a 32-arm selection on the low five bits of the amount
word, and each such rotation costs about 78 evaluation steps
(measured by the loop-and-binary-search method of the folder's brief).
Everything else is 32-bit arithmetic: `f` is a multiplication, an addition
and a rotation by the literal 5, about 16 steps, and one round, with two
`f`, two data-dependent rotations and two additions, about 190 steps. A
block therefore costs about 5,300 steps to encrypt and 5,400 to decrypt,
and forty of the block's 128 rotations, two per round, are the
data-dependent ones.

The key schedule costs about ten times the block, 52,000 to 55,000 steps,
and nearly all of it is bookkeeping rather than arithmetic. A loop may
carry a tuple of accumulators, and a step may begin with `let`, so one
step can compute A and B and store them into S[i] and L[j]. This file
still carries S, L, A and B through the mixing loop as one `Word[32]^54`
(S in positions 0 through 43, L in 44 through 51, A at 52 and B at 53) and
splits each of the paper's 132 iterations into two steps of an inner loop,
the first computing A and B into their slots (`mix`), the second storing
them into S[i] and L[j]; each of the four 54-word updates costs 54 steps,
about 220 of the roughly 370 steps an iteration takes. The modulus c of
the index j is a literal, so each key length has its own copy of the
mixing loop (`key_schedule_128`, `_192`, `_256`) around the shared `mix`,
and L is carried as eight words for every length with zero beyond c, which
the schedule never reads. The register rotation (A, B, C, D) = (B, C, D, A) and
the whitening are array literals, `[x[1], c, x[3], a]` and
`[x[0], x[1] + s[0], x[2], x[3] + s[1]]`, and decryption walks the round
keys with the static index `s[40 - 2 * i]`.

One vector, a key schedule and a block, costs about 60,000 steps. The
twelve pairs of the file (nine encryptions and three decryptions) evaluate
together at roughly 720,000 of the 1,048,576 steps; five more RC6-256
encryptions would still fit. Nothing was moved to a second file or dropped.

## Dissemination

### Files

- `rc6.or`: RC6-32/20/16, /24 and /32, the key schedule, encryption and
  decryption, with the twelve vector pairs.

### Running

```console
orangec eval algorithms/rc6/rc6.or
python3 algorithms/verify.py algorithms/rc6
```

`eval` prints every parameterless spec, so `p32` and `q32` appear before
the twelve pairs below.

### Vectors

| Spec | Source | Case |
| --- | --- | --- |
| `rc6_paper_128_1` | RC6 paper test vectors, as Crypto++ `TestData/rc6val.dat` entry 1 | RC6-32/20/16, key 00...00, plaintext 00...00, ciphertext 8fc3a536...9848a41e |
| `rc6_paper_128_2` | RC6 paper test vectors, `rc6val.dat` entry 2 | RC6-32/20/16, key 0123456789abcdef0112233445566778, plaintext 02132435...cedfe0f1 |
| `rc6_paper_128_2_decrypt` | `rc6val.dat` entry 2, decryption | the entry 2 ciphertext 524e192f...7ea43f18 decrypted to its plaintext |
| `rc6_paper_192_1` | RC6 paper test vectors, `rc6val.dat` entry 3; also Botan 1.11 `rc6.vec` | RC6-32/20/24, key 00...00 (24 bytes), plaintext 00...00 |
| `rc6_paper_192_2` | RC6 paper test vectors, `rc6val.dat` entry 4; also Botan 1.11 `rc6.vec` | RC6-32/20/24, key 01234567...ccddeeff0 (24 bytes), plaintext 02132435...cedfe0f1 |
| `rc6_paper_192_2_decrypt` | `rc6val.dat` entry 4, decryption | the entry 4 ciphertext 688329d0...f95291d4 decrypted to its plaintext |
| `rc6_paper_256_1` | RC6 paper test vectors, `rc6val.dat` entry 5; also Botan 1.11 `rc6.vec` | RC6-32/20/32, key 00...00 (32 bytes), plaintext 00...00 |
| `rc6_paper_256_2` | RC6 paper test vectors, `rc6val.dat` entry 6; also Botan 1.11 `rc6.vec` | RC6-32/20/32, key 01234567...98badcfe (32 bytes), plaintext 02132435...cedfe0f1 |
| `rc6_paper_256_2_decrypt` | `rc6val.dat` entry 6, decryption | the entry 6 ciphertext c8241816...674e5d48 decrypted to its plaintext |
| `bouncycastle_rc6test_0` | AES-submission reference KATs, Bouncy Castle `RC6Test.java` test 0; also Botan 1.11 `rc6.vec` | RC6-32/20/16, key 00...00, plaintext 80 00...00 |
| `bouncycastle_rc6test_1` | Bouncy Castle `RC6Test.java` test 1; also Botan 1.11 `rc6.vec` | RC6-32/20/24, key with byte 16 = 80 and the rest zero, plaintext 00...00 |
| `bouncycastle_rc6test_4` | Bouncy Castle `RC6Test.java` test 4; also Botan 1.11 `rc6.vec` | RC6-32/20/32, key with byte 0 = 10 and the rest zero, plaintext 00...00 |

The six `rc6_paper` encryptions are the test vectors appended to the RC6
paper (two per key length), copied from Crypto++'s
`TestData/rc6val.dat`, which carries them verbatim; the four 192- and
256-bit ones are also among the 1,219 cases of Botan 1.11's
`src/tests/data/block/rc6.vec`, which does not carry the two 128-bit ones.
The three decryptions state the same entries' plaintexts as their expected
values and so add no value from an oracle. The three `bouncycastle` rows
are known-answer tests of the AES submission's reference implementation
(`rc6-unix-refc`) as Bouncy Castle's
`core/src/test/java/org/bouncycastle/crypto/test/RC6Test.java` carries
them; Botan's `rc6.vec` carries all six of that file's cases. No
`_expected` value was produced by an oracle.

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
Orange file (`constants.py`). The vector literals in `rc6.or` were
generated by script from the two fetched vector files, not typed
(`gen_rc6_or.py`), as were the two 32-arm rotation selections.

There is no RC6 in `hashlib`, `cryptography` or `pycryptodome`, so a plain
Python RC6-32/20/b written from the paper's structure (`rc6.py`) serves as
the oracle; it encrypts and decrypts all six `rc6val.dat` entries, all six
`RC6Test.java` entries and all 1,219 entries of Botan's `rc6.vec` and
`validate.dat` (511 with 128-bit keys, 322 with 192-bit, 386 with 256-bit)
to their published values (`check_botan.py`), and it can dump the key
schedule and the round-by-round registers. The Orange file matched all
twelve vectors on its first evaluation, so the round-by-round comparison
was not needed. The step costs quoted above were measured by the brief's
loop-and-binary-search method (`measure.py`, `measure2.py`) and the
headroom by appending RC6-256 encryptions until ORC0301 (`budget.py`).

This entry is a reference evaluation of the RC6 specification under
`orangec eval`. It makes no constant-time, side-channel, performance or
certification claim, and it is not a corpus entry in the sense of The
Orange Book chapter 12.

## Gaps

None that prevented any planned vector. What this file's shape costs, and
the limit that remains:

- A computed rotation amount checks, including `(l + a + b) <<< (a + b)`.
  This source still writes the data-dependent rotation as a 32-arm
  selection (`rotate_left_by`, `rotate_right_by`) at about 78 steps
  instead of one operation. A literal amount on `Word[32]` is still only
  0 through 31.
- A loop step may begin with `let`, and a loop may carry a tuple of
  accumulators, so one step can compute A and B and store both. This
  source still splits each iteration into two steps of an inner loop
  (compute, then store) over a 54-word state; the four 54-word updates
  per iteration make the key schedule about ten times the cost of a block.
- An index modulus must be a literal, so `j = (j + 1) mod c` is written
  once per key length: three `key_schedule` specs around one `mix`.
