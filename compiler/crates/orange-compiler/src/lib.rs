//! Permanent, dependency-free foundations for the Orange compiler frontend.
//!
//! The crate currently owns source identity and spans, edition selection,
//! deterministic diagnostics, and lexical analysis. Parsing and semantic
//! analysis will be layered on these APIs rather than replacing them.

pub mod diagnostic;
pub mod edition;
pub mod lexer;
pub mod source;

pub use diagnostic::{Diagnostic, DiagnosticCode, Severity, render_diagnostics};
pub use edition::{Edition, ParseEditionError};
pub use lexer::{Lexed, Token, TokenKind, lex};
pub use source::{
    LineColumn, MAX_SOURCE_BYTES, SourceError, SourceFile, SourceId, SourceMap, Span, TextOffset,
};
