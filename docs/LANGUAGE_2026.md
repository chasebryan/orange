# Orange 2026 lexical and grammar specification

Status: normative pre-alpha S2 syntax under D-025 and accepted OEP-0002,
additively extended with accepted S3a syntax under D-026 and OEP-0003

Edition: `2026`

Snapshot: 2026-07-26

This document defines the complete lexical and syntactic language accepted by
the Orange 2026 parser. It is intentionally small. Acceptance establishes only
that source text has this shape. The separate
[`SEMANTICS_2026.md`](SEMANTICS_2026.md) defines the exact subset that has type,
name-resolution, Core, and reference-evaluation meaning. Syntax acceptance by
itself does not establish any semantic, proof, compilation, cryptographic, or
other correctness property.

The S3a extension was merged by PR #9 at commit
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. This records the implemented
pre-alpha boundary, not a stable syntax-compatibility guarantee; later S3 work
remains incomplete. D-003 candidate PF-01 is accepted through OEP-0004 at exact
revision `a82a5cec2ee4359dc2fe66171f17c93146747333`, while D-004 remains
unresolved.

> [!NOTE]
> The compiler also implements the S3b expression grammar proposed in
> [`EXPRESSIONS_2026.md`](EXPRESSIONS_2026.md) under OEP-0005, which is in the
> owner's review and not accepted. It adds parameters, calls, prefix and binary
> operators, and the single tokens `<<`, `>>`, `<<<`, and `>>>`. Every source
> this document accepts is still accepted. Until OEP-0005 is accepted,
> this document remains the accepted syntax boundary; on acceptance, the
> clauses listed in section 16 of that proposal are replaced.
>
> The S3c slice proposed in [`BINDINGS_2026.md`](BINDINGS_2026.md) under
> OEP-0006, also in review, builds on S3b with `let` bindings and `as`
> conversions. It adds no token and no reserved word: `let` and `as` keep
> their meaning as identifiers everywhere except in the two positions it
> defines.
>
> The S3d slice proposed in [`ARRAYS_2026.md`](ARRAYS_2026.md) under OEP-0007,
> also in review, builds on S3c with fixed-length array types `T^n`, array
> literals, and literal indices. It also adds no token and no reserved word.
>
> The S3e slice proposed in [`LOOPS_2026.md`](LOOPS_2026.md) under OEP-0008,
> also in review, builds on S3d with bounded loops, computed indices, updates,
> and fill literals. It adds no token and no reserved word either: `for`, `in`,
> and `with` are recognized by position.
>
> The S3f slice proposed in [`CONDITIONS_2026.md`](CONDITIONS_2026.md) under
> OEP-0009, also in review, builds on S3e with `Bool`, comparisons, logical
> operators, Euclidean division, and conditionals. It gives meaning to tokens
> the lexer already produces, and it adds no token and no reserved word:
> `if` and `else` are recognized by position, and `true` and `false` by scope.
>
> The S3g slice proposed in [`LOOKUPS_2026.md`](LOOKUPS_2026.md) under
> OEP-0010, also in review, builds on S3f with indices that depend on data,
> each proved in range from its type, and cheaper updates. It changes no
> grammar.
>
> The S3h slice proposed in [`MODULES_2026.md`](MODULES_2026.md) under
> OEP-0011, also in review, builds on S3g with programs of more than one
> module: `use` declarations at the head of a module, and calls qualified by a
> module name, as in `sha256::compress(h, block)`. It adds no token and no
> reserved word: `use` is recognized by position, and `::` is the existing
> `DOUBLE_COLON` token.
>
> The S3i slice proposed in [`MODULAR_2026.md`](MODULAR_2026.md) under
> OEP-0012, also in review, builds on S3h with the type `Mod[m]` of the
> integers modulo a constant, whose modulus is an expression, as in
> `Mod[(1 << 255) - 19]`, and `type` declarations at the head of a module. It
> adds no token and no reserved word: `type` is recognized by position, and
> `Mod` is an ordinary type name that takes an expression in brackets.
>
> The S3j slice proposed in [`BLOCKS_2026.md`](BLOCKS_2026.md) under
> OEP-0013, also in review, builds on S3i with blocks: a loop's step and each
> branch of a conditional may begin with `let` bindings, as a function's body
> does. It adds no token and no reserved word: `let` is recognized by
> position, as in S3c.
>
> The S3k slice proposed in [`TUPLES_2026.md`](TUPLES_2026.md) under
> OEP-0014, also in review, builds on S3j with tuples: a tuple type `(T, U)`,
> a tuple `(a, b)`, the selection `.k` of element k, and tuple patterns that
> name each element where a binding or a loop's accumulator is declared. It
> adds no token and no reserved word.
>
> The S3l slice proposed in [`BYTES_2026.md`](BYTES_2026.md) under OEP-0015,
> also in review, builds on S3k with bytes: a string is the array of its
> bytes, `hex"..."` writes one in hex, `++` joins two arrays, and `x[a..b]`
> takes a run of elements at bounds proved in range. It adds two tokens,
> `HEX_STRING` and `PLUS_PLUS`, and no reserved word: `hex` is a name unless a
> quote follows it directly.
>
> The S3m slice proposed in [`SIZES_2026.md`](SIZES_2026.md) under OEP-0016,
> also in review, builds on S3l with sizes: a `spec` may declare size parameters
> with finite ranges, as `spec pad[len in 1..120](m: Word[8]^len)`, and stands
> for one function for each of their values, each checked as if written out. It
> adds no token and no reserved word: `in` is a name except between a size
> parameter's name and its first bound.
>
> The S3n slice proposed in [`ORDER_2026.md`](ORDER_2026.md) under OEP-0017,
> also in review, builds on S3m with byte orders: `x as big T` and
> `x as little T` read words as words of another width, as a number, or as a
> residue, and write a number as words, first word most significant or least.
> It adds no token and no reserved word: `big` and `little` are names except
> directly after `as` and before a type.
>
> The S3o slice proposed in
> [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md) under OEP-0018, also in
> review, builds on S3n with type parameters: a `spec` may list the types it is
> written for, as `spec pow[K in {F, P, Q}](x: K, e: Int) -> K`, and stands for
> one function for each, each checked as if written out; a call names its
> instance by its types, `pow[F](x, e)`, or lets its arguments' types and its
> place choose. It adds no token and no reserved word: braces after a size
> parameter's `in` hold a list of types.
>
> The S3p slice proposed in [`LENGTHS_2026.md`](LENGTHS_2026.md) under
> OEP-0019, also in review, builds on S3o with long arrays and evaluation
> controls: an array, an array literal, and a byte string hold up to 65,536
> elements, so a `Word[16]` indexes the longest with no check at run time, and
> `orangec eval --steps`, `--spec`, and `--stats` set a run's step budget,
> evaluate only the functions named, and report the steps each used. It adds
> no token and no reserved word.
>
> The S3q slice proposed in [`TESTS_2026.md`](TESTS_2026.md) under OEP-0020,
> also in review, builds on S3p with known-answer tests and whole-value
> equality: `test "TITLE" { claim }` may stand among a module's functions, its
> claim a `Bool` checked as a function without parameters, `==` and `!=`
> compare arrays and tuples whole, and `orangec test` runs the root module's
> tests and reports each. It adds no token and no reserved word: `test`
> followed by a string begins a test only where a module member may begin.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Source representation

An Orange 2026 source file is valid UTF-8 and is at most 16 MiB
(`16 * 1024 * 1024` bytes). A source larger than that limit is rejected before
lexing. Source spans are half-open UTF-8 byte ranges `[start, end)` tied to one
source identity.

The only whitespace characters are:

| Character | Code point | Name |
| --- | --- | --- |
| `\t` | U+0009 | horizontal tab |
| `\n` | U+000A | line feed |
| `\r` | U+000D | carriage return |
| space | U+0020 | space |

No other Unicode character is whitespace. In particular, non-breaking space,
line separator, paragraph separator, and other Unicode space characters are
lexical errors.

For source-location reporting, line feed, carriage-return line feed, and bare
carriage return each form exactly one logical line ending. A carriage-return
line-feed pair must not count as two lines. Columns are one-based Unicode-scalar
positions; spans remain byte based.

## 2. Lexical grammar

Lexing is deterministic and edition aware. The scanner uses longest-token
matching for the punctuation listed below. Every non-trivia token retains its
exact source span. Exactly one zero-width `EOF` token follows the final retained
token.

### 2.1 Trivia and comments

Whitespace is trivia. A line comment begins with `//` and continues until the
next logical line ending or end of input. The line ending is not part of the
comment.

A block comment begins with `/*` and ends with the matching `*/`. Block comments
may nest. An unclosed block comment is a lexical error. Comment delimiters have
no string-like escape syntax.

Trivia separates tokens but does not appear in the syntax tree for this slice.

### 2.2 Identifiers and reserved words

An identifier has the following ASCII-only form:

```text
identifier       = identifier_start identifier_continue* ;
identifier_start = "A".."Z" | "a".."z" | "_" ;
identifier_continue = identifier_start | "0".."9" ;
```

Unicode letters and digits do not participate in identifiers. The following
spellings are reserved words in Orange 2026 and never lex as identifiers:

```text
edition  module  spec  impl  game  proof  claim
```

Only `edition`, `module`, `spec`, and `impl` have a grammatical role in this
slice. `game`, `proof`, and `claim` are lexical reservations only: the parser
must reject them wherever this document requires an identifier or declaration.
Reservation assigns no semantics to those words.

Identifiers are compared as their exact ASCII spellings. This syntax slice
does not perform normalization, case folding, name binding, or duplicate-name
checking.

### 2.3 Integer tokens

Integer tokens are decimal by default. Prefixes `0b` or `0B` select base 2;
prefixes `0x` or `0X` select base 16. Hexadecimal digits may be uppercase or
lowercase. Each integer requires at least one digit after any prefix.

An underscore may appear only as one separator between two digits valid for the
selected base. A leading, trailing, doubled, or otherwise misplaced underscore
is malformed. A letter or digit consumed as part of a candidate integer but
invalid for the selected base makes that complete candidate malformed.

The exact `2026` token selects the mandatory source edition. Other integer
tokens may appear as a syntactic type-width argument or as the magnitude of a
typed `spec` literal. Parsing an integer assigns no numeric value, supported
width, type, or evaluation meaning; those rules are defined separately in
[`SEMANTICS_2026.md`](SEMANTICS_2026.md).

### 2.4 String tokens

A string begins and ends with `"` on one logical line. Its supported escapes
are `\"`, `\\`, `\n`, `\r`, `\t`, `\0`, and `\xNN`, where each `N` is one ASCII
hexadecimal digit. An unsupported or incomplete escape is a lexical error. An
unclosed string or a string that reaches any logical line ending is a lexical
error.

String tokens have no grammatical role in this slice. The lexer does not assign
an encoding or runtime meaning to their contents.

### 2.5 Punctuation tokens

The punctuation tokens are:

```text
(  )  {  }  [  ]  ,  :  ;  .  ..  ::
+  -  *  /  %  &  &&  |  ||  ^  ~  !
=  <  >  ==  !=  <=  >=  ->  =>  ?
```

The parser grammar below uses `(`, `)`, `{`, `}`, `[`, `]`, `;`, `-`, and `->`.
Every other punctuation token is a lexical reservation without syntax or
semantics in this slice. `-` is admitted only as the optional sign immediately
before a typed-body integer token; it is not a general unary operator.

### 2.6 Lexical limits and failures

At most 262,144 non-trivia tokens are retained for one source, excluding the
required `EOF` token. At most 100 ordinary lexical diagnostics are emitted,
followed when necessary by one stable suppression diagnostic. A token-limit
diagnostic is a resource diagnostic and must not be hidden by that ordinary
diagnostic budget.

Invalid UTF-8, an oversized source, an unexpected character, an unclosed block
comment or string, an invalid string escape, a malformed integer, or exhaustion
of a lexical resource budget makes the source lexically invalid. A lexically
invalid source must not be parsed.

## 3. Syntactic grammar

The Orange 2026 parser in this slice accepts exactly the following grammar:

```text
source_file   = edition_decl module_decl EOF ;
edition_decl  = "edition" "2026" ";" ;
module_decl   = "module" IDENTIFIER "{" function_decl* "}" ;
function_decl = "spec" IDENTIFIER "(" ")" spec_tail
              | "impl" IDENTIFIER "(" ")" empty_body ;
spec_tail     = empty_body
              | "->" parsed_type "{" signed_integer "}" ;
empty_body     = "{" "}" ;
parsed_type    = IDENTIFIER ("[" INTEGER "]")? ;
signed_integer = "-"? INTEGER ;
```

`"2026"` in `edition_decl` means the exact decimal integer-token spelling
`2026`. Prefixes, separators, leading zeroes, or another value do not select the
edition. The declaration is mandatory and must be first.

One source contains exactly one module. A module may contain zero or more
function declarations in source order. Every function has one kind, one
identifier, and an empty parameter list. Both `spec` and `impl` retain the
legacy empty body. Only `spec` may instead have one parsed result type and a
body containing exactly one optionally negative integer token. Trivia may occur
between tokens wherever token boundaries permit it.

`parsed_type` deliberately accepts any identifier and either no width or one
integer width. This is a syntactic container, not support for a named type,
generic arguments, or arbitrary word widths. The semantic specification
recognizes only its exact documented type forms. Similarly, `signed_integer`
does not introduce general expressions, operators, or arithmetic.

This grammar is LL(1): each declaration begins with `spec` or `impl`, `}`
terminates the declaration list, and `{` versus `->` selects a `spec` tail.
There is no precedence, implicit semicolon, contextual keyword, or grammar
ambiguity in this slice. In particular, `impl name() -> Type { 1 }` is a syntax
error rather than a typed implementation declaration.

The following source is accepted:

```orange
edition 2026;
module demo {
  spec identity() {}
  impl rounds() {}
  spec answer() -> Int { 42 }
  spec byte() -> Word[8] { 0xff }
}
```

## 4. Syntax-tree mapping

Every accepted source maps to one deterministic syntax tree containing:

- one source-file node;
- one edition-declaration node with the exact edition token;
- one module node with its identifier;
- one function node per declaration, in source order, with its `spec` or `impl`
  kind and identifier;
- one empty-body marker for each legacy empty function; or, for a typed `spec`,
  one parsed-type node with its optional width span and one integer-literal node
  with its optional sign and magnitude span; and
- exact token spans sufficient to map every node back to its source extent.

The tree retains spelling and source structure only. A parsed type or integer
literal is not a resolved type or decoded value. The AST shape and each source
span are inputs to, not results of, semantic analysis.

Duplicate module-member names are syntactically valid and produce separate
function nodes. Whether accepted syntax has a name conflict is decided only by
the rules in [`SEMANTICS_2026.md`](SEMANTICS_2026.md).

No recovery node or missing token is permitted in a successful parse. A
recovered tree accompanying diagnostics is tooling evidence only and must not be
treated as an accepted Orange program.

## 5. Parser failure and resource behavior

The parser must process tokens in source order with deterministic lookahead and
diagnostics. It must reject a missing, unexpected, duplicated, or trailing
token rather than silently reinterpret it.

Parser work for one source is bounded by all of the following:

- 262,144 syntax nodes;
- 1,048,576 parser events or equivalent syntax elements;
- 100 ordinary parse diagnostics plus at most one suppression diagnostic; and
- recovery delimiter nesting depth 64.

The single parser-resource diagnostic is outside the ordinary diagnostic
budget, so exhaustion cannot be hidden by earlier syntax errors.

Exhausting any parser budget is a stable parse error and cannot produce success.
Error recovery may advance to a bounded declaration or delimiter boundary for
diagnostic quality, but it must always consume input or stop. It must not loop,
recurse without the depth bound, or accept a recovered source.

For the same source bytes and Orange 2026 edition, token sequence, syntax tree,
diagnostic order, diagnostic codes, and success or failure result must be
byte-for-byte deterministic across repeated executions on the same compiler
revision.

## 6. Explicit non-language surface

Orange 2026 currently defines none of the following:

- imports, multiple modules, nested modules, attributes, or visibility;
- parameters, generic arguments, contracts, or effects;
- statements, general expressions, bindings, calls, arithmetic, control flow,
  or bodies other than the empty and single-literal forms above;
- proof terms, proof rules, claims, games, or a proof-bearing or canonical Core;
- targets, layout, ABI, leakage behavior, lowering, optimization, code
  generation, packaging, linking, or releases.

This syntax document defines no type rule, name-resolution rule, literal value,
Core construction, evaluation, or execution behavior. The narrow rules for the
typed reference slice are exclusively in
[`SEMANTICS_2026.md`](SEMANTICS_2026.md); parser acceptance must not be described
as semantic validation. Adding another surface requires a directed or accepted
language decision, normative syntax and semantics as applicable,
implementation, negative cases, and migration review.

## 7. Conformance boundary

Syntactic conformance for this slice requires positive legacy-empty and typed
`spec` sources; exact type, width, sign, magnitude, body, and declaration spans;
one malformed case for each grammar boundary; typed-`impl` rejection;
reserved-word-as-name cases; syntactic duplicate-name acceptance; generic
parsed-type spellings without semantic filtering; Unicode whitespace and
identifier rejection; each logical line-ending form; lexical and parser
resource-limit cases; trailing-token rejection; and repeated parse equality.

The following stable rule identifiers group executable lexical and parser
conformance for only the base S2 boundary accepted under D-025 and OEP-0002 at
revision
`52a3460853636f7cbaa27f3e27d86e032e3c82d4`. The additive typed-`spec`,
parsed-type, and signed-integer grammar belongs to S3a under D-026 and OEP-0003
and is indexed separately by `S3A-GRAMMAR-01` in
[`SEMANTICS_2026.md`](SEMANTICS_2026.md). These identifiers add an enforced
traceability contract; they do not add source syntax, semantics, or a
source-compatibility guarantee. Section 6's change-control boundary and this
section's evidence caveat remain normative prose rather than separately
relabeled executable rules. The evidence column is a minimum layer requirement
enforced by
`compiler/crates/orangec/tests/s2_conformance.rs`; a named test is evidence for
the recorded implementation revision, not a proof of the grouped rule.

| Rule | Normative boundary | Required evidence |
| --- | --- | --- |
| `S2-SOURCE-01` | Valid UTF-8, pre-lex source-size rejection, same-source half-open byte spans, logical line endings, and scalar columns | Source unit and CLI |
| `S2-TRIVIA-01` | The exact whitespace set, line comments, nested block comments, and trivia exclusion from the tree | Conformance, lexer, and parser unit |
| `S2-NAME-01` | ASCII identifiers, exact reserved words, no normalization or case folding, reserved-word rejection in identifier positions, and syntactic duplicate-name acceptance without binding | Lexer and parser unit |
| `S2-INTEGER-01` | Decimal, binary, and hexadecimal tokens, exact prefix forms, digit sets, and separator placement | Conformance and lexer unit |
| `S2-STRING-01` | Line-bounded lexical-only string tokens, the exact supported escape set, and no grammar or runtime meaning | Conformance and lexer unit |
| `S2-PUNCT-01` | The exact punctuation-token inventory and longest-token matching, without assigning parser roles | Conformance and lexer unit |
| `S2-LEX-RESOURCE-01` | At most 262,144 retained non-trivia tokens excluding exactly one zero-width `EOF`, 100 ordinary diagnostics plus stable suppression, the token-limit resource diagnostic outside the ordinary budget, and lexical resource exhaustion invalidating the source | Lexer and injected-resource unit |
| `S2-GRAMMAR-01` | The exact source, edition, one-module, and legacy empty-`spec`/empty-`impl` grammar accepted by S2 | Conformance and parser unit |
| `S2-AST-01` | Deterministic base source, edition, module, and legacy-empty function nodes in source order with exact token spans; duplicate names remain separate nodes and no missing-token or recovery node is accepted | Parser unit |
| `S2-PARSE-DIAG-01` | Base-grammar missing, unexpected, duplicated, and trailing-token rejection with same-revision deterministic diagnostic order, codes, and responsible spans | Conformance and parser unit |
| `S2-PARSE-RESOURCE-01` | Node, event-or-equivalent, diagnostic, and recovery-depth bounds; recovery consumes input or stops without unbounded recursion; exhaustion and recovered sources fail closed | Parser and injected-resource unit |
| `S2-PHASE-01` | Lexical invalidity prevents parser acceptance and parser diagnostics from cascading | Conformance, CLI, and parser unit |
| `S2-DETERMINISM-01` | Repeatable token, base-tree, parser-diagnostic, ordering, and parse success-or-failure results | Conformance, lexer, and parser unit |

Tests demonstrate the behavior of the recorded implementation revision only.
They do not prove grammar completeness, parser correctness, semantic soundness,
security, or implementation independence.
