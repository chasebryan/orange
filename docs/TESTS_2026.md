# Orange 2026 known-answer tests specification

Status: proposed S3q semantics under OEP-0020, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3q of Orange 2026: **known-answer tests**, which
let a module state what its functions must give, as `test "TITLE" { claim }`,
and **whole-value equality**, which lets `==` and `!=` compare arrays and
tuples whole. `orangec test` runs a module's tests and reports each one. It is
a delta over the proposed S3p rules in [`LENGTHS_2026.md`](LENGTHS_2026.md),
which are a delta over [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md)
and the documents it extends. Everything those documents define and this one
does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0020](governance/oeps/OEP-0020-orange-2026-tests.md), which requires
OEP-0019. At that point it replaces the S3p clauses listed in section 11.
Until then, the compiler behavior it describes exists so that the proposal
can be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`AMOUNTS_2026.md`](AMOUNTS_2026.md), proposed under OEP-0021, extends this
> document with shift and rotation amounts computed from data, so a test may
> claim what a word shifted by any `Int` or word gives. Every source this
> document accepts keeps its meaning under it.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Every cryptographic standard ends in numbers: a key, a nonce, a message, and
the bytes an implementation must produce from them. RFC 8439 prints them in
every section and again in an appendix; FIPS 197 walks through a whole AES
encryption round by round. They are how an implementer knows the code is the
algorithm and not something near it. Through S3p an Orange program could
compute a known answer and print it, but the claim that it matched the
standard lived outside the program, in a runner that compared the output.

S3q puts the claim in the program, beside the functions it is about:

```orange
test "2.1.1: the quarter round" {
  quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
    == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
}

test "A.1 #2: the zero key's key stream, block 1" {
  let expected: Word[8]^64 =
    hex"9f 07 e7 be 55 51 38 7a 98 ba 97 7c 73 2d 08 0d" ++ ...;
  block([0; 32], 1, [0; 12]) == expected
}
```

A test is a claim: a `Bool` expression over the module's functions, with its
own `let` bindings, and a title that says where the claim comes from. The
claims above compare a tuple and an array whole, which no earlier slice
allowed; S3q defines `==` and `!=` for every type. `orangec test` checks the
program, runs the tests in order, and reports each:

```console
$ orangec test compiler/fixtures/s3q/valid-rfc8439-tests.or
test "2.1.1: the quarter round" ... ok
test "2.3.2: the block function" ... ok
...
7 tests: 7 passed, 0 failed
```

A test that fails by `==` shows both values and where they first differ, so a
wrong byte deep in a block is found at once:

```text
test "a tuple holding an array" ... FAILED
    left:  (0x01, [0x02, 0x03, 0x04])
    right: (0x01, [0x02, 0x03, 0x05])
    first difference at .1[2]
```

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3p extends it, is
unchanged: a test is checked completely before anything runs, and running it
is evaluation under the same rules as a function's. S3q adds one declaration
form (section 3), one language diagnostic code, `ORC0242` (section 4), the
comparison of arrays and tuples (section 6), and one command, `orangec test`
(section 8). It adds no token, no reserved word, no type, and no Core node:
a test is checked and lowered as a function without parameters, and a
comparison of arrays or tuples is the existing comparison node at an array or
tuple type.

## 3. Test declarations

The module production of `MODULAR_2026.md` section 3 gains tests, which may
stand anywhere among a module's functions:

```text
Module     = "module" Identifier "{" UseDecl* TypeDecl* Member* "}" ;
Member     = Function | TestDecl ;
TestDecl   = "test" STRING "{" binding* expression "}" ;
```

`STRING` is the token of `LANGUAGE_2026.md` section 2.4, and `binding` and
`expression` are those of a typed body (`BINDINGS_2026.md` section 3, as later
slices extend them).

- **The word.** `test` is not reserved. Where a module member may begin, an
  identifier spelled `test` begins a test declaration; everywhere else it is
  an ordinary name, so a function, a parameter, or a binding may be named
  `test`, and `test()` calls a function named `test`.
- **The title.** A quoted string follows the word: the title, whose text is
  the characters between the quotes as written. A title that is missing, or
  a hexadecimal string `hex"..."`, is the parser's `ORC0101`, "expected a
  quoted title after `test`", with the note "a test is written `test
  \"TITLE\" { EXPRESSION }`, its expression a `Bool`". Section 4 says which
  titles are admitted.
- **The body.** Braces follow the title and hold `let` bindings, if any, and
  one expression, exactly as a `spec` body does. A missing `{` is `ORC0101`,
  "expected `{` after the test's title"; a body without an expression after
  its bindings is `ORC0101`, "expected a result expression after the last
  binding", with the note "a test's body ends with the `Bool` expression that
  decides it".

A test declaration counts toward the per-source limits of tokens and syntax
nodes as a function does. A test has no name a call can use, no parameters,
no sizes, and no result type written; it cannot be called.

## 4. Titles

A title names its test in every report, so it must be readable as written
and unique within its module. It must be **1 through 128 bytes of printable
ASCII**, `0x20` through `0x7E`, with **no backslash**, and **no two tests of
a module may share one**. Since a title holds no backslash, it holds no
escape and no quote, so a report can print it between quotes exactly as the
source spells it. Each violation is `ORC0242` at the title, with the note "a
test's title is 1 through 128 printable ASCII characters, with no backslash,
and no two tests of a module share one":

- an empty title: "this test's title is empty", labeled "a title names the
  test in every report";
- a title of more than 128 bytes: "this test's title is N bytes long";
- a backslash: "a test's title holds no backslash", labeled "titles have no
  escapes";
- any other byte outside `0x20` through `0x7E`: "a test's title holds
  U+XXXX, which is not printable ASCII", labeled "at byte K of the title",
  for the first such character, K counting from 0; and
- a title an earlier test of the module has: "two tests of this module share
  a title", labeled "this title repeats an earlier test's", with the earlier
  title marked "first test is here".

Titles are checked in source order, each before its test's body, and a test
whose title is in error is still checked. `MAX_TEST_TITLE_BYTES` is 128 in
the reference implementation.

## 5. Checking

A test is checked as a `spec` function without parameters whose result type
is `Bool`, named `test` at its title:

- its body sees the module's type declarations, calls the module's
  functions, every instance of a sized or typed function fitted as any call
  is, and the functions of the modules it uses, qualified as `m::f(...)`;
- its expression must be a `Bool`; any other is the diagnostic an expected
  `Bool` gives, such as `ORC0214`, "an integer literal cannot have type
  `Bool`"; and
- no call reaches a test, so a test adds nothing to the module's call graph
  and cannot take part in a cycle.

**Only the root module's tests are checked and run.** A program is checked
from its root, the file given to `orangec`; the modules it uses are checked
for what the root can call, and their tests are neither checked nor lowered.
A module's own tests are checked when it is the root, as `orangec check
m.or` or `orangec test m.or`.

In Typed Reference Core, the root's tests follow its functions and their
instances, in source order, each a function without parameters of result
`Bool` whose name is `test` and whose title is kept. `CoreModule::tests`
returns them, and `CoreModule::entry_functions` no longer includes them, so
evaluation (`orangec eval`, `evaluate`, `evaluate_selected`) never runs a
test.

## 6. Whole-value equality

`==` and `!=` are defined for **every type**. For `Int`, words, `Bool`, and
`Mod[m]` they are unchanged. For an array or a tuple they compare the values
whole: two arrays of one type are equal exactly when every pair of elements
at the same index is equal, and two tuples of one type exactly when every
pair of parts at the same position is. `!=` is the negation of `==`.

The operands' type is found as S3f defines, from the first typed leaf of the
left operand, or else of the right. An array, a fill, or a tuple written out
takes its type from the other operand, so `x == [1, 2, 9, 4]`, `(1, 2) == p`,
and `mac(k, m, 2) == hex"a8 06 ..."` are all typed by their other side. Two
operands that are both written out have no type of their own: for `==` and
`!=` this is `ORC0227`, "the operands of `==` have no type of their own",
labeled "an array or tuple written out takes its type from where it is
used".

`<`, `<=`, `>`, and `>=` remain defined only for `Int` and words. On an array
or a tuple they are `ORC0215`, "`<` is not defined for `T`", with the note
"arrays are compared whole with `==` and `!=`; they have no order, so compare
elements, such as `x[0] < y[0]`" or "tuples are compared whole with `==` and
`!=`; they have no order, so compare elements, such as `p.0 < q.0`"; when
both operands are written out, "`<` is not defined for arrays and tuples",
with the note "arrays and tuples are compared whole with `==` and `!=`; they
have no order, so compare elements".

## 7. Costs

A comparison compares **every part** of its operands, whether or not an
earlier part differs, so its cost depends only on the operands' type and
size, never on where they differ:

| Operands | Steps |
| --- | --- |
| an array of n words or truth values | ceil(n / 64), at least 1, as an update of n elements |
| an array of numbers or residues | the sum of each pair's cost below |
| a tuple | the sum of its parts' costs |
| two `Int` values | one more than the 32-bit digits of the longer |
| two `Mod[m]` values | one more than the 32-bit digits of m |
| two words or two truth values | 1 |

The cost of a scalar comparison is unchanged. Comparing two arrays of 65,536
bytes costs 1,024 steps; comparing arrays of 256 bytes that differ in their
first byte costs what comparing ones that differ in their last does. The
reference evaluator is not constant-time (`CONDITIONS_2026.md` section 15),
and this rule is about steps, not time.

## 8. Running tests

`orangec test [--steps <N>] [--stats] <FILE>` checks exactly one source and
the modules it uses, as `orangec check` does, and then runs the root module's
tests in source order:

- **One budget.** The tests share one step budget, 1,048,576 steps unless
  `--steps N` sets it, as for `eval` (`LENGTHS_2026.md` section 7). Each test
  is evaluated as a function without parameters. A test **passes** when its
  expression is `true` and **fails** when it is `false`.
- **The report.** Standard output gets one line for each test, in source
  order, `test "TITLE" ... ok` or `test "TITLE" ... FAILED`, and a last line
  `N tests: P passed, F failed` (`1 test` for one). When a failed test's
  expression is a single `left == right`, two lines follow its line, each
  indented by four spaces: `left:` and two spaces before the left value, and
  `right:` and one space before the right, the values printed as `orangec
  eval` prints values. When the values are arrays or tuples, a third line
  follows, indented alike, `first difference at P`, where P names the first
  element that differs as `[i]`, or the first part as `.k`, followed into a
  part that is an array, as `.1[2]`. A test that fails otherwise, as
  `a && b`, has only its line.
- **Status.** `orangec test` exits with status 0 when every test passes,
  including when the module has none (`0 tests: 0 passed, 0 failed`), and
  with status 1 when any fails. A failed test is not a diagnostic: standard
  error stays empty.
- **A test that stops.** A test that exceeds the budget, or any other
  evaluation limit, stops the run. Nothing is written to standard output, the
  exit status is 1, and the diagnostic is `ORC0301` at the test's title,
  labeled "evaluation stopped while evaluating this test", with the limit's
  message and note followed by "no test outcome is reported". For the
  budget the note is "at most N evaluation steps are permitted", and while N
  is below the maximum, `orangec` adds "`orangec test --steps N` sets the
  budget, up to 1073741824 steps". A limit reached at one place in the
  source, an `Int` of more than 16,384 significant bits or a call nested
  more than 256 deep, marks that place with a secondary label, "result is
  too large for the reference evaluator" or "this call exceeds the depth
  limit", as `orangec eval` does.
- **`--stats`.** Standard error gets, after the report has been written to
  standard output in full, one line for each test, `test "TITLE": N steps`,
  and a last line `total: T of B steps`. If the report cannot be written, no
  step report is.
- **Options.** `--steps` and `--stats` apply to `eval` and `test`; `--spec`
  applies only to `eval`, which selects functions, and with `test` is the
  usage error "option `--spec` applies only to eval". `--steps` or `--stats`
  with `check`, `lex`, or a sealing command is "option `--steps` applies only
  to eval and test" (naming the option given). `test` without exactly one
  source is "command `test` requires exactly one source file", or "requires
  at least one source file" when none is given. Usage errors exit with
  status 2.

`orangec check` checks the root's tests and runs none; `orangec eval` runs
none. The library exposes `run_tests(core, step_limit)`, which returns a
`TestRun`: the outcomes in source order, each a `TestOutcome` with the test's
Core identity, its title, whether it passed, the two values a failed
`left == right` compared, and its steps; or, when a test stopped, no outcomes
and the diagnostic.

## 9. Resource limits and failure

The S3p budgets remain. S3q adds or refines the following.

- A title is at most 128 bytes, and its text is stored in storage reserved
  before it is written; a failure is the parser's resource-limit diagnostic.
- Titles are checked with one semantic event each, and duplicates are found
  from a sorted index of the module's titles, so checking n titles takes
  O(n log n) comparisons of at most 128 bytes.
- A comparison of arrays or tuples reads its operands in place and makes no
  new value.
- A test run reserves its outcomes, each title's copy, and the two values a
  failed `left == right` keeps before writing them; a failure is `ORC0301`,
  "reference evaluation result allocation failed", with the note "no test
  outcome is reported" and no partial report.

## 10. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged, and extends to tests: the same source, options, and edition give
the same diagnostics, Core, outcomes, report, and step report. The S3q
conformance runner (`compiler/crates/orangec/tests/s3q_conformance.rs`)
parses this index and requires exact agreement with its evidence map, under
the same rules as the S3b through S3p runners.

### S3q conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3Q-SYNTAX-01` | Section 3 | `test "TITLE" { bindings; expression }` is a module member wherever a function may be, `test` stays an ordinary name elsewhere, and a missing title or body is the specified `ORC0101`. | CLI and parser unit |
| `S3Q-TITLE-01` | Section 4 | A title of 1 through 128 printable ASCII bytes without a backslash, unique in its module, is admitted; any other is `ORC0242` at the title with the specified message, label, and note, in source order. | CLI and unit |
| `S3Q-CHECK-01` | Section 5 | A test is checked as a function without parameters of result `Bool` that calls the module's functions, their instances, and used modules' functions, and is lowered after the module's functions with its title kept; evaluation never runs it. | CLI and unit |
| `S3Q-ROOT-01` | Section 5 | Only the root module's tests are checked and run; a used module's tests are neither, and are checked when that module is the root. | Generated CLI and unit |
| `S3Q-EQUAL-01` | Section 6 | `==` and `!=` compare values of every type, arrays and tuples whole, typed by the other operand when one is written out; two written-out operands are `ORC0227`, and an order on arrays or tuples is `ORC0215` with the specified notes. | CLI and unit |
| `S3Q-COST-01` | Section 7 | A comparison compares every part at the specified cost, independent of where its operands differ, up to arrays of 65,536 elements. | CLI and unit |
| `S3Q-RUN-01` | Section 8 | `orangec test` runs the root's tests in source order under one budget and writes the specified report, with both values and the first difference of a failed `left == right`, exiting 0 when all pass and 1 when any fails. | CLI and unit |
| `S3Q-STOP-01` | Section 8 | A test that stops, at the budget or any other limit, ends the run with no report and `ORC0301` at its title with the specified label, secondary label, and notes, the budget note naming `orangec test --steps N`. | Generated CLI and unit |
| `S3Q-OPTIONS-01` | Section 8 | `--steps` and `--stats` apply to `eval` and `test`, `--spec` only to `eval`, `test` takes exactly one source, and the step report follows the committed report. | Generated CLI and unit |
| `S3Q-RES-01` | Section 9 | Title, outcome, and compared-value storage is reserved before it is written, and a failure gives no partial report. | Unit |
| `S3Q-COMPAT-01` | Section 11 | S3p sources keep their meaning, Core values, and output bytes, with only the specified diagnostic changes. | CLI and unit |
| `S3Q-DETERMINISM-01` | Section 10 | Repeated identical inputs produce identical status, diagnostics, reports, and step reports. | CLI and unit |

## 11. Relationship to S3p

When OEP-0020 is accepted, this document replaces these clauses of
`LENGTHS_2026.md` and the documents it extends:

- the module grammar of `MODULAR_2026.md` section 3, which admits `use` and
  `type` declarations and then functions, by section 3, which admits tests
  among the functions;
- the rule of `CONDITIONS_2026.md` section 5 and `TUPLES_2026.md` section 5
  that `==` and `!=` apply only to scalars, and that an array or tuple
  written out as a comparison's first typed leaf is an error, by section 6;
- the comparison costs of `CONDITIONS_2026.md` section 12, which S3q extends
  to arrays and tuples, by section 7; and
- the command line of `LENGTHS_2026.md` section 7, which S3q extends with
  `orangec test` and the options it takes, by section 8.

Every source that S3p accepts has no test declaration: at the start of a
member, an identifier `test` was `ORC0103`, "expected a `spec` or `impl`
function declaration". S3q accepts every such source with the same Core
values, the same output bytes, and the same steps. A source that S3p rejects
gets the same diagnostics, with these exceptions:

- a module member beginning with `test` and a string is now a test
  declaration;
- `==` or `!=` on arrays or tuples was `ORC0215`, "`==` is not defined for
  `T`", with the note "compare elements, such as `x[0] == y[0]`" or "such as
  `p.0 == q.0`"; it is now defined, and an order on them is `ORC0215` with
  the notes of section 6;
- a comparison whose first typed leaf was an array, a fill, or a tuple
  written out was `ORC0215`, "`==` is not defined for an array" or "for a
  tuple", at the operator; it is now typed by the other operand, and when
  the other is written out too it is `ORC0227` for `==` and `!=` and `ORC0215`,
  "`<` is not defined for arrays and tuples", for an order; and
- the notes of `ORC0215` for an order on arrays and tuples are those of
  section 6.

`orangec` gains the command `test` and its usage line `orangec test [--steps
<N>] [--stats] <FILE>`, `--stats` is described as "Report the steps each
function or test used, on stderr", and `--steps` or `--stats` with a command
that does not evaluate is "option `--steps` applies only to eval and test".
`orangec lex` is unchanged.

## 12. Explicit non-claims and future work

This slice defines no test with parameters, no property over many inputs, no
test that expects a failure or a diagnostic, no test of a used module run
from its user, no way to select, skip, or filter tests by title, no ordering
of arrays or tuples, and no `==` that stops at the first difference. A test
states one claim about one computation; it is evidence about the reference
evaluator at one revision, not a proof that a function meets its
specification for every input.

A test that reproduces a standard's example value is not thereby a verified
transcription of that standard, and a passing test establishes the tested
behavior of one implementation at one revision; it does not prove semantic
soundness, completeness, or implementation independence. The reference
evaluator is not constant-time.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
