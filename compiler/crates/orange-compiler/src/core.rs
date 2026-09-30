//! Typed, source-mapped Core for the Orange 2026 semantic fragment.

use std::cmp::Ordering;
use std::fmt;

use crate::parser::{BinaryOperator, UnaryOperator};
use crate::source::Span;

pub(crate) const MAX_EXACT_INTEGER_BITS: usize = 16_384;
const BINARY_LIMB_BITS: usize = 32;
const MAX_BINARY_LIMBS: usize = MAX_EXACT_INTEGER_BITS.div_ceil(BINARY_LIMB_BITS);
// One base-1,000,000,000 limb carries more than 27 binary bits. Using the
// weaker 27-bit bound keeps this capacity an integer-only, auditable upper
// bound rather than relying on floating-point logarithms.
const MAX_DECIMAL_LIMBS: usize = MAX_EXACT_INTEGER_BITS.div_ceil(27);

/// A successfully analyzed Orange module.
///
/// Core storage is read-only outside this crate so callers cannot reorder
/// functions, duplicate identities, or replace a checked value.
///
/// ```compile_fail
/// use orange_compiler::CoreModule;
///
/// fn discard_checked_functions(core: &mut CoreModule) {
///     core.functions.clear();
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreModule {
    /// Full source extent of the module declaration.
    pub(crate) span: Span,
    /// Exact ASCII module name.
    pub(crate) name: String,
    /// Typed functions in deterministic source order.
    pub(crate) functions: Vec<CoreFunction>,
}

impl CoreModule {
    /// Returns the full source extent of the module declaration.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact ASCII module name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns typed functions in deterministic source order.
    #[must_use]
    pub fn functions(&self) -> &[CoreFunction] {
        &self.functions
    }
}

/// A dense, source-ordered identity within one [`CoreModule`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreFunctionId(u32);

impl CoreFunctionId {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    /// Returns the zero-based source-order index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// One typed specification function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreFunction {
    /// Dense source-order identity.
    pub(crate) id: CoreFunctionId,
    /// Full source extent of the function declaration.
    pub(crate) span: Span,
    /// Exact ASCII function name.
    pub(crate) name: String,
    /// Source extent of the function name.
    pub(crate) name_span: Span,
    /// Parameter types in declaration order.
    pub(crate) parameters: Vec<CoreType>,
    /// Statically checked result type.
    pub(crate) result_type: CoreType,
    /// `let` bindings in source order, evaluated before the body.
    pub(crate) locals: Vec<CoreLocal>,
    /// Statically checked body.
    pub(crate) body: CoreExpression,
    /// Loops of the bindings and the body, numbered in source order of
    /// their `for` keywords.
    pub(crate) loops: Vec<CoreLoop>,
    /// Conditionals of the bindings, the body, and the loops, numbered in
    /// source order of their `if` keywords.
    pub(crate) conditionals: Vec<CoreConditional>,
}

impl CoreFunction {
    /// Returns the dense source-order identity.
    #[must_use]
    pub const fn id(&self) -> CoreFunctionId {
        self.id
    }

    /// Returns the full source extent of the function declaration.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact ASCII function name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source extent of the function name.
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.name_span
    }

    /// Returns parameter types in declaration order.
    #[must_use]
    pub fn parameters(&self) -> &[CoreType] {
        &self.parameters
    }

    /// Returns the statically checked result type.
    #[must_use]
    pub const fn result_type(&self) -> CoreType {
        self.result_type
    }

    /// Returns the `let` bindings in source order.
    #[must_use]
    pub fn locals(&self) -> &[CoreLocal] {
        &self.locals
    }

    /// Returns the statically checked body.
    #[must_use]
    pub const fn body(&self) -> &CoreExpression {
        &self.body
    }

    /// Returns the loops of the bindings and the body, numbered in source
    /// order of their `for` keywords.
    #[must_use]
    pub fn loops(&self) -> &[CoreLoop] {
        &self.loops
    }

    /// Returns the conditionals of the bindings, the body, and the loops,
    /// numbered in source order of their `if` keywords.
    #[must_use]
    pub fn conditionals(&self) -> &[CoreConditional] {
        &self.conditionals
    }
}

/// Highest admitted loop bound.
pub const MAX_LOOP_BOUND: u32 = 65_536;

/// One bounded loop `for i in start..end with s: T = init { step }`.
///
/// The loop's `Fold` node takes the initial value from its operand subtree;
/// the step is a separate expression evaluated once for each index from
/// `start` up to, but not including, `end`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreLoop {
    /// Full source extent of the loop, from `for` through `}`.
    pub(crate) span: Span,
    /// Exact ASCII name of the loop index.
    pub(crate) index_name: String,
    /// Exact ASCII name of the accumulator.
    pub(crate) accumulator_name: String,
    /// Declared type of the accumulator and of the loop.
    pub(crate) ty: CoreType,
    /// First index.
    pub(crate) start: u32,
    /// One past the last index; greater than `start`.
    pub(crate) end: u32,
    /// The number of the function's bindings in scope in the step.
    pub(crate) visible_locals: u32,
    /// The loops whose index and accumulator are in scope in the step,
    /// outermost first, ending with this loop.
    pub(crate) scope: Vec<u32>,
    /// Statically checked step.
    pub(crate) step: CoreExpression,
}

impl CoreLoop {
    /// Returns the full source extent of the loop.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact ASCII name of the loop index.
    #[must_use]
    pub fn index_name(&self) -> &str {
        &self.index_name
    }

    /// Returns the exact ASCII name of the accumulator.
    #[must_use]
    pub fn accumulator_name(&self) -> &str {
        &self.accumulator_name
    }

    /// Returns the declared type of the accumulator and of the loop.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        self.ty
    }

    /// Returns the first index.
    #[must_use]
    pub const fn start(&self) -> u32 {
        self.start
    }

    /// Returns one past the last index.
    #[must_use]
    pub const fn end(&self) -> u32 {
        self.end
    }

    /// Returns the number of the function's bindings in scope in the step.
    #[must_use]
    pub const fn visible_locals(&self) -> u32 {
        self.visible_locals
    }

    /// Returns the loops in scope in the step, outermost first, ending with
    /// this loop.
    #[must_use]
    pub fn scope(&self) -> &[u32] {
        &self.scope
    }

    /// Returns the statically checked step.
    #[must_use]
    pub const fn step(&self) -> &CoreExpression {
        &self.step
    }
}

/// One conditional `if c { a } else { b }`.
///
/// The conditional's `Choose` node consumes the condition, its one operand
/// subtree; each branch is a separate expression, and only the branch the
/// condition selects is evaluated. An `else if` arm is a conditional whose
/// `Choose` node ends the enclosing conditional's `else` branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreConditional {
    /// Source extent of the conditional from this `if` through its last `}`.
    pub(crate) span: Span,
    /// Type of both branches and of the conditional.
    pub(crate) ty: CoreType,
    /// The number of the function's bindings in scope in the branches.
    pub(crate) visible_locals: u32,
    /// The loops whose index and accumulator are in scope in the branches,
    /// outermost first.
    pub(crate) scope: Vec<u32>,
    /// The value when the condition is true.
    pub(crate) then_branch: CoreExpression,
    /// The value when the condition is false.
    pub(crate) else_branch: CoreExpression,
}

impl CoreConditional {
    /// Returns the source extent of the conditional from this `if` through
    /// its last `}`.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the type of both branches and of the conditional.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        self.ty
    }

    /// Returns the number of the function's bindings in scope in the
    /// branches.
    #[must_use]
    pub const fn visible_locals(&self) -> u32 {
        self.visible_locals
    }

    /// Returns the loops in scope in the branches, outermost first.
    #[must_use]
    pub fn scope(&self) -> &[u32] {
        &self.scope
    }

    /// Returns the value when the condition is true.
    #[must_use]
    pub const fn then_branch(&self) -> &CoreExpression {
        &self.then_branch
    }

    /// Returns the value when the condition is false.
    #[must_use]
    pub const fn else_branch(&self) -> &CoreExpression {
        &self.else_branch
    }
}

/// One `let` binding of a typed specification function.
///
/// A binding's value may use the function's parameters and the bindings
/// before it, and every binding is evaluated exactly once, in source order,
/// before the function's body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreLocal {
    /// Full source extent of the binding, from `let` through `;`.
    pub(crate) span: Span,
    /// Exact ASCII binding name.
    pub(crate) name: String,
    /// Source extent of the binding name.
    pub(crate) name_span: Span,
    /// Declared type of the binding.
    pub(crate) ty: CoreType,
    /// Statically checked bound expression.
    pub(crate) value: CoreExpression,
}

impl CoreLocal {
    /// Returns the full source extent of the binding.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact ASCII binding name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source extent of the binding name.
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.name_span
    }

    /// Returns the declared type of the binding.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        self.ty
    }

    /// Returns the statically checked bound expression.
    #[must_use]
    pub const fn value(&self) -> &CoreExpression {
        &self.value
    }
}

/// A typed expression tree stored in postorder.
///
/// Every node follows the nodes of its operands, so the final node is the
/// root and evaluation needs only a value stack. The representation is
/// private to this compiler and has no canonical encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreExpression {
    /// Nodes in postorder; the last node is the root.
    pub(crate) nodes: Vec<CoreNode>,
}

impl CoreExpression {
    /// Returns nodes in postorder; the last node is the root.
    #[must_use]
    pub fn nodes(&self) -> &[CoreNode] {
        &self.nodes
    }

    /// Returns the root node.
    #[must_use]
    pub fn root(&self) -> Option<&CoreNode> {
        self.nodes.last()
    }

    /// Returns the value of an expression that is exactly one literal.
    #[must_use]
    pub fn literal(&self) -> Option<&CoreValue> {
        match self.nodes.as_slice() {
            [
                CoreNode {
                    kind: CoreNodeKind::Literal(value),
                    ..
                },
            ] => Some(value),
            _ => None,
        }
    }
}

/// One typed Core expression node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreNode {
    /// Source extent of the expression this node represents.
    pub(crate) span: Span,
    /// Statically checked type of the node's value.
    pub(crate) ty: CoreType,
    /// Node operation.
    pub(crate) kind: CoreNodeKind,
}

impl CoreNode {
    /// Returns the source extent of the expression this node represents.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the statically checked type of the node's value.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        self.ty
    }

    /// Returns the node operation.
    #[must_use]
    pub const fn kind(&self) -> &CoreNodeKind {
        &self.kind
    }
}

/// The operation of one Core node.
///
/// Operands are the values of the immediately preceding complete subtrees,
/// in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreNodeKind {
    /// A checked literal value.
    Literal(CoreValue),
    /// The value of the parameter at this zero-based index.
    Parameter(u32),
    /// The value of the function's `let` binding at this zero-based index.
    Local(u32),
    /// A call of a function with this many argument subtrees.
    Call {
        /// The called function.
        function: CoreFunctionId,
        /// The number of argument subtrees.
        arguments: u32,
    },
    /// A prefix operator applied to one operand subtree.
    Unary(UnaryOperator),
    /// An arithmetic, bitwise, division, or logical operator applied to two
    /// operand subtrees of this node's type.
    Binary(BinaryOperator),
    /// A comparison of two operand subtrees of one type, giving a `Bool`.
    Compare {
        /// The comparison operator.
        operator: BinaryOperator,
        /// The type of both operands.
        operand: CoreType,
    },
    /// A shift or rotation of one operand subtree by a literal amount.
    Shift {
        /// The shift or rotation operator.
        operator: BinaryOperator,
        /// The amount, less than the operand's word width.
        amount: u32,
    },
    /// An explicit conversion of one operand subtree to this node's type:
    /// the operand's integer value, reduced modulo 2^n when the node's type
    /// is `Word[n]`.
    Convert {
        /// The operand's type.
        from: CoreType,
    },
    /// An array of this node's type built from its element subtrees, one
    /// per element, in index order.
    Array {
        /// The number of element subtrees.
        elements: u32,
    },
    /// The element at a fixed index of one array operand subtree.
    Index {
        /// The zero-based index, less than the operand's length.
        index: u32,
    },
    /// The element of an array operand subtree at the index given by an
    /// `Int` operand subtree, which analysis proved below the length.
    Select,
    /// A copy of an array operand subtree with the element at the index of
    /// an `Int` operand subtree replaced by a third operand subtree.
    Update,
    /// An array of this node's type holding copies of one element subtree.
    Fill,
    /// The final accumulator of the function's loop at this index, whose
    /// initial value is the one operand subtree.
    Fold(u32),
    /// The current index of the enclosing loop at this index, as an `Int`.
    LoopIndex(u32),
    /// The current accumulator of the enclosing loop at this index.
    Accumulator(u32),
    /// The value of the branch of the function's conditional at this index
    /// that the one `Bool` operand subtree selects.
    Choose(u32),
}

/// Types admitted by the typed expression fragment: `Int`, `Bool`, the four
/// word types, and fixed-length arrays of them.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CoreType {
    /// An exact, signed mathematical integer.
    Int,
    /// A truth value, `true` or `false`.
    Bool,
    /// An element of the integers modulo 2^8.
    Word8,
    /// An element of the integers modulo 2^16.
    Word16,
    /// An element of the integers modulo 2^32.
    Word32,
    /// An element of the integers modulo 2^64.
    Word64,
    /// A fixed-length array of one scalar type, written `T^n`.
    Array(ArrayType),
}

impl CoreType {
    #[cfg(test)]
    const SCALARS: &'static [Self] = &[
        Self::Int,
        Self::Bool,
        Self::Word8,
        Self::Word16,
        Self::Word32,
        Self::Word64,
    ];

    /// Returns the width of a word type, or `None` for `Int`, `Bool`, and
    /// arrays.
    #[must_use]
    pub const fn word_bits(self) -> Option<u32> {
        match self {
            Self::Int | Self::Bool | Self::Array(_) => None,
            Self::Word8 => Some(8),
            Self::Word16 => Some(16),
            Self::Word32 => Some(32),
            Self::Word64 => Some(64),
        }
    }

    /// Returns the word type of exactly this width, if it is admitted.
    #[must_use]
    pub const fn word_of_width(bits: u32) -> Option<Self> {
        match bits {
            8 => Some(Self::Word8),
            16 => Some(Self::Word16),
            32 => Some(Self::Word32),
            64 => Some(Self::Word64),
            _ => None,
        }
    }

    /// Returns whether this is `Int`, `Bool`, or a word type rather than an
    /// array.
    #[must_use]
    pub const fn is_scalar(self) -> bool {
        !matches!(self, Self::Array(_))
    }

    /// Returns whether this is `Int` or a word type: a type with arithmetic.
    #[must_use]
    pub const fn is_number(self) -> bool {
        !matches!(self, Self::Bool | Self::Array(_))
    }

    /// Returns the array type, or `None` for the scalar types.
    #[must_use]
    pub const fn as_array(self) -> Option<ArrayType> {
        match self {
            Self::Array(array) => Some(array),
            Self::Int | Self::Bool | Self::Word8 | Self::Word16 | Self::Word32 | Self::Word64 => {
                None
            }
        }
    }
}

impl fmt::Display for CoreType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int => formatter.write_str("Int"),
            Self::Bool => formatter.write_str("Bool"),
            Self::Word8 => formatter.write_str("Word[8]"),
            Self::Word16 => formatter.write_str("Word[16]"),
            Self::Word32 => formatter.write_str("Word[32]"),
            Self::Word64 => formatter.write_str("Word[64]"),
            Self::Array(array) => write!(formatter, "{}^{}", array.element(), array.length()),
        }
    }
}

/// Longest admitted array type.
pub const MAX_ARRAY_LENGTH: u32 = 256;

/// A fixed-length array type `T^n`: `n` values of the scalar type `T`, for
/// `n` from 1 through [`MAX_ARRAY_LENGTH`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArrayType {
    element: Scalar,
    length: u32,
}

/// The scalar element type of an array, kept separate so that `CoreType`
/// stays a small copyable value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Scalar {
    Int,
    Bool,
    Word8,
    Word16,
    Word32,
    Word64,
}

impl ArrayType {
    /// Returns the array type of `length` elements of `element`, or `None`
    /// when `element` is an array or `length` is outside 1 through
    /// [`MAX_ARRAY_LENGTH`].
    #[must_use]
    pub const fn new(element: CoreType, length: u32) -> Option<Self> {
        let element = match element {
            CoreType::Int => Scalar::Int,
            CoreType::Bool => Scalar::Bool,
            CoreType::Word8 => Scalar::Word8,
            CoreType::Word16 => Scalar::Word16,
            CoreType::Word32 => Scalar::Word32,
            CoreType::Word64 => Scalar::Word64,
            CoreType::Array(_) => return None,
        };
        if length == 0 || length > MAX_ARRAY_LENGTH {
            return None;
        }
        Some(Self { element, length })
    }

    /// Returns the scalar element type.
    #[must_use]
    pub const fn element(self) -> CoreType {
        match self.element {
            Scalar::Int => CoreType::Int,
            Scalar::Bool => CoreType::Bool,
            Scalar::Word8 => CoreType::Word8,
            Scalar::Word16 => CoreType::Word16,
            Scalar::Word32 => CoreType::Word32,
            Scalar::Word64 => CoreType::Word64,
        }
    }

    /// Returns the number of elements.
    #[must_use]
    pub const fn length(self) -> u32 {
        self.length
    }
}

/// Values admitted by the typed expression fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreValue {
    /// An exact mathematical integer.
    Int(ExactInteger),
    /// A truth value.
    Bool(bool),
    /// An element of the integers modulo 2^8.
    Word8(u8),
    /// An element of the integers modulo 2^16.
    Word16(u16),
    /// An element of the integers modulo 2^32.
    Word32(u32),
    /// An element of the integers modulo 2^64.
    Word64(u64),
    /// A fixed-length array of scalar values.
    Array(CoreArray),
}

/// An array value: its type and exactly that many elements of its element
/// type, in index order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreArray {
    ty: ArrayType,
    elements: Vec<CoreValue>,
}

impl CoreArray {
    /// Returns the array of type `ty` holding `elements`, or `None` unless
    /// there are exactly `ty.length()` elements, each of `ty.element()`.
    #[must_use]
    pub fn new(ty: ArrayType, elements: Vec<CoreValue>) -> Option<Self> {
        let length_matches = usize::try_from(ty.length()).ok() == Some(elements.len());
        (length_matches && elements.iter().all(|element| element.ty() == ty.element()))
            .then_some(Self { ty, elements })
    }

    /// Returns the array's type.
    #[must_use]
    pub const fn ty(&self) -> ArrayType {
        self.ty
    }

    /// Returns the elements in index order.
    #[must_use]
    pub fn elements(&self) -> &[CoreValue] {
        &self.elements
    }
}

impl CoreValue {
    /// Returns this value's static Core type.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        match self {
            Self::Int(_) => CoreType::Int,
            Self::Bool(_) => CoreType::Bool,
            Self::Word8(_) => CoreType::Word8,
            Self::Word16(_) => CoreType::Word16,
            Self::Word32(_) => CoreType::Word32,
            Self::Word64(_) => CoreType::Word64,
            Self::Array(array) => CoreType::Array(array.ty),
        }
    }

    /// Returns the word of type `ty` whose value is `value` reduced modulo
    /// its width, or `None` when `ty` is not a word type.
    pub(crate) fn word_from_u64(ty: CoreType, value: u64) -> Option<Self> {
        let [b0, b1, b2, b3, b4, b5, b6, b7] = value.to_le_bytes();
        match ty {
            CoreType::Int | CoreType::Bool | CoreType::Array(_) => None,
            CoreType::Word8 => Some(Self::Word8(b0)),
            CoreType::Word16 => Some(Self::Word16(u16::from_le_bytes([b0, b1]))),
            CoreType::Word32 => Some(Self::Word32(u32::from_le_bytes([b0, b1, b2, b3]))),
            CoreType::Word64 => Some(Self::Word64(u64::from_le_bytes([
                b0, b1, b2, b3, b4, b5, b6, b7,
            ]))),
        }
    }

    /// Returns a word's value as an unsigned integer, or `None` for the
    /// other values.
    pub(crate) fn word_as_u64(&self) -> Option<u64> {
        match self {
            Self::Int(_) | Self::Bool(_) | Self::Array(_) => None,
            Self::Word8(value) => Some(u64::from(*value)),
            Self::Word16(value) => Some(u64::from(*value)),
            Self::Word32(value) => Some(u64::from(*value)),
            Self::Word64(value) => Some(*value),
        }
    }
}

impl fmt::Display for CoreValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => value.fmt(formatter),
            Self::Bool(value) => value.fmt(formatter),
            Self::Word8(value) => write!(formatter, "0x{value:02x}"),
            Self::Word16(value) => write!(formatter, "0x{value:04x}"),
            Self::Word32(value) => write!(formatter, "0x{value:08x}"),
            Self::Word64(value) => write!(formatter, "0x{value:016x}"),
            Self::Array(array) => {
                formatter.write_str("[")?;
                for (index, element) in array.elements.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    element.fmt(formatter)?;
                }
                formatter.write_str("]")
            }
        }
    }
}

/// A signed integer with an arbitrary-precision, dependency-free magnitude.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactInteger {
    negative: bool,
    magnitude: Magnitude,
}

impl ExactInteger {
    pub(crate) const fn new(negative: bool, magnitude: Magnitude) -> Self {
        Self {
            negative: negative && !magnitude.is_zero(),
            magnitude,
        }
    }

    /// Returns whether this is strictly less than zero.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns whether this is zero.
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.magnitude.is_zero()
    }

    /// Returns the number of significant magnitude bits; zero has zero bits.
    #[must_use]
    pub fn magnitude_bits(&self) -> usize {
        self.magnitude.bit_len()
    }

    /// Returns the number of base-2^32 digits in the magnitude; zero has none.
    #[must_use]
    pub fn magnitude_digits(&self) -> usize {
        self.magnitude.limbs.len()
    }

    pub(crate) fn try_clone_with_reservation(
        &self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        Some(Self {
            negative: self.negative,
            magnitude: self.magnitude.try_clone_with_reservation(reserve_limbs)?,
        })
    }

    /// Returns the nonnegative integer `value`, or `None` if storage cannot
    /// be reserved.
    pub(crate) fn from_u64(
        value: u64,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        Some(Self::new(false, Magnitude::from_u64(value, reserve_limbs)?))
    }

    /// Returns this integer when its magnitude has at most 63 bits.
    pub(crate) fn to_i64(&self) -> Option<i64> {
        if self.magnitude_bits() > 63 {
            return None;
        }
        let magnitude = i64::try_from(self.magnitude.low_u64()).ok()?;
        Some(if self.negative {
            magnitude.checked_neg()?
        } else {
            magnitude
        })
    }

    /// Returns this integer modulo 2^64, as its representative from 0
    /// through 2^64 - 1.
    pub(crate) fn modulo_2_64(&self) -> u64 {
        let low = self.magnitude.low_u64();
        if self.negative {
            low.wrapping_neg()
        } else {
            low
        }
    }

    /// Returns the exact negation.
    pub(crate) fn negated(self) -> Self {
        Self::new(!self.negative, self.magnitude)
    }

    /// Returns the exact sum, or `None` if storage cannot be reserved.
    pub(crate) fn add(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.negative == other.negative {
            let magnitude = self.magnitude.add(&other.magnitude, reserve_limbs)?;
            return Some(Self::new(self.negative, magnitude));
        }
        // Opposite signs: subtract the smaller magnitude from the larger and
        // keep the sign of the operand with the larger magnitude.
        match self.magnitude.compare(&other.magnitude) {
            Ordering::Less => {
                let magnitude = other.magnitude.subtract(&self.magnitude, reserve_limbs)?;
                Some(Self::new(other.negative, magnitude))
            }
            Ordering::Equal | Ordering::Greater => {
                let magnitude = self.magnitude.subtract(&other.magnitude, reserve_limbs)?;
                Some(Self::new(self.negative, magnitude))
            }
        }
    }

    /// Returns the exact difference, or `None` if storage cannot be reserved.
    pub(crate) fn subtract(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        let negated_other = Self {
            negative: !other.negative && !other.magnitude.is_zero(),
            magnitude: other.magnitude.try_clone_with_reservation(reserve_limbs)?,
        };
        self.add(&negated_other, reserve_limbs)
    }

    /// Returns the exact product, or `None` if storage cannot be reserved.
    pub(crate) fn multiply(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        let magnitude = self.magnitude.multiply(&other.magnitude, reserve_limbs)?;
        Some(Self::new(self.negative != other.negative, magnitude))
    }

    /// Compares two integers by their exact values.
    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => self.magnitude.compare(&other.magnitude),
            (true, true) => other.magnitude.compare(&self.magnitude),
        }
    }

    /// Returns the Euclidean quotient and remainder of `self` by a nonzero
    /// `divisor`: the unique q and r with `self = divisor * q + r` and
    /// `0 <= r < |divisor|`. Returns `None` when `divisor` is zero or
    /// storage cannot be reserved.
    pub(crate) fn divide_euclid(
        &self,
        divisor: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<(Self, Self)> {
        let (quotient, remainder) = self.magnitude.divide(&divisor.magnitude, reserve_limbs)?;
        if !self.negative || remainder.is_zero() {
            // |self| = q |d| + r, so self = d (±q) + r with the sign of q
            // making d q nonnegative when self is, and negative otherwise.
            let negative = self.negative != divisor.negative;
            return Some((Self::new(negative, quotient), Self::new(false, remainder)));
        }
        // self = -(q |d| + r) with 0 < r < |d|, so
        // self = -(q + 1) |d| + (|d| - r).
        let one = Magnitude::from_u64(1, reserve_limbs)?;
        let quotient = quotient.add(&one, reserve_limbs)?;
        let remainder = divisor.magnitude.subtract(&remainder, reserve_limbs)?;
        Some((
            Self::new(!divisor.negative, quotient),
            Self::new(false, remainder),
        ))
    }
}

impl fmt::Display for ExactInteger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decimal = self.magnitude.decimal_limbs().ok_or(fmt::Error)?;
        if self.negative {
            formatter.write_str("-")?;
        }
        decimal.write(formatter)
    }
}

/// An unsigned arbitrary-precision integer stored in little-endian binary limbs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Magnitude {
    limbs: Vec<u32>,
}

impl Magnitude {
    pub(crate) const fn zero() -> Self {
        Self { limbs: Vec::new() }
    }

    pub(crate) fn multiply_add_with_reservation(
        &mut self,
        multiplier: u32,
        addend: u32,
        reserve_limb: impl FnOnce(&mut Vec<u32>) -> bool,
    ) -> bool {
        // Determine whether the result needs another limb before modifying the
        // existing representation. A failed growth reservation must leave a
        // reusable magnitude unchanged rather than containing a partial
        // multiply-add result.
        let mut carry = u64::from(addend);
        for limb in &self.limbs {
            let Some(value) = u64::from(*limb)
                .checked_mul(u64::from(multiplier))
                .and_then(|product| product.checked_add(carry))
            else {
                // The u32 operand domain proves this unreachable. Returning
                // failure still keeps a compiler artifact from being exposed
                // if that representation invariant ever changes.
                return false;
            };
            carry = value >> u32::BITS;
        }
        if carry != 0 && !reserve_limb(&mut self.limbs) {
            return false;
        }

        carry = u64::from(addend);
        for limb in &mut self.limbs {
            let Some(value) = u64::from(*limb)
                .checked_mul(u64::from(multiplier))
                .and_then(|product| product.checked_add(carry))
            else {
                return false;
            };
            // A binary limb is the low half of this exact two-limb value.
            let Ok(low_limb) = u32::try_from(value & u64::from(u32::MAX)) else {
                return false;
            };
            *limb = low_limb;
            carry = value >> u32::BITS;
        }
        if carry != 0 {
            // The maximum product plus carry is
            // u32::MAX * (u32::MAX + 1), so its high half fits one limb.
            let Ok(carry_limb) = u32::try_from(carry) else {
                return false;
            };
            self.limbs.push(carry_limb);
        }
        self.normalize();
        true
    }

    pub(crate) const fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    fn from_u64(value: u64, reserve_limbs: fn(&mut Vec<u32>, usize) -> bool) -> Option<Self> {
        let [b0, b1, b2, b3, b4, b5, b6, b7] = value.to_le_bytes();
        let low = u32::from_le_bytes([b0, b1, b2, b3]);
        let high = u32::from_le_bytes([b4, b5, b6, b7]);
        let length = if high != 0 { 2 } else { usize::from(low != 0) };
        let mut limbs = Vec::new();
        if length != 0 && !reserve_limbs(&mut limbs, length) {
            return None;
        }
        limbs.extend([low, high].into_iter().take(length));
        Some(Self { limbs })
    }

    /// Returns the magnitude modulo 2^64.
    fn low_u64(&self) -> u64 {
        let low = self.limbs.first().copied().map_or(0, u64::from);
        let high = self.limbs.get(1).copied().map_or(0, u64::from);
        (high << u32::BITS) | low
    }

    fn compare(&self, other: &Self) -> Ordering {
        self.limbs
            .len()
            .cmp(&other.limbs.len())
            .then_with(|| self.limbs.iter().rev().cmp(other.limbs.iter().rev()))
    }

    fn add(&self, other: &Self, reserve_limbs: fn(&mut Vec<u32>, usize) -> bool) -> Option<Self> {
        let (longer, shorter) = if self.limbs.len() >= other.limbs.len() {
            (&self.limbs, &other.limbs)
        } else {
            (&other.limbs, &self.limbs)
        };
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, longer.len().checked_add(1)?) {
            return None;
        }
        let mut carry = 0_u64;
        let mut shorter_limbs = shorter.iter();
        for limb in longer {
            let addend = shorter_limbs.next().map_or(0, |value| u64::from(*value));
            // Two u32 limbs and a one-bit carry cannot overflow u64.
            let sum = u64::from(*limb).checked_add(addend)?.checked_add(carry)?;
            limbs.push(low_limb(sum)?);
            carry = sum >> u32::BITS;
        }
        if carry != 0 {
            limbs.push(low_limb(carry)?);
        }
        let mut sum = Self { limbs };
        sum.normalize();
        Some(sum)
    }

    /// Returns `self - other`; `self` must not be smaller than `other`.
    fn subtract(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.compare(other) == Ordering::Less {
            return None;
        }
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, self.limbs.len()) {
            return None;
        }
        let mut borrow = false;
        let mut other_limbs = other.limbs.iter();
        for limb in &self.limbs {
            let subtrahend = other_limbs.next().copied().unwrap_or(0);
            let (partial, first_borrow) = limb.overflowing_sub(subtrahend);
            let (difference, second_borrow) = partial.overflowing_sub(u32::from(borrow));
            limbs.push(difference);
            borrow = first_borrow || second_borrow;
        }
        if borrow {
            return None;
        }
        let mut difference = Self { limbs };
        difference.normalize();
        Some(difference)
    }

    fn multiply(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.is_zero() || other.is_zero() {
            return Some(Self::zero());
        }
        let length = self.limbs.len().checked_add(other.limbs.len())?;
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, length) {
            return None;
        }
        limbs.resize(length, 0);
        for (row, left) in self.limbs.iter().enumerate() {
            let mut carry = 0_u64;
            let mut slots = limbs.get_mut(row..)?.iter_mut();
            for right in &other.limbs {
                let slot = slots.next()?;
                // (2^32 - 1)^2 + 2 * (2^32 - 1) = 2^64 - 1, so the
                // accumulated product, slot, and carry fit in u64.
                let value = u64::from(*left)
                    .checked_mul(u64::from(*right))?
                    .checked_add(u64::from(*slot))?
                    .checked_add(carry)?;
                *slot = low_limb(value)?;
                carry = value >> u32::BITS;
            }
            // The slot after this row has not been written by earlier rows.
            *slots.next()? = low_limb(carry)?;
        }
        let mut product = Self { limbs };
        product.normalize();
        Some(product)
    }

    /// Returns the quotient and remainder of `self` by a nonzero `divisor`,
    /// or `None` when `divisor` is zero or storage cannot be reserved.
    ///
    /// This is schoolbook long division in base 2^32, Algorithm D of Knuth,
    /// The Art of Computer Programming, volume 2, section 4.3.1: each
    /// quotient digit is estimated from the leading digits, corrected at most
    /// twice before the subtraction and once after it.
    fn divide(
        &self,
        divisor: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<(Self, Self)> {
        let (&top, rest) = divisor.limbs.split_last()?;
        if self.compare(divisor) == Ordering::Less {
            return Some((
                Self::zero(),
                self.try_clone_with_reservation(reserve_limbs)?,
            ));
        }
        if rest.is_empty() {
            return self.divide_by_limb(top, reserve_limbs);
        }
        // Normalize so that the divisor's leading digit has its top bit set;
        // the dividend gains one leading digit for the shifted-out bits.
        let shift = top.leading_zeros();
        let divisor_digits =
            shifted_left(&divisor.limbs, shift, divisor.limbs.len(), reserve_limbs)?;
        let mut dividend = shifted_left(
            &self.limbs,
            shift,
            self.limbs.len().checked_add(1)?,
            reserve_limbs,
        )?;
        let length = divisor_digits.len();
        let quotient_length = self.limbs.len().checked_sub(length)?.checked_add(1)?;
        let mut quotient = Vec::new();
        if !reserve_limbs(&mut quotient, quotient_length) {
            return None;
        }
        quotient.resize(quotient_length, 0);
        let leading = u64::from(*divisor_digits.last()?);
        let second = u64::from(*divisor_digits.get(length.checked_sub(2)?)?);
        for (position, digit) in quotient.iter_mut().enumerate().rev() {
            let window = dividend.get_mut(position..=position.checked_add(length)?)?;
            *digit = divide_step(window, &divisor_digits, leading, second)?;
        }
        let mut quotient = Self { limbs: quotient };
        quotient.normalize();
        // The remainder is the low digits of the dividend, shifted back.
        dividend.truncate(length);
        let mut remainder = Self {
            limbs: shifted_right(&dividend, shift, reserve_limbs)?,
        };
        remainder.normalize();
        Some((quotient, remainder))
    }

    /// Divides by one nonzero digit.
    fn divide_by_limb(
        &self,
        divisor: u32,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<(Self, Self)> {
        let divisor = u64::from(divisor);
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, self.limbs.len()) {
            return None;
        }
        limbs.resize(self.limbs.len(), 0);
        let mut remainder = 0_u64;
        for (slot, limb) in limbs.iter_mut().zip(&self.limbs).rev() {
            // remainder < divisor < 2^32, so the dividend fits in u64.
            let dividend = (remainder << u32::BITS) | u64::from(*limb);
            *slot = u32::try_from(dividend.checked_div(divisor)?).ok()?;
            remainder = dividend.checked_rem(divisor)?;
        }
        let mut quotient = Self { limbs };
        quotient.normalize();
        Some((quotient, Self::from_u64(remainder, reserve_limbs)?))
    }

    fn try_clone_with_reservation(
        &self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.limbs.is_empty() {
            return Some(Self::zero());
        }
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, self.limbs.len()) {
            return None;
        }
        limbs.extend_from_slice(&self.limbs);
        Some(Self { limbs })
    }

    pub(crate) fn bit_len(&self) -> usize {
        self.limbs.last().map_or(0, |most_significant| {
            self.limbs
                .len()
                .checked_sub(1)
                .and_then(|full_limbs| full_limbs.checked_mul(BINARY_LIMB_BITS))
                .and_then(|full_bits| {
                    u32::BITS
                        .checked_sub(most_significant.leading_zeros())
                        .and_then(|remaining_bits| usize::try_from(remaining_bits).ok())
                        .and_then(|remaining_bits| full_bits.checked_add(remaining_bits))
                })
                .unwrap_or(usize::MAX)
        })
    }

    pub(crate) fn to_u64(&self) -> Option<u64> {
        match self.limbs.as_slice() {
            [] => Some(0),
            [low] => Some(u64::from(*low)),
            [low, high] => Some((u64::from(*high) << u32::BITS) | u64::from(*low)),
            _ => None,
        }
    }

    fn normalize(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }

    fn decimal_limbs(&self) -> Option<DecimalLimbs> {
        const DECIMAL_LIMB: u64 = 1_000_000_000;

        if self.is_zero() {
            return Some(DecimalLimbs::zero());
        }
        let mut binary = [0_u32; MAX_BINARY_LIMBS];
        binary
            .get_mut(..self.limbs.len())?
            .copy_from_slice(&self.limbs);
        let mut binary_len = self.limbs.len();
        let mut decimal = DecimalLimbs::zero();
        while binary_len != 0 {
            let mut remainder = 0_u64;
            for limb in binary.get_mut(..binary_len)?.iter_mut().rev() {
                let dividend = (remainder << u32::BITS) | u64::from(*limb);
                *limb = u32::try_from(dividend / DECIMAL_LIMB).ok()?;
                remainder = dividend % DECIMAL_LIMB;
            }
            while let Some(last_index) = binary_len.checked_sub(1) {
                if binary.get(last_index).copied() != Some(0) {
                    break;
                }
                binary_len = last_index;
            }
            decimal.push(u32::try_from(remainder).ok()?)?;
        }

        Some(decimal)
    }
}

fn low_limb(value: u64) -> Option<u32> {
    u32::try_from(value & u64::from(u32::MAX)).ok()
}

/// Returns `limbs` shifted left by `shift` bits, `shift < 32`, as exactly
/// `length` digits; the digits beyond `limbs` start as zero.
fn shifted_left(
    limbs: &[u32],
    shift: u32,
    length: usize,
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<Vec<u32>> {
    let mut shifted = Vec::new();
    if !reserve_limbs(&mut shifted, length) {
        return None;
    }
    let mut carry = 0_u32;
    for limb in limbs {
        let wide = u64::from(*limb) << shift;
        shifted.push(low_limb(wide)? | carry);
        carry = u32::try_from(wide >> u32::BITS).ok()?;
    }
    if shifted.len() < length {
        shifted.push(carry);
    } else if carry != 0 {
        return None;
    }
    shifted.resize(length, 0);
    Some(shifted)
}

/// Returns `limbs` shifted right by `shift` bits, `shift < 32`.
fn shifted_right(
    limbs: &[u32],
    shift: u32,
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<Vec<u32>> {
    let mut shifted = Vec::new();
    if !reserve_limbs(&mut shifted, limbs.len()) {
        return None;
    }
    for (position, limb) in limbs.iter().enumerate() {
        let high = limbs.get(position.saturating_add(1)).copied().unwrap_or(0);
        let wide = (u64::from(high) << u32::BITS) | u64::from(*limb);
        shifted.push(low_limb(wide >> shift)?);
    }
    Some(shifted)
}

/// Computes one quotient digit of Knuth's Algorithm D and subtracts its
/// multiple of the divisor from `window`, the dividend's digits
/// `j..=j + n` for an `n`-digit normalized divisor. `leading` and `second`
/// are the divisor's two leading digits.
fn divide_step(window: &mut [u32], divisor: &[u32], leading: u64, second: u64) -> Option<u32> {
    const BASE: u64 = 1 << u32::BITS;
    let length = divisor.len();
    let top = u64::from(*window.get(length)?);
    let next = u64::from(*window.get(length.checked_sub(1)?)?);
    let third = u64::from(*window.get(length.checked_sub(2)?)?);
    // Estimate the digit from the window's two leading digits (step D3).
    let numerator = (top << u32::BITS) | next;
    let mut estimate = numerator.checked_div(leading)?;
    let mut remainder = numerator.checked_rem(leading)?;
    while estimate >= BASE || estimate.checked_mul(second)? > ((remainder << u32::BITS) | third) {
        estimate = estimate.checked_sub(1)?;
        remainder = remainder.checked_add(leading)?;
        if remainder >= BASE {
            break;
        }
    }
    // Multiply and subtract (step D4).
    let mut carry = 0_u64;
    let mut borrow = false;
    for (slot, digit) in window.iter_mut().zip(divisor) {
        // estimate < 2^32, so the product plus a carry below 2^32 fits.
        let product = estimate
            .checked_mul(u64::from(*digit))?
            .checked_add(carry)?;
        carry = product >> u32::BITS;
        let (partial, first_borrow) = slot.overflowing_sub(low_limb(product)?);
        let (difference, second_borrow) = partial.overflowing_sub(u32::from(borrow));
        *slot = difference;
        borrow = first_borrow || second_borrow;
    }
    let last = window.get_mut(length)?;
    let (partial, first_borrow) = last.overflowing_sub(u32::try_from(carry).ok()?);
    let (difference, second_borrow) = partial.overflowing_sub(u32::from(borrow));
    *last = difference;
    if first_borrow || second_borrow {
        // The estimate was one too large: add the divisor back (step D6).
        estimate = estimate.checked_sub(1)?;
        let mut carry = 0_u64;
        for (slot, digit) in window.iter_mut().zip(divisor) {
            let sum = u64::from(*slot)
                .checked_add(u64::from(*digit))?
                .checked_add(carry)?;
            *slot = low_limb(sum)?;
            carry = sum >> u32::BITS;
        }
        let last = window.get_mut(length)?;
        *last = last.wrapping_add(u32::try_from(carry).ok()?);
    }
    u32::try_from(estimate).ok()
}

struct DecimalLimbs {
    limbs: [u32; MAX_DECIMAL_LIMBS],
    len: usize,
}

impl DecimalLimbs {
    const fn zero() -> Self {
        Self {
            limbs: [0; MAX_DECIMAL_LIMBS],
            len: 0,
        }
    }

    fn push(&mut self, limb: u32) -> Option<()> {
        let next_len = self.len.checked_add(1)?;
        *self.limbs.get_mut(self.len)? = limb;
        self.len = next_len;
        Some(())
    }

    fn write(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len == 0 {
            return formatter.write_str("0");
        }
        let (most_significant, remaining) = self
            .limbs
            .get(..self.len)
            .and_then(<[u32]>::split_last)
            .ok_or(fmt::Error)?;
        write!(formatter, "{most_significant}")?;
        for limb in remaining.iter().rev() {
            write!(formatter, "{limb:09}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{SourceMap, TextOffset};

    #[test]
    fn build_profiles_preserve_debug_assertions_and_overflow_checks() {
        const { assert!(cfg!(debug_assertions)) };
        let overflow = std::panic::catch_unwind(|| {
            std::hint::black_box(u32::MAX) + std::hint::black_box(1_u32)
        });
        assert!(overflow.is_err());
    }

    fn test_span() -> Span {
        let mut sources = SourceMap::new();
        let id = sources.add("core.or", "x").unwrap();
        sources
            .get(id)
            .unwrap()
            .span(TextOffset::new(0), TextOffset::new(1))
            .unwrap()
    }

    #[test]
    fn exact_integer_decimal_formatting_does_not_collapse_large_values() {
        let mut magnitude = Magnitude::zero();
        for digit in [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
            assert!(magnitude.multiply_add_with_reservation(10, digit, |limbs| {
                limbs.try_reserve(1).is_ok()
            }));
        }
        let value = ExactInteger::new(true, magnitude);
        assert_eq!(value.to_string(), "-1234567890123456789");
    }

    #[test]
    fn magnitude_multiply_add_accepts_the_full_u32_domain_without_overflow() {
        let mut magnitude = Magnitude::zero();
        let reserve = |limbs: &mut Vec<u32>| limbs.try_reserve(1).is_ok();

        assert!(magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, reserve));
        assert_eq!(magnitude.limbs, [u32::MAX]);

        assert!(magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, reserve));
        assert_eq!(magnitude.limbs, [0, u32::MAX]);

        assert!(magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, reserve));
        assert_eq!(magnitude.limbs, [u32::MAX, 1, u32::MAX - 1]);
    }

    #[test]
    fn magnitude_limb_reservation_failure_preserves_existing_state() {
        let mut magnitude = Magnitude {
            limbs: vec![u32::MAX],
        };
        let before = magnitude.clone();

        assert!(!magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, |_| false));
        assert_eq!(magnitude, before);
    }

    #[test]
    fn maximum_exact_integer_uses_the_fixed_decimal_scratch_bound() {
        let mut magnitude = Magnitude::zero();
        for _ in 0..MAX_EXACT_INTEGER_BITS {
            assert!(
                magnitude
                    .multiply_add_with_reservation(2, 1, |limbs| { limbs.try_reserve(1).is_ok() })
            );
        }

        let decimal = magnitude.decimal_limbs().unwrap();
        assert!(decimal.len <= MAX_DECIMAL_LIMBS);
        assert_eq!(magnitude.bit_len(), MAX_EXACT_INTEGER_BITS);
    }

    #[test]
    fn zero_decimal_formatting_uses_no_decimal_limbs() {
        let magnitude = Magnitude::zero();

        assert_eq!(magnitude.decimal_limbs().unwrap().len, 0);
    }

    #[test]
    fn oversized_internal_magnitude_returns_a_formatting_error_without_output() {
        let value = ExactInteger::new(
            true,
            Magnitude {
                limbs: vec![1; MAX_BINARY_LIMBS + 1],
            },
        );
        let mut output = String::new();

        let result = std::fmt::write(&mut output, format_args!("{value}"));

        assert_eq!(result, Err(fmt::Error));
        assert!(output.is_empty());
    }

    #[test]
    fn decimal_limb_capacity_failure_preserves_existing_state() {
        let mut decimal = DecimalLimbs::zero();
        for limb in 0..MAX_DECIMAL_LIMBS {
            assert_eq!(decimal.push(u32::try_from(limb).unwrap()), Some(()));
        }
        let before = decimal.limbs;

        assert_eq!(decimal.push(u32::MAX), None);
        assert_eq!(decimal.len, MAX_DECIMAL_LIMBS);
        assert_eq!(decimal.limbs, before);
    }

    #[test]
    fn negative_zero_is_canonical_zero() {
        let value = ExactInteger::new(true, Magnitude::zero());
        assert!(!value.is_negative());
        assert!(value.is_zero());
        assert_eq!(value.to_string(), "0");
    }

    #[test]
    fn word_display_is_fixed_width_lowercase_hexadecimal() {
        assert_eq!(CoreValue::Word8(0).to_string(), "0x00");
        assert_eq!(CoreValue::Word8(10).to_string(), "0x0a");
        assert_eq!(CoreValue::Word8(255).to_string(), "0xff");
        assert_eq!(CoreValue::Word16(0).to_string(), "0x0000");
        assert_eq!(CoreValue::Word16(0xbeef).to_string(), "0xbeef");
        assert_eq!(CoreValue::Word32(1).to_string(), "0x00000001");
        assert_eq!(CoreValue::Word32(0x6a09_e667).to_string(), "0x6a09e667");
        assert_eq!(CoreValue::Word64(0).to_string(), "0x0000000000000000");
        assert_eq!(
            CoreValue::Word64(u64::MAX).to_string(),
            "0xffffffffffffffff"
        );
    }

    #[test]
    fn core_type_inventory_and_display_are_exact() {
        assert_eq!(
            CoreType::SCALARS,
            &[
                CoreType::Int,
                CoreType::Bool,
                CoreType::Word8,
                CoreType::Word16,
                CoreType::Word32,
                CoreType::Word64,
            ]
        );
        assert_eq!(
            CoreType::SCALARS
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            ["Int", "Bool", "Word[8]", "Word[16]", "Word[32]", "Word[64]"]
        );
        for ty in CoreType::SCALARS {
            let word = !matches!(ty, CoreType::Int | CoreType::Bool);
            assert_eq!(
                ty.word_bits().and_then(CoreType::word_of_width),
                word.then_some(*ty)
            );
            assert_eq!(ty.is_number(), *ty != CoreType::Bool);
            assert!(ty.is_scalar());
            assert_eq!(ty.as_array(), None);
        }
        assert_eq!(CoreType::word_of_width(12), None);
    }

    #[test]
    fn array_types_hold_one_to_256_scalars_and_display_as_powers() {
        for element in CoreType::SCALARS {
            for length in [1, 2, 16, MAX_ARRAY_LENGTH] {
                let array = ArrayType::new(*element, length).unwrap();
                assert_eq!(array.element(), *element);
                assert_eq!(array.length(), length);
                let ty = CoreType::Array(array);
                assert!(!ty.is_scalar());
                assert_eq!(ty.as_array(), Some(array));
                assert_eq!(ty.word_bits(), None);
                assert_eq!(ty.to_string(), format!("{element}^{length}"));
            }
            assert_eq!(ArrayType::new(*element, 0), None);
            assert_eq!(ArrayType::new(*element, MAX_ARRAY_LENGTH + 1), None);
            assert_eq!(ArrayType::new(*element, u32::MAX), None);
        }
        let array = CoreType::Array(ArrayType::new(CoreType::Word32, 4).unwrap());
        assert_eq!(ArrayType::new(array, 2), None);
        assert_eq!(array.to_string(), "Word[32]^4");
    }

    #[test]
    fn array_values_require_their_exact_length_and_element_type() {
        let ty = ArrayType::new(CoreType::Word8, 2).unwrap();
        let array =
            CoreArray::new(ty, vec![CoreValue::Word8(0x0f), CoreValue::Word8(0xf0)]).unwrap();
        assert_eq!(array.ty(), ty);
        assert_eq!(array.elements().len(), 2);
        let value = CoreValue::Array(array);
        assert_eq!(value.ty(), CoreType::Array(ty));
        assert_eq!(value.to_string(), "[0x0f, 0xf0]");
        assert_eq!(value.word_as_u64(), None);
        assert_eq!(CoreValue::word_from_u64(CoreType::Array(ty), 1), None);

        assert_eq!(CoreArray::new(ty, vec![CoreValue::Word8(1)]), None);
        assert_eq!(
            CoreArray::new(ty, vec![CoreValue::Word8(1), CoreValue::Word16(1)]),
            None
        );
        let integers = ArrayType::new(CoreType::Int, 3).unwrap();
        let negative = ExactInteger::new(
            true,
            Magnitude::from_u64(7, |limbs, count| limbs.try_reserve_exact(count).is_ok()).unwrap(),
        );
        let value = CoreValue::Array(
            CoreArray::new(
                integers,
                vec![
                    CoreValue::Int(ExactInteger::new(false, Magnitude::zero())),
                    CoreValue::Int(negative),
                    CoreValue::Int(ExactInteger::new(false, Magnitude::zero())),
                ],
            )
            .unwrap(),
        );
        assert_eq!(value.to_string(), "[0, -7, 0]");
    }

    #[test]
    fn core_function_id_accepts_the_full_u32_domain() {
        assert_eq!(CoreFunctionId::from_index(0).unwrap().index(), 0);
        let maximum = usize::try_from(u32::MAX).unwrap();
        assert_eq!(
            CoreFunctionId::from_index(maximum).unwrap().index(),
            u32::MAX
        );

        #[cfg(target_pointer_width = "64")]
        assert!(CoreFunctionId::from_index(maximum.checked_add(1).unwrap()).is_none());
    }

    #[test]
    fn core_accessors_preserve_source_order_and_derive_value_types() {
        let span = test_span();
        let functions = vec![
            CoreFunction {
                id: CoreFunctionId::from_index(0).unwrap(),
                span,
                name: String::from("integer"),
                name_span: span,
                parameters: Vec::new(),
                result_type: CoreType::Int,
                locals: Vec::new(),
                body: CoreExpression {
                    nodes: vec![CoreNode {
                        span,
                        ty: CoreType::Int,
                        kind: CoreNodeKind::Literal(CoreValue::Int(ExactInteger::new(
                            false,
                            Magnitude::zero(),
                        ))),
                    }],
                },
                loops: Vec::new(),
                conditionals: Vec::new(),
            },
            CoreFunction {
                id: CoreFunctionId::from_index(1).unwrap(),
                span,
                name: String::from("word"),
                name_span: span,
                parameters: vec![CoreType::Word32],
                result_type: CoreType::Word8,
                locals: vec![CoreLocal {
                    span,
                    name: String::from("low"),
                    name_span: span,
                    ty: CoreType::Word8,
                    value: CoreExpression {
                        nodes: vec![
                            CoreNode {
                                span,
                                ty: CoreType::Word32,
                                kind: CoreNodeKind::Parameter(0),
                            },
                            CoreNode {
                                span,
                                ty: CoreType::Word8,
                                kind: CoreNodeKind::Convert {
                                    from: CoreType::Word32,
                                },
                            },
                        ],
                    },
                }],
                body: CoreExpression {
                    nodes: vec![CoreNode {
                        span,
                        ty: CoreType::Word8,
                        kind: CoreNodeKind::Literal(CoreValue::Word8(8)),
                    }],
                },
                loops: Vec::new(),
                conditionals: Vec::new(),
            },
        ];
        let module = CoreModule {
            span,
            name: String::from("values"),
            functions,
        };

        assert_eq!(module.span(), span);
        assert_eq!(module.name(), "values");
        assert_eq!(module.functions()[0].id().index(), 0);
        assert_eq!(module.functions()[0].span(), span);
        assert_eq!(module.functions()[0].name(), "integer");
        assert_eq!(module.functions()[0].name_span(), span);
        assert_eq!(module.functions()[0].result_type(), CoreType::Int);
        assert_eq!(module.functions()[0].parameters(), []);
        assert!(module.functions()[0].locals().is_empty());
        assert_eq!(
            module.functions()[0].body().literal().map(CoreValue::ty),
            Some(CoreType::Int)
        );
        assert_eq!(module.functions()[0].body().nodes().len(), 1);
        assert_eq!(module.functions()[0].body().root().unwrap().span(), span);
        assert_eq!(
            module.functions()[0].body().root().unwrap().ty(),
            CoreType::Int
        );
        assert_eq!(module.functions()[1].id().index(), 1);
        assert_eq!(module.functions()[1].result_type(), CoreType::Word8);
        assert_eq!(module.functions()[1].parameters(), [CoreType::Word32]);
        let local = &module.functions()[1].locals()[0];
        assert_eq!(local.span(), span);
        assert_eq!(local.name(), "low");
        assert_eq!(local.name_span(), span);
        assert_eq!(local.ty(), CoreType::Word8);
        assert_eq!(local.value().nodes().len(), 2);
        assert_eq!(local.value().literal(), None);
        assert_eq!(
            local.value().root().unwrap().kind(),
            &CoreNodeKind::Convert {
                from: CoreType::Word32
            }
        );
        assert_eq!(
            module.functions()[1].body().literal(),
            Some(&CoreValue::Word8(8))
        );
        assert_eq!(
            module.functions()[1].body().root().unwrap().kind(),
            &CoreNodeKind::Literal(CoreValue::Word8(8))
        );

        let CoreModule {
            span: _,
            name: _,
            functions,
        } = module;
        for function in functions {
            let CoreFunction {
                id: _,
                span: _,
                name: _,
                name_span: _,
                parameters: _,
                result_type: _,
                locals,
                body,
                loops,
                conditionals,
            } = function;
            let local_values = locals.into_iter().map(|local| {
                let CoreLocal {
                    span: _,
                    name: _,
                    name_span: _,
                    ty: _,
                    value,
                } = local;
                value
            });
            let loop_steps = loops.into_iter().map(|r#loop| {
                let CoreLoop {
                    span: _,
                    index_name: _,
                    accumulator_name: _,
                    ty: _,
                    start: _,
                    end: _,
                    visible_locals: _,
                    scope: _,
                    step,
                } = r#loop;
                step
            });
            let branches = conditionals.into_iter().flat_map(|conditional| {
                let CoreConditional {
                    span: _,
                    ty: _,
                    visible_locals: _,
                    scope: _,
                    then_branch,
                    else_branch,
                } = conditional;
                [then_branch, else_branch]
            });
            for node in local_values
                .chain(std::iter::once(body))
                .chain(loop_steps)
                .chain(branches)
                .flat_map(|expression| expression.nodes)
            {
                let CoreNode { span: _, ty, kind } = node;
                match kind {
                    CoreNodeKind::Literal(
                        CoreValue::Int(_)
                        | CoreValue::Bool(_)
                        | CoreValue::Word8(_)
                        | CoreValue::Word16(_)
                        | CoreValue::Word32(_)
                        | CoreValue::Word64(_)
                        | CoreValue::Array(_),
                    )
                    | CoreNodeKind::Parameter(_)
                    | CoreNodeKind::Local(_)
                    | CoreNodeKind::Call { .. }
                    | CoreNodeKind::Unary(_)
                    | CoreNodeKind::Binary(_)
                    | CoreNodeKind::Shift { .. }
                    | CoreNodeKind::Convert { .. }
                    | CoreNodeKind::Array { .. }
                    | CoreNodeKind::Index { .. }
                    | CoreNodeKind::Select
                    | CoreNodeKind::Update
                    | CoreNodeKind::Fill
                    | CoreNodeKind::Fold(_)
                    | CoreNodeKind::LoopIndex(_)
                    | CoreNodeKind::Accumulator(_)
                    | CoreNodeKind::Compare { .. }
                    | CoreNodeKind::Choose(_) => {}
                }
                match ty {
                    CoreType::Int
                    | CoreType::Bool
                    | CoreType::Word8
                    | CoreType::Word16
                    | CoreType::Word32
                    | CoreType::Word64
                    | CoreType::Array(_) => {}
                }
            }
        }
    }

    fn reserve(limbs: &mut Vec<u32>, count: usize) -> bool {
        limbs.try_reserve_exact(count).is_ok()
    }

    fn exact(value: i128) -> ExactInteger {
        let mut magnitude = Magnitude::zero();
        let unsigned = value.unsigned_abs();
        for shift in (0..8).rev() {
            let digit = u32::try_from((unsigned >> (16 * shift)) & 0xffff).unwrap();
            assert!(
                magnitude.multiply_add_with_reservation(1 << 16, digit, |limbs| {
                    limbs.try_reserve(1).is_ok()
                })
            );
        }
        ExactInteger::new(value < 0, magnitude)
    }

    fn exact_corpus() -> Vec<i128> {
        let mut values = vec![
            0,
            1,
            2,
            i128::from(u32::MAX) - 1,
            i128::from(u32::MAX),
            1 << 32,
            (1 << 32) + 1,
            i128::from(i64::MAX),
            (1 << 63) + 12_345,
        ];
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        for _ in 0..12 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            values.push(i128::from(state >> (state % 61)));
        }
        let negatives = values.iter().map(|value| -value).collect::<Vec<_>>();
        values.extend(negatives);
        values
    }

    #[test]
    fn exact_integer_arithmetic_matches_an_i128_reference() {
        let corpus = exact_corpus();
        for &left in &corpus {
            assert_eq!(exact(left).to_string(), left.to_string());
            assert_eq!(exact(left).negated().to_string(), (-left).to_string());
            assert_eq!(exact(left).is_negative(), left < 0);
            for &right in &corpus {
                let (a, b) = (exact(left), exact(right));
                assert_eq!(
                    a.add(&b, reserve).unwrap().to_string(),
                    (left + right).to_string(),
                    "{left} + {right}"
                );
                assert_eq!(
                    a.subtract(&b, reserve).unwrap().to_string(),
                    (left - right).to_string(),
                    "{left} - {right}"
                );
                assert_eq!(
                    a.multiply(&b, reserve).unwrap().to_string(),
                    (left * right).to_string(),
                    "{left} * {right}"
                );
                // Every zero result is the canonical, unsigned zero.
                let difference = a.subtract(&a, reserve).unwrap();
                assert!(difference.is_zero() && !difference.is_negative());
                assert_eq!(difference, exact(0));
            }
        }
    }

    #[test]
    fn exact_integers_convert_to_i64_only_within_63_bits() {
        let corpus = exact_corpus();
        for &value in &corpus {
            let expected = i64::try_from(value).ok().filter(|value| *value != i64::MIN);
            assert_eq!(exact(value).to_i64(), expected, "{value}");
        }
        for value in [0, 1, -1, i128::from(i64::MAX), -i128::from(i64::MAX)] {
            assert_eq!(exact(value).to_i64().map(i128::from), Some(value));
        }
        // The magnitude of -2^63 has 64 bits, so it is refused like 2^63.
        for value in [i128::from(i64::MIN), 1 << 63, 1 << 64] {
            assert_eq!(exact(value).to_i64(), None, "{value}");
        }
        assert_eq!(MAX_LOOP_BOUND, 1 << 16);
    }

    #[test]
    fn multi_limb_arithmetic_is_exact() {
        let power = |bits: usize| {
            let mut value = exact(1);
            for _ in 0..bits {
                value = value.add(&value, reserve).unwrap();
            }
            value
        };
        for bits in [31, 32, 33, 63, 64, 65, 127, 128, 500, 1024] {
            let two_k = power(bits);
            let below = two_k.subtract(&exact(1), reserve).unwrap();
            let above = two_k.add(&exact(1), reserve).unwrap();
            // (2^k - 1)(2^k + 1) = 2^(2k) - 1, whose magnitude has 2k bits.
            let product = below.multiply(&above, reserve).unwrap();
            assert_eq!(product.magnitude_bits(), 2 * bits);
            assert_eq!(
                product.add(&exact(1), reserve).unwrap(),
                power(2 * bits),
                "{bits}"
            );
            assert_eq!(
                above.multiply(&below, reserve).unwrap(),
                product,
                "multiplication commutes"
            );
            assert_eq!(below.subtract(&above, reserve).unwrap().to_string(), "-2");
            let negative = below.clone().negated();
            assert_eq!(
                negative.multiply(&negative, reserve).unwrap(),
                below.multiply(&below, reserve).unwrap()
            );
        }
        assert_eq!(
            power(128).to_string(),
            "340282366920938463463374607431768211456"
        );
    }

    #[test]
    fn exact_comparison_and_euclidean_division_match_an_i128_reference() {
        let corpus = exact_corpus();
        for &left in &corpus {
            for &right in &corpus {
                let (a, b) = (exact(left), exact(right));
                assert_eq!(a.compare(&b), left.cmp(&right), "{left} <=> {right}");
                if right == 0 {
                    assert_eq!(a.divide_euclid(&b, reserve), None, "{left} / 0");
                    continue;
                }
                let (quotient, remainder) = a.divide_euclid(&b, reserve).unwrap();
                assert_eq!(
                    (quotient.to_string(), remainder.to_string()),
                    (
                        left.div_euclid(right).to_string(),
                        left.rem_euclid(right).to_string()
                    ),
                    "{left} / {right}"
                );
                // Zero results are the canonical, unsigned zero.
                assert!(!quotient.is_zero() || !quotient.is_negative());
                assert!(!remainder.is_negative());
            }
        }
    }

    /// Digits that drive Algorithm D through its corner cases: an estimate
    /// of 2^32, both corrections before the subtraction, and the rare
    /// add-back after it.
    const HARD_LIMBS: [u32; 6] = [0, 1, 0x7fff_ffff, 0x8000_0000, 0xffff_fffe, 0xffff_ffff];

    fn magnitudes(max_digits: usize) -> Vec<Magnitude> {
        let mut values = Vec::new();
        for digits in 1..=max_digits.min(3) {
            let count = HARD_LIMBS.len().pow(u32::try_from(digits).unwrap());
            for mut code in 0..count {
                let mut limbs = Vec::new();
                for _ in 0..digits {
                    limbs.push(HARD_LIMBS[code % HARD_LIMBS.len()]);
                    code /= HARD_LIMBS.len();
                }
                values.push(Magnitude { limbs });
            }
        }
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for digits in 1..=max_digits {
            for _ in 0..24 {
                let mut limbs = Vec::new();
                for _ in 0..digits {
                    state ^= state << 13;
                    state ^= state >> 7;
                    state ^= state << 17;
                    limbs.push(u32::try_from(state >> 32).unwrap());
                }
                values.push(Magnitude { limbs });
            }
        }
        for value in &mut values {
            value.normalize();
        }
        values
    }

    #[test]
    fn multi_limb_euclidean_division_satisfies_its_defining_identity() {
        let dividends = magnitudes(9);
        let divisors = magnitudes(4);
        let mut checked = 0_usize;
        for dividend in &dividends {
            for divisor in &divisors {
                for (left_negative, right_negative) in
                    [(false, false), (true, false), (false, true), (true, true)]
                {
                    let a = ExactInteger::new(left_negative, dividend.clone());
                    let b = ExactInteger::new(right_negative, divisor.clone());
                    let Some((quotient, remainder)) = a.divide_euclid(&b, reserve) else {
                        assert!(b.is_zero());
                        continue;
                    };
                    // a = b q + r with 0 <= r < |b|: exactly one such q and r.
                    let product = b.multiply(&quotient, reserve).unwrap();
                    assert_eq!(product.add(&remainder, reserve).unwrap(), a, "{a} / {b}");
                    assert!(!remainder.is_negative(), "{a} % {b}");
                    let size = ExactInteger::new(false, divisor.clone());
                    assert_eq!(remainder.compare(&size), Ordering::Less, "{a} % {b}");
                    checked += 1;
                }
            }
        }
        assert!(checked > 100_000, "{checked}");
        // Two known quotients, checked against an independent computation.
        let power = |bits: usize| {
            let mut value = exact(1);
            for _ in 0..bits {
                value = value.add(&value, reserve).unwrap();
            }
            value
        };
        let big = power(521).subtract(&exact(1), reserve).unwrap();
        let prime = power(255).subtract(&exact(19), reserve).unwrap();
        let (quotient, remainder) = big.divide_euclid(&prime, reserve).unwrap();
        assert_eq!(
            (quotient.to_string(), remainder.to_string()),
            (
                String::from(
                    "118571099379011784113736688648896417641748464297615937576404566024103044751333376"
                ),
                String::from("739327")
            )
        );
        let (quotient, remainder) = big.negated().divide_euclid(&prime, reserve).unwrap();
        assert_eq!(
            (quotient.to_string(), remainder.to_string()),
            (
                String::from(
                    "-118571099379011784113736688648896417641748464297615937576404566024103044751333377"
                ),
                String::from(
                    "57896044618658097711785492504343953926634992332820282019728792003956564080622"
                )
            )
        );
    }

    #[test]
    fn exact_arithmetic_storage_failures_return_none() {
        let (a, b) = (exact(1 << 40), exact(-(1 << 70)));
        let fail = |_: &mut Vec<u32>, _: usize| false;
        assert_eq!(a.add(&b, fail), None);
        assert_eq!(a.add(&a, fail), None);
        assert_eq!(a.subtract(&b, fail), None);
        assert_eq!(a.multiply(&b, fail), None);
        // Multiplying by zero allocates nothing.
        assert_eq!(a.multiply(&exact(0), fail), Some(exact(0)));
        assert_eq!(ExactInteger::from_u64(1, fail), None);
        assert_eq!(ExactInteger::from_u64(0, fail), Some(exact(0)));
    }

    #[test]
    fn word_conversions_match_an_i128_reference() {
        let mut corpus = exact_corpus();
        corpus.extend([
            i128::from(u64::MAX),
            i128::from(u64::MAX) + 1,
            -i128::from(u64::MAX),
            -(i128::from(u64::MAX) + 1),
            i128::MAX,
            -i128::MAX,
        ]);
        for value in corpus {
            let expected = u64::try_from(value.rem_euclid(1 << 64)).unwrap();
            assert_eq!(exact(value).modulo_2_64(), expected, "{value}");
        }
        for value in [0, 1, u64::from(u32::MAX), 1 << 32, u64::MAX] {
            let converted = ExactInteger::from_u64(value, reserve).unwrap();
            assert_eq!(converted, exact(i128::from(value)));
            assert_eq!(converted.to_string(), value.to_string());
            assert_eq!(converted.modulo_2_64(), value);
        }
    }
}
