//! Deterministic lexical analysis for the pre-alpha Orange surface language.

use std::fmt;

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::edition::Edition;
use crate::source::{SourceFile, Span};

/// Maximum non-trivia tokens retained for one source, excluding EOF.
pub const MAX_TOKENS_PER_SOURCE: usize = 262_144;

/// Maximum ordinary lexical errors retained before one suppression diagnostic.
pub const MAX_DIAGNOSTICS_PER_SOURCE: usize = 100;

/// Maximum source spelling copied into one malformed-integer diagnostic.
const MAX_INTEGER_SPELLING_IN_DIAGNOSTIC: usize = 80;
const MAX_RETAINED_DIAGNOSTICS: usize = MAX_DIAGNOSTICS_PER_SOURCE.saturating_add(2);

macro_rules! define_token_kinds {
    ($(#[$variant_doc:meta] $variant:ident => $name:literal,)+) => {
        /// A lexical token category.
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub enum TokenKind {
            $(#[$variant_doc] $variant,)+
        }

        impl TokenKind {
            /// Returns a stable, uppercase name suitable for tools and snapshots.
            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }
    };
}

define_token_kinds! {
    /// End of input. Its span is empty at the source length.
    Eof => "EOF",
    /// An ASCII identifier that is not reserved in the selected edition.
    Identifier => "IDENTIFIER",
    /// A base-2, base-10, or base-16 integer literal.
    Integer => "INTEGER",
    /// A double-quoted string literal.
    String => "STRING",
    /// `edition`
    KwEdition => "KW_EDITION",
    /// `module`
    KwModule => "KW_MODULE",
    /// `spec`
    KwSpec => "KW_SPEC",
    /// `impl`
    KwImpl => "KW_IMPL",
    /// `game`
    KwGame => "KW_GAME",
    /// `proof`
    KwProof => "KW_PROOF",
    /// `claim`
    KwClaim => "KW_CLAIM",
    /// `(`
    LeftParen => "LEFT_PAREN",
    /// `)`
    RightParen => "RIGHT_PAREN",
    /// `{`
    LeftBrace => "LEFT_BRACE",
    /// `}`
    RightBrace => "RIGHT_BRACE",
    /// `[`
    LeftBracket => "LEFT_BRACKET",
    /// `]`
    RightBracket => "RIGHT_BRACKET",
    /// `,`
    Comma => "COMMA",
    /// `:`
    Colon => "COLON",
    /// `;`
    Semicolon => "SEMICOLON",
    /// `.`
    Dot => "DOT",
    /// `..`
    DotDot => "DOT_DOT",
    /// `::`
    DoubleColon => "DOUBLE_COLON",
    /// `+`
    Plus => "PLUS",
    /// `-`
    Minus => "MINUS",
    /// `*`
    Star => "STAR",
    /// `/`
    Slash => "SLASH",
    /// `%`
    Percent => "PERCENT",
    /// `&`
    Ampersand => "AMPERSAND",
    /// `&&`
    AmpAmp => "AMP_AMP",
    /// `|`
    Pipe => "PIPE",
    /// `||`
    PipePipe => "PIPE_PIPE",
    /// `^`
    Caret => "CARET",
    /// `~`
    Tilde => "TILDE",
    /// `!`
    Bang => "BANG",
    /// `=`
    Equal => "EQUAL",
    /// `<`
    Less => "LESS",
    /// `>`
    Greater => "GREATER",
    /// `==`
    EqualEqual => "EQUAL_EQUAL",
    /// `!=`
    BangEqual => "BANG_EQUAL",
    /// `<=`
    LessEqual => "LESS_EQUAL",
    /// `>=`
    GreaterEqual => "GREATER_EQUAL",
    /// `->`
    Arrow => "ARROW",
    /// `=>`
    FatArrow => "FAT_ARROW",
    /// `?`
    Question => "QUESTION",
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
    /// Heap-backed non-trivia tokens and their final [`TokenKind::Eof`].
    tokens: Vec<Token>,
    /// Allocation-free EOF fallback used only when the initial token slot
    /// cannot be reserved.
    inline_eof: Option<Token>,
    /// Recoverable lexical errors in source order.
    diagnostics: Vec<Diagnostic>,
}

impl Lexed {
    /// Returns the immutable token stream, including its final EOF token.
    #[must_use]
    pub fn tokens(&self) -> &[Token] {
        self.inline_eof
            .as_ref()
            .map_or(&self.tokens, std::slice::from_ref)
    }

    /// Returns lexical diagnostics in deterministic source order.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether lexical analysis found any errors.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        !self.diagnostics.is_empty() || self.inline_eof.is_some()
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
    token_storage_failed: bool,
    reserve_token_slots: fn(&mut Vec<Token>, usize) -> bool,
    reserve_diagnostic_slots: fn(&mut Vec<Diagnostic>, usize) -> bool,
}

fn reserve_token_slots(tokens: &mut Vec<Token>, additional: usize) -> bool {
    let Some(required_capacity) = tokens.len().checked_add(additional) else {
        return false;
    };
    if required_capacity <= tokens.capacity() {
        return true;
    }

    // Grow geometrically so a long token stream cannot force one reallocation
    // and copy per token. Never request speculative growth beyond the complete
    // bounded stream (ordinary tokens plus EOF); the allocator may still grant
    // more capacity than requested.
    let maximum_capacity = MAX_TOKENS_PER_SOURCE.saturating_add(1);
    let growth_floor = tokens.capacity().saturating_mul(2).max(4);
    let target_capacity = required_capacity.max(growth_floor).min(maximum_capacity);
    let Some(additional_capacity) = target_capacity.checked_sub(tokens.len()) else {
        return false;
    };
    if target_capacity < required_capacity {
        return false;
    }
    tokens.try_reserve_exact(additional_capacity).is_ok()
}

fn reserve_diagnostic_slots(diagnostics: &mut Vec<Diagnostic>, capacity: usize) -> bool {
    diagnostics.try_reserve_exact(capacity).is_ok()
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
            token_storage_failed: false,
            reserve_token_slots,
            reserve_diagnostic_slots,
        }
    }

    fn run(mut self) -> Lexed {
        if !(self.reserve_diagnostic_slots)(&mut self.diagnostics, MAX_RETAINED_DIAGNOSTICS) {
            let eof = Token {
                kind: TokenKind::Eof,
                span: self.span(self.text.len(), self.text.len()),
            };
            return Lexed {
                tokens: self.tokens,
                inline_eof: Some(eof),
                diagnostics: self.diagnostics,
            };
        }
        if !(self.reserve_token_slots)(&mut self.tokens, 1) {
            let span = self.span(0, 0);
            let eof = Token {
                kind: TokenKind::Eof,
                span: self.span(self.text.len(), self.text.len()),
            };
            self.token_storage_failed = true;
            self.push_token_storage_diagnostic(span);
            return Lexed {
                tokens: self.tokens,
                inline_eof: Some(eof),
                diagnostics: self.diagnostics,
            };
        }

        while self.cursor < self.text.len() && !self.token_storage_failed {
            if self.skip_trivia() {
                continue;
            }
            if self.cursor >= self.text.len() {
                break;
            }

            let start = self.cursor;
            let Some(character) = self.peek_char() else {
                self.fail_cursor_invariant();
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
            } else if self.token_storage_failed {
                break;
            } else {
                if !self.advance_char() {
                    break;
                }
                let span = self.span(start, self.cursor);
                let edition = self.edition;
                self.push_ordinary_diagnostic(span, || {
                    let printable = PrintableCharacter(character);
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

        if !self.token_storage_failed {
            let end = self.text.len();
            // Every ordinary-token insertion preserves one already allocated
            // slot, so installing EOF cannot allocate or fail here.
            self.push_token(TokenKind::Eof, end, end);
        }
        Lexed {
            tokens: self.tokens,
            inline_eof: None,
            diagnostics: self.diagnostics,
        }
    }

    fn skip_trivia(&mut self) -> bool {
        let initial = self.cursor;
        while self.peek_char().is_some_and(is_ascii_whitespace) {
            if !self.advance_char() {
                return true;
            }
        }

        if self.starts_with("//") {
            if !self.advance_bytes(2) {
                return true;
            }
            while self
                .peek_char()
                .is_some_and(|character| !matches!(character, '\n' | '\r'))
            {
                if !self.advance_char() {
                    return true;
                }
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
        if !self.advance_bytes(2) {
            return;
        }
        let mut depth = 1_u32;

        while self.cursor < self.text.len() {
            if self.starts_with("/*") {
                depth = depth.saturating_add(1);
                if !self.advance_bytes(2) {
                    return;
                }
            } else if self.starts_with("*/") {
                depth = depth.saturating_sub(1);
                if !self.advance_bytes(2) {
                    return;
                }
                if depth == 0 {
                    return;
                }
            } else {
                if !self.advance_char() {
                    return;
                }
            }
        }

        let opening_end = start.saturating_add(2).min(self.text.len());
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
        if !self.advance_char() {
            return;
        }
        while self.peek_char().is_some_and(is_identifier_continue) {
            if !self.advance_char() {
                return;
            }
        }

        let Some(spelling) = self.text.get(start..self.cursor) else {
            self.fail_cursor_invariant();
            return;
        };
        let kind = keyword_kind(spelling, self.edition).unwrap_or(TokenKind::Identifier);
        self.push_token(kind, start, self.cursor);
    }

    fn lex_integer(&mut self, start: usize) {
        let (base, digits_start) = if self.starts_with("0x") || self.starts_with("0X") {
            if !self.advance_bytes(2) {
                return;
            }
            (16_u32, self.cursor)
        } else if self.starts_with("0b") || self.starts_with("0B") {
            if !self.advance_bytes(2) {
                return;
            }
            (2_u32, self.cursor)
        } else {
            (10_u32, self.cursor)
        };

        while self
            .peek_char()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            if !self.advance_char() {
                return;
            }
        }

        let end = self.cursor;
        let Some(digits) = self.text.get(digits_start..end) else {
            self.fail_cursor_invariant();
            return;
        };
        if !valid_digits(digits, base) {
            let span = self.span(start, end);
            let Some(literal) = self.text.get(start..end) else {
                self.fail_cursor_invariant();
                return;
            };
            self.push_ordinary_diagnostic(span, move || {
                let spelling = integer_spelling_for_diagnostic(literal);
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
        if !self.advance_bytes(1) {
            return;
        }
        let mut terminated = false;

        while let Some(character) = self.peek_char() {
            match character {
                '"' => {
                    if !self.advance_bytes(1) {
                        return;
                    }
                    terminated = true;
                    break;
                }
                '\n' | '\r' => break,
                '\\' => self.lex_escape(),
                _ => {
                    if !self.advance_char() {
                        return;
                    }
                }
            }
        }

        if !terminated {
            let opening_end = start.saturating_add(1).min(self.text.len());
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
        if !self.advance_bytes(1) {
            return;
        }
        let Some(escaped) = self.peek_char() else {
            self.invalid_escape(start);
            return;
        };

        match escaped {
            '"' | '\\' | 'n' | 'r' | 't' | '0' => {
                self.advance_char();
            }
            '\n' | '\r' => self.invalid_escape(start),
            'x' => {
                if !self.advance_bytes(1) {
                    return;
                }
                let mut valid = true;
                for _ in 0..2 {
                    match self.peek_char() {
                        Some(character) if character.is_ascii_hexdigit() => {
                            if !self.advance_char() {
                                return;
                            }
                        }
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
                if !self.advance_char() {
                    return;
                }
                self.invalid_escape(start);
            }
        }
    }

    fn invalid_escape(&mut self, start: usize) {
        let end = self.cursor;
        let span = self.span(start, end);
        let Some(spelling) = self.text.get(start..end) else {
            self.fail_cursor_invariant();
            return;
        };
        self.push_ordinary_diagnostic(span, move || {
            Diagnostic::error(
                DiagnosticCode::InvalidEscape,
                invalid_escape_message(spelling),
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
                if !self.advance_bytes(spelling.len()) {
                    return None;
                }
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
        if !self.advance_char() {
            return None;
        }
        Some(kind)
    }

    fn push_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        if self.token_storage_failed {
            return;
        }
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
        if kind != TokenKind::Eof && !(self.reserve_token_slots)(&mut self.tokens, 2) {
            let span = self.span(start, end);
            self.tokens.clear();
            self.tokens.push(Token {
                kind: TokenKind::Eof,
                span: self.span(self.text.len(), self.text.len()),
            });
            self.token_storage_failed = true;
            self.push_token_storage_diagnostic(span);
            return;
        }
        self.tokens.push(Token {
            kind,
            span: self.span(start, end),
        });
    }

    fn push_token_storage_diagnostic(&mut self, span: Span) {
        self.push_resource_diagnostic(
            Diagnostic::error(
                DiagnosticCode::LexicalResourceLimit,
                "lexer could not reserve the bounded token stream representation",
                span,
            )
            .with_label("complete token stream storage could not be reserved")
            .with_note("the source was not accepted and no parser input was produced"),
        );
    }

    fn fail_cursor_invariant(&mut self) {
        if self.token_storage_failed {
            return;
        }
        let end = self.text.len();
        let span = self.span(end, end);
        self.tokens.clear();
        // Construction reserves this EOF slot before scanning begins, and
        // clearing the token vector retains that capacity.
        self.tokens.push(Token {
            kind: TokenKind::Eof,
            span,
        });
        self.token_storage_failed = true;
        self.cursor = end;
        self.push_resource_diagnostic(
            Diagnostic::error(
                DiagnosticCode::LexicalResourceLimit,
                "lexer source cursor invariant failed",
                span,
            )
            .with_label("source text could not be sliced at a UTF-8 boundary")
            .with_note("the source was not accepted and no parser input was produced"),
        );
    }

    fn push_ordinary_diagnostic(&mut self, primary_span: Span, build: impl FnOnce() -> Diagnostic) {
        if self.ordinary_diagnostics_emitted < MAX_DIAGNOSTICS_PER_SOURCE {
            self.ordinary_diagnostics_emitted = self.ordinary_diagnostics_emitted.saturating_add(1);
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
        self.text
            .get(self.cursor..)
            .is_some_and(|remaining| remaining.starts_with(spelling))
    }

    fn peek_char(&self) -> Option<char> {
        self.text.get(self.cursor..)?.chars().next()
    }

    fn advance_char(&mut self) -> bool {
        let Some(character) = self.peek_char() else {
            self.fail_cursor_invariant();
            return false;
        };
        self.advance_bytes(character.len_utf8())
    }

    fn advance_bytes(&mut self, byte_count: usize) -> bool {
        let Some(next) = self
            .cursor
            .checked_add(byte_count)
            .filter(|&next| next <= self.text.len() && self.text.is_char_boundary(next))
        else {
            self.fail_cursor_invariant();
            return false;
        };
        self.cursor = next;
        true
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

struct PrintableCharacter(char);

impl fmt::Display for PrintableCharacter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_ascii_graphic() {
            write!(formatter, "'{}'", self.0)
        } else {
            write!(formatter, "U+{:04X}", u32::from(self.0))
        }
    }
}

fn invalid_escape_message(spelling: &str) -> String {
    const PREFIX: &str = "invalid string escape `";

    let escaped_bytes = spelling.chars().flat_map(char::escape_default).count();
    let capacity = PREFIX.len().saturating_add(escaped_bytes).saturating_add(1);
    let mut message = String::with_capacity(capacity);
    message.push_str(PREFIX);
    message.extend(spelling.chars().flat_map(char::escape_default));
    message.push('`');
    message
}

struct DiagnosticIntegerSpelling<'spelling> {
    prefix: &'spelling str,
    total_bytes: Option<usize>,
}

impl fmt::Display for DiagnosticIntegerSpelling<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.prefix)?;
        if let Some(total_bytes) = self.total_bytes {
            write!(formatter, "...<{total_bytes} bytes total>")?;
        }
        Ok(())
    }
}

fn integer_spelling_for_diagnostic(spelling: &str) -> DiagnosticIntegerSpelling<'_> {
    if spelling.len() <= MAX_INTEGER_SPELLING_IN_DIAGNOSTIC {
        return DiagnosticIntegerSpelling {
            prefix: spelling,
            total_bytes: None,
        };
    }

    let mut prefix_end = MAX_INTEGER_SPELLING_IN_DIAGNOSTIC;
    while !spelling.is_char_boundary(prefix_end) {
        prefix_end = prefix_end.saturating_sub(1);
    }
    let prefix = spelling.get(..prefix_end).unwrap_or_default();
    DiagnosticIntegerSpelling {
        prefix,
        total_bytes: Some(spelling.len()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
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
    fn token_kind_name_inventory_is_exact_and_unique() {
        let actual = TokenKind::ALL
            .iter()
            .map(|kind| kind.name())
            .collect::<Vec<_>>();
        let expected = [
            "EOF",
            "IDENTIFIER",
            "INTEGER",
            "STRING",
            "KW_EDITION",
            "KW_MODULE",
            "KW_SPEC",
            "KW_IMPL",
            "KW_GAME",
            "KW_PROOF",
            "KW_CLAIM",
            "LEFT_PAREN",
            "RIGHT_PAREN",
            "LEFT_BRACE",
            "RIGHT_BRACE",
            "LEFT_BRACKET",
            "RIGHT_BRACKET",
            "COMMA",
            "COLON",
            "SEMICOLON",
            "DOT",
            "DOT_DOT",
            "DOUBLE_COLON",
            "PLUS",
            "MINUS",
            "STAR",
            "SLASH",
            "PERCENT",
            "AMPERSAND",
            "AMP_AMP",
            "PIPE",
            "PIPE_PIPE",
            "CARET",
            "TILDE",
            "BANG",
            "EQUAL",
            "LESS",
            "GREATER",
            "EQUAL_EQUAL",
            "BANG_EQUAL",
            "LESS_EQUAL",
            "GREATER_EQUAL",
            "ARROW",
            "FAT_ARROW",
            "QUESTION",
        ];

        assert_eq!(actual, expected);
        for (index, &name) in actual.iter().enumerate() {
            assert!(!actual[..index].contains(&name), "duplicate name {name}");
            assert!(
                name.bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte == b'_'),
                "invalid stable token name {name}",
            );
        }
    }

    #[test]
    fn reserved_and_punctuation_spellings_are_exact() {
        let expected = [
            ("edition", TokenKind::KwEdition),
            ("module", TokenKind::KwModule),
            ("spec", TokenKind::KwSpec),
            ("impl", TokenKind::KwImpl),
            ("game", TokenKind::KwGame),
            ("proof", TokenKind::KwProof),
            ("claim", TokenKind::KwClaim),
            ("(", TokenKind::LeftParen),
            (")", TokenKind::RightParen),
            ("{", TokenKind::LeftBrace),
            ("}", TokenKind::RightBrace),
            ("[", TokenKind::LeftBracket),
            ("]", TokenKind::RightBracket),
            (",", TokenKind::Comma),
            (":", TokenKind::Colon),
            (";", TokenKind::Semicolon),
            (".", TokenKind::Dot),
            ("..", TokenKind::DotDot),
            ("::", TokenKind::DoubleColon),
            ("+", TokenKind::Plus),
            ("-", TokenKind::Minus),
            ("*", TokenKind::Star),
            ("/", TokenKind::Slash),
            ("%", TokenKind::Percent),
            ("&", TokenKind::Ampersand),
            ("&&", TokenKind::AmpAmp),
            ("|", TokenKind::Pipe),
            ("||", TokenKind::PipePipe),
            ("^", TokenKind::Caret),
            ("~", TokenKind::Tilde),
            ("!", TokenKind::Bang),
            ("=", TokenKind::Equal),
            ("<", TokenKind::Less),
            (">", TokenKind::Greater),
            ("==", TokenKind::EqualEqual),
            ("!=", TokenKind::BangEqual),
            ("<=", TokenKind::LessEqual),
            (">=", TokenKind::GreaterEqual),
            ("->", TokenKind::Arrow),
            ("=>", TokenKind::FatArrow),
            ("?", TokenKind::Question),
        ];
        let text = expected
            .iter()
            .map(|(spelling, _)| *spelling)
            .collect::<Vec<_>>()
            .join(" ");
        let (sources, lexed) = lex_text(&text);
        let source = sources.iter().next().unwrap();

        assert_eq!(lexed.diagnostics(), []);
        assert_eq!(lexed.tokens().len(), expected.len() + 1);
        for (token, (spelling, kind)) in lexed.tokens().iter().zip(expected) {
            assert_eq!(token.kind, kind, "{spelling}");
            assert_eq!(token.lexeme(source), Some(spelling), "{spelling}");
        }
        let eof = lexed.tokens().last().unwrap();
        assert_eq!(eof.kind, TokenKind::Eof);
        assert_eq!(eof.lexeme(source), Some(""));
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
        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .map(Diagnostic::message)
                .collect::<Vec<_>>(),
            [
                "invalid string escape `\\\\q`",
                "invalid string escape `\\\\x`",
            ]
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
    fn exact_token_boundary_preserves_the_mandatory_eof_slot() {
        let text = "x ".repeat(MAX_TOKENS_PER_SOURCE);
        let (_, lexed) = lex_text(&text);

        assert!(lexed.diagnostics.is_empty());
        assert_eq!(lexed.tokens.len(), MAX_TOKENS_PER_SOURCE + 1);
        assert_eq!(lexed.tokens.capacity(), MAX_TOKENS_PER_SOURCE + 1);
        assert_eq!(lexed.tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn token_slot_reservation_uses_bounded_amortized_growth() {
        const SAMPLE_TOKENS: usize = 4_096;

        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "").unwrap();
        let source = sources.get(id).unwrap();
        let token = Token {
            kind: TokenKind::Identifier,
            span: source.lexer_span(0, 0),
        };
        let mut tokens = Vec::new();
        let mut growth_events = 0_usize;

        for _ in 0..SAMPLE_TOKENS {
            let previous_capacity = tokens.capacity();
            assert!(reserve_token_slots(&mut tokens, 2));
            if tokens.capacity() != previous_capacity {
                growth_events = growth_events.saturating_add(1);
            }
            tokens.push(token);
        }

        assert!(
            growth_events <= 16,
            "observed {growth_events} growth events"
        );
        assert!(tokens.capacity() > tokens.len());
        assert!(tokens.capacity() <= MAX_TOKENS_PER_SOURCE + 1);
    }

    #[test]
    fn ordinary_token_reservation_failure_discards_partial_tokens_and_preserves_eof() {
        let text = "a b c d";
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let mut lexer = Lexer::new(source, Edition::E2026);
        lexer.reserve_token_slots = |tokens, additional| {
            if tokens.len() == 2 {
                false
            } else {
                reserve_token_slots(tokens, additional)
            }
        };

        let lexed = lexer.run();

        let [eof] = lexed.tokens.as_slice() else {
            panic!("resource failure must discard every ordinary token");
        };
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
            "complete token stream storage could not be reserved"
        );
        assert_eq!(source.slice(diagnostic.primary_span()), Some("c"));
        assert_eq!(
            diagnostic.notes(),
            &["the source was not accepted and no parser input was produced"]
        );
        let parsed = parse(source, &lexed);
        assert!(parsed.ast().is_none());
        assert!(parsed.diagnostics().is_empty());
    }

    #[test]
    fn initial_eof_reservation_failure_uses_the_inline_eof_fallback() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "a").unwrap();
        let source = sources.get(id).unwrap();
        let mut lexer = Lexer::new(source, Edition::E2026);
        lexer.reserve_token_slots = |_, _| false;

        let lexed = lexer.run();

        assert!(lexed.tokens.is_empty());
        let [eof] = lexed.tokens() else {
            panic!("the allocation-free fallback must expose exactly EOF");
        };
        assert_eq!(eof.kind, TokenKind::Eof);
        assert_eq!(eof.span.start(), source.byte_len());
        assert_eq!(eof.span.end(), source.byte_len());
        assert_eq!(lexed.diagnostics.len(), 1);
        let diagnostic = &lexed.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::LexicalResourceLimit);
        assert_eq!(diagnostic.primary_span().start().bytes(), 0);
        assert!(diagnostic.primary_span().is_empty());
        assert_eq!(
            diagnostic.label(),
            "complete token stream storage could not be reserved"
        );
        assert_eq!(
            diagnostic.notes(),
            &["the source was not accepted and no parser input was produced"]
        );
        let parsed = parse(source, &lexed);
        assert!(parsed.ast().is_none());
        assert!(parsed.diagnostics().is_empty());
    }

    #[test]
    fn diagnostic_vector_reservation_failure_is_an_allocation_free_lexical_failure() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "accepted").unwrap();
        let source = sources.get(id).unwrap();
        let mut lexer = Lexer::new(source, Edition::E2026);
        lexer.reserve_diagnostic_slots = |_, _| false;

        let lexed = lexer.run();

        assert!(lexed.has_errors());
        assert!(lexed.diagnostics().is_empty());
        assert_eq!(lexed.diagnostics.capacity(), 0);
        let [eof] = lexed.tokens() else {
            panic!("diagnostic allocation failure must expose exactly EOF");
        };
        assert_eq!(eof.kind, TokenKind::Eof);
        assert_eq!(eof.span.start(), source.byte_len());
        assert_eq!(eof.span.end(), source.byte_len());
        let parsed = parse(source, &lexed);
        assert!(parsed.ast().is_none());
        assert!(parsed.diagnostics().is_empty());
    }

    #[test]
    fn invalid_internal_utf8_cursor_discards_tokens_and_fails_closed() {
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", "é").unwrap();
        let source = sources.get(id).unwrap();
        let mut lexer = Lexer::new(source, Edition::E2026);
        lexer.cursor = 1;

        let lexed = lexer.run();

        let [eof] = lexed.tokens() else {
            panic!("cursor invariant failure must expose exactly EOF");
        };
        assert_eq!(eof.kind, TokenKind::Eof);
        assert_eq!(eof.span.start(), source.byte_len());
        assert_eq!(eof.span.end(), source.byte_len());
        let [diagnostic] = lexed.diagnostics() else {
            panic!("cursor invariant failure must emit exactly one diagnostic");
        };
        assert_eq!(diagnostic.code(), DiagnosticCode::LexicalResourceLimit);
        assert_eq!(diagnostic.message(), "lexer source cursor invariant failed");
        assert_eq!(
            diagnostic.label(),
            "source text could not be sliced at a UTF-8 boundary"
        );
        let parsed = parse(source, &lexed);
        assert!(parsed.ast().is_none());
        assert!(parsed.diagnostics().is_empty());
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
        assert_eq!(first.diagnostics.capacity(), MAX_RETAINED_DIAGNOSTICS);
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
    fn token_storage_failure_survives_the_ordinary_diagnostic_budget() {
        let mut text = "@".repeat(MAX_DIAGNOSTICS_PER_SOURCE + 1);
        text.push_str(" a b");
        let mut sources = SourceMap::new();
        let id = sources.add("test.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let mut lexer = Lexer::new(source, Edition::E2026);
        lexer.reserve_token_slots = |tokens, additional| {
            if tokens.len() == 1 {
                false
            } else {
                reserve_token_slots(tokens, additional)
            }
        };

        let lexed = lexer.run();

        let [eof] = lexed.tokens.as_slice() else {
            panic!("resource failure must discard every ordinary token");
        };
        assert_eq!(eof.kind, TokenKind::Eof);
        assert_eq!(eof.span.start(), source.byte_len());
        assert_eq!(lexed.diagnostics.len(), MAX_RETAINED_DIAGNOSTICS);
        assert_eq!(lexed.diagnostics.capacity(), MAX_RETAINED_DIAGNOSTICS);
        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::UnexpectedCharacter)
                .count(),
            MAX_DIAGNOSTICS_PER_SOURCE
        );
        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::TooManyLexicalErrors)
                .count(),
            1
        );
        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code() == DiagnosticCode::LexicalResourceLimit)
                .count(),
            1
        );
        let parsed = parse(source, &lexed);
        assert!(parsed.ast().is_none());
        assert!(parsed.diagnostics().is_empty());
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
    fn short_integer_diagnostic_spelling_is_borrowed() {
        let spelling = "123abc";
        let diagnostic_spelling = integer_spelling_for_diagnostic(spelling);

        assert_eq!(diagnostic_spelling.prefix.as_ptr(), spelling.as_ptr());
        assert_eq!(diagnostic_spelling.total_bytes, None);
        assert_eq!(diagnostic_spelling.to_string(), spelling);
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

    #[test]
    fn scalar_delimiter_corpus_preserves_progress_and_span_partitioning() {
        let mut scalars = (0_u32..=u32::from(u8::MAX))
            .filter_map(char::from_u32)
            .collect::<Vec<_>>();
        scalars.extend(['\u{202e}', '🟠']);

        for character in scalars {
            let corpus = [
                format!("a{character}b"),
                format!(r#""left\{character}right" tail"#),
                format!("/*{character}*/z"),
                format!("0x{character}f"),
                format!("/{character}/z"),
            ];

            for text in corpus {
                let mut sources = SourceMap::new();
                let id = sources.add("progress.or", text.clone()).unwrap();
                let source = sources.get(id).unwrap();
                let first = lex(source, Edition::E2026);
                let second = lex(source, Edition::E2026);

                assert_eq!(first, second, "nondeterministic result for {text:?}");
                assert_eq!(
                    first.tokens.last().map(|token| token.kind),
                    Some(TokenKind::Eof),
                    "missing EOF for {text:?}"
                );
                assert!(
                    first.tokens.iter().all(|token| {
                        token.span.source() == id
                            && source.slice(token.span).is_some()
                            && (token.kind == TokenKind::Eof || !token.span.is_empty())
                    }),
                    "invalid or empty ordinary token span for {text:?}"
                );
                assert!(
                    first.tokens.windows(2).all(|tokens| {
                        tokens[0].span.end().bytes() <= tokens[1].span.start().bytes()
                    }),
                    "overlapping token spans for {text:?}"
                );
                assert!(
                    first.diagnostics.iter().all(|diagnostic| {
                        diagnostic.primary_span().source() == id
                            && source.slice(diagnostic.primary_span()).is_some()
                    }),
                    "invalid diagnostic span for {text:?}"
                );
            }
        }
    }
}
