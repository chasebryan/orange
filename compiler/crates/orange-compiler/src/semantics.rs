//! Bounded name resolution, type checking, and Core construction.

use std::cmp::Ordering;
use std::fmt;

use crate::core::{
    CoreExpression, CoreFunction, CoreFunctionId, CoreModule, CoreNode, CoreNodeKind, CoreType,
    CoreValue, ExactInteger, MAX_EXACT_INTEGER_BITS, Magnitude,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::{
    BinaryExpression, BinaryOperator, CallExpression, Expression, ExpressionKind, FunctionBody,
    FunctionDeclaration, FunctionKind, Identifier, IntegerLiteral, Parameter, SyntaxTree,
    TypeSyntax, TypedBody, UnaryExpression, UnaryOperator,
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

const MAX_IDENTIFIER_BYTES_IN_DIAGNOSTIC: usize = 64;
const MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC: usize = 8;
const ADMITTED_TYPES: &str = "`Int`, `Word[8]`, `Word[16]`, `Word[32]`, and `Word[64]`";

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
/// acyclic.
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
        belongs(ty.span) && belongs(ty.name.span) && ty.width_span.is_none_or(belongs)
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
    nodes: Vec<CoreNode>,
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

/// One checked call from a typed `spec` to a typed `spec`.
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
    Unsupported,
}

fn classify_type(source: &SourceFile, syntax: &TypeSyntax) -> TypeClass {
    match (syntax.name.text.as_str(), syntax.width_span) {
        ("Int", None) => TypeClass::Resolved(CoreType::Int),
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

fn word_maximum(ty: CoreType) -> Option<u64> {
    match ty {
        CoreType::Int => None,
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
        let mut output = BodyOutput {
            nodes: Vec::new(),
            call_edges,
        };
        let checked =
            self.check_expression(&body.expression, result_type, &context, scope, &mut output);
        if !checked || self.halted {
            return None;
        }
        let nodes = output.nodes;
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
            nodes,
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
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        if self.halted {
            return false;
        }
        match &expression.kind {
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
                self.check_parameter_reference(name, expected, context, scope, output)
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
            ExpressionKind::Binary(binary) => {
                if !self.event(binary.operator_span) {
                    return false;
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

    fn check_parameter_reference(
        &mut self,
        name: &'ast Identifier,
        expected: CoreType,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let found = context
            .parameters
            .iter()
            .position(|parameter| parameter.name.text == name.text);
        let Some(index) = found else {
            if self.begin_report(name.span) {
                let spelling = identifier_spelling_for_diagnostic(&name.text);
                let function = identifier_spelling_for_diagnostic(&context.name.text);
                let mut diagnostic = Diagnostic::error(
                    DiagnosticCode::UnknownParameter,
                    format!("`{spelling}` is not a parameter of `{function}`"),
                    name.span,
                )
                .with_label("unknown name");
                diagnostic =
                    if first_declaration(scope.declarations, FunctionKind::Spec, &name.text)
                        .is_some()
                    {
                        diagnostic.with_note(format!(
                        "to call the function `{spelling}`, write `{spelling}()` with its arguments"
                    ))
                    } else {
                        diagnostic.with_note(
                            "a bare name in a `spec` body refers to one of its parameters",
                        )
                    };
                self.diagnostics.push(diagnostic);
            }
            return false;
        };
        // An unresolved parameter type was reported at its declaration.
        let Some(actual) = context.parameter_types.get(index).copied().flatten() else {
            return false;
        };
        if actual != expected {
            if self.begin_report(name.span) {
                let spelling = identifier_spelling_for_diagnostic(&name.text);
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!(
                            "`{spelling}` has type `{actual}`, but `{expected}` is required here"
                        ),
                        name.span,
                    )
                    .with_label(format!("expected `{expected}`"))
                    .with_note("Orange has no implicit conversions between types"),
                );
            }
            return false;
        }
        let Ok(index) = u32::try_from(index) else {
            self.resource_limit(
                name.span,
                "parameter index exceeds the u32 representation limit",
            );
            return false;
        };
        self.push_node(output, name.span, expected, CoreNodeKind::Parameter(index))
    }

    fn check_call(
        &mut self,
        expression: &'ast Expression,
        call: &'ast CallExpression,
        expected: CoreType,
        context: &BodyContext<'ast>,
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
            return false;
        }
        let Some(result_type) = signature.result_type else {
            return false;
        };
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
        if result_type != expected {
            if self.begin_report(expression.span) {
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
            return false;
        }
        if !arguments_checked {
            return false;
        }
        if !(self.reserve_call_edge_slot)(output.call_edges) {
            self.resource_limit(expression.span, "call graph storage allocation failed");
            return false;
        }
        output.call_edges.push(CallEdge {
            caller: context.id,
            callee: signature.id,
            span: expression.span,
        });
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

    fn unary_is_defined(&mut self, unary: &UnaryExpression, expected: CoreType) -> bool {
        let defined = match unary.operator {
            UnaryOperator::Negate => expected == CoreType::Int,
            UnaryOperator::Complement => expected != CoreType::Int,
        };
        if !defined && self.begin_report(unary.operator_span) {
            let operator = unary.operator.as_str();
            let note = match (unary.operator, expected.word_bits()) {
                (UnaryOperator::Negate, Some(bits)) => {
                    format!("write `0 - x` for negation modulo 2^{bits}")
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
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply => true,
            BinaryOperator::And
            | BinaryOperator::Or
            | BinaryOperator::Xor
            | BinaryOperator::ShiftLeft
            | BinaryOperator::ShiftRight
            | BinaryOperator::RotateLeft
            | BinaryOperator::RotateRight => expected != CoreType::Int,
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
                .with_note(if binary.operator.is_shift_or_rotation() {
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
            // node, one node per parameter type, and its expression nodes.
            let node_count = pending_function
                .parameters
                .len()
                .saturating_add(pending_function.nodes.len())
                .saturating_add(2);
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
                body: CoreExpression {
                    nodes: pending_function.nodes,
                },
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
        match classify_type(self.source, syntax) {
            TypeClass::Resolved(ty) => Some(ty),
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
}
