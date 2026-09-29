# Orange 2026 pure expression specification

Status: proposed S3b semantics under OEP-0005, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-28

This document defines slice S3b of Orange 2026: parameters, calls, and pure
expressions over mathematical integers and fixed-width words. It is a delta
over the accepted S2 lexical and grammar rules in
[`LANGUAGE_2026.md`](LANGUAGE_2026.md) and the accepted S3a typed-literal
semantics in [`SEMANTICS_2026.md`](SEMANTICS_2026.md). Everything those
documents define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0005](governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md). At
that point it replaces the S3a clauses listed in section 16, and the two older
documents are read through it. Until then, the compiler behavior it describes
exists so that the proposal can be reviewed against running code, and it
establishes no accepted language meaning. It accepts no D-004 candidate.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

A cryptographic standard defines its functions with a handful of word
operations and prints every expression with its grouping visible. S3b gives
Orange exactly that much:

```orange
edition 2026;
module sha256 {
  spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
    (x & y) ^ (~x & z)
  }
  spec big_sigma0(x: Word[32]) -> Word[32] {
    (x >>> 2) ^ (x >>> 13) ^ (x >>> 22)
  }
  spec sample() -> Word[32] { big_sigma0(0x6a09_e667) }
}
```

`orangec eval` prints `sha256::sample: Word[32] = 0xce20b47e`.

Three commitments shape every rule below.

- **A word is an element of a ring.** `Word[n]` denotes the integers modulo
  2^n, and `+`, `-`, and `*` on words are the ring operations. Nothing wraps
  by accident, because wrapping is the meaning.
- **Grouping is always visible.** Operators from different families never
  share a level without parentheses. The reader never needs a precedence
  table beyond the one every reader already knows: `*` before `+` and `-`.
- **Every program terminates.** Specifications are pure, total, and
  acyclic. Evaluation is bounded, deterministic, and fails closed.

## 2. Scope and phase boundary

The phase boundary of `SEMANTICS_2026.md` section 1 is unchanged: a source
reaches semantic analysis only after lexing and parsing with zero diagnostics,
and success requires zero semantic diagnostics and a complete Typed Reference
Core. `orangec check` runs lexing, parsing, and semantic analysis. `orangec
eval` runs the same phases and then evaluates. No partial syntax tree, Core,
or value sequence is an accepted result.

The accepted surface grows to: parameters of typed `spec` functions, calls
between typed specifications, integer literals, parameter references, prefix
and binary operators, parentheses, the types `Int`, `Word[8]`, `Word[16]`,
`Word[32]`, and `Word[64]`, call-graph acyclicity, Typed Reference Core
expressions, and deterministic reference evaluation. Empty functions and `impl`
declarations keep their S3a meaning exactly: valid syntax, no value, no call.

## 3. Lexical additions

Four punctuation tokens are added to `LANGUAGE_2026.md` section 2.5:

```text
<<  >>  <<<  >>>
```

Longest-token matching applies: a three-character spelling is preferred to the
two-character spelling it extends, and a two-character spelling to a single
character. So `<<<<` lexes as `<<<` then `<`, and `>>>>>` as `>>>` then `>>`.
No other token, reserved word, trivia rule, or lexical limit changes.

The parser uses `,`, `:`, `+`, `-`, `*`, `&`, `|`, `^`, `~`, and the four new
tokens in addition to the S3a tokens. Every other punctuation token remains a
lexical reservation.

## 4. Grammar

The S2 `source_file`, `edition_decl`, and `module_decl` productions are
unchanged. `function_decl` becomes:

```text
function_decl   = "spec" IDENTIFIER "(" ")" spec_tail
                | "spec" IDENTIFIER "(" parameters ")" typed_tail
                | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail       = empty_body | typed_tail ;
typed_tail      = "->" parsed_type "{" expression "}" ;
empty_body      = "{" "}" ;
parameters      = parameter ("," parameter)* ","? ;
parameter       = IDENTIFIER ":" parsed_type ;
parsed_type     = IDENTIFIER ("[" INTEGER "]")? ;

expression      = arithmetic | chain("&") | chain("|") | chain("^") | shift ;
arithmetic      = product (("+" | "-") product)* ;
product         = prefixed ("*" prefixed)* ;
chain(op)       = prefixed (op prefixed)+ ;
shift           = prefixed shift_operator prefixed ;
shift_operator  = "<<" | ">>" | "<<<" | ">>>" ;
prefixed        = literal | ("-" | "~") prefixed | primary ;
literal         = "-"? INTEGER ;
primary         = IDENTIFIER | call | "(" expression ")" ;
call            = IDENTIFIER "(" arguments? ")" ;
arguments       = expression ("," expression)* ","? ;
```

A typed body contains exactly one expression. A `-` immediately followed by an
`INTEGER` token is the sign of that literal, exactly as in S3a, so the S3a body
`{ -42 }` is still one literal. Any other `-` in prefix position is negation.
Whitespace and comments between the sign and the digits do not change this.

An identifier followed by `(` is a call; any other identifier in an expression
is a name. A parameter list and an argument list may end with one comma. A
`spec` with parameters must have a typed tail; `spec f(x: Int) {}` is a syntax
error. An `impl` declaration must have an empty parameter list and an empty
body.

The syntax tree gains one parameter node per parameter, holding its name and
parsed type, and one expression node per literal, name, call, prefix operator,
binary operator, and group. Calls keep their callee and arguments in order, and
operator nodes keep the operator's own span. Every node keeps its exact source
span. As in S2, the tree records spelling and structure only: parsed types,
parameter names, and call targets are resolved in sections 6 through 9.

## 5. Grouping

The first binary operator after an expression's first operand selects its
**operator group**:

| Group | Operators | Shape |
| --- | --- | --- |
| Arithmetic | `+` `-` `*` | `*` binds more tightly than `+` and `-`; all three associate to the left |
| And | `&` | chains of `&` associate to the left |
| Or | `\|` | chains of `\|` associate to the left |
| Exclusive or | `^` | chains of `^` associate to the left |
| Shift and rotation | `<<` `>>` `<<<` `>>>` | exactly two operands, not associative |

Prefix operators bind more tightly than every binary operator. There is no
other precedence. A binary operator from a different group, or a second shift
or rotation, at the same level is a syntax error (`ORC0108`) at that operator,
naming it and the operator textually before it:

```orange
spec bad(x: Word[32]) -> Word[32] { x >>> 2 ^ x >>> 13 }   // ORC0108 at `^`
spec good(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) }
```

`a + b * c` is accepted and means `a + (b * c)`. `a ^ b & c`, `a + b ^ c`, and
`a << 1 << 2` are rejected. One ungrouped expression produces one `ORC0108`,
however many operators follow the first ungrouped one. Parsing then continues
from the end of that expression, so later functions are still checked.

## 6. Names, parameters, and calls

Function namespaces are unchanged from `SEMANTICS_2026.md` section 3: keys are
`(spec | impl, exact identifier spelling)`, and every later same-kind duplicate
is an error.

Parameter names are compared by exact ASCII spelling. Within one function, the
second and each later parameter with an existing name is an error (`ORC0218`).
Parameters are visible only in the body of their own function.

In an expression:

- a **name** refers to the parameter of the enclosing function with that
  spelling. Any other name is an error (`ORC0211`), including the name of a
  function written without parentheses.
- a **call** refers to the typed `spec` in the same module with that spelling,
  declared before or after the caller. A call to an empty `spec`, to an
  `impl`, or to no function is an error (`ORC0212`).
- a call must supply exactly one argument per parameter (`ORC0213`).

The **call graph** has one edge for each call that semantic analysis examines
and that names a typed `spec`, even when the call is also wrong in another way,
such as a type mismatch or a wrong argument count. It must be acyclic. A
function that reaches itself through calls, directly or through others, is an
error (`ORC0217`). Recursion is not part of Orange 2026, so every accepted
program terminates.

## 7. Types

The admitted parsed types are exactly:

| Source form | Core type | Values |
| --- | --- | --- |
| `Int` | `Int` | all mathematical integers |
| `Word[8]` | `Word8` | the integers modulo 2^8 |
| `Word[16]` | `Word16` | the integers modulo 2^16 |
| `Word[32]` | `Word32` | the integers modulo 2^32 |
| `Word[64]` | `Word64` | the integers modulo 2^64 |

Each word value is represented by its canonical residue from 0 through
2^n - 1. `Int` has no width. A `Word` width must be one of the decimal
spellings `8`, `16`, `32`, or `64`, with no base prefix, separator, sign, or
leading zero. Type identifiers are case-sensitive.

Every other parsed form is an error. `Word` without a width and a `Word` width
outside the four spellings are `ORC0204`; `Int[8]`, `int`, `Integer`, and every
other shape are `ORC0203`. There is no inference, alias, subtyping,
overloading, coercion, or implicit conversion.

## 8. Typing

Every expression is checked against an **expected type**. Nothing is inferred.

- A function body is checked against the declared result type.
- Each argument of a call is checked against the callee's parameter type in
  the same position, and the callee's result type must equal the expected type.
- A name must have exactly the expected type.
- A parenthesized expression is checked against the expected type.
- Both operands of `+`, `-`, `*`, `&`, `|`, and `^`, the operand of a prefix
  operator, and the left operand of a shift or rotation are checked against the
  expected type.

A mismatch is `ORC0214`.

An integer literal takes the expected type. Its magnitude is decoded exactly as
in `SEMANTICS_2026.md` section 5, including the 16,384-significant-bit source
limit (`ORC0205`). For `Int`, a signed literal denotes its mathematical value,
and every negative zero is zero. For `Word[n]`, a sign is an error even on
zero (`ORC0206`), and a magnitude above 2^n - 1 is an error (`ORC0207`).
Literals never wrap, truncate, or saturate.

Operators are defined only where this table defines them. Using one elsewhere
is `ORC0215`, reported at the operator:

| Expression | on `Int` | on `Word[n]` |
| --- | --- | --- |
| `a + b`, `a - b`, `a * b` | exact | modulo 2^n |
| `-a` | exact negation | not defined; write `0 - a` |
| `a & b`, `a \| b`, `a ^ b` | not defined | bitwise and, or, exclusive or |
| `~a` | not defined | bitwise complement |
| `a << k`, `a >> k` | not defined | logical shift left, right |
| `a <<< k`, `a >>> k` | not defined | rotation left, right |

The **amount** `k` of a shift or rotation is not an expression. It must be an
unsigned integer literal, written directly, whose value is 0 through n - 1.
Anything else, including a name, a call, a group, a signed literal, or `n`
itself, is `ORC0216`.

## 9. Meaning

Evaluation is call by value. Operands and arguments are evaluated left to
right, and a call evaluates the callee's body with its parameters bound to the
argument values.

For `Int`, `+`, `-`, `*`, and prefix `-` have their exact mathematical meaning.
`Int` is unbounded; the evaluator's magnitude limit in section 14 is a resource
boundary, not a width.

For `Word[n]`, with `m` = 2^n and every value a residue in 0 through m - 1:

| Expression | Value |
| --- | --- |
| `a + b`, `a - b`, `a * b` | the residue of the exact result modulo m |
| `a & b`, `a \| b`, `a ^ b` | the bitwise operation on the n-bit binary representations |
| `~a` | m - 1 - a |
| `a << k` | the residue of a · 2^k modulo m |
| `a >> k` | the floor of a / 2^k |
| `a <<< k` | `(a << k) \| (a >> (n - k))` when 0 < k, and `a` when k = 0 |
| `a >>> k` | `(a >> k) \| (a << (n - k))` when 0 < k, and `a` when k = 0 |

These are the word operations of FIPS 180-4 sections 2.2.2 and 3.2 and of
RFC 8439 section 2.1, written the way those documents write them. Orange does not claim that any
program transcribes a standard; see section 17.

## 10. Diagnostics

Semantic analysis examines a source in this deterministic order, which also
decides which diagnostics fall within the S3a budget of 100:

1. For each function in source order: a same-kind duplicate name; then, for a
   typed `spec`, each parameter in order (duplicate name, then type), the
   result type, and the body expression.
2. After every function: call cycles, in the order of a depth-first search that
   starts from each unvisited function in ID order and follows each function's
   calls in body postorder, a call's arguments before the call. A call that
   reaches a function still on the search path closes a cycle and is reported
   once, at that call, naming the cycle.

Within one body, each node is examined before its operands, and operands left
to right. A node that is wrong in itself stops there, and its operands are not
checked: an undefined operator, an unknown name, an unknown function, or a
wrong argument count. A call whose result type differs from the expected type
is reported at the call, and its arguments are still checked against the
callee's parameter types, because those errors are independent. If the result
type of a function is unsupported, its body is not checked. A reference to a
parameter whose type is unsupported, and a call to a function whose signature
contains an unsupported type, are not reported again.

Every diagnostic has a primary span on the responsible token or expression.
`orangec` renders diagnostics sorted by primary source position and then by
code, as it does for every phase, so a call cycle appears at its closing call's
place in the source. The new categories are:

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0108` | parse | operators from different groups, or two shifts, share a level without parentheses |
| `ORC0211` | semantic | a name is not a parameter of the enclosing function |
| `ORC0212` | semantic | a call names no typed `spec` in the module |
| `ORC0213` | semantic | a call has the wrong number of arguments |
| `ORC0214` | semantic | an expression's type differs from the expected type |
| `ORC0215` | semantic | an operator is not defined for the expected type |
| `ORC0216` | semantic | a shift or rotation amount is not a literal from 0 through n - 1 |
| `ORC0217` | semantic | a call cycle makes a function depend on itself |
| `ORC0218` | semantic | a parameter name repeats within one function |

`ORC0204` names the four admitted widths. `ORC0106` also reports the parser
limits of section 14, and `ORC0301` the evaluation limits. The S3a codes keep
their meanings.

## 11. Typed Reference Core

A successful analysis produces one Typed Reference Core module:

```text
core_module    = module_name core_function* ;
core_function  = function_id function_name parameter_type* core_type body ;
core_type      = Int | Word8 | Word16 | Word32 | Word64 ;
body           = core_node+ ;
core_node      = core_type node_kind ;
node_kind      = literal value
               | parameter index
               | call function_id argument_count
               | unary (negate | complement)
               | binary (add | subtract | multiply | and | or | xor)
               | shift (shl | shr | rotl | rotr) amount ;
```

As in S3a, only typed `spec` declarations produce Core functions, in source
order, with the contiguous IDs `0` through `n - 1`. A body is the postorder
sequence of its expression tree: each node follows its operands, and a call
follows its arguments. Parentheses produce no node. A shift amount is part of
its node, not an operand. Every node carries its type, and the last node's type
is the function's result type. An S3a literal function is the special case with
no parameters and one literal node.

The Core still has no canonical encoding, digest, proof identity, refinement or
erasure relation, target meaning, or cross-revision ID stability. It is not
declared to be Spec Core, Impl Core, or their shared fragment.

## 12. Reference evaluation and display

`orangec eval FILE` keeps the operand rules of `SEMANTICS_2026.md` section 8.
After successful analysis, the evaluator visits Core functions in ascending ID
order and evaluates each function **with no parameters**, writing one line:

```text
module::name: Type = value\n
```

Functions with parameters are checked but not printed; they run only when
called. A source whose Core has no parameterless function writes zero bytes.

`Int` display is unchanged. A `Word[n]` value is written as `0x` followed by
exactly n/4 lowercase hexadecimal digits, with leading zeroes:

```text
demo::byte: Word[8] = 0x0f
demo::half: Word[16] = 0x0f0f
demo::word: Word[32] = 0x6a09e667
demo::lane: Word[64] = 0x0000000000000001
```

If any evaluation fails, no value line is written.

## 13. Examples

The ChaCha20 quarter round of RFC 8439 section 2.1, one output word at a time:

```orange
edition 2026;
module chacha20 {
  spec a1(a: Word[32], b: Word[32]) -> Word[32] { a + b }
  spec d1(a: Word[32], b: Word[32], d: Word[32]) -> Word[32] { (d ^ a1(a, b)) <<< 16 }
  spec c1(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    c + d1(a, b, d)
  }
  spec b1(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    (b ^ c1(a, b, c, d)) <<< 12
  }
}
```

The permanent fixtures in `compiler/fixtures/s3b/` carry the whole quarter
round, the SHA-256 round functions, and their recorded values from the
standards' own examples.

## 14. Resource limits and failure

The S3a budgets of `SEMANTICS_2026.md` section 9 remain. S3b adds or refines
the following.

**Parsing.** Each of these is a parser resource limit (`ORC0106`):

- **Nesting.** Groups, call argument lists, and prefix operators each open
  one nesting level. An expression may be enclosed by at most 64 levels.
- **Height.** An expression tree may have height at most 256. A literal or
  name has height 1; a prefix operator, group, or binary operator adds one to
  its tallest operand; a call adds one to its tallest argument, and a call with
  no arguments has height 1.
- **Lists.** A function declares at most 64 parameters, and a call supplies at
  most 256 arguments.

Nesting bounds the parser's recursion; height bounds every later tree walk.
Operator chains are parsed by iteration, so a long chain consumes height, not
nesting.

**Semantic events.** The S3a events remain: one per declaration-key lookup,
one per successful namespace installation, one per parsed-type identifier and
per width token, one per literal, one for its base prefix (including the
decimal default), one per significant digit, one per diagnostic attempt, and
one per Core-node construction attempt. S3b adds one event per parameter-name
uniqueness check, one per name, one per call, one per group, and one per
operator. A shift amount counts as a literal. The call-graph search consumes no
events: it visits each function and each already counted call once.

**Core nodes.** The module is one node. Each Core function counts one node for
itself, one for its result type, one per parameter type, and one per body node.
An S3a literal function therefore still counts three.

**Evaluation.** All parameterless functions of one source share one budget of
1,048,576 steps:

| Core node | Steps |
| --- | --- |
| literal, parameter, call | 1 |
| any operator on `Word[n]` | 1 |
| `-a` on `Int` | 1 + d(a) |
| `a + b`, `a - b` on `Int` | 1 + max(d(a), d(b)) |
| `a * b` on `Int` | 1 + d(a) · d(b) |

Here d(x) is the number of 32-bit limbs in the magnitude of x, and d(0) is 0.
An S3a literal function therefore still costs one step. The call stack holds at
most 256 frames, counting the evaluated function as the first. An `Int` result
whose magnitude would exceed 16,384 significant bits stops evaluation. Each of
these is `ORC0301`. The step limit is reported at the function being
evaluated. The depth and magnitude limits are reported at the call or operation
that would exceed them, with the evaluated function as a secondary span.

An acyclic program can still ask for exponential work: a function that calls
another twice, twenty levels deep, requests about a million calls. The step
budget is what bounds it.

Exhausting any budget, and any allocation failure inside the budgets, yields
one resource diagnostic, no Core, and no value line. The deepest sources the
limits admit must parse, analyze, and evaluate within 1 MiB of native stack.

## 15. Determinism and conformance

For identical source bytes, edition, compiler revision, and command, success or
failure, every diagnostic's category, order, and spans, the Core, every value,
the exit status, and every output byte must be identical across repeated runs
on the same supported host.

Conformance includes positive, negative, boundary, resource, and repeatability
evidence for every rule below. The S3b conformance runner
(`compiler/crates/orangec/tests/s3b_conformance.rs`) parses this index and
requires exact agreement with its evidence map. Every rule needs at least one
command-line case, fixture or generated, that runs twice with identical
results, except where the index says unit evidence alone. Every rule also
names at least one unit test, and each named test must be declared exactly once
as a `#[test]` function in its source's test module. As in S3a, that
traceability check is not proof that a named test exhausts its rule.

### S3b conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3B-LEX-01` | Section 3 | `<<`, `>>`, `<<<`, and `>>>` are single tokens under longest match, and no other token changes. | Generated CLI and unit |
| `S3B-GRAMMAR-01` | Section 4 | Parameters, calls, prefix operators, groups, trailing commas, and one-expression bodies parse exactly; S3a forms still parse; parameters on `impl` or with an empty body are syntax errors. | CLI and parser unit |
| `S3B-SIGN-01` | Section 4 | A `-` directly before an integer token is that literal's sign; every other prefix `-` is negation. | CLI and unit |
| `S3B-GROUP-01` | Section 5 | `*` binds before `+` and `-`; chains associate left; mixing groups or chaining shifts is one `ORC0108` per expression, at the ungrouped operator. | CLI and parser unit |
| `S3B-NAME-01` | Section 6 | Names resolve only to parameters of the enclosing function, and parameter names are unique within it. | CLI and unit |
| `S3B-CALL-01` | Section 6 | Calls resolve to typed `spec` functions in any source order; empty `spec`, `impl`, and unknown callees fail; argument counts match exactly. | CLI and unit |
| `S3B-CYCLE-01` | Sections 6 and 10 | Every call cycle, direct or indirect, is reported once at the call that closes it. | CLI and unit |
| `S3B-TYPE-01` | Section 7 | Exactly `Int` and `Word` at widths 8, 16, 32, and 64 are admitted; every other shape and width spelling fails. | CLI and unit |
| `S3B-CHECK-01` | Section 8 | Every expression is checked against its expected type, with no inference or conversion. | CLI and unit |
| `S3B-LIT-01` | Section 8 | Literals take the expected type; word literals are unsigned and within 0 through 2^n - 1 at every width. | CLI and unit |
| `S3B-OP-01` | Section 8 | Each operator is accepted exactly on the types its table row defines. | CLI and unit |
| `S3B-SHIFT-01` | Section 8 | Shift and rotation amounts are unsigned literals from 0 through n - 1. | CLI and unit |
| `S3B-WORD-01` | Section 9 | Word arithmetic, bitwise operations, shifts, and rotations compute exactly the table's residues at every width. | CLI and unit |
| `S3B-INT-01` | Section 9 | `Int` arithmetic is exact, including across calls and past 128 bits. | CLI and unit |
| `S3B-DIAG-01` | Section 10 | Diagnostic order, spans, and non-cascading behavior are exactly as specified. | CLI and unit |
| `S3B-CORE-01` | Section 11 | Core functions carry parameter types and one typed postorder body; IDs follow typed-spec source order. | Unit and CLI observation |
| `S3B-EVAL-01` | Section 12 | `eval` prints exactly the parameterless functions in ID order, evaluating calls by value, left to right. | CLI and unit |
| `S3B-EVAL-WORD-01` | Section 12 | `Word[n]` display is `0x` and exactly n/4 lowercase hexadecimal digits. | CLI and unit |
| `S3B-RES-NEST-01` | Section 14 | Nesting 64 is accepted and 65 fails with `ORC0106` for groups, calls, and prefix operators. | Generated CLI and parser unit |
| `S3B-RES-HEIGHT-01` | Section 14 | Height 256 is accepted and 257 fails with `ORC0106`. | Generated CLI and parser unit |
| `S3B-RES-LIST-01` | Section 14 | 64 parameters and 256 arguments are accepted, and one more of either fails with `ORC0106`. | Generated CLI and parser unit |
| `S3B-RES-EVENT-01` | Section 14 | Semantic events and Core nodes are counted exactly as listed. | Unit |
| `S3B-RES-DEPTH-01` | Section 14 | 256 frames are accepted and a 257th fails with `ORC0301`. | Generated CLI and unit |
| `S3B-RES-STEP-01` | Section 14 | The step table is exact; exactly 1,048,576 steps succeed and one more fails. | Generated CLI and unit |
| `S3B-RES-BITS-01` | Section 14 | A 16,384-bit `Int` result is displayed exactly and a larger one fails with `ORC0301`. | Generated CLI and unit |
| `S3B-RES-STACK-01` | Section 14 | The deepest admitted sources fit in 1 MiB of native stack. | Unit |
| `S3B-RES-FAIL-01` | Section 14 | Allocation failure in the parser, analyzer, or evaluator yields no partial tree, Core, or value. | Unit |
| `S3B-DETERMINISM-01` | Section 15 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 16. Relationship to S3a

When OEP-0005 is accepted, this document replaces these clauses:

- `LANGUAGE_2026.md` section 2.5, the token list and the sentence limiting `-`
  to a literal sign, by sections 3 and 4 above;
- `LANGUAGE_2026.md` sections 3 and 4, the grammar and syntax-tree mapping, by
  section 4, and section 6, where parameters, general expressions, calls, and
  arithmetic are no longer excluded;
- `SEMANTICS_2026.md` section 2 (grammar), by section 4;
- `SEMANTICS_2026.md` section 4, the admitted-type table and the rejection of
  every other word width, by section 7;
- `SEMANTICS_2026.md` section 7 (Core shape), by section 11;
- `SEMANTICS_2026.md` section 8, the sentence that evaluation calls no function
  and performs no arithmetic, and the `Word[8]`-only display rule, by
  section 12; and
- `SEMANTICS_2026.md` sections 9 and 11, where this document adds limits and
  removes parameters, operators, and calls from the list of exclusions.

Every source that S3a accepts is accepted by S3b with the same Core values and
the same output bytes. The difference runs the other way: `Word[16]`,
`Word[32]`, and `Word[64]`, which S3a rejected, are admitted, and `orangec
lex` now reports `<<`, `>>`, `<<<`, and `>>>` as single tokens. No source that
used those spellings was syntactically valid before. The S3a conformance
fixture that used `Word[16]` as its example of an unsupported width moves to
`Word[12]`, and the S3a messages that named only `Word[8]` now name the four
widths.

## 17. Explicit non-claims and future work

This slice defines no local bindings, tuples, booleans, comparisons,
conditionals, loops, division, remainder, conversions between types, variable
shift or rotation amounts, recursion, typed `impl` declarations, contracts,
effects, secrecy labels, failure values, imports, multiple modules, proofs,
claims, games, canonical Core, code generation, target, ABI, layout, leakage,
package, cryptographic, release, compatibility, or production behavior.

A function named like a standard's function, and evaluating to the standard's
example value, is not thereby a verified transcription of that standard. No
statement here says whether any expression would run in constant time on any
machine. Tests establish the tested behavior of one implementation at one
revision; they do not prove semantic soundness, completeness, or
implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
