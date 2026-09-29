//! Bounded name resolution, type checking, and Core construction.

use std::cmp::Ordering;
use std::fmt;

use crate::core::{
    ArrayType, CoreConditional, CoreExpression, CoreFunction, CoreFunctionId, CoreLocal, CoreLoop,
    CoreModule, CoreNode, CoreNodeKind, CoreType, CoreValue, ExactInteger, MAX_ARRAY_LENGTH,
    MAX_EXACT_INTEGER_BITS, MAX_LOOP_BOUND, Magnitude,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::{
    ArrayExpression, BinaryExpression, BinaryOperator, Binding, CallExpression,
    ConditionalExpression, ConversionExpression, Expression, ExpressionKind, FillExpression,
    FunctionBody, FunctionDeclaration, FunctionKind, Identifier, IndexExpression, IntegerLiteral,
    LoopExpression, MAX_ARRAY_ELEMENTS, Parameter, SyntaxTree, TypeSyntax, TypedBody,
    UnaryExpression, UnaryOperator, UpdateExpression,
};
use crate::source::{SourceFile, Span};

/// Maximum ordinary semantic errors retained before one suppression diagnostic.
pub const MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE: usize = 100;
const MAX_RETAINED_SEMANTIC_DIAGNOSTICS: usize =
    MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE.saturating_add(2);

/// Maximum Typed Reference Core nodes constructed for one source.
pub const MAX_CORE_NODES_PER_SOURCE: usize = 262_144;

/// Maximum semantic events performed for one source.
pub const MAX_SEMANTIC_EVENTS_PER_SOURCE: usize = 1_048_576;

/// Maximum significant bits retained for one exact mathematical integer.
pub const MAX_INTEGER_BITS: usize = 16_384;
const _: () = assert!(MAX_INTEGER_BITS == MAX_EXACT_INTEGER_BITS);
// An array literal can spell every admitted array type and no longer one.
const _: () = assert!(MAX_ARRAY_ELEMENTS == 256 && MAX_ARRAY_LENGTH == 256);

const MAX_IDENTIFIER_BYTES_IN_DIAGNOSTIC: usize = 64;
const MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC: usize = 8;
const ADMITTED_TYPES: &str = "`Int`, `Bool`, `Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`";
const ARRAY_OPERATOR_NOTE: &str =
    "operators apply to `Int`, `Bool`, and word values; apply them to elements, such as `x[0]`";
const STATIC_INDEX_NOTE: &str = "every index is proved in range when the program is checked: a \
     word index ranges over its type, and an `Int` index is built from integer literals, loop \
     indices, and words converted with `as Int`, using `+`, `-`, `*`, `/`, `%`, and conditionals";
const BOOL_OPERATOR_NOTE: &str = "the operators on `Bool` are `!`, `&&`, `||`, `==`, and `!=`";

/// The complete result of semantic analysis.
///
/// ```compile_fail
/// use orange_compiler::AnalysisResult;
///
/// fn replace_core(result: &mut AnalysisResult) {
///     result.core = None;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisResult {
    /// Typed Core, present only when semantic analysis produced no diagnostics.
    core: Option<CoreModule>,
    /// Semantic and semantic-resource diagnostics in deterministic order.
    diagnostics: Vec<Diagnostic>,
}

impl AnalysisResult {
    /// Returns the complete typed Core module, or `None` after analysis failure.
    #[must_use]
    pub const fn core(&self) -> Option<&CoreModule> {
        self.core.as_ref()
    }

    /// Returns semantic diagnostics in deterministic order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes this result and returns its complete typed Core module, if produced.
    #[must_use]
    pub fn into_core(self) -> Option<CoreModule> {
        self.core
    }

    /// Returns whether semantic analysis did not produce a complete Core module.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.core.is_none()
    }
}

/// Resolves and checks one successfully parsed Orange syntax tree.
///
/// Empty functions participate in namespace checking but do not enter Core.
/// Typed `spec` functions are checked in source order against the signatures
/// of every typed `spec` in the module, and the call graph among them must be
/// acyclic. Within a body, each `let` binding is checked in source order
/// before the result expression.
#[must_use]
pub fn analyze(source: &SourceFile, ast: &SyntaxTree) -> AnalysisResult {
    if !syntax_tree_belongs_to_source(source, ast) {
        return invalid_semantic_input(source, |diagnostics| {
            diagnostics.try_reserve_exact(1).is_ok()
        });
    }
    Analyzer::new(source, ast, Limits::DEFAULT).run()
}

fn syntax_tree_belongs_to_source(source: &SourceFile, ast: &SyntaxTree) -> bool {
    let source_id = source.id();
    let belongs = |span: Span| span.source() == source_id;
    let type_belongs = |ty: &TypeSyntax| {
        belongs(ty.span)
            && belongs(ty.name.span)
            && ty.width_span.is_none_or(belongs)
            && ty.length_span.is_none_or(belongs)
    };

    belongs(ast.span)
        && belongs(ast.edition.span)
        && belongs(ast.edition.value_span)
        && belongs(ast.module.span)
        && belongs(ast.module.name.span)
        && ast.module.functions.iter().all(|function| {
            belongs(function.span)
                && belongs(function.name.span)
                && function.parameters.iter().all(|parameter| {
                    belongs(parameter.span)
                        && belongs(parameter.name.span)
                        && type_belongs(&parameter.ty)
                })
                && match &function.body {
                    FunctionBody::Empty => true,
                    FunctionBody::Typed(body) => {
                        belongs(body.span)
                            && type_belongs(&body.result_type)
                            && body.bindings.iter().all(|binding| {
                                belongs(binding.span)
                                    && belongs(binding.name.span)
                                    && type_belongs(&binding.ty)
                                    && expression_belongs(&binding.value, &belongs)
                            })
                            && expression_belongs(&body.expression, &belongs)
                    }
                }
        })
}

/// Parser-established expression height bounds this recursion.
fn expression_belongs(expression: &Expression, belongs: &impl Fn(Span) -> bool) -> bool {
    belongs(expression.span)
        && match &expression.kind {
            ExpressionKind::Literal(literal) => {
                belongs(literal.span) && belongs(literal.magnitude_span)
            }
            ExpressionKind::Name(name) => belongs(name.span),
            ExpressionKind::Call(call) => {
                belongs(call.callee.span)
                    && call
                        .arguments
                        .iter()
                        .all(|argument| expression_belongs(argument, belongs))
            }
            ExpressionKind::Unary(unary) => {
                belongs(unary.operator_span) && expression_belongs(&unary.operand, belongs)
            }
            ExpressionKind::Binary(binary) => {
                belongs(binary.operator_span)
                    && expression_belongs(&binary.left, belongs)
                    && expression_belongs(&binary.right, belongs)
            }
            ExpressionKind::Parenthesized(inner) => expression_belongs(inner, belongs),
            ExpressionKind::Conversion(conversion) => {
                let target = &conversion.target;
                belongs(conversion.keyword_span)
                    && belongs(target.span)
                    && belongs(target.name.span)
                    && target.width_span.is_none_or(belongs)
                    && target.length_span.is_none_or(belongs)
                    && expression_belongs(&conversion.operand, belongs)
            }
            ExpressionKind::Array(array) => array
                .elements
                .iter()
                .all(|element| expression_belongs(element, belongs)),
            ExpressionKind::Fill(fill) => {
                belongs(fill.length_span) && expression_belongs(&fill.element, belongs)
            }
            ExpressionKind::Index(index) => {
                expression_belongs(&index.index, belongs)
                    && expression_belongs(&index.base, belongs)
            }
            ExpressionKind::Update(update) => {
                belongs(update.keyword_span)
                    && expression_belongs(&update.base, belongs)
                    && expression_belongs(&update.index, belongs)
                    && expression_belongs(&update.value, belongs)
            }
            ExpressionKind::Loop(r#loop) => {
                let ty = &r#loop.ty;
                belongs(r#loop.keyword_span)
                    && belongs(r#loop.index.span)
                    && belongs(r#loop.start_span)
                    && belongs(r#loop.end_span)
                    && belongs(r#loop.accumulator.span)
                    && belongs(ty.span)
                    && belongs(ty.name.span)
                    && ty.width_span.is_none_or(belongs)
                    && ty.length_span.is_none_or(belongs)
                    && expression_belongs(&r#loop.init, belongs)
                    && expression_belongs(&r#loop.step, belongs)
            }
            ExpressionKind::Conditional(conditional) => {
                belongs(conditional.else_span)
                    && conditional.arms.iter().all(|arm| {
                        belongs(arm.keyword_span)
                            && expression_belongs(&arm.condition, belongs)
                            && expression_belongs(&arm.value, belongs)
                    })
                    && expression_belongs(&conditional.otherwise, belongs)
            }
        }
}

fn invalid_semantic_input(
    source: &SourceFile,
    reserve_diagnostic: impl FnOnce(&mut Vec<Diagnostic>) -> bool,
) -> AnalysisResult {
    let mut diagnostics = Vec::new();
    if reserve_diagnostic(&mut diagnostics) {
        diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::InvalidSemanticInput,
                "semantic analysis received a syntax tree owned by another source",
                source.lexer_span(0, 0),
            )
            .with_label("analysis stopped at this source boundary")
            .with_note("parse and analyze each syntax tree with the same source file"),
        );
    }
    AnalysisResult {
        core: None,
        diagnostics,
    }
}

#[derive(Clone, Copy)]
struct Limits {
    diagnostics: usize,
    nodes: usize,
    events: usize,
    integer_bits: usize,
}

impl Limits {
    const DEFAULT: Self = Self {
        diagnostics: MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE,
        nodes: MAX_CORE_NODES_PER_SOURCE,
        events: MAX_SEMANTIC_EVENTS_PER_SOURCE,
        integer_bits: MAX_INTEGER_BITS,
    };
}

struct Analyzer<'source, 'ast> {
    source: &'source SourceFile,
    ast: &'ast SyntaxTree,
    diagnostics: Vec<Diagnostic>,
    ordinary_diagnostics: usize,
    diagnostic_limit_reported: bool,
    resource_limit_reported: bool,
    core_nodes: usize,
    events: usize,
    halted: bool,
    limits: Limits,
    reserve_pending_function_slot: fn(&mut Vec<PendingFunction>) -> bool,
    reserve_magnitude_limb: fn(&mut Vec<u32>) -> bool,
    reserve_range_limbs: fn(&mut Vec<u32>, usize) -> bool,
    reserve_core_name: fn(&mut String, usize) -> bool,
    reserve_diagnostic_slots: fn(&mut Vec<Diagnostic>, usize) -> bool,
    reserve_core_node_slot: fn(&mut Vec<CoreNode>) -> bool,
    reserve_call_edge_slot: fn(&mut Vec<CallEdge>) -> bool,
}

struct PendingFunction {
    span: Span,
    name: String,
    name_span: Span,
    parameters: Vec<CoreType>,
    result_type: CoreType,
    locals: Vec<CoreLocal>,
    nodes: Vec<CoreNode>,
    loops: Vec<CoreLoop>,
    conditionals: Vec<CoreConditional>,
}

/// The silently resolved signature of one typed `spec`, used to check calls
/// to it from any function in the module.
struct Signature {
    id: CoreFunctionId,
    parameters: Vec<Option<CoreType>>,
    result_type: Option<CoreType>,
}

impl Signature {
    fn is_complete(&self) -> bool {
        self.result_type.is_some() && self.parameters.iter().all(Option::is_some)
    }
}

/// One examined call from a typed `spec` to a typed `spec`.
struct CallEdge {
    caller: CoreFunctionId,
    callee: CoreFunctionId,
    span: Span,
}

/// Module-wide name and signature tables shared by every body check.
struct ModuleScope<'scope, 'ast> {
    declarations: &'scope DeclarationIndex<'ast>,
    signatures: &'scope [Option<Signature>],
}

/// Core nodes of the body being checked and the module's checked calls.
struct BodyOutput<'edges> {
    nodes: Vec<CoreNode>,
    call_edges: &'edges mut Vec<CallEdge>,
}

/// The function whose body is being checked.
struct BodyContext<'ast> {
    id: CoreFunctionId,
    name: &'ast Identifier,
    parameters: &'ast [Parameter],
    parameter_types: Vec<Option<CoreType>>,
    /// Every binding of the body, including those not yet in scope.
    bindings: &'ast [Binding],
    /// Types of the bindings in scope: exactly the first `binding_types.len()`.
    binding_types: Vec<Option<CoreType>>,
    /// Loops whose step is being checked, outermost first.
    loop_scopes: Vec<LoopScope<'ast>>,
    /// Checked loops of the function, by number. A loop's number is taken
    /// when its check starts, and its entry is filled once it is well formed.
    loops: Vec<Option<CoreLoop>>,
    /// Checked conditionals of the function, by number, taken and filled as
    /// loops' are.
    conditionals: Vec<Option<CoreConditional>>,
}

/// A loop whose index and accumulator are in scope.
struct LoopScope<'ast> {
    id: u32,
    index: &'ast Identifier,
    accumulator: &'ast Identifier,
    ty: CoreType,
    start: u32,
    end: u32,
}

/// A binding whose type resolved.
struct CheckedBinding {
    ty: CoreType,
    /// The value's Core nodes, present only when the binding is well formed.
    nodes: Option<Vec<CoreNode>>,
}

/// What a bare name refers to at one point of a body.
enum NameResolution<'ast> {
    Parameter(usize),
    Binding(usize),
    /// The index of the loop scope at this position.
    LoopIndex(usize),
    /// The accumulator of the loop scope at this position.
    Accumulator(usize),
    /// The `Bool` literal `true` or `false`, where no name of its spelling
    /// is in scope.
    BoolLiteral(bool),
    /// A binding of this body whose scope has not started.
    LaterBinding(&'ast Binding),
    Unknown,
}

impl<'ast> BodyContext<'ast> {
    /// Resolves a bare name: parameters first, then the bindings in scope,
    /// each list searched in source order.
    fn resolve(&self, name: &str) -> NameResolution<'ast> {
        if let Some(index) = self
            .parameters
            .iter()
            .position(|parameter| parameter.name.text == name)
        {
            return NameResolution::Parameter(index);
        }
        let (visible, later) = self
            .bindings
            .split_at_checked(self.binding_types.len())
            .unwrap_or((self.bindings, &[]));
        if let Some(index) = visible.iter().position(|binding| binding.name.text == name) {
            return NameResolution::Binding(index);
        }
        for (position, scope) in self.loop_scopes.iter().enumerate().rev() {
            if scope.index.text == name {
                return NameResolution::LoopIndex(position);
            }
            if scope.accumulator.text == name {
                return NameResolution::Accumulator(position);
            }
        }
        match name {
            "true" => return NameResolution::BoolLiteral(true),
            "false" => return NameResolution::BoolLiteral(false),
            _ => {}
        }
        later
            .iter()
            .find(|binding| binding.name.text == name)
            .map_or(NameResolution::Unknown, NameResolution::LaterBinding)
    }

    /// Returns the type of a name in scope without reporting.
    fn name_type(&self, name: &str) -> Option<CoreType> {
        match self.resolve(name) {
            NameResolution::Parameter(index) => self.parameter_types.get(index).copied().flatten(),
            NameResolution::Binding(index) => self.binding_types.get(index).copied().flatten(),
            NameResolution::LoopIndex(_) => Some(CoreType::Int),
            NameResolution::Accumulator(position) => {
                self.loop_scopes.get(position).map(|scope| scope.ty)
            }
            NameResolution::BoolLiteral(_) => Some(CoreType::Bool),
            NameResolution::LaterBinding(_) | NameResolution::Unknown => None,
        }
    }

    /// Returns the earlier declaration of `name` among the parameters, the
    /// bindings in scope, and the loop names in scope, if any.
    fn earlier_name(&self, name: &str) -> Option<(Span, &'static str)> {
        if let Some(parameter) = self
            .parameters
            .iter()
            .find(|parameter| parameter.name.text == name)
        {
            return Some((parameter.name.span, "the parameter is here"));
        }
        if let Some(binding) = self
            .bindings
            .get(..self.binding_types.len())
            .and_then(|visible| visible.iter().find(|binding| binding.name.text == name))
        {
            return Some((binding.name.span, "the binding is here"));
        }
        self.loop_scopes.iter().find_map(|scope| {
            if scope.index.text == name {
                Some((scope.index.span, "the loop index is here"))
            } else if scope.accumulator.text == name {
                Some((scope.accumulator.span, "the accumulator is here"))
            } else {
                None
            }
        })
    }
}

/// Returns the first name, call, conversion, index, array literal, or
/// comparison of `expression`, from left to right, outside call arguments,
/// shift amounts, and the conditions of conditionals.
///
/// Every other operator gives its result the type of its operands, a shift
/// or rotation amount is a literal, and a conditional has the type of its
/// values, so this leaf's type is the type of the whole expression. A
/// comparison is a leaf of type `Bool`. Integer literals take their type from
/// their context and are skipped. An array literal ends the search so that a
/// conversion can reject it. Parser-established expression height bounds this
/// recursion.
fn first_typed_leaf(expression: &Expression) -> Option<&Expression> {
    match &expression.kind {
        ExpressionKind::Literal(_) => None,
        ExpressionKind::Name(_)
        | ExpressionKind::Call(_)
        | ExpressionKind::Conversion(_)
        | ExpressionKind::Index(_)
        | ExpressionKind::Array(_)
        | ExpressionKind::Fill(_)
        | ExpressionKind::Loop(_) => Some(expression),
        ExpressionKind::Parenthesized(inner) => first_typed_leaf(inner),
        // An update has the type of the array it updates.
        ExpressionKind::Update(update) => first_typed_leaf(&update.base),
        ExpressionKind::Unary(unary) => first_typed_leaf(&unary.operand),
        ExpressionKind::Binary(binary) if binary.operator.is_comparison() => Some(expression),
        ExpressionKind::Binary(binary) => {
            let left = first_typed_leaf(&binary.left);
            if binary.operator.is_shift_or_rotation() {
                left
            } else {
                left.or_else(|| first_typed_leaf(&binary.right))
            }
        }
        ExpressionKind::Conditional(conditional) => conditional
            .arms
            .iter()
            .find_map(|arm| first_typed_leaf(&arm.value))
            .or_else(|| first_typed_leaf(&conditional.otherwise)),
    }
}

struct DeclarationEntry<'ast> {
    kind: FunctionKind,
    name: &'ast str,
    span: Span,
    source_index: usize,
}

type DeclarationIndex<'ast> = Vec<DeclarationEntry<'ast>>;

fn first_declaration<'index, 'ast>(
    declarations: &'index DeclarationIndex<'ast>,
    kind: FunctionKind,
    name: &str,
) -> Option<&'index DeclarationEntry<'ast>> {
    let index = declarations.partition_point(|entry| {
        entry.kind.cmp(&kind).then_with(|| entry.name.cmp(name)) == Ordering::Less
    });
    declarations
        .get(index)
        .filter(|entry| entry.kind == kind && entry.name == name)
}

fn reserve_pending_function_slot(functions: &mut Vec<PendingFunction>) -> bool {
    functions.try_reserve(1).is_ok()
}

fn reserve_magnitude_limb(limbs: &mut Vec<u32>) -> bool {
    limbs.try_reserve(1).is_ok()
}

fn reserve_core_name(name: &mut String, bytes: usize) -> bool {
    name.try_reserve_exact(bytes).is_ok()
}

fn reserve_diagnostic_slots(diagnostics: &mut Vec<Diagnostic>, capacity: usize) -> bool {
    diagnostics.try_reserve_exact(capacity).is_ok()
}

fn reserve_core_node_slot(nodes: &mut Vec<CoreNode>) -> bool {
    nodes.try_reserve(1).is_ok()
}

fn reserve_call_edge_slot(edges: &mut Vec<CallEdge>) -> bool {
    edges.try_reserve(1).is_ok()
}

/// The outcome of classifying a parsed type without reporting.
enum TypeClass {
    Resolved(CoreType),
    MissingWordWidth,
    UnsupportedWordWidth(Span),
    UnsupportedArrayLength(Span),
    Unsupported,
}

fn classify_type(source: &SourceFile, syntax: &TypeSyntax) -> TypeClass {
    let scalar = classify_scalar_type(source, syntax);
    match (scalar, syntax.length_span) {
        (TypeClass::Resolved(element), Some(length_span)) => array_length(source, length_span)
            .and_then(|length| ArrayType::new(element, length))
            .map_or(TypeClass::UnsupportedArrayLength(length_span), |array| {
                TypeClass::Resolved(CoreType::Array(array))
            }),
        (scalar, _) => scalar,
    }
}

/// Decodes an array length written as a decimal integer with no leading
/// zero and no underscore, as word widths are written.
fn array_length(source: &SourceFile, span: Span) -> Option<u32> {
    let spelling = source.slice(span)?;
    let canonical = !spelling.is_empty()
        && !spelling.starts_with('0')
        && spelling.bytes().all(|byte| byte.is_ascii_digit());
    if !canonical || spelling.len() > 3 {
        return None;
    }
    spelling.parse().ok()
}

fn classify_scalar_type(source: &SourceFile, syntax: &TypeSyntax) -> TypeClass {
    match (syntax.name.text.as_str(), syntax.width_span) {
        ("Int", None) => TypeClass::Resolved(CoreType::Int),
        ("Bool", None) => TypeClass::Resolved(CoreType::Bool),
        ("Word", Some(width_span)) => {
            let width = match source.slice(width_span) {
                Some("8") => Some(8),
                Some("16") => Some(16),
                Some("32") => Some(32),
                Some("64") => Some(64),
                _ => None,
            };
            width
                .and_then(CoreType::word_of_width)
                .map_or(TypeClass::UnsupportedWordWidth(width_span), |ty| {
                    TypeClass::Resolved(ty)
                })
        }
        ("Word", None) => TypeClass::MissingWordWidth,
        _ => TypeClass::Unsupported,
    }
}

fn silent_type(source: &SourceFile, syntax: &TypeSyntax) -> Option<CoreType> {
    match classify_type(source, syntax) {
        TypeClass::Resolved(ty) => Some(ty),
        _ => None,
    }
}

/// The least and greatest values an index can take, as exact integers.
type IndexRange = (ExactInteger, ExactInteger);

/// The least and greatest values of a word expression. Bounds are kept in
/// `u128` so that no bound computation of two 64-bit words overflows.
type WordRange = (u128, u128);

/// Returns the least value of the form 2^k - 1 that is at least `value`.
fn all_ones(value: u128) -> u128 {
    u128::MAX.checked_shr(value.leading_zeros()).unwrap_or(0)
}

/// Returns the range of `left operator right` for the ranges of its
/// operands, computed exactly, or `None` when storage cannot be reserved.
fn combine_ranges(
    operator: BinaryOperator,
    (left_low, left_high): &IndexRange,
    (right_low, right_high): &IndexRange,
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<IndexRange> {
    match operator {
        BinaryOperator::Add => Some((
            left_low.add(right_low, reserve_limbs)?,
            left_high.add(right_high, reserve_limbs)?,
        )),
        BinaryOperator::Subtract => Some((
            left_low.subtract(right_high, reserve_limbs)?,
            left_high.subtract(right_low, reserve_limbs)?,
        )),
        BinaryOperator::Multiply => {
            let mut products = [
                left_low.multiply(right_low, reserve_limbs)?,
                left_low.multiply(right_high, reserve_limbs)?,
                left_high.multiply(right_low, reserve_limbs)?,
                left_high.multiply(right_high, reserve_limbs)?,
            ];
            products.sort_unstable_by(ExactInteger::compare);
            let [low, _, _, high] = products;
            Some((low, high))
        }
        _ => None,
    }
}

/// Writes an exact integer in decimal, or returns `None` when storage
/// cannot be reserved.
fn render_exact(value: &ExactInteger) -> Option<String> {
    let mut text = String::new();
    fmt::write(&mut text, format_args!("{value}")).ok()?;
    Some(text)
}

fn reserve_range_limbs(limbs: &mut Vec<u32>, count: usize) -> bool {
    limbs.try_reserve_exact(count).is_ok()
}

/// Returns the range of `left / right` or `left % right` under the total
/// Euclidean rules of S3f, for exact operand ranges, or `None` when storage
/// cannot be reserved. Positive, zero, and negative divisors are considered
/// separately and their ranges joined.
fn divide_ranges(
    operator: BinaryOperator,
    left: &IndexRange,
    (divisor_low, divisor_high): &IndexRange,
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<IndexRange> {
    let one = ExactInteger::from_u64(1, reserve_limbs)?;
    let minus_one = one.try_clone_with_reservation(reserve_limbs)?.negated();
    let zero = ExactInteger::from_u64(0, reserve_limbs)?;
    let copy = |value: &ExactInteger| value.try_clone_with_reservation(reserve_limbs);
    let mut parts: [Option<IndexRange>; 3] = [None, None, None];
    if divisor_high.compare(&one) != Ordering::Less {
        let lowest = if divisor_low.compare(&one) == Ordering::Less {
            copy(&one)?
        } else {
            copy(divisor_low)?
        };
        parts[0] = Some(positive_divisor_range(
            operator,
            left,
            (&lowest, divisor_high),
            reserve_limbs,
        )?);
    }
    if divisor_low.compare(&zero) != Ordering::Greater
        && divisor_high.compare(&zero) != Ordering::Less
    {
        // x / 0 = 0 and x % 0 = x.
        parts[1] = Some(if operator == BinaryOperator::Divide {
            (copy(&zero)?, copy(&zero)?)
        } else {
            (copy(&left.0)?, copy(&left.1)?)
        });
    }
    if divisor_low.compare(&minus_one) != Ordering::Greater {
        // For d < 0, x / d = -(x / -d) and x % d = x % -d.
        let nearest = if divisor_high.compare(&minus_one) == Ordering::Greater {
            copy(&one)?
        } else {
            copy(divisor_high)?.negated()
        };
        let farthest = copy(divisor_low)?.negated();
        let (low, high) =
            positive_divisor_range(operator, left, (&nearest, &farthest), reserve_limbs)?;
        parts[2] = Some(if operator == BinaryOperator::Divide {
            (high.negated(), low.negated())
        } else {
            (low, high)
        });
    }
    parts
        .into_iter()
        .flatten()
        .reduce(|(low, high), (part_low, part_high)| {
            (
                if part_low.compare(&low) == Ordering::Less {
                    part_low
                } else {
                    low
                },
                if part_high.compare(&high) == Ordering::Greater {
                    part_high
                } else {
                    high
                },
            )
        })
}

/// Returns the range of `left / d` or `left % d` over divisors d from
/// `divisor_low` through `divisor_high`, where `1 <= divisor_low`, or `None`
/// when storage cannot be reserved.
fn positive_divisor_range(
    operator: BinaryOperator,
    (low, high): &IndexRange,
    (divisor_low, divisor_high): (&ExactInteger, &ExactInteger),
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<IndexRange> {
    let divide =
        |value: &ExactInteger, divisor: &ExactInteger| value.divide_euclid(divisor, reserve_limbs);
    if operator == BinaryOperator::Divide {
        // The quotient grows with x, and for a fixed x it moves toward zero
        // as d grows, so its extremes are at the corners.
        let mut corners = [
            divide(low, divisor_low)?.0,
            divide(low, divisor_high)?.0,
            divide(high, divisor_low)?.0,
            divide(high, divisor_high)?.0,
        ];
        corners.sort_unstable_by(ExactInteger::compare);
        let [least, _, _, greatest] = corners;
        return Some((least, greatest));
    }
    // With one divisor and one quotient, the remainder grows with x.
    if divisor_low.compare(divisor_high) == Ordering::Equal {
        let (low_quotient, low_remainder) = divide(low, divisor_low)?;
        let (high_quotient, high_remainder) = divide(high, divisor_low)?;
        if low_quotient.compare(&high_quotient) == Ordering::Equal {
            return Some((low_remainder, high_remainder));
        }
    }
    // Otherwise 0 <= r < d, and r <= x when x is not negative.
    let one = ExactInteger::from_u64(1, reserve_limbs)?;
    let largest = divisor_high.subtract(&one, reserve_limbs)?;
    let greatest = if !low.is_negative() && high.compare(&largest) == Ordering::Less {
        high.try_clone_with_reservation(reserve_limbs)?
    } else {
        largest
    };
    Some((ExactInteger::from_u64(0, reserve_limbs)?, greatest))
}

fn word_maximum(ty: CoreType) -> Option<u64> {
    match ty {
        CoreType::Int | CoreType::Bool | CoreType::Array(_) => None,
        CoreType::Word8 => Some(u64::from(u8::MAX)),
        CoreType::Word16 => Some(u64::from(u16::MAX)),
        CoreType::Word32 => Some(u64::from(u32::MAX)),
        CoreType::Word64 => Some(u64::MAX),
    }
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    fn new(source: &'source SourceFile, ast: &'ast SyntaxTree, limits: Limits) -> Self {
        Self {
            source,
            ast,
            diagnostics: Vec::new(),
            ordinary_diagnostics: 0,
            diagnostic_limit_reported: false,
            resource_limit_reported: false,
            core_nodes: 0,
            events: 0,
            halted: false,
            limits,
            reserve_pending_function_slot,
            reserve_magnitude_limb,
            reserve_range_limbs,
            reserve_core_name,
            reserve_diagnostic_slots,
            reserve_core_node_slot,
            reserve_call_edge_slot,
        }
    }

    fn run(self) -> AnalysisResult {
        self.run_with_reservations(
            |declarations, capacity| declarations.try_reserve(capacity).is_ok(),
            |functions, capacity| functions.try_reserve_exact(capacity).is_ok(),
        )
    }

    fn run_with_reservations(
        mut self,
        reserve_declarations: impl FnOnce(&mut DeclarationIndex<'ast>, usize) -> bool,
        reserve_core_functions: impl FnOnce(&mut Vec<CoreFunction>, usize) -> bool,
    ) -> AnalysisResult {
        if !(self.reserve_diagnostic_slots)(
            &mut self.diagnostics,
            MAX_RETAINED_SEMANTIC_DIAGNOSTICS,
        ) {
            return AnalysisResult {
                core: None,
                diagnostics: self.diagnostics,
            };
        }
        let mut declarations = DeclarationIndex::new();
        let declaration_capacity = self.ast.module.functions.len();
        if !reserve_declarations(&mut declarations, declaration_capacity) {
            self.resource_limit(
                self.ast.module.span,
                "semantic declaration namespace storage allocation failed",
            );
            return AnalysisResult {
                core: None,
                diagnostics: self.diagnostics,
            };
        }
        for (source_index, function) in self.ast.module.functions.iter().enumerate() {
            declarations.push(DeclarationEntry {
                kind: function.kind,
                name: &function.name.text,
                span: function.name.span,
                source_index,
            });
        }
        declarations.sort_unstable_by(|left, right| {
            left.kind
                .cmp(&right.kind)
                .then_with(|| left.name.cmp(right.name))
                .then_with(|| left.source_index.cmp(&right.source_index))
        });
        let Some(signatures) = self.collect_signatures() else {
            return AnalysisResult {
                core: None,
                diagnostics: self.diagnostics,
            };
        };
        let mut pending_functions = Vec::new();
        let mut call_edges = Vec::new();

        for (source_index, function) in self.ast.module.functions.iter().enumerate() {
            // One event for the declaration-key lookup.
            if !self.event(function.name.span) {
                break;
            }

            let Some(first) =
                first_declaration(&declarations, function.kind, function.name.text.as_str())
            else {
                self.resource_limit(
                    function.name.span,
                    "semantic declaration namespace index is inconsistent",
                );
                break;
            };
            if first.source_index != source_index {
                let span = function.name.span;
                if self.begin_report(span) {
                    let kind = function.kind.as_str();
                    let name = identifier_spelling_for_diagnostic(&function.name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::DuplicateFunction,
                            format!("duplicate {kind} function `{name}`"),
                            span,
                        )
                        .with_label("this declaration repeats a name in the same namespace")
                        .with_secondary_span(first.span, "first declaration is here")
                        .with_note("`spec` and `impl` use separate declaration namespaces"),
                    );
                }
            } else {
                // A successful first declaration performs a separate logical
                // namespace installation after its lookup.
                if !self.event(function.name.span) {
                    break;
                }
            }

            if let FunctionBody::Typed(body) = &function.body {
                if function.kind != FunctionKind::Spec {
                    let span = function.name.span;
                    if self.begin_report(span) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                DiagnosticCode::UnsupportedTypedFunction,
                                "typed bodies are supported only on `spec` functions",
                                span,
                            )
                            .with_label(
                                "this `impl` function has no semantics in the current fragment",
                            )
                            .with_note(
                                "use an empty `impl` body or move the typed body to a `spec` function",
                            ),
                        );
                    }
                    continue;
                }
                let Some(id) = signatures
                    .get(source_index)
                    .and_then(Option::as_ref)
                    .map(|signature| signature.id)
                else {
                    self.resource_limit(function.span, "semantic signature table is inconsistent");
                    break;
                };
                let context = BodyContext {
                    id,
                    name: &function.name,
                    parameters: &function.parameters,
                    parameter_types: Vec::new(),
                    bindings: &body.bindings,
                    binding_types: Vec::new(),
                    loop_scopes: Vec::new(),
                    loops: Vec::new(),
                    conditionals: Vec::new(),
                };
                let scope = ModuleScope {
                    declarations: &declarations,
                    signatures: &signatures,
                };
                if let Some(pending) =
                    self.analyze_typed_function(function, body, context, &scope, &mut call_edges)
                {
                    if (self.reserve_pending_function_slot)(&mut pending_functions) {
                        pending_functions.push(pending);
                    } else {
                        self.resource_limit(
                            function.span,
                            "semantic analysis could not allocate pending function storage",
                        );
                        break;
                    }
                }
                if self.halted {
                    break;
                }
            }
        }

        if !self.halted {
            self.check_call_graph(&signatures, &call_edges);
        }

        let core = if self.diagnostics.is_empty() && !self.halted {
            self.construct_core(pending_functions, reserve_core_functions)
        } else {
            None
        };
        AnalysisResult {
            core,
            diagnostics: self.diagnostics,
        }
    }

    /// Resolves every typed `spec` signature without events or diagnostics.
    ///
    /// Types are reported once, in source order, when their own declaration is
    /// checked; calls to a function whose signature did not resolve are then
    /// not reported again.
    fn collect_signatures(&mut self) -> Option<Vec<Option<Signature>>> {
        let functions = &self.ast.module.functions;
        let mut signatures = Vec::new();
        if signatures.try_reserve_exact(functions.len()).is_err() {
            self.resource_limit(
                self.ast.module.span,
                "semantic signature table allocation failed",
            );
            return None;
        }
        let mut next_id = 0_usize;
        for function in functions {
            let signature = match (&function.body, function.kind) {
                (FunctionBody::Typed(body), FunctionKind::Spec) => {
                    let Some(id) = CoreFunctionId::from_index(next_id) else {
                        self.resource_limit(
                            function.span,
                            "Core function identity exceeds the u32 representation limit",
                        );
                        return None;
                    };
                    next_id = next_id.saturating_add(1);
                    let mut parameters = Vec::new();
                    if parameters
                        .try_reserve_exact(function.parameters.len())
                        .is_err()
                    {
                        self.resource_limit(function.span, "semantic signature allocation failed");
                        return None;
                    }
                    parameters.extend(
                        function
                            .parameters
                            .iter()
                            .map(|parameter| silent_type(self.source, &parameter.ty)),
                    );
                    Some(Signature {
                        id,
                        parameters,
                        result_type: silent_type(self.source, &body.result_type),
                    })
                }
                _ => None,
            };
            signatures.push(signature);
        }
        Some(signatures)
    }

    fn analyze_typed_function(
        &mut self,
        function: &'ast FunctionDeclaration,
        body: &'ast TypedBody,
        mut context: BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        call_edges: &mut Vec<CallEdge>,
    ) -> Option<PendingFunction> {
        if context
            .parameter_types
            .try_reserve_exact(function.parameters.len())
            .is_err()
        {
            self.resource_limit(function.span, "parameter type storage allocation failed");
            return None;
        }
        for (index, parameter) in function.parameters.iter().enumerate() {
            // One event for the parameter-name uniqueness check.
            if !self.event(parameter.name.span) {
                return None;
            }
            let earlier = function.parameters.get(..index).and_then(|earlier| {
                earlier
                    .iter()
                    .find(|candidate| candidate.name.text == parameter.name.text)
            });
            if let Some(earlier) = earlier {
                let span = parameter.name.span;
                if self.begin_report(span) {
                    let name = identifier_spelling_for_diagnostic(&parameter.name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::DuplicateParameter,
                            format!("duplicate parameter `{name}`"),
                            span,
                        )
                        .with_label("this parameter repeats an earlier name")
                        .with_secondary_span(earlier.name.span, "first parameter is here")
                        .with_note("parameter names must be unique within one function"),
                    );
                }
            }
            let ty = self.analyze_type(&parameter.ty, "parameter type");
            context.parameter_types.push(ty);
        }
        let result_type = self.analyze_type(&body.result_type, "result type")?;
        let mut locals = Vec::new();
        if context
            .binding_types
            .try_reserve_exact(body.bindings.len())
            .is_err()
            || locals.try_reserve_exact(body.bindings.len()).is_err()
        {
            self.resource_limit(body.span, "binding storage allocation failed");
            return None;
        }
        let mut bindings_checked = true;
        for (index, binding) in body.bindings.iter().enumerate() {
            let checked =
                self.check_binding(function, body, index, &mut context, scope, call_edges);
            if self.halted {
                return None;
            }
            context
                .binding_types
                .push(checked.as_ref().map(|checked| checked.ty));
            if let Some(CheckedBinding {
                ty,
                nodes: Some(nodes),
            }) = checked
            {
                let name = self.copy_core_name(&binding.name.text, binding.name.span)?;
                locals.push(CoreLocal {
                    span: binding.span,
                    name,
                    name_span: binding.name.span,
                    ty,
                    value: CoreExpression { nodes },
                });
            } else {
                bindings_checked = false;
            }
        }
        let mut output = BodyOutput {
            nodes: Vec::new(),
            call_edges,
        };
        let checked = self.check_expression(
            &body.expression,
            result_type,
            &mut context,
            scope,
            &mut output,
        );
        if !checked || !bindings_checked || self.halted {
            return None;
        }
        let nodes = output.nodes;
        // Every loop and conditional of a well-formed function is well formed.
        let loops = context.loops.into_iter().collect::<Option<Vec<_>>>()?;
        let conditionals = context
            .conditionals
            .into_iter()
            .collect::<Option<Vec<_>>>()?;
        let parameters = context
            .parameter_types
            .iter()
            .copied()
            .collect::<Option<Vec<_>>>()?;
        let name = self.copy_core_name(&function.name.text, function.name.span)?;
        Some(PendingFunction {
            span: function.span,
            name,
            name_span: function.name.span,
            parameters,
            result_type,
            locals,
            nodes,
            loops,
            conditionals,
        })
    }

    /// Checks the binding at `index`: its name, its type, and its value
    /// against that type, using the parameters and the earlier bindings.
    ///
    /// Returns the binding's type and Core nodes, or `None` when the type
    /// did not resolve and the value was not checked. The value's nodes are
    /// present only when it is well typed.
    fn check_binding(
        &mut self,
        function: &'ast FunctionDeclaration,
        body: &'ast TypedBody,
        index: usize,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        call_edges: &mut Vec<CallEdge>,
    ) -> Option<CheckedBinding> {
        let binding = body.bindings.get(index)?;
        // One event for the binding-name uniqueness check.
        if !self.event(binding.name.span) {
            return None;
        }
        let parameter = function
            .parameters
            .iter()
            .find(|parameter| parameter.name.text == binding.name.text)
            .map(|parameter| (parameter.name.span, "the parameter is here"));
        let earlier = parameter.or_else(|| {
            body.bindings
                .get(..index)?
                .iter()
                .find(|earlier| earlier.name.text == binding.name.text)
                .map(|earlier| (earlier.name.span, "the first binding is here"))
        });
        if let Some((earlier_span, earlier_label)) = earlier {
            let span = binding.name.span;
            if self.begin_report(span) {
                let name = identifier_spelling_for_diagnostic(&binding.name.text);
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateBinding,
                        format!("duplicate binding `{name}`"),
                        span,
                    )
                    .with_label("this binding repeats an earlier name")
                    .with_secondary_span(earlier_span, earlier_label)
                    .with_note(
                        "each parameter and binding of a function has its own name; \
                         Orange has no shadowing",
                    ),
                );
            }
        }
        let ty = self.analyze_type(&binding.ty, "binding type")?;
        let mut output = BodyOutput {
            nodes: Vec::new(),
            call_edges,
        };
        let checked = self.check_expression(&binding.value, ty, context, scope, &mut output);
        Some(CheckedBinding {
            ty,
            nodes: (checked && earlier.is_none()).then_some(output.nodes),
        })
    }

    /// Checks `expression` against `expected` and appends its Core nodes in
    /// postorder. Returns whether the expression is well typed.
    ///
    /// Parser-established expression height bounds this recursion.
    fn check_expression(
        &mut self,
        expression: &'ast Expression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        if self.halted {
            return false;
        }
        match &expression.kind {
            ExpressionKind::Literal(literal) if !expected.is_number() => {
                if self.event(literal.span) {
                    if expected == CoreType::Bool {
                        self.report_integer_for_bool(expression.span);
                    } else {
                        self.report_scalar_for_array(
                            expression.span,
                            "an integer literal",
                            expected,
                        );
                    }
                }
                false
            }
            ExpressionKind::Literal(literal) => {
                // Literals keep the S3a accounting: one literal event followed
                // by prefix and significant-digit events.
                let Some(value) = self.analyze_literal(expected, literal) else {
                    return false;
                };
                self.push_node(
                    output,
                    expression.span,
                    expected,
                    CoreNodeKind::Literal(value),
                )
            }
            ExpressionKind::Parenthesized(inner) => {
                if !self.event(expression.span) {
                    return false;
                }
                self.check_expression(inner, expected, context, scope, output)
            }
            ExpressionKind::Name(name) => {
                if !self.event(name.span) {
                    return false;
                }
                self.check_name_reference(name, expected, context, scope, output)
            }
            ExpressionKind::Call(call) => {
                if !self.event(expression.span) {
                    return false;
                }
                self.check_call(expression, call, expected, context, scope, output)
            }
            ExpressionKind::Unary(unary) => {
                if !self.event(unary.operator_span) {
                    return false;
                }
                if !self.unary_is_defined(unary, expected) {
                    return false;
                }
                let operand =
                    self.check_expression(&unary.operand, expected, context, scope, output);
                operand
                    && self.push_node(
                        output,
                        expression.span,
                        expected,
                        CoreNodeKind::Unary(unary.operator),
                    )
            }
            ExpressionKind::Conversion(conversion) => {
                if !self.event(conversion.keyword_span) {
                    return false;
                }
                self.check_conversion(expression, conversion, expected, context, scope, output)
            }
            ExpressionKind::Array(array) => {
                if !self.event(expression.span) {
                    return false;
                }
                self.check_array(expression, array, expected, context, scope, output)
            }
            ExpressionKind::Fill(fill) => {
                // One event for the literal and one for its length token.
                if !self.event(expression.span) || !self.event(fill.length_span) {
                    return false;
                }
                self.check_fill(expression, fill, expected, context, scope, output)
            }
            ExpressionKind::Index(index) => {
                if !self.event(index.index_span()) {
                    return false;
                }
                self.check_index(expression, index, expected, context, scope, output)
            }
            ExpressionKind::Update(update) => {
                if !self.event(update.keyword_span) {
                    return false;
                }
                self.check_update(expression, update, expected, context, scope, output)
            }
            ExpressionKind::Loop(r#loop) => {
                if !self.event(r#loop.keyword_span) {
                    return false;
                }
                self.check_loop(expression, r#loop, expected, context, scope, output)
            }
            ExpressionKind::Conditional(conditional) => {
                self.check_conditional(expression, conditional, expected, context, scope, output)
            }
            ExpressionKind::Binary(binary) => {
                if !self.event(binary.operator_span) {
                    return false;
                }
                if binary.operator.is_comparison() {
                    return self
                        .check_comparison(expression, binary, expected, context, scope, output);
                }
                if !self.binary_is_defined(binary, expected) {
                    return false;
                }
                let left = self.check_expression(&binary.left, expected, context, scope, output);
                if binary.operator.is_shift_or_rotation() {
                    let amount = self.check_shift_amount(binary, expected);
                    return match (left, amount) {
                        (true, Some(amount)) => self.push_node(
                            output,
                            expression.span,
                            expected,
                            CoreNodeKind::Shift {
                                operator: binary.operator,
                                amount,
                            },
                        ),
                        _ => false,
                    };
                }
                let right = self.check_expression(&binary.right, expected, context, scope, output);
                left && right
                    && self.push_node(
                        output,
                        expression.span,
                        expected,
                        CoreNodeKind::Binary(binary.operator),
                    )
            }
        }
    }

    fn check_name_reference(
        &mut self,
        name: &'ast Identifier,
        expected: CoreType,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let (actual, kind) = match context.resolve(&name.text) {
            NameResolution::Parameter(index) => (
                context.parameter_types.get(index).copied(),
                u32::try_from(index).map(CoreNodeKind::Parameter),
            ),
            NameResolution::Binding(index) => (
                context.binding_types.get(index).copied(),
                u32::try_from(index).map(CoreNodeKind::Local),
            ),
            NameResolution::LoopIndex(position) => match context.loop_scopes.get(position) {
                Some(scope) => (
                    Some(Some(CoreType::Int)),
                    Ok(CoreNodeKind::LoopIndex(scope.id)),
                ),
                None => (None, Ok(CoreNodeKind::LoopIndex(0))),
            },
            NameResolution::Accumulator(position) => match context.loop_scopes.get(position) {
                Some(scope) => (
                    Some(Some(scope.ty)),
                    Ok(CoreNodeKind::Accumulator(scope.id)),
                ),
                None => (None, Ok(CoreNodeKind::Accumulator(0))),
            },
            NameResolution::BoolLiteral(value) => (
                Some(Some(CoreType::Bool)),
                Ok(CoreNodeKind::Literal(CoreValue::Bool(value))),
            ),
            NameResolution::LaterBinding(binding) => {
                if self.begin_report(name.span) {
                    let spelling = identifier_spelling_for_diagnostic(&name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnknownParameter,
                            format!("`{spelling}` is used before it is bound"),
                            name.span,
                        )
                        .with_label("not bound yet")
                        .with_secondary_span(binding.name.span, "the binding is here")
                        .with_note(
                            "a binding is in scope after its own `;`, for the bindings that \
                             follow it and the result",
                        ),
                    );
                }
                return false;
            }
            NameResolution::Unknown => {
                self.report_unknown_name(name, context, scope);
                return false;
            }
        };
        // An unresolved parameter or binding type was reported at its
        // declaration.
        let Some(Some(actual)) = actual else {
            return false;
        };
        if actual != expected {
            if self.begin_report(name.span) {
                let spelling = identifier_spelling_for_diagnostic(&name.text);
                let note = if actual.as_array().map(ArrayType::element) == Some(expected) {
                    format!("select one element with an index, such as `{spelling}[0]`")
                } else {
                    String::from("Orange has no implicit conversions between types")
                };
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!(
                            "`{spelling}` has type `{actual}`, but `{expected}` is required here"
                        ),
                        name.span,
                    )
                    .with_label(format!("expected `{expected}`"))
                    .with_note(note),
                );
            }
            return false;
        }
        let Ok(kind) = kind else {
            self.resource_limit(
                name.span,
                "parameter or binding index exceeds the u32 representation limit",
            );
            return false;
        };
        self.push_node(output, name.span, expected, kind)
    }

    #[cold]
    #[inline(never)]
    fn report_unknown_name(
        &mut self,
        name: &Identifier,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) {
        if !self.begin_report(name.span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(&name.text);
        let function = identifier_spelling_for_diagnostic(&context.name.text);
        let has_bindings = !context.bindings.is_empty();
        let mut diagnostic = Diagnostic::error(
            DiagnosticCode::UnknownParameter,
            if has_bindings {
                format!("`{spelling}` is not a parameter or binding of `{function}`")
            } else {
                format!("`{spelling}` is not a parameter of `{function}`")
            },
            name.span,
        )
        .with_label("unknown name");
        diagnostic =
            if first_declaration(scope.declarations, FunctionKind::Spec, &name.text).is_some() {
                diagnostic.with_note(format!(
                    "to call the function `{spelling}`, write `{spelling}()` with its arguments"
                ))
            } else if has_bindings {
                diagnostic.with_note(
                    "a bare name in a `spec` body refers to one of its parameters or bindings",
                )
            } else {
                diagnostic.with_note("a bare name in a `spec` body refers to one of its parameters")
            };
        self.diagnostics.push(diagnostic);
    }

    /// Checks `operand as Target` against `expected`.
    ///
    /// The target type is resolved and compared with `expected` first, as a
    /// call's result type is. The operand's own type is the type of its first
    /// typed leaf, and the operand is then checked against that type.
    fn check_conversion(
        &mut self,
        expression: &'ast Expression,
        conversion: &'ast ConversionExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let target = self.analyze_type(&conversion.target, "conversion type");
        if self.halted {
            return false;
        }
        let target_matches = match target {
            Some(target) if target != expected => {
                if self.begin_report(conversion.target.span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::TypeMismatch,
                            format!(
                                "this conversion gives `{target}`, but `{expected}` is required here"
                            ),
                            conversion.target.span,
                        )
                        .with_label(format!("expected `{expected}`"))
                        .with_note("`as` gives exactly the type written after it"),
                    );
                }
                false
            }
            Some(_) => true,
            None => false,
        };
        let Some(leaf) = first_typed_leaf(&conversion.operand) else {
            let span = conversion.operand.span;
            if self.begin_report(span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::UntypedConversionOperand,
                        "the operand of `as` has no type of its own",
                        span,
                    )
                    .with_label("a literal takes its type from where it is used")
                    .with_note(
                        "write the literal where its type is required, or give it a type \
                         with a `let` binding",
                    ),
                );
            }
            return false;
        };
        let from = self.leaf_type(leaf, context, scope);
        if matches!(leaf.kind, ExpressionKind::Array(_))
            || from.is_some_and(|from| !from.is_scalar())
        {
            let span = conversion.keyword_span;
            if self.begin_report(span) {
                let operand =
                    from.map_or_else(|| String::from("an array"), |from| format!("`{from}`"));
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::UnsupportedOperator,
                        format!("`as` is not defined for {operand}"),
                        span,
                    )
                    .with_label("`as` converts one `Int` or word value")
                    .with_note("convert each element, such as `x[0] as Int`"),
                );
            }
            return false;
        }
        let Some(from) = from else {
            // The leaf's own check reports why it has no type. That check
            // stops before comparing with the type passed here.
            self.check_expression(leaf, expected, context, scope, output);
            return false;
        };
        if from == CoreType::Bool || target == Some(CoreType::Bool) {
            let span = conversion.keyword_span;
            if self.begin_report(span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::UnsupportedOperator,
                        "`as` does not convert to or from `Bool`",
                        span,
                    )
                    .with_label("`as` converts one `Int` or word value")
                    .with_note(
                        "choose a number with a conditional, such as `if b { 1 } else { 0 }`, \
                         or compare a number, such as `x != 0`",
                    ),
                );
            }
            return false;
        }
        let operand = self.check_expression(&conversion.operand, from, context, scope, output);
        operand
            && target_matches
            && self.push_node(
                output,
                expression.span,
                expected,
                CoreNodeKind::Convert { from },
            )
    }

    /// Returns the type of a name, call, or conversion without reporting.
    fn leaf_type(
        &self,
        leaf: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<CoreType> {
        match &leaf.kind {
            ExpressionKind::Name(name) => context.name_type(&name.text),
            ExpressionKind::Call(call) => {
                let entry =
                    first_declaration(scope.declarations, FunctionKind::Spec, &call.callee.text)?;
                scope
                    .signatures
                    .get(entry.source_index)?
                    .as_ref()?
                    .result_type
            }
            ExpressionKind::Conversion(conversion) => silent_type(self.source, &conversion.target),
            ExpressionKind::Index(index) => self
                .leaf_type(&index.base, context, scope)?
                .as_array()
                .map(ArrayType::element),
            ExpressionKind::Loop(r#loop) => silent_type(self.source, &r#loop.ty),
            ExpressionKind::Update(update) => {
                self.leaf_type(first_typed_leaf(&update.base)?, context, scope)
            }
            ExpressionKind::Binary(binary) if binary.operator.is_comparison() => {
                Some(CoreType::Bool)
            }
            ExpressionKind::Literal(_)
            | ExpressionKind::Unary(_)
            | ExpressionKind::Binary(_)
            | ExpressionKind::Parenthesized(_)
            | ExpressionKind::Array(_)
            | ExpressionKind::Fill(_)
            | ExpressionKind::Conditional(_) => None,
        }
    }

    /// Checks an array literal against `expected`, which must be an array
    /// type of the same length; each element is checked against its element
    /// type.
    fn check_array(
        &mut self,
        expression: &'ast Expression,
        array: &'ast ArrayExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some(array_type) = expected.as_array() else {
            if self.begin_report(expression.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!("an array literal cannot have type `{expected}`"),
                        expression.span,
                    )
                    .with_label(format!("expected `{expected}`"))
                    .with_note("an array literal is written where an array type `T^n` is required"),
                );
            }
            return false;
        };
        let supplied = array.elements.len();
        let length_matches = usize::try_from(array_type.length()).ok() == Some(supplied);
        if !length_matches && self.begin_report(expression.span) {
            let length = array_type.length();
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::ArrayLengthMismatch,
                    format!(
                        "this array has {supplied} {}, but `{expected}` has {length}",
                        if supplied == 1 { "element" } else { "elements" },
                    ),
                    expression.span,
                )
                .with_label(format!(
                    "expected {length} {}",
                    if length == 1 { "element" } else { "elements" }
                ))
                .with_note("an array literal lists every element of its type exactly once"),
            );
        }
        let mut elements_checked = true;
        for element in &array.elements {
            elements_checked &=
                self.check_expression(element, array_type.element(), context, scope, output);
            if self.halted {
                return false;
            }
        }
        if !length_matches || !elements_checked {
            return false;
        }
        let Ok(elements) = u32::try_from(supplied) else {
            self.resource_limit(
                expression.span,
                "array element count exceeds the u32 representation limit",
            );
            return false;
        };
        self.push_node(
            output,
            expression.span,
            expected,
            CoreNodeKind::Array { elements },
        )
    }

    /// Checks `base[INDEX]` against `expected`.
    ///
    /// The base's type is found without reporting, as a conversion operand's
    /// is. It must be an array, the index must be below its length, and its
    /// element type must be `expected`; the base is then checked against its
    /// own type, so errors inside it are still reported.
    fn check_index(
        &mut self,
        expression: &'ast Expression,
        index: &'ast IndexExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some(base_type) = self.leaf_type(&index.base, context, scope) else {
            // The base's own check reports why it has no type and stops
            // before comparing with the type passed here.
            self.check_expression(&index.base, expected, context, scope, output);
            return false;
        };
        let Some(array_type) = base_type.as_array() else {
            if self.begin_report(index.base.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::NotAnArray,
                        format!("only an array can be indexed, but this has type `{base_type}`"),
                        index.base.span,
                    )
                    .with_label(format!("`{base_type}` has no elements"))
                    .with_note("an index selects one element of a value of type `T^n`"),
                );
            }
            self.check_expression(&index.base, base_type, context, scope, output);
            return false;
        };
        let literal = index.is_literal();
        let position = if literal {
            self.check_index_literal(index, array_type)
        } else {
            None
        };
        let element = array_type.element();
        let element_matches = element == expected;
        if !element_matches && self.begin_report(expression.span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("this element has type `{element}`, but `{expected}` is required here"),
                    expression.span,
                )
                .with_label(format!("expected `{expected}`"))
                .with_note("Orange has no implicit conversions between types"),
            );
        }
        let base = self.check_expression(&index.base, base_type, context, scope, output);
        if !literal {
            let in_range =
                self.check_static_index(&index.index, array_type, context, scope, output);
            return in_range
                && base
                && element_matches
                && self.push_node(output, expression.span, expected, CoreNodeKind::Select);
        }
        match position {
            Some(position) if base && element_matches => self.push_node(
                output,
                expression.span,
                expected,
                CoreNodeKind::Index { index: position },
            ),
            _ => false,
        }
    }

    /// Checks an index expression, then proves it in range for `array`.
    ///
    /// The index has the type of its first typed leaf when that is a word
    /// type, and is an `Int` otherwise. A word index ranges over its type,
    /// narrowed by its operators, and is converted to its unsigned `Int`
    /// value in Core; an `Int` index takes its range from its literals, loop
    /// indices, and converted words.
    fn check_static_index(
        &mut self,
        index: &'ast Expression,
        array: ArrayType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let word = first_typed_leaf(index)
            .and_then(|leaf| self.leaf_type(leaf, context, scope))
            .filter(|ty| word_maximum(*ty).is_some());
        let index_type = word.unwrap_or(CoreType::Int);
        if !self.check_expression(index, index_type, context, scope, output) {
            return false;
        }
        let length = array.length();
        let range = match word.and_then(word_maximum) {
            Some(maximum) => {
                let range = self.word_range(index, u128::from(maximum), context, scope);
                Ok(self.exact_word_range(index.span, range))
            }
            None => self.static_range(index, context, scope),
        };
        let range = match range {
            Ok(range) => range,
            Err(span) => {
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::NonStaticIndex,
                            "an `Int` index may use only integer literals, loop indices, and \
                             words converted with `as Int`",
                            span,
                        )
                        .with_label("this `Int` has no bound")
                        .with_note(STATIC_INDEX_NOTE),
                    );
                }
                return false;
            }
        };
        let in_range = range.as_ref().is_some_and(|(low, high)| {
            !low.is_negative() && high.to_i64().is_some_and(|high| high < i64::from(length))
        });
        if !in_range && self.begin_report(index.span) {
            let highest = length.saturating_sub(1);
            let array = CoreType::Array(array);
            let message = match &range {
                Some((low, high)) => match (render_exact(low), render_exact(high)) {
                    (Some(low_text), Some(_)) if low.compare(high) == Ordering::Equal => {
                        format!("index {low_text} is out of range for `{array}`")
                    }
                    (Some(low_text), Some(high_text)) => format!(
                        "this index runs from {low_text} through {high_text}, out of range \
                         for `{array}`"
                    ),
                    _ => format!("this index is out of range for `{array}`"),
                },
                None => format!(
                    "a bound of this index's range exceeds the {}-significant-bit limit of \
                     `Int`",
                    self.limits.integer_bits
                ),
            };
            self.diagnostics.push(
                Diagnostic::error(DiagnosticCode::IndexOutOfRange, message, index.span)
                    .with_label(format!("indices run from 0 through {highest}"))
                    .with_note(
                        "every value an index can take, over every loop index in it, must \
                         select an element",
                    ),
            );
        }
        match word {
            Some(from) if in_range => self.push_node(
                output,
                index.span,
                CoreType::Int,
                CoreNodeKind::Convert { from },
            ),
            _ => in_range,
        }
    }

    /// Returns the least and greatest values of a well-typed `Int` index
    /// built from literals, loop indices, words converted with `as Int`,
    /// parentheses, negation, `+`, `-`, `*`, `/`, `%`, and conditionals,
    /// computed exactly, or `None` when a bound's magnitude exceeds the
    /// integer limit or storage cannot be reserved (which is reported as a
    /// resource limit). Anything else is returned as the span of the first
    /// such part.
    ///
    /// Parser-established expression height bounds this recursion.
    fn static_range(
        &mut self,
        index: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Result<Option<IndexRange>, Span> {
        let range = match &index.kind {
            ExpressionKind::Literal(literal) => self.range_literal(literal).and_then(|value| {
                let copy = value.try_clone_with_reservation(self.reserve_range_limbs)?;
                Some((value, copy))
            }),
            ExpressionKind::Parenthesized(inner) => {
                return self.static_range(inner, context, scope);
            }
            ExpressionKind::Name(name) => match context.resolve(&name.text) {
                NameResolution::LoopIndex(position) => {
                    let scope = context.loop_scopes.get(position).ok_or(name.span)?;
                    let last = scope.end.checked_sub(1).ok_or(name.span)?;
                    ExactInteger::from_u64(u64::from(scope.start), self.reserve_range_limbs).zip(
                        ExactInteger::from_u64(u64::from(last), self.reserve_range_limbs),
                    )
                }
                _ => return Err(name.span),
            },
            ExpressionKind::Unary(unary) if unary.operator == UnaryOperator::Negate => {
                return Ok(self
                    .static_range(&unary.operand, context, scope)?
                    .map(|(low, high)| (high.negated(), low.negated())));
            }
            ExpressionKind::Binary(binary)
                if matches!(
                    binary.operator,
                    BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply
                ) =>
            {
                let left = self.static_range(&binary.left, context, scope)?;
                let right = self.static_range(&binary.right, context, scope)?;
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                combine_ranges(binary.operator, &left, &right, self.reserve_range_limbs)
            }
            ExpressionKind::Binary(binary) if binary.operator.is_division() => {
                let left = self.static_range(&binary.left, context, scope)?;
                let right = self.static_range(&binary.right, context, scope)?;
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                divide_ranges(binary.operator, &left, &right, self.reserve_range_limbs)
            }
            ExpressionKind::Conversion(conversion) => {
                let from = first_typed_leaf(&conversion.operand)
                    .and_then(|leaf| self.leaf_type(leaf, context, scope));
                match from {
                    Some(CoreType::Int) => {
                        return self.static_range(&conversion.operand, context, scope);
                    }
                    Some(from) => {
                        let Some(maximum) = word_maximum(from) else {
                            return Err(index.span);
                        };
                        let range = self.word_range(
                            &conversion.operand,
                            u128::from(maximum),
                            context,
                            scope,
                        );
                        return Ok(self.exact_word_range(index.span, range));
                    }
                    None => return Err(index.span),
                }
            }
            ExpressionKind::Conditional(conditional) => {
                let mut joined: Option<IndexRange> = None;
                for value in conditional
                    .arms
                    .iter()
                    .map(|arm| &arm.value)
                    .chain(std::iter::once(&conditional.otherwise))
                {
                    let Some((low, high)) = self.static_range(value, context, scope)? else {
                        return Ok(None);
                    };
                    joined = Some(match joined {
                        None => (low, high),
                        Some((least, greatest)) => (
                            if low.compare(&least) == Ordering::Less {
                                low
                            } else {
                                least
                            },
                            if high.compare(&greatest) == Ordering::Greater {
                                high
                            } else {
                                greatest
                            },
                        ),
                    });
                }
                joined
            }
            _ => return Err(index.span),
        };
        let Some(range) = range else {
            self.resource_limit(index.span, "index range storage allocation failed");
            return Ok(None);
        };
        let bits = self.limits.integer_bits;
        Ok((range.0.magnitude_bits() <= bits && range.1.magnitude_bits() <= bits).then_some(range))
    }

    /// Decodes an index literal exactly, without events or diagnostics; the
    /// literal was already checked as an `Int`. Returns `None` when storage
    /// cannot be reserved.
    fn range_literal(&self, literal: &IntegerLiteral) -> Option<ExactInteger> {
        let spelling = self.source.slice(literal.magnitude_span)?;
        let (radix, digits) = if let Some(digits) = spelling
            .strip_prefix("0b")
            .or_else(|| spelling.strip_prefix("0B"))
        {
            (2, digits)
        } else if let Some(digits) = spelling
            .strip_prefix("0x")
            .or_else(|| spelling.strip_prefix("0X"))
        {
            (16, digits)
        } else {
            (10, spelling)
        };
        let mut magnitude = Magnitude::zero();
        for character in digits.chars().filter(|character| *character != '_') {
            let digit = character.to_digit(radix)?;
            if !magnitude.multiply_add_with_reservation(radix, digit, self.reserve_magnitude_limb) {
                return None;
            }
        }
        Some(ExactInteger::new(literal.negative, magnitude))
    }

    /// Returns the least and greatest values of a well-typed word expression
    /// whose type's greatest value is `maximum`. Every word lies in its type,
    /// and an operator narrows that where its result provably does not wrap:
    /// `&`, `|`, `^`, `~`, `/`, `%`, shifts by a literal amount, and `+`,
    /// `-`, and `*` whose bounds stay within the type. Conditionals join
    /// their branches, and a conversion from a narrower word keeps its range.
    /// Everything else ranges over the whole type. No bound is ever more
    /// than `maximum`.
    ///
    /// Parser-established expression height bounds this recursion.
    fn word_range(
        &self,
        expression: &Expression,
        maximum: u128,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> WordRange {
        let whole = (0, maximum);
        match &expression.kind {
            ExpressionKind::Literal(literal) if !literal.negative => self
                .literal_value(literal)
                .filter(|value| *value <= maximum)
                .map_or(whole, |value| (value, value)),
            ExpressionKind::Parenthesized(inner) => self.word_range(inner, maximum, context, scope),
            ExpressionKind::Unary(unary) if unary.operator == UnaryOperator::Complement => {
                let (low, high) = self.word_range(&unary.operand, maximum, context, scope);
                (maximum.saturating_sub(high), maximum.saturating_sub(low))
            }
            ExpressionKind::Binary(binary) if binary.operator.is_shift_or_rotation() => {
                let (low, high) = self.word_range(&binary.left, maximum, context, scope);
                let amount = match &binary.right.kind {
                    ExpressionKind::Literal(literal) if !literal.negative => self
                        .literal_value(literal)
                        .and_then(|amount| u32::try_from(amount).ok()),
                    _ => None,
                };
                match (binary.operator, amount) {
                    (BinaryOperator::ShiftRight, Some(amount)) => (
                        low.checked_shr(amount).unwrap_or(0),
                        high.checked_shr(amount).unwrap_or(0),
                    ),
                    (BinaryOperator::ShiftLeft, Some(amount)) => {
                        match (low.checked_shl(amount), high.checked_shl(amount)) {
                            (Some(least), Some(greatest))
                                if greatest.checked_shr(amount) == Some(high)
                                    && greatest <= maximum =>
                            {
                                (least, greatest)
                            }
                            _ => whole,
                        }
                    }
                    _ => whole,
                }
            }
            ExpressionKind::Binary(binary) => {
                let (left_low, left_high) = self.word_range(&binary.left, maximum, context, scope);
                let (right_low, right_high) =
                    self.word_range(&binary.right, maximum, context, scope);
                let within = |low: Option<u128>, high: Option<u128>| match (low, high) {
                    (Some(low), Some(high)) if high <= maximum => (low, high),
                    _ => whole,
                };
                match binary.operator {
                    BinaryOperator::And => (0, left_high.min(right_high)),
                    BinaryOperator::Or => {
                        (left_low.max(right_low), all_ones(left_high.max(right_high)))
                    }
                    BinaryOperator::Xor => (0, all_ones(left_high.max(right_high))),
                    BinaryOperator::Add => within(
                        left_low.checked_add(right_low),
                        left_high.checked_add(right_high),
                    ),
                    BinaryOperator::Subtract if left_low >= right_high => within(
                        left_low.checked_sub(right_high),
                        left_high.checked_sub(right_low),
                    ),
                    BinaryOperator::Multiply => within(
                        left_low.checked_mul(right_low),
                        left_high.checked_mul(right_high),
                    ),
                    // x / 0 = 0, and a quotient never exceeds its dividend.
                    BinaryOperator::Divide => match (
                        left_low.checked_div(right_high),
                        left_high.checked_div(right_low),
                    ) {
                        (Some(low), Some(high)) => (low, high),
                        _ => (0, left_high),
                    },
                    // x % 0 = x, and otherwise 0 <= x % d <= min(x, d - 1).
                    BinaryOperator::Remainder if right_low > 0 && left_high < right_low => {
                        (left_low, left_high)
                    }
                    BinaryOperator::Remainder if right_low > 0 => {
                        (0, left_high.min(right_high.saturating_sub(1)))
                    }
                    BinaryOperator::Remainder => (0, left_high),
                    _ => whole,
                }
            }
            ExpressionKind::Conditional(conditional) => conditional
                .arms
                .iter()
                .map(|arm| &arm.value)
                .chain(std::iter::once(&conditional.otherwise))
                .map(|value| self.word_range(value, maximum, context, scope))
                .reduce(|(low, high), (least, greatest)| (low.min(least), high.max(greatest)))
                .unwrap_or(whole),
            ExpressionKind::Conversion(conversion) => first_typed_leaf(&conversion.operand)
                .and_then(|leaf| self.leaf_type(leaf, context, scope))
                .and_then(word_maximum)
                .map(|from| self.word_range(&conversion.operand, u128::from(from), context, scope))
                .filter(|(_, high)| *high <= maximum)
                .unwrap_or(whole),
            _ => whole,
        }
    }

    /// Converts a word range to exact integers, reporting a failed
    /// reservation as a resource limit at `span` and giving `None`.
    fn exact_word_range(&mut self, span: Span, (low, high): WordRange) -> Option<IndexRange> {
        let exact = |value: u128| {
            u64::try_from(value)
                .ok()
                .and_then(|value| ExactInteger::from_u64(value, self.reserve_range_limbs))
        };
        let range = exact(low).zip(exact(high));
        if range.is_none() {
            self.resource_limit(span, "index range storage allocation failed");
        }
        range
    }

    /// Decodes a literal's magnitude without allocating, or gives `None`
    /// when it exceeds 128 bits.
    fn literal_value(&self, literal: &IntegerLiteral) -> Option<u128> {
        let spelling = self.source.slice(literal.magnitude_span)?;
        let (radix, digits) = if let Some(digits) = spelling
            .strip_prefix("0b")
            .or_else(|| spelling.strip_prefix("0B"))
        {
            (2, digits)
        } else if let Some(digits) = spelling
            .strip_prefix("0x")
            .or_else(|| spelling.strip_prefix("0X"))
        {
            (16, digits)
        } else {
            (10, spelling)
        };
        digits
            .chars()
            .filter(|character| *character != '_')
            .try_fold(0_u128, |value, character| {
                value
                    .checked_mul(u128::from(radix))?
                    .checked_add(u128::from(character.to_digit(radix)?))
            })
    }

    /// Checks `base with [index] = value` against `expected`, which must be
    /// the base's array type.
    fn check_update(
        &mut self,
        expression: &'ast Expression,
        update: &'ast UpdateExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let base_type =
            first_typed_leaf(&update.base).and_then(|leaf| self.leaf_type(leaf, context, scope));
        if let Some(base_type) = base_type
            && base_type.is_scalar()
        {
            if self.begin_report(update.base.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::NotAnArray,
                        format!("only an array can be updated, but this has type `{base_type}`"),
                        update.base.span,
                    )
                    .with_label(format!("`{base_type}` has no elements"))
                    .with_note("`x with [i] = v` is the array `x` with one element replaced"),
                );
            }
            self.check_expression(&update.base, base_type, context, scope, output);
            return false;
        }
        let Some(array) = expected.as_array() else {
            if self.begin_report(expression.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!("an update gives an array, but `{expected}` is required here"),
                        expression.span,
                    )
                    .with_label(format!("expected `{expected}`"))
                    .with_note("`x with [i] = v` is the array `x` with one element replaced"),
                );
            }
            return false;
        };
        let base = self.check_expression(&update.base, expected, context, scope, output);
        if self.halted {
            return false;
        }
        let index = self.check_static_index(&update.index, array, context, scope, output);
        if self.halted {
            return false;
        }
        let value = self.check_expression(&update.value, array.element(), context, scope, output);
        base && index
            && value
            && self.push_node(output, expression.span, expected, CoreNodeKind::Update)
    }

    /// Checks `[element; n]` against `expected`, which must be an array type
    /// of length n.
    fn check_fill(
        &mut self,
        expression: &'ast Expression,
        fill: &'ast FillExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some(array) = expected.as_array() else {
            if self.begin_report(expression.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!("an array literal cannot have type `{expected}`"),
                        expression.span,
                    )
                    .with_label(format!("expected `{expected}`"))
                    .with_note("an array literal is written where an array type `T^n` is required"),
                );
            }
            return false;
        };
        let length = array_length(self.source, fill.length_span)
            .filter(|length| (1..=MAX_ARRAY_LENGTH).contains(length));
        match length {
            None => self.report_unsupported_array_length(fill.length_span),
            Some(length) if length != array.length() => {
                if self.begin_report(expression.span) {
                    let expected_length = array.length();
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::ArrayLengthMismatch,
                            format!(
                                "this array has {length} {}, but `{expected}` has {expected_length}",
                                if length == 1 { "element" } else { "elements" },
                            ),
                            expression.span,
                        )
                        .with_label(format!(
                            "expected {expected_length} {}",
                            if expected_length == 1 {
                                "element"
                            } else {
                                "elements"
                            }
                        ))
                        .with_note("`[e; n]` is the array of n copies of e"),
                    );
                }
            }
            Some(_) => {}
        }
        let element = self.check_expression(&fill.element, array.element(), context, scope, output);
        element
            && length == Some(array.length())
            && self.push_node(output, expression.span, expected, CoreNodeKind::Fill)
    }

    /// Checks `for i in a..b with s: T = init { step }` against `expected`.
    ///
    /// The loop takes its number before its initial value is checked, so
    /// loops are numbered in source order of their `for` keywords. The step
    /// is checked with the index and the accumulator in scope and becomes
    /// the loop's own Core expression.
    fn check_loop(
        &mut self,
        expression: &'ast Expression,
        r#loop: &'ast LoopExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let bounds = self.check_loop_bounds(r#loop);
        if self.halted {
            return false;
        }
        let mut names_unique = true;
        for (name, sibling) in [
            (&r#loop.index, None),
            (&r#loop.accumulator, Some(&r#loop.index)),
        ] {
            // One event for each loop name's uniqueness check.
            if !self.event(name.span) {
                return false;
            }
            let earlier = context.earlier_name(&name.text).or_else(|| {
                sibling
                    .filter(|sibling| sibling.text == name.text)
                    .map(|sibling| (sibling.span, "the loop index is here"))
            });
            if let Some((earlier_span, earlier_label)) = earlier {
                names_unique = false;
                if self.begin_report(name.span) {
                    let spelling = identifier_spelling_for_diagnostic(&name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::DuplicateBinding,
                            format!("duplicate name `{spelling}`"),
                            name.span,
                        )
                        .with_label("this name repeats a name in scope")
                        .with_secondary_span(earlier_span, earlier_label)
                        .with_note(
                            "each parameter, binding, loop index, and accumulator in scope has \
                             its own name; Orange has no shadowing",
                        ),
                    );
                }
            }
        }
        let Some(ty) = self.analyze_type(&r#loop.ty, "accumulator type") else {
            return false;
        };
        let type_matches = ty == expected;
        if !type_matches && self.begin_report(expression.span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("this loop has type `{ty}`, but `{expected}` is required here"),
                    expression.span,
                )
                .with_label(format!("expected `{expected}`"))
                .with_note("a loop's value is its accumulator after the last step"),
            );
        }
        let Ok(id) = u32::try_from(context.loops.len()) else {
            self.resource_limit(
                expression.span,
                "loop count exceeds the u32 representation limit",
            );
            return false;
        };
        if context.loops.try_reserve(1).is_err() {
            self.resource_limit(expression.span, "loop storage allocation failed");
            return false;
        }
        context.loops.push(None);
        let init = self.check_expression(&r#loop.init, ty, context, scope, output);
        let Some((start, end)) = bounds else {
            return false;
        };
        if self.halted {
            return false;
        }
        if context.loop_scopes.try_reserve(1).is_err() {
            self.resource_limit(expression.span, "loop scope allocation failed");
            return false;
        }
        context.loop_scopes.push(LoopScope {
            id,
            index: &r#loop.index,
            accumulator: &r#loop.accumulator,
            ty,
            start,
            end,
        });
        let mut step_output = BodyOutput {
            nodes: Vec::new(),
            call_edges: &mut *output.call_edges,
        };
        let step = self.check_expression(&r#loop.step, ty, context, scope, &mut step_output);
        let step_nodes = step_output.nodes;
        let mut loop_scope = Vec::new();
        let scope_reserved = loop_scope
            .try_reserve_exact(context.loop_scopes.len())
            .is_ok();
        loop_scope.extend(context.loop_scopes.iter().map(|scope| scope.id));
        context.loop_scopes.pop();
        if !scope_reserved {
            self.resource_limit(expression.span, "loop scope allocation failed");
            return false;
        }
        if !(init && step && names_unique && type_matches) || self.halted {
            return false;
        }
        let Ok(visible_locals) = u32::try_from(context.binding_types.len()) else {
            self.resource_limit(
                expression.span,
                "binding count exceeds the u32 representation limit",
            );
            return false;
        };
        let Some(index_name) = self.copy_core_name(&r#loop.index.text, r#loop.index.span) else {
            return false;
        };
        let Some(accumulator_name) =
            self.copy_core_name(&r#loop.accumulator.text, r#loop.accumulator.span)
        else {
            return false;
        };
        let Some(entry) = usize::try_from(id)
            .ok()
            .and_then(|id| context.loops.get_mut(id))
        else {
            self.resource_limit(expression.span, "semantic loop table is inconsistent");
            return false;
        };
        *entry = Some(CoreLoop {
            span: expression.span,
            index_name,
            accumulator_name,
            ty,
            start,
            end,
            visible_locals,
            scope: loop_scope,
            step: CoreExpression { nodes: step_nodes },
        });
        self.push_node(output, expression.span, expected, CoreNodeKind::Fold(id))
    }

    /// Decodes a loop's bounds and checks that they form a nonempty range
    /// within 0 through [`MAX_LOOP_BOUND`].
    fn check_loop_bounds(&mut self, r#loop: &LoopExpression) -> Option<(u32, u32)> {
        let start = self.loop_bound(r#loop.start_span)?;
        let end = self.loop_bound(r#loop.end_span)?;
        let limit = u64::from(MAX_LOOP_BOUND);
        let (span, message, label) = if start > limit || end > limit {
            let span = if start > limit {
                r#loop.start_span
            } else {
                r#loop.end_span
            };
            (
                span,
                format!("a loop bound must be at most {MAX_LOOP_BOUND}"),
                "loop bound too large",
            )
        } else if start >= end {
            (
                r#loop.end_span,
                format!("the loop range {start}..{end} is empty"),
                "a loop runs at least once",
            )
        } else {
            return Some((u32::try_from(start).ok()?, u32::try_from(end).ok()?));
        };
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(DiagnosticCode::InvalidLoopRange, message, span)
                    .with_label(label)
                    .with_note(
                        "a loop `for i in a..b` runs once for each i from a up to b - 1, with \
                         a < b <= 65536",
                    ),
            );
        }
        None
    }

    /// Decodes one loop bound. A bound above the u64 range decodes as the
    /// u64 maximum, which is too large.
    fn loop_bound(&mut self, span: Span) -> Option<u64> {
        let literal = IntegerLiteral {
            span,
            magnitude_span: span,
            negative: false,
        };
        let magnitude = self.parse_magnitude(&literal, self.limits.integer_bits)?;
        Some(magnitude.to_u64().unwrap_or(u64::MAX))
    }

    /// Decodes an index literal and checks it against the array's length.
    fn check_index_literal(&mut self, index: &IndexExpression, array: ArrayType) -> Option<u32> {
        let literal = IntegerLiteral {
            span: index.index_span(),
            magnitude_span: index.index_span(),
            negative: false,
        };
        let magnitude = self.parse_magnitude(&literal, self.limits.integer_bits)?;
        let length = array.length();
        let decoded = magnitude
            .to_u64()
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| *value < length);
        if decoded.is_none() && self.begin_report(index.index_span()) {
            let highest = length.saturating_sub(1);
            let spelling = self
                .source
                .slice(index.index_span())
                .map_or_else(String::new, |spelling| {
                    identifier_spelling_for_diagnostic(spelling).to_string()
                });
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::IndexOutOfRange,
                    format!(
                        "index `{spelling}` is out of range for `{}`",
                        CoreType::Array(array)
                    ),
                    index.index_span(),
                )
                .with_label(format!("indices run from 0 through {highest}"))
                .with_note("a literal index must be less than the array's length"),
            );
        }
        decoded
    }

    /// Checks a comparison `left op right` against `expected`, which must be
    /// `Bool`. Both operands have the type of the first typed leaf of the
    /// left operand, or else of the right operand, as a conversion operand
    /// has.
    fn check_comparison(
        &mut self,
        expression: &'ast Expression,
        binary: &'ast BinaryExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let operator = binary.operator.as_str();
        let result_matches = expected == CoreType::Bool;
        if !result_matches && self.begin_report(expression.span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("a comparison gives `Bool`, but `{expected}` is required here"),
                    expression.span,
                )
                .with_label(format!("expected `{expected}`"))
                .with_note("a conditional `if c { a } else { b }` chooses a value by a `Bool`"),
            );
        }
        let Some(leaf) = first_typed_leaf(&binary.left).or_else(|| first_typed_leaf(&binary.right))
        else {
            if self.begin_report(expression.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::UntypedComparison,
                        format!("the operands of `{operator}` have no type of their own"),
                        expression.span,
                    )
                    .with_label("a literal takes its type from where it is used")
                    .with_note(
                        "compare with a typed operand, such as a name, or give the literal a \
                         type with a `let` binding",
                    ),
                );
            }
            return false;
        };
        let Some(operand) = self.leaf_type(leaf, context, scope) else {
            // The leaf's own check reports why it has no type.
            self.check_expression(leaf, expected, context, scope, output);
            return false;
        };
        let defined = match binary.operator {
            BinaryOperator::Equal | BinaryOperator::NotEqual => operand.is_scalar(),
            _ => operand.is_number(),
        };
        if !defined {
            if self.begin_report(binary.operator_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::UnsupportedOperator,
                        format!("`{operator}` is not defined for `{operand}`"),
                        binary.operator_span,
                    )
                    .with_label(format!("the operands have type `{operand}`"))
                    .with_note(if operand.is_scalar() {
                        "`Bool` values are compared with `==` and `!=`; they have no order"
                    } else {
                        "compare elements, such as `x[0] == y[0]`"
                    }),
                );
            }
            return false;
        }
        let left = self.check_expression(&binary.left, operand, context, scope, output);
        if self.halted {
            return false;
        }
        let right = self.check_expression(&binary.right, operand, context, scope, output);
        left && right
            && result_matches
            && self.push_node(
                output,
                expression.span,
                CoreType::Bool,
                CoreNodeKind::Compare {
                    operator: binary.operator,
                    operand,
                },
            )
    }

    /// Checks `if c0 { v0 } else if c1 { v1 } ... else { w }` against
    /// `expected`.
    ///
    /// Each arm is a conditional of its own and takes its number when its
    /// check starts, so conditionals are numbered in source order of their
    /// `if` keywords. Every condition is checked as a `Bool` and every value
    /// against `expected`. The first condition's nodes join the enclosing
    /// expression, followed by the first arm's `choose` node; each later
    /// condition's nodes and its arm's `choose` node form the previous arm's
    /// `else` branch. Arms are checked in a loop, so a chain of any length
    /// adds no recursion.
    fn check_conditional(
        &mut self,
        expression: &'ast Expression,
        conditional: &'ast ConditionalExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let arm_count = conditional.arms.len();
        let mut arms: Vec<(u32, Span, Vec<CoreNode>)> = Vec::new();
        let mut else_branches: Vec<Vec<CoreNode>> = Vec::new();
        if arms.try_reserve_exact(arm_count).is_err()
            || else_branches.try_reserve_exact(arm_count).is_err()
        {
            self.resource_limit(expression.span, "conditional storage allocation failed");
            return false;
        }
        let mut well_formed = true;
        for (position, arm) in conditional.arms.iter().enumerate() {
            if !self.event(arm.keyword_span) {
                return false;
            }
            let Ok(id) = u32::try_from(context.conditionals.len()) else {
                self.resource_limit(
                    arm.keyword_span,
                    "conditional count exceeds the u32 representation limit",
                );
                return false;
            };
            if context.conditionals.try_reserve(1).is_err() {
                self.resource_limit(arm.keyword_span, "conditional storage allocation failed");
                return false;
            }
            context.conditionals.push(None);
            let condition = if position == 0 {
                self.check_expression(&arm.condition, CoreType::Bool, context, scope, output)
            } else {
                let mut branch = BodyOutput {
                    nodes: Vec::new(),
                    call_edges: &mut *output.call_edges,
                };
                let checked = self.check_expression(
                    &arm.condition,
                    CoreType::Bool,
                    context,
                    scope,
                    &mut branch,
                );
                else_branches.push(branch.nodes);
                checked
            };
            if self.halted {
                return false;
            }
            let mut branch = BodyOutput {
                nodes: Vec::new(),
                call_edges: &mut *output.call_edges,
            };
            let value = self.check_expression(&arm.value, expected, context, scope, &mut branch);
            if self.halted {
                return false;
            }
            well_formed = well_formed && condition && value;
            arms.push((id, arm.keyword_span, branch.nodes));
        }
        let mut branch = BodyOutput {
            nodes: Vec::new(),
            call_edges: &mut *output.call_edges,
        };
        let otherwise = self.check_expression(
            &conditional.otherwise,
            expected,
            context,
            scope,
            &mut branch,
        );
        if !(well_formed && otherwise) || self.halted {
            return false;
        }
        else_branches.push(branch.nodes);
        self.record_conditionals(expression, expected, arms, else_branches, context, output)
    }

    /// Fills the table entries of a well-formed conditional's arms and
    /// appends the first arm's `choose` node to `output`.
    fn record_conditionals(
        &mut self,
        expression: &Expression,
        expected: CoreType,
        arms: Vec<(u32, Span, Vec<CoreNode>)>,
        else_branches: Vec<Vec<CoreNode>>,
        context: &mut BodyContext<'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Ok(visible_locals) = u32::try_from(context.binding_types.len()) else {
            self.resource_limit(
                expression.span,
                "binding count exceeds the u32 representation limit",
            );
            return false;
        };
        let spans = arms
            .iter()
            .map(|(id, keyword_span, _)| {
                (
                    *id,
                    self.source
                        .span(keyword_span.start(), expression.span.end())
                        .unwrap_or(expression.span),
                )
            })
            .collect::<Vec<_>>();
        let mut later_arms = spans.iter().skip(1);
        for ((id, _, then_nodes), else_nodes) in arms.into_iter().zip(else_branches) {
            let mut else_branch = BodyOutput {
                nodes: else_nodes,
                call_edges: &mut *output.call_edges,
            };
            if let Some((next, next_span)) = later_arms.next()
                && !self.push_node(
                    &mut else_branch,
                    *next_span,
                    expected,
                    CoreNodeKind::Choose(*next),
                )
            {
                return false;
            }
            let mut loop_scope = Vec::new();
            if loop_scope
                .try_reserve_exact(context.loop_scopes.len())
                .is_err()
            {
                self.resource_limit(expression.span, "conditional scope allocation failed");
                return false;
            }
            loop_scope.extend(context.loop_scopes.iter().map(|scope| scope.id));
            let span = spans
                .iter()
                .find(|(candidate, _)| *candidate == id)
                .map_or(expression.span, |(_, span)| *span);
            let Some(entry) = usize::try_from(id)
                .ok()
                .and_then(|id| context.conditionals.get_mut(id))
            else {
                self.resource_limit(
                    expression.span,
                    "semantic conditional table is inconsistent",
                );
                return false;
            };
            *entry = Some(CoreConditional {
                span,
                ty: expected,
                visible_locals,
                scope: loop_scope,
                then_branch: CoreExpression { nodes: then_nodes },
                else_branch: CoreExpression {
                    nodes: else_branch.nodes,
                },
            });
        }
        let Some((first, _)) = spans.first() else {
            self.resource_limit(
                expression.span,
                "semantic conditional table is inconsistent",
            );
            return false;
        };
        self.push_node(
            output,
            expression.span,
            expected,
            CoreNodeKind::Choose(*first),
        )
    }

    #[cold]
    #[inline(never)]
    fn report_integer_for_bool(&mut self, span: Span) {
        if !self.begin_report(span) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                "an integer literal cannot have type `Bool`",
                span,
            )
            .with_label("expected `Bool`")
            .with_note("the `Bool` values are written `true` and `false`"),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_scalar_for_array(&mut self, span: Span, what: &str, expected: CoreType) {
        if !self.begin_report(span) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("{what} cannot have type `{expected}`"),
                span,
            )
            .with_label(format!("expected `{expected}`"))
            .with_note("an array value is written `[e0, e1, ...]`, one element per index"),
        );
    }

    fn check_call(
        &mut self,
        expression: &'ast Expression,
        call: &'ast CallExpression,
        expected: CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let callee_name = &call.callee;
        let spelling = identifier_spelling_for_diagnostic(&callee_name.text);
        let declaration =
            first_declaration(scope.declarations, FunctionKind::Spec, &callee_name.text);
        let signature =
            declaration.and_then(|entry| scope.signatures.get(entry.source_index)?.as_ref());
        let Some(signature) = signature else {
            if self.begin_report(callee_name.span) {
                let mut diagnostic = if let Some(entry) = declaration {
                    Diagnostic::error(
                        DiagnosticCode::UnknownFunction,
                        format!(
                            "`spec` function `{spelling}` has no typed body and cannot be called"
                        ),
                        callee_name.span,
                    )
                    .with_label("no value to call")
                    .with_secondary_span(entry.span, "declared without a result type here")
                } else {
                    Diagnostic::error(
                        DiagnosticCode::UnknownFunction,
                        format!("no typed `spec` function named `{spelling}` in this module"),
                        callee_name.span,
                    )
                    .with_label("unknown function")
                };
                diagnostic =
                    if first_declaration(scope.declarations, FunctionKind::Impl, &callee_name.text)
                        .is_some()
                    {
                        diagnostic.with_note(
                            "`impl` functions have no semantics yet and cannot be called",
                        )
                    } else {
                        diagnostic
                            .with_note("calls name a typed `spec` declared in the same module")
                    };
                self.diagnostics.push(diagnostic);
            }
            return false;
        };
        // An unresolved callee type was reported at the callee's declaration.
        if !signature.is_complete() {
            self.record_call_edge(context, signature, expression.span, output);
            return false;
        }
        if signature.parameters.len() != call.arguments.len() {
            if self.begin_report(expression.span) {
                let expected_count = signature.parameters.len();
                let supplied = call.arguments.len();
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::ArgumentCountMismatch,
                        format!(
                            "`{spelling}` takes {expected_count} {} but {supplied} {} supplied",
                            if expected_count == 1 {
                                "argument"
                            } else {
                                "arguments"
                            },
                            if supplied == 1 { "was" } else { "were" },
                        ),
                        expression.span,
                    )
                    .with_label("wrong number of arguments")
                    .with_note("every parameter receives exactly one argument"),
                );
            }
            self.record_call_edge(context, signature, expression.span, output);
            return false;
        }
        let Some(result_type) = signature.result_type else {
            return false;
        };
        // A result-type mismatch is reported at the call before its arguments
        // are checked; arguments are checked against the callee's parameter
        // types, so their errors are independent and are still reported.
        let result_matches = result_type == expected;
        if !result_matches && self.begin_report(expression.span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!(
                        "`{spelling}` returns `{result_type}`, but `{expected}` is required here"
                    ),
                    expression.span,
                )
                .with_label(format!("expected `{expected}`"))
                .with_note("Orange has no implicit conversions between types"),
            );
        }
        let mut arguments_checked = true;
        for (argument, parameter_type) in call.arguments.iter().zip(&signature.parameters) {
            let Some(parameter_type) = *parameter_type else {
                return false;
            };
            arguments_checked &=
                self.check_expression(argument, parameter_type, context, scope, output);
            if self.halted {
                return false;
            }
        }
        if !self.record_call_edge(context, signature, expression.span, output)
            || !result_matches
            || !arguments_checked
        {
            return false;
        }
        let Ok(arguments) = u32::try_from(call.arguments.len()) else {
            self.resource_limit(
                expression.span,
                "argument count exceeds the u32 representation limit",
            );
            return false;
        };
        self.push_node(
            output,
            expression.span,
            expected,
            CoreNodeKind::Call {
                function: signature.id,
                arguments,
            },
        )
    }

    /// Adds the call graph edge for an examined call to a typed `spec`. The
    /// edge does not depend on the call's types, so a cycle is reported even
    /// through a call that is also wrong. Returns whether the edge was stored.
    fn record_call_edge(
        &mut self,
        context: &BodyContext<'ast>,
        callee: &Signature,
        span: Span,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        if !(self.reserve_call_edge_slot)(output.call_edges) {
            self.resource_limit(span, "call graph storage allocation failed");
            return false;
        }
        output.call_edges.push(CallEdge {
            caller: context.id,
            callee: callee.id,
            span,
        });
        true
    }

    fn unary_is_defined(&mut self, unary: &UnaryExpression, expected: CoreType) -> bool {
        let defined = match unary.operator {
            UnaryOperator::Negate => expected == CoreType::Int,
            UnaryOperator::Complement => expected.word_bits().is_some(),
            UnaryOperator::Not => expected == CoreType::Bool,
        };
        if !defined && self.begin_report(unary.operator_span) {
            let operator = unary.operator.as_str();
            let note = match (unary.operator, expected.word_bits()) {
                _ if !expected.is_scalar() => String::from(ARRAY_OPERATOR_NOTE),
                _ if expected == CoreType::Bool => String::from(BOOL_OPERATOR_NOTE),
                (UnaryOperator::Negate, Some(bits)) => {
                    format!("write `0 - x` for negation modulo 2^{bits}")
                }
                (UnaryOperator::Not, Some(_)) => {
                    String::from("`!` negates a `Bool`; `~` is the bitwise complement of a word")
                }
                (UnaryOperator::Not, None) => {
                    String::from("`!` negates a `Bool`; `-` negates an `Int`")
                }
                _ => String::from("bitwise operators apply only to `Word[n]` values"),
            };
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    format!("prefix `{operator}` is not defined for `{expected}`"),
                    unary.operator_span,
                )
                .with_label(format!("`{expected}` is required here"))
                .with_note(note),
            );
        }
        defined
    }

    fn binary_is_defined(&mut self, binary: &BinaryExpression, expected: CoreType) -> bool {
        let defined = match binary.operator {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide
            | BinaryOperator::Remainder => expected.is_number(),
            BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Xor
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::RotateLeft
            | BinaryOperator::RotateRight => expected.word_bits().is_some(),
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => expected == CoreType::Bool,
            // A comparison is checked by `check_comparison`.
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => false,
        };
        if !defined && self.begin_report(binary.operator_span) {
            let operator = binary.operator.as_str();
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    format!("`{operator}` is not defined for `{expected}`"),
                    binary.operator_span,
                )
                .with_label(format!("`{expected}` is required here"))
                .with_note(if !expected.is_scalar() {
                    ARRAY_OPERATOR_NOTE
                } else if expected == CoreType::Bool {
                    BOOL_OPERATOR_NOTE
                } else if binary.operator.is_logical() {
                    "`&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators \
                     on words"
                } else if binary.operator.is_shift_or_rotation() {
                    "shifts and rotations apply only to `Word[n]` values"
                } else {
                    "bitwise operators apply only to `Word[n]` values"
                }),
            );
        }
        defined
    }

    fn check_shift_amount(&mut self, binary: &BinaryExpression, expected: CoreType) -> Option<u32> {
        let bits = expected.word_bits()?;
        let amount = &binary.right;
        let decoded = match &amount.kind {
            ExpressionKind::Literal(literal) if !literal.negative => {
                // An amount literal keeps the literal event accounting; an
                // oversized magnitude is reported by decoding alone.
                if !self.event(literal.span) {
                    return None;
                }
                let magnitude = self.parse_magnitude(literal, self.limits.integer_bits)?;
                magnitude
                    .to_u64()
                    .filter(|value| *value < u64::from(bits))
                    .and_then(|value| u32::try_from(value).ok())
            }
            _ => None,
        };
        if decoded.is_none() && self.begin_report(amount.span) {
            let operator = binary.operator.as_str();
            let highest = bits.saturating_sub(1);
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::InvalidShiftAmount,
                    format!(
                        "`{operator}` on `{expected}` needs an amount from 0 through {highest}"
                    ),
                    amount.span,
                )
                .with_label("amount must be an unsigned integer literal")
                .with_note(
                    "amounts are fixed literals; variable amounts are not part of Orange 2026",
                ),
            );
        }
        decoded
    }

    fn push_node(
        &mut self,
        output: &mut BodyOutput<'_>,
        span: Span,
        ty: CoreType,
        kind: CoreNodeKind,
    ) -> bool {
        if !(self.reserve_core_node_slot)(&mut output.nodes) {
            self.resource_limit(span, "typed Core expression storage allocation failed");
            return false;
        }
        output.nodes.push(CoreNode { span, ty, kind });
        true
    }

    /// Reports every call cycle among typed specifications.
    ///
    /// A depth-first search in function-ID order examines each function's call
    /// edges in checking order; each edge that returns to a function still on
    /// the search path closes a cycle and is reported once at that call. The
    /// search consumes no semantic events: it visits each function and each
    /// already counted call exactly once.
    fn check_call_graph(&mut self, signatures: &[Option<Signature>], edges: &[CallEdge]) {
        let function_count = signatures.iter().flatten().count();
        let mut names = Vec::new();
        let mut offsets = Vec::new();
        let mut targets: Vec<(CoreFunctionId, usize)> = Vec::new();
        let mut state = Vec::new();
        let mut path = Vec::new();
        if names.try_reserve_exact(function_count).is_err()
            || offsets
                .try_reserve_exact(function_count.saturating_add(1))
                .is_err()
            || targets.try_reserve_exact(edges.len()).is_err()
            || state.try_reserve_exact(function_count).is_err()
            || path.try_reserve_exact(function_count).is_err()
        {
            self.resource_limit(self.ast.module.span, "call graph storage allocation failed");
            return;
        }
        names.extend(
            self.ast
                .module
                .functions
                .iter()
                .zip(signatures)
                .filter(|(_, signature)| signature.is_some())
                .map(|(function, _)| &function.name),
        );
        // Group edges by caller; within one caller, edges keep the order in
        // which their calls finished checking.
        targets.extend(
            edges
                .iter()
                .enumerate()
                .map(|(index, edge)| (edge.caller, index)),
        );
        targets.sort_unstable();
        offsets.push(0_usize);
        let mut cursor = 0_usize;
        for index in 0..function_count {
            while targets
                .get(cursor)
                .is_some_and(|(caller, _)| usize::try_from(caller.index()).ok() == Some(index))
            {
                cursor = cursor.saturating_add(1);
            }
            offsets.push(cursor);
        }
        state.resize(function_count, VisitState::Unvisited);

        for root in 0..function_count {
            if state.get(root) != Some(&VisitState::Unvisited) {
                continue;
            }
            path.push((root, offsets.get(root).copied().unwrap_or(0)));
            if let Some(slot) = state.get_mut(root) {
                *slot = VisitState::OnPath;
            }
            while let Some((node, next_edge)) = path.last().copied() {
                let end = offsets.get(node.saturating_add(1)).copied().unwrap_or(0);
                if next_edge >= end {
                    path.pop();
                    if let Some(slot) = state.get_mut(node) {
                        *slot = VisitState::Done;
                    }
                    continue;
                }
                if let Some(top) = path.last_mut() {
                    top.1 = next_edge.saturating_add(1);
                }
                let Some(edge) = targets
                    .get(next_edge)
                    .and_then(|(_, index)| edges.get(*index))
                else {
                    self.resource_limit(self.ast.module.span, "call graph index is inconsistent");
                    return;
                };
                let Ok(target) = usize::try_from(edge.callee.index()) else {
                    self.resource_limit(edge.span, "call graph index is inconsistent");
                    return;
                };
                match state.get(target) {
                    Some(VisitState::Unvisited) => {
                        if let Some(slot) = state.get_mut(target) {
                            *slot = VisitState::OnPath;
                        }
                        path.push((target, offsets.get(target).copied().unwrap_or(0)));
                    }
                    Some(VisitState::OnPath) => {
                        self.report_cycle(edge, target, &path, &names);
                        if self.halted {
                            return;
                        }
                    }
                    Some(VisitState::Done) => {}
                    None => {
                        self.resource_limit(edge.span, "call graph index is inconsistent");
                        return;
                    }
                }
            }
        }
    }

    fn report_cycle(
        &mut self,
        edge: &CallEdge,
        target: usize,
        path: &[(usize, usize)],
        names: &[&Identifier],
    ) {
        if !self.begin_report(edge.span) {
            return;
        }
        let start = path
            .iter()
            .position(|(node, _)| *node == target)
            .unwrap_or(0);
        let cycle = path.get(start..).unwrap_or_default();
        let name_of = |index: usize| {
            names.get(index).map_or_else(String::new, |name| {
                identifier_spelling_for_diagnostic(&name.text).to_string()
            })
        };
        let target_name = name_of(target);
        let message = if cycle.len() <= 1 {
            format!("`{target_name}` calls itself")
        } else {
            let mut route = String::new();
            for (position, (node, _)) in cycle.iter().enumerate() {
                if position >= MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC {
                    route.push_str(" -> ...");
                    break;
                }
                if position != 0 {
                    route.push_str(" -> ");
                }
                route.push('`');
                route.push_str(&name_of(*node));
                route.push('`');
            }
            format!("call cycle {route} -> `{target_name}`")
        };
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::CallCycle, message, edge.span)
                .with_label("this call closes the cycle")
                .with_note(
                    "a `spec` may not depend on itself; recursion is not part of Orange 2026",
                ),
        );
    }

    fn construct_core(
        &mut self,
        pending: Vec<PendingFunction>,
        reserve_core_functions: impl FnOnce(&mut Vec<CoreFunction>, usize) -> bool,
    ) -> Option<CoreModule> {
        let module_name =
            self.copy_core_name(&self.ast.module.name.text, self.ast.module.name.span)?;
        // The module itself is one Core node, including for an empty Core.
        if !self.record_core_node(self.ast.module.span) {
            return None;
        }
        let capacity = pending.len();
        let mut functions = Vec::new();
        if !reserve_core_functions(&mut functions, capacity) {
            self.resource_limit(
                self.ast.module.span,
                "typed Core function storage allocation failed",
            );
            return None;
        }
        for pending_function in pending {
            // Each function contributes one function node, one result-type
            // node, one node per parameter type, and its expression nodes;
            // each binding contributes one binding node, one type node, and
            // its expression nodes.
            // Each loop contributes one loop node, one accumulator-type
            // node, and its step's nodes.
            let node_count = pending_function.locals.iter().fold(
                pending_function
                    .parameters
                    .len()
                    .saturating_add(pending_function.nodes.len())
                    .saturating_add(2),
                |count, local| {
                    count
                        .saturating_add(local.value.nodes.len())
                        .saturating_add(2)
                },
            );
            let node_count = pending_function
                .loops
                .iter()
                .fold(node_count, |count, r#loop| {
                    count
                        .saturating_add(r#loop.step.nodes.len())
                        .saturating_add(2)
                });
            // Each conditional contributes one table entry and its branches'
            // nodes.
            let node_count =
                pending_function
                    .conditionals
                    .iter()
                    .fold(node_count, |count, conditional| {
                        count
                            .saturating_add(conditional.then_branch.nodes.len())
                            .saturating_add(conditional.else_branch.nodes.len())
                            .saturating_add(1)
                    });
            for _ in 0..node_count {
                if !self.record_core_node(pending_function.span) {
                    return None;
                }
            }
            let Some(id) = CoreFunctionId::from_index(functions.len()) else {
                self.resource_limit(
                    pending_function.span,
                    "Core function identity exceeds the u32 representation limit",
                );
                return None;
            };
            functions.push(CoreFunction {
                id,
                span: pending_function.span,
                name: pending_function.name,
                name_span: pending_function.name_span,
                parameters: pending_function.parameters,
                result_type: pending_function.result_type,
                locals: pending_function.locals,
                body: CoreExpression {
                    nodes: pending_function.nodes,
                },
                loops: pending_function.loops,
                conditionals: pending_function.conditionals,
            });
        }
        Some(CoreModule {
            span: self.ast.module.span,
            name: module_name,
            functions,
        })
    }

    fn analyze_type(&mut self, syntax: &TypeSyntax, role: &str) -> Option<CoreType> {
        // The identifier and optional width are distinct parsed-type components.
        if !self.event(syntax.name.span) {
            return None;
        }
        if let Some(width_span) = syntax.width_span
            && !self.event(width_span)
        {
            return None;
        }
        if let Some(length_span) = syntax.length_span
            && !self.event(length_span)
        {
            return None;
        }
        match classify_type(self.source, syntax) {
            TypeClass::Resolved(ty) => Some(ty),
            TypeClass::UnsupportedArrayLength(length_span) => {
                self.report_unsupported_array_length(length_span);
                None
            }
            TypeClass::UnsupportedWordWidth(width_span) => {
                if self.begin_report(width_span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedWordWidth,
                            "`Word` width must be exactly 8, 16, 32, or 64",
                            width_span,
                        )
                        .with_label("unsupported word width")
                        .with_note("word widths do not coerce, truncate, or wrap"),
                    );
                }
                None
            }
            TypeClass::MissingWordWidth => {
                let span = syntax.name.span;
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedWordWidth,
                            "`Word` requires an exact width of 8, 16, 32, or 64",
                            span,
                        )
                        .with_label("missing word width")
                        .with_note("write the width in decimal, as in `Word[32]`"),
                    );
                }
                None
            }
            TypeClass::Unsupported => {
                if self.begin_report(syntax.span) {
                    let name = identifier_spelling_for_diagnostic(&syntax.name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedType,
                            format!("unsupported {role} `{name}`"),
                            syntax.span,
                        )
                        .with_label(format!("the admitted types are {ADMITTED_TYPES}"))
                        .with_note(
                            "types are resolved contextually and never inferred by spelling similarity",
                        ),
                    );
                }
                None
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn report_unsupported_array_length(&mut self, span: Span) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedArrayLength,
                    format!(
                        "an array length must be a decimal integer from 1 through \
                         {MAX_ARRAY_LENGTH}"
                    ),
                    span,
                )
                .with_label("unsupported array length")
                .with_note(
                    "write the length in decimal without leading zeros, as in `Word[32]^16`",
                ),
            );
        }
    }

    fn analyze_literal(
        &mut self,
        expected: CoreType,
        literal: &IntegerLiteral,
    ) -> Option<CoreValue> {
        // One literal event precedes shared exact-magnitude decoding. Word sign
        // and range classification follows only after the magnitude is valid.
        if !self.event(literal.span) {
            return None;
        }
        let magnitude = self.parse_magnitude(literal, self.limits.integer_bits)?;
        let Some(maximum) = word_maximum(expected) else {
            return Some(CoreValue::Int(ExactInteger::new(
                literal.negative,
                magnitude,
            )));
        };
        if literal.negative {
            if self.begin_report(literal.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::NegativeWordLiteral,
                        format!("`{expected}` literals cannot be negative"),
                        literal.span,
                    )
                    .with_label(format!(
                        "negative value is outside the range 0 through {maximum}"
                    ))
                    .with_note("fixed-width words do not wrap or coerce negative integers"),
                );
            }
            return None;
        }
        let value = magnitude.to_u64().filter(|value| *value <= maximum);
        if let Some(value) = value.and_then(|value| CoreValue::word_from_u64(expected, value)) {
            Some(value)
        } else {
            if self.begin_report(literal.magnitude_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::WordLiteralOutOfRange,
                        format!("literal is outside the range of `{expected}`"),
                        literal.magnitude_span,
                    )
                    .with_label(format!("expected a value from 0 through {maximum}"))
                    .with_note("fixed-width words do not truncate or wrap out-of-range integers"),
                );
            }
            None
        }
    }
    fn parse_magnitude(&mut self, literal: &IntegerLiteral, bit_limit: usize) -> Option<Magnitude> {
        let Some(spelling) = self.source.slice(literal.magnitude_span) else {
            self.resource_limit(
                literal.span,
                "integer literal span does not belong to the analyzed source",
            );
            return None;
        };
        // Prefix inspection is one event whether the decimal default or an
        // explicit binary/hexadecimal prefix is selected.
        if !self.event(literal.magnitude_span) {
            return None;
        }
        let (radix, digits) = if let Some(digits) = spelling
            .strip_prefix("0b")
            .or_else(|| spelling.strip_prefix("0B"))
        {
            (2, digits)
        } else if let Some(digits) = spelling
            .strip_prefix("0x")
            .or_else(|| spelling.strip_prefix("0X"))
        {
            (16, digits)
        } else {
            (10, spelling)
        };

        let mut magnitude = Magnitude::zero();
        let mut significant = false;
        for character in digits.chars() {
            if character == '_' {
                continue;
            }
            let Some(digit) = character.to_digit(radix) else {
                self.resource_limit(
                    literal.magnitude_span,
                    "semantic analysis received a malformed integer AST",
                );
                return None;
            };
            significant |= digit != 0;
            if significant && !self.event(literal.magnitude_span) {
                return None;
            }
            if !magnitude.multiply_add_with_reservation(radix, digit, self.reserve_magnitude_limb) {
                self.resource_limit(
                    literal.magnitude_span,
                    "exact integer magnitude storage allocation failed",
                );
                return None;
            }
            if magnitude.bit_len() > bit_limit {
                if self.begin_report(literal.magnitude_span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::IntegerMagnitudeLimit,
                            format!(
                                "integer magnitude exceeds the {}-significant-bit limit",
                                self.limits.integer_bits
                            ),
                            literal.magnitude_span,
                        )
                        .with_label("exact integer is too large for this semantic fragment")
                        .with_note("the literal is rejected rather than truncated or approximated"),
                    );
                }
                return None;
            }
        }
        Some(magnitude)
    }

    fn copy_core_name(&mut self, source: &str, span: Span) -> Option<String> {
        let mut name = String::new();
        if !(self.reserve_core_name)(&mut name, source.len()) {
            self.resource_limit(span, "typed Core name storage allocation failed");
            return None;
        }
        name.push_str(source);
        Some(name)
    }

    fn event(&mut self, span: Span) -> bool {
        if self.halted {
            return false;
        }
        if self.events >= self.limits.events {
            self.resource_limit(span, "semantic event budget exhausted");
            return false;
        }
        self.events = self.events.saturating_add(1);
        true
    }

    fn record_core_node(&mut self, span: Span) -> bool {
        if !self.event(span) {
            return false;
        }
        if self.core_nodes >= self.limits.nodes {
            self.resource_limit(span, "typed Core node budget exhausted");
            return false;
        }
        self.core_nodes = self.core_nodes.saturating_add(1);
        true
    }

    fn begin_report(&mut self, span: Span) -> bool {
        if self.halted {
            return false;
        }
        // Every ordinary diagnostic emission attempt consumes one semantic event,
        // including attempts suppressed by the diagnostic budget.
        if !self.event(span) {
            return false;
        }
        if self.ordinary_diagnostics < self.limits.diagnostics {
            self.ordinary_diagnostics = self.ordinary_diagnostics.saturating_add(1);
            true
        } else if !self.diagnostic_limit_reported {
            self.diagnostic_limit_reported = true;
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TooManySemanticErrors,
                    "too many semantic errors; further errors are suppressed",
                    span,
                )
                .with_label("semantic diagnostic limit reached")
                .with_note(format!(
                    "at most {} ordinary semantic diagnostics are retained per source",
                    self.limits.diagnostics
                )),
            );
            false
        } else {
            false
        }
    }

    fn resource_limit(&mut self, span: Span, detail: &str) {
        if !self.resource_limit_reported {
            self.resource_limit_reported = true;
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::SemanticResourceLimit,
                    "semantic analysis resource limit exceeded",
                    span,
                )
                .with_label(detail)
                .with_note("semantic analysis stopped without producing Core"),
            );
        }
        self.halted = true;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum VisitState {
    Unvisited,
    OnPath,
    Done,
}

struct DiagnosticIdentifierSpelling<'text> {
    prefix: &'text str,
    total_bytes: Option<usize>,
}

impl fmt::Display for DiagnosticIdentifierSpelling<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.prefix)?;
        if let Some(total_bytes) = self.total_bytes {
            write!(formatter, "...<{total_bytes} bytes total>")?;
        }
        Ok(())
    }
}

fn identifier_spelling_for_diagnostic(text: &str) -> DiagnosticIdentifierSpelling<'_> {
    if text.len() <= MAX_IDENTIFIER_BYTES_IN_DIAGNOSTIC {
        return DiagnosticIdentifierSpelling {
            prefix: text,
            total_bytes: None,
        };
    }

    let mut prefix_end = MAX_IDENTIFIER_BYTES_IN_DIAGNOSTIC;
    while !text.is_char_boundary(prefix_end) {
        prefix_end = prefix_end.saturating_sub(1);
    }
    let prefix = text.get(..prefix_end).unwrap_or_default();
    DiagnosticIdentifierSpelling {
        prefix,
        total_bytes: Some(text.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::Edition;
    use crate::lexer::lex;
    use crate::parser::parse;
    use crate::source::{SourceId, SourceMap};

    struct Fixture {
        sources: SourceMap,
        id: SourceId,
        ast: SyntaxTree,
    }

    impl Fixture {
        fn new(text: impl Into<String>) -> Self {
            let mut sources = SourceMap::new();
            let id = sources.add("semantic.or", text.into()).unwrap();
            let ast = {
                let source = sources.get(id).unwrap();
                let lexed = lex(source, Edition::E2026);
                assert_eq!(lexed.diagnostics(), []);
                let parsed = parse(source, &lexed);
                assert_eq!(parsed.diagnostics(), []);
                parsed.into_ast().unwrap()
            };
            Self { sources, id, ast }
        }

        fn source(&self) -> &SourceFile {
            self.sources.get(self.id).unwrap()
        }

        fn analyze(&self) -> AnalysisResult {
            analyze(self.source(), &self.ast)
        }

        fn analyze_with(&self, limits: Limits) -> AnalysisResult {
            Analyzer::new(self.source(), &self.ast, limits).run()
        }
    }

    fn typed_body_mut(ast: &mut SyntaxTree) -> &mut TypedBody {
        match &mut ast.module.functions.first_mut().unwrap().body {
            FunctionBody::Typed(body) => body,
            FunctionBody::Empty => unreachable!(),
        }
    }

    fn body_literal(body: &TypedBody) -> &IntegerLiteral {
        match &body.expression.kind {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!(),
        }
    }

    fn body_literal_mut(ast: &mut SyntaxTree) -> &mut IntegerLiteral {
        match &mut typed_body_mut(ast).expression.kind {
            ExpressionKind::Literal(literal) => literal,
            _ => unreachable!(),
        }
    }

    fn power_of_two_decimal(exponent: usize) -> String {
        let mut digits = vec![1_u8];
        for _ in 0..exponent {
            let mut carry = 0_u8;
            for digit in &mut digits {
                let doubled = *digit * 2 + carry;
                *digit = doubled % 10;
                carry = doubled / 10;
            }
            if carry != 0 {
                digits.push(carry);
            }
        }
        digits
            .iter()
            .rev()
            .map(|digit| char::from(b'0' + *digit))
            .collect()
    }

    fn reference_decimal_from_bits(bits: &str) -> String {
        let mut digits = vec![0_u8];
        for bit in bits.bytes() {
            let mut carry = bit - b'0';
            for digit in &mut digits {
                let doubled = *digit * 2 + carry;
                *digit = doubled % 10;
                carry = doubled / 10;
            }
            if carry != 0 {
                digits.push(carry);
            }
        }
        digits
            .iter()
            .rev()
            .map(|digit| char::from(b'0' + *digit))
            .collect()
    }

    fn reference_hexadecimal_from_bits(bits: &str) -> String {
        let padding = bits.len().next_multiple_of(4) - bits.len();
        let padded = format!("{}{}", "0".repeat(padding), bits);
        padded
            .as_bytes()
            .chunks(4)
            .map(|nibble| {
                let value = nibble
                    .iter()
                    .fold(0_u32, |value, bit| value * 2 + u32::from(*bit - b'0'));
                char::from_digit(value, 16).unwrap()
            })
            .collect()
    }

    fn group_digits(digits: &str, width: usize) -> String {
        let separators = digits.len().saturating_sub(1) / width;
        let mut grouped = String::with_capacity(digits.len() + separators);
        for (index, digit) in digits.char_indices() {
            if index != 0 && (digits.len() - index).is_multiple_of(width) {
                grouped.push('_');
            }
            grouped.push(digit);
        }
        grouped
    }

    fn deterministic_bits(bit_length: usize, state: &mut u64) -> String {
        let mut bits = String::with_capacity(bit_length);
        bits.push('1');
        for _ in 1..bit_length {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            bits.push(if *state & 1 == 0 { '0' } else { '1' });
        }
        bits
    }

    #[test]
    fn rejects_same_index_syntax_trees_from_another_source_map_repeatably() {
        let text = "edition 2026; module values { spec value() -> Int { 1 } }\n";
        let first = Fixture::new(text);
        let second = Fixture::new(text);

        let first_result = analyze(second.source(), &first.ast);
        let second_result = analyze(second.source(), &first.ast);

        assert_eq!(first_result, second_result);
        assert!(first_result.core.is_none());
        assert_eq!(first_result.diagnostics.len(), 1);
        assert_eq!(
            first_result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput
        );
        assert_eq!(
            first_result.diagnostics[0].primary_span().source(),
            second.source().id()
        );
        assert!(first_result.diagnostics[0].primary_span().is_empty());
        assert_eq!(
            first_result.diagnostics[0].primary_span().start().bytes(),
            0
        );
        assert_eq!(
            crate::diagnostic::render_diagnostics(&second.sources, &first_result.diagnostics),
            concat!(
                "error[ORC0210]: semantic analysis received a syntax tree owned by another source\n",
                " --> semantic.or:1:1\n",
                "  |\n",
                "1 | edition 2026; module values { spec value() -> Int { 1 } }\n",
                "  | ^ analysis stopped at this source boundary\n",
                "  = note: parse and analyze each syntax tree with the same source file\n",
            )
        );
    }

    #[test]
    fn rejects_every_foreign_nested_span_even_when_the_root_belongs_to_the_source() {
        let text = "edition 2026; module values { spec value() -> Word[8] { 1 } }\n";
        let first = Fixture::new(text);
        let second = Fixture::new(text);
        let foreign_function = second.ast.module.functions.first().unwrap();
        let foreign_body = match &foreign_function.body {
            FunctionBody::Typed(body) => body,
            FunctionBody::Empty => unreachable!(),
        };

        macro_rules! foreign_case {
            ($mutate:expr) => {{
                let mut ast = first.ast.clone();
                $mutate(&mut ast);
                ast
            }};
        }
        let cases = [
            foreign_case!(|ast: &mut SyntaxTree| ast.edition.span = second.ast.edition.span),
            foreign_case!(
                |ast: &mut SyntaxTree| ast.edition.value_span = second.ast.edition.value_span
            ),
            foreign_case!(|ast: &mut SyntaxTree| ast.module.span = second.ast.module.span),
            foreign_case!(|ast: &mut SyntaxTree| ast.module.name.span = second.ast.module.name.span),
            foreign_case!(
                |ast: &mut SyntaxTree| ast.module.functions.first_mut().unwrap().span =
                    foreign_function.span
            ),
            foreign_case!(|ast: &mut SyntaxTree| ast
                .module
                .functions
                .first_mut()
                .unwrap()
                .name
                .span = foreign_function.name.span),
            foreign_case!(|ast: &mut SyntaxTree| typed_body_mut(ast).span = foreign_body.span),
            foreign_case!(
                |ast: &mut SyntaxTree| typed_body_mut(ast).result_type.span =
                    foreign_body.result_type.span
            ),
            foreign_case!(
                |ast: &mut SyntaxTree| typed_body_mut(ast).result_type.name.span =
                    foreign_body.result_type.name.span
            ),
            foreign_case!(
                |ast: &mut SyntaxTree| typed_body_mut(ast).result_type.width_span =
                    foreign_body.result_type.width_span
            ),
            foreign_case!(|ast: &mut SyntaxTree| typed_body_mut(ast).expression.span =
                foreign_body.expression.span),
            foreign_case!(
                |ast: &mut SyntaxTree| body_literal_mut(ast).span = body_literal(foreign_body).span
            ),
            foreign_case!(
                |ast: &mut SyntaxTree| body_literal_mut(ast).magnitude_span =
                    body_literal(foreign_body).magnitude_span
            ),
        ];

        for (case_index, ast) in cases.iter().enumerate() {
            assert_eq!(ast.span.source(), first.source().id(), "case {case_index}");

            let first_result = analyze(first.source(), ast);
            let second_result = analyze(first.source(), ast);

            assert_eq!(first_result, second_result, "case {case_index}");
            assert!(first_result.core.is_none(), "case {case_index}");
            assert_eq!(first_result.diagnostics.len(), 1, "case {case_index}");
            assert_eq!(
                first_result.diagnostics[0].code(),
                DiagnosticCode::InvalidSemanticInput,
                "case {case_index}"
            );
            assert_eq!(
                first_result.diagnostics[0].primary_span().source(),
                first.source().id(),
                "case {case_index}"
            );
        }
    }

    #[test]
    fn foreign_input_diagnostic_reservation_failure_remains_fail_closed() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");

        let result = invalid_semantic_input(fixture.source(), |_| false);

        assert!(result.has_errors());
        assert!(result.core().is_none());
        assert!(result.diagnostics().is_empty());
        assert_eq!(result.diagnostics.capacity(), 0);
    }

    #[test]
    fn rejects_another_file_from_the_same_source_map() {
        let text = "edition 2026; module values { spec value() -> Int { 1 } }\n";
        let mut sources = SourceMap::new();
        let first_id = sources.add("first.or", text).unwrap();
        let second_id = sources.add("second.or", text).unwrap();
        let ast = {
            let first = sources.get(first_id).unwrap();
            let lexed = lex(first, Edition::E2026);
            parse(first, &lexed).into_ast().unwrap()
        };

        let result = analyze(sources.get(second_id).unwrap(), &ast);

        assert!(result.core.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput
        );
        assert_eq!(result.diagnostics[0].primary_span().source(), second_id);
    }

    #[test]
    fn production_limits_match_the_s3a_specification() {
        assert_eq!(MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE, 100);
        assert_eq!(MAX_CORE_NODES_PER_SOURCE, 262_144);
        assert_eq!(MAX_SEMANTIC_EVENTS_PER_SOURCE, 1_048_576);
        assert_eq!(MAX_INTEGER_BITS, 16_384);
        assert_eq!(Limits::DEFAULT.diagnostics, 100);
        assert_eq!(Limits::DEFAULT.nodes, 262_144);
        assert_eq!(Limits::DEFAULT.events, 1_048_576);
        assert_eq!(Limits::DEFAULT.integer_bits, 16_384);
    }

    #[test]
    fn exact_ints_accept_every_sign_class_in_every_radix() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec decimal_positive() -> Int { 1_234_567_890 }\n",
            "  spec decimal_zero() -> Int { 0 }\n",
            "  spec decimal_negative() -> Int { -10 }\n",
            "  spec decimal_negative_zero() -> Int { -0 }\n",
            "  spec binary_positive() -> Int { 0b1010_0101 }\n",
            "  spec binary_zero() -> Int { 0b0 }\n",
            "  spec binary_negative() -> Int { -0b1010_0101 }\n",
            "  spec binary_negative_zero() -> Int { -0B0 }\n",
            "  spec hexadecimal_positive() -> Int { 0Xdead_BEEF }\n",
            "  spec hexadecimal_zero() -> Int { 0x0 }\n",
            "  spec hexadecimal_negative() -> Int { -0x2a }\n",
            "  spec hexadecimal_negative_zero() -> Int { -0X0 }\n",
            "}\n",
        ));
        let result = fixture.analyze();
        assert_eq!(result.diagnostics, []);
        let values: Vec<_> = result
            .core
            .unwrap()
            .functions
            .iter()
            .map(|function| function.body.literal().unwrap().to_string())
            .collect();
        assert_eq!(
            values,
            [
                "1234567890",
                "0",
                "-10",
                "0",
                "165",
                "0",
                "-165",
                "0",
                "3735928559",
                "0",
                "-42",
                "0",
            ]
        );
    }

    #[test]
    fn integer_decoding_matches_a_deterministic_u128_reference_corpus() {
        let mut values = vec![
            0,
            1,
            u128::from(u32::MAX),
            1_u128 << 32,
            u128::from(u64::MAX),
            1_u128 << 64,
            u128::MAX,
        ];
        let mut state = 0x6a09_e667_f3bc_c908_bb67_ae85_84ca_a73b_u128;
        for _ in 0..32 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            values.push(state);
        }

        let mut source = String::from("edition 2026; module reference_corpus {\n");
        let mut expected = Vec::with_capacity(values.len() * 4);
        for (index, value) in values.into_iter().enumerate() {
            source.push_str(&format!("  spec decimal_{index}() -> Int {{ {value} }}\n"));
            expected.push(value.to_string());

            source.push_str(&format!(
                "  spec binary_{index}() -> Int {{ 0b{value:b} }}\n"
            ));
            expected.push(value.to_string());

            source.push_str(&format!(
                "  spec hexadecimal_{index}() -> Int {{ 0X{value:X} }}\n"
            ));
            expected.push(value.to_string());

            source.push_str(&format!(
                "  spec negative_{index}() -> Int {{ -0x{value:x} }}\n"
            ));
            expected.push(if value == 0 {
                String::from("0")
            } else {
                format!("-{value}")
            });
        }
        source.push_str("}\n");

        let fixture = Fixture::new(source);
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert_eq!(first.diagnostics, []);
        let observed: Vec<_> = first
            .core
            .unwrap()
            .functions
            .iter()
            .map(|function| function.body.literal().unwrap().to_string())
            .collect();
        assert_eq!(observed, expected);
    }

    #[test]
    fn multi_limb_integer_decoding_matches_independent_cross_radix_reference() {
        let bit_lengths = [
            31,
            32,
            33,
            63,
            64,
            65,
            127,
            128,
            129,
            255,
            256,
            257,
            511,
            512,
            513,
            1_023,
            1_024,
            1_025,
            4_095,
            4_096,
            4_097,
            MAX_INTEGER_BITS - 1,
            MAX_INTEGER_BITS,
        ];
        let mut state = 0xd1b5_4a32_d192_ed03_u64;
        let mut source = String::from("edition 2026; module multi_limb_reference {\n");
        let mut expected = Vec::with_capacity(bit_lengths.len() * 4);

        for (index, bit_length) in bit_lengths.into_iter().enumerate() {
            let bits = deterministic_bits(bit_length, &mut state);
            let decimal = reference_decimal_from_bits(&bits);
            let hexadecimal = reference_hexadecimal_from_bits(&bits);

            source.push_str(&format!(
                "  spec decimal_{index}() -> Int {{ {} }}\n",
                group_digits(&decimal, 4)
            ));
            expected.push(decimal.clone());

            source.push_str(&format!(
                "  spec binary_{index}() -> Int {{ 0b{} }}\n",
                group_digits(&bits, 5)
            ));
            expected.push(decimal.clone());

            source.push_str(&format!(
                "  spec hexadecimal_{index}() -> Int {{ 0X{} }}\n",
                group_digits(&hexadecimal.to_uppercase(), 3)
            ));
            expected.push(decimal.clone());

            source.push_str(&format!(
                "  spec negative_{index}() -> Int {{ -0x{} }}\n",
                group_digits(&hexadecimal, 7)
            ));
            expected.push(format!("-{decimal}"));
        }
        source.push_str("}\n");

        let fixture = Fixture::new(source);
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert_eq!(first.diagnostics, []);
        let core = first.core.as_ref().unwrap();
        let observed_core: Vec<_> = core
            .functions
            .iter()
            .map(|function| function.body.literal().unwrap().to_string())
            .collect();
        assert_eq!(observed_core, expected);

        let first_evaluation = crate::eval::evaluate(core);
        let second_evaluation = crate::eval::evaluate(core);
        assert_eq!(first_evaluation, second_evaluation);
        assert_eq!(first_evaluation.diagnostics(), []);
        let observed_evaluation: Vec<_> = first_evaluation
            .values()
            .unwrap()
            .iter()
            .map(|function| function.value().to_string())
            .collect();
        assert_eq!(observed_evaluation, expected);
    }

    #[test]
    fn large_integer_rendering_matches_decimal_doubling_reference() {
        let exponents = [128, 255, 256, 1_024, MAX_INTEGER_BITS - 1];
        let mut source = String::from("edition 2026; module large_integers {\n");
        let mut expected = Vec::with_capacity(exponents.len() + 2);
        for exponent in exponents {
            let decimal = power_of_two_decimal(exponent);
            source.push_str(&format!(
                "  spec power_{exponent}() -> Int {{ 0b1{} }}\n",
                "0".repeat(exponent)
            ));
            expected.push(decimal.clone());
            if exponent == MAX_INTEGER_BITS - 1 {
                source.push_str(&format!("  spec decimal_limit() -> Int {{ {decimal} }}\n"));
                expected.push(decimal.clone());
                source.push_str(&format!(
                    "  spec hexadecimal_limit() -> Int {{ 0x8{} }}\n",
                    "0".repeat((MAX_INTEGER_BITS - 4) / 4)
                ));
                expected.push(decimal);
            }
        }
        source.push_str("}\n");

        let fixture = Fixture::new(source);
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert_eq!(first.diagnostics, []);
        let observed: Vec<_> = first
            .core
            .unwrap()
            .functions
            .iter()
            .map(|function| function.body.literal().unwrap().to_string())
            .collect();
        assert_eq!(observed, expected);
    }

    #[test]
    fn integer_at_significant_bit_limit_is_exact() {
        let magnitude = format!("8{}", "0".repeat((MAX_INTEGER_BITS - 1) / 4));
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec huge() -> Int {{ 0x{magnitude} }} }}\n"
        ));
        let result = fixture.analyze();
        let core = result.core.unwrap();
        let Some(CoreValue::Int(value)) = core.functions[0].body.literal() else {
            panic!("expected exact integer");
        };
        assert_eq!(value.magnitude_bits(), MAX_INTEGER_BITS);
    }

    #[test]
    fn integer_over_significant_bit_limit_is_rejected_without_core() {
        let cases = [
            ("binary", format!("0b1{}", "0".repeat(MAX_INTEGER_BITS))),
            (
                "hexadecimal",
                format!("0x1{}", "0".repeat(MAX_INTEGER_BITS / 4)),
            ),
            ("decimal", power_of_two_decimal(MAX_INTEGER_BITS)),
        ];

        for (radix, literal) in cases {
            let fixture = Fixture::new(format!(
                "edition 2026; module values {{ spec huge() -> Int {{ {literal} }} }}\n"
            ));
            let first = fixture.analyze();
            let second = fixture.analyze();

            assert_eq!(first, second, "{radix} rejection must be repeatable");
            assert!(first.core.is_none(), "{radix} must not produce Core");
            assert_eq!(first.diagnostics.len(), 1, "{radix} diagnostic count");
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::IntegerMagnitudeLimit,
                "{radix} diagnostic code"
            );
            assert_eq!(
                fixture.source().slice(first.diagnostics[0].primary_span()),
                Some(literal.as_str()),
                "{radix} diagnostic span"
            );
        }
    }

    #[test]
    fn leading_zeroes_consume_no_significant_bit_or_event_budget() {
        let zeroes = "0".repeat(MAX_INTEGER_BITS + 1);
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec value() -> Word[8] {{ 0x{zeroes}2a }} }}\n"
        ));

        // Two namespace operations, two type-component inspections, one sign
        // inspection, one prefix inspection, two significant digits, and four
        // Core-node attempts. The leading zeroes consume neither bit budget nor
        // semantic events.
        let first = fixture.analyze_with(Limits {
            nodes: 4,
            events: 12,
            ..Limits::DEFAULT
        });
        let second = fixture.analyze_with(Limits {
            nodes: 4,
            events: 12,
            ..Limits::DEFAULT
        });
        assert_eq!(first, second);
        assert_eq!(first.diagnostics, []);
        assert_eq!(
            first.core.unwrap().functions[0].body.literal(),
            Some(&CoreValue::Word8(42))
        );

        let first = fixture.analyze_with(Limits {
            nodes: 4,
            events: 11,
            ..Limits::DEFAULT
        });
        let second = fixture.analyze_with(Limits {
            nodes: 4,
            events: 11,
            ..Limits::DEFAULT
        });
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(
            first.diagnostics.last().unwrap().code(),
            DiagnosticCode::SemanticResourceLimit
        );

        let negative_zero = Fixture::new(format!(
            "edition 2026; module values {{ spec value() -> Int {{ -0x{zeroes} }} }}\n"
        ));
        let limits = Limits {
            nodes: 4,
            events: 9,
            ..Limits::DEFAULT
        };
        let first = negative_zero.analyze_with(limits);
        let second = negative_zero.analyze_with(limits);
        assert_eq!(first, second);
        assert_eq!(first.diagnostics, []);
        assert_eq!(
            first.core.unwrap().functions[0]
                .body
                .literal()
                .unwrap()
                .to_string(),
            "0"
        );

        let limits = Limits {
            nodes: 4,
            events: 8,
            ..Limits::DEFAULT
        };
        let first = negative_zero.analyze_with(limits);
        let second = negative_zero.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(
            first.diagnostics.last().unwrap().code(),
            DiagnosticCode::SemanticResourceLimit
        );
    }

    #[test]
    fn word_boundaries_are_exact_and_stably_formatted() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec low() -> Word[8] { 0 }\n",
            "  spec one() -> Word[8] { 1 }\n",
            "  spec below_high() -> Word[8] { 254 }\n",
            "  spec high() -> Word[8] { 0xff }\n",
            "}\n",
        ));
        let result = fixture.analyze();
        let values: Vec<_> = result
            .core
            .unwrap()
            .functions
            .iter()
            .map(|function| function.body.literal().unwrap().to_string())
            .collect();
        assert_eq!(values, ["0x00", "0x01", "0xfe", "0xff"]);
    }

    #[test]
    fn every_word8_value_decodes_exactly_in_every_radix() {
        let mut source = String::from("edition 2026; module word8_corpus {\n");
        let mut expected = Vec::with_capacity(256 * 3);
        for value in u8::MIN..=u8::MAX {
            source.push_str(&format!(
                "  spec decimal_{value}() -> Word[8] {{ {value} }}\n"
            ));
            expected.push(CoreValue::Word8(value));

            source.push_str(&format!(
                "  spec binary_{value}() -> Word[8] {{ 0b{value:b} }}\n"
            ));
            expected.push(CoreValue::Word8(value));

            source.push_str(&format!(
                "  spec hexadecimal_{value}() -> Word[8] {{ 0X{value:X} }}\n"
            ));
            expected.push(CoreValue::Word8(value));
        }
        source.push_str("}\n");

        let fixture = Fixture::new(source);
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert_eq!(first.diagnostics, []);
        let observed: Vec<_> = first
            .core
            .unwrap()
            .functions
            .iter()
            .map(|function| function.body.literal().unwrap().clone())
            .collect();
        assert_eq!(observed, expected);
    }

    #[test]
    fn words_reject_negative_and_out_of_range_values_without_coercion() {
        let cases = [
            ("-0", DiagnosticCode::NegativeWordLiteral),
            ("-255", DiagnosticCode::NegativeWordLiteral),
            ("-256", DiagnosticCode::NegativeWordLiteral),
            (
                "-340282366920938463463374607431768211455",
                DiagnosticCode::NegativeWordLiteral,
            ),
            ("-0b0", DiagnosticCode::NegativeWordLiteral),
            ("-0b11111111", DiagnosticCode::NegativeWordLiteral),
            ("-0b100000000", DiagnosticCode::NegativeWordLiteral),
            (
                "-0b11111111111111111111111111111111111111111111111111111111111111111",
                DiagnosticCode::NegativeWordLiteral,
            ),
            ("-0x0", DiagnosticCode::NegativeWordLiteral),
            ("-0xff", DiagnosticCode::NegativeWordLiteral),
            ("-0x100", DiagnosticCode::NegativeWordLiteral),
            (
                "-0xffffffffffffffffffffffffffffffff",
                DiagnosticCode::NegativeWordLiteral,
            ),
            ("256", DiagnosticCode::WordLiteralOutOfRange),
            ("0b100000000", DiagnosticCode::WordLiteralOutOfRange),
            ("0x1_00", DiagnosticCode::WordLiteralOutOfRange),
        ];

        for (literal, code) in cases {
            let fixture = Fixture::new(format!(
                "edition 2026; module values {{ spec bad() -> Word[8] {{ {literal} }} }}\n"
            ));
            let first = fixture.analyze();
            let second = fixture.analyze();
            assert_eq!(first, second, "{literal} rejection must be repeatable");
            assert!(first.core.is_none(), "{literal} must not produce Core");
            assert_eq!(first.diagnostics.len(), 1, "{literal} diagnostic count");
            assert_eq!(first.diagnostics[0].code(), code, "{literal} code");
            assert_eq!(
                fixture.source().slice(first.diagnostics[0].primary_span()),
                Some(literal),
                "{literal} diagnostic span"
            );
        }

        let over_bit_limit = format!("0x1{}", "0".repeat(MAX_INTEGER_BITS / 4));
        for literal in [over_bit_limit.clone(), format!("-{over_bit_limit}")] {
            let fixture = Fixture::new(format!(
                "edition 2026; module values {{ spec bad() -> Word[8] {{ {literal} }} }}\n"
            ));
            let result = fixture.analyze();
            assert!(result.core.is_none());
            assert_eq!(result.diagnostics.len(), 1, "{literal} diagnostic count");
            assert_eq!(
                result.diagnostics[0].code(),
                DiagnosticCode::IntegerMagnitudeLimit,
                "{literal} must enforce the magnitude limit before word sign or range checks",
            );
            assert_eq!(
                fixture.source().slice(result.diagnostics[0].primary_span()),
                Some(over_bit_limit.as_str()),
                "{literal} magnitude diagnostic span",
            );
        }
    }

    #[test]
    fn namespace_uniqueness_is_per_kind_and_cites_the_first_declaration() {
        let duplicate = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec same() {}\n",
            "  spec same() -> Int { 1 }\n",
            "}\n",
        ));
        let result = duplicate.analyze();
        assert!(result.core.is_none());
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::DuplicateFunction
        );
        assert_eq!(
            duplicate
                .source()
                .slice(result.diagnostics[0].primary_span()),
            Some("same")
        );
        let [first_declaration] = result.diagnostics[0].secondary_spans() else {
            panic!("duplicate diagnostic must cite exactly one first declaration");
        };
        assert_eq!(
            duplicate.source().slice(first_declaration.span()),
            Some("same")
        );
        assert_eq!(first_declaration.label(), "first declaration is here");

        let cross_kind = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec same() {}\n",
            "  impl same() {}\n",
            "}\n",
        ));
        let result = cross_kind.analyze();
        assert_eq!(result.diagnostics, []);
        assert!(result.core.unwrap().functions.is_empty());
    }

    #[test]
    fn typed_impls_and_unadmitted_types_fail_closed() {
        let mut typed_impl =
            Fixture::new("edition 2026; module values { spec typed() -> Int { 1 } }\n");
        typed_impl.ast.module.functions[0].kind = FunctionKind::Impl;
        let result = typed_impl.analyze();
        assert!(result.core.is_none());
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::UnsupportedTypedFunction
        );
        assert_eq!(
            typed_impl
                .source()
                .slice(result.diagnostics[0].primary_span()),
            Some("typed")
        );

        let cases = [
            (
                "spec typed() -> Integer { 1 }",
                DiagnosticCode::UnsupportedType,
                "Integer",
            ),
            (
                "spec typed() -> Word { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "Word",
            ),
            (
                "spec typed() -> Word[12] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "12",
            ),
            (
                "spec typed() -> Word[128] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "128",
            ),
            (
                "spec typed() -> Word[016] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "016",
            ),
            (
                "spec typed(x: Word[7]) -> Word[8] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "7",
            ),
            (
                "spec typed(x: Integer) -> Int { 1 }",
                DiagnosticCode::UnsupportedType,
                "Integer",
            ),
            (
                "spec typed() -> Word[08] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "08",
            ),
            (
                "spec typed() -> Word[0x8] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "0x8",
            ),
            (
                "spec typed() -> Word[1_0] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
                "1_0",
            ),
            (
                "spec typed() -> Int[8] { 1 }",
                DiagnosticCode::UnsupportedType,
                "Int[8]",
            ),
        ];
        for (declaration, code, responsible_source) in cases {
            let fixture =
                Fixture::new(format!("edition 2026; module values {{ {declaration} }}\n"));
            let first = fixture.analyze();
            let second = fixture.analyze();
            assert_eq!(first, second, "{declaration} rejection must be repeatable");
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1, "{declaration} diagnostic count");
            assert_eq!(first.diagnostics[0].code(), code, "{declaration} code");
            assert_eq!(
                fixture.source().slice(first.diagnostics[0].primary_span()),
                Some(responsible_source),
                "{declaration} responsible source span"
            );
        }
    }

    #[test]
    fn independent_semantic_errors_preserve_source_order_and_responsible_spans() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec repeated() {}\n",
            "  spec repeated() {}\n",
            "  spec unsupported() -> Integer { 1 }\n",
            "  spec missing_width() -> Word { 1 }\n",
            "  spec bad_width() -> Word[12] { 1 }\n",
            "  spec negative() -> Word[8] { -1 }\n",
            "  spec out_of_range() -> Word[8] { 256 }\n",
            "}\n",
        ));

        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert!(first.core.is_none());

        let expected = [
            (DiagnosticCode::DuplicateFunction, "repeated"),
            (DiagnosticCode::UnsupportedType, "Integer"),
            (DiagnosticCode::UnsupportedWordWidth, "Word"),
            (DiagnosticCode::UnsupportedWordWidth, "12"),
            (DiagnosticCode::NegativeWordLiteral, "-1"),
            (DiagnosticCode::WordLiteralOutOfRange, "256"),
        ];
        assert_eq!(first.diagnostics.len(), expected.len());
        for (diagnostic, (code, responsible_source)) in first.diagnostics.iter().zip(expected) {
            assert_eq!(diagnostic.code(), code);
            assert_eq!(
                fixture.source().slice(diagnostic.primary_span()),
                Some(responsible_source)
            );
        }

        let [first_declaration] = first.diagnostics[0].secondary_spans() else {
            panic!("duplicate diagnostic must cite exactly one first declaration");
        };
        assert_eq!(
            fixture.source().slice(first_declaration.span()),
            Some("repeated")
        );

        let rendered = crate::diagnostic::render_diagnostics(&fixture.sources, &first.diagnostics);
        let rendered_codes: Vec<_> = rendered
            .lines()
            .filter_map(|line| {
                line.strip_prefix("error[")
                    .and_then(|rest| rest.split_once(']'))
                    .map(|(code, _)| code)
            })
            .collect();
        assert_eq!(
            rendered_codes,
            [
                "ORC0201", "ORC0203", "ORC0204", "ORC0204", "ORC0206", "ORC0207"
            ]
        );
        assert_eq!(
            rendered,
            crate::diagnostic::render_diagnostics(&fixture.sources, &second.diagnostics)
        );
    }

    #[test]
    fn compounded_declaration_failures_follow_semantic_traversal_order() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec repeated() {}\n",
            "  spec repeated() -> Word[12] { 1 }\n",
            "}\n",
        ));

        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(
            first
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            [
                DiagnosticCode::DuplicateFunction,
                DiagnosticCode::UnsupportedWordWidth,
            ]
        );
        assert_eq!(
            fixture.source().slice(first.diagnostics[0].primary_span()),
            Some("repeated")
        );
        assert_eq!(
            fixture.source().slice(first.diagnostics[1].primary_span()),
            Some("12")
        );
        let [first_declaration] = first.diagnostics[0].secondary_spans() else {
            panic!("duplicate diagnostic must cite exactly one first declaration");
        };
        assert_eq!(
            fixture.source().slice(first_declaration.span()),
            Some("repeated")
        );
    }

    #[test]
    fn core_ids_follow_only_typed_specs_in_source_order() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec empty() {}\n",
            "  impl also_empty() {}\n",
            "  spec first() -> Int { 7 }\n",
            "  spec second() -> Word[8] { 8 }\n",
            "}\n",
        ));
        let functions = fixture.analyze().core.unwrap().functions;
        assert_eq!(functions[0].id.index(), 0);
        assert_eq!(functions[0].name, "first");
        assert_eq!(functions[1].id.index(), 1);
        assert_eq!(functions[1].name, "second");
    }

    #[test]
    fn magnitude_limb_reservation_failure_returns_no_partial_core() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        let analyze_with_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_magnitude_limb = |_| false;
            analyzer.run()
        };

        let first = analyze_with_failure();
        let second = analyze_with_failure();
        assert_eq!(first, second);
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(fixture.source().slice(diagnostic.primary_span()), Some("1"));
        assert_eq!(
            diagnostic.label(),
            "exact integer magnitude storage allocation failed"
        );
    }

    #[test]
    fn index_range_reservation_failure_returns_no_partial_core() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec f(x: Word[8]^4) -> Word[8] { for i in 1..5 with s: Word[8] = 0 { s ^ x[i - 1] } }\n",
            "}\n",
        ));
        let analyze_with_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_range_limbs = |_, _| false;
            analyzer.run()
        };

        let first = analyze_with_failure();
        let second = analyze_with_failure();
        assert_eq!(first, second);
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(fixture.source().slice(diagnostic.primary_span()), Some("i"));
        assert_eq!(diagnostic.label(), "index range storage allocation failed");
    }

    #[test]
    fn pending_function_reservation_failure_returns_no_partial_core() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        let analyze_with_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_pending_function_slot = |_| false;
            analyzer.run()
        };

        let first = analyze_with_failure();
        let second = analyze_with_failure();
        assert_eq!(first, second);
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some("spec value() -> Int { 1 }")
        );
        assert_eq!(
            diagnostic.label(),
            "semantic analysis could not allocate pending function storage"
        );
    }

    #[test]
    fn diagnostic_vector_reservation_failure_returns_no_core_or_diagnostics() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_diagnostic_slots = |_, _| false;

        let analyzed = analyzer.run();

        assert!(analyzed.has_errors());
        assert!(analyzed.core().is_none());
        assert!(analyzed.diagnostics().is_empty());
        assert_eq!(analyzed.diagnostics.capacity(), 0);
    }

    #[test]
    fn core_name_reservation_failure_returns_no_partial_core() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        let analyze_with_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_core_name = |_, _| false;
            analyzer.run()
        };

        let first = analyze_with_failure();
        let second = analyze_with_failure();
        assert_eq!(first, second);
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some("value")
        );
        assert_eq!(
            diagnostic.label(),
            "typed Core name storage allocation failed"
        );
    }

    #[test]
    fn late_allocation_failures_discard_completed_pending_core() {
        let magnitude_fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Word[8] { 0 }\n",
            "  spec second() -> Int { 2 }\n",
            "}\n",
        ));
        let mut magnitude_analyzer = Analyzer::new(
            magnitude_fixture.source(),
            &magnitude_fixture.ast,
            Limits::DEFAULT,
        );
        magnitude_analyzer.reserve_magnitude_limb = |_| false;
        let magnitude_failure = magnitude_analyzer.run();

        assert!(magnitude_failure.core().is_none());
        assert_eq!(magnitude_failure.diagnostics().len(), 1);
        assert_eq!(
            magnitude_fixture
                .source()
                .slice(magnitude_failure.diagnostics()[0].primary_span()),
            Some("2")
        );
        assert_eq!(
            magnitude_failure.diagnostics()[0].label(),
            "exact integer magnitude storage allocation failed"
        );

        let pending_fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Int { 1 }\n",
            "  spec second() -> Int { 2 }\n",
            "}\n",
        ));
        let mut pending_analyzer = Analyzer::new(
            pending_fixture.source(),
            &pending_fixture.ast,
            Limits::DEFAULT,
        );
        pending_analyzer.reserve_pending_function_slot =
            |functions| functions.is_empty() && functions.try_reserve(1).is_ok();
        let pending_failure = pending_analyzer.run();

        assert!(pending_failure.core().is_none());
        assert_eq!(pending_failure.diagnostics().len(), 1);
        assert_eq!(
            pending_fixture
                .source()
                .slice(pending_failure.diagnostics()[0].primary_span()),
            Some("spec second() -> Int { 2 }")
        );
        assert_eq!(
            pending_failure.diagnostics()[0].label(),
            "semantic analysis could not allocate pending function storage"
        );

        let module_fixture = Fixture::new(concat!(
            "edition 2026; module module_identifier {\n",
            "  spec a() -> Int { 1 }\n",
            "  spec bb() -> Word[8] { 2 }\n",
            "}\n",
        ));
        let mut module_analyzer = Analyzer::new(
            module_fixture.source(),
            &module_fixture.ast,
            Limits::DEFAULT,
        );
        module_analyzer.reserve_core_name = |name, bytes| {
            bytes != "module_identifier".len() && name.try_reserve_exact(bytes).is_ok()
        };
        let module_failure = module_analyzer.run();

        assert!(module_failure.core().is_none());
        assert_eq!(module_failure.diagnostics().len(), 1);
        assert_eq!(
            module_fixture
                .source()
                .slice(module_failure.diagnostics()[0].primary_span()),
            Some("module_identifier")
        );
        assert_eq!(
            module_failure.diagnostics()[0].label(),
            "typed Core name storage allocation failed"
        );
    }

    #[test]
    fn declaration_namespace_reservation_failure_returns_no_partial_core() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec first() {}\n",
            "  impl second() {}\n",
            "}\n",
        ));
        let first = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
            .run_with_reservations(
                |_, capacity| {
                    assert_eq!(capacity, 2);
                    false
                },
                |functions, capacity| functions.try_reserve_exact(capacity).is_ok(),
            );
        let second = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
            .run_with_reservations(
                |_, _| false,
                |functions, capacity| functions.try_reserve_exact(capacity).is_ok(),
            );

        assert_eq!(first, second);
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(diagnostic.primary_span(), fixture.ast.module.span);
        assert_eq!(
            diagnostic.label(),
            "semantic declaration namespace storage allocation failed"
        );
        assert_eq!(
            diagnostic.notes(),
            &["semantic analysis stopped without producing Core"]
        );
    }

    #[test]
    fn long_identifier_diagnostics_have_bounded_messages() {
        let long_type = "N".repeat(1_024);
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec value() -> {long_type} {{ 1 }} }}\n"
        ));

        let first = fixture.analyze();
        let second = fixture.analyze();

        assert_eq!(first, second);
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        assert_eq!(
            first.diagnostics()[0].message(),
            format!(
                "unsupported result type `{}...<1024 bytes total>`",
                "N".repeat(MAX_IDENTIFIER_BYTES_IN_DIAGNOSTIC)
            )
        );
        assert!(first.diagnostics()[0].message().len() < 128);
    }

    #[test]
    fn core_function_reservation_failure_returns_no_partial_core() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        let first = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
            .run_with_reservations(
                |declarations, capacity| declarations.try_reserve(capacity).is_ok(),
                |_, capacity| {
                    assert_eq!(capacity, 1);
                    false
                },
            );
        let second = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
            .run_with_reservations(
                |declarations, capacity| declarations.try_reserve(capacity).is_ok(),
                |_, _| false,
            );

        assert_eq!(first, second);
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(diagnostic.primary_span(), fixture.ast.module.span);
        assert_eq!(
            diagnostic.message(),
            "semantic analysis resource limit exceeded"
        );
        assert_eq!(
            diagnostic.label(),
            "typed Core function storage allocation failed"
        );
        assert_eq!(
            diagnostic.notes(),
            &["semantic analysis stopped without producing Core"]
        );
    }

    #[test]
    fn diagnostic_and_resource_limits_fail_closed() {
        let diagnostics = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Nope { 1 }\n",
            "  spec second() -> Nope { 2 }\n",
            "}\n",
        ));
        let limits = Limits {
            diagnostics: 1,
            ..Limits::DEFAULT
        };
        let first = diagnostics.analyze_with(limits);
        let second = diagnostics.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(
            first
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            [
                DiagnosticCode::UnsupportedType,
                DiagnosticCode::TooManySemanticErrors
            ]
        );

        let diagnostic_attempt =
            Fixture::new("edition 2026; module values { spec value() -> Nope { 1 } }\n");
        let below_event_boundary = diagnostic_attempt.analyze_with(Limits {
            diagnostics: 0,
            events: 3,
            ..Limits::DEFAULT
        });
        assert_eq!(
            below_event_boundary
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            [DiagnosticCode::SemanticResourceLimit]
        );
        let at_event_boundary = diagnostic_attempt.analyze_with(Limits {
            diagnostics: 0,
            events: 4,
            ..Limits::DEFAULT
        });
        assert_eq!(
            at_event_boundary
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            [DiagnosticCode::TooManySemanticErrors]
        );

        let typed = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        for (limits, expected_span, expected_label) in [
            (
                Limits {
                    events: 0,
                    ..Limits::DEFAULT
                },
                typed.ast.module.functions[0].name.span,
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    nodes: 0,
                    ..Limits::DEFAULT
                },
                typed.ast.module.span,
                "typed Core node budget exhausted",
            ),
        ] {
            let first = typed.analyze_with(limits);
            let second = typed.analyze_with(limits);
            assert_eq!(first, second);
            assert!(first.core.is_none());
            let diagnostic = first.diagnostics.last().unwrap();
            assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
            assert_eq!(diagnostic.primary_span(), expected_span);
            assert_eq!(
                diagnostic.message(),
                "semantic analysis resource limit exceeded"
            );
            assert_eq!(diagnostic.label(), expected_label);
            assert_eq!(
                diagnostic.notes(),
                &["semantic analysis stopped without producing Core"]
            );
        }

        let suppressed_then_exhausted = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Nope { 1 }\n",
            "  spec second() -> Nope { 2 }\n",
            "}\n",
        ));
        // Each declaration consumes lookup, insertion, type-inspection, and
        // diagnostic-attempt events. The first attempt emits the suppression
        // record; the later suppressed attempt must still consume event eight.
        for (events, expected_codes) in [
            (
                7,
                &[
                    DiagnosticCode::TooManySemanticErrors,
                    DiagnosticCode::SemanticResourceLimit,
                ][..],
            ),
            (8, &[DiagnosticCode::TooManySemanticErrors][..]),
        ] {
            let limits = Limits {
                diagnostics: 0,
                events,
                ..Limits::DEFAULT
            };
            let first = suppressed_then_exhausted.analyze_with(limits);
            let second = suppressed_then_exhausted.analyze_with(limits);
            assert_eq!(first, second);
            assert!(first.core.is_none());
            assert_eq!(
                first
                    .diagnostics
                    .iter()
                    .map(Diagnostic::code)
                    .collect::<Vec<_>>(),
                expected_codes
            );
        }

        let compounded = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec repeated() {}\n",
            "  spec repeated() -> Word[12] { 1 }\n",
            "}\n",
        ));
        // The first declaration consumes lookup and insertion. The second
        // consumes lookup, duplicate-report, type-name, width, and
        // width-report events. At six events the final report attempt becomes
        // resource exhaustion; at seven it becomes diagnostic suppression.
        for (limits, expected_codes) in [
            (
                Limits {
                    diagnostics: 1,
                    events: 6,
                    ..Limits::DEFAULT
                },
                &[
                    DiagnosticCode::DuplicateFunction,
                    DiagnosticCode::SemanticResourceLimit,
                ][..],
            ),
            (
                Limits {
                    diagnostics: 1,
                    events: 7,
                    ..Limits::DEFAULT
                },
                &[
                    DiagnosticCode::DuplicateFunction,
                    DiagnosticCode::TooManySemanticErrors,
                ][..],
            ),
        ] {
            let first = compounded.analyze_with(limits);
            let second = compounded.analyze_with(limits);
            assert_eq!(first, second);
            assert!(first.core.is_none());
            assert_eq!(
                first
                    .diagnostics
                    .iter()
                    .map(Diagnostic::code)
                    .collect::<Vec<_>>(),
                expected_codes
            );
            assert_eq!(
                compounded
                    .source()
                    .slice(first.diagnostics[1].primary_span()),
                Some("12")
            );
        }
    }

    #[test]
    fn exact_ordinary_diagnostic_budget_emits_no_suppression_record() {
        let two_errors = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Nope { 1 }\n",
            "  spec second() -> Nope { 2 }\n",
            "}\n",
        ));
        let at_injected_limit = two_errors.analyze_with(Limits {
            diagnostics: 2,
            ..Limits::DEFAULT
        });
        assert!(at_injected_limit.core.is_none());
        assert_eq!(
            at_injected_limit
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            [
                DiagnosticCode::UnsupportedType,
                DiagnosticCode::UnsupportedType
            ]
        );

        let mut text = String::from("edition 2026; module values {\n");
        for index in 0..MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE {
            text.push_str(&format!("  spec bad{index}() -> Nope {{ {index} }}\n"));
        }
        let mut exceeded_text = text.clone();
        text.push_str("}\n");
        exceeded_text.push_str("  spec overflow() -> Nope { 0 }\n}\n");

        let at_limit = Fixture::new(text);
        let first = at_limit.analyze_with(Limits::DEFAULT);
        let second = at_limit.analyze_with(Limits::DEFAULT);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE);
        assert!(
            first
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code() == DiagnosticCode::UnsupportedType)
        );

        let exceeded = Fixture::new(exceeded_text).analyze_with(Limits::DEFAULT);
        assert!(exceeded.core.is_none());
        assert_eq!(
            exceeded.diagnostics.len(),
            MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE + 1
        );
        let shape = |diagnostic: &Diagnostic| {
            (
                diagnostic.code(),
                diagnostic.primary_span().start().bytes(),
                diagnostic.primary_span().end().bytes(),
            )
        };
        assert_eq!(
            exceeded.diagnostics[..MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE]
                .iter()
                .map(shape)
                .collect::<Vec<_>>(),
            first.diagnostics.iter().map(shape).collect::<Vec<_>>()
        );
        assert_eq!(
            exceeded.diagnostics.last().unwrap().code(),
            DiagnosticCode::TooManySemanticErrors
        );
    }

    #[test]
    fn complete_diagnostic_bound_requires_no_capacity_growth() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        assert!((analyzer.reserve_diagnostic_slots)(
            &mut analyzer.diagnostics,
            MAX_RETAINED_SEMANTIC_DIAGNOSTICS,
        ));
        let initial_capacity = analyzer.diagnostics.capacity();
        let span = fixture.ast.module.functions[0].name.span;

        for _ in 0..=MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE {
            if analyzer.begin_report(span) {
                analyzer.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnsupportedType,
                    "synthetic semantic error",
                    span,
                ));
            }
        }
        analyzer.resource_limit(span, "synthetic resource failure");

        assert_eq!(
            analyzer.diagnostics.len(),
            MAX_RETAINED_SEMANTIC_DIAGNOSTICS
        );
        assert_eq!(analyzer.diagnostics.capacity(), initial_capacity);
    }

    #[test]
    fn suppressed_semantic_diagnostics_are_not_constructed() {
        let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        let mut analyzer = Analyzer::new(
            fixture.source(),
            &fixture.ast,
            Limits {
                diagnostics: 0,
                ..Limits::DEFAULT
            },
        );
        analyzer.diagnostics.try_reserve_exact(1).unwrap();
        let span = fixture.ast.module.functions[0].name.span;
        let constructed = std::cell::Cell::new(0_usize);

        for _ in 0..2 {
            if analyzer.begin_report(span) {
                constructed.set(constructed.get().saturating_add(1));
                analyzer.diagnostics.push(Diagnostic::error(
                    DiagnosticCode::UnsupportedType,
                    "unused",
                    span,
                ));
            }
        }

        assert_eq!(constructed.get(), 0);
        assert_eq!(analyzer.diagnostics.len(), 1);
        assert_eq!(
            analyzer.diagnostics[0].code(),
            DiagnosticCode::TooManySemanticErrors
        );
    }

    #[test]
    fn injected_limits_match_normative_event_and_core_node_accounting() {
        let empty = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec empty() {}\n",
            "  impl also_empty() {}\n",
            "}\n",
        ));
        let limits = Limits {
            nodes: 1,
            events: 5,
            ..Limits::DEFAULT
        };
        let first = empty.analyze_with(limits);
        let second = empty.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.unwrap().functions.is_empty());

        let duplicate = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec same() {}\n",
            "  spec same() {}\n",
            "}\n",
        ));
        let first = duplicate.analyze_with(Limits {
            events: 4,
            ..Limits::DEFAULT
        });
        let second = duplicate.analyze_with(Limits {
            events: 4,
            ..Limits::DEFAULT
        });
        assert_eq!(first, second);
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::DuplicateFunction
        );
        let first = duplicate.analyze_with(Limits {
            events: 3,
            ..Limits::DEFAULT
        });
        let second = duplicate.analyze_with(Limits {
            events: 3,
            ..Limits::DEFAULT
        });
        assert_eq!(first, second);
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::SemanticResourceLimit
        );
        assert_eq!(
            first.diagnostics[0].label(),
            "semantic event budget exhausted"
        );

        let typed = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        // Two namespace operations, one type inspection, sign/prefix/digit
        // inspection, and four Core-node attempts total ten events. The Core is
        // one module plus one function, type, and value node.
        for (limits, expected_label) in [
            (
                Limits {
                    nodes: 3,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
            (
                Limits {
                    events: 9,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
        ] {
            let first = typed.analyze_with(limits);
            let second = typed.analyze_with(limits);
            assert_eq!(first, second);
            assert!(first.core.is_none());
            let diagnostic = first.diagnostics.last().unwrap();
            assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
            assert_eq!(
                diagnostic.primary_span(),
                typed.ast.module.functions[0].span
            );
            assert_eq!(diagnostic.label(), expected_label);
        }
        let limits = Limits {
            nodes: 4,
            events: 10,
            ..Limits::DEFAULT
        };
        let first = typed.analyze_with(limits);
        let second = typed.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_some());
        assert_eq!(first.diagnostics, []);

        let mixed = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec empty() {}\n",
            "  spec first() -> Int { 1 }\n",
            "  impl also_empty() {}\n",
            "  spec second() -> Word[8] { 2 }\n",
            "}\n",
        ));
        // Eight namespace operations, three type-component inspections, six
        // literal inspections, and seven Core-node attempts total 24 events.
        // The Core is one module plus three nodes for each typed specification;
        // the two empty declarations add no Core nodes.
        for (limits, expected_label) in [
            (
                Limits {
                    nodes: 6,
                    events: 24,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
            (
                Limits {
                    nodes: 7,
                    events: 23,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
        ] {
            let first = mixed.analyze_with(limits);
            let second = mixed.analyze_with(limits);
            assert_eq!(first, second);
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), expected_label);
            assert_eq!(
                first.diagnostics[0].primary_span(),
                mixed.ast.module.functions[3].span
            );
        }
        let first = mixed.analyze_with(Limits {
            nodes: 7,
            events: 24,
            ..Limits::DEFAULT
        });
        let second = mixed.analyze_with(Limits {
            nodes: 7,
            events: 24,
            ..Limits::DEFAULT
        });
        assert_eq!(first, second);
        assert_eq!(first.diagnostics, []);
        let core = first.core.unwrap();
        assert_eq!(core.functions.len(), 2);
        assert_eq!(core.functions[0].name, "first");
        assert_eq!(core.functions[1].name, "second");
    }

    #[test]
    fn analysis_is_repeatable_for_typed_success_and_failure() {
        let accepted = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec integer() -> Int { -42 }\n",
            "  spec word() -> Word[8] { 42 }\n",
            "}\n",
        ));
        let first = accepted.analyze();
        let second = accepted.analyze();
        assert_eq!(first, second);
        assert_eq!(first.core.unwrap().functions.len(), 2);

        let rejected =
            Fixture::new("edition 2026; module values { spec value() -> lowercase { 1 } }\n");
        let first = rejected.analyze();
        let second = rejected.analyze();
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics[0].code(), DiagnosticCode::UnsupportedType);

        let first = accepted.analyze_with(Limits {
            events: 0,
            ..Limits::DEFAULT
        });
        let second = accepted.analyze_with(Limits {
            events: 0,
            ..Limits::DEFAULT
        });
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::SemanticResourceLimit
        );
    }

    #[test]
    fn mutated_s3a_sources_preserve_phase_gates_and_repeatability() {
        let base = concat!(
            "edition 2026; module mutation_seed {\n",
            "  spec empty() {}\n",
            "  impl empty() {}\n",
            "  spec integer() -> Int { -42 }\n",
            "  spec word() -> Word[8] { 0xff }\n",
            "}\n",
        );
        let characters: Vec<_> = base
            .char_indices()
            .map(|(start, character)| (start, start + character.len_utf8()))
            .collect();
        let mut corpus = std::collections::BTreeSet::new();
        corpus.insert(base.to_owned());

        for &(start, end) in &characters {
            corpus.insert(format!("{}{}", &base[..start], &base[end..]));
            for replacement in ["@", "0", "_", "{", "}", "\"", "-", "\r", "é"] {
                corpus.insert(format!("{}{}{}", &base[..start], replacement, &base[end..]));
            }
        }
        for offset in characters
            .iter()
            .map(|(start, _)| *start)
            .chain(std::iter::once(base.len()))
        {
            for insertion in [
                "@", "0", "_", "{", "}", "\"", "-", "\r", "é", "/*", "*/", "//", "\0",
            ] {
                corpus.insert(format!(
                    "{}{}{}",
                    &base[..offset],
                    insertion,
                    &base[offset..]
                ));
            }
        }

        let fragments = [
            "edition", "2026", ";", "module", "spec", "impl", "Int", "Word", "[", "]", "(", ")",
            "{", "}", "-", "0", "1", "256", "name", " ", "\n", "\r\n", "//x\n", "/*x*/", "@", "\"",
            "é",
        ];
        let mut state = 0x6a09_e667_f3bc_c908_u64;
        for _ in 0..512 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let fragment_count = usize::try_from(state % 48 + 1).unwrap();
            let mut text = String::new();
            for _ in 0..fragment_count {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let index = usize::try_from(state % fragments.len() as u64).unwrap();
                text.push_str(fragments[index]);
            }
            corpus.insert(text);
        }
        assert!(corpus.len() > 2_500, "mutation corpus unexpectedly shrank");

        let total_cases = corpus.len();
        let mut lexical_failures = 0_usize;
        let mut parser_failures = 0_usize;
        let mut semantic_failures = 0_usize;
        let mut evaluated_successes = 0_usize;
        for (case_index, text) in corpus.into_iter().enumerate() {
            let mut sources = SourceMap::new();
            let id = sources.add("mutation.or", text).unwrap();
            let source = sources.get(id).unwrap();
            let assert_owned_spans = |diagnostics: &[Diagnostic]| {
                for diagnostic in diagnostics {
                    assert_eq!(
                        diagnostic.primary_span().source(),
                        source.id(),
                        "case {case_index}"
                    );
                    assert!(
                        source.slice(diagnostic.primary_span()).is_some(),
                        "case {case_index} has an invalid primary span"
                    );
                    for secondary in diagnostic.secondary_spans() {
                        assert_eq!(secondary.span().source(), source.id(), "case {case_index}");
                        assert!(
                            source.slice(secondary.span()).is_some(),
                            "case {case_index} has an invalid secondary span"
                        );
                    }
                }
            };
            let assert_repeatable_rendering =
                |first: &[Diagnostic], second: &[Diagnostic], phase: &str| {
                    assert_eq!(
                        crate::diagnostic::render_diagnostics(&sources, first),
                        crate::diagnostic::render_diagnostics(&sources, second),
                        "case {case_index} {phase} diagnostic rendering"
                    );
                };

            let first_lexed = lex(source, Edition::E2026);
            let second_lexed = lex(source, Edition::E2026);
            assert_eq!(first_lexed, second_lexed, "case {case_index} lexing");
            assert_owned_spans(first_lexed.diagnostics());
            assert_repeatable_rendering(
                first_lexed.diagnostics(),
                second_lexed.diagnostics(),
                "lexical",
            );
            if first_lexed.has_errors() {
                lexical_failures += 1;
                continue;
            }

            let first_parsed = parse(source, &first_lexed);
            let second_parsed = parse(source, &second_lexed);
            assert_eq!(first_parsed, second_parsed, "case {case_index} parsing");
            assert_owned_spans(first_parsed.diagnostics());
            assert_repeatable_rendering(
                first_parsed.diagnostics(),
                second_parsed.diagnostics(),
                "parser",
            );
            assert_eq!(
                first_parsed.ast().is_some(),
                first_parsed.diagnostics().is_empty(),
                "case {case_index} parser atomicity"
            );
            let Some(ast) = first_parsed.ast() else {
                parser_failures += 1;
                continue;
            };

            let first_analyzed = analyze(source, ast);
            let second_analyzed = analyze(source, ast);
            assert_eq!(
                first_analyzed, second_analyzed,
                "case {case_index} analysis"
            );
            assert_owned_spans(first_analyzed.diagnostics());
            assert_repeatable_rendering(
                first_analyzed.diagnostics(),
                second_analyzed.diagnostics(),
                "semantic",
            );
            assert_eq!(
                first_analyzed.core().is_some(),
                first_analyzed.diagnostics().is_empty(),
                "case {case_index} semantic atomicity"
            );
            let Some(core) = first_analyzed.core() else {
                semantic_failures += 1;
                continue;
            };

            let first_evaluated = crate::eval::evaluate(core);
            let second_evaluated = crate::eval::evaluate(core);
            assert_eq!(
                first_evaluated, second_evaluated,
                "case {case_index} evaluation"
            );
            assert_owned_spans(first_evaluated.diagnostics());
            assert_repeatable_rendering(
                first_evaluated.diagnostics(),
                second_evaluated.diagnostics(),
                "evaluation",
            );
            assert_eq!(
                first_evaluated.values().is_some(),
                first_evaluated.diagnostics().is_empty(),
                "case {case_index} evaluation atomicity"
            );
            assert!(
                first_evaluated.values().is_some(),
                "case {case_index} unexpectedly exhausted evaluation resources"
            );
            evaluated_successes += 1;
        }
        assert!(
            lexical_failures != 0,
            "mutation corpus missed lexical failure"
        );
        assert!(
            parser_failures != 0,
            "mutation corpus missed parser failure"
        );
        assert!(
            semantic_failures != 0,
            "mutation corpus missed semantic failure"
        );
        assert!(
            evaluated_successes != 0,
            "mutation corpus missed successful evaluation"
        );
        assert_eq!(
            lexical_failures + parser_failures + semantic_failures + evaluated_successes,
            total_cases,
            "mutation outcome partition drifted"
        );
    }

    #[test]
    fn analysis_is_repeatable_and_empty_modules_produce_empty_core() {
        let fixture =
            Fixture::new("edition 2026; module values { spec empty() {} impl empty() {} }\n");
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert!(first.core.unwrap().functions.is_empty());
    }

    /// Renders one function's postorder body Core as `(operation, source, type)`.
    fn core_nodes<'text>(
        fixture: &'text Fixture,
        function: &CoreFunction,
    ) -> Vec<(String, &'text str, CoreType)> {
        expression_nodes(fixture, &function.body)
    }

    /// Renders one postorder Core expression as `(operation, source, type)`.
    fn expression_nodes<'text>(
        fixture: &'text Fixture,
        expression: &CoreExpression,
    ) -> Vec<(String, &'text str, CoreType)> {
        expression
            .nodes
            .iter()
            .map(|node| {
                let operation = match &node.kind {
                    CoreNodeKind::Literal(value) => format!("literal {value}"),
                    CoreNodeKind::Parameter(index) => format!("parameter {index}"),
                    CoreNodeKind::Local(index) => format!("local {index}"),
                    CoreNodeKind::Convert { from } => format!("convert from {from}"),
                    CoreNodeKind::Call {
                        function,
                        arguments,
                    } => format!("call #{} with {arguments}", function.index()),
                    CoreNodeKind::Unary(operator) => format!("prefix {}", operator.as_str()),
                    CoreNodeKind::Binary(operator) => format!("infix {}", operator.as_str()),
                    CoreNodeKind::Shift { operator, amount } => {
                        format!("shift {} {amount}", operator.as_str())
                    }
                    CoreNodeKind::Array { elements } => format!("array of {elements}"),
                    CoreNodeKind::Index { index } => format!("index {index}"),
                    CoreNodeKind::Select => String::from("select"),
                    CoreNodeKind::Update => String::from("update"),
                    CoreNodeKind::Fill => String::from("fill"),
                    CoreNodeKind::Fold(id) => format!("loop #{id}"),
                    CoreNodeKind::LoopIndex(id) => format!("index of loop #{id}"),
                    CoreNodeKind::Accumulator(id) => format!("accumulator of loop #{id}"),
                    CoreNodeKind::Compare { operator, operand } => {
                        format!("compare {} on {operand}", operator.as_str())
                    }
                    CoreNodeKind::Choose(id) => format!("choose #{id}"),
                };
                (
                    operation,
                    fixture.source().slice(node.span).unwrap(),
                    node.ty,
                )
            })
            .collect()
    }

    /// Renders diagnostics as `(code, responsible source, message)`.
    fn reported<'text>(
        fixture: &'text Fixture,
        result: &AnalysisResult,
    ) -> Vec<(DiagnosticCode, &'text str, String)> {
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.code(),
                    fixture.source().slice(diagnostic.primary_span()).unwrap(),
                    diagnostic.message().to_owned(),
                )
            })
            .collect()
    }

    const TYPES: [CoreType; 5] = [
        CoreType::Int,
        CoreType::Word8,
        CoreType::Word16,
        CoreType::Word32,
        CoreType::Word64,
    ];

    fn module(members: &str) -> Fixture {
        Fixture::new(format!("edition 2026; module m {{\n{members}}}\n"))
    }

    fn accepted(members: &str) -> (Fixture, CoreModule) {
        let fixture = module(members);
        let result = fixture.analyze();
        assert_eq!(result.diagnostics, [], "{members}");
        let core = result.core.unwrap();
        (fixture, core)
    }

    fn rejected(members: &str) -> (Fixture, AnalysisResult) {
        let fixture = module(members);
        let first = fixture.analyze();
        assert_eq!(first, fixture.analyze(), "{members}");
        assert!(first.core.is_none(), "{members}");
        (fixture, first)
    }

    #[test]
    fn typed_core_is_postorder_with_exact_types_spans_and_operations() {
        let (fixture, core) = accepted(concat!(
            "  spec mix(x: Word[32], y: Word[32]) -> Word[32] { (x ^ ~y) <<< 7 }\n",
            "  spec use_mix() -> Word[32] { mix(1, 0xff) + 2 * 3 }\n",
            "  spec exact(n: Int) -> Int { -(n - -5) * n }\n",
        ));
        assert_eq!(
            core.functions
                .iter()
                .map(|function| (
                    function.id.index(),
                    function.name.as_str(),
                    function.parameters.clone(),
                    function.result_type
                ))
                .collect::<Vec<_>>(),
            [
                (
                    0,
                    "mix",
                    vec![CoreType::Word32, CoreType::Word32],
                    CoreType::Word32
                ),
                (1, "use_mix", vec![], CoreType::Word32),
                (2, "exact", vec![CoreType::Int], CoreType::Int),
            ]
        );
        let word = CoreType::Word32;
        let owned = |rows: &[(&str, &'static str, CoreType)]| {
            rows.iter()
                .map(|(operation, source, ty)| ((*operation).to_owned(), *source, *ty))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            core_nodes(&fixture, &core.functions[0]),
            owned(&[
                ("parameter 0", "x", word),
                ("parameter 1", "y", word),
                ("prefix ~", "~y", word),
                ("infix ^", "x ^ ~y", word),
                ("shift <<< 7", "(x ^ ~y) <<< 7", word),
            ])
        );
        assert_eq!(
            core_nodes(&fixture, &core.functions[1]),
            owned(&[
                ("literal 0x00000001", "1", word),
                ("literal 0x000000ff", "0xff", word),
                ("call #0 with 2", "mix(1, 0xff)", word),
                ("literal 0x00000002", "2", word),
                ("literal 0x00000003", "3", word),
                ("infix *", "2 * 3", word),
                ("infix +", "mix(1, 0xff) + 2 * 3", word),
            ])
        );
        assert_eq!(
            core_nodes(&fixture, &core.functions[2]),
            owned(&[
                ("parameter 0", "n", CoreType::Int),
                ("literal -5", "-5", CoreType::Int),
                ("infix -", "n - -5", CoreType::Int),
                ("prefix -", "-(n - -5)", CoreType::Int),
                ("parameter 0", "n", CoreType::Int),
                ("infix *", "-(n - -5) * n", CoreType::Int),
            ])
        );
        assert_eq!(
            core.functions[0].body.root().map(|node| node.kind.clone()),
            Some(CoreNodeKind::Shift {
                operator: BinaryOperator::RotateLeft,
                amount: 7
            })
        );
        assert_eq!(core.functions[0].body.literal(), None);
    }

    #[test]
    fn calls_resolve_in_any_order_and_acyclic_graphs_are_accepted() {
        let (fixture, core) = accepted(concat!(
            "  spec top() -> Int { left() + right() + left() }\n",
            "  spec left() -> Int { base(1) }\n",
            "  spec right() -> Int { base(2) * base(3) }\n",
            "  spec base(x: Int) -> Int { x * x }\n",
            "  spec unused() {}\n",
            "  impl unused() {}\n",
            "  spec after() -> Int { top() }\n",
        ));
        let calls = |index: usize| {
            core_nodes(&fixture, &core.functions[index])
                .into_iter()
                .filter(|(operation, _, _)| operation.starts_with("call"))
                .map(|(operation, source, _)| (operation, source))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            calls(0),
            [
                (String::from("call #1 with 0"), "left()"),
                (String::from("call #2 with 0"), "right()"),
                (String::from("call #1 with 0"), "left()"),
            ]
        );
        assert_eq!(
            calls(2),
            [
                (String::from("call #3 with 1"), "base(2)"),
                (String::from("call #3 with 1"), "base(3)"),
            ]
        );
        assert_eq!(calls(4), [(String::from("call #0 with 0"), "top()")]);
        assert_eq!(core.functions.len(), 5);
    }

    #[test]
    fn call_cycles_are_reported_once_at_the_closing_call() {
        let (fixture, result) = rejected(concat!(
            "  spec itself() -> Int { itself() + 1 }\n",
            "  spec ping() -> Int { pong() }\n",
            "  spec pong() -> Int { ping() }\n",
            "  spec a() -> Int { b() }\n",
            "  spec b() -> Int { c() }\n",
            "  spec c() -> Int { a() + a() }\n",
            "  spec into_cycle() -> Int { ping() }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::CallCycle,
                    "itself()",
                    String::from("`itself` calls itself")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "ping()",
                    String::from("call cycle `ping` -> `pong` -> `ping`")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "a()",
                    String::from("call cycle `a` -> `b` -> `c` -> `a`")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "a()",
                    String::from("call cycle `a` -> `b` -> `c` -> `a`")
                ),
            ]
        );
        let spans = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.primary_span())
            .collect::<Vec<_>>();
        assert_ne!(spans[2], spans[3], "each closing call is its own site");
        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.label(), "this call closes the cycle");
        assert_eq!(
            diagnostic.notes(),
            ["a `spec` may not depend on itself; recursion is not part of Orange 2026"]
        );
    }

    #[test]
    fn call_cycles_are_reported_through_calls_with_other_errors() {
        let (fixture, result) = rejected(concat!(
            "  spec a() -> Int { b() }\n",
            "  spec b() -> Word[8] { a() }\n",
            "  spec count() -> Int { arity(1) }\n",
            "  spec arity() -> Int { count() }\n",
            "  spec wide(x: Word[12]) -> Int { narrow() }\n",
            "  spec narrow() -> Int { wide(1) }\n",
            "  spec typed(x: Word[8]) -> Int { typed(256) }\n",
        ));
        assert_eq!(
            reported(&fixture, &result)
                .into_iter()
                .filter(|(code, _, _)| *code == DiagnosticCode::CallCycle)
                .collect::<Vec<_>>(),
            [
                (
                    DiagnosticCode::CallCycle,
                    "a()",
                    String::from("call cycle `a` -> `b` -> `a`")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "count()",
                    String::from("call cycle `count` -> `arity` -> `count`")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "wide(1)",
                    String::from("call cycle `wide` -> `narrow` -> `wide`")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "typed(256)",
                    String::from("`typed` calls itself")
                ),
            ]
        );
        assert_eq!(
            reported(&fixture, &result)
                .into_iter()
                .map(|(code, source, _)| (code, source))
                .filter(|(code, _)| *code != DiagnosticCode::CallCycle)
                .collect::<Vec<_>>(),
            [
                (DiagnosticCode::TypeMismatch, "b()"),
                (DiagnosticCode::TypeMismatch, "a()"),
                (DiagnosticCode::ArgumentCountMismatch, "arity(1)"),
                (DiagnosticCode::UnsupportedWordWidth, "12"),
                (DiagnosticCode::WordLiteralOutOfRange, "256"),
            ]
        );
    }

    #[test]
    fn long_call_cycles_have_bounded_messages() {
        let count = MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC + 2;
        let members = (0..count)
            .map(|index| {
                format!(
                    "  spec f{index}() -> Int {{ f{}() }}\n",
                    (index + 1) % count
                )
            })
            .collect::<String>();
        let (fixture, result) = rejected(&members);
        let route = (0..MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC)
            .map(|index| format!("`f{index}`"))
            .collect::<Vec<_>>()
            .join(" -> ");
        assert_eq!(
            reported(&fixture, &result),
            [(
                DiagnosticCode::CallCycle,
                "f0()",
                format!("call cycle {route} -> ... -> `f0`")
            )]
        );
    }

    #[test]
    fn names_and_calls_resolve_only_to_parameters_and_typed_specs() {
        let (fixture, result) = rejected(concat!(
            "  spec f(x: Int) -> Int { y }\n",
            "  spec g(x: Int) -> Int { helper }\n",
            "  spec helper() -> Int { 1 }\n",
            "  spec h() -> Int { missing() }\n",
            "  spec i() -> Int { legacy() }\n",
            "  spec legacy() {}\n",
            "  spec j() -> Int { body() }\n",
            "  impl body() {}\n",
            "  spec k(x: Int, x: Int) -> Int { x }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::UnknownParameter,
                    "y",
                    String::from("`y` is not a parameter of `f`")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "helper",
                    String::from("`helper` is not a parameter of `g`")
                ),
                (
                    DiagnosticCode::UnknownFunction,
                    "missing",
                    String::from("no typed `spec` function named `missing` in this module")
                ),
                (
                    DiagnosticCode::UnknownFunction,
                    "legacy",
                    String::from("`spec` function `legacy` has no typed body and cannot be called")
                ),
                (
                    DiagnosticCode::UnknownFunction,
                    "body",
                    String::from("no typed `spec` function named `body` in this module")
                ),
                (
                    DiagnosticCode::DuplicateParameter,
                    "x",
                    String::from("duplicate parameter `x`")
                ),
            ]
        );
        let notes = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.notes()[0].as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            notes,
            [
                "a bare name in a `spec` body refers to one of its parameters",
                "to call the function `helper`, write `helper()` with its arguments",
                "calls name a typed `spec` declared in the same module",
                "calls name a typed `spec` declared in the same module",
                "`impl` functions have no semantics yet and cannot be called",
                "parameter names must be unique within one function",
            ]
        );
        let [declared] = result.diagnostics[3].secondary_spans() else {
            panic!("an untyped callee must cite its declaration");
        };
        assert_eq!(fixture.source().slice(declared.span()), Some("legacy"));
        let [first] = result.diagnostics[5].secondary_spans() else {
            panic!("a duplicate parameter must cite the first parameter");
        };
        assert_eq!(declared.label(), "declared without a result type here");
        assert_eq!(first.label(), "first parameter is here");
        assert!(first.span().start() < result.diagnostics[5].primary_span().start());
    }

    #[test]
    fn calls_check_arity_argument_types_and_result_types() {
        let (fixture, result) = rejected(concat!(
            "  spec one(x: Word[8]) -> Word[8] { x }\n",
            "  spec two(x: Int, y: Int) -> Int { x + y }\n",
            "  spec int() -> Int { 1 }\n",
            "  spec a() -> Word[8] { one() }\n",
            "  spec b() -> Word[8] { one(1, 2) }\n",
            "  spec c() -> Int { two(1) }\n",
            "  spec d() -> Word[8] { one(256) }\n",
            "  spec e() -> Word[8] { int() }\n",
            "  spec f(w: Word[16]) -> Word[8] { one(w) }\n",
            "  spec g() -> Int { two(1, one(2)) }\n",
            "  spec h() -> Int { one(256) }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::ArgumentCountMismatch,
                    "one()",
                    String::from("`one` takes 1 argument but 0 were supplied")
                ),
                (
                    DiagnosticCode::ArgumentCountMismatch,
                    "one(1, 2)",
                    String::from("`one` takes 1 argument but 2 were supplied")
                ),
                (
                    DiagnosticCode::ArgumentCountMismatch,
                    "two(1)",
                    String::from("`two` takes 2 arguments but 1 was supplied")
                ),
                (
                    DiagnosticCode::WordLiteralOutOfRange,
                    "256",
                    String::from("literal is outside the range of `Word[8]`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "int()",
                    String::from("`int` returns `Int`, but `Word[8]` is required here")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "w",
                    String::from("`w` has type `Word[16]`, but `Word[8]` is required here")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "one(2)",
                    String::from("`one` returns `Word[8]`, but `Int` is required here")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "one(256)",
                    String::from("`one` returns `Word[8]`, but `Int` is required here")
                ),
                (
                    DiagnosticCode::WordLiteralOutOfRange,
                    "256",
                    String::from("literal is outside the range of `Word[8]`")
                ),
            ]
        );
        assert!(result.diagnostics.iter().all(|diagnostic| {
            diagnostic.code() != DiagnosticCode::TypeMismatch
                || diagnostic.notes() == ["Orange has no implicit conversions between types"]
        }));
    }

    #[test]
    fn operators_are_defined_only_for_their_types() {
        let (fixture, result) = rejected(concat!(
            "  spec a(x: Word[8]) -> Word[8] { -x }\n",
            "  spec b(n: Int) -> Int { ~n }\n",
            "  spec c(n: Int) -> Int { n & 1 }\n",
            "  spec d(n: Int) -> Int { n | 1 }\n",
            "  spec e(n: Int) -> Int { n ^ 1 }\n",
            "  spec f(n: Int) -> Int { n << 1 }\n",
            "  spec g(n: Int) -> Int { n >>> 1 }\n",
            "  spec h(x: Word[64]) -> Word[64] { -(x + unknown) }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::UnsupportedOperator,
                    "-",
                    String::from("prefix `-` is not defined for `Word[8]`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "~",
                    String::from("prefix `~` is not defined for `Int`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "&",
                    String::from("`&` is not defined for `Int`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "|",
                    String::from("`|` is not defined for `Int`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "^",
                    String::from("`^` is not defined for `Int`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "<<",
                    String::from("`<<` is not defined for `Int`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    ">>>",
                    String::from("`>>>` is not defined for `Int`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "-",
                    String::from("prefix `-` is not defined for `Word[64]`")
                ),
            ]
        );
        let notes = result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.notes()[0].as_str())
            .collect::<Vec<_>>();
        assert_eq!(
            notes,
            [
                "write `0 - x` for negation modulo 2^8",
                "bitwise operators apply only to `Word[n]` values",
                "bitwise operators apply only to `Word[n]` values",
                "bitwise operators apply only to `Word[n]` values",
                "bitwise operators apply only to `Word[n]` values",
                "shifts and rotations apply only to `Word[n]` values",
                "shifts and rotations apply only to `Word[n]` values",
                "write `0 - x` for negation modulo 2^64",
            ]
        );

        // Every arithmetic operator is defined for every type.
        let members = TYPES
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                format!("  spec f{index}(x: {ty}, y: {ty}) -> {ty} {{ x + y - x * y }}\n")
            })
            .collect::<String>();
        accepted(&members);
    }

    #[test]
    fn shift_and_rotation_amounts_are_literals_below_the_width() {
        let mut members = String::new();
        let mut expected = Vec::new();
        for ty in TYPES.iter().filter(|ty| **ty != CoreType::Int) {
            let bits = ty.word_bits().unwrap();
            for operator in ["<<", ">>", "<<<", ">>>"] {
                members.push_str(&format!(
                    "  spec ok{bits}_{}(x: {ty}) -> {ty} {{ (x {operator} 0) ^ (x {operator} {}) }}\n",
                    members.len(),
                    bits - 1
                ));
            }
        }
        let (fixture, core) = accepted(&members);
        let amounts = core
            .functions
            .iter()
            .flat_map(|function| core_nodes(&fixture, function))
            .filter_map(|(operation, _, _)| {
                operation
                    .strip_prefix("shift ")
                    .map(|rest| rest.rsplit_once(' ').unwrap().1.parse::<u32>().unwrap())
            })
            .collect::<Vec<_>>();
        assert_eq!(amounts.len(), 32);
        assert_eq!(amounts.iter().filter(|amount| **amount == 0).count(), 16);

        members.clear();
        for (index, (ty, amount)) in [
            ("Word[8]", "8"),
            ("Word[16]", "16"),
            ("Word[32]", "0x20"),
            ("Word[64]", "64"),
            ("Word[64]", "0x1_0000_0000_0000_0000"),
            ("Word[8]", "-1"),
            ("Word[8]", "x"),
            ("Word[8]", "(1)"),
            ("Word[8]", "1 + 1"),
        ]
        .into_iter()
        .enumerate()
        {
            let amount_source = if amount == "1 + 1" { "(1 + 1)" } else { amount };
            members.push_str(&format!(
                "  spec bad{index}(x: {ty}) -> {ty} {{ x <<< {amount_source} }}\n"
            ));
            let highest = TYPES
                .into_iter()
                .find(|candidate| candidate.to_string() == ty)
                .and_then(CoreType::word_bits)
                .unwrap()
                - 1;
            expected.push((
                DiagnosticCode::InvalidShiftAmount,
                amount_source,
                format!("`<<<` on `{ty}` needs an amount from 0 through {highest}"),
            ));
        }
        let (fixture, result) = rejected(&members);
        let reported = reported(&fixture, &result);
        assert_eq!(
            reported
                .iter()
                .map(|(code, source, message)| (*code, *source, message.as_str()))
                .collect::<Vec<_>>(),
            expected
                .iter()
                .map(|(code, source, message)| (*code, *source, message.as_str()))
                .collect::<Vec<_>>()
        );
        assert!(result.diagnostics.iter().all(|diagnostic| {
            diagnostic.label() == "amount must be an unsigned integer literal"
        }));
    }

    #[test]
    fn every_word_width_has_exact_literal_bounds() {
        for ty in TYPES.iter().filter(|ty| **ty != CoreType::Int) {
            let bits = ty.word_bits().unwrap();
            let maximum = u128::from(u64::MAX) >> (64 - bits);
            let (_, core) = accepted(&format!(
                "  spec max() -> {ty} {{ {maximum} }}\n  spec max_hex(x: {ty}) -> {ty} {{ x ^ 0x{maximum:x} }}\n"
            ));
            assert_eq!(
                core.functions[0]
                    .body
                    .literal()
                    .and_then(CoreValue::word_as_u64),
                Some(u64::try_from(maximum).unwrap())
            );

            let over = (maximum + 1).to_string();
            let (fixture, result) = rejected(&format!(
                "  spec over() -> {ty} {{ {over} }}\n  spec negative(x: {ty}) -> {ty} {{ x & -1 }}\n"
            ));
            assert_eq!(
                reported(&fixture, &result),
                [
                    (
                        DiagnosticCode::WordLiteralOutOfRange,
                        over.as_str(),
                        format!("literal is outside the range of `{ty}`")
                    ),
                    (
                        DiagnosticCode::NegativeWordLiteral,
                        "-1",
                        format!("`{ty}` literals cannot be negative")
                    ),
                ]
            );
        }
    }

    #[test]
    fn only_exact_decimal_word_widths_resolve() {
        let (_, core) =
            accepted("  spec a(x: Word[8], y: Word[16], z: Word[32], w: Word[64]) -> Int { 0 }\n");
        assert_eq!(
            core.functions[0].parameters,
            [
                CoreType::Word8,
                CoreType::Word16,
                CoreType::Word32,
                CoreType::Word64
            ]
        );
        let (fixture, result) = rejected(concat!(
            "  spec a(x: Word[12]) -> Int { 0 }\n",
            "  spec b(x: Word[128]) -> Int { 0 }\n",
            "  spec c(x: Word[0x20]) -> Int { 0 }\n",
            "  spec d(x: Word[032]) -> Int { 0 }\n",
            "  spec e(x: Word[3_2]) -> Int { 0 }\n",
            "  spec f(x: Word[0]) -> Int { 0 }\n",
        ));
        assert_eq!(
            reported(&fixture, &result)
                .into_iter()
                .map(|(code, source, _)| (code, source))
                .collect::<Vec<_>>(),
            [
                (DiagnosticCode::UnsupportedWordWidth, "12"),
                (DiagnosticCode::UnsupportedWordWidth, "128"),
                (DiagnosticCode::UnsupportedWordWidth, "0x20"),
                (DiagnosticCode::UnsupportedWordWidth, "032"),
                (DiagnosticCode::UnsupportedWordWidth, "3_2"),
                (DiagnosticCode::UnsupportedWordWidth, "0"),
            ]
        );
    }

    #[test]
    fn unresolved_signatures_are_reported_once_without_cascades() {
        let (fixture, result) = rejected(concat!(
            "  spec bad_parameter(x: Word[12]) -> Int { x }\n",
            "  spec bad_result() -> Float { 1 }\n",
            "  spec caller() -> Int { bad_parameter(1) + bad_result() }\n",
            "  spec wrong_count() -> Int { bad_parameter() }\n",
            "  spec hidden(n: Int) -> Int { ~(n + missing) }\n",
            "  spec shifted(n: Int) -> Int { n << n }\n",
        ));
        assert_eq!(
            reported(&fixture, &result)
                .into_iter()
                .map(|(code, source, _)| (code, source))
                .collect::<Vec<_>>(),
            [
                (DiagnosticCode::UnsupportedWordWidth, "12"),
                (DiagnosticCode::UnsupportedType, "Float"),
                (DiagnosticCode::UnsupportedOperator, "~"),
                (DiagnosticCode::UnsupportedOperator, "<<"),
            ]
        );
    }

    #[test]
    fn body_errors_precede_call_graph_errors_and_all_errors_are_ordered() {
        let (fixture, result) = rejected(concat!(
            "  spec loop_a() -> Int { loop_b() }\n",
            "  spec loop_b() -> Int { loop_a() }\n",
            "  spec later(x: Word[8]) -> Word[8] { x + 256 }\n",
            "  spec last() -> Int { nothing }\n",
        ));
        assert_eq!(
            reported(&fixture, &result)
                .into_iter()
                .map(|(code, source, _)| (code, source))
                .collect::<Vec<_>>(),
            [
                (DiagnosticCode::WordLiteralOutOfRange, "256"),
                (DiagnosticCode::UnknownParameter, "nothing"),
                (DiagnosticCode::CallCycle, "loop_a()"),
            ]
        );
    }

    #[test]
    fn expression_events_and_core_nodes_follow_the_normative_accounting() {
        // Lookup and installation (2), the parameter's uniqueness check, name,
        // and width (3), the result name and width (2), and one event for each
        // of `^`, `~`, and `x`, plus the literal's own event, prefix, and one
        // significant digit (6): 13 analysis events. Core is the module and
        // one function node, one result-type node, one parameter-type node,
        // and the four expression nodes `x`, `~x`, `1`, and `^`: 8 nodes, each
        // one more event.
        let operators = module("  spec f(x: Word[8]) -> Word[8] { ~x ^ 1 }\n");
        // `g`: 2 + 1 result + 3 literal = 6. `f`: 2 + 1 result + 1 group +
        // 1 call = 5. Core: module + (2 + 1) + (2 + 1) = 7 nodes. The call
        // graph check consumes no events.
        let calls = module("  spec g() -> Int { 1 }\n  spec f() -> Int { (g()) }\n");
        // 2 + 3 parameter + 2 result + `<<<` + `x` + amount literal event,
        // prefix, and two significant digits = 13. Core: module + 2 + 1 + 2.
        let shift = module("  spec s(x: Word[32]) -> Word[32] { x <<< 0x1f }\n");
        for (fixture, events, nodes) in [(&operators, 21, 8), (&calls, 18, 7), (&shift, 19, 6)] {
            let exact = fixture.analyze_with(Limits {
                events,
                nodes,
                ..Limits::DEFAULT
            });
            assert_eq!(exact.diagnostics, []);
            assert!(exact.core.is_some());
            for (limits, label) in [
                (
                    Limits {
                        events: events - 1,
                        nodes,
                        ..Limits::DEFAULT
                    },
                    "semantic event budget exhausted",
                ),
                (
                    Limits {
                        events,
                        nodes: nodes - 1,
                        ..Limits::DEFAULT
                    },
                    "typed Core node budget exhausted",
                ),
            ] {
                let first = fixture.analyze_with(limits);
                assert_eq!(first, fixture.analyze_with(limits));
                assert!(first.core.is_none());
                assert_eq!(first.diagnostics.len(), 1);
                assert_eq!(
                    first.diagnostics[0].code(),
                    DiagnosticCode::SemanticResourceLimit
                );
                assert_eq!(first.diagnostics[0].label(), label);
            }
        }
    }

    #[test]
    fn expression_storage_failures_return_no_partial_core() {
        let fixture = module("  spec g() -> Int { 1 }\n  spec f() -> Int { g() }\n");
        let node_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_core_node_slot = |_| false;
            analyzer.run()
        };
        let edge_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_call_edge_slot = |_| false;
            analyzer.run()
        };
        for (run, detail, source) in [
            (
                &node_failure as &dyn Fn() -> AnalysisResult,
                "typed Core expression storage allocation failed",
                "1",
            ),
            (&edge_failure, "call graph storage allocation failed", "g()"),
        ] {
            let first = run();
            assert_eq!(first, run());
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            let diagnostic = &first.diagnostics[0];
            assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
            assert_eq!(
                fixture.source().slice(diagnostic.primary_span()),
                Some(source)
            );
            assert_eq!(diagnostic.label(), detail);
        }
    }

    #[test]
    fn bindings_and_conversions_build_typed_core_in_source_order() {
        let (fixture, core) = accepted(concat!(
            "  spec load16(lo: Word[8], hi: Word[8]) -> Word[16] {\n",
            "    let wide: Word[16] = lo as Word[16];\n",
            "    let high: Word[16] = (hi as Word[16]) << 8;\n",
            "    wide | high\n",
            "  }\n",
            "  spec square(x: Word[32]) -> Int { let n: Int = x as Int; n * n }\n",
            "  spec plain() -> Int { 1 }\n",
        ));
        let owned = |rows: &[(&str, &'static str, CoreType)]| {
            rows.iter()
                .map(|(operation, source, ty)| ((*operation).to_owned(), *source, *ty))
                .collect::<Vec<_>>()
        };
        let load = &core.functions[0];
        assert_eq!(
            load.locals
                .iter()
                .map(|local| (
                    local.name(),
                    fixture.source().slice(local.span()).unwrap(),
                    fixture.source().slice(local.name_span()).unwrap(),
                    local.ty()
                ))
                .collect::<Vec<_>>(),
            [
                (
                    "wide",
                    "let wide: Word[16] = lo as Word[16];",
                    "wide",
                    CoreType::Word16
                ),
                (
                    "high",
                    "let high: Word[16] = (hi as Word[16]) << 8;",
                    "high",
                    CoreType::Word16
                ),
            ]
        );
        assert_eq!(
            expression_nodes(&fixture, &load.locals[0].value),
            owned(&[
                ("parameter 0", "lo", CoreType::Word8),
                ("convert from Word[8]", "lo as Word[16]", CoreType::Word16),
            ])
        );
        assert_eq!(
            expression_nodes(&fixture, &load.locals[1].value),
            owned(&[
                ("parameter 1", "hi", CoreType::Word8),
                ("convert from Word[8]", "hi as Word[16]", CoreType::Word16),
                ("shift << 8", "(hi as Word[16]) << 8", CoreType::Word16),
            ])
        );
        assert_eq!(
            core_nodes(&fixture, load),
            owned(&[
                ("local 0", "wide", CoreType::Word16),
                ("local 1", "high", CoreType::Word16),
                ("infix |", "wide | high", CoreType::Word16),
            ])
        );
        assert_eq!(
            core_nodes(&fixture, &core.functions[1]),
            owned(&[
                ("local 0", "n", CoreType::Int),
                ("local 0", "n", CoreType::Int),
                ("infix *", "n * n", CoreType::Int),
            ])
        );
        assert!(core.functions[2].locals.is_empty());
    }

    #[test]
    fn a_conversion_operand_has_the_type_of_its_first_typed_leaf() {
        let (fixture, core) = accepted(concat!(
            "  spec k() -> Word[8] { 7 }\n",
            "  spec a(x: Word[8]) -> Int { (x + 1) as Int }\n",
            "  spec b(x: Word[8]) -> Int { (1 + x) as Int }\n",
            "  spec c(x: Word[32]) -> Word[8] { ((x << 3) ^ 0xff) as Word[8] }\n",
            "  spec d() -> Word[64] { k() as Word[64] }\n",
            "  spec e(x: Word[8]) -> Int { ((x as Word[64]) * 3) as Int }\n",
            "  spec f(x: Int) -> Int { -x as Int }\n",
            "  spec g(x: Word[16]) -> Word[16] { ~x as Word[16] }\n",
        ));
        let conversions = core
            .functions
            .iter()
            .map(|function| {
                let root = function.body.root().unwrap();
                let CoreNodeKind::Convert { from } = root.kind else {
                    return None;
                };
                Some((
                    function.name.as_str(),
                    from,
                    root.ty,
                    fixture.source().slice(root.span).unwrap(),
                ))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            conversions,
            [
                None,
                Some(("a", CoreType::Word8, CoreType::Int, "(x + 1) as Int")),
                Some(("b", CoreType::Word8, CoreType::Int, "(1 + x) as Int")),
                Some((
                    "c",
                    CoreType::Word32,
                    CoreType::Word8,
                    "((x << 3) ^ 0xff) as Word[8]"
                )),
                Some(("d", CoreType::Word8, CoreType::Word64, "k() as Word[64]")),
                Some((
                    "e",
                    CoreType::Word64,
                    CoreType::Int,
                    "((x as Word[64]) * 3) as Int"
                )),
                Some(("f", CoreType::Int, CoreType::Int, "-x as Int")),
                Some(("g", CoreType::Word16, CoreType::Word16, "~x as Word[16]")),
            ]
        );
        // The literal takes the leaf's type.
        assert_eq!(
            core_nodes(&fixture, &core.functions[2])[0],
            (String::from("literal 0x01"), "1", CoreType::Word8)
        );
    }

    #[test]
    fn binding_names_are_unique_and_in_scope_only_after_their_binding() {
        let (fixture, result) = rejected(concat!(
            "  spec dup(x: Int) -> Int { let x: Int = 1; let t: Int = x; let t: Int = 2; t }\n",
            "  spec early() -> Int { let a: Int = b; let b: Int = b; b }\n",
            "  spec unknown(x: Int) -> Int { let a: Int = x; c }\n",
            "  spec function_name() -> Int { let a: Int = 1; early }\n",
            "  spec typed(x: Word[8]) -> Word[32] { let t: Word[8] = x; t }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::DuplicateBinding,
                    "x",
                    String::from("duplicate binding `x`")
                ),
                (
                    DiagnosticCode::DuplicateBinding,
                    "t",
                    String::from("duplicate binding `t`")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "b",
                    String::from("`b` is used before it is bound")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "b",
                    String::from("`b` is used before it is bound")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "c",
                    String::from("`c` is not a parameter or binding of `unknown`")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "early",
                    String::from("`early` is not a parameter or binding of `function_name`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "t",
                    String::from("`t` has type `Word[8]`, but `Word[32]` is required here")
                ),
            ]
        );
        let secondary = |index: usize| {
            let [secondary] = result.diagnostics[index].secondary_spans() else {
                panic!("diagnostic {index} must cite one earlier span");
            };
            (
                fixture.source().slice(secondary.span()).unwrap(),
                secondary.label(),
                secondary.span().start() < result.diagnostics[index].primary_span().start(),
            )
        };
        assert_eq!(secondary(0), ("x", "the parameter is here", true));
        assert_eq!(secondary(1), ("t", "the first binding is here", true));
        assert_eq!(secondary(2), ("b", "the binding is here", false));
        assert_eq!(secondary(3), ("b", "the binding is here", true));
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.notes()[0].as_str())
                .collect::<Vec<_>>(),
            [
                "each parameter and binding of a function has its own name; \
                 Orange has no shadowing",
                "each parameter and binding of a function has its own name; \
                 Orange has no shadowing",
                "a binding is in scope after its own `;`, for the bindings that follow it \
                 and the result",
                "a binding is in scope after its own `;`, for the bindings that follow it \
                 and the result",
                "a bare name in a `spec` body refers to one of its parameters or bindings",
                "to call the function `early`, write `early()` with its arguments",
                "Orange has no implicit conversions between types",
            ]
        );
    }

    #[test]
    fn conversion_errors_are_reported_once_in_checking_order() {
        let (fixture, result) = rejected(concat!(
            "  spec untyped() -> Word[8] { (1 + 2) as Word[8] }\n",
            "  spec shifted() -> Int { (1 << 3) as Int }\n",
            "  spec mismatch(x: Word[32]) -> Word[8] { x as Word[16] }\n",
            "  spec target(x: Word[32]) -> Word[8] { x as Word[12] }\n",
            "  spec unknown() -> Int { (y + 1) as Int }\n",
            "  spec range(x: Word[8]) -> Int { (x + 256) as Int }\n",
            "  spec both(x: Word[8]) -> Word[8] { (x + y) as Word[16] }\n",
            "  spec callee() -> Int { missing() as Int }\n",
            "  spec untyped_binding() -> Int { let q: Float = 1; q as Int }\n",
            "  spec inner(x: Word[8]) -> Int { (x as Word[7]) as Int }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::UntypedConversionOperand,
                    "(1 + 2)",
                    String::from("the operand of `as` has no type of its own")
                ),
                (
                    DiagnosticCode::UntypedConversionOperand,
                    "(1 << 3)",
                    String::from("the operand of `as` has no type of its own")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "Word[16]",
                    String::from(
                        "this conversion gives `Word[16]`, but `Word[8]` is required here"
                    )
                ),
                (
                    DiagnosticCode::UnsupportedWordWidth,
                    "12",
                    String::from("`Word` width must be exactly 8, 16, 32, or 64")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "y",
                    String::from("`y` is not a parameter of `unknown`")
                ),
                (
                    DiagnosticCode::WordLiteralOutOfRange,
                    "256",
                    String::from("literal is outside the range of `Word[8]`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "Word[16]",
                    String::from(
                        "this conversion gives `Word[16]`, but `Word[8]` is required here"
                    )
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "y",
                    String::from("`y` is not a parameter of `both`")
                ),
                (
                    DiagnosticCode::UnknownFunction,
                    "missing",
                    String::from("no typed `spec` function named `missing` in this module")
                ),
                (
                    DiagnosticCode::UnsupportedType,
                    "Float",
                    String::from("unsupported binding type `Float`")
                ),
                (
                    DiagnosticCode::UnsupportedWordWidth,
                    "7",
                    String::from("`Word` width must be exactly 8, 16, 32, or 64")
                ),
            ]
        );
        assert_eq!(
            result.diagnostics[0].notes(),
            [
                "write the literal where its type is required, or give it a type with a `let` \
              binding"
            ]
        );
        assert_eq!(
            result.diagnostics[2].notes(),
            ["`as` gives exactly the type written after it"]
        );
    }

    #[test]
    fn calls_inside_bindings_and_conversions_join_the_call_graph() {
        let (fixture, result) = rejected(concat!(
            "  spec a() -> Int { let t: Int = b(); t }\n",
            "  spec b() -> Int { c() as Int }\n",
            "  spec c() -> Word[8] { (a() as Word[8]) + 1 }\n",
            "  spec d() -> Int { let q: Word[8] = e(); q as Int }\n",
            "  spec e() -> Word[16] { d() as Word[16] }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::TypeMismatch,
                    "e()",
                    String::from("`e` returns `Word[16]`, but `Word[8]` is required here")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "a()",
                    String::from("call cycle `a` -> `b` -> `c` -> `a`")
                ),
                (
                    DiagnosticCode::CallCycle,
                    "d()",
                    String::from("call cycle `d` -> `e` -> `d`")
                ),
            ]
        );
    }

    #[test]
    fn binding_and_conversion_events_and_core_nodes_follow_the_normative_accounting() {
        // Lookup and installation (2), the parameter's uniqueness check, name,
        // and width (3), the result name (1); the binding's uniqueness check
        // and type name (2), `as` and its target name (2), and `x` (1); and
        // the result `n` (1): 12 analysis events. Core is the module, one
        // function node, one result-type node, one parameter-type node, the
        // body node `n`, one binding node, one binding-type node, and the
        // binding's nodes `x` and `as`: 9 nodes, each one more event.
        let fixture = module("  spec f(x: Word[8]) -> Int { let n: Int = x as Int; n }\n");
        let (events, nodes) = (21, 9);
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert!(exact.core.is_some());
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label);
        }
    }

    #[test]
    fn binding_storage_failures_return_no_partial_core() {
        let fixture = module("  spec f(x: Int) -> Int { let t: Int = x; t }\n");
        let name_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_core_name = |_, _| false;
            analyzer.run()
        };
        let node_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_core_node_slot = |_| false;
            analyzer.run()
        };
        for (run, detail, source) in [
            (
                &name_failure as &dyn Fn() -> AnalysisResult,
                "typed Core name storage allocation failed",
                "t",
            ),
            (
                &node_failure,
                "typed Core expression storage allocation failed",
                "x",
            ),
        ] {
            let first = run();
            assert_eq!(first, run());
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            let diagnostic = &first.diagnostics[0];
            assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
            assert_eq!(
                fixture.source().slice(diagnostic.primary_span()),
                Some(source)
            );
            assert_eq!(diagnostic.label(), detail);
        }
    }

    #[test]
    fn rejects_foreign_spans_in_bindings_and_conversions() {
        let text = "edition 2026; module values { \
                    spec value(x: Word[8]) -> Int { let t: Int = x as Int; t } }\n";
        let first = Fixture::new(text);
        let second = Fixture::new(text);
        let foreign = match &second.ast.module.functions[0].body {
            FunctionBody::Typed(body) => body,
            FunctionBody::Empty => unreachable!(),
        };
        let foreign_binding = &foreign.bindings[0];
        let ExpressionKind::Conversion(foreign_conversion) = &foreign_binding.value.kind else {
            unreachable!();
        };
        fn conversion_mut(ast: &mut SyntaxTree) -> &mut ConversionExpression {
            match &mut typed_body_mut(ast).bindings[0].value.kind {
                ExpressionKind::Conversion(conversion) => conversion,
                _ => unreachable!(),
            }
        }
        let mutations: [&dyn Fn(&mut SyntaxTree); 8] = [
            &|ast| typed_body_mut(ast).bindings[0].span = foreign_binding.span,
            &|ast| typed_body_mut(ast).bindings[0].name.span = foreign_binding.name.span,
            &|ast| typed_body_mut(ast).bindings[0].ty.span = foreign_binding.ty.span,
            &|ast| typed_body_mut(ast).bindings[0].ty.name.span = foreign_binding.ty.name.span,
            &|ast| typed_body_mut(ast).bindings[0].value.span = foreign_binding.value.span,
            &|ast| conversion_mut(ast).keyword_span = foreign_conversion.keyword_span,
            &|ast| conversion_mut(ast).target.span = foreign_conversion.target.span,
            &|ast| conversion_mut(ast).operand.span = foreign_conversion.operand.span,
        ];
        assert!(analyze(first.source(), &first.ast).core.is_some());
        for (case_index, mutate) in mutations.iter().enumerate() {
            let mut ast = first.ast.clone();
            mutate(&mut ast);
            let result = analyze(first.source(), &ast);
            assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
            assert!(result.core.is_none(), "case {case_index}");
            assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
            assert_eq!(
                result.diagnostics[0].code(),
                DiagnosticCode::InvalidSemanticInput,
                "case {case_index}"
            );
        }
    }

    #[test]
    fn let_and_as_are_ordinary_names_in_semantics() {
        let (fixture, core) = accepted(concat!(
            "  spec let(as: Int) -> Int { let let: Int = as; let as2: Int = let; let + as2 }\n",
            "  spec as(let: Word[8]) -> Int { (let) as Int }\n",
        ));
        assert_eq!(
            core_nodes(&fixture, &core.functions[0]),
            [
                (String::from("local 0"), "let", CoreType::Int),
                (String::from("local 1"), "as2", CoreType::Int),
                (String::from("infix +"), "let + as2", CoreType::Int),
            ]
        );
        assert_eq!(core.functions[1].name, "as");
    }

    fn array_of(element: CoreType, length: u32) -> CoreType {
        CoreType::Array(ArrayType::new(element, length).unwrap())
    }

    #[test]
    fn arrays_and_indices_build_typed_core_in_postorder() {
        let (fixture, core) = accepted(concat!(
            "  spec rot(x: Word[32]^4) -> Word[32]^4 { [x[1], x[2], x[3], x[0]] }\n",
            "  spec pick(x: Word[32]^4) -> Word[32] { rot(x)[3] ^ x[0x0] }\n",
            "  spec pair() -> Int^2 { let p: Int^2 = [1, -2,]; p }\n",
            "  spec first() -> Int { pair()[0] }\n",
        ));
        let words = array_of(CoreType::Word32, 4);
        let owned = |rows: &[(&str, &'static str, CoreType)]| {
            rows.iter()
                .map(|(operation, source, ty)| ((*operation).to_owned(), *source, *ty))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            core.functions
                .iter()
                .map(|function| (function.parameters.clone(), function.result_type))
                .collect::<Vec<_>>(),
            [
                (vec![words], words),
                (vec![words], CoreType::Word32),
                (vec![], array_of(CoreType::Int, 2)),
                (vec![], CoreType::Int),
            ]
        );
        assert_eq!(
            core_nodes(&fixture, &core.functions[0]),
            owned(&[
                ("parameter 0", "x", words),
                ("index 1", "x[1]", CoreType::Word32),
                ("parameter 0", "x", words),
                ("index 2", "x[2]", CoreType::Word32),
                ("parameter 0", "x", words),
                ("index 3", "x[3]", CoreType::Word32),
                ("parameter 0", "x", words),
                ("index 0", "x[0]", CoreType::Word32),
                ("array of 4", "[x[1], x[2], x[3], x[0]]", words),
            ])
        );
        assert_eq!(
            core_nodes(&fixture, &core.functions[1]),
            owned(&[
                ("parameter 0", "x", words),
                ("call #0 with 1", "rot(x)", words),
                ("index 3", "rot(x)[3]", CoreType::Word32),
                ("parameter 0", "x", words),
                ("index 0", "x[0x0]", CoreType::Word32),
                ("infix ^", "rot(x)[3] ^ x[0x0]", CoreType::Word32),
            ])
        );
        let pair = &core.functions[2];
        assert_eq!(pair.locals[0].ty(), array_of(CoreType::Int, 2));
        assert_eq!(
            expression_nodes(&fixture, &pair.locals[0].value),
            owned(&[
                ("literal 1", "1", CoreType::Int),
                ("literal -2", "-2", CoreType::Int),
                ("array of 2", "[1, -2,]", array_of(CoreType::Int, 2)),
            ])
        );
        assert_eq!(
            core_nodes(&fixture, pair),
            owned(&[("local 0", "p", array_of(CoreType::Int, 2))])
        );
        assert_eq!(
            core_nodes(&fixture, &core.functions[3]),
            owned(&[
                ("call #2 with 0", "pair()", array_of(CoreType::Int, 2)),
                ("index 0", "pair()[0]", CoreType::Int),
            ])
        );
    }

    #[test]
    fn array_lengths_resolve_only_as_exact_decimals_from_1_through_256() {
        for (length, resolved) in [
            ("1", Some(1)),
            ("2", Some(2)),
            ("16", Some(16)),
            ("255", Some(255)),
            ("256", Some(256)),
            ("0", None),
            ("257", None),
            ("1000", None),
            ("01", None),
            ("007", None),
            ("0x10", None),
            ("0b1", None),
            ("1_6", None),
            ("99999999999999999999", None),
        ] {
            let members = format!("  spec f(x: Word[8]^{length}) -> Word[8] {{ 1 }}\n");
            let fixture = module(&members);
            let result = fixture.analyze();
            match resolved {
                Some(resolved) => {
                    assert_eq!(result.diagnostics, [], "{length}");
                    assert_eq!(
                        result.core.unwrap().functions[0].parameters,
                        [array_of(CoreType::Word8, resolved)]
                    );
                }
                None => {
                    assert!(result.core.is_none(), "{length}");
                    assert_eq!(
                        reported(&fixture, &result),
                        [(
                            DiagnosticCode::UnsupportedArrayLength,
                            length,
                            String::from(
                                "an array length must be a decimal integer from 1 through 256"
                            )
                        )],
                        "{length}"
                    );
                    assert_eq!(
                        result.diagnostics[0].notes(),
                        ["write the length in decimal without leading zeros, as in `Word[32]^16`"]
                    );
                }
            }
        }
        // The element type is resolved first; an unresolved element type is
        // reported alone.
        let (fixture, result) = rejected(concat!(
            "  spec a(x: Float^4) -> Int { 1 }\n",
            "  spec b(x: Word^4) -> Int { 1 }\n",
            "  spec c(x: Word[7]^0) -> Int { 1 }\n",
            "  spec d() -> Int^0 { [1] }\n",
            "  spec e() -> Int { let t: Word[8]^300 = [1]; 1 }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::UnsupportedType,
                    "Float^4",
                    String::from("unsupported parameter type `Float`")
                ),
                (
                    DiagnosticCode::UnsupportedWordWidth,
                    "Word",
                    String::from("`Word` requires an exact width of 8, 16, 32, or 64")
                ),
                (
                    DiagnosticCode::UnsupportedWordWidth,
                    "7",
                    String::from("`Word` width must be exactly 8, 16, 32, or 64")
                ),
                (
                    DiagnosticCode::UnsupportedArrayLength,
                    "0",
                    String::from("an array length must be a decimal integer from 1 through 256")
                ),
                (
                    DiagnosticCode::UnsupportedArrayLength,
                    "300",
                    String::from("an array length must be a decimal integer from 1 through 256")
                ),
            ]
        );
    }

    #[test]
    fn array_errors_are_reported_once_in_checking_order() {
        let (fixture, result) = rejected(concat!(
            "  spec count(x: Word[8]) -> Word[8]^3 { [x, x] }\n",
            "  spec one(x: Word[8]) -> Word[8]^2 { [x] }\n",
            "  spec element(x: Word[8]) -> Word[8]^2 { [x, 256] }\n",
            "  spec both() -> Word[8]^2 { [y, 1, 2] }\n",
            "  spec range(x: Word[8]^4) -> Word[8] { x[4] }\n",
            "  spec huge(x: Word[8]^4) -> Word[8] { x[99999999999999999999] }\n",
            "  spec scalar(x: Word[8]) -> Word[8] { x[0] }\n",
            "  spec mismatch(x: Word[8]^4) -> Word[16] { x[0] }\n",
            "  spec mismatch_range(x: Word[8]^4) -> Word[16] { x[9] }\n",
            "  spec unknown() -> Word[8] { z[0] }\n",
            "  spec whole(x: Word[8]^4) -> Word[8] { x }\n",
            "  spec other(x: Word[8]^4) -> Word[16] { x }\n",
            "  spec literal() -> Word[8]^4 { 1 }\n",
            "  spec array_for_scalar() -> Int { [1] }\n",
            "  spec nested() -> Int^1 { [[1]] }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::ArrayLengthMismatch,
                    "[x, x]",
                    String::from("this array has 2 elements, but `Word[8]^3` has 3")
                ),
                (
                    DiagnosticCode::ArrayLengthMismatch,
                    "[x]",
                    String::from("this array has 1 element, but `Word[8]^2` has 2")
                ),
                (
                    DiagnosticCode::WordLiteralOutOfRange,
                    "256",
                    String::from("literal is outside the range of `Word[8]`")
                ),
                (
                    DiagnosticCode::ArrayLengthMismatch,
                    "[y, 1, 2]",
                    String::from("this array has 3 elements, but `Word[8]^2` has 2")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "y",
                    String::from("`y` is not a parameter of `both`")
                ),
                (
                    DiagnosticCode::IndexOutOfRange,
                    "4",
                    String::from("index `4` is out of range for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::IndexOutOfRange,
                    "99999999999999999999",
                    String::from("index `99999999999999999999` is out of range for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::NotAnArray,
                    "x",
                    String::from("only an array can be indexed, but this has type `Word[8]`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "x[0]",
                    String::from(
                        "this element has type `Word[8]`, but `Word[16]` is required here"
                    )
                ),
                (
                    DiagnosticCode::IndexOutOfRange,
                    "9",
                    String::from("index `9` is out of range for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "x[9]",
                    String::from(
                        "this element has type `Word[8]`, but `Word[16]` is required here"
                    )
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "z",
                    String::from("`z` is not a parameter of `unknown`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "x",
                    String::from("`x` has type `Word[8]^4`, but `Word[8]` is required here")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "x",
                    String::from("`x` has type `Word[8]^4`, but `Word[16]` is required here")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "1",
                    String::from("an integer literal cannot have type `Word[8]^4`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "[1]",
                    String::from("an array literal cannot have type `Int`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "[1]",
                    String::from("an array literal cannot have type `Int`")
                ),
            ]
        );
        let notes = |index: usize| result.diagnostics[index].notes().to_vec();
        assert_eq!(
            notes(0),
            ["an array literal lists every element of its type exactly once"]
        );
        assert_eq!(
            result.diagnostics[5].label(),
            "indices run from 0 through 3"
        );
        assert_eq!(
            notes(5),
            ["a literal index must be less than the array's length"]
        );
        assert_eq!(result.diagnostics[7].label(), "`Word[8]` has no elements");
        assert_eq!(
            notes(12),
            ["select one element with an index, such as `x[0]`"]
        );
        assert_eq!(
            notes(13),
            ["Orange has no implicit conversions between types"]
        );
        assert_eq!(
            notes(14),
            ["an array value is written `[e0, e1, ...]`, one element per index"]
        );
        assert_eq!(
            notes(15),
            ["an array literal is written where an array type `T^n` is required"]
        );
    }

    #[test]
    fn operators_and_conversions_apply_to_elements_not_arrays() {
        let (fixture, result) = rejected(concat!(
            "  spec add(x: Word[8]^4) -> Word[8]^4 { x + x }\n",
            "  spec xor(x: Word[8]^4) -> Word[8]^4 { x ^ x }\n",
            "  spec rot(x: Word[8]^4) -> Word[8]^4 { x <<< 1 }\n",
            "  spec not(x: Word[8]^4) -> Word[8]^4 { ~x }\n",
            "  spec neg(x: Int^4) -> Int^4 { -x }\n",
            "  spec convert(x: Word[8]^4) -> Int { x as Int }\n",
            "  spec convert_call() -> Word[8] { rows() as Word[8] }\n",
            "  spec convert_literal() -> Int { [1] as Int }\n",
            "  spec rows() -> Word[8]^2 { [1, 2] }\n",
            // An undefined `as` stops there, as every undefined operator
            // does: `back()` is not examined, so it closes no cycle.
            "  spec convert_cycle() -> Int { back() as Int }\n",
            "  spec back() -> Word[8]^1 { [convert_cycle() as Word[8]] }\n",
        ));
        let note = String::from(ARRAY_OPERATOR_NOTE);
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::UnsupportedOperator,
                    "+",
                    String::from("`+` is not defined for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "^",
                    String::from("`^` is not defined for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "<<<",
                    String::from("`<<<` is not defined for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "~",
                    String::from("prefix `~` is not defined for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "-",
                    String::from("prefix `-` is not defined for `Int^4`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "as",
                    String::from("`as` is not defined for `Word[8]^4`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "as",
                    String::from("`as` is not defined for `Word[8]^2`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "as",
                    String::from("`as` is not defined for an array")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "as",
                    String::from("`as` is not defined for `Word[8]^1`")
                ),
            ]
        );
        for diagnostic in &result.diagnostics[..5] {
            assert_eq!(diagnostic.notes(), [note.as_str()]);
        }
        for diagnostic in &result.diagnostics[5..] {
            assert_eq!(
                diagnostic.notes(),
                ["convert each element, such as `x[0] as Int`"]
            );
        }
        // Operators on elements and conversions of elements are ordinary.
        let (fixture, core) = accepted(concat!(
            "  spec mix(x: Word[8]^2, n: Int^2) -> Int {\n",
            "    ((x[0] ^ ~x[1]) as Int) + -n[1] * ((x[1] <<< 3) as Int)\n",
            "  }\n",
        ));
        let operations = core_nodes(&fixture, &core.functions[0])
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>();
        assert!(
            operations.contains(&String::from("index 1")),
            "{operations:?}"
        );
    }

    #[test]
    fn array_events_and_core_nodes_follow_the_normative_accounting() {
        // Lookup and installation (2); the parameter's uniqueness check,
        // name, width, and length (4); the result's name, width, and length
        // (3); the array literal (1); the index `[1]`, its prefix and one
        // significant digit, and its base `x` (4); and the literal `0x05`,
        // its prefix, and one significant digit (3): 17 analysis events.
        // Core is the module, one function node, one result-type node, one
        // parameter-type node, and the body nodes `x`, `x[1]`, `0x05`, and
        // the array: 8 nodes, each one more event.
        let fixture = module("  spec f(x: Word[8]^2) -> Word[8]^2 { [x[1], 0x05] }\n");
        let (events, nodes) = (25, 8);
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert!(exact.core.is_some());
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label);
        }
        // An index literal decodes against the significant-bit limit before
        // its range is checked.
        let fixture = module(&format!(
            "  spec f(x: Word[8]^2) -> Word[8] {{ x[0x{}] }}\n",
            "f".repeat(4097)
        ));
        let result = fixture.analyze();
        assert!(result.core.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::IntegerMagnitudeLimit
        );
    }

    #[test]
    fn array_storage_failures_return_no_partial_core() {
        let fixture = module("  spec f(x: Int) -> Int^2 { [x, x] }\n");
        let first = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_core_node_slot =
                |nodes| nodes.len() < 2 && nodes.try_reserve(1).is_ok();
            analyzer.run()
        };
        let result = first();
        assert_eq!(result, first());
        assert!(result.core.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some("[x, x]")
        );
        assert_eq!(
            diagnostic.label(),
            "typed Core expression storage allocation failed"
        );
    }

    #[test]
    fn rejects_foreign_spans_in_arrays_and_indices() {
        let text = "edition 2026; module values { \
                    spec value(x: Word[8]^2) -> Word[8]^2 { [x[1], x[0]] } }\n";
        let first = Fixture::new(text);
        let second = Fixture::new(text);
        let foreign_function = &second.ast.module.functions[0];
        let foreign = match &foreign_function.body {
            FunctionBody::Typed(body) => body,
            FunctionBody::Empty => unreachable!(),
        };
        let ExpressionKind::Array(foreign_array) = &foreign.expression.kind else {
            unreachable!();
        };
        let ExpressionKind::Index(foreign_index) = &foreign_array.elements[0].kind else {
            unreachable!();
        };
        let foreign_length = foreign_function.parameters[0].ty.length_span;
        let foreign_result_length = foreign.result_type.length_span;
        fn index_mut(ast: &mut SyntaxTree) -> &mut IndexExpression {
            let ExpressionKind::Array(array) = &mut typed_body_mut(ast).expression.kind else {
                unreachable!();
            };
            match &mut array.elements[0].kind {
                ExpressionKind::Index(index) => index,
                _ => unreachable!(),
            }
        }
        let mutations: [&dyn Fn(&mut SyntaxTree); 6] = [
            &|ast| ast.module.functions[0].parameters[0].ty.length_span = foreign_length,
            &|ast| typed_body_mut(ast).result_type.length_span = foreign_result_length,
            &|ast| typed_body_mut(ast).expression.span = foreign.expression.span,
            &|ast| {
                let ExpressionKind::Array(array) = &mut typed_body_mut(ast).expression.kind else {
                    unreachable!();
                };
                array.elements[1].span = foreign_array.elements[1].span;
            },
            &|ast| index_mut(ast).index.span = foreign_index.index.span,
            &|ast| index_mut(ast).base.span = foreign_index.base.span,
        ];
        assert!(analyze(first.source(), &first.ast).core.is_some());
        for (case_index, mutate) in mutations.iter().enumerate() {
            let mut ast = first.ast.clone();
            mutate(&mut ast);
            let result = analyze(first.source(), &ast);
            assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
            assert!(result.core.is_none(), "case {case_index}");
            assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
            assert_eq!(
                result.diagnostics[0].code(),
                DiagnosticCode::InvalidSemanticInput,
                "case {case_index}"
            );
        }
    }

    /// Renders a function's loops as `(index, accumulator, type, range,
    /// visible bindings, scope)`.
    fn loop_headers(
        function: &CoreFunction,
    ) -> Vec<(String, String, CoreType, String, u32, Vec<u32>)> {
        function
            .loops
            .iter()
            .map(|r#loop| {
                (
                    r#loop.index_name().to_owned(),
                    r#loop.accumulator_name().to_owned(),
                    r#loop.ty(),
                    format!("{}..{}", r#loop.start(), r#loop.end()),
                    r#loop.visible_locals(),
                    r#loop.scope().to_vec(),
                )
            })
            .collect()
    }

    #[test]
    fn loops_updates_fills_and_selections_build_typed_core_in_postorder() {
        let (fixture, core) = accepted(concat!(
            "  spec sum(x: Int^4) -> Int { for i in 0..4 with s: Int = 0 { s + x[i] } }\n",
            "  spec odd(x: Word[8]^8) -> Word[8]^4 {\n",
            "    for i in 0..4 with s: Word[8]^4 = [0; 4] { s with [i] = x[2 * i + 1] }\n",
            "  }\n",
            "  spec grid() -> Int {\n",
            "    let base: Int = 10;\n",
            "    for i in 0..2 with a: Int = base { for j in 0..3 with b: Int = a { b + i * j } }\n",
            "  }\n",
            "  spec first() -> Int {\n",
            "    for i in 0..2 with a: Int = for j in 5..7 with b: Int = 0 { b + j } { a + i }\n",
            "  }\n",
        ));
        let owned = |rows: &[(&str, &'static str, CoreType)]| {
            rows.iter()
                .map(|(operation, source, ty)| ((*operation).to_owned(), *source, *ty))
                .collect::<Vec<_>>()
        };
        let ints = array_of(CoreType::Int, 4);
        let bytes = array_of(CoreType::Word8, 8);
        let half = array_of(CoreType::Word8, 4);

        let sum = &core.functions[0];
        assert_eq!(
            core_nodes(&fixture, sum),
            owned(&[
                ("literal 0", "0", CoreType::Int),
                (
                    "loop #0",
                    "for i in 0..4 with s: Int = 0 { s + x[i] }",
                    CoreType::Int
                ),
            ])
        );
        assert_eq!(
            loop_headers(sum),
            [(
                String::from("i"),
                String::from("s"),
                CoreType::Int,
                String::from("0..4"),
                0,
                vec![0]
            )]
        );
        assert_eq!(
            expression_nodes(&fixture, sum.loops[0].step()),
            owned(&[
                ("accumulator of loop #0", "s", CoreType::Int),
                ("parameter 0", "x", ints),
                ("index of loop #0", "i", CoreType::Int),
                ("select", "x[i]", CoreType::Int),
                ("infix +", "s + x[i]", CoreType::Int),
            ])
        );

        let odd = &core.functions[1];
        assert_eq!(
            core_nodes(&fixture, odd)
                .into_iter()
                .map(|(operation, _, ty)| (operation, ty))
                .collect::<Vec<_>>(),
            [
                (String::from("literal 0x00"), CoreType::Word8),
                (String::from("fill"), half),
                (String::from("loop #0"), half),
            ]
        );
        assert_eq!(
            expression_nodes(&fixture, odd.loops[0].step()),
            owned(&[
                ("accumulator of loop #0", "s", half),
                ("index of loop #0", "i", CoreType::Int),
                ("parameter 0", "x", bytes),
                ("literal 2", "2", CoreType::Int),
                ("index of loop #0", "i", CoreType::Int),
                ("infix *", "2 * i", CoreType::Int),
                ("literal 1", "1", CoreType::Int),
                ("infix +", "2 * i + 1", CoreType::Int),
                ("select", "x[2 * i + 1]", CoreType::Word8),
                ("update", "s with [i] = x[2 * i + 1]", half),
            ])
        );

        // Loops are numbered in source order of their `for` keywords; a step
        // sees the bindings in scope and every enclosing loop.
        let grid = &core.functions[2];
        assert_eq!(
            loop_headers(grid),
            [
                (
                    String::from("i"),
                    String::from("a"),
                    CoreType::Int,
                    String::from("0..2"),
                    1,
                    vec![0]
                ),
                (
                    String::from("j"),
                    String::from("b"),
                    CoreType::Int,
                    String::from("0..3"),
                    1,
                    vec![0, 1]
                ),
            ]
        );
        assert_eq!(
            core_nodes(&fixture, grid)
                .into_iter()
                .map(|(operation, _, _)| operation)
                .collect::<Vec<_>>(),
            ["local 0", "loop #0"]
        );
        assert_eq!(
            expression_nodes(&fixture, grid.loops[0].step())
                .into_iter()
                .map(|(operation, _, _)| operation)
                .collect::<Vec<_>>(),
            ["accumulator of loop #0", "loop #1"]
        );
        assert_eq!(
            expression_nodes(&fixture, grid.loops[1].step())
                .into_iter()
                .map(|(operation, _, _)| operation)
                .collect::<Vec<_>>(),
            [
                "accumulator of loop #1",
                "index of loop #0",
                "index of loop #1",
                "infix *",
                "infix +"
            ]
        );

        // A loop in an initial value is not inside the outer loop's scope.
        let first = &core.functions[3];
        assert_eq!(
            loop_headers(first)
                .into_iter()
                .map(|(index, _, _, range, _, scope)| (index, range, scope))
                .collect::<Vec<_>>(),
            [
                (String::from("i"), String::from("0..2"), vec![0]),
                (String::from("j"), String::from("5..7"), vec![1]),
            ]
        );
        assert_eq!(
            core_nodes(&fixture, first)
                .into_iter()
                .map(|(operation, _, _)| operation)
                .collect::<Vec<_>>(),
            ["literal 0", "loop #1", "loop #0"]
        );
    }

    #[test]
    fn index_ranges_follow_interval_arithmetic_over_loop_ranges() {
        let accepted_indices = [
            "x[i - 1]",
            "x[4 - i]",
            "x[-i + 4]",
            "x[i * -1 + 4]",
            "x[(i - 1) * 1]",
            "x[0x3]",
            // Bounds are exact however wide the literals.
            "x[9223372036854775808 - 9223372036854775808]",
            "x[i - 1 + 0x1_0000_0000_0000_0000_0000_0000_0000_0000 \
               - 0x1_0000_0000_0000_0000_0000_0000_0000_0000]",
            "x[(i - 1) * 0x1_0000_0000_0000_0000 * 0x1_0000_0000_0000_0000 * 0 + i - 1]",
        ];
        for index in accepted_indices {
            accepted(&format!(
                "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
            ));
        }
        // Each bound is computed separately, so `i - i` runs from -3 through
        // 3 over 1..5 though its value is always 0, and `(i - 3) * (i - 3)`
        // runs from -2 through 4 over 1..5.
        for (index, message) in [
            (
                "x[i]",
                "this index runs from 1 through 4, out of range for `Word[8]^4`",
            ),
            (
                "x[i - 2]",
                "this index runs from -1 through 2, out of range for `Word[8]^4`",
            ),
            (
                "x[i - i]",
                "this index runs from -3 through 3, out of range for `Word[8]^4`",
            ),
            (
                "x[i + 3 - i]",
                "this index runs from 0 through 6, out of range for `Word[8]^4`",
            ),
            (
                "x[(i - 3) * (i - 3)]",
                "this index runs from -2 through 4, out of range for `Word[8]^4`",
            ),
            ("x[-1 + 5]", "index 4 is out of range for `Word[8]^4`"),
            (
                "x[i + 0x1_0000_0000_0000_0000]",
                "this index runs from 18446744073709551617 through 18446744073709551620, \
                 out of range for `Word[8]^4`",
            ),
        ] {
            let (fixture, result) = rejected(&format!(
                "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
            ));
            assert_eq!(
                result.diagnostics.len(),
                1,
                "{index}: {:?}",
                result.diagnostics
            );
            let diagnostic = &result.diagnostics[0];
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::IndexOutOfRange,
                "{index}"
            );
            assert_eq!(diagnostic.message(), message, "{index}");
            assert_eq!(diagnostic.label(), "indices run from 0 through 3");
            assert_eq!(
                Some(format!(
                    "x[{}]",
                    fixture.source().slice(diagnostic.primary_span()).unwrap()
                )),
                Some(index.to_owned())
            );
        }
        let (_, result) = rejected(
            "  spec f(x: Word[8]^4) -> Word[8] { for i in 0..4 with s: Word[8] = 0 { s ^ x[i - 5] } }\n",
        );
        assert_eq!(
            result.diagnostics[0].message(),
            "this index runs from -5 through -2, out of range for `Word[8]^4`"
        );
        // A bound may have as many significant bits as an `Int` value, and no
        // more: 2^16382 + 2^16382 has 16,384 bits, 2^16383 + 2^16383 one more.
        let wide = |top: char| format!("0x{top}{}", "0".repeat(4095));
        let (half, full) = (wide('4'), wide('8'));
        accepted(&format!(
            "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 \
             {{ s ^ x[{half} + {half} - {half} - {half} + i - 1] }} }}\n"
        ));
        let (_, result) = rejected(&format!(
            "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 \
             {{ s ^ x[{full} + {full} - {full} - {full} + i - 1] }} }}\n"
        ));
        assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::IndexOutOfRange
        );
        assert_eq!(
            result.diagnostics[0].message(),
            "a bound of this index's range exceeds the 16384-significant-bit limit of `Int`"
        );
        assert_eq!(
            result.diagnostics[0].label(),
            "indices run from 0 through 3"
        );
    }

    #[test]
    fn loop_update_and_fill_errors_are_reported_once_in_checking_order() {
        let (fixture, result) = rejected(concat!(
            "  spec variable(x: Word[8]^4, i: Int) -> Word[8] { x[i + 1] }\n",
            "  spec call(x: Word[8]^4) -> Word[8] { x[width(x)] }\n",
            "  spec width(x: Word[8]^4) -> Int { 4 }\n",
            "  spec empty() -> Int { for i in 3..3 with s: Int = 0 { s } }\n",
            "  spec long() -> Int { for i in 0..65537 with s: Int = 0 { s } }\n",
            "  spec longest() -> Int { for i in 65535..65536 with s: Int = 0 { s + i } }\n",
            "  spec same(x: Int) -> Int { for x in 0..2 with s: Int = 0 { s } }\n",
            "  spec twin() -> Int { for i in 0..2 with i: Int = 0 { i } }\n",
            "  spec inner() -> Int { for i in 0..2 with s: Int = 0 { for j in 0..2 with s: Int = 0 { s } } }\n",
            "  spec typed() -> Word[8] { for i in 0..2 with s: Int = 0 { s } }\n",
            "  spec scalar(x: Word[8]) -> Word[8] { x with [0] = 1 }\n",
            "  spec gives(x: Word[8]^2) -> Word[8] { x with [0] = 1 }\n",
            "  spec value(x: Word[8]^2) -> Word[8]^2 { x with [0] = 256 }\n",
            "  spec short() -> Word[8]^4 { [0; 3] }\n",
            "  spec none() -> Word[8]^4 { [0; 0] }\n",
            "  spec fill() -> Int { [0; 2] }\n",
            "  spec outside() -> Int { let a: Int = for i in 0..2 with s: Int = 0 { s + i }; s }\n",
            "  spec whole(x: Word[8]^2) -> Word[8]^2 { for i in 0..2 with s: Word[8]^2 = x { s + x } }\n",
            "  spec store(x: Word[8]^2) -> Word[8]^2 { for i in 0..3 with s: Word[8]^2 = x { s with [i] = 0 } }\n",
            "  spec step(x: Word[8]^2) -> Word[8]^2 { for i in 0..2 with s: Word[8]^2 = x { i } }\n",
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::NonStaticIndex,
                    "i",
                    String::from(
                        "an `Int` index may use only integer literals, loop indices, and words \
                         converted with `as Int`"
                    )
                ),
                (
                    DiagnosticCode::NonStaticIndex,
                    "width(x)",
                    String::from(
                        "an `Int` index may use only integer literals, loop indices, and words \
                         converted with `as Int`"
                    )
                ),
                (
                    DiagnosticCode::InvalidLoopRange,
                    "3",
                    String::from("the loop range 3..3 is empty")
                ),
                (
                    DiagnosticCode::InvalidLoopRange,
                    "65537",
                    String::from("a loop bound must be at most 65536")
                ),
                (
                    DiagnosticCode::DuplicateBinding,
                    "x",
                    String::from("duplicate name `x`")
                ),
                (
                    DiagnosticCode::DuplicateBinding,
                    "i",
                    String::from("duplicate name `i`")
                ),
                (
                    DiagnosticCode::DuplicateBinding,
                    "s",
                    String::from("duplicate name `s`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "for i in 0..2 with s: Int = 0 { s }",
                    String::from("this loop has type `Int`, but `Word[8]` is required here")
                ),
                (
                    DiagnosticCode::NotAnArray,
                    "x",
                    String::from("only an array can be updated, but this has type `Word[8]`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "x with [0] = 1",
                    String::from("an update gives an array, but `Word[8]` is required here")
                ),
                (
                    DiagnosticCode::WordLiteralOutOfRange,
                    "256",
                    String::from("literal is outside the range of `Word[8]`")
                ),
                (
                    DiagnosticCode::ArrayLengthMismatch,
                    "[0; 3]",
                    String::from("this array has 3 elements, but `Word[8]^4` has 4")
                ),
                (
                    DiagnosticCode::UnsupportedArrayLength,
                    "0",
                    String::from("an array length must be a decimal integer from 1 through 256")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "[0; 2]",
                    String::from("an array literal cannot have type `Int`")
                ),
                (
                    DiagnosticCode::UnknownParameter,
                    "s",
                    String::from("`s` is not a parameter or binding of `outside`")
                ),
                (
                    DiagnosticCode::UnsupportedOperator,
                    "+",
                    String::from("`+` is not defined for `Word[8]^2`")
                ),
                (
                    DiagnosticCode::IndexOutOfRange,
                    "i",
                    String::from("this index runs from 0 through 2, out of range for `Word[8]^2`")
                ),
                (
                    DiagnosticCode::TypeMismatch,
                    "i",
                    String::from("`i` has type `Int`, but `Word[8]^2` is required here")
                ),
            ]
        );
        let secondary = |index: usize| {
            result.diagnostics[index]
                .secondary_spans()
                .iter()
                .map(|secondary| {
                    (
                        fixture.source().slice(secondary.span()).unwrap(),
                        secondary.label().to_owned(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(secondary(4), [("x", String::from("the parameter is here"))]);
        assert_eq!(
            secondary(5),
            [("i", String::from("the loop index is here"))]
        );
        assert_eq!(
            secondary(6),
            [("s", String::from("the accumulator is here"))]
        );
        assert_eq!(result.diagnostics[0].notes(), [STATIC_INDEX_NOTE]);
    }

    #[test]
    fn loop_events_and_core_nodes_follow_the_normative_accounting() {
        // Lookup and installation (2); the parameter's uniqueness check,
        // name, width, and length (4); the result's name and width (2); the
        // loop (1); the bound `0` (prefix, no significant digit: 1) and the
        // bound `2` (prefix and one digit: 2); the two loop-name checks (2);
        // the accumulator type's name and width (2); the initial literal
        // `0` (literal and prefix: 2); and the step's `^`, `s`, the index
        // `x[i]`, its base `x`, and its index `i` (5): 23 analysis events.
        // Core is the module, one function node, one result-type node, one
        // parameter-type node, the body nodes `0` and the loop, one loop
        // node, one accumulator-type node, and the step nodes `s`, `x`, `i`,
        // the selection, and `^`: 13 nodes, each one more event.
        let fixture = module(
            "  spec f(x: Word[8]^2) -> Word[8] { for i in 0..2 with s: Word[8] = 0 { s ^ x[i] } }\n",
        );
        let (events, nodes) = (36, 13);
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert!(exact.core.is_some());
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label);
        }
        // An update is one event, and a fill two: the literal and its length.
        let fixture = module("  spec f(x: Word[8]^2) -> Word[8]^2 { [0; 2] with [1] = x[0] }\n");
        // Lookup and installation (2); the parameter (4); the result (3); the
        // update (1); the fill and its length (2); the element `0` (2); the
        // index `1` (literal, prefix, and one digit: 3); the value's index,
        // its prefix and digit-free `0`, and its base (3): 20 events, and
        // the module, function, result, parameter, `0`, fill, `1`, `x`,
        // `x[0]`, and the update: 10 nodes.
        let exact = fixture.analyze_with(Limits {
            events: 30,
            nodes: 10,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        let short = fixture.analyze_with(Limits {
            events: 29,
            nodes: 10,
            ..Limits::DEFAULT
        });
        assert_eq!(
            short.diagnostics[0].label(),
            "semantic event budget exhausted"
        );
        // A loop bound decodes against the significant-bit limit before its
        // range is checked.
        let fixture = module(&format!(
            "  spec f() -> Int {{ for i in 0..0x{} with s: Int = 0 {{ s }} }}\n",
            "f".repeat(4097)
        ));
        let result = fixture.analyze();
        assert!(result.core.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::IntegerMagnitudeLimit
        );
    }

    #[test]
    fn loop_step_storage_failures_return_no_partial_core() {
        let fixture =
            module("  spec f(x: Int) -> Int { for i in 0..2 with s: Int = x { s + i } }\n");
        let first = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_core_node_slot =
                |nodes| nodes.len() < 2 && nodes.try_reserve(1).is_ok();
            analyzer.run()
        };
        let result = first();
        assert_eq!(result, first());
        assert!(result.core.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some("s + i")
        );
    }

    #[test]
    fn rejects_foreign_spans_in_loops_updates_and_fills() {
        let text = "edition 2026; module values { \
                    spec value(x: Word[8]^2) -> Word[8]^2 { \
                    for i in 0..2 with s: Word[8]^2 = [0; 2] { s with [i] = x[i] } } }\n";
        let first = Fixture::new(text);
        let second = Fixture::new(text);
        fn loop_of(ast: &SyntaxTree) -> &LoopExpression {
            let FunctionBody::Typed(body) = &ast.module.functions[0].body else {
                unreachable!();
            };
            let ExpressionKind::Loop(r#loop) = &body.expression.kind else {
                unreachable!();
            };
            r#loop
        }
        fn loop_mut(ast: &mut SyntaxTree) -> &mut LoopExpression {
            let ExpressionKind::Loop(r#loop) = &mut typed_body_mut(ast).expression.kind else {
                unreachable!();
            };
            r#loop
        }
        fn update_mut(ast: &mut SyntaxTree) -> &mut UpdateExpression {
            match &mut loop_mut(ast).step.kind {
                ExpressionKind::Update(update) => update,
                _ => unreachable!(),
            }
        }
        fn fill_mut(ast: &mut SyntaxTree) -> &mut FillExpression {
            match &mut loop_mut(ast).init.kind {
                ExpressionKind::Fill(fill) => fill,
                _ => unreachable!(),
            }
        }
        let foreign = loop_of(&second.ast).clone();
        let ExpressionKind::Update(foreign_update) = &foreign.step.kind else {
            unreachable!();
        };
        let ExpressionKind::Fill(foreign_fill) = &foreign.init.kind else {
            unreachable!();
        };
        let ExpressionKind::Index(foreign_index) = &foreign_update.value.kind else {
            unreachable!();
        };
        type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
        let mutations: Vec<Mutation<'_>> = vec![
            Box::new(|ast| loop_mut(ast).keyword_span = foreign.keyword_span),
            Box::new(|ast| loop_mut(ast).index.span = foreign.index.span),
            Box::new(|ast| loop_mut(ast).start_span = foreign.start_span),
            Box::new(|ast| loop_mut(ast).end_span = foreign.end_span),
            Box::new(|ast| loop_mut(ast).accumulator.span = foreign.accumulator.span),
            Box::new(|ast| loop_mut(ast).ty.length_span = foreign.ty.length_span),
            Box::new(|ast| update_mut(ast).keyword_span = foreign_update.keyword_span),
            Box::new(|ast| update_mut(ast).index.span = foreign_update.index.span),
            Box::new(|ast| fill_mut(ast).length_span = foreign_fill.length_span),
            Box::new(|ast| fill_mut(ast).element.span = foreign_fill.element.span),
            Box::new(|ast| {
                let ExpressionKind::Index(index) = &mut update_mut(ast).value.kind else {
                    unreachable!();
                };
                index.index.span = foreign_index.index.span;
            }),
        ];
        assert!(analyze(first.source(), &first.ast).core.is_some());
        for (case_index, mutate) in mutations.iter().enumerate() {
            let mut ast = first.ast.clone();
            mutate(&mut ast);
            let result = analyze(first.source(), &ast);
            assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
            assert!(result.core.is_none(), "case {case_index}");
            assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
            assert_eq!(
                result.diagnostics[0].code(),
                DiagnosticCode::InvalidSemanticInput,
                "case {case_index}"
            );
        }
    }

    /// Renders a function's conditionals as `(source, type, visible
    /// bindings, scope)`.
    fn conditional_headers<'text>(
        fixture: &'text Fixture,
        function: &CoreFunction,
    ) -> Vec<(&'text str, CoreType, u32, Vec<u32>)> {
        function
            .conditionals
            .iter()
            .map(|conditional| {
                (
                    fixture.source().slice(conditional.span()).unwrap(),
                    conditional.ty(),
                    conditional.visible_locals(),
                    conditional.scope().to_vec(),
                )
            })
            .collect()
    }

    #[test]
    fn conditions_comparisons_and_divisions_build_typed_core_in_postorder() {
        let (fixture, core) = accepted(concat!(
            "  spec pick(c: Bool, x: Int) -> Int { if c { x } else if x < 0 { 0 - x } else { 7 } }\n",
            "  spec differ(a: Word[8], b: Word[8]) -> Bool { !(a == b) && true }\n",
            "  spec split(x: Int) -> Int^2 { [x / 3, x % 3] }\n",
            "  spec evens() -> Int {\n",
            "    let two: Int = 2;\n",
            "    for i in 0..4 with s: Int = 0 { if (i % two) == 0 { s + i } else { s } }\n",
            "  }\n",
        ));
        let owned = |rows: &[(&str, &'static str, CoreType)]| {
            rows.iter()
                .map(|(operation, source, ty)| ((*operation).to_owned(), *source, *ty))
                .collect::<Vec<_>>()
        };
        let whole = "if c { x } else if x < 0 { 0 - x } else { 7 }";
        let later = "if x < 0 { 0 - x } else { 7 }";

        let pick = &core.functions[0];
        assert_eq!(
            core_nodes(&fixture, pick),
            owned(&[
                ("parameter 0", "c", CoreType::Bool),
                ("choose #0", whole, CoreType::Int),
            ])
        );
        assert_eq!(
            conditional_headers(&fixture, pick),
            [
                (whole, CoreType::Int, 0, vec![]),
                (later, CoreType::Int, 0, vec![])
            ]
        );
        assert_eq!(
            expression_nodes(&fixture, pick.conditionals[0].then_branch()),
            owned(&[("parameter 1", "x", CoreType::Int)])
        );
        // The later arm's condition and choice are the first arm's `else`.
        assert_eq!(
            expression_nodes(&fixture, pick.conditionals[0].else_branch()),
            owned(&[
                ("parameter 1", "x", CoreType::Int),
                ("literal 0", "0", CoreType::Int),
                ("compare < on Int", "x < 0", CoreType::Bool),
                ("choose #1", later, CoreType::Int),
            ])
        );
        assert_eq!(
            expression_nodes(&fixture, pick.conditionals[1].then_branch()),
            owned(&[
                ("literal 0", "0", CoreType::Int),
                ("parameter 1", "x", CoreType::Int),
                ("infix -", "0 - x", CoreType::Int),
            ])
        );
        assert_eq!(
            expression_nodes(&fixture, pick.conditionals[1].else_branch()),
            owned(&[("literal 7", "7", CoreType::Int)])
        );

        assert_eq!(
            core_nodes(&fixture, &core.functions[1]),
            owned(&[
                ("parameter 0", "a", CoreType::Word8),
                ("parameter 1", "b", CoreType::Word8),
                ("compare == on Word[8]", "a == b", CoreType::Bool),
                ("prefix !", "!(a == b)", CoreType::Bool),
                ("literal true", "true", CoreType::Bool),
                ("infix &&", "!(a == b) && true", CoreType::Bool),
            ])
        );
        assert_eq!(
            core_nodes(&fixture, &core.functions[2]),
            owned(&[
                ("parameter 0", "x", CoreType::Int),
                ("literal 3", "3", CoreType::Int),
                ("infix /", "x / 3", CoreType::Int),
                ("parameter 0", "x", CoreType::Int),
                ("literal 3", "3", CoreType::Int),
                ("infix %", "x % 3", CoreType::Int),
                ("array of 2", "[x / 3, x % 3]", array_of(CoreType::Int, 2)),
            ])
        );

        // A conditional in a loop's step records the loop and the bindings
        // in scope.
        let evens = &core.functions[3];
        assert_eq!(
            conditional_headers(&fixture, evens),
            [(
                "if (i % two) == 0 { s + i } else { s }",
                CoreType::Int,
                1,
                vec![0]
            )]
        );
        assert_eq!(
            expression_nodes(&fixture, evens.loops[0].step()),
            owned(&[
                ("index of loop #0", "i", CoreType::Int),
                ("local 0", "two", CoreType::Int),
                ("infix %", "i % two", CoreType::Int),
                ("literal 0", "0", CoreType::Int),
                ("compare == on Int", "(i % two) == 0", CoreType::Bool),
                (
                    "choose #0",
                    "if (i % two) == 0 { s + i } else { s }",
                    CoreType::Int
                ),
            ])
        );
    }

    #[test]
    fn conditionals_are_numbered_in_source_order_of_their_if_keywords() {
        let (fixture, core) = accepted(concat!(
            "  spec f(a: Bool, b: Bool) -> Int {\n",
            "    if (if a { b } else { false }) { if b { 1 } else { 2 } } else if a { 3 } else { 4 }\n",
            "  }\n",
        ));
        assert_eq!(
            conditional_headers(&fixture, &core.functions[0])
                .into_iter()
                .map(|(source, ty, _, _)| (source, ty))
                .collect::<Vec<_>>(),
            [
                (
                    "if (if a { b } else { false }) { if b { 1 } else { 2 } } else if a { 3 } else { 4 }",
                    CoreType::Int
                ),
                ("if a { b } else { false }", CoreType::Bool),
                ("if b { 1 } else { 2 }", CoreType::Int),
                ("if a { 3 } else { 4 }", CoreType::Int),
            ]
        );
        // A comparison's operands and a conversion's operand may be
        // conditionals; their type comes from their first typed value.
        accepted(concat!(
            "  spec g(c: Bool, x: Word[8]) -> Bool { (if c { 1 } else { x }) < 3 }\n",
            "  spec h(c: Bool, x: Word[8]) -> Int { (if c { 1 } else { x }) as Int }\n",
            "  spec k(c: Bool, x: Int) -> Bool { 3 == (if c { x } else { 1 }) }\n",
        ));
    }

    #[test]
    fn bool_literals_resolve_only_where_no_name_of_their_spelling_is_in_scope() {
        let (fixture, core) = accepted(concat!(
            "  spec parameter(true: Int) -> Int { true }\n",
            "  spec binding() -> Int { let false: Int = 5; false }\n",
            "  spec index() -> Int { for true in 0..3 with s: Int = 0 { s + true } }\n",
            "  spec before() -> Bool { let x: Bool = true; let true: Bool = false; x && true }\n",
            "  spec named(if: Int) -> Int { if + 1 }\n",
        ));
        let operations = |index: usize| {
            core_nodes(&fixture, &core.functions[index])
                .into_iter()
                .map(|(operation, _, _)| operation)
                .collect::<Vec<_>>()
        };
        assert_eq!(operations(0), ["parameter 0"]);
        assert_eq!(operations(1), ["local 0"]);
        assert_eq!(
            expression_nodes(&fixture, core.functions[2].loops[0].step())
                .into_iter()
                .map(|(operation, _, _)| operation)
                .collect::<Vec<_>>(),
            ["accumulator of loop #0", "index of loop #0", "infix +"]
        );
        // `true` in the first binding is the literal: the binding named
        // `true` starts after it.
        assert_eq!(
            expression_nodes(&fixture, &core.functions[3].locals[0].value)
                .into_iter()
                .map(|(operation, _, _)| operation)
                .collect::<Vec<_>>(),
            ["literal true"]
        );
        assert_eq!(operations(3), ["local 0", "local 1", "infix &&"]);
        assert_eq!(operations(4), ["parameter 0", "literal 1", "infix +"]);
    }

    #[test]
    fn condition_and_comparison_errors_are_reported_once_in_checking_order() {
        let (fixture, result) = rejected(concat!(
            "  spec gives(x: Int) -> Int { x < 1 }\n",
            "  spec untyped() -> Bool { 1 < 2 }\n",
            "  spec order(a: Bool, b: Bool) -> Bool { a < b }\n",
            "  spec arrays(x: Word[8]^2, y: Word[8]^2) -> Bool { x == y }\n",
            "  spec mixed(x: Word[8], y: Int) -> Bool { x == y }\n",
            "  spec literal() -> Bool { 1 }\n",
            "  spec both() -> Bool { true && 1 }\n",
            "  spec not(x: Word[8]) -> Word[8] { !x }\n",
            "  spec negate(b: Bool) -> Bool { -b }\n",
            "  spec sum(a: Bool, b: Bool) -> Bool { a + b }\n",
            "  spec bits(a: Bool, b: Bool) -> Bool { a & b }\n",
            "  spec logic(x: Word[8], y: Word[8]) -> Word[8] { x && y }\n",
            "  spec cond(x: Int) -> Int { if x { 1 } else { 0 } }\n",
            "  spec branches(c: Bool) -> Int { if c { 1 } else { true } }\n",
            "  spec to(c: Bool) -> Int { c as Int }\n",
            "  spec from(x: Int) -> Bool { x as Bool }\n",
            "  spec divide(a: Bool) -> Bool { a / a }\n",
            "  spec later() -> Bool { let x: Bool = y; let y: Bool = true; x }\n",
            "  spec unknown() -> Bool { z == 1 }\n",
            "  spec every(c: Bool, x: Int) -> Int { if x { y } else if c { 1 < 2 } else { c } }\n",
        ));
        let rows = [
            (
                DiagnosticCode::TypeMismatch,
                "x < 1",
                "a comparison gives `Bool`, but `Int` is required here",
            ),
            (
                DiagnosticCode::UntypedComparison,
                "1 < 2",
                "the operands of `<` have no type of their own",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "<",
                "`<` is not defined for `Bool`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "==",
                "`==` is not defined for `Word[8]^2`",
            ),
            (
                DiagnosticCode::TypeMismatch,
                "y",
                "`y` has type `Int`, but `Word[8]` is required here",
            ),
            (
                DiagnosticCode::TypeMismatch,
                "1",
                "an integer literal cannot have type `Bool`",
            ),
            (
                DiagnosticCode::TypeMismatch,
                "1",
                "an integer literal cannot have type `Bool`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "!",
                "prefix `!` is not defined for `Word[8]`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "-",
                "prefix `-` is not defined for `Bool`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "+",
                "`+` is not defined for `Bool`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "&",
                "`&` is not defined for `Bool`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "&&",
                "`&&` is not defined for `Word[8]`",
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x",
                "`x` has type `Int`, but `Bool` is required here",
            ),
            (
                DiagnosticCode::TypeMismatch,
                "true",
                "`true` has type `Bool`, but `Int` is required here",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "as",
                "`as` does not convert to or from `Bool`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "as",
                "`as` does not convert to or from `Bool`",
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "/",
                "`/` is not defined for `Bool`",
            ),
            (
                DiagnosticCode::UnknownParameter,
                "y",
                "`y` is used before it is bound",
            ),
            (
                DiagnosticCode::UnknownParameter,
                "z",
                "`z` is not a parameter of `unknown`",
            ),
            // Every part of a conditional is checked, in source order.
            (
                DiagnosticCode::TypeMismatch,
                "x",
                "`x` has type `Int`, but `Bool` is required here",
            ),
            (
                DiagnosticCode::UnknownParameter,
                "y",
                "`y` is not a parameter of `every`",
            ),
            (
                DiagnosticCode::TypeMismatch,
                "1 < 2",
                "a comparison gives `Bool`, but `Int` is required here",
            ),
            (
                DiagnosticCode::UntypedComparison,
                "1 < 2",
                "the operands of `<` have no type of their own",
            ),
            (
                DiagnosticCode::TypeMismatch,
                "c",
                "`c` has type `Bool`, but `Int` is required here",
            ),
        ];
        assert_eq!(
            reported(&fixture, &result),
            rows.iter()
                .map(|(code, source, message)| (*code, *source, (*message).to_owned()))
                .collect::<Vec<_>>()
        );
        let notes = |index: usize| result.diagnostics[index].notes().to_vec();
        assert_eq!(
            notes(2),
            ["`Bool` values are compared with `==` and `!=`; they have no order"]
        );
        assert_eq!(notes(3), ["compare elements, such as `x[0] == y[0]`"]);
        assert_eq!(
            notes(5),
            ["the `Bool` values are written `true` and `false`"]
        );
        assert_eq!(
            notes(7),
            ["`!` negates a `Bool`; `~` is the bitwise complement of a word"]
        );
        assert_eq!(notes(9), [BOOL_OPERATOR_NOTE]);
        assert_eq!(
            notes(11),
            [
                "`&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators on words"
            ]
        );
    }

    #[test]
    fn index_ranges_follow_euclidean_division_and_its_total_rules() {
        for (range, index) in [
            ("0..8", "x[i / 2]"),
            ("0..100", "x[i % 4]"),
            ("0..100", "x[(i + 3) % 4]"),
            ("0..100", "x[-i % 4]"),
            ("0..100", "x[(0 - i) % -4]"),
            ("1..4", "x[7 / (i + 1)]"),
            ("0..4", "x[i % 0]"),
            ("0..4", "x[i / 0]"),
            ("0..3", "x[(i - 3) / -1]"),
            ("4..8", "x[i % 4]"),
            // Bounds are exact, so division is exact far beyond 64 bits.
            (
                "0..4",
                "x[(0x1_0000_0000_0000_0000 / 0x1_0000_0000_0000_0000) * i]",
            ),
            (
                "0..4",
                "x[(i + 0x1_0000_0000_0000_0000) % 0x1_0000_0000_0000_0000]",
            ),
            ("0..4", "x[i / (0 - 0x1_0000_0000_0000_0000)]"),
        ] {
            accepted(&format!(
                "  spec f(x: Word[8]^4) -> Word[8] {{ for i in {range} with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
            ));
        }
        for (range, index, message) in [
            (
                "0..9",
                "x[i / 2]",
                "this index runs from 0 through 4, out of range for `Word[8]^4`",
            ),
            (
                "0..100",
                "x[i % 5]",
                "this index runs from 0 through 4, out of range for `Word[8]^4`",
            ),
            (
                "0..4",
                "x[i / (i - i)]",
                "this index runs from -3 through 3, out of range for `Word[8]^4`",
            ),
            (
                "0..8",
                "x[i % 0]",
                "this index runs from 0 through 7, out of range for `Word[8]^4`",
            ),
            (
                "0..4",
                "x[i / -1]",
                "this index runs from -3 through 0, out of range for `Word[8]^4`",
            ),
            (
                "0..4",
                "x[9 % 5]",
                "index 4 is out of range for `Word[8]^4`",
            ),
            (
                "0..4",
                "x[-1 / 2]",
                "index -1 is out of range for `Word[8]^4`",
            ),
            (
                "0..4",
                "x[(0x1_0000_0000_0000_0000 * 3) / 2]",
                "index 27670116110564327424 is out of range for `Word[8]^4`",
            ),
            (
                "0..4",
                "x[(i + 0x1_0000_0000_0000_0000) % 0x1_0000_0000_0000_0001]",
                "this index runs from 0 through 18446744073709551616, out of range for `Word[8]^4`",
            ),
            (
                "0..4",
                "x[(i + 0x1_0000_0000_0000_0000) % 0]",
                "this index runs from 18446744073709551616 through 18446744073709551619, out of range for `Word[8]^4`",
            ),
        ] {
            let (fixture, result) = rejected(&format!(
                "  spec f(x: Word[8]^4) -> Word[8] {{ for i in {range} with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
            ));
            assert_eq!(
                result.diagnostics.len(),
                1,
                "{index}: {:?}",
                result.diagnostics
            );
            let diagnostic = &result.diagnostics[0];
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::IndexOutOfRange,
                "{index}"
            );
            assert_eq!(diagnostic.message(), message, "{index}");
            assert_eq!(
                Some(format!(
                    "x[{}]",
                    fixture.source().slice(diagnostic.primary_span()).unwrap()
                )),
                Some(index.to_owned())
            );
        }
        // Division by a parameter is not static.
        let (fixture, result) = rejected(
            "  spec f(x: Word[8]^4, n: Int) -> Word[8] { for i in 0..4 with s: Word[8] = 0 { s ^ x[i / n] } }\n",
        );
        assert_eq!(
            reported(&fixture, &result),
            [(
                DiagnosticCode::NonStaticIndex,
                "n",
                String::from(
                    "an `Int` index may use only integer literals, loop indices, and words \
                         converted with `as Int`"
                )
            )]
        );
        assert_eq!(result.diagnostics[0].notes(), [STATIC_INDEX_NOTE]);
    }

    #[test]
    fn word_indices_range_over_their_types_and_narrow_through_operators() {
        let spec = |length: u32, index: &str| {
            format!(
                "  spec f(t: Word[8]^{length}, x: Word[8], y: Word[16], z: Word[64]) -> Word[8] {{ t[{index}] }}\n"
            )
        };
        for (length, index) in [
            // A word index ranges over its type.
            (256, "x"),
            (256, "z >> 56"),
            (256, "y >> 8"),
            (256, "y as Word[8]"),
            (256, "x + x"),
            (256, "x - 1"),
            // `&`, `|`, `^`, `~`, `/`, `%`, and literal shifts narrow it.
            (64, "(x >> 2) & 63"),
            (64, "x >> 2"),
            (16, "x & 15"),
            (16, "15 & x"),
            (16, "x >> 4"),
            (16, "x % 16"),
            (16, "y % 16"),
            (16, "(x >> 4) ^ (x & 15)"),
            (16, "(x & 7) | 8"),
            (16, "x / 16"),
            (16, "(x & 63) / 4"),
            (32, "~(x | 224)"),
            (32, "(x & 15) << 1"),
            // `+`, `-`, and `*` narrow only where they cannot wrap.
            (32, "(x & 15) + 16"),
            (32, "(x & 15) * 2"),
            (32, "31 - (x & 15)"),
            // A wider word converted from a narrower one keeps its range.
            (16, "(x & 15) as Word[16]"),
            // Conditionals join their branches.
            (16, "if x < 3 { x & 7 } else { 15 }"),
            // An `Int` index may convert words.
            (256, "((x as Int) / 2) + 128"),
            (4, "(x as Int) % 4"),
            (4, "(x & 3) as Int"),
            (256, "(y as Int) / 256"),
            (16, "if x < 3 { 0 } else { (x as Int) / 16 }"),
        ] {
            let (_, core) = accepted(&spec(length, index));
            // A word index is proved in range and then converted to `Int`,
            // so every selection takes an `Int` position.
            let [.., root, position, select] = core.functions()[0].body().nodes() else {
                panic!("{index}");
            };
            assert!(matches!(select.kind(), CoreNodeKind::Select), "{index}");
            assert_eq!(position.ty(), CoreType::Int, "{index}");
            if !index.contains("as Int") {
                let CoreNodeKind::Convert { from } = position.kind() else {
                    panic!("{index}");
                };
                assert_eq!(root.ty(), *from, "{index}");
                assert_eq!(root.span(), position.span(), "{index}");
            }
        }
        for (length, index, message) in [
            (
                16,
                "x",
                "this index runs from 0 through 255, out of range for `Word[8]^16`",
            ),
            (
                255,
                "x",
                "this index runs from 0 through 255, out of range for `Word[8]^255`",
            ),
            (
                256,
                "z",
                "this index runs from 0 through 18446744073709551615, out of range for `Word[8]^256`",
            ),
            (
                16,
                "x & 31",
                "this index runs from 0 through 31, out of range for `Word[8]^16`",
            ),
            (
                16,
                "(x & 15) + 1",
                "this index runs from 1 through 16, out of range for `Word[8]^16`",
            ),
            (
                16,
                "(x & 15) - 1",
                "this index runs from 0 through 255, out of range for `Word[8]^16`",
            ),
            (
                16,
                "x % 17",
                "this index runs from 0 through 16, out of range for `Word[8]^16`",
            ),
            (
                16,
                "x % (x & 15)",
                "this index runs from 0 through 255, out of range for `Word[8]^16`",
            ),
            (
                16,
                "(x & 15) << 5",
                "this index runs from 0 through 255, out of range for `Word[8]^16`",
            ),
            (
                16,
                "y as Word[8]",
                "this index runs from 0 through 255, out of range for `Word[8]^16`",
            ),
            (
                16,
                "if x < 3 { x & 7 } else { 16 }",
                "this index runs from 0 through 16, out of range for `Word[8]^16`",
            ),
            (
                256,
                "(x as Int) + 1",
                "this index runs from 1 through 256, out of range for `Word[8]^256`",
            ),
            (
                256,
                "(x as Int) - 1",
                "this index runs from -1 through 254, out of range for `Word[8]^256`",
            ),
        ] {
            let (fixture, result) = rejected(&spec(length, index));
            assert_eq!(
                reported(&fixture, &result),
                [(DiagnosticCode::IndexOutOfRange, index, message.to_owned())],
                "{index}"
            );
        }
        // An `Int` parameter has no bound, and neither does a word
        // converted to `Int` and then multiplied by one.
        for (index, part) in [("n", "n"), ("(x as Int) * n", "n")] {
            let (fixture, result) = rejected(&format!(
                "  spec f(t: Word[8]^4, x: Word[8], n: Int) -> Word[8] {{ t[{index}] }}\n"
            ));
            assert_eq!(
                reported(&fixture, &result),
                [(
                    DiagnosticCode::NonStaticIndex,
                    part,
                    String::from(
                        "an `Int` index may use only integer literals, loop indices, and words \
                         converted with `as Int`"
                    )
                )],
                "{index}"
            );
            assert_eq!(result.diagnostics[0].notes(), [STATIC_INDEX_NOTE]);
        }
    }

    #[test]
    fn condition_events_and_core_nodes_follow_the_normative_accounting() {
        // Lookup and installation (2); the parameter's uniqueness check and
        // name (2); the result's name (1); the arm's `if` (1); the condition
        // `c` (1); and the values `1` and `2` (literal, prefix, and one
        // digit: 3 each): 13 analysis events. Core is the module, one
        // function node, one result-type node, one parameter-type node, the
        // body nodes `c` and the choice, one conditional node, and the
        // branch nodes `1` and `2`: 9 nodes, each one more event.
        let conditional = module("  spec f(c: Bool) -> Int { if c { 1 } else { 2 } }\n");
        // Lookup and installation (2); the parameter's uniqueness check,
        // name, and width (3); the result's name (1); the comparison (1);
        // `x` (1); and `1` (3): 11 events. Core is the module, the function,
        // the result and parameter types, and `x`, `1`, and the comparison:
        // 7 nodes.
        let comparison = module("  spec g(x: Word[8]) -> Bool { x < 1 }\n");
        for (fixture, events, nodes) in [(&conditional, 22, 9), (&comparison, 18, 7)] {
            let exact = fixture.analyze_with(Limits {
                events,
                nodes,
                ..Limits::DEFAULT
            });
            assert_eq!(exact.diagnostics, []);
            assert!(exact.core.is_some());
            for (limits, label) in [
                (
                    Limits {
                        events: events - 1,
                        nodes,
                        ..Limits::DEFAULT
                    },
                    "semantic event budget exhausted",
                ),
                (
                    Limits {
                        events,
                        nodes: nodes - 1,
                        ..Limits::DEFAULT
                    },
                    "typed Core node budget exhausted",
                ),
            ] {
                let first = fixture.analyze_with(limits);
                assert_eq!(first, fixture.analyze_with(limits));
                assert!(first.core.is_none());
                assert_eq!(first.diagnostics.len(), 1);
                assert_eq!(
                    first.diagnostics[0].code(),
                    DiagnosticCode::SemanticResourceLimit
                );
                assert_eq!(first.diagnostics[0].label(), label);
            }
        }
        // Each arm of a chain is one event and one conditional node, and
        // each later arm's condition and choice are nodes of the earlier
        // arm's `else` branch: 2 + 2 + 1 events for the signature, 3 for
        // the arms, 3 for the conditions, and 12 for the four values; 4
        // nodes for the module and signature, 2 in the body, 4 for each of
        // the first two conditionals, and 3 for the last.
        let chain = module(
            "  spec f(c: Bool) -> Int { if c { 1 } else if c { 2 } else if c { 3 } else { 4 } }\n",
        );
        let (events, nodes) = (23 + 17, 17);
        let exact = chain.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert_eq!(exact.core.unwrap().functions[0].conditionals.len(), 3);
        for limits in [
            Limits {
                events: events - 1,
                nodes,
                ..Limits::DEFAULT
            },
            Limits {
                events,
                nodes: nodes - 1,
                ..Limits::DEFAULT
            },
        ] {
            assert!(chain.analyze_with(limits).core.is_none());
        }
    }

    #[test]
    fn branch_storage_failures_return_no_partial_core() {
        let fixture = module("  spec f(c: Bool, x: Int) -> Int { if c { x + 1 } else { x } }\n");
        let first = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_core_node_slot =
                |nodes| nodes.len() < 2 && nodes.try_reserve(1).is_ok();
            analyzer.run()
        };
        let result = first();
        assert_eq!(result, first());
        assert!(result.core.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        let diagnostic = &result.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some("x + 1")
        );
    }

    #[test]
    fn rejects_foreign_spans_in_conditionals() {
        let text = "edition 2026; module values { \
                    spec value(c: Bool, x: Int) -> Int { \
                    if c { x } else if x < 1 { x / 2 } else { 0 } } }\n";
        let first = Fixture::new(text);
        let second = Fixture::new(text);
        fn conditional_of(ast: &SyntaxTree) -> &ConditionalExpression {
            let FunctionBody::Typed(body) = &ast.module.functions[0].body else {
                unreachable!();
            };
            let ExpressionKind::Conditional(conditional) = &body.expression.kind else {
                unreachable!();
            };
            conditional
        }
        fn conditional_mut(ast: &mut SyntaxTree) -> &mut ConditionalExpression {
            let ExpressionKind::Conditional(conditional) = &mut typed_body_mut(ast).expression.kind
            else {
                unreachable!();
            };
            conditional
        }
        let foreign = conditional_of(&second.ast).clone();
        type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
        let mutations: Vec<Mutation<'_>> = vec![
            Box::new(|ast| conditional_mut(ast).else_span = foreign.else_span),
            Box::new(|ast| {
                conditional_mut(ast).arms[0].keyword_span = foreign.arms[0].keyword_span;
            }),
            Box::new(|ast| {
                conditional_mut(ast).arms[1].keyword_span = foreign.arms[1].keyword_span;
            }),
            Box::new(|ast| {
                conditional_mut(ast).arms[0].condition.span = foreign.arms[0].condition.span;
            }),
            Box::new(|ast| {
                let ExpressionKind::Binary(binary) =
                    &mut conditional_mut(ast).arms[1].condition.kind
                else {
                    unreachable!();
                };
                let ExpressionKind::Binary(foreign_binary) = &foreign.arms[1].condition.kind else {
                    unreachable!();
                };
                binary.operator_span = foreign_binary.operator_span;
            }),
            Box::new(|ast| {
                conditional_mut(ast).arms[1].value.span = foreign.arms[1].value.span;
            }),
            Box::new(|ast| conditional_mut(ast).otherwise.span = foreign.otherwise.span),
        ];
        assert!(analyze(first.source(), &first.ast).core.is_some());
        for (case_index, mutate) in mutations.iter().enumerate() {
            let mut ast = first.ast.clone();
            mutate(&mut ast);
            let result = analyze(first.source(), &ast);
            assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
            assert!(result.core.is_none(), "case {case_index}");
            assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
            assert_eq!(
                result.diagnostics[0].code(),
                DiagnosticCode::InvalidSemanticInput,
                "case {case_index}"
            );
        }
    }
}
