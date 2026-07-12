//! Bounded, deterministic parsing for the minimal Orange 2026 source grammar.

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::edition::Edition;
use crate::lexer::{Lexed, MAX_TOKENS_PER_SOURCE, Token, TokenKind};
use crate::source::{SourceFile, Span};

/// Maximum ordinary syntax errors retained before one suppression diagnostic.
pub const MAX_PARSE_DIAGNOSTICS_PER_SOURCE: usize = 100;

/// Maximum AST nodes constructed for one source.
pub const MAX_SYNTAX_NODES_PER_SOURCE: usize = 262_144;

/// Maximum parser events (token advances, diagnostics, and node constructions).
pub const MAX_PARSE_EVENTS_PER_SOURCE: usize = 1_048_576;

/// Maximum delimiter nesting inspected while recovering from malformed syntax.
pub const MAX_RECOVERY_DELIMITER_DEPTH: usize = 64;

/// A complete minimal Orange source file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxTree {
    /// Extent from the edition keyword through the module's closing brace.
    pub span: Span,
    /// The mandatory source-edition declaration.
    pub edition: EditionDeclaration,
    /// The source's single module.
    pub module: ModuleDeclaration,
}

/// The mandatory `edition 2026;` declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EditionDeclaration {
    /// Full declaration extent.
    pub span: Span,
    /// Edition selected by the declaration.
    pub edition: Edition,
    /// Exact span of the `2026` spelling.
    pub value_span: Span,
}

/// A single `module NAME { ... }` declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleDeclaration {
    /// Full module extent.
    pub span: Span,
    /// Module name.
    pub name: Identifier,
    /// Functions in source order.
    pub functions: Vec<FunctionDeclaration>,
}

/// A minimal empty-body function declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionDeclaration {
    /// Full function extent.
    pub span: Span,
    /// Whether this is a `spec` or `impl` declaration.
    pub kind: FunctionKind,
    /// Function name.
    pub name: Identifier,
}

/// The two function declaration categories admitted by the minimal grammar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionKind {
    /// A `spec` declaration.
    Spec,
    /// An `impl` declaration.
    Impl,
}

/// An owned ASCII identifier and its source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    /// Original identifier spelling.
    pub text: String,
    /// Exact spelling extent.
    pub span: Span,
}

/// The complete result of parsing one token stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseResult {
    /// Complete AST, present only when parsing produced zero diagnostics.
    pub ast: Option<SyntaxTree>,
    /// Syntax and parser-resource diagnostics in deterministic source order.
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    /// Returns whether parsing did not produce a complete AST.
    ///
    /// This is also true when parsing was skipped because lexing failed.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.ast.is_none()
    }
}

/// Parses the minimal Orange 2026 grammar from a complete lexer result.
///
/// The grammar is intentionally closed: one edition declaration, one module,
/// and zero or more parameterless `spec` or `impl` functions with empty bodies.
/// Lexically invalid sources are not parsed and therefore cannot produce an
/// AST or cascading parser diagnostics.
#[must_use]
pub fn parse(source: &SourceFile, lexed: &Lexed) -> ParseResult {
    if lexed.has_errors() {
        return ParseResult {
            ast: None,
            diagnostics: Vec::new(),
        };
    }
    Parser::new(source, lexed.tokens(), Limits::DEFAULT).run()
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
        }
    }

    fn run(mut self) -> ParseResult {
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
            self.resource_limit("parser received an invalid lexer token stream".to_owned());
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
        let Some(eof) = self.tokens.last() else {
            return false;
        };
        if eof.kind != TokenKind::Eof
            || !eof.span.is_empty()
            || eof.span.start() != self.source.byte_len()
            || self.source.slice(eof.span) != Some("")
        {
            return false;
        }
        if self.tokens[..self.tokens.len() - 1]
            .iter()
            .any(|token| token.kind == TokenKind::Eof)
        {
            return false;
        }
        if self
            .tokens
            .iter()
            .any(|token| self.source.slice(token.span).is_none())
        {
            return false;
        }
        self.tokens
            .windows(2)
            .all(|tokens| tokens[0].span.end() <= tokens[1].span.start())
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
                        functions.push(function);
                    }
                }
                _ => {
                    self.report(
                        DiagnosticCode::ExpectedFunctionDeclaration,
                        "expected a `spec` or `impl` function declaration",
                        self.current_span(),
                        "this token cannot begin a module member",
                        "the minimal grammar admits only parameterless functions with empty bodies",
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
            "minimal functions have an empty parameter list",
            &[
                TokenKind::RightParen,
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
                TokenKind::LeftBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::RightBrace,
                TokenKind::Eof,
            ],
        );
        let left_brace = self.consume_or_recover(
            TokenKind::LeftBrace,
            "`{` to begin the empty function body",
            "minimal function bodies are written `{}`",
            &[
                TokenKind::RightBrace,
                TokenKind::KwSpec,
                TokenKind::KwImpl,
                TokenKind::Eof,
            ],
        );

        let right_brace = if self.current_kind() == TokenKind::RightBrace {
            self.bump()
        } else {
            self.expected(
                "`}` immediately after the function body's `{`",
                "nonempty function bodies are outside the minimal grammar",
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

        match (name, left_paren, right_paren, left_brace, right_brace) {
            (Some(name), Some(_), Some(_), Some(_), Some(right_brace)) if self.record_node() => {
                Some(FunctionDeclaration {
                    span: self.join(keyword.span, right_brace.span),
                    kind,
                    name,
                })
            }
            _ => None,
        }
    }

    fn parse_identifier(&mut self, role: &str) -> Option<Identifier> {
        if self.current_kind() != TokenKind::Identifier {
            self.expected(
                &format!("an identifier for the {role}"),
                "reserved words cannot be used as names",
            );
            return None;
        }
        let token = self.bump()?;
        let text = token.lexeme(self.source)?.to_owned();
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
        self.report(
            DiagnosticCode::ExpectedSyntax,
            format!("expected {expected}"),
            self.current_span(),
            format!("found {}", self.current_kind().name()),
            note,
        );
    }

    fn report(
        &mut self,
        code: DiagnosticCode,
        message: impl Into<String>,
        span: Span,
        label: impl Into<String>,
        note: impl Into<String>,
    ) {
        if self.halted || !self.event() {
            return;
        }
        if self.ordinary_diagnostics < self.limits.diagnostics {
            self.ordinary_diagnostics += 1;
            self.diagnostics.push(
                Diagnostic::error(code, message, span)
                    .with_label(label)
                    .with_note(note),
            );
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
                    depth += 1;
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
        self.nodes += 1;
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
        self.events += 1;
        true
    }

    fn resource_limit(&mut self, message: String) {
        self.halted = true;
        if self.resource_limit_reported {
            return;
        }
        self.resource_limit_reported = true;
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::ParserResourceLimit,
                message,
                self.current_span(),
            )
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
        assert_eq!(
            source.slice(ast.module.functions[0].span),
            Some("spec one() {}")
        );
        assert_eq!(ast.module.functions[1].kind, FunctionKind::Impl);
        assert_eq!(source.slice(ast.module.span), Some(&text[14..]));
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
        ];

        for text in corpus {
            let (_, _, parsed) = parse_text(text);
            assert!(parsed.has_errors(), "accepted {text:?}");
            assert!(parsed.ast.is_none(), "partial AST escaped for {text:?}");
        }
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
        let parsed = parse(second_sources.get(second_id).unwrap(), &foreign);

        assert!(parsed.ast.is_none());
        assert_eq!(parsed.diagnostics.len(), 1);
        assert_eq!(
            parsed.diagnostics[0].code(),
            DiagnosticCode::ParserResourceLimit
        );
        assert_eq!(
            parsed.diagnostics[0].message(),
            "parser received an invalid lexer token stream"
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
