//! Bounded name resolution, type checking, and Core construction.

use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt;

use crate::core::{
    ArrayType, CoreArray, CoreBinding, CoreConditional, CoreExpression, CoreFunction,
    CoreFunctionId, CoreLocal, CoreLoop, CoreModule, CoreNode, CoreNodeKind, CoreType, CoreValue,
    ExactInteger, MAX_ARRAY_LENGTH, MAX_EXACT_INTEGER_BITS, MAX_LOOP_BOUND, MAX_MODULUS_BITS,
    Magnitude, Modulus, Residue, TupleType,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::{
    ArrayExpression, BinaryExpression, BinaryOperator, Binding, ByteOrder, ByteString,
    CallExpression, ConditionalExpression, ConversionExpression, Expression, ExpressionKind,
    FillExpression, FunctionBody, FunctionDeclaration, FunctionKind, Identifier, IndexExpression,
    IntegerLiteral, LoopExpression, MAX_ARRAY_ELEMENTS, MAX_SIZES_PER_FUNCTION, Parameter, Pattern,
    ProjectExpression, Size, SizeParameter, SliceExpression, SliceRange, SliceUpdateExpression,
    SyntaxTree, TestDeclaration, TupleExpression, TypeSyntax, TypedBody, TypedName,
    UnaryExpression, UnaryOperator, UpdateExpression,
};
use crate::source::{SourceFile, Span, TextOffset};

mod answers;
mod bytes;
mod linking;
mod order;
mod ranges;
mod sizes;
mod tuples;
mod types;

pub use answers::MAX_TEST_TITLE_BYTES;
use linking::*;
use ranges::*;
use sizes::*;
use tuples::*;
use types::*;

/// Maximum ordinary semantic errors retained before one suppression diagnostic.
pub const MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE: usize = 100;
const MAX_RETAINED_SEMANTIC_DIAGNOSTICS: usize =
    MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE.saturating_add(2);

/// Maximum Typed Reference Core nodes constructed for one source.
pub const MAX_CORE_NODES_PER_SOURCE: usize = 262_144;

/// Maximum semantic events performed for one source.
pub const MAX_SEMANTIC_EVENTS_PER_SOURCE: usize = 1_048_576;

/// Maximum modules in one program: the root and every module it reaches
/// through `use` declarations.
pub const MAX_MODULES_PER_PROGRAM: usize = 64;

/// Maximum significant bits retained for one exact mathematical integer.
pub const MAX_INTEGER_BITS: usize = 16_384;
const _: () = assert!(MAX_INTEGER_BITS == MAX_EXACT_INTEGER_BITS);
// An array literal can spell every admitted array type and no longer one,
// and a loop can visit every element of the longest.
const _: () = assert!(MAX_ARRAY_ELEMENTS == 65_536 && MAX_ARRAY_LENGTH == 65_536);
const _: () = assert!(MAX_LOOP_BOUND == MAX_ARRAY_LENGTH);

const MAX_IDENTIFIER_BYTES_IN_DIAGNOSTIC: usize = 64;
const MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC: usize = 8;
const MAX_MODULES_IN_CYCLE_DIAGNOSTIC: usize = 8;
const ADMITTED_TYPES: &str = "`Int`, `Bool`, `Word[8]`, `Word[16]`, `Word[32]`, `Word[64]`, \
     `Mod[m]`, and the names of earlier `type` declarations";
const ARRAY_OPERATOR_NOTE: &str = "operators apply to `Int`, `Bool`, word, and residue values; \
     apply them to elements, such as `x[0]`";
const TUPLE_OPERATOR_NOTE: &str = "operators apply to `Int`, `Bool`, word, and residue values; \
     apply them to elements, such as `p.0`";

/// Returns the note of an operator applied to an array or a tuple.
fn aggregate_operator_note(ty: &CoreType) -> &'static str {
    if ty.as_tuple().is_some() {
        TUPLE_OPERATOR_NOTE
    } else {
        ARRAY_OPERATOR_NOTE
    }
}
const MODULUS_NOTE: &str = "a modulus is a constant built from integer literals with `+`, `-`, \
     `*`, `<<`, and parentheses, as in `Mod[(1 << 255) - 19]`";
const BUILT_IN_TYPE_NAMES: [&str; 4] = ["Int", "Bool", "Word", "Mod"];
const STATIC_INDEX_NOTE: &str = "every index is proved in range when the program is checked: a \
     word index ranges over its type, and an `Int` index is built from integer literals, loop \
     indices, and words converted with `as Int`, using `+`, `-`, `*`, `/`, `%`, and conditionals";
const BOOL_OPERATOR_NOTE: &str = "the operators on `Bool` are `!`, `&&`, `||`, `==`, and `!=`";
const SHIFT_AMOUNT_NOTE: &str = "an amount written as one integer literal is from 0 through n - 1; \
     any other amount is computed, an `Int` or a word, such as `x <<< r` or `x >> (i % 8)`";

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

/// Resolves and checks one successfully parsed Orange syntax tree as a
/// program of one module.
///
/// Empty functions participate in namespace checking but do not enter Core.
/// Typed `spec` functions are checked in source order against the signatures
/// of every typed `spec` in the module, and the call graph among them must be
/// acyclic. Within a body, each `let` binding is checked in source order
/// before the result expression. No other module is supplied, so a `use`
/// declaration names an unknown module; [`analyze_program`] checks a module
/// together with the modules it uses.
#[must_use]
pub fn analyze(source: &SourceFile, ast: &SyntaxTree) -> AnalysisResult {
    analyze_program((source, ast), &[])
}

/// Resolves, checks, and links a program: a root module and the modules it
/// uses, directly or through other modules.
///
/// A `use NAME;` declaration names the module of `modules` whose name is
/// `NAME`; a supplied module the root does not reach is ignored and does not
/// count toward [`MAX_MODULES_PER_PROGRAM`]. No other supplied module shares
/// the name of a module of the program, a module uses each module at most
/// once, and the uses form no cycle. Each reachable module is then checked on its
/// own, as [`analyze`] checks one module and with its own per-source limits,
/// in dependency order: every module after the modules it uses. A call
/// `NAME::f(...)` resolves to the typed `spec` function `f` of the used module
/// `NAME`.
///
/// The linked Core carries the root's name and extent. Its functions are
/// those of the used modules in dependency order, then the root's, with
/// dense identities across the program; [`CoreModule::entry_functions`]
/// returns the root's.
#[must_use]
pub fn analyze_program(
    root: (&SourceFile, &SyntaxTree),
    modules: &[(&SourceFile, &SyntaxTree)],
) -> AnalysisResult {
    let (_, root_ast) = root;
    let root_span = root_ast.module.span;
    let mut program = Vec::new();
    if program
        .try_reserve_exact(modules.len().saturating_add(1))
        .is_err()
    {
        return program_resource_limit(
            root_ast.module.span,
            "semantic program module table allocation failed",
        );
    }
    program.push(root);
    program.extend_from_slice(modules);
    for &(source, ast) in &program {
        if !syntax_tree_belongs_to_source(source, ast) {
            return invalid_semantic_input(source, |diagnostics| {
                diagnostics.try_reserve_exact(1).is_ok()
            });
        }
    }
    let graph = match ModuleGraph::build(&program, root_span) {
        Ok(graph) => graph,
        Err(diagnostics) => {
            return AnalysisResult {
                core: None,
                diagnostics,
            };
        }
    };
    link_program(&program, &graph, root_span)
}

fn program_resource_limit(span: Span, detail: &str) -> AnalysisResult {
    let mut diagnostics = Vec::new();
    if diagnostics.try_reserve_exact(1).is_ok() {
        diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::SemanticResourceLimit,
                "semantic analysis resource limit exceeded",
                span,
            )
            .with_label(detail)
            .with_note("semantic analysis stopped without producing Core"),
        );
    }
    AnalysisResult {
        core: None,
        diagnostics,
    }
}

fn syntax_tree_belongs_to_source(source: &SourceFile, ast: &SyntaxTree) -> bool {
    let source_id = source.id();
    let belongs = |span: Span| span.source() == source_id;
    let type_belongs = |ty: &TypeSyntax| type_belongs(ty, &belongs);
    let function_belongs = |function: &FunctionDeclaration| {
        belongs(function.span)
            && belongs(function.name.span)
            && function.sizes.iter().all(|size| {
                belongs(size.span)
                    && belongs(size.name.span)
                    && belongs(size.start_span)
                    && belongs(size.end_span)
                    && size.types.iter().all(&type_belongs)
            })
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
                        && bindings_belong(&body.bindings, &belongs)
                        && expression_belongs(&body.expression, &belongs)
                }
            }
    };

    belongs(ast.span)
        && belongs(ast.edition.span)
        && belongs(ast.edition.value_span)
        && belongs(ast.module.span)
        && belongs(ast.module.name.span)
        && ast
            .module
            .uses
            .iter()
            .all(|declaration| belongs(declaration.span) && belongs(declaration.name.span))
        && ast.module.types.iter().all(|declaration| {
            belongs(declaration.span)
                && belongs(declaration.name.span)
                && type_belongs(&declaration.ty)
        })
        && ast.module.functions.iter().all(&function_belongs)
        && ast.module.tests.iter().all(|test| {
            belongs(test.span) && belongs(test.title.span) && function_belongs(test.function())
        })
}

/// Parser-established expression height, which counts a modulus inside an
/// expression, bounds this recursion.
fn type_belongs(ty: &TypeSyntax, belongs: &impl Fn(Span) -> bool) -> bool {
    belongs(ty.span)
        && belongs(ty.name.span)
        && ty.width_span.is_none_or(belongs)
        && ty
            .length
            .as_ref()
            .is_none_or(|length| size_belongs(length, belongs))
        && ty
            .modulus()
            .is_none_or(|modulus| expression_belongs(modulus, belongs))
        // A tuple type's elements are never tuples: one level of recursion.
        && ty
            .elements
            .iter()
            .all(|element| type_belongs(element, belongs))
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
                    && call.module().is_none_or(|module| belongs(module.span))
                    && call
                        .sizes()
                        .iter()
                        .chain(&call.arguments)
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
                belongs(conversion.keyword_span)
                    && conversion.order.is_none_or(|(_, span)| belongs(span))
                    && type_belongs(&conversion.target, belongs)
                    && expression_belongs(&conversion.operand, belongs)
            }
            ExpressionKind::Array(array) => array
                .elements
                .iter()
                .all(|element| expression_belongs(element, belongs)),
            ExpressionKind::Fill(fill) => {
                size_belongs(&fill.length, belongs) && expression_belongs(&fill.element, belongs)
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
                belongs(r#loop.keyword_span)
                    && belongs(r#loop.index.span)
                    && size_belongs(&r#loop.start, belongs)
                    && size_belongs(&r#loop.end, belongs)
                    && pattern_belongs(&r#loop.accumulator, belongs)
                    && expression_belongs(&r#loop.init, belongs)
                    && bindings_belong(&r#loop.step_bindings, belongs)
                    && expression_belongs(&r#loop.step, belongs)
            }
            ExpressionKind::Tuple(tuple) => tuple
                .elements
                .iter()
                .all(|element| expression_belongs(element, belongs)),
            ExpressionKind::Project(project) => {
                belongs(project.position_span) && expression_belongs(&project.base, belongs)
            }
            ExpressionKind::Bytes(_) => true,
            ExpressionKind::Slice(slice) => {
                expression_belongs(&slice.base, belongs) && range_belongs(&slice.range, belongs)
            }
            ExpressionKind::SliceUpdate(update) => {
                belongs(update.keyword_span)
                    && expression_belongs(&update.base, belongs)
                    && range_belongs(&update.range, belongs)
                    && expression_belongs(&update.value, belongs)
            }
            ExpressionKind::Conditional(conditional) => {
                belongs(conditional.else_span)
                    && conditional.arms.iter().all(|arm| {
                        belongs(arm.keyword_span)
                            && expression_belongs(&arm.condition, belongs)
                            && bindings_belong(&arm.bindings, belongs)
                            && expression_belongs(&arm.value, belongs)
                    })
                    && bindings_belong(&conditional.otherwise_bindings, belongs)
                    && expression_belongs(&conditional.otherwise, belongs)
            }
        }
}

/// Returns whether every span of a size belongs.
/// Parser-established expression height bounds the recursion through it.
fn size_belongs(size: &Size, belongs: &impl Fn(Span) -> bool) -> bool {
    belongs(size.span)
        && size
            .expression()
            .is_none_or(|expression| expression_belongs(expression, belongs))
}

/// Returns whether every span of a slice's bounds belongs.
/// Parser-established expression height bounds the recursion through them.
fn range_belongs(range: &SliceRange, belongs: &impl Fn(Span) -> bool) -> bool {
    belongs(range.span)
        && belongs(range.dots_span)
        && range
            .start
            .iter()
            .chain(&range.end)
            .all(|bound| expression_belongs(bound, belongs))
}

/// Returns whether every span of a list of `let` bindings belongs.
/// Parser-established expression height bounds the recursion through
/// their values.
fn bindings_belong(bindings: &[Binding], belongs: &impl Fn(Span) -> bool) -> bool {
    bindings.iter().all(|binding| {
        belongs(binding.span)
            && pattern_belongs(&binding.pattern, belongs)
            && expression_belongs(&binding.value, belongs)
    })
}

/// Returns whether every span of a binding's or an accumulator's pattern
/// belongs.
fn pattern_belongs(pattern: &Pattern, belongs: &impl Fn(Span) -> bool) -> bool {
    belongs(pattern.span())
        && pattern.names().iter().all(|typed| {
            belongs(typed.span) && belongs(typed.name.span) && type_belongs(&typed.ty, belongs)
        })
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
    /// Identity of this module's first typed `spec` in the linked program:
    /// the number of typed `spec` functions in the modules checked before it.
    id_offset: usize,
    /// Whether this module's known-answer tests are checked and become part
    /// of its Core: only the root's are.
    tests: bool,
    reserve_pending_function_slot: fn(&mut Vec<PendingFunction>) -> bool,
    reserve_magnitude_limb: fn(&mut Vec<u32>) -> bool,
    reserve_range_limbs: fn(&mut Vec<u32>, usize) -> bool,
    reserve_core_name: fn(&mut String, usize) -> bool,
    reserve_diagnostic_slots: fn(&mut Vec<Diagnostic>, usize) -> bool,
    reserve_core_node_slot: fn(&mut Vec<CoreNode>) -> bool,
    reserve_call_edge_slot: fn(&mut Vec<CallEdge>) -> bool,
    types: TypeTable<'ast>,
}

struct PendingFunction {
    span: Span,
    name: String,
    name_span: Span,
    sizes: Vec<u32>,
    instance: String,
    parameters: Vec<CoreType>,
    result_type: CoreType,
    locals: Vec<CoreLocal>,
    nodes: Vec<CoreNode>,
    loops: Vec<CoreLoop>,
    conditionals: Vec<CoreConditional>,
    /// A known-answer test's title; `None` for a function.
    title: Option<String>,
}

/// The silently resolved signature of one typed `spec`, used to check calls
/// to it from any function in the module: one for each of its instances.
struct Signature<'ast> {
    /// Identity of the first instance; the others follow it in order.
    id: CoreFunctionId,
    /// The function's size parameters, empty for a function without sizes.
    sizes: &'ast [SizeParameter],
    /// The ranges of its size parameters, or `None` when they are malformed
    /// and the function has no instances.
    ranges: Option<SizeRanges>,
    /// The types each type parameter lists, resolved outside every
    /// instance, by the parameter's position; empty for a size.
    listed: Vec<Vec<Option<CoreType>>>,
    /// The listed types as written, by the parameter's position.
    spellings: Vec<Vec<String>>,
    /// The parameter and result types of each instance, in order.
    instances: Vec<InstanceSignature>,
}

struct InstanceSignature {
    parameters: Vec<Option<CoreType>>,
    result_type: Option<CoreType>,
}

impl InstanceSignature {
    fn is_complete(&self) -> bool {
        self.result_type.is_some() && self.parameters.iter().all(Option::is_some)
    }
}

impl Signature<'_> {
    /// Returns the identity of the instance at `index`.
    fn instance_id(&self, index: usize) -> Option<CoreFunctionId> {
        usize::try_from(self.id.index())
            .ok()?
            .checked_add(index)
            .and_then(CoreFunctionId::from_index)
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
    signatures: &'scope [Option<Signature<'ast>>],
    /// The modules this module uses, in the order of its `use` declarations.
    imports: &'scope [ImportScope<'scope, 'ast>],
}

/// The name and signature tables of one used module.
struct ImportScope<'scope, 'ast> {
    name: &'ast str,
    declarations: &'scope DeclarationIndex<'ast>,
    signatures: &'scope [Option<Signature<'ast>>],
}

impl<'scope, 'ast> ModuleScope<'scope, 'ast> {
    /// Returns the tables a call's function name resolves in: this module's
    /// for an unqualified call, or the used module's for `NAME::f(...)`.
    /// Returns `None` when `NAME` is not a module this module uses.
    fn tables_for(
        &self,
        call: &CallExpression,
    ) -> Option<(
        &'scope DeclarationIndex<'ast>,
        &'scope [Option<Signature<'ast>>],
    )> {
        match call.module() {
            None => Some((self.declarations, self.signatures)),
            Some(module) => self
                .imports
                .iter()
                .find(|import| import.name == module.text)
                .map(|import| (import.declarations, import.signatures)),
        }
    }
}

/// Core nodes of the body being checked and the module's checked calls.
struct BodyOutput<'edges> {
    nodes: Vec<CoreNode>,
    call_edges: &'edges mut Vec<CallEdge>,
}

/// The function whose body is being checked.
struct BodyContext<'ast> {
    id: CoreFunctionId,
    /// The instance being checked: the function's sizes and their values.
    instance: Instance<'ast>,
    name: &'ast Identifier,
    parameters: &'ast [Parameter],
    parameter_types: Vec<Option<CoreType>>,
    /// Every binding of the body, including those not yet in scope.
    bindings: &'ast [Binding],
    /// Types of the bindings in scope: exactly the first `binding_types.len()`.
    binding_types: Vec<Option<CoreType>>,
    /// Loops whose step is being checked, outermost first.
    loop_scopes: Vec<LoopScope<'ast>>,
    /// Loop steps and conditional branches being checked, outermost first,
    /// each with its `let` bindings.
    blocks: Vec<BlockScope<'ast>>,
    /// The `let` bindings of each step or branch already checked, in
    /// checking order, for a report of a name used outside its block.
    finished_blocks: Vec<&'ast [Binding]>,
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
    accumulator: &'ast Pattern,
    ty: CoreType,
    start: u32,
    end: u32,
}

/// A loop's step or a conditional's branch being checked, whose `let`
/// bindings come into scope one after another.
struct BlockScope<'ast> {
    owner: BlockOwner,
    /// Every binding of the block, including those not yet in scope.
    bindings: &'ast [Binding],
    /// Types of the bindings in scope: exactly the first
    /// `binding_types.len()`.
    binding_types: Vec<Option<CoreType>>,
}

/// What holds a block: the step of a loop, or a branch of a conditional,
/// by number within the function.
#[derive(Clone, Copy)]
enum BlockOwner {
    Step(u32),
    Branch(u32),
}

impl BlockOwner {
    /// Returns the Core node that reads the binding at `index` of this
    /// block.
    const fn node(self, index: u32) -> CoreNodeKind {
        match self {
            Self::Step(loop_id) => CoreNodeKind::StepBinding { loop_id, index },
            Self::Branch(conditional) => CoreNodeKind::BranchBinding { conditional, index },
        }
    }
}

/// A binding whose type resolved.
struct CheckedBinding {
    ty: CoreType,
    /// The value's Core nodes, present only when the binding is well formed.
    nodes: Option<Vec<CoreNode>>,
}

/// What a bare name refers to at one point of a body.
///
/// A name that a tuple pattern binds carries its position in the pattern:
/// it stands for that element of the pattern's value.
enum NameResolution<'ast> {
    /// A size parameter, whose value is fixed in the instance.
    Size(u32),
    Parameter(usize),
    Binding(usize, Option<u32>),
    /// The binding at `index` of the block at `block` in the block scopes.
    BlockBinding {
        block: usize,
        index: usize,
        element: Option<u32>,
    },
    /// The index of the loop scope at this position.
    LoopIndex(usize),
    /// The accumulator of the loop scope at this position.
    Accumulator(usize, Option<u32>),
    /// The `Bool` literal `true` or `false`, where no name of its spelling
    /// is in scope.
    BoolLiteral(bool),
    /// A name of a binding of this body or of a block being checked whose
    /// scope has not started, and whether it is a block's.
    LaterBinding(&'ast TypedName, bool),
    Unknown,
}

/// Returns whether `pattern` binds `name`: `Some(None)` when the pattern is
/// that one name, `Some(Some(k))` when `name` is element `k` of a tuple
/// pattern, and `None` when the pattern does not bind it.
fn pattern_element(pattern: &Pattern, name: &str) -> Option<Option<u32>> {
    match pattern {
        Pattern::Name(typed) => (typed.name.text == name).then_some(None),
        Pattern::Tuple(tuple) => tuple
            .elements
            .iter()
            .position(|typed| typed.name.text == name)
            .and_then(|position| u32::try_from(position).ok())
            .map(Some),
    }
}

/// Returns the first binding of `bindings` that binds `name`, and the
/// name's position in its tuple pattern, if it has one.
fn find_binding(bindings: &[Binding], name: &str) -> Option<(usize, Option<u32>)> {
    bindings
        .iter()
        .enumerate()
        .find_map(|(index, binding)| Some((index, pattern_element(&binding.pattern, name)?)))
}

/// Returns the typed name `name` of `pattern`, if the pattern binds it.
fn pattern_name<'pattern>(pattern: &'pattern Pattern, name: &str) -> Option<&'pattern TypedName> {
    pattern.names().iter().find(|typed| typed.name.text == name)
}

/// Returns the type of what a name stands for: the whole value's type, or,
/// for a name of a tuple pattern, the type of its element.
fn element_type(whole: Option<CoreType>, element: Option<u32>) -> Option<CoreType> {
    match element {
        None => whole,
        Some(position) => whole?.as_tuple()?.element(position).cloned(),
    }
}

impl<'ast> BodyContext<'ast> {
    /// Resolves a bare name: sizes first, then parameters, then the body's
    /// bindings in scope, then the bindings in scope of the blocks being
    /// checked, each list searched in source order.
    fn resolve(&self, name: &str) -> NameResolution<'ast> {
        if let Some((_, value)) = self.instance.find(name) {
            return NameResolution::Size(value);
        }
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
        if let Some((index, element)) = find_binding(visible, name) {
            return NameResolution::Binding(index, element);
        }
        for (block, scope) in self.blocks.iter().enumerate().rev() {
            if let Some((index, element)) = find_binding(scope.visible(), name) {
                return NameResolution::BlockBinding {
                    block,
                    index,
                    element,
                };
            }
        }
        for (position, scope) in self.loop_scopes.iter().enumerate().rev() {
            if scope.index.text == name {
                return NameResolution::LoopIndex(position);
            }
            if let Some(element) = pattern_element(scope.accumulator, name) {
                return NameResolution::Accumulator(position, element);
            }
        }
        match name {
            "true" => return NameResolution::BoolLiteral(true),
            "false" => return NameResolution::BoolLiteral(false),
            _ => {}
        }
        self.blocks
            .iter()
            .rev()
            .flat_map(BlockScope::later)
            .map(|binding| (binding, true))
            .chain(later.iter().map(|binding| (binding, false)))
            .find_map(|(binding, in_block)| {
                pattern_name(&binding.pattern, name).map(|typed| (typed, in_block))
            })
            .map_or(NameResolution::Unknown, |(typed, in_block)| {
                NameResolution::LaterBinding(typed, in_block)
            })
    }

    /// Returns the type of a name in scope without reporting.
    fn name_type(&self, name: &str) -> Option<CoreType> {
        match self.resolve(name) {
            NameResolution::Size(_) => Some(CoreType::Int),
            NameResolution::Parameter(index) => self.parameter_types.get(index).cloned().flatten(),
            NameResolution::Binding(index, element) => {
                element_type(self.binding_types.get(index).cloned().flatten(), element)
            }
            NameResolution::BlockBinding {
                block,
                index,
                element,
            } => element_type(
                self.blocks
                    .get(block)?
                    .binding_types
                    .get(index)
                    .cloned()
                    .flatten(),
                element,
            ),
            NameResolution::LoopIndex(_) => Some(CoreType::Int),
            NameResolution::Accumulator(position, element) => element_type(
                self.loop_scopes.get(position).map(|scope| scope.ty.clone()),
                element,
            ),
            NameResolution::BoolLiteral(_) => Some(CoreType::Bool),
            NameResolution::LaterBinding(..) | NameResolution::Unknown => None,
        }
    }

    /// Returns the earlier declaration of `name` among the sizes, the
    /// parameters, the bindings in scope, and the loop names in scope, if any.
    fn earlier_name(&self, name: &str) -> Option<(Span, &'static str)> {
        if let Some((size, _)) = self.instance.find(name) {
            return Some((size.name.span, "the size parameter is here"));
        }
        if let Some(parameter) = self
            .parameters
            .iter()
            .find(|parameter| parameter.name.text == name)
        {
            return Some((parameter.name.span, "the parameter is here"));
        }
        if let Some(typed) = self
            .bindings
            .get(..self.binding_types.len())
            .into_iter()
            .flatten()
            .chain(self.blocks.iter().flat_map(BlockScope::visible))
            .find_map(|binding| pattern_name(&binding.pattern, name))
        {
            return Some((typed.name.span, "the binding is here"));
        }
        self.loop_scopes.iter().find_map(|scope| {
            if scope.index.text == name {
                Some((scope.index.span, "the loop index is here"))
            } else {
                pattern_name(scope.accumulator, name)
                    .map(|typed| (typed.name.span, "the accumulator is here"))
            }
        })
    }
}

impl<'ast> BlockScope<'ast> {
    /// Returns the block's bindings in scope, in source order.
    fn visible(&self) -> &'ast [Binding] {
        self.bindings
            .get(..self.binding_types.len())
            .unwrap_or(self.bindings)
    }

    /// Returns the block's bindings whose scope has not started.
    fn later(&self) -> &'ast [Binding] {
        self.bindings
            .get(self.binding_types.len()..)
            .unwrap_or_default()
    }
}

/// Returns the first name, call, conversion, index, element of a tuple,
/// array or tuple literal, or comparison of `expression`, from left to
/// right, outside call arguments, shift amounts, and the conditions of
/// conditionals.
///
/// Every other operator gives its result the type of its operands, a shift
/// or rotation amount is a literal, and a conditional has the type of its
/// values, so this leaf's type is the type of the whole expression. A
/// comparison is a leaf of type `Bool`. Integer literals take their type from
/// their context and are skipped. An array or tuple literal ends the search
/// so that a conversion can reject it. A branch's leaf that names one of the branch's
/// own `let` bindings, directly or through indices, is skipped too, because
/// those bindings are not in scope where the type is needed.
/// Parser-established expression height bounds this recursion.
fn first_typed_leaf(expression: &Expression) -> Option<&Expression> {
    match &expression.kind {
        ExpressionKind::Literal(_) => None,
        ExpressionKind::Name(_)
        | ExpressionKind::Call(_)
        | ExpressionKind::Conversion(_)
        | ExpressionKind::Index(_)
        | ExpressionKind::Array(_)
        | ExpressionKind::Fill(_)
        | ExpressionKind::Loop(_)
        | ExpressionKind::Tuple(_)
        | ExpressionKind::Project(_)
        | ExpressionKind::Bytes(_)
        | ExpressionKind::Slice(_) => Some(expression),
        ExpressionKind::Parenthesized(inner) => first_typed_leaf(inner),
        // An update has the type of the array it updates.
        ExpressionKind::Update(update) => first_typed_leaf(&update.base),
        ExpressionKind::SliceUpdate(update) => first_typed_leaf(&update.base),
        ExpressionKind::Unary(unary) => first_typed_leaf(&unary.operand),
        // A comparison has type `Bool`, and `++` has a length of its own.
        ExpressionKind::Binary(binary)
            if binary.operator.is_comparison() || binary.operator.is_concatenation() =>
        {
            Some(expression)
        }
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
            .find_map(|arm| branch_leaf(&arm.bindings, &arm.value))
            .or_else(|| branch_leaf(&conditional.otherwise_bindings, &conditional.otherwise)),
    }
}

/// The label of an operand with no type because each branch that could give
/// one names a binding of its own.
const BRANCH_BINDING_LEAF_LABEL: &str = "a branch's own bindings are not in scope outside it";

/// Returns whether the search for the first typed leaf of `expression`
/// passes over a branch whose leaf names one of that branch's bindings.
/// Parser-established expression height bounds this recursion.
fn passes_branch_binding(expression: &Expression) -> bool {
    let block = |bindings: &[Binding], value: &Expression| {
        (first_typed_leaf(value).is_some() && branch_leaf(bindings, value).is_none())
            || passes_branch_binding(value)
    };
    match &expression.kind {
        ExpressionKind::Parenthesized(inner) => passes_branch_binding(inner),
        ExpressionKind::Update(update) => passes_branch_binding(&update.base),
        ExpressionKind::SliceUpdate(update) => passes_branch_binding(&update.base),
        ExpressionKind::Unary(unary) => passes_branch_binding(&unary.operand),
        ExpressionKind::Binary(binary)
            if !binary.operator.is_comparison() && !binary.operator.is_concatenation() =>
        {
            passes_branch_binding(&binary.left)
                || (!binary.operator.is_shift_or_rotation() && passes_branch_binding(&binary.right))
        }
        ExpressionKind::Conditional(conditional) => {
            conditional
                .arms
                .iter()
                .any(|arm| block(&arm.bindings, &arm.value))
                || block(&conditional.otherwise_bindings, &conditional.otherwise)
        }
        _ => false,
    }
}

/// Returns the first typed leaf of a branch's value unless it names one of
/// the branch's bindings.
fn branch_leaf<'expression>(
    bindings: &[Binding],
    value: &'expression Expression,
) -> Option<&'expression Expression> {
    let leaf = first_typed_leaf(value)?;
    let bound =
        leaf_root_name(leaf).is_some_and(|name| find_binding(bindings, &name.text).is_some());
    (!bound).then_some(leaf)
}

/// Returns the name a name or index leaf reads, through any indices.
/// Parser-established expression height bounds this recursion.
fn leaf_root_name(leaf: &Expression) -> Option<&Identifier> {
    match &leaf.kind {
        ExpressionKind::Name(name) => Some(name),
        ExpressionKind::Index(index) => leaf_root_name(&index.base),
        ExpressionKind::Project(project) => leaf_root_name(&project.base),
        ExpressionKind::Slice(slice) => leaf_root_name(&slice.base),
        _ => None,
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
            id_offset: 0,
            tests: true,
            reserve_pending_function_slot,
            reserve_magnitude_limb,
            reserve_range_limbs,
            reserve_core_name,
            reserve_diagnostic_slots,
            reserve_core_node_slot,
            reserve_call_edge_slot,
            types: TypeTable::new(),
        }
    }

    /// Places this module's typed `spec` identities after `offset` others.
    const fn with_id_offset(mut self, offset: usize) -> Self {
        self.id_offset = offset;
        self
    }

    /// Sets whether this module's known-answer tests are checked and become
    /// part of its Core.
    const fn with_tests(mut self, tests: bool) -> Self {
        self.tests = tests;
        self
    }

    #[cfg(test)]
    fn run(self) -> AnalysisResult {
        self.run_with_reservations(
            |declarations, capacity| declarations.try_reserve(capacity).is_ok(),
            |functions, capacity| functions.try_reserve_exact(capacity).is_ok(),
        )
    }

    #[cfg(test)]
    fn run_with_reservations(
        self,
        reserve_declarations: impl FnOnce(&mut DeclarationIndex<'ast>, usize) -> bool,
        reserve_core_functions: impl FnOnce(&mut Vec<CoreFunction>, usize) -> bool,
    ) -> AnalysisResult {
        let outcome = self.run_linked(&[], reserve_declarations, reserve_core_functions);
        AnalysisResult {
            core: outcome.core,
            diagnostics: outcome.diagnostics,
        }
    }

    /// Checks this module against the tables of the modules it uses, given
    /// in the order of its `use` declarations.
    fn run_linked(
        mut self,
        imports: &[ImportScope<'_, 'ast>],
        reserve_declarations: impl FnOnce(&mut DeclarationIndex<'ast>, usize) -> bool,
        reserve_core_functions: impl FnOnce(&mut Vec<CoreFunction>, usize) -> bool,
    ) -> ModuleOutcome<'ast> {
        if !(self.reserve_diagnostic_slots)(
            &mut self.diagnostics,
            MAX_RETAINED_SEMANTIC_DIAGNOSTICS,
        ) {
            return ModuleOutcome {
                core: None,
                diagnostics: self.diagnostics,
                tables: None,
            };
        }
        let mut declarations = DeclarationIndex::new();
        let declaration_capacity = self.ast.module.functions.len();
        if !reserve_declarations(&mut declarations, declaration_capacity) {
            self.resource_limit(
                self.ast.module.span,
                "semantic declaration namespace storage allocation failed",
            );
            return ModuleOutcome {
                core: None,
                diagnostics: self.diagnostics,
                tables: None,
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
        self.resolve_moduli();
        self.analyze_type_declarations();
        let Some(signatures) = self.collect_signatures() else {
            return ModuleOutcome {
                core: None,
                diagnostics: self.diagnostics,
                tables: None,
            };
        };
        let mut pending_functions = Vec::new();
        let mut call_edges = Vec::new();

        // One event for each `use` declaration, whose module the program's
        // module graph has already resolved.
        for declaration in &self.ast.module.uses {
            if !self.event(declaration.name.span) {
                break;
            }
        }

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
                let Some(signature) = signatures.get(source_index).and_then(Option::as_ref) else {
                    self.resource_limit(function.span, "semantic signature table is inconsistent");
                    break;
                };
                // Malformed size parameters are reported once, and the
                // function then has no instances to check.
                if !self.check_size_parameters(function) {
                    if self.halted {
                        break;
                    }
                    continue;
                }
                let Some(ranges) = signature.ranges else {
                    self.resource_limit(function.span, "semantic size table is inconsistent");
                    break;
                };
                let scope = ModuleScope {
                    declarations: &declarations,
                    signatures: &signatures,
                    imports,
                };
                // Each instance is checked as the function written out with
                // its sizes' values, in order, until one is in error.
                for index in 0..ranges.instances() {
                    let (Some(instance), Some(id)) = (
                        ranges.instance(&function.sizes, index),
                        signature.instance_id(index),
                    ) else {
                        self.resource_limit(function.span, "semantic size table is inconsistent");
                        break;
                    };
                    self.enter_instance(instance, &signature.listed);
                    let context = BodyContext {
                        id,
                        instance,
                        name: &function.name,
                        parameters: &function.parameters,
                        parameter_types: Vec::new(),
                        bindings: &body.bindings,
                        binding_types: Vec::new(),
                        loop_scopes: Vec::new(),
                        blocks: Vec::new(),
                        finished_blocks: Vec::new(),
                        loops: Vec::new(),
                        conditionals: Vec::new(),
                    };
                    let reported = self.diagnostics.len();
                    let pending = self.analyze_typed_function(
                        function,
                        body,
                        context,
                        &scope,
                        &mut call_edges,
                    );
                    self.name_instance(function, instance, reported);
                    let Some(pending) = pending else {
                        break;
                    };
                    if (self.reserve_pending_function_slot)(&mut pending_functions) {
                        pending_functions.push(pending);
                    } else {
                        self.resource_limit(
                            function.span,
                            "semantic analysis could not allocate pending function storage",
                        );
                        break;
                    }
                    if self.halted || self.diagnostics.len() > reported {
                        break;
                    }
                }
                self.enter_instance(Instance::NONE, &[]);
                if self.halted {
                    break;
                }
            }
        }

        if !self.halted {
            self.check_call_graph(&signatures, &call_edges);
        }
        if self.tests && !self.halted {
            let scope = ModuleScope {
                declarations: &declarations,
                signatures: &signatures,
                imports,
            };
            // The tests' identities follow every instance of the functions.
            let first_id = signatures
                .iter()
                .flatten()
                .map(|signature| signature.instances.len())
                .fold(self.id_offset, usize::saturating_add);
            let mut test_edges = Vec::new();
            self.analyze_tests(&scope, first_id, &mut pending_functions, &mut test_edges);
        }

        let core = if self.diagnostics.is_empty() && !self.halted {
            self.construct_core(pending_functions, reserve_core_functions)
        } else {
            None
        };
        ModuleOutcome {
            core,
            diagnostics: self.diagnostics,
            tables: Some(ModuleTables {
                declarations,
                signatures,
            }),
        }
    }

    /// Resolves every typed `spec` signature without events or diagnostics.
    ///
    /// Types are reported once, in source order, when their own declaration is
    /// checked; calls to a function whose signature did not resolve are then
    /// not reported again.
    fn collect_signatures(&mut self) -> Option<Vec<Option<Signature<'ast>>>> {
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
                    let Some(id) = self
                        .id_offset
                        .checked_add(next_id)
                        .and_then(CoreFunctionId::from_index)
                    else {
                        self.resource_limit(
                            function.span,
                            "Core function identity exceeds the u32 representation limit",
                        );
                        return None;
                    };
                    let listed: Vec<Vec<Option<CoreType>>> = function
                        .sizes
                        .iter()
                        .map(|size| {
                            size.types
                                .iter()
                                .map(|ty| silent_type(self.source, &self.types, ty))
                                .collect()
                        })
                        .collect();
                    // A list with a type that does not resolve, or one type
                    // twice, is reported at the declaration; like malformed
                    // sizes, it leaves the function no instances, so that
                    // its callers are not reported again.
                    let ranges = SizeRanges::of(self.source, function)
                        .filter(|_| listed.iter().all(|types| distinct_types(types)));
                    let count = ranges.map_or(0, |ranges| ranges.instances());
                    next_id = next_id.saturating_add(count);
                    let spellings = type_spellings(self.source, &function.sizes);
                    let mut instances = Vec::new();
                    if instances.try_reserve_exact(count).is_err() {
                        self.resource_limit(function.span, "semantic signature allocation failed");
                        return None;
                    }
                    for index in 0..count {
                        let Some(instance) =
                            ranges.and_then(|ranges| ranges.instance(&function.sizes, index))
                        else {
                            self.resource_limit(
                                function.span,
                                "semantic size table is inconsistent",
                            );
                            return None;
                        };
                        self.enter_instance(instance, &listed);
                        let mut parameters = Vec::new();
                        if parameters
                            .try_reserve_exact(function.parameters.len())
                            .is_err()
                        {
                            self.resource_limit(
                                function.span,
                                "semantic signature allocation failed",
                            );
                            return None;
                        }
                        parameters.extend(
                            function.parameters.iter().map(|parameter| {
                                silent_type(self.source, &self.types, &parameter.ty)
                            }),
                        );
                        let result_type = silent_type(self.source, &self.types, &body.result_type);
                        // Each part of a size evaluated for a signature is
                        // one event, as when a body is checked.
                        if !self.charge_size_events(function.name.span) {
                            return None;
                        }
                        instances.push(InstanceSignature {
                            parameters,
                            result_type,
                        });
                    }
                    self.enter_instance(Instance::NONE, &[]);
                    Some(Signature {
                        id,
                        sizes: &function.sizes,
                        ranges,
                        listed,
                        spellings,
                        instances,
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
                .push(checked.as_ref().map(|checked| checked.ty.clone()));
            if let Some(CheckedBinding {
                ty,
                nodes: Some(nodes),
            }) = checked
            {
                let name = self.pattern_core_name(&binding.pattern)?;
                locals.push(CoreLocal {
                    span: binding.span,
                    name,
                    name_span: pattern_name_span(&binding.pattern),
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
            &result_type.clone(),
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
            .cloned()
            .collect::<Option<Vec<_>>>()?;
        let name = self.copy_core_name(&function.name.text, function.name.span)?;
        let mut sizes = Vec::new();
        if sizes
            .try_reserve_exact(context.instance.values().len())
            .is_err()
        {
            self.resource_limit(function.span, "size storage allocation failed");
            return None;
        }
        sizes.extend_from_slice(context.instance.values());
        let instance = context
            .instance
            .suffix(&type_spellings(self.source, &function.sizes));
        Some(PendingFunction {
            span: function.span,
            name,
            name_span: function.name.span,
            sizes,
            instance,
            parameters,
            result_type,
            locals,
            nodes,
            loops,
            conditionals,
            title: None,
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
        let names = binding.pattern.names();
        let mut unique = true;
        for (position, typed) in names.iter().enumerate() {
            // One event for each bound name's uniqueness check.
            if !self.event(typed.name.span) {
                return None;
            }
            let text = &typed.name.text;
            // A type parameter names a type, not a value, so a binding may
            // share its name, as a parameter may.
            let size = function
                .sizes
                .iter()
                .find(|size| !size.is_type() && size.name.text == *text)
                .map(|size| (size.name.span, "the size parameter is here"));
            let parameter = function
                .parameters
                .iter()
                .find(|parameter| parameter.name.text == *text)
                .map(|parameter| (parameter.name.span, "the parameter is here"));
            let earlier = size
                .or(parameter)
                .or_else(|| {
                    body.bindings
                        .get(..index)?
                        .iter()
                        .find_map(|earlier| pattern_name(&earlier.pattern, text))
                        .map(|earlier| (earlier.name.span, "the first binding is here"))
                })
                .or_else(|| {
                    names
                        .get(..position)?
                        .iter()
                        .find(|earlier| earlier.name.text == *text)
                        .map(|earlier| (earlier.name.span, "the first name is here"))
                });
            if let Some((earlier_span, earlier_label)) = earlier {
                unique = false;
                let span = typed.name.span;
                if self.begin_report(span) {
                    let name = identifier_spelling_for_diagnostic(text);
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
        }
        let ty = self.analyze_pattern_type(&binding.pattern, "binding type")?;
        let mut output = BodyOutput {
            nodes: Vec::new(),
            call_edges,
        };
        let checked =
            self.check_expression(&binding.value, &ty.clone(), context, scope, &mut output);
        Some(CheckedBinding {
            ty,
            nodes: (checked && unique).then_some(output.nodes),
        })
    }

    /// Checks `expression` against `expected` and appends its Core nodes in
    /// postorder. Returns whether the expression is well typed.
    ///
    /// Parser-established expression height bounds this recursion.
    fn check_expression(
        &mut self,
        expression: &'ast Expression,
        expected: &CoreType,
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
                    if *expected == CoreType::Bool {
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
                    expected.clone(),
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
                        expected.clone(),
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
                if !self.event(expression.span) || !self.event(fill.length.span) {
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
            ExpressionKind::Tuple(tuple) => {
                if !self.event(expression.span) {
                    return false;
                }
                self.check_tuple(expression, tuple, expected, context, scope, output)
            }
            ExpressionKind::Project(project) => {
                if !self.event(project.position_span) {
                    return false;
                }
                self.check_project(expression, project, expected, context, scope, output)
            }
            ExpressionKind::Bytes(bytes) => {
                if !self.event(expression.span) {
                    return false;
                }
                self.check_bytes(expression, *bytes, expected, output)
            }
            ExpressionKind::Slice(slice) => {
                if !self.event(slice.range.span) {
                    return false;
                }
                self.check_slice(expression, slice, expected, context, scope, output)
            }
            ExpressionKind::SliceUpdate(update) => {
                if !self.event(update.keyword_span) {
                    return false;
                }
                self.check_slice_update(expression, update, expected, context, scope, output)
            }
            ExpressionKind::Binary(binary) => {
                if !self.event(binary.operator_span) {
                    return false;
                }
                if binary.operator.is_comparison() {
                    return self
                        .check_comparison(expression, binary, expected, context, scope, output);
                }
                if binary.operator.is_concatenation() {
                    return self
                        .check_concatenation(expression, binary, expected, context, scope, output);
                }
                if !self.binary_is_defined(binary, expected) {
                    return false;
                }
                let left = self.check_expression(&binary.left, expected, context, scope, output);
                if binary.operator.is_shift_or_rotation() {
                    if !matches!(binary.right.kind, ExpressionKind::Literal(_)) {
                        let amount = self.check_computed_amount(binary, context, scope, output);
                        return match (left, amount) {
                            (true, Some(amount)) => self.push_node(
                                output,
                                expression.span,
                                expected.clone(),
                                CoreNodeKind::ShiftBy {
                                    operator: binary.operator,
                                    amount,
                                },
                            ),
                            _ => false,
                        };
                    }
                    let amount = self.check_shift_amount(binary, expected);
                    return match (left, amount) {
                        (true, Some(amount)) => self.push_node(
                            output,
                            expression.span,
                            expected.clone(),
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
                        expected.clone(),
                        CoreNodeKind::Binary(binary.operator),
                    )
            }
        }
    }

    fn check_name_reference(
        &mut self,
        name: &'ast Identifier,
        expected: &CoreType,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let (whole, kind, element) = match context.resolve(&name.text) {
            // A size is an `Int` constant of the instance.
            NameResolution::Size(value) => {
                let Some(value) =
                    ExactInteger::from_u64(u64::from(value), self.reserve_range_limbs)
                else {
                    self.resource_limit(name.span, "size storage allocation failed");
                    return false;
                };
                (
                    Some(Some(CoreType::Int)),
                    Ok(CoreNodeKind::Literal(CoreValue::Int(value))),
                    None,
                )
            }
            NameResolution::Parameter(index) => (
                context.parameter_types.get(index).cloned(),
                u32::try_from(index).map(CoreNodeKind::Parameter),
                None,
            ),
            NameResolution::Binding(index, element) => (
                context.binding_types.get(index).cloned(),
                u32::try_from(index).map(CoreNodeKind::Local),
                element,
            ),
            NameResolution::BlockBinding {
                block,
                index,
                element,
            } => match context.blocks.get(block) {
                Some(block) => (
                    block.binding_types.get(index).cloned(),
                    u32::try_from(index).map(|index| block.owner.node(index)),
                    element,
                ),
                None => (None, Ok(CoreNodeKind::Local(0)), None),
            },
            NameResolution::LoopIndex(position) => match context.loop_scopes.get(position) {
                Some(scope) => (
                    Some(Some(CoreType::Int)),
                    Ok(CoreNodeKind::LoopIndex(scope.id)),
                    None,
                ),
                None => (None, Ok(CoreNodeKind::LoopIndex(0)), None),
            },
            NameResolution::Accumulator(position, element) => {
                match context.loop_scopes.get(position) {
                    Some(scope) => (
                        Some(Some(scope.ty.clone())),
                        Ok(CoreNodeKind::Accumulator(scope.id)),
                        element,
                    ),
                    None => (None, Ok(CoreNodeKind::Accumulator(0)), None),
                }
            }
            NameResolution::BoolLiteral(value) => (
                Some(Some(CoreType::Bool)),
                Ok(CoreNodeKind::Literal(CoreValue::Bool(value))),
                None,
            ),
            NameResolution::LaterBinding(typed, in_block) => {
                if self.begin_report(name.span) {
                    let spelling = identifier_spelling_for_diagnostic(&name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnknownParameter,
                            format!("`{spelling}` is used before it is bound"),
                            name.span,
                        )
                        .with_label("not bound yet")
                        .with_secondary_span(typed.name.span, "the binding is here")
                        .with_note(if in_block {
                            "a binding is in scope after its own `;`, for the bindings that \
                             follow it and the value of its step or branch"
                        } else {
                            "a binding is in scope after its own `;`, for the bindings that \
                             follow it and the result"
                        }),
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
        let Some(Some(whole)) = whole else {
            return false;
        };
        // A name of a tuple pattern stands for one element of the pattern's
        // value: its Core reads the value and then selects the element.
        let Some(actual) = element_type(Some(whole.clone()), element) else {
            self.resource_limit(name.span, "semantic pattern table is inconsistent");
            return false;
        };
        if actual != *expected {
            if self.begin_report(name.span) {
                let spelling = identifier_spelling_for_diagnostic(&name.text);
                let note = if actual.as_array().map(ArrayType::element).as_ref() == Some(expected) {
                    format!("select one element with an index, such as `{spelling}[0]`")
                } else if actual
                    .as_tuple()
                    .is_some_and(|tuple| tuple.elements().contains(expected))
                {
                    format!("select one element by its position, such as `{spelling}.0`")
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
        match element {
            None => self.push_node(output, name.span, expected.clone(), kind),
            Some(index) => {
                self.push_node(output, name.span, whole, kind)
                    && self.push_node(
                        output,
                        name.span,
                        expected.clone(),
                        CoreNodeKind::Project { index },
                    )
            }
        }
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
        // A binding of a step or branch already checked is out of scope.
        let finished = context.finished_blocks.iter().rev().find_map(|bindings| {
            bindings
                .iter()
                .find_map(|binding| pattern_name(&binding.pattern, &name.text))
        });
        if let Some(binding) = finished {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnknownParameter,
                    format!("`{spelling}` is not in scope here"),
                    name.span,
                )
                .with_label("unknown name")
                .with_secondary_span(binding.name.span, "a binding of this name is here")
                .with_note(
                    "a binding of a loop's step or a branch is in scope only within that \
                     step or branch",
                ),
            );
            return;
        }
        let has_bindings = !context.bindings.is_empty()
            || context
                .blocks
                .iter()
                .any(|block| !block.bindings.is_empty());
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
        diagnostic = if self.types.sizes.instance.find_type(&name.text).is_some() {
            diagnostic.with_note(format!(
                "`{spelling}` is a type parameter: it names a type, not a value, so it is \
                 written where a type is, as in `let x: {spelling} = 0;`"
            ))
        } else if first_declaration(scope.declarations, FunctionKind::Spec, &name.text).is_some() {
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
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        if conversion.order.is_some() {
            return self.check_packing(expression, conversion, expected, context, scope, output);
        }
        let target = self.analyze_type(&conversion.target, "conversion type");
        if self.halted {
            return false;
        }
        let target_matches = match target.clone() {
            Some(target) if target != *expected => {
                self.report_conversion_mismatch(conversion.target.span, &target, expected);
                false
            }
            Some(_) => true,
            None => false,
        };
        let Some(leaf) = first_typed_leaf(&conversion.operand) else {
            self.report_untyped_conversion(&conversion.operand);
            return false;
        };
        if let Some(target) = target.clone().filter(|target| !target.is_scalar()) {
            self.report_array_conversion(conversion.target.span, &target);
            return false;
        }
        let from = self.leaf_type(leaf, context, scope);
        if matches!(
            leaf.kind,
            ExpressionKind::Array(_) | ExpressionKind::Tuple(_)
        ) || from.clone().is_some_and(|from| !from.is_scalar())
        {
            let tuple = matches!(leaf.kind, ExpressionKind::Tuple(_));
            self.report_array_operand(conversion.keyword_span, from, tuple);
            return false;
        }
        let Some(from) = from else {
            // The leaf's own check reports why it has no type. That check
            // stops before comparing with the type passed here.
            self.check_untyped(leaf, expected, context, scope, output);
            return false;
        };
        if from == CoreType::Bool || target == Some(CoreType::Bool) {
            self.report_bool_conversion(conversion.keyword_span);
            return false;
        }
        let operand =
            self.check_expression(&conversion.operand, &from.clone(), context, scope, output);
        operand
            && target_matches
            && self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::Convert { from },
            )
    }

    // The reports of a conversion are out of line, so that the frame of
    // `check_conversion`, which nested conversions stack, stays small.
    #[cold]
    #[inline(never)]
    fn report_conversion_mismatch(&mut self, span: Span, target: &CoreType, expected: &CoreType) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("this conversion gives `{target}`, but `{expected}` is required here"),
                    span,
                )
                .with_label(format!("expected `{expected}`"))
                .with_note("`as` gives exactly the type written after it"),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_untyped_conversion(&mut self, operand: &Expression) {
        let span = operand.span;
        if !self.begin_report(span) {
            return;
        }
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UntypedConversionOperand,
            "the operand of `as` has no type of its own",
            span,
        );
        self.diagnostics.push(if passes_branch_binding(operand) {
            diagnostic.with_label(BRANCH_BINDING_LEAF_LABEL).with_note(
                "bind the conditional's value with a typed `let` first, or convert within \
                 each branch",
            )
        } else {
            diagnostic
                .with_label("a literal takes its type from where it is used")
                .with_note(
                    "write the literal where its type is required, or give it a type with a \
                     `let` binding",
                )
        });
    }

    #[cold]
    #[inline(never)]
    fn report_array_operand(&mut self, span: Span, from: Option<CoreType>, tuple: bool) {
        if self.begin_report(span) {
            let tuple = tuple || from.as_ref().is_some_and(|from| from.as_tuple().is_some());
            let words = from.as_ref().and_then(CoreType::words).is_some();
            let operand = from.map_or_else(
                || String::from(if tuple { "a tuple" } else { "an array" }),
                |from| format!("`{from}`"),
            );
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    format!("`as` is not defined for {operand}"),
                    span,
                )
                .with_label("`as` converts one `Int`, word, or residue value")
                .with_note(if tuple {
                    "convert each element, such as `p.0 as Int`"
                } else if words {
                    "name a byte order to read the words as one number or as words of another \
                     width, as in `x as big Int`, or convert each element, such as `x[0] as Int`"
                } else {
                    "convert each element, such as `x[0] as Int`"
                }),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_bool_conversion(&mut self, span: Span) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    "`as` does not convert to or from `Bool`",
                    span,
                )
                .with_label("`as` converts one `Int`, word, or residue value")
                .with_note(
                    "choose a number with a conditional, such as `if b { 1 } else { 0 }`, or \
                     compare a number, such as `x != 0`",
                ),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_array_conversion(&mut self, span: Span, target: &CoreType) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    if target.as_tuple().is_some() {
                        format!("`as` does not convert to the tuple type `{target}`")
                    } else {
                        format!("`as` does not convert to the array type `{target}`")
                    },
                    span,
                )
                .with_label("`as` gives one `Int`, word, or residue value")
                .with_note(if target.as_tuple().is_some() {
                    String::from("convert each element, such as `(p.0 as Int, p.1 as Int)`")
                } else if target.words().is_some() {
                    format!(
                        "name a byte order to write words as `{target}`, as in `as big \
                         {target}`, or build the array from its elements"
                    )
                } else {
                    String::from("convert each element, such as `x[0] as Int`")
                }),
            );
        }
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
                let (declarations, signatures) = scope.tables_for(call)?;
                let entry = first_declaration(declarations, FunctionKind::Spec, &call.callee.text)?;
                let signature = signatures.get(entry.source_index)?.as_ref()?;
                self.silent_instance(call, signature, context, scope)?
                    .result_type
                    .clone()
            }
            ExpressionKind::Conversion(conversion) => {
                silent_type(self.source, &self.types, &conversion.target)
            }
            ExpressionKind::Index(index) => {
                // Chained selections grow height without delimiter nesting.
                // Walk their spine in one frame, then apply each exact
                // element type. Rank bounds stop invalid extra selections.
                let mut base = &index.base;
                let mut selections = 1_usize;
                while let ExpressionKind::Index(inner) = &base.kind {
                    selections = selections.saturating_add(1);
                    base = &inner.base;
                }
                let mut ty = self.leaf_type(base, context, scope)?;
                for _ in 0..selections {
                    ty = ty.as_array()?.element();
                }
                Some(ty)
            }
            ExpressionKind::Project(project) => self
                .leaf_type(&project.base, context, scope)?
                .as_tuple()?
                .element(project.position)
                .cloned(),
            ExpressionKind::Loop(r#loop) => {
                silent_pattern_type(self.source, &self.types, &r#loop.accumulator)
            }
            ExpressionKind::Update(update) => {
                self.leaf_type(first_typed_leaf(&update.base)?, context, scope)
            }
            ExpressionKind::Binary(binary) if binary.operator.is_comparison() => {
                Some(CoreType::Bool)
            }
            ExpressionKind::Binary(binary) if binary.operator.is_concatenation() => {
                self.concatenation_type(binary, context, scope)
            }
            ExpressionKind::Bytes(_) => {
                let length = self.array_length_of(leaf, context, scope)?;
                ArrayType::new(&CoreType::Word8, length).map(CoreType::Array)
            }
            ExpressionKind::Slice(slice) => {
                let base = self.leaf_type(&slice.base, context, scope)?;
                let array = base.as_array()?;
                let length = self.slice_length(&slice.range, array.length(), context)?;
                ArrayType::new(&array.element(), length).map(CoreType::Array)
            }
            ExpressionKind::SliceUpdate(update) => {
                self.leaf_type(first_typed_leaf(&update.base)?, context, scope)
            }
            ExpressionKind::Literal(_)
            | ExpressionKind::Unary(_)
            | ExpressionKind::Binary(_)
            | ExpressionKind::Parenthesized(_)
            | ExpressionKind::Array(_)
            | ExpressionKind::Fill(_)
            | ExpressionKind::Conditional(_)
            | ExpressionKind::Tuple(_) => None,
        }
    }

    /// Checks an array literal against `expected`, which must be an array
    /// type of the same length; each element is checked against its element
    /// type.
    fn check_array(
        &mut self,
        expression: &'ast Expression,
        array: &'ast ArrayExpression,
        expected: &CoreType,
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
                self.check_expression(element, &array_type.element(), context, scope, output);
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
            expected.clone(),
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
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let mut expression = expression;
        let mut index = index;
        let base_type = loop {
            if let Some(ty) = self.leaf_type(&index.base, context, scope) {
                break ty;
            }
            // Invalid extra selections have no type. Descend to the first
            // diagnosable suffix without a checker frame per suffix.
            if let ExpressionKind::Index(inner) = &index.base.kind {
                expression = &index.base;
                index = inner;
                continue;
            }
            self.check_untyped(&index.base, expected, context, scope, output);
            return false;
        };
        let Some(array_type) = base_type.as_array() else {
            self.report_not_an_array(index.base.span, &base_type, false);
            self.check_expression(&index.base, &base_type, context, scope, output);
            return false;
        };
        let literal = index.is_literal();
        let position = if literal {
            self.check_index_literal(index, array_type)
        } else {
            None
        };
        let element = array_type.element();
        let element_matches = element == *expected;
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
        let base = self.check_expression(&index.base, &base_type, context, scope, output);
        if !literal {
            let in_range =
                self.check_static_index(&index.index, array_type, context, scope, output);
            return in_range
                && base
                && element_matches
                && self.push_node(
                    output,
                    expression.span,
                    expected.clone(),
                    CoreNodeKind::Select,
                );
        }
        match position {
            Some(position) if base && element_matches => self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::Index { index: position },
            ),
            _ => false,
        }
    }

    /// Reports an index or an update, when `update`, of a value of
    /// `base_type`, which is not an array.
    #[cold]
    #[inline(never)]
    fn report_not_an_array(&mut self, span: Span, base_type: &CoreType, update: bool) {
        if !self.begin_report(span) {
            return;
        }
        let tuple = base_type.as_tuple().is_some();
        let message = if update {
            format!("only an array can be updated, but this has type `{base_type}`")
        } else {
            format!("only an array can be indexed, but this has type `{base_type}`")
        };
        let label = if tuple {
            format!("`{base_type}` is a tuple, not an array")
        } else {
            format!("`{base_type}` has no elements")
        };
        let note = match (update, tuple) {
            (false, false) => "an index selects one element of a value of type `T^n`",
            (false, true) => "a tuple's element is selected by its position, such as `p.0`",
            (true, false) => "`x with [i] = v` is the array `x` with one element replaced",
            (true, true) => "a tuple with one element replaced is written anew, such as `(v, p.1)`",
        };
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::NotAnArray, message, span)
                .with_label(label)
                .with_note(note),
        );
    }

    /// Checks `base with [index] = value` against `expected`, which must be
    /// the base's array type.
    fn check_update(
        &mut self,
        expression: &'ast Expression,
        update: &'ast UpdateExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let base_type =
            first_typed_leaf(&update.base).and_then(|leaf| self.leaf_type(leaf, context, scope));
        if let Some(base_type) = base_type
            && base_type.as_array().is_none()
        {
            self.report_not_an_array(update.base.span, &base_type, true);
            self.check_expression(&update.base, &base_type, context, scope, output);
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
        let value = self.check_expression(&update.value, &array.element(), context, scope, output);
        base && index
            && value
            && self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::Update,
            )
    }

    /// Checks `[element; n]` against `expected`, which must be an array type
    /// of length n.
    fn check_fill(
        &mut self,
        expression: &'ast Expression,
        fill: &'ast FillExpression,
        expected: &CoreType,
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
        let length = self.checked_length(&fill.length);
        if self.halted {
            return false;
        }
        match length {
            None => {}
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
        let element =
            self.check_expression(&fill.element, &array.element(), context, scope, output);
        element
            && length == Some(array.length())
            && self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::Fill,
            )
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
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let bounds = self.check_loop_bounds(r#loop);
        if self.halted {
            return false;
        }
        let mut names_unique = self.check_new_name(&r#loop.index, context, None, &[]);
        let accumulator = r#loop.accumulator.names();
        for (position, typed) in accumulator.iter().enumerate() {
            let earlier_names = accumulator.get(..position).unwrap_or_default();
            names_unique &=
                self.check_new_name(&typed.name, context, Some(&r#loop.index), earlier_names);
        }
        if self.halted {
            return false;
        }
        let Some(ty) = self.analyze_pattern_type(&r#loop.accumulator, "accumulator type") else {
            return false;
        };
        let type_matches = ty == *expected;
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
        let init = self.check_expression(&r#loop.init, &ty.clone(), context, scope, output);
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
            ty: ty.clone(),
            start,
            end,
        });
        let mut step_output = BodyOutput {
            nodes: Vec::new(),
            call_edges: &mut *output.call_edges,
        };
        let (bindings, step) = self.check_block(
            BlockOwner::Step(id),
            &r#loop.step_bindings,
            &r#loop.step,
            &ty.clone(),
            context,
            scope,
            &mut step_output,
        );
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
        let Some(bindings) = bindings.filter(|_| init && step && names_unique && type_matches)
        else {
            return false;
        };
        if self.halted {
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
        let Some(accumulator_name) = self.pattern_core_name(&r#loop.accumulator) else {
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
            bindings,
            step: CoreExpression { nodes: step_nodes },
        });
        self.push_node(
            output,
            expression.span,
            expected.clone(),
            CoreNodeKind::Fold(id),
        )
    }

    /// Checks a loop's step or a conditional's branch: its `let` bindings in
    /// order, each against its declared type and in scope from the binding
    /// after it, and then its value against `expected`. The bindings' value
    /// nodes and then the value's nodes are appended to `output`.
    ///
    /// Returns the bindings' Core records, present only when every binding
    /// is well formed, and whether the value is well typed.
    #[allow(clippy::too_many_arguments)]
    fn check_block(
        &mut self,
        owner: BlockOwner,
        bindings: &'ast [Binding],
        value: &'ast Expression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> (Option<Vec<CoreBinding>>, bool) {
        let mut binding_types = Vec::new();
        let mut records = Vec::new();
        if context.blocks.try_reserve(1).is_err()
            || binding_types.try_reserve_exact(bindings.len()).is_err()
            || records.try_reserve_exact(bindings.len()).is_err()
        {
            self.resource_limit(value.span, "block binding storage allocation failed");
            return (None, false);
        }
        context.blocks.push(BlockScope {
            owner,
            bindings,
            binding_types,
        });
        let mut well_formed = true;
        for binding in bindings {
            match self.check_block_binding(binding, context, scope, output) {
                Some((ty, checked)) => {
                    if let Some(block) = context.blocks.last_mut() {
                        block.binding_types.push(ty.clone());
                    }
                    let Some(ty) = ty.filter(|_| checked && well_formed) else {
                        well_formed = false;
                        continue;
                    };
                    let Ok(end) = u32::try_from(output.nodes.len()) else {
                        self.resource_limit(
                            binding.span,
                            "node count exceeds the u32 representation limit",
                        );
                        well_formed = false;
                        break;
                    };
                    let Some(name) = self.pattern_core_name(&binding.pattern) else {
                        well_formed = false;
                        break;
                    };
                    records.push(CoreBinding {
                        span: binding.span,
                        name,
                        name_span: pattern_name_span(&binding.pattern),
                        ty,
                        end,
                    });
                }
                None => {
                    well_formed = false;
                    break;
                }
            }
        }
        let checked =
            !self.halted && self.check_expression(value, expected, context, scope, output);
        context.blocks.pop();
        if !bindings.is_empty() {
            if context.finished_blocks.try_reserve(1).is_err() {
                self.resource_limit(value.span, "block binding storage allocation failed");
                return (None, false);
            }
            context.finished_blocks.push(bindings);
        }
        (well_formed.then_some(records), checked)
    }

    /// Checks one binding of a block: that its name is new in scope, its
    /// type, and its value against that type.
    ///
    /// Returns the binding's type, when it resolved, and whether the binding
    /// is well formed, or `None` when analysis stopped.
    fn check_block_binding(
        &mut self,
        binding: &'ast Binding,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> Option<(Option<CoreType>, bool)> {
        let names = binding.pattern.names();
        let mut unique = true;
        for (position, typed) in names.iter().enumerate() {
            let earlier_names = names.get(..position).unwrap_or_default();
            unique &= self.check_new_name(&typed.name, context, None, earlier_names);
        }
        if self.halted {
            return None;
        }
        let ty = self.analyze_pattern_type(&binding.pattern, "binding type");
        if self.halted {
            return None;
        }
        let Some(ty) = ty else {
            return Some((None, false));
        };
        let checked = self.check_expression(&binding.value, &ty.clone(), context, scope, output);
        if self.halted {
            return None;
        }
        Some((Some(ty), checked && unique))
    }

    /// Checks that a name a loop or a block introduces repeats no name in
    /// scope, not `index` when it is a sibling loop index, and none of the
    /// earlier names of its own pattern, and reports it if it does. One
    /// event is counted for the check.
    ///
    /// Returns whether the name is new.
    fn check_new_name(
        &mut self,
        name: &Identifier,
        context: &BodyContext<'ast>,
        index: Option<&Identifier>,
        earlier_names: &[TypedName],
    ) -> bool {
        if !self.event(name.span) {
            return false;
        }
        let earlier = context
            .earlier_name(&name.text)
            .or_else(|| {
                index
                    .filter(|index| index.text == name.text)
                    .map(|index| (index.span, "the loop index is here"))
            })
            .or_else(|| {
                earlier_names
                    .iter()
                    .find(|earlier| earlier.name.text == name.text)
                    .map(|earlier| (earlier.name.span, "the first name is here"))
            });
        let Some((earlier_span, earlier_label)) = earlier else {
            return true;
        };
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
                    "each parameter, binding, loop index, and accumulator in scope has its \
                     own name; Orange has no shadowing",
                ),
            );
        }
        false
    }

    /// Decodes a loop's bounds and checks that they form a nonempty range
    /// within 0 through [`MAX_LOOP_BOUND`].
    fn check_loop_bounds(&mut self, r#loop: &LoopExpression) -> Option<(u32, u32)> {
        let start = self.loop_bound(&r#loop.start)?;
        let end = self.loop_bound(&r#loop.end)?;
        let limit = u64::from(MAX_LOOP_BOUND);
        let (span, message, label) = if start > limit || end > limit {
            let span = if start > limit {
                r#loop.start.span
            } else {
                r#loop.end.span
            };
            (
                span,
                format!("a loop bound must be at most {MAX_LOOP_BOUND}"),
                "loop bound too large",
            )
        } else if start >= end {
            (
                r#loop.end.span,
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

    /// Decodes one loop bound, an integer token or a size. A bound above
    /// the u64 range decodes as the u64 maximum, which is too large.
    fn loop_bound(&mut self, bound: &Size) -> Option<u64> {
        let span = bound.span;
        if let Some(expression) = bound.expression() {
            let value = self.size_value(expression)?;
            if value.is_negative() {
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::InvalidLoopRange,
                            "a loop bound must be at least 0",
                            span,
                        )
                        .with_label("negative loop bound in this instance")
                        .with_note(
                            "a loop `for i in a..b` runs once for each i from a up to b - 1, with \
                             0 <= a < b <= 65536",
                        ),
                    );
                }
                return None;
            }
            return Some(
                value
                    .to_i64()
                    .and_then(|value| u64::try_from(value).ok())
                    .unwrap_or(u64::MAX),
            );
        }
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

    #[cold]
    #[inline(never)]
    fn report_comparison_mismatch(&mut self, span: Span, expected: &CoreType) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("a comparison gives `Bool`, but `{expected}` is required here"),
                    span,
                )
                .with_label(format!("expected `{expected}`"))
                .with_note("a conditional `if c { a } else { b }` chooses a value by a `Bool`"),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_untyped_comparison(&mut self, span: Span, binary: &BinaryExpression) {
        if !self.begin_report(span) {
            return;
        }
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UntypedComparison,
            format!(
                "the operands of `{}` have no type of their own",
                binary.operator.as_str()
            ),
            span,
        );
        self.diagnostics.push(
            if passes_branch_binding(&binary.left) || passes_branch_binding(&binary.right) {
                diagnostic.with_label(BRANCH_BINDING_LEAF_LABEL).with_note(
                    "bind the conditional's value with a typed `let` first, or compare within \
                     each branch",
                )
            } else {
                diagnostic
                    .with_label("a literal takes its type from where it is used")
                    .with_note(
                        "compare with a typed operand, such as a name, or give the literal a \
                         type with a `let` binding",
                    )
            },
        );
    }

    /// Reports a comparison whose operands are both arrays, fills, or
    /// tuples written out, which take their types from where they are used.
    #[cold]
    #[inline(never)]
    fn report_aggregate_comparison(&mut self, binary: &BinaryExpression, span: Span) {
        // An order is refused whatever the literals' type would be.
        if !matches!(
            binary.operator,
            BinaryOperator::Equal | BinaryOperator::NotEqual
        ) {
            if self.begin_report(binary.operator_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::UnsupportedOperator,
                        format!(
                            "`{}` is not defined for arrays and tuples",
                            binary.operator.as_str()
                        ),
                        binary.operator_span,
                    )
                    .with_label("both operands are written out as arrays or tuples")
                    .with_note(
                        "arrays and tuples are compared whole with `==` and `!=`; they have no \
                         order, so compare elements",
                    ),
                );
            }
            return;
        }
        if !self.begin_report(span) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::UntypedComparison,
                format!(
                    "the operands of `{}` have no type of their own",
                    binary.operator.as_str()
                ),
                span,
            )
            .with_label("an array or tuple written out takes its type from where it is used")
            .with_note(
                "compare with a typed operand, such as a name, or give one side a type with a \
                 `let` binding",
            ),
        );
    }

    /// Checks a comparison `left op right` against `expected`, which must be
    /// `Bool`. Both operands have the type of the first typed leaf of the
    /// left operand, or else of the right operand, as a conversion operand
    /// has; an array, a fill, or a tuple written out as that leaf takes the
    /// right operand's type instead. `==` and `!=` compare values of every
    /// type, arrays and tuples whole; `<`, `<=`, `>`, and `>=` only ordered
    /// scalars.
    fn check_comparison(
        &mut self,
        expression: &'ast Expression,
        binary: &'ast BinaryExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let operator = binary.operator.as_str();
        let result_matches = *expected == CoreType::Bool;
        if !result_matches {
            self.report_comparison_mismatch(expression.span, expected);
        }
        let right_leaf = first_typed_leaf(&binary.right);
        let Some(leaf) = first_typed_leaf(&binary.left).or(right_leaf) else {
            self.report_untyped_comparison(expression.span, binary);
            return false;
        };
        let aggregate = |leaf: &Expression| {
            matches!(
                leaf.kind,
                ExpressionKind::Array(_) | ExpressionKind::Fill(_) | ExpressionKind::Tuple(_)
            )
        };
        let operand = match self.leaf_type(leaf, context, scope) {
            Some(operand) => operand,
            None if aggregate(leaf) => {
                let other = right_leaf
                    .filter(|other| !std::ptr::eq(*other, leaf))
                    .and_then(|other| Some((other, self.leaf_type(other, context, scope)?)));
                match other {
                    Some((_, operand)) => operand,
                    None => {
                        // A right operand whose own check reports why it has
                        // no type is reported there; two literals here.
                        match right_leaf.filter(|other| !aggregate(other)) {
                            Some(other) => {
                                self.check_untyped(other, expected, context, scope, output);
                            }
                            None => self.report_aggregate_comparison(binary, expression.span),
                        }
                        return false;
                    }
                }
            }
            None => {
                // The leaf's own check reports why it has no type.
                self.check_untyped(leaf, expected, context, scope, output);
                return false;
            }
        };
        let defined = match binary.operator {
            BinaryOperator::Equal | BinaryOperator::NotEqual => true,
            _ => operand.is_ordered(),
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
                    .with_note(if operand.modulus().is_some() {
                        "residues are compared with `==` and `!=`; they have no order, so \
                         compare least residues, such as `(x as Int) < (y as Int)`"
                    } else if operand.is_scalar() {
                        "`Bool` values are compared with `==` and `!=`; they have no order"
                    } else if operand.as_tuple().is_some() {
                        "tuples are compared whole with `==` and `!=`; they have no order, so \
                         compare elements, such as `p.0 < q.0`"
                    } else {
                        "arrays are compared whole with `==` and `!=`; they have no order, so \
                         compare elements, such as `x[0] < y[0]`"
                    }),
                );
            }
            return false;
        }
        let left = self.check_expression(&binary.left, &operand.clone(), context, scope, output);
        if self.halted {
            return false;
        }
        let right = self.check_expression(&binary.right, &operand.clone(), context, scope, output);
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
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let arm_count = conditional.arms.len();
        let mut arms: Vec<(u32, Span, Vec<CoreNode>, Vec<CoreBinding>)> = Vec::new();
        let mut else_branches: Vec<Vec<CoreNode>> = Vec::new();
        if arms.try_reserve_exact(arm_count).is_err()
            || else_branches.try_reserve_exact(arm_count).is_err()
        {
            self.resource_limit(expression.span, "conditional storage allocation failed");
            return false;
        }
        let mut well_formed = true;
        // The last `else` branch belongs to the last arm's conditional.
        let mut last = None;
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
            last = Some(id);
            let condition = if position == 0 {
                self.check_expression(&arm.condition, &CoreType::Bool, context, scope, output)
            } else {
                let mut branch = BodyOutput {
                    nodes: Vec::new(),
                    call_edges: &mut *output.call_edges,
                };
                let checked = self.check_expression(
                    &arm.condition,
                    &CoreType::Bool,
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
            let (bindings, value) = self.check_block(
                BlockOwner::Branch(id),
                &arm.bindings,
                &arm.value,
                expected,
                context,
                scope,
                &mut branch,
            );
            if self.halted {
                return false;
            }
            match bindings {
                Some(bindings) if condition && value => {
                    arms.push((id, arm.keyword_span, branch.nodes, bindings));
                }
                _ => well_formed = false,
            }
        }
        let Some(last) = last else {
            return false;
        };
        let mut branch = BodyOutput {
            nodes: Vec::new(),
            call_edges: &mut *output.call_edges,
        };
        let (bindings, otherwise) = self.check_block(
            BlockOwner::Branch(last),
            &conditional.otherwise_bindings,
            &conditional.otherwise,
            expected,
            context,
            scope,
            &mut branch,
        );
        let Some(else_bindings) = bindings.filter(|_| well_formed && otherwise) else {
            return false;
        };
        if self.halted {
            return false;
        }
        else_branches.push(branch.nodes);
        self.record_conditionals(
            expression,
            expected,
            arms,
            (else_branches, else_bindings),
            context,
            output,
        )
    }

    /// Fills the table entries of a well-formed conditional's arms and
    /// appends the first arm's `choose` node to `output`.
    fn record_conditionals(
        &mut self,
        expression: &Expression,
        expected: &CoreType,
        arms: Vec<(u32, Span, Vec<CoreNode>, Vec<CoreBinding>)>,
        (else_branches, else_bindings): (Vec<Vec<CoreNode>>, Vec<CoreBinding>),
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
            .map(|(id, keyword_span, ..)| {
                (
                    *id,
                    self.source
                        .span(keyword_span.start(), expression.span.end())
                        .unwrap_or(expression.span),
                )
            })
            .collect::<Vec<_>>();
        let mut later_arms = spans.iter().skip(1);
        // Only the last conditional's `else` branch is the written `else`
        // block; each earlier one holds the next arm's condition and choice.
        let mut last_else_bindings = Some(else_bindings);
        let count = arms.len();
        for (position, ((id, _, then_nodes, then_bindings), else_nodes)) in
            arms.into_iter().zip(else_branches).enumerate()
        {
            let mut else_branch = BodyOutput {
                nodes: else_nodes,
                call_edges: &mut *output.call_edges,
            };
            if let Some((next, next_span)) = later_arms.next()
                && !self.push_node(
                    &mut else_branch,
                    *next_span,
                    expected.clone(),
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
            let else_bindings = if position.saturating_add(1) == count {
                last_else_bindings.take().unwrap_or_default()
            } else {
                Vec::new()
            };
            *entry = Some(CoreConditional {
                span,
                ty: expected.clone(),
                visible_locals,
                scope: loop_scope,
                then_bindings,
                then_branch: CoreExpression { nodes: then_nodes },
                else_bindings,
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
            expected.clone(),
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
    fn report_scalar_for_array(&mut self, span: Span, what: &str, expected: &CoreType) {
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
            .with_note(if expected.as_tuple().is_some() {
                "a tuple value is written `(e0, e1, ...)`, one element per position"
            } else {
                "an array value is written `[e0, e1, ...]`, one element per index"
            }),
        );
    }

    fn check_call(
        &mut self,
        expression: &'ast Expression,
        call: &'ast CallExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let callee_name = &call.callee;
        let spelling = identifier_spelling_for_diagnostic(&callee_name.text);
        let Some((declarations, signatures)) = scope.tables_for(call) else {
            if let Some(module) = call.module() {
                self.report_module_not_used(module);
            }
            return false;
        };
        let declaration = first_declaration(declarations, FunctionKind::Spec, &callee_name.text);
        let signature = declaration.and_then(|entry| signatures.get(entry.source_index)?.as_ref());
        let Some(signature) = signature else {
            if self.begin_report(callee_name.span) {
                let place = call.module().map_or_else(
                    || String::from("this module"),
                    |module| {
                        format!(
                            "module `{}`",
                            identifier_spelling_for_diagnostic(&module.text)
                        )
                    },
                );
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
                        format!("no typed `spec` function named `{spelling}` in {place}"),
                        callee_name.span,
                    )
                    .with_label("unknown function")
                };
                let imported_by = (call.module().is_none() && declaration.is_none())
                    .then(|| {
                        scope.imports.iter().find(|import| {
                            first_declaration(
                                import.declarations,
                                FunctionKind::Spec,
                                &callee_name.text,
                            )
                            .is_some()
                        })
                    })
                    .flatten();
                diagnostic =
                    if first_declaration(declarations, FunctionKind::Impl, &callee_name.text)
                        .is_some()
                    {
                        diagnostic.with_note(
                            "`impl` functions have no semantics yet and cannot be called",
                        )
                    } else if let Some(import) = imported_by {
                        let module = identifier_spelling_for_diagnostic(import.name);
                        diagnostic.with_note(format!(
                            "the used module `{module}` declares `{spelling}`; call it as \
                         `{module}::{spelling}(...)`"
                        ))
                    } else if call.module().is_some() {
                        diagnostic
                            .with_note("a qualified call names a typed `spec` of the used module")
                    } else if scope.imports.is_empty() {
                        diagnostic
                            .with_note("calls name a typed `spec` declared in the same module")
                    } else {
                        diagnostic.with_note(
                        "calls name a typed `spec` declared in the same module, or one of a used \
                         module as `NAME::f(...)`",
                    )
                    };
                self.diagnostics.push(diagnostic);
            }
            return false;
        };
        // The instance called: its sizes are computed in the caller's
        // instance and must lie in the callee's ranges.
        let Some((id, signature)) =
            self.called_instance(expression, call, signature, expected, context, scope)
        else {
            return false;
        };
        // An unresolved callee type was reported at the callee's declaration.
        if !signature.is_complete() {
            self.record_call_edge(context, id, expression.span, output);
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
            self.record_call_edge(context, id, expression.span, output);
            return false;
        }
        let Some(result_type) = signature.result_type.clone() else {
            return false;
        };
        // A result-type mismatch is reported at the call before its arguments
        // are checked; arguments are checked against the callee's parameter
        // types, so their errors are independent and are still reported.
        let result_matches = result_type == *expected;
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
            let Some(parameter_type) = parameter_type.clone() else {
                return false;
            };
            arguments_checked &=
                self.check_expression(argument, &parameter_type, context, scope, output);
            if self.halted {
                return false;
            }
        }
        if !self.record_call_edge(context, id, expression.span, output)
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
            expected.clone(),
            CoreNodeKind::Call {
                function: id,
                arguments,
            },
        )
    }

    /// Reports `NAME::f(...)` where `NAME` is not a module this module uses.
    fn report_module_not_used(&mut self, module: &Identifier) {
        if !self.begin_report(module.span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(&module.text);
        let own = &self.ast.module.name;
        let diagnostic = if module.text == own.text {
            Diagnostic::error(
                DiagnosticCode::ModuleNotUsed,
                format!("`{spelling}` is the calling module"),
                module.span,
            )
            .with_label("a module does not qualify calls to itself")
            .with_note("call a function of the same module without a module name, as in `f(x)`")
        } else {
            Diagnostic::error(
                DiagnosticCode::ModuleNotUsed,
                format!(
                    "module `{spelling}` is not used by `{}`",
                    identifier_spelling_for_diagnostic(&own.text)
                ),
                module.span,
            )
            .with_label("no `use` declaration names this module")
            .with_note(format!(
                "declare `use {spelling};` at the head of the module to call its functions"
            ))
        };
        self.diagnostics.push(diagnostic);
    }

    /// Adds the call graph edge for an examined call to a typed `spec`. The
    /// edge does not depend on the call's types, so a cycle is reported even
    /// through a call that is also wrong. A call into a used module adds no
    /// edge: the module graph is acyclic, so no cycle passes through it.
    /// Returns whether the edge was stored or was not needed.
    fn record_call_edge(
        &mut self,
        context: &BodyContext<'ast>,
        callee: CoreFunctionId,
        span: Span,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        if usize::try_from(callee.index()).is_ok_and(|index| index < self.id_offset) {
            return true;
        }
        if !(self.reserve_call_edge_slot)(output.call_edges) {
            self.resource_limit(span, "call graph storage allocation failed");
            return false;
        }
        output.call_edges.push(CallEdge {
            caller: context.id,
            callee,
            span,
        });
        true
    }

    fn unary_is_defined(&mut self, unary: &UnaryExpression, expected: &CoreType) -> bool {
        let defined = match unary.operator {
            UnaryOperator::Negate => *expected == CoreType::Int || expected.modulus().is_some(),
            UnaryOperator::Complement => expected.word_bits().is_some(),
            UnaryOperator::Not => *expected == CoreType::Bool,
        };
        if !defined && self.begin_report(unary.operator_span) {
            let operator = unary.operator.as_str();
            let note = match (unary.operator, expected.word_bits()) {
                _ if !expected.is_scalar() => String::from(aggregate_operator_note(expected)),
                _ if *expected == CoreType::Bool => String::from(BOOL_OPERATOR_NOTE),
                (UnaryOperator::Negate, Some(bits)) => {
                    format!("write `0 - x` for negation modulo 2^{bits}")
                }
                (UnaryOperator::Not, Some(_)) => {
                    String::from("`!` negates a `Bool`; `~` is the bitwise complement of a word")
                }
                (UnaryOperator::Not, None) => {
                    String::from("`!` negates a `Bool`; `-` negates an `Int` or a residue")
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

    fn binary_is_defined(&mut self, binary: &BinaryExpression, expected: &CoreType) -> bool {
        let defined = match binary.operator {
            BinaryOperator::Add
            | BinaryOperator::Subtract
            | BinaryOperator::Multiply
            | BinaryOperator::Divide => expected.is_number(),
            BinaryOperator::Remainder => expected.is_ordered(),
            BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Xor
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::RotateLeft
            | BinaryOperator::RotateRight => expected.word_bits().is_some(),
            BinaryOperator::LogicalAnd | BinaryOperator::LogicalOr => *expected == CoreType::Bool,
            // `++` is checked by `check_concatenation`.
            BinaryOperator::Concat => false,
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
                    aggregate_operator_note(expected)
                } else if *expected == CoreType::Bool {
                    BOOL_OPERATOR_NOTE
                } else if binary.operator == BinaryOperator::Remainder {
                    "a residue is already reduced; `%` applies to `Int` and word values, such \
                     as `(x as Int) % 16`"
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

    /// Checks the amount of a shift or rotation that is not one integer
    /// literal and returns its type. Its type is its first typed leaf's
    /// when that is a word, and `Int` otherwise, as an index's is; every
    /// value of that type is an amount, so no range is examined.
    fn check_computed_amount(
        &mut self,
        binary: &'ast BinaryExpression,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> Option<CoreType> {
        let amount = first_typed_leaf(&binary.right)
            .and_then(|leaf| self.leaf_type(leaf, context, scope))
            .filter(|ty| ty.word_bits().is_some())
            .unwrap_or(CoreType::Int);
        self.check_expression(&binary.right, &amount, context, scope, output)
            .then_some(amount)
    }

    fn check_shift_amount(
        &mut self,
        binary: &BinaryExpression,
        expected: &CoreType,
    ) -> Option<u32> {
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
                .with_label(format!("a literal amount is from 0 through {highest}"))
                .with_note(SHIFT_AMOUNT_NOTE),
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
            // node, and its step's nodes; each binding of a step or branch
            // contributes one binding node and one type node, its value's
            // nodes being the step's or branch's.
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
                        .saturating_add(r#loop.bindings.len().saturating_mul(2))
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
                            .saturating_add(
                                conditional
                                    .then_bindings
                                    .len()
                                    .saturating_add(conditional.else_bindings.len())
                                    .saturating_mul(2),
                            )
                    });
            for _ in 0..node_count {
                if !self.record_core_node(pending_function.span) {
                    return None;
                }
            }
            let Some(id) = self
                .id_offset
                .checked_add(functions.len())
                .and_then(CoreFunctionId::from_index)
            else {
                self.resource_limit(
                    pending_function.span,
                    "Core function identity exceeds the u32 representation limit",
                );
                return None;
            };
            let module = self.copy_core_name(&module_name, self.ast.module.name.span)?;
            functions.push(CoreFunction {
                id,
                module,
                span: pending_function.span,
                name: pending_function.name,
                name_span: pending_function.name_span,
                sizes: pending_function.sizes,
                instance: pending_function.instance,
                parameters: pending_function.parameters,
                result_type: pending_function.result_type,
                locals: pending_function.locals,
                body: CoreExpression {
                    nodes: pending_function.nodes,
                },
                loops: pending_function.loops,
                conditionals: pending_function.conditionals,
                title: pending_function.title,
            });
        }
        // Tests follow the functions, so they are the last of them.
        let tests = functions
            .iter()
            .filter(|function| function.title.is_some())
            .count();
        Some(CoreModule {
            span: self.ast.module.span,
            name: module_name,
            functions,
            entry: 0,
            tests,
        })
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
mod tests;
