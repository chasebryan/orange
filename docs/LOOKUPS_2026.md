# Orange 2026 lookups specification

Status: proposed S3g semantics under OEP-0010, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3g of Orange 2026: indices that depend on data,
such as the S-box lookup `s[a[i]]` of AES, each proved in range before
anything runs, and cheaper functional updates. It is a delta over the
proposed S3f rules in [`CONDITIONS_2026.md`](CONDITIONS_2026.md), which are a
delta over [`LOOPS_2026.md`](LOOPS_2026.md) and the documents it extends.
Everything those documents define and this one does not mention is
unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0010](governance/oeps/OEP-0010-orange-2026-lookups.md), which requires
OEP-0009. At that point it replaces the S3f and S3e clauses listed in section
10. Until then, the compiler behavior it describes exists so that the proposal
can be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`MODULES_2026.md`](MODULES_2026.md), proposed under OEP-0011, extends this
> document with programs of more than one module, in which a module names the
> modules it uses and calls their functions by module name, as in
> `sha256::compress(h, block)`. Every source this document accepts keeps its
> meaning under it.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Much of symmetric cryptography is written as tables. FIPS 197 defines AES's
SubBytes as "the S-box of Fig. 7" applied to each byte of the state, DES has
eight S-boxes of 64 entries, and the table-driven CRC reads one entry per
byte of its input. Through S3f an index could depend only on literals and loop
indices, so none of these could be written as its standard writes it.

S3g lets an index depend on data and keeps the promise that no index is ever
out of range while a program runs. The range of an index is now taken from
its type: a byte selects one of 256 entries, so it may index any table of 256
entries, and `x & 15` or `x >> 4` may index a table of 16.

```orange
spec sub_bytes(s: Word[8]^256, a: Word[8]^16) -> Word[8]^16 {
  for i in 0..16 with b: Word[8]^16 = a { b with [i] = s[a[i]] }
}

spec sub_word(s: Word[8]^256, w: Word[32]) -> Word[32] {
  ((s[w >> 24] as Word[32]) << 24)
    | ((s[(w >> 16) & 0xff] as Word[32]) << 16)
    | ((s[(w >> 8) & 0xff] as Word[32]) << 8)
    | (s[w & 0xff] as Word[32])
}
```

With S3g, AES-128 is an Orange module whose S-box is derived as FIPS 197
section 5.1.1 defines it, and `orangec eval` reproduces the examples of
Appendix B and Appendix C.1 byte for byte.

Three commitments shape every rule below.

- **Every index is proved in range.** A word index ranges over its type,
  narrowed by operators whose results provably stay smaller, so `s[a[i]]` is
  accepted for a table of 256 entries and rejected, with the range it would
  take, for a table of 255.
- **The rule is one a reader can apply.** Each operator's range follows from
  its operands' ranges by one line of section 5, and nothing is inferred from
  conditions, from other indices, or from values computed elsewhere.
- **Lookups are meaning, not timing.** A lookup keyed by a secret is the
  classic source of cache-timing leaks in table-driven code. S3g lets a
  specification say what the standard says; how a machine performs the
  lookup, and whether it may, belongs to the implementation stratum
  (section 11).

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2 is unchanged. The
accepted surface grows to indices whose first typed leaf is a word, in
selections and updates alike, and `Int` indices that convert words with
`as Int` or choose with conditionals. Every S3f form keeps its meaning.
Indices built from `Int` parameters, `Int` bindings, calls that give an `Int`,
or elements of `Int` arrays are still not accepted, because nothing bounds
them.

## 3. Grammar

No production, token, or reserved word is added or changed. S3g is a change
to the checking of indices and to evaluation costs only.

## 4. The type of an index

An index `b[x]` whose index is one integer literal is the S3d literal index,
with the S3d rules. Any other index x, in a selection `b[x]` or an update
`b with [x] = v`, is checked in place of step 2 of `LOOPS_2026.md` section 5
as follows:

1. The type of x's **first typed leaf** is found without reporting, as a
   comparison operand's is in `CONDITIONS_2026.md` section 5.
2. If that type is a word type `Word[n]`, x is a **word index** and is
   checked as a `Word[n]` expression. Its range is found by section 5.
3. Otherwise x is an **`Int` index** and is checked as an `Int` expression, as
   in S3f. Its range is found by section 6.

So `s[a[i]]` for a byte array `a`, `t[x & 15]` for a byte `x`, and
`t[w >> 24]` for a 32-bit word `w` are word indices; `k[(254 - i) / 8]` and
`t[(x as Int) + 128]` are `Int` indices. An index that is not well typed
reports its own error, as it would anywhere else, and its range is not
examined.

**Meaning.** A selection evaluates b, then x, and selects the element at the
position given by x's value, an unsigned integer for a word index. An update
evaluates b, then x, then v, as in S3e.

## 5. Ranges of word indices

The range of a word expression of type `Word[n]` is a least and a greatest
value, both from 0 through 2^n - 1. It is computed from the syntax, bottom up:

| Expression | Range |
| --- | --- |
| an integer literal k | k through k |
| `(a)` | a's range |
| `~a` | 2^n - 1 - hi(a) through 2^n - 1 - lo(a) |
| `a & b` | 0 through min(hi(a), hi(b)) |
| `a \| b` | max(lo(a), lo(b)) through m(max(hi(a), hi(b))) |
| `a ^ b` | 0 through m(max(hi(a), hi(b))) |
| `a >> k`, k a literal | lo(a) >> k through hi(a) >> k |
| `a << k`, k a literal | lo(a) << k through hi(a) << k, when hi(a) << k is below 2^n |
| `a + b` | lo(a) + lo(b) through hi(a) + hi(b), when that is below 2^n |
| `a - b` | lo(a) - hi(b) through hi(a) - lo(b), when lo(a) ≥ hi(b) |
| `a * b` | lo(a) · lo(b) through hi(a) · hi(b), when that is below 2^n |
| `a / b` | lo(a) / hi(b) through hi(a) / lo(b) when lo(b) > 0, else 0 through hi(a) |
| `a % b` | a's range when lo(b) > hi(a); else 0 through min(hi(a), hi(b) - 1) when lo(b) > 0; else 0 through hi(a) |
| a conditional | the least and greatest bounds of all its values |
| `a as Word[n]` from `Word[m]` | a's range as a `Word[m]` expression, when its greatest value is below 2^n |
| anything else | 0 through 2^n - 1 |

Here lo and hi are the least and greatest values of an operand's range, and
m(v) is the least value of the form 2^j - 1 that is at least v, the value
with every bit set up to v's highest set bit. Where the table's condition
fails, the operator could wrap, and the expression ranges over its whole
type. A rotation, a shift by an amount that is not a literal, a name, a call,
an element of an array, a loop, and a conversion from `Int` or from a wider
word whose range does not fit are "anything else".

Every row follows from the meaning of the operator in `EXPRESSIONS_2026.md`
and `CONDITIONS_2026.md`: `a & b` is at most either operand, `a / 0` is 0,
`a % 0` is a, and `a % b` is below b. So `x & 15` runs from 0 through 15,
`x >> 4` from 0 through 15 for a byte x, `(x & 15) + 16` from 16 through 31,
`~(x | 224)` from 0 through 31, and `(x & 15) - 1` over its whole type,
because it wraps when `x & 15` is 0.

If the greatest value is not below the array's length, that is `ORC0223` at
the index, naming the range: `t[x]` with a byte x and a table of 16 entries is
"this index runs from 0 through 255, out of range for `Word[8]^16`". A word
index's least value is never negative.

## 6. Ranges of `Int` indices

The static indices of `CONDITIONS_2026.md` section 9 may now also contain:

- a conversion `a as Int` whose operand is a word expression, with the range
  of section 5 for the operand's type, so `(x as Int) + 128` over a byte x
  runs from 128 through 383;
- a conversion `a as Int` whose operand is an `Int` expression, with its
  operand's range; and
- a conditional, whose range runs from the least to the greatest bound of all
  its values. Its conditions are checked as usual and do not narrow the
  range.

An `Int` index is built only from integer literals, indices of enclosing
loops, words converted with `as Int`, parentheses, prefix `-`, binary `+`,
`-`, `*`, `/`, and `%`, and conditionals. The first part that is anything
else is `ORC0226` at that part, "an `Int` index may use only integer
literals, loop indices, and words converted with `as Int`", labeled "this
`Int` has no bound". An `Int` parameter, an `Int` binding or accumulator, a
call that gives an `Int`, and an element of an `Int` array are such parts:
an `Int` can be as large as 16,384 bits, and nothing in the source bounds
it.

Ranges combine exactly as in S3e and S3f, and the range check and its
`ORC0223` message are unchanged.

## 7. Diagnostics and Typed Reference Core

The examination order of `CONDITIONS_2026.md` section 10 is unchanged. An
index is checked as its type and then its range is examined, so an index that
is not well typed reports its own error and no range error.

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0223` | semantic | also a word index, or an `Int` index with converted words or conditionals, whose range leaves the array; the note is now "every value an index can take, over every loop index and word in it, must select an element" |
| `ORC0226` | semantic | now only an `Int` index with a part that has no bound; the message, label, and note are those of section 6, and the note reads "every index is proved in range when the program is checked: a word index ranges over its type, and an `Int` index is built from integer literals, loop indices, and words converted with `as Int`, using `+`, `-`, `*`, `/`, `%`, and conditionals" |

Every other code keeps its meaning.

**Core.** The Core of `CONDITIONS_2026.md` section 11 is unchanged in shape.
A word index's nodes are followed, in postorder, by one `convert` node of type
`Int` from the index's word type, with the index's span, and the `select` or
`update` node consumes that node as its position. An `Int` index is as in S3f.
So every position in Core is an `Int`, and the evaluator's range check, which
fails closed on inconsistent Core, is unchanged.

## 8. Resource limits and failure

The S3f budgets remain. S3g adds or refines the following.

**Semantic events and Core nodes.** A word index costs the events of its
parts, as any expression does, and its `convert` node is one Core node, and so
one event. Finding an index's type and computing its range consume no events.

**Evaluation.** An `update` or `fill` of an array of n elements costs
⌈n/64⌉ steps beyond its operands', where S3e charged n: one step for up to 64
elements, and 4 for a table of 256. A loop may therefore update a table on
every iteration without the per-source budget of 1,048,576 steps being spent
on copying, and no update or fill costs more than it did in S3e. A word
index's `convert` node costs one step, as every conversion does. Every other
cost of `CONDITIONS_2026.md` section 12 is unchanged.

Exhausting any budget, and any allocation failure, yields one resource
diagnostic, no Core, and no value line. Inconsistent Core, such as a position
outside its array, must stop evaluation with no value.

## 9. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3g conformance runner
(`compiler/crates/orangec/tests/s3g_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3f runners: every rule needs command-line evidence that runs
twice with identical results, except where the index says unit evidence
alone, and every rule names unit tests declared exactly once in their
sources' test modules.

### S3g conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3G-TYPE-01` | Section 4 | An index has the type of its first typed leaf when that is a word type and is an `Int` otherwise, in selections and updates alike. | CLI and unit |
| `S3G-WORD-01` | Section 5 | A word index ranges over its type, narrowed exactly by the rows of section 5; a range that leaves the array is `ORC0223` naming it. | CLI and unit |
| `S3G-INT-01` | Section 6 | An `Int` index may convert words with `as Int` and choose with conditionals; a part with no bound is `ORC0226`. | CLI and unit |
| `S3G-EVAL-01` | Sections 4 to 6 | A lookup selects, and an update replaces, the element at its index's value; AES-128 matches FIPS 197 and the CRC-32 of "123456789" is 0xcbf43926. | CLI and unit |
| `S3G-DIAG-01` | Section 7 | Diagnostic order, messages, labels, notes, and spans are exactly as specified. | CLI and unit |
| `S3G-CORE-01` | Section 7 | A word index is followed in postorder by one `convert` node to `Int` with the index's span. | Unit and CLI observation |
| `S3G-RES-EVENT-01` | Section 8 | Semantic events and Core nodes for word indices are counted exactly. | Unit |
| `S3G-RES-STEP-01` | Section 8 | Updates and fills cost ⌈n/64⌉ steps and a word index's conversion one step. | Generated CLI and unit |
| `S3G-COMPAT-01` | Section 10 | S3f programs keep their meaning, messages, and costs, except as section 10 lists. | CLI and unit |
| `S3G-DETERMINISM-01` | Section 9 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 10. Relationship to S3f

When OEP-0010 is accepted, this document replaces these clauses of
`CONDITIONS_2026.md` and the documents it extends:

- step 2 of `LOOPS_2026.md` section 5 and the index of an update in its
  section 6, by section 4;
- the static indices of `CONDITIONS_2026.md` section 9, extended by sections
  5 and 6;
- the `ORC0223` and `ORC0226` rows, by section 7; and
- the cost of an `update` or `fill` node in `LOOPS_2026.md` section 10, by
  section 8.

Every source that S3f accepts is accepted by S3g with the same Core and the
same output bytes, because none of its indices is a word index. Its
evaluation costs no more steps, and fewer when it updates or fills an array
of more than one element. A source that S3f rejects gets the same diagnostics,
with these exceptions:

- an index whose first typed leaf is a word was checked as an `Int`, so it
  was `ORC0214` at that leaf, or `ORC0215` at a word operator such as `&`
  that `Int` does not have; it is now a word index, accepted or `ORC0223`;
- an index with a conversion to `Int` or a conditional was `ORC0226` at that
  part; it is now accepted or `ORC0223`;
- `ORC0226` has the new message, label, and note of section 6, where S3f
  said "an index may use only integer literals and loop indices", and
  `ORC0223` has the new note of section 7; and
- a source that exhausted the step budget through updates or fills may now
  finish under `eval`.

This slice also corrects S3f's recognition of `if`: before the word `as`, or
the word `with` followed by `[`, `if` starts a conditional when a `}` that
returns to that depth is followed by `else`, as it already did before `(`,
`-`, and `[`. So `if as { 1 } else { 0 }` is a conditional on a name spelled
`as`, and `if as Int` still converts a name spelled `if`.
`CONDITIONS_2026.md` sections 3 and 12 state the corrected rule.

## 11. Explicit non-claims and future work

This slice defines no index built from unbounded `Int` values, no index
narrowed by a condition (`if x < 16 { t[x] } else { 0 }` is rejected for a
table of 16, because the range of `x` is its type's), no signed words, no
index parameters that are themselves static, no table larger than 256
entries, and none of the exclusions of `CONDITIONS_2026.md` section 15 that
this document does not lift.

A lookup is a mathematical selection, not a memory access. A table lookup
keyed by a secret is the classic cache-timing leak of software AES, and
nothing here says how any machine would perform one, or in what time. S3f's
static indices are still recognizable by their syntax: an index is static
exactly when it is built from literals and loop indices with `+`, `-`, `*`,
`/`, and `%`. Every other accepted index may depend on data. A later decision
on the implementation stratum (D-004) and the native target (D-011) must say
what code generation does with such an index where it may be secret: for
example, compile it to a scan of the whole table, or reject it under a
constant-time profile. Until then Orange makes no timing, secrecy, or leakage
claim about any lookup, and no statement here concerns code generation.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
