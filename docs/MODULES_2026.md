# Orange 2026 modules specification

Status: proposed S3h semantics under OEP-0011, in owner review; not accepted

Edition: `2026`

Snapshot: 2026-09-30

This document defines slice S3h of Orange 2026: programs of more than one
module, in which a module names the modules it uses and calls their typed
`spec` functions by module name, as in `sha256::compress(h, block)`. It is a
delta over the proposed S3g rules in [`LOOKUPS_2026.md`](LOOKUPS_2026.md),
which are a delta over [`CONDITIONS_2026.md`](CONDITIONS_2026.md) and the
documents it extends. Everything those documents define and this one does not
mention is unchanged.

It is proposed, not accepted. It becomes normative only when the owner accepts
[OEP-0011](governance/oeps/OEP-0011-orange-2026-modules.md), which requires
OEP-0010. At that point it replaces the S3g clauses listed in section 11.
Until then, the compiler behavior it describes exists so that the proposal can
be reviewed against running code, and it establishes no accepted language
meaning. It accepts no D-004 candidate.

> [!NOTE]
> [`MODULAR_2026.md`](MODULAR_2026.md), proposed under OEP-0012, extends this
> document with the integers modulo a constant, as `Mod[(1 << 255) - 19]`, and
> with `type` declarations, which follow a module's `use` declarations. A
> residue type crosses a module boundary by its value, and a type name stays in
> its module. [`BLOCKS_2026.md`](BLOCKS_2026.md), proposed under OEP-0013, lets
> a loop's step and each branch begin with `let` bindings, and
> [`TUPLES_2026.md`](TUPLES_2026.md), proposed under OEP-0014, adds tuples,
> whose types cross a module boundary by their elements, and
> [`BYTES_2026.md`](BYTES_2026.md), proposed under OEP-0015, adds byte strings,
> joins, and slices, which cross a module boundary as the arrays they are, and
> [`SIZES_2026.md`](SIZES_2026.md), proposed under OEP-0016, adds sized
> functions, whose instances are called across a module boundary as
> `sha256::sha256(m)` or `m::f[2](x)`, and [`ORDER_2026.md`](ORDER_2026.md),
> proposed under OEP-0017, adds conversions in a byte order, which convert
> values of every module's types alike, and
> [`TYPE_PARAMETERS_2026.md`](TYPE_PARAMETERS_2026.md), proposed under OEP-0018,
> adds type parameters, whose instances are called across a module boundary as
> `field::cube[Kyber](5)`: a type entry names a type in the caller's module,
> matched by equality with the types the callee lists.
> [`LENGTHS_2026.md`](LENGTHS_2026.md), proposed under OEP-0019, lets `orangec
> eval --spec` evaluate only the named functions of the root module.
> [`TESTS_2026.md`](TESTS_2026.md), proposed under OEP-0020, adds known-answer
> tests, of which only the root module's are checked and run: a used module's
> tests are neither, until that module is the root. Every source this document
> accepts keeps its meaning under all nine.

The terms **must**, **must not**, and **may** are normative in this document.

## 1. The idea

Cryptography is built in layers. HMAC is defined over a hash function, HKDF
over HMAC, and an AEAD over a cipher and an authenticator. Through S3g an
Orange program was one module in one file, so every construction had to carry
its own copy of every primitive beneath it: the Daylight example holds
SHA-256, HMAC, HKDF, ChaCha20, and Poly1305 in one module of more than 500
lines.

S3h lets a module name the modules it builds on and call their functions by
module name, so each standard can be written once, in its own file, and read
on its own.

```orange
module hmac {
  use sha256;

  spec keyed(key: Word[8]^64, pad: Word[8]) -> Word[8]^64 {
    for i in 0..64 with b: Word[8]^64 = key { b with [i] = key[i] ^ pad }
  }

  spec block(d: Word[8]^32) -> Word[8]^64 {
    for i in 0..32 with b: Word[8]^64 = [0; 64] { b with [i] = d[i] }
  }

  spec mac(key: Word[8]^64, m: Word[8]^64, length: Int) -> Word[8]^32 {
    let inner: Word[32]^8 = sha256::compress(sha256::initial(), keyed(key, 0x36));
    let outer: Word[32]^8 = sha256::compress(sha256::initial(), keyed(key, 0x5c));
    let text: Word[8]^32 =
      sha256::digest(sha256::compress(inner, sha256::last_block(m, length, 64 + length)));
    sha256::digest(sha256::compress(outer, sha256::last_block(block(text), 32, 96)))
  }
}
```

With S3h, `compiler/fixtures/s3h/` holds SHA-256, HMAC, and HKDF as three
modules in three files, and a program that uses them reproduces the SHA-256
example of FIPS 180-4, test cases 1 and 2 of RFC 4231, and test case 1 of RFC
5869 byte for byte.

Three commitments shape every rule below.

- **A module means what it meant alone.** Each module is checked on its own,
  against the declarations of the modules it uses, and gives the same Core
  whoever uses it. There is no import of names into scope, no renaming, and no
  way for one module to change another's meaning.
- **Every dependency is written down.** A call into another module names that
  module, and the module declares each module it uses at its head, so a reader
  sees where every function comes from without searching.
- **Programs stay finite and acyclic.** The modules of a program use each
  other without cycles, so every program is checked in one pass from the
  modules that use nothing up to its root, and the call graph of a program is
  acyclic exactly when each module's own call graph is.

## 2. Scope and phase boundary

The phase boundary of `EXPRESSIONS_2026.md` section 2 is unchanged, with one
addition. A **program** is a **root module** together with every module it
uses, directly or through other modules. Lexing and parsing still apply to one
source at a time. Semantic analysis, which through S3g took one syntax tree,
now takes a program: the root's syntax tree and the syntax trees of the
modules it may use. Reference evaluation takes the program's linked Core and
evaluates the root.

Every S3g source keeps its meaning. A module with no `use` declaration is a
program of one module, checked exactly as S3g checks it.

## 3. Grammar

The module production of `EXPRESSIONS_2026.md` section 3 gains `use`
declarations before its functions, and a call may be qualified by a module
name.

```text
Module     = "module" Identifier "{" UseDecl* Function* "}" ;
UseDecl    = "use" Identifier ";" ;
Call       = ( Identifier "::" )? Identifier "(" Arguments? ")" ;
```

`use` is recognized by position, as `let`, `for`, and `if` are: it begins a
declaration only at the head of a module, before its first function, where no
other identifier may stand. Elsewhere it is an ordinary identifier, so a
parameter, binding, function, or module may be named `use`. The token `::` is
the S1 token `DOUBLE_COLON`, which no S3g form accepted.

A qualified name is always called: `sha256::initial` without `(` is `ORC0101`,
"expected `(` after the qualified function name", with the note "a name
qualified by its module is always called, as in `sha256::initial()`". A `use`
after a function is `ORC0103` at the word `use`, labeled "a `use` declaration
cannot follow a function", with the note "`use` declarations come first in a
module, before its functions". A `use` declaration names one module and ends
with `;`; anything else is `ORC0101`. A module may have at most 64 `use`
declarations; the 65th is `ORC0106`, "module has more than 64 `use`
declarations".

**Meaning.** `use m;` declares that this module calls functions of the module
named `m`. A call `m::f(a, b)` calls the typed `spec` function `f` of the
module `m`.

## 4. Modules and programs

The modules of a program are the root and every module the root reaches by
`use` declarations. Resolution is by name, among the modules supplied with the
root, and a `use` names the first supplied module of its name. A supplied
module that the root does not reach is not part of the program: it is not
checked, does not count toward the module limit of section 9, and contributes
nothing, unless it shares its name with a module of the program.

Before any module is checked, the **module graph** is examined. Starting from
the root, depth first, with each module's uses in source order, each module is
examined when it is first reached:

1. Every other supplied module of its name is `ORC0231`, "duplicate module
   `m`", at the name of whichever of the two was supplied later, with the
   other as a secondary span.
2. Each of its `use` declarations is then examined in source order:
   - a module used a second time by the same module is `ORC0231`,
     "module `m` is used twice", at the second declaration, with the first as
     a secondary span;
   - a module that uses itself is `ORC0230`, "module `m` uses itself", at the
     declaration;
   - a name that no supplied module has is `ORC0228`, "no module named `m` in
     this program", at the name.
3. A use that reaches a module whose examination has started but not finished
   closes a cycle. It is `ORC0230`, "module cycle `a` -> `b` -> `a`", at that
   declaration, naming the modules of the cycle in order, at most eight of
   them before `-> ...`.
4. A use that reaches a 65th module stops the examination at once, as section
   9 says.

If the module graph has any error, analysis stops with those diagnostics and
no module is checked. Otherwise the modules are put in **dependency order**,
the order in which the depth-first search of step 2 finishes them: every
module comes after every module it uses, and the root comes last. A module
used by several others appears once.

Each module is then checked in dependency order as S3g checks one module, with
its own per-source limits, and with one addition: the names of qualified calls
are resolved as section 5 says. A module that uses a module with errors is
still checked, against that module's declarations, so a program reports the
errors of each of its modules. Diagnostics are reported module by module in
dependency order, each module's in the order S3g gives them.

Every name a module declares stays in that module. A function name may be
declared by several modules of a program; `a::f` and `b::f` are different
functions, and an unqualified call names a function of the calling module
only.

## 5. Qualified calls

A call `m::f(...)` in a module M is resolved as follows.

1. `m` must be named by a `use` declaration of M. Otherwise the call is
   `ORC0229` at `m`: "module `m` is not used by `M`", labeled "no `use`
   declaration names this module", with the note "declare `use m;` at the head
   of the module to call its functions". When `m` is M itself, the message is
   "`m` is the calling module", labeled "a module does not qualify calls to
   itself", with the note "call a function of the same module without a module
   name, as in `f(x)`".
2. `f` must be a typed `spec` function of `m`. Otherwise the call is
   `ORC0212` at `f`, "no typed `spec` function named `f` in module `m`", or,
   for a `spec` of `m` without a typed body, "`spec` function `f` has no typed
   body and cannot be called" with its declaration as a secondary span. The
   note is the S3g note for an `impl` function when `m` has one named `f`, and
   otherwise "a qualified call names a typed `spec` of the used module".
3. The call is then checked exactly as a call to a function of M would be:
   argument count, result type, and each argument against the parameter type
   of `f`, with the S3g messages.

An unqualified call `f(...)` resolves in M only, as in S3g. When it is
`ORC0212` and M has no `impl` function named `f`, whose S3g note is kept, the
note depends on M's uses: when M declares no `spec` named `f` and a module M
uses declares one, it is "the used module `m` declares `f`; call it as
`m::f(...)`"; when M uses no module, it is the S3g note unchanged; and
otherwise it is "calls name a typed `spec` declared in the same module, or one
of a used module as `NAME::f(...)`".

A qualified call is a leaf with the result type of `f` wherever S3g takes the
type of a first typed leaf, as in a comparison, a conversion, or an index.

A qualified call adds no edge to M's call graph. Since uses are acyclic, no
cycle of calls can pass through another module, and the S3b call-cycle rule
applies to each module's own calls, with its own function names.

## 6. Diagnostics and Typed Reference Core

| Code | Phase | Meaning |
| --- | --- | --- |
| `ORC0228` | semantic | a `use` declaration names no module of the program |
| `ORC0229` | semantic | a call is qualified by a module its module does not use, or by its own module |
| `ORC0230` | semantic | a module uses itself, or the uses of a program form a cycle |
| `ORC0231` | semantic | two modules of a program have one name, or a module uses a module twice |

The module-graph diagnostics are retained under the per-source bound of 100
ordinary diagnostics, followed by one `ORC0208`; they consume no semantic
events. `ORC0101`, `ORC0103`, `ORC0106`, and `ORC0212` gain the uses of
sections 3 and 5. Every other code keeps its meaning.

**Core.** The Core of one module is unchanged in shape: a qualified call is a
`call` node naming its callee's identity, as any call is. Each Core function
now records the name of the module that declares it.

A program's **linked Core** carries the root's name and extent. Its functions
are those of each module in dependency order, each module's in source order,
so the root's come last. A function's identity is its position in that list:
dense across the program, and increasing in dependency order. The position of
the root's first function is recorded, and the root's functions are the
**entry functions**. A program of one module has the Core S3g gives it, with
the module's name on each function and its entry at position 0.

## 7. Evaluation

Reference evaluation evaluates the root's parameterless functions in source
order, exactly as S3g evaluates a module's, and prints each as
`root::name: Type = value`. The functions of used modules are evaluated only
through calls, so a used module's parameterless functions are never printed.
A call into another module costs what any call costs. The per-source budget
of 1,048,576 steps covers the whole evaluation of a program, whichever
modules its steps run in.

## 8. Modules in `orangec`

`orangec check` and `orangec eval` read a program as follows. The root is the
named source. The module `m` of a `use m;` declaration is read from the file
`m.or` in the directory of the root file, or in the current directory when
the root is read from standard input. A module name is an ASCII identifier,
so it names one file in that directory and no path outside it.

- Each module is read, lexed, and parsed once, in the order in which a `use`
  first names it, starting with the root's uses, and all of a program's
  sources share one source map, so diagnostics name the file they concern.
- A name that is the root's own is not read; the module graph reports it.
- Every module's bytes are charged to the invocation's source budget of 64 MiB,
  and a module file must be a regular UTF-8 file of at most 16 MiB, as every
  source is.
- At most 64 modules besides the root are read. A program that reaches more is
  then refused by the module limit of section 9.
- A module that cannot be read is `ORC1001`, with the note "`use m;` in module
  `M` reads the module `m` from this file"; a module that is not UTF-8, is too
  large, or does not lex or parse reports what that failure reports for any
  source. Any of these stops that program with no semantic analysis and no
  value.
- A file `m.or` that declares a module of another name is still read, and the
  module graph then reports `use m;` as `ORC0228`, since no module of the
  program is named `m`. Its own `use` declarations name no module of the
  program and are not followed: no file is read for them.

`orangec lex` does not read used modules. When one invocation names several
sources, each is the root of its own program.

## 9. Resource limits and failure

The S3g budgets remain, per module. S3h adds the following.

- A module has at most 64 `use` declarations (section 3).
- A program has at most 64 modules, its root included. A use that reaches a
  65th module is `ORC0209` at the root module, labeled "program reaches more
  than 64 modules"; the module graph's examination stops there, that is the
  program's only diagnostic, and no module is checked. Supplied modules that
  the root does not reach are not counted, and the graph's work is linear in
  the number supplied.
- Each `use` declaration costs one semantic event of its module's budget, when
  the module is checked. The module graph consumes no events.
- Each module is checked under its own per-source limits: at most 100 ordinary
  diagnostics, 262,144 Core nodes, and 1,048,576 semantic events.
- The linked Core holds at most 64 modules' functions, and their identities
  must fit in 32 bits; an allocation failure anywhere yields one resource
  diagnostic and no Core.

A syntax tree whose spans, including those of its `use` declarations and of a
call's module name, do not all belong to the source it is supplied with is
`ORC0210` for that source, and nothing is checked.

## 10. Determinism and conformance

The determinism requirement of `EXPRESSIONS_2026.md` section 15 applies
unchanged, and extends to programs: the same files give the same modules in
the same dependency order, the same diagnostics, and the same output bytes.
The S3h conformance runner (`compiler/crates/orangec/tests/s3h_conformance.rs`)
parses this index and requires exact agreement with its evidence map, under
the same rules as the S3b through S3g runners.

### S3h conformance rule index

| Rule ID | Clause | Executable obligation | Evidence layer |
| --- | --- | --- | --- |
| `S3H-SYNTAX-01` | Section 3 | `use` declarations come first in a module, `use` is a word only there, and a qualified name is always called; malformed forms are `ORC0101`, `ORC0103`, or `ORC0106`. | CLI and parser unit |
| `S3H-GRAPH-01` | Section 4 | No other supplied module shares a name with a module of the program, each use names a supplied module once and not its own module, and uses form no cycle; otherwise `ORC0228`, `ORC0230`, or `ORC0231` in the specified order. | CLI and unit |
| `S3H-ORDER-01` | Section 4 | Modules are checked and linked in dependency order, each once, and a module that uses one with errors is still checked. | CLI and unit |
| `S3H-CALL-01` | Section 5 | A qualified call resolves only in a used module and is checked as any call is; otherwise `ORC0229` or `ORC0212` with the specified notes. | CLI and unit |
| `S3H-CORE-01` | Section 6 | Linked Core lists the used modules' functions in dependency order and then the root's, with dense identities, module names, and the root's entry. | Unit and CLI observation |
| `S3H-EVAL-01` | Section 7 | Only the root's parameterless functions are evaluated, under one step budget; SHA-256, HMAC, and HKDF across modules match their published vectors. | CLI and unit |
| `S3H-CLI-01` | Section 8 | `orangec` reads `m.or` beside the root, or in the current directory for standard input, once per module, and reports an unreadable module with the using module. | Generated CLI and unit |
| `S3H-RES-01` | Section 9 | A module has at most 64 uses and a program reaches at most 64 modules, however many are supplied; every other budget applies per module. | Generated CLI and unit |
| `S3H-COMPAT-01` | Section 11 | S3g sources keep their meaning, Core values, messages, and output bytes. | CLI and unit |
| `S3H-DETERMINISM-01` | Section 10 | Repeated identical inputs produce identical status, diagnostics, and output bytes. | CLI and unit |

## 11. Relationship to S3g

When OEP-0011 is accepted, this document replaces these clauses of
`LOOKUPS_2026.md` and the documents it extends:

- the module production of `EXPRESSIONS_2026.md` section 3 and the call
  production, by section 3;
- the input of semantic analysis in `EXPRESSIONS_2026.md` section 2, by
  sections 2 and 4;
- the resolution of calls in `EXPRESSIONS_2026.md`, by section 5;
- the Core module and the evaluation of a module's functions, by sections 6
  and 7; and
- the reading of sources by `orangec check` and `orangec eval`, by section 8.

Every source that S3g accepts is accepted by S3h with the same Core values,
the same messages, and the same output bytes: it has no `use` declaration and
no qualified call, so it is a program of one module. Its Core functions now
also carry its module's name. A source that S3g rejects gets the same
diagnostics, with these exceptions:

- a `use` declaration at the head of a module was `ORC0103` at `use`; it is
  now accepted, or rejected by sections 3 and 4;
- a qualified call was `ORC0101` at `::`; it is now accepted, or rejected by
  sections 3 and 5.

## 12. Explicit non-claims and future work

This slice defines no import of names into scope, no renaming, no re-export,
no visibility or privacy, no module parameters, no modules nested in modules,
no module paths or packages, no search path beyond the root's directory, no
qualified names other than calls, and none of the exclusions of
`LOOKUPS_2026.md` section 11 that this document does not lift. Every typed
`spec` function of a module may be called by any module that uses it.

A module is a unit of meaning, not of compilation. No separate compilation,
interface file, linking of object code, or caching is defined, and nothing
here concerns code generation. Reading `m.or` beside the root is a rule of
`orangec`, not of the language: a program is its root and the modules it
reaches among those supplied with it, and another host may supply them another
way.

A fixture that reproduces a standard's example value is not thereby a verified
transcription of that standard. Tests establish the tested behavior of one
implementation at one revision; they do not prove semantic soundness,
completeness, or implementation independence.

Adding any excluded behavior requires a later accepted decision, a normative
specification, positive and negative conformance cases, resource analysis, and
migration review.
