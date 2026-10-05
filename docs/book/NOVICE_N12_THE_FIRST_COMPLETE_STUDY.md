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

### N12.8 The sixteen words the block function starts from

Section 2.3 names three inputs and one layout.

The key is 256 bits, read as eight 32-bit little-endian integers. The
nonce is 96 bits, read as three 32-bit little-endian integers. The block
count is one 32-bit little-endian integer. The output is 64 bytes. The
RFC also says that the original ChaCha used a 64-bit nonce and a 64-bit
block count, and that this document changes that layout. The study
follows the document in front of you, RFC 8439, not the 2008 paper's
widths. Bernstein's paper is the source of the quarter round. It is not
the source of these input sizes.

The state is filled in four rows:

```text
cccccccc  cccccccc  cccccccc  cccccccc
kkkkkkkk  kkkkkkkk  kkkkkkkk  kkkkkkkk
kkkkkkkk  kkkkkkkk  kkkkkkkk  kkkkkkkk
bbbbbbbb  nnnnnnnn  nnnnnnnn  nnnnnnnn
```

`c` is a constant, `k` a key word, `b` the block count, `n` a nonce word.
The constants are fixed by the RFC as these four words, in this order:

```text
0x61707865  0x3320646e  0x79622d32  0x6b206574
```

They are not an arbitrary salt. They are the sixteen ASCII bytes of the
sentence fragment `expand 32-byte k`, stored little-endian. The bytes, in
the order a reader of the sentence meets them, are

```text
65 78 70 61  6e 64 20 33  32 2d 62 79  74 65 20 6b
```

which is `e`, `x`, `p`, `a`, `n`, `d`, the space, `3`, `2`, `-`, `b`,
`y`, `t`, `e`, the space, `k`.

A little-endian word takes the first byte as the least significant byte.
The first four bytes `65 78 70 61` are therefore the word whose low byte
is `0x65` and whose high byte is `0x61`, written `0x61707865`. The next
three groups give `0x3320646e`, `0x79622d32`, and `0x6b206574`.

**Proposition N12.4.** The four constant words in RFC 8439 §2.3 are the
little-endian 32-bit readings of the sixteen ASCII bytes of
`expand 32-byte k`, taken four at a time from the left.

*Proof.* The sixteen byte values are the ASCII codes of those characters,
in that order. Grouping them as above and placing the first byte of each
group in the low position produces the four words the RFC prints. There
are sixteen bytes and four groups, so the grouping is exhaustive. □

Read the hex of `0x61707865` from the left and you see `61 70 78 65`,
which is `a`, `p`, `x`, `e`. That is the same four letters in the
opposite order. The word is not the string `apxe`. The string is `expa`,
and the hex display of a little-endian word shows the high byte on the
left. Both readings are determined. Mixing them produces a different
constant, and a different constant produces a different function. The
test vector later in the section will not forgive that swap, because the
first row of the state is these four words and every later word depends
on them.

Section 2.3.2 chooses a key, a nonce, and a block count:

```text
Key = 00:01:02:03:04:05:06:07:08:09:0a:0b:0c:0d:0e:0f:
      10:11:12:13:14:15:16:17:18:19:1a:1b:1c:1d:1e:1f
Nonce = 00:00:00:09:00:00:00:4a:00:00:00:00
Block Count = 1
```

The key is thirty-two bytes. The RFC says they have no structure until
they are copied into the state. Copy them in little-endian groups of
four. The first group is `00 01 02 03`. The low byte is `0x00` and the
high byte is `0x03`, so the word is `0x03020100`. The same reversal on
each following group gives the eight key words

```text
0x03020100  0x07060504  0x0b0a0908  0x0f0e0d0c
0x13121110  0x17161514  0x1b1a1918  0x1f1e1d1c
```

The block count is the integer 1. As a little-endian word it is
`0x00000001`. The nonce's three groups are `00 00 00 09`,
`00 00 00 4a`, and `00 00 00 00`, so the three words are `0x09000000`,
`0x4a000000`, and `0x00000000`.

Lay these sixteen words into the matrix in the order the RFC states:
constants, key, block count, nonce.

```text
61707865  3320646e  79622d32  6b206574
03020100  07060504  0b0a0908  0f0e0d0c
13121110  17161514  1b1a1918  1f1e1d1c
00000001  09000000  4a000000  00000000
```

**Proposition N12.5.** Under the little-endian grouping just stated, the
inputs of RFC 8439 §2.3.2 produce this matrix, and the matrix is the one
the RFC prints under “ChaCha state with the key setup.”

*Proof.* Proposition N12.4 gives the first row. The eight key groups are
the eight reversals displayed above, and they occupy indices 4 through 11
because the RFC places the key immediately after the constants. Index 12
is the block count, whose only set bit is the low bit of the integer 1.
Indices 13, 14, and 15 are the three nonce groups, each reversed. The
resulting sixteen words are the sixteen words in the RFC's setup matrix,
in the same order. □

This matrix is an input to the rounds. It is not yet the output. A
program that returns the setup and stops has computed Proposition N12.5
and has not computed the block function.

### N12.9 The first column, by hand

The block function runs twenty rounds, in ten repetitions of eight
quarter rounds. The RFC lists them in §2.3:

```text
QUARTERROUND(0, 4,  8, 12)
QUARTERROUND(1, 5,  9, 13)
QUARTERROUND(2, 6, 10, 14)
QUARTERROUND(3, 7, 11, 15)
QUARTERROUND(0, 5, 10, 15)
QUARTERROUND(1, 6, 11, 12)
QUARTERROUND(2, 7,  8, 13)
QUARTERROUND(3, 4,  9, 14)
```

The first four calls are a column round. Each call takes one column of
the matrix. The second four are a diagonal round. The RFC's picture of
`QUARTERROUND(1, 5, 9, 13)` in §2.2 is the second of the column calls.
The test vector of §2.2.1 is the third of the diagonal calls, run on a
different state. The indices in the list above are the same indices.
The state they read, at the start of §2.3.2, is the matrix of
Proposition N12.5, not the random matrix of §2.2.1.

Eighty quarter rounds is eighty times eight updates. Doing all of them
by hand would be a copying exercise. The study does the first call in
full, so that the same eight updates are visible on the actual §2.3.2
state, and then it uses the RFC's printed checkpoint after all twenty
rounds for the additions that follow. The checkpoint is the RFC's
sentence, not a result derived here. That division is part of the claim
you will be asked to state.

The first call is `QUARTERROUND(0, 4, 8, 12)`. From Proposition N12.5,

```text
a = 0x61707865
b = 0x03020100
c = 0x13121110
d = 0x00000001
```

`a + b` in integers is 1634760805 + 50462976 = 1685223781 = `0x64727965`.
That is less than 2³², so `a1 = 0x64727965`.

`d` is 1. XOR with 1 flips the low bit and leaves the other thirty-one
bits alone. The low byte of `a1` is `0x65`, whose low bit is 1, so the
low byte becomes `0x64`. Thus `d XOR a1 = 0x64727964`. Rotation by 16
exchanges the halves: `d1 = 0x79646472`.

`c + d1` byte by byte, low byte first:

```text
0x10 + 0x72 = 16 + 114 = 130 = 0x82
0x11 + 0x64 = 17 + 100 = 117 = 0x75
0x12 + 0x64 = 18 + 100 = 118 = 0x76
0x13 + 0x79 = 19 + 121 = 140 = 0x8c
```

No byte sum reaches 256, so there is no carry between bytes, and
`c1 = 0x8c767582`. In integers, 319951120 + 2036622450 = 2356573570,
which equals that word and is less than 2³².

`b XOR c1 = 0x03020100 XOR 0x8c767582 = 0x8f747482`. The bits, then the
bits after a left rotation by 12:

```text
1000 1111 0111 0100 0111 0100 1000 0010
0100 0111 0100 1000 0010 1000 1111 0111
```

The second line is `b1 = 0x474828f7`.

`a1 + b1` in integers is 1685223781 + 1195911415 = 2881135196 =
`0xabbaa25c`, still below 2³², so `a2 = 0xabbaa25c`.

`d1 XOR a2 = 0x79646472 XOR 0xabbaa25c = 0xd2dec62e`. Rotation by 8 cycles
the bytes, and the old high byte `0xd2` reenters as the low byte:
`d2 = 0xdec62ed2`.

The next sum crosses the modulus.

```text
0x8c767582 + 0xdec62ed2 = 2356573570 + 3737530066 = 6094103636
6094103636 = 1 × 4294967296 + 1799136340
1799136340 = 0x6b3ca454
```

So `c2 = 0x6b3ca454`. The integer 6094103636 is what you would keep if
the addition were the addition of Chapter 2 rather than C1.

`b1 XOR c2 = 0x474828f7 XOR 0x6b3ca454 = 0x2c748ca3`. The bits, then the
bits after a left rotation by 7:

```text
0010 1100 0111 0100 1000 1100 1010 0011
0011 1010 0100 0110 0101 0001 1001 0110
```

The second line is `b2 = 0x3a465196`.

**Proposition N12.6.** Under C1–C3, the first quarter round of the
§2.3.2 column round replaces indices 0, 4, 8, and 12 with `0xabbaa25c`,
`0x3a465196`, `0x6b3ca454`, and `0xdec62ed2`, and leaves the other twelve
words of the setup matrix unchanged.

*Proof.* The four input words are the words Proposition N12.5 places at
those indices. The eight updates are the calculations above, and the one
sum that meets or exceeds 2³² is reduced by exactly one multiple of that
modulus. The definition of `QUARTERROUND` writes results only at the
indices it names. □

The RFC does not print this intermediate. Proposition N12.6 is a derived
checkpoint, not a copied one. When a program later prints the same four
words, the agreement is between the program and this calculation. It is
not, by itself, agreement with a line of the RFC. The RFC's next printed
state is the state after all twenty rounds.

### N12.10 What the twenty rounds leave, and what the addition does

After the list of eight quarter rounds, §2.3 says that ChaCha20 runs
that list ten times, twenty rounds in all, eighty quarter rounds. It
then adds the original input words to the output words, modulo 2³², and
serializes. The pseudocode in §2.3.1 is the same sentence in another
notation:

```text
inner_block(state):
   Qround(state, 0, 4, 8, 12)
   Qround(state, 1, 5, 9, 13)
   Qround(state, 2, 6, 10, 14)
   Qround(state, 3, 7, 11, 15)
   Qround(state, 0, 5, 10, 15)
   Qround(state, 1, 6, 11, 12)
   Qround(state, 2, 7, 8, 13)
   Qround(state, 3, 4, 9, 14)
   end

chacha20_block(key, counter, nonce):
   state = constants | key | counter | nonce
   initial_state = state
   for i=1 upto 10
      inner_block(state)
      end
   state += initial_state
   return serialize(state)
   end
```

The bar in `constants | key | counter | nonce` is concatenation, the
RFC's own note under the pseudocode. It is not the bitwise OR of
Chapter 3. The same glyph will mean OR again when a program uses `|`.
The pseudocode's bar does not.

The loop variable `i` runs from 1 through 10 inclusive. The body does
not read `i`. Ten iterations are the whole use of the variable. The RFC
also says that if the prose and the pseudocode still conflict, the prose
and the test vectors are normative. On this function the three agree:
eight quarter rounds, ten times, then add the original words, then
serialize little-endian.

Section 2.3.2 prints the state after those twenty rounds:

```text
837778ab  e238d763  a67ae21e  5950bb2f
c4f2d0c7  fc62bb2f  8fa018fc  3f5ec7b7
335271c2  f29489f3  eabda8fc  82e46ebd
d19c12b4  b04e16de  9e83d0cb  4e3c50a2
```

Take that matrix as the RFC's checkpoint. Do not pretend it was
recomputed from the setup by hand in this lesson. The addition that
follows can be done by hand, because it is sixteen independent sums.
Each sum is one word of this matrix plus the word of Proposition N12.5
at the same index, reduced modulo 2³².

Index 0 does not wrap. The integers are 2205644971 and 1634760805.
Their sum is 3840405776 = `0xe4e7f110`, and 3840405776 < 4294967296.

Index 1 does wrap.

```text
3795375971 + 857760878 = 4653136849
4653136849 = 1 × 4294967296 + 358169553
358169553 = 0x15593bd1
```

The same rule on all sixteen indices produces the following residues.
`fit` means the integer sum was already a 32-bit word. `wrap` means one
multiple of 2³² was subtracted. The last column is the residue.

```text
index   integer sum   case   residue
    0    3840405776    fit    0xe4e7f110
    1    4653136849    wrap   0x15593bd1
    2    4829548368    wrap   0x1fdd0f50
    3    3295748259    fit    0xc47120a3
    4    3354710471    fit    0xc7f4d1c7
    5    4352163891    wrap   0x0368c033
    6    2594841092    fit    0x9aaa2204
    7    1315755203    fit    0x4e6cd4c3
    8    1180992210    fit    0x466482d2
    9    4457144071    wrap   0x09aa9f07
   10    4392993300    wrap   0x05d7c214
   11    2718075865    fit    0xa2028bd9
   12    3516666549    fit    0xd19c12b5
   13    3108902622    fit    0xb94e16de
   14    3900952779    fit    0xe883d0cb
   15    1312575650    fit    0x4e3c50a2
```

Five positions wrap: 1, 2, 5, 9, and 10. The other eleven do not.
Index 12 is the block count. Its original word is 1, and the checkpoint
word plus 1 is 3516666549, which still fits, so the residue is one more
than the checkpoint word. That is a fact about these two integers. It is
not a fact about every block count. If the checkpoint word were
`0xffffffff`, adding 1 would wrap to 0.

**Proposition N12.7.** Grant the §2.3.2 matrix printed after twenty
rounds, and grant the setup of Proposition N12.5. Under C1, the sixteen
sums of corresponding words are the sixteen residues in the table, and
those residues are the matrix the RFC prints as “ChaCha state at the end
of the ChaCha20 operation.”

*Proof.* Each row of the table is one pair. The five wrapping rows are
the pairs whose integer sum is at least 2³², and each of those sums is
less than 2 × 2³² because each addend is less than 2³², so subtracting
2³² once is the residue C1 requires. The eleven other sums are already
in range. Reading the residues in order gives

```text
e4e7f110  15593bd1  1fdd0f50  c47120a3
c7f4d1c7  0368c033  9aaa2204  4e6cd4c3
466482d2  09aa9f07  05d7c214  a2028bd9
d19c12b5  b94e16de  e883d0cb  4e3c50a2
```

which is the RFC's final matrix. The hypothesis that the after-rounds
matrix is the result of twenty rounds is the RFC's statement. This
proposition does not prove that hypothesis. It proves the addition that
the RFC places after it. □

A program that matches the final matrix might be right because its rounds
are right, or it might be right because an error in the rounds was
cancelled by an error in the addition. Proposition N12.7 separates those
possibilities for this vector: the addition, given the printed
checkpoint, is determined. A transcription can be checked against the
checkpoint and against the final matrix separately. The lesson's program
will print both.

### N12.11 Sixty-four bytes, least significant first

`serialize` writes the words one by one, each word little-endian. For a
word `w`, the four bytes are the residues

```text
w mod 256
floor(w / 256) mod 256
floor(w / 65536) mod 256
floor(w / 16777216) mod 256
```

Equivalently, they are the four bytes of `w` from low to high. The block
is those four bytes for index 0, then index 1, and so on through index 15.
The length is 16 × 4 = 64.

Take the first final word, `0xe4e7f110`, which is 3840405776. The
hex digits of that word already are the bytes, high byte first:
`e4`, `e7`, `f1`, `10`. Little-endian order reverses them:

```text
10 f1 e7 e4
```

**Proposition N12.8.** The little-endian serialization of `0xe4e7f110`
is the four bytes `10 f1 e7 e4`.

*Proof.* `0x10 = 16` and `3840405776 − 16` is divisible by 256, because
the low eight bits of `0xe4e7f110` are `0001 0000`. Dividing the
remaining word `0xe4e7f1` by the same rule yields `0xf1`, then `0xe7`,
then `0xe4`. Those are the four residues in the definition, in order. □

The RFC's serialized block begins

```text
000  10 f1 e7 e4 d1 3b 59 15 50 0f dd 1f a3 20 71 c4
016  c7 d1 f4 c7 33 c0 68 03 04 22 aa 9a c3 d4 6c 4e
032  d2 82 64 46 07 9f aa 09 14 c2 d7 05 d9 8b 02 a2
048  b5 12 9c d1 de 16 4e b9 cb d0 83 e8 a2 50 3c 4e
```

The first four bytes are Proposition N12.8. The next four are the
little-endian bytes of `0x15593bd1`, namely `d1 3b 59 15`. The row that
starts at offset 48 is the little-endian bytes of the last four words.
Index 12 is `0xd19c12b5`, whose low byte is `0xb5`, so offset 48 begins
`b5 12 9c d1`. That is what the RFC prints.

**Proposition N12.9.** Serializing the sixteen residues of Proposition
N12.7 in little-endian order produces the sixty-four bytes printed as
the serialized block in RFC 8439 §2.3.2.

*Proof.* Proposition N12.8 is the first word. Each later word is a
32-bit residue, so it has a unique four-byte little-endian form, and
the RFC's lines are those forms in index order: the line at offset 0
holds indices 0 through 3, offset 16 holds indices 4 through 7, offset
32 holds indices 8 through 11, and offset 48 holds indices 12 through
15. Comparing each word's four bytes with the corresponding four
positions in those lines matches throughout. The ASCII column to the
right of the RFC's hex is a rendering of the same bytes. It is not a
second value. □

The comparison in that proof is finite and exhaustive: sixteen words,
four bytes each. It is the sort of check a truth table is, scaled to
sixty-four positions. It does not show that a different final matrix
would serialize to the same bytes. Little-endian encoding of a 32-bit
word is unique, so a different word would differ in at least one byte.

Two readings remain easy to confuse, and both have now appeared. The
setup's key bytes `00 01 02 03` became the word `0x03020100` because
loading is little-endian. The final word `0xe4e7f110` becomes the bytes
`10 f1 e7 e4` because storing is little-endian. The same rule, applied
in opposite directions to the same byte order, is why the hex of a word
and the hex of its stored bytes are reversals of each other. A test that
expects `e4 e7 f1 10` is a test of the opposite order. It can fail while
every round is right.

The pseudocode returns `serialize(state)`. The sixty-four bytes are the
block function's result. The final matrix is the value that serialization
reads. Both are in the RFC. A study that checks only the matrix has not
yet checked the function the pseudocode returns. A study that checks
only the bytes has not shown which of the two stages produced them. The
program in the next sections prints both, and the tests name both.

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
