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

/// A parameterless function declaration in the current Orange 2026 grammar.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDeclaration {
    /// Full function extent.
    pub(crate) span: Span,
    /// Whether this is a `spec` or `impl` declaration.
    pub(crate) kind: FunctionKind,
    /// Function name.
    pub(crate) name: Identifier,
    /// Empty legacy syntax or the typed literal body available to `spec`.
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

    /// Returns the empty legacy syntax or typed literal body.
    #[must_use]
    pub const fn body(&self) -> &FunctionBody {
        &self.body
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
    /// A result type and signed integer literal, admitted only for `spec`.
    TypedLiteral(TypedLiteralBody),
}

/// The complete typed tail of a literal `spec`, from `->` through `}`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedLiteralBody {
    /// Full extent from the `->` token through the body's closing brace.
    pub(crate) span: Span,
    /// Syntactic result type; semantic analysis resolves its meaning.
    pub(crate) result_type: TypeSyntax,
    /// The function body's sole signed integer literal.
    pub(crate) literal: IntegerLiteral,
}

impl TypedLiteralBody {
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

    /// Returns the function body's sole signed integer literal.
    #[must_use]
    pub const fn literal(&self) -> &IntegerLiteral {
        &self.literal
    }
}

/// A syntactic result type name with an optional integer width argument.
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
    reserve_identifier_text: fn(&mut String, usize) -> bool,
    reserve_diagnostic_slots: fn(&mut Vec<Diagnostic>, usize) -> bool,
}

fn reserve_function_slot(functions: &mut Vec<FunctionDeclaration>) -> bool {
    functions.try_reserve(1).is_ok()
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
                        "Orange 2026 admits empty functions and typed literal `spec` functions",
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
            "functions in this grammar have an empty parameter list",
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
        let right_paren = self.consume_or_recover(
            TokenKind::RightParen,
            "`)` after `(`",
            "parameters are not part of the minimal grammar",
            &[
                TokenKind::Arrow,
                TokenKind::LeftBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ],
        );

        let (body, body_end) = match self.current_kind() {
            TokenKind::LeftBrace => self.parse_empty_function_body(),
            TokenKind::Arrow if kind == FunctionKind::Spec => self.parse_typed_literal_body(),
            TokenKind::Arrow => {
                self.report(
                    DiagnosticCode::ExpectedSyntax,
                    "typed literal bodies are allowed only on `spec` functions",
                    self.current_span(),
                    "an `impl` function cannot have a typed literal body",
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
                        "a `spec` is either legacy-empty or has a typed literal body"
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

        match (name, left_paren, right_paren, body, body_end) {
            (Some(name), Some(_), Some(_), Some(body), Some(body_end)) if self.record_node() => {
                Some(FunctionDeclaration {
                    span: self.join(keyword.span, body_end.span),
                    kind,
                    name,
                    body,
                })
            }
            _ => None,
        }
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

    fn parse_typed_literal_body(&mut self) -> (Option<FunctionBody>, Option<Token>) {
        let arrow = self.bump();
        let result_type = self.parse_type_syntax();
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
            "typed `spec` bodies contain exactly one signed integer literal",
            &[
                TokenKind::Minus,
                TokenKind::Integer,
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ],
        );
        let literal = self.parse_integer_literal();
        if literal.is_none()
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
            self.expected(
                "`}` immediately after the integer literal",
                "typed `spec` bodies contain exactly one signed integer literal",
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

        match (arrow, result_type, left_brace, literal, right_brace) {
            (Some(arrow), Some(result_type), Some(_), Some(literal), Some(right_brace))
                if self.record_node() =>
            {
                (
                    Some(FunctionBody::TypedLiteral(TypedLiteralBody {
                        span: self.join(arrow.span, right_brace.span),
                        result_type,
                        literal,
                    })),
                    Some(right_brace),
                )
            }
            (_, _, _, _, right_brace) => (None, right_brace),
        }
    }

    fn parse_type_syntax(&mut self) -> Option<TypeSyntax> {
        let name = self.parse_identifier("result type")?;
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
                "an integer literal in the typed `spec` body",
                "the body contains exactly one optionally negative integer literal",
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
        let FunctionBody::TypedLiteral(answer_body) = &answer.body else {
            panic!("expected a typed literal body");
        };
        assert_eq!(source.slice(answer_body.span), Some("-> Int { -0x2a }"));
        assert_eq!(source.slice(answer_body.result_type.span), Some("Int"));
        assert_eq!(answer_body.result_type.name.text, "Int");
        assert_eq!(answer_body.result_type.width_span, None);
        assert_eq!(source.slice(answer_body.literal.span), Some("-0x2a"));
        assert_eq!(
            source.slice(answer_body.literal.magnitude_span),
            Some("0x2a")
        );
        assert!(answer_body.literal.negative);

        let byte = &ast.module.functions[1];
        let FunctionBody::TypedLiteral(byte_body) = &byte.body else {
            panic!("expected a typed literal body");
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
        assert_eq!(source.slice(byte_body.literal.span), Some("255"));
        assert!(!byte_body.literal.negative);
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
        let FunctionBody::TypedLiteral(body) = &function.body else {
            panic!("expected a typed literal body");
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
        assert_eq!(source.slice(body.literal.span), Some("-1_000"));
        assert!(body.literal.negative);
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
            "typed literal bodies are allowed only on `spec` functions"
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
}
