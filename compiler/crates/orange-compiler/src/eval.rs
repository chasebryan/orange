//! Deterministic reference evaluation for typed Orange Core.
//!
//! Evaluation is an explicit stack machine over postorder Core expressions,
//! so neither expression depth nor call depth uses the host call stack.

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::{
    CoreFunction, CoreFunctionId, CoreModule, CoreNode, CoreNodeKind, CoreType, CoreValue,
    ExactInteger, MAX_EXACT_INTEGER_BITS,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::{BinaryOperator, UnaryOperator};
use crate::source::Span;

/// Maximum reference-evaluation steps performed for one source module.
pub const MAX_EVALUATION_STEPS_PER_SOURCE: usize = 1_048_576;

/// Maximum nested calls, counting the evaluated function as the first frame.
pub const MAX_CALL_DEPTH: usize = 256;

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

/// Evaluates every typed Core function without parameters, in source order.
///
/// Functions with parameters are evaluated only through calls.
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
    stack: fn(&mut Vec<Value>, usize) -> bool,
    frames: fn(&mut Vec<Frame<'_>>, usize) -> bool,
}

impl Reservations {
    const DEFAULT: Self = Self {
        name: reserve_name,
        value_limbs: reserve_value_limbs,
        diagnostics: reserve_diagnostics,
        stack: reserve_stack,
        frames: reserve_frames,
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

fn reserve_stack(values: &mut Vec<Value>, count: usize) -> bool {
    values.try_reserve(count).is_ok()
}

fn reserve_frames(frames: &mut Vec<Frame<'_>>, count: usize) -> bool {
    frames.try_reserve(count).is_ok()
}

/// A runtime value. Exact integers are shared so that loading a literal or
/// parameter never copies its digits.
#[derive(Clone, Debug)]
enum Value {
    Int(Rc<ExactInteger>),
    Word(u64),
}

/// One active function evaluation.
struct Frame<'core> {
    function: &'core CoreFunction,
    /// Index of the next node to evaluate.
    next: usize,
    /// Index in the shared value stack of this frame's first argument.
    base: usize,
}

/// Why evaluation stopped without a value.
enum Stop {
    Steps,
    CallDepth(Span),
    IntegerBits(Span),
    Allocation(&'static str),
    InconsistentCore,
}

struct Machine<'core> {
    core: &'core CoreModule,
    literals: Vec<Vec<Option<Rc<ExactInteger>>>>,
    steps: usize,
    step_limit: usize,
    reservations: Reservations,
    stack: Vec<Value>,
    frames: Vec<Frame<'core>>,
}

fn digits(value: &ExactInteger) -> usize {
    value.magnitude_digits()
}

fn word_mask(ty: CoreType) -> Option<u64> {
    match ty {
        CoreType::Int => None,
        CoreType::Word8 => Some(u64::from(u8::MAX)),
        CoreType::Word16 => Some(u64::from(u16::MAX)),
        CoreType::Word32 => Some(u64::from(u32::MAX)),
        CoreType::Word64 => Some(u64::MAX),
    }
}

/// Applies a word operator modulo 2^n. Every operand is already reduced, and
/// 2^n divides 2^64, so wrapping 64-bit arithmetic followed by the mask is
/// exact arithmetic modulo 2^n.
fn word_binary(operator: BinaryOperator, mask: u64, left: u64, right: u64) -> Option<u64> {
    let value = match operator {
        BinaryOperator::Add => left.wrapping_add(right),
        BinaryOperator::Subtract => left.wrapping_sub(right),
        BinaryOperator::Multiply => left.wrapping_mul(right),
        BinaryOperator::And => left & right,
        BinaryOperator::Or => left | right,
        BinaryOperator::Xor => left ^ right,
        BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight
        | BinaryOperator::RotateLeft
        | BinaryOperator::RotateRight => return None,
    };
    Some(value & mask)
}

/// Shifts or rotates an n-bit word by `amount`, where `amount < bits`.
fn word_shift(
    operator: BinaryOperator,
    bits: u32,
    mask: u64,
    value: u64,
    amount: u32,
) -> Option<u64> {
    if amount >= bits {
        return None;
    }
    let rotate_left = |by: u32| -> Option<u64> {
        // For 0 < by < bits this is (value << by) | (value >> (bits - by));
        // for by = 0 the second term is zero because value < 2^bits.
        let high = value.checked_shl(by).unwrap_or(0);
        let low = value.checked_shr(bits.checked_sub(by)?).unwrap_or(0);
        Some((high | low) & mask)
    };
    match operator {
        BinaryOperator::ShiftLeft => Some(value.checked_shl(amount).unwrap_or(0) & mask),
        BinaryOperator::ShiftRight => Some(value.checked_shr(amount).unwrap_or(0)),
        BinaryOperator::RotateLeft => rotate_left(amount),
        BinaryOperator::RotateRight => rotate_left(bits.checked_sub(amount)?.checked_rem(bits)?),
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::And
        | BinaryOperator::Or
        | BinaryOperator::Xor => None,
    }
}

impl<'core> Machine<'core> {
    fn charge(&mut self, cost: usize) -> Result<(), Stop> {
        let steps = self.steps.checked_add(cost).ok_or(Stop::Steps)?;
        if steps > self.step_limit {
            return Err(Stop::Steps);
        }
        self.steps = steps;
        Ok(())
    }

    fn push(&mut self, value: Value) -> Result<(), Stop> {
        if !(self.reservations.stack)(&mut self.stack, 1) {
            return Err(Stop::Allocation(
                "evaluation value stack could not be reserved",
            ));
        }
        self.stack.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<Value, Stop> {
        self.stack.pop().ok_or(Stop::InconsistentCore)
    }

    fn pop_int(&mut self) -> Result<Rc<ExactInteger>, Stop> {
        match self.pop()? {
            Value::Int(value) => Ok(value),
            Value::Word(_) => Err(Stop::InconsistentCore),
        }
    }

    fn pop_word(&mut self) -> Result<u64, Stop> {
        match self.pop()? {
            Value::Word(value) => Ok(value),
            Value::Int(_) => Err(Stop::InconsistentCore),
        }
    }

    fn checked_int(&self, value: ExactInteger, span: Span) -> Result<Value, Stop> {
        if value.magnitude_bits() > MAX_EXACT_INTEGER_BITS {
            return Err(Stop::IntegerBits(span));
        }
        Ok(Value::Int(Rc::new(value)))
    }

    /// Evaluates one parameterless function to completion.
    fn run(&mut self, root: &'core CoreFunction) -> Result<Value, Stop> {
        self.stack.clear();
        self.frames.clear();
        if !(self.reservations.frames)(&mut self.frames, 1) {
            return Err(Stop::Allocation(
                "evaluation call stack could not be reserved",
            ));
        }
        self.frames.push(Frame {
            function: root,
            next: 0,
            base: 0,
        });
        loop {
            let Some(frame) = self.frames.last() else {
                return Err(Stop::InconsistentCore);
            };
            let function = frame.function;
            let base = frame.base;
            let offset = frame.next;
            let Some(node) = function.body.nodes.get(offset) else {
                // The frame's body is complete: its value replaces its arguments.
                let value = self.pop()?;
                self.frames.pop();
                if self.stack.len() != base {
                    self.stack.truncate(base);
                    if self.stack.len() != base {
                        return Err(Stop::InconsistentCore);
                    }
                }
                if self.frames.is_empty() {
                    return Ok(value);
                }
                self.push(value)?;
                continue;
            };
            if let Some(frame) = self.frames.last_mut() {
                frame.next = frame.next.saturating_add(1);
            }
            self.step(function, base, offset, node)?;
        }
    }

    fn step(
        &mut self,
        function: &'core CoreFunction,
        base: usize,
        offset: usize,
        node: &'core CoreNode,
    ) -> Result<(), Stop> {
        match &node.kind {
            CoreNodeKind::Literal(value) => {
                self.charge(1)?;
                let value = match value {
                    CoreValue::Int(_) => {
                        let index = usize::try_from(function.id.index())
                            .map_err(|_| Stop::InconsistentCore)?;
                        let shared = self
                            .literals
                            .get(index)
                            .and_then(|literals| literals.get(offset))
                            .and_then(Option::as_ref)
                            .ok_or(Stop::InconsistentCore)?;
                        Value::Int(Rc::clone(shared))
                    }
                    word => Value::Word(word.word_as_u64().ok_or(Stop::InconsistentCore)?),
                };
                self.push(value)
            }
            CoreNodeKind::Parameter(index) => {
                self.charge(1)?;
                let index = usize::try_from(*index).map_err(|_| Stop::InconsistentCore)?;
                let slot = base.checked_add(index).ok_or(Stop::InconsistentCore)?;
                let value = self
                    .stack
                    .get(slot)
                    .cloned()
                    .ok_or(Stop::InconsistentCore)?;
                self.push(value)
            }
            CoreNodeKind::Call {
                function: callee,
                arguments,
            } => {
                self.charge(1)?;
                if self.frames.len() >= MAX_CALL_DEPTH {
                    return Err(Stop::CallDepth(node.span));
                }
                let callee_index =
                    usize::try_from(callee.index()).map_err(|_| Stop::InconsistentCore)?;
                let callee = self
                    .core
                    .functions
                    .get(callee_index)
                    .ok_or(Stop::InconsistentCore)?;
                let arguments = usize::try_from(*arguments).map_err(|_| Stop::InconsistentCore)?;
                if callee.parameters.len() != arguments {
                    return Err(Stop::InconsistentCore);
                }
                let callee_base = self
                    .stack
                    .len()
                    .checked_sub(arguments)
                    .filter(|callee_base| *callee_base >= base)
                    .ok_or(Stop::InconsistentCore)?;
                if !(self.reservations.frames)(&mut self.frames, 1) {
                    return Err(Stop::Allocation(
                        "evaluation call stack could not be reserved",
                    ));
                }
                self.frames.push(Frame {
                    function: callee,
                    next: 0,
                    base: callee_base,
                });
                Ok(())
            }
            CoreNodeKind::Unary(operator) => {
                let value = match (operator, word_mask(node.ty)) {
                    (UnaryOperator::Negate, None) => {
                        let operand = self.pop_int()?;
                        self.charge(digits(&operand).saturating_add(1))?;
                        let negated = operand
                            .try_clone_with_reservation(self.reservations.value_limbs)
                            .ok_or(Stop::Allocation(
                                "exact integer storage could not be reserved",
                            ))?
                            .negated();
                        self.checked_int(negated, node.span)?
                    }
                    (UnaryOperator::Complement, Some(mask)) => {
                        self.charge(1)?;
                        Value::Word(!self.pop_word()? & mask)
                    }
                    _ => return Err(Stop::InconsistentCore),
                };
                self.push(value)
            }
            CoreNodeKind::Binary(operator) => {
                let value = if let Some(mask) = word_mask(node.ty) {
                    self.charge(1)?;
                    let right = self.pop_word()?;
                    let left = self.pop_word()?;
                    Value::Word(
                        word_binary(*operator, mask, left, right).ok_or(Stop::InconsistentCore)?,
                    )
                } else {
                    let right = self.pop_int()?;
                    let left = self.pop_int()?;
                    let reserve = self.reservations.value_limbs;
                    let result = match operator {
                        BinaryOperator::Add | BinaryOperator::Subtract => {
                            self.charge(digits(&left).max(digits(&right)).saturating_add(1))?;
                            if *operator == BinaryOperator::Add {
                                left.add(&right, reserve)
                            } else {
                                left.subtract(&right, reserve)
                            }
                        }
                        BinaryOperator::Multiply => {
                            self.charge(
                                digits(&left)
                                    .checked_mul(digits(&right))
                                    .ok_or(Stop::Steps)?
                                    .saturating_add(1),
                            )?;
                            left.multiply(&right, reserve)
                        }
                        _ => return Err(Stop::InconsistentCore),
                    };
                    let result = result.ok_or(Stop::Allocation(
                        "exact integer storage could not be reserved",
                    ))?;
                    self.checked_int(result, node.span)?
                };
                self.push(value)
            }
            CoreNodeKind::Shift { operator, amount } => {
                self.charge(1)?;
                let bits = node.ty.word_bits().ok_or(Stop::InconsistentCore)?;
                let mask = word_mask(node.ty).ok_or(Stop::InconsistentCore)?;
                let value = self.pop_word()?;
                let shifted = word_shift(*operator, bits, mask, value, *amount)
                    .ok_or(Stop::InconsistentCore)?;
                self.push(Value::Word(shifted))
            }
        }
    }
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
    let roots = core
        .functions
        .iter()
        .filter(|function| function.parameters.is_empty())
        .count();
    let capacity = roots.min(step_limit);
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
    let Some(literals) = share_literals(core) else {
        return allocation_failure(
            diagnostics,
            core.span,
            "evaluated exact integer storage could not be reserved",
        );
    };
    let mut machine = Machine {
        core,
        literals,
        steps: 0,
        step_limit,
        reservations,
        stack: Vec::new(),
        frames: Vec::new(),
    };
    let mut shared_module = None;
    for function in core
        .functions
        .iter()
        .filter(|function| function.parameters.is_empty())
    {
        let steps_before = machine.steps;
        let value = match machine.run(function) {
            Ok(value) => value,
            Err(stop) => {
                return stopped(
                    diagnostics,
                    function,
                    stop,
                    step_limit,
                    steps_before == machine.steps,
                );
            }
        };
        let mut name = String::new();
        if !(reservations.name)(&mut name, function.name.len()) {
            return allocation_failure(
                diagnostics,
                function.name_span,
                "evaluated function name storage could not be reserved",
            );
        }
        name.push_str(&function.name);
        let value = match value {
            Value::Int(value) => match value.try_clone_with_reservation(reservations.value_limbs) {
                Some(value) => CoreValue::Int(value),
                None => {
                    return allocation_failure(
                        diagnostics,
                        function.name_span,
                        "evaluated exact integer storage could not be reserved",
                    );
                }
            },
            Value::Word(value) => {
                let Some(value) = CoreValue::word_from_u64(function.result_type, value) else {
                    return stopped(
                        diagnostics,
                        function,
                        Stop::InconsistentCore,
                        step_limit,
                        false,
                    );
                };
                value
            }
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

/// Shares every `Int` literal once so that evaluation never copies literal digits.
fn share_literals(core: &CoreModule) -> Option<Vec<Vec<Option<Rc<ExactInteger>>>>> {
    let mut shared = Vec::new();
    shared.try_reserve_exact(core.functions.len()).ok()?;
    for function in &core.functions {
        let mut literals = Vec::new();
        literals.try_reserve_exact(function.body.nodes.len()).ok()?;
        for node in &function.body.nodes {
            literals.push(match &node.kind {
                CoreNodeKind::Literal(CoreValue::Int(value)) => Some(Rc::new(
                    value.try_clone_with_reservation(reserve_value_limbs)?,
                )),
                _ => None,
            });
        }
        shared.push(literals);
    }
    Some(shared)
}

fn stopped(
    diagnostics: Vec<Diagnostic>,
    function: &CoreFunction,
    stop: Stop,
    step_limit: usize,
    before_function: bool,
) -> EvaluationResult {
    let diagnostic = match stop {
        Stop::Steps => Diagnostic::error(
            DiagnosticCode::EvaluationResourceLimit,
            "reference evaluation step limit exceeded",
            function.name_span,
        )
        .with_label(if before_function {
            "evaluation stopped before this function"
        } else {
            "evaluation stopped while evaluating this function"
        })
        .with_note(format!(
            "at most {step_limit} evaluation steps are permitted"
        )),
        Stop::CallDepth(span) => Diagnostic::error(
            DiagnosticCode::EvaluationResourceLimit,
            "reference evaluation call depth limit exceeded",
            span,
        )
        .with_label("this call exceeds the depth limit")
        .with_secondary_span(function.name_span, "evaluation of this function")
        .with_note(format!(
            "at most {MAX_CALL_DEPTH} nested calls are permitted"
        )),
        Stop::IntegerBits(span) => Diagnostic::error(
            DiagnosticCode::EvaluationResourceLimit,
            format!(
                "exact integer result exceeds the {MAX_EXACT_INTEGER_BITS}-significant-bit limit"
            ),
            span,
        )
        .with_label("result is too large for the reference evaluator")
        .with_secondary_span(function.name_span, "evaluation of this function")
        .with_note("`Int` is unbounded; this is a resource limit, not a finite width"),
        Stop::Allocation(label) => Diagnostic::error(
            DiagnosticCode::EvaluationResourceLimit,
            "reference evaluation result allocation failed",
            function.name_span,
        )
        .with_label(label),
        Stop::InconsistentCore => Diagnostic::error(
            DiagnosticCode::EvaluationResourceLimit,
            "reference evaluation received inconsistent Core",
            function.name_span,
        )
        .with_label("evaluation stopped in this function"),
    };
    evaluation_failure(
        diagnostics,
        diagnostic.with_note("no partial value set is returned"),
    )
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
                "at most 1 evaluation steps are permitted",
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
