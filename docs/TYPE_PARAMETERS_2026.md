# Orange 2026 type parameters specification

Status: proposed S3o semantics under OEP-0018, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3o of Orange 2026: **type parameters**, which
let one `spec` stand for a finite family of functions, one for each type in a
list, such as exponentiation in each of the prime fields of Curve25519,
Poly1305, ML-KEM, and ML-DSA, or the round of SHA-256 and SHA-512 on words of
either width; and **typed calls**, which name one member of the family by its
types or let the call's arguments and place choose it. It is a delta over the
proposed S3n rules in [`ORDER_2026.md`](ORDER_2026.md), which are a delta over
[`SIZES_2026.md`](SIZES_2026.md) and the documents it extends. Everything
those documents define and this one does not mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0018](governance/oeps/OEP-0018-orange-2026-type-parameters.md), which
requires OEP-0017. At that point it replaces the S3n clauses listed in
section 12. Until then, the compiler behavior it describes exists so that the
proposal can be reviewed against running code, and it establishes no accepted
language meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`LENGTHS_2026.md`](LENGTHS_2026.md), proposed under OEP-0019, extends this
> document with arrays of up to 65,536 elements and with evaluation controls:
> a type parameter may list array types of any admitted length, a function
> still has at most 256 instances, and `orangec eval --spec pow` evaluates
> every instance of `pow` alone. Every source this document accepts keeps its
> meaning under it.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Cryptography computes the same way in many types. Square-and-multiply raises
an element to a power in any field; Fermat's little theorem inverts in any
field of prime order; Euler's criterion tells squares from nonsquares in any
of them. RFC 7748 does it modulo 2^255 − 19, RFC 8032 also modulo the order
of the Curve25519 subgroup, RFC 8439 modulo 2^130 − 5, and FIPS 203 and 204
modulo 3329 and 8380417. FIPS 180-4 defines Ch, Maj, and the round of
SHA-256 and SHA-512 by the same formulas on words of 32 and of 64 bits.
Through S3n each of these had to be written once for each type, word for
word the same.

S3o writes it once. A function declares a **type parameter** in square
brackets, listing the types it is written for, and uses its name where a type
is written:

```orange
type F = Mod[(1 << 255) - 19];
type P = Mod[(1 << 130) - 5];
type Q = Mod[3329];

// The modulus m: one more than the least residue of -1.
spec modulus[K in {F, P, Q}]() -> Int {
  let minus_one: K = 0 - 1;
  (minus_one as Int) + 1
}

// x^e for 0 <= e < 2^256, squaring x once for each bit of e.
spec pow[K in {F, P, Q}](x: K, e: Int) -> K {
  let (square: K, power: K, rest: Int) =
    for i in 0..256 with (square: K, power: K, rest: Int) = (x, 1, e) {
      (square * square, if (rest % 2) == 1 { power * square } else { power }, rest / 2)
    };
  power
}

// Fermat's little theorem, in every field at once.
spec inverse[K in {F, P, Q}](x: K) -> K { pow(x, modulus[K]() - 2) }

spec kem_root() -> Q { pow(17, 128) }
```

`pow` stands for three functions, `pow[F]`, `pow[P]`, and `pow[Q]`, one for
each listed type. Each is an **instance**: the function with its type
parameter replaced by that type. The compiler checks every instance before
anything runs, exactly as it would check the function written out with that
type, so every operator, conversion, literal, and call is checked in every
field. A call names its instance by its types, as `modulus[K]()`, or lets its
arguments choose: `pow(x, ...)` with `x` of type `K` calls the instance for
`K`. Where the arguments do not decide, the type the call's place expects
does: `pow(17, 128)` returns `Q` in `kem_root`, so it calls `pow[Q]`.

Nothing is generic at run time. A type parameter is not a type variable that
the checker reasons about; it is a type, different in each instance, and the
instances are finite in number. What is checked for `pow` is checked for each
of its three instances separately, as S3m checks each instance of a sized
function.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2, as S3n extends it, is
unchanged: types are resolved during analysis, from the source alone, and no
type depends on a value computed when a program runs. Every S3n source keeps
its meaning (section 12).

## 3. Grammar

S3o adds no token and no reserved word. `in` is a name except where S3m and
this document read it.

The size parameter production of `SIZES_2026.md` section 3 becomes:

```text
size_params  = "[" size_param ( "," size_param )* "]" ;
size_param   = IDENTIFIER "in" ( INTEGER ".." INTEGER | type_list ) ;
type_list    = "{" listed_type ( "," listed_type )* "}" ;
listed_type  = type | tuple_type ;
```

A **type parameter** is a size parameter whose bounds are replaced by a list
of one or more types in braces, as `K in {F, P, Q}`. A listed type is any
type a parameter may have: `Int`, `Bool`, a word, `Mod[m]`, the name of a
`type` declaration, an array type, or a tuple type. A function has at most
four parameters in brackets, sizes and types together.

A call's brackets hold its **entries**, one for each of the callee's
parameters in brackets, in order. The `sizes` production of
`SIZES_2026.md` section 3 is unchanged: every entry is parsed as an
expression, and section 6 says which expressions name types. The scan that
decides whether a name followed by `[` is a call (`SIZES_2026.md` section 3)
also admits `^` and, directly after a name, an integer in brackets, so that
`ch[Word[32]](e, f, g)` and `sum[Word[8]^4, 2](x)` are calls. It reads each
token at most twice over a whole source.

The parse errors are `ORC0101`, with the note "a type parameter is written
`K in {F, L}` and names each type its function is checked for, as in `spec
square[K in {F, L}](x: K) -> K { x * x }`":

| Form | Message | Label |
| --- | --- | --- |
| `spec f[K in {}]` | "expected a listed type after `{`" | found token |
| `spec f[K in {Int,}]` | "expected a listed type after `,`" | found token |
| `spec f[K in {Int Bool}]`, `spec f[K in {Int]` | "expected `,` or `}` after the listed type" | found token |
| `spec f[K in {1}]` | "expected an identifier for the listed type" | found token |
| `spec f[K in {Int} n in 1..2]` | "expected `,` or `]` after the type parameter" | found token |
| a fifth parameter in brackets | "a function has at most 4 size and type parameters", from its name through its `}` or second bound | "one parameter in brackets too many" |
| `impl f[K in {Int}]() {}` | "`impl` functions have no type parameters" | "type parameters are allowed only on typed `spec` functions" |

After an error inside a list, parsing resumes after its `}`, so the rest of
the function and the functions after it are parsed and reported on their own.
A function with only size parameters keeps the S3m messages: its fifth size
is "a function has at most 4 size parameters".

**Nesting and height.** A listed type counts as the same type written as a
parameter's type. A type written as a call's entry is an expression, so
`Word[32]` there opens one nesting level for its brackets, as an index does.

The syntax tree gains, on a function's parameter in brackets, its listed
types in source order, empty for a size parameter; the spans of its first and
second bound are then the spans of `{` and `}`, and its own span runs from its
name through `}`.

## 4. Type parameters and instances

A type parameter `K in {T1, ..., Tk}` takes each listed type in turn. Its
**value** in an instance is the position of its type in the list, counted
from 0.

**Listed types.** The listed types are resolved once, outside every instance
of the function, before any of its instances is checked, with every rule of
the earlier documents for a parameter's type:

- a listed type that does not resolve is reported as the same type would be
  as a parameter's type, such as `ORC0203` "unsupported listed type `H`" or
  `ORC0204` for `Word[7]`;
- a listed type that uses one of the function's sizes, as `Word[8]^n`, is
  `ORC0237`, as any size that is not built from integer literals is, with the
  added note "a listed type is one type for every instance of its function,
  resolved before any size has a value, so its lengths are written without
  sizes"; and
- a type listed twice, however it is spelled, is `ORC0241` (new), "`K` lists
  the type `Mod[3329]` twice", at the repetition, labeled "this is the same
  type as an earlier one", with the secondary label "first listed here" and
  the note "a type parameter lists each type once, so that each instance has a
  type of its own". Types are the same when they are equal as types: `Q` and
  `G` declared as `Mod[3329]` are one type.

**Names.** A type parameter's name is in scope, as a type, in the function's
parameter types, result type, and body: wherever a type is written, including
`let` annotations, loop accumulators, array element types (`K^n`), tuple
types, conversion targets (`x as K`, `x as big K`), and the entries of calls.
It is not a value.

- A type parameter named as a built-in type, `Int`, `Bool`, `Word`, or `Mod`,
  is `ORC0233`, "`Int` is a built-in type", labeled "a type parameter cannot
  name a built-in type"; one named as a `type` declaration of the module is
  `ORC0233`, "duplicate type name `F`", labeled "this type parameter repeats
  a declared type's name", with the secondary label "the `type` declaration is
  here". Both have the note "a type parameter names a type of its own, so its
  name is not a built-in type's or a `type` declaration's".
- Two parameters in brackets of one name, a size and a type or two types, are
  `ORC0218`, "duplicate parameter `K`", at the repetition, labeled "this name
  is already a parameter in brackets", with the secondary label "first
  parameter in brackets is here" and the note "each size and type parameter
  in a function's brackets has a name of its own".
- Types and values have separate names, as a `type` declaration's name and a
  parameter's may already be the same. A parameter, binding, loop index, or
  accumulator may therefore have a type parameter's name, as `spec
  f[K in {Q}](K: Q) -> Q { K }`, where `K` in the body is the parameter.
- A type parameter's name where a value is expected, as `spec f[K in {F,
  Q}](x: K) -> K { K }`, is the `ORC0211` of an unknown name, with the added
  note "`K` is a type parameter: it names a type, not a value, so it is written
  where a type is, as in `let x: K = 0;`".

A modulus is not a type position: `Mod[K]` is `ORC0232`, as any modulus that
is not built from integer literals is.

**Instances.** A function's instances are the tuples of values of all its
parameters in brackets, sizes and types together, one value from each, in the
order of `SIZES_2026.md` section 4: the first parameter changes slowest.
`spec zeros[W in {Word[8], Word[16]}, n in 1..3]` has the instances
`zeros[Word[8], 1]`, `zeros[Word[8], 2]`, `zeros[Word[16], 1]`, and
`zeros[Word[16], 2]`, in that order. An instance is named as a call names it,
each type written as the function lists it, with every run of spaces written
as one space.

A function has at most 256 instances: more, as `spec many[K in {Int, Bool}, n
in 0..200]` with 400, is `ORC0238`, "`many` has 400 instances, but a function
has at most 256", from its first parameter in brackets through its last,
labeled "too many instances", with the note "a function has one instance for
each combination of its sizes' values and its type parameters' types, at most
256 in all".

A function's parameters in brackets are checked in order, each size's range
or each type parameter's name and listed types, and then each name against
the earlier ones; the count of instances is checked only when every range and
list is valid and no name repeats. A function any of whose parameters in
brackets is in error has no instances: its body is not checked, it
contributes no Core, and a call to it, whether it names an instance or leaves
its arguments to choose one, is not reported again.

## 5. Checking instances

A function with type parameters is checked once for each of its instances,
in order, as a sized function is (`SIZES_2026.md` section 6). Each instance is
checked as the function written out with each type parameter's name replaced
by its type in that instance, and every rule of the earlier documents applies
to it: in `spec remainder[K in {Word[8], Q}](x: K) -> K { x % x }`, the
instance `remainder[Word[8]]` is valid and `remainder[Q]` is `ORC0215`, "`%`
is not defined for `Mod[3329]`".

The first instance of a function in error is the only one reported. Each of
its diagnostics, apart from those that report an exhausted budget, has the
note "in the instance `remainder[Q]`, the first of `remainder` in error: a
function is checked once for each type of its type parameters", or, for a
function with sizes too, "... a function is checked once for each value of
its sizes and each type of its type parameters". The instances after it are
not checked.

Before any body is checked, each function's instances receive their
signatures, resolved without reporting, as for sized functions.

## 6. Calls

A call of a function with type parameters names one of its instances.

**Calls that write their entries.** `f[e1, ..., ek](args)` gives one entry
for each of `f`'s parameters in brackets, in order: a size for each size
parameter, evaluated as `SIZES_2026.md` section 7 says, and a type for each
type parameter. A type entry is one of:

- `Int` or `Bool`;
- `Word[8]`, `Word[16]`, `Word[32]`, or `Word[64]`, the width an integer
  token;
- an array `T^n`, as `Word[8]^4`, where T is any entry of this list that
  names a type other than an array or a tuple and n is an integer token from
  1 through 256;
- the name of a `type` declaration of the calling module, which may name any
  type, `Mod[m]`, arrays, and tuples included; or
- a type parameter of the calling function, which is its type in the caller's
  instance.

The type an entry names is matched against the callee's listed types by
equality: `field::cube[Kyber](5)`, with `Kyber` declared as `Mod[3329]` in the
caller and `Q` as `Mod[3329]` in the callee, calls `cube[Q]`.

- A number of entries other than the callee's parameters in brackets is
  `ORC0239`, "`square` takes 1 type in brackets, but this call gives 2" (or
  "takes 1 size and 1 type", "takes 2 types"), at the call, labeled "wrong
  number of entries in brackets", with the note "a function with type
  parameters is called with one entry in brackets for each of its sizes and
  types, in order, as in `pow[F](x, e)`, or without brackets when its
  arguments choose one instance".
- An entry that names no type where a type parameter stands, as `square[x](x)`
  or `square[3](x)`, is `ORC0241`, "`square` takes a type for `K` here",
  labeled "this is not a type", with the note "a type in a call's brackets is
  `Int`, `Bool`, `Word[n]`, an array of one of them such as `Word[8]^4`, the
  name of a `type` declaration, or a type parameter of the calling function".
- A type that the parameter does not list is `ORC0241`, "`square` is defined
  for `K` in {F, Q}", labeled "this type is not listed", with the note "a call
  gives each type parameter one of the types it lists, in brackets, as in
  `pow[F](x, e)`".
- A type entry where a size parameter stands is a size that is not built from
  integer literals, `ORC0237`, as in S3m.

The entries are checked from left to right, and the first in error is the
only one reported.

**Calls that write none.** A call without brackets of a function with type
parameters calls the one instance that **fits** it, found without reporting:

1. Each argument's type is found as the type of a conversion's operand is
   (`ORDER_2026.md` section 4): the type of its first typed leaf, with array
   literals and fills typed by their elements and their length. An argument
   whose type is not found this way, but which is an array literal, a fill, a
   join of them, or an update of one, has the length of its syntax, as `[1, 2,
   3]` has 3. Each argument is read once.
2. An instance fits when each of its parameters has the type found for the
   argument in its place, or, for an argument with only a length, is an array
   of that length. An argument with neither fits every instance.
3. When exactly one instance fits, the call calls it. When several fit, the
   call calls the one among them whose result type is the type the call's
   place expects, if exactly one has it.

The place of a call is where its value is checked against a type: an
argument of another call, a `let` with a type, a function's result, an
operand of an arithmetic operator whose other operand gives the type, and so
on. A call is also found without reporting where its own type is needed to
check its place, as the operand of a conversion or a comparison, the base of
an index, a slice, or a projection, or an operand of `++`: there the place
gives no type, and several fitting instances are reported as below.

- Several instances that fit, and not exactly one of them with the expected
  result, are `ORC0239`, "this call fits more than one instance of `square`,
  among them `square[F]` and `square[Q]`", naming the first two, at the call,
  labeled "write the types in brackets" (or "write the sizes and types in
  brackets").
- No instance that fits is `ORC0241`, "no instance of `square` takes arguments
  of these types", at the call, labeled "an argument of type `Word[32]` is
  given" (or "arguments of types ... are given", or "no instance fits these
  arguments"), with the note "`square` is defined for `K` in {F, Q}".

Both have the note "a call that writes no brackets calls the one instance of
its function whose parameters have its arguments' types, and among several,
the one whose result has the type its place expects; any other call writes
its types in brackets, as in `pow[F](x, e)`". A call whose number of
arguments differs from the function's parameters is checked against the first
instance, where it is `ORC0213` as before. A function with only size
parameters keeps the S3m rule of fitting by its arguments' lengths.

**The instance's signature.** Once named, the call is checked as a call of
that instance, as in S3m: its arguments against the instance's parameter
types, and its value has the instance's result type, so a call whose instance
gives another type than its place expects is `ORC0214`, as `square(x)` with
`x: F` where `Q` is required. A qualified call `m::f[T](x)` or `m::f(x)` names
an instance of `f` in the used module `m` the same way, the entries' type
names resolved in the calling module.

**Cycles.** The call graph is a graph of instances, as in S3m, so `f[F]`
calling `f[Q]` is no cycle.

## 7. Meaning

An instance means what the function written out with its types means. A call
of an instance evaluates its arguments from left to right and applies that
instance.

`orangec eval` evaluates every instance of each `spec` of the root module
without value parameters, in order, and names each as a call names it:

```text
fields::modulus[F]: Int = 57896044618658097711785492504343953926634992332820282019728792003956564819949
fields::modulus[Q]: Int = 3329
types::zeros[Word[16], 2]: Word[16]^2 = [0x0000, 0x0000]
fields::kem_root: Mod[3329] = 3328
```

## 8. Typed Reference Core

Each instance is one Core function, as in S3m. It records its function's name,
the values of its parameters in brackets in declaration order (for a type
parameter, the position of its type in the list), and its instance's name as a
call writes its brackets, `[F]` or `[Word[16], 2]`, empty for a function
without them. An instance's Core is the Core of the function written out with
its types: every type in it is concrete, and a type parameter leaves no trace
in any node. Core gains no node kind, and a function without type parameters
has exactly its S3n Core.

The Core is still internal and noncanonical, with no encoding, digest, or
proof role.

## 9. Evaluation

Types cost nothing at run time: an instance's steps are those of the function
written out with its types. The per-source budget of 1,048,576 steps covers
every instance evaluated, and each evaluated value records the values and the
name of its instance.

## 10. Resource limits and failure

The S3n budgets remain. S3o adds or refines the following.

- A function has at most 4 parameters in brackets, sizes and types together,
  and 256 instances, counted over all of them.
- Each parameter in brackets costs two semantic events when its function is
  checked, one for its range or list and one for its name; each listed type
  costs what the same type costs as a parameter's type; and each type entry
  of a call costs one semantic event each time the call is checked.
- Finding a call's instance without reporting reads each argument once, for
  its type or for its length, and each operand of `++` and each branch of a
  conditional once, for its length and its element type together. Calls
  nested in each other's arguments therefore cost work linear in their depth
  at each level that fits one, not work that doubles with each level.
- An allocation failure while resolving a type parameter's listed types is
  `ORC0209`, labeled "type parameter storage allocation failed", and while
  recording an evaluated instance's name `ORC0301`, "reference evaluation
  result allocation failed", labeled "evaluated function instance storage
  could not be reserved". Neither gives partial output.

The deepest sources the limits admit, including 63 calls without brackets
nested in each other's arguments, each fitting its instance by its argument's
type, by its argument's elements and length, or by the type its place
expects, and 63 calls that name their instances nested in each other, must
parse, analyze, and evaluate within 1 MiB of native stack. A syntax tree whose
spans, including those of every listed type, do not all belong to the source
it is supplied with is `ORC0210` for that source, and nothing is checked.

## 11. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged: instances are checked, numbered, and evaluated in one order, and
listed types are resolved in source order, so the same source gives the same
diagnostics, Core, and output bytes. The S3o conformance runner
(`compiler/crates/orangec/tests/s3o_conformance.rs`) parses this index and
requires exact agreement with its evidence map, under the same rules as the
S3b through S3n runners.

### S3o conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3O-SYNTAX-01` | Section 3 | Type parameters parse as braced lists of types with exact spans, calls' brackets admit type entries including `Word[n]` and `T^n`, and malformed lists are `ORC0101` with the specified messages, recovering after the list's `}`. | CLI and parser unit |
| `S3O-DECL-01` | Section 4 | Listed types resolve outside every instance, without sizes and each once (`ORC0241`); a type parameter's name is no built-in or declared type's (`ORC0233`) and no other bracket parameter's (`ORC0218`), and is a type, not a value; instances are the product of all parameters in brackets, first slowest, at most 256 (`ORC0238`). | CLI and unit |
| `S3O-INSTANCE-01` | Section 5 | Every instance is checked with its types wherever a type is written, and only the first instance of a function in error is reported, named in a note. | CLI and unit |
| `S3O-CALL-01` | Section 6 | A call's entries name one instance: each type entry a listed type (`ORC0241`), counted with the sizes (`ORC0239`), from the caller's types and declarations, across modules as within one. | CLI and unit |
| `S3O-FIT-01` | Section 6 | A call without brackets fits exactly one instance by its arguments' types or literal lengths and, among several, by the type its place expects (`ORC0239`, `ORC0241`), and is reported where its place needs its type. | CLI and unit |
| `S3O-CORE-01` | Section 8 | Each instance is one Core function recording its parameters' values, a type as its position, and its instance's name, with concrete types throughout. | Unit and CLI observation |
| `S3O-EVAL-01` | Sections 7 and 9 | Instances evaluate as the functions written out, and `orangec eval` evaluates and names every instance of each root `spec` without value parameters; field arithmetic in five prime fields and SHA-256 and SHA-512 sharing their round match RFC 7748, RFC 8032, RFC 8439, FIPS 203, FIPS 204, and FIPS 180-4. | CLI and unit |
| `S3O-RES-01` | Section 10 | Parameters in brackets, instances, listed types, and type entries, their events, allocations, nesting, stack use, and the work of fitting nested calls are bounded as specified; the limits of 4 parameters and 256 instances are exact, and foreign spans are `ORC0210`. | Generated CLI and unit |
| `S3O-COMPAT-01` | Section 12 | S3n sources keep their meaning, Core values, and output bytes, with only the specified diagnostic changes. | CLI and unit |
| `S3O-DETERMINISM-01` | Section 11 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 12. Relationship to S3n

When OEP-0018 is accepted, this document replaces these clauses of
`ORDER_2026.md` and the documents it extends:

- the size parameter production and the call scan of `SIZES_2026.md`
  section 3, by section 3;
- the rules of size parameters and instances of `SIZES_2026.md` section 4,
  for type parameters, by section 4;
- the resolution of type names of `MODULAR_2026.md`, for type parameters, by
  section 4;
- the checking of instances and of calls of `SIZES_2026.md` sections 6 and 7,
  for functions with type parameters, by sections 5 and 6; and
- the Core function record and the display of evaluated values, by sections 7
  and 8.

Every source that S3n accepts has no type parameter, so S3o accepts it with
the same Core values and the same output bytes. A source that S3n rejects
gets the same diagnostics, with these exceptions:

- braces after a size parameter's `in` were `ORC0101`, "expected the size's
  first bound"; they are now a type parameter;
- a name followed by brackets holding `^`, or a name and an integer in
  brackets, and then `(`, as `g[a[1]](b)`, was an index, and the `(` after it
  was `ORC0101`, "expected `}` after the body expression"; it is now a call,
  whose entries section 6 checks, so `g[a[1]](b)` of a sized `g` is
  `ORC0237`;
- a fifth size parameter of a function that also has a type parameter is "a
  function has at most 4 size and type parameters", labeled "one parameter in
  brackets too many"; and
- a call of a sized function that fits by lengths, or a call that fits no
  instance, is found the same way, but the checker reads each argument and
  branch once: a source whose calls nest in conditional arguments, which
  took time doubling with each level, is now checked in time linear in its
  depth, with the same diagnostics.

`orangec lex` is unchanged.

## 13. Explicit non-claims and future work

This slice defines no type variables and no reasoning over all types: nothing
is proved for a type that a function does not list, only for each listed type,
one instance at a time. It defines no bounds or classes of types (such as
"any field"), no type parameters on `type` declarations, no types computed
from sizes or values, no types inferred from anything but a call's arguments
and its place, no residue type written in a call's brackets except through a
`type` declaration's name, no tuple types written there except the same way,
and none of the exclusions of `ORDER_2026.md` section 12 that this document
does not lift.

An instance is checked and evaluated as a function; nothing here fixes how a
backend would compile a family of instances, and the reference evaluator is
not constant-time (`CONDITIONS_2026.md` section 15).

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
