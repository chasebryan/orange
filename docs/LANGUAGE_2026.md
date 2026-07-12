# Orange 2026 lexical and grammar specification

Status: normative pre-alpha syntax under D-025 and the effective owner direction
recorded by provisional OEP-0002

Edition: `2026`

Snapshot: 2026-07-12

This document defines the complete lexical and syntactic language accepted by
the first Orange parser. It is intentionally small. Acceptance establishes only
that source text has this shape; it does not establish typing, name resolution,
execution, proof, compilation, cryptographic correctness, or any other semantic
property.

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

Except for the exact `2026` token in the mandatory edition declaration, integer
tokens have no grammatical role in this slice. Any other well-formed integer is
therefore lexically valid and still rejected by the parser.

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

The parser grammar below uses only `(`, `)`, `{`, `}`, and `;`. Every other
punctuation token is a lexical reservation without syntax or semantics in this
slice.

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
function_decl = function_kind IDENTIFIER "(" ")" empty_body ;
function_kind = "spec" | "impl" ;
empty_body    = "{" "}" ;
```

`"2026"` in `edition_decl` means the exact decimal integer-token spelling
`2026`. Prefixes, separators, leading zeroes, or another value do not select the
edition. The declaration is mandatory and must be first.

One source contains exactly one module. A module may contain zero or more
function declarations in source order. A function has exactly one kind, one
identifier, an empty parameter list, and an empty body. Trivia may occur between
tokens wherever token boundaries permit it.

This grammar is LL(1): each declaration begins with one distinct reserved word,
and `}` terminates the declaration list. There is no precedence, implicit
semicolon, contextual keyword, or grammar ambiguity in this slice.

The following source is accepted:

```orange
edition 2026;
module demo {
  spec identity() {}
  impl rounds() {}
}
```

## 4. Syntax-tree mapping

Every accepted source maps to one deterministic syntax tree containing:

- one source-file node;
- one edition-declaration node with the exact edition token;
- one module node with its identifier;
- one function node per declaration, in source order, with its `spec` or `impl`
  kind and identifier; and
- exact token spans sufficient to map every node back to its source extent.

Duplicate module-member names are syntactically valid and produce separate
function nodes. Whether names conflict is future name-resolution work.

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
- parameters, return types, generic arguments, contracts, or effects;
- statements, expressions, bindings, control flow, or non-empty bodies;
- type rules, name resolution, evaluation, or execution behavior;
- proof terms, proof rules, claims, games, or a semantic Core;
- targets, layout, ABI, leakage behavior, lowering, optimization, code
  generation, packaging, linking, or releases.

Acceptance by this parser must not be described as validation of any item in
that list. Adding one requires a later directed or accepted language decision,
normative specification, implementation, negative cases, and migration review.

## 7. Conformance boundary

Conformance for this slice requires positive sources, one malformed case for
each grammar boundary, reserved-word-as-name cases, duplicate-name acceptance,
Unicode whitespace and identifier rejection, each logical line-ending form,
lexical and parser resource-limit cases, trailing-token rejection, and repeated
parse equality.

Tests demonstrate the behavior of the recorded implementation revision only.
They do not prove grammar completeness, parser correctness, semantic soundness,
security, or implementation independence.
