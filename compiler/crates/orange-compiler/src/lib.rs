//! Permanent, dependency-free foundations for the Orange compiler frontend.
//!
//! The crate currently owns source identity and spans, edition selection,
//! deterministic diagnostics, lexical analysis, and the bounded minimal
//! parser. Semantic analysis will be layered on these APIs.

pub mod diagnostic;
pub mod edition;
pub mod lexer;
pub mod parser;
pub mod source;

pub use diagnostic::{Diagnostic, DiagnosticCode, Severity, render_diagnostics};
pub use edition::{Edition, ParseEditionError};
pub use lexer::{Lexed, Token, TokenKind, lex};
pub use parser::{
    EditionDeclaration, FunctionDeclaration, FunctionKind, Identifier,
    MAX_PARSE_DIAGNOSTICS_PER_SOURCE, MAX_PARSE_EVENTS_PER_SOURCE, MAX_RECOVERY_DELIMITER_DEPTH,
    MAX_SYNTAX_NODES_PER_SOURCE, ModuleDeclaration, ParseResult, SyntaxTree, parse,
};
pub use source::{
    LineColumn, MAX_SOURCE_BYTES, SourceError, SourceFile, SourceId, SourceMap, Span, TextOffset,
};
