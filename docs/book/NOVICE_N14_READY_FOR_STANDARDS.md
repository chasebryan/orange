# The Orange Book

By Chase Bryan

## Part 1, The Novice

N14: Ready for Standards. Draft 2026-10-05.

Continue from
[Modules and Provenance](NOVICE_N13_MODULES_AND_PROVENANCE.md#n13-modules-and-provenance).
The program below is a cut of that lesson's pad seam. It keeps the
byte XOR, the short-key block, and the qualified call. It does not
keep the hash. The reading habit is
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program).
The intermediate names are
[Name the Intermediate Step](NOVICE_N7_NAME_THE_INTERMEDIATE_STEP.md#n7-name-the-intermediate-step).
The separation of a universal claim from one Match is
[Say What You Mean](NOVICE_LOGIC.md#n9-say-what-you-mean)
and
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study).

This lesson is **N14**. The locked label is N14. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers. The
manuscript chapter titled *Evidence That Survives the Build* keeps
that title and that job. When the text says “Chapter 3” or “§2”, the
first is a novice chapter and the second is a section of a cited
standard.

N14 is a gate. It does not add a language form. It does not begin
J2, J3, or J4, which are Block A of the Journeyman. It does not
begin J5. J5 is the study that derives the SHA-256 compression
function and its message schedule from FIPS 180-4 §6.2.2. N13
transcribed those operations and did not derive them. This lesson
neither transcribes them nor derives them. The section number is a
pointer. The functions are not on this page.

## N14: Ready for Standards

> “Perhaps the largest group of readers will consist of people who want to read a full and unambiguous description of Rijndael.”
>
> — Joan Daemen and Vincent Rijmen, *The Design of Rijndael* (2002), Preface. [S12]

The sentence names a group of readers and the object they want. It
does not say they have read the description. Wanting a specification
is not reading one, and reading one page of a specification is not
carrying the assumptions the next page uses. This lesson is the gate
in front of that reading. You will do four acts on one small program,
at the same time. If you cannot do them on a second program this
chapter did not walk, you are not ready for J2–J4.

Five outcomes finish the lesson. None of them is a certificate.

1. You can walk one Orange program and name each construct the file
   actually contains.
2. You can derive one intermediate the program computes, and put the
   arithmetic beside the Orange name.
3. You can say what one `test` matches, and you can say which inputs
   and which claims it does not mention. A Match is not called
   verified.
4. You can list the assumptions a Block A reader and a J5 reader
   still have to carry: the edition pin, the byte order, the padding,
   the module seam, and the non-claims.
5. You can apply the same four acts to a listing this chapter did not
   walk. Missing any one of them is a failure. The failure means you
   are not ready for J2–J4.

### N14.1 Why a gate exists

Each of N7 through N13 taught one habit and then stopped. Journeyman
work uses the habits together. A reader who can rotate a word and
cannot say what a test failed to mention will misread the first
vector. A reader who can name a module and cannot compute the byte
the call returns will trust the qualifier in place of the value.

The habits, named so they can be demanded at once:

N7 names an intermediate with `let` and refuses to leave it inside an
unbroken expression. N8 reads a diagnostic, or reads silence, and
does not treat either one as a standard. N9 gives a claim a domain
and a quantifier, and it separates that claim from one example. N10
separates a count from a distribution and a finite check from a
security claim. N11 separates a hash from an authentication check.
N12 derives one construction from a stated standard, transcribes it,
and says which claim the transcription does not reach. N13 puts the
hash in one module and the keyed layer in another, pins an expected
byte to a document, and reads a passing test as a Match on the
inputs the test wrote.

Block A needs those acts on whatever listing it opens. J5 needs them
on a standard this gate does not open. Doing them one lesson at a
time, each on the example that lesson chose, is not the same skill
as doing them together on a listing you did not write. The desk in
§N14.6 is that demand. Copying §N14.2 through §N14.5 onto the listing
they already walked is not meeting it.

### N14.2 Interpret a program

Five assumptions bound the walk. A later sentence that needs a
further fact names it there.

**Assumption N14.1 — The edition token is the pin.** Both files begin
with the declaration `edition 2026;`. That token is the edition this
compiler requires. It is not a date inside FIPS 180-4, and it is not
the August 2015 pin N13 recorded for that publication. [T8]

**Assumption N14.2 — These files do not choose a byte order.** Every
value they compute is a `Word[8]` or an array of `Word[8]`. They do
not shift a `Word[32]`, and they do not narrow one with `as`. Silence
on byte order is not a choice of byte order. N12 serializes a ChaCha
word with the low byte first. N13 writes a SHA-256 word with the high
byte first. Those are two orders. This page does not repeat either
conversion.

**Assumption N14.3 — The zero fill is one padding, not the other.**
`case1_key` starts from 64 zero bytes and writes `0x0b` at indices 0
through 19. That is RFC 2104 §2, step (1), for a 20-byte key of that
byte, which is Proposition N13.4 on this key. It is not the padding
in FIPS 180-4 §5.1.1. That padding appends the byte `0x80`, zeros,
and a length in bits. This listing does not write `0x80`.

**Assumption N14.4 — The qualifier is the seam.** `use pad;` before
any function of `pad_seam`, and a call written `pad::f(...)`, are
Assumption N13.3 applied to these two files. An unqualified name is
looked up in the calling module only. `pad` contains no `use`. The
split is two declaration scopes. It is not a security boundary. [T7]

**Assumption N14.5 — A Match is not a verification.** A silent check
means the source was well-formed under the checks this compiler runs.
An evaluation means the parameterless specs that ran produced the
printed values. A passing test means the `Bool` in that test was
true on the inputs the test wrote. None of those is a proof for
every key, none is HMAC, none is SHA-256, and none is called
verified. [C2]

The forms the listings use, and no others, are the ones N7, N8, N12,
and N13 already ran: `edition`, `module`, `spec`, `let`, `Word[8]`,
hexadecimal byte literals, `^`, a bounded `for`, a fill `[0; 64]`,
an array update `with`, an index, `use`, a qualified call, `Bool`,
`&&`, `==`, and `test`. Listings N14.1 and N14.2 contain no `if`,
no tuple, no shift, no rotation, and no `as`. Listing N14.3, which
the desk exam does not walk in advance, uses `+` and `<<<` on
`Word[32]`. A reading of the pad files that names one of the absent
forms has added a construct those files do not have.

**Listing N14.1 — `pad.or`**

```orange
edition 2026;
module pad {
  spec inner_pad() -> Word[8] { 0x36 }
  spec outer_pad() -> Word[8] { 0x5c }
  spec xor_byte(k: Word[8], p: Word[8]) -> Word[8] { k ^ p }
  spec keyed(key: Word[8]^64, p: Word[8]) -> Word[8]^64 {
    for i in 0..64 with b: Word[8]^64 = key { b with [i] = key[i] ^ p }
  }
  spec case1_key() -> Word[8]^64 {
    for i in 0..20 with b: Word[8]^64 = [0; 64] { b with [i] = 0x0b }
  }
}
```

Read the constructs in source order.

`edition 2026;` is Assumption N14.1. One source file holds one
module. The name of the module is `pad`, and the file that holds it
is `pad.or`.

`inner_pad` and `outer_pad` take no arguments and return one byte
each. The bodies are the literals `0x36` and `0x5c`. Those are the
pad bytes RFC 2104 §2 fixes, the same two bytes N13 used. The names
are Orange names. They are not the RFC's names ipad and opad, which
are the 64-byte strings, not the repeated byte.

`xor_byte` takes two bytes and returns their bitwise exclusive or.
The body is the operator `^` and nothing else. Assumption N13.4 is
that operator on `Word[8]`: one bit at a time, high bit on the left.

`keyed` takes a 64-byte block and one pad byte. The `for` walks the
integer literals 0 through 63. The accumulator starts as the key.
At index `i` the update `b with [i] = key[i] ^ p` replaces that one
byte with the XOR and leaves the other bytes as they were. The
result is a new array. The parameter `key` is not assigned.

`case1_key` takes no arguments. The accumulator starts as 64 copies
of the byte `0`. The loop `0..20` writes `0x0b` at indices 0 through
19 and does not write indices 20 through 63. Those forty-four bytes
stay `0`. That is Assumption N14.3 on this one key. The function
does not take a length. A key of another length is a different
function, and this file does not contain it.

The module has no `test`. A check of the file does not compare a
byte with a standard.

```sh
./compiler/target/debug/orangec check pad.or
./compiler/target/debug/orangec eval --spec case1_key pad.or
```

Check is silent. The status is 0. Silence means the source was
well-formed. It does not mean the twenty bytes are the RFC's key.

`eval --spec case1_key` runs that one parameterless spec. It does
not run `inner_pad` or `outer_pad`, and it does not run a test.
The option is the one N8 and N13 used.

**Expected evaluation output:**

```text
pad::case1_key: Word[8]^64 = [0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
```

Count the `0x0b` entries. There are twenty. Every later entry is
`0x00`. That array is the only key the next file uses.

**Listing N14.2 — `pad_seam.or`**

The file sits in the same directory as `pad.or`. The compiler reads
`pad.or` because of the `use`, not because the two texts were pasted
into one module.

```orange
edition 2026;
module pad_seam {
  use pad;

  spec inner0() -> Word[8] {
    let key: Word[8]^64 = pad::case1_key();
    let byte: Word[8] = key[0];
    pad::xor_byte(byte, pad::inner_pad())
  }

  test "RFC 2104 pads differ by 0x6a on the RFC 4231 4.2 key block" {
    let key: Word[8]^64 = pad::case1_key();
    let inner: Word[8]^64 = pad::keyed(key, pad::inner_pad());
    let outer: Word[8]^64 = pad::keyed(key, pad::outer_pad());
    for i in 0..64 with ok: Bool = true { ok && ((inner[i] ^ outer[i]) == 0x6a) }
  }
}
```

`edition 2026;` is the same pin as in Listing N14.1. The module name
is `pad_seam`. The declaration `use pad;` is the only `use`, and it
stands before either function. Assumption N14.4 says that placement
is required.

`inner0` has two bindings and one result. `key` denotes the block
`pad::case1_key` returns. The qualifier `pad::` is the seam. An
unqualified `case1_key` would look for a function of `pad_seam`, and
`pad_seam` does not declare one. `byte` denotes index 0 of that
block. The index is the literal `0`, which is inside 0 through 63.
The result is `pad::xor_byte` applied to `byte` and
`pad::inner_pad()`. The result's name, for the rest of this lesson,
is `inner0`. The body does not call `keyed`, and it does not call
`outer_pad`.

The `test` declares one `Bool`. It binds the same key, then the
64-byte block `keyed` returns for the inner pad byte, then the
64-byte block `keyed` returns for the outer pad byte. The `for`
walks indices 0 through 63. The accumulator starts as `true`. At
each index the conjunction `&&` keeps it true only when that index's
two bytes XOR to `0x6a`. One failing index makes the `Bool` false.
The title names RFC 2104 and RFC 4231 §4.2. The title is not the
`Bool`. [T6]

```sh
./compiler/target/debug/orangec check pad_seam.or
./compiler/target/debug/orangec eval --spec inner0 pad_seam.or
./compiler/target/debug/orangec test pad_seam.or
```

Both files are in the directory where you run the command. Check is
silent. Eval prints the one parameterless spec of the root. It does
not run the test. Test runs the root's test. It does not run a test
of `pad`, because `pad` is not the root and `pad` has no test.

**Expected evaluation output:**

```text
pad_seam::inner0: Word[8] = 0x3d
```

**Test report:**

```text
test "RFC 2104 pads differ by 0x6a on the RFC 4231 4.2 key block" ... ok
1 test: 1 passed, 0 failed
```

The evaluation and the report are two sentences. The first says
`inner0` printed `0x3d` on this run. The second says the test's
`Bool` was true on this run. The report does not contain `0x3d`.
The evaluation does not walk the sixty-four indices. §N14.3 is the
first sentence, derived by hand. §N14.4 is the second, and what it
does not say.

### N14.3 Derive one op

The Orange name is `inner0`. The value to derive is the result of
that spec, not the test's `Bool`.

`case1_key` writes `0x0b` at index 0, by the loop `0..20` in Listing
N14.1. So the binding `byte` denotes `0x0b`. `inner_pad` denotes
`0x36`. `xor_byte` is `^` on those two bytes.

Write both bytes with the high bit on the left:

```text
byte      0x0b = 00001011
inner_pad 0x36 = 00110110
XOR              00111101 = 0x3d
```

Check the nibbles. `0000 XOR 0011 = 0011`, which is `0x3`.
`1011 XOR 0110 = 1101`, which is `0xd`. The byte is `0x3d`.

As integers in the range 0 through 255, the same bytes are 11 and
54. Exclusive or is not addition. 11 + 54 = 65, and 65 is `0x41`,
which is not the byte above. The bit table is the operation. The
sum is a different operation, and `inner0` does not perform it.

**Proposition N14.1.** Under Assumption N13.4 and the reading of
Listings N14.1 and N14.2 just given, `inner0()` denotes `0x3d`.

*Proof.* Index 0 of `case1_key` is `0x0b`, because that index is
among 0 through 19 and the loop writes `0x0b` there. `inner_pad`
is the literal `0x36`. The result of `inner0` is `xor_byte` of
those two bytes, which is their bitwise exclusive or. The eight
bit positions computed above yield `00111101`, which is `0x3d`.
The argument used index 0 only. □

The proposition does not mention index 1, and it does not mention
`outer_pad`. A reader who replaces `0x3d` with `0x6a` has computed
the test's expected XOR of the two pad results, which is a
different expression. That expression is §N14.4. It is not
`inner0`.

### N14.4 Explain a test's domain

The test in Listing N14.2 builds one block, `case1_key`. Its first
twenty bytes are `0x0b` and its other forty-four bytes are `0x00`.
It computes `keyed` of that block with `0x36`, and `keyed` of that
block with `0x5c`, and it asks whether those two results XOR to
`0x6a` at every index from 0 through 63.

That is the whole domain. Two bytes of key material occur in it,
`0x0b` and `0x00`, each repeated. The indices are sixty-four
positions of this one block. The report line ends in `ok`. That
means the `Bool` was true on those positions on this run.

The test does not cover a third byte. Proposition N13.2 states the
same XOR identity for every byte from `0x00` through `0xff`. The
test feeds two of those 256 bytes. A passing report is a Match on
that pair, at the indices where each of them sits.
Do not call that Match verified.

The test does not cover a second key. RFC 4231 §4.3 is a different
key. RFC 4231 §4.7 is a key of 131 bytes. Listing N14.2 has neither.
A passing report about §4.2 does not mention them.

The test does not cover HMAC. It never calls a hash.
The test does not cover SHA-256. The compression function and the message schedule
are not in either file. Matching `0x6a` at these indices does not
produce a tag, and it does not check a tag.

The test does not cover `inner0`. Its `Bool` never compares a byte
with `0x3d`. You can delete `inner0` from the file and the test
still has the same `Bool`. You can change `inner0` to return
`0x00` and the test still passes, because the test does not read
that spec. The evaluation of `inner0` is the check of Proposition
N14.1 on this run. The test report is not that check.

The test does not say why the equation holds. Proposition N13.2
proved it for an arbitrary byte, one bit position at a time, from
the table for `^`. The test does not contain that table. The
proof did not become a proof by the test passing, and the test did
not become a proof by sitting in the same file as a call to
`keyed`. Assumption N9.4 would require a chain `Q(n) ⇒ Q(n + 1)`
from `Q(n)` alone. This test has no such step. It is a
conjunction across sixty-four indices of one block.

`orangec check pad_seam.or` accepts the test and does not run it.
Silence after check is consistent with a `Bool` that would have
been false. The report is the command that runs the `Bool`.

### N14.5 Locate assumptions

A reader who opens Block A, and a reader who later opens J5, carries
five assumptions that this gate does not discharge. The list is
the one to write from the files, not from memory of a slogan.

1. **Edition.** `edition 2026;` pins the language edition. The
   standard's own edition is a second pin. For the hash N13 called,
   that pin is FIPS PUB 180-4, August 2015, DOI
   `10.6028/NIST.FIPS.180-4`, which N13 recorded as [D1]. Listing
   N14.2 does not contain that date. Carrying only the Orange
   edition into J5 drops the publication you are about to read.

2. **Endianness.** Assumption N14.2: these files never assemble a
   word from bytes. ChaCha in N12 is little-endian on the wire.
   SHA-256 in N13 is big-endian on the wire. J5 has to use the
   order the standard states, and it has to notice when a listing
   is silent. Silence is not little-endian and it is not
   big-endian.

3. **Padding.** Assumption N14.3: the zeros in `case1_key` are the
   short-key pad of RFC 2104 §2. The byte `0x80` and the bit-length
   field are FIPS 180-4 §5.1.1. They are a different padding. A
   reader who treats the zero fill as the hash padding will build
   the wrong final block and can still pass a test that never
   looks at that block. This lesson's test never looks at it.

4. **Module seam.** Assumption N14.4: `pad::` is the only way
   `pad_seam` names a function of `pad`. The root's test does not
   run `pad`'s tests. `pad` has none. A later stack that adds a
   hash module beside these two files does not make `keyed` into
   that hash. The qualifier you can point at is the seam. A
   comment that says “hash” is not.

5. **Non-claims.** Assumption N14.5, in one place: well-formed is
   not matched; matched on these inputs is not matched on every
   input; matched is not verified; the pad identity is not a tag;
   a tag is not an authentication decision, which N11 kept as a
   separate definition; nothing on this page is a security claim
   or a certificate that you may begin J2.

J5 may derive FIPS 180-4 §6.2.2. The derivation has a schedule, a
round, and constants. None of those words is a license to paste
them into this lesson. They are not here. When you need them, you
are no longer inside N14.

### N14.6 Desk exam

The verdict is one sentence, and you can fail it.

If you cannot, on a listing this chapter did not walk, name each
construct the file contains, derive one intermediate beside its
Orange name, state the finite domain of one test or state that the
file has no test and what that absence fails to establish, and
fill the five headings in §N14.5, then you are not ready for
J2–J4. Reaching this page does not change the sentence. There is
no partial credit that becomes readiness.

Listing N14.3 is the listing the chapter did not walk. It is the
sample line from N12, copied here so the gate can demand it without
sending you out of the file. The walk in §N14.2 was the pad seam.
A correct reading of `inner0` does not answer these exercises.

**Listing N14.3 — `sample_line.or`**

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

The commands that belong to this file are check and eval. The file
has no `test`. Running the test command is part of seeing what
that absence prints. It is not a Match.

```sh
./compiler/target/debug/orangec check sample_line.or
./compiler/target/debug/orangec eval sample_line.or
./compiler/target/debug/orangec test sample_line.or
```

Check is silent.

**Expected evaluation output:**

```text
sample_line::rolled: Word[32] = 0xcc5fed3c
```

**Test report:**

```text
0 tests: 0 passed, 0 failed
```

`0xcc5fed3c` is the result of `rolled`, which is `mixed` rotated
left by 7. It is not the binding `sum`. The test report counts
zero tests. Zero failed is not a comparison against RFC 8439. A
file can print a value and still have established no Match.

Do the eight exercises before you read the worked answers. A wrong
line is a place to recompute. If the recomputation still misses,
the verdict in the first paragraph of this section stands.

**Exercise N14.1 — Interpret Listing N14.3.** Name each construct
the file contains, in source order. Then name three constructs
that appear in Listing N14.2 and do not appear in Listing N14.3.

**Exercise N14.2 — Derive `sum`.** Compute the binding `sum` in
Listing N14.3 by bytes, low byte first. Put the arithmetic beside
the name `sum`. Say whether that result is `rolled()`.

**Exercise N14.3 — What the absent test does not cover.** The
report `0 tests: 0 passed, 0 failed` was printed for Listing N14.3.
State what that report establishes, and state two claims it does
not establish. Separately, for the test in Listing N14.2, name
the finite domain and name three claims that test does not cover.
One of the three must be a claim about SHA-256.

**Exercise N14.4 — Fill the assumption map.** From Listings N14.1
and N14.2, and from what they refuse to contain, write one
sentence under each heading: edition, endianness, padding, module
seam, non-claims. A sentence that moves the FIPS message padding
into `case1_key` is a miss. Compare with the worked answer after
you have written the five sentences.

**Exercise N14.5 — The other byte the test feeds.** Index 20 of
`case1_key` is `0x00`. Compute `0x00 XOR 0x36` and `0x00 XOR 0x5c`
by bits, then XOR those two results. Say whether the test's `Bool`
mentions `0x36` as an expected byte, or only mentions `0x6a`.

**Exercise N14.6 — Where the compression function is.** Which of
these appear in Listings N14.1 through N14.3: a message schedule,
a compression round, a table of round constants? Say where a J5
reader goes for FIPS 180-4 §6.2.2, and say what this page
provides instead.

**Exercise N14.7 — The fail path.** Suppose your bytes for `sum`
disagree with the worked answer, and a second attempt from the
same rules still disagrees. Write the readiness sentence. Do not
replace it with a plan to begin J2 and repair the sum later.

**Exercise N14.8 — Two orders, neither written here.** Which of
N12 and N13 writes the high byte of a 32-bit word first? Which
heading in §N14.5 does that fact belong under? Listing N14.3 adds
two `Word[32]` values and rotates one. Does that listing choose
the ChaCha byte order, the SHA-256 byte order, or neither?

```text
n14-ledger
byte0 = 11
inner-pad = 54
inner0 = 61
outer-pad = 92
pad-xor = 106
zero-inner = 54
zero-outer = 92
key-ones = 20
key-zeros = 44
sample-sum = 2023406814
```

`0x0b = 11`, `0x36 = 54`, `0x3d = 61`, `0x5c = 92`, and
`0x6a = 106`. `zero-inner` is `0x00 XOR 0x36`. `sample-sum` is the
integer `0x77777777 + 0x01234567`, which is the binding `sum` when
Assumption C1 subtracts nothing. The reference check recomputes
each line from those definitions.

## Worked answers

**N14.1.** In source order, Listing N14.3 contains: the edition
declaration `edition 2026;`; one module named `sample_line`; one
spec `rolled` with an empty parameter list and result type
`Word[32]`; three bindings `c`, `d`, and `b`, each a `Word[32]`
hexadecimal literal; the binding `sum`, defined as `c + d`; the
binding `mixed`, defined as `b ^ sum`; and the result
`mixed <<< 7`. It contains no second spec, no `test`, no `use`,
no qualified name, no array, and no `for`.

Three constructs in Listing N14.2 and absent here are `use`, a
qualified call such as `pad::inner_pad`, and `test`. A list that
includes `for` or `^` as absent from Listing N14.3 is wrong about
`^`: Listing N14.3 uses `^` in `mixed`. It does not use `for`.

**N14.2.** Low byte first, under Assumption C1. `0x77 + 0x67 = 0xde`,
with no carry. `0x77 + 0x45 = 0xbc`, with no carry.
`0x77 + 0x23 = 0x9a`, with no carry. `0x77 + 0x01 = 0x78`, with
no carry. So `sum = 0x789abcde`. The integer sum is
2004318071 + 19088743 = 2023406814, which equals `0x789abcde` and
is less than 2³², so C1 subtracts nothing. `rolled()` is
`(b XOR sum) <<< 7`, which is `0xcc5fed3c`. That is not `sum`.

**N14.3.** The report establishes that the root declared no test,
so none failed. It does not establish that `sum` equals
`0x789abcde`. It does not establish that `rolled()` equals the
sample in RFC 8439 §2.1. Those claims need a comparison the file
does not contain. The evaluation of `rolled` is a different
command, and it prints the rotation, not a test report.

The test in Listing N14.2 matches one predicate: on the single
block `case1_key`, at each index from 0 through 63,
`(inner[i] XOR outer[i]) == 0x6a`. It does not cover HMAC. It
does not cover a key other than that block. It does not cover
SHA-256: no listing in this lesson contains the compression
function or the message schedule, and the `Bool` does not compare
a digest.

**N14.4.** Edition: both files pin `edition 2026;`, and neither
file pins FIPS 180-4 to August 2015. Endianness: the pad files
use only bytes, so they choose no word order; N12 and N13 chose
opposite orders, and J5 must carry the order its standard states.
Padding: `case1_key` zero-fills a 64-byte block and writes twenty
bytes of `0x0b`; that is the short-key step of RFC 2104 §2, not
the `0x80` and length field of FIPS 180-4 §5.1.1. Module seam:
`pad_seam` calls `pad` only by `pad::` after `use pad;`, and `pad`
uses nothing. Non-claims: the passing pad test is a Match on
that one block; it is not verified, it is not HMAC, and it is not
permission to begin J2.

**N14.5.**

```text
0x00 = 00000000
0x36 = 00110110
XOR  = 00110110 = 0x36

0x00 = 00000000
0x5c = 01011100
XOR  = 01011100 = 0x5c

0x36 = 00110110
0x5c = 01011100
XOR  = 01101010 = 0x6a
```

Index 20 therefore agrees with the test's equation. The `Bool`
compares the XOR of the two keyed bytes with `0x6a`. It does not
require the keyed byte itself to be `0x36`. A different bug that
still XORed to `0x6a` at every index would pass.

**N14.6.** None of the three appears. Listings N14.1 through
N14.3 have no schedule, no round, and no constant table. A J5
reader goes forward to a derivation of FIPS 180-4 §6.2.2. N13
transcribed that section and did not derive it. This page
provides the pointer and withholds the functions. Copying a
round in from N13 in order to feel ready is leaving the gate.

**N14.7.** I am not ready for J2–J4. The disagreement means the
four acts are not yet available on a listing the chapter did not
walk. Beginning Block A does not repair the binding.

**N14.8.** N13 writes the high byte first. N12 writes the low
byte first. The fact belongs under endianness. Listing N14.3
adds and rotates `Word[32]` values. It does not split a word
into bytes, so it chooses neither order. The printed word
`0xcc5fed3c` is the value of `rolled`, not a serialization.

## Sources and epigraph record

The quotation is the borrowed sentence. The derivations are the
lesson's. The pad bytes and the key block are the copies N13
already pinned; this lesson uses them and does not open a second
vector.

**[S12] Joan Daemen and Vincent Rijmen.** *The Design of
Rijndael: AES — The Advanced Encryption Standard.* Springer-Verlag,
2002. The epigraph is one sentence of the Preface, in the PDF
dated November 26, 2001, hosted by the first author. The sentence
begins “Perhaps the largest group,” in the paragraph headed
“Structure of this book,” immediately after the sentence that says
the authors had two kinds of readers in mind. Wording was checked
on 2026-10-05 against that PDF. The
sentence contains no hyphenation and no inner quotation. No
translation is involved. The preface goes on to describe Rijndael.
This lesson does not. The sentence is not an endorsement of
Orange, and it is not a claim that a reader who wants a
specification has read one.
This record's tag is [S12].

Source: <https://cs.ru.nl/~joan/papers/JDA_VRI_Rijndael_2002.pdf>

**[T8] Orange edition.** The declaration `edition 2026;` is the
edition token required at the start of a source.
`docs/LANGUAGE_2026.md` specifies that token, and OEP-0002 records
the exact decimal `2026` as mandatory. OEP-0002's grammar is the
early empty-body grammar. It does not by itself authorize `use`,
`for`, or `test`. Those forms are the later surfaces N7, N12, and
N13 already ran on this compiler. The listings use the token.
They do not treat OEP-0002's empty body as their grammar.
This record's tag is [T8].

The module form is the one N13 recorded as [T7]. The test form is
the one N12 recorded as [T6]. `orangec check` checks a test and
does not run it. `orangec eval --spec` runs the named
parameterless spec of the root and does not run tests.
`orangec test` runs the root module's tests after the checks. A
root with no test prints `0 tests: 0 passed, 0 failed`.

**[C2] Gate surface.** The listings use `Word[8]`, `^`, a bounded
`for`, `use`, a qualified call, and `test`, as specified for the
slices N12 and N13 already checked, plus the edition token in
[T8]. Listing N14.3 uses `Word[32]`, `+`, `^`, and `<<<` under
N12's assumptions C1 through C3. A passing test is a Match on the
inputs it writes. It is not a quantifier, and it is not a
security claim.
This record's tag is [C2].

The FIPS publication, the NIST “abc” sample, and RFC 4231 remain
N13's [D1], [N1], and [V1]. This lesson cites the publication's
section numbers and does not copy its schedule or its round. RFC
2104 remains N13's [S11]. The twenty bytes `0x0b` are RFC 4231
§4.2's key, used here only as the block `case1_key` builds.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

N14 adds eight exercises with worked answers. The integer ledger
is recomputed by `tools/test_book_foundations.py`. The Orange
listings are the three fenced programs in this file.
`compiler/crates/orangec/tests/book_novice.rs` runs them. Those
checks do not establish a cryptographic security claim, they do
not derive FIPS 180-4 §6.2.2, and they do not accept a proposal.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction. Owner review is pending. No deployment
recommendation and no certificate of competence is made. A reader
who cannot do the four acts on Listing N14.3 is not ready for
J2–J4.
