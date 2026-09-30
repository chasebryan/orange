//! Typed, source-mapped Core for the Orange 2026 semantic fragment.

use std::cmp::Ordering;
use std::fmt;
use std::sync::Arc;

use crate::parser::{BinaryOperator, ByteOrder, UnaryOperator};
use crate::source::Span;

pub(crate) const MAX_EXACT_INTEGER_BITS: usize = 16_384;
const BINARY_LIMB_BITS: usize = 32;
const MAX_BINARY_LIMBS: usize = MAX_EXACT_INTEGER_BITS.div_ceil(BINARY_LIMB_BITS);
// One base-1,000,000,000 limb carries more than 27 binary bits. Using the
// weaker 27-bit bound keeps this capacity an integer-only, auditable upper
// bound rather than relying on floating-point logarithms.
const MAX_DECIMAL_LIMBS: usize = MAX_EXACT_INTEGER_BITS.div_ceil(27);

/// A successfully analyzed Orange program: a root module linked with the
/// modules it uses.
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
    /// Exact ASCII name of the root module.
    pub(crate) name: String,
    /// Typed functions of the used modules in dependency order, each module
    /// in source order, then the root's in source order.
    pub(crate) functions: Vec<CoreFunction>,
    /// Position of the root's first function in `functions`.
    pub(crate) entry: usize,
}

impl CoreModule {
    /// Returns the full source extent of the root module's declaration.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact ASCII name of the root module.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns every typed function of the program: the used modules'
    /// functions in dependency order, then the root's. A function's position
    /// is its identity's index.
    #[must_use]
    pub fn functions(&self) -> &[CoreFunction] {
        &self.functions
    }

    /// Returns the root module's typed functions in source order.
    #[must_use]
    pub fn entry_functions(&self) -> &[CoreFunction] {
        self.functions.get(self.entry..).unwrap_or_default()
    }
}

/// A dense identity within one [`CoreModule`]: the function's position in
/// [`CoreModule::functions`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreFunctionId(u32);

impl CoreFunctionId {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    /// Returns the zero-based position in the linked program.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// One typed specification function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreFunction {
    /// Dense identity in the linked program.
    pub(crate) id: CoreFunctionId,
    /// Exact ASCII name of the module that declares the function.
    pub(crate) module: String,
    /// Full source extent of the function declaration.
    pub(crate) span: Span,
    /// Exact ASCII function name.
    pub(crate) name: String,
    /// Source extent of the function name.
    pub(crate) name_span: Span,
    /// The values of the function's sizes in this instance, in declaration
    /// order; empty for a function without sizes.
    pub(crate) sizes: Vec<u32>,
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
    /// Returns the dense identity in the linked program.
    #[must_use]
    pub const fn id(&self) -> CoreFunctionId {
        self.id
    }

    /// Returns the exact ASCII name of the module that declares the function.
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
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

    /// Returns the values of the function's sizes in this instance, in
    /// declaration order, or an empty slice for a function without sizes.
    #[must_use]
    pub fn sizes(&self) -> &[u32] {
        &self.sizes
    }

    /// Returns parameter types in declaration order.
    #[must_use]
    pub fn parameters(&self) -> &[CoreType] {
        &self.parameters
    }

    /// Returns the statically checked result type.
    #[must_use]
    pub fn result_type(&self) -> CoreType {
        self.result_type.clone()
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
/// `start` up to, but not including, `end`. When the step has `let`
/// bindings, the step expression holds each binding's value subtree in
/// source order and then the subtree of the step's value, so evaluating its
/// nodes in order leaves each binding's value in its slot before the value
/// that uses it.
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
    /// The step's `let` bindings in source order, evaluated at every step.
    pub(crate) bindings: Vec<CoreBinding>,
    /// Statically checked step: the bindings' value subtrees, then the
    /// step's value subtree.
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
    pub fn ty(&self) -> CoreType {
        self.ty.clone()
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

    /// Returns the step's `let` bindings in source order.
    #[must_use]
    pub fn bindings(&self) -> &[CoreBinding] {
        &self.bindings
    }

    /// Returns the statically checked step: the value subtree of each of
    /// its bindings, in order, and then the subtree of its value.
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
/// `Choose` node ends the enclosing conditional's `else` branch. A branch
/// with `let` bindings holds each binding's value subtree in source order
/// and then the subtree of its value, as a loop's step does.
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
    /// The `then` branch's `let` bindings in source order.
    pub(crate) then_bindings: Vec<CoreBinding>,
    /// The value when the condition is true: its bindings' value subtrees,
    /// then its value's subtree.
    pub(crate) then_branch: CoreExpression,
    /// The `else` branch's `let` bindings in source order.
    pub(crate) else_bindings: Vec<CoreBinding>,
    /// The value when the condition is false: its bindings' value subtrees,
    /// then its value's subtree.
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
    pub fn ty(&self) -> CoreType {
        self.ty.clone()
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

    /// Returns the `then` branch's `let` bindings in source order.
    #[must_use]
    pub fn then_bindings(&self) -> &[CoreBinding] {
        &self.then_bindings
    }

    /// Returns the value when the condition is true: the value subtree of
    /// each of its bindings, in order, and then the subtree of its value.
    #[must_use]
    pub const fn then_branch(&self) -> &CoreExpression {
        &self.then_branch
    }

    /// Returns the `else` branch's `let` bindings in source order.
    #[must_use]
    pub fn else_bindings(&self) -> &[CoreBinding] {
        &self.else_bindings
    }

    /// Returns the value when the condition is false: the value subtree of
    /// each of its bindings, in order, and then the subtree of its value.
    #[must_use]
    pub const fn else_branch(&self) -> &CoreExpression {
        &self.else_branch
    }
}

/// One `let` binding at the start of a loop's step or a conditional's
/// branch.
///
/// Its value is a subtree of the step or branch expression that holds it,
/// evaluated each time that step or branch is, after the bindings before it
/// and before the step's or branch's value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreBinding {
    /// Full source extent of the binding, from `let` through `;`.
    pub(crate) span: Span,
    /// Exact ASCII binding name.
    pub(crate) name: String,
    /// Source extent of the binding name.
    pub(crate) name_span: Span,
    /// Declared type of the binding.
    pub(crate) ty: CoreType,
    /// Offset in the step or branch expression just after the binding's
    /// value subtree, which begins where the previous binding's ends, or at
    /// the start for the first binding.
    pub(crate) end: u32,
}

impl CoreBinding {
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
    pub fn ty(&self) -> CoreType {
        self.ty.clone()
    }

    /// Returns the offset in the step or branch expression just after the
    /// binding's value subtree.
    #[must_use]
    pub const fn end(&self) -> u32 {
        self.end
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
    pub fn ty(&self) -> CoreType {
        self.ty.clone()
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
    pub fn ty(&self) -> CoreType {
        self.ty.clone()
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
    /// A conversion of one operand subtree in a byte order. The operand and
    /// this node's type are words or arrays of words of one total width, or
    /// one of them is `Int` or `Mod[m]`: the words stand for the number they
    /// spell in that order, which a number reduces modulo the width's power
    /// of two and a residue modulo m.
    Pack {
        /// The operand's type.
        from: CoreType,
        /// The byte order.
        order: ByteOrder,
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
    /// The value of a `let` binding of the step of an enclosing loop in
    /// this step.
    StepBinding {
        /// The loop's number within its function.
        loop_id: u32,
        /// The binding's zero-based position among the step's bindings.
        index: u32,
    },
    /// The value of a `let` binding of the branch of an enclosing
    /// conditional that is being evaluated.
    BranchBinding {
        /// The conditional's number within its function.
        conditional: u32,
        /// The binding's zero-based position among the branch's bindings.
        index: u32,
    },
    /// The value of the branch of the function's conditional at this index
    /// that the one `Bool` operand subtree selects.
    Choose(u32),
    /// A tuple of this node's type built from its element subtrees, one per
    /// element, in order.
    Tuple {
        /// The number of element subtrees.
        elements: u32,
    },
    /// The element at a fixed position of one tuple operand subtree.
    Project {
        /// The zero-based position, less than the operand's number of
        /// elements.
        index: u32,
    },
    /// The array of this node's type holding the elements of a first array
    /// operand subtree followed by those of a second, of the same element
    /// type.
    Concat,
    /// The array of this node's type, of length L, holding the elements of
    /// an array operand subtree from the index given by an `Int` start
    /// subtree up to, but not including, the index given by an `Int` end
    /// subtree. Analysis proved that the end is the start plus L and that
    /// both lie within the operand.
    Slice,
    /// A copy of an array operand subtree with its elements from the index
    /// of an `Int` start subtree up to the index of an `Int` end subtree
    /// replaced by the elements of a fourth, array operand subtree of that
    /// length.
    SliceUpdate,
}

/// Types admitted by the typed expression fragment: `Int`, `Bool`, the four
/// word types, the integers modulo a constant, fixed-length arrays of them,
/// and tuples of all of these.
///
/// Every type but a tuple is a plain value; a tuple type shares its element
/// list, so cloning any type never allocates.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    /// An element of the integers modulo m, written `Mod[m]`.
    Mod(Modulus),
    /// A fixed-length array of one scalar type, written `T^n`.
    Array(ArrayType),
    /// A tuple of scalar and array types, written `(T0, T1, ...)`.
    Tuple(TupleType),
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

    /// Returns the width of a word type, or `None` for `Int`, `Bool`,
    /// `Mod[m]`, and arrays.
    #[must_use]
    pub const fn word_bits(&self) -> Option<u32> {
        match self {
            Self::Int | Self::Bool | Self::Mod(_) | Self::Array(_) | Self::Tuple(_) => None,
            Self::Word8 => Some(8),
            Self::Word16 => Some(16),
            Self::Word32 => Some(32),
            Self::Word64 => Some(64),
        }
    }

    /// Returns the width of each word and the number of words of a word
    /// type, one word, or of an array of words, or `None` for the other
    /// types: the words a conversion in a byte order packs or unpacks.
    #[must_use]
    pub fn words(&self) -> Option<(u32, u32)> {
        match self {
            Self::Array(array) => array
                .element()
                .word_bits()
                .map(|bits| (bits, array.length())),
            _ => self.word_bits().map(|bits| (bits, 1)),
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

    /// Returns whether this is `Int`, `Bool`, a word type, or `Mod[m]` rather
    /// than an array or a tuple.
    #[must_use]
    pub const fn is_scalar(&self) -> bool {
        !matches!(self, Self::Array(_) | Self::Tuple(_))
    }

    /// Returns whether this is `Int`, a word type, or `Mod[m]`: a type with
    /// arithmetic and integer literals.
    #[must_use]
    pub const fn is_number(&self) -> bool {
        !matches!(self, Self::Bool | Self::Array(_) | Self::Tuple(_))
    }

    /// Returns whether this is `Int` or a word type: a number with an order.
    /// The integers modulo m have no order that their arithmetic respects.
    #[must_use]
    pub const fn is_ordered(&self) -> bool {
        !matches!(
            self,
            Self::Bool | Self::Mod(_) | Self::Array(_) | Self::Tuple(_)
        )
    }

    /// Returns the modulus of `Mod[m]`, or `None` for the other types.
    #[must_use]
    pub const fn modulus(&self) -> Option<Modulus> {
        match self {
            Self::Mod(modulus) => Some(*modulus),
            Self::Int
            | Self::Bool
            | Self::Word8
            | Self::Word16
            | Self::Word32
            | Self::Word64
            | Self::Array(_)
            | Self::Tuple(_) => None,
        }
    }

    /// Returns the array type, or `None` for the other types.
    #[must_use]
    pub const fn as_array(&self) -> Option<ArrayType> {
        match self {
            Self::Array(array) => Some(*array),
            Self::Int
            | Self::Bool
            | Self::Word8
            | Self::Word16
            | Self::Word32
            | Self::Word64
            | Self::Mod(_)
            | Self::Tuple(_) => None,
        }
    }

    /// Returns the tuple type, or `None` for the other types.
    #[must_use]
    pub const fn as_tuple(&self) -> Option<&TupleType> {
        match self {
            Self::Tuple(tuple) => Some(tuple),
            Self::Int
            | Self::Bool
            | Self::Word8
            | Self::Word16
            | Self::Word32
            | Self::Word64
            | Self::Mod(_)
            | Self::Array(_) => None,
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
            Self::Mod(modulus) => write!(formatter, "Mod[{modulus}]"),
            Self::Array(array) => write!(formatter, "{}^{}", array.element(), array.length()),
            Self::Tuple(tuple) => {
                formatter.write_str("(")?;
                for (index, element) in tuple.elements().iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    element.fmt(formatter)?;
                }
                formatter.write_str(")")
            }
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

/// The scalar element type of an array, kept separate so that an array's
/// element is a scalar by construction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum Scalar {
    Int,
    Bool,
    Word8,
    Word16,
    Word32,
    Word64,
    Mod(Modulus),
}

impl ArrayType {
    /// Returns the array type of `length` elements of `element`, or `None`
    /// when `element` is an array or a tuple or `length` is outside 1
    /// through [`MAX_ARRAY_LENGTH`].
    #[must_use]
    pub const fn new(element: &CoreType, length: u32) -> Option<Self> {
        let element = match element {
            CoreType::Int => Scalar::Int,
            CoreType::Bool => Scalar::Bool,
            CoreType::Word8 => Scalar::Word8,
            CoreType::Word16 => Scalar::Word16,
            CoreType::Word32 => Scalar::Word32,
            CoreType::Word64 => Scalar::Word64,
            CoreType::Mod(modulus) => Scalar::Mod(*modulus),
            CoreType::Array(_) | CoreType::Tuple(_) => return None,
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
            Scalar::Mod(modulus) => CoreType::Mod(modulus),
        }
    }

    /// Returns the number of elements.
    #[must_use]
    pub const fn length(self) -> u32 {
        self.length
    }
}

/// Most elements of a tuple type.
pub const MAX_TUPLE_ELEMENTS: u32 = 16;

/// A tuple type `(T0, T1, ...)`: from 2 through [`MAX_TUPLE_ELEMENTS`]
/// values, each of a scalar or an array type. A tuple holds no tuple.
///
/// The element list is shared, so a tuple type is cloned without
/// allocating.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TupleType {
    elements: Arc<[CoreType]>,
}

impl TupleType {
    /// Returns the tuple type of `elements`, or `None` when there are fewer
    /// than two or more than [`MAX_TUPLE_ELEMENTS`] of them or one of them is
    /// a tuple.
    ///
    /// The element types are copied once into the shared list. Stable Rust
    /// cannot report the failure of that allocation, of at most
    /// [`MAX_TUPLE_ELEMENTS`] types, so its failure aborts the process
    /// rather than returning `None`; callers reserve their own element
    /// storage fallibly before calling this.
    #[must_use]
    pub fn new(elements: &[CoreType]) -> Option<Self> {
        let count = u32::try_from(elements.len()).ok()?;
        let admitted = (2..=MAX_TUPLE_ELEMENTS).contains(&count)
            && elements
                .iter()
                .all(|element| !matches!(element, CoreType::Tuple(_)));
        admitted.then(|| Self {
            elements: Arc::from(elements),
        })
    }

    /// Returns the element types in order.
    #[must_use]
    pub fn elements(&self) -> &[CoreType] {
        &self.elements
    }

    /// Returns the type of the element at `index`, if there is one.
    #[must_use]
    pub fn element(&self, index: u32) -> Option<&CoreType> {
        self.elements.get(usize::try_from(index).ok()?)
    }

    /// Returns the number of elements, from 2 through
    /// [`MAX_TUPLE_ELEMENTS`].
    #[must_use]
    pub fn len(&self) -> u32 {
        u32::try_from(self.elements.len()).unwrap_or(u32::MAX)
    }

    /// Returns `false`: a tuple has at least two elements.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
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
    /// An element of the integers modulo m.
    Mod(Residue),
    /// A fixed-length array of scalar values.
    Array(CoreArray),
    /// A tuple of scalar and array values.
    Tuple(CoreTuple),
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

/// A tuple value: its type and exactly one value of each of its element
/// types, in order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreTuple {
    ty: TupleType,
    elements: Vec<CoreValue>,
}

impl CoreTuple {
    /// Returns the tuple of type `ty` holding `elements`, or `None` unless
    /// there is exactly one element of each of `ty`'s element types, in
    /// order.
    #[must_use]
    pub fn new(ty: TupleType, elements: Vec<CoreValue>) -> Option<Self> {
        let matches = ty.elements().len() == elements.len()
            && ty
                .elements()
                .iter()
                .zip(&elements)
                .all(|(expected, element)| element.ty() == *expected);
        matches.then_some(Self { ty, elements })
    }

    /// Returns the tuple's type.
    #[must_use]
    pub const fn ty(&self) -> &TupleType {
        &self.ty
    }

    /// Returns the elements in order.
    #[must_use]
    pub fn elements(&self) -> &[CoreValue] {
        &self.elements
    }

    /// Returns the elements, consuming the tuple.
    #[must_use]
    pub fn into_elements(self) -> Vec<CoreValue> {
        self.elements
    }
}

impl CoreValue {
    /// Returns this value's static Core type.
    #[must_use]
    pub fn ty(&self) -> CoreType {
        match self {
            Self::Int(_) => CoreType::Int,
            Self::Bool(_) => CoreType::Bool,
            Self::Word8(_) => CoreType::Word8,
            Self::Word16(_) => CoreType::Word16,
            Self::Word32(_) => CoreType::Word32,
            Self::Word64(_) => CoreType::Word64,
            Self::Mod(residue) => CoreType::Mod(residue.modulus),
            Self::Array(array) => CoreType::Array(array.ty),
            Self::Tuple(tuple) => CoreType::Tuple(tuple.ty.clone()),
        }
    }

    /// Returns the word of type `ty` whose value is `value` reduced modulo
    /// its width, or `None` when `ty` is not a word type.
    pub(crate) fn word_from_u64(ty: &CoreType, value: u64) -> Option<Self> {
        let [b0, b1, b2, b3, b4, b5, b6, b7] = value.to_le_bytes();
        match ty {
            CoreType::Int
            | CoreType::Bool
            | CoreType::Mod(_)
            | CoreType::Array(_)
            | CoreType::Tuple(_) => None,
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
            Self::Int(_) | Self::Bool(_) | Self::Mod(_) | Self::Array(_) | Self::Tuple(_) => None,
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
            Self::Mod(residue) => residue.value.fmt(formatter),
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
            Self::Tuple(tuple) => {
                formatter.write_str("(")?;
                for (index, element) in tuple.elements.iter().enumerate() {
                    if index != 0 {
                        formatter.write_str(", ")?;
                    }
                    element.fmt(formatter)?;
                }
                formatter.write_str(")")
            }
        }
    }
}

/// The most bits a modulus may have: `Mod[m]` admits 2 <= m < 2^521, so
/// every standardized prime field, that of P-521 included, has a type.
pub const MAX_MODULUS_BITS: usize = 521;
const MODULUS_LIMBS: usize = MAX_MODULUS_BITS.div_ceil(BINARY_LIMB_BITS);

/// The modulus m of the type `Mod[m]`, from 2 through 2^521 - 1.
///
/// It is kept in fixed little-endian binary limbs, so that a Core type stays
/// a small copyable value that still names its modulus exactly. Two moduli
/// are equal exactly when their values are, and they are ordered by value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Modulus {
    limbs: [u32; MODULUS_LIMBS],
}

impl Ord for Modulus {
    fn cmp(&self, other: &Self) -> Ordering {
        self.limbs.iter().rev().cmp(other.limbs.iter().rev())
    }
}

impl PartialOrd for Modulus {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Modulus {
    /// Returns the modulus `value`, or `None` unless
    /// 2 <= `value` < 2^[`MAX_MODULUS_BITS`].
    #[must_use]
    pub fn new(value: &ExactInteger) -> Option<Self> {
        if value.negative || value.magnitude_bits() > MAX_MODULUS_BITS {
            return None;
        }
        let mut limbs = [0; MODULUS_LIMBS];
        for (slot, limb) in limbs.iter_mut().zip(&value.magnitude.limbs) {
            *slot = *limb;
        }
        let modulus = Self { limbs };
        (modulus.bits() >= 2).then_some(modulus)
    }

    /// Returns the number of significant bits of m.
    #[must_use]
    pub fn bits(&self) -> usize {
        let used = self.used_limbs();
        self.limbs.get(..used).map_or(0, Magnitude::limbs_bit_len)
    }

    /// Returns the number of limbs below the most significant nonzero one,
    /// that one included.
    fn used_limbs(&self) -> usize {
        self.limbs
            .iter()
            .rposition(|limb| *limb != 0)
            .map_or(0, |position| position.saturating_add(1))
    }

    /// Returns m as an exact integer, or `None` if storage cannot be
    /// reserved.
    pub(crate) fn to_exact(
        self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<ExactInteger> {
        let used = self.limbs.get(..self.used_limbs())?;
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, used.len()) {
            return None;
        }
        limbs.extend_from_slice(used);
        Some(ExactInteger::new(false, Magnitude { limbs }))
    }

    /// Returns whether `value` is a least residue: 0 <= `value` < m.
    #[must_use]
    pub fn contains(&self, value: &ExactInteger) -> bool {
        if value.negative {
            return false;
        }
        let used = self.limbs.get(..self.used_limbs()).unwrap_or_default();
        Magnitude::compare_limbs(&value.magnitude.limbs, used) == Ordering::Less
    }

    /// Returns the least residue of `value` modulo m, or `None` if storage
    /// cannot be reserved.
    pub(crate) fn reduce(
        &self,
        value: &ExactInteger,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<ExactInteger> {
        if self.contains(value) {
            return value.try_clone_with_reservation(reserve_limbs);
        }
        let modulus = self.to_exact(reserve_limbs)?;
        Some(value.divide_euclid(&modulus, reserve_limbs)?.1)
    }

    /// Returns the least residue r with `value` * r = 1 modulo m when `value`
    /// is a unit, that is when gcd(`value`, m) = 1, and 0 otherwise, by the
    /// extended Euclidean algorithm; `value` is a least residue. Returns
    /// `None` if storage cannot be reserved.
    pub(crate) fn inverse(
        &self,
        value: &ExactInteger,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<ExactInteger> {
        let modulus = self.to_exact(reserve_limbs)?;
        let mut previous = (
            value.try_clone_with_reservation(reserve_limbs)?,
            ExactInteger::from_u64(1, reserve_limbs)?,
        );
        let mut current = (
            modulus.try_clone_with_reservation(reserve_limbs)?,
            ExactInteger::from_u64(0, reserve_limbs)?,
        );
        // Each step keeps previous.0 = previous.1 * value and
        // current.0 = current.1 * value modulo m, and the remainders shrink
        // to gcd(value, m) in at most about 1.44 * 521 steps.
        while !current.0.is_zero() {
            let (quotient, remainder) = previous.0.divide_euclid(&current.0, reserve_limbs)?;
            let coefficient = previous.1.subtract(
                &quotient.multiply(&current.1, reserve_limbs)?,
                reserve_limbs,
            )?;
            previous = std::mem::replace(&mut current, (remainder, coefficient));
        }
        let one = ExactInteger::from_u64(1, reserve_limbs)?;
        if previous.0.compare(&one) == Ordering::Equal {
            Some(previous.1.divide_euclid(&modulus, reserve_limbs)?.1)
        } else {
            ExactInteger::from_u64(0, reserve_limbs)
        }
    }

    /// Returns m when it is below 2^64.
    #[must_use]
    pub fn to_u64(&self) -> Option<u64> {
        let used = self.limbs.get(..self.used_limbs())?;
        match used {
            [] => Some(0),
            [low] => Some(u64::from(*low)),
            [low, high] => Some((u64::from(*high) << u32::BITS) | u64::from(*low)),
            _ => None,
        }
    }

    /// Returns `(true, 2^bit - m)` when m < 2^`bit` and `(false, m - 2^bit)`
    /// otherwise, when that difference is below 2^64.
    fn offset_from_power(&self, bit: usize) -> Option<(bool, u64)> {
        let mut power = [0_u32; MODULUS_LIMBS];
        let limb = bit.checked_div(BINARY_LIMB_BITS)?;
        let shift = u32::try_from(bit.checked_rem(BINARY_LIMB_BITS)?).ok()?;
        *power.get_mut(limb)? = 1_u32.checked_shl(shift)?;
        let (larger, smaller, below) = match Self::compare_raw(&power, &self.limbs) {
            Ordering::Greater => (power, self.limbs, true),
            Ordering::Equal | Ordering::Less => (self.limbs, power, false),
        };
        let mut difference = [0_u32; MODULUS_LIMBS];
        let mut borrow = 0_u64;
        for ((slot, high), low) in difference.iter_mut().zip(larger).zip(smaller) {
            let subtrahend = u64::from(low).checked_add(borrow)?;
            let (value, next) = if u64::from(high) >= subtrahend {
                (u64::from(high).checked_sub(subtrahend)?, 0)
            } else {
                (
                    u64::from(high)
                        .checked_add(1 << 32)?
                        .checked_sub(subtrahend)?,
                    1,
                )
            };
            *slot = u32::try_from(value).ok()?;
            borrow = next;
        }
        let [low, high, rest @ ..] = difference;
        rest.iter()
            .all(|limb| *limb == 0)
            .then(|| (below, (u64::from(high) << u32::BITS) | u64::from(low)))
    }

    fn compare_raw(left: &[u32; MODULUS_LIMBS], right: &[u32; MODULUS_LIMBS]) -> Ordering {
        left.iter().rev().cmp(right.iter().rev())
    }
}

/// Displays m in decimal below 2^64. A larger modulus within 2^64 of a power
/// of two displays as `(1 << k) - c`, `(1 << k) + c`, or `1 << k`, with the
/// smaller c, as the standards of prime fields write them; any other displays
/// in hexadecimal.
impl fmt::Display for Modulus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(value) = self.to_u64() {
            return write!(formatter, "{value}");
        }
        let bits = self.bits();
        let above = bits
            .checked_sub(1)
            .and_then(|bit| self.offset_from_power(bit).map(|offset| (bit, offset)));
        let below = self.offset_from_power(bits).map(|offset| (bits, offset));
        let nearest = match (below, above) {
            (Some(below), Some(above)) => Some(if above.1.1 < below.1.1 { above } else { below }),
            (below, above) => below.or(above),
        };
        match nearest {
            Some((bit, (_, 0))) => write!(formatter, "1 << {bit}"),
            Some((bit, (true, offset))) => write!(formatter, "(1 << {bit}) - {offset}"),
            Some((bit, (false, offset))) => write!(formatter, "(1 << {bit}) + {offset}"),
            None => {
                let used = self.limbs.get(..self.used_limbs()).ok_or(fmt::Error)?;
                let (most, rest) = used.split_last().ok_or(fmt::Error)?;
                write!(formatter, "0x{most:x}")?;
                for limb in rest.iter().rev() {
                    write!(formatter, "{limb:08x}")?;
                }
                Ok(())
            }
        }
    }
}

/// An element of `Mod[m]`: its modulus and its least residue r, 0 <= r < m.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Residue {
    modulus: Modulus,
    value: ExactInteger,
}

impl Residue {
    /// Returns the element of `Mod[modulus]` whose least residue is `value`,
    /// or `None` unless 0 <= `value` < `modulus`.
    #[must_use]
    pub fn new(modulus: Modulus, value: ExactInteger) -> Option<Self> {
        modulus.contains(&value).then_some(Self { modulus, value })
    }

    /// Returns the modulus.
    #[must_use]
    pub const fn modulus(&self) -> Modulus {
        self.modulus
    }

    /// Returns the least residue, from 0 through m - 1.
    #[must_use]
    pub const fn value(&self) -> &ExactInteger {
        &self.value
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

    /// Returns the magnitude's base-2^32 digits, least significant first,
    /// without leading zero digits.
    pub(crate) fn magnitude_limbs(&self) -> &[u32] {
        &self.magnitude.limbs
    }

    /// Returns the nonnegative integer whose base-2^32 digits, least
    /// significant first, are `limbs`, which may end in zero digits.
    pub(crate) fn from_limbs(limbs: Vec<u32>) -> Self {
        let mut magnitude = Magnitude { limbs };
        magnitude.normalize();
        Self::new(false, magnitude)
    }

    /// Returns the nonnegative integer `value`, or `None` if storage cannot
    /// be reserved.
    pub(crate) fn from_u64(
        value: u64,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        Some(Self::new(false, Magnitude::from_u64(value, reserve_limbs)?))
    }

    /// Returns 2^`bit`, or `None` if storage cannot be reserved.
    pub(crate) fn power_of_two(
        bit: usize,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        let limb = bit.checked_div(BINARY_LIMB_BITS)?;
        let shift = u32::try_from(bit.checked_rem(BINARY_LIMB_BITS)?).ok()?;
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, limb.checked_add(1)?) {
            return None;
        }
        limbs.resize(limb, 0);
        limbs.push(1_u32.checked_shl(shift)?);
        Some(Self::new(false, Magnitude { limbs }))
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
        Self::compare_limbs(&self.limbs, &other.limbs)
    }

    /// Compares two normalized little-endian limb sequences by value.
    fn compare_limbs(left: &[u32], right: &[u32]) -> Ordering {
        left.len()
            .cmp(&right.len())
            .then_with(|| left.iter().rev().cmp(right.iter().rev()))
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
        Self::limbs_bit_len(&self.limbs)
    }

    /// The significant bits of normalized little-endian limbs.
    fn limbs_bit_len(limbs: &[u32]) -> usize {
        limbs.last().map_or(0, |most_significant| {
            limbs
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
                word.then_some(ty.clone())
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
                let array = ArrayType::new(element, length).unwrap();
                assert_eq!(array.element(), *element);
                assert_eq!(array.length(), length);
                let ty = CoreType::Array(array);
                assert!(!ty.is_scalar());
                assert_eq!(ty.as_array(), Some(array));
                assert_eq!(ty.word_bits(), None);
                assert_eq!(ty.to_string(), format!("{element}^{length}"));
            }
            assert_eq!(ArrayType::new(element, 0), None);
            assert_eq!(ArrayType::new(element, MAX_ARRAY_LENGTH + 1), None);
            assert_eq!(ArrayType::new(element, u32::MAX), None);
        }
        let array = CoreType::Array(ArrayType::new(&CoreType::Word32, 4).unwrap());
        assert_eq!(ArrayType::new(&array, 2), None);
        assert_eq!(array.to_string(), "Word[32]^4");
    }

    #[test]
    fn array_values_require_their_exact_length_and_element_type() {
        let ty = ArrayType::new(&CoreType::Word8, 2).unwrap();
        let array =
            CoreArray::new(ty, vec![CoreValue::Word8(0x0f), CoreValue::Word8(0xf0)]).unwrap();
        assert_eq!(array.ty(), ty);
        assert_eq!(array.elements().len(), 2);
        let value = CoreValue::Array(array);
        assert_eq!(value.ty(), CoreType::Array(ty));
        assert_eq!(value.to_string(), "[0x0f, 0xf0]");
        assert_eq!(value.word_as_u64(), None);
        assert_eq!(CoreValue::word_from_u64(&CoreType::Array(ty), 1), None);

        assert_eq!(CoreArray::new(ty, vec![CoreValue::Word8(1)]), None);
        assert_eq!(
            CoreArray::new(ty, vec![CoreValue::Word8(1), CoreValue::Word16(1)]),
            None
        );
        let integers = ArrayType::new(&CoreType::Int, 3).unwrap();
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
    fn tuple_types_hold_two_through_sixteen_flat_elements() {
        let words = CoreType::Array(ArrayType::new(&CoreType::Word8, 4).unwrap());
        let pair = TupleType::new(&[CoreType::Int, words.clone()]).unwrap();
        assert_eq!(pair.elements(), [CoreType::Int, words.clone()]);
        assert_eq!(pair.len(), 2);
        assert!(!pair.is_empty());
        assert_eq!(pair.element(1), Some(&words));
        assert_eq!(pair.element(2), None);
        assert_eq!(pair.element(u32::MAX), None);
        let ty = CoreType::Tuple(pair.clone());
        assert!(!ty.is_scalar());
        assert!(!ty.is_number());
        assert_eq!(ty.as_tuple(), Some(&pair));
        assert_eq!(ty.as_array(), None);
        assert_eq!(ty.word_bits(), None);
        assert_eq!(ty.to_string(), "(Int, Word[8]^4)");
        for element in CoreType::SCALARS {
            for length in [2, 3, MAX_TUPLE_ELEMENTS] {
                let elements = vec![element.clone(); usize::try_from(length).unwrap()];
                assert_eq!(TupleType::new(&elements).unwrap().len(), length);
            }
            assert_eq!(TupleType::new(&[]), None);
            assert_eq!(TupleType::new(std::slice::from_ref(element)), None);
            let over = vec![element.clone(); usize::try_from(MAX_TUPLE_ELEMENTS + 1).unwrap()];
            assert_eq!(TupleType::new(&over), None);
        }
        // A tuple is never an element of a tuple or of an array.
        assert_eq!(TupleType::new(&[ty.clone(), CoreType::Int]), None);
        assert_eq!(TupleType::new(&[CoreType::Int, ty.clone()]), None);
        assert_eq!(ArrayType::new(&ty, 2), None);
    }

    #[test]
    fn tuple_values_hold_one_value_of_each_element_type() {
        let ty = TupleType::new(&[CoreType::Word8, CoreType::Bool]).unwrap();
        let tuple = CoreTuple::new(
            ty.clone(),
            vec![CoreValue::Word8(0x2a), CoreValue::Bool(true)],
        )
        .unwrap();
        assert_eq!(tuple.ty(), &ty);
        assert_eq!(tuple.elements().len(), 2);
        let value = CoreValue::Tuple(tuple.clone());
        assert_eq!(value.ty(), CoreType::Tuple(ty.clone()));
        assert_eq!(value.to_string(), "(0x2a, true)");
        assert_eq!(value.word_as_u64(), None);
        assert_eq!(
            CoreValue::word_from_u64(&CoreType::Tuple(ty.clone()), 1),
            None
        );
        assert_eq!(
            tuple.into_elements(),
            [CoreValue::Word8(0x2a), CoreValue::Bool(true)]
        );

        assert_eq!(CoreTuple::new(ty.clone(), vec![CoreValue::Word8(1)]), None);
        assert_eq!(
            CoreTuple::new(ty.clone(), vec![CoreValue::Bool(true), CoreValue::Word8(1)]),
            None
        );
        assert_eq!(
            CoreTuple::new(
                ty,
                vec![
                    CoreValue::Word8(1),
                    CoreValue::Bool(true),
                    CoreValue::Bool(false)
                ]
            ),
            None
        );
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
                module: String::from("helpers"),
                span,
                name: String::from("integer"),
                name_span: span,
                sizes: Vec::new(),
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
                module: String::from("values"),
                span,
                name: String::from("word"),
                name_span: span,
                sizes: Vec::new(),
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
            entry: 1,
        };

        assert_eq!(module.span(), span);
        assert_eq!(module.name(), "values");
        assert_eq!(module.functions().len(), 2);
        assert_eq!(module.entry_functions(), &module.functions()[1..]);
        assert_eq!(module.functions()[0].module(), "helpers");
        assert_eq!(module.functions()[1].module(), "values");
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
            entry: _,
        } = module;
        for function in functions {
            let CoreFunction {
                id: _,
                module: _,
                span: _,
                name: _,
                name_span: _,
                sizes: _,
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
                    bindings,
                    step,
                } = r#loop;
                bindings.into_iter().for_each(block_binding_fields);
                step
            });
            let branches = conditionals.into_iter().flat_map(|conditional| {
                let CoreConditional {
                    span: _,
                    ty: _,
                    visible_locals: _,
                    scope: _,
                    then_bindings,
                    then_branch,
                    else_bindings,
                    else_branch,
                } = conditional;
                then_bindings
                    .into_iter()
                    .chain(else_bindings)
                    .for_each(block_binding_fields);
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
                        | CoreValue::Mod(_)
                        | CoreValue::Array(_)
                        | CoreValue::Tuple(_),
                    )
                    | CoreNodeKind::Parameter(_)
                    | CoreNodeKind::Local(_)
                    | CoreNodeKind::Call { .. }
                    | CoreNodeKind::Unary(_)
                    | CoreNodeKind::Binary(_)
                    | CoreNodeKind::Shift { .. }
                    | CoreNodeKind::Convert { .. }
                    | CoreNodeKind::Pack { .. }
                    | CoreNodeKind::Array { .. }
                    | CoreNodeKind::Index { .. }
                    | CoreNodeKind::Select
                    | CoreNodeKind::Update
                    | CoreNodeKind::Fill
                    | CoreNodeKind::Fold(_)
                    | CoreNodeKind::LoopIndex(_)
                    | CoreNodeKind::Accumulator(_)
                    | CoreNodeKind::StepBinding { .. }
                    | CoreNodeKind::BranchBinding { .. }
                    | CoreNodeKind::Compare { .. }
                    | CoreNodeKind::Choose(_)
                    | CoreNodeKind::Tuple { .. }
                    | CoreNodeKind::Project { .. }
                    | CoreNodeKind::Concat
                    | CoreNodeKind::Slice
                    | CoreNodeKind::SliceUpdate => {}
                }
                match ty {
                    CoreType::Int
                    | CoreType::Bool
                    | CoreType::Word8
                    | CoreType::Word16
                    | CoreType::Word32
                    | CoreType::Word64
                    | CoreType::Mod(_)
                    | CoreType::Array(_)
                    | CoreType::Tuple(_) => {}
                }
            }
        }
    }

    /// Names every field of a block binding, so that a new field must be
    /// considered here.
    fn block_binding_fields(binding: CoreBinding) {
        let CoreBinding {
            span: _,
            name: _,
            name_span: _,
            ty: _,
            end: _,
        } = binding;
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

    /// 2^`bit` - `offset`.
    fn power_minus(bit: usize, offset: i128) -> ExactInteger {
        ExactInteger::power_of_two(bit, reserve)
            .unwrap()
            .subtract(&exact(offset), reserve)
            .unwrap()
    }

    fn modulus_of(value: &ExactInteger) -> Modulus {
        Modulus::new(value).unwrap()
    }

    /// 2^256 - 2^224 + 2^192 + 2^96 - 1, the prime of P-256.
    fn p256() -> ExactInteger {
        [(224, -1), (192, 1), (96, 1)].into_iter().fold(
            power_minus(256, 1),
            |value, (bit, sign)| {
                let power = ExactInteger::power_of_two(bit, reserve).unwrap();
                if sign < 0 {
                    value.subtract(&power, reserve).unwrap()
                } else {
                    value.add(&power, reserve).unwrap()
                }
            },
        )
    }

    #[test]
    fn moduli_range_from_two_through_two_to_the_521_minus_one() {
        for value in [-7, -1, 0, 1] {
            assert_eq!(Modulus::new(&exact(value)), None, "{value}");
        }
        assert_eq!(
            Modulus::new(&exact(2)).map(|modulus| modulus.bits()),
            Some(2)
        );
        let widest = modulus_of(&power_minus(521, 1));
        assert_eq!(widest.bits(), MAX_MODULUS_BITS);
        assert_eq!(widest.to_u64(), None);
        assert_eq!(Modulus::new(&power_minus(521, 0)), None);
        assert_eq!(Modulus::new(&power_minus(600, 1)), None);
        assert_eq!(modulus_of(&exact(3329)).to_u64(), Some(3329));

        // Moduli are equal exactly when their values are, and ordered by
        // value.
        assert_eq!(modulus_of(&exact(3329)), modulus_of(&exact(3329)));
        let mut moduli = [
            power_minus(255, 19),
            exact(3329),
            power_minus(130, 5),
            exact(2),
            power_minus(64, 0),
        ]
        .map(|value| modulus_of(&value));
        moduli.sort();
        assert_eq!(
            moduli.map(|modulus| modulus.to_string()),
            ["2", "3329", "1 << 64", "(1 << 130) - 5", "(1 << 255) - 19"]
        );
    }

    #[test]
    fn moduli_display_as_their_standards_write_them() {
        let cases = [
            (exact(3329), String::from("3329")),
            (
                exact(i128::from(u64::MAX)),
                String::from("18446744073709551615"),
            ),
            (power_minus(64, 0), String::from("1 << 64")),
            (power_minus(64, -5), String::from("(1 << 64) + 5")),
            (power_minus(127, -45), String::from("(1 << 127) + 45")),
            (power_minus(130, 5), String::from("(1 << 130) - 5")),
            (power_minus(255, 19), String::from("(1 << 255) - 19")),
            (
                power_minus(256, 4_294_968_273),
                String::from("(1 << 256) - 4294968273"),
            ),
            (power_minus(521, 1), String::from("(1 << 521) - 1")),
            (
                power_minus(200, i128::from(u64::MAX)),
                String::from("(1 << 200) - 18446744073709551615"),
            ),
            (
                power_minus(200, i128::from(u64::MAX) + 1),
                format!("0x{}{}", "f".repeat(34), "0".repeat(16)),
            ),
            (
                p256(),
                String::from("0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff"),
            ),
        ];
        for (value, display) in cases {
            assert_eq!(modulus_of(&value).to_string(), display, "{value}");
        }
    }

    #[test]
    fn residues_reduce_and_invert_exactly() {
        let seven = modulus_of(&exact(7));
        assert!(seven.contains(&exact(0)) && seven.contains(&exact(6)));
        assert!(!seven.contains(&exact(7)) && !seven.contains(&exact(-1)));
        for (value, residue) in [(-1, 6), (-7, 0), (7, 0), (-15, 6), (100, 2), (0, 0)] {
            assert_eq!(
                seven.reduce(&exact(value), reserve),
                Some(exact(residue)),
                "{value}"
            );
        }
        for (value, inverse) in [(0, 0), (1, 1), (2, 4), (3, 5), (6, 6)] {
            assert_eq!(
                seven.inverse(&exact(value), reserve),
                Some(exact(inverse)),
                "{value}"
            );
        }
        // Modulo 256 exactly the odd residues are units; the others give 0.
        let byte = modulus_of(&exact(256));
        for value in 0..256 {
            let inverse = byte
                .inverse(&exact(value), reserve)
                .unwrap()
                .to_i64()
                .unwrap();
            if value % 2 == 1 {
                assert_eq!((i128::from(inverse) * value) % 256, 1, "{value}");
            } else {
                assert_eq!(inverse, 0, "{value}");
            }
        }
        // RFC 8032's d = -121665 / 121666 modulo 2^255 - 19.
        let p = power_minus(255, 19);
        let field = modulus_of(&p);
        let inverse = field.inverse(&exact(121_666), reserve).unwrap();
        let d = field
            .reduce(
                &exact(-121_665).multiply(&inverse, reserve).unwrap(),
                reserve,
            )
            .unwrap();
        assert_eq!(
            d.to_string(),
            "37095705934669439343138083508754565189542113879843219016388785533085940283555"
        );
        assert!(Residue::new(field, d).is_some());
        assert!(Residue::new(field, p).is_none());
        assert!(Residue::new(seven, exact(-1)).is_none());
        let residue = Residue::new(seven, exact(3)).unwrap();
        assert_eq!((residue.modulus(), residue.value()), (seven, &exact(3)));
        assert_eq!(field.to_exact(|_, _| false), None);
        assert_eq!(field.inverse(&exact(2), |_, _| false), None);
        assert_eq!(field.reduce(&exact(-2), |_, _| false), None);
    }

    #[test]
    fn residue_types_and_values_display_their_modulus_and_least_residue() {
        let field = CoreType::Mod(modulus_of(&power_minus(255, 19)));
        assert_eq!(field.to_string(), "Mod[(1 << 255) - 19]");
        assert!(field.is_scalar() && field.is_number() && !field.is_ordered());
        assert_eq!(field.word_bits(), None);
        assert_eq!(field.modulus(), Some(modulus_of(&power_minus(255, 19))));
        assert_eq!(CoreType::Int.modulus(), None);
        let array = ArrayType::new(&field, 4).unwrap();
        assert_eq!(array.element(), field);
        assert_eq!(CoreType::Array(array).to_string(), "Mod[(1 << 255) - 19]^4");
        let seven = modulus_of(&exact(7));
        let value = CoreValue::Mod(Residue::new(seven, exact(6)).unwrap());
        assert_eq!(value.to_string(), "6");
        assert_eq!(value.ty(), CoreType::Mod(seven));
        assert_eq!(value.word_as_u64(), None);
        assert_eq!(CoreValue::word_from_u64(&CoreType::Mod(seven), 6), None);
    }

    #[test]
    fn block_bindings_and_their_reads_keep_their_positions() {
        let span = test_span();
        let binding = |name: &str, ty: CoreType, end: u32| CoreBinding {
            span,
            name: String::from(name),
            name_span: span,
            ty,
            end,
        };
        let node = |ty: CoreType, kind: CoreNodeKind| CoreNode { span, ty, kind };
        // A step `{ let t: Word[8] = s; let u: Word[8] = t; u }`: each
        // binding's value subtree ends where its `end` says, and the value
        // reads the second binding.
        let r#loop = CoreLoop {
            span,
            index_name: String::from("i"),
            accumulator_name: String::from("s"),
            ty: CoreType::Word8,
            start: 0,
            end: 2,
            visible_locals: 0,
            scope: vec![0],
            bindings: vec![
                binding("t", CoreType::Word8, 1),
                binding("u", CoreType::Word8, 2),
            ],
            step: CoreExpression {
                nodes: vec![
                    node(CoreType::Word8, CoreNodeKind::Accumulator(0)),
                    node(
                        CoreType::Word8,
                        CoreNodeKind::StepBinding {
                            loop_id: 0,
                            index: 0,
                        },
                    ),
                    node(
                        CoreType::Word8,
                        CoreNodeKind::StepBinding {
                            loop_id: 0,
                            index: 1,
                        },
                    ),
                ],
            },
        };
        let names = r#loop
            .bindings()
            .iter()
            .map(|binding| {
                (
                    binding.name(),
                    binding.ty(),
                    binding.end(),
                    binding.span() == span && binding.name_span() == span,
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            names,
            [
                ("t", CoreType::Word8, 1, true),
                ("u", CoreType::Word8, 2, true),
            ]
        );
        assert_eq!(
            r#loop.step().root().map(CoreNode::kind),
            Some(&CoreNodeKind::StepBinding {
                loop_id: 0,
                index: 1
            })
        );

        // A conditional keeps each branch's bindings apart.
        let conditional = CoreConditional {
            span,
            ty: CoreType::Int,
            visible_locals: 0,
            scope: Vec::new(),
            then_bindings: vec![binding("a", CoreType::Int, 1)],
            then_branch: CoreExpression {
                nodes: vec![
                    node(CoreType::Int, CoreNodeKind::Parameter(0)),
                    node(
                        CoreType::Int,
                        CoreNodeKind::BranchBinding {
                            conditional: 0,
                            index: 0,
                        },
                    ),
                ],
            },
            else_bindings: Vec::new(),
            else_branch: CoreExpression {
                nodes: vec![node(CoreType::Int, CoreNodeKind::Parameter(0))],
            },
        };
        assert_eq!(conditional.then_bindings().len(), 1);
        assert_eq!(conditional.then_bindings()[0].name(), "a");
        assert!(conditional.else_bindings().is_empty());
        assert_eq!(
            conditional.then_branch().root().map(CoreNode::kind),
            Some(&CoreNodeKind::BranchBinding {
                conditional: 0,
                index: 0
            })
        );
    }
}
