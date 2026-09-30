//! Deterministic reference evaluation for typed Orange Core.
//!
//! Evaluation is an explicit stack machine over postorder Core expressions,
//! so neither expression depth nor call depth uses the host call stack.

use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::core::{
    ArrayType, CoreArray, CoreBinding, CoreConditional, CoreExpression, CoreFunction,
    CoreFunctionId, CoreModule, CoreNode, CoreNodeKind, CoreTuple, CoreType, CoreValue,
    ExactInteger, MAX_EXACT_INTEGER_BITS, Modulus, Residue, TupleType,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::{BinaryOperator, ByteOrder, UnaryOperator};
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
    /// The values of the function's sizes in the instance evaluated, empty
    /// for a function without sizes.
    sizes: Vec<u32>,
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

    /// Returns the values of the function's sizes in the instance
    /// evaluated, or an empty slice for a function without sizes.
    #[must_use]
    pub fn sizes(&self) -> &[u32] {
        &self.sizes
    }

    /// Returns the statically checked result type.
    #[must_use]
    pub fn result_type(&self) -> CoreType {
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
        write!(formatter, "{}::{}", self.module(), self.name())?;
        // An instance of a sized function is named as a call names it.
        if let Some((first, rest)) = self.sizes.split_first() {
            write!(formatter, "[{first}")?;
            for size in rest {
                write!(formatter, ", {size}")?;
            }
            formatter.write_str("]")?;
        }
        write!(formatter, ": {} = {}", self.result_type(), self.value())
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

/// Evaluates every typed Core function of the root module without
/// parameters, in source order.
///
/// Functions with parameters, and every function of a used module, are
/// evaluated only through calls. One step budget covers the whole program.
#[must_use]
pub fn evaluate(core: &CoreModule) -> EvaluationResult {
    evaluate_with_limit(core, MAX_EVALUATION_STEPS_PER_SOURCE)
}

/// The result of calling one function through an [`Evaluator`].
///
/// ```compile_fail
/// use orange_compiler::CallResult;
///
/// fn forge_steps(result: &mut CallResult) {
///     result.steps = 0;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallResult {
    /// The exact result, present only when the call produced no diagnostics.
    value: Option<CoreValue>,
    /// Evaluation-resource diagnostics.
    diagnostics: Vec<Diagnostic>,
    /// Steps charged to the call, including a step that exceeded its limit.
    steps: usize,
}

impl CallResult {
    /// Returns the exact result, or `None` after evaluation failure.
    #[must_use]
    pub const fn value(&self) -> Option<&CoreValue> {
        self.value.as_ref()
    }

    /// Consumes this result and returns its value, if produced.
    #[must_use]
    pub fn into_value(self) -> Option<CoreValue> {
        self.value
    }

    /// Returns evaluation-resource diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns the steps the call used; after failure, those completed
    /// before it stopped.
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.steps
    }

    /// Returns whether the call did not produce a value.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.value.is_none()
    }
}

/// A typed Core module prepared for calls on values the host supplies.
///
/// [`evaluate`] evaluates every entry function without parameters under one
/// budget per program. An evaluator instead calls one function at a time on
/// arguments of exactly its parameter types, and gives every call its own
/// step limit. Literals are prepared once and shared by every call.
pub struct Evaluator<'core> {
    machine: Machine<'core>,
}

impl fmt::Debug for Evaluator<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Evaluator")
            .field("module", &self.machine.core.name)
            .finish_non_exhaustive()
    }
}

impl<'core> Evaluator<'core> {
    /// Prepares `core` for calls, or returns `None` when storage for its
    /// shared literals cannot be reserved.
    #[must_use]
    pub fn new(core: &'core CoreModule) -> Option<Self> {
        Some(Self {
            machine: Machine {
                core,
                literals: share_literals(core)?,
                steps: 0,
                step_limit: 0,
                reservations: Reservations::DEFAULT,
                stack: Vec::new(),
                frames: Vec::new(),
                inner_frames: 0,
            },
        })
    }

    /// Returns the prepared module.
    #[must_use]
    pub const fn module(&self) -> &'core CoreModule {
        self.machine.core
    }

    /// Returns the root module's function named `name`, if there is one and
    /// it declares no size parameters.
    ///
    /// Functions of the modules the root uses are reached only through the
    /// root's calls, so two modules may each declare a function of one name.
    /// A function with size parameters is one Core function per instance, all
    /// of one name, and is reached one instance at a time through
    /// [`Evaluator::instance`].
    #[must_use]
    pub fn function(&self, name: &str) -> Option<&'core CoreFunction> {
        self.instance(name, &[])
    }

    /// Returns the instance of the root module's function named `name` whose
    /// sizes, in declaration order, are `sizes`, if there is one. Empty
    /// `sizes` name a function without size parameters.
    #[must_use]
    pub fn instance(&self, name: &str, sizes: &[u32]) -> Option<&'core CoreFunction> {
        self.machine
            .core
            .entry_functions()
            .iter()
            .find(|function| function.name == name && function.sizes() == sizes)
    }

    /// Calls `function` on `arguments` within `step_limit` evaluation steps.
    ///
    /// Returns `None`, before any evaluation, unless `function` belongs to
    /// the prepared module and `arguments` have exactly its parameter types.
    #[must_use]
    pub fn call(
        &mut self,
        function: &'core CoreFunction,
        arguments: &[CoreValue],
        step_limit: usize,
    ) -> Option<CallResult> {
        let index = usize::try_from(function.id.index()).ok()?;
        let owned = self.machine.core.functions.get(index)?;
        if !std::ptr::eq(owned, function)
            || function.parameters.len() != arguments.len()
            || function
                .parameters
                .iter()
                .zip(arguments)
                .any(|(ty, argument)| argument.ty() != *ty)
        {
            return None;
        }
        let mut diagnostics = Vec::new();
        if !reserve_diagnostics(&mut diagnostics, 1) {
            return Some(CallResult {
                value: None,
                diagnostics,
                steps: 0,
            });
        }
        let machine = &mut self.machine;
        machine.steps = 0;
        machine.step_limit = step_limit;
        let value = host_values(arguments, machine.reservations)
            .and_then(|arguments| machine.run(function, arguments))
            .and_then(|value| result_value(value, &function.result_type, machine.reservations));
        let steps = machine.steps;
        // Release the call's values and frames; their storage is kept.
        machine.stack.clear();
        machine.frames.clear();
        machine.inner_frames = 0;
        Some(match value {
            Ok(value) => CallResult {
                value: Some(value),
                diagnostics,
                steps,
            },
            Err(stop) => CallResult {
                value: None,
                diagnostics: stopped(diagnostics, function, stop, step_limit, steps == 0)
                    .diagnostics,
                steps,
            },
        })
    }
}

/// Copies host values into the machine's shared representation.
fn host_values(arguments: &[CoreValue], reservations: Reservations) -> Result<Vec<Value>, Stop> {
    let mut values = Vec::new();
    if !(reservations.stack)(&mut values, arguments.len()) {
        return Err(Stop::Allocation(
            "evaluation value stack could not be reserved",
        ));
    }
    for argument in arguments {
        values.push(host_value(argument, reservations)?);
    }
    Ok(values)
}

fn host_value(value: &CoreValue, reservations: Reservations) -> Result<Value, Stop> {
    Ok(match value {
        CoreValue::Int(value) => Value::Int(Rc::new(
            value
                .try_clone_with_reservation(reservations.value_limbs)
                .ok_or(Stop::Allocation(
                    "exact integer storage could not be reserved",
                ))?,
        )),
        CoreValue::Bool(value) => Value::Bool(*value),
        CoreValue::Mod(residue) => Value::Mod(Rc::new(
            residue
                .value()
                .try_clone_with_reservation(reservations.value_limbs)
                .ok_or(Stop::Allocation(
                    "exact integer storage could not be reserved",
                ))?,
        )),
        CoreValue::Array(array) => {
            let mut elements = Vec::new();
            if !(reservations.array)(&mut elements, array.elements().len()) {
                return Err(Stop::Allocation(
                    "evaluation array storage could not be reserved",
                ));
            }
            for element in array.elements() {
                if matches!(element, CoreValue::Array(_) | CoreValue::Tuple(_)) {
                    return Err(Stop::InconsistentCore);
                }
                elements.push(host_value(element, reservations)?);
            }
            Value::Array(Rc::new(ArrayValue {
                ty: array.ty(),
                elements,
            }))
        }
        CoreValue::Tuple(tuple) => {
            let mut elements = Vec::new();
            if !(reservations.array)(&mut elements, tuple.elements().len()) {
                return Err(Stop::Allocation(
                    "evaluation tuple storage could not be reserved",
                ));
            }
            // A tuple holds no tuple, so this recursion is one level deep.
            for element in tuple.elements() {
                if matches!(element, CoreValue::Tuple(_)) {
                    return Err(Stop::InconsistentCore);
                }
                elements.push(host_value(element, reservations)?);
            }
            Value::Tuple(Rc::new(TupleValue {
                ty: tuple.ty().clone(),
                elements,
            }))
        }
        word => Value::Word(word.word_as_u64().ok_or(Stop::InconsistentCore)?),
    })
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

/// A runtime value. Exact integers, residues, and arrays are shared so that
/// loading a literal, parameter, or binding never copies their contents.
#[derive(Clone, Debug)]
enum Value {
    Int(Rc<ExactInteger>),
    Bool(bool),
    Word(u64),
    /// The least residue of an element of `Mod[m]`, from 0 through m - 1;
    /// the modulus is the type's.
    Mod(Rc<ExactInteger>),
    Array(Rc<ArrayValue>),
    Tuple(Rc<TupleValue>),
}

/// An array value with the type it was built at.
#[derive(Debug)]
struct ArrayValue {
    ty: ArrayType,
    elements: Vec<Value>,
}

/// A tuple value with the type it was built at.
#[derive(Debug)]
struct TupleValue {
    ty: TupleType,
    elements: Vec<Value>,
}

/// One active function evaluation, or one active loop step or conditional
/// branch within it.
///
/// A function frame's values occupy the shared stack from `base`: its
/// arguments, then the values of its completed bindings, then intermediate
/// values. A loop frame shares its function's `base` and keeps its index and
/// accumulator itself; its step's values lie above the stack length at which
/// the loop began. A branch frame shares its function's `base` too; its
/// values lie above the stack length at which the branch began. A step's or
/// branch's `let` bindings take their slots there in order, each once its
/// value subtree is complete, and its intermediate values lie above them.
struct Frame<'core> {
    function: &'core CoreFunction,
    /// The expression being evaluated; see [`expression_part`].
    part: usize,
    /// Index of the next node of that expression to evaluate.
    next: usize,
    /// Index in the shared value stack of the function's first argument.
    base: usize,
    /// What this frame evaluates.
    kind: FrameKind,
}

/// The expression a frame evaluates.
enum FrameKind {
    /// A function's bindings and body.
    Body,
    /// One loop's steps.
    Loop(ActiveLoop),
    /// One branch of a conditional.
    Branch(ActiveBranch),
}

impl FrameKind {
    const fn active_loop(&self) -> Option<&ActiveLoop> {
        match self {
            Self::Loop(active) => Some(active),
            Self::Body | Self::Branch(_) => None,
        }
    }

    const fn active_branch(&self) -> Option<&ActiveBranch> {
        match self {
            Self::Branch(active) => Some(active),
            Self::Body | Self::Loop(_) => None,
        }
    }
}

/// The state of one branch being evaluated.
struct ActiveBranch {
    /// The conditional's number within its function.
    id: usize,
    /// Whether this is the `then` branch.
    then: bool,
    /// Stack length when the branch began.
    floor: usize,
    /// The number of the branch's bindings whose values are in their slots.
    bound: usize,
}

/// The state of one loop between steps.
struct ActiveLoop {
    /// The loop's number within its function.
    id: usize,
    /// The current index.
    index: u32,
    /// The current accumulator.
    accumulator: Value,
    /// Stack length when the loop began.
    floor: usize,
    /// The number of the current step's bindings whose values are in their
    /// slots.
    bound: usize,
}

/// Returns the `let` bindings of the branch of `conditional` that
/// `then` names.
fn branch_bindings(conditional: &CoreConditional, then: bool) -> &[CoreBinding] {
    if then {
        conditional.then_bindings()
    } else {
        conditional.else_bindings()
    }
}

/// Returns a function's binding value at `part`, its body when `part` is
/// the number of bindings, a loop's step for each of the next parts in loop
/// order, and then each conditional's `then` and `else` branches in
/// conditional order.
fn expression_part(function: &CoreFunction, part: usize) -> Option<&CoreExpression> {
    match function.locals.get(part) {
        Some(local) => Some(&local.value),
        None if part == function.locals.len() => Some(&function.body),
        None => {
            let id = part.checked_sub(function.locals.len())?.checked_sub(1)?;
            if let Some(r#loop) = function.loops.get(id) {
                return Some(&r#loop.step);
            }
            let branch = id.checked_sub(function.loops.len())?;
            let conditional = function.conditionals.get(branch.checked_div(2)?)?;
            Some(if branch.checked_rem(2)? == 0 {
                &conditional.then_branch
            } else {
                &conditional.else_branch
            })
        }
    }
}

/// Returns the part of the branch of conditional `id` that `condition`
/// selects.
fn branch_part(function: &CoreFunction, id: usize, condition: bool) -> Option<usize> {
    function
        .locals
        .len()
        .checked_add(1)?
        .checked_add(function.loops.len())?
        .checked_add(id.checked_mul(2)?)?
        .checked_add(usize::from(!condition))
}

/// Returns whether a value has exactly the type `ty`.
fn has_type(value: &Value, ty: &CoreType) -> bool {
    match (value, ty) {
        (Value::Int(_), CoreType::Int) | (Value::Bool(_), CoreType::Bool) => true,
        (Value::Word(word), ty) => word_mask(ty).is_some_and(|mask| (*word & !mask) == 0),
        (Value::Mod(value), CoreType::Mod(modulus)) => modulus.contains(value),
        (Value::Array(array), CoreType::Array(array_type)) => array.ty == *array_type,
        (Value::Tuple(tuple), CoreType::Tuple(tuple_type)) => tuple.ty == *tuple_type,
        _ => false,
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

/// Shared `Int`, residue, and array literals, indexed by function,
/// expression part, and node.
type SharedLiterals = Vec<Vec<Vec<Option<SharedLiteral>>>>;

/// A literal built once before evaluation and shared by every evaluation
/// of its node.
enum SharedLiteral {
    /// The value of an `Int` literal or the least residue of a residue
    /// literal.
    Integer(Rc<ExactInteger>),
    /// An array literal, such as a byte string.
    Array(Rc<ArrayValue>),
}

struct Machine<'core> {
    core: &'core CoreModule,
    literals: SharedLiterals,
    steps: usize,
    step_limit: usize,
    reservations: Reservations,
    stack: Vec<Value>,
    frames: Vec<Frame<'core>>,
    /// The number of loop and branch frames in `frames`.
    inner_frames: usize,
}

fn digits(value: &ExactInteger) -> usize {
    value.magnitude_digits()
}

/// The 32-bit digits of a modulus: the unit of the cost of arithmetic in
/// `Mod[m]`.
fn modulus_digits(modulus: Modulus) -> usize {
    modulus.bits().div_ceil(32)
}

const INTEGER_STORAGE: &str = "exact integer storage could not be reserved";

/// The steps of an `update` or `fill` of `length` elements: one for each 64
/// elements it writes, or part of 64, so that a step stays about as much
/// work as one limb operation and never costs more than one per element.
const fn bulk_cost(length: usize) -> usize {
    if length <= 64 { 1 } else { length.div_ceil(64) }
}

fn word_mask(ty: &CoreType) -> Option<u64> {
    match ty {
        CoreType::Int
        | CoreType::Bool
        | CoreType::Mod(_)
        | CoreType::Array(_)
        | CoreType::Tuple(_) => None,
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
        // The total rules x / 0 = 0 and x % 0 = x.
        BinaryOperator::Divide => left.checked_div(right).unwrap_or(0),
        BinaryOperator::Remainder => left.checked_rem(right).unwrap_or(left),
        BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight
        | BinaryOperator::RotateLeft
        | BinaryOperator::RotateRight
        | BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual
        | BinaryOperator::LogicalAnd
        | BinaryOperator::LogicalOr
        | BinaryOperator::Concat => return None,
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
        | BinaryOperator::Xor
        | BinaryOperator::Divide
        | BinaryOperator::Remainder
        | BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::Less
        | BinaryOperator::LessEqual
        | BinaryOperator::Greater
        | BinaryOperator::GreaterEqual
        | BinaryOperator::LogicalAnd
        | BinaryOperator::LogicalOr
        | BinaryOperator::Concat => None,
    }
}

/// Returns whether `ordering` satisfies the comparison `operator`.
fn compares(operator: BinaryOperator, ordering: std::cmp::Ordering) -> Option<bool> {
    use std::cmp::Ordering::{Equal, Greater, Less};
    Some(match operator {
        BinaryOperator::Equal => ordering == Equal,
        BinaryOperator::NotEqual => ordering != Equal,
        BinaryOperator::Less => ordering == Less,
        BinaryOperator::LessEqual => ordering != Greater,
        BinaryOperator::Greater => ordering == Greater,
        BinaryOperator::GreaterEqual => ordering != Less,
        _ => return None,
    })
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
            Value::Bool(_) | Value::Word(_) | Value::Mod(_) | Value::Array(_) | Value::Tuple(_) => {
                Err(Stop::InconsistentCore)
            }
        }
    }

    fn pop_bool(&mut self) -> Result<bool, Stop> {
        match self.pop()? {
            Value::Bool(value) => Ok(value),
            Value::Int(_) | Value::Word(_) | Value::Mod(_) | Value::Array(_) | Value::Tuple(_) => {
                Err(Stop::InconsistentCore)
            }
        }
    }

    fn pop_word(&mut self) -> Result<u64, Stop> {
        match self.pop()? {
            Value::Word(value) => Ok(value),
            Value::Int(_) | Value::Bool(_) | Value::Mod(_) | Value::Array(_) | Value::Tuple(_) => {
                Err(Stop::InconsistentCore)
            }
        }
    }

    /// Pops a least residue of `Mod[modulus]`.
    fn pop_residue(&mut self, modulus: Modulus) -> Result<Rc<ExactInteger>, Stop> {
        match self.pop()? {
            Value::Mod(value) if modulus.contains(&value) => Ok(value),
            Value::Int(_)
            | Value::Bool(_)
            | Value::Word(_)
            | Value::Mod(_)
            | Value::Array(_)
            | Value::Tuple(_) => Err(Stop::InconsistentCore),
        }
    }

    /// Applies `+`, `-`, `*`, or `/` in `Mod[modulus]` to the top two
    /// values. Addition and subtraction cost d + 1 steps for a modulus of d
    /// 32-bit digits, multiplication 2d^2 + 1 for the product and its
    /// reduction, and division 64d^2 + 1 for the inverse, which the extended
    /// Euclidean algorithm finds in at most about 46d rounds of O(d) work.
    fn residue_binary(
        &mut self,
        operator: BinaryOperator,
        modulus: Modulus,
    ) -> Result<Value, Stop> {
        let right = self.pop_residue(modulus)?;
        let left = self.pop_residue(modulus)?;
        let reserve = self.reservations.value_limbs;
        let digits = modulus_digits(modulus);
        let square = digits.checked_mul(digits).ok_or(Stop::Steps)?;
        let exact = match operator {
            BinaryOperator::Add | BinaryOperator::Subtract => {
                self.charge(digits.saturating_add(1))?;
                if operator == BinaryOperator::Add {
                    left.add(&right, reserve)
                } else {
                    left.subtract(&right, reserve)
                }
            }
            BinaryOperator::Multiply => {
                self.charge(square.checked_mul(2).ok_or(Stop::Steps)?.saturating_add(1))?;
                left.multiply(&right, reserve)
            }
            // x / y = x y^-1 when y is a unit, and 0 otherwise: x / 0 = 0.
            BinaryOperator::Divide => {
                self.charge(square.checked_mul(64).ok_or(Stop::Steps)?.saturating_add(1))?;
                modulus
                    .inverse(&right, reserve)
                    .and_then(|inverse| left.multiply(&inverse, reserve))
            }
            _ => return Err(Stop::InconsistentCore),
        };
        let reduced = exact
            .and_then(|exact| modulus.reduce(&exact, reserve))
            .ok_or(Stop::Allocation(INTEGER_STORAGE))?;
        Ok(Value::Mod(Rc::new(reduced)))
    }

    /// Pops an operand of type `from` and returns its integer value: an
    /// `Int` itself, a word's unsigned value, or a residue's least residue.
    fn pop_integer(&mut self, from: &CoreType) -> Result<Rc<ExactInteger>, Stop> {
        match from {
            CoreType::Int => self.pop_int(),
            CoreType::Mod(modulus) => self.pop_residue(*modulus),
            _ => {
                let mask = word_mask(from).ok_or(Stop::InconsistentCore)?;
                let word = self.pop_word()?;
                if word & !mask != 0 {
                    return Err(Stop::InconsistentCore);
                }
                ExactInteger::from_u64(word, self.reservations.value_limbs)
                    .map(Rc::new)
                    .ok_or(Stop::Allocation(INTEGER_STORAGE))
            }
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
        for element in self.stack.drain(start..) {
            if !has_type(&element, &ty.element()) {
                return Err(Stop::InconsistentCore);
            }
            elements.push(element);
        }
        Ok(Value::Array(Rc::new(ArrayValue { ty, elements })))
    }

    /// Builds a tuple of type `ty` from the top `count` values of the
    /// stack, which lie above `floor`.
    fn build_tuple(&mut self, ty: &TupleType, count: usize, floor: usize) -> Result<Value, Stop> {
        if ty.elements().len() != count {
            return Err(Stop::InconsistentCore);
        }
        let start = self
            .stack
            .len()
            .checked_sub(count)
            .filter(|start| *start >= floor)
            .ok_or(Stop::InconsistentCore)?;
        let mut elements = Vec::new();
        if !(self.reservations.array)(&mut elements, count) {
            return Err(Stop::Allocation(
                "evaluation tuple storage could not be reserved",
            ));
        }
        for (element, element_type) in self.stack.drain(start..).zip(ty.elements()) {
            if !has_type(&element, element_type) {
                return Err(Stop::InconsistentCore);
            }
            elements.push(element);
        }
        Ok(Value::Tuple(Rc::new(TupleValue {
            ty: ty.clone(),
            elements,
        })))
    }

    fn checked_int(&self, value: ExactInteger, span: Span) -> Result<Value, Stop> {
        if value.magnitude_bits() > MAX_EXACT_INTEGER_BITS {
            return Err(Stop::IntegerBits(span));
        }
        Ok(Value::Int(Rc::new(value)))
    }

    /// Evaluates one function to completion on `arguments`, which have
    /// exactly its parameter types.
    fn run(&mut self, root: &'core CoreFunction, arguments: Vec<Value>) -> Result<Value, Stop> {
        self.stack.clear();
        self.frames.clear();
        self.inner_frames = 0;
        if root.parameters.len() != arguments.len() {
            return Err(Stop::InconsistentCore);
        }
        // The arguments occupy the stack from the root frame's base, 0.
        if !(self.reservations.stack)(&mut self.stack, arguments.len()) {
            return Err(Stop::Allocation(
                "evaluation value stack could not be reserved",
            ));
        }
        self.stack.extend(arguments);
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
            kind: FrameKind::Body,
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
                match frame.kind {
                    FrameKind::Loop(_) => {
                        self.finish_step()?;
                        continue;
                    }
                    FrameKind::Branch(_) => {
                        self.finish_branch()?;
                        continue;
                    }
                    FrameKind::Body => {}
                }
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
            self.bind(offset)?;
            let scope = self.scope()?;
            if let Some(frame) = self.frames.last_mut() {
                frame.next = frame.next.saturating_add(1);
            }
            self.step(function, base, part, offset, scope, node)?;
        }
    }

    /// Returns the number of bindings the top frame's node may read, and
    /// the stack length above which its expression's intermediate values
    /// lie.
    fn scope(&self) -> Result<(usize, usize), Stop> {
        let frame = self.frames.last().ok_or(Stop::InconsistentCore)?;
        let function = frame.function;
        let (visible, floor, bound) = match &frame.kind {
            FrameKind::Loop(active) => (
                function
                    .loops
                    .get(active.id)
                    .map(|r#loop| r#loop.visible_locals),
                active.floor,
                active.bound,
            ),
            FrameKind::Branch(active) => (
                function
                    .conditionals
                    .get(active.id)
                    .map(|conditional| conditional.visible_locals),
                active.floor,
                active.bound,
            ),
            FrameKind::Body => {
                return Ok((
                    frame.part,
                    frame
                        .base
                        .checked_add(function.parameters.len())
                        .and_then(|floor| floor.checked_add(frame.part))
                        .ok_or(Stop::InconsistentCore)?,
                ));
            }
        };
        let visible = visible
            .and_then(|visible| usize::try_from(visible).ok())
            .ok_or(Stop::InconsistentCore)?;
        let floor = floor.checked_add(bound).ok_or(Stop::InconsistentCore)?;
        Ok((visible, floor))
    }

    /// Gives the next binding of the step or branch in the top frame its
    /// slot when its value subtree ends before the node at `offset`: its
    /// value, the one value above the step's or branch's earlier bindings,
    /// stays on the stack.
    fn bind(&mut self, offset: usize) -> Result<(), Stop> {
        let Some(frame) = self.frames.last_mut() else {
            return Err(Stop::InconsistentCore);
        };
        let function = frame.function;
        let (bindings, floor, bound) = match &mut frame.kind {
            FrameKind::Body => return Ok(()),
            FrameKind::Loop(active) => (
                function
                    .loops
                    .get(active.id)
                    .map(|r#loop| r#loop.bindings()),
                active.floor,
                &mut active.bound,
            ),
            FrameKind::Branch(active) => (
                function
                    .conditionals
                    .get(active.id)
                    .map(|conditional| branch_bindings(conditional, active.then)),
                active.floor,
                &mut active.bound,
            ),
        };
        let bindings = bindings.ok_or(Stop::InconsistentCore)?;
        let Some(binding) = bindings.get(*bound) else {
            return Ok(());
        };
        if usize::try_from(binding.end()).ok() != Some(offset) {
            return Ok(());
        }
        let slots = floor
            .checked_add(*bound)
            .and_then(|slots| slots.checked_add(1))
            .ok_or(Stop::InconsistentCore)?;
        if self.stack.len() != slots
            || !self
                .stack
                .last()
                .is_some_and(|value| has_type(value, &binding.ty()))
        {
            return Err(Stop::InconsistentCore);
        }
        *bound = slots.saturating_sub(floor);
        Ok(())
    }

    /// Pops the value of the step or branch whose bindings are `bindings`
    /// and whose values lie above `floor`, and removes the bindings' values
    /// once every one of them has had its slot.
    fn finish_block(
        &mut self,
        bindings: &[CoreBinding],
        floor: usize,
        bound: usize,
    ) -> Result<Value, Stop> {
        let value = self.pop()?;
        if bound != bindings.len() || floor.checked_add(bound) != Some(self.stack.len()) {
            return Err(Stop::InconsistentCore);
        }
        self.stack.truncate(floor);
        Ok(value)
    }

    /// Completes one step of the loop in the top frame: its value becomes
    /// the accumulator, and either the next step begins or the loop's value
    /// replaces the loop.
    fn finish_step(&mut self) -> Result<(), Stop> {
        let Some(frame) = self.frames.last() else {
            return Err(Stop::InconsistentCore);
        };
        let function = frame.function;
        let FrameKind::Loop(active) = &frame.kind else {
            return Err(Stop::InconsistentCore);
        };
        let r#loop = function
            .loops
            .get(active.id)
            .ok_or(Stop::InconsistentCore)?;
        let (floor, bound) = (active.floor, active.bound);
        let value = self.finish_block(r#loop.bindings(), floor, bound)?;
        let Some(frame) = self.frames.last_mut() else {
            return Err(Stop::InconsistentCore);
        };
        let FrameKind::Loop(active) = &mut frame.kind else {
            return Err(Stop::InconsistentCore);
        };
        if self.stack.len() != active.floor || !has_type(&value, &r#loop.ty) {
            return Err(Stop::InconsistentCore);
        }
        active.bound = 0;
        active.accumulator = value;
        active.index = active.index.checked_add(1).ok_or(Stop::InconsistentCore)?;
        if active.index < r#loop.end {
            frame.next = 0;
            // One step for each iteration.
            return self.charge(1);
        }
        let Some(Frame {
            kind: FrameKind::Loop(finished),
            ..
        }) = self.frames.pop()
        else {
            return Err(Stop::InconsistentCore);
        };
        self.inner_frames = self.inner_frames.saturating_sub(1);
        self.push(finished.accumulator)
    }

    /// Completes the branch in the top frame: its value replaces the
    /// conditional.
    fn finish_branch(&mut self) -> Result<(), Stop> {
        let Some(Frame {
            function,
            kind: FrameKind::Branch(finished),
            ..
        }) = self.frames.pop()
        else {
            return Err(Stop::InconsistentCore);
        };
        let conditional = function
            .conditionals
            .get(finished.id)
            .ok_or(Stop::InconsistentCore)?;
        let value = self.finish_block(
            branch_bindings(conditional, finished.then),
            finished.floor,
            finished.bound,
        )?;
        if self.stack.len() != finished.floor || !has_type(&value, &conditional.ty) {
            return Err(Stop::InconsistentCore);
        }
        self.inner_frames = self.inner_frames.saturating_sub(1);
        self.push(value)
    }

    /// Returns whether the loops active in the top frames of the current
    /// function are exactly `scope`, outermost first.
    fn active_loops_are(&self, scope: &[u32]) -> bool {
        let active = self
            .frames
            .iter()
            .rev()
            .take_while(|frame| !matches!(frame.kind, FrameKind::Body))
            .filter_map(|frame| frame.kind.active_loop())
            .map(|active| active.id);
        active.eq(scope
            .iter()
            .rev()
            .map(|id| usize::try_from(*id).unwrap_or(usize::MAX)))
    }

    /// Begins the branch of conditional `id` of `function` that the popped
    /// condition selects.
    fn begin_branch(
        &mut self,
        function: &'core CoreFunction,
        base: usize,
        id: u32,
        ty: &CoreType,
    ) -> Result<(), Stop> {
        self.charge(1)?;
        let condition = self.pop_bool()?;
        let index = usize::try_from(id).map_err(|_| Stop::InconsistentCore)?;
        let conditional = function
            .conditionals
            .get(index)
            .ok_or(Stop::InconsistentCore)?;
        // The loops active in this function must be exactly those that
        // enclose the conditional.
        if !self.active_loops_are(&conditional.scope) || conditional.ty != *ty {
            return Err(Stop::InconsistentCore);
        }
        let part = branch_part(function, index, condition).ok_or(Stop::InconsistentCore)?;
        if !(self.reservations.frames)(&mut self.frames, 1) {
            return Err(Stop::Allocation(
                "evaluation call stack could not be reserved",
            ));
        }
        self.frames.push(Frame {
            function,
            part,
            next: 0,
            base,
            kind: FrameKind::Branch(ActiveBranch {
                id: index,
                then: condition,
                floor: self.stack.len(),
                bound: 0,
            }),
        });
        self.inner_frames = self.inner_frames.saturating_add(1);
        Ok(())
    }

    /// Returns the value of binding `index` of a step or branch whose
    /// bindings are `bindings`, whose values lie above `floor`, and whose
    /// first `bound` bindings have had their slots; the binding must be one
    /// of those.
    fn bound_value(
        &self,
        bindings: &[CoreBinding],
        (floor, bound): (usize, usize),
        index: u32,
        ty: &CoreType,
    ) -> Result<Value, Stop> {
        let index = usize::try_from(index)
            .ok()
            .filter(|index| *index < bound)
            .ok_or(Stop::InconsistentCore)?;
        let binding = bindings.get(index).ok_or(Stop::InconsistentCore)?;
        let slot = floor.checked_add(index).ok_or(Stop::InconsistentCore)?;
        let value = self.stack.get(slot).ok_or(Stop::InconsistentCore)?;
        if binding.ty != *ty || !has_type(value, ty) {
            return Err(Stop::InconsistentCore);
        }
        Ok(value.clone())
    }

    /// Returns the branch of conditional `id` being evaluated in the
    /// function whose branch or step is being evaluated.
    fn active_branch(&self, id: u32) -> Result<&ActiveBranch, Stop> {
        let id = usize::try_from(id).map_err(|_| Stop::InconsistentCore)?;
        self.frames
            .iter()
            .rev()
            .take_while(|frame| !matches!(frame.kind, FrameKind::Body))
            .filter_map(|frame| frame.kind.active_branch())
            .find(|active| active.id == id)
            .ok_or(Stop::InconsistentCore)
    }

    /// Returns the innermost active loop numbered `id` of the function
    /// whose step is being evaluated.
    fn active_loop(&self, id: u32) -> Result<&ActiveLoop, Stop> {
        let id = usize::try_from(id).map_err(|_| Stop::InconsistentCore)?;
        self.frames
            .iter()
            .rev()
            .take_while(|frame| !matches!(frame.kind, FrameKind::Body))
            .filter_map(|frame| frame.kind.active_loop())
            .find(|active| active.id == id)
            .ok_or(Stop::InconsistentCore)
    }

    /// Pops an `Int` index and returns it as a position below `length`.
    fn pop_position(&mut self, length: usize) -> Result<usize, Stop> {
        let index = self.pop_int()?;
        index
            .to_i64()
            .and_then(|index| usize::try_from(index).ok())
            .filter(|index| *index < length)
            .ok_or(Stop::InconsistentCore)
    }

    /// Pops the `Int` end and then the `Int` start of a slice of `length`
    /// elements, which analysis proved to lie `length` apart from 0 up.
    /// The end is checked against the array when the run is taken.
    fn pop_bounds(&mut self, length: usize) -> Result<(usize, usize), Stop> {
        let position = |value: Rc<ExactInteger>| {
            value
                .to_i64()
                .and_then(|value| usize::try_from(value).ok())
                .ok_or(Stop::InconsistentCore)
        };
        let end = position(self.pop_int()?)?;
        let start = position(self.pop_int()?)?;
        if start.checked_add(length) != Some(end) {
            return Err(Stop::InconsistentCore);
        }
        Ok((start, end))
    }

    /// Begins the loop numbered `id` of `function` with the popped initial
    /// accumulator.
    fn begin_loop(
        &mut self,
        function: &'core CoreFunction,
        base: usize,
        id: u32,
        ty: &CoreType,
    ) -> Result<(), Stop> {
        self.charge(1)?;
        let accumulator = self.pop()?;
        let index = usize::try_from(id).map_err(|_| Stop::InconsistentCore)?;
        let r#loop = function.loops.get(index).ok_or(Stop::InconsistentCore)?;
        // The loops active in this function must be exactly those that
        // enclose this one.
        let enclosing = r#loop
            .scope
            .split_last()
            .filter(|(last, _)| **last == id)
            .map(|(_, enclosing)| enclosing)
            .ok_or(Stop::InconsistentCore)?;
        if !self.active_loops_are(enclosing)
            || r#loop.ty != *ty
            || r#loop.start >= r#loop.end
            || !has_type(&accumulator, ty)
        {
            return Err(Stop::InconsistentCore);
        }
        let part = function
            .locals
            .len()
            .checked_add(1)
            .and_then(|part| part.checked_add(index))
            .ok_or(Stop::InconsistentCore)?;
        if !(self.reservations.frames)(&mut self.frames, 1) {
            return Err(Stop::Allocation(
                "evaluation call stack could not be reserved",
            ));
        }
        self.frames.push(Frame {
            function,
            part,
            next: 0,
            base,
            kind: FrameKind::Loop(ActiveLoop {
                id: index,
                index: r#loop.start,
                accumulator,
                floor: self.stack.len(),
                bound: 0,
            }),
        });
        self.inner_frames = self.inner_frames.saturating_add(1);
        // One step for the first iteration.
        self.charge(1)
    }

    fn step(
        &mut self,
        function: &'core CoreFunction,
        base: usize,
        part: usize,
        offset: usize,
        (visible, floor): (usize, usize),
        node: &'core CoreNode,
    ) -> Result<(), Stop> {
        match &node.kind {
            CoreNodeKind::Literal(value) => {
                self.charge(1)?;
                let literal = value;
                let value = match literal {
                    CoreValue::Int(_) | CoreValue::Mod(_) | CoreValue::Array(_) => {
                        let index = usize::try_from(function.id.index())
                            .map_err(|_| Stop::InconsistentCore)?;
                        let shared = self
                            .literals
                            .get(index)
                            .and_then(|parts| parts.get(part))
                            .and_then(|literals| literals.get(offset))
                            .and_then(Option::as_ref)
                            .ok_or(Stop::InconsistentCore)?;
                        let value = match (literal, shared) {
                            (CoreValue::Int(_), SharedLiteral::Integer(shared)) => {
                                Value::Int(Rc::clone(shared))
                            }
                            (CoreValue::Mod(_), SharedLiteral::Integer(shared)) => {
                                Value::Mod(Rc::clone(shared))
                            }
                            (CoreValue::Array(_), SharedLiteral::Array(shared)) => {
                                Value::Array(Rc::clone(shared))
                            }
                            _ => return Err(Stop::InconsistentCore),
                        };
                        if !has_type(&value, &node.ty) {
                            return Err(Stop::InconsistentCore);
                        }
                        value
                    }
                    CoreValue::Bool(value) => Value::Bool(*value),
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
                // Only a binding before the current expression has a value.
                let index = usize::try_from(*index)
                    .ok()
                    .filter(|index| *index < visible)
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
                // Loop and branch frames do not count toward the call depth.
                if self.frames.len().saturating_sub(self.inner_frames) >= MAX_CALL_DEPTH {
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
                    .filter(|callee_base| *callee_base >= floor)
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
                    kind: FrameKind::Body,
                });
                Ok(())
            }
            CoreNodeKind::Unary(operator) => {
                let value = match (operator, word_mask(&node.ty)) {
                    (UnaryOperator::Not, None) if node.ty == CoreType::Bool => {
                        self.charge(1)?;
                        Value::Bool(!self.pop_bool()?)
                    }
                    (UnaryOperator::Negate, None) if node.ty == CoreType::Int => {
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
                    // -x = m - x for x != 0, and -0 = 0.
                    (UnaryOperator::Negate, None) if node.ty.modulus().is_some() => {
                        let modulus = node.ty.modulus().ok_or(Stop::InconsistentCore)?;
                        let operand = self.pop_residue(modulus)?;
                        self.charge(modulus_digits(modulus).saturating_add(1))?;
                        let reserve = self.reservations.value_limbs;
                        let negated = if operand.is_zero() {
                            operand.try_clone_with_reservation(reserve)
                        } else {
                            modulus
                                .to_exact(reserve)
                                .and_then(|modulus| modulus.subtract(&operand, reserve))
                        }
                        .ok_or(Stop::Allocation(INTEGER_STORAGE))?;
                        Value::Mod(Rc::new(negated))
                    }
                    _ => return Err(Stop::InconsistentCore),
                };
                self.push(value)
            }
            CoreNodeKind::Binary(operator) => {
                let value = if node.ty == CoreType::Bool {
                    self.charge(1)?;
                    let right = self.pop_bool()?;
                    let left = self.pop_bool()?;
                    // Both operands are evaluated: `&&` and `||` are strict.
                    Value::Bool(match operator {
                        BinaryOperator::LogicalAnd => left && right,
                        BinaryOperator::LogicalOr => left || right,
                        _ => return Err(Stop::InconsistentCore),
                    })
                } else if let Some(mask) = word_mask(&node.ty) {
                    self.charge(1)?;
                    let right = self.pop_word()?;
                    let left = self.pop_word()?;
                    Value::Word(
                        word_binary(*operator, mask, left, right).ok_or(Stop::InconsistentCore)?,
                    )
                } else if let Some(modulus) = node.ty.modulus() {
                    self.residue_binary(*operator, modulus)?
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
                        BinaryOperator::Divide | BinaryOperator::Remainder => {
                            self.charge(
                                digits(&left)
                                    .checked_mul(digits(&right).max(1))
                                    .ok_or(Stop::Steps)?
                                    .saturating_add(1),
                            )?;
                            if right.is_zero() {
                                // The total rules x / 0 = 0 and x % 0 = x.
                                if *operator == BinaryOperator::Divide {
                                    ExactInteger::from_u64(0, reserve)
                                } else {
                                    left.try_clone_with_reservation(reserve)
                                }
                            } else {
                                left.divide_euclid(&right, reserve)
                                    .map(|(quotient, remainder)| {
                                        if *operator == BinaryOperator::Divide {
                                            quotient
                                        } else {
                                            remainder
                                        }
                                    })
                            }
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
            CoreNodeKind::Compare { operator, operand } => {
                if node.ty != CoreType::Bool {
                    return Err(Stop::InconsistentCore);
                }
                let ordering = match operand {
                    CoreType::Int => {
                        let right = self.pop_int()?;
                        let left = self.pop_int()?;
                        self.charge(digits(&left).max(digits(&right)).saturating_add(1))?;
                        left.compare(&right)
                    }
                    CoreType::Bool
                        if !matches!(
                            operator,
                            BinaryOperator::Equal | BinaryOperator::NotEqual
                        ) =>
                    {
                        return Err(Stop::InconsistentCore);
                    }
                    CoreType::Bool => {
                        self.charge(1)?;
                        let right = self.pop_bool()?;
                        let left = self.pop_bool()?;
                        left.cmp(&right)
                    }
                    // Residues are compared for equality only.
                    CoreType::Mod(_)
                        if !matches!(
                            operator,
                            BinaryOperator::Equal | BinaryOperator::NotEqual
                        ) =>
                    {
                        return Err(Stop::InconsistentCore);
                    }
                    CoreType::Mod(modulus) => {
                        let right = self.pop_residue(*modulus)?;
                        let left = self.pop_residue(*modulus)?;
                        self.charge(modulus_digits(*modulus).saturating_add(1))?;
                        left.compare(&right)
                    }
                    ty => {
                        let mask = word_mask(ty).ok_or(Stop::InconsistentCore)?;
                        self.charge(1)?;
                        let right = self.pop_word()?;
                        let left = self.pop_word()?;
                        if (left | right) & !mask != 0 {
                            return Err(Stop::InconsistentCore);
                        }
                        left.cmp(&right)
                    }
                };
                let value = compares(*operator, ordering).ok_or(Stop::InconsistentCore)?;
                self.push(Value::Bool(value))
            }
            CoreNodeKind::Choose(id) => {
                if self.stack.len() <= floor {
                    return Err(Stop::InconsistentCore);
                }
                self.begin_branch(function, base, *id, &node.ty)
            }
            CoreNodeKind::Shift { operator, amount } => {
                self.charge(1)?;
                let bits = node.ty.word_bits().ok_or(Stop::InconsistentCore)?;
                let mask = word_mask(&node.ty).ok_or(Stop::InconsistentCore)?;
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
                // Elements are intermediate values of the current expression.
                let array = self.build_array(ty, length, floor)?;
                self.push(array)
            }
            CoreNodeKind::Select => {
                self.charge(1)?;
                let length = self
                    .stack
                    .len()
                    .checked_sub(2)
                    .filter(|below| *below >= floor)
                    .and_then(|below| match self.stack.get(below) {
                        Some(Value::Array(array)) if array.ty.element() == node.ty => {
                            Some(array.elements.len())
                        }
                        _ => None,
                    })
                    .ok_or(Stop::InconsistentCore)?;
                let position = self.pop_position(length)?;
                let Value::Array(array) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                let element = array
                    .elements
                    .get(position)
                    .cloned()
                    .ok_or(Stop::InconsistentCore)?;
                self.push(element)
            }
            CoreNodeKind::Update => {
                let ty = node.ty.as_array().ok_or(Stop::InconsistentCore)?;
                let length = usize::try_from(ty.length()).map_err(|_| Stop::InconsistentCore)?;
                // One step per 64 elements copied, or part of 64.
                self.charge(bulk_cost(length))?;
                if self
                    .stack
                    .len()
                    .checked_sub(3)
                    .is_none_or(|below| below < floor)
                {
                    return Err(Stop::InconsistentCore);
                }
                let value = self.pop()?;
                let position = self.pop_position(length)?;
                let Value::Array(array) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                if array.ty != ty || !has_type(&value, &ty.element()) {
                    return Err(Stop::InconsistentCore);
                }
                let mut elements = Vec::new();
                if !(self.reservations.array)(&mut elements, length) {
                    return Err(Stop::Allocation(
                        "evaluation array storage could not be reserved",
                    ));
                }
                elements.extend(array.elements.iter().cloned());
                let slot = elements.get_mut(position).ok_or(Stop::InconsistentCore)?;
                *slot = value;
                self.push(Value::Array(Rc::new(ArrayValue { ty, elements })))
            }
            CoreNodeKind::Fill => {
                let ty = node.ty.as_array().ok_or(Stop::InconsistentCore)?;
                let length = usize::try_from(ty.length()).map_err(|_| Stop::InconsistentCore)?;
                // One step per 64 elements written, or part of 64.
                self.charge(bulk_cost(length))?;
                if self.stack.len() <= floor {
                    return Err(Stop::InconsistentCore);
                }
                let element = self.pop()?;
                if !has_type(&element, &ty.element()) {
                    return Err(Stop::InconsistentCore);
                }
                let mut elements = Vec::new();
                if !(self.reservations.array)(&mut elements, length) {
                    return Err(Stop::Allocation(
                        "evaluation array storage could not be reserved",
                    ));
                }
                elements.extend(std::iter::repeat_n(element, length));
                self.push(Value::Array(Rc::new(ArrayValue { ty, elements })))
            }
            CoreNodeKind::Fold(id) => {
                if self.stack.len() <= floor {
                    return Err(Stop::InconsistentCore);
                }
                self.begin_loop(function, base, *id, &node.ty)
            }
            CoreNodeKind::LoopIndex(id) => {
                self.charge(1)?;
                if node.ty != CoreType::Int {
                    return Err(Stop::InconsistentCore);
                }
                let index = self.active_loop(*id)?.index;
                let value = ExactInteger::from_u64(u64::from(index), self.reservations.value_limbs)
                    .ok_or(Stop::Allocation(
                        "exact integer storage could not be reserved",
                    ))?;
                self.push(Value::Int(Rc::new(value)))
            }
            CoreNodeKind::Accumulator(id) => {
                self.charge(1)?;
                let accumulator = self.active_loop(*id)?.accumulator.clone();
                if !has_type(&accumulator, &node.ty) {
                    return Err(Stop::InconsistentCore);
                }
                self.push(accumulator)
            }
            CoreNodeKind::StepBinding { loop_id, index } => {
                self.charge(1)?;
                let active = self.active_loop(*loop_id)?;
                let r#loop = function
                    .loops
                    .get(active.id)
                    .ok_or(Stop::InconsistentCore)?;
                let value = self.bound_value(
                    r#loop.bindings(),
                    (active.floor, active.bound),
                    *index,
                    &node.ty,
                )?;
                self.push(value)
            }
            CoreNodeKind::BranchBinding { conditional, index } => {
                self.charge(1)?;
                let active = self.active_branch(*conditional)?;
                let bindings = function
                    .conditionals
                    .get(active.id)
                    .map(|conditional| branch_bindings(conditional, active.then))
                    .ok_or(Stop::InconsistentCore)?;
                let value =
                    self.bound_value(bindings, (active.floor, active.bound), *index, &node.ty)?;
                self.push(value)
            }
            CoreNodeKind::Index { index } => {
                self.charge(1)?;
                let Value::Array(array) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                if array.ty.element() != node.ty {
                    return Err(Stop::InconsistentCore);
                }
                let index = usize::try_from(*index).map_err(|_| Stop::InconsistentCore)?;
                let element = array
                    .elements
                    .get(index)
                    .cloned()
                    .ok_or(Stop::InconsistentCore)?;
                self.push(element)
            }
            CoreNodeKind::Tuple { elements } => {
                let ty = node.ty.as_tuple().ok_or(Stop::InconsistentCore)?;
                let count = usize::try_from(*elements).map_err(|_| Stop::InconsistentCore)?;
                // One step per element, as for an array; a tuple has at
                // least two.
                self.charge(count.max(1))?;
                // Elements are intermediate values of the current expression.
                let tuple = self.build_tuple(ty, count, floor)?;
                self.push(tuple)
            }
            CoreNodeKind::Project { index } => {
                self.charge(1)?;
                let Value::Tuple(tuple) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                if tuple.ty.element(*index) != Some(&node.ty) {
                    return Err(Stop::InconsistentCore);
                }
                let index = usize::try_from(*index).map_err(|_| Stop::InconsistentCore)?;
                let element = tuple
                    .elements
                    .get(index)
                    .cloned()
                    .ok_or(Stop::InconsistentCore)?;
                self.push(element)
            }
            CoreNodeKind::Concat => {
                let ty = node.ty.as_array().ok_or(Stop::InconsistentCore)?;
                let length = usize::try_from(ty.length()).map_err(|_| Stop::InconsistentCore)?;
                // One step per 64 elements written, or part of 64.
                self.charge(bulk_cost(length))?;
                if self
                    .stack
                    .len()
                    .checked_sub(2)
                    .is_none_or(|below| below < floor)
                {
                    return Err(Stop::InconsistentCore);
                }
                let (Value::Array(right), Value::Array(left)) = (self.pop()?, self.pop()?) else {
                    return Err(Stop::InconsistentCore);
                };
                if left.ty.element() != ty.element()
                    || right.ty.element() != ty.element()
                    || left.elements.len().checked_add(right.elements.len()) != Some(length)
                {
                    return Err(Stop::InconsistentCore);
                }
                let mut elements = Vec::new();
                if !(self.reservations.array)(&mut elements, length) {
                    return Err(Stop::Allocation(
                        "evaluation array storage could not be reserved",
                    ));
                }
                elements.extend(left.elements.iter().cloned());
                elements.extend(right.elements.iter().cloned());
                self.push(Value::Array(Rc::new(ArrayValue { ty, elements })))
            }
            CoreNodeKind::Slice => {
                let ty = node.ty.as_array().ok_or(Stop::InconsistentCore)?;
                let length = usize::try_from(ty.length()).map_err(|_| Stop::InconsistentCore)?;
                // One step per 64 elements copied, or part of 64.
                self.charge(bulk_cost(length))?;
                if self
                    .stack
                    .len()
                    .checked_sub(3)
                    .is_none_or(|below| below < floor)
                {
                    return Err(Stop::InconsistentCore);
                }
                let (start, end) = self.pop_bounds(length)?;
                let Value::Array(array) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                if array.ty.element() != ty.element() {
                    return Err(Stop::InconsistentCore);
                }
                let run = array
                    .elements
                    .get(start..end)
                    .ok_or(Stop::InconsistentCore)?;
                let mut elements = Vec::new();
                if !(self.reservations.array)(&mut elements, length) {
                    return Err(Stop::Allocation(
                        "evaluation array storage could not be reserved",
                    ));
                }
                elements.extend(run.iter().cloned());
                self.push(Value::Array(Rc::new(ArrayValue { ty, elements })))
            }
            CoreNodeKind::SliceUpdate => {
                let ty = node.ty.as_array().ok_or(Stop::InconsistentCore)?;
                let length = usize::try_from(ty.length()).map_err(|_| Stop::InconsistentCore)?;
                // One step per 64 elements copied, or part of 64.
                self.charge(bulk_cost(length))?;
                if self
                    .stack
                    .len()
                    .checked_sub(4)
                    .is_none_or(|below| below < floor)
                {
                    return Err(Stop::InconsistentCore);
                }
                let Value::Array(value) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                let (start, end) = self.pop_bounds(value.elements.len())?;
                let Value::Array(array) = self.pop()? else {
                    return Err(Stop::InconsistentCore);
                };
                if array.ty != ty || value.ty.element() != ty.element() {
                    return Err(Stop::InconsistentCore);
                }
                let mut elements = Vec::new();
                if !(self.reservations.array)(&mut elements, length) {
                    return Err(Stop::Allocation(
                        "evaluation array storage could not be reserved",
                    ));
                }
                elements.extend(array.elements.iter().cloned());
                let run = elements.get_mut(start..end).ok_or(Stop::InconsistentCore)?;
                run.clone_from_slice(&value.elements);
                self.push(Value::Array(Rc::new(ArrayValue { ty, elements })))
            }
            CoreNodeKind::Pack { from, order } => {
                if self
                    .stack
                    .len()
                    .checked_sub(1)
                    .is_none_or(|below| below < floor)
                {
                    return Err(Stop::InconsistentCore);
                }
                let value = self.pack(from, *order, &node.ty)?;
                self.push(value)
            }
            CoreNodeKind::Convert { from } => {
                self.charge(1)?;
                if *from == CoreType::Bool || node.ty == CoreType::Bool {
                    return Err(Stop::InconsistentCore);
                }
                let value = match (word_mask(from), &node.ty) {
                    (Some(_), ty) if word_mask(ty).is_some() => {
                        let mask = word_mask(ty).ok_or(Stop::InconsistentCore)?;
                        Value::Word(self.pop_word()? & mask)
                    }
                    // A residue's value is its least residue, and a
                    // conversion to `Mod[m]` reduces the operand's value
                    // modulo m, which costs one step per digit of the
                    // operand and of m.
                    (_, CoreType::Mod(modulus)) => {
                        let modulus = *modulus;
                        let value = self.pop_integer(from)?;
                        self.charge(digits(&value).saturating_mul(modulus_digits(modulus)))?;
                        let reduced = modulus
                            .reduce(&value, self.reservations.value_limbs)
                            .ok_or(Stop::Allocation(INTEGER_STORAGE))?;
                        Value::Mod(Rc::new(reduced))
                    }
                    (_, CoreType::Int) => Value::Int(self.pop_integer(from)?),
                    (_, ty) => {
                        let mask = word_mask(ty).ok_or(Stop::InconsistentCore)?;
                        Value::Word(self.pop_integer(from)?.modulo_2_64() & mask)
                    }
                };
                self.push(value)
            }
        }
    }
}

impl Machine<'_> {
    /// Pops a value of type `from` and converts it to `to` in `order`. The
    /// words on one side spell a number of their total width, most
    /// significant word first for `big` and least significant first for
    /// `little`: words convert to the words of the same number, to that
    /// number, or to its residue modulo m, and a number converts to the
    /// words of its residue modulo 2^width.
    #[inline(never)]
    fn pack(&mut self, from: &CoreType, order: ByteOrder, to: &CoreType) -> Result<Value, Stop> {
        let (bits, count) = from
            .words()
            .or_else(|| to.words())
            .ok_or(Stop::InconsistentCore)?;
        let width = bits.checked_mul(count).ok_or(Stop::InconsistentCore)?;
        if to
            .words()
            .is_some_and(|(to_bits, to_count)| to_bits.checked_mul(to_count) != Some(width))
        {
            return Err(Stop::InconsistentCore);
        }
        // One step for each 64 bits packed, or part of 64.
        let cost = usize::try_from(width.div_ceil(64)).map_err(|_| Stop::InconsistentCore)?;
        self.charge(cost)?;
        let digits_needed =
            usize::try_from(width.div_ceil(32)).map_err(|_| Stop::InconsistentCore)?;
        let mut limbs = Vec::new();
        if !(self.reservations.value_limbs)(&mut limbs, digits_needed) {
            return Err(Stop::Allocation(INTEGER_STORAGE));
        }
        limbs.resize(digits_needed, 0);
        if from.words().is_some() {
            let operand = self.pop()?;
            let words: &[Value] = match (&operand, from) {
                (Value::Array(array), CoreType::Array(ty)) if array.ty == *ty => &array.elements,
                (Value::Word(_), _) if from.as_array().is_none() => std::slice::from_ref(&operand),
                _ => return Err(Stop::InconsistentCore),
            };
            for (index, word) in words.iter().enumerate() {
                let Value::Word(word) = word else {
                    return Err(Stop::InconsistentCore);
                };
                let place = word_place(order, index, count)?;
                write_word(&mut limbs, bits, place, *word)?;
            }
        } else {
            let value = self.pop_integer(from)?;
            write_residue(&mut limbs, &value, width)?;
        }
        match to {
            CoreType::Int => Ok(Value::Int(Rc::new(ExactInteger::from_limbs(limbs)))),
            CoreType::Mod(modulus) => {
                let value = ExactInteger::from_limbs(limbs);
                // As for `as Mod[m]`: one step per digit of the value and
                // of m.
                self.charge(digits(&value).saturating_mul(modulus_digits(*modulus)))?;
                let reduced = modulus
                    .reduce(&value, self.reservations.value_limbs)
                    .ok_or(Stop::Allocation(INTEGER_STORAGE))?;
                Ok(Value::Mod(Rc::new(reduced)))
            }
            CoreType::Array(ty) => {
                let (bits, count) = to.words().ok_or(Stop::InconsistentCore)?;
                let length = usize::try_from(count).map_err(|_| Stop::InconsistentCore)?;
                let mut elements = Vec::new();
                if !(self.reservations.array)(&mut elements, length) {
                    return Err(Stop::Allocation(
                        "evaluation array storage could not be reserved",
                    ));
                }
                for index in 0..length {
                    let place = word_place(order, index, count)?;
                    elements.push(Value::Word(read_word(&limbs, bits, place)?));
                }
                Ok(Value::Array(Rc::new(ArrayValue { ty: *ty, elements })))
            }
            CoreType::Word8 | CoreType::Word16 | CoreType::Word32 | CoreType::Word64 => {
                let (bits, _) = to.words().ok_or(Stop::InconsistentCore)?;
                Ok(Value::Word(read_word(&limbs, bits, 0)?))
            }
            CoreType::Bool | CoreType::Tuple(_) => Err(Stop::InconsistentCore),
        }
    }
}

/// The place of the word at `index` of `count` words in the number they
/// spell: 0 for the least significant word.
fn word_place(order: ByteOrder, index: usize, count: u32) -> Result<u32, Stop> {
    let index = u32::try_from(index).map_err(|_| Stop::InconsistentCore)?;
    match order {
        ByteOrder::Little => Ok(index),
        ByteOrder::Big => count
            .checked_sub(1)
            .and_then(|last| last.checked_sub(index))
            .ok_or(Stop::InconsistentCore),
    }
}

/// Writes the `bits`-bit word at `place` into `limbs`, base-2^32 digits
/// least significant first, which hold zeros there. Each admitted width
/// divides 64, so a word lies within one digit or fills two.
fn write_word(limbs: &mut [u32], bits: u32, place: u32, word: u64) -> Result<(), Stop> {
    if bits < 64 && word.checked_shr(bits) != Some(0) {
        return Err(Stop::InconsistentCore);
    }
    let offset = bits.checked_mul(place).ok_or(Stop::InconsistentCore)?;
    let digit = usize::try_from(offset / 32).map_err(|_| Stop::InconsistentCore)?;
    let [b0, b1, b2, b3, b4, b5, b6, b7] = word.to_le_bytes();
    let low = u32::from_le_bytes([b0, b1, b2, b3]);
    let high = u32::from_le_bytes([b4, b5, b6, b7]);
    let target = limbs.get_mut(digit).ok_or(Stop::InconsistentCore)?;
    *target |= low.checked_shl(offset % 32).ok_or(Stop::InconsistentCore)?;
    if bits == 64 {
        let next = digit.checked_add(1).ok_or(Stop::InconsistentCore)?;
        *limbs.get_mut(next).ok_or(Stop::InconsistentCore)? = high;
    }
    Ok(())
}

/// Reads the `bits`-bit word at `place` of `limbs`, base-2^32 digits least
/// significant first.
fn read_word(limbs: &[u32], bits: u32, place: u32) -> Result<u64, Stop> {
    let offset = bits.checked_mul(place).ok_or(Stop::InconsistentCore)?;
    let digit = usize::try_from(offset / 32).map_err(|_| Stop::InconsistentCore)?;
    let low = u64::from(*limbs.get(digit).ok_or(Stop::InconsistentCore)?);
    if bits == 64 {
        let next = digit.checked_add(1).ok_or(Stop::InconsistentCore)?;
        let high = u64::from(*limbs.get(next).ok_or(Stop::InconsistentCore)?);
        return Ok((high << 32) | low);
    }
    let mask = 1_u64
        .checked_shl(bits)
        .and_then(|bound| bound.checked_sub(1))
        .ok_or(Stop::InconsistentCore)?;
    let word = low.checked_shr(offset % 32).ok_or(Stop::InconsistentCore)?;
    Ok(word & mask)
}

/// Writes the residue of `value` modulo 2^`width` into `limbs`, the
/// `width`/32 base-2^32 digits, rounded up, least significant first, which
/// hold zeros: a negative value's residue is its two's complement.
fn write_residue(limbs: &mut [u32], value: &ExactInteger, width: u32) -> Result<(), Stop> {
    for (digit, magnitude) in limbs.iter_mut().zip(value.magnitude_limbs()) {
        *digit = *magnitude;
    }
    if value.is_negative() {
        // 2^width - |value|, modulo 2^width: the complement plus one.
        let mut carry = true;
        for digit in limbs.iter_mut() {
            let (sum, overflow) = (!*digit).overflowing_add(u32::from(carry));
            *digit = sum;
            carry = overflow;
        }
    }
    let spare = width % 32;
    if spare != 0 {
        let top = limbs.last_mut().ok_or(Stop::InconsistentCore)?;
        *top &= 1_u32
            .checked_shl(spare)
            .and_then(|bound| bound.checked_sub(1))
            .ok_or(Stop::InconsistentCore)?;
    }
    Ok(())
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
        .entry_functions()
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
        inner_frames: 0,
    };
    let mut shared_module = None;
    for function in core
        .entry_functions()
        .iter()
        .filter(|function| function.parameters.is_empty())
    {
        let steps_before = machine.steps;
        let value = match machine.run(function, Vec::new()) {
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
        let mut sizes = Vec::new();
        if sizes.try_reserve_exact(function.sizes.len()).is_err() {
            return allocation_failure(
                diagnostics,
                function.name_span,
                "evaluated function sizes could not be reserved",
            );
        }
        sizes.extend_from_slice(&function.sizes);
        let value = match result_value(value, &function.result_type, reservations) {
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
            sizes,
            value,
        });
    }
    EvaluationResult {
        values: Some(values),
        diagnostics,
    }
}

/// Copies an evaluated value of type `ty` out of the machine's shared storage.
fn result_value(
    value: Value,
    ty: &CoreType,
    reservations: Reservations,
) -> Result<CoreValue, Stop> {
    match (value, ty) {
        (Value::Int(value), CoreType::Int) => value
            .try_clone_with_reservation(reservations.value_limbs)
            .map(CoreValue::Int)
            .ok_or(Stop::Allocation(
                "evaluated exact integer storage could not be reserved",
            )),
        (Value::Bool(value), CoreType::Bool) => Ok(CoreValue::Bool(value)),
        (Value::Word(value), ty) => {
            CoreValue::word_from_u64(ty, value).ok_or(Stop::InconsistentCore)
        }
        (Value::Mod(value), CoreType::Mod(modulus)) => {
            result_residue(&value, *modulus, reservations)
        }
        (Value::Array(array), CoreType::Array(array_type)) if array.ty == *array_type => {
            let mut elements = Vec::new();
            if !(reservations.result_array)(&mut elements, array.elements.len()) {
                return Err(Stop::Allocation(
                    "evaluated array storage could not be reserved",
                ));
            }
            for element in &array.elements {
                elements.push(result_element(
                    element,
                    &array_type.element(),
                    reservations,
                )?);
            }
            CoreArray::new(*array_type, elements)
                .map(CoreValue::Array)
                .ok_or(Stop::InconsistentCore)
        }
        (Value::Tuple(tuple), CoreType::Tuple(tuple_type)) if tuple.ty == *tuple_type => {
            let mut elements = Vec::new();
            if !(reservations.result_array)(&mut elements, tuple.elements.len()) {
                return Err(Stop::Allocation(
                    "evaluated tuple storage could not be reserved",
                ));
            }
            // A tuple's elements are scalars and arrays, so this recursion
            // is one level deep.
            for (element, element_type) in tuple.elements.iter().zip(tuple_type.elements()) {
                if matches!(element, Value::Tuple(_)) {
                    return Err(Stop::InconsistentCore);
                }
                elements.push(result_value(element.clone(), element_type, reservations)?);
            }
            CoreTuple::new(tuple_type.clone(), elements)
                .map(CoreValue::Tuple)
                .ok_or(Stop::InconsistentCore)
        }
        _ => Err(Stop::InconsistentCore),
    }
}

/// Copies one scalar array element; arrays have no array elements.
fn result_element(
    element: &Value,
    ty: &CoreType,
    reservations: Reservations,
) -> Result<CoreValue, Stop> {
    match (element, ty) {
        (Value::Int(value), CoreType::Int) => value
            .try_clone_with_reservation(reservations.value_limbs)
            .map(CoreValue::Int)
            .ok_or(Stop::Allocation(
                "evaluated exact integer storage could not be reserved",
            )),
        (Value::Bool(value), CoreType::Bool) => Ok(CoreValue::Bool(*value)),
        (Value::Word(value), ty) => {
            CoreValue::word_from_u64(ty, *value).ok_or(Stop::InconsistentCore)
        }
        (Value::Mod(value), CoreType::Mod(modulus)) => {
            result_residue(value, *modulus, reservations)
        }
        _ => Err(Stop::InconsistentCore),
    }
}

/// Copies one least residue of `Mod[modulus]`.
fn result_residue(
    value: &ExactInteger,
    modulus: Modulus,
    reservations: Reservations,
) -> Result<CoreValue, Stop> {
    let value = value
        .try_clone_with_reservation(reservations.value_limbs)
        .ok_or(Stop::Allocation(
            "evaluated exact integer storage could not be reserved",
        ))?;
    Residue::new(modulus, value)
        .map(CoreValue::Mod)
        .ok_or(Stop::InconsistentCore)
}

/// Shares every `Int`, residue, and array literal once so that evaluation
/// never copies literal digits or a byte string's bytes.
fn share_literals(core: &CoreModule) -> Option<SharedLiterals> {
    let mut shared = Vec::new();
    shared.try_reserve_exact(core.functions.len()).ok()?;
    for function in &core.functions {
        let mut parts = Vec::new();
        parts
            .try_reserve_exact(
                function
                    .locals
                    .len()
                    .checked_add(1)?
                    .checked_add(function.loops.len())?
                    .checked_add(function.conditionals.len().checked_mul(2)?)?,
            )
            .ok()?;
        // The parts in the order of `expression_part`.
        let expressions = function
            .locals
            .iter()
            .map(|local| &local.value)
            .chain(std::iter::once(&function.body))
            .chain(function.loops.iter().map(|r#loop| &r#loop.step))
            .chain(
                function
                    .conditionals
                    .iter()
                    .flat_map(|conditional| [&conditional.then_branch, &conditional.else_branch]),
            );
        for expression in expressions {
            let mut literals = Vec::new();
            literals.try_reserve_exact(expression.nodes.len()).ok()?;
            for node in &expression.nodes {
                literals.push(match &node.kind {
                    CoreNodeKind::Literal(CoreValue::Int(value)) => Some(SharedLiteral::Integer(
                        Rc::new(value.try_clone_with_reservation(reserve_value_limbs)?),
                    )),
                    CoreNodeKind::Literal(CoreValue::Mod(residue)) => {
                        Some(SharedLiteral::Integer(Rc::new(
                            residue
                                .value()
                                .try_clone_with_reservation(reserve_value_limbs)?,
                        )))
                    }
                    CoreNodeKind::Literal(CoreValue::Array(array)) => {
                        let mut elements = Vec::new();
                        elements.try_reserve_exact(array.elements().len()).ok()?;
                        for element in array.elements() {
                            elements.push(shared_element(element)?);
                        }
                        Some(SharedLiteral::Array(Rc::new(ArrayValue {
                            ty: array.ty(),
                            elements,
                        })))
                    }
                    _ => None,
                });
            }
            parts.push(literals);
        }
        shared.push(parts);
    }
    Some(shared)
}

/// Returns the evaluator's value of a scalar element of an array literal,
/// or `None` when storage cannot be reserved or the element is not a
/// scalar.
fn shared_element(element: &CoreValue) -> Option<Value> {
    Some(match element {
        CoreValue::Int(value) => Value::Int(Rc::new(
            value.try_clone_with_reservation(reserve_value_limbs)?,
        )),
        CoreValue::Mod(residue) => Value::Mod(Rc::new(
            residue
                .value()
                .try_clone_with_reservation(reserve_value_limbs)?,
        )),
        CoreValue::Bool(value) => Value::Bool(*value),
        CoreValue::Array(_) | CoreValue::Tuple(_) => return None,
        word => Value::Word(word.word_as_u64()?),
    })
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
            // An array of n elements costs its elements' steps and n more;
            // every index costs one step.
            ("  spec a() -> Word[8]^3 { [1, 2, 3] }\n", 6),
            ("  spec a() -> Int^1 { [-(1)] }\n", 4),
            (
                "  spec a() -> Word[8] { let t: Word[8]^2 = [1, 2]; t[1] ^ t[0] }\n",
                9,
            ),
            // A word index costs its steps and one more for its conversion
            // to `Int`.
            (
                "  spec a() -> Word[8] { let t: Word[8]^2 = [1, 2]; let x: Word[8] = 3; t[x & 1] }\n",
                11,
            ),
            // A loop costs one step and one more per iteration, plus its
            // initial value's and every step's; an index or accumulator
            // read costs one step, as does a selection.
            (
                "  spec f() -> Int { for i in 0..3 with s: Int = 0 { s } }\n",
                8,
            ),
            (
                "  spec f() -> Word[8] { for i in 5..6 with s: Word[8] = 1 { s } }\n",
                4,
            ),
            (
                "  spec f() -> Word[8] { let t: Word[8]^2 = [1, 2]; \
                 for i in 0..2 with s: Word[8] = 0 { s ^ t[i] } }\n",
                18,
            ),
            // A step's or branch's binding costs its value's steps each time
            // the step or branch is evaluated; each read of it costs one.
            (
                "  spec f() -> Int { for i in 0..3 with s: Int = 0 { let t: Int = s; t } }\n",
                11,
            ),
            // An update or fill of n elements costs ceil(n / 64) steps
            // beyond its operands'.
            ("  spec a() -> Word[8]^3 { [1, 2, 3] with [0] = 9 }\n", 9),
            ("  spec a() -> Int^1 { [5; 1] with [0] = 9 }\n", 5),
            ("  spec a() -> Word[8]^4 { [7; 4] }\n", 2),
            ("  spec a() -> Word[8]^64 { [7; 64] }\n", 2),
            ("  spec a() -> Word[8]^65 { [7; 65] }\n", 3),
            (
                "  spec a() -> Word[8]^256 { [7; 256] with [255] = 1 }\n",
                11,
            ),
            // `true`, `false`, `!`, `&&`, `||`, and every comparison of
            // words or `Bool` values cost one step; comparing integers
            // costs 1 + max(d1, d2).
            ("  spec b() -> Bool { !(true && false) || true }\n", 6),
            ("  spec b() -> Bool { let a: Word[8] = 1; a < 2 }\n", 4),
            ("  spec b() -> Bool { true != false }\n", 3),
            ("  spec b() -> Bool { let a: Int = 1; a < 4294967296 }\n", 6),
            // A conditional costs its condition's steps, one to choose, and
            // the chosen branch's steps; a later arm is the earlier arm's
            // `else` branch.
            ("  spec i() -> Int { if true { 1 } else { 2 * 3 } }\n", 3),
            (
                "  spec i() -> Int { if false { 1 } else if true { 2 * 3 } else { 4 } }\n",
                8,
            ),
            (
                "  spec i() -> Int { if true { let a: Int = 2; a * a } else { 1 } }\n",
                7,
            ),
            // A tuple of n elements costs its elements' steps and n more, as
            // an array does; `.k` costs one step, so a read of a name of a
            // tuple pattern costs two.
            ("  spec t() -> (Int, Word[8]) { (1, 2) }\n", 4),
            ("  spec t() -> Int { let p: (Int, Int) = (1, 2); p.1 }\n", 6),
            (
                "  spec t() -> Word[8] { let (a: Word[8], b: Word[8]) = (1, 2); a ^ b }\n",
                9,
            ),
            (
                "  spec t() -> (Int, Int) { for i in 0..2 with (a: Int, b: Int) = (0, 1) { (b, a) } }\n",
                19,
            ),
            // A byte string costs one step, as every literal does; a join,
            // a slice, or a slice update of an n-element result costs
            // ceil(n / 64) steps beyond its operands', and an omitted bound
            // costs the step of the literal it stands for.
            ("  spec a() -> Word[8]^3 { \"abc\" }\n", 1),
            (
                "  spec a() -> Word[8]^6 { \"abc\" ++ hex\"64 65 66\" }\n",
                3,
            ),
            ("  spec a() -> Word[8]^65 { [0; 64] ++ \"a\" }\n", 5),
            (
                "  spec a() -> Word[8]^2 { let t: Word[8]^4 = \"abcd\"; t[1..3] }\n",
                5,
            ),
            (
                "  spec a() -> Word[8]^2 { let t: Word[8]^4 = \"abcd\"; t[2..] }\n",
                5,
            ),
            (
                "  spec a() -> Word[8]^4 { \"abcd\" with [0..2] = \"xy\" }\n",
                5,
            ),
            (
                "  spec a() -> Word[8]^129 { let t: Word[8]^256 = [7; 256]; t[0..129] }\n",
                11,
            ),
            // A conversion in a byte order costs one step for each 64 bits
            // of its words, or part of 64, beyond its operand's, and into
            // `Mod[m]` 1 * d * dm more, as `as Mod[m]` does.
            (
                "  spec w() -> Word[64] { \"abcdefgh\" as big Word[64] }\n",
                2,
            ),
            (
                "  spec w() -> Word[8]^9 { (hex\"00\" as big Word[8]^1) ++ [1; 8] }\n",
                5,
            ),
            (
                "  spec w() -> Word[32] { let b: Word[8]^4 = [1, 2, 3, 4]; b as big Word[32] }\n",
                10,
            ),
            (
                "  spec w() -> Word[8]^65 { let f: Word[8]^65 = [7; 65]; f as little Word[8]^65 }\n",
                13,
            ),
            ("  spec m() -> Mod[7] { hex\"ff\" as big Mod[7] }\n", 3),
            (
                "  spec i() -> Int { let x: Word[8]^256 = [0xff; 256]; x as big Int }\n",
                38,
            ),
            // Integer division costs 1 + d1 * max(d2, 1); word division
            // costs one step.
            ("  spec i() -> Int { let a: Int = 4294967296; a / 3 }\n", 6),
            ("  spec i() -> Int { let a: Int = 4294967296; a % 0 }\n", 6),
            ("  spec i() -> Int { let a: Int = 0; a / 3 }\n", 4),
            ("  spec w() -> Word[8] { let a: Word[8] = 200; a / 7 }\n", 4),
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
        let nested_by = |prefix: &str, core: &str, suffix: &str, depth: usize| {
            format!("{}{core}{}", prefix.repeat(depth), suffix.repeat(depth))
        };
        let nested = |prefix: &str, core: &str, suffix: &str| {
            nested_by(prefix, core, suffix, MAX_EXPRESSION_NESTING)
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
                "{}x{}",
                "h([".repeat(MAX_EXPRESSION_NESTING / 2),
                "])[0]".repeat(MAX_EXPRESSION_NESTING / 2)
            ),
            format!(
                "h([{}x{}])[0]",
                "(".repeat(MAX_EXPRESSION_NESTING - 2),
                ")".repeat(MAX_EXPRESSION_NESTING - 2)
            ),
            format!(
                "{}x",
                (0..MAX_BINDINGS_PER_BODY)
                    .map(|index| format!("let v{index}: Word[32] = {};", nested("(", "x", ")")))
                    .collect::<String>()
            ),
            // Loops nested in steps and in initial values, updates nested in
            // updated values, and an index nested in groups.
            format!(
                "{}x{}",
                (0..MAX_EXPRESSION_NESTING)
                    .map(|index| format!("for i{index} in 0..1 with s{index}: Word[32] = x {{ "))
                    .collect::<String>(),
                " }".repeat(MAX_EXPRESSION_NESTING)
            ),
            format!(
                "{}x{}",
                (0..MAX_EXPRESSION_NESTING)
                    .map(|index| format!("for i{index} in 0..1 with s{index}: Word[32] = "))
                    .collect::<String>(),
                (0..MAX_EXPRESSION_NESTING)
                    .rev()
                    .map(|index| format!(" {{ s{index} }}"))
                    .collect::<String>()
            ),
            format!(
                "{}x{}",
                "h([x] with [0] = ".repeat(MAX_EXPRESSION_NESTING / 2),
                ")[0]".repeat(MAX_EXPRESSION_NESTING / 2)
            ),
            format!(
                "h([x])[{}0{}]",
                "(".repeat(MAX_EXPRESSION_NESTING - 1),
                ")".repeat(MAX_EXPRESSION_NESTING - 1)
            ),
            // Conditionals nested in values, in `else` values, and in
            // conditions, and one `else if` chain of 4096 arms, every one of
            // which is taken.
            nested("if x == x { ", "x", " } else { x }"),
            nested("if x != x { x } else { ", "x", " }"),
            format!(
                "if {}x == x{} {{ x }} else {{ x }}",
                "if ".repeat(MAX_EXPRESSION_NESTING - 1),
                " { true } else { false }".repeat(MAX_EXPRESSION_NESTING - 1)
            ),
            format!("{}{{ x }}", "if x == 0 { 0 } else ".repeat(4096)),
            // Residues converted in groups, a modulus nested in groups, a
            // tall modulus, and residue arithmetic nested in groups.
            format!(
                "{}x{} as Word[32]",
                "(".repeat(MAX_EXPRESSION_NESTING - 1),
                " as Mod[7])".repeat(MAX_EXPRESSION_NESTING - 1)
            ),
            format!(
                "(x as Mod[{}7{}]) as Word[32]",
                "(".repeat(MAX_EXPRESSION_NESTING - 2),
                ")".repeat(MAX_EXPRESSION_NESTING - 2)
            ),
            format!(
                "(x as Mod[1{}]) as Word[32]",
                " + 1".repeat(MAX_EXPRESSION_HEIGHT - 4)
            ),
            format!(
                "let y: Mod[3329] = x as Mod[3329]; ({}y{}) as Word[32]",
                "y - y * (".repeat(MAX_EXPRESSION_NESTING - 1),
                ")".repeat(MAX_EXPRESSION_NESTING - 1)
            ),
            // A binding in every nested step and branch, a step and a branch
            // of the most bindings each nested as deeply as a group can be,
            // and bindings whose values nest.
            format!(
                "{}x{}",
                (0..MAX_EXPRESSION_NESTING)
                    .map(|index| format!(
                        "for i{index} in 0..1 with s{index}: Word[32] = x {{ \
                         let t{index}: Word[32] = s{index}; "
                    ))
                    .collect::<String>(),
                " }".repeat(MAX_EXPRESSION_NESTING)
            ),
            format!(
                "{}x{}",
                (0..MAX_EXPRESSION_NESTING)
                    .map(|index| format!("if x == x {{ let t{index}: Word[32] = x; "))
                    .collect::<String>(),
                " } else { x }".repeat(MAX_EXPRESSION_NESTING)
            ),
            format!(
                "for i in 0..1 with s: Word[32] = x {{ {}s }}",
                (0..MAX_BINDINGS_PER_BODY)
                    .map(|index| format!(
                        "let v{index}: Word[32] = {};",
                        nested_by("(", "s", ")", MAX_EXPRESSION_NESTING - 1)
                    ))
                    .collect::<String>()
            ),
            format!(
                "if x == x {{ x }} else {{ {}x }}",
                (0..MAX_BINDINGS_PER_BODY)
                    .map(|index| format!(
                        "let v{index}: Word[32] = {};",
                        nested_by("(", "x", ")", MAX_EXPRESSION_NESTING - 1)
                    ))
                    .collect::<String>()
            ),
            format!(
                "{}x{}",
                (0..MAX_EXPRESSION_NESTING / 2)
                    .map(|index| format!("if x == x {{ let t{index}: Word[32] = "))
                    .collect::<String>(),
                (0..MAX_EXPRESSION_NESTING / 2)
                    .rev()
                    .map(|index| format!("; t{index} }} else {{ x }}"))
                    .collect::<String>()
            ),
            // A join of the most operands an expression's height admits,
            // bounds nested in groups, slice updates nested in updated
            // values, and slices of calls nested in calls.
            format!(
                "let y: Word[32]^{} = [x]{}; y[0]",
                MAX_EXPRESSION_HEIGHT - 1,
                " ++ [x]".repeat(MAX_EXPRESSION_HEIGHT - 2)
            ),
            format!(
                "let y: Word[32]^1 = h([x])[{}0{}..1]; y[0]",
                "(".repeat(MAX_EXPRESSION_NESTING - 2),
                ")".repeat(MAX_EXPRESSION_NESTING - 2)
            ),
            format!(
                "{}h([x]){}[0]",
                "h([x] with [..1] = ".repeat(MAX_EXPRESSION_NESTING / 2 - 1),
                ")".repeat(MAX_EXPRESSION_NESTING / 2 - 1)
            ),
            format!(
                "let y: Word[32]^1 = {}[x]{}; y[0]",
                "h(".repeat(MAX_EXPRESSION_NESTING - 1),
                ")[..1]".repeat(MAX_EXPRESSION_NESTING - 1)
            ),
            // Sized calls nested in sized calls, calls without sizes that
            // take the instance their arguments fit nested in each other,
            // and a fill's length nested in groups.
            nested("s[1](", "x", ")"),
            format!(
                "{}[x]{}[0]",
                "t(".repeat(MAX_EXPRESSION_NESTING - 1),
                ")".repeat(MAX_EXPRESSION_NESTING - 1)
            ),
            format!(
                "h([x; {}1{}])[0]",
                "(".repeat(MAX_EXPRESSION_NESTING - 2),
                ")".repeat(MAX_EXPRESSION_NESTING - 2)
            ),
            // Tuples of calls' elements nested in calls, and a tuple
            // pattern in every nested step of loops whose accumulators are
            // tuples.
            format!(
                "{}x{}",
                "p((".repeat(MAX_EXPRESSION_NESTING / 2),
                ", x)).0".repeat(MAX_EXPRESSION_NESTING / 2)
            ),
            format!(
                "let r: (Word[32], Word[32]) = {}(c{last}, d{last}){}; r.1",
                (0..MAX_EXPRESSION_NESTING - 1)
                    .map(|index| format!(
                        "for i{index} in 0..1 with (a{index}: Word[32], b{index}: Word[32]) = \
                         (x, x) {{ let (c{index}: Word[32], d{index}: Word[32]) = \
                         (b{index}, a{index}); "
                    ))
                    .collect::<String>(),
                " }".repeat(MAX_EXPRESSION_NESTING - 1),
                last = MAX_EXPRESSION_NESTING - 2
            ),
            // Words packed and unpacked in turn, nested in groups, and an
            // array literal of calls packed in calls.
            nested_by(
                "((",
                "x",
                " as little Word[8]^4) as big Word[32])",
                MAX_EXPRESSION_NESTING / 2,
            ),
            nested_by("g([", "x", "] as big Word[32])", MAX_EXPRESSION_NESTING / 2),
        ];
        let sources = bodies
            .iter()
            .map(|body| {
                format!(
                    "edition 2026; module m {{\n  spec g(x: Word[32]) -> Word[32] {{ x }}\n  \
                     spec h(x: Word[32]^1) -> Word[32]^1 {{ x }}\n  \
                     spec p(t: (Word[32], Word[32])) -> (Word[32], Word[32]) {{ t }}\n  \
                     spec s[n in 1..2](x: Word[32]) -> Word[32] {{ x }}\n  \
                     spec t[n in 1..3](x: Word[32]^n) -> Word[32]^n {{ x }}\n  \
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
    fn every_instance_of_a_sized_root_is_evaluated_and_named_by_its_sizes() {
        let (_, core) = analyzed(concat!(
            "edition 2026; module sizes {\n",
            "  spec zeros[n in 2..4]() -> Word[8]^n { [0; n] }\n",
            "  spec count[a in 1..3, b in 5..6]() -> Int { (a * 100) + b }\n",
            "  spec twice[k in 1..3](x: Word[8]^k) -> Word[8]^(2 * k) { x ++ x }\n",
            "  spec pair() -> Word[8]^4 { twice(hex\"ab cd\") }\n",
            "}\n"
        ));
        let result = evaluate(&core);
        assert_eq!(result.diagnostics(), []);
        let values = result.values().unwrap();
        assert_eq!(
            values.iter().map(ToString::to_string).collect::<Vec<_>>(),
            [
                "sizes::zeros[2]: Word[8]^2 = [0x00, 0x00]",
                "sizes::zeros[3]: Word[8]^3 = [0x00, 0x00, 0x00]",
                "sizes::count[1, 5]: Int = 105",
                "sizes::count[2, 5]: Int = 205",
                "sizes::pair: Word[8]^4 = [0xab, 0xcd, 0xab, 0xcd]",
            ]
        );
        assert_eq!(values[2].sizes(), [1, 5]);
        assert_eq!(values[4].sizes(), []);
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

    /// Renders `count` words of `bits` bits as Orange prints them: one word,
    /// or an array when `array` is set.
    fn render_words(bits: u32, words: &[u128], array: bool) -> String {
        let rendered = words
            .iter()
            .map(|word| render_word(bits, *word))
            .collect::<Vec<_>>();
        if array {
            format!("[{}]", rendered.join(", "))
        } else {
            rendered.concat()
        }
    }

    /// The type of `count` words of `bits` bits: one word, or an array.
    fn words_type(bits: u32, count: u32, array: bool) -> String {
        if array {
            format!("Word[{bits}]^{count}")
        } else {
            format!("Word[{bits}]")
        }
    }

    /// Splits `value`, of `bits * count` bits, into its words in `order`.
    fn split(value: u128, bits: u32, count: u32, big: bool) -> Vec<u128> {
        let mask = if bits == 128 {
            u128::MAX
        } else {
            (1 << bits) - 1
        };
        (0..count)
            .map(|index| {
                let place = if big { count - 1 - index } else { index };
                (value >> (bits * place)) & mask
            })
            .collect()
    }

    #[test]
    fn byte_orders_match_a_wide_reference_for_every_pair_of_widths() {
        let values = [
            0_u128,
            1,
            u128::MAX,
            0x0123_4567_89ab_cdef_fedc_ba98_7654_3210,
            0x8000_0000_0000_0000_0000_0000_0000_0001,
            0x00ff_00ff_00ff_00ff_00ff_00ff_00ff_00ff,
        ];
        let mut members = String::new();
        let mut expected = Vec::new();
        let mut count = 0;
        for (_, from_bits) in WORDS {
            for (_, to_bits) in WORDS {
                let lcm = from_bits.max(to_bits);
                for total in [lcm, 64.max(lcm), 128] {
                    let mask = if total == 128 {
                        u128::MAX
                    } else {
                        (1 << total) - 1
                    };
                    let (from_count, to_count) = (total / from_bits, total / to_bits);
                    for (value_index, value) in values.iter().enumerate() {
                        let value = value & mask;
                        for big in [true, false] {
                            // A single word is written both as a word and
                            // as an array of one, once each.
                            let from_array = from_count > 1 || value_index % 2 == 1;
                            let to_array = to_count > 1 || value_index % 3 == 1;
                            let words = split(value, from_bits, from_count, big);
                            let literal = if from_array {
                                format!(
                                    "[{}]",
                                    words
                                        .iter()
                                        .map(|word| format!("{word:#x}"))
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                )
                            } else {
                                format!("{:#x}", words[0])
                            };
                            let (from, to) = (
                                words_type(from_bits, from_count, from_array),
                                words_type(to_bits, to_count, to_array),
                            );
                            let order = if big { "big" } else { "little" };
                            let name = format!("p{count}");
                            count += 1;
                            members.push_str(&format!(
                                "  spec {name}() -> {to} {{ let v: {from} = {literal}; \
                                 v as {order} {to} }}\n"
                            ));
                            let result = split(value, to_bits, to_count, big);
                            expected.push(format!(
                                "{name} = {}",
                                render_words(to_bits, &result, to_array)
                            ));
                            // The number the words spell.
                            let name = format!("p{count}");
                            count += 1;
                            members.push_str(&format!(
                                "  spec {name}() -> Int {{ let v: {from} = {literal}; \
                                 v as {order} Int }}\n"
                            ));
                            expected.push(format!("{name} = {value}"));
                        }
                    }
                }
            }
        }
        assert_eq!(values_of(&members), expected);
    }

    #[test]
    fn byte_orders_write_numbers_as_their_residues_and_read_residues() {
        assert_eq!(
            values_of(concat!(
                "  spec minus_one() -> Word[8]^3 { let n: Int = -1; n as big Word[8]^3 }\n",
                "  spec minus_two() -> Word[16]^2 { let n: Int = -2; n as little Word[16]^2 }\n",
                "  spec wraps() -> Word[8]^2 { let n: Int = 65539; n as big Word[8]^2 }\n",
                "  spec negative_wraps() -> Word[32] { let n: Int = -4294967297; n as big Word[32] }\n",
                "  spec wide() -> Word[64]^4 {\n",
                "    let n: Int = 1606938044258990275541962092341162602522202993782792835301376;\n",
                "    n as little Word[64]^4\n",
                "  }\n",
                "  spec all_ones() -> Word[64]^4 { let n: Int = -1; n as big Word[64]^4 }\n",
                "  spec residue() -> Word[8]^2 { let m: Mod[65537] = -1; m as big Word[8]^2 }\n",
                "  spec small_residue() -> Word[8]^2 { let m: Mod[7] = 6; m as big Word[8]^2 }\n",
                "  spec reduced() -> Mod[7] { hex\"ff\" as big Mod[7] }\n",
                "  spec prime() -> Mod[(1 << 130) - 5] {\n",
                "    let b: Word[8]^17 = [0xff; 17];\n",
                "    b as little Mod[(1 << 130) - 5]\n",
                "  }\n",
                "  spec round_trip() -> Word[8]^4 {\n",
                "    let x: Word[32] = 0xdeadbeef;\n",
                "    ((x as little Word[8]^4) as big Word[32]) as big Word[8]^4\n",
                "  }\n",
            )),
            [
                "minus_one = [0xff, 0xff, 0xff]",
                "minus_two = [0xfffe, 0xffff]",
                "wraps = [0x00, 0x03]",
                "negative_wraps = 0xffffffff",
                "wide = [0x0000000000000000, 0x0000000000000000, 0x0000000000000000, \
                 0x0000000000000100]",
                "all_ones = [0xffffffffffffffff, 0xffffffffffffffff, 0xffffffffffffffff, \
                 0xffffffffffffffff]",
                "residue = [0x00, 0x00]",
                "small_residue = [0x00, 0x06]",
                "reduced = 3",
                // 2^136 - 1 modulo 2^130 - 5 is 2^136 - 1 - 64 * (2^130 - 5).
                "prime = 319",
                "round_trip = [0xef, 0xbe, 0xad, 0xde]",
            ]
        );
    }

    #[test]
    fn byte_orders_reach_the_widest_array_and_integer() {
        // 256 words of 64 bits spell a number of 16,384 bits, the widest an
        // `Int` holds, and unpack back to themselves.
        let values = values_of(concat!(
            "  spec widest() -> Word[64]^256 {\n",
            "    let x: Word[64]^256 = [0xffffffffffffffff; 256];\n",
            "    (x as big Int) as little Word[64]^256\n",
            "  }\n",
            "  spec ones() -> Word[64]^32 {\n",
            "    let x: Word[8]^256 = [0xff; 256];\n",
            "    let n: Int = x as big Int;\n",
            "    n as little Word[64]^32\n",
            "  }\n",
            // 2^2048 is 0 modulo 2^2048.
            "  spec wraps() -> Word[64]^32 {\n",
            "    let x: Word[8]^256 = [0xff; 256];\n",
            "    let n: Int = x as big Int;\n",
            "    (n + 1) as little Word[64]^32\n",
            "  }\n",
        ));
        let ones = |count: usize| vec!["0xffffffffffffffff"; count].join(", ");
        let zeros = vec!["0x0000000000000000"; 32].join(", ");
        assert_eq!(
            values,
            [
                format!("widest = [{}]", ones(256)),
                format!("ones = [{}]", ones(32)),
                format!("wraps = [{zeros}]"),
            ]
        );
    }

    #[test]
    fn byte_order_limb_reservation_failure_returns_no_values() {
        let packed = core(concat!(
            "edition 2026; module values {\n",
            "  spec packed() -> Word[32] { \"abcd\" as big Word[32] }\n",
            "}\n",
        ));
        let result = evaluate_with_reservations(
            &packed,
            MAX_EVALUATION_STEPS_PER_SOURCE,
            |values, capacity| values.try_reserve_exact(capacity).is_ok(),
            Reservations {
                value_limbs: |_, _| false,
                ..Reservations::DEFAULT
            },
        );
        assert!(result.values().is_none());
        assert_eq!(result.diagnostics().len(), 1);
        assert_eq!(
            result.diagnostics()[0].code(),
            DiagnosticCode::EvaluationResourceLimit
        );
        assert_eq!(
            result.diagnostics()[0].label(),
            "exact integer storage could not be reserved"
        );
        let array_failure = evaluate_with_reservations(
            &core(concat!(
                "edition 2026; module values {\n",
                "  spec unpacked() -> Word[8]^4 { let x: Word[32] = 1; x as big Word[8]^4 }\n",
                "}\n",
            )),
            MAX_EVALUATION_STEPS_PER_SOURCE,
            |values, capacity| values.try_reserve_exact(capacity).is_ok(),
            Reservations {
                array: |_, _| false,
                ..Reservations::DEFAULT
            },
        );
        assert!(array_failure.values().is_none());
        assert_eq!(
            array_failure.diagnostics()[0].label(),
            "evaluation array storage could not be reserved"
        );
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

    #[test]
    fn arrays_evaluate_in_index_order_and_display_every_element() {
        let members = concat!(
            "  spec rows() -> Word[8]^4 { [1, 0x20, 0xff, 7] }\n",
            "  spec pick() -> Word[8] { rows()[2] ^ rows()[0] }\n",
            "  spec ints() -> Int^2 { let p: Int^2 = [-5, 18446744073709551616]; [p[1], p[0]] }\n",
            "  spec rev(x: Word[32]^3) -> Word[32]^3 { [x[2], x[1], x[0]] }\n",
            "  spec run() -> Word[32]^3 { rev([1, 2, rev([7, 8, 9])[0]]) }\n",
            "  spec one() -> Word[64]^1 { [0xffffffffffffffff] }\n",
        );
        assert_eq!(
            values_of(members),
            [
                "rows = [0x01, 0x20, 0xff, 0x07]",
                "pick = 0xfe",
                "ints = [18446744073709551616, -5]",
                "run = [0x00000009, 0x00000002, 0x00000001]",
                "one = [0xffffffffffffffff]",
            ]
        );
        // Every admitted length evaluates, and elements keep their order.
        let long = (0..256)
            .map(|index| format!("{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let members = format!(
            "  spec all() -> Word[8]^256 {{ [{long}] }}\n  \
             spec last() -> Word[8] {{ all()[255] }}\n"
        );
        let values = values_of(&members);
        assert_eq!(values[1], "last = 0xff");
        let expected = (0..256)
            .map(|index| render_word(8, index))
            .collect::<Vec<_>>()
            .join(", ");
        assert_eq!(values[0], format!("all = [{expected}]"));
    }

    #[test]
    fn chacha20_quarter_round_on_an_array_matches_rfc_8439() {
        // RFC 8439 section 2.1.1, with the state as one `Word[32]^4`.
        let members = concat!(
            "  spec quarter_round(s: Word[32]^4) -> Word[32]^4 {\n",
            "    let a1: Word[32] = s[0] + s[1];\n",
            "    let d1: Word[32] = (s[3] ^ a1) <<< 16;\n",
            "    let c1: Word[32] = s[2] + d1;\n",
            "    let b1: Word[32] = (s[1] ^ c1) <<< 12;\n",
            "    let a2: Word[32] = a1 + b1;\n",
            "    let d2: Word[32] = (d1 ^ a2) <<< 8;\n",
            "    let c2: Word[32] = c1 + d2;\n",
            "    let b2: Word[32] = (b1 ^ c2) <<< 7;\n",
            "    [a2, b2, c2, d2]\n",
            "  }\n",
            "  spec test_vector() -> Word[32]^4 {\n",
            "    quarter_round([0x11111111, 0x01020304, 0x9b8d6f43, 0x01234567])\n",
            "  }\n",
        );
        assert_eq!(
            values_of(members),
            ["test_vector = [0xea2a92f4, 0xcb1cf8ce, 0x4581472e, 0x5881c4bb]"]
        );
    }

    #[test]
    fn inconsistent_arrays_and_indices_fail_closed() {
        let base = core(concat!(
            "edition 2026; module m {\n",
            "  spec rows() -> Word[8]^2 { [1, 2] }\n",
            "  spec pick() -> Word[8] { rows()[1] }\n",
            "  spec wrap(x: Word[8]) -> Word[8]^1 { [x] }\n",
            "  spec run() -> Word[8]^1 { wrap(3) }\n",
            "}\n"
        ));
        fn words(length: u32) -> CoreType {
            CoreType::Array(ArrayType::new(&CoreType::Word8, length).unwrap())
        }
        let mutations: [fn(&mut CoreModule); 9] = [
            // An array claims more elements than its type has.
            |core| core.functions[0].body.nodes[2].kind = CoreNodeKind::Array { elements: 3 },
            // An array claims fewer elements than were computed.
            |core| core.functions[0].body.nodes[2].kind = CoreNodeKind::Array { elements: 1 },
            // An array node has a scalar type.
            |core| core.functions[0].body.nodes[2].ty = CoreType::Word8,
            // An element does not fit the element type.
            |core| {
                core.functions[0].body.nodes[0].kind =
                    CoreNodeKind::Literal(CoreValue::Word16(0x100));
            },
            // An index is past the end.
            |core| core.functions[1].body.nodes[1].kind = CoreNodeKind::Index { index: 2 },
            // An index has a type other than the element type.
            |core| core.functions[1].body.nodes[1].ty = CoreType::Word16,
            // An index is applied to a scalar.
            |core| {
                core.functions[1].body.nodes[0].kind = CoreNodeKind::Literal(CoreValue::Word8(1));
            },
            // An array takes its element from the caller's arguments.
            |core| {
                core.functions[2].body.nodes.remove(0);
            },
            // A result has a different array type than its value.
            |core| core.functions[0].result_type = words(1),
        ];
        assert_eq!(base.functions[0].result_type, words(2));
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
    fn array_storage_reservation_failures_return_no_values() {
        let core = core(concat!(
            "edition 2026; module m {\n",
            "  spec rows() -> Int^2 { [1, 2] }\n",
            "}\n"
        ));
        for (reservations, label) in [
            (
                Reservations {
                    array: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluation array storage could not be reserved",
            ),
            (
                Reservations {
                    result_array: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluated array storage could not be reserved",
            ),
            (
                Reservations {
                    value_limbs: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluated exact integer storage could not be reserved",
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
            assert_eq!(diagnostic.primary_span(), core.functions[0].name_span);
        }
    }

    #[test]
    fn loops_updates_and_fills_evaluate_in_index_order() {
        let members = concat!(
            "  spec sum() -> Int { for i in 0..10 with s: Int = 0 { s + i } }\n",
            "  spec product() -> Int { for i in 3..5 with p: Int = 1 { p * i } }\n",
            "  spec grid() -> Int {\n",
            "    for i in 0..3 with s: Int = 0 { for j in 0..4 with t: Int = s { t + i * j } }\n",
            "  }\n",
            "  spec reversed(x: Word[8]^4) -> Word[8]^4 {\n",
            "    for i in 0..4 with r: Word[8]^4 = x { r with [i] = x[3 - i] }\n",
            "  }\n",
            "  spec reverse() -> Word[8]^4 { reversed([1, 2, 3, 4]) }\n",
            "  spec filled() -> Word[16]^3 { [0xbeef; 3] }\n",
            "  spec updated() -> Int^3 { ([1, 2, 3] with [0] = -1) with [2] = 5 }\n",
            "  spec inner(x: Int) -> Int { for j in 0..3 with t: Int = x { t + j } }\n",
            "  spec outer() -> Int { for i in 0..2 with s: Int = 0 { inner(s) + i } }\n",
            "  spec bound() -> Int { let k: Int = 5; for i in 0..3 with s: Int = k { s + k * i } }\n",
            "  spec bound_twice() -> Int { let a: Int = for i in 0..3 with s: Int = 0 { s + i }; a * 2 }\n",
            "  spec longest() -> Word[32] { for i in 0..65536 with s: Word[32] = 0 { s + 1 } }\n",
            "  spec last() -> Int { for i in 65535..65536 with s: Int = 0 { i } }\n",
        );
        assert_eq!(
            values_of(members),
            [
                "sum = 45",
                "product = 12",
                "grid = 18",
                "reverse = [0x04, 0x03, 0x02, 0x01]",
                "filled = [0xbeef, 0xbeef, 0xbeef]",
                "updated = [-1, 2, 5]",
                "outer = 7",
                "bound = 20",
                "bound_twice = 6",
                "longest = 0x00010000",
                "last = 65535",
            ]
        );
    }

    #[test]
    fn word_indices_select_and_update_by_value() {
        let members = concat!(
            "  spec squares() -> Word[8]^16 {\n",
            "    for i in 0..16 with t: Word[8]^16 = [0; 16] { t with [i] = (i as Word[8]) * (i as Word[8]) }\n",
            "  }\n",
            "  spec pick(x: Word[8]) -> Word[8] { squares()[x & 15] }\n",
            "  spec low() -> Word[8] { pick(0xf7) }\n",
            "  spec high() -> Word[8] { pick(0x0f) }\n",
            "  spec marked() -> Word[8]^4 { let k: Word[16] = 0x2a; [9; 4] with [k % 4] = 1 }\n",
            "  spec swap(s: Word[8]^4, i: Word[8], j: Word[8]) -> Word[8]^4 {\n",
            "    (s with [i & 3] = s[j & 3]) with [j & 3] = s[i & 3]\n",
            "  }\n",
            "  spec swapped() -> Word[8]^4 { swap([1, 2, 3, 4], 4, 7) }\n",
            "  spec wide() -> Int { let z: Word[64] = 0xff00000000000000; let t: Int^4 = [10, 20, 30, 40]; t[(z >> 62) as Int] }\n",
        );
        assert_eq!(
            values_of(members),
            [
                "squares = [0x00, 0x01, 0x04, 0x09, 0x10, 0x19, 0x24, 0x31, 0x40, 0x51, 0x64, 0x79, 0x90, 0xa9, 0xc4, 0xe1]",
                "low = 0x31",
                "high = 0xe1",
                "marked = [0x09, 0x09, 0x01, 0x09]",
                "swapped = [0x04, 0x02, 0x03, 0x01]",
                "wide = 40",
            ]
        );
    }

    #[test]
    fn loop_frames_do_not_count_toward_the_call_depth() {
        let mut members = String::new();
        for index in 0..MAX_CALL_DEPTH - 1 {
            members.push_str(&format!(
                "  spec f{index}() -> Int {{ for i in 0..1 with s: Int = 0 {{ f{}() }} }}\n",
                index + 1
            ));
        }
        members.push_str(&format!(
            "  spec f{}() -> Int {{ 7 }}\n",
            MAX_CALL_DEPTH - 1
        ));
        let values = values_of(&members);
        assert_eq!(values.len(), MAX_CALL_DEPTH);
        assert!(values.iter().all(|value| value.ends_with(" = 7")));
    }

    #[test]
    fn inconsistent_loops_updates_and_fills_fail_closed() {
        let base = core(concat!(
            "edition 2026; module m {\n",
            "  spec sum() -> Int { for i in 0..3 with s: Int = 0 { s + i } }\n",
            "  spec pick() -> Word[8] {\n",
            "    let t: Word[8]^2 = [1, 2];\n",
            "    for i in 0..2 with s: Word[8] = 0 { s ^ t[i] }\n",
            "  }\n",
            "  spec up() -> Word[8]^2 { [1, 2] with [1] = 3 }\n",
            "  spec fill() -> Word[8]^2 { [4; 2] }\n",
            "}\n"
        ));
        assert_eq!(base.functions[0].loops[0].step.nodes.len(), 3);
        assert_eq!(base.functions[1].loops[0].step.nodes.len(), 5);
        let mutations: [fn(&mut CoreModule); 16] = [
            // A loop has no iterations.
            |core| core.functions[0].loops[0].start = 3,
            // A loop's bounds are reversed.
            |core| {
                core.functions[0].loops[0].start = 2;
                core.functions[0].loops[0].end = 1;
            },
            // A loop has a different type than its node.
            |core| core.functions[0].loops[0].ty = CoreType::Word8,
            // A loop node names a loop the function lacks.
            |core| core.functions[0].body.nodes[1].kind = CoreNodeKind::Fold(1),
            // A loop claims an enclosing loop that is not active.
            |core| core.functions[0].loops[0].scope = vec![0, 0],
            // A loop's scope does not end with the loop itself.
            |core| core.functions[0].loops[0].scope = vec![1],
            // An index is read outside its loop.
            |core| core.functions[0].body.nodes[0].kind = CoreNodeKind::LoopIndex(0),
            // An accumulator is read outside its loop.
            |core| core.functions[0].body.nodes[0].kind = CoreNodeKind::Accumulator(0),
            // An accumulator read claims a different type.
            |core| core.functions[0].loops[0].step.nodes[0].ty = CoreType::Word8,
            // A step leaves no value behind.
            |core| core.functions[0].loops[0].step.nodes.clear(),
            // A step reads a binding that is not in scope.
            |core| core.functions[1].loops[0].visible_locals = 0,
            // A loop runs past the end of the array it selects from.
            |core| core.functions[1].loops[0].end = 3,
            // A selection claims a different element type.
            |core| core.functions[1].loops[0].step.nodes[3].ty = CoreType::Word16,
            // An update stores a value that does not fit its element type.
            |core| {
                core.functions[2].body.nodes[4].kind =
                    CoreNodeKind::Literal(CoreValue::Word16(0x103));
            },
            // An update has a scalar type.
            |core| core.functions[2].body.nodes[5].ty = CoreType::Word8,
            // A fill repeats a value that does not fit its element type.
            |core| {
                core.functions[3].body.nodes[0].kind =
                    CoreNodeKind::Literal(CoreValue::Word16(0x104));
            },
        ];
        assert_eq!(base.functions[2].body.nodes[5].kind, CoreNodeKind::Update);
        assert_eq!(base.functions[3].body.nodes[1].kind, CoreNodeKind::Fill);
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
    fn int_division_and_comparisons_match_an_i128_reference() {
        let corpus: [i128; 12] = [
            0,
            1,
            2,
            7,
            -7,
            -1,
            4_294_967_295,
            4_294_967_296,
            -18_446_744_073_709_551_617,
            170_141_183_460_469_231_731_687_303_715_884_105_727,
            -99_999_999_999_999_999_999,
            12_345_678_901_234_567_890,
        ];
        let mut members = String::new();
        let mut expected = Vec::new();
        for (i, &left) in corpus.iter().enumerate() {
            for (j, &right) in corpus.iter().enumerate() {
                let (quotient, remainder) = if right == 0 {
                    // The total rules: x / 0 = 0 and x % 0 = x.
                    (0, left)
                } else {
                    (left.div_euclid(right), left.rem_euclid(right))
                };
                members.push_str(&format!(
                    "  spec q{i}_{j}() -> Int^2 {{ let a: Int = {left}; let b: Int = {right}; \
                     [a / b, a % b] }}\n"
                ));
                expected.push(format!("q{i}_{j} = [{quotient}, {remainder}]"));
                members.push_str(&format!(
                    "  spec c{i}_{j}() -> Bool^6 {{ let a: Int = {left}; let b: Int = {right}; \
                     [a == b, a != b, a < b, a <= b, a > b, a >= b] }}\n"
                ));
                expected.push(format!(
                    "c{i}_{j} = [{}, {}, {}, {}, {}, {}]",
                    left == right,
                    left != right,
                    left < right,
                    left <= right,
                    left > right,
                    left >= right
                ));
            }
        }
        assert_eq!(values_of(&members), expected);
    }

    #[test]
    fn word_division_and_comparisons_are_unsigned_at_every_width() {
        for (ty, bits) in WORDS {
            let corpus = word_corpus(bits);
            let mut members = String::new();
            let mut expected = Vec::new();
            for (index, (&left, &right)) in corpus.iter().zip(corpus.iter().rev()).enumerate() {
                for (label, divisor) in [("n", right), ("z", 0)] {
                    // Orange's total rule: x / 0 = 0 and x % 0 = x.
                    let quotient = left.checked_div(divisor).unwrap_or(0);
                    let remainder = left.checked_rem(divisor).unwrap_or(left);
                    members.push_str(&format!(
                        "  spec d{label}{index}() -> {ty}^2 {{ let a: {ty} = {left}; \
                         [a / {divisor}, a % {divisor}] }}\n"
                    ));
                    expected.push(format!(
                        "d{label}{index} = [{}, {}]",
                        render_word(bits, quotient),
                        render_word(bits, remainder)
                    ));
                }
                members.push_str(&format!(
                    "  spec c{index}() -> Bool^6 {{ let a: {ty} = {left}; let b: {ty} = {right}; \
                     [a == b, a != b, a < b, a <= b, a > b, a >= b] }}\n"
                ));
                expected.push(format!(
                    "c{index} = [{}, {}, {}, {}, {}, {}]",
                    left == right,
                    left != right,
                    left < right,
                    left <= right,
                    left > right,
                    left >= right
                ));
            }
            assert_eq!(values_of(&members), expected, "{ty}");
        }
    }

    #[test]
    fn conditions_choose_exactly_one_branch_in_source_order() {
        let members = concat!(
            "  spec truth() -> Bool^2 { [true, false] }\n",
            "  spec logic() -> Bool^6 {\n",
            "    [true && false, true || false, !true, !(false || false), true == false, true != false]\n",
            "  }\n",
            "  spec sign(x: Int) -> Int { if x < 0 { -1 } else if x == 0 { 0 } else { 1 } }\n",
            "  spec signs() -> Int^3 { [sign(-5), sign(0), sign(12)] }\n",
            "  spec max(a: Word[8], b: Word[8]) -> Word[8] { if a < b { b } else { a } }\n",
            "  spec larger() -> Word[8] { max(0x7f, 0x80) }\n",
            "  spec clamp(x: Int) -> Int { let lo: Int = 0; if x < lo { lo } else if x > 255 { 255 } else { x } }\n",
            "  spec clamps() -> Int^3 { [clamp(-3), clamp(300), clamp(42)] }\n",
            "  spec evens() -> Int { for i in 0..10 with s: Int = 0 { if (i % 2) == 0 { s + i } else { s } } }\n",
            "  spec ring(x: Word[8]^5) -> Word[8]^5 { for i in 0..5 with r: Word[8]^5 = x { r with [i] = x[(i + 1) % 5] } }\n",
            "  spec rotated() -> Word[8]^5 { ring([1, 2, 3, 4, 5]) }\n",
            "  spec nested(a: Bool, b: Bool) -> Int { if a { if b { 3 } else { 2 } } else { if b { 1 } else { 0 } } }\n",
            "  spec table() -> Int^4 { [nested(false, false), nested(false, true), nested(true, false), nested(true, true)] }\n",
            "  spec chosen() -> Int { let one: Int = 1; let t: Bool = one < 2; if t { 10 } else { 20 } }\n",
        );
        assert_eq!(
            values_of(members),
            [
                "truth = [true, false]",
                "logic = [false, true, false, true, false, true]",
                "signs = [-1, 0, 1]",
                "larger = 0x80",
                "clamps = [0, 255, 42]",
                "evens = 20",
                "rotated = [0x02, 0x03, 0x04, 0x05, 0x01]",
                "table = [0, 1, 2, 3]",
                "chosen = 10",
            ]
        );
    }

    #[test]
    fn only_the_chosen_branch_is_evaluated() {
        // The untaken branch would exceed the whole step budget.
        let members = concat!(
            "  spec heavy(x: Int) -> Int {\n",
            "    for i in 0..65536 with s: Int = x { for j in 0..65536 with t: Int = s { t } }\n",
            "  }\n",
            "  spec lazy() -> Int { if true { 1 } else { heavy(0) } }\n",
            "  spec lazier() -> Int { let one: Int = 1; if false { heavy(0) } else if one == 1 { 2 } else { heavy(one) } }\n",
            "  spec guarded(n: Int) -> Int { if n == 0 { 0 } else { 1000 / n } }\n",
            "  spec guards() -> Int^2 { [guarded(0), guarded(8)] }\n",
        );
        assert_eq!(
            values_of(members),
            ["lazy = 1", "lazier = 2", "guards = [0, 125]"]
        );
        // Both operands of `&&` and `||` are evaluated.
        let (diagnostic, _, _) = single_failure(
            concat!(
                "  spec heavy(x: Int) -> Bool {\n",
                "    for i in 0..65536 with s: Bool = true { for j in 0..65536 with t: Bool = s { t } }\n",
                "  }\n",
                "  spec strict() -> Bool { false && heavy(0) }\n",
            ),
            MAX_EVALUATION_STEPS_PER_SOURCE,
        );
        assert_eq!(
            diagnostic.message(),
            "reference evaluation step limit exceeded"
        );
    }

    #[test]
    fn branch_frames_do_not_count_toward_the_call_depth() {
        let mut members = String::new();
        for index in 0..MAX_CALL_DEPTH - 1 {
            members.push_str(&format!(
                "  spec f{index}() -> Int {{ if true {{ if false {{ 0 }} else {{ f{}() }} }} else {{ 1 }} }}\n",
                index + 1
            ));
        }
        members.push_str(&format!(
            "  spec f{}() -> Int {{ 7 }}\n",
            MAX_CALL_DEPTH - 1
        ));
        let values = values_of(&members);
        assert_eq!(values.len(), MAX_CALL_DEPTH);
        assert!(values.iter().all(|value| value.ends_with(" = 7")));
    }

    #[test]
    fn inconsistent_conditions_and_comparisons_fail_closed() {
        let base = core(concat!(
            "edition 2026; module m {\n",
            "  spec pick() -> Int { if true { 1 } else { 2 } }\n",
            "  spec less() -> Bool { let a: Word[8] = 1; a < 2 }\n",
            "  spec same() -> Bool { true == false }\n",
            "  spec inner() -> Int {\n",
            "    let k: Int = 3;\n",
            "    for i in 0..2 with s: Int = 0 { if i == 0 { k } else { s } }\n",
            "  }\n",
            "  spec not() -> Bool { !false }\n",
            "  spec and() -> Bool { true && false }\n",
            "}\n"
        ));
        assert_eq!(
            base.functions[0].body.nodes[1].kind,
            CoreNodeKind::Choose(0)
        );
        assert!(matches!(
            base.functions[1].body.nodes[2].kind,
            CoreNodeKind::Compare { .. }
        ));
        assert!(matches!(
            base.functions[2].body.nodes[2].kind,
            CoreNodeKind::Compare { .. }
        ));
        assert_eq!(base.functions[3].loops[0].step.nodes.len(), 4);
        assert_eq!(
            base.functions[3].loops[0].step.nodes[3].kind,
            CoreNodeKind::Choose(0)
        );
        let mutations: [fn(&mut CoreModule); 14] = [
            // A choice names a conditional the function lacks.
            |core| core.functions[0].body.nodes[1].kind = CoreNodeKind::Choose(1),
            // A conditional has a different type than its node.
            |core| core.functions[0].conditionals[0].ty = CoreType::Word8,
            // A conditional claims an enclosing loop that is not active.
            |core| core.functions[0].conditionals[0].scope = vec![0],
            // A branch leaves no value behind.
            |core| core.functions[0].conditionals[0].then_branch.nodes.clear(),
            // A branch leaves a value of another type.
            |core| {
                core.functions[0].conditionals[0].then_branch.nodes[0].kind =
                    CoreNodeKind::Literal(CoreValue::Word8(1));
            },
            // A condition is not a `Bool`.
            |core| {
                core.functions[0].body.nodes[0].kind = CoreNodeKind::Literal(CoreValue::Word8(1));
            },
            // A choice has nothing to choose on.
            |core| {
                core.functions[0].body.nodes.remove(0);
            },
            // A comparison claims a type other than `Bool`.
            |core| core.functions[1].body.nodes[2].ty = CoreType::Word8,
            // A comparison claims operands of a narrower type than they have.
            |core| {
                core.functions[1].body.nodes[1].kind =
                    CoreNodeKind::Literal(CoreValue::Word16(0x100));
            },
            // `Bool` values are ordered.
            |core| {
                core.functions[2].body.nodes[2].kind = CoreNodeKind::Compare {
                    operator: BinaryOperator::Less,
                    operand: CoreType::Bool,
                };
            },
            // A branch reads a binding that is not in scope.
            |core| core.functions[3].conditionals[0].visible_locals = 0,
            // A branch in a loop claims no enclosing loop.
            |core| core.functions[3].conditionals[0].scope.clear(),
            // `!` is applied to an integer.
            |core| {
                core.functions[4].body.nodes[0].kind = CoreNodeKind::Literal(CoreValue::Int(
                    ExactInteger::from_u64(1, |limbs, count| {
                        limbs.try_reserve_exact(count).is_ok()
                    })
                    .unwrap(),
                ));
            },
            // `&&` is replaced by an arithmetic operator.
            |core| core.functions[5].body.nodes[2].kind = CoreNodeKind::Binary(BinaryOperator::Add),
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
    fn branch_and_division_reservation_failures_return_no_values() {
        for (members, reservations, label) in [
            // The branch's frame is the only frame after the root's.
            (
                "  spec f() -> Word[8] { if true { 1 } else { 2 } }\n",
                Reservations {
                    frames: |frames, count| {
                        frames.is_empty() && frames.try_reserve_exact(count).is_ok()
                    },
                    ..Reservations::DEFAULT
                },
                "evaluation call stack could not be reserved",
            ),
            (
                "  spec f() -> Int { let a: Int = 4294967296; a / 3 }\n",
                Reservations {
                    value_limbs: |limbs, _| limbs.capacity() > 0,
                    ..Reservations::DEFAULT
                },
                "exact integer storage could not be reserved",
            ),
        ] {
            let core = core(&format!("edition 2026; module m {{\n{members}}}\n"));
            let run = || {
                evaluate_with_reservations(
                    &core,
                    MAX_EVALUATION_STEPS_PER_SOURCE,
                    |values, capacity| values.try_reserve_exact(capacity).is_ok(),
                    reservations,
                )
            };
            let first = run();
            assert_eq!(first, run(), "{members}");
            assert!(first.values().is_none(), "{members}");
            let [diagnostic] = first.diagnostics() else {
                panic!("an allocation failure must produce exactly one diagnostic");
            };
            assert_eq!(
                diagnostic.message(),
                "reference evaluation result allocation failed"
            );
            assert_eq!(diagnostic.label(), label, "{members}");
        }
    }

    thread_local! {
        static ARRAY_RESERVATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }

    /// Refuses the second array reservation of the current evaluation.
    fn refuse_second_array(elements: &mut Vec<Value>, count: usize) -> bool {
        let calls = ARRAY_RESERVATIONS.with(|calls| {
            calls.set(calls.get() + 1);
            calls.get()
        });
        calls != 2 && elements.try_reserve_exact(count).is_ok()
    }

    #[test]
    fn loop_update_and_fill_reservation_failures_return_no_values() {
        for (members, reservations, label) in [
            // The loop's frame is the only frame after the root's.
            (
                "  spec f() -> Word[8] { for i in 0..2 with s: Word[8] = 1 { s } }\n",
                Reservations {
                    frames: |frames, count| {
                        frames.is_empty() && frames.try_reserve_exact(count).is_ok()
                    },
                    ..Reservations::DEFAULT
                },
                "evaluation call stack could not be reserved",
            ),
            // The index 0 needs no storage; the index 1 does.
            (
                "  spec f() -> Int { for i in 0..2 with s: Int = 0 { i } }\n",
                Reservations {
                    value_limbs: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "exact integer storage could not be reserved",
            ),
            // The fill is the first array and the update the second.
            (
                "  spec f() -> Word[8]^2 { [4; 2] with [1] = 3 }\n",
                Reservations {
                    array: refuse_second_array,
                    ..Reservations::DEFAULT
                },
                "evaluation array storage could not be reserved",
            ),
            (
                "  spec f() -> Word[8]^2 { [4; 2] }\n",
                Reservations {
                    array: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluation array storage could not be reserved",
            ),
        ] {
            let core = core(&format!("edition 2026; module m {{\n{members}}}\n"));
            let run = || {
                ARRAY_RESERVATIONS.with(|calls| calls.set(0));
                evaluate_with_reservations(
                    &core,
                    MAX_EVALUATION_STEPS_PER_SOURCE,
                    |values, capacity| values.try_reserve_exact(capacity).is_ok(),
                    reservations,
                )
            };
            let first = run();
            assert_eq!(first, run(), "{members}");
            assert!(first.values().is_none(), "{members}");
            let [diagnostic] = first.diagnostics() else {
                panic!("an allocation failure must produce exactly one diagnostic");
            };
            assert_eq!(
                diagnostic.message(),
                "reference evaluation result allocation failed"
            );
            assert_eq!(diagnostic.label(), label, "{members}");
        }
    }

    /// Nested array literals are rejected at the second level, but the whole
    /// pipeline still walks all of them within 1 MiB of stack.
    #[test]
    fn deeply_nested_rejected_arrays_fit_in_one_mebibyte_of_stack() {
        use crate::parser::MAX_EXPRESSION_NESTING;
        let text = format!(
            "edition 2026; module m {{ spec f(x: Word[32]) -> Word[32]^1 {{ {}x{} }} }}\n",
            "[".repeat(MAX_EXPRESSION_NESTING),
            "]".repeat(MAX_EXPRESSION_NESTING)
        );
        let worker = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || {
                let mut sources = SourceMap::new();
                let id = sources.add("evaluate.or", text).unwrap();
                let source = sources.get(id).unwrap();
                let lexed = lex(source, Edition::E2026);
                let parsed = parse(source, &lexed);
                assert_eq!(parsed.diagnostics(), []);
                let analyzed = analyze(source, parsed.ast().unwrap());
                analyzed
                    .diagnostics()
                    .iter()
                    .map(|diagnostic| (diagnostic.code(), diagnostic.message().to_owned()))
                    .collect::<Vec<_>>()
            })
            .unwrap();
        assert_eq!(
            worker.join().unwrap(),
            [(
                DiagnosticCode::TypeMismatch,
                String::from("an array literal cannot have type `Word[32]`")
            )]
        );
    }

    /// A chain of joins as long as an expression's height admits is checked
    /// in one frame, so a rejected chain reports once within 1 MiB of stack
    /// wherever its offending operand is.
    #[test]
    fn long_rejected_joins_fit_in_one_mebibyte_of_stack() {
        use crate::parser::MAX_EXPRESSION_HEIGHT;
        let length = MAX_EXPRESSION_HEIGHT - 1;
        let joined = " ++ [x]".repeat(MAX_EXPRESSION_HEIGHT - 3);
        let bodies = [
            format!("let y: Word[32]^{length} = missing ++ [x]{joined}; y[0]"),
            format!("let y: Word[32]^{length} = [x]{joined} ++ missing; y[0]"),
            format!("let y: Word[32]^{length} = \"\\xff\" ++ [x]{joined}; y[0]"),
            format!("let y: Word[32]^{length} = [x]{joined}; y[0]"),
        ];
        let sources = bodies
            .iter()
            .map(|body| {
                format!(
                    "edition 2026; module m {{\n  spec f(x: Word[32]) -> Word[32] {{ {body} }}\n}}\n"
                )
            })
            .collect::<Vec<_>>();
        let worker = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || {
                sources
                    .iter()
                    .map(|text| {
                        let mut sources = SourceMap::new();
                        let id = sources.add("evaluate.or", text.as_str()).unwrap();
                        let source = sources.get(id).unwrap();
                        let lexed = lex(source, Edition::E2026);
                        let parsed = parse(source, &lexed);
                        assert_eq!(parsed.diagnostics(), []);
                        let analyzed = analyze(source, parsed.ast().unwrap());
                        analyzed
                            .diagnostics()
                            .iter()
                            .map(|diagnostic| (diagnostic.code(), diagnostic.message().to_owned()))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap();
        let unknown = (
            DiagnosticCode::UnknownParameter,
            String::from("`missing` is not a parameter or binding of `f`"),
        );
        assert_eq!(
            worker.join().unwrap(),
            [
                vec![unknown.clone()],
                vec![unknown],
                vec![(
                    DiagnosticCode::TypeMismatch,
                    String::from(
                        "this byte string has type `Word[8]^1`, but `Word[32]^1` is required here"
                    ),
                )],
                vec![(
                    DiagnosticCode::ArrayLengthMismatch,
                    format!(
                        "`++` joins {} and 1 elements, {} in all, but `Word[32]^{length}` has \
                         {length}",
                        length - 2,
                        length - 1
                    ),
                )],
            ]
        );
    }

    /// A modulus that is not a constant is rejected, but the moduli written
    /// within it, nested as deeply as the parser admits, are still evaluated
    /// within 1 MiB of stack.
    #[test]
    fn deeply_nested_rejected_moduli_fit_in_one_mebibyte_of_stack() {
        use crate::parser::MAX_EXPRESSION_NESTING;
        let levels = (MAX_EXPRESSION_NESTING - 2) / 2;
        let nested = format!(
            "{}7{}",
            "(0 as Mod[".repeat(levels),
            "]) as Int".repeat(levels)
        );
        let text = format!(
            "edition 2026; module m {{\n  spec f(x: Mod[{nested}]) -> Int {{ 0 }}\n  \
             spec g(x: Word[32]) -> Word[32] {{ (x as Mod[{nested}]) as Word[32] }}\n}}\n"
        );
        let worker = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || {
                let mut sources = SourceMap::new();
                let id = sources.add("evaluate.or", text).unwrap();
                let source = sources.get(id).unwrap();
                let lexed = lex(source, Edition::E2026);
                let parsed = parse(source, &lexed);
                assert_eq!(parsed.diagnostics(), []);
                let analyzed = analyze(source, parsed.ast().unwrap());
                analyzed
                    .diagnostics()
                    .iter()
                    .map(|diagnostic| (diagnostic.code(), diagnostic.label().to_owned()))
                    .collect::<Vec<_>>()
            })
            .unwrap();
        let expected = (
            DiagnosticCode::InvalidModulus,
            String::from("not a constant integer expression"),
        );
        assert_eq!(worker.join().unwrap(), vec![expected; 2 * levels]);
    }

    fn bytes(values: &[u8]) -> CoreValue {
        let ty = ArrayType::new(&CoreType::Word8, u32::try_from(values.len()).unwrap()).unwrap();
        CoreValue::Array(
            CoreArray::new(
                ty,
                values.iter().map(|byte| CoreValue::Word8(*byte)).collect(),
            )
            .unwrap(),
        )
    }

    const CALLS: &str = concat!(
        "edition 2026; module calls {\n",
        "  spec mix(key: Word[8]^4, data: Word[8]^4) -> Word[8]^4 {\n",
        "    for i in 0..4 with out: Word[8]^4 = data { out with [i] = (data[i] ^ key[i]) <<< 1 }\n",
        "  }\n",
        "  spec scale(x: Int, flag: Bool, w: Word[64]) -> Int {\n",
        "    if flag { x * (w as Int) } else { -x }\n",
        "  }\n",
        "  spec spin(n: Word[32]) -> Word[32] { for i in 0..60000 with a: Word[32] = n { a + 1 } }\n",
        "  spec constant() -> Word[8] { 7 }\n",
        "}\n",
    );

    #[test]
    fn calls_run_one_function_on_host_values_with_exact_types() {
        let core = core(CALLS);
        let mut evaluator = Evaluator::new(&core).unwrap();
        assert_eq!(evaluator.module().name(), "calls");
        let mix = evaluator.function("mix").unwrap();
        let result = evaluator
            .call(
                mix,
                &[bytes(&[1, 2, 3, 0x80]), bytes(&[0x10, 0x20, 0x30, 0x01])],
                1_000,
            )
            .unwrap();
        assert_eq!(result.diagnostics(), []);
        assert!(!result.has_errors());
        assert_eq!(result.value(), Some(&bytes(&[0x22, 0x44, 0x66, 0x03])));
        assert!(result.steps() > 0);
        let scale = evaluator.function("scale").unwrap();
        let big = ExactInteger::from_u64(u64::MAX, reserve_value_limbs).unwrap();
        let result = evaluator
            .call(
                scale,
                &[
                    CoreValue::Int(big),
                    CoreValue::Bool(true),
                    CoreValue::Word64(u64::MAX),
                ],
                1_000,
            )
            .unwrap();
        assert_eq!(
            result.value().unwrap().to_string(),
            "340282366920938463426481119284349108225"
        );
        let negative = ExactInteger::from_u64(5, reserve_value_limbs).unwrap();
        let result = evaluator
            .call(
                scale,
                &[
                    CoreValue::Int(negative),
                    CoreValue::Bool(false),
                    CoreValue::Word64(3),
                ],
                1_000,
            )
            .unwrap();
        assert_eq!(result.into_value().unwrap().to_string(), "-5");
        // A parameterless function can be called too.
        let constant = evaluator.function("constant").unwrap();
        let result = evaluator.call(constant, &[], 1).unwrap();
        assert_eq!(result.value(), Some(&CoreValue::Word8(7)));
        assert_eq!(result.steps(), 1);
    }

    #[test]
    fn sized_functions_are_found_one_instance_at_a_time() {
        let core = core(concat!(
            "edition 2026; module sized {\n",
            "  spec pick[n in 1..3](x: Word[8]^n) -> Word[8] { x[n - 1] }\n",
            "  spec plain() -> Int { 1 }\n",
            "}\n",
        ));
        let mut evaluator = Evaluator::new(&core).unwrap();
        // Every instance of `pick` has its name, so a lookup by name alone
        // finds none of them rather than whichever comes first.
        assert!(evaluator.function("pick").is_none());
        assert!(evaluator.instance("pick", &[]).is_none());
        assert!(evaluator.instance("pick", &[3]).is_none());
        assert!(evaluator.instance("pick", &[1, 1]).is_none());
        let second = evaluator.instance("pick", &[2]).unwrap();
        assert_eq!(second.sizes(), [2]);
        assert!(evaluator.call(second, &[bytes(&[5])], 100).is_none());
        let result = evaluator.call(second, &[bytes(&[5, 6])], 100).unwrap();
        assert_eq!(result.value(), Some(&CoreValue::Word8(6)));
        let first = evaluator.instance("pick", &[1]).unwrap();
        let result = evaluator.call(first, &[bytes(&[5])], 100).unwrap();
        assert_eq!(result.value(), Some(&CoreValue::Word8(5)));
        let plain = evaluator.function("plain").unwrap();
        assert!(std::ptr::eq(
            plain,
            evaluator.instance("plain", &[]).unwrap()
        ));
        assert!(evaluator.instance("plain", &[1]).is_none());
    }

    #[test]
    fn calls_reject_foreign_functions_and_mistyped_arguments_before_evaluation() {
        let core = core(CALLS);
        let other = self::core(CALLS);
        let mut evaluator = Evaluator::new(&core).unwrap();
        let mix = evaluator.function("mix").unwrap();
        let foreign = other.functions().first().unwrap();
        let key = bytes(&[0; 4]);
        assert!(evaluator.function("absent").is_none());
        assert!(
            evaluator
                .call(foreign, &[key.clone(), key.clone()], 1_000)
                .is_none()
        );
        assert!(
            evaluator
                .call(mix, std::slice::from_ref(&key), 1_000)
                .is_none()
        );
        assert!(
            evaluator
                .call(mix, &[key.clone(), key.clone(), key.clone()], 1_000)
                .is_none()
        );
        assert!(
            evaluator
                .call(mix, &[key.clone(), bytes(&[0; 5])], 1_000)
                .is_none()
        );
        assert!(
            evaluator
                .call(mix, &[key.clone(), CoreValue::Word8(0)], 1_000)
                .is_none()
        );
        // The evaluator is still usable after refusals.
        assert!(
            evaluator
                .call(mix, &[key.clone(), key], 1_000)
                .unwrap()
                .value()
                .is_some()
        );
    }

    #[test]
    fn every_call_has_its_own_step_limit() {
        let core = core(CALLS);
        let mut evaluator = Evaluator::new(&core).unwrap();
        let spin = evaluator.function("spin").unwrap();
        let exact = evaluator
            .call(spin, &[CoreValue::Word32(1)], 1_000_000)
            .unwrap();
        assert_eq!(exact.value(), Some(&CoreValue::Word32(60_001)));
        let needed = exact.steps();
        // The same call succeeds with exactly its steps, twice in a row,
        // and fails with one fewer.
        for _ in 0..2 {
            let again = evaluator
                .call(spin, &[CoreValue::Word32(1)], needed)
                .unwrap();
            assert_eq!(again.value(), Some(&CoreValue::Word32(60_001)));
            assert_eq!(again.steps(), needed);
        }
        let short = evaluator
            .call(spin, &[CoreValue::Word32(1)], needed - 1)
            .unwrap();
        assert!(short.has_errors());
        assert_eq!(short.value(), None);
        let [diagnostic] = short.diagnostics() else {
            panic!("expected one diagnostic");
        };
        assert_eq!(diagnostic.code(), DiagnosticCode::EvaluationResourceLimit);
        assert_eq!(
            diagnostic.message(),
            "reference evaluation step limit exceeded"
        );
        // A failed call leaves the evaluator ready for the next one.
        let after = evaluator
            .call(spin, &[CoreValue::Word32(7)], needed)
            .unwrap();
        assert_eq!(after.value(), Some(&CoreValue::Word32(60_007)));
    }

    #[test]
    fn calls_agree_with_whole_module_evaluation() {
        let text = concat!(
            "edition 2026; module agree {\n",
            "  spec f(x: Word[32]) -> Word[32] { (x <<< 7) ^ 0x9e3779b9 }\n",
            "  spec g() -> Word[32] { f(0x01234567) }\n",
            "}\n",
        );
        let core = core(text);
        let evaluated = evaluate(&core);
        let expected = evaluated.values().unwrap().first().unwrap().value().clone();
        let mut evaluator = Evaluator::new(&core).unwrap();
        let f = evaluator.function("f").unwrap();
        let g = evaluator.function("g").unwrap();
        let direct = evaluator
            .call(f, &[CoreValue::Word32(0x0123_4567)], 100)
            .unwrap();
        assert_eq!(direct.value(), Some(&expected));
        assert_eq!(
            evaluator.call(g, &[], 100).unwrap().value(),
            Some(&expected)
        );
        assert!(format!("{evaluator:?}").contains("agree"));
    }

    /// The inverse of `value` modulo `modulus` when they are coprime, and 0
    /// otherwise, by the extended Euclidean algorithm over `i128`.
    fn reference_inverse(value: u128, modulus: u128) -> u128 {
        let (mut old, mut current) = (
            i128::try_from(value).unwrap(),
            i128::try_from(modulus).unwrap(),
        );
        let (mut old_coefficient, mut coefficient) = (1_i128, 0_i128);
        while current != 0 {
            let quotient = old.div_euclid(current);
            (old, current) = (current, old - quotient * current);
            (old_coefficient, coefficient) =
                (coefficient, old_coefficient - quotient * coefficient);
        }
        if old == 1 {
            u128::try_from(old_coefficient.rem_euclid(i128::try_from(modulus).unwrap())).unwrap()
        } else {
            0
        }
    }

    #[test]
    fn residue_arithmetic_matches_a_u128_reference() {
        // Small, composite, prime, and 61- and 64-bit prime moduli.
        let moduli: [u128; 6] = [2, 7, 256, 3329, (1 << 61) - 1, (1 << 64) - 59];
        let mut members = String::new();
        let mut expected = Vec::new();
        for (i, &modulus) in moduli.iter().enumerate() {
            let corpus =
                [0, 1, 2, modulus / 2, modulus - 2, modulus - 1].map(|value| value % modulus);
            for (j, &left) in corpus.iter().enumerate() {
                for (k, &right) in corpus.iter().enumerate() {
                    let sum = (left + right) % modulus;
                    let difference = (left + modulus - right) % modulus;
                    let product = (left * right) % modulus;
                    let quotient = (left * reference_inverse(right, modulus)) % modulus;
                    let negation = (modulus - left) % modulus;
                    members.push_str(&format!(
                        "  spec r{i}_{j}_{k}() -> Mod[{modulus}]^5 {{ \
                         let a: Mod[{modulus}] = {left}; let b: Mod[{modulus}] = {right}; \
                         [a + b, a - b, a * b, a / b, -a] }}\n"
                    ));
                    expected.push(format!(
                        "r{i}_{j}_{k} = [{sum}, {difference}, {product}, {quotient}, {negation}]"
                    ));
                    members.push_str(&format!(
                        "  spec e{i}_{j}_{k}() -> Bool^2 {{ \
                         let a: Mod[{modulus}] = {left}; let b: Mod[{modulus}] = {right}; \
                         [a == b, a != b] }}\n"
                    ));
                    expected.push(format!(
                        "e{i}_{j}_{k} = [{}, {}]",
                        left == right,
                        left != right
                    ));
                }
            }
        }
        assert_eq!(values_of(&members), expected);
    }

    #[test]
    fn conversions_reduce_into_residues_and_give_least_residues_out() {
        let integers: [i128; 7] = [
            0,
            -1,
            6,
            7,
            -18_446_744_073_709_551_617,
            170_141_183_460_469_231_731_687_303_715_884_105_727,
            -170_141_183_460_469_231_731_687_303_715_884_105_727,
        ];
        let moduli: [i128; 3] = [7, 256, (1 << 89) - 1];
        let mut members = String::new();
        let mut expected = Vec::new();
        for (i, &value) in integers.iter().enumerate() {
            for (j, &modulus) in moduli.iter().enumerate() {
                let residue = value.rem_euclid(modulus);
                let low = u8::try_from(residue % 256).unwrap();
                members.push_str(&format!(
                    "  spec c{i}_{j}() -> Int^2 {{ let n: Int = {value}; \
                     let r: Mod[{modulus}] = n as Mod[{modulus}]; \
                     [r as Int, (r as Word[8]) as Int] }}\n"
                ));
                expected.push(format!("c{i}_{j} = [{residue}, {low}]"));
            }
        }
        // Words reduce by their unsigned values, and residues convert
        // between moduli through their least residues.
        members.push_str(concat!(
            "  spec w() -> Mod[7]^4 { let x: Word[64] = 0xffffffffffffffff; ",
            "let y: Mod[256] = -1; let z: Mod[11] = 10; ",
            "[x as Mod[7], y as Mod[7], z as Mod[7], (x as Word[8]) as Mod[7]] }\n",
        ));
        expected.push(String::from("w = [1, 3, 3, 3]"));
        assert_eq!(values_of(&members), expected);
    }

    #[test]
    fn residue_steps_follow_the_normative_cost_table() {
        // For a modulus of d 32-bit digits, `+`, `-`, prefix `-`, `==`, and
        // `!=` cost 1 + d, `*` costs 1 + 2d^2, and `/` costs 1 + 64d^2. A
        // conversion to `Mod[m]` costs 1 + d times the operand's digits;
        // one from a residue costs 1. A literal or a read costs 1.
        for (members, steps) in [
            ("  spec r() -> Mod[7] { let a: Mod[7] = 3; a + a }\n", 5),
            ("  spec r() -> Mod[7] { let a: Mod[7] = 3; a - a }\n", 5),
            ("  spec r() -> Mod[7] { let a: Mod[7] = 3; a * a }\n", 6),
            ("  spec r() -> Mod[7] { let a: Mod[7] = 3; a / a }\n", 68),
            ("  spec r() -> Mod[7] { let a: Mod[7] = 3; -a }\n", 4),
            ("  spec r() -> Bool { let a: Mod[7] = 3; a == a }\n", 5),
            (
                "  spec r() -> Mod[(1 << 255) - 19] { let a: Mod[(1 << 255) - 19] = 3; a * a }\n",
                132,
            ),
            (
                "  spec r() -> Mod[(1 << 255) - 19] { let a: Mod[(1 << 255) - 19] = 3; a / a }\n",
                4100,
            ),
            (
                "  spec r() -> Mod[(1 << 64) + 13] { let a: Mod[(1 << 64) + 13] = 3; a + a }\n",
                7,
            ),
            // `n` has three digits and the modulus two.
            (
                "  spec r() -> Mod[(1 << 61) - 1] { let n: Int = 18446744073709551616; \
                 n as Mod[(1 << 61) - 1] }\n",
                9,
            ),
            ("  spec r() -> Mod[7] { let n: Int = 0; n as Mod[7] }\n", 3),
            ("  spec r() -> Int { let a: Mod[7] = 3; a as Int }\n", 3),
        ] {
            let core = core(&format!("edition 2026; module m {{\n{members}}}\n"));
            let exact = evaluate_with_limit(&core, steps);
            assert_eq!(exact.diagnostics(), [], "{members}");
            let short = evaluate_with_limit(&core, steps - 1);
            assert!(short.values().is_none(), "{members}");
        }
    }

    #[test]
    fn residues_cross_the_call_interface_with_exact_types() {
        let core = core(concat!(
            "edition 2026; module ring {\n",
            "  type F = Mod[(1 << 130) - 5];\n",
            "  spec mul(x: F, y: F) -> F { x * y }\n",
            "}\n",
        ));
        let mut evaluator = Evaluator::new(&core).unwrap();
        let mul = evaluator.function("mul").unwrap();
        let field = evaluator.module().functions()[0].parameters()[0].clone();
        let Some(modulus) = field.modulus() else {
            panic!("expected a residue type");
        };
        let residue = |value: u64| {
            CoreValue::Mod(
                Residue::new(
                    modulus,
                    ExactInteger::from_u64(value, reserve_value_limbs).unwrap(),
                )
                .unwrap(),
            )
        };
        let result = evaluator
            .call(mul, &[residue(u64::MAX), residue(u64::MAX)], 1_000)
            .unwrap();
        assert_eq!(result.diagnostics(), []);
        assert_eq!(
            result.value().unwrap().to_string(),
            "340282366920938463426481119284349108225"
        );
        assert_eq!(result.value().unwrap().ty(), field);
        // A residue of another modulus is refused before evaluation.
        let other = Modulus::new(&ExactInteger::from_u64(7, reserve_value_limbs).unwrap()).unwrap();
        let foreign = CoreValue::Mod(
            Residue::new(
                other,
                ExactInteger::from_u64(3, reserve_value_limbs).unwrap(),
            )
            .unwrap(),
        );
        assert!(evaluator.call(mul, &[foreign, residue(1)], 1_000).is_none());
    }

    #[test]
    fn block_bindings_are_evaluated_at_every_step_and_read_from_their_slots() {
        let members = concat!(
            "  spec mix() -> Word[32] {\n",
            "    for i in 0..64 with s: Word[32] = 0x6a09e667 {\n",
            "      let t: Word[32] = s ^ (i as Word[32]);\n",
            "      let u: Word[32] = (t <<< 7) + t;\n",
            "      u ^ (s >>> 3)\n",
            "    }\n",
            "  }\n",
            "  spec pick(n: Int) -> Int {\n",
            "    if n < 0 { let m: Int = -n; m * m }\n",
            "    else if n == 0 { 0 }\n",
            "    else { let d: Int = n + 1; let e: Int = d * n; e - d }\n",
            "  }\n",
            "  spec picks() -> Int^3 { [pick(-3), pick(0), pick(4)] }\n",
            "  spec grid() -> Int {\n",
            "    for i in 0..3 with s: Int = 0 {\n",
            "      let row: Int = i * 10;\n",
            "      s + (for j in 0..3 with t: Int = 0 {\n",
            "        let cell: Int = row + j;\n",
            "        if (cell % 2) == 0 { let half: Int = cell / 2; t + half } else { t }\n",
            "      })\n",
            "    }\n",
            "  }\n",
            "  spec f(x: Int) -> Int { for i in 0..2 with s: Int = x { let y: Int = g(s); y + 1 } }\n",
            "  spec g(k: Int) -> Int { if k > 0 { let d: Int = k * 2; d } else { 0 } }\n",
            "  spec calls() -> Int { f(1) }\n",
            "  spec residues() -> Int {\n",
            "    (for i in 0..3 with s: Mod[7] = 3 { let t: Mod[7] = s * s + (i as Mod[7]); t }) as Int\n",
            "  }\n",
        );
        let mut mix = 0x6a09_e667_u32;
        for i in 0..64 {
            let t = mix ^ i;
            let u = t.rotate_left(7).wrapping_add(t);
            mix = u ^ mix.rotate_right(3);
        }
        assert_eq!(
            values_of(members),
            [
                format!("mix = {}", render_word(32, u128::from(mix))),
                String::from("picks = [9, 0, 15]"),
                // Rows 0, 10, and 20 contribute 0 + 1, 5 + 6, and 10 + 11.
                String::from("grid = 33"),
                String::from("calls = 7"),
                // 3 * 3 + 0 = 2, 2 * 2 + 1 = 5, and 5 * 5 + 2 = 6 modulo 7.
                String::from("residues = 6"),
            ]
        );
    }

    #[test]
    fn only_the_chosen_branch_binds_its_names() {
        // The untaken branch's binding would exceed the whole step budget.
        let members = concat!(
            "  spec heavy(x: Int) -> Int {\n",
            "    for i in 0..65536 with s: Int = x { for j in 0..65536 with t: Int = s { t } }\n",
            "  }\n",
            "  spec lazy() -> Int { if true { let one: Int = 1; one } else { let h: Int = heavy(0); h } }\n",
        );
        assert_eq!(values_of(members), ["lazy = 1"]);
    }

    #[test]
    fn inconsistent_blocks_fail_closed() {
        let base = core(concat!(
            "edition 2026; module m {\n",
            "  spec step() -> Word[8] { for i in 0..2 with s: Word[8] = 1 { let t: Word[8] = s + s; t ^ s } }\n",
            "  spec branch() -> Int { if true { let a: Int = 2; a * a } else { let b: Int = 3; b } }\n",
            "}\n"
        ));
        let step = &base.functions[0].loops[0];
        assert_eq!(step.bindings()[0].end(), 3);
        assert_eq!(
            step.step.nodes[3].kind,
            CoreNodeKind::StepBinding {
                loop_id: 0,
                index: 0
            }
        );
        let branch = &base.functions[1].conditionals[0];
        assert_eq!(branch.then_bindings()[0].end(), 1);
        assert_eq!(
            branch.then_branch.nodes[1].kind,
            CoreNodeKind::BranchBinding {
                conditional: 0,
                index: 0
            }
        );
        let mutations: [fn(&mut CoreModule); 11] = [
            // A step reads a binding the step lacks.
            |core| core.functions[0].loops[0].bindings.clear(),
            // A binding's value ends past the end of its step.
            |core| core.functions[0].loops[0].bindings[0].end = 6,
            // A binding's value is empty.
            |core| core.functions[0].loops[0].bindings[0].end = 0,
            // A binding's value has another type than the binding.
            |core| core.functions[0].loops[0].bindings[0].ty = CoreType::Word16,
            // A step reads a binding that has no value yet.
            |core| {
                core.functions[0].loops[0].step.nodes[3].kind = CoreNodeKind::StepBinding {
                    loop_id: 0,
                    index: 1,
                };
            },
            // A read names a loop that is not active.
            |core| {
                core.functions[0].loops[0].step.nodes[3].kind = CoreNodeKind::StepBinding {
                    loop_id: 1,
                    index: 0,
                };
            },
            // A read claims another type than its binding's.
            |core| core.functions[0].loops[0].step.nodes[3].ty = CoreType::Word16,
            // The chosen branch reads a binding only its other branch has.
            |core| {
                let conditional = &mut core.functions[1].conditionals[0];
                conditional.else_bindings = std::mem::take(&mut conditional.then_bindings);
            },
            // A branch read names a conditional that is not being evaluated.
            |core| {
                core.functions[1].conditionals[0].then_branch.nodes[1].kind =
                    CoreNodeKind::BranchBinding {
                        conditional: 1,
                        index: 0,
                    };
            },
            // A function body reads a branch's binding.
            |core| {
                core.functions[1].body.nodes[0].kind = CoreNodeKind::BranchBinding {
                    conditional: 0,
                    index: 0,
                };
                core.functions[1].body.nodes[0].ty = CoreType::Int;
            },
            // A branch claims a binding that leaves no value of its own.
            |core| {
                let conditional = &mut core.functions[1].conditionals[0];
                let mut second = conditional.then_bindings[0].clone();
                second.end = 2;
                conditional.then_bindings.push(second);
            },
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
    fn tuples_hold_their_elements_in_order_and_select_them_by_position() {
        let members = concat!(
            "  spec pair() -> (Int, Word[8]^2) { (-3, [1, 2]) }\n",
            "  spec second() -> Word[8] { pair().1[1] }\n",
            "  spec swap(p: (Int, Bool)) -> (Bool, Int) { (p.1, p.0) }\n",
            "  spec swapped() -> (Bool, Int) { swap((7, true)) }\n",
            "  spec fibonacci() -> Int {\n",
            "    let (f: Int, g: Int) = for i in 0..10 with (a: Int, b: Int) = (0, 1) { (b, a + b) };\n",
            "    g - f\n",
            "  }\n",
            "  spec rounds() -> (Word[8], Word[8]) {\n",
            "    for i in 0..3 with (x: Word[8], y: Word[8]) = (1, 2) {\n",
            "      let (s: Word[8], t: Word[8]) = (x + y, x ^ y); (t <<< 1, s)\n",
            "    }\n",
            "  }\n",
        );
        // (x, y) = (1, 2), then (6, 3), (10, 9), and (6, 19).
        assert_eq!(
            values_of(members),
            [
                "pair = (-3, [0x01, 0x02])",
                "second = 0x02",
                "swapped = (true, 7)",
                "fibonacci = 34",
                "rounds = (0x06, 0x13)",
            ]
        );
    }

    #[test]
    fn tuple_storage_reservation_failures_return_no_values() {
        let core = core(concat!(
            "edition 2026; module m {\n",
            "  spec pair() -> (Int, Int) { (1, 2) }\n",
            "}\n"
        ));
        for (reservations, label) in [
            (
                Reservations {
                    array: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluation tuple storage could not be reserved",
            ),
            (
                Reservations {
                    result_array: |_, _| false,
                    ..Reservations::DEFAULT
                },
                "evaluated tuple storage could not be reserved",
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
        }
    }

    #[test]
    fn inconsistent_tuples_fail_closed() {
        let base = core(concat!(
            "edition 2026; module m {\n",
            "  spec pair() -> (Int, Bool) { (1, true) }\n",
            "  spec first() -> Int { pair().0 }\n",
            "}\n"
        ));
        let pair = base.functions[0].result_type.clone();
        assert_eq!(
            base.functions[0].body.nodes[2].kind,
            CoreNodeKind::Tuple { elements: 2 }
        );
        assert_eq!(
            base.functions[1].body.nodes[1].kind,
            CoreNodeKind::Project { index: 0 }
        );
        let ints = CoreType::Tuple(TupleType::new(&[CoreType::Int, CoreType::Int]).unwrap());
        assert_ne!(pair, ints);
        type Mutation<'a> = Box<dyn Fn(&mut CoreModule) + 'a>;
        let mutations: [Mutation<'_>; 8] = [
            // A tuple claims more elements than its type has.
            Box::new(|core| {
                core.functions[0].body.nodes[2].kind = CoreNodeKind::Tuple { elements: 3 };
            }),
            // A tuple claims fewer elements than its type has.
            Box::new(|core| {
                core.functions[0].body.nodes[2].kind = CoreNodeKind::Tuple { elements: 1 };
            }),
            // A tuple's type is not a tuple type.
            Box::new(|core| core.functions[0].body.nodes[2].ty = CoreType::Int),
            // A tuple's element has another type than its type says.
            Box::new(|core| {
                core.functions[0].body.nodes[2].ty = ints.clone();
                core.functions[0].result_type = ints.clone();
            }),
            // A selection names an element the tuple lacks.
            Box::new(|core| {
                core.functions[1].body.nodes[1].kind = CoreNodeKind::Project { index: 2 };
            }),
            // A selection claims another type than its element's.
            Box::new(|core| core.functions[1].body.nodes[1].ty = CoreType::Bool),
            // A selection applies to a value that is not a tuple.
            Box::new(|core| {
                let literal = core.functions[0].body.nodes[0].clone();
                core.functions[1].body.nodes[0] = literal;
            }),
            // A function claims a tuple result of another type.
            Box::new(|core| core.functions[0].result_type = ints.clone()),
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
    fn byte_strings_joins_and_slices_evaluate_in_index_order() {
        let members = concat!(
            r#"  spec text() -> Word[8]^5 { "a\\\"\x00~" }"#,
            "\n",
            "  spec hex() -> Word[8]^4 { hex\"DE ad be EF\" }\n",
            "  spec joined() -> Word[16]^3 { [1, 2] ++ [3] }\n",
            "  spec padded() -> Word[8]^8 { \"abc\" ++ hex\"80\" ++ [0; 3] ++ [24] }\n",
            "  spec halves(x: Word[8]^4) -> Word[8]^4 { x[2..] ++ x[..2] }\n",
            "  spec swapped() -> Word[8]^4 { halves(\"abcd\") }\n",
            "  spec reversed() -> Word[8]^4 {\n",
            "    let t: Word[8]^4 = \"abcd\";\n",
            "    for i in 0..4 with y: Word[8]^4 = t { y with [i..i + 1] = t[3 - i..4 - i] }\n",
            "  }\n",
            "  spec spliced() -> Int^4 { [1, 2, 3, 4] with [1..3] = [-5; 2] }\n",
            "  spec fresh() -> Word[8]^4 {\n",
            "    for i in 0..2 with y: Word[8]^4 = \"wxyz\" { \"abcd\" with [i..i + 1] = \"-\" }\n",
            "  }\n",
        );
        // A literal is shared, never changed: the second step's `"abcd"`
        // is the first's, unchanged by the first step's update.
        assert_eq!(
            values_of(members),
            [
                "text = [0x61, 0x5c, 0x22, 0x00, 0x7e]",
                "hex = [0xde, 0xad, 0xbe, 0xef]",
                "joined = [0x0001, 0x0002, 0x0003]",
                "padded = [0x61, 0x62, 0x63, 0x80, 0x00, 0x00, 0x00, 0x18]",
                "swapped = [0x63, 0x64, 0x61, 0x62]",
                "reversed = [0x64, 0x63, 0x62, 0x61]",
                "spliced = [1, -5, -5, 4]",
                "fresh = [0x61, 0x2d, 0x63, 0x64]",
            ]
        );
    }

    #[test]
    fn join_and_slice_reservation_failures_return_no_values() {
        for members in [
            "  spec a() -> Word[8]^6 { \"abc\" ++ \"def\" }\n",
            "  spec a() -> Word[8]^2 { let t: Word[8]^4 = \"abcd\"; t[1..3] }\n",
            "  spec a() -> Word[8]^4 { \"abcd\" with [0..2] = \"xy\" }\n",
        ] {
            let core = core(&format!("edition 2026; module m {{\n{members}}}\n"));
            let run = || {
                evaluate_with_reservations(
                    &core,
                    MAX_EVALUATION_STEPS_PER_SOURCE,
                    |values, capacity| values.try_reserve_exact(capacity).is_ok(),
                    Reservations {
                        array: |_, _| false,
                        ..Reservations::DEFAULT
                    },
                )
            };
            let first = run();
            assert_eq!(first, run());
            assert!(first.values().is_none(), "{members}");
            let [diagnostic] = first.diagnostics() else {
                panic!("an allocation failure must produce exactly one diagnostic");
            };
            assert_eq!(
                diagnostic.message(),
                "reference evaluation result allocation failed"
            );
            assert_eq!(
                diagnostic.label(),
                "evaluation array storage could not be reserved",
                "{members}"
            );
        }
    }

    #[test]
    fn inconsistent_joins_and_slices_fail_closed() {
        let base = core(concat!(
            "edition 2026; module m {\n",
            "  spec join() -> Word[8]^4 { \"ab\" ++ \"cd\" }\n",
            "  spec slice() -> Word[8]^2 { let t: Word[8]^4 = \"abcd\"; t[1..3] }\n",
            "  spec update() -> Word[8]^4 { let t: Word[8]^4 = \"abcd\"; t with [1..3] = \"xy\" }\n",
            "}\n"
        ));
        fn words(element: &CoreType, length: u32) -> CoreType {
            CoreType::Array(ArrayType::new(element, length).unwrap())
        }
        fn int(value: u64) -> CoreNodeKind {
            CoreNodeKind::Literal(CoreValue::Int(
                ExactInteger::from_u64(value, |limbs, count| {
                    limbs.try_reserve_exact(count).is_ok()
                })
                .unwrap(),
            ))
        }
        assert_eq!(base.functions[0].body.nodes[2].kind, CoreNodeKind::Concat);
        assert_eq!(base.functions[1].body.nodes[3].kind, CoreNodeKind::Slice);
        assert_eq!(
            base.functions[2].body.nodes[4].kind,
            CoreNodeKind::SliceUpdate
        );
        let mutations: [fn(&mut CoreModule); 14] = [
            // A join's type is not an array type.
            |core| core.functions[0].body.nodes[2].ty = CoreType::Word8,
            // A join claims more elements than its operands have.
            |core| {
                core.functions[0].body.nodes[2].ty = words(&CoreType::Word8, 5);
                core.functions[0].result_type = words(&CoreType::Word8, 5);
            },
            // A join's operands have another element type than its own.
            |core| {
                core.functions[0].body.nodes[2].ty = words(&CoreType::Word16, 4);
                core.functions[0].result_type = words(&CoreType::Word16, 4);
            },
            // A join's operand is not an array.
            |core| {
                core.functions[0].body.nodes[1].kind = CoreNodeKind::Literal(CoreValue::Word8(1));
                core.functions[0].body.nodes[1].ty = CoreType::Word8;
            },
            // A join takes an operand from below its expression.
            |core| {
                core.functions[0].body.nodes.remove(0);
            },
            // A byte string claims another type than its bytes have.
            |core| core.functions[0].body.nodes[0].ty = words(&CoreType::Word8, 3),
            // A slice ends before it starts.
            |core| {
                core.functions[1].body.nodes[1].kind = int(3);
                core.functions[1].body.nodes[2].kind = int(1);
            },
            // A slice reaches past the end of its array.
            |core| {
                core.functions[1].body.nodes[1].kind = int(3);
                core.functions[1].body.nodes[2].kind = int(5);
            },
            // A slice claims another length than its bounds give.
            |core| {
                core.functions[1].body.nodes[3].ty = words(&CoreType::Word8, 3);
                core.functions[1].result_type = words(&CoreType::Word8, 3);
            },
            // A slice's bound is not an `Int`.
            |core| {
                core.functions[1].body.nodes[1].kind = CoreNodeKind::Literal(CoreValue::Word8(1));
                core.functions[1].body.nodes[1].ty = CoreType::Word8;
            },
            // A slice's array has another element type than its own.
            |core| {
                core.functions[1].body.nodes[3].ty = words(&CoreType::Word16, 2);
                core.functions[1].result_type = words(&CoreType::Word16, 2);
            },
            // A slice update's value has another length than its bounds.
            |core| {
                core.functions[2].body.nodes[3].kind = CoreNodeKind::Literal(CoreValue::Array(
                    CoreArray::new(
                        ArrayType::new(&CoreType::Word8, 3).unwrap(),
                        vec![CoreValue::Word8(0); 3],
                    )
                    .unwrap(),
                ));
                core.functions[2].body.nodes[3].ty = words(&CoreType::Word8, 3);
            },
            // A slice update claims another type than its array's.
            |core| {
                core.functions[2].body.nodes[4].ty = words(&CoreType::Word8, 5);
                core.functions[2].result_type = words(&CoreType::Word8, 5);
            },
            // A slice update takes an operand from below its expression.
            |core| {
                core.functions[2].body.nodes.remove(3);
            },
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
}
