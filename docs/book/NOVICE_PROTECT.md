# The Orange Book

By Chase Bryan

## Part 1, The Novice

N11: Protect More Than Appearance. Draft 2026-10-05.

Continue from [Count What You Do Not Know](NOVICE_PROBABILITY.md#n10-count-what-you-do-not-know).
This lesson is **N11**. The locked label is N11. It is not a manuscript
chapter numeral. The original manuscript keeps its own chapter numbers.
N11 is not manuscript Chapter 11, *Standards as Versioned Inputs*, and it
is not N7, N8, N9, or N10.

You already have, from
[Chapter 1, §1.2](NOVICE_OPENING.md#12-changing-the-appearance), a public
reversal that changes how a message looks and gives the receiver no
advantage over a carrier who knows the method. You have, from §1.3, the
separation of a public method and a selected key, and from §1.6 the
separation of confidentiality, integrity, and authenticity. Chapter 3 gives
XOR and its cancellation identity. Chapter 6 gives congruence and the unique
remainder. N9 gives sets, functions, inverses, quantifiers, and
Assumption N9.4. N10 gives finite probability, conditional probability,
and independence. This lesson uses those results. It does not teach them
again.

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
N9.18. Negating a quantifier is Proposition N9.11. Induction by a
successor step is Assumption N9.4: the base `P(0)`, and the step
`P(n) ⇒ P(n + 1)` with hypothesis `P(n)` alone. When that step is proved
for an arbitrary nonnegative integer and the proof never depends on a
particular bound, the second paragraph of Assumption N9.4 gives `P(n)`
for every nonnegative integer `n`. When this lesson says “every letter,”
the set of letters has already been named.

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

**Proof.** Let `Q(n)` say, for a nonnegative integer `n`: for every
integer `a` and every positive integer `m` with `m ≤ n + 1`, there exist
integers `x` and `y` such that `a · x + m · y = gcd(a, m)`.

Base case. `Q(0)` asks only for the modulus `m = 1`. Then
`gcd(a, 1) = 1`, because `1` divides `a` and no positive divisor is
greater than `1`. Take `x = 0` and `y = 1`. So `Q(0)`.

Inductive step. Fix a nonnegative integer `n` and assume `Q(n)`. Let
`S` be the set of pairs `(a, m)` in which `a` is an integer and `m` is
a positive integer with `m ≤ n + 2`. The statement `Q(n + 1)` says that
every pair in `S` has integers `x` and `y` with
`a · x + m · y = gcd(a, m)`.

Let `S1` be the pairs in `S` with `m ≤ n + 1`, and let `S2` be the
pairs in `S` with `m = n + 2`. Every pair in `S` lies in exactly one of
`S1` and `S2`.

On `S1`, the hypothesis `Q(n)` supplies `x` and `y`.

On `S2`, take a pair `(a, m)`. Write `a = q · m + r` with `0 ≤ r < m`,
as §6.3 provides.
Exactly one of `r = 0` and `r > 0` holds. If `r = 0`, Proposition N11.7
says `gcd(a, m) = m`. Take `x = 0` and `y = 1`. If `r > 0`, then `r`
is a positive integer and `r ≤ m - 1 = n + 1`. The hypothesis `Q(n)`
applies to the integer `m` and the modulus `r`: there exist integers
`x'` and `y'` with

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

Both subsets satisfy the claim, so Definition N9.18 gives it on `S`.
Definition N9.17 gives `Q(n + 1)`. The step is `Q(n) ⇒ Q(n + 1)`. The
hypothesis was `Q(n)` only. The step did not assume `Q(k)` for any `k`
other than `n`, and it did not use a particular bound. The second
paragraph of Assumption N9.4 therefore yields `Q(n)` for every
nonnegative integer `n`.

Given a positive integer `m` and an integer `a`, set `n = m - 1`. Then
`n` is a nonnegative integer and `m ≤ n + 1`, so `Q(n)` supplies the
integers `x` and `y` the proposition asks for.

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
divisor, by two uses of Proposition N11.7. The step of Proposition N11.8
substitutes `1 = 26 - 5 · 5`, so `5 · (-5) + 26 · 1 = 1`. The
representative of `-5` modulo `26` is `21`, because
`-5 = (-1) · 26 + 21`. And `5 · 21 = 105`,
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
congruence `4 ≡ a · 2 (mod 26)` holds for `a = 2`, because `2 · 2 = 4`,
and for `a = 15`, because `15 · 2 = 30 ≡ 4`. Those residues are not
congruent modulo `26`. Only `15` is on the invertible list, so the
coprime restriction of Definition N11.9 leaves one multiplier here. The
restriction does not always leave one. Take plaintext difference `13`
and ciphertext difference `13`. For every odd `a`, write `a = 2t + 1`.
Then `a · 13 = 26t + 13 ≡ 13 (mod 26)`. Every invertible residue is odd
and is not `13`, and `13` itself is not invertible, so all twelve
allowed multipliers satisfy the difference. Each then determines a `b`
from one plaintext-ciphertext pair, and those twelve keys are distinct.
Two letters are not, by themselves, a determination of the affine key.
The difference of the plaintexts has to be invertible before the
multiplication in Example N11.7 has a unique `a`.

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

### N11.10 A substitution is any permutation

A shift is one family of permutations of `A26`. An affine key is a larger
family. A substitution allows every permutation. The cost of that
generality is a key you can no longer write as one or two residues, and
the gain is not secrecy against the attack this section carries out.

**Definition N11.10 — Substitution scheme.** A **permutation** of `A26`
is a bijection from `A26` to `A26`. The **substitution scheme** has
plaintext set and ciphertext set `A26`, and its key set is the set of all
permutations of `A26`. For a key `π`,

```text
E(π, x) = π(x),
D(π, y) = π_inv(y),
```

where `π_inv` is the inverse function, which exists because `π` is
bijective. On a string, apply `π` at each position, using the same
permutation throughout.

**Proposition N11.12 — The substitution scheme is correct.** For every
permutation `π` and every `x` in `A26`, `π_inv(π(x)) = x`.

**Proof.** That equation is the left-inverse half of bijectivity,
Definition N9.14. It holds for every `x` in the domain, which is `A26`.

**Proposition N11.13 — The key set has `26!` elements.** The number of
permutations of a finite set of `n` elements is `n!`, the product
`n · (n - 1) · ... · 1`, with `0!` and `1!` both equal to `1`. In
particular the substitution key set has

```text
26! = 403291461126605635584000000
```

elements.

**Proof.** Count the permutations of a set of `n` elements by choosing
images in order. The image of a first chosen element may be any of the
`n` elements. The image of a second may be any of the remaining `n - 1`,
and so on, down to one choice for the last element. Every permutation
arises once, because a permutation is determined by those images and the
choices were forced to be distinct. The product is `n!`. For `n = 26`,
compute the product. The reference ledger recomputes it by multiplying
the integers from `1` through `26`. That multiplication is the proof’s
last step written as a finite check. It is not a sampling of keys.

The shift keys are `26` of these permutations, one for each addend. The
affine keys are `312` of them. Both numbers are counts of subsets of the
same key set. A sentence that says the substitution scheme is stronger
than a shift because `26!` is larger than `26` has compared cardinalities
and stopped. Proposition N10.15 already forbids that stop. The attack
below does not try the keys one by one. A larger set of keys changes the
exhaustive-trial count and need not change the attack you actually have.

**Example N11.8 — A keyword permutation.** Write the cipher alphabet
`CIPHERABDFGJKLMNOQSTUVWXYZ`. It has twenty-six letters and no repeats:
the keyword `CIPHER`, then the unused letters of the roster in order.
Number positions from `0`. Plaintext residue `i` maps to the residue of
the letter in position `i` of that alphabet. The images of `0` through
`25` are

```text
2, 8, 15, 7, 4, 17, 0, 1, 3, 5, 6, 9, 10, 11, 12, 13, 14, 16, 18, 19, 20, 21, 22, 23, 24, 25.
```

This is not an affine map. An affine map with `π(0) = 2` has addend `2`,
and `π(1) = 8` would force multiplier `6`. Then `π(2)` would be
`6 · 2 + 2 = 14`. The table’s third entry, the image of `2`, is `15`,
not `14`. One disagreement removes the map from the affine family.
Proposition N11.12 still applies, because the table is a permutation
whether or not it is affine. The inverse table, sending each image back
to its input, begins `6, 7, 0` because `0` is the image of `6` (the
letter `A` sits in position `6` of the cipher alphabet), `1` is the image
of `7`, and `2` is the image of `0`.

`HELLO` is `[7, 4, 11, 11, 14]`. The table sends `7` to `1`, `4` to `4`,
`11` to `9`, and `14` to `12`. The ciphertext residues are
`[1, 4, 9, 9, 12]`, the letters `BEJJM`. Decrypt by the inverse table:
position `1` holds `7`, position `4` holds `4`, position `9` holds `11`,
position `12` holds `14`. The plaintext returns.

**Listing N11.6 — `substitution.or`**

```orange
edition 2026;
module substitution {
  type Letter = Mod[26];

  spec encrypt(p: Letter^5) -> Letter^5 {
    let table: Letter^26 = [2, 8, 15, 7, 4, 17, 0, 1, 3, 5, 6, 9, 10, 11, 12, 13, 14, 16, 18, 19, 20, 21, 22, 23, 24, 25];
    for i in 0..5 with s: Letter^5 = p {
      s with [i] = table[p[i] as Int]
    }
  }

  spec decrypt(c: Letter^5) -> Letter^5 {
    let inv: Letter^26 = [6, 7, 0, 8, 4, 9, 10, 3, 1, 11, 12, 13, 14, 15, 16, 2, 17, 5, 18, 19, 20, 21, 22, 23, 24, 25];
    for i in 0..5 with s: Letter^5 = c {
      s with [i] = inv[c[i] as Int]
    }
  }

  spec hello() -> Letter^5 {
    encrypt([7, 4, 11, 11, 14])
  }

  spec back() -> Letter^5 {
    decrypt(hello())
  }

  test "keyword substitution round trip" {
    back() == [7, 4, 11, 11, 14]
  }
}
```

**Expected evaluation output:**

```text
substitution::hello: Mod[26]^5 = [1, 4, 9, 9, 12]
substitution::back: Mod[26]^5 = [7, 4, 11, 11, 14]
```

**Test report:**

```text
test "keyword substitution round trip" ... ok
1 test: 1 passed, 0 failed
```

The index `p[i] as Int` is legal because a value of `Mod[26]` converts
to an `Int` in `0` through `25`, and the table has length `26`. The
checker proves that range. It does not prove that `table` is a
permutation. A table with two equal entries would still typecheck, and
`decrypt` as written would not be its inverse. The test’s true `Bool`
says that this inverse table undoes this forward table on `HELLO`. It
does not say that every array of twenty-six residues is a key of
Definition N11.10.

### N11.11 Breaking a substitution from counts

Trying `26!` keys is an exhaustive procedure under Assumption N10.4. It
is not the procedure this section uses. The procedure uses the fact that
a substitution applies one permutation at every position, so the number
of times a ciphertext symbol appears equals the number of times its
plaintext symbol appeared.

**Example N11.9 — Five symbols, distinct counts.** Shrink the alphabet
to `A5 = {0, 1, 2, 3, 4}` so the whole attack fits on one page. The same
definitions, with `5` in place of `26`, give a substitution scheme on
`A5`. The key in this example is the permutation `π` with

```text
π(0) = 2, π(1) = 0, π(2) = 4, π(3) = 1, π(4) = 3.
```

The plaintext is the length-`20` string that consists of `0` eight times,
then `1` five times, then `2` four times, then `3` twice, then `4` once.
The counts are `8, 5, 4, 2, 1`. They are pairwise distinct and sum to
`20`.

**Assumption N11.8 — Known counts, unknown symbols.** The adversary is
given the ciphertext string and the list of plaintext counts
`8, 5, 4, 2, 1` in that order of plaintext symbols `0` through `4`. The
adversary is not told which ciphertext symbol carries which count. The
recognition rule matches the ciphertext symbol of count `8` to plaintext
symbol `0`, the symbol of count `5` to plaintext symbol `1`, and so on.
The rule needs the counts to be pairwise distinct. A tie is a different
example, given after the listing.

**Listing N11.7 — `small_sub.or`**

```orange
edition 2026;
module small_sub {
  type Sym = Mod[5];

  spec encrypt() -> Sym^20 {
    let table: Sym^5 = [2, 0, 4, 1, 3];
    let p: Sym^20 = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 4];
    for i in 0..20 with s: Sym^20 = p {
      s with [i] = table[p[i] as Int]
    }
  }

  spec counts() -> Int^5 {
    let c: Sym^20 = encrypt();
    for i in 0..20 with n: Int^5 = [0; 5] {
      n with [c[i] as Int] = n[c[i] as Int] + 1
    }
  }
}
```

**Expected evaluation output:**

```text
small_sub::encrypt: Mod[5]^20 = [2, 2, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 4, 4, 4, 4, 1, 1, 3]
small_sub::counts: Int^5 = [5, 2, 8, 1, 4]
```

Read `counts` before you reconstruct `π`. Index `0` holds `5`, index `1`
holds `2`, index `2` holds `8`, index `3` holds `1`, index `4` holds `4`.
Ciphertext symbol `2` appears eight times, symbol `0` appears five times,
symbol `4` appears four times, symbol `1` appears twice, and symbol `3`
appears once. Assumption N11.8 matches them in that order to plaintext
symbols `0, 1, 2, 3, 4`. The recovered map on ciphertext symbols is

```text
2 ↦ 0, 0 ↦ 1, 4 ↦ 2, 1 ↦ 3, 3 ↦ 4,
```

which is `π_inv`. Applying it to the ciphertext returns the plaintext
string of the listing. You did not try `5! = 120` keys. You sorted five
counts. The factorial is the size of the key set. The sort is a different
procedure, and on this distribution it determines the key.

**Proposition N11.14 — Distinct counts determine the permutation.**
Suppose a substitution on a finite alphabet is applied to a string, and
the plaintext symbols have pairwise distinct positive counts. An
adversary who is given the ciphertext and those counts, and who matches
equal counts, recovers `π_inv` and therefore the plaintext.

**Proof.** Let symbol `s` occur `n_s` times in the plaintext, with
`n_s ≠ n_t` whenever `s ≠ t`. In the ciphertext, `π(s)` occurs `n_s`
times, because each occurrence of `s` contributes one occurrence of
`π(s)` and no other plaintext symbol does. The matching rule sends the
unique ciphertext symbol of count `n_s` back to `s`. That symbol is
`π(s)`, so the rule sends `π(s)` to `s`, which is `π_inv`. Every
plaintext symbol appears, because every count is positive, so every
value of `π` is recovered. Apply `π_inv` to each ciphertext position.

The hypothesis that the counts are pairwise distinct is necessary for
this proof. It is not a decoration.

**Counterexample — a tie.** Suppose the plaintext on `A5` uses symbol
`0` twice and symbol `1` twice, and uses nothing else. The counts of
`0` and `1` are equal. After any permutation, two ciphertext symbols
appear twice each. Matching counts cannot tell which of those two
ciphertext symbols is `π(0)`. Both assignments are consistent with the
counts. Assumption N11.8’s rule does not apply, and Proposition N11.14
does not claim a unique key. A reader who breaks the tie by preference
has added an assumption. Write it down or stop at the two remaining
keys.

The same proposition, with alphabet `A26`, is why a substitution
ciphertext whose plaintext letter counts are pairwise distinct is
determined by those counts. English, as a natural language, is not such
a list, and Assumption N11.5 refuses to pretend that it is. If you want
the proposition, you state the counts of the message at hand. A published
table of typical English frequencies is a different assumption, and this
lesson does not adopt one. The five-symbol example is complete without
it.

What `26!` still measures is the exhaustive-trial count under Assumption
N10.4 if the only procedure you allow yourself is trying keys, and if a
recognition rule answers yes or no for each trial. Proposition N11.14 is
not that procedure. Shannon’s sentence in N10, that different systems
require different amounts of work, is the comparison between “try `26!`
permutations” and “sort twenty-six counts.” This lesson does not convert
either procedure into hours.

### N11.12 A repeating key is several shifts

A substitution uses one permutation everywhere. A Vigenère key uses a
short string of shift keys and repeats it.

**Definition N11.11 — Vigenère scheme.** Fix a period `p ≥ 1`. A key is
a string `k` of length `p` over `A26`. The plaintext and the ciphertext
are strings over `A26` of any positive length `n`. Position `i`, counting
from `0`, is encrypted by the shift whose key is `k` at index `i mod p`:

```text
E(k, x)_i = the representative of x_i + k_(i mod p),
D(k, y)_i = the representative of y_i - k_(i mod p).
```

The index `i mod p` is the remainder of §6.3 on dividing `i` by `p`. It
lies in `0` through `p - 1`.

**Proposition N11.15 — The Vigenère scheme is correct.** For every
period `p ≥ 1`, every key of that length, every length `n ≥ 1`, and
every plaintext string of length `n`, decrypting the ciphertext with the
same key returns the plaintext.

**Proof.** At position `i`, the shift correctness of Proposition N11.4
applies to the addend at index `i mod p`. The addend used to decrypt is
the same residue, because `i mod p` depends on `i` and `p`, not on the
letter. Each position returns the original residue.

If `p = 1`, the key is one residue and the scheme is the shift scheme.
If `p = n` and the key string is as long as the message and is not
reused on a second message, the construction is no longer a short key
repeated. That longer-key case is the one-time pad’s addition form,
taken up later under its own hypotheses. Using one name for both hides
the hypothesis that the key repeats. This section keeps `p` fixed and
allows `n` to grow past `p`.

**Example N11.10 — ATTACK under DOG.** Let the key be `DOG`, residues
`[3, 14, 6]`, so `p = 3`. The plaintext `ATTACK` is
`[0, 19, 19, 0, 2, 10]`. The addends by position are
`3, 14, 6, 3, 14, 6`. The sums are

```text
0 + 3 = 3,
19 + 14 = 33 ≡ 7,
19 + 6 = 25,
0 + 3 = 3,
2 + 14 = 16,
10 + 6 = 16.
```

The ciphertext is `[3, 7, 25, 3, 16, 16]`, letters `DHZDQQ`. A second
message under the same key, `MEET` = `[12, 4, 4, 19]`, uses addends
`3, 14, 6, 3`. The sums are `15, 18, 10, 22`, letters `PSKW`.

**Listing N11.8 — `vigenere.or`**

```orange
edition 2026;
module vigenere {
  type Letter = Mod[26];

  spec attack() -> Letter^6 {
    let p: Letter^6 = [0, 19, 19, 0, 2, 10];
    let k: Letter^3 = [3, 14, 6];
    for i in 0..6 with s: Letter^6 = p {
      s with [i] = p[i] + k[i % 3]
    }
  }

  spec meet() -> Letter^4 {
    let p: Letter^4 = [12, 4, 4, 19];
    let k: Letter^3 = [3, 14, 6];
    for i in 0..4 with s: Letter^4 = p {
      s with [i] = p[i] + k[i % 3]
    }
  }

  spec cancel() -> Letter^4 {
    let c1: Letter^6 = attack();
    let c2: Letter^4 = meet();
    [c1[0] - c2[0], c1[1] - c2[1], c1[2] - c2[2], c1[3] - c2[3]]
  }
}
```

**Expected evaluation output:**

```text
vigenere::attack: Mod[26]^6 = [3, 7, 25, 3, 16, 16]
vigenere::meet: Mod[26]^4 = [15, 18, 10, 22]
vigenere::cancel: Mod[26]^4 = [14, 15, 15, 7]
```

The remainder `i % 3` is in `0, 1, 2` for every index of either loop, so
`k[i % 3]` selects an element of a length-`3` key. `cancel` subtracts the
two ciphertexts on the four shared positions. The key is not an input to
`cancel`. The printed value `[14, 15, 15, 7]` is nevertheless the
difference of the plaintexts `ATTA` and `MEET`:

```text
0 - 12 ≡ 14,
19 - 4 = 15,
19 - 4 = 15,
0 - 19 ≡ 7.
```

The key cancelled. The next proposition says why that is every repeating
key, not a coincidence of `DOG`.

**Proposition N11.16 — A repeated addend cancels in a difference.** Let
two plaintext strings be encrypted with the same Vigenère key `k` of
period `p`. At any position `i` where both strings have a symbol,

```text
E(k, x)_i - E(k, w)_i ≡ x_i - w_i (mod 26).
```

Inside one string, if position `i + p` exists, then

```text
E(k, x)_(i+p) - E(k, x)_i ≡ x_(i+p) - x_i (mod 26).
```

**Proof.** The first ciphertext symbol is congruent to `x_i` plus the key
residue at index `i mod p`, and the second ciphertext symbol is congruent
to `w_i` plus that same residue. Subtract. The two copies of the key
residue cancel, leaving `x_i - w_i`. For the second claim, the key index
at position `i + p` is `(i + p) mod p`. Adding `p` does not change the
remainder on division by `p`, so the index equals `i mod p`. The two
positions use the same addend, and the addend cancels in the difference.

`cancel` in Listing N11.8 is the first claim on four positions. You can
read the plaintext difference off the ciphertexts without knowing `DOG`.
What you cannot read off, from that difference alone, is either plaintext.
The difference `14` at the first position is consistent with many pairs
of residues. The output of a difference does not determine both inputs.
A guess of one plaintext determines the other. Guess that the first
message begins `ATTA`. Subtract the difference residues:

```text
0 - 14 ≡ 12, 19 - 15 = 4, 19 - 15 = 4, 0 - 7 ≡ 19,
```

which is `MEET`. A wrong guess of the first message yields a wrong second
message, still consistent with the difference. The attack produces a
relation. A recognition rule, Assumption N11.7, is what promotes one pair
of strings to the plaintexts.

**Example N11.11 — A repeated block suggests a period.** Let the
plaintext be `THECATTHE` and let the key again be `DOG`. The length is
`9` and the period is `3`, so the first `THE` and the `THE` that starts
at position `6` sit a multiple of the period apart. Proposition N11.16’s
second claim, applied three times, says those two blocks have the same
ciphertext: both are shifted by `D`, then `O`, then `G`. The ciphertext
residues are

```text
22, 21, 10, 5, 14, 25, 22, 21, 10,
```

letters `WVKFOZWVK`. The block `WVK` occurs at the start and again at
position `6`. The distance is `6`. The period `3` divides `6`. An
adversary who sees only the ciphertext can list the distances between
repeated blocks. A distance is not yet a period. It is a multiple of the
period when the repeated plaintext block was aligned with the key, which
is an extra hypothesis. Here the hypothesis holds because the plaintext
was built that way. If a repeated ciphertext block came from two
different plaintext blocks that the shifts happened to push together, the
distance would not be evidence of a period. The string `WVKFOZWVK` does
not, by itself, tell those two histories apart. The construction does.

Once a period `p` is assumed, each residue class of positions is a shift
cipher with its own one-residue key. Positions `0, 3, 6` of the
ciphertext are `22, 5, 22`. If the recognition rule says the plaintext
letters in those positions are `T`, `C`, `T`, residues `19, 2, 19`, then
the key residue is `22 - 19 = 3`, which is `D`, and it agrees on the
middle position: `2 + 3 = 5`. The other two classes recover `O` and `G`
by the same subtraction. The Vigenère ciphertext has been reduced to
three shift breaks. Each break is §N11.8, on a shorter string. The
period was the whole extra ingredient.

**Listing N11.9 — `wide_key.or`, intentionally rejected**

```orange
edition 2026;
module wide_key {
  spec enc() -> Mod[26]^6 {
    let p: Mod[26]^6 = [0, 19, 19, 0, 2, 10];
    let k: Mod[26]^3 = [3, 14, 6];
    for i in 0..6 with s: Mod[26]^6 = p {
      s with [i] = p[i] + k[i % 4]
    }
  }
}
```

**Diagnostic:**

```text
error[ORC0223]: this index runs from 0 through 3, out of range for `Mod[26]^3`
 --> <stdin>:7:29
  |
7 |       s with [i] = p[i] + k[i % 4]
  |                             ^^^^^ indices run from 0 through 2
  = note: every value an index can take, over every loop index and word in it, must select an element
```

The status is `1` for both `check` and `eval`. Standard output is empty.
The key array has length `3`, so the only legal indices are `0, 1, 2`.
The expression `i % 4`, for `i` in `0` through `5`, takes each value
`0, 1, 2, 3`. The value `3` does not select an element. The checker
computes that set of remainders before any step runs. It does not wrap
the index, and it does not try the body for the values of `i` that would
have been safe.

The rejected source is what a period mistake looks like in this language.
The key was built with period `3`. The index used period `4`. Those are
different functions, and one of them does not denote a value on this
array. A program that used `% 3` is Listing N11.8, which evaluates.
Changing `4` to `3` is a repair only if period `3` is the function you
meant. If you meant a length-`4` key, the repair is a longer key array,
and the ciphertext changes. Read the code, the locus, and the note, and
change the source only in the way that matches the intended function.

Each classical construction above is a correct symmetric scheme. Each
fails concealment against an adversary who knows the method, once a
recognition rule, a count, or a repeated key is supplied as stated. None
of them is an authentication check. Decrypting `DHZDQQ` under a guessed
key and obtaining letters does not tell the receiver that the sender used
that key. The next sections separate the words people reach for when they
try to repair the failure by saying “use a random key,” and then they
state the conditions under which a pad meets the epigraph.

### N11.13 A key, a distribution, a nonce, and a counter

Four words get used as if they were one. They name four different objects.
This section gives each object one definition and then shows a value that
satisfies one and fails the others. The one-time pad needs all four
sentences kept apart, because its hypotheses do not survive being merged.

**Definition N11.12 — Key.** In a symmetric scheme, a **key** is an
element of the key set `K`. It is an input to `E` and to `D`. Under
Assumption N11.1 the set `K` and the functions are public. The selected
element is not part of the method. The adversary’s view, Definition
N11.3, does not include it unless a further assumption puts it there.

**Definition N11.13 — Distribution of a key.** A **distribution** of a
key is a finite probability space whose outcomes are the elements of `K`,
or of a stated subset of `K`. It is the object Definition N10.4 names.
A single key string is not a distribution. A missing distribution is not
filled in by Assumption N10.2.

**Definition N11.14 — Nonce.** A **nonce** is a value, drawn from a
stated set, that a stated construction assumes will not be repeated
together with the same key. The assumption is about repetition. It is
not an assumption about secrecy. A nonce may be sent in the clear beside
the ciphertext. Publishing it does not, by this definition, publish the
key. Repeating it means the construction’s assumption is false, whatever
the key still is.

**Definition N11.15 — Counter.** A **counter** is an integer in a stated
sequence whose successive values differ by one, starting from a stated
start, or it is one term of that sequence. The sequence `0, 1, 2, ...`
up to a stated bound is a counter. A counter can be used as a nonce when
the construction’s “not repeated” assumption is discharged by “the next
integer has not been used with this key.” The counter is still the
sequence. The nonce is still the non-repetition assumption. One integer
can play both roles only after both sentences have been written.

**Example N11.12 — Four values, four jobs.**

1. The residue `3` in Example N11.3 is a key of the shift scheme. No
   distribution has been stated for it. It is not a nonce: the scheme
   reuses it at every letter, and the definition of the shift scheme says
   so. It is not a counter.
2. The uniform distribution on `A26` is a distribution. It is not a key.
   A key is one residue. This distribution has twenty-six outcomes.
   Saying “the key is uniform” without naming the set is not this
   distribution. Assumption N10.2 still requires the set and the weights.
3. Suppose three ciphertexts are sent with the public integers `0`, `1`,
   and `2` beside them, and the construction says each integer will be
   used once with a fixed secret key. Those integers are a counter. They
   are being used as nonces. They are not keys: they are in the
   adversary’s view. An adversary who reads `1` has not read the secret
   key.
4. Suppose an eight-bit string is drawn from the uniform distribution on
   the `256` byte strings and is sent in the clear, and the construction
   forbids repeating that string with the same secret key. The drawn
   string is a nonce. The uniform distribution is the distribution of
   that draw, not the nonce itself, and not the secret key. A second
   draw that happens to equal the first violates Definition N11.14 even
   though the secret key was not revealed. The collision probability of
   such draws is an N10 question about the stated space. It is not a
   reason to call the nonce a key.

Predict which of the four a reused Vigenère key is. The string `DOG` is a
key of Definition N11.11. Reusing it on `ATTACK` and on `MEET` is the
scheme as defined: one key, two messages. It is not a nonce failure
inside the Vigenère definition, because that definition does not assume
the key will be used once. The cancellation in Proposition N11.16 is what
reuse costs. If a different construction had assumed the key string would
not be repeated, that assumption would be Definition N11.14, and the
example would violate it. The word you reach for depends on the sentence
the construction actually contains.

“Random” is still not a name for any of the four. N10 refused the word as
a technical term. A draw is described by its space. A key is described by
membership in `K`. A nonce is described by a non-repetition assumption. A
counter is described by its sequence. A sentence that says “use a random
nonce as the key” has stacked three definitions and stated none of them.

### N11.14 The one-time pad

The classical schemes failed because a short key was reused across
positions or across messages, and the reuse left an equation the
adversary could write. The one-time pad is the scheme that refuses that
reuse in the definition, and then proves what the refusal buys. The
proof uses one symbol first, where the arithmetic is visible, and then
a finite string of symbols.

**Definition N11.16 — One-time pad, one symbol.** Let `q ≥ 2` be an
integer, and let `A` be a set of `q` symbols, represented as the residues
`0` through `q - 1`. The **one-time pad** on one symbol has plaintext set
and ciphertext set and key set all equal to `A`. Encryption and
decryption are

```text
E(k, m) = the representative of m + k modulo q,
D(k, c) = the representative of c - k modulo q.
```

The following conditions are part of the definition whenever a secrecy
claim is made. They are not part of correctness.

1. The key `K` is uniform on `A`: each residue has weight `1/q`.
2. The key choice and the message choice are independent in the sense of
   Definition N10.10.
3. One key is used for one message. The probability space is the space of
   pairs `(m, k)`, not a space of two messages sharing `k`.
4. The receiver is given `k`. The adversary’s view is the ciphertext and
   the method, not `k`.

Condition 4 is an access assumption. Conditions 1 through 3 are
statements about a finite probability space. Mixing them produces
sentences that cannot be checked.

**Proposition N11.17 — The one-symbol pad is correct.** For every `k` and
every `m` in `A`, `D(k, E(k, m)) = m`.

**Proof.** This is Proposition N11.4 with modulus `q` in place of `26`.
The proof there used uniqueness of the representative and the
cancellation `(m + k) - k = m`. Neither step used the value `26`.

When `q = 2`, addition modulo `2` is XOR on one bit. The four pairs are
the truth table:

| `m` | `k` | `m + k` mod `2` | `m XOR k` |
| --- | --- | --- | --- |
| 0 | 0 | 0 | 0 |
| 0 | 1 | 1 | 1 |
| 1 | 0 | 1 | 1 |
| 1 | 1 | 0 | 0 |

The two operation columns agree on every row. A one-bit pad may therefore
be written as XOR. A pad on a larger alphabet may not. XOR is an
operation on bits, defined in Chapter 3 by that table and then positionwise
on strings. Modular addition is the operation in Definition N11.16 for a
general `q`.

**Listing N11.10 — `bitpad.or`**

```orange
edition 2026;
module bitpad {
  spec table() -> Mod[2]^4 {
    [0 + 0, 0 + 1, 1 + 0, 1 + 1]
  }

  spec words() -> Word[8]^4 {
    [0x00 ^ 0x00, 0x00 ^ 0x01, 0x01 ^ 0x00, 0x01 ^ 0x01]
  }
}
```

**Expected evaluation output:**

```text
bitpad::table: Mod[2]^4 = [0, 1, 1, 0]
bitpad::words: Word[8]^4 = [0x00, 0x01, 0x01, 0x00]
```

The residue column is the third column of the table. The word column is
XOR on the low bit of a byte, with every higher bit zero. The two arrays
agree. That agreement is the four-row check, exhaustive for one bit. It
is not a definition of XOR on `Mod[26]`.

**Listing N11.11 — `residue_xor.or`, intentionally rejected**

```orange
edition 2026;
module residue_xor {
  spec mix(x: Mod[26], k: Mod[26]) -> Mod[26] {
    x ^ k
  }
}
```

**Diagnostic:**

```text
error[ORC0215]: `^` is not defined for `Mod[26]`
 --> <stdin>:4:7
  |
4 |     x ^ k
  |       ^ `Mod[26]` is required here
  = note: bitwise operators apply only to `Word[n]` values
```

The status is `1`. Standard output is empty. The code is `ORC0215`. The
locus is the operator `^`. The note says bitwise operators apply to
words. A letter residue is not a word. The shift scheme’s addition is
legal on `Mod[26]`, and Listing N11.1 evaluates it. XOR is legal on
`Word[8]`, and Listing N11.10 evaluates it. Writing XOR on a letter
residue is neither function. The checker refuses the conflation. A
one-time pad on the English alphabet, in this lesson, is addition modulo
`26` under Definition N11.16. A one-time pad on bits is XOR. The epigraph’s
Vernam system, in Shannon’s §10, is the alphabet form with a key as long
as the message. The binary form is the case `q = 2` of the same
definition. The rejected listing is what it looks like to pretend the
two spellings are one operator.

**Definition N11.17 — Perfect secrecy.** Let `M` be a message-valued
outcome on a finite probability space, and let `C` be a ciphertext-valued
outcome on the same space. The pair `(M, C)` has **perfect secrecy** when,
for every message `m` and every ciphertext `c` with `P(C = c) > 0`,

```text
P(M = m | C = c) = P(M = m).
```

If `P(C = c) = 0`, Definition N10.7 assigns no conditional probability.
That gap is not filled with zero, and it is not counted as a failure of
the equality. The equality that is required is exactly the case Shannon
names in the sentence after the definition quoted at the opening: the a
posteriori probability equals the a priori probability. On a finite list
of weights, those are the conditional weights of Proposition N10.5 and
the original weights. No entropy is used. N10 did not define a weighted
substitute for `log2`, and this lesson does not add one.

**Proposition N11.18 — The one-symbol pad has perfect secrecy.** Assume
conditions 1 through 3 of Definition N11.16. Let the message distribution
on `A` be any finite probability space: weights nonnegative and summing
to `1`. Then for every message `m` and every ciphertext `c`,

```text
P(M = m | C = c) = P(M = m),
```

and `P(C = c) = 1/q`, so the conditioning event is never the zero event.

**Proof.** The outcomes of the joint space are pairs `(m, k)`.
Independence and uniformity give the pair the weight
`P(M = m) · (1/q)`. The ciphertext equals `c` and the message equals `m`
together exactly when `k` is the representative of `c - m`. There is one
such `k`. Therefore

```text
P(M = m and C = c) = P(M = m) / q.
```

Sum over the `q` possible messages. The weights `P(M = m)` sum to `1`, so

```text
P(C = c) = 1/q.
```

The sum does not depend on `c`. Every ciphertext has the same
probability, and that probability is positive because `q ≥ 2`. Definition
N10.7 therefore supplies

```text
P(M = m | C = c) = (P(M = m) / q) / (1/q) = P(M = m).
```

The message distribution cancelled. It did not have to be uniform. That
is the content of the proposition: whatever the a priori weights were,
within the class of finite probability spaces, the ciphertext leaves them
as they were.

Read the quantifiers. For every message distribution, for every message,
for every ciphertext, the equality holds. A single numerical example is
not this statement. The example is still worth computing, because it is
where a dropped hypothesis shows up.

**Example N11.13 — A biased message and a uniform key.** Let `q = 2`,
`P(M = 0) = 1/4`, and `P(M = 1) = 3/4`. The key is uniform, weight `1/2`
each, and independent of `M`. Then

```text
P(M = 0 and C = 0) = (1/4) · (1/2) = 1/8,
P(M = 1 and C = 0) = (3/4) · (1/2) = 3/8,
P(C = 0) = 1/8 + 3/8 = 1/2,
P(M = 0 | C = 0) = (1/8) / (1/2) = 1/4.
```

The conditional probability equals the prior. The ciphertext `0` is more
often produced by message `1` than by message `0`, because message `1` is
more common: the joint weights are `3/8` and `1/8`. Dividing by
`P(C = 0)` scales both, and the ratio returns to `1/4` and `3/4`. Seeing
the ciphertext did not update the message weights. That is perfect
secrecy on this space. It is not a claim that the adversary assigns equal
weight to the two messages. The adversary’s uncertainty, in the sense of
§N10.14, remains the biased list.

**Counterexample — a key that is not uniform.** Keep the same message
weights and drop condition 1. Let `P(K = 0) = 3/4` and `P(K = 1) = 1/4`,
still independent of `M`. The pair weights at ciphertext `0` are

```text
P(M = 0 and C = 0) = (1/4) · (3/4) = 3/16,
P(M = 1 and C = 0) = (3/4) · (1/4) = 3/16,
P(C = 0) = 3/8,
P(M = 0 | C = 0) = (3/16) / (3/8) = 1/2.
```

The prior was `1/4`. The conditional probability is `1/2`. They are not
equal, so Definition N11.17 fails. The same space at ciphertext `1` gives

```text
P(M = 0 and C = 1) = (1/4) · (1/4) = 1/16,
P(C = 1) = 1/16 + (3/4) · (3/4) = 1/16 + 9/16 = 10/16 = 5/8,
P(M = 0 | C = 1) = (1/16) / (5/8) = 1/10,
```

which is also not `1/4`. One biased key distribution is enough to reject
the universal claim “every pad is perfectly secret.” The word “pad” in
that sentence omitted condition 1. The omission is the whole
counterexample.

**Definition N11.18 — One-time pad on a string.** Let the message be a
string of length `n ≥ 1` over `A`. The key is a string of length `n` over
`A`. Encrypt and decrypt position by position with Definition N11.16. The
secrecy conditions become: each key symbol is uniform on `A`; the `n` key
symbols are independent of each other and of the message, in the sense of
Definition N10.10 extended by multiplying one weight per symbol, as N10
states for more than two choices; the key string is used for one message
string; the receiver is given the key string and the adversary is not.

**Proposition N11.19 — The string pad has perfect secrecy.** Under those
conditions, for every message string `m` and every ciphertext string `c`
of length `n`,

```text
P(M = m | C = c) = P(M = m),
```

and `P(C = c) = 1/q^n`.

**Proof.** There are `q^n` key strings. Independence and uniformity give
each key string weight `1/q^n`, by the product of `n` factors `1/q`. For
a fixed message string `m` and ciphertext string `c`, exactly one key
string satisfies the positionwise equation `c_i ≡ m_i + k_i`: take `k_i`
to be the representative of `c_i - m_i`. Independence of the key choice
and the message choice gives

```text
P(M = m and C = c) = P(M = m) / q^n.
```

Sum over all `q^n` message strings. The message weights sum to `1`, so
`P(C = c) = 1/q^n`, for every `c`. Divide. The conditional probability
equals `P(M = m)`.

The proof is the one-symbol proof with `q` replaced by `q^n` and with the
unique key string in place of the unique key symbol. The replacement is
legitimate because both proofs use the same two facts: exactly one key
takes a given message to a given ciphertext, and that key has the same
weight no matter which message was named. A shift scheme has the first
fact for each single key and does not have the second fact across
messages of length greater than one, because one key symbol is shared.
The shared symbol is why Proposition N11.16 can cancel it.

**What you must not drop.** If the key string is shorter than the message
and a symbol is reused, you are no longer in Definition N11.18. You are
in the Vigenère scheme, and Proposition N11.16 applies. If the key string
has length `n` but is used on a second message, you are in the next
section. If the key symbols are uniform but the message and the key are
not independent, Definition N10.10 does not give the product weight, and
the line `P(M = m and C = c) = P(M = m) / q^n` is unavailable. The proof
stops at the hypothesis it used. It does not stop at a slogan about pads.

The length of the key string is `n` symbols. The distribution is uniform
on `q^n` strings. The adversary’s uncertainty about the message, given
the ciphertext, is the original message distribution. Those are the three
quantities of §N10.14, and on this scheme they do different jobs. The key
length equals the message length. The distribution is uniform. The
uncertainty about the message is whatever it was before the ciphertext
arrived. A shorter key can have a uniform distribution on its own shorter
set and still leave the message uncertainty smaller than it started, as
the biased-key counterexample and the Vigenère cancellation both show,
each by dropping a different hypothesis.

### N11.15 The same pad used twice

**Proposition N11.20 — Two encryptions with one key string cancel.** Let
`m1` and `m2` be bit strings of length `n`, and let `k` be a bit string
of length `n`. Let `c1 = m1 ⊕ k` and `c2 = m2 ⊕ k`, with XOR the
positionwise operation of Chapter 3. Then `c1 ⊕ c2 = m1 ⊕ m2`.

**Proof.** Work at one position. The key bit is `0` or `1`. If it is `0`,
both ciphertext bits equal the corresponding message bits, so their XOR
equals the XOR of the message bits. If it is `1`, each ciphertext bit is
the message bit flipped. XOR of the two flipped bits equals XOR of the
two original bits: flipping both inputs leaves `0 XOR 0 = 0` and
`1 XOR 1 = 0` as the equal case, and `0 XOR 1 = 1` and `1 XOR 0 = 1` as
the unequal case. Both values of the key bit are covered, which is a
proof by cases on a two-element set. Every position behaves the same way,
and XOR does not change the length, so the strings are equal.

The proof did not assume the key was uniform or secret. Cancellation is
an identity of the operation. It holds for the all-zero key, which hides
nothing, and for every other key. Secrecy hypotheses are not what make
the identity true. They are what make a single use of the key leave the
message weights unchanged. The identity says a second use publishes the
message difference.

**Listing N11.12 — `pad.or`**

```orange
edition 2026;
module pad {
  spec enc(m: Word[8], k: Word[8]) -> Word[8] {
    m ^ k
  }

  spec sample() -> Word[8]^3 {
    let m1: Word[8] = 0x41;
    let m2: Word[8] = 0x42;
    let k: Word[8] = 0x3c;
    [enc(m1, k), enc(m2, k), enc(m1, k) ^ enc(m2, k)]
  }

  spec plain() -> Word[8] {
    0x41 ^ 0x42
  }

  test "two-time pad cancels the key" {
    let m1: Word[8] = 0x41;
    let m2: Word[8] = 0x42;
    let k: Word[8] = 0x3c;
    (enc(m1, k) ^ enc(m2, k)) == (m1 ^ m2)
  }
}
```

**Expected evaluation output:**

```text
pad::sample: Word[8]^3 = [0x7d, 0x7e, 0x03]
pad::plain: Word[8] = 0x03
```

**Test report:**

```text
test "two-time pad cancels the key" ... ok
1 test: 1 passed, 0 failed
```

Compute the bytes before you trust the line. `0x41` is `01000001`.
`0x3c` is `00111100`. XOR gives `01111101`, which is `0x7d`. `0x42` is
`01000010`, and XOR with the same key gives `01111110`, which is `0x7e`.
XOR of the two ciphertexts is `00000011`, which is `0x03`. XOR of the
two plaintexts is the same `0x03`, because they differ only in the last
bit. The test’s `Bool` is that equality on these three bytes. Proposition
N11.20 is the identity for every triple of equal-length strings. The test
is one triple. A passing test is not the proposition, and the proposition
is not a secrecy claim.

**Proposition N11.21 — A second use destroys perfect secrecy.** Let two
message bytes `M1` and `M2` be drawn independently and uniformly from the
`256` byte values, and let `K` be an independent uniform byte used as the
pad for both. Let `C1 = M1 ⊕ K` and `C2 = M2 ⊕ K`. Let `d` be the byte
`0x03`. Then `P(M1 ⊕ M2 = d) = 1/256`, while

```text
P(M1 ⊕ M2 = d | C1 ⊕ C2 = d) = 1.
```

The two probabilities differ, so the pair of messages and the pair of
ciphertexts do not have perfect secrecy.

**Proof.** For each of the `256` values of `M1` there is exactly one `M2`
with `M1 ⊕ M2 = d`, namely `M2 = M1 ⊕ d`. Independence and uniformity
give each pair weight `1/256 · 1/256`, so the event has probability
`256 / 65536 = 1/256`. On the other side, Proposition N11.20 says
`C1 ⊕ C2 = M1 ⊕ M2` for every key. The event `C1 ⊕ C2 = d` is the same
set of outcomes as the event `M1 ⊕ M2 = d`. Conditioning on an event
that is identical to the event being asked yields conditional probability
`1`, provided the event has positive probability, which `1/256` is. A
probability `1` is not a probability `1/256`.

The numerical gap is the whole failure. An adversary who sees `0x7d` and
`0x7e` computes `0x03` and now knows the plaintext difference with
conditional probability `1`. Before seeing the ciphertexts, that
difference was one byte among `256`. The key never appears in the
difference. Withholding `k` does not withhold `m1 ⊕ m2`.

If the adversary also has a recognition rule that accepts one of the two
messages, the other message is determined. Guess `M1 = 0x41`. Then
`M2 = 0x41 ⊕ 0x03 = 0x42`. The guess is an extra assumption, of the same
kind as Assumption N11.7. Without it, the adversary knows the difference
and does not know either byte. That is already enough to reject perfect
secrecy for the pair. It is not a recovery of both messages from nothing.

Condition 3 of Definition N11.16 is what this section removed. Putting it
back, with an independent fresh key for the second byte, returns the
setup to Proposition N11.19 on a message string of length `2`, or to two
separate one-symbol pads. The fresh key is a second draw, not a second
name for the first draw. A counter beside the ciphertext does not supply
that draw. A nonce assumption that the key will not be reused is the
sentence condition 3 already is. Calling the key a nonce does not create
a second independent sample.

### N11.16 The finish line

The six outcomes from the opening are discharged as follows.

1. Encoding, encryption, hashing, and authentication are Definitions
   N11.1, N11.2, N11.4, and N11.5. The public shift `E3` is an encoding
   and a correct scheme with a one-element key set, and it reveals the
   message. Parity is a hash function on two-bit strings and collides.
   A public parity tag accepts the carrier’s substitute. Decryption that
   always returns a letter is not an authentication check.
2. The shift, affine, and substitution schemes are Definitions N11.6,
   N11.9, and N11.10. Correctness is Propositions N11.4, N11.10, and
   N11.12. The inverse exists for the affine multiplier exactly under
   Proposition N11.9. The breaks are Example N11.4 under the rule “equals
   `HELLO`,” Example N11.7 under two known pairs, and Example N11.9 under
   distinct counts.
3. The Vigenère scheme is Definition N11.11. A known period reduces to
   shifts, as Example N11.11 does for `DOG`. The same key on two messages
   cancels by Proposition N11.16, and Listing N11.8 prints the difference.
4. Key, distribution, nonce, and counter are Definitions N11.12 through
   N11.15. Example N11.12 separates four values.
5. The one-time pad’s conditions are Definitions N11.16 and N11.18.
   Perfect secrecy is Definition N11.17. The one-symbol proof is
   Proposition N11.18, and the string proof is Proposition N11.19. The
   second use is Proposition N11.20 and Proposition N11.21, with Listing
   N11.12 as one byte triple.
6. A passing test is one true `Bool` on one run. An evaluation print is
   one value of the listed function. The ledger recomputes the finite
   claims it names. None of those three is Proposition N11.18, and none
   of them says that a construction is fit to deploy.

### N11.17 What was not established

The propositions state their hypotheses and stop. Several nearby sentences
are not theorems of this lesson.

No procedure is given for producing a key that meets condition 1. A
uniform distribution is a mathematical object. A machine that emits bytes
is an implementation, and nothing here proves that a particular machine’s
output is that distribution. N10 already separated those subjects. This
lesson inherits the separation.

No authentication check is built. Proposition N11.3 still applies to a
public tag. A pad ciphertext can be altered. Decryption will return some
plaintext. The receiver who needed to reject the alteration was promised
something Definition N11.16 does not contain.

A function that stretches a short key into a long string and then adds
that string to the message is not Definition N11.18. The definition
requires the key string itself to be the uniform independent string of
length `n`. A stretch is a different function, with a different key set,
and its secrecy is not Proposition N11.19. This lesson does not define
that function. A practical stream cipher, a block cipher, a hash function
in the sense of a standard, and a message authentication code are later
constructions. They are not introduced here, and none of the listings is
one of them.

Perfect secrecy is not a computational claim. It does not mention the
work of trying keys. The shift scheme’s trial count of `26` and the
substitution scheme’s trial count of `26!` are exhaustive counts under
Assumption N10.4. They are not approximations to Proposition N11.18, and
Proposition N11.18 is not a reason to call a large finite key set
perfectly secret. The biased-key counterexample has a key set of size `2`
and already fails. Size was not the hypothesis that failed.

Shannon’s §10 also discusses a key as long as an infinite message and
names the Vernam system as realizing that type of secrecy. The proof in
this lesson is the finite case, on a stated alphabet and a stated length.
It does not prove the infinite case, and it does not cite entropy. The
epigraph’s sentence is the finite definition’s consequence, which
Proposition N11.18 proves under the stated conditions. It is not a remark
about `KHOOR`, about `DHZDQQ`, or about a pad whose key was used twice.

Listing N11.2, Listing N11.9, and Listing N11.11 are rejections. Each one
is evidence about that source and this checker: a residue literal outside
the modulus, a key index whose proved range leaves the array, and a
bitwise operator on a residue type. A rejection is not a proof of the
proposition the repaired program is meant to illustrate. The repaired
programs are the listings that evaluate. Their printed values match the
hand calculations on the inputs named beside them.

The reference test recomputes the ledger: the factorial, the affine key
count, the one-symbol posterior weights, the two-time probabilities, and
the exhaustive identities on the small alphabets it loops over. That test
does not execute Orange. The Orange test executes the listings. Neither
test is a cryptographic security claim, a recommendation of a key length,
or an acceptance of the language proposal the compiler implements.

### N11.18 Work at the desk

**Exercise N11.1 — Name the job.** For each item, say which of
Definitions N11.1, N11.2, N11.4, and N11.5 applies, and which of the
other three does not, by pointing at the clause that fails. (a) The
public map from a byte to two hexadecimal digits, with the public inverse
on its image. (b) The shift scheme with key set `A26`. (c) Parity of two
bits, from Example N11.1. (d) A receiver who accepts a message when a
recomputed public parity bit matches a tag sent beside it.

**Exercise N11.2 — The meeting line.** `MEET` was sent as `PHHW` by adding
three. Under Assumption N11.1, what does the carrier compute, and what
plaintext results? Is the one-element scheme correct? Does the carrier’s
success contradict correctness?

**Exercise N11.3 — One wrap, then the quantifier.** Take `x = 24` and
`k = 3` in the shift scheme. Compute `E(k, x)` and `D(k, E(k, x))` from
the definition of the representative. Which step of Proposition N11.4 is
this pair an instance of, and which keys does the pair not cover?

**Exercise N11.4 — Read a row.** In the output of Listing N11.3, which
index is the row `[7, 4, 11, 11, 14]`? If the recognition rule is changed
to “accept every string whose first residue is `10`,” which keys are
accepted? What does Assumption N11.7 require you to say about the
sender’s key?

**Exercise N11.5 — Invert or refuse.** For each of `9`, `13`, `14`, and
`25` modulo `26`, either give the inverse residue or give a common
divisor greater than `1`. Use Proposition N11.9, not a search of the
Orange operator `/`.

**Exercise N11.6 — Two pairs.** An affine key sends `1` to `8` and `2` to
`13`. Find `a` and `b`. Why does the plaintext difference allow the
multiplication that Example N11.7 used?

**Exercise N11.7 — The operator that returned zero.** Listing N11.5
prints `0` for `2 / 2` on `Mod[26]`. What value would Euclidean division
of the integers `2` and `2` give? Why does the residue operation differ,
and what does acceptance of the listing not prove?

**Exercise N11.8 — Three counts.** The integers `26`, `312`, and `26!`
each count a set named in this lesson. Name the set, and name the
procedure whose worst-case trial count under Assumption N10.4 is that
integer. Which of the three procedures is Proposition N11.14?

**Exercise N11.9 — Sort the five counts.** Using only the array
`[5, 2, 8, 1, 4]` from Listing N11.7 and Assumption N11.8, write `π_inv`
as the images of ciphertext symbols `0` through `4`. How many keys of the
five-symbol substitution scheme did you try?

**Exercise N11.10 — The difference you were given.** Two Vigenère
ciphertexts under one unknown key have difference
`[14, 15, 15, 7]` on their first four positions. Assume the second
plaintext is `MEET`. Recover the first four plaintext residues. Which
proposition lets you do this without the key? What do you know if the
assumption about `MEET` is false?

**Exercise N11.11 — The rejected period.** In Listing N11.9, what is the
code, and what set of indices does the note’s range describe? Does the
loop body run? If the intended period is `3`, what is the one-token
repair, and why is a longer key a different repair?

**Exercise N11.12 — Four labels.** Label each item as a key, a
distribution, a nonce, a counter, or a combination you can justify from
Definitions N11.12 through N11.15. (a) The residue `3` used for every
letter of `HELLO`. (b) The uniform weights `1/26` on `A26`, with no
residue selected. (c) The public integers `0, 1, 2` sent once each beside
three ciphertexts under one fixed secret key. (d) A second copy of the
byte `0x3c` used as the pad for `0x42` after it was already used as the
pad for `0x41`.

**Exercise N11.13 — The uniform key.** Repeat Example N11.13 far enough
to compute `P(M = 1 | C = 0)`. Why does the larger joint weight on
message `1` not produce a larger conditional probability than `3/4`?

**Exercise N11.14 — The biased key.** Under the counterexample weights
`P(K = 0) = 3/4` and `P(K = 1) = 1/4`, compute `P(M = 0 | C = 1)`. Which
condition of Definition N11.16 fails, and which equality of Definition
N11.17 fails with it?

**Exercise N11.15 — Two bytes.** Take `m1 = 0x41`, `m2 = 0x42`, and
`k = 0x3c`. Compute `c1`, `c2`, and `c1 ⊕ c2`. Compare with `m1 ⊕ m2`.
If a recognition rule says the first plaintext is `0x41`, what is the
second? What does the adversary know about the difference if the
recognition rule is absent and the messages were uniform and independent?

**Exercise N11.16 — Three subjects.** State one fact established by the
passing test in Listing N11.1, one fact established by Proposition N11.4,
and one fact that neither the test nor the proposition establishes about
the carrier who sees `KHOOR`.

## Worked answers

**N11.1.** (a) Encoding, Definition N11.1: both directions are public and
there is no key. It is not a hash function, because a byte and its two
hexadecimal digits are not a domain strictly larger than the codomain in
the sense that would force a collision; the inverse recovers the byte.
It is not an authentication check. (b) Encryption, Definition N11.2, with
key set `A26`. It is not an encoding, because the key is selected from a
set rather than written into a single public function. It is not, by that
definition, an authentication check: there is no `V`. (c) Hash function,
Definition N11.4: four strings into two bits. It collides, as the table
showed. It is not encryption: there is no key. (d) Authentication check,
Definition N11.5, and it is complete for honest tags. It does not meet
the hope that every substitute is rejected. Proposition N11.3 says the
carrier who recomputes the public parity produces an accepted pair.

**N11.2.** The carrier subtracts three. `P` returns to `M`, and `PHHW`
returns to `MEET`. The one-element scheme is correct by Proposition
N11.1, which is Proposition N11.4 at the only key. The carrier’s success
uses correctness. It does not contradict it. Correctness says the holder
of the key recovers the plaintext. The carrier holds the only key.

**N11.3.** `24 + 3 = 27 = 1 · 26 + 1`, so `E(3, 24) = 1`. Then
`1 - 3 = -2 = (-1) · 26 + 24`, so the representative is `24`. This pair
is one instance of the congruence `(x + k) - k ≡ x` inside the proof of
Proposition N11.4. It covers the key `3` and the plaintext `24`. It does
not cover any other pair. The quantifiers in the proposition cover the
rest.

**N11.4.** The row is index `3`. The rule “first residue is `10`” accepts
only index `0`, whose row is `[10, 7, 14, 14, 17]`. Assumption N11.7
requires you to say that the rule, not the ciphertext alone, selected
that key. If the sender’s key was `3`, this rule accepts a different key.
The arithmetic did not fail. The assumption did.

**N11.5.** `gcd(9, 26) = 1`. From `26 = 2 · 9 + 8`, `9 = 1 · 8 + 1`, and
`8 = 8 · 1 + 0`, the gcd is `1`. Bézout back-substitution gives an
inverse; the representative is `3`, because `9 · 3 = 27 ≡ 1 (mod 26)`.
`13` shares the divisor `13` with `26`. `14` shares the divisor `2`.
`25 ≡ -1`, and `25 · 25 = 625`. `625 - 24 · 26 = 625 - 624 = 1`, so the
inverse of `25` is `25`.

**N11.6.** Subtract the congruences: `13 - 8 ≡ a · (2 - 1)`, so
`a ≡ 5 (mod 26)`. Then `b ≡ 8 - 5 · 1 = 3`. Check: `5 · 2 + 3 = 13`. The
plaintext difference is `1`, and `gcd(1, 26) = 1`, so Proposition N11.9
says the multiplier of the difference is unique. That is the same
multiplication Example N11.7 used, with a difference of `1` instead of a
difference of `3`.

**N11.7.** Euclidean division of the integers gives quotient `1` and
remainder `0`, because `2 = 2 · 1 + 0`. The residue operation `/` on
`Mod[26]` multiplies by a modular inverse, and `2` has none, so the
operation returns `0`. Acceptance shows that `/` is defined on the type.
It does not show that `2` is coprime to `26`, and it does not show that
the result is an integer quotient.

**N11.8.** `26` is `|A26|`, the shift key set, and the worst-case trial
count for trying shift keys. `312` is the affine key set of Definition
N11.9, twelve multipliers times twenty-six addends. `26!` is the number
of permutations of `A26`, the substitution key set. Trying every
permutation is the procedure whose worst-case count is `26!`. Proposition
N11.14 is not that procedure. It matches counts. On the five-symbol
example it sorts five integers.

**N11.9.** The array says ciphertext `2` has count `8`, `0` has count
`5`, `4` has count `4`, `1` has count `2`, and `3` has count `1`.
Matching to plaintext symbols `0` through `4` gives
`π_inv(2) = 0`, `π_inv(0) = 1`, `π_inv(4) = 2`, `π_inv(1) = 3`,
`π_inv(3) = 4`. In order of ciphertext symbols `0, 1, 2, 3, 4`, the
images are `1, 3, 0, 4, 2`. No key was tried. The matching used five
counts.

**N11.10.** Proposition N11.16 says the ciphertext difference equals the
plaintext difference. Adding it to `MEET` = `[12, 4, 4, 19]` gives
`[0, 19, 19, 0]`, because `12 + 14 = 26 ≡ 0`, `4 + 15 = 19`, and
`19 + 7 = 26 ≡ 0`. If `MEET` is not the second plaintext, the recovered
string is not the first plaintext. The difference remains true and the
identification does not.

**N11.11.** The code is `ORC0223`. The index `i % 4` runs through `0, 1,
2, 3`, and the key has indices `0, 1, 2`. The body does not run. The
one-token repair, if the intended period is `3`, is to write `% 3`. A
longer key is the repair for a different function, one whose period is
`4`, and its ciphertexts are not Listing N11.8’s.

**N11.12.** (a) A key of the shift scheme, reused on purpose. Not a
nonce, and not a counter. (b) A distribution, and not a key: no residue
has been selected. (c) A counter used as a nonce, public, and not the
secret key. (d) A second use of one key byte. It violates condition 3 of
Definition N11.16. Calling the byte a nonce records the assumption that
was broken. It does not make the second encryption a fresh pad.

**N11.13.** `P(M = 1 and C = 0) = 3/8` and `P(C = 0) = 1/2`, so
`P(M = 1 | C = 0) = 3/4`, which equals the prior. The joint weight is
larger because the prior is larger. Dividing by the same `P(C = 0)`
scales every message, and the ratios return to the priors. That scaling
is the last line of the proof of Proposition N11.18.

**N11.14.** `P(M = 0 and C = 1) = 1/16` and `P(C = 1) = 5/8`, so the
conditional probability is `1/10`. Condition 1 fails: the key is not
uniform. The equality `P(M = 0 | C = 1) = P(M = 0)` fails because `1/10`
is not `1/4`.

**N11.15.** `c1 = 0x7d`, `c2 = 0x7e`, and `c1 ⊕ c2 = 0x03 = m1 ⊕ m2`. If
the rule says the first plaintext is `0x41`, the second is
`0x41 ⊕ 0x03 = 0x42`. If the messages are independent and uniform and no
recognition rule is stated, the adversary knows the difference equals
`0x03` with conditional probability `1`, and the prior of that difference
was `1/256`. Neither plaintext is singled out by the difference alone.

**N11.16.** The test establishes that `back() == [7, 4, 11, 11, 14]`
evaluated to true on one run of Listing N11.1. Proposition N11.4
establishes correctness for every key and every one-symbol plaintext, and
then for every position of a string. Neither one establishes that the
carrier who sees `KHOOR` fails to recover `HELLO`. Under the recognition
rule of Example N11.4 the carrier recovers it by trying the twenty-six
keys. Under the empty recognition rule the carrier is left with
twenty-six candidates. The proposition does not choose between those
rules.

```text
n11-ledger
shift-keys = 26/1
affine-units = 12/1
affine-keys = 312/1
excluded-affine-pairs = 364/1
shift-expected-trials = 27/2
affine-expected-trials = 313/2
substitution-keys = 403291461126605635584000000/1
small-substitution-keys = 120/1
uniform-joint-m0-c0 = 1/8
uniform-joint-m1-c0 = 3/8
uniform-cipher = 1/2
uniform-posterior-m0 = 1/4
biased-joint-m0-c0 = 3/16
biased-joint-m1-c0 = 3/16
biased-cipher-0 = 3/8
biased-posterior-m0-c0 = 1/2
biased-joint-m0-c1 = 1/16
biased-cipher-1 = 5/8
biased-posterior-m0-c1 = 1/10
two-time-prior = 1/256
two-time-posterior = 1/1
pad-difference = 3/1
hello-row = 3/1
inverse-of-5 = 21/1
inverse-of-9 = 3/1
inverse-of-25 = 25/1
known-plaintext-b = 3/1
```

## Source note for the epigraph

**[S9] Claude E. Shannon.** “Communication Theory of Secrecy Systems.”
*Bell System Technical Journal* 28(4), 1949, pp. 656–715. The epigraph is
the sentence that begins the page whose footer is 680, in §10, “Perfect
Secrecy.” The preceding sentence, which starts at the bottom of the page
whose footer is 679, defines perfect secrecy by the condition that the a
posteriori probabilities equal the a priori probabilities independently
of the values of the cryptograms. A mathematical symbol between “for all”
and “the a posteriori” in that preceding sentence is set in a distinct
face and is not part of the epigraph sentence. The epigraph sentence
itself contains no such symbol. Wording was checked on 2026-10-05 against
the retypeset PDF hosted by the University of Wisconsin–Madison, the same
copy used for [S1] and [S8]. This is a retypeset copy, not a scan of the
1949 printing. No translation is involved. The sentence says that, in the
case just defined, interception gives the cryptanalyst no information. It
does not say that every transformed message has that property, and it is
not a claim that this lesson has established perfect secrecy for any
construction except the one-time pad under the conditions of Propositions
N11.18 and N11.19.

Source: <https://pages.cs.wisc.edu/~rist/642-spring-2014/shannon-secrecy.pdf>

Epigraph verification establishes wording and attribution, not publication-
rights clearance.

## Evidence boundary

N11 adds sixteen exercises with worked answers. The ledger is recomputed
by `tools/test_book_foundations.py`. That check does not execute Orange.
The Orange listings are executed by
`compiler/crates/orangec/tests/book_novice.rs`. A passing test or a
matching print is one run of one source. Neither check establishes a
cryptographic security claim, a key-length recommendation, or acceptance
of a language proposal.

The compiler used to produce the printed values and the three diagnostics
identifies itself as `orangec 0.0.1 (Orange edition 2026; implemented
slice S3t)`. The listings use residue types, bounded loops, and tests
that this binary accepts. Implementation of those slices is not
acceptance of the proposals, and it adds no cryptographic claim.

The lesson was drafted with Grok 4.7 in Cursor on 2026-10-05 at the
owner's direction, and it is stacked after N10 on
`book/novice-journeyman-master-opening`. Proposition N11.8 cites
Assumption N9.4 only as the successor step `Q(n) ⇒ Q(n + 1)`, with
hypothesis `Q(n)` alone. Owner review is pending. No deployment claim
is made.
