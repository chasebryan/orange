# Orange 2026 typed-literal semantic specification

Status: normative accepted pre-alpha S3a semantics under D-026 and OEP-0003

Edition: `2026`

Snapshot: 2026-07-12

This document defines the complete semantic boundary of Orange 2026's first
typed reference-evaluation slice. It is a narrow delta over the accepted S2
lexical and grammar rules in [`LANGUAGE_2026.md`](LANGUAGE_2026.md). That
document remains authoritative for source representation, tokens, the legacy
empty-declaration grammar, syntax trees, parser diagnostics, and parser limits.
This document authoritatively adds only the typed-`spec` alternative and its
semantics under the later owner direction recorded by D-026 and accepted
OEP-0003.

The corresponding implementation and documentation were merged by PR #9 at
commit `6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. Acceptance closes this bounded
typed-literal slice only. Orange remains pre-alpha, later S3 semantics remain
incomplete, and D-003 and D-004 remain unresolved.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Scope and phase boundary

An Orange source reaches semantic analysis only after it has passed Orange 2026
lexing and parsing with zero diagnostics. Semantic success requires zero
semantic diagnostics and successful construction of the Typed Reference Core
defined below.

`orangec check` must run lexical, syntactic, and semantic validation. `orangec
eval` must run the same phases before evaluating the Core. A diagnostic from any
phase makes the source unsuccessful. No recovered or partial syntax tree, Core,
or evaluation output is an accepted Orange program result.

The accepted semantic surface consists only of declaration-name uniqueness,
typed specification literals, the exact `Int` and `Word[8]` types, Typed
Reference Core construction, and deterministic literal evaluation. Empty
functions remain valid syntax but gain no type, value, or execution meaning.

## 2. Additive grammar

The S2 `source_file`, `edition_decl`, and `module_decl` productions remain
unchanged. Its `function_decl` production is factored as follows so every S2
empty declaration remains accepted:

```text
function_decl   = "spec" IDENTIFIER "(" ")" spec_tail
                | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail       = empty_body
                | "->" parsed_type "{" signed_integer "}" ;
empty_body      = "{" "}" ;
parsed_type     = IDENTIFIER ("[" INTEGER "]")? ;
signed_integer  = "-"? INTEGER ;
```

The following forms are accepted syntax:

```orange
edition 2026;
module demo {
  spec placeholder() {}
  impl placeholder() {}
  spec answer() -> Int { 42 }
  spec mask() -> Word[8] { 0xff }
}
```

Only `spec` has a typed alternative. Parameters, typed `impl` declarations,
operators, calls, names as expressions, multiple literals, semicolons in bodies,
and every other nonliteral expression or statement are outside this grammar.

`parsed_type` is a syntactic container. Parsing an identifier or optional width
does not make that type semantically supported.

## 3. Declaration namespaces

Function names are compared by their exact ASCII spelling. One module has two
function namespaces distinguished by declaration kind:

```text
namespace key = (spec | impl, exact identifier spelling)
```

Every declaration, empty or typed, must have a unique key. The second and each
later declaration with an existing key is a semantic error. A `spec` and an
`impl` with the same spelling have different keys and are both permitted.

Declaration order does not change uniqueness. A name creates no callable,
importable, linkable, or externally visible symbol in this slice. The module
name is retained for Core and display identity only.

## 4. Parsed and accepted types

A parsed type contains one identifier and either no bracketed integer or exactly
one bracketed integer. Semantic acceptance recognizes exactly these contextual
forms:

| Source form | Core type | Value domain |
| --- | --- | --- |
| `Int` | `Int` | all mathematical integers |
| `Word[8]` | `Word8` | unsigned integers from 0 through 255 |

`Int` must have no width. `Word` must have the exact bracketed decimal token
spelling `8`, without a base prefix, separator, sign, or leading zero. Type
identifiers are case-sensitive.

Every other parsed form is a semantic error. This includes `Int[8]`, `Word`,
`Word[08]`, `Word[0x8]`, `Word[1_0]`, every other word width, and every other
identifier. There is no inference, alias, subtyping, overloading, coercion, or
implicit conversion.

## 5. Signed integer literals

The `INTEGER` token supplies a nonnegative magnitude under the base and digit-
separator rules in [`LANGUAGE_2026.md`](LANGUAGE_2026.md#23-integer-tokens).
An optional preceding `-` is part of `signed_integer`; it is not a general unary
operator.

Before constructing a value, the analyzer must:

1. remove the permitted base prefix and digit separators;
2. decode the remaining digits exactly in base 2, 10, or 16;
3. determine the magnitude's significant bit length after ignoring leading
   zeroes; and
4. reject a magnitude longer than 16,384 significant bits.

Zero has zero significant bits regardless of its source spelling. Leading
zeroes do not consume the significant-bit budget.

For `Int`, the decoded magnitude denotes itself when no minus sign is present
and its mathematical negation when the sign is present. Every spelling of
negative zero normalizes to zero. `Int` is an unbounded mathematical domain; the
source magnitude limit is a semantic source-representation boundary, not a
finite-width integer definition.

For `Word[8]`, a minus sign is a semantic error even when the magnitude is zero.
A magnitude greater than 255 is a semantic error. An accepted magnitude denotes
that exact unsigned value. No value wraps, truncates, saturates, or coerces.

## 6. Semantic acceptance and diagnostics

Semantic analysis proceeds deterministically in source order:

1. inspect every declaration key and report same-kind duplicates;
2. for each typed `spec`, validate its parsed type;
3. decode and validate its literal under that type; and
4. if no semantic diagnostic exists, construct Typed Reference Core functions
   in typed-spec source order.

An error in one typed declaration does not authorize a partial Core. The
analyzer may continue within its budgets to report independent later errors,
but the result remains unsuccessful.

Semantic diagnostic categories must distinguish at least duplicate declaration,
unsupported type, invalid type argument, significant-integer limit, negative
word literal, word-literal range, semantic diagnostic suppression, and semantic
resource exhaustion. Each diagnostic has a source span tied to the responsible
name, type component, sign, or literal. Diagnostic ordering, categories, and
source spans must be deterministic.

## 7. Typed Reference Core

A successful semantic result contains one Typed Reference Core module:

```text
core_module   = module_name core_function* ;
core_function = function_id function_name core_type core_value ;
core_type     = Int | Word8 ;
core_value    = normalized_mathematical_integer
              | unsigned_8_bit_integer ;
```

Only typed `spec` declarations produce `core_function` entries. Empty `spec`
and every empty `impl` declaration are absent from the Core. Core functions
retain source order. Their IDs are the contiguous integers `0` through `n - 1`
assigned over typed specifications only.

The module and function names are their exact source spellings. An `Int` value
is its normalized mathematical integer. A `Word8` value is its exact unsigned
integer in the inclusive range 0 through 255.

The Core contains no source-level type spelling variants, sign token, operators,
calls, bindings, effects, dynamic failure, proof terms, axioms, implementation
declarations, target information, or leakage information.

The Typed Reference Core is an internal typed compiler boundary. This slice
defines no canonical encoding, serialization, content digest, theorem
fingerprint, proof identity, refinement relation, erasure relation, external
schema, or cross-revision ID stability.

## 8. Reference evaluation and display

`orangec eval FILE` accepts exactly one source operand. A missing or additional
source operand is a command-usage error. `-` may name standard input under the
existing single-read and source-size rules.

After successful analysis, the evaluator visits every Core function in
ascending function-ID order. It writes exactly one line per function:

```text
module::name: Type = value\n
```

The module and function fields use their exact ASCII source spellings. `Type` is
exactly `Int` or `Word[8]`.

An `Int` value is printed in base 10 without a plus sign or leading zeroes.
Mathematical zero is `0`; a negative nonzero value has exactly one leading `-`.
A `Word[8]` value is printed as `0x` followed by exactly two lowercase
hexadecimal digits, including a leading zero when necessary.

Examples are:

```text
demo::answer: Int = 42
demo::mask: Word[8] = 0xff
```

A successful Core with no functions writes zero bytes. If lexing, parsing,
analysis, evaluation, or output fails, the command must not intentionally emit
any partial value sequence. Evaluation does not execute an `impl`, call a
function, perform arithmetic, or select a machine target.

## 9. Resource limits and failure

Semantic processing of one source is bounded by all of the following:

- 100 ordinary semantic diagnostics followed by at most one suppression
  diagnostic;
- 262,144 Typed Reference Core nodes;
- 1,048,576 semantic events;
- 16,384 significant bits in any decoded integer magnitude; and
- 1,048,576 reference-evaluation steps.

The semantic diagnostic budget counts emitted ordinary diagnostics. An attempt
to emit the next ordinary diagnostic emits the suppression diagnostic once and
suppresses later ordinary diagnostics.

A Core module, Core function, Core type, and Core value each count as one Core
node. Empty declarations do not create Core nodes.

A semantic event is one of:

- one attempted declaration-key lookup or insertion;
- one inspection of a parsed-type identifier or optional width token;
- one inspection of a literal sign, prefix, or significant source digit;
- one semantic diagnostic emission attempt; or
- one Core-node construction attempt.

One reference-evaluation step is consumed for each Core function visited for
rendering. Formatting the already bounded literal does not add steps.

Resource exhaustion emits one stable resource diagnostic outside the ordinary
diagnostic budget, prevents Core acceptance, and prevents evaluation output.
An allocation, I/O, or host failure inside the limits is not an Orange semantic
value and must not be reported as successful evaluation.

## 10. Determinism and conformance

For identical source bytes, Orange edition, compiler revision, and command, the
following must be identical across repeated runs on the same supported host:

- semantic success or failure;
- diagnostic categories, order, and source spans;
- normalized types and values;
- Core function order and IDs;
- exit status; and
- evaluation output bytes.

Conformance includes positive, negative, boundary, resource, and repeatability
cases for every normative rule. At minimum it covers:

- legacy empty and mixed empty/typed modules;
- typed-spec syntax and typed-impl rejection;
- same-kind duplicates and cross-kind equal names;
- each accepted and parsed-but-unsupported type shape;
- positive, zero, negative, and negative-zero `Int` values in every lexical
  base;
- `Word[8]` values 0, 1, 254, and 255;
- a negative word, negative word zero, and values 256 and greater;
- the exact significant-bit boundary;
- Core node, semantic event, diagnostic, and evaluation-step limits;
- exact Core source order and IDs;
- exact decimal and two-digit lowercase hexadecimal output;
- successful empty output; and
- repeated semantic and evaluation equality.

### S3a conformance rule index

The identifiers below are stable conformance labels for the existing normative
obligations; they do not add semantics or widen this snapshot. The conformance
runner requires exact agreement between this index and its named
executable-evidence map and requires every declared broad evidence layer (CLI,
generated CLI, parser unit, or unit) for each rule. It additionally requires the
corresponding injected-writer, injected-limit, or host-fault capability on tests
named for specialized evidence labels. Host-failure evidence separately requires
I/O, allocation, and non-regular host-boundary failures. Every named test must
have exactly one unconditional declaration at its expected harness location:
integration tests at file root and unit tests directly inside the source's
unique `#[cfg(test)] mod tests` container. Declarations inside comments, strings,
nested functions, or alternate or disabled modules do not qualify. That
traceability check is not proof that a named test exhausts its rule. Production
constants remain policy-bound; rules whose maxima cannot be reached by valid
public source before an earlier bound use explicitly named injected-limit tests
for exact accounting and fail-closed behavior.

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3A-PHASE-01` | Sections 1 and 6 | Clean lexing and parsing gate semantic analysis; `check` and `eval` fail on any compiler-phase diagnostic. | CLI and unit |
| `S3A-GRAMMAR-01` | Section 2 | Legacy empty declarations remain accepted, only `spec` gains the exact typed-literal alternative, and excluded body forms remain syntax errors. | CLI and parser unit |
| `S3A-DECL-01` | Section 3 | Namespace keys use declaration kind and exact ASCII name; cross-kind equals succeed and every later same-kind duplicate fails. | CLI and unit |
| `S3A-TYPE-INT-01` | Section 4 | Only exact, unparameterized `Int` lowers to Core `Int`. | CLI and unit |
| `S3A-TYPE-WORD8-01` | Section 4 | Only exact `Word[8]` with decimal width spelling `8` lowers to `Word8`. | CLI and unit |
| `S3A-TYPE-REJECT-01` | Section 4 | Every other parsed type shape or case variant fails without inference or coercion. | CLI and unit |
| `S3A-LIT-DECODE-01` | Section 5 | Admitted prefixes and separators are removed and magnitudes decode exactly in bases 2, 10, and 16. | CLI and unit |
| `S3A-LIT-ZEROES-01` | Sections 5 and 9 | Zero has zero significant bits and leading zeroes consume neither the significant-bit nor semantic-event budget. | Generated CLI and unit |
| `S3A-LIT-BITS-01` | Sections 5 and 9 | Exactly 16,384 significant bits succeed and 16,385 fail with the stable magnitude-limit category. | Generated CLI and unit |
| `S3A-INT-01` | Section 5 | `Int` denotes exact mathematical integers, including canonical zero for every negative-zero spelling. | CLI and unit |
| `S3A-WORD-SIGN-01` | Section 5 | Every minus sign on `Word[8]`, including `-0`, is an error. | CLI and unit |
| `S3A-WORD-RANGE-01` | Section 5 | Values 0 through 255 are exact and larger values fail without wrapping, truncation, saturation, or coercion. | CLI and unit |
| `S3A-DIAG-01` | Section 6 | Required categories, source order, responsible spans, and independent later diagnostics are deterministic. | CLI and unit |
| `S3A-ATOMIC-01` | Sections 1 and 6 | Any compiler or evaluator diagnostic prevents an accepted partial Core or value sequence. | CLI and unit |
| `S3A-CORE-MEMBERSHIP-01` | Section 7 | One Core module contains only typed specifications; empty `spec` and `impl` declarations are absent. | CLI and unit |
| `S3A-CORE-ORDER-01` | Sections 6 and 7 | Core functions retain typed-spec source order and receive contiguous IDs from zero. | Unit and CLI observation |
| `S3A-CORE-CONTENT-01` | Sections 3 and 7 | Core retains exact names and normalized type/value content without excluded source or target meaning. | Unit and CLI observation |
| `S3A-CLI-EVAL-01` | Section 8 | `eval` requires exactly one file or standard-input operand under inherited read limits. | CLI |
| `S3A-EVAL-LINE-01` | Section 8 | Evaluation visits ascending IDs and emits exactly one correctly framed line per Core function. | CLI and unit |
| `S3A-EVAL-INT-01` | Section 8 | `Int` display is canonical base 10 with only the required negative sign. | CLI and unit |
| `S3A-EVAL-WORD8-01` | Section 8 | `Word[8]` display is `0x` plus exactly two lowercase hexadecimal digits. | CLI and unit |
| `S3A-EVAL-EMPTY-01` | Section 8 | A successful empty Core emits zero bytes. | CLI and unit |
| `S3A-EVAL-OUTPUT-FAIL-01` | Section 8 | A detected output failure is unsuccessful and evaluation does not intentionally continue a partial value sequence. | Injected writer unit |
| `S3A-RES-DIAG-01` | Section 9 | One hundred ordinary semantic diagnostics are followed by at most one suppression diagnostic. | Generated CLI and unit |
| `S3A-RES-CORE-01` | Section 9 | The Core-node cap and module/function/type/value accounting are exact; empty declarations add no nodes. | Injected-limit unit |
| `S3A-RES-EVENT-01` | Section 9 | The semantic-event cap and every listed event category are counted exactly. | Injected-limit unit |
| `S3A-RES-EVAL-01` | Section 9 | Evaluation consumes one step per visited function and formatting consumes no extra step. | Injected-limit unit |
| `S3A-RES-FAIL-01` | Section 9 | Resource exhaustion has one stable out-of-band diagnostic and prevents Core or value acceptance. | Injected-limit unit |
| `S3A-HOST-FAIL-01` | Section 9 | Allocation, I/O, and host failures are never reported as successful Orange evaluation. | Representative fault-injection unit |
| `S3A-DETERMINISM-01` | Section 10 | Repeated identical inputs preserve success, diagnostics, normalized Core, IDs, status, and output bytes. | CLI and unit |

Tests establish only the tested implementation behavior at an identified
revision. They do not prove semantic soundness, completeness, implementation
independence, or any excluded claim.

## 11. Explicit non-claims and future work

This specification defines no parameters, operators, calls, bindings,
statements, control flow, recursion, dynamic failure values, conversions, type
inference, contracts, effects, imports, proofs, claims, games, canonical Core,
code generation, compilation correctness, target, ABI, layout, leakage,
package, cryptographic, release, compatibility, or production behavior.

The Typed Reference Core is not declared to be Spec Core, Impl Core, their
shared fragment, or a proof language. No relationship between a `spec` and an
`impl` is established, including when they have the same name.

Adding any excluded behavior requires a later directed or accepted semantic
decision, normative specification, positive and negative conformance cases,
resource analysis, and migration review.
