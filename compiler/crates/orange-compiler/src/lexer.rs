//! Deterministic lexical analysis for the pre-alpha Orange surface language.

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::edition::Edition;
use crate::source::{SourceFile, Span};

/// Maximum non-trivia tokens retained for one source, excluding EOF.
pub const MAX_TOKENS_PER_SOURCE: usize = 262_144;

/// Maximum ordinary lexical errors retained before one suppression diagnostic.
pub const MAX_DIAGNOSTICS_PER_SOURCE: usize = 100;

/// Maximum source spelling copied into one malformed-integer diagnostic.
const MAX_INTEGER_SPELLING_IN_DIAGNOSTIC: usize = 80;

/// A lexical token category.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TokenKind {
    /// End of input. Its span is empty at the source length.
    Eof,
    /// An ASCII identifier that is not reserved in the selected edition.
    Identifier,
    /// A base-2, base-10, or base-16 integer literal.
    Integer,
    /// A double-quoted string literal.
    String,
    /// `edition`
    KwEdition,
    /// `module`
    KwModule,
    /// `spec`
    KwSpec,
    /// `impl`
    KwImpl,
    /// `game`
    KwGame,
    /// `proof`
    KwProof,
    /// `claim`
    KwClaim,
    /// `(`
    LeftParen,
    /// `)`
    RightParen,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `[`
    LeftBracket,
    /// `]`
    RightBracket,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `.`
    Dot,
    /// `..`
    DotDot,
    /// `::`
    DoubleColon,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `&`
    Ampersand,
    /// `&&`
    AmpAmp,
    /// `|`
    Pipe,
    /// `||`
    PipePipe,
    /// `^`
    Caret,
    /// `~`
    Tilde,
    /// `!`
    Bang,
    /// `=`
    Equal,
    /// `<`
    Less,
    /// `>`
    Greater,
    /// `==`
    EqualEqual,
    /// `!=`
    BangEqual,
    /// `<=`
    LessEqual,
    /// `>=`
    GreaterEqual,
    /// `->`
    Arrow,
    /// `=>`
    FatArrow,
    /// `?`
    Question,
}

impl TokenKind {
    /// Returns a stable, uppercase name suitable for tools and snapshots.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Eof => "EOF",
            Self::Identifier => "IDENTIFIER",
            Self::Integer => "INTEGER",
            Self::String => "STRING",
            Self::KwEdition => "KW_EDITION",
            Self::KwModule => "KW_MODULE",
            Self::KwSpec => "KW_SPEC",
            Self::KwImpl => "KW_IMPL",
            Self::KwGame => "KW_GAME",
            Self::KwProof => "KW_PROOF",
            Self::KwClaim => "KW_CLAIM",
            Self::LeftParen => "LEFT_PAREN",
            Self::RightParen => "RIGHT_PAREN",
            Self::LeftBrace => "LEFT_BRACE",
            Self::RightBrace => "RIGHT_BRACE",
            Self::LeftBracket => "LEFT_BRACKET",
            Self::RightBracket => "RIGHT_BRACKET",
            Self::Comma => "COMMA",
            Self::Colon => "COLON",
            Self::Semicolon => "SEMICOLON",
            Self::Dot => "DOT",
            Self::DotDot => "DOT_DOT",
            Self::DoubleColon => "DOUBLE_COLON",
            Self::Plus => "PLUS",
            Self::Minus => "MINUS",
            Self::Star => "STAR",
            Self::Slash => "SLASH",
            Self::Percent => "PERCENT",
            Self::Ampersand => "AMPERSAND",
            Self::AmpAmp => "AMP_AMP",
            Self::Pipe => "PIPE",
            Self::PipePipe => "PIPE_PIPE",
            Self::Caret => "CARET",
            Self::Tilde => "TILDE",
            Self::Bang => "BANG",
            Self::Equal => "EQUAL",
            Self::Less => "LESS",
            Self::Greater => "GREATER",
            Self::EqualEqual => "EQUAL_EQUAL",
            Self::BangEqual => "BANG_EQUAL",
            Self::LessEqual => "LESS_EQUAL",
            Self::GreaterEqual => "GREATER_EQUAL",
            Self::Arrow => "ARROW",
            Self::FatArrow => "FAT_ARROW",
            Self::Question => "QUESTION",
        }
    }
}

/// A token and its exact half-open source span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    /// The lexical category.
    pub kind: TokenKind,
    /// The original source extent.
    pub span: Span,
}

impl Token {
    /// Returns the original spelling when this token belongs to `source`.
    #[must_use]
    pub fn lexeme(self, source: &SourceFile) -> Option<&str> {
        source.slice(self.span)
    }
}

/// The complete result of lexing one source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lexed {
    /// Non-trivia tokens, always ending in exactly one [`TokenKind::Eof`].
    tokens: Vec<Token>,
    /// Recoverable lexical errors in source order.
    diagnostics: Vec<Diagnostic>,
}

impl Lexed {
    /// Returns the immutable token stream, including its final EOF token.
    #[must_use]
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    /// Returns lexical diagnostics in deterministic source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether lexical analysis found any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}

/// Lexes one source according to the selected language edition.
#[must_use]
pub fn lex(source: &SourceFile, edition: Edition) -> Lexed {
    Lexer::new(source, edition).run()
}

struct Lexer<'source> {
    source: &'source SourceFile,
    text: &'source str,
    edition: Edition,
    cursor: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    ordinary_diagnostics_emitted: usize,
    token_limit_reported: bool,
    diagnostic_limit_reported: bool,
}

impl<'source> Lexer<'source> {
    fn new(source: &'source SourceFile, edition: Edition) -> Self {
        Self {
            source,
            text: source.text(),
            edition,
            cursor: 0,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
            ordinary_diagnostics_emitted: 0,
            token_limit_reported: false,
            diagnostic_limit_reported: false,
        }
    }

    fn run(self) -> Lexed {
        self.run_with_eof_reservation(|tokens| tokens.try_reserve_exact(1).is_ok())
    }

    fn run_with_eof_reservation(
        mut self,
        reserve_eof: impl FnOnce(&mut Vec<Token>) -> bool,
    ) -> Lexed {
        while self.cursor < self.text.len() {
            if self.skip_trivia() {
                continue;
            }
            if self.cursor >= self.text.len() {
                break;
            }

            let start = self.cursor;
            let Some(character) = self.peek_char() else {
                break;
            };
            if is_identifier_start(character) {
                self.lex_identifier(start);
            } else if character.is_ascii_digit() {
                self.lex_integer(start);
            } else if character == '"' {
                self.lex_string(start);
            } else if let Some(kind) = self.lex_punctuation() {
                self.push_token(kind, start, self.cursor);
            } else {
                self.advance_char();
                let span = self.span(start, self.cursor);
                let edition = self.edition;
                self.push_ordinary_diagnostic(span, || {
                    let printable = printable_character(character);
                    Diagnostic::error(
                        DiagnosticCode::UnexpectedCharacter,
                        format!("unexpected character {printable}"),
                        span,
                    )
                    .with_label(format!("character is not part of Orange {edition}"))
                    .with_note("identifiers are ASCII in this pre-alpha edition")
                });
            }
        }

        let end = self.text.len();
        if self.tokens.len() == MAX_TOKENS_PER_SOURCE && !reserve_eof(&mut self.tokens) {
            // The stream is already at its deterministic maximum, so there is
            // an ordinary token slot available to preserve the mandatory EOF
            // invariant without making another allocation attempt. The
            // resource diagnostic prevents this incomplete stream from being
            // consumed by the parser.
            *self
                .tokens
                .last_mut()
                .expect("the positive token limit guarantees a final token") = Token {
                kind: TokenKind::Eof,
                span: self.span(end, end),
            };
            self.push_resource_diagnostic(
                Diagnostic::error(
                    DiagnosticCode::LexicalResourceLimit,
                    "lexer could not reserve the bounded token stream representation",
                    self.span(end, end),
                )
                .with_label("lexing stopped before token storage became incomplete")
                .with_note("the source was not accepted and no parser input was produced"),
            );
        } else {
            self.push_token(TokenKind::Eof, end, end);
        }
        Lexed {
            tokens: self.tokens,
            diagnostics: self.diagnostics,
        }
    }

    fn skip_trivia(&mut self) -> bool {
        let initial = self.cursor;
        while self.peek_char().is_some_and(is_ascii_whitespace) {
            self.advance_char();
        }

        if self.starts_with("//") {
            self.cursor += 2;
            while self
                .peek_char()
                .is_some_and(|character| !matches!(character, '\n' | '\r'))
            {
                self.advance_char();
            }
            return true;
        }

        if self.starts_with("/*") {
            self.lex_block_comment();
            return true;
        }

        self.cursor != initial
    }

    fn lex_block_comment(&mut self) {
        let start = self.cursor;
        self.cursor += 2;
        let mut depth = 1_u32;

        while self.cursor < self.text.len() {
            if self.starts_with("/*") {
                depth = depth.saturating_add(1);
                self.cursor += 2;
            } else if self.starts_with("*/") {
                depth -= 1;
                self.cursor += 2;
                if depth == 0 {
                    return;
                }
            } else {
                self.advance_char();
            }
        }

        let opening_end = (start + 2).min(self.text.len());
        let span = self.span(start, opening_end);
        self.push_ordinary_diagnostic(span, || {
            Diagnostic::error(
                DiagnosticCode::UnterminatedBlockComment,
                "unterminated block comment",
                span,
            )
            .with_label("this comment is never closed")
            .with_note("block comments may nest, and every opening `/*` needs a closing `*/`")
        });
    }

    fn lex_identifier(&mut self, start: usize) {
        self.advance_char();
        while self.peek_char().is_some_and(is_identifier_continue) {
            self.advance_char();
        }

        let spelling = &self.text[start..self.cursor];
        let kind = keyword_kind(spelling, self.edition).unwrap_or(TokenKind::Identifier);
        self.push_token(kind, start, self.cursor);
    }

    fn lex_integer(&mut self, start: usize) {
        let (base, digits_start) = if self.starts_with("0x") || self.starts_with("0X") {
            self.cursor += 2;
            (16_u32, self.cursor)
        } else if self.starts_with("0b") || self.starts_with("0B") {
            self.cursor += 2;
            (2_u32, self.cursor)
        } else {
            (10_u32, self.cursor)
        };

        while self
            .peek_char()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            self.advance_char();
        }

        let end = self.cursor;
        let digits = &self.text[digits_start..end];
        if !valid_digits(digits, base) {
            let span = self.span(start, end);
            let text = self.text;
            self.push_ordinary_diagnostic(span, move || {
                let spelling = integer_spelling_for_diagnostic(&text[start..end]);
                Diagnostic::error(
                    DiagnosticCode::MalformedInteger,
                    format!("malformed base-{base} integer literal `{spelling}`"),
                    span,
                )
                .with_label("invalid digits or separators")
                .with_note("underscores may appear only once between two valid digits")
            });
        }
        self.push_token(TokenKind::Integer, start, end);
    }

    fn lex_string(&mut self, start: usize) {
        self.cursor += 1;
        let mut terminated = false;

        while let Some(character) = self.peek_char() {
            match character {
                '"' => {
                    self.cursor += 1;
                    terminated = true;
                    break;
                }
                '\n' | '\r' => break,
                '\\' => self.lex_escape(),
                _ => self.advance_char(),
            }
        }

        if !terminated {
            let opening_end = (start + 1).min(self.text.len());
            let span = self.span(start, opening_end);
            self.push_ordinary_diagnostic(span, || {
                Diagnostic::error(
                    DiagnosticCode::UnterminatedString,
                    "unterminated string literal",
                    span,
                )
                .with_label("this string is never closed")
                .with_note("pre-alpha Orange strings cannot cross a line boundary")
            });
        }
        self.push_token(TokenKind::String, start, self.cursor);
    }

    fn lex_escape(&mut self) {
        let start = self.cursor;
        self.cursor += 1;
        let Some(escaped) = self.peek_char() else {
            self.invalid_escape(start);
            return;
        };

        match escaped {
            '"' | '\\' | 'n' | 'r' | 't' | '0' => self.advance_char(),
            '\n' | '\r' => self.invalid_escape(start),
            'x' => {
                self.cursor += 1;
                let mut valid = true;
                for _ in 0..2 {
                    match self.peek_char() {
                        Some(character) if character.is_ascii_hexdigit() => self.advance_char(),
                        _ => {
                            valid = false;
                            break;
                        }
                    }
                }
                if !valid {
                    self.invalid_escape(start);
                }
            }
            _ => {
                self.advance_char();
                self.invalid_escape(start);
            }
        }
    }

    fn invalid_escape(&mut self, start: usize) {
        let end = self.cursor;
        let span = self.span(start, end);
        let text = self.text;
        self.push_ordinary_diagnostic(span, move || {
            let spelling = &text[start..end];
            Diagnostic::error(
                DiagnosticCode::InvalidEscape,
                format!("invalid string escape `{}`", escape_for_message(spelling)),
                span,
            )
            .with_label("unsupported escape")
            .with_note(r#"supported escapes are \", \\, \n, \r, \t, \0, and \xNN"#)
        });
    }

    fn lex_punctuation(&mut self) -> Option<TokenKind> {
        const DOUBLE: [(&str, TokenKind); 10] = [
            ("..", TokenKind::DotDot),
            ("::", TokenKind::DoubleColon),
            ("&&", TokenKind::AmpAmp),
            ("||", TokenKind::PipePipe),
            ("==", TokenKind::EqualEqual),
            ("!=", TokenKind::BangEqual),
            ("<=", TokenKind::LessEqual),
            (">=", TokenKind::GreaterEqual),
            ("->", TokenKind::Arrow),
            ("=>", TokenKind::FatArrow),
        ];
        for (spelling, kind) in DOUBLE {
            if self.starts_with(spelling) {
                self.cursor += spelling.len();
                return Some(kind);
            }
        }

        let kind = match self.peek_char()? {
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            ',' => TokenKind::Comma,
            ':' => TokenKind::Colon,
            ';' => TokenKind::Semicolon,
            '.' => TokenKind::Dot,
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '&' => TokenKind::Ampersand,
            '|' => TokenKind::Pipe,
            '^' => TokenKind::Caret,
            '~' => TokenKind::Tilde,
            '!' => TokenKind::Bang,
            '=' => TokenKind::Equal,
            '<' => TokenKind::Less,
            '>' => TokenKind::Greater,
            '?' => TokenKind::Question,
            _ => return None,
        };
        self.advance_char();
        Some(kind)
    }

    fn push_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        if kind != TokenKind::Eof && self.tokens.len() >= MAX_TOKENS_PER_SOURCE {
            if !self.token_limit_reported {
                self.token_limit_reported = true;
                self.push_resource_diagnostic(
                    Diagnostic::error(
                        DiagnosticCode::LexicalTokenLimit,
                        format!("source exceeds the {MAX_TOKENS_PER_SOURCE}-token lexical limit"),
                        self.span(start, end),
                    )
                    .with_label("token limit reached")
                    .with_note("split the source into smaller files"),
                );
            }
            return;
        }
        self.tokens.push(Token {
            kind,
            span: self.span(start, end),
        });
    }

    fn push_ordinary_diagnostic(&mut self, primary_span: Span, build: impl FnOnce() -> Diagnostic) {
        if self.ordinary_diagnostics_emitted < MAX_DIAGNOSTICS_PER_SOURCE {
            self.ordinary_diagnostics_emitted += 1;
            self.diagnostics.push(build());
        } else if !self.diagnostic_limit_reported {
            self.diagnostic_limit_reported = true;
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TooManyLexicalErrors,
                    format!("stopped reporting after {MAX_DIAGNOSTICS_PER_SOURCE} lexical errors"),
                    primary_span,
                )
                .with_label("further lexical errors are suppressed")
                .with_note("fix the reported errors before checking this source again"),
            );
        }
    }

    fn push_resource_diagnostic(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    fn span(&self, start: usize, end: usize) -> Span {
        self.source.lexer_span(start, end)
    }

    fn starts_with(&self, spelling: &str) -> bool {
        self.text[self.cursor..].starts_with(spelling)
    }

    fn peek_char(&self) -> Option<char> {
        self.text[self.cursor..].chars().next()
    }

    fn advance_char(&mut self) {
        if let Some(character) = self.peek_char() {
            self.cursor += character.len_utf8();
        }
    }
}

const fn keyword_kind(spelling: &str, edition: Edition) -> Option<TokenKind> {
    match edition {
        Edition::E2026 => match spelling.as_bytes() {
            b"edition" => Some(TokenKind::KwEdition),
            b"module" => Some(TokenKind::KwModule),
            b"spec" => Some(TokenKind::KwSpec),
            b"impl" => Some(TokenKind::KwImpl),
            b"game" => Some(TokenKind::KwGame),
            b"proof" => Some(TokenKind::KwProof),
            b"claim" => Some(TokenKind::KwClaim),
            _ => None,
        },
    }
}

const fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

const fn is_ascii_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\r' | '\n')
}

const fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_'
}

fn valid_digits(digits: &str, base: u32) -> bool {
    if digits.is_empty() || digits.starts_with('_') || digits.ends_with('_') {
        return false;
    }

    let mut previous_was_separator = false;
    for character in digits.chars() {
        if character == '_' {
            if previous_was_separator {
                return false;
            }
            previous_was_separator = true;
        } else if character.is_digit(base) {
            previous_was_separator = false;
        } else {
            return false;
        }
    }
    true
}

fn printable_character(character: char) -> String {
    if character.is_ascii_graphic() {
        format!("'{character}'")
    } else {
        format!("U+{:04X}", u32::from(character))
    }
}

fn escape_for_message(text: &str) -> String {
    text.chars().flat_map(char::escape_default).collect()
}

fn integer_spelling_for_diagnostic(spelling: &str) -> String {
    if spelling.len() <= MAX_INTEGER_SPELLING_IN_DIAGNOSTIC {
        return spelling.to_owned();
    }

    let mut prefix_end = MAX_INTEGER_SPELLING_IN_DIAGNOSTIC;
    while !spelling.is_char_boundary(prefix_end) {
        prefix_end -= 1;
    }
    format!(
        "{}...<{} bytes total>",
        &spelling[..prefix_end],
        spelling.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{SourceMap, TextOffset};

    fn lex_text(text: &str) -> (SourceMap, Lexed) {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let lexed = lex(sources.get(id).unwrap(), Edition::E2026);
        (sources, lexed)
    }

    fn kinds(lexed: &Lexed) -> Vec<TokenKind> {
        lexed.tokens.iter().map(|token| token.kind).collect()
    }

    #[test]
    fn recognizes_keywords_identifiers_literals_and_punctuation() {
        let (sources, lexed) = lex_text(
            "edition module spec impl game proof claim name _x 12 0b1010 0xCA_FE \"ok\\n\" \
             (){}[],:;...::+-*/%&&&|||^~!==<<=>>=->=>?",
        );

        assert!(lexed.diagnostics.is_empty());
        assert_eq!(lexed.tokens.last().unwrap().kind, TokenKind::Eof);
        assert_eq!(lexed.tokens[0].kind, TokenKind::KwEdition);
        assert_eq!(lexed.tokens[1].kind, TokenKind::KwModule);
        assert_eq!(lexed.tokens[7].kind, TokenKind::Identifier);
        assert_eq!(lexed.tokens[9].kind, TokenKind::Integer);
        assert_eq!(lexed.tokens[12].kind, TokenKind::String);
        let source = sources.iter().next().unwrap();
        assert_eq!(lexed.tokens[10].lexeme(source), Some("0b1010"));
        assert!(kinds(&lexed).contains(&TokenKind::Arrow));
        assert!(kinds(&lexed).contains(&TokenKind::FatArrow));
        assert!(kinds(&lexed).contains(&TokenKind::DotDot));
    }

    #[test]
    fn ignores_line_comments_and_nested_block_comments() {
        let (_, lexed) = lex_text("one // ignored\n /* outer /* inner */ done */ two");
        assert!(lexed.diagnostics.is_empty());
        assert_eq!(
            kinds(&lexed),
            vec![TokenKind::Identifier, TokenKind::Identifier, TokenKind::Eof]
        );
    }

    #[test]
    fn uses_only_ascii_whitespace_and_stops_comments_at_all_line_endings() {
        let (_, lexed) = lex_text("one//a\rtwo//b\r\nthree//c\nfour\u{00a0}five");
        assert_eq!(
            kinds(&lexed),
            vec![
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Identifier,
                TokenKind::Eof,
            ]
        );
        assert_eq!(lexed.diagnostics.len(), 1);
        assert_eq!(
            lexed.diagnostics[0].code(),
            DiagnosticCode::UnexpectedCharacter
        );
        assert_eq!(
            lexed.diagnostics[0].message(),
            "unexpected character U+00A0"
        );
    }

    #[test]
    fn reports_an_unterminated_nested_comment_at_its_outer_opening() {
        let (_, lexed) = lex_text("ok /* outer /* inner */");
        assert_eq!(lexed.diagnostics.len(), 1);
        assert_eq!(
            lexed.diagnostics[0].code(),
            DiagnosticCode::UnterminatedBlockComment
        );
        assert_eq!(lexed.diagnostics[0].primary_span().start().bytes(), 3);
        assert_eq!(lexed.diagnostics[0].primary_span().len(), 2);
    }

    #[test]
    fn validates_integer_bases_and_separator_placement() {
        let (_, lexed) = lex_text("0x 0b102 1__2 12_ 123abc 0xCA_FE 1_000");
        let malformed: Vec<_> = lexed
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code() == DiagnosticCode::MalformedInteger)
            .map(Diagnostic::message)
            .collect();
        assert_eq!(malformed.len(), 5);
        assert!(malformed[0].contains("`0x`"));
        assert!(malformed[4].contains("`123abc`"));
    }

    #[test]
    fn reports_invalid_escapes_and_recovers_at_the_closing_quote() {
        let (_, lexed) = lex_text("\"bad\\q and \\xZ\" after");
        assert_eq!(
            kinds(&lexed),
            vec![TokenKind::String, TokenKind::Identifier, TokenKind::Eof]
        );
        assert_eq!(lexed.diagnostics.len(), 2);
        assert!(
            lexed
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code() == DiagnosticCode::InvalidEscape)
        );
    }

    #[test]
    fn unterminated_string_stops_before_every_logical_line_ending() {
        for ending in ["\n", "\r\n", "\r"] {
            let text = format!("\"first{ending}next");
            let (sources, lexed) = lex_text(&text);
            assert_eq!(lexed.diagnostics.len(), 1, "{ending:?}");
            assert_eq!(
                lexed.diagnostics[0].code(),
                DiagnosticCode::UnterminatedString
            );
            let source = sources.iter().next().unwrap();
            assert_eq!(lexed.tokens[0].lexeme(source), Some("\"first"));
            assert_eq!(lexed.tokens[1].lexeme(source), Some("next"));
        }
    }

    #[test]
    fn rejects_non_ascii_identifiers_one_scalar_at_a_time() {
        let (_, lexed) = lex_text("café β");
        assert_eq!(lexed.diagnostics.len(), 2);
        assert_eq!(lexed.diagnostics[0].primary_span().len(), 2);
        assert_eq!(lexed.diagnostics[1].primary_span().len(), 2);
        assert_eq!(
            lexed.diagnostics[0].message(),
            "unexpected character U+00E9"
        );
    }

    #[test]
    fn bounds_the_token_stream_and_reports_the_limit_once() {
        let text = "x ".repeat(MAX_TOKENS_PER_SOURCE + 2);
        let (_, lexed) = lex_text(&text);

        assert_eq!(lexed.tokens.len(), MAX_TOKENS_PER_SOURCE + 1);
        assert_eq!(lexed.tokens.last().unwrap().kind, TokenKind::Eof);
        assert_eq!(lexed.diagnostics.len(), 1);
        assert_eq!(
            lexed.diagnostics[0].code(),
            DiagnosticCode::LexicalTokenLimit
        );
        assert_eq!(
            lexed.diagnostics[0].message(),
            format!("source exceeds the {MAX_TOKENS_PER_SOURCE}-token lexical limit")
        );
    }

    #[test]
    fn exact_token_boundary_reserves_only_the_mandatory_eof_slot() {
        let text = "x ".repeat(MAX_TOKENS_PER_SOURCE);
        let (_, lexed) = lex_text(&text);

        assert!(lexed.diagnostics.is_empty());
        assert_eq!(lexed.tokens.len(), MAX_TOKENS_PER_SOURCE + 1);
        assert_eq!(lexed.tokens.capacity(), MAX_TOKENS_PER_SOURCE + 1);
        assert_eq!(lexed.tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn eof_reservation_failure_preserves_eof_and_fails_closed() {
        let text = "x ".repeat(MAX_TOKENS_PER_SOURCE);
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let source = sources.get(id).unwrap();

        let lexed = Lexer::new(source, Edition::E2026).run_with_eof_reservation(|tokens| {
            assert_eq!(tokens.len(), MAX_TOKENS_PER_SOURCE);
            false
        });

        assert_eq!(lexed.tokens.len(), MAX_TOKENS_PER_SOURCE);
        assert_eq!(lexed.tokens.capacity(), MAX_TOKENS_PER_SOURCE);
        let eof = lexed.tokens.last().unwrap();
        assert_eq!(eof.kind, TokenKind::Eof);
        assert_eq!(eof.span.start(), source.byte_len());
        assert_eq!(eof.span.end(), source.byte_len());
        assert_eq!(lexed.diagnostics.len(), 1);
        let diagnostic = &lexed.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::LexicalResourceLimit);
        assert_eq!(
            diagnostic.message(),
            "lexer could not reserve the bounded token stream representation"
        );
        assert_eq!(
            diagnostic.label(),
            "lexing stopped before token storage became incomplete"
        );
        assert_eq!(
            diagnostic.notes(),
            &["the source was not accepted and no parser input was produced"]
        );
    }

    #[test]
    fn token_limit_survives_the_ordinary_diagnostic_budget() {
        let mut text = "@".repeat(MAX_DIAGNOSTICS_PER_SOURCE + 1);
        text.push(' ');
        text.push_str(&"x ".repeat(MAX_TOKENS_PER_SOURCE + 1));
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let first = lex(source, Edition::E2026);
        let second = lex(source, Edition::E2026);

        assert_eq!(first, second);
        assert_eq!(first.tokens.len(), MAX_TOKENS_PER_SOURCE + 1);
        assert_eq!(first.diagnostics.len(), MAX_DIAGNOSTICS_PER_SOURCE + 2);
        assert_eq!(
            first
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedCharacter)
                .count(),
            MAX_DIAGNOSTICS_PER_SOURCE
        );
        assert_eq!(
            first
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::TooManyLexicalErrors)
                .count(),
            1
        );
        assert_eq!(
            first
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::LexicalTokenLimit)
                .count(),
            1
        );
    }

    #[test]
    fn bounds_lexical_diagnostics_and_emits_one_suppression_record() {
        let text = "@".repeat(MAX_DIAGNOSTICS_PER_SOURCE + 2);
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let first = lex(source, Edition::E2026);
        let second = lex(source, Edition::E2026);

        assert_eq!(first, second);
        assert_eq!(first.diagnostics.len(), MAX_DIAGNOSTICS_PER_SOURCE + 1);
        assert!(
            first.diagnostics[..MAX_DIAGNOSTICS_PER_SOURCE]
                .iter()
                .all(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedCharacter)
        );
        assert_eq!(
            first.diagnostics.last().unwrap().code(),
            DiagnosticCode::TooManyLexicalErrors
        );
        assert_eq!(
            first.diagnostics.last().unwrap().message(),
            format!("stopped reporting after {MAX_DIAGNOSTICS_PER_SOURCE} lexical errors")
        );
    }

    #[test]
    fn suppressed_ordinary_diagnostics_are_not_constructed() {
        use std::cell::Cell;

        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "@").unwrap();
        let source = sources.get(id).unwrap();
        let span = source.span(TextOffset::new(0), TextOffset::new(1)).unwrap();
        let constructed = Cell::new(0_usize);
        let mut lexer = Lexer::new(source, Edition::E2026);

        for _ in 0..(MAX_DIAGNOSTICS_PER_SOURCE + 2) {
            lexer.push_ordinary_diagnostic(span, || {
                constructed.set(constructed.get() + 1);
                Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "unexpected", span)
            });
        }

        assert_eq!(constructed.get(), MAX_DIAGNOSTICS_PER_SOURCE);
        assert_eq!(lexer.diagnostics.len(), MAX_DIAGNOSTICS_PER_SOURCE + 1);
        assert_eq!(
            lexer.diagnostics.last().unwrap().code(),
            DiagnosticCode::TooManyLexicalErrors
        );
    }

    #[test]
    fn bounds_long_malformed_integer_messages() {
        let text = format!("1{}", "a".repeat(MAX_INTEGER_SPELLING_IN_DIAGNOSTIC * 32));
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text.clone()).unwrap();
        let source = sources.get(id).unwrap();
        let first = lex(source, Edition::E2026);
        let second = lex(source, Edition::E2026);

        assert_eq!(first, second);
        assert_eq!(first.diagnostics.len(), 1);
        let message = first.diagnostics[0].message();
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::MalformedInteger
        );
        assert!(message.len() < MAX_INTEGER_SPELLING_IN_DIAGNOSTIC + 100);
        assert!(message.contains(&format!("<{} bytes total>", text.len())));
        assert!(!message.contains(&text));
    }

    #[test]
    fn malformed_corpus_is_deterministic_and_preserves_valid_spans() {
        let corpus = [
            "",
            "\0\u{1b}\u{7f}",
            "🟠 module",
            "/* /* deeply */ still open",
            "\"escape at eof\\",
            "\"bad \\x0Z \\u{}\"",
            "0x__ 0b_ 99bottles",
            "// no final newline",
            "\r\n\t@\rnext",
            "..::&&||==!=<=>=->=>",
        ];

        for text in corpus {
            let mut sources = SourceMap::new();
            let id = sources.add("corpus.or", text).unwrap();
            let source = sources.get(id).unwrap();
            let first = lex(source, Edition::E2026);
            let second = lex(source, Edition::E2026);

            assert_eq!(first, second, "nondeterministic result for {text:?}");
            assert_eq!(
                first.tokens.last().map(|token| token.kind),
                Some(TokenKind::Eof)
            );
            assert!(
                first.tokens.windows(2).all(|tokens| {
                    tokens[0].span.end().bytes() <= tokens[1].span.start().bytes()
                })
            );
            assert!(
                first
                    .tokens
                    .iter()
                    .all(|token| token.lexeme(source).is_some())
            );
            assert!(first.diagnostics.iter().all(|diagnostic| {
                diagnostic.primary_span().source() == id
                    && source.slice(diagnostic.primary_span()).is_some()
            }));
        }
    }
}
