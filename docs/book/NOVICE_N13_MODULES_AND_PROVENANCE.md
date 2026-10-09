# The Orange Book

By Chase Bryan

## Part 1, The Novice

N13: Modules and Provenance. Draft 2026-10-05.

Continue from
[The First Complete Study](NOVICE_N12_THE_FIRST_COMPLETE_STUDY.md#n12-the-first-complete-study).
This lesson does not use the ChaCha20 block. It uses the one-module
programs of N7 and N12, the diagnostic habit of
[Read and Repair a Program](NOVICE_N8_READ_AND_REPAIR.md#n8-read-and-repair-a-program),
the separation of a hash from an authentication check in
[Protect More Than Appearance](NOVICE_PROTECT.md#n11-protect-more-than-appearance),
and XOR from
[Chapter 3](NOVICE_OPENING.md#chapter-3-a-rule-you-can-undo).

This lesson is **N13**. The locked label is N13. It is not a manuscript
chapter numeral. The original manuscript keeps its own numbers.
Manuscript Chapter 13, *Interoperability and External Validation*, keeps
that number and that job. When the text says “Chapter 3” or “§2”, the
first is a novice chapter and the second is a section of a cited
standard.

N13 stops at the seam between a hash and the keyed layer that calls it.
It does not derive the SHA-256 round. That derivation is later work.
It does not study HMAC past the pads, the two calls, and two short
vectors. The security argument for the construction is later work.
Neither omission is a sketch of the missing study.

## N13: Modules and Provenance

> “The definition of HMAC requires a cryptographic hash function, which we denote by H, and a secret key K.”
>
> — H. Krawczyk, M. Bellare, and R. Canetti, RFC 2104 (February 1997), §2. [S11]

The sentence names two requirements, and the comma keeps them apart.
H is not K. A definition that needs both is not the definition of
either one. The lesson takes that comma as a file boundary. You will
put H in one module and the keyed layer in another, call across the
boundary by name, and pin every expected byte to the document that
prints it. You will also say, of each passing test, which inputs it
matched and which inputs it did not mention.

Five outcomes finish the lesson. None of them is a certificate.

1. You can say why a module that computes a hash is not a module that
   computes HMAC, and you can point at the seam in the formula.
2. You can put the hash in one file and the keyed layer in another,
   and call the hash by its module name. An unqualified name stays in
   the file that wrote it.
3. You can pin an expected digest to a document, an edition, and a
   vector, and you can say what that comparison does not cover.
4. You can prove one claim about the pads from the definitions, and
   you can read an Orange test of a finite piece of that claim as a
   Match on the inputs the test wrote.
5. You can read a seam diagnostic, repair only the text it identifies,
   and say what the repaired call still does not establish.

### N13.1 Why one file is not a stack

Definition N11.4 made a hash a function from a larger finite set to a
smaller one. The definition has no key. Definition N11.5 made an
authentication check a function that may take a key and that returns
true or false. HMAC, as RFC 2104 §2 writes it, produces a tag from a
key and a text by calling a hash twice. The tag is not the check. This
lesson transcribes the production of the tag on a stated domain. It
does not transcribe the check, and it does not prove that a
substituted tag is rejected.

Section 2 of the RFC writes the production as one formula. The inner
call is applied first:

```text
H(K XOR opad, H(K XOR ipad, text))
```

Read the parentheses. The inner H receives a string built from the key
and the text. The outer H receives a string built from the key and
whatever the inner H returned. Both calls are H. Neither call is the
formula. The formula is the composition. The seam is the place where
the composition names H instead of containing H.

The same section fixes two strings. For a hash whose block is B bytes,
ipad is the byte `0x36` repeated B times, and opad is the byte `0x5c`
repeated B times. The RFC prints the second byte as `0x5C` in §2 and
as `0x5c` in the appendix. Those are one byte. The letters i and o are
the RFC's names for inner and outer. They are not Orange names.

A single Orange file can hold the rounds of H and the two calls. The
bytes can be the bytes the standard prints. The file is still one
module. Every `spec` in it is a function of that module, and a call
written `compress(...)` names a function of that same module. Nothing
on the page tells a later reader which functions are H and which
functions are the composition, except the comments the author
remembered to write. Comments are not the language's calls. When the
composition forgets to call H, the checker has no module name to
complain about, because no module name was declared.

Two files make the seam a name the checker reads. The file `sha256.or`
declares `module sha256` and uses no other module. The file `hmac.or`
declares `module hmac`, and the declaration at its head is
`use sha256;`. A call that wants H is written `sha256::compress(...)`
or `sha256::hash(...)`. The qualifier is the seam. Deleting it is a
different program. §N13.5 is the diagnostic.

Two files are two declaration scopes. They are not two security
boundaries. A reader who can see the call can still use the tag as if
it were a cipher, or call the hash as if it took a key. The split does
not make SHA-256 collision-resistant, and it does not make the tag
unforgeable. N11 refused those phrases until an adversary and a
success condition exist. This lesson refuses them for the same reason.
A passing test is not that missing definition.

### N13.2 The module boundary

Five assumptions bound the rest of the lesson. A later sentence that
needs a further fact names it there.

**Assumption N13.1 — What the hash module owns.** The module `sha256`
is the hash. On a message of at most 55 bytes, held in a 64-byte array
with a length, `hash` denotes SHA-256 as FIPS 180-4 §6.2 specifies it.
`initial`, `compress`, `last_block`, and `digest` are the operations
the keyed module calls when the string is longer than 55 bytes. This
lesson does not derive the schedule, the round, or the constants.
Those names are a transcription of the sections cited in the listing.
The evidence this lesson offers for the transcription is the one
message in §N13.3.

**Assumption N13.2 — What the keyed module owns.** The module `hmac`
owns the pad byte, the placement of a key that is already at most 64
bytes into a 64-byte block, and the two nested calls. It does not own
the compression function. It does not own a key longer than 64 bytes.
Its `mac` is defined only when the text length `L` satisfies
`0 ≤ L ≤ 55`.

**Assumption N13.3 — A call across the seam names the module.** On
this compiler, `use m;` at the head of a module, before any function,
declares that this module may call typed `spec` functions of the
module `m`. The call `m::f(...)` denotes `f` of that module. An
unqualified call denotes a function of the calling module only. There
is no second form that brings `f` into scope without the qualifier.
The file `m.or` is read from the directory of the file that contains
the `use`. This is the behavior specified in `docs/MODULES_2026.md`,
proposed under OEP-0011, and implemented by the compiler this lesson
runs. Implementation is not acceptance of the proposal. [T7]

**Assumption N13.4 — XOR on a byte is XOR on its bits.** `^` on
`Word[8]` is the bitwise exclusive or of Chapter 3, one bit at a time,
with the high bit written on the left as Chapter 2 writes a byte. Two
bytes are equal exactly when all eight bits agree. The byte `0x00` is
the byte whose eight bits are 0.

**Assumption N13.5 — The copied bytes are the consulted bytes.** The
digest of “abc” is the one-block sample named in §N13.3. The HMAC
bytes are the HMAC-SHA-256 lines of RFC 4231 §4.2 and §4.3. A later
erratum would have to be read before it replaced either copy. None is
applied here.

The forms Assumption N13.3 allows, and that the listings use, are
these:

```text
use sha256;
sha256::hash(block, length)
```

`use` begins a declaration only before the first function. A qualified
name is always called. The listings call. They do not mention a
qualified name and then stop. A module's tests run only when that
module is the root. `orangec test hmac.or` reads `sha256.or` because
of the `use`, checks it, and does not run the tests written inside it.
`orangec test sha256.or` does not read `hmac.or`, because `sha256`
uses nothing. A passing HMAC test is not a second run of the “abc”
test. The “abc” test runs when `sha256.or` is the root.

**Listing N13.1 — `sha256.or`**

```orange
edition 2026;
module sha256 {
  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }
  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }
  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }
  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (~x & z) }
  spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] { (x & y) ^ (x & z) ^ (y & z) }

  // Section 4.2.2: the first 32 bits of the fractional parts of the cube
  // roots of the first 64 primes.
  spec round_constants() -> Word[32]^64 {
    [
      0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
      0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
      0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
      0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
      0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
      0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
      0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
      0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ]
  }

  // Section 5.3.3: the first 32 bits of the fractional parts of the square
  // roots of the first eight primes.
  spec initial() -> Word[32]^8 {
    [
      0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
      0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ]
  }

  // Section 6.2.2, step 1: the message schedule of one block, whose sixteen
  // big-endian words are W_0 through W_15.
  spec schedule(block: Word[8]^64) -> Word[32]^64 {
    let head: Word[32]^64 = for t in 0..16 with w: Word[32]^64 = [0; 64] {
      w with [t] = ((block[4 * t] as Word[32]) << 24) | ((block[4 * t + 1] as Word[32]) << 16)
        | ((block[4 * t + 2] as Word[32]) << 8) | (block[4 * t + 3] as Word[32])
    };
    for t in 16..64 with w: Word[32]^64 = head {
      w with [t] = small_sigma1(w[t - 2]) + w[t - 7] + small_sigma0(w[t - 15]) + w[t - 16]
    }
  }

  // Section 6.2.2, step 3: one round on the working variables a through h.
  spec round(v: Word[32]^8, k: Word[32], w: Word[32]) -> Word[32]^8 {
    let t1: Word[32] = v[7] + big_sigma1(v[4]) + ch(v[4], v[5], v[6]) + k + w;
    let t2: Word[32] = big_sigma0(v[0]) + maj(v[0], v[1], v[2]);
    [t1 + t2, v[0], v[1], v[2], v[3] + t1, v[4], v[5], v[6]]
  }

  // Section 6.2.2, steps 2 through 4: one 64-byte block absorbed into the
  // hash value `h`.
  spec compress(h: Word[32]^8, block: Word[8]^64) -> Word[32]^8 {
    let k: Word[32]^64 = round_constants();
    let w: Word[32]^64 = schedule(block);
    let v: Word[32]^8 = for t in 0..64 with s: Word[32]^8 = h { round(s, k[t], w[t]) };
    for i in 0..8 with out: Word[32]^8 = v { out with [i] = out[i] + h[i] }
  }

  // 256^k, for k from 0 through 7.
  spec weight(k: Int) -> Int {
    for j in 0..7 with w: Int = 1 { if j < k { w * 256 } else { w } }
  }

  // Section 5.1.1: the last block of a message that is `total` bytes long
  // and whose final `length` bytes, at most 55, are held in `m`. It holds
  // those bytes, the byte 0x80, zeros, and the message's length in bits as a
  // 64-bit big-endian integer.
  spec last_block(m: Word[8]^64, length: Int, total: Int) -> Word[8]^64 {
    let bits: Int = 8 * total;
    for j in 0..64 with b: Word[8]^64 = [0; 64] {
      b with [j] = if j < length { m[j] }
        else if j == length { 0x80 }
        else if j < 56 { 0 }
        else { (bits / weight(63 - j)) as Word[8] }
    }
  }

  spec be_bytes(x: Word[32]) -> Word[8]^4 {
    [(x >> 24) as Word[8], (x >> 16) as Word[8], (x >> 8) as Word[8], x as Word[8]]
  }

  // Section 6.2.2: the hash value as 32 big-endian bytes.
  spec digest(h: Word[32]^8) -> Word[8]^32 {
    for i in 0..8 with d: Word[8]^32 = [0; 32] {
      for j in 0..4 with e: Word[8]^32 = d { e with [4 * i + j] = be_bytes(h[i])[j] }
    }
  }

  // Section 6.2: the hash of at most 55 bytes held in `m`.
  spec hash(m: Word[8]^64, length: Int) -> Word[8]^32 {
    digest(compress(initial(), last_block(m, length, length)))
  }

  spec abc() -> Word[8]^32 {
    let m: Word[8]^3 = [0x61, 0x62, 0x63];
    hash(for i in 0..3 with b: Word[8]^64 = [0; 64] { b with [i] = m[i] }, 3)
  }

  test "FIPS 180-4 5.1.1 abc, NIST SHA-256 one-block sample" {
    abc() == [
      0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde,
      0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
      0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    ]
  }
}
```

Read the listing by the names `hmac` is allowed to call, not by
re-deriving the round. `initial` is the eight-word value FIPS 180-4
§5.3.3 prints. `compress` absorbs one 64-byte block. `last_block`
builds the final block of a message whose length in bytes is `total`
and whose last `length` bytes, at most 55, sit at the front of `m`.
`digest` writes the eight words as 32 bytes, high byte first.
`hash` is `digest` of one `compress` of `initial` with that final
block, and only when the whole message is the final block. The
constants in `round_constants` are the words §4.2.2 prints. The body
of `round` is §6.2.2's step 3, transcribed. Journeyman work can derive
those words. This page checks one message and then calls the names.

Three specs take no arguments: `round_constants`, `initial`, and
`abc`, in that source order. A bare `orangec eval sha256.or` prints
all three. The two tables are already on the page. The command this
lesson checks is `eval --spec abc`. The option is the one N8 used. It
selects parameterless specs of the root. It does not run a test.
`orangec test` runs the root's tests, and it rejects `--spec`.

```sh
./compiler/target/debug/orangec check sha256.or
./compiler/target/debug/orangec eval --spec abc sha256.or
./compiler/target/debug/orangec test sha256.or
```

Check is silent. The status is 0. Silence means the source was
well-formed. It does not mean the digest matched.

**Expected evaluation output:**

```text
sha256::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]
```

**Test report:**

```text
test "FIPS 180-4 5.1.1 abc, NIST SHA-256 one-block sample" ... ok
1 test: 1 passed, 0 failed
```

The report says the `Bool` in that test was true on this run. §N13.3
says which bytes were compared, and what was not compared. The report
does not say it.

**Listing N13.2 — `hmac.or`**

The file sits in the same directory as `sha256.or`. The compiler reads
`sha256.or` because of the `use`, not because the two texts were
pasted together.

```orange
edition 2026;
module hmac {
  use sha256;

  spec keyed(key: Word[8]^64, pad: Word[8]) -> Word[8]^64 {
    for i in 0..64 with b: Word[8]^64 = key { b with [i] = key[i] ^ pad }
  }

  spec block(d: Word[8]^32) -> Word[8]^64 {
    for i in 0..32 with b: Word[8]^64 = [0; 64] { b with [i] = d[i] }
  }

  spec mac(key: Word[8]^64, m: Word[8]^64, length: Int) -> Word[8]^32 {
    let inner: Word[32]^8 = sha256::compress(sha256::initial(), keyed(key, 0x36));
    let outer: Word[32]^8 = sha256::compress(sha256::initial(), keyed(key, 0x5c));
    let text: Word[8]^32 =
      sha256::digest(sha256::compress(inner, sha256::last_block(m, length, 64 + length)));
    sha256::digest(sha256::compress(outer, sha256::last_block(block(text), 32, 96)))
  }

  spec case1_key() -> Word[8]^64 {
    for i in 0..20 with b: Word[8]^64 = [0; 64] { b with [i] = 0x0b }
  }

  spec hi_there() -> Word[8]^32 {
    let data: Word[8]^8 = [0x48, 0x69, 0x20, 0x54, 0x68, 0x65, 0x72, 0x65];
    mac(case1_key(), for i in 0..8 with b: Word[8]^64 = [0; 64] { b with [i] = data[i] }, 8)
  }

  test "RFC 2104 pads differ by 0x6a on the RFC 4231 4.2 key" {
    let key: Word[8]^64 = case1_key();
    let inner: Word[8]^64 = keyed(key, 0x36);
    let outer: Word[8]^64 = keyed(key, 0x5c);
    for i in 0..64 with ok: Bool = true { ok && ((inner[i] ^ outer[i]) == 0x6a) }
  }

  test "RFC 2104 step 1 zero-pad of the RFC 4231 4.2 key" {
    let key: Word[8]^64 = case1_key();
    let head: Bool = for i in 0..20 with ok: Bool = true { ok && (key[i] == 0x0b) };
    let tail: Bool = for i in 20..64 with ok: Bool = true { ok && (key[i] == 0) };
    head && tail
  }

  test "RFC 4231 4.2 HMAC-SHA-256 test case 1" {
    hi_there() == [
      0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce,
      0xaf, 0x0b, 0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7,
      0x26, 0xe9, 0x37, 0x6c, 0x2e, 0x32, 0xcf, 0xf7,
    ]
  }

  test "RFC 4231 4.3 HMAC-SHA-256 test case 2" {
    let key: Word[8]^4 = [0x4a, 0x65, 0x66, 0x65];
    let data: Word[8]^28 = [
      0x77, 0x68, 0x61, 0x74, 0x20, 0x64, 0x6f, 0x20, 0x79, 0x61, 0x20, 0x77, 0x61, 0x6e,
      0x74, 0x20, 0x66, 0x6f, 0x72, 0x20, 0x6e, 0x6f, 0x74, 0x68, 0x69, 0x6e, 0x67, 0x3f,
    ];
    mac(
      for i in 0..4 with b: Word[8]^64 = [0; 64] { b with [i] = key[i] },
      for i in 0..28 with b: Word[8]^64 = [0; 64] { b with [i] = data[i] },
      28
    ) == [
      0x5b, 0xdc, 0xc1, 0x46, 0xbf, 0x60, 0x75, 0x4e, 0x6a, 0x04, 0x24, 0x26,
      0x08, 0x95, 0x75, 0xc7, 0x5a, 0x00, 0x3f, 0x08, 0x9d, 0x27, 0x39, 0x83,
      0x9d, 0xec, 0x58, 0xb9, 0x64, 0xec, 0x38, 0x43,
    ]
  }
}
```

Count the qualified calls in `mac`. There are ten. `initial` appears
twice, `compress` four times, `last_block` twice, and `digest` twice.
The four names are the four operations Assumption N13.1 assigned to
the hash module. `hash` is not among them. §N13.4 is the reason.
`keyed`, `block`, `mac`, `case1_key`, and `hi_there` are functions of
`hmac`. An unqualified call inside this file names one of those, or it
names nothing this file declares. `case1_key` is the 64-byte block of
one vector. RFC 2104 does not define a function by that name.

```sh
./compiler/target/debug/orangec check hmac.or
./compiler/target/debug/orangec eval hmac.or
./compiler/target/debug/orangec test hmac.or
```

Both files are in the directory where you run the command. Check is
silent. Eval prints the two parameterless specs, in source order.
`case1_key` is first. Twenty bytes are `0x0b` and the remaining
forty-four are `0x00`. That array is the input the pad test and
`hi_there` share. `hi_there` is the tag.

**Expected evaluation output:**

```text
hmac::case1_key: Word[8]^64 = [0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
hmac::hi_there: Word[8]^32 = [0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b, 0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c, 0x2e, 0x32, 0xcf, 0xf7]
```

**Test report:**

```text
test "RFC 2104 pads differ by 0x6a on the RFC 4231 4.2 key" ... ok
test "RFC 2104 step 1 zero-pad of the RFC 4231 4.2 key" ... ok
test "RFC 4231 4.2 HMAC-SHA-256 test case 1" ... ok
test "RFC 4231 4.3 HMAC-SHA-256 test case 2" ... ok
4 tests: 4 passed, 0 failed
```

Four tests passed. The first is the pad claim on one key. The second
is the zero-pad of that same key. The third and fourth are the two
HMAC-SHA-256 vectors. Each one is a `Bool` on the inputs written in
that test. The next two sections say what those `Bool`s are, and what
they are not.

### N13.3 Provenance of an expected

A known-answer test compares a value the program computes with a value
you copied. The copy is part of the claim. A title that says “the
digest” is not a pin. The pin has four parts: the document, the
edition, the vector's identity in that document, and a statement of
what the comparison does not cover. Assumption N13.5 is the pin for
the two digests this lesson copies. The details are here so the
assumption can be checked against the pages.

**Provenance — SHA-256 of “abc”.** The algorithm is FIPS PUB 180-4,
*Secure Hash Standard (SHS)*, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. NIST's note on the archived March 2012 file
says the 2015 edition supersedes it and that the only change is the
Applicability clause. The technical specification consulted here is
that text. Section 5.1.1 pads the 8-bit ASCII message “abc”, whose bit
length is `8 × 3 = 24`. The section prints the padding diagram. It
does not print the digest.

The 32-byte digest is the one-block message sample of NIST's SHA-256
examples, input message “abc”, message digest

```text
BA7816BF 8F01CFEA 414140DE 5DAE2223 B00361A3 96177A9C B410FF61 F20015AD
```

That examples file is not a section of FIPS 180-4. The test title
names both documents for that reason. The bytes `0x61`, `0x62`, and
`0x63` in `abc` are the ASCII bytes of “a”, “b”, and “c”, in that
order, which is the message §5.1.1 pads. The expected array is the
examples line, one byte at a time, in the order the line is printed.

The test does not cover the examples file's two-block message. It does
not cover the empty message. It does not cover a message longer than
55 bytes, which `hash` does not denote. It does not prove that every
block agrees with §6.2.2. It does not say the evaluator is
constant-time. It does not say anything about collisions. A passing
report is a Match of this one 32-byte array on this one message. Do
not call that Match verified.

RFC 4231's normative reference for the hash is FIPS 180-2, August
2002, with Change Notice 1 dated February 2004, not FIPS 180-4. The
HMAC bytes below are pinned to RFC 4231. They are not pinned to a
claim that RFC 4231 names the 2015 edition.

**Provenance — HMAC-SHA-256, RFC 4231 test case 1.** The definition of
the tag is RFC 2104, February 1997, §2. The vector is RFC 4231,
*Identifiers and Test Vectors for HMAC-SHA-224, HMAC-SHA-256,
HMAC-SHA-384, and HMAC-SHA-512*, M. Nystrom, December 2005, §4.2,
test case 1, the HMAC-SHA-256 line. The key is twenty bytes `0x0b`.
The data line is `4869205468657265` with the note `("Hi There")`.
Those eight hex bytes are `H`, `i`, space, `T`, `h`, `e`, `r`, `e`.
The HMAC-SHA-256 digest printed under that data is

```text
b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7
```

The same section also prints HMAC-SHA-224, HMAC-SHA-384, and
HMAC-SHA-512. The test compares the SHA-256 line only. A match against
the SHA-224 line would be a match against a different scheme.

The test does not cover §4.3 except insofar as the next test does.
It does not cover §4.4 or §4.5. It does not cover §4.6, which
truncates the tag to 128 bits, because `mac` returns 32 bytes. It
does not cover §4.7 or §4.8. Section 4.7's key is 131 bytes. That
length is outside `mac`. It does not cover a key longer than 64 bytes
at all. It does not establish the security discussion in RFC 2104 §6.
A passing report is a Match of this one tag on this one key and this
one eight-byte text. Do not call that Match verified.

**Provenance — HMAC-SHA-256, RFC 4231 test case 2.** Same definition,
RFC 2104 §2. The vector is RFC 4231 §4.3, test case 2. The key line is
`4a656665` with the note `("Jefe")`. Those four bytes are `J`, `e`,
`f`, `e`. The data is the 28 bytes of “what do ya want for nothing?”,
split in the RFC across two quoted pieces that join into that
sentence. The HMAC-SHA-256 digest is

```text
5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843
```

The section's heading in the RFC is “Test with a key shorter than the
length of the HMAC output.” The key is 4 bytes and the SHA-256 tag is
32 bytes, so the heading's comparison holds for this line. The test
does not cover a key of any other short length. It does not cover the
other three digests in the same section. A passing report is a Match
on this key and this 28-byte text. Do not call that Match verified.

The four `Bool`s in Listing N13.2 are independent claims. Passing the
pad test does not run `hi_there`. Passing test case 1 does not run
test case 2. The report's last line counts them. It does not merge
them into one universal sentence.

### N13.4 The composition claim

The composition is the formula in §N13.1. Two facts about it can be
proved from the definitions without running the hash. A third fact
says why the inner call cannot be `hash`. The Orange tests that follow
the proofs are Matches on stated finite domains. They are not the
proofs.

**Proposition N13.1 — The block is 64 bytes.** SHA-256's message block
is 64 bytes. For that hash, RFC 2104's B is 64, and both ipad and opad
have length 64.

*Proof.* FIPS 180-4 states that for SHA-256 each message block has 512
bits. A byte is 8 bits. `512 = 64 × 8`, so a block is 64 bytes. RFC
2104 defines B as the byte-length of H's block and defines ipad as the
byte `0x36` repeated B times. The same sentences define opad with
`0x5c`. Both strings have length B. For this H, that length is 64. □

The type `Word[8]^64` is that length. The type is not the proof. The
division is the proof. A different hash, whose block were 128 bytes,
would make B = 128 under the same RFC sentence. SHA-256 is not that
hash. The parenthetical in RFC 2104 §2 that says B = 64 names the hash
functions that RFC discusses, which are MD5, SHA-1, and RIPEMD. It
does not name SHA-256. Proposition N13.1 does not borrow that
parenthetical. It uses the FIPS sentence about SHA-256.

**Proposition N13.2 — The two pad bytes differ in a fixed way.** For
every byte `k`,

```text
(k XOR 0x36) XOR (k XOR 0x5c) = 0x6a
```

and `0x6a ≠ 0x00`.

*Proof.* Write the two pad bytes with the high bit on the left:

```text
0x36 = 00110110
0x5c = 01011100
XOR  = 01101010 = 0x6a
```

`0x6a` is not `0x00`, so the pad bytes differ in at least one bit.

Now take an arbitrary byte `k` and one bit position. Let `b` be `k`'s
bit there, `p` the bit of `0x36`, and `q` the bit of `0x5c`. The
result bit is `(b XOR p) XOR (b XOR q)`. Chapter 3's table is `0 XOR 0
= 0`, `0 XOR 1 = 1`, `1 XOR 0 = 1`, and `1 XOR 1 = 0`.

If `b = 0`, then `0 XOR p = p` and `0 XOR q = q`, so the result is
`p XOR q`.

If `b = 1`, then `1 XOR p` is the flip of `p` and `1 XOR q` is the
flip of `q`. Flipping both inputs leaves XOR unchanged: the flipped
pairs are `11`, `10`, `01`, and `00`, and the table sends them to `0`,
`1`, `1`, and `0`, which are the results of the unflipped pairs `00`,
`01`, `10`, and `11`. So the result is again `p XOR q`.

Every bit position therefore contributes the corresponding bit of
`0x36 XOR 0x5c`, which is `0x6a`. The byte `k` was arbitrary. □

The eight positions do not depend on one another, and the byte `k`
does not depend on a previous byte. The argument is one argument for
an arbitrary byte. It is not a chain.

Assumption N9.4 is the successor step on a stated bound: prove `Q(0)`,
and prove `Q(n) ⇒ Q(n + 1)` from `Q(n)` alone. The pad identity is not
that shape. There is no `Q(n)` whose truth at byte `n` is what the
proof at byte `n + 1` uses. Citing the assumption here would be citing
a rule the proof does not apply. The finite form of that assumption is
available when a claim really is a chain. This claim is not.

**Proposition N13.3 — The keyed blocks differ at every index.** Let `K`
be any string of 64 bytes. Define `I[i] = K[i] XOR 0x36` and
`O[i] = K[i] XOR 0x5c` for each integer `i` with `0 ≤ i ≤ 63`. Then
`I[i] XOR O[i] = 0x6a`, so `I` and `O` differ at every index.

*Proof.* `K[i]` is a byte. Proposition N13.2 applied to that byte is
the displayed equation. The index `i` was an arbitrary integer in the
stated range. The argument named no other index. □

Listing N13.2's first test builds one `K`, the block `case1_key`,
computes `keyed(K, 0x36)` and `keyed(K, 0x5c)`, and checks the
equation at each `i` in `0..64`. Those are the indices `0` through
`63`. The key's first twenty bytes are `0x0b` and the rest are `0x00`,
so the bytes of `K` that the test actually feeds to Proposition N13.2
are only `0x0b` and `0x00`. A passing report is a Match on those 64
positions of that one key. Do not call that Match verified. The
proposition is the claim for every 64-byte key. The test is not that
claim. The proof did not become a proof by the test passing, and the
test did not become a proof by sitting next to the proof.

One index is worth doing in full so the hex is not only a citation of
the proposition. Take `K[0] = 0x0b`.

```text
0x0b = 00001011
0x36 = 00110110
XOR  = 00111101 = 0x3d

0x0b = 00001011
0x5c = 01011100
XOR  = 01010111 = 0x57

0x3d = 00111101
0x57 = 01010111
XOR  = 01101010 = 0x6a
```

Index 0 of this key agrees with Proposition N13.2. The other
sixty-three indices of this key are the same proposition on whichever
of the two bytes sits there. They are not sixty-three further ideas.

**Proposition N13.4 — A shorter key is zero-padded, and a longer key
is outside this module.** Let `L` be an integer with `0 ≤ L < 64`, and
let the key be `L` bytes. RFC 2104 §2, step (1), appends zeros until
the string has length B. For this hash, B = 64, so the number of zero
bytes is `64 − L`. A key longer than B is not handled by that step
alone: the same section says to hash it with H first and then to use
the resulting L-byte string as the key. For SHA-256 that digest is 32
bytes, which step (1) then pads with 32 zeros. `keyed` and `mac` take
a `Word[8]^64`. They contain no call that hashes a key.

*Proof of the short-key count.* Start from 64 zeros and write the key
into indices `0` through `L − 1` only. An index `j` with
`L ≤ j ≤ 63` is never written, so it remains `0`. The number of
integers from `L` through `63` inclusive is `64 − L`. □

The long-key sentence is a reading of RFC 2104 §2, not a property of
`mac`. RFC 4231 §4.7 is the vector that needs it: the key is 131
bytes, and 131 is greater than 64. Listing N13.2 has no input of that
length and no test of that section. A passing test of §4.2 does not
speak about §4.7.

For the key in §4.2, `L = 20`, so the zero count is `64 − 20 = 44`.
The second test checks that `case1_key` has `0x0b` at indices `0`
through `19` and `0` at indices `20` through `63`. A passing report is
a Match on that one block. Do not call that Match verified. It does
not mention a key of length 19, or of length 4, except that test case
2 builds a different block of length 4 by the same fill-and-write.
Test case 2's block is checked only by being the key argument of
`mac`, not by a second zero-pad test.

**Proposition N13.5 — The inner string is outside `hash`.** For every
integer `L ≥ 0`, the inner string of RFC 2104 steps (2) and (3) has
length `64 + L`. That length is greater than 55, so the string is
outside `sha256::hash`.

*Proof.* The string XORed with ipad has length 64 by Proposition
N13.1, and XOR does not change length. Appending a text of length `L`
adds `L` bytes. `64 + L ≥ 64`, and `64 > 55`. Assumption N13.1 limits
`hash` to messages of at most 55 bytes. □

So `mac` cannot call `hash` on the inner string, for any text,
including the empty text. It calls `compress` on the 64-byte keyed
pad, which is one full block, and `last_block` on the text. The
`total` argument of that `last_block` is `64 + L`, the length of the
inner string, not the length of the text alone. The outer string is
the 64-byte opad block followed by the 32-byte inner digest, so its
length is `64 + 32 = 96`. The call writes that `96` as a literal. It
is the pad plus one SHA-256 digest. It is not a third copy of the
text's length.

For test case 1, `L = 8`, so the inner string has length 72. The bit
length stored in the final block is `8 × 72 = 576`. In hex, `576 =
0x240`, so the last two bytes of that 64-byte block are `0x02` and
`0x40`, and the six length bytes before them are `0x00`. For the outer
string the bit length is `8 × 96 = 768 = 0x300`, so the last two bytes
are `0x03` and `0x00`. Those bytes are what `last_block` computes from
`total` under Euclidean division of the positive integer `8 × total`
by the powers of 256. The HMAC tests do not print them. They match the
final tag, which depends on them. A wrong `total` that still happened
to produce the RFC's tag on these two texts would pass. The tests are
not a proof that `96` is the outer length. Proposition N13.5 and the
count `64 + 32` are that argument.

The same bound `L ≤ 55` is what lets the text occupy one final block.
`last_block` writes the text, then the byte `0x80`, then zeros, then
an 8-byte length. The length occupies indices 56 through 63. A text of
56 or more bytes would meet that field. Assumption N13.2 stops at 55
so that the `0x80` still has an index of its own. Test case 1 uses 8.
Test case 2 uses 28. Neither test is the bound.

```text
n13-ledger
block-bits = 512
block-bytes = 64
pad-xor = 106
byte0-inner = 61
byte0-outer = 87
case1-key-len = 20
case1-zero-bytes = 44
inner-total = 72
inner-bits = 576
outer-total = 96
outer-bits = 768
```

`0x6a = 106`, `0x3d = 61`, and `0x57 = 87`. The reference check
recomputes each line from those definitions. It also checks Proposition
N13.2 for every byte from `0` through `255`, which is the whole domain
of one byte. That exhaustive check is not the Orange test. The Orange
test remains the 64 positions of one key.

### N13.5 Failure at the seam

A seam fails in a particular way. The call names a module the file
does not use, or it names a function the used module does not declare,
or the `use` names a file that is not beside this one. Each of those
is a diagnostic the compiler prints, and each one has a repair that
does not rewrite the hash. The habit is N8's: read the code, the
message, the locus, and the note, then change only what the note
identifies.

**Listing N13.3 — `seam_gap.or`, intentionally rejected**

```orange
edition 2026;
module seam_gap {
  spec tag() -> Word[8]^32 {
    sha256::hash([0; 64], 0)
  }
}
```

```sh
./compiler/target/debug/orangec check -
```

**Diagnostic:**

```text
error[ORC0229]: module `sha256` is not used by `seam_gap`
 --> <stdin>:4:5
  |
4 |     sha256::hash([0; 64], 0)
  |     ^^^^^^ no `use` declaration names this module
  = note: declare `use sha256;` at the head of the module to call its functions
```

The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic and no value.

The code is `ORC0229`. The message says module `sha256` is not used by
`seam_gap`. The locus is `<stdin>`, line 4, column 5, the start of the
name `sha256`. Six carets cover that name. The label says no `use`
declaration names this module. The note says to declare `use sha256;`
at the head of the module. The note does not say to copy `hash` into
`seam_gap`, and it does not say to delete the call. The qualifier is
the text the carets mark. The repair keeps the qualifier and adds the
declaration the note names.

Adding the declaration is not, by itself, the whole repair. The file
`sha256.or` has to sit in the directory of the file that uses it, and
`hash` has to be a typed `spec` of that module. Listing N13.4 is the
repaired call. It is the “abc” message from §N13.3, reached by
`sha256::hash`. It is not `mac`.

**Listing N13.4 — `seam_repaired.or`**

```orange
edition 2026;
module seam_repaired {
  use sha256;

  spec abc() -> Word[8]^32 {
    let m: Word[8]^3 = [0x61, 0x62, 0x63];
    sha256::hash(for i in 0..3 with b: Word[8]^64 = [0; 64] { b with [i] = m[i] }, 3)
  }

  test "FIPS 180-4 5.1.1 abc, called as sha256::hash" {
    abc() == [
      0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde,
      0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c,
      0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad,
    ]
  }
}
```

```sh
./compiler/target/debug/orangec check seam_repaired.or
./compiler/target/debug/orangec eval seam_repaired.or
./compiler/target/debug/orangec test seam_repaired.or
```

`sha256.or` is in the same directory. Check is silent.

**Expected evaluation output:**

```text
seam_repaired::abc: Word[8]^32 = [0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00, 0x15, 0xad]
```

**Test report:**

```text
test "FIPS 180-4 5.1.1 abc, called as sha256::hash" ... ok
1 test: 1 passed, 0 failed
```

The bytes are the same 32 bytes as Listing N13.1. The call that
produced them is `sha256::hash`, written in another module. A passing
report is a Match of that one call on the message “abc”. Do not call
that Match verified. It does not run `mac`. It does not mention a key.
Repairing the seam restored the call the diagnostic was about. It did
not promote the call into HMAC.

**Listing N13.5 — `wrong_name.or`, intentionally rejected**

```orange
edition 2026;
module wrong_name {
  use sha256;

  spec tag() -> Word[8]^32 {
    sha256::final([0; 64], 0)
  }
}
```

```sh
./compiler/target/debug/orangec check wrong_name.or
```

Run that command in the directory that contains `sha256.or`, and pass
the file name `wrong_name.or`. The locus below uses that name.

**Diagnostic:**

```text
error[ORC0212]: no typed `spec` function named `final` in module `sha256`
 --> wrong_name.or:6:13
  |
6 |     sha256::final([0; 64], 0)
  |             ^^^^^ unknown function
  = note: a qualified call names a typed `spec` of the used module
```

The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic.

The code is `ORC0212`. The message says there is no typed `spec`
named `final` in module `sha256`. The caret starts at column 13, under
`final`, not under `sha256`. The module name was accepted: the `use`
is present, and the file was read. The label is `unknown function`.
The note says a qualified call names a typed `spec` of the used
module. `sha256` declares `hash`, `compress`, `initial`, `last_block`,
and `digest`. It does not declare `final`. The minimal repair is to
call one of the names it does declare. Listing N13.4 calls `hash`.
Replacing `sha256` by some other module name would answer a different
diagnostic, the one Listing N13.3 already showed, and would leave
`final` in place if that other module were somehow used.

**Listing N13.6 — `missing_file.or`, intentionally rejected**

```orange
edition 2026;
module missing_file {
  use absent;

  spec one() -> Int { 1 }
}
```

```sh
./compiler/target/debug/orangec check missing_file.or
```

Pass the file name `missing_file.or`, with no directory prefix, from
the directory that does not contain `absent.or`.

**Diagnostic:**

```text
error[ORC1001]: could not read source file `absent.or`
  = note: file was not found
  = note: `use absent;` in module `missing_file` reads the module `absent` from this file
```

The status is 1. Standard output is empty. `eval` and `test` print
the same diagnostic and no value. There is no source line and no
caret. The name that failed was a file name, not a span inside
`missing_file.or`. The code is `ORC1001`. The message names
`absent.or`. The first note says the file was not found. The second
note says that `use absent;` in module `missing_file` reads the module
`absent` from this file. Assumption N13.3 is that sentence, printed
as a diagnostic. The repair is not to invent a module called `absent`.
The repair is to use a module whose file is in this directory and
whose declared name is the name in the `use`. Listing N13.2 does that
with `sha256`. A program that `use`s `sha256` while the file beside it
declares `module hash` is the same failure: the file that was read is
not a module of the name the `use` asked for, and the graph stops on
the missing name.

The three diagnostics are three different sentences. `ORC0229` means
the qualifier names a module this module does not use. `ORC0212` means
the module is used and the function is not among its typed specs.
`ORC1001` means the file named by the `use` could not be read. Copying
a repair from one of them onto another leaves the failure the compiler
actually reported.

### N13.6 Work at the desk

The ten exercises are the check. A wrong answer is a place to
recompute. The worked answers are the finished path, not a path to
skip. Nothing in them is a grade, and nothing in them is a security
claim.

**Exercise N13.1 — Two calls and one formula.** In
`H(K XOR opad, H(K XOR ipad, text))`, which occurrences are H, and
which object is the HMAC tag? Using Definition N11.4, say what would
still have to be true of the sets before H is a hash in that
definition's sense. Using Definition N11.5, say what the formula has
not yet provided.

**Exercise N13.2 — Where an unqualified name is looked up.** Quote the
`use` line of Listing N13.2 and the first qualified call in `mac`. If
the qualifier were deleted from `sha256::initial()`, which module
would be searched for `initial`, and does that module declare it?

**Exercise N13.3 — Pin “abc”.** Name the document and edition that
specify SHA-256, the section that uses the message “abc”, and the
document that prints the 32-byte digest. Give two facts the “abc” test
does not establish.

**Exercise N13.4 — One byte of the pad.** Compute `0x36 XOR 0x5c` by
bits. Then compute `(0x0b XOR 0x36) XOR (0x0b XOR 0x5c)`.

**Exercise N13.5 — Three key lengths.** B = 64. How many zero bytes
does RFC 2104 step (1) append to a key of 20 bytes? To a key of 64
bytes? For the 131-byte key of RFC 4231 §4.7, what does RFC 2104 §2
require before that padding, and does `mac` do it?

**Exercise N13.6 — Why the inner call is not `hash`.** Test case 1's
text is 8 bytes. Give the length of the inner string. Why is
`sha256::hash` the wrong function for that string? Which two hash
operations does `mac` use instead, and what integer does it pass as
`total`?

**Exercise N13.7 — Repair the missing `use`.** From Listing N13.3's
diagnostic, give the code, the column where the caret starts, the
label, and the note. What is the minimal repair, and which listing is
that repaired call? What does a passing test of the repair establish
about HMAC?

**Exercise N13.8 — Repair the wrong function.** In Listing N13.5, why
is the caret under `final` rather than under `sha256`? What does the
note require the used module to contain? Which function does Listing
N13.4 call instead?

**Exercise N13.9 — A Match on a smaller domain.** State the domain of
the pad test in Listing N13.2 and the domain of Proposition N13.3.
Why is a passing report a Match on the test's domain? Why does that
claim not have the shape `Q(n) ⇒ Q(n + 1)`?

**Exercise N13.10 — What remains open.** Name one assumption a passing
test of `hi_there` still depends on. Name one RFC 4231 section the
module does not contain. Say what a silent `orangec check` of
`hmac.or` established, and say one thing it did not establish.

## Worked answers

**N13.1.** The inner H and the outer H are the two occurrences. The
HMAC tag is the value of the whole formula: both calls, both pads,
and the key. H is a hash in the sense of Definition N11.4 only when
it is a function from a set `S` to a set `T` with `|S| > |T|` and
`|T| ≥ 1`. For the messages of length exactly 55 that `hash` accepts,
`|S| = 256^55 = 2^440` and `|T| = 2^256`, so the cardinalities hold
on that set. The formula does not provide Definition N11.5's check.
A check returns true or false on a plaintext, a tag, and, when the
construction says so, a key. Nothing in the formula returns a truth
value, and nothing in it rejects a substitute tag.

**N13.2.** The declaration is `use sha256;`. The first qualified call
in `mac` is `sha256::compress`, and its first argument is the call
`sha256::initial()`. Deleting the qualifier from `sha256::initial()`
leaves `initial()`, which Assumption N13.3 looks up in the calling
module, `hmac`. Listing N13.2 does not declare `initial`. The name
`initial` in `sha256` is a different function and is not a candidate
for that unqualified call.

**N13.3.** SHA-256 is specified by FIPS PUB 180-4, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. Section 5.1.1 uses the 8-bit ASCII message
“abc” as its padding example and does not print a digest. The 32-byte
digest is the one-block message sample of NIST's SHA-256 examples,
input message “abc”. Two facts the test does not establish: it does
not establish the examples file's two-block digest, and it does not
establish that every message of length at most 55 matches §6.2.2. The
report is a Match on the one message `abc` writes.

**N13.4.**

```text
0x36 = 00110110
0x5c = 01011100
XOR  = 01101010 = 0x6a
```

```text
0x0b XOR 0x36 = 0x3d
0x0b XOR 0x5c = 0x57
0x3d XOR 0x57 = 0x6a
```

The second result is Proposition N13.2 on the byte `0x0b`. It is the
index-0 case written out under Proposition N13.3.

**N13.5.** For a 20-byte key, step (1) appends `64 − 20 = 44` zero
bytes. For a 64-byte key it appends `64 − 64 = 0` zero bytes. The
131-byte key of RFC 4231 §4.7 is longer than B. RFC 2104 §2 requires
H of that key first. SHA-256's digest is 32 bytes, and step (1) then
appends 32 zero bytes. `mac` takes a `Word[8]^64` and does not call
the hash on a key. Proposition N13.4 is that boundary.

**N13.6.** The inner string has length `64 + 8 = 72`. Proposition
N13.5 says every inner string has length at least 64, and 64 is
already greater than 55, so `sha256::hash` does not denote it. `mac`
calls `sha256::compress` on the 64-byte pad and `sha256::last_block`
on the text, with `total` equal to `64 + length`, which is 72 for
this text. The bit length in that final block is `8 × 72 = 576`.

**N13.7.** The code is `ORC0229`. The caret starts at column 5. The
label is `no use declaration names this module`. The note tells
you to declare `use sha256;` at the head of the module. The
minimal repair is that declaration, together with `sha256.or` beside
the file and a call to a typed `spec` that module declares. Listing
N13.4 is that repaired call. Its passing test is a Match of
`sha256::hash` on “abc”. It does not call `mac`, it does not take a
key, and it does not establish HMAC.

**N13.8.** The caret is under `final` because the module name was
resolved. The `use` is present, `sha256.or` was read, and the missing
name is the function. The note requires the callee to be a typed
`spec` of the used module. Listing N13.4 calls `sha256::hash`, which
is one of those specs. It does not call `final`.

**N13.9.** The pad test's domain is the 64 indices of the one block
`case1_key` returns. The bytes of that block are twenty copies of
`0x0b` and forty-four copies of `0x00`. Proposition N13.3's domain is
every 64-byte string and every index from 0 through 63. A passing
report says the `Bool` was true on the test's 64 positions. That is a
Match on that domain. Do not call that Match verified. The proof of
the proposition uses an arbitrary index and the byte at that index.
It does not use a property of the preceding index. Assumption N9.4
would require a step `Q(n) ⇒ Q(n + 1)` from `Q(n)` alone. The pad
claim does not have that step.

**N13.10.** The passing test of `hi_there` still depends on
Assumption N13.1: the functions `mac` calls denote SHA-256 on the
blocks it passes them. The “abc” Match is one message of three bytes.
It is not a proof of `compress` on the keyed pad. The module does not
contain RFC 4231 §4.7. A silent check established that `hmac.or` and
the `sha256.or` it uses were well-formed, including the `use` and the
ten qualified calls. It did not run the four tests, and it did not
establish that `hi_there` equals the §4.2 digest. That equality is
the test report, which is a Match on that vector and not a
verification of the construction.

## Sources and epigraph record

The quotation and the copied vectors are the borrowed words. The
proofs are the lesson's.

**[S11] H. Krawczyk, M. Bellare, and R. Canetti.** “HMAC: Keyed-Hashing
for Message Authentication,” RFC 2104, February 1997, Informational.
The epigraph is the first sentence of §2, “Definition of HMAC,” on
page 3 of the plain-text RFC. The sentence contains no inner quotation
marks. Wording was checked on 2026-10-05 against the RFC Editor text.
No translation is involved. The same section is the source of the pad
bytes, of step (1)'s zero-padding, and of the rule that a key longer
than B is hashed first. Those rules are cited as the RFC. The
epigraph is not an endorsement of Orange, and it is not the security
argument of §6, which this lesson does not transcribe.
This record's tag is [S11].

Source: <https://www.rfc-editor.org/rfc/rfc2104>

**[D1] FIPS PUB 180-4.** National Institute of Standards and
Technology, *Secure Hash Standard (SHS)*, August 2015, DOI
`10.6028/NIST.FIPS.180-4`. The technical text consulted on 2026-10-05
is the archived file whose cover states that this edition supersedes
the March 2012 publication and that the only change is the
Applicability clause. Section 4 states that a SHA-256 message block
has 512 bits. Section 4.2.2 prints the sixty-four round constants.
Section 5.1.1 prints the padding of the message “abc” and does not
print a digest. Section 5.3.3 prints the eight-word initial hash
value. Section 6.2.2 is the hash computation the listing transcribes
and does not derive.

<https://doi.org/10.6028/NIST.FIPS.180-4>

**[N1] NIST SHA-256 examples.** The one-block message sample, input
message “abc”, in the file titled as a SHA-256 message-digest example,
consulted on 2026-10-05. The message digest line is the 32 bytes in
Listing N13.1's test. This file is not a section of [D1]. The test
title names both so that the digest is not attributed to a section
that does not print it.

<https://csrc.nist.gov/CSRC/media/Projects/Cryptographic-Standards-and-Guidelines/documents/examples/SHA256.pdf>

**[V1] RFC 4231.** M. Nystrom, “Identifiers and Test Vectors for
HMAC-SHA-224, HMAC-SHA-256, HMAC-SHA-384, and HMAC-SHA-512,” December
2005, Standards Track. Sections 4.2 and 4.3 were copied for test cases
1 and 2, HMAC-SHA-256 lines only. Section 4.7 was read for the
131-byte key and was not transcribed. The RFC's normative hash
reference is FIPS 180-2, not [D1]. Consulted on 2026-10-05 against the
RFC Editor plain text. No erratum was applied.

<https://www.rfc-editor.org/rfc/rfc4231>

**[T7] Orange modules.** `docs/MODULES_2026.md`, proposed under
OEP-0011. A module names the modules it uses with `use` declarations
before its functions, and it calls their typed `spec` functions by
module name, as in `sha256::compress(...)`. The compiler reads module
`NAME` from the file `NAME.or` beside the file that uses it. Only the
root module's tests are run. The listings use that surface and no
other module form. Implementation of the proposal is not acceptance
of the proposal.
This record's tag is [T7].

The test form is the one N12 recorded as [T6]. `orangec check` checks
a test and does not run it. `orangec eval` runs parameterless specs
and does not run tests. `orangec test` runs the root module's tests
after the checks. A failed check prints the diagnostic and no report.
The titles in Listings N13.1, N13.2, and N13.4 name the document and
the vector that supplied the expected bytes.

Epigraph verification establishes wording and attribution, not
publication-rights clearance.

## Evidence boundary

N13 adds ten exercises with worked answers. The integer ledger is
recomputed by `tools/test_book_foundations.py`, which also checks
Proposition N13.2 for every byte from 0 through 255. That exhaustive
byte check is not Listing N13.2's pad test. The Orange listings are
the six fenced programs in this file.
`compiler/crates/orangec/tests/book_novice.rs` runs them. Those checks
do not establish a cryptographic security claim, they do not prove
Assumption N13.1 from a smaller theory, and they do not accept
OEP-0011.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction. Owner review is pending. No deployment
recommendation and no certificate of competence is made. A passing
test is a Match on the inputs it writes. It is not called verified.
