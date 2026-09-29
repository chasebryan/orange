# D-006 v0.3 shared statements

Status: foundation-neutral shared input for the D-006 `d006-v0.3` suite;
frozen by the epoch that binds its digest

This file fixes the meaning of every symbol, statement and fixture in the
JSON files beside it. The JSON files carry the machine-readable terms. The
reference in [`tools/d006_shared.py`](../../../../../tools/d006_shared.py) is
an executable reading of this file that computes every expected observation;
it is an oracle for observations only and is never part of either
candidate's proofs. A disagreement between this file, the JSON and the
reference is a shared-input defect: it is repaired for both candidates and
opens a new epoch.

## 1. Term syntax

Every statement, observation and negative case is a term written as a JSON
array whose first element names its form.

| Form | Meaning |
| --- | --- |
| `["sym", ID]` | The candidate declaration mapped to shared symbol `ID` |
| `["var", NAME]` | A bound variable |
| `["nat", DIGITS]` | A natural-number literal in decimal |
| `["bytes", HEX]` | A `List (Word 8)` literal, one element per hex byte pair |
| `["str", TEXT]` | A `String` literal; every shared string is ASCII |
| `["list", TYPE, ITEMS]` | A `List TYPE` literal built from `C-08` and `C-09` |
| `["app", F, ARGS...]` | Application of `F` to its arguments in order |
| `["forall", BINDERS, BODY]` and `["exists", BINDERS, BODY]` | Quantifiers over `[NAME, TYPE]` binders |
| `["imp", A, B]` | Implication, or the function type from `A` to `B` |
| `["and", ...]`, `["or", ...]`, `["not", A]`, `["iff", A, B]` | Connectives; `and` and `or` take two or more parts |
| `["eq", A, B]`, `["ne", A, B]` | Equality and its negation |
| `["lt", A, B]`, `["le", A, B]`, `["nat_add", A, B]` | Order and addition on naturals |
| `["hole"]` | An argument deliberately left for the candidate's inference |
| `["sort", "Prop"]`, `["sort", "Type"]` | Propositions and small types |

A shared symbol's type lists its arguments in order. Terms supply every
listed argument explicitly, type and width arguments included, so a
candidate renders a symbol in its fully explicit form. A candidate maps each
symbol to exactly one declaration. Its rendering of a statement must check
against the declaration the candidate names for that statement; a weaker,
stronger or reshaped statement is a failed parity check, not a variance.

## 2. DS-01 Core fragment

`ds01-core-fragment.json` lists the symbols with their types and meanings.
`Word w` has exactly one value for each natural below `2^w`, and every word
operation is the arithmetic its meaning states: addition is modulo `2^w`,
`word_not` is `(2^w - 1) - x`, `word_rotl` rotates by `r mod w` and is the
identity at width 0, and `word_shl` is `(x * 2^r) mod 2^w`.
`decode_words` reads a count byte `n`. It fails with `too_many` when `n`
exceeds 16, with `truncated` when fewer than `4n` bytes follow (or the input
is empty), and with `trailing` when more follow. Otherwise it returns the `n`
big-endian words in order. `encode_words` writes the count modulo 256 and
then each word big-endian. `QuarterRound` (M-01) is one parameterized
construct instantiated twice; its definition is in the JSON.

Observations are closed equations `lhs = rhs`. The candidate proves each by
its declared computation method, and that method's evaluator is listed in
its trust inventory. Negative cases take one of these forms:

- `term`: the rendered term must be refused.
- `obligation`: the equation, proved by the computation method, must fail.
- `exhaustive`: the statement, proved by the candidate's declared exhaustive
  evaluation method, must end within the negative-case ceiling as a timeout
  or resource exhaustion.
- `recursive_definition`: the rendered recursive definition must be refused.
- `axiom_use`: an axiom of `False` is declared and used to prove the
  statement; the build may succeed, but the trust audit must report it.
- `candidate_patch`: the candidate supplies a patch to its own source that
  makes the stated change; the affected theorems must stop checking.

## 3. DS-02 Sieve

Sieve is a suite-only language with 8-bit words, booleans, variables,
fixed-size arrays of words, bounded loops, public branches and one release
construct. The constructors are listed in `ds02-sieve.json`.

### 3.1 Typing

A label is `public` or `secret`; the join of two labels is `secret` when
either is. An environment lists declared variables (a type and a label each)
and declared arrays (a size and a label each). Expression typing gives a
type and a label, or nothing:

- A literal has its value's type and label `public`.
- `var x` has the declared type and label of `x`, and nothing when `x` is
  undeclared.
- `add`, `xor` and `band` take two words and give a word; `eq` takes two
  words and gives a bool. The label is the join of the operands' labels.
- `get a i` needs a declared array `a` and a `public` word `i`, and gives a
  word with the array's label.

`well_typed` is true exactly when:

- `skip`: always.
- `assign x e`: `x` is declared, `e` has `x`'s type, and `e`'s label is
  `public` or `x` is `secret`.
- `set a i v`: `a` is declared, `i` is a `public` word, `v` is a word, and
  `v`'s label is `public` or `a` is `secret`.
- `seq c1 c2`: both parts are well typed.
- `cond e c1 c2`: `e` is a `public` bool and both branches are well typed.
- `loop n c`: `c` is well typed.
- `declassify x e`: `x` is declared and `public`, and `e` has `x`'s type at
  any label.

### 3.2 Evaluation

A state lists the variables' values and the arrays' contents. Evaluation of
an expression gives `ok v t`, `fail t` or `stuck`, where `t` is the list of
observations in evaluation order. Operands evaluate left to right; the first
`fail` or `stuck` ends evaluation, and a failure keeps the observations made
before it.

- A literal gives itself with no observations.
- `var x` gives the value of `x`, or `stuck` when `x` has no value.
- An operator applied to two words gives the word or bool result
  (addition is modulo 256); any other operand values are `stuck`.
- `get a i` with `i` evaluating to word `n`: when `n` is below the length of
  array `a` it gives element `n` and appends `read a n`; otherwise it fails
  after appending `oob a n`. A non-word index or a missing array is `stuck`.

`step` gives `done`, `next c' s' t`, `fail t` or `stuck`:

- `skip` is `done`.
- `assign x e` with `e` giving `ok v t` sets `x` to `v` and gives
  `next skip s' t`; a missing `x` is `stuck`.
- `set a i v` evaluates `i`, then `v`, keeping `i`'s observations before
  `v`'s. With word index `n` and word value
  `m`, it writes `m` at `n` and appends `write a n`, or fails after appending
  `oob a n` when `n` is not below the array's length. A non-word index or
  value, or a missing array, is `stuck`.
- `seq skip c2` is `next c2 s []`. Otherwise `seq c1 c2` steps `c1` and
  wraps the new command as `seq c1' c2`; a failure or `stuck` passes through.
- `cond e c1 c2` with `e` giving bool `b` is `next` of the taken branch with
  `branch b` appended; a word condition is `stuck`.
- `loop 0 c` is `next skip s []`, and `loop (n + 1) c` is
  `next (seq c (loop n c)) s []`.
- `declassify x e` behaves like `assign x e` and appends `release v`.

In every case an expression failure gives `fail t` and an expression `stuck`
gives `stuck`. `run n s c` stops with `out_of_fuel c s` when `n` is 0, with
`halted s` on `done`, with `failed` on `fail t` (keeping `t`), and with
`wedged` on `stuck`; otherwise it appends the step's observations and
continues with `n - 1`. `run_trace` and `run_outcome` are its two parts.

### 3.3 Well-formedness and public equivalence

`wf g s` holds when the state has exactly as many variables and arrays as
the environment declares, each variable's value has its declared type, and
each array has its declared size. `low_eq g s1 s2` holds when the two states
agree on every declared `public` variable and every declared `public`
array. `outcome_low_eq g o1 o2` holds when both outcomes are `halted` with
`low_eq` states, both are `failed`, both are `out_of_fuel` with the same
command and `low_eq` states, or both are `wedged`.

## 4. DS-03 canonical records (OCR1)

An OCR1 input is at most 4096 bytes: the magic `OCR`, version byte `1`, a
record count, and the records. Numbers are unsigned and one or two bytes: a
byte below `0x80` is its own value, and a first byte `b0` of `0x80` or more is
followed by `b1` giving `(b0 - 0x80) + 128 * b1`. A second byte of `0x80` or
more is `malformed_number`; a second byte of zero is `noncanonical_number`,
because the value has a shorter encoding.

A record is a tag, a name and a body. The name is a length from 1 to 200
followed by that many bytes of well-formed UTF-8 (the table of RFC 3629,
section 4: no overlong forms, no surrogates, nothing above U+10FFFF). Names
are strictly increasing in bytewise order. Tag 1 is a definition with a
32-byte digest. Tag 2 is a theorem with a 32-byte fingerprint and up to 64
strictly increasing references to earlier records. Tag 3 is a claim with one
reference to an earlier theorem and a level from 0 to 3.

Decoding runs left to right and reports the first failure as a code and a
path. It first refuses inputs over 4096 bytes (`oversized`, path `input`).
The magic and version fail with `bad_magic`, `unknown_version` or
`truncated` at `header`; the count fails at `count` and is `oversized` above
64. Record `k` fails at `records/k/tag` (`unknown_field` for any other tag),
`records/k/name` (`invalid_name` for a length of 0 or over 200,
`invalid_utf8`, `duplicate_name`, or `noncanonical_order` when the name sorts
before the previous one), `records/k/digest`, `records/k/fingerprint`,
`records/k/refs` (`oversized` above 64), `records/k/refs/j` or
`records/k/ref`, and `records/k/level` (`invalid_level` above 3). A
reference at or beyond the count is `reference_escape`; one at or beyond its
own record's index is `cyclic_reference`; a theorem reference that does not
increase is `noncanonical_order`; and a claim citing anything but a theorem
is `reference_kind`. Running out of bytes anywhere is `truncated` at the
current path, and bytes after the last record are `trailing_data` at
`trailing`.

`records_valid` holds exactly when there are at most 64 records, every name
is 1 to 200 bytes of well-formed UTF-8, the names strictly increase, every
digest and fingerprint is 32 bytes, every theorem has at most 64 strictly
increasing references each below its own index, every claim's reference is
below its own index and names a theorem, every level is at most 3, and the
encoding is at most 4096 bytes. Candidates define it from these conditions,
not from the decoder, so that D3-TH01 and D3-TH03 together say the decoder
accepts exactly the valid lists. (The reference computes it through its codec,
which is equivalent.) The standalone verdict
line for an accepted input is `accept` followed by one field per record:
`def:NAME:DIGEST`, `thm:NAME:FINGERPRINT:REFS` or `claim:NAME:REF:LEVEL`,
with names and digests in lowercase hex and references as comma-separated
decimals. A refused input gives `reject CODE PATH`.

## 5. DS-04 certificate replay

The canonical bit-blast, the CNF text, the LRAT grammar and the reverse unit
propagation rule are the numbered rules in `ds04-lrat-obligation.json`. Each
line of a certificate is checked in this order: its syntax (`parse`), its
identifier (`id_order`), its literals (`var_range`, then `lemma_form` for a
repeated or complementary literal), its hints' signs (`rat_unsupported`),
and then its hints in order (`unknown_hint` for a clause that is not active,
`rup` for a failed propagation). A deletion of a clause that is not active is
`unknown_deletion`, and any line after the empty clause is `trailing`.

## 6. Diagnostics

`taxonomy.json` lists the shared diagnostic categories. A candidate's
diagnostic for a negative case must name the case's shared ID (in its file
name or message), one expected category, and a source location, in at most
64 KiB of output. A crash, a hang past the ceiling, or success is never a
rejection.
