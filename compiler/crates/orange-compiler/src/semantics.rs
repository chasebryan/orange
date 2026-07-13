//! Bounded name resolution, type checking, and Core construction.

use std::collections::BTreeMap;

use crate::core::{
    CoreFunction, CoreFunctionId, CoreModule, CoreType, CoreValue, ExactInteger, Magnitude,
};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::{
    FunctionBody, FunctionDeclaration, FunctionKind, IntegerLiteral, SyntaxTree, TypeSyntax,
    TypedLiteralBody,
};
use crate::source::{SourceFile, Span};

/// Maximum ordinary semantic errors retained before one suppression diagnostic.
pub const MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE: usize = 100;

/// Maximum Typed Reference Core nodes constructed for one source.
pub const MAX_CORE_NODES_PER_SOURCE: usize = 262_144;

/// Maximum semantic events performed for one source.
pub const MAX_SEMANTIC_EVENTS_PER_SOURCE: usize = 1_048_576;

/// Maximum significant bits retained for one exact mathematical integer.
pub const MAX_INTEGER_BITS: usize = 16_384;

/// The complete result of semantic analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisResult {
    /// Typed Core, present only when semantic analysis produced no diagnostics.
    pub core: Option<CoreModule>,
    /// Semantic and semantic-resource diagnostics in deterministic source order.
    pub diagnostics: Vec<Diagnostic>,
}

impl AnalysisResult {
    /// Returns whether semantic analysis did not produce a complete Core module.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.core.is_none()
    }
}

/// Resolves and checks one successfully parsed Orange syntax tree.
///
/// Empty functions participate in namespace checking but do not enter Core.
/// The first semantic fragment assigns meaning only to typed `spec` functions.
#[must_use]
pub fn analyze(source: &SourceFile, ast: &SyntaxTree) -> AnalysisResult {
    Analyzer::new(source, ast, Limits::DEFAULT).run()
}

#[derive(Clone, Copy)]
struct Limits {
    diagnostics: usize,
    nodes: usize,
    events: usize,
    integer_bits: usize,
}

impl Limits {
    const DEFAULT: Self = Self {
        diagnostics: MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE,
        nodes: MAX_CORE_NODES_PER_SOURCE,
        events: MAX_SEMANTIC_EVENTS_PER_SOURCE,
        integer_bits: MAX_INTEGER_BITS,
    };
}

struct Analyzer<'source, 'ast> {
    source: &'source SourceFile,
    ast: &'ast SyntaxTree,
    diagnostics: Vec<Diagnostic>,
    ordinary_diagnostics: usize,
    diagnostic_limit_reported: bool,
    resource_limit_reported: bool,
    core_nodes: usize,
    events: usize,
    halted: bool,
    limits: Limits,
}

struct PendingFunction {
    span: Span,
    name: String,
    name_span: Span,
    result_type: CoreType,
    value: CoreValue,
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    fn new(source: &'source SourceFile, ast: &'ast SyntaxTree, limits: Limits) -> Self {
        Self {
            source,
            ast,
            diagnostics: Vec::new(),
            ordinary_diagnostics: 0,
            diagnostic_limit_reported: false,
            resource_limit_reported: false,
            core_nodes: 0,
            events: 0,
            halted: false,
            limits,
        }
    }

    fn run(mut self) -> AnalysisResult {
        let mut declarations = BTreeMap::<(FunctionKind, String), Span>::new();
        let mut pending_functions = Vec::new();

        for function in &self.ast.module.functions {
            // One event for the declaration-key lookup.
            if !self.event(function.name.span) {
                break;
            }

            let key = (function.kind, function.name.text.clone());
            if let Some(first_span) = declarations.get(&key).copied() {
                let kind = function_kind_name(function.kind);
                self.report(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateFunction,
                        format!("duplicate {kind} function `{}`", function.name.text),
                        function.name.span,
                    )
                    .with_label("this declaration repeats a name in the same namespace")
                    .with_secondary_span(first_span, "first declaration is here")
                    .with_note("`spec` and `impl` use separate declaration namespaces"),
                );
            } else {
                // A successful first declaration performs a separate insertion.
                if !self.event(function.name.span) {
                    break;
                }
                declarations.insert(key, function.name.span);
            }

            if let FunctionBody::TypedLiteral(body) = &function.body {
                if function.kind != FunctionKind::Spec {
                    self.report(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedTypedFunction,
                            "typed literal bodies are supported only on `spec` functions",
                            function.name.span,
                        )
                        .with_label("this `impl` function has no semantics in the current fragment")
                        .with_note("use an empty `impl` body or move the typed literal to a `spec` function"),
                    );
                    continue;
                }
                if let Some(pending) = self.analyze_typed_function(function, body) {
                    pending_functions.push(pending);
                }
            }
        }

        let core = if self.diagnostics.is_empty() && !self.halted {
            self.construct_core(pending_functions)
        } else {
            None
        };
        AnalysisResult {
            core,
            diagnostics: self.diagnostics,
        }
    }

    fn analyze_typed_function(
        &mut self,
        function: &FunctionDeclaration,
        body: &TypedLiteralBody,
    ) -> Option<PendingFunction> {
        let result_type = self.analyze_type(&body.result_type)?;
        let value = self.analyze_literal(result_type, &body.literal)?;
        Some(PendingFunction {
            span: function.span,
            name: function.name.text.clone(),
            name_span: function.name.span,
            result_type,
            value,
        })
    }

    fn construct_core(&mut self, pending: Vec<PendingFunction>) -> Option<CoreModule> {
        // The module itself is one Core node, including for an empty Core.
        if !self.record_core_node(self.ast.module.span) {
            return None;
        }
        let mut functions = Vec::with_capacity(pending.len());
        for pending_function in pending {
            // Each admitted entry contributes one function, type, and value node.
            if !self.record_core_node(pending_function.span)
                || !self.record_core_node(pending_function.span)
                || !self.record_core_node(pending_function.span)
            {
                return None;
            }
            let Some(id) = CoreFunctionId::from_index(functions.len()) else {
                self.resource_limit(
                    pending_function.span,
                    "Core function identity exceeds the u32 representation limit",
                );
                return None;
            };
            functions.push(CoreFunction {
                id,
                span: pending_function.span,
                name: pending_function.name,
                name_span: pending_function.name_span,
                result_type: pending_function.result_type,
                value: pending_function.value,
            });
        }
        Some(CoreModule {
            span: self.ast.module.span,
            name: self.ast.module.name.text.clone(),
            functions,
        })
    }

    fn analyze_type(&mut self, syntax: &TypeSyntax) -> Option<CoreType> {
        // The identifier and optional width are distinct parsed-type components.
        if !self.event(syntax.name.span) {
            return None;
        }
        if let Some(width_span) = syntax.width_span
            && !self.event(width_span)
        {
            return None;
        }
        match (syntax.name.text.as_str(), syntax.width_span) {
            ("Int", None) => Some(CoreType::Int),
            ("Word", Some(width_span)) => {
                if self.source.slice(width_span) == Some("8") {
                    Some(CoreType::Word8)
                } else {
                    self.report(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedWordWidth,
                            "only the exact type `Word[8]` is supported",
                            width_span,
                        )
                        .with_label("unsupported word width")
                        .with_note("word widths do not coerce, truncate, or wrap"),
                    );
                    None
                }
            }
            ("Word", None) => {
                self.report(
                    Diagnostic::error(
                        DiagnosticCode::UnsupportedWordWidth,
                        "`Word` requires the exact width `[8]`",
                        syntax.name.span,
                    )
                    .with_label("missing supported word width")
                    .with_note("write `Word[8]`"),
                );
                None
            }
            _ => {
                self.report(
                    Diagnostic::error(
                        DiagnosticCode::UnsupportedType,
                        format!("unsupported result type `{}`", syntax.name.text),
                        syntax.span,
                    )
                    .with_label("the current semantic fragment admits only `Int` and `Word[8]`")
                    .with_note(
                        "types are resolved contextually and never inferred by spelling similarity",
                    ),
                );
                None
            }
        }
    }

    fn analyze_literal(
        &mut self,
        result_type: CoreType,
        literal: &IntegerLiteral,
    ) -> Option<CoreValue> {
        // Sign inspection precedes exact magnitude decoding for both domains.
        if !self.event(literal.span) {
            return None;
        }
        let magnitude = self.parse_magnitude(literal, self.limits.integer_bits)?;
        match result_type {
            CoreType::Int => Some(CoreValue::Int(ExactInteger::new(
                literal.negative,
                magnitude,
            ))),
            CoreType::Word8 => {
                if literal.negative {
                    self.report(
                        Diagnostic::error(
                            DiagnosticCode::NegativeWordLiteral,
                            "`Word[8]` literals cannot be negative",
                            literal.span,
                        )
                        .with_label("negative value is outside the range 0 through 255")
                        .with_note("fixed-width words do not wrap or coerce negative integers"),
                    );
                    return None;
                }
                if let Some(value) = magnitude.to_u8() {
                    Some(CoreValue::Word8(value))
                } else {
                    self.report(
                        Diagnostic::error(
                            DiagnosticCode::WordLiteralOutOfRange,
                            "literal is outside the range of `Word[8]`",
                            literal.magnitude_span,
                        )
                        .with_label("expected a value from 0 through 255")
                        .with_note(
                            "fixed-width words do not truncate or wrap out-of-range integers",
                        ),
                    );
                    None
                }
            }
        }
    }

    fn parse_magnitude(&mut self, literal: &IntegerLiteral, bit_limit: usize) -> Option<Magnitude> {
        let Some(spelling) = self.source.slice(literal.magnitude_span) else {
            self.resource_limit(
                literal.span,
                "integer literal span does not belong to the analyzed source",
            );
            return None;
        };
        // Prefix inspection is one event whether the decimal default or an
        // explicit binary/hexadecimal prefix is selected.
        if !self.event(literal.magnitude_span) {
            return None;
        }
        let (radix, digits) = if let Some(digits) = spelling
            .strip_prefix("0b")
            .or_else(|| spelling.strip_prefix("0B"))
        {
            (2, digits)
        } else if let Some(digits) = spelling
            .strip_prefix("0x")
            .or_else(|| spelling.strip_prefix("0X"))
        {
            (16, digits)
        } else {
            (10, spelling)
        };

        let mut magnitude = Magnitude::zero();
        let mut significant = false;
        for character in digits.chars() {
            if character == '_' {
                continue;
            }
            let Some(digit) = character.to_digit(radix) else {
                self.resource_limit(
                    literal.magnitude_span,
                    "semantic analysis received a malformed integer AST",
                );
                return None;
            };
            significant |= digit != 0;
            if significant && !self.event(literal.magnitude_span) {
                return None;
            }
            magnitude.multiply_add(radix, digit);
            if magnitude.bit_len() > bit_limit {
                self.report(
                    Diagnostic::error(
                        DiagnosticCode::IntegerMagnitudeLimit,
                        format!(
                            "integer magnitude exceeds the {}-significant-bit limit",
                            self.limits.integer_bits
                        ),
                        literal.magnitude_span,
                    )
                    .with_label("exact integer is too large for this semantic fragment")
                    .with_note("the literal is rejected rather than truncated or approximated"),
                );
                return None;
            }
        }
        Some(magnitude)
    }

    fn event(&mut self, span: Span) -> bool {
        if self.halted {
            return false;
        }
        if self.events >= self.limits.events {
            self.resource_limit(span, "semantic event budget exhausted");
            return false;
        }
        self.events += 1;
        true
    }

    fn record_core_node(&mut self, span: Span) -> bool {
        if !self.event(span) {
            return false;
        }
        if self.core_nodes >= self.limits.nodes {
            self.resource_limit(span, "typed Core node budget exhausted");
            return false;
        }
        self.core_nodes += 1;
        true
    }

    fn report(&mut self, diagnostic: Diagnostic) {
        if self.halted {
            return;
        }
        // Every ordinary diagnostic emission attempt consumes one semantic event,
        // including attempts suppressed by the diagnostic budget.
        if !self.event(diagnostic.primary_span()) {
            return;
        }
        if self.ordinary_diagnostics < self.limits.diagnostics {
            self.ordinary_diagnostics += 1;
            self.diagnostics.push(diagnostic);
        } else if !self.diagnostic_limit_reported {
            self.diagnostic_limit_reported = true;
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TooManySemanticErrors,
                    "too many semantic errors; further errors are suppressed",
                    diagnostic.primary_span(),
                )
                .with_label("semantic diagnostic limit reached")
                .with_note(format!(
                    "at most {} ordinary semantic diagnostics are retained per source",
                    self.limits.diagnostics
                )),
            );
        }
    }

    fn resource_limit(&mut self, span: Span, detail: &str) {
        if !self.resource_limit_reported {
            self.resource_limit_reported = true;
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::SemanticResourceLimit,
                    "semantic analysis resource limit exceeded",
                    span,
                )
                .with_label(detail)
                .with_note("semantic analysis stopped without producing Core"),
            );
        }
        self.halted = true;
    }
}

const fn function_kind_name(kind: FunctionKind) -> &'static str {
    match kind {
        FunctionKind::Spec => "spec",
        FunctionKind::Impl => "impl",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edition::Edition;
    use crate::lexer::lex;
    use crate::parser::parse;
    use crate::source::{SourceId, SourceMap};

    struct Fixture {
        sources: SourceMap,
        id: SourceId,
        ast: SyntaxTree,
    }

    impl Fixture {
        fn new(text: impl Into<String>) -> Self {
            let mut sources = SourceMap::new();
            let id = sources.add("semantic.or", text.into()).unwrap();
            let ast = {
                let source = sources.get(id).unwrap();
                let lexed = lex(source, Edition::E2026);
                assert_eq!(lexed.diagnostics(), []);
                let parsed = parse(source, &lexed);
                assert_eq!(parsed.diagnostics, []);
                parsed.ast.unwrap()
            };
            Self { sources, id, ast }
        }

        fn source(&self) -> &SourceFile {
            self.sources.get(self.id).unwrap()
        }

        fn analyze(&self) -> AnalysisResult {
            analyze(self.source(), &self.ast)
        }

        fn analyze_with(&self, limits: Limits) -> AnalysisResult {
            Analyzer::new(self.source(), &self.ast, limits).run()
        }
    }

    #[test]
    fn exact_ints_accept_every_sign_class_in_every_radix() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec decimal_positive() -> Int { 1_234_567_890 }\n",
            "  spec decimal_zero() -> Int { 0 }\n",
            "  spec decimal_negative() -> Int { -10 }\n",
            "  spec decimal_negative_zero() -> Int { -0 }\n",
            "  spec binary_positive() -> Int { 0b1010_0101 }\n",
            "  spec binary_zero() -> Int { 0b0 }\n",
            "  spec binary_negative() -> Int { -0b1010_0101 }\n",
            "  spec binary_negative_zero() -> Int { -0B0 }\n",
            "  spec hexadecimal_positive() -> Int { 0Xdead_BEEF }\n",
            "  spec hexadecimal_zero() -> Int { 0x0 }\n",
            "  spec hexadecimal_negative() -> Int { -0x2a }\n",
            "  spec hexadecimal_negative_zero() -> Int { -0X0 }\n",
            "}\n",
        ));
        let result = fixture.analyze();
        assert_eq!(result.diagnostics, []);
        let values: Vec<_> = result
            .core
            .unwrap()
            .functions
            .iter()
            .map(|function| function.value.to_string())
            .collect();
        assert_eq!(
            values,
            [
                "1234567890",
                "0",
                "-10",
                "0",
                "165",
                "0",
                "-165",
                "0",
                "3735928559",
                "0",
                "-42",
                "0",
            ]
        );
    }

    #[test]
    fn integer_at_significant_bit_limit_is_exact() {
        let magnitude = format!("8{}", "0".repeat((MAX_INTEGER_BITS - 1) / 4));
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec huge() -> Int {{ 0x{magnitude} }} }}\n"
        ));
        let result = fixture.analyze();
        let core = result.core.unwrap();
        let CoreValue::Int(value) = &core.functions[0].value else {
            panic!("expected exact integer");
        };
        assert_eq!(value.magnitude_bits(), MAX_INTEGER_BITS);
    }

    #[test]
    fn integer_over_significant_bit_limit_is_rejected_without_core() {
        let magnitude = format!("1{}", "0".repeat(MAX_INTEGER_BITS / 4));
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec huge() -> Int {{ 0x{magnitude} }} }}\n"
        ));
        let result = fixture.analyze();
        assert!(result.core.is_none());
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::IntegerMagnitudeLimit
        );
    }

    #[test]
    fn word_boundaries_are_exact_and_stably_formatted() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec low() -> Word[8] { 0 }\n",
            "  spec one() -> Word[8] { 1 }\n",
            "  spec below_high() -> Word[8] { 254 }\n",
            "  spec high() -> Word[8] { 0xff }\n",
            "}\n",
        ));
        let result = fixture.analyze();
        let values: Vec<_> = result
            .core
            .unwrap()
            .functions
            .iter()
            .map(|function| function.value.to_string())
            .collect();
        assert_eq!(values, ["0x00", "0x01", "0xfe", "0xff"]);
    }

    #[test]
    fn words_reject_negative_and_out_of_range_values_without_coercion() {
        for (literal, code) in [
            ("-1", DiagnosticCode::NegativeWordLiteral),
            ("-0", DiagnosticCode::NegativeWordLiteral),
            ("256", DiagnosticCode::WordLiteralOutOfRange),
            ("0x1_00", DiagnosticCode::WordLiteralOutOfRange),
        ] {
            let fixture = Fixture::new(format!(
                "edition 2026; module values {{ spec bad() -> Word[8] {{ {literal} }} }}\n"
            ));
            let result = fixture.analyze();
            assert!(result.core.is_none());
            assert_eq!(result.diagnostics[0].code(), code);
        }

        let over_bit_limit = format!("1{}", "0".repeat(MAX_INTEGER_BITS / 4));
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec bad() -> Word[8] {{ 0x{over_bit_limit} }} }}\n"
        ));
        let result = fixture.analyze();
        assert!(result.core.is_none());
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::IntegerMagnitudeLimit
        );
    }

    #[test]
    fn namespace_uniqueness_is_per_kind_and_cites_the_first_declaration() {
        let duplicate = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec same() {}\n",
            "  spec same() -> Int { 1 }\n",
            "}\n",
        ));
        let result = duplicate.analyze();
        assert!(result.core.is_none());
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::DuplicateFunction
        );
        assert_eq!(result.diagnostics[0].secondary_spans().len(), 1);

        let cross_kind = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec same() {}\n",
            "  impl same() {}\n",
            "}\n",
        ));
        let result = cross_kind.analyze();
        assert_eq!(result.diagnostics, []);
        assert!(result.core.unwrap().functions.is_empty());
    }

    #[test]
    fn typed_impls_and_unadmitted_types_fail_closed() {
        let mut typed_impl =
            Fixture::new("edition 2026; module values { spec typed() -> Int { 1 } }\n");
        typed_impl.ast.module.functions[0].kind = FunctionKind::Impl;
        let result = typed_impl.analyze();
        assert!(result.core.is_none());
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::UnsupportedTypedFunction
        );

        let cases = [
            (
                "spec typed() -> Integer { 1 }",
                DiagnosticCode::UnsupportedType,
            ),
            (
                "spec typed() -> Word { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
            ),
            (
                "spec typed() -> Word[16] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
            ),
            (
                "spec typed() -> Word[08] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
            ),
            (
                "spec typed() -> Word[0x8] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
            ),
            (
                "spec typed() -> Word[1_0] { 1 }",
                DiagnosticCode::UnsupportedWordWidth,
            ),
            (
                "spec typed() -> Int[8] { 1 }",
                DiagnosticCode::UnsupportedType,
            ),
        ];
        for (declaration, code) in cases {
            let fixture =
                Fixture::new(format!("edition 2026; module values {{ {declaration} }}\n"));
            let result = fixture.analyze();
            assert!(result.core.is_none());
            assert_eq!(result.diagnostics[0].code(), code);
        }
    }

    #[test]
    fn core_ids_follow_only_typed_specs_in_source_order() {
        let fixture = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec empty() {}\n",
            "  impl also_empty() {}\n",
            "  spec first() -> Int { 7 }\n",
            "  spec second() -> Word[8] { 8 }\n",
            "}\n",
        ));
        let functions = fixture.analyze().core.unwrap().functions;
        assert_eq!(functions[0].id.index(), 0);
        assert_eq!(functions[0].name, "first");
        assert_eq!(functions[1].id.index(), 1);
        assert_eq!(functions[1].name, "second");
    }

    #[test]
    fn diagnostic_and_resource_limits_fail_closed() {
        let diagnostics = Fixture::new(concat!(
            "edition 2026; module values {\n",
            "  spec first() -> Nope { 1 }\n",
            "  spec second() -> Nope { 2 }\n",
            "}\n",
        ));
        let result = diagnostics.analyze_with(Limits {
            diagnostics: 1,
            ..Limits::DEFAULT
        });
        assert!(result.core.is_none());
        assert_eq!(
            result
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            [
                DiagnosticCode::UnsupportedType,
                DiagnosticCode::TooManySemanticErrors
            ]
        );

        let typed = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        for limits in [
            Limits {
                events: 0,
                ..Limits::DEFAULT
            },
            Limits {
                nodes: 0,
                ..Limits::DEFAULT
            },
        ] {
            let result = typed.analyze_with(limits);
            assert!(result.core.is_none());
            assert_eq!(
                result.diagnostics.last().unwrap().code(),
                DiagnosticCode::SemanticResourceLimit
            );
        }
    }

    #[test]
    fn injected_limits_match_normative_event_and_core_node_accounting() {
        let empty = Fixture::new("edition 2026; module values {}\n");
        let accepted = empty.analyze_with(Limits {
            nodes: 1,
            events: 1,
            ..Limits::DEFAULT
        });
        assert!(accepted.core.is_some());

        let typed = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
        // Two namespace operations, one type inspection, sign/prefix/digit
        // inspection, and four Core-node attempts total ten events. The Core is
        // one module plus one function, type, and value node.
        for limits in [
            Limits {
                nodes: 3,
                ..Limits::DEFAULT
            },
            Limits {
                events: 9,
                ..Limits::DEFAULT
            },
        ] {
            let result = typed.analyze_with(limits);
            assert!(result.core.is_none());
            assert_eq!(
                result.diagnostics.last().unwrap().code(),
                DiagnosticCode::SemanticResourceLimit
            );
        }
        let accepted = typed.analyze_with(Limits {
            nodes: 4,
            events: 10,
            ..Limits::DEFAULT
        });
        assert!(accepted.core.is_some());
        assert_eq!(accepted.diagnostics, []);
    }

    #[test]
    fn analysis_is_repeatable_and_empty_modules_produce_empty_core() {
        let fixture =
            Fixture::new("edition 2026; module values { spec empty() {} impl empty() {} }\n");
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second);
        assert!(first.core.unwrap().functions.is_empty());
    }
}
