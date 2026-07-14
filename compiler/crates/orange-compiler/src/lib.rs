//! Permanent, dependency-free foundations for the Orange compiler frontend.
//!
//! The crate currently owns source identity and spans, edition selection,
//! deterministic diagnostics, lexical analysis, bounded parsing, typed semantic
//! analysis, Core construction, and deterministic reference evaluation.

pub mod core;
pub mod diagnostic;
pub mod edition;
pub mod eval;
pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod source;

pub use core::{CoreFunction, CoreFunctionId, CoreModule, CoreType, CoreValue, ExactInteger};
pub use diagnostic::{Diagnostic, DiagnosticCode, SecondarySpan, Severity, render_diagnostics};
pub use edition::{Edition, ParseEditionError};
pub use eval::{EvaluatedFunction, EvaluationResult, MAX_EVALUATION_STEPS_PER_SOURCE, evaluate};
pub use lexer::{Lexed, Token, TokenKind, lex};
pub use parser::{
    EditionDeclaration, FunctionBody, FunctionDeclaration, FunctionKind, Identifier,
    IntegerLiteral, MAX_PARSE_DIAGNOSTICS_PER_SOURCE, MAX_PARSE_EVENTS_PER_SOURCE,
    MAX_RECOVERY_DELIMITER_DEPTH, MAX_SYNTAX_NODES_PER_SOURCE, ModuleDeclaration, ParseResult,
    SyntaxTree, TypeSyntax, TypedLiteralBody, parse,
};
pub use semantics::{
    AnalysisResult, MAX_CORE_NODES_PER_SOURCE, MAX_INTEGER_BITS,
    MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE, MAX_SEMANTIC_EVENTS_PER_SOURCE, analyze,
};
pub use source::{
    LineColumn, MAX_SOURCE_BYTES, RenderedSourceName, SourceError, SourceFile, SourceId, SourceMap,
    Span, TextOffset,
};
