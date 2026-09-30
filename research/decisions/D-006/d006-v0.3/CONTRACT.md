# D-006 v0.3 candidate contract (for the people and agents building the candidates)

This is the working contract for building the two D-006 candidates, C-01 Rocq and
C-02 Lean 4, against the frozen shared packet. Read it whole before writing code.
The candidates were built in a laboratory work directory and now live beside
this file; paths below are relative to the repository root.

## Where things are

- Shared packet (read-only, frozen, never edit):
  `research/decisions/D-006/d006-v0.3/shared-inputs/`
  - `semantics.md` fixes what every symbol means. Read the section for your case.
  - `dsNN-*.json` carry the signature, theorem statements, observations and
    negative cases as neutral terms.
  - `tools/d006_shared.py` is a plain-Python reference for
    every computation (useful to understand exact behaviour; never import it
    into a proof).
- Candidates:
  - Rocq: `research/decisions/D-006/d006-v0.3/rocq/` (sources in `theories/`, logical root `D006`)
  - Lean 4: `research/decisions/D-006/d006-v0.3/lean4/`
- Harness: `tools/d006_render.py` (renders the shared terms) and
  `tools/d006_check.py` (builds, checks, classifies).
  Do not edit the harness. If it is wrong or blocks you, stop and report exactly
  what and why; the orchestrator fixes it for both candidates.
- Toolchains (read-only):
  - Rocq 9.2.0 with Stdlib 9.2.0: `/opt/d006/rocq-9.2.0` (`bin/coqc`, `bin/coqchk`)
  - Lean 4.34.1: `/opt/d006/lean-4.34.1-linux` (`bin/lean`, `bin/lake`, `bin/leanchecker`)
  - CaDiCaL 3.0.1: `/opt/d006/cadical-src/build/cadical` (DS-04 only)
  No network package registries are reachable. Use only each toolchain's own
  standard library (Rocq Stdlib; Lean core `Init`/`Std`/`Lean`). No Mathlib,
  no Batteries, no opam or Reservoir packages.

## The development loop

```sh
python3 tools/d006_check.py dev research/decisions/D-006/d006-v0.3/rocq DS-01            # or lean4, and any cases
python3 tools/d006_check.py dev research/decisions/D-006/d006-v0.3/rocq DS-01 --positives-only
python3 tools/d006_check.py dev research/decisions/D-006/d006-v0.3/rocq DS-01 --only D1-N02,D1-N10 --negatives-only
python3 tools/d006_check.py dev research/decisions/D-006/d006-v0.3/rocq DS-04 --run-time    # also D4-R02 to D4-R05
```

It copies the candidate into a fresh work directory, runs the adapter's
`build.serial` steps, renders one check file per case (parity for every
theorem, M-01 instance checks, one proof per observation by the declared
method, a trust audit of each) and one file per negative case, runs them, and
prints one line per item. `0 failure(s)` is the goal. Set `D006_WORK` to use a
private work directory when several builders run at once (for example
`D006_WORK=/tmp/d006-dev-rocq-ds02`).

## The adapter (`adapter.json` plus `adapter.d/*.json` in the candidate root)

`adapter.json` holds the DS-01 base. Every later case adds its own fragment,
`adapter.d/ds02.json`, `adapter.d/ds03.json`, `adapter.d/ds04.json`,
`adapter.d/ds05.json`, so builders working on different cases never edit the
same file. Fragments are merged in file-name order: maps merge key by key,
lists append (a fragment's `classifier` rules go before the base rules), and
a scalar may only repeat the value already there. A fragment therefore adds
its own `symbols`, `theorems`, `patches`, `preamble.<case>`,
`methods.<kind>.<case>`, `build.serial`/`build.declared_parallel` steps,
`classifier` rules and `trust` entries. Only edit `adapter.json` itself if
you are the DS-01 builder. The Rocq `adapter.json` is a complete worked
example.

`schema_version` is `d006-v0.3-adapter-1`. Fields:

- `build.serial` / `build.declared_parallel`: argv lists run in order from the
  copied source root. `{toolchain}` is the toolchain root; `{jobs}` is 1 or 4.
  Keep every source file in the build; nothing is precompiled or checked in.
- `check.argv`: how one generated file is checked against the built library
  (`{file}`, `{src}`, `{audit}` placeholders). Already set; extend only if a new
  library path is needed.
- `preamble`: a map from case (`DS-01`, ...) to the text at the top of that
  case's generated files. Imports and nothing that changes a statement's
  meaning (no notations that reinterpret rendered syntax, no axioms).
- `literals.nat`, `literals.str`: templates for literals (`{}` is the digits or
  the escaped text). `nat_scope` (Rocq) scopes `<`, `<=`, `+` on naturals.
  `list_literal: true` renders list literals with list syntax; only allowed
  when T-05/C-08/C-09 map to the language's own list type.
- `symbols`: every shared T-, C-, F-, S-, R-, B- id your case uses maps to
  exactly ONE global declaration name (fully qualified is safest). The renderer
  writes `@name` applied to every argument the shared type lists, in order,
  type and width arguments included. So your declaration's explicit argument
  list must match the shared signature exactly (e.g. `word_add (w : N) (x y :
  Word w)`; constructor `ok` takes `A E` then the value). Shared types given as
  strings (DS-02 onward, e.g. `"S-C15 assign Nat Expr"`) mean constructor
  arguments in that order.
- `modules.M-01`: how to instantiate the parameterized quarter round with
  `{w} {r1} {r2} {r3} {r4}` (already set for Rocq).
- `theorems`: shared theorem id to the candidate theorem name. The harness
  renders the shared statement and checks your theorem against it (Rocq:
  `Definition parity : <statement> := <yours>.`; Lean: `theorem parity :
  <statement> := <yours>`). Your theorem must be exactly the statement, up to
  definitional unfolding; weaker, stronger or reshaped is a failure.
- `methods.observation` (a map from case, or `default`, to a string): the one
  tactic that proves every observation `lhs = rhs` of that case. It must be a
  computation (Rocq: `vm_compute; reflexivity`; Lean: `decide`, `decide
  +kernel`, `rfl`, or `native_decide` if you accept listing the native
  evaluator and the axioms it adds in the trust inventory). Never a tactic
  that looks the answer up or special-cases a fixture.
- `methods.exhaustive`: the tactic the DS-01 `exhaustive` negative uses; it must
  be a genuine exhaustive evaluation (it is expected to time out on 2^32 cases).
- `methods.instance`: proves the M-01 instance equations (Rocq: `reflexivity`).
- `patches`: negative/mutation id to a unified diff (`patch -p1` from the
  candidate root, paths `a/...` and `b/...`) that makes exactly the stated
  change. For `artifact_truncation` (D2-M05) the value is an object:
  `{"artifact": ARTIFACT, "check": ARGV}`, where ARTIFACT is the compiled
  file holding the theorem, relative to the source root, and ARGV is an
  independent check or load of it with an `{artifact}` placeholder.
- `classifier`: ordered rules `{category, phases?, pattern}`; the first rule
  whose phase list contains the failing item's phase (`statement`,
  `definition`, `proof`, `audit`, `build`, `artifact`) and whose regex matches
  the tool's message decides the category. Rules must be honest descriptions
  of the tool's messages; never write a rule that matches a case id.
- `trust.allowed_assumptions`: the names the audit may report (Rocq `Print
  Assumptions`, Lean `#print axioms`). Anything else reported is
  `undeclared_trust`. List only what your proofs genuinely rely on, with a
  reason in `trust.inventory`. Primitive types and operations (Rocq PrimString,
  Uint63) appear in Rocq audits and must be listed if used. `sorry`, `admit`,
  `Admitted`, `Axiom` in candidate sources are forbidden.
- `trust.allowed_assumption_patterns`: a list of `{pattern, reason}` for a
  tool that names a fresh assumption at every use (Lean 4.34's `native_decide`
  adds `<decl>._native.native_decide.ax_N_M`). The regex must match the whole
  reported name, must not match any other kind of assumption, and must not
  contain a case, fixture or observation id; list the tool it stands for in
  `trust.inventory`. An optional `cases` list limits the row to those cases,
  so a widening declared for one case cannot pass another case's audit.
- `trust.inventory`: a list of `{name, kind, reason}` for the kernel, the
  evaluators your methods use, every allowed assumption, and any extraction or
  runtime. Be complete.

## Symbols fixed across the candidate

Each symbol maps to one declaration for the whole candidate, so these choices
are shared by every case:

- Rocq: T-01 `bool`, T-02 `N` (binary naturals), T-05 `list`, C-08 `nil`,
  C-09 `cons`, T-08 `PrimString.string` (literal `"..."%pstring`), F-21
  `N.pow`, F-22 `app`, and the DS-01 declarations in `theories/Core.v`.
- Lean: T-01 `Bool`, T-02 `Nat`, T-05 `List`, C-08 `List.nil`, C-09
  `List.cons`, T-08 `String`, F-21 `Nat.pow`, F-22 `List.append`; T-03 is
  `BitVec` or a thin wrapper, T-04 `Vector` from core or a subtype.

## Rules

- Idiomatic, readable code in each language. Short doc comments where they help
  a reviewer. No `sorry`/`admit`/`Admitted`/axioms. No new external dependencies.
- Never edit the shared packet, the harness, or the other candidate.
- Never special-case a fixture, observation or negative id in candidate code.
- Put each case in its own module: Rocq `theories/Core.v` (DS-01),
  `theories/Sieve.v` (DS-02), `theories/Records.v` (DS-03),
  `theories/Lrat.v` (DS-04), `extraction/` (DS-05); Lean `D006/Core.lean`,
  `D006/Sieve.lean`, `D006/Records.lean`, `D006/Lrat.lean`, `Checker/`.
  Later cases import earlier ones (DS-02, DS-03 and DS-04 import Core).
- Add your module to `build.serial` and `build.declared_parallel` in order.
- Directory and file names must not contain the words epoch, candidate,
  result, replay, review or decision (singular or plural) as a separate
  path component or word segment; the repository validator refuses them.
- Keep a short implementation log in `NOTES.md` in the candidate root: what you
  built, design choices, dead ends, anything a reviewer should know, and wall
  time spent. Append; do not rewrite other entries.
- When done, the dev loop must print `0 failure(s)` for your case, and you
  report back: files changed, the dev loop's final output, trust inventory,
  and open issues.

## Case-specific notes

- DS-01 negatives D1-N07 (exhaustive) and D1-N09 are expected to hit the
  120-second negative ceiling or memory exhaustion; the dev loop waits for
  them. Use `--positives-only` while iterating.
- DS-02: `D2-M05` (artifact truncation) needs `patches.D2-M05` as an object
  (see above); every other mutation is a patch file or a rendered obligation.
  Sieve variables and arrays are indexed by naturals (`var 0` is the first
  declared variable).
- DS-03: the 44 observations include every error fixture; `R-C20 failure`
  carries a `String` path, so the Rocq candidate builds `PrimString` paths.
  `records_valid` must be defined from the stated conditions, not from the
  decoder (semantics.md section 4).
- DS-04: besides the five theorems the harness renders observations
  `D4-CNF01`/`D4-CNF02` (`cnf_text` of each obligation equals the canonical
  CNF text, as a string literal) and `D4-G-V-00` to `D4-G-V-11`
  (`lrat_verdict` on the golden certificate and each mutation equals the
  reference verdict; `accept` is `B-C03`, a rejection is `B-C04 "code" line`).
  The certificates are string literals up to 4 MiB (V-08), so the checker must
  be efficient under the declared computation method. D4-TH01 must be proved
  through the checker (the golden certificate accepted by `lrat_verdict`, then
  D4-TH04 and D4-TH02), not by a direct bit-vector proof.
- DS-05 builds a standalone checker from the in-prover definitions (Rocq
  extraction to OCaml; Lean compiled C) with the interface in
  `ds05-standalone-checker.json`, for x86-64 natively and for AArch64 (Rocq:
  OCaml 4.14.1 bytecode on `/opt/d006/ocaml-4.14.1-aarch64/bin/ocamlrun` or
  linked with its `lib/libcamlrun.a` by `aarch64-linux-gnu-gcc`; Lean: the
  emitted C compiled by `/opt/d006/lean-4.34.1-linux/bin/clang
  --target=aarch64-unknown-linux-gnu --sysroot /opt/d006/lean-4.34.1-linux_aarch64`
  against that tree's `include` and `lib/lean`). Run AArch64 binaries with
  `qemu-aarch64-static -L /usr/aarch64-linux-gnu`.

## How the epoch runs your commands (read before choosing flags)

The dev loop runs commands directly. The epoch runs every build, check and
negative step as an unprivileged lab user inside fresh namespaces (no network)
and a Landlock sandbox, with these per-process caps from `tools/fs_sandbox.c`
for both candidates: 4 GiB address space, 600 CPU seconds, 512 MiB per file,
1024 open files, 256 processes. Readable: `/usr`, `/proc`, `/dev/urandom`, the
toolchain tree and your copied sources; writable: the run directory only.
Nothing under `/etc` is readable unless the candidate declares it in
`host_files` (Rocq declares `/etc/ocamlfind.conf`).

- Lean 4 reserves a 1 GiB stack per thread by default, which the 4 GiB cap
  refuses. Pass `--tstack=65536` to every `lean` invocation (the DS-01 adapter
  does). Compiled Lean programs (lake, leanchecker, your DS-05 checker) read
  `LEAN_STACK_SIZE_KB`; the orchestrator sets it in `adapter.d/ds06.json`.
- Artifacts the build writes must stay inside the source copy; `.lake` and
  compiled files are never copied from the candidate root.
- The orchestrator owns `adapter.d/ds06.json` (re-check commands, deterministic
  artifacts, fault hooks, host files). Do not edit it; tell the orchestrator if
  your case needs an entry there (for example a new compiled module).

## DS-05: the standalone checker's adapter entry

Read `shared-inputs/ds05-standalone-checker.json` whole. The checker is built
from the in-prover definitions of DS-03 (`decode_records` and the verdict line
of semantics.md section 4) and DS-04 (`cnf_text` and `lrat_verdict`), never
rewritten by hand: Rocq extracts them to OCaml; Lean compiles them to C. The
hand-written part is only the command-line driver (argument parsing, file
reading, printing), which counts as trusted glue in M-11 and must stay small.
Interface (exit 0 unless stated):

- `CHECKER records PATH` prints one line: `accept` followed by one field per
  record (`def:NAME:DIGEST`, `thm:NAME:FINGERPRINT:REFS`,
  `claim:NAME:REF:LEVEL`; hex lowercase, refs comma-separated decimals), or
  `reject CODE PATH`. The expected lines are the `standalone` fields of the
  DS-03 observations.
- `CHECKER cnf OBLIGATION` (`B-C01` or `B-C02`) prints the canonical CNF text.
- `CHECKER lrat OBLIGATION CNF_PATH CERTIFICATE_PATH` prints `accept` or
  `reject CODE LINE`.
- `CHECKER` with no arguments prints usage on standard error and exits 2.

The fragment `adapter.d/ds05.json` adds:

```json
"standalone": {
  "sources": ["globs of the driver and generated sources, relative to the source root"],
  "hosts": {
    "H-01": {"build": [["argv", "..."]], "binary": "relative path of the built checker",
             "launch": ["{binary}"], "runtime": []},
    "H-02": {"build": [["argv", "..."]], "binary": "relative path",
             "launch": ["/usr/bin/qemu-aarch64-static", "-L", "/usr/aarch64-linux-gnu", "..."],
             "runtime": ["{toolchain_aarch64}/...", "..."]}
  },
  "dependency": "a path relative to the toolchain root that the build needs (removed for D5-N06)"
}
```

Test it with the dev loop, which builds each host's checker and runs the whole
corpus directly (no sandbox; the epoch adds it and runs the negatives):

```sh
python3 tools/d006_check.py dev research/decisions/D-006/d006-v0.3/rocq DS-05 [--host H-01] [--fresh] [--only D5-R-F01,...]
```

`--fresh` also runs the pinned solver for a fresh certificate and adds its
V-00 to V-11 variants, as the epoch does.

`build` steps run after the prover build, from the source root, in the same
sandbox (placeholders `{toolchain}`, `{toolchain_aarch64}`, `{src}`,
`{jobs}`). `launch` is the command prefix the runner appends the interface
arguments to; `{binary}` is the absolute path of the built checker. `runtime`
lists every file or directory outside `/usr` the checker reads at run time
(for example Rocq's AArch64 `ocamlrun`); at check time nothing else from a
toolchain tree is readable, so no prover, plugin or solver can be used. The
runner computes the shared-object closure, the stripped and unstripped sizes,
the source-to-binary digests, and runs the whole corpus and the negatives
D5-N01 to D5-N07 itself. Rocq's H-02 path is TC-06 bytecode run by TC-10's
`ocamlrun`; Lean's is the emitted C compiled with TC-01's clang for
`aarch64-unknown-linux-gnu` against TC-11 (see the case notes above). Add a
build-phase `classifier` rule for the message your toolchain prints when the
`dependency` is missing (expected categories `parse_failure` or
`unsupported_feature`).
