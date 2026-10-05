# Orange documentation

Status: living index of repository written materials; not a normative
specification, assurance report, or release claim

Snapshot: 2026-10-05

This folder holds the reader guide, language and tool contracts, roadmap,
decision records, and governance materials for Orange. It is the place to
start when the root [`README.md`](../README.md) is not enough. Nothing here
accepts an OEP, closes a foundational decision, or authorizes a release.

## Start here

| Document | Role |
| --- | --- |
| [THE_ORANGE_BOOK.md](THE_ORANGE_BOOK.md) | Reader's guide: why Orange exists, what has been built, and what remains open |
| [LANGUAGE_2026.md](LANGUAGE_2026.md) | Normative Orange 2026 grammar (accepted S2 / S3a syntax) |
| [SEMANTICS_2026.md](SEMANTICS_2026.md) | Normative typed-literal semantics (accepted S3a) |
| [compiler/README.md](../compiler/README.md) | Compiler commands, diagnostics, fixtures, and conformance runners |
| [algorithms/README.md](../algorithms/README.md) | Standards-sourced reference programs and published vectors |

## Proposed language slices (implemented, in owner review)

These define the S3b through S3t surface the compiler implements today. Their
OEPs remain in Review; implementation and tests are provisional evidence, not
acceptance.

| Slice | Specification | OEP |
| --- | --- | --- |
| S3b | [EXPRESSIONS_2026.md](EXPRESSIONS_2026.md) | [OEP-0005](governance/oeps/OEP-0005-orange-2026-pure-spec-expressions.md) |
| S3c | [BINDINGS_2026.md](BINDINGS_2026.md) | [OEP-0006](governance/oeps/OEP-0006-orange-2026-bindings-and-conversions.md) |
| S3d | [ARRAYS_2026.md](ARRAYS_2026.md) | [OEP-0007](governance/oeps/OEP-0007-orange-2026-fixed-length-arrays.md) |
| S3e | [LOOPS_2026.md](LOOPS_2026.md) | [OEP-0008](governance/oeps/OEP-0008-orange-2026-bounded-loops.md) |
| S3f | [CONDITIONS_2026.md](CONDITIONS_2026.md) | [OEP-0009](governance/oeps/OEP-0009-orange-2026-conditions.md) |
| S3g | [LOOKUPS_2026.md](LOOKUPS_2026.md) | [OEP-0010](governance/oeps/OEP-0010-orange-2026-lookups.md) |
| S3h | [MODULES_2026.md](MODULES_2026.md) | [OEP-0011](governance/oeps/OEP-0011-orange-2026-modules.md) |
| S3i | [MODULAR_2026.md](MODULAR_2026.md) | [OEP-0012](governance/oeps/OEP-0012-orange-2026-modular-arithmetic.md) |
| S3j | [BLOCKS_2026.md](BLOCKS_2026.md) | [OEP-0013](governance/oeps/OEP-0013-orange-2026-blocks.md) |
| S3k | [TUPLES_2026.md](TUPLES_2026.md) | [OEP-0014](governance/oeps/OEP-0014-orange-2026-tuples.md) |
| S3l | [BYTES_2026.md](BYTES_2026.md) | [OEP-0015](governance/oeps/OEP-0015-orange-2026-bytes.md) |
| S3m | [SIZES_2026.md](SIZES_2026.md) | [OEP-0016](governance/oeps/OEP-0016-orange-2026-sizes.md) |
| S3n | [ORDER_2026.md](ORDER_2026.md) | [OEP-0017](governance/oeps/OEP-0017-orange-2026-byte-order.md) |
| S3o | [TYPE_PARAMETERS_2026.md](TYPE_PARAMETERS_2026.md) | [OEP-0018](governance/oeps/OEP-0018-orange-2026-type-parameters.md) |
| S3p | [LENGTHS_2026.md](LENGTHS_2026.md) | [OEP-0019](governance/oeps/OEP-0019-orange-2026-lengths.md) |
| S3q | [TESTS_2026.md](TESTS_2026.md) | [OEP-0020](governance/oeps/OEP-0020-orange-2026-tests.md) |
| S3r | [AMOUNTS_2026.md](AMOUNTS_2026.md) | [OEP-0021](governance/oeps/OEP-0021-orange-2026-computed-amounts.md) |
| S3s | [NESTED_ARRAYS_2026.md](NESTED_ARRAYS_2026.md) | [OEP-0023](governance/oeps/OEP-0023-orange-2026-nested-arrays.md) |
| S3t | [STATIC_MODULI_2026.md](STATIC_MODULI_2026.md) | [OEP-0024](governance/oeps/OEP-0024-orange-2026-static-moduli.md) |

The implemented tip is **S3t**. OEP-0022 is a draft informational development
plan, not a language-slice acceptance record.

## Frontend tool contracts

These are permanent W3 tooling boundaries. They do not accept semantic OEPs
or close S8.

| Document | Command | Boundary |
| --- | --- | --- |
| [FORMATTER_2026.md](FORMATTER_2026.md) | `orangec fmt` | Syntax-only layout; preserves token spellings and comment bytes |
| [DOCUMENTATION_2026.md](DOCUMENTATION_2026.md) | `orangec doc` | Syntax-only offline HTML for one parsed source |
| [WITNESS_REPLAY_2026.md](WITNESS_REPLAY_2026.md) | `orangec replay` | Typed local arguments and one Boolean witness execution |

## Product direction, decisions, and governance

| Document | Role |
| --- | --- |
| [ROADMAP.md](ROADMAP.md) | Dependency-ordered solo capability stages |
| [DECISIONS.md](DECISIONS.md) | Decision register (D-IDs) |
| [RELEASE_1_0_EXECUTION.md](RELEASE_1_0_EXECUTION.md) | Complete 1.0 execution map under owner direction |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Intended end-state architecture (mostly aspirational) |
| [ASSURANCE.md](ASSURANCE.md) | Proposed assurance and claim model |
| [PROJECT_CHARTER.md](PROJECT_CHARTER.md) | Charter and scope |
| [RESEARCH.md](RESEARCH.md) | Landscape relative to verified-cryptography work |
| [USER_JOURNEYS.md](USER_JOURNEYS.md) | Intended user journeys |
| [GATE0_TRACEABILITY.md](GATE0_TRACEABILITY.md) | Historical Gate 0 requirement mapping under solo bootstrap |
| [governance/oeps/README.md](governance/oeps/README.md) | Orange Enhancement Proposal process and index |
| [governance/adrs/README.md](governance/adrs/README.md) | Architecture Decision Record process |

## Security and operations

| Document | Role |
| --- | --- |
| [security/THREAT_MODEL.md](security/THREAT_MODEL.md) | Solo-bootstrap threat and control inventory |
| [security/OSPS_BASELINE.md](security/OSPS_BASELINE.md) | Open Source Project Security baseline evidence |
| [security/SECRETS_AND_INCIDENTS.md](security/SECRETS_AND_INCIDENTS.md) | Secrets handling and incident response |
| [operations/CI_DEPENDENCIES.md](operations/CI_DEPENDENCIES.md) | CI dependency inventory |
| [operations/GITHUB_CONTROLS.md](operations/GITHUB_CONTROLS.md) | GitHub control-plane records |

## Related materials outside `docs/`

| Path | Role |
| --- | --- |
| [`../GOVERNANCE.md`](../GOVERNANCE.md) | Solo governance authority |
| [`../CONTRIBUTING.md`](../CONTRIBUTING.md) | Contribution boundary under unresolved D-018 |
| [`../policy/README.md`](../policy/README.md) | Repository policy and gate sandbox |
| [`../tabula/README.md`](../tabula/README.md) | Local workbench (separate from the language) |
| [`../compiler/schemes/README.md`](../compiler/schemes/README.md) | Reference sealing schemes and file format 1 |
| [`../research/decisions/`](../research/decisions/) | Decision laboratories and epoch evidence |

## How to keep this index honest

When a language slice, tool contract, or top-level guide is added, update this
file in the same change. Do not describe proposed OEPs as accepted, and do not
treat algorithms, fixtures, or passing tests as proof, independent review, or
release readiness.
