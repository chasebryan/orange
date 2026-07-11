# Gate 0 reproducibility contract

Status: provisional Gate 0 architecture; non-product and non-release

Snapshot: 2026-07-11

This document defines how Orange Gate 0 decision evidence is captured and
replayed. Its purpose is to make architecture choices reviewable for the life
of the project, including choices that are rejected. It does not define a
language, package, proof, compiler, registry, or release format, and it does not
claim that a future Orange binary is reproducible.

The version `0.1` schemas under [`schemas/gate0`](../schemas/gate0) are
provisional records for this contract. Ratification or replacement requires a
decision record, new identifiers, migration tests, and retention of the
original bytes. A validator accepting one of these records proves record shape,
not the truth of a claim or correctness of a decision.

## 1. Reproducibility levels

Every evidence manifest declares the strongest level actually demonstrated in
`reproducibility_level`:

1. **Replayable method** (`replayable_method`): exact inputs, tools, arguments,
   environment, and
   expected observations are recorded, but rerun access or output equality has
   not been independently demonstrated.
2. **Deterministic decision case** (`deterministic_decision_case`): the same
   recorded environment and inputs
   reproduce the declared output manifest.
3. **Independent decision reproduction**
   (`independent_decision_reproduction`): an identified reviewer repeats the
   case from archived inputs in an independently provisioned environment and
   obtains the declared output manifest.
4. **Future release reproducibility:** multiple independent builders reproduce
   published release artifacts from a signed source revision and release
   recipe.

Gate 0 records may demonstrate levels 1–3. Level 4 belongs to the future
release process and cannot be inferred from a deterministic Gate 0 case.
Semantic agreement, test success, a checked proof, and byte-for-byte equality
are different observations and must be recorded separately.

## 2. Evidence unit

A permanent Gate 0 decision case contains:

- the D-ID or OEP under evaluation and the question it answers;
- the strongest demonstrated Gate 0 reproducibility level;
- the exact source revision;
- every input, configuration, tool, proof, output, and log with a SHA-256
  digest and byte size;
- exact external-source provenance and archival state;
- one replay record as described below;
- expected output-manifest digest and exit code;
- separate expected and actual observations with status and evidence-file
  references;
- associated claim IDs and trust-inventory IDs;
- limitations, unsupported cases, and known nondeterminism; and
- an independent replay record when that level is claimed.

Evidence for a rejected option remains addressable and replayable. A later
decision may supersede its conclusion, but never mutates its recorded inputs or
outputs. Corrections create a new manifest whose
`supersedes_manifest_digest` identifies the canonical digest of the old
manifest. The old record remains immutable.

## 3. Canonical bytes and digests

### 3.1 Text files

Repository-controlled text uses UTF-8 without a byte-order mark, LF line
endings, no NUL bytes, and one final LF. Text generation must not depend on
platform-native newline translation. File names use UTF-8 and the repository
records their exact code points; case folding and Unicode normalization by a
host filesystem are not accepted as silent transformations.

### 3.2 JSON

Human-reviewed JSON may be indented in Git. Before hashing a JSON record, a
producer must:

1. decode strict UTF-8 and reject a byte-order mark;
2. reject duplicate object member names, lone Unicode surrogates, numbers that
   cannot be represented without loss in the allowed I-JSON profile, and all
   other non-I-JSON values;
3. validate the instance against its declared schema, including `format`
   assertions;
4. serialize it with the
   [JSON Canonicalization Scheme, RFC 8785](https://www.rfc-editor.org/rfc/rfc8785);
   and
5. hash the resulting canonical UTF-8 bytes.

Gate 0 records use non-negative integers for sizes, exit codes, and counts and
decimal strings for environment-variable values such as epochs. Numeric JSON
values do not use floating-point syntax and remain within the schema's exact
interoperable range. Object-member order in a checked-in, pretty JSON file is
presentation only; canonical member order comes from RFC 8785. A producer must
use an RFC 8785 implementation and its conformance vectors; ordinary
`sort_keys` JSON output is not a substitute because JCS has specific UTF-16
property-order, string, and ECMAScript number rules.

Array order remains data. Producers must use the following deterministic order
for collections that are logically sets:

| Collection | Ordering key |
| --- | --- |
| Files and archived paths | UTF-8 byte order of normalized repository-relative path |
| Claims and claim references | stable claim ID |
| Observations and independent reproductions | stable observation or reproduction ID |
| Evidence bases | stable basis ID |
| Assumptions and axioms | stable assumption or axiom ID |
| TCB components, models, and contracts | stable record ID |
| Standards, errata, clauses, and vectors | stable record ID, then locator |
| Selected Actions, repository controls, and exceptions | stable Action identity, control, or exception ID |

No producer may rely on hash-map iteration, directory enumeration, database
row order, locale collation, or concurrent completion order.

### 3.3 Digest meaning

All `0.1` schema digests use lowercase hexadecimal SHA-256. A file digest
covers the exact stored bytes unless that field's definition explicitly says
it covers canonical JSON, in which case it covers the RFC 8785 form. An input
manifest is the canonical JSON array of input file records ordered by path;
each record contains path, role, media type, executable bit, byte size, and
content digest. The `input_manifest_digest` is the digest of that canonical
array. The output-manifest digest is computed the same way.

A manifest never authenticates itself by embedding a digest of its complete
bytes. Its identity is supplied by an enclosing signed record or content store.
Changing one byte, mode, path, tool identity, or environment value creates a
new digest and therefore a new evidence unit.

## 4. Deterministic execution profile

### 4.1 Time

Recorded observation times use UTC with exactly second precision:
`YYYY-MM-DDTHH:MM:SSZ`. Local time zones, daylight-saving rules, and implicit
"current time" are forbidden inputs.

Replay sets `TZ=UTC` and `SOURCE_DATE_EPOCH` to a recorded non-negative Unix
timestamp. For repository-derived cases, the provisional default is the author
timestamp of the exact source commit, obtained with:

```sh
git show -s --format=%ct <full-commit-id>
```

The chosen value is written into the evidence manifest. Tools must use it in
place of wall-clock time for generated metadata. If a tool cannot do so, the
affected bytes are nondeterministic and the limitation must be recorded; they
cannot participate in a byte-reproducibility claim.

### 4.2 Locale and environment

Replay begins from an empty or allowlisted environment and sets at least:

```text
LANG=C
LC_ALL=C
SOURCE_DATE_EPOCH=<recorded integer>
TZ=UTC
```

The four baseline values occupy named properties in `environment`. Every other
allowlisted variable that can affect behavior appears in the
`additional_variables` object. Baseline names cannot be repeated there. An
empty object means there are no additional variables; it does not authorize
inheritance. Home directories, usernames, terminal width, color support, CPU
count, umask, host-specific cache paths, and inherited compiler flags are not
implicit. A case that intentionally varies one of them records each variant as
a distinct run.

### 4.3 Randomness and concurrency

Every randomized case uses `randomness.mode: "fixed"` and records the generator
name, exact version, content digest, seed encoding and value, and the mapping
from the recorded seed to generator state. Entropy from the OS, time, process
ID, addresses, or scheduling is forbidden during deterministic replay. A
non-random case records `mode: "none"`, `encoding: "none"`, and
`value: "not-used"`; generator-only fields are forbidden in that mode.

Every replay has a `concurrency` record. Serial execution uses `mode: "serial"`
and one worker. Parallel work that produces the same bytes and diagnostics uses
`deterministic_parallel` and records at least two workers. If parallel behavior
cannot be made deterministic, `pinned_limited` pins the worker count and
requires the exact limitation; affected outputs cannot support byte-equality
claims. Race-dependent discovery or first-completer selection is not valid
evidence.

### 4.4 Paths and filesystem behavior

Recorded paths are repository-relative, use `/`, and contain neither an
absolute prefix, backslash, NUL, empty segment, `.` segment, nor `..` segment.
The provisional schema rejects these lexical forms, including Windows drive
prefixes and trailing `/`. The replay-only `working_directory` field may use
`.` as an explicit repository-root sentinel; content and archive paths may
not. A closure validator must additionally require Unicode NFC, resolve the
path beneath a freshly created workspace, and reject escape after symlink
resolution. Inputs may not depend on an unrecorded file in the user home,
system temporary directory, global package cache, or parent directory.

File type and executable state are inputs. Symlinks, hard links, sparse files,
device nodes, sockets, permission-sensitive behavior, case collisions, and
Unicode-normalization collisions are rejected unless the case explicitly tests
them and records host requirements. Temporary names must not appear in
canonical outputs.

### 4.5 Network

The replay phase runs with network access denied. DNS, registries, remote APIs,
time services, license servers, and mutable URLs are not reproducible inputs.
All permitted external material is acquired in a separate capture phase,
digested, provenance-recorded, and either archived or accompanied by a lawful,
digest-verifying acquisition procedure.

An online lookup may gather research but cannot close a replayed claim. If
redistribution or access terms prevent archiving, the manifest records
`metadata_only` or `acquisition_required`, the expected digest, the rights
reference, and the resulting limitation. Failure to reacquire exact bytes
leaves the replay unavailable; it never authorizes substitution of a newer
document.

## 5. Replay record

The evidence manifest stores commands as an `argv` array, not a shell string.
The first element is a non-empty executable name; later elements may be empty
because an empty argument is distinct from an omitted argument. This prevents
shell parsing, quoting, globbing, and injection from becoming hidden inputs.
The record includes:

- repository-relative working directory;
- complete argument vector;
- four baseline environment values, the complete additional-variable allowlist,
  and `SOURCE_DATE_EPOCH`;
- denied network policy and empty allowed-host list;
- fixed randomness and concurrency records;
- every tool name, exact version, content digest, and acquisition method;
- canonical input- and expected-output-manifest digests; and
- expected process exit code.

The manifest's `observations` array keeps semantic agreement, schema
validation, proof checking, test execution, byte equality, external review,
and repository-state observations distinct. Each item records an expectation,
one of `expected_only`, `matched`, `mismatched`, or `unresolved`, and the files
that support it. `matched` and `mismatched` require an actual observation;
`unresolved` requires an explanation. A replay command and exit code never
silently imply a stronger observation.

Replay starts from a clean checkout of the full recorded commit, verifies every
input and tool digest before execution, denies network access, and writes only
to a new output directory. It then compares exit code, output paths, modes,
sizes, and digests. Unexpected output is a failure. Missing output, timeout,
resource exhaustion, tool crash, unavailable dependency, or digest mismatch is
`unresolved` or `not_satisfied` as the applicable claim policy defines; none is
success.

Logs capture standard output and standard error separately as raw bytes.
Diagnostics that contain acceptable nondeterminism require an explicit
normalization rule whose source and output are both retained. A normalization
rule may remove irrelevant presentation data; it may not erase a failed check,
changed result, warning, assumption, or trust expansion.

## 6. External-source and standards capture

Before a standard, erratum, test vector, proof artifact, corpus, or external
tool becomes a decision input, capture:

- issuing organization and exact document identifier, edition, and date;
- primary publisher or authorized mirror URI;
- UTC retrieval time and exact byte digest;
- archive path or digest-verifying reacquisition instructions;
- redistribution terms and review status;
- applicable errata and the rationale for applicability;
- clause or vector locators used by the case;
- transcription path, digest, and independent-review state; and
- unresolved patent, export, certification, or access review without turning a
  technical inventory into legal advice.

A completed rights, patent, or export review requires a durable review
reference. A `single_review` transcription or technical review identifies one
reviewer; `independent_review` identifies at least two distinct reviewers. The
status never substitutes for reviewer identity or the retained review record.

Mutable web pages are archived as exact bytes when terms permit. A screenshot
alone is not a normative input. Generated transcriptions never replace the
source; both are retained, and a transcription error invalidates dependent
claims until corrected and replayed.

## 7. Review and retention

The author runs the replay from a clean workspace and records its manifest. An
independent reproduction uses a separate checkout and tool acquisition path,
then records a stable reproduction ID, reviewer identity, UTC performance time,
environment, source-manifest digest, expected and observed output-manifest
digests, result, and differences. A `matched` result has no differences;
`different` and `unresolved` results retain at least one difference or blocking
reason. A manifest may claim `independent_decision_reproduction` only when
`independent_reproductions` contains at least one such record. The author cannot
turn their own second run into independent evidence.

Inputs, manifests, outputs needed to interpret the decision, raw logs, and
review records are retained even after an option is rejected. Secrets,
personal data, embargoed vulnerabilities, and third-party bytes that cannot be
lawfully redistributed are never committed merely for reproducibility; the
record instead documents protected retention or exact reacquisition and the
assurance limitation.

## 8. Future release distinction

A future release process will need a separately ratified, hermetic build and
provenance design. At minimum it is expected to bind signed source and tag,
locked dependencies, builder identities, isolated build recipes, SBOM and CBOM,
SLSA-style provenance, signatures and transparency evidence, target and TCB
inventories, and at least two independent rebuild witnesses. Update and
rollback policy are additional gates.

Passing the Gate 0 fixtures, replaying an architecture comparison, or using
`SOURCE_DATE_EPOCH` is not release evidence. No Gate 0 manifest may label
itself an Orange release, production proof bundle, or claim-bearing artifact.

## 9. Failure policy

Reproducibility fails closed. A record is not accepted when a required input is
missing, a digest is malformed or mismatched, a reference is unresolved, an
external source silently changes, a path escapes the workspace, a schema is
unknown, a tool is mutable, the network is needed during replay, a completed
review lacks its accountable reference or required reviewers, an independent
reproduction lacks its identified replay record, or an outcome depends on
unrecorded state. The owner must correct and version the record, reduce the
claim, or leave the result explicitly unresolved.
