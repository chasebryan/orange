# C-01 Rocq candidate: implementation log

## DS-01 Core fragment (2026-09-29, first case built; about 40 minutes)

- `Word w` is a record of a binary natural and an `Is_true (val <? 2 ^ w)`
  bound. `truth` turns a boolean equation into that bound by matching on the
  boolean only, so a closed word's bound computes to `I` and closed
  observations hold by `vm_compute; reflexivity`. Equality of words follows
  from equality of values because `Is_true b` has at most one proof
  (`Is_true_unique`); no axiom is needed.
- `Seq A n` is a list with the same kind of boolean length bound.
- `word_rotl` is `if w =? 0 then x else ...` so the width-0 identity is
  explicit (N's `r mod 0` is `r`). D1-TH06 is proved bit by bit
  (`rotl_bits`, then `N.bits_inj`) with the two cases of `(r mod w + s mod w)`.
- Byte conversion uses Horner form, `((a*256+b)*256+c)*256+d`, and repeated
  division by 256, which makes both round trips a chain of `N.div_mod`
  rewrites; `lia` alone did not close the four-division form.
- M-01 is a module functor `QuarterRound (P : QRParams)`; F-18 and F-19 are
  its two instances. The harness re-instantiates it with fresh parameter
  modules and checks the instances by `reflexivity`.
- The exhaustive method is `exhaustive_word`: `exhaustive_eq` reduces
  `forall x : Word w, L x = R x` to a boolean enumeration of all 2^w values
  (`all_below`, split in halves), evaluated by `vm_compute`. On 2^32 values it
  runs into the 120-second negative ceiling, as D1-N07 expects.
- `Print Assumptions` is empty for every DS-01 theorem and observation.

## DS-03 canonical records, OCR1 (2026-09-29; about 30 minutes)

- `theories/Records.v`, fragment `adapter.d/ds03.json`. `Record` (the name
  is a legal Rocq identifier), `ErrorCode` and `Failure` follow the shared
  signature exactly. The failure path is a `PrimString.string` built with
  `PrimString.cat` from literals and a decimal numeral (`decimal`, fuel
  `S (N.size_nat n)`, since a number has at most as many decimal digits as
  bits).
- The decoder is a chain of readers `bytes -> Result (A * bytes) Failure`
  joined by a local `let?` notation over `bind`. Every recursion is
  structural: `utf8_valid` matches up to four bytes deep, `split_at` on the
  byte count, `read_refs` on the reference count and `read_records` on the
  record count (at most 64). Each record is read against the records before
  it (`earlier`), so its index, the previous name and a claim's target come
  from that list. Check order and paths match the reference exactly
  (unknown tag before the name, UTF-8 before duplicate/order, escape before
  cyclic before order/kind).
- `records_valid` is the conjunction of the stated conditions: at most 64
  records, `forallb name_ok` (1 to 200 bytes, UTF-8), names `increasing`
  under `bytes_lt`, `bodies_ok` (each record's body checked against the
  records before it: 32-byte digest/fingerprint, at most 64 strictly
  increasing references below its index, a claim reference below its index
  naming a theorem, level at most 3), and `size (encode_records l) <= 4096`.
  The decoder is not used.
- Proofs: every reader has an `_ok` lemma (what it consumed is the encoding
  of what it returned, plus the checks it made) and an `_app` lemma (it reads
  back an encoding whose checks hold). `prefix_ok` (names and bodies of the
  records so far) is the loop invariant; `prefix_ok_snoc` and
  `prefix_ok_app_l` connect it to the stepwise checks. `decode_ok` gives
  D3-TH02, D3-TH03 and D3-TH04 at once; D3-TH05 follows from D3-TH02;
  D3-TH01 is `read_records_app` from the empty prefix.
- Dead ends: `injection` on `ok (x, rest) = ok (y, rest')` normalizes the
  components (it unfolded `128 * val b1` into a match), which then no longer
  matched `N.ltb_spec`; `ok_pair` keeps them as they are. `lia` alone does
  not know `N.modulo`/`N.div` by constants (zify maps them to `Z.rem`/
  `Z.quot`), so `arith` adds `Z.quot_rem_to_equations` and
  `Z.div_mod_to_equations`.
- Harness issue H-01 (list literals with a bare-symbol element type rendered
  `(@cons @X ...)`, which Rocq does not parse) was reported and fixed in the
  renderer; see `HARNESS_ISSUES.md`.
- Trust: every DS-03 audit reports `PrimString.string` (the type of a failure
  path, in every statement mentioning `Failure`) and `PrimString.cat` (used by
  the decoder); both are listed in `trust`. The observations on
  `records_valid` and `utf8_valid` are closed under the global context.
- Beyond the 44 observations, a throwaway differential test (not in the
  candidate) checked 1500 random and mutated inputs, record lists and byte
  strings against `tools/d006_shared.py` by `vm_compute; reflexivity`, plus
  targeted cases (64 records, multi-digit paths, the 4096-byte limit).
- Observation check time: about 5 s for all 44 (the 4101-byte D3-O19
  included).

## DS-02 Sieve (2026-09-29; about 90 minutes)

- `theories/Sieve.v`, fragment `adapter.d/ds02.json`, patches
  `patches/D2-M01.patch`, `D2-M02.patch`, `D2-M06.patch`. Sieve.v imports
  only Core and Stdlib; its build step lands between Core and Records.
- Names: expression, command and observation constructors carry a
  CompCert-style prefix (`Elit`, `Eeq`, `Cset`, `Cseq`, `Oread`, ...)
  because the shared names `eq`, `set`, `seq`, `get`, `add`, `read`, `write`
  would shadow `Logic.eq`, `List.seq` and friends inside the module. Every
  other constructor keeps its shared name. `VarDecl`, `ArrDecl`, `Env` and
  `State` are records whose constructors are the shared `vdecl`, `adecl`,
  `env`, `state` (projections `vty`/`vlabel`, `asize`/`alabel`,
  `vars`/`arrs`, `values`/`arrays`).
- Names of variables and arrays are `N` (T-02). `lookup l i` is
  `nth_error l (N.to_nat i)`, and `put` is a structural update that leaves a
  list without that position unchanged; both are `simpl never` so lemma
  equations rewrite stably.
- The semantics follows the reference's order of checks exactly, including
  the corner cases: a word operator whose left operand is any value and whose
  right operand fails gives `fail (t1 ++ t2)`, not stuck; `set` checks the
  index is a word before evaluating the value and checks the array exists only
  after the value; `seq skip c2` matches on `c1` (the literal reading of
  semantics.md), with `wrap` for a step of `c1`; `loop` matches `N0`/`Npos`
  and uses `N.pred`. `run` is structural in a `nat` fuel (`N.to_nat n`);
  `run_trace`/`run_outcome` are its two projections.
- `wf` is two `Forall2`s; `low_eq` quantifies over declared public names by
  `lookup`; `outcome_low_eq` is by cases. Words are compared through their
  values (`word_ext` from Core), so `val_eq_dec` needs no axiom.
- Proofs: `eval_typed` (a typed expression gives a value of its type or
  fails) and `eval_agree_typed` (in low-equivalent states it makes the same
  observations, with the same value when public). D2-TH01 and D2-TH02 are
  separate inductions on the command; D2-TH03 is an induction on the command
  after `change (lockstep g (step s1 c) (step s2 c))`, with small
  introduction lemmas for the four disjuncts; D2-TH04 is an induction on the
  fuel from D2-TH03 and D2-TH02; D2-TH05 is D2-TH04 plus `p_plus_well_typed`;
  D2-TH06 is `repeat constructor` for `wf`, a case split on the name for
  `low_eq` and `reflexivity` for the two traces. Sieve.v uses kernel
  conversion only (no `vm_compute`); the VM is used only by the declared
  observation method.
- Mutations: each patch's first build error lands in a theorem the mutation
  makes false: D2-M01 in D2-TH03 (the cond case asserts a public condition),
  D2-M02 in D2-TH02 (the assign case), D2-M06 in D2-TH03. D2-M05 truncates
  `theories/Sieve.vo` and runs `coqchk -silent -Q theories D006 -norec
  {artifact}` (coqchk accepts a `.vo` path; the intact file checks with exit
  0). Rocq prints `Fatal Error: User error: .../Sieve.vo: premature end of
  file. Try to rebuild it.`; the fragment adds an artifact-phase rule
  `premature end of file` -> `parse_failure`, since the base rule matched it
  only through its generic `Error` alternative.
- Dead end: the first version proved progress and preservation together in
  one lemma whose cond case destructed the condition's label. D2-M01 then
  broke that lemma (and so D2-TH01/D2-TH02, which stay true under D2-M01)
  before reaching D2-TH03, and D2-M02 took progress down with preservation.
  Now the progress and preservation cond cases accept any label, and only
  D2-TH03 asserts `public`.
- Trust: every DS-02 audit (6 parity, 27 observations) is `Closed under the
  global context`; no trust additions.
- Beyond the 27 observations, a throwaway differential test (scratch only,
  not in the candidate) compared `step`, `run_trace`, `run_outcome` (fuel 0 to
  11) and `well_typed` with `tools/d006_shared.py` on 2900 random programs of
  depth 3 with random or well-formed states and random environments (11,600
  equations, rendered through the harness and proved by `vm_compute;
  reflexivity`); all agree. Every result kind occurs (next, done, fail,
  stuck; about a fifth of the programs well typed).
- Times: Sieve.v builds in about 1.5 s; the DS-02 dev loop takes about 18 s.
- For the orchestrator (`adapter.d/ds06.json`, not edited here): D006.Sieve
  is a new compiled module; it needs a DS-02 re-check entry (`coqchk -silent
  -Q theories D006 -norec D006.Sieve`) and a place in the DS-06 re-check list.

## DS-04 LRAT certificate check (2026-09-29/30; about 2 hours of active work over three sessions cut by usage-limit pauses)

- `theories/Lrat.v` (imports Core only), fragment `adapter.d/ds04.json`; build
  step after Records. No negatives exist for DS-04 in the shared JSON; the
  dev loop checks the 5 parities and 14 observations.
- Meaning: `holds o` is `forall x y : Word 32, eval_bv (lhs o) x y =
  eval_bv (rhs o) x y` over a small term language `BV` (x, y, add, xor, and,
  shl) evaluated with the DS-01 word operations, so D4-TH02/TH03 are
  `iff_refl` (the rendered statements are the unfolded meaning).
- Bit-blast: a state-passing blaster (`mk`, `bitwise`, `adder`, `bits`,
  `or_chain`, `miter`, `blast`) that follows the numbered rules literally:
  variable 1 false, inputs 2..65, gates from 66 in creation order, left
  before right, shift-in and initial carry literal 1. `cnf_clauses` is
  `[-1]`, the gates' Tseitin clauses in creation order, and the output unit;
  `cnf_text` prints them (PrimString `make`/`cat`, balanced concatenation).
  D4-CNF01/02 check the text byte for byte (512 variables, 1535 clauses).
- Checker: `lrat_verdict o f p` compares `f` with `cnf_text o`
  (`cnf_mismatch`), refuses `length p > 4 MiB` (`oversized`), then runs a
  byte machine over the primitive string: one byte per `step`, driven by
  `drive 23` (fuel doubling; at most 2^22 + 1 steps are ever needed, so the
  exhausted-fuel branch is unreachable). Tokens are classified while read
  (canonical decimals, `-`, `d`); any syntax problem in a line, and a last
  line without its line feed, is `parse` at that line, which is exactly the
  reference's order because `trailing` is decided as soon as the empty
  clause is derived (any further byte belongs to the next line). A complete
  line goes to `line_step`: deletion (`del_targets`, then `delete_all`,
  `unknown_deletion`), or addition (`all_nums`, `split_zero`, `hint_list`
  for syntax; then `id_order`, `var_range`, `lemma_form` via `falsify`, which
  refuses any variable seen twice, `rat_unsupported`, then `rup`). Active
  clauses are a `PositiveMap` keyed by clause id; the RUP assignment is a
  `PositiveMap bool` keyed by variable (the reference's literal set is always
  consistent, so the two agree). Numbers are binary `N`/`Z`, so huge ids,
  literals and hints behave as in Python.
- Soundness (D4-TH04): the only invariant is `entails cs db` (every
  assignment satisfying the blasted clauses satisfies every active clause);
  the parser needs no proof. `rup_sound` is the usual unit-propagation
  argument (`true_clause_open`), `falsify_agrees` sets up the negated lemma,
  deletions shrink the map, `drive_sound` lifts `step_sound`. Accept is
  reachable only through an empty lemma whose propagation succeeded, so
  `check_certificate_sound` gives `unsat (cnf_clauses o)`.
- Blast correctness: `unsat (cnf_clauses o) -> holds o`. For words that break
  the equation, `v := value gates (inputs x y)` evaluates every gate from the
  gates before it; `value_ok` gives every gate equation, so every Tseitin
  clause holds (`gate_clauses_true`), and `blast_spec` (from `bits_spec`,
  `adder_spec`, `bitwise_spec`, `miter_spec`, all relative to "every gate of
  the final state holds", with `extends` for monotonicity) says the output
  variable is "some bit differs". The word-level facts are `wbits_add`
  (ripple carry: `add_bits_spec` with an explicit carry, then
  `N.mod_unique`), `wbits_xor`/`wbits_and` (`to_bits_zip`) and `wbits_shl`
  (`nth_ext`). Two structural facts about the two concrete circuits (gates
  numbered above the inputs, strictly decreasing, each above its operands)
  are checked by `vm_compute` in `blast_wf` rather than proved generically;
  they say nothing about the bit-vector identity.
- D4-TH01 is `golden_certificate_accepted` (`lrat_verdict carry_save
  (cnf_text carry_save) golden_certificate = accept` by `vm_compute`), then
  D4-TH04 and D4-TH02. The golden certificate is embedded in Lrat.v as a
  string literal (61850 bytes, SHA-256 9b75f94c...1276, checked when it was
  written); Rocq cannot read a file at compile time without a plugin.
  D4-TH05 evaluates the counterexample x = y = 1 (2 versus 1).
- Trust: every DS-04 audit lists the primitive string type and `make`,
  `length`, `get`, `compare`, `cat`, and `PrimInt63.int`, `add`, `eqb`,
  `ltb`, `leb`; the new ones are in the fragment's `trust`. Only the
  bytecode VM (already declared) evaluates; no `native_compute`. Integer
  literals need PrimInt63's notations imported, so that import is confined to
  a module `Chars` and audits print qualified names.
- Measured (this host, 4 GiB address-space and 600 s CPU limits applied):
  Lrat.v builds in 1.7 to 3.5 s at 427 MiB peak RSS; the DS-04 check file
  (5.1 MB of source, with the 4.26 MB V-08 literal) checks in 5.8 s at
  670 MiB; `coqchk -norec D006.Lrat` takes 5 s at 242 MiB, all four modules
  39 s at 1.15 GiB. A fully processed certificate of 4194302 bytes (padding
  deletions, then the golden proof) is accepted in 1.6 s by `vm_compute`
  plus 1.7 s at `Qed`, 603 MiB. Lrat.vo is byte-identical across rebuilds.
- Differential test (scratch only, not in the candidate): 423 verdicts
  against `tools/d006_shared.py`, namely V-00 to V-11 (V-08 aside) of the
  golden certificate and of a fresh CaDiCaL 3.0.1 certificate
  (`--lrat --no-binary --seed=0`, accepted), 400 random byte/token/line
  mutations, and the 4 MiB case above; all agree (every rejection code
  occurs).
- Dead ends: `simpl` on goals holding `wbits` of a word expands 32 symbolic
  bits through `add_bits`/`carry_bits` and never finishes (`simpl N.b2n in S`
  hung the build); the proofs use `cbn [...]` or `change` instead.
  `Arguments mk : simpl never` did not stop `simpl in H` from reducing
  `let (g, s) := mk ...`, so the blaster lemmas use `cbn [bitwise]` and
  friends; `cbn [bits]` still unrolled `firstn 32` in the shift case, which
  `change` restores.
- For DS-05: `cnf_text` and `lrat_verdict` use PrimString and PrimInt63;
  extraction must map them (the kernel's `Pstring`/`Uint63` OCaml modules).
- For the orchestrator (`adapter.d/ds06.json`, not edited here): D006.Lrat
  is a new compiled module; it needs a DS-04 re-check entry (`coqchk -silent
  -Q theories D006 -norec D006.Lrat`) and a place in the DS-06 list.

## DS-05 standalone checker (2026-09-30; about 30 minutes of wall time)

- Files: `extraction/Checker.v` (in-prover entry points), `extraction/Extract.v`
  (extraction), `extraction/uint63.ml` (copied runtime subset),
  `extraction/driver.ml` (command line), fragment `adapter.d/ds05.json`.
  Nothing under `theories/` changed.
- Entry points, all `PrimString.string` in and out, so the driver passes
  file bytes and prints results unchanged: `records_line` (section 4's
  verdict line of `decode_records` of the input's bytes), DS-04's
  `cnf_text`, and `lrat_line` (`lrat_verdict` as `accept` or
  `reject CODE LINE`). `bytes_of` turns a primitive string into
  `list (Word 8)` with `PrimString.get` and a 256-leaf search tree over
  primitive integers built once (`all_bytes`, 8 comparisons a byte, shared
  leaves), driven by a doubling `scan` (depth 24 > `max_length`, so no
  unary fuel and a shallow stack). Hex and decimal fields reuse
  `Records.decimal`/`Lrat.decimal`.
- `decode_prefix` (proved, no axioms; `Print Assumptions` lists only
  `PrimString.string`/`cat`): `decode_records (firstn 4097 l) =
  decode_records l`. `records_line` decodes that prefix, because the
  extracted `length`/`N.of_nat` behind `decode_records`' size test are not
  tail recursive: before it, a records file of 300 KB (bytecode) or 1 MB
  (native) died with `Stack_overflow` instead of `reject oversized input`.
  Now 16 MB takes 2.9 s natively and 75 s under qemu; above
  `PrimString.max_length` (16777211 bytes) the driver refuses the file.
- Extraction: `Extract.v` uses only the mappings Rocq ships,
  `ExtrOCamlPString` and `ExtrOCamlInt63` (PrimString to the kernel's
  `Pstring`, PrimInt63 to the kernel's `Uint63`, plus Rocq `bool`/`prod` to
  OCaml's). No hand-written `Extract` directive; N, Z, positive, lists and
  `PositiveMap` stay Rocq's own. It writes `extraction/extracted.ml(i)`
  during the prover build (`build.serial`, after Lrat), so `sources`
  (`extraction/*.v`, `*.ml`, `*.mli`) covers the generated code when the
  runner takes its manifest.
- Primitive runtime: the host builds copy the kernel's own
  `lib/rocq-runtime/kernel/pstring.ml(i)` from the toolchain and compile it
  unchanged. The kernel's `uint63.ml` cannot be used whole: its `to_float`
  is an external whose C stub is in the Rocq VM's `libcoqrun_stubs.a`
  (inside `rocq_interp.o`, x86-64 only), which the AArch64 `ocamlrun` does
  not have. `extraction/uint63.ml` is the subset used (`of_int`, `zero`,
  `l_and`, `add`, `lt`, `le`, `to_int_min`, `equal`), each definition copied
  unchanged from the kernel's file; it asserts a 64-bit word.
- Driver (59 lines, trusted glue): argument parsing (`B-C01`/`B-C02` to the
  extracted constructors), whole-file reads through `Pstring.of_string`,
  printing. Exit 0 with a verdict, 1 for an unreadable or too-long input,
  2 for usage (on standard error), 3 when the computation raises (stack or
  memory).
- Hosts: H-01 `/usr/bin/ocamlopt` (TC-06, no findlib, so no
  `/etc/ocamlfind.conf` read) to `extraction/checker`, 1455896 bytes
  (1050368 stripped), loading only libm and libc; runtime `[]`. H-02
  `/usr/bin/ocamlc` to `extraction/checker.byte`, 243834 bytes of bytecode
  (no ELF, so no stripped size), run as `qemu-aarch64-static -L
  /usr/aarch64-linux-gnu {toolchain_aarch64}/bin/ocamlrun {binary}`; runtime
  `[{toolchain_aarch64}/bin/ocamlrun]` (its digest matches TC-10's). strace
  shows nothing else outside /usr is read (ocamlrun only stats a missing
  `lib/ocaml/ld.conf`). Both binaries are byte-identical when built in two
  different directories.
- `dependency`: `lib/rocq-runtime/kernel/pstring.ml`. Without it, the first
  H-01 step prints `/usr/bin/cp: cannot stat '.../pstring.ml': No such file
  or directory`; the build-phase rule classifies that as
  `unsupported_feature` (the primitive-string runtime is missing).
  Simulated as the runner does it: `unsupported_feature`.
- Checked: the DS-05 dev loop with `--fresh` passes on both hosts (58 items
  each, with usage); DS-03 and DS-04 positives still pass. Also ran the
  whole build and both checkers as uid 60606 under `tools/fs_sandbox.c`
  (Landlock, the epoch's read/write roots, the checker seeing only binary,
  runtime and corpus): all 44 golden-corpus items pass on both hosts.
  Times: native 2 to 22 ms an item; qemu 50 ms (records) to 0.9 s
  (golden certificate).
- Dead ends: `Extraction "extraction/x.ml"` warns that the output directory
  defaults to the working directory; `Set Extraction Output Directory`
  fixes it. Patterns on `Result` need the local `Arguments ok {A E}` as in
  Records.v.
- Open issues (not fixed here):
  - Harness: the launcher runs every step through `/usr/bin/env -i`, whose
    execvp retries an ENOEXEC binary with `/bin/sh`. The D5-N01 launch (the
    AArch64 `ocamlrun` without qemu) then exits 2 with `ocamlrun: 1: Syntax
    error: "&" unexpected`, which neither exit 126/127 nor `NO_TARGET`
    matches, so the epoch would give D5-N01 no category. This affects any
    AArch64 binary launched this way, Lean's included.
  - DS-04: `Lrat.step` ends a line with `rev (t :: tokens m)`, and Stdlib
    `rev` is quadratic. The extracted checker (and `vm_compute`) needs 4 s
    for one 20,000-token line and over 120 s for 200,000 tokens; the
    reference answers `reject lemma_form 1` instantly. `rev_append` would be
    linear. Very long lines would also run deep in the extracted
    non-tail-recursive list functions. The corpus has neither.
  - For the orchestrator: `extraction/Checker.vo`, `Extract.vo` and
    `extracted.ml(i)` are deterministic build outputs (DS-06 artifacts
    list). M-11's default roles do not count `extraction/`: `driver.ml` and
    `uint63.ml` are trusted glue, `Checker.v`/`Extract.v` in-prover
    sources.
