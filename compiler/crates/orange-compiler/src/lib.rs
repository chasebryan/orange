//! Permanent, dependency-free foundations for the Orange compiler frontend.
//!
//! The crate currently owns source identity and spans, edition selection,
//! deterministic diagnostics, lexical analysis, bounded parsing, typed semantic
//! analysis with calls, pure expressions, bindings, conversions,
//! fixed-length arrays, and bounded loops, Core construction, and
//! deterministic reference evaluation.

pub mod core;
pub mod diagnostic;
pub mod documentation;
pub mod edition;
pub mod eval;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod semantics;
pub mod source;

pub use core::{
    ArrayType, CoreArray, CoreBinding, CoreConditional, CoreExpression, CoreFunction,
    CoreFunctionId, CoreLocal, CoreLoop, CoreModule, CoreNode, CoreNodeKind, CoreTuple, CoreType,
    CoreValue, ExactInteger, MAX_ARRAY_LENGTH, MAX_LOOP_BOUND, MAX_MODULUS_BITS,
    MAX_TUPLE_ELEMENTS, Modulus, Residue, TupleType,
};
pub use diagnostic::{Diagnostic, DiagnosticCode, SecondarySpan, Severity, render_diagnostics};
pub use documentation::{
    DocumentationResult, MAX_DOCUMENTATION_EVENTS_PER_SOURCE, MAX_DOCUMENTATION_HTML_BYTES,
    document_source,
};
pub use edition::{Edition, ParseEditionError};
pub use eval::{
    CallResult, EvaluatedFunction, EvaluationResult, Evaluator, MAX_CALL_DEPTH,
    MAX_EVALUATION_STEPS_PER_SOURCE, TestOutcome, TestRun, evaluate, evaluate_selected, run_tests,
};
pub use formatter::{
    FormatResult, MAX_FORMAT_EVENTS_PER_SOURCE, MAX_FORMATTED_SOURCE_BYTES, format_source,
};
pub use lexer::{
    Lexed, MAX_DIAGNOSTICS_PER_SOURCE as MAX_LEXICAL_DIAGNOSTICS_PER_SOURCE, MAX_TOKENS_PER_SOURCE,
    Token, TokenKind, lex,
};
pub use parser::{
    ArrayExpression, BinaryExpression, BinaryOperator, Binding, ByteOrder, CallExpression,
    ConditionalArm, ConditionalExpression, ConversionExpression, EditionDeclaration, Expression,
    ExpressionKind, FillExpression, FunctionBody, FunctionDeclaration, FunctionKind, Identifier,
    IndexExpression, IntegerLiteral, LoopExpression, MAX_ARGUMENTS_PER_CALL, MAX_ARRAY_ELEMENTS,
    MAX_BINDINGS_PER_BODY, MAX_EXPRESSION_HEIGHT, MAX_EXPRESSION_NESTING,
    MAX_PARAMETERS_PER_FUNCTION, MAX_PARSE_DIAGNOSTICS_PER_SOURCE, MAX_PARSE_EVENTS_PER_SOURCE,
    MAX_RECOVERY_DELIMITER_DEPTH, MAX_SYNTAX_NODES_PER_SOURCE, MAX_TYPES_PER_MODULE,
    MAX_USES_PER_MODULE, ModuleDeclaration, Parameter, ParseResult, SyntaxTree, TestDeclaration,
    TestTitle, TypeDeclaration, TypeSyntax, TypedBody, UnaryExpression, UnaryOperator,
    UpdateExpression, UseDeclaration, parse,
};
pub use semantics::{
    AnalysisResult, MAX_CORE_NODES_PER_SOURCE, MAX_INTEGER_BITS, MAX_MODULES_PER_PROGRAM,
    MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE, MAX_SEMANTIC_EVENTS_PER_SOURCE, MAX_TEST_TITLE_BYTES,
    analyze, analyze_program,
};
pub use source::{
    LineColumn, MAX_SOURCE_BYTES, RenderedSourceName, SourceError, SourceFile, SourceId, SourceMap,
    Span, TextOffset,
};
