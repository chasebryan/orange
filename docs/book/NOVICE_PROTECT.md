# The Orange Book

By Chase Bryan

## Part 1, The Novice

N11: Protect More Than Appearance. Draft 2026-10-05.

Continue from [Count What You Do Not Know](NOVICE_PROBABILITY.md#n10-count-what-you-do-not-know).
This lesson is **N11**. The label is provisional in the same sense as N10:
final numbering waits on the integration plan. The original manuscript keeps
its own chapter numbers. N11 is not manuscript Chapter 11, *Standards as
Versioned Inputs*, and it is not N7, N8, N9, or N10.

You already have, from
[Chapter 1, §1.2](NOVICE_OPENING.md#12-changing-the-appearance), a public
reversal that changes how a message looks and gives the receiver no
advantage over a carrier who knows the method. You have, from §1.3, the
separation of a public method and a selected key, and from §1.6 the
separation of confidentiality, integrity, and authenticity. Chapter 3 gives
XOR and its cancellation identity. Chapter 6 gives congruence and the unique
remainder. N9 gives sets, functions, inverses, and quantifiers. N10 gives
finite probability, conditional probability, and independence. This lesson
uses those results. It does not teach them again.

**Mathematical claim.** A result below is a claim about a stated finite
alphabet, a stated function, or a stated finite probability space. The
quantifiers name their domains, as Definition N9.17 requires.

**Finite check.** The reference test recomputes the ledger at the end:
round trips, key counts, one exhaustive break, and the posterior weights
in the one-time-pad examples. A finite check confirms those values. It
does not replace a proof whose domain is every alphabet size.

**Implementation behavior.** An Orange listing denotes the function its
body writes, on the inputs evaluation actually runs, under the compiler
that accepts the file. A silent check means the source was well-formed.
A printed value means that run produced that value. Neither one is the
mathematical claim, and neither one is a security claim.

### Outcomes

Six outcomes finish the lesson. They are stated here so you can see the
target. The finish line, after the constructions, is the same list with
the sections that discharge each item.

1. You can separate encoding, encryption, hashing, and authentication by
   the claim each definition makes, and you can give a counterexample that
   defeats a conflation of two of them.
2. You can specify the shift, the affine map, and a substitution, prove
   when an inverse exists, and carry out a break whose recognition rule is
   written down.
3. You can specify a repeating-key Vigenère map, reduce a known period to
   independent shifts, and show that the same key on two messages cancels.
4. You can say, of a stated value, whether the sentence in front of you is
   about a key, a distribution, a nonce, or a counter.
5. You can state the one-time pad's conditions and prove perfect secrecy
   for one symbol from N10's definitions, then extend the same arithmetic
   to a finite string. You can show, on a concrete XOR, what a second use
   of the same pad reveals.
6. You can say what an Orange evaluation of these listings established,
   what the exhaustive ledger established, and what neither established.

## N11: Protect More Than Appearance

> “In this case, intercepting the message has given the cryptanalyst no information.”
>
> — Claude E. Shannon, *Communication Theory of Secrecy Systems* (1949),
> §10, p. 680 of the retypeset copy. [S9]

The case in that sentence is the one Shannon has just defined: for every
cryptogram, the a posteriori probabilities of the messages equal their a
priori probabilities. Interception then leaves those probabilities as they
stood. The sentence is a definition's consequence, not a property of every
scrambled string. Most of this lesson is about constructions that change
appearance and still hand the cryptanalyst information. The one-time pad,
under conditions stated later and not under a slogan, is the construction
that meets the definition. Nothing before that section has earned the
epigraph's conclusion.

### N11.1 What you may take as given

Six assumptions bound every later step. They are assumptions of this
lesson. A later sentence that needs a further assumption will name it
there, rather than silently widening one of these.

**Assumption N11.1 — The carrier knows the method.** The adversary in
this lesson knows which construction is in use, including every public
table, every alphabet, and every algorithm step. The adversary does not,
merely by that assumption, know a selected key. This is the reading of
Shannon's system-known hypothesis already used in
[§1.2](NOVICE_OPENING.md#12-changing-the-appearance). It is an analytical
assumption. It is not a report that every historical opponent knew every
system, and it is not a permission to weaken the assumption when a
construction would otherwise fail.

**Assumption N11.2 — Two results may be cited.** Proposition 3.1, the
cancellation identity `(x ⊕ k) ⊕ k = x` for finite equal-length bit
strings, and the existence and uniqueness of the Euclidean quotient and
remainder in §6.3, are already established. So are the congruence facts
you used to reduce an integer to a representative in `0` through `m - 1`.
Citing them is not a new proof of them.

**Assumption N11.3 — Collections and proofs keep their N9 meanings.** A
set is fixed by its members. A function sends each element of its domain
to one element of its codomain. Injective, surjective, and bijective keep
Definitions N9.12 through N9.14. A universal or existential claim names
its domain, as Definition N9.17 requires. Proof by cases is Definition
N9.18. Negating a quantifier is Proposition N9.11. When this lesson says
“every letter,” the set of letters has already been named.

**Assumption N11.4 — Probability keeps its N10 meanings.** A finite
probability space is Definition N10.4. An event's probability is the sum
of the weights of the outcomes where the event says yes. Conditional
probability is Definition N10.7, and it assigns no number when the
conditioning event has probability zero. Independence of events is
Definition N10.9. Independence of choices is Definition N10.10. Key
length, the distribution a key is drawn from, and an adversary's
uncertainty remain three separate quantities, as Proposition N10.15
separated them. The word “random” is still not a technical term. A draw
is described by the space that was stated.

**Assumption N11.5 — A model alphabet is not a natural language.** Unless
a sentence says otherwise, the alphabet `A26` is the set of twenty-six
integers `{0, 1, ..., 25}`. The public names `A` through `Z` are the
roster in that order: `A` names `0`, `B` names `1`, and `Z` names `25`.
A string is a finite sequence of elements of `A26`, as Chapter 2 fixed
strings. English spelling, word boundaries, and letter frequencies are
not properties of `A26`. When a break uses a recognition rule or a
frequency count, that rule is an extra assumption, written in the
example, and it applies only to the strings the example names.

**Assumption N11.6 — Orange denotes the function, and the checker is not
the proof.** The listings use the forms N7 and N8 already ran: `edition`,
`module`, `spec`, `let`, `as`, arrays, literal and proved indices, `if`,
tuples, and bounded `for`. They also use the residue type `Mod[m]`, which
this compiler accepts: arithmetic on `Mod[m]` yields a representative in
`0` through `m - 1`. The version line of the binary used to run the
listings reports implemented slice S3t. That report is an identification
of a tool, as Chapter 4 required of a compiler identity. It is not an
acceptance of the modular proposal, and it adds no cryptographic claim.
A listing that the checker rejects is evidence about that source and that
checker. It is not evidence about a different source that happens to
implement a similar idea.

One boundary is easy to cross because the alphabet is small. A successful
decryption of one string is one example, in the sense of §1.5. A
universal claim needs a quantifier and a proof, or an exhaustive check
whose set is exactly the domain of the claim. The ledger at the end is
the second kind on the sets it names. It is not a silent upgrade of those
sets to every construction that has ever been called a cipher.

### N11.2 Changing the marks is a public function

Take the meeting line from §1.2 and apply a second public rule, chosen
now so that it is obviously not hiding anything. Replace each letter by
the letter three places later in the roster of Assumption N11.5, wrapping
from `Z` back to `A`, and leave the spaces where they are. The spaces are
not elements of `A26`. They are separators in the written example. The
function on letters is the thing being defined.

```text
MEET AT THE BRIDGE
PHHW DW WKH EULGJH
```

You can check one letter before trusting the line. `M` is residue `12`.
Adding three gives `15`, which the roster names `P`. `E` is `4`, and `4 + 3
= 7`, which the roster names `H`. The space is copied. The same rule
produces the rest. Predict `G` before reading on. `G` is `6`, and `6 + 3
= 9`, which the roster names `J`. The last letter of the second line is
that `J`.

The receiver who knows the rule subtracts three and wraps the other way.
`P` returns to `M`. So does the carrier, because Assumption N11.1 gives
the carrier the same rule. The second line looks less like the first.
Both readers recover the first. You have changed the appearance. You have
not created a difference between the two readers.

This is the same separation §1.2 drew with reversal. The new ingredient
is that the rule is a translation along a cycle of twenty-six, the cycle
Chapter 6 taught you to call reduction modulo `26`. Reversal needed no
modulus. The translation does. The secrecy failure is unchanged: a public
rule, fully known, is available to every reader the assumption names.

Call the translation `E3` for a moment, with the `3` written into the
name because this paragraph is not yet varying it. For every residue `x`
in `A26`,

```text
E3(x) = the representative of x + 3 in 0 through 25.
```

**Proposition N11.1 — A fixed public shift undoes itself.** For every
`x` in `A26`, subtracting three from `E3(x)`, and reducing modulo `26`,
returns `x`.

**Proof.** Let `y` be the representative of `x + 3`. By the meaning of
congruence in §6.2, `y ≡ x + 3 (mod 26)`, so `y - 3 ≡ x (mod 26)`. Both
`x` and the representative of `y - 3` lie in `0` through `25`. Uniqueness
of the representative, from §6.3, says they are the same integer.

The proof covers every residue, including the wrap. Take `x = 24`, the
letter `Y`. Then `x + 3 = 27 = 1 × 26 + 1`, so `E3(24) = 1`, the letter
`B`. Subtracting three from `1` gives `-2`, and `-2 = (-1) × 26 + 24`, so
the representative is `24`. The same arithmetic, not a second rule, covers
`Z`. One example did not carry the other. The quantifier did.

Proposition N11.1 is a fact about a function from `A26` to `A26`. It is
not a fact about the carrier. The carrier's recovery is the same function
applied by a second person who was given the rule. If you now hide the
rule, you have changed the adversary assumption in the middle of the
argument. Assumption N11.1 forbids that rescue. A construction that is
safe only against a reader who does not know the construction has not
been tested against the adversary this lesson is using.

There is a second temptation. The second line “looks encrypted.” That
sentence reports a feeling about unfamiliar letters. It does not report
a key, a distribution, or an adversary's conditional knowledge. N10
already refused to call a string random because the format allowed many
strings. The same refusal applies here. Unfamiliar appearance is not one
of the three quantities in §N10.14.

### N11.3 Encoding is a public change of representation

The roster that sends `0` to `A` and `25` to `Z` is itself a function.
So is hexadecimal notation from Chapter 2, which sends a byte to two
hexadecimal digits, and the reading that sends those digits back to the
byte. Both directions are public. Neither takes a selected secret.

**Definition N11.1 — Encoding.** An **encoding** of a set `S` into a set
`T` is a function `f` from `S` to `T`, together with a function `g` from
the image of `f` to `S`, both of them public under Assumption N11.1, such
that `g(f(s)) = s` for every `s` in `S`. The function `g` is a public
left inverse on that image, in the sense of Definition N9.11. The
encoding does not take a key.

The residue map is an encoding of the twenty-six letter names into
`A26`. Hexadecimal notation, on the set of bytes, is an encoding into the
set of two-digit hexadecimal strings. Reversal of a finite string, the
operation §1.4 specified, is an encoding of the set of finite strings
into itself: the public inverse is reversal again, which §1.5 already
proved. Proposition N11.1 says that `E3` is also an encoding of `A26`
into itself. The inverse is “subtract three and reduce.” The number
three is written in the definition. It is not a key selected from a set
and withheld. Folding it into the name `E3` is the honest syntax. A
reader who calls `E3` a cipher because the output looks scrambled has
used the appearance test that Definition N11.1 does not contain.

**Counterexample — a public shift labeled as encryption.** Suppose a
note says “`PHHW` is the encryption of `MEET`.” Under Definition N11.1
the string `PHHW` is the encoding of `MEET` by `E3`, once spaces are set
aside as separators. The note has renamed an encoding. The rename does
not add a key, and it does not change Proposition N11.1. The carrier
still subtracts three.

Encodings can lose information, and then they fail Definition N11.1
because no left inverse exists. The map that sends every letter to `A`
is public and is not an encoding: every letter lands on one name, so no
function on that image can return both `M` and `E`. Proposition N9.8
makes the same point in cardinalities. The image has one element and the
domain has twenty-six, so the map is not injective, and a non-injective
function has no left inverse. Calling that collapse an encoding would
make the word mean “any public function,” which is a different
definition. This lesson keeps Definition N11.1.

An encoding can also be awkward without failing the definition. The
residue of `M` is `12`, which is not the byte that a widespread character
table uses for the glyph `M`. Both are public functions from a letter
name to a number. They disagree because they are different functions.
Chapter 2 already separated a number from its written form. Two
encodings of one set are two functions. Agreement is a further claim,
proved or tested on the domain you name, and not a consequence of both
being called encodings.

### N11.4 Encryption is a family of maps, indexed by a key

Chapter 1 defined a key as a value shared by sender and receiver and
intended to remain unknown to the adversary. The method describes how to
use a key. The selected key determines a particular use. This section
turns that paragraph into a function family, so later constructions have
one object to be.

**Definition N11.2 — Symmetric encryption scheme.** A **symmetric
encryption scheme** is three finite sets and two functions. The sets are
a plaintext set `P`, a ciphertext set `C`, and a key set `K`. The
functions are an encryption function `E` and a decryption function `D`.
The function `E` takes a pair `(k, p)`, with `k` in `K` and `p` in `P`,
and returns an element of `C`. The function `D` takes a pair `(k, c)`,
with `k` in `K` and `c` in `C`, and returns an element of `P`. The
scheme is **correct** when

```text
∀ k ∈ K, ∀ p ∈ P, D(k, E(k, p)) = p.
```

Both functions are public, in the sense of Assumption N11.1. The selected
key is not public merely because the functions are. Correctness is a
statement about pairs `(k, p)`. It does not mention an adversary, and it
does not say that `c` conceals `p`.

The quantifiers are in the order Definition N9.17 makes you write. “For
every key, for every plaintext, decryption undoes encryption.” The
witness for an existential claim is a different statement and is not
what correctness says. One worked pair, such as `E3` on `M`, is one pair.
It does not discharge the universal quantifiers.

`E3` can be placed inside this shape, and the placement shows what the
shape does not add. Take `P = C = A26` and `K = {3}`. Define `E(3, x) =
E3(x)` and `D(3, y)` by subtracting three. Proposition N11.1 is exactly
correctness for this scheme. The key set has one element, and Assumption
N11.1 gives that element to the adversary because it is part of the
method: there is no other element the name could have meant. A correct
scheme with a public one-element key set is an encoding that has been
written with an idle key slot. Correctness holds. Concealment does not
even have a definition yet. Do not let the word “encryption” in the
scheme's name stand in for a concealment claim that the definition
postpones.

**Definition N11.3 — What the adversary is given.** Fix a correct
symmetric scheme and a distribution on `K × P`, in the sense of a finite
probability space whose outcomes are the pairs `(k, p)`. Encrypting
produces a ciphertext `c = E(k, p)`. The adversary's **view** in this
lesson is the pair consisting of that ciphertext and the public
description of `E` and `D`. The view does not include `k`, unless the
distribution or a later assumption says that `k` is a function of public
data. Conditional knowledge of an event about `p`, given an event about
`c`, is Definition N10.8 applied to this space. That is the formal
object behind “what the ciphertext reveals.” It is a calculated list of
weights. It is not a report of a person's beliefs.

This is why a correct scheme can still fail. Correctness says the
receiver, who is given `k`, recovers `p`. The view says the adversary is
given `c` and the method. Those are different inputs. Proposition 3.1
already had this shape for XOR: the holder of the mask recovers the
string, and the output alone does not determine both inputs. Definition
N11.3 is that separation, written for a scheme instead of for one
operator.

**Counterexample — correctness with a published key.** Publish `k = 3`
and encrypt `MEET` with `E3`. Correctness holds by Proposition N11.1.
The adversary's view includes the ciphertext and the method, and the
method includes the only key. Conditional knowledge of the plaintext
given the ciphertext is a one-row list: the plaintext is `MEET`. The
scheme is correct and reveals the message. A test that checks
`D(3, E(3, p)) = p` on this string, or on all twenty-six residues, passes
and still does not speak to the adversary's view. The test and the
concealment question are different subjects. §1.5 already drew that
line for reversal. The scheme vocabulary does not move it.

A scheme whose key set is large has not, by that cardinality, answered
the concealment question either. Cardinality is a count of keys.
Proposition N10.15 says a count is not a distribution and not an
adversary's uncertainty. The shift construction later in this lesson has
twenty-six keys. You will be able to try every one. The size is the
reason the trial finishes, not a reason to skip the trial.

### N11.5 A hash sends a larger set into a smaller one

Some sentences use “hash” for a short name computed from a longer
string. The job being claimed is not concealment and not recovery. It is
a summary. This lesson defines the word narrowly enough to prove one
fact, and it does not define a practical hash function. No listing below
implements one, and none is a substitute for a standard you have not
been given.

**Definition N11.4 — Hash function, in this lesson.** Let `S` and `T` be
finite sets with `|S| > |T|` and `|T| ≥ 1`. A **hash function** from `S`
to `T` is a function `H` from `S` to `T`. A **collision** is a pair of
distinct elements of `S` with the same image under `H`.

The definition requires the domain to be strictly larger than the
codomain. A bijective encoding is not a hash function under this
definition, because an encoding that meets Definition N11.1 on a finite
set of equal cardinality is bijective by Proposition N9.10, and a
bijection has no collision. The inequality on the cardinalities is the
whole distinction. It is a requirement on the sets, not a judgment about
whether the function looks one-way.

**Proposition N11.2 — A hash function in this lesson has a collision.**
Let `H` be a hash function from `S` to `T` as in Definition N11.4. Then
there exist distinct `s1, s2` in `S` with `H(s1) = H(s2)`.

**Proof.** Suppose, for a contradiction, that no such pair exists. Then
`H` is injective. Proposition N9.8 says the image of `H` has cardinality
`|S|`. The image is a subset of `T`, so `|S| = |image| ≤ |T|`. Definition
N11.4 says `|S| > |T|`. Those two comparisons of integers cannot hold
together. The supposed absence of a collision is therefore false.
Proposition N9.11 turns “not injective” into the existential claim the
proposition states: some two distinct inputs share an output.

The proof does not name the colliding pair. Existence is not a witness.
When the sets are small enough to list, a finite check can name one
pair, and that pair is a witness for those sets. The witness does not
extend the proof to a function whose domain you have not listed.

**Example N11.1 — Parity of two bits.** Let `S` be the set of two-bit
strings, so `|S| = 4` by the count in Chapter 2. Let `T = {0, 1}`. Define
`H` by `H(ab) = a ⊕ b`, the XOR of the two bits, which is the parity bit
from Chapter 3. Then `|S| > |T|`, so `H` is a hash function in the sense
of Definition N11.4. Compute the four images. `H(00) = 0`, `H(01) = 1`,
`H(10) = 1`, `H(11) = 0`. The distinct strings `00` and `11` collide, and
so do `01` and `10`. Proposition N11.2 promised at least one collision.
The table shows two pairs. The table is an exhaustive check on a
four-element set. It is the whole domain, so on this domain it is a
proof as well as a check. It is not a proof about a parity function on
longer strings. That generalization is the same argument as Proposition
N11.2 applied to a larger `S`, and it needs its own sentence if you want
it.

Parity is a useful function. It is not, under this definition, a secret.
Anyone who knows the two bits can recompute the bit. Anyone who knows
only the parity bit cannot recover the two bits, because of the
collision: the output `0` is consistent with both `00` and `11`. Failure
to recover the input is not concealment in the sense Definition N11.3
set up for encryption. There is no key. The output was not produced to
be unreadable. It was produced to be short. Those are different
specifications, and a function can meet one and fail the other.

**Counterexample — a short output with no collision on its actual
domain.** Let `S` be `{00, 01}` and let `H` be the same parity rule,
restricted to those two strings. Then `H(00) = 0` and `H(01) = 1`. The
restriction is injective. Definition N11.4 does not call it a hash
function, because `|S| = |T| = 2`. The formula “XOR the bits together”
does not decide the question. The cardinalities do. A sentence that
calls every parity computation a hash has changed the definition.

N10's birthday bound counts collisions among independent uniform draws
from one set. Proposition N11.2 is a different fact. It says a function
into a smaller set collides somewhere, with no probability and no draws.
Do not quote the birthday bound as the proof of Proposition N11.2. The
bound answers another question: how the collision probability grows when
you draw elements, under Assumption N10.3. A hash function can have a
collision that those draws never hit. Existence and a draw are different
quantifiers.

This lesson does not define preimage resistance, second-preimage
resistance, or collision resistance as security properties. Each of
those phrases would need an adversary, a distribution, and a success
condition. None of the three is supplied by Definition N11.4. Using the
phrases here would be the hopeful-set failure of Assumption N9.2: a name
without a membership rule. A later chapter that defines them has to
define them. This one will not borrow the names in advance.

### N11.6 Authentication is a check the receiver redoes

Chapter 1 separated a message the carrier cannot read from a message the
carrier cannot replace undetected. The second problem needs its own
definition. Concealment does not answer it, and a hash function as just
defined does not answer it either, unless the receiver has something the
carrier lacks.

**Definition N11.5 — Authentication check.** An **authentication check**
is a function `V` that takes a plaintext `p`, a tag `t` from a stated
tag set, and, when the construction says so, a key `k`, and returns one
of the two truth values from Chapter 3. The receiver **accepts** when
`V` returns true and **rejects** when `V` returns false. A scheme that
produces tags is **complete** on a stated set of messages when, for every
message in that set, the tag the scheme produces for that message makes
`V` accept. Completeness is the analogue of correctness. It says the
honest tag passes. It does not say that a substituted tag fails.

The tag is not the ciphertext of Definition N11.2, though a construction
may send both. The tag is an extra value whose only job in this
definition is to be recomputed or compared. A receiver who accepts every
tag has a complete check and an empty one: `V` returns true for every
pair, so every honest tag passes, and so does every substitute. The
universal claim “every forgery is rejected” is false for that `V`, by
Proposition N9.11, because there exists a forgery that is accepted.
Completeness does not include that universal claim.

**Example N11.2 — A parity tag on two bits.** Let the plaintext be a
two-bit string `ab`, and let the tag be the bit `a ⊕ b`. The check `V`
accepts when the received tag equals the parity of the received string.
Completeness holds: the sender's tag matches, by the definition of the
tag. Now suppose the carrier replaces `ab` by the other string of the
same parity, and keeps the tag. If the message was `00` and the tag was
`0`, the substitute `11` still has parity `0`. The check accepts. The
carrier did not need a key. The tag was a deterministic public function
of the message, so the carrier can compute a fresh tag for any
substitute as well. Keeping the old tag was enough for this particular
substitute; recomputing would also have worked, because the function is
public.

Read that against Definition N11.3. There is still no encryption here.
The failure is an integrity failure of the kind §1.6 named. A public tag
is an encoding of the message into the tag set, or a hash of it when the
tag set is smaller, written beside the message. Assumption N11.1 gives
that function to the carrier. The carrier who can edit the message and
the tag can produce a new pair that `V` accepts, whenever the carrier
can evaluate `V`'s public rule. A check with no secret input distinguishes
accidental damage only to the extent that the damage fails to land on
another completing pair. It does not distinguish the carrier from the
sender.

**Proposition N11.3 — A public tag function does not reject a carrier who
can recompute it.** Let `T` be a function from a plaintext set `P` to a
tag set, and let `V(p, t)` accept exactly when `t = T(p)`. Let `p` and
`p'` be elements of `P`, and set `t' = T(p')`. Then `V(p', t')` accepts.
If the carrier can replace the pair `(p, T(p))` by `(p', t')`, the
receiver accepts `p'`.

**Proof.** `t' = T(p')` is the condition `V` uses for acceptance. The
carrier's replacement is that pair. Acceptance follows from the
definition of `V`. The argument does not depend on `p'` differing from
`p`. When `p'` does differ, the accepted plaintext is not the one the
sender formed. Completeness for the sender's own messages remains true
and is not the claim that failed.

The proposition is the reason a later construction that wants the
carrier to be unable to retag a message has to give the receiver an
input the carrier does not have, or give up the goal. That input is a
key, in the role Chapter 1 gave the word, used inside `T` and `V`. This
lesson does not build such a keyed tag. Building one would be a message
authentication code, and that construction has prerequisites this lesson
has not stated: what the key distribution is, what the tag length is,
and what the forgery event is inside a probability space. Naming the gap
is the obligation. Filling it with a new primitive would skip the order
the curriculum imposed.

**Counterexample — concealment mistaken for a check.** Encrypt `MEET` by
the public shift `E3` and send `PHHW`, with no separate tag. The receiver
who subtracts three obtains `MEET` and accepts the result because
decryption succeeded. Success was guaranteed by correctness, for every
ciphertext that is a letter string: every string in `A26` of the right
length is `E3` of exactly one plaintext, by Proposition N11.1 applied to
each position. The carrier who replaces `PHHW` by `PHHH` produces a
ciphertext that decrypts to `MEEE`: `H` is residue `7`, and `7 - 3 = 4`,
which the roster names `E`. The decryption function still returns a
plaintext. Nothing in the
scheme returns false. There is no `V`. A receiver who treats “decryption
produced letters” as an authentication check has invented a `V` that
accepts every letter string. Proposition N11.3 is not even needed. The
invented check has no rejecting output.

That is the content of the title. Appearance changed from `MEET` to
`PHHW`. The change did not become a test the receiver can fail. Protecting
the message against substitution is a different requirement from changing
its appearance, and it stays different after the appearance change is
given the name encryption. The classical constructions in the next
sections are encryption schemes under Definition N11.2. They are not
authentication checks. When you break one, you are answering the
confidentiality question under a stated recognition rule. You are not
discovering that the scheme also failed to detect tampering. It was
never given that job. A receiver who needed that job was reading a
promise the definition does not contain.

### N11.7 The shift scheme

The translation `E3` used one fixed addend. A shift scheme lets the
addend be the key.

**Definition N11.6 — Shift scheme.** The **shift scheme** on `A26` is the
symmetric scheme with `P = C = A26`, `K = A26`, and

```text
E(k, x) = the representative of x + k in 0 through 25,
D(k, y) = the representative of y - k in 0 through 25.
```

Strings are encrypted position by position with the same key. For a
string `p` of length `n` and one key `k`, the ciphertext string has
length `n`, and position `i` is `E(k, p_i)`. Decryption applies `D(k, ·)`
at each position. The key is one residue, not a residue per position.
That last sentence is the whole difference between this scheme and the
one-time pad defined later. Do not smooth it over.

**Proposition N11.4 — The shift scheme is correct.** For every `k` in
`A26` and every `x` in `A26`, `D(k, E(k, x)) = x`. The same identity
holds at every position of a finite string.

**Proof.** `E(k, x) ≡ x + k (mod 26)` and
`D(k, E(k, x)) ≡ (x + k) - k = x (mod 26)`. Both `x` and the
representative of `(x + k) - k` lie in `0` through `25`. Uniqueness from
§6.3 says they are equal. A string is a sequence of residues. Apply the
one-residue fact at each position. The length is unchanged because each
position produces one residue.

The proof is Proposition N11.1 with `3` replaced by an arbitrary `k`.
The replacement is legitimate because no step used the value three. A
reader who checked only key `3` has a test, not this proof.

**Proposition N11.5 — Each key is a bijection.** For each fixed `k` in
`A26`, the map `x ↦ E(k, x)` is a bijection from `A26` to `A26`.

**Proof.** Proposition N11.4 says `D(k, ·)` is a left inverse, so the map
is injective: if `E(k, x1) = E(k, x2)`, apply `D(k, ·)` and obtain
`x1 = x2`. Domain and codomain are finite of equal cardinality `26`.
Proposition N9.10 turns injectivity into bijectivity.

So every residue is the ciphertext of exactly one plaintext, for a fixed
key. That is why “decryption produced a letter” can never fail. The
carrier’s substitute in §N11.6 was using this fact.

**Example N11.3 — HELLO under key 3.** The string `HELLO` is the residue
string `[7, 4, 11, 11, 14]`. Key `3` adds three at each position. The
sums are `10, 7, 14, 14, 17`, all already in range, and the roster names
them `KHOOR`. Predict the decryption of `K` before you look at a program.
`K` is `10`, and `10 - 3 = 7`, which is `H`. The other four letters
follow by the same subtraction.

The Orange spelling of this example is addition in the residue type
`Mod[26]`. The type is the modulus. An operation on it returns a
representative in `0` through `25`, so the reduction §6.3 named is not a
step the program can forget. Forgetting it would be a different program,
on `Int`, and the checker would be looking at that different program.

**Listing N11.1 — `shift.or`**

```orange
edition 2026;
module shift {
  type Letter = Mod[26];

  spec encrypt(x: Letter, k: Letter) -> Letter {
    x + k
  }

  spec decrypt(y: Letter, k: Letter) -> Letter {
    y - k
  }

  spec hello() -> Letter^5 {
    let k: Letter = 3;
    let p: Letter^5 = [7, 4, 11, 11, 14];
    for i in 0..5 with s: Letter^5 = p {
      s with [i] = encrypt(p[i], k)
    }
  }

  spec back() -> Letter^5 {
    let k: Letter = 3;
    let c: Letter^5 = hello();
    for i in 0..5 with s: Letter^5 = c {
      s with [i] = decrypt(c[i], k)
    }
  }

  test "shift HELLO round trip" {
    back() == [7, 4, 11, 11, 14]
  }
}
```

**Expected evaluation output:**

```text
shift::hello: Mod[26]^5 = [10, 7, 14, 14, 17]
shift::back: Mod[26]^5 = [7, 4, 11, 11, 14]
```

**Test report:**

```text
test "shift HELLO round trip" ... ok
1 test: 1 passed, 0 failed
```

Read the listing against the definition, not against the output. `Letter`
names `Mod[26]`. `encrypt` is `E` from Definition N11.6. `decrypt` is
`D`. The loop index `i` runs through `0, 1, 2, 3, 4`, which Proposition
N7.7’s style of range check accepts for an array of length `5`: each `i`
satisfies `0 ≤ i ≤ 4`. The body writes a new array. It does not overwrite
`p`. `hello` is the ciphertext of Example N11.3. `back` applies `D` and
returns the plaintext residues. The test’s `Bool` is the one equality
`back() == [7, 4, 11, 11, 14]`. A passing test means that `Bool` was true
on this run, which is N8’s reading of a passing test. It is one string
and one key. Proposition N11.4 is the reason the equality holds, and the
test is not the proof of the proposition. The test does not quantify over
keys.

**Implementation behavior, separated from the proof.** The checker
accepted Listing N11.1. Acceptance means the source met the rules the
compiler implements for this slice, including that every `Letter`
operation stays in `Mod[26]`. It does not mean the compiler proved
Proposition N11.4. The printed arrays match the hand calculation. That
match is evidence that this evaluation and the hand calculation agree on
these inputs. It is not, by itself, the universal quantifier in the
proposition.

A literal that does not fit the modulus is rejected before any value is
produced. The residue `26` is not an element of `A26`. Reducing it would
give `0`, and a language that reduced it silently would turn a mistake
about the alphabet into the letter `A`. This compiler does not.

**Listing N11.2 — `bad_letter.or`, intentionally rejected**

```orange
edition 2026;
module bad_letter {
  spec off() -> Mod[26] {
    26
  }
}
```

```sh
./compiler/target/debug/orangec check -
echo $?
```

**Diagnostic:**

```text
error[ORC0207]: literal is outside the range of `Mod[26]`
 --> <stdin>:4:5
  |
4 |     26
  |     ^^ the literal's magnitude is not less than the modulus
  = note: a literal of `Mod[m]` has a magnitude n less than m, and `-n` stands for m - n; residues do not reduce out-of-range literals
```

The status is `1`. Standard output is empty. The code is `ORC0207`. The
locus is the literal `26`. The label says the magnitude is not less than
the modulus. The note distinguishes two facts you already have from
§6.3 and from this type. The integer `26` is congruent to `0` modulo
`26`, and the representative of that class is `0`. The literal rule does
not apply that reduction for you. A residue literal has to be written as
a magnitude strictly less than the modulus, or as the negative of such a
magnitude, in which case `-n` denotes `m - n`. The integer `26` is
neither. If you meant the letter `A`, the source that says so is the
literal `0`. If you meant an integer that you will reduce yourself, the
source that says so uses `Int` and the remainder operation, which is a
different type and a different function. The diagnostic is the checker
refusing to choose between those two readings.

Eval of the same source prints the same diagnostic and no value. No step
runs. There is no residue `0` waiting on the other side of the rejection.

### N11.8 Breaking a shift by trying the keys

**Assumption N11.7 — A stated recognition rule.** A break in this lesson
that “recovers the plaintext” uses a recognition rule written in the
example. The rule says yes or no for each candidate string. It is the
recognition clause of Assumption N10.4, specialized to the candidates
the example generates. A rule that is not written down is not in force.
In particular, “it looks like English” is not a rule until the example
says which strings count as a yes.

**Example N11.4 — One rule that isolates a key.** The ciphertext is
`KHOOR`, residues `[10, 7, 14, 14, 17]`. The recognition rule is: accept
a candidate exactly when it equals `HELLO`. The key set is `A26`. Try
`D(k, ·)` for every `k`.

You can compute the twenty-six strings yourself before trusting the
listing. Key `0` leaves the ciphertext unchanged, because adding zero
does nothing: `KHOOR`. Key `1` subtracts one: `J`, `G`, `N`, `N`, `Q`.
Key `3` subtracts three and produces `HELLO`, which the rule accepts.
The other keys are in the listing’s table, one row per key, in order
from `0` through `25`.

**Listing N11.3 — `shift_break.or`**

```orange
edition 2026;
module shift_break {
  type Letter = Mod[26];
  type Row = Letter^5;

  spec decrypt(y: Letter, k: Letter) -> Letter {
    y - k
  }

  spec candidates() -> Row^26 {
    let c: Letter^5 = [10, 7, 14, 14, 17];
    for k in 0..26 with rows: Row^26 = [[0; 5]; 26] {
      let key: Letter = k as Letter;
      rows with [k] = [
        decrypt(c[0], key),
        decrypt(c[1], key),
        decrypt(c[2], key),
        decrypt(c[3], key),
        decrypt(c[4], key)
      ]
    }
  }
}
```

**Expected evaluation output:**

```text
shift_break::candidates: (Mod[26]^5)^26 = [[10, 7, 14, 14, 17], [9, 6, 13, 13, 16], [8, 5, 12, 12, 15], [7, 4, 11, 11, 14], [6, 3, 10, 10, 13], [5, 2, 9, 9, 12], [4, 1, 8, 8, 11], [3, 0, 7, 7, 10], [2, 25, 6, 6, 9], [1, 24, 5, 5, 8], [0, 23, 4, 4, 7], [25, 22, 3, 3, 6], [24, 21, 2, 2, 5], [23, 20, 1, 1, 4], [22, 19, 0, 0, 3], [21, 18, 25, 25, 2], [20, 17, 24, 24, 1], [19, 16, 23, 23, 0], [18, 15, 22, 22, 25], [17, 14, 21, 21, 24], [16, 13, 20, 20, 23], [15, 12, 19, 19, 22], [14, 11, 18, 18, 21], [13, 10, 17, 17, 20], [12, 9, 16, 16, 19], [11, 8, 15, 15, 18]]
```

The printed type is `(Mod[26]^5)^26`. The source spelled the row as
`Row`. Evaluation prints the type the alias names. Row `k` is
`D(k, KHOOR)`. Row `3` is `[7, 4, 11, 11, 14]`, which is `HELLO`, the
unique row the recognition rule accepts. The loop bound `0..26` and the
conversion `k as Letter` are checked together: `k` takes only the
integers `0` through `25`, and each of those is a legal literal of
`Mod[26]`. The fill `[[0; 5]; 26]` is the starting table of zero rows.
The loop replaces row `k`. It does not read a row it has not yet
replaced, so the zeros are only a starting value, not a candidate the
procedure returns.

**Proposition N11.6 — Under this rule the key is determined.** In
Example N11.4, exactly one key in `A26` produces a candidate the
recognition rule accepts, and that key is `3`.

**Proof.** Listing N11.3, or the same twenty-six subtractions done by
hand, lists every candidate. Compare each row with `[7, 4, 11, 11, 14]`.
Row `3` matches. Suppose another key `k` also matches. Then
`D(k, c) = D(3, c)` at every position, so `c_i - k ≡ c_i - 3 (mod 26)`,
hence `k ≡ 3 (mod 26)`. Both keys lie in `A26`, so `k = 3`.

The proof uses the list. The list is finite and complete because `K` is
`A26` and the loop, or the hand enumeration, covers every element once.
This is an exhaustive argument of the kind §N9.22 describes, on a set of
twenty-six keys. It is not a proof about a different recognition rule.

**Counterexample — the same ciphertext, a rule that does not isolate.**
Replace the recognition rule by: accept every string of five residues.
Every row passes. There are twenty-six accepted plaintexts, one per key.
Assumption N10.4’s recognition clause is what failed, not the arithmetic.
The ciphertext still determines a set of twenty-six pairs `(k, p)` with
`E(k, p)` equal to the ciphertext. Without a rule that rejects twenty-five
of the plaintexts, the set is the answer. Calling one of them “the”
plaintext adds a sentence the problem did not contain.

**Counterexample — a rule that accepts a wrong key.** Let the rule
accept a candidate when its first residue is `10`. Row `0` is accepted.
The plaintext of row `0` is the ciphertext itself, and the key is `0`.
If the sender’s key was `3`, this rule accepts a key the sender did not
use. The break recovered what the rule asked for. It did not recover the
sender’s key. A recognition rule is an assumption about the plaintext,
and a false assumption yields a false identification. The arithmetic
does not notice.

**What the break does not say.** It does not say that twenty-six is a
safe or an unsafe size in general. It says that on this scheme, with
this ciphertext, under Assumption N11.7 with the rule “equals `HELLO`,”
one key survives and the trial count is `26`. Proposition N10.16 would
give expected trial number `(26+1)/2 = 27/2` if the key were uniform on
`A26` and the trials ran in a fixed order until the accepting row. That
expectation is an average of positions. It is not a promise about this
one ciphertext, whose accepting row sits at index `3`. The worst-case
count on a complete pass is still `26`. Shannon’s remark, quoted in N10,
that the amount of work varies by system, is illustrated here by a small
number. The illustration is not a computed work characteristic for any
other system.

Key `0` is a lawful key. It encrypts every plaintext to itself.
Correctness holds and concealment fails for a reason you can point at:
the ciphertext equals the plaintext, so the adversary’s view contains
the message. A uniform distribution on `A26` gives this key weight
`1/26`. The existence of a useless key does not by itself make the
scheme fail on other keys. It does forbid any sentence that says every
key conceals. The quantifier would be false, and Proposition N9.11 says
the negation is “some key does not conceal.” Key `0` is the witness.

### N11.9 The affine scheme

A shift adds a key. An affine scheme multiplies by a key and then adds
another. Multiplication modulo `26` is not like addition. Some
multipliers have no inverse, and a scheme that uses one of them is not
correct.

**Definition N11.7 — Common divisor.** Let `m` be a positive integer and
let `a` be an integer. A positive integer `d` **divides** `a` when
`a = d · q` for some integer `q`. The same `d` is a **common divisor** of
`a` and `m` when it divides both.

**Definition N11.8 — Greatest common divisor.** The **greatest common
divisor** `gcd(a, m)` is the greatest positive common divisor of `a` and
`m`. It exists: `1` is a common divisor, and every positive divisor of
`m` is at most `m`, so the set of positive common divisors is a nonempty
finite set of positive integers and has a greatest element.

**Proposition N11.7 — The remainder has the same common divisors.** Let
`a = q · m + r` with `0 ≤ r < m`, as §6.3 provides. A positive integer
divides both `a` and `m` exactly when it divides both `m` and `r`. In
particular, `gcd(a, m) = gcd(m, r)` when `r > 0`, and `gcd(a, m) = m`
when `r = 0`.

**Proof.** Suppose `d` divides `a` and `m`. Then `a = d · s` and
`m = d · t` for integers `s` and `t`, so
`r = a - q · m = d · (s - q · t)`. Thus `d` divides `r`, and it already
divides `m`. Suppose instead `d` divides `m` and `r`. Then
`a = q · m + r` is a sum of multiples of `d`, so `d` divides `a`. The two
sets of common divisors are equal, so their greatest elements are equal
when `r > 0`. When `r = 0`, `m` divides `a`, every common divisor divides
`m`, and `m` itself is a common divisor. The greatest is `m`.

**Proposition N11.8 — Bézout’s identity.** For every integer `a` and
every positive integer `m`, there exist integers `x` and `y` such that

```text
a · x + m · y = gcd(a, m).
```

**Proof.** Induct on the positive integer `m`, using Assumption N9.4.
If `m = 1`, then `gcd(a, 1) = 1`, because `1` divides `a` and no positive
divisor is greater than `1`. Take `x = 0` and `y = 1`.

Fix `m > 1`, and assume the claim is already proved for every positive
modulus strictly less than `m`, and for every integer in the role of `a`.
Write `a = q · m + r` with `0 ≤ r < m`. If `r = 0`, Proposition N11.7
says `gcd(a, m) = m`. Take `x = 0` and `y = 1`. If `r > 0`, then `r` is
a positive integer less than `m`, so the inductive hypothesis applies to
the pair whose integer is `m` and whose modulus is `r`: there exist
integers `x'` and `y'` with

```text
m · x' + r · y' = gcd(m, r).
```

Proposition N11.7 says `gcd(m, r) = gcd(a, m)`. Substitute
`r = a - q · m`:

```text
m · x' + (a - q · m) · y' = a · y' + m · (x' - q · y').
```

Take `x = y'` and `y = x' - q · y'`. The linear combination equals
`gcd(a, m)`.

The induction is on the modulus, not on `a`. The inductive step calls
the claim for a smaller positive modulus, which is the remainder. Each
remainder is a nonnegative integer strictly below the previous modulus,
so the descent cannot continue forever. That is what the induction is
for. You do not need a separate termination argument beside it.

**Proposition N11.9 — Invertible multipliers are the multipliers coprime
to the modulus.** Let `m` be positive and let `a` be an integer. There
exists an integer `x` with `a · x ≡ 1 (mod m)` if and only if
`gcd(a, m) = 1`. When such an `x` exists, its representative in `0`
through `m - 1` is unique.

**Proof.** Suppose `a · x = 1 + m · t` for some integer `t`. Then
`a · x + m · (-t) = 1`. Any common divisor of `a` and `m` divides the
left side and therefore divides `1`. The only positive divisor of `1` is
`1`, so `gcd(a, m) = 1`.

Suppose conversely `gcd(a, m) = 1`. Proposition N11.8 supplies integers
`x` and `y` with `a · x + m · y = 1`. Then `a · x ≡ 1 (mod m)`.

For uniqueness of the representative, suppose `a · x1 ≡ 1` and
`a · x2 ≡ 1 (mod m)`. Then `a · (x1 - x2) ≡ 0 (mod m)`, so `m` divides
`a · (x1 - x2)`. From the previous paragraph, `a · x + m · y = 1` for
some integers `x` and `y`. Therefore

```text
x1 - x2 = (a · x + m · y) · (x1 - x2)
        = x · (a · (x1 - x2)) + m · (y · (x1 - x2)).
```

The product `a · (x1 - x2)` is a multiple of `m`, so the first summand
is a multiple of `m`. The second summand is a multiple of `m` as
written. Hence `m` divides `x1 - x2`. Two representatives in `0` through
`m - 1` that differ by a multiple of `m` are equal, by the uniqueness
argument in §6.3.

The representative is called the **inverse** of `a` modulo `m`. It is an
integer in `0` through `m - 1`, not a second kind of number.

**Example N11.5 — The multipliers modulo 26.** A residue `a` in `A26`
has an inverse exactly when `gcd(a, 26) = 1`. Since `26 = 2 · 13`, a residue shares a factor greater than `1` with
`26` exactly when it is even or it is `13`. The multiples of `13` in
`A26` are `0` and `13`, and `0` is already even, so the extra odd
residue is `13` alone. The residues that remain are the odds other than
`13`:

```text
1, 3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25.
```

There are twelve of them. For a witness, `gcd(5, 26)`: `26 = 5 · 5 + 1`,
and `5 = 1 · 5 + 0`, so the positive remainder `1` is the greatest common
divisor, by two uses of Proposition N11.7. Bézout from that division:
`1 = 26 - 5 · 5`, so `5 · (-5) + 26 · 1 = 1`. The representative of `-5`
modulo `26` is `21`, because `-5 = (-1) · 26 + 21`. And `5 · 21 = 105`,
`105 = 4 · 26 + 1`, so `5 · 21 ≡ 1 (mod 26)`. The inverse of `5` is `21`.

For a failure, `gcd(2, 26) = 2`, not `1`. There is no inverse. The same
conclusion from parity, without quoting the gcd: `2 · x` is even, `26 · t`
is even, and `1` is odd, so `2 · x = 1 + 26 · t` cannot hold for any
integers `x` and `t`.

**Definition N11.9 — Affine scheme.** The **affine scheme** on `A26` has
plaintext set and ciphertext set `A26`. A key is a pair `(a, b)` with
`a` one of the twelve residues in Example N11.5 and `b` any residue in
`A26`. Thus `|K| = 12 · 26 = 312`. Encryption and decryption are

```text
E((a, b), x) = the representative of a · x + b,
D((a, b), y) = the representative of a_inv · (y - b),
```

where `a_inv` is the inverse of `a` modulo `26` from Proposition N11.9.

**Proposition N11.10 — The affine scheme is correct.** For every key
`(a, b)` and every `x` in `A26`, `D((a, b), E((a, b), x)) = x`.

**Proof.** Let `y ≡ a · x + b (mod 26)`. Then
`y - b ≡ a · x (mod 26)`, and multiplying by `a_inv` gives
`a_inv · (y - b) ≡ a_inv · a · x ≡ 1 · x ≡ x (mod 26)`, where the middle
step is Proposition N11.9. Representatives in `A26` agree.

The shift scheme sits inside this one. The keys `(1, b)` are the shifts
by `b`, because multiplying by `1` leaves `x` unchanged. Key `(1, 3)` is
Example N11.3. The affine scheme is a larger family, not a different
alphabet.

**Example N11.6 — One letter under `(5, 8)`.** Take `x = 7`, the letter
`H`. Then `5 · 7 + 8 = 43`, and `43 = 1 · 26 + 17`, so the ciphertext
residue is `17`, the letter `R`. Decrypt: `17 - 8 = 9`, and
`21 · 9 = 189`, `189 = 7 · 26 + 7` because `7 · 26 = 182` and
`189 - 182 = 7`. The plaintext residue returns. A second letter, `E`,
is residue `4`. `5 · 4 + 8 = 28 ≡ 2`, the letter `C`.

**Listing N11.4 — `affine.or`**

```orange
edition 2026;
module affine {
  type Letter = Mod[26];

  spec encrypt(x: Letter) -> Letter {
    let a: Letter = 5;
    let b: Letter = 8;
    (a * x) + b
  }

  spec decrypt(y: Letter) -> Letter {
    let ainv: Letter = 21;
    let b: Letter = 8;
    ainv * (y - b)
  }

  spec round() -> Letter {
    decrypt(encrypt(7))
  }

  spec image() -> Letter^2 {
    [encrypt(7), encrypt(4)]
  }

  spec collide() -> Letter^2 {
    let a: Letter = 2;
    [a * 0, a * 13]
  }

  test "affine round trip at 7" {
    round() == 7
  }
}
```

**Expected evaluation output:**

```text
affine::round: Mod[26] = 7
affine::image: Mod[26]^2 = [17, 2]
affine::collide: Mod[26]^2 = [0, 0]
```

**Test report:**

```text
test "affine round trip at 7" ... ok
1 test: 1 passed, 0 failed
```

The parentheses in `(a * x) + b` write the order Definition N11.9 uses.
The two operators are the residue operations of one modulus. `round` is
Example N11.6’s decryption of `H`. `image` is the pair `(R, C)` as
residues `(17, 2)`. The test checks one plaintext, `7`. Proposition
N11.10 is the universal claim. The test is one pair.

`collide` is not a decryption. It multiplies by `2`, which Example N11.5
excluded from the key set. Both `0` and `13` land on `0`:
`2 · 13 = 26 ≡ 0`. The two plaintexts are distinct and the images are
not. The map `x ↦ 2 · x` on `A26` is not injective, so it has no left
inverse, and no function `D` can make the scheme correct for `a = 2`.
Definition N11.9 keeps `a = 2` out of `K` for that reason. A program can
still write the multiplication. The type accepts it because
multiplication is defined on every pair of residues, not only on the
invertible ones. The checker has not been asked to prove Proposition
N11.9.

That last gap has a sharp edge in the operator `/`. On `Int`, `/` and
`%` are the Euclidean quotient and remainder of §6.3, with the total
convention the language states for a zero divisor. On `Mod[m]`, `/` is a
different function: it multiplies by the modular inverse when Proposition
N11.9 supplies one, and it returns `0` when the multiplier has no
inverse. The symbol looks like division. The two types do not share a
meaning for it.

**Listing N11.5 — `residue_div.or`**

```orange
edition 2026;
module residue_div {
  spec missing() -> Mod[26] {
    2 / 2
  }

  spec present() -> Mod[26] {
    1 / 5
  }
}
```

**Expected evaluation output:**

```text
residue_div::missing: Mod[26] = 0
residue_div::present: Mod[26] = 21
```

Predict `2 / 2` as if it were an integer quotient, and you will predict
`1`. The printed value is `0`. The multiplier `2` has no inverse modulo
`26`, so the residue operation returns `0`. That `0` is not the quotient
of `2` by `2`, and it is not an inverse. `1 / 5` prints `21`, which is
the inverse computed in Example N11.5, because `5 · 21 ≡ 1` and
multiplying `1` by that inverse leaves `21`. A decryption written `y / a`
typechecks for `a = 2` and denotes the constant `0` on every ciphertext,
since every residue divided by `2` in this type is `0`. Correctness fails
openly: the plaintext is not recovered. The file is still accepted.
Acceptance is a fact about defined operators. It is not a certificate
that `a` was coprime to `26`.

**Example N11.7 — Two known pairs determine this key.** Suppose you are
given that some affine key sent `7` to `17` and `4` to `2`, and you are
not given `a` or `b`. Subtract the congruences:

```text
17 - 2 ≡ a · (7 - 4) (mod 26),
15 ≡ a · 3 (mod 26).
```

The difference `3` is on the list in Example N11.5, so it has an inverse.
`3 · 9 = 27 ≡ 1`, so the inverse is `9`. Multiply both sides by `9`:
`a ≡ 15 · 9 = 135 ≡ 5 (mod 26)`, because `135 - 5 · 26 = 135 - 130 = 5`.
Then `b ≡ 17 - 5 · 7 = 17 - 35 = -18 ≡ 8 (mod 26)`, because
`-18 + 26 = 8`. The key is `(5, 8)`, the key Listing N11.4 used. Two
plaintext-ciphertext pairs, with an invertible difference of plaintexts,
replaced a search over `312` keys by arithmetic.

**Counterexample — a difference that is not invertible.** Suppose the
known plaintexts differ by `2`, and the ciphertexts differ by `4`. The
congruence `4 ≡ a · 2 (mod 26)` has solutions. Try `a = 2`:
`2 · 2 = 4`. Try `a = 15`: `15 · 2 = 30 ≡ 4`. Both `2` and `15` satisfy
the difference, and they are not congruent modulo `26`. Even after you
restrict `a` to the twelve invertible residues, more than one can remain;
here `15` is invertible and `2` is not, so the invertible case of this
particular congruence may isolate `15` once Definition N11.9’s restriction
is in force. Change the numbers to a difference of `13`. The congruence
`0 ≡ a · 13 (mod 26)` holds for every even `a`, because `a · 13` is then
a multiple of `26`. Among the odd invertible residues, `a · 13 ≡ 13` when
`a` is odd, since `a = 2t + 1` gives `a · 13 = 26t + 13 ≡ 13`, not `0`.
So no invertible `a` satisfies `a · 13 ≡ 0 (mod 26)`. A pair of known
letters whose residues differ by `13` cannot occur under two different
plaintexts for an invertible multiplier, unless the ciphertext difference
is `13` rather than `0`. The point of the counterexample is the first
one: without the coprime restriction, `4 ≡ a · 2` does not determine
`a`. Known plaintext is not a spell that ignores Proposition N11.9.

**Proposition N11.11 — The key count is three hundred twelve.** The set
of pairs `(a, b)` in Definition N11.9 has `312` elements.

**Proof.** Example N11.5 lists twelve allowed values of `a`. For each of
them, `b` runs through `26` residues. The pairs are distinct when either
component differs. The product of the counts is `312`.

Under Assumption N11.7, an exhaustive break tries these `312` keys, not
`26 · 26 = 676`. The extra `364` pairs are the ones Definition N11.9
excluded because correctness fails. Trying them is a search of a larger
set than the scheme’s key set. If your recognition rule accepts a
candidate that only a non-invertible multiplier produces, you have left
the scheme. Say so. If the rule accepts nothing outside the `312`, the
worst-case trial count under Assumption N10.4 is `312`, and the expected
position for a uniform key is `(312 + 1)/2 = 313/2`. Neither number is
the integer `26` from the shift scheme, and neither is `26!`, which has
not been given a job yet. Proposition N10.15 is in force: the length of
a key written as two residues is not the size of this distribution. Two
residues can name `676` pairs. This distribution uses `312` of them.
