# The Orange Book

By Chase Bryan

## Novice continuation

Lesson N9. Draft 2026-10-05.

Read this after [Chapter 6](NOVICE_PROGRAMMING.md#chapter-6-words-have-edges).
Chapters 1–6 of the novice sequence are the prerequisite. Lesson N7, Name
the Intermediate Step, and lesson N8, Read and Repair a Program, are earlier
in the teaching order and are not in this file. Nothing proved below uses
their unwritten material.

The label **N9** is the locked novice-lesson label. It is not manuscript
Chapter 9, and it is not Chapter 7, Chapter 8, or any other numeral already
used in [the original manuscript](../THE_ORANGE_BOOK.md). Those chapters
keep their numerals, including Chapter 7, *No Disposable Prototype*, through
Chapter 12. When this lesson says “Chapter 3” or “Chapter 6”, it means the
novice chapter with that number in the opening or the programming
continuation, not the manuscript chapter that happens to share the numeral.

## N9: Say What You Mean

> “providing provably secure cryptosystems, changing this ancient art into a science.”
>
> — Whitfield Diffie and Martin E. Hellman, *New Directions in
> Cryptography* (1976), §I, p. 644; excerpt. [S7]

You already have proofs in hand. Reversing a finite sequence twice restores
it. XOR with a retained mask cancels. A Euclidean division produces one
quotient and one remainder. Each of those arguments says *every* element of
some collection, and each one stops at a stated boundary.

This lesson names the collections, the correspondences, and the shapes of
argument those proofs already use. It does not replace the proofs, and it
does not fulfill the hope in the epigraph. A sentence about provable
security is not itself a proof, and nothing below is a cryptosystem.

### N9.1 What you may take as given

Four assumptions bound every later step. They are assumptions of this
lesson, not facts the surrounding world has been shown to obey.

**Assumption N9.1 — Two truth values.** Every statement taken up in this
lesson is true or false, and not both. The two values are the ones from
Chapter 3: false, written `0`, and true, written `1`.

**Assumption N9.2 — No set without a rule.** A collection is available here
only after a membership rule says, for each candidate, whether it belongs.
A hopeful description such as “the secure keys” is not yet a set.

**Assumption N9.3 — Two results may be cited.** Proposition 3.1, the
cancellation identity `(x ⊕ k) ⊕ k = x` for finite equal-length bit
strings, and the existence and uniqueness of the Euclidean quotient and
remainder in §6.3, are already established. Their internal write-ups are
not repeated. Citing them is not a new proof of them.

**Assumption N9.4 — Induction on the nonnegative integers.** The
nonnegative integers are `0, 1, 2,` and so on: the integers that are not
negative, including zero. To prove a statement `P(n)` for every
nonnegative integer `n`, it is enough to prove `P(0)`, and to prove that
for an arbitrary nonnegative integer `n`, if `P(n)` is true then
`P(n + 1)` is true. This is a proof rule adopted here. It is not derived
in this lesson from a list of axioms.

A **proof**, as Chapter 1 used the word, is an argument establishing a
stated conclusion from the stated rules and assumptions. A **test**
examines selected cases. Those meanings stay in force. The new names below
are patterns a proof can have, and they do not widen a conclusion past the
sets the proof mentions.

One boundary is easy to miss because it is syntactic. This lesson does not
introduce Orange bindings, arrays, conditions, tuples, or bounded
iteration. Those belong to N7. When an Orange expression appears below, it
uses only forms already written in Chapters 5 and 6.

### N9.2 A set is fixed by its members

**Definition N9.1 — Set and membership.** A **set** is a collection
determined by its members. An object that belongs to a set is an
**element** or **member** of that set. The symbol `∈` means “is a member
of.” The symbol `∉` means “is not a member of.”

Write a small set by listing members between braces. This is a **roster**.
The set of bit values used in Chapter 3 is `{0, 1}`.

A **set builder** describes the members by a rule instead of a finished
list. Read `{ n : n is an integer and 0 ≤ n ≤ 255 }` as “the set of `n`
such that `n` is an integer and `0 ≤ n ≤ 255`.” The colon is the builder’s
punctuation. It is not the colon in an Orange parameter and not a
cardinality bar.

**Example N9.1 — Byte values.** Let `B` be that builder set. Then
`0 ∈ B`, `255 ∈ B`, `256 ∉ B`, and `-1 ∉ B`. The last two failures are
different facts. `256` is an integer that is too large for the inequality.
`-1` is an integer that is too small. Chapter 6 already separated the
integer `-1` from the byte representative `255`. Membership in `B` is the
set form of that separation: `255 ∈ B` and `-1 ∉ B` can both be true.

Predict the status of `2` before reading on. The rule is `0 ≤ n ≤ 255`,
and `2` satisfies it, so `2 ∈ B`. A reader who has just been shown bits
sometimes answers as if `B` were `{0, 1}`. The braces do not carry that
restriction unless the roster or the builder says so.

**Example N9.2 — A phrase that is not yet a set.** “The set of masks that
hide a message” has no membership rule in the material so far. Proposition
3.1 holds for the all-zero mask, and that mask hides nothing. Until a
later lesson states a hiding requirement precisely enough to accept or
reject each candidate, Assumption N9.2 refuses the phrase as a set. Do not
fill the gap by listing a few masks you like.

### N9.3 Equal members, equal sets

**Definition N9.2 — Equality of sets.** Two sets are equal when they have
the same members. Every member of either set is a member of the other.
Order in a roster does not record a further fact, and writing a member
twice does not put it in twice.

So `{0, 1} = {1, 0} = {0, 1, 1}`. The third roster is a redundant
description of a two-member set, not a set of three members.

A bit **string** is not a set. Chapter 2 fixed strings as ordered finite
sequences. The string `10` has a first symbol and a second symbol. The set
`{1, 0}` does not. The string `11` has length two. The roster `{1, 1}`
describes a one-member set. If a later claim says “the inputs,” decide
whether it means a set of values or a string of symbols before you count.

**Definition N9.3 — Cardinality.** A set available in this lesson is
**finite** when its distinct members can be listed completely. The
**cardinality** of that set is the length of a repetition-free list. Write
it `|S|`. The bars are cardinality, not absolute value, and not the
builder’s punctuation.

For the sets actually counted below, the list is given by a roster, by a
count already proved in Chapter 2, or by an argument in this lesson. This
is not a proof that every abstract finite set has an order-independent
cardinality derived from axioms we have not stated.

Thus `|{0, 1}| = 2` and, by the count of eight-bit strings in Chapter 2,
`|B| = 256`. The integer `256` is `|B|`. It is not a member of `B`.

### N9.4 Subsets and the empty set

**Definition N9.4 — Subset.** A set `S` is a **subset** of a set `T`,
written `S ⊆ T`, when every member of `S` is a member of `T`. Equality is
allowed: every set is a subset of itself, because every member of it is a
member of it.

**Definition N9.5 — Empty set.** The **empty set**, written `∅`, is the
set with no members.

**Proposition N9.1 — The empty set is a subset.** If `T` is any set
available in this lesson, then `∅ ⊆ T`.

**Proof.** The subset condition asks us to take each member of `∅` and
find it in `T`. There is no member of `∅` to check. There is therefore no
member of `∅` that fails to lie in `T`. The condition holds.

The same reasoning does not say that `∅` is a member of `T`. Membership
and being a subset are different relations. `∅ ⊆ {0, 1}` is true.
`∅ ∈ {0, 1}` is false: the members of `{0, 1}` are `0` and `1`, and
neither is the empty set.

A proper shrinking is also not automatic from the symbol. Say that `S` is
a **proper subset** of `T` when `S ⊆ T` and `S` is not equal to `T`. Then
`∅` is a proper subset of `B`, and `B` is not a proper subset of `B`.

**Example N9.3 — Even byte values.** Let `E` be the set of members of `B`
that are even, taking even as Chapter 3 did: divisible into pairs of units
with nothing left over. Then `E ⊆ B`, because the builder of `E` starts
from members of `B`. Also `1 ∉ E`, so `E` is a proper subset. The
repetition-free list `0, 2, 4, ..., 254` has `128` entries: one even value
for each of `0` through `127`, namely twice that index. So `|E| = 128`.

A sample of successful tests is not this kind of object. If you check
cancellation on ten masks, you have ten examined pairs, in the sense of a
test from Chapter 1. You do not thereby have a subset of “the masks that
work” unless that larger set has already been defined, and the sample does
not become the full set by being called a subset.

### N9.5 Sets built from sets

**Definition N9.6 — Union, intersection, and difference.** Let `S` and `T`
be sets. Their **union** `S ∪ T` contains exactly the objects that are
members of `S` or of `T`, or of both. Their **intersection** `S ∩ T`
contains exactly the objects that are members of both. Their **difference**
`S ∖ T` contains exactly the members of `S` that are not members of `T`.

The “or” in the union is the inclusive OR of Chapter 3. An element in both
sets is in the union once, not twice, because sets do not store duplicates.

**Example N9.4 — Bits.** `{0} ∪ {1} = {0, 1}`. `{0} ∩ {1} = ∅`.
`B ∖ E` is the set of odd members of `B`, and it also has cardinality
`128`, because the `256` byte values split into the evens and the odds
with no overlap and nothing left out. In symbols,
`E ∪ (B ∖ E) = B` and `E ∩ (B ∖ E) = ∅`.

**Definition N9.7 — Ordered pair and cartesian product.** An **ordered
pair** `(a, b)` records a first component `a` and a second component `b`.
Order matters: `(0, 1)` and `(1, 0)` are different pairs. A pair is not a
two-member set. The **cartesian product** `S × T` is the set of all
ordered pairs whose first component is in `S` and whose second component
is in `T`.

The symbol `×` is doing new work. In §6.2 it was multiplication of
integers. Here it builds a set of pairs. The sentence has to say which.
`B × B` is not the integer `256 × 256` until you take a cardinality.

**Proposition N9.2 — Cardinality of a product.** If `S` and `T` are finite,
then `|S × T| = |S| · |T|`. The dot is ordinary multiplication of the two
cardinalities.

**Proof.** Group the pairs by their first component. For each fixed `a` in
`S`, the pairs `(a, b)` with `b` in `T` are in one-to-one correspondence
with the members of `T`: different second components give different pairs,
and every such pair arises. That slice has `|T|` pairs. Different choices
of `a` give disjoint slices, because an ordered pair has only one first
component. Every pair belongs to the slice of its first component. There
are `|S|` slices, so the total is `|S| · |T|`.

**Example N9.5 — The XOR table and the byte check.** The four rows of the
two-bit truth table are the four members of `{0, 1} × {0, 1}`. Proposition
N9.2 gives `2 · 2 = 4`. The exhaustive byte cancellation check from
Chapter 3 runs through `B × B`, which has `256 · 256 = 65536` pairs. The
proposition counts the domain of that check. It does not rerun the check,
and it does not extend the check to strings longer than eight bits.

### N9.6 Relations

**Definition N9.8 — Relation.** A **relation** from a set `S` to a set `T`
is a subset of `S × T`. When `S` and `T` are the same set, the relation is
a relation **on** that set. If `(a, b)` belongs to the relation, say that
`a` is related to `b`.

A relation may pair one element with many elements, with one, or with
none. Nothing in the definition stops that.

**Example N9.6 — Differing by one.** On the integers, let `a` be related
to `b` when `a - b = 1` or `b - a = 1`. Then `0` is related to `1`, and
`1` is related to `0`. The integer `0` is not related to itself, because
`0 - 0 = 0`, which is neither `1` nor `-1`. And `0` is related to `1` and
`1` is related to `2`, but `0` is not related to `2`, because `2 - 0 = 2`.

**Definition N9.9 — Equivalence relation.** A relation on a set `S` is
**reflexive** when every element of `S` is related to itself. It is
**symmetric** when, for all `a` and `b` in `S`, if `a` is related to `b`
then `b` is related to `a`. It is **transitive** when, for all `a`, `b`,
and `c` in `S`, if `a` is related to `b` and `b` is related to `c`, then
`a` is related to `c`. An **equivalence relation** is a relation that is
reflexive, symmetric, and transitive.

Example N9.6 fails reflexivity and fails transitivity. It is symmetric,
because the two equations `a - b = 1` and `b - a = 1` swap when the names
swap. One surviving property is not enough for the definition.

Congruence, from §6.2, is a different relation on the integers. For a
positive modulus `m`, the pair `(n, r)` is in the relation when `n - r` is
an integer multiple of `m`. The symbol already in use is `n ≡ r (mod m)`.
The representative in `{0, 1, ..., m - 1}` is one related integer, not the
only related integer. Nineteen is congruent to `3` modulo `16`, and also
to `19`, and also to `35`.

**Proposition N9.3 — Congruence is an equivalence relation.** For each
positive integer `m`, congruence modulo `m` is an equivalence relation on
the integers.

**Proof.** Reflexive: `n - n = 0`, and `0 = 0 · m`, so `n ≡ n (mod m)`.

Symmetric: if `n - r = q · m` for an integer `q`, then
`r - n = (-q) · m`, so `r ≡ n (mod m)`.

Transitive: if `n - b = q · m` and `b - c = s · m`, add the equations.
The sum of the left sides is `n - c`, and the sum of the right sides is
`(q + s) · m`. So `n ≡ c (mod m)`.

The proof uses the definition of congruence and the arithmetic of integer
multiplication and addition. It does not reopen the existence of a unique
representative. Uniqueness, from §6.3, says something further: among the
many integers congruent to `n`, exactly one lies in the range
`0` through `m - 1`. The relation and the representative rule are not the
same object. The next sections give the vocabulary for that difference.

### N9.7 Functions, images, and the codomain

Section 5.4 fixed three words for this book. A **function** associates
each permitted input with one output. The **domain** is the set of
permitted inputs. The **codomain** is the specified set in which the
outputs must lie. Those are the canonical meanings. What §5.4 did not
need, and what a reversible rule needs, is the set of outputs actually
attained and the conditions under which the association can be undone.

**Definition N9.10 — Image.** Let `f` be a function with domain `S` and
codomain `T`. The **image** of `f` is `{ f(a) : a ∈ S }`. It is a subset
of `T`. The parentheses in `f(a)` mean application of the function, not an
ordered pair and not an Orange argument list, though an Orange call is one
way a later program can denote an application.

The image is allowed to be a proper subset of the codomain. Collapsing
those two sets is a standard way to claim an output exists when the rule
never produces it.

**Example N9.7 — A retained mask.** Fix a length `n` and a bit string `k`
of that length. Let `S` be the set of bit strings of length `n`. Define
`f` on `S` by `f(x) = x ⊕ k`, with codomain `S`. Each input has one
output, of the same length, so `f` is a function from `S` to `S`. This
uses the bitwise definition of XOR. It does not yet say whether `f` can
be undone. That is Proposition N9.5, after the inverse lemma.

**Example N9.8 — Multiplication by two on bytes.** Define `d` on `B` by
`d(x) =` the representative of `2x` modulo `256`, with codomain `B`.
Chapter 6’s word multiplication on `Word[8]` denotes this function when
the inputs are byte values. It is a function: §6.3 supplies exactly one
representative. The expression `x * 2` in a `Word[8]` context is the
Orange spelling of this same map, not a second mathematical object.

Compute two inputs. `d(0) = 0`. `d(128) =` the representative of `256`,
which is `0`. Two domain elements share an output. The image is not
forced to be all of `B` by the fact that the codomain was declared to be
`B`.

The representative rule of §6.3 is a function from the integers to the
set of representatives. Its **graph** is the relation
`{ (n, r) : r is the representative of n }`. That relation contains
exactly one pair for each integer `n`. Congruence modulo `256` is a
different relation on the integers: `19` is congruent both to `3` and to
`19`, so the pairs `(19, 3)` and `(19, 19)` are both in it. A function,
in the §5.4 sense, is precisely a relation that contains exactly one
pair for each element of the domain. Congruence fails that test. The
representative rule meets it.

### N9.8 One direction of an inverse

**Definition N9.11 — Left and right inverses.** Let `f` be a function from
`S` to `T`, and let `g` be a function from `T` to `S`. Then `g` is a
**left inverse** of `f` when `g(f(a)) = a` for every `a` in `S`. And `f`
is a **right inverse** of `g` in that same situation. When both
`g(f(a)) = a` for every `a` in `S` and `f(g(b)) = b` for every `b` in
`T`, each function is a **two-sided inverse** of the other.

**Proposition N9.4 — Inverses restrict the image.** Let `f` go from `S` to
`T` and `g` from `T` to `S`.

1. If `g` is a left inverse of `f`, and `f(a1) = f(a2)`, then `a1 = a2`.
2. If `f(g(b)) = b` for every `b` in `T`, then every member of `T` is in
   the image of `f`.

**Proof.** For the first claim, apply `g` to the assumed equality.
`g(f(a1)) = g(f(a2))`. Each side is the original element, so `a1 = a2`.

For the second, take any `b` in `T`. The element `g(b)` is in `S`, and
`f` sends it to `b`. So `b` is an attained output.

The first claim says a left inverse prevents two inputs from sharing an
output. The second says a right inverse forces the image to fill the
codomain. Neither claim rewrites the bit argument inside Proposition 3.1.
They only say what an equation of the form `g(f(a)) = a` is for.

### N9.9 Injective, surjective, and bijective

**Definition N9.12 — Injective.** A function `f` from `S` to `T` is
**injective** when distinct members of `S` have distinct outputs. Equivalently,
`f(a1) = f(a2)` implies `a1 = a2`.

**Definition N9.13 — Surjective.** The function is **surjective** when its
image equals its codomain. Every member of `T` is `f(a)` for at least one
`a` in `S`.

**Definition N9.14 — Bijective.** The function is **bijective** when it is
both injective and surjective.

Proposition N9.4, read against these definitions, says: a left inverse
implies injective, and a right inverse implies surjective. A two-sided
inverse implies bijective.

**Proposition N9.5 — XOR with a retained mask is bijective.** Let `S` be
the set of bit strings of a fixed finite length `n`, and let `k` be one
such string. The function `f(x) = x ⊕ k`, with codomain `S`, is bijective.
The function `f` itself is a two-sided inverse of `f`.

**Proof.** Let `g` be `f` itself. Proposition 3.1, cited under Assumption
N9.3, says `g(f(x)) = x` for every `x` in `S`. The same identity, with the
input renamed, says `f(g(y)) = y` for every `y` in `S`. Those are the two
equations in Definition N9.11. Proposition N9.4 supplies injectivity and
surjectivity.

What is new is the classification. What is old is the identity. Do not
describe this paragraph as a second discovery that mask bits flip and flip
back. That reasoning remains where it was written.

The all-zero string is an allowed `k`. For that choice, `f(x) = x` for
every `x`, which is bijective and hides nothing. Bijective does not mean
secret. Chapter 3 already gave this counterexample against a secrecy
reading of cancellation. The new word does not make the old counterexample
obsolete.

**Proposition N9.6 — Doubling on bytes is neither.** The function `d` of
Example N9.8 is not injective and not surjective.

**Proof.** `d(0) = 0` and `d(128) = 0`, with `0 ≠ 128`. Not injective.

Suppose some `x` in `B` had `d(x) = 1`. Then `2x = 256 · q + 1` for the
quotient `q` from §6.3. The left side is `2 · x`, an even integer. The
right side is one more than an even integer, so it is odd. No integer is
both even and odd. So `1` is not in the image, and `d` is not surjective.

The same parity step shows every value in the image is even, so the image
is a subset of `E`. For the other inclusion, take `y` in `E`. Then
`y = 2t` for `t = y / 2`, and `t` is an integer from `0` through `127`,
hence in `B`. Also `2t` is already between `0` and `254`, so its
representative modulo `256` is itself. Thus `d(t) = y`. The image is
exactly `E`.

**Proposition N9.7 — Equal finite cardinality is not the whole story.**
Let the domain and the codomain both be the nonnegative integers, and let
`s(n) = n + 1`. Then `s` is injective and not surjective.

**Proof.** If `s(n) = s(m)`, then `n + 1 = m + 1`. Subtracting `1` is the
cancellation already used for integer counters in Chapter 2, so `n = m`.
Injective.
The codomain element `0` is not `n + 1` for any nonnegative integer `n`,
because `n + 1` is at least `1`. Not surjective.

Propositions N9.6 and N9.7 fail surjectivity for different reasons. The
byte map fails on a finite set whose domain and codomain have the same
cardinality. The successor map fails on an infinite set where “same
cardinality” has not even been defined. The next proposition isolates the
finite equal-cardinality case. It does not apply to `s`, and this lesson
does not define cardinality for infinite sets.

### N9.10 Finite sets of equal cardinality

**Proposition N9.8 — Image size matches injectivity.** Let `f` be a
function from a finite set `S` to a set `T`. Then `|image of f| = |S|` if
and only if `f` is injective.

**Proof.** List the distinct members of `S` as `a1` through `a_|S|`. If
`f` is injective, the outputs `f(a1)` through `f(a_|S|)` are pairwise
distinct, so that list is a repetition-free list of the image and the
cardinalities agree.

If `f` is not injective, then `f(ai) = f(aj)` for some `i ≠ j`. Form a
list of outputs that skips `aj`. Every image element is still some
`f(ak)`: the skipped input’s output equals the kept input’s output. The
resulting list of outputs has length `|S| - 1` before removing any further
duplicates, so a repetition-free list of the image is no longer than
`|S| - 1`. The cardinalities differ.

**Proposition N9.9 — A full-size subset is the whole set.** Let `T` be
finite and let `S ⊆ T`. If `|S| = |T|`, then `S = T`.

**Proof.** Suppose some `b` in `T` is not in `S`. A repetition-free list
of `S` does not contain `b`. Adding `b` produces a repetition-free list of
a subset of `T` of length `|S| + 1`. Therefore `|T|` is at least
`|S| + 1`, which contradicts `|S| = |T|`. The supposed missing `b` does
not exist. Every member of `T` is a member of `S`. Combined with
`S ⊆ T`, Definition N9.2 gives `S = T`.

The second paragraph assumed a missing element, derived that the
cardinalities cannot agree, and rejected the assumption. That shape will
be named proof by contradiction in §N9.20. The reasoning here does not
depend on the later name.

**Proposition N9.10 — Finite sets of equal size.** Let `S` and `T` be
finite, with `|S| = |T|`, and let `f` be a function from `S` to `T`. The
following are equivalent: `f` is injective; `f` is surjective; `f` is
bijective.

**Proof.** The image is a subset of `T`. If `f` is injective, Proposition
N9.8 gives `|image| = |S| = |T|`, so Proposition N9.9 gives
`image = T`. That is surjectivity. If `f` is surjective, then
`image = T`, so `|image| = |T| = |S|`, and Proposition N9.8 gives
injectivity. Either property yields both, which is bijectivity. A
bijective function has both properties by Definition N9.14.

Apply this to `d`. Domain and codomain are both `B`, with cardinality
`256`. Proposition N9.6 showed that `d` is not injective. Proposition
N9.10 therefore says, without a second parity argument, that `d` is not
surjective. The parity argument remains the one that names the image.
The cardinality proposition only ties the two failures together. It does
not apply to the successor function, whose domain is not a finite set we
have assigned a cardinality.

### N9.11 A rule you can undo

Chapter 3’s title is a demand for a criterion, not a mood. A rule can be
undone, in the sense of this lesson, when the function that states the
rule has a two-sided inverse on the stated domain and codomain.

For the mask map, Proposition N9.5 says the inverse rule is the same XOR
again. You undo `f` by retaining `k` and applying `f`. The output alone is
not the inverse’s input specification. Section 3.6 already showed two
input pairs can share an XOR output. The function that is bijective is the
one-variable function with `k` held fixed, not the two-variable function
`(x, k) ↦ x ⊕ k` from `S × S` to `S`.

That two-variable map is surjective and not injective when `n ≥ 1`.
Write `0` for the all-zero string of length `n`. Surjective: the pair
`(y, 0)` is sent to `y ⊕ 0`. Each bit of `y` is combined with `0`, and
the truth table leaves the bit unchanged, so the output is `y`. Not
injective: at every position, a bit XOR itself is `0`, by the two rows
of the XOR table in which the inputs agree. So `k ⊕ k` is the all-zero
string for every `k`. Both `(0, 0)` and `(k, k)` are sent there. Choose
`k` with a single `1`, which exists because `n ≥ 1`. Then `(k, k)` is
not the pair `(0, 0)`. Calling XOR “invertible” without saying which
function is meant confuses a bijection with a map that deletes a whole
input.

Doubling cannot be undone on `B`. A two-sided inverse would be a left
inverse, hence `d` would be injective by Proposition N9.4, contradicting
Proposition N9.6. The later contradiction section extracts the pattern.
The concrete obstruction is already the pair `0` and `128`.

Compare the small composition in §6.10 only at the level of order. There,
addition, XOR, and rotation are undone by the matching inverse steps in
the opposite order. The general fact used, and not reproved, is: if each
step has a two-sided inverse, the composite’s two-sided inverse is the
composite of the inverses in reverse order. One reason is enough to see
the discipline. If `f` is followed by `h`, and `g` undoes `f` while `p`
undoes `h`, then `g(p(h(f(a)))) = g(f(a)) = a`, and the other direction is
the same calculation from the other end. Subtracting seven first, in the
§6.10 warning, applies the wrong inverse to the outer step.

### N9.12 Mathematics, tests, and implementations

Keep three subjects apart when you report Proposition N9.5.

The **mathematical claim** is the proposition: for every finite length and
every equal-length pair, the mask map is a bijection. Its evidence is
Proposition 3.1 together with Proposition N9.4. The length is not fixed
at eight.

A **finite exhaustive test** is a proof only of what its set contains. The
`65536` byte pairs are `B × B`. A correct examination of every pair
establishes the identity on that set. It is the special case `n = 8` of
the mathematical claim, not the claim itself. A run of ten pairs
establishes ten pairs. Calling the ten a subset of `B × B` is true and
useless unless you also say that the property was not checked on the
complement.

An **implementation observation** is about a program and a machine.
Chapters 5 and 6 already give the form
`spec f(x: Word[8]) -> Word[8]` with a body such as `x ^ k` or `x * 2`.
Acceptance of either body is a fact about the grammar and the types those
chapters stated. Both bodies denote functions from `B` to `B` under the
reading in §6.4 and Example N9.8. Only the XOR body is bijective for each
fixed `k`. The doubling body is `d`. A checker that accepts the file has
not been asked to test injectivity.

This lesson adds no Orange listing and reports no new compiler run. The
distinction just made is about the functions already defined. It is not
evidence that a particular binary accepted a particular file on a
particular day.

### N9.13 Statements, predicates, and implication

A **statement**, in this lesson, is a sentence to which Assumption N9.1
applies: it is true or false. A **predicate** is a sentence with one or
more free names, which becomes a statement once each free name is either
replaced by an element or bound by a quantifier. The letter in a predicate
is a placeholder, not an implicit “every.”

**Example N9.9 — Binding `x`.** The sentence `x ∈ B` is a predicate.
Substituting `255` produces the true statement `255 ∈ B`. Substituting
`256` produces the false statement `256 ∈ B`. Until you do one of those,
or bind `x`, you do not have a truth value to defend.

**Definition N9.15 — Implication.** If `P` and `Q` are statements, the
**implication** `P ⇒ Q` is the statement read “if `P`, then `Q`.” Its
truth value is fixed by this table.

| `P` | `Q` | `P ⇒ Q` |
| --- | --- | --- |
| `0` | `0` | `1` |
| `0` | `1` | `1` |
| `1` | `0` | `0` |
| `1` | `1` | `1` |

The implication is false only in the third row: the hypothesis holds and
the conclusion does not. In particular, a false hypothesis makes the
implication true whether `Q` is true or false. That is a rule of this
table, not a claim about causes. `P ⇒ Q` does not say that `P` is true,
and it does not say that `Q` is true.

**Example N9.10 — A mask bit that is zero.** Fix one bit position. Let `P`
say “the mask bit in this position is `1`.” Let `Q` say “the data bit in
this position flips when XOR uses that mask.” If the mask bit is actually
`0`, then `P` is false, so `P ⇒ Q` is true even though a zero mask bit
does not flip the data bit. The implication did not claim that a flip
happened. It claimed that a flip happens in the case where the mask bit
is one. Chapter 3’s cancellation proof needed exactly that case split.
The implication symbol is how the split’s inactive case stays true
without pretending the inactive case flipped a bit.

Proposition N9.1 is the subset form of the same row. “If `x ∈ ∅`, then
`x ∈ T`” has a hypothesis that never holds.

The **converse** of `P ⇒ Q` is `Q ⇒ P`. The table does not make these
equal. `1 ⇒ 0` is false while `0 ⇒ 1` is true. Swapping an implication
because the English sounds reversible is a different statement. Later,
the zero mask separates a bijective map from a nonzero mask: one
implication can hold while its converse fails.

### N9.14 Two uses of the word equivalence

**Definition N9.16 — Logical equivalence.** Statements `P` and `Q` are
**logically equivalent**, written `P ⇔ Q`, when `P ⇒ Q` and `Q ⇒ P` are
both true. Equivalently, `P` and `Q` have the same truth value.

This is a relation on statements. An equivalence relation, Definition
N9.9, is a relation on the elements of a chosen set. Congruence modulo
`m` is the second kind. `P ⇔ Q` is the first kind. English uses
“equivalent” for both. The symbols are different so that a proof can say
which one it means.

If you need a chain of equivalences as a single argument, transitivity of
“has the same truth value” is the reason a chain is legitimate: equality
of truth values is reflexive, symmetric, and transitive on the two-element
set of truth values. That tiny equivalence relation is not congruence and
not the definition of `⇔`. It is why a chain of `⇔` steps preserves truth.

### N9.15 Quantifiers

**Definition N9.17 — Quantifiers.** Let `P(x)` be a predicate and `S` a
set. The **universal** statement `∀ x ∈ S, P(x)` is true when `P(a)` is
true for every element `a` of `S`. The **existential** statement
`∃ x ∈ S, P(x)` is true when `P(a)` is true for at least one element `a`
of `S`. The set `S` is the **domain of quantification**. Omitting it is
not allowed in a final claim. A pronoun such as “every string” must
already have named the set of strings.

On the empty set, the universal statement is true: there is no element
that fails `P`. The existential statement is false: there is no element
that satisfies `P`. This matches Proposition N9.1 and refuses a fake
witness.

Order is part of the statement. Let `S` be the set of bit strings of one
fixed length `n ≥ 1`, so that a string with a one-bit exists. Let `0`
denote the all-zero string of that length.

**Example N9.11 — Four readings of “XOR fixes a string.”**

1. `∀ x ∈ S, ∀ k ∈ S, (x ⊕ k) ⊕ k = x`. True, by Proposition 3.1. The
   domain is every pair, not one example.
2. `∃ k ∈ S, ∀ x ∈ S, x ⊕ k = x`. True. Take `k = 0`. XOR with zero leaves
   each bit as it was, by the two truth-table rows in which the mask bit
   is `0`.
3. `∀ x ∈ S, ∃ k ∈ S, x ⊕ k = x`. True. For each `x`, the same witness
   `k = 0` works. The witness is allowed to depend on `x`. Here it happens
   not to.
4. `∃ x ∈ S, ∀ k ∈ S, x ⊕ k = x`. False. Take any candidate `x`. Let `k`
   be the string with a single `1` in one position and `0` elsewhere,
   which exists because `n ≥ 1`. Then `x ⊕ k` differs from `x` in that
   position, so this `x` is not fixed by every `k`.

Statement 2 says one mask works for every string. Statement 4 says one
string is fixed by every mask. They are not interchangeable. Statement 3
lets the mask depend on the string. Statement 2 does not.

A witness for an existential claim must be an element of the stated
domain. “Some longer string” does not prove an existential claim about
`S`.

### N9.16 Negation of a quantified statement

**Proposition N9.11 — Negating quantifiers.** Let `P(x)` be a predicate
on a set `S` available in this lesson.

1. The negation of `∀ x ∈ S, P(x)` is logically equivalent to
   `∃ x ∈ S, ¬ P(x)`.
2. The negation of `∃ x ∈ S, P(x)` is logically equivalent to
   `∀ x ∈ S, ¬ P(x)`.

Here `¬` is NOT applied to a statement, as Chapter 3 used NOT on a truth
value.

**Proof.** Consider the universal claim. If it is false, then it is not
the case that every element satisfies `P`. By Assumption N9.1 and the
definition of `∀`, some element of `S` fails `P`, which is the existential
claim. If the existential claim is true, that failing element is a reason
the universal claim is false. If `S` is empty, the universal claim is
true, its negation is false, and the existential claim of a failure is
also false. The two sides agree.

The existential half is the universal half applied to `¬ P`, together
with `¬ ¬ P(x)` having the same truth value as `P(x)`. Negating twice
returns the original truth value because NOT swaps the two values in
Chapter 3’s table.

The negation of “every byte doubles to an even value” is “some byte
doubles to an odd value,” not “every byte doubles to an odd value.” The
first of those three is true, by the parity half of Proposition N9.6. Its
negation is therefore false. The “every output is odd” sentence is also
false, and a false sentence is not automatically the negation of a true
one. Two false sentences can sit side by side. Exercise 3.10 asked for
this distinction in words. The proposition is the general rule.

### N9.17 Proof by cases

**Definition N9.18 — Proof by cases.** A **proof by cases** shows that a
set `S` is the union of finitely many subsets `S1` through `Sm`, that
those subsets are pairwise disjoint, and that the desired statement holds
on each subset. The subsets are the **cases**. From the pieces, the
statement holds on `S`, because every element of `S` lies in exactly one
case.

The disjointness condition keeps you from hiding an element in two cases
and checking neither carefully. If you prefer overlapping cases, you must
still show each element is in at least one checked case. Disjointness is
the cleaner default.

**Proposition N9.12 — OR through XOR and AND.** For bits `a` and `b`,
`a OR b` equals `(a XOR b) XOR (a AND b)`.

**Proof.** The domain is `{0, 1} × {0, 1}`, which has four elements by
Proposition N9.2. Take those four pairs as the cases.

| `a` | `b` | `a OR b` | `a XOR b` | `a AND b` | `(a XOR b) XOR (a AND b)` |
| --- | --- | --- | --- | --- | --- |
| `0` | `0` | `0` | `0` | `0` | `0` |
| `0` | `1` | `1` | `1` | `0` | `1` |
| `1` | `0` | `1` | `1` | `0` | `1` |
| `1` | `1` | `1` | `0` | `1` | `1` |

Each row uses one line of the Chapter 3 tables. The last column equals
the OR column in every row. The cases cover the product. That is the
proof.

Four rows establish the identity for bits. They do not, by themselves,
establish a new identity for bytes. A byte extension needs the extra
sentence Chapter 3 already required for bitwise operations: apply the bit
identity at each position. Say that sentence if you want the byte claim.
The table does not whisper it.

This case proof is not the cancellation proof. Cancellation classifies
one mask bit as `0` or `1` and tracks a data bit. Here the cases are
input pairs of a different identity. Same pattern, different statement.

### N9.18 Proof by contradiction

**Definition N9.19 — Proof by contradiction.** To prove a statement `Q`,
a **proof by contradiction** proves the implication `¬ Q ⇒ F`, where `F`
is a statement already known to be false. By the implication table, the
only way for that implication to be true with a false conclusion is for
`¬ Q` itself to be false. By Assumption N9.1, `Q` is then true.

The known falsehood has to be a real one: an element both even and odd, a
cardinality both equal to `n` and at least `n + 1`, or two names for one
function value that the definition says cannot differ. A surprising
sentence is not yet a falsehood.

**Proposition N9.13 — Doubling has no two-sided inverse on the bytes.**
There is no function `g` from `B` to `B` such that `g(d(x)) = x` for
every `x` in `B` and `d(g(y)) = y` for every `y` in `B`.

**Proof.** Suppose `g` were such a function. The equation `g(d(x)) = x`
says `g` is a left inverse of `d`. Proposition N9.4 would make `d`
injective. Proposition N9.6 says `d` is not injective. A function cannot
be both injective and not injective. The supposition is false.

The short form that names the collision: `g(d(0)) = 0` and
`g(d(128)) = 128` would require `g(0) = 0` and `g(0) = 128`. That is the
same contradiction, written at the element where injectivity fails.

One counterexample still refutes a universal claim, as Exercise 3.5 said
about AND. Contradiction is the pattern you are using when the universal
claim is “every element has a preimage” or “a left inverse exists,” and
the counterexample is the thing that cannot exist together with the
supposition.

### N9.19 The contrapositive

**Definition N9.20 — Contrapositive.** The **contrapositive** of
`P ⇒ Q` is `¬ Q ⇒ ¬ P`.

**Proposition N9.14 — An implication matches its contrapositive.** For
any statements `P` and `Q`, the implication `P ⇒ Q` is logically
equivalent to `¬ Q ⇒ ¬ P`.

**Proof.** Compute both sides on the four rows. NOT swaps `0` and `1`.

| `P` | `Q` | `P ⇒ Q` | `¬ Q` | `¬ P` | `¬ Q ⇒ ¬ P` |
| --- | --- | --- | --- | --- | --- |
| `0` | `0` | `1` | `1` | `1` | `1` |
| `0` | `1` | `1` | `0` | `1` | `1` |
| `1` | `0` | `0` | `1` | `0` | `0` |
| `1` | `1` | `1` | `0` | `0` | `1` |

The third column and the last column agree. By Definition N9.16 the
statements are logically equivalent. The table is a finite exhaustive
proof on the set of truth assignments, which has four elements. It is not
a proof about byte values until `P` and `Q` are statements about them.

The converse is not in the table’s conclusion. Compare row `P = 0`,
`Q = 1`: the implication is true and the converse `Q ⇒ P` is `1 ⇒ 0`,
which is false.

**Example N9.12 — Odds are missed, in the easier direction.** Let `P(y)`
say `y ∈ B` and `y` is odd. Let `Q(y)` say `y` is not in the image of
`d`. The claim `∀ y ∈ B, P(y) ⇒ Q(y)` has contrapositive
`∀ y ∈ B, ¬ Q(y) ⇒ ¬ P(y)`, which reads: if `y` is in the image of `d`,
then `y` is even. That contrapositive is the parity half of Proposition
N9.6, and it is the direction the algebra gives directly. Proposition
N9.14 moves the truth of that direction back to the original sentence.

**Example N9.13 — A true implication with a false converse.** Let `P` say
“`k` is not the all-zero string” and `Q` say “`f(x) = x ⊕ k` is
bijective,” for `k` in the set `S` of Proposition N9.5. Then `P ⇒ Q` is
true because `Q` is true for every `k`, zero included, so the implication
table’s conclusion column is true in both rows where we might have cared.
The converse `Q ⇒ P` is false when `k` is zero: `Q` is true and `P` is
false. A nonzero mask is sufficient for bijectivity only in the weak
sense that the implication holds. It is not necessary. The zero mask
remains the witness.

### N9.20 Induction

Assumption N9.4 is the whole rule. A proof that uses it has two
obligations and one announcement. The **base case** is `P(0)`. The
**inductive step** proves `P(n) ⇒ P(n + 1)` for an arbitrary nonnegative
integer `n`. Inside that step, `P(n)` is the **inductive hypothesis**.
You may use it only after saying that you are proving the step, and only
as a hypothesis of that implication. You may not treat `P(n)` as something
already proved for every `n` while you are still proving the step.

**Proposition N9.15 — How many bit strings.** Let `C(n)` be the set of bit
strings of length `n`, for each nonnegative integer `n`. Then
`|C(n)| = 2ⁿ`.

**Proof.** Base case: Exercise 2.6 already recorded that there is exactly
one string of length zero, the empty string. So `|C(0)| = 1 = 2⁰`.

Inductive step: fix `n ≥ 0` and assume `|C(n)| = 2ⁿ`. Every string of
length `n + 1` is a bit, `0` or `1`, followed by a string of length `n`.
Those two families are disjoint because the first symbols differ, and
every longer string falls into one of them. Each family is in one-to-one
correspondence with `C(n)`. Therefore
`|C(n + 1)| = 2ⁿ + 2ⁿ = 2ⁿ · 2 = 2ⁿ⁺¹`.

Assumption N9.4 yields the claim for every nonnegative integer `n`.

Exercise 2.9 asked for the successor construction in words. The argument
above is that construction organized as a base case and a step. The count
`|B| = |C(8)| = 256` is the case `n = 8`. Chapter 2’s count of eight-bit
strings is this case, not a rival theorem.

Induction does not say: the claim held for the first few values I tried,
so it holds in general. That is a sample.

**Example N9.14 — Where a fake induction breaks.** Let `P(n)` say: for
every byte value `v`, a left shift by `n` positions returns `v`. The
intended reading of “shift by `n`” for `n ≥ 8` is the mathematical one
from Chapter 6’s description of a left shift: move `n` places toward the
most significant end, discard bits that leave, and insert zeros. This
example is about that operation on eight positions. It is not a claim
about which Orange literals the compiler accepts.

`P(0)` is true: moving zero places changes nothing. The inductive step is
false. `P(0)` holds and `P(1)` does not. The byte `0x81`, binary
`10000001`, shifts left by one to `00000010`, which is not `0x81`. An
implication `P(0) ⇒ P(1)` with a true hypothesis and a false conclusion is
the false row of Definition N9.15. Assumption N9.4 therefore gives no
license to conclude `P(n)` for every `n`. Checking `n = 0` was necessary
and not sufficient.

### N9.21 Invariants

**Definition N9.21 — Invariant.** Let `f` be a function from `S` to `S`.
A predicate `I` on `S` is an **invariant** of `f` when
`∀ x ∈ S, I(x) ⇒ I(f(x))`.

If you apply `f` any number of times, an invariant that holds at the start
holds at every later iterate. That extension is an induction. Let `P(n)`
say `I(fⁿ(x))`, where `fⁿ` means `n` applications and `f⁰` means the
identity. The base case is `I(x)`. The step is the invariant applied to
the element `fⁿ(x)`. This is often the useful shape: the inductive
statement is the invariant after `n` steps, not a new idea at each `n`.

**Example N9.15 — Ones under a rotation.** Number the eight bit positions
of a byte `0` through `7`, with position `0` the least significant bit,
the rightmost bit in the writing of Chapter 6. Let `L` be left rotation
by one position: the bit in position `i` moves to position `(i + 1)`
modulo `8`, using the representative in `0` through `7`. Let `R` be right
rotation by one: the bit in position `i` moves to position `(i - 1)`
modulo `8`. For `i = 0`, that representative is `7`.

Apply `L` and then `R`. A bit that starts at `i` sits at `(i + 1) mod 8`
and then at `((i + 1) - 1) mod 8 = i`. Every bit returns, and no bit value
is rewritten. So `R(L(v)) = v` for every byte `v`. The other order returns
each position as well. Thus `L` and `R` are two-sided inverses. By
Proposition N9.4, `L` is bijective on `B`.

Let `I(v)` say nothing about values; the invariant we want is equality of
a count. Let `N(v)` be the number of positions among `0` through `7` whose
bit in `v` is `1`. Left rotation sends positions to positions bijectively:
distinct positions land in distinct positions, and every position is hit.
A position contributes to `N(v)` exactly when its bit is `1`, and that
same bit arrives at exactly one new position. So `N(L(v)) = N(v)` for
every byte `v`. The predicate `N(v) = c`, for a fixed integer `c`, is an
invariant of `L`. Iterating, `N` is unchanged by any number of left
rotations. The same holds for `R`.

The trace of `0x81` in §6.7 is one element of this set: two ones before,
two ones after a left rotation by one. The argument here is the general
one. The trace is not.

A left **shift** by one is not an invariant of `N`. It discards the bit in
position `7` and writes `0` into position `0`. For `v = 0x81`, binary
`10000001`, `N(v) = 2`. The shift produces `00000010`, and `N` becomes
`1`. Also, preservation on a single input does not prove an invariant.
The byte `0x01` shifts to `0x02`, and both have `N = 1`. That pair is
compatible with an invariant and does not establish the universal
quantifier in Definition N9.21.

Doubling fails the same predicate. `0x80` has `N = 1`. `d(0x80) = 0`, and
`N(0) = 0`. Staying inside `B` is true of `d` and is the wrong invariant
to quote if the question was whether ones, or injectivity, survive.
“The result is still a byte” restates the codomain. It does not state an
invariant that implies a two-sided inverse.

N7 will eventually have a syntax for repetition in Orange. This section
does not use it. The invariant is a statement about a function on a set.
It does not become true or false according to which syntax a program
uses to iterate.

### N9.22 Finite exhaustive arguments

**Proposition N9.16 — A complete finite check.** Let `S` be finite, and
let `P(x)` be a predicate on `S`. Suppose a list contains each element of
`S` at least once, and suppose that for each listed element the
corresponding statement `P(a)` has been established. Then
`∀ x ∈ S, P(x)`.

**Proof.** Take any `a` in `S`. It occurs on the list. The corresponding
establishment gives `P(a)`. The element was arbitrary. Definition N9.17
gives the universal claim.

The proposition is short because the work is in the two suppositions. You
owe a reason that the list is complete, and you owe a correct treatment
of each listed element. A missing row falsifies the first. A row whose
arithmetic is wrong falsifies the second. Either fault blocks the
proposition, even if the set has four elements and the table looks
authoritative.

Proposition N9.12 is this pattern on a four-element set. Proposition
N9.14 is the same pattern on the four truth assignments. The byte
cancellation survey, if each of the `65536` pairs is actually checked and
checked correctly, is this pattern on `B × B`. It yields
`∀ (x, k) ∈ B × B, (x ⊕ k) ⊕ k = x`. It does not yield the same sentence
with `B` replaced by the set of sixteen-bit strings.

That larger product is why a different pattern exists. There are `2¹⁶`
strings of length `16`, and `(2¹⁶) · (2¹⁶) = 2³² = 4294967296` pairs.
Proposition N9.2 does the counting. A hand list of that product is not the
proof you want to write, and an unchecked assertion that a machine walked
the whole product is an implementation claim, which brings the machine
into the assumptions. Proposition 3.1 covers length `16` because its
reasoning applies at each position, for an arbitrary finite length.
Proposition N9.15’s induction is the pattern that scales a one-step
counting relation to every length. Exhaustion is the pattern that scales
only as far as the set you actually cover.

A sample of `1000` byte pairs is a test. If those pairs were computed
correctly, you have `1000` truths. You do not have Proposition N9.16’s
hypothesis, because `1000` is not `|B × B|`.

### N9.23 The edge of the claim

You can now say a function is bijective, or say it is not, and point to
the lemma or the collision that decides the matter. You can negate
“every,” and you can refuse a converse that the table does not give you.
You can match a proof pattern to the set: cases or a full table when the
set is small and listed, contradiction when a supposition collides with a
cited fact, contrapositive when the swapped form is the one the algebra
computes, induction when the set is the nonnegative integers, an invariant
when a step preserves a predicate you still need later.

You may not say that XOR is invertible and stop. Section N9.11 fixed the
function that is bijective and the function that is not. You may not say
that a passing `check` of `x * 2` shows a bijection. You may not say that
this lesson made a cryptosystem provably secure. Diffie and Hellman’s
sentence remains their 1976 statement of a direction. The sets and proof
patterns here are part of the language in which such a direction can
later be stated without a missing quantifier. They are not the security
argument.

### Work at the desk

**Exercise N9.1 — Decide membership.** For the byte set `B`, classify
`0`, `1`, `2`, `255`, `256`, and `-1`. State the membership rule you use.

**Exercise N9.2 — Do not count a roster’s ink.** What is
`|{1, 0, 1}|`? Is the bit string `10` equal to that set? Is the bit
string `11` equal to the set described by the roster `{1, 1}`?

**Exercise N9.3 — Subsets.** Is `∅ ⊆ B`? Is `{256} ⊆ B`? How many members
does `E`, the set of even byte values, have? Explain why a list of five
masks you tested is not, by itself, a proof of a universal claim about
`B`.

**Exercise N9.4 — Count the pairs.** List every member of
`{0, 1} × {0, 1}`. How many members does `B × B` have, and why is that
the number of byte-and-mask pairs in the Chapter 3 survey?

**Exercise N9.5 — One surviving property.** On the integers, `a` is
related to `b` when `a - b = 1` or `b - a = 1`. Is the relation
reflexive? Symmetric? Transitive? If a property fails, give elements
that show the failure.

**Exercise N9.6 — Function or relation.** Using modulus `256`, is the
rule “send each integer to its representative in `B`” a function from
the integers to `B`? Is congruence modulo `256` a function from the
integers to the integers? Which definition decides each answer?

**Exercise N9.7 — Name the image.** For doubling `d` on `B`, show a
collision, show one missed value, and describe the image exactly.

**Exercise N9.8 — Undo the mask map, not a mood.** Using Proposition N9.4
and Proposition 3.1, prove that `f(x) = x ⊕ k` is bijective on the
strings of length `n`. Then take `k` all zeros and say what bijectivity
does not establish.

**Exercise N9.9 — Three subjects.** Take the claim
`(x ⊕ k) ⊕ k = x` for finite equal-length bit strings. What does a
correct check of ten pairs establish? What does a correct check of all
`65536` byte pairs establish? What does Proposition 3.1 establish? What
does one successful evaluation of one Orange call establish?

**Exercise N9.10 — Bind the name.** Is `x ∈ B` a statement? Give one
substitution that makes it true and one that makes it false.

**Exercise N9.11 — The inactive case.** One mask bit is `0`. Let `P` say
that this mask bit is `1`, and let `Q` say that this data bit flips.
What is the truth value of `P ⇒ Q`, and which row of the implication
table did you use?

**Exercise N9.12 — Order the doubles.** Let `d` be doubling on `B`. For
each statement, say true or false and give the reason.

1. `∀ x ∈ B, d(x)` is even.
2. `∃ x ∈ B, d(x) = 1`.
3. `∀ y ∈ B, ∃ x ∈ B, d(x) = y`.
4. `∃ x ∈ B, ∀ y ∈ B, d(x) = y`.
5. `∀ x ∈ B, ∃ y ∈ B, y = d(x)`.

**Exercise N9.13 — Negate the sentence you mean.** Negate
`∀ x ∈ B, d(x)` is even. Negate `∃ x ∈ B, d(x) = 1`. Say which of the
two original statements is true, which negation is true, and why
`∀ x ∈ B, d(x)` is odd is not the negation of the first statement.

**Exercise N9.14 — Four cases, not a slogan.** Prove Proposition N9.12
by cases. State the set the cases partition. State what the proof does
not establish about bytes until you add a further sentence.

**Exercise N9.15 — A collision that blocks an inverse.** Prove there is
no two-sided inverse of `d` on `B`. Put the false supposition first and
name the two facts it cannot survive.

**Exercise N9.16 — Swap the sides, not the quantifier’s meaning.** Write
the contrapositive of “if `y ∈ B` is odd, then `y` is not in the image
of `d`.” Which direction does the parity calculation prove directly?
Separately, explain why “if `k` is nonzero, then XOR with `k` is
bijective” can be true while its converse is false.

**Exercise N9.17 — Base, step, and a broken step.** From the proof of
Proposition N9.15, compute `|C(0)|` and `|C(3)|`. For the shift claim in
Example N9.14, show that the base case holds and that the inductive step
fails at `n = 0`. Use `0x81` as the witness for that failure.

**Exercise N9.18 — What the step preserves.** Let `N` be the number of
ones. Compute `N(0x81)`, `N` after one left rotation of `0x81`, and `N`
after one left shift of `0x81`. Compute `N(0x80)` and `N(d(0x80))`.
Which of these operations has `N` as an invariant on all of `B`, and
why does `0x01` shifting to `0x02` not prove that a shift preserves `N`?

**Exercise N9.19 — Completeness is an obligation.** How many ordered
pairs of sixteen-bit strings are there? Why does that number not refute
Proposition 3.1 at length `16`? Why can a four-row table still fail to
be a proof?

**Exercise N9.20 — Say which function.** Rewrite “XOR is invertible” as
a quantified statement about a named function, a domain, and a codomain,
in a form that Proposition N9.5 makes true. Then give a second reading,
also about XOR, that is false. Do not repair the false reading by
changing the words until it becomes the true one. The point is to keep
both sentences visible.

## Worked answers: N9

These answers cover the exercises above. They are not substitutes for
your attempt. When an answer differs from yours, compare the definition
used at the first differing step.

**N9.1.** The rule is membership in
`{ n : n is an integer and 0 ≤ n ≤ 255 }`. Belong: 0, 1, 2, and 255.
Do not belong: 256 and -1.

**N9.2.** The set has two elements. The string `10` is not that set.
The string `11` is not the one-element set written `{1, 1}`.

**N9.3.** The empty set is a subset. `{256}` is not a subset. The even
byte values number 128. Five tested masks are five tests. A universal
claim about `B` needs every element of `B`, or a proof that covers `B`,
not a list of five.

**N9.4.** The pairs are `(0, 0)`, `(0, 1)`, `(1, 0)`, and `(1, 1)`.
There are 4 pairs. The byte pair set has 65536 elements, because
`|B × B| = 256 · 256` and the Chapter 3 survey uses one pair per
byte value and mask.

**N9.5.** Not reflexive: `0 - 0 = 0`. Symmetric: the two equations swap
when the names swap. Not transitive: 0 and 2 differ by 2, while each
differs by 1 from 1.

**N9.6.** The representative rule is a function. Congruence on the
integers is not a function. Section 5.4 requires exactly one output;
§6.3 gives one representative; congruence of `19` with both `3` and
`19` gives two.

**N9.7.** Both 0 and 128 send to 0. The value 1 is missed. The image is
the 128 even byte values.

**N9.8.** Proposition 3.1 says `f(f(x)) = x` for every string `x` of
length `n`. That equation is both a left-inverse equation and a
right-inverse equation for `f` with itself. Proposition N9.4 yields
injectivity and surjectivity, hence a bijection. The map is bijective.
The all-zero mask is one such bijection and hides nothing.

**N9.9.** Ten pairs establish ten pairs. All 65536 byte pairs establish
the byte domain. Proposition 3.1 establishes every finite equal length.
One evaluation establishes one execution of one input.

**N9.10.** It is a predicate, not a statement, until `x` is bound.
`255 ∈ B` is true. `256 ∈ B` is false.

**N9.11.** The implication is true. The hypothesis is false, so the row
is `P = 0`, and both rows with `P = 0` have implication value `1`.

**N9.12.** In the order asked: true; false; false; false; true. The
first holds because every value `2x - 256q` is even. The second fails
because `1` is odd. The third fails because odd `y` are missed; it is
surjectivity of `d`. The fourth fails because one output cannot equal
every byte; already `d(x) = 0` and `d(x) = 1` cannot hold together.
The fifth holds because `d` is a function into `B`: take `y = d(x)`.

**N9.13.** The negation of the universal claim is an existential claim,
and that negation is false. The negation of the existential claim is a
universal claim, and that negation is true. The original universal
claim is the true one. The sentence “every double is odd” is a
different universal claim. It is false, and it is stronger than one
counterexample. The negation of “every double is even” is “some double
is odd.”

**N9.14.** The cases are the four members of `{0, 1} × {0, 1}`. Direct
calculation gives, in order `00`, `01`, `10`, `11`, the common values
`0`, `1`, `1`, `1` for OR and for `(a XOR b) XOR (a AND b)`. All four
rows agree. The proof establishes the bit identity. A byte claim needs
the further sentence that the bit identity applies at each position.

**N9.15.** Suppose `g` is a two-sided inverse of `d` on `B`. Then `g` is
a left inverse, so Proposition N9.4 makes `d` injective, which
contradicts Proposition N9.6. No such inverse exists: it would have to
send 0 to both 0 and 128.

**N9.16.** The contrapositive is: if a byte is in the image, then it is
even. The parity calculation proves that direction directly: an attained
value `2x - 256q` is even. Proposition N9.14 transfers it to the odd-miss
wording. The converse of “a nonzero mask implies a bijective XOR map” is
false, because the zero mask is a bijection. The forward implication is
true because every mask, zero or not, gives a bijection, so a nonzero
mask does too.

**N9.17.** From the base case, `|C(0)| = 1`, and `2⁰ = 1`. The step
doubles the count, so `|C(1)| = 2`, `|C(2)| = 4`, and `|C(3)| = 8`.
Thus `2^0 = 1` and `2^3 = 8`. For the shift claim, a shift by zero
returns every byte, so the base case holds. The inductive step fails
when n = 0, since a left shift by one does not preserve 0x81: `10000001`
becomes `00000010`.

**N9.18.** `0x81` is `10000001`, so `N(0x81) = 2`. One left rotation
yields `00000011`, which still has two ones. One left shift yields
`00000010`, which has one. `0x80` is `10000000`, so `N(0x80) = 1`, while
`d(0x80) = 0` and `N(0) = 0`. Rotation has `N` as an invariant on all of
`B`, because it permutes the eight positions and moves each one-bit onto
exactly one position. The pair `0x01` and `0x02` shows one input whose
ones-count happens to survive a shift. An invariant requires the
implication for every byte. The byte `0x81` is a counterexample for the
shift.

**N9.19.** There are 4294967296 pairs of 16-bit strings. That count does
not refute Proposition 3.1, because the proposition’s proof is a
per-position argument for an arbitrary finite length, not a claim that
someone listed `2³²` pairs. A four-row table with a wrong row is not a
proof. Completeness of the row set and correctness of each row are both
required.

**N9.20.** A true reading: for every finite length `n` and every bit
string `k` of that length, the function `f` from the set of length-`n`
bit strings to itself given by `f(x) = x ⊕ k` is bijective, and `f` is
its two-sided inverse. A false reading is: from the output alone, both
inputs of two-variable XOR are uniquely determined. For `n ≥ 1` the pairs
`(0, 0)` and `(k, k)` with nonzero `k` share the all-zero output and are
different pairs.

## Sources and epigraph record

**[S7] Whitfield Diffie and Martin E. Hellman.** “New Directions in
Cryptography.” *IEEE Transactions on Information Theory*, vol. IT-22,
no. 6, November 1976, pp. 644–654. The epigraph is an eleven-word excerpt
from the first paragraph of §I, on p. 644. The full sentence in that
paragraph is: “At the same time, theoretical developments in information
theory and computer science show promise of providing provably secure
cryptosystems, changing this ancient art into a science.” The excerpt is
the contiguous clause beginning at “providing” and ending at “science.”
No word was substituted.

Wording and page header were checked on 2026-10-05 against the text
extraction of the PDF hosted by Martin Hellman at
<https://ee.stanford.edu/~hellman/publications/24.pdf>.
The extraction renders the page’s opening small capitals with a gap
(“W E STAND TODAY”); the epigraph is not that opening sentence and does
not depend on repairing it. The consulted file is the author-hosted PDF,
not a separate inspection of a physical journal issue. No translation is
involved. The sentence states a 1976 research direction. It is not a
theorem of this lesson and not an endorsement of any construction.

Epigraph verification establishes wording and attribution, not
publication-rights clearance. The definitions, proofs, examples, and
exercises are original drafting for this book.

## Evidence boundary

Lesson N9 adds definitions, propositions, twenty exercises, and worked
answers. It adds no Orange listing. The nine listings and the Rust
integration test attached to Chapters 4–6 are unchanged. No new compiler
run is reported.

The Python checks that accompany this lesson recompute the finite counts,
truth tables, doubling image, rotation and shift examples, and exercise
structure. They do not execute Orange, do not establish a cryptographic
security claim, and do not constitute independent review.

The drafting of this lesson is AI-assisted with Grok 4.7 in Cursor, at
Chase Bryan’s direction, 2026-10-05. Owner review of the lesson is
pending. The project’s Current, Directed, Proposed, and Future
distinctions, the license boundary, and the manuscript’s existing source
disclosures remain in force. N7 and N8 are not written here. Manuscript
Chapters 1–17 are not renumbered here.
