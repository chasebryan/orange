# The Orange Book

By Chase Bryan

Opening manuscript for the three-part book. Draft 2026-10-05.

## A word before we begin

You do not need to arrive knowing how to program. You do not need to know
what a cryptographic key is, why computers use binary, or what makes a proof
different from a persuasive explanation. Those are things I intend to teach
you, not conditions for being allowed to begin.

Keep somewhere to work beside the book. Paper is sufficient for these opening
chapters. Writing, typing, or recording your reasoning aloud all serve the
same purpose: they make your thought available for inspection. You will need
that more often than you need a faster computer.

When I ask you to predict an answer, stop before reading mine. A prediction
makes a misunderstanding visible. When yours is wrong, find the step that
produced it. Crossing out the final number without examining the reasoning
leaves the useful part of the mistake untouched.

I will give each new idea a name after giving you something to attach the name
to. Some definitions have numbers so we can return to them precisely. The
number is an address, not something to memorize. Exercises have addresses,
too. A reference such as **Exercise 2.4** means the fourth exercise in Chapter 2.

There are three parts to this book: **Part 1, The Novice**; **Part 2, The
Journeyman**; and **Part 3, The Master**. They describe stages of study, not
kinds of people. Start where you can explain the reasoning, not merely
recognize the vocabulary. Returning to a foundation is not a demotion.

The opening chapters build the language we will need: messages, rules,
numbers, representations, and logical operations. Orange will give us a way
to express increasingly substantial constructions. The later chapters ask
what connects a mathematical description to an implementation and what
justifies trusting that connection. The existing book's investigations of
meaning, evidence, compilers, and trust remain part of that journey.

I will distinguish a definition from a fact, an example from a general
argument, and a proposed feature from one that has been built. Orange is a
living, pre-alpha project. A sentence in this book cannot grant its compiler
a capability it does not have. The project's normative documents and accepted
decisions control its language; this book teaches you to read and question
them. It is not itself a language specification, proof certificate, security
assurance, license grant, or product release.

The exercises are part of the explanation. Their answers appear after the
three chapters, with reasons rather than scores. Read them after making a
serious attempt. Then close the answer and reconstruct the argument yourself.

We will begin with a message. No machine is required yet.

## Part 1, The Novice

## Chapter 1: Before You Hide Anything

> “the enemy knows the system being used.”
>
> — Claude E. Shannon, *Communication Theory of Secrecy Systems* (1949),
> §2, p. 662; excerpt. [S1]

### 1.1 A message and an unwanted reader

Write these words:

```text
MEET AT THE BRIDGE
```

Imagine that you want one friend to read them, but not the person carrying
the paper. Nothing about the sentence itself makes that possible. Anyone
who sees it and understands the words can learn where you intend to meet.

The problem is not that the message lacks meaning. It has exactly the meaning
you need. The problem is that the wrong person can obtain it.

For this example, call the person sending the message **the sender**, the
friend **the receiver**, and the person trying to learn what you are hiding
**the adversary**. These are roles. The sender need not be a human typing at
a keyboard, and an adversary need not look suspicious. For now, three people
and a sheet of paper are enough.

Tell me what the adversary can do. Can they only look at the paper? Can they
replace it? Can they ask you to send another message? Those are different
problems. We will begin with the first: the adversary can see a copy of
what you send.

That restriction is an **assumption**: something we take as given while
examining this particular problem. It is not a promise that a real carrier
will behave so politely.

**Definition 1.1 — Confidentiality.** Confidentiality concerns keeping
information from parties who are not authorized to learn it. A precise claim
must identify the information, the parties, and the circumstances it covers.

Here, the information is the meeting location. The intended receiver is
allowed to learn it; the carrier is not. We have not yet said how to achieve
that separation. We have only made the question specific enough to work on.

### 1.2 Changing the appearance

Try reversing the characters, including the spaces:

```text
MEET AT THE BRIDGE
EGDIRB EHT TA TEEM
```

A **character** is an individual symbol in our written message. Here, each
letter and each space occupies one position. To reverse the message, read
those positions from the last to the first.

The result looks less familiar. Your friend can recover the original by
reversing it again. So can the carrier, once the carrier knows the rule.

You have changed the appearance of the message without creating a difference
between what your friend can do and what the carrier can do. Both have the
same written material and the same method of recovery.

Now hide the reversal rule. Perhaps the carrier fails to notice it. That is a
possible event, but it is not the same claim as protection against someone
who understands your method. We must not improve the description of the
adversary halfway through the argument merely because our design needs a
less capable opponent.

Shannon's opening quotation asks us to examine a system with its method
already known to the opponent. It is an analytical assumption, not a claim
that every opponent actually knows every system. The value is that it
removes a convenient excuse: *perhaps no one will understand what we did*.

Return to the paper. Under that assumption, reversal gives the receiver no
advantage. This follows from the example's rules; no claim about every
possible cryptographic system is needed.

### 1.3 A rule and a key are different things

Imagine a cabinet with many possible lock settings. Knowing how its lock is
constructed is different from knowing which setting opens this cabinet.
That distinction suggests a better question for our message: can the
receiver possess something useful that the observer lacks?

In the shared-secret setting we will study first, that additional input is
a **key**: a value shared by the sender and receiver and intended to remain
unknown to the adversary. The method describes how to use a key; the selected
key determines a particular use of the method. Merely adding a key does not
make the method secure. It gives us something whose role we can examine.

The cabinet is an analogy, not a cryptographic argument. A physical lock can
be cut away; a copied digital value does not become a second metal key. Keep
the distinction between public mechanism and selected setting, and leave
the rest of the cabinet behind.

A message supplied to an encryption method is called **plaintext**. The
result is called **ciphertext**. **Encryption** computes that result;
**decryption** attempts to recover the original using the required
information. Plaintext need not be ordinary language. Later, it may be a
sequence of numbers representing a file.

These names describe roles in a construction. Calling an output
*ciphertext* is not evidence that the construction protects it.

There are two separate questions. Can the intended receiver recover the
message? What can the adversary learn? A method might answer the first
perfectly and fail the second completely. Our reversal experiment already
demonstrates that distinction.

### 1.4 Describe the operation before trusting the result

Suppose I give you this instruction:

> Reverse the message.

Does that mean reverse the letters inside each word, reverse the order of
the words, or reverse every character including spaces? We selected the last
meaning in §1.2, but the short instruction alone does not tell you that.

A **specification** states the required behavior precisely enough for the
purpose at hand. Here is one for our paper exercise:

> Treat the message as a finite sequence of characters. Copy its characters
> from last to first, preserving each character exactly. For an empty
> message, produce an empty message.

A **sequence** is an ordered collection: which item comes first, second,
and so on matters. **Finite** means it has a limited number of items.
**Empty** means it has none. An empty message and a message containing one
space are therefore different inputs.

The material you give an operation is its **input**. What it produces is its
**output**. An **implementation** is a concrete way of carrying out the
specified operation. You could implement this reversal with paper, cards,
or a computer program.

The specification says what must happen. An implementation may or may not
do it correctly. Giving both the same name does not establish agreement.

Try the input `AB C`. The specified output is `C BA`. Reversing the letters
inside each word, while leaving the words in place, returns `BA C`.
Reversing the order of the words returns `C AB`. The three results differ,
so this input separates the three readings the short instruction allowed.

### 1.5 What one successful example establishes

You reversed the meeting message twice and recovered the original. That is
one successful example. Now consider the stronger statement:

> Reversing any finite sequence twice restores that sequence.

The word *any* adds an obligation. We now owe an argument about all the
permitted sequences, not a report about the one on our desk.

Take an arbitrary sequence: any one you like, without selecting it for a
special property. On the first reversal, the first character becomes last,
the second becomes second-last, and so on. On the second reversal, each
character returns to the position it had before. No character changes and
none is removed. The empty sequence also returns unchanged, because both
operations have no characters to copy.

That reasoning does not depend on the letters in the meeting message or on
its length. It covers the specified operation on every finite sequence.
This is a **proof**: an argument establishing a stated conclusion from the
stated rules and assumptions.

Notice what the argument does not establish. It does not show that an
unknown computer program really performs this reversal. It does not show
that the carrier cannot recover the message. We proved a particular
property of a particular operation. Its scope does not expand because the
proof is convincing.

A **test** examines behavior on selected cases. Tests help us find mistakes
and collect evidence about implementations. In a finite, precisely defined
model, a correct examination of *every* case can also establish a universal
claim about that model. The distinction is not “machines versus reasoning.”
It is what was covered and what follows from that coverage.

If we tested ten messages, we covered ten messages. If we proved the statement
above, we covered every finite sequence under the definition. If we executed
a program, we also relied on the machinery that performed the execution.
Keep those subjects separate.

### 1.6 Hiding a message is not every kind of protection

Change the situation. The carrier can now replace your paper with another
one. Your friend receives:

```text
MEET AT THE TOWER
```

The new concern is whether the received message was altered. That is a
question of **integrity**. Another concern is whether the message came from
an accepted source. That is a question of **authenticity**. For our purposes,
we need to be able to distinguish an authorized message from the carrier's
substitute. These are goals to investigate, not properties our reversal
method possesses.

Neither question is answered merely by making a message unreadable to the
carrier. Even detecting changes would not force the carrier to deliver the
paper. Protection against reading, alteration, impersonation, and
non-delivery must not be silently bundled into one reassuring word.

There is a further limit. Suppose a check establishes that a message was
created by someone holding a particular secret key. That alone does not
establish which human held it. To make an identity claim, you would also need
an account of how the key is associated with that person and who else could
use it.

You do not need the mechanisms yet. You need to keep the questions distinct.
This is the first habit I want you to carry into the rest of the book:
when you hear that something is protected, ask what the protection is
supposed to prevent.

### 1.7 Work at the desk

Use invented messages throughout these exercises. No real password or
private information is needed.

**Exercise 1.1 — Follow the specified rule.** Reverse `AB C`, including its
space. Reverse the result. Then give an input for which reversing the letters
inside each word would differ from reversing the complete sequence.

**Exercise 1.2 — Distinguish the inputs.** Explain why an empty sequence,
one space, and the character `0` are three different inputs. What does our
reversal specification produce for each?

**Exercise 1.3 — Name the missing claim.** A designer says, “Every message I
tried came back correctly, so no outsider can read it.” Identify two steps
in the conclusion that the reported evidence does not establish.

**Exercise 1.4 — Change the opponent.** For the meeting message, describe
one concern when the carrier may only observe and one additional concern
when the carrier may replace the paper. Do not propose a mechanism yet.

**Exercise 1.5 — Defend the general statement.** Explain why reversing twice
restores a sequence containing repeated letters. Your explanation must not
rely on recognizing each letter as unique.

**Exercise 1.6 — Inspect the subject.** You have proved the mathematical
reversal property. A program named `reverse` fails on an empty input. Is your
proof contradicted? Explain what has and has not been established.

You can now separate a message from its appearance, a requirement from its
implementation, and a successful example from a general argument. Next we
need a way to describe the characters themselves. Begin with two marks.

## Chapter 2: Two Marks, Many Possibilities

> “The significant aspect is that the actual message is one selected from a set of possible messages.”
>
> — Claude E. Shannon, *A Mathematical Theory of Communication* (1948),
> introduction, p. 1 of the corrected reprint. [S2]

### 2.1 A number is not its written form

Place five marks on your paper:

```text
| | | | |
```

The quantity is five. You can write it as `5`, spell it as *five*, or leave
it as five strokes. Those are different representations of the same quantity.
A **representation** is a way of expressing something using selected symbols
and rules for interpreting them.

The distinction becomes important when an unfamiliar representation appears.
New symbols do not necessarily mean new mathematics.

A **digit** is one symbol used to write a number in a numeral system. Our
usual decimal system uses ten digits, `0` through `9`. Zero says that a
quantity is absent at a particular place; it is not an absence of notation.
The written numeral `105` uses its zero to distinguish one hundred five
from fifteen.

In decimal notation, a place gives a digit its weight. In `23`, the `2`
represents two tens and the `3` represents three ones. Put them together and
you have twenty-three.

The sign `+` means addition: combining quantities. The sign `=` means the
expressions on its two sides have the same value. We can therefore write:

```text
23 = 20 + 3
```

The sign `×` means multiplication. For the whole-number examples here,
`2 × 10` means two groups of ten. So the same calculation is:

```text
23 = (2 × 10) + (3 × 1)
```

The parentheses group a calculation. They tell you to treat what is inside
as a unit before combining it with the rest. We will use them whenever the
grouping should not be left to guesswork.

Read the equation aloud. If the symbols are getting ahead of the meaning,
return to the groups of ten and the individual marks. The symbols are a
shorter way to record that reasoning.

### 2.2 What happens when the digits run out?

After decimal `9`, there is no single digit for the next quantity. We use
`10`: one group of ten and no leftover ones. After `99`, both positions are
full, so the next quantity is written `100`.

Now allow only two digits, `0` and `1`. You can write zero and one. For two,
you must already introduce a second position:

```text
Decimal:   0   1   2   3    4    5    6    7     8
Binary:    0   1  10  11  100  101  110  111  1000
```

This is **binary**, or **base-two**, notation. A **base** tells us how many
digit values a positional numeral system uses. Moving one place to the left
multiplies the place value by the base. Decimal uses ten; binary uses two.

Thus binary positions have weights one, two, four, eight, sixteen, and so
on, reading from right to left. Each weight is twice the previous one.

Do not pronounce binary `10` as *ten* while you are learning the notation.
Call it *one-zero*, then determine its value from its base. The same written
digits can represent different quantities when the interpretation changes.

**Definition 2.1 — Bit.** A bit is a binary digit: one value chosen from `0`
and `1`. A **bit string** is a finite sequence of bits. Its **length** is the
number of positions in that sequence.

The bit string `0010` has four positions. Interpreted as an unsigned binary
number, its value is two. **Unsigned** here means that the representation
has no provision for negative values. The leading zeros do not increase
its numerical value, but they remain part of the four-position string.

Consequently, `10` and `0010` can represent the same unsigned number while
remaining different bit strings. Their lengths differ. Never remove leading
zeros from a fixed-length object simply because the number still looks right.

### 2.3 Read a binary number without guessing

Put the bit string `1101` below its place values:

```text
Place value:   8   4   2   1
Bit:           1   1   0   1
Contribution:  8   4   0   1
```

A `1` includes the quantity assigned to that position. A `0` includes none
of it. The value is therefore eight plus four plus one: thirteen.

Now reverse the task. Write thirteen using four binary positions. Begin with
the largest place value, eight. It fits into thirteen, so write `1` there.
Five remains. Four fits into five, so write another `1`. One remains. Two
does not fit into one, so write `0`. Finally, one fits: write `1`.

You have reconstructed `1101`. Subtraction was the bookkeeping: the sign
`−` means taking a quantity away, so thirteen minus eight leaves five. At
every step, keep track of what the chosen positions already represent and
what remains to be represented.

Try nine before continuing. With place values eight, four, two, one, you
should obtain `1001`: eight and one, with neither four nor two.

This method terminates with the ones position for whole numbers that fit in
the positions available. A **whole number** here means zero or a positive
counting number. Fractions and negative numbers require further conventions;
we have not claimed that this representation covers them.

### 2.4 Count the possibilities before choosing one

One position has two possible values. With two positions, each first value
can be followed by either second value:

```text
00   01   10   11
```

That makes four strings. With three positions, put `0` before each of those
four strings, then put `1` before each. You have eight strings, with no
repetition and none missing.

The same step works at every finite length. From each existing string, form
one new string by writing `0` in front and another by writing `1` in front.
Every string of the new length begins with exactly one of those bits and
continues with exactly one old string. The construction therefore misses
none and counts none twice, so the new count is twice the old count. Adding
a bit does not merely add one possibility.

The counts below are that doubling, from one position through eight.

```text
Positions:       1   2   3    4    5    6     7     8
Possible strings:2   4   8   16   32   64   128   256
```

We abbreviate repeated multiplication using a **power**. The notation `2³`,
read *two to the third power*, means `2 × 2 × 2`, which is eight. In general,
`2ⁿ` means a product containing `n` factors of two. The letter `n` stands for
the chosen number of positions. Such a letter is a **variable**: a name whose
value is supplied by the situation under discussion.

For zero positions there is one possible string: the empty string. That
agrees with the convention `2⁰ = 1`. There is one way to choose nothing; it
is not the same as having no possible choice at all.

**Definition 2.2 — Byte.** In this book, a byte consists of eight bits.
There are `2⁸ = 256` possible byte values. As unsigned numbers, they range
from zero through 255, inclusive.

Why not through 256? Because zero already occupies one of the 256
possibilities. Counting the integers from zero through 255 gives 256 values.
The number of available values and the largest available value are not the
same quantity.

Now inspect a byte whose value is already known to an observer. It still
occupies eight bit positions, but those positions do not leave the observer
with 256 equally plausible possibilities. Storage width and uncertainty are
different things. Our counting argument describes what the format permits;
it does not say how a value was selected or who knows it.

### 2.5 Four bits at a time

Long binary strings are difficult to copy accurately. We can write them more
compactly by using a digit system with sixteen values. It is called
**hexadecimal**, or **base sixteen**, usually shortened to **hex**.

Decimal gives us only ten familiar digit shapes. Hex uses `A` through `F`
for the remaining six values:

| Decimal value | Four-bit binary | Hex digit |
| --- | --- | --- |
| 0 | `0000` | `0` |
| 1 | `0001` | `1` |
| 2 | `0010` | `2` |
| 3 | `0011` | `3` |
| 4 | `0100` | `4` |
| 5 | `0101` | `5` |
| 6 | `0110` | `6` |
| 7 | `0111` | `7` |
| 8 | `1000` | `8` |
| 9 | `1001` | `9` |
| 10 | `1010` | `A` |
| 11 | `1011` | `B` |
| 12 | `1100` | `C` |
| 13 | `1101` | `D` |
| 14 | `1110` | `E` |
| 15 | `1111` | `F` |

Each row connects three representations of one value. `D` is not a secret
replacement for thirteen. In this notation, it is the digit for thirteen.

Four bits have exactly sixteen possible strings, so one hex digit can
represent exactly one four-bit group. Two hex digits represent a byte:

```text
Binary groups:  1010  0110
Hex digits:       A     6
```

The spaces above separate groups for your eyes; they are not extra bits.
The hex numeral `A6` has the decimal value

```text
(10 × 16) + 6 = 166.
```

We will write `0xA6` when a prefix is useful to announce hexadecimal. A
**prefix** is notation placed before what it qualifies. Here, `0x` tells
you how to read the following digits; it contributes no additional bits to
the value. The letters may also appear in lowercase: `0xa6` represents the
same number under this convention.

For binary, we will sometimes use the prefix `0b`. Thus these three
notations have the same numerical value:

```text
0b00001101 = 0x0D = 13
```

The two prefixed numerals make the interpretation explicit. The final,
unprefixed number is decimal. The binary display also shows eight positions,
which may matter when the object is a byte rather than an unrestricted
whole number.

### 2.6 Bits do not carry their interpretation on their faces

Write the byte `00001101`. Under the unsigned numerical interpretation,
you know its value: thirteen.

Now imagine a device with eight indicator lamps. Use one bit per lamp:
`1` means lit and `0` means unlit. The same pattern describes which lamps
are lit. We have not changed the bits; we have changed what the positions
refer to.

You could instead define a small message table assigning a sentence to each
byte value. Give thirteen the sentence `THE DOOR IS OPEN`. The byte now
selects that sentence under your table. A person with the same table can
interpret it. A person given only the byte has not been given the table.

A **character encoding** similarly establishes rules relating characters
to representations. We will examine actual text encodings later. For now,
do not silently turn an arbitrary byte into a letter, a number, or an
instruction. Each reading requires a convention.

That does not mean representations are arbitrary during an actual
conversation between programs. Once a format is specified, the participants
must obey it. Freedom to design a representation is not permission to
reinterpret someone else's message after receiving it.

Return to your meeting message. Before a computer can operate on it, there
must be an account of how its characters are represented. Changing that
representation does not, by itself, establish confidentiality. If an
observer has the same reversible conversion rule, the observer can apply it.
This is why writing text as numbers should never be offered as a security
argument merely because the result looks unfamiliar.

### 2.7 A short audit of a byte

You receive this description:

> The value is `0xFF`. It contains 256 possibilities and is therefore random.

Separate the statements.

`0xFF` is a particular value. Both hex digits are fifteen, so its unsigned
value is `(15 × 16) + 15 = 255`. Its eight-bit representation is `11111111`.

The *byte format* permits 256 different values. This particular byte is one
of them. The description gives no account of a selection process. It
therefore supplies no justification for the word *random*.

You have rejected an unsupported conclusion without needing an advanced
theory of randomness. You only had to keep the object, its representation,
and the process that selected it apart.

### 2.8 Work at the desk

**Exercise 2.1 — Read the places.** Convert `10110` to decimal. Write the
place value above every bit and show each contribution.

**Exercise 2.2 — Build the representation.** Write nineteen as an eight-bit
unsigned binary string and as two hexadecimal digits. Show the remainder
after selecting each nonzero binary place.

**Exercise 2.3 — Count, then find the maximum.** How many five-bit strings
are possible? What is the largest unsigned number represented by five bits?
Explain why your two answers differ by one.

**Exercise 2.4 — Separate equality from identity.** In what sense are `1`
and `00000001` equal? In what sense are they different? State the
interpretation that makes each answer true.

**Exercise 2.5 — Work across groups.** Convert `0x3C` to an eight-bit binary
string and to decimal. Explain the contribution of each hex digit.

**Exercise 2.6 — Examine the empty case.** List all zero-length bit strings.
How many have you listed? Explain why the answer is not zero.

**Exercise 2.7 — Inspect an unsupported conclusion.** Someone shows you a
32-position string containing both zeros and ones and calls it random.
Which fact can you check by looking, and which claim needs information about
how the value was selected?

**Exercise 2.8 — Repair a representation claim.** A file is displayed as
hexadecimal. Its owner says it has therefore been encrypted. Explain what
the display demonstrates and what it does not.

**Exercise 2.9 — Establish the counting rule.** Explain why adding one
unrestricted bit doubles the number of possible strings. Your explanation
must show both that every new string is accounted for and that none is
counted twice.

You can now read the marks. Next, make a rule act on them.

## Chapter 3: A Rule You Can Undo

> “Anyone, from the most clueless amateur to the best cryptographer, can create an algorithm that he himself can’t break.”
>
> — Bruce Schneier, “Schneier's Law” (2011), quoting his 1998 formulation. [S3]

### 3.1 Start with a question that has two answers

Take the statement “these two bits are equal.” Given their values, you can
answer either true or false. In the two-valued logic used here, those are
the available **truth values**.

A **Boolean value** is one of those two values. We may represent false by
`0` and true by `1`, but this is a chosen interpretation of the digits.
It does not make every byte containing a one into the English word *true*.
The numerical and logical readings must still be distinguished.

A **logical operation** produces a truth value according to a stated rule.
Begin with **NOT**, which changes true to false and false to true:

| Input | NOT input |
| --- | --- |
| `0` | `1` |
| `1` | `0` |

This is a **truth table**. It lists every allowed input and the result
assigned to it. There are only two inputs here, so the table defines the
operation completely.

An **operator** is a symbol or word naming an operation. An **operand** is
a value the operation acts on. NOT takes one operand. We will next define
operations that take two.

### 3.2 Three ways to combine two bits

With two input bits, there are four input pairs. Keep the first and second
positions distinct, even when the operation happens to give the same result
in either order.

**AND** gives `1` only when both inputs are `1`. **OR**, in its inclusive
sense, gives `1` when at least one input is `1`; that includes the case where
both are. **XOR**, pronounced *ex-or* and short for *exclusive OR*, gives `1`
when exactly one of the two inputs is `1`.

| First bit | Second bit | AND | OR | XOR |
| --- | --- | --- | --- | --- |
| `0` | `0` | `0` | `0` | `0` |
| `0` | `1` | `0` | `1` | `1` |
| `1` | `0` | `0` | `1` | `1` |
| `1` | `1` | `1` | `1` | `0` |

Read the final row carefully. OR allows both inputs to be one. XOR does not.
This is the precise difference that the word *exclusive* contributes.

You can also read XOR as a test for difference: equal input bits give zero;
different input bits give one. That is the same operation described from
another angle, not an additional rule you have to memorize.

The phrase “exactly one” describes the two-input definition. When we later
combine three or more bits by repeated XOR, do not assume that the result
still means exactly one input was one. We will calculate what repeated
application actually does.

### 3.3 From a bit to a string

A **bitwise** operation applies a bit rule separately at corresponding
positions. For now, both strings must have the same length. Put one beneath
the other so the positions line up:

```text
First input:   1011
Second input:  0110
XOR result:    1101
```

At the first position, `1 XOR 0` gives `1`. At the second, `0 XOR 1` gives
`1`. At the third, `1 XOR 1` gives `0`. At the fourth, `1 XOR 0` gives `1`.
Nothing is carried from one position to the next.

That last fact distinguishes bitwise XOR from ordinary integer addition.
Under the unsigned interpretation, the inputs above are eleven and six,
while the result is thirteen. Eleven plus six is seventeen, not thirteen.
A similar-looking symbol cannot be allowed to change the operation silently.

For mathematical discussion, I will write XOR as `⊕`. The previous example
is therefore:

```text
1011 ⊕ 0110 = 1101
```

Here the digit groups are bit strings, not decimal numerals. The symbol
`⊕` is mathematical notation in this book, not a line of Orange source.
We will introduce executable syntax separately.

### 3.4 One operand as a set of instructions

Hold the second input fixed and examine just two rows at a time.

When the second input is `0`, the output equals the first input: `0` remains
`0`, and `1` remains `1`. When the second input is `1`, the output is the
opposite of the first: `0` becomes `1`, and `1` becomes `0`.

So XOR with zero means **leave this bit alone**. XOR with one means
**flip this bit**. To flip a bit is to replace it with its opposite value.

A bit string used to select such per-position behavior is often called a
**mask**. In our example, the mask `0110` leaves the first and fourth
positions alone and flips the second and third.

This is a useful way to predict the result without reciting four table
lookups. It also exposes why XOR can undo itself.

Apply the same mask again:

```text
Original:       1011
Mask:           0110
First result:   1101
Same mask:      0110
Second result:  1011
```

Every unchanged position stays unchanged. Every flipped position flips
back. We recovered the original string without storing a separate copy
of it inside this calculation.

We did retain the mask. Do not leave that fact out of the explanation.

### 3.5 Prove the rule, not just this example

**Proposition 3.1 — Cancellation with a retained mask.** For any two
finite, equal-length bit strings `x` and `k`, applying XOR with `k` twice
returns `x`:

```text
(x ⊕ k) ⊕ k = x.
```

The letters name arbitrary strings, not special values. The parentheses
say to compute `x ⊕ k` first, then XOR that result with `k`.

**Proof.** Choose any position. If the bit of `k` there is zero, neither
application changes the corresponding bit of `x`. If it is one, the first
application flips that bit and the second flips it back. Those are all
possible values for a bit of `k`. Thus every position ends with its original
value. XOR preserves the number and order of positions, so the complete
result is `x`. For empty strings, there are no positions to alter, and the
result is again the empty string.

The argument covers every allowed length and every allowed pair of strings.
It does not rely on `1011`, on the number thirteen, or on a mask containing
exactly two ones.

Now narrow the conclusion to what we actually established. Given the output
and the retained mask, we can recover the input. We have not established
that an observer who lacks the mask cannot learn the input. That would need
a different argument, including an account of how the mask is selected,
what the observer knows, and how the construction is used.

A publicly known all-zero mask satisfies Proposition 3.1 while hiding
nothing. This is a counterexample to the assertion that reversibility
alone establishes secrecy. It is not a rule that a random secret mask must
never happen to be all zeros. Selection, disclosure, and a particular
selected value are different issues.

### 3.6 Where did the other input go?

You may notice a tension. Two input bits go into XOR, but only one output
bit comes out. How can the operation be reversible?

It is not reversible as a way of recovering *both unknown inputs from the
output alone*. The output `1` could have come from `0 XOR 1` or `1 XOR 0`.
The output `0` could have come from `0 XOR 0` or `1 XOR 1`. Each possible
output leaves two possible input pairs.

Proposition 3.1 makes a narrower statement. It holds one input, the mask,
available. Once that input is known, the output determines the other input.
The missing information has not been conjured out of the result. It was
retained in `k`.

Compare AND. If the retained second operand is `1`, AND leaves the first
operand unchanged, so you can recover it. If the retained second operand is
`0`, the result is always `0`, whether the first operand was `0` or `1`.
Recovery is then impossible from those two known values alone.

This does not make AND a bad operation. It makes it a different operation.
Whether discarded information is a defect depends on what you required
of the computation.

### 3.7 Grouping is part of the calculation

Calculate these expressions using the truth table:

```text
(1 XOR 1) XOR 1
1 XOR (1 XOR 1)
```

Both give one. The first combines the left pair, producing zero, and then
combines zero with one. The second combines the right pair first, with the
same final result.

An **even** whole number can be divided into pairs without anything left
over. An **odd** whole number leaves one over. The word **parity** names this
even-or-odd distinction.

For XOR, this agreement holds for every three input bits. We can establish
it by listing all eight cases, or by observing that each input one toggles
the accumulated result, starting from zero. An even number of ones leaves
zero; an odd number leaves one. The number of ones, not their grouping,
determines the result. Exercise 3.7 asks you to check the complete table
rather than accept that observation without inspection.

Repeated XOR of bits records the parity of the number of ones. Three ones
therefore give one, despite not containing exactly one one.

Do not generalize this freedom of grouping to unrelated operations. Compare:

```text
(1 OR 0) AND 0 = 0
1 OR (0 AND 0) = 1
```

The parentheses describe different computations. We obtained different
results. A computer language needs rules to resolve grouping, but those
rules must be learned from that language rather than guessed from how an
expression looks.

### 3.8 What your experiment can honestly say

Suppose you test cancellation for every pair of byte values. There are
256 choices for the first byte and, for each, 256 choices for the mask.
That makes `256 × 256 = 65,536` pairs.

If a correct checker examines every one of those pairs and finds that
cancellation holds, it establishes the property for that complete
byte-sized model. It has not checked every longer string. Proposition 3.1
covers arbitrary finite equal lengths because its argument applies at each
position, regardless of how many positions there are.

There is another difference. A program implementing the checker runs
through some machinery. A failure in that machinery could invalidate what
you think was examined. A hand proof also requires scrutiny: a missing
case or an unjustified step can invalidate it. Neither the word *tested*
nor the word *proved* should prevent you from asking to see the work.

For these opening exercises, the executable checks accompanying the book
use Python as a reference calculation. They do not execute Orange and do
not certify its compiler. Their role is to catch transcription and example
errors while the mathematical argument states the broader property.

Schneier's quotation at the opening concerns the limits of a designer's
own inability to break a construction. Our small result illustrates the
same discipline of scope. You can now prove something useful about XOR.
That achievement does not require you to pretend that you have proved
something else about an encryption system.

### 3.9 Work at the desk

**Exercise 3.1 — Recover the definition.** Without looking back, write the
four rows of the two-input XOR table. Explain the final row in words.

**Exercise 3.2 — Compare operations.** For `1010` and `1100`, compute
bitwise AND, OR, and XOR. Keep all four positions in each result.

**Exercise 3.3 — Use a mask.** Apply the mask `00111100` to `10100110` using
XOR. Predict which positions change before calculating the output. Apply
the mask again to check your answer.

**Exercise 3.4 — Retain the right information.** An XOR result is `1101`
and the retained mask is `0110`. Recover the original input. Then explain
why the result alone would not determine the original input.

**Exercise 3.5 — Find a counterexample.** Someone claims that applying AND
with the same mask twice always restores the original input. Give a
one-bit counterexample. Explain why one counterexample is sufficient to
reject a statement that says *always*.

**Exercise 3.6 — Inspect the missing condition.** Does Proposition 3.1
specify an operation on `101` and `11`? Could a designer extend the operation
to those inputs? Distinguish “undefined by this specification” from
“impossible to define.”

**Exercise 3.7 — Check grouping exhaustively.** List all eight triples of
bits. For each, calculate `(a XOR b) XOR c` and `a XOR (b XOR c)`.
Compare the results and state exactly which domain your table covers.

**Exercise 3.8 — Separate properties.** Explain why a reversible operation
with a publicly known all-zero mask does not establish confidentiality.
State the narrower property that still holds.

**Exercise 3.9 — Change the scale.** Why are there 65,536 byte-and-mask
pairs? Why does checking them all not, by itself, check every pair of
sixteen-bit strings?

**Exercise 3.10 — Read the quantifiers.** Explain the difference between
“I found one input that works,” “every permitted input works,” and “there
is a permitted input that fails.” Which two statements cannot both be true?

Your next task is to express an operation as a complete program: say which
values it accepts, what it returns, and what every symbol in its source
means. We will not ask the machine to fill in an intention we failed to
write down.

## Answers and worked reasoning

These answers cover the exercises above. They are not substitutes for your
attempt. When an answer differs from yours, compare the rule used at the
first differing step.

### Chapter 1 answers

**1.1.** The reversal is `C BA`; reversing again gives `AB C`. For the input
`AB CD`, reversing within each word gives `BA DC`, while reversing the whole
sequence gives `DC BA`. The methods implement different requirements.

**1.2.** Their lengths are zero, one, and one. The latter two contain
different characters: a space and the digit character `0`. Each reverses
to itself, but that does not make the three inputs identical.

**1.3.** Tests of selected messages do not establish correct recovery for
every permitted message. Correct recovery, even if proved universally,
does not establish that an outsider cannot also recover the message.
The report also leaves its tested inputs and implementation unspecified.

**1.4.** Observation raises the concern that the meeting location becomes
known to the carrier. Replacement adds the concern that the receiver acts
on a substituted location. These require different claims; merely stating
that a message is confidential does not settle the replacement case.

**1.5.** The reversal argument tracks positions rather than unique letter
identities. Each position is moved to its opposite position and then back.
Equal characters in different positions do not invalidate that reasoning.

**1.6.** The proof concerns the specified mathematical operation, including
its empty case. The program's failure shows that the program does not meet
that specification on the reported input, assuming the failure report is
accurate. Its name does not make it the operation proved correct.

### Chapter 2 answers

**2.1.** The weights are sixteen, eight, four, two, one. The contributions
are sixteen, zero, four, two, zero. The total is twenty-two.

**2.2.** Nineteen minus sixteen leaves three; three minus two leaves one;
one minus one leaves zero. The eight-bit representation is `00010011`.
Grouped by fours, that is `0001 0011`, or `0x13`. The leading hex digit
contributes `1 × 16 = 16`, and the last digit contributes `3`, so
`(1 × 16) + 3 = 19`. A leading `1` in the decimal numeral 19 would
contribute ten instead.

**2.3.** Five unrestricted positions give `2⁵ = 32` strings. The largest
unsigned value is `11111`, or `16 + 8 + 4 + 2 + 1 = 31`. Zero is one of
the 32 values, which is why the maximum is one less than the count.

**2.4.** Under unsigned binary interpretation, both have numerical value
one. As bit strings, their lengths differ: one position versus eight.
Equality of interpreted numerical values is not identity of stored strings.

**2.5.** `0x3C` is `00111100`: `3` corresponds to `0011`, and `C` to `1100`.
Its decimal value is `(3 × 16) + 12 = 60`.

**2.6.** There is exactly one string with no positions: the empty string.
Writing it as a pair of empty quotation marks is one way to name it; the
quotation marks are notation, not contents of the string. There are no
bits to choose differently, so no second zero-length string exists.

**2.7.** You can check the number of positions and whether both digit
values occur. Randomness is a claim about selection, not a property
established by this visual mixture. Many fully predictable procedures
produce mixed strings.

**2.8.** The display provides a hexadecimal representation. Without
additional facts, it demonstrates no secret-dependent protection. Anyone
with the same hex convention can recover the represented byte values.

**2.9.** Take each old string and make two new strings, one by adding `0`
and one by adding `1` at the front. Every new string begins with exactly
one of those bits and has exactly one old string as its remainder. The
construction therefore misses none and counts none twice.

### Chapter 3 answers

**3.1.** In the order `00`, `01`, `10`, `11`, the XOR results are `0`, `1`,
`1`, `0`. In the last case both inputs are one, so the condition requiring exactly
one input to be one is false.

**3.2.** AND gives `1000`; OR gives `1110`; XOR gives `0110`. In every
column, use the rule for that particular operation. Do not carry between
columns.

**3.3.** The middle four positions selected by the ones in `00111100`
change. The result is `10011010`. XOR with `00111100` again gives
`10100110`.

**3.4.** `1101 ⊕ 0110 = 1011`. Without the retained mask, another input
and mask can yield the same result. For example, `1101` with mask `0000`
also produces `1101`. The output alone does not select one original.

**3.5.** Start with input `1` and mask `0`. The first AND gives `0`, and
the second gives `0` again, not the original `1`. An *always* statement
includes this permitted case. One failure is enough to contradict it.

**3.6.** No: the proposition requires equal-length strings. A designer
could define a padding or alignment rule for unequal lengths, but that
would be additional semantics. The present specification does not choose
one, and the present proof must not be silently applied to an unspecified
extension.

**3.7.** For triples `000`, `001`, `010`, `011`, `100`, `101`, `110`,
`111`, both groupings give, in order, `0`, `1`, `1`, `0`, `1`, `0`, `0`,
`1`. The table covers every triple of individual bits. To extend it to
equal-length strings, apply the bit result separately at every position;
that extra argument identifies the larger domain.

**3.8.** XOR with a publicly known all-zero mask leaves the input visible.
The narrower property is recovery under a retained mask: applying the same
mask twice returns the original. The failed secrecy inference does not
contradict cancellation.

**3.9.** For each of 256 possible input bytes, there are 256 possible
masks. Sixteen-bit strings have 65,536 values each, so pairs have
`65,536 × 65,536 = 4,294,967,296` possibilities. A test over byte pairs
has not examined those additional inputs.

**3.10.** The first statement asserts at least one success; the second
asserts success for all permitted inputs; the third asserts at least one
failure. The second and third contradict each other when they concern
the same operation, domain, and meaning of success. The first and third
can both be true.

## Source notes and epigraph record

The quoted words below are the chapter epigraphs only. The exercises,
examples, and explanatory prose are newly drafted for this book. The
quotations frame questions; they do not replace the arguments in the text.

**[S1] Claude E. Shannon.** “Communication Theory of Secrecy Systems.”
*Bell System Technical Journal* 28(4), 1949, pp. 656–715. The epigraph is
a seven-word excerpt from §2, p. 662. The surrounding sentence introduces
knowledge of the system as an assumption. Wording and location were checked
against the rendered page of the retypeset copy hosted by the University of
Wisconsin–Madison. This is a retypeset copy, not a scan of the original
printing. No translation is involved. Checked 2026-10-05.

Source: <https://pages.cs.wisc.edu/~rist/642-spring-2014/shannon-secrecy.pdf>

**[S2] Claude E. Shannon.** “A Mathematical Theory of Communication.”
*Bell System Technical Journal* 27, 1948, pp. 379–423 and 623–656. The
17-word epigraph is from the introduction, p. 1 of the corrected reprint
hosted by Harvard Mathematics. Wording and location were checked against
the rendered page. The quotation concerns a message selected from possible
messages; it does not assert that every message is equally probable.
No translation is involved. Checked 2026-10-05.

Source: <https://people.math.harvard.edu/~ctm/home/text/others/shannon/entropy/entropy.pdf>

**[S3] Bruce Schneier.** “Schneier's Law,” 15 April 2011, on the author's
website. The 19-word epigraph is the initial block quotation, which Schneier
identifies as his 1998 formulation. Wording was checked on the author's
page. The attribution identifies the page actually consulted rather than
claiming inspection of a 1998 original. No translation is involved.
Checked 2026-10-05.

Source: <https://www.schneier.com/blog/archives/2011/04/schneiers_law.html>

Epigraph sourcing is not a determination of publication permissions.
Any publication-rights decisions remain separate from verification of
wording and attribution.

## Draft status and provenance

This is the first newly written opening of the three-part *Orange Book*,
not a completed novice curriculum or a second companion book. The integrated
navigation and mapping to all seventeen existing chapters are in
[the book index](README.md). Existing source chapters and their status
qualifications have not been rewritten or removed in this increment.

The new prose, examples, exercises, and editorial integration were drafted
with ChatGPT (GPT-6 Astra Pro) at the owner's direction on 2026-10-05.
Authorship remains attributed to Chase Bryan; owner review is pending.
No independent review, proof-checker acceptance, compiler verification,
or release qualification is claimed. The accompanying reference tests
check the elementary worked examples in Python, not an Orange executable.
