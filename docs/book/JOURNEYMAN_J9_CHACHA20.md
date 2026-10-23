# The Orange Book

By Chase Bryan

## Part 2, The Journeyman

J9: ChaCha20. Draft 2026-10-23.

Continue from
[SHA-512 and the Rest of the SHA-2 Family](JOURNEYMAN_J6_SHA512_AND_THE_SHA2_FAMILY.md#j6-sha-512-and-the-rest-of-the-sha-2-family).
J6 derived SHA-512. This lesson is a stream cipher, not a hash.
[Name the Intermediate Step](NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md#n7-name-the-intermediate-step)
named each update of the quarter round.
[Byte Order and Format Boundaries](JOURNEYMAN_J4_BYTE_ORDER_AND_FORMAT_BOUNDARIES.md#j4-byte-order-and-format-boundaries)
is the byte-order lesson. The manuscript chapter
[Orange 2026: The Smallest Honest Slice](../THE_ORANGE_BOOK.md#chapter-8-orange-2026-the-smallest-honest-slice)
uses the quarter round as an illustration of the slice. This lesson
cites those three. It does not repeat their programs, and it does
not repeat their proofs.
[Standards as Versioned Inputs](JOURNEYMAN_J2_STANDARDS_AS_VERSIONED_INPUTS.md#j2-standards-as-versioned-inputs)
already recorded the June 2018 file and Proposition J2.1, the
product `(2^32 - 1) * 64`. This lesson cites that product. The
reason the state layout produces it is proved here.

This lesson is **J9**. The locked label is J9. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. When
the text says “§2.1” or “§2.4”, the number is a section of RFC 8439,
June 2018.

## J9: ChaCha20

> “The purpose of this document is to serve as a stable reference for IETF documents making use of these algorithms.”
>
> — Yoav Nir and Adam Langley, RFC 8439, *ChaCha20 and Poly1305 for IETF Protocols* (June 2018), §1. [S20]

The sentence is in §1, after the sentence that names Bernstein's
papers. It states a purpose: a stable reference for other IETF
documents. It does not say that a changed plaintext produces a
changed ciphertext, and it does not say that a nonce may be reused.
The Abstract's second paragraph is J2's epigraph, [S13]. This
sentence is not that sentence. Section 1 also asserts speed and a
claim about observation. This lesson does not adopt those sentences
and does not prove them. Section 4 is the security considerations
section. It is named and not transcribed. Poly1305 is §2.5. The
AEAD is §2.8. Neither is built on this page.

**The only-this-stack test.** You will derive the quarter round,
the block function, and encryption from the June 2018 text, then
run the transcription with `orangec check`, `orangec eval`, and
`orangec test` on the compiler this tree builds. You will also feed
the compiler programs it rejects, and programs it accepts whose
tests fail. The compiler's version line is
`orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`. Every
listing is written for that slice. Every fenced program is Current
on that slice. No listing is marked Proposed. A form this slice does
not implement is not given a syntax here. A program the compiler
rejects is still Current: the rejection is the result the listing
exists to show.

### J9.1 The sections, in the order the RFC uses them

Six outcomes finish the lesson. None of them is a certificate, and
none of them is conferred by reaching the last page.

1. You can name the sections of RFC 8439, June 2018, that define
   the quarter round, the block function, and encryption, in the
   order §2 uses them, and you can separate that file from the
   64-bit counter Bernstein's ChaCha used.
2. You can compute the §2.1.1 quarter round, including the one sum
   that crosses `2^32`, and you can invert those eight assignments.
3. You can apply one quarter round to the §2.2.1 state and say
   which twelve words do not move.
4. You can set up the §2.3.2 block, compute the first column
   quarter round by hand, and say why the feed-forward sum is not
   an input to the inverse of the 20-round permutation. You can
   match the serialized block and the five Appendix A.1 keystreams.
5. You can encrypt the §2.4.2 text and Appendix A.2 vectors 1, 2,
   and 3, and you can derive the message-length bound from the
   32-bit counter and the 96-bit nonce.
6. You can predict what fails, and how, when the first rotation is
   15, when a key, counter, or nonce is loaded in the other byte
   order, when the first quarter round uses the diagonal indices,
   when the sunscreen counter is 0, and when one nonce encrypts
   two different plaintexts.

The edition is RFC 8439, *ChaCha20 and Poly1305 for IETF
Protocols*, June 2018, which obsoletes RFC 7539. The plain-text
file J2 retrieved on 2026-10-05 has SHA-256
`25bef70fbf7a07ff45c2fe4cb7c6ce954eac687413d8610603268b4e4415324c`.
The file checked for the sentences below, on 2026-10-23, has that
same digest. [T9] is J2's retrieval record. [T16] records that this
lesson uses that file and does not claim a second download.

**Assumption J9.1 — The pin is the edition, not the title.**
“ChaCha20 and Poly1305 for IETF Protocols” is the title of RFC 7539
and of RFC 8439. The algorithms, the section numbers, and the test
vectors below are the June 2018 file. Lesson J2 is the lesson that
makes a pin a procedure. This lesson cites that file's digest. It
does not repeat J2's comparison of the two editions' `P_MAX`
sentences.

**Assumption J9.2 — A word is 32 bits, and `+` is the residue.**
§2.1 says `+` denotes integer addition modulo `2^32`. The same
modulus is the note under the block function in §2.3: addition of
the original input words to the output words is modulo `2^32`.
`2^32 = 4294967296`. The residue is the unique remainder in `0`
through `2^32 − 1`. On this compiler, `+` of two `Word[32]` values
is that residue, and so is `-`. `^` is exclusive or. `<<< n` is a
left rotation by `n` bits. `>>> n` is a right rotation by `n` bits.
The distances in the quarter round are the literals `16`, `12`,
`8`, and `7`. The hand arithmetic below is the mathematical claim.
A Match says the program agreed on the inputs the test wrote.

**Assumption J9.3 — The RFC loads key, counter, and nonce
little-endian.** §2.3 says the key is eight 32-bit little-endian
integers, the nonce is three 32-bit little-endian integers, and the
block count is a 32-bit little-endian integer. Serialization at the
end of the block is the same order. Lesson J4 is the lesson that
treats byte order. The correct listings below use `as little`. The
constant words are the little-endian reading of the 16 bytes in the
string `expand 32-byte k`. J4 already derived the first of those
words, `0x61707865`. This lesson uses that word. It does not repeat
the place-value proof.

**Assumption J9.4 — The eight names are N7's, written as one
function.** §2.1 updates `a`, `b`, `c`, and `d` in place:

```text
a += b; d ^= a; d <<<= 16;
c += d; b ^= c; b <<<= 12;
a += b; d ^= a; d <<<= 8;
c += d; b ^= c; b <<<= 7;
```

N7 bound each result once: `a1`, `d1`, `c1`, `b1`, `a2`, `d2`,
`c2`, `b2`. A name in Orange is not assigned twice. The listings
use N7's names inside one spec, `quarter_round`, and return the
four final words as a tuple `(a2, b2, c2, d2)`. That order is `a`,
`b`, `c`, `d` after the quarter round, which is the order §2.1.1
prints. It is not the order the eight lines finish in. The last
line updates `b`. The tuple still prints `a`, then `b`, then `c`,
then `d`. The names in the listings are ASCII. The mapping is:

```text
QUARTERROUND(a, b, c, d)           quarter_round
a += b; d ^= a; d <<<= 16;         a1, d1, <<< 16
c += d; b ^= c; b <<<= 12;         c1, b1, <<< 12
a += b; d ^= a; d <<<= 8;          a2, d2, <<< 8
c += d; b ^= c; b <<<= 7;          c2, b2, <<< 7
the reverse composition            inverse_quarter_round
inner_block                        the body of rounds
for i=1 upto 10                    for i in 1..11
chacha20_block                     block
constants | key | counter | nonce  setup
state += initial_state             feed
serialize                          as little Word[8]^64
chacha20_encrypt                   encrypt
counter+j                          counter + (j as Word[32])
partial-block discard              full[..114], full[..127], full[..375]
```

The right-hand column is the spelling in Listings J9.1, J9.3, and
J9.7. The eight in-place lines are `a1` through `b2`, with
`<<< 16`, `<<< 12`, `<<< 8`, and `<<< 7`. `inner_block` is the
body of `rounds`. It is not a separate spec.

**Assumption J9.5 — A Match is one `Bool` on the inputs written in
that test.** Well-formed source is not a Match. A Match on these
inputs is not a Match on every input.
A Match is not called verified.
Nothing on this page is a security claim. A count of evaluator
steps is the number `orangec test --stats` prints for a test. It
is not a count of machine operations, and this lesson draws no
conclusion about speed from it.

**Assumption J9.6 — The hand calculation and the program denote the
same word only where both perform the same operations under J9.2
through J9.4.** Agreement on one input is one point. The domain of
one 32-bit word is `2^32` inputs. The domain of a 16-word state is
`2^512` inputs. The state after 20 rounds, as §2.3.2 prints it, is
checked by the listing. It is not recomputed by hand for all 80
quarter rounds. One column quarter round is.

**Assumption J9.7 — A shorter message is a slice of a whole number
of blocks.** §2.4 says the plaintext need not be a multiple of 512
bits, and that extra keystream from the last block is discarded.
The encryption spec in this lesson takes `blocks` in `1` through
`6` and a plaintext of `64 * blocks` bytes. A message of 114 bytes
is written as 128 bytes, encrypted, and cut with `[..114]`. The
cut is the discard. There is no `floor` in the listing. The size
parameter is a block count, not the byte length 114.

### J9.2 The quarter round is a bijection

§2.1. Let `W` be the integers modulo `2^32`, the set `Word[32]`
under Assumption J9.2. A quarter round is a function from `W^4` to
`W^4`.

**Lemma J9.1.** For an integer `r` with `0 ≤ r ≤ 32`, left rotation
by `r` and right rotation by `r` are inverse functions on `W`.
Exclusive or with a fixed word is an involution. Addition of a
fixed word is a bijection, and its inverse is subtraction of that
word.

*Proof.* Number the bits of a word `0` through `31`, bit `0` the
least significant. Left rotation by `r` sends bit `i` to bit
`(i + r) mod 32`. Right rotation by `r` sends that bit to
`(i + r − r) mod 32 = i`. When `r` is `0` or `32`, both rotations
are the identity. So each rotation is a bijection, and the two
are inverses.

Exclusive or with a fixed word `u` is its own inverse: `(x ^ u) ^ u = x`,
because `u ^ u = 0` and `x ^ 0 = x`. A function that is its own
inverse is a bijection.

Addition of a fixed word `u` sends `x` to `x + u`. Subtraction of
`u` sends that sum to `(x + u) − u = x`. Every word is `y − u` for
`y = x + u` when `x = y − u`, so the map is bijective. □

**Proposition J9.1.** Each of the four double-steps in §2.1 is a
bijection of the four-word state. The quarter round, their
composition, is a bijection of `W^4`. The function
`inverse_quarter_round` in Listing J9.1 is its inverse.

*Proof.* Write the state as `(a, b, c, d)`. The first double-step
is `a1 = a + b` and `d1 = (d ^ a1) <<< 16`, with `b` and `c`
unchanged. Given `(a1, b, c, d1)`, Lemma J9.1 recovers
`d = (d1 >>> 16) ^ a1` and `a = a1 − b`. The second double-step
replaces `c` by `c1 = c + d1` and `b` by `b1 = (b ^ c1) <<< 12`.
Given the new four words, `b = (b1 >>> 12) ^ c1` and `c = c1 − d1`.
The third replaces `a1` by `a2 = a1 + b1` and `d1` by
`d2 = (d1 ^ a2) <<< 8`. The fourth replaces `c1` by `c2 = c1 + d2`
and `b1` by `b2 = (b1 ^ c2) <<< 7`. Each is the same shape as the
first, with a different pair of words and a different distance
(`12`, `8`, or `7`). Each therefore has an inverse of the same
shape.

The quarter round is the composition of these four bijections, so
it is a bijection. An inverse of a composition is the composition
of the inverses in reverse order. That reverse order is:

```text
b1 = (b2 >>> 7) ^ c2
c1 = c2 - d2
d1 = (d2 >>> 8) ^ a2
a1 = a2 - b1
b  = (b1 >>> 12) ^ c1
c  = c1 - d1
d  = (d1 >>> 16) ^ a1
a  = a1 - b
```

`c2` is known before `c1` is recovered, because `c2` is one of the
four output words. The same is true of `a2` and `b2`. Listing J9.1
writes those eight lines and returns `(a, b, c, d)`. □

The fourth double-step's inverse uses `c2` while `c2` is still the
output word. It does not need `c1` yet. The third double-step's
inverse uses `b1` from the line above it. The order is forced.

**Proposition J9.2.** On the §2.1.1 input
`(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)`, the eight
results are `a1 = 0x12131415`, `d1 = 0x51721330`,
`c1 = 0xecff8273`, `b1 = 0xd8177edf`, `a2 = 0xea2a92f4`,
`d2 = 0x5881c4bb`, `c2 = 0x4581472e`, and `b2 = 0xcb1cf8ce`.
The only sum that crosses `2^32` is the sum inside `c2`.

*Proof.* `0x11111111 + 0x01020304 = 303240213 = 0x12131415`, and
`303240213 < 4294967296`, so `a1` does not wrap.

`0x01234567 ^ 0x12131415 = 0x13305172`. Left rotation by 16 swaps
the two halves of a word: `0x1330` and `0x5172` exchange, and
`d1 = 0x51721330`.

`0x9b8d6f43 + 0x51721330 = 3976168051 = 0xecff8273`. The sum is
below `2^32`, so `c1 = 0xecff8273`.

`0x01020304 ^ 0xecff8273 = 0xedfd8177`. Left rotation by 12 of
that word is `b1 = 0xd8177edf`.

`0x12131415 + 0xd8177edf = 3928658676 = 0xea2a92f4`. The sum is
below `2^32`. This is the final `a` that §2.1.1 prints.

`0x51721330 ^ 0xea2a92f4 = 0xbb5881c4`. Left rotation by 8 is
`d2 = 0x5881c4bb`, the final `d`.

`0xecff8273 + 0x5881c4bb = 5461067566`. Subtract one modulus:
`5461067566 − 4294967296 = 1166100270 = 0x4581472e`. So
`c2 = 0x4581472e`, the final `c`, and this is the wrap.

`0xd8177edf ^ 0x4581472e = 0x9d9639f1`. Left rotation by 7 is
`b2 = 0xcb1cf8ce`, the final `b`.

§2.1.1 prints `(a, b, c, d) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)`
after the quarter round. That is `(a2, b2, c2, d2)`. □

N7 already ran this vector as four specs. The new fact on this
page is the wrap inside `c2`, and the inverse.

**Listing J9.1 — `quarter.or`**

```orange
edition 2026;
module quarter {
  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec inverse_quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let b1: Word[32] = (b >>> 7) ^ c;
    let c1: Word[32] = c - d;
    let d1: Word[32] = (d >>> 8) ^ a;
    let a1: Word[32] = a - b1;
    let b0: Word[32] = (b1 >>> 12) ^ c1;
    let c0: Word[32] = c1 - d1;
    let d0: Word[32] = (d1 >>> 16) ^ a1;
    let a0: Word[32] = a1 - b0;
    (a0, b0, c0, d0)
  }

  spec a1() -> Word[32] { 0x11111111 + 0x01020304 }
  spec d1() -> Word[32] { (0x01234567 ^ 0x12131415) <<< 16 }
  spec c2() -> Word[32] {
    let q: Quad = quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567);
    q.2
  }
  spec vector() -> Quad { quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }
  spec recovered() -> Quad { inverse_quarter_round(0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb) }

  test "RFC 8439 2.1.1" {
    vector() == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
  }
  test "inverse of RFC 8439 2.1.1" {
    recovered() == (0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
  }
}
```

```sh
./compiler/target/debug/orangec eval --spec a1 -
./compiler/target/debug/orangec eval --spec d1 -
./compiler/target/debug/orangec eval --spec c2 -
./compiler/target/debug/orangec eval --spec vector -
./compiler/target/debug/orangec eval --spec recovered -
./compiler/target/debug/orangec test -
```

Check is silent. The evaluations print the words Proposition J9.2
derived, and the four words the inverse returns.

**Expected evaluation output:**

```text
quarter::a1: Word[32] = 0x12131415
quarter::d1: Word[32] = 0x51721330
quarter::c2: Word[32] = 0x4581472e
quarter::vector: (Word[32], Word[32], Word[32], Word[32]) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
quarter::recovered: (Word[32], Word[32], Word[32], Word[32]) = (0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
```

**Test report:**

```text
test "RFC 8439 2.1.1" ... ok
test "inverse of RFC 8439 2.1.1" ... ok
2 tests: 2 passed, 0 failed
```

The status of `test` is 0. The stderr of `orangec test --stats -`
on this listing, which is an evaluator-step count under Assumption
J9.5, is:

```text
test "RFC 8439 2.1.1": 54 steps
test "inverse of RFC 8439 2.1.1": 54 steps
total: 108 of 1048576 steps
```

The second test is Proposition J9.1 on the one input §2.1.1 names.
It is not a proof that the inverse works on every four-tuple. The
proof is the proposition. Do not call that Match verified.

### J9.3 One quarter round on sixteen words

§2.2. The state is sixteen words. `QUARTERROUND(x, y, z, w)` runs
the quarter round on the words at those indices and leaves the
other twelve alone. §2.2.1 applies `QUARTERROUND(2, 7, 8, 13)` to
a state the section says was generated randomly. Indices `2`, `7`,
`8`, and `13` are one diagonal of the matrix diagram in §1.1.
§2.2.1 says that call is part of a diagonal round.

**Proposition J9.3.** On that state, `a1 = 0x7bc3d2fd`, the sum
inside `c1` crosses `2^32`, and `c1 = 0x17ab6e07`. The four words
that change are index 2 `0xbdb886dc`, index 7 `0xcfacafd2`, index
8 `0xe46bea80`, and index 13 `0xccc07c79`. Index 0 stays
`0x879531e0`.

*Proof.* The input words are `a = 0x516461b1` at index 2,
`b = 0x2a5f714c` at index 7, `c = 0x53372767` at index 8, and
`d = 0x3d631689` at index 13.

`0x516461b1 + 0x2a5f714c = 2076431101 = 0x7bc3d2fd`, below
`2^32`, so `a1 = 0x7bc3d2fd`.

`0x3d631689 ^ 0x7bc3d2fd = 0x46a0c474`. Rotation by 16 swaps the
halves: `d1 = 0xc47446a0`.

`0x53372767 + 0xc47446a0 = 4692078087`. Subtract one modulus:
`4692078087 − 4294967296 = 397110791 = 0x17ab6e07`. So
`c1 = 0x17ab6e07`.

The remaining four assignments are the same rules, and they
produce `b1 = 0x41f4b3df`, `a2 = 0xbdb886dc`, `d2 = 0xccc07c79`,
`c2 = 0xe46bea80`, and `b2 = 0xcfacafd2`. The pre-rotation words
are `b ^ c1 = 0x3df41f4b`, `d1 ^ a2 = 0x79ccc07c`, and
`b1 ^ c2 = 0xa59f595f`. None of `a2`, `d2`, or `c2` wraps.
`a2 + b1` is the sum `3182986972`, which is already `a2`.

Those four results are the words §2.2.1 marks as changed. The
quarter round does not read index 0, and it does not write index
0, so index 0 remains `0x879531e0`. □

N12's ledger reduced `0x53372767 + 0xc47446a0` as a sample of
wrapping. The same sum is `c1` of this section. The derivation
above is the §2.2.1 calculation. It is not a second copy of N12's
exercise.

**Listing J9.2 — `on_state.or`**

```orange
edition 2026;
module on_state {
  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec sample() -> Word[32]^16 {
    [
      0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a,
      0x44c20ef3, 0x3390af7f, 0xd9fc690b, 0x2a5f714c,
      0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963,
      0x5c971061, 0x3d631689, 0x2098d9d6, 0x91dbd320,
    ]
  }

  spec applied() -> Word[32]^16 {
    let s: Word[32]^16 = sample();
    let q: Quad = quarter_round(s[2], s[7], s[8], s[13]);
    let s2: Word[32]^16 = s with [2] = q.0;
    let s7: Word[32]^16 = s2 with [7] = q.1;
    let s8: Word[32]^16 = s7 with [8] = q.2;
    s8 with [13] = q.3
  }

  spec word2() -> Word[32] { applied()[2] }

  spec mid_c() -> Word[32] {
    let s: Word[32]^16 = sample();
    let a1: Word[32] = s[2] + s[7];
    let d1: Word[32] = (s[13] ^ a1) <<< 16;
    s[8] + d1
  }

  test "RFC 8439 2.2.1 wrapped c" { mid_c() == 0x17ab6e07 }
  test "RFC 8439 2.2.1 word 2" { word2() == 0xbdb886dc }
  test "RFC 8439 2.2.1 word 7" { applied()[7] == 0xcfacafd2 }
  test "RFC 8439 2.2.1 word 8" { applied()[8] == 0xe46bea80 }
  test "RFC 8439 2.2.1 word 13" { applied()[13] == 0xccc07c79 }
  test "RFC 8439 2.2.1 word 0 unchanged" { applied()[0] == 0x879531e0 }
}
```

An array update returns a new array. `s with [2] = q.0` is one
expression. The next index is a second binding, `s2 with [7]`,
because `with` does not chain on the word `q.0`. The four updates
are the four indices §2.2 names. The other twelve words are the
sample's.

```sh
./compiler/target/debug/orangec eval --spec mid_c -
./compiler/target/debug/orangec eval --spec word2 -
./compiler/target/debug/orangec test -
```

**Expected evaluation output:**

```text
on_state::mid_c: Word[32] = 0x17ab6e07
on_state::word2: Word[32] = 0xbdb886dc
```

**Test report:**

```text
test "RFC 8439 2.2.1 wrapped c" ... ok
test "RFC 8439 2.2.1 word 2" ... ok
test "RFC 8439 2.2.1 word 7" ... ok
test "RFC 8439 2.2.1 word 8" ... ok
test "RFC 8439 2.2.1 word 13" ... ok
test "RFC 8439 2.2.1 word 0 unchanged" ... ok
6 tests: 6 passed, 0 failed
```

The status is 0. The stderr of `orangec test --stats -` on this
listing, which is an evaluator-step count under Assumption J9.5,
is:

```text
test "RFC 8439 2.2.1 wrapped c": 50 steps
test "RFC 8439 2.2.1 word 2": 103 steps
test "RFC 8439 2.2.1 word 7": 102 steps
test "RFC 8439 2.2.1 word 8": 102 steps
test "RFC 8439 2.2.1 word 13": 102 steps
test "RFC 8439 2.2.1 word 0 unchanged": 102 steps
total: 561 of 1048576 steps
```

`mid_c` is the wrapped residue in Proposition J9.3. The last test
is the word the quarter round does not touch.

### J9.4 The block function

§2.3. The inputs are a 256-bit key, a 96-bit nonce, and a 32-bit
block count. The output is 64 bytes. The state is sixteen words:

```text
constant constant constant constant
key      key      key      key
key      key      key      key
counter  nonce    nonce    nonce
```

The four constants are `0x61707865`, `0x3320646e`, `0x79622d32`,
and `0x6b206574`. They are the little-endian words of
`expand 32-byte k`, which is what the setup in the listings
concatenates. A column round is the four calls

```text
QUARTERROUND(0, 4, 8, 12)
QUARTERROUND(1, 5, 9, 13)
QUARTERROUND(2, 6, 10, 14)
QUARTERROUND(3, 7, 11, 15)
```

A diagonal round is the four calls

```text
QUARTERROUND(0, 5, 10, 15)
QUARTERROUND(1, 6, 11, 12)
QUARTERROUND(2, 7, 8, 13)
QUARTERROUND(3, 4, 9, 14)
```

§2.3 runs a column round and then a diagonal round, and repeats
that pair until 20 rounds have run. §2.3.1 writes the same eight
calls in `inner_block`, and `for i=1 upto 10` around it. Ten
iterations are twenty rounds, and twenty rounds are eighty quarter
rounds.

The listings write the loop as `for i in 1..11`. The end is
exclusive, so the body runs ten times. The body does not read `i`.
The RFC's `upto 10` and this slice's exclusive end are the same
ten iterations. They are not the same notation.

**Proposition J9.4.** Each of the eighty quarter-round maps is a
bijection of `W^16`. Their composition `Π`, in the order §2.3
names, is a bijection of `W^16`.

*Proof.* `QUARTERROUND(x, y, z, w)` applies the bijection of
Proposition J9.1 to those four coordinates and the identity to the
other twelve. A product of a bijection and an identity is a
bijection of the whole sixteen-word state: the four coordinates
can be restored by the inverse quarter round, and the other twelve
are already themselves. A composition of bijections is a
bijection. There are four quarter rounds in a column round, four
in a diagonal round, and ten pairs, which is eighty maps. `Π` is
that composition. The inverse of `Π` is those eighty inverse
quarter rounds, in reverse order. □

**Proposition J9.5.** Let `f(S) = S + Π(S)`, the sum in each
coordinate under Assumption J9.2. The block function serializes
`f(S)`. The inverse of `Π`, applied to `f(S)`, returns `S` if and
only if `S = 0`. The §2.3.2 initial state is not `0`.

*Proof.* `Π` is a bijection, so `Π^{-1}(Π(S)) = S`. Feeding `f(S)`
to that inverse computes `Π^{-1}(S + Π(S))`. This equals `S` if
and only if `S + Π(S) = Π(S)`, because `Π^{-1}` is injective,
which holds if and only if `S = 0`.

Word 0 of the §2.3.2 initial state is `0x61707865`, not `0`. So
the inverse of `Π`, given only the block output, does not return
the initial state.

The same fact is visible at one coordinate the RFC prints twice.
After 20 rounds, word 0 is `0x837778ab`. After the feed-forward
addition, word 0 is `0xe4e7f110`. Those are not the same word, so
the published state is not `Π(S)`. The addition that relates them
does not wrap: `0x61707865 + 0x837778ab = 3840405776 = 0xe4e7f110`.
Word 1 does wrap: `0x3320646e + 0xe238d763 = 4653136849`, and
`4653136849 − 4294967296 = 358169553 = 0x15593bd1`, which is the
word §2.3.2 prints in position 1 of the final matrix. □

Proposition J9.5 does not say that no function at all inverts `f`.
It says that the inverse of `Π` does not. The procedure
Proposition J9.4 names reads `Π(S)`. The block publishes
`S + Π(S)`. Subtracting one summand from the published word would
recover the other, and neither summand is the published word. The
key sits in words 4 through 11 of `S`. Those words are not a
coordinate of `f(S)` by themselves.

The twenty-round matrix is the RFC's printed matrix. This page
does not expand all eighty quarter rounds by hand. It expands the
first one.

**Proposition J9.6.** On the §2.3.2 setup, the first column
quarter round, `QUARTERROUND(0, 4, 8, 12)`, sends word 0 from
`0x61707865` to `0xabbaa25c`. The sum inside its `c2` crosses
`2^32`, and that `c2` is `0x6b3ca454`.

*Proof.* The setup §2.3.2 prints is

```text
61707865  3320646e  79622d32  6b206574
03020100  07060504  0b0a0908  0f0e0d0c
13121110  17161514  1b1a1918  1f1e1d1c
00000001  09000000  4a000000  00000000
```

Word 4 is the first key word. The key bytes begin `00 01 02 03`.
Little-endian, that word is `0x03020100`. Word 12 is the block
count `1`. Word 13 is the first nonce word. The nonce begins
`00 00 00 09`, and little-endian that word is `0x09000000`.

The first quarter round reads `a = 0x61707865`, `b = 0x03020100`,
`c = 0x13121110`, and `d = 0x00000001`.

`a1 = 0x61707865 + 0x03020100 = 1685223781 = 0x64727965`.

`d ^ a1 = 0x64727964`. Rotation by 16 gives `d1 = 0x79646472`.

`c1 = 0x13121110 + 0x79646472 = 2356573570 = 0x8c767582`.

`b ^ c1 = 0x8f747482`. Rotation by 12 gives `b1 = 0x474828f7`.

`a2 = 0x64727965 + 0x474828f7 = 2881135196 = 0xabbaa25c`. This is
the new word 0. The sum does not wrap.

`d1 ^ a2 = 0xd2dec62e`. Rotation by 8 gives `d2 = 0xdec62ed2`.

`c1 + d2 = 0x8c767582 + 0xdec62ed2 = 6094103636`. Subtract one
modulus: `6094103636 − 4294967296 = 1799136340 = 0x6b3ca454`.
That is `c2`.

`b1 ^ c2 = 0x2c748ca3`. Rotation by 7 gives `b2 = 0x3a465196`. □

N12's ledger reduced this same `c2` sum as a column sample. The
derivation above is the first quarter round of §2.3.2.

Appendix A.1 prints five keystreams. Each is one call of the block
function. Vector 1 is the all-zero key, the all-zero nonce, and
counter 0. Vector 2 changes the counter to 1. Vector 3 changes the
last key byte to `0x01`, with counter 1 and a zero nonce. Vector 4
uses key byte 1 equal to `0xff`, counter 2, and a zero nonce.
Vector 5 uses a zero key, counter 0, and nonce word 2 equal to 2,
whose little-endian bytes are `00 00 00 02` in the last four nonce
bytes. The tests compare the serialized blocks with those five
keystreams. They do not compare the intermediate matrices Appendix
A.1 also prints, except the §2.3.2 matrix, which is not one of the
five.

**Listing J9.3 — `block.or`**

```orange
edition 2026;
module block {

  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec setup(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[32]^16 {
    ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16
  }

  spec rounds(initial: Word[32]^16) -> Word[32]^16 {
    let (x0: Word[32], x1: Word[32], x2: Word[32], x3: Word[32],
         x4: Word[32], x5: Word[32], x6: Word[32], x7: Word[32],
         x8: Word[32], x9: Word[32], x10: Word[32], x11: Word[32],
         x12: Word[32], x13: Word[32], x14: Word[32], x15: Word[32]) =
      for i in 1..11 with (
        s0: Word[32], s1: Word[32], s2: Word[32], s3: Word[32],
        s4: Word[32], s5: Word[32], s6: Word[32], s7: Word[32],
        s8: Word[32], s9: Word[32], s10: Word[32], s11: Word[32],
        s12: Word[32], s13: Word[32], s14: Word[32], s15: Word[32]
      ) = (
        initial[0], initial[1], initial[2], initial[3],
        initial[4], initial[5], initial[6], initial[7],
        initial[8], initial[9], initial[10], initial[11],
        initial[12], initial[13], initial[14], initial[15]
      ) {
        let (c0: Word[32], c4: Word[32], c8: Word[32], c12: Word[32]) = quarter_round(s0, s4, s8, s12);
        let (c1: Word[32], c5: Word[32], c9: Word[32], c13: Word[32]) = quarter_round(s1, s5, s9, s13);
        let (c2: Word[32], c6: Word[32], c10: Word[32], c14: Word[32]) = quarter_round(s2, s6, s10, s14);
        let (c3: Word[32], c7: Word[32], c11: Word[32], c15: Word[32]) = quarter_round(s3, s7, s11, s15);
        let (d0: Word[32], d5: Word[32], d10: Word[32], d15: Word[32]) = quarter_round(c0, c5, c10, c15);
        let (d1: Word[32], d6: Word[32], d11: Word[32], d12: Word[32]) = quarter_round(c1, c6, c11, c12);
        let (d2: Word[32], d7: Word[32], d8: Word[32], d13: Word[32]) = quarter_round(c2, c7, c8, c13);
        let (d3: Word[32], d4: Word[32], d9: Word[32], d14: Word[32]) = quarter_round(c3, c4, c9, c14);
        (d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15)
      };
    [x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15]
  }

  spec feed(initial: Word[32]^16, worked: Word[32]^16) -> Word[32]^16 {
    for i in 0..16 with out: Word[32]^16 = worked { out with [i] = out[i] + initial[i] }
  }

  spec block(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[8]^64 {
    let initial: Word[32]^16 = setup(key, counter, nonce);
    feed(initial, rounds(initial)) as little Word[8]^64
  }

  spec key32() -> Word[8]^32 { hex"00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f" }
  spec nonce232() -> Word[8]^12 { hex"00 00 00 09 00 00 00 4a 00 00 00 00" }

  spec setup_words() -> Word[32]^16 { setup(key32(), 1, nonce232()) }
  spec after_rounds() -> Word[32]^16 {
    let initial: Word[32]^16 = setup_words();
    rounds(initial)
  }
  spec added_words() -> Word[32]^16 {
    let initial: Word[32]^16 = setup_words();
    feed(initial, rounds(initial))
  }
  spec serialized() -> Word[8]^64 { block(key32(), 1, nonce232()) }
  spec column_a() -> Word[32] {
    let s: Word[32]^16 = setup_words();
    let q: Quad = quarter_round(s[0], s[4], s[8], s[12]);
    q.0
  }
  spec column_c() -> Word[32] {
    let s: Word[32]^16 = setup_words();
    let q: Quad = quarter_round(s[0], s[4], s[8], s[12]);
    q.2
  }

  test "RFC 8439 2.3.2 setup word 0" { setup_words()[0] == 0x61707865 }
  test "RFC 8439 2.3.2 setup word 4" { setup_words()[4] == 0x03020100 }
  test "RFC 8439 2.3.2 setup word 12" { setup_words()[12] == 0x00000001 }
  test "RFC 8439 2.3.2 setup word 13" { setup_words()[13] == 0x09000000 }
  test "RFC 8439 2.3.2 after rounds word 0" { after_rounds()[0] == 0x837778ab }
  test "RFC 8439 2.3.2 added word 0" { added_words()[0] == 0xe4e7f110 }
  test "RFC 8439 2.3.2 added word 1" { added_words()[1] == 0x15593bd1 }
  test "RFC 8439 2.3.2 serialized" {
    serialized() == hex"10 f1 e7 e4 d1 3b 59 15 50 0f dd 1f a3 20 71 c4 c7 d1 f4 c7 33 c0 68 03 04 22 aa 9a c3 d4 6c 4e d2 82 64 46 07 9f aa 09 14 c2 d7 05 d9 8b 02 a2 b5 12 9c d1 de 16 4e b9 cb d0 83 e8 a2 50 3c 4e"
  }
  test "first column word 0" { column_a() == 0xabbaa25c }
  test "first column word 2 wraps" { column_c() == 0x6b3ca454 }
  test "RFC 8439 A.1 vector 1" {
    block(hex"00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00", 0, hex"00 00 00 00 00 00 00 00 00 00 00 00") == hex"76 b8 e0 ad a0 f1 3d 90 40 5d 6a e5 53 86 bd 28 bd d2 19 b8 a0 8d ed 1a a8 36 ef cc 8b 77 0d c7 da 41 59 7c 51 57 48 8d 77 24 e0 3f b8 d8 4a 37 6a 43 b8 f4 15 18 a1 1c c3 87 b6 69 b2 ee 65 86"
  }
  test "RFC 8439 A.1 vector 2" {
    block(hex"00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00", 1, hex"00 00 00 00 00 00 00 00 00 00 00 00") == hex"9f 07 e7 be 55 51 38 7a 98 ba 97 7c 73 2d 08 0d cb 0f 29 a0 48 e3 65 69 12 c6 53 3e 32 ee 7a ed 29 b7 21 76 9c e6 4e 43 d5 71 33 b0 74 d8 39 d5 31 ed 1f 28 51 0a fb 45 ac e1 0a 1f 4b 79 4d 6f"
  }

  test "RFC 8439 A.1 vector 3" {
    block(hex"00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 01", 1, hex"00 00 00 00 00 00 00 00 00 00 00 00") == hex"3a eb 52 24 ec f8 49 92 9b 9d 82 8d b1 ce d4 dd 83 20 25 e8 01 8b 81 60 b8 22 84 f3 c9 49 aa 5a 8e ca 00 bb b4 a7 3b da d1 92 b5 c4 2f 73 f2 fd 4e 27 36 44 c8 b3 61 25 a6 4a dd eb 00 6c 13 a0"
  }
  test "RFC 8439 A.1 vector 4" {
    block(hex"00 ff 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00", 2, hex"00 00 00 00 00 00 00 00 00 00 00 00") == hex"72 d5 4d fb f1 2e c4 4b 36 26 92 df 94 13 7f 32 8f ea 8d a7 39 90 26 5e c1 bb be a1 ae 9a f0 ca 13 b2 5a a2 6c b4 a6 48 cb 9b 9d 1b e6 5b 2c 09 24 a6 6c 54 d5 45 ec 1b 73 74 f4 87 2e 99 f0 96"
  }
  test "RFC 8439 A.1 vector 5" {
    block(hex"00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00", 0, hex"00 00 00 00 00 00 00 00 00 00 00 02") == hex"c2 c6 4d 37 8c d5 36 37 4a e2 04 b9 ef 93 3f cd 1a 8b 22 88 b3 df a4 96 72 ab 76 5b 54 ee 27 c7 8a 97 0e 0e 95 5c 14 f3 a8 8e 74 1b 97 c2 86 f7 5f 8f c2 99 e8 14 83 62 fa 19 8a 39 53 1b ed 6d"
  }
}
```

`rounds` binds sixteen words because the loop accumulator is a
tuple. The body writes every word on every iteration, including a
word the quarter round just produced. `feed` adds the initial
state back, one index at a time. `1..11` is the ten iterations of
§2.3.1.

```sh
./compiler/target/debug/orangec eval --spec column_a -
./compiler/target/debug/orangec eval --spec column_c -
./compiler/target/debug/orangec test -
```

**Expected evaluation output:**

```text
block::column_a: Word[32] = 0xabbaa25c
block::column_c: Word[32] = 0x6b3ca454
```

**Test report:**

```text
test "RFC 8439 2.3.2 setup word 0" ... ok
test "RFC 8439 2.3.2 setup word 4" ... ok
test "RFC 8439 2.3.2 setup word 12" ... ok
test "RFC 8439 2.3.2 setup word 13" ... ok
test "RFC 8439 2.3.2 after rounds word 0" ... ok
test "RFC 8439 2.3.2 added word 0" ... ok
test "RFC 8439 2.3.2 added word 1" ... ok
test "RFC 8439 2.3.2 serialized" ... ok
test "first column word 0" ... ok
test "first column word 2 wraps" ... ok
test "RFC 8439 A.1 vector 1" ... ok
test "RFC 8439 A.1 vector 2" ... ok
test "RFC 8439 A.1 vector 3" ... ok
test "RFC 8439 A.1 vector 4" ... ok
test "RFC 8439 A.1 vector 5" ... ok
15 tests: 15 passed, 0 failed
```

The status is 0. The stderr of `orangec test --stats -` on this
listing, which is an evaluator-step count under Assumption J9.5,
is:

```text
test "RFC 8439 2.3.2 setup word 0": 26 steps
test "RFC 8439 2.3.2 setup word 4": 26 steps
test "RFC 8439 2.3.2 setup word 12": 26 steps
test "RFC 8439 2.3.2 setup word 13": 26 steps
test "RFC 8439 2.3.2 after rounds word 0": 4216 steps
test "RFC 8439 2.3.2 added word 0": 4396 steps
test "RFC 8439 2.3.2 added word 1": 4396 steps
test "RFC 8439 2.3.2 serialized": 4406 steps
test "first column word 0": 73 steps
test "first column word 2 wraps": 73 steps
test "RFC 8439 A.1 vector 1": 4403 steps
test "RFC 8439 A.1 vector 2": 4403 steps
test "RFC 8439 A.1 vector 3": 4403 steps
test "RFC 8439 A.1 vector 4": 4403 steps
test "RFC 8439 A.1 vector 5": 4403 steps
total: 39679 of 1048576 steps
```

`column_a` and `column_c` are Proposition J9.6. The setup tests
are the four words §2.3.2 prints at indices 0, 4, 12, and 13. The
word after 20 rounds and the two feed-forward words are the words
Proposition J9.5 quotes from the RFC's matrices. The serialized
test is the 64-byte block §2.3.2 prints. The five Appendix A.1
tests are the five keystreams. Each of those five uses 4403 steps.
A Match on these fifteen inputs is not a Match on every key.

### J9.5 A rotation of 15

§2.1 fixes the first distance at 16. Replacing 16 by 15 is a
different function. Check does not reject it. Both distances are
literals, and both are in range for a 32-bit rotation.

**Proposition J9.7.** If the first rotation is 15, then on the
§2.1.1 input `a1` is still `0x12131415`, the rotated word is
`0x28b90998` rather than `0x51721330`, and the final `a` is
`0x59d11069`.

*Proof.* `a1` is `a + b`. It does not use a rotation. Proposition
J9.2's first line stands: `a1 = 0x12131415`. The exclusive or is
still `0x13305172`.

Rotation by 16 of that word is `0x51721330`. Rotation by 15 is
rotation by 16 followed by a right rotation of 1, because
`15 = 16 − 1` and the rotations compose. `0x51721330` is even, so
a right rotation of 1 is the integer quotient by 2:
`0x51721330 / 2 = 0x28b90998`. That is the word in place of `d1`.

The other six assignments, with this `d1` and with distances 12,
8, and 7 unchanged, are:

```text
c1 = 0x9b8d6f43 + 0x28b90998 = 0xc44678db
b1 = (0x01020304 ^ 0xc44678db) <<< 12 = 0x47bdfc54
a2 = 0x12131415 + 0x47bdfc54 = 0x59d11069
d2 = (0x28b90998 ^ 0x59d11069) <<< 8 = 0x6819f171
c2 = (0xc44678db + 0x6819f171) mod 2^32 = 0x2c606a4c
b2 = (0x47bdfc54 ^ 0x2c606a4c) <<< 7 = 0xeecb0c35
```

The sum inside `c2` is `5039483468`, and
`5039483468 − 4294967296 = 744516172 = 0x2c606a4c`. The final `a`
is `a2 = 0x59d11069`, not `0xea2a92f4`. □

The prediction, before the test runs, is that `a1` still matches
and the final `a` does not. The test that demands `0xea2a92f4` is
the §2.1.1 word. The function is not that quarter round.

**Listing J9.4 — `rot15.or`, the distance is the fault**

```orange
edition 2026;
module rot15 {
  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 15;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec a1() -> Word[32] { 0x11111111 + 0x01020304 }
  spec d_after_roll() -> Word[32] { (0x01234567 ^ 0x12131415) <<< 15 }
  spec a_final() -> Word[32] {
    let q: Quad = quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567);
    q.0
  }

  test "a before the roll" { a1() == 0x12131415 }
  test "RFC 8439 2.1.1 a" { a_final() == 0xea2a92f4 }
}
```

The only change from Listing J9.1's quarter round is `<<< 15` in
the first rotation.

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec a1 -
./compiler/target/debug/orangec eval --spec d_after_roll -
./compiler/target/debug/orangec eval --spec a_final -
./compiler/target/debug/orangec test -
```

Check is silent. Silence here means the altered quarter round was
well-formed.

**Expected evaluation output:**

```text
rot15::a1: Word[32] = 0x12131415
rot15::d_after_roll: Word[32] = 0x28b90998
rot15::a_final: Word[32] = 0x59d11069
```

**Test report:**

```text
test "a before the roll" ... ok
test "RFC 8439 2.1.1 a" ... FAILED
    left:  0x59d11069
    right: 0xea2a92f4
2 tests: 1 passed, 1 failed
```

The status of `test` is 1. Standard error is empty. `left` is the
word Proposition J9.7 derived. `right` is the word §2.1.1 prints.

### J9.6 The other byte order

§2.3's three loads are little-endian. `as big` is a different
function. Check accepts both. J4 is where the two orders are
defined. The three words below are the §2.3.2 inputs, one word at
a time.

**Proposition J9.8.** The key bytes `00 01 02 03` are the word
`0x03020100` little-endian and the word `0x00010203` big-endian.
The counter `1`, written big-endian as the bytes `00 00 00 01` and
then read little-endian, is `0x01000000`, not `0x00000001`. The
nonce bytes `00 00 00 09` are `0x09000000` little-endian and
`0x00000009` big-endian.

*Proof.* Little-endian places the first byte at the least
significant place. `00 01 02 03` is
`0x00 + 0x01 * 256 + 0x02 * 65536 + 0x03 * 16777216 = 50462976 = 0x03020100`.
Big-endian places the first byte at the most significant place:
`0x00 * 16777216 + 0x01 * 65536 + 0x02 * 256 + 0x03 = 66051 = 0x00010203`.

The integer `1` as four big-endian bytes is `00 00 00 01`. Reading
those bytes little-endian puts `0x01` in the most significant
place: `1 * 16777216 = 16777216 = 0x01000000`. The block count §2.3
wants is the integer `1`, which is `0x00000001`.

`00 00 00 09` little-endian is `9 * 16777216 = 150994944 = 0x09000000`.
Big-endian it is the integer `9`, which is `0x00000009`. □

The tests demand the three little-endian words. The listing loads
the other order. The tests are the §2.3 words. The loads are not.

**Listing J9.5 — `big_load.or`, the order is the fault**

```orange
edition 2026;
module big_load {
  spec key_word() -> Word[32] {
    let key: Word[8]^4 = hex"00 01 02 03";
    let words: Word[32]^1 = key as big Word[32]^1;
    words[0]
  }

  spec counter_word() -> Word[32] {
    let counter: Word[32] = 1;
    let bytes: Word[8]^4 = counter as big Word[8]^4;
    let words: Word[32]^1 = bytes as little Word[32]^1;
    words[0]
  }

  spec nonce_word() -> Word[32] {
    let nonce: Word[8]^4 = hex"00 00 00 09";
    let words: Word[32]^1 = nonce as big Word[32]^1;
    words[0]
  }

  test "RFC 8439 2.3 key word" { key_word() == 0x03020100 }
  test "RFC 8439 2.3 counter word" { counter_word() == 0x00000001 }
  test "RFC 8439 2.3 nonce word" { nonce_word() == 0x09000000 }
}
```

The counter is bound with `let` before `as`. A bare integer is not
an operand of `as` on this slice. J5's Listing J5.1 is that
rejection. This listing does not repeat it. The fault here is the
order, and the order is well-formed.

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec key_word -
./compiler/target/debug/orangec eval --spec counter_word -
./compiler/target/debug/orangec eval --spec nonce_word -
./compiler/target/debug/orangec test -
```

**Expected evaluation output:**

```text
big_load::key_word: Word[32] = 0x00010203
big_load::counter_word: Word[32] = 0x01000000
big_load::nonce_word: Word[32] = 0x00000009
```

**Test report:**

```text
test "RFC 8439 2.3 key word" ... FAILED
    left:  0x00010203
    right: 0x03020100
test "RFC 8439 2.3 counter word" ... FAILED
    left:  0x01000000
    right: 0x00000001
test "RFC 8439 2.3 nonce word" ... FAILED
    left:  0x00000009
    right: 0x09000000
3 tests: 0 passed, 3 failed
```

The status is 1. Standard error is empty. Each `left` is the word
Proposition J9.8 assigned to the wrong order. Each `right` is the
word §2.3 uses.

### J9.7 Diagonal indices on the first call

§2.3 runs the column indices first. The first call is
`QUARTERROUND(0, 4, 8, 12)`. The diagonal call on column 0's `a`
is `QUARTERROUND(0, 5, 10, 15)`. Those four words are not the
column quadruple. Check does not reject either call. Both are
applications of the same quarter round.

**Proposition J9.9.** On the §2.3.2 setup,
`QUARTERROUND(0, 5, 10, 15)` sends word 0 to `0xc0bf2761`, not to
`0xabbaa25c`.

*Proof.* The four words are `a = 0x61707865`, `b = 0x07060504` at
index 5, `c = 0x1b1a1918` at index 10, and `d = 0x00000000` at
index 15.

`a1 = 0x61707865 + 0x07060504 = 1752595817 = 0x68767d69`.

`d ^ a1 = 0x68767d69`, because `d` is `0`. Rotation by 16 gives
`d1 = 0x7d696876`.

`c1 = 0x1b1a1918 + 0x7d696876 = 2558755214 = 0x9883818e`.

`b ^ c1 = 0x9f85848a`. Rotation by 12 gives `b1 = 0x5848a9f8`.

`a2 = 0x68767d69 + 0x5848a9f8 = 3233752929 = 0xc0bf2761`. The sum
does not wrap. Proposition J9.6's word 0 was `0xabbaa25c`. These
are different words, so the diagonal call is not the first column
call. □

The later seventy-nine quarter rounds are not needed for the
inequality. The first call already disagrees.

**Listing J9.6 — `diag_first.or`, the indices are the fault**

```orange
edition 2026;
module diag_first {
  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec setup() -> Word[32]^16 {
    let key: Word[8]^32 = hex"00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f";
    let counter: Word[32] = 1;
    let nonce: Word[8]^12 = hex"00 00 00 09 00 00 00 4a 00 00 00 00";
    ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16
  }

  spec first_a() -> Word[32] {
    let s: Word[32]^16 = setup();
    let q: Quad = quarter_round(s[0], s[5], s[10], s[15]);
    q.0
  }

  test "RFC 8439 2.3 first column word 0" { first_a() == 0xabbaa25c }
}
```

The setup is the §2.3.2 setup. The quarter round reads indices
`0`, `5`, `10`, and `15`. The test demands the column word from
Proposition J9.6.

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec first_a -
./compiler/target/debug/orangec test -
```

**Expected evaluation output:**

```text
diag_first::first_a: Word[32] = 0xc0bf2761
```

**Test report:**

```text
test "RFC 8439 2.3 first column word 0" ... FAILED
    left:  0xc0bf2761
    right: 0xabbaa25c
1 test: 0 passed, 1 failed
```

The status is 1. Standard error is empty. `left` is Proposition
J9.9. `right` is Proposition J9.6.

### J9.8 Encryption, and how long a message may be

§2.4. ChaCha20 calls the block function with a fixed key, a fixed
nonce, and a counter that increases by one for each block. The
ciphertext is the plaintext exclusive-ored with that keystream.
Decryption is the same function: exclusive or is its own inverse,
so the same keystream takes the ciphertext back to the plaintext.
The initial counter may be any 32-bit word. §2.4 says it is
usually zero or one, and that one is the choice when the zero
block is used for something else.

**Proposition J9.10.** Fix a key and a nonce. Two blocks under
that pair have the same input state if and only if they have the
same counter. A 32-bit counter takes `2^32` distinct values, so
one pair produces at most `2^32` distinct blocks. §2.8's note 1
uses `2^32 − 1` of them. At 64 bytes a block, that note's length
is `(2^32 − 1) * 64 = 274877906880`. The length §2.3's phrase
“`2^32` blocks” names, at the same block size, is
`2^32 * 64 = 274877906944`.

*Proof.* Under §2.3 the state is determined by the constants, the
key, the counter, and the nonce. The constants do not change. With
the key and the nonce fixed, the only word that can still change
is word 12, the counter. Distinct counters are distinct states.
Equal counters are the same state. A word has `2^32` values, from
`0` through `2^32 − 1`, so there are `2^32` distinct states and at
most `2^32` distinct blocks.

§2.8, note 1, says a single invocation encrypts `2^32 − 1` blocks
of 64 bytes, because of the size of the block counter, and prints
`274,877,906,880` bytes. The note's count is one less than
`2^32`: the counters `1` through `2^32 − 1`, which is the range
left when counter `0` is not part of the message. There are
`2^32 − 1` such counters. Then

```text
(2^32 − 1) * 64 = (2^32 − 1) * 2^6 = 2^38 − 2^6
```

`2^38 = 274877906944`, and `274877906944 − 64 = 274877906880`.
Proposition J2.1 is this product. The proof of the product is that
proposition. The layout reason is the paragraph above, and it is
not in J2.

§2.3 says the same counter limits one `(key, nonce)` pair to
`2^32` blocks. That count includes every value of word 12, from
`0` through `2^32 − 1`. The byte length is `2^32 * 64 = 2^38 = 274877906944`.
The two sections print two counts. Both are read off word 12.
They differ by one block, which is 64 bytes. □

§2.3 states that the original ChaCha had a 64-bit nonce and 64-bit
block count. §2.8, note 1, says one can return to the original
design, reduce the nonce to 64 bits, and use the integer at
position 13 as the top 32 bits of a 64-bit block counter. The note
prints `1,180,591,620,717,411,303,360` bytes. That integer is
`(2^64 − 1) * 64`. It is not a value this lesson's setup computes.
The setup in the listings leaves words 13 through 15 as the nonce,
which is what §2.3 writes.

**Proposition J9.11.** `(2^64 − 1) * 64 = 1180591620717411303360`.

*Proof.* `64 = 2^6`, so `(2^64 − 1) * 64 = 2^70 − 2^6 = 2^70 − 64`.
`2^10 = 1024`, and `2^70 = 1180591620717411303424`. Subtract 64:
`1180591620717411303424 − 64 = 1180591620717411303360`. □

The §2.4.2 plaintext is 114 bytes, the sunscreen sentence. The
initial counter is 1. The nonce is
`00:00:00:00:00:00:00:4a:00:00:00:00`, which differs from the
§2.3.2 nonce in byte 3: that example used `09` where this one uses
`00`. The first plaintext byte is `0x4c`, the ASCII byte of `L`.
The keystream figure's first byte is `0x22`. Their exclusive or
is `0x4c ^ 0x22 = 0x6e`, the first ciphertext byte the section
prints.

Appendix A.2 vector 1 is 64 zero bytes under a zero key, a zero
nonce, and counter 0. Its ciphertext is Appendix A.1's first
keystream, because the plaintext is zero and exclusive or with
zero is the identity. Vector 3 is 127 bytes, counter 42, and the
nonce whose last word is 2. Vector 2 is 375 bytes, counter 1, and
that same nonce shape, with the key whose last byte is `0x01`.
375 bytes is five blocks of 64 and 55 more bytes, so the listing
pads to six blocks, 384 bytes, and keeps the first 375. 127 bytes
pads to two blocks. 114 bytes pads to two blocks.

The padding bytes are zeros. They are not part of the plaintext.
They are encrypted and then dropped by the slice. The dropped
bytes are the extra keystream exclusive-ored with zero, which is
the extra keystream. That is the discard §2.4 describes, written
with the slice this compiler has.

**Listing J9.7 — `encrypt.or`**

```orange
edition 2026;
module encrypt {

  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec setup(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[32]^16 {
    ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16
  }

  spec rounds(initial: Word[32]^16) -> Word[32]^16 {
    let (x0: Word[32], x1: Word[32], x2: Word[32], x3: Word[32],
         x4: Word[32], x5: Word[32], x6: Word[32], x7: Word[32],
         x8: Word[32], x9: Word[32], x10: Word[32], x11: Word[32],
         x12: Word[32], x13: Word[32], x14: Word[32], x15: Word[32]) =
      for i in 1..11 with (
        s0: Word[32], s1: Word[32], s2: Word[32], s3: Word[32],
        s4: Word[32], s5: Word[32], s6: Word[32], s7: Word[32],
        s8: Word[32], s9: Word[32], s10: Word[32], s11: Word[32],
        s12: Word[32], s13: Word[32], s14: Word[32], s15: Word[32]
      ) = (
        initial[0], initial[1], initial[2], initial[3],
        initial[4], initial[5], initial[6], initial[7],
        initial[8], initial[9], initial[10], initial[11],
        initial[12], initial[13], initial[14], initial[15]
      ) {
        let (c0: Word[32], c4: Word[32], c8: Word[32], c12: Word[32]) = quarter_round(s0, s4, s8, s12);
        let (c1: Word[32], c5: Word[32], c9: Word[32], c13: Word[32]) = quarter_round(s1, s5, s9, s13);
        let (c2: Word[32], c6: Word[32], c10: Word[32], c14: Word[32]) = quarter_round(s2, s6, s10, s14);
        let (c3: Word[32], c7: Word[32], c11: Word[32], c15: Word[32]) = quarter_round(s3, s7, s11, s15);
        let (d0: Word[32], d5: Word[32], d10: Word[32], d15: Word[32]) = quarter_round(c0, c5, c10, c15);
        let (d1: Word[32], d6: Word[32], d11: Word[32], d12: Word[32]) = quarter_round(c1, c6, c11, c12);
        let (d2: Word[32], d7: Word[32], d8: Word[32], d13: Word[32]) = quarter_round(c2, c7, c8, c13);
        let (d3: Word[32], d4: Word[32], d9: Word[32], d14: Word[32]) = quarter_round(c3, c4, c9, c14);
        (d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15)
      };
    [x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15]
  }

  spec feed(initial: Word[32]^16, worked: Word[32]^16) -> Word[32]^16 {
    for i in 0..16 with out: Word[32]^16 = worked { out with [i] = out[i] + initial[i] }
  }

  spec block(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[8]^64 {
    let initial: Word[32]^16 = setup(key, counter, nonce);
    feed(initial, rounds(initial)) as little Word[8]^64
  }


  spec encrypt[blocks in 1..7](
    key: Word[8]^32, counter: Word[32], nonce: Word[8]^12, plaintext: Word[8]^(64 * blocks)
  ) -> Word[8]^(64 * blocks) {
    let stream: Word[8]^(64 * blocks) =
      for j in 0..blocks with s: Word[8]^(64 * blocks) = [0; (64 * blocks)] {
        s with [64 * j..64 * j + 64] = block(key, counter + (j as Word[32]), nonce)
      };
    for i in 0..(64 * blocks) with c: Word[8]^(64 * blocks) = plaintext {
      c with [i] = c[i] ^ stream[i]
    }
  }

  spec key32() -> Word[8]^32 { hex"00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f" }
  spec nonce242() -> Word[8]^12 { hex"00 00 00 00 00 00 00 4a 00 00 00 00" }
  spec sunscreen() -> Word[8]^114 {
    "Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
  }
  spec sunscreen_ct() -> Word[8]^114 {
    let padded: Word[8]^128 = sunscreen() ++ [0; 14];
    let full: Word[8]^128 = encrypt(key32(), 1, nonce242(), padded);
    full[..114]
  }
  spec opened() -> Word[8]^114 {
    let padded: Word[8]^128 = sunscreen_ct() ++ [0; 14];
    let full: Word[8]^128 = encrypt(key32(), 1, nonce242(), padded);
    full[..114]
  }

  test "RFC 8439 2.4.2" {
    sunscreen_ct() == hex"6e 2e 35 9a 25 68 f9 80 41 ba 07 28 dd 0d 69 81 e9 7e 7a ec 1d 43 60 c2 0a 27 af cc fd 9f ae 0b f9 1b 65 c5 52 47 33 ab 8f 59 3d ab cd 62 b3 57 16 39 d6 24 e6 51 52 ab 8f 53 0c 35 9f 08 61 d8 07 ca 0d bf 50 0d 6a 61 56 a3 8e 08 8a 22 b6 5e 52 bc 51 4d 16 cc f8 06 81 8c e9 1a b7 79 37 36 5a f9 0b bf 74 a3 5b e6 b4 0b 8e ed f2 78 5e 42 87 4d"
  }
  test "decrypt is encrypt again" { opened() == sunscreen() }

  spec a2_one() -> Word[8]^64 {
    encrypt(hex"00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00", 0, hex"00 00 00 00 00 00 00 00 00 00 00 00", [0; 64])
  }
  test "RFC 8439 A.2 vector 1" {
    a2_one() == hex"76 b8 e0 ad a0 f1 3d 90 40 5d 6a e5 53 86 bd 28 bd d2 19 b8 a0 8d ed 1a a8 36 ef cc 8b 77 0d c7 da 41 59 7c 51 57 48 8d 77 24 e0 3f b8 d8 4a 37 6a 43 b8 f4 15 18 a1 1c c3 87 b6 69 b2 ee 65 86"
  }


  spec a2_three() -> Word[8]^127 {
    let pt: Word[8]^127 =
    hex"27 54 77 61 73 20 62 72 69 6c 6c 69 67 2c 20 61" ++
    hex"6e 64 20 74 68 65 20 73 6c 69 74 68 79 20 74 6f" ++
    hex"76 65 73 0a 44 69 64 20 67 79 72 65 20 61 6e 64" ++
    hex"20 67 69 6d 62 6c 65 20 69 6e 20 74 68 65 20 77" ++
    hex"61 62 65 3a 0a 41 6c 6c 20 6d 69 6d 73 79 20 77" ++
    hex"65 72 65 20 74 68 65 20 62 6f 72 6f 67 6f 76 65" ++
    hex"73 2c 0a 41 6e 64 20 74 68 65 20 6d 6f 6d 65 20" ++
    hex"72 61 74 68 73 20 6f 75 74 67 72 61 62 65 2e";
    let padded: Word[8]^128 = pt ++ [0; 1];
    let full: Word[8]^128 = encrypt(
      hex"1c 92 40 a5 eb 55 d3 8a f3 33 88 86 04 f6 b5 f0 47 39 17 c1 40 2b 80 09 9d ca 5c bc 20 70 75 c0",
      42,
      hex"00 00 00 00 00 00 00 00 00 00 00 02",
      padded
    );
    full[..127]
  }
  spec a2_two() -> Word[8]^375 {
    let pt: Word[8]^375 =
    hex"41 6e 79 20 73 75 62 6d 69 73 73 69 6f 6e 20 74" ++
    hex"6f 20 74 68 65 20 49 45 54 46 20 69 6e 74 65 6e" ++
    hex"64 65 64 20 62 79 20 74 68 65 20 43 6f 6e 74 72" ++
    hex"69 62 75 74 6f 72 20 66 6f 72 20 70 75 62 6c 69" ++
    hex"63 61 74 69 6f 6e 20 61 73 20 61 6c 6c 20 6f 72" ++
    hex"20 70 61 72 74 20 6f 66 20 61 6e 20 49 45 54 46" ++
    hex"20 49 6e 74 65 72 6e 65 74 2d 44 72 61 66 74 20" ++
    hex"6f 72 20 52 46 43 20 61 6e 64 20 61 6e 79 20 73" ++
    hex"74 61 74 65 6d 65 6e 74 20 6d 61 64 65 20 77 69" ++
    hex"74 68 69 6e 20 74 68 65 20 63 6f 6e 74 65 78 74" ++
    hex"20 6f 66 20 61 6e 20 49 45 54 46 20 61 63 74 69" ++
    hex"76 69 74 79 20 69 73 20 63 6f 6e 73 69 64 65 72" ++
    hex"65 64 20 61 6e 20 22 49 45 54 46 20 43 6f 6e 74" ++
    hex"72 69 62 75 74 69 6f 6e 22 2e 20 53 75 63 68 20" ++
    hex"73 74 61 74 65 6d 65 6e 74 73 20 69 6e 63 6c 75" ++
    hex"64 65 20 6f 72 61 6c 20 73 74 61 74 65 6d 65 6e" ++
    hex"74 73 20 69 6e 20 49 45 54 46 20 73 65 73 73 69" ++
    hex"6f 6e 73 2c 20 61 73 20 77 65 6c 6c 20 61 73 20" ++
    hex"77 72 69 74 74 65 6e 20 61 6e 64 20 65 6c 65 63" ++
    hex"74 72 6f 6e 69 63 20 63 6f 6d 6d 75 6e 69 63 61" ++
    hex"74 69 6f 6e 73 20 6d 61 64 65 20 61 74 20 61 6e" ++
    hex"79 20 74 69 6d 65 20 6f 72 20 70 6c 61 63 65 2c" ++
    hex"20 77 68 69 63 68 20 61 72 65 20 61 64 64 72 65" ++
    hex"73 73 65 64 20 74 6f";
    let padded: Word[8]^384 = pt ++ [0; 9];
    let full: Word[8]^384 = encrypt(
      hex"00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 01",
      1,
      hex"00 00 00 00 00 00 00 00 00 00 00 02",
      padded
    );
    full[..375]
  }
  test "RFC 8439 A.2 vector 3" {
    a2_three() == (
    hex"62 e6 34 7f 95 ed 87 a4 5f fa e7 42 6f 27 a1 df" ++
    hex"5f b6 91 10 04 4c 0d 73 11 8e ff a9 5b 01 e5 cf" ++
    hex"16 6d 3d f2 d7 21 ca f9 b2 1e 5f b1 4c 61 68 71" ++
    hex"fd 84 c5 4f 9d 65 b2 83 19 6c 7f e4 f6 05 53 eb" ++
    hex"f3 9c 64 02 c4 22 34 e3 2a 35 6b 3e 76 43 12 a6" ++
    hex"1a 55 32 05 57 16 ea d6 96 25 68 f8 7d 3f 3f 77" ++
    hex"04 c6 a8 d1 bc d1 bf 4d 50 d6 15 4b 6d a7 31 b1" ++
    hex"87 b5 8d fd 72 8a fa 36 75 7a 79 7a c1 88 d1"
    )
  }
  test "RFC 8439 A.2 vector 2" {
    a2_two() == (
    hex"a3 fb f0 7d f3 fa 2f de 4f 37 6c a2 3e 82 73 70" ++
    hex"41 60 5d 9f 4f 4f 57 bd 8c ff 2c 1d 4b 79 55 ec" ++
    hex"2a 97 94 8b d3 72 29 15 c8 f3 d3 37 f7 d3 70 05" ++
    hex"0e 9e 96 d6 47 b7 c3 9f 56 e0 31 ca 5e b6 25 0d" ++
    hex"40 42 e0 27 85 ec ec fa 4b 4b b5 e8 ea d0 44 0e" ++
    hex"20 b6 e8 db 09 d8 81 a7 c6 13 2f 42 0e 52 79 50" ++
    hex"42 bd fa 77 73 d8 a9 05 14 47 b3 29 1c e1 41 1c" ++
    hex"68 04 65 55 2a a6 c4 05 b7 76 4d 5e 87 be a8 5a" ++
    hex"d0 0f 84 49 ed 8f 72 d0 d6 62 ab 05 26 91 ca 66" ++
    hex"42 4b c8 6d 2d f8 0e a4 1f 43 ab f9 37 d3 25 9d" ++
    hex"c4 b2 d0 df b4 8a 6c 91 39 dd d7 f7 69 66 e9 28" ++
    hex"e6 35 55 3b a7 6c 5c 87 9d 7b 35 d4 9e b2 e6 2b" ++
    hex"08 71 cd ac 63 89 39 e2 5e 8a 1e 0e f9 d5 28 0f" ++
    hex"a8 ca 32 8b 35 1c 3c 76 59 89 cb cf 3d aa 8b 6c" ++
    hex"cc 3a af 9f 39 79 c9 2b 37 20 fc 88 dc 95 ed 84" ++
    hex"a1 be 05 9c 64 99 b9 fd a2 36 e7 e8 18 b0 4b 0b" ++
    hex"c3 9c 1e 87 6b 19 3b fe 55 69 75 3f 88 12 8c c0" ++
    hex"8a aa 9b 63 d1 a1 6f 80 ef 25 54 d7 18 9c 41 1f" ++
    hex"58 69 ca 52 c5 b8 3f a3 6f f2 16 b9 c1 d3 00 62" ++
    hex"be bc fd 2d c5 bc e0 91 19 34 fd a7 9a 86 f6 e6" ++
    hex"98 ce d7 59 c3 ff 9b 64 77 33 8f 3d a4 f9 cd 85" ++
    hex"14 ea 99 82 cc af b3 41 b2 38 4d d9 02 f3 d1 ab" ++
    hex"7a c6 1d d2 9c 6f 21 ba 5b 86 2f 37 30 e3 7c fd" ++
    hex"c4 fd 80 6c 22 f2 21"
    )
  }
}
```

`encrypt` is sized by `blocks in 1..7`, six instances. `j as Word[32]`
is the loop index converted to a word before it is added to the
counter. The index of a `for` is not already a `Word[32]`. The
sunscreen string is 114 bytes. Fourteen zero bytes make 128. The
result is cut at 114. Decrypting calls `encrypt` again. Appendix
A.2 vector 2 is the 375-byte cut of a 384-byte encryption.

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec test -
```

Check is silent.

**Test report:**

```text
test "RFC 8439 2.4.2" ... ok
test "decrypt is encrypt again" ... ok
test "RFC 8439 A.2 vector 1" ... ok
test "RFC 8439 A.2 vector 3" ... ok
test "RFC 8439 A.2 vector 2" ... ok
5 tests: 5 passed, 0 failed
```

The status is 0. The stderr of `orangec test --stats -` on this
listing, which is an evaluator-step count under Assumption J9.5,
is:

```text
test "RFC 8439 2.4.2": 10400 steps
test "decrypt is encrypt again": 20796 steps
test "RFC 8439 A.2 vector 1": 5134 steps
test "RFC 8439 A.2 vector 3": 10434 steps
test "RFC 8439 A.2 vector 2": 32934 steps
total: 79698 of 1048576 steps
```

The first test is the §2.4.2 ciphertext. The second is decryption
on that ciphertext. The other three are Appendix A.2 vectors 1, 3,
and 2, in the order the listing writes them. Vector 2 is the long
one, at 32934 steps. These five Matches are five inputs. They are
not a statement about every plaintext under `2^32` blocks.

### J9.9 The counter off by one

§2.4.2 sets the initial counter to 1. The first ciphertext byte is
`0x6e`, from `0x4c ^ 0x22`, as §J9.8 computed. Counter `0` is a
different word 12. By Proposition J9.10 it is a different block.
The test still demands `0x6e`.

The proposition predicts the failure. It does not predict the byte
the wrong counter produces. That byte depends on 20 rounds.
Assumption J9.6 keeps those rounds in the program. The evaluation
prints the byte. The report's `left` is that byte, and `right` is
`0x6e`.

**Listing J9.8 — `counter_off.or`, the counter is the fault**

```orange
edition 2026;
module counter_off {
  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec setup(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[32]^16 {
    ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16
  }

  spec rounds(initial: Word[32]^16) -> Word[32]^16 {
    let (x0: Word[32], x1: Word[32], x2: Word[32], x3: Word[32],
         x4: Word[32], x5: Word[32], x6: Word[32], x7: Word[32],
         x8: Word[32], x9: Word[32], x10: Word[32], x11: Word[32],
         x12: Word[32], x13: Word[32], x14: Word[32], x15: Word[32]) =
      for i in 1..11 with (
        s0: Word[32], s1: Word[32], s2: Word[32], s3: Word[32],
        s4: Word[32], s5: Word[32], s6: Word[32], s7: Word[32],
        s8: Word[32], s9: Word[32], s10: Word[32], s11: Word[32],
        s12: Word[32], s13: Word[32], s14: Word[32], s15: Word[32]
      ) = (
        initial[0], initial[1], initial[2], initial[3],
        initial[4], initial[5], initial[6], initial[7],
        initial[8], initial[9], initial[10], initial[11],
        initial[12], initial[13], initial[14], initial[15]
      ) {
        let (c0: Word[32], c4: Word[32], c8: Word[32], c12: Word[32]) = quarter_round(s0, s4, s8, s12);
        let (c1: Word[32], c5: Word[32], c9: Word[32], c13: Word[32]) = quarter_round(s1, s5, s9, s13);
        let (c2: Word[32], c6: Word[32], c10: Word[32], c14: Word[32]) = quarter_round(s2, s6, s10, s14);
        let (c3: Word[32], c7: Word[32], c11: Word[32], c15: Word[32]) = quarter_round(s3, s7, s11, s15);
        let (d0: Word[32], d5: Word[32], d10: Word[32], d15: Word[32]) = quarter_round(c0, c5, c10, c15);
        let (d1: Word[32], d6: Word[32], d11: Word[32], d12: Word[32]) = quarter_round(c1, c6, c11, c12);
        let (d2: Word[32], d7: Word[32], d8: Word[32], d13: Word[32]) = quarter_round(c2, c7, c8, c13);
        let (d3: Word[32], d4: Word[32], d9: Word[32], d14: Word[32]) = quarter_round(c3, c4, c9, c14);
        (d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15)
      };
    [x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15]
  }

  spec block(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[8]^64 {
    let initial: Word[32]^16 = setup(key, counter, nonce);
    let worked: Word[32]^16 = rounds(initial);
    let added: Word[32]^16 = for i in 0..16 with out: Word[32]^16 = worked { out with [i] = out[i] + initial[i] };
    added as little Word[8]^64
  }

  spec key32() -> Word[8]^32 { hex"00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f" }
  spec nonce242() -> Word[8]^12 { hex"00 00 00 00 00 00 00 4a 00 00 00 00" }
  spec stream_byte() -> Word[8] { block(key32(), 0, nonce242())[0] }
  spec first_byte() -> Word[8] { 0x4c ^ stream_byte() }

  test "RFC 8439 2.4.2 first ciphertext byte" { first_byte() == 0x6e }
}
```

The call is `block(key32(), 0, nonce242())`. The sunscreen listing
uses counter `1` and the same key and nonce. The first plaintext
byte is the integer `0x4c`, written as a word so the exclusive or
has two operands of one type.

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec first_byte -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
counter_off::first_byte: Word[8] = 0xe3
```

**Test report:**

```text
test "RFC 8439 2.4.2 first ciphertext byte" ... FAILED
    left:  0xe3
    right: 0x6e
1 test: 0 passed, 1 failed
```

The status is 1. Standard error is empty. `0xe3` is the program's
byte. It is not a hand residue of the eighty quarter rounds.

### J9.10 One nonce, two plaintexts

§2.3 says words 13 through 15 must not be repeated for the same
key. The sentence is a requirement on the inputs. The algebra
below says what the ciphertext equation does when the requirement
is ignored. It does not build a nonce, and it does not search for
one.

**Proposition J9.12.** Let `k` be one keystream block. Let `p1` and
`p2` be plaintexts of equal length, and let `c1 = p1 ^ k` and
`c2 = p2 ^ k`, exclusive or in each byte. Then `c1 ^ c2 = p1 ^ p2`.
The keystream cancels. The key does not appear in the result.

*Proof.* Exclusive or is associative and commutative, and
`k ^ k = 0`, and `x ^ 0 = x`. So

```text
c1 ^ c2 = (p1 ^ k) ^ (p2 ^ k) = p1 ^ p2 ^ (k ^ k) = p1 ^ p2
```

The key was an input of the block function that produced `k`. It
is not a factor of `p1 ^ p2`. If `p1` is known, then
`p2 = p1 ^ (c1 ^ c2)`, which is `p2` and not the key. □

The listing takes four bytes `00 00 00 00` and four bytes
`01 02 03 04`, one key, the §2.4.2 nonce, and counter `1` for
both. Then `p1 ^ p2 = 01 02 03 04`. A test that expects four zero
bytes is the claim that the plaintexts were equal. They were not.
The test that expects `p1 ^ p2` is Proposition J9.12. That Match
shows the cancellation on these four bytes. It does not show that
a key was recovered, because the proposition says the key is not
in the result.

**Listing J9.9 — `nonce_reuse.or`**

```orange
edition 2026;
module nonce_reuse {
  type Quad = (Word[32], Word[32], Word[32], Word[32]);

  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Quad {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    let a2: Word[32] = a1 + b1;
    let d2: Word[32] = (d1 ^ a2) <<< 8;
    let c2: Word[32] = c1 + d2;
    let b2: Word[32] = (b1 ^ c2) <<< 7;
    (a2, b2, c2, d2)
  }

  spec block(key: Word[8]^32, counter: Word[32], nonce: Word[8]^12) -> Word[8]^64 {
    let initial: Word[32]^16 =
      ("expand 32-byte k" ++ key ++ (counter as little Word[8]^4) ++ nonce) as little Word[32]^16;
    let (x0: Word[32], x1: Word[32], x2: Word[32], x3: Word[32],
         x4: Word[32], x5: Word[32], x6: Word[32], x7: Word[32],
         x8: Word[32], x9: Word[32], x10: Word[32], x11: Word[32],
         x12: Word[32], x13: Word[32], x14: Word[32], x15: Word[32]) =
      for i in 1..11 with (
        s0: Word[32], s1: Word[32], s2: Word[32], s3: Word[32],
        s4: Word[32], s5: Word[32], s6: Word[32], s7: Word[32],
        s8: Word[32], s9: Word[32], s10: Word[32], s11: Word[32],
        s12: Word[32], s13: Word[32], s14: Word[32], s15: Word[32]
      ) = (
        initial[0], initial[1], initial[2], initial[3],
        initial[4], initial[5], initial[6], initial[7],
        initial[8], initial[9], initial[10], initial[11],
        initial[12], initial[13], initial[14], initial[15]
      ) {
        let (c0: Word[32], c4: Word[32], c8: Word[32], c12: Word[32]) = quarter_round(s0, s4, s8, s12);
        let (c1: Word[32], c5: Word[32], c9: Word[32], c13: Word[32]) = quarter_round(s1, s5, s9, s13);
        let (c2: Word[32], c6: Word[32], c10: Word[32], c14: Word[32]) = quarter_round(s2, s6, s10, s14);
        let (c3: Word[32], c7: Word[32], c11: Word[32], c15: Word[32]) = quarter_round(s3, s7, s11, s15);
        let (d0: Word[32], d5: Word[32], d10: Word[32], d15: Word[32]) = quarter_round(c0, c5, c10, c15);
        let (d1: Word[32], d6: Word[32], d11: Word[32], d12: Word[32]) = quarter_round(c1, c6, c11, c12);
        let (d2: Word[32], d7: Word[32], d8: Word[32], d13: Word[32]) = quarter_round(c2, c7, c8, c13);
        let (d3: Word[32], d4: Word[32], d9: Word[32], d14: Word[32]) = quarter_round(c3, c4, c9, c14);
        (d0, d1, d2, d3, d4, d5, d6, d7, d8, d9, d10, d11, d12, d13, d14, d15)
      };
    let worked: Word[32]^16 = [x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, x12, x13, x14, x15];
    let added: Word[32]^16 = for k in 0..16 with out: Word[32]^16 = worked { out with [k] = out[k] + initial[k] };
    added as little Word[8]^64
  }

  spec key32() -> Word[8]^32 { hex"00 01 02 03 04 05 06 07 08 09 0a 0b 0c 0d 0e 0f 10 11 12 13 14 15 16 17 18 19 1a 1b 1c 1d 1e 1f" }
  spec nonce242() -> Word[8]^12 { hex"00 00 00 00 00 00 00 4a 00 00 00 00" }
  spec p1() -> Word[8]^4 { hex"00 00 00 00" }
  spec p2() -> Word[8]^4 { hex"01 02 03 04" }

  spec c1() -> Word[8]^4 {
    let ks: Word[8]^64 = block(key32(), 1, nonce242());
    [p1()[0] ^ ks[0], p1()[1] ^ ks[1], p1()[2] ^ ks[2], p1()[3] ^ ks[3]]
  }
  spec c2() -> Word[8]^4 {
    let ks: Word[8]^64 = block(key32(), 1, nonce242());
    [p2()[0] ^ ks[0], p2()[1] ^ ks[1], p2()[2] ^ ks[2], p2()[3] ^ ks[3]]
  }
  spec cipher_xor() -> Word[8]^4 {
    [c1()[0] ^ c2()[0], c1()[1] ^ c2()[1], c1()[2] ^ c2()[2], c1()[3] ^ c2()[3]]
  }
  spec plain_xor() -> Word[8]^4 {
    [p1()[0] ^ p2()[0], p1()[1] ^ p2()[1], p1()[2] ^ p2()[2], p1()[3] ^ p2()[3]]
  }

  test "ciphertext XOR equals plaintext XOR" { cipher_xor() == plain_xor() }
  test "ciphertext XOR is four zero bytes" { cipher_xor() == hex"00 00 00 00" }
}
```

Both ciphertexts call `block` with counter `1` and `nonce242`.
The second test demands four zero bytes. The plaintexts differ in
every byte.

```sh
./compiler/target/debug/orangec check -
./compiler/target/debug/orangec eval --spec cipher_xor -
./compiler/target/debug/orangec test -
```

Check is silent.

**Expected evaluation output:**

```text
nonce_reuse::cipher_xor: Word[8]^4 = [0x01, 0x02, 0x03, 0x04]
```

**Test report:**

```text
test "ciphertext XOR equals plaintext XOR" ... ok
test "ciphertext XOR is four zero bytes" ... FAILED
    left:  [0x01, 0x02, 0x03, 0x04]
    right: [0x00, 0x00, 0x00, 0x00]
    first difference at [0]
2 tests: 1 passed, 1 failed
```

The status is 1. Standard error is empty. The passing test is
Proposition J9.12 on these four bytes. `left` on the failing test
is `p1 ^ p2`. `right` is the zero block the plaintexts are not.

### J9.11 What this slice does not denote

Two bounds in the compiler are the reason the message-length
statement stays an integer.

An array length on this slice is a decimal integer from 1 through
65536. A word array of `(2^32 − 1) * 64` bytes is not a value the
listing can build. The length in Proposition J9.10 is the integer
the RFC prints. It is not an array in Listing J9.7.

A size parameter `n in a..b` takes each integer from `a` up to,
but not including, `b`, and a function has at most 256 instances.
Appendix A.2 vector 2 is 375 bytes. A size parameter that ran from
1 through 375 would be 375 instances. Listing J9.7 does not do
that. It uses six block sizes and a slice. The rejected listing
below is the 375-instance form, so the diagnostic is on the page.

**Listing J9.10 — `too_wide.or`, intentionally rejected**

```orange
edition 2026;
module too_wide {
  spec msg() -> Word[8]^65537 { [0; 65537] }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0221]: an array length must be a decimal integer from 1 through 65536
 --> <stdin>:3:25
  |
3 |   spec msg() -> Word[8]^65537 { [0; 65537] }
  |                         ^^^^^ unsupported array length
  = note: write the length in decimal without leading zeros, as in `Word[32]^16`
```

The status is 1. Standard output is empty. `eval` and `test`
print the same diagnostic and no value. The locus is the length
`65537`. One more than 65536 is the length the note refuses.

**Listing J9.11 — `too_many.or`, intentionally rejected**

```orange
edition 2026;
module too_many {
  spec encrypt[len in 1..376](plaintext: Word[8]^len) -> Word[8]^len { plaintext }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0238]: `encrypt` has 375 instances, but a function has at most 256
 --> <stdin>:3:16
  |
3 |   spec encrypt[len in 1..376](plaintext: Word[8]^len) -> Word[8]^len { plaintext }
  |                ^^^^^^^^^^^^^ too many instances
  = note: a size parameter `n in a..b` takes each value from a up to, but not including, b, with a < b <= 65536, and a function has at most 256 instances
```

The status is 1. Standard output is empty. `eval` and `test`
print the same diagnostic and no value. `1..376` is 375 values.
The note states the cap of 256 instances. Listing J9.7's range
`1..7` is six instances, which is under the cap, and `6 * 64 = 384`,
which is enough bytes for the 375-byte vector after the padding
§J9.8 describes.

The original 64-bit counter is a different initialization: word 13
would join the counter, and the nonce would be two words. §2.3
does not write that initialization. This lesson does not write it
either. Proposition J9.11 is the byte count §2.8 prints for that
design. The listing that would build the state is not on this
page.

Poly1305 and the AEAD are the rest of the RFC. They are not
quarter rounds. They are not in these eleven listings.

### J9.12 Exercises

**Exercise J9.1.** For the §2.1.1 input, give `a1`, the integer sum
inside `c2` before reduction, and `c2`.

**Exercise J9.2.** From `a1 = 0x12131415` and `d1 = 0x51721330`, with
`b = 0x01020304` still the input `b`, recover `a` and `d` by the
first double-step's inverse. Do not run the other six inverse lines.

**Exercise J9.3.** For the §2.2.1 quarter round, give the integer
sum inside `c1` before reduction, the residue `c1`, and the value
of index 0 after the call.

**Exercise J9.4.** Word 0 of `Π(S)` on the §2.3.2 example is
`0x837778ab`. Word 0 of `f(S)` is `0xe4e7f110`. Say why applying
the inverse of `Π` to the block output does not return the initial
state, and give the integer `0x61707865 + 0x837778ab`.

**Exercise J9.5.** The first rotation is 15. Give the word in place
of `d1`, and the final `a`, on the §2.1.1 input.

**Exercise J9.6.** Give the three words Listing J9.5 computes for
the key bytes `00 01 02 03`, the counter `1` written big-endian and
read little-endian, and the nonce bytes `00 00 00 09` read
big-endian. Give the three words §2.3 uses instead.

**Exercise J9.7.** Give `(2^32 − 1) * 64`, `2^32 * 64`, and
`(2^64 − 1) * 64`. Say which of the first two §2.3 names, and
which §2.8 note 1 names.

**Exercise J9.8.** Two plaintexts are the four bytes `00 00 00 00`
and the four bytes `01 02 03 04`, encrypted with one keystream.
Give `c1 ^ c2`. Say whether that value contains the key.

## Worked answers

**J9.1.** `a1 = 0x11111111 + 0x01020304 = 303240213 = 0x12131415`.
The sum inside `c2` is `0xecff8273 + 0x5881c4bb = 5461067566`.
`5461067566 − 4294967296 = 1166100270 = 0x4581472e`, so
`c2 = 0x4581472e`. Proposition J9.2 is this calculation. The sum
is the one wrap in the §2.1.1 quarter round.

**J9.2.** `(d1 >>> 16) ^ a1 = 0x13305172 ^ 0x12131415 = 0x01234567`,
which is `d`. `a1 − b = 0x12131415 − 0x01020304 = 0x11111111`,
which is `a`. Those two lines are the inverse of the first
double-step in Proposition J9.1. The other six lines are not
required for this pair.

**J9.3.** `0x53372767 + 0xc47446a0 = 4692078087`.
`4692078087 − 4294967296 = 397110791 = 0x17ab6e07`, so
`c1 = 0x17ab6e07`. Index 0 is not an index of
`QUARTERROUND(2, 7, 8, 13)`, so it stays `0x879531e0`. Proposition
J9.3 is this calculation.

**J9.4.** The inverse of `Π` returns `S` from `Π(S)`, by
Proposition J9.4. The block output is `f(S) = S + Π(S)`, not
`Π(S)`. Word 0 of those two states differs:
`0x837778ab` is not `0xe4e7f110`. Proposition J9.5 says the inverse
returns `S` from `f(S)` only when `S = 0`, and word 0 of the
initial state is `0x61707865`. The sum is
`0x61707865 + 0x837778ab = 3840405776 = 0xe4e7f110`, and it does
not cross `2^32`. The answer does not say that `f` has no inverse
of any kind.

**J9.5.** The exclusive or is still `0x13305172`. Rotation by 15
is `0x28b90998`. The final `a` is `0x59d11069`. Proposition J9.7
is this calculation. `a1` is still `0x12131415`, because the
rotation is not in that sum.

**J9.6.** The listing's three words are `0x00010203`,
`0x01000000`, and `0x00000009`. §2.3 uses `0x03020100`,
`0x00000001`, and `0x09000000`. Proposition J9.8 is the
place-value argument. The tests fail because they demand the
second triple and the program computes the first.

**J9.7.** `(2^32 − 1) * 64 = 274877906880`.
`2^32 * 64 = 274877906944`.
`(2^64 − 1) * 64 = 1180591620717411303360`.
§2.3 names `2^32` blocks, which is the second integer at 64 bytes
each. §2.8 note 1 names `2^32 − 1` blocks and prints the first
integer. The third integer is Proposition J9.11, the figure §2.8
prints for the 64-bit counter. Proposition J2.1 is the first
product. This answer does not rebuild J2's comparison of RFC 7539
with RFC 8439.

**J9.8.** `c1 ^ c2 = p1 ^ p2 = 01 02 03 04`, by Proposition J9.12.
The key is not in that value. The keystream cancelled. Knowing one
plaintext recovers the other plaintext, not the key.

```text
j9-ledger
a1-sum = 303240213
d-xor = 321933682
d1 = 1366430512
c2-sum = 5461067566
c2 = 1166100270
state-c-sum = 4692078087
state-c = 397110791
column-c-sum = 6094103636
column-c = 1799136340
column-a = 2881135196
feed0 = 3840405776
feed1-sum = 4653136849
feed1 = 358169553
rot15-d = 683215256
rot15-a = 1506873449
diag-a = 3233752929
key-le = 50462976
key-be = 66051
nonce-le = 150994944
nonce-be = 9
counter-swapped = 16777216
plain0 = 76
stream0 = 34
cipher0 = 110
blocks = 4294967295
block-bytes = 64
p-max = 274877906880
p-full = 274877906944
orig-bytes = 1180591620717411303360
reuse = 16909060
```

The ledger is the integers in the proofs. `a1-sum` is Proposition
J9.2's first sum. `c2-sum` and `c2` are the wrap in that
proposition. `state-c-sum` is Proposition J9.3. `column-c-sum`,
`column-c`, and `column-a` are Proposition J9.6. `feed0` and
`feed1` are Proposition J9.5. `rot15-d` and `rot15-a` are
Proposition J9.7. `diag-a` is Proposition J9.9. `key-le` through
`counter-swapped` are Proposition J9.8. `plain0`, `stream0`, and
`cipher0` are the first-byte exclusive or in §J9.8. `blocks` is
`2^32 − 1`. `p-max` is Proposition J9.10's §2.8 count. `p-full`
is the §2.3 count. `orig-bytes` is Proposition J9.11. `reuse` is
the integer `0x01020304` in Proposition J9.12.

## Sources and epigraph record

The quotation is the borrowed wording. The proofs are the lesson's.
The test-vector bytes are the RFC's.

**[S20] Yoav Nir and Adam Langley.** RFC 8439, *ChaCha20 and
Poly1305 for IETF Protocols*, June 2018. The header line is
`Obsoletes: 7539`. The epigraph is the sentence in §1 that begins
“The purpose of this document”. The sentences contain no inner
quotation marks. Wording was checked on 2026-10-23 against the
plain-text file whose SHA-256 is the digest J2 recorded in [T9].
No translation is involved. [S13] is J2's record for the Abstract
sentence that begins “RFC 7539, the predecessor of this document”.
This record is [S20] so that the epigraph tag is not a second
definition of [S13]. Sections used for the derivation, and not as
the epigraph, are §1.1, §2.1, §2.1.1, §2.2, §2.2.1, §2.3, §2.3.1,
§2.3.2, §2.4, §2.4.1, and §2.4.2. Appendix A.1 and Appendix A.2
supply the keystreams and ciphertexts the listings test. §2.5 and
§2.8 are named. §2.5 is not transcribed. §2.8 note 1 is the
length sentence in Proposition J9.10 and the return to a 64-bit
counter at position 13 before Proposition J9.11. Section 4 is
named and not transcribed.
This record's tag is [S20].

Source: <https://www.rfc-editor.org/rfc/rfc8439>

**[T16] Retrieved file.** This lesson uses the June 2018 plain-text
file of RFC 8439. J2 retrieved it on 2026-10-05 and recorded the
SHA-256 in [T9]:
`25bef70fbf7a07ff45c2fe4cb7c6ce954eac687413d8610603268b4e4415324c`.
The copy checked on 2026-10-23 has that digest. This lesson does
not claim a second download. The quarter-round vector, the state
vector, the block matrices, the sunscreen ciphertext, the five
Appendix A.1 keystreams, and Appendix A.2 vectors 1, 2, and 3 are
copied from that file.
This record's tag is [T16].

Source: <https://www.rfc-editor.org/rfc/rfc8439.txt>

**[C10] ChaCha surface.** The listings use `edition 2026`,
`module`, `spec`, `let`, `type`, `Word[8]`, `Word[32]`, `^`, `+`,
`-`, `<<<`, `>>>`, `++`, `hex"00"`, a string literal, `[0; n]`,
`for`, `with`, a slice `[..n]`, a range update, `as little`,
`as big`, a tuple, a sized parameter `blocks in 1..7`, and `test`.
`as little` and `as big` are implemented on the S3u binary and are
specified by OEP-0017, which this lesson does not accept. An
integer literal is not an operand of `as` until a `let` types it.
A `for` bound in these listings is a literal or a size parameter,
and the end is exclusive. The body of the round loop does not read
the index. A function has at most 256 instances. An array length
is at most 65536. There is no `floor`. A partial block is a slice
after a padded encryption. The version line these commands print
is `orangec 0.0.1 (Orange edition 2026; implemented slice S3u)`.
This record's tag is [C10].

The test form is the one N12 recorded as [T6]. `orangec check`
checks a test and does not run it. `orangec eval --spec` runs one
parameterless spec and does not run tests. `orangec test` runs the
tests. A failed check prints the diagnostic and no report. A failed
test prints `left` and `right` and no diagnostic.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

J9 adds eight exercises with worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`. The Orange listings
are the eleven fenced programs in this file.
`compiler/crates/orangec/tests/book_j9.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not prove
Assumption J9.2 from a smaller theory of the compiler, and they do
not accept OEP-0017. Reaching the last page is not a certificate.
Proposition J9.5 does not say that the feed-forward sum has no
inverse of any kind. It says the inverse of the 20-round
permutation does not read that sum.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-23 at the
owner's direction. Owner review is pending.
