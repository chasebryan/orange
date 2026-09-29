# Orange 2026 bindings and conversions specification

Status: proposed S3c semantics under OEP-0006, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-28

This document defines slice S3c of Orange 2026: typed `let` bindings inside a
`spec` body and explicit `as` conversions between the admitted types. It is a
delta over the proposed S3b expression rules in
[`EXPRESSIONS_2026.md`](EXPRESSIONS_2026.md), which are themselves a delta over
[`LANGUAGE_2026.md`](LANGUAGE_2026.md) and
[`SEMANTICS_2026.md`](SEMANTICS_2026.md). Everything those documents define and
this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0006](governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md),
which requires OEP-0005. At that point it replaces the S3b clauses listed in
section 12. Until then, the compiler behavior it describes exists so that the
proposal can be reviewed against running code, and it establishes no accepted
language meaning. It accepts no D-004 candidate.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

A standard names its intermediate values. RFC 8439 writes the ChaCha20
quarter round as a sequence of named updates, and FIPS 180-4 computes `T1` and
`T2` before it forms the new working variables. S3c lets a specification do the
same, and lets it move a value between types only where it says so:

```orange
edition 2026;
module chacha20 {
  spec quarter_a(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {
    let a1: Word[32] = a + b;
    let d1: Word[32] = (d ^ a1) <<< 16;
    let c1: Word[32] = c + d1;
    let b1: Word[32] = (b ^ c1) <<< 12;
    a1 + b1
  }

  spec load_le32(b0: Word[8], b1: Word[8], b2: Word[8], b3: Word[8]) -> Word[32] {
    (b0 as Word[32]) | ((b1 as Word[32]) << 8) | ((b2 as Word[32]) << 16)
      | ((b3 as Word[32]) << 24)
  }

  spec sample() -> Word[32] { quarter_a(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }
}
```

`orangec eval` prints `chacha20::sample: Word[32] = 0xea2a92f4`, the `a` word
of the RFC 8439 section 2.1.1 test vector.

Three commitments shape every rule below.

- **A name means one thing.** A binding is immutable, is evaluated once, and
  may not reuse the name of a parameter or an earlier binding. There is no
  shadowing and no assignment.
- **Every binding states its type.** The reader never infers a type; the
  declaration says it.
- **Every change of type is written.** `as` is the only conversion. It keeps
  an integer's value and, into a word type, takes the residue modulo 2^n. The
  S3b rule stands: nothing converts implicitly.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2 is unchanged. The
accepted surface grows to: `let` bindings at the start of a typed `spec` body,
references to bindings, and `as` conversions between any two of `Int`,
`Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`. Empty functions and `impl`
declarations keep their meaning exactly.

## 3. Grammar

No token and no reserved word is added. The S3b `typed_tail` and `expression`
productions become:

```text
typed_tail      = "->" parsed_type "{" binding* expression "}" ;
binding         = "let" IDENTIFIER ":" parsed_type "=" expression ";" ;

expression      = arithmetic | chain("&") | chain("|") | chain("^") | shift
                | conversion ;
conversion      = prefixed "as" parsed_type ;
```

Every other S3b production is unchanged. `let` and `as` are identifier tokens,
recognized by position:

- An identifier spelled `let` starts a binding when it is the first token of a
  body item and the next token is an identifier. Anywhere else it is a name.
- An identifier spelled `as` is the conversion keyword when it directly
  follows a complete `prefixed` operand. Anywhere else it is a name.

So `let`, `as`, and every S3b program that uses them as names keep their
meaning. One consequence is deliberate: a body item that begins with `let`
followed by a name is always a binding, so converting a parameter named `let`
at the start of a result expression is written `(let) as Int`.

A body holds zero or more bindings and then exactly one result expression. A
body that ends with a binding, a binding without its type, its `=`, or its `;`,
and a `;` after the result are syntax errors (`ORC0101`).

The syntax tree gains one binding node per binding, holding its name, parsed
type, and bound expression, and one conversion node per conversion, holding its
operand, the span of `as`, and its parsed target type. Every node keeps its
exact source span. A binding's span runs from `let` through its `;`; a
conversion's span runs from its operand through its target type.

## 4. Grouping

A conversion applies to exactly one `prefixed` operand: a literal, a name, a
call, a group, or a prefix operator applied to one of these. Prefix operators
therefore bind more tightly than `as`: `-x as Int` converts `-x`.

A conversion is a group of its own. An `as` that follows a binary operator's
right operand, a binary operator that follows a conversion, and a second `as`
at the same level are syntax errors (`ORC0108`) at the later token, naming it
and the token before it:

```orange
spec bad(x: Word[8], y: Word[8]) -> Word[32] { x + y as Word[32] }       // ORC0108 at `as`
spec good(x: Word[8], y: Word[8]) -> Word[32] { (x + y) as Word[32] }
spec also_good(x: Word[8], y: Word[8]) -> Word[32] { (x as Word[32]) + (y as Word[32]) }
```

The two good forms mean different things, which is why neither is chosen
silently: the first adds modulo 2^8 and then widens; the second widens and
then adds modulo 2^32. As in S3b, one ungrouped expression produces one
`ORC0108`, and parsing continues after it.

## 5. Names and scope

Within one typed `spec`, parameters and bindings share one set of names,
compared by exact ASCII spelling. A binding whose name equals a parameter's or
an earlier binding's is an error (`ORC0219`) at the binding's name. Orange has
no shadowing.

A binding is **in scope** from the end of its own `;` to the end of the body:
in the values of the bindings after it and in the result expression. It is not
in scope in its own value.

In an expression, a name refers to the parameter with that spelling, if there
is one, and otherwise to the binding in scope with that spelling. Any other
name is an error (`ORC0211`):

- a name that matches a binding not yet in scope, including the binding being
  defined, is reported as used before it is bound, citing that binding;
- in a body that has bindings, an unknown name is reported as not a parameter
  or binding of the function; and
- in a body without bindings, the S3b message is unchanged.

## 6. Typing

**Bindings.** A binding's parsed type is resolved exactly as a parameter type
is (`EXPRESSIONS_2026.md` section 7). Its value is checked against that type,
and every use of the binding has that type. If the type does not resolve, the
error is reported once where the type is written, the value is not checked,
and uses of the binding are not reported again.

**Conversions.** A conversion `e as T` is checked against an expected type
`E`:

1. The target `T` is resolved. If it resolves and differs from `E`, that is an
   error (`ORC0214`) at the target.
2. The operand's own type is the type of its **first typed leaf**: the first
   name, call, or conversion in `e`, from left to right, looking through
   groups, prefix operators, and both operands of arithmetic and bitwise
   operators, and only the left operand of a shift or rotation. Its type is
   the declared type of the name, the result type of the called function, or
   the target type of the inner conversion. Because every S3b operator gives
   its result the type of its operands, this is exactly the type `e` must
   have. Literals take their type from their context and are not leaves.
3. If `e` has no typed leaf, as in `(1 + 2) as Word[8]`, that is an error
   (`ORC0220`) at `e`. Write the literal where its type is required instead,
   or give it a type with a binding.
4. If the leaf has no type, because the name or function is unknown or its
   declared type did not resolve, the leaf's own error is reported and the
   rest of `e` is not checked.
5. Otherwise `e` is checked against its own type, by the S3b rules.

Every pair of admitted types is a valid conversion, including a type to
itself. There is still no implicit conversion: a name, call, or conversion of
one type where another is expected is `ORC0214`, exactly as in S3b.

## 7. Meaning

**Bindings.** A call binds the parameters to the argument values, then
evaluates each binding once, in source order, binding its name to the value,
and then evaluates the result expression. A binding's value is computed even
if nothing uses it; every computation is pure and total, so this is visible
only in the evaluation step count.

**Conversions.** Let v be the operand's integer value: the integer itself for
`Int`, and the canonical residue from 0 through 2^m - 1 for `Word[m]`. Then:

| Target | Value |
| --- | --- |
| `Int` | v |
| `Word[n]` | the residue of v modulo 2^n, from 0 through 2^n - 1 |

The residue is the mathematical one, so a negative integer converts as
two's-complement arithmetic would suggest, and widening a word keeps its value:

| Expression | Value |
| --- | --- |
| `n as Word[8]`, with `n = -1: Int` | `0xff` |
| `n as Word[16]`, with `n = 65537: Int` | `0x0001` |
| `w as Int`, with `w = 0xffffffffffffffff: Word[64]` | `18446744073709551615` |
| `b as Word[64]`, with `b = 0xff: Word[8]` | `0x00000000000000ff` |
| `w as Word[16]`, with `w = 0x0123456789abcdef: Word[64]` | `0xcdef` |

There is no signed word type and no sign extension. A standard that reads a
word as signed would need a later slice.

## 8. Diagnostics

The examination order of `EXPRESSIONS_2026.md` section 10 is refined: for a
typed `spec`, each parameter in order, then the result type, then each binding
in order (its name's uniqueness, then its type, then its value), then the
result expression. If the result type does not resolve, neither the bindings
nor the result is checked. Within an expression, a conversion is examined
before its operand, as an operator is: its target type, then the comparison
with the expected type, then the operand. Call cycles are still reported after
every function, and calls inside binding values and conversion operands are
edges like any other.

The new and widened categories are:

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0108` | parse | `as` shares a level with a binary operator or another `as` without parentheses |
| `ORC0211` | semantic | a name is not a parameter or a binding in scope |
| `ORC0214` | semantic | an expression's type, or a conversion's target, differs from the expected type |
| `ORC0219` | semantic | a binding repeats the name of a parameter or an earlier binding |
| `ORC0220` | semantic | the operand of `as` has no typed leaf |

`ORC0106` also reports the binding limit of section 10. Every other code keeps
its meaning.

## 9. Typed Reference Core

The Core of `EXPRESSIONS_2026.md` section 11 gains locals and two node kinds:

```text
core_function  = function_id function_name parameter_type* core_type
                 core_local* body ;
core_local     = local_name core_type body ;
node_kind      = ...
               | local index
               | convert from_type ;
```

Each binding becomes one Core local, in source order, carrying its name, its
type, and its value as a postorder body whose last node has that type. A
`local index` node reads the binding at that zero-based index, which is always
earlier than the local being computed. A `convert` node has the target type
and records its operand's type. The Core is still internal and noncanonical,
with no encoding, digest, or proof role. A function without bindings has no
locals, and its Core is exactly its S3b Core.

## 10. Resource limits and failure

The S3b budgets of `EXPRESSIONS_2026.md` section 14 remain. S3c adds or refines
the following.

**Parsing.** A typed body declares at most 256 bindings; a 257th is a parser
resource limit (`ORC0106`) at that binding. A conversion adds one to its
operand's tree height and opens no nesting level. Each binding's value is its
own expression, bounded separately by the nesting and height limits.

**Semantic events.** S3c adds one event per binding-name uniqueness check and
one per conversion. A binding's type and a conversion's target count as parsed
types (one event per identifier and one per width token). The search for a
conversion operand's first typed leaf consumes no events: each expression node
is visited by at most one such search, which stops at the first nested
conversion or call, and the node is then counted when it is checked.

**Core nodes.** Each Core local counts one node for itself, one for its type,
and one per node of its value.

**Evaluation.** A `local` node and a `convert` node each cost one step. A
binding costs nothing beyond the steps of its value. Bindings occupy the frame
of their function, so they add no call depth.

Exhausting any budget, and any allocation failure, yields one resource
diagnostic, no Core, and no value line. The deepest sources the limits admit,
including 64 levels of nested conversions and 256 bindings each nested 64
levels deep, must parse, analyze, and evaluate within 1 MiB of native stack.

## 11. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged. The S3c conformance runner
(`compiler/crates/orangec/tests/s3c_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the S3b
runner: every rule needs command-line evidence that runs twice with identical
results, except where the index says unit evidence alone, and every rule names
unit tests declared exactly once in their sources' test modules.

### S3c conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3C-GRAMMAR-01` | Section 3 | Bindings and conversions parse with exact spans; a binding without its type, `=`, or `;`, or a body without a result, is `ORC0101`. | CLI and parser unit |
| `S3C-CONTEXT-01` | Section 3 | `let` and `as` are names everywhere except where a binding or a conversion begins. | CLI and parser unit |
| `S3C-GROUP-01` | Section 4 | `as` converts one prefixed operand; beside a binary operator or another `as` it is one `ORC0108` at the later token. | CLI and parser unit |
| `S3C-SCOPE-01` | Section 5 | Bindings are in scope after their `;`; parameters and bindings never share a name; use before binding is `ORC0211`. | CLI and unit |
| `S3C-BIND-01` | Section 6 | A binding's value is checked against its declared type, and an unresolved binding type is reported once. | CLI and unit |
| `S3C-CONV-TYPE-01` | Section 6 | A conversion's operand has the type of its first typed leaf; no leaf is `ORC0220`; the target must be the expected type. | CLI and unit |
| `S3C-CONV-01` | Section 7 | Every pair of admitted types converts to the value, reduced modulo 2^n for a word target. | CLI and unit |
| `S3C-EVAL-01` | Sections 7 and 8 | Bindings are evaluated once each, in order, before the result, and calls inside them join the call graph. | CLI and unit |
| `S3C-DIAG-01` | Section 8 | Diagnostic order, spans, and non-cascading behavior are exactly as specified. | CLI and unit |
| `S3C-CORE-01` | Section 9 | Core functions carry typed locals in source order, with `local` and `convert` nodes. | Unit and CLI observation |
| `S3C-RES-BIND-01` | Section 10 | 256 bindings are accepted and a 257th fails with `ORC0106`. | Generated CLI and parser unit |
| `S3C-RES-EVENT-01` | Section 10 | Semantic events and Core nodes for bindings and conversions are counted exactly. | Unit |
| `S3C-RES-STEP-01` | Section 10 | Binding reads and conversions cost one step each, and bindings cost nothing more. | Unit |
| `S3C-RES-STACK-01` | Section 10 | The deepest admitted sources with conversions and bindings fit in 1 MiB of native stack. | Unit |
| `S3C-RES-FAIL-01` | Section 10 | Allocation failure, foreign syntax trees, and inconsistent Core yield no partial tree, Core, or value. | Unit |
| `S3C-COMPAT-01` | Section 12 | S3b programs, including those that use `let` and `as` as names, keep their meaning and messages. | CLI and unit |
| `S3C-DETERMINISM-01` | Section 11 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 12. Relationship to S3b

When OEP-0006 is accepted, this document replaces these clauses of
`EXPRESSIONS_2026.md`:

- section 4, the `typed_tail` and `expression` productions and the sentence
  that a typed body contains exactly one expression, by section 3;
- section 5, where grouping gains the conversion of section 4;
- section 6, where a name may also refer to a binding in scope, by section 5;
- section 8, the sentence that there is no conversion, by section 6: implicit
  conversion is still absent, and `as` is the one explicit form;
- sections 10, 11, and 14, extended by sections 8, 9, and 10; and
- section 17, which no longer lists local bindings or conversions between
  types among the exclusions.

Every source that S3b accepts is accepted by S3c with the same Core values and
the same output bytes, because `let` and `as` are not reserved. A source that
S3b rejects gets the same diagnostics unless it contains `let` followed by a
name at the start of a typed body, or `as` directly after a complete operand;
those spellings were never valid, and they now begin a binding or a
conversion. The `ORC0211` message changes only in a body that has bindings,
which S3b could not express.

## 13. Explicit non-claims and future work

This slice defines no tuples, booleans, comparisons, conditionals, loops,
mutation or reassignment, shadowing, type inference, signed words, sign
extension, arbitrary word widths, division, remainder, variable shift or
rotation amounts, recursion, typed `impl` declarations, contracts, effects,
secrecy labels, failure values, imports, multiple modules, proofs, claims,
games, canonical Core, code generation, target, ABI, layout, leakage, package,
cryptographic, release, compatibility, or production behavior.

The absence of tuples is visible in the ChaCha20 fixture: each output word of
the quarter round is its own function. Tuples, or a record of four words, are
the natural next step.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. No statement here says whether any conversion
or expression would run in constant time on any machine. Tests establish the
tested behavior of one implementation at one revision; they do not prove
semantic soundness, completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
