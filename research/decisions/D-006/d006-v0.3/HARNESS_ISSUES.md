# Harness issues

## H-01 (Rocq, DS-03): list literals with a bare-symbol element type do not parse

Reported by the Rocq DS-03 builder, 2026-09-29.

- Where: `tools/d006_render.py`, `Language.items`, which renders the element
  type with `kind_text = self.term(kind)`. For a `["list", ["sym", ID], ...]`
  term whose element type is a bare symbol (DS-03 `R-T01` and `T-02`), Rocq
  gets `(@cons @D006.Records.Record x rest)` and `(@nil @D006.Records.Record)`.
  Rocq does not accept `@name` as an argument without parentheses (the
  renderer's own `argument()` comment says so). DS-01 never hit this because
  its list element types are applications (`(@D006.Core.Word (8%N))`).
- Command: `D006_WORK=/tmp/d006-dev-rocq-ds03 python3 -u tools/d006_check.py dev rocq DS-03`
  (reproduced standalone with the rendered `Check.v`).
- Output excerpt (D3-O01, the first observation with a record list):

  ```text
  File "./Obs.v", line 8, characters 6710-6711:
  Error:
  Syntax error: '|' or ',' or ')' expected after [term level 200] (in [term]).
  ```

  at `... (@D006.Records.Failure) (@cons @D006.Reco...`. Minimal repro:
  `Check (@cons @N (0%N) (@nil (@N))).` fails with the same syntax error;
  `Check (@cons (@N) (0%N) (@nil (@N))).` succeeds.
- Affected: every DS-03 observation with a record list or a reference list:
  D3-O01 to D3-O04 and D3-O31 to D3-O37 (D3-O02 and D3-O32 through
  `(@nil @D006.Records.Record)`). No candidate-side workaround exists: every
  mapped symbol is rendered as `@name`, and a preamble may not reinterpret
  rendered syntax.
- Expected: the element type rendered in argument form, i.e.
  `kind_text = self.argument(kind)` in `Language.items` (Lean's `@X` is an
  atom, so this changes nothing for Lean 4).

Resolution of H-01 (orchestrator): fixed in tools/d006_render.py, Language.items renders the element type through `argument`, which parenthesizes a bare `@symbol` for Rocq. Rocq DS-01 positives re-run clean.

## H-02 (Lean 4, all cases, dev loop only): RLIMIT_AS stops `lean` before it reads the file

Reported by the Lean 4 DS-01 builder, 2026-09-29. Not blocking: the Lean
adapter works around it with tool flags (below); no statement or method changed.

- Where: `tools/d006_check.py`, `_limit`, used by `run_plain`, sets
  `RLIMIT_AS` (address space) to `ceiling["memory_bytes"]`: 4 GiB for
  negatives, 8 GiB for positives and builds. The epoch runner
  (`tools/d006_run.py`) limits memory use with a cgroup
  (`memory.limit_in_bytes`) instead, so the two disagree for any tool that
  reserves more address space than it uses.
- Why Lean hits it: Lean 4.34.1 reserves a 1 GiB stack per worker thread by
  default (`--tstack`), plus one fixed 1 GiB region and a 64 MiB glibc arena
  per thread. Measured on this host (4 cores, default flags, 8 OS threads):
  VmSize 8.1 GiB, VmPeak 8.4 GB, VmRSS 0.9 GB while checking one DS-01 file.
  With `--threads=1` it still starts 5 threads and needs 4.8 GiB.
- Command: the dev loop with the Lean adapter's `check.argv` set to the plain
  `["{toolchain}/bin/lean", "--json", "{file}"]`:
  `D006_WORK=... python3 -u tools/d006_check.py dev <lean candidate> DS-01 --negatives-only --only D1-N01,D1-N04,D1-N08`
- Output:

  ```text
  FAIL D1-N01     term                 expected=['type_failure'] got=None diag=None 76ms 'failed outside the case: failed to create thread: Resource temporarily unavailable\n'
  FAIL D1-N04     obligation           expected=['disproved_obligation'] got=None diag=None 86ms 'failed outside the case: failed to create thread: Resource temporarily unavailable\n'
  FAIL D1-N08     axiom_use            expected=['undeclared_trust'] got=None diag=False 68ms "audit reports ['<no audit>']"
  ```

  Every Lean negative fails this way. Positives (8 GiB) passed in my runs
  with default flags, but the VmPeak measured above is over 8 GiB, so they
  can fail when more threads start (for example on a host with more cores).
- Current workaround (C-02 `adapter.json`): `check.argv` passes
  `--threads=4 --tstack=65536` (64 MiB thread stacks; 3.3 GB of address
  space) and the build steps pass `--tstack=65536`. This also changes when
  the kernel's deep-recursion guard fires, which is why it is recorded here.
- Proposed fix: make `run_plain` meter what the epoch runner meters, e.g.
  run each step in a transient cgroup with the same memory limit (or poll the
  process group's RSS from `/proc/<pid>/status` and kill at the ceiling), and
  keep `RLIMIT_AS` only as a loose backstop (for example 4x the ceiling).
  Then the Lean adapter can drop the `--tstack`/`--threads` pins.

## H-03 (Lean 4, DS-03): the 4097-byte observation D3-O19 does not elaborate at Lean's default `maxRecDepth`

Reported by the Lean 4 DS-03 builder, 2026-09-29. Blocks D3-O19 only; no
candidate definition can change it, because the failure is in elaborating the
rendered statement, before the proof method runs.

- Where: `tools/d006_render.py`, `Language.items` with the Lean adapter's
  `"list_literal": false`: a `["bytes", HEX]` term becomes one nested
  `(@List.cons (@D006.Word (8 : Nat)) (@D006.word_of_nat (8 : Nat) (b : Nat)) ...)`
  per byte. D3-O19 (fixture R-F19, "input above 4096 bytes") is 4097 bytes, so
  its left-hand side is 4097 applications deep. Lean's elaborator recurses once
  or more per nesting level and stops at the default `maxRecDepth` (512).
- Reproduction (standalone, Core only, with the dev loop's flags
  `--threads=4 --tstack=65536`): a file with
  `theorem t2 : (@List.length (@D006.Word (8 : Nat)) <the 4097-deep rendered literal>) = 4097 := by decide +kernel`
  gives

  ```text
  T1.lean:5:18753: error: maximum recursion depth has been reached
  use `set_option maxRecDepth <num>` to increase limit
  ```

  (column 18753 is about 270 levels into the term).
- Two honest fixes, both verified on the same file (each checks in about 4 s):
  1. `"list_literal": true` in the Lean `adapter.json` (a DS-01/orchestrator
     scalar; fragments may not change it). T-05, C-08, C-09 are core `List`,
     `List.nil`, `List.cons`, so the contract allows it; Lean's `[a, b, ...]`
     macro splits long literals to keep the syntax shallow, and
     `(([...]) : @List (@D006.Word (8 : Nat)))` elaborates with the defaults.
     It changes the rendering of every Lean list literal (DS-01 included), so
     DS-01 needs a re-run.
  2. `set_option maxRecDepth 100000` in the DS-03 preamble (or
     `-DmaxRecDepth=100000` on the check command). It changes no statement's
     meaning, only an elaborator limit, but the DS-03 brief says the preamble
     holds imports only.
- Until one of these is chosen, D3-O19 fails in the Lean dev loop with
  "maximum recursion depth has been reached" in its statement.

Resolution of H-03 (orchestrator, recorded by the Lean DS-03 builder): option 1, `"list_literal": true` in the Lean `adapter.json`; the DS-03 preamble stays imports only. Lean DS-01 positives and DS-03 (D3-O19 included) pass with it.

## H-04: artifact negatives had no diagnostic conformance (fixed)

The smoke run of Rocq DS-01 to DS-03 reported M-15 at 15/16 because the
artifact-truncation negative (D2-M05) never computed `diagnostic_conforms`.
An artifact's location is its file, so the harness now requires the
diagnostic to name the truncated file, besides the shared id, a category and
bounded output. Both candidates' D2-M05 diagnostics conform.

## H-05: the Rocq DS-06 re-check rechecked Stdlib (fixed)

`coqchk -norec D006.Core D006.Sieve D006.Records` applies `-norec` to the
first module only, so the other two were checked with their dependencies,
Stdlib included (31 s instead of 2 s). The DS-06 entry now repeats `-norec`
for each module, which matches `leanchecker MODULE...` checking only the
named modules.

## H-06: a command the host cannot execute ran as a shell script (fixed)

Found by the Rocq DS-05 builder. Steps ran as `fs_sandbox RULES -- env -i
VARS COMMAND`, and `env` uses `execvp`, which retries a file the kernel refuses
with ENOEXEC as a `/bin/sh` script. Negative D5-N01 (the AArch64 checker
launched without its emulator) therefore ended in a shell syntax error, exit 2,
with no category. The launcher now runs `env -i VARS fs_sandbox RULES --
COMMAND`: `env` only clears the environment, and the sandbox's own `execv`
reports the refusal as `failed at execute (errno 8)`, exit 125. That message is
the step's failure, not a launcher defect, and D5-N01 reads it as
`unmet_target_assumption`. The sandbox source is unchanged.

## H-07: DS-04 solver claims read the step state record (fixed before any epoch)

The first sandboxed DS-04 smoke passed the whole step-state record to the
solver claim instead of its kind, so every solver run, the fresh certificate
included, was claimed `unknown`. The claim now takes the kind and refuses
anything else; D4-R01 to D4-R05 are re-run in the next smoke.

H-06 follow-up: the sandbox takes only an absolute program path, and `env`
had been searching PATH for adapter commands such as Lean's `mkdir -p`. The
launcher now resolves a bare program name on the step's own PATH the way
`execvp` does (a name with a slash against the step's directory), and a name
it cannot find fails the step with ENOENT. The recorded argv keeps the
adapter's spelling.

## H-08: D6-F04 was judged before its evidence existed (fixed before any epoch)

The full smoke reported D6-F04 unmet for both candidates although both
builds completed and the read-only home was unchanged: the runner called
the fault's rule before recording `home_unchanged`, so the rule always saw
it missing. The evidence is now recorded first.

## H-09: a correction round's archive manifest bound the earlier summary (fixed after the epoch)

Epoch `d006-e-c7b6648ae3988234297f` was summarized after its first attempt,
as the correction window expects. The correction round's `execute` then
rewrote the archive manifest and listed that summary in it, so once
`summarize` rewrote `summary.json` for the latest attempt the archive could
not verify: the manifest wanted the first attempt's summary, and `verify`
wanted a summary that regenerates from the records. Every other file matched
the manifest.

The runner now leaves `summary.json` out of an archive manifest, since
`verify` regenerates it anyway. The epoch's archive manifest was edited to
match: its one `summary.json` row (SHA-256
`23ee6f605f15ce7c5736ae683e7bc6e31d8e836f9e2843a96adc1191b6175d45`, the first
attempt's summary) was removed, after checking that the remaining 1,170 rows
equal what the fixed runner writes for the same tree. The manifest's SHA-256
went from `021755ccf47be23919331aa0d483a041399c94d51e8328399752fee1b2eb94cd`
to `7cd6b81b08bcfd6e5f4d411010cf1d53ef8786ec338b36d16ff090fa3ea400f1`, which
the export records as `archive_manifest_sha256`. No record or log changed.
The runner the epoch bound is unchanged at the epoch's revisions.
