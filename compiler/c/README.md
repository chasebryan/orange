# Standalone C compiler

Status: provisional owner-directed frontend for the Orange 2026 expression,
binding, conversion, array, bounded-loop, conditional, lookup, module,
residue, block, and tuple fragment. It does
not amend D-008, does not select a D-010 output path, and does not replace
the Rust frontend.

`compiler/c` is a dependency-free C11 program. It lexes, parses, checks, and
reference-evaluates one Orange source without linking to the Rust compiler and
without using any third-party library. The binary is another `orangec` for
that fragment: `check`, `eval`, and `lex` use the same token names, the same
value spelling, and the same diagnostic codes as the Rust frontend on the
programs this slice admits.

## What it implements

The admitted source is edition 2026. A program is a root module plus every
module it reaches through `use`. `orangec check` and `orangec eval` read
module `m` of `use m;` from `m.or` beside the root (`lex` reads only the file
it is given). A module names each used module once, before its functions, and
calls that module's functions as `m::f(...)`. The module graph is acyclic and
is checked before any module. Each module is then checked on its own, after
the modules it uses. A typed `spec` may have parameters, `let` bindings, and one result
expression. The scalar types are `Int`, `Bool`, `Word[8]`, `Word[16]`,
`Word[32]`, `Word[64]`, and `Mod[m]`. A module may name a type with `type`
after its `use` declarations and before its functions. `Mod[m]` is the
residue ring of a constant modulus from 2 through 2^521 - 1. `+`, `-`, `*`,
and prefix `-` reduce to the least residue; `/` multiplies by an inverse and
is 0 when there is none. A fixed-length array `T^n` holds n values of one
of those scalars, with n a decimal integer from 1 through 256. Expressions are
literals, names, calls, parentheses, array literals, indices, exact integer
arithmetic, Euclidean `/` and `%`, word ring arithmetic, bitwise operators,
shifts, rotations, comparisons, `!`, `&&`, `||`, and `as` conversions. A loop
`for i in a..b with s: T = start { step }` folds `step` from the literal bound
`a` up to `b`. An index is proved in range before evaluation. A word index
ranges over its type, narrowed by `&`, `|`, `^`, `~`, shifts by a literal,
`+`, `-`, `*`, `/`, `%`, conversions, and conditionals where the result cannot
wrap. An `Int` index is built from integer literals, loop indices, words and
residues converted with `as Int`, `+`, `-`, `*`, `/`, `%`, and conditionals. `x with
[i] = v` replaces one element, and `[v; n]` repeats a value. `if c { a } else
{ b }` chooses one value; an `else if` chain is one conditional, and only the
chosen branch is evaluated. A loop's step and each branch of a conditional
may begin with `let` bindings. A step's bindings are evaluated afresh at
every step, a branch's only when that branch is chosen, and each name is in
scope only within its step or branch. A tuple type `(T, U)` holds two through
16 scalars or arrays. `(a, b)` builds one, `.k` selects an element, and a
`let` or `with` pattern names each element. A byte string `"..."` or `hex"..."` is
the array `Word[8]^n` of its bytes, `++` joins two arrays, and a slice
`x[a..b]` or slice update `x with [a..b] = v` reads or replaces a run whose
bounds are integer literals and loop indices proved in range before the
program runs. A function may take size parameters, `spec f[n in a..b](...)`,
and is checked once for each value in range. Lengths, fill lengths, and loop
bounds may be written with those sizes, and a call is `f[2](x)` or is fitted
from the lengths of its arguments. `x as big T` and `x as little T` pack a
word or an array of words into words of another width with the same number of
bits, into `Int`, or into `Mod[m]`, and unpack a number back into words. The
first word is most significant for `big` and least significant for `little`.
`for`, `in`, `with`,
`if`, and `else` are names outside those positions. `true` and `false` are
`Bool` values where no parameter or binding of that spelling is in scope.
Empty `spec` and `impl` declarations parse and have no value.

`eval` prints one line for each parameterless typed `spec` of the root, in
source order. Functions of a used module run only when the root calls them.
The whole program shares one step budget.

```text
module::name: Type = value
```

Words are fixed-width lowercase hexadecimal. `Int` values and residues are
exact decimal integers. A residue is the least residue, from 0 through m - 1.
`Bool` values print as `true` and `false`. A modulus below 2^64 prints in
decimal; a larger one prints as `(1 << k) - c`, `(1 << k) + c`, or `1 << k`
when it is within 2^64 of a power of two, and otherwise in hexadecimal. An
array prints as `[e0, e1, ...]` and its type as `T^n`. Functions with
parameters are checked and run only when called.

An array literal lists every element. A fill states the length in decimal.
An index follows a name, a call, or an accumulator. Operators and conversions
apply to elements. Loop bounds are integer literals with `0 <= a < b <= 65536`.
Arrays of arrays, empty arrays, and computed loop bounds are rejected.

Typed `spec` functions may take type parameters, written `K in {T1, T2}` and
checked once for each listed type. Tests, lengths above 256, and computed
shift amounts are rejected rather than given a new meaning.
The Rust `orangec` remains the frontend for those slices.

## What it does not claim

This program is a reference checker and interpreter. It does not generate
native code, C, or LLVM IR. It does not claim constant-time execution,
verification, cryptographic correctness, or production readiness. Integer and
word results are the reference values of the admitted fragment; they are not a
certificate and not a selected compiler backend.

Build it with a C11 compiler such as GCC or Clang:

```sh
make -C compiler/c
compiler/c/out/orangec eval path/to/file.or
```

`make -C compiler/c test` builds an address-sanitized binary, runs the
exact-integer self-test, and compares `check`, `eval`, and `lex` with the Rust
`orangec` on the S3a through S3j fixtures, including ChaCha20, SHA-256,
Poly1305, X25519, ChaCha20-Poly1305, AES-128, the HMAC/HKDF program rooted
at `valid-vectors.or`, the modular X25519, Poly1305, and field fixtures, and
the block-let X25519, SHA-256, and block fixtures.
The Rust binary is only a
test oracle. Running the C compiler does not require it.

## Limits

The frontend fails closed. A source is at most 16 MiB. Lexing keeps at most
262,144 tokens. Expressions nest at most 64 levels and stay within height 256.
A function has at most 64 parameters, 256 bindings, and a call has at most 256
arguments. An array literal or fill has at most 256 elements. A loop takes at
most 65,536 steps. An `Int` magnitude has at most 16,384 significant bits.
A program reaches at most 64 modules, and each module has at most 64 `use`
declarations and 64 `type` declarations. A modulus has at most 521 bits. Reference evaluation of the whole program shares 1,048,576 steps
and 256 call frames. An update or
fill of n elements costs one step per 64 elements. Arrays are released once no
live value holds them. Printing uses as much text as the value's spelling
needs, including an array of 256 full-width integers. Exhausting a limit, or
failing to retain an array or its spelling, produces one resource diagnostic
and no value lines.
