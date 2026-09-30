# D-006 v0.3 epoch runner

`tools/d006_run.py` runs the D-006 v0.3 protocol for both candidates, records
every step and summarizes the evidence. It uses the Python standard library
and the laboratory host's own tools. It selects nothing: every summary it
writes carries the suite's nonclaims, and the conclusion stays `inconclusive`
until the owner-performed parts of the protocol exist.

## Commands

```sh
python3 tools/d006_run.py prepare [--root DIR]
python3 tools/d006_run.py execute ARCHIVE [--revision REV]
python3 tools/d006_run.py summarize ARCHIVE
python3 tools/d006_run.py export ARCHIVE DEST
python3 tools/d006_run.py verify ARCHIVE_OR_EXPORT
```

`prepare` refuses a working tree with uncommitted changes, regenerates the
shared inputs with `tools/d006_shared.py check`, stores every toolchain archive
under its SHA-256, compiles `tools/fs_sandbox.c`, and writes `packet.json`.
The epoch name is `d006-e-` and the first 20 hexadecimal digits of the
SHA-256 of what the packet binds: the shared-input manifest, the suite
overlay, the toolchain record, the runner, the harness, the shared reference,
the sandbox source, the toolchain archives, the ceilings, the sandbox limits,
the laboratory uid and the meter. The bootstrap seed for every confidence
interval derives from the same hash. There is no owner freeze record; a
change to anything bound opens a new epoch.

`execute` runs the preregistered plan: workspace W1 is provisioned from a
`git archive` of the packet's revision, then five cold bootstraps, three
deterministic replays in each CPU mode, one warmup and thirty timed pairs per
case, the nine DS-06 faults, and finally two deterministic replays in a
separately provisioned workspace W2. Candidates alternate as the overlay's
`execution_order` says. `--revision` runs a correction round (AM-09) from a
later revision whose bound inputs are byte-identical.

`summarize` computes M-01 to M-18, the hard gates, the materiality labels and
the conclusion from the latest attempt's records. `export` writes the
committed form of a verified archive: the packet, the records and logs as
JSON lines in gzip chunks (at most 8 MiB uncompressed each, compressed with
no file name or time so the same lines give the same bytes), the summary and
a manifest; the toolchain archives stay out, named by digest in the packet.
A record's projection, standalone results and candidate tree, and each
step's invocation (argv, directory, environment, ceiling class, CPU set and
label), which every replay repeats unchanged, are stored once by digest in
`objects-NN.jsonl.gz` and restored when the export is read. `verify` checks
either form: the epoch name and seed against the packet, every file against
its manifest, every record's epoch and projection digest, every step's logs,
and that the summary regenerates byte for byte. On a clone that has the
packet's revision it also checks each bound file at that revision.

## Isolation (AM-05, AM-06)

Every step starts as uid 60606 with no supplementary groups, then enters new
user, mount, IPC, UTS, PID and network namespaces, drops every capability,
sets `no_new_privs`, and runs under `fs_sandbox` with an environment cleared
to the overlay's allowlist plus the adapter's declared variables. Landlock
makes `/usr`, `/proc`, `/sys/devices/system/cpu`, `/dev/urandom`, the
toolchains, the candidate sources and the adapter's declared host files
read-only; only the step's run root, its temporary directory and `/dev/null`
are writable. The sandbox caps each process at 4 GiB of address space, 600
CPU seconds, 512 MiB per file, 1024 open files and 256 processes, with no
core files. Each step also runs in its own cgroup with the overlay's memory
and pid ceilings; the wall ceiling kills the whole cgroup.

A step ends in exactly one state: `completed`, `failed` (nonzero exit),
`crash` (a signal), `timeout`, `oversized_output`, `resource_exhaustion`
(a memory kill, a temp overrun or the pid limit) or `killed_by_runner`
(DS-06 fault F09 only). Only `completed` can count as success.

## DS-04 solver runs

The pinned solver runs in the sandbox with the shared argv on the shared CNF,
which the candidate's own `cnf_text` must equal (D4-CNF01, D4-CNF02). A run
supports a claim only through its exit code and step state: 20 with a
certificate file hands the certificate to the candidate, 20 without one is
`failed_certificate`, 10 is `disproved_obligation`, 0 and anything else are
`unknown`, and a timeout or resource stop keeps its own category. D4-R01 is
the fresh certificate. D4-R02 runs on B-C02, reads x and y from the model and
has the candidate compute both sides of the identity at them; the case passes
only when the two values differ. D4-R03 uses the pinned `unknown_argv`, D4-R04
runs under the `solver_zero_wall` ceiling, and D4-R05 removes the certificate
after an unsatisfiable run and has the candidate's checker reject the empty
text. Each case counts as a negative in M-03; its category is the runner's,
not a candidate diagnostic, so M-15 leaves it out.

## Archive layout

```text
d006-e-XXXXXXXXXXXXXXXXXXXX/
  packet.json          what the epoch binds, the host and the sandbox build
  toolchains/          toolchain archives by SHA-256 (not exported)
  records/NNNN-PROFILE-CANDIDATE.json
  logs/SHA256          every step's stdout and stderr, by digest
  manifest.json        every file above with its size and SHA-256
  summary.json         written by summarize
```

Records are canonical JSON (sorted keys, no spaces, a final newline) with
schema `d006-v0.3-record-1`, the epoch, an ordinal and a profile:

| Profile | What it records |
| --- | --- |
| `execution` | the attempt number, the candidates' revision and the plan |
| `provision` | a workspace's checkout, toolchain unpack steps and candidate tree |
| `cold_bootstrap` | unpack, build and checker-build steps from an empty root, the build state, the deterministic artifact manifest and the workspace bytes |
| `deterministic_replay` | every positive, negative and fresh-certificate outcome, DS-04's run-time cases, the DS-05 corpus verdicts, the projection and its digests |
| `timed_replay` | one warmup or one pair member for one case: the re-check steps and their measurements |
| `fault` | one DS-06 fault: its evidence, the build state, the projection digest beside the reference and whether the fault's rule was met |

Each step row carries its argv, working directory and environment with
workspace paths replaced by tokens (`$RUN`, `$WORKSPACE`, `$TOOLCHAIN`), its
ceiling class and CPU set, its state, and wall time, CPU time, peak resident
memory, cgroup peak, process peak and temp bytes, with its logs by digest.

## Summary

`summary.json` has schema `d006-v0.3-summary-1`. For each candidate it lists
the case states, the fault verdicts, M-01 to M-18 and the eight hard gates,
each gate `pass`, `fail` or `unresolved`. The comparative table gives each
measured metric exactly one materiality label with its raw ratio and interval.
The conclusion is `tie` only when both candidates are eligible under the full
plan and every label is `practically_equivalent`; otherwise it is
`inconclusive`, with its reasons. A recommendation needs the owner's per-axis
rationale (suite section 8), which no runner writes. H-02 timings are emulated
and never compared with H-01's.
