# The Orange Book

By Chase Bryan

## Part 1, The Novice

N12: The First Complete Study. Draft 2026-10-05.

Continue from [Count What You Do Not Know](NOVICE_PROBABILITY.md#n10-count-what-you-do-not-know).
Lesson N11, Protect More Than Appearance, sits between that lesson and
this one in the teaching order. Its intended anchor is
`n11-protect-more-than-appearance`. This lesson does not depend on N11's
text. It uses the quarter round from
[Name the Intermediate Step](NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md#n7-name-the-intermediate-step)
and the reading habit from
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).

This lesson is **N12**. It is not a manuscript chapter numeral. The
original manuscript keeps its own numbers. When the text says “Chapter 6”
or “§2.1”, the first is a novice chapter and the second is a section of
RFC 8439.

## N12: The First Complete Study

> “I speculate that ChaCha has similar resistance to “ChaCha” against the attack, but of course this has to be checked carefully.”
>
> — Daniel J. Bernstein, *ChaCha, a variant of Salsa20* (2008.01.28), §2.2. [S9]

The sentence is a speculation, and the clause after the comma is the
author refusing to let the speculation stand in for a check. This lesson
takes that refusal as its method. The check it actually performs is
narrower than the attack Bernstein is discussing. You will study one
function, on stated inputs, against a stated standard. You will say, at
the end, which claims that study supports and which claims it leaves
untouched. A speculation that survived one vector is still a speculation.

### N12.1 What you will be able to do

Four outcomes finish the lesson. They are the Part 1 readiness check in
working form. None of them is a certificate, and none of them is conferred
by reaching the last page.

1. **Read one construction as the standard writes it.** You can take the
   ChaCha20 block function in RFC 8439 §2.3, including the quarter round
   it calls, and say what each symbol in that text denotes before any
   program runs.
2. **Derive the elementary results by hand.** You can compute the
   quarter-round vector of §2.1.1 and the state vector of §2.2.1 from the
   operations, and you can compute the initial state and the first column
   of the §2.3.2 vector the same way.
3. **Transcribe it, test it, and repair it.** You can write the block
   function in Orange so that each update keeps the standard's name and
   order, attach known-answer tests whose expected values cite the RFC
   section they came from, read one deliberate error's diagnostic, and
   change only the text that diagnostic identifies.
4. **Locate the boundary of the claim.** You can separate a mathematical
   property, a test, and an implementation behavior, and you can point at
   the assumption each one still depends on.

The construction is the ChaCha20 block function of RFC 8439 §2.3, checked
against the test vector of §2.3.2. The quarter round is §2.1. The single
quarter round on a sixteen-word state is §2.2. Those three sections are
the whole study. Encryption, Poly1305, and the AEAD construction are later
sections of the same RFC. They are named here so you can see the edge of
the page you are on. They are not transcribed.

### N12.2 Why this construction

A complete study needs a construction small enough to derive and large
enough that a transcription can be wrong in a way a single operation
cannot show. The quarter round alone is already in N7. A second copy of
that listing would teach the same four words again. The block function is
the next object the standard defines, and it is still one pure function:
sixteen words in, sixty-four bytes out, no key schedule beyond the loading
of those words, no branch on the data.

Four facts make it the right object for the surface you have.

The state is one array of sixteen `Word[32]` values. Literal indices
select them. N7 already proved an index by showing that every value it
can take lies in range, and already rejected an `Int` parameter used as
an index. The block function's indices are constants in the standard
(`0`, `4`, `8`, `12`, and the seven other quadruples). They can be
written as literal indices. A parameter that tried to carry a quadruple's
indices would meet the rejection you already know.

The repetition is ten applications of one function. The RFC writes
`for i=1 upto 10`. Orange writes a bounded `for` whose end is exclusive,
so the same ten integers are `1..11`. The body does not read `i`. The
index is the counter the standard names. The checker still requires the
bounds to be integer literals, which these are.

The feed-forward sum at the end is sixteen additions of `Word[32]`. One
of them wraps in the §2.3.2 vector, and several do not. You can see the
residue rule from Chapter 6 on a real standard vector, in some positions
and not in others.

The serialization is little-endian, four bytes from each word. Shifts and
`as` are enough. The byte string the RFC prints is then a second reading
of the same sixteen words, and a test can demand either reading.

Other constructions in the repository are real and are the wrong size for
this lesson. SHA-256 needs a message schedule and sixty-four rounds of a
different function. AES needs a field and a table. Poly1305 needs a
modulus the novice lessons have not built up from the integers you have.
The block function needs addition modulo 2³², XOR, and rotation, which
are the operators Proposition N7.2 already assumed, applied to more words
and then added back to the input.

The choice is also a limitation. ChaCha20 as a cipher calls this function
once per block and XORs the bytes with plaintext. That call, that XOR,
and the rule for the counter are §2.4. Studying the block function does
not study the cipher. Bernstein's sentence is about resistance to an
attack across rounds. Matching §2.3.2 does not check that resistance. The
match checks that one transcription, on one input, produced the bytes the
RFC prints.

### N12.3 What you may take as given

Five assumptions bound the mathematics. Two further assumptions bound the
programs, and they arrive with the programs. The five are available now.

**Assumption C1.** `+` on a pair of 32-bit words denotes addition modulo
2³². The integer sum is reduced by subtracting 2³² as many times as
needed to land in 0 through 2³² − 1. For a sum of two such words, that is
either no subtraction or one. `2³² = 4294967296`.

**Assumption C2.** `^` denotes bitwise exclusive or, the operation of
Chapter 3, applied independently at each of the 32 bit positions.

**Assumption C3.** `<<< n` for a literal `n` with `0 ≤ n < 32` denotes
left rotation by `n`: the high `n` bits move to the low end, and the
remaining `32 − n` bits move up by `n`. This is the RFC's “n-bit left
roll.” Rotation by 16 exchanges the two 16-bit halves. Rotation by 8
cycles the four bytes one place toward the high end.

**Assumption C4.** The test vectors copied below are the vectors printed
in RFC 8439 §2.1.1, §2.2.1, and §2.3.2 of the June 2018 document. A
disagreement between a later erratum and this text would have to be
resolved by reading the erratum. None is applied here.

**Assumption C5.** A hand calculation and a program denote the same word
only where both are defined to be the same sequence of operations under
C1–C3. Agreement on one input is one point. The domain of four words is
2¹²⁸ inputs. The domain of a sixteen-word state is 2⁵¹² inputs.

N7's Proposition N7.2 used C1–C3 for the quarter round and concluded that
the four Orange functions denote the standard's four results on every
input. That proposition is available. This lesson recomputes two vectors
in full so that the block function is not resting on a result you have
only been told. Recomputation is the study's first pages, not a second
proof of the same proposition.

### N12.4 The quarter round, as the standard writes it

RFC 8439 §2.1 operates on four 32-bit unsigned integers `a`, `b`, `c`,
and `d`. The text gives the operation in C-like notation:

```text
a += b; d ^= a; d <<<= 16;
c += d; b ^= c; b <<<= 12;
a += b; d ^= a; d <<<= 8;
c += d; b ^= c; b <<<= 7;
```

Read each statement as a new value of that letter. `a += b` means the new
`a` is the old `a` plus `b`, modulo 2³². The letters on the right are the
values those letters have when the statement is reached, so the second
`a += b` adds the already updated `a` to the already updated `b`. The RFC
uses one letter for both. Orange, under N7's assumption A1, will give the
new value a fresh name. The names used here are the ones N7 fixed:

```text
a1 = a + b
d1 = (d XOR a1) <<< 16
c1 = c + d1
b1 = (b XOR c1) <<< 12
a2 = a1 + b1
d2 = (d1 XOR a2) <<< 8
c2 = c1 + d2
b2 = (b1 XOR c2) <<< 7
```

Final `a` is `a2`. Final `b` is `b2`. Final `c` is `c2`. Final `d` is
`d2`. The order is the order of the statements. Stopping after `d1` is a
different function.

### N12.5 Three operations from the fourth line

Before the test vector, §2.1 shows one example and labels it as the add,
XOR, and roll from the fourth line. The numbers are:

```text
a = 0x11111111
b = 0x01020304
c = 0x77777777
d = 0x01234567
```

The fourth line is `c += d; b ^= c; b <<<= 7`. The example applies those
three operations to these original four numbers. It does not first apply
the earlier six statements. The `c` that is added to `d` is still
`0x77777777`.

Add byte by byte, low byte first. `0x77 + 0x67 = 119 + 103 = 222 = 0xde`,
and 222 is less than 256, so there is no carry into the next byte. The
same pattern holds for the other three bytes: `0x77 + 0x45 = 0xbc`,
`0x77 + 0x23 = 0x9a`, and `0x77 + 0x01 = 0x78`. So

```text
c + d = 0x789abcde
```

The integer sum is 2004318071 + 19088743 = 2023406814, which equals
`0x789abcde` and is less than 2³². Assumption C1 therefore leaves the sum
unchanged.

XOR with the original `b`:

```text
0x01020304 XOR 0x789abcde = 0x7998bfda
```

Check the bytes: `0x01 XOR 0x78 = 0x79`, `0x02 XOR 0x9a = 0x98`,
`0x03 XOR 0xbc = 0xbf`, `0x04 XOR 0xde = 0xda`.

Rotate left by 7. The bits of `0x7998bfda`, grouped by fours, are

```text
0111 1001 1001 1000 1011 1111 1101 1010
```

The high seven bits are `0111100`. The other twenty-five bits move up,
and those seven bits reenter at the low end:

```text
1100 1100 0101 1111 1110 1101 0011 1100
```

That word is `0xcc5fed3c`. The RFC prints the same three results:
`0x789abcde`, `0x7998bfda`, and `0xcc5fed3c`.

**Proposition N12.1.** On the four numbers printed in the example of
RFC 8439 §2.1, the three operations of the fourth line, applied to the
original `c`, `d`, and `b`, produce `0x789abcde`, `0x7998bfda`, and
`0xcc5fed3c`.

*Proof.* The byte sums above have no carry out of a byte, and the integer
sum is less than 2³², so C1 leaves `c + d` equal to `0x789abcde`. The four
byte XORs are C2, and they produce `0x7998bfda`. The bit movement is C3
for `n = 7`, and the resulting bits are `0xcc5fed3c`. □

The proposition stops at those three operations. It does not say what a
quarter round returns. The next section changes `c` and runs all eight
updates. A reader who substitutes the example's rotated `b` for the
quarter round's final `b` has applied Proposition N12.1 outside its
hypotheses.

### N12.6 The quarter-round vector, derived again

Section 2.1.1 keeps `a`, `b`, and `d` from the example and replaces `c`:

```text
a = 0x11111111
b = 0x01020304
c = 0x9b8d6f43
d = 0x01234567
```

The RFC states that a quarter round sends these to

```text
a = 0xea2a92f4
b = 0xcb1cf8ce
c = 0x4581472e
d = 0x5881c4bb
```

N7 computed the same eight names and recorded the same four results.
Compute them again here. The study has to be able to stand on its own
desk, and the place the arithmetic becomes interesting is the one
addition that crosses 2³².

`0x11111111 + 0x01020304 = 0x12131415`. In integers that is
286331153 + 16909060 = 303240213, and 303240213 < 4294967296, so
`a1 = 0x12131415`.

`0x01234567 XOR 0x12131415 = 0x13305172`. Rotation by 16 exchanges the
halves: `d1 = 0x51721330`.

`c + d1` produces no carry from one byte into the next:

```text
0x43 + 0x30 = 0x73
0x6f + 0x13 = 0x82
0x8d + 0x72 = 0xff
0x9b + 0x51 = 0xec
```

So `c1 = 0xecff8273`. The integer sum is 2609737539 + 1366430512 =
3976168051, still below 2³².

`0x01020304 XOR 0xecff8273 = 0xedfd8177`. Rotation by 12 moves the high
twelve bits, `0xedf`, to the low end and the low twenty bits, `0xd8177`,
to the high end. Thus `b1 = 0xd8177edf`.

`a1 + b1 = 0x12131415 + 0xd8177edf`. In integers,
303240213 + 3625418463 = 3928658676 = `0xea2a92f4`, and
3928658676 < 4294967296, so `a2 = 0xea2a92f4`. This is already the RFC's
final `a`.

`0x51721330 XOR 0xea2a92f4 = 0xbb5881c4`. Rotation by 8 cycles the bytes:
`d2 = 0x5881c4bb`. This is the RFC's final `d`.

The next addition crosses the modulus. In ordinary integers,

```text
0xecff8273 + 0x5881c4bb = 3976168051 + 1484899515 = 5461067566
5461067566 = 1 × 4294967296 + 1166100270
1166100270 = 0x4581472e
```

C1 drops the multiple of 2³², so `c2 = 0x4581472e`. That is the RFC's
final `c`. Keeping 5461067566 would be the integer sum, which is a
different operation. The next XOR uses the residue, not the integer sum:

```text
0xd8177edf XOR 0x4581472e = 0x9d9639f1
```

The bits of `0x9d9639f1`, then the bits after a left rotation by 7:

```text
1001 1101 1001 0110 0011 1001 1111 0001
1100 1011 0001 1100 1111 1000 1100 1110
```

The second line is `b2 = 0xcb1cf8ce`, the RFC's final `b`.

**Proposition N12.2.** Under C1–C3, the eight updates send
`(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)` to
`(0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)`.

*Proof.* Each displayed sum, XOR, and rotation is the corresponding
update, and each sum that was compared with 2³² was reduced exactly when
C1 requires it. The four final names are the four words the RFC prints
for this input. □

The proof is one input. Proposition N7.2, which you may use, is the claim
for every input, and it still assumes C1–C3 rather than discharging them.
Proposition N12.2 does not become that universal claim by being the RFC's
chosen point. It is the checkpoint the rest of the study is allowed to
start from: if a later transcription of the quarter round disagrees with
these four words on these four inputs, the transcription is wrong here,
whatever it does on any other input.

Notice what changed from the sample. The sample's `c` was `0x77777777`.
The vector's `c` is `0x9b8d6f43`. The sample stopped at three operations
on the original words. The vector runs eight updates, and its final `b`
is `0xcb1cf8ce`, not the sample's `0xcc5fed3c`. Both calculations agree
with the RFC. They agree with different sentences of the RFC.

### N12.7 One quarter round on sixteen words

Section 2.2 defines the state as sixteen 32-bit unsigned integers, drawn
as a matrix in row-major order. The RFC's diagram, using the C convention
that the first index is zero, is

```text
 0  1  2  3
 4  5  6  7
 8  9 10 11
12 13 14 15
```

`QUARTERROUND(x, y, z, w)` means: take the words at those four indices as
`a`, `b`, `c`, and `d`, run the quarter round, and write the four results
back to the same four indices. Every other index keeps its word.

The section's illustration is `QUARTERROUND(1, 5, 9, 13)`, which touches
the second column. The test vector in §2.2.1 is a different quadruple,
`QUARTERROUND(2, 7, 8, 13)`, which the RFC calls part of a diagonal round.
The input state, printed in four rows of four words, is

```text
879531e0  c5ecf37d  516461b1  c9a62f8a
44c20ef3  3390af7f  d9fc690b  2a5f714c
53372767  b00a5631  974c541a  359e9963
5c971061  3d631689  2098d9d6  91dbd320
```

Read row by row into indices 0 through 15. The four words the operation
reads are

```text
a = index 2  = 0x516461b1
b = index 7  = 0x2a5f714c
c = index 8  = 0x53372767
d = index 13 = 0x3d631689
```

The other twelve words are spectators. Derive the eight updates.

`a + b` in integers is 1365533105 + 710897996 = 2076431101 = `0x7bc3d2fd`.
That integer is less than 2³², so `a1 = 0x7bc3d2fd`.

XOR with `d`, nibble by nibble from the high end:

```text
0x3d631689
0x7bc3d2fd
0x46a0c474
```

`3 XOR 7 = 4`, `d XOR b = 6`, `6 XOR c = a`, `3 XOR 3 = 0`,
`1 XOR d = c`, `6 XOR 2 = 4`, `8 XOR f = 7`, `9 XOR d = 4`. So the word
entering the rotation is `0x46a0c474`. Rotation by 16 exchanges the
halves: `d1 = 0xc47446a0`.

`c + d1` does cross 2³². In integers,
1396123495 + 3295954592 = 4692078087. Divide by the modulus once:

```text
4692078087 = 1 × 4294967296 + 397110791
397110791 = 0x17ab6e07
```

C1 therefore sets `c1 = 0x17ab6e07`. The leading `1` in `0x117ab6e07` is
the multiple of 2³². Writing `c1` as `0x117ab6e07` would be the integer
sum, nine hex digits, which does not fit in a 32-bit word.

`b XOR c1 = 0x2a5f714c XOR 0x17ab6e07 = 0x3df41f4b`. The bits, then the
bits after a left rotation by 12:

```text
0011 1101 1111 0100 0001 1111 0100 1011
0100 0001 1111 0100 1011 0011 1101 1111
```

The high twelve bits were `0011 1101 1111`, which is `0x3df`, and they
now occupy the low end. The low twenty bits were `0x41f4b`, and they now
occupy the high end. Thus `b1 = 0x41f4b3df`.

`a1 + b1` in integers is 2076431101 + 1106555871 = 3182986972 =
`0xbdb886dc`, which is less than 2³², so `a2 = 0xbdb886dc`.

`d1 XOR a2 = 0xc47446a0 XOR 0xbdb886dc = 0x79ccc07c`. Rotation by 8 cycles
the bytes one place toward the high end, and the old high byte `0x79`
reenters as the low byte: `d2 = 0xccc07c79`.

`c1 + d2` in integers is 397110791 + 3435166841 = 3832277632 =
`0xe46bea80`. Compare with the modulus: 3832277632 < 4294967296, so this
addition does not wrap. `c2 = 0xe46bea80`. The previous addition wrapped
and this one does not. Both are C1. The difference is the integer, not a
second rule.

`b1 XOR c2 = 0x41f4b3df XOR 0xe46bea80 = 0xa59f595f`. The bits, then the
bits after a left rotation by 7:

```text
1010 0101 1001 1111 0101 1001 0101 1111
1100 1111 1010 1100 1010 1111 1101 0010
```

The second line is `b2 = 0xcfacafd2`.

The four results are therefore

```text
index  2 = a2 = 0xbdb886dc
index  7 = b2 = 0xcfacafd2
index  8 = c2 = 0xe46bea80
index 13 = d2 = 0xccc07c79
```

Place them back into the matrix and leave the other twelve words as they
were:

```text
879531e0  c5ecf37d  bdb886dc  c9a62f8a
44c20ef3  3390af7f  d9fc690b  cfacafd2
e46bea80  b00a5631  974c541a  359e9963
5c971061  ccc07c79  2098d9d6  91dbd320
```

The RFC prints the same matrix, with asterisks on the four changed words.
The asterisks are the RFC's way of pointing. They are not part of the
state.

**Proposition N12.3.** Under C1–C3, `QUARTERROUND(2, 7, 8, 13)` applied
to the §2.2.1 input state changes exactly the words at indices 2, 7, 8,
and 13, and those words become `0xbdb886dc`, `0xcfacafd2`, `0xe46bea80`,
and `0xccc07c79`.

*Proof.* By the definition in §2.2, the operation runs the quarter round
on the words at the four named indices and writes the results to those
indices. The eight updates were computed above, and each sum was reduced
modulo 2³² exactly when the integer sum was at least 2³². The other
twelve indices are not among the four written, so their words are the
input words. The four written words are `a2`, `b2`, `c2`, and `d2`. □

Again the proof is one state. It is the state the RFC chose. Agreement
with the printed matrix supports the calculation on that state. It does
not, by itself, show that every other quadruple of indices would be
handled by the same eight updates. That further claim is the definition:
`QUARTERROUND` is the quarter round, whatever the indices are, provided
the indices select four words. The calculation shows that this reading of
the definition reproduces the printed matrix. A transcription that
changed index 2 and also changed index 3 would be a different function,
even if its four new words included `0xbdb886dc`.

The addition that wrapped is `c + d1`. The addition that did not wrap,
and that a tired eye might “correct” into a wrap, is `c1 + d2`. Both
integers are on the page above so that the comparison with 4294967296 can
be repeated without trusting the hex.

## Sources and epigraph record

The quotations and the copied vectors are the only borrowed words. The
derivations are the lesson's.

**[S9] Daniel J. Bernstein.** “ChaCha, a variant of Salsa20.” Document
date 2008.01.28. Permanent ID `4027b5256e17b9796842e6d0f68b0b5e`. The
epigraph is one sentence of §2.2, “The ChaCha quarter-round,” on page 3
of the six-page paper hosted by the author. The sentence follows his
report of an attack by Aumasson, Fischer, Khazaei, Meier, and Rechberger,
and it qualifies his own speculation about resistance. Wording was checked
on 2026-10-05 against that PDF. The PDF font encodes the letters ff, where
they occur in nearby words such as “diffusion,” as the ligature U+FB00;
the epigraph sentence does not contain that ligature. The double quotes
around the inner ChaCha are U+201C and U+201D, as printed. No translation
is involved. The sentence is not a claim that the check in this lesson is
the check Bernstein required, and it is not an endorsement of Orange.

Source: <https://cr.yp.to/chacha/chacha-20080128.pdf>

**[R1] RFC 8439.** Y. Nir and A. Langley, “ChaCha20 and Poly1305 for IETF
Protocols,” June 2018. Sections 2.1, 2.1.1, 2.2, and 2.2.1 were copied
for the operation, the sample line, and the two quarter-round vectors.
The RFC's word for rotation is “roll.” Consulted 2026-10-05 against the
plain-text document from the RFC Editor. No erratum was applied.

<https://www.rfc-editor.org/rfc/rfc8439>

Epigraph verification establishes wording and attribution, not
publication-rights clearance.
