# Findings while building the D-006 v0.3 candidates

Facts a reviewer of the epoch should know, recorded as they were found.
Contributor-produced and unreviewed.

- Lean 4 reserves a 1 GiB stack per thread by default, which the sandbox's
  4 GiB address-space cap refuses ("failed to create thread"). Every `lean`
  step passes `--tstack=65536`, and compiled Lean programs (`leanchecker`, the
  DS-05 checker) get `LEAN_STACK_SIZE_KB=65536`.
- Host reads: Rocq's `coqc` reads `/etc/ocamlfind.conf` (declared as a host
  file); Lean reads `/dev/urandom` (in every step's read-only set).
- Artifact truncation (D2-M05): Rocq's `coqchk` rejects a truncated `.vo` with
  "premature end of file". Lean's `lean` and `leanchecker` map an `.olean`
  without bounds checks and die with SIGSEGV and no message on a truncated
  one, so the Lean candidate adds `Loader/OleanCheck.lean`, a structural check
  of the file followed by a kernel replay, as its declared artifact checker.
  That loader is trusted code in the Lean check path and is listed in its
  trust inventory.
- Reproducibility seen in the dev loop: Rocq `.vo` files are byte-identical
  across build paths; Lean `.olean` files are byte-identical across
  `--threads=1` and `--threads=4` and across build directories.
- Lean's `omega` on goals with conjunctions or disjunctions, and `x == x` on
  derived `BEq`, bring in `Classical.choice`; the Lean candidate avoids both so
  its audits show only `propext` and `Quot.sound`.
- DS-04 checker cost (Rocq, seen while building DS-05): `Lrat.step` ends each
  line with Stdlib `rev`, which is quadratic in the line's length. A
  20,000-token line takes 4 s and a 200,000-token line over 120 s, in the
  extracted checker and under `vm_compute` alike, where the reference answers
  at once. The shared corpus has no such line, so no case observes it.
- DS-05 stack depth (Rocq): extracted list functions are not tail-recursive,
  and a 300 KB records file overflowed the stack in bytecode. The candidate
  proves `decode_prefix` (decoding the first 4097 bytes gives the whole
  input's verdict) and decodes only that prefix.
- DS-05 stack depth (Lean): `D006.Lrat.lines` uses one stack frame per
  certificate line, and a certificate within the 4 MiB limit can have about
  4 million lines. The worst case needs 192 to 256 MiB of stack on x86-64 and
  256 to 384 MiB under AArch64 emulation, so the Lean checker runs with
  `LEAN_STACK_SIZE_KB=524288`; the runtime's own reservations leave the
  process at about 2.8 GiB of its 4 GiB address-space cap.
- DS-05 closures: the Rocq checker is native OCaml on H-01 and OCaml bytecode
  on the pinned AArch64 `ocamlrun` for H-02 (243,834 bytes of bytecode plus
  the runtime); the Lean checker is compiled C, 2.95 MB stripped on each host,
  statically linking the Lean runtime, GMP, libuv and libc++ and loading only
  glibc.
- Lean address space under parallel builds (found by the epoch's first
  attempt): `lean --threads=4 --tstack=65536` on `D006/Sieve.lean` peaks at
  4.16 to 4.28 GiB of address space, over the 4 GiB cap, because glibc
  reserves a 64 MiB malloc arena for most of its 14 to 16 threads. A failed
  arena reservation is survivable, but a failed thread stack is not, so the
  build aborted only when a thread start crossed the cap: in one of the four
  parallel builds, the second workspace's ("failed to create thread: Resource
  temporarily unavailable"). The correction sets
  `MALLOC_ARENA_MAX=2` for every Lean step, which brings the build steps to
  2.91 to 3.30 GiB (Sieve, Records, Lrat and the checker, measured outside the
  sandbox with `--threads=4`). The sandbox caps are unchanged.
