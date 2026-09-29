//! Bounded, deterministic parsing for the minimal Orange 2026 source grammar.

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::edition::Edition;
use crate::lexer::{Lexed, MAX_TOKENS_PER_SOURCE, Token, TokenKind};
use crate::source::{SourceFile, Span};

/// Maximum ordinary syntax errors retained before one suppression diagnostic.
pub const MAX_PARSE_DIAGNOSTICS_PER_SOURCE: usize = 100;
const MAX_RETAINED_PARSE_DIAGNOSTICS: usize = MAX_PARSE_DIAGNOSTICS_PER_SOURCE.saturating_add(2);

/// Maximum AST nodes constructed for one source.
pub const MAX_SYNTAX_NODES_PER_SOURCE: usize = 262_144;

/// Maximum parser events (token advances, diagnostics, and node constructions).
pub const MAX_PARSE_EVENTS_PER_SOURCE: usize = 1_048_576;

/// Maximum delimiter nesting inspected while recovering from malformed syntax.
pub const MAX_RECOVERY_DELIMITER_DEPTH: usize = 64;

/// Maximum groups, call argument lists, and prefix operators enclosing any
/// one subexpression.
///
/// This bounds the parser's recursion. Like C's minimum of 63 nested
/// parenthesized expressions, it is far beyond what readable source needs.
pub const MAX_EXPRESSION_NESTING: usize = 64;

/// Maximum height of one expression tree, counting every operator, group,
/// and call as one level above its tallest operand.
///
/// Long operator chains grow a tree's height without nesting, so this bounds
/// every later traversal separately from [`MAX_EXPRESSION_NESTING`].
pub const MAX_EXPRESSION_HEIGHT: usize = 256;

/// Maximum parameters declared by one function.
pub const MAX_PARAMETERS_PER_FUNCTION: usize = 64;

/// Maximum arguments supplied by one call.
pub const MAX_ARGUMENTS_PER_CALL: usize = 256;

/// Maximum `let` bindings in one typed body.
pub const MAX_BINDINGS_PER_BODY: usize = 256;

/// A complete minimal Orange source file.
///
/// Parsed nodes are read-only outside this crate so later stages can rely on
/// parser-established source ownership, spans, and ordering.
///
/// ```compile_fail
/// use orange_compiler::{ModuleDeclaration, SyntaxTree};
///
/// fn replace_module(tree: &mut SyntaxTree, module: ModuleDeclaration) {
///     tree.module = module;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxTree {
    /// Extent from the edition keyword through the module's closing brace.
    pub(crate) span: Span,
    /// The mandatory source-edition declaration.
    pub(crate) edition: EditionDeclaration,
    /// The source's single module.
    pub(crate) module: ModuleDeclaration,
}

impl SyntaxTree {
    /// Returns the extent from the edition keyword through the module's closing brace.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the mandatory source-edition declaration.
    #[must_use]
    pub const fn edition(&self) -> &EditionDeclaration {
        &self.edition
    }

    /// Returns the source's single module.
    #[must_use]
    pub const fn module(&self) -> &ModuleDeclaration {
        &self.module
    }
}

/// The mandatory `edition 2026;` declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditionDeclaration {
    /// Full declaration extent.
    pub(crate) span: Span,
    /// Edition selected by the declaration.
    pub(crate) edition: Edition,
    /// Exact span of the `2026` spelling.
    pub(crate) value_span: Span,
}

impl EditionDeclaration {
    /// Returns the full declaration extent.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the edition selected by the declaration.
    #[must_use]
    pub const fn edition(&self) -> Edition {
        self.edition
    }

    /// Returns the exact span of the `2026` spelling.
    #[must_use]
    pub const fn value_span(&self) -> Span {
        self.value_span
    }
}

/// A single `module NAME { ... }` declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDeclaration {
    /// Full module extent.
    pub(crate) span: Span,
    /// Module name.
    pub(crate) name: Identifier,
    /// Functions in source order.
    pub(crate) functions: Vec<FunctionDeclaration>,
}

impl ModuleDeclaration {
    /// Returns the full module extent.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the module name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns functions in source order.
    #[must_use]
    pub fn functions(&self) -> &[FunctionDeclaration] {
        &self.functions
    }
}

/// A function declaration in the current Orange 2026 grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDeclaration {
    /// Full function extent.
    pub(crate) span: Span,
    /// Whether this is a `spec` or `impl` declaration.
    pub(crate) kind: FunctionKind,
    /// Function name.
    pub(crate) name: Identifier,
    /// Parameters in source order; nonempty only for a typed `spec`.
    pub(crate) parameters: Vec<Parameter>,
    /// Empty legacy syntax or the typed body available to `spec`.
    pub(crate) body: FunctionBody,
}

impl FunctionDeclaration {
    /// Returns the full function extent.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns whether this is a `spec` or `impl` declaration.
    #[must_use]
    pub const fn kind(&self) -> FunctionKind {
        self.kind
    }

    /// Returns the function name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns parameters in source order.
    #[must_use]
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }

    /// Returns the empty legacy syntax or typed body.
    #[must_use]
    pub const fn body(&self) -> &FunctionBody {
        &self.body
    }
}

/// One `name: Type` parameter of a typed `spec` function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    /// Extent from the parameter name through its type.
    pub(crate) span: Span,
    /// Parameter name.
    pub(crate) name: Identifier,
    /// Syntactic parameter type; semantic analysis resolves its meaning.
    pub(crate) ty: TypeSyntax,
}

impl Parameter {
    /// Returns the extent from the parameter name through its type.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the parameter name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the syntactic parameter type.
    #[must_use]
    pub const fn ty(&self) -> &TypeSyntax {
        &self.ty
    }
}

macro_rules! define_function_kinds {
    ($($(#[$variant_doc:meta])* $variant:ident => $spelling:literal,)+) => {
        /// The function declaration categories admitted by the minimal grammar.
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum FunctionKind {
            $($(#[$variant_doc])* $variant,)+
        }

        impl FunctionKind {
            /// Returns the stable source-language spelling of this declaration kind.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $spelling,)+
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }
    };
}

define_function_kinds! {
    /// A `spec` declaration.
    Spec => "spec",
    /// An `impl` declaration.
    Impl => "impl",
}

/// The syntactic body forms admitted for a function declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FunctionBody {
    /// The legacy `{}` form, which remains syntax-only.
    Empty,
    /// A result type and one body expression, admitted only for `spec`.
    Typed(Box<TypedBody>),
}

/// The complete typed tail of a `spec`, from `->` through `}`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedBody {
    /// Full extent from the `->` token through the body's closing brace.
    pub(crate) span: Span,
    /// Syntactic result type; semantic analysis resolves its meaning.
    pub(crate) result_type: TypeSyntax,
    /// `let` bindings in source order, before the result expression.
    pub(crate) bindings: Vec<Binding>,
    /// The expression that gives the function's value.
    pub(crate) expression: Expression,
}

impl TypedBody {
    /// Returns the full extent from `->` through the body's closing brace.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the syntactic result type.
    #[must_use]
    pub const fn result_type(&self) -> &TypeSyntax {
        &self.result_type
    }

    /// Returns the `let` bindings in source order.
    #[must_use]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// Returns the expression that gives the function's value.
    #[must_use]
    pub const fn expression(&self) -> &Expression {
        &self.expression
    }
}

/// One `let name: Type = expression;` binding in a typed body.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    /// Full extent from `let` through the closing `;`.
    pub(crate) span: Span,
    /// Bound name.
    pub(crate) name: Identifier,
    /// Syntactic declared type; semantic analysis resolves its meaning.
    pub(crate) ty: TypeSyntax,
    /// The bound expression.
    pub(crate) value: Expression,
}

impl Binding {
    /// Returns the full extent from `let` through the closing `;`.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the bound name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the syntactic declared type.
    #[must_use]
    pub const fn ty(&self) -> &TypeSyntax {
        &self.ty
    }

    /// Returns the bound expression.
    #[must_use]
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

/// One syntactic expression and its exact source extent.
///
/// Expression trees are at most [`MAX_EXPRESSION_HEIGHT`] levels high, so
/// every later traversal is bounded by the parser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    /// Full expression extent, including any grouping parentheses.
    pub(crate) span: Span,
    /// Expression form.
    pub(crate) kind: ExpressionKind,
}

impl Expression {
    /// Returns the full expression extent, including grouping parentheses.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the expression form.
    #[must_use]
    pub const fn kind(&self) -> &ExpressionKind {
        &self.kind
    }
}

/// The expression forms of the Orange 2026 grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExpressionKind {
    /// An integer literal, optionally with a sign written directly before it.
    Literal(IntegerLiteral),
    /// A bare identifier, which names a parameter or a binding.
    Name(Identifier),
    /// A call of a named function.
    Call(CallExpression),
    /// A prefix operator and its operand.
    Unary(UnaryExpression),
    /// An infix operator and its two operands.
    Binary(BinaryExpression),
    /// An expression enclosed in grouping parentheses.
    Parenthesized(Box<Expression>),
    /// An explicit conversion `operand as Type`, boxed so that it does not
    /// enlarge every expression.
    Conversion(Box<ConversionExpression>),
}

/// An explicit conversion `operand as Type`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionExpression {
    /// The converted operand.
    pub(crate) operand: Expression,
    /// Exact extent of the `as` keyword.
    pub(crate) keyword_span: Span,
    /// Syntactic target type; semantic analysis resolves its meaning.
    pub(crate) target: TypeSyntax,
}

impl ConversionExpression {
    /// Returns the converted operand.
    #[must_use]
    pub fn operand(&self) -> &Expression {
        &self.operand
    }

    /// Returns the exact extent of the `as` keyword.
    #[must_use]
    pub const fn keyword_span(&self) -> Span {
        self.keyword_span
    }

    /// Returns the syntactic target type.
    #[must_use]
    pub const fn target(&self) -> &TypeSyntax {
        &self.target
    }
}

/// A call `name(arguments)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallExpression {
    /// Called function name.
    pub(crate) callee: Identifier,
    /// Arguments in source order.
    pub(crate) arguments: Vec<Expression>,
}

impl CallExpression {
    /// Returns the called function name.
    #[must_use]
    pub const fn callee(&self) -> &Identifier {
        &self.callee
    }

    /// Returns arguments in source order.
    #[must_use]
    pub fn arguments(&self) -> &[Expression] {
        &self.arguments
    }
}

/// A prefix operator applied to one operand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnaryExpression {
    /// The operator.
    pub(crate) operator: UnaryOperator,
    /// Exact extent of the operator token.
    pub(crate) operator_span: Span,
    /// The operand.
    pub(crate) operand: Box<Expression>,
}

impl UnaryExpression {
    /// Returns the operator.
    #[must_use]
    pub const fn operator(&self) -> UnaryOperator {
        self.operator
    }

    /// Returns the exact extent of the operator token.
    #[must_use]
    pub const fn operator_span(&self) -> Span {
        self.operator_span
    }

    /// Returns the operand.
    #[must_use]
    pub fn operand(&self) -> &Expression {
        &self.operand
    }
}

/// An infix operator applied to two operands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BinaryExpression {
    /// The operator.
    pub(crate) operator: BinaryOperator,
    /// Exact extent of the operator token.
    pub(crate) operator_span: Span,
    /// The left operand.
    pub(crate) left: Box<Expression>,
    /// The right operand, or the amount of a shift or rotation.
    pub(crate) right: Box<Expression>,
}

impl BinaryExpression {
    /// Returns the operator.
    #[must_use]
    pub const fn operator(&self) -> BinaryOperator {
        self.operator
    }

    /// Returns the exact extent of the operator token.
    #[must_use]
    pub const fn operator_span(&self) -> Span {
        self.operator_span
    }

    /// Returns the left operand.
    #[must_use]
    pub fn left(&self) -> &Expression {
        &self.left
    }

    /// Returns the right operand, or the amount of a shift or rotation.
    #[must_use]
    pub fn right(&self) -> &Expression {
        &self.right
    }
}

macro_rules! define_operators {
    (
        $(#[$enum_doc:meta])* $name:ident {
            $($(#[$variant_doc:meta])* $variant:ident => $spelling:literal,)+
        }
    ) => {
        $(#[$enum_doc])*
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum $name {
            $($(#[$variant_doc])* $variant,)+
        }

        impl $name {
            /// Returns the exact source spelling of this operator.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $spelling,)+
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }
    };
}

define_operators! {
    /// A prefix operator.
    UnaryOperator {
        /// `-`: exact negation.
        Negate => "-",
        /// `~`: bitwise complement.
        Complement => "~",
    }
}

define_operators! {
    /// An infix operator.
    BinaryOperator {
        /// `+`: addition.
        Add => "+",
        /// `-`: subtraction.
        Subtract => "-",
        /// `*`: multiplication.
        Multiply => "*",
        /// `&`: bitwise and.
        And => "&",
        /// `|`: bitwise inclusive or.
        Or => "|",
        /// `^`: bitwise exclusive or.
        Xor => "^",
        /// `<<`: logical shift left.
        ShiftLeft => "<<",
        /// `>>`: logical shift right.
        ShiftRight => ">>",
        /// `<<<`: rotation left.
        RotateLeft => "<<<",
        /// `>>>`: rotation right.
        RotateRight => ">>>",
    }
}

impl BinaryOperator {
    /// Returns whether this operator is a shift or rotation, whose right
    /// operand is an amount rather than a value.
    #[must_use]
    pub const fn is_shift_or_rotation(self) -> bool {
        matches!(
            self,
            Self::ShiftLeft | Self::ShiftRight | Self::RotateLeft | Self::RotateRight
        )
    }

    const fn token_kind(self) -> TokenKind {
        match self {
            Self::Add => TokenKind::Plus,
            Self::Subtract => TokenKind::Minus,
            Self::Multiply => TokenKind::Star,
            Self::And => TokenKind::Ampersand,
            Self::Or => TokenKind::Pipe,
            Self::Xor => TokenKind::Caret,
            Self::ShiftLeft => TokenKind::LessLess,
            Self::ShiftRight => TokenKind::GreaterGreater,
            Self::RotateLeft => TokenKind::LessLessLess,
            Self::RotateRight => TokenKind::GreaterGreaterGreater,
        }
    }

    const fn from_token(kind: TokenKind) -> Option<Self> {
        Some(match kind {
            TokenKind::Plus => Self::Add,
            TokenKind::Minus => Self::Subtract,
            TokenKind::Star => Self::Multiply,
            TokenKind::Ampersand => Self::And,
            TokenKind::Pipe => Self::Or,
            TokenKind::Caret => Self::Xor,
            TokenKind::LessLess => Self::ShiftLeft,
            TokenKind::GreaterGreater => Self::ShiftRight,
            TokenKind::LessLessLess => Self::RotateLeft,
            TokenKind::GreaterGreaterGreater => Self::RotateRight,
            _ => return None,
        })
    }
}

/// A syntactic type name with an optional integer width argument.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeSyntax {
    /// Full type extent, including `[WIDTH]` when present.
    pub(crate) span: Span,
    /// Exact type-name spelling and span.
    pub(crate) name: Identifier,
    /// Exact span of the width integer, excluding brackets.
    pub(crate) width_span: Option<Span>,
}

impl TypeSyntax {
    /// Returns the full type extent, including `[WIDTH]` when present.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact type-name spelling and span.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the exact span of the width integer, excluding brackets.
    #[must_use]
    pub const fn width_span(&self) -> Option<Span> {
        self.width_span
    }
}

/// A signed integer literal syntax node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntegerLiteral {
    /// Full extent, including a leading `-` when present.
    pub(crate) span: Span,
    /// Exact extent of the magnitude's integer token.
    pub(crate) magnitude_span: Span,
    /// Whether the literal has a leading `-` token.
    pub(crate) negative: bool,
}

impl IntegerLiteral {
    /// Returns the full extent, including a leading `-` when present.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact extent of the magnitude's integer token.
    #[must_use]
    pub const fn magnitude_span(&self) -> Span {
        self.magnitude_span
    }

    /// Returns whether the literal has a leading `-` token.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.negative
    }
}

/// An owned ASCII identifier and its source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    /// Original identifier spelling.
    pub(crate) text: String,
    /// Exact spelling extent.
    pub(crate) span: Span,
}

impl Identifier {
    /// Returns the original identifier spelling.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the exact spelling extent.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

/// The complete result of parsing one token stream.
///
/// ```compile_fail
/// use orange_compiler::ParseResult;
///
/// fn replace_ast(result: &mut ParseResult) {
///     result.ast = None;
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseResult {
    /// Complete AST, present only when parsing produced zero diagnostics.
    ast: Option<SyntaxTree>,
    /// Syntax and parser-resource diagnostics in deterministic source order.
    diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    /// Returns the complete AST, or `None` after parsing failure or skipped parsing.
    #[must_use]
    pub const fn ast(&self) -> Option<&SyntaxTree> {
        self.ast.as_ref()
    }

    /// Returns syntax and parser-resource diagnostics in deterministic source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Consumes this result and returns its complete AST, if one was produced.
    #[must_use]
    pub fn into_ast(self) -> Option<SyntaxTree> {
        self.ast
    }

    /// Returns whether parsing did not produce a complete AST.
    ///
    /// This is also true when parsing was skipped because lexing failed.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.ast.is_none()
    }
}

/// Parses the minimal Orange 2026 grammar from a complete lexer result.
///
/// The grammar is intentionally closed: one edition declaration, one module,
/// and zero or more parameterless functions. Legacy `spec` and `impl`
/// declarations retain empty bodies; `spec` additionally admits a syntactic
/// result type and one signed integer literal.
/// Lexically invalid sources are not parsed and therefore cannot produce an
/// AST or cascading parser diagnostics. Lexer results owned by another source
/// are rejected before this lexical-error shortcut is considered.
#[must_use]
pub fn parse(source: &SourceFile, lexed: &Lexed) -> ParseResult {
    if !lexed_is_owned_by(source, lexed) {
        return invalid_parser_input(source, |diagnostics| {
            diagnostics.try_reserve_exact(1).is_ok()
        });
    }
    if lexed.has_errors() {
        return ParseResult {
            ast: None,
            diagnostics: Vec::new(),
        };
    }
    Parser::new(source, lexed.tokens(), Limits::DEFAULT).run()
}

fn invalid_parser_input(
    source: &SourceFile,
    reserve_diagnostic: impl FnOnce(&mut Vec<Diagnostic>) -> bool,
) -> ParseResult {
    let mut diagnostics = Vec::new();
    if reserve_diagnostic(&mut diagnostics) {
        diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::InvalidParserInput,
                "parser received lexer output owned by another source",
                source.lexer_span(0, 0),
            )
            .with_label("parsing stopped at this source boundary")
            .with_note("lex and parse each token stream with the same source file"),
        );
    }
    ParseResult {
        ast: None,
        diagnostics,
    }
}

fn lexed_is_owned_by(source: &SourceFile, lexed: &Lexed) -> bool {
    !lexed.tokens().is_empty()
        && lexed
            .tokens()
            .iter()
            .all(|token| token.span.source() == source.id())
        && lexed.diagnostics().iter().all(|diagnostic| {
            diagnostic.primary_span().source() == source.id()
                && diagnostic
                    .secondary_spans()
                    .iter()
                    .all(|secondary| secondary.span().source() == source.id())
        })
}

#[derive(Clone, Copy)]
struct Limits {
    diagnostics: usize,
    nodes: usize,
    events: usize,
    recovery_depth: usize,
}

impl Limits {
    const DEFAULT: Self = Self {
        diagnostics: MAX_PARSE_DIAGNOSTICS_PER_SOURCE,
        nodes: MAX_SYNTAX_NODES_PER_SOURCE,
        events: MAX_PARSE_EVENTS_PER_SOURCE,
        recovery_depth: MAX_RECOVERY_DELIMITER_DEPTH,
    };
}

const BODY_SHAPE_NOTE: &str =
    "a typed `spec` body holds `let` bindings, if any, and then one result expression";

/// Something that continues an expression after an operand: a binary
/// operator or the conversion keyword `as`.
#[derive(Clone, Copy)]
enum Joiner {
    Binary(BinaryOperator),
    As,
}

impl Joiner {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Binary(operator) => operator.as_str(),
            Self::As => "as",
        }
    }
}

struct Parser<'source, 'tokens> {
    source: &'source SourceFile,
    tokens: &'tokens [Token],
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
    ordinary_diagnostics: usize,
    diagnostic_limit_reported: bool,
    resource_limit_reported: bool,
    nodes: usize,
    events: usize,
    halted: bool,
    limits: Limits,
    reserve_function_slot: fn(&mut Vec<FunctionDeclaration>) -> bool,
    reserve_parameter_slot: fn(&mut Vec<Parameter>) -> bool,
    reserve_argument_slot: fn(&mut Vec<Expression>) -> bool,
    reserve_binding_slot: fn(&mut Vec<Binding>) -> bool,
    reserve_identifier_text: fn(&mut String, usize) -> bool,
    reserve_diagnostic_slots: fn(&mut Vec<Diagnostic>, usize) -> bool,
}

fn reserve_function_slot(functions: &mut Vec<FunctionDeclaration>) -> bool {
    functions.try_reserve(1).is_ok()
}

fn reserve_parameter_slot(parameters: &mut Vec<Parameter>) -> bool {
    parameters.try_reserve(1).is_ok()
}

fn reserve_argument_slot(arguments: &mut Vec<Expression>) -> bool {
    arguments.try_reserve(1).is_ok()
}

fn reserve_binding_slot(bindings: &mut Vec<Binding>) -> bool {
    bindings.try_reserve(1).is_ok()
}

fn reserve_identifier_text(text: &mut String, bytes: usize) -> bool {
    text.try_reserve_exact(bytes).is_ok()
}

fn reserve_diagnostic_slots(diagnostics: &mut Vec<Diagnostic>, capacity: usize) -> bool {
    diagnostics.try_reserve_exact(capacity).is_ok()
}

impl<'source, 'tokens> Parser<'source, 'tokens> {
    fn new(source: &'source SourceFile, tokens: &'tokens [Token], limits: Limits) -> Self {
        Self {
            source,
            tokens,
            cursor: 0,
            diagnostics: Vec::new(),
            ordinary_diagnostics: 0,
            diagnostic_limit_reported: false,
            resource_limit_reported: false,
            nodes: 0,
            events: 0,
            halted: false,
            limits,
            reserve_function_slot,
            reserve_parameter_slot,
            reserve_argument_slot,
            reserve_binding_slot,
            reserve_identifier_text,
            reserve_diagnostic_slots,
        }
    }

    fn run(mut self) -> ParseResult {
        if !(self.reserve_diagnostic_slots)(&mut self.diagnostics, MAX_RETAINED_PARSE_DIAGNOSTICS) {
            return ParseResult {
                ast: None,
                diagnostics: self.diagnostics,
            };
        }
        if self.tokens.len() > MAX_TOKENS_PER_SOURCE + 1 {
            self.resource_limit(format!(
                "parser input exceeds the {}-token stream limit",
                MAX_TOKENS_PER_SOURCE + 1
            ));
            return ParseResult {
                ast: None,
                diagnostics: self.diagnostics,
            };
        }
        if !self.token_stream_is_valid() {
            self.resource_limit("parser received an invalid lexer token stream");
            return ParseResult {
                ast: None,
                diagnostics: self.diagnostics,
            };
        }

        let edition = self.parse_edition_declaration();
        let module = self.parse_module_declaration();

        if !self.halted && self.current_kind() != TokenKind::Eof {
            self.report(
                DiagnosticCode::TrailingSyntax,
                "syntax follows the source module",
                self.current_span(),
                "only one module is allowed per source",
                "remove the trailing tokens or move declarations inside the module",
            );
            self.recover_to(&[TokenKind::Eof]);
        }

        let ast = if self.diagnostics.is_empty() {
            edition.zip(module).and_then(|(edition, module)| {
                let span = self.join(edition.span, module.span);
                self.record_node().then_some(SyntaxTree {
                    span,
                    edition,
                    module,
                })
            })
        } else {
            None
        };

        ParseResult {
            ast: if self.diagnostics.is_empty() {
                ast
            } else {
                None
            },
            diagnostics: self.diagnostics,
        }
    }

    fn token_stream_is_valid(&self) -> bool {
        let Some((eof, preceding)) = self.tokens.split_last() else {
            return false;
        };
        if eof.kind != TokenKind::Eof
            || !eof.span.is_empty()
            || eof.span.start() != self.source.byte_len()
            || self.source.slice(eof.span) != Some("")
        {
            return false;
        }
        if preceding.iter().any(|token| token.kind == TokenKind::Eof) {
            return false;
        }
        if self
            .tokens
            .iter()
            .any(|token| self.source.slice(token.span).is_none())
        {
            return false;
        }
        self.tokens.windows(2).all(|tokens| {
            let [left, right] = tokens else {
                return false;
            };
            left.span.end() <= right.span.start()
        })
    }

    fn parse_edition_declaration(&mut self) -> Option<EditionDeclaration> {
        let keyword = if self.current_kind() == TokenKind::KwEdition {
            self.bump()
        } else {
            self.expected("`edition`", "every source begins with `edition 2026;`");
            self.recover_to(&[
                TokenKind::Integer,
                TokenKind::Semicolon,
                TokenKind::KwModule,
                TokenKind::Eof,
            ]);
            None
        };

        let value = if self.current_kind() == TokenKind::Integer {
            let token = self.bump();
            if token.is_some_and(|token| token.lexeme(self.source) == Some("2026")) {
                token
            } else {
                self.report(
                    DiagnosticCode::UnsupportedSourceEdition,
                    "source edition must be exactly `2026`",
                    token.map_or_else(|| self.current_span(), |token| token.span),
                    "unsupported source edition",
                    "Orange currently defines only the 2026 edition",
                );
                None
            }
        } else {
            self.expected("the integer `2026`", "write `edition 2026;`");
            self.recover_to(&[TokenKind::Semicolon, TokenKind::KwModule, TokenKind::Eof]);
            None
        };

        let semicolon = if self.current_kind() == TokenKind::Semicolon {
            self.bump()
        } else {
            self.expected("`;` after the edition", "write `edition 2026;`");
            self.recover_to(&[TokenKind::KwModule, TokenKind::Eof]);
            None
        };

        match (keyword, value, semicolon) {
            (Some(keyword), Some(value), Some(semicolon)) if self.record_node() => {
                Some(EditionDeclaration {
                    span: self.join(keyword.span, semicolon.span),
                    edition: Edition::E2026,
                    value_span: value.span,
                })
            }
            _ => None,
        }
    }

    fn parse_module_declaration(&mut self) -> Option<ModuleDeclaration> {
        let keyword = if self.current_kind() == TokenKind::KwModule {
            self.bump()
        } else {
            self.expected("`module`", "an Orange source contains exactly one module");
            self.recover_to(&[TokenKind::Identifier, TokenKind::LeftBrace, TokenKind::Eof]);
            None
        };

        let name = self.parse_identifier("module name");
        if name.is_none() && !matches!(self.current_kind(), TokenKind::LeftBrace | TokenKind::Eof) {
            self.recover_to(&[TokenKind::LeftBrace, TokenKind::Eof]);
        }

        let left_brace = if self.current_kind() == TokenKind::LeftBrace {
            self.bump()
        } else {
            self.expected(
                "`{` after the module name",
                "module declarations use braces",
            );
            self.recover_to(&[
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ]);
            None
        };

        let mut functions = Vec::new();
        while !self.halted && !matches!(self.current_kind(), TokenKind::RightBrace | TokenKind::Eof)
        {
            let before = self.cursor;
            match self.current_kind() {
                TokenKind::KwSpec | TokenKind::KwImpl => {
                    if let Some(function) = self.parse_function_declaration() {
                        if (self.reserve_function_slot)(&mut functions) {
                            functions.push(function);
                        } else {
                            self.resource_limit_at(
                                "parser could not allocate module function storage",
                                function.span,
                            );
                        }
                    }
                }
                _ => {
                    self.report(
                        DiagnosticCode::ExpectedFunctionDeclaration,
                        "expected a `spec` or `impl` function declaration",
                        self.current_span(),
                        "this token cannot begin a module member",
                        "Orange 2026 admits empty functions and typed `spec` functions",
                    );
                    self.recover_to(&[
                        TokenKind::KwSpec,
                        TokenKind::KwImpl,
                        TokenKind::RightBrace,
                        TokenKind::Eof,
                    ]);
                }
            }
            if !self.halted && self.cursor == before {
                self.bump();
            }
        }

        let right_brace = if self.current_kind() == TokenKind::RightBrace {
            self.bump()
        } else {
            self.expected(
                "`}` to close the module",
                "close the module before end of file",
            );
            None
        };

        match (keyword, name, left_brace, right_brace) {
            (Some(keyword), Some(name), Some(_), Some(right_brace)) if self.record_node() => {
                Some(ModuleDeclaration {
                    span: self.join(keyword.span, right_brace.span),
                    name,
                    functions,
                })
            }
            _ => None,
        }
    }

    fn parse_function_declaration(&mut self) -> Option<FunctionDeclaration> {
        let keyword = self.bump()?;
        let kind = match keyword.kind {
            TokenKind::KwSpec => FunctionKind::Spec,
            TokenKind::KwImpl => FunctionKind::Impl,
            _ => return None,
        };

        let name = self.parse_identifier("function name");
        if name.is_none()
            && !matches!(
                self.current_kind(),
                TokenKind::LeftParen
                    | TokenKind::KwSpec
                    | TokenKind::KwImpl
                    | TokenKind::RightBrace
                    | TokenKind::Eof
            )
        {
            self.recover_to(&[
                TokenKind::LeftParen,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ]);
        }

        let left_paren = self.consume_or_recover(
            TokenKind::LeftParen,
            "`(` after the function name",
            "a function name is followed by its parameter list",
            &[
                TokenKind::RightParen,
                TokenKind::Arrow,
                TokenKind::LeftBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ],
        );
        let parameters = if left_paren.is_some() && self.current_kind() == TokenKind::Identifier {
            self.parse_parameter_list()
        } else {
            Some(Vec::new())
        };
        let right_paren = self.consume_or_recover(
            TokenKind::RightParen,
            "`)` to close the parameter list",
            "parameters are written `name: Type` and separated by commas",
            &[
                TokenKind::Arrow,
                TokenKind::LeftBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ],
        );
        let has_parameters = parameters.as_ref().is_some_and(|list| !list.is_empty());
        if kind == FunctionKind::Impl
            && let Some(first) = parameters.as_ref().and_then(|list| list.first())
        {
            self.report(
                DiagnosticCode::ExpectedSyntax,
                "`impl` functions have an empty parameter list",
                first.span,
                "parameters are allowed only on typed `spec` functions",
                "keep the legacy `impl name() {}` form until implementation semantics are defined",
            );
        }

        let (body, body_end) = match self.current_kind() {
            TokenKind::LeftBrace if kind == FunctionKind::Spec && has_parameters => {
                self.report(
                    DiagnosticCode::ExpectedSyntax,
                    "expected `->` after the parameter list",
                    self.current_span(),
                    "a `spec` with parameters needs a result type and a body expression",
                    "write `spec name(x: Type) -> Type { expression }`",
                );
                self.recover_to(&[
                    TokenKind::KwSpec,
                    TokenKind::KwImpl,
                    TokenKind::RightBrace,
                    TokenKind::Eof,
                ]);
                (None, None)
            }
            TokenKind::LeftBrace => self.parse_empty_function_body(),
            TokenKind::Arrow if kind == FunctionKind::Spec => self.parse_typed_body(),
            TokenKind::Arrow => {
                self.report(
                    DiagnosticCode::ExpectedSyntax,
                    "typed bodies are allowed only on `spec` functions",
                    self.current_span(),
                    "an `impl` function cannot have a typed body",
                    "keep the legacy `impl name() {}` form until implementation semantics are defined",
                );
                self.recover_to(&[
                    TokenKind::KwSpec,
                    TokenKind::KwImpl,
                    TokenKind::RightBrace,
                    TokenKind::Eof,
                ]);
                (None, None)
            }
            _ => {
                self.expected(
                    if kind == FunctionKind::Spec {
                        "`{}` or `->` after the parameter list"
                    } else {
                        "`{` to begin the empty `impl` body"
                    },
                    if kind == FunctionKind::Spec {
                        "a `spec` is either legacy-empty or has a typed body"
                    } else {
                        "typed `impl` bodies are not part of this syntax"
                    },
                );
                self.recover_to(&[
                    TokenKind::KwSpec,
                    TokenKind::KwImpl,
                    TokenKind::RightBrace,
                    TokenKind::Eof,
                ]);
                (None, None)
            }
        };

        match (name, left_paren, parameters, right_paren, body, body_end) {
            (Some(name), Some(_), Some(parameters), Some(_), Some(body), Some(body_end))
                if self.record_node() =>
            {
                Some(FunctionDeclaration {
                    span: self.join(keyword.span, body_end.span),
                    kind,
                    name,
                    parameters,
                    body,
                })
            }
            _ => None,
        }
    }

    fn parse_parameter_list(&mut self) -> Option<Vec<Parameter>> {
        let mut parameters = Vec::new();
        let mut complete = true;
        loop {
            match self.parse_parameter() {
                Some(parameter) => {
                    if parameters.len() >= MAX_PARAMETERS_PER_FUNCTION {
                        self.resource_limit_at(
                            format!(
                                "function declares more than {MAX_PARAMETERS_PER_FUNCTION} parameters"
                            ),
                            parameter.span,
                        );
                        return None;
                    }
                    if !(self.reserve_parameter_slot)(&mut parameters) {
                        self.resource_limit_at(
                            "parser could not allocate parameter storage",
                            parameter.span,
                        );
                        return None;
                    }
                    parameters.push(parameter);
                }
                None => {
                    complete = false;
                    self.recover_to(&[
                        TokenKind::Comma,
                        TokenKind::RightParen,
                        TokenKind::Arrow,
                        TokenKind::LeftBrace,
                        TokenKind::KwSpec,
                        TokenKind::KwImpl,
                        TokenKind::RightBrace,
                        TokenKind::Eof,
                    ]);
                }
            }
            if self.halted || self.current_kind() != TokenKind::Comma {
                break;
            }
            self.bump();
            // A trailing comma before `)` is permitted.
            if self.current_kind() == TokenKind::RightParen {
                break;
            }
        }
        complete.then_some(parameters)
    }

    fn parse_parameter(&mut self) -> Option<Parameter> {
        let name = self.parse_identifier("parameter name")?;
        if self.current_kind() == TokenKind::Colon {
            self.bump();
        } else {
            self.expected(
                "`:` after the parameter name",
                "parameters are written `name: Type`",
            );
            return None;
        }
        let ty = self.parse_type_syntax("parameter type")?;
        let span = self.join(name.span, ty.span);
        self.record_node().then_some(Parameter { span, name, ty })
    }

    fn parse_empty_function_body(&mut self) -> (Option<FunctionBody>, Option<Token>) {
        let left_brace = self.bump();
        let right_brace = if self.current_kind() == TokenKind::RightBrace {
            self.bump()
        } else {
            self.expected(
                "`}` immediately after the function body's `{`",
                "legacy empty function bodies are written `{}`",
            );
            self.recover_to(&[
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ]);
            if self.current_kind() == TokenKind::RightBrace {
                self.bump()
            } else {
                None
            }
        };

        match (left_brace, right_brace) {
            (Some(_), Some(right_brace)) => (Some(FunctionBody::Empty), Some(right_brace)),
            _ => (None, right_brace),
        }
    }

    fn parse_typed_body(&mut self) -> (Option<FunctionBody>, Option<Token>) {
        let arrow = self.bump();
        let result_type = self.parse_type_syntax("result type");
        if result_type.is_none()
            && !matches!(
                self.current_kind(),
                TokenKind::LeftBrace
                    | TokenKind::KwSpec
                    | TokenKind::KwImpl
                    | TokenKind::RightBrace
                    | TokenKind::Eof
            )
        {
            self.recover_to(&[
                TokenKind::LeftBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ]);
        }

        let left_brace = self.consume_or_recover(
            TokenKind::LeftBrace,
            "`{` after the result type",
            BODY_SHAPE_NOTE,
            &[
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ],
        );
        let bindings = if left_brace.is_some() {
            self.parse_bindings()
        } else {
            None
        };
        let expression = match &bindings {
            Some(bindings)
                if !bindings.is_empty() && self.current_kind() == TokenKind::RightBrace =>
            {
                self.expected(
                    "a result expression after the last binding",
                    "a typed `spec` body ends with the expression that gives its value",
                );
                None
            }
            Some(_) => self.parse_expression(0).map(|(expression, _)| expression),
            None => None,
        };
        if expression.is_none()
            && !matches!(
                self.current_kind(),
                TokenKind::RightBrace | TokenKind::KwSpec | TokenKind::KwImpl | TokenKind::Eof
            )
        {
            self.recover_to(&[
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ]);
        }

        let right_brace = if self.current_kind() == TokenKind::RightBrace {
            self.bump()
        } else {
            if expression.is_some() {
                self.expected("`}` after the body expression", BODY_SHAPE_NOTE);
            }
            self.recover_to(&[
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ]);
            if self.current_kind() == TokenKind::RightBrace {
                self.bump()
            } else {
                None
            }
        };

        match (arrow, result_type, bindings, expression, right_brace) {
            (
                Some(arrow),
                Some(result_type),
                Some(bindings),
                Some(expression),
                Some(right_brace),
            ) if self.record_node() => (
                Some(FunctionBody::Typed(Box::new(TypedBody {
                    span: self.join(arrow.span, right_brace.span),
                    result_type,
                    bindings,
                    expression,
                }))),
                Some(right_brace),
            ),
            (_, _, _, _, right_brace) => (None, right_brace),
        }
    }

    /// Parses the `let` bindings at the start of a typed body.
    ///
    /// `let` is recognized by position, not reserved: it starts a binding
    /// only when an identifier follows it, which no expression allows.
    fn parse_bindings(&mut self) -> Option<Vec<Binding>> {
        let mut bindings = Vec::new();
        while self.current_is_word("let") && self.next_kind() == TokenKind::Identifier {
            let binding = self.parse_binding()?;
            if bindings.len() >= MAX_BINDINGS_PER_BODY {
                self.resource_limit_at(
                    format!("typed body declares more than {MAX_BINDINGS_PER_BODY} bindings"),
                    binding.span,
                );
                return None;
            }
            if !(self.reserve_binding_slot)(&mut bindings) {
                self.resource_limit_at("parser could not allocate binding storage", binding.span);
                return None;
            }
            bindings.push(binding);
        }
        Some(bindings)
    }

    fn parse_binding(&mut self) -> Option<Binding> {
        let keyword = self.bump()?;
        let name = self.parse_identifier("binding")?;
        self.expect(
            TokenKind::Colon,
            "`:` and the binding's type",
            "every binding states its type, as in `let t: Word[32] = x + y;`",
        )?;
        let ty = self.parse_type_syntax("binding type")?;
        self.expect(
            TokenKind::Equal,
            "`=` after the binding's type",
            "a binding is written `let name: Type = expression;`",
        )?;
        let (value, _) = self.parse_expression(0)?;
        let semicolon = self.expect(
            TokenKind::Semicolon,
            "`;` after the bound expression",
            "each binding ends with `;`; the body's last item is its result expression",
        )?;
        let span = self.join(keyword.span, semicolon.span);
        self.record_node().then_some(Binding {
            span,
            name,
            ty,
            value,
        })
    }

    /// Parses one expression enclosed by `level` groups, call argument
    /// lists, and prefix operators.
    ///
    /// Returns the expression and its tree height. The first binary operator
    /// after the first operand selects the expression's operator group;
    /// operators from another group must be parenthesized.
    ///
    /// Recursion happens only where a new nesting level opens, and every
    /// opener checks [`MAX_EXPRESSION_NESTING`] first, so the parser's stack
    /// use is bounded independently of expression length. Operator chains are
    /// parsed by loops.
    fn parse_expression(&mut self, level: usize) -> Option<(Expression, usize)> {
        if level > MAX_EXPRESSION_NESTING {
            self.nesting_limit(self.current_span());
            return None;
        }
        // The operator groups are parsed in this one function, not in
        // helpers, so each nesting level adds as few frames as possible.
        let mut expression = self.parse_unary(level)?;
        let previous = match self.current_joiner() {
            None => return Some(expression),
            Some(Joiner::As) => {
                expression = self.parse_conversion(expression)?;
                Joiner::As
            }
            Some(Joiner::Binary(
                group @ (BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply),
            )) => {
                // `*` binds tighter than `+` and `-`; each is left-associative.
                // `sum` holds the completed terms and the additive operator
                // awaiting the current product.
                let mut previous = group;
                let mut sum: Option<((Expression, usize), BinaryOperator, Span)> = None;
                loop {
                    let operator = match self.current_kind() {
                        TokenKind::Star => BinaryOperator::Multiply,
                        TokenKind::Plus => BinaryOperator::Add,
                        TokenKind::Minus => BinaryOperator::Subtract,
                        _ => break,
                    };
                    previous = operator;
                    let operator_span = self.bump()?.span;
                    let operand = self.parse_unary(level)?;
                    if operator == BinaryOperator::Multiply {
                        expression =
                            self.binary_node(expression, operator, operator_span, operand)?;
                    } else {
                        let term = match sum.take() {
                            Some((left, pending, pending_span)) => {
                                self.binary_node(left, pending, pending_span, expression)?
                            }
                            None => expression,
                        };
                        sum = Some((term, operator, operator_span));
                        expression = operand;
                    }
                }
                if let Some((left, pending, pending_span)) = sum {
                    expression = self.binary_node(left, pending, pending_span, expression)?;
                }
                Joiner::Binary(previous)
            }
            Some(Joiner::Binary(
                group @ (BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Xor),
            )) => {
                while self.current_kind() == group.token_kind() {
                    let operator_span = self.bump()?.span;
                    let operand = self.parse_unary(level)?;
                    expression = self.binary_node(expression, group, operator_span, operand)?;
                }
                Joiner::Binary(group)
            }
            Some(Joiner::Binary(
                group @ (BinaryOperator::ShiftLeft
                | BinaryOperator::ShiftRight
                | BinaryOperator::RotateLeft
                | BinaryOperator::RotateRight),
            )) => {
                let operator_span = self.bump()?.span;
                let amount = self.parse_unary(level)?;
                expression = self.binary_node(expression, group, operator_span, amount)?;
                Joiner::Binary(group)
            }
        };

        if let Some(ungrouped) = self.current_joiner() {
            self.report_ungrouped(ungrouped, previous);
            // Continue through the remaining operators and conversions so
            // that one ungrouped expression produces one diagnostic and
            // parsing stays aligned.
            while let Some(joiner) = self.current_joiner() {
                expression = match joiner {
                    Joiner::Binary(operator) => {
                        let operator_span = self.bump()?.span;
                        let operand = self.parse_unary(level)?;
                        self.binary_node(expression, operator, operator_span, operand)?
                    }
                    Joiner::As => self.parse_conversion(expression)?,
                };
            }
        }
        Some(expression)
    }

    fn current_joiner(&self) -> Option<Joiner> {
        if self.current_is_word("as") {
            return Some(Joiner::As);
        }
        BinaryOperator::from_token(self.current_kind()).map(Joiner::Binary)
    }

    #[cold]
    #[inline(never)]
    fn report_ungrouped(&mut self, ungrouped: Joiner, previous: Joiner) {
        let span = self.current_span();
        self.report_lazy(span, || {
            Diagnostic::error(
                DiagnosticCode::UngroupedOperators,
                format!(
                    "`{}` follows `{}` without grouping parentheses",
                    ungrouped.as_str(),
                    previous.as_str()
                ),
                span,
            )
            .with_label("ungrouped operator")
            .with_note(match (previous, ungrouped) {
                (Joiner::As, _) | (_, Joiner::As) => {
                    "`as` converts exactly one operand; parenthesize the conversion or the \
                     expression it converts"
                }
                (Joiner::Binary(previous), Joiner::Binary(ungrouped))
                    if previous.is_shift_or_rotation() && ungrouped.is_shift_or_rotation() =>
                {
                    "a shift or rotation takes exactly two operands; parenthesize one of them"
                }
                (Joiner::Binary(_), Joiner::Binary(_)) => {
                    "operators from different groups have no relative precedence in Orange; \
                     parenthesize the part that applies first"
                }
            })
        });
    }

    /// Parses `as Type` after a complete operand.
    #[inline(never)]
    fn parse_conversion(
        &mut self,
        (operand, operand_height): (Expression, usize),
    ) -> Option<(Expression, usize)> {
        let keyword_span = self.bump()?.span;
        let target = self.parse_type_syntax("conversion type")?;
        let height = self.node_height(operand_height, keyword_span)?;
        let span = self.join(operand.span, target.span);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Conversion(Box::new(ConversionExpression {
                    operand,
                    keyword_span,
                    target,
                })),
            },
            height,
        ))
    }

    #[inline(never)]
    fn binary_node(
        &mut self,
        (left, left_height): (Expression, usize),
        operator: BinaryOperator,
        operator_span: Span,
        (right, right_height): (Expression, usize),
    ) -> Option<(Expression, usize)> {
        let height = self.node_height(left_height.max(right_height), operator_span)?;
        let span = self.join(left.span, right.span);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Binary(BinaryExpression {
                    operator,
                    operator_span,
                    left: Box::new(left),
                    right: Box::new(right),
                }),
            },
            height,
        ))
    }

    fn node_height(&mut self, child_height: usize, span: Span) -> Option<usize> {
        let height = child_height.saturating_add(1);
        if height > MAX_EXPRESSION_HEIGHT {
            self.height_limit(span);
            return None;
        }
        Some(height)
    }

    #[cold]
    #[inline(never)]
    fn nesting_limit(&mut self, span: Span) {
        self.resource_limit_at(
            format!(
                "expression nesting exceeds the {MAX_EXPRESSION_NESTING}-level limit \
                 for groups, calls, and prefix operators"
            ),
            span,
        );
    }

    #[cold]
    #[inline(never)]
    fn height_limit(&mut self, span: Span) {
        self.resource_limit_at(
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit"),
            span,
        );
    }

    /// Opens one nesting level at the current token, or reports the limit.
    fn open_level(&mut self, level: usize) -> Option<usize> {
        let inner = level.saturating_add(1);
        if inner > MAX_EXPRESSION_NESTING {
            self.nesting_limit(self.current_span());
            return None;
        }
        Some(inner)
    }

    fn parse_unary(&mut self, level: usize) -> Option<(Expression, usize)> {
        let operator = match self.current_kind() {
            // A sign written directly before an integer token is part of the
            // literal, exactly as in the S3a typed-literal body.
            TokenKind::Minus if self.next_kind() == TokenKind::Integer => {
                return self.parse_literal_expression();
            }
            TokenKind::Minus => UnaryOperator::Negate,
            TokenKind::Tilde => UnaryOperator::Complement,
            _ => return self.parse_primary(level),
        };
        let inner = self.open_level(level)?;
        let operator_span = self.bump()?.span;
        let operand = self.parse_unary(inner)?;
        self.unary_node(operator, operator_span, operand)
    }

    #[inline(never)]
    fn unary_node(
        &mut self,
        operator: UnaryOperator,
        operator_span: Span,
        (operand, operand_height): (Expression, usize),
    ) -> Option<(Expression, usize)> {
        let height = self.node_height(operand_height, operator_span)?;
        let span = self.join(operator_span, operand.span);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Unary(UnaryExpression {
                    operator,
                    operator_span,
                    operand: Box::new(operand),
                }),
            },
            height,
        ))
    }

    fn parse_primary(&mut self, level: usize) -> Option<(Expression, usize)> {
        match self.current_kind() {
            TokenKind::Integer => self.parse_literal_expression(),
            TokenKind::Identifier if self.next_kind() == TokenKind::LeftParen => {
                self.parse_call(level)
            }
            TokenKind::Identifier => self.parse_name_expression(),
            TokenKind::LeftParen => {
                let inner = self.open_level(level)?;
                let left_paren = self.bump()?.span;
                let group = self.parse_expression(inner)?;
                self.finish_group(left_paren, group)
            }
            _ => {
                self.expected(
                    "an expression",
                    "an expression is an integer literal, a parameter, a call, a prefix \
                     operator, or a parenthesized expression",
                );
                None
            }
        }
    }

    #[inline(never)]
    fn parse_literal_expression(&mut self) -> Option<(Expression, usize)> {
        let literal = self.parse_integer_literal()?;
        Some((
            Expression {
                span: literal.span,
                kind: ExpressionKind::Literal(literal),
            },
            1,
        ))
    }

    #[inline(never)]
    fn parse_name_expression(&mut self) -> Option<(Expression, usize)> {
        let name = self.parse_identifier("parameter")?;
        Some((
            Expression {
                span: name.span,
                kind: ExpressionKind::Name(name),
            },
            1,
        ))
    }

    #[inline(never)]
    fn finish_group(
        &mut self,
        left_paren: Span,
        (inner, inner_height): (Expression, usize),
    ) -> Option<(Expression, usize)> {
        let right_paren = self.consume_or_recover(
            TokenKind::RightParen,
            "`)` to close the group",
            "every `(` in an expression needs a matching `)`",
            &[
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ],
        )?;
        let height = self.node_height(inner_height, left_paren)?;
        let span = self.join(left_paren, right_paren.span);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Parenthesized(Box::new(inner)),
            },
            height,
        ))
    }

    fn parse_call(&mut self, level: usize) -> Option<(Expression, usize)> {
        let inner = self.open_level(level)?;
        let callee = self.parse_identifier("called function")?;
        self.bump()?;
        let mut arguments = Vec::new();
        let mut argument_height = 0_usize;
        while self.current_kind() != TokenKind::RightParen {
            let argument = self.parse_expression(inner)?;
            argument_height = argument_height.max(argument.1);
            if !self.push_argument(&mut arguments, argument.0) {
                return None;
            }
            match self.current_kind() {
                TokenKind::Comma => {
                    self.bump()?;
                }
                TokenKind::RightParen => break,
                _ => {
                    self.expected(
                        "`,` or `)` after the argument",
                        "arguments are separated by commas",
                    );
                    return None;
                }
            }
        }
        let right_paren = self.bump()?;
        let height = self.node_height(argument_height, callee.span)?;
        let span = self.join(callee.span, right_paren.span);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Call(CallExpression { callee, arguments }),
            },
            height,
        ))
    }

    #[inline(never)]
    fn push_argument(&mut self, arguments: &mut Vec<Expression>, argument: Expression) -> bool {
        if arguments.len() >= MAX_ARGUMENTS_PER_CALL {
            self.resource_limit_at(
                format!("call supplies more than {MAX_ARGUMENTS_PER_CALL} arguments"),
                argument.span,
            );
            return false;
        }
        if !(self.reserve_argument_slot)(arguments) {
            self.resource_limit_at("parser could not allocate argument storage", argument.span);
            return false;
        }
        arguments.push(argument);
        true
    }

    fn parse_type_syntax(&mut self, role: &str) -> Option<TypeSyntax> {
        let name = self.parse_identifier(role)?;
        let mut end = name.span;
        let mut width_span = None;

        if self.current_kind() == TokenKind::LeftBracket {
            self.bump();
            let width = if self.current_kind() == TokenKind::Integer {
                self.bump()
            } else {
                self.expected(
                    "an integer width after `[`",
                    "width-parameter syntax is written `Name[WIDTH]`",
                );
                None
            };
            width_span = width.map(|token| token.span);

            let right_bracket = if self.current_kind() == TokenKind::RightBracket {
                self.bump()
            } else {
                self.expected(
                    "`]` after the type width",
                    "width-parameter syntax is written `Name[WIDTH]`",
                );
                self.recover_to(&[
                    TokenKind::RightBracket,
                    TokenKind::LeftBrace,
                    TokenKind::KwSpec,
                    TokenKind::KwImpl,
                    TokenKind::RightBrace,
                    TokenKind::Eof,
                ]);
                if self.current_kind() == TokenKind::RightBracket {
                    self.bump()
                } else {
                    None
                }
            };
            if let Some(right_bracket) = right_bracket {
                end = right_bracket.span;
            }
            if width.is_none() || right_bracket.is_none() {
                return None;
            }
        }

        self.record_node().then_some(TypeSyntax {
            span: self.join(name.span, end),
            name,
            width_span,
        })
    }

    fn parse_integer_literal(&mut self) -> Option<IntegerLiteral> {
        let minus = if self.current_kind() == TokenKind::Minus {
            self.bump()
        } else {
            None
        };
        if self.current_kind() != TokenKind::Integer {
            self.expected(
                "an integer literal",
                "a sign is part of a literal only when an integer follows it",
            );
            return None;
        }
        let magnitude = self.bump()?;
        let span = minus.map_or(magnitude.span, |minus| {
            self.join(minus.span, magnitude.span)
        });
        self.record_node().then_some(IntegerLiteral {
            span,
            magnitude_span: magnitude.span,
            negative: minus.is_some(),
        })
    }

    fn parse_identifier(&mut self, role: &str) -> Option<Identifier> {
        if self.current_kind() != TokenKind::Identifier {
            self.expected_message("reserved words cannot be used as names", || {
                format!("expected an identifier for the {role}")
            });
            return None;
        }
        let token = self.bump()?;
        let spelling = token.lexeme(self.source)?;
        let mut text = String::new();
        if !(self.reserve_identifier_text)(&mut text, spelling.len()) {
            self.resource_limit_at(
                "parser could not allocate identifier text storage",
                token.span,
            );
            return None;
        }
        text.push_str(spelling);
        self.record_node().then_some(Identifier {
            text,
            span: token.span,
        })
    }

    /// Consumes a token of `kind`, or reports what was expected.
    fn expect(&mut self, kind: TokenKind, expected: &str, note: &str) -> Option<Token> {
        if self.current_kind() == kind {
            return self.bump();
        }
        self.expected(expected, note);
        None
    }

    /// Returns whether the current token is the identifier spelled `word`.
    fn current_is_word(&self, word: &str) -> bool {
        self.tokens
            .get(self.cursor)
            .filter(|token| token.kind == TokenKind::Identifier)
            .and_then(|token| token.lexeme(self.source))
            == Some(word)
    }

    fn consume_or_recover(
        &mut self,
        kind: TokenKind,
        expected: &str,
        note: &str,
        recovery: &[TokenKind],
    ) -> Option<Token> {
        if self.current_kind() == kind {
            return self.bump();
        }
        self.expected(expected, note);
        self.recover_to(recovery);
        None
    }

    fn expected(&mut self, expected: &str, note: &str) {
        self.expected_message(note, || format!("expected {expected}"));
    }

    fn expected_message(&mut self, note: &str, build_message: impl FnOnce() -> String) {
        let span = self.current_span();
        let found = self.current_kind();
        self.report_lazy(span, || {
            Diagnostic::error(DiagnosticCode::ExpectedSyntax, build_message(), span)
                .with_label(format!("found {}", found.name()))
                .with_note(note)
        });
    }

    fn report(
        &mut self,
        code: DiagnosticCode,
        message: impl Into<String>,
        span: Span,
        label: impl Into<String>,
        note: impl Into<String>,
    ) {
        self.report_lazy(span, || {
            Diagnostic::error(code, message, span)
                .with_label(label)
                .with_note(note)
        });
    }

    fn report_lazy(&mut self, span: Span, build: impl FnOnce() -> Diagnostic) {
        if self.halted || !self.event() {
            return;
        }
        if self.ordinary_diagnostics < self.limits.diagnostics {
            self.ordinary_diagnostics = self.ordinary_diagnostics.saturating_add(1);
            self.diagnostics.push(build());
        } else if !self.diagnostic_limit_reported {
            self.diagnostic_limit_reported = true;
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TooManySyntaxErrors,
                    format!(
                        "stopped reporting after {} syntax errors",
                        self.limits.diagnostics
                    ),
                    span,
                )
                .with_label("further syntax errors are suppressed")
                .with_note("fix the reported errors before parsing this source again"),
            );
        }
    }

    fn recover_to(&mut self, recovery: &[TokenKind]) {
        let mut depth = 0_usize;
        while !self.halted && self.current_kind() != TokenKind::Eof {
            let kind = self.current_kind();
            if depth == 0 && recovery.contains(&kind) {
                return;
            }
            match kind {
                TokenKind::LeftParen | TokenKind::LeftBrace | TokenKind::LeftBracket => {
                    depth = depth.saturating_add(1);
                    if depth > self.limits.recovery_depth {
                        self.resource_limit(format!(
                            "parser recovery exceeds the {}-delimiter nesting limit",
                            self.limits.recovery_depth
                        ));
                        return;
                    }
                }
                TokenKind::RightParen | TokenKind::RightBrace | TokenKind::RightBracket => {
                    depth = depth.saturating_sub(1);
                }
                _ => {}
            }
            self.bump();
        }
    }

    fn current_kind(&self) -> TokenKind {
        self.tokens
            .get(self.cursor)
            .map_or(TokenKind::Eof, |token| token.kind)
    }

    fn next_kind(&self) -> TokenKind {
        self.tokens
            .get(self.cursor.saturating_add(1))
            .map_or(TokenKind::Eof, |token| token.kind)
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.cursor)
            .map(|token| token.span)
            .filter(|span| self.source.slice(*span).is_some())
            .unwrap_or_else(|| {
                self.source
                    .lexer_span(self.source.text().len(), self.source.text().len())
            })
    }

    fn bump(&mut self) -> Option<Token> {
        if self.halted || !self.event() {
            return None;
        }
        let token = self.tokens.get(self.cursor).copied().or_else(|| {
            Some(Token {
                kind: TokenKind::Eof,
                span: self.current_span(),
            })
        })?;
        if token.kind != TokenKind::Eof {
            self.cursor = self.cursor.saturating_add(1);
        }
        Some(token)
    }

    fn join(&self, first: Span, last: Span) -> Span {
        self.source
            .span(first.start(), last.end())
            .unwrap_or_else(|| self.current_span())
    }

    fn record_node(&mut self) -> bool {
        if self.halted || !self.event() {
            return false;
        }
        if self.nodes >= self.limits.nodes {
            self.resource_limit(format!(
                "parser exceeds the {}-syntax-node limit",
                self.limits.nodes
            ));
            return false;
        }
        self.nodes = self.nodes.saturating_add(1);
        true
    }

    fn event(&mut self) -> bool {
        if self.events >= self.limits.events {
            self.resource_limit(format!(
                "parser exceeds the {}-event limit",
                self.limits.events
            ));
            return false;
        }
        self.events = self.events.saturating_add(1);
        true
    }

    fn resource_limit(&mut self, message: impl Into<String>) {
        let span = self.current_span();
        self.resource_limit_at(message, span);
    }

    fn resource_limit_at(&mut self, message: impl Into<String>, span: Span) {
        self.halted = true;
        if self.resource_limit_reported {
            return;
        }
        self.resource_limit_reported = true;
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::ParserResourceLimit, message, span)
                .with_label("deterministic parser resource limit reached")
                .with_note("simplify or split the source before parsing it again"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Lexed, lex};
    use crate::source::{SourceMap, TextOffset};

    struct CountedMessage<'counter>(&'counter std::cell::Cell<usize>);

    impl From<CountedMessage<'_>> for String {
        fn from(message: CountedMessage<'_>) -> Self {
            message.0.set(message.0.get().saturating_add(1));
            Self::from("counted parser resource failure")
        }
    }

    #[test]
    fn function_kind_inventory_and_spellings_are_exact() {
        assert_eq!(FunctionKind::ALL, &[FunctionKind::Spec, FunctionKind::Impl]);
        assert_eq!(
            FunctionKind::ALL
                .iter()
                .map(|kind| kind.as_str())
                .collect::<Vec<_>>(),
            ["spec", "impl"]
        );
    }

    fn literal_of(body: &TypedBody) -> &IntegerLiteral {
        match &body.expression.kind {
            ExpressionKind::Literal(literal) => literal,
            other => panic!("expected a literal body, found {other:?}"),
        }
    }

    fn parse_text(text: &str) -> (SourceMap, Lexed, ParseResult) {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let parsed = parse(source, &lexed);
        (sources, lexed, parsed)
    }

    #[test]
    fn builds_a_complete_ast_with_exact_spans() {
        let text = "edition 2026;\nmodule demo {\n  spec one() {}\n  impl two() {}\n}";
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty());
        let ast = parsed.ast.unwrap();
        let source = sources.iter().next().unwrap();

        assert_eq!(source.slice(ast.span), Some(text));
        assert_eq!(source.slice(ast.edition.span), Some("edition 2026;"));
        assert_eq!(source.slice(ast.edition.value_span), Some("2026"));
        assert_eq!(ast.edition.edition, Edition::E2026);
        assert_eq!(ast.module.name.text, "demo");
        assert_eq!(ast.module.functions.len(), 2);
        assert_eq!(ast.module.functions[0].kind, FunctionKind::Spec);
        assert_eq!(ast.module.functions[0].name.text, "one");
        assert_eq!(ast.module.functions[0].body, FunctionBody::Empty);
        assert_eq!(
            source.slice(ast.module.functions[0].span),
            Some("spec one() {}")
        );
        assert_eq!(ast.module.functions[1].kind, FunctionKind::Impl);
        assert_eq!(source.slice(ast.module.span), Some(&text[14..]));
    }

    #[test]
    fn builds_typed_literal_spec_nodes_with_exact_spans() {
        let text = concat!(
            "edition 2026; module demo { ",
            "spec answer() -> Int { -0x2a } ",
            "spec byte() -> Word[8] { 255 } ",
            "impl legacy() {} ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty());
        let ast = parsed.ast.unwrap();
        let source = sources.iter().next().unwrap();

        let answer = &ast.module.functions[0];
        assert_eq!(answer.kind, FunctionKind::Spec);
        assert_eq!(
            source.slice(answer.span),
            Some("spec answer() -> Int { -0x2a }")
        );
        let FunctionBody::Typed(answer_body) = &answer.body else {
            panic!("expected a typed body");
        };
        let answer_literal = literal_of(answer_body);
        assert_eq!(source.slice(answer_body.span), Some("-> Int { -0x2a }"));
        assert_eq!(source.slice(answer_body.result_type.span), Some("Int"));
        assert_eq!(answer_body.result_type.name.text, "Int");
        assert_eq!(answer_body.result_type.width_span, None);
        assert_eq!(source.slice(answer_literal.span), Some("-0x2a"));
        assert_eq!(source.slice(answer_literal.magnitude_span), Some("0x2a"));
        assert!(answer_literal.negative);

        let byte = &ast.module.functions[1];
        let FunctionBody::Typed(byte_body) = &byte.body else {
            panic!("expected a typed body");
        };
        assert_eq!(source.slice(byte_body.result_type.span), Some("Word[8]"));
        assert_eq!(byte_body.result_type.name.text, "Word");
        assert_eq!(
            byte_body
                .result_type
                .width_span
                .and_then(|span| source.slice(span)),
            Some("8")
        );
        assert_eq!(source.slice(literal_of(byte_body).span), Some("255"));
        assert!(!literal_of(byte_body).negative);
        assert_eq!(ast.module.functions[2].body, FunctionBody::Empty);
    }

    #[test]
    fn parses_generic_type_and_literal_syntax_without_assigning_semantics() {
        let text = "edition 2026; module m { spec f() -> FutureType[0x10] { -1_000 } }";
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty());
        let source = sources.iter().next().unwrap();
        let function = &parsed.ast.unwrap().module.functions[0];
        let FunctionBody::Typed(body) = &function.body else {
            panic!("expected a typed body");
        };

        assert_eq!(body.result_type.name.text, "FutureType");
        assert_eq!(
            source.slice(body.result_type.span),
            Some("FutureType[0x10]")
        );
        assert_eq!(
            body.result_type
                .width_span
                .and_then(|span| source.slice(span)),
            Some("0x10")
        );
        assert_eq!(source.slice(literal_of(body).span), Some("-1_000"));
        assert!(literal_of(body).negative);
    }

    #[test]
    fn accepts_duplicate_function_names_as_syntax_in_source_order() {
        let text = "edition 2026; module demo { spec same() {} impl same() {} }";
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty());
        let ast = parsed.ast.unwrap();
        let source = sources.iter().next().unwrap();

        assert_eq!(ast.module.functions.len(), 2);
        assert_eq!(ast.module.functions[0].name.text, "same");
        assert_eq!(ast.module.functions[1].name.text, "same");
        assert_eq!(ast.module.functions[0].kind, FunctionKind::Spec);
        assert_eq!(ast.module.functions[1].kind, FunctionKind::Impl);
        assert_ne!(
            ast.module.functions[0].name.span,
            ast.module.functions[1].name.span
        );
        assert_eq!(
            source.slice(ast.module.functions[0].name.span),
            Some("same")
        );
        assert_eq!(
            source.slice(ast.module.functions[1].name.span),
            Some("same")
        );
    }

    #[test]
    fn accepts_an_empty_module() {
        let text = "edition 2026; module empty {}";
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty());
        let ast = parsed.ast.unwrap();
        let source = sources.iter().next().unwrap();

        assert_eq!(ast.module.name.text, "empty");
        assert!(ast.module.functions.is_empty());
        assert_eq!(source.slice(ast.span), Some(text));
    }

    #[test]
    fn accepts_lf_crlf_and_bare_cr_as_logical_line_endings() {
        for ending in ["\n", "\r\n", "\r"] {
            let text = format!(
                "edition 2026;{ending}module demo {{{ending}// member{ending}spec f() {{}}{ending}}}"
            );
            let (sources, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{ending:?}");
            assert!(parsed.diagnostics.is_empty(), "{ending:?}");
            let source = sources.iter().next().unwrap();
            assert_eq!(source.line_text(1), Some("edition 2026;"));
            assert_eq!(source.line_text(2), Some("module demo {"));
            assert_eq!(source.line_text(3), Some("// member"));
            assert_eq!(source.line_text(4), Some("spec f() {}"));
            assert_eq!(
                source.line_column(parsed.ast.unwrap().module.functions[0].span.start()),
                Some(crate::source::LineColumn { line: 4, column: 1 })
            );
        }
    }

    #[test]
    fn diagnoses_every_malformed_production_without_an_ast() {
        let corpus = [
            "2026; module m {}",
            "edition; module m {}",
            "edition 2026 module m {}",
            "edition 2026; m {}",
            "edition 2026; module {}",
            "edition 2026; module m spec f() {} }",
            "edition 2026; module m { f() {} }",
            "edition 2026; module m { spec () {} }",
            "edition 2026; module m { spec f) {} }",
            "edition 2026; module m { spec f( {} }",
            "edition 2026; module m { spec f() } }",
            "edition 2026; module m { spec f() { x } }",
            "edition 2026; module m { spec f() {}",
            "edition 2026; module m { spec f() -> { 1 } }",
            "edition 2026; module m { spec f() -> Word[] { 1 } }",
            "edition 2026; module m { spec f() -> Word[8 { 1 } }",
            "edition 2026; module m { spec f() -> Int 1 }",
            "edition 2026; module m { spec f() -> Int {} }",
            "edition 2026; module m { spec f() -> Int { - } }",
            "edition 2026; module m { spec f() -> Int { 1 2 } }",
            "edition 2026; module m { spec f() -> Int { +5 } }",
            "edition 2026; module m { spec f() -> Word[-8] { 1 } }",
            "edition 2026; module m { spec f() -> Word[+8] { 1 } }",
            "edition 2026; module m { impl f() -> Int { 1 } }",
        ];

        for text in corpus {
            let (_, _, parsed) = parse_text(text);
            assert!(parsed.has_errors(), "accepted {text:?}");
            assert!(parsed.ast.is_none(), "partial AST escaped for {text:?}");
        }
    }

    #[test]
    fn rejects_typed_impl_and_recovers_to_the_next_member() {
        let text = concat!(
            "edition 2026; module m { ",
            "impl forbidden() -> Int { 1 } ",
            "spec allowed() -> Int { 2 } ",
            "}"
        );
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let first = parse(source, &lexed);
        let second = parse(source, &lexed);

        assert!(lexed.diagnostics().is_empty());
        assert_eq!(first, second);
        assert!(first.ast.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(first.diagnostics[0].code(), DiagnosticCode::ExpectedSyntax);
        assert_eq!(
            first.diagnostics[0].message(),
            "typed bodies are allowed only on `spec` functions"
        );
    }

    #[test]
    fn rejects_nonexact_edition_and_trailing_syntax_with_stable_codes() {
        let (_, _, wrong) = parse_text("edition 02026; module m {}");
        assert_eq!(
            wrong.diagnostics[0].code(),
            DiagnosticCode::UnsupportedSourceEdition
        );
        assert_eq!(
            wrong.diagnostics[0].message(),
            "source edition must be exactly `2026`"
        );

        let (_, _, trailing) = parse_text("edition 2026; module m {} module n {}");
        assert_eq!(
            trailing.diagnostics[0].code(),
            DiagnosticCode::TrailingSyntax
        );
        assert!(trailing.ast.is_none());
    }

    #[test]
    fn reserved_words_are_never_accepted_as_names() {
        for text in [
            "edition 2026; module game {}",
            "edition 2026; module m { spec proof() {} }",
            "edition 2026; module m { impl claim() {} }",
            "edition 2026; module m { spec f() -> proof { 1 } }",
        ] {
            let (_, lexed, parsed) = parse_text(text);
            assert!(lexed.diagnostics().is_empty());
            assert_eq!(parsed.diagnostics[0].code(), DiagnosticCode::ExpectedSyntax);
            assert!(parsed.diagnostics[0].message().contains("identifier"));
            assert!(parsed.ast.is_none());
        }
    }

    #[test]
    fn unicode_is_not_whitespace_and_does_not_destabilize_parsing() {
        let mut sources = SourceMap::new();
        let id = sources
            .add("test.or", "edition\u{00a0}2026; module m {}")
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let first = parse(source, &lexed);
        assert_eq!(lexed.diagnostics().len(), 1);
        assert_eq!(
            lexed.diagnostics()[0].code(),
            DiagnosticCode::UnexpectedCharacter
        );
        let second = parse(source, &lexed);
        assert_eq!(first, second);
        assert!(first.diagnostics.is_empty());
        assert!(first.ast.is_none());
        assert!(first.has_errors());
    }

    #[test]
    fn caps_syntax_diagnostics_with_one_suppression_record() {
        let mut text = String::from("edition 2026; module m {");
        for index in 0..(MAX_PARSE_DIAGNOSTICS_PER_SOURCE + 2) {
            text.push_str(&format!(" 1 spec f{index}() {{}}"));
        }
        text.push_str(" }");
        let (_, lexed, parsed) = parse_text(&text);
        assert!(lexed.diagnostics().is_empty());
        assert_eq!(
            parsed.diagnostics.len(),
            MAX_PARSE_DIAGNOSTICS_PER_SOURCE + 1
        );
        assert_eq!(
            parsed.diagnostics.last().unwrap().code(),
            DiagnosticCode::TooManySyntaxErrors
        );
    }

    #[test]
    fn suppressed_syntax_diagnostics_are_not_constructed() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "").unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let mut parser = Parser::new(
            source,
            lexed.tokens(),
            Limits {
                diagnostics: 0,
                ..Limits::DEFAULT
            },
        );
        parser.diagnostics.try_reserve_exact(1).unwrap();
        let constructed = std::cell::Cell::new(0_usize);

        for _ in 0..2 {
            parser.expected_message("unused", || {
                constructed.set(constructed.get().saturating_add(1));
                String::from("unused")
            });
        }

        assert_eq!(constructed.get(), 0);
        assert_eq!(parser.diagnostics.len(), 1);
        assert_eq!(
            parser.diagnostics[0].code(),
            DiagnosticCode::TooManySyntaxErrors
        );
    }

    #[test]
    fn repeated_parser_resource_failures_do_not_construct_messages() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "").unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
        parser.diagnostics.try_reserve_exact(1).unwrap();
        let conversions = std::cell::Cell::new(0_usize);
        let span = source.lexer_span(0, 0);

        parser.resource_limit_at(CountedMessage(&conversions), span);
        parser.resource_limit_at(CountedMessage(&conversions), span);

        assert_eq!(conversions.get(), 1);
        assert_eq!(parser.diagnostics.len(), 1);
        assert_eq!(
            parser.diagnostics[0].code(),
            DiagnosticCode::ParserResourceLimit
        );
    }

    #[test]
    fn complete_diagnostic_bound_requires_no_capacity_growth() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "").unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
        assert!((parser.reserve_diagnostic_slots)(
            &mut parser.diagnostics,
            MAX_RETAINED_PARSE_DIAGNOSTICS,
        ));
        let initial_capacity = parser.diagnostics.capacity();

        for _ in 0..=MAX_PARSE_DIAGNOSTICS_PER_SOURCE {
            parser.report(
                DiagnosticCode::ExpectedSyntax,
                "synthetic syntax error",
                parser.current_span(),
                "synthetic label",
                "synthetic note",
            );
        }
        parser.resource_limit(String::from("synthetic resource failure"));

        assert_eq!(parser.diagnostics.len(), MAX_RETAINED_PARSE_DIAGNOSTICS);
        assert_eq!(parser.diagnostics.capacity(), initial_capacity);
    }

    #[test]
    fn bounds_recovery_delimiter_depth() {
        let text = format!(
            "edition 2026; module m {{ {} x }}",
            "{".repeat(MAX_RECOVERY_DELIMITER_DEPTH + 1)
        );
        let (_, _, parsed) = parse_text(&text);
        assert!(
            parsed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code() == DiagnosticCode::ParserResourceLimit)
        );
        assert!(parsed.ast.is_none());
    }

    #[test]
    fn diagnostic_vector_reservation_failure_returns_no_ast_or_diagnostics() {
        let mut sources = SourceMap::new();
        let id = sources
            .add("test.or", "edition 2026; module m { spec value() {} }")
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
        parser.reserve_diagnostic_slots = |_, _| false;

        let parsed = parser.run();

        assert!(parsed.has_errors());
        assert!(parsed.ast().is_none());
        assert!(parsed.diagnostics().is_empty());
        assert_eq!(parsed.diagnostics.capacity(), 0);
    }

    #[test]
    fn module_function_reservation_failure_returns_no_partial_ast() {
        let mut sources = SourceMap::new();
        let id = sources
            .add("test.or", "edition 2026; module m { spec value() {} }")
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let parse_with_failure = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_function_slot = |_| false;
            parser.run()
        };

        let first = parse_with_failure();
        let second = parse_with_failure();
        assert_eq!(first, second);
        assert!(first.ast().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::ParserResourceLimit);
        assert_eq!(
            diagnostic.message(),
            "parser could not allocate module function storage"
        );
        assert_eq!(
            source.slice(diagnostic.primary_span()),
            Some("spec value() {}")
        );
        assert_eq!(
            diagnostic.label(),
            "deterministic parser resource limit reached"
        );
    }

    #[test]
    fn identifier_reservation_failure_returns_no_partial_ast() {
        let mut sources = SourceMap::new();
        let id = sources
            .add("test.or", "edition 2026; module identifier { }")
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let parse_with_failure = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_identifier_text = |_, _| false;
            parser.run()
        };

        let first = parse_with_failure();
        let second = parse_with_failure();
        assert_eq!(first, second);
        assert!(first.ast().is_none());
        assert_eq!(first.diagnostics().len(), 1);
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::ParserResourceLimit);
        assert_eq!(
            diagnostic.message(),
            "parser could not allocate identifier text storage"
        );
        assert_eq!(source.slice(diagnostic.primary_span()), Some("identifier"));
        assert_eq!(
            diagnostic.label(),
            "deterministic parser resource limit reached"
        );
    }

    #[test]
    fn late_allocation_failures_discard_completed_declarations() {
        let mut slot_sources = SourceMap::new();
        let slot_id = slot_sources
            .add(
                "slot.or",
                "edition 2026; module m { spec first() {} spec second() {} }",
            )
            .unwrap();
        let slot_source = slot_sources.get(slot_id).unwrap();
        let slot_lexed = lex(slot_source, Edition::E2026);
        let mut slot_parser = Parser::new(slot_source, slot_lexed.tokens(), Limits::DEFAULT);
        slot_parser.reserve_function_slot =
            |functions| functions.is_empty() && functions.try_reserve(1).is_ok();

        let slot_failure = slot_parser.run();

        assert!(slot_failure.ast().is_none());
        assert_eq!(slot_failure.diagnostics().len(), 1);
        assert_eq!(
            slot_failure.diagnostics()[0].message(),
            "parser could not allocate module function storage"
        );
        assert_eq!(
            slot_source.slice(slot_failure.diagnostics()[0].primary_span()),
            Some("spec second() {}")
        );

        let mut identifier_sources = SourceMap::new();
        let identifier_id = identifier_sources
            .add(
                "identifier.or",
                concat!(
                    "edition 2026; module m {\n",
                    "  spec a() -> Word[8] { 1 }\n",
                    "  spec b() -> Int { 2 }\n",
                    "}\n",
                ),
            )
            .unwrap();
        let identifier_source = identifier_sources.get(identifier_id).unwrap();
        let identifier_lexed = lex(identifier_source, Edition::E2026);
        let mut identifier_parser = Parser::new(
            identifier_source,
            identifier_lexed.tokens(),
            Limits::DEFAULT,
        );
        identifier_parser.reserve_identifier_text =
            |text, bytes| bytes != "Int".len() && text.try_reserve_exact(bytes).is_ok();

        let identifier_failure = identifier_parser.run();

        assert!(identifier_failure.ast().is_none());
        assert_eq!(identifier_failure.diagnostics().len(), 1);
        assert_eq!(
            identifier_failure.diagnostics()[0].message(),
            "parser could not allocate identifier text storage"
        );
        assert_eq!(
            identifier_source.slice(identifier_failure.diagnostics()[0].primary_span()),
            Some("Int")
        );
    }

    #[test]
    fn enforces_internal_event_and_node_limits_without_large_inputs() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "edition 2026; module m {}").unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);

        let event_limited = Parser::new(
            source,
            lexed.tokens(),
            Limits {
                events: 2,
                ..Limits::DEFAULT
            },
        )
        .run();
        assert!(event_limited.diagnostics.iter().any(|diagnostic| {
            diagnostic.code() == DiagnosticCode::ParserResourceLimit
                && diagnostic.message().contains("event")
        }));

        let node_limited = Parser::new(
            source,
            lexed.tokens(),
            Limits {
                nodes: 1,
                ..Limits::DEFAULT
            },
        )
        .run();
        assert!(node_limited.diagnostics.iter().any(|diagnostic| {
            diagnostic.code() == DiagnosticCode::ParserResourceLimit
                && diagnostic.message().contains("syntax-node")
        }));
    }

    #[test]
    fn parser_is_repeatable_and_malformed_corpus_never_panics() {
        let corpus = [
            "",
            "edition",
            "edition 2026;",
            "edition 2026; module",
            "edition 2026; module m {{{{{[[[(((",
            "edition 2026; module m { spec f(((((((( }",
            "edition 2026; module m { game proof claim } garbage",
            "edition 2026; module m { spec f() { \u{1f7e0} } }",
            "edition 2026; module m { } } } }",
        ];

        for text in corpus {
            let mut sources = SourceMap::new();
            let id = sources.add("corpus.or", text).unwrap();
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::E2026);
            let first = parse(source, &lexed);
            let second = parse(source, &lexed);
            assert_eq!(first, second, "nondeterministic parse for {text:?}");
            assert!(
                first.ast.is_none() || !lexed.diagnostics().is_empty(),
                "malformed corpus produced an error-free AST for {text:?}"
            );
            assert!(
                first
                    .diagnostics
                    .iter()
                    .all(|diagnostic| { source.slice(diagnostic.primary_span()).is_some() })
            );
        }
    }

    #[test]
    fn parser_rejects_a_synthetic_token_stream_above_the_lexical_cap() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "").unwrap();
        let source = sources.get(id).unwrap();
        let span = source.span(TextOffset::new(0), TextOffset::new(0)).unwrap();
        let tokens = vec![
            Token {
                kind: TokenKind::Eof,
                span
            };
            MAX_TOKENS_PER_SOURCE + 2
        ];
        let parsed = Parser::new(source, &tokens, Limits::DEFAULT).run();
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code(),
            DiagnosticCode::ParserResourceLimit
        );
    }

    #[test]
    fn rejects_a_lexer_result_owned_by_another_source() {
        let text = "edition 2026; module m {}";
        let mut first_sources = SourceMap::new();
        let first_id = first_sources.add("first.or", text).unwrap();
        let foreign = lex(first_sources.get(first_id).unwrap(), Edition::E2026);

        let mut second_sources = SourceMap::new();
        let second_id = second_sources.add("second.or", text).unwrap();
        let source = second_sources.get(second_id).unwrap();
        let first = parse(source, &foreign);
        let second = parse(source, &foreign);

        assert_eq!(first, second);
        assert!(first.ast.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::InvalidParserInput
        );
        assert_eq!(first.diagnostics[0].primary_span().source(), source.id());
        assert!(first.diagnostics[0].primary_span().is_empty());
        assert_eq!(
            crate::diagnostic::render_diagnostics(&second_sources, &first.diagnostics),
            concat!(
                "error[ORC0107]: parser received lexer output owned by another source\n",
                " --> second.or:1:1\n",
                "  |\n",
                "1 | edition 2026; module m {}\n",
                "  | ^ parsing stopped at this source boundary\n",
                "  = note: lex and parse each token stream with the same source file\n",
            )
        );
    }

    #[test]
    fn foreign_input_diagnostic_reservation_failure_remains_fail_closed() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "edition 2026; module m {}").unwrap();
        let source = sources.get(id).unwrap();

        let result = invalid_parser_input(source, |_| false);

        assert!(result.has_errors());
        assert!(result.ast().is_none());
        assert!(result.diagnostics().is_empty());
        assert_eq!(result.diagnostics.capacity(), 0);
    }

    #[test]
    fn rejects_a_lexically_erroneous_result_owned_by_another_source() {
        let text = "edition 2026; module m { @ }";
        let mut first_sources = SourceMap::new();
        let first_id = first_sources.add("first.or", text).unwrap();
        let foreign = lex(first_sources.get(first_id).unwrap(), Edition::E2026);
        assert!(foreign.has_errors());

        let mut second_sources = SourceMap::new();
        let second_id = second_sources.add("second.or", text).unwrap();
        let source = second_sources.get(second_id).unwrap();
        let first = parse(source, &foreign);
        let second = parse(source, &foreign);

        assert_eq!(first, second);
        assert!(first.ast.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::InvalidParserInput
        );
        assert_eq!(first.diagnostics[0].primary_span().source(), source.id());
        assert_eq!(
            first.diagnostics[0].message(),
            "parser received lexer output owned by another source"
        );
    }

    #[test]
    fn rejects_structurally_invalid_internal_token_streams() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "edition 2026; module m {}").unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let valid = lexed.tokens().to_vec();

        let mut missing_eof = valid.clone();
        missing_eof.pop();
        let mut early_eof = valid.clone();
        early_eof.insert(0, *valid.last().unwrap());
        let mut nonempty_eof = valid.clone();
        nonempty_eof.last_mut().unwrap().span = valid[0].span;
        let mut overlapping = valid.clone();
        overlapping[1].span = valid[0].span;

        for tokens in [
            Vec::new(),
            missing_eof,
            early_eof,
            nonempty_eof,
            overlapping,
        ] {
            let parsed = Parser::new(source, &tokens, Limits::DEFAULT).run();
            assert!(parsed.ast.is_none());
            assert_eq!(parsed.diagnostics.len(), 1);
            assert_eq!(
                parsed.diagnostics[0].code(),
                DiagnosticCode::ParserResourceLimit
            );
        }
    }

    #[test]
    fn operator_inventories_spellings_and_tokens_are_exact() {
        assert_eq!(
            UnaryOperator::ALL
                .iter()
                .map(|operator| operator.as_str())
                .collect::<Vec<_>>(),
            ["-", "~"]
        );
        assert_eq!(
            BinaryOperator::ALL
                .iter()
                .map(|operator| operator.as_str())
                .collect::<Vec<_>>(),
            ["+", "-", "*", "&", "|", "^", "<<", ">>", "<<<", ">>>"]
        );
        for operator in BinaryOperator::ALL {
            assert_eq!(
                BinaryOperator::from_token(operator.token_kind()),
                Some(*operator)
            );
            assert_eq!(
                operator.is_shift_or_rotation(),
                operator.as_str().starts_with("<<") || operator.as_str().starts_with(">>"),
                "{operator:?}"
            );
        }
        for kind in [
            TokenKind::Tilde,
            TokenKind::Arrow,
            TokenKind::Comma,
            TokenKind::LeftParen,
            TokenKind::Identifier,
            TokenKind::Integer,
        ] {
            assert_eq!(BinaryOperator::from_token(kind), None, "{kind:?}");
        }
    }

    /// Wraps `body` in a four-parameter `Word[32]` spec named `f`.
    fn spec_source(body: &str) -> String {
        format!(
            "edition 2026; module m {{ \
             spec f(a: Word[32], b: Word[32], c: Word[32], d: Word[32]) -> Word[32] {{ {body} }} }}"
        )
    }

    fn body_expression(text: &str) -> (SourceMap, Expression) {
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty(), "{text:?}");
        assert!(
            parsed.diagnostics.is_empty(),
            "{text:?}: {:?}",
            parsed.diagnostics
        );
        let function = parsed.ast.unwrap().module.functions.pop().unwrap();
        let FunctionBody::Typed(body) = function.body else {
            panic!("expected a typed body in {text:?}");
        };
        (sources, body.expression)
    }

    /// Renders an expression tree with explicit structure: binary and unary
    /// nodes in parentheses, source groups in brackets, and literals exactly
    /// as spelled.
    fn shape(source: &SourceFile, expression: &Expression) -> String {
        match &expression.kind {
            ExpressionKind::Literal(literal) => source.slice(literal.span).unwrap().to_owned(),
            ExpressionKind::Name(name) => name.text.clone(),
            ExpressionKind::Call(call) => format!(
                "{}({})",
                call.callee.text,
                call.arguments
                    .iter()
                    .map(|argument| shape(source, argument))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ExpressionKind::Unary(unary) => format!(
                "({}{})",
                unary.operator.as_str(),
                shape(source, &unary.operand)
            ),
            ExpressionKind::Binary(binary) => format!(
                "({} {} {})",
                shape(source, &binary.left),
                binary.operator.as_str(),
                shape(source, &binary.right)
            ),
            ExpressionKind::Parenthesized(inner) => format!("[{}]", shape(source, inner)),
            ExpressionKind::Conversion(conversion) => format!(
                "({} as {})",
                shape(source, &conversion.operand),
                source.slice(conversion.target.span).unwrap()
            ),
        }
    }

    fn tree_height(expression: &Expression) -> usize {
        1 + match &expression.kind {
            ExpressionKind::Literal(_) | ExpressionKind::Name(_) => 0,
            ExpressionKind::Call(call) => call.arguments.iter().map(tree_height).max().unwrap_or(0),
            ExpressionKind::Unary(unary) => tree_height(&unary.operand),
            ExpressionKind::Binary(binary) => {
                tree_height(&binary.left).max(tree_height(&binary.right))
            }
            ExpressionKind::Parenthesized(inner) => tree_height(inner),
            ExpressionKind::Conversion(conversion) => tree_height(&conversion.operand),
        }
    }

    #[test]
    fn builds_parameters_calls_and_operators_with_exact_spans() {
        let text = concat!(
            "edition 2026; module sha { ",
            "spec big_sigma0(x: Word[32]) -> Word[32] ",
            "{ (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) } ",
            "spec apply(x: Word[32], y: Int,) -> Word[32] { big_sigma0(~x) } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();

        let sigma = &ast.module.functions[0];
        assert_eq!(sigma.parameters.len(), 1);
        assert_eq!(source.slice(sigma.parameters[0].span), Some("x: Word[32]"));
        assert_eq!(sigma.parameters[0].name.text, "x");
        assert_eq!(source.slice(sigma.parameters[0].ty.span), Some("Word[32]"));
        let FunctionBody::Typed(body) = &sigma.body else {
            panic!("expected a typed body");
        };
        assert_eq!(
            source.slice(body.span),
            Some("-> Word[32] { (x >>> 2) ^ (x >>> 13) ^ (x >>> 22) }")
        );
        assert_eq!(
            source.slice(body.expression.span),
            Some("(x >>> 2) ^ (x >>> 13) ^ (x >>> 22)")
        );
        assert_eq!(
            shape(source, &body.expression),
            "(([(x >>> 2)] ^ [(x >>> 13)]) ^ [(x >>> 22)])"
        );
        let ExpressionKind::Binary(outer) = &body.expression.kind else {
            panic!("expected a binary root");
        };
        assert_eq!(outer.operator, BinaryOperator::Xor);
        assert_eq!(source.slice(outer.operator_span), Some("^"));
        assert_eq!(
            outer.operator_span.start(),
            TextOffset::new(u32::try_from(text.rfind('^').unwrap()).unwrap())
        );
        assert_eq!(
            source.slice(outer.left.span),
            Some("(x >>> 2) ^ (x >>> 13)")
        );
        assert_eq!(source.slice(outer.right.span), Some("(x >>> 22)"));
        let ExpressionKind::Parenthesized(group) = &outer.right.kind else {
            panic!("expected a parenthesized right operand");
        };
        assert_eq!(source.slice(group.span), Some("x >>> 22"));
        let ExpressionKind::Binary(rotation) = &group.kind else {
            panic!("expected a rotation");
        };
        assert_eq!(rotation.operator, BinaryOperator::RotateRight);
        assert_eq!(source.slice(rotation.operator_span), Some(">>>"));

        let apply = &ast.module.functions[1];
        assert_eq!(
            apply
                .parameters
                .iter()
                .map(|parameter| (
                    parameter.name.text.as_str(),
                    source.slice(parameter.ty.span).unwrap()
                ))
                .collect::<Vec<_>>(),
            [("x", "Word[32]"), ("y", "Int")]
        );
        let FunctionBody::Typed(apply_body) = &apply.body else {
            panic!("expected a typed body");
        };
        let ExpressionKind::Call(call) = &apply_body.expression.kind else {
            panic!("expected a call");
        };
        assert_eq!(call.callee.text, "big_sigma0");
        assert_eq!(source.slice(call.callee.span), Some("big_sigma0"));
        assert_eq!(
            source.slice(apply_body.expression.span),
            Some("big_sigma0(~x)")
        );
        assert_eq!(call.arguments.len(), 1);
        let ExpressionKind::Unary(complement) = &call.arguments[0].kind else {
            panic!("expected a complement argument");
        };
        assert_eq!(complement.operator, UnaryOperator::Complement);
        assert_eq!(source.slice(complement.operator_span), Some("~"));
        assert_eq!(source.slice(call.arguments[0].span), Some("~x"));
    }

    #[test]
    fn groups_arithmetic_by_precedence_and_every_operator_leftward() {
        let cases = [
            ("a + b * c - d", "((a + (b * c)) - d)"),
            ("a * b + c", "((a * b) + c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a - b - c", "((a - b) - c)"),
            ("a - b + c * d * a", "((a - b) + ((c * d) * a))"),
            ("a ^ b ^ c", "((a ^ b) ^ c)"),
            ("a & b & c & d", "(((a & b) & c) & d)"),
            ("a | b | c", "((a | b) | c)"),
            ("(a ^ b) & c", "([(a ^ b)] & c)"),
            ("a & (b | c)", "(a & [(b | c)])"),
            ("(a + b) * c", "([(a + b)] * c)"),
            ("a << 3", "(a << 3)"),
            ("a >> 0x1f", "(a >> 0x1f)"),
            ("a <<< 7", "(a <<< 7)"),
            ("a >>> 1_0", "(a >>> 1_0)"),
            ("(a <<< 7) ^ b", "([(a <<< 7)] ^ b)"),
            ("a << (1 + 2)", "(a << [(1 + 2)])"),
            ("~a & b", "((~a) & b)"),
            ("-a * b", "((-a) * b)"),
            ("-1 * a", "(-1 * a)"),
            ("- 1", "- 1"),
            ("a - -1", "(a - -1)"),
            ("a -1", "(a - 1)"),
            ("a - - b", "(a - (-b))"),
            ("- -a", "(-(-a))"),
            ("~~a", "(~(~a))"),
            ("-(a)", "(-[a])"),
            ("((a))", "[[a]]"),
            ("g()", "g()"),
            ("g(a, b + 1, h())", "g(a, (b + 1), h())"),
            ("g(a, b,)", "g(a, b)"),
            ("g(h(i(a)))", "g(h(i(a)))"),
            ("g(a ^ b, (c))", "g((a ^ b), [c])"),
            ("g(a) * g(b) + 1", "((g(a) * g(b)) + 1)"),
            ("0", "0"),
            ("a", "a"),
        ];
        for (body, expected) in cases {
            let (sources, expression) = body_expression(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert_eq!(shape(source, &expression), expected, "{body:?}");
            assert_eq!(source.slice(expression.span), Some(body), "{body:?}");
        }
    }

    #[test]
    fn requires_parentheses_between_operator_groups_with_one_diagnostic() {
        let cases = [
            ("a ^ b & c", 6, "&", "^", false),
            ("a & b ^ c", 6, "^", "&", false),
            ("a | b & c", 6, "&", "|", false),
            ("a + b ^ c", 6, "^", "+", false),
            ("a * b & c", 6, "&", "*", false),
            ("a & b + c", 6, "+", "&", false),
            ("a & b * c", 6, "*", "&", false),
            ("a + b * c | d", 10, "|", "*", false),
            ("a >>> 1 + b", 8, "+", ">>>", false),
            ("a ^ b << 1", 6, "<<", "^", false),
            ("a << 1 << 2", 7, "<<", "<<", true),
            ("a <<< 1 >>> 2", 8, ">>>", "<<<", true),
            ("a >> 1 << 2", 7, "<<", ">>", true),
            ("a ^ b & c | d + a", 6, "&", "^", false),
            ("g(a ^ b & c)", 8, "&", "^", false),
            ("(a ^ b & c)", 7, "&", "^", false),
        ];
        for (body, offset, ungrouped, previous, both_shifts) in cases {
            let text = spec_source(body);
            let (sources, lexed, parsed) = parse_text(&text);
            let source = sources.iter().next().unwrap();
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "{body:?}");
            assert_eq!(
                parsed.diagnostics.len(),
                1,
                "{body:?}: {:?}",
                parsed.diagnostics
            );
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::UngroupedOperators,
                "{body:?}"
            );
            assert_eq!(
                diagnostic.message(),
                format!("`{ungrouped}` follows `{previous}` without grouping parentheses"),
                "{body:?}"
            );
            assert_eq!(diagnostic.label(), "ungrouped operator");
            assert_eq!(source.slice(diagnostic.primary_span()), Some(ungrouped));
            let expected = text.find(body).unwrap() + offset;
            assert_eq!(
                diagnostic.primary_span().start(),
                TextOffset::new(u32::try_from(expected).unwrap()),
                "{body:?}"
            );
            assert_eq!(
                diagnostic.notes(),
                [if both_shifts {
                    "a shift or rotation takes exactly two operands; parenthesize one of them"
                } else {
                    "operators from different groups have no relative precedence in Orange; \
                     parenthesize the part that applies first"
                }],
                "{body:?}"
            );
        }
    }

    #[test]
    fn ungrouped_operators_do_not_cascade_across_functions() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec f(a: Word[8], b: Word[8]) -> Word[8] { a ^ b & a | b } ",
            "spec g(a: Word[8], b: Word[8]) -> Word[8] { a + b ^ a } ",
            "spec h(a: Word[8]) -> Word[8] { (a ^ a) & a } ",
            "}"
        );
        let (sources, _, parsed) = parse_text(text);
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .map(|diagnostic| (
                    diagnostic.code(),
                    source.slice(diagnostic.primary_span()).unwrap()
                ))
                .collect::<Vec<_>>(),
            [
                (DiagnosticCode::UngroupedOperators, "&"),
                (DiagnosticCode::UngroupedOperators, "^"),
            ]
        );
    }

    #[test]
    fn rejects_malformed_parameters_calls_and_expressions() {
        let cases = [
            "spec f(a) -> Int { a }",
            "spec f(a:) -> Int { a }",
            "spec f(: Int) -> Int { 1 }",
            "spec f(a Int) -> Int { a }",
            "spec f(a: Int b: Int) -> Int { a }",
            "spec f(a: Int,,) -> Int { a }",
            "spec f(,) -> Int { 1 }",
            "spec f(a: Word[]) -> Int { 1 }",
            "spec f(spec: Int) -> Int { 1 }",
            "spec f(a: Int) {}",
            "spec f(a: Int) { a }",
            "impl f(a: Int) {}",
            "spec f() -> Int { g(1 2) }",
            "spec f() -> Int { g(,) }",
            "spec f() -> Int { g(1,,) }",
            "spec f() -> Int { g( }",
            "spec f() -> Int { g(1 }",
            "spec f() -> Int { (1 }",
            "spec f() -> Int { () }",
            "spec f() -> Int { 1 + }",
            "spec f() -> Int { * 1 }",
            "spec f() -> Int { ~ }",
            "spec f() -> Int { - }",
            "spec f() -> Int { a b }",
            "spec f() -> Int { 1 (2) }",
            "spec f() -> Int { g h() }",
            "spec f() -> Int { 1 ~ 2 }",
            "spec f() -> Int { 1 } }",
            "spec f() -> Int { 1 + 2 -> }",
            "spec f() -> Int { proof(1) }",
            "spec f() -> Int { claim }",
        ];
        for member in cases {
            let text = format!("edition 2026; module m {{ {member} }}");
            let (_, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{member:?}");
            assert!(parsed.ast.is_none(), "accepted {member:?}");
            assert!(!parsed.diagnostics.is_empty(), "{member:?}");
            assert!(
                parsed.diagnostics.iter().all(|diagnostic| diagnostic.code()
                    == DiagnosticCode::ExpectedSyntax
                    || diagnostic.code() == DiagnosticCode::TrailingSyntax),
                "{member:?}: {:?}",
                parsed.diagnostics
            );
        }
    }

    #[test]
    fn parameters_are_rejected_on_impl_and_untyped_spec_forms() {
        let (sources, _, parsed) = parse_text(concat!(
            "edition 2026; module m { ",
            "impl f(a: Int) {} ",
            "spec g(b: Int) {} ",
            "spec ok() -> Int { 1 } ",
            "}"
        ));
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .map(|diagnostic| (
                    diagnostic.message(),
                    source.slice(diagnostic.primary_span()).unwrap()
                ))
                .collect::<Vec<_>>(),
            [
                ("`impl` functions have an empty parameter list", "a: Int"),
                ("expected `->` after the parameter list", "{"),
            ]
        );
    }

    fn nested(prefix: &str, count: usize, core: &str, suffix: &str) -> String {
        format!("{}{core}{}", prefix.repeat(count), suffix.repeat(count))
    }

    fn assert_resource_limited(body: &str, message: &str, at: &str) {
        let text = spec_source(body);
        let (sources, lexed, parsed) = parse_text(&text);
        let source = sources.iter().next().unwrap();
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1, "{:?}", parsed.diagnostics);
        let diagnostic = &parsed.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::ParserResourceLimit);
        assert_eq!(diagnostic.message(), message);
        assert_eq!(
            source.slice(diagnostic.primary_span()),
            Some(at),
            "{body:?}"
        );
    }

    #[test]
    fn bounds_expression_nesting_for_every_opener() {
        let message = format!(
            "expression nesting exceeds the {MAX_EXPRESSION_NESTING}-level limit \
             for groups, calls, and prefix operators"
        );
        type Form = (&'static str, fn(usize) -> String, &'static str);
        let forms: [Form; 5] = [
            ("groups", |count| nested("(", count, "a", ")"), "("),
            ("complements", |count| nested("~", count, "a", ""), "~"),
            ("negations", |count| nested("-", count, "a", ""), "-"),
            ("calls", |count| nested("g(", count, "a", ")"), "g"),
            (
                "operands",
                |count| nested("a + a * (", count, "a", ")"),
                "(",
            ),
        ];
        for (name, build, opener) in forms {
            let (_, expression) = body_expression(&spec_source(&build(MAX_EXPRESSION_NESTING)));
            assert!(tree_height(&expression) > MAX_EXPRESSION_NESTING, "{name}");
            assert_resource_limited(&build(MAX_EXPRESSION_NESTING + 1), &message, opener);
        }

        // Every opener draws on one budget.
        let mixed = |groups: usize, complements: usize| {
            nested("(", groups, &nested("~", complements, "a", ""), ")")
        };
        let (_, expression) = body_expression(&spec_source(&mixed(32, 32)));
        assert_eq!(tree_height(&expression), 65);
        assert_resource_limited(&mixed(32, 33), &message, "~");
        assert_resource_limited(&mixed(33, 32), &message, "~");

        // Sibling groups do not nest.
        let siblings = vec![nested("(", MAX_EXPRESSION_NESTING, "a", ")"); 4].join(" ^ ");
        let (_, expression) = body_expression(&spec_source(&siblings));
        assert_eq!(tree_height(&expression), MAX_EXPRESSION_NESTING + 4);

        // Wide calls are not deep.
        let wide = format!("g({})", vec!["a"; MAX_ARGUMENTS_PER_CALL].join(", "));
        let (_, expression) = body_expression(&spec_source(&wide));
        assert_eq!(tree_height(&expression), 2);
    }

    #[test]
    fn bounds_expression_tree_height_for_operator_chains() {
        let message =
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit");
        let chains: [(&str, &str); 4] =
            [(" ^ a", "^"), (" + a", "+"), (" * a", "*"), (" - 1", "-")];
        for (link, operator) in chains {
            let chain = |count: usize| format!("a{}", link.repeat(count));
            let (_, expression) = body_expression(&spec_source(&chain(MAX_EXPRESSION_HEIGHT - 1)));
            assert_eq!(tree_height(&expression), MAX_EXPRESSION_HEIGHT, "{link:?}");
            assert_resource_limited(&chain(MAX_EXPRESSION_HEIGHT), &message, operator);
        }

        // A tall operand raises the height of every node above it.
        let tall = format!("(a{})", " ^ a".repeat(MAX_EXPRESSION_HEIGHT - 2));
        let (_, expression) = body_expression(&spec_source(&tall));
        assert_eq!(tree_height(&expression), MAX_EXPRESSION_HEIGHT);
        for (body, at) in [
            (format!("a | {tall}"), "|"),
            (format!("{tall} * a"), "*"),
            (format!("({tall})"), "("),
            (format!("g({tall})"), "g"),
            (format!("~{tall}"), "~"),
            (format!("{tall} as Int"), "as"),
        ] {
            assert_resource_limited(&body, &message, at);
        }
    }

    #[test]
    fn bounds_parameters_per_function_and_arguments_per_call() {
        let parameters = |count: usize| {
            (0..count)
                .map(|index| format!("p{index}: Int"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let accepted = format!(
            "edition 2026; module m {{ spec f({}) -> Int {{ 1 }} }}",
            parameters(MAX_PARAMETERS_PER_FUNCTION)
        );
        let (_, _, parsed) = parse_text(&accepted);
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(
            parsed.ast.unwrap().module.functions[0].parameters.len(),
            MAX_PARAMETERS_PER_FUNCTION
        );

        let rejected = format!(
            "edition 2026; module m {{ spec f({}) -> Int {{ 1 }} }}",
            parameters(MAX_PARAMETERS_PER_FUNCTION + 1)
        );
        let (sources, _, parsed) = parse_text(&rejected);
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code(),
            DiagnosticCode::ParserResourceLimit
        );
        assert_eq!(
            parsed.diagnostics[0].message(),
            format!("function declares more than {MAX_PARAMETERS_PER_FUNCTION} parameters")
        );
        assert_eq!(
            source.slice(parsed.diagnostics[0].primary_span()),
            Some(format!("p{MAX_PARAMETERS_PER_FUNCTION}: Int").as_str())
        );

        let call = |count: usize| spec_source(&format!("g({})", vec!["1"; count].join(", ")));
        let (_, expression) = body_expression(&call(MAX_ARGUMENTS_PER_CALL));
        let ExpressionKind::Call(parsed_call) = expression.kind else {
            panic!("expected a call");
        };
        assert_eq!(parsed_call.arguments.len(), MAX_ARGUMENTS_PER_CALL);

        let (_, _, parsed) = parse_text(&call(MAX_ARGUMENTS_PER_CALL + 1));
        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code(),
            DiagnosticCode::ParserResourceLimit
        );
        assert_eq!(
            parsed.diagnostics[0].message(),
            format!("call supplies more than {MAX_ARGUMENTS_PER_CALL} arguments")
        );
    }

    #[test]
    fn parameter_and_argument_reservation_failures_return_no_partial_ast() {
        let mut sources = SourceMap::new();
        let id = sources
            .add(
                "test.or",
                "edition 2026; module m { spec f(x: Int) -> Int { g(x) } }",
            )
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);

        let parameter_failure = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_parameter_slot = |_| false;
            parser.run()
        };
        let argument_failure = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_argument_slot = |_| false;
            parser.run()
        };
        for (run, message, span) in [
            (
                &parameter_failure as &dyn Fn() -> ParseResult,
                "parser could not allocate parameter storage",
                "x: Int",
            ),
            (
                &argument_failure,
                "parser could not allocate argument storage",
                "x",
            ),
        ] {
            let first = run();
            assert_eq!(first, run());
            assert!(first.ast.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            let diagnostic = &first.diagnostics[0];
            assert_eq!(diagnostic.code(), DiagnosticCode::ParserResourceLimit);
            assert_eq!(diagnostic.message(), message);
            assert_eq!(source.slice(diagnostic.primary_span()), Some(span));
        }
    }

    #[test]
    fn builds_bindings_and_conversions_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec pack(a: Word[8], b: Word[8]) -> Word[32] { ",
            "let wide: Word[32] = a as Word[32]; ",
            "let t: Word[32] = (wide << 8) | (b as Word[32]); ",
            "t } ",
            "spec plain() -> Int { 1 } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let FunctionBody::Typed(body) = &ast.module.functions[0].body else {
            panic!("expected a typed body");
        };
        assert_eq!(
            source.slice(body.span),
            Some(concat!(
                "-> Word[32] { let wide: Word[32] = a as Word[32]; ",
                "let t: Word[32] = (wide << 8) | (b as Word[32]); t }"
            ))
        );
        assert_eq!(body.bindings().len(), 2);
        let wide = &body.bindings()[0];
        assert_eq!(
            source.slice(wide.span()),
            Some("let wide: Word[32] = a as Word[32];")
        );
        assert_eq!(wide.name().text, "wide");
        assert_eq!(source.slice(wide.name().span), Some("wide"));
        assert_eq!(source.slice(wide.ty().span), Some("Word[32]"));
        assert_eq!(source.slice(wide.value().span), Some("a as Word[32]"));
        let ExpressionKind::Conversion(conversion) = &wide.value().kind else {
            panic!("expected a conversion");
        };
        assert_eq!(source.slice(conversion.keyword_span()), Some("as"));
        assert_eq!(source.slice(conversion.operand().span), Some("a"));
        assert_eq!(source.slice(conversion.target().span), Some("Word[32]"));
        assert_eq!(conversion.target().name.text, "Word");
        assert_eq!(
            conversion
                .target()
                .width_span
                .and_then(|span| source.slice(span)),
            Some("32")
        );

        let t = &body.bindings()[1];
        assert_eq!(
            source.slice(t.span()),
            Some("let t: Word[32] = (wide << 8) | (b as Word[32]);")
        );
        assert_eq!(
            shape(source, t.value()),
            "([(wide << 8)] | [(b as Word[32])])"
        );
        assert_eq!(shape(source, body.expression()), "t");
        assert_eq!(source.slice(body.expression().span), Some("t"));

        let FunctionBody::Typed(plain) = &ast.module.functions[1].body else {
            panic!("expected a typed body");
        };
        assert!(plain.bindings().is_empty());
    }

    #[test]
    fn conversions_apply_to_one_complete_operand() {
        let cases = [
            ("a as Int", "(a as Int)"),
            ("a as Word", "(a as Word)"),
            ("-a as Int", "((-a) as Int)"),
            ("~a as Word[8]", "((~a) as Word[8])"),
            ("-1 as Word[8]", "(-1 as Word[8])"),
            ("g(a) as Int", "(g(a) as Int)"),
            ("(a + b) as Word[8]", "([(a + b)] as Word[8])"),
            ("(a as Int) as Word[8]", "([(a as Int)] as Word[8])"),
            ("(a as Word[64]) + b", "([(a as Word[64])] + b)"),
            ("a ^ (b as Word[32])", "(a ^ [(b as Word[32])])"),
            ("g(a as Int, b as Int)", "g((a as Int), (b as Int))"),
            ("(a as Word[32]) << 8", "([(a as Word[32])] << 8)"),
        ];
        for (body, expected) in cases {
            let (sources, expression) = body_expression(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert_eq!(shape(source, &expression), expected, "{body:?}");
            assert_eq!(source.slice(expression.span), Some(body), "{body:?}");
        }
        // A conversion is one level above its operand.
        let (_, expression) = body_expression(&spec_source("a as Int"));
        assert_eq!(tree_height(&expression), 2);
    }

    #[test]
    fn let_and_as_remain_ordinary_names() {
        let cases = [
            ("let", "let"),
            ("as", "as"),
            ("let + as", "(let + as)"),
            ("let(as)", "let(as)"),
            ("as as Int", "(as as Int)"),
            // `let` before a name always starts a binding, so a conversion
            // of a name `let` groups it.
            ("(let) as Int", "([let] as Int)"),
            ("g(let, as)", "g(let, as)"),
        ];
        for (body, expected) in cases {
            let (sources, expression) = body_expression(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert_eq!(shape(source, &expression), expected, "{body:?}");
        }
        let text = concat!(
            "edition 2026; module m { ",
            "spec let(as: Int) -> Int { let let: Int = as; let as2: Int = let; let(as2) } ",
            "}"
        );
        let (sources, _, parsed) = parse_text(text);
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let function = &parsed.ast.unwrap().module.functions[0];
        assert_eq!(function.name.text, "let");
        let FunctionBody::Typed(body) = &function.body else {
            panic!("expected a typed body");
        };
        assert_eq!(
            body.bindings()
                .iter()
                .map(|binding| (binding.name().text.as_str(), shape(source, binding.value())))
                .collect::<Vec<_>>(),
            [("let", String::from("as")), ("as2", String::from("let"))]
        );
        assert_eq!(shape(source, body.expression()), "let(as2)");
    }

    #[test]
    fn requires_parentheses_around_conversions_with_one_diagnostic() {
        let cases = [
            ("a as Int as Word[8]", 9, "as", "as"),
            ("a as Word[32] + b", 14, "+", "as"),
            ("a + b as Int", 6, "as", "+"),
            ("a * b as Int", 6, "as", "*"),
            ("a ^ b as Int", 6, "as", "^"),
            ("a << 1 as Int", 7, "as", "<<"),
            ("a >>> 1 as Int", 8, "as", ">>>"),
            ("a as Int << 1", 9, "<<", "as"),
            ("g(a as Int + b)", 11, "+", "as"),
            ("(a + b as Int)", 7, "as", "+"),
        ];
        for (body, offset, ungrouped, previous) in cases {
            let text = spec_source(body);
            let (sources, lexed, parsed) = parse_text(&text);
            let source = sources.iter().next().unwrap();
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "{body:?}");
            assert_eq!(
                parsed.diagnostics.len(),
                1,
                "{body:?}: {:?}",
                parsed.diagnostics
            );
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(diagnostic.code(), DiagnosticCode::UngroupedOperators);
            assert_eq!(
                diagnostic.message(),
                format!("`{ungrouped}` follows `{previous}` without grouping parentheses"),
                "{body:?}"
            );
            assert_eq!(source.slice(diagnostic.primary_span()), Some(ungrouped));
            let expected = text.find(body).unwrap() + offset;
            assert_eq!(
                diagnostic.primary_span().start(),
                TextOffset::new(u32::try_from(expected).unwrap()),
                "{body:?}"
            );
            assert_eq!(
                diagnostic.notes(),
                [
                    "`as` converts exactly one operand; parenthesize the conversion or the \
                  expression it converts"
                ],
                "{body:?}"
            );
        }
    }

    #[test]
    fn rejects_malformed_bindings_with_exact_messages() {
        let cases = [
            ("let a = 1; a", "expected `:` and the binding's type"),
            (
                "let a: = 1; a",
                "expected an identifier for the binding type",
            ),
            ("let a: Int 1; a", "expected `=` after the binding's type"),
            ("let a: Int = ; a", "expected an expression"),
            (
                "let a: Int = 1 a",
                "expected `;` after the bound expression",
            ),
            (
                "let a: Int = 1;",
                "expected a result expression after the last binding",
            ),
            (
                "let a: Int = 1; let b: Int = a;",
                "expected a result expression after the last binding",
            ),
            ("a; a", "expected `}` after the body expression"),
            (
                "let a: Int = 1; a; a",
                "expected `}` after the body expression",
            ),
            (
                "a let b: Int = 1; b",
                "expected `}` after the body expression",
            ),
            (
                "let a: Int = let b: Int = 1; a",
                "expected `;` after the bound expression",
            ),
            (
                "let 1: Int = 1; a",
                "expected `}` after the body expression",
            ),
        ];
        for (body, message) in cases {
            let text = format!("edition 2026; module m {{ spec f() -> Int {{ {body} }} }}");
            let (_, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            let diagnostic = parsed.diagnostics.first().unwrap();
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{body:?}"
            );
            assert_eq!(diagnostic.message(), message, "{body:?}");
            assert!(
                parsed
                    .diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.code() == DiagnosticCode::ExpectedSyntax),
                "{body:?}: {:?}",
                parsed.diagnostics
            );
        }
    }

    #[test]
    fn bounds_bindings_per_body() {
        let bindings = |count: usize| {
            (0..count)
                .map(|index| format!("let v{index}: Int = {index}; "))
                .collect::<String>()
        };
        let source_with = |count: usize| {
            format!(
                "edition 2026; module m {{ spec f() -> Int {{ {}0 }} }}",
                bindings(count)
            )
        };
        let (_, _, parsed) = parse_text(&source_with(MAX_BINDINGS_PER_BODY));
        assert!(parsed.diagnostics.is_empty());
        let FunctionBody::Typed(body) = &parsed.ast.unwrap().module.functions[0].body else {
            panic!("expected a typed body");
        };
        assert_eq!(body.bindings().len(), MAX_BINDINGS_PER_BODY);

        let (sources, _, parsed) = parse_text(&source_with(MAX_BINDINGS_PER_BODY + 1));
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
        let diagnostic = &parsed.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::ParserResourceLimit);
        assert_eq!(
            diagnostic.message(),
            format!("typed body declares more than {MAX_BINDINGS_PER_BODY} bindings")
        );
        assert_eq!(
            source.slice(diagnostic.primary_span()),
            Some(format!("let v{MAX_BINDINGS_PER_BODY}: Int = {MAX_BINDINGS_PER_BODY};").as_str())
        );
    }

    #[test]
    fn binding_reservation_failure_returns_no_partial_ast() {
        let mut sources = SourceMap::new();
        let id = sources
            .add(
                "test.or",
                "edition 2026; module m { spec f(x: Int) -> Int { let t: Int = x; t } }",
            )
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let run = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_binding_slot = |_| false;
            parser.run()
        };
        let first = run();
        assert_eq!(first, run());
        assert!(first.ast.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        let diagnostic = &first.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::ParserResourceLimit);
        assert_eq!(
            diagnostic.message(),
            "parser could not allocate binding storage"
        );
        assert_eq!(
            source.slice(diagnostic.primary_span()),
            Some("let t: Int = x;")
        );
    }

    #[test]
    fn expression_parsing_is_repeatable_and_malformed_expressions_never_panic() {
        let bodies = [
            "(((((",
            ")))))",
            "a ^ ^ b",
            "a <<<< 1",
            "a >>>> 1",
            "g(((a, b), c)",
            "~-~-~-",
            "a + b ^ c & d | e << 1 >>> 2",
            "g(g(g(g(g(",
            "1 2 3 4",
            ",,,,",
            "a -> b",
            "a :: b",
            "a as",
            "as as as as",
            "let let let",
            "let a: Int = let a: Int = a;",
            "(a as Word[32] as",
        ];
        for body in bodies {
            let mut sources = SourceMap::new();
            let id = sources.add("test.or", spec_source(body)).unwrap();
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::E2026);
            let first = parse(source, &lexed);
            assert_eq!(first, parse(source, &lexed), "{body:?}");
            assert!(first.ast.is_none(), "{body:?}");
        }
    }
}
