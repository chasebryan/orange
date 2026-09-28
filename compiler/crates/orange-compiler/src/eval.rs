//! Deterministic reference evaluation for typed Orange Core.
//!
//! Evaluation is an explicit stack machine over postorder Core expressions,
//! so neither expression depth nor call depth uses the host call stack.

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::{
    ArrayType, CoreArray, CoreExpression, CoreFunction, CoreFunctionId, CoreModule, CoreNode,
    CoreNodeKind, CoreType, CoreValue, ExactInteger, MAX_EXACT_INTEGER_BITS,
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
    array: fn(&mut Vec<Value>, usize) -> bool,
    result_array: fn(&mut Vec<CoreValue>, usize) -> bool,
}

impl Reservations {
    const DEFAULT: Self = Self {
        name: reserve_name,
        value_limbs: reserve_value_limbs,
        diagnostics: reserve_diagnostics,
        stack: reserve_stack,
        frames: reserve_frames,
        array: reserve_array,
        result_array: reserve_result_array,
    };
}

fn reserve_array(elements: &mut Vec<Value>, count: usize) -> bool {
    elements.try_reserve_exact(count).is_ok()
}

fn reserve_result_array(elements: &mut Vec<CoreValue>, count: usize) -> bool {
    elements.try_reserve_exact(count).is_ok()
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

/// A runtime value. Exact integers and arrays are shared so that loading a
/// literal, parameter, or binding never copies their contents.
#[derive(Clone, Debug)]
enum Value {
    Int(Rc<ExactInteger>),
    Word(u64),
    Array(Rc<Vec<Value>>),
}

/// One active function evaluation.
///
/// A frame's values occupy the shared stack from `base`: its arguments, then
/// the values of its completed bindings, then intermediate values.
struct Frame<'core> {
    function: &'core CoreFunction,
    /// The expression being evaluated: a binding's index, or the number of
    /// bindings for the body.
    part: usize,
    /// Index of the next node of that expression to evaluate.
    next: usize,
    /// Index in the shared value stack of this frame's first argument.
    base: usize,
}

/// Returns a function's binding value at `part`, or its body when `part` is
/// the number of bindings.
fn expression_part(function: &CoreFunction, part: usize) -> Option<&CoreExpression> {
    match function.locals.get(part) {
        Some(local) => Some(&local.value),
        None => (part == function.locals.len()).then_some(&function.body),
    }
}

/// Why evaluation stopped without a value.
enum Stop {
    Steps,
    CallDepth(Span),
    IntegerBits(Span),
    Allocation(&'static str),
    InconsistentCore,
}

/// Shared `Int` literals, indexed by function, expression part, and node.
type SharedLiterals = Vec<Vec<Vec<Option<Rc<ExactInteger>>>>>;

struct Machine<'core> {
    core: &'core CoreModule,
    literals: SharedLiterals,
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
        CoreType::Int | CoreType::Array(_) => None,
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
            Value::Word(_) | Value::Array(_) => Err(Stop::InconsistentCore),
        }
    }

    fn pop_word(&mut self) -> Result<u64, Stop> {
        match self.pop()? {
            Value::Word(value) => Ok(value),
            Value::Int(_) | Value::Array(_) => Err(Stop::InconsistentCore),
        }
    }

    /// Replaces the top `length` values of the current expression with one
    /// array of type `ty` holding them in order.
    fn build_array(&mut self, ty: ArrayType, length: usize, floor: usize) -> Result<Value, Stop> {
        if usize::try_from(ty.length()).ok() != Some(length) {
            return Err(Stop::InconsistentCore);
        }
        let start = self
            .stack
            .len()
            .checked_sub(length)
            .filter(|start| *start >= floor)
            .ok_or(Stop::InconsistentCore)?;
        let mut elements = Vec::new();
        if !(self.reservations.array)(&mut elements, length) {
            return Err(Stop::Allocation(
                "evaluation array storage could not be reserved",
            ));
        }
        let word = word_mask(ty.element());
        for element in self.stack.drain(start..) {
            let consistent = match (&element, word) {
                (Value::Int(_), None) => true,
                (Value::Word(value), Some(mask)) => (*value & !mask) == 0,
                _ => false,
            };
            if !consistent {
                return Err(Stop::InconsistentCore);
            }
            elements.push(element);
        }
        Ok(Value::Array(Rc::new(elements)))
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
            part: 0,
            next: 0,
            base: 0,
        });
        loop {
            let Some(frame) = self.frames.last() else {
                return Err(Stop::InconsistentCore);
            };
            let function = frame.function;
            let base = frame.base;
            let part = frame.part;
            let offset = frame.next;
            let expression = expression_part(function, part).ok_or(Stop::InconsistentCore)?;
            let Some(node) = expression.nodes.get(offset) else {
                if part < function.locals.len() {
                    // The binding's value stays on the stack as its slot,
                    // directly after the arguments and earlier bindings.
                    let slots = base
                        .checked_add(function.parameters.len())
                        .and_then(|slots| slots.checked_add(part))
                        .and_then(|slots| slots.checked_add(1))
                        .ok_or(Stop::InconsistentCore)?;
                    if self.stack.len() != slots {
                        return Err(Stop::InconsistentCore);
                    }
                    if let Some(frame) = self.frames.last_mut() {
                        frame.part = part.saturating_add(1);
                        frame.next = 0;
                    }
                    continue;
                }
                // The frame's body is complete: its value replaces its
                // arguments and bindings.
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
            self.step(function, base, part, offset, node)?;
        }
    }

    fn step(
        &mut self,
        function: &'core CoreFunction,
        base: usize,
        part: usize,
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
                            .and_then(|parts| parts.get(part))
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
            CoreNodeKind::Local(index) => {
                self.charge(1)?;
                // Only a binding before the current part has a value.
                let index = usize::try_from(*index)
                    .ok()
                    .filter(|index| *index < part)
                    .ok_or(Stop::InconsistentCore)?;
                let slot = base
                    .checked_add(function.parameters.len())
                    .and_then(|slot| slot.checked_add(index))
                    .ok_or(Stop::InconsistentCore)?;
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
                    part: 0,
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
            CoreNodeKind::Array { elements } => {
                let ty = node.ty.as_array().ok_or(Stop::InconsistentCore)?;
                let length = usize::try_from(*elements).map_err(|_| Stop::InconsistentCore)?;
                // One step per element; an array has at least one.
                self.charge(length.max(1))?;
                // Elements are intermediate values above the arguments and
                // the bindings already evaluated.
                let floor = base
                    .checked_add(function.parameters.len())
                    .and_then(|floor| floor.checked_add(part))
                    .ok_or(Stop::InconsistentCore)?;
                let array = self.build_array(ty, length, floor)?;
                self.push(array)
            }
            CoreNodeKind::Index { index } => {
                self.charge(1)?;
                let Value::Array(array) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                let index = usize::try_from(*index).map_err(|_| Stop::InconsistentCore)?;
                let element = array.get(index).cloned().ok_or(Stop::InconsistentCore)?;
                let consistent = match (&element, word_mask(node.ty)) {
                    (Value::Int(_), None) => node.ty == CoreType::Int,
                    (Value::Word(_), Some(_)) => true,
                    _ => false,
                };
                if !consistent {
                    return Err(Stop::InconsistentCore);
                }
                self.push(element)
            }
            CoreNodeKind::Convert { from } => {
                self.charge(1)?;
                let value = match (word_mask(*from), word_mask(node.ty)) {
                    (None, None) => Value::Int(self.pop_int()?),
                    (None, Some(mask)) => Value::Word(self.pop_int()?.modulo_2_64() & mask),
                    (Some(_), Some(mask)) => Value::Word(self.pop_word()? & mask),
                    (Some(_), None) => {
                        let word = self.pop_word()?;
                        let value = ExactInteger::from_u64(word, self.reservations.value_limbs)
                            .ok_or(Stop::Allocation(
                                "exact integer storage could not be reserved",
                            ))?;
                        Value::Int(Rc::new(value))
                    }
                };
                self.push(value)
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
        let value = match result_value(value, function.result_type, reservations) {
            Ok(value) => value,
            Err(Stop::Allocation(label)) => {
                return allocation_failure(diagnostics, function.name_span, label);
            }
            Err(stop) => return stopped(diagnostics, function, stop, step_limit, false),
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

/// Copies an evaluated value of type `ty` out of the machine's shared storage.
fn result_value(value: Value, ty: CoreType, reservations: Reservations) -> Result<CoreValue, Stop> {
    match (value, ty) {
        (Value::Int(value), CoreType::Int) => value
            .try_clone_with_reservation(reservations.value_limbs)
            .map(CoreValue::Int)
            .ok_or(Stop::Allocation(
                "evaluated exact integer storage could not be reserved",
            )),
        (Value::Word(value), ty) => {
            CoreValue::word_from_u64(ty, value).ok_or(Stop::InconsistentCore)
        }
        (Value::Array(array), CoreType::Array(array_type)) => {
            let mut elements = Vec::new();
            if !(reservations.result_array)(&mut elements, array.len()) {
                return Err(Stop::Allocation(
                    "evaluated array storage could not be reserved",
                ));
            }
            for element in array.iter() {
                elements.push(result_element(element, array_type.element(), reservations)?);
            }
            CoreArray::new(array_type, elements)
                .map(CoreValue::Array)
                .ok_or(Stop::InconsistentCore)
        }
        _ => Err(Stop::InconsistentCore),
    }
}

/// Copies one scalar array element; arrays have no array elements.
fn result_element(
    element: &Value,
    ty: CoreType,
    reservations: Reservations,
) -> Result<CoreValue, Stop> {
    match (element, ty) {
        (Value::Int(value), CoreType::Int) => value
            .try_clone_with_reservation(reservations.value_limbs)
            .map(CoreValue::Int)
            .ok_or(Stop::Allocation(
                "evaluated exact integer storage could not be reserved",
            )),
        (Value::Word(value), ty) => {
            CoreValue::word_from_u64(ty, *value).ok_or(Stop::InconsistentCore)
        }
        _ => Err(Stop::InconsistentCore),
    }
}

/// Shares every `Int` literal once so that evaluation never copies literal digits.
fn share_literals(core: &CoreModule) -> Option<SharedLiterals> {
    let mut shared = Vec::new();
    shared.try_reserve_exact(core.functions.len()).ok()?;
    for function in &core.functions {
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(function.locals.len().checked_add(1)?)
            .ok()?;
        let expressions = function
            .locals
            .iter()
            .map(|local| &local.value)
            .chain(std::iter::once(&function.body));
        for expression in expressions {
            let mut literals = Vec::new();
            literals.try_reserve_exact(expression.nodes.len()).ok()?;
            for node in &expression.nodes {
                literals.push(match &node.kind {
                    CoreNodeKind::Literal(CoreValue::Int(value)) => Some(Rc::new(
                        value.try_clone_with_reservation(reserve_value_limbs)?,
                    )),
                    _ => None,
                });
            }
            parts.push(literals);
        }
        shared.push(parts);
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

    /// Evaluates `members` in a module named `m` and renders every value.
    fn values_of(members: &str) -> Vec<String> {
        let core = core(&format!("edition 2026; module m {{\n{members}}}\n"));
        let result = evaluate(&core);
        assert_eq!(result.diagnostics(), [], "{members}");
        assert_eq!(result, evaluate(&core));
        result
            .values()
            .unwrap()
            .iter()
            .map(|value| format!("{} = {}", value.name(), value.value()))
            .collect()
    }

    const WORDS: [(&str, u32); 4] = [
        ("Word[8]", 8),
        ("Word[16]", 16),
        ("Word[32]", 32),
        ("Word[64]", 64),
    ];

    fn render_word(bits: u32, value: u128) -> String {
        let digits = usize::try_from(bits / 4).unwrap();
        format!("0x{value:0digits$x}")
    }

    /// Edge values and a deterministic xorshift stream for one width.
    fn word_corpus(bits: u32) -> Vec<u128> {
        let modulus = 1_u128 << bits;
        let mut values = vec![
            0,
            1,
            2,
            modulus - 1,
            modulus - 2,
            modulus >> 1,
            (modulus >> 1) - 1,
        ];
        let mut state = 0x9e37_79b9_7f4a_7c15_u64 ^ u64::from(bits);
        for _ in 0..9 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            values.push(u128::from(state) % modulus);
        }
        values
    }

    #[test]
    fn word_operators_match_a_wide_reference_at_every_width() {
        for (ty, bits) in WORDS {
            let modulus = 1_u128 << bits;
            let corpus = word_corpus(bits);
            let mut members = String::new();
            let mut expected = Vec::new();
            for (index, (&left, &right)) in corpus.iter().zip(corpus.iter().rev()).enumerate() {
                let cases = [
                    ("+", (left + right) % modulus),
                    ("-", (left + modulus - right) % modulus),
                    ("*", (left * right) % modulus),
                    ("&", left & right),
                    ("|", left | right),
                    ("^", left ^ right),
                ];
                for (operator_index, (operator, value)) in cases.into_iter().enumerate() {
                    let name = format!("op{index}_{operator_index}");
                    members.push_str(&format!(
                        "  spec {name}() -> {ty} {{ {left} {operator} 0x{right:x} }}\n"
                    ));
                    expected.push(format!("{name} = {}", render_word(bits, value)));
                }
                let name = format!("not{index}");
                members.push_str(&format!("  spec {name}() -> {ty} {{ ~{left} }}\n"));
                expected.push(format!(
                    "{name} = {}",
                    render_word(bits, (modulus - 1) ^ left)
                ));
            }
            assert_eq!(values_of(&members), expected, "{ty}");
        }
    }

    #[test]
    fn shifts_and_rotations_match_the_reference_for_every_amount() {
        for (ty, bits) in WORDS {
            let modulus = 1_u128 << bits;
            let mask = modulus - 1;
            let rotate_left = |value: u128, amount: u32| {
                ((value << amount) | (value >> ((bits - amount) % bits))) & mask
            };
            let mut members = String::new();
            let mut expected = Vec::new();
            for value in [mask, 0x81 % modulus, word_corpus(bits)[9]] {
                for amount in 0..bits {
                    let cases = [
                        ("<<", (value << amount) & mask),
                        (">>", value >> amount),
                        ("<<<", rotate_left(value, amount)),
                        (">>>", rotate_left(value, (bits - amount) % bits)),
                    ];
                    for (operator_index, (operator, result)) in cases.into_iter().enumerate() {
                        let name = format!("s{}_{amount}_{operator_index}", members.len());
                        members.push_str(&format!(
                            "  spec {name}() -> {ty} {{ {value} {operator} {amount} }}\n"
                        ));
                        expected.push(format!("{name} = {}", render_word(bits, result)));
                    }
                }
            }
            assert_eq!(values_of(&members), expected, "{ty}");
        }
    }

    #[test]
    fn sha256_round_zero_matches_the_fips_example() {
        // FIPS 180-4 example "abc": after round t = 0, a = 5d6aebcd and
        // e = fa2a4622, from the initial hash value, K0, and W0 = 61626380.
        let values = values_of(concat!(
            "  spec big_sigma0(x: Word[32]) -> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }\n",
            "  spec big_sigma1(x: Word[32]) -> Word[32] { (x >>> 6) ^ (x >>> 11) ^ (x >>> 25) }\n",
            "  spec small_sigma0(x: Word[32]) -> Word[32] { (x >>> 7) ^ (x >>> 18) ^ (x >> 3) }\n",
            "  spec small_sigma1(x: Word[32]) -> Word[32] { (x >>> 17) ^ (x >>> 19) ^ (x >> 10) }\n",
            "  spec choose(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {\n",
            "    (x & y) ^ (~x & z)\n",
            "  }\n",
            "  spec majority(x: Word[32], y: Word[32], z: Word[32]) -> Word[32] {\n",
            "    (x & y) ^ (x & z) ^ (y & z)\n",
            "  }\n",
            "  spec t1(e: Word[32], f: Word[32], g: Word[32], h: Word[32], k: Word[32], w: Word[32])\n",
            "    -> Word[32] { h + big_sigma1(e) + choose(e, f, g) + k + w }\n",
            "  spec t2(a: Word[32], b: Word[32], c: Word[32]) -> Word[32] {\n",
            "    big_sigma0(a) + majority(a, b, c)\n",
            "  }\n",
            "  spec sigma0_of_a() -> Word[32] { big_sigma0(0x6a09e667) }\n",
            "  spec sigma1_of_e() -> Word[32] { big_sigma1(0x510e527f) }\n",
            "  spec schedule0() -> Word[32] { small_sigma0(0x61626380) }\n",
            "  spec schedule1() -> Word[32] { small_sigma1(0x61626380) }\n",
            "  spec ch() -> Word[32] { choose(0x510e527f, 0x9b05688c, 0x1f83d9ab) }\n",
            "  spec maj() -> Word[32] { majority(0x6a09e667, 0xbb67ae85, 0x3c6ef372) }\n",
            "  spec round0_a() -> Word[32] {\n",
            "    t1(0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19, 0x428a2f98, 0x61626380)\n",
            "      + t2(0x6a09e667, 0xbb67ae85, 0x3c6ef372)\n",
            "  }\n",
            "  spec round0_e() -> Word[32] {\n",
            "    0xa54ff53a + t1(0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19, 0x428a2f98, 0x61626380)\n",
            "  }\n",
        ));
        assert_eq!(
            values,
            [
                "sigma0_of_a = 0xce20b47e",
                "sigma1_of_e = 0x3587272b",
                "schedule0 = 0x940e90ef",
                "schedule1 = 0x7da86405",
                "ch = 0x1f85c98c",
                "maj = 0x3a6fe667",
                "round0_a = 0x5d6aebcd",
                "round0_e = 0xfa2a4622",
            ]
        );
    }

    #[test]
    fn chacha20_quarter_round_matches_rfc_8439() {
        // RFC 8439 section 2.1.1, one intermediate value per function.
        let values = values_of(concat!(
            "  spec a1(a: Word[32], b: Word[32]) -> Word[32] { a + b }\n",
            "  spec d1(a: Word[32], b: Word[32], d: Word[32]) -> Word[32] {\n",
            "    (d ^ a1(a, b)) <<< 16\n",
            "  }\n",
            "  spec c1(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {\n",
            "    c + d1(a, b, d)\n",
            "  }\n",
            "  spec b1(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {\n",
            "    (b ^ c1(a, b, c, d)) <<< 12\n",
            "  }\n",
            "  spec a2(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {\n",
            "    a1(a, b) + b1(a, b, c, d)\n",
            "  }\n",
            "  spec d2(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {\n",
            "    (d1(a, b, d) ^ a2(a, b, c, d)) <<< 8\n",
            "  }\n",
            "  spec c2(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {\n",
            "    c1(a, b, c, d) + d2(a, b, c, d)\n",
            "  }\n",
            "  spec b2(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {\n",
            "    (b1(a, b, c, d) ^ c2(a, b, c, d)) <<< 7\n",
            "  }\n",
            "  spec a() -> Word[32] { a2(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }\n",
            "  spec b() -> Word[32] { b2(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }\n",
            "  spec c() -> Word[32] { c2(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }\n",
            "  spec d() -> Word[32] { d2(0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567) }\n",
        ));
        assert_eq!(
            values,
            [
                "a = 0xea2a92f4",
                "b = 0xcb1cf8ce",
                "c = 0x4581472e",
                "d = 0x5881c4bb"
            ]
        );
    }

    #[test]
    fn int_arithmetic_is_exact_through_calls() {
        let values = values_of(concat!(
            "  spec square(x: Int) -> Int { x * x }\n",
            "  spec two_to_128() -> Int { 18446744073709551616 * 18446744073709551616 }\n",
            "  spec tower() -> Int { square(square(12345678901234567890)) }\n",
            "  spec signs() -> Int { -(3 - 10) * -4 }\n",
            "  spec zero() -> Int { 5 - 5 + -0 * -7 }\n",
            "  spec crossing() -> Int { 1 - 18446744073709551617 + 18446744073709551616 }\n",
            "  spec negate_zero() -> Int { -(0 * -1) }\n",
        ));
        assert_eq!(
            values,
            [
                "two_to_128 = 340282366920938463463374607431768211456",
                "tower = 23230572289118153328333583928030329684079829544396666111742077337982514410000",
                "signs = -28",
                "zero = 0",
                "crossing = 0",
                "negate_zero = 0",
            ]
        );
    }

    fn analyzed(text: &str) -> (SourceMap, CoreModule) {
        let mut sources = SourceMap::new();
        let id = sources.add("evaluate.or", text).unwrap();
        let core = {
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::E2026);
            let parsed = parse(source, &lexed);
            assert_eq!(parsed.diagnostics(), []);
            let analyzed = analyze(source, parsed.ast().unwrap());
            assert_eq!(analyzed.diagnostics(), []);
            analyzed.into_core().unwrap()
        };
        (sources, core)
    }

    /// Evaluates `members` expecting one diagnostic, and returns it with the
    /// source text its primary span covers.
    fn single_failure(members: &str, step_limit: usize) -> (Diagnostic, String, CoreModule) {
        let text = format!("edition 2026; module m {{\n{members}}}\n");
        let (sources, core) = analyzed(&text);
        let result = evaluate_with_limit(&core, step_limit);
        assert_eq!(result, evaluate_with_limit(&core, step_limit));
        assert!(result.values().is_none());
        let [diagnostic] = result.diagnostics() else {
            panic!(
                "expected exactly one diagnostic: {:?}",
                result.diagnostics()
            );
        };
        assert_eq!(diagnostic.code(), DiagnosticCode::EvaluationResourceLimit);
        let covered = sources
            .iter()
            .next()
            .unwrap()
            .slice(diagnostic.primary_span())
            .unwrap()
            .to_owned();
        (diagnostic.clone(), covered, core)
    }

    #[test]
    fn int_results_are_bounded_by_the_significant_bit_limit() {
        // Thirteen squarings of 2 give 2^8192, and (2^8192 - 1)(2^8192 + 1) is
        // 2^16384 - 1: exactly the 16,384-bit limit.
        let squarings = |count: usize| format!("{}2{}", "s(".repeat(count), ")".repeat(count));
        let tower = squarings(13);
        let at_limit = format!("({tower} - 1) * ({tower} + 1)");
        let square = "  spec s(x: Int) -> Int { x * x }\n";
        let values = values_of(&format!(
            "{square}  spec at_limit() -> Int {{ {at_limit} }}\n  spec negative() -> Int {{ -({at_limit}) }}\n"
        ));
        assert_eq!(values.len(), 2);

        for (body, responsible) in [
            (format!("{at_limit} + 1"), None),
            (squarings(14), Some("x * x")),
            (format!("-({at_limit}) - 1"), None),
        ] {
            let (diagnostic, covered, core) = single_failure(
                &format!("{square}  spec over() -> Int {{ {body} }}\n"),
                MAX_EVALUATION_STEPS_PER_SOURCE,
            );
            assert_eq!(
                diagnostic.message(),
                "exact integer result exceeds the 16384-significant-bit limit"
            );
            assert_eq!(covered, responsible.unwrap_or(&body));
            let [function] = diagnostic.secondary_spans() else {
                panic!("the bit limit must cite the evaluated function");
            };
            assert_eq!(function.span(), core.functions[1].name_span);
        }
    }

    #[test]
    fn only_parameterless_functions_are_evaluated_and_reported() {
        assert_eq!(
            values_of(concat!(
                "  spec double(x: Word[16]) -> Word[16] { x + x }\n",
                "  spec four() -> Word[16] { double(double(1)) }\n",
                "  spec unused(x: Int) -> Int { x }\n",
                "  spec legacy() {}\n",
                "  impl legacy() {}\n",
            )),
            ["four = 0x0004"]
        );
        assert_eq!(
            values_of("  spec only(x: Int) -> Int { x }\n"),
            Vec::<String>::new()
        );
    }

    fn call_chain(length: usize) -> String {
        let mut members = String::new();
        for index in 0..length {
            members.push_str(&format!(
                "  spec f{index}() -> Int {{ f{}() }}\n",
                index + 1
            ));
        }
        members.push_str(&format!("  spec f{length}() -> Int {{ 7 }}\n"));
        members
    }

    #[test]
    fn call_depth_counts_the_evaluated_function_as_the_first_frame() {
        let values = values_of(&call_chain(MAX_CALL_DEPTH - 1));
        assert_eq!(values.len(), MAX_CALL_DEPTH);
        assert!(values.iter().all(|value| value.ends_with(" = 7")));

        let (diagnostic, covered, core) =
            single_failure(&call_chain(MAX_CALL_DEPTH), MAX_EVALUATION_STEPS_PER_SOURCE);
        assert_eq!(
            diagnostic.message(),
            "reference evaluation call depth limit exceeded"
        );
        assert_eq!(covered, format!("f{MAX_CALL_DEPTH}()"));
        assert_eq!(diagnostic.label(), "this call exceeds the depth limit");
        let [function] = diagnostic.secondary_spans() else {
            panic!("the depth limit must cite the evaluated function");
        };
        assert_eq!(function.span(), core.functions[0].name_span);
        assert_eq!(
            diagnostic.notes(),
            [
                "at most 256 nested calls are permitted",
                "no partial value set is returned"
            ]
        );
    }

    #[test]
    fn steps_follow_the_normative_cost_table() {
        // Word: every literal, operator, shift, and call costs one step, and
        // parameter loads cost one step each.
        // Int: negation costs 1 + d, addition and subtraction 1 + max(d1, d2),
        // and multiplication 1 + d1 * d2, where d counts 32-bit limbs.
        for (members, steps) in [
            ("  spec w() -> Word[8] { (1 + 2) ^ ~3 }\n", 6),
            ("  spec w() -> Word[32] { (1 <<< 3) >> 1 }\n", 3),
            (
                "  spec f(x: Word[8]) -> Word[8] { x + x }\n  spec w() -> Word[8] { f(7) }\n",
                5,
            ),
            ("  spec i() -> Int { 4294967296 * 4294967296 + 1 }\n", 12),
            ("  spec i() -> Int { -18446744073709551616 }\n", 1),
            ("  spec i() -> Int { -(18446744073709551616) }\n", 5),
            ("  spec i() -> Int { 0 - 0 }\n", 3),
            (
                "  spec f(x: Int) -> Int { x + x }\n  spec i() -> Int { f(7) }\n",
                6,
            ),
            // A binding costs its value's steps; each read of it costs one.
            (
                "  spec i() -> Int { let a: Int = 1; let b: Int = a + a; b }\n",
                6,
            ),
            // Every conversion costs one step.
            (
                "  spec w() -> Word[8] { let a: Word[32] = 0x1ff; a as Word[8] }\n",
                3,
            ),
            (
                "  spec i() -> Int { let a: Word[64] = 0xffffffffffffffff; (a as Int) * 2 }\n",
                7,
            ),
            (
                "  spec w() -> Word[8] { let n: Int = -18446744073709551617; n as Word[8] }\n",
                3,
            ),
        ] {
            let core = core(&format!("edition 2026; module m {{\n{members}}}\n"));
            let exact = evaluate_with_limit(&core, steps);
            assert_eq!(exact.diagnostics(), [], "{members}");
            let short = evaluate_with_limit(&core, steps - 1);
            assert!(short.values().is_none(), "{members}");
            assert_eq!(
                short.diagnostics()[0].message(),
                "reference evaluation step limit exceeded"
            );
            assert_eq!(
                short.diagnostics()[0].label(),
                if steps == 1 {
                    "evaluation stopped before this function"
                } else {
                    "evaluation stopped while evaluating this function"
                },
                "{members}"
            );
        }
    }

    #[test]
    fn exponential_call_trees_stop_at_the_step_limit() {
        let mut members = String::from("  spec d0(x: Word[8]) -> Word[8] { x + x }\n");
        for level in 1..=24 {
            members.push_str(&format!(
                "  spec d{level}(x: Word[8]) -> Word[8] {{ d{0}(x) ^ d{0}(x) }}\n",
                level - 1
            ));
        }
        members.push_str("  spec root() -> Word[8] { d24(1) }\n");
        let (diagnostic, covered, _) = single_failure(&members, MAX_EVALUATION_STEPS_PER_SOURCE);
        assert_eq!(
            diagnostic.message(),
            "reference evaluation step limit exceeded"
        );
        assert_eq!(covered, "root");
        assert_eq!(
            diagnostic.label(),
            "evaluation stopped while evaluating this function"
        );
    }

    #[test]
    fn value_and_call_stack_reservation_failures_return_no_values() {
        let core = core(concat!(
            "edition 2026; module m {\n",
            "  spec id(x: Word[8]) -> Word[8] { x }\n",
            "  spec root() -> Word[8] { id(1) }\n",
            "}\n",
        ));
        for (reservations, label) in [
            (
                Reservations {
                    stack: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluation value stack could not be reserved",
            ),
            (
                Reservations {
                    frames: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluation call stack could not be reserved",
            ),
            (
                Reservations {
                    frames: |frames, count| frames.is_empty() && frames.try_reserve(count).is_ok(),
                    ..Reservations::DEFAULT
                },
                "evaluation call stack could not be reserved",
            ),
        ] {
            let run = || {
                evaluate_with_reservations(
                    &core,
                    MAX_EVALUATION_STEPS_PER_SOURCE,
                    |values, capacity| values.try_reserve_exact(capacity).is_ok(),
                    reservations,
                )
            };
            let first = run();
            assert_eq!(first, run());
            assert!(first.values().is_none());
            let [diagnostic] = first.diagnostics() else {
                panic!("an allocation failure must produce exactly one diagnostic");
            };
            assert_eq!(
                diagnostic.message(),
                "reference evaluation result allocation failed"
            );
            assert_eq!(diagnostic.label(), label);
            assert_eq!(diagnostic.primary_span(), core.functions[1].name_span);
        }
    }

    /// Runs every accepted worst case through the whole pipeline on a thread
    /// whose stack is far smaller than a default thread's, unoptimized.
    #[test]
    fn deepest_accepted_sources_fit_in_one_mebibyte_of_stack() {
        use crate::parser::{MAX_BINDINGS_PER_BODY, MAX_EXPRESSION_HEIGHT, MAX_EXPRESSION_NESTING};
        let nested = |prefix: &str, core: &str, suffix: &str| {
            format!(
                "{}{core}{}",
                prefix.repeat(MAX_EXPRESSION_NESTING),
                suffix.repeat(MAX_EXPRESSION_NESTING)
            )
        };
        let bodies = [
            nested("(", "x", ")"),
            nested("~", "x", ""),
            nested("g(", "x", ")"),
            nested("x + x * (", "x", ")"),
            nested("x + x * g(", "x", ")"),
            nested("x ^ (", "x", ")"),
            format!("x{}", " ^ x".repeat(MAX_EXPRESSION_HEIGHT - 1)),
            format!("(x{})", " + x".repeat(MAX_EXPRESSION_HEIGHT - 2)),
            nested("(", "x", " as Word[32])"),
            nested("g(", "x", " as Word[32])"),
            format!(
                "{}x",
                (0..MAX_BINDINGS_PER_BODY)
                    .map(|index| format!("let v{index}: Word[32] = {};", nested("(", "x", ")")))
                    .collect::<String>()
            ),
        ];
        let sources = bodies
            .iter()
            .map(|body| {
                format!(
                    "edition 2026; module m {{\n  spec g(x: Word[32]) -> Word[32] {{ x }}\n  \
                     spec f(x: Word[32]) -> Word[32] {{ {body} }}\n  \
                     spec root() -> Word[32] {{ f(0x9e3779b9) }}\n}}\n"
                )
            })
            .collect::<Vec<_>>();
        let worker = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || {
                sources
                    .iter()
                    .map(|text| {
                        let (_, core) = analyzed(text);
                        evaluate(&core).values().map(<[EvaluatedFunction]>::len)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap();
        assert_eq!(worker.join().unwrap(), vec![Some(1); bodies.len()]);
    }

    #[test]
    fn conversions_match_a_wide_reference_for_every_type_pair() {
        let modulus = |bits: u32| 1_i128 << bits;
        let mut sources: Vec<(String, i128)> = [
            0,
            1,
            -1,
            255,
            256,
            -256,
            65_535,
            1 << 32,
            -(1 << 32) - 1,
            i128::from(u64::MAX),
            i128::from(u64::MAX) + 1,
            -i128::from(u64::MAX) - 1,
            (1 << 100) + 0x1234_5678,
            i128::MAX,
            -i128::MAX,
        ]
        .into_iter()
        .map(|value| (String::from("Int"), value))
        .collect();
        for (name, bits) in WORDS {
            for value in word_corpus(bits) {
                sources.push((String::from(name), i128::try_from(value).unwrap()));
            }
        }
        let targets = [("Int", None), ("Word[8]", Some(8)), ("Word[16]", Some(16))]
            .into_iter()
            .chain([("Word[32]", Some(32)), ("Word[64]", Some(64))]);
        let targets = targets.collect::<Vec<_>>();
        let mut members = String::new();
        let mut expected = Vec::new();
        for (index, (from, value)) in sources.iter().enumerate() {
            for (target_index, (to, bits)) in targets.iter().enumerate() {
                let literal = if from == "Int" {
                    value.to_string()
                } else {
                    format!("{value:#x}")
                };
                let name = format!("c{index}_{target_index}");
                members.push_str(&format!(
                    "  spec {name}() -> {to} {{ let v: {from} = {literal}; v as {to} }}\n"
                ));
                let rendered = match bits {
                    None => value.to_string(),
                    Some(bits) => render_word(
                        *bits,
                        u128::try_from(value.rem_euclid(modulus(*bits))).unwrap(),
                    ),
                };
                expected.push(format!("{name} = {rendered}"));
            }
        }
        assert_eq!(values_of(&members), expected);
    }

    #[test]
    fn bindings_are_evaluated_once_in_order_and_read_from_their_slots() {
        let members = concat!(
            "  spec f(x: Word[32], y: Word[32]) -> Word[32] {\n",
            "    let a: Word[32] = x + y;\n",
            "    let b: Word[32] = a ^ x;\n",
            "    let c: Word[32] = g(b, a);\n",
            "    (a | b) + c\n",
            "  }\n",
            "  spec g(p: Word[32], q: Word[32]) -> Word[32] {\n",
            "    let r: Word[32] = p - q;\n",
            "    r <<< 5\n",
            "  }\n",
            "  spec run() -> Word[32] { f(0x01234567, 0x89abcdef) }\n",
            "  spec nested() -> Int { let n: Int = 3; let m: Int = h(n * n) + n; m }\n",
            "  spec h(k: Int) -> Int { let twice: Int = k + k; twice * twice }\n",
        );
        let (x, y) = (0x0123_4567_u32, 0x89ab_cdef_u32);
        let a = x.wrapping_add(y);
        let b = a ^ x;
        let c = b.wrapping_sub(a).rotate_left(5);
        let result = (a | b).wrapping_add(c);
        assert_eq!(
            values_of(members),
            [
                format!("run = {}", render_word(32, u128::from(result))),
                String::from("nested = 327"),
            ]
        );
    }

    #[test]
    fn inconsistent_locals_and_conversions_fail_closed() {
        let base = core(concat!(
            "edition 2026; module m {\n",
            "  spec f() -> Word[8] { let t: Word[8] = 1; t }\n",
            "  spec g() -> Word[8] { let n: Int = 1; n as Word[8] }\n",
            "}\n"
        ));
        let mutations: [fn(&mut CoreModule); 4] = [
            // The body reads a binding that does not exist.
            |core| core.functions[0].locals.clear(),
            // A binding reads its own slot.
            |core| core.functions[0].locals[0].value.nodes[0].kind = CoreNodeKind::Local(0),
            // A conversion claims an operand of the wrong kind.
            |core| {
                core.functions[1].body.nodes[1].kind = CoreNodeKind::Convert {
                    from: CoreType::Word8,
                };
            },
            // A binding leaves no value behind.
            |core| core.functions[0].locals[0].value.nodes.clear(),
        ];
        for (index, mutate) in mutations.iter().enumerate() {
            let mut core = base.clone();
            mutate(&mut core);
            let result = evaluate(&core);
            assert_eq!(result, evaluate(&core), "case {index}");
            assert!(result.values().is_none(), "case {index}");
            assert_eq!(
                result.diagnostics()[0].message(),
                "reference evaluation received inconsistent Core",
                "case {index}"
            );
        }
    }

    #[test]
    fn word_to_int_conversion_allocation_failure_returns_no_values() {
        let core = core(concat!(
            "edition 2026; module m {\n",
            "  spec i() -> Int { let w: Word[64] = 0xffffffffffffffff; w as Int }\n",
            "}\n"
        ));
        let reservations = Reservations {
            value_limbs: |_, _| false,
            ..Reservations::DEFAULT
        };
        let result = evaluate_with_reservations(
            &core,
            MAX_EVALUATION_STEPS_PER_SOURCE,
            |values, capacity| values.try_reserve_exact(capacity).is_ok(),
            reservations,
        );
        assert!(result.values().is_none());
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(
            result.diagnostics()[0].message(),
            "reference evaluation result allocation failed"
        );
        assert_eq!(
            result.diagnostics()[0].label(),
            "exact integer storage could not be reserved"
        );
    }
}
