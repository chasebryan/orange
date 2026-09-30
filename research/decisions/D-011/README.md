# D-011 native target laboratory

Status: contributor-produced, unreviewed; `d011-v0.1` suite and laboratory
written, no measured epoch, no target envelope selected

This directory holds the inputs for the comparison of initial native target
envelopes defined by
[`NATIVE_TARGET_DECISION_SUITE.md`](../../../docs/NATIVE_TARGET_DECISION_SUITE.md).
The suite never picks; the choice belongs to the owner.

The native code here is contributor-written portable C standing in for code
Orange does not yet generate. It is not Orange output. The laboratory measures
target feasibility, not Orange code generation, and it never compares emulated
timings with native ones.

## Files

- [`d011-v0.1/suite-packet.json`](d011-v0.1/suite-packet.json) is the
  canonical packet: candidates, tuples, profiles, toolchains, build flags,
  cases, metrics, gates, axes, rules, review scopes, run profiles, the 64
  subjects with their oracles, 13 negatives, 17 trace groups, the ISA and ABI
  inventory, named gaps, the empty owner-input template, and the SHA-256 of
  every bound input.
- [`d011-v0.1/kernels/`](d011-v0.1/kernels/) holds the five stand-in files:
  `kernels.c`, `accel.c`, `runtime.c`, `d011_kernels.h` and `abi_probe.rs`.
- [`tools/d011_suite.py`](../../../tools/d011_suite.py) generates and checks
  the packet and runs the laboratory. It uses only the Python standard library.
- [`tools/tests/test_d011_suite.py`](../../../tools/tests/test_d011_suite.py)
  tests the packet, the oracles and the laboratory's pure logic without
  running a compiler or emulator.

Archives are written outside the repository.

## Using the laboratory

The laboratory needs a Linux host that allows unprivileged user namespaces,
with the tools the suite lists (section 9 of the suite). Build `orangec` from
the base revision into `compiler/target/release/orangec` first, then:

```sh
python3 tools/d011_suite.py check
python3 tools/d011_suite.py run --profile dev
python3 tools/d011_suite.py verify d011-e-<id>
```

Epochs are written under `/tmp/orange-d011`. For the evidence run the owner
saves a completed copy of `python3 tools/d011_suite.py owner-template` under
`/tmp/orange-d011/owner-input/` and runs
`python3 tools/d011_suite.py run --profile measured --owner-input NAME`. The
owner input is written by the owner only; the laboratory records its digest
and does not verify who wrote it. Native runs on owner hardware use the driver
ELFs and request files the epoch writes under `products/`.

## Development epoch

Not yet run at this commit.

## What the measured run still needs

- Acquisitions: `gcc-13-riscv64-linux-gnu` (TC-03), and the Rust standard
  libraries for `aarch64-unknown-linux-gnu` and `riscv64gc-unknown-linux-gnu`.
  Each is a dependency the owner admits under D-018 before it is used. Without
  them HG-01 stays unresolved for T-RV64 and HG-06 for T-A64 and T-RV64.
- Owner input: a verdict on each of the 14 inventory rows (9 required), the
  owner's own devices per tuple, a native run of an archived driver on each,
  a SIGILL on an AArch64 device without the Cryptographic Extension (gap G-09),
  every review scope NR-01 to NR-10, a distinguishing rule and, for DR-1, a
  solo slice capacity.
- The `measured` profile itself: `-O2`, `-O3` and `-Os`, three build
  repetitions, 30 native runs after one warm-up, five emulated runs, a batch of
  20 per timed run, and eight trace variants per group.

## Judgment calls

- Consolidated kernels. The stand-in kernels are five files, with one section
  per kernel family selected by a `D011_PART_*` macro, so each family is still
  its own object while the repository gains few files.
- Crypto flags narrowed to the instructions used: AES-NI and PCLMUL on x86-64,
  `+aes` (AES and PMULL) on AArch64, and Zkne with Zbkc on RV64.
- TE-05 kept as briefed, with RV64GC as its third tuple.
- The portable path is shared by every candidate. HG-04, HG-05, HG-07 and
  HG-08 are vacuous for it, so TE-04 passes them vacuously and is eligible
  whenever the portable path is. The summary labels those passes vacuous and
  AX-02 does not count them.
- HG-08 needs more than a named device: the owner records the output digest of
  an archived driver run natively on it, compared with the emulated output.
- The launcher maps the current user (`--map-current-user`) rather than root
  inside the namespace, and runs the unchanged `fs-sandbox` with its own caps.
- Orange oracles are bound by path and SHA-256 at the base revision and
  evaluated with `orangec eval`, rather than copied into this directory.
- AES-192 and GCM IVs other than 96 bits are outside the stand-in kernel
  (gaps G-04 and G-05); N-07 checks that a 24-byte key is refused.
- Timing sends the request file repeated in one process and times an empty
  request beside it, because sandbox launch alone costs about 10 to 14
  milliseconds on the development host.
