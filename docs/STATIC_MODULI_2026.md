# Orange 2026 static modulus specification

Status: proposed S3t semantics under OEP-0024, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-10-01

This document defines S3t: modulus expressions evaluated separately for each
finite size instance. It is a delta over proposed S3s in
[`NESTED_ARRAYS_2026.md`](NESTED_ARRAYS_2026.md), the size rules in
[`SIZES_2026.md`](SIZES_2026.md), and the constant-modulus rules in
[`MODULAR_2026.md`](MODULAR_2026.md). Clauses not changed here retain their
earlier meaning. Implementation supplies provisional evidence for owner review;
it does not accept [OEP-0024](governance/oeps/OEP-0024-orange-2026-static-moduli.md)
or choose a semantic stratum, proof foundation, or target.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. Static expressions and scope

A sized `spec` may use its own size parameter names in `Mod[e]`:

```orange
spec add[bits in 5..7](a: Mod[(1 << bits) - 19],
                       b: Mod[(1 << bits) - 19]) -> Mod[(1 << bits) - 19] {
  a + b
}
```

The two concrete instances use moduli 13 and 45. A modulus need not be prime.
The existing total modular operations retain their meanings for both instances.

The static expression vocabulary is integer literals, a function's own size
names, parentheses, and the binary operators `+`, `-`, `*`, and `<<`.
Existing signed integer literals retain their parsing rule. No general
unary-negation, division, remainder, call, comparison, rotation, or other
expression is admitted as a modulus expression. A runtime parameter, `let`
binding, loop index, type parameter, function, or unknown name is not a static
size name. S3t adds no token, reserved word, or parser production. The bounded
call/index disambiguation recognizes complete direct `Mod[e]` type arguments,
including `<<`, before applying these static expression checks.

Every own declared size parameter is in scope regardless of its order among
the function's parameter brackets. The same static scope applies in the
function signature and body: parameter
and result types, typed `let` bindings, loop accumulators, conversion targets,
and direct explicit type arguments such as `identity[Mod[n]](x)` may use its
size names. Module-level aliases remain concrete types with constant modulus
expressions. A finite type-parameter list remains independent of all size
parameters; `T in {Mod[n]}` is rejected even if `n` is declared before `T`.
S3t does not add parameterized aliases or mutually dependent parameter lists.

## 2. Exact arithmetic and modulus bounds

Each static expression must evaluate as an exact mathematical integer.
Its intermediate magnitudes obey the existing 16,384-bit integer limit.
Each shift amount must be from 0 through that limit, and its result must
also satisfy the magnitude limit. A negative or excessive static shift is
rejected; it does not use the runtime computed-shift fallback of S3r.

Each concrete modulus must be from 2 through 2^521 − 1, inclusive.
Zero, one, a negative result, or an excessive modulus is rejected before
reference evaluation. An unsupported expression, out-of-scope name, invalid
modulus or static shift is `ORC0232`. Existing integer-magnitude and resource
diagnostics (`ORC0205` and `ORC0209`) retain their meaning when their respective
bounds are exhausted.
No arithmetic overflow may admit a modulus or wrap it into a valid one.

## 3. Finite instances and cached types

All declared concrete size/type combinations must be checked eagerly, as
if the function were written separately for each combination. A function
with one invalid concrete instance is rejected even if no call selects it.
The existing finite range, size-value and specialization budgets remain:
the combined instance count is at most 256. This slice adds no universal
typechecking over an unbounded size domain.

The modulus is computed in the current instance's size environment. A cached
signature, body annotation, conversion or explicit type argument must not
reuse a modulus computed for another instance. Invalid-instance diagnostics
must identify the offending instance using the existing specialization note.
An uncalled definition receives the same checks as a called definition.

## 4. Concrete domain identity

Every admitted instance lowers to the existing concrete `Mod[m]` type with
its exact evaluated modulus. Different expressions with the same value name
the same domain. Different evaluated moduli name different domains even if
their value ranges overlap, their sizes match, or both moduli are prime.

Parameters, results, calls, literals, arithmetic, comparisons and equality
retain exact residue-domain typing. A value of `Mod[13]` must not be accepted
where `Mod[45]` is required. No implicit residue conversion or embedding is
introduced. Explicit `as` conversions retain S3c/S3i semantics and are checked
against the concrete target in their current instance.

## 5. Calls and fitting

Explicit size arguments select the existing concrete instance, after which
parameter and result checking uses its evaluated modulus. For a call without
explicit size arguments, argument and expected-result types participate in
the existing finite fitting procedure. For a callee with only size parameters,
this argument/expected-result fitting applies when its signature contains a
size-dependent modulus. Ordinary size-only callees retain their prior
array-length fitting and diagnostics. A unique fitting instance is selected;
no fit or several
fitting instances retain their existing rejection and diagnostics (`ORC0238`
for no fitting size instance, `ORC0239` for several).

The analyzer must not fit a call using an unresolved modulus expression or
equate distinct instances merely because both contain residues. An explicit
type argument `Mod[e]` is evaluated in the caller's current size environment
before the callee's finite type list is fitted. Ordinary indexing retains its
meaning; recognizing an explicit type argument must not reinterpret a runtime
array index as a modulus declaration.

## 6. Arrays, matrices, and tuples

A concrete size-dependent residue domain may occur wherever a constant
residue domain could occur: as an array leaf, inside a tuple, or within the
existing structural operations. For example:

```orange
spec pair[n in 2..4](x: Mod[n]) -> (Mod[n]^2, Mod[n]) {
  ([x, x], x)
}
```

Each instance checks scalar domain, rank, every array axis and total scalar
count under the existing rules. Specializing a type parameter to a concrete
row or matrix retains S3s's structural boundaries. S3t adds no new array rank,
tuple nesting, symbolic row alias, implicit flattening, or shape constraint.

## 7. Bindings, conversions, branches, and loops

A typed binding or loop accumulator whose type contains `Mod[e]` must resolve
e in the current size instance. The initializer, replacement state, branch
results and function result must fit that exact concrete type. Nested lexical
scopes do not introduce another static environment or allow runtime values
into a modulus expression.

Byte-order conversions keep their existing source/target and rank rules.
A conversion into a residue uses the concrete modulus of the selected
instance; a matrix is not flattened implicitly. Modular division remains
total as specified by S3i; introducing size-dependent moduli does not change
the treatment of zero or noninvertible divisors.

## 8. Imported modules

Qualified calls and imported definitions obey the same specialization rules
as local calls. A callee's own size names resolve to its concrete instance,
while a caller's explicit `Mod[e]` argument resolves in the caller's instance.
Names must not leak between modules or between unrelated functions. A cached
imported instance retains its exact concrete domain and source identity.
The existing acyclic module graph and bounded linking rules remain.

## 9. Failure and resource accounting

Static checking precedes reference evaluation and must fail closed on any
invalid instance, domain mismatch, ambiguity or exhausted budget. No partial
Core or successful value output may be exposed for a failed program. Static
expression arithmetic is subject to the existing bounded semantic work and
integer checks; adding a size name does not bypass those limits.
Resolution charges expression parts and integer-literal prefix/significant
digits under the existing semantic-event budget each time it resolves a
signature or body type position.

The public Core type representation, node kinds, evaluation-step costs and
CLI interface remain unchanged. `Mod[e]` does not survive in Core as a runtime
symbolic modulus: it is the concrete `Mod[m]` already supported by the
evaluator. Equal concrete programs retain their existing evaluation costs.

## 10. Compatibility and claim boundary

S3s programs without newly admitted static size names retain their types,
values, diagnostic boundaries and reference-evaluation costs. Constant
modulus expressions continue to obey their existing vocabulary and bounds.
Their original diagnostic notes retain exact text, including constant faults
inside sized definitions. The original note for a call entry that is not a
type also remains unchanged. The static-scope modulus note, which explains
own instance sizes and the concrete alias/list boundary, applies only to new
dependent-modulus faults and faults in the direct `Mod[e]` type-argument path.
Previously rejected references to a function's own finite size names become
valid only in the positions defined here and only when every instance passes.

Finite exhaustive specialization is implementation evidence, not a universal
proof about a generic program. No primality test, field proof, refinement
contract, theorem, certificate, authoritative checker, native layout, ABI,
leakage result, cryptographic conformance or release is added. A later checked
implementation must establish its own obligations under accepted S4–S7
boundaries.

## Bounded normative rule index

`compiler/crates/orangec/tests/s3t_conformance.rs` binds the following exact
rule IDs to executable CLI evidence. The index and runner must agree; the
negative corpus is part of the proposed semantic boundary.

| Rule | Normative subject | Evidence class |
| --- | --- | --- |
| S3T-01 | Static modulus vocabulary and scope, including excluded dependent type lists | CLI and generated CLI |
| S3T-02 | Exact static arithmetic, shift, intermediate and concrete modulus bounds | CLI and generated CLI |
| S3T-03 | Eager finite instantiation and instance-local cached types | CLI and generated CLI |
| S3T-04 | Exact evaluated residue-domain identity and mismatch rejection | CLI and generated CLI |
| S3T-05 | Explicit calls, argument/expected-result fitting and explicit type arguments | CLI and generated CLI |
| S3T-06 | Structural arrays, matrices and tuples with concrete residue leaves | CLI and generated CLI |
| S3T-07 | Bindings, conversions, branches and loop accumulator types | CLI and generated CLI |
| S3T-08 | Imported-module specialization and static scope separation | CLI |
| S3T-09 | Fail-closed invalid instances and retained resource/Core/cost boundaries | CLI and generated CLI |
| S3T-10 | Compatibility, finite-check interpretation and unchanged non-claims | CLI and generated CLI |
