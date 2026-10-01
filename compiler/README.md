# Orange compiler

Status: production-lineage, pre-alpha; S3a under accepted OEP-0003; S3b through
S3t proposed under OEP-0005 through OEP-0021, OEP-0023 and OEP-0024, in owner review

This workspace contains the first executable slice of the Orange compiler. It
is intentionally small, but its source identities, byte spans, language-edition
boundary, diagnostic codes, and deterministic token stream are permanent
interfaces to extend rather than a disposable prototype.

Nothing here makes a verification, correctness, constant-time, or production
readiness claim. `orangec check` performs lexical, syntactic, and bounded
semantic validation. The accepted S3a slice assigns meaning to closed typed
`spec` literals. The S3b slice, proposed in
[`docs/EXPRESSIONS_2026.md`](../docs/EXPRESSIONS_2026.md) and in owner review
under OEP-0005, extends it to pure typed `spec` functions: parameters, calls,
`Int` and `Word[8]` through `Word[64]`, exact integer arithmetic, word ring
arithmetic, bitwise operators, shifts, and rotations. The S3c slice, proposed
in [`docs/BINDINGS_2026.md`](../docs/BINDINGS_2026.md) and in owner review
under OEP-0006, adds typed `let` bindings and explicit `as` conversions among
those five types. The S3d slice, proposed in
[`docs/ARRAYS_2026.md`](../docs/ARRAYS_2026.md) and in owner review under
OEP-0007, adds fixed-length arrays `T^n` of those types, array literals, and
literal indices. The S3e slice, proposed in
[`docs/LOOPS_2026.md`](../docs/LOOPS_2026.md) and in owner review under
OEP-0008, adds loops over literal ranges, indices computed from loop indices and
proved in range before evaluation, updates of one element, and fill literals.
The S3f slice, proposed in
[`docs/CONDITIONS_2026.md`](../docs/CONDITIONS_2026.md) and in owner review
under OEP-0009, adds `Bool`, comparisons, strict logical operators, total
Euclidean division and remainder, and conditionals that always have both
branches. The S3g slice, proposed in
[`docs/LOOKUPS_2026.md`](../docs/LOOKUPS_2026.md) and in owner review under
OEP-0010, lets an index depend on data: an index whose first typed leaf is a
word ranges over its type, narrowed by its operators, an `Int` index may
convert words with `as Int`, every index is still proved in range before
evaluation, and an update or fill costs one step per 64 elements. The S3h
slice, proposed in [`docs/MODULES_2026.md`](../docs/MODULES_2026.md) and in
owner review under OEP-0011, lets a program span several modules, one per
file: a module declares the modules it uses, calls their functions as
`m::f(...)`, and is checked once, after them; `orangec` reads the module `m`
from `m.or` beside the root. The S3i slice, proposed in
[`docs/MODULAR_2026.md`](../docs/MODULAR_2026.md) and in owner review under
OEP-0012, adds `Mod[m]`, the integers modulo a constant m from 2 through
2^521 - 1 written as its standard writes it, as `Mod[(1 << 255) - 19]`, whose
`+`, `-`, and `*` reduce by themselves and whose `/` multiplies by an inverse
and gives 0 when there is none, and `type` declarations that name a type for
the rest of a module. The S3j slice, proposed in
[`docs/BLOCKS_2026.md`](../docs/BLOCKS_2026.md) and in owner review under
OEP-0013, lets a loop's step and each branch of a conditional begin with
`let` bindings, as a body does: a step's bindings are evaluated afresh at
every step, a branch's only when it is chosen, and each is in scope only
within its step or branch. The S3k slice, proposed in
[`docs/TUPLES_2026.md`](../docs/TUPLES_2026.md) and in owner review under
OEP-0014, adds tuples: a tuple type `(T, U)` of two through 16 scalar or array
elements, a tuple `(a, b)`, the selection `.k` of element k, and tuple patterns
that name each element where a binding or a loop's accumulator is declared, so
that a function gives several values and a loop carries several accumulators.
The S3l slice, proposed in [`docs/BYTES_2026.md`](../docs/BYTES_2026.md) and in
owner review under OEP-0015, adds bytes: a byte string `"..."` or `hex"..."` is
the array `Word[8]^n` of its bytes, `++` joins two arrays, and a slice
`x[a..b]` and a slice update `x with [a..b] = v` read and replace a run of
elements whose bounds are literals and loop indices proved in range. The S3m
slice, proposed in [`docs/SIZES_2026.md`](../docs/SIZES_2026.md) and in owner
review under OEP-0016, adds sizes: a `spec` may declare size parameters with
finite ranges, `spec f[n in 1..5](x: Word[8]^n)`, and stands for one instance
for each value of its sizes, each checked as the function written out; sizes
write array lengths, fill lengths, and loop bounds; and a call names its
instance by its sizes, `f[2](x)`, or by its arguments' lengths. The S3n
slice, proposed in [`docs/ORDER_2026.md`](../docs/ORDER_2026.md) and in owner
review under OEP-0017, adds byte orders: `x as big T` and `x as little T` read
a word or an array of words as the words of another width with the same
number of bits, as an `Int`, or as a `Mod[m]`, and write an `Int` or a residue
as words, the first word most significant for `big` and least significant for
`little`, so that `block as big Word[32]^16` gives SHA-256's message words. The
S3o slice, proposed in
[`docs/TYPE_PARAMETERS_2026.md`](../docs/TYPE_PARAMETERS_2026.md) and in owner
review under OEP-0018, adds type parameters: a `spec` may list the types it is
written for, `spec pow[K in {F, P, Q}](x: K, e: Int) -> K`, and stands for one
instance for each listed type, each checked as the function written out with
that type; and a call names its instance by its types, `pow[F](x, e)`, or by
its arguments' types and, where they do not decide, the type its place
expects. The S3p slice, proposed in
[`docs/LENGTHS_2026.md`](../docs/LENGTHS_2026.md) and in owner review under
OEP-0019, lets an array, an array literal, and a byte string hold up to
65,536 elements, so that a `Word[16]` indexes the longest with no check at
run time, and gives `orangec eval` a step budget of its caller's choosing,
`--steps`, a choice of functions, `--spec`, and a report of the steps each
used, `--stats`. The S3q slice, proposed in
[`docs/TESTS_2026.md`](../docs/TESTS_2026.md) and in owner review under
OEP-0020, lets a module state its known answers as `test "TITLE" { claim }`
beside its functions, compares arrays and tuples whole with `==` and `!=`,
and adds `orangec test`, which runs the root module's tests and reports each.
The S3r slice, proposed in [`docs/AMOUNTS_2026.md`](../docs/AMOUNTS_2026.md)
and in owner review under OEP-0021, lets a shift or rotation take an amount
computed from data, an `Int` or a word, such as `x <<< r` or `x >> (i % 8)`,
with a meaning at every amount: a shift is multiplication or division by a
power of two kept to the word, so a shift by the width or more gives 0 and a
negative amount shifts the other way, and a rotation turns by its amount
modulo the width. The S3s slice, proposed in
[`docs/NESTED_ARRAYS_2026.md`](../docs/NESTED_ARRAYS_2026.md) and in owner
review under OEP-0023, adds arrays of scalar rows with exact shapes, at most
65,536 scalar elements, and chained indices. Rows retain their types through
updates, slicing, concatenation, tuples, and finite specialization. The S3t
slice, proposed in [`docs/STATIC_MODULI_2026.md`](../docs/STATIC_MODULI_2026.md)
and in owner review under OEP-0024, admits own finite size names in modulus
expressions. Every instance is checked eagerly with its exact concrete residue
domain, including signatures, body annotations, conversions and direct type
arguments. Module aliases and type-parameter lists remain concrete. All twenty
lower to a noncanonical Typed Reference Core and are reference-evaluated. Unbounded loops, typed `impl`, proof checking,
verified lowering, and code generation do not exist.

This boundary was merged by
[PR #9](https://github.com/chasebryan/orange/pull/9) as commit
`6c0bd3021cf2df603e08808e4660724ca1e2b2a5`. Orange remains pre-alpha; that
merge creates no stable public compatibility promise. Accepted D-003 and
OEP-0004 establish PF-01 at exact revision
`a82a5cec2ee4359dc2fe66171f17c93146747333`. Later S3 semantics remain
incomplete, and D-004 remains unresolved.

## Run

The workspace requires the pinned Rust 1.96.1 toolchain and has no third-party
Rust dependencies. This pre-alpha slice does not declare or test a lower MSRV.

```sh
scripts/ci/check-repository
cargo test --manifest-path compiler/Cargo.toml --workspace
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/hello.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- check compiler/fixtures/typed-answer.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/typed-answer.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3b/valid-sha256-functions.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3b/valid-chacha20-quarter-round.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3h/valid-vectors.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval compiler/fixtures/s3i/valid-x25519.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- eval --steps 2097152 --stats compiler/fixtures/s3p/valid-lengths.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- test compiler/fixtures/s3q/valid-rfc8439-tests.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- test compiler/fixtures/s3r/valid-rc6.or
cargo run --manifest-path compiler/Cargo.toml -p orangec -- lex compiler/fixtures/hello.or
cargo test --manifest-path compiler/Cargo.toml -p orangec --test s2_conformance --locked --offline
cargo test --manifest-path compiler/Cargo.toml -p orangec --test s3a_conformance --locked --offline
cargo test --manifest-path compiler/Cargo.toml -p orangec --test s3b_conformance --locked --offline
```

### Identify the compiler on your path

`orangec --version` reports the package version, edition, and latest
implemented language slice:

```console
$ orangec --version
orangec 0.0.1 (Orange edition 2026; implemented slice S3s)
```

The slice identifies implemented behavior; its proposal's acceptance status
is listed above. It is not a release version or a verification claim. Older
binaries reported the same `0.0.1` package version without a slice, so that
number alone does not show which syntax they support.

If a documented example is refused, compare the compiler on your path with
a fresh build of your checkout, from the repository root:

```sh
command -v orangec
orangec --version
cargo build --manifest-path compiler/Cargo.toml -p orangec --release --locked --offline
compiler/target/release/orangec --version
```

Run the example with `compiler/target/release/orangec` to use that build
directly. An older binary elsewhere on your path does not change when the
checkout is rebuilt.

## Sealing files

`orangec keygen`, `enc`, `dec`, and `schemes` seal files with authenticated
ciphers written in Orange. XChaCha20-Poly1305 (the default),
ChaCha20-Poly1305, and Ascon-AEAD128 are built in from [`schemes/`](schemes/),
and any Orange program with the sealing interface is a scheme too. Every byte
of cryptography runs on the reference evaluator through `Evaluator::call`,
which evaluates one specification on host values under its own step budget;
`crates/orangec/src/crypt.rs` only moves bytes, one evaluator per core.
[`schemes/README.md`](schemes/README.md) specifies the interface, the key file,
and sealed-file format 1, and states the limits: the evaluator is not
constant-time, nothing is verified, and keys are stored unencrypted.

```sh
cargo run --manifest-path compiler/Cargo.toml -p orangec -- schemes
cargo run --manifest-path compiler/Cargo.toml -p orangec -- keygen -o /tmp/demo.key
cargo run --manifest-path compiler/Cargo.toml -p orangec -- enc --key /tmp/demo.key -o /tmp/readme.orange README.md
cargo run --manifest-path compiler/Cargo.toml -p orangec -- dec --key /tmp/demo.key -o /tmp/readme.md /tmp/readme.orange
cargo test --manifest-path compiler/Cargo.toml -p orangec --test crypt --locked --offline
```

A sealing command that fails exits with status 1 and one of these codes; a
malformed command line is a usage error with status 2.

| Code | Failure |
| --- | --- |
| `ORC1009` | A key file is missing, malformed, readable by other users, made for another scheme, or already present where `keygen` would write. |
| `ORC1010` | A scheme is unknown, does not compile, or does not implement the sealing interface. |
| `ORC1011` | The file to seal or open cannot be read, or is too large to seal. |
| `ORC1012` | An output already exists or cannot be written. |
| `ORC1013` | A sealed file has a malformed header, another format version, or ends inside a chunk. |
| `ORC1014` | A chunk is not authentic; nothing is written. |
| `ORC1015` | Operating-system randomness is unavailable. |

## D-004 pre-epoch decision laboratory

The `orange-compiler` integration tests contain a standard-library-only,
non-product D-004 laboratory. Its v0.5 parser still authenticates the immutable
`draft_unfrozen` review subject byte for byte. A separate
`d004-v0.6-reviewed-protocol` layer records the `solo-reviewed` D004-PRE-01
disposition at exact review-subject revision
`7d09a27369649855ce987c76315271b0d34a20ef`. It preserves the 5-candidate by
5-case matrix, 14 required relationships, 10 hard gates, 26 named mutations,
73 candidate-neutral suite subjects, and five-graph/70-row mapping catalog:

The acceptance covers those immutable review subjects. The resulting v0.6
implementation closure remains `provisional_pending_exact_merged_revision`
until the validated overlay is available at an exact merged revision.

```sh
cargo test --manifest-path compiler/Cargo.toml -p orange-compiler \
  --test d004_decision_suite --locked --offline
```

The proposal manifest names 14 missing-edge, 13 identity-substitution, and five
each of ambiguity, unsupported-behavior, and domain-exhaustion proposals. The
three added identity targets bind the model, dependency manifest, and positive
subject. The cross-cutting and case-subject catalogs byte-materialize exactly
73 suite-only subjects under the immutable v0.5
`draft_unreviewed_input_only` snapshot. Closed parsers authenticate the ordered
proposal and named-mutation joins, case baselines, and per-subject identities,
then bounded structural integrity oracles check the fixed models in memory.
Those checks are not candidate executions and create no candidate observation,
result, evidence, verdict, or capability credit.

D004-PRE-01 finds the five fixture classes reviewed and sufficient only for
bounded suite coverage: 5 ambiguity, 14 missing-edge, 13 identity-substitution,
5 unsupported, and 5 resource-exhaustion subjects. It reviews the five
candidate graphs and 70 SR rows only as symmetric, falsifiable test hypotheses;
their semantic status remains unaccepted and their conformance remains
unresolved until execution. Candidate-adapter failure cannot satisfy an
unsupported subject, and domain exhaustion verifies no replay ceiling.

The laboratory also checks a closed, in-memory future-schema descriptor that
enumerates all 31 top-level future case-record fields, their nested shapes, and
their cross-field invariants. It binds both authenticated subject catalogs into
an exact 73-row observation oracle and the candidate-mapping catalog into five
exact graph/map identity rows, so a future result cannot supply or broaden its
own allowed outcomes or substitute another mapping. The descriptor defines a
10-field scheduled-slot preimage contract and fail-closed verdict conditions,
including checked resource bounds and SR dependency joins. The 25-slot identity
plan is expanded in repetition-major order to exactly three planned executions
per unit, or 75 reviewed schedule rows. Each repetition requires a fresh empty
candidate-specific cache and equality of the specified deterministic fields.
No concrete scheduled-execution digest exists until the epoch, packet identity,
and executable manifests are frozen.

This synthetic contract accepts no populated records, launches no process or
adapter, and persists nothing. The Python run harness in `tools/d004_run.py`
supplies the adapter, closed payload schemas, executable and dependency
manifests, enforcing isolation and result parsers. Epoch
`d004-e-4aaf8a83a01693d543c4` ran all 75 executions and closed 20 of 25
required candidate-case units with 75 of 75 result records,
contributor-produced and unreviewed; see the D-004 laboratory README. Selection
and conclusion remain null. The v0.8 harness in `tools/d004_v08_run.py` adds
SC-06 and SC-07. Epoch `d004-e-633e0aa831615cda3e06` ran all 105 executions and
closed 28 of 35 units with 105 of 105 result records, and the owner's
isolation-first rule leaves only ST-REL; that result is contributor-produced,
unreviewed and not a D-004 recommendation. D-004 remains proposed, S3b through
S3o are implemented and await owner review under OEP-0005 through OEP-0018, both
`roadmap_gate_credit` and `readiness_credit` remain `none`, and Orange's 3-of-10
(30%) binary gate-closure score is unchanged.

## D-005 decision laboratory

The `orange-compiler` integration tests also contain a standard-library-only,
non-product D-005 laboratory. It strictly parses and canonicalizes the
checked-in draft decision packet, binds its SHA-256 identity, verifies the exact
4-candidate by 8-case inventory and frozen resource ceilings, preserves all 50
case mutations and five historical v0.1 dangers, verifies the packet's bound
suite, schema, and legacy-manifest bytes, and prepares one deterministic 32-slot
replay schedule:

```sh
cargo test --manifest-path compiler/Cargo.toml -p orange-compiler \
  --test d005_decision_suite --locked --offline
```

The laboratory also prepares a draft adapter transport boundary. It constructs
canonical requests bound to the packet, replay plan, candidate, case, and
repetition identities, then validates synthetic already-captured process output
against a closed canonical response envelope and the packet's exact byte
ceiling. A valid envelope exposes only an `UnvalidatedPayload`; it is not a case
verdict, atomic claim outcome, evidence record, or recommendation. Nonzero
exit, signal, timeout, launch/I/O failure, unsupported isolation, truncation,
stderr, malformed or noncanonical JSON, identity substitution, and output
overflow all fail closed.

The same in-memory boundary expands the existing 32-slot base schedule across
two workspace identities and three render identities into exactly 192 unique
transport identities. Base slot is the outer key, workspace is the next key,
and render is the inner key; this is a canonical serialization order, not an
authorized physical execution order. A closed 32-record binding table supplies
the input-manifest and payload-schema digests reused by each slot's six
identities. Equal digests are allowed, so this draft does not decide whether a
future schema is global, per candidate, or per case.

For synthetic already-captured bytes, the laboratory can also create and
strictly revalidate a canonical integrity receipt. The receipt binds the exact
transport slot and request digest to termination, truncation flags, and raw
stdout/stderr lengths and SHA-256 digests while fixing isolation to
`not_evaluated`, payload status to `unvalidated`, and evidence status to `none`.
An exact in-memory inventory rejects missing, extra, reordered, duplicated, or
cross-slot observations. Receipt integrity does not make a failed capture pass:
the existing fail-closed response validator remains the next boundary, and no
opaque payloads are compared across repetitions.

There is intentionally no subprocess launcher or candidate payload validator.
The standard library alone cannot enforce the suite's future process-tree,
network, filesystem, CPU, memory, file, descriptor, and cleanup boundary, and
the draft epoch has not frozen executable identities or candidate schemas. The
laboratory therefore executes no candidate adapter and records 0/32 completed
candidate-case runs, no evidence, and no selection. It does not alter compiler
behavior, ratify a public claim schema, close D-005, or advance the version
1.0.0 gate count.

## D-006 pre-epoch decision laboratory

The integration tests also contain a standard-library-only, input-only D-006
laboratory. It strictly parses the draft-unfrozen packet and its seven-row case
index, binds their canonical identities and the exact raw bytes of the
unchanged proof-foundation decision suite, and verifies the candidate, case,
metric, gate, owner-scope, protocol-count, dependency, tool, resource, and
nonclaim inventories:

```sh
cargo test --manifest-path compiler/Cargo.toml -p orange-compiler \
  --test d006_decision_suite --locked --offline
```

Every DS-01 through DS-07 row records absent shared inputs and candidate
mappings, zero executable fixtures, unresolved coverage, and an active freeze
blocker. The laboratory enumerates the exact 14 Rocq/Lean candidate-case
identities in memory. That case-major inventory is a canonical serialization,
not a physical execution order.

D-004 and D-005 acceptance remains absent. Candidate tool versions, dependency
graphs, D-018 admissions, acquisitions, installations, resources, host and
timeout policy, adapters, result/replay schema, correction window, materiality
bands, and owner review remain absent or unassigned. The laboratory launches no
process, writes no research artifact, freezes no epoch, records 0/14 executions
and no evidence, and selects or recommends no proof foundation. It authorizes
no proof-bearing implementation and does not advance the version 1.0.0 gate
count.

## D-009 pre-epoch decision laboratory

The integration tests also contain a standard-library-only, input-only D-009
solver-trust laboratory. It strictly parses the draft-unfrozen packet and its
eight-row case index, binds their canonical identities and the exact raw bytes
of the unchanged solver-trust decision suite, and verifies the candidate,
case, metric, gate, owner-scope, protocol-count, dependency, candidate-state,
resource, and nonclaim inventories:

```sh
cargo test --manifest-path compiler/Cargo.toml -p orange-compiler \
  --test d009_decision_suite --locked --offline
```

Every TC-01 through TC-08 row records absent shared inputs and candidate
mappings, zero executable fixtures, unresolved coverage, and an active freeze
blocker. The laboratory enumerates exactly 24 candidate-case identities for
the checked-artifact, kernel-only, and direct trusted-solver strategies. That
case-major inventory is a deterministic in-memory serialization, not a
physical execution order.

The required decision dependencies remain unaccepted. Candidate
implementations, dependency admissions, adapters, resources, host and timeout
policy, result/replay schema, correction window, materiality bands, and owner
review remain absent or unassigned. The laboratory launches no process or
solver, writes no file or research artifact, adds no crate or dependency,
freezes no epoch, records 0/24 executions and no evidence, and produces or
checks no solver result, certificate, or proof. It selects or recommends no
solver-trust policy, authorizes no claim-closing credit, and does not advance
the version 1.0.0 gate count.

For a local source-install rehearsal, use a fresh private install root:

```sh
orange_install_root="$(mktemp -d)"
cargo install --path compiler/crates/orangec --root "$orange_install_root" \
  --bin orangec --profile release --locked --offline
"$orange_install_root/bin/orangec" eval compiler/fixtures/typed-answer.or
```

This exercises Cargo's local path installation without consulting the network.
It does not publish a crate, create or validate a release artifact, establish a
compatibility or support commitment, or complete `USER_JOURNEYS.md` J-01.

The protected repository gate runs all Rust targets in both debug and optimized
release profiles. The release profile retains debug assertions and integer
overflow checks, so optimization cannot silently weaken internal invariants;
the individual commands are useful for focused development. A separate
production-only Clippy pass denies unchecked arithmetic, silent `as`
conversions, UTF-8 string slicing, indexing, unwrap/expect, and explicit panic
sites while leaving test assertions available to state fixture invariants.
The same isolated gate fixes a private file-creation mask and captures one source
archive before Cargo runs. A sanitized, NUL-delimited Git index inventory admits
exactly tracked paths, while archive bytes come from the working tree so tracked
local edits are tested; untracked and ignored local state cannot enter. The
archive format, path order, timestamps, numeric owner/group fields, and file
modes are fixed: ordinary files are `0644`, admitted executables are `0755`,
and hard links are archived as independent files. Before Cargo runs, every
tracked path in the working tree and first extraction must be a regular,
non-symlinked file, their executable classifications and bytes must match, and a
fresh Git inventory must match the original path list, rejecting observed type,
executable-mode, content, or membership edits during capture. The copied
validator opens the archive and inventory through read-only descriptors, unlinks
their filesystem names, and closes those descriptors before any copied Python
or Rust code executes. Every copied command then runs as PID 1 with private
mount, PID, `/proc`, network, IPC, and UTS namespaces. The UTS namespace uses
the fixed `orange-gate` hostname, and the fresh IPC namespace starts without
System V message queues, semaphore sets, or shared-memory segments. The gate
uses an unprivileged user namespace when the host permits it. Otherwise, as on
Ubuntu 23.10 and newer, `make` asks `sudo` for the invoking account's password
once unless a cached credential or passwordless rule already covers it. That
path needs a `sudo` policy that keeps the credential between commands and a host
that allows user namespaces (`user.max_user_namespaces` above zero); otherwise
`make` stops with a message naming the setting. A fixed `sudo` supervisor then
creates those namespaces, switches to the invoking numeric user and group
holding only `CAP_SYS_ADMIN`, so the invoking account owns the user namespace it
creates next, and writes that namespace's one-line maps from outside it so they
admit only that user and group. `setpriv` then restores that user and group,
clears supplementary groups, and enables `no_new_privs`.
The user-namespace path retains its namespace-granted capabilities only long
enough for `setpriv` to remove every capability from the inheritable, permitted,
effective, bounding, and ambient sets. The privileged supervisor removes the
same sets while restoring the invoking identity. Both paths assert the restored
identity, distinct IPC and UTS namespace identities, fixed hostname, empty
System V IPC tables, five empty capability sets, private process view, and
empty route table.
Before the drop, the namespace supervisor bind-mounts the selected toolchain
read-only and covers `/home` with a private non-executable `tmpfs`. A protected C
launcher built from the captured tree requires Landlock ABI 3 or newer, permits
directory-name traversal but grants file reads and execution only to `/usr`, the
gate launcher's private `/proc/1`, the private `/proc/sysvipc` tables, the gate
tool roots, and the selected toolchain, and grants
writes only to the two gate roots and four admitted character devices. Host
account/configuration files under `/etc` and kernel/device state under `/sys`
remain unreadable, with representative runtime assertions. The launcher also
closes every inherited descriptor above standard error, resets ordinary
catchable signal dispositions to default, and empties the ordinary signal mask
before execution. The launcher also fixes hard
ceilings of 4 GiB of virtual address space and 600 CPU seconds per process,
512 MiB per file, 1,024 open files, 256 processes for the real user inside the
gate's private user namespace (the kernel exempts the global root user, so a gate
invoked as root has no process ceiling), and zero core-file bytes, preserving any lower inherited hard ceiling. Copied commands
receive isolated `HOME`, `TMPDIR`, and `PATH` values, disable system Git
configuration, and bind global Git configuration to `/dev/null`; a runtime
assertion confirms that the original checkout is unreadable. The namespace
supervisor kills its child if supervision is interrupted. Trusted gate operations
alone retain the descriptors used for later extraction and identity checks. The
copied validator first policy-checks
that exact exported tree before its foundation test modules import.
After those tests, it policy-checks the tree again before Cargo, so Python-test
drift cannot reach Rust execution. Formatting, linting, documentation, and Rust
tests use the same extracted check root. A third policy check runs after all Rust
commands. The gate then verifies that the original archive and path inventory
retained their captured identities, extracts a fresh reference, compares the
NUL-safe sorted non-directory membership of all three compiler input roots, and
compares every tracked file's type, complete mode, and bytes with the reference.
These exact comparisons reject added source entries and policy-valid
tracked-source drift before the gate can pass. Optimized
`orangec` builds use independently created temporary ancestors, relocated
source roots, separate Cargo homes, and separate target trees whose names differ
in bytes, length, and directory depth. Both artifacts must be regular
non-symlink files with identical complete modes and bytes. This is
source-relocated same-host reproducibility evidence, not a cross-platform or
independently rebuilt claim. The gate also installs `orangec` from the captured
source with fresh isolated Cargo, target, and install roots under
`--locked --offline`; the installed executable must match the reproducible
artifact's complete mode and bytes, produce no output for the typed-fixture
`check` on either output channel, and produce the exact expected three stdout
lines with empty stderr for `eval`. This remains a non-publishing source-install
rehearsal with the boundaries stated above.

Landlock does not mediate every metadata operation, and the gate intentionally
allows directory listing without file contents so the selected Rust toolchain can
discover its sysroot. Copied commands receive read-only `/dev/null` as standard
input and share one write-only anonymous pipe for standard output and error; a
trusted outer `/usr/bin/cat` relays that merged stream to the caller's final
output sink. The caller can still close or truncate that final sink, and the two
copied output channels are intentionally indistinguishable. The system C
compiler, launcher source, relay, kernel, mount implementation, and allowlisted
system roots remain trusted boundaries. The gate launcher's PID-1 metadata and
private System V IPC tables remain readable; representative global kernel/CPU
and dynamic `/proc/self` content is asserted unreadable. Resource ceilings are not aggregate
cgroup budgets: virtual address space and CPU time are limited per process,
file size is limited per file, aggregate resident memory is not capped, and the
process ceiling counts only processes in the gate's private user namespace, not
other processes of the same account.

`orangec` accepts up to 256 source inputs in argument order. Argument parsing
inspects at most 4 MiB (`4 * 1024 * 1024` bytes) of encoded command-line
arguments per invocation, charged before each argument is interpreted.
Exceeding the byte allowance is a usage error before any source read. Regular
files are processed incrementally; `-` is the only stream input and reads
standard input at most once. Integration coverage requires exactly 256 valid
inputs to succeed silently and 257 operands to fail as a usage error before any
source read. It
also interleaves file, standard-input, and file failures in exact operand order;
a repeated `-` emits exactly one `ORC1004` group and still processes a later
operand. The global `--edition` option may appear before or after the command
but at most once; a repeated split or inline form is a usage error before any
source read. `--` ends option parsing so dash-prefixed source paths remain
addressable.
The portable regular-file boundary checks path-entry metadata without following
a final symlink before opening and after reading, rejecting an observed symlink
as non-regular. It checks descriptor metadata after opening and again after
reading, and requires the final path entry to remain a regular file. Linux
x86-64 and AArch64 opens also request `O_NOFOLLOW | O_NONBLOCK`, so a final
symlink swap fails at the descriptor open and a swapped FIFO cannot wait for a
peer. On Unix, the opened descriptor's device, inode, mode, owner, group, link
count, length, modification time, and change time must match both path
snapshots and remain stable through the read. The completed byte snapshot must
also have exactly the descriptor's reported length. Before the final metadata
comparison, `orangec` seeks the same opened descriptor to offset zero and
requires a second bounded read to match every retained byte plus exact EOF. The
verification read allocates no second source snapshot and does not charge the
invocation's buffered-source allowance twice. Other hosts compare length and
modification time at each boundary. This remains short of race-free path
confinement: portable-host opens can still block on a swapped special file,
parent components are not confined, a path can change away and back between
snapshots, and coordinated mutation or unusual filesystem semantics that
reproduce the same bytes and metadata across both reads can evade the
comparison. Compile untrusted filesystem trees from a stable copied file or
standard input inside an appropriate host sandbox; full path confinement is not
claimed.
A source whose module has `use` declarations is the root of a program. For
`check`, `eval`, and `test`, each `use m;` reads the module `m` from the file `m.or` in
the root file's directory, or in the current directory when the root is `-`.
A module name is an ASCII identifier, so it names one file in that directory
and no path outside it. Each module is read once per program, in the order a
`use` first names it, through the same regular-file boundary, 16 MiB
per-source limit, UTF-8 check, and shared per-invocation source budget as a
named source, and at most 64 modules besides the root are read. A file that
declares a module of another name is kept, so that the module graph reports
the `use` that read it, but its own uses are not followed. A module that
cannot be read is `ORC1001` with a note naming the `use` and its module; any
failure to read, decode, lex, or parse a module stops that program before
semantic analysis. `lex` reads no module, and each operand of an invocation is
the root of its own program. A scheme program given to the sealing commands by
path reads its modules the same way, under one 64 MiB budget shared with its
own bytes.
`eval` accepts exactly one source and begins output only after complete
validation and evaluation. A host output failure can leave an
already-written prefix, but returns status 1; a broken pipe remains quiet and
is never reported as successful evaluation. Once a standard-output or
standard-error write failure is observed, later source operands are not read or
compiled, and a partially accepted diagnostic prefix is not retried. Ordinary
source failures still aggregate diagnostics across later inputs. Successful
`check` commands are silent. Diagnostics go to standard
error and use exit status 1; distinct compiler or host error groups have exactly
one blank separator with no leading or extra trailing blank group.
Every parser, semantic-analysis, and evaluation result is classified
fail-closed: diagnostics take precedence, an artifact is accepted only without
diagnostics, and an absent artifact without diagnostics emits `ORC1006` as an
internal compiler or resource failure.
Command-line usage errors use status 2 when their diagnostic is written; a
detected usage-output failure uses status 1 without reading source input. A
usage diagnostic has one blank separator before the exact help text and one
trailing newline, while help and version output failures follow the same status
1 transport rule. All three paths flush explicitly and treat a detected flush
failure as status 1. Transient `Interrupted` results from source reads,
verification seeks, output writes, and explicit output flushes are retried
without duplicating accepted bytes. They fail closed
after 1,024 consecutive attempts for one operation.
Source reads that reach this boundary retain `ORC1001` and identify the exact
retry limit instead of attributing the local limit to the operating system.
Every output adapter rejects an impossible write count larger than the
offered byte slice. Compilation diagnostics are also explicitly flushed after
their final error group. When diagnostics and buffered token output are both
pending, the diagnostic stream is flushed first so a detected diagnostic-flush
failure discards token bytes that have not escaped the process. `orangec` caps standard error at 64 MiB (`64 * 1024 * 1024` bytes)
per invocation. Reaching the cap returns status 1 and stops before later source
operands; because the diagnostic channel itself is exhausted, an already
accepted prefix can end without a final limit notice. After any detected stream
failure, retained buffered standard output is discarded instead of being
flushed as later command output.
Compilation standard output is explicitly flushed only after successful token
or evaluation bytes have been queued; untouched output and diagnostic streams
are not flushed for a silent `check` or empty `eval`. A source with lexical
errors is not parsed, and a source with syntax errors is not analyzed. File and
standard-input reads stop at a deterministic 16 MiB per-source limit. Larger
individual inputs fail with `ORC1003` before lexing. `orangec` buffers at most
64 MiB (`64 * 1024 * 1024` bytes) across all source operands per invocation;
the first operand that would exceed the remaining total budget fails with
`ORC1008`. Bytes consume that shared budget as soon as they are read into the
bounded input buffer, even when the operand is later rejected. The one-byte
probe used to diagnose per-source overflow is also charged whenever aggregate
budget remains, so a rejected oversized operand cannot donate that byte to a
later operand. Once no aggregate budget remains, a reader may consume one
unbuffered probe byte only to distinguish end of input from overflow. Source
bytes are never buffered without a bound. CLI-derived rendered source names
reserve their complete escaped representation before encoding.
Source-map slots, borrowed
source-name and source-text copies, and derived line/column indexes also use
checked reservations; an allocation failure rejects the source through
`ORC1005` without exposing partial source state or consuming an insertion ID.
Already owned `String` inputs move into the map without an additional
source-data copy.
Lexing uses bounded amortized fallible growth while preserving one allocated EOF
slot and never requesting speculative capacity beyond the complete token-stream
limit. It fallibly reserves the complete 102-record diagnostic-vector bound
before scanning. Failure to reserve that vector exposes only the allocation-free
EOF fallback and is classified by the CLI as a fail-closed `ORC1006` internal
resource failure. A token-storage reservation failure emits `ORC0008`, discards all
ordinary tokens, and cannot expose a parser-acceptable partial stream; failure
to reserve even the initial heap slot uses an allocation-free inline EOF
fallback, so the public token stream still contains exactly one final EOF. An
impossible internal UTF-8 cursor mismatch follows the same atomic rejection
boundary: it emits `ORC0008`, discards partial tokens, and exposes only EOF.
String scanning determines its closing quote or line/end-of-file boundary
before admitting invalid-escape diagnostics. An unterminated-string diagnostic
anchored at the opening quote therefore precedes later escape diagnostics in
both the raw result and the ordinary-diagnostic budget. Escaped contents are
rescanned at most once, preserving bounded linear work without a pending-error
allocation.
Parsing reserves every owned identifier copy and each module-function slot
before installing them, and fallibly pre-reserves its complete 102-record
diagnostic-vector bound. Identifier or declaration reservation failure emits
`ORC0106`; diagnostic-vector reservation failure returns no AST or diagnostic
and is classified by the CLI as `ORC1006`.
Semantic analysis reserves and deterministically sorts the complete declaration
namespace index, checks
exact-integer limb growth, owned Core-name copies, and each pending
typed-function slot, then reserves the complete Core function table before
installing its first entry. It also fallibly pre-reserves its complete
102-record diagnostic-vector bound. Ordinary representation failures emit
`ORC0209`; diagnostic-vector reservation failure returns no Core or diagnostic
and is classified by the CLI as `ORC1006`. Identifier spellings echoed by
semantic diagnostics are capped at 64 bytes plus a deterministic total-length
suffix.
Lexical, parser, and semantic reporting admit an ordinary diagnostic before
constructing its owned message, label, note, or secondary-span fields.
Post-limit attempts create at most the one suppression record and construct no
discarded ordinary diagnostic.
Reference evaluation reserves the complete value-set vector before evaluating
the first function and checks every copied function name and exact-integer limb
vector. It also fallibly reserves its single possible diagnostic slot before
evaluating. Ordinary reservation failures emit `ORC0301` and expose no partial
value set; diagnostic-slot reservation failure returns no values or diagnostic
and is classified by the CLI as `ORC1006`. The shared module-name `Arc` control
block still uses the standard infallible allocator API because stable Rust does
not provide a fallible `Arc` constructor.
These checked container reservations do not make the entire diagnostic path
out-of-memory recoverable. The infallible `RenderedSourceName::from_text` and
`RenderedSourceName::from_os_str` convenience constructors, owned diagnostic
messages/labels/notes, owned CLI usage and error strings, rendered diagnostic
output, and the shared `Arc` control block still use standard infallible Rust
allocation APIs. The CLI uses the fallible rendered-name constructors. The
input and output bounds limit amplification, but process-level allocator
exhaustion can still abort instead of producing an Orange diagnostic.
Within that residual boundary, diagnostic messages, labels, notes, and source
names are escaped directly into the final rendered output instead of first
materializing expanded copies. Excerpt escaping writes into only the bounded
40-before/80-after window described below.
Exact-integer decimal display uses fixed stack arrays sized from the normative
16,384-bit limit, then writes base-1,000,000,000 limbs directly to the
destination without heap scratch or a materialized decimal output string.

Diagnostic source excerpts include at most 40 Unicode scalars before and 80
from the responsible position. Unit coverage renders the full 100-diagnostic
frontend budget beside a 1 MiB line and requires the complete deterministic
output to remain below 64 KiB, preventing line length from multiplying output
memory per diagnostic.

CLI-derived source names, diagnostic excerpts, and echoed command or option
text use an ASCII-safe escape representation for backslashes, controls, and
non-ASCII scalars. Invalid encoded path bytes use `\xNN` identities rather than
lossy replacement, and literal escape-looking path input remains
distinguishable from the byte it resembles. Library-provided raw source names
are escaped injectively during diagnostic rendering; the CLI passes a tagged
canonical name so its already encoded OS path is not escaped a second time.

`orangec lex` streams token records and escapes each spelling through a fixed
4 KiB scratch buffer instead of materializing an expanded token string. Lex
output for multiple sources uses an exact `== SOURCE ==` header and one blank
separator in argument order. A lexical error returns status 1 but does not
suppress that source's bounded token stream or later sources. Lex output still
can be larger than the accepted source because of escape notation and per-token
metadata. `orangec` caps standard output at 64 MiB (`64 * 1024 * 1024` bytes)
per invocation. Reaching that limit returns status 1, emits `ORC1007`, stops
before reading later sources, and can leave an already-accepted output prefix;
callers processing untrusted input should also cap time.

Accepted S3a assigns no separate semantic budget to evaluation-output bytes.
Each successful output line repeats the module name, so a source with a long
module name and many typed specifications can request output much larger than
its input. The CLI shares the evaluated module identity and streams values
through a fixed buffer, while the operational 64 MiB standard-output ceiling
above bounds the bytes actually accepted per invocation. Exceeding it is an
unsuccessful output operation rather than an accepted partial evaluation.
Apply caller-side time limits before using `orangec eval` on untrusted sources.
`--steps` raises the evaluation budget up to 1073741824 steps, and an
evaluation makes at most 64 array elements for each step, so a caller who
raises it on an untrusted source should also cap its memory. The same holds
for `orangec test`, whose report prints the values a failed `left == right`
compared, so a test over secret material discloses it when it fails.

## Frozen lexical boundary

The only supported language edition is Orange 2026. Its current lexical rules
are deliberately conservative:

- each source is at most 16 MiB of UTF-8, and spans are half-open UTF-8 byte
  ranges;
- each source map receives a unique nonzero process-local identity; exhausting
  the 64-bit identity space is a sticky failure and can never wrap into an
  earlier map's span ownership; the CLI uses the fallible constructor and
  reports exhaustion as a source-representation error;
- whitespace is limited to tab, line feed, carriage return, and space; line
  feed, CRLF, and bare carriage return each form one logical line ending;
- identifiers use ASCII letters, digits, and `_` (the first character cannot be
  a digit);
- `edition`, `module`, `spec`, `impl`, `game`, `proof`, and `claim` are reserved;
- decimal, `0b` binary, and `0x` hexadecimal integers allow single underscores
  between digits;
- quoted strings have a small, validated escape set and cannot cross lines;
- `//` comments and nested `/* ... */` comments are trivia; and
- punctuation outside the minimal grammar is still tokenized but has no
  accepted syntactic or semantic role.

Adding syntax requires an edition-aware decision. Token names and `ORCxxxx`
diagnostic meanings are stable automation surfaces; wording and source excerpts
may improve without reusing a code for a different error.

## Orange 2026 grammar

The parser accepts exactly one edition declaration followed by exactly one
module. Empty `spec` and `impl` functions remain valid. A typed `spec` declares
its parameters, one result type, and a body of `let` bindings followed by one
result expression:

```text
source_file     = edition_decl module_decl EOF ;
edition_decl    = "edition" "2026" ";" ;
module_decl     = "module" IDENTIFIER "{" use_decl* type_decl* member* "}" ;
member          = function_decl | test_decl ;
use_decl        = "use" IDENTIFIER ";" ;
type_decl       = "type" IDENTIFIER "=" declared_type ";" ;
function_decl   = "spec" IDENTIFIER "(" ")" spec_tail
                | "spec" IDENTIFIER size_params? "(" parameters? ")" typed_tail
                | "impl" IDENTIFIER "(" ")" empty_body ;
test_decl       = "test" STRING "{" binding* expression "}" ;
size_params     = "[" size_param ("," size_param)* "]" ;
size_param      = IDENTIFIER "in" (INTEGER ".." INTEGER | type_list) ;
type_list       = "{" declared_type ("," declared_type)* "}" ;
spec_tail       = empty_body | typed_tail ;
typed_tail      = "->" declared_type "{" binding* expression "}" ;
binding         = "let" pattern "=" expression ";" ;
pattern         = typed_name | "(" typed_name ("," typed_name)+ ","? ")" ;
typed_name      = IDENTIFIER ":" declared_type ;
empty_body      = "{" "}" ;
parameters      = parameter ("," parameter)* ","? ;
parameter       = IDENTIFIER ":" declared_type ;
declared_type   = element_type | tuple_type ;
tuple_type      = "(" element_type ("," element_type)+ ","? ")" ;
element_type    = parsed_type ("^" size)? ;
size            = INTEGER | IDENTIFIER | "(" expression ")" ;
parsed_type     = "Mod" "[" expression "]" | IDENTIFIER ("[" INTEGER "]")? ;

expression      = arithmetic | chain("&") | chain("|") | chain("^") | shift
                | conversion | update | comparison | chain("&&")
                | chain("||") | division | chain("++") ;
arithmetic      = product (("+" | "-") product)* ;
product         = prefixed ("*" prefixed)* ;
chain(op)       = prefixed (op prefixed)+ ;
shift           = prefixed shift_operator prefixed ;
shift_operator  = "<<" | ">>" | "<<<" | ">>>" ;
comparison      = prefixed ("==" | "!=" | "<" | "<=" | ">" | ">=") prefixed ;
division        = prefixed ("/" | "%") prefixed ;
conversion      = prefixed "as" (parsed_type | tuple_type | order declared_type) ;
order           = "big" | "little" ;
update          = prefixed "with" "[" (expression | range) "]" "=" expression ;
prefixed        = literal | ("-" | "~" | "!") prefixed | primary ;
literal         = "-"? INTEGER ;
primary         = IDENTIFIER suffix? | call suffix? | "(" expression ")"
                | byte_string | tuple | array | fill | loop | conditional ;
byte_string     = STRING | HEX_STRING ;
suffix          = "." INTEGER (index | slice)? | index | slice ;
slice           = "[" range "]" ;
range           = expression ".." expression? | ".." expression ;
tuple           = "(" expression ("," expression)+ ","? ")" ;
call            = (IDENTIFIER "::")? IDENTIFIER sizes? "(" arguments? ")" ;
sizes           = "[" expression ("," expression)* "]" ;
arguments       = expression ("," expression)* ","? ;
index           = "[" INTEGER "]" | "[" expression "]" ;
array           = "[" expression ("," expression)* ","? "]" ;
fill            = "[" expression ";" size "]" ;
loop            = "for" IDENTIFIER "in" size ".." size
                  "with" pattern "=" expression block ;
conditional     = "if" expression block "else" alternative ;
alternative     = block | conditional ;
block           = "{" binding* expression "}" ;
```

`let`, `as`, `for`, `in`, `with`, `if`, `else`, `use`, and `type` are
contextual: they are ordinary names everywhere except where a binding, a
conversion, a loop, a size parameter, an update, a conditional, or, at the
head of a module, a `use` or `type` declaration begins, and `Mod` takes a
modulus only when a
bracket follows it. `hex` begins a hex string only when a quote follows it
directly, and a name followed by brackets is a call with sizes or types only
when the brackets hold integers, names, a name's `[n]`, `+`, `-`, `*`, `/`,
`%`, `^`, commas, and parentheses and `(` follows them. `big` and `little` are byte orders only
directly after `as` and before `(` or a name other than `as` and `with`, and
only after one may a conversion's type have a length. `true` and `false` are the `Bool` values only
where no parameter, binding, or loop name of that spelling is in scope. For
example:

```orange
edition 2026;
module demo {
  type Z7 = Mod[7];
  spec identity() {}
  impl rounds() {}
  spec answer() -> Int { 42 }
  spec mask() -> Word[8] { 0xff }
  spec big_sigma0(x: Word[32]) -> Word[32] {
    (x >>> 2) ^ (x >>> 13) ^ (x >>> 22)
  }
  spec sample() -> Word[32] { big_sigma0(0x6a09_e667) }
  spec load_le16(b: Word[8]^2) -> Word[16] {
    let low: Word[16] = b[0] as Word[16];
    low | ((b[1] as Word[16]) << 8)
  }
  spec pair() -> Word[16]^2 { [load_le16([0x34, 0x12]), 0xbeef] }
  spec reverse(x: Word[8]^4) -> Word[8]^4 {
    for i in 0..4 with r: Word[8]^4 = [0; 4] { r with [i] = x[3 - i] }
  }
  spec backwards() -> Word[8]^4 { reverse([1, 2, 3, 4]) }
  spec sign(x: Int) -> Int { if x < 0 { -1 } else if x == 0 { 0 } else { 1 } }
  spec residues() -> Int^2 { [-7 % 2, sign(-7 / 2)] }
  spec field() -> Z7^2 { [3 * 5, 1 / 3] }
  spec squares() -> Int { for i in 0..4 with s: Int = 0 { let sq: Int = i * i; s + sq } }
  spec divmod(a: Int, b: Int) -> (Int, Int) { (a / b, a % b) }
  spec split() -> Int { let (q: Int, r: Int) = divmod(17, 5); q * 10 + r }
  spec fib() -> (Int, Int) { for i in 0..10 with (a: Int, b: Int) = (0, 1) { (b, a + b) } }
  spec greeting() -> Word[8]^5 { "Hi" ++ hex"20 21" ++ "!" }
  spec middle() -> Word[8]^3 { greeting()[1..4] }
  spec zeros[n in 1..3]() -> Word[8]^n { [0; n] }
  spec total[n in 1..9](x: Int^n) -> Int { for i in 0..n with s: Int = 0 { s + x[i] } }
  spec six() -> Int { total([1, 2, 3]) }
  spec text() -> Word[32] { "abcd" as big Word[32] }
  spec bytes() -> Word[8]^4 { let w: Word[32] = 0x01020304; w as little Word[8]^4 }
  spec double[K in {Word[8], Z7}](x: K) -> K { x + x }
  spec doubled() -> (Word[8], Z7) { (double(200), double[Z7](5)) }
  spec one[K in {Int, Z7}]() -> K { 1 }
}
```

The only precedence is that prefix operators bind first and `*` binds more
tightly than `+` and `-`. Operators from different groups, two shifts, two
comparisons, or two divisions at one level are `ORC0108`, so
`(x >>> 2) ^ (x >>> 13)`, `(a * b) % p`, and `(a < b) && (b < c)` need their
parentheses. A `-`
immediately before an integer token is that literal's sign, so the S3a body
`{ -42 }` is still one literal.

The parser accepts generic type syntax so unsupported forms receive semantic
diagnostics. Semantics admits exactly `Int`, `Word[8]`, `Word[16]`, `Word[32]`,
`Word[64]`, `Bool`, and `Mod[m]`, the names of earlier `type` declarations,
in a function with type parameters their names,
arrays `T^n` of them with n from 1 through 65536, and tuples `(T, U, ...)` of two
through 16 of those types and arrays, none of them a tuple, and checks
every expression against an expected type with no inference or coercion. `Int` is mathematical within the evaluator's resource
bounds and never wraps. `Word[n]` is the ring of integers modulo 2^n: `+`, `-`,
and `*` wrap because that is their meaning, while a literal must already fit
and never coerces, truncates, or wraps. A shift or rotation amount written as
one integer literal is unsigned and from 0 through n - 1; any other amount is
computed, an `Int` or a word of any width, and has a value at every amount: a
shift by n or more gives 0, a negative amount shifts the other way, and a
rotation turns by its amount modulo n. Division is Euclidean on `Int` and
unsigned on words, and total: `-7 % 2` is 1, `x / 0` is 0, and `x % 0` is x.
`Bool` has only `!`, `&&`, `||`, `==`, and `!=`, both operands of `&&` and `||`
are always evaluated, and no conversion joins it to a number. `Mod[m]` holds the
least residues 0 through m - 1 of a modulus that is a constant of literals,
`+`, `-`, `*`, `<<`, and parentheses; its literals lie strictly between -m and
m, it has `+`, `-`, `*`, `/`, prefix `-`, `==`, and `!=` of one modulus and no
order, and `x / y` is 0 when y has no inverse. `as` converts among `Int`,
words, and residues by least residues, so `t[x as Int]` indexes by a residue.
With a byte order, `x as big T` and `x as little T` read a word or an array
of words as the words of another width with the same number of bits, as an
`Int`, or as a `Mod[m]`, and write an `Int` or a residue as words by its
residue modulo 2 to the power of their width, the first word most significant
for `big` and least significant for `little`.
`p.k` selects element k of a tuple, counted from zero, and no operator,
order, conversion, or index applies to a whole tuple. `==` and `!=` are
defined for every type and compare arrays and tuples whole, every part
whether or not an earlier one differs, and an array or tuple written out
takes its type from the other operand. A byte string
`"..."` of printable ASCII characters and escapes, or `hex"..."` of hex digit
pairs, is the array `Word[8]^n` of its 1 through 65536 bytes; `a ++ b` joins two
arrays of one element type; and `x[a..b]` and `x with [a..b] = v` read and
replace the elements from index a up to b, whose bounds are built from
literals and loop indices and proved a fixed positive distance apart and in
range at every step. A conditional
evaluates only its chosen branch, bindings included. Names are the enclosing
function's parameters and earlier bindings, in a loop's step its index and
accumulator, and in a step or branch its own earlier bindings and those of the
steps and branches around it, each name of a tuple pattern among them; Orange
has no shadowing, so none of these may repeat a name in scope. Calls name typed `spec` functions of the same module, or, as
`m::f(...)`, of a module it uses, and each module's call graph must be
acyclic, as must the uses of a program. A `spec` with size parameters
`[n in a..b, ...]`, each with a < b ≤ 65536, is checked once for each value
of its sizes, as the function written out with that value; a size, built from integer literals and
size parameters with `+`, `-`, `*`, `/`, `%`, and parentheses, writes an array
length, a fill length, or a loop bound, and a size parameter's name is an
`Int` constant. A call `f[2](x)` names its instance by its sizes, and `f(x)`
names the one instance whose array parameters have its arguments' lengths.
A type parameter `[K in {T, U, ...}]` lists distinct types, each resolved
once and written without sizes; its name is a type in the function's
signature and body, not a value, and the function is checked once for each of
its types. A function has at most four size and type parameters and 256
instances in all, one for each combination of its sizes' values and types.
A call `f[T](x)` names its instance by its types as well as its sizes, and
`f(x)` names the one instance whose parameters have its arguments' types,
and among several, the one whose result has the type its place expects.
A loop runs over bounds that are literals or sizes with
0 ≤ a < b ≤ 65536, and every index is proved in range before evaluation: an
expression of literals, sizes, and loop indices by its values, and an index
keyed by data by the range of its word type. Duplicate names are syntactically valid, then semantic
analysis rejects a duplicate within the same declaration-kind namespace or
parameter list. Empty declarations have no value, and a typed `impl` remains a
syntax error.

The reusable syntax-tree and Typed Reference Core nodes, together with parser,
analysis, and evaluation result envelopes, are read-only outside the compiler
crate. Callers can inspect parsed, checked, and evaluated spans, names,
source-ordered declarations, identities, types, and values through accessors,
but cannot mutate compiler-established structure or replace a checked or
evaluated value.
Parsing rejects lexer output paired with a different source as `ORC0107`, even
when that lexer output already contains errors. Semantic analysis rejects a
syntax tree paired with a different source as `ORC0210`. A Core function's
reported type is derived from its value, so a type/value mismatch is not
representable at the public Core boundary.

`orangec eval` prints every typed specification without parameters of the
root module in source order, every instance of one with sizes or types in
order and named by them; the functions of the modules it uses run only
when called. Functions with parameters are checked but run only when called,
and words print as fixed-width lowercase hexadecimal:

```text
demo::answer: Int = 42
demo::mask: Word[8] = 0xff
demo::sample: Word[32] = 0xce20b47e
demo::pair: Word[16]^2 = [0x1234, 0xbeef]
demo::backwards: Word[8]^4 = [0x04, 0x03, 0x02, 0x01]
demo::residues: Int^2 = [1, -1]
demo::field: Mod[7]^2 = [1, 5]
demo::squares: Int = 14
demo::split: Int = 32
demo::fib: (Int, Int) = (55, 89)
demo::greeting: Word[8]^5 = [0x48, 0x69, 0x20, 0x21, 0x21]
demo::middle: Word[8]^3 = [0x69, 0x20, 0x21]
demo::zeros[1]: Word[8]^1 = [0x00]
demo::zeros[2]: Word[8]^2 = [0x00, 0x00]
demo::six: Int = 6
demo::text: Word[32] = 0x61626364
demo::bytes: Word[8]^4 = [0x04, 0x03, 0x02, 0x01]
demo::doubled: (Word[8], Mod[7]) = (0x90, 3)
demo::one[Int]: Int = 1
demo::one[Z7]: Mod[7] = 1
```

Three options shape an evaluation. `--steps N` sets the step budget of the
whole evaluation, shared by every function it evaluates, from 1 through
1073741824 (1,024 times the default of 1048576); a step-limit diagnostic
names the option while the budget is below that. `--spec NAME`, repeatable
for up to 64 names, evaluates only the named functions without parameters of
the root module, every instance of one with sizes or types, in source order,
and checks the rest; a name that matches none is `ORC1016` and evaluates
nothing. `--stats` writes one line to standard error for each evaluated
function and a total against the budget, after the values:

```console
$ orangec eval --steps 2097152 --spec pepin --stats compiler/fixtures/s3p/valid-lengths.or
lengths::pepin: (Mod[65537], Mod[65537], Bool) = (65536, 21846, true)
lengths::pepin: 1452583 steps
total: 1452583 of 2097152 steps
```

`--steps` and `--stats` also apply to `orangec test`; each of the three is a
usage error with any other command, and `--spec` with `test`.

`orangec test` checks exactly one program as `check` does and runs the root
module's known-answer tests in source order under one step budget. A test is
`test "TITLE" { bindings; claim }` among a module's functions, its title 1
through 128 printable ASCII characters without a backslash and unique in its
module (`ORC0242`), and its claim a `Bool` checked as a function without
parameters; the tests of used modules are neither checked nor run, and
`orangec eval` runs no test. The report is written to standard output, one
line per test and a count, with both values and the first difference of a
failed `left == right`; status is 0 when every test passes and 1 when any
fails, with standard error empty:

```console
$ orangec test compiler/fixtures/s3q/failing-tests.or
test "a word, rotated" ... ok
test "a word, rotated the wrong way" ... FAILED
    left:  0x00000080
    right: 0x00000100
test "an array" ... FAILED
    left:  [0x01, 0x02, 0x03, 0x04]
    right: [0x01, 0x02, 0x09, 0x04]
    first difference at [2]
test "a tuple holding an array" ... FAILED
    left:  (0x01, [0x02, 0x03, 0x04])
    right: (0x01, [0x02, 0x03, 0x05])
    first difference at .1[2]
test "a residue" ... FAILED
    left:  1
    right: 2
test "two claims at once" ... FAILED
test "unequal, as claimed" ... ok
7 tests: 2 passed, 5 failed
```

A test that exceeds the budget writes no report: `ORC0301` at its title, with
the note that no test outcome is reported. `--stats` writes each test's steps
and the total to standard error after the report.

The accepted S3a rules and non-claims are in
[`docs/SEMANTICS_2026.md`](../docs/SEMANTICS_2026.md), and the proposed S3b
through S3s rules, limits, and non-claims are in
[`docs/EXPRESSIONS_2026.md`](../docs/EXPRESSIONS_2026.md),
[`docs/BINDINGS_2026.md`](../docs/BINDINGS_2026.md),
[`docs/ARRAYS_2026.md`](../docs/ARRAYS_2026.md),
[`docs/LOOPS_2026.md`](../docs/LOOPS_2026.md),
[`docs/CONDITIONS_2026.md`](../docs/CONDITIONS_2026.md),
[`docs/LOOKUPS_2026.md`](../docs/LOOKUPS_2026.md),
[`docs/MODULES_2026.md`](../docs/MODULES_2026.md),
[`docs/MODULAR_2026.md`](../docs/MODULAR_2026.md),
[`docs/BLOCKS_2026.md`](../docs/BLOCKS_2026.md),
[`docs/TUPLES_2026.md`](../docs/TUPLES_2026.md),
[`docs/BYTES_2026.md`](../docs/BYTES_2026.md),
[`docs/SIZES_2026.md`](../docs/SIZES_2026.md),
[`docs/ORDER_2026.md`](../docs/ORDER_2026.md),
[`docs/TYPE_PARAMETERS_2026.md`](../docs/TYPE_PARAMETERS_2026.md),
[`docs/LENGTHS_2026.md`](../docs/LENGTHS_2026.md),
[`docs/TESTS_2026.md`](../docs/TESTS_2026.md),
[`docs/AMOUNTS_2026.md`](../docs/AMOUNTS_2026.md), and
[`docs/NESTED_ARRAYS_2026.md`](../docs/NESTED_ARRAYS_2026.md). None of them defines
unbounded loops, effects, proof meaning, implementation refinement, timing,
target behavior, ABI, leakage property, output code, package or release
behavior, or cryptographic construction. A function that evaluates to a
standard's example value is not thereby a verified transcription of that
standard.

## S2 conformance index

`docs/LANGUAGE_2026.md` groups the base D-025/OEP-0002 lexical and syntactic
boundary into 13 stable `S2-*` rule identifiers. The protected
`crates/orangec/tests/s2_conformance.rs` runner requires that exact rule
inventory and each rule's evidence-layer declaration, rejects missing, unknown,
duplicate, or uncovered identifiers, and binds every evidence entry to one
unconditional test at the expected integration- or unit-test harness depth.
The same check binds the exact Cargo workspace/package manifests and the
compiler crate's unconditional `source`, `lexer`, and `parser` module
registrations, so target-discovery, harness, feature-gating, or ancestor
`cfg` changes cannot silently remove mapped evidence. This is a registration
contract: the execution claim for every mapped test binary belongs to the
protected full repository gate, which clears caller Cargo configuration and
target runners. A standalone S2 target run checks the index and its direct
cases, but is not independent proof that the other mapped binaries ran.

The runner adds direct same-revision checks for the exact supported string
escape set, punctuation longest matching across every shared prefix, ASCII
trivia and uppercase radix prefixes, malformed production diagnostic codes and
spans, lexical exclusion of parsing, and repeated structural equality. It also
maps the existing source, lexer, parser, and CLI unit evidence for limits,
fail-closed phase and resource behavior, syntax-tree shape, diagnostic order,
and determinism. This
index adds traceability for the already accepted S2 syntax at revision
`52a3460853636f7cbaa27f3e27d86e032e3c82d4`; it does not assign the additive
typed-`spec` grammar to S2 or add a source construct, semantic rule, proof,
source-compatibility guarantee, or S3 authority.

## S3a CLI conformance corpus

`fixtures/s3a/` contains an exact ten-file black-box corpus for already accepted
S3a behavior. Three fixtures must evaluate successfully and seven must fail
closed. The corpus covers:

- empty declarations, mixed empty and typed declarations, and cross-kind equal
  names;
- positive, zero, negative, and negative-zero `Int` observations in decimal,
  binary, and hexadecimal source forms;
- exact `Word[8]` observations at 0, 1, 254, and 255;
- typed-`impl` syntax rejection; and
- the stable duplicate, unsupported-type, word-width, integer-magnitude,
  negative-word, and word-range diagnostic categories.

`crates/orangec/tests/s3a_conformance.rs` checks the directory inventory rather
than accepting an extra fixture implicitly. It invokes both `orangec check` and
`orangec eval` twice per fixture and requires identical status, standard output,
and standard error. Accepted cases have silent checking, exact evaluation
output, and no diagnostic. Rejected cases have status 1, no partial output, the
exact ordered diagnostic-code sequence, the expected diagnostic meaning, and
the exact primary line and column for every diagnostic. Check and evaluation
rejection bytes must agree.

The same protected runner binds every named evidence entry to one unconditional
test at its true integration- or unit-test harness location. It rejects
comment, string, nested-function, macro-token-tree, and controlling-attribute
lookalikes; binds the exact Cargo workspace and package manifests; and requires
unconditional compiler-crate registration of `core`, `eval`, `parser`,
`semantics`, and `source`. This establishes registration. The execution claim
for all mapped test binaries belongs to the protected full repository gate,
which clears caller Cargo configuration and target runners; a standalone S3a
target run does not show that the other mapped binaries executed.

Six generated black-box cases exercise boundaries and combinations that are
impractical as ordinary expected-output fixtures. They cover the exact
significant-bit boundary, leading-zero neutrality, the exact semantic diagnostic
budget with additional post-suppression attempts, mixed-category diagnostic
source ordering, case-sensitive names and types, and every later same-kind
duplicate. Every generated command runs twice. The rejected boundary and
diagnostic-budget sources run through both commands; the accepted 16,384-bit
boundary uses silent checking only so the test does not intentionally capture
an enormous decimal evaluation result.

CLI integration coverage places lexical, parser, and semantic failures before,
after, and between otherwise valid typed declarations. `eval` remains
repeatably unsuccessful with zero value bytes in every such ordering, and an
earlier phase diagnostic prevents later-phase cascades. Multi-file `check`
coverage interleaves semantic, valid, lexical, and parser inputs and requires
repeatable diagnostics in argument order rather than phase or code order.

Semantic unit coverage also aggregates independent duplicate, unsupported-type,
word-width, negative-word, and word-range failures in one source. It requires
the raw and rendered diagnostic sequences to remain in source order, checks the
exact responsible source slice for every primary span, and checks that a
duplicate's secondary span names the first declaration. A duplicate typed
declaration is still type-checked in semantic traversal order; focused limit
tests pin the exact event at which its second failure becomes diagnostic
suppression or resource exhaustion.

A deterministic unit mutation corpus deletes, replaces, and inserts characters
at every boundary of an accepted mixed S3a module, then adds bounded sequences
of grammar fragments, comments, line endings, malformed characters, and
Unicode. The resulting set contains more than 2,500 unique sources. Every
mutant runs twice through the same gated lexical, parser, semantic, and
evaluation pipeline; phase results must be structurally equal, rendered
diagnostic bytes must match, success must remain atomic, and every primary and
secondary diagnostic span must belong to the mutated source. The corpus must
reach lexical, parser, and semantic rejection as well as successful evaluation.

Two frontend byte corpora exercise boundaries outside valid source grammar. On
Unix, 512 generated raw argument strings are classified twice in command,
option, operand, and post-`--` positions; every error is ASCII and contains no
control byte. A platform-independent corpus sends 518 fixed and generated raw
source byte strings—including every possible one-byte input—through `check`,
`eval`, and `lex` twice each. Status and output bytes must repeat exactly,
invalid UTF-8 must reach `ORC1002`, diagnostics may contain only ASCII plus line
feeds, and token output may additionally use its canonical tab separators.

The ten-file external corpus alone is not the full S3a evidence set. The
conformance runner parses the stable 30-rule index in
`docs/SEMANTICS_2026.md`, rejects missing or unknown rule IDs, and binds every
rule to named external or internal tests. It also binds the exact evidence-layer
declaration and requires the corresponding CLI, generated-CLI, parser-unit, or
unit observation for every rule. Specialized labels additionally require a
named test classified as an injected writer or injected limit; host-failure
coverage separately requires I/O, allocation, and non-regular host-boundary
failures. Each named test must have exactly one unconditional declaration at the
expected harness location: integration tests at file root and unit tests
directly inside the source's unique `#[cfg(test)] mod tests` container.
Declarations inside comments, strings, nested functions, or alternate or
disabled modules do not qualify. This is an exact evidence map, not a claim that
a named test exhausts its rule. Policy validation binds the production
constants to the specification, while injected-limit unit tests exercise exact
semantic-event, Core-node, and evaluation accounting and fail-closed behavior
at reachable boundaries. This indexed mapping does not complete S3 and adds no
source construct, semantic rule, canonical Core identity, proof, target, claim,
or S3b authority.

## S3b expression conformance

`fixtures/s3b/` contains an exact fourteen-file corpus for the proposed S3b
behavior: five fixtures must evaluate successfully and nine must fail closed.
The accepted fixtures cover `Int` arithmetic, word ring arithmetic at every
width, calls and grouping, the SHA-256 functions of FIPS 180-4 through round 0
of the "abc" example, and the ChaCha20 quarter round against the test vector of
RFC 8439 section 2.1.1. The rejected fixtures cover parameter syntax, ungrouped
operators, unknown names and calls, argument counts, types and undefined
operators, shift amounts, word literals and widths, call cycles, and the
diagnostic order across all of them.

`crates/orangec/tests/s3b_conformance.rs` checks the directory inventory, runs
`orangec check` and `orangec eval` twice per fixture, and requires identical
bytes each time. Accepted cases must check silently and print exact values.
Rejected cases must fail with no partial output, the exact ordered diagnostic
codes, their primary lines and columns, and their meaning.

The runner parses the 28-rule S3b index in `docs/EXPRESSIONS_2026.md`, rejects
missing, unknown, and duplicate rule IDs, and binds every rule to named CLI,
generated-CLI, parser-unit, or unit tests declared exactly once at their
harness locations. Generated cases pin each resource limit at its exact
boundary: 64 nesting levels for every opener, expression height 256, 64
parameters and 256 arguments, 256 call frames, the shared 1,048,576-step
evaluation budget, and the 16,384-bit `Int` result limit. The shift and
rotation tokens are checked for longest-match lexing. This corpus establishes
the tested behavior of one implementation; it does not accept OEP-0005, prove
the rules sound, or complete S3.

## S3c binding and conversion conformance

`fixtures/s3c/` contains an exact ten-file corpus for the proposed S3c
behavior: five fixtures must evaluate successfully and five must fail closed.
The accepted fixtures cover widening, narrowing, and the residue rule, byte
and word order in the little-endian convention of RFC 8439 and the big-endian
convention of FIPS 180-4, `let` and `as` used as ordinary names, the ChaCha20
quarter round written with named steps, and SHA-256 message words and round 0
of the "abc" example. The rejected fixtures cover binding syntax, duplicate
and late names, binding types, conversion targets and operands, and ungrouped
conversions.

`crates/orangec/tests/s3c_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3b runner. It parses the 17-rule S3c index in
`docs/BINDINGS_2026.md`, binds every rule to named CLI, generated-CLI,
parser-unit, or unit tests declared exactly once at their harness locations,
and pins the 256-binding limit at its exact boundary with a generated source.
This corpus establishes the tested behavior of one implementation; it does not
accept OEP-0006, prove the rules sound, or complete S3.

## S3d array conformance

`fixtures/s3d/` contains an exact eight-file corpus for the proposed S3d
behavior: three fixtures must evaluate successfully and five must fail closed.
The accepted fixtures cover array types, literals, and indices for `Int` and
every word width, the whole ChaCha20 block function checked against the
serialized block of RFC 8439 section 2.3.2, and the SHA-256 message schedule
and first two rounds of the FIPS 180-4 "abc" example with the working
variables as one `Word[32]^8`. The rejected fixtures cover array syntax,
lengths, literal counts and kinds, indices, and operators and conversions on
whole arrays.

`crates/orangec/tests/s3d_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3c runner. It parses the 17-rule S3d index in
`docs/ARRAYS_2026.md`, binds every rule to named CLI, generated-CLI,
parser-unit, or unit tests declared exactly once at their harness locations,
and pins the 65,536-element literal limit and the 65,536 length limit at their
exact boundaries with generated sources. This corpus establishes the tested behavior
of one implementation; it does not accept OEP-0007, prove the rules sound, or
complete S3.

## S3e loop conformance

`fixtures/s3e/` contains an exact seven-file corpus for the proposed S3e
behavior: three fixtures must evaluate successfully and four must fail closed.
The accepted fixtures cover loops over `Int`, words, and arrays, nested loops,
indices computed from loop indices, updates, fill literals, the longest
admitted loop, `for`, `in`, and `with` used as ordinary names, the whole
SHA-256 hash of both FIPS 180-4 examples ("abc" and the two-block message),
and the ChaCha20 encryption of the "sunscreen" plaintext of RFC 8439 section
2.4.2. The rejected fixtures cover loop syntax, bounds, names, scopes and
types, indices that are not static or not in range, and updates and fills of
the wrong kind, length, or element.

`crates/orangec/tests/s3e_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3d runner. It parses the 18-rule S3e index in
`docs/LOOPS_2026.md`, binds every rule to named CLI, generated-CLI,
parser-unit, or unit tests declared exactly once at their harness locations,
and pins the 65536 loop bound at its exact boundary with generated sources,
together with two nested maximal loops that analysis admits and the evaluation
step budget stops. This corpus establishes the tested behavior of one
implementation; it does not accept OEP-0008, prove the rules sound, or
complete S3.

## S3f condition conformance

`fixtures/s3f/` contains an exact eight-file corpus for the proposed S3f
behavior: four fixtures must evaluate successfully and four must fail closed.
The accepted fixtures cover `Bool` values and operators, comparisons of
integers and of words as unsigned numbers, Euclidean and unsigned division
with their rules for zero, conditionals and `else if` chains, indices that
divide loop indices, the contextual words used as ordinary names, X25519 on
the first test vector of RFC 7748 section 5.2, Poly1305 on the example of
RFC 8439 section 2.5.2, and ChaCha20-Poly1305 on the "sunscreen" example of
section 2.8.2. The rejected fixtures cover conditional syntax and mixed
operator groups; conditions, branches, operators, and conversions of the wrong
type; comparisons without a type and orders of arrays; and indices whose division
leaves their array.

`crates/orangec/tests/s3f_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3e runner. It parses the 18-rule S3f index in
`docs/CONDITIONS_2026.md`, binds every rule to named CLI, generated-CLI,
parser-unit, or unit tests declared exactly once at their harness locations,
and generates a 4096-arm `else if` chain, an untaken branch whose evaluation
would exceed the step budget, and the same branch taken, which fails closed.
This corpus establishes the tested behavior of one implementation; it does not
accept OEP-0009, prove the rules sound, or complete S3.

## S3g lookup conformance

`fixtures/s3g/` contains an exact four-file corpus for the proposed S3g
behavior: two fixtures must evaluate successfully and two must fail closed.
The accepted fixtures cover lookups keyed by bytes and by nibbles, updates
keyed by data, ranges narrowed by `&`, `>>`, `%`, conditionals, and
conversions, `Int` indices built from converted words, a table-driven CRC-32
against its check value, and AES-128 with its S-box derived as FIPS 197
section 5.1.1 defines it, against the examples of Appendices B and C.1 and
the inverse cipher. The rejected fixtures cover word indices whose type or
range is too wide, operators that could wrap, and `Int` indices built from
parameters, calls, or elements, or whose converted words do not fit.

`crates/orangec/tests/s3g_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3f runner. It parses the 10-rule S3g index in
`docs/LOOKUPS_2026.md`, binds every rule to named CLI, generated-CLI, or unit
tests declared exactly once at their harness locations, and generates a
source whose updates spend the step budget exactly, the same source one step
over, which fails closed, and the inversion of a 256-byte permutation by
updates keyed by its own values. This corpus establishes the tested behavior
of one implementation; it does not accept OEP-0010, prove the rules sound, or
complete S3.

## S3h module conformance

`fixtures/s3h/` contains an exact ten-file corpus for the proposed S3h
behavior: four programs, of which one must evaluate successfully and three must
fail closed, and six modules they use. The accepted program uses SHA-256,
HMAC, and HKDF as three modules, `hkdf` using `hmac` and `hmac` using
`sha256`, and reproduces the SHA-256 example of FIPS 180-4, test cases 1 and 2
of RFC 4231, and test case 1 of RFC 5869. The rejected programs cover a cycle
of uses, a module that uses itself, a module used twice, a file that declares
another module's name, calls qualified by a module not used or by the
calling module, a function the used module does not declare, a used module's
function called without its module, the wrong number of arguments, a result
of the wrong type, and a module file that does not exist.

`crates/orangec/tests/s3h_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3g runner, with each program's modules beside it. It
parses the 10-rule S3h index in `docs/MODULES_2026.md`, binds every rule to
named CLI, generated-CLI, or unit tests declared exactly once at their harness
locations, and generates a diamond of uses read once each, a program read from
standard input, a chain of 64 modules that links and one more that fails
closed, a module with 65 `use` declarations, and one step budget spent across
modules. This corpus establishes the tested behavior of one implementation; it
does not accept OEP-0011, prove the rules sound, or complete S3.

## S3i modular conformance

`fixtures/s3i/` contains an exact seven-program corpus for the proposed S3i
behavior, of which three must evaluate successfully and four must fail closed.
The accepted programs write X25519 over `Mod[(1 << 255) - 19]` and Poly1305
over `Mod[(1 << 130) - 5]` with no reduction in sight, reproducing the first
test vector of RFC 7748 section 5.2 and the tag of RFC 8439 section 2.5.2, and
compute constants in the rings their standards define: ML-KEM's zeta^128 and
128^-1 modulo 3329, Ed25519's d and square root of -1, and P-256's generator
on its curve. The rejected programs cover moduli that are too small, negative,
too wide, not constant, missing, or too large to compute; `type` declarations
out of order, naming built-in types, repeated, used before they are declared,
or adding a third array dimension (the former rank-two rejection is extended
by S3s); residue literals out of range; order, remainder,
and bitwise operators on residues; two moduli in one operator or call; `as`
to `Bool` or an array type; and a residue used directly as an index.

`crates/orangec/tests/s3i_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3h runner. It parses the 13-rule S3i index in
`docs/MODULAR_2026.md`, binds every rule to named CLI, generated-CLI, or unit
tests declared exactly once at their harness locations, and generates 64
`type` declarations and a 65th, the moduli 2, 2^521 - 1, and 2^521, literals
at the edges of `Mod[3329]`, and residues as indices at the edges of their
tables. This corpus establishes the tested behavior of one implementation; it
does not accept OEP-0012, prove the rules sound, or complete S3.

## S3j block conformance

`fixtures/s3j/` contains an exact six-program corpus for the proposed S3j
behavior, of which three must evaluate successfully and three must fail
closed. The accepted programs write SHA-256 with each round's working
variables a through h and its words T1 and T2 bound inside the loop's step,
reproducing the digests of "abc" and of the two-block message of FIPS 180-4's
examples; write the Montgomery ladder of X25519 as one loop whose step binds
every value RFC 7748 names, reproducing the first test vector of section 5.2;
and exercise steps and branches with bindings, nested blocks, a chain whose
every arm binds, residues, and names bound again in separate blocks. The
rejected programs cover a block with no value, a binding without its `;` or
its type, a value before a binding; a binding that repeats a parameter, a loop
index, an earlier binding of its block, or a body binding in scope; a name
used before its binding or outside its block; a binding's value of another
type, a branch whose value is a binding of another type, a binding of an
unknown type, and a conversion of a conditional whose
branches both end in their own bindings.

`crates/orangec/tests/s3j_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3i runner. It parses the 8-rule S3j index in
`docs/BLOCKS_2026.md`, binds every rule to named CLI, generated-CLI, or unit
tests declared exactly once at their harness locations, and generates a step
and a branch of 256 bindings and of 257. This corpus establishes the tested
behavior of one implementation; it does not accept OEP-0013, prove the rules
sound, or complete S3.

## S3k tuple conformance

`fixtures/s3k/` contains an exact seven-program corpus for the proposed S3k
behavior, of which four must evaluate successfully and three must fail
closed. The accepted programs write SHA-256 with the working variables a
through h as the loop's eight named accumulators, reproducing the digests of
"abc" and of the two-block message of FIPS 180-4's examples; write the ChaCha20
quarter round as a function of four words that gives four, and the block
function with the sixteen words of its state named through each double round,
reproducing RFC 8439's vectors of sections 2.1.1 and 2.3.2; write
Ascon-Hash256 over a state of five named 64-bit words, reproducing entries 1,
2, and 9 of the designers' known-answer file; and exercise a 256-bit addition
with a sum and a carry, a pair of accumulators, the extended Euclidean
algorithm, and tuples holding `Bool` values and chosen by conditionals. The
rejected programs cover a tuple type of one element, a tuple type holding a
tuple, an array of tuples, a tuple of one element, positions not written in
decimal, a second `.k`, `.k` after an index, and patterns missing a type or a
second name; names that repeat within a pattern, a parameter, a binding, or a
loop index, a name read before its pattern is bound or after its loop; and
tuples of another length or type, a tuple where a scalar is required, `.k` on
an array or past the last element, an element of another type, an index,
an order, arithmetic, or conversion of a whole tuple, a pattern of another
type or of an unknown type, and orders of a tuple or an array written out.

`crates/orangec/tests/s3k_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3j runner. It parses the 8-rule S3k index in
`docs/TUPLES_2026.md`, binds every rule to named CLI, generated-CLI, or unit
tests declared exactly once at their harness locations, and generates tuple
types, tuples, and patterns of 16 parts and of 17. This corpus establishes the
tested behavior of one implementation; it does not accept OEP-0014, prove the
rules sound, or complete S3.

## S3l bytes conformance

`fixtures/s3l/` contains an exact six-program corpus for the proposed S3l
behavior, of which three must evaluate successfully and three must fail
closed. The accepted programs write HMAC-SHA-256 with RFC 4231's keys and
messages as the RFC prints them, SHA-256's padding joined with `++`, and each
block's words read through four-byte slices, reproducing FIPS 180-4's digest
of "abc" and RFC 4231's test cases 1 and 2; write the ChaCha20-Poly1305 AEAD of
RFC 8439 with its plaintext as text and its key, nonce, and additional data in
hex, the key stream and Poly1305's input joined with `++`, and the one-time key
and each sixteen-byte block taken by slices, reproducing section 2.8.2's
ciphertext and tag and verifying the tag; and exercise every escape, spaced
and mixed-case hex, joins of words and of `Bool` values, open slices, slices
and slice updates inside loops, and a rotation by slices. The rejected programs
cover hex strings with a letter, a prefix, punctuation, an odd digit, a split
byte, an escape, and no closing quote; `hex` spaced from its quote, a slice
with no bounds or with a step, an index or slice of a slice or of a byte
string, `++` mixed with `+` in either order, and a slice update with no bounds;
and byte strings of another length or element type, with a character outside
printable ASCII, or empty, joins of another length, into a scalar, or of a word
or of words of another width, slices past the end, empty, backward, keyed by
data, of a length that changes from step to step, or of a loop index times
itself, slices of another length, element type, or base type, a slice update of
a value of another length or of a word, and a join of 300 elements.

`crates/orangec/tests/s3l_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3k runner. It parses the 10-rule S3l index in
`docs/BYTES_2026.md`, binds every rule to named CLI, generated-CLI, or unit
tests declared exactly once at their harness locations, and generates byte
strings and hex strings of 65,536 bytes and of 65,537 and byte strings holding a raw
tab and a raw delete, which the repository keeps out of its sources. This corpus establishes the
tested behavior of one implementation; it does not accept OEP-0015, prove the
rules sound, or complete S3.

## S3m sizes conformance

`fixtures/s3m/` contains an exact six-program corpus for the proposed S3m
behavior, of which four must evaluate successfully and two must fail closed.
The accepted programs write SHA-256 once for every message of 1 through 119
bytes, its padding with a length computed from the message's size and its
blocks absorbed by a loop whose bound is a size, reproducing FIPS 180-4's
digests of "abc" and of its 56-byte two-block message; write HMAC-SHA-256 once
for every key of 1 through 63 bytes and message of 1 through 55 over that
SHA-256, used as a module, reproducing RFC 4231's test cases 1 and 2; write
Poly1305 once for every message of 1 through 255 bytes, reproducing RFC 8439
section 2.5.2's tag; and exercise sums over arrays of every length, instances
of one and two sizes displayed by their sizes, lengths computed from sizes,
tuples of sized halves, slices bounded by a size, Euclidean division of
sizes, and calls with and without sizes. The rejected programs cover a size
parameter without `in`, with a named bound, or a fifth; a computed length,
fill length, or loop bound without parentheses; a qualified sized call without
arguments; an empty range, a bound over 65536, 361 instances, a repeated size
name, a parameter named like a size, a length written with a parameter, an
index out of range in the first instance, a length of 0 in the first
instance, a size outside its range, two sizes for one, sizes for a function
without them, a call whose argument's length fits no instance and one that
fits two, and a cycle between two instances.

`crates/orangec/tests/s3m_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3l runner. It parses the 10-rule S3m index in
`docs/SIZES_2026.md`, binds every rule to named CLI, generated-CLI, or unit
tests declared exactly once at their harness locations, and generates a
function of four sizes with 256 instances, a function with every array length
from 1 through 256, and a size bound of 65536, and each with one instance or
one bound more. This corpus establishes the tested behavior of one
implementation; it does not accept OEP-0016, prove the rules sound, or
complete S3.

## S3n byte order conformance

`fixtures/s3n/` contains an exact eight-program corpus for the proposed S3n
behavior, of which six must evaluate successfully and two must fail closed.
The accepted programs write SHA-256 and SHA-512 with each block read as
big-endian words and each digest written as big-endian bytes, reproducing FIPS
180-4's digests of "abc" and of its two-block messages and an independent
implementation's at each padding limit and at the longest message each
admits; ChaCha20 with its state read from "expand 32-byte k", the key, the
counter, and the nonce as little-endian words, reproducing RFC 8439's block of
section 2.3.2 and ciphertext of section 2.4.2; Poly1305 with its key's halves
and each block read as little-endian numbers and residues, reproducing section
2.5.2's tag and two vectors of Appendix A.3; X25519 with its scalar and
coordinate read and its result written as little-endian residues, reproducing
RFC 7748 section 5.2's vector; and exercise both orders on words of every
width, text, round trips, joined and quartered words, numbers, negative and
oversized numbers, residues, and a size's value. The rejected programs cover
words of different widths in both directions, a byte order between two
numbers, from `Bool`, to `Bool`, to an array of residues, and from an array of
`Int`, an array literal without a typed element, a target other than the
expected type, and conversions of and to arrays of words without a byte
order; and an array type after `as` without a byte order and `big` as a type's
name before another `as`.

`crates/orangec/tests/s3n_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3m runner. It parses the 8-rule S3n index in
`docs/ORDER_2026.md`, binds every rule to named CLI, generated-CLI, or unit
tests declared exactly once at their harness locations, and generates a
program that converts words of every width in both orders to words of every
width and to `Int`, checked against a reference computed in the runner, and
converts `Word[64]^256` to `Int` and back, and one whose targets have one
element more than an array may. This corpus establishes the tested behavior of
one implementation; it does not accept OEP-0017, prove the rules sound, or
complete S3.

## S3o type parameter conformance

`fixtures/s3o/` contains an exact five-program corpus for the proposed S3o
behavior, of which three must evaluate successfully and two must fail closed.
The accepted programs write exponentiation, Fermat inversion, and Euler's
criterion once for five prime fields, the field and subgroup order of
Curve25519, the field of Poly1305, and the moduli of ML-KEM and ML-DSA,
reproducing each modulus, RFC 8032's square root of −1, and the primitive
roots of unity of FIPS 203 and FIPS 204; write Ch, Maj, the round, and the
final addition of SHA-256 and SHA-512 once for words of both widths,
reproducing FIPS 180-4's digests of "abc" and of its two-block messages; and
exercise one body for `Int`, a word, and a residue, conversions to a type
parameter, arrays, tuples, and loops of it, `Bool`, array, and tuple types in
a list, sizes beside types, typed roots evaluated once for each instance, and
calls named by their types or fitted by their arguments and their place. The
rejected programs cover a type listed twice, a type parameter named like a
built-in type, a declared type, or another parameter, a size inside a listed
type, unknown and malformed listed types, too many instances, a type
parameter's name as a value, an instance in error named in a note, an
unlisted type entry, a value as a type entry, too many entries, a call whose
type nothing decides, a call that fits no instance, and a call fitted to the
wrong result; and an empty list, a trailing comma, a missing comma, an
unclosed list, and a fifth parameter in brackets.

`crates/orangec/tests/s3o_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3n runner. It parses the 10-rule S3o index in
`docs/TYPE_PARAMETERS_2026.md`, binds every rule to named CLI, generated-CLI,
or unit tests declared exactly once at their harness locations, and generates
a program whose one function lists 64 residue types beside a size of four
values, evaluating all 256 instances, and one whose list adds `Int` for 260
instances and an error. This corpus establishes the tested behavior of one
implementation; it does not accept OEP-0018, prove the rules sound, or
complete S3.

## S3p lengths conformance

`fixtures/s3p/` contains an exact three-program corpus for the proposed S3p
behavior, of which two must evaluate successfully and one must fail closed.
The accepted programs write ChaCha20 and Poly1305 once for messages of 1
through 256 whole blocks and reproduce RFC 8439's long vectors as the RFC
prints them: the 375-byte text and ciphertext of appendix A.2 test vector 2,
the tags of appendix A.3 test vectors 2 and 3 over the same text, and the
265-byte ciphertext of appendix A.5, which authenticates under its tag and
opens to the RFC's plaintext; and build a table of the 65,536 powers of 3
modulo the Fermat prime 2^16 + 1 in rows placed with slice updates, read it
by 16-bit words for Pepin's test, and exercise fills, joins, slices, and byte
orders at 65,536 elements and a conversion to `Int` at the 16,384-bit limit,
under `--steps 2097152 --stats`. The rejected program covers a length, a
fill, a join, and a slice one element past the limit, a 16-bit index into an
array one element short, a 32-bit index into the longest array, and a
conversion target past the limit.

`crates/orangec/tests/s3p_conformance.rs` runs the same repeatable `check` and
`eval` protocol as the S3o runner, with each fixture's `eval` options. It
parses the 11-rule S3p index in `docs/LENGTHS_2026.md`, binds every rule to
named CLI, generated-CLI, or unit tests declared exactly once at their harness
locations, and checks RFC 8439's values against the vectors pinned in the
D-011 suite. It generates a literal of all 65,536 16-bit words and a byte
string of 65,536 bytes, read with `--spec`, and one element or byte more;
evaluations at the default budget, at exactly the steps needed, at the most
admitted, and one step short, and every malformed `--steps`; selections of
functions in source order, of every instance of a sized one, and of names
that match nothing or are malformed, and a 65th name; and both output streams
in one file, where the report follows the values, and each option with a
command other than `eval`. This corpus establishes the tested behavior of one
implementation; it does not accept OEP-0019, prove the rules sound, or
complete S3.

## S3q known-answer test conformance

`fixtures/s3q/` contains an exact five-program corpus for the proposed S3q
behavior: two programs must check and run their tests or evaluate
successfully, one must check and report failed tests with status 1, and two
must fail closed. The accepted programs write seven of RFC 8439's examples
and test vectors as tests, with inputs and expected bytes as the RFC prints
them: the quarter round of section 2.1.1, the block function of section
2.3.2, the zero key's key stream of appendix A.1 test vectors 1 and 2, a
nonce that changes every block, Poly1305 of section 2.5.2, and appendix A.3
test vector 1; and compare words, truth values, residues, tuples holding
arrays, and arrays of 256 and 65,536 elements whole, with exact steps under
`--stats` that do not depend on where the operands differ. The failing
program's report shows a word, an array, a tuple holding an array, and a
residue that differ, with their values and first differences, and a claim of
two comparisons that has only its line. The rejected programs cover an empty
title, a letter outside ASCII, a backslash, a repeated title, a claim that is not a `Bool`, an
order on a tuple and on two tuples written out, two arrays written out, an
unknown function, and a test without a title.

`crates/orangec/tests/s3q_conformance.rs` runs each fixture's commands,
`check`, `test`, and `eval` with their options, twice each, and requires
identical status, standard output, and standard error. It parses the 12-rule
S3q index in `docs/TESTS_2026.md` and binds every rule to named CLI,
generated-CLI, parser-unit, or unit tests declared exactly once at their
harness locations. It generates titles of 1 and 128 bytes, and titles
holding a raw tab or delete, which the repository keeps out of its sources,
or 129 bytes; a program whose used module has a test that
is not a `Bool` and one that fails, neither checked nor run from the root and
both when that module is the root; a test that stops at a budget one step
short of the run and within the first test; both output streams in one file,
where the step report follows the report; every usage error of `orangec
test`; and comparisons of 65,536 bytes that differ at any position in
identical steps. This corpus establishes the tested behavior of one
implementation; it does not accept OEP-0020, prove the rules sound, or
complete S3.

## S3r computed amount conformance

`fixtures/s3r/` contains an exact six-program corpus for the proposed S3r
behavior: four programs must check, run their tests, and evaluate
successfully, and two must fail closed. The accepted programs compare every
operator on a byte against its definition at amounts below, at, and past the
width, of both signs, and of any size; turn a 64-bit word by a byte and shift
by a size; reverse a byte's bits, count the set bits of a 32-byte key, and
pick a nibble at a computed position from a table; write RC6-32/20/16 with
its key schedule and rounds rotating by the amounts its key and data choose,
and reproduce the paper's two 128-bit-key vectors both ways; write SHA3-256
of FIPS 202 for messages of 1 through 133 bytes with rho turning each lane by
(t + 1)(t + 2)/2 and iota placing bits at 2^j - 1, and reproduce NIST's
examples for "abc" and the 448-bit message; and derive ML-KEM's transform
constants by BitRev7 and powers of 17 modulo 3329, as FIPS 203 section 4.3
defines them, reproducing Appendix A. Exact steps under `--stats` are pinned.
The rejected programs cover a literal amount at the width and one with a
sign, amounts that are a truth value, a residue, an array, or a comparison, a
shift of an `Int`, an unknown name, an index whose computed shift ranges past
its table, and an amount built with an operator and not grouped.

`crates/orangec/tests/s3r_conformance.rs` runs each fixture's commands,
`check`, `test`, and `eval` with their options, twice each, and requires
identical status, standard output, and standard error. It parses the 10-rule
S3r index in `docs/AMOUNTS_2026.md` and binds every rule to named CLI,
generated-CLI, or unit tests declared exactly once at their harness
locations. It generates programs that compare every operator at every width
with a reference for `Int` amounts from -(2n + 1) through 2n + 1 and around
2^63, 2^64, 2^126, and 2^127, and for word amounts of every width in both
directions; programs whose amounts of 2 through 16,384 bits cost the same
steps; and programs that write a literal amount at every width or with a
sign, which are refused, and the same amounts grouped, which are computed.
This corpus establishes the tested behavior of one implementation; it does
not accept OEP-0021, prove the rules sound, or complete S3.

## S3s nested-array conformance

`fixtures/s3s/` and `crates/orangec/tests/s3s_conformance.rs` cover exact
rectangular shapes, nested literals and fills, both index axes, row updates,
row slices and joins, matrices in tuples, finite specialization, recursive
equality, and rejected implicit flattening. Generated cases reach the scalar
product limit, exceed it, vary row widths and modular domains, and compare
equal costs at different mismatch positions. The 12-rule index in
`docs/NESTED_ARRAYS_2026.md` binds these checks to proposed S3s behavior.
They establish implementation behavior and do not accept OEP-0023 or prove
ML-KEM, transformation, leakage, or refinement properties.

## Layout

- `crates/orange-compiler`: reusable source, span, diagnostic, edition, lexer,
  syntax-tree, parser, semantic, Core, and evaluator library;
- `crates/orange-compiler/tests/d004_decision_suite.rs`: historical input-only
  and v0.6 reviewed-not-executable D-004 protocol, subject-catalog, exact
  75-record replay-plan, and future result-contract checks;
- `crates/orange-compiler/tests/d005_decision_suite.rs`: draft-only D-005
  packet, transport-identity, and synthetic capture-integrity checks;
- `crates/orange-compiler/tests/d006_decision_suite.rs`: input-only D-006
  pre-epoch packet, case-index, and identity-inventory checks;
- `crates/orangec`: thin file/stdin CLI with deterministic `check`, `eval`, and
  `lex` behavior, and the sealing commands `keygen`, `enc`, `dec`, and
  `schemes`;
- `crates/orangec/src/crypt.rs`: the sealing commands and sealed-file format 1;
- `crates/orangec/tests/crypt.rs`: black-box sealing tests, including files
  sealed by an independent implementation of the format;
- `crates/orangec/tests/s2_conformance.rs`: protected indexed S2 lexical and
  parser conformance runner;
- `crates/orangec/tests/s3a_conformance.rs`: exact repeatable black-box S3a
  corpus runner;
- `crates/orangec/tests/s3b_conformance.rs`: exact repeatable S3b corpus,
  rule-index, and resource-limit runner;
- `crates/orangec/tests/s3c_conformance.rs`: exact repeatable S3c corpus,
  rule-index, and binding-limit runner;
- `crates/orangec/tests/s3d_conformance.rs`: exact repeatable S3d corpus,
  rule-index, and element- and length-limit runner;
- `crates/orangec/tests/s3e_conformance.rs`: exact repeatable S3e corpus,
  rule-index, and loop-bound runner;
- `crates/orangec/tests/s3f_conformance.rs`: exact repeatable S3f corpus,
  rule-index, and conditional-chain runner;
- `crates/orangec/tests/s3g_conformance.rs`: exact repeatable S3g corpus,
  rule-index, and update-cost runner;
- `crates/orangec/tests/s3h_conformance.rs`: exact repeatable S3h corpus,
  rule-index, and module-reading runner;
- `crates/orangec/tests/s3i_conformance.rs`: exact repeatable S3i corpus,
  rule-index, and modulus-limit runner;
- `crates/orangec/tests/s3j_conformance.rs`: exact repeatable S3j corpus,
  rule-index, and bindings-per-block runner;
- `crates/orangec/tests/s3k_conformance.rs`: exact repeatable S3k corpus,
  rule-index, and tuple-size runner;
- `crates/orangec/tests/s3l_conformance.rs`: exact repeatable S3l corpus,
  rule-index, and byte-string-length runner;
- `crates/orangec/tests/s3m_conformance.rs`: exact repeatable S3m corpus,
  rule-index, and instance-limit runner;
- `crates/orangec/tests/s3n_conformance.rs`: exact repeatable S3n corpus,
  rule-index, and width runner;
- `crates/orangec/tests/s3o_conformance.rs`: exact repeatable S3o corpus,
  rule-index, and instance-limit runner;
- `crates/orangec/tests/s3p_conformance.rs`: exact repeatable S3p corpus,
  rule-index, length-limit, and evaluation-option runner;
- `crates/orangec/tests/s3q_conformance.rs`: exact repeatable S3q corpus,
  rule-index, test-run, and report runner;
- `crates/orangec/tests/s3r_conformance.rs`: exact repeatable S3r corpus,
  rule-index, reference-amount, and amount-cost runner;
- `crates/orangec/tests/s3s_conformance.rs`: repeatable nested-array corpus,
  rule-index, shape-limit, and recursive-cost runner;
- `fixtures/hello.or`: permanent legacy syntax fixture;
- `fixtures/typed-answer.or`: permanent typed-literal evaluation fixture;
- `fixtures/s3a/`: exact three-positive/seven-negative S3a CLI fixture corpus;
- `fixtures/s3b/`: exact five-positive/nine-negative S3b CLI fixture corpus;
- `fixtures/s3c/`: exact five-positive/five-negative S3c CLI fixture corpus;
- `fixtures/s3d/`: exact three-positive/five-negative S3d CLI fixture corpus;
- `fixtures/s3e/`: exact three-positive/four-negative S3e CLI fixture corpus;
- `fixtures/s3f/`: exact four-positive/four-negative S3f CLI fixture corpus;
- `fixtures/s3g/`: exact two-positive/two-negative S3g CLI fixture corpus;
- `fixtures/s3h/`: exact one-positive/three-negative S3h CLI program corpus
  and the six modules its programs use;
- `fixtures/s3i/`: exact three-positive/four-negative S3i CLI fixture corpus;
- `fixtures/s3j/`: exact three-positive/three-negative S3j CLI fixture corpus;
- `fixtures/s3k/`: exact four-positive/three-negative S3k CLI fixture corpus;
- `fixtures/s3l/`: exact three-positive/three-negative S3l CLI fixture corpus;
- `fixtures/s3m/`: exact four-positive/two-negative S3m CLI fixture corpus;
- `fixtures/s3n/`: exact six-positive/two-negative S3n CLI fixture corpus;
- `fixtures/s3o/`: exact three-positive/two-negative S3o CLI fixture corpus;
- `fixtures/s3p/`: exact two-positive/one-negative S3p CLI fixture corpus;
- `fixtures/s3q/`: exact two-positive/one-failing/two-negative S3q CLI fixture
  corpus;
- `fixtures/s3r/`: exact four-positive/two-negative S3r CLI fixture corpus;
- `fixtures/s3s/`: nested-array positive, negative, and failed-equality corpus;
  and
- `schemes/`: the built-in sealing schemes, each an Orange program ending in
  its known answers, and the specification of the scheme interface and
  sealed-file format 1.
