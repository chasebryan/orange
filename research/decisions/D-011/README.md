# D-011 native target laboratory

Status: contributor-produced, unreviewed; `d011-v0.1` suite and laboratory
written, one measured epoch run and exported, no owner input, no target
envelope selected

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
  running a compiler or emulator, and verifies every committed export.
- [`d011-v0.1/run/d011-e-291c7d1e803c802bcfd0/`](d011-v0.1/run/d011-e-291c7d1e803c802bcfd0/)
  is the export of the measured epoch: its `epoch.json`, `packet.json` and
  `summary.json`, its 1,083 records as gzip-compressed JSON lines, and a
  manifest.

Archives are written outside the repository. An export keeps an archive's
epoch, packet, summary and records and binds the rest by digest: `index.json`
is rebuilt from the record lines, and the 33 driver ELFs under `products/` are
named by SHA-256 only. `verify` rebuilds the
archive manifest from those parts, requires it to hash to the digest the
export names, and then recomputes the summary from the records byte for byte.

## Using the laboratory

The laboratory needs a Linux host that allows unprivileged user namespaces,
with the tools the suite lists (section 9 of the suite). Build `orangec` from
the base revision into `compiler/target/release/orangec` first, then:

```sh
python3 tools/d011_suite.py check
python3 tools/d011_suite.py run --profile dev
python3 tools/d011_suite.py verify d011-e-<id>
python3 tools/d011_suite.py export d011-e-<id>
```

`export` writes `d011-v0.1/run/d011-e-<id>/` from a verified archive. On a
clone without the archive, `verify d011-e-<id>` checks the export instead.

Epochs are written under `/tmp/orange-d011`. For the evidence run the owner
saves a completed copy of `python3 tools/d011_suite.py owner-template` under
`/tmp/orange-d011/owner-input/` and runs
`python3 tools/d011_suite.py run --profile measured --owner-input NAME`. The
owner input is written by the owner only; the laboratory records its digest
and does not verify who wrote it. Native runs on owner hardware use the driver
ELFs and request files the epoch writes under `products/`.

## Measured epoch

Epoch `d011-e-291c7d1e803c802bcfd0` ran the `measured` profile on 2026-09-30
from 09:32 UTC, at repository commit
`0d084ae9dd5dc7ff1cae6811576804aa939fb55b` (a commit of the pull request that
added this laboratory) with a clean working tree. `orangec` and the Orange
oracles come from the base revision
`59caa344e176bc6d8f4e5429a00f3d82eeb2799a`. The host was a contributor
machine (Ubuntu 24.04.4, Linux 6.18, x86-64, 4 CPUs with AES, PCLMULQDQ and
AVX2), not owner hardware. The run took 8 minutes 32 seconds and wrote 1,083
records. `verify` passed on the 6.5 MB archive and passes on its 225 KB
export. Packet SHA-256:
`57df5286ff134f0e5b710a7c8e6f121d8d80296b334c87e1f5dac2a02da9cb9d`.

The profile built `-O2`, `-O3` and `-Os` three times each, timed 30 native
runs after one warm-up and five emulated runs, sent the request file 20 times
per timed run, and traced eight variants per group. No owner input was
supplied, so the epoch concludes `inconclusive: no valid owner input`.

All 64 subjects had agreeing oracles: the published literal, `orangec eval`
of the 15 bound Orange sources, and the library oracle.

| Case | Gate | T-X64 | T-A64 | T-RV64 | Portable path |
| --- | --- | --- | --- | --- | --- |
| NT-01 known answers | HG-02 | pass: 1644/1644, native and emulated identical 12/12 | pass: 822/822 emulated | pass: 411/411 emulated | pass |
| NT-02 inventory | HG-04 | pass: class A 0, B 168, C 0, D 26, E 3; control 12/12 | pass: A 0, B 163, C 0, D 26, E 4; control 12/12 | pass: A 0, B 137, C 0, D 61, E 6; control 6/6 | vacuous |
| NT-03 traces | HG-05 | pass: 180/180 groups, control differs 12/12 | pass: 180/180, control 12/12 | pass: 90/90, control 6/6 | vacuous |
| NT-04 negatives | HG-03 | pass: 132/132 | unresolved: 126/132, N-12 has no emulated CPU without FEAT_AES in the six crypto-profile builds | pass: 66/66 | pass |
| NT-05 C ABI | HG-06 | pass: cross-links 2/2, Rust probe 128/128, symbol violations 0 | unresolved: cross-links 2/2, no Rust `aarch64` standard library | unresolved: one C toolchain, no Rust `riscv64gc` standard library | pass |
| NT-06 inventory rows | HG-07 | unresolved: 0/3 verified | unresolved: 0/3 | unresolved: 0/3 | vacuous |
| NT-07 owner hardware | HG-08 | unresolved: no owner input | unresolved | unresolved | vacuous |
| NT-08 resources | HG-09, HG-01 | pass, pass: 36/36 builds | pass, pass: 36/36 builds | pass, unresolved: 18/24 builds, TC-03 absent | pass, pass |

Every build rebuilt to identical bytes. Resource figures (contributor host;
build cost is one repetition of all three optimization levels, the other
figures are for the `-O2` reference build):

| Figure | T-X64 | T-A64 | T-RV64 |
| --- | --- | --- | --- |
| Build CPU / wall, ms | 17154 / 17593 | 19416 / 19960 | 8390 / 8624 |
| Kernel code bytes, all families | 15086 | 12020 | 20404 |
| Crypto-profile glue / target bytes | 2106 / 108 | 1940 / 116 | 3096 / 758 |
| Target-specific source lines | 52 | 53 | 91 |
| Distinct kernel mnemonics | 79 | 77 | 56 |
| Emulated known-answer median, ms (launch overhead) | 339 (15) | 333 (14) | 253 (14) |
| Native known-answer median, ms (launch overhead) | 81 (9) | none | none |
| Trace blocks, reference build | 1360719 | 1407283 | 177987 |
| Trace wall time, ms | 100409 | 102453 | 43126 |
| CI projection, ms | 123578 | 126699 | 53264 |

The portable path's CI projection is 3243 ms. The emulated and native columns
are different quantities and are not compared.

| Candidate | Gates not passing | Eligible | AX-01 | AX-02 | AX-03 | AX-04 ms | AX-05 | AX-06 | AX-07 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TE-01 | HG-03, HG-06, HG-07, HG-08 unresolved | no | 2 | 17 | 52 | 253520 | 251 | 6 | 2 |
| TE-02 | HG-07, HG-08 unresolved | no | 1 | 12 | 26 | 126821 | 131 | 3 | 1 |
| TE-03 | HG-03, HG-06, HG-07, HG-08 unresolved | no | 1 | 10 | 26 | 129942 | 130 | 6 | 1 |
| TE-04 | none (HG-04, HG-05, HG-07, HG-08 vacuous) | yes | 0 | 5 | 0 | 3243 | 0 | 3 | 0 |
| TE-05 | HG-01, HG-03, HG-06, HG-07, HG-08 unresolved | no | 3 | 22 | 113 | 306784 | 390 | 9 | 3 |

No gate failed. Every non-passing gate is unresolved for one of three
reasons: owner input the contributor cannot supply (HG-07, HG-08), a tool not
installed on the host (HG-01 for T-RV64, HG-06 for T-A64 and T-RV64), or an
emulator limit (HG-03 for T-A64). TE-04 is eligible only through vacuous
passes, as the suite's anti-gaming rule 5 describes; the summary lists it as
eligible and still concludes `inconclusive`, because the conclusion needs the
owner's input and review scopes before any candidate is recommended.

Every gate state matches the two earlier epochs below; the measured profile
changed counts and costs, not outcomes.

### Tools recorded by the measured epoch

| Tool | Version | SHA-256 |
| --- | --- | --- |
| `x86_64-linux-gnu-gcc-13` | 13.3.0-6ubuntu2~24.04.1 | `1b99826121ae6682a634e5efe09bd3e3df58ce58e0b28f849114ab5b89139c26` |
| its `cc1` | 13.3.0 | `5d1679131184e2de4435b426eb264bf13472fe026db8e5c6bc97445814e8e2f4` |
| `x86_64-linux-gnu-as` | binutils 2.42 | `21aff249b692b5c31a44007491f922dcb49f41323e362c57d2ada3f52eddb7f0` |
| `x86_64-linux-gnu-ld.bfd` | binutils 2.42 | `e9ceb054c12207970f2726dfc07e9a66b411602748628baf27399f02a9bbb31b` |
| `aarch64-linux-gnu-gcc-13` | 13.3.0-6ubuntu2~24.04.1 | `cd90adc7801f4595267f61a5d25bd3a0c6beb2f9f1f107ab919a97a12972dc9a` |
| its `cc1` | 13.3.0 | `a7f69616cf8ddbf5c52a5b61bc4c11f504ed9421e282e9083f0a7a22a3268dad` |
| `aarch64-linux-gnu-as` | binutils 2.42 | `a315904827ba6f80ad0b58ac3a4672f56a76e639cec2d6833f80b475a28445a3` |
| `aarch64-linux-gnu-ld.bfd` | binutils 2.42 | `7c903ac277dd1f5c4397277db865f12d8239fe45782f71afbeb3d42180a4b1ae` |
| `riscv64-linux-gnu-gcc-13` | not installed | none |
| `clang` | 18.1.3 (1ubuntu1) | `8ef402d453d1ba4902e4ee0f0f847f6cfa01400c95aa43c24e97818b9c0e3f45` |
| `lld` | 18.1.3 | `7ad9a0e8fe6d0e79b71172d731e33872c0274e49fceb7b516d774876d5a58ade` |
| `llvm-objdump` | 18.1.3 | `4f98b86448d23bd1f858c50e93fbfc799f1c3640961a95e3b6c89d229a1b91bb` |
| `llvm-readelf` (`llvm-readobj`) | 18.1.3 | `8ed942a8c33f191480253ff7f236b7e49966c7441d12063d49dce9743aba9a6d` |
| `qemu-x86_64-static` | 8.2.2+ds-0ubuntu1.18 | `90181b5552cd1a0b60d8edcc9f1cf8e38647579ef697d3155e2b6150e8fbcee0` |
| `qemu-aarch64-static` | 8.2.2+ds-0ubuntu1.18 | `e4f8d99e9ff69c3cefffab71cee358ce2af1ecba1282d04c3eeb44ef76f5a71e` |
| `qemu-riscv64-static` | 8.2.2+ds-0ubuntu1.18 | `be89e201dc6112ef101617ab575c95119ad123f80ca2b51b6c36ef218ac9df65` |
| `rustc` | 1.96.1 (31fca3adb 2026-06-26) | `4a84e05991ad6f2a84c1361c29b52b38c390365bd1fc269b1936c88d482a8928` |
| `python3.11` | 3.11.15 | `f56a588548dd013906ae1dcd1b6faa417f4e204da634ff354840d9643e78ff9e` |
| `libcrypto.so.3` | OpenSSL 3.0.13 | `d6fc1bc9de29c55fc905f77edba1ccc7c7a50b32bd2bb9086b0d0b00104eafc4` |
| `orangec` | 0.0.1, built from the base revision | `9f11217e0f81c9902f4626e7fc4e98b8e0f2aae32560ec21178c8aa3e9e02631` |
| `fs-sandbox` | built from `tools/fs_sandbox.c` | `8c763dd60dafc99d4f54e87c2c59dc02ecb808391b9a6709a9f12368d101df53` |
| `unshare` | util-linux 2.39.3 | `51bcc77ba5db162c80028f861f0a2770d728c1de80773816d863f28d7a817adb` |
| `setpriv` | util-linux 2.39.3 | `96b083b79c32fd2f0c29657e88e20c7495839349fc64ad5d0503f32d26bf8733` |

`rustc` links the probe with the system `cc`, which is the same
`x86_64-linux-gnu-gcc-13` binary. The two earlier epochs recorded the same
tools and digests.

## Earlier epochs

An earlier measured epoch, `d011-e-59b28dba6bd33a57cecb`, ran at 08:34 UTC
from commit `d517505` with the previous packet
(`a34d8e461e5964db4fbce80ec07338b4076555524e71b4a46f31478b86a9ce86`). Review
then moved the tool version probes inside the sandbox and bound the owner's
feature-negative and native-run attestations to a driver and a device, which
changed the packet, so the epoch was run again. It reached the same gate
states, counts and conclusion with different timings, and its archive is kept
outside the repository and not exported.

### Development epoch

Before the measured runs, epoch `d011-e-221362817d5023f337a1` ran the `dev`
profile on the same host (one `-O2` build per toolchain, a batch of two per
timed run). It took 77 seconds, wrote 413 records and verified, and it
concludes `inconclusive` by rule because a development epoch is not a result.
It ran from the laboratory's draft commit before it was rebased onto `main`,
so that commit is not in this repository's history; its packet is the
previous one (`a34d8e46…`). Its archive is kept outside the repository and
not exported.

## What a conclusive epoch still needs

- Acquisitions: `gcc-13-riscv64-linux-gnu` (TC-03), and the Rust standard
  libraries for `aarch64-unknown-linux-gnu` and `riscv64gc-unknown-linux-gnu`.
  Each is a dependency the owner admits under D-018 before it is used. Without
  them HG-01 stays unresolved for T-RV64 and HG-06 for T-A64 and T-RV64.
- Owner input: a verdict on each of the 14 inventory rows (9 required), the
  owner's own devices per tuple, a native run of an archived driver on each,
  a SIGILL from each AArch64 crypto-profile driver on a named device without
  the Cryptographic Extension (gap G-09),
  every review scope NR-01 to NR-10, a distinguishing rule and, for DR-1, a
  solo slice capacity.
- A new `measured` epoch with that input (`--owner-input NAME`). The run
  reads the input and records it among the epoch's records, so it cannot be
  added to this epoch afterwards. On the contributor host the measured profile
  took about nine minutes.

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
  an archived driver run natively on that device, compared with the emulated
  output, and a run naming a device the owner did not attest makes the owner
  input invalid. An N-12 attestation settles only the driver it names.
- The launcher maps the current user (`--map-current-user`) rather than root
  inside the namespace, and runs the unchanged `fs-sandbox` with its own caps.
- Orange oracles are bound by path and SHA-256 at the base revision and
  evaluated with `orangec eval`, rather than copied into this directory.
- AES-192 and GCM IVs other than 96 bits are outside the stand-in kernel
  (gaps G-04 and G-05); N-07 checks that a 24-byte key is refused.
- Timing sends the request file repeated in one process and times an empty
  request beside it, because sandbox launch alone costs about 10 to 15
  milliseconds on the development host.
- Epochs live under the fixed root `/tmp/orange-d011` and arguments only name
  entries that already exist there (or, for `verify`, an export under
  `d011-v0.1/run/`), so no command-line argument becomes a path or a command
  element.
- The repository keeps an export rather than the archive. The driver ELFs are
  not committed, because a later epoch rebuilds them to identical bytes and
  the owner's native runs need an epoch with owner input anyway; the export
  names each one by SHA-256.
