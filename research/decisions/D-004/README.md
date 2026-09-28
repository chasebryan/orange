# D-004 pre-epoch decision laboratory

Status: `run_recorded_review_pending`. The v0.7 tranche builds every epoch
prerequisite the reviewed protocol listed as absent, and epoch
`d004-e-4aaf8a83a01693d543c4` has run all 75 scheduled executions. Its results
are contributor-produced and unreviewed. The v0.8 suite adds cases and measures
that separate the candidates v0.7 could not, and epoch
`d004-e-aee8a5dee258f7ce2078` has run all 105 of its executions under the
owner's isolation-first rule, which leaves ST-REL. The owner chose that rule
knowing which candidate each rule would leave. Those results are also
contributor-produced and unreviewed. No conclusion or semantic-strata selection
exists. D004-PRE-01 remains
`solo-reviewed`, and the v0.6 implementation closure is
`provisional_pending_exact_merged_revision`.

On 2026-07-26 the Orange Project Owner accepted D004-PRE-01 at exact review-
subject revision `7d09a27369649855ce987c76315271b0d34a20ef`. That direction
authorizes the reviewed `d004-v0.6-reviewed-protocol` tranche and work on its
remaining executable-freeze prerequisites. It does not accept D-004, select or
prefer a candidate, freeze or authorize an evidence epoch, authorize S3b, or
change roadmap, release, or readiness status.

On 2026-07-26 the owner accepted D-003 candidate PF-01. Accepted OEP-0004 binds
that disposition to exact revision
`a82a5cec2ee4359dc2fe66171f17c93146747333`; the current D-004 packet records
it as `accepted_exact_revision_oep_closure` without inferring any D-004
disposition.

The v0.5 packet, catalogs, manifests, and the exact bytes of
[`SEMANTIC_STRATA_DECISION_SUITE.md`](../../../docs/SEMANTIC_STRATA_DECISION_SUITE.md)
at the review-subject revision are immutable historical review subjects. Their
embedded `draft_unreviewed_input_only`, unresolved, and
`owner_protocol_review: none` fields describe that v0.5 snapshot and are not
rewritten. The v0.6
protocol records the later owner disposition as a content-addressed overlay.
Because the direction preceded that overlay's commit identity, its implementation
closure remains provisional until the validated bytes are available at an exact
merged revision.

The v0.5 input-only laboratory consists of:

- `d004-v0.5-draft-packet.json` is the immutable base packet;
- `d004-v0.2-named-mutations.json` preserves the 26 named mutation
  definitions;
- `d004-v0.4-case-subjects.json` byte-materializes the five positive and 26
  named-mutation subjects;
- `d004-v0.2-cross-cutting-fixture-proposals.json` preserves the 42 frozen
  proposal identities and their original unreviewed-definition status;
- `d004-v0.3-cross-cutting-executable-fixtures.json` byte-materializes one
  candidate-neutral structural subject for each frozen proposal identity;
- `d004-v0.5-candidate-mappings.json` carries five unreviewed structural
  candidate graph proposals and 70 unresolved SR mapping rows as authenticated
  input-only data under schema `d004-candidate-mapping-catalog-v0.1`; and
- this README states the current lifecycle and non-claim boundary.

The successor tranche adds exactly three reviewed, uninstantiated protocol
artifacts under `d004-v0.6/protocol/`:

- `d004-pre-01-owner-record.json` records the `solo-reviewed` owner direction
  and exact review subjects;
- `reviewed-protocol.json` closes the seven protocol-review gaps while retaining
  an unfrozen null epoch and zero evidence; and
- `reviewed-replay-plan.json` expands the 25 logical candidate-case units into
  the reviewed repetition-major order of 75 planned executions without
  assigning an epoch, packet identity, executable manifests, or result credit.

Their raw-file/canonical-subject SHA-256 pairs are:

- owner record:
  `cbdfa3e07245a6843100a6b17860ea5dcec39f7f341b194faeee91e2ae585f3c` /
  `587c3ad11bf6e0d3dddc02cd7ba53896f54f8e08ec59ed1ece286e13fe9b0c9d`;
- reviewed protocol:
  `1111889b47edf24e88926bf8fa6770cf84ebcd1abb1d7dc2687dc42e0135fb53` /
  `c67c17bdf68eb0619ec7698dd2807912251a0556931fa79b6838a9d5c6a9bd98`;
  and
- reviewed replay plan:
  `45632f796c7c08d26e668b277ccaff5679ccb82857732c3b8beead66198a3eb7` /
  `a18084768cd05f6fcffdea724e330c0c81c5caac7816213156f4f9967fc5cb1b`.

The v0.5 packet binds 23 exact raw inputs: the current suite, product-form
packet, accepted S2 and S3a premises, journey identities, S3a conformance
runner and fixture corpus, both v0.2 definition manifests, the v0.3
cross-cutting catalog, the v0.4 case-subject catalog, and the v0.5 candidate-
mapping catalog. It also binds the canonical identities of both manifests and
all three catalogs.
Replay preparation authenticates
every raw input before it parses the closed manifests, validates the ordered
42-entry proposal join, 31-entry case-subject joins, and five-graph/70-row
candidate-mapping joins, recomputes every record and subject digest, and
evaluates the bounded structural integrity oracles.

The v0.5 packet's canonical SHA-256, excluding its terminal line feed, is
`b6df1a38f8a1eb6a80a8864324c21a81cb292d4c48e1981b4547bad41933b340`;
its raw-file SHA-256, including that line feed, is
`ec3a0a593d1dab7a6ace874dae4fd03c1ae0656cf301897ccabf51cb109c4009`.

The case-subject catalog contains one positive subject for every SC-01 through
SC-05 case and one independently addressed subject for each of the 26 named
mutations. Every mutation binds its exact v0.2 manifest record and same-case
positive baseline. Its canonical SHA-256 is
`5b9e734b6bad7913072e87adb29c58547d67bdcb46af942eb6bbc79d0e68166e`;
its raw-file SHA-256 is
`6266b8e38ad1a83fb777278fc0369844749b2915eabb40ba1ddfc9efa7c985f2`.
Declared expectations contain no observed
state, match, verdict, result, evidence, or candidate capability credit.

The catalog's 42 subjects comprise 14 missing-edge, 13
identity-substitution (including model, dependency-manifest, and positive-subject
identity targets), and five each for ambiguity, explicit suite-domain
unsupported behavior, and deterministic suite-domain exhaustion. Each subject
has an independent canonical SHA-256 identity. The catalog itself has
canonical SHA-256
`0516a84260bcc4d8ebb64e0cd3416deb5c43a86b7f5cd882ca757c924e575767`
and raw-file SHA-256
`5fea65960c47818243d41076dd96a6cab2dbd6d4038fd354a3f5ba30a12622ae`.
Integrity preflight is not candidate execution and persists no observed state,
match, result, verdict, evidence, or capability credit. It invokes no candidate
adapter, process, tool, compiler path, or network. Suite-domain unsupported is
not candidate-adapter inability, and suite-domain exhaustion does not exercise
a replay ceiling.

All 73 case and cross-cutting subjects remain candidate-neutral suite-only
inputs. D004-PRE-01 finds the 5 ambiguity, 14 missing-edge, 13 identity-
substitution, 5 unsupported, and 5 resource-exhaustion subjects reviewed and
sufficient solely for bounded suite coverage. This disposition creates no
candidate capability or evidence credit and does not turn their historical
source catalogs into accepted Orange semantics.

The candidate-mapping catalog contains one historical structural graph proposal
for each of the five candidates and one row for every candidate and
SR-01-through-SR-14 pair, for 70 rows total. D004-PRE-01 reviews all five graphs
and 70 rows only as symmetric, falsifiable test hypotheses. Their semantic
status remains unaccepted and conformance remains unresolved until execution;
no adapter, execution, evidence, freeze, selection, or readiness credit
follows.
Its catalog-subject SHA-256 is
`e3b790857ee21a0c651995919aaadb9bf59b05367a8dce99bab6afb6e7d2543f`;
its canonical SHA-256 is
`c967d7db8ea5049da054129367ec61cd80d729b8ce8cd34c95a76e42c67c97b8`,
and its raw-file SHA-256 is
`70765c64936bbb8aafd6e101fbf20c85396eb722d70e55bb9311d14bfbb15156`.

The Rust laboratory also checks a closed, in-memory future-schema descriptor
that enumerates all 31 top-level future case-record fields, their nested shapes,
and their cross-field invariants. It binds both authenticated subject catalogs
into an exact 73-row observation oracle and binds the authenticated candidate-
mapping catalog into five exact graph/map identity rows; future result fields
must resolve those catalogs and cannot broaden their outcomes. It also defines a
10-field scheduled-slot preimage contract and necessary-and-sufficient verdict
conditions, including checked resource bounds and SR dependency joins. The
25-slot identity plan exists. The reviewed replay policy now requires exactly
three deterministic repetitions per unit, a fresh empty candidate-specific
cache per execution, and equality of the specified deterministic fields. The
reviewed replay plan therefore contains 75 ordered but uninstantiated execution
rows. No concrete scheduled-execution digest is available until an epoch,
packet identity, and executable manifests are frozen. This synthetic contract
accepts no populated records, launches no process or adapter, and persists
nothing. At v0.6 its epoch was null and unfrozen, execution was
unauthorized, and execution evidence was 0 completed of 25 required
candidate-case units and 0 of 75 result records; the v0.7 run below records
the first epoch. Selection and conclusion remain null, and both
`roadmap_gate_credit` and `readiness_credit` remain `none`.

## v0.7 run prerequisites

On 2026-09-28 the owner directed that development never waits on a freeze
unless the owner asks for one, and that older repository rules are not the
standard for current Orange work. The owner then chose to run D-004 after the
prerequisites are built. The v0.7 tranche follows those directions. It is
contributor-produced and is not an owner record.

- [`tools/d004_adapter.py`](../../../tools/d004_adapter.py) is one
  candidate-neutral adapter for all five candidates. It derives each
  candidate's semantic model, endpoint inventory and parameter bindings from
  that candidate's reviewed graph, evaluates suite subjects at the candidate's
  own crossings, and never receives an oracle row or declared expectation.
  Slots owned by open decisions stay symbolic; a crossing delegated to a host
  whose identity is still open reports `unsupported`.
- [`tools/d004_run.py`](../../../tools/d004_run.py) captures the host tool,
  dependency and environment manifests, derives a content-addressed epoch,
  and runs each execution under user, mount, IPC, UTS, PID and network
  namespaces, `setpriv`, the repository Landlock sandbox, and per-execution
  memory and pids cgroups. It writes closed case records and parses records,
  repetition closures, correction records and the archive manifest.
- `d004-v0.7/adapter-bundle.json` publishes the adapter contract, rule table,
  slot bindings, candidate-model digests, per-case input manifests, the record
  contract and the archive layout.
- `d004-v0.7/protocol/prerequisites-overlay.json` records how each of the nine
  reviewed epoch blockers is now met and eight amendments to the reviewed
  protocol.

The amendments replace the separate owner freeze record with a mechanical
epoch identity, derive the packet, replay plan and 75 scheduled identities
before the first execution, reconcile the record fields, keep adapter requests
identical across repetitions, label records contributor-produced and
unreviewed, treat any adapter or runner change as a shared change that starts
a new epoch, forbid a tie-break rule written after results exist from choosing
in the same epoch, and fail the epoch closed when the process-tree meter
cannot report a measurement.

The raw-file/canonical SHA-256 pairs are:

- adapter bundle:
  `577d071cf5480e10cd24a53747238b726f2931129237c53c4546ecba8ef55da5` /
  `28f422dd6a482a4e9a2c30bebf47fd1b50ba520072207c2b4463881c931eaf80`; and
- prerequisites overlay:
  `ddb999ef447d4864cab3bbe79a2e09799c21824a5d8076464ccfd556e1e7bee4` /
  `a2e1830b840a3ac2e46332ddd2fda9675ec09bc7d1acc4bdb5cddef61289da59`.

## v0.7 epoch run

Epoch `d004-e-4aaf8a83a01693d543c4` was prepared and run on 2026-09-28 from
source revision `2a1fa50e9e245548771a1b4d51d5a1662b1f0fe8`. Its packet binds
the bundle, the overlay, run harness raw SHA-256
`8f385cd6b3c077c80a6aeacae7c8847e0dca0a045d91b269c02074db50aa3597`, the host
tool, dependency and environment manifests, all five candidate models and all
five input manifests. All 75 executions ran once each in the reviewed physical
order under the enforcing launcher. Every execution exited normally within the
ceilings: wall time 89 to 142 ms, peak memory 11.2 to 11.9 MB for the whole
process tree, no temporary storage, and no stderr output.

| Candidate | Cases passed | Closed units |
| --- | --- | --- |
| ST-REL | SC-01 to SC-05 | 5 of 5 |
| ST-UNI | SC-01 to SC-05 | 5 of 5 |
| ST-DUAL | SC-01 to SC-05 | 5 of 5 |
| ST-MIRROR | SC-01 to SC-05 | 5 of 5 |
| ST-HOST | none | 0 of 5 |

Canonical evidence is now 20/25/75: 20 closed candidate-case units of 25
required, and 75 of 75 result records. Four candidates are complete, and no
case is complete across all five candidates. The three repetitions of every
slot are byte-identical in their deterministic fields, so every failure is a
deterministic failure, not an independent or flaky one.

ST-HOST fails every case for one reason. It delegates SR-04, SR-05, SR-08,
SR-11, SR-12, and SR-13 to hosts whose identities are parameters of the open
D-006 and D-011 decisions, and the adapter reports those relationships as
`unsupported` until a host is selected. SC-04 and SC-05 also depend on those
crossings for their positive and mutation observations, so 3 and 7
observations there are `unsupported` where the oracle requires `succeeded` or
`rejected`.

The main finding is about the suite. ST-REL, ST-UNI, ST-DUAL, and ST-MIRROR
are indistinguishable under it: the shared adapter evaluates every candidate
by the same rules at that candidate's own crossings, and each of the four
passes every case. The suite as reviewed can therefore rule out a candidate
that cannot yet reach a relationship, but it cannot choose among candidates
that can. Choosing among the four needs either an owner judgment on grounds
the suite does not measure, or a successor suite with cases that separate
them. The epoch packet fixes the selection rule as absent and forbids a
tie-break written after results exist, so no selection follows from this run.

The runner's archive is about 13.5 MB. Its committed form under
`d004-v0.7/run/` keeps only what cannot be re-derived: the packet, the host
captures, each execution's state, measurements and diagnostics, and the 25
distinct adapter outputs. [`tools/d004_archive.py`](../../../tools/d004_archive.py)
rebuilds the full archive byte for byte and checks it against archive-manifest
SHA-256 `8538b9008dcb455410c2a23044f8d2f6046ef81b0dc5f56847556f323720eb85`,
then re-runs the harness's `verify` over it. The raw-file/canonical SHA-256
pairs are:

- archive index:
  `e142ad7e177c6b4a6d507176c60151175a82464c912ac3d222de3c0ffe9f0af0` /
  `716d1132e73edfb06f06e6f134ade9220a83d98d1c6ee780803d54e8fe5427f6`; and
- adapter outputs:
  `e14031975071c0b6777a3dcd4bc40621b70f9ff16ea40f8d695c043e41c8774b` /
  `aec7e3bc5ed42dfb66e9cb014215ab750f0d95dedaca88b0c7ce1d5c5c162995`.

Three earlier epochs ran the same schedule and produced the same verdicts,
closures, mismatches and unsatisfied relationships in every slot. All three
were superseded before any review, the first two by code-scanning fixes and
the third by a review finding, because any change to the adapter or
run-harness bytes starts a new epoch:

- `d004-e-eefa7ffe75b0a9765894` ran from revision
  `571c5bd1ffad65f304d8171bfe4f68d382598575`. The fix after it stopped the
  adapter and the run harness accepting caller-chosen command-line paths. Its
  archive-manifest SHA-256 was
  `eb96d805127067c628c62b82cf37059566c303320fc6e2c626f709625e700a49`.
- `d004-e-b2b129e87916beb23d3e` ran from revision
  `93502f483b6ffab33857bf4ce64610df4cab81b3`. The fix after it stopped the
  run harness copying the trusted-components list through a string subscript
  that code scanning reads as a secret. Its archive-manifest SHA-256 was
  `14eea69d349f62f1d9495e5e83dbcb0f59a3a63a8596f6eb5f338b7b301718ed`.
- `d004-e-d1458087bc013d1e734f` ran from revision
  `b65f0b8056f3222b9e9222c08446b9ae01c4f6fa`. The fix after it made execute
  re-capture the tool, dependency and environment manifests, including the
  isolation probe, and refuse to run unless they equal the ones prepare
  recorded. Its archive-manifest SHA-256 was
  `4e3bbc8d8df5eb9c7ab1dc043aa8197c03332b8bd7a1926655920a34c0ebe20e`.

No superseded archive is committed.

Any later change to the adapter, the run harness, the bundle, the overlay or
their bound inputs starts a new epoch; this one stays reproducible only from
these bytes. D-004 remains proposed pending owner review of these results,
S3b remains blocked, and Orange's binary gate-closure score remains 3 of 10
(30%).

## Stored S3a inputs

The v0.5 packet binds 17 inputs outside this laboratory by path and raw
SHA-256: the S2 and S3a language and semantics documents, OEP-0003, the product
form packet, the user journeys, the S3a conformance runner and its eleven
fixtures. Those files are live parts of the language and change as later slices
land. So that D-004 records what it measured without pinning the live
specification, byte-identical copies of those 17 files are stored under
`baseline/`, at the same relative paths the packet names. On 2026-09-28 the
owner chose this over keeping the live files pinned.

The foundation validator checks each packet binding against its stored copy,
and the Rust laboratory embeds the copies. The live files keep only their
ordinary Gate 0 protected digests, so S3b and later slices can amend them
without touching D-004. No digest in the packet, the protocol, the bundle, the
overlay or the committed run changes, and the v0.7 epoch identity is unchanged,
because the run harness never read these files. The suite document stays bound
at its live path, because it is this laboratory's own record.

## v0.8 suite

The v0.7 run could not separate ST-REL, ST-UNI, ST-DUAL, and ST-MIRROR. Their
v0.5 graphs differ only in how they group semantic members: which members
exist, which members are views of a parent, and which crossings stay inside one
authority. On 2026-09-28 the owner chose to build a successor suite that can
tell them apart. The v0.8 tranche adds that suite. It is contributor-produced
and is not an owner record.

v0.8 keeps SC-01 to SC-05 and adds two cases:

- **SC-06, semantic evolution.** Four suite-fixed changes: a new target
  operation class (E1), a revised memory model (E2), a revised sampling model
  (E3), and a revised proof-evidence interface (E4). Each change re-identifies
  every subject class under the member that owns the changed construct. Where
  a change reaches a class that a suite section 4 invariant protects (Spec Core
  for E1 to E3, runtime subjects for E4), the candidate must carry an open
  isolation obligation.
- **SC-07, within-authority relabeling.** For SR-07 to SR-11, a probe presents
  a subject from one side of the crossing as the other. It must be rejected at
  a member boundary or by a named discrimination judgment. A crossing inside
  one authority with no named judgment fails the case.

A construct belongs to the member that holds its facet, and ownership is the
authority root of that member. A crossing whose two sides share one semantic
authority root needs a discrimination judgment, except SR-06 and SR-12, which
stay inside one authority for every candidate. The adapter derives both from
each candidate's reviewed graph.

SC-06 and SC-07 record five measures, all counts where fewer is better:
isolation obligations, the part of them that protects Spec Core, re-identified
subject classes, discrimination judgments, and independent semantic
definitions. Pass or fail alone is not expected to separate the four
candidates, because each can meet both cases. When more than one candidate
passes, suite section 8 leaves the result inconclusive until the owner records
a non-compensable distinguishing rule, and AM-07 lets such a rule decide only
an epoch that runs after it is recorded. The v0.8 adapter bundle offers four
such rules: isolation first, fewest definitions first, spec isolation first,
and dominance only. `prepare` refuses until the overlay records the owner's
choice, and the epoch packet binds it.

- [`tools/d004_v08_adapter.py`](../../../tools/d004_v08_adapter.py) and
  [`tools/d004_v08_run.py`](../../../tools/d004_v08_run.py) extend the v0.7
  adapter and harness in new files, so the v0.7 run keeps verifying from its
  own bytes. The schedule is a 5 by 7 rotation, run three times for 105
  executions.
- `d004-v0.8/case-subjects.json` holds the SC-06 and SC-07 positive subjects
  and seven named mutations.
- `d004-v0.8/adapter-bundle.json` publishes the adapter contract, the new
  rules and computations, each candidate's judgments and definitions, the
  measures and the distinguishing rules.
- `d004-v0.8/protocol/suite-overlay.json` binds the v0.7 bundle and overlay,
  adds amendments AM-09 to AM-13, which replace AM-02's execution count and
  AM-07's first sentence, and records the owner's distinguishing rule with the
  context in which it was chosen.

All three regenerate byte for byte (`python3 tools/d004_v08_run.py check`).
The raw-file/canonical SHA-256 pairs are:

- v0.8 case subjects:
  `ecbb05bc18f0c35e7dd11083184702f6492bfdaa80d3351b69bfdc994c7cd125` /
  `bbe14dca3b7d60f38df9357491a49ea75581c932e9b1d680f7783a4ea35b8d7f`;
- v0.8 adapter bundle:
  `e4dca7aa537403c76be925aaa8d9bdec54725071c7ca39ca250be3e8e88ca3ba` /
  `d756959e3258d5ac86cb416e5993fc2f90119867a600170af7b69abe4dadd02c`; and
- v0.8 suite overlay:
  `658e899497fd1b2860f63a43039123facf56a1ff99f7b34273eefbfc427c2806` /
  `b86e791fd1a49624816ab0fcb63ed633819cb66f3e2ca70955a6cfe6bbc28618`.

On 2026-09-28 the owner chose isolation first: fewest isolation obligations,
then fewest re-identified subject classes, then fewest discrimination
judgments, then fewest semantic definitions. The overlay records that choice,
so every v0.8 epoch packet binds it. The choice was not blind. The measures are
deterministic functions of the candidate graphs and the suite, and the decision
card, posted at 10:14 UTC, showed the candidate each rule was predicted to
select. The contributor who built the suite recommended isolation first on that
card, giving as reasons that evidence has to survive frequent new targets, that
the suite's invariants are about isolation, and that the rule matches the
earlier ST-REL research recommendation. At about 10:27 UTC a local trial of the
harness, `d004-e-f23ea422b328ad18e04a`, ran all 105 executions with the
isolation-first rule marked as a dry run, verified, and reproduced those
predictions; its archive-manifest SHA-256 was
`fbcd6067f32a28367ba58fa5d29377cb51d933d2155764afc0b9c17eedac7850`. That trial
is not a D-004 epoch, and its archive is not committed. The owner chose at
11:01 UTC. The overlay's rule record states this context. So the rule was bound
before any v0.8 epoch ran, but it was chosen with its outcome known, on a
recommendation that matched the existing research favorite, and it is not a
preregistration made in ignorance of the result.

## v0.8 epoch run

Epoch `d004-e-aee8a5dee258f7ce2078` was prepared and run on 2026-09-28 from
source revision `690dea814bc29a08af1139b7ce07bc2dc6332ca5`. Its packet binds
the v0.8 bundle and overlay, and with them the owner's rule, v0.8 run harness
raw SHA-256
`f439889a8d5e46491569dcbf136714edc5ede9cd069945184e049194781314a5`, the host
tool, dependency and environment manifests, all five candidate models and all
seven input manifests. All 105 executions ran once each in the rotation order
under the same enforcing launcher as v0.7, after execute re-checked the host
context that prepare recorded. Every execution exited normally within the
ceilings: wall time 88 to 147 ms, peak memory 10.8 to 12.4 MB for the whole
process tree, no temporary storage, and no stderr output.

| Candidate | Cases passed | Closed units |
| --- | --- | --- |
| ST-REL | SC-01 to SC-07 | 7 of 7 |
| ST-UNI | SC-01 to SC-07 | 7 of 7 |
| ST-DUAL | SC-01 to SC-07 | 7 of 7 |
| ST-MIRROR | SC-01 to SC-07 | 7 of 7 |
| ST-HOST | none | 0 of 7 |

Evidence for this epoch is 28/35/105: 28 closed candidate-case units of 35
required, and 105 of 105 result records. The three repetitions of every slot
are byte-identical in their deterministic fields. ST-HOST fails SC-01 to SC-05
for the reason v0.7 found, and fails SC-06 and SC-07 the same way: their
positive subjects cross SR-04, SR-05, SR-08 and SR-11, which it delegates to
hosts owned by the open D-006 and D-011 decisions, so each reports
`unsupported` where the oracle requires `succeeded`.

As designed, pass or fail does not separate the four other candidates. The
measures do, but which candidate they leave depends on the rule. Every measure
is a count where fewer is better:

| Candidate | Isolation obligations | Of them, Spec Core | Re-identified classes | Discrimination judgments | Semantic definitions |
| --- | --- | --- | --- | --- | --- |
| ST-REL | 0 | 0 | 6 | 0 | 5 |
| ST-UNI | 10 | 6 | 36 | 5 | 1 |
| ST-DUAL | 4 | 0 | 28 | 2 | 2 |
| ST-MIRROR | 0 | 0 | 7 | 1 | 5 |

The owner's rule, isolation first, compares isolation obligations, then
re-identified classes, then discrimination judgments, then semantic
definitions. ST-REL and ST-MIRROR tie at zero isolation obligations, and ST-REL
re-identifies six subject classes to ST-MIRROR's seven, so the rule leaves
ST-REL (`recommend_st_rel`). Fewest definitions first would have left ST-UNI,
spec isolation first would have left ST-DUAL, and dominance only would have
removed only ST-MIRROR, leaving ST-REL, ST-UNI and ST-DUAL inconclusive. The
owner chose knowing this, as the "v0.8 suite" section above records. The
summary scopes this result to the candidates that close all seven cases and to
SS-G05 and the SS-G03 structure only. It is not a D-004 recommendation under
suite section 8 until the owner disposes every candidate and every hard gate,
so `selection` stays null.

The runner's archive is about 15.6 MB. Its committed form under
`d004-v0.8/run/` keeps the same parts as v0.7's, and
[`tools/d004_archive.py`](../../../tools/d004_archive.py) rebuilds it byte for
byte, checks archive-manifest SHA-256
`73566b1acc8a959aae426ca167e170c3d4f7ad02fb988c4a7337533f6ff6f015`, then
re-runs the v0.8 harness's `verify`. The raw-file/canonical SHA-256 pairs are:

- v0.8 archive index:
  `9c52a12a78d7dd0f28417e77f1a497d76664d2a29c2c1129650eb37df89dc7df` /
  `edd2d1b45b72eb12a7e5b03693cdf185fc5579bd52dbbfd53c50ce5f7ddf630c`; and
- v0.8 adapter outputs:
  `e521e83471a10e75b55bc07bf297560e600bc2dd61e0fd0e525bb84d42e1a3f9` /
  `bb23588ac192d870c79c8716cac13f21c44e32b050121cfb2402f28405de3ceb`.

Two earlier v0.8 epochs ran the same schedule and produced the same verdicts,
closures, measures and rule result in every slot. Both were superseded before
any owner review:

- `d004-e-afadaff4cbc1bb294d3e` ran before the v0.8 harness gained the host
  re-check, and no review had seen it. Its archive-manifest SHA-256 was
  `6851937e0a6826657436b5dc523291c08790bed56f955d2e60245dfcd4b3ce74`.
- `d004-e-1b3a184e9895cc149b09` ran from revision
  `f39d181775ae2992479e0857083c3d7e02a172c1`. A contributor review of that
  run found the gaps the next fix closes. The fix makes the overlay name what
  AM-09 and AM-13 replace in AM-02 and AM-07 and record the context of the
  owner's choice. It also makes the harness read each case's
  measures strictly and report the rule result as inconclusive when a
  candidate that closes every case lacks a full measure set. Its
  archive-manifest SHA-256 was
  `84e2fd244b04db0276941d34e7208f1f2b46f0643c58f320e9030e401eaccb3c`.

No superseded v0.8 archive is committed.

Any later change to the v0.8 adapter, the v0.8 harness, the v0.8 documents,
the v0.7 bundle or overlay they bind, or their inputs starts a new epoch. D-004
remains proposed pending owner review of both runs, S3b remains blocked by
D-004 in the decision register, and Orange's binary gate-closure score remains
3 of 10 (30%).
