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
    pub fn has_errors(&self) -> bool {
        self.values.is_none()
    }
}

/// Evaluates every typed Core function in source order.
#[must_use]
pub fn evaluate(core: &CoreModule) -> EvaluationResult {
    evaluate_with_limit(core, MAX_EVALUATION_STEPS_PER_SOURCE)
}

fn evaluate_with_limit(core: &CoreModule, step_limit: usize) -> EvaluationResult {
    let mut values = Vec::with_capacity(core.functions.len().min(step_limit));
    let mut shared_module = None;
    for (steps, function) in core.functions.iter().enumerate() {
        if steps >= step_limit {
            return EvaluationResult {
                values: None,
                diagnostics: vec![
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
                ],
            };
        }
        let module = Arc::clone(shared_module.get_or_insert_with(|| Arc::from(core.name.as_str())));
        values.push(EvaluatedFunction {
            id: function.id,
            module,
            name: function.name.clone(),
            value: function.value.clone(),
        });
    }
    EvaluationResult {
        values: Some(values),
        diagnostics: Vec::new(),
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
        let result = evaluate_with_limit(&core, 1);
        assert!(result.has_errors());
        assert!(result.values().is_none());
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(
            result.diagnostics()[0].code(),
            DiagnosticCode::EvaluationResourceLimit
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
