# Dependency-ordered solo roadmap

Status: directed active roadmap under D-023 and OEP-0001

Snapshot: 2026-07-26

Orange is developed by one owner. This roadmap assumes no contributors,
independent reviewers, auditors, laboratories, partner organizations, or
separate operational roles. It contains no staffing-based calendar promise.
Outside participation may be incorporated only after it becomes real and the
owner explicitly changes the operating model.

## 1. How to read this roadmap

Orange still follows the no-disposable-prototype rule. Every merged compiler
component belongs to the intended production lineage and therefore needs stable
boundaries, deterministic behavior, diagnostics, tests, documentation, and a
clear migration story.

The former aggregate Gate 0 blocked all implementation until the entire 1.0
institutional plan was staffed and externally reviewed. D-023 supersedes that
barrier at its honest 0/7 state. Work now advances through incremental
capability gates:

- an unresolved decision blocks only the component or claim that depends on it;
- a test result does not become proof;
- owner review is never described as independent review;
- unavailable external evidence limits claims rather than unrelated work; and
- a partial system is always labeled pre-alpha and never marketed as complete.

## 2. Product direction

The long-term goal remains a language and toolchain for specifying,
implementing, and verifying cryptography. The intended system includes:

- an editioned Orange language and normative semantics;
- a deterministic frontend, reference evaluator, compiler, and developer tools;
- explicit, artifact-scoped claim records rather than a generic `verified`
  label;
- canonical proof and evidence formats with offline replay;
- a small published trusted computing base for each claim;
- native artifacts and a stable foreign-function boundary; and
- standards-sourced cryptography packages with precise provenance.

These are product directions, not descriptions of current features. The project
does not promise a date, LTS window, certification, external audit, independent
rebuild, or multi-person governance.

## 3. Solo workstreams

One owner performs the work, but the boundaries remain distinct:

| ID | Workstream | Permanent responsibility |
| --- | --- | --- |
| W0 | Product and decisions | Scope, decisions, naming, licensing, support, claim wording |
| W1 | Language and semantics | Grammar, types, effects, memory, erasure, leakage semantics |
| W2 | Proof and metatheory | Proof IR, checker, certificates, metatheory, trust reporting |
| W3 | Frontend and tools | Source model, lexer, parser, diagnostics, formatter, evaluator, LSP |
| W4 | Compiler and targets | IRs, lowering, validation, object paths, ABI, bootstrap |
| W5 | Cryptography corpus | Standards provenance, specifications, implementations, tests, proofs |
| W6 | Package and release | Manifests, locks, evidence bundles, builds, provenance, updates |
| W7 | Assurance and conformance | Threat model, adversarial tests, fuzzing, claim validation |
| W8 | Documentation and adoption | References, tutorials, examples, migrations, usability notes |

Separating workstreams prevents one successful test from leaking assurance into
another boundary. It does not imply separate people or independent review.

## 4. Dependency rules

```text
source files and diagnostics
           |
           v
editioned grammar and parser
           |
           v
name resolution and typed semantic core
       |                 |
       v                 v
reference evaluator   canonical Core
                         |
              +----------+----------+
              |                     |
              v                     v
         proof boundary         compiler IRs
              |                     |
              +----------+----------+
                         |
                         v
               target artifacts and claims
                         |
                         v
             cryptography corpus and releases
```

Critical ordering rules:

- Syntax may evolve during pre-alpha, but every accepted construct needs a
  documented grammar and deterministic parse.
- Type checking and evaluation require explicit arithmetic, failure, and name
  resolution semantics.
- Proof-bearing work requires D-006 and the canonical Core boundary to be
  selected; proof-neutral frontend work does not.
- Constant-time or leakage claims require D-012 and a target model; ordinary
  lexing and parsing do not.
- ABI or native-object claims require memory, layout, target, and foreign-boundary
  decisions.
- Cryptographic claims require exact standards and errata provenance, vectors,
  negative cases, and complete assumptions.
- Release claims never inherit from development checks.

## 5. Capability stages

### S0 — Repository foundation

Status: complete enough to support implementation

The repository has governance records, threat and assurance models, pinned CI,
dependency controls, provisional evidence schemas, adversarial policy fixtures,
and deterministic repository validation. The legacy Gate 0 institutional exit
criteria were not completed; D-023 retired them as an aggregate implementation
barrier.

### S1 — Compiler foundation

Status: complete at merged revision
`469bdec6037f20c8d099d61a09a3d19a55c88231`

Scope:

- pinned Rust edition and toolchain;
- no third-party Rust crates;
- source identity and UTF-8 byte spans;
- deterministic lexer with comments, identifiers, literals, punctuation, and
  reserved words;
- structured diagnostics with stable codes and source locations;
- `orangec` command-line input, output, and exit-code contract; and
- formatting, lint, unit, integration, malformed-input, and repeatability tests.

Exit test: the exact source inventory passes offline locked Rust checks and the
repository policy suite; malformed source produces bounded diagnostics rather
than a panic; repeated runs are byte-identical. Passing S1 makes no grammar,
semantic, proof, code-generation, cryptographic, or production claim.

### S2 — Editioned grammar and parser

Status: complete at merged revision
`52a3460853636f7cbaa27f3e27d86e032e3c82d4` under D-025 and accepted OEP-0002

Permanent outcomes:

- a mandatory exact `edition 2026;` marker;
- the normative lexical and grammar specification in
  [`LANGUAGE_2026.md`](LANGUAGE_2026.md);
- exactly one module containing empty `spec` or `impl` functions;
- a precisely source-mapped syntax tree;
- bounded error recovery and stable parse diagnostics;
- positive, malformed, ambiguity, duplicate-name, Unicode, line-ending,
  resource-limit, and repeatability cases; and
- exact source and policy inventory.

The directed grammar contains no imports, multiple modules, parameters, types,
expressions, non-empty bodies, semantics, proofs, targets, ABI, leakage, code
generation, packaging, or release behavior. `game`, `proof`, and `claim` remain
lexical reservations only.

Exit test: every accepted form maps to one syntax tree; every rejected form has
a stable error category; resource exhaustion fails closed; and mutation and
repeated parsing reveal no unexplained acceptance, panic, hang, or
nondeterminism. Required hosted checks and local offline checks pass at the
exact merged revision.

### S3 — Semantic core and reference evaluator

Status: active after completed S3a under D-026 and accepted OEP-0003

Permanent outcomes:

- module and name resolution;
- explicit types for mathematical integers and fixed-width words;
- function, binding, control-flow, and failure semantics;
- a typed Core boundary;
- deterministic reference evaluation; and
- one conformance fixture per normative rule.

Exit test: the specification, type checker, evaluator, diagnostics, and
conformance cases agree for every supported construct. No proof or native-code
claim is implied.

#### S3a — Typed literal specifications

Status: completed under D-026 and accepted OEP-0003 at merged revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`

The first bounded semantic slice preserves legacy empty `spec` and `impl`
declarations and adds only `spec NAME() -> TYPE { SIGNED_INTEGER }`. Semantic
acceptance recognizes mathematical `Int` and unsigned `Word[8]`, enforces
same-kind declaration-name uniqueness, lowers typed specifications to a
source-ordered Typed Reference Core, and evaluates those closed literal values
deterministically.

The slice has exact semantic diagnostic, Core-node, integer-input, semantic-
event, and evaluation-step budgets. It defines no operators, calls, parameters,
bindings, control flow, dynamic failure values, typed implementations, canonical
Core encoding, proof identity, refinement, code generation, ABI, leakage,
package, release, or cryptographic behavior. S3a does not complete S3.

Closure evidence: PR #9 merged at `2026-07-13T00:42:10Z` after Required CI run
`29215790064`, Dependency Review run `29215790110`, and CodeQL run
`29215789258` passed. At exact merged revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`, Required CI run `29215877872`,
Workflow Online Audit run `29215877891`, External Links run `29215877874`,
OpenSSF Scorecard run `29215877875`, and dynamic CodeQL run `29215877437` also
completed successfully. The merged slice passed 89 Rust tests, including the
documentation test, 95 Python policy tests, and policy version 0.2.3 with zero
findings.

Post-acceptance conformance/control refresh: PR #11 head
`7d54594349cc7afe0cacf60ebc9f1d8f5e913fee` passed Required CI run
`29292600483`, Dependency Review run `29292600471`, and CodeQL run
`29292598799`, then squash-merged at `2026-07-13T23:20:22Z` as exact `main`
revision `23352bcde976b86890db28ea4d375a31e6354bca`. At that revision, Required
CI run `29292740885`, Workflow Online Audit run `29292740874`, External Links
run `29292740884`, OpenSSF Scorecard run `29292740941`, and dynamic CodeQL run
`29292740478` completed successfully. Required CI covered policy version 0.2.6,
92 Rust tests including the documentation test, 103 Python policy tests, and
zero foundation-validator findings. This refresh adds the exact external S3a
black-box corpus and draft D-003/D-004 research protocols; it does not change
the S3a acceptance revision, accept either draft decision, or authorize S3b.

### S4 — Proof and claim boundary

Status: pending symmetric execution and exact-revision OEP acceptance of D-005,
D-006, and D-009 plus their dependent decisions

Permanent outcomes:

- a selected public-assurance architecture through the symmetric D-005 suite
  and an Accepted exact-revision OEP, with ten separate claim families, four
  exact atomic outcomes, complete evidence/TCB closure, and no aggregate
  upgrade;
- a selected proof foundation through the symmetric D-006 suite and an Accepted
  exact-revision OEP; any explicitly smaller initial proof scope is bounded
  inside that decision and cannot bypass the comparison;
- canonical Core and Proof IR identities;
- authoritative checking rules and an implementation-diverse checker where
  useful;
- fail-closed certificate and solver policy;
- axiom, assumption, and trust inventories; and
- mixed-status claim records.

Exit test: all four D-005 candidates have complete 8/8 case records and both
D-006 candidates have complete 7/7 case records, and all three D-009 candidates
have complete 8/8 case records (24/24 total) under their frozen epochs. The
selected assurance model, proof foundation, and solver-trust policy each pass
every non-compensable gate; all three owner comparative records are
`solo-reviewed`; and all three Accepted OEPs bind their exact validated
revisions. Implemented claim-policy,
subject/context substitution, trust-closure, composition, lifecycle,
malformed-proof, and missing-certificate paths fail closed. Solver timeouts,
unknowns, unavailable authorities, stale evidence, and aggregate summaries
never satisfy or upgrade an atomic claim. Repeated offline replay agrees.
Decision-case replay is capped at level 2 and independent review remains
`unavailable` while the operating model stays solo. The decision suites or OEPs
alone do not close S4 without the implemented and tested proof/claim boundary.
The current D-005 laboratory prepares only a digest-bound adapter request and a
fail-closed synthetic capture validator. It has no subprocess launcher or
candidate payload semantics and leaves execution at 0/32; this preparation is
not S4 evidence or gate closure.

It also enumerates the exact 32-by-2-by-3 in-memory transport identity matrix
and can bind synthetic raw captures to canonical integrity receipts. The 192
identity order does not authorize a physical run order; receipt verification
leaves isolation unevaluated, payloads unvalidated, and evidence absent. Missing
or cross-slot observations fail closed, but no opaque payload comparison,
candidate result, execution credit, or readiness credit follows.

The D-006 input-only laboratory likewise binds only its draft packet, unchanged
suite, and seven zero-fixture case blockers, then enumerates 14 in-memory
candidate-case identities. It admits or installs no proof tool, assigns no
physical order or execution resources, and leaves D-004/D-005 dependencies,
result/replay schemas, owner review, selection, and all 14 executions unresolved.
This preparation is neither S4 evidence nor readiness credit.

The D-009 input-only laboratory binds its candidate-neutral draft packet,
unchanged solver-trust suite, and eight zero-fixture case blockers, then
enumerates the exact 24 case-major, candidate-minor SP-01/SP-02/SP-03 identities
in memory. D-004 and D-005 acceptance remains absent; D-006 and D-007 remain
downstream consumers rather than cyclic prerequisites. The laboratory admits,
acquires, installs, or executes no solver, proof assistant, certificate checker,
adapter, runner, observer, or isolation backend; assigns no physical order or
execution resources; validates no proof, certificate, counterexample, theorem,
claim, or cache result; and records 0/24 executions, no evidence, no selection,
and no conclusion. This preparation neither changes the logical TCB nor closes
S4, authorizes solver-backed proof search, or changes Orange's 3/10 (30%)
binary gate-closure score; that score is not release readiness.

### S5 — Compiler IRs and one output path

Status: pending the applicable S3 semantics and symmetric execution,
solo-reviewed comparison, and exact-revision OEP acceptance of D-010

Begin with one bounded output path selected by the D-010 decision procedure.
The five candidates are the theorem/certificate hybrid direct-native path,
mechanized proof-per-pass direct-native path, versioned Jasmin boundary,
portable C11 boundary, and versioned LLVM IR boundary. C11 and LLVM IR remain
separate candidates. Do not create a target matrix or inherit a claim past the
selected boundary before one path is correct, inspectable, and explicitly
scoped.

Permanent outcomes include the selected path's semantic or versioned boundary,
deterministic lowering, validators, exact authorities and assumptions,
reference-semantic differential tests, corruption and substitution rejection,
and an explicit claim frontier. A native claim must satisfy its stronger target,
leakage, ABI, and final-byte obligations. An interoperability-only frontier must
mark downstream compilation and target properties `unsupported` or external
rather than inheriting them.

The D-010 decision prerequisite requires complete 8/8 records for all five
candidates (40/40 total) under one frozen symmetric epoch, CR-01 through CR-11
as `solo-reviewed` owner scopes, one all-hard-gates-pass recommendation, and an
Accepted exact-revision OEP. Those records select a strategy; they do not by
themselves implement or close S5.

The current D-010 input-only laboratory binds only its candidate-neutral draft
packet, unchanged suite, and eight zero-fixture case blockers, then enumerates
40 case-major, candidate-minor identities in memory. It admits or executes no
compiler or external tool, assigns no physical order or resources, and records
0/40 executions, no evidence, no selection, and no conclusion. It creates no
IR, output, object, certificate, target, ABI, leakage, or final-byte evidence.
This preparation is neither S5 implementation evidence nor readiness credit.

### S6 — Memory, leakage, ABI, and native targets

Status: pending semantic and target decisions

Add ownership, buffers, layout, erasure, leakage traces, target feature models,
one stable foreign boundary, and one native tuple at a time. Each advertised
claim names the exact target, ABI, feature profile, object bytes, assumptions,
and unsupported cases.

### S7 — Cryptography corpus

Status: pending S3 through S6 as applicable

Admit one standards-sourced primitive at a time. Each package retains exact
standards and errata provenance, rights notes, specification, implementation,
vectors, negative cases, interoperability results, performance observations,
and an explicit claim matrix. No local test is called certification.

### S8 — Packages, developer tools, and preview releases

Status: pending usable language behavior

Add immutable resolution, manifests and locks, offline bundles, formatter, LSP,
documentation generator, evidence inspector, and source archives. A solo preview
release requires an explicit release decision, exact source and artifact
digests, reproducible owner build instructions, known limitations, and support
dates. It cannot claim independent rebuild or multi-party release controls.

## 6. Immediate sequence

S3a evidence closure is complete at exact merged revision
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`, with OEP-0003 accepted and its
local and hosted evidence recorded.

S3b expands the Typed Reference Core with pure expressions and calls. It did
not wait for D-004: under the owner's direction of 2026-09-28 that development
should not freeze unless the owner asks for it, S3b was built and tested while
D-004 remains open, and it assumes only what every D-004 candidate gives the
specification stratum. Its acceptance requires:

1. retaining accepted D-003 and
   [OEP-0004](governance/oeps/OEP-0004-standalone-orange-product-form.md) as
   the exact-revision PF-01 standalone product-form boundary; and
2. the owner's acceptance of
   [OEP-0005](governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md),
   which bounds the S3b surface in [`EXPRESSIONS_2026.md`](EXPRESSIONS_2026.md)
   with explicit conformance, resource, compatibility, threat, and non-claim
   boundaries.

When D-004 is decided, it places the S3b Core within the chosen strata. That
placement changes no S3b source.

S3c follows S3b on the same terms. It adds typed `let` bindings and explicit
`as` conversions among `Int` and the four word types, so that a transcription
can name a standard's intermediate values and state its byte order. It is
implemented and tested, and its acceptance requires the owner's acceptance of
[OEP-0006](governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md),
which bounds the S3c surface in [`BINDINGS_2026.md`](BINDINGS_2026.md) and
builds on OEP-0005. Like S3b, it assumes only pure, total, deterministic
meaning, so the placement D-004 decides changes no S3c source either.

S3d follows S3c. It adds fixed-length arrays `T^n` of `Int` or word values,
array literals, and indices that are literals the analyzer proves in range,
so that a specification can hold a cipher's whole state as one value. The
ChaCha20 block function of RFC 8439 and the first SHA-256 rounds of FIPS 180-4
are now whole Orange programs that reproduce the standards' example values. It
is implemented and tested, and its acceptance requires the owner's acceptance
of [OEP-0007](governance/oeps/OEP-0007-orange-2026-fixed-length-arrays.md),
which bounds the S3d surface in [`ARRAYS_2026.md`](ARRAYS_2026.md) and builds
on OEP-0006. It too assumes only pure, total, deterministic meaning.

S3e follows S3d. It adds loops over literal ranges, indices built from integer
literals and loop indices that the analyzer proves in range for every step,
updates of one element, and fill literals, so that ten double rounds are one
expression and a standard's "for t = 16 to 63" is written as the standard
writes it. The whole SHA-256 hash of both FIPS 180-4 examples and the ChaCha20
encryption of RFC 8439 section 2.4.2 are now Orange programs that reproduce the
published digests and ciphertext. It is implemented and tested, and its
acceptance requires the owner's acceptance of
[OEP-0008](governance/oeps/OEP-0008-orange-2026-bounded-loops.md), which bounds
the S3e surface in [`LOOPS_2026.md`](LOOPS_2026.md) and builds on OEP-0007.
Like the slices before it, it assumes only pure, total, deterministic meaning.

S3f follows S3e. It adds the type `Bool`, comparisons, strict logical
operators, Euclidean division and remainder that are total at zero, and
conditionals that always have both branches, so that a field element reduces
modulo its prime and a ladder chooses by a bit of its scalar. The X25519
function of RFC 7748, Poly1305 of RFC 8439, and the ChaCha20-Poly1305 AEAD
construction are now Orange programs that reproduce the published test vector,
tag, ciphertext, and AEAD tag. It is implemented and tested, and its
acceptance requires the owner's acceptance of
[OEP-0009](governance/oeps/OEP-0009-orange-2026-conditions.md), which bounds
the S3f surface in [`CONDITIONS_2026.md`](CONDITIONS_2026.md) and builds on
OEP-0008. Like the slices before it, it assumes only pure, total,
deterministic meaning, and it makes no timing claim: a conditional chooses a
value, and how an implementation decides is a question for the implementation
stratum.

S3g follows S3f. It lets an index depend on data while still proving it in
range before evaluation: an index whose first typed leaf is a word ranges over
its type, narrowed by its operators, and an `Int` index may also convert words
with `as Int` and choose with conditionals. Updates and fills cost one step
per 64 elements, so that a table can change on every iteration of a loop. AES-128
of FIPS 197, with its S-box derived from inverses in GF(2^8), and a
table-driven CRC-32 are now Orange programs that reproduce the published
examples and check value. It is implemented and tested, and its acceptance
requires the owner's acceptance of
[OEP-0010](governance/oeps/OEP-0010-orange-2026-lookups.md), which bounds the
S3g surface in [`LOOKUPS_2026.md`](LOOKUPS_2026.md) and builds on OEP-0009. It
reverses the S3e rule that made lookups keyed by data inexpressible, and like
the slices before it, it assumes only pure, total, deterministic meaning and
makes no timing claim: how a lookup keyed by a secret is compiled is a
question for code generation.

S3h follows S3g. It lets a program span several modules, one per file: a
module declares the modules it uses at its head and calls their functions by
module name, as in `sha256::compress(h, block)`. The uses of a program form no
cycle, each module is checked once, after the modules it uses, and against
their declarations only, and evaluation prints only the root's values under
one step budget. `orangec` reads the module `m` from `m.or` beside the root.
SHA-256, HMAC, and HKDF are now three modules whose program reproduces the
examples of FIPS 180-4, RFC 4231, and RFC 5869. It is implemented and tested,
and its acceptance requires the owner's acceptance of
[OEP-0011](governance/oeps/OEP-0011-orange-2026-modules.md), which bounds the
S3h surface in [`MODULES_2026.md`](MODULES_2026.md) and builds on OEP-0010.
Like the slices before it, it assumes only pure, total, deterministic meaning.

S3i follows S3h. It adds `Mod[m]`, the integers modulo a constant m from 2
through 2^521 - 1, whose modulus is written as its standard writes it, as
`Mod[(1 << 255) - 19]`, and `type` declarations that name a type for the rest
of a module. Residues reduce by themselves under `+`, `-`, and `*`, `/`
multiplies by the inverse and gives 0 for a non-unit, two moduli are two
types, and `as` converts among `Int`, words, and residues by least residues.
X25519 and Poly1305 are now written over their fields with no reduction in
sight and reproduce RFC 7748's and RFC 8439's examples, and the constants of
ML-KEM, Ed25519, and P-256 are computed in the rings their standards define.
It is implemented and tested, and its acceptance requires the owner's
acceptance of
[OEP-0012](governance/oeps/OEP-0012-orange-2026-modular-arithmetic.md), which
bounds the S3i surface in [`MODULAR_2026.md`](MODULAR_2026.md) and builds on
OEP-0011. Like the slices before it, it assumes only pure, total,
deterministic meaning, and it makes no timing claim about residue arithmetic.

S3j follows S3i. It lets a loop's step and each branch of a conditional begin
with `let` bindings, as a function's body does. A step's bindings are
evaluated afresh at every step and a branch's only when it is chosen, each is
in scope only within its step or branch, and none may repeat a name in scope.
SHA-256's rounds now name a through h, T1, and T2 inside the loop that runs
them, and X25519's ladder names every value RFC 7748 names inside one loop;
both reproduce their standards' examples. It is implemented and tested, and
its acceptance requires the owner's acceptance of
[OEP-0013](governance/oeps/OEP-0013-orange-2026-blocks.md), which bounds the
S3j surface in [`BLOCKS_2026.md`](BLOCKS_2026.md) and builds on OEP-0012. Like
the slices before it, it assumes only pure, total, deterministic meaning.

S3k follows S3j. It adds tuples: a tuple type `(T, U)` of two through 16
elements, each a scalar or an array, a tuple `(a, b)`, the selection `.k` of
element k, and tuple patterns that name each element where a `let` binding or
a loop's accumulator is declared, so that a function gives several values and
a loop carries several accumulators. SHA-256 carries a through h as eight
named accumulators, ChaCha20's quarter round takes four words and gives four
as RFC 8439 writes it, and Ascon-Hash256 carries its state as five named
words; all three reproduce their standards' values. It is implemented and
tested, and its acceptance requires the owner's acceptance of
[OEP-0014](governance/oeps/OEP-0014-orange-2026-tuples.md), which bounds the
S3k surface in [`TUPLES_2026.md`](TUPLES_2026.md) and builds on OEP-0013. Like
the slices before it, it assumes only pure, total, deterministic meaning.

S3l follows S3k. It adds bytes: a byte string `"..."` is the array
`Word[8]^n` of the ASCII bytes of its text and a hex string `hex"..."` that of
its hex digit pairs, `++` joins two arrays, and a slice `x[a..b]` and a slice
update `x with [a..b] = v` read and replace a run of elements whose bounds are
built from literals and loop indices, proved a fixed distance apart and in
range before the program runs. HMAC-SHA-256 writes RFC 4231's keys and
messages as the RFC prints them and pads SHA-256's input with `++`, and
ChaCha20-Poly1305 writes RFC 8439's plaintext as text and its key, nonce, and
additional data in hex; both reproduce their standards' values. It is
implemented and tested, and its acceptance requires the owner's acceptance of
[OEP-0015](governance/oeps/OEP-0015-orange-2026-bytes.md), which bounds the
S3l surface in [`BYTES_2026.md`](BYTES_2026.md) and builds on OEP-0014. Like
the slices before it, it assumes only pure, total, deterministic meaning.

S3m follows S3l. It adds sizes: a `spec` may declare size parameters with
finite ranges, as `spec pad[len in 1..120](m: Word[8]^len)`, and stands for
one instance for each value of its sizes, at most 256, each checked as the
function written out with those values; sizes built from integer literals and
size parameters write array lengths, fill lengths, and loop bounds; and a call
names its instance by its sizes, as `pad[3](m)`, or by its arguments' lengths.
SHA-256 is written once for every message of 1 through 119 bytes,
HMAC-SHA-256 once for every key of 1 through 63 bytes and message of 1
through 55, and Poly1305 once for every message of 1 through 255 bytes, and
each reproduces its standard's values. It is implemented and tested, and its
acceptance requires the owner's acceptance of
[OEP-0016](governance/oeps/OEP-0016-orange-2026-sizes.md), which bounds the
S3m surface in [`SIZES_2026.md`](SIZES_2026.md) and builds on OEP-0015. Like
the slices before it, it assumes only pure, total, deterministic meaning.
Moduli written with parameters, so that one `spec` can serve every field,
positions given as parameters, so that one quarter round can act on four
positions of a whole state, conversions between bytes and words, and slices
at positions computed from data are the next candidate slices.

Only one slice is stabilized at a time. Research may run ahead, but code for a
dependent stage does not claim completion before its inputs are explicit.
The accepted
[D-003 product-form decision packet](PRODUCT_FORM_DECISION_PACKET.md) and the conditional
[D-004 semantic-strata decision suite](SEMANTIC_STRATA_DECISION_SUITE.md)
define the owner-executable records and research that may run ahead. Explicit
acceptance on 2026-07-26 and Accepted OEP-0004 bind D-003 candidate PF-01 to
exact revision `a82a5cec2ee4359dc2fe66171f17c93146747333`, but they do not
accept D-004 or authorize S3b. The immutable D-004 v0.5 review subject binds
exactly 73 candidate-neutral suite-only subjects and the exact historical suite
bytes; its `draft_unreviewed_input_only` fields remain unchanged. On 2026-07-26
the owner accepted D004-PRE-01 as `solo-reviewed` at exact review-subject
revision `7d09a27369649855ce987c76315271b0d34a20ef`.

That acceptance covers the immutable review subjects. The successor overlay's
implementation closure remains `provisional_pending_exact_merged_revision`
until its validated bytes are available at an exact merged revision.

The successor `d004-v0.6-reviewed-protocol` finds 5 ambiguity, 14 missing-edge,
13 identity-substitution, 5 unsupported, and 5 resource-exhaustion subjects
sufficient only for bounded suite coverage. It reviews all five candidate
graphs and 70 SR mappings only as symmetric, falsifiable test hypotheses, not
accepted Orange semantics or capability evidence. The reviewed replay plan
assigns three deterministic repetitions to each of 25 candidate-case units, for
75 planned executions. The v0.7 tranche built the adapter, closed payload
schemas, executable manifests, enforcing isolation and result parsers, and
replaced the separate owner freeze record with a content-addressed epoch
identity. Epoch `d004-e-4aaf8a83a01693d543c4` ran all 75 executions on
2026-09-28. ST-REL, ST-UNI, ST-DUAL, and ST-MIRROR each passed all five cases
with byte-identical repetitions; ST-HOST failed all five because six
relationships it delegates to hosts owned by the open D-006 and D-011 decisions
are unsupported. Evidence is 20 closed of 25 required units and 75 of 75 result
records, contributor-produced and unreviewed. The suite cannot separate the
four passing candidates, so selection and conclusion remain null. The v0.8 suite
adds SC-06 (semantic evolution) and SC-07 (within-authority relabeling) with
five cost measures, and the owner chose the isolation-first distinguishing rule
knowing which candidate each rule was predicted to select. Epoch
`d004-e-633e0aa831615cda3e06` ran all 105 executions on 2026-09-28: the same
four candidates closed all seven cases and ST-HOST closed none, for 28 closed of
35 required units and 105 of 105 result records. Isolation first leaves only
ST-REL, which ties ST-MIRROR at zero isolation obligations and re-identifies six
subject classes to its seven. That result is contributor-produced and
unreviewed, and it is not a D-004 recommendation until the owner disposes every
candidate and hard gate. D-004 remains proposed pending owner review, S3 remains
incomplete, S3b through S3m are implemented and await owner review under
OEP-0005 through OEP-0016, and Orange remains 30% complete by its unchanged
3-of-10 binary gate-closure score.

## 7. Quality and claim metrics

The solo project tracks evidence it can actually produce:

- deterministic test and fixture pass rate;
- malformed-input rejection, panic, hang, and resource-limit results;
- diagnostic stability and source-span accuracy;
- conformance coverage per implemented rule;
- differential mismatches between implemented paths;
- dependency and trusted-component count;
- offline build and replay success from a clean local environment;
- unresolved semantic, security, and compatibility questions; and
- exact claim/non-claim coverage for each artifact.

Contributor count, independent reviews, external pilots, certifications, and
laboratory results are unavailable metrics in solo mode and are not schedule
dependencies.

## 8. Definition of progress

Progress means a permanent boundary became more complete, deterministic,
documented, tested, and honest about its limitations. Lines of code, screenshots,
syntax breadth, or passing tests outside a stated boundary do not close a gate.

### Orange 1.0.0 completion metric

Orange is **30% complete toward Orange 1.0.0 by gate closure: 3 of 10 binary
gates are closed**.

The denominator is the nine top-level capability stages S0 through S8 plus one
final stable-release gate. S0 is closed for its D-023-directed repository
foundation scope; S1 and S2 are closed. S3 is active; S4 through S8 and the
stable-release gate remain open. Nested slices receive no separate credit:
completed S3a evidence advances S3 but does not close it.

The stable-release gate closes only when all ten 1.0 criteria in
[`PROJECT_CHARTER.md`](PROJECT_CHARTER.md#9-what-end-means), all eight
owner-executable journeys in [`USER_JOURNEYS.md`](USER_JOURNEYS.md), and all
applicable release-identity, solo-release-gate, stable-toolchain, and publication
requirements in [`RELEASE_POLICY.md`](../RELEASE_POLICY.md) are satisfied. The
score is `100 * closed gates / 10`; active work receives no fractional credit.

This percentage measures scope-gate closure. It is not an estimate of remaining
effort or time, an assurance-strength score, or a claim that a release is
currently authorized.

The roadmap changes when evidence or owner direction changes. A future
collaborative mode may add review and operational capabilities, but the current
roadmap remains executable by one person without waiting for that event.
