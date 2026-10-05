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

### N12.12 A program is a transcription, and a transcription can be counted

The hand work stopped where counting eighty quarter rounds would have
become copying. The program's job is to continue the same operations
without inventing new ones. Every name in the listings below is one
update you have already computed, or one of the RFC's own names:
`quarter_round`, `column_round`, `diagonal_round`, `inner_block`,
`rounds`, `add_original`, `serialize`.

**Assumption C6.** On `Word[32]`, `>> n` for a literal `n` with
`0 ≤ n < 32` denotes the right shift that discards the low `n` bits.
`w as Word[8]` keeps the low eight bits of `w`, which is `w mod 256`.
So `(w >> 8) as Word[8]` is `floor(w / 256) mod 256`. The parentheses
are the grouping from Listing N7.3: `as` converts one operand.

**Assumption C7.** `[0; 64]` is a fill literal. It denotes an array of
length 64 whose every element is the value `0`. The length is the
decimal integer after the semicolon. The element is checked against the
array's element type before evaluation. The literal builds a new array.
It does not reserve a region that later text overwrites.

**Assumption C8.** `s with [k] = v` is the array equal to `s` except at
the literal or proved index `k`, where it holds `v`. A chain of `with`
updates is evaluated from the inside out. Each call of `quarter_round`
in the listings is one call. Writing `.0` and `.1` from that one result
is selection, not a second round.

C8 is the point a fused expression gets wrong. Suppose a step both
updates index `i` and then calls `quarter_round` again on the updated
array to obtain `.1`. The second call reads the new word at `i` as its
`a`. That is a different function from calling once and selecting both
components. The checker accepts either spelling when the indices are in
range. The names `q0`, `q1`, `q2`, and `q3` exist so that each column
or diagonal is one call.

### N12.13 The sample line and the quarter-round vector

Listing N12.1 is Proposition N12.1 and nothing else. The three bindings
are the fourth line of §2.1 applied to the original words. There is no
`a1` and no `d1`. The result is the sample's rotated `b`.

**Listing N12.1 — `sample_line.or`**

```orange
edition 2026;
module sample_line {
  spec rolled() -> Word[32] {
    let c: Word[32] = 0x77777777;
    let d: Word[32] = 0x01234567;
    let b: Word[32] = 0x01020304;
    let sum: Word[32] = c + d;
    let mixed: Word[32] = b ^ sum;
    mixed <<< 7
  }
}
```

**Expected evaluation output:**

```text
sample_line::rolled: Word[32] = 0xcc5fed3c
```

`0xcc5fed3c` is 3428838716. It is the value Proposition N12.1 names.
Evaluation printing it is the implementation agreeing with that
proposition on this input. The module has no test. A silent check of
this file says the bindings are well-formed. It does not say they are
the quarter round.

Listing N12.2 is the eight updates, returned as one tuple, in the order
`(a2, b2, c2, d2)`. That order is the standard's `(a, b, c, d)`, which
is not alphabetical and not the order of the indices in a diagonal.
Position `.1` is final `b`. The test's right-hand side is the four words
printed in RFC 8439 §2.1.1, copied from that section. They were not
produced by running this listing and pasting the output back into the
source.

**Listing N12.2 — `quarter_vector.or`**

```orange
edition 2026;
module quarter_vector {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> (Word[32], Word[32], Word[32], Word[32]) {
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

  spec vector() -> (Word[32], Word[32], Word[32], Word[32]) {
    quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
  }

  test "RFC 8439 2.1.1" {
    vector() == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
  }
}
```

**Expected evaluation output:**

```text
quarter_vector::vector: (Word[32], Word[32], Word[32], Word[32]) = (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
```

**Test report:**

```text
test "RFC 8439 2.1.1" ... ok
1 test: 1 passed, 0 failed
```

**Proposition N12.10.** Under C1–C3 and the tuple reading of N7's
assumption A5, `vector()` denotes
`(0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)`.

*Proof.* The eight bindings are the eight updates of §2.1. Proposition
N12.2 computed them on these inputs. The result tuple places `a2`, `b2`,
`c2`, and `d2` in positions 0, 1, 2, and 3. □

The test report says that `Bool` was true on this run. The domain of the
test is this one four-tuple, not the 2¹²⁸ four-tuples of the quarter
round. Proposition N7.2 remains the universal claim, and it still rests
on C1–C3.

### N12.14 Writing four results back into sixteen words

`QUARTERROUND(2, 7, 8, 13)` reads four words and writes four words.
Listing N12.3 does that with one call and four updates. `q.0` is `a2`
and belongs at index 2. `q.1` is `b2` and belongs at index 7. `q.2` is
`c2` at index 8. `q.3` is `d2` at index 13. The other twelve positions
are never the target of a `with`.

**Listing N12.3 — `state_quarter.or`**

```orange
edition 2026;
module state_quarter {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> (Word[32], Word[32], Word[32], Word[32]) {
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

  spec after() -> Word[32]^16 {
    let s: Word[32]^16 = [
      0x879531e0, 0xc5ecf37d, 0x516461b1, 0xc9a62f8a,
      0x44c20ef3, 0x3390af7f, 0xd9fc690b, 0x2a5f714c,
      0x53372767, 0xb00a5631, 0x974c541a, 0x359e9963,
      0x5c971061, 0x3d631689, 0x2098d9d6, 0x91dbd320
    ];
    let q: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[2], s[7], s[8], s[13]);
    let s1: Word[32]^16 = s with [2] = q.0;
    let s2: Word[32]^16 = s1 with [7] = q.1;
    let s3: Word[32]^16 = s2 with [8] = q.2;
    s3 with [13] = q.3
  }

  test "RFC 8439 2.2.1" {
    after() == [
      0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a,
      0x44c20ef3, 0x3390af7f, 0xd9fc690b, 0xcfacafd2,
      0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963,
      0x5c971061, 0xccc07c79, 0x2098d9d6, 0x91dbd320
    ]
  }
}
```

**Expected evaluation output:**

```text
state_quarter::after: Word[32]^16 = [0x879531e0, 0xc5ecf37d, 0xbdb886dc, 0xc9a62f8a, 0x44c20ef3, 0x3390af7f, 0xd9fc690b, 0xcfacafd2, 0xe46bea80, 0xb00a5631, 0x974c541a, 0x359e9963, 0x5c971061, 0xccc07c79, 0x2098d9d6, 0x91dbd320]
```

**Test report:**

```text
test "RFC 8439 2.2.1" ... ok
1 test: 1 passed, 0 failed
```

The right-hand side of the test is the matrix derived in Proposition
N12.3 and printed in §2.2.1. The asterisks in the RFC are not bytes of
the state, so they are not in the array. The expected value's provenance
is that section of the RFC. The listing is the transcription.

**Proposition N12.11.** Under C1–C3 and C8, `after()` denotes the
sixteen-word state of Proposition N12.3.

*Proof.* The array `s` is the §2.2.1 input, row by row. The call is
`quarter_round` on indices 2, 7, 8, and 13, which is the quarter round
by Proposition N12.10's reading of the body, applied to those words.
C8 writes `q.0` through `q.3` at indices 2, 7, 8, and 13, and each
`with` leaves every other index unchanged. The composition of the four
updates is therefore the state Proposition N12.3 describes. □

### N12.15 The first column is a derived checkpoint

The RFC does not print the state after only `QUARTERROUND(0, 4, 8, 12)`
on the §2.3.2 setup. Proposition N12.6 does. Listing N12.4 is that
proposition's four words and a test whose expected tuple is those words.
The provenance of the expected tuple is the hand calculation, not a line
of §2.3.2. If the listing disagrees with the proposition, the
transcription of the quarter round is wrong on this input, even if some
later bug were to make the final block look right.

**Listing N12.4 — `opened_column.or`**

```orange
edition 2026;
module opened_column {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> (Word[32], Word[32], Word[32], Word[32]) {
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

  spec column() -> (Word[32], Word[32], Word[32], Word[32]) {
    quarter_round(0x61707865, 0x03020100, 0x13121110, 0x00000001)
  }

  test "first column of RFC 8439 2.3.2" {
    column() == (0xabbaa25c, 0x3a465196, 0x6b3ca454, 0xdec62ed2)
  }
}
```

**Expected evaluation output:**

```text
opened_column::column: (Word[32], Word[32], Word[32], Word[32]) = (0xabbaa25c, 0x3a465196, 0x6b3ca454, 0xdec62ed2)
```

**Test report:**

```text
test "first column of RFC 8439 2.3.2" ... ok
1 test: 1 passed, 0 failed
```

`(0xabbaa25c, 0x3a465196, 0x6b3ca454, 0xdec62ed2)` is `(a2, b2, c2, d2)`
from Proposition N12.6. A passing report here supports the transcription
against the hand calculation. It does not support the twenty-round
checkpoint, because that checkpoint has not been used.

### N12.16 The block function, in the standard's order

The column round is four quarter rounds on the four columns. Their index
quadruples are `(0, 4, 8, 12)`, `(1, 5, 9, 13)`, `(2, 6, 10, 14)`, and
`(3, 7, 11, 15)`. Within one column the outputs land on those same
indices: `.0` on the first, `.1` on the second, `.2` on the third, `.3`
on the fourth. The columns read disjoint indices, so the four calls may
be made on the original state and written afterward. Listing N12.5 does
that. It does not call `quarter_round` a second time to fetch `.1`.

The diagonal round is not the same arithmetic progression. Its
quadruples are

```text
(0, 5, 10, 15)
(1, 6, 11, 12)
(2, 7,  8, 13)
(3, 4,  9, 14)
```

The second of these writes `.3` at index 12, which is smaller than the
index that receives `.2`. A rule “add 4 each time,” which happens to
describe a column, describes none of these four lines. The diagonal
listing therefore names the sixteen positions itself. `q1.3` is the new
`d` of `QUARTERROUND(1, 6, 11, 12)`, and it is stored at index 12.

`inner_block` is the diagonal round of the column round, which is one
pass through the RFC's list of eight calls.

`rounds` is the pseudocode's loop. The RFC writes `for i=1 upto 10`.
Orange's end bound is exclusive, so the integers 1 through 10 are
written `1..11`. The body does not read `i`. Ten values of `i` means
ten applications.

**Proposition N12.12.** Let `inner` be a function on states. The loop
`for i in 1..11 with acc = s { inner(acc) }` denotes `inner` composed
with itself ten times, applied to `s`.

*Proof.* The index takes the integers `start, start + 1, …, end − 1`
under N7's assumption A6. Here that list is 1, 2, …, 10, which has ten
elements. The accumulator starts as `s`. Each step replaces it by
`inner` of its current value, and the step does not use `i`, so the
replacement does not depend on which of those integers is current.
After one step the value is `inner(s)`. If after `k` steps the value is
the `k`-fold composition, the next step applies `inner` once more.
After ten steps the value is the tenfold composition. □

`add_original` is the pseudocode's `state += initial_state`, word by
word. The index runs through `0..16`, which is 0 through 15. Both
arrays have length 16, so `worked[i]` and `original[i]` are in range.
The sum is C1, because both operands are `Word[32]`.

`serialize` builds a length-64 array of bytes. The fill `[0; 64]` is
the starting value required by the loop form; every one of those zeros
is replaced. For each `i` from 0 through 15, the four indices
`4 * i`, `4 * i + 1`, `4 * i + 2`, and `4 * i + 3` run from 0 through
63. The low byte of word `i` is `words[i] as Word[8]`. The next three
bytes are the low eight bits of the word shifted right by 8, 16, and 24.
That is the little-endian layout of Proposition N12.8, applied to every
word.

`setup` is the matrix of Proposition N12.5, written in index order.
`after_rounds` is the tenfold inner block of that matrix, which the RFC
prints as the state after twenty rounds. `words` adds the setup back.
`bytes` serializes `words`. Evaluating the module evaluates each
parameterless spec separately. `after_rounds` and `words` each run the
rounds. The second run is a second evaluation, not a saved matrix.

The two tests name their provenance in the title. The right-hand side of
the word test is the final matrix of §2.3.2, the matrix Proposition
N12.7 derived from the RFC's after-rounds checkpoint plus the setup.
The right-hand side of the byte test is the serialized block of that
same section, the sixty-four bytes of Proposition N12.9. Neither side
was taken from this listing's output.

**Listing N12.5 — `chacha_block.or`**

```orange
edition 2026;
module chacha_block {
  spec quarter_round(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> (Word[32], Word[32], Word[32], Word[32]) {
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

  spec column_round(s: Word[32]^16) -> Word[32]^16 {
    let q0: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[0], s[4], s[8], s[12]);
    let q1: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[1], s[5], s[9], s[13]);
    let q2: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[2], s[6], s[10], s[14]);
    let q3: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[3], s[7], s[11], s[15]);
    let t0: Word[32]^16 = s with [0] = q0.0;
    let t1: Word[32]^16 = t0 with [4] = q0.1;
    let t2: Word[32]^16 = t1 with [8] = q0.2;
    let t3: Word[32]^16 = t2 with [12] = q0.3;
    let t4: Word[32]^16 = t3 with [1] = q1.0;
    let t5: Word[32]^16 = t4 with [5] = q1.1;
    let t6: Word[32]^16 = t5 with [9] = q1.2;
    let t7: Word[32]^16 = t6 with [13] = q1.3;
    let t8: Word[32]^16 = t7 with [2] = q2.0;
    let t9: Word[32]^16 = t8 with [6] = q2.1;
    let t10: Word[32]^16 = t9 with [10] = q2.2;
    let t11: Word[32]^16 = t10 with [14] = q2.3;
    let t12: Word[32]^16 = t11 with [3] = q3.0;
    let t13: Word[32]^16 = t12 with [7] = q3.1;
    let t14: Word[32]^16 = t13 with [11] = q3.2;
    t14 with [15] = q3.3
  }

  spec diagonal_round(s: Word[32]^16) -> Word[32]^16 {
    let q0: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[0], s[5], s[10], s[15]);
    let q1: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[1], s[6], s[11], s[12]);
    let q2: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[2], s[7], s[8], s[13]);
    let q3: (Word[32], Word[32], Word[32], Word[32]) = quarter_round(s[3], s[4], s[9], s[14]);
    let t0: Word[32]^16 = s with [0] = q0.0;
    let t1: Word[32]^16 = t0 with [5] = q0.1;
    let t2: Word[32]^16 = t1 with [10] = q0.2;
    let t3: Word[32]^16 = t2 with [15] = q0.3;
    let t4: Word[32]^16 = t3 with [1] = q1.0;
    let t5: Word[32]^16 = t4 with [6] = q1.1;
    let t6: Word[32]^16 = t5 with [11] = q1.2;
    let t7: Word[32]^16 = t6 with [12] = q1.3;
    let t8: Word[32]^16 = t7 with [2] = q2.0;
    let t9: Word[32]^16 = t8 with [7] = q2.1;
    let t10: Word[32]^16 = t9 with [8] = q2.2;
    let t11: Word[32]^16 = t10 with [13] = q2.3;
    let t12: Word[32]^16 = t11 with [3] = q3.0;
    let t13: Word[32]^16 = t12 with [4] = q3.1;
    let t14: Word[32]^16 = t13 with [9] = q3.2;
    t14 with [14] = q3.3
  }

  spec inner_block(s: Word[32]^16) -> Word[32]^16 {
    diagonal_round(column_round(s))
  }

  spec rounds(s: Word[32]^16) -> Word[32]^16 {
    for i in 1..11 with acc: Word[32]^16 = s { inner_block(acc) }
  }

  spec add_original(worked: Word[32]^16, original: Word[32]^16) -> Word[32]^16 {
    for i in 0..16 with acc: Word[32]^16 = worked {
      acc with [i] = worked[i] + original[i]
    }
  }

  spec serialize(words: Word[32]^16) -> Word[8]^64 {
    let blank: Word[8]^64 = [0; 64];
    for i in 0..16 with acc: Word[8]^64 = blank {
      (((acc with [4 * i] = (words[i] as Word[8]))
        with [4 * i + 1] = ((words[i] >> 8) as Word[8]))
        with [4 * i + 2] = ((words[i] >> 16) as Word[8]))
        with [4 * i + 3] = ((words[i] >> 24) as Word[8])
    }
  }

  spec setup() -> Word[32]^16 {
    [
      0x61707865, 0x3320646e, 0x79622d32, 0x6b206574,
      0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c,
      0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c,
      0x00000001, 0x09000000, 0x4a000000, 0x00000000
    ]
  }

  spec after_rounds() -> Word[32]^16 { rounds(setup()) }

  spec words() -> Word[32]^16 {
    let original: Word[32]^16 = setup();
    add_original(rounds(original), original)
  }

  spec bytes() -> Word[8]^64 { serialize(words()) }

  test "RFC 8439 2.3.2 words" {
    words() == [
      0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3,
      0xc7f4d1c7, 0x0368c033, 0x9aaa2204, 0x4e6cd4c3,
      0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9,
      0xd19c12b5, 0xb94e16de, 0xe883d0cb, 0x4e3c50a2
    ]
  }

  test "RFC 8439 2.3.2 bytes" {
    bytes() == [
      0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20, 0x71, 0xc4,
      0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a, 0xc3, 0xd4, 0x6c, 0x4e,
      0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2, 0xd7, 0x05, 0xd9, 0x8b, 0x02, 0xa2,
      0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9, 0xcb, 0xd0, 0x83, 0xe8, 0xa2, 0x50, 0x3c, 0x4e
    ]
  }
}
```

**Expected evaluation output:**

```text
chacha_block::setup: Word[32]^16 = [0x61707865, 0x3320646e, 0x79622d32, 0x6b206574, 0x03020100, 0x07060504, 0x0b0a0908, 0x0f0e0d0c, 0x13121110, 0x17161514, 0x1b1a1918, 0x1f1e1d1c, 0x00000001, 0x09000000, 0x4a000000, 0x00000000]
chacha_block::after_rounds: Word[32]^16 = [0x837778ab, 0xe238d763, 0xa67ae21e, 0x5950bb2f, 0xc4f2d0c7, 0xfc62bb2f, 0x8fa018fc, 0x3f5ec7b7, 0x335271c2, 0xf29489f3, 0xeabda8fc, 0x82e46ebd, 0xd19c12b4, 0xb04e16de, 0x9e83d0cb, 0x4e3c50a2]
chacha_block::words: Word[32]^16 = [0xe4e7f110, 0x15593bd1, 0x1fdd0f50, 0xc47120a3, 0xc7f4d1c7, 0x0368c033, 0x9aaa2204, 0x4e6cd4c3, 0x466482d2, 0x09aa9f07, 0x05d7c214, 0xa2028bd9, 0xd19c12b5, 0xb94e16de, 0xe883d0cb, 0x4e3c50a2]
chacha_block::bytes: Word[8]^64 = [0x10, 0xf1, 0xe7, 0xe4, 0xd1, 0x3b, 0x59, 0x15, 0x50, 0x0f, 0xdd, 0x1f, 0xa3, 0x20, 0x71, 0xc4, 0xc7, 0xd1, 0xf4, 0xc7, 0x33, 0xc0, 0x68, 0x03, 0x04, 0x22, 0xaa, 0x9a, 0xc3, 0xd4, 0x6c, 0x4e, 0xd2, 0x82, 0x64, 0x46, 0x07, 0x9f, 0xaa, 0x09, 0x14, 0xc2, 0xd7, 0x05, 0xd9, 0x8b, 0x02, 0xa2, 0xb5, 0x12, 0x9c, 0xd1, 0xde, 0x16, 0x4e, 0xb9, 0xcb, 0xd0, 0x83, 0xe8, 0xa2, 0x50, 0x3c, 0x4e]
```

**Test report:**

```text
test "RFC 8439 2.3.2 words" ... ok
test "RFC 8439 2.3.2 bytes" ... ok
2 tests: 2 passed, 0 failed
```

Read the four printed lines against the four checkpoints. `setup` is
Proposition N12.5. `after_rounds` is the RFC's matrix after twenty
rounds. `words` is Proposition N12.7's residue column. `bytes` is the
serialized block. The tests ask the last two of those questions again,
as `Bool` values, and leave the first two as printed specs. A printed
spec is not a test. It becomes a comparison only when you hold it
against the page.

**Proposition N12.13.** Grant C1–C3 and C6–C8, and grant that
`column_round` and `diagonal_round` apply the eight `QUARTERROUND`
calls of §2.3 to the indices written in Listing N12.5. Then `words()`
denotes the final matrix of §2.3.2 on the setup of Proposition N12.5,
and `bytes()` denotes the serialized block of that section, if the
tenfold inner block equals the RFC's after-rounds matrix.

*Proof.* `setup()` is that matrix by the literals and Proposition N12.5.
`rounds` is the tenfold composition of `inner_block` by Proposition
N12.12. The hypothesis says that composition equals the printed
after-rounds matrix on this setup. `add_original` adds corresponding
words modulo 2³², which is Proposition N12.7, so `words()` is the final
matrix. `serialize` writes each of those words little-endian, which is
Proposition N12.9, so `bytes()` is the serialized block. □

The hypothesis is still a hypothesis. The hand calculation did not
expand all eighty quarter rounds. The evidence offered for it, in this
lesson, is that `after_rounds()` printed the RFC's matrix, together
with the structural reading of the eight index quadruples. That is
evidence about this input and this transcription. It is not a proof
that every state would match a standard the RFC does not tabulate.

### N12.17 One index past the last word

The feed-forward is the step a tired transcription shifts by one. The
last original word is index 15. The expression `original[i + 1]`, with
`i` running through 0 through 15, asks for indices 1 through 16. Index
16 does not select an element of a length-16 array. The repair is to
name index `i` on both arrays. No other token is involved.

Predict the diagnostic before you read it. The code should be the
out-of-range index code from N7 and N8, `ORC0223`. The message should
name the range the checker computed, 1 through 16, and the type
`Word[32]^16`. The label should name the legal range, 0 through 15.
No value should be printed, because the program is rejected before any
addition runs. The same diagnostic should appear from `check`, from
`eval`, and from `test`, and `test` should print no report. A false
`Bool` is a report. An ill-formed index never becomes a `Bool`.

**Listing N12.6 — `shifted_sum.or`, intentionally rejected**

```orange
edition 2026;
module shifted_sum {
  spec add_original(worked: Word[32]^16, original: Word[32]^16) -> Word[32]^16 {
    for i in 0..16 with acc: Word[32]^16 = worked {
      acc with [i] = worked[i] + original[i + 1]
    }
  }
}
```

**Diagnostic:**

```text
error[ORC0223]: this index runs from 1 through 16, out of range for `Word[32]^16`
 --> <stdin>:5:43
  |
5 | ...     acc with [i] = worked[i] + original[i + 1]
  |                                             ^^^^^ indices run from 0 through 15
  = note: every value an index can take, over every loop index and word in it, must select an element
```

The caret covers `i + 1`. The snippet begins with `...` because the
compiler's line window keeps the span and drops the front of a long
line. The dropped characters are indentation. They are not part of the
expression. The status is 1. Standard output is empty.

**Proposition N12.14.** In Listing N12.6, every value of `i + 1` is an
integer from 1 through 16, and 16 is not a legal index of
`Word[32]^16`. Replacing `i + 1` by `i` makes every index in that loop
a legal index, and the resulting step is the word-wise sum of
Proposition N12.7.

*Proof.* Under A6, `i` in `0..16` takes 0, 1, …, 15. Adding 1 shifts
that interval to 1, 2, …, 16. An index of a length-16 array must
satisfy `0 ≤ k < 16`. The value 16 fails. The interval of `i` itself
is 0 through 15, which is exactly the legal set. The step
`worked[i] + original[i]` is then the sum C1 places at each index, which
is the update in the proof of Proposition N12.7. □

Listing N12.5 already contains that repaired step, in `add_original`.
The minimal edit from Listing N12.6 to that function is the deletion of
` + 1`. Expanding the edit into a rewrite of `quarter_round` would
change a function the diagnostic did not mention. The diagnostic's
subject is the index expression. The repair's subject is that
expression.

Two failures remain different after this repair, and both can still
happen to a correct-looking file. A silent check of Listing N12.5 says
the indices are in range and the tests are well-formed `Bool`s. It does
not say the `Bool`s are true. A passing test says those `Bool`s were
true on this run. It does not say the rounds match the standard on any
input the test does not mention. The off-by-one never reached either of
those reports. It stopped at the index.

### N12.18 What was established

Separate the three layers on this one study.

**Mathematical claims.** Propositions N12.1 through N12.9 are about
stated words under C1–C5. N12.1 is three operations. N12.2 is one
quarter round. N12.3 is one quarter round on one state. N12.4 and N12.5
are the little-endian reading of the constants and of the §2.3.2 inputs.
N12.6 is the first column. N12.7 is the feed-forward, granted the RFC's
after-rounds matrix. N12.8 and N12.9 are serialization. None of these
proves that the tenfold inner block equals that after-rounds matrix.
That equality is the RFC's printed checkpoint, confirmed by one run of
one transcription.

**Tests.** Four tests ran.

The test titled `RFC 8439 2.1.1` compares one quarter round with the
four words of that section. Its domain is one element of a set of size
2¹²⁸.

The test titled `RFC 8439 2.2.1` compares one state update with that
section's matrix. Its domain is one element of a set of size 2⁵¹².

The test titled `first column of RFC 8439 2.3.2` compares one quarter
round with Proposition N12.6. Its expected value is the hand
calculation. Its domain is those four input words.

The tests titled `RFC 8439 2.3.2 words` and `RFC 8439 2.3.2 bytes`
compare `words()` and `bytes()` with the final matrix and the serialized
block of that section. Their domain is the one key, the one nonce, and
the one block count written in `setup`. A pass says the `Bool` was true
on this evaluator, for this source, on this run.

**Implementation behavior.** `orangec check` accepted Listings N12.1
through N12.5 and rejected Listing N12.6 with `ORC0223`. `eval` printed
the lines recorded above and printed nothing for Listing N12.6. `test`
printed the reports recorded above. Those are facts about this compiler
binary and these source texts. They are not facts about every compiler
that might accept a similar syntax, and they are not a proof that the
evaluator implements C1–C3.

The following are outside every one of those layers.

The ChaCha20 encryption algorithm of §2.4, which XORs the serialized
block with plaintext and increments the block count, is not transcribed.
Matching §2.3.2 does not match a ciphertext.

Poly1305, the one-time authenticator, and the AEAD construction are
later sections of the same RFC. They are not studied here.

Bernstein's 2008 widths, a 64-bit nonce and a 64-bit block count, are
not the widths in RFC 8439 §2.3. The epigraph's speculation about
resistance to an attack is not tested by a known-answer vector. He said
the speculation had to be checked carefully. This lesson checked a
different sentence.

The 8-round and 12-round variants, and the 128-bit key, are out of
scope in the RFC's own words. They are out of scope here too.

No claim is made about constant time, about side channels, about the
cost model beyond the fact that these listings finished inside the
evaluator's default step budget, or about native code. This compiler
has no native code generation for these listings.

No claim is made that the operator meanings C1–C3 have been proved from
a smaller semantics inside this lesson. They are assumptions, named so
that a later doubt has an address.

A reader who finishes the listings has one complete path from the
standard's text to a tested transcription, and a stated list of what
that path does not reach. That is the study. It is not a certificate,
and it is not a verdict on ChaCha20.

### N12.19 The calls inside one round commute with each other

A column round looks like four separate calls, and the listing writes
them in the RFC's order: indices `(0, 4, 8, 12)`, then `(1, 5, 9, 13)`,
then `(2, 6, 10, 14)`, then `(3, 7, 11, 15)`. The order is the order the
standard writes. On this particular list it is not an extra constraint,
and you should see why before you trust a later function that does have
an order.

Each of those four quadruples uses four indices. The sixteen indices
0 through 15 appear once each. No index is shared. A quarter round reads
only its four words and writes only its four words. If two calls use
disjoint indices, neither call reads a word the other writes. Each
result is therefore determined by the state before either call. Writing
the two results into one state is the same state whichever result is
written first.

**Proposition N12.15.** The four column calls of §2.3, applied to one
state, produce one combined state, and that combined state does not
depend on the order of the four calls. The same is true of the four
diagonal calls. The column round and the diagonal round do not commute
with each other.

*Proof.* The column quadruples are a partition of `{0, 1, …, 15}`: their
union is the whole set and they are pairwise disjoint, which is checked
by listing the sixteen indices once each. Disjoint indices mean each
call's inputs are words the other calls do not change, so each output
quadruple is a function of the incoming state alone. The combined state
replaces every index by the unique call that owns it. A replacement
determined per index does not depend on the sequence of the writes.

The diagonal quadruples
`(0, 5, 10, 15)`, `(1, 6, 11, 12)`, `(2, 7, 8, 13)`, and
`(3, 4, 9, 14)` are also a partition of the same set. The same argument
applies to them.

The diagonal calls read words the column calls write. `QUARTERROUND(0, 5, 10, 15)`
reads index 0, and the first column call writes index 0. Those two calls
are not disjoint. Running the diagonal round on the original state and
the column round afterward is a different function from running the
column round first: the second function feeds updated words into the
diagonal, and the first function feeds updated words into the columns.
The RFC's inner block is the second function. Listing N12.5 writes
`diagonal_round(column_round(s))`, which is that order. The commuting
result lets the four `let q` bindings inside `column_round` all read the
same `s`. It does not let `diagonal_round` read that same `s`. □

The practical consequence is narrow and worth keeping. You may reorder
the four column bindings among themselves and denote the same state.
You may not swap `column_round` and `diagonal_round` and still claim the
RFC's inner block. A test of the final block can fail for that swap
even though every individual quarter round is the right function. The
failure would be a false `Bool`, not an `ORC` code, because both orders
are well-typed. The names in the listing are the record of which order
was chosen.

### N12.20 The serialize indices are known before the loop runs

`serialize` writes four bytes per word. The index expressions are
`4 * i`, `4 * i + 1`, `4 * i + 2`, and `4 * i + 3`, with `i` drawn from
`0..16`. This is the same kind of obligation as Proposition N7.7. The
checker accepts the loop only after it has computed the range. The
calculation is short enough to do by hand, and it is the reason a length
of 64 is the matching length for sixteen words.

**Proposition N12.16.** If `i` is an integer and `0 ≤ i ≤ 15`, then

```text
0 ≤ 4i ≤ 60
1 ≤ 4i + 1 ≤ 61
2 ≤ 4i + 2 ≤ 62
3 ≤ 4i + 3 ≤ 63
```

Every one of those integers selects an element of an array of length 64,
and every index of that array is one of them for exactly one `i` and one
offset in `{0, 1, 2, 3}`.

*Proof.* From `i ≥ 0`, `4i ≥ 0`. From `i ≤ 15`, `4i ≤ 60`. Adding a
constant `r` with `0 ≤ r ≤ 3` adds `r` to both ends, which is the four
displayed intervals. Each of those integers is at least 0 and at most 63,
so it satisfies `0 ≤ k < 64`. For the converse, any index `k` with
`0 ≤ k ≤ 63` has a unique quotient and remainder on division by 4:
`k = 4i + r` with `0 ≤ r ≤ 3` and `0 ≤ i ≤ 15`. That pair `(i, r)` is
the unique step and offset that write `k`. The fill literal starts as
64 zeros, and the loop writes every index exactly once, so the zeros do
not survive into the result. □

The proposition is why `[0; 64]` is long enough and not longer. A length
of 63 would reject `4 * i + 3` when `i` is 15, because 63 is not less
than 63. A length of 65 would leave index 64 holding the fill's zero,
and the serialized block would be the RFC's sixty-four bytes followed by
a zero the RFC does not print. The type `Word[8]^64` is the length the
partition requires.

### N12.21 One more wrapping sum, computed in full

Index 1 of the feed-forward was computed in §N12.10. Index 5 is the same
rule on another pair, and it is worth doing once more so the table is not
a list you have only been asked to believe. The after-rounds word at
index 5 is `0xfc62bb2f` = 4234328879. The setup word at index 5 is
`0x07060504` = 117835012. The key bytes that produced the setup word are
`04 05 06 07`, reversed into the word. The addition does not care about
that history. It adds the two words.

```text
4234328879 + 117835012 = 4352163891
4352163891 = 1 × 4294967296 + 57196595
57196595 = 0x0368c033
```

Check the subtraction: 4352163891 − 4294967296 = 57196595. The hex of
57196595 is `0x0368c033`, which is index 5 of the RFC's final matrix.
The integer sum is greater than the modulus and less than twice the
modulus, so one subtraction is the whole of C1. This is the table's row
for index 5.

Index 12 is the contrasting case, and it is small. The after-rounds word
is `0xd19c12b4` = 3516666548. The setup word is the block count, 1.
3516666548 + 1 = 3516666549 = `0xd19c12b5`, and 3516666549 < 4294967296,
so the residue is the integer sum. The final matrix's index 12 is one
more than the checkpoint's index 12 because, on these two integers, the
sum fits. Proposition N12.7's table records both rows. The two
calculations are the justification of those two rows; the other fourteen
rows are the same operation on the pairs listed beside them.

### N12.22 Where each diagonal result is stored

The column round's landing rule is easy to say: the four results of
`QUARTERROUND(x, y, z, w)` go back to `x`, `y`, `z`, and `w`, in that
order. The diagonal round uses the same rule. It looks harder only
because the indices are not an arithmetic progression. Write the rule
once, then apply it to the four calls, and the sixteen stores in
`diagonal_round` are determined.

`q0` is `QUARTERROUND(0, 5, 10, 15)`. Then `q0.0` is the new `a` and
belongs at index 0, `q0.1` at index 5, `q0.2` at index 10, and `q0.3`
at index 15.

`q1` is `QUARTERROUND(1, 6, 11, 12)`. Then `q1.0` belongs at index 1,
`q1.1` at index 6, `q1.2` at index 11, and `q1.3` at index 12. Index 12
receiving `.3` is the line a column-trained eye miscopies as index 16
or as index 15. Twelve is the last index of this quadruple. It is a
legal index. It is not the next integer after 11.

`q2` is `QUARTERROUND(2, 7, 8, 13)`. This is the same quadruple as
§2.2.1, applied now to whatever state the column round produced, not to
the random matrix of that section. `q2.0` belongs at index 2, `q2.1` at
index 7, `q2.2` at index 8, and `q2.3` at index 13. Index 8 is less
than index 7. The store `t9 with [8] = q2.2` in Listing N12.5 is that
fact. It is not a transposition.

`q3` is `QUARTERROUND(3, 4, 9, 14)`. Then `q3.0` belongs at index 3,
`q3.1` at index 4, `q3.2` at index 9, and `q3.3` at index 14.

**Proposition N12.17.** The sixteen stores in `diagonal_round` place
each selected component at the index the corresponding `QUARTERROUND`
names for that component, and every index from 0 through 15 is the
target of exactly one store.

*Proof.* The four paragraphs above are the four calls. Each call
contributes four pairs `(index, component)`. Listing those sixteen pairs
gives the targets
0, 5, 10, 15, 1, 6, 11, 12, 2, 7, 8, 13, 3, 4, 9, 14. Sorted, they are
0 through 15 once each, which is the partition already used in
Proposition N12.15. The listing's `with` expressions are those pairs:
`q0.0` at 0, `q0.1` at 5, and so on, through `q3.3` at 14. A store that
swapped `q1.3` from index 12 to index 15 would put two results at 15,
because `q0.3` already belongs there, and it would leave index 12
holding the column round's word. That state is not the diagonal round. □

The column round has the same shape of argument with the other
partition. The listing writes those stores in the same style. Once the
rule is the rule, the length of `column_round` and `diagonal_round` is
the length of the RFC's list. It is not an unrolled mystery.

### N12.23 The block is not yet a cipher

Lesson N11, at the anchor `n11-protect-more-than-appearance`, is the
place where encoding, encryption, hashing, and authentication are
separated, and where a key, a nonce, and a counter get their jobs. This
lesson does not borrow N11's proofs. It does use the separation, because
the block function is easy to misname once the bytes look random.

The output of `chacha20_block` is 64 bytes. On the §2.3.2 input those
bytes are the serialized block. They are a function of a public
algorithm, a key, a nonce, and a block count. Anyone who has those four
things and the transcription can recompute them. The bytes do not hide
the key. The key was an input. Hiding would be a different claim, about
an adversary who does not have the key, and this lesson has not stated
such an adversary.

Encryption, in the sense §2.4 of the RFC then defines, takes those 64
bytes and XORs them with plaintext to produce ciphertext, then moves to
the next block count for the next 64 bytes of plaintext. That XOR is
not in Listing N12.5. A reader who treats `bytes()` as ciphertext of the
key, or of the nonce, has renamed the output. The key and the nonce are
readable in `setup`, in the clear, in the same program.

Encoding, in the sense of a public reversible format, is also the wrong
name. Little-endian serialization is an encoding of the sixteen words:
Proposition N12.9 says the bytes determine the words and the words
determine the bytes. The block function wrapped around that encoding is
not undone by reading the bytes backward. Recovering the key from the
64 bytes is not the inverse of `serialize`. This lesson does not prove
that recovery is hard. It only refuses to call the serialization the
whole function, and it refuses to call the whole function an encoding of
the message. There is no message in §2.3.

Authentication is a third name the output has not earned. A tag is a
value used to detect a change. Nothing in the block function compares a
received block with a recomputed block, and Poly1305 is a later section
of the RFC. Matching §2.3.2 does not authenticate anything.

The epigraph sits next to this distinction. Bernstein's sentence is
about resistance to an attack, and he marks the sentence as a
speculation that has to be checked. The check performed here is a
known-answer check of one function on one input. A function can be the
function the standard wrote and still be the wrong tool for hiding a
message, and a function can be the right tool in a larger construction
this page does not build. Both of those are open. The page closes only
the transcription it names.

### N12.24 The readiness check


Part 2 begins when you can do four things with a small complete program.
The index of this book states them, and it states what they are not. They
are demonstrated reasoning. They are not a certificate. Reading this
lesson does not confer them. The twelve exercises are the demonstration.
The last four are the four abilities, in the order the index names them:
interpret the program, derive an elementary operation, explain the domain
of a test, and locate an assumption in a claim.

Nothing in the answers is a grade. A wrong answer is a place to recompute.
The worked answers exist so that a recomputation has a finished path to
compare with, not so that the path can be skipped.

The four abilities have addresses in Listing N12.5, so the check is not a
mood. Interpreting the program means saying what `for i in 1..11` does to
`inner_block`, which is Exercise N12.9. Deriving an elementary operation
means producing a word or a byte from the stated inputs, as Exercises
N12.6, N12.7, and N12.8 do for the first column, the feed-forward, and
one serialization. Explaining the domain of a test means naming the key,
the nonce, and the block count the `Bool` actually mentions, and naming
an input it does not mention, which is Exercise N12.10. Locating an
assumption means pointing at C1 or C3 inside the claim that the bytes
matched, and refusing to let that match stand for §2.4, which is Exercise
N12.12. A reader who can do those four things on this listing has met the
Part 1 bar this book set. A reader who can only say that the tests passed
has met a different, smaller bar, and §N12.18 is the list of what that
smaller bar leaves open.

### N12.25 Work at the desk

**Exercise N12.1 — Two results the RFC prints.** From the sample numbers
of §2.1, compute `c + d`, the XOR with `b`, and the rotation by 7. Then
state the final `b` of §2.1.1. Why are the two results different words?

**Exercise N12.2 — The sum that crosses.** In the §2.2.1 quarter round,
`c + d1` is the addition that wraps. Give the integer sum, the multiple
of 2³², and the residue `c1`. Give one later addition in that same
quarter round that does not wrap.

**Exercise N12.3 — The spectators.** List the twelve indices that
`QUARTERROUND(2, 7, 8, 13)` does not write. Give the word that remains
at index 0, and the word that replaces index 8.

**Exercise N12.4 — Read the constants in sentence order.** Start from
`0x61707865`, `0x3320646e`, `0x79622d32`, and `0x6b206574`. Recover the
sixteen ASCII bytes in memory order, and write the characters. Which
four characters do you get if you instead read `0x61707865` from the
high byte on the left?

**Exercise N12.5 — Load, then store.** The key begins `00 01 02 03`.
Give the little-endian word. Then give the four bytes you would write
if you serialized that word little-endian. Explain why the second
sequence is the first sequence.

**Exercise N12.6 — The first column's residue.** On the §2.3.2 setup,
`QUARTERROUND(0, 4, 8, 12)` has one wrapping addition. Name it, give
the integer sum and the residue, and give the four final words in the
order `(a, b, c, d)`.

**Exercise N12.7 — Add the checkpoint.** The word at index 1 after
twenty rounds is `0xe238d763`. The setup word at index 1 is
`0x3320646e`. Compute the feed-forward residue. Which row of the table
in §N12.10 is this, and which RFC matrix does the residue belong to?

**Exercise N12.8 — Four bytes at offset 48.** Serialize `0xd19c12b5`
little-endian. Which index of the final matrix is that word, and which
offset in the serialized block holds its first byte?

**Exercise N12.9 — Interpret the loop.** In Listing N12.5, `rounds`
is `for i in 1..11 with acc: Word[32]^16 = s { inner_block(acc) }`.
List the integers `i` takes. Say what the body uses from `i`. Say what
the value of the loop is after the last step, in terms of `inner_block`
and `s`.

**Exercise N12.10 — The domain of one test.** The test
`RFC 8439 2.3.2 words` passed on the recorded run. State the key, the
nonce, and the block count that run used. State one input the test does
not mention. State what a pass does not say about that input.

**Exercise N12.11 — Read the diagnostic, then change one expression.**
For Listing N12.6, give the code, the computed range, and the legal
range. Give the minimal repair. Say why rewriting `quarter_round` is
not that repair.

**Exercise N12.12 — Locate what is still assumed.** Listing N12.5's
byte test matched the RFC's serialized block. Name two assumptions,
among C1–C3, that this match does not discharge. Name one construction
in RFC 8439 that the match does not transcribe. Say what the readiness
check is, and what it is not.

The integers used above are collected here so a later check can recompute
them from the operations rather than from the prose around them.

```text
n12-ledger
sample-sum = 2023406814
quarter-wrap-sum = 5461067566
quarter-wrap-residue = 1166100270
state-wrap-sum = 4692078087
state-wrap-residue = 397110791
column-wrap-sum = 6094103636
column-wrap-residue = 1799136340
feed0 = 3840405776
feed1-sum = 4653136849
feed1-residue = 358169553
```

## Worked answers

**N12.1.** The sample uses `c = 0x77777777` and the original `b` and `d`.
Byte by byte, `0x77 + 0x67 = 0xde`, `0x77 + 0x45 = 0xbc`,
`0x77 + 0x23 = 0x9a`, and `0x77 + 0x01 = 0x78`, with no carry out of a
byte. The integer sum is 2004318071 + 19088743 = 2023406814 =
`0x789abcde`, and 2023406814 < 4294967296, so the word sum is that
value. XOR with `0x01020304` is `0x7998bfda`: the bytes are
`0x79`, `0x98`, `0xbf`, `0xda`. The bits of `0x7998bfda` are
`0111 1001 1001 1000 1011 1111 1101 1010`. A left rotation by 7 moves
the high seven bits `0111100` to the low end and yields
`1100 1100 0101 1111 1110 1101 0011 1100`, which is `0xcc5fed3c`.

Section 2.1.1 keeps `a`, `b`, and `d`, replaces `c` by `0x9b8d6f43`,
and runs all eight updates. Its final `b` is `0xcb1cf8ce`. The sample
never forms `a1`, `d1`, `c1`, `b1`, `a2`, or `d2`. Its `c` is a
different word. Both results are what the RFC prints, for two different
sentences. The sample's rotated `b` is not a failed quarter round.

**N12.2.** In §2.2.1, `a1 = 0x7bc3d2fd` and
`d XOR a1 = 0x46a0c474`, so `d1 = 0xc47446a0` after the rotation by 16.
`c = 0x53372767 = 1396123495` and `d1 = 3295954592`. The integer sum is
4692078087. Subtract one modulus:
4692078087 − 4294967296 = 397110791 = `0x17ab6e07`. That residue is
`c1`. The later addition `c1 + d2` is 397110791 + 3435166841 =
3832277632 = `0xe46bea80`, and 3832277632 < 4294967296, so `c2` equals
the integer sum. Same rule, two different integers.

**N12.3.** The operation writes indices 2, 7, 8, and 13. The other
indices are 0, 1, 3, 4, 5, 6, 9, 10, 11, 12, 14, and 15. Index 0 keeps
`0x879531e0`. Index 8 receives `c2`, which is `0xe46bea80`. A
transcription that also wrote index 3 would be a different function
even if index 8 were right.

**N12.4.** Little-endian order puts the low byte of each word first.
`0x61707865` contributes `65 78 70 61`. `0x3320646e` contributes
`6e 64 20 33`. `0x79622d32` contributes `32 2d 62 79`. `0x6b206574`
contributes `74 65 20 6b`. The sixteen bytes are
`65 78 70 61 6e 64 20 33 32 2d 62 79 74 65 20 6b`, and the characters
are `expand 32-byte k`. Reading `0x61707865` from the high byte on the
left gives `61 70 78 65`, the characters `apxe`. Those are the same
four letters in the other order. The constant the RFC names is the
little-endian word, not the left-to-right reading of its hex digits.

**N12.5.** The bytes `00 01 02 03` have `0x00` as the first and therefore
least significant byte. The word is `0x03020100`. Serializing that word
little-endian writes the low byte first, so the bytes are `00 01 02 03`
again. Loading and storing are the same order. The hex display
`03020100` looks reversed only because hex writes the high byte on the
left. The byte sequence did not change.

**N12.6.** The inputs are `a = 0x61707865`, `b = 0x03020100`,
`c = 0x13121110`, and `d = 0x00000001`. Then `a1 = 0x64727965` and
`d1 = 0x79646472`. The sum `c + d1` is 319951120 + 2036622450 =
2356573570 = `0x8c767582`, which fits, so `c1` does not wrap.
`b1 = 0x474828f7` and `a2 = 0xabbaa25c`, which also fits.
`d2 = 0xdec62ed2`. The wrapping sum is `c1 + d2`:
2356573570 + 3737530066 = 6094103636 = 1 × 4294967296 + 1799136340,
and 1799136340 = `0x6b3ca454`. So `c2 = 0x6b3ca454` and, after the
rotation by 7, `b2 = 0x3a465196`. The four final words in order
`(a, b, c, d)` are `0xabbaa25c`, `0x3a465196`, `0x6b3ca454`,
`0xdec62ed2`. That is the expected tuple of the test in Listing N12.4.
The RFC does not print this line. The tuple's provenance is this
calculation.

**N12.7.** `0xe238d763 = 3795375971` and `0x3320646e = 857760878`.
The integer sum is 4653136849. Subtract 4294967296 once:
4653136849 − 4294967296 = 358169553 = `0x15593bd1`. This is index 1 of
the table in §N12.10, the first row marked `wrap`. The residue is the
word at index 1 of the RFC's matrix “ChaCha state at the end of the
ChaCha20 operation,” not the word of the after-rounds checkpoint and
not a byte of the serialized block. The serialized bytes of this
residue are `d1 3b 59 15`, which begin at offset 4.

**N12.8.** `0xd19c12b5` has low byte `0xb5`, then `0x12`, then `0x9c`,
then `0xd1`. The four bytes are `b5 12 9c d1`. The word is index 12 of
the final matrix, the block count after the feed-forward. Index 12
starts at byte `12 × 4 = 48`. The RFC's line marked `048` begins
`b5 12 9c d1`. The addition that produced the word did not wrap:
the checkpoint word `0xd19c12b4` plus the setup word `1` is
`0xd19c12b5`, and that integer is still below 2³². On a different
checkpoint word the same block count could wrap. This vector does not.

**N12.9.** The end bound is exclusive, so `i` takes 1, 2, 3, 4, 5, 6,
7, 8, 9, and 10. That is the RFC's `for i=1 upto 10`. The body is the
call `inner_block(acc)`. The call's only argument is the accumulator.
`i` is not read. After the last step the loop's value is `inner_block`
applied ten times to `s`, by Proposition N12.12. Replacing `1..11` by
`0..10` would take a different list of integers and the same number of
them. Because the body ignores `i`, the denoted state would be the same.
The spelling `1..11` is the one that names the integers the RFC names.
A body that used `i` as a rotation amount would make the two spellings
different functions. This body does not.

**N12.10.** `setup` in Listing N12.5 is the §2.3.2 input. The key is the
thirty-two bytes `00` through `1f`. The nonce is
`00:00:00:09:00:00:00:4a:00:00:00:00`. The block count is 1. One input
the test does not mention is the same key and nonce with block count 0,
which is a different word at index 12 and therefore a different state.
A pass says `words()` equalled the RFC's final matrix on the state that
was written. It does not say what `words()` returns on the block-count-0
state. It does not say the tenfold composition matches the standard on
every state. It does not say the evaluator implements addition modulo
2³² on every pair of words. Those would be different sentences, and
this test is not their evidence.

**N12.11.** The code is `ORC0223`. The message says the index runs from
1 through 16 and is out of range for `Word[32]^16`. The label says the
legal indices run from 0 through 15. The caret covers `i + 1`. Check,
eval, and test all stop there, and none of them prints a value or a
test report. The minimal repair is to replace `original[i + 1]` by
`original[i]`. The resulting step is the sum in `add_original` of
Listing N12.5. `quarter_round` does not appear in Listing N12.6. A
rewrite of it would change a function the diagnostic did not reject,
and it would leave the illegal index in place.

**N12.12.** The byte test can pass while C1, C2, and C3 remain
assumptions. Two of them are enough to answer: C1, that `+` on
`Word[32]` is addition modulo 2³², and C3, that `<<< n` is left
rotation by the literal `n`. The match shows that this run, under
whatever the evaluator did, produced the RFC's bytes. It does not prove
the evaluator's `+` and `<<<` from a smaller theory. If the evaluator
disagreed with C1 on some other pair of words, this test would not see
that pair.

The match does not transcribe §2.4, the ChaCha20 encryption algorithm,
which XORs the serialized block with plaintext. It also does not
transcribe Poly1305 or the AEAD construction. Any one of those is a
sufficient example. The serialized block is an input to those
constructions. It is not those constructions.

The readiness check is the four abilities exercised here: interpreting
Listing N12.5's loop, deriving an elementary operation such as the
feed-forward or the serialization, explaining the domain of the word
test, and locating C1 and C3 inside the claim that the bytes matched.
It is demonstrated reasoning on this page. It is not a certificate, and
it is not conferred by having turned the page. Finishing it does not
establish the claims §N12.18 places outside the study.

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


The block-function sections of the same RFC were consulted with the
quarter-round sections. §2.3 states the input layout and the constants.
§2.3.1 is the pseudocode, including the note that the prose and the test
vectors are normative if a conflict remains. §2.3.2 is the setup matrix,
the matrix after twenty rounds, the matrix after the feed-forward, and
the serialized block. The sixty-four bytes in Proposition N12.9 were
compared with that serialized block in the plain-text RFC on 2026-10-05.
The ASCII rendering to the right of the RFC's hex is not a second vector.

**[L1] Orange loops and fill literals.** `docs/LOOPS_2026.md` and the S3e
fixtures. A loop's bounds are integer literals. `[0; 64]` is the fill
form that slice accepts. An index built from the loop index with `+` and
`*` is proved in range before evaluation. The diagnostic quoted for
Listing N12.6 is `ORC0223`, including the computed range 1 through 16.
Implementation of the slice is not acceptance of the proposal.

**[T1] Orange tests.** `docs/TESTS_2026.md`, proposed under OEP-0020.
`orangec check` checks a test and does not run it. `orangec eval` runs
parameterless specs and does not run tests. `orangec test` runs the root
module's tests after the checks. A failed check prints the diagnostic and
no report. The titles in Listings N12.2 through N12.5 name the RFC
section or the hand proposition that supplied the expected value.

**[H1] Shifts.** Chapter 6 and `docs/EXPRESSIONS_2026.md`. `>> n` for a
literal `n` is the right shift used in `serialize`. Narrowing with
`as Word[8]` is the conversion from N7. The parentheses around each
shift are the grouping Listing N7.3 required for `as`.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

N12 adds twelve exercises with worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`. The Orange listings are
the six fenced programs in this file. `compiler/crates/orangec/tests/book_novice.rs`
runs them. Those checks do not establish a cryptographic security claim,
and they do not prove C1–C3.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction. It is the capstone of Part 1 on
`book/novice-journeyman-master-opening`. Owner review is pending. No
deployment recommendation, no certificate of competence, and no
proof-checker acceptance is made.
