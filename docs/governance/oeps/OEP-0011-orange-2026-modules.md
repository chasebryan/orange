---
number: OEP-0011
title: Orange 2026 programs of more than one module
authors:
  - Chase Bryan
champion: Chase Bryan
status: Review
type: Standards
created: 2026-09-30
updated: 2026-09-30
discussion: owner-direction-2026-09-30-s3h
related-decisions:
  - D-002
  - D-004
  - D-011
  - D-023
  - D-025
  - D-026
related-adrs: []
requires:
  - OEP-0001
  - OEP-0002
  - OEP-0003
  - OEP-0004
  - OEP-0005
  - OEP-0006
  - OEP-0007
  - OEP-0008
  - OEP-0009
  - OEP-0010
supersedes: []
superseded-by: null
review-authorities:
  - Orange Project Owner
decision-date: null
decision-revision: null
approval-records: []
---

# OEP-0011: Orange 2026 programs of more than one module

## Abstract

A module may name, at its head, the modules it uses, and call their typed
`spec` functions by module name. A program is a root module with every module
it reaches this way; its uses must form no cycle. Each module is checked on
its own, in dependency order, against the declarations of the modules it
uses, and the program's Core lists every module's functions with the root's
last. Evaluation prints only the root's values, under one step budget.
`orangec` reads the module `m` of `use m;` from the file `m.or` beside the
root.

With this slice, HMAC is written over SHA-256 as RFC 2104 writes it, in its
own file:

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

The normative text is [`docs/MODULES_2026.md`](../../MODULES_2026.md). An
implementation, 4 programs over 6 modules, and a 10-rule conformance runner
accompany it so that the proposal can be reviewed against running code. This
proposal is in **Review** and requires OEP-0010, which is also in review. It
accepts no D-004 candidate and gives the Typed Reference Core no canonical or
proof role.

## Motivation

Cryptography is specified in layers, and each layer's standard cites the one
beneath it. HMAC (RFC 2104) is defined over any iterated hash, HKDF (RFC 5869)
over HMAC, an AEAD over a cipher and an authenticator, and HPKE over a KEM, a
KDF, and an AEAD. A reviewer checks each layer against its own standard.

Through S3g an Orange program was one module in one file. Every construction
had to carry its own copy of every primitive beneath it, so the Daylight
example holds SHA-256, HMAC, HKDF, ChaCha20, and Poly1305 in one module of
more than 500 lines, and every further construction would repeat the
primitives it shares with the others. A copy is a second place to
review and a second place for a transcription error. With modules, SHA-256 is
written once, checked once against FIPS 180-4, and used by name.

A module system for specifications must not make meaning depend on context.
The designs that import names into scope, re-export them, or resolve them
along search paths make a reader ask which declaration a name denotes. S3h
answers that question at every call: a call into another module names the
module, and the module declares each module it uses at its head.

## Scope and non-goals

This proposal defines `use` declarations, qualified calls, the module graph of
a program and its errors, dependency order, checking each module against the
modules it uses, linked Core with entry functions, evaluation of a program,
the reading of modules by `orangec`, and their limits. It adds the diagnostic
codes `ORC0228` through `ORC0231`.

It does not define imports of names into scope, renaming, re-exports,
visibility or privacy, module parameters or functors, nested modules, module
paths or packages, search paths beyond the root's directory, qualified names
other than calls, separate compilation, interface files, or caching. It makes
no timing, secrecy, or leakage claim.

### Strata assumption

As for S3b through S3g, S3h assumes only what every D-004 candidate gives the
Specification role: pure, total, deterministic meaning. A qualified call is a
call to a total function, and a program without cycles of uses has the
meaning its modules' functions give it, so under `ST-REL`, `ST-UNI`,
`ST-DUAL`, `ST-MIRROR`, and `ST-HOST` alike the source surface needs no
change. Whether modules of different strata may use each other, and which
direction a refinement may cross a module boundary, is D-004's question and
is left open.

## Specification

[`docs/MODULES_2026.md`](../../MODULES_2026.md) is the complete normative
text. In summary:

- **Syntax.** `use m;` declarations come first in a module, at most 64 of
  them; `use` is a word only there. A call may be qualified as `m::f(...)`,
  and a qualified name is always called.
- **Programs.** A program is a root and every module it reaches by uses,
  among the modules supplied with it, at most 64 in all; a supplied module it
  does not reach is ignored and not counted. No other supplied module shares
  a name with a module of the program, a module uses another at most once and
  never itself, and uses form no cycle (`ORC0228`, `ORC0230`, `ORC0231`). The graph is examined
  depth first from the root, and modules are ordered as the search finishes
  them, so each comes after every module it uses and the root comes last.
- **Checking.** Each module is checked in that order, as S3g checks one
  module, under its own per-source limits. `m::f` must name a module that the
  calling module uses (`ORC0229`) and a typed `spec` of it (`ORC0212`), and is
  then checked as any call. Unqualified calls resolve in the calling module
  only, so two modules may declare functions of one name.
- **Core.** Each Core function records its module. A program's linked Core
  lists every module's functions in dependency order with dense identities,
  and marks the root's functions as its entries.
- **Evaluation.** Only the root's parameterless functions are evaluated and
  printed, and one budget of 1,048,576 steps covers the whole program.
- **`orangec`.** The module `m` is read from `m.or` in the root file's
  directory, or the current directory for standard input, once per program,
  under the invocation's source budget.

## Alternatives

Textual inclusion, in which a directive pastes another file's text into a
module, was rejected. It gives a function a different meaning in every file
that includes it, makes diagnostics depend on the including file, and lets two
copies of one primitive diverge.

Importing names into scope, as `use sha256::compress;` or `use sha256::*;`
would, was deferred. It saves the qualifier at the cost of a reader's
certainty about which declaration a name denotes, and it needs rules for
shadowing and conflicts that S3h does not have to state. Every S3h program
stays valid if such imports are added later.

Resolving uses along a search path, or through a package manifest, was
deferred. The rule that `m.or` lies beside the root is enough for the
fixtures and the Daylight example, and it is a rule of
`orangec`, not of the language: another host may supply the modules another
way.

Allowing cycles of uses between modules whose functions do not call each
other in a cycle was rejected. It would require analyzing the program's whole
call graph before checking any module, and it lets a module's meaning depend
on modules that depend on it. Acyclic uses let each module be checked once,
after its dependencies, and keep the S3b rule that every function's call
graph is finite and acyclic.

Evaluating every module's parameterless functions was rejected. A used module
is a library; its values are its own tests, not the program's output.

Visibility, so that a module could keep helpers private, was deferred. A
specification has no secrets to hide from its readers, and every function of
a used module being callable keeps the rule to one sentence.

## Compatibility and migration

Every source that S3g accepts has no `use` declaration and no qualified call,
so it is a program of one module and S3h accepts it with the same Core values,
messages, and output bytes. Its Core functions now also record their module's
name, and its Core records its entry at position 0. A source that S3g rejects
gets the same diagnostics, except that a `use` declaration at the head of a
module, which was `ORC0103`, and a qualified call, which was `ORC0101` at
`::`, are now accepted or reported by the S3h rules.

The public Rust API gains `analyze_program`, `CoreModule::entry_functions`,
`CoreFunction::module`, `UseDeclaration`, and the limits
`MAX_MODULES_PER_PROGRAM` and `MAX_USES_PER_MODULE`. `analyze` is unchanged in
signature and analyzes a program of one module. `CallExpression::module`
returns the qualifier of a call.

`orangec check` and `orangec eval` now read files beside the root when the
root has `use` declarations. A root without them reads nothing more.

Rollback reverts the parser, analyzer, Core, evaluator, command-line
interface, tests, fixtures, and normative documents together.

## Semantic and claim effects

This proposal gives exact meaning to programs of several modules. The
supported claim remains deterministic, bounded analysis and evaluation of the
documented fragment at a recorded implementation revision. It establishes no
soundness, proof, refinement, compilation, cryptographic correctness,
constant-time behavior, compatibility, independent review, or production
readiness.

## TCB, axiom, and proof effects

The parser, analyzer, Core constructor, evaluator, and command-line interface
remain engineering trust dependencies. The module graph, the per-module scopes
of qualified calls, the offset of Core identities, and the reading of module
files are new trusted code. Unit tests check the graph's errors and order,
qualified resolution, dense identities across a chain of 64 modules, and
spans that do not belong to their source. No axiom, theorem, proof rule,
certificate, checker, or solver is introduced.

## Threat, abuse, and leakage effects

`orangec check` and `orangec eval` now open files that a command line did not
name: the file `m.or` for each `use m;`. A module name is an ASCII identifier,
so the file lies in the root's directory, or the current directory for
standard input, and no name reaches outside it. Every module is read under the
same regular-file, size, and UTF-8 rules as a named source, and its bytes are
charged to the invocation's source budget, so a program cannot read more than
a command line could. A program reads at most 64 modules besides its root,
each once; a file that declares a module of another name is kept for the
diagnostic, but its uses are not followed. The module graph enters at most 64
modules, so its work is linear in the number of modules supplied. A module
file that an attacker can place beside the root changes the program's
meaning, as the root file itself would; S3h adds no protection against a
directory the reader does not control.

No secrecy label or leakage property is defined.

## Target and ABI effects

None. A module is a unit of meaning, not of compilation or linking.

## Standards, errata, and provenance

FIPS 180-4 sections 4.1.2, 4.2.2, 5.1.1, 5.3.3, and 6.2, RFC 2104, and RFC
5869 motivate the layering. The fixtures check the SHA-256 digest of "abc"
from FIPS 180-4's examples, test cases 1 and 2 of RFC 4231 for
HMAC-SHA-256, and the pseudorandom key and output key material of test case 1
of RFC 5869. No standard gains normative authority through this proposal.

## Dependencies, licenses, and IP

No dependency is added.

## Conformance, tests, and evidence

`compiler/crates/orangec/tests/s3h_conformance.rs` binds the 10 rules of the
specification's index to evidence and fails on any drift. Four programs over
six modules run through `orangec check` and `eval` twice each, and generated
programs check a diamond of uses, a module read for standard input, a module
that declares another name, a chain of 64 modules and one more, 65 uses in one
module, and one step budget across modules. Unit tests cover the parser, the
module graph and its order, qualified resolution and its notes, dense Core
identities, entry functions, spans from other sources, and module reading.
The S2 through S3g runners continue to pass unchanged.

## Operations, release, and recovery

No service, package, key, or release is added.

## Support and deprecation

The fragment is pre-alpha and best effort under D-022, with no compatibility
promise.

## Unresolved questions

- Whether imports of names into scope should follow, and with what rule for
  conflicts.
- Whether modules should be found along a search path or through a manifest,
  so that programs in different directories can share modules.
- Whether a module should be able to keep functions private.
- Whether modules of different D-004 strata may use each other, and in which
  direction.
- Whether module parameters, so that HMAC is written over any hash, are worth
  their surface.

## Decision record

On 2026-09-28 the owner directed that Orange continue its implementation goals
and that development not freeze unless the owner asks, and on 2026-09-29 that
development of Orange continue. Programs of more than one file were the next
slice on the roadmap after S3g. This proposal records the S3h surface built
under that direction and is presented for the owner's review. It is not
accepted. Acceptance is the owner's decision alone; until it is recorded here
with a decision date, reviewed revision, and `solo-reviewed` approval record,
this proposal authorizes nothing by itself.
