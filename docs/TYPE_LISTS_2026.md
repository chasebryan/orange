# Orange 2026 named type lists

Status: proposed S3x semantics under OEP-0028, in owner review; not accepted

Edition: `2026`

This document defines S3x: a list of types named once and reused by several
functions. It builds on the type parameters of
[`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md). A `types` declaration
is not a type. Nothing is generic at run time: each function is still checked
once for each listed type, as if that type had been written in its brackets.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Declaration

A module may declare type lists after its `use` declarations and before its
functions. `type` aliases and `types` lists may stand in either order. The
form is `types NAME = {TYPE, ...};`, with one or more types separated by
commas and no trailing comma. `types` is that word only where a `type`
declaration could begin. Elsewhere it is a name.

A module has at most 64 `types` declarations. A list has at most 256 types,
the most instances one function may have. The name is not a built-in type,
not the name of a `type` declaration, and not the name of another list.
Each listed type resolves where the list is written, using built-in types
and `type` names declared before it, and each resolved type appears once.

## 2. Use

A type parameter may name a list instead of writing braces: `K in Fields`.
The function is checked once for each type `Fields` lists, in that order,
with `K` standing for that type, exactly as `K in { ... }` with those types
written out. A call still names a concrete type, `f[P](x)`, or lets the
arguments and the place choose among the listed types. Several functions
may name the same list. A list may stand beside size parameters. The
product of the sizes and the list's length is still at most 256 instances.

An identifier after `in` names a list unless `..` follows, in which case
the bounds are a size and must be integer literals.

## 3. What a list is not

A list name is not a type. It must not appear as a parameter type, a result,
a `type` alias's right-hand side, or an entry of a list. A type parameter
must not take a list's name. A list name from another module is not in
scope; a caller names a concrete type, and fitting uses the resolved type.
`K in P` where `P` is a `type` declaration is rejected; the braces `K in {P}`
remain the way to list one type.

## 4. Diagnostics

| Fault | Code |
| --- | --- |
| A `types` declaration is malformed, follows a function, or a `use` follows a list | `ORC0101` or `ORC0103` |
| A list names a built-in, repeats a type or list name, is unknown, is used as a type, or a type is used as a list, or the list has more than 256 types | `ORC0244` |
| A list contains the same type twice | `ORC0241` |
| A function that uses a well-formed list still has more than 256 instances | `ORC0238` |

A list that failed is reported at the declaration. Functions that name it
are not reported again for that failure, and they have no instances.

## 5. Core and compatibility

The Core of `spec f[K in Fields]` is the Core of `spec f[K in {T, ...}]`
with the list's types written out: one function for each type, named by
that type as the list writes it. No new Core node is added. A source that
S3u accepted has no `types` declaration and no type parameter whose `in`
is followed by a name, so its Core, values, and output are unchanged.
`K in {F, L}` is unchanged.

## 6. Non-claims

This slice adds no type variable, no bound that ranges over every type, no
parameterized `type` declaration, no import of a list, and no proof. It
makes no timing, leakage, cryptographic-security, or production claim.
Accepting it is the owner's decision through OEP-0028.

### S3x conformance rule index

The runner `compiler/crates/orangec/tests/s3x_conformance.rs` parses this
index and requires exact agreement with its executable evidence map.

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3X-01` | Section 1 | A `types` declaration names one or more types, and each function that uses it is checked once per type, in order, under the type's spelling. | CLI |
| `S3X-02` | Section 2 | A call names a listed type or fits one by its arguments and the type its place expects, including beside a size parameter. | CLI |
| `S3X-03` | Section 3 | A caller in another module fits the callee by the resolved type and does not name the list. | CLI |
| `S3X-04` | Sections 1 and 3 | A malformed list, a list after a function, a `use` after a list, an unknown list, a type used as a list, and a list used as a type are rejected. | CLI and generated CLI |
| `S3X-05` | Section 1 | A repeated type is `ORC0241` at the list; 256 types are admitted and 257 are rejected; a longer product of instances is `ORC0238`. | CLI and generated CLI |
| `S3X-06` | Section 5 | An inline `K in {F, L}` keeps its value, and a source with no `types` declaration still evaluates. | CLI and generated CLI |
| `S3X-07` | Section 1 | `orangec fmt --check` accepts the corpus that parses, and `orangec doc` names each list. | CLI and generated CLI |
| `S3X-08` | Section 5 | Repeated identical valid or rejected inputs have identical status, diagnostics, and output bytes. | CLI and generated CLI |
