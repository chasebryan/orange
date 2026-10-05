# The Orange Reference Manual

<img src="images/orange-reference-manual-cover.png" width="400" alt="The Orange Reference Manual cover: orange emblem and oversized ORANGE lettering on black.">

By Chase Bryan

Status: companion lookup for the implemented Orange 2026 surface (compiler slice S3t). Not an accepted grammar, and not the teaching Book.

Snapshot: 2026-10-05

Edition: `2026`

---

This manual is a lookup companion to the Orange 2026 compiler surface.
It restates that surface for retrieval. It does not replace
[`LANGUAGE_2026.md`](LANGUAGE_2026.md), [`SEMANTICS_2026.md`](SEMANTICS_2026.md),
or the in-review slice specifications those documents name. It is not
[`THE_ORANGE_BOOK.md`](THE_ORANGE_BOOK.md). Where this manual and those
records differ, those records control.

## Contents

- [Preface](#preface)
  - [§1. Scope](#1-scope)
  - [§2. Conformance, Formality, and Normative Terminology](#2-conformance-formality-and-normative-terminology)
  - [§3. Mathematical Notations and Typographical Conventions](#3-mathematical-notations-and-typographical-conventions)
  - [§4. Language Status Register and Slice Traceability](#4-language-status-register-and-slice-traceability)

- [Part I: Lexical & Concrete Syntactic Grammar](#part-i-lexical--concrete-syntactic-grammar)
  - [§5. Byte-Level Encoding, Normalization Invariants, and Source Limits](#5-byte-level-encoding-normalization-invariants-and-source-limits)
  - [§6. Lexical Analysis, Maximal Munch, and State-Machine Tokenization](#6-lexical-analysis-maximal-munch-and-state-machine-tokenization)
  - [§7. Whitespace, Trivia Preservation, and Formatting Idempotence](#7-whitespace-trivia-preservation-and-formatting-idempotence)
  - [§8. Nested Block and Line Comments](#8-nested-block-and-line-comments)
  - [§9. Identifiers, Reserved Words, and Contextual Keywords](#9-identifiers-reserved-words-and-contextual-keywords)
  - [§10. Numeric Literals: Radices, Magnitudes, and Underscore Grammar](#10-numeric-literals-radices-magnitudes-and-underscore-grammar)
  - [§11. Byte Sequences, Escape Grammar, and Hexadecimal Strings](#11-byte-sequences-escape-grammar-and-hexadecimal-strings)
  - [§12. Punctuation, Delimiters, Operators, and Lexical Budgets](#12-punctuation-delimiters-operators-and-lexical-budgets)

- [Part II: Compilation Units and Module Architecture](#part-ii-compilation-units-and-module-architecture)
  - [§13. Compilation Units, Edition Invariants, and Header Syntax](#13-compilation-units-edition-invariants-and-header-syntax)
  - [§14. Module Enclosures, Top-Level Member Ordering, and Syntax Trees](#14-module-enclosures-top-level-member-ordering-and-syntax-trees)
  - [§15. Hermetic Module Resolution and Filesystem Path Mapping](#15-hermetic-module-resolution-and-filesystem-path-mapping)
  - [§16. Import dependency graphs and cycle rejection](#16-import-dependency-graphs-and-cycle-rejection)
  - [§17. Type Aliases, Structural Equivalence, and Shadowing Restrictions](#17-type-aliases-structural-equivalence-and-shadowing-restrictions)
  - [§18. Multi-Strata Architectural Separation and Disjoint Namespaces](#18-multi-strata-architectural-separation-and-disjoint-namespaces)

- [Part III: Formal Type System & Algebraic Foundations](#part-iii-formal-type-system--algebraic-foundations)
  - [§19. Semantic Value Domains and Mathematical Universes](#19-semantic-value-domains-and-mathematical-universes)
  - [§20. Mathematical Integers ($\mathbb{Z}$) and the `Int` Type](#20-mathematical-integers-mathbbz-and-the-int-type)
  - [§21. Word Rings: Algebraic Foundations of $\mathbb{Z}/2^W\mathbb{Z}$ ($W \in \{8, 16, 32, 64\}$)](#21-word-rings-algebraic-foundations-of-mathbbz2wmathbbz-w-in-8-16-32-64)
  - [§22. Residue Fields and Modular Arithmetic ($\mathbb{Z}/m\mathbb{Z}$ for $2 \le m \le 2^{521}-1$)](#22-residue-fields-and-modular-arithmetic-mathbbzmmathbbz-for-2-le-m-le-2521-1)
  - [§23. Boolean Truth Values and Propositional Logic (`Bool`)](#23-boolean-truth-values-and-propositional-logic-bool)
  - [§24. Fixed-Length Array Spaces ($T^n$ for $1 \le n \le 65,536$)](#24-fixed-length-array-spaces-tn-for-1-le-n-le-65536)
  - [§25. Rank-two arrays](#25-rank-two-arrays)
  - [§26. Heterogeneous Product Types: Tuples ($(T_0, \dots, T_{k-1})$ for $2 \le k \le 16$)](#26-heterogeneous-product-types-tuples-t_0-dots-t_k-1-for-2-le-k-le-16)
  - [§27. Byte sequences (`Word[8]^n`)](#27-byte-sequences-word8n)
  - [§28. Explicit Value Conversions (`expr as T`) and Typing Judgments](#28-explicit-value-conversions-expr-as-t-and-typing-judgments)
  - [§29. Endianness Homomorphisms and Bit-Preserving Packing (`as big T`, `as little T`)](#29-endianness-homomorphisms-and-bit-preserving-packing-as-big-t-as-little-t)
  - [§30. Dependent Finite Size Parameters and Monomorphization](#30-dependent-finite-size-parameters-and-monomorphization)
  - [§31. Finite Type Parameter Domains ($[K \in \{T_1, \dots, T_m\}]$)](#31-finite-type-parameter-domains-k-in-t_1-dots-t_m)

- [Part IV: Static Semantics (Typing Rules & Judgments)](#part-iv-static-semantics-typing-rules--judgments)
  - [§32. Typing Contexts: Signature ($\Sigma$), Size ($\Theta$), Type ($\Delta$), and Variable ($\Gamma$) Environments](#32-typing-contexts-signature-sigma-size-theta-type-delta-and-variable-gamma-environments)
  - [§33. Subtyping, Coercion Freedom, and Structural Type Equality](#33-subtyping-coercion-freedom-and-structural-type-equality)
  - [§34. Expression Typing Rules and Formal Judgments](#34-expression-typing-rules-and-formal-judgments)
  - [§35. Expression grouping](#35-expression-grouping)
  - [§36. What an index may depend on](#36-what-an-index-may-depend-on)
  - [§37. Rank-two arrays and slices](#37-rank-two-arrays-and-slices)

- [Part V: Dynamic Semantics (Operational & Reduction Rules)](#part-v-dynamic-semantics-operational--reduction-rules)
  - [§38. Abstract Syntax and Typed Reference Core Lowering](#38-abstract-syntax-and-typed-reference-core-lowering)
  - [§39. Evaluation Environments, Value Stores, and Step Budgets](#39-evaluation-environments-value-stores-and-step-budgets)
  - [§40. Small-Step Operational Semantics (SOS) and Big-Step Reduction](#40-small-step-operational-semantics-sos-and-big-step-reduction)
  - [§41. Operational Reduction of Variable Shifts, Rotations, and Inversions](#41-operational-reduction-of-variable-shifts-rotations-and-inversions)
  - [§42. Bounded Iteration Semantics and Step-Cost Accounting](#42-bounded-iteration-semantics-and-step-cost-accounting)
  - [§43. Known-Answer Specification Tests (`test`) and Whole-Aggregate Equality](#43-known-answer-specification-tests-test-and-whole-aggregate-equality)

- [Part VI: Properties that are not theorems](#part-vi-properties-that-are-not-theorems)
  - [§44. Evaluator limits and the absence of a proof](#44-evaluator-limits-and-the-absence-of-a-proof)

- [Part VII: Partial standard transcriptions](#part-vii-partial-standard-transcriptions)
  - [§48. How a transcription is scoped](#48-how-a-transcription-is-scoped)
  - [§49. Partial transcription: FIPS 180-4 SHA-256](#49-partial-transcription-fips-180-4-sha-256)
  - [§50. Partial transcription: RFC 8439 ChaCha20 quarter round](#50-partial-transcription-rfc-8439-chacha20-quarter-round)
  - [§51. Partial transcription: RFC 7748 Montgomery ladder step](#51-partial-transcription-rfc-7748-montgomery-ladder-step)
  - [§52. Partial transcription: RFC 8439 Poly1305 field operations](#52-partial-transcription-rfc-8439-poly1305-field-operations)

- [Part VIII: Strata that are not implemented language](#part-viii-strata-that-are-not-implemented-language)
  - [§53. Empty `impl`, reserved words, and open decisions](#53-empty-impl-reserved-words-and-open-decisions)

- [Part IX: Diagnostic code catalog](#part-ix-diagnostic-code-catalog)
  - [§93. Diagnostic Philosophy, Severity Structure, and Error Budgets](#93-diagnostic-philosophy-severity-structure-and-error-budgets)
  - [§94. Lexical Diagnostics (`ORC0001`–`ORC0009`): Formal Predicates, Triggers, Examples, Fixes](#94-lexical-diagnostics-orc0001orc0009-formal-predicates-triggers-examples-fixes)
  - [§95. Syntactic Diagnostics (`ORC0101`–`ORC0108`): Formal Predicates, Triggers, Examples, Fixes](#95-syntactic-diagnostics-orc0101orc0108-formal-predicates-triggers-examples-fixes)
  - [§96. Semantic & Type Diagnostics (`ORC0201`–`ORC0242`): Formal Predicates, Triggers, Examples, Fixes](#96-semantic--type-diagnostics-orc0201orc0242-formal-predicates-triggers-examples-fixes)
  - [§97. Formatter Diagnostics (`ORC0250`–`ORC0252`): Formal Predicates, Triggers, Examples, Fixes](#97-formatter-diagnostics-orc0250orc0252-formal-predicates-triggers-examples-fixes)
  - [§98. Documentation Diagnostics (`ORC0260`–`ORC0261`): Formal Predicates, Triggers, Examples, Fixes](#98-documentation-diagnostics-orc0260orc0261-formal-predicates-triggers-examples-fixes)
  - [§99. Witness Replay Diagnostics (`ORC0270`–`ORC0274`): Formal Predicates, Triggers, Examples, Fixes](#99-witness-replay-diagnostics-orc0270orc0274-formal-predicates-triggers-examples-fixes)
  - [§100. Evaluator & Resource Diagnostics (`ORC0301`): Formal Predicates, Triggers, Examples, Fixes](#100-evaluator--resource-diagnostics-orc0301-formal-predicates-triggers-examples-fixes)

- [Part X: Toolchain, limits, and the implemented grammar](#part-x-toolchain-limits-and-the-implemented-grammar)
  - [§101. The driver `orangec`](#101-the-driver-orangec)
  - [§102. Resource limits](#102-resource-limits)
  - [§103. Implemented parser surface, restated](#103-implemented-parser-surface-restated)

---

## Preface

### §1. Scope

Subject: this file, as a companion to the records named below.
Assumption: the reader wants the implemented surface or the status of a stratum, not a teaching argument.
Evidence: the documents and compiler constants cited in each section. This file adds no acceptance, proof, or cryptographic claim.

- Accepted normative syntax is [`LANGUAGE_2026.md`](LANGUAGE_2026.md) (D-025, OEP-0002), additively extended by the accepted S3a grammar in [`SEMANTICS_2026.md`](SEMANTICS_2026.md) (D-026, OEP-0003).
- The compiler also implements slices S3b through S3t. Their specifications are in the owner's review and are not accepted. [`LANGUAGE_2026.md`](LANGUAGE_2026.md) names each one. [`THE_ORANGE_BOOK.md`](THE_ORANGE_BOOK.md) Appendix A is a convenience summary of that implemented surface; where the summary and a slice specification differ, the slice specification controls.
- The teaching manuscript is the Book. This manual does not restate it.

### §2. Conformance, Formality, and Normative Terminology

The words **must**, **must not**, and **may** in a section that restates an accepted document or the implemented compiler mean the same as in that source. They do not promote an in-review slice to an accepted specification, and they do not apply to §53.

Three conformance subjects are distinct:

| Subject | What conformance means | What it does not mean |
| --- | --- | --- |
| Accepted source | The source is in the LANGUAGE_2026 / SEMANTICS_2026 grammar and satisfies the accepted semantic rules | Later S3 behavior |
| Implemented pre-alpha compiler | `orangec` 0.0.1 on a named revision accepts the source under the in-review S3b–S3t rules and prints the diagnostics and values those rules describe | An accepted language, a proof, or a cryptographic validation |
| Unimplemented stratum | No compiler obligation | A future syntax, ABI, or proof kernel |

An **accepted slice** is one ratified under the project governance model at an exact revision. S3b–S3t are not in that class. An empty `impl` declaration is accepted syntax and has no semantic meaning.

### §3. Mathematical Notations and Typographical Conventions

Throughout this manual, formal syntax and semantics are presented using standard
programming language theory conventions:

- $\mathbb{Z}$: The set of mathematical integers $\{\dots, -2, -1, 0, 1, 2, \dots\}$.
- $\mathbb{N}$: The set of non-negative integers $\{0, 1, 2, \dots\}$.
- $\mathbb{Z} / 2^W \mathbb{Z}$: The quotient ring of integers modulo $2^W$.
- $\mathbb{Z} / m \mathbb{Z}$: The ring (or finite field when $m$ is prime) of
  integers modulo $m \ge 2$.
- $\mathbb{B} = \{\text{true}, \text{false}\}$: The two-element Boolean algebra.
- $\mathbb{V}$: The universe of all closed semantic values in the language.
- $\sigma \in \text{Store}$: The structured memory store mapping physical places
  to values.
- $\rho \in \text{Env} = \text{Ident} \to \mathbb{V}$: The local evaluation environment
  mapping variable identifiers to values.
- $\Gamma \in \text{TyEnv} = \text{Ident} \to \tau$: The static typing environment
  mapping variables to types.
- $\Sigma$: The global module and signature environment.
- $\Theta$: The finite size parameter environment.
- $\Delta$: The finite type parameter environment.
- $\Gamma \vdash e : \tau$: Static typing judgment asserting that expression $e$
  possesses type $\tau$ under environment $\Gamma$.
- $\langle e, \rho \rangle \Downarrow v$: Big-step operational reduction judgment
  asserting that expression $e$ evaluates to value $v$ under environment $\rho$.
- $\langle S, \sigma \rangle \longrightarrow \langle S', \sigma' \rangle$: Small-step
  imperative transition judgment.
- $\text{wp}(S, Q)$: Weakest precondition predicate transformer.
- $a \mathbin{\Vert} b$: Bitwise concatenation of bit sequences.
- $H(x)$: Cryptographic hash function (SHA-256 unless otherwise qualified).

### §4. Language Status Register and Slice Traceability

The development of Orange follows a gated, incremental slice roadmap. The table
below records the formal status of every feature slice and architectural stratum:

| Slice / subject | Acceptance | Compiler (`orangec` 0.0.1, slice S3t) | Controlling record |
| --- | --- | --- | --- |
| S2 lexical and empty-declaration syntax | Accepted | Accepted | D-025, OEP-0002, `LANGUAGE_2026.md` |
| S3a typed `spec` literals (`Int`, `Word[8]`) | Accepted | Accepted | D-026, OEP-0003, `SEMANTICS_2026.md` |
| S3b–S3t (expressions through static moduli) | In review; not accepted | Implemented | OEP-0005–OEP-0021, OEP-0023, OEP-0024 and the slice documents named in `LANGUAGE_2026.md` |
| Empty `impl` declaration | Syntax accepted with S2 | No type, value, or execution | `SEMANTICS_2026.md` §1 |
| `game`, `proof`, `claim` | Reserved words only | No production and no semantics | `LANGUAGE_2026.md` §2.2 |
| Imperative `impl` bodies, machine code, games, proofs, claims, FFI, leakage policies | Not selected | Not implemented | D-004 proposed; D-005 proposed; D-006 investigate; D-007 proposed and dependent on D-006; D-011 proposed; D-012 investigate. See §53. |

`spec` and `impl` use separate declaration namespaces: the same spelling may occur once in each, and a second declaration of the same kind is `ORC0201`. A call names a typed `spec`. An empty `impl` is not callable.


---

## Part I: Lexical & Concrete Syntactic Grammar

### §5. Byte-Level Encoding, Normalization Invariants, and Source Limits

1. **Source Encoding:** An Orange source compilation unit MUST consist of a contiguous
   sequence of octets representing valid UTF-8 according to RFC 3629 / Unicode Standard.
2. **Unexpected characters, including U+FEFF:**
   U+FEFF is not whitespace and not a token character. The lexer reports it as
   `ORC0001` (`unexpected character U+FEFF`) at whatever offset it occurs outside
   a string. There is no separate BOM production.
3. **File size:**
   A source is at most 16 MiB (`16 * 1024 * 1024` bytes). The driver rejects a
   larger file before lexing with `ORC1001`. `ORC0008` is not that diagnostic:
   `ORC0008` means the lexer could not retain its bounded token-stream
   representation.
4. **Line endings:**
   Line feed, carriage-return line feed, and a bare carriage return each form
   one logical line ending. A carriage-return line-feed pair is one line, not two.
   Columns are one-based Unicode scalar positions. Spans are byte ranges.
5. **Characters outside the alphabet:**
   The only whitespace code points are U+0009, U+000A, U+000D, and U+0020.
   Any other code point that is not part of a token, comment, or string is
   `ORC0001`. That includes other C0 controls, C1 controls, DEL, U+061C,
   U+200E, U+200F, U+2028, U+2029, U+202A–U+202E, and U+2066–U+2069. The lexer
   does not run a separate Trojan Source pass; those characters fail because
   they are not in the alphabet.
6. **Absence of Canonical Normalization:**
   The lexer does not apply Unicode Normalization Forms (NFC, NFD, NFKC, NFKD).
   Source text is compared strictly as raw byte sequences. Identifiers are
   restricted to ASCII, rendering multi-byte Unicode comparison outside strings
   lexically impossible.

### §6. Lexical Analysis, Maximal Munch, and State-Machine Tokenization

1. **Maximal Munch Rule:**
   The lexical analyzer operates as a deterministic finite automaton (DFA) consuming
   input characters greedily. At any scan point, the longest sequence of characters
   forming a valid lexical token is selected.
   - Example: `<<<` is scanned as the single rotation token `<<<`, never as `<` followed
     by `<<` or three `<` tokens.
   - Example: `::` is scanned as `DOUBLE_COLON`, never as two `COLON` tokens.
   - Example: `++` is scanned as `PLUS_PLUS`, never as two `PLUS` tokens.
2. **Deterministic Token Spans:**
   Every scanned token carries an exact half-open byte span:
   $$\text{Span} = [\text{byte\_start}, \text{byte\_end})$$
   alongside 1-indexed line and column numbers. Diagnostics refer strictly to
   these byte spans.

### §7. Whitespace, Trivia Preservation, and Formatting Idempotence

1. **Whitespace Definition:**
   Whitespace consists of:
   - Space (`0x20`, U+0020)
   - Horizontal Tab (`0x09`, U+0009)
   - Carriage Return (`0x0D`, U+000D), including a bare carriage return
   - Line Feed (`0x0A`, U+000A)
2. Whitespace serves solely to delimit tokens and has no syntactic significance
   except inside string literals.
3. **The Formatter Contract:**
   The compiler driver incorporates a canonical syntax formatter (`orangec fmt`).
   The formatter is governed by the following mathematical laws:
   - **Idempotence:** $\text{fmt}(\text{fmt}(\text{src})) = \text{fmt}(\text{src})$
   - **Token Identity Invariant:** Stripping trivia from $\text{fmt}(\text{src})$
     yields a token sequence identical in spellings and values to the unformatted input.
   - **Comment Preservation:** All line comments and block comments are preserved
     with byte-identical content and exact inter-token relative order.
   - **Check Mode:** Running `orangec fmt --check FILE` verifies byte-level equality.
     Any discrepancy emits diagnostic `ORC0252` and exits with code 1.
   - Formatting resource exhaustion emits `ORC0250`; internal comment preservation
     inconsistency emits `ORC0251`.

### §8. Nested Block and Line Comments

1. **Line Comments:**
   Begin with ASCII `//` and extend to the next logical line ending or end of input. The line ending is not part of the comment.
   Line comments have no closing delimiter and cannot nest.
2. **Block Comments:**
   Begin with ASCII `/*` and terminate with `*/`.
   - **Arbitrary Nesting Invariant:**
     The lexer maintains an integer nesting depth counter $d \in \mathbb{N}$,
     initialized to $0$.
     - Upon encountering `/*`, $d \leftarrow d + 1$.
     - Upon encountering `*/`, $d \leftarrow d - 1$.
     - The comment terminates when $d = 0$.
   - **Unterminated Comments:**
     If EOF is reached while $d > 0$, lexing aborts with diagnostic `ORC0002`
     pointing to the primary span of the opening `/*` that lacked a closing delimiter.

### §9. Identifiers, Reserved Words, and Contextual Keywords

1. **Identifier Syntax:**
   Identifiers are ASCII-only and conform to the regular grammar:
   $$\text{identifier} = [A\text{-}Za\text{-}z\_][A\text{-}Za\text{-}z0\text{-}9\_]^*$$
   Non-ASCII Unicode characters in identifiers emit diagnostic `ORC0001`.
2. **Permanently Reserved Words:**
   The following 7 spellings are permanently reserved across all syntactic positions:
   ```text
   edition    module    spec    impl    game    proof    claim
   ```
   A reserved word cannot be used as an identifier for a function, variable,
   parameter, type, or module. Any attempt to do so emits diagnostic `ORC0101`.
3. **Position words, not reserved words:**
   `let`, `as`, `for`, `in`, `with`, `if`, `else`, `use`, `type`, `hex`, `big`,
   `little`, `true`, and `false` are identifiers. Each has a grammatical role
   only in the position recorded in [`THE_ORANGE_BOOK.md`](THE_ORANGE_BOOK.md)
   Appendix A. `requires`, `ensures`, `invariant`, `variant`, `erase`, and
   `declassify` are not position words in the implemented surface.

### §10. Numeric Literals: Radices, Magnitudes, and Underscore Grammar

An integer literal denotes an exact non-negative integer:

```text
integer_literal = decimal_literal | hex_literal | binary_literal ;
decimal_literal = digit (digit | "_")* ;
hex_literal     = ("0x" | "0X") hex_digit (hex_digit | "_")* ;
binary_literal  = ("0b" | "0B") binary_digit (binary_digit | "_")* ;
digit           = "0".."9" ;
hex_digit       = digit | "a".."f" | "A".."F" ;
binary_digit    = "0" | "1" ;
```

1. **Radix Selection:**
   - Prefix `0x` or `0X` selects hexadecimal (base 16).
   - Prefix `0b` or `0B` selects binary (base 2).
   - Lack of prefix selects decimal (base 10).
2. **At Least One Digit:** An integer literal MUST contain at least one valid digit
   following the radix prefix. An isolated `0x` or `0b` emits diagnostic `ORC0005`.
3. **Digit Separator Rules:**
   - Underscores (`_`) are visual separators.
   - An underscore MUST be placed strictly between two valid digits of the chosen base.
   - Leading underscores (`_100` is an identifier, not an integer).
   - Underscores immediately following a prefix (`0x_ff`, `0b_10`) emit `ORC0005`.
   - Trailing underscores (`100_`, `0xff_`) emit `ORC0005`.
   - Consecutive underscores (`10__00`) emit `ORC0005`.
4. **Digit Set Conformance:** Any character consumed after a prefix that is not a
   valid digit of that base (e.g. `0b102` or `0x1g`) causes the token to fail
   with diagnostic `ORC0005`.
5. **Magnitude budget:**
   An exact integer uses at most 16,384 significant bits (`MAX_INTEGER_BITS`
   in `compiler/crates/orange-compiler/src/semantics.rs`). A magnitude past that
   budget is `ORC0205`. Shifts and rotations are not defined on `Int`; `1 << 4096`
   is `ORC0215`, not `ORC0205`.

### §11. Byte Sequences, Escape Grammar, and Hexadecimal Strings

Orange provides two concrete representations for immutable byte sequences (`Word[8]^n`):

#### 1. ASCII String Literals (`"..."`)
```text
string_literal = "\"" string_char* "\"" ;
string_char    = (ascii_printable - ("\"" | "\\")) | escape_seq ;
escape_seq     = "\\\"" | "\\\\" | "\\n" | "\\r" | "\\t" | "\\0" | "\\x" hex_digit hex_digit ;
```
- Delimited by double quotes (`0x22`).
- MUST reside on a single logical line. Unclosed strings before `\n` emit `ORC0003`.
- Internal characters MUST be printable ASCII (`0x20`–`0x7E`) or a valid escape.
  Non-printable ASCII or multi-byte UTF-8 codepoints emit `ORC0235` or `ORC0001`.
- **Admitted Escapes:**
  - `\"` (0x22), `\\` (0x5C), `\n` (0x0A), `\r` (0x0D), `\t` (0x09), `\0` (0x00).
  - `\xNN`: An exact byte value specified by two hexadecimal digits `NN` (`0x00`–`0xFF`).
- Any other escape (e.g. `\a`, `\e`, `\u`, `\U`) emits diagnostic `ORC0004`.

#### 2. Hexadecimal String Literals (`hex"..."`)
```text
hex_string_literal = "hex\"" (hex_digit hex_digit | " ")* "\"" ;
```
- Introduced by the token `hex` immediately followed by `"`.
- Consists of pairs of hexadecimal digits representing raw bytes, separated by
  optional spaces (`0x20`).
- An odd number of hex digits, non-hex characters (other than space), or unclosed
  quotes emit diagnostic `ORC0009`.

### §12. Punctuation, Delimiters, Operators, and Lexical Budgets

1. **Closed Operator & Delimiter Inventory:**
   ```text
   Delimiters:
     (   )   {   }   [   ]   ,   :   ;   .   ..   ::

   Arithmetic Operators:
     +   -   *   /   %

   Bitwise Operators:
     &   |   ^   ~

   Shift and Rotation Operators:
     <<   >>   <<<   >>>

   Relational Operators:
     ==   !=   <   <=   >   >=

   Logical Operators:
     !   &&   ||

   Structural Combinators:
     =   with   ++   ->   =>   ?
   ```
2. **Lexical resource limits** (`compiler/crates/orange-compiler/src/lexer.rs`):
   - At most 262,144 non-trivia tokens per source, excluding one zero-width `EOF`.
     Past that budget the diagnostic is `ORC0006`.
   - At most 100 ordinary lexical diagnostics, then one suppression diagnostic
     `ORC0007`.
   - Failure to retain the bounded token stream is `ORC0008`.

---

## Part II: Compilation Units and Module Architecture

### §13. Compilation Units, Edition Invariants, and Header Syntax

1. **Compilation unit:**
   A compilation unit is the UTF-8 source the driver is given. The `.or` suffix
   is the name `use m;` resolves (`m.or`). It is not a requirement that the root
   path end in `.or`.
2. **Mandatory Edition Header:**
   The first non-trivia grammatical production in every Orange source file MUST
   be an edition declaration:
   ```orange
   edition 2026;
   ```
3. **Edition Validation Predicates:**
   - Let $E$ be the edition token. If $E \ne 2026$, the compiler emits diagnostic `ORC0102`.
   - Omitting the terminating semicolon emits `ORC0101`.
   - Repeating the edition declaration or placing it after any other statement
     emits `ORC0104`.
4. The edition declaration binds the source text permanently to the Orange 2026
   grammar and semantics.

### §14. Module Enclosures, Top-Level Member Ordering, and Syntax Trees

1. **Single Module Invariant:**
   Every Orange source file MUST declare exactly one module immediately following
   the edition header:
   ```orange
   edition 2026;

   module module_name {
       // Declarations in strict order:
       // 1. Imports: use ...;
       // 2. Type Aliases: type ... = ...;
       // 3. Members: spec, impl, test
   }
   ```
2. Any tokens appearing after the closing brace `}` of the module declaration
   emit diagnostic `ORC0104`.
3. **Member order:**
   - `use` declarations come first.
   - `type` declarations come next. A `use` after a `type` is `ORC0103`.
   - Then `spec`, `impl`, and `test`, in any order among themselves.
   - `game`, `proof`, and `claim` are not member productions. A `use` or `type`
     after a function is `ORC0103`.

### §15. Hermetic Module Resolution and Filesystem Path Mapping

1. **Hermeticity:** Orange modules are resolved hermetically without ambient
   network queries or implicit system paths.
2. **File mapping** (`orangec`, `load_used_modules`):
   - `use m;` reads `m.or` from the directory of the root file, or from the
     current directory when the root is standard input.
   - The search is not the importing file's directory, and there is no
     `Orange.toml` package root.
   - A module is read once. At most 64 modules are in a program, root included.
   - A file whose declared module name differs from `m` is still read; its
     `use` declarations are not followed.
   - A module that cannot be read is `ORC0228`.
3. **Qualified Identifiers:**
   - Imported definitions are referenced using the module prefix: `m::symbol`.
   - Calling `m::symbol` when `use m;` was not declared in the importing module
     emits diagnostic `ORC0229`.
   - Unqualified imports (wildcards like `use m::*;`) do not exist in Orange.

### §16. Import dependency graphs and cycle rejection

1. **Dependency Graph Invariant:**
   Let $G = (V, E)$ be the module dependency graph where $V$ is the set of all
   modules and $(u, v) \in E \iff u \text{ contains } \texttt{use } v;$.
   $G$ MUST be a Directed Acyclic Graph (DAG).
2. **Cycle rejection:**
   The linker depth-first searches from the root (`semantics/linking.rs`).
   A `use` that returns to a module already on the path is `ORC0230`, labeled
   at that `use`. Modules the root does not reach are not entered. The
   algorithm is not Tarjan's SCC algorithm.
3. **Duplicate Imports:**
   - Importing the same module twice in one source file emits `ORC0231`.
   - Defining two distinct files with identical module identifiers in the same
     program emits `ORC0231`.

### §17. Type Aliases, Structural Equivalence, and Shadowing Restrictions

1. A module may introduce nominal names for types using `type`:
   ```orange
   type State = Word[32]^8;
   type Fe = Mod[(1 << 255) - 19];
   ```
2. **Structural Equivalence:**
   Type aliases are transparent: during semantic analysis, every occurrence of
   an alias is expanded to its underlying type. Type equivalence in Orange is
   strictly structural:
   $$\text{type } A = T \implies A \equiv T$$
3. **Anti-Shadowing Invariants:**
   - A type alias identifier MUST NOT shadow built-in types (`Int`, `Bool`, `Word`, and `Mod`). Defining `type Int = Word[64];` emits `ORC0233`.
   - Declaring duplicate type alias names within the same module emits `ORC0233`.

### §18. Multi-Strata Architectural Separation and Disjoint Namespaces

1. Orange enforces an ontological separation between distinct modes of cryptographic
   computation:
   - **`spec`**: Pure mathematical functions. Computes over algebraic rings and fields.
     Total, acyclic, and terminating.
   - **`impl`**: Imperative executable procedures. Computes over physical memory places
     and mutable buffers. Affine ownership, contracts, and zeroization.
   - **`machine impl`**: Low-level machine code with register constraints, SIMD types,
     and hardware crypto intrinsics.
   - **`game`**: Probabilistic programs, uniform sampling, stateful oracles, and
     cryptographic reductions.
   - **`proof`**: Proof IR terms establishing refinement, equivalence, and safety.
2. **What the namespaces do, and do not, connect:**
   `spec` and `impl` are separate declaration namespaces, so `spec sha256` and
   `impl sha256` may both occur. A call still names a typed `spec`; there is
   no stratum selector and no `claim` production connecting them. `machine impl`
   is not a grammatical form. The Core carries no proof identity between a
   `spec` and an `impl`.

---

## Part III: Formal Type System & Algebraic Foundations

### §19. Semantic Value Domains and Mathematical Universes

We define the universe of semantic values $\mathbb{V}$ as the disjoint union of
concrete value domains:

$$\mathbb{V} = \mathbb{V}_{\text{Int}} \uplus \mathbb{V}_{\text{Word}} \uplus \mathbb{V}_{\text{Mod}} \uplus \mathbb{V}_{\text{Bool}} \uplus \mathbb{V}_{\text{Array}} \uplus \mathbb{V}_{\text{Tuple}}$$

Where:
- $\mathbb{V}_{\text{Int}}$: mathematical integers whose magnitudes use at most 16,384 significant bits
- $\mathbb{V}_{\text{Word}} = \biguplus_{W \in \{8, 16, 32, 64\}} (\mathbb{Z} / 2^W \mathbb{Z})$
- $\mathbb{V}_{\text{Mod}} = \biguplus_{m \in [2, 2^{521}-1]} (\mathbb{Z} / m \mathbb{Z})$
- $\mathbb{V}_{\text{Bool}} = \{\text{true}, \text{false}\}$
- $\mathbb{V}_{\text{Array}} = \biguplus_{\tau, n} \mathbb{V}_\tau^n$ ($1 \le n \le 65,536$)
- $\mathbb{V}_{\text{Tuple}} = \biguplus_{k \in [2, 16]} (\mathbb{V}_{\tau_0} \times \dots \times \mathbb{V}_{\tau_{k-1}})$

### §20. Mathematical Integers ($\mathbb{Z}$) and the `Int` Type

1. The `Int` type models the algebraic ring of mathematical integers:
   $$(\mathbb{Z}, +, \cdot, -, 0, 1)$$
2. **Exact Arithmetic:** Operations on `Int` never overflow or wrap.
3. **Bit budget:** An `Int` value uses at most 16,384 significant bits.
   Past that budget the diagnostic is `ORC0205`.
4. **Absence of Coercion:** An `Int` CANNOT be passed to a function expecting `Word[W]`
   or `Mod[m]` without an explicit `as` cast.

### §21. Word Rings: Algebraic Foundations of $\mathbb{Z}/2^W\mathbb{Z}$ ($W \in \{8, 16, 32, 64\}$)

1. The type `Word[W]` represents the quotient ring:
   $$R_W = \mathbb{Z} / 2^W \mathbb{Z}$$
2. **Width Restriction:** $W \in \{8, 16, 32, 64\}$. Any other width emits `ORC0204`.
3. **Ring Axiom Invariance:**
   For all $a, b, c \in R_W$:
   - Associativity: $(a + b) + c = a + (b + c)$ and $(a \cdot b) \cdot c = a \cdot (b \cdot c)$
   - Commutativity: $a + b = b + a$ and $a \cdot b = b \cdot a$
   - Identity: $a + 0 = a$ and $a \cdot 1 = a$
   - Additive Inverse: $a + (2^W - a) \equiv 0 \pmod{2^W}$
   - Distributivity: $a \cdot (b + c) = (a \cdot b) + (a \cdot c)$
4. **Wrapping as Primary Semantics:**
   In Orange word arithmetic, $(2^W - 1) + 1 = 0$ is not an overflow error; it is
   the rigorous algebraic definition of the ring. Fixed-width words cannot produce
   arithmetic faults.
5. **Literal Enclosure:**
   A literal $v$ checked as `Word[W]` MUST satisfy $0 \le v \le 2^W - 1$.
   $v < 0$ emits `ORC0206`; $v \ge 2^W$ emits `ORC0207`.

### §22. Residue Fields and Modular Arithmetic ($\mathbb{Z}/m\mathbb{Z}$ for $2 \le m \le 2^{521}-1$)

1. The type `Mod[m]` represents the quotient structure:
   $$R_m = \mathbb{Z} / m \mathbb{Z}$$
   When $m$ is prime, $R_m \cong \mathbb{F}_m$ (the Galois field of order $m$).
2. **Modulus Range:** $2 \le m \le 2^{521} - 1$. Moduli outside this range emit `ORC0232`.
3. **Canonical Residues:** Every element of `Mod[m]` is uniquely represented by its
   canonical integer residue $r \in [0, m - 1]$.
4. **Inversion and Division:**
   - Division $a / b$ computes $a \cdot b^{-1} \pmod m$.
   - $b^{-1}$ is computed via the Extended Euclidean Algorithm:
     $$\gcd(b, m) = 1 \implies b \cdot u + m \cdot v = 1 \implies b^{-1} \equiv u \pmod m$$
   - **Deterministic Total Behavior:** If $\gcd(b, m) \ne 1$ (including $b \equiv 0$),
     $a / b$ evaluates deterministically to $0$.

### §23. Boolean Truth Values and Propositional Logic (`Bool`)

1. The `Bool` type forms a Boolean algebra $(\mathbb{B}, \land, \lor, \neg, \text{false}, \text{true})$.
2. Relational operators produce `Bool`:
   Equality `==` and `!=` is defined for `Int`, `Word[W]`, and `Mod[m]`, and for arrays and tuples of one type. Order `<`, `<=`, `>`, and `>=` is defined for `Int` and for unsigned `Word[W]`. It is not defined for `Mod[m]` (`ORC0215`: residues have no order).
3. On `Bool`, `&&` and `||` are conjunction and disjunction and evaluate
   every operand. They are not defined on other types. There is no `impl`
   short-circuit form.

### §24. Fixed-Length Array Spaces ($T^n$ for $1 \le n \le 65,536$)

1. An array type $T^n$ represents the $n$-ary Cartesian power:
   $$T^n = \underbrace{T \times T \times \dots \times T}_{n \text{ times}}$$
2. **Length Domain:** $1 \le n \le 65,536$. Lengths $< 1$ or $> 65,536$ emit `ORC0221`.
3. **Array Literal Typing:**
   $$\frac{\forall i \in [0, n-1].\ \Gamma \vdash e_i : T}{\Gamma \vdash [e_0, e_1, \dots, e_{n-1}] : T^n}$$
   If the literal element count $k \ne n$, the compiler emits diagnostic `ORC0222`.
4. **Fill Literal Typing:**
   $$\frac{\Gamma \vdash v : T \quad 1 \le n \le 65,536}{\Gamma \vdash [v; n] : T^n}$$

### §25. Rank-two arrays

1. The implemented rank-two form is an alias of an array of rows, not a
   repeated caret and not a matrix type former. See §37.
2. The scalar product of the two lengths is at most 65,536 (`ORC0221`).
3. `m[i][j]` is two index suffixes. It is not pointer dereference.

### §26. Heterogeneous Product Types: Tuples ($(T_0, \dots, T_{k-1})$ for $2 \le k \le 16$)

1. A tuple type $(T_0, \dots, T_{k-1})$ represents the heterogeneous product:
   $$\prod_{i=0}^{k-1} T_i = T_0 \times T_1 \times \dots \times T_{k-1}$$
2. **Arity:** $2 \le k \le 16$. A tuple's elements are scalars or arrays, never tuples.
3. **Projection Typing:**
   $$\frac{\Gamma \vdash t : (T_0, \dots, T_{k-1}) \quad 0 \le j < k}{\Gamma \vdash t.j : T_j}$$
   Attempting field projection on a non-tuple emits `ORC0234`.

### §27. Byte sequences (`Word[8]^n`)

1. There is no `Byte` type. `spec b() -> Byte { 1 }` is `ORC0203`. The
   built-in types are `Int`, `Bool`, `Word[n]`, and `Mod[m]`. A module may
   declare `type Byte = Word[8]`; that alias is not built in.
2. A string `"..."` or `hex"..."` has type `Word[8]^n`, where $n$ is the byte
   length, from 1 through 65,536.
3. **Concatenation Typing:**
   $$\frac{\Gamma \vdash A : T^a \quad \Gamma \vdash B : T^b \quad a + b \le 65,536}{\Gamma \vdash (A \mathbin{+\!+} B) : T^{a+b}}$$

### §28. Explicit Value Conversions (`expr as T`) and Typing Judgments

Orange strictly rejects implicit type coercions. Conversions MUST be explicit:

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{AdmissibleCast}(\tau_{\text{src}}, \tau_{\text{dst}})}{\Gamma \vdash (e \text{ as } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

#### Admissible Cast Table
| Source Type ($\tau_{\text{src}}$) | Target Type ($\tau_{\text{dst}}$) | Operational Semantic Meaning |
| :--- | :--- | :--- |
| `Word[W]` | `Int` | Maps residue $x \in [0, 2^W-1]$ to exact integer $x \in \mathbb{Z}$. |
| Any scalar other than `Bool` | Any scalar other than `Bool` | Take the integer value (the least residue, for a residue). For `Word[n]` or `Mod[m]`, reduce modulo $2^n$ or $m$. The conversion does not reject an out-of-range `Int`. |
| `Bool` | any, or any to `Bool` | Not defined (`ORC0215`). |
| Words, a word, or an array of words, in a byte order | Words of the same number of bits, `Int`, or `Mod[m]` | `as big` / `as little`, §29. A different bit width is `ORC0240`. |

Applying `as` to an untyped literal (e.g. `(42) as Word[32]`) emits diagnostic `ORC0220`.

### §29. Endianness Homomorphisms and Bit-Preserving Packing (`as big T`, `as little T`)

In cryptographic specifications, byte arrays are routinely repacked into words:

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{TotalBits}(\tau_{\text{src}}) = \text{TotalBits}(\tau_{\text{dst}})}{\Gamma \vdash (e \text{ as big } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

$$\frac{\Gamma \vdash e : \tau_{\text{src}} \quad \text{TotalBits}(\tau_{\text{src}}) = \text{TotalBits}(\tau_{\text{dst}})}{\Gamma \vdash (e \text{ as little } \tau_{\text{dst}}) : \tau_{\text{dst}}}$$

1. **Bit Invariance Requirement:**
   $$\text{TotalBits}(\tau_{\text{src}}) \ne \text{TotalBits}(\tau_{\text{dst}}) \implies \text{Diagnostic}(\text{ORC0240})$$
2. **Packing Equations:**
   Let $B : \text{Word}[8]^k$ be an array of bytes converted to $W : \text{Word}[8k]$:
   - **Big-Endian:**
     $$W = \sum_{i=0}^{k-1} B[i] \cdot 2^{8(k - 1 - i)}$$
   - **Little-Endian:**
     $$W = \sum_{i=0}^{k-1} B[i] \cdot 2^{8i}$$

### §30. Dependent Finite Size Parameters and Monomorphization

1. A specification function may declare finite size parameters:
   ```orange
   spec pad[len in 1..64](msg: Word[8]^len) -> Word[8]^64 { ... }
   ```
2. **Domain:** `n in a..b` takes each integer from `a` up to but not including `b`.
   The bounds satisfy `a < b` and `b <= 65536`. `a` may be 0. An empty or
   reversed range, or a bound above 65536, is `ORC0238`. A function has at most
   four size parameters and at most 256 instances.
3. **Instances:** Each instance is checked as the function written out with
   those values. Only the first instance in error is reported. Sizes cost
   nothing at run time.
4. Calling with an invalid size parameter count emits `ORC0239`.

### §31. Finite Type Parameter Domains ($[K \in \{T_1, \dots, T_m\}]$)

1. A specification function may declare finite type parameters:
   ```orange
   spec square[K in {Mod[p], Mod[q]}](x: K) -> K { x * x }
   ```
2. The domain set $\{T_1, \dots, T_m\}$ MUST be finite, non-empty, and explicitly listed.
3. Repeating a type in the set, or instantiating with a type outside the set, emits
   diagnostic `ORC0241`.


---

## Part IV: Static Semantics (Typing Rules & Judgments)

### §32. Typing Contexts: Signature ($\Sigma$), Size ($\Theta$), Type ($\Delta$), and Variable ($\Gamma$) Environments

Static semantics in the `spec` stratum operates over a four-component typing context:

$$\mathcal{C} = \langle \Sigma, \Theta, \Delta, \Gamma \rangle$$

1. **Global Signature Context ($\Sigma$):**
   $$\Sigma : \text{ModIdent} \times \text{FuncIdent} \to (\tau_0 \times \dots \times \tau_{k-1} \to \tau_{\text{ret}})$$
   Maps qualified function identifiers to their declared parameter and return signatures.
2. **Size Parameter Context ($\Theta$):**
   $$\Theta : \text{SizeIdent} \to [\text{low}, \text{high}]$$
   Maps static size parameter identifiers to their closed integer bounding intervals.
3. **Type Parameter Context ($\Delta$):**
   $$\Delta : \text{TypeIdent} \to \mathcal{P}(\text{Types})$$
   Maps type parameter variables to their finite, explicitly admitted type candidate sets.
4. **Local Typing Context ($\Gamma$):**
   $$\Gamma : \text{VarIdent} \to \tau$$
   Maps local variable bindings and function parameters to their concrete types.

### §33. Subtyping, Coercion Freedom, and Structural Type Equality

1. **Coercion Freedom:**
   The Orange type system contains no implicit coercions, subtyping, or width widening:
   $$\tau_1 \le \tau_2 \iff \tau_1 \equiv \tau_2$$
2. **Definitional Structural Type Equality:**
   Type equality $\tau_1 \equiv \tau_2$ is reflexive, symmetric, and transitive:
   - $\text{Int} \equiv \text{Int}$
   - $\text{Bool} \equiv \text{Bool}$
   - $\text{Word}[W_1] \equiv \text{Word}[W_2] \iff W_1 = W_2$
   - $\text{Mod}[m_1] \equiv \text{Mod}[m_2] \iff m_1 = m_2$
   - $T_1^{n_1} \equiv T_2^{n_2} \iff T_1 \equiv T_2 \land n_1 = n_2$
   - $(T_0, \dots, T_{k-1}) \equiv (U_0, \dots, U_{m-1}) \iff k = m \land (\forall i < k.\ T_i \equiv U_i)$

### §34. Expression Typing Rules and Formal Judgments

We formalize the static semantics of the `spec` stratum as an inductive inference system
over the four-component typing context $\mathcal{C} = \langle \Sigma, \Theta, \Delta, \Gamma \rangle$.
The primary typing judgment is written:

$$\mathcal{C} \vdash e : \tau$$

denoting that under signatures $\Sigma$, size parameters $\Theta$, type parameters $\Delta$,
and local variables $\Gamma$, expression $e$ is well-typed with unique principal type $\tau$.

#### 1. Identifiers and Variables
$$\frac{x : \tau \in \Gamma}{\langle \Sigma, \Theta, \Delta, \Gamma \rangle \vdash x : \tau} \quad (\text{T-Var})$$

$$\frac{x \notin \Gamma \quad x \notin \text{dom}(\Sigma)}{\langle \Sigma, \Theta, \Delta, \Gamma \rangle \vdash x : \text{Error}(\text{ORC0211})} \quad (\text{T-Var-Err})$$

#### 2. Literals and Constants
An integer literal whose magnitude uses at most 16,384 significant bits may be typed `Int` when its context requires `Int`. A magnitude past that budget is `ORC0205`. A negative literal is the token `-` immediately before the integer token.

$$\frac{b \in \{\text{true}, \text{false}\}}{\mathcal{C} \vdash b : \text{Bool}} \quad (\text{T-Bool-Lit})$$

$$\frac{s \text{ is ASCII string of length } n \quad 1 \le n \le 65,536}{\mathcal{C} \vdash s : \text{Word}[8]^n} \quad (\text{T-String-Lit})$$

$$\frac{h \text{ is hex string of } 2n \text{ valid nibbles} \quad 1 \le n \le 65,536}{\mathcal{C} \vdash h : \text{Word}[8]^n} \quad (\text{T-Hex-Lit})$$

#### 3. Ring and Arithmetic Expressions
Arithmetic operations are typed according to their underlying algebraic domains.
Mixed-type arithmetic is strictly rejected:

$$\frac{\mathcal{C} \vdash a : \text{Int} \quad \mathcal{C} \vdash b : \text{Int}}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Int}} \quad (\text{op} \in \{+, -, *, /, \%\}) \quad (\text{T-Arith-Int})$$

$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad \mathcal{C} \vdash b : \text{Word}[W]}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Word}[W]} \quad (\text{op} \in \{+, -, *, /, \%\}) \quad (\text{T-Arith-Word})$$

$$\frac{\mathcal{C} \vdash a : \text{Mod}[m] \quad \mathcal{C} \vdash b : \text{Mod}[m]}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Mod}[m]} \quad (\text{op} \in \{+, -, *, /\}) \quad (\text{T-Arith-Mod})$$

$$\frac{\mathcal{C} \vdash a : \tau_1 \quad \mathcal{C} \vdash b : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Error}(\text{ORC0214})} \quad (\text{T-Arith-Mismatch})$$

$$\frac{\mathcal{C} \vdash a : \tau \quad \text{op} \text{ undefined for } \tau}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Error}(\text{ORC0215})} \quad (\text{T-Arith-Undefined})$$

$$\frac{\mathcal{C} \vdash a : \text{Int}}{\mathcal{C} \vdash {-}a : \text{Int}} \quad (\text{T-Neg-Int}) \qquad \frac{\mathcal{C} \vdash a : \text{Mod}[m]}{\mathcal{C} \vdash {-}a : \text{Mod}[m]} \quad (\text{T-Neg-Mod})$$

#### 4. Bitwise Operators
Bitwise operators are strictly restricted to word rings:

$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad \mathcal{C} \vdash b : \text{Word}[W]}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Word}[W]} \quad (\text{op} \in \{\&, |, \hat{}\}) \quad (\text{T-Bitwise})$$

$$\frac{\mathcal{C} \vdash a : \text{Word}[W]}{\mathcal{C} \vdash {\sim}a : \text{Word}[W]} \quad (\text{T-Bitwise-Not})$$

$$\frac{\mathcal{C} \vdash a : \tau \quad \tau \ne \text{Word}[W]}{\mathcal{C} \vdash {\sim}a : \text{Error}(\text{ORC0215})} \quad (\text{T-Bitwise-Not-Err})$$

#### 5. Shift and Rotation Operators (§S3b, §S3r)
$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad \mathcal{C} \vdash k : T_k \quad T_k \in \{\text{Int}, \text{Word}[U]\}}{\mathcal{C} \vdash a \mathbin{\text{op}} k : \text{Word}[W]} \quad (\text{op} \in \{<<, >>, <<<, >>>\}) \quad (\text{T-Shift})$$

$$\frac{\mathcal{C} \vdash a : \text{Word}[W] \quad k \in \text{Literals} \quad (k < 0 \lor k \ge W)}{\mathcal{C} \vdash a \mathbin{\text{op}} k : \text{Error}(\text{ORC0216})} \quad (\text{T-Shift-Lit-Range})$$

#### 6. Relational Comparisons
Relational equality and orderings operate over homogeneous scalars:

`==` and `!=` apply to `Int`, `Word[W]`, `Mod[m]`, and to whole arrays and tuples. `<`, `<=`, `>`, and `>=` apply to `Int` and to unsigned words. They do not apply to `Mod[m]`, `Bool`, arrays, or tuples (`ORC0215`).

$$\frac{\mathcal{C} \vdash a : \tau_1 \quad \mathcal{C} \vdash b : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Error}(\text{ORC0214})} \quad (\text{T-Rel-Mismatch})$$

$$\frac{a, b \text{ are untyped integer literals}}{\mathcal{C} \vdash a \mathbin{\text{cmp}} b : \text{Error}(\text{ORC0227})} \quad (\text{T-Rel-Untyped})$$

#### 7. Logical Connectives
$$\frac{\mathcal{C} \vdash a : \text{Bool} \quad \mathcal{C} \vdash b : \text{Bool}}{\mathcal{C} \vdash a \mathbin{\text{op}} b : \text{Bool}} \quad (\text{op} \in \{\&\&, ||\}) \quad (\text{T-Logic})$$

$$\frac{\mathcal{C} \vdash a : \text{Bool}}{\mathcal{C} \vdash !a : \text{Bool}} \quad (\text{T-Logic-Not})$$

#### 8. Local Bindings and Pattern Destructuring (§S3c)
$$\frac{\mathcal{C} \vdash e : \tau \quad x \notin \Gamma \quad \langle \Sigma, \Theta, \Delta, (\Gamma, x : \tau) \rangle \vdash \text{body} : \tau_{\text{body}}}{\mathcal{C} \vdash (\text{let } x : \tau = e; \ \text{body}) : \tau_{\text{body}}} \quad (\text{T-Let})$$

A tuple pattern is `let (x0: T0, ..., xk: Tk) = e`, with a type on every name. An array is not a tuple: `let (a: Word[32], ...) = state` is `ORC0214` when `state` is an array. Names follow the same duplicate rule as other bindings (`ORC0219`).

$$\frac{x \in \Gamma}{\mathcal{C} \vdash (\text{let } x : \tau = e; \ \text{body}) : \text{Error}(\text{ORC0219})} \quad (\text{T-Let-Shadow})$$

#### 9. Conditionals (§S3f)
$$\frac{\mathcal{C} \vdash c : \text{Bool} \quad \mathcal{C} \vdash e_1 : \tau \quad \mathcal{C} \vdash e_2 : \tau}{\mathcal{C} \vdash (\text{if } c \ \{ e_1 \} \ \text{else } \{ e_2 \}) : \tau} \quad (\text{T-If})$$

$$\frac{\mathcal{C} \vdash c : \text{Bool} \quad \mathcal{C} \vdash e_1 : \tau_1 \quad \mathcal{C} \vdash e_2 : \tau_2 \quad \tau_1 \not\equiv \tau_2}{\mathcal{C} \vdash (\text{if } c \ \{ e_1 \} \ \text{else } \{ e_2 \}) : \text{Error}(\text{ORC0214})} \quad (\text{T-If-Mismatch})$$

#### 10. Array Construction, Indexing, Slicing, and Functional Update (§S3d, §S3e, §S3g)
$$\frac{\forall i \in [0, n-1].\ \mathcal{C} \vdash e_i : T \quad 1 \le n \le 65,536}{\mathcal{C} \vdash [e_0, e_1, \dots, e_{n-1}] : T^n} \quad (\text{T-Array-Lit})$$

$$\frac{\mathcal{C} \vdash v : T \quad 1 \le n \le 65,536}{\mathcal{C} \vdash [v; n] : T^n} \quad (\text{T-Array-Fill})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{C} \vdash i : \text{Index}(n)}{\mathcal{C} \vdash A[i] : T} \quad (\text{T-Index})$$

$$\frac{\mathcal{C} \vdash A : T^n \quad \mathcal{C} \vdash i : \text{Index}(n) \quad \mathcal{C} \vdash v : T}{\mathcal{C} \vdash (A \text{ with } [i] = v) : T^n} \quad (\text{T-Update})$$

A slice `x[a..b]` takes elements from `a` up to but not including `b`. The bounds are integer literals and loop indices combined by `+`, `-`, and `*` by a constant. The distance `b - a` is a fixed positive length, proved in range at every step (`ORC0236` otherwise). An omitted bound is 0 or the length. A slice's position does not depend on data.

$$\frac{\mathcal{C} \vdash A : T^a \quad \mathcal{C} \vdash B : T^b \quad a + b \le 65,536}{\mathcal{C} \vdash (A \mathbin{+\!+} B) : T^{a+b}} \quad (\text{T-Concat})$$

$$\frac{\mathcal{C} \vdash X : \tau \quad \tau \not\equiv T^n}{\mathcal{C} \vdash X[i] : \text{Error}(\text{ORC0224})} \quad (\text{T-Not-Array})$$

#### 11. Bounded Iteration Loops (§S3e)
A loop `for i in a..b with p = e { body }` requires `0 <= a < b <= 65536`. An empty or reversed range is `ORC0225`. The accumulator pattern is a typed name or a tuple of typed names, not a bare name. `i` is an `Int`.

#### 12. Function Application and Monomorphization

Sizes and type arguments share one bracket list, as in `f[2](x)` or `pow[F](x, e)`. There is no second bracket list. A call has at most four bracket entries and at most 256 arguments. `f(args)` selects the single instance whose parameters match; literal lengths decide only where a type does not.

### §35. Expression grouping

Operators from different groups, or two shifts, two comparisons, or two divisions, do not share a level. The missing parentheses are `ORC0108`.

- `+` and `-` chain.
- `*` chains. `/` and `%` are a separate division form and do not chain with `*` or with each other.
- `&`, `|`, `^`, `&&`, `||`, and `++` each chain inside their own group.
- A shift or rotation is one operator between two operands.
- A comparison is one operator between two operands.
- A conversion or an update shares a level with no operator and with no other conversion or update.
- `^` after a type is an array length. Elsewhere `^` is exclusive or.

Expressions nest at most 64 levels and reach height 256 (`MAX_EXPRESSION_NESTING`, `MAX_EXPRESSION_HEIGHT` in `parser.rs`).

### §36. What an index may depend on

Subject: array indexing in the implemented S3g surface, as specified in [`LOOKUPS_2026.md`](LOOKUPS_2026.md), which is in review.
This compiler does not publish the interval lattice written in earlier drafts of this section. The rule that is implemented is the one in Appendix A of the Book and in that slice document:

An index is checked as the word type of its first name, call, conversion, or element, and ranges over that type, narrowed by its operators. Otherwise it is an `Int` built from integer literals, loop indices, and words converted with `as Int`, using `+`, `-`, `*`, `/`, `%`, and conditionals. A word index of width `W` is in `0 .. 2^W`. That is why `s[a[i]]` is admitted when `s` has length 256 and `a[i]` is a `Word[8]`. An index that is not proved below the length is `ORC0223`. No index is checked again at run time.

### §37. Rank-two arrays and slices

Rank-two arrays are aliases, not a repeated caret. `type Row = Word[32]^4; type Matrix = Row^4;` is the form in [`NESTED_ARRAYS_2026.md`](NESTED_ARRAYS_2026.md). Both axes are checked, and their scalar product is at most 65,536. Repeated `^` in one type is rejected. `m[i][j]` is two index suffixes. A slice's bounds do not depend on data (§34).

## Part V: Dynamic Semantics (Operational & Reduction Rules)

### §38. Abstract Syntax and Typed Reference Core Lowering

Before evaluation, the compiler lowers the typed AST into **Typed Reference Core IR** (TRC):

$$\text{Source AST} \xrightarrow{\text{Lex/Parse}} \text{Surface AST} \xrightarrow{\text{Elaborate \& Type}} \text{Core IR (TRC)}$$

TRC enforces four architectural invariants:
1. **Full Monomorphization:** All size parameters $[n \in low..high]$ and type parameters
   $[K \in \{T_1, \dots\}]$ are replaced with specialized, ground monomorphic instances.
2. **Type De-Aliasing:** All transparent `type` aliases are eliminated in favor of
   canonical structural representations.
3. The Core is the compiler's internal form for one revision. It is not a
   De Bruijn calculus, and this manual does not define a second core. The Book
   records that the Core is noncanonical and carries no proof identity.

### §39. Evaluation Environments, Value Stores, and Step Budgets

#### 1. Semantic Values ($\mathbb{V}$)
$$\begin{array}{rcll}
v & ::= & c_{\text{Int}} & (\text{integers } n \in \mathbb{Z}) \\
  & \mid & c_{\text{Word}[W]} & (\text{ring elements } r \in [0, 2^W - 1]) \\
  & \mid & c_{\text{Mod}[m]} & (\text{residue elements } r \in [0, m - 1]) \\
  & \mid & \text{true} \mid \text{false} & (\text{booleans}) \\
  & \mid & [v_0, v_1, \dots, v_{n-1}] & (\text{arrays of length } n) \\
  & \mid & (v_0, v_1, \dots, v_{k-1}) & (\text{tuples of arity } k)
\end{array}$$

#### 2. Evaluation Environment ($\rho$)
$$\rho \in \text{Env} = \text{Ident} \rightharpoonup \mathbb{V}$$

An immutable association mapping variable identifiers to semantic values.

#### 3. Deterministic Step Budget ($K$)
Dynamic execution is guarded by a monotonically decreasing step counter $K \in \mathbb{N}$,
The default budget is 1,048,576 steps. `--steps` sets a budget from 1 through 1,073,741,824. Exhaustion is `ORC0301`.
Not every operation costs one step. An update, a fill, a join, a slice, or a slice update costs one step per 64 elements, or part of 64. A byte string costs one. A conversion in a byte order costs one step per 64 bits, or part of 64. A shift or rotation costs one step whatever the amount.

### §40. Small-Step Operational Semantics (SOS) and Big-Step Reduction

We define the small-step evaluation relation over expressions:

$$e \longrightarrow e'$$

using evaluation contexts $E[\cdot]$ that formalize strict left-to-right evaluation order:

$$\begin{array}{rcl}
E & ::= & [\cdot] \mid E \mathbin{op} e_2 \mid v_1 \mathbin{op} E \mid \mathbf{unop}\; E \\
  & \mid & \text{let } x : \tau = E; \ e_2 \\
  & \mid & \text{let } (x_0, \dots, x_{k-1}) = E; \ e_2 \\
  & \mid & \text{if } E \ \{ e_1 \} \ \text{else } \{ e_2 \} \\
  & \mid & E[e] \mid v[E] \\
  & \mid & E \text{ with } [e_1] = e_2 \mid v \text{ with } [E] = e_2 \mid v \text{ with } [v_1] = E \\
  & \mid & [v_0, \dots, v_{i-1}, E, e_{i+1}, \dots] \\
  & \mid & (v_0, \dots, v_{i-1}, E, e_{i+1}, \dots) \\
  & \mid & E.j \\
  & \mid & E \text{ as } \tau \mid E \text{ as big } \tau \mid E \text{ as little } \tau \\
  & \mid & f(v_0, \dots, v_{i-1}, E, e_{i+1}, \dots)
\end{array}$$

#### Contextual Transition Rule
$$\frac{r \longrightarrow_{\text{redex}} r'}{E[r] \longrightarrow E[r']} \quad (\text{SOS-Context})$$

#### Primitive Redex Contractions ($r \longrightarrow_{\text{redex}} r'$)

Surface `let` patterns carry a type on every name (§34). The names in the rules below are the typed surface patterns with the types omitted from the formula.
1. **Variable Lookup:**
   $$\rho(x) = v \implies x \longrightarrow_{\text{redex}} v \quad (\text{R-Var})$$
2. **Word Ring Operations ($W \in \{8, 16, 32, 64\}$):**
   $$v_1 +_{W} v_2 \longrightarrow_{\text{redex}} (v_1 + v_2) \bmod 2^W \quad (\text{R-Add-Word})$$
   $$v_1 -_{W} v_2 \longrightarrow_{\text{redex}} (v_1 - v_2 + 2^W) \bmod 2^W \quad (\text{R-Sub-Word})$$
   $$v_1 *_{W} v_2 \longrightarrow_{\text{redex}} (v_1 \cdot v_2) \bmod 2^W \quad (\text{R-Mul-Word})$$
3. **Modular Residue Operations ($m \in [2, 2^{521}-1]$):**
   $$v_1 +_{m} v_2 \longrightarrow_{\text{redex}} (v_1 + v_2) \bmod m \quad (\text{R-Add-Mod})$$
   $$v_1 -_{m} v_2 \longrightarrow_{\text{redex}} (v_1 - v_2 + m) \bmod m \quad (\text{R-Sub-Mod})$$
   $$v_1 *_{m} v_2 \longrightarrow_{\text{redex}} (v_1 \cdot v_2) \bmod m \quad (\text{R-Mul-Mod})$$
   $$\gcd(v_2, m) = 1 \implies v_1 /_{m} v_2 \longrightarrow_{\text{redex}} (v_1 \cdot v_2^{-1}) \bmod m \quad (\text{R-Div-Mod})$$
   $$\gcd(v_2, m) \ne 1 \implies v_1 /_{m} v_2 \longrightarrow_{\text{redex}} 0 \quad (\text{R-Div-Mod-Zero})$$
4. **Conditionals:**
   $$\text{if } \text{true} \ \{ e_1 \} \ \text{else } \{ e_2 \} \longrightarrow_{\text{redex}} e_1 \quad (\text{R-If-True})$$
   $$\text{if } \text{false} \ \{ e_1 \} \ \text{else } \{ e_2 \} \longrightarrow_{\text{redex}} e_2 \quad (\text{R-If-False})$$
5. **Local Binding Substitution:**
   $$\text{let } x : \tau = v; \ e_2 \longrightarrow_{\text{redex}} e_2[v / x] \quad (\text{R-Let})$$
   $$\text{let } (x_0, \dots, x_{k-1}) = (v_0, \dots, v_{k-1}); \ e_2 \longrightarrow_{\text{redex}} e_2[v_0/x_0, \dots, v_{k-1}/x_{k-1}] \quad (\text{R-Let-Tuple})$$
6. **Array Indexing and Update:**
   $$[v_0, \dots, v_{n-1}][k] \longrightarrow_{\text{redex}} v_k \quad (\text{where } 0 \le k < n) \quad (\text{R-Index})$$
   $$([v_0, \dots, v_{n-1}] \text{ with } [k] = u) \longrightarrow_{\text{redex}} [v_0, \dots, v_{k-1}, u, v_{k+1}, \dots, v_{n-1}] \quad (\text{R-Update})$$
7. **Tuple Projection:**
   $$(v_0, \dots, v_{k-1}).j \longrightarrow_{\text{redex}} v_j \quad (\text{where } 0 \le j < k) \quad (\text{R-Tuple-Proj})$$
8. **Function Invocation:**
   $$f(v_0, \dots, v_{m-1}) \longrightarrow_{\text{redex}} e_{\text{body}}[v_0/p_0, \dots, v_{m-1}/p_{m-1}] \quad (\text{R-Call})$$

### §41. Operational Reduction of Variable Shifts, Rotations, and Inversions

Let $v_x \in [0, 2^W - 1]$ be a word residue and $k \in \mathbb{Z}$ an evaluation of amount:

1. **Logical Left Shift ($v_x \ll k$):**
   $$v = \begin{cases} (v_x \cdot 2^k) \bmod 2^W & \text{if } 0 \le k < W \\ 0 & \text{if } k \ge W \\ \lfloor v_x / 2^{-k} \rfloor & \text{if } k < 0 \end{cases}$$
2. **Logical Right Shift ($v_x \gg k$):**
   $$v = \begin{cases} \lfloor v_x / 2^k \rfloor & \text{if } 0 \le k < W \\ 0 & \text{if } k \ge W \\ (v_x \cdot 2^{-k}) \bmod 2^W & \text{if } k < 0 \end{cases}$$
3. **Left Circular Rotation ($v_x \lll k$):**
   Let $r = k \bmod W$ with $0 \le r < W$:
   $$v = ((v_x \ll r) \mathbin{|} (v_x \gg (W - r))) \bmod 2^W$$
4. **Right Circular Rotation ($v_x \ggg k$):**
   Let $r = k \bmod W$ with $0 \le r < W$:
   $$v = ((v_x \gg r) \mathbin{|} (v_x \ll (W - r))) \bmod 2^W$$

### §42. Bounded Iteration Semantics and Step-Cost Accounting

Consider the bounded iteration loop:
$$L = \text{for } i \text{ in } \text{low}..\text{high} \text{ with } a = e_{\text{init}} \ \{ e_{\text{body}} \}$$

1. **Iteration Bound:** Let $N = \text{high} - \text{low} \in \mathbb{N}$. Accepted loops satisfy `0 <= low < high <= 65536`, so `N = high - low` is at least 1 and at most 65,536.
2. **Operational Transition Sequence:**
   - Initialize accumulator: $\langle e_{\text{init}}, \rho, K \rangle \Downarrow \langle a_0, K_0 \rangle$.
   - For each step $j = 0, 1, \dots, N-1$:
     - Construct iteration environment: $\rho_j = \rho[i \mapsto \text{low} + j, a \mapsto a_j]$.
     - Evaluate loop body: $\langle e_{\text{body}}, \rho_j, K_j \rangle \Downarrow \langle a_{j+1}, K_{j+1} \rangle$.
     - Verify step budget: $K_{j+1} > 0$. If $K_{j+1} = 0$, abort with `ORC0301`.
   - The loop expression evaluates to final accumulator $a_N$.

### §43. Known-Answer Specification Tests (`test`) and Whole-Aggregate Equality

1. **Test Execution Semantics:**
   A test `test "Title" { e }` is evaluated under the empty environment $\rho_0$:
   $$\langle e, \rho_0, K_0 \rangle \Downarrow \langle v, K' \rangle$$
   - `orangec test` prints `test "TITLE" ... ok` or `test "TITLE" ... FAILED`.
   - Exit status is 1 when any test fails.
2. **Whole-Aggregate Structural Equality (`==`):**
   Structural equality recursively compares compound elements:
   - For arrays $A, B : T^n$:
     $$A == B \iff \bigwedge_{i=0}^{n-1} (A[i] == B[i])$$
   - For tuples $A, B : (T_0, \dots, T_{k-1})$:
     $$A == B \iff \bigwedge_{j=0}^{k-1} (A.j == B.j)$$

---

## Part VI: Properties that are not theorems

### §44. Evaluator limits and the absence of a proof

Subject: reference evaluation of a program that has already passed semantic analysis on one `orangec` revision.
Assumptions: the step budget is not exhausted, every `Int` stays within 16,384 significant bits, and no resource diagnostic is raised.
Evidence scope: the evaluator's tests for that revision. This repository does not contain a machine-checked proof of strong normalization, type safety, confluence, or an endianness isomorphism.

Earlier drafts of this manual stated those four results as proved metatheorems of Orange. They are withdrawn. The sketches omitted the step budget (`ORC0301`), the integer-bit budget (`ORC0205`), and the aggregate step costs in §39, and they were not checked by `orange-check` or by any other kernel. D-006 is `investigate`. D-007 is `proposed` and depends on D-006. No proof kernel is part of this compiler.

What the implemented evaluator does provide, under the assumptions above, is a deterministic result for the same source bytes and the same revision: the same values, the same diagnostic codes, and the same success or failure. That is an implementation property of the recorded revision, not a theorem of this manual.

---

## Part VII: Partial standard transcriptions

### §48. How a transcription is scoped

Subject: the four modules in §49–§52.
Assumption: the reader wants a program the implemented surface accepts, not a claim that a standard has been transcribed in full or validated as cryptography.
Evidence: `orangec check` accepts each module, and `orangec test` reports the single ChaCha20 quarter-round test as ok. No other known-answer vector is claimed here. The slice specifications remain in review.

These modules are not complete SHA-256, ChaCha20, X25519, or Poly1305. They omit constants, padding, and the rest of each standard. They are illustrations of wording that typechecks. They are not evidence of functional correctness against the standard.

Operator spelling used by those illustrations, and by the implemented word and array surface:

| Standard notation | Orange spelling | Subject |
| --- | --- | --- |
| $x \\oplus y$ | `x ^ y` | `Word[n]` |
| $x \\land y$ | `x & y` | `Word[n]` |
| $\\neg x$ | `~x` | `Word[n]` |
| addition modulo $2^{32}$ | `x + y` | both operands `Word[32]` |
| $\\mathrm{ROTR}^n(x)$ | `x >>> n` | `Word[n]`; a literal amount is in `0 .. width - 1` |
| $\\mathrm{SHR}^n(x)$ | `x >> n` | `Word[n]` |
| concatenation | `A ++ B` | arrays of one element type |

### §49. Partial transcription: FIPS 180-4 SHA-256

Clauses shown: FIPS 180-4 functions Ch, Maj, $\Sigma_0$, $\Sigma_1$, $\sigma_0$, $\sigma_1$; the eight initial hash words; a message-schedule step; one compression step. Not shown: the round constants, padding, and the full compression function.

An array is not a tuple. The eight state words are `state[0]` through `state[7]`. A pattern `let (a, b, c, d, e, f, g, h) = state` is not the implemented syntax: the names need types, and the right-hand side would still be an array (`ORC0214`).

```orange
edition 2026;

module sha256_spec {
    spec ch(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
        (x & y) ^ (~x & z)
    }

    spec maj(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {
        (x & y) ^ (x & z) ^ (y & z)
    }

    spec big_sigma0(x: Word[32]) -> Word[32] {
        (x >>> 2) ^ (x >>> 13) ^ (x >>> 22)
    }

    spec big_sigma1(x: Word[32]) -> Word[32] {
        (x >>> 6) ^ (x >>> 11) ^ (x >>> 25)
    }

    spec small_sigma0(x: Word[32]) -> Word[32] {
        (x >>> 7) ^ (x >>> 18) ^ (x >> 3)
    }

    spec small_sigma1(x: Word[32]) -> Word[32] {
        (x >>> 17) ^ (x >>> 19) ^ (x >> 10)
    }

    spec initial_state() -> Word[32]^8 {
        [
            0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
            0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19
        ]
    }

    spec expand_message(block: Word[8]^64) -> Word[32]^64 {
        let words: Word[32]^16 = block as big Word[32]^16;
        for i in 16..64 with w: Word[32]^64 = words ++ [0; 48] {
            let s0: Word[32] = small_sigma0(w[i - 15]);
            let s1: Word[32] = small_sigma1(w[i - 2]);
            let next_w: Word[32] = w[i - 16] + s0 + w[i - 7] + s1;
            w with [i] = next_w
        }
    }

    spec compress_step(
        state: Word[32]^8,
        w_i: Word[32],
        k_i: Word[32]
    ) -> Word[32]^8 {
        let a: Word[32] = state[0];
        let b: Word[32] = state[1];
        let c: Word[32] = state[2];
        let d: Word[32] = state[3];
        let e: Word[32] = state[4];
        let f: Word[32] = state[5];
        let g: Word[32] = state[6];
        let h: Word[32] = state[7];
        let t1: Word[32] = h + big_sigma1(e) + ch(e, f, g) + k_i + w_i;
        let t2: Word[32] = big_sigma0(a) + maj(a, b, c);
        [t1 + t2, a, b, c, d + t1, e, f, g]
    }
}
```

### §50. Partial transcription: RFC 8439 ChaCha20 quarter round

Subject: RFC 8439 §2.1 and the §2.1.1 quarter-round vector only.
`orangec test` on this module prints the test as ok. That is one vector, not a ChaCha20 block function.

```orange
edition 2026;

module chacha20_spec {
    spec quarter_round(
        a: Word[32],
        b: Word[32],
        c: Word[32],
        d: Word[32]
    ) -> (Word[32], Word[32], Word[32], Word[32]) {
        let a1: Word[32] = a + b;
        let d1: Word[32] = (d ^ a1) <<< 16;
        let c1: Word[32] = c + d1;
        let b1: Word[32] = (b ^ c1) <<< 12;
        let a2: Word[32] = a1 + b1;
        let d2: Word[32] = (d1 ^ a2) <<< 8;
        let c2: Word[32] = c1 + d2;
        let b2: Word[32] = (b1 ^ c2) <<< 7;
        (a2, b2, c2, d2)
    }

    test "RFC 8439 Section 2.1.1 ChaCha20 Quarter-Round" {
        let (a: Word[32], b: Word[32], c: Word[32], d: Word[32]) = quarter_round(
            0x11111111,
            0x01020304,
            0x9b8d6f43,
            0x01234567
        );
        (a == 0xea2a92f4) && (b == 0xcb1cf8ce) && (c == 0x4581472e) && (d == 0x5881c4bb)
    }
}
```

### §51. Partial transcription: RFC 7748 Montgomery ladder step

Subject: the Curve25519 field `2^255 - 19` and one differential addition step in the shape of RFC 7748 §5.
`121665` is a `Fe` literal binding. `121665 as Fe` is `ORC0220` because a bare literal has no type. Division by a non-unit is defined and yields 0; this step does not use that case. No X25519 scalar multiplication is claimed.

```orange
edition 2026;

module curve25519_spec {
    type Fe = Mod[(1 << 255) - 19];

    spec fe_add(a: Fe, b: Fe) -> Fe { a + b }
    spec fe_sub(a: Fe, b: Fe) -> Fe { a - b }
    spec fe_mul(a: Fe, b: Fe) -> Fe { a * b }
    spec fe_sq(a: Fe) -> Fe { a * a }
    spec fe_inv(a: Fe) -> Fe {
        let one: Fe = 1;
        one / a
    }

    spec montgomery_ladder_step(
        x1: Fe,
        x2: Fe,
        z2: Fe,
        x3: Fe,
        z3: Fe
    ) -> (Fe, Fe, Fe, Fe) {
        let da: Fe = (x3 - z3) * (x2 + z2);
        let cb: Fe = (x3 + z3) * (x2 - z2);
        let next_x3: Fe = (da + cb) * (da + cb);
        let next_z3: Fe = x1 * ((da - cb) * (da - cb));
        let aa: Fe = (x2 + z2) * (x2 + z2);
        let bb: Fe = (x2 - z2) * (x2 - z2);
        let e: Fe = aa - bb;
        let a24: Fe = 121665;
        let next_x2: Fe = aa * bb;
        let next_z2: Fe = e * (aa + (a24 * e));
        (next_x2, next_z2, next_x3, next_z3)
    }
}
```

### §52. Partial transcription: RFC 8439 Poly1305 field operations

Subject: the Poly1305 prime `2^130 - 5`, the `r` clamp from RFC 8439 §2.5.1, and one field multiplication of an accumulator by `r`.
Not shown: block splitting, the pad bit, or the secret `s` addition. No MAC tag is claimed.

```orange
edition 2026;

module poly1305_spec {
    type Fe = Mod[(1 << 130) - 5];

    spec clamp_r(r_bytes: Word[8]^16) -> Fe {
        let r_words: Word[32]^4 = r_bytes as little Word[32]^4;
        let c0: Word[32] = r_words[0] & 0x0fffffff;
        let c1: Word[32] = r_words[1] & 0x0ffffffc;
        let c2: Word[32] = r_words[2] & 0x0ffffffc;
        let c3: Word[32] = r_words[3] & 0x0ffffffc;
        let clamped: Word[8]^16 = [c0, c1, c2, c3] as little Word[8]^16;
        clamped as little Fe
    }

    spec poly1305_block(acc: Fe, r: Fe, block_val: Fe) -> Fe {
        (acc + block_val) * r
    }
}
```

---

## Part VIII: Strata that are not implemented language

### §53. Empty `impl`, reserved words, and open decisions

Subject: every stratum other than the `spec` surface implemented through S3t.
Assumption: none of D-004, D-005, D-006, D-007, D-011, or D-012 has been accepted.
Evidence: `LANGUAGE_2026.md` §2.2 and §6, `SEMANTICS_2026.md` §1, and the decision register. The compiler reserves `game`, `proof`, and `claim` and gives them no production.

| Spelling or topic | What the compiler does | What is not language |
| --- | --- | --- |
| `impl name() {}` | Accepted syntax. No type, value, or execution. | Bodies, places, assignments, loops as commands |
| `spec` / `impl` namespaces | Separate declaration namespaces (`ORC0201` note in `semantics.rs`) | A call-site stratum selector, or a refinement obligation between them |
| `game`, `proof`, `claim` | Lexical reservations. Rejected where an identifier or a member is required | Sampling, adversaries, proof terms, claim records |
| Machine code, vectors, intrinsics | Not implemented | `Vec128`, AES-NI, ABI layouts, register conventions, generated C or Rust headers |
| Information flow | Not implemented. D-012 is `investigate` and forbids a constant-time claim before it is decided | `public` / `secret` types, `declassify`, speculative noninterference policies |
| `requires`, `ensures`, `erase`, `old` | Not position words | Contracts, loop variants, zeroization statements |
| `orange-check`, Proof IR, LRAT, CBOM, SLSA | Not part of this toolchain | A TCB calculus or an evidence bundle format |

Earlier drafts of this manual stated ownership, separation logic, SIMD intrinsics, ABI frames, games, and claim schemas as normative Orange 1.0 rules, including a `Byte` alias of `Word[8]` in an ABI table and an `orange-check` kernel. Those sections are withdrawn. They contradicted the decision register, and they described syntax the parser does not accept. The Book remains the place that explains why those strata were proposed. This manual does not copy that explanation and does not turn it into lookup rules.

## Part IX: Diagnostic code catalog

### §93. Diagnostic Philosophy, Severity Structure, and Error Budgets

1. **Permanence of Diagnostic Identifiers:**
   Diagnostic codes in Orange are immutable elements of the compiler's user-facing
   interface. Existing numeric identifiers MUST NOT be repurposed, redefined, or
   reordered across releases.
2. **Severity Hierarchy:**
   - The severity of all compile-time diagnostic failures is fixed as `error`.
   - Any emitted error immediately invalidates downstream compiler phases (e.g. an
     error during parsing prevents semantic analysis; an error during semantic analysis
     prevents evaluation or code generation).
3. **Structured Diagnostic Record:**
   Every emitted diagnostic is a record:
   $$\text{Diag} = \langle \text{Code}, \text{Severity}, \text{PrimarySpan}, \text{Label}, \text{SecondarySpans}, \text{Notes} \rangle$$
4. **Diagnostic budget:**
   Each of lexing, parsing, and semantic analysis retains at most 100 ordinary
   diagnostics (`MAX_DIAGNOSTICS_PER_SOURCE`, `MAX_PARSE_DIAGNOSTICS_PER_SOURCE`,
   `MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE`), then one suppression diagnostic:
   `ORC0007`, `ORC0105`, or `ORC0208`. A resource diagnostic sits outside that
   ordinary budget.
5. **Catalog scope:**
   The headings below name codes and variants from
   `compiler/crates/orange-compiler/src/diagnostic.rs`. That enum is the
   inventory. Blocks that quoted a compiler rendering have been removed: they
   were not copies of `orangec` output. A trigger formula in this catalog is a
   lookup aid. Where it disagrees with `diagnostic.rs` or with a slice
   specification, those sources control. There is no code generation phase.

---

### §94. Lexical Diagnostics (`ORC0001`–`ORC0009`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0001` — `UnexpectedCharacter`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(c) \iff c \notin \text{Alphabet}(\text{Edition 2026}) \lor c \in \text{ForbiddenControls} \lor c = \text{BOM}$$
- **Rationale:** Prevents Trojan Source attacks, encoding ambiguities, and accidental
  syntax corruption from foreign codepoints.
- **Erroneous Example:**
  ```orange
  spec hash(m: Word[8]^32) -> Word[32] {
      let @temp = 10; // '@' has no lexical meaning
  }
  ```
- **Remediation:** Remove the character or replace it with valid ASCII tokens:
  ```orange
  spec hash(m: Word[8]^32) -> Word[32] {
      let temp: Word[32] = 10;
  }
  ```

#### `ORC0002` — `UnterminatedBlockComment`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{EOF reached} \land \text{CommentNestingDepth} > 0$$
- **Rationale:** Eliminates silent swallowing of source code caused by unclosed comments.
- **Erroneous Example:**
  ```orange
  /* Outer comment
     /* Nested comment */
  spec main() -> Int { 42 }
  ```
- **Remediation:** Append `*/` to close all open comment nesting levels.

#### `ORC0003` — `UnterminatedString`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{Scanned}(\texttt{"}) \land (\text{NextChar} = \texttt{\textbackslash n} \lor \text{NextChar} = \text{EOF})$$
- **Rationale:** Prevents multi-line string confusion and enforces single-line byte string literals.
- **Erroneous Example:**
  ```orange
  test "Unclosed test vector title {
      true
  }
  ```
- **Remediation:** Close the string literal with `"` on the same logical line.

#### `ORC0004` — `InvalidEscape`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\texttt{\textbackslash}c) \iff c \notin \{\texttt{"}, \texttt{\textbackslash}, \texttt{n}, \texttt{r}, \texttt{t}, \texttt{0}, \texttt{x}\} \lor (c = \texttt{x} \land \neg \text{IsHexDigitPair}(\text{Next}_2))$$
- **Rationale:** Ensures byte string literals have deterministic, unambiguous binary decodings.
- **Erroneous Example:**
  ```orange
  let s = "invalid \u0041 escape"; // Unicode escapes unsupported
  let bad_hex = "\x1z";
  ```
- **Remediation:** Use only supported escapes (`\"`, `\\`, `\n`, `\r`, `\t`, `\0`, `\xNN`):
  ```orange
  let s = "valid \x41 escape";
  let good_hex = "\x1a";
  ```

#### `ORC0005` — `MalformedInteger`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(tok) \iff \text{InvalidDigitsForBase}(tok) \lor \text{MisplacedUnderscore}(tok) \lor \text{MissingDigitsAfterPrefix}(tok)$$
- **Rationale:** Enforces strict lexical boundaries for numeric literals, eliminating silent base misinterpretations.
- **Erroneous Example:**
  ```orange
  let a = 0x_10;    // Underscore immediately after prefix
  let b = 100_;     // Trailing underscore
  let c = 0b102;    // '2' invalid for base 2
  ```
- **Remediation:** Place underscores only between valid digits:
  ```orange
  let a = 0x10;
  let b = 100;
  let c = 0b101;
  ```

#### `ORC0006` — `LexicalTokenLimit`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  More than 262,144 non-trivia tokens, excluding `EOF`.
- **Rationale:** Protects the compiler against algorithmic complexity attacks and out-of-memory crashes.
- **Remediation:** Partition large compilation units into multiple modular source files.

#### `ORC0007` — `TooManyLexicalErrors`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  A 101st ordinary lexical error. The diagnostic says reporting stopped after 100 lexical errors.
- **Remediation:** Fix early lexical errors and re-run compilation.

#### `ORC0008` — `LexicalResourceLimit`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  The lexer could not reserve its bounded token stream, or its source cursor could not be sliced on a UTF-8 boundary. A source over 16 MiB is `ORC1001` in the driver, before lexing.
- **Remediation:** Keep source files under 16 MiB.

#### `ORC0009` — `MalformedHexString`
- **Subsystem:** Lexical Analyzer
- **Formal Trigger Predicate:**
  $$\text{Trigger}(tok) \iff \text{Prefix}(tok) = \texttt{hex"} \land (\text{OddDigitCount}(tok) \lor \text{ContainsInvalidHexChars}(tok))$$
- **Rationale:** Byte sequences require exact 8-bit alignment.
- **Erroneous Example:**
  ```orange
  let key = hex"012";     // Odd number of nibbles (3 nibbles)
  let iv  = hex"01 02 gg"; // 'g' is not a hexadecimal digit
  ```
- **Remediation:** Provide exact byte pairs separated by optional single spaces:
  ```orange
  let key = hex"01 20";
  let iv  = hex"01 02 ff";
  ```

---

### §95. Syntactic Diagnostics (`ORC0101`–`ORC0108`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0101` — `ExpectedSyntax`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{CurrentToken} \ne \text{ExpectedProductionToken}$$
- **Erroneous Example:**
  ```orange
  edition 2026 // Missing semicolon
  module sha256 { }
  ```
- **Remediation:** Supply the required syntactic delimiter: `edition 2026;`.

#### `ORC0102` — `UnsupportedSourceEdition`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger}(E) \iff E \ne 2026$$
- **Erroneous Example:**
  ```orange
  edition 2025;
  ```
- **Remediation:** Specify the supported language edition: `edition 2026;`.

#### `ORC0103` — `ExpectedFunctionDeclaration`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  A module member does not start with `spec`, `impl`, or `test` in member position, or a `use`/`type` is out of order. `game`, `proof`, and `claim` are not admitted members.
- **Remediation:** Begin module declarations with a valid top-level member keyword.

#### `ORC0104` — `TrailingSyntax`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{CurrentToken} \ne \text{EOF} \text{ after } \text{ModuleClosingBrace}$$
- **Erroneous Example:**
  ```orange
  edition 2026;
  module a { }
  module b { } // Second module in same file
  ```
- **Remediation:** Place each module in its own dedicated source file.

#### `ORC0105` — `TooManySyntaxErrors`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  A 101st ordinary parse error. Suppression is `ORC0105` after 100.
- **Remediation:** Resolve initial syntactic failures.

#### `ORC0106` — `ParserResourceLimit`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  One of: 262,144 syntax nodes, 1,048,576 parser events, recovery delimiter depth 64. Expression nesting is 64 and expression height is 256.
- **Remediation:** Simplify deeply nested parenthesized expressions.

#### `ORC0107` — `InvalidParserInput`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{TokenSourceId} \ne \text{ParserSourceId}$$
- **Remediation:** Internal compiler bug; report issue.

#### `ORC0108` — `UngroupedOperators`
- **Subsystem:** Parser
- **Formal Trigger Predicate:**
  $$\text{Trigger}(op_1, op_2) \iff \text{Group}(op_1) \ne \text{Group}(op_2) \land \neg \text{Parenthesized}(op_1, op_2)$$
- **Rationale:** Eliminates cryptographic vulnerabilities arising from unexpected operator precedence.
- **Erroneous Example:**
  ```orange
  spec f(a: Word[32], b: Word[32], c: Word[32]) -> Word[32] {
      a & b ^ c
  }
  spec g(x: Word[32]) -> Word[32] {
      x << 2 >> 1
  }
  ```
- **Remediation:** Enclose distinct operator groups in explicit parentheses:
  ```orange
  spec f(a: Word[32], b: Word[32], c: Word[32]) -> Word[32] {
      (a & b) ^ c
  }
  spec g(x: Word[32]) -> Word[32] {
      (x << 2) >> 1
  }
  ```

---

### §96. Semantic & Type Diagnostics (`ORC0201`–`ORC0242`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0201` — `DuplicateFunction`
- **Subsystem:** Semantic Analyzer (Symbol Table Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, M) \iff \exists g \in \text{Declarations}(M).\ f \ne g \land \text{Stratum}(f) = \text{Stratum}(g) \land \text{Ident}(f) = \text{Ident}(g)$$
- **Theoretical Rationale:** Function overloading within a single semantic stratum introduces
  call-graph ambiguities and complicates automated theorem proving. Every symbol in a stratum
  must have a unique canonical denotation.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module crypto {
      spec process(x: Word[32]) -> Word[32] { x + 1 }
      spec process(x: Word[64]) -> Word[64] { x + 1 } // Duplicate identifier
  }
  ```
- **Remediation:** Disambiguate by assigning distinct semantic function identifiers:
  ```orange
  edition 2026;
  module crypto {
      spec process32(x: Word[32]) -> Word[32] { x + 1 }
      spec process64(x: Word[64]) -> Word[64] { x + 1 }
  }
  ```

#### `ORC0202` — `UnsupportedTypedFunction`
- **Subsystem:** Semantic Analyzer (Slice Capability Gate)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \mathcal{S}) \iff \text{Stratum}(f) \ne \texttt{spec} \land \text{HasTypedBody}(f) \land \text{ActiveSlice}(\mathcal{S}) \prec \text{SliceRequired}(f)$$
- **Theoretical Rationale:** Language slices (S1 through S3t) gate feature stabilization.
  Declaring typed imperative `impl` or `machine impl` procedure bodies when targeting pure
  specification slices prevents undefined evaluation behavior.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module test_impl {
      impl compute(x: Word[32]) -> Word[32] { x + 1 }
  }
  ```
- **Remediation:** Use `spec` for pure mathematical specifications in slice S3t:
  ```orange
  edition 2026;
  module test_impl {
      spec compute(x: Word[32]) -> Word[32] { x + 1 }
  }
  ```

#### `ORC0203` — `UnsupportedType`
- **Subsystem:** Semantic Analyzer (Type Elaborator)
- **Formal Trigger Predicate:**
  The type is not `Int`, `Bool`, `Word[8|16|32|64]`, `Mod[m]`, an array of those, a tuple of those, or an earlier `type` name. `Byte` is not in the set (`ORC0203`).
- **Theoretical Rationale:** IEEE-754 floating-point numbers, unbounded dynamic pointers,
  and recursive algebraic data types introduce nondeterministic rounding, platform divergence,
  and side-channel leakages. They are strictly excluded from Orange's type universe.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_type {
      spec float_op(x: Float64) -> Float64 { x }
  }
  ```
- **Remediation:** Model computations using fixed-width words, modular rings, or exact integers:
  ```orange
  edition 2026;
  module bad_type {
      spec word_op(x: Word[64]) -> Word[64] { x }
  }
  ```

#### `ORC0204` — `UnsupportedWordWidth`
- **Subsystem:** Semantic Analyzer (Type Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(W) \iff W \notin \{8, 16, 32, 64\}$$
- **Theoretical Rationale:** Word arithmetic is grounded in physical machine register rings
  $\mathbb{Z}/2^W\mathbb{Z}$. Arbitrary-width words (e.g. 24-bit or 48-bit) cannot execute
  with constant-time hardware guarantees without non-standard masking.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module word_width {
      spec bad_word(x: Word[24]) -> Word[24] { x }
  }
  ```
- **Remediation:** Use standard machine word widths ($8, 16, 32, 64$):
  ```orange
  edition 2026;
  module word_width {
      spec good_word(x: Word[32]) -> Word[32] { x }
  }
  ```

#### `ORC0205` — `IntegerMagnitudeLimit`
- **Subsystem:** Semantic Analyzer (Constant Evaluator)
- **Formal Trigger Predicate:**
  An exact integer needs more than 16,384 significant bits.
- **Trigger:** more than 16,384 significant bits (`MAX_INTEGER_BITS`).
- **Not this diagnostic:** `spec huge() -> Int { 1 << 4096 }` is `ORC0215`, because `<<` is not defined on `Int`.
- **Remediation:** keep the integer within 16,384 significant bits.

#### `ORC0206` — `NegativeWordLiteral`
- **Subsystem:** Semantic Analyzer (Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(lit, \tau) \iff \tau = \text{Word}[W] \land lit < 0$$
- **Theoretical Rationale:** Word elements belong to $\mathbb{Z}/2^W\mathbb{Z}$ and are
  canonically represented by residues $r \in [0, 2^W - 1]$. Negative literal syntax obscures
  ring semantics; negation must be expressed via explicit arithmetic subtraction.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module neg_word {
      spec mask() -> Word[8] { -1 }
  }
  ```
- **Remediation:** Use canonical unsigned hex literals or ring subtraction:
  ```orange
  edition 2026;
  module neg_word {
      spec mask() -> Word[8] { 0xff }
  }
  ```

#### `ORC0207` — `WordLiteralOutOfRange`
- **Subsystem:** Semantic Analyzer (Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(lit, \tau) \iff (\tau = \text{Word}[W] \land lit \ge 2^W) \lor (\tau = \text{Mod}[m] \land lit \ge m)$$
- **Theoretical Rationale:** Prevents silent truncation bugs where developers accidentally
  specify a literal exceeding the ring or residue modulus.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module out_of_range {
      spec b() -> Word[8] { 256 }
      spec m() -> Mod[17] { 17 }
  }
  ```
- **Remediation:** Specify canonical residues in $[0, 2^W - 1]$ or $[0, m - 1]$:
  ```orange
  edition 2026;
  module out_of_range {
      spec b() -> Word[8] { 255 }
      spec m() -> Mod[17] { 16 }
  }
  ```

#### `ORC0208` — `TooManySemanticErrors`
- **Subsystem:** Semantic Analyzer (Error Recovery Driver)
- **Formal Trigger Predicate:**
  A 101st ordinary semantic error. Suppression is `ORC0208` after 100.
- **Theoretical Rationale:** Bounds diagnostic output to prevent terminal flooding and
  cascading nonsensical error reports.
- **Remediation:** Resolve initial semantic typing failures and re-run `orangec check`.

#### `ORC0209` — `SemanticResourceLimit`
- **Subsystem:** Semantic Analyzer (Memory Monitor)
- **Formal Trigger Predicate:**
  The semantic event budget (1,048,576) or the Core node budget (262,144) is exhausted, or a bounded allocation fails. There is no type-inference depth of 1,024 and no 64 MiB type table.
- **Theoretical Rationale:** Protects the compiler against pathological type structures
  and cyclic elaboration attacks.
- **Remediation:** Simplify deeply nested tuple expressions or split huge modules.

#### `ORC0210` — `InvalidSemanticInput`
- **Subsystem:** Semantic Analyzer (AST Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{ASTSourceFileId} \ne \text{ContextSourceFileId}$$
- **Theoretical Rationale:** Internal assertion failure verifying AST integrity between
  parser output and type checker input.
- **Remediation:** Internal compiler consistency defect; recompile or report issue.

#### `ORC0211` — `UnknownParameter`
- **Subsystem:** Semantic Analyzer (Scope Resolution)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(x, \Gamma, \Sigma) \iff x \notin \text{dom}(\Gamma) \land x \notin \text{dom}(\Sigma)$$
- **Theoretical Rationale:** Orange requires explicit lexical declarations for all identifiers.
  Ambient global variables do not exist.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module scoping {
      spec add_one(x: Word[32]) -> Word[32] { x + y }
  }
  ```
- **Remediation:** Bind `y` as a function parameter or in a local `let` binding:
  ```orange
  edition 2026;
  module scoping {
      spec add_one(x: Word[32], y: Word[32]) -> Word[32] { x + y }
  }
  ```

#### `ORC0212` — `UnknownFunction`
- **Subsystem:** Semantic Analyzer (Call Graph Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \Sigma) \iff f \notin \text{dom}(\Sigma)$$
- **Theoretical Rationale:** Function invocations must resolve to statically known,
  typed signatures in the current module or imported dependencies.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module calls {
      spec test_call() -> Word[32] { helper(42) }
  }
  ```
- **Remediation:** Define `spec helper(...)` or import its declaring module using `use`.

#### `ORC0213` — `ArgumentCountMismatch`
- **Subsystem:** Semantic Analyzer (Call Site Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \text{args}) \iff |\text{args}| \ne \text{ParamCount}(\Sigma(f))$$
- **Theoretical Rationale:** Functions have fixed arity; variadic arguments and default
  parameters are excluded to ensure deterministic call sequences.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module arity {
      spec add(a: Int, b: Int) -> Int { a + b }
      spec invoke() -> Int { add(1) }
  }
  ```
- **Remediation:** Supply all required arguments: `add(1, 2)`.

#### `ORC0214` — `TypeMismatch`
- **Subsystem:** Semantic Analyzer (Unification Engine)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\tau_{\text{actual}}, \tau_{\text{expected}}) \iff \tau_{\text{actual}} \not\equiv \tau_{\text{expected}}$$
- **Theoretical Rationale:** Orange forbids implicit type coercions and subtyping widening.
  All type mismatches indicate conceptual or structural bugs.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module mismatch {
      spec assign() -> Word[32] {
          let x: Word[32] = 42 as Word[8];
          x
      }
  }
  ```
- **Remediation:** Use explicit conversion to the expected type: `42 as Word[32]`.

#### `ORC0215` — `UnsupportedOperator`
- **Subsystem:** Semantic Analyzer (Operator Resolution)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(op, \tau) \iff op \notin \text{AdmittedOperators}(\tau)$$
- **Theoretical Rationale:** Prevents applying bitwise operations to non-ring types
  (such as `Int` or `Bool`) or applying arithmetic operators to tuples.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_op {
      spec int_xor(a: Int, b: Int) -> Int { a ^ b }
  }
  ```
- **Remediation:** Convert integer operands to fixed-width words before bitwise operations:
  ```orange
  edition 2026;
  module bad_op {
      spec word_xor(a: Word[32], b: Word[32]) -> Word[32] { a ^ b }
  }
  ```

#### `ORC0216` — `InvalidShiftAmount`
- **Subsystem:** Semantic Analyzer (Shift Verification)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(k, W) \iff k \in \text{Literals} \land (k < 0 \lor k \ge W)$$
- **Theoretical Rationale:** Literal shifts of $k \ge W$ or $k < 0$ produce undefined
  or non-portable results across target architectures (e.g. x86 masks shift counts by 31/63
  while ARM masks by 255).
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_shift {
      spec rot(x: Word[32]) -> Word[32] { x <<< 32 }
  }
  ```
- **Remediation:** Specify a literal shift amount $0 \le k < W$:
  ```orange
  edition 2026;
  module bad_shift {
      spec rot(x: Word[32]) -> Word[32] { x <<< 16 }
  }
  ```

#### `ORC0217` — `CallCycle`
- **Subsystem:** Semantic Analyzer (Call Graph DAG Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(G_{\text{call}}) \iff \exists f.\ f \to^+ f \text{ in } G_{\text{call}}$$
- **Theoretical Rationale:** Strongly normalizes the specification stratum. Banning recursive
  cycles guarantees that all specification functions terminate in bounded time.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module cycle {
      spec f(x: Int) -> Int { g(x) }
      spec g(x: Int) -> Int { f(x) }
  }
  ```
- **Remediation:** Eliminate recursion; formulate algorithms using bounded `for` loops.

#### `ORC0218` — `DuplicateParameter`
- **Subsystem:** Semantic Analyzer (Parameter Binder)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f) \iff \exists i \ne j.\ \text{ParamName}_i(f) = \text{ParamName}_j(f)$$
- **Theoretical Rationale:** Prevents lexical shadowing within the procedure's parameter
  environment.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module dup_param {
      spec f(x: Word[32], x: Word[32]) -> Word[32] { x }
  }
  ```
- **Remediation:** Provide unique identifiers for each parameter: `spec f(a: Word[32], b: Word[32])`.

#### `ORC0219` — `DuplicateBinding`
- **Subsystem:** Semantic Analyzer (Scope Resolution)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(x, \Gamma) \iff x \in \text{dom}(\Gamma)$$
- **Theoretical Rationale:** Variable shadowing in cryptographic kernels routinely conceals
  accidental overwrite bugs (e.g. shadowing an accumulator variable inside a loop).
- **Erroneous Example:**
  ```orange
  edition 2026;
  module shadow {
      spec f(x: Word[32]) -> Word[32] {
          let x: Word[32] = 10;
          x
      }
  }
  ```
- **Remediation:** Assign a distinct variable name: `let x1: Word[32] = 10;`.

#### `ORC0220` — `UntypedConversionOperand`
- **Subsystem:** Semantic Analyzer (Cast Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(e) \iff e \text{ as } \tau \land e \text{ is an untyped literal}$$
- **Theoretical Rationale:** Untyped integer literals must be bound to a known type before
  undergoing conversion, avoiding ambiguous conversion paths.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module untyped_cast {
      spec bad() -> Word[32] { (42) as Word[32] }
  }
  ```
- **Remediation:** Bind to a typed local variable or use typed literal syntax:
  ```orange
  edition 2026;
  module untyped_cast {
      spec good() -> Word[32] {
          let x: Word[32] = 42;
          x
      }
  }
  ```

#### `ORC0221` — `UnsupportedArrayLength`
- **Subsystem:** Semantic Analyzer (Array Type Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(n) \iff n < 1 \lor n > 65,536$$
- **Theoretical Rationale:** Zero-length arrays introduce degenerate algebraic properties
  and indexing ambiguities. Upper bounding arrays at $65,536$ elements ensures that stack
  allocation bounds $\text{StackFrameSize} \le 64\text{ KiB}$ are preserved by construction.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_array {
      type Empty = Word[8]^0;      // Zero length is prohibited
      type Massive = Word[32]^70000; // Exceeds 65,536 ceiling
  }
  ```
- **Remediation:** Declare array dimensions within the closed interval $[1, 65536]$:
  ```orange
  edition 2026;
  module bad_array {
      type Valid = Word[8]^64;
  }
  ```

#### `ORC0222` — `ArrayLengthMismatch`
- **Subsystem:** Semantic Analyzer (Literal Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(lit, n) \iff lit = [e_0, \dots, e_{k-1}] \land k \ne n$$
- **Theoretical Rationale:** Array literals are statically sized. Initializer elements must
  correspond bijectively to declared vector slots to prevent uninitialized memory slots.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module arr_len {
      spec get_iv() -> Word[32]^4 {
          [0x10, 0x20, 0x30] // Only 3 elements supplied for length 4
      }
  }
  ```
- **Remediation:** Provide exactly $n$ elements, or use the fill expression `[val; n]`:
  ```orange
  edition 2026;
  module arr_len {
      spec get_iv() -> Word[32]^4 {
          [0x10, 0x20, 0x30, 0x40]
      }
  }
  ```

#### `ORC0223` — `IndexOutOfRange`
- **Subsystem:** Semantic Analyzer (Static Interval Bounds Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(i, n) \iff \mathcal{I}(i) = [l, u] \land (l < 0 \lor u \ge n)$$
- **Theoretical Rationale:** Memory safety is mathematically verified at compile time.
  Any index whose evaluated interval cannot be proved strictly within $[0, n-1]$ is rejected.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bounds {
      spec pick(arr: Word[8]^16) -> Word[8] {
          arr[16] // Index 16 out of bounds for array of length 16
      }
  }
  ```
- **Remediation:** Use indices within $[0, n-1]$:
  ```orange
  edition 2026;
  module bounds {
      spec pick(arr: Word[8]^16) -> Word[8] {
          arr[15]
      }
  }
  ```

#### `ORC0224` — `NotAnArray`
- **Subsystem:** Semantic Analyzer (Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(X) \iff \text{Type}(X) \not\equiv T^n$$
- **Theoretical Rationale:** Bracket indexing `X[i]` is defined exclusively over fixed-length
  Cartesian power array types. Applying brackets to scalar words or tuples is prohibited.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module not_arr {
      spec index_word(w: Word[32]) -> Word[8] {
          w[0] // Cannot index directly into scalar Word[32]
      }
  }
  ```
- **Remediation:** Convert word to a byte array before indexing:
  ```orange
  edition 2026;
  module not_arr {
      spec index_word(w: Word[32]) -> Word[8] {
          let bytes: Word[8]^4 = w as big Word[8]^4;
          bytes[0]
      }
  }
  ```

#### `ORC0225` — `InvalidLoopRange`
- **Subsystem:** Semantic Analyzer (Loop Verification)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\text{low}, \text{high}) \iff \text{low} < 0 \lor \text{low} > \text{high} \lor \text{high} > 65,536$$
- **Theoretical Rationale:** Loop bounds in `spec` must be compile-time verifiable, finite,
  non-negative, and monotonically non-decreasing to guarantee bounded normalization.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_loop {
      spec rev() -> Word[32] {
          for i in 10..5 with acc = 0 { acc + i } // Decreasing range prohibited
      }
  }
  ```
- **Remediation:** Ensure bounds satisfy $0 \le \text{low} \le \text{high} \le 65,536$:
  ```orange
  edition 2026;
  module bad_loop {
      spec rev() -> Word[32] {
          for i in 5..10 with acc = 0 { acc + i }
      }
  }
  ```

#### `ORC0226` — `NonStaticIndex`
- **Subsystem:** Semantic Analyzer (Slice S1/S2 Enforcer)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(i, \mathcal{S}) \iff \text{ActiveSlice}(\mathcal{S}) \prec \text{S3g} \land \neg \text{IsCompileTimeConstant}(i)$$
- **Theoretical Rationale:** Data-dependent indexing is restricted to slices with formal
  narrowing (S3g+). In earlier slices, indices must be static compile-time constants.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module static_idx {
      spec lookup(table: Word[8]^16, dynamic_i: Int) -> Word[8] {
          table[dynamic_i]
      }
  }
  ```
- **Remediation:** In S3g+, use data-dependent lookups with typed word indices narrowed to bounds.

#### `ORC0227` — `UntypedComparison`
- **Subsystem:** Semantic Analyzer (Relational Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(a \mathbin{\text{cmp}} b) \iff \text{IsUntypedLit}(a) \land \text{IsUntypedLit}(b)$$
- **Theoretical Rationale:** Relational comparisons require ground types to establish whether
  comparison follows algebraic integer order ($\mathbb{Z}$), word ring order ($\mathbb{Z}/2^W\mathbb{Z}$),
  or modular residue order.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module cmp_err {
      spec test_cmp() -> Bool { (10 == 20) } // Both operands untyped literals
  }
  ```
- **Remediation:** Bind operands to typed variables before comparing:
  ```orange
  edition 2026;
  module cmp_err {
      spec test_cmp() -> Bool {
          let a: Int = 10;
          let b: Int = 20;
          a == b
      }
  }
  ```

#### `ORC0228` — `UnknownModule`
- **Subsystem:** Semantic Analyzer (Module Import Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m) \iff \neg \text{FileExists}(m \mathbin{\Vert} \text{".or"})$$
- **Theoretical Rationale:** Hermetic module resolution requires that every imported module
  corresponds to a deterministically discoverable source file on disk.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module app {
      use missing_module; // File missing_module.or does not exist
  }
  ```
- **Remediation:** Put `missing_module.or` in the root file's directory. There is no package-root file.

#### `ORC0229` — `ModuleNotUsed`
- **Subsystem:** Semantic Analyzer (Qualified Name Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m, \Sigma) \iff m\text{::}f \in \text{References} \land \texttt{use } m \notin \text{Imports}$$
- **Theoretical Rationale:** Prevents ambient resolution of undeclared dependencies. All module
  dependencies must be explicitly imported at the top of the compilation unit.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module client {
      spec call_external() -> Word[32] {
          sha256::hash_word(0) // 'use sha256;' was not declared
      }
  }
  ```
- **Remediation:** Add `use sha256;` to the module's import header.

#### `ORC0230` — `ModuleCycle`
- **Subsystem:** Semantic analysis (`semantics/linking.rs` depth-first search from the root)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(G_{\text{mod}}) \iff \exists m.\ m \to^+ m \text{ in } G_{\text{mod}}$$
- **Theoretical Rationale:** Cyclically dependent modules create mutually referential
  compilation environments and preclude deterministic, topological order compilation.
- **Erroneous Example:**
  ```text
  // File a.or
  edition 2026;
  module a { use b; }

  // File b.or
  edition 2026;
  module b { use a; }
  ```
- **Remediation:** Factor shared declarations into a leaf module `c.or` imported by both `a` and `b`.

#### `ORC0231` — `DuplicateModule`
- **Subsystem:** Semantic Analyzer (Module Header Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m) \iff \text{Count}(\text{Imports}(m)) > 1 \lor \text{DuplicateModuleSource}(m)$$
- **Theoretical Rationale:** Modules must have distinct names across a program, and a module
  cannot be imported redundantly within a single file.
- **Remediation:** Remove duplicate `use` statements or rename colliding modules.

#### `ORC0232` — `InvalidModulus`
- **Subsystem:** Semantic Analyzer (Residue Field Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(m) \iff m < 2 \lor m > 2^{521} - 1$$
- **Theoretical Rationale:** Moduli $< 2$ do not form residue rings or fields. The upper bound
  $2^{521} - 1$ admits NIST P-521 while maintaining bounded arithmetic complexity.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_mod {
      type Degenerate = Mod[1]; // Modulus 1 is algebraically trivial
  }
  ```
- **Remediation:** Specify an admitted modulus $2 \le m \le 2^{521} - 1$.

#### `ORC0233` — `DuplicateTypeName`
- **Subsystem:** Semantic Analyzer (Type Alias Resolver)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(T) \iff T \in \text{BuiltinTypes} \lor T \in \text{dom}(\text{DeclaredAliases})$$
- **Theoretical Rationale:** Prevents shadowing of builtin primitive types (`Int`, `Bool`, `Word`)
  and rejects re-declarations of type aliases in the same module.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module dup_type {
      type Int = Word[64]; // Cannot redefine builtin type Int
  }
  ```
- **Remediation:** Choose an unreserved, unique type alias name.

#### `ORC0234` — `NotATuple`
- **Subsystem:** Semantic Analyzer (Projection Type Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(t) \iff \text{Type}(t) \not\equiv (T_0, \dots, T_{k-1})$$
- **Theoretical Rationale:** Dot-index field projections (`t.0`, `t.1`) are valid exclusively
  on heterogeneous product tuple types.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module tuple_err {
      spec proj(w: Word[32]) -> Word[32] {
          w.0 // Cannot apply tuple projection to word
      }
  }
  ```
- **Remediation:** Apply `.j` projections only to tuple values: `(x, y).0`.

#### `ORC0235` — `UnprintableByteString`
- **Subsystem:** Lexical & Semantic String Checker
- **Formal Trigger Predicate:**
  $$\text{Trigger}(s) \iff \exists c \in \text{Chars}(s).\ c < 0\text{x}20 \lor c > 0\text{x}7E$$
- **Theoretical Rationale:** Prevents non-printable and invisible Unicode characters from
  obfuscating cryptographic test titles or string contents.
- **Remediation:** Use `\xNN` escape sequences or `hex"..."` literals.

#### `ORC0236` — `SliceLength`
- **Subsystem:** Semantic Analyzer (Slice Bounds Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(a, b, n) \iff a > b \lor b > n \lor a < 0$$
- **Theoretical Rationale:** Array slice ranges $a..b$ must define valid, monotonically
  non-decreasing sub-intervals within the parent array bounds.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module slice_err {
      spec bad_slice(arr: Word[8]^16) -> Word[8]^8 {
          arr[10..5] // Inverted slice bounds
      }
  }
  ```
- **Remediation:** Use a positive constant length in range, as in `arr[5..10]`. An empty slice (`a == b`) is `ORC0236`.

#### `ORC0237` — `NonStaticSize`
- **Subsystem:** Semantic Analyzer (Size Parameter Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(sz) \iff \neg \text{IsCompileTimeConstant}(sz)$$
- **Theoretical Rationale:** Array dimensions and finite size arguments must evaluate to
  static constants at compile time to support monomorphization.
- **Remediation:** Supply an integer literal or compile-time constant size expression.

#### `ORC0238` — `SizeRange`
- **Subsystem:** Semantic Analyzer (Monomorphization Engine)
- **Formal Trigger Predicate:**
  `n in a..b` takes each integer from `a` up to but not including `b`. A size outside that half-open interval is `ORC0238`. For `n in 1..64`, `pad[64]` and `pad[70]` are both outside; `pad[63]` is inside.

#### `ORC0239` — `SizeCount`
- **Subsystem:** Semantic Analyzer (Call Site Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f, \text{sizes}) \iff |\text{sizes}| \ne \text{DeclaredSizeParamCount}(f)$$
- **Theoretical Rationale:** Procedures expect an exact number of size parameters.
- **Remediation:** Supply the exact number of size parameters declared in the signature.

#### `ORC0240` — `PackedWidth`
- **Subsystem:** Semantic Analyzer (Endian Packing Checker)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\tau_1, \tau_2) \iff \text{TotalBits}(\tau_1) \ne \text{TotalBits}(\tau_2)$$
- **Theoretical Rationale:** Bit-preserving packing (`as big T`, `as little T`) requires
  that total bit widths match exactly: $\text{bits}(\tau_1) = \text{bits}(\tau_2)$.
  Packing cannot synthesize or drop bits.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module packed_err {
      spec pack(bytes: Word[8]^15) -> Word[32]^4 {
          bytes as big Word[32]^4 // 120 bits != 128 bits
      }
  }
  ```
- **Remediation:** Ensure both source and target types have identical total bit widths:
  ```orange
  edition 2026;
  module packed_err {
      spec pack(bytes: Word[8]^16) -> Word[32]^4 {
          bytes as big Word[32]^4
      }
  }
  ```

#### `ORC0241` — `TypeParameter`
- **Subsystem:** Semantic Analyzer (Finite Type Parameter Elaborator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(T, \mathcal{S}) \iff T \notin \mathcal{S} \lor \text{DuplicatesInSet}(\mathcal{S})$$
- **Theoretical Rationale:** Type parameters are restricted to finite, explicitly enumerated
  sets $\{T_1, \dots, T_m\}$ to ensure exhaustive monomorphization. Instantiating with an
  unlisted type is rejected.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module type_param {
      spec square[K in {Mod[17], Mod[31]}](x: K) -> K { x * x }
      spec call_sq() -> Int {
          square[Int](42) // 'Int' is not in the parameter set
      }
  }
  ```
- **Remediation:** Pass a type in the list. `square[Mod[17]](x)` is admitted when `Mod[17]` is listed and `x` has that type. `square[Int]` is not.

#### `ORC0242` — `TestTitle`
- **Subsystem:** Semantic Analyzer (Test Validator)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(title) \iff |title| = 0 \lor |title| > 128 \lor \text{DuplicateTestTitle}(title)$$
- **Theoretical Rationale:** Test titles identify verification test vectors in evidence bundles
  and TAP reports. They must be unique, non-empty, and under 128 characters.
- **Remediation:** Provide a unique ASCII title between 1 and 128 characters.

---

### §97. Formatter Diagnostics (`ORC0250`–`ORC0252`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0250` — `FormatResourceLimit`
- **Subsystem:** Syntax Formatter (`orangec fmt`)
- **Formal Trigger Predicate:**
  The formatter event budget is 1,048,576 (`4 * 262,144`).
- **Theoretical Rationale:** Prevents DOS during automated source formatting.
- **Remediation:** Partition large compilation units into modular files under 16 MiB.

#### `ORC0251` — `FormattingInconsistency`
- **Subsystem:** Syntax Formatter (`orangec fmt`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{TokenStream}(\text{fmt}(\text{src})) \ne \text{TokenStream}(\text{src})$$
- **Theoretical Rationale:** Formatter invariant assertion: formatting MUST preserve the exact
  non-trivia token stream. If the AST changes, formatting halts immediately.
- **Remediation:** Internal compiler fault; report issue.

#### `ORC0252` — `FormattingRequired`
- **Subsystem:** Syntax Formatter (`orangec fmt --check`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(\text{src}) \iff \text{SourceBytes}(\text{src}) \ne \text{FormattedBytes}(\text{src})$$
- **Theoretical Rationale:** Enforces canonical source formatting in CI pipelines.
- **Erroneous Example:** Unformatted source text evaluated under `--check`.
- **Remediation:** Run `orangec fmt FILE` to canonically reformat whitespace.

---

### §98. Documentation Diagnostics (`ORC0260`–`ORC0261`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0260` — `DocumentationResourceLimit`
- **Subsystem:** Offline Documentation Generator (`orangec doc`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{DocItems} > 1,048,576 \lor \text{GeneratedBytes} > 16,777,216$$
- **Theoretical Rationale:** Limits memory footprint during offline single-page HTML documentation.
- **Remediation:** Split package across multiple smaller modules.

#### `ORC0261` — `DocumentationInconsistency`
- **Subsystem:** Offline Documentation Generator (`orangec doc`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{DuplicateAnchorId}() \lor \text{MalformedDocSpan}()$$
- **Theoretical Rationale:** Generated documentation anchors must correspond bijectively
  to verified AST declaration nodes.
- **Remediation:** Check for conflicting declaration identifiers.

---

### §99. Witness Replay Diagnostics (`ORC0270`–`ORC0274`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0270` — `NoncanonicalArgumentValue`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(arg) \iff \neg \text{IsCanonicalDecimalOrHex}(arg)$$
- **Theoretical Rationale:** Counterexample replay inputs must use canonical decimal or
  lowercase hex encodings to ensure cross-platform reproducibility.
- **Remediation:** Format argument vector entries using canonical decimal or lowercase hex (`0x...`).

#### `ORC0271` — `ArgumentValueMismatch`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(args, f) \iff \text{Shape}(args) \ne \text{ParamShape}(\Sigma(f))$$
- **Theoretical Rationale:** Replay arguments must match the exact parameter types and array
  lengths of the target boolean specification function.
- **Remediation:** Supply an argument vector matching the function's parameter signature.

#### `ORC0272` — `ArgumentDecodeResourceLimit`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  Argument decoding exceeds `MAX_ARGUMENT_VALUE_NODES` (16 × 262,144), `MAX_ARGUMENT_INTEGER_LIMBS` (4 × 1,048,576), or `MAX_ARGUMENT_DECODE_WORK` (64 × 1,048,576) in `arguments.rs`.
- **Theoretical Rationale:** Bounds memory allocated when decoding counterexample witness vectors.
- **Remediation:** Partition large witness inputs.

#### `ORC0273` — `InvalidWitnessReplayBinding`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(f) \iff \text{ReturnType}(f) \ne \text{Bool}$$
- **Theoretical Rationale:** Witness replay checks whether a property holds or is falsified.
  The target specification function MUST return `Bool`.
- **Erroneous Example:**
  ```orange
  edition 2026;
  module bad_replay {
      spec hash(x: Word[32]) -> Word[32] { x } // Returns Word[32], not Bool
  }
  ```
- **Remediation:** Target a boolean predicate function `spec prop(...) -> Bool`.

#### `ORC0274` — `WitnessReplayInconsistency`
- **Subsystem:** Witness Replay Driver (`orangec replay`)
- **Formal Trigger Predicate:**
  $$\text{Trigger} \iff \text{InternalReplayBindingFailure}()$$
- **Theoretical Rationale:** Internal assertion failure during witness argument binding.
- **Remediation:** Internal compiler bug; report issue.

---

### §100. Evaluator & Resource Diagnostics (`ORC0301`): Formal Predicates, Triggers, Examples, Fixes

#### `ORC0301` — `EvaluationResourceLimit`
- **Subsystem:** Reference Evaluator (`orangec eval`)
- **Formal Trigger Predicate:**
  $$\text{Trigger}(K) \iff K \le 0 \quad (\text{where step counter starts at } K_0 = 1,048,576 \text{ or } \texttt{--steps})$$
- **Theoretical Rationale:** Guarantees that evaluation halts deterministically even when
  evaluating computationally intensive algorithms, defending against infinite evaluation loops.
- **Erroneous Example:**
  ```console
  $ orangec eval --steps 100 sha256.or
  error[ORC0301]: evaluation resource limit exceeded: step budget '100' exhausted
  ```
- **Remediation:** Increase step budget using `--steps <COUNT>`:
  ```console
  $ orangec eval --steps 2097152 sha256.or
  ```

---

## Part X: Toolchain, limits, and the implemented grammar

### §101. The driver `orangec`

Subject: the `orangec` 0.0.1 command line on the S3t revision.
Evidence: `orangec --help` and `orangec --version` from that binary. Appendix A of the Book restates the same commands.

```text
orangec 0.0.1 (Orange edition 2026; implemented slice S3t)
```

```text
Usage: orangec [OPTIONS] <check|eval|lex> <FILE>...
       orangec eval [--steps <N>] [--spec <NAME>]... [--stats] <FILE>
       orangec test [--steps <N>] [--stats] <FILE>
       orangec fmt <FILE>
       orangec fmt --check <FILE>...
       orangec doc <FILE>
       orangec replay --function <MODULE::NAME> [--instance <N[,N...]>]
                      --witness <FILE> [--steps <N>] [--stats] <SOURCE>
       orangec keygen [--scheme <NAME>] [-o <FILE>]
       orangec <enc|dec> [--key <FILE>] [--scheme <NAME>] [-o <FILE>] <FILE>
       orangec schemes [<NAME>...]
```

| Command | Behavior | Status |
| --- | --- | --- |
| `check` | Lex, parse, and semantic analysis. Silent on success. | 0, or 1 on a diagnostic, or 2 on usage |
| `eval` | Validate one program, then print each parameterless typed `spec` of the root, and each instance of a sized one | 0, or 1 on a diagnostic or resource limit |
| `lex` | Print the token stream with byte spans | 0 on success |
| `test` | Run the root module's tests in source order. The line is `test "TITLE" ... ok` or `... FAILED` | 1 when any test fails |
| `fmt` / `fmt --check` | Print one formatted source, or check 1 through 256 sources | `ORC0252` when `--check` requires formatting |
| `doc` | Standalone offline HTML for one parsed source | 0, or 1 on a documentation diagnostic |
| `replay` | Reference-evaluate one Boolean `spec` on exact typed arguments. The result line ends in `holds_for_this_witness` or `falsified` | 0 on a completed replay |
| `keygen`, `enc`, `dec`, `schemes` | Reference sealing tools. The help text says sealing is not constant-time and the schemes are not verified | usage as in `--help` |

`--stats` writes each evaluated function's or test's step count and the total to standard error. It does not report memory words. `--steps` is an integer from 1 through 1,073,741,824, default 1,048,576, at most once, on `eval`, `test`, and `replay`. `--spec` names up to 64 parameterless functions and applies to `eval` only. `--edition` is only `2026`. A file name `-` reads UTF-8 source from standard input once.

### §102. Resource limits

Numbers are the named constants in the compiler crates. A breach fails closed with the diagnostic in the last column.

| Limit | Constant | Breach |
| --- | --- | --- |
| Source file | 16 MiB, `MAX_SOURCE_BYTES` | `ORC1001` before lexing |
| Non-trivia tokens, excluding `EOF` | 262,144 | `ORC0006` |
| Ordinary diagnostics per phase | 100, then one suppression | `ORC0007`, `ORC0105`, `ORC0208` |
| Syntax nodes | 262,144 | `ORC0106` |
| Parser events | 1,048,576 | `ORC0106` |
| Recovery delimiter depth | 64 | `ORC0106` |
| Expression nesting / height | 64 / 256 | `ORC0106` |
| Significant bits of one `Int` | 16,384 | `ORC0205` |
| Word widths | 8, 16, 32, 64 | `ORC0204` |
| Modulus `m` | `2 <= m <= 2^521 - 1` | `ORC0232` |
| Array length, and scalar product of a rank-two array | 65,536 | `ORC0221` |
| Tuple width | 2 through 16 | rejected by the grammar or by `ORC0214` / the tuple rules; `ORC0234` is `.k` on a non-tuple, not an arity code |
| Modules in one program, root included | 64 | semantic resource diagnostic |
| `use` and `type` declarations in one module | 64 each | semantic resource diagnostic |
| Size parameters / bracket entries | 4 | `ORC0238` / `ORC0239` |
| Instances of one function | 256 | `ORC0238` |
| Parameters / bindings | 64 / 256 | semantic resource diagnostic |
| Call arguments | 256 | `ORC0213` when the count differs; the budget is the cap |
| Default evaluation steps | 1,048,576; `--steps` at most 1,073,741,824 | `ORC0301` |
| Core nodes / semantic events | 262,144 / 1,048,576 | `ORC0209` |
| Documentation events / HTML bytes | 1,048,576 / 16 MiB | `ORC0260` |
| Formatter events | 1,048,576 | `ORC0250` |

### §103. Implemented parser surface, restated

Subject: the parser implemented through S3t.
Assumption: S3b–S3t remain in review. This section does not accept them.
Evidence: Appendix A of [`THE_ORANGE_BOOK.md`](THE_ORANGE_BOOK.md), with the suffix production replaced by the S3s extension in [`NESTED_ARRAYS_2026.md`](NESTED_ARRAYS_2026.md). S3t adds no production ([`STATIC_MODULI_2026.md`](STATIC_MODULI_2026.md)). Accepted syntax is still the smaller grammar in [`LANGUAGE_2026.md`](LANGUAGE_2026.md) §3 and [`SEMANTICS_2026.md`](SEMANTICS_2026.md) §2.

The Book's appendix says the summary yields where a slice document differs. The only such difference applied here is:

```text
suffix = projection? index* slice? ;
```

A suffix contains at least one projection, index, or slice. A slice ends the suffix. Repeated `^` in one type is not a production: one `^` applies to a `parsed_type`, and a rank-two array is an alias of an array (§37).

```text
source_file     = edition_decl module_decl EOF ;
edition_decl    = "edition" "2026" ";" ;
module_decl     = "module" IDENTIFIER "{" use_decl* type_decl* member* "}" ;
member          = function_decl | test_decl ;
use_decl        = "use" IDENTIFIER ";" ;
type_decl       = "type" IDENTIFIER "=" declared_type ";" ;
function_decl   = "spec" IDENTIFIER "(" ")" spec_tail
                | "spec" IDENTIFIER size_params? "(" parameters? ")" typed_tail
                | "impl" IDENTIFIER "(" ")" empty_body ;
test_decl       = "test" STRING "{" binding* expression "}" ;
size_params     = "[" size_param ("," size_param)* "]" ;
size_param      = IDENTIFIER "in" (INTEGER ".." INTEGER | type_list) ;
type_list       = "{" declared_type ("," declared_type)* "}" ;
spec_tail       = empty_body | typed_tail ;
typed_tail      = "->" declared_type "{" binding* expression "}" ;
binding         = "let" pattern "=" expression ";" ;
pattern         = typed_name | "(" typed_name ("," typed_name)+ ","? ")" ;
typed_name      = IDENTIFIER ":" declared_type ;
empty_body      = "{" "}" ;
parameters      = parameter ("," parameter)* ","? ;
parameter       = IDENTIFIER ":" declared_type ;
declared_type   = element_type | tuple_type ;
tuple_type      = "(" element_type ("," element_type)+ ","? ")" ;
element_type    = parsed_type ("^" size)? ;
size            = INTEGER | IDENTIFIER | "(" expression ")" ;
parsed_type     = "Mod" "[" expression "]" | IDENTIFIER ("[" INTEGER "]")? ;

expression      = arithmetic | chain("&") | chain("|") | chain("^") | shift
                | comparison | chain("&&") | chain("||") | division
                | chain("++") | conversion | update ;
conversion      = prefixed "as" (parsed_type | tuple_type | order declared_type) ;
order           = "big" | "little" ;
update          = prefixed "with" "[" (expression | range) "]" "=" expression ;
arithmetic      = product (("+" | "-") product)* ;
product         = prefixed ("*" prefixed)* ;
chain(op)       = prefixed (op prefixed)+ ;
shift           = prefixed shift_operator prefixed ;
shift_operator  = "<<" | ">>" | "<<<" | ">>>" ;
comparison      = prefixed compare_op prefixed ;
compare_op      = "==" | "!=" | "<" | "<=" | ">" | ">=" ;
division        = prefixed ("/" | "%") prefixed ;
prefixed        = literal | ("-" | "~" | "!") prefixed | primary ;
literal         = "-"? INTEGER ;
primary         = IDENTIFIER suffix? | call suffix? | "(" expression ")"
                | byte_string | tuple | array | fill | loop | conditional ;
byte_string     = STRING | HEX_STRING ;
suffix          = projection? index* slice? ;
projection      = "." INTEGER ;
tuple           = "(" expression ("," expression)+ ","? ")" ;
index           = "[" INTEGER "]" | "[" expression "]" ;
slice           = "[" range "]" ;
range           = expression ".." expression? | ".." expression ;
array           = "[" expression ("," expression)* ","? "]" ;
fill            = "[" expression ";" size "]" ;
loop            = "for" IDENTIFIER "in" size ".." size
                  "with" pattern "=" expression block ;
conditional     = "if" expression block "else" (block | conditional) ;
block           = "{" binding* expression "}" ;
call            = (IDENTIFIER "::")? IDENTIFIER sizes? "(" arguments? ")" ;
sizes           = "[" expression ("," expression)* "]" ;
arguments       = expression ("," expression)* ","? ;
```

`game`, `proof`, and `claim` do not appear. `Byte` does not appear. `let` patterns carry types. Size parameters and type parameters share one bracket list (`size_param = IDENTIFIER "in" (INTEGER ".." INTEGER | type_list)`).
