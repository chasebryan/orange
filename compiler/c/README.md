# Standalone C compiler

Status: provisional owner-directed frontend for the Orange 2026 expression,
binding, conversion, and fixed-length array fragment. It does not amend
D-008, does not select a D-010 output path, and does not replace the Rust
frontend.

`compiler/c` is a dependency-free C11 program. It lexes, parses, checks, and
reference-evaluates one Orange source without linking to the Rust compiler and
without using any third-party library. The binary is another `orangec` for
that fragment: `check`, `eval`, and `lex` use the same token names, the same
value spelling, and the same diagnostic codes as the Rust frontend on the
programs this slice admits.

## What it implements

The admitted source is edition 2026 with one module of `spec` and `impl`
functions. A typed `spec` may have parameters, `let` bindings, and one result
expression. The scalar types are `Int`, `Word[8]`, `Word[16]`, `Word[32]`, and
`Word[64]`. A fixed-length array `T^n` holds n values of one of those scalars,
with n a decimal integer from 1 through 256. Expressions are literals, names,
calls, parentheses, array literals, literal indices, exact integer arithmetic,
word ring arithmetic, bitwise operators, shifts, rotations, and `as`
conversions. Empty `spec` and `impl` declarations parse and have no value.

`eval` prints one line for each parameterless typed `spec`, in source order:

```text
module::name: Type = value
```

Words are fixed-width lowercase hexadecimal. `Int` values are exact decimal
integers, including negatives. An array prints as `[e0, e1, ...]` and its type
as `T^n`. Functions with parameters are checked and run only when called.

An array literal lists every element, and an index is one integer literal
after a name or a call. Operators and conversions apply to elements. Arrays of
arrays, empty arrays, and variable indices are rejected.

Later slices are outside this frontend. Loops, conditionals, multiple modules,
`Mod`, tuples, byte strings, size parameters, byte order, type parameters,
tests, lengths above 256, and computed shift amounts are rejected rather than
given a new meaning. The Rust `orangec` remains the frontend for those slices.

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
`orangec` on the S3a, S3b, S3c, and S3d fixtures. The Rust binary is only a
test oracle. Running the C compiler does not require it.

## Limits

The frontend fails closed. A source is at most 16 MiB. Lexing keeps at most
262,144 tokens. Expressions nest at most 64 levels and stay within height 256.
A function has at most 64 parameters, 256 bindings, and a call has at most 256
arguments. An array literal has at most 256 elements. An `Int` magnitude has
at most 16,384 significant bits. Reference
evaluation shares 1,048,576 steps and 256 call frames. Exhausting a limit
produces one resource diagnostic and no value lines.
