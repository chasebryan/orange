---
number: OEP-0020
title: Orange 2026 known-answer tests and whole-value equality
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3q
related-decisions:
  - D-002
  - D-004
  - D-011
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
  - OEP-0003
  - OEP-0004
  - OEP-0005
  - OEP-0006
  - OEP-0007
  - OEP-0008
  - OEP-0009
  - OEP-0010
  - OEP-0011
  - OEP-0012
  - OEP-0013
  - OEP-0014
  - OEP-0015
  - OEP-0016
  - OEP-0017
  - OEP-0018
  - OEP-0019
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0020: Orange 2026 known-answer tests and whole-value equality

## Abstract

A module states what its functions must give as **known-answer tests**,
`test "TITLE" { claim }`, beside the functions they are about: a title that
says where the claim comes from, and a `Bool` expression with its own `let`
bindings. `==` and `!=` compare arrays and tuples **whole**, so a claim can
compare a block, a tag, or a state in one expression. `orangec test` checks
the program, runs its tests in order under one step budget, and reports each;
a failed `left == right` shows both values and where they first differ.

```orange
test "2.1.1: the quarter round" {
  quarter_round(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567)
    == (0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb)
}
```

```console
$ orangec test compiler/fixtures/s3q/valid-rfc8439-tests.or
test "2.1.1: the quarter round" ... ok
test "2.3.2: the block function" ... ok
...
7 tests: 7 passed, 0 failed
```

The normative text is [`docs/TESTS_2026.md`](../../TESTS_2026.md). An
implementation, five programs, and a 12-rule conformance runner accompany it
so that the proposal can be reviewed against running code. This proposal is
in **Review** and requires OEP-0019, which is also in review. It accepts no
D-004 candidate and gives the Typed Reference Core no canonical or proof
role.

## Motivation

Every cryptographic standard ends in numbers: RFC 8439 prints a key, a nonce,
and the expected bytes in every section and again in an appendix, and FIPS
197 walks through an AES encryption round by round. They are how an
implementer knows the code is the algorithm and not something near it.
Through S3p an Orange program could compute a known answer and print it, but
the claim that it matched the standard lived outside the program, in a
conformance runner that compared text. A reader of the program could not see
which values were checked, and a cryptanalyst who changed a function had no
way to ask the program whether it was still the algorithm.

The claims themselves were also awkward to write: `==` compared only scalars,
so a 64-byte block was checked element by element or not at all. Arrays and
tuples are the values cryptography is made of, and comparing them whole is
the comparison a standard means.

## Scope and non-goals

This proposal defines test declarations and their titles, how a test is
checked and lowered, `==` and `!=` on every type with their costs, the
`orangec test` command with its report, status, budget, and options, and the
diagnostic `ORC0242` for titles. It adds no token, reserved word, type, or
Core node: a test is a function without parameters of result `Bool`, and a
comparison of arrays or tuples is the existing comparison node.

It does not define tests with parameters, properties over many inputs, tests
that expect a failure or a diagnostic, running a used module's tests from its
user, selecting or skipping tests by title, an order on arrays or tuples, or
an `==` that stops at the first difference. It makes no timing, secrecy, or
leakage claim.

### Strata assumption

As for S3b through S3p, S3q assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A test is a claim
about the values of pure functions, and whole-value equality is equality of
those values part by part. S3q therefore has the same meaning under `ST-REL`,
`ST-UNI`, `ST-DUAL`, `ST-MIRROR`, and `ST-HOST`, and the source surface needs
no change. Under a stratum that separates specification from implementation,
tests are the natural place to state that an implementation agrees with its
specification on published inputs.

## Specification

[`docs/TESTS_2026.md`](../../TESTS_2026.md) is the complete normative text.
In summary:

- **Declarations.** `test "TITLE" { bindings; expression }` may stand
  anywhere among a module's functions. `test` is not reserved: elsewhere it is
  an ordinary name, so a function may still be called `test`.
- **Titles.** 1 through 128 bytes of printable ASCII with no backslash,
  unique within the module; each violation is `ORC0242` at the title.
- **Checking.** A test is checked as a function without parameters of result
  `Bool`; it calls the module's functions, their instances, and the functions
  of the modules it uses. Only the root module's tests are checked and run.
  In Core the root's tests follow its functions, each with its title, and
  evaluation never runs them.
- **Equality.** `==` and `!=` are defined for every type; arrays and tuples
  compare whole, typed by the other operand when one is written out. Two
  written-out operands are `ORC0227`. `<`, `<=`, `>`, and `>=` remain for
  `Int` and words only.
- **Costs.** A comparison compares every part, whether or not an earlier one
  differs: ceil(n / 64) steps for an array of n words or truth values, each
  pair's cost for numbers and residues, and the sum of the parts for a tuple.
- **Running.** `orangec test [--steps <N>] [--stats] <FILE>` writes `test
  "TITLE" ... ok` or `... FAILED` for each test in source order, the values
  and first difference of a failed `left == right`, and a count; it exits 0
  when every test passes and 1 when any fails. A test that stops ends the run
  with `ORC0301` at its title and no report. The library gains `run_tests`,
  `TestRun`, and `TestOutcome`.

## Alternatives

A separate test file, as many languages use, was rejected: a known answer
belongs beside the function it checks, where a reader of the algorithm sees
it, and a separate file would need its own way to name the module under test.

Tests as ordinary functions with a naming convention, such as `test_*`,
were rejected: a title can say "RFC 8439 section 2.3.2" where an identifier
cannot, and a convention would make every existing function named `test_x`
change meaning.

Reserving the word `test` was rejected: every program that used it as a name
would break, and a test declaration is recognized by position alone.

An `==` on arrays that stops at the first difference was rejected: its cost
would depend on where the operands differ, which is a timing channel in
waiting when the operands are a computed tag and a received one. Comparing
every part costs the same wherever the difference is.

Diagnostics for failed tests were rejected: a failed claim is a result of the
program, not an error in it. The report goes to standard output, and the
status says whether every claim held.

## Compatibility and migration

Every source that S3p accepts has no test declaration, and is accepted with
the same Core values, output bytes, and steps. A source that S3p rejects gets
the same diagnostics, except that a module member beginning `test "` is now a
test; `==` and `!=` on arrays and tuples are now defined; a comparison whose
first typed leaf was written out is now typed by its other operand; and an
order on arrays or tuples is `ORC0215` with new notes.

`orangec` gains the command `test`, its usage line, and `ORC0242`. `--steps`
and `--stats` now apply to `eval` and `test`, so the usage error with another
command reads "option `--steps` applies only to eval and test"; `--spec`
stays with `eval`. `--stats` is described as reporting "the steps each
function or test used". `orangec lex` and the sealing commands are unchanged.

The public Rust API gains `run_tests`, `TestRun`, `TestOutcome`,
`TestDeclaration`, `TestTitle`, `CoreModule::tests`, `CoreFunction::title`,
`MAX_TEST_TITLE_BYTES`, and `DiagnosticCode::TestTitle`.
`CoreModule::entry_functions` no longer returns the root's tests.

Rollback reverts the parser, analyzer, Core, evaluator, command-line
interface, tests, fixtures, and normative documents together.

## Semantic and claim effects

This proposal adds a declaration form that states claims and a comparison of
compound values. The supported claim remains deterministic, bounded analysis
and evaluation of the documented fragment at a recorded implementation
revision. A passing test is evidence about the reference evaluator at one
revision; it establishes no soundness, proof, refinement, compilation,
cryptographic correctness, constant-time behavior, compatibility, independent
review, or production readiness.

## TCB, axiom, and proof effects

The lexer, parser, analyzer, Core constructor, evaluator, and command-line
interface remain engineering trust dependencies. The test parser, the title
checks, whole-value comparison, the test runner, and the report are new
trusted code. Unit tests check exact spans and messages for declarations and
titles, checking and lowering, comparisons of every type and their costs,
test runs in order with their sides and steps, stopping, storage failure,
the report and first difference, and option parsing. No axiom, theorem,
proof rule, certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

Titles are bounded at 128 bytes and checked with one semantic event each;
duplicates are found from a sorted index, so n titles cost O(n log n)
comparisons. A comparison reads its operands in place and makes no value, and
its steps depend only on its operands' type and size. A test run reserves
its outcomes, titles, and compared values before writing them, and a failure
is `ORC0301` with no partial report. Tests share the one step budget of the
run, bounded as for `eval`.

The report prints computed values. A test over a secret key prints the values
it compared when it fails; tests are for published known answers, and the
reference evaluator is not constant-time.

## Target and ABI effects

None.

## Standards, errata, and provenance

The RFC 8439 fixture states section 2.1.1 (the quarter round), section 2.3.2
(the block function), appendix A.1 test vectors 1 and 2 (the zero key's key
stream), section 2.5.2 (Poly1305 of "Cryptographic Forum Research Group"),
and appendix A.3 test vector 1 as tests, with inputs and expected bytes as
the RFC prints them. No standard gains normative authority through this
proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3q_conformance.rs` binds the 12 rules of the
specification's index to evidence and fails on any drift. Five programs run
through `orangec check`, `test`, and `eval` twice each, and generated tests
exercise titles at and past their limits, root-only checking, a test that
stops, every option and usage error, and comparisons of 65,536 elements
differing anywhere. Unit tests
cover parsing, titles, checking, lowering, comparison and its cost, test
runs, the report, options, and allocation. The S2 through S3p runners, the
`orangec enc` tests, and the algorithms corpus continue to pass.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether a test should be able to claim that a call stops, or that a source
  is rejected with a given diagnostic, so negative vectors live in the
  program too.
- Whether a module's user should be able to run the tests of the modules it
  uses, and how a report would name them.
- Whether tests should take parameters drawn from a table, so one claim
  covers every row of a standard's vector file.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue, with the vision of a technical and beautiful
language for cryptographers, cryptologists, and cryptanalysts. Known answers
written in the program, beside the algorithm they check, were the next
candidate after S3p's long arrays. This proposal records the S3q surface
built under that direction and is presented for the owner's review. It is not
accepted. Acceptance is the owner's decision alone; until it is recorded here
with a decision date, reviewed revision, and `solo-reviewed` approval record,
this proposal authorizes nothing by itself.
