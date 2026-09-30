# Orange 2026 sizes specification

Status: proposed S3m semantics under OEP-0016, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3m of Orange 2026: **size parameters**, which
let one `spec` stand for a finite family of functions, one for each value of
its sizes, such as SHA-256 of every message from 1 through 119 bytes; the
**sizes** that write array lengths, fill lengths, and loop bounds from them;
and **sized calls**, which name one member of the family. Every member is
checked as if it were written out by hand, so every rule of the earlier
documents applies to it unchanged. It is a delta over the proposed S3l rules
in [`BYTES_2026.md`](BYTES_2026.md), which are a delta over
[`TUPLES_2026.md`](TUPLES_2026.md) and the documents it extends. Everything
those documents define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0016](governance/oeps/OEP-0016-orange-2026-sizes.md), which requires
OEP-0015. At that point it replaces the S3l clauses listed in section 13.
Until then, the compiler behavior it describes exists so that the proposal can
be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`ORDER_2026.md`](ORDER_2026.md), proposed under OEP-0017, extends this
> document with conversions in a byte order, which may write a size-dependent
> length after the order, as `m as big Word[32]^(16 * blocks)`, and convert a
> size's value, as `(8 * len) as big Word[8]^8`, and
> [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md), proposed under OEP-0018,
> admits type parameters beside size parameters, `K in {F, P, Q}`, with at most
> four parameters in brackets and 256 instances in all, and fits a call without
> brackets by its arguments' types, reading each argument once.
> [`LENGTHS_2026.md`](LENGTHS_2026.md), proposed under OEP-0019, lets a length
> written with sizes reach 65,536, while a function still has at most 256
> instances. Every source this document accepts keeps its meaning under all
> three.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

A standard defines its algorithms for inputs of many lengths. FIPS 180-4 pads
a message of any length to whole 64-byte blocks and absorbs them one at a
time; RFC 2104 hashes the key, padded to a block, followed by a message of any
length; RFC 8439 section 2.5 feeds Poly1305 a message of any length sixteen
bytes at a time, the last block holding what is left. Through S3l every array
in Orange had a length written as an integer, so an Orange program could hash
a message of 3 bytes or of 56, but a function for both had to be written
twice.

S3m writes it once. A function declares **size parameters** in square
brackets, each with a finite range, and uses them where a length is written:

```orange
// FIPS 180-4 section 5.1.1: the message, the byte 80, zeros, and the
// message's length in bits fill ((len + 8) / 64) + 1 blocks.
spec pad[len in 1..120](m: Word[8]^len) -> Word[8]^(64 * (((len + 8) / 64) + 1)) {
  m ++ hex"80" ++ [0; ((64 * (((len + 8) / 64) + 1)) - len - 3)]
    ++ [((8 * len) / 256) as Word[8], (8 * len) as Word[8]]
}

spec absorb[blocks in 1..4](p: Word[8]^(64 * blocks)) -> Word[8]^32 {
  digest(for b in 0..blocks with h: Word[32]^8 = initial_hash() {
    compress(h, p[64 * b..64 * b + 64])
  })
}

spec sha256[len in 1..120](m: Word[8]^len) -> Word[8]^32 { absorb(pad(m)) }

spec abc() -> Word[8]^32 { sha256("abc") }
```

`pad` stands for 119 functions, `pad[1]` through `pad[119]`, one for each
value of `len`. Each is an **instance**: the function with its sizes replaced
by their values. The compiler checks every instance before anything runs,
exactly as it would check the same function written out with that value, so
every length is proved, every index and slice is proved in range, and every
budget is counted, in every instance. A call names its instance by its sizes,
as `pad[3](m)`, or, as above, by the lengths of its arguments: `sha256("abc")`
calls `sha256[3]`, whose parameter has 3 elements, and `absorb(pad(m))` calls
the instance of `absorb` whose parameter has the length of `pad`'s result.

Nothing is symbolic. A size is not a variable that the checker reasons about;
it is a number, different in each instance, and the instances are finite in
number. What is proved for `sha256` is proved for each of its 119 instances
separately, which is all a finite family of functions needs.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3l extends it, is
unchanged: sizes are evaluated during analysis, from the source alone, and no
size depends on a value computed when a program runs. Every S3l source keeps
its meaning (section 13).

## 3. Grammar

S3m adds no token and no reserved word. `in` is a name except between a size
parameter's name and its first bound, as it is in a loop.

The function, array type, fill, loop, and call productions of the earlier
documents become:

```text
function     = "spec" IDENTIFIER size_params? "(" parameters? ")" ( "->" type body | "{" "}" )
             | "impl" IDENTIFIER "(" ")" "{" "}" ;
size_params  = "[" size_param ( "," size_param )* "]" ;
size_param   = IDENTIFIER "in" INTEGER ".." INTEGER ;
size         = INTEGER | IDENTIFIER | "(" expression ")" ;
array_type   = element_type "^" size ;
fill         = "[" expression ";" size "]" ;
loop         = "for" IDENTIFIER "in" size ".." size "with" accumulator "=" expression step ;
call         = ( IDENTIFIER "::" )? IDENTIFIER sizes? "(" arguments? ")" ;
sizes        = "[" expression ( "," expression )* "]" ;
```

A function has at most four size parameters, and a call gives at most four
sizes. A size parameter's bounds are integer tokens, in any radix the lexer
admits. A size is an integer token, as every length and bound was before, a
name, or an expression in parentheses: a length computed from sizes is written
`Word[8]^(2 * n)`, `[0; (64 - klen)]`, or `for i in 0..(n - 1)`, and its
parentheses open one nesting level. A sized call's sizes are expressions.
Section 5 says which expressions have values as sizes.

**Sized calls and indices.** A name followed by `[` is a sized call when the
brackets hold only integer tokens, names, `+`, `-`, `*`, `/`, `%`, commas, and
balanced parentheses, and `(` follows the `]`; otherwise it is an index or a
slice, as before. So `a[i](b)` and `sha256[2](m)` are sized calls, while
`a[i]`, `a[1..3]`, and `a[x[0]]` are an index, a slice, and an index, and in
`a[i](b)[0]` the final `[0]` indexes the call's value. A
qualified name `m::f` is always a call, so `m::f[` always begins sizes. The
scan that decides reads each token at most once over a whole source.

The parse errors are `ORC0101`. Those of size parameters have the note "a
sized function is written `spec f[n in 1..5](x: Word[8]^n) -> Type { ... }`
and checked once for each n from 1 up to, but not including, 5"; those of
sized calls "a sized function is called with its sizes in brackets before its
arguments, as in `sha256[2](m)`":

| Form | Message | Label or note |
| --- | --- | --- |
| `spec f[n 1..3]` | "expected `in` after the size's name" | size parameter note |
| `spec f[n in a..3]`, `spec f[n in 1 3]` | "expected the size's first bound", "expected `..` between the size's bounds" | size parameter note |
| `spec f[n in 1..]`, `spec f[n in 1..m]` | "expected the size's second bound" | size parameter note |
| `spec f[n in 1..3 m in 1..2]`, `spec f[]` | "expected `,` or `]` after the size parameter", "expected an identifier for the size parameter" | size parameter note |
| a fifth size parameter | "a function has at most 4 size parameters", from its name through its second bound | label "one size parameter too many" |
| `impl f[n in 1..2]() {}` | "`impl` functions have no size parameters" | label "size parameters are allowed only on typed `spec` functions" |
| `spec f[n in 1..2]() {}` | "expected `->` after the parameter list" | label "a `spec` with parameters or sizes needs a result type and a body expression" |
| `Word[8]^n + 1` | "expected the end of the type after its array length" | "an array length computed from sizes is written in parentheses, as in `Word[8]^(2 * n)`" |
| `[0; 2 * n]` | "expected `]` after the array length" | "a length computed from sizes is written in parentheses, as in `[0; (2 * n)]`" |
| `for i in n * 2..4`, `for i in 0..n - 1 with` | "expected `..` between the loop's bounds", "expected `with` and the loop's accumulator" | "a bound computed from sizes is written in parentheses, as in `for i in 0..(n - 1) with s: Type = start { step }`" |
| `Word[8]^-1`, `[0; -1]`, `for i in 0..-n` | "expected a length after `^`", "expected an array length after `;`", "expected the loop's second bound" | the notes of the earlier documents |
| `g[1, 2 3](a)` | "expected `,` or `]` after the size" | sized call note |
| a fifth size in a call | "a call gives at most 4 sizes", at that size | label "one size too many" |
| `m::g[1]` | "expected `(` after the sizes" | sized call note |

**Nesting and height.** A size's parentheses count one nesting level of the
64-level budget and one level of height, as a group does. A size is part of
the expression or type that holds it: a fill's length, a loop's bounds, and a
call's sizes count toward the height of the fill, the loop, and the call, so
`[0; ((1))]` has height 4 and `g[1](a)` height 2, and the limit of 256 on an
expression's height is unchanged.

The syntax tree gains, on a function, its size parameters, each with its
name, the spans of its bounds, and its own span from its name through its
second bound; on an array type, a fill, and a loop, a size in place of each
integer span, holding its span and, for a name or a group, its expression;
and on a call, its sizes, kept with its module qualifier, if any, apart from
its arguments.

## 4. Size parameters and instances

A size parameter `n in a..b` takes each integer value from a up to, but not
including, b. Its bounds must satisfy `a < b <= 65536`, the bound of a loop:

- a range whose bounds are equal or reversed is `ORC0238` (new), "the size
  range 3..3 is empty", at its second bound, labeled "a size takes at least
  one value"; and
- a bound over 65536 is `ORC0238`, "a size's bound must be at most 65536", at
  that bound (the first when both are), labeled "bound too large".

A function's **instances** are the tuples of values of its sizes, one value
from each range. A function without size parameters has one instance, with no
sizes. A function has at most 256 instances: more, as `spec many[a in 1..20,
b in 1..20]` with 361, is `ORC0238`, "`many` has 361 instances, but a function
has at most 256", from its first size parameter through its last, labeled "too
many instances". All three have the note "a size parameter `n in a..b` takes
each value from a up to, but not including, b, with a < b <= 65536, and a
function has at most 256 instances".

Instances are ordered by their values, the first size changing slowest:
`spec code[a in 1..3, b in 7..9]` has the instances `code[1, 7]`,
`code[1, 8]`, `code[2, 7]`, and `code[2, 8]`, in that order. An instance is
named as a call names it, `f[2]` or `f[1, 3]`.

**Names.** A size parameter's name is in scope in the function's parameter
types, result type, and body. Size parameters, parameters, bindings, loop
indices, and accumulators share one namespace, and each name is unique:

- a size parameter or a parameter that repeats a size parameter's name is
  `ORC0218`, "duplicate parameter `n`", at the repetition, labeled "this name
  is already a size parameter", with a secondary label "first size parameter
  is here" or "the size parameter is here" and the note "size parameters and
  parameters share one namespace, and each name is unique"; and
- a binding, loop index, or accumulator that repeats one is `ORC0219`, as a
  repeated parameter's name is, with the secondary label "the size parameter
  is here".

A function's size parameters are checked in order, each range and then its
name, and then its parameters' names against them; the count of instances is
checked only when every range is valid and no name repeats. A function any of
whose size parameters is in error has no instances: its body is not checked
and it contributes no Core.

## 5. Sizes

A **size** is an expression that has a value in each instance, computed
during analysis. Its parts are integer literals, the names of the function's
size parameters, prefix `-`, parentheses, and the binary operators `+`, `-`,
`*`, `/`, and `%`. The value is exact: `/` and `%` are Euclidean and total as
for `Int` (`CONDITIONS_2026.md`), so `x / 0` is 0 and `x % 0` is x. Grouping
follows the rules of `EXPRESSIONS_2026.md` section 5, as for any expression.

- Any other part, such as a parameter, a binding, a loop index, a call, a
  comparison, or a conversion, is `ORC0237` (new), "a size may use only
  integer literals and size parameters", at the first such part, labeled "this
  is neither", with the note "a size is fixed in each instance of its
  function: it is built from integer literals and the function's size
  parameters with `+`, `-`, `*`, `/`, `%`, and parentheses".
- A literal, or the value of any part, with more significant bits than the
  limit of `Int` (16,384, `SEMANTICS_2026.md` section 9) is `ORC0205`,
  "integer magnitude exceeds the 16384-significant-bit limit", at that part,
  labeled "this part of the size is too large". Every part is computed, so a
  part too large is an error even where it cancels.

A part that is not a size is reported before a part too large, wherever each
stands; otherwise the leftmost fault is reported. A size is evaluated from
left to right, operands before their operator.

**Where sizes stand.**

- *Array lengths.* In `T^s`, the value of s in the instance must be from 1
  through 256. Any other value is `ORC0221`, "this array length is 0, but an
  array has 1 through 256 elements" (or "this array length is far outside 1
  through 256" beyond the range of a 64-bit integer), at the length, labeled
  "unsupported array length in this instance", with the note "a length
  written with sizes is computed in each instance of its function, and every
  instance's lengths are from 1 through 256". An integer token is read as
  before.
- *Fill lengths.* In `[e; s]`, the same holds for s.
- *Loop bounds.* In `for i in s..t`, each bound's value must be from 0 through
  65536. A negative value is `ORC0225`, "a loop bound must be at least 0",
  labeled "negative loop bound in this instance"; every other rule of
  `LOOPS_2026.md` applies to the values, so `for i in n..n` is "the loop range
  1..1 is empty" in the instance where n is 1.
- *Sized calls.* Each of a call's sizes is a size of the caller's instance
  (section 7).
- *Values.* A size parameter's name where an expression stands is an `Int`
  constant: its value in the instance. It is a typed leaf of type `Int`.
  Where an index or a slice bound is analyzed, it counts as an integer
  literal of that value, so `x[n - 2..]` is a static slice and `x[n]` is
  checked against the array's length in every instance. It keeps the forms a
  slice bound may take: `x[n / 2..]` is still `ORC0226`.

A modulus is not a size: `Mod[n]` is `ORC0232`, as any modulus that is not
built from integer literals is (`MODULAR_2026.md`).

## 6. Checking instances

A sized function is checked once for each of its instances, in order. Each
instance is checked as the function written out with its sizes replaced by
their values: its parameter and result types are resolved with those values,
its body is checked against them, and every rule of the earlier documents
applies, with every length, index, slice, and budget counted in that
instance.

The first instance of a function in error is the only one reported. Each of
its diagnostics, apart from those that report an exhausted budget, has the
note "in the instance `f[2]`, the first of `f` in error: a sized function is
checked once for each value of its sizes", and the instances after it are not
checked. So `spec last[n in 1..5](x: Word[8]^n) -> Word[8] { x[3] }` is
reported once, "index `3` is out of range for `Word[8]^1`", in the instance
`last[1]`, although `last[2]` and `last[3]` are in error too.

Before any body is checked, each function's instances receive their
signatures: their parameter and result types, resolved without reporting, so
that a call in any function can be checked against the instance it names.

## 7. Calls

A call of a function with size parameters names one of its instances.

**Calls that write sizes.** `f[s1, ..., sk](args)` gives one size for each of
`f`'s size parameters, in order. The sizes are evaluated in the caller's
instance, from left to right (section 5), and each value must lie in its
parameter's range.

- A number of sizes other than `f`'s is `ORC0239` (new), "`first` takes 1
  size, but this call gives 2", or for a function without size parameters
  "`outside` has no size parameters, but this call gives 1 size", at the call,
  labeled "wrong number of sizes", with the note "a sized function is called
  with one value for each of its sizes, in brackets before its arguments, as
  in `sha256[2](m)`; a function without sizes is called without brackets".
- A value outside its range is `ORC0238`, "`first` is defined for `n` in
  1..4", at that size, labeled "this size is 4" (or "this size is far outside
  that range"), with the note of section 4.

**Calls that write none.** A call without sizes of a function with size
parameters calls the one instance whose array parameters have the lengths of
the call's arguments. For each parameter that is an array in the first
instance, the length of the argument is **found without reporting**, as a
join's operand's is (`BYTES_2026.md` section 6); a call of a sized function
without sizes has the result type of the instance it calls, so found lengths
pass through calls such as `absorb(pad(m))`. An instance **fits** when each of
those parameters of its own is an array of the found length; an argument
whose length is not found fits every instance. The instances are tried in
order:

- exactly one fits: the call calls it;
- two or more fit, as for every instance of a function without array
  parameters: `ORC0239`, "this call fits more than one instance of `count`,
  among them `count[1]` and `count[2]`", naming the first two, at the call,
  labeled "write the sizes in brackets"; and
- none fits: `ORC0238`, "no instance of `first` takes arguments of these
  lengths", at the call, labeled "an array of length 9 is given" (or "arrays
  of lengths 4, 9 are given", or "no instance fits these arguments"), with
  the note "`first` is defined for `n` in 1..4".

Both have the note "a call that writes no sizes calls the one instance of its
function whose array parameters have the lengths of its arguments; any other
call writes its sizes in brackets, as in `absorb[2](p)`". A call whose number
of arguments differs from the function's parameters is checked against the
first instance, where it is `ORC0213` as before.

**The instance's signature.** Once named, the call is checked as a call of
that instance: its arguments against the instance's parameter types, and the
call's value has the instance's result type. A qualified call `m::f[2](x)` or
`m::f(x)` names an instance of `f` in the used module `m` in the same way.

**Cycles.** The call graph is a graph of instances. A call from one instance
to another that closes a cycle is `ORC0217`, as before, naming the instances,
as "call cycle `swap[1]` -> `swap[2]` -> `swap[1]`". A call that closes a
cycle in several instances of its function is reported once. Instances of one
function that call one another without a cycle, as `f[2]` calling `f[1]`, are
not a cycle.

## 8. Meaning

An instance means what the function written out with its sizes' values
means. A size parameter's name in an expression is the integer it stands for.
A call of an instance evaluates its arguments from left to right and applies
that instance.

`orangec eval` evaluates every instance of each `spec` of the root module
without value parameters, in order, and names each as a call names it:

```text
sizes::zeros[1]: Word[8]^1 = [0x00]
sizes::zeros[2]: Word[8]^2 = [0x00, 0x00]
sizes::code[1, 7]: Int = 17
sizes::total: Int = 16
```

A function without size parameters is displayed as before.

## 9. Typed Reference Core

Each instance is one Core function, which records its function's name and the
values of its sizes in declaration order, empty for a function without size
parameters. Core function identities are assigned in the order of the
module's functions, each function's instances taking consecutive identities in
order. An instance's Core is the Core of the function written out with its
sizes' values: a size parameter's name in an expression is one `literal` node
of type `Int`, and every type in it is concrete. A call node refers to the
identity of the instance it names. Core gains no node kind, and a function
without size parameters has exactly its S3l Core.

The Core is still internal and noncanonical, with no encoding, digest, or
proof role.

## 10. Evaluation

Sizes cost nothing at run time: a length, a fill length, or a loop bound
written with sizes costs what the same integer costs, and a size parameter's
name in an expression costs one step, as an integer literal does. An
instance's steps are those of the function written out. The per-source budget
of 1,048,576 steps covers every instance evaluated, and each evaluated value
records the sizes of its instance.

## 11. Resource limits and failure

The S3l budgets remain. S3m adds or refines the following.

- A function has at most 4 size parameters and 256 instances, and a call
  gives at most 4 sizes; each bound of a size parameter is at most 65536.
- Nesting and height are as in section 3.
- Each size parameter costs two semantic events when its function is
  checked, one for its range and one for its name. Each part of a size that
  is evaluated costs one semantic event, parentheses included, each time it is
  evaluated: once for each instance's signature and once for each instance's
  body. With the length `(((n + 1) * 2) - 3)`, a function of one instance
  costs 22 events more than the same function with the length written `1`:
  two for its size parameter and ten for the length's parts, counted for its
  signature and for its result type. Since every instance is checked, a
  function's cost is the sum of its instances' costs, and the per-source
  budgets of 1,048,576 semantic events and 262,144 Core nodes bound all of
  them together.
- An allocation failure while computing a size is `ORC0209`, "size storage
  allocation failed", at the part, and while recording an evaluated instance's
  sizes `ORC0301`, "reference evaluation result allocation failed", labeled
  "evaluated function sizes could not be reserved". Neither gives partial
  output.

The deepest sources the limits admit, including 63 sized calls nested in each
other's arguments, 63 calls without sizes nested in each other, each calling
the instance its argument fits, and a fill whose length is nested in 62
groups, must parse, analyze, and evaluate within 1 MiB of native stack. A
syntax tree whose spans, including those of every size parameter, its name,
and its bounds, every size, and every call's sizes, do not all belong to the
source it is supplied with is `ORC0210` for that source, and nothing is
checked.

## 12. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged: instances are checked, numbered, and evaluated in one order, so
the same source gives the same diagnostics, Core, and output bytes. The S3m
conformance runner (`compiler/crates/orangec/tests/s3m_conformance.rs`)
parses this index and requires exact agreement with its evidence map, under
the same rules as the S3b through S3l runners.

### S3m conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3M-SYNTAX-01` | Section 3 | Size parameters, sizes in array types, fills, and loop bounds, and sized calls parse with exact spans, levels, and heights; a name followed by bracketed sizes and `(` is a sized call and any other brackets an index or a slice; malformed forms are `ORC0101` with the specified messages and notes. | CLI and parser unit |
| `S3M-SIZE-01` | Section 4 | A size parameter's range is nonempty with bounds at most 65536, a function has at most 256 instances, ordered with the first size slowest (`ORC0238`), and size parameters share one namespace with parameters and bindings (`ORC0218`, `ORC0219`). | CLI and unit |
| `S3M-VALUE-01` | Section 5 | A size is built from integer literals and size parameters (`ORC0237`), computed exactly within the significant-bit limit of `Int` with Euclidean, total `/` and `%`, and gives every length from 1 through 256 (`ORC0221`) and every loop bound as specified; a size's name is an `Int` constant, and a literal in indices and slice bounds. | CLI and unit |
| `S3M-INSTANCE-01` | Section 6 | Every instance is checked in order as the function written out with its sizes' values, and only the first instance of a function in error is reported, named in a note. | CLI and unit |
| `S3M-CALL-01` | Section 7 | A call names one instance by its sizes, counted and in range (`ORC0239`, `ORC0238`), or by the found lengths of its arguments, fitting exactly one (`ORC0239`, `ORC0238`), across modules as within one; cycles between instances are `ORC0217`, reported once per closing call. | CLI and unit |
| `S3M-CORE-01` | Section 9 | Each instance is one Core function recording its sizes, with consecutive identities in order, a size's name as an `Int` literal node, and calls referring to the instances they name. | Unit and CLI observation |
| `S3M-EVAL-01` | Sections 8 and 10 | Instances evaluate as the functions written out, and `orangec eval` evaluates and names every instance of each root `spec` without value parameters; SHA-256, HMAC-SHA-256, and Poly1305 written once for every length match FIPS 180-4, RFC 4231, and RFC 8439. | CLI and unit |
| `S3M-RES-01` | Section 11 | Size parameters, instances, sizes, and sized calls, their events, allocations, nesting, height, and stack use are bounded as specified, the limits of 4 sizes, 256 instances, and bounds of 65536 are exact, and foreign spans are `ORC0210`. | Generated CLI and unit |
| `S3M-COMPAT-01` | Section 13 | S3l sources keep their meaning, Core values, and output bytes, with only the specified diagnostic changes. | CLI and unit |
| `S3M-DETERMINISM-01` | Section 12 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 13. Relationship to S3l

When OEP-0016 is accepted, this document replaces these clauses of
`BYTES_2026.md` and the documents it extends:

- the function declaration of `EXPRESSIONS_2026.md`, for size parameters, by
  sections 3 and 4;
- the array type of `ARRAYS_2026.md`, the fill and loop productions of
  `LOOPS_2026.md`, and the call productions of `EXPRESSIONS_2026.md` and
  `MODULES_2026.md`, by section 3;
- the name resolution of `BINDINGS_2026.md`, for size parameters, by sections
  4 and 5;
- the checking of a function's signature and body, once for each instance, by
  section 6, and of calls, by section 7; and
- the Core function record and the display of evaluated values, by sections 8
  and 9.

Every source that S3l accepts has no size parameter and writes every length
and loop bound as an integer token, so S3m accepts it with the same Core
values and the same output bytes. A source that S3l rejects gets the same
diagnostics, with these exceptions:

- square brackets after a function's name in a declaration were `ORC0101`;
  they are now size parameters;
- a name or a parenthesized expression as an array length, a fill length, or
  a loop bound was `ORC0101`; it is now a size, rejected by section 5 unless
  it is built from integer literals and size parameters, and the message for
  anything else after `^` is now "expected a length after `^`" where it was
  "expected an integer length after `^`";
- a name followed by bracketed size tokens and `(` was an index followed by an
  unexpected `(`, `ORC0101`; it is now a sized call, rejected by section 7
  when the function has no size parameters.

`orangec lex` is unchanged.

## 14. Explicit non-claims and future work

This slice defines no symbolic reasoning about sizes: nothing is proved for
all values of a size, only for each value in its finite range, one instance
at a time. It defines no size inferred from anything but the lengths of a
call's arguments, no size parameters on `type` declarations or moduli, no
sizes as run-time values beyond constants, no sets of values other than
ranges, no instances beyond 256 per function, no empty arrays and so no
instance with a length of 0, no recursion over sizes, and none of the
exclusions of `BYTES_2026.md` section 14 that this document does not lift.

An instance is checked and evaluated as a function; nothing here fixes how a
backend would compile a family of instances, whether as copies, as one
function with a length argument, or otherwise, and the reference evaluator is
not constant-time (`CONDITIONS_2026.md` section 15).

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
