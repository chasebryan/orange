# C-02 Lean 4 candidate: implementation log

## DS-01 Core fragment (2026-09-29, first case built; about 28 minutes, 02:11 to 02:39 UTC)

What is here: `D006/Core.lean` (every DS-01 T-, C- and F- symbol, M-01 and
D1-TH01 to D1-TH12), `adapter.json` (the Lean base adapter) and
`patches/D1-N10.patch`. The dev loop prints `0 failure(s)` for DS-01: 12
parity checks, 2 instance checks, 20 observations, 10 negatives.

### Design

- `Word w` is `abbrev Word w := BitVec w`. Core's `BitVec w` wraps a
  `Fin (2 ^ w)`, so it has exactly one value per natural below `2 ^ w`, and
  the abbreviation keeps core's `DecidableEq` and decision instances. Each
  word operation is a one-line `def` over core (`BitVec.ofNat`, `toNat`, `+`,
  `^^^`, `&&&`, `~~~`, `rotateLeft`, `<<<`). Next to them, short lemmas check
  that core computes what semantics.md states: `n % 2 ^ w`, addition mod
  `2 ^ w`, `2 ^ w - 1 - x`, `x * 2 ^ r % 2 ^ w`. For rotation, core's
  `rotateLeft x r` is `rotateLeftAux x (r % w)`: it reduces the amount mod
  `w` itself, and at width 0 `BitVec 0` has one value, so it is the identity
  (`word_rotl_mod`, `word_rotl_width_zero`, `getMsbD_word_rotl`).
- Width arguments are implicit where a word argument fixes them
  (`word_add {w} (x y)`), explicit in `word_of_nat (w n)`. The harness writes
  `@name` with every argument, so only the binder order has to match the
  shared signature. Theorems keep all binders explicit so that
  `theorem parity : <statement> := D006.name` elaborates as is.
- `Seq A n` is `abbrev Seq A n := Vector A n` (core). Two core operations
  are avoided on purpose: `Vector.zipWith` goes through `Array.zipWith`,
  whose loop is well-founded recursion, and `Vector.reverse` likewise. The
  elaborator treats them as irreducible, so plain `decide` and `rfl` get
  stuck on them; `decide +kernel` does get through (the kernel unfolds the
  accessibility proof), but that depends on how core proves termination.
  So `seq_map2` zips the element lists with `List.zipWith`, and the
  little-endian conversions list their bytes explicitly: every observation
  also evaluates under plain `decide`. The only recursive function in the
  file, `group4`, is structural.
- Endianness: `word32 a b c d` is the Horner form
  `((a*256 + b)*256 + c)*256 + d` (so "first byte most significant" is
  readable in the definition), and `byte x k` is `x.extractLsb' (8*k) 8`.
  All three round trips go through `toNat` and `omega`, which handles
  division and remainder by numerals.
- `Result A E` is a new inductive because core's `Except` takes its
  arguments in the order `E A` and the shared C-04 is `ok A E value`.
  `DecodeError` is an enumeration; both derive `DecidableEq`.
- F-16 maps directly to `List.length` (it already returns `Nat`); F-21 and
  F-22 are `Nat.pow` and `List.append`.
- M-01: Lean has no ML-style functors; its parameter mechanism is ordinary
  arguments. `quarterRound (w r1 r2 r3 r4 : Nat)` is defined once, and
  `chacha_qr := quarterRound 32 16 12 8 7`, `toy_qr := quarterRound 8 4 3 2 1`.
  The instance template is `(@D006.quarterRound {w} {r1} {r2} {r3} {r4})`,
  checked by `rfl`.
- D1-TH06 is proved bit by bit with core's
  `getMsbD_rotateLeft : (x.rotateLeft r).getMsbD i = (i < w && x.getMsbD ((r + i) % w))`
  and `Nat.add_mod_mod`. D1-TH11 splits the decoder's `if` chain; the
  byte-level fact is `group4_spec` (`4 n` bytes give `n` words and encode
  back to the same bytes).

### Methods

- Observations: `decide +kernel`. The kernel alone reduces the `Decidable`
  instance (the elaborator does not evaluate first), like Rocq's
  `vm_compute`: the evaluator is the kernel with its GMP arithmetic on `Nat`
  literals. The whole DS-01 check file (12 parity checks, 2 instances, 20
  observations, 34 audits) checks in under 0.5 s. Plain `decide` also proves
  every observation, but it first evaluates in the elaborator, under the
  elaborator's own limits (`maxRecDepth`, heartbeats,
  `exponentiation.threshold`; on D1-N09 it warns about the threshold and stops
  at "maximum recursion depth has been reached"), and the kernel then
  evaluates again. With `+kernel` the only evaluator is the kernel.
- Exhaustive: also `decide +kernel`. For `∀ x : Word w, P x` it uses core's
  `BitVec.instDecidableForallBitVec`, which splits on one bit at a time down
  to all `2 ^ w` values. It is a real enumeration: a file proving the true
  `∀ x, word_xor x x = 0` and refuting the false `∀ x, word_add x x = 0` at
  one width, under the 4 GiB negative ceiling, takes 0.9 s at width 8, 3.5 s
  at 12 and 13.6 s at 14, and runs out of memory at 16 (memory grows with
  the number of cases evaluated). At width 32 (D1-N07) it ends with
  `std::bad_alloc` after about 27 s, classified `resource_exhaustion`.
- Instances: `rfl`.

### Negatives (message, category)

- D1-N01, D1-N03: "Application type mismatch" -> `type_failure`.
- D1-N02: "fail to show termination for d1_n02_loop" -> `non_total`.
- D1-N04, D1-N05: "Tactic `decide` proved that the proposition ... is false"
  -> `disproved_obligation`.
- D1-N06: "don't know how to synthesize placeholder for argument `w`" -> `type_failure`.
- D1-N07: out of memory (see above) -> `resource_exhaustion`.
- D1-N08: `#print axioms` reports `neg_D1_N08_false` -> `undeclared_trust`.
- D1-N09: "(kernel) the kernel refused to evaluate `Nat.pow` because its
  second argument does not fit in a 32-bit unsigned integer". Lean's kernel
  has a fixed size ceiling on the exponent it will evaluate, and that ceiling
  ends the step, so the classifier maps it to `resource_exhaustion` (the
  taxonomy's "size ceiling"). It fails in about 0.3 s, not by timing out.
- D1-N10: the patch makes the trailing branch return the counted words,
  `.ok (group4 (rest.take (4 * n)))`, instead of `.err .trailing`. The
  patched build then fails with "unsolved goals" in `decoding_is_canonical`
  only (D1-TH10 still checks) -> `proof_failure`.

### Classifier

Rules match Lean 4.34.1's own wording: recursion, heartbeat, stack, memory
and kernel size ceilings first (any phase); termination failures (`definition`,
`build`); type mismatches and unresolved placeholders or instances
(`statement`); `decide` refutation (`proof`); `decide` stuck without a verdict
as `unknown` (`proof`); anything else in a proof or a library rebuild as
`proof_failure`. Lean's "(deterministic) timeout" is a heartbeat limit, which
counts allocations rather than wall-clock time, so it counts as resource
exhaustion.

### Trust

- `#print axioms` reports only `propext` and `Quot.sound`, for the theorems
  and for most observations and instances. In the observations they come from
  proof fields inside core definitions (the bound proofs of `BitVec.xor`,
  `BitVec.not`, `BitVec.or`, Vector index proofs, and `group4`'s
  structural-recursion encoding), never from computation. `Classical.choice`
  does not appear. No `native_decide`, so no `Lean.ofReduceBool` and no compiler.
- Imported `.olean` files (core `Init`, and the built `D006.Core`) are not
  re-checked at `lean`'s default trust level; this is listed in the inventory.

### Build

- Direct `lean -o .lake/build/lib/lean/D006/Core.olean D006/Core.lean` from
  the source root after a `mkdir -p`; `LEAN_PATH={src}/.lake/build/lib/lean`
  for the build and the checks. `.lake` is skipped when the harness copies
  the candidate, so no build output is ever copied in. No lake, no network.
- The `.olean` is byte-identical across `--threads=1` and `--threads=4`, and
  across build directories (checked by sha256 on four builds).
- `--json` on the build too, so that a failed patched build gives positioned
  messages.

### Harness notes, dead ends

- H-02 in `HARNESS_ISSUES.md`: the dev loop limits address space
  (`RLIMIT_AS`). Lean's default 1 GiB thread stacks make plain `lean` fail
  at start under the 4 GiB negative ceiling ("failed to create thread:
  Resource temporarily unavailable"). The adapter pins `--threads=4
  --tstack=65536` for checks and `--tstack=65536` for builds. No statement
  or method is affected.
- For DS-02 (artifact truncation): importing a `.olean` cut in half makes
  `lean` die with SIGSEGV (exit 139) and print nothing, so a check for it
  needs something other than a plain import.
- Small dead ends: `simp` with the byte-list lemma also rewrote under the
  `flatMap` lambda so the induction hypothesis no longer matched (fixed with
  `rw`); the first `rotations_compose` proof had a redundant `add_comm`.

## DS-03 canonical records, OCR1 (2026-09-29; about 90 minutes, 02:38 to 04:08 UTC)

What is here: `D006/Records.lean` (imports `D006.Core` only; every R- symbol,
the proofs and D3-TH01 to D3-TH05) and `adapter.d/ds03.json` (symbols,
theorems, preamble, the Records build step after Sieve's, trust inventory).
The dev loop prints `0 failure(s)` for DS-03: 5 parity checks and 44
observations (DS-03 has no negatives), and DS-01 positives still pass.

### Design

- Everything lives in namespace `D006.Records` (the module's name), so no
  helper can clash with `D006.Sieve` if a later module imports both.
  `Record`, `ErrorCode` and `Failure` are inductives deriving `DecidableEq`;
  their constructors take their arguments in the signature's order, and
  `Failure.failure` carries a core `String` path (T-08). `Result` is Core's.
- Encoder: `encode_records = header ++ uvar_encode count ++ flatMap
  encode_record`, and `encode_record r = tag :: uvar_encode name.length ++
  name ++ encode_fields r`. `uvar_encode` is total (two bytes wrap mod 256
  above `2 ^ 14`, where the reference raises); a valid list only has numbers
  up to 200, so this never matters for a theorem.
- Decoder: a small parser, `structure Parser α` wrapping
  `List (Word 8) → Result (α × List (Word 8)) Failure`, with a `Monad`
  instance so each step reads as `do` code in the reference's order:
  `readMagic` (three bytes), version, `readNumber` (one or two bytes,
  malformed before noncanonical), then `readRecords count [] count`, where
  `readRecord` reads tag, `readName` (length, bytes, UTF-8, duplicate, order)
  and `readFields` by tag. Loops recurse structurally on the number of items
  left (records, references), so everything reduces in the kernel; the only
  length check is `decode_records`' first `4096 < b.length`. Paths are built
  with `"records/" ++ toString k ++ "/name"` and friends, which the kernel
  evaluates.
- `Parser` is a structure, not a `def` of a function type: with a `def`,
  `(p >>= f) bs` is not type-correct at reducible transparency, and `rw` and
  `split` fail on it ("function expected"). With `.run` everything rewrites.
- `records_valid` is written from semantics.md section 4 and never calls the
  decoder: `l.length ≤ 64 && records_ok_after [] l && (encode_records
  l).length ≤ 4096`, where `records_ok_after` checks each record against the
  records before it (`record_ok before r`): a valid name (`1 ≤ length ≤ 200`,
  `utf8_valid`), a name that sorts after the previous record's
  (`bytes_lt`, bytewise with a proper prefix first; checked on adjacent
  records, which is "strictly increasing"), and `fields_ok`: 32-byte digest
  or fingerprint, at most 64 strictly `increasing` references each below
  `before.length`, a claim's reference below `before.length` naming a
  theorem (`is_theorem before ref`), level at most 3. The size condition is
  stated on the encoding, as semantics.md states it.
- `utf8_valid` is a structural left-to-right scan: `utf8_scan pending bs`
  keeps the ranges still owed by the current character, and `utf8_ranges`
  is the RFC 3629 table (lead byte to continuation ranges) written out.
- Proofs: for each step a backwards lemma (`*_ok`: if the step succeeds, the
  bytes it read are the encoding of its value, which meets what the step
  checked) and a forwards one (`*_encode`/`*_bind`: on such an encoding the
  step returns the value and the rest). `readFile_ok` gives D3-TH02 and
  D3-TH03 (with the 4096 check for the size condition), `readFile_encode`
  gives D3-TH01, D3-TH04 is the first check, D3-TH05 is D3-TH02 twice. The
  name order needs `bytes_lt` to be irreflexive, asymmetric and total on
  distinct lists (the decoder tests `name = prev` and `bytes_lt name prev`,
  validity states `bytes_lt prev name`).

### Checks beyond the dev loop

- Differential test against the plain-Python reference (scratch file, not
  kept): 150 mutated fixtures through `decode_records` (all 16 error codes
  appear; 34 inputs accepted), 120 random record lists
  through `records_valid` (33 valid, 87 not), `encode_records` and a decode
  of the encoding, and 150 random byte strings through `utf8_valid`; 660
  `decide +kernel` examples, all agree with the reference (a deliberately
  wrong path is refused, so the check is real). About 27 s.
- `Records.olean` is byte-identical at `--threads=1` and `--threads=4`.
  `leanchecker D006.Records` (toolchain on `PATH`, `LEAN_STACK_SIZE_KB`
  set) re-checks it in about 1 s.
- The DS-03 check file checks in about 7 s; most of it is parsing and
  elaborating the literals (D3-O19 alone is 4097 bytes). No single item takes
  more than 0.3 s.

### Trust

- `#print axioms`: the five theorems use `propext` and `Quot.sound`; 37
  observations use `propext` only, the seven `utf8_valid` observations use
  nothing. `propext` comes from proof fields, not computation: core's
  `String.append` proves its result is valid UTF-8, and the recursion
  encodings of `increasing` and of `before[i]?` use it. Inventory additions:
  the built `D006.Records`, the kernel's String literal reduction, and the
  DS-03 note on `propext`.
- `Classical.choice` showed up at first and was removed: `omega` proves a
  compound goal (a conjunction or disjunction of arithmetic facts) through
  classical logic, while atomic goals stay constructive, so the proofs call
  `omega` only on atomic goals. `beq_self_eq_true` (`x == x`) also pulled
  it in, so the definitions use `decide (a = b)` instead of `==`.

### Harness notes, dead ends

- H-03 in `HARNESS_ISSUES.md`: with `list_literal: false` the 4097-byte
  input of D3-O19 was rendered as 4097 nested `List.cons` and the elaborator
  stopped at `maxRecDepth` while elaborating the statement. The orchestrator
  set `list_literal: true` in `adapter.json` (Lean's `[...]` literal keeps
  long lists shallow); nothing in DS-03 depends on the preamble.
- `obtain ⟨..., rfl⟩` in the backwards proofs substitutes a byte list,
  which reintroduces the older hypothesis named `h` after the newer one, so
  the next step silently used the wrong `h`. The lemmas were reshaped
  (`checkRef_bind_ok`, `readMagic_bind_ok`) so no substitution happens
  mid-proof.
- While the DS-02 fragment's Sieve step did not yet compile, the dev loop
  was run on a private copy without `ds02.json`; the final runs use the
  real candidate directory (Sieve builds now).

## DS-02 Sieve (2026-09-29; about 1 h 40 min of work: 02:38 to 04:12 and 05:17 to 05:21 UTC, paused in between by a usage limit)

What is here: `D006/Sieve.lean` (every S- symbol, D2-TH01 to D2-TH06),
`adapter.d/ds02.json`, `patches/D2-M01.patch`, `D2-M02.patch`,
`D2-M06.patch`, and `Loader/OleanCheck.lean` (the independent check for
D2-M05). The dev loop prints `0 failure(s)` for DS-02: 6 parity checks, 27
observations, 6 mutations. No change to `Core.lean` or `adapter.json`.

### Design

- Types are plain inductives, the four record-like ones are structures with
  the shared constructor names (`VarDecl.vdecl`, `ArrDecl.adecl`, `Env.env`,
  `State.state`), so the renderer's `@D006.Sieve.Env.env vars arrs` is the
  constructor and proofs get projections (`g.vars`, `s.arrs`). `public` is a
  keyword in Lean 4.34, so the constructor is declared as `«public»`; it is
  still referred to as `Label.public`, and the adapter name
  `D006.Sieve.Label.public` resolves.
- Words are Core's `Word 8`; the operators use Core's `word_add`,
  `word_xor`, `word_and`; indices go through `nat_of_word`.
- `step` and `run` follow `d006_shared.py` case by case (evaluation order,
  where observations are kept on failure, `stuck` for a missing variable or
  array, `seq skip c` silent). The `seq` wrapping is `StepResult.inSeq` so
  that a lemma can talk about it. `run` returns `(outcome, trace)`;
  `run_trace` and `run_outcome` are its two projections. The recursive call
  is matched once (`match run n s' c' with | (o, t') => ...`) rather than
  projected twice, so kernel evaluation stays linear in the fuel.
- `wf` uses a small `Forall₂` (same length, related position by position)
  with a structural `Decidable` instance, so `decide` checks it on closed
  states. `low_eq` is stated as in semantics.md: for every index whose
  declaration is public, the two states' `xs[i]?` agree. It is not decidable
  as stated; D2-TH06's one instance is proved by hand (`low_eq_sigma`).
- Everything is structurally recursive; no well-founded recursion, no
  `partial`, so the kernel reduces `well_typed`, `step` and `run` directly.

### Proofs

- One expression lemma, `eval_agree` (two well-formed, publicly equivalent
  states: same observations, both fail or both succeed with values of the
  type, equal when public), and one step lemma, `step_agree`, by structural
  recursion on the command, into `StepAgree`: both `done`; both fail alike;
  or both `next` with the same command, which is well typed, well-formed
  states, and either the same observations with publicly equivalent states
  or observations that differ only in a final release of different values.
- D2-TH01 and D2-TH02 are `step_agree` with the two states equal
  (`low_eq_refl`): `StepAgree` of `stuck` is `False`, and the `next` case
  carries typing and well-formedness. D2-TH03 unfolds `StepAgree` into the
  four shared disjuncts. D2-TH04 is induction on the fuel with
  `run_done`/`run_fail`/`run_next`. D2-TH05 is D2-TH04 applied to Γ+ and P+
  (`by decide` for `well_typed Γ+ P+`). D2-TH06: `decide` for `wf` and the
  two traces of P- (`p = r1 = r2 = []`), `low_eq_sigma` for the rest.
- Trust: the audits report only `propext` and `Quot.sound` (already allowed);
  `Classical.choice` does not appear. No trust additions; the fragment adds
  inventory entries only (the built D006.Sieve, the axioms' provenance, the
  kernel as the observation evaluator, the loader below).

### Methods and numbers

- Observations: the base `decide +kernel`, unchanged (no `methods` entry).
  The DS-02 check file (6 parity checks, 27 observations, 33 audits) checks
  in about 0.8 s under a 4 GiB address-space limit. Building `Sieve.lean`
  takes about 4 to 5 s; the `.olean` (4.6 MB) is byte-identical with
  `--threads=1` and `--threads=4` and across build directories.

### Mutations

- D2-M01 (patch: the `cond` rule matches `some (.ty_bool, _)`): the build
  fails in `wt_cond`, the typing inversion `step_agree` uses for branches
  ("Application type mismatch ... typeOf g e = some (Ty.ty_bool, Label.public)")
  -> `proof_failure`.
- D2-M02 (patch: `assign` stores into `x + 1`): the build fails in the
  assignment case of `step_agree` (`wf.setVar w1 hd` no longer applies)
  -> `proof_failure`.
- D2-M03, D2-M04 (rendered obligations): "Tactic `decide` proved that the
  proposition ... is false" -> `disproved_obligation`.
- D2-M06 (patch: D2-TH02's tactic block replaces D2-TH03's): "Unknown
  identifier `s`" in `lockstep_noninterference` -> `proof_failure`.
- The three patches are regenerated from the current `Sieve.lean` by exact
  text substitutions (`diff -u`), so their context always matches.
- D2-M05 (artifact truncation of `.lake/build/lib/lean/D006/Sieve.olean`):
  as the DS-01 note says, `lean` importing a truncated `.olean` dies with
  SIGSEGV and no output, and so does `leanchecker` (tried: exit 139). Lean
  maps the file and follows its pointers without bounds checks. So the check
  is `lean --run Loader/OleanCheck.lean {artifact}`, a small loader in Lean
  (Init/Lean only, interpreted): stage 1 reads the bytes and checks the
  header (marker, Lean version and commit), that the objects tile the file
  exactly (sizes from the object headers, layouts from `lean.h`), and that
  the root and every reachable pointer point at an object in the file;
  stage 2, only for a sound file, loads it as `leanchecker` does and replays
  every declaration through the kernel. For the half file it prints
  `error: <file>: malformed compiled module: ...` and exits 1; the fragment's
  `artifact`-phase rule maps "malformed/incompatible compiled module" to
  `parse_failure` and "kernel check failed" to `proof_failure`.
  Tested: on the intact Sieve.olean and Core.olean it passes both stages
  (128k objects, 982 declarations replayed, about 8.7 s under a 4 GiB
  limit); stage 1 also accepts eight toolchain `.olean` files (Prelude,
  BitVec lemmas, Lean.Meta.Basic, ...); cuts at 0, 91 bytes, 10 %, 33 %,
  50 %, 77 % and all but 5 bytes each give a message, never a crash. It is
  not in the build (no module imports it; `lean --run` elaborates it from
  source each time).

### Dead ends and small things

- The first `run` projected the recursive call twice (`(run ..).1`,
  `t ++ (run ..).2`); replaced by one `match` before any measurement.
- Dot-notation on list-recursive `Forall₂` proofs needed explicit list
  arguments in the recursive calls (`getElem? (xs := xs) (ys := ys) h.2 hx`),
  otherwise the expected type fixed the wrong lists.
- `simp only [Agree]` on an implication turned `public = public → ...` into
  `True → ...`; the case splits now simplify the hypothesis instead
  (`intro ha <;> simp only [Agree] at ha`), which also closes the mismatched
  cases.
- The loader first freed the module's compacted region while the replayed
  constants were still referenced and crashed on a valid file; it now keeps
  the regions mapped until exit.
- For the orchestrator's `ds06.json`: DS-02 adds the compiled module
  `D006.Sieve` (recheck `leanchecker D006.Sieve`, 1 s; exit 0).

## DS-04 LRAT-backed bit-vector proof (started 2026-09-29 04:08 UTC, finished 2026-09-30 about 00:50 UTC; interrupted by usage-limit pauses and two container restarts, so the span is not the working time)

What is here: `D006/Lrat.lean` (imports `D006.Core` only; the DS-04
signature, the canonical bit-blast, the LRAT checker, its soundness proof and
D4-TH01 to D4-TH05) and `adapter.d/ds04.json` (symbols, theorems, preamble,
the Lrat build step, `methods.observation.DS-04 = native_decide`, the
native_decide assumption pattern and the trust inventory). The dev loop
prints `0 failure(s)` for DS-04: 5 parity checks and 14 observations
(D4-CNF01/02, D4-G-V-00 to 11); DS-04 has no negatives. The epoch's
fresh-certificate observations D4-F-V-00 to 11 also pass (checked with the
harness's own `certificate_observations` and `run_check_source`).

### Design

- `Obligation.lhs`/`rhs` are `Term`s over `x`, `y`, add, xor, and and shift
  by a constant. `Term.eval` gives the DS-01 word meaning, so `holds` unfolds
  to the shared statements and D4-TH02/D4-TH03 are `Iff.rfl`. D4-TH05 applies
  the hypothesis to `x = y = 1` and closes `2 ≠ 1` by `decide`.
- `Term.blast` is the canonical Tseitin encoding of the shared rules, blasting
  the same `Term`s: variable 1 is constant false, x's bits are 2..33 and y's
  34..65, one gate per variable after that. The ripple-carry adder follows
  rule 3 (`t`, `s`, `u`, `v`, `c'`), `shl` is `(replicate k 1 ++ bits).take 32`,
  and the miter is XOR per bit followed by an OR chain. `blast` returns the
  variable count and the clause list; `cnf_text` prints DIMACS. It matches the
  shared `ds04-carry-save.cnf` byte for byte.
- `lrat_verdict` compares the CNF text, refuses anything over 4 MiB, then
  works on the certificate's bytes (`toByteArray.data.toList`). It splits
  lines, recording for each whether it ended in a line feed. Tokens are parsed
  under the canonical decimal rule, and each line is checked in the shared
  order: `parse`, `id_order`, `var_range`, `lemma_form`, `rat_unsupported`,
  `unknown_hint`/`rup` along the hints, then `unknown_deletion`,
  `trailing` and `no_empty_clause 0`. Active clauses and the propagation
  assignment live in binary tries keyed by naturals, least significant bit
  first, with `get?`/`set` by well-founded recursion and the lemmas
  `get?_leaf` and `get?_set`. This keeps lookups logarithmic in the compiled
  checker and gives simple lemmas.
- Soundness (D4-TH04, `accepted_certificate_proves`), for every obligation
  and every pair of strings:
  - `litTrue` reads a literal under a total assignment, and literal 0 is never
    true.
  - `Agrees` says every literal in the trie assignment is true.
  - `propagate_sound`: RUP along the hints cannot reach a conflict while
    every active clause is true.
  - `Replay.Sat` is the invariant that every active clause is true.
    `checkLine_sound` and `replay_sound` preserve it, so an accepting run
    under a satisfying assignment would have derived a true empty clause.
    `verdict_sound` concludes that an accepted certificate leaves the CNF
    from `blast o` with no satisfying assignment.
  - `blast_satisfiable` is the other half: when the identity fails for some
    `x`, `y`, evaluating the gates on them (`value (inputs x y) gates`)
    satisfies every clause. The chain is `Circuit.Sound` and `Den`/`Grows`
    for the numbering, `Forall2` from blasted literals to `bitsOf` the
    word, then `bitsOf_xor/and/shl` and `bitsOf_add` (the adder's bits
    against `BitVec.getLsbD_add` through `carryAt`), and finally
    `miter_spec` with `any_xor_of_ne`.
- D4-TH01 (`carry_save_identity`) is
  `holds_carry_save.mp (accepted_certificate_proves .carry_save _ _ golden_certificate_accepted)`.
  `golden_certificate` is the shared golden file as one string-gap literal,
  one certificate line per source line; it is checked equal to the file.
  `golden_certificate_accepted` (`lrat_verdict .carry_save (cnf_text
  .carry_save) golden_certificate = .accept`) is the only `native_decide` in
  the module.

### Methods and numbers (this container, x86-64, under a 4 GiB address-space limit)

- `methods.observation.DS-04 = native_decide`. The DS-04 check file (5.9 MB,
  with the 4.2 MB V-08 literal, 5 parity checks, 14 observations and their
  audits) takes 2.3 s wall, 2.3 s CPU and 569 MB peak RSS. This is the
  heaviest DS-04 check step.
- Build step `lean --threads=1 ... D006/Lrat.lean`: 7.0 s wall, 7.0 s CPU,
  589 MB peak RSS; this is the heaviest DS-04 step. At `--threads=4`: 3.7 s
  wall. The `.olean` is byte-identical at 1 and 4 threads and to the dev
  loop's. The dev loop's whole build (Core, Sieve, Records, Lrat) took
  18.9 s.
- `leanchecker D006.Lrat` (`LEAN_STACK_SIZE_KB=65536`): exit 0, 2.4 s,
  443 MB. For the orchestrator's `ds06.json`, the recheck entry is
  `["{toolchain}/bin/leanchecker", "D006.Lrat"]`.
- Differential test against `d006_shared` through the compiled definitions
  (`lean --run` on a small driver reading files):
  - 400 random mutations of the golden and fresh certificates: one to three
    edits of line deletion, duplication or swap, digit changes, hint swaps
    or removals, inserted characters, truncation, id changes and extra
    literals or hints, with occasional B-C02 claims and altered CNF text.
  - 92 targeted cases: dropped tail lines, no final newline, empty input,
    CRLF, duplicate or complementary lemma literals, `-0`, `+n`, leading
    zeros, re-deletion, and exactly 4 MiB and one byte over.
  - Result: 0 mismatches. Every rejection code appears (parse,
    cnf_mismatch, oversized, id_order, var_range, lemma_form,
    rat_unsupported, unknown_hint, rup, unknown_deletion, trailing,
    no_empty_clause), as do accepts.

### Kernel evaluation: why native_decide, and what I tried

The brief asked for kernel-checked computation where it fits. For reading a
certificate it does not fit. In 4.34.1 a `String` literal reaches the kernel
as `String.ofList [chars]`. Its bytes come out of `List.utf8Encode` and
`List.toByteArray`, a push/append chain whose kernel whnf is quadratic in the
length, so every function that looks at the bytes pays for it. (For
comparison, `ByteArray.append` on `⟨⟨a.data.toList ++ b.data.toList⟩⟩` is
linear.) Measured with `decide +kernel` and `lean --tstack=65536`:

| experiment | time | peak memory |
| --- | --- | --- |
| baseline file (import only, no literal) | 1.8 s | 469 MB |
| `utf8ByteSize` of a 1 KB literal | 3.4 s | 589 MB |
| `utf8ByteSize` of a 16 KB literal | 23 s | 2.27 GB |
| newline count of a 7.7 KB literal | 68 s | 3.7 GB |
| golden certificate as per-line literals, left-nested join | killed at 600 s | 13.6 GB |
| golden certificate as per-line literals, right-nested join | out of memory after 62 s | 6 GB cap |
| golden certificate as a 62 K-element `UInt8` list literal | heartbeat timeout after 50 s | n/a |
| Nat-chunk encoding: newline count | 56 s | 3 GB |
| Nat-chunk encoding: decode plus length only | 19 s | 1.7 GB |
| recursor-style (non-compiled) definitions of the byte functions | stack overflow | n/a |

The golden certificate is 62 KB and V-08 is 4.2 MB, and none of these runs
even reached the checker itself. The coordinator agreed to option (b): keep
`native_decide` only for the observations and the golden-acceptance step of
D4-TH01, and keep everything else kernel-checked.

### Trust

- In 4.34.1, `native_decide` (`Lean.Meta.Native.nativeEqTrue`) compiles the
  decision procedure, evaluates it with the IR interpreter and the compiled
  Init runtime, and adds a fresh axiom
  `<decl>._native.native_decide.ax_N_M : e = true`. `#print axioms` reports
  that name, not `Lean.ofReduceBool` (now deprecated). The fragment allows
  exactly that shape with
  `[A-Za-z0-9_.]+\._native\.native_decide\.ax_[0-9]+_[0-9]+`, and the
  inventory entry names what it trusts: the compiler, code generator, IR
  interpreter and runtime, including the C primitives for String, ByteArray
  and GMP Nat, together with the numbers above.
- Audits:
  - `holds_carry_save`, `holds_carry_save_unshifted`,
    `carry_save_unshifted_refuted`, `accepted_certificate_proves` and every
    soundness and blast lemma: at most `[propext, Quot.sound]`.
  - `golden_certificate_accepted` and `carry_save_identity`: `[propext,
    Quot.sound, D006.Lrat.golden_certificate_accepted._native.native_decide.ax_1_1]`.
  - Each observation: `[propext, Quot.sound, obs_..._native.native_decide.ax_1_1]`.
- Keeping `Classical.choice` out took some work. It came in through:
  - `Nat.mod_pow_succ` (replaced by a constructive `mod_two_pow_succ'` from
    `Nat.div_add_mod`, `Nat.add_mul_mod_self_left` and `Nat.mod_eq_of_lt`),
  - `BitVec.carry_succ` (replaced by `carry_succ'`),
  - `List.filter_eq_nil_iff` (replaced by `List.mem_filter` and
    `List.not_mem_nil`),
  - `simpa` on a `decide` equation (replaced by `Option.some.inj` and
    `decide_eq_true`/`decide_eq_false`),
  - `simp_all`,
  - `omega` on a conjunction goal. `omega` proving `2 ≤ 2 + i ∧ 2 + i < 34`
    uses `Classical.choice`; `⟨by omega, by omega⟩` does not.
  - `by simp at hl` closing a length mismatch to `False` in a match arm
    (replaced by `nomatch hl`).

  The base inventory's elaborator entry still says "No native_decide, no
  compiler". That stays true for DS-01 to DS-03; for DS-04, the ds04.json
  entry declares the widening.

### Dead ends and small things

- In `Term.eval`, the pattern variables `x`/`y` resolved to the constructors
  `Term.x`/`Term.y`; the binders are now `u v`.
- There is no `List.Forall₂` in core, so there is a small `Forall2` inductive
  with a simp lemma for the nil case.
- `unfold miter` left a `have` that `split` could not see through;
  `simp only [miter]` works. The nil case needed
  `generalize hz : List.zipWith ... = Z` before `cases`.
- `omega` did not see bounds inside structure hypotheses, so they are pulled
  out with `have` first.
- `if_pos`/`if_neg` are deprecated in 4.34.1; `ite_eq_left`/`ite_eq_right` do
  the same job.
- A `pkill -f` loop killed my own shell once. Since then, processes are
  stopped by explicit pid.

## DS-05 standalone checker (2026-09-30, 01:01 to 01:30 UTC, about 30 minutes)

What is here: `Checker/Main.lean` (the driver, 123 lines, 66 without doc
comments) and `adapter.d/ds05.json` (the C emission in `build.serial` and
`build.declared_parallel`, the `standalone` entry for H-01 and H-02, one
build-phase classifier rule, trust inventory). Nothing else changed. The dev
loop `dev lean4 DS-05 --fresh` prints `0 failure(s)`: both host builds, the
56 corpus items on each host (30 record fixtures, 2 CNF texts, golden and
fresh V-00 to V-11) and the usage check. `dev lean4 DS-03 DS-04
--positives-only` still passes (68 items).

### Design

- The checker is the Lean compiler's C for `D006.Core`, `D006.Records` and
  `D006.Lrat` plus the driver, linked with the Lean runtime. The driver calls
  `decode_records`, `cnf_text` and `lrat_verdict` and computes nothing else;
  it parses the command line, reads the files (`IO.FS.readBinFile` for
  records, `IO.FS.readFile` for the CNF and the certificate, so a non-UTF-8
  file is an error), prints the verdict line and sets the exit code (0 with
  the output, 2 with the usage on standard error, 1 with `error: ...` when a
  file cannot be read). The section 4 records line is written in the driver:
  a table of the 16 code names (`codeText`, a plain match; `ErrorCode.text`
  would clash with dot notation on `D006.Records.ErrorCode`), lowercase hex
  two digits per byte, references joined by commas, `accept` alone for an
  empty set. The lrat line is `accept` or `reject CODE LINE`.
- C emission is part of the prover build (`build.serial` after Lrat):
  `lean --json --threads=1 --tstack=65536 --root=. -c .lake/build/ir/<M>.c
  <M>.lean` for Core, Records, Lrat and Checker/Main. Reasons: the runner
  records the `sources` manifest (driver, the three Lean sources, the four C
  files) before the host builds run, so the C must exist by then; and the C
  is the same for both hosts. Cost: the earlier steps only write `.olean`, so
  these steps elaborate Core, Records and Lrat a second time (2.2 s, 3.7 s,
  6.6 s at one thread; Main 0.8 s); the dev loop's whole build went from about
  19 s to 31.5 s. If `-c` were added to the existing Core/Records/Lrat steps
  (adapter.json, ds03.json, ds04.json, which I do not edit), the three
  re-elaborations could go. No `.olean` is written by these steps.
- Host builds, from the C: one `clang -c` per module into
  `.lake/build/obj/<triple>/`, then one link into
  `.lake/build/bin/<triple>/d006-checker`. Flags are leanc's (`leanc
  --print-cflags` plus the internal ones `leanc -v` shows: `--sysroot`,
  `-nostdinc -isystem <tree>/include/clang`, `-O3 -DNDEBUG` as Lake's release
  builds), and the link names only what the checker needs: `libInit.a`,
  `libleanrt.a`, static `libc++`, `libc++abi`, `libunwind`, `libgmp`,
  `libuv`, and dynamic glibc through the trees' `lib/glibc` stubs. leanc's own
  list also has Lean, Lake, Std, leancpp, ssl and crypto; the checker uses
  none of them.
  - H-01: TC-01's clang, native.
  - H-02: TC-01's clang with `--target=aarch64-unknown-linux-gnu --sysroot
    {toolchain_aarch64} -resource-dir {toolchain_aarch64}/lib/clang/22`, and
    TC-11's `include`, `lib`, `lib/glibc` and `lib/lean`. Launched with
    `/usr/bin/qemu-aarch64-static -L /usr/aarch64-linux-gnu {binary}`.
- Dynamic, not static, on H-02: TC-11's `lib/glibc` has no `libc.a`, and a
  static binary would leave D5-N01 (run without qemu) and D5-N02 (runtime
  withheld) nothing to fail on. Dynamic, both fail at the loader (below).
- `runtime` is empty on both hosts: GMP, libuv, the C++ library, the Lean
  runtime and Init are linked in; the only shared objects are glibc's, from
  `/lib/x86_64-linux-gnu` (H-01) or `/usr/aarch64-linux-gnu` (H-02), both
  under `/usr`. No file of a toolchain tree is read at check time.
- `dependency` is `lib/lean/libleanrt.a` (the Lean runtime). Without it the
  H-01 link prints `ld.lld: error: unable to find library -lleanrt`; the
  fragment's build rule (`ld.lld: error: unable to find library -l...` or
  clang's `fatal error: '...' file not found`) maps it to
  `unsupported_feature`. Checked by rerunning the H-01 build steps against a
  `cp -al` copy of TC-01 with that file removed and classifying with the
  harness's own `errors`/`classify`.

### Stack and address space

- Compiled Lean runs `main` on a thread whose stack is `LEAN_STACK_SIZE_KB`
  (default 1 GiB). `D006.Lrat.lines` is not tail recursive: one frame per
  certificate line, and a certificate within 4 MiB can have 4,194,304 lines.
  With 4 MiB of line feeds (`reject parse 1`): x86-64 overflows at 192 MiB
  and passes at 256 MiB; AArch64 under qemu overflows at 256 MiB and passes
  at 384 MiB ("Stack overflow detected. Aborting.", exit 134, when too small).
- Besides the stack, the runtime reserves about 2.2 GiB of address space (two
  1 GiB regions, two 64 MiB malloc arenas; 4 threads including libuv's
  io_uring poller). With the 1 GiB default a records run needs between 3 and
  3.5 GiB, so under the 4 GiB cap less than 0.7 GiB is left for the heap.
- So both hosts declare `LEAN_STACK_SIZE_KB=524288` in their run
  `environment`: it covers the worst case on both hosts, and VmSize is then
  2.76 GiB (the worst case passes even under a 2.75 GiB cap).

### Checks beyond the dev loop

- Under `tools/fs_sandbox.c` (compiled in a scratch directory; Landlock with
  only /usr, /proc, /sys/devices/system/cpu, /dev/urandom, the binary and the
  corpus readable, and its 4 GiB address-space and other caps): records, lrat
  and cnf commands on both hosts give the expected output, and so do four
  stress certificates (4 MiB of line feeds, 699,050 deletion lines, a
  2.6 MB single line, 1.3 M hints), which also agree with `d006_shared`.
- Simulated negatives: D5-N01 (AArch64 binary without qemu) "cannot execute
  binary file: Exec format error", exit 126; D5-N02 (`-L` to an empty
  directory) "qemu-aarch64-static: Could not open
  '/lib/ld-linux-aarch64.so.1': No such file or directory", exit 255; both
  match the runner's `NO_TARGET`. D5-N06 as above. D5-N03/N04/N05/N07 are
  the runner's own digest checks or corpus items.
- Determinism: the four C files are byte-identical at `--threads=1` and
  `--threads=4` and across build directories; both binaries are
  byte-identical across build directories (no paths embedded).
- Sizes: H-01 4,589,872 bytes (2,951,960 stripped), H-02 5,106,896 (2,969,544
  stripped). Closure: H-01 `ld-linux-x86-64.so.2`, `libc.so.6`,
  `libpthread.so.0`, `libdl.so.2`, `librt.so.1`; H-02 the same names
  (`/lib/ld-linux-aarch64.so.1` interpreter). `--gc-sections` drops the
  62 KB golden certificate literal and everything else the three entry
  points do not reach.
- Times (dev loop, this container): host builds about 4 s each (four
  compiles and a link), both fine under a 4 GiB cap; the 56 corpus items take
  0.65 s in all on H-01 (at most 52 ms each) and 31 s under qemu on H-02 (at
  most 0.73 s each, most of it qemu and runtime start-up).

### Dead ends and small things

- First link without libuv and libunwind: undefined `uv_*` (the runtime
  starts a libuv loop at initialization) and `_Unwind_*` (libc++abi).
- First AArch64 link used TC-01's clang resource directory: "cannot open
  crtbeginS.o" and a missing `aarch64-unknown-linux-gnu/libclang_rt.builtins.a`;
  `-resource-dir {toolchain_aarch64}/lib/clang/22` takes TC-11's.
- `leanir` (in `bin/`) emits C from `.ir` files for the module system's
  postponed compilation; it does not apply to these non-`module` files, so
  the C comes from `lean -c`.
- For the orchestrator: the adapter has no `surface`, so M-11's default roles
  do not count `Checker/*.lean` (trusted glue); `ds06.json` could list the C
  files (`.lake/build/ir/D006/*.c`, `.lake/build/ir/Checker/*.c`) as
  deterministic artifacts.

### DS-05 follow-up (orchestrator)

The prover build now emits the C of Core, Records and Lrat in the same `lean`
invocation that writes each `.olean` (`-o ... -c ...`), instead of elaborating
the three modules a second time; only `Checker/Main.lean` has its own `-c`
step. The serial build went from about 31.5 s to 18.9 s in the dev loop, and
DS-01 to DS-04 positives and DS-05 (`--fresh`, both hosts) still print
`0 failure(s)`. DS-06 lists the emitted C as deterministic artifacts.
