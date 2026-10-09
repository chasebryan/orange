# Orange 2026 lengths and evaluation controls specification

Status: proposed S3p semantics under OEP-0019, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3p of Orange 2026: **long arrays**, which lets
an array, an array literal, and a byte string hold up to 65,536 elements, as
many as a loop visits and as many values as a 16-bit word takes; and
**evaluation controls**, which let `orangec eval` run a program under a
larger step budget, evaluate only the functions it names, and report the
steps each one used. It is a delta over the proposed S3o rules in
[`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md), which are a delta over
[`ORDER_2026.md`](ORDER_2026.md) and the documents it extends. Everything
those documents define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0019](governance/oeps/OEP-0019-orange-2026-lengths.md), which requires
OEP-0018. At that point it replaces the S3o clauses listed in section 10.
Until then, the compiler behavior it describes exists so that the proposal
can be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`TESTS_2026.md`](TESTS_2026.md), proposed under OEP-0020, extends this
> document with known-answer tests and whole-value equality: `orangec test`
> takes `--steps` and `--stats` as `eval` does and runs a module's tests under
> one budget, `--spec` stays with `eval`, the usage error for `--steps` or
> `--stats` with another command names both, and two arrays of 65,536
> elements compare in 1,024 steps. Every source this document accepts keeps
> its meaning under it.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

The objects of cryptography outgrew 256 bytes long ago. An ML-KEM-512
encapsulation key is 800 bytes and its ciphertext 768 (FIPS 203, section 8);
ML-KEM-1024's decapsulation key is 3,168. An ML-DSA-44 public key is 1,312
bytes and its signature 2,420 (FIPS 204, section 4). An RSA-2048 modulus, and
every OAEP block under it, is 256 bytes, and RSA-4096's are 512. Even RFC 8439
prints test vectors of 375 and 265 bytes. Through S3o an array held at most
256 elements, so every one of these had to be cut into pieces and carried as
a head and a tail.

S3p lifts the limit to 65,536, which is 2^16: the most iterations a loop has
always had, and exactly the values of a 16-bit word, so a `Word[16]` indexes
an array of 65,536 elements with no check at run time. Nothing else about
arrays changes. The standards' vectors are written as the standards print
them:

```orange
// RFC 8439 appendix A.5: 265 bytes of ciphertext, sixteen to a line.
spec a5_ciphertext() -> Word[8]^265 {
  hex"64 a0 86 15 75 86 1a f4 60 f0 62 c7 9b e6 43 bd" ++
    hex"5e 80 5c fd 34 5c f3 89 f1 08 67 0a c7 6c 8c b2" ++
    ...
    hex"a6 ad 5c b4 02 2b 02 70 9b"
}

// Section 2.4 for a message of `blocks` whole blocks, up to 16 KiB.
spec encrypt[blocks in 1..257](
  key: Word[8]^32, counter: Word[32], nonce: Word[8]^12, plaintext: Word[8]^(64 * blocks),
) -> Word[8]^(64 * blocks) { ... }

// Appendix A.2, test vector 2: 375 bytes, padded to six blocks and cut back.
encrypt(key, 1, nonce, ietf() ++ [0; 9])[..375]
```

Longer computations need a larger budget than the 1,048,576 steps a source
has by default, and a reader of a long program wants to run one part of it
and see what it costs. `orangec eval` gains three options:

```console
$ orangec eval --steps 2097152 --spec pepin --stats compiler/fixtures/s3p/valid-lengths.or
lengths::pepin: (Mod[65537], Mod[65537], Bool) = (65536, 21846, true)
lengths::pepin: 1452583 steps
total: 1452583 of 2097152 steps
```

The first line is standard output; the step report is written to standard
error.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3o extends it, is
unchanged: every length is known and checked before anything runs. S3p adds
no syntax, no token, no reserved word, no Core node, and no language
diagnostic code. It adds one run-time failure, a conversion of words to a
number larger than the evaluator's exact-integer limit (section 5), and one
command-line diagnostic code, `ORC1016` (section 7). Every S3o source keeps
its meaning (section 10).

## 3. Lengths

An array type `T^n` has **1 through 65,536** elements. The limit is
`MAX_ARRAY_LENGTH` of the reference implementation and equals the loop bound,
`MAX_LOOP_BOUND`. It applies to every way a length arises:

- **Written lengths.** A length written as an integer is decimal, without a
  sign, an underscore, or a leading zero, and from 1 through 65,536; any
  other is `ORC0221`, "an array length must be a decimal integer from 1
  through 65536", at the length.
- **Lengths written with sizes.** A length computed in an instance of a sized
  function outside 1 through 65,536 is `ORC0221`, "this array length is N,
  but an array has 1 through 65536 elements", with the note naming the same
  range (`SIZES_2026.md` section 5).
- **Array literals.** A literal lists at most 65,536 elements; a 65,537th is
  the parser's `ORC0106`, "array literal has more than 65536 elements", at
  that element, so a literal can spell every admitted array type and no
  longer one.
- **Byte strings.** A byte string holds 1 through 65,536 bytes; a 65,537th is
  `ORC0221`, "a byte string holds at most 65536 bytes", at the string, with
  the note "a byte string is an array `Word[8]^n` of 1 through 65536 bytes;
  join longer runs with `++`". Decoding reserves storage for no more bytes
  than the string's spelling has characters, at most 65,536, and stops at the
  65,537th byte, so a long spelling is never decoded in full.
- **Fills, joins, slices, and slice updates.** Each has the length its type
  gives, so each reaches 65,536 and none passes it: a join whose parts sum
  past the length expected of it is `ORC0222`, as before.

Size parameters are unchanged: a size's bounds are at most 65,536 and a
function has at most 256 instances. A size therefore reaches every admitted
length, but one function covers at most 256 of them. A function over whole
blocks, as `encrypt[blocks in 1..257]` above, covers every message of up to
256 blocks; a message that ends inside a block is padded to the block's end
and cut back with a slice.

## 4. Indices

Index and slice proofs are those of `LOOKUPS_2026.md` and `BYTES_2026.md`,
unchanged: every value an index can take, over every loop index and word in
it, must select an element, and every element a slice can take must be one.
At the new limit they give:

- a `Word[16]` index, whose values are 0 through 65,535, is in range of every
  array of 65,536 elements and of no shorter one, which is `ORC0223`, "this
  index runs from 0 through 65535, out of range for `Word[8]^65535`";
- a loop index of `for i in 0..65536` indexes an array of 65,536 elements;
- a `Word[32]` or `Word[64]` index is in range of no array, since no array
  has 2^32 elements.

A lookup in a table of 65,536 entries by a 16-bit word, such as a 16-bit
S-box or a table of the powers of a generator modulo the Fermat prime
2^16 + 1, is therefore checked before anything runs, as a lookup by a byte in
a table of 256 always was.

## 5. Conversions in a byte order

A conversion in a byte order (`ORDER_2026.md`) reads words of any admitted
length and writes words of any width with the same bits: 8,192 words of 64
bits are 65,536 bytes, and back. Through S3o no array held more than 16,384
bits, so a conversion to `Int` could never exceed the evaluator's
exact-integer limit of 16,384 significant bits. Arrays now hold up to
4,194,304 bits, and the number that words spell is an `Int` like any other:

- words convert to `Int` or `Mod[m]` while the number they spell has at most
  16,384 significant bits, leading zero words not counting;
- a larger number stops the evaluation at the conversion with `ORC0301`,
  "exact integer result exceeds the 16384-significant-bit limit", labeled
  "result is too large for the reference evaluator", with the evaluated
  function as a secondary span and the note that `Int` is unbounded and this
  is a resource limit, as for every other exact integer (`EXPRESSIONS_2026.md`
  section 14); and
- a number converts to words of any width, which hold its residue modulo 2 to
  the power of their width, as before.

This is a run-time failure, not a type error, because it depends on the
value: 2,049 bytes led by a zero byte spell 2^16384 − 1 and convert, and led
by a one byte they spell a number of 16,385 bits and stop.

## 6. Costs

The step table of `EXPRESSIONS_2026.md` section 14, as the later slices extend
it, is unchanged at every length. In particular, an update, a fill, a join, a
slice, and a slice update of an array of n elements each cost ceil(n / 64)
steps; a conversion in a byte order costs one step for each 64 bits of its
width. An array literal costs n steps beyond its elements, so a list of n
one-step literals costs 2n steps. Two consequences follow.

- **Memory stays proportional to steps.** Every operation that makes an
  array makes at most 64 elements for each step it costs, as it did at 256,
  so the memory an evaluation uses is bounded by its budget.
- **A long array is built in rows.** An update of one element of an array of
  65,536 costs 1,024 steps, so filling such an array one element at a time
  costs 65,536 × 1,024 = 67,108,864 steps. Filling rows of 256 in arrays of
  their own and placing each with one slice update costs 256 × (256 × 4 +
  1,024) = 524,288 steps for the updates, and the rows' own work. The
  fixture `valid-lengths.or` builds a table of 65,536 residues that way in
  about 1.45 million steps.

## 7. Evaluation controls

`orangec eval` takes three options, before or after its command, each only
with `eval`:

- **`--steps N`** sets the step budget of the whole evaluation of the source,
  shared by every function it evaluates in source order. N is a decimal
  number without sign or leading zero, from 1 through 1,073,741,824 (1,024
  times the default); any other value is a usage error, exit status 2, "option
  `--steps` takes a number of steps from 1 through 1073741824", and a second
  `--steps` is "option `--steps` may be specified at most once". Without it
  the budget is 1,048,576. Exceeding the budget is `ORC0301`, "reference
  evaluation step limit exceeded", at the function it stopped, with the notes
  "at most N evaluation steps are permitted" and "no partial value set is
  returned"; while N is below the maximum, `orangec` adds the note
  "`orangec eval --steps N` sets the budget, up to 1073741824 steps".
- **`--spec NAME`** evaluates only the root module's functions without
  parameters named NAME, every instance of a sized or typed one, in source
  order; the others are checked and not evaluated. It may be given up to 64
  times with distinct names, and a repeated name counts once. NAME is an
  Orange identifier, an ASCII letter or `_` followed by ASCII letters, digits,
  and `_`; any other is a usage error, "option `--spec` takes the name of a
  function", and a 65th name is "option `--spec` names at most 64 functions".
  A NAME that names no function without parameters of the root module is
  `ORC1016`, "module `m` has no function `NAME` without parameters", with the
  note "`--spec` names a function of the evaluated module that takes no
  parameters; a function with sizes is evaluated in every instance"; nothing
  is evaluated and the exit status is 1.
- **`--stats`** writes to standard error, after the values, one line for each
  evaluated function, `module::name[instance]: N steps` (`1 step` for one),
  in the order the values are printed, and a last line `total: T of B steps`,
  where T is their sum and B the budget. It is written only when the
  evaluation succeeds, and only after its values have been written to
  standard output in full, so a terminal shows the report below them; if
  they cannot be written, no report is.

Any of the three with `check`, `lex`, or a sealing command is a usage error,
"option `--steps` applies only to eval" (naming the option given). The step
counts are those of the cost table and are deterministic: the same source,
options, and edition give the same values and the same report.

The library exposes the same controls: `evaluate_selected(core, step_limit,
select)` evaluates the root module's functions without parameters that
`select` admits within `step_limit` steps together, and
`EvaluatedFunction::steps` returns the steps each used. `evaluate(core)` is
`evaluate_selected` admitting every function within 1,048,576 steps.

## 8. Resource limits and failure

The S3o budgets remain. S3p adds or refines the following.

- Each written length, literal element, and byte of a byte string costs what
  it did: the per-source limits on tokens, syntax nodes, semantic events, and
  Core nodes bound every long literal, and a literal of 65,536 elements is a
  flat list that adds no nesting.
- An evaluation under `--steps N` makes at most 64 elements for each of its N
  steps (section 6) besides its source's own literals. A budget of
  1,073,741,824 steps may therefore ask for more memory than a machine has;
  every array and integer is reserved before it is written, and a failure to
  reserve is `ORC0301`, "reference evaluation result allocation failed", with
  no partial output.
- The `--spec` list holds at most 64 names of at most the argument limit's
  bytes, reserved before it is written; a failure is a usage error.
- The step report is formatted after evaluation from the values' own counts
  and adds no evaluation step.
- A sealing scheme of `orangec enc` (`compiler/schemes/`) may seal chunks of
  up to 65,536 bytes with their tag, since its chunk is an array; the file
  format and its header are unchanged.

## 9. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged, and extends to the step report: the same source, options, and
edition give the same diagnostics, Core, output bytes, and report. The S3p
conformance runner (`compiler/crates/orangec/tests/s3p_conformance.rs`)
parses this index and requires exact agreement with its evidence map, under
the same rules as the S3b through S3o runners.

### S3p conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3P-LENGTH-01` | Section 3 | An array type, a length written with sizes, a fill, a join, a slice, and a slice update have 1 through 65,536 elements, written lengths in decimal of at most five digits without a leading zero; any other is `ORC0221` or `ORC0222` with the specified messages. | CLI and unit |
| `S3P-LITERAL-01` | Section 3 | An array literal lists at most 65,536 elements, a 65,537th being `ORC0106` at that element, and a byte string holds 1 through 65,536 bytes, a 65,537th being `ORC0221` at the string with decoding stopped there. | CLI and parser unit |
| `S3P-INDEX-01` | Section 4 | Index and slice proofs are unchanged at the new limit: a 16-bit word or a loop index through 65,535 indexes an array of 65,536 elements, and an index or slice that can leave its array is `ORC0223` with the specified message. | CLI and unit |
| `S3P-ORDER-01` | Section 5 | Words of any admitted length convert to words of every width, and to `Int` or `Mod[m]` while the number has at most 16,384 significant bits; a larger number stops at the conversion with the specified `ORC0301`. | CLI and unit |
| `S3P-COST-01` | Section 6 | Costs follow the unchanged table at every length, and RFC 8439's long vectors, A.2 test vector 2, A.3 test vectors 2 and 3, and A.5, reproduce the RFC byte for byte. | CLI and unit |
| `S3P-STEPS-01` | Section 7 | `orangec eval --steps N` sets the budget of the whole evaluation from 1 through 1,073,741,824, once; other values are the specified usage errors; exceeding the budget is `ORC0301` at the function it stopped, with the budget and the option named in its notes. | Generated CLI and unit |
| `S3P-SPEC-01` | Section 7 | `--spec NAME` evaluates only the root module's named functions without parameters, every instance, in source order, for at most 64 distinct names; a malformed name is a usage error and a name that matches none is `ORC1016` with nothing evaluated. | Generated CLI and unit |
| `S3P-STATS-01` | Section 7 | `--stats` writes each evaluated function's steps and their total against the budget to standard error after the values, and each of the three options with a command other than `eval` is the specified usage error. | Generated CLI and unit |
| `S3P-RES-01` | Section 8 | Long arrays keep allocation within 64 elements per step, reservation failures give no partial output, and the `--spec` list is bounded. | Unit |
| `S3P-COMPAT-01` | Section 10 | S3o sources keep their meaning, Core values, and output bytes, with only the specified diagnostic changes. | CLI and unit |
| `S3P-DETERMINISM-01` | Section 9 | Repeated identical inputs produce identical status, diagnostics, output bytes, and step reports. | CLI and unit |

## 10. Relationship to S3o

When OEP-0019 is accepted, this document replaces these clauses of
`TYPE_PARAMETERS_2026.md` and the documents it extends:

- the length limit of 256 of `ARRAYS_2026.md` sections 4 and 9, of
  `LOOPS_2026.md` section 7 for fills, of `BYTES_2026.md` sections 5 and 11
  for byte strings, joins, and slices, and of `SIZES_2026.md` section 5 for
  lengths written with sizes, by section 3;
- the widths of `ORDER_2026.md` sections 4 and 8, where every width is at
  most 16,384 bits and a conversion to `Int` never exceeds the
  significant-bit limit, by section 5; and
- the single, fixed budget of `EXPRESSIONS_2026.md` section 14 as
  `orangec eval` applies it, and the command line of the compiler guide, by
  section 7.

Every source that S3o accepts has arrays of at most 256 elements, which S3p
accepts with the same Core values, the same output bytes, and the same steps.
A source that S3o rejects gets the same diagnostics, with these exceptions:

- a length, literal, byte string, fill, join, or slice of 257 through 65,536
  elements was `ORC0221`, `ORC0106`, or `ORC0222`; it is now accepted where
  its types agree;
- every message and note that named the limit 256 now names 65536, as "an
  array length must be a decimal integer from 1 through 65536";
- a `Word[16]` index into an array of 257 through 65,536 elements was
  `ORC0223` or, for a length past 256, preceded by `ORC0221`; into an array of
  65,536 elements it is now in range; and
- a conversion of words wider than 16,384 bits to `Int` or `Mod[m]` could not
  be written; it is now accepted and stops at run time only if its value
  exceeds the limit.

`orangec` gains the options of section 7 and the code `ORC1016`, its usage
text gains the line `orangec eval [--steps <N>] [--spec <NAME>]... [--stats]
<FILE>` and the three options, and a step-limit diagnostic it prints gains the
note naming `--steps`. `orangec enc` reads a sealed file whose header names a
chunk of up to 65,536 sealed bytes, where it refused more than 256. `orangec
lex` is unchanged.

## 11. Explicit non-claims and future work

This slice defines no array longer than 65,536 elements, no length known only
at run time, no array of arrays, no change to how many instances a function
has, no budget per function or per call, no budget for checking (which the
per-source limits still fix), and no change to the cost of any operation. The
step report counts the reference evaluator's steps; it is not a measure of
time on any machine, of a native implementation's cost, or of side channels,
and the reference evaluator is not constant-time (`CONDITIONS_2026.md`
section 15).

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
