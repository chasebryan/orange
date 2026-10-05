# Standalone C compiler

Status: provisional owner-directed frontend for the Orange 2026 expression,
binding, conversion, array, bounded-loop, conditional, lookup, module,
residue, block, tuple, byte-string, and size-parameter fragment (S3a through
S3m). It does not amend D-008, does not select a D-010 output path, and does
not replace the Rust frontend.

`compiler/c` is a dependency-free C11 program. It lexes, parses, checks, and
reference-evaluates one Orange program without linking to the Rust compiler and
without using any third-party library. The binary is another `orangec` for
that fragment: `check`, `eval`, and `lex` use the same token names, the same
value spelling, and the same diagnostic codes as the Rust frontend on the
programs this slice admits.

## Files

- `include/compile.h` — `orange_main`, the only entry `src/main.c` calls
- `include/bigint.h` — arena and exact `Int` magnitudes
- `src/main.c` — process entry
- `src/compile.c` — lexer, parser, checker, reference evaluator, module graph, and CLI
- `src/bigint.c` — limb arithmetic and the exact-integer self-test
- `tests/differential.py` — `check`, `eval`, and `lex` against the Rust `orangec` on the S3a through S3m fixtures

`src/compile.c` records the slice boundary and the fail-closed forms in its file comment. That comment is a map of this frontend, not a language rule.

## What it implements

The admitted source is edition 2026. A program is a root module plus every module it reaches through `use`.

- `orangec check` and `orangec eval` read module `m` of `use m;` from `m.or` beside the root. `lex` reads only the file it is given.
- A module names each used module once, before its functions, and calls that module's functions as `m::f(...)`.
- The module graph is acyclic and is checked before any module. Each module is then checked on its own, after the modules it uses.
- A module may name a type with `type` after its `use` declarations and before its functions.
- The scalar types are `Int`, `Bool`, `Word[8]`, `Word[16]`, `Word[32]`, `Word[64]`, and `Mod[m]`.
- `Mod[m]` is the residue ring of a constant modulus from 2 through 2^521 - 1. The constant is built from integer literals with `+`, `-`, `*`, `<<`, and parentheses, evaluated once, and published into the modulus table. `+`, `-`, `*`, and prefix `-` reduce to the least residue. `/` multiplies by an inverse and is 0 when there is none.
- A cross-module `Mod` call compares those modulus values, not each file's private table index, and retags the value at the module boundary.
- A typed `spec` may have parameters, `let` bindings, and one result expression. A binding whose type was already rejected does not also check its initializer. A rejected result type (`Float`, a later `type` name, or `Mod[1]`) does not also check the body.
- A fixed-length array `T^n` holds n values of one of those scalars, with n a decimal integer from 1 through 256.
- A tuple type `(T, U)` holds 2 through 16 scalars or arrays. `(a, b)` builds one from left to right, `.k` selects an element, and a `let` or `with` pattern names each element. `let(x)` is a call, not a pattern.
- A pattern name that repeats the loop index is a duplicate name. A pattern name used outside the loop is not in scope. A tuple inside a tuple, and an array of tuples, are rejected even when the nested type is a `type` alias. Whole-tuple `==` and `!=` stay rejected.
- `p.01`, `p.0.1`, and `x[0].1` are each one syntax error at the offending token.
- Expressions are literals, names, calls, parentheses, array literals, indices, tuple construction, exact integer arithmetic, Euclidean `/` and `%`, word ring arithmetic, bitwise operators, shifts, rotations, comparisons, `!`, `&&`, `||`, and `as` conversions. A rejected `!`, `&&`, or `||` is reported on its own; the operands are not also checked.
- A loop `for i in a..b with s: T = start { step }` folds `step` from the literal bound `a` up to `b`. The accumulator may be one name or a tuple pattern.
- An index is proved in range before evaluation. A word index ranges over its type, narrowed by `&`, `|`, `^`, `~`, shifts by a literal, `+`, `-`, `*`, `/`, `%`, conversions, and conditionals where the result cannot wrap. An `Int` index is built from integer literals, loop indices, words and residues converted with `as Int`, `+`, `-`, `*`, `/`, `%`, and conditionals.
- A loop bound or index literal that does not fit the 16,384-bit budget is an oversized literal. A bound or index that fits and is still out of range is a range error.
- A second index of a scalar is a type error, reported once. A loop step that fails after its opening brace does not also reject the function close.
- A rejected `as` target is reported, and the operand is still checked.
- `x with [i] = v` replaces one element, and `[v; n]` repeats a value.
- A byte string `"..."` or `hex"..."` is the array `Word[8]^n` of its bytes, with n from 1 through 256. Each character is printable ASCII, from a space through `~`, or an escape. A hex string writes those bytes as pairs of hex digits, and a space may separate bytes. The quote follows `hex` with no space.
- A hex string that is never closed is a lexical error. A character that is not a hex digit or a space, or a digit with no partner, is a lexical error at that first bad character. `hex "00"`, with a space before the quote, is a syntax error.
- A non-printable or non-ASCII byte string is rejected. An empty `""` is rejected.
- `++` joins two arrays of one element type. The lengths add, and a join longer than 256 elements is rejected.
- A slice `x[a..b]` holds the `b - a` elements from index `a`. `x with [a..b] = v` replaces that run. At least one bound is written. The bounds are integer literals and loop indices with `+`, `-`, and `*` by a constant, and they are proved in range before the program runs. The length is one positive number at every step.
- A runtime bound, such as the parameter in `data`, or a non-linear bound, such as `i * i` in `squared`, is rejected. A length that changes from step to step is rejected. A slice whose last step leaves the array is rejected.
- A function may take at most 4 size parameters, written `spec f[n in a..b](...)`, as in `mac[len in 1..256]`. `n` takes each value from `a` up to, but not including, `b`, with `a < b` and both bounds at most 65536. The function is checked once for each combination, and it has at most 256 instances. The first size changes slowest. An empty range, a bound past 65536, and a product past the cap (`many` has 361) are rejected, and then the body is not checked.
- Checking stops at the first instance that reports an error. `last` fails at `n = 1`. `none` fails at `n = 0`.
- A size is an `Int` constant inside its instance. It is built from integer literals and that function's size parameters with `+`, `-`, `*`, `/`, `%`, and parentheses. `/` and `%` are Euclidean, the same rules as for `Int`, so `blocks[1]` is 3. Lengths, fills, loop bounds, indices, and slice bounds may use those sizes.
- A length or bound computed from sizes is parenthesized. `Word[8]^(n + 1)`, `[0; 2 * n]`, and `0..n - 1` are syntax errors. The admitted forms are `Word[8]^(2 * n)`, `[0; (2 * n)]`, and `0..(n - 1)`.
- A call writes one size in brackets for each size parameter, `f[2](x)` or `sha256::sha256[n](...)`, or writes none and selects the one instance whose array lengths match the arguments. An out-of-range size, an extra size, a size on a function that has none, no matching instance, and more than one match are rejected.
- Instances may call one another. A cycle such as `swap` at 1 calling 2 calling 1 is rejected.
- `if c { a } else { b }` chooses one value; an `else if` chain is one conditional, and only the chosen branch is evaluated.
- A loop's step and each branch of a conditional may begin with `let` bindings. A step's bindings run afresh at every step, a branch's only when that branch is chosen, and each name is in scope only inside its step or branch. Those bindings do not give the enclosing `if` a type of its own.
- `for`, `in`, `with`, `if`, and `else` are names outside those positions.
- `true` and `false` are `Bool` values where no parameter or binding of that spelling is in scope.
- Empty `spec` and `impl` declarations parse and have no value.

`eval` prints one line for each parameterless typed `spec` of the root, in
source order, and one line for each instance of a sized spec that has no
value parameters. Functions of a used module run only when the root calls
them. The whole program shares one step budget.

```text
module::name: Type = value
module::name[n]: Type = value
```

Words are fixed-width lowercase hexadecimal. `Int` values and residues are
exact decimal integers. A residue is the least residue, from 0 through m - 1.
`Bool` values print as `true` and `false`. A modulus below 2^64 prints in
decimal; a larger one prints as `(1 << k) - c`, `(1 << k) + c`, or `1 << k`
when it is within 2^64 of a power of two, and otherwise in hexadecimal. An
array prints as `[e0, e1, ...]` and its type as `T^n`. A tuple prints as
`(e0, e1, ...)` and its type as `(T, U)`. Functions with value parameters are
checked and run only when called.

An array literal lists every element. A fill states the length in decimal or
as a parenthesized size. An index follows a name, a call, or an accumulator.
Operators and conversions apply to elements. A loop bound is an integer
literal or a size, with `0 <= a < b <= 65536`. Arrays of arrays, empty
arrays, and a loop bound that is neither a literal nor a size are rejected.

Later slices are outside this frontend. Byte order, type parameters, tests,
lengths above 256, computed shift amounts, and whole-tuple equality are
rejected rather than given a new meaning. The Rust `orangec` remains the
frontend for those slices. This frontend does not implement S3n or any later
slice.

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
`orangec` on the S3a through S3m fixtures, including ChaCha20, SHA-256,
Poly1305, X25519, ChaCha20-Poly1305, AES-128, the HMAC/HKDF program rooted at
`valid-vectors.or`, the modular X25519, Poly1305, and field fixtures, the
block-let X25519, SHA-256, and block fixtures, the tuple Ascon, ChaCha20,
SHA-256, and tuple fixtures, the byte-string ChaCha20-Poly1305, HMAC-SHA-256,
and bytes fixtures, and the sized Poly1305 program (255 instances and the
RFC 8439 section 2.5.2 tag), HMAC-SHA-256 over sized `sha256.or`, and the
size fixtures. The Rust binary is only a test oracle. Running the C compiler
does not require it.

## Limits

The frontend fails closed. A source is at most 16 MiB. Lexing keeps at most
262,144 tokens. Expressions nest at most 64 levels and stay within height 256.
A function has at most 64 parameters, 4 size parameters, 256 instances, 256
bindings, and a call has at most 256 arguments. An array literal or fill has
at most 256 elements. A tuple has at most 16 elements. A loop takes at most
65,536 steps. An `Int` magnitude has at most 16,384 significant bits. A
program reaches at most 64 modules, and each module has at most 64 `use`
declarations and 64 `type` declarations. A modulus has at most 521 bits.
Reference evaluation of the whole program shares 1,048,576 steps and 256 call
frames. An update or fill of n elements costs one step per 64 elements.
Arrays are released once no live value holds them. Printing uses as much text
as the value's spelling needs, including an array of 256 full-width integers.
Exhausting a limit, or failing to retain an array or its spelling, produces
one resource diagnostic and no value lines.
