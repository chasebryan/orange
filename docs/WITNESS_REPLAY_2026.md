# Orange 2026 local witness replay contract

Status: permanent pre-alpha reference tooling under owner direction;
no semantic acceptance, solver selection, claim authority or release is recorded here

Edition: `2026`

Snapshot: 2026-10-02

This tool decodes a local argument file against a checked function's concrete
parameter types and reference-evaluates its Boolean result. The implemented
language marker remains S3x. Its result describes one supplied witness under
that evaluator; it is not a proof or a product atomic claim.

## Command and checked source

```text
orangec replay --function MODULE::NAME [--instance N[,N...]] --witness FILE|- [--steps N] [--stats] SOURCE|-
```

Exactly one source root and one witness operand are required. At most one of
those operands may read standard input. `--edition 2026` and `--` retain their
ordinary meanings. There is no output-file or write mode. `--spec` keeps its
existing `eval` meaning and does not select replay functions.

The source and its imported modules are fully read and must pass complete
lexical, syntactic and semantic validation before the selected function is
checked and the witness operand is read. `--function` selects a qualified checked
specification whose concrete return type is `Bool`. The optional numeric
`--instance` vector names every declared size/type-domain position in the
selected instance's Core metadata, in declaration order: size entries are
the actual admitted size values and type entries are zero-based positions in
the declared finite type lists. It does not use source-spelled type labels.
Omitting `--instance` selects only an empty vector: every size/type-parameter
function requires its complete numeric vector, even for a singleton domain.
The selected function must be the exact function object at its identifier in
the evaluator's checked Core. Decoded arguments must exactly match that
function's parameter types before the evaluator is called. The argument
vector carries no separate Core identity. Named test entries are not replay
functions.

## Argument file

The complete file is an outer argument list `[VALUE, VALUE]`, or `[]` for no
arguments, with at most one final LF. Each value must have its exact current
`CoreValue` display spelling. The outer list is an argument envelope, not a
source expression or a type inference context.

| Concrete parameter type | Value spelling |
| --- | --- |
| `Int` | Canonical signed decimal: `0` or an optional minus followed by a nonzero first digit and decimal digits. |
| `Bool` | `true` or `false`. |
| `Word[8]`, `Word[16]`, `Word[32]`, `Word[64]` | `0x` followed by exactly 2, 4, 8 or 16 lowercase hexadecimal digits, including leading zeroes. |
| `Mod[m]` | Canonical nonnegative decimal strictly below the exact checked modulus `m`. |
| Array or rectangular matrix | Brackets with exactly the checked number of elements/rows, recursively separated by comma and one space. |
| Tuple | Parentheses with exactly the checked fields, recursively separated by comma and one space. |

Whitespace, signs, case, delimiters and digit counts are part of this local
encoding. There are no comments, expressions, byte strings, fill literals,
alias names, implicit conversions or modular reduction. For example, the
argument list for parameters `Bool`, `Word[16]`, `Mod[7]` is
`[true, 0x000a, 6]`. The same decimal text may inhabit different residue types;
the checked parameter type supplies the exact domain. Rank, every array axis,
tuple fields, word width and modulus are checked, rather than inferred from
text.

This is a typed local value encoding for the current reference frontend. It
is not a selected solver model format, canonical Core encoding, signed
witness schema or cross-revision source/proof/evidence identity.

## Result and failure boundary

A completed Boolean result is `Falsified` when the function returns `false`,
or `HoldsForThisWitness` when it returns `true`. Both completed outcomes use
exit status 0. Neither establishes a universal property, a solver's correctness,
a checked implementation refinement or an authoritative atomic claim. The
complete stdout record identifies the function, numeric instance metadata,
actual argument values and their exact parameter types. The emitted outcome
spellings are `falsified` and `holds_for_this_witness`. The output has three
lines, for example:

```text
m::p[1, 0]: holds_for_this_witness
parameter_types: [Int, Mod[7]]
arguments: [7, 6]
```

The numeric instance is always present; a function with no size/type parameters
uses `[]`. `--stats` reports the qualified numeric instance's step count and
`total: N of B steps` on standard error. The individual count uses `step`
for one step.

The existing evaluator performs the call under its bounded steps, call depth
and integer magnitude rules. `--steps` retains the existing range from 1
through 1,073,741,824, with default 1,048,576. A decode, binding, evaluation or
transport failure returns status 1 with diagnostics; invalid usage returns 2.
A failed replay emits no completed outcome. Complete result construction
precedes stdout writing; a detected host write or flush failure can leave an
accepted prefix and returns status 1.

| Diagnostic | Meaning |
| --- | --- |
| `ORC0270` | Argument value is not canonically encoded. |
| `ORC0271` | Argument value/count does not match the checked parameter type. |
| `ORC0272` | Argument decoding resource limit. |
| `ORC0273` | Invalid witness replay binding. |
| `ORC0274` | Inconsistent witness replay construction. |
| `ORC0301` | Existing reference-evaluation resource exhaustion. |

Existing source, input, UTF-8 and host-output diagnostics retain their meaning.

## Resource limits

Each witness is a UTF-8 `SourceFile` of at most 16 MiB
(`16 * 1024 * 1024` witness bytes). Witness bytes share the CLI's existing
64 MiB invocation input envelope with the source root and its imported modules.
The decoder admits at most 4,194,304 value nodes, at most 4,194,304 retained
binary integer limbs, and at most 67,108,864 decoding work items. The existing
16,384-bit integer bound and concrete type/shape limits remain unchanged.
The public ceilings are `MAX_ARGUMENT_VALUE_NODES`,
`MAX_ARGUMENT_INTEGER_LIMBS` and `MAX_ARGUMENT_DECODE_WORK`. Scanning, value
transitions and integer construction consume checked work;
the required aggregate node count is checked from the parameter types before
decoded-value allocation. Budgets are shared across all arguments. Resource
failure returns no partial decoded argument list or completed replay result.

## Regression evidence

The [argument decoder regressions](../compiler/crates/orange-compiler/src/arguments.rs)
include `aggregate_rows_and_tuple_fields_keep_exact_shapes_and_moduli`,
`rejects_noncanonical_partial_and_trailing_values_without_artifacts`,
`exact_node_limb_and_work_budgets_fail_closed`, and
`maximum_individual_tuple_matrix_shape_is_admitted_and_preserved`. Separate
cases cover decimal/word boundaries, original error spans, the shared aggregate
budget, allocation failure and bounded stack use.

The [replayer regressions](../compiler/crates/orange-compiler/src/witness.rs)
include `replay_records_exact_false_and_true_observations_without_global_claims`,
`foreign_equal_core_same_dense_id_is_rejected_before_evaluation`,
`exact_steps_one_short_zero_and_recovery_preserve_original_evaluator_failure`,
and `finite_size_type_modulus_and_linked_instances_replay_exactly`. These cover
exact ownership, completed observations and original evaluator failure states.

The [CLI harness](../compiler/crates/orangec/tests/witness_replay.rs) includes
`replay_selects_complete_linked_numeric_instances`,
`replay_metadata_never_uses_source_spelled_instance_comments`,
`replay_rejects_noncanonical_packets_and_values_without_stdout`, and
`replay_steps_and_stats_describe_only_completed_output`. Other cases cover
scalar/aggregate domains, one stdin role, paths and rejected usage/input.
CLI unit regressions include
`replay_preflights_complete_stdout_and_buffers_fallibly`,
`replay_charges_source_imports_and_witness_to_one_envelope`, and
`replay_stats_follow_committed_output_and_transport_failures_stop_them`. They
check complete-output preflight, shared input limits and host failures.
These are reference-tool regressions, not claim or decision acceptance.

## Product and decision status

The [architecture](ARCHITECTURE.md), [roadmap](ROADMAP.md) and
[complete 1.0 execution record](RELEASE_1_0_EXECUTION.md) retain the wider
claim and replay obligations. This tool supplies concrete reference replay
without selecting a solver or accepting S3 semantics. It gives no D-009
candidate-case execution credit: that suite still requires its actual
candidate-specific runs and exact protocol evidence. It changes no frozen
evidence, owner assurance task, acceptance record or decision authority.
