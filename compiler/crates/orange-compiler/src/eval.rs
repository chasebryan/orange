//! Deterministic reference evaluation for typed Orange Core.

use std::fmt;

use crate::core::{CoreFunctionId, CoreModule, CoreType, CoreValue};
use crate::diagnostic::{Diagnostic, DiagnosticCode};

/// Maximum reference-evaluation steps performed for one source module.
pub const MAX_EVALUATION_STEPS_PER_SOURCE: usize = 1_048_576;

/// One evaluated function result in deterministic Core source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluatedFunction {
    /// Identity copied from the source-ordered Core function.
    pub id: CoreFunctionId,
    /// Exact ASCII module name.
    pub module: String,
    /// Exact ASCII function name.
    pub name: String,
    /// Statically checked result type.
    pub result_type: CoreType,
    /// Exact evaluated value.
    pub value: CoreValue,
}

impl fmt::Display for EvaluatedFunction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}::{}: {} = {}",
            self.module, self.name, self.result_type, self.value
        )
    }
}

/// The complete result of reference evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationResult {
    /// Results in source order, present only when evaluation produced no diagnostics.
    pub values: Option<Vec<EvaluatedFunction>>,
    /// Evaluation-resource diagnostics in deterministic source order.
    pub diagnostics: Vec<Diagnostic>,
}

impl EvaluationResult {
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
    let mut values = Vec::with_capacity(core.functions.len());
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
        values.push(EvaluatedFunction {
            id: function.id,
            module: core.name.clone(),
            name: function.name.clone(),
            result_type: function.result_type,
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
        assert_eq!(parsed.diagnostics, []);
        let analyzed = analyze(source, parsed.ast.as_ref().unwrap());
        assert_eq!(analyzed.diagnostics, []);
        analyzed.core.unwrap()
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
        assert_eq!(result.diagnostics, []);
        let rendered: Vec<_> = result
            .values
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
        assert_eq!(result.values, Some(Vec::new()));
        assert_eq!(result.diagnostics, []);
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
        assert!(result.values.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::EvaluationResourceLimit
        );
    }

    #[test]
    fn evaluation_is_repeatable() {
        let core = core("edition 2026; module values { spec answer() -> Int { 42 } }\n");
        assert_eq!(evaluate(&core), evaluate(&core));
    }
}
