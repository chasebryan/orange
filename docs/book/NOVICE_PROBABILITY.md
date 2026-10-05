# The Orange Book

By Chase Bryan

## Part 1, The Novice

N10: Count What You Do Not Know. Draft 2026-10-05.

Continue from [Words Have Edges](NOVICE_PROGRAMMING.md#chapter-6-words-have-edges).
This lesson is **N10**. The locked label is N10. It is not a manuscript
chapter numeral. The original manuscript keeps its own chapter numbers.
N10 is not one of those chapters, and it is not N7, N8, or N9.

N7 names intermediate steps in Orange. N9 gives collections and
assignments their formal names. This lesson uses neither of those
vocabularies. A sentence that points at N9's word is marked as a preview
of that lesson, not a substitute for it.

You already know, from
[Chapter 2, §2.4](NOVICE_OPENING.md#24-count-the-possibilities-before-choosing-one),
that `n` unrestricted bit positions allow `2^n` strings, and from
[§2.7](NOVICE_OPENING.md#27-a-short-audit-of-a-byte) that the width of a
byte is not the same fact as how the byte was chosen. This lesson uses
those facts. It does not teach them again.

**Mathematical claim.** A result below is a claim about stated integers,
fractions, or a stated finite list of weights.

**Finite check.** The reference test recomputes the ledger at the end from
the same rules. A finite check confirms those values. It does not turn one
example into a rule for every list.

**Implementation behavior.** An Orange listing denotes the function its
body writes, on the inputs evaluation actually runs. A silent check means
the source was well-formed. A printed value means that run produced that
value. A passing test means the test's `Bool` was true. None of those is
the quantifier in a proposition, and none is a security claim. [C1]

The named surface of these listings is `Int`, `Bool`, comparisons,
Euclidean `/`, `let`, fixed-length arrays, a bounded `for` whose
bounds are integer literals, and `test`. There is no fraction type. The
expression `1 / 2` is Euclidean division of integers. Its value is `0`,
not the fraction one half. Every rational comparison a listing asks the
compiler to check is an integer identity. `a/b = c/d` exactly when
`a*d = b*c`, by Definition N10.1, or the sum is written on a common
denominator. A comparison of two expressions that are only literals is
rejected: a literal does not choose `Int` by itself, so the listings pass
the integers through a parameter of type `Int`. There is no operation
that draws an outcome. A listing counts and compares. It does not sample.

Not yet expressible on that named surface, and not given a stand-in
listing: a fraction as a value; a weight the language attaches to an
outcome; a loop of no steps; a quantifier over every positive integer or
every event; an exponential function; and a weighted substitute for
`log2`. Where a proposition needs one of those, the proof is the claim,
and the listing checks the integers the section names. The section says
which part is which.

## N10: Count What You Do Not Know

> “Although it is always possible in principle to determine these solutions (by trial of each possible key for example), different enciphering systems show a wide variation in the amount of work required.”
>
> — Claude E. Shannon, *Communication Theory of Secrecy Systems* (1949),
> §21, p. 703 of the retypeset copy. [S8]

### N10.1 Equal parts

Take a unit and cut it into two pieces of the same size. One of those
pieces is a half. Cut another unit into three pieces of the same size.
One of those pieces is a third. The half and the third are not the same
piece, and adding the numerators and the denominators as if they were
counts of the same kind produces `2/5`, which is neither piece and not
their sum.

**Definition N10.1 — Fraction.** Let `a` be an integer and let `b` be a
positive integer. The fraction `a/b` is `a` pieces, each of size one-`b`th
of a unit. The integer `a` may be negative, zero, or positive. A negative
numerator names that many pieces owed, rather than held. Two fractions
`a/b` and `c/d` are equal exactly when `a*d = b*c`.

The equality test uses only integer multiplication, which you already
have. It does not ask the two numerals to look the same. `1/2` and `2/4`
are equal because `1*4 = 2*2`. They are different numerals for one
fraction.

**Proposition N10.1 — Scaling.** If `k` is a positive integer, then
`a/b = (k*a)/(k*b)`.

**Proof.** Cross-multiply: `a*(k*b) = k*a*b` and `b*(k*a) = k*a*b`. The
two products are the same integer, so Definition N10.1 says the fractions
are equal.

**Definition N10.2 — Sum and product of fractions.** The sum of `a/b` and
`c/d` is `(a*d + b*c)/(b*d)`. The product is `(a*c)/(b*d)`. If `c` is not
zero, the quotient `(a/b)/(c/d)` is `(a*d)/(b*c)`.

These are definitions of operations on numerals. They would be useless if
replacing a fraction by an equal fraction changed the sum. The next
proposition says that it does not.

**Proposition N10.2 — Sums and products respect equality.** Suppose
`a/b = a'/b'` and `c/d = c'/d'`. Then the two sums are equal, and the two
products are equal.

**Proof.** The hypotheses say `a*b' = b*a'` and `c*d' = d*c'`. For the
sums, compare `(a*d + b*c)/(b*d)` with `(a'*d' + b'*c')/(b'*d')`. Their
cross-product on the left is `(a*d + b*c)*b'*d'`, which splits into
`a*b'*d*d' + b*b'*c*d'`. The first term equals `b*a'*d*d'` because
`a*b' = b*a'`. The second equals `b*b'*d*c'` because `c*d' = d*c'`. That
is the cross-product on the right. Definition N10.1 therefore says the
sums are equal. For the products, `(a*c)*(b'*d') = a*b'*c*d'`. Replace
`a*b'` by `b*a'` and then `c*d'` by `d*c'`. The result is
`(a'*c')*(b*d)`.

A fraction `m/1` is the integer `m` written as a fraction: `m*1 = 1*m`.
We write `1` for `1/1`. Adding fractions that already share a denominator
collapses by Proposition N10.1. In particular,
`a/b + c/b = (a+c)/b`.

Now add the half and the third correctly. `1/2 + 1/3 = (1*3 + 2*1)/(2*3)
= 5/6`. The common denominator is `6`: three sixths plus two sixths. The
numeral `2/5` came from a different rule, adding across the bar. That rule
is not Definition N10.2.

**Counterexample — digit cancellation.** `16/64` equals `1/4`, because
`16*4 = 64`. The digit `6` appears in both numerals, and striking both
sixes also leaves `1/4`. That coincidence is not the scaling rule. The
scaling rule cancels a common factor of the two integers, and here the
common factor is `16`: `16/64 = (16*1)/(16*4) = 1/4`. The same digit trick
fails on `12/24`. The fraction equals `1/2`, because `12*2 = 24`. Striking
the digit `2` leaves `1/4`, and `12*4 = 48`, which is not `24`. So `12/24`
is not `1/4`.

**Listing N10.1 — `cross.or`**

```orange
edition 2026;
module cross {
  spec equal(a: Int, b: Int, c: Int, d: Int) -> Bool {
    (a * d) == (b * c)
  }

  spec euclidean_half() -> Int { 1 / 2 }

  spec half_equals_two_quarters() -> Bool { equal(1, 2, 2, 4) }

  spec sum_num() -> Int { 1 * 3 + 2 * 1 }
  spec sum_den() -> Int { 2 * 3 }

  spec bad_num() -> Int { 1 + 1 }
  spec bad_den() -> Int { 2 + 3 }

  spec scaled() -> Bool { equal(1, 2, 2 * 1, 2 * 2) }

  spec sums_agree() -> Bool {
    equal(2 * 6 + 4 * 2, 4 * 6, sum_num(), sum_den())
  }

  spec products_agree() -> Bool {
    equal(2 * 2, 4 * 6, 1 * 1, 2 * 3)
  }

  spec sixteen() -> Bool { equal(16, 64, 1, 4) }

  spec sixteen_scaled() -> Bool { equal(16, 64, 16 * 1, 16 * 4) }

  spec twelve() -> Bool { equal(12, 24, 1, 2) }

  spec struck() -> Bool { equal(12, 24, 1, 4) }

  spec common_num() -> Int { 3 + 2 + 1 }
  spec common_den() -> Int { 6 }

  spec successive_num() -> Int { 5 * 6 + 6 * 1 }
  spec successive_den() -> Int { 6 * 6 }

  spec shared_three() -> Bool { equal(1 + 1 + 1, 6, 3, 6) }

  spec rejected_num() -> Int { 21 + 14 + 6 }
  spec rejected_den() -> Int { 42 }

  test "one half equals two quarters" { half_equals_two_quarters() }

  test "half plus a third is five sixths" {
    equal(sum_num(), sum_den(), 5, 6)
  }

  test "adding across the bar is a different numeral" {
    equal(bad_num(), bad_den(), 2, 5) && !equal(bad_num(), bad_den(), sum_num(), sum_den())
  }

  test "scaling by two preserves the half" { scaled() }

  test "an equal replacement preserves the sum and the product" {
    sums_agree() && products_agree()
  }

  test "sixteen over sixty-four equals one quarter" {
    sixteen() && sixteen_scaled()
  }

  test "striking a digit is not scaling" {
    twelve() && !struck()
  }

  test "three unit fractions sum to one" {
    equal(common_num(), common_den(), 1, 1)
      && equal(successive_num(), successive_den(), common_num(), common_den())
      && shared_three()
  }

  test "one half, one third, and one seventh sum to forty-one forty-seconds" {
    equal(rejected_num(), rejected_den(), 41, 42) && (rejected_num() != rejected_den())
  }
}
```

**Expected evaluation output:**

```text
cross::euclidean_half: Int = 0
cross::half_equals_two_quarters: Bool = true
cross::sum_num: Int = 5
cross::sum_den: Int = 6
cross::bad_num: Int = 2
cross::bad_den: Int = 5
cross::scaled: Bool = true
cross::sums_agree: Bool = true
cross::products_agree: Bool = true
cross::sixteen: Bool = true
cross::sixteen_scaled: Bool = true
cross::twelve: Bool = true
cross::struck: Bool = false
cross::common_num: Int = 6
cross::common_den: Int = 6
cross::successive_num: Int = 36
cross::successive_den: Int = 36
cross::shared_three: Bool = true
cross::rejected_num: Int = 41
cross::rejected_den: Int = 42
```

**Test report:**

```text
test "one half equals two quarters" ... ok
test "half plus a third is five sixths" ... ok
test "adding across the bar is a different numeral" ... ok
test "scaling by two preserves the half" ... ok
test "an equal replacement preserves the sum and the product" ... ok
test "sixteen over sixty-four equals one quarter" ... ok
test "striking a digit is not scaling" ... ok
test "three unit fractions sum to one" ... ok
test "one half, one third, and one seventh sum to forty-one forty-seconds" ... ok
9 tests: 9 passed, 0 failed
```

`equal` is Definition N10.1. `half_equals_two_quarters` is `1/2 = 2/4`.
`sum_num` and `sum_den` are the half plus the third, `5/6`. `bad_num` and
`bad_den` are `2/5`, the numeral from adding across the bar. `scaled` is
Proposition N10.1 at `k = 2` on the half. `sums_agree` replaces `1/2` by
`2/4` and `1/3` by `2/6`, then compares the sums. `products_agree` compares
the products of those same pairs. Those two `Bool` values are one instance
of Proposition N10.2. The proof is the claim for every quadruple the
hypotheses name. The test calls `equal` on the quadruples written here.

`sixteen` is `16/64 = 1/4`. `sixteen_scaled` cancels the common factor
`16`, which is Proposition N10.1. `twelve` is `12/24 = 1/2`. `struck` asks
whether that fraction equals `1/4`, and the printed value is false.
`common_num` adds `1/2`, `1/3`, and `1/6` on the denominator `6`.
`successive_num` adds them by Definition N10.2 in two steps. The two sums
agree, and both equal `1`. `shared_three` is three copies of `1/6`.
`rejected_num` and `rejected_den` are the sum `41/42` from the
counterexample in §N10.3. That sum is not `1`.

`euclidean_half` prints `0`. That is what `/` denotes on `Int`. It is not
the fraction one half. The parentheses in `(a * d) == (b * c)` keep
multiplication and comparison in separate groups. A quantifier over every
pair of fractions is the proof, not a type this listing has.

**Assumption N10.1 — Finite explicit list.** Every probability claim in
this lesson names a finite list of outcomes. An outcome is one named row
of that list. Writing the same row twice is a different list, not a
shorthand.

### N10.2 A ratio compares two counts

Three marked slips lie in a bowl of eight slips. The comparison of those
two counts is three to eight.

**Definition N10.3 — Ratio.** The ratio of an integer `m` to a positive
integer `n` is the fraction `m/n`. The notation `m:n` names that same
fraction when the sentence is comparing the two counts. The colon does not
create a second kind of number.

The ratio `3:8` is the fraction `3/8`. It records the comparison. It does
not, by itself, say that a slip will be drawn, or that every slip is
equally likely to be drawn. A ratio becomes a probability only after a
distribution is stated. Listing N10.1's `equal` is the comparison of the
two counts once both integers are written. The absence of a distribution
is not a `Bool` the compiler can compute from a sentence that was not
given.

### N10.3 Weights on a finite list

**Definition N10.4 — Finite probability space.** A finite probability
space is a finite list of distinct outcomes together with one fraction for
each outcome, called its weight. Every weight is greater than or equal to
zero, and the weights sum to `1`.

**Assumption N10.2 — Stated weights.** The weights are part of the claim.
A missing weight is not silently filled with the uniform value.

**Example N10.1 — Four strings, unequal weights.** Take the outcomes `00`,
`01`, `10`, and `11`, with weights `1/2`, `1/6`, `1/6`, and `1/6`. Each
weight is nonnegative. Their sum is `1/2 + 1/6 + 1/6 + 1/6 = 1/2 + 3/6 =
1`. This is a finite probability space. The strings are two bits long.
That length is the fact from Chapter 2. The weights are a second fact.

**Counterexample — weights that do not sum to one.** The three fractions
`1/2`, `1/3`, and `1/7` are nonnegative, and
`1/2 + 1/3 + 1/7 = 21/42 + 14/42 + 6/42 = 41/42`. They do not sum to `1`,
so they are not a finite probability space. The nearby list `1/2`, `1/3`,
`1/6` does sum to `1`. The difference is the arithmetic, not the number of
rows.

A reader who has not been given weights, and who writes `1/4` on each of
four strings, has added a sentence the problem did not contain. Assumption
N10.2 forbids that silent step. The sentence “the draw is uniform on these
four strings” would make `1/4` the right weight. Absence of the sentence
does not.

### N10.4 Events

**Definition N10.5 — Event.** An event is a rule that says yes or no for
each outcome on the list. The probability of the event, written `P(E)`
when the event is called `E`, is the sum of the weights of the outcomes
for which the rule says yes. If the rule says yes for no outcome, the sum
is `0`.

Preview: [N9](NOVICE_LOGIC.md#n92-a-set-is-fixed-by-its-members) calls the
outcomes for which a rule says yes a set. This lesson does not use that
theory. The rule itself is enough, and on a short list you can also name
the matching rows.

**Proposition N10.3 — Bounds and complement.** In a finite probability
space, `0 ≤ P(E) ≤ 1`. If “not E” says yes exactly where `E` says no, then
`P(not E) = 1 - P(E)`.

**Proof.** Every weight is nonnegative, so a sum of some of them is
nonnegative. Split the list into the rows where `E` says yes and the rows
where it says no. No row is in both groups, and every row is in one. The
two sums therefore add to the sum of all the weights, which is `1`. The
yes-sum is `P(E)` and is one of two nonnegative fractions that add to `1`,
so it is at most `1`. The no-sum is the complement.

In Example N10.1, let `E` say yes when the second bit is `1`. The matching
rows are `01` and `11`. `P(E) = 1/6 + 1/6 = 1/3`. The complement is `2/3`.
The two fractions sum to `1`.

**Listing N10.2 — `weights.or`**

```orange
edition 2026;
module weights {
  spec numerators() -> Int^4 { [3, 1, 1, 1] }

  spec total() -> Int {
    let w: Int^4 = numerators();
    for i in 0..4 with s: Int = 0 { s + w[i] }
  }

  spec none_negative() -> Bool {
    let w: Int^4 = numerators();
    for i in 0..4 with ok: Bool = true { ok && (w[i] >= 0) }
  }

  spec second_bit_num() -> Int { 1 + 1 }

  spec complement_num() -> Int { total() - second_bit_num() }

  spec uniform_copies(n: Int) -> Int { n * 1 }

  test "the four weights are a probability space" {
    none_negative() && (total() == 6)
  }

  test "the second bit has probability one third" {
    (second_bit_num() * 3) == (total() * 1)
  }

  test "the complement is two thirds and restores the total" {
    ((complement_num() * 3) == (total() * 2)) && ((second_bit_num() + complement_num()) == total())
  }

  test "uniform copies on four strings and on the byte" {
    (uniform_copies(4) == 4) && (uniform_copies(256) == 256)
  }
}
```

**Expected evaluation output:**

```text
weights::numerators: Int^4 = [3, 1, 1, 1]
weights::total: Int = 6
weights::none_negative: Bool = true
weights::second_bit_num: Int = 2
weights::complement_num: Int = 4
```

**Test report:**

```text
test "the four weights are a probability space" ... ok
test "the second bit has probability one third" ... ok
test "the complement is two thirds and restores the total" ... ok
test "uniform copies on four strings and on the byte" ... ok
4 tests: 4 passed, 0 failed
```

The numerators `3, 1, 1, 1` are Example N10.1 on the common denominator
`6`: `1/2` is three sixths, and each `1/6` is one sixth. `total` is their
sum, `6`, so the weights sum to `1`. `none_negative` is the
nonnegativity check on these four rows. `second_bit_num` is `2`, and
`2/6 = 1/3`. `complement_num` is `4`, and `4/6 = 2/3`. The two numerators
add back to `6`, which is Proposition N10.3 on this event.

`uniform_copies` is the integer `N` in Proposition N10.4: `N` copies of
`1/N` sum to `N/N`. The calls check `N = 4` and `N = 256`. An array whose
length is a parameter `N` is not a type on this surface. The length `4`
above is a literal. The proof of Proposition N10.4 is the claim for every
positive `N`. The byte weight `1/256` is that proposition with `N = 256`,
together with Chapter 2's count of the strings. Listing N10.1 already
recorded that `1/2 + 1/3 + 1/7` sums to `41/42`.

### N10.5 Uniform choice

**Definition N10.6 — Uniform distribution.** The uniform distribution on
a list of `N` outcomes, with `N` positive, gives every outcome the weight
`1/N`.

**Proposition N10.4 — The uniform weights form a probability space.** The
`N` weights `1/N` are nonnegative and sum to `1`.

**Proof.** `N` copies of `1/N` sum to `N/N = 1`, by the shared-denominator
rule in §N10.1.

Chapter 2 already counted the byte strings: there are `256` of them. The
uniform distribution on those strings gives each string weight `1/256`.
That is one stated distribution. The byte `0xFF` is still one particular
string. Section 2.7 already refused to call that string random merely
because the format allows `256` values. This lesson keeps the refusal and
replaces the rejected word with a stated list of weights. We will not use
“random” as a technical term here.

### N10.6 Conditional probability

**Definition N10.7 — Conditional probability.** Let `A` and `B` be events
on the same finite probability space, and suppose `P(B)` is positive. The
conditional probability of `A` given `B`, written `P(A | B)`, is
`P(A and B) / P(B)`. The event “A and B” says yes only when both rules say
yes. If `P(B) = 0`, Definition N10.7 assigns no number. It does not assign
zero.

**Proposition N10.5 — Conditioning produces new weights.** Suppose `P(B)`
is positive. Give each outcome where `B` says yes a new weight equal to its
old weight divided by `P(B)`, and drop the outcomes where `B` says no. The
new weights are nonnegative and sum to `1`.

**Proof.** Each old weight is nonnegative and `P(B)` is positive, so each
quotient is nonnegative. The sum of the old weights on the retained rows
is `P(B)`. Dividing that sum by `P(B)` yields `1`.

Return to Example N10.1. Let `B` say yes when the first bit is `0`. Then
`P(B) = 1/2 + 1/6 = 2/3`. Let `A` say yes only for `00`. The event “A and
B” is the same as `A`, because `00` already has first bit `0`, so
`P(A and B) = 1/2`. Therefore `P(A | B) = (1/2)/(2/3) = 3/4`. The other
retained row is `01`, and `(1/6)/(2/3) = 1/4`. Proposition N10.5 says these
two new weights sum to `1`. They do: `3/4 + 1/4 = 1`.

Reverse the question. `P(B | A) = (1/2)/(1/2) = 1`. The two conditional
probabilities are `3/4` and `1`. They answer different questions. The first
asks how much of the first-bit-zero weight sits on `00`. The second asks
whether `00` has first bit zero, which it does.

**Counterexample — a zero condition.** Let `Z` say yes only if the outcome
is both `00` and `11`. No row satisfies that rule, so `P(Z) = 0`.
Definition N10.7 does not produce `P(B | Z)`. Filling in zero would invent
a quotient whose denominator is zero, and there would be no retained row
on which Proposition N10.5 could place weight `1`.

The same observation, under the uniform distribution on the same four
strings, gives a different conditional list. Each old weight is `1/4`, and
`P(B) = 1/2`, so each of `00` and `01` receives new weight `(1/4)/(1/2) =
1/2`. The strings still have two bits. The observation is still “the first
bit is `0`.” The conditional weights are not the weights from Example
N10.1. The distribution that was conditioned is a different object from the
observation.

**Listing N10.3 — `condition.or`**

```orange
edition 2026;
module condition {
  spec equal_parts(a: Int, b: Int, c: Int, d: Int) -> Bool {
    (a * d) == (b * c)
  }

  spec den() -> Int { 6 }
  spec event_num() -> Int { 3 }
  spec condition_num() -> Int { 3 + 1 }

  spec given_00() -> Bool {
    ((event_num() * den()) * 4) == ((den() * condition_num()) * 3)
  }

  spec given_01() -> Bool {
    ((1 * den()) * 4) == ((den() * condition_num()) * 1)
  }

  spec new_weights() -> Bool { equal_parts(3 + 1, 4, 1, 1) }

  spec reverse() -> Bool { (event_num() * den()) == (den() * event_num()) }

  spec uniform_given() -> Bool { equal_parts(1 * 2, 4 * 1, 1, 2) }

  spec zero_quotient() -> Int { 1 / 0 }

  test "00 given a first bit of 0 is three quarters" { given_00() }

  test "01 given a first bit of 0 is one quarter" { given_01() }

  test "the two new weights sum to one" { new_weights() }

  test "the reverse conditional is one" { reverse() }

  test "the uniform observation gives one half" { uniform_given() }
}
```

**Expected evaluation output:**

```text
condition::den: Int = 6
condition::event_num: Int = 3
condition::condition_num: Int = 4
condition::given_00: Bool = true
condition::given_01: Bool = true
condition::new_weights: Bool = true
condition::reverse: Bool = true
condition::uniform_given: Bool = true
condition::zero_quotient: Int = 0
```

**Test report:**

```text
test "00 given a first bit of 0 is three quarters" ... ok
test "01 given a first bit of 0 is one quarter" ... ok
test "the two new weights sum to one" ... ok
test "the reverse conditional is one" ... ok
test "the uniform observation gives one half" ... ok
5 tests: 5 passed, 0 failed
```

On the denominator `6`, the weight of `00` is `3` and the first-bit-zero
rows sum to `4`. `given_00` is `(3/6) / (4/6) = 3/4`. `given_01` is
`(1/6) / (4/6) = 1/4`. `new_weights` is `3/4 + 1/4 = 1`, which is
Proposition N10.5 on these two retained rows. `reverse` is
`(3/6) / (3/6) = 1`. `uniform_given` is `(1/4) / (1/2) = 1/2`.

`zero_quotient` prints `0`. That is Euclidean division by zero on `Int`.
Definition N10.7 assigns no number when the conditioning event has
probability `0`. The printed `0` is the operator the definition refuses
to use. There is no listing of a conditional probability whose
denominator is zero, because that number is not defined.

### N10.7 Conditional knowledge

**Definition N10.8 — Conditional knowledge.** Relative to one stated
finite probability space and one stated event `B` with positive
probability, conditional knowledge of the outcome is the list of new
weights from Proposition N10.5. Conditional knowledge of an event `A` is
the number `P(A | B)`.

This is not a description of a person's private confidence. It is a number,
or a list of numbers, computed from two stated ingredients: the space, and
the event being used as the condition. If either ingredient is missing, the
definition does not apply.

In this lesson, an adversary's uncertainty means that conditional list,
under a stated description of what the adversary is assumed to know. The
description is an assumption. Changing it changes the list, as the uniform
and unequal calculations just did, even when the strings and the announced
bit stay fixed.

### N10.8 Independence

**Definition N10.9 — Independence.** Events `A` and `B` are independent
when `P(A and B) = P(A)*P(B)`.

**Proposition N10.6 — The conditional form.** If `P(B)` is positive, then
`A` and `B` are independent exactly when `P(A | B) = P(A)`.

**Proof.** `P(A | B) = P(A)` means `P(A and B)/P(B) = P(A)`. Multiply both
sides by the positive fraction `P(B)`. The products of fractions from
Definition N10.2 turn that equation into `P(A and B) = P(A)*P(B)`, and the
same steps run backward.

**Counterexample — disjoint events.** Under the uniform distribution on
the four strings, let `A` say “first bit `0`” and let `C` say “first bit
`1`.” No string has both first bits, so `P(A and C) = 0`. Each event has
probability `1/2`, and the product is `1/4`. Since `0` is not `1/4`, the
events are not independent. Also `P(A | C) = 0`, which is not `P(A)`.

Two events can fail to overlap and still be the wrong picture of
“unrelated.” Here they are mutually exclusive: one of them happening rules
the other out. Independence would have left the probability of `A`
unchanged by news of `C`.

**Example N10.2 — Drawing without putting a key back.** Three named keys,
`A`, `B`, and `C`. Draw a first key uniformly from the three, then a second
key uniformly from the two that remain. There are `3*2 = 6` possible
orders, and each has weight `(1/3)*(1/2) = 1/6`.

Let `F` say “the first key is `A`” and let `S` say “the second key is `A`.”
`P(F) = 1/3`. The second key is `A` in two of the six orders, so
`P(S) = 1/3`. No order has `A` in both positions, so `P(F and S) = 0`. The
product `P(F)*P(S)` is `1/9`. The events are not independent. Given that
the first key is `A`, the conditional probability that the second is `A`
is `0`, not `1/3`.

The second draw is uniform on what remains. That local uniformity does not
make the two events independent. The removal changed the second list.

### N10.9 Independent choices and a product of counts

**Definition N10.10 — Independent choices.** Suppose each choice has its
own finite probability space. A combined outcome picks one outcome from
each choice. The choices are independent when the weight of every combined
outcome is the product of the weights of its components.

**Proposition N10.7 — Two uniform choices.** If the first choice is
uniform on `M` outcomes and the second is uniform on `N` outcomes, and the
choices are independent, then there are `M*N` combined outcomes and each
has weight `1/(M*N)`. The combined space is the uniform distribution on
those pairs.

**Proof.** Each component weight is `1/M` or `1/N`, so each pair has
weight `(1/M)*(1/N) = 1/(M*N)`. There are `M` choices for the first
component and, for each of them, `N` choices for the second, so there are
`M*N` pairs. `M*N` copies of `1/(M*N)` sum to `1`.

For more than two choices, use the same definition: multiply one more
weight for each added choice. Chapter 2 already used the special case in
which every choice is a bit and every weight is `1/2`. Then `L` independent
uniform bit choices give weight `1/2^L` to each of the `2^L` strings.
That sentence adds a distribution to the count Chapter 2 already proved.
If the bit choices are not independent, or not uniform, the format may
still contain `2^L` strings while the weights are not all `1/2^L`.

One uniform byte and one independent uniform bit therefore give
`256*2 = 512` combined outcomes, each of weight `1/512`. The probability
that the byte is zero and the bit is zero is the weight of that one
combined outcome, `1/512`. The factor `256` is Chapter 2's byte count. It
was not re-counted here.

**Proposition N10.8 — Events from different choices.** Suppose two choices
are independent in the sense of Definition N10.10. Let `A` depend only on
the first choice, and let `B` depend only on the second. Then `A` and `B`
are independent as events.

**Proof.** `P(A and B)` sums the products of the component weights over the
pairs where both rules say yes. That sum factors as the sum of the first
weights over `A`, times the sum of the second weights over `B`, because
every retained first outcome is paired with every retained second outcome
and the weight is a product. Those two sums are `P(A)` and `P(B)`.

Example N10.2 is the contrasting case. Its second list depends on the
first draw, so the weight of an order is not the product of two weights
taken from fixed lists. Proposition N10.8 does not apply, and the events
were not independent.

**Listing N10.4 — `independent.or`**

```orange
edition 2026;
module independent {
  spec equal_parts(a: Int, b: Int, c: Int, d: Int) -> Bool {
    (a * d) == (b * c)
  }

  spec differs(a: Int, b: Int) -> Bool { a != b }

  spec orders() -> Int { 3 * 2 }

  spec draw_weight() -> Bool { equal_parts(1 * 1, 3 * 2, 1, 6) }

  spec separate() -> Bool { equal_parts(1 * 1, 3 * 3, 1, 9) }

  spec combined() -> Int { 256 * 2 }

  spec factored_sum() -> Int { 3 * 1 + 2 * 1 }
  spec factored_den() -> Int { 6 * 4 }
  spec product_of_sums() -> Int { 5 * 1 }

  test "disjoint events are not independent" { differs(0 * 4, 1 * 1) }

  test "six orders each of weight one sixth" {
    (orders() == 6) && draw_weight()
  }

  test "drawing without replacement is not independence" {
    differs(0 * 9, 1) && separate()
  }

  test "one uniform byte and one independent bit" { combined() == 512 }

  test "the sum of the products equals the product of the sums" {
    (factored_sum() == product_of_sums()) && (factored_den() == 24)
  }
}
```

**Expected evaluation output:**

```text
independent::orders: Int = 6
independent::draw_weight: Bool = true
independent::separate: Bool = true
independent::combined: Int = 512
independent::factored_sum: Int = 5
independent::factored_den: Int = 24
independent::product_of_sums: Int = 5
```

**Test report:**

```text
test "disjoint events are not independent" ... ok
test "six orders each of weight one sixth" ... ok
test "drawing without replacement is not independence" ... ok
test "one uniform byte and one independent bit" ... ok
test "the sum of the products equals the product of the sums" ... ok
5 tests: 5 passed, 0 failed
```

`differs(0 * 4, 1 * 1)` is the disjoint counterexample: the intersection
has probability `0`, and `(1/2)*(1/2) = 1/4`. `orders` is `3*2 = 6`.
`draw_weight` is `(1/3)*(1/2) = 1/6`. `separate` is `(1/3)*(1/3) = 1/9`.
The intersection of “first is `A`” and “second is `A`” has weight `0`,
which is not `1/9`. `combined` is `256*2 = 512`, Proposition N10.7 on one
byte and one bit. The factor `256` is Chapter 2's count.

`factored_sum` and `product_of_sums` are one instance of the factoring in
the proof of Proposition N10.8. The first choice has numerators `3, 2, 1`
on denominator `6`, and the event on that choice keeps `3` and `2`. The
second choice has numerators `1, 3` on denominator `4`, and the event on
that choice keeps `1`. The sum of the products is `3*1 + 2*1 = 5` over
`6*4 = 24`. The product of the sums is `5*1` over the same denominator.
The proof says this factoring holds for every pair of independent choices.
The listing checks this pair. A sum over an arbitrary collection of
weights is the proof's finite sum, not a single array type of parameter
length.

### N10.10 The sum that counts pairs

The birthday count needs the sum of the first `m` positive integers.

**Proposition N10.9 — Sum of the first m positive integers.** For a
positive integer `m`, `1 + 2 + ... + m = m*(m+1)/2`. The sum of no terms
is `0`, which agrees with the same formula at `m = 0`.

**Proof.** Let `S` be the sum, and write the same terms in reverse order.
Adding the two copies pairs `1` with `m`, `2` with `m-1`, and so on. Each
pair sums to `m+1`, and there are `m` pairs. Thus `2*S = m*(m+1)`, so
`S = m*(m+1)/2`.

The number of unordered pairs among `n` people is the number of ways to
choose a later person for each earlier one: `(n-1) + (n-2) + ... + 1`.
By Proposition N10.9, that sum is `n*(n-1)/2`.

**Listing N10.5 — `pair_sum.or`**

```orange
edition 2026;
module pair_sum {
  spec formula(m: Int) -> Int { (m * (m + 1)) / 2 }

  spec added_through_ten() -> Int {
    for i in 1..11 with s: Int = 0 { s + i }
  }

  spec added_through_four() -> Int {
    for i in 1..5 with s: Int = 0 { s + i }
  }

  spec pairs_among(n: Int) -> Int { (n * (n - 1)) / 2 }

  test "the sum through ten matches the formula" {
    added_through_ten() == formula(10)
  }

  test "the sum through four is ten" {
    (added_through_four() == 10) && (formula(4) == 10)
  }

  test "the formula at zero is zero" { formula(0) == 0 }

  test "three people give three unordered pairs" { pairs_among(3) == 3 }

  test "twenty-three people give two hundred fifty-three pairs" {
    pairs_among(23) == 253
  }
}
```

**Expected evaluation output:**

```text
pair_sum::added_through_ten: Int = 55
pair_sum::added_through_four: Int = 10
```

**Test report:**

```text
test "the sum through ten matches the formula" ... ok
test "the sum through four is ten" ... ok
test "the formula at zero is zero" ... ok
test "three people give three unordered pairs" ... ok
test "twenty-three people give two hundred fifty-three pairs" ... ok
5 tests: 5 passed, 0 failed
```

`formula` is the right-hand side of Proposition N10.9. The loops add
`1` through `10` and `1` through `4`. A loop's bounds are literals, so
these two ranges are two checks, not a check for every `m`. The pairing
argument is the proof for every nonnegative `m`. `pairs_among(3)` is the
three unordered pairs in Example N10.3. `pairs_among(23)` is the `253`
pairs in Example N10.5. Division in `formula` is exact on these inputs
because `m*(m+1)` is even. The parentheses put that division outside the
product.

The sum of no terms is `0`, and `formula(0)` prints that integer through
the test. A loop cannot add no terms. The bounds `0..0` are empty, and
the checker rejects them before any step.

**Listing N10.6 — `empty_sum.or`, intentionally rejected**

```orange
edition 2026;
module empty_sum {
  spec none() -> Int {
    for i in 0..0 with s: Int = 0 { s + i }
  }
}
```

The diagnostic is `ORC0225`: `the loop range 0..0 is empty`.

```text
error[ORC0225]: the loop range 0..0 is empty
 --> <stdin>:4:17
  |
4 |     for i in 0..0 with s: Int = 0 { s + i }
  |                 ^ a loop runs at least once
  = note: a loop `for i in a..b` runs once for each i from a up to b - 1, with a < b <= 65536
```

No value is printed. The empty sum stays the clause in Proposition N10.9
that sets the sum of no terms equal to `0`. It is not a loop this surface
can run.

### N10.11 An elementary birthday bound

**Assumption N10.3 — The birthday model.** There are `n` people and `d`
days, with `n` and `d` positive integers. Each person has one day. The `n`
choices are independent and uniform on the `d` days, in the sense of
Definition N10.10. A collision is the event that at least two people have
the same day.

There are `d^n` combined outcomes, each of weight `1/d^n`.

If `n` is greater than `d`, a collision is certain. A list of distinct
days cannot be longer than the list of all `d` days: after `d` people have
taken `d` different days, no unused day remains. So when `n > d`, the
no-collision event is empty and the collision probability is `1`.

If `1 ≤ n ≤ d`, the probability that all `n` days are distinct is the
product `(d/d)*((d-1)/d)*...*((d-n+1)/d)`. The first factor is `1`. Each
later factor is the proportion of days still unused if all earlier days
were distinct. Equivalently, the probability is the product of `(1 - k/d)`
for `k` from `1` through `n-1`. When `n = 1`, that product has no factors.
An empty product is `1`, and one person has no pair to collide with.

**Example N10.3 — Three people, five days.** The distinct-day probability
is `(4/5)*(3/5) = 12/25`. The collision probability is `1 - 12/25 = 13/25`.

Count the same `5^3 = 125` outcomes a second way. All three days distinct:
`5*4*3 = 60` outcomes. All three days equal: `5` outcomes. Exactly one
pair equal and the third day different: choose which pair, in `3` ways,
choose its day, in `5` ways, and choose a different day for the remaining
person, in `4` ways, giving `3*5*4 = 60` outcomes. Exactly two pairs is
impossible: if person 1 matches person 2 and person 1 matches person 3,
then person 2 matches person 3 as well. The three classes sum to
`60 + 60 + 5 = 125`. The collision outcomes are the last two classes,
`65` outcomes, and `65/125 = 13/25`. The sequential product and the
classification agree.

Let `S` be `n*(n-1)/(2*d)`. This is the number of unordered pairs divided
by the number of days. For Example N10.3, `S = 3*2/(2*5) = 3/5`.

**Proposition N10.10 — Upper bound.** Under Assumption N10.3, the
collision probability is at most `S`. It is also at most `1`, by
Proposition N10.3. When `S` is greater than `1`, the bound `S` is true and
weaker than the bound `1`.

**Proof.** Fix one unordered pair of people. The outcomes in which those
two share a day number `d^(n-1)`: the shared day may be any of the `d`
days, and each other person may be any day. If an outcome has at least one
colliding pair, it is counted at least once when these counts are added
over all unordered pairs. An outcome with several colliding pairs is
counted several times. The number of colliding outcomes is therefore at
most the sum of the pair-counts, which is
`(n*(n-1)/2)*d^(n-1)`. Divide by the `d^n` equally weighted outcomes. The
probability is at most `n*(n-1)/(2*d) = S`.

**Proposition N10.11 — A product bound.** Let `a1`, `a2`, through `am` be
fractions with `0 ≤ ai ≤ 1`. Then the product of the terms `(1 - ai)` is
at most `1/(1 + a1 + ... + am)`.

**Proof.** First, `1 - x ≤ 1/(1+x)` whenever `0 ≤ x ≤ 1`. Indeed
`(1-x)*(1+x) = 1 - x*x ≤ 1`, and `1+x` is positive, so dividing by it
yields the inequality. Second, the product `(1+a1)*...*(1+am)` is at least
`1 + a1 + ... + am`. Start from `1`, whose excess over `1 + 0` is zero.
If a quantity `Q` is at least `1+s` and `a` is nonnegative, then
`Q*(1+a) = Q + Q*a` is at least `(1+s) + (1+s)*a`, because `Q` is at least
`1+s` and `a` is nonnegative. And `(1+s)*(1+a) = 1 + s + a + s*a`, which
is at least `1 + s + a`. Each new factor therefore adds its own `a` to the
lower bound. Third, if `0 ≤ p ≤ q` and `0 ≤ r ≤ s`, then `p*r ≤ q*s`:
first `p*r ≤ q*r`, then `q*r ≤ q*s`. Apply that to the nonnegative terms
`1 - ai ≤ 1/(1+ai)`. The product of the left sides is at most the product
of the right sides, which is `1` divided by the product of the `(1+ai)`,
which is at most `1` divided by `1` plus the sum of the `ai`.

**Proposition N10.12 — Lower bound, when n is at most d.** Under
Assumption N10.3, if `n ≤ d`, the collision probability is at least
`S/(1+S)`.

**Proof.** The distinct-day probability is the product of `(1 - k/d)` for
`k` from `1` through `n-1`. Each `k/d` satisfies `0 ≤ k/d < 1` because
`k ≤ n-1 ≤ d-1`. Proposition N10.11 says the product is at most
`1/(1+S)`, since `S` is exactly the sum of those `k/d`. Subtract from `1`:
the collision probability is at least `1 - 1/(1+S) = S/(1+S)`.

In Example N10.3, `S = 3/5` and `S/(1+S) = (3/5)/(8/5) = 3/8`. The exact
collision probability is `13/25`. And `3/8 ≤ 13/25 ≤ 3/5`, since
`3/8 = 75/200`, `13/25 = 104/200`, and `3/5 = 120/200`.

**Example N10.4 — The upper bound goes quiet.** Five people and five days.
Here `n = d`, so the distinct-day product still applies:
`(4/5)*(3/5)*(2/5)*(1/5) = 24/625`. The collision probability is
`1 - 24/625 = 601/625`. The pair quantity is `S = 5*4/(2*5) = 2`, which
is greater than `1`. Proposition N10.10's bound of `2` says less than the
bound `1` already proved for every event. The lower bound remains
`2/(1+2) = 2/3`, and `601/625` is greater than `2/3` because
`601*3 = 1803` and `625*2 = 1250`.

**Example N10.5 — Twenty-three people and 365 days.** This is still
Assumption N10.3, with `n = 23` and `d = 365`. It is not a claim about
human birthdays, which need not be uniform or independent, and it is not
a recommendation about keys. Here
`S = 23*22/(2*365) = 253/365`, and
`S/(1+S) = 253/(365+253) = 253/618`.

Compare the bounds with `1/2`. `253/618 < 1/2` because `506 < 618`.
`253/365 > 1/2` because `506 > 365`. A number known only to lie between
`253/618` and `253/365` could lie on either side of `1/2`. The two bounds
do not prove the comparison with one half.

The exact collision probability is `1` minus the product of `(365-k)/365`
for `k` from `1` through `22`. That is a ratio of two integers. Twice the
numerator is smaller than the denominator `365^22`, so the distinct-day
probability is smaller than `1/2` and the collision probability is greater
than `1/2`. The reference test performs that integer comparison. It does
not establish a rounded percentage, a fact about calendars, or a length a
key ought to have.

A sharper bound that uses the exponential function is not proved here. The
inequality it needs has not been built. The named surface has no
exponential function, so that bound has no listing.

**Listing N10.7 — `birthday.or`**

```orange
edition 2026;
module birthday {
  spec at_most(a: Int, b: Int) -> Bool { a <= b }
  spec above(a: Int, b: Int) -> Bool { a > b }

  spec distinct_num() -> Int { 4 * 3 }
  spec distinct_den() -> Int { 5 * 5 }
  spec collision_num() -> Int { distinct_den() - distinct_num() }

  spec all_distinct() -> Int { 5 * 4 * 3 }
  spec one_pair() -> Int { 3 * 5 * 4 }
  spec all_equal() -> Int { 5 }
  spec classified() -> Int { all_distinct() + one_pair() + all_equal() }
  spec cube() -> Int {
    for i in 0..3 with p: Int = 1 { p * 5 }
  }
  spec collision_outcomes() -> Int { one_pair() + all_equal() }
  spec pair_hits() -> Int { 3 * (5 * 5) }

  spec upper_num() -> Int { (3 * 2) / 2 }
  spec upper_den() -> Int { 5 }
  spec lower_den() -> Int { upper_den() + upper_num() }

  spec gap_num() -> Int { 4 * 6 }
  spec square_gap() -> Int { 5 * 5 - 1 }
  spec expanded() -> Int { 6 * 7 }
  spec sum_bound() -> Int { 5 * 5 + 5 * 1 + 5 * 2 }

  spec distinct_num_5() -> Int { 4 * 3 * 2 * 1 }
  spec distinct_den_5() -> Int {
    for i in 0..4 with p: Int = 1 { p * 5 }
  }
  spec collision_num_5() -> Int { distinct_den_5() - distinct_num_5() }
  spec quiet_upper() -> Int { (5 * 4) / (2 * 5) }

  spec crowded() -> Int {
    for i in 0..3 with p: Int = 1 { p * 2 }
  }
  spec crowded_distinct() -> Int { 2 * 1 * 0 }

  spec pairs_23() -> Int { (23 * 22) / 2 }

  spec twice_distinct_23() -> Int {
    for k in 1..23 with p: Int = 2 { p * (365 - k) }
  }

  spec days_power_23() -> Int {
    for k in 1..23 with p: Int = 1 { p * 365 }
  }

  spec above_half() -> Bool { twice_distinct_23() < days_power_23() }

  test "three people and five days, counted two ways" {
    (distinct_num() == 12)
      && (distinct_den() == 25)
      && (collision_num() == 13)
      && (classified() == cube())
      && (cube() == 125)
      && (collision_outcomes() == 65)
      && ((collision_outcomes() * distinct_den()) == (cube() * collision_num()))
  }

  test "the pair hits bound the colliding outcomes" {
    (collision_outcomes() <= pair_hits()) && ((pair_hits() * upper_den()) == (cube() * upper_num()))
  }

  test "the bounds sit around thirteen twenty-fifths" {
    (upper_num() == 3)
      && (lower_den() == 8)
      && at_most(3 * 25, 8 * 13)
      && at_most(13 * 5, 25 * 3)
  }

  test "the two algebraic steps on one fifth and two fifths" {
    (gap_num() == square_gap())
      && (gap_num() < (5 * 5))
      && (expanded() == 42)
      && (sum_bound() == 40)
      && (expanded() >= sum_bound())
  }

  test "five people and five days" {
    (distinct_num_5() == 24)
      && (distinct_den_5() == 625)
      && (collision_num_5() == 601)
      && (quiet_upper() == 2)
      && above(601 * 3, 625 * 2)
  }

  test "three people and two days collide with certainty" {
    (crowded() == 8) && (crowded_distinct() == 0) && ((crowded() - crowded_distinct()) == crowded())
  }

  test "twenty-three people and three hundred sixty-five days" {
    (pairs_23() == 253)
      && ((2 * pairs_23()) < (365 + pairs_23()))
      && ((2 * pairs_23()) > 365)
      && above_half()
  }
}
```

**Expected evaluation output:**

```text
birthday::distinct_num: Int = 12
birthday::distinct_den: Int = 25
birthday::collision_num: Int = 13
birthday::all_distinct: Int = 60
birthday::one_pair: Int = 60
birthday::all_equal: Int = 5
birthday::classified: Int = 125
birthday::cube: Int = 125
birthday::collision_outcomes: Int = 65
birthday::pair_hits: Int = 75
birthday::upper_num: Int = 3
birthday::upper_den: Int = 5
birthday::lower_den: Int = 8
birthday::gap_num: Int = 24
birthday::square_gap: Int = 24
birthday::expanded: Int = 42
birthday::sum_bound: Int = 40
birthday::distinct_num_5: Int = 24
birthday::distinct_den_5: Int = 625
birthday::collision_num_5: Int = 601
birthday::quiet_upper: Int = 2
birthday::crowded: Int = 8
birthday::crowded_distinct: Int = 0
birthday::pairs_23: Int = 253
birthday::twice_distinct_23: Int = 231237366038862245876140619588542231527395949045350400000
birthday::days_power_23: Int = 234662135214110469141956898203822336135218143463134765625
birthday::above_half: Bool = true
```

**Test report:**

```text
test "three people and five days, counted two ways" ... ok
test "the pair hits bound the colliding outcomes" ... ok
test "the bounds sit around thirteen twenty-fifths" ... ok
test "the two algebraic steps on one fifth and two fifths" ... ok
test "five people and five days" ... ok
test "three people and two days collide with certainty" ... ok
test "twenty-three people and three hundred sixty-five days" ... ok
7 tests: 7 passed, 0 failed
```

`distinct_num` and `distinct_den` are Example N10.3's product `12/25`.
`collision_num` is `13`. The classification is `60 + 60 + 5 = 125`, and
`cube` is `5^3`. `65/125 = 13/25` is the cross-multiplication in the
first test. `pair_hits` is `3 * 5^2 = 75`, the count in the proof of
Proposition N10.10 for these three people, and `65 ≤ 75`. Dividing by
`125` gives the upper bound `3/5`. `lower_den` is `8`, so `S/(1+S)` is
`3/8`. The comparisons `3/8 ≤ 13/25 ≤ 3/5` are `at_most(75, 104)` and
`at_most(65, 75)`.

`gap_num` and `square_gap` are the first step of Proposition N10.11 at
`x = 1/5`: `(1 - 1/5)*(1 + 1/5) = 1 - (1/5)^2 = 24/25`, and `24 < 25`.
`expanded` and `sum_bound` are the second step at `1/5` and `2/5`:
`(6/5)*(7/5) = 42/25` and `1 + 1/5 + 2/5 = 40/25`. The general product,
over every finite list of fractions in `0` through `1`, is the proof.
The listing checks these two fractions.

`distinct_num_5` is `24` and `distinct_den_5` is `625`, so Example N10.4's
collision numerator is `601`. `quiet_upper` is `2`. `601/625` sits above
`2/3` because `601*3 > 625*2`. `crowded` is `2^3 = 8` and
`crowded_distinct` is `0`, the case `n > d` in the paragraph before
Example N10.3: three people and two days. The collision count equals the
size of the space, so the probability is `1`.

`pairs_23` is `253`. Twice that integer is less than `365 + 253` and
greater than `365`, so `253/618 < 1/2 < 253/365`. `twice_distinct_23`
starts at `2` and multiplies `(365 - k)` for each `k` from `1` through
`22`. That is twice the product of `364` down through `343`.
`days_power_23` multiplies `365` that same number of times. `above_half`
is the comparison of those two integers. It is true. The printed integers
are that comparison. They are not a rounded percentage, a fact about
calendars, or a length a key ought to have. Propositions N10.10 and
N10.11 remain the claims for every positive `n` and `d` Assumption N10.3
names.

### N10.12 Expected value

**Definition N10.11 — Expected value.** A numerical quantity on a finite
probability space assigns one fraction to each outcome. The expected value
of the quantity is the sum, over the outcomes, of the outcome's weight
times the fraction assigned to it. If the quantity is called `v`, the
expected value is written `E[v]`.

**Proposition N10.13 — Constants, sums, and indicators.** If `v` assigns
the same fraction `c` to every outcome, then `E[v] = c`. If `v` and `w`
are two quantities, then `E[v+w] = E[v] + E[w]`. This splitting does not
require the quantities to come from independent events. If a quantity is
`1` on the outcomes where an event `E` says yes and `0` on the others, its
expected value is `P(E)`.

**Proof.** The constant case factors `c` out of a finite sum of weights,
leaving `c` times `1`. For the sum, each outcome contributes
`weight * (v + w) = weight*v + weight*w`, and a finite sum may be split
into those two groups of terms. The indicator sum retains the weights
where the assigned fraction is `1` and drops the rest, because multiplying
by `0` contributes nothing. The retained sum is `P(E)`.

Apply this to pairs. For each unordered pair of people, let the indicator
be `1` when that pair shares a day. Each such indicator has expected value
`1/d`, because Proposition N10.10's count gives that pair probability
`1/d`. There are `n*(n-1)/2` pairs. Linearity adds the expected values, so
the expected number of colliding unordered pairs is `S`. The addition is
valid even when the pair-events are not independent.

They need not be independent as a group. For three people, every two
pair-events are independent: the probability that two specified pairs both
match is `1/d^2`, which equals the product of the separate probabilities
`1/d`. All three pair-events together have probability `1/d^2`, because
they hold exactly when all three days are equal. The product of the three
separate probabilities is `1/d^3`. For `d = 5` those fractions are `1/25`
and `1/125`, which are not equal. The expected number of true pair-events
is still `3/5`. The probability of at least one collision is still
`13/25`. Definition N10.11 produces the first number. Definition N10.5
produces the second. Neither number is a misprint of the other. Listing
N10.9 checks these integers, the constant and sum clauses of Proposition
N10.13 on Example N10.1, and the trial counts of §N10.15.

### N10.13 The logarithm asks the inverse question

Chapter 2 fixed `2^k` as a product of `k` factors of two, and fixed
`2^0 = 1`. The powers increase: `2^(k+1) = 2*2^k`, which is strictly
larger than `2^k` for every `k ≥ 0`.

**Definition N10.12 — Base-two logarithm.** Let `N` be a positive integer.
If some integer `k ≥ 0` satisfies `2^k = N`, then `log2(N)` is that
integer `k`. If no such integer exists, this lesson does not assign
`log2(N)` a value. The lower bracket of `N` is the largest integer `k ≥ 0`
with `2^k ≤ N`. The upper bracket is the smallest integer `k ≥ 0` with
`2^k ≥ N`.

**Proposition N10.14 — The exponent is unique, and the brackets exist.**
There is at most one `k` with `2^k = N`. For every positive integer `N`,
both brackets exist, and they are equal exactly when `N` is a power of
two. In that case both equal `log2(N)`.

**Proof.** If `2^a = 2^b = N` and `a < b`, then `2^a < 2^b` because the
powers strictly increase, which contradicts equality. So the exponent is
unique when it exists. For the brackets, start at `2^0 = 1`. Repeated
doubling increases the value by at least `1` at every step, since
`2^(k+1) - 2^k = 2^k ≥ 1`. After enough doublings the power exceeds `N`.
Among the finitely many exponents whose powers are at most `N`, one is
largest. The next exponent is the upper bracket.

Thus `log2(1) = 0` and `log2(256) = 8`, because Chapter 2 already recorded
`2^8 = 256`. The integer `10` is not a power of two. Its brackets are `3`
and `4`, because `2^3 = 8 ≤ 10 ≤ 16 = 2^4`, and no integer lies strictly
between `3` and `4`. There is no power of two strictly between `8` and
`16`.

Sixteen bit positions allow `2^16` strings. Sixteen factors of two are two
groups of eight, so `2^16 = 2^8 * 2^8 = 256*256`. Chapter 2 already
computed `256 × 256 = 65,536` while counting byte-and-mask pairs. The
integer is the same. The interpretation here is different: it counts
sixteen-bit strings, not pairs of bytes.

**Listing N10.8 — `brackets.or`**

```orange
edition 2026;
module brackets {
  spec two_to_2() -> Int {
    for i in 0..2 with p: Int = 1 { p * 2 }
  }

  spec two_to_3() -> Int {
    for i in 0..3 with p: Int = 1 { p * 2 }
  }

  spec two_to_4() -> Int {
    for i in 0..4 with p: Int = 1 { p * 2 }
  }

  spec two_to_8() -> Int {
    for i in 0..8 with p: Int = 1 { p * 2 }
  }

  spec two_to_16() -> Int {
    for i in 0..16 with p: Int = 1 { p * 2 }
  }

  spec byte_length() -> Int { 8 }
  spec byte_support() -> Int { 256 }
  spec short_length() -> Int { 16 }
  spec short_support() -> Int { 4 }
  spec after_bit() -> Int { 2 }

  test "the powers that the brackets and the byte use" {
    (two_to_2() == 4)
      && (two_to_3() == 8)
      && (two_to_4() == 16)
      && (two_to_8() == 256)
      && (two_to_16() == (256 * 256))
      && (two_to_16() == 65536)
  }

  test "ten lies strictly between eight and sixteen" {
    (two_to_3() <= 10) && (10 <= two_to_4()) && (two_to_3() != 10) && (two_to_4() != 10)
  }

  test "length, support, and the announcement are three integers" {
    (byte_length() == 8)
      && (byte_support() == two_to_8())
      && (short_length() == 16)
      && (short_support() == two_to_2())
      && (short_length() != two_to_2())
      && (after_bit() == 2)
      && (after_bit() != short_length())
  }
}
```

**Expected evaluation output:**

```text
brackets::two_to_2: Int = 4
brackets::two_to_3: Int = 8
brackets::two_to_4: Int = 16
brackets::two_to_8: Int = 256
brackets::two_to_16: Int = 65536
brackets::byte_length: Int = 8
brackets::byte_support: Int = 256
brackets::short_length: Int = 16
brackets::short_support: Int = 4
brackets::after_bit: Int = 2
```

**Test report:**

```text
test "the powers that the brackets and the byte use" ... ok
test "ten lies strictly between eight and sixteen" ... ok
test "length, support, and the announcement are three integers" ... ok
3 tests: 3 passed, 0 failed
```

Each `two_to_k` multiplies `k` factors of two, starting from `1`. The
results are `4`, `8`, `16`, `256`, and `65536`. So `log2(4) = 2`,
`log2(8) = 3`, `log2(16) = 4`, `log2(256) = 8`, and `log2(65536) = 16`,
in the sense of Definition N10.12. The brackets of `10` are `3` and `4`
because `8 ≤ 10 ≤ 16` and neither power equals `10`. There is no integer
strictly between `3` and `4`, which is why the definition assigns no
`log2(10)`. The surface has no logarithm operation. The listing computes
the powers the definition names and compares them with `10`.

`byte_length` is `8` and `byte_support` is `256`. `short_length` is `16`
and `short_support` is `4`, which equals `2^2` and does not equal `16`.
`after_bit` is `2`, the size of the conditional list in §N10.14 after the
last-bit announcement. Proposition N10.15 says these quantities can vary
separately. The proof is the four specified spaces in that section. The
listing records the integers those spaces use. It does not search the
space of all distributions. A search over every finite list of weights is
not expressible on this surface.

`log2` of the number of strings in a format equals the number of bit
positions only for that full list of strings. It does not measure a
nonuniform distribution, and this lesson does not define a weighted
substitute for it. That later quantity is not introduced here, and it must
not be smuggled in under the name `log2`.

### N10.14 Length, distribution, and uncertainty

Three quantities have been used above. They are not synonyms.

The **key length** is the number of positions in the string, in the sense
of Chapter 2. A byte has length `8`. A sixteen-bit string has length `16`.

The **distribution** is the finite probability space from which the string
is drawn. Assumption N10.2 requires it to be stated.

The **adversary's uncertainty** is the conditional knowledge from
Definition N10.8, for a stated space and a stated event describing what
that adversary is assumed to know.

**Proposition N10.15 — The three quantities vary separately.** None of the
three determines the other two in general.

**Proof by three specified spaces.** First: the uniform distribution on
all `256` byte strings has length `8`, support `256`, and, for an adversary
who knows that distribution and not the draw, uncertainty `256` equal
weights of `1/256`. Second: the space whose only outcome is the byte
`0x00`, with weight `1`, still has length `8`. The distribution is one
row. An adversary who knows that distribution has a conditional list of
one row even before any further announcement. Third: Example N10.1 and the
uniform space on the same four strings have the same length and can receive
the same announcement, “the first bit is `0`.” Their conditional lists are
`3/4`, `1/4` and `1/2`, `1/2` respectively.

A fourth space shows length departing from the logarithm of the support.
Let the draw be uniform on the four strings `0x0000`, `0x0001`, `0x0002`,
and `0x0003`. Each weight is `1/4`. Each numeral is four hexadecimal
digits, and Chapter 2 assigns four bits to one hexadecimal digit, so the
length is `16`. The support has `4` strings, and `log2(4) = 2` because
`2^2 = 4`. The length is not `2`. The format of all sixteen-bit strings
has `65,536` members. This distribution uses four of them.

If this adversary knows the four-string distribution and has not seen the
draw, the uncertainty is those four strings, each of weight `1/4`. Suppose
the adversary is then told that the last bit is `0`. The rows `0x0000` and
`0x0002` remain. Their old weights sum to `1/2`, so each new weight is
`(1/4)/(1/2) = 1/2`. The length is still `16`. The original distribution
has not been edited. The conditional list has changed.

If the problem does not say that the adversary knows the support has only
four strings, Definition N10.8 does not authorize us to describe that
adversary's uncertainty by this conditional list. We would be supplying a
missing assumption.

### N10.15 Brute force counts trials

**Assumption N10.4 — Recognized search.** The candidate keys form a stated
finite list of size `N`, and the actual key is one of them. A trial of a
candidate returns the correct yes or no. Trials do not change the key. The
search does not repeat a candidate. The order of trials is fixed before
the key is drawn.

Shannon's sentence, quoted at the opening, separates two remarks: in
principle, trying keys can isolate a solution, and different systems
require different amounts of work. This section counts trials under
Assumption N10.4. It does not estimate hours, and it does not say that
trying keys is the best method, or a method you should use.

Under that assumption the exhaustive count of trials, and the worst-case
position of the correct key, are both `N`. Every candidate is tried once
in a complete pass, and the correct key might be last.

**Proposition N10.16 — Expected position.** Suppose in addition that the
key is uniform on the list. The trial number `T` on which the search
succeeds is uniform on `1` through `N`, and `E[T] = (N+1)/2`.

**Proof.** Each key occupies one fixed position and has weight `1/N`, so
each position has probability `1/N`. The expected value is
`(1/N)*(1 + 2 + ... + N)`. Proposition N10.9 converts the sum into
`N*(N+1)/2`, and multiplying by `1/N` leaves `(N+1)/2`.

For `N = 4`, the expected trial number is `5/2`. The probability that the
correct key is among the first two trials is `2/4 = 1/2`. The expectation
`5/2` is an average of the positions `1`, `2`, `3`, and `4`. It is not a
promise that the key appears by trial `2`. Stopping after two trials
succeeds with probability `1/2` in this model, not with probability greater
than `1/2`.

The base-two logarithm of a four-element list is `2`. That integer is not
`5/2`, and it is not the worst-case count `4`. It answers Definition
N10.12's question, how many times two must be multiplied to produce the
size of the list. It does not answer Proposition N10.16's question.

**Counterexample — no recognition.** Take one ciphertext byte and try all
`256` possible masks under XOR, an operation Chapter 3 already defined.
Each mask produces a definite byte. The trial count `256` is the size of
the format. Without a separate rule that accepts one of those bytes and
rejects the others, the list of results is not an identified message.
Assumption N10.4's recognition clause is doing real work. Removing it
removes the warrant for calling the count a count of trials until
identification. This paragraph is not an attack procedure and not a claim
about any cipher.

Searching the full list of sixteen-bit strings, while the distribution is
the four-string space in §N10.14, counts the format. It does not count the
distribution. After the last-bit announcement, the conditional list has
size `2`. Under Assumption N10.4 applied to that conditional uniform list,
the worst-case trial count is `2` and the expected trial count is
`(2+1)/2 = 3/2`.

**Listing N10.9 — `expect.or`**

```orange
edition 2026;
module expect {
  spec equal_parts(a: Int, b: Int, c: Int, d: Int) -> Bool {
    (a * d) == (b * c)
  }

  spec differs(a: Int, b: Int) -> Bool { a != b }

  spec constant_num() -> Int { 5 * 6 }
  spec value_num() -> Int { 3 * 1 }
  spec other_num() -> Int { 1 * 1 + 1 * 1 + 1 * 1 }
  spec both_num() -> Int { 3 * 1 + 1 * 1 + 1 * 1 + 1 * 1 }
  spec den() -> Int { 6 }

  spec indicator_num() -> Int { 1 + 1 }

  spec trial_sum() -> Int {
    for i in 1..5 with s: Int = 0 { s + i }
  }

  spec remaining_sum() -> Int {
    for i in 1..3 with s: Int = 0 { s + i }
  }

  test "a constant factors out of the weights" {
    constant_num() == (5 * den())
  }

  test "the sum of the expectations is the expectation of the sum" {
    ((value_num() + other_num()) == both_num()) && (both_num() == den())
  }

  test "the second-bit indicator has the event's numerator" {
    indicator_num() == 2
  }

  test "three pair indicators expect three fifths, not thirteen twenty-fifths" {
    differs(3 * 25, 5 * 13) && differs(1 * 125, 25 * 1)
  }

  test "four uniform keys have expected trial number five halves" {
    (trial_sum() == 10) && ((2 * trial_sum()) == (4 * 5))
  }

  test "stopping after two of four trials has probability one half" {
    equal_parts(2, 4, 1, 2)
  }

  test "the logarithm two is neither five halves nor four" {
    differs(2 * 2, 5) && differs(2, 4)
  }

  test "the remaining list has expected trial number three halves" {
    (remaining_sum() == 3) && ((2 * remaining_sum()) == (2 * 3))
  }
}
```

**Expected evaluation output:**

```text
expect::constant_num: Int = 30
expect::value_num: Int = 3
expect::other_num: Int = 3
expect::both_num: Int = 6
expect::den: Int = 6
expect::indicator_num: Int = 2
expect::trial_sum: Int = 10
expect::remaining_sum: Int = 3
```

**Test report:**

```text
test "a constant factors out of the weights" ... ok
test "the sum of the expectations is the expectation of the sum" ... ok
test "the second-bit indicator has the event's numerator" ... ok
test "three pair indicators expect three fifths, not thirteen twenty-fifths" ... ok
test "four uniform keys have expected trial number five halves" ... ok
test "stopping after two of four trials has probability one half" ... ok
test "the logarithm two is neither five halves nor four" ... ok
test "the remaining list has expected trial number three halves" ... ok
8 tests: 8 passed, 0 failed
```

On Example N10.1's denominator `6`, a constant `5` contributes `5*6`.
That is `E[v] = c` for this space, the constant clause of Proposition
N10.13. `value_num` is the weight of `00` times `1`. `other_num` is the
sum of the other three weights times `1`. `both_num` adds the quantity
that is `1` on every row. The sum of the two expectations equals the
expectation of the sum, and that common numerator is `6`. The indicator
of the second bit has numerator `2`, the same numerator as `P(E)` in
Listing N10.2. The proof of Proposition N10.13 is the splitting of a
finite sum. The listing checks this space.

`differs(3 * 25, 5 * 13)` says `3/5` is not `13/25`. Those are the
expected number of colliding pairs and the collision probability for
three people and five days. `differs(1 * 125, 25 * 1)` says `1/25` is
not `1/125`, the triple pair-event against the product of the three
separate probabilities. Linearity does not need that product.

`trial_sum` adds `1` through `4`. Twice that sum equals `4*5`, which is
Proposition N10.16 at `N = 4`: `E[T] = 5/2`. Stopping after two trials
is `2/4 = 1/2`. The integer `2`, which is `log2(4)` by Listing N10.8, is
not `5/2` and is not the worst-case count `4`. `remaining_sum` adds `1`
through `2`, and twice that sum equals `2*3`, so the expected trial
count on the conditional list of size `2` is `3/2`.

The counterexample with no recognition still has a format of `256`
masks. Listing N10.8's `byte_support` is that count. Assumption N10.4's
recognition clause is a sentence. Its absence is not an integer the
listing can test. Removing the sentence removes the warrant for calling
`256` a count of trials until identification. The listing does not
supply the missing sentence.

### N10.16 What was proved, what was checked, and what was not built

The propositions state their assumptions and prove the stated comparisons.
The ledger records the rational values the reference test recomputes,
including the integer comparison that places the 23-and-365 collision
probability above `1/2`. That Python test does not execute Orange.
Listing N10.7 computes the same comparison as two `Int` values and prints
the `Bool`. A passing test confirms the `Bool` written in that listing.
It does not sample a key, and it does not establish a security claim.

Nothing above recommends a key length, asserts that a construction is fit
to deploy, or treats Shannon's work characteristic as a number we
computed. Conditional knowledge is a calculated list, not a report of
someone's beliefs. The birthday model is Assumption N10.3 and nothing
else.

### N10.17 Work at the desk

**Exercise N10.1 — Add on a common denominator.** Compute
`1/2 + 1/3 + 1/6`. Then compute `(1+1+1)/(2+3+6)`. Which result is the
sum from Definition N10.2?

**Exercise N10.2 — Reject a digit cancellation.** Is `16/64` equal to
`1/4`? Does striking the digit `6` apply Proposition N10.1? Is `12/24`
equal to the numeral produced by striking the digit `2`?

**Exercise N10.3 — Accept or reject the weights.** Three outcomes have
weights `1/2`, `1/3`, and `1/6`. Three other outcomes have weights `1/2`,
`1/3`, and `1/7`. Which list can be a finite probability space?

**Exercise N10.4 — Uniform is not a synonym for unknown.** Four two-bit
strings are listed and no weights are given. A reader assigns `1/4` to
each. Which assumption does that violate, and which extra sentence would
make `1/4` correct?

**Exercise N10.5 — Complement.** In Example N10.1, compute the probability
that the second bit is `1`, and the probability that it is not.

**Exercise N10.6 — Both directions.** In that same space, compute
`P(the key is 00 | the first bit is 0)` and
`P(the first bit is 0 | the key is 00)`.

**Exercise N10.7 — A zero-probability condition.** Why does Definition
N10.7 assign no value to a conditional probability given the event “the
key is `00` and the key is `11`”? What fails if the missing value is
filled in as zero?

**Exercise N10.8 — Disjoint is not independent.** Under the uniform
distribution on the four strings, let `A` say “first bit `0`” and let `C`
say “first bit `1`.” Compute `P(A and C)` and `P(A)*P(C)`. Are the events
independent?

**Exercise N10.9 — Draw without putting back.** Use Example N10.2. Compute
`P(the second key is A)`, `P(the second is A | the first is A)`, and
`P(the first is A)*P(the second is A)`. Are the two events independent?

**Exercise N10.10 — An independent bit beside a byte.** One uniform byte
and one independent uniform bit. How many combined outcomes are there?
What is the probability that the byte is zero and the bit is zero? Which
Chapter 2 count did you reuse?

**Exercise N10.11 — A five-day room.** Three people, five days, Assumption
N10.3. Compute the collision probability, `S`, and `S/(1+S)`. Check both
bounds against the exact probability.

**Exercise N10.12 — When the upper bound goes quiet.** Five people, five
days. Compute `S` and the exact collision probability. Once every event
is known to have probability at most `1`, what does the upper bound `S`
add?

**Exercise N10.13 — Bounds that straddle one half.** For 23 people and
365 days under Assumption N10.3, compute `S` and `S/(1+S)`. Do these two
bounds alone prove that the collision probability is greater than `1/2`?
What comparison does prove it, and what does that comparison not claim?

**Exercise N10.14 — The pair count is an expectation.** For three people
and five days, give the expected number of colliding unordered pairs and
the probability of at least one shared day. Why may the expectations be
added even though the three pair-events are not independent as a group?

**Exercise N10.15 — Brackets, not a bit length.** A draw is uniform on ten
explicitly named keys, each stored as one byte. Give the key length in
bits, the number of keys in the distribution, and the two brackets of
`10`. Why does this lesson not assign an integer `log2(10)`?

**Exercise N10.16 — Separate the three quantities, then count trials.**
The format is sixteen-bit strings. The draw is uniform on `0x0000`,
`0x0001`, `0x0002`, and `0x0003`. The adversary knows that distribution
and has not seen the draw. State the length, the distribution, and the
uncertainty. The adversary is then told that the last bit is `0`, and
Assumption N10.4 applies to the resulting conditional list. Give the size
of that list, the worst-case number of trials, and the expected trial
number. Why is `2`, the logarithm of the original four-key list, not that
expected trial number?

## Worked answers

**N10.1.** `1/2 + 1/3 + 1/6 = 3/6 + 2/6 + 1/6 = 6/6 = 1`. The other
numeral is `3/11`. Only the first is the sum from Definition N10.2.

**N10.2.** `16/64 = 1/4` because `16*4 = 64`. Striking the digit `6` is
not Proposition N10.1. The common factor is `16`. `12/24 = 1/2` because
`12*2 = 24`. Striking the digit `2` leaves `1/4`, and `12*4 = 48`, which
is not `24`.

**N10.3.** The weights `1/2`, `1/3`, and `1/6` are nonnegative and sum to
`1`, so they form a finite probability space once each is attached to one
outcome. The weights `1/2`, `1/3`, and `1/7` sum to `41/42`, not `1`.

**N10.4.** The assignment violates Assumption N10.2. The sentence “the
draw is uniform on these four strings” would make each weight `1/4`.

**N10.5.** The probability is `1/6 + 1/6 = 1/3`. The complement is `2/3`.
They sum to `1`, as Proposition N10.3 requires.

**N10.6.** `P(the key is 00 and the first bit is 0) = 1/2` and
`P(the first bit is 0) = 2/3`, so the first conditional probability is
`3/4`. The second is `(1/2)/(1/2) = 1`.

**N10.7.** No outcome is both `00` and `11`, so the conditioning event has
probability `0`. Definition N10.7 assigns no number. Assigning zero would
invent a quotient by zero, and Proposition N10.5 would have no retained
row on which to place total weight `1`.

**N10.8.** The intersection is empty, so its probability is `0`. The
product is `(1/2)*(1/2) = 1/4`. The events are not independent.

**N10.9.** `P(the second key is A) = 1/3`. Given that the first is `A`,
the probability that the second is `A` is `0`. The product of the separate
probabilities is `1/9`. The events are not independent.

**N10.10.** There are `256*2 = 512` combined outcomes. The stated pair has
weight `1/512`. The factor `256` is the number of byte strings from
Chapter 2. The factor `2` is the number of values of one bit.

**N10.11.** The collision probability is `13/25`. `S = 3/5`. The lower
bound is `3/8`. And `3/8 ≤ 13/25 ≤ 3/5`.

**N10.12.** `S = 2`. The exact collision probability is `601/625`. The
bound `2` is weaker than the bound `1` already proved for every event.
The lower bound `2/3` still sits below `601/625`.

**N10.13.** `S = 253/365` and `S/(1+S) = 253/618`. Because
`253/618 < 1/2 < 253/365`, the bounds alone do not prove a comparison
with `1/2`. The exact collision probability is greater than `1/2` because
twice the product of the twenty-two numerators `364` through `343` is
smaller than `365` to the 22nd power. That is a fact about Assumption
N10.3 with these two integers. It is not a fact about human birthdays and
not a key-length recommendation.

**N10.14.** The expected number of colliding unordered pairs is `3/5`.
The collision probability is `13/25`. Proposition N10.13 adds expected
values without an independence hypothesis. The three pair-events are not
independent as a group: for five days, the probability that all three
pairs match is `1/25`, while the product of the three separate
probabilities is `1/125`.

**N10.15.** A byte has length `8` by Definition 2.2. The distribution has
ten keys. The brackets of `10` are `3` and `4`, because `8 ≤ 10 ≤ 16` and
no integer exponent lies strictly between `3` and `4`. Definition N10.12
assigns `log2` only when a power of two equals the count. `10` is not such
a count.

**N10.16.** The length is `16`. The distribution is uniform on the four
named strings, each of weight `1/4`. Before the extra announcement, the
uncertainty is that same list. After “the last bit is `0`,” the remaining
outcomes are `0x0000` and `0x0002`, each of conditional weight `1/2`. The
conditional list has size `2`. The worst-case trial count is `2`. The
expected trial count is `3/2`. The logarithm `2` of the original support
is not `3/2`. On the original list it is also not the expected trial count
`5/2` and not the worst-case count `4`.

```text
n10-ledger
sum-half-third = 5/6
bad-numerator-sum = 2/5
three-fraction-sum = 1/1
rejected-weights = 41/42
conditional-unequal-00 = 3/4
conditional-unequal-01 = 1/4
conditional-uniform-00 = 1/2
conditional-reverse = 1/1
second-bit-one = 1/3
second-bit-complement = 2/3
disjoint-product = 1/4
draw-product = 1/9
byte-bit-count = 512/1
byte-bit-zero = 1/512
birthday-3-5-distinct = 12/25
birthday-3-5-collision = 13/25
birthday-3-5-upper = 3/5
birthday-3-5-lower = 3/8
birthday-5-5-collision = 601/625
birthday-5-5-lower = 2/3
birthday-23-upper = 253/365
birthday-23-lower = 253/618
expected-pairs-3-5 = 3/5
triple-pair-event = 1/25
triple-pair-product = 1/125
expected-trials-4 = 5/2
early-stop-4 = 1/2
expected-trials-remaining = 3/2
sixteen-bit-count = 65536/1
```

## Source note for the epigraph

**[S8] Claude E. Shannon.** “Communication Theory of Secrecy Systems.”
*Bell System Technical Journal* 28(4), 1949, pp. 656–715. The epigraph is
the first sentence on the page whose footer is 703, in §21, “The Work
Characteristic.” Wording and page footer were checked on 2026-10-05 against
the retypeset PDF hosted by the University of Wisconsin–Madison, the same
copy used for [S1]. This is a retypeset copy, not a scan of the 1949
printing. No translation is involved. The sentence says that trying each
possible key can determine the solutions in principle, and that the amount
of work varies. It is not a method prescribed by this lesson and not a
computed work characteristic.

Source: <https://pages.cs.wisc.edu/~rist/642-spring-2014/shannon-secrecy.pdf>

Epigraph verification establishes wording and attribution, not publication-
rights clearance.

## Source note for the listings

**[C1] Counting surface.** The listings use `Int` arithmetic as specified
in `docs/EXPRESSIONS_2026.md`; `Bool`, comparisons, and Euclidean `/`
as specified in `docs/CONDITIONS_2026.md`; `let` and fixed-length
arrays as N7 uses them; bounded `for` as specified in `docs/LOOPS_2026.md`;
and `test` as specified in `docs/TESTS_2026.md`. A loop bound is an integer
literal with `0 ≤ start < end ≤ 65536`. Comparing two expressions that are
only literals is `ORC0227`. A parameter of type `Int` gives the comparison
a type. `1 / 0` evaluates to `0`. The diagnostic quoted for Listing N10.6
is `ORC0225`. Implementation of these slices is not acceptance of the
proposals, and it adds no cryptographic claim.

## Evidence boundary

N10 adds sixteen exercises with worked answers. The rational ledger is
recomputed by `tools/test_book_foundations.py`. That check does not execute
Orange. The nine Orange listings are executed by
`compiler/crates/orangec/tests/book_novice.rs`. Eight of them check and
evaluate. `empty_sum` is rejected with `ORC0225` before any step. A
passing test is one `Bool`. It does not sample a key, and it does not
establish a cryptographic security claim. The listings do not draw an
outcome. A draw would require a source of choices this lesson does not
invent.

The lesson was drafted with Grok 4.7 on 2026-10-05 at the owner's
direction. It is stacked after N9 on `book/novice-journeyman-master-opening`.
N10 does not re-teach N9's vocabulary. Owner review is pending. No
key-length recommendation, deployment claim, or proof-checker acceptance
is made.
