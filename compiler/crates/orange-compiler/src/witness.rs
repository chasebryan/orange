//! Reference replay of a complete typed witness against one exact function.

use crate::arguments::DecodedArguments;
use crate::core::{CoreFunction, CoreType, CoreValue};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::eval::Evaluator;

/// An observation of one checked Boolean function on one supplied input.
///
/// These observations provide no proof, claim status, solver authority or
/// persisted obligation/model identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WitnessReplayOutcome {
    /// The exact function returned `false` for this witness.
    Falsified,
    /// The exact function returned `true` for this witness alone.
    HoldsForThisWitness,
    /// No Boolean observation was established.
    Failed,
}

/// The complete result of replaying one witness.
#[derive(Debug, Eq, PartialEq)]
pub struct WitnessReplayResult<'core> {
    function: &'core CoreFunction,
    outcome: WitnessReplayOutcome,
    steps: usize,
    diagnostics: ReplayDiagnostics,
}

#[derive(Debug, Eq, PartialEq)]
enum ReplayDiagnostics {
    None,
    Binding(Vec<Diagnostic>),
    Evaluation(crate::eval::CallResult),
}

impl<'core> WitnessReplayResult<'core> {
    /// Returns the exact function supplied for this replay, even on failure.
    #[must_use]
    pub const fn function(&self) -> &'core CoreFunction {
        self.function
    }
    /// Returns the Boolean observation or explicit failure.
    #[must_use]
    pub const fn outcome(&self) -> WitnessReplayOutcome {
        self.outcome
    }
    /// Returns steps with the original [`Evaluator::call`] accounting.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }
    /// Returns binding failures or the original evaluator diagnostics.
    ///
    /// Allocation failure may leave this slice empty; [`Self::has_errors`]
    /// still reports failure and no Boolean outcome is returned.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match &self.diagnostics {
            ReplayDiagnostics::None => &[],
            ReplayDiagnostics::Binding(diagnostics) => diagnostics,
            ReplayDiagnostics::Evaluation(result) => result.diagnostics(),
        }
    }
    /// Returns whether no Boolean observation was established.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        matches!(self.outcome, WitnessReplayOutcome::Failed)
    }
}

/// Replays complete arguments against the exact evaluator-owned function.
///
/// The function must belong by pointer identity to the evaluator's checked
/// module, must return `Bool`, must not be a known-answer test, and must have
/// exactly the decoded argument types. Every call has the supplied step limit;
/// zero allows no evaluation step. Linked functions and concrete finite
/// instances are admitted. A true observation proves nothing about other inputs.
#[must_use]
pub fn replay_witness<'core>(
    evaluator: &mut Evaluator<'core>,
    function: &'core CoreFunction,
    arguments: &DecodedArguments,
    step_limit: usize,
) -> WitnessReplayResult<'core> {
    let owned = usize::try_from(function.id().index())
        .ok()
        .and_then(|index| evaluator.module().functions().get(index));
    if !owned.is_some_and(|owned| std::ptr::eq(owned, function))
        || function.title().is_some()
        || function.result_type() != CoreType::Bool
        || function.parameters().len() != arguments.values().len()
        || function
            .parameters()
            .iter()
            .zip(arguments.values())
            .any(|(ty, value)| *ty != value.ty())
    {
        return failed(
            function,
            DiagnosticCode::InvalidWitnessReplayBinding,
            "witness replay requires the exact owned Boolean function and argument types",
            0,
        );
    }
    let Some(result) = evaluator.call(function, arguments.values(), step_limit) else {
        return failed(
            function,
            DiagnosticCode::WitnessReplayInconsistency,
            "witness replay lost its checked function binding",
            0,
        );
    };
    let steps = result.steps();
    if result.has_errors() {
        return WitnessReplayResult {
            function,
            outcome: WitnessReplayOutcome::Failed,
            steps,
            diagnostics: ReplayDiagnostics::Evaluation(result),
        };
    }
    match result.value() {
        Some(CoreValue::Bool(value)) => WitnessReplayResult {
            function,
            outcome: if *value {
                WitnessReplayOutcome::HoldsForThisWitness
            } else {
                WitnessReplayOutcome::Falsified
            },
            steps,
            diagnostics: ReplayDiagnostics::None,
        },
        _ => failed(
            function,
            DiagnosticCode::WitnessReplayInconsistency,
            "witness replay did not return its checked Boolean result",
            steps,
        ),
    }
}

fn failed<'core>(
    function: &'core CoreFunction,
    code: DiagnosticCode,
    message: &'static str,
    steps: usize,
) -> WitnessReplayResult<'core> {
    let mut diagnostics = Vec::new();
    if diagnostics.try_reserve_exact(1).is_ok() {
        diagnostics.push(
            Diagnostic::error(code, message, function.name_span())
                .with_note("no witness observation was produced"),
        );
    }
    WitnessReplayResult {
        function,
        outcome: WitnessReplayOutcome::Failed,
        steps,
        diagnostics: ReplayDiagnostics::Binding(diagnostics),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::arguments::decode_arguments;
    use crate::core::CoreModule;
    use crate::{Edition, SourceMap, analyze_program, lex, parse};

    fn program(texts: &[&str]) -> CoreModule {
        let mut sources = SourceMap::new();
        let mut ids = Vec::new();
        let mut asts = Vec::new();
        for (index, text) in texts.iter().enumerate() {
            let id = sources.add(format!("module-{index}.or"), *text).unwrap();
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::E2026);
            assert_eq!(lexed.diagnostics(), []);
            let parsed = parse(source, &lexed);
            assert_eq!(parsed.diagnostics(), [], "{text}");
            ids.push(id);
            asts.push(parsed.into_ast().unwrap());
        }
        let modules: Vec<_> = ids
            .iter()
            .zip(&asts)
            .map(|(id, ast)| (sources.get(*id).unwrap(), ast))
            .collect();
        let result = analyze_program(modules[0], &modules[1..]);
        assert_eq!(result.diagnostics(), []);
        result.into_core().unwrap()
    }

    fn arguments(function: &CoreFunction, text: &str) -> DecodedArguments {
        let mut sources = SourceMap::new();
        let id = sources.add("witness.values", text).unwrap();
        let decoded = decode_arguments(sources.get(id).unwrap(), function.parameters());
        assert_eq!(decoded.diagnostics(), [], "{text}");
        decoded.into_arguments().unwrap()
    }

    fn function<'core>(
        core: &'core CoreModule,
        module: &str,
        name: &str,
        sizes: &[u32],
    ) -> &'core CoreFunction {
        core.functions()
            .iter()
            .find(|function| {
                function.module() == module && function.name() == name && function.sizes() == sizes
            })
            .unwrap()
    }

    #[test]
    fn replay_records_exact_false_and_true_observations_without_global_claims() {
        let core =
            program(&["edition 2026; module m { spec equal(x: Int, y: Int) -> Bool { x == y } }"]);
        let function = function(&core, "m", "equal", &[]);
        let mut evaluator = Evaluator::new(&core).unwrap();
        let false_arguments = arguments(function, "[1, 2]");
        let false_result = replay_witness(&mut evaluator, function, &false_arguments, 100);
        assert_eq!(false_result.outcome(), WitnessReplayOutcome::Falsified);
        assert!(!false_result.has_errors());
        assert!(std::ptr::eq(false_result.function(), function));
        assert_eq!(false_result.diagnostics(), []);
        assert!(false_result.steps() > 0);
        let true_arguments = arguments(function, "[1, 1]");
        let true_result = replay_witness(&mut evaluator, function, &true_arguments, 100);
        assert_eq!(
            true_result.outcome(),
            WitnessReplayOutcome::HoldsForThisWitness
        );
        assert_eq!(true_result.steps(), false_result.steps());
    }

    #[test]
    fn foreign_equal_core_same_dense_id_is_rejected_before_evaluation() {
        let core = program(&["edition 2026; module m { spec p(x: Int) -> Bool { x == 0 } }"]);
        let clone = core.clone();
        let owned = function(&core, "m", "p", &[]);
        let foreign = function(&clone, "m", "p", &[]);
        assert_eq!(foreign.id(), owned.id());
        assert_eq!(foreign, owned);
        let arguments = arguments(owned, "[0]");
        let mut evaluator = Evaluator::new(&core).unwrap();
        let result = replay_witness(&mut evaluator, foreign, &arguments, 100);
        assert_eq!(result.outcome(), WitnessReplayOutcome::Failed);
        assert_eq!(result.steps(), 0);
        assert_eq!(
            result.diagnostics()[0].code(),
            DiagnosticCode::InvalidWitnessReplayBinding
        );
        assert_eq!(
            replay_witness(&mut evaluator, owned, &arguments, 100).outcome(),
            WitnessReplayOutcome::HoldsForThisWitness
        );
    }

    #[test]
    fn excludes_tests_non_boolean_results_wrong_counts_and_cross_modulus_arguments() {
        let core = program(&[concat!(
            "edition 2026; module m { ",
            "spec value(x: Mod[7]) -> Mod[7] { x } ",
            "spec p(x: Mod[7]) -> Bool { x == 0 } ",
            "spec q(x: Mod[11]) -> Bool { x == 0 } ",
            "test \"a known answer\" { true } }"
        )]);
        let mut evaluator = Evaluator::new(&core).unwrap();
        let p = function(&core, "m", "p", &[]);
        let q = function(&core, "m", "q", &[]);
        let q_arguments = arguments(q, "[0]");
        let p_arguments = arguments(p, "[0]");
        let test = &core.tests()[0];
        let empty = arguments(test, "[]");
        for (function, arguments) in [
            (p, &q_arguments),
            (p, &empty),
            (test, &empty),
            (function(&core, "m", "value", &[]), &p_arguments),
        ] {
            let result = replay_witness(&mut evaluator, function, arguments, 100);
            assert!(result.has_errors());
            assert_eq!(result.steps(), 0);
            assert_eq!(
                result.diagnostics()[0].code(),
                DiagnosticCode::InvalidWitnessReplayBinding
            );
        }
    }

    #[test]
    fn exact_steps_one_short_zero_and_recovery_preserve_original_evaluator_failure() {
        let core = program(&[
            "edition 2026; module m { spec p(x: Int, y: Int) -> Bool { (x * x) == y } }",
        ]);
        let function = function(&core, "m", "p", &[]);
        let arguments = arguments(function, "[7, 49]");
        let mut evaluator = Evaluator::new(&core).unwrap();
        let steps = replay_witness(&mut evaluator, function, &arguments, 100).steps();
        assert!(steps > 1);
        let exact = replay_witness(&mut evaluator, function, &arguments, steps);
        assert_eq!(exact.outcome(), WitnessReplayOutcome::HoldsForThisWitness);
        assert_eq!(exact.steps(), steps);
        for limit in [0, steps - 1] {
            let original = evaluator.call(function, arguments.values(), limit).unwrap();
            let result = replay_witness(&mut evaluator, function, &arguments, limit);
            assert!(result.has_errors());
            assert_eq!(result.steps(), original.steps());
            assert_eq!(result.diagnostics(), original.diagnostics());
            assert_eq!(
                result.diagnostics()[0].code(),
                DiagnosticCode::EvaluationResourceLimit
            );
            assert!(result.steps() <= steps);
        }
        assert_eq!(
            replay_witness(&mut evaluator, function, &arguments, steps).outcome(),
            WitnessReplayOutcome::HoldsForThisWitness
        );
    }

    #[test]
    fn finite_size_type_modulus_and_linked_instances_replay_exactly() {
        let core = program(&[
            concat!(
                "edition 2026; module root { use lib; ",
                "spec p[n in 1..3, T in {Int, Mod[7]}](x: T^n) -> Bool { lib::p[n, T](x) } ",
                "spec residue[m in 7..9](x: Mod[m]) -> Bool { x == 0 } }"
            ),
            "edition 2026; module lib { spec p[n in 1..3, T in {Int, Mod[7]}](x: T^n) -> Bool { x[0] == 0 } }",
        ]);
        let mut evaluator = Evaluator::new(&core).unwrap();
        for module in ["root", "lib"] {
            for n in [1, 2] {
                for type_index in [0, 1] {
                    let function = function(&core, module, "p", &[n, type_index]);
                    let input = if n == 1 { "[[0]]" } else { "[[0, 1]]" };
                    let arguments = arguments(function, input);
                    assert_eq!(
                        replay_witness(&mut evaluator, function, &arguments, 100).outcome(),
                        WitnessReplayOutcome::HoldsForThisWitness
                    );
                    assert_eq!(arguments.values()[0].ty(), function.parameters()[0]);
                }
            }
        }
        let seven = function(&core, "root", "residue", &[7]);
        let eight = function(&core, "root", "residue", &[8]);
        let seven_arguments = arguments(seven, "[6]");
        assert_eq!(
            replay_witness(&mut evaluator, seven, &seven_arguments, 100).outcome(),
            WitnessReplayOutcome::Falsified
        );
        assert_eq!(
            replay_witness(&mut evaluator, eight, &seven_arguments, 100).diagnostics()[0].code(),
            DiagnosticCode::InvalidWitnessReplayBinding
        );
        let eight_arguments = arguments(eight, "[7]");
        assert_eq!(
            replay_witness(&mut evaluator, eight, &eight_arguments, 100).outcome(),
            WitnessReplayOutcome::Falsified
        );
    }

    #[test]
    fn all_concrete_scalar_and_aggregate_parameters_replay_without_coercion() {
        let core = program(&[concat!(
            "edition 2026; module m { type Row = Mod[11]^2; type Matrix = Row^2; ",
            "spec p(a: Int, b: Bool, c: Word[8], d: Word[16], e: Word[32], f: Word[64], g: Mod[7], h: Int^2, i: Matrix, j: (Bool, Matrix, Word[8]^2)) -> Bool { ",
            "let x: Bool = a == -123; let y: Bool = b == true; let z: Bool = c == 255; ",
            "x && y && z && (d == 65535) && (e == 4294967295) && (f == 18446744073709551615) && (g == 6) && (h == [1, 2]) && (i[1][1] == 10) && (j.0 == true) } }"
        )]);
        let function = function(&core, "m", "p", &[]);
        let arguments = arguments(
            function,
            "[-123, true, 0xff, 0xffff, 0xffffffff, 0xffffffffffffffff, 6, [1, 2], [[0, 1], [9, 10]], (true, [[1, 2], [3, 4]], [0x00, 0xff])]",
        );
        let mut evaluator = Evaluator::new(&core).unwrap();
        assert_eq!(
            replay_witness(&mut evaluator, function, &arguments, 1000).outcome(),
            WitnessReplayOutcome::HoldsForThisWitness
        );
    }

    #[test]
    fn integer_runtime_magnitude_exhaustion_remains_failed_not_falsified() {
        let core = program(&["edition 2026; module m { spec p(x: Int) -> Bool { (x * x) == 0 } }"]);
        let function = function(&core, "m", "p", &[]);
        let integer = crate::core::ExactInteger::power_of_two(16_383, |limbs, count| {
            limbs.try_reserve_exact(count).is_ok()
        })
        .unwrap();
        let arguments = arguments(function, &format!("[{integer}]"));
        let mut evaluator = Evaluator::new(&core).unwrap();
        let original = evaluator
            .call(function, arguments.values(), 1_048_576)
            .unwrap();
        let result = replay_witness(&mut evaluator, function, &arguments, 1_048_576);
        assert!(result.has_errors());
        assert_eq!(result.diagnostics(), original.diagnostics());
        assert_eq!(
            result.diagnostics()[0].code(),
            DiagnosticCode::EvaluationResourceLimit
        );
        assert!(
            result.diagnostics()[0]
                .message()
                .contains("significant-bit")
        );
        assert!(
            result.diagnostics()[0]
                .notes()
                .iter()
                .any(|note| note.contains("`Int` is unbounded"))
        );
    }

    #[test]
    fn nearby_521_bit_moduli_bind_equal_decimal_values_to_distinct_domains() {
        let core = program(&[concat!(
            "edition 2026; module m { ",
            "spec p(x: Mod[(1 << 521) - 1]) -> Bool { x == 0 } ",
            "spec q(x: Mod[(1 << 521) - 2]) -> Bool { x == 0 } }"
        )]);
        let p = function(&core, "m", "p", &[]);
        let q = function(&core, "m", "q", &[]);
        let mut evaluator = Evaluator::new(&core).unwrap();
        let p_arguments = arguments(p, "[1]");
        let q_arguments = arguments(q, "[1]");
        assert_ne!(p_arguments.values(), q_arguments.values());
        assert_eq!(
            replay_witness(&mut evaluator, p, &p_arguments, 100).outcome(),
            WitnessReplayOutcome::Falsified
        );
        assert_eq!(
            replay_witness(&mut evaluator, q, &q_arguments, 100).outcome(),
            WitnessReplayOutcome::Falsified
        );
        assert_eq!(
            replay_witness(&mut evaluator, p, &q_arguments, 100).diagnostics()[0].code(),
            DiagnosticCode::InvalidWitnessReplayBinding
        );
        assert_eq!(
            replay_witness(&mut evaluator, q, &p_arguments, 100).diagnostics()[0].code(),
            DiagnosticCode::InvalidWitnessReplayBinding
        );
    }

    #[test]
    fn replay_and_drop_fit_in_one_mebibyte_stack_for_nested_aggregate_parameters() {
        std::thread::Builder::new().stack_size(1 << 20).spawn(|| {
            let core = program(&["edition 2026; module m { type R = Int^256; type M = R^256; spec p(x: (M, Bool)) -> Bool { x.1 } }"]);
            let function = function(&core, "m", "p", &[]);
            let row = format!("[{}]", vec!["0"; 256].join(", "));
            let input = format!("[([{}], false)]", vec![row; 256].join(", "));
            let arguments = arguments(function, &input);
            let mut evaluator = Evaluator::new(&core).unwrap();
            let result = replay_witness(&mut evaluator, function, &arguments, 100);
            assert_eq!(result.outcome(), WitnessReplayOutcome::Falsified);
        }).unwrap().join().unwrap();
    }
}
