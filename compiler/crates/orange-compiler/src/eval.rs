//! Deterministic reference evaluation for typed Orange Core.

use std::fmt;
use std::sync::Arc;

use crate::core::{CoreFunctionId, CoreModule, CoreType, CoreValue};
use crate::diagnostic::{Diagnostic, DiagnosticCode};

/// Maximum reference-evaluation steps performed for one source module.
pub const MAX_EVALUATION_STEPS_PER_SOURCE: usize = 1_048_576;

/// One evaluated function result in deterministic Core source order.
///
/// Evaluated values are read-only outside this crate so their source identity,
/// order, and checked type cannot be rewritten after evaluation.
///
/// ```compile_fail
/// use orange_compiler::{CoreType, EvaluatedFunction};
///
/// fn forge_type(value: &mut EvaluatedFunction) {
///     value.result_type = CoreType::Word8;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluatedFunction {
    /// Identity copied from the source-ordered Core function.
    id: CoreFunctionId,
    /// Exact ASCII module name, shared by results from the same module.
    module: Arc<str>,
    /// Exact ASCII function name.
    name: String,
    /// Exact evaluated value.
    value: CoreValue,
}

impl EvaluatedFunction {
    /// Returns the source-ordered Core function identity.
    #[must_use]
    pub const fn id(&self) -> CoreFunctionId {
        self.id
    }

    /// Returns the exact ASCII module name.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }

    /// Returns the exact ASCII function name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the statically checked result type.
    #[must_use]
    pub const fn result_type(&self) -> CoreType {
        self.value.ty()
    }

    /// Returns the exact evaluated value.
    #[must_use]
    pub const fn value(&self) -> &CoreValue {
        &self.value
    }
}

impl fmt::Display for EvaluatedFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}::{}: {} = {}",
            self.module(),
            self.name(),
            self.result_type(),
            self.value()
        )
    }
}

/// The complete result of reference evaluation.
///
/// ```compile_fail
/// use orange_compiler::EvaluationResult;
///
/// fn replace_values(result: &mut EvaluationResult) {
///     result.values = None;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationResult {
    /// Results in source order, present only when evaluation produced no diagnostics.
    values: Option<Vec<EvaluatedFunction>>,
    /// Evaluation-resource diagnostics in deterministic source order.
    diagnostics: Vec<Diagnostic>,
}

impl EvaluationResult {
    /// Returns results in source order, or `None` after evaluation failure.
    #[must_use]
    pub fn values(&self) -> Option<&[EvaluatedFunction]> {
        self.values.as_deref()
    }

    /// Returns evaluation-resource diagnostics in deterministic source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes this result and returns its complete value set, if produced.
    #[must_use]
    pub fn into_values(self) -> Option<Vec<EvaluatedFunction>> {
        self.values
    }

    /// Returns whether reference evaluation did not produce a complete value set.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.values.is_none()
    }
}

/// Evaluates every typed Core function in source order.
#[must_use]
pub fn evaluate(core: &CoreModule) -> EvaluationResult {
    evaluate_with_limit(core, MAX_EVALUATION_STEPS_PER_SOURCE)
}

fn evaluate_with_limit(core: &CoreModule, step_limit: usize) -> EvaluationResult {
    evaluate_with_limit_and_reservation(core, step_limit, |values, capacity| {
        values.try_reserve_exact(capacity).is_ok()
    })
}

fn evaluate_with_limit_and_reservation(
    core: &CoreModule,
    step_limit: usize,
    reserve_values: impl FnOnce(&mut Vec<EvaluatedFunction>, usize) -> bool,
) -> EvaluationResult {
    evaluate_with_reservations(core, step_limit, reserve_values, Reservations::DEFAULT)
}

#[derive(Clone, Copy)]
struct Reservations {
    name: fn(&mut String, usize) -> bool,
    value_limbs: fn(&mut Vec<u32>, usize) -> bool,
    diagnostics: fn(&mut Vec<Diagnostic>, usize) -> bool,
}

impl Reservations {
    const DEFAULT: Self = Self {
        name: reserve_name,
        value_limbs: reserve_value_limbs,
        diagnostics: reserve_diagnostics,
    };
}

fn reserve_name(name: &mut String, bytes: usize) -> bool {
    name.try_reserve_exact(bytes).is_ok()
}

fn reserve_value_limbs(limbs: &mut Vec<u32>, count: usize) -> bool {
    limbs.try_reserve_exact(count).is_ok()
}

fn reserve_diagnostics(diagnostics: &mut Vec<Diagnostic>, count: usize) -> bool {
    diagnostics.try_reserve_exact(count).is_ok()
}

fn evaluate_with_reservations(
    core: &CoreModule,
    step_limit: usize,
    reserve_values: impl FnOnce(&mut Vec<EvaluatedFunction>, usize) -> bool,
    reservations: Reservations,
) -> EvaluationResult {
    let mut diagnostics = Vec::new();
    if !(reservations.diagnostics)(&mut diagnostics, 1) {
        return EvaluationResult {
            values: None,
            diagnostics,
        };
    }
    let capacity = core.functions.len().min(step_limit);
    let mut values = Vec::new();
    if !reserve_values(&mut values, capacity) {
        return evaluation_failure(
            diagnostics,
            Diagnostic::error(
                DiagnosticCode::EvaluationResourceLimit,
                "reference evaluation value-set allocation failed",
                core.span,
            )
            .with_label("complete value set could not be reserved")
            .with_note("no partial value set is returned"),
        );
    }
    let mut shared_module = None;
    for (steps, function) in core.functions.iter().enumerate() {
        if steps >= step_limit {
            return evaluation_failure(
                diagnostics,
                Diagnostic::error(
                    DiagnosticCode::EvaluationResourceLimit,
                    "reference evaluation step limit exceeded",
                    function.name_span,
                )
                .with_label("evaluation stopped before this function")
                .with_note(format!(
                    "at most {step_limit} function evaluation steps are permitted"
                ))
                .with_note("no partial value set is returned"),
            );
        }
        let mut name = String::new();
        if !(reservations.name)(&mut name, function.name.len()) {
            return allocation_failure(
                diagnostics,
                function.name_span,
                "evaluated function name storage could not be reserved",
            );
        }
        name.push_str(&function.name);
        let Some(value) = function
            .value
            .try_clone_with_reservation(reservations.value_limbs)
        else {
            return allocation_failure(
                diagnostics,
                function.name_span,
                "evaluated exact integer storage could not be reserved",
            );
        };
        let module = Arc::clone(shared_module.get_or_insert_with(|| Arc::from(core.name.as_str())));
        values.push(EvaluatedFunction {
            id: function.id,
            module,
            name,
            value,
        });
    }
    EvaluationResult {
        values: Some(values),
        diagnostics,
    }
}

fn allocation_failure(
    diagnostics: Vec<Diagnostic>,
    span: crate::source::Span,
    label: &'static str,
) -> EvaluationResult {
    evaluation_failure(
        diagnostics,
        Diagnostic::error(
            DiagnosticCode::EvaluationResourceLimit,
            "reference evaluation result allocation failed",
            span,
        )
        .with_label(label)
        .with_note("no partial value set is returned"),
    )
}

fn evaluation_failure(
    mut diagnostics: Vec<Diagnostic>,
    diagnostic: Diagnostic,
) -> EvaluationResult {
    diagnostics.push(diagnostic);
    EvaluationResult {
        values: None,
        diagnostics,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::Edition;
    use crate::lexer::lex;
    use crate::parser::parse;
    use crate::semantics::analyze;
    use crate::source::SourceMap;

    fn core(text: &str) -> CoreModule {
        let mut sources = SourceMap::new();
        let id = sources.add("evaluate.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        assert_eq!(lexed.diagnostics(), []);
        let parsed = parse(source, &lexed);
        assert_eq!(parsed.diagnostics(), []);
        let analyzed = analyze(source, parsed.ast().unwrap());
        assert_eq!(analyzed.diagnostics(), []);
        analyzed.into_core().unwrap()
    }

    #[test]
    fn production_evaluation_limit_matches_the_s3a_specification() {
        assert_eq!(MAX_EVALUATION_STEPS_PER_SOURCE, 1_048_576);
    }

    #[test]
    fn evaluates_all_values_in_source_order_with_stable_display() {
        let core = core(concat!(
            "edition 2026; module values {\n",
            "  spec negative() -> Int { -12345678901234567890 }\n",
            "  spec low() -> Word[8] { 10 }\n",
            "  spec high() -> Word[8] { 255 }\n",
            "}\n",
        ));
        let result = evaluate(&core);
        assert_eq!(result.diagnostics(), []);
        let rendered: Vec<_> = result
            .values()
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(
            rendered,
            [
                "values::negative: Int = -12345678901234567890",
                "values::low: Word[8] = 0x0a",
                "values::high: Word[8] = 0xff",
            ]
        );
    }

    #[test]
    fn empty_core_evaluates_to_an_empty_value_set() {
        let core = core("edition 2026; module values { spec empty() {} }\n");
        let result = evaluate(&core);
        assert_eq!(result.values().unwrap(), []);
        assert_eq!(result.diagnostics(), []);
    }

    #[test]
    fn evaluation_limit_fails_without_partial_values() {
        let core = core(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Int { 1 }\n",
            "  spec second() -> Int { 2 }\n",
            "}\n",
        ));
        let first = evaluate_with_limit(&core, 1);
        let second = evaluate_with_limit(&core, 1);
        assert_eq!(first, second);
        assert!(first.has_errors());
        assert!(first.values().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        assert_eq!(first.diagnostics.capacity(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::EvaluationResourceLimit);
        assert_eq!(diagnostic.primary_span(), core.functions[1].name_span);
        assert_eq!(
            diagnostic.message(),
            "reference evaluation step limit exceeded"
        );
        assert_eq!(
            diagnostic.label(),
            "evaluation stopped before this function"
        );
        assert_eq!(
            diagnostic.notes(),
            &[
                "at most 1 function evaluation steps are permitted",
                "no partial value set is returned",
            ]
        );

        let first = evaluate_with_limit(&core, 2);
        let second = evaluate_with_limit(&core, 2);
        assert_eq!(first, second);
        assert_eq!(first.diagnostics(), []);
        assert_eq!(first.values().unwrap().len(), 2);
    }

    #[test]
    fn evaluation_limit_stops_before_late_name_or_value_allocations() {
        let core = core(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Word[8] { 1 }\n",
            "  spec must_not_copy() -> Int { 2 }\n",
            "}\n",
        ));
        let result = evaluate_with_reservations(
            &core,
            1,
            |values, capacity| values.try_reserve_exact(capacity).is_ok(),
            Reservations {
                name: |name, bytes| {
                    bytes != "must_not_copy".len() && name.try_reserve_exact(bytes).is_ok()
                },
                value_limbs: |_, _| false,
                ..Reservations::DEFAULT
            },
        );

        assert!(result.values().is_none());
        let [diagnostic] = result.diagnostics() else {
            panic!("the step limit must produce exactly one diagnostic");
        };
        assert_eq!(diagnostic.code(), DiagnosticCode::EvaluationResourceLimit);
        assert_eq!(
            diagnostic.message(),
            "reference evaluation step limit exceeded"
        );
        assert_eq!(diagnostic.primary_span(), core.functions[1].name_span);
    }

    #[test]
    fn diagnostic_slot_reservation_failure_returns_no_values_or_diagnostics() {
        let core = core("edition 2026; module values { spec answer() -> Int { 42 } }\n");
        let result = evaluate_with_reservations(
            &core,
            MAX_EVALUATION_STEPS_PER_SOURCE,
            |values, capacity| values.try_reserve_exact(capacity).is_ok(),
            Reservations {
                diagnostics: |_, _| false,
                ..Reservations::DEFAULT
            },
        );

        assert!(result.has_errors());
        assert!(result.values().is_none());
        assert!(result.diagnostics().is_empty());
        assert_eq!(result.diagnostics.capacity(), 0);
    }

    #[test]
    fn value_set_reservation_failure_returns_no_partial_values() {
        let core = core(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Int { 1 }\n",
            "  spec second() -> Int { 2 }\n",
            "}\n",
        ));
        let first = evaluate_with_limit_and_reservation(&core, 2, |_, capacity| {
            assert_eq!(capacity, 2);
            false
        });
        let second = evaluate_with_limit_and_reservation(&core, 2, |_, _| false);

        assert_eq!(first, second);
        assert!(first.has_errors());
        assert!(first.values().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::EvaluationResourceLimit);
        assert_eq!(diagnostic.primary_span(), core.span);
        assert_eq!(
            diagnostic.message(),
            "reference evaluation value-set allocation failed"
        );
        assert_eq!(
            diagnostic.label(),
            "complete value set could not be reserved"
        );
        assert_eq!(diagnostic.notes(), &["no partial value set is returned"]);
    }

    #[test]
    fn function_name_reservation_failure_returns_no_partial_values() {
        let core = core("edition 2026; module values { spec answer() -> Int { 42 } }\n");
        let evaluate_with_failure = || {
            evaluate_with_reservations(
                &core,
                MAX_EVALUATION_STEPS_PER_SOURCE,
                |values, capacity| values.try_reserve_exact(capacity).is_ok(),
                Reservations {
                    name: |_, _| false,
                    ..Reservations::DEFAULT
                },
            )
        };

        let first = evaluate_with_failure();
        let second = evaluate_with_failure();
        assert_eq!(first, second);
        assert!(first.values().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::EvaluationResourceLimit);
        assert_eq!(diagnostic.primary_span(), core.functions[0].name_span);
        assert_eq!(
            diagnostic.label(),
            "evaluated function name storage could not be reserved"
        );
        assert_eq!(diagnostic.notes(), &["no partial value set is returned"]);
    }

    #[test]
    fn exact_value_reservation_failure_returns_no_partial_values() {
        let core = core("edition 2026; module values { spec answer() -> Int { 42 } }\n");
        let evaluate_with_failure = || {
            evaluate_with_reservations(
                &core,
                MAX_EVALUATION_STEPS_PER_SOURCE,
                |values, capacity| values.try_reserve_exact(capacity).is_ok(),
                Reservations {
                    value_limbs: |_, _| false,
                    ..Reservations::DEFAULT
                },
            )
        };

        let first = evaluate_with_failure();
        let second = evaluate_with_failure();
        assert_eq!(first, second);
        assert!(first.values().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::EvaluationResourceLimit);
        assert_eq!(diagnostic.primary_span(), core.functions[0].name_span);
        assert_eq!(
            diagnostic.label(),
            "evaluated exact integer storage could not be reserved"
        );
        assert_eq!(diagnostic.notes(), &["no partial value set is returned"]);
    }

    #[test]
    fn late_allocation_failures_discard_completed_values() {
        let name_core = core(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Word[8] { 1 }\n",
            "  spec second_name() -> Int { 2 }\n",
            "}\n",
        ));
        let name_failure = evaluate_with_reservations(
            &name_core,
            MAX_EVALUATION_STEPS_PER_SOURCE,
            |values, capacity| values.try_reserve_exact(capacity).is_ok(),
            Reservations {
                name: |name, bytes| {
                    bytes != "second_name".len() && name.try_reserve_exact(bytes).is_ok()
                },
                ..Reservations::DEFAULT
            },
        );

        assert!(name_failure.values().is_none());
        assert_eq!(name_failure.diagnostics().len(), 1);
        assert_eq!(
            name_failure.diagnostics()[0].primary_span(),
            name_core.functions[1].name_span
        );
        assert_eq!(
            name_failure.diagnostics()[0].label(),
            "evaluated function name storage could not be reserved"
        );

        let value_core = core(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Word[8] { 1 }\n",
            "  spec second() -> Int { 2 }\n",
            "}\n",
        ));
        let value_failure = evaluate_with_reservations(
            &value_core,
            MAX_EVALUATION_STEPS_PER_SOURCE,
            |values, capacity| values.try_reserve_exact(capacity).is_ok(),
            Reservations {
                value_limbs: |_, _| false,
                ..Reservations::DEFAULT
            },
        );

        assert!(value_failure.values().is_none());
        assert_eq!(value_failure.diagnostics().len(), 1);
        assert_eq!(
            value_failure.diagnostics()[0].primary_span(),
            value_core.functions[1].name_span
        );
        assert_eq!(
            value_failure.diagnostics()[0].label(),
            "evaluated exact integer storage could not be reserved"
        );
    }

    #[test]
    fn evaluation_is_repeatable() {
        let core = core("edition 2026; module values { spec answer() -> Int { 42 } }\n");
        assert_eq!(evaluate(&core), evaluate(&core));
    }

    #[test]
    fn module_identity_is_shared_across_evaluated_values() {
        let core = core(concat!(
            "edition 2026; module a_very_long_shared_module_name {\n",
            "  spec first() -> Int { 1 }\n",
            "  spec second() -> Int { 2 }\n",
            "}\n",
        ));
        let result = evaluate(&core);
        let values = result.values().unwrap();
        assert!(Arc::ptr_eq(&values[0].module, &values[1].module));
    }
}
