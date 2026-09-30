# D-011 native target decision suite

Status: contributor-produced, unreviewed draft; no target envelope selected

Suite version: `d011-v0.1`

Snapshot: 2026-09-30

Base revision: `59caa344e176bc6d8f4e5429a00f3d82eeb2799a`

## What this suite measures, and what it does not

The native code in this suite is contributor-written portable C that stands in
for code Orange does not yet generate. It is not Orange output, and nothing
here measures Orange code generation. The laboratory measures target
feasibility: whether each candidate target tuple can build, run, check,
inventory and trace a flagship-shaped corpus with the tools that exist today,
and what that costs a solo owner.

Emulated timings are never compared with native ones. The laboratory runs every
tuple under QEMU user-mode emulation and runs natively only on its own host's
tuple. Emulated time is reported as a CI-cost proxy for the same tuple, never
as a performance figure, and never set beside a native figure.

The suite makes the D-011 choice measurable and runs the evidence. It never
picks. The choice belongs to the owner, Chase Bryan. Every record the
laboratory writes is labeled `contributor-produced, unreviewed`. The suite
writes no freeze, owner approval or owner review in the owner's name, and a
contributor host is never evidence of owner-accessible hardware.

The machine-readable form of this suite is the packet
[`suite-packet.json`](../research/decisions/D-011/d011-v0.1/suite-packet.json),
generated and checked by [`tools/d011_suite.py`](../tools/d011_suite.py).
The packet binds this document and the five stand-in kernel files by SHA-256.
Laboratory usage and the development run are described in the
[D-011 research README](../research/decisions/D-011/README.md).

## Solo-mode disposition

Under D-023 the owner may run and review every part of this suite, and each
such review is `solo-reviewed`. Independent review is unavailable; metric M-21
records that and is never scored. Under the
[solo development envelope](GATE0_SUPPORT_ENVELOPES.md), invariant S-05 admits
one bounded target slice at a time, so metric M-22 and axis AX-07 count the
slices a candidate needs rather than pretending they can be staffed in
parallel.

## 1. Decision boundary

This suite supplies the comparison asked for by
[D-011](DECISIONS.md#d-011--initial-native-target-envelope). It asks which
claim-bearing native target tuples Orange 1.0 should commit to, each with a
baseline and a crypto-extension feature profile, beside a portable C path that
every candidate keeps as an interoperability output.

It does not decide the compiler strategy (D-010), the leakage model (D-012),
the stable foreign boundary (D-013), the flagship corpus membership (D-015) or
the host-tool platforms. It borrows bounded stand-ins for each: C kernels for
Orange output, an emulated control-flow trace for a leakage observation, a
minimal C ABI with a Rust caller for the foreign boundary, and a corpus shaped
like the D-015 recommendation. None of those stand-ins accepts or predetermines
the decision it stands in for. The laboratory is Linux-only; host tools for
macOS and Windows are outside what it measures.

The four acceptance-evidence items D-011 names map onto the cases:

| Acceptance evidence | Cases |
| --- | --- |
| Resource estimate per target | NT-08 |
| ISA/ABI model availability | NT-05, NT-06 |
| Owner-accessible hardware evidence | NT-07 |
| Flagship-corpus feasibility | NT-01, NT-02, NT-03, NT-04 |

## 2. Candidates and parity

The candidate set is symmetric. Each candidate is the portable path plus a set
of native tuples, and each receives the same cases, subjects, gates and budgets.

| ID | Candidate | Native tuples | Why it is in the set |
| --- | --- | --- | --- |
| TE-01 | x86-64 and AArch64 | T-X64, T-A64 | The envelope the decision register currently recommends. Being the current recommendation carries no weight here. |
| TE-02 | x86-64 only | T-X64 | The most common server and desktop tuple; one claim-bearing native target. |
| TE-03 | AArch64 only | T-A64 | The most common mobile and a growing server tuple; one claim-bearing native target. |
| TE-04 | Portable C interoperability only | none | No claim-bearing native target at 1.0: Orange emits portable C behind the C ABI, the user's compiler owns machine code, and every native constant-time claim is unsupported. |
| TE-05 | x86-64, AArch64 and RV64GC | T-X64, T-A64, T-RV64 | TE-01 plus RV64GC, the base of the RVA20 and RVA22 Linux profiles. Kept as briefed: RV64GC is an open ISA with a public golden reference model (`sail-riscv`) and a ratified data-independent-timing extension (Zkt), as recorded from knowledge and not checked from this machine. |

The candidates are nested: TE-02 and TE-03 sit inside TE-01, which sits inside
TE-05, and TE-04 is the empty envelope. A candidate's result is computed from
its members only, so no candidate can borrow evidence from a tuple it does not
include, and dropping a tuple after seeing its evidence is a different
candidate, not a repair.

### Target tuples

| ID | Tuple | ABI | Baseline ISA (P-BASE) | Crypto profile (P-CRYPTO) | C toolchains | Data-independent-timing mechanism |
| --- | --- | --- | --- | --- | --- | --- |
| T-X64 | `x86_64-unknown-linux-gnu` | System V AMD64 psABI, LP64 | x86-64 baseline (`-march=x86-64`) | plus AES-NI and PCLMULQDQ | TC-01, TC-04 | Intel DOITM; AMD publishes separate guidance |
| T-A64 | `aarch64-unknown-linux-gnu` | AAPCS64, LP64 | Armv8-A (`-march=armv8-a`) | plus FEAT_AES and FEAT_PMULL (`+aes`) | TC-02, TC-04 | FEAT_DIT (PSTATE.DIT, Armv8.4-A) |
| T-RV64 | `riscv64-unknown-linux-gnu` | RISC-V ELF psABI, LP64D | RV64GC (`-march=rv64gc`) | plus Zkne and Zbkc | TC-03, TC-04 | Zkt |

All three are ELF64 little-endian Linux user-mode tuples. The laboratory
emulates each with the matching `qemu-*-static` binary at CPU model `max`, and
runs natively only on its own host's tuple (T-X64 on the development host).

### The portable path

`C-PORTABLE` is the same kernels built by every available C toolchain for the
laboratory host's own tuple and called from Rust through `extern "C"` by the
ABI probe. Every candidate includes it, so its gate results are identical for
all five. It makes no machine-code claim: the no-division, trace, inventory and
owner-hardware gates (HG-04, HG-05, HG-07, HG-08) are vacuous for it and are
recorded as vacuous passes, never as evidence.

### Feature profiles

- `P-BASE`: the tuple's baseline ISA, portable kernels only. The accelerated
  operations answer `unsupported`.
- `P-CRYPTO`: the baseline plus the tuple's AES and carry-less multiply
  extension (`-DD011_CRYPTO_PROFILE`). It adds accelerated AES and AES-GCM
  operations whose GHASH uses a different algorithm from the portable kernel,
  so agreement between the two is not one algorithm agreeing with itself.

## 3. The stand-in kernels and the flagship-shaped corpus

The five kernel files under `research/decisions/D-011/d011-v0.1/kernels/` are
contributor-written and are bound by digest in the packet:

- `kernels.c` holds the portable kernels in seven sections: SHA-2, HMAC and
  HKDF, ChaCha20 and Poly1305, X25519, AES and AES-GCM, Keccak (SHA3-256 and
  SHAKE128), and constant-time helpers. Each object is compiled from one
  section with one `D011_PART_*` macro, so each kernel family is one object.
- `accel.c` holds the crypto-profile GHASH glue and one target section per
  architecture: AES-NI and PCLMUL intrinsics, the Arm AES and PMULL
  intrinsics, and RISC-V `aes64es`, `aes64esm`, `clmul` and `clmulh` through
  inline assembly.
- `runtime.c` holds per-target system calls and `_start`, `memcpy` and its
  relatives, a request driver and two detector controls.
- `d011_kernels.h` is the C ABI the kernels expose.
- `abi_probe.rs` is the Rust caller that exercises that ABI.

Every build is freestanding and static, so the only code in a driver is code
in these files plus whatever the compiler inserts. The kernel objects may
import nothing beyond `memcpy`, `memmove`, `memset`, `memcmp` and `d011_`
symbols.

The driver reads length-prefixed requests on standard input and writes one
status and output per request. Statuses are `ok`, `rejected`, `unsupported` and
`malformed`; input cut short ends the process with exit status 2 after the
complete responses.

### Subjects and oracles

The corpus has 64 known-answer subjects in twelve groups. A subject counts only
when every oracle available for it agrees on the expected bytes (metric M-02):

1. the published value carried as a literal in an Orange source under
   `algorithms/` or `compiler/fixtures/` at the base revision;
2. the Orange reference evaluation of the matching computation, run with
   `orangec eval` built from the base revision; and
3. a library oracle: Python `hashlib` and `hmac`, OpenSSL 3.0.13 `libcrypto`
   through `ctypes`, or a small Python reference where no library call fits.

| Group | Subjects | Orange oracle source | Library oracle | Published source |
| --- | --- | --- | --- | --- |
| K-SHA256 | 5 | `algorithms/sha2/sha2.or`, `compiler/fixtures/s3d/valid-sha256-rounds.or` | `hashlib`, Python reference | FIPS 180-4 examples; Botan `sha2_32.vec`; FIPS 180-4 worked example |
| K-SHA512 | 3 | `algorithms/sha2/sha2.or` | `hashlib` | FIPS 180-4 examples; a value `algorithms/sha2` recorded |
| K-HMAC | 7 | `algorithms/hmac-hkdf/hmac-hkdf.or` | `hmac` | RFC 4231 cases 1, 2, 3, 4, 6, 7; Wycheproof tcId 171 |
| K-HKDF | 3 | `algorithms/hmac-hkdf/hmac-hkdf-rfc5869.or` | `hmac` composed as RFC 5869 | RFC 5869 A.1 to A.3 |
| K-CHACHA | 10 | `algorithms/chacha20/chacha20.or` | OpenSSL ChaCha20 | RFC 8439 2.3.2, 2.4.2, A.1, A.2 |
| K-POLY | 14 | `algorithms/chacha20-poly1305/chacha20-poly1305.or` | OpenSSL Poly1305 and ChaCha20 | RFC 8439 2.5.2, 2.6.2, A.3, A.4 |
| K-AEAD | 4 | `algorithms/chacha20-poly1305/chacha20-poly1305.or` | OpenSSL ChaCha20-Poly1305 | RFC 8439 2.8.2, A.5; Wycheproof tcId 71 |
| K-X25519 | 4 | the four `algorithms/x25519/` sources | OpenSSL X25519 | RFC 7748 5.2 and 6.1; Wycheproof tcId 1 |
| K-AES | 3 | `algorithms/aes/aes.or` | OpenSSL AES-ECB | FIPS 197 B, C.1, C.3 |
| K-GCM | 6 | the three `algorithms/aes-gcm/` sources for 96-bit IVs | OpenSSL AES-GCM | GCM specification test cases 1, 2, 4, 13, 16 |
| K-SHA3 | 3 | `algorithms/sha3/sha3.or` | `hashlib` | Botan `sha3.vec`; values `algorithms/sha3` recorded |
| K-SHAKE | 2 | `algorithms/sha3/sha3.or` | `hashlib` | values `algorithms/sha3` recorded |

K-SHA256-05 exercises the round function alone through a probe operation. Its
expected value is the FIPS 180-4 worked example carried by
`compiler/crates/orangec/tests/s3d_conformance.rs`, checked against a Python
reference of section 6.2.2. The K-AES and K-GCM subjects also run through the
accelerated operations in the crypto profile. The Orange sources are bound by
path and SHA-256 at the base revision rather than copied, so the packet adds no
duplicate of `algorithms/`.

## 4. Required decision cases

Each case runs for every tuple of every candidate and for the portable path.

### NT-01: Known-answer agreement

Acceptance evidence: flagship-corpus feasibility.

Every subject, on every build, returns the expected status and bytes:
emulated for every tuple, and natively as well where the laboratory host is the
tuple. The native and emulated responses of the host tuple must be identical.
Precondition: the subject's oracles agree.

Metrics M-01, M-02, M-03. Gate HG-02.

### NT-02: Static instruction inventory

Acceptance evidence: flagship-corpus feasibility.

`llvm-objdump` disassembles every kernel and crypto-profile object, and each
instruction is classified:

| Class | Instructions | Role |
| --- | --- | --- |
| A | division and square root | Gate: must be absent from kernel code, because latency depends on operand values on common implementations |
| B | conditional branches | Informational; whether a branch depends on a secret is NT-03's question |
| C | indirect jumps and calls | Informational; on RV64 an `auipc` and `jalr` pair is a direct call and is not counted |
| D | multiplications | Informational; data-independent latency is promised only under DOITM, FEAT_DIT or Zkt |
| E | crypto-extension instructions | Informational; present only in crypto-profile objects |

The division control in the runtime must show class A on every build, so the
detector is proven on each build rather than assumed.

Metrics M-05 to M-09, M-11. Gate HG-04.

### NT-03: Control-flow trace equivalence

Acceptance evidence: flagship-corpus feasibility.

For each of 16 kernel trace groups, the laboratory derives several secret
variants of one public shape, runs each under QEMU with `-d nochain,exec`, and
hashes the sequence of guest program counters of the translated blocks. Every
variant of a group must give one hash. The first-difference control, an
early-exit comparison, must give different hashes, so the observer is proven
able to see a secret-dependent branch on each build.

This is emulated control flow only. It sees no cache, memory-address, power or
timing channel, and it is not a constant-time claim.

Metrics M-10, M-11. Gate HG-05.

### NT-04: Negative and fail-closed behaviour

Acceptance evidence: flagship-corpus feasibility.

| ID | Check | Expected |
| --- | --- | --- |
| N-01 | ChaCha20-Poly1305 open with a changed tag byte | `rejected`, no plaintext |
| N-02 | ChaCha20-Poly1305 open with a flipped ciphertext bit | `rejected`, no plaintext |
| N-03 | AES-GCM open with a flipped tag bit | `rejected`, no plaintext |
| N-04 | AES-GCM open with changed additional data | `rejected`, no plaintext |
| N-05 | Accelerated AES-GCM open with a flipped tag bit | `rejected`, no plaintext |
| N-06 | SHA-256 probe with a 63-byte block | `malformed` |
| N-07 | AES with a 24-byte key | `malformed` |
| N-08 | HKDF asking for more than 255 blocks | `malformed` |
| N-09 | Unknown operation | `unsupported` |
| N-10 | Accelerated AES in the baseline profile | `unsupported` |
| N-11 | A request whose argument is cut short | exit status 2 after the complete responses |
| N-12 | Crypto-profile binary on a CPU model without the extension | SIGILL, no response |
| N-13 | One flipped bit in the linked SHA-256 round constants | the SHA-256 known answers fail |

N-01 also has an Orange oracle: `algorithms/chacha20-poly1305` evaluates the
same tampered verification to `false`. The unit tests check that the library
oracle rejects N-01 to N-05 as well. N-12 runs under QEMU with CPU model
`qemu64` for T-X64 and `rv64` for T-RV64. QEMU 8.2 has no AArch64 CPU model
without FEAT_AES, so for T-A64 the check stays unresolved until the owner
records a SIGILL on a device without the extension (gap G-09).

Metric M-04. Gate HG-03.

### NT-05: C ABI agreement

Acceptance evidence: ISA/ABI model availability.

Kernel objects from one C toolchain are linked with the runtime from the other
and must agree with the known answers. The Rust ABI probe calls the kernels
through `extern "C"` and must agree. Kernel objects may import nothing outside
the allowed set, and relocation types are recorded for NT-08.

Metrics M-12, M-13, M-14. Gate HG-06.

### NT-06: ISA and ABI model inventory

Acceptance evidence: ISA/ABI model availability.

The packet lists, per tuple, the ISA reference, the psABI, the formal or
machine-readable model and the data-independent-timing mechanism. The
contributor recorded every row from knowledge; none was checked from the
laboratory host, which has no web access. A row counts only when the owner
verifies it in the owner input. Section 9 lists the rows.

Metric M-15. Gate HG-07.

### NT-07: Owner-accessible hardware

Acceptance evidence: owner-accessible hardware evidence.

The laboratory records its own host as a contributor machine. The owner input
names, per tuple, the devices the owner can run natively, and records the
SHA-256 of the standard output of an archived driver run natively on one of
them with the archived request file. The laboratory compares that digest with
the emulated output of the same driver. Absent owner input leaves the gate
unresolved; a declared absence of hardware or a disagreeing run fails it.

Metric M-16. Gate HG-08.

### NT-08: Resource estimate

Acceptance evidence: resource estimate per target.

Per tuple the laboratory records:

- the toolchains needed and the acquisitions still missing;
- compile and link CPU and wall time for one repetition;
- whether every repeated build is bit-for-bit identical;
- executable bytes per kernel family and per crypto-profile object;
- target-specific source lines, distinct kernel mnemonics and relocation types;
- known-answer wall time, native only where the host is the tuple and emulated
  otherwise, each with its own launch-overhead baseline and never compared;
- trace sizes and trace wall time;
- a CI-time projection on the laboratory host; and
- the bounded target slices the tuple needs under E-SOLO.

Metrics M-17 to M-22. Gates HG-09 and HG-01.

## 5. Metrics

| ID | Metric | Definition |
| --- | --- | --- |
| M-01 | Known-answer agreement | subjects matching / subjects run, per build and execution mode |
| M-02 | Oracle agreement | subjects whose available oracles all agree / subjects |
| M-03 | Native and emulated agreement | host-tuple builds whose native and emulated responses are identical |
| M-04 | Negative checks | negative checks passing / negative checks applicable, per build |
| M-05 | Class A in kernel code | division and square-root instructions in kernel objects |
| M-06 | Class B in kernel code | conditional branches (informational) |
| M-07 | Class C in kernel code | indirect jumps and calls (informational) |
| M-08 | Class D in kernel code | multiplications (informational; DIT assumption) |
| M-09 | Class E | crypto-extension instructions in crypto-profile objects (informational) |
| M-10 | Trace equivalence | trace groups with one distinct trace / trace groups run |
| M-11 | Detector controls | division control found by the inventory; first-difference control traces differ |
| M-12 | Cross-toolchain agreement | cross-linked builds agreeing / cross-linked builds |
| M-13 | Rust probe agreement | probe subjects matching / probe subjects run |
| M-14 | Symbol surface | undefined symbols in kernel objects outside the allowed set |
| M-15 | Inventory verification | required inventory rows the owner verified / required rows |
| M-16 | Owner-attested devices | devices per tuple in the owner input, and matching native runs |
| M-17 | Build cost | compile and link CPU and wall milliseconds per build |
| M-18 | Kernel code bytes | executable bytes per kernel family and per crypto-profile object |
| M-19 | Known-answer wall time | native and emulated medians, reported separately and never compared |
| M-20 | Target surface | target-specific source lines, distinct mnemonics, relocation types |
| M-21 | Independent review | unavailable under E-SOLO; recorded, never scored |
| M-22 | Solo slices | bounded target slices needed (E-SOLO S-05: one at a time) |

## 6. Hard gates and anti-gaming rules

A gate has one of four states: `pass`, `fail`, `unresolved` or `unsupported`.
States combine by the precedence `unsupported > fail > unresolved > pass`. A
tuple's gate is the worst of its builds, and a candidate's gate is the worst of
the portable path and its member tuples. A candidate whose entries for a gate
are all vacuous records a vacuous pass.

| ID | Gate | Case | Rule |
| --- | --- | --- | --- |
| HG-01 | Matrix completeness | NT-08 | Every planned build of the tuple compiled, linked and ran every case. A build failure fails; an absent toolchain (an acquisition) leaves it unresolved. |
| HG-02 | Known answers | NT-01 | Every subject matches on every build and execution mode. A mismatch with agreeing oracles fails; an oracle conflict leaves the subject unresolved. |
| HG-03 | Fail-closed | NT-04 | Every applicable negative check passes. A check the laboratory cannot perform on the tuple stays unresolved until the owner attests the SIGILL on a device without the extension. |
| HG-04 | No division in kernel code | NT-02 | No class A instruction in any kernel or crypto-profile object, and the division control is detected on every build (otherwise unresolved). |
| HG-05 | Trace equivalence | NT-03 | Every kernel trace group has one distinct trace on every build, and the first-difference control differs (otherwise unresolved). |
| HG-06 | C ABI agreement | NT-05 | Cross-toolchain links and the Rust probe agree and the symbol surface holds. A missing second toolchain or Rust standard library leaves it unresolved. |
| HG-07 | ISA and ABI inventory verified | NT-06 | The owner verified every required inventory row of the tuple; a rejected row fails; otherwise unresolved. |
| HG-08 | Owner hardware | NT-07 | The owner attests a device for the tuple and records a native run of an archived driver whose output digest equals the emulated one. A declared absence or a disagreeing run fails; otherwise unresolved. |
| HG-09 | Resource estimate | NT-08 | Every resource field is present and every build rebuilt bit for bit; a nondeterministic build fails. |

Anti-gaming rules:

1. The packet fixes the subjects, negatives, trace groups, build flags, run
   profiles and budgets before any run. The laboratory refuses to start when the
   committed packet differs from the generator or a bound input's digest has
   changed.
2. A subject whose oracles disagree is unresolved, never quietly dropped, and
   the disagreement is archived.
3. The division and trace detectors have a positive control on every build. A
   detector whose control fails leaves its gate unresolved rather than passing
   it. The flipped-constant check N-13 is the known-answer check's own control.
4. An absent tool is an acquisition that leaves a gate unresolved. It is never
   a pass, and never a fail that could eliminate a candidate cheaply.
5. A vacuous pass is labeled vacuous and is not counted on AX-02. TE-04 passes
   the machine-code gates vacuously, so it is eligible whenever the portable
   path is. When no native candidate closes its gates, the procedure therefore
   reaches TE-04; that reads as "no native candidate closed its evidence", not
   as evidence of TE-04's merits.
6. Emulated and native times are separate fields with separate launch-overhead
   baselines and are never compared, subtracted or ranked against each other.
7. Axes are reported, never summed or weighted. The owner chooses a
   distinguishing rule and records it in the owner input; the laboratory only
   applies it.
8. A development-profile epoch is never a result.

## 7. Comparative axes

The candidates are nested, so gates cannot separate two candidates that both
pass. The axes describe what each candidate costs and covers.

| ID | Axis | Measure | Better | Material difference |
| --- | --- | --- | --- | --- |
| AX-01 | Claim coverage | claim-bearing native tuples | higher | any |
| AX-02 | Evidence closure | non-vacuous gate evaluations passing | higher | any |
| AX-03 | Leakage-risk surface | class D instructions in kernel code needing a DIT assumption | lower | at least 10 percent |
| AX-04 | Build and test cost | projected CI milliseconds per run on the laboratory host | lower | at least 20 percent |
| AX-05 | ISA and ABI surface | distinct kernel mnemonics plus target-specific source lines | lower | at least 10 percent |
| AX-06 | Toolchain dependencies | distinct toolchains needed plus missing acquisitions | lower | any |
| AX-07 | Solo capacity | bounded target slices needed under E-SOLO S-05 | lower | any |

## 8. Laboratory

### Isolation

Every compile, link, disassembly, emulated run, native run and evaluation runs
under the same launcher, with no network:

```text
/usr/bin/unshare --user --map-current-user --mount --ipc --uts --pid --fork
    --kill-child=KILL --mount-proc --net
/usr/bin/setpriv --bounding-set=-all --inh-caps=-all --ambient-caps=-all
    --no-new-privs
/bin/sh -c '"$@"; exit $?' d011-init
/usr/bin/env -i <fixed environment>
fs-sandbox --dir / --ro <inputs> --rw <outputs> --rw /dev/null -- <command>
```

`fs-sandbox` is built for each epoch from the unchanged `tools/fs_sandbox.c`.
It closes inherited descriptors and keeps its own caps: 4 GiB of address
space, 600 CPU seconds, 512 MiB per file and 256 processes. The laboratory adds
a wall-clock limit per step and records each step's exit, resource use and
output digests.

### Builds

Each build is one tuple, one C toolchain, one profile and one optimization
level. Every kernel section, crypto-profile section and runtime section is
compiled to its own object with the common flags

```text
-std=c11 -ffreestanding -fno-builtin -fno-stack-protector
-fno-asynchronous-unwind-tables -fno-unwind-tables -fPIE -Wall -Wextra -I.
```

plus `-fno-tree-loop-distribute-patterns` for GNU C, the tuple's target and
profile flags, and `-DD011_PART_<section>`. GNU C links with `-nostdlib -static
-no-pie`; Clang links with `-nostdlib -static -fuse-ld=lld`. RV64 builds add
`-mno-relax` and `-Wl,--no-relax`. Each build is repeated, and the repetitions
must be identical byte for byte.

### Run profiles

| Profile | Optimizations | Build repetitions | Native runs | Emulated runs | Batch repeats | Trace variants | Purpose |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `dev` | `-O2` | 2 | 1, no warm-up | 1 | 2 | 3 | exercise every stage once; not a result |
| `measured` | `-O2`, `-O3`, `-Os` | 3 | 30 after 1 warm-up | 5 | 20 | 8 | the evidence run the owner reviews |

A timed run sends the whole known-answer request file, repeated by the batch
count, in one process. A run with an empty request is timed beside it as the
launch-overhead baseline and reported separately.

## 9. Tools and ISA/ABI models

### Tools

The laboratory records the path, version string and SHA-256 of every tool in
each epoch's `tools` record. The development host provided:

| ID | Tool | Version | Role | Present |
| --- | --- | --- | --- | --- |
| TC-01 | GNU C for x86-64 with binutils | gcc-13 13.3.0-6ubuntu2~24.04.1, binutils 2.42-4ubuntu2.10 | C compiler and linker for T-X64 | yes |
| TC-02 | GNU C cross for AArch64 with binutils | gcc-13-aarch64-linux-gnu 13.3.0-6ubuntu2~24.04.1cross1, binutils 2.42-4ubuntu2.10 | C compiler and linker for T-A64 | yes |
| TC-03 | GNU C cross for RV64 | gcc-13-riscv64-linux-gnu | C compiler and linker for T-RV64 | no (acquisition) |
| TC-04 | Clang and LLD | 18.1.3 (1:18.1.3-1ubuntu1) | C compiler and linker for every tuple | yes |
| TC-05 | llvm-objdump, llvm-readelf | 18.1.3 | instruction inventory and relocations | yes |
| TC-06 | QEMU user-mode emulators | qemu-user-static 1:8.2.2+ds-0ubuntu1.18 | emulated execution and traces | yes |
| TC-07 | rustc | 1.96.1, `x86_64-unknown-linux-gnu` standard library only | Rust ABI probe | yes; other standard libraries are acquisitions |
| TC-08 | Python | 3 (`hashlib`, `hmac`, `ctypes`) | laboratory runner and library oracle | yes |
| TC-09 | OpenSSL libcrypto | 3.0.13 (libssl3t64 3.0.13-0ubuntu3.7) | library oracle | yes |
| TC-10 | orangec | 0.0.1 built from the base revision | Orange reference-evaluation oracle | yes |
| TC-11 | fs-sandbox | built per epoch from `tools/fs_sandbox.c` | filesystem sandbox | yes |
| TC-12 | util-linux `unshare`, `setpriv` | 2.39.3 (2.39.3-9ubuntu6.5) | namespaces and privilege drop | yes |

These are distribution packages and a rustup toolchain on a contributor
machine. Any tool used for the owner's measured run still needs the owner's
dependency admission under D-018; this suite records none.

### ISA and ABI model inventory

Every row below was recorded from the contributor's knowledge and was not
checked from this machine. The status of each is
`contributor_recorded_unverified` until the owner verifies or rejects it.

| Row | Tuple | Kind | Required | Source |
| --- | --- | --- | --- | --- |
| IA-X64-01 | T-X64 | ISA reference | yes | Intel 64 and IA-32 Architectures Software Developer's Manual (document 325462) |
| IA-X64-02 | T-X64 | ISA reference | no | AMD64 Architecture Programmer's Manual (publication 24592 and following) |
| IA-X64-03 | T-X64 | ABI | yes | System V ABI, AMD64 Architecture Processor Supplement (x86-64 psABI) |
| IA-X64-04 | T-X64 | Formal model | no | No vendor formal model; community models cover subsets, for example the ACL2 x86 ISA model and Sail x86 work |
| IA-X64-05 | T-X64 | DIT mechanism | yes | Intel Data Operand Independent Timing Mode (DOITM) and its instruction list |
| IA-A64-01 | T-A64 | ISA reference | yes | Arm Architecture Reference Manual for A-profile architecture (DDI 0487) |
| IA-A64-02 | T-A64 | ABI | yes | AAPCS64 and ELF for the Arm 64-bit Architecture (Arm `abi-aa`) |
| IA-A64-03 | T-A64 | Formal model | no | Arm's machine-readable A-profile specification (ASL) and its Sail translation |
| IA-A64-04 | T-A64 | DIT mechanism | yes | FEAT_DIT: PSTATE.DIT and the instructions it covers (DDI 0487) |
| IA-RV-01 | T-RV64 | ISA reference | yes | The RISC-V Instruction Set Manual, Volumes I and II |
| IA-RV-02 | T-RV64 | ABI | yes | RISC-V ELF psABI (LP64D) |
| IA-RV-03 | T-RV64 | Formal model | no | `sail-riscv`, the RISC-V golden reference model |
| IA-RV-04 | T-RV64 | DIT mechanism | yes | Zkt, data-independent execution latency (scalar cryptography v1.0) |
| IA-RV-05 | T-RV64 | Profile | no | RVA20 and RVA22 profiles (RV64GC base) |

## 10. Evidence packet and archive layout

The packet `d011-v0.1/suite-packet.json` is canonical JSON: sorted keys, no
spaces, integers only and one final line feed. `tools/d011_suite.py generate`
writes it, and `tools/d011_suite.py check` confirms that the committed bytes
equal the generator's output and that every bound input still has its digest.

`tools/d011_suite.py run --profile dev|measured` writes one epoch directory
under the fixed root `/tmp/orange-d011`, named `d011-e-` plus the first 20 hex
digits of the SHA-256 of the epoch identity (suite version, packet digest,
profile, base revision, repository head, tools, host and start time):

```text
d011-e-<id>/
  epoch.json               identity and the stand-in statement
  packet.json              the packet the epoch ran
  records/<sha256>.json    one content-addressed record per step
  products/                driver ELFs and request files for owner native runs
  index.json               stage, key and digest of every record, in order
  summary.json             per-tuple gates and metrics, candidates, conclusion
  archive-manifest.json    SHA-256 of every file in the epoch
```

`tools/d011_suite.py verify EPOCH` rechecks every file against the manifest,
rejects any file the manifest or the record index does not list, checks that
every record belongs to the epoch, and recomputes `summary.json` from the
records alone. Archives stay outside the repository.

Command-line arguments never become filesystem paths or command elements. An
epoch or an owner input is named by an entry that already exists under the
root, working files live under `/tmp/orange-d011/work`, and `orangec` is read
from `compiler/target/release/orangec` in the repository. Only a step that
completes (exit status 0, no limit reached) counts as a success; an expected
failure, such as N-11's exit status or N-12's SIGILL, is matched on that exit
code or signal and never on a limit. Records and summaries carry no floats and
no integers beyond 2^53 - 1, and the summary is computed only from the
records, never by querying the host.

## 11. Owner input and review scopes

`tools/d011_suite.py owner-template` prints an empty owner input. Only the
owner completes it, saves it under `/tmp/orange-d011/owner-input/` and names it
with `run --owner-input NAME`; the laboratory records its digest and content
and does not verify who wrote it. It holds the attested devices and declared absences per
tuple, native runs of archived drivers, feature-negative attestations, a
verdict on every inventory row, the review scopes done, the distinguishing
rule, and the owner's solo slice capacity.

| Scope | Covers |
| --- | --- |
| NR-01 | Custody and candidate parity: the packet, the candidate set and its symmetry |
| NR-02 | Flagship corpus and oracles: subjects, oracle sources, named gaps |
| NR-03 | x86-64 evidence: T-X64 builds, inventory, traces, negatives |
| NR-04 | AArch64 evidence: T-A64 builds, inventory, traces, negatives |
| NR-05 | RV64GC evidence: T-RV64 builds, inventory, traces, negatives |
| NR-06 | Portable C path: C-PORTABLE builds and the Rust probe |
| NR-07 | ISA and ABI inventory: every NT-06 row, verified or rejected |
| NR-08 | Hardware attestation: the owner's own devices per tuple |
| NR-09 | Resource and solo capacity: NT-08 estimates against E-SOLO |
| NR-10 | Comparative disposition: the distinguishing rule and the final choice |

## 12. Decision procedure

1. Evaluate every gate per tuple and for the portable path, then per candidate
   by the precedence over the portable path and the candidate's tuples.
2. A development-profile epoch concludes `inconclusive`.
3. Without owner input covering every review scope, the epoch concludes
   `inconclusive`.
4. A candidate is eligible when every gate passes. None eligible:
   `inconclusive`. One eligible: recommend it.
5. Several eligible: the candidates are nested, so the owner's distinguishing
   rule decides. With none recorded the epoch concludes `inconclusive`; a rule
   that leaves several concludes `tie`.
6. No weighted score is computed.
7. The conclusion is advice to the owner. The choice, and any freeze, is the
   owner's alone and is not recorded by the tool.

| Rule | Name | Rule text |
| --- | --- | --- |
| DR-1 | Coverage first within capacity | Among eligible candidates whose AX-07 slices fit the owner's declared solo slice capacity, prefer the highest AX-01; equal coverage is a tie. |
| DR-2 | Solo first | Prefer the fewest AX-07 slices, then the highest AX-01; otherwise a tie. |
| DR-3 | Cost first | Prefer the lowest AX-04, then the lowest AX-06, by the materiality bands; otherwise a tie. |
| DR-4 | Dominance only | Recommend a candidate only if it is no worse on every axis and materially better on at least one than every other eligible candidate; otherwise inconclusive. |

The rules predict different winners. Applied to the development epoch's axes
as if all five candidates were eligible, DR-1 gives a TE-02 and TE-03 tie at
capacity 1, TE-01 at capacity 2 and TE-05 at capacity 3; DR-2 and DR-3 give
TE-04; and DR-4 is inconclusive. The owner should record the rule before a
measured run and say in NR-10 if it was chosen afterwards.

## 13. Completion criteria

The suite is complete for owner decision when:

- a `measured` epoch has run with every toolchain, Rust standard library and
  emulator the packet names present, or with each absence accepted by the
  owner as an acquisition that stays unresolved;
- `verify` passes on that epoch;
- the owner input records a verdict on every inventory row, the owner's own
  devices and native runs per tuple, the AArch64 feature negative, every review
  scope and a distinguishing rule; and
- a laboratory rerun with that owner input produces the summary the owner
  reviews.

None of these is met at this snapshot. The development epoch described in the
research README exercises every stage and is not a result.

### Named gaps

| ID | Gap | Detail |
| --- | --- | --- |
| G-01 | Ed25519 signatures | RFC 8032; proposed D-015 family. No stand-in kernel and no Orange source in `algorithms/` at the base revision. |
| G-02 | ML-KEM arithmetic | FIPS 203; only its SHA-3 and SHAKE core is covered. `algorithms/README.md` lists ML-KEM-512 as being written. |
| G-03 | Post-quantum signatures | FIPS 204 or FIPS 205; no stand-in kernel. |
| G-04 | AES-192 | FIPS 197 C.2 is reproduced by `algorithms/aes/aes.or`, but the stand-in kernel supports 128- and 256-bit keys only; N-07 checks that a 24-byte key is refused. |
| G-05 | GCM with IVs other than 96 bits | SP 800-38D section 7.1; GCM test case 6 and Wycheproof tcId 2 (`algorithms/aes-gcm/aes-gcm-iv.or`) need a GHASH-derived J0, which the kernel does not implement. |
| G-06 | XChaCha20 and XChaCha20-Poly1305 | Reproduced in `algorithms/` but outside the stand-in kernel. |
| G-07 | HMAC truncation and HKDF edge cases | RFC 4231 test case 5 and Wycheproof HKDF tcId 69 are not driver subjects. |
| G-08 | Memory-address and timing channels | QEMU user mode offers no memory-access trace without plugins, which the distribution package does not build; NT-03 sees control flow only. |
| G-09 | AArch64 feature negative | QEMU 8.2's AArch64 CPU models all implement FEAT_AES, so N-12 cannot run for T-A64 in emulation. |
| G-10 | RV64 second C toolchain and Rust standard libraries | `gcc-13-riscv64-linux-gnu` and the Rust `aarch64` and `riscv64gc` standard libraries are not installed on the laboratory host. |

### Nonclaims

- The kernels are contributor-written C standing in for code Orange does not
  yet generate; nothing here measures Orange code generation.
- No build is claimed constant-time. Trace equivalence is emulated control
  flow; it sees no cache, memory-address, power or timing channel.
- Emulated timings are never compared with native timings; emulated time is a
  CI-cost proxy, not a performance figure.
- ISA and ABI inventory rows are recorded from the contributor's knowledge, not
  checked from the laboratory host, and count only when the owner verifies
  them.
- The laboratory host is a contributor machine; it is never evidence of
  owner-accessible hardware.
- Every record is contributor-produced and unreviewed; no owner decision,
  review or freeze is recorded.
