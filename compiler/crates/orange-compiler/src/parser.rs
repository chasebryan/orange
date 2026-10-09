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

/// Maximum elements written in one array literal, which is also the longest
/// admitted array type.
pub const MAX_ARRAY_ELEMENTS: usize = 65_536;

/// Maximum elements of a tuple type, a tuple expression, or a tuple
/// pattern. A tuple has at least two.
pub const MAX_TUPLE_ELEMENTS: usize = 16;

/// Maximum `use` declarations in one module.
pub const MAX_USES_PER_MODULE: usize = 64;

/// Maximum `type` declarations in one module.
pub const MAX_TYPES_PER_MODULE: usize = 64;

/// Maximum size parameters of one function, and so sizes of one call.
pub const MAX_SIZES_PER_FUNCTION: usize = 4;

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
    /// `use` declarations in source order, all before the functions.
    pub(crate) uses: Vec<UseDeclaration>,
    /// `type` declarations in source order, after the `use` declarations and
    /// before the functions.
    pub(crate) types: Vec<TypeDeclaration>,
    /// Functions in source order.
    pub(crate) functions: Vec<FunctionDeclaration>,
    /// Known-answer tests in source order, which may stand among the
    /// functions.
    pub(crate) tests: Vec<TestDeclaration>,
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

    /// Returns the `use` declarations in source order.
    #[must_use]
    pub fn uses(&self) -> &[UseDeclaration] {
        &self.uses
    }

    /// Returns the `type` declarations in source order.
    #[must_use]
    pub fn types(&self) -> &[TypeDeclaration] {
        &self.types
    }

    /// Returns functions in source order.
    #[must_use]
    pub fn functions(&self) -> &[FunctionDeclaration] {
        &self.functions
    }

    /// Returns the known-answer tests in source order.
    #[must_use]
    pub fn tests(&self) -> &[TestDeclaration] {
        &self.tests
    }
}

/// A known-answer test, `test "TITLE" { BINDINGS EXPRESSION }`: a closed
/// `Bool` expression, named by its title, that `orangec test` evaluates and
/// that no function calls.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestDeclaration {
    /// Full extent, from `test` through the closing brace.
    pub(crate) span: Span,
    /// The title between the quotes.
    pub(crate) title: TestTitle,
    /// The test as a typed `spec` without parameters whose result type is
    /// `Bool`, named `test` at the title's extent.
    pub(crate) function: FunctionDeclaration,
}

impl TestDeclaration {
    /// Returns the full extent, from `test` through the closing brace.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the title.
    #[must_use]
    pub const fn title(&self) -> &TestTitle {
        &self.title
    }

    /// Returns the body: its `let` bindings and the `Bool` expression that
    /// decides the test.
    #[must_use]
    pub fn body(&self) -> Option<&TypedBody> {
        match &self.function.body {
            FunctionBody::Typed(body) => Some(body),
            FunctionBody::Empty => None,
        }
    }

    /// Returns the test as a function without parameters whose result type
    /// is `Bool`.
    pub(crate) const fn function(&self) -> &FunctionDeclaration {
        &self.function
    }
}

/// The title of a known-answer test, as written between its quotes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestTitle {
    /// The exact spelling between the quotes.
    pub(crate) text: String,
    /// Extent of the title, including its quotes.
    pub(crate) span: Span,
}

impl TestTitle {
    /// Returns the exact spelling between the quotes.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the extent of the title, including its quotes.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }
}

/// A `use NAME;` declaration, which names another module whose typed `spec`
/// functions this module calls as `NAME::function(...)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UseDeclaration {
    /// Full declaration extent, from `use` through the semicolon.
    pub(crate) span: Span,
    /// The used module's name.
    pub(crate) name: Identifier,
}

impl UseDeclaration {
    /// Returns the full declaration extent.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the used module's name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }
}

/// A `type NAME = TYPE;` declaration, which names a type for the rest of its
/// module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeDeclaration {
    /// Full declaration extent, from `type` through the semicolon.
    pub(crate) span: Span,
    /// The declared name.
    pub(crate) name: Identifier,
    /// The type it names.
    pub(crate) ty: TypeSyntax,
}

impl TypeDeclaration {
    /// Returns the full declaration extent.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the declared name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the type the name stands for.
    #[must_use]
    pub const fn ty(&self) -> &TypeSyntax {
        &self.ty
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
    /// Size parameters in source order; nonempty only for a sized `spec`.
    pub(crate) sizes: Vec<SizeParameter>,
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

    /// Returns the size parameters in source order.
    #[must_use]
    pub fn sizes(&self) -> &[SizeParameter] {
        &self.sizes
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

/// One size parameter `n in a..b` of a sized `spec`, or one type
/// parameter `K in {F, L}`. The function is checked once for each value of
/// `n` from `a` up to, but not including, `b`, or once for each type the
/// braces list, as if it were written out once for each.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SizeParameter {
    /// Extent from the name through the second bound or the closing brace.
    pub(crate) span: Span,
    /// The size's or the type parameter's name.
    pub(crate) name: Identifier,
    /// Exact extent of the first bound's integer token, or of `{`.
    pub(crate) start_span: Span,
    /// Exact extent of the second bound's integer token, or of `}`.
    pub(crate) end_span: Span,
    /// The listed types in source order, nonempty only for a type
    /// parameter.
    pub(crate) types: Vec<TypeSyntax>,
}

impl SizeParameter {
    /// Returns the extent from the name through the second bound.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the size's name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the exact extent of the first bound, or of `{` for a type
    /// parameter.
    #[must_use]
    pub const fn start_span(&self) -> Span {
        self.start_span
    }

    /// Returns the exact extent of the second bound, or of `}` for a type
    /// parameter.
    #[must_use]
    pub const fn end_span(&self) -> Span {
        self.end_span
    }

    /// Returns the listed types of a type parameter in source order, or an
    /// empty slice for a size parameter.
    #[must_use]
    pub fn types(&self) -> &[TypeSyntax] {
        &self.types
    }

    /// Returns whether this is a type parameter, `K in {F, L}`.
    #[must_use]
    pub const fn is_type(&self) -> bool {
        !self.types.is_empty()
    }
}

/// A size: an array type's length, a fill's length, or a loop bound. It is
/// an integer literal, the name of a size parameter, or a parenthesized
/// size expression, and it has one value in each instance of its function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Size {
    /// Exact extent: the integer token, the name, or the group with its
    /// parentheses.
    pub(crate) span: Span,
    /// The name or the group, or `None` for an integer token.
    pub(crate) expression: Option<Box<Expression>>,
}

impl Size {
    /// Returns the exact extent of the size.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the name or the group, or `None` for an integer token.
    #[must_use]
    pub fn expression(&self) -> Option<&Expression> {
        self.expression.as_deref()
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

/// One `let` binding in a typed body or a block:
/// `let name: Type = expression;` or, with a tuple pattern,
/// `let (a: T, b: U) = expression;`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Binding {
    /// Full extent from `let` through the closing `;`.
    pub(crate) span: Span,
    /// What the binding names: one typed name or a tuple pattern.
    pub(crate) pattern: Pattern,
    /// The bound expression.
    pub(crate) value: Expression,
}

impl Binding {
    /// Returns the full extent from `let` through the closing `;`.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns what the binding names.
    #[must_use]
    pub const fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    /// Returns the bound expression.
    #[must_use]
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

/// A name and its declared type, `name: Type`, as a binding, an accumulator,
/// or an element of a tuple pattern writes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedName {
    /// Full extent from the name through its type.
    pub(crate) span: Span,
    /// The name.
    pub(crate) name: Identifier,
    /// Syntactic declared type; semantic analysis resolves its meaning.
    pub(crate) ty: TypeSyntax,
}

impl TypedName {
    /// Returns the full extent from the name through its type.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the name.
    #[must_use]
    pub const fn name(&self) -> &Identifier {
        &self.name
    }

    /// Returns the syntactic declared type.
    #[must_use]
    pub const fn ty(&self) -> &TypeSyntax {
        &self.ty
    }
}

/// What a binding or a loop's accumulator names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Pattern {
    /// One name and its type, `name: Type`.
    Name(Box<TypedName>),
    /// A tuple pattern `(a: T, b: U, ...)`, which names each element of a
    /// tuple value.
    Tuple(Box<TuplePattern>),
}

impl Pattern {
    /// Returns the full extent of the pattern.
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Self::Name(name) => name.span,
            Self::Tuple(tuple) => tuple.span,
        }
    }

    /// Returns the typed names the pattern binds, in source order: one for
    /// a name, two through [`MAX_TUPLE_ELEMENTS`] for a tuple pattern.
    #[must_use]
    pub fn names(&self) -> &[TypedName] {
        match self {
            Self::Name(name) => std::slice::from_ref(&**name),
            Self::Tuple(tuple) => &tuple.elements,
        }
    }
}

/// A tuple pattern `(a: T, b: U, ...)` of two through
/// [`MAX_TUPLE_ELEMENTS`] typed names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TuplePattern {
    /// Full extent from `(` through `)`.
    pub(crate) span: Span,
    /// The typed names in source order.
    pub(crate) elements: Vec<TypedName>,
}

impl TuplePattern {
    /// Returns the full extent from `(` through `)`.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the typed names in source order.
    #[must_use]
    pub fn elements(&self) -> &[TypedName] {
        &self.elements
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
    /// An array literal `[e0, e1, ...]`, boxed like a conversion.
    Array(Box<ArrayExpression>),
    /// An array literal `[e; n]` of `n` copies of one element.
    Fill(Box<FillExpression>),
    /// One element `base[index]` of an array.
    Index(Box<IndexExpression>),
    /// A copy of an array with one element replaced:
    /// `base with [index] = value`.
    Update(Box<UpdateExpression>),
    /// A bounded loop `for i in a..b with s: T = init { step }`.
    Loop(Box<LoopExpression>),
    /// A conditional `if c { a } else { b }`, with any number of
    /// `else if` arms.
    Conditional(Box<ConditionalExpression>),
    /// A tuple `(e0, e1, ...)` of two through [`MAX_TUPLE_ELEMENTS`]
    /// elements.
    Tuple(Box<TupleExpression>),
    /// One element `base.k` of a tuple, selected by its position.
    Project(Box<ProjectExpression>),
    /// A byte string: `"..."` of printable ASCII characters and escapes, or
    /// `hex"..."` of hex digit pairs.
    Bytes(ByteString),
    /// A run of consecutive elements `base[start..end]` of an array.
    Slice(Box<SliceExpression>),
    /// A copy of an array with a run of consecutive elements replaced:
    /// `base with [start..end] = value`.
    SliceUpdate(Box<SliceUpdateExpression>),
}

/// A byte string literal, whose bytes are its spelling's: the characters
/// and escapes of `"..."`, or the hex digit pairs of `hex"..."`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteString {
    /// Whether the literal is a hex string `hex"..."`.
    pub(crate) hex: bool,
}

impl ByteString {
    /// Returns whether the literal is a hex string `hex"..."`.
    #[must_use]
    pub const fn is_hex(self) -> bool {
        self.hex
    }
}

/// The bounds `start..end` of a slice, either of which may be omitted: an
/// omitted start is 0 and an omitted end is the array's length.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceRange {
    /// Exact extent from the first bound, or `..`, through the last bound,
    /// or `..`.
    pub(crate) span: Span,
    /// The first element's index, when written.
    pub(crate) start: Option<Expression>,
    /// Exact extent of `..`.
    pub(crate) dots_span: Span,
    /// The index after the last element, when written.
    pub(crate) end: Option<Expression>,
}

impl SliceRange {
    /// Returns the exact extent of the bounds and `..`, excluding brackets.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the first element's index, when written.
    #[must_use]
    pub const fn start(&self) -> Option<&Expression> {
        self.start.as_ref()
    }

    /// Returns the exact extent of `..`.
    #[must_use]
    pub const fn dots_span(&self) -> Span {
        self.dots_span
    }

    /// Returns the index after the last element, when written.
    #[must_use]
    pub const fn end(&self) -> Option<&Expression> {
        self.end.as_ref()
    }
}

/// A slice `base[start..end]`, where `base` is a name, a call, or a
/// projection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceExpression {
    /// The sliced array.
    pub(crate) base: Expression,
    /// The bounds, excluding brackets.
    pub(crate) range: SliceRange,
}

impl SliceExpression {
    /// Returns the sliced array.
    #[must_use]
    pub const fn base(&self) -> &Expression {
        &self.base
    }

    /// Returns the bounds, excluding brackets.
    #[must_use]
    pub const fn range(&self) -> &SliceRange {
        &self.range
    }
}

/// A slice update `base with [start..end] = value`: the array `base` with
/// the elements from `start` up to `end` replaced by those of `value`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SliceUpdateExpression {
    /// The updated array.
    pub(crate) base: Expression,
    /// Exact extent of the `with` keyword.
    pub(crate) keyword_span: Span,
    /// The bounds of the replaced elements, excluding brackets.
    pub(crate) range: SliceRange,
    /// The new elements.
    pub(crate) value: Expression,
}

impl SliceUpdateExpression {
    /// Returns the updated array.
    #[must_use]
    pub const fn base(&self) -> &Expression {
        &self.base
    }

    /// Returns the exact extent of the `with` keyword.
    #[must_use]
    pub const fn keyword_span(&self) -> Span {
        self.keyword_span
    }

    /// Returns the bounds of the replaced elements.
    #[must_use]
    pub const fn range(&self) -> &SliceRange {
        &self.range
    }

    /// Returns the new elements.
    #[must_use]
    pub const fn value(&self) -> &Expression {
        &self.value
    }
}

/// A tuple expression `(e0, e1, ...)` of two through
/// [`MAX_TUPLE_ELEMENTS`] elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TupleExpression {
    /// Elements in source order.
    pub(crate) elements: Vec<Expression>,
}

impl TupleExpression {
    /// Returns the elements in source order.
    #[must_use]
    pub fn elements(&self) -> &[Expression] {
        &self.elements
    }
}

/// One element `base.k` of a tuple.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectExpression {
    /// The tuple: a name or a call.
    pub(crate) base: Expression,
    /// Exact extent of the position's integer token.
    pub(crate) position_span: Span,
    /// The zero-based position, written in decimal. A position too large
    /// for `u32` is kept as `u32::MAX`, which no tuple has.
    pub(crate) position: u32,
}

impl ProjectExpression {
    /// Returns the tuple.
    #[must_use]
    pub const fn base(&self) -> &Expression {
        &self.base
    }

    /// Returns the exact extent of the position's integer token.
    #[must_use]
    pub const fn position_span(&self) -> Span {
        self.position_span
    }

    /// Returns the zero-based position.
    #[must_use]
    pub const fn position(&self) -> u32 {
        self.position
    }
}

/// An array literal `[e0, e1, ...]` with at least one element.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArrayExpression {
    /// Elements in source order.
    pub(crate) elements: Vec<Expression>,
}

impl ArrayExpression {
    /// Returns the elements in source order.
    #[must_use]
    pub fn elements(&self) -> &[Expression] {
        &self.elements
    }
}

/// An array literal `[element; n]` of `n` copies of one element.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FillExpression {
    /// The repeated element.
    pub(crate) element: Expression,
    /// The number of copies.
    pub(crate) length: Size,
}

impl FillExpression {
    /// Returns the repeated element.
    #[must_use]
    pub fn element(&self) -> &Expression {
        &self.element
    }

    /// Returns the exact extent of the length.
    #[must_use]
    pub const fn length_span(&self) -> Span {
        self.length.span
    }

    /// Returns the number of copies.
    #[must_use]
    pub const fn length(&self) -> &Size {
        &self.length
    }
}

/// An element selection `base[index]`, where `base` is a name or a call.
///
/// An index written as one unsigned integer token is a literal index, as in
/// S3d; any other index is an expression that semantic analysis must prove
/// in range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexExpression {
    /// The indexed name or call.
    pub(crate) base: Expression,
    /// The index, excluding brackets.
    pub(crate) index: Expression,
}

impl IndexExpression {
    /// Returns the indexed name or call.
    #[must_use]
    pub fn base(&self) -> &Expression {
        &self.base
    }

    /// Returns the index, excluding brackets.
    #[must_use]
    pub fn index(&self) -> &Expression {
        &self.index
    }

    /// Returns the exact extent of the index, excluding brackets.
    #[must_use]
    pub const fn index_span(&self) -> Span {
        self.index.span
    }

    /// Returns whether the index is one unsigned integer token.
    #[must_use]
    pub const fn is_literal(&self) -> bool {
        matches!(
            &self.index.kind,
            ExpressionKind::Literal(IntegerLiteral {
                negative: false,
                ..
            })
        )
    }
}

/// A functional update `base with [index] = value`: the array `base` with
/// the element at `index` replaced by `value`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateExpression {
    /// The updated array.
    pub(crate) base: Expression,
    /// Exact extent of the `with` keyword.
    pub(crate) keyword_span: Span,
    /// The replaced element's index, excluding brackets.
    pub(crate) index: Expression,
    /// The indices within that element, each excluding its brackets:
    /// `x with [i][j] = v` replaces element j of row i. Empty when the
    /// element itself is replaced.
    pub(crate) path: Vec<Expression>,
    /// The new element.
    pub(crate) value: Expression,
}

impl UpdateExpression {
    /// Returns the updated array.
    #[must_use]
    pub fn base(&self) -> &Expression {
        &self.base
    }

    /// Returns the exact extent of the `with` keyword.
    #[must_use]
    pub const fn keyword_span(&self) -> Span {
        self.keyword_span
    }

    /// Returns the replaced element's index.
    #[must_use]
    pub fn index(&self) -> &Expression {
        &self.index
    }

    /// Returns the indices within the replaced element, in order: `j` of
    /// `x with [i][j] = v`.
    #[must_use]
    pub fn path(&self) -> &[Expression] {
        &self.path
    }

    /// Returns the new element.
    #[must_use]
    pub fn value(&self) -> &Expression {
        &self.value
    }
}

/// A bounded loop `for i in a..b with s: T = init { step }`.
///
/// The loop's value is `s` after `step` has been applied once for each `i`
/// from `a` up to, but not including, `b`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopExpression {
    /// Exact extent of the `for` keyword.
    pub(crate) keyword_span: Span,
    /// The loop index.
    pub(crate) index: Identifier,
    /// The first bound.
    pub(crate) start: Size,
    /// The second bound.
    pub(crate) end: Size,
    /// The accumulator: one typed name, or a tuple pattern that names each
    /// element of a tuple accumulator.
    pub(crate) accumulator: Pattern,
    /// The accumulator's initial value.
    pub(crate) init: Expression,
    /// `let` bindings at the start of the step, in source order, evaluated
    /// again at every step before the step's value.
    pub(crate) step_bindings: Vec<Binding>,
    /// The step's value, which gives the accumulator's next value.
    pub(crate) step: Expression,
}

impl LoopExpression {
    /// Returns the exact extent of the `for` keyword.
    #[must_use]
    pub const fn keyword_span(&self) -> Span {
        self.keyword_span
    }

    /// Returns the loop index.
    #[must_use]
    pub const fn index(&self) -> &Identifier {
        &self.index
    }

    /// Returns the exact extent of the first bound.
    #[must_use]
    pub const fn start_span(&self) -> Span {
        self.start.span
    }

    /// Returns the exact extent of the second bound.
    #[must_use]
    pub const fn end_span(&self) -> Span {
        self.end.span
    }

    /// Returns the first bound.
    #[must_use]
    pub const fn start(&self) -> &Size {
        &self.start
    }

    /// Returns the second bound.
    #[must_use]
    pub const fn end(&self) -> &Size {
        &self.end
    }

    /// Returns the accumulator's pattern.
    #[must_use]
    pub const fn accumulator(&self) -> &Pattern {
        &self.accumulator
    }

    /// Returns the accumulator's initial value.
    #[must_use]
    pub fn init(&self) -> &Expression {
        &self.init
    }

    /// Returns the `let` bindings at the start of the step, in source order.
    #[must_use]
    pub fn step_bindings(&self) -> &[Binding] {
        &self.step_bindings
    }

    /// Returns the step's value.
    #[must_use]
    pub fn step(&self) -> &Expression {
        &self.step
    }
}

/// A conditional `if c0 { v0 } else if c1 { v1 } ... else { w }`.
///
/// Its value is the value of the first arm whose condition is true, or the
/// value after the last `else` when none is.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConditionalExpression {
    /// The arms in source order; there is at least one.
    pub(crate) arms: Vec<ConditionalArm>,
    /// Exact extent of the last `else` keyword.
    pub(crate) else_span: Span,
    /// `let` bindings at the start of the last `else` branch, in source
    /// order.
    pub(crate) otherwise_bindings: Vec<Binding>,
    /// The value when no condition is true.
    pub(crate) otherwise: Expression,
}

impl ConditionalExpression {
    /// Returns the arms in source order.
    #[must_use]
    pub fn arms(&self) -> &[ConditionalArm] {
        &self.arms
    }

    /// Returns the exact extent of the last `else` keyword.
    #[must_use]
    pub const fn else_span(&self) -> Span {
        self.else_span
    }

    /// Returns the `let` bindings at the start of the last `else` branch.
    #[must_use]
    pub fn otherwise_bindings(&self) -> &[Binding] {
        &self.otherwise_bindings
    }

    /// Returns the value when no condition is true.
    #[must_use]
    pub fn otherwise(&self) -> &Expression {
        &self.otherwise
    }
}

/// One arm `if condition { value }` of a conditional.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConditionalArm {
    /// Exact extent of the arm's `if` keyword.
    pub(crate) keyword_span: Span,
    /// The condition, a `Bool` expression.
    pub(crate) condition: Expression,
    /// `let` bindings at the start of the arm's branch, in source order.
    pub(crate) bindings: Vec<Binding>,
    /// The value when the condition is the first true one.
    pub(crate) value: Expression,
}

impl ConditionalArm {
    /// Returns the exact extent of the arm's `if` keyword.
    #[must_use]
    pub const fn keyword_span(&self) -> Span {
        self.keyword_span
    }

    /// Returns the condition.
    #[must_use]
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    /// Returns the `let` bindings at the start of the arm's branch.
    #[must_use]
    pub fn bindings(&self) -> &[Binding] {
        &self.bindings
    }

    /// Returns the value.
    #[must_use]
    pub fn value(&self) -> &Expression {
        &self.value
    }
}

/// An explicit conversion `operand as Type`, or `operand as big Type` and
/// `operand as little Type` with a byte order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConversionExpression {
    /// The converted operand.
    pub(crate) operand: Expression,
    /// Exact extent of the `as` keyword.
    pub(crate) keyword_span: Span,
    /// The byte order and the exact extent of its word, if one is named.
    pub(crate) order: Option<(ByteOrder, Span)>,
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

    /// Returns the byte order, if one is named.
    #[must_use]
    pub fn order(&self) -> Option<ByteOrder> {
        self.order.map(|(order, _)| order)
    }

    /// Returns the exact extent of the byte order's word, if one is named.
    #[must_use]
    pub fn order_span(&self) -> Option<Span> {
        self.order.map(|(_, span)| span)
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
    /// The module named before `::` and the sizes in brackets, boxed
    /// together so that they do not enlarge every expression; `None` for an
    /// unqualified call without sizes.
    pub(crate) qualifiers: Option<Box<CallQualifiers>>,
    /// Called function name.
    pub(crate) callee: Identifier,
    /// Arguments in source order.
    pub(crate) arguments: Vec<Expression>,
}

/// The module and the sizes of a call that names either.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CallQualifiers {
    /// The module named before `::`, for a call of a used module's function.
    pub(crate) module: Option<Identifier>,
    /// The sizes of `name[sizes](arguments)` in source order, empty for a
    /// function without size parameters.
    pub(crate) sizes: Vec<Expression>,
}

impl CallExpression {
    /// Returns the module named before `::`, or `None` for a call of a
    /// function of the calling module.
    #[must_use]
    pub fn module(&self) -> Option<&Identifier> {
        self.qualifiers.as_deref()?.module.as_ref()
    }

    /// Returns the called function name.
    #[must_use]
    pub const fn callee(&self) -> &Identifier {
        &self.callee
    }

    /// Returns the sizes in source order, empty for a call without sizes.
    #[must_use]
    pub fn sizes(&self) -> &[Expression] {
        self.qualifiers
            .as_deref()
            .map_or(&[], |qualifiers| &qualifiers.sizes)
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
        /// `!`: logical negation.
        Not => "!",
    }
}

define_operators! {
    /// The byte order of a conversion that packs words into words of another
    /// width or into a number, or unpacks them: the order in which the
    /// elements of an array of words stand in the number they spell.
    ByteOrder {
        /// `big`: the first element is the most significant.
        Big => "big",
        /// `little`: the first element is the least significant.
        Little => "little",
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
        /// `/`: quotient.
        Divide => "/",
        /// `%`: remainder.
        Remainder => "%",
        /// `==`: equality.
        Equal => "==",
        /// `!=`: inequality.
        NotEqual => "!=",
        /// `<`: less than.
        Less => "<",
        /// `<=`: less than or equal to.
        LessEqual => "<=",
        /// `>`: greater than.
        Greater => ">",
        /// `>=`: greater than or equal to.
        GreaterEqual => ">=",
        /// `&&`: logical and, of two `Bool` values.
        LogicalAnd => "&&",
        /// `||`: logical or, of two `Bool` values.
        LogicalOr => "||",
        /// `++`: concatenation of two arrays.
        Concat => "++",
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

    /// Returns whether this operator compares two values and gives a
    /// `Bool`.
    #[must_use]
    pub const fn is_comparison(self) -> bool {
        matches!(
            self,
            Self::Equal
                | Self::NotEqual
                | Self::Less
                | Self::LessEqual
                | Self::Greater
                | Self::GreaterEqual
        )
    }

    /// Returns whether this operator is `/` or `%`.
    #[must_use]
    pub const fn is_division(self) -> bool {
        matches!(self, Self::Divide | Self::Remainder)
    }

    /// Returns whether this operator is `&&` or `||`.
    #[must_use]
    pub const fn is_logical(self) -> bool {
        matches!(self, Self::LogicalAnd | Self::LogicalOr)
    }

    /// Returns whether this operator is `++`, which joins two arrays.
    #[must_use]
    pub const fn is_concatenation(self) -> bool {
        matches!(self, Self::Concat)
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
            Self::Divide => TokenKind::Slash,
            Self::Remainder => TokenKind::Percent,
            Self::Equal => TokenKind::EqualEqual,
            Self::NotEqual => TokenKind::BangEqual,
            Self::Less => TokenKind::Less,
            Self::LessEqual => TokenKind::LessEqual,
            Self::Greater => TokenKind::Greater,
            Self::GreaterEqual => TokenKind::GreaterEqual,
            Self::LogicalAnd => TokenKind::AmpAmp,
            Self::LogicalOr => TokenKind::PipePipe,
            Self::Concat => TokenKind::PlusPlus,
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
            TokenKind::Slash => Self::Divide,
            TokenKind::Percent => Self::Remainder,
            TokenKind::EqualEqual => Self::Equal,
            TokenKind::BangEqual => Self::NotEqual,
            TokenKind::Less => Self::Less,
            TokenKind::LessEqual => Self::LessEqual,
            TokenKind::Greater => Self::Greater,
            TokenKind::GreaterEqual => Self::GreaterEqual,
            TokenKind::AmpAmp => Self::LogicalAnd,
            TokenKind::PipePipe => Self::LogicalOr,
            TokenKind::PlusPlus => Self::Concat,
            _ => return None,
        })
    }
}

/// A syntactic type name with an optional integer width argument or modulus
/// and an optional array length, as in `Word[32]^16` or
/// `Mod[(1 << 255) - 19]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeSyntax {
    /// Full type extent, including `[WIDTH]` and `^LENGTH` when present.
    pub(crate) span: Span,
    /// Exact type-name spelling and span.
    pub(crate) name: Identifier,
    /// Exact span of the width integer, excluding brackets.
    pub(crate) width_span: Option<Span>,
    /// The modulus expression of `Mod[...]`, excluding brackets.
    pub(crate) modulus: Option<Box<Expression>>,
    /// The array length, excluding `^`.
    pub(crate) length: Option<Size>,
    /// The element types of a tuple type `(T0, T1, ...)`, in order, and
    /// empty for every other type. A tuple type has no name: its `name` is
    /// empty and spans its `(`.
    pub(crate) elements: Vec<TypeSyntax>,
}

impl TypeSyntax {
    /// Returns the full type extent, including `[WIDTH]` and `^LENGTH` when
    /// present.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact span of the array length, excluding `^`.
    #[must_use]
    pub fn length_span(&self) -> Option<Span> {
        self.length.as_ref().map(Size::span)
    }

    /// Returns the array length, excluding `^`.
    #[must_use]
    pub const fn length(&self) -> Option<&Size> {
        self.length.as_ref()
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

    /// Returns the modulus expression of `Mod[...]`, excluding brackets.
    #[must_use]
    pub fn modulus(&self) -> Option<&Expression> {
        self.modulus.as_deref()
    }

    /// Returns the element types of a tuple type in order, or an empty
    /// slice for every other type.
    #[must_use]
    pub fn elements(&self) -> &[TypeSyntax] {
        &self.elements
    }

    /// Returns whether this is a tuple type `(T0, T1, ...)`.
    #[must_use]
    pub fn is_tuple(&self) -> bool {
        !self.elements.is_empty()
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

/// What a body's diagnostics say it is: the token it follows, the shape of
/// its contents, and what its last expression does.
struct BodyShape {
    opener: &'static str,
    note: &'static str,
    tail: &'static str,
}

const SPEC_BODY_SHAPE: BodyShape = BodyShape {
    opener: "`{` after the result type",
    note: BODY_SHAPE_NOTE,
    tail: "a typed `spec` body ends with the expression that gives its value",
};

const TEST_SHAPE_NOTE: &str =
    "a test is written `test \"TITLE\" { EXPRESSION }`, its expression a `Bool`";

const TEST_BODY_SHAPE: BodyShape = BodyShape {
    opener: "`{` after the test's title",
    note: "a test's body holds `let` bindings, if any, and then one `Bool` expression",
    tail: "a test's body ends with the `Bool` expression that decides it",
};

const COMPUTED_FILL_NOTE: &str =
    "a length computed from sizes is written in parentheses, as in `[0; (2 * n)]`";

const COMPUTED_BOUND_NOTE: &str = "a bound computed from sizes is written in parentheses, as in \
     `for i in 0..(n - 1) with s: Type = start { step }`";

const SIZED_CALL_NOTE: &str = "a sized function is called with its sizes in brackets before its \
     arguments, as in `sha256[2](m)`";

const SIZE_PARAMETER_NOTE: &str = "a sized function is written `spec f[n in 1..5](x: Word[8]^n) \
     -> Type { ... }` and checked once for each n from 1 up to, but not including, 5";

const TYPE_PARAMETER_NOTE: &str = "a type parameter is written `K in {F, L}` and names each type \
     its function is checked for, as in `spec square[K in {F, L}](x: K) -> K { x * x }`";

const LOOP_SHAPE_NOTE: &str = "a loop is written `for i in 0..n with s: Type = start { step }`";

const STEP_SHAPE_NOTE: &str = "a loop's step holds `let` bindings, if any, and then the \
     expression that gives the accumulator's next value";

const CONDITIONAL_SHAPE_NOTE: &str =
    "a conditional is written `if condition { value } else { other value }`";

const BRANCH_SHAPE_NOTE: &str =
    "each branch of a conditional holds `let` bindings, if any, and then its value";

const TUPLE_NOTE: &str = "a tuple is written `(a, b)` with two through 16 elements; `(a)` \
     without a comma is a group";

const TUPLE_TYPE_NOTE: &str = "a tuple type is written `(T, U)` with two through 16 element \
     types, each `Int`, `Bool`, a word, a residue, or an array of one";

const PATTERN_NOTE: &str = "a tuple pattern names two through 16 values, each with its type, \
     as in `let (sum: Word[64], carry: Word[64]) = add(x, y, c);`";

/// The note of a hex string written with a space after `hex`.
const SPACED_HEX_NOTE: &str =
    "a hex string's quote follows `hex` directly, with no space, as in `hex\"00 1f a0\"`";

/// The note of a malformed slice.
const SLICE_NOTE: &str = "a slice is written `x[a..b]`, the elements of `x` from index a up to but \
     not including index b, or `x[a..]` or `x[..b]` to run to the end or from the start";

/// What an update replaces: one element at an index, or the elements of a
/// slice, with the height of the index or of the taller bound.
enum UpdateTarget {
    /// An index, the indices within that element, and their tallest
    /// height.
    Index(Expression, Vec<Expression>, usize),
    Slice(SliceRange, usize),
}

/// Most indices of an update, one per dimension of an array.
const UPDATE_INDICES: usize = 4;
const _: () = assert!(crate::core::MAX_ARRAY_DIMENSIONS == 4);

/// The note of a malformed update of an element of a row.
const PATH_UPDATE_NOTE: &str = "an element of a row is updated with `x with [i][j] = v`, one \
     index per dimension; a run of a row is updated as `x with [i] = (x[i] with [a..b] = v)`";

/// The note of a malformed slice update.
const SLICE_UPDATE_NOTE: &str = "a slice update is written `x with [a..b] = values`, the array `x` \
     with its elements from index a up to but not including index b replaced";

const PROJECTION_NOTE: &str = "a tuple's element is selected by its position, counted from \
     zero and written in decimal, as in `pair.0` or `pair.1`";

/// How a pattern's parts are described in diagnostics.
struct PatternRole {
    name: &'static str,
    colon: &'static str,
    colon_note: &'static str,
    ty: &'static str,
}

const BINDING_ROLE: PatternRole = PatternRole {
    name: "binding",
    colon: "`:` and the binding's type",
    colon_note: "every binding states its type, as in `let t: Word[32] = x + y;`",
    ty: "binding type",
};

const ACCUMULATOR_ROLE: PatternRole = PatternRole {
    name: "accumulator",
    colon: "`:` and the accumulator's type",
    colon_note: "every accumulator states its type, as in `with s: Word[32]^16 = x`",
    ty: "accumulator type",
};

/// Something that continues an expression after an operand: a binary
/// operator, the conversion keyword `as`, or the update keyword `with`.
#[derive(Clone, Copy)]
enum Joiner {
    Binary(BinaryOperator),
    As,
    With,
}

impl Joiner {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Binary(operator) => operator.as_str(),
            Self::As => "as",
            Self::With => "with",
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
    reserve_arm_slot: fn(&mut Vec<ConditionalArm>) -> bool,
    reserve_type_slot: fn(&mut Vec<TypeSyntax>) -> bool,
    reserve_name_slot: fn(&mut Vec<TypedName>) -> bool,
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

fn reserve_arm_slot(arms: &mut Vec<ConditionalArm>) -> bool {
    arms.try_reserve(1).is_ok()
}

fn reserve_binding_slot(bindings: &mut Vec<Binding>) -> bool {
    bindings.try_reserve(1).is_ok()
}

fn reserve_type_slot(types: &mut Vec<TypeSyntax>) -> bool {
    types.try_reserve(1).is_ok()
}

fn reserve_name_slot(names: &mut Vec<TypedName>) -> bool {
    names.try_reserve(1).is_ok()
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
            reserve_arm_slot,
            reserve_type_slot,
            reserve_name_slot,
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

        let mut uses = Vec::new();
        let mut types = Vec::new();
        // `use` and `type` are recognized by position, as `let` is: they
        // start declarations only before the module's first function, where
        // no other identifier may appear. The `use` declarations come first.
        while !self.halted && (self.current_is_word("use") || self.current_is_word("type")) {
            let before = self.cursor;
            if self.current_is_word("type") {
                if let Some(declaration) = self.parse_type_declaration() {
                    self.push_type(&mut types, declaration);
                }
            } else if types.is_empty() {
                if let Some(declaration) = self.parse_use_declaration() {
                    self.push_use(&mut uses, declaration);
                }
            } else {
                self.report(
                    DiagnosticCode::ExpectedFunctionDeclaration,
                    "expected a `type` declaration or a function",
                    self.current_span(),
                    "a `use` declaration cannot follow a `type` declaration",
                    "a module's `use` declarations come first, then its `type` declarations, \
                     then its functions",
                );
                self.parse_use_declaration();
            }
            if !self.halted && self.cursor == before {
                self.bump();
            }
        }

        let mut functions = Vec::new();
        let mut tests = Vec::new();
        while !self.halted && !matches!(self.current_kind(), TokenKind::RightBrace | TokenKind::Eof)
        {
            let before = self.cursor;
            match self.current_kind() {
                // `test` is recognized by position, as `use` and `type` are:
                // it starts a test only where a function could start.
                TokenKind::Identifier if self.current_is_word("test") => {
                    if let Some(test) = self.parse_test_declaration() {
                        if tests.try_reserve(1).is_ok() {
                            tests.push(test);
                        } else {
                            self.resource_limit_at(
                                "parser could not allocate module test storage",
                                test.span,
                            );
                        }
                    }
                }
                TokenKind::Identifier if self.current_is_word("use") => {
                    self.report(
                        DiagnosticCode::ExpectedFunctionDeclaration,
                        "expected a `spec` or `impl` function declaration",
                        self.current_span(),
                        "a `use` declaration cannot follow a function",
                        "`use` declarations come first in a module, before its functions",
                    );
                    self.recover_to(&[
                        TokenKind::KwSpec,
                        TokenKind::KwImpl,
                        TokenKind::RightBrace,
                        TokenKind::Eof,
                    ]);
                }
                TokenKind::Identifier if self.current_is_word("type") => {
                    self.report(
                        DiagnosticCode::ExpectedFunctionDeclaration,
                        "expected a `spec` or `impl` function declaration",
                        self.current_span(),
                        "a `type` declaration cannot follow a function",
                        "`type` declarations come before a module's functions",
                    );
                    self.recover_to(&[
                        TokenKind::KwSpec,
                        TokenKind::KwImpl,
                        TokenKind::RightBrace,
                        TokenKind::Eof,
                    ]);
                }
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
                    uses,
                    types,
                    functions,
                    tests,
                })
            }
            _ => None,
        }
    }

    /// Parses `test "TITLE" { BINDINGS EXPRESSION }`, with the current token
    /// the word `test`. The test becomes a typed `spec` without parameters
    /// whose result type is `Bool`, both named at the title.
    fn parse_test_declaration(&mut self) -> Option<TestDeclaration> {
        let keyword = self.bump()?;
        let title = if self.current_kind() == TokenKind::String {
            let token = self.bump()?;
            let spelling = token.lexeme(self.source)?;
            // An unterminated title, already reported, has no closing quote.
            let inner = spelling.strip_prefix('"').unwrap_or(spelling);
            let inner = inner.strip_suffix('"').unwrap_or(inner);
            let text = self.test_text(inner, token.span)?;
            self.record_node().then_some(TestTitle {
                text,
                span: token.span,
            })
        } else {
            self.expected("a quoted title after `test`", TEST_SHAPE_NOTE);
            None
        };
        if title.is_none() && self.current_kind() != TokenKind::LeftBrace {
            self.recover_to(&[
                TokenKind::LeftBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ]);
        }
        if title.is_none() && self.current_kind() != TokenKind::LeftBrace {
            return None;
        }
        let (contents, right_brace) = self.parse_body_block(&TEST_BODY_SHAPE);
        let (title, (bindings, expression), right_brace) = (title?, contents?, right_brace?);
        // The result type `Bool` and the name `test` are the title's.
        let bool_name = self.test_text("Bool", title.span)?;
        let test_name = self.test_text("test", title.span)?;
        let bool_type = TypeSyntax {
            span: title.span,
            name: Identifier {
                text: bool_name,
                span: title.span,
            },
            width_span: None,
            modulus: None,
            length: None,
            elements: Vec::new(),
        };
        let span = self.join(keyword.span, right_brace.span);
        let body_span = self.join(title.span, right_brace.span);
        (self.record_node()).then(|| TestDeclaration {
            span,
            function: FunctionDeclaration {
                span,
                kind: FunctionKind::Spec,
                name: Identifier {
                    text: test_name,
                    span: title.span,
                },
                sizes: Vec::new(),
                parameters: Vec::new(),
                body: FunctionBody::Typed(Box::new(TypedBody {
                    span: body_span,
                    result_type: bool_type,
                    bindings,
                    expression,
                })),
            },
            title,
        })
    }

    /// Copies text of a test declaration into storage reserved as an
    /// identifier's is.
    fn test_text(&mut self, text: &str, span: Span) -> Option<String> {
        let mut owned = String::new();
        if !(self.reserve_identifier_text)(&mut owned, text.len()) {
            self.resource_limit_at("parser could not allocate test title storage", span);
            return None;
        }
        owned.push_str(text);
        Some(owned)
    }

    /// Parses `use NAME;`, with the current token the word `use`.
    fn parse_use_declaration(&mut self) -> Option<UseDeclaration> {
        let keyword = self.bump()?;
        let name = self.parse_identifier("used module");
        let Some(name) = name else {
            self.recover_to(&[
                TokenKind::Semicolon,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ]);
            if self.current_kind() == TokenKind::Semicolon {
                self.bump();
            }
            return None;
        };
        let semicolon = self.consume_or_recover(
            TokenKind::Semicolon,
            "`;` after the used module's name",
            "a `use` declaration names one module and ends with `;`",
            &[
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ],
        )?;
        self.record_node().then(|| UseDeclaration {
            span: self.join(keyword.span, semicolon.span),
            name,
        })
    }

    /// Parses `type NAME = TYPE;`, with the current token the word `type`.
    fn parse_type_declaration(&mut self) -> Option<TypeDeclaration> {
        const SHAPE: &str = "a `type` declaration is written `type NAME = TYPE;`";
        let recovery = [
            TokenKind::Semicolon,
            TokenKind::KwSpec,
            TokenKind::KwImpl,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ];
        let keyword = self.bump()?;
        let Some(name) = self.parse_identifier("type name") else {
            self.recover_to(&recovery);
            if self.current_kind() == TokenKind::Semicolon {
                self.bump();
            }
            return None;
        };
        let ty = self
            .expect(TokenKind::Equal, "`=` after the type name", SHAPE)
            .and_then(|_| self.parse_type_syntax("declared type", true, 0))
            .map(|(ty, _)| ty);
        let Some(ty) = ty else {
            self.recover_to(&recovery);
            if self.current_kind() == TokenKind::Semicolon {
                self.bump();
            }
            return None;
        };
        let semicolon = self.consume_or_recover(
            TokenKind::Semicolon,
            "`;` after the declared type",
            SHAPE,
            &[
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ],
        )?;
        self.record_node().then(|| TypeDeclaration {
            span: self.join(keyword.span, semicolon.span),
            name,
            ty,
        })
    }

    #[inline(never)]
    fn push_type(&mut self, types: &mut Vec<TypeDeclaration>, declaration: TypeDeclaration) {
        if types.len() >= MAX_TYPES_PER_MODULE {
            self.resource_limit_at(
                format!("module has more than {MAX_TYPES_PER_MODULE} `type` declarations"),
                declaration.span,
            );
            return;
        }
        if types.try_reserve(1).is_err() {
            self.resource_limit_at("parser could not allocate `type` storage", declaration.span);
            return;
        }
        types.push(declaration);
    }

    #[inline(never)]
    fn push_use(&mut self, uses: &mut Vec<UseDeclaration>, declaration: UseDeclaration) {
        if uses.len() >= MAX_USES_PER_MODULE {
            self.resource_limit_at(
                format!("module has more than {MAX_USES_PER_MODULE} `use` declarations"),
                declaration.span,
            );
            return;
        }
        if uses.try_reserve(1).is_err() {
            self.resource_limit_at("parser could not allocate `use` storage", declaration.span);
            return;
        }
        uses.push(declaration);
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
                    | TokenKind::LeftBracket
                    | TokenKind::KwSpec
                    | TokenKind::KwImpl
                    | TokenKind::RightBrace
                    | TokenKind::Eof
            )
        {
            self.recover_to(&[
                TokenKind::LeftParen,
                TokenKind::LeftBracket,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ]);
        }

        let sizes = if self.current_kind() == TokenKind::LeftBracket {
            let sizes = self.parse_size_parameters();
            if sizes.is_none() {
                self.recover_to(&[
                    TokenKind::LeftParen,
                    TokenKind::KwSpec,
                    TokenKind::KwImpl,
                    TokenKind::RightBrace,
                    TokenKind::Eof,
                ]);
            }
            sizes
        } else {
            Some(Vec::new())
        };
        if kind == FunctionKind::Impl
            && let Some(first) = sizes.as_ref().and_then(|list| list.first())
        {
            let (message, label) = if first.is_type() {
                (
                    "`impl` functions have no type parameters",
                    "type parameters are allowed only on typed `spec` functions",
                )
            } else {
                (
                    "`impl` functions have no size parameters",
                    "size parameters are allowed only on typed `spec` functions",
                )
            };
            self.report(
                DiagnosticCode::ExpectedSyntax,
                message,
                first.span,
                label,
                "keep the legacy `impl name() {}` form until implementation semantics are defined",
            );
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
        let has_parameters = parameters.as_ref().is_some_and(|list| !list.is_empty())
            || sizes.as_ref().is_some_and(|list| !list.is_empty());
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
                    "a `spec` with parameters or sizes needs a result type and a body expression",
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

        match (
            name,
            sizes,
            left_paren,
            parameters,
            right_paren,
            body,
            body_end,
        ) {
            (
                Some(name),
                Some(sizes),
                Some(_),
                Some(parameters),
                Some(_),
                Some(body),
                Some(body_end),
            ) if self.record_node() => Some(FunctionDeclaration {
                span: self.join(keyword.span, body_end.span),
                kind,
                name,
                sizes,
                parameters,
                body,
            }),
            _ => None,
        }
    }

    /// Parses the size and type parameters `[n in a..b, K in {F, L}, ...]`
    /// of a function, at most [`MAX_SIZES_PER_FUNCTION`] of them, each size
    /// with integer bounds and each type parameter with a braced list of
    /// types.
    #[inline(never)]
    fn parse_size_parameters(&mut self) -> Option<Vec<SizeParameter>> {
        self.bump()?;
        let mut sizes = Vec::new();
        loop {
            let name = self.parse_identifier("size parameter")?;
            if !self.current_is_word("in") {
                self.expected("`in` after the size's name", SIZE_PARAMETER_NOTE);
                return None;
            }
            self.bump()?;
            if self.current_kind() == TokenKind::LeftBrace {
                let parameter = self.parse_type_parameter(name)?;
                if sizes.len() >= MAX_SIZES_PER_FUNCTION {
                    self.report(
                        DiagnosticCode::ExpectedSyntax,
                        format!(
                            "a function has at most {MAX_SIZES_PER_FUNCTION} size and type \
                             parameters"
                        ),
                        parameter.span,
                        "one parameter in brackets too many",
                        TYPE_PARAMETER_NOTE,
                    );
                    return None;
                }
                if sizes.try_reserve(1).is_err() {
                    self.resource_limit_at(
                        "parser could not allocate size storage",
                        parameter.name.span,
                    );
                    return None;
                }
                sizes.push(parameter);
                match self.current_kind() {
                    TokenKind::Comma => {
                        self.bump()?;
                        continue;
                    }
                    TokenKind::RightBracket => break,
                    _ => {
                        self.expected("`,` or `]` after the type parameter", TYPE_PARAMETER_NOTE);
                        return None;
                    }
                }
            }
            let start_span = self
                .expect(
                    TokenKind::Integer,
                    "the size's first bound",
                    SIZE_PARAMETER_NOTE,
                )?
                .span;
            self.expect(
                TokenKind::DotDot,
                "`..` between the size's bounds",
                SIZE_PARAMETER_NOTE,
            )?;
            let end_span = self
                .expect(
                    TokenKind::Integer,
                    "the size's second bound",
                    SIZE_PARAMETER_NOTE,
                )?
                .span;
            if sizes.len() >= MAX_SIZES_PER_FUNCTION {
                let kinds = if sizes.iter().any(SizeParameter::is_type) {
                    "size and type parameters"
                } else {
                    "size parameters"
                };
                self.report(
                    DiagnosticCode::ExpectedSyntax,
                    format!("a function has at most {MAX_SIZES_PER_FUNCTION} {kinds}"),
                    self.join(name.span, end_span),
                    "one size parameter too many",
                    SIZE_PARAMETER_NOTE,
                );
                return None;
            }
            if sizes.try_reserve(1).is_err() {
                self.resource_limit_at("parser could not allocate size storage", name.span);
                return None;
            }
            let span = self.join(name.span, end_span);
            sizes.push(SizeParameter {
                span,
                name,
                start_span,
                end_span,
                types: Vec::new(),
            });
            if !self.record_node() {
                return None;
            }
            match self.current_kind() {
                TokenKind::Comma => {
                    self.bump()?;
                }
                TokenKind::RightBracket => break,
                _ => {
                    self.expected("`,` or `]` after the size parameter", SIZE_PARAMETER_NOTE);
                    return None;
                }
            }
        }
        self.bump()?;
        Some(sizes)
    }

    /// Parses the braced list of a type parameter `K in {F, L}` at its `{`,
    /// after the parameter's name and `in`: one or more types separated by
    /// commas. After a syntax error in the list, parsing resumes after its
    /// `}`, so that the function's `(` is found.
    #[inline(never)]
    fn parse_type_parameter(&mut self, name: Identifier) -> Option<SizeParameter> {
        let parameter = self.parse_type_list(name);
        if parameter.is_none() {
            self.recover_to(&[
                TokenKind::RightBrace,
                TokenKind::LeftParen,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
            ]);
            if self.current_kind() == TokenKind::RightBrace {
                self.bump();
            }
        }
        parameter
    }

    fn parse_type_list(&mut self, name: Identifier) -> Option<SizeParameter> {
        let open = self.bump()?;
        let mut types = Vec::new();
        loop {
            if self.current_kind() == TokenKind::RightBrace {
                self.expected(
                    if types.is_empty() {
                        "a listed type after `{`"
                    } else {
                        "a listed type after `,`"
                    },
                    TYPE_PARAMETER_NOTE,
                );
                return None;
            }
            let (ty, _) = self.parse_type_syntax("listed type", true, 0)?;
            if types.try_reserve(1).is_err() {
                self.resource_limit_at("parser could not allocate type storage", ty.span);
                return None;
            }
            types.push(ty);
            match self.current_kind() {
                TokenKind::Comma => {
                    self.bump()?;
                }
                TokenKind::RightBrace => break,
                _ => {
                    self.expected("`,` or `}` after the listed type", TYPE_PARAMETER_NOTE);
                    return None;
                }
            }
        }
        let close = self.bump()?;
        if !self.record_node() {
            return None;
        }
        Some(SizeParameter {
            span: self.join(name.span, close.span),
            name,
            start_span: open.span,
            end_span: close.span,
            types,
        })
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
        let (ty, _) = self.parse_type_syntax("parameter type", true, 0)?;
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

    /// Parses a body's braces and contents, `{ BINDINGS EXPRESSION }`, with
    /// the current token expected to be `{`. Returns the bindings and the
    /// expression when both parsed, and the closing brace when it was found.
    fn parse_body_block(
        &mut self,
        shape: &BodyShape,
    ) -> (Option<(Vec<Binding>, Expression)>, Option<Token>) {
        let left_brace = self.consume_or_recover(
            TokenKind::LeftBrace,
            shape.opener,
            shape.note,
            &[
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ],
        );
        let body_start = self.cursor;
        let bindings = if left_brace.is_some() {
            self.parse_bindings(0).map(|(bindings, _)| bindings)
        } else {
            None
        };
        let expression = match &bindings {
            Some(bindings)
                if !bindings.is_empty() && self.current_kind() == TokenKind::RightBrace =>
            {
                self.expected("a result expression after the last binding", shape.tail);
                None
            }
            Some(_) => self.parse_expression(0).map(|(expression, _)| expression),
            None => None,
        };
        if expression.is_none() {
            // An error inside a loop's step or a conditional's branch leaves
            // their braces open; recovery skips to the body's own `}`.
            let open = self.open_braces_since(body_start);
            if open > 0
                || !matches!(
                    self.current_kind(),
                    TokenKind::RightBrace | TokenKind::KwSpec | TokenKind::KwImpl | TokenKind::Eof
                )
            {
                self.recover_to_depth(
                    &[
                        TokenKind::RightBrace,
                        TokenKind::KwSpec,
                        TokenKind::KwImpl,
                        TokenKind::Eof,
                    ],
                    open,
                );
            }
        }

        let right_brace = if self.current_kind() == TokenKind::RightBrace {
            self.bump()
        } else {
            if expression.is_some() {
                self.expected("`}` after the body expression", shape.note);
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

        match (bindings, expression) {
            (Some(bindings), Some(expression)) => (Some((bindings, expression)), right_brace),
            _ => (None, right_brace),
        }
    }

    fn parse_typed_body(&mut self) -> (Option<FunctionBody>, Option<Token>) {
        let arrow = self.bump();
        let result_type = self
            .parse_type_syntax("result type", true, 0)
            .map(|(ty, _)| ty);
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

        let (contents, right_brace) = self.parse_body_block(&SPEC_BODY_SHAPE);
        let (bindings, expression) = contents.unzip();

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

    /// Parses the `let` bindings at the start of a typed body, a loop's
    /// step, or a conditional's branch, each type and value enclosed by
    /// `level` nesting levels as the block that holds them is.
    ///
    /// Returns the bindings and the greatest height of their types' moduli
    /// and their values. `let` is recognized by position, not reserved: it
    /// starts a binding only when an identifier follows it, which no
    /// expression allows.
    fn parse_bindings(&mut self, level: usize) -> Option<(Vec<Binding>, usize)> {
        let mut bindings = Vec::new();
        let mut height = 0_usize;
        while self.current_is_word("let") && self.starts_binding() {
            let (binding, binding_height) = self.parse_binding(level)?;
            if bindings.len() >= MAX_BINDINGS_PER_BODY {
                self.resource_limit_at(
                    if level == 0 {
                        format!("typed body declares more than {MAX_BINDINGS_PER_BODY} bindings")
                    } else {
                        format!("a block declares more than {MAX_BINDINGS_PER_BODY} bindings")
                    },
                    binding.span,
                );
                return None;
            }
            if !(self.reserve_binding_slot)(&mut bindings) {
                self.resource_limit_at("parser could not allocate binding storage", binding.span);
                return None;
            }
            height = height.max(binding_height);
            bindings.push(binding);
        }
        Some((bindings, height))
    }

    /// Returns whether the `let` at the cursor starts a binding.
    ///
    /// `let` is recognized by position: it starts a binding when a name
    /// follows it, or a tuple pattern whose first element is `name:`, which
    /// no call of a function named `let` can begin with. A parenthesized
    /// list of names followed by `=`, such as `let (a, b) =`, is a tuple
    /// pattern whose types are missing, since no expression is followed by
    /// `=`; it is parsed as a binding so that the missing type is reported.
    fn starts_binding(&self) -> bool {
        let at = |offset: usize| self.kind_at(self.cursor.saturating_add(offset));
        match self.next_kind() {
            TokenKind::Identifier => true,
            TokenKind::LeftParen if at(2) == TokenKind::Identifier && at(3) == TokenKind::Colon => {
                true
            }
            TokenKind::LeftParen => {
                // At most one name for each element of the longest pattern.
                let mut offset = 2_usize;
                for _ in 0..=MAX_TUPLE_ELEMENTS {
                    match at(offset) {
                        TokenKind::Identifier => offset = offset.saturating_add(1),
                        TokenKind::RightParen => break,
                        _ => return false,
                    }
                    match at(offset) {
                        TokenKind::Comma => offset = offset.saturating_add(1),
                        TokenKind::RightParen => break,
                        _ => return false,
                    }
                }
                at(offset) == TokenKind::RightParen
                    && at(offset.saturating_add(1)) == TokenKind::Equal
            }
            _ => false,
        }
    }

    fn parse_binding(&mut self, level: usize) -> Option<(Binding, usize)> {
        let keyword = self.bump()?;
        let (pattern, type_height) = self.parse_pattern(&BINDING_ROLE, level)?;
        self.expect(
            TokenKind::Equal,
            "`=` after the binding's type",
            "a binding is written `let name: Type = expression;`",
        )?;
        let (value, value_height) = self.parse_expression(level)?;
        let semicolon = self.expect(
            TokenKind::Semicolon,
            "`;` after the bound expression",
            if level == 0 {
                "each binding ends with `;`; the body's last item is its result expression"
            } else {
                "each binding ends with `;`; the block's last item is its value"
            },
        )?;
        let span = self.join(keyword.span, semicolon.span);
        self.record_node().then_some((
            Binding {
                span,
                pattern,
                value,
            },
            type_height.max(value_height),
        ))
    }

    /// Parses what a binding or an accumulator names: `name: Type`, or a
    /// tuple pattern `(a: T, b: U, ...)` of two through
    /// [`MAX_TUPLE_ELEMENTS`] typed names.
    ///
    /// Returns the pattern and the greatest height of its types.
    #[inline(never)]
    fn parse_pattern(&mut self, role: &PatternRole, level: usize) -> Option<(Pattern, usize)> {
        if self.current_kind() != TokenKind::LeftParen {
            let (name, height) = self.parse_typed_name(role, level)?;
            return Some((Pattern::Name(Box::new(name)), height));
        }
        let left_paren = self.bump()?.span;
        let mut elements = Vec::new();
        let mut height = 0_usize;
        loop {
            let (element, element_height) = self.parse_pattern_element(role, level)?;
            if elements.len() >= MAX_TUPLE_ELEMENTS {
                self.resource_limit_at(
                    format!("a tuple pattern names more than {MAX_TUPLE_ELEMENTS} values"),
                    element.span,
                );
                return None;
            }
            if !(self.reserve_name_slot)(&mut elements) {
                self.resource_limit_at("parser could not allocate pattern storage", element.span);
                return None;
            }
            height = height.max(element_height);
            elements.push(element);
            match self.current_kind() {
                TokenKind::Comma if elements.len() == 1 => {
                    self.bump()?;
                }
                TokenKind::Comma => {
                    self.bump()?;
                    // A trailing comma before `)` is permitted.
                    if self.current_kind() == TokenKind::RightParen {
                        break;
                    }
                }
                TokenKind::RightParen if elements.len() >= 2 => break,
                TokenKind::RightParen => {
                    self.expected("`,` and another name in the pattern", PATTERN_NOTE);
                    return None;
                }
                _ => {
                    self.expected("`,` or `)` after the pattern's element", PATTERN_NOTE);
                    return None;
                }
            }
        }
        let right_paren = self.bump()?.span;
        let span = self.join(left_paren, right_paren);
        self.record_node().then_some((
            Pattern::Tuple(Box::new(TuplePattern { span, elements })),
            height,
        ))
    }

    /// Parses one `name: Type` of a tuple pattern.
    fn parse_pattern_element(
        &mut self,
        role: &PatternRole,
        level: usize,
    ) -> Option<(TypedName, usize)> {
        if self.current_kind() != TokenKind::Identifier {
            self.expected("a name for the pattern's next value", PATTERN_NOTE);
            return None;
        }
        if self.next_kind() != TokenKind::Colon {
            self.bump()?;
            self.expected("`:` and the type of the name", PATTERN_NOTE);
            return None;
        }
        self.parse_typed_name(role, level)
    }

    fn parse_typed_name(&mut self, role: &PatternRole, level: usize) -> Option<(TypedName, usize)> {
        let name = self.parse_identifier(role.name)?;
        self.expect(TokenKind::Colon, role.colon, role.colon_note)?;
        let (ty, height) = self.parse_type_syntax(role.ty, true, level)?;
        let span = self.join(name.span, ty.span);
        Some((TypedName { span, name, ty }, height))
    }

    /// Parses the `let` bindings, if any, at the start of a loop's step or a
    /// conditional's branch, after its `{`, and checks that a value follows
    /// them.
    ///
    /// Returns the bindings and their greatest height. The caller parses the
    /// value itself, so that a step or branch nested in a value adds no
    /// frame of this function to the stack.
    #[inline(never)]
    fn parse_block_bindings(&mut self, level: usize, note: &str) -> Option<(Vec<Binding>, usize)> {
        let (bindings, height) = self.parse_bindings(level)?;
        if !bindings.is_empty() && self.current_kind() == TokenKind::RightBrace {
            self.expected("a value after the last binding", note);
            return None;
        }
        Some((bindings, height))
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
            Some(joiner @ (Joiner::As | Joiner::With)) => {
                expression = self.parse_postfix(joiner, expression, level)?;
                joiner
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
                group @ (BinaryOperator::And
                | BinaryOperator::Or
                | BinaryOperator::Xor
                | BinaryOperator::LogicalAnd
                | BinaryOperator::LogicalOr
                | BinaryOperator::Concat),
            )) => {
                while self.current_kind() == group.token_kind() {
                    let operator_span = self.bump()?.span;
                    let operand = self.parse_unary(level)?;
                    expression = self.binary_node(expression, group, operator_span, operand)?;
                }
                Joiner::Binary(group)
            }
            // Shifts, rotations, divisions, and comparisons take exactly
            // two operands.
            Some(Joiner::Binary(group)) => {
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
                    Joiner::As | Joiner::With => self.parse_postfix(joiner, expression, level)?,
                };
            }
        }
        Some(expression)
    }

    fn current_joiner(&self) -> Option<Joiner> {
        if self.current_is_word("as") {
            return Some(Joiner::As);
        }
        // `with` is recognized by position, as `as` is: it updates an array
        // only when `[` follows it, which no operand allows.
        if self.current_is_word("with") && self.next_kind() == TokenKind::LeftBracket {
            return Some(Joiner::With);
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
                (_, Joiner::With) | (Joiner::With, _) => {
                    "`with` updates exactly one array; parenthesize the update or the \
                     expression it updates"
                }
                (Joiner::As, Joiner::Binary(BinaryOperator::Xor)) => {
                    "`as` converts exactly one operand; parenthesize the conversion or the \
                     expression it converts. For an array type, name a byte order first, as \
                     in `as big Word[32]^16`"
                }
                (Joiner::As, _) | (_, Joiner::As) => {
                    "`as` converts exactly one operand; parenthesize the conversion or the \
                     expression it converts"
                }
                (Joiner::Binary(previous), Joiner::Binary(ungrouped))
                    if previous.is_shift_or_rotation() && ungrouped.is_shift_or_rotation() =>
                {
                    "a shift or rotation takes exactly two operands; parenthesize one of them"
                }
                (Joiner::Binary(previous), Joiner::Binary(ungrouped))
                    if previous.is_division() && ungrouped.is_division() =>
                {
                    "`/` and `%` take exactly two operands; parenthesize one of them"
                }
                (Joiner::Binary(previous), Joiner::Binary(ungrouped))
                    if previous.is_comparison() && ungrouped.is_comparison() =>
                {
                    "a comparison takes exactly two operands; join two comparisons with `&&` \
                     or `||`"
                }
                (Joiner::Binary(_), Joiner::Binary(_)) => {
                    "operators from different groups have no relative precedence in Orange; \
                     parenthesize the part that applies first"
                }
            })
        });
    }

    /// Parses the conversion or update that `joiner` starts after a
    /// complete operand. One call site per joiner position keeps the frame
    /// of [`Self::parse_expression`], which recurses, small.
    #[inline(never)]
    fn parse_postfix(
        &mut self,
        joiner: Joiner,
        operand: (Expression, usize),
        level: usize,
    ) -> Option<(Expression, usize)> {
        match joiner {
            Joiner::With => self.parse_update(operand, level),
            Joiner::As | Joiner::Binary(_) => self.parse_conversion(operand, level),
        }
    }

    /// Parses `as Type`, `as big Type`, or `as little Type` after a
    /// complete operand. Only a conversion with a byte order may name an
    /// array type, so that `x as Word[8] ^ y` stays an ungrouped operator.
    #[inline(never)]
    fn parse_conversion(
        &mut self,
        (operand, operand_height): (Expression, usize),
        level: usize,
    ) -> Option<(Expression, usize)> {
        let keyword_span = self.bump()?.span;
        let order = match self.byte_order() {
            Some(order) => Some((order, self.bump()?.span)),
            None => None,
        };
        let (target, target_height) =
            self.parse_type_syntax("conversion type", order.is_some(), level)?;
        let height = self.node_height(operand_height.max(target_height), keyword_span)?;
        let span = self.join(operand.span, target.span);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Conversion(Box::new(ConversionExpression {
                    operand,
                    keyword_span,
                    order,
                    target,
                })),
            },
            height,
        ))
    }

    /// Returns the byte order the current token names after `as`: an
    /// identifier spelled `big` or `little` followed by the start of a type,
    /// `(` or an identifier other than `as` and `with`. Anything else after
    /// `as` is the type itself, so a type named `big` is still converted to
    /// with `as big`.
    fn byte_order(&self) -> Option<ByteOrder> {
        let order = if self.current_is_word("big") {
            ByteOrder::Big
        } else if self.current_is_word("little") {
            ByteOrder::Little
        } else {
            return None;
        };
        let next = self.cursor.saturating_add(1);
        let starts_type = match self.kind_at(next) {
            TokenKind::LeftParen => true,
            TokenKind::Identifier => !self.is_word_at(next, "as") && !self.is_word_at(next, "with"),
            _ => false,
        };
        starts_type.then_some(order)
    }

    /// Parses `with [index] = value` or `with [start..end] = value` after a
    /// complete operand. The value extends as far as an expression can, so
    /// an update ends its operator chain. Whichever form the brackets hold,
    /// the value is parsed in this frame, so updates nested in values cost
    /// one frame each.
    #[inline(never)]
    fn parse_update(
        &mut self,
        base: (Expression, usize),
        level: usize,
    ) -> Option<(Expression, usize)> {
        let inner = self.open_level(level)?;
        let keyword_span = self.bump()?.span;
        self.bump()?;
        let target = self.parse_update_target(inner)?;
        let value = self.parse_expression(inner)?;
        self.update_node(base, keyword_span, target, value)
    }

    /// Parses what an update replaces, through its `=`: `index] =`, or the
    /// bounds of a slice and `] =`.
    #[inline(never)]
    fn parse_update_target(&mut self, inner: usize) -> Option<UpdateTarget> {
        let start = if self.current_kind() == TokenKind::DotDot {
            None
        } else {
            Some(self.parse_expression(inner)?)
        };
        if self.current_kind() == TokenKind::DotDot {
            let (range, height) = self.parse_slice_range(start, inner, SLICE_UPDATE_NOTE)?;
            self.expect(
                TokenKind::RightBracket,
                "`]` after the slice",
                SLICE_UPDATE_NOTE,
            )?;
            self.expect(
                TokenKind::Equal,
                "`=` after the updated slice",
                SLICE_UPDATE_NOTE,
            )?;
            return Some(UpdateTarget::Slice(range, height));
        }
        // Without `..` first, an index was parsed.
        let (index, mut height) = start?;
        self.expect(
            TokenKind::RightBracket,
            "`]` after the index",
            "an update is written `x with [i] = value`",
        )?;
        // Each further `[j]` selects within the element the one before it
        // reached, one dimension deeper.
        let mut path = Vec::new();
        while self.current_kind() == TokenKind::LeftBracket {
            if path.len() >= UPDATE_INDICES.saturating_sub(1) {
                self.expected("`=` after the updated index", PATH_UPDATE_NOTE);
                return None;
            }
            self.bump()?;
            let (inner_index, inner_height) = self.parse_expression(inner)?;
            if self.current_kind() == TokenKind::DotDot {
                self.expected("`]` after the index", PATH_UPDATE_NOTE);
                return None;
            }
            self.expect(
                TokenKind::RightBracket,
                "`]` after the index",
                PATH_UPDATE_NOTE,
            )?;
            if path.try_reserve(1).is_err() {
                self.resource_limit_at(
                    "parser could not allocate update storage",
                    inner_index.span,
                );
                return None;
            }
            height = height.max(inner_height);
            path.push(inner_index);
        }
        self.expect(
            TokenKind::Equal,
            "`=` after the updated index",
            "an update is written `x with [i] = value`, or `x with [i][j] = value` for an \
             element of a row",
        )?;
        Some(UpdateTarget::Index(index, path, height))
    }

    /// Builds the update of `base` at `target` with `value`.
    #[inline(never)]
    fn update_node(
        &mut self,
        (base, base_height): (Expression, usize),
        keyword_span: Span,
        target: UpdateTarget,
        (value, value_height): (Expression, usize),
    ) -> Option<(Expression, usize)> {
        let target_height = match &target {
            UpdateTarget::Index(_, _, height) | UpdateTarget::Slice(_, height) => *height,
        };
        let height = self.node_height(
            base_height.max(target_height).max(value_height),
            keyword_span,
        )?;
        let span = self.join(base.span, value.span);
        let kind = match target {
            UpdateTarget::Index(index, path, _) => {
                ExpressionKind::Update(Box::new(UpdateExpression {
                    base,
                    keyword_span,
                    index,
                    path,
                    value,
                }))
            }
            UpdateTarget::Slice(range, _) => {
                ExpressionKind::SliceUpdate(Box::new(SliceUpdateExpression {
                    base,
                    keyword_span,
                    range,
                    value,
                }))
            }
        };
        self.record_node()
            .then_some((Expression { span, kind }, height))
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
                 for groups, calls, arrays, indices, loops, conditionals, updates, moduli, and \
                 prefix operators"
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
            TokenKind::Bang => UnaryOperator::Not,
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
            // `for` is recognized by position, as `let` is: it starts a loop
            // only when an identifier follows it, which no expression allows.
            TokenKind::Identifier
                if self.current_is_word("for") && self.next_kind() == TokenKind::Identifier =>
            {
                self.parse_loop(level)
            }
            TokenKind::Identifier if self.current_is_word("if") && self.starts_conditional() => {
                self.parse_conditional(level)
            }
            TokenKind::Identifier
                if matches!(
                    self.next_kind(),
                    TokenKind::LeftParen | TokenKind::DoubleColon
                ) || self.starts_sized_call() =>
            {
                let call = self.parse_call(level)?;
                self.parse_index_suffix(call, level)
            }
            TokenKind::Identifier => {
                let name = self.parse_name_expression()?;
                self.parse_index_suffix(name, level)
            }
            TokenKind::LeftParen => {
                let inner = self.open_level(level)?;
                let left_paren = self.bump()?.span;
                let group = self.parse_expression(inner)?;
                self.finish_group(left_paren, group, inner)
            }
            TokenKind::LeftBracket => self.parse_array(level),
            TokenKind::String | TokenKind::HexString => self.parse_byte_string(),
            _ => {
                self.expected(
                    "an expression",
                    "an expression is an integer literal, a name, a call, an array, a loop, a \
                     conditional, a prefix operator, or a parenthesized expression",
                );
                None
            }
        }
    }

    /// Parses an optional `.k` and then `[index]` selections after a name
    /// or a call.
    ///
    /// An index that is one integer token opens no nesting level, exactly as
    /// in S3d; any other index is an expression one level deeper.
    #[inline(never)]
    fn parse_index_suffix(
        &mut self,
        (base, base_height): (Expression, usize),
        level: usize,
    ) -> Option<(Expression, usize)> {
        if self.current_kind() == TokenKind::Dot {
            return self.parse_projection(base, base_height, level);
        }
        let mut selected = (base, base_height);
        // Each selection increases the checked tree height; a chain uses
        // one parser frame rather than one frame per dimension.
        while self.current_kind() == TokenKind::LeftBracket {
            selected = self.parse_array_index(selected, level)?;
        }
        Some(selected)
    }

    #[inline(never)]
    fn parse_array_index(
        &mut self,
        (base, base_height): (Expression, usize),
        level: usize,
    ) -> Option<(Expression, usize)> {
        let left_bracket = self.bump()?.span;
        let (index, index_height) = if self.current_kind() == TokenKind::Integer
            && self.next_kind() == TokenKind::RightBracket
        {
            self.parse_literal_expression()?
        } else if self.current_kind() == TokenKind::RightBracket {
            self.expected(
                "an index after `[`",
                "an array element is selected by an index, such as `x[0]` or `x[i + 1]`",
            );
            return None;
        } else {
            let inner = level.saturating_add(1);
            if inner > MAX_EXPRESSION_NESTING {
                self.nesting_limit(left_bracket);
                return None;
            }
            if self.current_kind() == TokenKind::DotDot {
                return self.finish_slice((base, base_height), left_bracket, None, inner);
            }
            let index = self.parse_expression(inner)?;
            if self.current_kind() == TokenKind::DotDot {
                return self.finish_slice((base, base_height), left_bracket, Some(index), inner);
            }
            index
        };
        if self.current_kind() != TokenKind::RightBracket {
            self.expected(
                "`]` after the index",
                "an array element is selected by an index, such as `x[0]` or `x[i + 1]`",
            );
            return None;
        }
        let right_bracket = self.bump()?.span;
        if self.current_kind() == TokenKind::Dot {
            self.expected(
                "an operator or the end of the expression",
                "an array's elements are not tuples, so an element has no `.k`",
            );
            return None;
        }
        let height = self.node_height(base_height.max(index_height), left_bracket)?;
        let span = self.join(base.span, right_bracket);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Index(Box::new(IndexExpression { base, index })),
            },
            height,
        ))
    }

    /// Parses the rest of a slice `base[start..end]` at its `..`, after its
    /// start when one is written. Both bounds are at the level `inner` that
    /// the brackets open.
    #[inline(never)]
    fn finish_slice(
        &mut self,
        (base, base_height): (Expression, usize),
        left_bracket: Span,
        start: Option<(Expression, usize)>,
        inner: usize,
    ) -> Option<(Expression, usize)> {
        let (range, range_height) = self.parse_slice_range(start, inner, SLICE_NOTE)?;
        if self.current_kind() != TokenKind::RightBracket {
            self.expected("`]` after the slice", SLICE_NOTE);
            return None;
        }
        let right_bracket = self.bump()?.span;
        if self.current_kind() == TokenKind::LeftBracket {
            self.expected(
                "an operator or the end of the expression",
                "a slice is taken once, from a name, a call, or a tuple's element; bind it with \
                 `let` to select from it",
            );
            return None;
        }
        if self.current_kind() == TokenKind::Dot {
            self.expected(
                "an operator or the end of the expression",
                "a slice is an array, not a tuple, so it has no `.k`",
            );
            return None;
        }
        let height = self.node_height(base_height.max(range_height), left_bracket)?;
        let span = self.join(base.span, right_bracket);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Slice(Box::new(SliceExpression { base, range })),
            },
            height,
        ))
    }

    /// Parses the bounds `start..end` of a slice at `..`, after its start
    /// when one is written, and returns them with the height of the taller
    /// bound, or 0 when neither is written. At least one bound is required;
    /// `note` shows the form when none is.
    #[inline(never)]
    fn parse_slice_range(
        &mut self,
        start: Option<(Expression, usize)>,
        inner: usize,
        note: &str,
    ) -> Option<(SliceRange, usize)> {
        let dots_span = self.bump()?.span;
        let end = if self.current_kind() == TokenKind::RightBracket {
            None
        } else {
            Some(self.parse_expression(inner)?)
        };
        if start.is_none() && end.is_none() {
            self.expected("a bound of the slice after `..`", note);
            return None;
        }
        let first = start.as_ref().map_or(dots_span, |(start, _)| start.span);
        let last = end.as_ref().map_or(dots_span, |(end, _)| end.span);
        let span = self.join(first, last);
        let height = start
            .as_ref()
            .map_or(0, |(_, height)| *height)
            .max(end.as_ref().map_or(0, |(_, height)| *height));
        Some((
            SliceRange {
                span,
                start: start.map(|(start, _)| start),
                dots_span,
                end: end.map(|(end, _)| end),
            },
            height,
        ))
    }

    /// Parses `.k` after a name or a call, the element of a tuple at
    /// position `k`, and then an optional `[index]` after it.
    #[inline(never)]
    fn parse_projection(
        &mut self,
        base: Expression,
        base_height: usize,
        level: usize,
    ) -> Option<(Expression, usize)> {
        let dot = self.bump()?.span;
        let token = self.expect(
            TokenKind::Integer,
            "an element's position after `.`",
            PROJECTION_NOTE,
        )?;
        let text = token.lexeme(self.source)?;
        let decimal = text.bytes().all(|byte| byte.is_ascii_digit())
            && (text == "0" || !text.starts_with('0'));
        if !decimal {
            self.report(
                DiagnosticCode::ExpectedSyntax,
                "expected an element's position in decimal",
                token.span,
                "found INTEGER",
                PROJECTION_NOTE,
            );
            return None;
        }
        let position = text.parse::<u32>().unwrap_or(u32::MAX);
        if self.current_kind() == TokenKind::Dot {
            self.expected(
                "an operator or the end of the expression",
                "a tuple's elements are not tuples, so an element is selected once",
            );
            return None;
        }
        let height = self.node_height(base_height, dot)?;
        let span = self.join(base.span, token.span);
        if !self.record_node() {
            return None;
        }
        let projection = Expression {
            span,
            kind: ExpressionKind::Project(Box::new(ProjectExpression {
                base,
                position_span: token.span,
                position,
            })),
        };
        // The next token is not `.`, so this parses at most an index.
        self.parse_index_suffix((projection, height), level)
    }

    /// Parses the rest of a tuple `(e0, e1, ...)` after its first element,
    /// at the `,` that follows it.
    ///
    /// A tuple has two through [`MAX_TUPLE_ELEMENTS`] elements; a trailing
    /// comma is permitted after the second.
    #[inline(never)]
    fn parse_tuple(
        &mut self,
        left_paren: Span,
        (first, first_height): (Expression, usize),
        inner: usize,
    ) -> Option<(Expression, usize)> {
        let mut elements = Vec::new();
        if !self.push_tuple_element(&mut elements, first) {
            return None;
        }
        let mut element_height = first_height;
        while self.current_kind() == TokenKind::Comma {
            self.bump()?;
            if self.current_kind() == TokenKind::RightParen {
                if elements.len() >= 2 {
                    break;
                }
                self.expected("another element after `,`", TUPLE_NOTE);
                return None;
            }
            let (element, height) = self.parse_expression(inner)?;
            element_height = element_height.max(height);
            if !self.push_tuple_element(&mut elements, element) {
                return None;
            }
        }
        let right_paren = self
            .expect(
                TokenKind::RightParen,
                "`,` or `)` after the tuple's element",
                TUPLE_NOTE,
            )?
            .span;
        let height = self.node_height(element_height, left_paren)?;
        let span = self.join(left_paren, right_paren);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Tuple(Box::new(TupleExpression { elements })),
            },
            height,
        ))
    }

    fn push_tuple_element(&mut self, elements: &mut Vec<Expression>, element: Expression) -> bool {
        if elements.len() >= MAX_TUPLE_ELEMENTS {
            self.resource_limit_at(
                format!("a tuple has more than {MAX_TUPLE_ELEMENTS} elements"),
                element.span,
            );
            return false;
        }
        if !(self.reserve_argument_slot)(elements) {
            self.resource_limit_at("parser could not allocate tuple storage", element.span);
            return false;
        }
        elements.push(element);
        true
    }

    /// Parses an array literal `[e0, e1, ...]` with at least one element and
    /// an optional trailing comma.
    fn parse_array(&mut self, level: usize) -> Option<(Expression, usize)> {
        let inner = self.open_level(level)?;
        let left_bracket = self.bump()?.span;
        let mut elements = Vec::new();
        let mut element_height = 0_usize;
        loop {
            if self.current_kind() == TokenKind::RightBracket {
                if !elements.is_empty() {
                    break;
                }
                self.expected(
                    "an array element",
                    "an array has at least one element; Orange 2026 has no empty arrays",
                );
                return None;
            }
            let element = self.parse_expression(inner)?;
            if elements.is_empty() && self.current_kind() == TokenKind::Semicolon {
                return self.finish_fill(left_bracket, element, inner);
            }
            element_height = element_height.max(element.1);
            if !self.push_element(&mut elements, element.0) {
                return None;
            }
            match self.current_kind() {
                TokenKind::Comma => {
                    self.bump()?;
                }
                TokenKind::RightBracket => break,
                _ => {
                    self.expected(
                        "`,` or `]` after the array element",
                        "array elements are separated by commas",
                    );
                    return None;
                }
            }
        }
        let right_bracket = self.bump()?.span;
        let height = self.node_height(element_height, left_bracket)?;
        let span = self.join(left_bracket, right_bracket);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Array(Box::new(ArrayExpression { elements })),
            },
            height,
        ))
    }

    /// Parses `; n]` after the element of a fill literal `[element; n]`,
    /// whose element is at the level `inner` that the brackets open.
    #[inline(never)]
    fn finish_fill(
        &mut self,
        left_bracket: Span,
        (element, element_height): (Expression, usize),
        inner: usize,
    ) -> Option<(Expression, usize)> {
        self.bump()?;
        let (length, length_height) = self.parse_size(
            inner,
            "an array length after `;`",
            "`[e; n]` is the array of n copies of e, such as `[0; 64]`",
        )?;
        if self.size_continues() {
            self.expected("`]` after the array length", COMPUTED_FILL_NOTE);
            return None;
        }
        let right_bracket = self
            .expect(
                TokenKind::RightBracket,
                "`]` after the array length",
                "`[e; n]` is the array of n copies of e, such as `[0; 64]`",
            )?
            .span;
        let height = self.node_height(element_height.max(length_height), left_bracket)?;
        let span = self.join(left_bracket, right_bracket);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Fill(Box::new(FillExpression { element, length })),
            },
            height,
        ))
    }

    /// Parses `for i in a..b with s: Type = init { step }`.
    ///
    /// The initial value and the step are one nesting level deeper than the
    /// loop. Only this function's small frame stays on the stack while they
    /// are parsed; the header and the node are built by helpers.
    fn parse_loop(&mut self, level: usize) -> Option<(Expression, usize)> {
        let inner = self.open_level(level)?;
        let (header, type_height) = self.parse_loop_header(inner)?;
        let init = self.parse_expression(inner)?;
        self.expect(
            TokenKind::LeftBrace,
            "`{` before the loop's step",
            LOOP_SHAPE_NOTE,
        )?;
        let step_bindings = self.parse_block_bindings(inner, STEP_SHAPE_NOTE)?;
        let step = self.parse_expression(inner)?;
        self.finish_loop(header, type_height, init, step_bindings, step)
    }

    /// Parses a loop from `for` through the `=` before its initial value.
    #[inline(never)]
    ///
    /// Also returns the height of the accumulator type's modulus, or zero.
    fn parse_loop_header(&mut self, level: usize) -> Option<(Box<LoopExpression>, usize)> {
        let keyword_span = self.bump()?.span;
        let index = self.parse_identifier("loop index")?;
        if !self.current_is_word("in") {
            self.expected("`in` after the loop index", LOOP_SHAPE_NOTE);
            return None;
        }
        self.bump()?;
        let (start, start_height) =
            self.parse_size(level, "the loop's first bound", LOOP_SHAPE_NOTE)?;
        if self.size_continues() {
            self.expected("`..` between the loop's bounds", COMPUTED_BOUND_NOTE);
            return None;
        }
        self.expect(
            TokenKind::DotDot,
            "`..` between the loop's bounds",
            LOOP_SHAPE_NOTE,
        )?;
        let (end, end_height) =
            self.parse_size(level, "the loop's second bound", LOOP_SHAPE_NOTE)?;
        if self.size_continues() {
            self.expected("`with` and the loop's accumulator", COMPUTED_BOUND_NOTE);
            return None;
        }
        if !self.current_is_word("with") {
            self.expected("`with` and the loop's accumulator", LOOP_SHAPE_NOTE);
            return None;
        }
        self.bump()?;
        let (accumulator, type_height) = self.parse_pattern(&ACCUMULATOR_ROLE, level)?;
        let type_height = type_height.max(start_height).max(end_height);
        self.expect(
            TokenKind::Equal,
            "`=` after the accumulator's type",
            LOOP_SHAPE_NOTE,
        )?;
        // The initial value and the step are placeholders until parsed.
        let placeholder = Expression {
            span: keyword_span,
            kind: ExpressionKind::Name(index.clone()),
        };
        Some((
            Box::new(LoopExpression {
                keyword_span,
                index,
                start,
                end,
                accumulator,
                init: placeholder.clone(),
                step_bindings: Vec::new(),
                step: placeholder,
            }),
            type_height,
        ))
    }

    /// Completes a loop after its step: the closing `}`, the tree height,
    /// and the node.
    #[inline(never)]
    fn finish_loop(
        &mut self,
        mut header: Box<LoopExpression>,
        type_height: usize,
        (init, init_height): (Expression, usize),
        (step_bindings, bindings_height): (Vec<Binding>, usize),
        (step, step_height): (Expression, usize),
    ) -> Option<(Expression, usize)> {
        let right_brace = self
            .expect(
                TokenKind::RightBrace,
                "`}` after the loop's step",
                STEP_SHAPE_NOTE,
            )?
            .span;
        let height = self.node_height(
            init_height
                .max(bindings_height)
                .max(step_height)
                .max(type_height),
            header.keyword_span,
        )?;
        let span = self.join(header.keyword_span, right_brace);
        header.init = init;
        header.step_bindings = step_bindings;
        header.step = step;
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Loop(header),
            },
            height,
        ))
    }

    /// Returns whether the `if` at the cursor starts a conditional.
    ///
    /// `if` is recognized by position, as `for` is. Before an integer, `!`,
    /// `~`, or an identifier that does not continue an expression, a name
    /// `if` could not stand, so `if` starts a conditional. Before `(`, `-`,
    /// or `[`, `if` could also be a name that is called, subtracted from, or
    /// indexed, so the tokens after it are examined: `if` starts a
    /// conditional exactly when a brace group closed at its own depth is
    /// followed directly by `else`, which follows `}` nowhere else. The
    /// examination costs one parser event per token and stops at the end of
    /// the expression `if` stands in.
    fn starts_conditional(&mut self) -> bool {
        let next = self.cursor.saturating_add(1);
        match self.kind_at(next) {
            // After the word `as`, or `with` before `[`, an `if` named as a
            // value may continue; it is a conditional only if its brace
            // group is followed by `else`.
            TokenKind::Identifier => {
                let continues = self.is_word_at(next, "as")
                    || (self.is_word_at(next, "with")
                        && self.kind_at(next.saturating_add(1)) == TokenKind::LeftBracket);
                !continues || self.else_follows(next)
            }
            TokenKind::Integer | TokenKind::Bang | TokenKind::Tilde => true,
            TokenKind::LeftParen | TokenKind::Minus | TokenKind::LeftBracket => {
                self.else_follows(next)
            }
            _ => false,
        }
    }

    /// Returns whether a brace group that closes at the depth of `start` is
    /// followed directly by `else` before that depth's expression ends.
    #[inline(never)]
    fn else_follows(&mut self, start: usize) -> bool {
        let mut depth = 0_usize;
        let mut position = start;
        loop {
            if self.halted || !self.event() {
                return false;
            }
            let kind = self.kind_at(position);
            match kind {
                TokenKind::Eof => return false,
                TokenKind::LeftParen | TokenKind::LeftBracket | TokenKind::LeftBrace => {
                    depth = depth.saturating_add(1);
                }
                TokenKind::RightParen | TokenKind::RightBracket | TokenKind::RightBrace => {
                    let Some(outer) = depth.checked_sub(1) else {
                        return false;
                    };
                    depth = outer;
                    if depth == 0
                        && kind == TokenKind::RightBrace
                        && self.is_word_at(position.saturating_add(1), "else")
                    {
                        return true;
                    }
                }
                TokenKind::Semicolon | TokenKind::Comma if depth == 0 => return false,
                _ => {}
            }
            position = position.saturating_add(1);
        }
    }

    /// Parses `if c { a } else if ... else { b }`.
    ///
    /// Every condition and value is one nesting level deeper than the
    /// conditional, and an `else if` arm opens no further level, so this loop
    /// parses a chain of any length. Only this function's small frame stays
    /// on the stack while the parts are parsed.
    fn parse_conditional(&mut self, level: usize) -> Option<(Expression, usize)> {
        let inner = self.open_level(level)?;
        let mut arms = Vec::new();
        let mut height = 0_usize;
        loop {
            let keyword_span = self.bump()?.span;
            let condition = self.parse_expression(inner)?;
            self.expect(
                TokenKind::LeftBrace,
                "`{` after the condition",
                CONDITIONAL_SHAPE_NOTE,
            )?;
            let bindings = self.parse_block_bindings(inner, BRANCH_SHAPE_NOTE)?;
            let value = self.parse_expression(inner)?;
            let else_span = self.close_arm(
                &mut arms,
                &mut height,
                keyword_span,
                condition,
                bindings,
                value,
            )?;
            if self.current_is_word("if") {
                continue;
            }
            self.expect(
                TokenKind::LeftBrace,
                "`{` or `if` after `else`",
                CONDITIONAL_SHAPE_NOTE,
            )?;
            let otherwise_bindings = self.parse_block_bindings(inner, BRANCH_SHAPE_NOTE)?;
            let otherwise = self.parse_expression(inner)?;
            return self.finish_conditional(arms, height, else_span, otherwise_bindings, otherwise);
        }
    }

    /// Completes one arm after its value: the closing `}`, the `else` that
    /// must follow it, and the arm's place in the conditional. Returns the
    /// span of `else`.
    #[inline(never)]
    fn close_arm(
        &mut self,
        arms: &mut Vec<ConditionalArm>,
        height: &mut usize,
        keyword_span: Span,
        (condition, condition_height): (Expression, usize),
        (bindings, bindings_height): (Vec<Binding>, usize),
        (value, value_height): (Expression, usize),
    ) -> Option<Span> {
        self.expect(
            TokenKind::RightBrace,
            "`}` after the value",
            BRANCH_SHAPE_NOTE,
        )?;
        if !self.current_is_word("else") {
            self.expected(
                "`else` and the value when the condition is false",
                "every `if` has an `else`, so that a conditional always has a value",
            );
            return None;
        }
        if !(self.reserve_arm_slot)(arms) {
            self.resource_limit_at(
                "parser could not allocate conditional storage",
                keyword_span,
            );
            return None;
        }
        *height = (*height)
            .max(condition_height)
            .max(bindings_height)
            .max(value_height);
        arms.push(ConditionalArm {
            keyword_span,
            condition,
            bindings,
            value,
        });
        Some(self.bump()?.span)
    }

    /// Completes a conditional after its last value: the closing `}`, the
    /// tree height, and the node.
    #[inline(never)]
    fn finish_conditional(
        &mut self,
        arms: Vec<ConditionalArm>,
        height: usize,
        else_span: Span,
        (otherwise_bindings, bindings_height): (Vec<Binding>, usize),
        (otherwise, otherwise_height): (Expression, usize),
    ) -> Option<(Expression, usize)> {
        let right_brace = self
            .expect(
                TokenKind::RightBrace,
                "`}` after the value",
                BRANCH_SHAPE_NOTE,
            )?
            .span;
        let first = arms.first()?.keyword_span;
        let height = self.node_height(height.max(bindings_height).max(otherwise_height), first)?;
        let span = self.join(first, right_brace);
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Conditional(Box::new(ConditionalExpression {
                    arms,
                    else_span,
                    otherwise_bindings,
                    otherwise,
                })),
            },
            height,
        ))
    }

    #[inline(never)]
    fn push_element(&mut self, elements: &mut Vec<Expression>, element: Expression) -> bool {
        if elements.len() >= MAX_ARRAY_ELEMENTS {
            self.resource_limit_at(
                format!("array literal has more than {MAX_ARRAY_ELEMENTS} elements"),
                element.span,
            );
            return false;
        }
        if !(self.reserve_argument_slot)(elements) {
            self.resource_limit_at("parser could not allocate array storage", element.span);
            return false;
        }
        elements.push(element);
        true
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

    /// Parses a byte string `"..."` or `hex"..."`, whose bytes are decoded
    /// by semantic analysis.
    #[inline(never)]
    fn parse_byte_string(&mut self) -> Option<(Expression, usize)> {
        let hex = self.current_kind() == TokenKind::HexString;
        let token = self.bump()?;
        if self.current_kind() == TokenKind::LeftBracket {
            self.expected(
                "an operator or the end of the expression",
                "a byte string is not indexed or sliced where it is written; bind it with `let` \
                 to select from it",
            );
            return None;
        }
        self.record_node().then_some((
            Expression {
                span: token.span,
                kind: ExpressionKind::Bytes(ByteString { hex }),
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

    /// Finishes a group after its expression, or a tuple when a `,`
    /// follows its first element.
    #[inline(never)]
    fn finish_group(
        &mut self,
        left_paren: Span,
        (inner, inner_height): (Expression, usize),
        level: usize,
    ) -> Option<(Expression, usize)> {
        if self.current_kind() == TokenKind::Comma {
            return self.parse_tuple(left_paren, (inner, inner_height), level);
        }
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
        let module = if self.next_kind() == TokenKind::DoubleColon {
            let module = self.parse_identifier("module")?;
            self.bump()?;
            Some(module)
        } else {
            None
        };
        let callee = self.parse_identifier("called function")?;
        let sizes = if self.current_kind() == TokenKind::LeftBracket {
            self.parse_call_size_list(inner)?
        } else {
            (Vec::new(), 0)
        };
        if self.current_kind() != TokenKind::LeftParen {
            self.expected(
                "`(` after the qualified function name",
                "a name qualified by its module is always called, as in `sha256::initial()`",
            );
            return None;
        }
        self.finish_call(module, callee, sizes, inner)
    }

    /// Parses the sizes `[s0, s1, ...]` of a call at their `[`. The sizes are
    /// at the level `inner` that the call opens. Kept out of line so that a
    /// call's own frame, which every nested call repeats, holds no
    /// expression.
    #[inline(never)]
    fn parse_call_size_list(&mut self, inner: usize) -> Option<(Vec<Expression>, usize)> {
        self.bump()?;
        let first = self.parse_expression(inner)?;
        self.parse_call_sizes(first, inner)
    }

    /// Parses the rest of a call's sizes `[s0, s1, ...]` after its first
    /// size, through the `]`, at most [`MAX_SIZES_PER_FUNCTION`] of them.
    /// A `(` must follow.
    #[inline(never)]
    fn parse_call_sizes(
        &mut self,
        (first, first_height): (Expression, usize),
        inner: usize,
    ) -> Option<(Vec<Expression>, usize)> {
        let mut sizes = Vec::new();
        let mut height = first_height;
        if sizes.try_reserve(1).is_err() {
            self.resource_limit_at("parser could not allocate size storage", first.span);
            return None;
        }
        sizes.push(first);
        while self.current_kind() == TokenKind::Comma {
            self.bump()?;
            let (size, size_height) = self.parse_expression(inner)?;
            if sizes.len() >= MAX_SIZES_PER_FUNCTION {
                self.report(
                    DiagnosticCode::ExpectedSyntax,
                    format!("a call gives at most {MAX_SIZES_PER_FUNCTION} sizes"),
                    size.span,
                    "one size too many",
                    SIZED_CALL_NOTE,
                );
                return None;
            }
            if sizes.try_reserve(1).is_err() {
                self.resource_limit_at("parser could not allocate size storage", size.span);
                return None;
            }
            height = height.max(size_height);
            sizes.push(size);
        }
        self.expect(
            TokenKind::RightBracket,
            "`,` or `]` after the size",
            SIZED_CALL_NOTE,
        )?;
        if self.current_kind() != TokenKind::LeftParen {
            self.expected("`(` after the sizes", SIZED_CALL_NOTE);
            return None;
        }
        Some((sizes, height))
    }

    /// Parses a call's arguments at its `(`, after its name and sizes, and
    /// builds the call. The arguments are at the level `inner` that the
    /// call opens.
    #[inline(never)]
    fn finish_call(
        &mut self,
        module: Option<Identifier>,
        callee: Identifier,
        (sizes, size_height): (Vec<Expression>, usize),
        inner: usize,
    ) -> Option<(Expression, usize)> {
        self.bump()?;
        let mut arguments = Vec::new();
        let mut argument_height = size_height;
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
        let start = module.as_ref().map_or(callee.span, |module| module.span);
        let span = self.join(start, right_paren.span);
        let qualifiers = if module.is_some() || !sizes.is_empty() {
            Some(Box::new(CallQualifiers { module, sizes }))
        } else {
            None
        };
        self.record_node().then_some((
            Expression {
                span,
                kind: ExpressionKind::Call(CallExpression {
                    qualifiers,
                    callee,
                    arguments,
                }),
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

    /// Returns whether a name followed by `[` starts a call of a function
    /// with sizes or types, `name[sizes](arguments)`: whether the brackets
    /// hold only tokens a list of sizes and types can hold (integers, names,
    /// `+`, `-`, `*`, `/`, `%`, `^`, parentheses, commas, a name's `[n]`,
    /// as in `Word[32]`, and `Mod[expression]` with `<<`) and a `(` follows
    /// them. Nested brackets within a modulus stop the scan. Thus the
    /// scans of all a source's brackets read each token at most twice.
    fn starts_sized_call(&self) -> bool {
        if self.next_kind() != TokenKind::LeftBracket {
            return false;
        }
        let mut position = self.cursor.saturating_add(2);
        let mut depth = 0_usize;
        let mut modulus = false;
        loop {
            match self.kind_at(position) {
                TokenKind::Integer
                | TokenKind::Identifier
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Caret
                | TokenKind::Comma => {}
                TokenKind::LeftParen => depth = depth.saturating_add(1),
                TokenKind::RightParen if depth > 0 => depth = depth.saturating_sub(1),
                TokenKind::LessLess if modulus => {}
                TokenKind::LeftBracket
                    if !modulus && self.is_word_at(position.saturating_sub(1), "Mod") =>
                {
                    modulus = true;
                }
                TokenKind::RightBracket if modulus => modulus = false,
                // A word type given to a type parameter, as in `ch[Word[32]](e, f, g)`.
                TokenKind::LeftBracket
                    if !modulus
                        && self.kind_at(position.saturating_sub(1)) == TokenKind::Identifier
                        && self.kind_at(position.saturating_add(1)) == TokenKind::Integer
                        && self.kind_at(position.saturating_add(2)) == TokenKind::RightBracket =>
                {
                    position = position.saturating_add(2);
                }
                TokenKind::RightBracket if depth == 0 => {
                    return self.kind_at(position.saturating_add(1)) == TokenKind::LeftParen;
                }
                _ => return false,
            }
            position = position.saturating_add(1);
        }
    }

    /// Returns whether an arithmetic operator follows a size, as when a
    /// computed size is written without its parentheses.
    fn size_continues(&self) -> bool {
        matches!(
            self.current_kind(),
            TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
        )
    }

    /// Parses a size: an integer token, a name, or a parenthesized
    /// expression one nesting level deeper than `level`, and returns it with
    /// its tree height. `what` and `note` describe the size when none is
    /// written. The word `with` is never a size, so a loop that leaves out
    /// its second bound is reported there.
    #[inline(never)]
    fn parse_size(&mut self, level: usize, what: &str, note: &str) -> Option<(Size, usize)> {
        match self.current_kind() {
            TokenKind::Integer => {
                let span = self.bump()?.span;
                Some((
                    Size {
                        span,
                        expression: None,
                    },
                    0,
                ))
            }
            TokenKind::Identifier if !self.current_is_word("with") => {
                let (name, height) = self.parse_name_expression()?;
                Some((
                    Size {
                        span: name.span,
                        expression: Some(Box::new(name)),
                    },
                    height,
                ))
            }
            TokenKind::LeftParen => {
                let inner = self.open_level(level)?;
                let left_paren = self.bump()?.span;
                let group = self.parse_expression(inner)?;
                let (group, height) = self.finish_group(left_paren, group, inner)?;
                Some((
                    Size {
                        span: group.span,
                        expression: Some(Box::new(group)),
                    },
                    height,
                ))
            }
            _ => {
                self.expected(what, note);
                None
            }
        }
    }

    /// Parses `Name`, `Name[WIDTH]`, `Mod[MODULUS]`, and, when `array` is
    /// set, any of them followed by `^LENGTH`. A conversion target is never
    /// an array, so a `^` after it stays an operator. A modulus is an
    /// expression one nesting level deeper than `level`; its tree height, or
    /// zero without one, is returned with the type so that an expression
    /// holding the type counts it toward [`MAX_EXPRESSION_HEIGHT`].
    fn parse_type_syntax(
        &mut self,
        role: &str,
        array: bool,
        level: usize,
    ) -> Option<(TypeSyntax, usize)> {
        if self.current_kind() == TokenKind::LeftParen {
            return self.parse_tuple_type(role, level);
        }
        let name = self.parse_identifier(role)?;
        let mut end = name.span;
        let mut width_span = None;
        let mut modulus = None;
        let mut modulus_height = 0;

        if name.text == "Mod" && self.current_kind() == TokenKind::LeftBracket {
            let inner = self.open_level(level)?;
            self.bump();
            let (expression, height) = self.parse_expression(inner)?;
            let right_bracket = self.expect(
                TokenKind::RightBracket,
                "`]` after the modulus",
                "a modulus type is written `Mod[MODULUS]`, as in `Mod[(1 << 255) - 19]`",
            )?;
            end = right_bracket.span;
            modulus = Some(Box::new(expression));
            modulus_height = height;
        } else if self.current_kind() == TokenKind::LeftBracket {
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

        let mut length = None;
        let mut length_height = 0;
        if array && self.current_kind() == TokenKind::Caret {
            self.bump();
            let (size, height) = self.parse_size(
                level,
                "a length after `^`",
                "an array type is written `Type^LENGTH`, such as `Word[32]^16`, `Word[8]^n`, or \
                 `Word[8]^(64 * n)`",
            )?;
            end = size.span;
            length = Some(size);
            length_height = height;
            if self.size_continues() {
                self.expected(
                    "the end of the type after its array length",
                    "an array length computed from sizes is written in parentheses, as in \
                     `Word[8]^(2 * n)`",
                );
                return None;
            }
            if self.current_kind() == TokenKind::Caret {
                self.expected(
                    "the end of the type after its array length",
                    "name the row type with a `type` declaration, then write `Row^LENGTH`; \
                     repeated `^` dimensions are not type syntax",
                );
                return None;
            }
        }

        self.record_node().then_some((
            TypeSyntax {
                span: self.join(name.span, end),
                name,
                width_span,
                modulus,
                length,
                elements: Vec::new(),
            },
            modulus_height.max(length_height),
        ))
    }

    /// Parses a tuple type `(T0, T1, ...)` of two through
    /// [`MAX_TUPLE_ELEMENTS`] element types, none of them a tuple type.
    #[inline(never)]
    fn parse_tuple_type(&mut self, role: &str, level: usize) -> Option<(TypeSyntax, usize)> {
        let left_paren = self.bump()?.span;
        let Some((elements, height)) = self.parse_tuple_type_elements(role, level) else {
            self.recover_past_group();
            return None;
        };
        let right_paren = self.bump()?.span;
        if self.current_kind() == TokenKind::Caret {
            self.expected(
                "the end of the type after the tuple",
                "an array's elements are `Int`, `Bool`, words, or residues; arrays of \
                 tuples are not part of Orange 2026",
            );
            return None;
        }
        let span = self.join(left_paren, right_paren);
        self.record_node().then_some((
            TypeSyntax {
                span,
                name: Identifier {
                    text: String::new(),
                    span: left_paren,
                },
                width_span: None,
                modulus: None,
                length: None,
                elements,
            },
            height,
        ))
    }

    /// Parses the element types of a tuple type after its `(`, up to and
    /// not including its `)`, with the greatest height among them.
    fn parse_tuple_type_elements(
        &mut self,
        role: &str,
        level: usize,
    ) -> Option<(Vec<TypeSyntax>, usize)> {
        let mut elements = Vec::new();
        let mut height = 0_usize;
        loop {
            match self.current_kind() {
                TokenKind::LeftParen => {
                    self.expected(
                        "an element type",
                        "a tuple's elements are `Int`, `Bool`, words, residues, and arrays of \
                         them; a tuple holds no tuple",
                    );
                    return None;
                }
                TokenKind::RightParen => {
                    self.expected("an element type", TUPLE_TYPE_NOTE);
                    return None;
                }
                _ => {}
            }
            let (element, element_height) = self.parse_type_syntax(role, true, level)?;
            if elements.len() >= MAX_TUPLE_ELEMENTS {
                self.resource_limit_at(
                    format!("a tuple type has more than {MAX_TUPLE_ELEMENTS} elements"),
                    element.span,
                );
                return None;
            }
            if !(self.reserve_type_slot)(&mut elements) {
                self.resource_limit_at("parser could not allocate type storage", element.span);
                return None;
            }
            height = height.max(element_height);
            elements.push(element);
            match self.current_kind() {
                TokenKind::Comma if elements.len() == 1 => {
                    self.bump()?;
                }
                TokenKind::Comma => {
                    self.bump()?;
                    // A trailing comma before `)` is permitted.
                    if self.current_kind() == TokenKind::RightParen {
                        return Some((elements, height));
                    }
                }
                TokenKind::RightParen if elements.len() >= 2 => return Some((elements, height)),
                TokenKind::RightParen => {
                    self.expected("`,` and another element type", TUPLE_TYPE_NOTE);
                    return None;
                }
                _ => {
                    self.expected("`,` or `)` after the element type", TUPLE_TYPE_NOTE);
                    return None;
                }
            }
        }
    }

    /// After an error inside a parenthesized group whose `(` is consumed,
    /// skips past the group's matching `)`, so that an enclosing list does
    /// not read the group's commas as its own. It stops before a token that
    /// cannot appear in a type, which leaves the rest to the caller's own
    /// recovery.
    #[cold]
    #[inline(never)]
    fn recover_past_group(&mut self) {
        let mut depth = 1_usize;
        while !self.halted {
            match self.current_kind() {
                TokenKind::LeftParen | TokenKind::LeftBracket => {
                    depth = depth.saturating_add(1);
                    if depth > self.limits.recovery_depth {
                        self.resource_limit(format!(
                            "parser recovery exceeds the {}-delimiter nesting limit",
                            self.limits.recovery_depth
                        ));
                        return;
                    }
                }
                TokenKind::RightParen | TokenKind::RightBracket => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        self.bump();
                        return;
                    }
                }
                TokenKind::LeftBrace
                | TokenKind::RightBrace
                | TokenKind::Semicolon
                | TokenKind::Arrow
                | TokenKind::Equal
                | TokenKind::KwSpec
                | TokenKind::KwImpl
                | TokenKind::Eof => return,
                _ => {}
            }
            self.bump();
        }
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
        // `hex "00"` is the identifier `hex` followed by a string, so the
        // note says how a hex string is written.
        let spaced_hex = found == TokenKind::String
            && self
                .cursor
                .checked_sub(1)
                .is_some_and(|previous| self.is_word_at(previous, "hex"));
        self.report_lazy(span, || {
            Diagnostic::error(DiagnosticCode::ExpectedSyntax, build_message(), span)
                .with_label(format!("found {}", found.name()))
                .with_note(if spaced_hex { SPACED_HEX_NOTE } else { note })
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
        self.recover_to_depth(recovery, 0);
    }

    /// Returns the number of `{` tokens from `start` up to the cursor that
    /// are not closed before it.
    #[inline(never)]
    fn open_braces_since(&self, start: usize) -> usize {
        self.tokens
            .get(start..self.cursor)
            .unwrap_or_default()
            .iter()
            .fold(0_usize, |open, token| match token.kind {
                TokenKind::LeftBrace => open.saturating_add(1),
                TokenKind::RightBrace => open.saturating_sub(1),
                _ => open,
            })
    }

    /// Skips tokens until one of `recovery` at delimiter depth zero, starting
    /// `depth` delimiters deep.
    fn recover_to_depth(&mut self, recovery: &[TokenKind], depth: usize) {
        let mut depth = depth;
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

    fn kind_at(&self, position: usize) -> TokenKind {
        self.tokens
            .get(position)
            .map_or(TokenKind::Eof, |token| token.kind)
    }

    /// Returns whether the token at `position` is the identifier spelled
    /// `word`.
    fn is_word_at(&self, position: usize, word: &str) -> bool {
        self.tokens
            .get(position)
            .filter(|token| token.kind == TokenKind::Identifier)
            .and_then(|token| token.lexeme(self.source))
            == Some(word)
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
            ByteOrder::ALL
                .iter()
                .map(|order| order.as_str())
                .collect::<Vec<_>>(),
            ["big", "little"]
        );
        assert_eq!(
            UnaryOperator::ALL
                .iter()
                .map(|operator| operator.as_str())
                .collect::<Vec<_>>(),
            ["-", "~", "!"]
        );
        assert_eq!(
            BinaryOperator::ALL
                .iter()
                .map(|operator| operator.as_str())
                .collect::<Vec<_>>(),
            [
                "+", "-", "*", "&", "|", "^", "<<", ">>", "<<<", ">>>", "/", "%", "==", "!=", "<",
                "<=", ">", ">=", "&&", "||", "++"
            ]
        );
        for operator in BinaryOperator::ALL {
            let spelling = operator.as_str();
            assert_eq!(
                BinaryOperator::from_token(operator.token_kind()),
                Some(*operator)
            );
            assert_eq!(
                operator.is_shift_or_rotation(),
                spelling.starts_with("<<") || spelling.starts_with(">>"),
                "{operator:?}"
            );
            assert_eq!(
                operator.is_comparison(),
                ["==", "!=", "<", "<=", ">", ">="].contains(&spelling),
                "{operator:?}"
            );
            assert_eq!(
                operator.is_division(),
                ["/", "%"].contains(&spelling),
                "{operator:?}"
            );
            assert_eq!(
                operator.is_logical(),
                ["&&", "||"].contains(&spelling),
                "{operator:?}"
            );
            assert_eq!(
                operator.is_concatenation(),
                spelling == "++",
                "{operator:?}"
            );
        }
        for kind in [
            TokenKind::Tilde,
            TokenKind::Bang,
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
                "{}{}({})",
                call.module()
                    .map_or_else(String::new, |module| format!("{}::", module.text)),
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
                "({} as {}{})",
                shape(source, &conversion.operand),
                conversion
                    .order()
                    .map_or_else(String::new, |order| format!("{} ", order.as_str())),
                source.slice(conversion.target.span).unwrap()
            ),
            ExpressionKind::Array(array) => format!(
                "{{{}}}",
                array
                    .elements
                    .iter()
                    .map(|element| shape(source, element))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ExpressionKind::Fill(fill) => format!(
                "{{{}; {}}}",
                shape(source, &fill.element),
                source.slice(fill.length_span()).unwrap()
            ),
            ExpressionKind::Index(index) => format!(
                "{}[{}]",
                shape(source, &index.base),
                shape(source, &index.index)
            ),
            ExpressionKind::Update(update) => format!(
                "({} with [{}]{} = {})",
                shape(source, &update.base),
                shape(source, &update.index),
                update
                    .path
                    .iter()
                    .map(|index| format!("[{}]", shape(source, index)))
                    .collect::<String>(),
                shape(source, &update.value)
            ),
            ExpressionKind::Loop(r#loop) => format!(
                "(for {} in {}..{} with {} = {} {{ {} }})",
                r#loop.index.text,
                source.slice(r#loop.start_span()).unwrap(),
                source.slice(r#loop.end_span()).unwrap(),
                source.slice(r#loop.accumulator.span()).unwrap(),
                shape(source, &r#loop.init),
                shape(source, &r#loop.step)
            ),
            ExpressionKind::Tuple(tuple) => format!(
                "(tuple {})",
                tuple
                    .elements
                    .iter()
                    .map(|element| shape(source, element))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ExpressionKind::Project(project) => {
                format!("{}.{}", shape(source, &project.base), project.position)
            }
            ExpressionKind::Bytes(bytes) => format!(
                "(bytes{} {})",
                if bytes.is_hex() { " hex" } else { "" },
                source.slice(expression.span).unwrap()
            ),
            ExpressionKind::Slice(slice) => format!(
                "{}[{}]",
                shape(source, &slice.base),
                range_shape(source, &slice.range)
            ),
            ExpressionKind::SliceUpdate(update) => format!(
                "({} with [{}] = {})",
                shape(source, &update.base),
                range_shape(source, &update.range),
                shape(source, &update.value)
            ),
            ExpressionKind::Conditional(conditional) => format!(
                "({} else {{ {} }})",
                conditional
                    .arms
                    .iter()
                    .map(|arm| format!(
                        "if {} {{ {} }}",
                        shape(source, &arm.condition),
                        shape(source, &arm.value)
                    ))
                    .collect::<Vec<_>>()
                    .join(" else "),
                shape(source, &conditional.otherwise)
            ),
        }
    }

    /// The height of a type's modulus, or of its elements' moduli, or zero.
    fn type_height(ty: &TypeSyntax) -> usize {
        ty.modulus()
            .map_or(0, tree_height)
            .max(ty.length.as_ref().map_or(0, size_height))
            .max(ty.elements().iter().map(type_height).max().unwrap_or(0))
    }

    /// The height of a size: none for an integer token, else its name's or
    /// group's.
    fn size_height(size: &Size) -> usize {
        size.expression().map_or(0, tree_height)
    }

    /// The greatest height of a pattern's types.
    fn pattern_height(pattern: &Pattern) -> usize {
        pattern
            .names()
            .iter()
            .map(|typed| type_height(&typed.ty))
            .max()
            .unwrap_or(0)
    }

    /// Renders a slice's bounds, with an omitted bound left empty.
    fn range_shape(source: &SourceFile, range: &SliceRange) -> String {
        format!(
            "{}..{}",
            range
                .start
                .as_ref()
                .map_or_else(String::new, |start| shape(source, start)),
            range
                .end
                .as_ref()
                .map_or_else(String::new, |end| shape(source, end))
        )
    }

    /// The height of a slice's bounds, or 0 when neither is written.
    fn range_height(range: &SliceRange) -> usize {
        range
            .start
            .iter()
            .chain(&range.end)
            .map(tree_height)
            .max()
            .unwrap_or(0)
    }

    /// The typed name of a binding or an accumulator that is one name.
    fn named(pattern: &Pattern) -> &TypedName {
        let Pattern::Name(typed) = pattern else {
            panic!("expected one name");
        };
        typed
    }

    fn tree_height(expression: &Expression) -> usize {
        1 + match &expression.kind {
            ExpressionKind::Literal(_) | ExpressionKind::Name(_) | ExpressionKind::Bytes(_) => 0,
            ExpressionKind::Slice(slice) => {
                tree_height(&slice.base).max(range_height(&slice.range))
            }
            ExpressionKind::SliceUpdate(update) => tree_height(&update.base)
                .max(range_height(&update.range))
                .max(tree_height(&update.value)),
            ExpressionKind::Call(call) => call
                .sizes()
                .iter()
                .chain(&call.arguments)
                .map(tree_height)
                .max()
                .unwrap_or(0),
            ExpressionKind::Unary(unary) => tree_height(&unary.operand),
            ExpressionKind::Binary(binary) => {
                tree_height(&binary.left).max(tree_height(&binary.right))
            }
            ExpressionKind::Parenthesized(inner) => tree_height(inner),
            ExpressionKind::Conversion(conversion) => {
                tree_height(&conversion.operand).max(type_height(&conversion.target))
            }
            ExpressionKind::Array(array) => {
                array.elements.iter().map(tree_height).max().unwrap_or(0)
            }
            ExpressionKind::Fill(fill) => tree_height(&fill.element).max(size_height(&fill.length)),
            ExpressionKind::Index(index) => tree_height(&index.base).max(tree_height(&index.index)),
            ExpressionKind::Update(update) => tree_height(&update.base)
                .max(tree_height(&update.index))
                .max(update.path.iter().map(tree_height).max().unwrap_or(0))
                .max(tree_height(&update.value)),
            ExpressionKind::Loop(r#loop) => tree_height(&r#loop.init)
                .max(block_height(&r#loop.step_bindings, &r#loop.step))
                .max(pattern_height(&r#loop.accumulator))
                .max(size_height(&r#loop.start))
                .max(size_height(&r#loop.end)),
            ExpressionKind::Tuple(tuple) => {
                tuple.elements.iter().map(tree_height).max().unwrap_or(0)
            }
            ExpressionKind::Project(project) => tree_height(&project.base),
            ExpressionKind::Conditional(conditional) => conditional
                .arms
                .iter()
                .map(|arm| tree_height(&arm.condition).max(block_height(&arm.bindings, &arm.value)))
                .max()
                .unwrap_or(0)
                .max(block_height(
                    &conditional.otherwise_bindings,
                    &conditional.otherwise,
                )),
        }
    }

    /// The height of a loop's step or a conditional's branch: the greatest
    /// height of its bindings' types and values and of its value.
    fn block_height(bindings: &[Binding], value: &Expression) -> usize {
        bindings
            .iter()
            .map(|binding| tree_height(&binding.value).max(pattern_height(&binding.pattern)))
            .max()
            .unwrap_or(0)
            .max(tree_height(value))
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
             for groups, calls, arrays, indices, loops, conditionals, updates, moduli, and \
             prefix operators"
        );
        type Form = (&'static str, fn(usize) -> String, &'static str);
        let forms: [Form; 16] = [
            (
                "conditional values",
                |count| nested("if a { ", count, "a", " } else { a }"),
                "if",
            ),
            (
                "conditional conditions",
                |count| nested("if ", count, "a", " { a } else { a }"),
                "if",
            ),
            (
                "else values",
                |count| nested("if a { a } else { ", count, "a", " }"),
                "if",
            ),
            ("nots", |count| nested("!", count, "a", ""), "!"),
            ("groups", |count| nested("(", count, "a", ")"), "("),
            (
                "loops",
                |count| nested("for i in 0..1 with s: Int = 0 { ", count, "a", " }"),
                "for",
            ),
            (
                "updates",
                |count| nested("a with [0] = ", count, "a", ""),
                "with",
            ),
            ("indices", |count| nested("a[", count, "a", "]"), "["),
            ("arrays", |count| nested("[", count, "a", "]"), "["),
            (
                "indexed calls",
                |count| nested("g(", count, "a", ")[0]"),
                "g",
            ),
            ("complements", |count| nested("~", count, "a", ""), "~"),
            ("negations", |count| nested("-", count, "a", ""), "-"),
            ("calls", |count| nested("g(", count, "a", ")"), "g"),
            ("sized calls", |count| nested("g[1](", count, "a", ")"), "g"),
            (
                "calls with two sizes",
                |count| nested("g[1, n](a, ", count, "a", ")"),
                "g",
            ),
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

        // An `else if` chain is one conditional, one level deep however long
        // it runs.
        let chain = format!(
            "{}{{ a }}",
            "if a { a } else ".repeat(4 * MAX_EXPRESSION_NESTING)
        );
        let (_, expression) = body_expression(&spec_source(&chain));
        assert_eq!(tree_height(&expression), 2);
        let ExpressionKind::Conditional(conditional) = &expression.kind else {
            panic!("expected a conditional");
        };
        assert_eq!(conditional.arms().len(), 4 * MAX_EXPRESSION_NESTING);
    }

    #[test]
    fn bounds_expression_tree_height_for_operator_chains() {
        let message =
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit");
        let chains: [(&str, &str); 5] = [
            (" ^ a", "^"),
            (" + a", "+"),
            (" * a", "*"),
            (" - 1", "-"),
            ("[0]", "["),
        ];
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
        assert_eq!(named(wide.pattern()).name().text, "wide");
        assert_eq!(
            source.slice(named(wide.pattern()).name().span),
            Some("wide")
        );
        assert_eq!(
            source.slice(named(wide.pattern()).ty().span),
            Some("Word[32]")
        );
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
    fn byte_orders_follow_as_when_a_type_follows_them() {
        let cases = [
            (
                "b as big Word[32]",
                "(b as big Word[32])",
                Some(ByteOrder::Big),
            ),
            (
                "b as little Word[8]^4",
                "(b as little Word[8]^4)",
                Some(ByteOrder::Little),
            ),
            (
                "b as big Word[8]^(4 * n)",
                "(b as big Word[8]^(4 * n))",
                Some(ByteOrder::Big),
            ),
            ("b as big Int", "(b as big Int)", Some(ByteOrder::Big)),
            (
                "b as little Mod[(1 << 130) - 5]",
                "(b as little Mod[(1 << 130) - 5])",
                Some(ByteOrder::Little),
            ),
            (
                "b as big (Int, Int)",
                "(b as big (Int, Int))",
                Some(ByteOrder::Big),
            ),
            (
                "b as little Block",
                "(b as little Block)",
                Some(ByteOrder::Little),
            ),
            // An array type follows only a byte order, so `^` after its
            // element type starts the length.
            (
                "b as big Word[8] ^ n",
                "(b as big Word[8] ^ n)",
                Some(ByteOrder::Big),
            ),
            // Without a type after it, `big` or `little` is the type.
            ("b as big", "(b as big)", None),
            ("b as little", "(b as little)", None),
            ("(b as big) + c", "([(b as big)] + c)", None),
            ("g(b as little, c)", "g((b as little), c)", None),
            ("big as big big", "(big as big big)", Some(ByteOrder::Big)),
            ("little as little", "(little as little)", None),
            (
                "g(b as big Word[8]^4, c)",
                "g((b as big Word[8]^4), c)",
                Some(ByteOrder::Big),
            ),
            (
                "(b as little Word[32]^2) as big Word[64]",
                "([(b as little Word[32]^2)] as big Word[64])",
                Some(ByteOrder::Big),
            ),
        ];
        for (body, expected, order) in cases {
            let (sources, expression) = body_expression(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert_eq!(shape(source, &expression), expected, "{body:?}");
            assert_eq!(source.slice(expression.span), Some(body), "{body:?}");
            let conversion = expression_conversion(&expression);
            assert_eq!(conversion.order(), order, "{body:?}");
            assert_eq!(
                conversion.order_span().and_then(|span| source.slice(span)),
                order.map(ByteOrder::as_str),
                "{body:?}"
            );
        }
        // The order's word is a token of the conversion, which stays one
        // level above its operand.
        let (_, expression) = body_expression(&spec_source("a as big Int"));
        assert_eq!(tree_height(&expression), 2);
    }

    /// Returns the conversion an expression is, looking through groups.
    fn expression_conversion(expression: &Expression) -> &ConversionExpression {
        match &expression.kind {
            ExpressionKind::Conversion(conversion) => conversion,
            ExpressionKind::Parenthesized(inner) => expression_conversion(inner),
            ExpressionKind::Binary(binary) => expression_conversion(&binary.left),
            ExpressionKind::Call(call) => expression_conversion(&call.arguments[0]),
            _ => panic!("expected a conversion"),
        }
    }

    #[test]
    fn an_array_type_after_as_without_a_byte_order_is_ungrouped() {
        let cases = [
            ("a as Word[8]^4", 12, "^", "as", true),
            ("a as big as Int", 9, "as", "as", false),
        ];
        for (body, offset, ungrouped, previous, array) in cases {
            let text = spec_source(body);
            let (sources, _, parsed) = parse_text(&text);
            let source = sources.iter().next().unwrap();
            assert_eq!(parsed.diagnostics.len(), 1, "{body:?}");
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(diagnostic.code(), DiagnosticCode::UngroupedOperators);
            assert_eq!(
                diagnostic.message(),
                format!("`{ungrouped}` follows `{previous}` without grouping parentheses"),
            );
            assert_eq!(source.slice(diagnostic.primary_span()), Some(ungrouped));
            let expected = text.find(body).unwrap() + offset;
            assert_eq!(
                diagnostic.primary_span().start(),
                TextOffset::new(u32::try_from(expected).unwrap()),
                "{body:?}"
            );
            let note = if array {
                "`as` converts exactly one operand; parenthesize the conversion or the \
                 expression it converts. For an array type, name a byte order first, as in \
                 `as big Word[32]^16`"
            } else {
                "`as` converts exactly one operand; parenthesize the conversion or the \
                 expression it converts"
            };
            assert_eq!(diagnostic.notes(), [note], "{body:?}");
        }
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
                .map(|binding| {
                    (
                        named(binding.pattern()).name().text.as_str(),
                        shape(source, binding.value()),
                    )
                })
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
    fn builds_array_types_literals_and_indices_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec rows(x: Word[32]^16, n: Int^2) -> Word[32]^4 { ",
            "let r: Word[8]^2 = [n[1] as Word[8], 0x0f,]; ",
            "[x[0], x[15] ^ g(x)[3], x[1], (x[2])] } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let function = &ast.module.functions[0];

        let x = &function.parameters[0].ty;
        assert_eq!(source.slice(x.span), Some("Word[32]^16"));
        assert_eq!(x.width_span.and_then(|span| source.slice(span)), Some("32"));
        assert_eq!(
            x.length_span().and_then(|span| source.slice(span)),
            Some("16")
        );
        let n = &function.parameters[1].ty;
        assert_eq!(source.slice(n.span), Some("Int^2"));
        assert_eq!(n.width_span, None);
        assert_eq!(
            n.length_span().and_then(|span| source.slice(span)),
            Some("2")
        );

        let FunctionBody::Typed(body) = &function.body else {
            panic!("expected a typed body");
        };
        assert_eq!(source.slice(body.result_type.span), Some("Word[32]^4"));
        let binding = &body.bindings()[0];
        assert_eq!(
            source.slice(named(binding.pattern()).ty().span),
            Some("Word[8]^2")
        );
        assert_eq!(
            source.slice(binding.value().span),
            Some("[n[1] as Word[8], 0x0f,]")
        );
        let ExpressionKind::Array(literal) = &binding.value().kind else {
            panic!("expected an array literal");
        };
        assert_eq!(literal.elements().len(), 2);
        assert_eq!(shape(source, binding.value()), "{(n[1] as Word[8]), 0x0f}");

        let result = body.expression();
        assert_eq!(
            source.slice(result.span),
            Some("[x[0], x[15] ^ g(x)[3], x[1], (x[2])]")
        );
        assert_eq!(
            shape(source, result),
            "{x[0], (x[15] ^ g(x)[3]), x[1], [x[2]]}"
        );
        let ExpressionKind::Array(literal) = &result.kind else {
            panic!("expected an array literal");
        };
        let ExpressionKind::Binary(xor) = &literal.elements()[1].kind else {
            panic!("expected `^`");
        };
        let ExpressionKind::Index(index) = &xor.right.kind else {
            panic!("expected an index");
        };
        assert_eq!(source.slice(xor.right.span), Some("g(x)[3]"));
        assert_eq!(source.slice(index.base().span), Some("g(x)"));
        assert_eq!(source.slice(index.index_span()), Some("3"));
        // An array is one level above its tallest element, and an index one
        // level above its base.
        assert_eq!(tree_height(result), 5);
        assert_eq!(tree_height(&literal.elements()[0]), 2);
    }

    #[test]
    fn array_types_parse_only_where_a_type_is_declared() {
        // A conversion target is never an array, so `^` after it is the
        // operator, which needs grouping.
        let text = spec_source("a as Word[32] ^ b");
        let (_, _, parsed) = parse_text(&text);
        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1, "{:?}", parsed.diagnostics);
        assert_eq!(
            parsed.diagnostics[0].code(),
            DiagnosticCode::UngroupedOperators
        );
        assert_eq!(
            parsed.diagnostics[0].message(),
            "`^` follows `as` without grouping parentheses"
        );
        // A parenthesized conversion is an ordinary `^` operand.
        let (sources, expression) = body_expression(&spec_source("(a as Word[32]) ^ b"));
        let source = sources.iter().next().unwrap();
        assert_eq!(shape(source, &expression), "([(a as Word[32])] ^ b)");
        // Every length spelling parses; the analyzer decides which resolve.
        for length in ["1", "256", "257", "0", "007", "0x10", "1_0"] {
            let text = format!(
                "edition 2026; module m {{ spec f(x: Int^{length}) -> Word[8]^{length} {{ \
                 let t: Word[16]^{length} = x; t }} }}"
            );
            let (sources, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{length}");
            assert!(
                parsed.diagnostics.is_empty(),
                "{length}: {:?}",
                parsed.diagnostics
            );
            let source = sources.iter().next().unwrap();
            let function = &parsed.ast.unwrap().module.functions[0];
            let FunctionBody::Typed(body) = &function.body else {
                panic!("expected a typed body");
            };
            for ty in [
                &function.parameters[0].ty,
                &body.result_type,
                named(body.bindings()[0].pattern()).ty(),
            ] {
                assert_eq!(
                    ty.length_span().and_then(|span| source.slice(span)),
                    Some(length)
                );
            }
        }
    }

    #[test]
    fn rejects_malformed_arrays_with_exact_messages() {
        let cases = [
            ("[]", "expected an array element"),
            ("[,]", "expected an expression"),
            ("[a b]", "expected `,` or `]` after the array element"),
            ("[a,, b]", "expected an expression"),
            ("[a", "expected `,` or `]` after the array element"),
            ("a[]", "expected an index after `[`"),
            ("a[b c]", "expected `]` after the index"),
            ("a[0 + 1", "expected `]` after the index"),
            ("a[0, 1]", "expected `]` after the index"),
            ("a[0", "expected `]` after the index"),
            (
                "a[0].0",
                "expected an operator or the end of the expression",
            ),
            ("(a)[0]", "expected `}` after the body expression"),
            ("[a][0]", "expected `}` after the body expression"),
            ("1[0]", "expected `}` after the body expression"),
        ];
        for (body, message) in cases {
            let text = spec_source(body);
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
        }

        let types = [
            ("Word[8]^", "expected a length after `^`"),
            ("Word[8]^-1", "expected a length after `^`"),
            ("Word[8]^[2]", "expected a length after `^`"),
            (
                "Word[8]^n + 1",
                "expected the end of the type after its array length",
            ),
            (
                "Int^2 * n",
                "expected the end of the type after its array length",
            ),
            (
                "Word[8]^2^2",
                "expected the end of the type after its array length",
            ),
        ];
        for (ty, message) in types {
            for text in [
                format!("edition 2026; module m {{ spec f(x: {ty}) -> Int {{ 1 }} }}"),
                format!("edition 2026; module m {{ spec f() -> {ty} {{ 1 }} }}"),
                format!("edition 2026; module m {{ spec f() -> Int {{ let t: {ty} = 1; 1 }} }}"),
            ] {
                let (_, lexed, parsed) = parse_text(&text);
                assert!(lexed.diagnostics().is_empty(), "{text:?}");
                assert!(parsed.ast.is_none(), "accepted {text:?}");
                let diagnostic = parsed.diagnostics.first().unwrap();
                assert_eq!(diagnostic.code(), DiagnosticCode::ExpectedSyntax);
                assert_eq!(diagnostic.message(), message, "{text:?}");
            }
        }
        let text = "edition 2026; module m { spec f() -> Word[8]^2^2 { [1, 2] } }";
        let (_, _, parsed) = parse_text(text);
        assert_eq!(parsed.diagnostics.len(), 1, "{:?}", parsed.diagnostics);
        assert_eq!(
            parsed.diagnostics[0].notes(),
            [
                "name the row type with a `type` declaration, then write `Row^LENGTH`; \
                 repeated `^` dimensions are not type syntax"
            ]
        );
    }

    #[test]
    fn bounds_elements_per_array_literal() {
        let literal = |count: usize| {
            format!(
                "[{}]",
                (0..count)
                    .map(|index| format!("e{index}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let (_, expression) = body_expression(&spec_source(&literal(MAX_ARRAY_ELEMENTS)));
        let ExpressionKind::Array(array) = &expression.kind else {
            panic!("expected an array literal");
        };
        assert_eq!(array.elements().len(), MAX_ARRAY_ELEMENTS);
        assert_eq!(tree_height(&expression), 2);
        assert_resource_limited(
            &literal(MAX_ARRAY_ELEMENTS + 1),
            &format!("array literal has more than {MAX_ARRAY_ELEMENTS} elements"),
            &format!("e{MAX_ARRAY_ELEMENTS}"),
        );
    }

    #[test]
    fn array_reservation_failure_returns_no_partial_ast() {
        let mut sources = SourceMap::new();
        let id = sources
            .add(
                "test.or",
                "edition 2026; module m { spec f(x: Int) -> Int^2 { [x, x] } }",
            )
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let run = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_argument_slot = |_| false;
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
            "parser could not allocate array storage"
        );
        assert_eq!(source.slice(diagnostic.primary_span()), Some("x"));
    }

    #[test]
    fn builds_loops_updates_fills_and_expression_indices_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec schedule(m: Word[32]^16) -> Word[32]^64 { ",
            "let head: Word[32]^64 = for t in 0..16 with w: Word[32]^64 = [0; 64] ",
            "{ w with [t] = m[t] }; ",
            "for t in 16..0x40 with w: Word[32]^64 = head ",
            "{ w with [t] = w[t - 2] + w[2 * t - 16] } } ",
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

        let head = body.bindings()[0].value();
        assert_eq!(
            source.slice(head.span),
            Some("for t in 0..16 with w: Word[32]^64 = [0; 64] { w with [t] = m[t] }")
        );
        let ExpressionKind::Loop(first) = &head.kind else {
            panic!("expected a loop");
        };
        assert_eq!(source.slice(first.keyword_span()), Some("for"));
        assert_eq!(first.index().text, "t");
        assert_eq!(source.slice(first.index().span), Some("t"));
        assert_eq!(source.slice(first.start_span()), Some("0"));
        assert_eq!(source.slice(first.end_span()), Some("16"));
        assert_eq!(named(first.accumulator()).name().text, "w");
        assert_eq!(
            source.slice(named(first.accumulator()).ty().span),
            Some("Word[32]^64")
        );
        assert_eq!(source.slice(first.init().span), Some("[0; 64]"));
        let ExpressionKind::Fill(fill) = &first.init().kind else {
            panic!("expected a fill literal");
        };
        assert_eq!(source.slice(fill.element().span), Some("0"));
        assert_eq!(source.slice(fill.length_span()), Some("64"));
        assert_eq!(source.slice(first.step().span), Some("w with [t] = m[t]"));
        let ExpressionKind::Update(update) = &first.step().kind else {
            panic!("expected an update");
        };
        assert_eq!(source.slice(update.keyword_span()), Some("with"));
        assert_eq!(source.slice(update.base().span), Some("w"));
        assert_eq!(source.slice(update.index().span), Some("t"));
        assert_eq!(source.slice(update.value().span), Some("m[t]"));
        let ExpressionKind::Index(index) = &update.value().kind else {
            panic!("expected an index");
        };
        assert!(!index.is_literal());
        assert_eq!(source.slice(index.index_span()), Some("t"));
        assert_eq!(
            shape(source, head),
            "(for t in 0..16 with w: Word[32]^64 = {0; 64} { (w with [t] = m[t]) })"
        );
        // A loop is one level above the taller of its initial value and step.
        assert_eq!(tree_height(head), 4);

        let result = body.expression();
        assert_eq!(
            shape(source, result),
            "(for t in 16..0x40 with w: Word[32]^64 = head \
             { (w with [t] = (w[(t - 2)] + w[((2 * t) - 16)])) })"
        );
        let ExpressionKind::Loop(second) = &result.kind else {
            panic!("expected a loop");
        };
        assert_eq!(source.slice(second.end_span()), Some("0x40"));
        // An update's value extends as far as an expression can.
        let ExpressionKind::Update(update) = &second.step().kind else {
            panic!("expected an update");
        };
        assert_eq!(
            source.slice(update.value().span),
            Some("w[t - 2] + w[2 * t - 16]")
        );
        assert_eq!(tree_height(result), 7);

        // A literal index keeps its S3d form.
        let (sources, expression) = body_expression(&spec_source("g(a)[3]"));
        let source = sources.iter().next().unwrap();
        let ExpressionKind::Index(index) = &expression.kind else {
            panic!("expected an index");
        };
        assert!(index.is_literal());
        assert_eq!(source.slice(index.index_span()), Some("3"));
        assert_eq!(tree_height(&expression), 3);
        for body in ["a[-3]", "a[(3)]", "a[3 + 0]"] {
            let (_, expression) = body_expression(&spec_source(body));
            let ExpressionKind::Index(index) = &expression.kind else {
                panic!("expected an index in {body:?}");
            };
            assert!(!index.is_literal(), "{body:?}");
        }
    }

    #[test]
    fn loop_and_update_words_are_recognized_only_by_position() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec f(for: Int, in: Int) -> Int { for + in } ",
            "spec g(with: Word[8]^2) -> Word[8]^2 { with with [0] = with[1] } ",
            "spec h(x: Int) -> Int { for(x) } ",
            "spec for(x: Int) -> Int { x } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let shapes = ast.module.functions[..3]
            .iter()
            .map(|function| {
                let FunctionBody::Typed(body) = &function.body else {
                    panic!("expected a typed body");
                };
                shape(source, body.expression())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            shapes,
            ["(for + in)", "(with with [0] = with[1])", "for(x)"]
        );

        // `with` updates only when `[` follows it.
        let (_, lexed, parsed) = parse_text(&spec_source("a with b"));
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.ast.is_none());
        assert_eq!(
            parsed.diagnostics[0].message(),
            "expected `}` after the body expression"
        );
    }

    #[test]
    fn rejects_malformed_loops_updates_and_fills_with_exact_messages() {
        let cases = [
            (
                "for i 0..2 with s: Int = 0 { s }",
                "expected `in` after the loop index",
            ),
            (
                "for i in [0]..2 with s: Int = 0 { s }",
                "expected the loop's first bound",
            ),
            (
                "for i in n * 2..4 with s: Int = 0 { s }",
                "expected `..` between the loop's bounds",
            ),
            (
                "for i in -1..2 with s: Int = 0 { s }",
                "expected the loop's first bound",
            ),
            (
                "for i in 0 2 with s: Int = 0 { s }",
                "expected `..` between the loop's bounds",
            ),
            (
                "for i in 0..-2 with s: Int = 0 { s }",
                "expected the loop's second bound",
            ),
            (
                "for i in 0.. with s: Int = 0 { s }",
                "expected the loop's second bound",
            ),
            (
                "for i in 0..n + 1 with s: Int = 0 { s }",
                "expected `with` and the loop's accumulator",
            ),
            (
                "for i in 0..2 { s }",
                "expected `with` and the loop's accumulator",
            ),
            (
                "for i in 0..2 with s = 0 { s }",
                "expected `:` and the accumulator's type",
            ),
            (
                "for i in 0..2 with s: Int { s }",
                "expected `=` after the accumulator's type",
            ),
            (
                "for i in 0..2 with s: Int = 0 s",
                "expected `{` before the loop's step",
            ),
            (
                "for i in 0..2 with s: Int = 0 { s, }",
                "expected `}` after the loop's step",
            ),
            (
                "for 1 in 0..2 with s: Int = 0 { s }",
                "expected `}` after the body expression",
            ),
            ("a with [0] 1", "expected `=` after the updated index"),
            ("a with [0 = 1", "expected `]` after the index"),
            ("a with [] = 1", "expected an expression"),
            ("[0; -1]", "expected an array length after `;`"),
            ("[0; n * 2]", "expected `]` after the array length"),
            ("[0; 4, 1]", "expected `]` after the array length"),
            ("[1, 2; 4]", "expected `,` or `]` after the array element"),
        ];
        for (body, message) in cases {
            let text = spec_source(body);
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
        }

        // An update, like a conversion, needs parentheses among operators.
        for (body, message) in [
            (
                "a + b with [0] = 1",
                "`with` follows `+` without grouping parentheses",
            ),
            (
                "a as Word[8] with [0] = 1",
                "`with` follows `as` without grouping parentheses",
            ),
        ] {
            let (_, _, parsed) = parse_text(&spec_source(body));
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            let diagnostic = parsed.diagnostics.first().unwrap();
            assert_eq!(diagnostic.code(), DiagnosticCode::UngroupedOperators);
            assert_eq!(diagnostic.message(), message, "{body:?}");
            assert_eq!(
                diagnostic.notes(),
                [
                    "`with` updates exactly one array; parenthesize the update or the \
                  expression it updates"
                ]
            );
        }
        let (_, parsed) = {
            let (sources, _, parsed) = parse_text(&spec_source("(a + b) with [0] = 1 + 2"));
            (sources, parsed)
        };
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    }

    #[test]
    fn update_paths_parse_one_index_per_dimension_up_to_four() {
        for (body, expected, path) in [
            ("a with [0][1] = b", "(a with [0][1] = b)", 1),
            (
                "a with [b][c + 1][d] = 7",
                "(a with [b][(c + 1)][d] = 7)",
                2,
            ),
            (
                "a with [0][1][2][3] = (b with [0] = c)",
                "(a with [0][1][2][3] = [(b with [0] = c)])",
                3,
            ),
        ] {
            let (sources, expression) = body_expression(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert_eq!(shape(source, &expression), expected, "{body:?}");
            assert_eq!(source.slice(expression.span), Some(body));
            let ExpressionKind::Update(update) = &expression.kind else {
                panic!("expected an update in {body:?}");
            };
            assert_eq!(update.path().len(), path, "{body:?}");
            // Each index keeps its own span, inside its own brackets.
            for index in update.path() {
                let text = source.slice(index.span).unwrap();
                assert!(body.contains(&format!("[{text}]")), "{body:?}: {text:?}");
            }
        }
        for (body, message, note) in [
            (
                "a with [0][1][2][3][4] = b",
                "expected `=` after the updated index",
                PATH_UPDATE_NOTE,
            ),
            (
                "a with [0][1..2] = b",
                "expected `]` after the index",
                PATH_UPDATE_NOTE,
            ),
            (
                "a with [0][1] b",
                "expected `=` after the updated index",
                "an update is written `x with [i] = value`, or `x with [i][j] = value` for an \
                 element of a row",
            ),
        ] {
            let (_, lexed, parsed) = parse_text(&spec_source(body));
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            let diagnostic = parsed.diagnostics.first().unwrap();
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{body:?}"
            );
            assert_eq!(diagnostic.message(), message, "{body:?}");
            assert_eq!(diagnostic.notes(), [note], "{body:?}");
        }
        // A slice update still takes one range, and no path follows it.
        let (_, _, parsed) = parse_text(&spec_source("a with [0..1][0] = b"));
        assert!(parsed.ast.is_none());
        assert_eq!(
            parsed.diagnostics.first().unwrap().message(),
            "expected `=` after the updated slice"
        );
    }

    #[test]
    fn builds_conditionals_comparisons_and_divisions_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec sign(x: Int) -> Int { if x < 0 { -1 } else if x == 0 { 0 } else { 1 } } ",
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
        let expression = body.expression();
        assert_eq!(
            source.slice(expression.span),
            Some("if x < 0 { -1 } else if x == 0 { 0 } else { 1 }")
        );
        let ExpressionKind::Conditional(conditional) = &expression.kind else {
            panic!("expected a conditional");
        };
        // An `else if` chain is one conditional with an arm per `if`.
        assert_eq!(conditional.arms().len(), 2);
        let keywords = conditional
            .arms()
            .iter()
            .map(|arm| arm.keyword_span().start())
            .collect::<Vec<_>>();
        assert_eq!(
            keywords,
            [
                TextOffset::new(u32::try_from(text.find("if x <").unwrap()).unwrap()),
                TextOffset::new(u32::try_from(text.find("if x ==").unwrap()).unwrap()),
            ]
        );
        for arm in conditional.arms() {
            assert_eq!(source.slice(arm.keyword_span()), Some("if"));
        }
        assert_eq!(
            source.slice(conditional.arms()[0].condition().span),
            Some("x < 0")
        );
        assert_eq!(source.slice(conditional.arms()[0].value().span), Some("-1"));
        assert_eq!(
            source.slice(conditional.arms()[1].condition().span),
            Some("x == 0")
        );
        assert_eq!(source.slice(conditional.else_span()), Some("else"));
        assert_eq!(
            conditional.else_span().start(),
            TextOffset::new(u32::try_from(text.rfind("else").unwrap()).unwrap())
        );
        assert_eq!(source.slice(conditional.otherwise().span), Some("1"));
        assert_eq!(
            shape(source, expression),
            "(if (x < 0) { -1 } else if (x == 0) { 0 } else { 1 })"
        );
        // A conditional is one level above its tallest part.
        assert_eq!(tree_height(expression), 3);

        for (body, expected) in [
            ("a / b", "(a / b)"),
            ("a % b", "(a % b)"),
            ("a != b", "(a != b)"),
            ("a <= b", "(a <= b)"),
            ("a >= b", "(a >= b)"),
            ("a > b", "(a > b)"),
            ("a && b && c", "((a && b) && c)"),
            ("a || b || c", "((a || b) || c)"),
            ("!a && !!b", "((!a) && (!(!b)))"),
            ("(a + b) < (c * d)", "([(a + b)] < [(c * d)])"),
            ("-a / ~b", "((-a) / (~b))"),
            ("(a < b) == (c < d)", "([(a < b)] == [(c < d)])"),
            ("if (a) { b } else { c }", "(if [a] { b } else { c })"),
            ("if -a { b } else { c }", "(if (-a) { b } else { c })"),
            ("if !a { b } else { c }", "(if (!a) { b } else { c })"),
            ("if ~a { b } else { c }", "(if (~a) { b } else { c })"),
            ("if 1 { b } else { c }", "(if 1 { b } else { c })"),
            ("if [a] { b } else { c }", "(if {a} { b } else { c })"),
            ("if a[0] { b } else { c }", "(if a[0] { b } else { c })"),
            (
                "if a { if b { c } else { d } } else { a }",
                "(if a { (if b { c } else { d }) } else { a })",
            ),
            (
                "if if a { b } else { c } { d } else { a }",
                "(if (if a { b } else { c }) { d } else { a })",
            ),
            ("if a { b } else { c } + d", "((if a { b } else { c }) + d)"),
            (
                "for i in 0..2 with s: Int = 0 { if a { s } else { i } }",
                "(for i in 0..2 with s: Int = 0 { (if a { s } else { i }) })",
            ),
        ] {
            let (sources, expression) = body_expression(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert_eq!(shape(source, &expression), expected, "{body:?}");
        }
    }

    #[test]
    fn conditional_words_are_recognized_only_by_position() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec f(if: Int, else: Int) -> Int { if + else } ",
            "spec g(if: Word[8]^2) -> Word[8]^2 { if with [0] = if[1] } ",
            "spec h(x: Int) -> Int { if(x) - if(x) } ",
            "spec if(x: Int) -> Int { x } ",
            "spec k(if: Int) -> Int { (if as Int) * if } ",
            "spec t(true: Int, false: Int) -> Int { true - false } ",
            "spec n(if: Int^2) -> Int { if[0] - if[1] } ",
            "spec u(as: Bool) -> Int { if as { 1 } else { 0 } } ",
            "spec w(with: Bool^2) -> Int { if with[0] { 1 } else { 0 } } ",
            "spec v(if: Word[8]) -> Word[8] { (if as Word[8]) + 1 } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let shapes = ast
            .module
            .functions
            .iter()
            .filter(|function| function.name.text != "if")
            .map(|function| {
                let FunctionBody::Typed(body) = &function.body else {
                    panic!("expected a typed body");
                };
                shape(source, body.expression())
            })
            .collect::<Vec<_>>();
        assert_eq!(
            shapes,
            [
                "(if + else)",
                "(if with [0] = if[1])",
                "(if(x) - if(x))",
                "([(if as Int)] * if)",
                "(true - false)",
                "(if[0] - if[1])",
                "(if as { 1 } else { 0 })",
                "(if with[0] { 1 } else { 0 })",
                "([(if as Word[8])] + 1)",
            ]
        );
    }

    #[test]
    fn rejects_malformed_conditionals_with_exact_messages() {
        let cases = [
            ("if a b } else { c }", "expected `{` after the condition"),
            (
                "if a { b }",
                "expected `else` and the value when the condition is false",
            ),
            (
                "if a { b } c",
                "expected `else` and the value when the condition is false",
            ),
            ("if a { b } else c", "expected `{` or `if` after `else`"),
            ("if a { b } else if", "expected an expression"),
            ("if a { b, } else { c }", "expected `}` after the value"),
            ("if a { b } else { c, }", "expected `}` after the value"),
            ("if a { } else { c }", "expected an expression"),
            ("if a { b } else { }", "expected an expression"),
            // Before `(`, `if` starts a conditional only when `else` follows
            // a brace group; otherwise it is a call of a function `if`.
            ("if (a) { b }", "expected `}` after the body expression"),
        ];
        for (body, message) in cases {
            let text = spec_source(body);
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
        }
        let (_, _, parsed) = parse_text(&spec_source("if a { b }"));
        assert_eq!(
            parsed.diagnostics[0].notes(),
            ["every `if` has an `else`, so that a conditional always has a value"]
        );

        // Comparisons and divisions take exactly two operands, and operator
        // groups still need parentheses.
        for (body, ungrouped, previous, note) in [
            (
                "a < b < c",
                "<",
                "<",
                "a comparison takes exactly two operands; join two comparisons with `&&` or `||`",
            ),
            (
                "a == b != c",
                "!=",
                "==",
                "a comparison takes exactly two operands; join two comparisons with `&&` or `||`",
            ),
            (
                "a / b / c",
                "/",
                "/",
                "`/` and `%` take exactly two operands; parenthesize one of them",
            ),
            (
                "a / b % c",
                "%",
                "/",
                "`/` and `%` take exactly two operands; parenthesize one of them",
            ),
            (
                "a && b || c",
                "||",
                "&&",
                "operators from different groups have no relative precedence in Orange; \
                 parenthesize the part that applies first",
            ),
            (
                "a == b && c",
                "&&",
                "==",
                "operators from different groups have no relative precedence in Orange; \
                 parenthesize the part that applies first",
            ),
            (
                "a + b < c",
                "<",
                "+",
                "operators from different groups have no relative precedence in Orange; \
                 parenthesize the part that applies first",
            ),
            (
                "(a + b) < c * d",
                "*",
                "<",
                "operators from different groups have no relative precedence in Orange; \
                 parenthesize the part that applies first",
            ),
            (
                "a * b / c",
                "/",
                "*",
                "operators from different groups have no relative precedence in Orange; \
                 parenthesize the part that applies first",
            ),
            (
                "a & b == c",
                "==",
                "&",
                "operators from different groups have no relative precedence in Orange; \
                 parenthesize the part that applies first",
            ),
            (
                "a < b as Int",
                "as",
                "<",
                "`as` converts exactly one operand; parenthesize the conversion or the \
                 expression it converts",
            ),
        ] {
            let (sources, _, parsed) = parse_text(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            assert_eq!(parsed.diagnostics.len(), 1, "{body:?}");
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(diagnostic.code(), DiagnosticCode::UngroupedOperators);
            assert_eq!(
                diagnostic.message(),
                format!("`{ungrouped}` follows `{previous}` without grouping parentheses"),
                "{body:?}"
            );
            assert_eq!(source.slice(diagnostic.primary_span()), Some(ungrouped));
            assert_eq!(diagnostic.notes(), [note], "{body:?}");
        }
    }

    #[test]
    fn errors_inside_branches_and_steps_recover_to_the_next_function() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec f(c: Bool) -> Int { if c { 1, 2 } else { 3 } } ",
            "spec g() -> Int { for i in 0..2 with s: Int = 0 { if true { s } else { s i } } } ",
            "spec h(c: Bool) -> Int { let a: Int = if c { 1 } else { (2 }; a } ",
            "spec k(c: Bool) -> Int { if c { 1 } else { 2 } } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        let source = sources.iter().next().unwrap();
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.ast.is_none());
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .map(|diagnostic| (
                    diagnostic.code(),
                    diagnostic.message(),
                    source.slice(diagnostic.primary_span()).unwrap()
                ))
                .collect::<Vec<_>>(),
            [
                (
                    DiagnosticCode::ExpectedSyntax,
                    "expected `}` after the value",
                    ","
                ),
                (
                    DiagnosticCode::ExpectedSyntax,
                    "expected `}` after the value",
                    "i"
                ),
                (
                    DiagnosticCode::ExpectedSyntax,
                    "expected `)` to close the group",
                    "}"
                ),
            ]
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
            "[[[[[",
            "]]]]]",
            "a[a[a[",
            "[a,[b,[c,",
            "a[0][0][",
            "[][][]",
            "for for for",
            "for i in 0..",
            "for i in 0..1 with s: Int = for",
            "for i in 0..2 with s: Int = 0 { s",
            "with with with [",
            "x with [ ] = 1",
            "x with [0] = x with [",
            "[0; [0; [0;",
            "a[i + [a[i -",
            "if if if",
            "if a { if b {",
            "if (((",
            "if (a) { b } else if (c",
            "if a { b } else if c { d } else",
            "else else else",
            "a < < b",
            "!!!",
            "a && && b",
            "a / / b",
            "if - - - {",
            "if [ { } ] else",
            "if a { b } else { c } else { d }",
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

    #[test]
    fn builds_use_declarations_and_qualified_calls_with_exact_spans() {
        let text = concat!(
            "edition 2026; module hmac { use sha256; use  pad ; ",
            "spec f(x: Word[32]) -> Word[32] { sha256::big_sigma0(x) ^ g(pad::k()[3]) } ",
            "spec g(x: Word[32]) -> Word[32] { x } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        assert_eq!(
            ast.module
                .uses()
                .iter()
                .map(|declaration| (
                    declaration.name().text.as_str(),
                    source.slice(declaration.name().span).unwrap(),
                    source.slice(declaration.span()).unwrap()
                ))
                .collect::<Vec<_>>(),
            [
                ("sha256", "sha256", "use sha256;"),
                ("pad", "pad", "use  pad ;")
            ]
        );
        assert_eq!(ast.module.functions().len(), 2);
        let FunctionBody::Typed(body) = &ast.module.functions()[0].body else {
            panic!("expected a typed body");
        };
        assert_eq!(
            shape(source, &body.expression),
            "(sha256::big_sigma0(x) ^ g(pad::k()[3]))"
        );
        let ExpressionKind::Binary(binary) = &body.expression.kind else {
            panic!("expected a binary root");
        };
        let ExpressionKind::Call(call) = &binary.left.kind else {
            panic!("expected a qualified call");
        };
        assert_eq!(
            call.module().map(|module| module.text.as_str()),
            Some("sha256")
        );
        assert_eq!(source.slice(call.module().unwrap().span), Some("sha256"));
        assert_eq!(call.callee().text, "big_sigma0");
        assert_eq!(
            source.slice(binary.left.span),
            Some("sha256::big_sigma0(x)")
        );
        let ExpressionKind::Call(local) = &binary.right.kind else {
            panic!("expected a local call");
        };
        assert!(local.module().is_none());

        // `use` is a word only at the head of a module; elsewhere it is an
        // ordinary name.
        let (_, lexed, parsed) = parse_text(concat!(
            "edition 2026; module use { ",
            "spec use(use: Int) -> Int { let use: Int = use; use } ",
            "spec g() -> Int { use::use(1) } ",
            "}"
        ));
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert!(parsed.ast.unwrap().module.uses().is_empty());
    }

    #[test]
    fn rejects_malformed_use_declarations_and_qualified_names() {
        let cases = [
            "use;",
            "use 1;",
            "use a",
            "use a b;",
            "use a::b;",
            "use a; use",
            "spec f() -> Int { 1 } use a;",
            "spec f() -> Int { m::g }",
            "spec f() -> Int { m::g + 1 }",
            "spec f() -> Int { m::(1) }",
            "spec f() -> Int { m::g::h() }",
            "spec f() -> Int { ::g() }",
            "spec f() -> Int { m:: }",
            "spec f() -> Int { 1::g() }",
        ];
        for member in cases {
            let text = format!("edition 2026; module m {{ {member} }}");
            let (_, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{member:?}");
            assert!(parsed.ast.is_none(), "accepted {member:?}");
            assert!(!parsed.diagnostics.is_empty(), "{member:?}");
            assert!(
                parsed.diagnostics.iter().all(|diagnostic| matches!(
                    diagnostic.code(),
                    DiagnosticCode::ExpectedSyntax
                        | DiagnosticCode::TrailingSyntax
                        | DiagnosticCode::ExpectedFunctionDeclaration
                )),
                "{member:?}: {:?}",
                parsed.diagnostics
            );
        }

        let (sources, _, parsed) = parse_text(concat!(
            "edition 2026; module m { use a; ",
            "spec f() -> Int { 1 } use b; ",
            "spec g() -> Int { a::h } ",
            "}"
        ));
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .map(|diagnostic| (
                    diagnostic.code(),
                    diagnostic.message(),
                    source.slice(diagnostic.primary_span()).unwrap(),
                    diagnostic.label(),
                    diagnostic.notes().to_vec()
                ))
                .collect::<Vec<_>>(),
            [
                (
                    DiagnosticCode::ExpectedFunctionDeclaration,
                    "expected a `spec` or `impl` function declaration",
                    "use",
                    "a `use` declaration cannot follow a function",
                    vec![
                        "`use` declarations come first in a module, before its functions"
                            .to_owned()
                    ]
                ),
                (
                    DiagnosticCode::ExpectedSyntax,
                    "expected `(` after the qualified function name",
                    "}",
                    "found RIGHT_BRACE",
                    vec![
                        "a name qualified by its module is always called, as in `sha256::initial()`"
                            .to_owned()
                    ]
                ),
            ]
        );
    }

    #[test]
    fn bounds_use_declarations_per_module() {
        let module = |count: usize| {
            format!(
                "edition 2026; module m {{ {} spec f() -> Int {{ 1 }} }}",
                (0..count)
                    .map(|index| format!("use u{index};"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        let (_, _, parsed) = parse_text(&module(MAX_USES_PER_MODULE));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(parsed.ast.unwrap().module.uses().len(), MAX_USES_PER_MODULE);

        let (sources, _, parsed) = parse_text(&module(MAX_USES_PER_MODULE + 1));
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code(),
            DiagnosticCode::ParserResourceLimit
        );
        assert_eq!(
            parsed.diagnostics[0].message(),
            format!("module has more than {MAX_USES_PER_MODULE} `use` declarations")
        );
        assert_eq!(
            source.slice(parsed.diagnostics[0].primary_span()),
            Some(format!("use u{MAX_USES_PER_MODULE};").as_str())
        );
    }

    #[test]
    fn builds_type_declarations_and_modulus_types_with_exact_spans() {
        let text = concat!(
            "edition 2026; module field { use h; type F = Mod[(1 << 255) - 19]; type  V = F^4 ; ",
            "spec g(x: V) -> Mod[7] { (x[0] as Mod[3329]) as Mod[7] } ",
            "spec k() -> Mod[2] { let t: Mod[2]^3 = for i in 0..1 with s: Mod[2]^3 = [0; 3] { s }; t[1] } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let slice = |span: Span| source.slice(span).unwrap();
        let modulus = |ty: &TypeSyntax| ty.modulus().map(|modulus| slice(modulus.span()));
        assert_eq!(ast.module.uses().len(), 1);
        assert_eq!(
            ast.module
                .types()
                .iter()
                .map(|declaration| (
                    declaration.name().text.as_str(),
                    slice(declaration.span()),
                    slice(declaration.ty().span),
                    modulus(declaration.ty()),
                    declaration.ty().length_span().map(slice)
                ))
                .collect::<Vec<_>>(),
            [
                (
                    "F",
                    "type F = Mod[(1 << 255) - 19];",
                    "Mod[(1 << 255) - 19]",
                    Some("(1 << 255) - 19"),
                    None
                ),
                ("V", "type  V = F^4 ;", "F^4", None, Some("4")),
            ]
        );
        let f_modulus = ast.module.types()[0].ty().modulus().unwrap();
        assert_eq!(shape(source, f_modulus), "([(1 << 255)] - 19)");

        let [g, k] = ast.module.functions() else {
            panic!("expected two functions");
        };
        let FunctionBody::Typed(body) = &g.body else {
            panic!("expected a typed body");
        };
        assert_eq!(modulus(&g.parameters()[0].ty), None);
        assert_eq!(modulus(&body.result_type), Some("7"));
        let ExpressionKind::Conversion(outer) = &body.expression.kind else {
            panic!("expected a conversion");
        };
        assert_eq!(modulus(&outer.target), Some("7"));
        let ExpressionKind::Parenthesized(inner) = &outer.operand.kind else {
            panic!("expected a group");
        };
        let ExpressionKind::Conversion(inner) = &inner.kind else {
            panic!("expected a conversion");
        };
        assert_eq!(slice(inner.target.span), "Mod[3329]");
        assert_eq!(modulus(&inner.target), Some("3329"));
        let FunctionBody::Typed(body) = &k.body else {
            panic!("expected a typed body");
        };
        assert_eq!(modulus(&named(&body.bindings[0].pattern).ty), Some("2"));
        let ExpressionKind::Loop(r#loop) = &body.bindings[0].value.kind else {
            panic!("expected a loop");
        };
        assert_eq!(slice(named(&r#loop.accumulator).ty.span), "Mod[2]^3");
        assert_eq!(modulus(&named(&r#loop.accumulator).ty), Some("2"));

        // `type` is a word only at the head of a module, and `Mod` takes a
        // modulus only when a bracket follows it.
        let (_, lexed, parsed) = parse_text(concat!(
            "edition 2026; module type { ",
            "spec type(type: Int, x: Mod) -> Int { let type: Int = type; type } ",
            "spec Mod(Mod: Int) -> Int { Mod } ",
            "}"
        ));
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let ast = parsed.ast.unwrap();
        assert!(ast.module.types().is_empty());
        assert_eq!(ast.module.functions()[0].parameters()[1].ty.modulus(), None);
    }

    #[test]
    fn rejects_malformed_type_declarations_and_moduli() {
        let cases = [
            "type;",
            "type F;",
            "type F = ;",
            "type F = Int",
            "type 1 = Int;",
            "type F = Int^2^2;",
            "type F = Mod[];",
            "type F = Mod[7;",
            "type F = Mod[7]] ;",
            "type F = Mod[7](;",
            "spec f(x: Mod[) -> Int { 0 }",
            "spec f(x: Mod[7 7]) -> Int { 0 }",
            "spec f() -> Int { 0 as Mod[] }",
        ];
        for member in cases {
            let text = format!("edition 2026; module m {{ {member} spec g() -> Int {{ 1 }} }}");
            let (_, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{member:?}");
            assert!(parsed.ast.is_none(), "accepted {member:?}");
            assert!(!parsed.diagnostics.is_empty(), "{member:?}");
            assert!(
                parsed.diagnostics.iter().all(|diagnostic| matches!(
                    diagnostic.code(),
                    DiagnosticCode::ExpectedSyntax
                        | DiagnosticCode::TrailingSyntax
                        | DiagnosticCode::ExpectedFunctionDeclaration
                )),
                "{member:?}: {:?}",
                parsed.diagnostics
            );
        }

        let (sources, _, parsed) = parse_text(concat!(
            "edition 2026; module m { use a; type F = Mod[7]; use b; ",
            "spec f() -> F { 1 } type G = F; ",
            "}"
        ));
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(
            parsed
                .diagnostics
                .iter()
                .map(|diagnostic| (
                    diagnostic.code(),
                    diagnostic.message(),
                    source.slice(diagnostic.primary_span()).unwrap(),
                    diagnostic.label(),
                    diagnostic.notes().to_vec()
                ))
                .collect::<Vec<_>>(),
            [
                (
                    DiagnosticCode::ExpectedFunctionDeclaration,
                    "expected a `type` declaration or a function",
                    "use",
                    "a `use` declaration cannot follow a `type` declaration",
                    vec![
                        "a module's `use` declarations come first, then its `type` declarations, \
                         then its functions"
                            .to_owned()
                    ]
                ),
                (
                    DiagnosticCode::ExpectedFunctionDeclaration,
                    "expected a `spec` or `impl` function declaration",
                    "type",
                    "a `type` declaration cannot follow a function",
                    vec!["`type` declarations come before a module's functions".to_owned()]
                ),
            ]
        );
    }

    #[test]
    fn bounds_type_declarations_per_module() {
        let module = |count: usize| {
            format!(
                "edition 2026; module m {{ {} spec f() -> Int {{ 1 }} }}",
                (0..count)
                    .map(|index| format!("type T{index} = Mod[{}];", index + 2))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        let (_, _, parsed) = parse_text(&module(MAX_TYPES_PER_MODULE));
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        assert_eq!(
            parsed.ast.unwrap().module.types().len(),
            MAX_TYPES_PER_MODULE
        );

        let (sources, _, parsed) = parse_text(&module(MAX_TYPES_PER_MODULE + 1));
        let source = sources.iter().next().unwrap();
        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code(),
            DiagnosticCode::ParserResourceLimit
        );
        assert_eq!(
            parsed.diagnostics[0].message(),
            format!("module has more than {MAX_TYPES_PER_MODULE} `type` declarations")
        );
        assert_eq!(
            source.slice(parsed.diagnostics[0].primary_span()),
            Some(
                format!(
                    "type T{MAX_TYPES_PER_MODULE} = Mod[{}];",
                    MAX_TYPES_PER_MODULE + 2
                )
                .as_str()
            )
        );
    }

    #[test]
    fn a_modulus_counts_toward_nesting_and_the_height_of_its_expression() {
        // A group, a conversion, and an outer conversion stand above the
        // modulus `1 + 1 + ...`, whose height is one more than its additions;
        // an accumulator's type counts toward its loop as a target does.
        let tall = |additions: usize| format!("1{}", " + 1".repeat(additions));
        let additions = MAX_EXPRESSION_HEIGHT - 4;
        let forms: [fn(&str) -> String; 2] = [
            |modulus| format!("(a as Mod[{modulus}]) as Word[32]"),
            |modulus| format!("(for i in 0..1 with s: Mod[{modulus}] = 0 {{ s }}) as Word[32]"),
        ];
        let message =
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit");
        for form in forms {
            let (_, expression) = body_expression(&spec_source(&form(&tall(additions))));
            assert_eq!(tree_height(&expression), MAX_EXPRESSION_HEIGHT);
            assert_resource_limited(&form(&tall(additions + 1)), &message, "as");
        }

        // A modulus opens one nesting level.
        let nested = |groups: usize| {
            format!(
                "{}a as Mod[7]{} as Word[32]",
                "(".repeat(groups),
                ")".repeat(groups)
            )
        };
        body_expression(&spec_source(&nested(MAX_EXPRESSION_NESTING - 1)));
        let message = format!(
            "expression nesting exceeds the {MAX_EXPRESSION_NESTING}-level limit \
             for groups, calls, arrays, indices, loops, conditionals, updates, moduli, and \
             prefix operators"
        );
        assert_resource_limited(&nested(MAX_EXPRESSION_NESTING), &message, "[");
    }

    #[test]
    fn builds_blocks_in_steps_and_branches_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec mix(a: Word[32], b: Word[32]) -> Word[32] { ",
            "for i in 0..4 with s: Word[32] = a { ",
            "let t: Word[32] = s ^ b; let u: Word[32] = t >>> 7; u + s } } ",
            "spec pick(c: Bool, x: Int) -> Int { ",
            "if c { let y: Int = x + 1; y * y } else if x < 0 { 0 } ",
            "else { let z: Int = x; z } } ",
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
        assert!(body.bindings().is_empty());
        let ExpressionKind::Loop(r#loop) = &body.expression().kind else {
            panic!("expected a loop");
        };
        assert_eq!(
            source.slice(body.expression().span),
            Some(concat!(
                "for i in 0..4 with s: Word[32] = a { ",
                "let t: Word[32] = s ^ b; let u: Word[32] = t >>> 7; u + s }"
            ))
        );
        let spans = r#loop
            .step_bindings()
            .iter()
            .map(|binding| {
                (
                    source.slice(binding.span()).unwrap(),
                    named(binding.pattern()).name().text.as_str(),
                    source.slice(named(binding.pattern()).ty().span).unwrap(),
                    shape(source, binding.value()),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            spans,
            [
                (
                    "let t: Word[32] = s ^ b;",
                    "t",
                    "Word[32]",
                    "(s ^ b)".to_owned()
                ),
                (
                    "let u: Word[32] = t >>> 7;",
                    "u",
                    "Word[32]",
                    "(t >>> 7)".to_owned()
                ),
            ]
        );
        assert_eq!(shape(source, r#loop.step()), "(u + s)");
        assert_eq!(source.slice(r#loop.step().span), Some("u + s"));

        let FunctionBody::Typed(body) = &ast.module.functions[1].body else {
            panic!("expected a typed body");
        };
        let ExpressionKind::Conditional(conditional) = &body.expression().kind else {
            panic!("expected a conditional");
        };
        let arms = conditional.arms();
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0].bindings().len(), 1);
        assert_eq!(
            source.slice(arms[0].bindings()[0].span()),
            Some("let y: Int = x + 1;")
        );
        assert_eq!(shape(source, arms[0].value()), "(y * y)");
        assert!(arms[1].bindings().is_empty());
        assert_eq!(shape(source, arms[1].value()), "0");
        assert_eq!(conditional.otherwise_bindings().len(), 1);
        assert_eq!(
            source.slice(conditional.otherwise_bindings()[0].span()),
            Some("let z: Int = x;")
        );
        assert_eq!(shape(source, conditional.otherwise()), "z");
    }

    #[test]
    fn rejects_malformed_blocks_with_exact_messages() {
        let step = "a loop's step holds `let` bindings, if any, and then the expression \
                    that gives the accumulator's next value";
        let branch =
            "each branch of a conditional holds `let` bindings, if any, and then its value";
        let binding = "each binding ends with `;`; the block's last item is its value";
        let cases = [
            (
                "for i in 0..2 with s: Word[32] = a { let t: Word[32] = s; }",
                "expected a value after the last binding",
                step,
            ),
            (
                "for i in 0..2 with s: Word[32] = a { let t: Word[32] = s t }",
                "expected `;` after the bound expression",
                binding,
            ),
            (
                "for i in 0..2 with s: Word[32] = a { let t = s; t }",
                "expected `:` and the binding's type",
                "every binding states its type, as in `let t: Word[32] = x + y;`",
            ),
            (
                "for i in 0..2 with s: Word[32] = a { s; let t: Word[32] = s; t }",
                "expected `}` after the loop's step",
                step,
            ),
            (
                "if c == d { let t: Word[32] = a; } else { b }",
                "expected a value after the last binding",
                branch,
            ),
            (
                "if c == d { a } else { let t: Word[32] = b; }",
                "expected a value after the last binding",
                branch,
            ),
            (
                "if c == d { a } else if c < d { let t: Word[32] = b; } else { c }",
                "expected a value after the last binding",
                branch,
            ),
            (
                "if c == d { a } else { let t: Word[32] = b t }",
                "expected `;` after the bound expression",
                binding,
            ),
            (
                "if c == d { let t: Word[32] = a; t; t } else { b }",
                "expected `}` after the value",
                branch,
            ),
            (
                "if c == d { a } else { b; let t: Word[32] = b; t }",
                "expected `}` after the value",
                branch,
            ),
        ];
        for (body, message, note) in cases {
            let (_, lexed, parsed) = parse_text(&spec_source(body));
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            assert_eq!(
                parsed.diagnostics.len(),
                1,
                "{body:?}: {:?}",
                parsed.diagnostics
            );
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{body:?}"
            );
            assert_eq!(diagnostic.message(), message, "{body:?}");
            assert_eq!(diagnostic.notes(), [note], "{body:?}");
        }
    }

    #[test]
    fn bounds_bindings_per_block() {
        let bindings = |count: usize| {
            (0..count)
                .map(|index| format!("let v{index}: Word[32] = a; "))
                .collect::<String>()
        };
        let forms: [fn(&str) -> String; 3] = [
            |bindings| format!("for i in 0..2 with s: Word[32] = a {{ {bindings}s }}"),
            |bindings| format!("if a == b {{ {bindings}c }} else {{ d }}"),
            |bindings| format!("if a == b {{ c }} else {{ {bindings}d }}"),
        ];
        let message = format!("a block declares more than {MAX_BINDINGS_PER_BODY} bindings");
        let last = format!("let v{MAX_BINDINGS_PER_BODY}: Word[32] = a;");
        for form in forms {
            body_expression(&spec_source(&form(&bindings(MAX_BINDINGS_PER_BODY))));
            assert_resource_limited(&form(&bindings(MAX_BINDINGS_PER_BODY + 1)), &message, &last);
        }
    }

    #[test]
    fn block_bindings_count_toward_the_height_of_their_expression() {
        // The loop or conditional stands above its block; the binding's
        // value `a + a + ...` is one level taller than its additions.
        let tall = |additions: usize| format!("a{}", " + a".repeat(additions));
        let additions = MAX_EXPRESSION_HEIGHT - 2;
        type Form = (fn(&str) -> String, &'static str);
        let forms: [Form; 3] = [
            (
                |value| {
                    format!("for i in 0..1 with s: Word[32] = a {{ let t: Word[32] = {value}; t }}")
                },
                "for",
            ),
            (
                |value| format!("if a == b {{ let t: Word[32] = {value}; t }} else {{ c }}"),
                "if",
            ),
            (
                |value| format!("if a == b {{ c }} else {{ let t: Word[32] = {value}; t }}"),
                "if",
            ),
        ];
        let message =
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit");
        for (form, at) in forms {
            let (_, expression) = body_expression(&spec_source(&form(&tall(additions))));
            assert_eq!(tree_height(&expression), MAX_EXPRESSION_HEIGHT);
            assert_resource_limited(&form(&tall(additions + 1)), &message, at);
        }
    }

    #[test]
    fn builds_tuples_projections_and_patterns_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "type Pair = (Word[64], Word[64]^4,); ",
            "spec f(p: (Int, Bool), q: Pair) -> (Int, Int) { ",
            "let (s: Int, c: Bool) = p; ",
            "for i in 0..2 with (a: Int, b: Int) = (s, p.0,) { ",
            "let (x: Int, y: Int) = g(a, q); (b, h(x, y).1[i]) } } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let slices = |types: &[TypeSyntax]| {
            types
                .iter()
                .map(|ty| source.slice(ty.span).unwrap())
                .collect::<Vec<_>>()
        };

        // A tuple type: its extent, a trailing comma, and its element types.
        let pair = ast.module.types()[0].ty();
        assert!(pair.is_tuple());
        assert_eq!(source.slice(pair.span), Some("(Word[64], Word[64]^4,)"));
        assert_eq!(slices(pair.elements()), ["Word[64]", "Word[64]^4"]);
        assert!(pair.name.text.is_empty());
        assert_eq!(source.slice(pair.name.span), Some("("));
        let function = &ast.module.functions[0];
        assert_eq!(
            slices(function.parameters[0].ty.elements()),
            ["Int", "Bool"]
        );
        assert!(!function.parameters[1].ty.is_tuple());
        let FunctionBody::Typed(body) = &function.body else {
            panic!("expected a typed body");
        };
        assert_eq!(source.slice(body.result_type.span), Some("(Int, Int)"));
        assert_eq!(slices(body.result_type.elements()), ["Int", "Int"]);

        // A binding's tuple pattern names each element with its type.
        let names = |pattern: &Pattern| {
            pattern
                .names()
                .iter()
                .map(|typed| {
                    (
                        typed.name.text.clone(),
                        source.slice(typed.ty.span).unwrap(),
                        source.slice(typed.span).unwrap(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let binding = &body.bindings()[0];
        assert_eq!(
            source.slice(binding.span()),
            Some("let (s: Int, c: Bool) = p;")
        );
        assert_eq!(
            source.slice(binding.pattern().span()),
            Some("(s: Int, c: Bool)")
        );
        assert_eq!(
            names(binding.pattern()),
            [
                ("s".to_owned(), "Int", "s: Int"),
                ("c".to_owned(), "Bool", "c: Bool")
            ]
        );

        // A loop's accumulators, a tuple with a trailing comma, a tuple
        // pattern in a step, and a projection of a call followed by an index.
        let ExpressionKind::Loop(r#loop) = &body.expression().kind else {
            panic!("expected a loop");
        };
        assert_eq!(
            source.slice(r#loop.accumulator().span()),
            Some("(a: Int, b: Int)")
        );
        assert_eq!(
            names(r#loop.accumulator()),
            [
                ("a".to_owned(), "Int", "a: Int"),
                ("b".to_owned(), "Int", "b: Int")
            ]
        );
        assert_eq!(source.slice(r#loop.init().span), Some("(s, p.0,)"));
        assert_eq!(shape(source, r#loop.init()), "(tuple s, p.0)");
        let step = &r#loop.step_bindings()[0];
        assert_eq!(
            source.slice(step.pattern().span()),
            Some("(x: Int, y: Int)")
        );
        assert_eq!(shape(source, step.value()), "g(a, q)");
        assert_eq!(shape(source, r#loop.step()), "(tuple b, h(x, y).1[i])");
        let ExpressionKind::Tuple(tuple) = &r#loop.step().kind else {
            panic!("expected a tuple");
        };
        let ExpressionKind::Index(index) = &tuple.elements()[1].kind else {
            panic!("expected an index");
        };
        let ExpressionKind::Project(project) = &index.base.kind else {
            panic!("expected a projection");
        };
        assert_eq!(source.slice(index.base.span), Some("h(x, y).1"));
        assert_eq!(source.slice(project.position_span), Some("1"));
        assert_eq!(project.position, 1);

        // A group of one expression stays a group, and a position too large
        // for `u32` is kept as `u32::MAX`.
        let (sources, expression) = body_expression(&spec_source("(a)"));
        assert_eq!(shape(sources.iter().next().unwrap(), &expression), "[a]");
        let (_, expression) = body_expression(&spec_source("a.4294967296"));
        let ExpressionKind::Project(project) = &expression.kind else {
            panic!("expected a projection");
        };
        assert_eq!(project.position, u32::MAX);
    }

    #[test]
    fn rejects_malformed_tuples_with_exact_messages() {
        let tuple_type = TUPLE_TYPE_NOTE;
        let tuple = TUPLE_NOTE;
        let pattern = PATTERN_NOTE;
        let projection = PROJECTION_NOTE;
        let nested_type = "a tuple's elements are `Int`, `Bool`, words, residues, and arrays of \
                           them; a tuple holds no tuple";
        let cases = [
            (
                "let t: (Int) = a; t",
                "expected `,` and another element type",
                tuple_type,
            ),
            ("let t: () = a; t", "expected an element type", tuple_type),
            (
                "let t: (Int,) = a; t",
                "expected an element type",
                tuple_type,
            ),
            (
                "let t: (Int Int) = a; t",
                "expected `,` or `)` after the element type",
                tuple_type,
            ),
            (
                "let t: ((Int, Int), Int) = a; t",
                "expected an element type",
                nested_type,
            ),
            (
                "let t: (Int, Int)^2 = a; t",
                "expected the end of the type after the tuple",
                "an array's elements are `Int`, `Bool`, words, or residues; arrays of tuples \
                 are not part of Orange 2026",
            ),
            ("(a,)", "expected another element after `,`", tuple),
            (
                "(a, b c)",
                "expected `,` or `)` after the tuple's element",
                tuple,
            ),
            (
                "a.x",
                "expected an element's position after `.`",
                projection,
            ),
            (
                "a.01",
                "expected an element's position in decimal",
                projection,
            ),
            (
                "a.0x1",
                "expected an element's position in decimal",
                projection,
            ),
            (
                "a.1_0",
                "expected an element's position in decimal",
                projection,
            ),
            (
                "a.0.1",
                "expected an operator or the end of the expression",
                "a tuple's elements are not tuples, so an element is selected once",
            ),
            (
                "a[0].1",
                "expected an operator or the end of the expression",
                "an array's elements are not tuples, so an element has no `.k`",
            ),
            (
                "let (x: Int) = a; x",
                "expected `,` and another name in the pattern",
                pattern,
            ),
            (
                "let (x, y) = a; x",
                "expected `:` and the type of the name",
                pattern,
            ),
            (
                "let (x: Int, 5) = a; x",
                "expected a name for the pattern's next value",
                pattern,
            ),
            (
                "let (): Int = a; a",
                "expected `}` after the body expression",
                "a typed `spec` body holds `let` bindings, if any, and then one result \
                 expression",
            ),
            (
                "let (x: Int y: Int) = a; x",
                "expected `,` or `)` after the pattern's element",
                pattern,
            ),
            (
                "for i in 0..2 with (s: Int, t) = a { s }",
                "expected `:` and the type of the name",
                pattern,
            ),
        ];
        for (body, message, note) in cases {
            let (_, lexed, parsed) = parse_text(&spec_source(body));
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            assert_eq!(
                parsed.diagnostics.len(),
                1,
                "{body:?}: {:?}",
                parsed.diagnostics
            );
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{body:?}"
            );
            assert_eq!(diagnostic.message(), message, "{body:?}");
            assert_eq!(diagnostic.notes(), [note], "{body:?}");
        }

        // An error inside a parameter's tuple type is reported once: the
        // parameter list resumes after the tuple, not at its inner commas.
        let (_, _, parsed) = parse_text(
            "edition 2026; module m { spec f(p: ((Int, Int), Int), q: Int) -> Int { q } \
             spec g() -> Int { 0 } }",
        );
        assert_eq!(parsed.diagnostics.len(), 1, "{:?}", parsed.diagnostics);

        // `let` followed by a call's arguments is still a call of `let`.
        let (sources, expression) = body_expression(&spec_source("let(a, b)"));
        assert_eq!(
            shape(sources.iter().next().unwrap(), &expression),
            "let(a, b)"
        );
    }

    #[test]
    fn bounds_tuple_elements() {
        let list = |count: usize, item: &dyn Fn(usize) -> String| {
            (0..count).map(item).collect::<Vec<_>>().join(", ")
        };
        let most = MAX_TUPLE_ELEMENTS;
        let over = MAX_TUPLE_ELEMENTS + 1;
        let types = |count| format!("let t: ({}) = a; t", list(count, &|_| "Int".to_owned()));
        let tuples = |count| format!("({})", list(count, &|_| "a".to_owned()));
        let patterns = |count| {
            format!(
                "let ({}) = a; v0",
                list(count, &|index| format!("v{index}: Int"))
            )
        };
        body_expression(&spec_source(&types(most)));
        assert_resource_limited(
            &types(over),
            &format!("a tuple type has more than {most} elements"),
            "Int",
        );
        let (_, expression) = body_expression(&spec_source(&tuples(most)));
        let ExpressionKind::Tuple(tuple) = &expression.kind else {
            panic!("expected a tuple");
        };
        assert_eq!(tuple.elements().len(), most);
        assert_resource_limited(
            &tuples(over),
            &format!("a tuple has more than {most} elements"),
            "a",
        );
        body_expression(&spec_source(&patterns(most)));
        assert_resource_limited(
            &patterns(over),
            &format!("a tuple pattern names more than {most} values"),
            &format!("v{most}: Int"),
        );
    }

    #[test]
    fn tuples_and_projections_count_toward_expression_height() {
        // A tuple stands one level above its tallest element, and a
        // projection one level above its base, here a call.
        let tall = |additions: usize| format!("a{}", " + a".repeat(additions));
        type Form = (fn(&str) -> String, &'static str, usize);
        let forms: [Form; 2] = [
            (
                |value| format!("(a, {value})"),
                "(",
                MAX_EXPRESSION_HEIGHT - 2,
            ),
            (
                |value| format!("g({value}).0"),
                ".",
                MAX_EXPRESSION_HEIGHT - 3,
            ),
        ];
        let message =
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit");
        for (form, at, additions) in forms {
            let (_, expression) = body_expression(&spec_source(&form(&tall(additions))));
            assert_eq!(tree_height(&expression), MAX_EXPRESSION_HEIGHT);
            assert_resource_limited(&form(&tall(additions + 1)), &message, at);
        }
    }

    #[test]
    fn tuple_reservation_failures_return_no_partial_ast() {
        let mut sources = SourceMap::new();
        let id = sources
            .add(
                "test.or",
                "edition 2026; module m { spec f(x: (Int, Int)) -> Int { \
                 let (a: Int, b: Int) = x; a } }",
            )
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let type_failure = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_type_slot = |_| false;
            parser.run()
        };
        let name_failure = || {
            let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
            parser.reserve_name_slot = |_| false;
            parser.run()
        };
        for (run, message, span) in [
            (
                &type_failure as &dyn Fn() -> ParseResult,
                "parser could not allocate type storage",
                "Int",
            ),
            (
                &name_failure,
                "parser could not allocate pattern storage",
                "a: Int",
            ),
        ] {
            let first = run();
            assert_eq!(first, run());
            assert!(first.ast.is_none());
            assert_eq!(first.diagnostics.len(), 1, "{:?}", first.diagnostics);
            let diagnostic = &first.diagnostics[0];
            assert_eq!(diagnostic.code(), DiagnosticCode::ParserResourceLimit);
            assert_eq!(diagnostic.message(), message);
            assert_eq!(source.slice(diagnostic.primary_span()), Some(span));
        }

        // A tuple's elements share the argument slots' reservation hook.
        let id = sources
            .add(
                "tuple.or",
                "edition 2026; module m { spec f(x: Int) -> (Int, Int) { (x, x) } }",
            )
            .unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let mut parser = Parser::new(source, lexed.tokens(), Limits::DEFAULT);
        parser.reserve_argument_slot = |_| false;
        let result = parser.run();
        assert!(result.ast.is_none());
        assert_eq!(result.diagnostics.len(), 1);
        assert_eq!(
            result.diagnostics[0].message(),
            "parser could not allocate tuple storage"
        );
        assert_eq!(
            source.slice(result.diagnostics[0].primary_span()),
            Some("x")
        );
    }

    #[test]
    fn builds_byte_strings_joins_and_slices_with_exact_spans() {
        let cases = [
            ("\"ab\"", "(bytes \"ab\")"),
            ("hex\"00 ff\"", "(bytes hex hex\"00 ff\")"),
            ("a ++ b ++ c", "((a ++ b) ++ c)"),
            ("a ++ (b ++ c)", "(a ++ [(b ++ c)])"),
            ("a[1..3]", "a[1..3]"),
            ("a[..3]", "a[..3]"),
            ("a[1..]", "a[1..]"),
            ("g(a)[4 * i..4 * i + 4]", "g(a)[(4 * i)..((4 * i) + 4)]"),
            ("p.0[1..2]", "p.0[1..2]"),
            ("a[0..2] ++ \"xy\"", "(a[0..2] ++ (bytes \"xy\"))"),
            ("a with [1..3] = \"xy\"", "(a with [1..3] = (bytes \"xy\"))"),
            ("a with [..2] = b[2..]", "(a with [..2] = b[2..])"),
            ("a with [2..] = b ++ c", "(a with [2..] = (b ++ c))"),
            ("a with [0] = b", "(a with [0] = b)"),
        ];
        for (body, expected) in cases {
            let (sources, expression) = body_expression(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert_eq!(shape(source, &expression), expected, "{body:?}");
        }

        let text = spec_source("a with [1..3] = x[0..2] ++ y[..2] ++ z[1..] ++ hex\"00\"");
        let (sources, expression) = body_expression(&text);
        let source = sources.iter().next().unwrap();
        let ExpressionKind::SliceUpdate(update) = &expression.kind else {
            panic!("expected a slice update");
        };
        assert_eq!(
            source.slice(expression.span),
            Some("a with [1..3] = x[0..2] ++ y[..2] ++ z[1..] ++ hex\"00\"")
        );
        assert_eq!(source.slice(update.keyword_span()), Some("with"));
        assert_eq!(source.slice(update.range().span()), Some("1..3"));
        assert_eq!(source.slice(update.range().dots_span()), Some(".."));
        assert_eq!(
            update
                .range()
                .start()
                .and_then(|start| source.slice(start.span)),
            Some("1")
        );
        assert_eq!(
            update.range().end().and_then(|end| source.slice(end.span)),
            Some("3")
        );
        let mut joined = update.value();
        let mut operands = Vec::new();
        while let ExpressionKind::Binary(binary) = &joined.kind {
            assert!(binary.operator.is_concatenation());
            assert_eq!(source.slice(binary.operator_span), Some("++"));
            operands.push(&*binary.right);
            joined = &binary.left;
        }
        operands.push(joined);
        operands.reverse();
        let slices = operands
            .iter()
            .map(|operand| {
                let ExpressionKind::Slice(slice) = &operand.kind else {
                    return (source.slice(operand.span).unwrap(), "", "");
                };
                (
                    source.slice(operand.span).unwrap(),
                    source.slice(slice.range().span()).unwrap(),
                    source.slice(slice.base().span).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            slices,
            [
                ("x[0..2]", "0..2", "x"),
                ("y[..2]", "..2", "y"),
                ("z[1..]", "1..", "z"),
                ("hex\"00\"", "", ""),
            ]
        );
        let ExpressionKind::Bytes(bytes) = &operands[3].kind else {
            panic!("expected a byte string");
        };
        assert!(bytes.is_hex());
    }

    #[test]
    fn rejects_malformed_bytes_and_slices_with_exact_messages() {
        let once = "a slice is taken once, from a name, a call, or a tuple's element; bind it \
                    with `let` to select from it";
        let cases = [
            (
                "a[..]",
                "expected a bound of the slice after `..`",
                SLICE_NOTE,
            ),
            (
                "a[0..1][0]",
                "expected an operator or the end of the expression",
                once,
            ),
            (
                "a[0..1][0..1]",
                "expected an operator or the end of the expression",
                once,
            ),
            (
                "a[0..1].0",
                "expected an operator or the end of the expression",
                "a slice is an array, not a tuple, so it has no `.k`",
            ),
            ("a[0..1..2]", "expected `]` after the slice", SLICE_NOTE),
            (
                "\"ab\"[0]",
                "expected an operator or the end of the expression",
                "a byte string is not indexed or sliced where it is written; bind it with \
                 `let` to select from it",
            ),
            (
                "a with [..] = b",
                "expected a bound of the slice after `..`",
                SLICE_UPDATE_NOTE,
            ),
            (
                "a with [0..2 = b",
                "expected `]` after the slice",
                SLICE_UPDATE_NOTE,
            ),
            (
                "a with [0..2] b",
                "expected `=` after the updated slice",
                SLICE_UPDATE_NOTE,
            ),
            (
                "hex \"00\"",
                "expected `}` after the body expression",
                SPACED_HEX_NOTE,
            ),
            (
                "let k: Word[8]^1 = hex \"00\"; k",
                "expected `;` after the bound expression",
                SPACED_HEX_NOTE,
            ),
        ];
        for (body, message, note) in cases {
            let (_, lexed, parsed) = parse_text(&spec_source(body));
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            assert_eq!(
                parsed.diagnostics.len(),
                1,
                "{body:?}: {:?}",
                parsed.diagnostics
            );
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{body:?}"
            );
            assert_eq!(diagnostic.message(), message, "{body:?}");
            assert_eq!(diagnostic.notes(), [note], "{body:?}");
        }

        // `++` is a group of its own: it mixes with no other operator
        // without parentheses.
        for (body, ungrouped, previous) in [
            ("a ++ b + c", "+", "++"),
            ("a + b ++ c", "++", "+"),
            ("a ++ b == c", "==", "++"),
            ("a ++ b as Word[8]^2", "as", "++"),
        ] {
            let (sources, _, parsed) = parse_text(&spec_source(body));
            let source = sources.iter().next().unwrap();
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            assert_eq!(parsed.diagnostics.len(), 1, "{body:?}");
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
            assert_eq!(source.slice(diagnostic.primary_span()), Some(ungrouped));
        }
    }

    #[test]
    fn slices_and_joins_count_toward_expression_height() {
        // A slice stands one level above the taller of its base and its
        // bounds, a slice update one above the tallest of its base, bounds,
        // and value, and `++` one above its taller operand.
        let tall = |additions: usize| format!("a{}", " + a".repeat(additions));
        type Form = (fn(&str) -> String, &'static str, usize);
        let forms: [Form; 5] = [
            (
                |value| format!("x[0..{value}]"),
                "[",
                MAX_EXPRESSION_HEIGHT - 2,
            ),
            (
                |value| format!("x[[{value}]..]"),
                "[",
                MAX_EXPRESSION_HEIGHT - 3,
            ),
            (
                |value| format!("x with [0..{value}] = y"),
                "with",
                MAX_EXPRESSION_HEIGHT - 2,
            ),
            (
                |value| format!("x with [..2] = [{value}]"),
                "with",
                MAX_EXPRESSION_HEIGHT - 3,
            ),
            (
                |value| format!("x ++ [{value}]"),
                "++",
                MAX_EXPRESSION_HEIGHT - 3,
            ),
        ];
        let message =
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit");
        for (form, at, additions) in forms {
            let (_, expression) = body_expression(&spec_source(&form(&tall(additions))));
            assert_eq!(tree_height(&expression), MAX_EXPRESSION_HEIGHT);
            assert_resource_limited(&form(&tall(additions + 1)), &message, at);
        }

        // A byte string is one node of height 1, whatever its length.
        let (_, expression) = body_expression(&spec_source(&format!("\"{}\"", "a".repeat(300))));
        assert_eq!(tree_height(&expression), 1);
    }

    #[test]
    fn parses_size_parameters_sized_types_and_sized_calls_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { use n; ",
            "spec f[len in 1..0x78, k in 0b1..3](x: Word[8]^len, y: Int^(2 * k)) ",
            "-> Word[8]^(len + 1) { g[len, (k)](x) ++ n::h[1](y) ++ [0; (len - 1)] } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let function = &ast.module.functions[0];
        let slice = |span: Span| source.slice(span).unwrap();
        assert_eq!(
            function
                .sizes()
                .iter()
                .map(|size| (
                    slice(size.span()),
                    size.name().text(),
                    slice(size.start_span()),
                    slice(size.end_span())
                ))
                .collect::<Vec<_>>(),
            [
                ("len in 1..0x78", "len", "1", "0x78"),
                ("k in 0b1..3", "k", "0b1", "3"),
            ]
        );
        let lengths = function
            .parameters
            .iter()
            .map(|parameter| slice(parameter.ty.length.as_ref().unwrap().span()))
            .collect::<Vec<_>>();
        assert_eq!(lengths, ["len", "(2 * k)"]);
        let FunctionBody::Typed(body) = &function.body else {
            panic!("expected a typed body");
        };
        assert_eq!(
            slice(body.result_type.length.as_ref().unwrap().span()),
            "(len + 1)"
        );
        let ExpressionKind::Binary(outer) = &body.expression.kind else {
            panic!("expected a join");
        };
        let ExpressionKind::Binary(inner) = &outer.left.kind else {
            panic!("expected a join");
        };
        let ExpressionKind::Call(local) = &inner.left.kind else {
            panic!("expected a call");
        };
        assert!(local.module().is_none());
        assert_eq!(local.callee().text(), "g");
        assert_eq!(
            local
                .sizes()
                .iter()
                .map(|size| slice(size.span))
                .collect::<Vec<_>>(),
            ["len", "(k)"]
        );
        assert_eq!(slice(inner.left.span), "g[len, (k)](x)");
        let ExpressionKind::Call(qualified) = &inner.right.kind else {
            panic!("expected a call");
        };
        assert_eq!(qualified.module().unwrap().text(), "n");
        assert_eq!(slice(qualified.sizes()[0].span), "1");
        assert_eq!(slice(inner.right.span), "n::h[1](y)");
        let ExpressionKind::Fill(fill) = &outer.right.kind else {
            panic!("expected a fill");
        };
        assert_eq!(slice(fill.length().span()), "(len - 1)");
    }

    #[test]
    fn a_name_before_brackets_is_indexed_unless_a_call_follows() {
        let shapes = [
            ("a[1]", "index"),
            ("a[i]", "index"),
            ("a[1..2]", "slice"),
            ("a[1](b)", "call 1"),
            ("a[i](b)", "call 1"),
            ("a[1, 2](b)", "call 2"),
            ("a[1, i + 1, 3, 4](b)", "call 4"),
            ("a[1](b)[0]", "index"),
            ("a[1](b)[0..1]", "slice"),
            ("n::a[1](b)", "call 1"),
            // Since S3o, a name's `[n]` and `^` may be written in a call's
            // brackets, for a type such as `Word[8]^4`.
            ("a[b[1]](c)", "call 1"),
            ("a[Word[8]^4, 2](c)", "call 2"),
            ("a[Mod[n]](c)", "call 1"),
            ("a[Mod[(1 << bits) - 19]^4, 2](c)", "call 2"),
        ];
        for (body, shape) in shapes {
            let (_, expression) = body_expression(&spec_source(body));
            let found = match &expression.kind {
                ExpressionKind::Index(_) => String::from("index"),
                ExpressionKind::Slice(_) => String::from("slice"),
                ExpressionKind::Call(call) => format!("call {}", call.sizes().len()),
                _ => String::from("other"),
            };
            assert_eq!(found, shape, "{body:?}");
        }
    }

    #[test]
    fn rejects_malformed_sizes_with_exact_messages() {
        let declarations = [
            (
                "spec f[n 1..3]() -> Int { 1 }",
                "expected `in` after the size's name",
            ),
            (
                "spec f[n in a..3]() -> Int { 1 }",
                "expected the size's first bound",
            ),
            (
                "spec f[n in 1 3]() -> Int { 1 }",
                "expected `..` between the size's bounds",
            ),
            (
                "spec f[n in 1..]() -> Int { 1 }",
                "expected the size's second bound",
            ),
            (
                "spec f[n in 1..3 m in 1..2]() -> Int { 1 }",
                "expected `,` or `]` after the size parameter",
            ),
            (
                "spec f[]() -> Int { 1 }",
                "expected an identifier for the size parameter",
            ),
            (
                "spec f[a in 1..2, b in 1..2, c in 1..2, d in 1..2, e in 1..2]() -> Int { 1 }",
                "a function has at most 4 size parameters",
            ),
            (
                "impl f[n in 1..2]() {}",
                "`impl` functions have no size parameters",
            ),
            (
                "spec f[n in 1..2]() {}",
                "expected `->` after the parameter list",
            ),
        ];
        for (declaration, message) in declarations {
            let text = format!("edition 2026; module m {{ {declaration} }}");
            let (_, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{text:?}");
            assert!(parsed.ast.is_none(), "accepted {text:?}");
            let diagnostic = parsed.diagnostics.first().unwrap();
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{text:?}"
            );
            assert_eq!(diagnostic.message(), message, "{text:?}");
        }
        let calls = [
            ("g[1, 2]", "expected `]` after the index"),
            ("g[1, 2 3](a)", "expected `,` or `]` after the size"),
            ("g[1, 2, 3, 4, 5](a)", "a call gives at most 4 sizes"),
            ("n::g[1]", "expected `(` after the sizes"),
            ("n::g[1 2](a)", "expected `,` or `]` after the size"),
            ("n::g[](a)", "expected an expression"),
            ("[0; n + 1]", "expected `]` after the array length"),
        ];
        for (body, message) in calls {
            let (_, lexed, parsed) = parse_text(&spec_source(body));
            assert!(lexed.diagnostics().is_empty(), "{body:?}");
            assert!(parsed.ast.is_none(), "accepted {body:?}");
            let diagnostic = parsed.diagnostics.first().unwrap();
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{body:?}"
            );
            assert_eq!(diagnostic.message(), message, "{body:?}");
        }
    }

    #[test]
    fn parses_type_parameters_as_lists_of_types_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec f[K in {F, Word[32], Mod[(1 << 130) - 5], Word[8]^4, (Int, Bool)}, n in 1..3]",
            "(x: K) -> K { g[K, Word[16], Word[8]^4, n](x) } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let source = sources.iter().next().unwrap();
        let ast = parsed.ast.unwrap();
        let function = &ast.module.functions[0];
        let slice = |span: Span| source.slice(span).unwrap();
        let [typed, sized] = function.sizes() else {
            panic!("expected two parameters in brackets");
        };
        assert!(typed.is_type());
        assert!(!sized.is_type());
        assert_eq!(
            (
                slice(typed.span()),
                typed.name().text(),
                slice(typed.start_span()),
                slice(typed.end_span())
            ),
            (
                "K in {F, Word[32], Mod[(1 << 130) - 5], Word[8]^4, (Int, Bool)}",
                "K",
                "{",
                "}"
            )
        );
        assert_eq!(
            typed
                .types()
                .iter()
                .map(|ty| slice(ty.span))
                .collect::<Vec<_>>(),
            [
                "F",
                "Word[32]",
                "Mod[(1 << 130) - 5]",
                "Word[8]^4",
                "(Int, Bool)"
            ]
        );
        assert!(sized.types().is_empty());
        assert_eq!(slice(sized.span()), "n in 1..3");
        let FunctionBody::Typed(body) = &function.body else {
            panic!("expected a typed body");
        };
        let ExpressionKind::Call(call) = &body.expression.kind else {
            panic!("expected a call");
        };
        assert_eq!(
            call.sizes()
                .iter()
                .map(|size| slice(size.span))
                .collect::<Vec<_>>(),
            ["K", "Word[16]", "Word[8]^4", "n"]
        );
    }

    #[test]
    fn rejects_malformed_type_parameters_with_exact_messages() {
        let declarations = [
            (
                "spec f[K in {}]() -> Int { 1 }",
                "expected a listed type after `{`",
            ),
            (
                "spec f[K in {Int,}]() -> Int { 1 }",
                "expected a listed type after `,`",
            ),
            (
                "spec f[K in {Int Bool}]() -> Int { 1 }",
                "expected `,` or `}` after the listed type",
            ),
            (
                "spec f[K in {Int]() -> Int { 1 }",
                "expected `,` or `}` after the listed type",
            ),
            (
                "spec f[K in {1}]() -> Int { 1 }",
                "expected an identifier for the listed type",
            ),
            (
                "spec f[K in {Int} n in 1..2]() -> Int { 1 }",
                "expected `,` or `]` after the type parameter",
            ),
            (
                "spec f[A in {Int}, B in {Int}, C in {Int}, D in {Int}, E in {Int}]() -> Int { 1 }",
                "a function has at most 4 size and type parameters",
            ),
            (
                "spec f[a in 1..2, B in {Int}, c in 1..2, d in 1..2, e in 1..2]() -> Int { 1 }",
                "a function has at most 4 size and type parameters",
            ),
            (
                "impl f[K in {Int}]() {}",
                "`impl` functions have no type parameters",
            ),
        ];
        for (declaration, message) in declarations {
            let text = format!("edition 2026; module m {{ {declaration} }}");
            let (_, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{text:?}");
            assert!(parsed.ast.is_none(), "accepted {text:?}");
            let diagnostic = parsed.diagnostics.first().unwrap();
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{text:?}"
            );
            assert_eq!(diagnostic.message(), message, "{text:?}");
        }
        // After an error in a list, parsing resumes after its `}`, so the
        // function's parameters and the next function add no error.
        for list in ["{}", "{Int,}", "{Int Bool}", "{Word[7] Bool}"] {
            let text = format!(
                "edition 2026; module m {{ spec f[K in {list}](x: Int) -> Int {{ x }} \
                 spec g() -> Int {{ 1 }} }}"
            );
            let (_, _, parsed) = parse_text(&text);
            assert_eq!(
                parsed.diagnostics.len(),
                1,
                "{text:?}: {:?}",
                parsed.diagnostics
            );
        }
    }

    #[test]
    fn parses_test_declarations_among_functions_with_exact_spans() {
        let text = concat!(
            "edition 2026; module m { ",
            "spec one() -> Int { 1 } ",
            "test \"one is one\" { let x: Int = one(); x == 1 } ",
            "spec test() -> Int { 2 } ",
            "test \"RFC 8439 2.1.1: #1\" { test() == 2 } ",
            "}"
        );
        let (sources, lexed, parsed) = parse_text(text);
        assert!(lexed.diagnostics().is_empty());
        assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
        let ast = parsed.ast.unwrap();
        let source = sources.iter().next().unwrap();

        // A function may be named `test`: the word opens a declaration
        // only where a function could start.
        let names = ast
            .module
            .functions
            .iter()
            .map(|function| function.name.text.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, ["one", "test"]);
        let tests = ast.module.tests();
        assert_eq!(tests.len(), 2);

        let first = &tests[0];
        assert_eq!(
            source.slice(first.span()),
            Some("test \"one is one\" { let x: Int = one(); x == 1 }")
        );
        assert_eq!(first.title().text(), "one is one");
        assert_eq!(source.slice(first.title().span()), Some("\"one is one\""));
        let body = first.body().unwrap();
        assert_eq!(body.bindings.len(), 1);
        assert_eq!(source.slice(body.expression.span), Some("x == 1"));
        assert!(matches!(
            &body.expression.kind,
            ExpressionKind::Binary(binary) if binary.operator == BinaryOperator::Equal
        ));

        // The test is checked as a `spec` without parameters named `test`,
        // whose result type `Bool` and name are placed at its title.
        let function = first.function();
        assert_eq!(function.kind, FunctionKind::Spec);
        assert_eq!(function.name.text, "test");
        assert_eq!(function.name.span, first.title().span());
        assert!(function.sizes.is_empty());
        assert!(function.parameters.is_empty());
        assert_eq!(body.result_type.name.text, "Bool");
        assert_eq!(body.result_type.span, first.title().span());
        assert_eq!(
            source.slice(body.span),
            Some("\"one is one\" { let x: Int = one(); x == 1 }")
        );

        let second = &tests[1];
        assert_eq!(second.title().text(), "RFC 8439 2.1.1: #1");
        assert_eq!(
            source.slice(second.body().unwrap().expression.span),
            Some("test() == 2")
        );
    }

    #[test]
    fn test_titles_are_kept_as_written_for_the_checker() {
        // The parser keeps a title's text between its quotes as written,
        // escapes included; the checker decides what a title may hold.
        for (title, text) in [
            ("\"\"", ""),
            ("\"tab\\there\"", "tab\\there"),
            ("\"caf\u{e9}\"", "caf\u{e9}"),
        ] {
            let source = format!("edition 2026; module m {{ test {title} {{ true }} }}");
            let (_, lexed, parsed) = parse_text(&source);
            assert!(lexed.diagnostics().is_empty(), "{source:?}");
            assert!(parsed.diagnostics.is_empty(), "{source:?}");
            let ast = parsed.ast.unwrap();
            assert_eq!(ast.module.tests()[0].title().text(), text);
        }
    }

    #[test]
    fn rejects_malformed_test_declarations_with_exact_messages() {
        let declarations = [
            ("test { true }", "expected a quoted title after `test`"),
            ("test one { true }", "expected a quoted title after `test`"),
            (
                "test hex\"01\" { true }",
                "expected a quoted title after `test`",
            ),
            ("test \"t\" true", "expected `{` after the test's title"),
            ("test \"t\" { }", "expected an expression"),
            (
                "test \"t\" { let x: Bool = true; }",
                "expected a result expression after the last binding",
            ),
            (
                "test \"t\" { true false }",
                "expected `}` after the body expression",
            ),
        ];
        for (declaration, message) in declarations {
            let text =
                format!("edition 2026; module m {{ {declaration} spec g() -> Int {{ 1 }} }}");
            let (_, lexed, parsed) = parse_text(&text);
            assert!(lexed.diagnostics().is_empty(), "{text:?}");
            assert!(parsed.ast.is_none(), "accepted {text:?}");
            assert_eq!(
                parsed.diagnostics.len(),
                1,
                "{text:?}: {:?}",
                parsed.diagnostics
            );
            let diagnostic = &parsed.diagnostics[0];
            assert_eq!(
                diagnostic.code(),
                DiagnosticCode::ExpectedSyntax,
                "{text:?}"
            );
            assert_eq!(diagnostic.message(), message, "{text:?}");
        }
        let (_, _, parsed) = parse_text("edition 2026; module m { test { true } }");
        assert_eq!(
            parsed.diagnostics[0].notes(),
            ["a test is written `test \"TITLE\" { EXPRESSION }`, its expression a `Bool`"]
        );
    }

    #[test]
    fn sizes_count_toward_the_height_of_their_expression() {
        // A size's group, a sized call's sizes, and a loop's computed bounds
        // are parts of the expression that holds them.
        for (body, height) in [
            ("[0; ((1))]", 4),
            ("g[((1))](a)", 4),
            ("for i in ((1))..2 with s: Int = 0 { s }", 4),
            ("g[1](a)", 2),
            ("[0; n]", 2),
        ] {
            let (_, expression) = body_expression(&spec_source(body));
            assert_eq!(tree_height(&expression), height, "{body:?}");
        }
        let message =
            format!("expression tree height exceeds the {MAX_EXPRESSION_HEIGHT}-level limit");
        let size = |count: usize| format!("[0; (1{})]", " + 1".repeat(count));
        let (_, expression) = body_expression(&spec_source(&size(MAX_EXPRESSION_HEIGHT - 3)));
        assert_eq!(tree_height(&expression), MAX_EXPRESSION_HEIGHT);
        assert_resource_limited(&size(MAX_EXPRESSION_HEIGHT - 2), &message, "[");
    }
}
