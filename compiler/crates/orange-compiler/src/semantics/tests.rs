//! Unit tests of semantic analysis.

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
            assert_eq!(parsed.diagnostics(), []);
            parsed.into_ast().unwrap()
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

/// The typed name of a binding or an accumulator that is one name.
fn named_of(pattern: &Pattern) -> &TypedName {
    let Pattern::Name(typed) = pattern else {
        panic!("expected one name");
    };
    typed
}

fn named_mut(pattern: &mut Pattern) -> &mut TypedName {
    let Pattern::Name(typed) = pattern else {
        panic!("expected one name");
    };
    typed
}

fn typed_body_mut(ast: &mut SyntaxTree) -> &mut TypedBody {
    match &mut ast.module.functions.first_mut().unwrap().body {
        FunctionBody::Typed(body) => body,
        FunctionBody::Empty => unreachable!(),
    }
}

fn body_literal(body: &TypedBody) -> &IntegerLiteral {
    match &body.expression.kind {
        ExpressionKind::Literal(literal) => literal,
        _ => unreachable!(),
    }
}

fn body_literal_mut(ast: &mut SyntaxTree) -> &mut IntegerLiteral {
    match &mut typed_body_mut(ast).expression.kind {
        ExpressionKind::Literal(literal) => literal,
        _ => unreachable!(),
    }
}

fn power_of_two_decimal(exponent: usize) -> String {
    let mut digits = vec![1_u8];
    for _ in 0..exponent {
        let mut carry = 0_u8;
        for digit in &mut digits {
            let doubled = *digit * 2 + carry;
            *digit = doubled % 10;
            carry = doubled / 10;
        }
        if carry != 0 {
            digits.push(carry);
        }
    }
    digits
        .iter()
        .rev()
        .map(|digit| char::from(b'0' + *digit))
        .collect()
}

fn reference_decimal_from_bits(bits: &str) -> String {
    let mut digits = vec![0_u8];
    for bit in bits.bytes() {
        let mut carry = bit - b'0';
        for digit in &mut digits {
            let doubled = *digit * 2 + carry;
            *digit = doubled % 10;
            carry = doubled / 10;
        }
        if carry != 0 {
            digits.push(carry);
        }
    }
    digits
        .iter()
        .rev()
        .map(|digit| char::from(b'0' + *digit))
        .collect()
}

fn reference_hexadecimal_from_bits(bits: &str) -> String {
    let padding = bits.len().next_multiple_of(4) - bits.len();
    let padded = format!("{}{}", "0".repeat(padding), bits);
    padded
        .as_bytes()
        .chunks(4)
        .map(|nibble| {
            let value = nibble
                .iter()
                .fold(0_u32, |value, bit| value * 2 + u32::from(*bit - b'0'));
            char::from_digit(value, 16).unwrap()
        })
        .collect()
}

fn group_digits(digits: &str, width: usize) -> String {
    let separators = digits.len().saturating_sub(1) / width;
    let mut grouped = String::with_capacity(digits.len() + separators);
    for (index, digit) in digits.char_indices() {
        if index != 0 && (digits.len() - index).is_multiple_of(width) {
            grouped.push('_');
        }
        grouped.push(digit);
    }
    grouped
}

fn deterministic_bits(bit_length: usize, state: &mut u64) -> String {
    let mut bits = String::with_capacity(bit_length);
    bits.push('1');
    for _ in 1..bit_length {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        bits.push(if *state & 1 == 0 { '0' } else { '1' });
    }
    bits
}

#[test]
fn rejects_same_index_syntax_trees_from_another_source_map_repeatably() {
    let text = "edition 2026; module values { spec value() -> Int { 1 } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);

    let first_result = analyze(second.source(), &first.ast);
    let second_result = analyze(second.source(), &first.ast);

    assert_eq!(first_result, second_result);
    assert!(first_result.core.is_none());
    assert_eq!(first_result.diagnostics.len(), 1);
    assert_eq!(
        first_result.diagnostics[0].code(),
        DiagnosticCode::InvalidSemanticInput
    );
    assert_eq!(
        first_result.diagnostics[0].primary_span().source(),
        second.source().id()
    );
    assert!(first_result.diagnostics[0].primary_span().is_empty());
    assert_eq!(
        first_result.diagnostics[0].primary_span().start().bytes(),
        0
    );
    assert_eq!(
        crate::diagnostic::render_diagnostics(&second.sources, &first_result.diagnostics),
        concat!(
            "error[ORC0210]: semantic analysis received a syntax tree owned by another source\n",
            " --> semantic.or:1:1\n",
            "  |\n",
            "1 | edition 2026; module values { spec value() -> Int { 1 } }\n",
            "  | ^ analysis stopped at this source boundary\n",
            "  = note: parse and analyze each syntax tree with the same source file\n",
        )
    );
}

#[test]
fn rejects_every_foreign_nested_span_even_when_the_root_belongs_to_the_source() {
    let text = "edition 2026; module values { spec value() -> Word[8] { 1 } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    let foreign_function = second.ast.module.functions.first().unwrap();
    let foreign_body = match &foreign_function.body {
        FunctionBody::Typed(body) => body,
        FunctionBody::Empty => unreachable!(),
    };

    macro_rules! foreign_case {
        ($mutate:expr) => {{
            let mut ast = first.ast.clone();
            $mutate(&mut ast);
            ast
        }};
    }
    let cases = [
        foreign_case!(|ast: &mut SyntaxTree| ast.edition.span = second.ast.edition.span),
        foreign_case!(|ast: &mut SyntaxTree| ast.edition.value_span = second.ast.edition.value_span),
        foreign_case!(|ast: &mut SyntaxTree| ast.module.span = second.ast.module.span),
        foreign_case!(|ast: &mut SyntaxTree| ast.module.name.span = second.ast.module.name.span),
        foreign_case!(
            |ast: &mut SyntaxTree| ast.module.functions.first_mut().unwrap().span =
                foreign_function.span
        ),
        foreign_case!(
            |ast: &mut SyntaxTree| ast.module.functions.first_mut().unwrap().name.span =
                foreign_function.name.span
        ),
        foreign_case!(|ast: &mut SyntaxTree| typed_body_mut(ast).span = foreign_body.span),
        foreign_case!(
            |ast: &mut SyntaxTree| typed_body_mut(ast).result_type.span =
                foreign_body.result_type.span
        ),
        foreign_case!(
            |ast: &mut SyntaxTree| typed_body_mut(ast).result_type.name.span =
                foreign_body.result_type.name.span
        ),
        foreign_case!(
            |ast: &mut SyntaxTree| typed_body_mut(ast).result_type.width_span =
                foreign_body.result_type.width_span
        ),
        foreign_case!(|ast: &mut SyntaxTree| typed_body_mut(ast).expression.span =
            foreign_body.expression.span),
        foreign_case!(
            |ast: &mut SyntaxTree| body_literal_mut(ast).span = body_literal(foreign_body).span
        ),
        foreign_case!(
            |ast: &mut SyntaxTree| body_literal_mut(ast).magnitude_span =
                body_literal(foreign_body).magnitude_span
        ),
    ];

    for (case_index, ast) in cases.iter().enumerate() {
        assert_eq!(ast.span.source(), first.source().id(), "case {case_index}");

        let first_result = analyze(first.source(), ast);
        let second_result = analyze(first.source(), ast);

        assert_eq!(first_result, second_result, "case {case_index}");
        assert!(first_result.core.is_none(), "case {case_index}");
        assert_eq!(first_result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            first_result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
        assert_eq!(
            first_result.diagnostics[0].primary_span().source(),
            first.source().id(),
            "case {case_index}"
        );
    }
}

#[test]
fn foreign_input_diagnostic_reservation_failure_remains_fail_closed() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");

    let result = invalid_semantic_input(fixture.source(), |_| false);

    assert!(result.has_errors());
    assert!(result.core().is_none());
    assert!(result.diagnostics().is_empty());
    assert_eq!(result.diagnostics.capacity(), 0);
}

#[test]
fn rejects_another_file_from_the_same_source_map() {
    let text = "edition 2026; module values { spec value() -> Int { 1 } }\n";
    let mut sources = SourceMap::new();
    let first_id = sources.add("first.or", text).unwrap();
    let second_id = sources.add("second.or", text).unwrap();
    let ast = {
        let first = sources.get(first_id).unwrap();
        let lexed = lex(first, Edition::E2026);
        parse(first, &lexed).into_ast().unwrap()
    };

    let result = analyze(sources.get(second_id).unwrap(), &ast);

    assert!(result.core.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code(),
        DiagnosticCode::InvalidSemanticInput
    );
    assert_eq!(result.diagnostics[0].primary_span().source(), second_id);
}

#[test]
fn production_limits_match_the_s3a_specification() {
    assert_eq!(MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE, 100);
    assert_eq!(MAX_CORE_NODES_PER_SOURCE, 262_144);
    assert_eq!(MAX_SEMANTIC_EVENTS_PER_SOURCE, 1_048_576);
    assert_eq!(MAX_INTEGER_BITS, 16_384);
    assert_eq!(Limits::DEFAULT.diagnostics, 100);
    assert_eq!(Limits::DEFAULT.nodes, 262_144);
    assert_eq!(Limits::DEFAULT.events, 1_048_576);
    assert_eq!(Limits::DEFAULT.integer_bits, 16_384);
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
        .map(|function| function.body.literal().unwrap().to_string())
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
fn integer_decoding_matches_a_deterministic_u128_reference_corpus() {
    let mut values = vec![
        0,
        1,
        u128::from(u32::MAX),
        1_u128 << 32,
        u128::from(u64::MAX),
        1_u128 << 64,
        u128::MAX,
    ];
    let mut state = 0x6a09_e667_f3bc_c908_bb67_ae85_84ca_a73b_u128;
    for _ in 0..32 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        values.push(state);
    }

    let mut source = String::from("edition 2026; module reference_corpus {\n");
    let mut expected = Vec::with_capacity(values.len() * 4);
    for (index, value) in values.into_iter().enumerate() {
        source.push_str(&format!("  spec decimal_{index}() -> Int {{ {value} }}\n"));
        expected.push(value.to_string());

        source.push_str(&format!(
            "  spec binary_{index}() -> Int {{ 0b{value:b} }}\n"
        ));
        expected.push(value.to_string());

        source.push_str(&format!(
            "  spec hexadecimal_{index}() -> Int {{ 0X{value:X} }}\n"
        ));
        expected.push(value.to_string());

        source.push_str(&format!(
            "  spec negative_{index}() -> Int {{ -0x{value:x} }}\n"
        ));
        expected.push(if value == 0 {
            String::from("0")
        } else {
            format!("-{value}")
        });
    }
    source.push_str("}\n");

    let fixture = Fixture::new(source);
    let first = fixture.analyze();
    let second = fixture.analyze();
    assert_eq!(first, second);
    assert_eq!(first.diagnostics, []);
    let observed: Vec<_> = first
        .core
        .unwrap()
        .functions
        .iter()
        .map(|function| function.body.literal().unwrap().to_string())
        .collect();
    assert_eq!(observed, expected);
}

#[test]
fn multi_limb_integer_decoding_matches_independent_cross_radix_reference() {
    let bit_lengths = [
        31,
        32,
        33,
        63,
        64,
        65,
        127,
        128,
        129,
        255,
        256,
        257,
        511,
        512,
        513,
        1_023,
        1_024,
        1_025,
        4_095,
        4_096,
        4_097,
        MAX_INTEGER_BITS - 1,
        MAX_INTEGER_BITS,
    ];
    let mut state = 0xd1b5_4a32_d192_ed03_u64;
    let mut source = String::from("edition 2026; module multi_limb_reference {\n");
    let mut expected = Vec::with_capacity(bit_lengths.len() * 4);

    for (index, bit_length) in bit_lengths.into_iter().enumerate() {
        let bits = deterministic_bits(bit_length, &mut state);
        let decimal = reference_decimal_from_bits(&bits);
        let hexadecimal = reference_hexadecimal_from_bits(&bits);

        source.push_str(&format!(
            "  spec decimal_{index}() -> Int {{ {} }}\n",
            group_digits(&decimal, 4)
        ));
        expected.push(decimal.clone());

        source.push_str(&format!(
            "  spec binary_{index}() -> Int {{ 0b{} }}\n",
            group_digits(&bits, 5)
        ));
        expected.push(decimal.clone());

        source.push_str(&format!(
            "  spec hexadecimal_{index}() -> Int {{ 0X{} }}\n",
            group_digits(&hexadecimal.to_uppercase(), 3)
        ));
        expected.push(decimal.clone());

        source.push_str(&format!(
            "  spec negative_{index}() -> Int {{ -0x{} }}\n",
            group_digits(&hexadecimal, 7)
        ));
        expected.push(format!("-{decimal}"));
    }
    source.push_str("}\n");

    let fixture = Fixture::new(source);
    let first = fixture.analyze();
    let second = fixture.analyze();
    assert_eq!(first, second);
    assert_eq!(first.diagnostics, []);
    let core = first.core.as_ref().unwrap();
    let observed_core: Vec<_> = core
        .functions
        .iter()
        .map(|function| function.body.literal().unwrap().to_string())
        .collect();
    assert_eq!(observed_core, expected);

    let first_evaluation = crate::eval::evaluate(core);
    let second_evaluation = crate::eval::evaluate(core);
    assert_eq!(first_evaluation, second_evaluation);
    assert_eq!(first_evaluation.diagnostics(), []);
    let observed_evaluation: Vec<_> = first_evaluation
        .values()
        .unwrap()
        .iter()
        .map(|function| function.value().to_string())
        .collect();
    assert_eq!(observed_evaluation, expected);
}

#[test]
fn large_integer_rendering_matches_decimal_doubling_reference() {
    let exponents = [128, 255, 256, 1_024, MAX_INTEGER_BITS - 1];
    let mut source = String::from("edition 2026; module large_integers {\n");
    let mut expected = Vec::with_capacity(exponents.len() + 2);
    for exponent in exponents {
        let decimal = power_of_two_decimal(exponent);
        source.push_str(&format!(
            "  spec power_{exponent}() -> Int {{ 0b1{} }}\n",
            "0".repeat(exponent)
        ));
        expected.push(decimal.clone());
        if exponent == MAX_INTEGER_BITS - 1 {
            source.push_str(&format!("  spec decimal_limit() -> Int {{ {decimal} }}\n"));
            expected.push(decimal.clone());
            source.push_str(&format!(
                "  spec hexadecimal_limit() -> Int {{ 0x8{} }}\n",
                "0".repeat((MAX_INTEGER_BITS - 4) / 4)
            ));
            expected.push(decimal);
        }
    }
    source.push_str("}\n");

    let fixture = Fixture::new(source);
    let first = fixture.analyze();
    let second = fixture.analyze();
    assert_eq!(first, second);
    assert_eq!(first.diagnostics, []);
    let observed: Vec<_> = first
        .core
        .unwrap()
        .functions
        .iter()
        .map(|function| function.body.literal().unwrap().to_string())
        .collect();
    assert_eq!(observed, expected);
}

#[test]
fn integer_at_significant_bit_limit_is_exact() {
    let magnitude = format!("8{}", "0".repeat((MAX_INTEGER_BITS - 1) / 4));
    let fixture = Fixture::new(format!(
        "edition 2026; module values {{ spec huge() -> Int {{ 0x{magnitude} }} }}\n"
    ));
    let result = fixture.analyze();
    let core = result.core.unwrap();
    let Some(CoreValue::Int(value)) = core.functions[0].body.literal() else {
        panic!("expected exact integer");
    };
    assert_eq!(value.magnitude_bits(), MAX_INTEGER_BITS);
}

#[test]
fn integer_over_significant_bit_limit_is_rejected_without_core() {
    let cases = [
        ("binary", format!("0b1{}", "0".repeat(MAX_INTEGER_BITS))),
        (
            "hexadecimal",
            format!("0x1{}", "0".repeat(MAX_INTEGER_BITS / 4)),
        ),
        ("decimal", power_of_two_decimal(MAX_INTEGER_BITS)),
    ];

    for (radix, literal) in cases {
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec huge() -> Int {{ {literal} }} }}\n"
        ));
        let first = fixture.analyze();
        let second = fixture.analyze();

        assert_eq!(first, second, "{radix} rejection must be repeatable");
        assert!(first.core.is_none(), "{radix} must not produce Core");
        assert_eq!(first.diagnostics.len(), 1, "{radix} diagnostic count");
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::IntegerMagnitudeLimit,
            "{radix} diagnostic code"
        );
        assert_eq!(
            fixture.source().slice(first.diagnostics[0].primary_span()),
            Some(literal.as_str()),
            "{radix} diagnostic span"
        );
    }
}

#[test]
fn leading_zeroes_consume_no_significant_bit_or_event_budget() {
    let zeroes = "0".repeat(MAX_INTEGER_BITS + 1);
    let fixture = Fixture::new(format!(
        "edition 2026; module values {{ spec value() -> Word[8] {{ 0x{zeroes}2a }} }}\n"
    ));

    // Two namespace operations, two type-component inspections, one sign
    // inspection, one prefix inspection, two significant digits, and four
    // Core-node attempts. The leading zeroes consume neither bit budget nor
    // semantic events.
    let first = fixture.analyze_with(Limits {
        nodes: 4,
        events: 12,
        ..Limits::DEFAULT
    });
    let second = fixture.analyze_with(Limits {
        nodes: 4,
        events: 12,
        ..Limits::DEFAULT
    });
    assert_eq!(first, second);
    assert_eq!(first.diagnostics, []);
    assert_eq!(
        first.core.unwrap().functions[0].body.literal(),
        Some(&CoreValue::Word8(42))
    );

    let first = fixture.analyze_with(Limits {
        nodes: 4,
        events: 11,
        ..Limits::DEFAULT
    });
    let second = fixture.analyze_with(Limits {
        nodes: 4,
        events: 11,
        ..Limits::DEFAULT
    });
    assert_eq!(first, second);
    assert!(first.core.is_none());
    assert_eq!(
        first.diagnostics.last().unwrap().code(),
        DiagnosticCode::SemanticResourceLimit
    );

    let negative_zero = Fixture::new(format!(
        "edition 2026; module values {{ spec value() -> Int {{ -0x{zeroes} }} }}\n"
    ));
    let limits = Limits {
        nodes: 4,
        events: 9,
        ..Limits::DEFAULT
    };
    let first = negative_zero.analyze_with(limits);
    let second = negative_zero.analyze_with(limits);
    assert_eq!(first, second);
    assert_eq!(first.diagnostics, []);
    assert_eq!(
        first.core.unwrap().functions[0]
            .body
            .literal()
            .unwrap()
            .to_string(),
        "0"
    );

    let limits = Limits {
        nodes: 4,
        events: 8,
        ..Limits::DEFAULT
    };
    let first = negative_zero.analyze_with(limits);
    let second = negative_zero.analyze_with(limits);
    assert_eq!(first, second);
    assert!(first.core.is_none());
    assert_eq!(
        first.diagnostics.last().unwrap().code(),
        DiagnosticCode::SemanticResourceLimit
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
        .map(|function| function.body.literal().unwrap().to_string())
        .collect();
    assert_eq!(values, ["0x00", "0x01", "0xfe", "0xff"]);
}

#[test]
fn every_word8_value_decodes_exactly_in_every_radix() {
    let mut source = String::from("edition 2026; module word8_corpus {\n");
    let mut expected = Vec::with_capacity(256 * 3);
    for value in u8::MIN..=u8::MAX {
        source.push_str(&format!(
            "  spec decimal_{value}() -> Word[8] {{ {value} }}\n"
        ));
        expected.push(CoreValue::Word8(value));

        source.push_str(&format!(
            "  spec binary_{value}() -> Word[8] {{ 0b{value:b} }}\n"
        ));
        expected.push(CoreValue::Word8(value));

        source.push_str(&format!(
            "  spec hexadecimal_{value}() -> Word[8] {{ 0X{value:X} }}\n"
        ));
        expected.push(CoreValue::Word8(value));
    }
    source.push_str("}\n");

    let fixture = Fixture::new(source);
    let first = fixture.analyze();
    let second = fixture.analyze();
    assert_eq!(first, second);
    assert_eq!(first.diagnostics, []);
    let observed: Vec<_> = first
        .core
        .unwrap()
        .functions
        .iter()
        .map(|function| function.body.literal().unwrap().clone())
        .collect();
    assert_eq!(observed, expected);
}

#[test]
fn words_reject_negative_and_out_of_range_values_without_coercion() {
    let cases = [
        ("-0", DiagnosticCode::NegativeWordLiteral),
        ("-255", DiagnosticCode::NegativeWordLiteral),
        ("-256", DiagnosticCode::NegativeWordLiteral),
        (
            "-340282366920938463463374607431768211455",
            DiagnosticCode::NegativeWordLiteral,
        ),
        ("-0b0", DiagnosticCode::NegativeWordLiteral),
        ("-0b11111111", DiagnosticCode::NegativeWordLiteral),
        ("-0b100000000", DiagnosticCode::NegativeWordLiteral),
        (
            "-0b11111111111111111111111111111111111111111111111111111111111111111",
            DiagnosticCode::NegativeWordLiteral,
        ),
        ("-0x0", DiagnosticCode::NegativeWordLiteral),
        ("-0xff", DiagnosticCode::NegativeWordLiteral),
        ("-0x100", DiagnosticCode::NegativeWordLiteral),
        (
            "-0xffffffffffffffffffffffffffffffff",
            DiagnosticCode::NegativeWordLiteral,
        ),
        ("256", DiagnosticCode::WordLiteralOutOfRange),
        ("0b100000000", DiagnosticCode::WordLiteralOutOfRange),
        ("0x1_00", DiagnosticCode::WordLiteralOutOfRange),
    ];

    for (literal, code) in cases {
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec bad() -> Word[8] {{ {literal} }} }}\n"
        ));
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second, "{literal} rejection must be repeatable");
        assert!(first.core.is_none(), "{literal} must not produce Core");
        assert_eq!(first.diagnostics.len(), 1, "{literal} diagnostic count");
        assert_eq!(first.diagnostics[0].code(), code, "{literal} code");
        assert_eq!(
            fixture.source().slice(first.diagnostics[0].primary_span()),
            Some(literal),
            "{literal} diagnostic span"
        );
    }

    let over_bit_limit = format!("0x1{}", "0".repeat(MAX_INTEGER_BITS / 4));
    for literal in [over_bit_limit.clone(), format!("-{over_bit_limit}")] {
        let fixture = Fixture::new(format!(
            "edition 2026; module values {{ spec bad() -> Word[8] {{ {literal} }} }}\n"
        ));
        let result = fixture.analyze();
        assert!(result.core.is_none());
        assert_eq!(result.diagnostics.len(), 1, "{literal} diagnostic count");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::IntegerMagnitudeLimit,
            "{literal} must enforce the magnitude limit before word sign or range checks",
        );
        assert_eq!(
            fixture.source().slice(result.diagnostics[0].primary_span()),
            Some(over_bit_limit.as_str()),
            "{literal} magnitude diagnostic span",
        );
    }
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
    assert_eq!(
        duplicate
            .source()
            .slice(result.diagnostics[0].primary_span()),
        Some("same")
    );
    let [first_declaration] = result.diagnostics[0].secondary_spans() else {
        panic!("duplicate diagnostic must cite exactly one first declaration");
    };
    assert_eq!(
        duplicate.source().slice(first_declaration.span()),
        Some("same")
    );
    assert_eq!(first_declaration.label(), "first declaration is here");

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
    assert_eq!(
        typed_impl
            .source()
            .slice(result.diagnostics[0].primary_span()),
        Some("typed")
    );

    let cases = [
        (
            "spec typed() -> Integer { 1 }",
            DiagnosticCode::UnsupportedType,
            "Integer",
        ),
        (
            "spec typed() -> Word { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "Word",
        ),
        (
            "spec typed() -> Word[12] { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "12",
        ),
        (
            "spec typed() -> Word[128] { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "128",
        ),
        (
            "spec typed() -> Word[016] { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "016",
        ),
        (
            "spec typed(x: Word[7]) -> Word[8] { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "7",
        ),
        (
            "spec typed(x: Integer) -> Int { 1 }",
            DiagnosticCode::UnsupportedType,
            "Integer",
        ),
        (
            "spec typed() -> Word[08] { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "08",
        ),
        (
            "spec typed() -> Word[0x8] { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "0x8",
        ),
        (
            "spec typed() -> Word[1_0] { 1 }",
            DiagnosticCode::UnsupportedWordWidth,
            "1_0",
        ),
        (
            "spec typed() -> Int[8] { 1 }",
            DiagnosticCode::UnsupportedType,
            "Int[8]",
        ),
    ];
    for (declaration, code, responsible_source) in cases {
        let fixture = Fixture::new(format!("edition 2026; module values {{ {declaration} }}\n"));
        let first = fixture.analyze();
        let second = fixture.analyze();
        assert_eq!(first, second, "{declaration} rejection must be repeatable");
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1, "{declaration} diagnostic count");
        assert_eq!(first.diagnostics[0].code(), code, "{declaration} code");
        assert_eq!(
            fixture.source().slice(first.diagnostics[0].primary_span()),
            Some(responsible_source),
            "{declaration} responsible source span"
        );
    }
}

#[test]
fn independent_semantic_errors_preserve_source_order_and_responsible_spans() {
    let fixture = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec repeated() {}\n",
        "  spec repeated() {}\n",
        "  spec unsupported() -> Integer { 1 }\n",
        "  spec missing_width() -> Word { 1 }\n",
        "  spec bad_width() -> Word[12] { 1 }\n",
        "  spec negative() -> Word[8] { -1 }\n",
        "  spec out_of_range() -> Word[8] { 256 }\n",
        "}\n",
    ));

    let first = fixture.analyze();
    let second = fixture.analyze();
    assert_eq!(first, second);
    assert!(first.core.is_none());

    let expected = [
        (DiagnosticCode::DuplicateFunction, "repeated"),
        (DiagnosticCode::UnsupportedType, "Integer"),
        (DiagnosticCode::UnsupportedWordWidth, "Word"),
        (DiagnosticCode::UnsupportedWordWidth, "12"),
        (DiagnosticCode::NegativeWordLiteral, "-1"),
        (DiagnosticCode::WordLiteralOutOfRange, "256"),
    ];
    assert_eq!(first.diagnostics.len(), expected.len());
    for (diagnostic, (code, responsible_source)) in first.diagnostics.iter().zip(expected) {
        assert_eq!(diagnostic.code(), code);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some(responsible_source)
        );
    }

    let [first_declaration] = first.diagnostics[0].secondary_spans() else {
        panic!("duplicate diagnostic must cite exactly one first declaration");
    };
    assert_eq!(
        fixture.source().slice(first_declaration.span()),
        Some("repeated")
    );

    let rendered = crate::diagnostic::render_diagnostics(&fixture.sources, &first.diagnostics);
    let rendered_codes: Vec<_> = rendered
        .lines()
        .filter_map(|line| {
            line.strip_prefix("error[")
                .and_then(|rest| rest.split_once(']'))
                .map(|(code, _)| code)
        })
        .collect();
    assert_eq!(
        rendered_codes,
        [
            "ORC0201", "ORC0203", "ORC0204", "ORC0204", "ORC0206", "ORC0207"
        ]
    );
    assert_eq!(
        rendered,
        crate::diagnostic::render_diagnostics(&fixture.sources, &second.diagnostics)
    );
}

#[test]
fn compounded_declaration_failures_follow_semantic_traversal_order() {
    let fixture = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec repeated() {}\n",
        "  spec repeated() -> Word[12] { 1 }\n",
        "}\n",
    ));

    let first = fixture.analyze();
    let second = fixture.analyze();
    assert_eq!(first, second);
    assert!(first.core.is_none());
    assert_eq!(
        first
            .diagnostics
            .iter()
            .map(Diagnostic::code)
            .collect::<Vec<_>>(),
        [
            DiagnosticCode::DuplicateFunction,
            DiagnosticCode::UnsupportedWordWidth,
        ]
    );
    assert_eq!(
        fixture.source().slice(first.diagnostics[0].primary_span()),
        Some("repeated")
    );
    assert_eq!(
        fixture.source().slice(first.diagnostics[1].primary_span()),
        Some("12")
    );
    let [first_declaration] = first.diagnostics[0].secondary_spans() else {
        panic!("duplicate diagnostic must cite exactly one first declaration");
    };
    assert_eq!(
        fixture.source().slice(first_declaration.span()),
        Some("repeated")
    );
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
fn magnitude_limb_reservation_failure_returns_no_partial_core() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    let analyze_with_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_magnitude_limb = |_| false;
        analyzer.run()
    };

    let first = analyze_with_failure();
    let second = analyze_with_failure();
    assert_eq!(first, second);
    assert!(first.core().is_none());
    assert_eq!(first.diagnostics().len(), 1);
    let diagnostic = &first.diagnostics()[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(fixture.source().slice(diagnostic.primary_span()), Some("1"));
    assert_eq!(
        diagnostic.label(),
        "exact integer magnitude storage allocation failed"
    );
}

#[test]
fn index_range_reservation_failure_returns_no_partial_core() {
    let fixture = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec f(x: Word[8]^4) -> Word[8] { for i in 1..5 with s: Word[8] = 0 { s ^ x[i - 1] } }\n",
        "}\n",
    ));
    let analyze_with_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_range_limbs = |_, _| false;
        analyzer.run()
    };

    let first = analyze_with_failure();
    let second = analyze_with_failure();
    assert_eq!(first, second);
    assert!(first.core().is_none());
    assert_eq!(first.diagnostics().len(), 1);
    let diagnostic = &first.diagnostics()[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(fixture.source().slice(diagnostic.primary_span()), Some("i"));
    assert_eq!(diagnostic.label(), "index range storage allocation failed");
}

#[test]
fn pending_function_reservation_failure_returns_no_partial_core() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    let analyze_with_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_pending_function_slot = |_| false;
        analyzer.run()
    };

    let first = analyze_with_failure();
    let second = analyze_with_failure();
    assert_eq!(first, second);
    assert!(first.core().is_none());
    assert_eq!(first.diagnostics().len(), 1);
    let diagnostic = &first.diagnostics()[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(
        fixture.source().slice(diagnostic.primary_span()),
        Some("spec value() -> Int { 1 }")
    );
    assert_eq!(
        diagnostic.label(),
        "semantic analysis could not allocate pending function storage"
    );
}

#[test]
fn diagnostic_vector_reservation_failure_returns_no_core_or_diagnostics() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
    analyzer.reserve_diagnostic_slots = |_, _| false;

    let analyzed = analyzer.run();

    assert!(analyzed.has_errors());
    assert!(analyzed.core().is_none());
    assert!(analyzed.diagnostics().is_empty());
    assert_eq!(analyzed.diagnostics.capacity(), 0);
}

#[test]
fn core_name_reservation_failure_returns_no_partial_core() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    let analyze_with_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_core_name = |_, _| false;
        analyzer.run()
    };

    let first = analyze_with_failure();
    let second = analyze_with_failure();
    assert_eq!(first, second);
    assert!(first.core().is_none());
    assert_eq!(first.diagnostics().len(), 1);
    let diagnostic = &first.diagnostics()[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(
        fixture.source().slice(diagnostic.primary_span()),
        Some("value")
    );
    assert_eq!(
        diagnostic.label(),
        "typed Core name storage allocation failed"
    );
}

#[test]
fn late_allocation_failures_discard_completed_pending_core() {
    let magnitude_fixture = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec first() -> Word[8] { 0 }\n",
        "  spec second() -> Int { 2 }\n",
        "}\n",
    ));
    let mut magnitude_analyzer = Analyzer::new(
        magnitude_fixture.source(),
        &magnitude_fixture.ast,
        Limits::DEFAULT,
    );
    magnitude_analyzer.reserve_magnitude_limb = |_| false;
    let magnitude_failure = magnitude_analyzer.run();

    assert!(magnitude_failure.core().is_none());
    assert_eq!(magnitude_failure.diagnostics().len(), 1);
    assert_eq!(
        magnitude_fixture
            .source()
            .slice(magnitude_failure.diagnostics()[0].primary_span()),
        Some("2")
    );
    assert_eq!(
        magnitude_failure.diagnostics()[0].label(),
        "exact integer magnitude storage allocation failed"
    );

    let pending_fixture = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec first() -> Int { 1 }\n",
        "  spec second() -> Int { 2 }\n",
        "}\n",
    ));
    let mut pending_analyzer = Analyzer::new(
        pending_fixture.source(),
        &pending_fixture.ast,
        Limits::DEFAULT,
    );
    pending_analyzer.reserve_pending_function_slot =
        |functions| functions.is_empty() && functions.try_reserve(1).is_ok();
    let pending_failure = pending_analyzer.run();

    assert!(pending_failure.core().is_none());
    assert_eq!(pending_failure.diagnostics().len(), 1);
    assert_eq!(
        pending_fixture
            .source()
            .slice(pending_failure.diagnostics()[0].primary_span()),
        Some("spec second() -> Int { 2 }")
    );
    assert_eq!(
        pending_failure.diagnostics()[0].label(),
        "semantic analysis could not allocate pending function storage"
    );

    let module_fixture = Fixture::new(concat!(
        "edition 2026; module module_identifier {\n",
        "  spec a() -> Int { 1 }\n",
        "  spec bb() -> Word[8] { 2 }\n",
        "}\n",
    ));
    let mut module_analyzer = Analyzer::new(
        module_fixture.source(),
        &module_fixture.ast,
        Limits::DEFAULT,
    );
    module_analyzer.reserve_core_name =
        |name, bytes| bytes != "module_identifier".len() && name.try_reserve_exact(bytes).is_ok();
    let module_failure = module_analyzer.run();

    assert!(module_failure.core().is_none());
    assert_eq!(module_failure.diagnostics().len(), 1);
    assert_eq!(
        module_fixture
            .source()
            .slice(module_failure.diagnostics()[0].primary_span()),
        Some("module_identifier")
    );
    assert_eq!(
        module_failure.diagnostics()[0].label(),
        "typed Core name storage allocation failed"
    );
}

#[test]
fn declaration_namespace_reservation_failure_returns_no_partial_core() {
    let fixture = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec first() {}\n",
        "  impl second() {}\n",
        "}\n",
    ));
    let first = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
        .run_with_reservations(
            |_, capacity| {
                assert_eq!(capacity, 2);
                false
            },
            |functions, capacity| functions.try_reserve_exact(capacity).is_ok(),
        );
    let second = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
        .run_with_reservations(
            |_, _| false,
            |functions, capacity| functions.try_reserve_exact(capacity).is_ok(),
        );

    assert_eq!(first, second);
    assert!(first.core().is_none());
    assert_eq!(first.diagnostics().len(), 1);
    let diagnostic = &first.diagnostics()[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(diagnostic.primary_span(), fixture.ast.module.span);
    assert_eq!(
        diagnostic.label(),
        "semantic declaration namespace storage allocation failed"
    );
    assert_eq!(
        diagnostic.notes(),
        &["semantic analysis stopped without producing Core"]
    );
}

#[test]
fn long_identifier_diagnostics_have_bounded_messages() {
    let long_type = "N".repeat(1_024);
    let fixture = Fixture::new(format!(
        "edition 2026; module values {{ spec value() -> {long_type} {{ 1 }} }}\n"
    ));

    let first = fixture.analyze();
    let second = fixture.analyze();

    assert_eq!(first, second);
    assert!(first.core().is_none());
    assert_eq!(first.diagnostics().len(), 1);
    assert_eq!(
        first.diagnostics()[0].message(),
        format!(
            "unsupported result type `{}...<1024 bytes total>`",
            "N".repeat(MAX_IDENTIFIER_BYTES_IN_DIAGNOSTIC)
        )
    );
    assert!(first.diagnostics()[0].message().len() < 128);
}

#[test]
fn core_function_reservation_failure_returns_no_partial_core() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    let first = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
        .run_with_reservations(
            |declarations, capacity| declarations.try_reserve(capacity).is_ok(),
            |_, capacity| {
                assert_eq!(capacity, 1);
                false
            },
        );
    let second = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT)
        .run_with_reservations(
            |declarations, capacity| declarations.try_reserve(capacity).is_ok(),
            |_, _| false,
        );

    assert_eq!(first, second);
    assert!(first.core().is_none());
    assert_eq!(first.diagnostics().len(), 1);
    let diagnostic = &first.diagnostics()[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(diagnostic.primary_span(), fixture.ast.module.span);
    assert_eq!(
        diagnostic.message(),
        "semantic analysis resource limit exceeded"
    );
    assert_eq!(
        diagnostic.label(),
        "typed Core function storage allocation failed"
    );
    assert_eq!(
        diagnostic.notes(),
        &["semantic analysis stopped without producing Core"]
    );
}

#[test]
fn diagnostic_and_resource_limits_fail_closed() {
    let diagnostics = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec first() -> Nope { 1 }\n",
        "  spec second() -> Nope { 2 }\n",
        "}\n",
    ));
    let limits = Limits {
        diagnostics: 1,
        ..Limits::DEFAULT
    };
    let first = diagnostics.analyze_with(limits);
    let second = diagnostics.analyze_with(limits);
    assert_eq!(first, second);
    assert!(first.core.is_none());
    assert_eq!(
        first
            .diagnostics
            .iter()
            .map(Diagnostic::code)
            .collect::<Vec<_>>(),
        [
            DiagnosticCode::UnsupportedType,
            DiagnosticCode::TooManySemanticErrors
        ]
    );

    let diagnostic_attempt =
        Fixture::new("edition 2026; module values { spec value() -> Nope { 1 } }\n");
    let below_event_boundary = diagnostic_attempt.analyze_with(Limits {
        diagnostics: 0,
        events: 3,
        ..Limits::DEFAULT
    });
    assert_eq!(
        below_event_boundary
            .diagnostics
            .iter()
            .map(Diagnostic::code)
            .collect::<Vec<_>>(),
        [DiagnosticCode::SemanticResourceLimit]
    );
    let at_event_boundary = diagnostic_attempt.analyze_with(Limits {
        diagnostics: 0,
        events: 4,
        ..Limits::DEFAULT
    });
    assert_eq!(
        at_event_boundary
            .diagnostics
            .iter()
            .map(Diagnostic::code)
            .collect::<Vec<_>>(),
        [DiagnosticCode::TooManySemanticErrors]
    );

    let typed = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    for (limits, expected_span, expected_label) in [
        (
            Limits {
                events: 0,
                ..Limits::DEFAULT
            },
            typed.ast.module.functions[0].name.span,
            "semantic event budget exhausted",
        ),
        (
            Limits {
                nodes: 0,
                ..Limits::DEFAULT
            },
            typed.ast.module.span,
            "typed Core node budget exhausted",
        ),
    ] {
        let first = typed.analyze_with(limits);
        let second = typed.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        let diagnostic = first.diagnostics.last().unwrap();
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(diagnostic.primary_span(), expected_span);
        assert_eq!(
            diagnostic.message(),
            "semantic analysis resource limit exceeded"
        );
        assert_eq!(diagnostic.label(), expected_label);
        assert_eq!(
            diagnostic.notes(),
            &["semantic analysis stopped without producing Core"]
        );
    }

    let suppressed_then_exhausted = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec first() -> Nope { 1 }\n",
        "  spec second() -> Nope { 2 }\n",
        "}\n",
    ));
    // Each declaration consumes lookup, insertion, type-inspection, and
    // diagnostic-attempt events. The first attempt emits the suppression
    // record; the later suppressed attempt must still consume event eight.
    for (events, expected_codes) in [
        (
            7,
            &[
                DiagnosticCode::TooManySemanticErrors,
                DiagnosticCode::SemanticResourceLimit,
            ][..],
        ),
        (8, &[DiagnosticCode::TooManySemanticErrors][..]),
    ] {
        let limits = Limits {
            diagnostics: 0,
            events,
            ..Limits::DEFAULT
        };
        let first = suppressed_then_exhausted.analyze_with(limits);
        let second = suppressed_then_exhausted.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(
            first
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            expected_codes
        );
    }

    let compounded = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec repeated() {}\n",
        "  spec repeated() -> Word[12] { 1 }\n",
        "}\n",
    ));
    // The first declaration consumes lookup and insertion. The second
    // consumes lookup, duplicate-report, type-name, width, and
    // width-report events. At six events the final report attempt becomes
    // resource exhaustion; at seven it becomes diagnostic suppression.
    for (limits, expected_codes) in [
        (
            Limits {
                diagnostics: 1,
                events: 6,
                ..Limits::DEFAULT
            },
            &[
                DiagnosticCode::DuplicateFunction,
                DiagnosticCode::SemanticResourceLimit,
            ][..],
        ),
        (
            Limits {
                diagnostics: 1,
                events: 7,
                ..Limits::DEFAULT
            },
            &[
                DiagnosticCode::DuplicateFunction,
                DiagnosticCode::TooManySemanticErrors,
            ][..],
        ),
    ] {
        let first = compounded.analyze_with(limits);
        let second = compounded.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(
            first
                .diagnostics
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            expected_codes
        );
        assert_eq!(
            compounded
                .source()
                .slice(first.diagnostics[1].primary_span()),
            Some("12")
        );
    }
}

#[test]
fn exact_ordinary_diagnostic_budget_emits_no_suppression_record() {
    let two_errors = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec first() -> Nope { 1 }\n",
        "  spec second() -> Nope { 2 }\n",
        "}\n",
    ));
    let at_injected_limit = two_errors.analyze_with(Limits {
        diagnostics: 2,
        ..Limits::DEFAULT
    });
    assert!(at_injected_limit.core.is_none());
    assert_eq!(
        at_injected_limit
            .diagnostics
            .iter()
            .map(Diagnostic::code)
            .collect::<Vec<_>>(),
        [
            DiagnosticCode::UnsupportedType,
            DiagnosticCode::UnsupportedType
        ]
    );

    let mut text = String::from("edition 2026; module values {\n");
    for index in 0..MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE {
        text.push_str(&format!("  spec bad{index}() -> Nope {{ {index} }}\n"));
    }
    let mut exceeded_text = text.clone();
    text.push_str("}\n");
    exceeded_text.push_str("  spec overflow() -> Nope { 0 }\n}\n");

    let at_limit = Fixture::new(text);
    let first = at_limit.analyze_with(Limits::DEFAULT);
    let second = at_limit.analyze_with(Limits::DEFAULT);
    assert_eq!(first, second);
    assert!(first.core.is_none());
    assert_eq!(first.diagnostics.len(), MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE);
    assert!(
        first
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code() == DiagnosticCode::UnsupportedType)
    );

    let exceeded = Fixture::new(exceeded_text).analyze_with(Limits::DEFAULT);
    assert!(exceeded.core.is_none());
    assert_eq!(
        exceeded.diagnostics.len(),
        MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE + 1
    );
    let shape = |diagnostic: &Diagnostic| {
        (
            diagnostic.code(),
            diagnostic.primary_span().start().bytes(),
            diagnostic.primary_span().end().bytes(),
        )
    };
    assert_eq!(
        exceeded.diagnostics[..MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE]
            .iter()
            .map(shape)
            .collect::<Vec<_>>(),
        first.diagnostics.iter().map(shape).collect::<Vec<_>>()
    );
    assert_eq!(
        exceeded.diagnostics.last().unwrap().code(),
        DiagnosticCode::TooManySemanticErrors
    );
}

#[test]
fn complete_diagnostic_bound_requires_no_capacity_growth() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
    assert!((analyzer.reserve_diagnostic_slots)(
        &mut analyzer.diagnostics,
        MAX_RETAINED_SEMANTIC_DIAGNOSTICS,
    ));
    let initial_capacity = analyzer.diagnostics.capacity();
    let span = fixture.ast.module.functions[0].name.span;

    for _ in 0..=MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE {
        if analyzer.begin_report(span) {
            analyzer.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnsupportedType,
                "synthetic semantic error",
                span,
            ));
        }
    }
    analyzer.resource_limit(span, "synthetic resource failure");

    assert_eq!(
        analyzer.diagnostics.len(),
        MAX_RETAINED_SEMANTIC_DIAGNOSTICS
    );
    assert_eq!(analyzer.diagnostics.capacity(), initial_capacity);
}

#[test]
fn suppressed_semantic_diagnostics_are_not_constructed() {
    let fixture = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    let mut analyzer = Analyzer::new(
        fixture.source(),
        &fixture.ast,
        Limits {
            diagnostics: 0,
            ..Limits::DEFAULT
        },
    );
    analyzer.diagnostics.try_reserve_exact(1).unwrap();
    let span = fixture.ast.module.functions[0].name.span;
    let constructed = std::cell::Cell::new(0_usize);

    for _ in 0..2 {
        if analyzer.begin_report(span) {
            constructed.set(constructed.get().saturating_add(1));
            analyzer.diagnostics.push(Diagnostic::error(
                DiagnosticCode::UnsupportedType,
                "unused",
                span,
            ));
        }
    }

    assert_eq!(constructed.get(), 0);
    assert_eq!(analyzer.diagnostics.len(), 1);
    assert_eq!(
        analyzer.diagnostics[0].code(),
        DiagnosticCode::TooManySemanticErrors
    );
}

#[test]
fn injected_limits_match_normative_event_and_core_node_accounting() {
    let empty = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec empty() {}\n",
        "  impl also_empty() {}\n",
        "}\n",
    ));
    let limits = Limits {
        nodes: 1,
        events: 5,
        ..Limits::DEFAULT
    };
    let first = empty.analyze_with(limits);
    let second = empty.analyze_with(limits);
    assert_eq!(first, second);
    assert!(first.core.unwrap().functions.is_empty());

    let duplicate = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec same() {}\n",
        "  spec same() {}\n",
        "}\n",
    ));
    let first = duplicate.analyze_with(Limits {
        events: 4,
        ..Limits::DEFAULT
    });
    let second = duplicate.analyze_with(Limits {
        events: 4,
        ..Limits::DEFAULT
    });
    assert_eq!(first, second);
    assert_eq!(first.diagnostics.len(), 1);
    assert_eq!(
        first.diagnostics[0].code(),
        DiagnosticCode::DuplicateFunction
    );
    let first = duplicate.analyze_with(Limits {
        events: 3,
        ..Limits::DEFAULT
    });
    let second = duplicate.analyze_with(Limits {
        events: 3,
        ..Limits::DEFAULT
    });
    assert_eq!(first, second);
    assert_eq!(first.diagnostics.len(), 1);
    assert_eq!(
        first.diagnostics[0].code(),
        DiagnosticCode::SemanticResourceLimit
    );
    assert_eq!(
        first.diagnostics[0].label(),
        "semantic event budget exhausted"
    );

    let typed = Fixture::new("edition 2026; module values { spec value() -> Int { 1 } }\n");
    // Two namespace operations, one type inspection, sign/prefix/digit
    // inspection, and four Core-node attempts total ten events. The Core is
    // one module plus one function, type, and value node.
    for (limits, expected_label) in [
        (
            Limits {
                nodes: 3,
                ..Limits::DEFAULT
            },
            "typed Core node budget exhausted",
        ),
        (
            Limits {
                events: 9,
                ..Limits::DEFAULT
            },
            "semantic event budget exhausted",
        ),
    ] {
        let first = typed.analyze_with(limits);
        let second = typed.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        let diagnostic = first.diagnostics.last().unwrap();
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            diagnostic.primary_span(),
            typed.ast.module.functions[0].span
        );
        assert_eq!(diagnostic.label(), expected_label);
    }
    let limits = Limits {
        nodes: 4,
        events: 10,
        ..Limits::DEFAULT
    };
    let first = typed.analyze_with(limits);
    let second = typed.analyze_with(limits);
    assert_eq!(first, second);
    assert!(first.core.is_some());
    assert_eq!(first.diagnostics, []);

    let mixed = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec empty() {}\n",
        "  spec first() -> Int { 1 }\n",
        "  impl also_empty() {}\n",
        "  spec second() -> Word[8] { 2 }\n",
        "}\n",
    ));
    // Eight namespace operations, three type-component inspections, six
    // literal inspections, and seven Core-node attempts total 24 events.
    // The Core is one module plus three nodes for each typed specification;
    // the two empty declarations add no Core nodes.
    for (limits, expected_label) in [
        (
            Limits {
                nodes: 6,
                events: 24,
                ..Limits::DEFAULT
            },
            "typed Core node budget exhausted",
        ),
        (
            Limits {
                nodes: 7,
                events: 23,
                ..Limits::DEFAULT
            },
            "semantic event budget exhausted",
        ),
    ] {
        let first = mixed.analyze_with(limits);
        let second = mixed.analyze_with(limits);
        assert_eq!(first, second);
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::SemanticResourceLimit
        );
        assert_eq!(first.diagnostics[0].label(), expected_label);
        assert_eq!(
            first.diagnostics[0].primary_span(),
            mixed.ast.module.functions[3].span
        );
    }
    let first = mixed.analyze_with(Limits {
        nodes: 7,
        events: 24,
        ..Limits::DEFAULT
    });
    let second = mixed.analyze_with(Limits {
        nodes: 7,
        events: 24,
        ..Limits::DEFAULT
    });
    assert_eq!(first, second);
    assert_eq!(first.diagnostics, []);
    let core = first.core.unwrap();
    assert_eq!(core.functions.len(), 2);
    assert_eq!(core.functions[0].name, "first");
    assert_eq!(core.functions[1].name, "second");
}

#[test]
fn analysis_is_repeatable_for_typed_success_and_failure() {
    let accepted = Fixture::new(concat!(
        "edition 2026; module values {\n",
        "  spec integer() -> Int { -42 }\n",
        "  spec word() -> Word[8] { 42 }\n",
        "}\n",
    ));
    let first = accepted.analyze();
    let second = accepted.analyze();
    assert_eq!(first, second);
    assert_eq!(first.core.unwrap().functions.len(), 2);

    let rejected =
        Fixture::new("edition 2026; module values { spec value() -> lowercase { 1 } }\n");
    let first = rejected.analyze();
    let second = rejected.analyze();
    assert_eq!(first, second);
    assert!(first.core.is_none());
    assert_eq!(first.diagnostics[0].code(), DiagnosticCode::UnsupportedType);

    let first = accepted.analyze_with(Limits {
        events: 0,
        ..Limits::DEFAULT
    });
    let second = accepted.analyze_with(Limits {
        events: 0,
        ..Limits::DEFAULT
    });
    assert_eq!(first, second);
    assert!(first.core.is_none());
    assert_eq!(
        first.diagnostics[0].code(),
        DiagnosticCode::SemanticResourceLimit
    );
}

#[test]
fn mutated_s3a_sources_preserve_phase_gates_and_repeatability() {
    let base = concat!(
        "edition 2026; module mutation_seed {\n",
        "  spec empty() {}\n",
        "  impl empty() {}\n",
        "  spec integer() -> Int { -42 }\n",
        "  spec word() -> Word[8] { 0xff }\n",
        "}\n",
    );
    let characters: Vec<_> = base
        .char_indices()
        .map(|(start, character)| (start, start + character.len_utf8()))
        .collect();
    let mut corpus = std::collections::BTreeSet::new();
    corpus.insert(base.to_owned());

    for &(start, end) in &characters {
        corpus.insert(format!("{}{}", &base[..start], &base[end..]));
        for replacement in ["@", "0", "_", "{", "}", "\"", "-", "\r", "é"] {
            corpus.insert(format!("{}{}{}", &base[..start], replacement, &base[end..]));
        }
    }
    for offset in characters
        .iter()
        .map(|(start, _)| *start)
        .chain(std::iter::once(base.len()))
    {
        for insertion in [
            "@", "0", "_", "{", "}", "\"", "-", "\r", "é", "/*", "*/", "//", "\0",
        ] {
            corpus.insert(format!(
                "{}{}{}",
                &base[..offset],
                insertion,
                &base[offset..]
            ));
        }
    }

    let fragments = [
        "edition", "2026", ";", "module", "spec", "impl", "Int", "Word", "[", "]", "(", ")", "{",
        "}", "-", "0", "1", "256", "name", " ", "\n", "\r\n", "//x\n", "/*x*/", "@", "\"", "é",
    ];
    let mut state = 0x6a09_e667_f3bc_c908_u64;
    for _ in 0..512 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let fragment_count = usize::try_from(state % 48 + 1).unwrap();
        let mut text = String::new();
        for _ in 0..fragment_count {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let index = usize::try_from(state % fragments.len() as u64).unwrap();
            text.push_str(fragments[index]);
        }
        corpus.insert(text);
    }
    assert!(corpus.len() > 2_500, "mutation corpus unexpectedly shrank");

    let total_cases = corpus.len();
    let mut lexical_failures = 0_usize;
    let mut parser_failures = 0_usize;
    let mut semantic_failures = 0_usize;
    let mut evaluated_successes = 0_usize;
    for (case_index, text) in corpus.into_iter().enumerate() {
        let mut sources = SourceMap::new();
        let id = sources.add("mutation.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let assert_owned_spans = |diagnostics: &[Diagnostic]| {
            for diagnostic in diagnostics {
                assert_eq!(
                    diagnostic.primary_span().source(),
                    source.id(),
                    "case {case_index}"
                );
                assert!(
                    source.slice(diagnostic.primary_span()).is_some(),
                    "case {case_index} has an invalid primary span"
                );
                for secondary in diagnostic.secondary_spans() {
                    assert_eq!(secondary.span().source(), source.id(), "case {case_index}");
                    assert!(
                        source.slice(secondary.span()).is_some(),
                        "case {case_index} has an invalid secondary span"
                    );
                }
            }
        };
        let assert_repeatable_rendering =
            |first: &[Diagnostic], second: &[Diagnostic], phase: &str| {
                assert_eq!(
                    crate::diagnostic::render_diagnostics(&sources, first),
                    crate::diagnostic::render_diagnostics(&sources, second),
                    "case {case_index} {phase} diagnostic rendering"
                );
            };

        let first_lexed = lex(source, Edition::E2026);
        let second_lexed = lex(source, Edition::E2026);
        assert_eq!(first_lexed, second_lexed, "case {case_index} lexing");
        assert_owned_spans(first_lexed.diagnostics());
        assert_repeatable_rendering(
            first_lexed.diagnostics(),
            second_lexed.diagnostics(),
            "lexical",
        );
        if first_lexed.has_errors() {
            lexical_failures += 1;
            continue;
        }

        let first_parsed = parse(source, &first_lexed);
        let second_parsed = parse(source, &second_lexed);
        assert_eq!(first_parsed, second_parsed, "case {case_index} parsing");
        assert_owned_spans(first_parsed.diagnostics());
        assert_repeatable_rendering(
            first_parsed.diagnostics(),
            second_parsed.diagnostics(),
            "parser",
        );
        assert_eq!(
            first_parsed.ast().is_some(),
            first_parsed.diagnostics().is_empty(),
            "case {case_index} parser atomicity"
        );
        let Some(ast) = first_parsed.ast() else {
            parser_failures += 1;
            continue;
        };

        let first_analyzed = analyze(source, ast);
        let second_analyzed = analyze(source, ast);
        assert_eq!(
            first_analyzed, second_analyzed,
            "case {case_index} analysis"
        );
        assert_owned_spans(first_analyzed.diagnostics());
        assert_repeatable_rendering(
            first_analyzed.diagnostics(),
            second_analyzed.diagnostics(),
            "semantic",
        );
        assert_eq!(
            first_analyzed.core().is_some(),
            first_analyzed.diagnostics().is_empty(),
            "case {case_index} semantic atomicity"
        );
        let Some(core) = first_analyzed.core() else {
            semantic_failures += 1;
            continue;
        };

        let first_evaluated = crate::eval::evaluate(core);
        let second_evaluated = crate::eval::evaluate(core);
        assert_eq!(
            first_evaluated, second_evaluated,
            "case {case_index} evaluation"
        );
        assert_owned_spans(first_evaluated.diagnostics());
        assert_repeatable_rendering(
            first_evaluated.diagnostics(),
            second_evaluated.diagnostics(),
            "evaluation",
        );
        assert_eq!(
            first_evaluated.values().is_some(),
            first_evaluated.diagnostics().is_empty(),
            "case {case_index} evaluation atomicity"
        );
        assert!(
            first_evaluated.values().is_some(),
            "case {case_index} unexpectedly exhausted evaluation resources"
        );
        evaluated_successes += 1;
    }
    assert!(
        lexical_failures != 0,
        "mutation corpus missed lexical failure"
    );
    assert!(
        parser_failures != 0,
        "mutation corpus missed parser failure"
    );
    assert!(
        semantic_failures != 0,
        "mutation corpus missed semantic failure"
    );
    assert!(
        evaluated_successes != 0,
        "mutation corpus missed successful evaluation"
    );
    assert_eq!(
        lexical_failures + parser_failures + semantic_failures + evaluated_successes,
        total_cases,
        "mutation outcome partition drifted"
    );
}

#[test]
fn analysis_is_repeatable_and_empty_modules_produce_empty_core() {
    let fixture = Fixture::new("edition 2026; module values { spec empty() {} impl empty() {} }\n");
    let first = fixture.analyze();
    let second = fixture.analyze();
    assert_eq!(first, second);
    assert!(first.core.unwrap().functions.is_empty());
}

/// Renders one function's postorder body Core as `(operation, source, type)`.
fn core_nodes<'text>(
    fixture: &'text Fixture,
    function: &CoreFunction,
) -> Vec<(String, &'text str, CoreType)> {
    expression_nodes(fixture, &function.body)
}

/// Renders one postorder Core expression as `(operation, source, type)`.
fn expression_nodes<'text>(
    fixture: &'text Fixture,
    expression: &CoreExpression,
) -> Vec<(String, &'text str, CoreType)> {
    expression
        .nodes
        .iter()
        .map(|node| {
            let operation = match &node.kind {
                CoreNodeKind::Literal(value) => format!("literal {value}"),
                CoreNodeKind::Parameter(index) => format!("parameter {index}"),
                CoreNodeKind::Local(index) => format!("local {index}"),
                CoreNodeKind::Convert { from } => format!("convert from {from}"),
                CoreNodeKind::Call {
                    function,
                    arguments,
                } => format!("call #{} with {arguments}", function.index()),
                CoreNodeKind::Unary(operator) => format!("prefix {}", operator.as_str()),
                CoreNodeKind::Binary(operator) => format!("infix {}", operator.as_str()),
                CoreNodeKind::Shift { operator, amount } => {
                    format!("shift {} {amount}", operator.as_str())
                }
                CoreNodeKind::Array { elements } => format!("array of {elements}"),
                CoreNodeKind::Index { index } => format!("index {index}"),
                CoreNodeKind::Select => String::from("select"),
                CoreNodeKind::Update => String::from("update"),
                CoreNodeKind::Fill => String::from("fill"),
                CoreNodeKind::Fold(id) => format!("loop #{id}"),
                CoreNodeKind::LoopIndex(id) => format!("index of loop #{id}"),
                CoreNodeKind::Accumulator(id) => format!("accumulator of loop #{id}"),
                CoreNodeKind::StepBinding { loop_id, index } => {
                    format!("binding {index} of loop #{loop_id}")
                }
                CoreNodeKind::BranchBinding { conditional, index } => {
                    format!("binding {index} of branch #{conditional}")
                }
                CoreNodeKind::Compare { operator, operand } => {
                    format!("compare {} on {operand}", operator.as_str())
                }
                CoreNodeKind::Choose(id) => format!("choose #{id}"),
                CoreNodeKind::Tuple { elements } => format!("tuple of {elements}"),
                CoreNodeKind::Project { index } => format!("element {index}"),
                CoreNodeKind::Concat => String::from("concat"),
                CoreNodeKind::Slice => String::from("slice"),
                CoreNodeKind::SliceUpdate => String::from("slice update"),
            };
            (
                operation,
                fixture.source().slice(node.span).unwrap(),
                node.ty.clone(),
            )
        })
        .collect()
}

/// Renders diagnostics as `(code, responsible source, message)`.
fn reported<'text>(
    fixture: &'text Fixture,
    result: &AnalysisResult,
) -> Vec<(DiagnosticCode, &'text str, String)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code(),
                fixture.source().slice(diagnostic.primary_span()).unwrap(),
                diagnostic.message().to_owned(),
            )
        })
        .collect()
}

const TYPES: [CoreType; 5] = [
    CoreType::Int,
    CoreType::Word8,
    CoreType::Word16,
    CoreType::Word32,
    CoreType::Word64,
];

fn module(members: &str) -> Fixture {
    Fixture::new(format!("edition 2026; module m {{\n{members}}}\n"))
}

fn accepted(members: &str) -> (Fixture, CoreModule) {
    let fixture = module(members);
    let result = fixture.analyze();
    assert_eq!(result.diagnostics, [], "{members}");
    let core = result.core.unwrap();
    (fixture, core)
}

fn rejected(members: &str) -> (Fixture, AnalysisResult) {
    let fixture = module(members);
    let first = fixture.analyze();
    assert_eq!(first, fixture.analyze(), "{members}");
    assert!(first.core.is_none(), "{members}");
    (fixture, first)
}

#[test]
fn typed_core_is_postorder_with_exact_types_spans_and_operations() {
    let (fixture, core) = accepted(concat!(
        "  spec mix(x: Word[32], y: Word[32]) -> Word[32] { (x ^ ~y) <<< 7 }\n",
        "  spec use_mix() -> Word[32] { mix(1, 0xff) + 2 * 3 }\n",
        "  spec exact(n: Int) -> Int { -(n - -5) * n }\n",
    ));
    assert_eq!(
        core.functions
            .iter()
            .map(|function| (
                function.id.index(),
                function.name.as_str(),
                function.parameters.clone(),
                function.result_type.clone()
            ))
            .collect::<Vec<_>>(),
        [
            (
                0,
                "mix",
                vec![CoreType::Word32, CoreType::Word32],
                CoreType::Word32
            ),
            (1, "use_mix", vec![], CoreType::Word32),
            (2, "exact", vec![CoreType::Int], CoreType::Int),
        ]
    );
    let word = CoreType::Word32;
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        core_nodes(&fixture, &core.functions[0]),
        owned(&[
            ("parameter 0", "x", word.clone()),
            ("parameter 1", "y", word.clone()),
            ("prefix ~", "~y", word.clone()),
            ("infix ^", "x ^ ~y", word.clone()),
            ("shift <<< 7", "(x ^ ~y) <<< 7", word.clone()),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[1]),
        owned(&[
            ("literal 0x00000001", "1", word.clone()),
            ("literal 0x000000ff", "0xff", word.clone()),
            ("call #0 with 2", "mix(1, 0xff)", word.clone()),
            ("literal 0x00000002", "2", word.clone()),
            ("literal 0x00000003", "3", word.clone()),
            ("infix *", "2 * 3", word.clone()),
            ("infix +", "mix(1, 0xff) + 2 * 3", word),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[2]),
        owned(&[
            ("parameter 0", "n", CoreType::Int),
            ("literal -5", "-5", CoreType::Int),
            ("infix -", "n - -5", CoreType::Int),
            ("prefix -", "-(n - -5)", CoreType::Int),
            ("parameter 0", "n", CoreType::Int),
            ("infix *", "-(n - -5) * n", CoreType::Int),
        ])
    );
    assert_eq!(
        core.functions[0].body.root().map(|node| node.kind.clone()),
        Some(CoreNodeKind::Shift {
            operator: BinaryOperator::RotateLeft,
            amount: 7
        })
    );
    assert_eq!(core.functions[0].body.literal(), None);
}

#[test]
fn calls_resolve_in_any_order_and_acyclic_graphs_are_accepted() {
    let (fixture, core) = accepted(concat!(
        "  spec top() -> Int { left() + right() + left() }\n",
        "  spec left() -> Int { base(1) }\n",
        "  spec right() -> Int { base(2) * base(3) }\n",
        "  spec base(x: Int) -> Int { x * x }\n",
        "  spec unused() {}\n",
        "  impl unused() {}\n",
        "  spec after() -> Int { top() }\n",
    ));
    let calls = |index: usize| {
        core_nodes(&fixture, &core.functions[index])
            .into_iter()
            .filter(|(operation, _, _)| operation.starts_with("call"))
            .map(|(operation, source, _)| (operation, source))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        calls(0),
        [
            (String::from("call #1 with 0"), "left()"),
            (String::from("call #2 with 0"), "right()"),
            (String::from("call #1 with 0"), "left()"),
        ]
    );
    assert_eq!(
        calls(2),
        [
            (String::from("call #3 with 1"), "base(2)"),
            (String::from("call #3 with 1"), "base(3)"),
        ]
    );
    assert_eq!(calls(4), [(String::from("call #0 with 0"), "top()")]);
    assert_eq!(core.functions.len(), 5);
}

#[test]
fn call_cycles_are_reported_once_at_the_closing_call() {
    let (fixture, result) = rejected(concat!(
        "  spec itself() -> Int { itself() + 1 }\n",
        "  spec ping() -> Int { pong() }\n",
        "  spec pong() -> Int { ping() }\n",
        "  spec a() -> Int { b() }\n",
        "  spec b() -> Int { c() }\n",
        "  spec c() -> Int { a() + a() }\n",
        "  spec into_cycle() -> Int { ping() }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::CallCycle,
                "itself()",
                String::from("`itself` calls itself")
            ),
            (
                DiagnosticCode::CallCycle,
                "ping()",
                String::from("call cycle `ping` -> `pong` -> `ping`")
            ),
            (
                DiagnosticCode::CallCycle,
                "a()",
                String::from("call cycle `a` -> `b` -> `c` -> `a`")
            ),
            (
                DiagnosticCode::CallCycle,
                "a()",
                String::from("call cycle `a` -> `b` -> `c` -> `a`")
            ),
        ]
    );
    let spans = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.primary_span())
        .collect::<Vec<_>>();
    assert_ne!(spans[2], spans[3], "each closing call is its own site");
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.label(), "this call closes the cycle");
    assert_eq!(
        diagnostic.notes(),
        ["a `spec` may not depend on itself; recursion is not part of Orange 2026"]
    );
}

#[test]
fn call_cycles_are_reported_through_calls_with_other_errors() {
    let (fixture, result) = rejected(concat!(
        "  spec a() -> Int { b() }\n",
        "  spec b() -> Word[8] { a() }\n",
        "  spec count() -> Int { arity(1) }\n",
        "  spec arity() -> Int { count() }\n",
        "  spec wide(x: Word[12]) -> Int { narrow() }\n",
        "  spec narrow() -> Int { wide(1) }\n",
        "  spec typed(x: Word[8]) -> Int { typed(256) }\n",
    ));
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .filter(|(code, _, _)| *code == DiagnosticCode::CallCycle)
            .collect::<Vec<_>>(),
        [
            (
                DiagnosticCode::CallCycle,
                "a()",
                String::from("call cycle `a` -> `b` -> `a`")
            ),
            (
                DiagnosticCode::CallCycle,
                "count()",
                String::from("call cycle `count` -> `arity` -> `count`")
            ),
            (
                DiagnosticCode::CallCycle,
                "wide(1)",
                String::from("call cycle `wide` -> `narrow` -> `wide`")
            ),
            (
                DiagnosticCode::CallCycle,
                "typed(256)",
                String::from("`typed` calls itself")
            ),
        ]
    );
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .map(|(code, source, _)| (code, source))
            .filter(|(code, _)| *code != DiagnosticCode::CallCycle)
            .collect::<Vec<_>>(),
        [
            (DiagnosticCode::TypeMismatch, "b()"),
            (DiagnosticCode::TypeMismatch, "a()"),
            (DiagnosticCode::ArgumentCountMismatch, "arity(1)"),
            (DiagnosticCode::UnsupportedWordWidth, "12"),
            (DiagnosticCode::WordLiteralOutOfRange, "256"),
        ]
    );
}

#[test]
fn long_call_cycles_have_bounded_messages() {
    let count = MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC + 2;
    let members = (0..count)
        .map(|index| {
            format!(
                "  spec f{index}() -> Int {{ f{}() }}\n",
                (index + 1) % count
            )
        })
        .collect::<String>();
    let (fixture, result) = rejected(&members);
    let route = (0..MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC)
        .map(|index| format!("`f{index}`"))
        .collect::<Vec<_>>()
        .join(" -> ");
    assert_eq!(
        reported(&fixture, &result),
        [(
            DiagnosticCode::CallCycle,
            "f0()",
            format!("call cycle {route} -> ... -> `f0`")
        )]
    );
}

#[test]
fn names_and_calls_resolve_only_to_parameters_and_typed_specs() {
    let (fixture, result) = rejected(concat!(
        "  spec f(x: Int) -> Int { y }\n",
        "  spec g(x: Int) -> Int { helper }\n",
        "  spec helper() -> Int { 1 }\n",
        "  spec h() -> Int { missing() }\n",
        "  spec i() -> Int { legacy() }\n",
        "  spec legacy() {}\n",
        "  spec j() -> Int { body() }\n",
        "  impl body() {}\n",
        "  spec k(x: Int, x: Int) -> Int { x }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UnknownParameter,
                "y",
                String::from("`y` is not a parameter of `f`")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "helper",
                String::from("`helper` is not a parameter of `g`")
            ),
            (
                DiagnosticCode::UnknownFunction,
                "missing",
                String::from("no typed `spec` function named `missing` in this module")
            ),
            (
                DiagnosticCode::UnknownFunction,
                "legacy",
                String::from("`spec` function `legacy` has no typed body and cannot be called")
            ),
            (
                DiagnosticCode::UnknownFunction,
                "body",
                String::from("no typed `spec` function named `body` in this module")
            ),
            (
                DiagnosticCode::DuplicateParameter,
                "x",
                String::from("duplicate parameter `x`")
            ),
        ]
    );
    let notes = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.notes()[0].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        notes,
        [
            "a bare name in a `spec` body refers to one of its parameters",
            "to call the function `helper`, write `helper()` with its arguments",
            "calls name a typed `spec` declared in the same module",
            "calls name a typed `spec` declared in the same module",
            "`impl` functions have no semantics yet and cannot be called",
            "parameter names must be unique within one function",
        ]
    );
    let [declared] = result.diagnostics[3].secondary_spans() else {
        panic!("an untyped callee must cite its declaration");
    };
    assert_eq!(fixture.source().slice(declared.span()), Some("legacy"));
    let [first] = result.diagnostics[5].secondary_spans() else {
        panic!("a duplicate parameter must cite the first parameter");
    };
    assert_eq!(declared.label(), "declared without a result type here");
    assert_eq!(first.label(), "first parameter is here");
    assert!(first.span().start() < result.diagnostics[5].primary_span().start());
}

#[test]
fn calls_check_arity_argument_types_and_result_types() {
    let (fixture, result) = rejected(concat!(
        "  spec one(x: Word[8]) -> Word[8] { x }\n",
        "  spec two(x: Int, y: Int) -> Int { x + y }\n",
        "  spec int() -> Int { 1 }\n",
        "  spec a() -> Word[8] { one() }\n",
        "  spec b() -> Word[8] { one(1, 2) }\n",
        "  spec c() -> Int { two(1) }\n",
        "  spec d() -> Word[8] { one(256) }\n",
        "  spec e() -> Word[8] { int() }\n",
        "  spec f(w: Word[16]) -> Word[8] { one(w) }\n",
        "  spec g() -> Int { two(1, one(2)) }\n",
        "  spec h() -> Int { one(256) }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::ArgumentCountMismatch,
                "one()",
                String::from("`one` takes 1 argument but 0 were supplied")
            ),
            (
                DiagnosticCode::ArgumentCountMismatch,
                "one(1, 2)",
                String::from("`one` takes 1 argument but 2 were supplied")
            ),
            (
                DiagnosticCode::ArgumentCountMismatch,
                "two(1)",
                String::from("`two` takes 2 arguments but 1 was supplied")
            ),
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "256",
                String::from("literal is outside the range of `Word[8]`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "int()",
                String::from("`int` returns `Int`, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "w",
                String::from("`w` has type `Word[16]`, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "one(2)",
                String::from("`one` returns `Word[8]`, but `Int` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "one(256)",
                String::from("`one` returns `Word[8]`, but `Int` is required here")
            ),
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "256",
                String::from("literal is outside the range of `Word[8]`")
            ),
        ]
    );
    assert!(result.diagnostics.iter().all(|diagnostic| {
        diagnostic.code() != DiagnosticCode::TypeMismatch
            || diagnostic.notes() == ["Orange has no implicit conversions between types"]
    }));
}

#[test]
fn operators_are_defined_only_for_their_types() {
    let (fixture, result) = rejected(concat!(
        "  spec a(x: Word[8]) -> Word[8] { -x }\n",
        "  spec b(n: Int) -> Int { ~n }\n",
        "  spec c(n: Int) -> Int { n & 1 }\n",
        "  spec d(n: Int) -> Int { n | 1 }\n",
        "  spec e(n: Int) -> Int { n ^ 1 }\n",
        "  spec f(n: Int) -> Int { n << 1 }\n",
        "  spec g(n: Int) -> Int { n >>> 1 }\n",
        "  spec h(x: Word[64]) -> Word[64] { -(x + unknown) }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UnsupportedOperator,
                "-",
                String::from("prefix `-` is not defined for `Word[8]`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "~",
                String::from("prefix `~` is not defined for `Int`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "&",
                String::from("`&` is not defined for `Int`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "|",
                String::from("`|` is not defined for `Int`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "^",
                String::from("`^` is not defined for `Int`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "<<",
                String::from("`<<` is not defined for `Int`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                ">>>",
                String::from("`>>>` is not defined for `Int`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "-",
                String::from("prefix `-` is not defined for `Word[64]`")
            ),
        ]
    );
    let notes = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.notes()[0].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        notes,
        [
            "write `0 - x` for negation modulo 2^8",
            "bitwise operators apply only to `Word[n]` values",
            "bitwise operators apply only to `Word[n]` values",
            "bitwise operators apply only to `Word[n]` values",
            "bitwise operators apply only to `Word[n]` values",
            "shifts and rotations apply only to `Word[n]` values",
            "shifts and rotations apply only to `Word[n]` values",
            "write `0 - x` for negation modulo 2^64",
        ]
    );

    // Every arithmetic operator is defined for every type.
    let members = TYPES
        .iter()
        .enumerate()
        .map(|(index, ty)| {
            format!("  spec f{index}(x: {ty}, y: {ty}) -> {ty} {{ x + y - x * y }}\n")
        })
        .collect::<String>();
    accepted(&members);
}

#[test]
fn shift_and_rotation_amounts_are_literals_below_the_width() {
    let mut members = String::new();
    let mut expected = Vec::new();
    for ty in TYPES.iter().filter(|ty| **ty != CoreType::Int) {
        let bits = ty.word_bits().unwrap();
        for operator in ["<<", ">>", "<<<", ">>>"] {
            members.push_str(&format!(
                "  spec ok{bits}_{}(x: {ty}) -> {ty} {{ (x {operator} 0) ^ (x {operator} {}) }}\n",
                members.len(),
                bits - 1
            ));
        }
    }
    let (fixture, core) = accepted(&members);
    let amounts = core
        .functions
        .iter()
        .flat_map(|function| core_nodes(&fixture, function))
        .filter_map(|(operation, _, _)| {
            operation
                .strip_prefix("shift ")
                .map(|rest| rest.rsplit_once(' ').unwrap().1.parse::<u32>().unwrap())
        })
        .collect::<Vec<_>>();
    assert_eq!(amounts.len(), 32);
    assert_eq!(amounts.iter().filter(|amount| **amount == 0).count(), 16);

    members.clear();
    for (index, (ty, amount)) in [
        ("Word[8]", "8"),
        ("Word[16]", "16"),
        ("Word[32]", "0x20"),
        ("Word[64]", "64"),
        ("Word[64]", "0x1_0000_0000_0000_0000"),
        ("Word[8]", "-1"),
        ("Word[8]", "x"),
        ("Word[8]", "(1)"),
        ("Word[8]", "1 + 1"),
    ]
    .into_iter()
    .enumerate()
    {
        let amount_source = if amount == "1 + 1" { "(1 + 1)" } else { amount };
        members.push_str(&format!(
            "  spec bad{index}(x: {ty}) -> {ty} {{ x <<< {amount_source} }}\n"
        ));
        let highest = TYPES
            .into_iter()
            .find(|candidate| candidate.to_string() == ty)
            .and_then(|candidate| candidate.word_bits())
            .unwrap()
            - 1;
        expected.push((
            DiagnosticCode::InvalidShiftAmount,
            amount_source,
            format!("`<<<` on `{ty}` needs an amount from 0 through {highest}"),
        ));
    }
    let (fixture, result) = rejected(&members);
    let reported = reported(&fixture, &result);
    assert_eq!(
        reported
            .iter()
            .map(|(code, source, message)| (*code, *source, message.as_str()))
            .collect::<Vec<_>>(),
        expected
            .iter()
            .map(|(code, source, message)| (*code, *source, message.as_str()))
            .collect::<Vec<_>>()
    );
    assert!(
        result.diagnostics.iter().all(|diagnostic| {
            diagnostic.label() == "amount must be an unsigned integer literal"
        })
    );
}

#[test]
fn every_word_width_has_exact_literal_bounds() {
    for ty in TYPES.iter().filter(|ty| **ty != CoreType::Int) {
        let bits = ty.word_bits().unwrap();
        let maximum = u128::from(u64::MAX) >> (64 - bits);
        let (_, core) = accepted(&format!(
            "  spec max() -> {ty} {{ {maximum} }}\n  spec max_hex(x: {ty}) -> {ty} {{ x ^ 0x{maximum:x} }}\n"
        ));
        assert_eq!(
            core.functions[0]
                .body
                .literal()
                .and_then(CoreValue::word_as_u64),
            Some(u64::try_from(maximum).unwrap())
        );

        let over = (maximum + 1).to_string();
        let (fixture, result) = rejected(&format!(
            "  spec over() -> {ty} {{ {over} }}\n  spec negative(x: {ty}) -> {ty} {{ x & -1 }}\n"
        ));
        assert_eq!(
            reported(&fixture, &result),
            [
                (
                    DiagnosticCode::WordLiteralOutOfRange,
                    over.as_str(),
                    format!("literal is outside the range of `{ty}`")
                ),
                (
                    DiagnosticCode::NegativeWordLiteral,
                    "-1",
                    format!("`{ty}` literals cannot be negative")
                ),
            ]
        );
    }
}

#[test]
fn only_exact_decimal_word_widths_resolve() {
    let (_, core) =
        accepted("  spec a(x: Word[8], y: Word[16], z: Word[32], w: Word[64]) -> Int { 0 }\n");
    assert_eq!(
        core.functions[0].parameters,
        [
            CoreType::Word8,
            CoreType::Word16,
            CoreType::Word32,
            CoreType::Word64
        ]
    );
    let (fixture, result) = rejected(concat!(
        "  spec a(x: Word[12]) -> Int { 0 }\n",
        "  spec b(x: Word[128]) -> Int { 0 }\n",
        "  spec c(x: Word[0x20]) -> Int { 0 }\n",
        "  spec d(x: Word[032]) -> Int { 0 }\n",
        "  spec e(x: Word[3_2]) -> Int { 0 }\n",
        "  spec f(x: Word[0]) -> Int { 0 }\n",
    ));
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .map(|(code, source, _)| (code, source))
            .collect::<Vec<_>>(),
        [
            (DiagnosticCode::UnsupportedWordWidth, "12"),
            (DiagnosticCode::UnsupportedWordWidth, "128"),
            (DiagnosticCode::UnsupportedWordWidth, "0x20"),
            (DiagnosticCode::UnsupportedWordWidth, "032"),
            (DiagnosticCode::UnsupportedWordWidth, "3_2"),
            (DiagnosticCode::UnsupportedWordWidth, "0"),
        ]
    );
}

#[test]
fn unresolved_signatures_are_reported_once_without_cascades() {
    let (fixture, result) = rejected(concat!(
        "  spec bad_parameter(x: Word[12]) -> Int { x }\n",
        "  spec bad_result() -> Float { 1 }\n",
        "  spec caller() -> Int { bad_parameter(1) + bad_result() }\n",
        "  spec wrong_count() -> Int { bad_parameter() }\n",
        "  spec hidden(n: Int) -> Int { ~(n + missing) }\n",
        "  spec shifted(n: Int) -> Int { n << n }\n",
    ));
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .map(|(code, source, _)| (code, source))
            .collect::<Vec<_>>(),
        [
            (DiagnosticCode::UnsupportedWordWidth, "12"),
            (DiagnosticCode::UnsupportedType, "Float"),
            (DiagnosticCode::UnsupportedOperator, "~"),
            (DiagnosticCode::UnsupportedOperator, "<<"),
        ]
    );
}

#[test]
fn body_errors_precede_call_graph_errors_and_all_errors_are_ordered() {
    let (fixture, result) = rejected(concat!(
        "  spec loop_a() -> Int { loop_b() }\n",
        "  spec loop_b() -> Int { loop_a() }\n",
        "  spec later(x: Word[8]) -> Word[8] { x + 256 }\n",
        "  spec last() -> Int { nothing }\n",
    ));
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .map(|(code, source, _)| (code, source))
            .collect::<Vec<_>>(),
        [
            (DiagnosticCode::WordLiteralOutOfRange, "256"),
            (DiagnosticCode::UnknownParameter, "nothing"),
            (DiagnosticCode::CallCycle, "loop_a()"),
        ]
    );
}

#[test]
fn expression_events_and_core_nodes_follow_the_normative_accounting() {
    // Lookup and installation (2), the parameter's uniqueness check, name,
    // and width (3), the result name and width (2), and one event for each
    // of `^`, `~`, and `x`, plus the literal's own event, prefix, and one
    // significant digit (6): 13 analysis events. Core is the module and
    // one function node, one result-type node, one parameter-type node,
    // and the four expression nodes `x`, `~x`, `1`, and `^`: 8 nodes, each
    // one more event.
    let operators = module("  spec f(x: Word[8]) -> Word[8] { ~x ^ 1 }\n");
    // `g`: 2 + 1 result + 3 literal = 6. `f`: 2 + 1 result + 1 group +
    // 1 call = 5. Core: module + (2 + 1) + (2 + 1) = 7 nodes. The call
    // graph check consumes no events.
    let calls = module("  spec g() -> Int { 1 }\n  spec f() -> Int { (g()) }\n");
    // 2 + 3 parameter + 2 result + `<<<` + `x` + amount literal event,
    // prefix, and two significant digits = 13. Core: module + 2 + 1 + 2.
    let shift = module("  spec s(x: Word[32]) -> Word[32] { x <<< 0x1f }\n");
    for (fixture, events, nodes) in [(&operators, 21, 8), (&calls, 18, 7), (&shift, 19, 6)] {
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert!(exact.core.is_some());
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label);
        }
    }
}

#[test]
fn expression_storage_failures_return_no_partial_core() {
    let fixture = module("  spec g() -> Int { 1 }\n  spec f() -> Int { g() }\n");
    let node_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_core_node_slot = |_| false;
        analyzer.run()
    };
    let edge_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_call_edge_slot = |_| false;
        analyzer.run()
    };
    for (run, detail, source) in [
        (
            &node_failure as &dyn Fn() -> AnalysisResult,
            "typed Core expression storage allocation failed",
            "1",
        ),
        (&edge_failure, "call graph storage allocation failed", "g()"),
    ] {
        let first = run();
        assert_eq!(first, run());
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        let diagnostic = &first.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some(source)
        );
        assert_eq!(diagnostic.label(), detail);
    }
}

#[test]
fn bindings_and_conversions_build_typed_core_in_source_order() {
    let (fixture, core) = accepted(concat!(
        "  spec load16(lo: Word[8], hi: Word[8]) -> Word[16] {\n",
        "    let wide: Word[16] = lo as Word[16];\n",
        "    let high: Word[16] = (hi as Word[16]) << 8;\n",
        "    wide | high\n",
        "  }\n",
        "  spec square(x: Word[32]) -> Int { let n: Int = x as Int; n * n }\n",
        "  spec plain() -> Int { 1 }\n",
    ));
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    let load = &core.functions[0];
    assert_eq!(
        load.locals
            .iter()
            .map(|local| (
                local.name(),
                fixture.source().slice(local.span()).unwrap(),
                fixture.source().slice(local.name_span()).unwrap(),
                local.ty()
            ))
            .collect::<Vec<_>>(),
        [
            (
                "wide",
                "let wide: Word[16] = lo as Word[16];",
                "wide",
                CoreType::Word16
            ),
            (
                "high",
                "let high: Word[16] = (hi as Word[16]) << 8;",
                "high",
                CoreType::Word16
            ),
        ]
    );
    assert_eq!(
        expression_nodes(&fixture, &load.locals[0].value),
        owned(&[
            ("parameter 0", "lo", CoreType::Word8),
            ("convert from Word[8]", "lo as Word[16]", CoreType::Word16),
        ])
    );
    assert_eq!(
        expression_nodes(&fixture, &load.locals[1].value),
        owned(&[
            ("parameter 1", "hi", CoreType::Word8),
            ("convert from Word[8]", "hi as Word[16]", CoreType::Word16),
            ("shift << 8", "(hi as Word[16]) << 8", CoreType::Word16),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, load),
        owned(&[
            ("local 0", "wide", CoreType::Word16),
            ("local 1", "high", CoreType::Word16),
            ("infix |", "wide | high", CoreType::Word16),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[1]),
        owned(&[
            ("local 0", "n", CoreType::Int),
            ("local 0", "n", CoreType::Int),
            ("infix *", "n * n", CoreType::Int),
        ])
    );
    assert!(core.functions[2].locals.is_empty());
}

#[test]
fn a_conversion_operand_has_the_type_of_its_first_typed_leaf() {
    let (fixture, core) = accepted(concat!(
        "  spec k() -> Word[8] { 7 }\n",
        "  spec a(x: Word[8]) -> Int { (x + 1) as Int }\n",
        "  spec b(x: Word[8]) -> Int { (1 + x) as Int }\n",
        "  spec c(x: Word[32]) -> Word[8] { ((x << 3) ^ 0xff) as Word[8] }\n",
        "  spec d() -> Word[64] { k() as Word[64] }\n",
        "  spec e(x: Word[8]) -> Int { ((x as Word[64]) * 3) as Int }\n",
        "  spec f(x: Int) -> Int { -x as Int }\n",
        "  spec g(x: Word[16]) -> Word[16] { ~x as Word[16] }\n",
    ));
    let conversions = core
        .functions
        .iter()
        .map(|function| {
            let root = function.body.root().unwrap();
            let CoreNodeKind::Convert { from } = &root.kind else {
                return None;
            };
            Some((
                function.name.as_str(),
                from.clone(),
                root.ty.clone(),
                fixture.source().slice(root.span).unwrap(),
            ))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        conversions,
        [
            None,
            Some(("a", CoreType::Word8, CoreType::Int, "(x + 1) as Int")),
            Some(("b", CoreType::Word8, CoreType::Int, "(1 + x) as Int")),
            Some((
                "c",
                CoreType::Word32,
                CoreType::Word8,
                "((x << 3) ^ 0xff) as Word[8]"
            )),
            Some(("d", CoreType::Word8, CoreType::Word64, "k() as Word[64]")),
            Some((
                "e",
                CoreType::Word64,
                CoreType::Int,
                "((x as Word[64]) * 3) as Int"
            )),
            Some(("f", CoreType::Int, CoreType::Int, "-x as Int")),
            Some(("g", CoreType::Word16, CoreType::Word16, "~x as Word[16]")),
        ]
    );
    // The literal takes the leaf's type.
    assert_eq!(
        core_nodes(&fixture, &core.functions[2])[0],
        (String::from("literal 0x01"), "1", CoreType::Word8)
    );
}

#[test]
fn binding_names_are_unique_and_in_scope_only_after_their_binding() {
    let (fixture, result) = rejected(concat!(
        "  spec dup(x: Int) -> Int { let x: Int = 1; let t: Int = x; let t: Int = 2; t }\n",
        "  spec early() -> Int { let a: Int = b; let b: Int = b; b }\n",
        "  spec unknown(x: Int) -> Int { let a: Int = x; c }\n",
        "  spec function_name() -> Int { let a: Int = 1; early }\n",
        "  spec typed(x: Word[8]) -> Word[32] { let t: Word[8] = x; t }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::DuplicateBinding,
                "x",
                String::from("duplicate binding `x`")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "t",
                String::from("duplicate binding `t`")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "b",
                String::from("`b` is used before it is bound")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "b",
                String::from("`b` is used before it is bound")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "c",
                String::from("`c` is not a parameter or binding of `unknown`")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "early",
                String::from("`early` is not a parameter or binding of `function_name`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "t",
                String::from("`t` has type `Word[8]`, but `Word[32]` is required here")
            ),
        ]
    );
    let secondary = |index: usize| {
        let [secondary] = result.diagnostics[index].secondary_spans() else {
            panic!("diagnostic {index} must cite one earlier span");
        };
        (
            fixture.source().slice(secondary.span()).unwrap(),
            secondary.label(),
            secondary.span().start() < result.diagnostics[index].primary_span().start(),
        )
    };
    assert_eq!(secondary(0), ("x", "the parameter is here", true));
    assert_eq!(secondary(1), ("t", "the first binding is here", true));
    assert_eq!(secondary(2), ("b", "the binding is here", false));
    assert_eq!(secondary(3), ("b", "the binding is here", true));
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.notes()[0].as_str())
            .collect::<Vec<_>>(),
        [
            "each parameter and binding of a function has its own name; \
             Orange has no shadowing",
            "each parameter and binding of a function has its own name; \
             Orange has no shadowing",
            "a binding is in scope after its own `;`, for the bindings that follow it \
             and the result",
            "a binding is in scope after its own `;`, for the bindings that follow it \
             and the result",
            "a bare name in a `spec` body refers to one of its parameters or bindings",
            "to call the function `early`, write `early()` with its arguments",
            "Orange has no implicit conversions between types",
        ]
    );
}

#[test]
fn conversion_errors_are_reported_once_in_checking_order() {
    let (fixture, result) = rejected(concat!(
        "  spec untyped() -> Word[8] { (1 + 2) as Word[8] }\n",
        "  spec shifted() -> Int { (1 << 3) as Int }\n",
        "  spec mismatch(x: Word[32]) -> Word[8] { x as Word[16] }\n",
        "  spec target(x: Word[32]) -> Word[8] { x as Word[12] }\n",
        "  spec unknown() -> Int { (y + 1) as Int }\n",
        "  spec range(x: Word[8]) -> Int { (x + 256) as Int }\n",
        "  spec both(x: Word[8]) -> Word[8] { (x + y) as Word[16] }\n",
        "  spec callee() -> Int { missing() as Int }\n",
        "  spec untyped_binding() -> Int { let q: Float = 1; q as Int }\n",
        "  spec inner(x: Word[8]) -> Int { (x as Word[7]) as Int }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UntypedConversionOperand,
                "(1 + 2)",
                String::from("the operand of `as` has no type of its own")
            ),
            (
                DiagnosticCode::UntypedConversionOperand,
                "(1 << 3)",
                String::from("the operand of `as` has no type of its own")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "Word[16]",
                String::from("this conversion gives `Word[16]`, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::UnsupportedWordWidth,
                "12",
                String::from("`Word` width must be exactly 8, 16, 32, or 64")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "y",
                String::from("`y` is not a parameter of `unknown`")
            ),
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "256",
                String::from("literal is outside the range of `Word[8]`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "Word[16]",
                String::from("this conversion gives `Word[16]`, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "y",
                String::from("`y` is not a parameter of `both`")
            ),
            (
                DiagnosticCode::UnknownFunction,
                "missing",
                String::from("no typed `spec` function named `missing` in this module")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Float",
                String::from("unsupported binding type `Float`")
            ),
            (
                DiagnosticCode::UnsupportedWordWidth,
                "7",
                String::from("`Word` width must be exactly 8, 16, 32, or 64")
            ),
        ]
    );
    assert_eq!(
        result.diagnostics[0].notes(),
        [
            "write the literal where its type is required, or give it a type with a `let` \
          binding"
        ]
    );
    assert_eq!(
        result.diagnostics[2].notes(),
        ["`as` gives exactly the type written after it"]
    );
}

#[test]
fn calls_inside_bindings_and_conversions_join_the_call_graph() {
    let (fixture, result) = rejected(concat!(
        "  spec a() -> Int { let t: Int = b(); t }\n",
        "  spec b() -> Int { c() as Int }\n",
        "  spec c() -> Word[8] { (a() as Word[8]) + 1 }\n",
        "  spec d() -> Int { let q: Word[8] = e(); q as Int }\n",
        "  spec e() -> Word[16] { d() as Word[16] }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::TypeMismatch,
                "e()",
                String::from("`e` returns `Word[16]`, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::CallCycle,
                "a()",
                String::from("call cycle `a` -> `b` -> `c` -> `a`")
            ),
            (
                DiagnosticCode::CallCycle,
                "d()",
                String::from("call cycle `d` -> `e` -> `d`")
            ),
        ]
    );
}

#[test]
fn binding_and_conversion_events_and_core_nodes_follow_the_normative_accounting() {
    // Lookup and installation (2), the parameter's uniqueness check, name,
    // and width (3), the result name (1); the binding's uniqueness check
    // and type name (2), `as` and its target name (2), and `x` (1); and
    // the result `n` (1): 12 analysis events. Core is the module, one
    // function node, one result-type node, one parameter-type node, the
    // body node `n`, one binding node, one binding-type node, and the
    // binding's nodes `x` and `as`: 9 nodes, each one more event.
    let fixture = module("  spec f(x: Word[8]) -> Int { let n: Int = x as Int; n }\n");
    let (events, nodes) = (21, 9);
    let exact = fixture.analyze_with(Limits {
        events,
        nodes,
        ..Limits::DEFAULT
    });
    assert_eq!(exact.diagnostics, []);
    assert!(exact.core.is_some());
    for (limits, label) in [
        (
            Limits {
                events: events - 1,
                nodes,
                ..Limits::DEFAULT
            },
            "semantic event budget exhausted",
        ),
        (
            Limits {
                events,
                nodes: nodes - 1,
                ..Limits::DEFAULT
            },
            "typed Core node budget exhausted",
        ),
    ] {
        let first = fixture.analyze_with(limits);
        assert_eq!(first, fixture.analyze_with(limits));
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::SemanticResourceLimit
        );
        assert_eq!(first.diagnostics[0].label(), label);
    }
}

#[test]
fn binding_storage_failures_return_no_partial_core() {
    let fixture = module("  spec f(x: Int) -> Int { let t: Int = x; t }\n");
    let name_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_core_name = |_, _| false;
        analyzer.run()
    };
    let node_failure = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_core_node_slot = |_| false;
        analyzer.run()
    };
    for (run, detail, source) in [
        (
            &name_failure as &dyn Fn() -> AnalysisResult,
            "typed Core name storage allocation failed",
            "t",
        ),
        (
            &node_failure,
            "typed Core expression storage allocation failed",
            "x",
        ),
    ] {
        let first = run();
        assert_eq!(first, run());
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        let diagnostic = &first.diagnostics[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some(source)
        );
        assert_eq!(diagnostic.label(), detail);
    }
}

#[test]
fn rejects_foreign_spans_in_bindings_and_conversions() {
    let text = "edition 2026; module values { \
                spec value(x: Word[8]) -> Int { let t: Int = x as Int; t } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    let foreign = match &second.ast.module.functions[0].body {
        FunctionBody::Typed(body) => body,
        FunctionBody::Empty => unreachable!(),
    };
    let foreign_binding = &foreign.bindings[0];
    let ExpressionKind::Conversion(foreign_conversion) = &foreign_binding.value.kind else {
        unreachable!();
    };
    fn conversion_mut(ast: &mut SyntaxTree) -> &mut ConversionExpression {
        match &mut typed_body_mut(ast).bindings[0].value.kind {
            ExpressionKind::Conversion(conversion) => conversion,
            _ => unreachable!(),
        }
    }
    let mutations: [&dyn Fn(&mut SyntaxTree); 8] = [
        &|ast| typed_body_mut(ast).bindings[0].span = foreign_binding.span,
        &|ast| {
            named_mut(&mut typed_body_mut(ast).bindings[0].pattern)
                .name
                .span = named_of(&foreign_binding.pattern).name.span;
        },
        &|ast| {
            named_mut(&mut typed_body_mut(ast).bindings[0].pattern)
                .ty
                .span = named_of(&foreign_binding.pattern).ty.span;
        },
        &|ast| {
            named_mut(&mut typed_body_mut(ast).bindings[0].pattern)
                .ty
                .name
                .span = named_of(&foreign_binding.pattern).ty.name.span;
        },
        &|ast| typed_body_mut(ast).bindings[0].value.span = foreign_binding.value.span,
        &|ast| conversion_mut(ast).keyword_span = foreign_conversion.keyword_span,
        &|ast| conversion_mut(ast).target.span = foreign_conversion.target.span,
        &|ast| conversion_mut(ast).operand.span = foreign_conversion.operand.span,
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}

#[test]
fn let_and_as_are_ordinary_names_in_semantics() {
    let (fixture, core) = accepted(concat!(
        "  spec let(as: Int) -> Int { let let: Int = as; let as2: Int = let; let + as2 }\n",
        "  spec as(let: Word[8]) -> Int { (let) as Int }\n",
    ));
    assert_eq!(
        core_nodes(&fixture, &core.functions[0]),
        [
            (String::from("local 0"), "let", CoreType::Int),
            (String::from("local 1"), "as2", CoreType::Int),
            (String::from("infix +"), "let + as2", CoreType::Int),
        ]
    );
    assert_eq!(core.functions[1].name, "as");
}

fn array_of(element: CoreType, length: u32) -> CoreType {
    CoreType::Array(ArrayType::new(&element, length).unwrap())
}

#[test]
fn arrays_and_indices_build_typed_core_in_postorder() {
    let (fixture, core) = accepted(concat!(
        "  spec rot(x: Word[32]^4) -> Word[32]^4 { [x[1], x[2], x[3], x[0]] }\n",
        "  spec pick(x: Word[32]^4) -> Word[32] { rot(x)[3] ^ x[0x0] }\n",
        "  spec pair() -> Int^2 { let p: Int^2 = [1, -2,]; p }\n",
        "  spec first() -> Int { pair()[0] }\n",
    ));
    let words = array_of(CoreType::Word32, 4);
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        core.functions
            .iter()
            .map(|function| (function.parameters.clone(), function.result_type.clone()))
            .collect::<Vec<_>>(),
        [
            (vec![words.clone()], words.clone()),
            (vec![words.clone()], CoreType::Word32),
            (vec![], array_of(CoreType::Int, 2)),
            (vec![], CoreType::Int),
        ]
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[0]),
        owned(&[
            ("parameter 0", "x", words.clone()),
            ("index 1", "x[1]", CoreType::Word32),
            ("parameter 0", "x", words.clone()),
            ("index 2", "x[2]", CoreType::Word32),
            ("parameter 0", "x", words.clone()),
            ("index 3", "x[3]", CoreType::Word32),
            ("parameter 0", "x", words.clone()),
            ("index 0", "x[0]", CoreType::Word32),
            ("array of 4", "[x[1], x[2], x[3], x[0]]", words.clone()),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[1]),
        owned(&[
            ("parameter 0", "x", words.clone()),
            ("call #0 with 1", "rot(x)", words.clone()),
            ("index 3", "rot(x)[3]", CoreType::Word32),
            ("parameter 0", "x", words),
            ("index 0", "x[0x0]", CoreType::Word32),
            ("infix ^", "rot(x)[3] ^ x[0x0]", CoreType::Word32),
        ])
    );
    let pair = &core.functions[2];
    assert_eq!(pair.locals[0].ty(), array_of(CoreType::Int, 2));
    assert_eq!(
        expression_nodes(&fixture, &pair.locals[0].value),
        owned(&[
            ("literal 1", "1", CoreType::Int),
            ("literal -2", "-2", CoreType::Int),
            ("array of 2", "[1, -2,]", array_of(CoreType::Int, 2)),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, pair),
        owned(&[("local 0", "p", array_of(CoreType::Int, 2))])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[3]),
        owned(&[
            ("call #2 with 0", "pair()", array_of(CoreType::Int, 2)),
            ("index 0", "pair()[0]", CoreType::Int),
        ])
    );
}

#[test]
fn array_lengths_resolve_only_as_exact_decimals_from_1_through_256() {
    for (length, resolved) in [
        ("1", Some(1)),
        ("2", Some(2)),
        ("16", Some(16)),
        ("255", Some(255)),
        ("256", Some(256)),
        ("0", None),
        ("257", None),
        ("1000", None),
        ("01", None),
        ("007", None),
        ("0x10", None),
        ("0b1", None),
        ("1_6", None),
        ("99999999999999999999", None),
    ] {
        let members = format!("  spec f(x: Word[8]^{length}) -> Word[8] {{ 1 }}\n");
        let fixture = module(&members);
        let result = fixture.analyze();
        match resolved {
            Some(resolved) => {
                assert_eq!(result.diagnostics, [], "{length}");
                assert_eq!(
                    result.core.unwrap().functions[0].parameters,
                    [array_of(CoreType::Word8, resolved)]
                );
            }
            None => {
                assert!(result.core.is_none(), "{length}");
                assert_eq!(
                    reported(&fixture, &result),
                    [(
                        DiagnosticCode::UnsupportedArrayLength,
                        length,
                        String::from(
                            "an array length must be a decimal integer from 1 through 256"
                        )
                    )],
                    "{length}"
                );
                assert_eq!(
                    result.diagnostics[0].notes(),
                    ["write the length in decimal without leading zeros, as in `Word[32]^16`"]
                );
            }
        }
    }
    // The element type is resolved first; an unresolved element type is
    // reported alone.
    let (fixture, result) = rejected(concat!(
        "  spec a(x: Float^4) -> Int { 1 }\n",
        "  spec b(x: Word^4) -> Int { 1 }\n",
        "  spec c(x: Word[7]^0) -> Int { 1 }\n",
        "  spec d() -> Int^0 { [1] }\n",
        "  spec e() -> Int { let t: Word[8]^300 = [1]; 1 }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UnsupportedType,
                "Float^4",
                String::from("unsupported parameter type `Float`")
            ),
            (
                DiagnosticCode::UnsupportedWordWidth,
                "Word",
                String::from("`Word` requires an exact width of 8, 16, 32, or 64")
            ),
            (
                DiagnosticCode::UnsupportedWordWidth,
                "7",
                String::from("`Word` width must be exactly 8, 16, 32, or 64")
            ),
            (
                DiagnosticCode::UnsupportedArrayLength,
                "0",
                String::from("an array length must be a decimal integer from 1 through 256")
            ),
            (
                DiagnosticCode::UnsupportedArrayLength,
                "300",
                String::from("an array length must be a decimal integer from 1 through 256")
            ),
        ]
    );
}

#[test]
fn array_errors_are_reported_once_in_checking_order() {
    let (fixture, result) = rejected(concat!(
        "  spec count(x: Word[8]) -> Word[8]^3 { [x, x] }\n",
        "  spec one(x: Word[8]) -> Word[8]^2 { [x] }\n",
        "  spec element(x: Word[8]) -> Word[8]^2 { [x, 256] }\n",
        "  spec both() -> Word[8]^2 { [y, 1, 2] }\n",
        "  spec range(x: Word[8]^4) -> Word[8] { x[4] }\n",
        "  spec huge(x: Word[8]^4) -> Word[8] { x[99999999999999999999] }\n",
        "  spec scalar(x: Word[8]) -> Word[8] { x[0] }\n",
        "  spec mismatch(x: Word[8]^4) -> Word[16] { x[0] }\n",
        "  spec mismatch_range(x: Word[8]^4) -> Word[16] { x[9] }\n",
        "  spec unknown() -> Word[8] { z[0] }\n",
        "  spec whole(x: Word[8]^4) -> Word[8] { x }\n",
        "  spec other(x: Word[8]^4) -> Word[16] { x }\n",
        "  spec literal() -> Word[8]^4 { 1 }\n",
        "  spec array_for_scalar() -> Int { [1] }\n",
        "  spec nested() -> Int^1 { [[1]] }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::ArrayLengthMismatch,
                "[x, x]",
                String::from("this array has 2 elements, but `Word[8]^3` has 3")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "[x]",
                String::from("this array has 1 element, but `Word[8]^2` has 2")
            ),
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "256",
                String::from("literal is outside the range of `Word[8]`")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "[y, 1, 2]",
                String::from("this array has 3 elements, but `Word[8]^2` has 2")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "y",
                String::from("`y` is not a parameter of `both`")
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "4",
                String::from("index `4` is out of range for `Word[8]^4`")
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "99999999999999999999",
                String::from("index `99999999999999999999` is out of range for `Word[8]^4`")
            ),
            (
                DiagnosticCode::NotAnArray,
                "x",
                String::from("only an array can be indexed, but this has type `Word[8]`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x[0]",
                String::from("this element has type `Word[8]`, but `Word[16]` is required here")
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "9",
                String::from("index `9` is out of range for `Word[8]^4`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x[9]",
                String::from("this element has type `Word[8]`, but `Word[16]` is required here")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "z",
                String::from("`z` is not a parameter of `unknown`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x",
                String::from("`x` has type `Word[8]^4`, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x",
                String::from("`x` has type `Word[8]^4`, but `Word[16]` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "1",
                String::from("an integer literal cannot have type `Word[8]^4`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "[1]",
                String::from("an array literal cannot have type `Int`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "[1]",
                String::from("an array literal cannot have type `Int`")
            ),
        ]
    );
    let notes = |index: usize| result.diagnostics[index].notes().to_vec();
    assert_eq!(
        notes(0),
        ["an array literal lists every element of its type exactly once"]
    );
    assert_eq!(
        result.diagnostics[5].label(),
        "indices run from 0 through 3"
    );
    assert_eq!(
        notes(5),
        ["a literal index must be less than the array's length"]
    );
    assert_eq!(result.diagnostics[7].label(), "`Word[8]` has no elements");
    assert_eq!(
        notes(12),
        ["select one element with an index, such as `x[0]`"]
    );
    assert_eq!(
        notes(13),
        ["Orange has no implicit conversions between types"]
    );
    assert_eq!(
        notes(14),
        ["an array value is written `[e0, e1, ...]`, one element per index"]
    );
    assert_eq!(
        notes(15),
        ["an array literal is written where an array type `T^n` is required"]
    );
}

#[test]
fn operators_and_conversions_apply_to_elements_not_arrays() {
    let (fixture, result) = rejected(concat!(
        "  spec add(x: Word[8]^4) -> Word[8]^4 { x + x }\n",
        "  spec xor(x: Word[8]^4) -> Word[8]^4 { x ^ x }\n",
        "  spec rot(x: Word[8]^4) -> Word[8]^4 { x <<< 1 }\n",
        "  spec not(x: Word[8]^4) -> Word[8]^4 { ~x }\n",
        "  spec neg(x: Int^4) -> Int^4 { -x }\n",
        "  spec convert(x: Word[8]^4) -> Int { x as Int }\n",
        "  spec convert_call() -> Word[8] { rows() as Word[8] }\n",
        "  spec convert_literal() -> Int { [1] as Int }\n",
        "  spec rows() -> Word[8]^2 { [1, 2] }\n",
        // An undefined `as` stops there, as every undefined operator
        // does: `back()` is not examined, so it closes no cycle.
        "  spec convert_cycle() -> Int { back() as Int }\n",
        "  spec back() -> Word[8]^1 { [convert_cycle() as Word[8]] }\n",
    ));
    let note = String::from(ARRAY_OPERATOR_NOTE);
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UnsupportedOperator,
                "+",
                String::from("`+` is not defined for `Word[8]^4`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "^",
                String::from("`^` is not defined for `Word[8]^4`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "<<<",
                String::from("`<<<` is not defined for `Word[8]^4`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "~",
                String::from("prefix `~` is not defined for `Word[8]^4`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "-",
                String::from("prefix `-` is not defined for `Int^4`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "as",
                String::from("`as` is not defined for `Word[8]^4`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "as",
                String::from("`as` is not defined for `Word[8]^2`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "as",
                String::from("`as` is not defined for an array")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "as",
                String::from("`as` is not defined for `Word[8]^1`")
            ),
        ]
    );
    for diagnostic in &result.diagnostics[..5] {
        assert_eq!(diagnostic.notes(), [note.as_str()]);
    }
    for diagnostic in &result.diagnostics[5..] {
        assert_eq!(
            diagnostic.notes(),
            ["convert each element, such as `x[0] as Int`"]
        );
    }
    // Operators on elements and conversions of elements are ordinary.
    let (fixture, core) = accepted(concat!(
        "  spec mix(x: Word[8]^2, n: Int^2) -> Int {\n",
        "    ((x[0] ^ ~x[1]) as Int) + -n[1] * ((x[1] <<< 3) as Int)\n",
        "  }\n",
    ));
    let operations = core_nodes(&fixture, &core.functions[0])
        .into_iter()
        .map(|(operation, _, _)| operation)
        .collect::<Vec<_>>();
    assert!(
        operations.contains(&String::from("index 1")),
        "{operations:?}"
    );
}

#[test]
fn array_events_and_core_nodes_follow_the_normative_accounting() {
    // Lookup and installation (2); the parameter's uniqueness check,
    // name, width, and length (4); the result's name, width, and length
    // (3); the array literal (1); the index `[1]`, its prefix and one
    // significant digit, and its base `x` (4); and the literal `0x05`,
    // its prefix, and one significant digit (3): 17 analysis events.
    // Core is the module, one function node, one result-type node, one
    // parameter-type node, and the body nodes `x`, `x[1]`, `0x05`, and
    // the array: 8 nodes, each one more event.
    let fixture = module("  spec f(x: Word[8]^2) -> Word[8]^2 { [x[1], 0x05] }\n");
    let (events, nodes) = (25, 8);
    let exact = fixture.analyze_with(Limits {
        events,
        nodes,
        ..Limits::DEFAULT
    });
    assert_eq!(exact.diagnostics, []);
    assert!(exact.core.is_some());
    for (limits, label) in [
        (
            Limits {
                events: events - 1,
                nodes,
                ..Limits::DEFAULT
            },
            "semantic event budget exhausted",
        ),
        (
            Limits {
                events,
                nodes: nodes - 1,
                ..Limits::DEFAULT
            },
            "typed Core node budget exhausted",
        ),
    ] {
        let first = fixture.analyze_with(limits);
        assert_eq!(first, fixture.analyze_with(limits));
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::SemanticResourceLimit
        );
        assert_eq!(first.diagnostics[0].label(), label);
    }
    // An index literal decodes against the significant-bit limit before
    // its range is checked.
    let fixture = module(&format!(
        "  spec f(x: Word[8]^2) -> Word[8] {{ x[0x{}] }}\n",
        "f".repeat(4097)
    ));
    let result = fixture.analyze();
    assert!(result.core.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code(),
        DiagnosticCode::IntegerMagnitudeLimit
    );
}

#[test]
fn array_storage_failures_return_no_partial_core() {
    let fixture = module("  spec f(x: Int) -> Int^2 { [x, x] }\n");
    let first = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_core_node_slot = |nodes| nodes.len() < 2 && nodes.try_reserve(1).is_ok();
        analyzer.run()
    };
    let result = first();
    assert_eq!(result, first());
    assert!(result.core.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(
        fixture.source().slice(diagnostic.primary_span()),
        Some("[x, x]")
    );
    assert_eq!(
        diagnostic.label(),
        "typed Core expression storage allocation failed"
    );
}

#[test]
fn rejects_foreign_spans_in_arrays_and_indices() {
    let text = "edition 2026; module values { \
                spec value(x: Word[8]^2) -> Word[8]^2 { [x[1], x[0]] } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    let foreign_function = &second.ast.module.functions[0];
    let foreign = match &foreign_function.body {
        FunctionBody::Typed(body) => body,
        FunctionBody::Empty => unreachable!(),
    };
    let ExpressionKind::Array(foreign_array) = &foreign.expression.kind else {
        unreachable!();
    };
    let ExpressionKind::Index(foreign_index) = &foreign_array.elements[0].kind else {
        unreachable!();
    };
    let foreign_length = foreign_function.parameters[0].ty.length_span;
    let foreign_result_length = foreign.result_type.length_span;
    fn index_mut(ast: &mut SyntaxTree) -> &mut IndexExpression {
        let ExpressionKind::Array(array) = &mut typed_body_mut(ast).expression.kind else {
            unreachable!();
        };
        match &mut array.elements[0].kind {
            ExpressionKind::Index(index) => index,
            _ => unreachable!(),
        }
    }
    let mutations: [&dyn Fn(&mut SyntaxTree); 6] = [
        &|ast| ast.module.functions[0].parameters[0].ty.length_span = foreign_length,
        &|ast| typed_body_mut(ast).result_type.length_span = foreign_result_length,
        &|ast| typed_body_mut(ast).expression.span = foreign.expression.span,
        &|ast| {
            let ExpressionKind::Array(array) = &mut typed_body_mut(ast).expression.kind else {
                unreachable!();
            };
            array.elements[1].span = foreign_array.elements[1].span;
        },
        &|ast| index_mut(ast).index.span = foreign_index.index.span,
        &|ast| index_mut(ast).base.span = foreign_index.base.span,
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}

/// Renders a function's loops as `(index, accumulator, type, range,
/// visible bindings, scope)`.
fn loop_headers(function: &CoreFunction) -> Vec<(String, String, CoreType, String, u32, Vec<u32>)> {
    function
        .loops
        .iter()
        .map(|r#loop| {
            (
                r#loop.index_name().to_owned(),
                r#loop.accumulator_name().to_owned(),
                r#loop.ty(),
                format!("{}..{}", r#loop.start(), r#loop.end()),
                r#loop.visible_locals(),
                r#loop.scope().to_vec(),
            )
        })
        .collect()
}

#[test]
fn loops_updates_fills_and_selections_build_typed_core_in_postorder() {
    let (fixture, core) = accepted(concat!(
        "  spec sum(x: Int^4) -> Int { for i in 0..4 with s: Int = 0 { s + x[i] } }\n",
        "  spec odd(x: Word[8]^8) -> Word[8]^4 {\n",
        "    for i in 0..4 with s: Word[8]^4 = [0; 4] { s with [i] = x[2 * i + 1] }\n",
        "  }\n",
        "  spec grid() -> Int {\n",
        "    let base: Int = 10;\n",
        "    for i in 0..2 with a: Int = base { for j in 0..3 with b: Int = a { b + i * j } }\n",
        "  }\n",
        "  spec first() -> Int {\n",
        "    for i in 0..2 with a: Int = for j in 5..7 with b: Int = 0 { b + j } { a + i }\n",
        "  }\n",
    ));
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    let ints = array_of(CoreType::Int, 4);
    let bytes = array_of(CoreType::Word8, 8);
    let half = array_of(CoreType::Word8, 4);

    let sum = &core.functions[0];
    assert_eq!(
        core_nodes(&fixture, sum),
        owned(&[
            ("literal 0", "0", CoreType::Int),
            (
                "loop #0",
                "for i in 0..4 with s: Int = 0 { s + x[i] }",
                CoreType::Int
            ),
        ])
    );
    assert_eq!(
        loop_headers(sum),
        [(
            String::from("i"),
            String::from("s"),
            CoreType::Int,
            String::from("0..4"),
            0,
            vec![0]
        )]
    );
    assert_eq!(
        expression_nodes(&fixture, sum.loops[0].step()),
        owned(&[
            ("accumulator of loop #0", "s", CoreType::Int),
            ("parameter 0", "x", ints),
            ("index of loop #0", "i", CoreType::Int),
            ("select", "x[i]", CoreType::Int),
            ("infix +", "s + x[i]", CoreType::Int),
        ])
    );

    let odd = &core.functions[1];
    assert_eq!(
        core_nodes(&fixture, odd)
            .into_iter()
            .map(|(operation, _, ty)| (operation, ty))
            .collect::<Vec<_>>(),
        [
            (String::from("literal 0x00"), CoreType::Word8),
            (String::from("fill"), half.clone()),
            (String::from("loop #0"), half.clone()),
        ]
    );
    assert_eq!(
        expression_nodes(&fixture, odd.loops[0].step()),
        owned(&[
            ("accumulator of loop #0", "s", half.clone()),
            ("index of loop #0", "i", CoreType::Int),
            ("parameter 0", "x", bytes),
            ("literal 2", "2", CoreType::Int),
            ("index of loop #0", "i", CoreType::Int),
            ("infix *", "2 * i", CoreType::Int),
            ("literal 1", "1", CoreType::Int),
            ("infix +", "2 * i + 1", CoreType::Int),
            ("select", "x[2 * i + 1]", CoreType::Word8),
            ("update", "s with [i] = x[2 * i + 1]", half),
        ])
    );

    // Loops are numbered in source order of their `for` keywords; a step
    // sees the bindings in scope and every enclosing loop.
    let grid = &core.functions[2];
    assert_eq!(
        loop_headers(grid),
        [
            (
                String::from("i"),
                String::from("a"),
                CoreType::Int,
                String::from("0..2"),
                1,
                vec![0]
            ),
            (
                String::from("j"),
                String::from("b"),
                CoreType::Int,
                String::from("0..3"),
                1,
                vec![0, 1]
            ),
        ]
    );
    assert_eq!(
        core_nodes(&fixture, grid)
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>(),
        ["local 0", "loop #0"]
    );
    assert_eq!(
        expression_nodes(&fixture, grid.loops[0].step())
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>(),
        ["accumulator of loop #0", "loop #1"]
    );
    assert_eq!(
        expression_nodes(&fixture, grid.loops[1].step())
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>(),
        [
            "accumulator of loop #1",
            "index of loop #0",
            "index of loop #1",
            "infix *",
            "infix +"
        ]
    );

    // A loop in an initial value is not inside the outer loop's scope.
    let first = &core.functions[3];
    assert_eq!(
        loop_headers(first)
            .into_iter()
            .map(|(index, _, _, range, _, scope)| (index, range, scope))
            .collect::<Vec<_>>(),
        [
            (String::from("i"), String::from("0..2"), vec![0]),
            (String::from("j"), String::from("5..7"), vec![1]),
        ]
    );
    assert_eq!(
        core_nodes(&fixture, first)
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>(),
        ["literal 0", "loop #1", "loop #0"]
    );
}

#[test]
fn index_ranges_follow_interval_arithmetic_over_loop_ranges() {
    let accepted_indices = [
        "x[i - 1]",
        "x[4 - i]",
        "x[-i + 4]",
        "x[i * -1 + 4]",
        "x[(i - 1) * 1]",
        "x[0x3]",
        // Bounds are exact however wide the literals.
        "x[9223372036854775808 - 9223372036854775808]",
        "x[i - 1 + 0x1_0000_0000_0000_0000_0000_0000_0000_0000 \
           - 0x1_0000_0000_0000_0000_0000_0000_0000_0000]",
        "x[(i - 1) * 0x1_0000_0000_0000_0000 * 0x1_0000_0000_0000_0000 * 0 + i - 1]",
    ];
    for index in accepted_indices {
        accepted(&format!(
            "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
        ));
    }
    // Each bound is computed separately, so `i - i` runs from -3 through
    // 3 over 1..5 though its value is always 0, and `(i - 3) * (i - 3)`
    // runs from -2 through 4 over 1..5.
    for (index, message) in [
        (
            "x[i]",
            "this index runs from 1 through 4, out of range for `Word[8]^4`",
        ),
        (
            "x[i - 2]",
            "this index runs from -1 through 2, out of range for `Word[8]^4`",
        ),
        (
            "x[i - i]",
            "this index runs from -3 through 3, out of range for `Word[8]^4`",
        ),
        (
            "x[i + 3 - i]",
            "this index runs from 0 through 6, out of range for `Word[8]^4`",
        ),
        (
            "x[(i - 3) * (i - 3)]",
            "this index runs from -2 through 4, out of range for `Word[8]^4`",
        ),
        ("x[-1 + 5]", "index 4 is out of range for `Word[8]^4`"),
        (
            "x[i + 0x1_0000_0000_0000_0000]",
            "this index runs from 18446744073709551617 through 18446744073709551620, \
             out of range for `Word[8]^4`",
        ),
    ] {
        let (fixture, result) = rejected(&format!(
            "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
        ));
        assert_eq!(
            result.diagnostics.len(),
            1,
            "{index}: {:?}",
            result.diagnostics
        );
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            diagnostic.code(),
            DiagnosticCode::IndexOutOfRange,
            "{index}"
        );
        assert_eq!(diagnostic.message(), message, "{index}");
        assert_eq!(diagnostic.label(), "indices run from 0 through 3");
        assert_eq!(
            Some(format!(
                "x[{}]",
                fixture.source().slice(diagnostic.primary_span()).unwrap()
            )),
            Some(index.to_owned())
        );
    }
    let (_, result) = rejected(
        "  spec f(x: Word[8]^4) -> Word[8] { for i in 0..4 with s: Word[8] = 0 { s ^ x[i - 5] } }\n",
    );
    assert_eq!(
        result.diagnostics[0].message(),
        "this index runs from -5 through -2, out of range for `Word[8]^4`"
    );
    // A bound may have as many significant bits as an `Int` value, and no
    // more: 2^16382 + 2^16382 has 16,384 bits, 2^16383 + 2^16383 one more.
    let wide = |top: char| format!("0x{top}{}", "0".repeat(4095));
    let (half, full) = (wide('4'), wide('8'));
    accepted(&format!(
        "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 \
         {{ s ^ x[{half} + {half} - {half} - {half} + i - 1] }} }}\n"
    ));
    let (_, result) = rejected(&format!(
        "  spec f(x: Word[8]^4) -> Word[8] {{ for i in 1..5 with s: Word[8] = 0 \
         {{ s ^ x[{full} + {full} - {full} - {full} + i - 1] }} }}\n"
    ));
    assert_eq!(result.diagnostics.len(), 1, "{:?}", result.diagnostics);
    assert_eq!(
        result.diagnostics[0].code(),
        DiagnosticCode::IndexOutOfRange
    );
    assert_eq!(
        result.diagnostics[0].message(),
        "a bound of this index's range exceeds the 16384-significant-bit limit of `Int`"
    );
    assert_eq!(
        result.diagnostics[0].label(),
        "indices run from 0 through 3"
    );
}

#[test]
fn loop_update_and_fill_errors_are_reported_once_in_checking_order() {
    let (fixture, result) = rejected(concat!(
        "  spec variable(x: Word[8]^4, i: Int) -> Word[8] { x[i + 1] }\n",
        "  spec call(x: Word[8]^4) -> Word[8] { x[width(x)] }\n",
        "  spec width(x: Word[8]^4) -> Int { 4 }\n",
        "  spec empty() -> Int { for i in 3..3 with s: Int = 0 { s } }\n",
        "  spec long() -> Int { for i in 0..65537 with s: Int = 0 { s } }\n",
        "  spec longest() -> Int { for i in 65535..65536 with s: Int = 0 { s + i } }\n",
        "  spec same(x: Int) -> Int { for x in 0..2 with s: Int = 0 { s } }\n",
        "  spec twin() -> Int { for i in 0..2 with i: Int = 0 { i } }\n",
        "  spec inner() -> Int { for i in 0..2 with s: Int = 0 { for j in 0..2 with s: Int = 0 { s } } }\n",
        "  spec typed() -> Word[8] { for i in 0..2 with s: Int = 0 { s } }\n",
        "  spec scalar(x: Word[8]) -> Word[8] { x with [0] = 1 }\n",
        "  spec gives(x: Word[8]^2) -> Word[8] { x with [0] = 1 }\n",
        "  spec value(x: Word[8]^2) -> Word[8]^2 { x with [0] = 256 }\n",
        "  spec short() -> Word[8]^4 { [0; 3] }\n",
        "  spec none() -> Word[8]^4 { [0; 0] }\n",
        "  spec fill() -> Int { [0; 2] }\n",
        "  spec outside() -> Int { let a: Int = for i in 0..2 with s: Int = 0 { s + i }; s }\n",
        "  spec whole(x: Word[8]^2) -> Word[8]^2 { for i in 0..2 with s: Word[8]^2 = x { s + x } }\n",
        "  spec store(x: Word[8]^2) -> Word[8]^2 { for i in 0..3 with s: Word[8]^2 = x { s with [i] = 0 } }\n",
        "  spec step(x: Word[8]^2) -> Word[8]^2 { for i in 0..2 with s: Word[8]^2 = x { i } }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::NonStaticIndex,
                "i",
                String::from(
                    "an `Int` index may use only integer literals, loop indices, and words \
                     converted with `as Int`"
                )
            ),
            (
                DiagnosticCode::NonStaticIndex,
                "width(x)",
                String::from(
                    "an `Int` index may use only integer literals, loop indices, and words \
                     converted with `as Int`"
                )
            ),
            (
                DiagnosticCode::InvalidLoopRange,
                "3",
                String::from("the loop range 3..3 is empty")
            ),
            (
                DiagnosticCode::InvalidLoopRange,
                "65537",
                String::from("a loop bound must be at most 65536")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "x",
                String::from("duplicate name `x`")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "i",
                String::from("duplicate name `i`")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "s",
                String::from("duplicate name `s`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "for i in 0..2 with s: Int = 0 { s }",
                String::from("this loop has type `Int`, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::NotAnArray,
                "x",
                String::from("only an array can be updated, but this has type `Word[8]`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x with [0] = 1",
                String::from("an update gives an array, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "256",
                String::from("literal is outside the range of `Word[8]`")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "[0; 3]",
                String::from("this array has 3 elements, but `Word[8]^4` has 4")
            ),
            (
                DiagnosticCode::UnsupportedArrayLength,
                "0",
                String::from("an array length must be a decimal integer from 1 through 256")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "[0; 2]",
                String::from("an array literal cannot have type `Int`")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "s",
                String::from("`s` is not a parameter or binding of `outside`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "+",
                String::from("`+` is not defined for `Word[8]^2`")
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "i",
                String::from("this index runs from 0 through 2, out of range for `Word[8]^2`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "i",
                String::from("`i` has type `Int`, but `Word[8]^2` is required here")
            ),
        ]
    );
    let secondary = |index: usize| {
        result.diagnostics[index]
            .secondary_spans()
            .iter()
            .map(|secondary| {
                (
                    fixture.source().slice(secondary.span()).unwrap(),
                    secondary.label().to_owned(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(secondary(4), [("x", String::from("the parameter is here"))]);
    assert_eq!(
        secondary(5),
        [("i", String::from("the loop index is here"))]
    );
    assert_eq!(
        secondary(6),
        [("s", String::from("the accumulator is here"))]
    );
    assert_eq!(result.diagnostics[0].notes(), [STATIC_INDEX_NOTE]);
}

#[test]
fn loop_events_and_core_nodes_follow_the_normative_accounting() {
    // Lookup and installation (2); the parameter's uniqueness check,
    // name, width, and length (4); the result's name and width (2); the
    // loop (1); the bound `0` (prefix, no significant digit: 1) and the
    // bound `2` (prefix and one digit: 2); the two loop-name checks (2);
    // the accumulator type's name and width (2); the initial literal
    // `0` (literal and prefix: 2); and the step's `^`, `s`, the index
    // `x[i]`, its base `x`, and its index `i` (5): 23 analysis events.
    // Core is the module, one function node, one result-type node, one
    // parameter-type node, the body nodes `0` and the loop, one loop
    // node, one accumulator-type node, and the step nodes `s`, `x`, `i`,
    // the selection, and `^`: 13 nodes, each one more event.
    let fixture = module(
        "  spec f(x: Word[8]^2) -> Word[8] { for i in 0..2 with s: Word[8] = 0 { s ^ x[i] } }\n",
    );
    let (events, nodes) = (36, 13);
    let exact = fixture.analyze_with(Limits {
        events,
        nodes,
        ..Limits::DEFAULT
    });
    assert_eq!(exact.diagnostics, []);
    assert!(exact.core.is_some());
    for (limits, label) in [
        (
            Limits {
                events: events - 1,
                nodes,
                ..Limits::DEFAULT
            },
            "semantic event budget exhausted",
        ),
        (
            Limits {
                events,
                nodes: nodes - 1,
                ..Limits::DEFAULT
            },
            "typed Core node budget exhausted",
        ),
    ] {
        let first = fixture.analyze_with(limits);
        assert_eq!(first, fixture.analyze_with(limits));
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::SemanticResourceLimit
        );
        assert_eq!(first.diagnostics[0].label(), label);
    }
    // An update is one event, and a fill two: the literal and its length.
    let fixture = module("  spec f(x: Word[8]^2) -> Word[8]^2 { [0; 2] with [1] = x[0] }\n");
    // Lookup and installation (2); the parameter (4); the result (3); the
    // update (1); the fill and its length (2); the element `0` (2); the
    // index `1` (literal, prefix, and one digit: 3); the value's index,
    // its prefix and digit-free `0`, and its base (3): 20 events, and
    // the module, function, result, parameter, `0`, fill, `1`, `x`,
    // `x[0]`, and the update: 10 nodes.
    let exact = fixture.analyze_with(Limits {
        events: 30,
        nodes: 10,
        ..Limits::DEFAULT
    });
    assert_eq!(exact.diagnostics, []);
    let short = fixture.analyze_with(Limits {
        events: 29,
        nodes: 10,
        ..Limits::DEFAULT
    });
    assert_eq!(
        short.diagnostics[0].label(),
        "semantic event budget exhausted"
    );
    // A loop bound decodes against the significant-bit limit before its
    // range is checked.
    let fixture = module(&format!(
        "  spec f() -> Int {{ for i in 0..0x{} with s: Int = 0 {{ s }} }}\n",
        "f".repeat(4097)
    ));
    let result = fixture.analyze();
    assert!(result.core.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(
        result.diagnostics[0].code(),
        DiagnosticCode::IntegerMagnitudeLimit
    );
}

#[test]
fn loop_step_storage_failures_return_no_partial_core() {
    let fixture = module("  spec f(x: Int) -> Int { for i in 0..2 with s: Int = x { s + i } }\n");
    let first = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_core_node_slot = |nodes| nodes.len() < 2 && nodes.try_reserve(1).is_ok();
        analyzer.run()
    };
    let result = first();
    assert_eq!(result, first());
    assert!(result.core.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(
        fixture.source().slice(diagnostic.primary_span()),
        Some("s + i")
    );
}

#[test]
fn rejects_foreign_spans_in_loops_updates_and_fills() {
    let text = "edition 2026; module values { \
                spec value(x: Word[8]^2) -> Word[8]^2 { \
                for i in 0..2 with s: Word[8]^2 = [0; 2] { s with [i] = x[i] } } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    fn loop_of(ast: &SyntaxTree) -> &LoopExpression {
        let FunctionBody::Typed(body) = &ast.module.functions[0].body else {
            unreachable!();
        };
        let ExpressionKind::Loop(r#loop) = &body.expression.kind else {
            unreachable!();
        };
        r#loop
    }
    fn loop_mut(ast: &mut SyntaxTree) -> &mut LoopExpression {
        let ExpressionKind::Loop(r#loop) = &mut typed_body_mut(ast).expression.kind else {
            unreachable!();
        };
        r#loop
    }
    fn update_mut(ast: &mut SyntaxTree) -> &mut UpdateExpression {
        match &mut loop_mut(ast).step.kind {
            ExpressionKind::Update(update) => update,
            _ => unreachable!(),
        }
    }
    fn fill_mut(ast: &mut SyntaxTree) -> &mut FillExpression {
        match &mut loop_mut(ast).init.kind {
            ExpressionKind::Fill(fill) => fill,
            _ => unreachable!(),
        }
    }
    let foreign = loop_of(&second.ast).clone();
    let ExpressionKind::Update(foreign_update) = &foreign.step.kind else {
        unreachable!();
    };
    let ExpressionKind::Fill(foreign_fill) = &foreign.init.kind else {
        unreachable!();
    };
    let ExpressionKind::Index(foreign_index) = &foreign_update.value.kind else {
        unreachable!();
    };
    type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
    let mutations: Vec<Mutation<'_>> = vec![
        Box::new(|ast| loop_mut(ast).keyword_span = foreign.keyword_span),
        Box::new(|ast| loop_mut(ast).index.span = foreign.index.span),
        Box::new(|ast| loop_mut(ast).start_span = foreign.start_span),
        Box::new(|ast| loop_mut(ast).end_span = foreign.end_span),
        Box::new(|ast| {
            named_mut(&mut loop_mut(ast).accumulator).name.span =
                named_of(&foreign.accumulator).name.span;
        }),
        Box::new(|ast| {
            named_mut(&mut loop_mut(ast).accumulator).ty.length_span =
                named_of(&foreign.accumulator).ty.length_span;
        }),
        Box::new(|ast| update_mut(ast).keyword_span = foreign_update.keyword_span),
        Box::new(|ast| update_mut(ast).index.span = foreign_update.index.span),
        Box::new(|ast| fill_mut(ast).length_span = foreign_fill.length_span),
        Box::new(|ast| fill_mut(ast).element.span = foreign_fill.element.span),
        Box::new(|ast| {
            let ExpressionKind::Index(index) = &mut update_mut(ast).value.kind else {
                unreachable!();
            };
            index.index.span = foreign_index.index.span;
        }),
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}

/// Renders a function's conditionals as `(source, type, visible
/// bindings, scope)`.
fn conditional_headers<'text>(
    fixture: &'text Fixture,
    function: &CoreFunction,
) -> Vec<(&'text str, CoreType, u32, Vec<u32>)> {
    function
        .conditionals
        .iter()
        .map(|conditional| {
            (
                fixture.source().slice(conditional.span()).unwrap(),
                conditional.ty(),
                conditional.visible_locals(),
                conditional.scope().to_vec(),
            )
        })
        .collect()
}

#[test]
fn conditions_comparisons_and_divisions_build_typed_core_in_postorder() {
    let (fixture, core) = accepted(concat!(
        "  spec pick(c: Bool, x: Int) -> Int { if c { x } else if x < 0 { 0 - x } else { 7 } }\n",
        "  spec differ(a: Word[8], b: Word[8]) -> Bool { !(a == b) && true }\n",
        "  spec split(x: Int) -> Int^2 { [x / 3, x % 3] }\n",
        "  spec evens() -> Int {\n",
        "    let two: Int = 2;\n",
        "    for i in 0..4 with s: Int = 0 { if (i % two) == 0 { s + i } else { s } }\n",
        "  }\n",
    ));
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    let whole = "if c { x } else if x < 0 { 0 - x } else { 7 }";
    let later = "if x < 0 { 0 - x } else { 7 }";

    let pick = &core.functions[0];
    assert_eq!(
        core_nodes(&fixture, pick),
        owned(&[
            ("parameter 0", "c", CoreType::Bool),
            ("choose #0", whole, CoreType::Int),
        ])
    );
    assert_eq!(
        conditional_headers(&fixture, pick),
        [
            (whole, CoreType::Int, 0, vec![]),
            (later, CoreType::Int, 0, vec![])
        ]
    );
    assert_eq!(
        expression_nodes(&fixture, pick.conditionals[0].then_branch()),
        owned(&[("parameter 1", "x", CoreType::Int)])
    );
    // The later arm's condition and choice are the first arm's `else`.
    assert_eq!(
        expression_nodes(&fixture, pick.conditionals[0].else_branch()),
        owned(&[
            ("parameter 1", "x", CoreType::Int),
            ("literal 0", "0", CoreType::Int),
            ("compare < on Int", "x < 0", CoreType::Bool),
            ("choose #1", later, CoreType::Int),
        ])
    );
    assert_eq!(
        expression_nodes(&fixture, pick.conditionals[1].then_branch()),
        owned(&[
            ("literal 0", "0", CoreType::Int),
            ("parameter 1", "x", CoreType::Int),
            ("infix -", "0 - x", CoreType::Int),
        ])
    );
    assert_eq!(
        expression_nodes(&fixture, pick.conditionals[1].else_branch()),
        owned(&[("literal 7", "7", CoreType::Int)])
    );

    assert_eq!(
        core_nodes(&fixture, &core.functions[1]),
        owned(&[
            ("parameter 0", "a", CoreType::Word8),
            ("parameter 1", "b", CoreType::Word8),
            ("compare == on Word[8]", "a == b", CoreType::Bool),
            ("prefix !", "!(a == b)", CoreType::Bool),
            ("literal true", "true", CoreType::Bool),
            ("infix &&", "!(a == b) && true", CoreType::Bool),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[2]),
        owned(&[
            ("parameter 0", "x", CoreType::Int),
            ("literal 3", "3", CoreType::Int),
            ("infix /", "x / 3", CoreType::Int),
            ("parameter 0", "x", CoreType::Int),
            ("literal 3", "3", CoreType::Int),
            ("infix %", "x % 3", CoreType::Int),
            ("array of 2", "[x / 3, x % 3]", array_of(CoreType::Int, 2)),
        ])
    );

    // A conditional in a loop's step records the loop and the bindings
    // in scope.
    let evens = &core.functions[3];
    assert_eq!(
        conditional_headers(&fixture, evens),
        [(
            "if (i % two) == 0 { s + i } else { s }",
            CoreType::Int,
            1,
            vec![0]
        )]
    );
    assert_eq!(
        expression_nodes(&fixture, evens.loops[0].step()),
        owned(&[
            ("index of loop #0", "i", CoreType::Int),
            ("local 0", "two", CoreType::Int),
            ("infix %", "i % two", CoreType::Int),
            ("literal 0", "0", CoreType::Int),
            ("compare == on Int", "(i % two) == 0", CoreType::Bool),
            (
                "choose #0",
                "if (i % two) == 0 { s + i } else { s }",
                CoreType::Int
            ),
        ])
    );
}

#[test]
fn conditionals_are_numbered_in_source_order_of_their_if_keywords() {
    let (fixture, core) = accepted(concat!(
        "  spec f(a: Bool, b: Bool) -> Int {\n",
        "    if (if a { b } else { false }) { if b { 1 } else { 2 } } else if a { 3 } else { 4 }\n",
        "  }\n",
    ));
    assert_eq!(
        conditional_headers(&fixture, &core.functions[0])
            .into_iter()
            .map(|(source, ty, _, _)| (source, ty))
            .collect::<Vec<_>>(),
        [
            (
                "if (if a { b } else { false }) { if b { 1 } else { 2 } } else if a { 3 } else { 4 }",
                CoreType::Int
            ),
            ("if a { b } else { false }", CoreType::Bool),
            ("if b { 1 } else { 2 }", CoreType::Int),
            ("if a { 3 } else { 4 }", CoreType::Int),
        ]
    );
    // A comparison's operands and a conversion's operand may be
    // conditionals; their type comes from their first typed value.
    accepted(concat!(
        "  spec g(c: Bool, x: Word[8]) -> Bool { (if c { 1 } else { x }) < 3 }\n",
        "  spec h(c: Bool, x: Word[8]) -> Int { (if c { 1 } else { x }) as Int }\n",
        "  spec k(c: Bool, x: Int) -> Bool { 3 == (if c { x } else { 1 }) }\n",
    ));
}

#[test]
fn bool_literals_resolve_only_where_no_name_of_their_spelling_is_in_scope() {
    let (fixture, core) = accepted(concat!(
        "  spec parameter(true: Int) -> Int { true }\n",
        "  spec binding() -> Int { let false: Int = 5; false }\n",
        "  spec index() -> Int { for true in 0..3 with s: Int = 0 { s + true } }\n",
        "  spec before() -> Bool { let x: Bool = true; let true: Bool = false; x && true }\n",
        "  spec named(if: Int) -> Int { if + 1 }\n",
    ));
    let operations = |index: usize| {
        core_nodes(&fixture, &core.functions[index])
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>()
    };
    assert_eq!(operations(0), ["parameter 0"]);
    assert_eq!(operations(1), ["local 0"]);
    assert_eq!(
        expression_nodes(&fixture, core.functions[2].loops[0].step())
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>(),
        ["accumulator of loop #0", "index of loop #0", "infix +"]
    );
    // `true` in the first binding is the literal: the binding named
    // `true` starts after it.
    assert_eq!(
        expression_nodes(&fixture, &core.functions[3].locals[0].value)
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>(),
        ["literal true"]
    );
    assert_eq!(operations(3), ["local 0", "local 1", "infix &&"]);
    assert_eq!(operations(4), ["parameter 0", "literal 1", "infix +"]);
}

#[test]
fn condition_and_comparison_errors_are_reported_once_in_checking_order() {
    let (fixture, result) = rejected(concat!(
        "  spec gives(x: Int) -> Int { x < 1 }\n",
        "  spec untyped() -> Bool { 1 < 2 }\n",
        "  spec order(a: Bool, b: Bool) -> Bool { a < b }\n",
        "  spec arrays(x: Word[8]^2, y: Word[8]^2) -> Bool { x == y }\n",
        "  spec mixed(x: Word[8], y: Int) -> Bool { x == y }\n",
        "  spec literal() -> Bool { 1 }\n",
        "  spec both() -> Bool { true && 1 }\n",
        "  spec not(x: Word[8]) -> Word[8] { !x }\n",
        "  spec negate(b: Bool) -> Bool { -b }\n",
        "  spec sum(a: Bool, b: Bool) -> Bool { a + b }\n",
        "  spec bits(a: Bool, b: Bool) -> Bool { a & b }\n",
        "  spec logic(x: Word[8], y: Word[8]) -> Word[8] { x && y }\n",
        "  spec cond(x: Int) -> Int { if x { 1 } else { 0 } }\n",
        "  spec branches(c: Bool) -> Int { if c { 1 } else { true } }\n",
        "  spec to(c: Bool) -> Int { c as Int }\n",
        "  spec from(x: Int) -> Bool { x as Bool }\n",
        "  spec divide(a: Bool) -> Bool { a / a }\n",
        "  spec later() -> Bool { let x: Bool = y; let y: Bool = true; x }\n",
        "  spec unknown() -> Bool { z == 1 }\n",
        "  spec every(c: Bool, x: Int) -> Int { if x { y } else if c { 1 < 2 } else { c } }\n",
    ));
    let rows = [
        (
            DiagnosticCode::TypeMismatch,
            "x < 1",
            "a comparison gives `Bool`, but `Int` is required here",
        ),
        (
            DiagnosticCode::UntypedComparison,
            "1 < 2",
            "the operands of `<` have no type of their own",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "<",
            "`<` is not defined for `Bool`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "==",
            "`==` is not defined for `Word[8]^2`",
        ),
        (
            DiagnosticCode::TypeMismatch,
            "y",
            "`y` has type `Int`, but `Word[8]` is required here",
        ),
        (
            DiagnosticCode::TypeMismatch,
            "1",
            "an integer literal cannot have type `Bool`",
        ),
        (
            DiagnosticCode::TypeMismatch,
            "1",
            "an integer literal cannot have type `Bool`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "!",
            "prefix `!` is not defined for `Word[8]`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "-",
            "prefix `-` is not defined for `Bool`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "+",
            "`+` is not defined for `Bool`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "&",
            "`&` is not defined for `Bool`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "&&",
            "`&&` is not defined for `Word[8]`",
        ),
        (
            DiagnosticCode::TypeMismatch,
            "x",
            "`x` has type `Int`, but `Bool` is required here",
        ),
        (
            DiagnosticCode::TypeMismatch,
            "true",
            "`true` has type `Bool`, but `Int` is required here",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "as",
            "`as` does not convert to or from `Bool`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "as",
            "`as` does not convert to or from `Bool`",
        ),
        (
            DiagnosticCode::UnsupportedOperator,
            "/",
            "`/` is not defined for `Bool`",
        ),
        (
            DiagnosticCode::UnknownParameter,
            "y",
            "`y` is used before it is bound",
        ),
        (
            DiagnosticCode::UnknownParameter,
            "z",
            "`z` is not a parameter of `unknown`",
        ),
        // Every part of a conditional is checked, in source order.
        (
            DiagnosticCode::TypeMismatch,
            "x",
            "`x` has type `Int`, but `Bool` is required here",
        ),
        (
            DiagnosticCode::UnknownParameter,
            "y",
            "`y` is not a parameter of `every`",
        ),
        (
            DiagnosticCode::TypeMismatch,
            "1 < 2",
            "a comparison gives `Bool`, but `Int` is required here",
        ),
        (
            DiagnosticCode::UntypedComparison,
            "1 < 2",
            "the operands of `<` have no type of their own",
        ),
        (
            DiagnosticCode::TypeMismatch,
            "c",
            "`c` has type `Bool`, but `Int` is required here",
        ),
    ];
    assert_eq!(
        reported(&fixture, &result),
        rows.iter()
            .map(|(code, source, message)| (*code, *source, (*message).to_owned()))
            .collect::<Vec<_>>()
    );
    let notes = |index: usize| result.diagnostics[index].notes().to_vec();
    assert_eq!(
        notes(2),
        ["`Bool` values are compared with `==` and `!=`; they have no order"]
    );
    assert_eq!(notes(3), ["compare elements, such as `x[0] == y[0]`"]);
    assert_eq!(
        notes(5),
        ["the `Bool` values are written `true` and `false`"]
    );
    assert_eq!(
        notes(7),
        ["`!` negates a `Bool`; `~` is the bitwise complement of a word"]
    );
    assert_eq!(notes(9), [BOOL_OPERATOR_NOTE]);
    assert_eq!(
        notes(11),
        ["`&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators on words"]
    );
}

#[test]
fn index_ranges_follow_euclidean_division_and_its_total_rules() {
    for (range, index) in [
        ("0..8", "x[i / 2]"),
        ("0..100", "x[i % 4]"),
        ("0..100", "x[(i + 3) % 4]"),
        ("0..100", "x[-i % 4]"),
        ("0..100", "x[(0 - i) % -4]"),
        ("1..4", "x[7 / (i + 1)]"),
        ("0..4", "x[i % 0]"),
        ("0..4", "x[i / 0]"),
        ("0..3", "x[(i - 3) / -1]"),
        ("4..8", "x[i % 4]"),
        // Bounds are exact, so division is exact far beyond 64 bits.
        (
            "0..4",
            "x[(0x1_0000_0000_0000_0000 / 0x1_0000_0000_0000_0000) * i]",
        ),
        (
            "0..4",
            "x[(i + 0x1_0000_0000_0000_0000) % 0x1_0000_0000_0000_0000]",
        ),
        ("0..4", "x[i / (0 - 0x1_0000_0000_0000_0000)]"),
    ] {
        accepted(&format!(
            "  spec f(x: Word[8]^4) -> Word[8] {{ for i in {range} with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
        ));
    }
    for (range, index, message) in [
        (
            "0..9",
            "x[i / 2]",
            "this index runs from 0 through 4, out of range for `Word[8]^4`",
        ),
        (
            "0..100",
            "x[i % 5]",
            "this index runs from 0 through 4, out of range for `Word[8]^4`",
        ),
        (
            "0..4",
            "x[i / (i - i)]",
            "this index runs from -3 through 3, out of range for `Word[8]^4`",
        ),
        (
            "0..8",
            "x[i % 0]",
            "this index runs from 0 through 7, out of range for `Word[8]^4`",
        ),
        (
            "0..4",
            "x[i / -1]",
            "this index runs from -3 through 0, out of range for `Word[8]^4`",
        ),
        (
            "0..4",
            "x[9 % 5]",
            "index 4 is out of range for `Word[8]^4`",
        ),
        (
            "0..4",
            "x[-1 / 2]",
            "index -1 is out of range for `Word[8]^4`",
        ),
        (
            "0..4",
            "x[(0x1_0000_0000_0000_0000 * 3) / 2]",
            "index 27670116110564327424 is out of range for `Word[8]^4`",
        ),
        (
            "0..4",
            "x[(i + 0x1_0000_0000_0000_0000) % 0x1_0000_0000_0000_0001]",
            "this index runs from 0 through 18446744073709551616, out of range for `Word[8]^4`",
        ),
        (
            "0..4",
            "x[(i + 0x1_0000_0000_0000_0000) % 0]",
            "this index runs from 18446744073709551616 through 18446744073709551619, out of range for `Word[8]^4`",
        ),
    ] {
        let (fixture, result) = rejected(&format!(
            "  spec f(x: Word[8]^4) -> Word[8] {{ for i in {range} with s: Word[8] = 0 {{ s ^ {index} }} }}\n"
        ));
        assert_eq!(
            result.diagnostics.len(),
            1,
            "{index}: {:?}",
            result.diagnostics
        );
        let diagnostic = &result.diagnostics[0];
        assert_eq!(
            diagnostic.code(),
            DiagnosticCode::IndexOutOfRange,
            "{index}"
        );
        assert_eq!(diagnostic.message(), message, "{index}");
        assert_eq!(
            Some(format!(
                "x[{}]",
                fixture.source().slice(diagnostic.primary_span()).unwrap()
            )),
            Some(index.to_owned())
        );
    }
    // Division by a parameter is not static.
    let (fixture, result) = rejected(
        "  spec f(x: Word[8]^4, n: Int) -> Word[8] { for i in 0..4 with s: Word[8] = 0 { s ^ x[i / n] } }\n",
    );
    assert_eq!(
        reported(&fixture, &result),
        [(
            DiagnosticCode::NonStaticIndex,
            "n",
            String::from(
                "an `Int` index may use only integer literals, loop indices, and words \
                     converted with `as Int`"
            )
        )]
    );
    assert_eq!(result.diagnostics[0].notes(), [STATIC_INDEX_NOTE]);
}

#[test]
fn word_indices_range_over_their_types_and_narrow_through_operators() {
    let spec = |length: u32, index: &str| {
        format!(
            "  spec f(t: Word[8]^{length}, x: Word[8], y: Word[16], z: Word[64]) -> Word[8] {{ t[{index}] }}\n"
        )
    };
    for (length, index) in [
        // A word index ranges over its type.
        (256, "x"),
        (256, "z >> 56"),
        (256, "y >> 8"),
        (256, "y as Word[8]"),
        (256, "x + x"),
        (256, "x - 1"),
        // `&`, `|`, `^`, `~`, `/`, `%`, and literal shifts narrow it.
        (64, "(x >> 2) & 63"),
        (64, "x >> 2"),
        (16, "x & 15"),
        (16, "15 & x"),
        (16, "x >> 4"),
        (16, "x % 16"),
        (16, "y % 16"),
        (16, "(x >> 4) ^ (x & 15)"),
        (16, "(x & 7) | 8"),
        (16, "x / 16"),
        (16, "(x & 63) / 4"),
        (32, "~(x | 224)"),
        (32, "(x & 15) << 1"),
        // `+`, `-`, and `*` narrow only where they cannot wrap.
        (32, "(x & 15) + 16"),
        (32, "(x & 15) * 2"),
        (32, "31 - (x & 15)"),
        // A wider word converted from a narrower one keeps its range.
        (16, "(x & 15) as Word[16]"),
        // Conditionals join their branches.
        (16, "if x < 3 { x & 7 } else { 15 }"),
        // An `Int` index may convert words.
        (256, "((x as Int) / 2) + 128"),
        (4, "(x as Int) % 4"),
        (4, "(x & 3) as Int"),
        (256, "(y as Int) / 256"),
        (16, "if x < 3 { 0 } else { (x as Int) / 16 }"),
    ] {
        let (_, core) = accepted(&spec(length, index));
        // A word index is proved in range and then converted to `Int`,
        // so every selection takes an `Int` position.
        let [.., root, position, select] = core.functions()[0].body().nodes() else {
            panic!("{index}");
        };
        assert!(matches!(select.kind(), CoreNodeKind::Select), "{index}");
        assert_eq!(position.ty(), CoreType::Int, "{index}");
        if !index.contains("as Int") {
            let CoreNodeKind::Convert { from } = position.kind() else {
                panic!("{index}");
            };
            assert_eq!(root.ty(), *from, "{index}");
            assert_eq!(root.span(), position.span(), "{index}");
        }
    }
    for (length, index, message) in [
        (
            16,
            "x",
            "this index runs from 0 through 255, out of range for `Word[8]^16`",
        ),
        (
            255,
            "x",
            "this index runs from 0 through 255, out of range for `Word[8]^255`",
        ),
        (
            256,
            "z",
            "this index runs from 0 through 18446744073709551615, out of range for `Word[8]^256`",
        ),
        (
            16,
            "x & 31",
            "this index runs from 0 through 31, out of range for `Word[8]^16`",
        ),
        (
            16,
            "(x & 15) + 1",
            "this index runs from 1 through 16, out of range for `Word[8]^16`",
        ),
        (
            16,
            "(x & 15) - 1",
            "this index runs from 0 through 255, out of range for `Word[8]^16`",
        ),
        (
            16,
            "x % 17",
            "this index runs from 0 through 16, out of range for `Word[8]^16`",
        ),
        (
            16,
            "x % (x & 15)",
            "this index runs from 0 through 255, out of range for `Word[8]^16`",
        ),
        (
            16,
            "(x & 15) << 5",
            "this index runs from 0 through 255, out of range for `Word[8]^16`",
        ),
        (
            16,
            "y as Word[8]",
            "this index runs from 0 through 255, out of range for `Word[8]^16`",
        ),
        (
            16,
            "if x < 3 { x & 7 } else { 16 }",
            "this index runs from 0 through 16, out of range for `Word[8]^16`",
        ),
        (
            256,
            "(x as Int) + 1",
            "this index runs from 1 through 256, out of range for `Word[8]^256`",
        ),
        (
            256,
            "(x as Int) - 1",
            "this index runs from -1 through 254, out of range for `Word[8]^256`",
        ),
    ] {
        let (fixture, result) = rejected(&spec(length, index));
        assert_eq!(
            reported(&fixture, &result),
            [(DiagnosticCode::IndexOutOfRange, index, message.to_owned())],
            "{index}"
        );
    }
    // An `Int` parameter has no bound, and neither does a word
    // converted to `Int` and then multiplied by one.
    for (index, part) in [("n", "n"), ("(x as Int) * n", "n")] {
        let (fixture, result) = rejected(&format!(
            "  spec f(t: Word[8]^4, x: Word[8], n: Int) -> Word[8] {{ t[{index}] }}\n"
        ));
        assert_eq!(
            reported(&fixture, &result),
            [(
                DiagnosticCode::NonStaticIndex,
                part,
                String::from(
                    "an `Int` index may use only integer literals, loop indices, and words \
                     converted with `as Int`"
                )
            )],
            "{index}"
        );
        assert_eq!(result.diagnostics[0].notes(), [STATIC_INDEX_NOTE]);
    }
}

#[test]
fn word_index_events_and_core_nodes_follow_the_normative_accounting() {
    // Lookup and installation (2); `t`'s uniqueness check, name, width,
    // and length (4); `x`'s uniqueness check, name, and width (3); the
    // result's name and width (2); then `t` (1), the index (1), `x` (1),
    // `&` (1), and `15` (literal, prefix, and two digits: 4): 19 analysis
    // events. Computing the range consumes none. Core is the module, one
    // function node, one result-type node, two parameter-type nodes, and
    // the body nodes `t`, `x`, `15`, `&`, the conversion to `Int`, and
    // the selection: 11 nodes, each one more event.
    let fixture = module("  spec f(t: Word[8]^16, x: Word[8]) -> Word[8] { t[x & 15] }\n");
    let (events, nodes) = (30, 11);
    let exact = fixture.analyze_with(Limits {
        events,
        nodes,
        ..Limits::DEFAULT
    });
    assert_eq!(exact.diagnostics, []);
    assert!(exact.core.is_some());
    for (limits, label) in [
        (
            Limits {
                events: events - 1,
                nodes,
                ..Limits::DEFAULT
            },
            "semantic event budget exhausted",
        ),
        (
            Limits {
                events,
                nodes: nodes - 1,
                ..Limits::DEFAULT
            },
            "typed Core node budget exhausted",
        ),
    ] {
        let first = fixture.analyze_with(limits);
        assert_eq!(first, fixture.analyze_with(limits));
        assert!(first.core.is_none());
        assert_eq!(first.diagnostics.len(), 1);
        assert_eq!(
            first.diagnostics[0].code(),
            DiagnosticCode::SemanticResourceLimit
        );
        assert_eq!(first.diagnostics[0].label(), label);
    }
}

#[test]
fn condition_events_and_core_nodes_follow_the_normative_accounting() {
    // Lookup and installation (2); the parameter's uniqueness check and
    // name (2); the result's name (1); the arm's `if` (1); the condition
    // `c` (1); and the values `1` and `2` (literal, prefix, and one
    // digit: 3 each): 13 analysis events. Core is the module, one
    // function node, one result-type node, one parameter-type node, the
    // body nodes `c` and the choice, one conditional node, and the
    // branch nodes `1` and `2`: 9 nodes, each one more event.
    let conditional = module("  spec f(c: Bool) -> Int { if c { 1 } else { 2 } }\n");
    // Lookup and installation (2); the parameter's uniqueness check,
    // name, and width (3); the result's name (1); the comparison (1);
    // `x` (1); and `1` (3): 11 events. Core is the module, the function,
    // the result and parameter types, and `x`, `1`, and the comparison:
    // 7 nodes.
    let comparison = module("  spec g(x: Word[8]) -> Bool { x < 1 }\n");
    for (fixture, events, nodes) in [(&conditional, 22, 9), (&comparison, 18, 7)] {
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert!(exact.core.is_some());
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label);
        }
    }
    // Each arm of a chain is one event and one conditional node, and
    // each later arm's condition and choice are nodes of the earlier
    // arm's `else` branch: 2 + 2 + 1 events for the signature, 3 for
    // the arms, 3 for the conditions, and 12 for the four values; 4
    // nodes for the module and signature, 2 in the body, 4 for each of
    // the first two conditionals, and 3 for the last.
    let chain = module(
        "  spec f(c: Bool) -> Int { if c { 1 } else if c { 2 } else if c { 3 } else { 4 } }\n",
    );
    let (events, nodes) = (23 + 17, 17);
    let exact = chain.analyze_with(Limits {
        events,
        nodes,
        ..Limits::DEFAULT
    });
    assert_eq!(exact.diagnostics, []);
    assert_eq!(exact.core.unwrap().functions[0].conditionals.len(), 3);
    for limits in [
        Limits {
            events: events - 1,
            nodes,
            ..Limits::DEFAULT
        },
        Limits {
            events,
            nodes: nodes - 1,
            ..Limits::DEFAULT
        },
    ] {
        assert!(chain.analyze_with(limits).core.is_none());
    }
}

#[test]
fn branch_storage_failures_return_no_partial_core() {
    let fixture = module("  spec f(c: Bool, x: Int) -> Int { if c { x + 1 } else { x } }\n");
    let first = || {
        let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
        analyzer.reserve_core_node_slot = |nodes| nodes.len() < 2 && nodes.try_reserve(1).is_ok();
        analyzer.run()
    };
    let result = first();
    assert_eq!(result, first());
    assert!(result.core.is_none());
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
    assert_eq!(
        fixture.source().slice(diagnostic.primary_span()),
        Some("x + 1")
    );
}

#[test]
fn rejects_foreign_spans_in_conditionals() {
    let text = "edition 2026; module values { \
                spec value(c: Bool, x: Int) -> Int { \
                if c { x } else if x < 1 { x / 2 } else { 0 } } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    fn conditional_of(ast: &SyntaxTree) -> &ConditionalExpression {
        let FunctionBody::Typed(body) = &ast.module.functions[0].body else {
            unreachable!();
        };
        let ExpressionKind::Conditional(conditional) = &body.expression.kind else {
            unreachable!();
        };
        conditional
    }
    fn conditional_mut(ast: &mut SyntaxTree) -> &mut ConditionalExpression {
        let ExpressionKind::Conditional(conditional) = &mut typed_body_mut(ast).expression.kind
        else {
            unreachable!();
        };
        conditional
    }
    let foreign = conditional_of(&second.ast).clone();
    type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
    let mutations: Vec<Mutation<'_>> = vec![
        Box::new(|ast| conditional_mut(ast).else_span = foreign.else_span),
        Box::new(|ast| {
            conditional_mut(ast).arms[0].keyword_span = foreign.arms[0].keyword_span;
        }),
        Box::new(|ast| {
            conditional_mut(ast).arms[1].keyword_span = foreign.arms[1].keyword_span;
        }),
        Box::new(|ast| {
            conditional_mut(ast).arms[0].condition.span = foreign.arms[0].condition.span;
        }),
        Box::new(|ast| {
            let ExpressionKind::Binary(binary) = &mut conditional_mut(ast).arms[1].condition.kind
            else {
                unreachable!();
            };
            let ExpressionKind::Binary(foreign_binary) = &foreign.arms[1].condition.kind else {
                unreachable!();
            };
            binary.operator_span = foreign_binary.operator_span;
        }),
        Box::new(|ast| {
            conditional_mut(ast).arms[1].value.span = foreign.arms[1].value.span;
        }),
        Box::new(|ast| conditional_mut(ast).otherwise.span = foreign.otherwise.span),
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}

/// Modules of one program in one source map, the first the root.
struct Program {
    sources: SourceMap,
    ids: Vec<SourceId>,
    asts: Vec<SyntaxTree>,
}

impl Program {
    fn new(texts: &[&str]) -> Self {
        let mut sources = SourceMap::new();
        let mut ids = Vec::new();
        let mut asts = Vec::new();
        for (index, text) in texts.iter().enumerate() {
            let id = sources.add(format!("m{index}.or"), *text).unwrap();
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::E2026);
            assert_eq!(lexed.diagnostics(), [], "{text}");
            let parsed = parse(source, &lexed);
            assert_eq!(parsed.diagnostics(), [], "{text}");
            ids.push(id);
            asts.push(parsed.into_ast().unwrap());
        }
        Self { sources, ids, asts }
    }

    fn modules(&self) -> Vec<(&SourceFile, &SyntaxTree)> {
        self.ids
            .iter()
            .zip(&self.asts)
            .map(|(id, ast)| (self.sources.get(*id).unwrap(), ast))
            .collect()
    }

    fn analyze(&self) -> AnalysisResult {
        let modules = self.modules();
        analyze_program(modules[0], &modules[1..])
    }

    fn slice(&self, span: Span) -> &str {
        self.sources
            .get(span.source())
            .unwrap()
            .slice(span)
            .unwrap()
    }

    /// Each diagnostic's code, message, and primary text.
    fn report(&self, result: &AnalysisResult) -> Vec<(DiagnosticCode, String, String)> {
        result
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                (
                    diagnostic.code(),
                    diagnostic.message().to_owned(),
                    self.slice(diagnostic.primary_span()).to_owned(),
                )
            })
            .collect()
    }
}

fn call_targets(expression: &CoreExpression) -> Vec<u32> {
    expression
        .nodes()
        .iter()
        .filter_map(|node| match node.kind() {
            CoreNodeKind::Call { function, .. } => Some(function.index()),
            _ => None,
        })
        .collect()
}

#[test]
fn links_used_modules_in_dependency_order_with_dense_identities() {
    let program = Program::new(&[
        concat!(
            "edition 2026; module main { use hmac; use sha; ",
            "spec digest() -> Word[32] { hmac::tag(sha::k()) } ",
            "spec local(x: Word[32]) -> Word[32] { x } ",
            "spec answer() -> Int { sha::seven() * 6 } ",
            "}"
        ),
        "edition 2026; module hmac { use sha; spec tag(x: Word[32]) -> Word[32] { sha::mix(x ^ 0x36363636) } }",
        // Never reached from the root, so never checked.
        "edition 2026; module unused { spec never() -> Int { nope } }",
        concat!(
            "edition 2026; module sha { ",
            "spec k() -> Word[32] { 0x428a2f98 } ",
            "spec mix(x: Word[32]) -> Word[32] { (x >>> 2) ^ k() } ",
            "spec seven() -> Int { 7 } ",
            "}"
        ),
    ]);
    let result = program.analyze();
    assert_eq!(result.diagnostics(), []);
    assert_eq!(result, program.analyze());
    let core = result.core().unwrap();
    assert_eq!(core.name(), "main");
    assert_eq!(core.span(), program.asts[0].module.span);
    assert_eq!(
        core.functions()
            .iter()
            .map(|function| (function.id().index(), function.module(), function.name()))
            .collect::<Vec<_>>(),
        [
            (0, "sha", "k"),
            (1, "sha", "mix"),
            (2, "sha", "seven"),
            (3, "hmac", "tag"),
            (4, "main", "digest"),
            (5, "main", "local"),
            (6, "main", "answer"),
        ]
    );
    assert_eq!(
        core.entry_functions()
            .iter()
            .map(CoreFunction::name)
            .collect::<Vec<_>>(),
        ["digest", "local", "answer"]
    );
    assert_eq!(call_targets(core.functions()[1].body()), [0]);
    assert_eq!(call_targets(core.functions()[3].body()), [1]);
    assert_eq!(call_targets(core.functions()[4].body()), [0, 3]);
    assert_eq!(call_targets(core.functions()[6].body()), [2]);
    assert_eq!(program.slice(core.functions()[3].name_span()), "tag");
    assert_eq!(core.functions()[3].name_span().source(), program.ids[1]);

    let evaluated = crate::eval::evaluate(core);
    assert_eq!(evaluated.diagnostics(), []);
    let k = 0x428a_2f98_u32;
    let digest = (k ^ 0x3636_3636).rotate_right(2) ^ k;
    assert_eq!(
        evaluated
            .values()
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        [
            format!("main::digest: Word[32] = 0x{digest:08x}"),
            String::from("main::answer: Int = 42"),
        ]
    );

    // A module used by no one is checked alone, as the root.
    let alone = Program::new(&["edition 2026; module sha { spec k() -> Word[32] { 1 } }"]);
    let core = alone.analyze().into_core().unwrap();
    assert_eq!(core.entry_functions(), core.functions());
    assert_eq!(core.functions()[0].module(), "sha");
}

#[test]
fn a_chain_of_sixty_four_modules_links_and_evaluates() {
    let texts = (0..MAX_MODULES_PER_PROGRAM)
        .map(|index| {
            if index + 1 == MAX_MODULES_PER_PROGRAM {
                format!("edition 2026; module m{index} {{ spec v() -> Int {{ 1 }} }}")
            } else {
                let next = index + 1;
                format!(
                    "edition 2026; module m{index} {{ use m{next}; \
                     spec v() -> Int {{ m{next}::v() + 1 }} }}"
                )
            }
        })
        .collect::<Vec<_>>();
    // Supplied in reverse, so that order comes from the uses alone.
    let mut ordered = vec![texts[0].as_str()];
    ordered.extend(texts[1..].iter().rev().map(String::as_str));
    let program = Program::new(&ordered);
    let core = program.analyze().into_core().unwrap();
    assert_eq!(core.functions().len(), MAX_MODULES_PER_PROGRAM);
    assert_eq!(core.functions()[0].module(), "m63");
    assert_eq!(core.entry_functions().len(), 1);
    let evaluated = crate::eval::evaluate(&core);
    assert_eq!(
        evaluated.values().unwrap()[0].to_string(),
        "m0::v: Int = 64"
    );

    // Modules the root does not reach are ignored and not counted, even
    // when they share a name with each other.
    let mut texts = texts;
    let unreached = (0..200)
        .map(|index| {
            format!(
                "edition 2026; module spare{} {{ use m0; use spare{}; }}",
                index % 150,
                index % 150
            )
        })
        .collect::<Vec<_>>();
    let mut supplied = texts.iter().map(String::as_str).collect::<Vec<_>>();
    supplied.extend(unreached.iter().map(String::as_str));
    let program = Program::new(&supplied);
    let core = program.analyze().into_core().unwrap();
    assert_eq!(core.functions().len(), MAX_MODULES_PER_PROGRAM);
    let alone = ["edition 2026; module main { spec v() -> Int { 7 } }"]
        .into_iter()
        .chain(texts[1..].iter().map(String::as_str))
        .chain(["edition 2026; module m64 {}"])
        .collect::<Vec<_>>();
    assert_eq!(alone.len(), MAX_MODULES_PER_PROGRAM + 1);
    let program = Program::new(&alone);
    let core = program.analyze().into_core().unwrap();
    assert_eq!(core.functions().len(), 1);
    assert_eq!(
        crate::eval::evaluate(&core).values().unwrap()[0].to_string(),
        "main::v: Int = 7"
    );

    // A 65th reachable module stops the search before any module is
    // checked, however many modules are supplied.
    texts[MAX_MODULES_PER_PROGRAM - 1] =
        String::from("edition 2026; module m63 { use m64; spec v() -> Int { m64::v() + 1 } }");
    texts.push(String::from(
        "edition 2026; module m64 { spec v() -> Int { 1 } }",
    ));
    for supplied in [
        texts.iter().map(String::as_str).collect::<Vec<_>>(),
        texts
            .iter()
            .chain(&unreached)
            .map(String::as_str)
            .collect::<Vec<_>>(),
    ] {
        let program = Program::new(&supplied);
        let result = program.analyze();
        assert_eq!(
            program.report(&result),
            [(
                DiagnosticCode::SemanticResourceLimit,
                String::from("semantic analysis resource limit exceeded"),
                program.slice(program.asts[0].module.span).to_owned()
            )]
        );
        assert_eq!(
            result.diagnostics()[0].label(),
            "program reaches more than 64 modules"
        );
    }
}

#[test]
fn a_module_of_the_program_has_no_namesake() {
    // A supplied module that shares the root's name, or that of a used
    // module, is reported when the search enters the module of the
    // program; namesakes the program never reaches are ignored.
    let program = Program::new(&[
        "edition 2026; module main { use lib; spec v() -> Int { lib::one() } }",
        "edition 2026; module lib { spec one() -> Int { 1 } }",
        "edition 2026; module spare { }",
        "edition 2026; module main { }",
        "edition 2026; module spare { }",
        "edition 2026; module lib { }",
    ]);
    let result = program.analyze();
    assert_eq!(
        program.report(&result),
        [
            (
                DiagnosticCode::DuplicateModule,
                String::from("duplicate module `main`"),
                String::from("main")
            ),
            (
                DiagnosticCode::DuplicateModule,
                String::from("duplicate module `lib`"),
                String::from("lib")
            ),
        ]
    );
    assert_eq!(
        result.diagnostics()[0].primary_span().source(),
        program.ids[3]
    );
    assert_eq!(
        result.diagnostics()[0].secondary_spans()[0].span().source(),
        program.ids[0]
    );
    assert_eq!(
        result.diagnostics()[1].primary_span().source(),
        program.ids[5]
    );

    let program = Program::new(&[
        "edition 2026; module main { use lib; spec v() -> Int { lib::one() } }",
        "edition 2026; module lib { spec one() -> Int { 1 } }",
        "edition 2026; module spare { }",
        "edition 2026; module spare { }",
    ]);
    assert!(program.analyze().into_core().is_some());
}

#[test]
fn module_graph_errors_name_the_use_that_causes_them() {
    let alone = Program::new(&["edition 2026; module main { use sha; spec f() -> Int { 1 } }"]);
    let result = alone.analyze();
    assert_eq!(result, analyze(alone.modules()[0].0, &alone.asts[0]));
    assert_eq!(
        alone.report(&result),
        [(
            DiagnosticCode::UnknownModule,
            String::from("no module named `sha` in this program"),
            String::from("sha")
        )]
    );

    let program = Program::new(&[
        "edition 2026; module main { use a; use main; use a; use gone; }",
        "edition 2026; module a { use b; }",
        "edition 2026; module b { use a; use c; }",
        "edition 2026; module c { use main; }",
        "edition 2026; module a { }",
    ]);
    let result = program.analyze();
    assert!(result.core().is_none());
    assert_eq!(
        program.report(&result),
        [
            (
                DiagnosticCode::ModuleCycle,
                String::from("module `main` uses itself"),
                String::from("use main;")
            ),
            (
                DiagnosticCode::DuplicateModule,
                String::from("module `a` is used twice"),
                String::from("use a;")
            ),
            (
                DiagnosticCode::UnknownModule,
                String::from("no module named `gone` in this program"),
                String::from("gone")
            ),
            (
                DiagnosticCode::DuplicateModule,
                String::from("duplicate module `a`"),
                String::from("a")
            ),
            (
                DiagnosticCode::ModuleCycle,
                String::from("module cycle `a` -> `b` -> `a`"),
                String::from("use a;")
            ),
            (
                DiagnosticCode::ModuleCycle,
                String::from("module cycle `main` -> `a` -> `b` -> `c` -> `main`"),
                String::from("use main;")
            ),
        ]
    );
    // Each module's namesakes are reported when the search first enters it.
    let duplicate = &result.diagnostics()[3];
    assert_eq!(duplicate.primary_span().source(), program.ids[4]);
    assert_eq!(duplicate.secondary_spans().len(), 1);
    assert_eq!(
        duplicate.secondary_spans()[0].span().source(),
        program.ids[1]
    );
    let twice = &result.diagnostics()[1];
    assert_eq!(twice.primary_span().source(), program.ids[0]);
    assert!(twice.primary_span().start() > twice.secondary_spans()[0].span().start());
    assert_eq!(
        result.diagnostics()[4].primary_span().source(),
        program.ids[2]
    );
    assert_eq!(
        result.diagnostics()[5].primary_span().source(),
        program.ids[3]
    );
    assert_eq!(
        result.diagnostics()[5].label(),
        "this `use` closes the cycle"
    );
}

#[test]
fn qualified_calls_resolve_only_in_used_modules() {
    let program = Program::new(&[
        concat!(
            "edition 2026; module main { use lib; ",
            "spec a() -> Int { other::one() } ",
            "spec b() -> Int { main::a() } ",
            "spec c() -> Int { lib::three() } ",
            "spec d() -> Int { one() } ",
            "spec e() -> Int { lib::fast() } ",
            "spec f() -> Int { lib::two() } ",
            "spec g() -> Word[8] { lib::one() } ",
            "spec h() -> Int { lib::empty() } ",
            "spec i() -> Int { lib::two(lib::one() + 1) + (g() as Int) } ",
            "}"
        ),
        concat!(
            "edition 2026; module lib { ",
            "spec one() -> Int { 1 } ",
            "spec two(x: Int) -> Int { x } ",
            "impl fast() {} ",
            "spec empty() {} ",
            "}"
        ),
    ]);
    let result = program.analyze();
    assert_eq!(
        program.report(&result),
        [
            (
                DiagnosticCode::ModuleNotUsed,
                String::from("module `other` is not used by `main`"),
                String::from("other")
            ),
            (
                DiagnosticCode::ModuleNotUsed,
                String::from("`main` is the calling module"),
                String::from("main")
            ),
            (
                DiagnosticCode::UnknownFunction,
                String::from("no typed `spec` function named `three` in module `lib`"),
                String::from("three")
            ),
            (
                DiagnosticCode::UnknownFunction,
                String::from("no typed `spec` function named `one` in this module"),
                String::from("one")
            ),
            (
                DiagnosticCode::UnknownFunction,
                String::from("no typed `spec` function named `fast` in module `lib`"),
                String::from("fast")
            ),
            (
                DiagnosticCode::ArgumentCountMismatch,
                String::from("`two` takes 1 argument but 0 were supplied"),
                String::from("lib::two()")
            ),
            (
                DiagnosticCode::TypeMismatch,
                String::from("`one` returns `Int`, but `Word[8]` is required here"),
                String::from("lib::one()")
            ),
            (
                DiagnosticCode::UnknownFunction,
                String::from("`spec` function `empty` has no typed body and cannot be called"),
                String::from("empty")
            ),
        ]
    );
    let notes = result
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.notes()[0].as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        notes[..5],
        [
            "declare `use other;` at the head of the module to call its functions",
            "call a function of the same module without a module name, as in `f(x)`",
            "a qualified call names a typed `spec` of the used module",
            "the used module `lib` declares `one`; call it as `lib::one(...)`",
            "`impl` functions have no semantics yet and cannot be called",
        ]
    );
    let empty = &result.diagnostics()[7];
    assert_eq!(empty.secondary_spans()[0].span().source(), program.ids[1]);
    assert_eq!(program.slice(empty.secondary_spans()[0].span()), "empty");
}

#[test]
fn each_module_is_checked_even_when_a_module_it_uses_has_errors() {
    let program = Program::new(&[
        concat!(
            "edition 2026; module main { use lib; ",
            "spec p() -> Int { q() } ",
            "spec q() -> Int { p() + lib::fine() } ",
            "spec r() -> Int { lib::gone() } ",
            "}"
        ),
        "edition 2026; module lib { spec bad() -> Int { nope } spec fine() -> Int { 1 } }",
    ]);
    let result = program.analyze();
    assert_eq!(
        program.report(&result),
        [
            (
                DiagnosticCode::UnknownParameter,
                String::from("`nope` is not a parameter of `bad`"),
                String::from("nope")
            ),
            (
                DiagnosticCode::UnknownFunction,
                String::from("no typed `spec` function named `gone` in module `lib`"),
                String::from("gone")
            ),
            (
                DiagnosticCode::CallCycle,
                String::from("call cycle `p` -> `q` -> `p`"),
                String::from("p()")
            ),
        ]
    );
    assert_eq!(
        result.diagnostics()[0].primary_span().source(),
        program.ids[1]
    );
}

#[test]
fn rejects_foreign_use_and_qualifier_spans_and_foreign_modules() {
    let text = "edition 2026; module main { use lib; spec f() -> Int { lib::one() } }";
    let lib = "edition 2026; module lib { spec one() -> Int { 1 } }";
    let first = Program::new(&[text, lib]);
    let second = Program::new(&[text, lib]);
    fn call_of(ast: &mut SyntaxTree) -> &mut CallExpression {
        match &mut typed_body_mut(ast).expression.kind {
            ExpressionKind::Call(call) => call,
            _ => unreachable!(),
        }
    }
    let foreign_use = second.asts[0].module.uses[0].clone();
    let foreign_module = second.asts[0].clone();
    let mut foreign_qualifier = second.asts[0].clone();
    let foreign_qualifier_span = call_of(&mut foreign_qualifier)
        .module
        .as_ref()
        .unwrap()
        .span;
    let mutations: [&dyn Fn(&mut SyntaxTree); 3] = [
        &|ast| ast.module.uses[0].span = foreign_use.span,
        &|ast| ast.module.uses[0].name.span = foreign_use.name.span,
        &|ast| call_of(ast).module.as_mut().unwrap().span = foreign_qualifier_span,
    ];
    let modules = first.modules();
    for (index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.asts[0].clone();
        mutate(&mut ast);
        let result = analyze_program((modules[0].0, &ast), &modules[1..]);
        assert!(result.core().is_none(), "case {index}");
        assert_eq!(
            result
                .diagnostics()
                .iter()
                .map(Diagnostic::code)
                .collect::<Vec<_>>(),
            [DiagnosticCode::InvalidSemanticInput],
            "case {index}"
        );
    }

    // A used module whose tree belongs to another source.
    let result = analyze_program(modules[0], &[(modules[1].0, &foreign_module)]);
    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        DiagnosticCode::InvalidSemanticInput
    );
    assert_eq!(
        result.diagnostics()[0].primary_span().source(),
        first.ids[1]
    );
}

/// `Mod[value]` for a modulus below 2^64.
fn residue_type(value: u64) -> CoreType {
    let exact = ExactInteger::from_u64(value, reserve_range_limbs).unwrap();
    CoreType::Mod(Modulus::new(&exact).unwrap())
}

/// Renders diagnostics as `(code, responsible source, label)`.
fn labelled<'text>(
    fixture: &'text Fixture,
    result: &AnalysisResult,
) -> Vec<(DiagnosticCode, &'text str, String)> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code(),
                fixture.source().slice(diagnostic.primary_span()).unwrap(),
                diagnostic.label().to_owned(),
            )
        })
        .collect()
}

#[test]
fn residues_build_typed_core_in_postorder() {
    let (fixture, core) = accepted(concat!(
        "  type F = Mod[7];\n",
        "  spec f(x: F, w: Word[8]) -> F { -1 + x * (w as F) - -x }\n",
        "  spec g(x: F) -> Int { (x / 3) as Int }\n",
        "  spec h(x: F) -> Bool { x != 6 }\n",
    ));
    let f = residue_type(7);
    assert_eq!(
        core.functions
            .iter()
            .map(|function| (function.parameters.clone(), function.result_type.clone()))
            .collect::<Vec<_>>(),
        [
            (vec![f.clone(), CoreType::Word8], f.clone()),
            (vec![f.clone()], CoreType::Int),
            (vec![f.clone()], CoreType::Bool),
        ]
    );
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        core_nodes(&fixture, &core.functions[0]),
        owned(&[
            ("literal 6", "-1", f.clone()),
            ("parameter 0", "x", f.clone()),
            ("parameter 1", "w", CoreType::Word8),
            ("convert from Word[8]", "w as F", f.clone()),
            ("infix *", "x * (w as F)", f.clone()),
            ("infix +", "-1 + x * (w as F)", f.clone()),
            ("parameter 0", "x", f.clone()),
            ("prefix -", "-x", f.clone()),
            ("infix -", "-1 + x * (w as F) - -x", f.clone()),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[1]),
        owned(&[
            ("parameter 0", "x", f.clone()),
            ("literal 3", "3", f.clone()),
            ("infix /", "x / 3", f.clone()),
            ("convert from Mod[7]", "(x / 3) as Int", CoreType::Int),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[2]),
        owned(&[
            ("parameter 0", "x", f.clone()),
            ("literal 6", "6", f),
            ("compare != on Mod[7]", "x != 6", CoreType::Bool),
        ])
    );
}

#[test]
fn moduli_are_constants_from_two_through_two_to_the_521_minus_one() {
    // Moduli are values: every spelling of 7 names one type, and the
    // extreme moduli are admitted.
    let (_, core) = accepted(concat!(
        "  spec f(x: Mod[7]) -> Mod[3 + 4] { x }\n",
        "  spec g(x: Mod[0b111]) -> Mod[(1 << 3) - 1] { f(x) * f(x) }\n",
        "  spec two(x: Mod[2]) -> Mod[1 + 1] { x }\n",
        "  spec wide(x: Mod[(1 << 521) - 1]) -> Mod[(1 << 521) - 1] { x }\n",
        "  spec p256(x: Mod[(1 << 256) - (1 << 224) + (1 << 192) + (1 << 96) - 1]) -> Bool { \
         x == -3 }\n",
    ));
    let seven = residue_type(7);
    assert_eq!(core.functions[0].parameters, std::slice::from_ref(&seven));
    assert_eq!(core.functions[1].result_type, seven);
    assert_eq!(core.functions[2].result_type, residue_type(2));
    assert_eq!(
        core.functions
            .iter()
            .skip(3)
            .map(|function| function.parameters[0].to_string())
            .collect::<Vec<_>>(),
        [
            "Mod[(1 << 521) - 1]",
            "Mod[0xffffffff00000001000000000000000000000000ffffffffffffffffffffffff]",
        ]
    );
    let Some(CoreType::Mod(wide)) = core.functions[3].parameters.first().cloned() else {
        panic!("expected a residue type");
    };
    assert_eq!(wide.bits(), MAX_MODULUS_BITS);

    let (fixture, result) = rejected(concat!(
        "  spec a(x: Mod[1]) -> Int { 0 }\n",
        "  spec b(x: Mod[0 - 5]) -> Int { 0 }\n",
        "  spec c(x: Mod[1 << 521]) -> Int { 0 }\n",
        "  spec d(x: Mod[q]) -> Int { 0 }\n",
        "  spec e(x: Mod[3329 / 1]) -> Int { 0 }\n",
        "  spec f(x: Mod[-(7)]) -> Int { 0 }\n",
        "  spec g(x: Mod) -> Int { 0 }\n",
        "  spec h(x: Mod[1 << 16385]) -> Int { 0 }\n",
        "  spec i(x: Mod[(1 << 16383) * 4]) -> Int { 0 }\n",
        "  spec j(x: Mod[0 - (1 << 100)]) -> Int { 0 }\n",
        "  spec k(x: Word[8]) -> Int { (x as Mod[f(1)]) as Int }\n",
        "  spec l() -> Int { (for i in 0..1 with s: Mod[0] = 0 { s }) as Int }\n",
    ));
    // Every modulus is evaluated before any type is resolved, so the
    // missing modulus, found when `g`'s type is resolved, comes last.
    assert_eq!(
        labelled(&fixture, &result),
        [
            (
                DiagnosticCode::InvalidModulus,
                "1",
                String::from("this modulus is 1")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "0 - 5",
                String::from("this modulus is -5")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "1 << 521",
                String::from("this modulus has 522 bits")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "q",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "3329 / 1",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "-(7)",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "16385",
                String::from("a shift amount in a modulus is from 0 through 16384")
            ),
            (
                DiagnosticCode::IntegerMagnitudeLimit,
                "(1 << 16383) * 4",
                String::from("this value of the modulus is too large")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "0 - (1 << 100)",
                String::from("this modulus is negative")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "f(1)",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "0",
                String::from("this modulus is 0")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "Mod",
                String::from("missing modulus")
            ),
        ]
    );
    assert!(result.diagnostics.iter().all(|diagnostic| {
        diagnostic.code() != DiagnosticCode::InvalidModulus || diagnostic.notes() == [MODULUS_NOTE]
    }));
    assert_eq!(
        result.diagnostics[0].message(),
        "a modulus must be a constant from 2 through 2^521 - 1"
    );
}

#[test]
fn moduli_written_within_a_modulus_are_evaluated_too() {
    // A modulus that is not a constant is reported at its first part
    // that is not, and every modulus of a conversion or loop written
    // within it is evaluated after it, in source order.
    let (fixture, result) = rejected(concat!(
        "  type T = Mod[((0 as Mod[7]) as Int) + ((0 as Mod[(0 as Mod[0]) as Int]) as Int)];\n",
        "  spec a(x: Mod[(0 as Mod[1]) as Int]) -> Int { 0 }\n",
        "  spec b(x: Word[8]) -> Int { \
         (x as Mod[(for i in 0..1 with s: Mod[0 - 5] = 0 { s }) as Int]) as Int }\n",
    ));
    assert_eq!(
        labelled(&fixture, &result),
        [
            (
                DiagnosticCode::InvalidModulus,
                "(0 as Mod[7]) as Int",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "(0 as Mod[0]) as Int",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "0",
                String::from("this modulus is 0")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "(0 as Mod[1]) as Int",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "1",
                String::from("this modulus is 1")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "(for i in 0..1 with s: Mod[0 - 5] = 0 { s }) as Int",
                String::from("not a constant integer expression")
            ),
            (
                DiagnosticCode::InvalidModulus,
                "0 - 5",
                String::from("this modulus is -5")
            ),
        ]
    );
}

#[test]
fn type_names_resolve_in_declaration_order_within_their_module() {
    let (_, core) = accepted(concat!(
        "  type F = Mod[7];\n",
        "  type Pair = F^2;\n",
        "  type Count = Int;\n",
        "  spec f(p: Pair, n: Count) -> F { p[1] + (n as F) }\n",
    ));
    let f = residue_type(7);
    assert_eq!(
        core.functions[0].parameters,
        [array_of(f.clone(), 2), CoreType::Int]
    );
    assert_eq!(core.functions[0].result_type, f);

    let (fixture, result) = rejected(concat!(
        "  type Int = Word[8];\n",
        "  type Bool = Int;\n",
        "  type Word = Int;\n",
        "  type Mod = Mod[7];\n",
        "  type K = Mod[7];\n",
        "  type K = Mod[11];\n",
        "  type L = M;\n",
        "  type M = Word[8];\n",
        "  type Block = Word[32]^16;\n",
        "  type Blocks = Block^2;\n",
        "  type N = Mod[1];\n",
        "  spec f(x: Block^2) -> K { 0 }\n",
        "  spec g(x: Unknown) -> K { 0 }\n",
        "  spec h(x: K[3]) -> K { 0 }\n",
        "  spec i(x: N) -> K { 0 }\n",
    ));
    // Moduli are evaluated first, then declarations are resolved in
    // order, then functions are checked; `N`'s use is not reported again.
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::InvalidModulus,
                "1",
                String::from("a modulus must be a constant from 2 through 2^521 - 1")
            ),
            (
                DiagnosticCode::DuplicateTypeName,
                "Int",
                String::from("`Int` is a built-in type")
            ),
            (
                DiagnosticCode::DuplicateTypeName,
                "Bool",
                String::from("`Bool` is a built-in type")
            ),
            (
                DiagnosticCode::DuplicateTypeName,
                "Word",
                String::from("`Word` is a built-in type")
            ),
            (
                DiagnosticCode::DuplicateTypeName,
                "Mod",
                String::from("`Mod` is a built-in type")
            ),
            (
                DiagnosticCode::DuplicateTypeName,
                "K",
                String::from("duplicate type name `K`")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "M",
                String::from("unsupported declared type `M`")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Block^2",
                String::from("`Block` is an array type, so this is an array of arrays")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Block^2",
                String::from("`Block` is an array type, so this is an array of arrays")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Unknown",
                String::from("unsupported parameter type `Unknown`")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "K[3]",
                String::from("unsupported parameter type `K`")
            ),
        ]
    );
    let duplicate = &result.diagnostics[5];
    assert_eq!(
        duplicate
            .secondary_spans()
            .iter()
            .map(|secondary| (
                fixture.source().slice(secondary.span()).unwrap(),
                secondary.label()
            ))
            .collect::<Vec<_>>(),
        [("K", "first declaration is here")]
    );
    assert_eq!(
        result.diagnostics[6].notes(),
        [
            "`M` is declared by a later `type` declaration; a `type` declaration uses only the \
          names declared before it"
        ]
    );
    assert_eq!(
        result.diagnostics[1].notes(),
        ["the built-in types are `Int`, `Bool`, `Word[n]`, and `Mod[m]`"]
    );
}

#[test]
fn residue_literals_lie_strictly_between_minus_the_modulus_and_the_modulus() {
    let (fixture, core) = accepted(concat!(
        "  spec f() -> Mod[7]^5 { [-6, -1, 0, -0, 6] }\n",
        "  spec g() -> Mod[(1 << 130) - 5] { -0x3fffffffffffffffffffffffffffffffa }\n",
    ));
    let rendered = |function: &CoreFunction| {
        core_nodes(&fixture, function)
            .into_iter()
            .map(|(operation, _, _)| operation)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        rendered(&core.functions[0]),
        [
            "literal 1",
            "literal 6",
            "literal 0",
            "literal 0",
            "literal 6",
            "array of 5"
        ]
    );
    assert_eq!(rendered(&core.functions[1]), ["literal 1"]);

    let (fixture, result) = rejected(concat!(
        "  spec a() -> Mod[7] { 7 }\n",
        "  spec b() -> Mod[7] { -7 }\n",
        "  spec c() -> Mod[(1 << 130) - 5] { 0x3fffffffffffffffffffffffffffffffb }\n",
        "  spec d() -> Mod[7] { true }\n",
    ));
    assert_eq!(
        labelled(&fixture, &result),
        [
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "7",
                String::from("the literal's magnitude is not less than the modulus")
            ),
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "7",
                String::from("the literal's magnitude is not less than the modulus")
            ),
            (
                DiagnosticCode::WordLiteralOutOfRange,
                "0x3fffffffffffffffffffffffffffffffb",
                String::from("the literal's magnitude is not less than the modulus")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "true",
                String::from("expected `Mod[7]`")
            ),
        ]
    );
    assert_eq!(
        result.diagnostics[0].message(),
        "literal is outside the range of `Mod[7]`"
    );
}

#[test]
fn residues_have_ring_operators_and_equality_but_no_order_remainder_or_bits() {
    let members = concat!(
        "  spec a(x: Mod[7]) -> Bool { x < x }\n",
        "  spec b(x: Mod[7]) -> Bool { x >= 1 }\n",
        "  spec c(x: Mod[7]) -> Mod[7] { x % 2 }\n",
        "  spec d(x: Mod[7]) -> Mod[7] { ~x }\n",
        "  spec e(x: Mod[7]) -> Mod[7] { !x }\n",
        "  spec f(x: Mod[7]) -> Mod[7] { x & 1 }\n",
        "  spec g(x: Mod[7]) -> Mod[7] { x << 1 }\n",
        "  spec h(x: Mod[7]) -> Mod[7] { x && x }\n",
    );
    let (fixture, result) = rejected(members);
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .map(|(code, source, _)| (code, source))
            .collect::<Vec<_>>(),
        [
            (DiagnosticCode::UnsupportedOperator, "<"),
            (DiagnosticCode::UnsupportedOperator, ">="),
            (DiagnosticCode::UnsupportedOperator, "%"),
            (DiagnosticCode::UnsupportedOperator, "~"),
            (DiagnosticCode::UnsupportedOperator, "!"),
            (DiagnosticCode::UnsupportedOperator, "&"),
            (DiagnosticCode::UnsupportedOperator, "<<"),
            (DiagnosticCode::UnsupportedOperator, "&&"),
        ]
    );
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.notes()[0].as_str())
            .collect::<Vec<_>>(),
        [
            "residues are compared with `==` and `!=`; they have no order, so compare least \
             residues, such as `(x as Int) < (y as Int)`",
            "residues are compared with `==` and `!=`; they have no order, so compare least \
             residues, such as `(x as Int) < (y as Int)`",
            "a residue is already reduced; `%` applies to `Int` and word values, such as \
             `(x as Int) % 16`",
            "bitwise operators apply only to `Word[n]` values",
            "`!` negates a `Bool`; `-` negates an `Int` or a residue",
            "bitwise operators apply only to `Word[n]` values",
            "shifts and rotations apply only to `Word[n]` values",
            "`&&` and `||` apply to `Bool` values; `&` and `|` are the bitwise operators on \
             words",
        ]
    );

    // Two moduli are two types, whatever their values share.
    let (fixture, result) = rejected(concat!(
        "  spec a(x: Mod[7], y: Mod[14]) -> Mod[7] { x + y }\n",
        "  spec b(y: Mod[14]) -> Mod[7] { (y as Mod[7]) + a(y, y) }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::TypeMismatch,
                "y",
                String::from("`y` has type `Mod[14]`, but `Mod[7]` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "y",
                String::from("`y` has type `Mod[14]`, but `Mod[7]` is required here")
            ),
        ]
    );

    // Every ring operator, `==`, and `!=` are defined for a residue.
    accepted(concat!(
        "  type F = Mod[(1 << 255) - 19];\n",
        "  spec f(x: F, y: F) -> F { -(x + y - x * y) / (y - x) }\n",
        "  spec g(x: F, y: F) -> Bool { (x == y) != (-x != y) }\n",
    ));
}

#[test]
fn residues_convert_to_and_from_numbers_and_index_through_least_residues() {
    let (_, core) = accepted(concat!(
        "  spec f(x: Mod[7], w: Word[8], n: Int) -> Int^3 { \
         [x as Int, (w as Mod[7]) as Int, ((n as Mod[7]) as Word[64]) as Int] }\n",
        "  spec g(x: Mod[7]) -> Word[8] { \
         let t: Word[8]^7 = [1, 2, 3, 4, 5, 6, 7]; t[x as Int] + t[x as Word[8]] }\n",
        "  spec h(x: Mod[256]) -> Word[8] { \
         let t: Word[8]^256 = [0; 256]; t[x as Word[8]] ^ t[x as Int] }\n",
    ));
    assert_eq!(core.functions.len(), 3);

    let (fixture, result) = rejected(concat!(
        "  spec a(x: Mod[7]) -> Word[8] { let t: Word[8]^6 = [0; 6]; t[x as Int] }\n",
        "  spec b(x: Mod[7]) -> Word[8] { let t: Word[8]^6 = [0; 6]; t[x as Word[8]] }\n",
        "  spec c(x: Mod[300]) -> Word[8] { let t: Word[8]^255 = [0; 255]; t[x as Word[8]] }\n",
        "  spec d(x: Mod[7]) -> Word[8] { let t: Word[8]^7 = [0; 7]; t[x] }\n",
        "  spec e(x: Mod[7]) -> Bool { x as Bool }\n",
    ));
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .map(|(code, source, _)| (code, source))
            .collect::<Vec<_>>(),
        [
            (DiagnosticCode::IndexOutOfRange, "x as Int"),
            (DiagnosticCode::IndexOutOfRange, "x as Word[8]"),
            (DiagnosticCode::IndexOutOfRange, "x as Word[8]"),
            (DiagnosticCode::TypeMismatch, "x"),
            (DiagnosticCode::UnsupportedOperator, "as"),
        ]
    );
}

#[test]
fn residue_types_cross_modules_by_value_and_type_names_stay_in_their_module() {
    let lib = concat!(
        "edition 2026; module lib { type F = Mod[(1 << 130) - 5]; ",
        "spec square(x: F) -> F { x * x } }",
    );
    let main = concat!(
        "edition 2026; module main { use lib; type P = Mod[(1 << 130) - 5]; ",
        "spec f(x: P) -> P { lib::square(x) + lib::square(-1) } }",
    );
    let program = Program::new(&[main, lib]);
    let modules = program.modules();
    let result = analyze_program(modules[0], &modules[1..]);
    assert_eq!(result.diagnostics(), []);

    let main = concat!(
        "edition 2026; module main { use lib; ",
        "spec f(x: F) -> Int { 0 } }",
    );
    let program = Program::new(&[main, lib]);
    let modules = program.modules();
    let result = analyze_program(modules[0], &modules[1..]);
    assert_eq!(
        result
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.code(), diagnostic.message()))
            .collect::<Vec<_>>(),
        [(
            DiagnosticCode::UnsupportedType,
            "unsupported parameter type `F`"
        )]
    );
}

#[test]
fn modulus_events_follow_the_normative_accounting() {
    // Beside `spec f(x: Int) -> Int { 0 }`, the modulus `3 + 4` costs
    // one event for each of its three nodes and a prefix and a digit
    // event for each literal: 7 events. It adds no Core node.
    let events_for = |members: &str| {
        let fixture = module(members);
        (1..200)
            .find(|events| {
                fixture
                    .analyze_with(Limits {
                        events: *events,
                        ..Limits::DEFAULT
                    })
                    .core
                    .is_some()
            })
            .unwrap()
    };
    let plain = events_for("  spec f(x: Int) -> Int { 0 }\n");
    let residue = events_for("  spec f(x: Mod[3 + 4]) -> Int { 0 }\n");
    assert_eq!(residue, plain + 7);
    // A `type` declaration costs one event for its name's lookup and
    // the events of its type; a use of the name costs what `Int` does.
    let named = events_for("  type F = Mod[3 + 4];\n  spec f(x: F) -> Int { 0 }\n");
    assert_eq!(named, plain + 7 + 2);
}

#[test]
fn modulus_storage_failures_return_no_partial_core() {
    for (members, responsible) in [
        ("  spec f(x: Mod[3 + 4]) -> Int { 0 }\n", "3 + 4"),
        ("  spec f() -> Mod[7] { -1 }\n", "-1"),
    ] {
        let fixture = module(members);
        let analyze_with_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_range_limbs = |_, _| false;
            analyzer.run()
        };
        let first = analyze_with_failure();
        assert_eq!(first, analyze_with_failure());
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1, "{members}");
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some(responsible)
        );
        assert_eq!(
            diagnostic.label(),
            "exact integer storage allocation failed"
        );
    }
}

#[test]
fn rejects_foreign_spans_in_type_declarations_and_moduli() {
    let text = "edition 2026; module values { type F = Mod[7]; \
                spec value(x: F) -> Int { \
                ((for i in 0..1 with s: Mod[11] = 0 { s }) as Mod[13]) as Int } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    fn conversion_mut(ast: &mut SyntaxTree) -> &mut ConversionExpression {
        let ExpressionKind::Conversion(outer) = &mut typed_body_mut(ast).expression.kind else {
            unreachable!();
        };
        let ExpressionKind::Parenthesized(inner) = &mut outer.operand.kind else {
            unreachable!();
        };
        let ExpressionKind::Conversion(conversion) = &mut inner.kind else {
            unreachable!();
        };
        conversion
    }
    fn loop_mut(ast: &mut SyntaxTree) -> &mut LoopExpression {
        let ExpressionKind::Parenthesized(inner) = &mut conversion_mut(ast).operand.kind else {
            unreachable!();
        };
        let ExpressionKind::Loop(r#loop) = &mut inner.kind else {
            unreachable!();
        };
        r#loop
    }
    let mut foreign = second.ast.clone();
    let declaration = foreign.module.types[0].clone();
    let target = conversion_mut(&mut foreign).target.clone();
    let accumulator = named_of(&loop_mut(&mut foreign).accumulator).ty.clone();
    let modulus_span = |ty: &TypeSyntax| ty.modulus().unwrap().span;
    type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
    let mutations: Vec<Mutation<'_>> = vec![
        Box::new(|ast| ast.module.types[0].span = declaration.span),
        Box::new(|ast| ast.module.types[0].name.span = declaration.name.span),
        Box::new(|ast| ast.module.types[0].ty.span = declaration.ty.span),
        Box::new(|ast| {
            ast.module.types[0].ty.modulus.as_mut().unwrap().span = modulus_span(&declaration.ty);
        }),
        Box::new(|ast| {
            conversion_mut(ast).target.modulus.as_mut().unwrap().span = modulus_span(&target);
        }),
        Box::new(|ast| {
            named_mut(&mut loop_mut(ast).accumulator)
                .ty
                .modulus
                .as_mut()
                .unwrap()
                .span = modulus_span(&accumulator);
        }),
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}

/// Renders a block's bindings as `(name, source, type, end)`.
fn block_bindings<'text>(
    fixture: &'text Fixture,
    bindings: &[CoreBinding],
) -> Vec<(String, &'text str, CoreType, u32)> {
    bindings
        .iter()
        .map(|binding| {
            (
                binding.name().to_owned(),
                fixture.source().slice(binding.span()).unwrap(),
                binding.ty(),
                binding.end(),
            )
        })
        .collect()
}

#[test]
fn blocks_build_typed_core_with_each_binding_before_its_value() {
    let (fixture, core) = accepted(concat!(
        "  spec mix(a: Word[32], b: Word[32]) -> Word[32] {\n",
        "    for i in 0..4 with s: Word[32] = a {\n",
        "      let t: Word[32] = s ^ b; let u: Word[32] = t >>> 7; u + t\n",
        "    }\n",
        "  }\n",
        "  spec pick(c: Bool, x: Int) -> Int {\n",
        "    if c { let y: Int = x + 1; y * y } else if x < 0 { 0 } else { let z: Int = x; z - 1 }\n",
        "  }\n",
        "  spec nested(x: Int) -> Int {\n",
        "    let w: Int = 2;\n",
        "    for i in 0..2 with s: Int = x {\n",
        "      let t: Int = s + i;\n",
        "      for j in 0..2 with u: Int = t { let v: Int = u + t * w; if v < 0 { let n: Int = -v; n } else { v } }\n",
        "    }\n",
        "  }\n",
    ));
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    let word = CoreType::Word32;

    // Each binding's value subtree comes first, in source order, and the
    // step's value reads the bindings from their slots.
    let mix = &core.functions[0];
    assert_eq!(
        block_bindings(&fixture, mix.loops[0].bindings()),
        [
            (
                String::from("t"),
                "let t: Word[32] = s ^ b;",
                word.clone(),
                3
            ),
            (
                String::from("u"),
                "let u: Word[32] = t >>> 7;",
                word.clone(),
                5
            ),
        ]
    );
    assert_eq!(
        expression_nodes(&fixture, mix.loops[0].step()),
        owned(&[
            ("accumulator of loop #0", "s", word.clone()),
            ("parameter 1", "b", word.clone()),
            ("infix ^", "s ^ b", word.clone()),
            ("binding 0 of loop #0", "t", word.clone()),
            ("shift >>> 7", "t >>> 7", word.clone()),
            ("binding 1 of loop #0", "u", word.clone()),
            ("binding 0 of loop #0", "t", word.clone()),
            ("infix +", "u + t", word),
        ])
    );

    // A branch's bindings belong to its conditional: the last `else`
    // of a chain is the last arm's.
    let pick = &core.functions[1];
    assert_eq!(pick.conditionals.len(), 2);
    assert_eq!(
        block_bindings(&fixture, pick.conditionals[0].then_bindings()),
        [(String::from("y"), "let y: Int = x + 1;", CoreType::Int, 3)]
    );
    assert_eq!(
        expression_nodes(&fixture, pick.conditionals[0].then_branch()),
        owned(&[
            ("parameter 1", "x", CoreType::Int),
            ("literal 1", "1", CoreType::Int),
            ("infix +", "x + 1", CoreType::Int),
            ("binding 0 of branch #0", "y", CoreType::Int),
            ("binding 0 of branch #0", "y", CoreType::Int),
            ("infix *", "y * y", CoreType::Int),
        ])
    );
    assert!(pick.conditionals[0].else_bindings().is_empty());
    assert!(pick.conditionals[1].then_bindings().is_empty());
    assert_eq!(
        block_bindings(&fixture, pick.conditionals[1].else_bindings()),
        [(String::from("z"), "let z: Int = x;", CoreType::Int, 1)]
    );
    assert_eq!(
        expression_nodes(&fixture, pick.conditionals[1].else_branch()),
        owned(&[
            ("parameter 1", "x", CoreType::Int),
            ("binding 0 of branch #1", "z", CoreType::Int),
            ("literal 1", "1", CoreType::Int),
            ("infix -", "z - 1", CoreType::Int),
        ])
    );

    // An inner step sees the outer step's bindings, and a branch inside
    // it sees both.
    let nested = &core.functions[2];
    assert_eq!(
        block_bindings(&fixture, nested.loops[0].bindings()),
        [(String::from("t"), "let t: Int = s + i;", CoreType::Int, 3)]
    );
    assert_eq!(
        block_bindings(&fixture, nested.loops[1].bindings()),
        [(
            String::from("v"),
            "let v: Int = u + t * w;",
            CoreType::Int,
            5
        )]
    );
    assert_eq!(
        expression_nodes(&fixture, nested.loops[1].step())
            .into_iter()
            .take(5)
            .collect::<Vec<_>>(),
        owned(&[
            ("accumulator of loop #1", "u", CoreType::Int),
            ("binding 0 of loop #0", "t", CoreType::Int),
            ("local 0", "w", CoreType::Int),
            ("infix *", "t * w", CoreType::Int),
            ("infix +", "u + t * w", CoreType::Int),
        ])
    );
    assert_eq!(
        expression_nodes(&fixture, nested.conditionals[0].then_branch()),
        owned(&[
            ("binding 0 of loop #1", "v", CoreType::Int),
            ("prefix -", "-v", CoreType::Int),
            ("binding 0 of branch #0", "n", CoreType::Int),
        ])
    );
    assert_eq!(
        conditional_headers(&fixture, nested)
            .into_iter()
            .map(|(_, _, visible, scope)| (visible, scope))
            .collect::<Vec<_>>(),
        [(1, vec![0, 1])]
    );
}

#[test]
fn unresolved_block_binding_types_are_reported_once_without_cascades() {
    // As for a body's binding, the value of a block binding whose type
    // does not resolve is not checked, and its uses are not reported;
    // the block's other parts still are.
    let (fixture, result) = rejected(concat!(
        "  spec step(x: Word[32]) -> Word[32] {\n",
        "    for i in 0..2 with s: Word[32] = x {\n",
        "      let t: Wide = missing;\n",
        "      let u: Word[32] = t;\n",
        "      s ^ u ^ gone\n",
        "    }\n",
        "  }\n",
        "  spec branch(c: Bool) -> Int {\n",
        "    if c { let t: Word[12] = missing; t } else { absent }\n",
        "  }\n",
    ));
    assert_eq!(
        reported(&fixture, &result)
            .into_iter()
            .map(|(code, source, _)| (code, source))
            .collect::<Vec<_>>(),
        [
            (DiagnosticCode::UnsupportedType, "Wide"),
            (DiagnosticCode::UnknownParameter, "gone"),
            (DiagnosticCode::UnsupportedWordWidth, "12"),
            (DiagnosticCode::UnknownParameter, "absent"),
        ]
    );
}

#[test]
fn block_names_are_unique_and_in_scope_only_within_their_block() {
    let (fixture, result) = rejected(concat!(
        "  spec parameter(x: Int) -> Int { for i in 0..2 with s: Int = x { let x: Int = s; x } }\n",
        "  spec index() -> Int { for i in 0..2 with s: Int = 0 { let i: Int = s; i } }\n",
        "  spec twice(c: Bool) -> Int { if c { let t: Int = 1; let t: Int = 2; t } else { 0 } }\n",
        "  spec body(c: Bool) -> Int { let t: Int = 1; if c { let t: Int = 2; t } else { t } }\n",
        "  spec inner() -> Int { for i in 0..2 with s: Int = 0 { let j: Int = s; for j in 0..2 with u: Int = s { u } } }\n",
        "  spec early() -> Int { for i in 0..2 with s: Int = 0 { let a: Int = b; let b: Int = s; a } }\n",
        "  spec outside(c: Bool) -> Int { if c { let t: Int = 1; t } else { t } }\n",
        "  spec after() -> Int { let a: Int = for i in 0..2 with s: Int = 0 { let t: Int = s; t }; t }\n",
        "  spec typed(x: Word[8]) -> Word[32] { if true { let t: Word[8] = x; t } else { 0 } }\n",
        "  spec siblings(c: Bool) -> Int { if c { let t: Int = 1; t } else { let t: Int = 2; t } }\n",
        "  spec steps() -> Int { for i in 0..2 with s: Int = 0 { let t: Int = s; t } + for j in 0..2 with u: Int = 0 { let t: Int = u; t } }\n",
    ));
    let duplicate = "each parameter, binding, loop index, and accumulator in scope has \
                     its own name; Orange has no shadowing";
    let outside = "a binding of a loop's step or a branch is in scope only within that \
                   step or branch";
    let rows = [
        // A block's binding repeats no name in scope: not a parameter,
        // a loop index, an earlier binding of its block or of the body.
        (
            DiagnosticCode::DuplicateBinding,
            "x",
            "duplicate name `x`",
            Some(("x", "the parameter is here")),
            duplicate,
        ),
        (
            DiagnosticCode::DuplicateBinding,
            "i",
            "duplicate name `i`",
            Some(("i", "the loop index is here")),
            duplicate,
        ),
        (
            DiagnosticCode::DuplicateBinding,
            "t",
            "duplicate name `t`",
            Some(("t", "the binding is here")),
            duplicate,
        ),
        (
            DiagnosticCode::DuplicateBinding,
            "t",
            "duplicate name `t`",
            Some(("t", "the binding is here")),
            duplicate,
        ),
        // A loop inside a block does not reuse the block's names either.
        (
            DiagnosticCode::DuplicateBinding,
            "j",
            "duplicate name `j`",
            Some(("j", "the binding is here")),
            duplicate,
        ),
        (
            DiagnosticCode::UnknownParameter,
            "b",
            "`b` is used before it is bound",
            Some(("b", "the binding is here")),
            "a binding is in scope after its own `;`, for the bindings that follow it and \
             the value of its step or branch",
        ),
        // Outside its step or branch a block's binding is not in scope.
        (
            DiagnosticCode::UnknownParameter,
            "t",
            "`t` is not in scope here",
            Some(("t", "a binding of this name is here")),
            outside,
        ),
        (
            DiagnosticCode::UnknownParameter,
            "t",
            "`t` is not in scope here",
            Some(("t", "a binding of this name is here")),
            outside,
        ),
        (
            DiagnosticCode::TypeMismatch,
            "t",
            "`t` has type `Word[8]`, but `Word[32]` is required here",
            None,
            "Orange has no implicit conversions between types",
        ),
        // Sibling branches and separate steps may each bind a name.
    ];
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| (
                diagnostic.code(),
                fixture.source().slice(diagnostic.primary_span()).unwrap(),
                diagnostic.message(),
                diagnostic.secondary_spans().first().map(|secondary| (
                    fixture.source().slice(secondary.span()).unwrap(),
                    secondary.label()
                )),
                diagnostic.notes()[0].as_str(),
            ))
            .collect::<Vec<_>>(),
        rows
    );
    // The duplicate in `body` cites the body's binding, before it; the
    // name used too early in `early` is bound after it.
    let cited = |index: usize| {
        result.diagnostics[index].secondary_spans()[0]
            .span()
            .start()
            < result.diagnostics[index].primary_span().start()
    };
    assert_eq!(
        (0..8).map(cited).collect::<Vec<_>>(),
        [true, true, true, true, true, false, true, true]
    );
}

#[test]
fn block_events_and_core_nodes_follow_the_normative_accounting() {
    // Lookup and installation (2); the parameter's uniqueness check and
    // name (2); the result's name (1); the loop (1); the bounds `0` (1)
    // and `1` (2); the two loop-name checks (2); the accumulator type's
    // name (1); the initial `x` (1); the binding's uniqueness check and
    // type name (2) and its value `s` (1); and the step's value `t`
    // (1): 17 analysis events. Core is the module, one function node,
    // one result-type node, one parameter-type node, the body nodes `x`
    // and the loop, one loop node, one accumulator-type node, one
    // binding node, one binding-type node, and the step nodes `s` and
    // `t`: 12 nodes, each one more event.
    let step =
        module("  spec f(x: Int) -> Int { for i in 0..1 with s: Int = x { let t: Int = s; t } }\n");
    // The conditional of the S3f accounting (13 events and 9 nodes) with
    // `1` bound in its `then` branch: the binding's uniqueness check and
    // type name (2), its value `1` (3), and the branch's value `t` (1)
    // in place of `1` (3), for 16 events; one binding node, one
    // binding-type node, and the value `1` more: 12 nodes.
    let branch = module("  spec f(c: Bool) -> Int { if c { let t: Int = 1; t } else { 2 } }\n");
    for (fixture, events, nodes) in [(&step, 29, 12), (&branch, 28, 12)] {
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert!(exact.core.is_some());
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label);
        }
    }
}

#[test]
fn a_branch_binding_gives_no_type_where_the_branch_is_a_leaf() {
    // A conversion operand's type comes from its first typed leaf; a
    // branch whose value is its own binding is passed over for the next
    // branch, because that binding is not in scope at the conversion.
    let (fixture, core) = accepted(concat!(
        "  spec widen(c: Bool, x: Word[8]) -> Word[32] {\n",
        "    (if c { let t: Word[8] = x; t } else { x }) as Word[32]\n",
        "  }\n",
        "  spec element(c: Bool, x: Word[8]^2) -> Word[32] {\n",
        "    (if c { let t: Word[8]^2 = x; t[1] } else { x[0] }) as Word[32]\n",
        "  }\n",
    ));
    for function in &core.functions {
        let nodes = core_nodes(&fixture, function);
        assert_eq!(
            nodes
                .last()
                .map(|(operation, _, ty)| (operation.as_str(), ty.clone())),
            Some(("convert from Word[8]", CoreType::Word32))
        );
    }
    let (fixture, result) = rejected(concat!(
        "  spec only(c: Bool, x: Word[8]) -> Word[32] {\n",
        "    (if c { let t: Word[8] = x; t } else { let u: Word[8] = x; u }) as Word[32]\n",
        "  }\n",
        "  spec compared(c: Bool, x: Word[8]) -> Bool {\n",
        "    (if c { let t: Word[8] = x; t } else { let u: Word[8] = x; u + 1 }) == 1\n",
        "  }\n",
        "  spec literal() -> Word[32] { 1 as Word[32] }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UntypedConversionOperand,
                "(if c { let t: Word[8] = x; t } else { let u: Word[8] = x; u })",
                String::from("the operand of `as` has no type of its own")
            ),
            (
                DiagnosticCode::UntypedComparison,
                "(if c { let t: Word[8] = x; t } else { let u: Word[8] = x; u + 1 }) == 1",
                String::from("the operands of `==` have no type of their own")
            ),
            (
                DiagnosticCode::UntypedConversionOperand,
                "1",
                String::from("the operand of `as` has no type of its own")
            ),
        ]
    );
    // The label and note say why: the branches' bindings are out of
    // scope where the type is needed. A bare literal keeps its S3c text.
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.label(), diagnostic.notes()[0].as_str()))
            .collect::<Vec<_>>(),
        [
            (
                "a branch's own bindings are not in scope outside it",
                "bind the conditional's value with a typed `let` first, or convert within \
                 each branch",
            ),
            (
                "a branch's own bindings are not in scope outside it",
                "bind the conditional's value with a typed `let` first, or compare within \
                 each branch",
            ),
            (
                "a literal takes its type from where it is used",
                "write the literal where its type is required, or give it a type with a \
                 `let` binding",
            ),
        ]
    );
}

#[test]
fn rejects_foreign_spans_in_blocks() {
    let text = "edition 2026; module values { \
                spec value(c: Bool, x: Int) -> Int { \
                for i in 0..2 with s: Int = x { let t: Int = s + i; \
                if c { let u: Int = t; u } else { let v: Int = t; v } } } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    fn loop_of(ast: &mut SyntaxTree) -> &mut LoopExpression {
        let ExpressionKind::Loop(r#loop) = &mut typed_body_mut(ast).expression.kind else {
            unreachable!();
        };
        r#loop
    }
    fn conditional_of(r#loop: &mut LoopExpression) -> &mut ConditionalExpression {
        let ExpressionKind::Conditional(conditional) = &mut r#loop.step.kind else {
            unreachable!();
        };
        conditional
    }
    let mut foreign_ast = second.ast.clone();
    let foreign_loop = loop_of(&mut foreign_ast).clone();
    let foreign_step = foreign_loop.step_bindings[0].clone();
    let mut foreign_loop_copy = foreign_loop.clone();
    let foreign_conditional = conditional_of(&mut foreign_loop_copy).clone();
    let foreign_then = foreign_conditional.arms[0].bindings[0].clone();
    let foreign_else = foreign_conditional.otherwise_bindings[0].clone();
    type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
    let mutations: Vec<Mutation<'_>> = vec![
        Box::new(|ast| loop_of(ast).step_bindings[0].span = foreign_step.span),
        Box::new(|ast| {
            named_mut(&mut loop_of(ast).step_bindings[0].pattern)
                .name
                .span = named_of(&foreign_step.pattern).name.span;
        }),
        Box::new(|ast| {
            named_mut(&mut loop_of(ast).step_bindings[0].pattern)
                .ty
                .span = named_of(&foreign_step.pattern).ty.span;
        }),
        Box::new(|ast| loop_of(ast).step_bindings[0].value.span = foreign_step.value.span),
        Box::new(|ast| {
            conditional_of(loop_of(ast)).arms[0].bindings[0].span = foreign_then.span;
        }),
        Box::new(|ast| {
            conditional_of(loop_of(ast)).arms[0].bindings[0].value.span = foreign_then.value.span;
        }),
        Box::new(|ast| {
            named_mut(&mut conditional_of(loop_of(ast)).otherwise_bindings[0].pattern)
                .name
                .span = named_of(&foreign_else.pattern).name.span;
        }),
        Box::new(|ast| {
            named_mut(&mut conditional_of(loop_of(ast)).otherwise_bindings[0].pattern)
                .ty
                .span = named_of(&foreign_else.pattern).ty.span;
        }),
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}

fn tuple_of(elements: &[CoreType]) -> CoreType {
    CoreType::Tuple(TupleType::new(elements).unwrap())
}

#[test]
fn tuples_build_typed_core_with_elements_in_order() {
    let (fixture, core) = accepted(concat!(
        "  spec pair(a: Int, b: Word[8]) -> (Int, Word[8]) { (a + 1, b) }\n",
        "  spec first(p: (Int, Word[8])) -> Int { p.0 }\n",
        "  spec call() -> Word[8] { pair(1, 2).1 }\n",
        "  spec split(p: (Int, Word[8])) -> Int { let (x: Int, y: Word[8]) = p; x + (y as Int) }\n",
        "  spec fold() -> Int {\n",
        "    let (s: Int, t: Int) = for i in 0..3 with (a: Int, b: Int) = (0, 1) {\n",
        "      let (c: Int, d: Int) = (b, a); (c, c + d)\n",
        "    };\n",
        "    t\n",
        "  }\n",
    ));
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    let pair = tuple_of(&[CoreType::Int, CoreType::Word8]);
    let ints = tuple_of(&[CoreType::Int, CoreType::Int]);

    // A tuple is its elements, left to right, and then one tuple node.
    assert_eq!(core.functions[0].result_type, pair);
    assert_eq!(
        core_nodes(&fixture, &core.functions[0]),
        owned(&[
            ("parameter 0", "a", CoreType::Int),
            ("literal 1", "1", CoreType::Int),
            ("infix +", "a + 1", CoreType::Int),
            ("parameter 1", "b", CoreType::Word8),
            ("tuple of 2", "(a + 1, b)", pair.clone()),
        ])
    );
    // `.k` is its base and then one element node, of a name or a call.
    assert_eq!(core.functions[1].parameters[0], pair);
    assert_eq!(
        core_nodes(&fixture, &core.functions[1]),
        owned(&[
            ("parameter 0", "p", pair.clone()),
            ("element 0", "p.0", CoreType::Int),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[2]),
        owned(&[
            ("literal 1", "1", CoreType::Int),
            ("literal 0x02", "2", CoreType::Word8),
            ("call #0 with 2", "pair(1, 2)", pair.clone()),
            ("element 1", "pair(1, 2).1", CoreType::Word8),
        ])
    );
    // A tuple pattern is one binding of the tuple's type, named by its
    // names; each name reads the binding and selects its element.
    let split = &core.functions[3];
    assert_eq!(split.locals.len(), 1);
    assert_eq!(split.locals[0].name(), "(x, y)");
    assert_eq!(
        fixture.source().slice(split.locals[0].name_span()),
        Some("(x: Int, y: Word[8])")
    );
    assert_eq!(split.locals[0].ty(), pair);
    assert_eq!(
        core_nodes(&fixture, split),
        owned(&[
            ("local 0", "x", pair.clone()),
            ("element 0", "x", CoreType::Int),
            ("local 0", "y", pair.clone()),
            ("element 1", "y", CoreType::Word8),
            ("convert from Word[8]", "y as Int", CoreType::Int),
            ("infix +", "x + (y as Int)", CoreType::Int),
        ])
    );
    // Accumulators named by a pattern are one accumulator of the tuple's
    // type, and a step's pattern is one step binding.
    let fold = &core.functions[4];
    assert_eq!(fold.loops[0].ty(), ints);
    assert_eq!(fold.loops[0].accumulator_name(), "(a, b)");
    assert_eq!(
        block_bindings(&fixture, fold.loops[0].bindings()),
        [(
            String::from("(c, d)"),
            "let (c: Int, d: Int) = (b, a);",
            ints.clone(),
            5
        )]
    );
    assert_eq!(
        expression_nodes(&fixture, fold.loops[0].step()),
        owned(&[
            ("accumulator of loop #0", "b", ints.clone()),
            ("element 1", "b", CoreType::Int),
            ("accumulator of loop #0", "a", ints.clone()),
            ("element 0", "a", CoreType::Int),
            ("tuple of 2", "(b, a)", ints.clone()),
            ("binding 0 of loop #0", "c", ints.clone()),
            ("element 0", "c", CoreType::Int),
            ("binding 0 of loop #0", "c", ints.clone()),
            ("element 0", "c", CoreType::Int),
            ("binding 0 of loop #0", "d", ints.clone()),
            ("element 1", "d", CoreType::Int),
            ("infix +", "c + d", CoreType::Int),
            ("tuple of 2", "(c, c + d)", ints.clone()),
        ])
    );
    assert_eq!(fold.locals[0].name(), "(s, t)");
    assert_eq!(
        core_nodes(&fixture, fold)
            .into_iter()
            .rev()
            .take(2)
            .collect::<Vec<_>>(),
        owned(&[("element 1", "t", CoreType::Int), ("local 0", "t", ints),])
    );
}

#[test]
fn tuple_pattern_names_are_unique_and_scoped_like_bindings() {
    let (fixture, result) = rejected(concat!(
        "  spec within() -> Int { let (a: Int, a: Int) = (1, 2); a }\n",
        "  spec parameter(a: Int) -> Int { let (a: Int, b: Int) = (1, 2); b }\n",
        "  spec binding() -> Int { let b: Int = 1; let (a: Int, b: Int) = (1, 2); a }\n",
        "  spec index() -> (Int, Int) { for i in 0..2 with (i: Int, s: Int) = (0, 0) { (s, s) } }\n",
        "  spec step() -> (Int, Int) {\n",
        "    for i in 0..2 with (a: Int, s: Int) = (0, 0) { let (t: Int, a: Int) = (s, s); (t, a) }\n",
        "  }\n",
        "  spec itself() -> Int { let (a: Int, b: Int) = (1, a); b }\n",
        "  spec after() -> Int { let r: (Int, Int) = for i in 0..2 with (a: Int, b: Int) = (0, 1) { (b, a) }; a }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::DuplicateBinding,
                "a",
                String::from("duplicate binding `a`")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "a",
                String::from("duplicate binding `a`")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "b",
                String::from("duplicate binding `b`")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "i",
                String::from("duplicate name `i`")
            ),
            (
                DiagnosticCode::DuplicateBinding,
                "a",
                String::from("duplicate name `a`")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "a",
                String::from("`a` is used before it is bound")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "a",
                String::from("`a` is not a parameter or binding of `after`")
            ),
        ]
    );
    // A duplicate within one pattern cites the pattern's first name.
    let within = &result.diagnostics[0];
    assert_eq!(within.label(), "this binding repeats an earlier name");
    assert_eq!(
        within
            .secondary_spans()
            .iter()
            .map(|secondary| (
                fixture.source().slice(secondary.span()).unwrap(),
                secondary.label()
            ))
            .collect::<Vec<_>>(),
        [("a", "the first name is here")]
    );
}

#[test]
fn tuple_types_and_selections_are_checked_once_in_order() {
    let (fixture, result) = rejected(concat!(
        "  type Pair = (Int, Int);\n",
        "  spec nested(p: (Pair, Int)) -> Int { 0 }\n",
        "  spec array(p: Pair^2) -> Int { 0 }\n",
        "  spec count() -> Pair { (1, 2, 3) }\n",
        "  spec scalar() -> Int { (1, 2) }\n",
        "  spec element() -> Pair { (true, false) }\n",
        "  spec whole(p: Pair) -> Int { p }\n",
        "  spec not_tuple(x: Int^2) -> Int { x.0 }\n",
        "  spec position(p: Pair) -> Int { p.2 }\n",
        "  spec far(p: Pair) -> Int { p.4294967296 }\n",
        "  spec selected(p: Pair) -> Bool { p.0 }\n",
        "  spec indexed(p: Pair) -> Int { p[0] }\n",
        "  spec updated(p: Pair) -> Pair { p with [0] = 1 }\n",
        "  spec equal(p: Pair, q: Pair) -> Bool { p == q }\n",
        "  spec added(p: Pair, q: Pair) -> Pair { p + q }\n",
        "  spec negated(p: Pair) -> Pair { -p }\n",
        "  spec converted(p: Pair) -> Int { p as Int }\n",
        "  spec converted_to(x: Int) -> Pair { x as Pair }\n",
        "  spec pattern() -> Int { let (a: Int, b: Bool) = (1, 2); a }\n",
        "  spec not_a_value() -> Int { let (a: Int, b: Int) = 5; a }\n",
        "  spec nested_name() -> Int { let (a: (Int, Int), b: Int) = (1, 2); b }\n",
        "  spec compared(p: Pair) -> Bool { (1, 2) == p }\n",
        "  spec compared_arrays(x: Int^2) -> Bool { [1, 2] != x }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UnsupportedType,
                "Pair",
                String::from("`Pair` is a tuple type, so this is a tuple of tuples")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Pair^2",
                String::from("`Pair` is a tuple type, so this is an array of tuples")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "(1, 2, 3)",
                String::from("this tuple has 3 elements, but `(Int, Int)` has 2")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "(1, 2)",
                String::from("a tuple cannot have type `Int`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "true",
                String::from("`true` has type `Bool`, but `Int` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "false",
                String::from("`false` has type `Bool`, but `Int` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "p",
                String::from("`p` has type `(Int, Int)`, but `Int` is required here")
            ),
            (
                DiagnosticCode::NotATuple,
                "x",
                String::from(
                    "only a tuple has elements selected by position, but this has type \
                     `Int^2`"
                )
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "2",
                String::from("`(Int, Int)` has no element 2")
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "4294967296",
                String::from("`(Int, Int)` has no element 4294967296")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "p.0",
                String::from("this element has type `Int`, but `Bool` is required here")
            ),
            (
                DiagnosticCode::NotAnArray,
                "p",
                String::from("only an array can be indexed, but this has type `(Int, Int)`")
            ),
            (
                DiagnosticCode::NotAnArray,
                "p",
                String::from("only an array can be updated, but this has type `(Int, Int)`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "==",
                String::from("`==` is not defined for `(Int, Int)`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "+",
                String::from("`+` is not defined for `(Int, Int)`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "-",
                String::from("prefix `-` is not defined for `(Int, Int)`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "as",
                String::from("`as` is not defined for `(Int, Int)`")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "Pair",
                String::from("`as` does not convert to the tuple type `(Int, Int)`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "2",
                String::from("an integer literal cannot have type `Bool`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "5",
                String::from("an integer literal cannot have type `(Int, Int)`")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "(Int, Int)",
                String::from("`(Int, Int)` is a tuple type, so this is a tuple of tuples")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "==",
                String::from("`==` is not defined for a tuple")
            ),
            (
                DiagnosticCode::UnsupportedOperator,
                "!=",
                String::from("`!=` is not defined for an array")
            ),
        ]
    );
}

#[test]
fn unresolved_pattern_types_are_reported_once_without_cascades() {
    // As for a binding of one name, a pattern whose element type does
    // not resolve is reported once at each such type; its value is not
    // checked, and none of its names is reported where it is used.
    let (fixture, result) = rejected(concat!(
        "  spec body() -> Int { let (a: Wide, b: Int, c: Narrow) = (missing, 2, 3); a + b + c }\n",
        "  spec step() -> Int {\n",
        "    let r: Int = for i in 0..2 with (s: Int, t: Wide) = (0, missing) { (s + t, t) };\n",
        "    r\n",
        "  }\n",
        "  spec branch(c: Bool) -> Int { if c { let (x: Wide, y: Int) = (1, 2); x } else { 0 } }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UnsupportedType,
                "Wide",
                String::from("unsupported binding type `Wide`")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Narrow",
                String::from("unsupported binding type `Narrow`")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Wide",
                String::from("unsupported accumulator type `Wide`")
            ),
            (
                DiagnosticCode::UnsupportedType,
                "Wide",
                String::from("unsupported binding type `Wide`")
            ),
        ]
    );
}

#[test]
fn tuple_events_and_core_nodes_follow_the_normative_accounting() {
    // `let t: Int = x; t` with `x` an `Int` parameter takes 9 analysis
    // events and 8 Core nodes (17 events in all). The pattern
    // `(t: Int, u: Int)` with the value `(x, x)` adds 5 events: the
    // pattern (1), the second name's uniqueness check and type name
    // (2), the tuple (1), and its second element (1). Its type is still
    // one type node; the tuple and its second element add 2 nodes, and
    // the read of `t` is the binding's read and an element node, 1
    // more: 14 events and 11 nodes, 25 events in all.
    let pattern = module("  spec f(x: Int) -> Int { let (t: Int, u: Int) = (x, x); t }\n");
    // `g() -> Int { 1 }` and `f() -> Int { g() }` take 10 analysis
    // events and 7 nodes. The tuple result type `(Int, Int)` adds its
    // event and its second type name (2), the tuple (1) and the literal
    // `2` (3), and `.1` one event: 17 events. The tuple, the literal and
    // the element node add 3 nodes: 10, and 27 events in all.
    let projection = module(concat!(
        "  spec g() -> (Int, Int) { (1, 2) }\n",
        "  spec f() -> Int { g().1 }\n",
    ));
    for (fixture, events, nodes) in [(&pattern, 25, 11), (&projection, 27, 10)] {
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, []);
        assert!(exact.core.is_some());
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none());
            assert_eq!(first.diagnostics.len(), 1);
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label);
        }
    }
}

#[test]
fn rejects_foreign_spans_in_tuples() {
    let text = "edition 2026; module values { \
                spec value(p: (Int, Int)) -> (Int, Int) { \
                let (a: Int, b: Int) = (p.0, p.1); \
                for i in 0..2 with (s: Int, t: Int) = (a, b) { (t, s + i) } } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    fn tuple_pattern(pattern: &mut Pattern) -> &mut crate::parser::TuplePattern {
        let Pattern::Tuple(tuple) = pattern else {
            unreachable!();
        };
        tuple
    }
    fn binding_of(ast: &mut SyntaxTree) -> &mut Binding {
        &mut typed_body_mut(ast).bindings[0]
    }
    fn tuple_of_value(expression: &mut Expression) -> &mut TupleExpression {
        let ExpressionKind::Tuple(tuple) = &mut expression.kind else {
            unreachable!();
        };
        tuple
    }
    fn project_of(expression: &mut Expression) -> &mut ProjectExpression {
        let ExpressionKind::Project(project) = &mut expression.kind else {
            unreachable!();
        };
        project
    }
    let mut foreign_ast = second.ast.clone();
    let foreign_binding = binding_of(&mut foreign_ast).clone();
    let foreign_body = typed_body_mut(&mut foreign_ast).clone();
    let mut foreign_value = foreign_binding.value.clone();
    let foreign_element = tuple_of_value(&mut foreign_value).elements[1].clone();
    let mut foreign_element_copy = foreign_element.clone();
    let foreign_position = project_of(&mut foreign_element_copy).position_span;
    let mut foreign_pattern = foreign_binding.pattern.clone();
    let foreign_names = tuple_pattern(&mut foreign_pattern).clone();
    let foreign_expression = foreign_body.expression.clone();
    type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
    let mutations: Vec<Mutation<'_>> = vec![
        Box::new(|ast| tuple_pattern(&mut binding_of(ast).pattern).span = foreign_names.span),
        Box::new(|ast| {
            tuple_pattern(&mut binding_of(ast).pattern).elements[1]
                .name
                .span = foreign_names.elements[1].name.span;
        }),
        Box::new(|ast| {
            tuple_pattern(&mut binding_of(ast).pattern).elements[1]
                .ty
                .span = foreign_names.elements[1].ty.span;
        }),
        Box::new(|ast| binding_of(ast).value.span = foreign_binding.value.span),
        Box::new(|ast| {
            tuple_of_value(&mut binding_of(ast).value).elements[1].span = foreign_element.span;
        }),
        Box::new(|ast| {
            project_of(&mut tuple_of_value(&mut binding_of(ast).value).elements[1]).position_span =
                foreign_position;
        }),
        Box::new(|ast| typed_body_mut(ast).expression.span = foreign_expression.span),
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}

#[test]
fn byte_strings_joins_and_slices_build_typed_core_in_postorder() {
    let (fixture, core) = accepted(concat!(
        r#"  spec text() -> Word[8]^10 { "\"\\\n\r\t\0\x7F~ a" }"#,
        "\n",
        "  spec hex() -> Word[8]^3 { hex\" 00Ff  1a \" }\n",
        "  spec join(x: Word[8]^2) -> Word[8]^5 { x ++ [1; 2] ++ \"z\" }\n",
        "  spec slice(x: Word[8]^4) -> Word[8]^2 { x[1..3] }\n",
        "  spec open(x: Word[8]^4) -> Word[8]^3 { x[..3] }\n",
        "  spec rest(x: Word[8]^4) -> Word[8]^3 { x[1..] }\n",
        "  spec splice(x: Word[8]^4) -> Word[8]^4 { x with [2..] = \"hi\" }\n",
        "  spec reverse(x: Word[8]^4) -> Word[8]^4 {\n",
        "    for i in 0..4 with y: Word[8]^4 = x { y with [i..i + 1] = x[3 - i..4 - i] }\n",
        "  }\n",
    ));
    let owned = |rows: &[(&str, &'static str, CoreType)]| {
        rows.iter()
            .map(|(operation, source, ty)| ((*operation).to_owned(), *source, ty.clone()))
            .collect::<Vec<_>>()
    };
    let bytes = |length| array_of(CoreType::Word8, length);

    // A byte string is one literal node holding its bytes: each escape
    // is one byte, and a hex string's spaces are not bytes.
    assert_eq!(core.functions[0].result_type, bytes(10));
    assert_eq!(
        core_nodes(&fixture, &core.functions[0]),
        owned(&[(
            "literal [0x22, 0x5c, 0x0a, 0x0d, 0x09, 0x00, 0x7f, 0x7e, 0x20, 0x61]",
            r#""\"\\\n\r\t\0\x7F~ a""#,
            bytes(10)
        )])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[1]),
        owned(&[("literal [0x00, 0xff, 0x1a]", "hex\" 00Ff  1a \"", bytes(3))])
    );
    // `++` associates to the left, and each join is one node after its
    // operands.
    assert_eq!(
        core_nodes(&fixture, &core.functions[2]),
        owned(&[
            ("parameter 0", "x", bytes(2)),
            ("literal 0x01", "1", CoreType::Word8),
            ("fill", "[1; 2]", bytes(2)),
            ("concat", "x ++ [1; 2]", bytes(4)),
            ("literal [0x7a]", "\"z\"", bytes(1)),
            ("concat", "x ++ [1; 2] ++ \"z\"", bytes(5)),
        ])
    );
    // A slice is its base, its start, its end, and one node; an omitted
    // bound is an `Int` literal, 0 or the base's length, spanning `..`.
    assert_eq!(
        core_nodes(&fixture, &core.functions[3]),
        owned(&[
            ("parameter 0", "x", bytes(4)),
            ("literal 1", "1", CoreType::Int),
            ("literal 3", "3", CoreType::Int),
            ("slice", "x[1..3]", bytes(2)),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[4]),
        owned(&[
            ("parameter 0", "x", bytes(4)),
            ("literal 0", "..", CoreType::Int),
            ("literal 3", "3", CoreType::Int),
            ("slice", "x[..3]", bytes(3)),
        ])
    );
    assert_eq!(
        core_nodes(&fixture, &core.functions[5]),
        owned(&[
            ("parameter 0", "x", bytes(4)),
            ("literal 1", "1", CoreType::Int),
            ("literal 4", "..", CoreType::Int),
            ("slice", "x[1..]", bytes(3)),
        ])
    );
    // A slice update is its base, its bounds, its value, and one node.
    assert_eq!(
        core_nodes(&fixture, &core.functions[6]),
        owned(&[
            ("parameter 0", "x", bytes(4)),
            ("literal 2", "2", CoreType::Int),
            ("literal 4", "..", CoreType::Int),
            ("literal [0x68, 0x69]", "\"hi\"", bytes(2)),
            ("slice update", "x with [2..] = \"hi\"", bytes(4)),
        ])
    );
    // Bounds that fall as the index rises are proved in range too.
    let reverse = &core.functions[7];
    assert_eq!(
        expression_nodes(&fixture, reverse.loops[0].step()),
        owned(&[
            ("accumulator of loop #0", "y", bytes(4)),
            ("index of loop #0", "i", CoreType::Int),
            ("index of loop #0", "i", CoreType::Int),
            ("literal 1", "1", CoreType::Int),
            ("infix +", "i + 1", CoreType::Int),
            ("parameter 0", "x", bytes(4)),
            ("literal 3", "3", CoreType::Int),
            ("index of loop #0", "i", CoreType::Int),
            ("infix -", "3 - i", CoreType::Int),
            ("literal 4", "4", CoreType::Int),
            ("index of loop #0", "i", CoreType::Int),
            ("infix -", "4 - i", CoreType::Int),
            ("slice", "x[3 - i..4 - i]", bytes(1)),
            (
                "slice update",
                "y with [i..i + 1] = x[3 - i..4 - i]",
                bytes(4)
            ),
        ])
    );
}

#[test]
fn byte_strings_hold_one_through_256_printable_bytes() {
    let at_limit = format!(
        "  spec text() -> Word[8]^256 {{ \"{}\" }}\n  spec hex() -> Word[8]^256 {{ hex\"{}\" }}\n",
        "a".repeat(256),
        "01 ".repeat(256),
    );
    let (_, core) = accepted(&at_limit);
    assert_eq!(core.functions.len(), 2);
    let long = format!("\"{}\"", "a".repeat(257));
    let long_hex = format!("hex\"{}\"", "ff".repeat(257));
    // Decoding stops at the byte past the limit, before the character
    // that follows it.
    let past = format!("\"{}é\"", "a".repeat(300));
    let (fixture, result) = rejected(&format!(
        concat!(
            "  spec tab() -> Word[8]^3 {{ \"a\tb\" }}\n",
            "  spec delete() -> Word[8]^1 {{ \"\u{7f}\" }}\n",
            "  spec accent() -> Word[8]^3 {{ \"éa€\" }}\n",
            "  spec euro() -> Word[8]^3 {{ \"a€\" }}\n",
            "  spec empty() -> Word[8]^1 {{ \"\" }}\n",
            "  spec long() -> Word[8]^256 {{ {long} }}\n",
            "  spec long_hex() -> Word[8]^256 {{ {long_hex} }}\n",
            "  spec past() -> Word[8]^256 {{ {past} }}\n",
        ),
        long = long,
        long_hex = long_hex,
        past = past,
    ));
    let too_long = String::from("a byte string holds at most 256 bytes");
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::UnprintableByteString,
                "\t",
                String::from("U+0009 is not a printable ASCII character")
            ),
            (
                DiagnosticCode::UnprintableByteString,
                "\u{7f}",
                String::from("U+007F is not a printable ASCII character")
            ),
            (
                DiagnosticCode::UnprintableByteString,
                "é",
                String::from("U+00E9 is not a printable ASCII character")
            ),
            (
                DiagnosticCode::UnprintableByteString,
                "€",
                String::from("U+20AC is not a printable ASCII character")
            ),
            (
                DiagnosticCode::UnsupportedArrayLength,
                "\"\"",
                String::from("a byte string holds at least one byte")
            ),
            (
                DiagnosticCode::UnsupportedArrayLength,
                long.as_str(),
                too_long.clone()
            ),
            (
                DiagnosticCode::UnsupportedArrayLength,
                long_hex.as_str(),
                too_long.clone()
            ),
            (
                DiagnosticCode::UnsupportedArrayLength,
                past.as_str(),
                too_long
            ),
        ]
    );
    // An ASCII character's label gives its byte, and any other's its UTF-8
    // bytes, as a hex string.
    assert_eq!(
        result
            .diagnostics
            .iter()
            .take(4)
            .map(Diagnostic::label)
            .collect::<Vec<_>>(),
        [
            "its byte is written `hex\"09\"`",
            "its byte is written `hex\"7f\"`",
            "its UTF-8 bytes are written `hex\"c3 a9\"`",
            "its UTF-8 bytes are written `hex\"e2 82 ac\"`",
        ]
    );
    assert_eq!(result.diagnostics[4].label(), "this string is empty");
    assert_eq!(
        result.diagnostics[5].label(),
        "this string holds more than 256"
    );
}

#[test]
fn slice_bounds_are_static_in_range_and_a_fixed_length_apart() {
    let (fixture, result) = rejected(concat!(
        "  spec beyond(x: Word[8]^4) -> Word[8]^2 { x[3..5] }\n",
        "  spec before(x: Word[8]^4) -> Word[8]^2 { x[-1..1] }\n",
        "  spec rising(x: Word[8]^4) -> Word[8]^4 {\n",
        "    for i in 0..4 with y: Word[8]^4 = x { y with [i + 1..i + 2] = [0] }\n",
        "  }\n",
        "  spec falling(x: Word[8]^4) -> Word[8]^1 {\n",
        "    for i in 0..4 with y: Word[8]^1 = [0] { x[4 - i..5 - i] }\n",
        "  }\n",
        "  spec negated(x: Word[8]^4) -> Word[8]^1 {\n",
        "    for i in 0..2 with y: Word[8]^1 = [0] { x[-i..-i + 1] }\n",
        "  }\n",
        "  spec empty(x: Word[8]^4) -> Word[8]^1 { x[2..2] }\n",
        "  spec one(x: Word[8]^4) -> Word[8]^1 { x[2..1] }\n",
        "  spec far(x: Word[8]^4) -> Word[8]^1 { x[0..9223372036854775808] }\n",
        "  spec growing(x: Word[8]^4) -> Word[8]^1 {\n",
        "    for i in 0..2 with y: Word[8]^1 = [0] { x[i..2 * i + 1] }\n",
        "  }\n",
        "  spec bound(x: Word[8]^4) -> Word[8]^1 { let n: Int = 1; x[n..2] }\n",
        "  spec quotient(x: Word[8]^4) -> Word[8]^1 { x[4 / 2..3] }\n",
        "  spec product(x: Word[8]^4) -> Word[8]^1 {\n",
        "    for i in 0..2 with y: Word[8]^1 = [0] { x[i * i..i * i + 1] }\n",
        "  }\n",
        "  spec typed(x: Word[8]^4, w: Word[8]) -> Word[8]^1 { x[w..2] }\n",
        "  spec length(x: Word[8]^4) -> Word[8]^3 { x[..2] }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::IndexOutOfRange,
                "3..5",
                String::from(
                    "this slice reaches elements 3 through 4, out of range for `Word[8]^4`"
                )
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "-1..1",
                String::from(
                    "this slice reaches elements -1 through 0, out of range for `Word[8]^4`"
                )
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "i + 1..i + 2",
                String::from(
                    "this slice reaches elements 1 through 4, out of range for `Word[8]^4`"
                )
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "4 - i..5 - i",
                String::from(
                    "this slice reaches elements 1 through 4, out of range for `Word[8]^4`"
                )
            ),
            (
                DiagnosticCode::IndexOutOfRange,
                "-i..-i + 1",
                String::from(
                    "this slice reaches elements -1 through 0, out of range for `Word[8]^4`"
                )
            ),
            (
                DiagnosticCode::SliceLength,
                "2..2",
                String::from("this slice is empty: its bounds are equal")
            ),
            (
                DiagnosticCode::SliceLength,
                "2..1",
                String::from("this slice ends 1 element before it starts")
            ),
            (
                DiagnosticCode::SliceLength,
                "0..9223372036854775808",
                String::from("this slice's bounds are too far apart")
            ),
            (
                DiagnosticCode::SliceLength,
                "i..2 * i + 1",
                String::from("the length of this slice changes from step to step")
            ),
            (
                DiagnosticCode::NonStaticIndex,
                "n",
                String::from("a slice's bounds may use only integer literals and loop indices")
            ),
            (
                DiagnosticCode::NonStaticIndex,
                "4 / 2",
                String::from("a slice's bounds may use only integer literals and loop indices")
            ),
            (
                DiagnosticCode::NonStaticIndex,
                "*",
                String::from("a slice's bound may multiply a loop index only by a constant")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "w",
                String::from("`w` has type `Word[8]`, but `Int` is required here")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "x[..2]",
                String::from("this slice has 2 elements, but `Word[8]^3` has 3")
            ),
        ]
    );
}

#[test]
fn slice_bounds_are_held_to_the_significant_bit_limit_of_int() {
    let fixture = module(concat!(
        "  spec f(x: Word[8]^4) -> Word[8]^1 {\n",
        "    for i in 0..2 with y: Word[8]^1 = [0] { x[(i * 200) * 2..(i * 200) * 2 + 1] }\n",
        "  }\n",
    ));
    let limits = Limits {
        integer_bits: 8,
        ..Limits::DEFAULT
    };
    let result = fixture.analyze_with(limits);
    assert_eq!(result, fixture.analyze_with(limits));
    assert!(result.core.is_none());
    assert_eq!(
        reported(&fixture, &result),
        [(
            DiagnosticCode::IndexOutOfRange,
            "(i * 200) * 2",
            String::from("a part of this bound exceeds the 8-significant-bit limit of `Int`")
        )]
    );
}

#[test]
fn joins_are_checked_once_in_order() {
    let (fixture, result) = rejected(concat!(
        "  spec scalar() -> Word[32] { \"ab\" ++ \"cd\" }\n",
        "  spec left(w: Word[32]) -> Word[8]^4 { w ++ missing }\n",
        "  spec right(w: Word[32]) -> Word[8]^4 { \"ab\" ++ w }\n",
        "  spec tuple(p: (Int, Int)) -> Int^4 { p ++ [1, 2] }\n",
        "  spec unknown() -> Word[8]^4 { missing ++ \"ab\" }\n",
        "  spec unprintable() -> Word[8]^4 { \"é\" ++ \"ab\" }\n",
        "  spec unprintable_right() -> Word[8]^4 { \"ab\" ++ \"é\" }\n",
        "  spec sum(x: Word[8]^2, y: Word[8]^3) -> Word[8]^4 { x ++ y }\n",
        "  spec fills(x: Word[8]^4) -> Word[8]^4 { x ++ 1 }\n",
        "  spec literal(x: Word[8]^2) -> Word[8]^4 { 1 ++ x }\n",
        "  spec elements(x: Word[8]^2, y: Word[32]^2) -> Word[8]^4 { x ++ y }\n",
        "  spec chain(x: Word[8]^2) -> Word[8]^8 { x ++ x ++ x }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::TypeMismatch,
                "++",
                String::from("`++` joins arrays, but `Word[32]` is required here")
            ),
            (
                DiagnosticCode::NotAnArray,
                "w",
                String::from("only arrays can be joined, but this has type `Word[32]`")
            ),
            (
                DiagnosticCode::NotAnArray,
                "w",
                String::from("only arrays can be joined, but this has type `Word[32]`")
            ),
            (
                DiagnosticCode::NotAnArray,
                "p",
                String::from("only arrays can be joined, but this has type `(Int, Int)`")
            ),
            (
                DiagnosticCode::UnknownParameter,
                "missing",
                String::from("`missing` is not a parameter of `unknown`")
            ),
            (
                DiagnosticCode::UnprintableByteString,
                "é",
                String::from("U+00E9 is not a printable ASCII character")
            ),
            (
                DiagnosticCode::UnprintableByteString,
                "é",
                String::from("U+00E9 is not a printable ASCII character")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "++",
                String::from("`++` joins 2 and 3 elements, 5 in all, but `Word[8]^4` has 4")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "++",
                String::from(
                    "the left operand of `++` has 4 elements, leaving none of the 4 of \
                     `Word[8]^4` for the right"
                )
            ),
            (
                DiagnosticCode::TypeMismatch,
                "1",
                String::from("an integer literal cannot have type `Word[8]^4`")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "y",
                String::from("`y` has type `Word[32]^2`, but `Word[8]^2` is required here")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "++",
                String::from("`++` joins 4 and 2 elements, 6 in all, but `Word[8]^8` has 8")
            ),
        ]
    );
}

#[test]
fn a_conditional_joined_takes_its_length_from_a_branch_without_bindings() {
    // A conditional's length is its first typed leaf's, or else that of
    // the first branch that binds no names and has a length of its own.
    let (_, core) = accepted(concat!(
        "  spec middle(c: Bool) -> Word[8]^6 { \"a\" ++ (if c { [1, 2, 3] } else { \"xyz\" }) ++ \"de\" }\n",
        "  spec bound(c: Bool) -> Word[8]^5 {\n",
        "    (if c { let t: Word[8]^3 = \"abc\"; t } else { [1, 2, 3] }) ++ \"de\"\n",
        "  }\n",
    ));
    assert_eq!(core.functions.len(), 2);
    let (fixture, result) = rejected(concat!(
        "  spec short(c: Bool) -> Word[8]^6 { \"a\" ++ (if c { [1, 2, 3] } else { [4, 5] }) ++ \"de\" }\n",
        "  spec long(c: Bool) -> Word[8]^6 { \"ab\" ++ (if c { [1, 2, 3, 4] } else { \"wxyz\" }) ++ \"de\" }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::ArrayLengthMismatch,
                "[4, 5]",
                String::from("this array has 2 elements, but `Word[8]^3` has 3")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "++",
                String::from("`++` joins 6 and 2 elements, 8 in all, but `Word[8]^6` has 6")
            ),
        ]
    );
}

#[test]
fn slices_and_slice_updates_are_typed_against_the_required_array() {
    // A slice gives an array of its base's element type and its own
    // length, and a slice update gives its base's array type; each is
    // reported where it stands, and a base that is not an array where it
    // is written.
    let (fixture, result) = rejected(concat!(
        "  spec scalar(x: Word[8]^4) -> Word[8] { x[1..3] }\n",
        "  spec element(x: Word[32]^4) -> Word[8]^2 { x[1..3] }\n",
        "  spec word(w: Word[32]) -> Word[32]^1 { w[0..1] }\n",
        "  spec tuple(p: (Int, Int)) -> Int^1 { p[0..1] }\n",
        "  spec length(x: Word[8]^4) -> Word[8]^3 { x[1..3] }\n",
        "  spec wanted(x: Word[8]^4) -> Word[8] { x with [0..2] = \"ab\" }\n",
        "  spec base(w: Word[32]) -> Word[32]^4 { w with [0..1] = [0] }\n",
        "  spec pair(p: (Int, Int)) -> (Int, Int) { p with [0..1] = [0] }\n",
        "  spec value(x: Word[8]^4) -> Word[8]^4 { x with [1..3] = [1, 2, 3] }\n",
        "  spec values(x: Word[8]^4) -> Word[8]^4 { x with [1..] = \"ab\" }\n",
    ));
    assert_eq!(
        reported(&fixture, &result),
        [
            (
                DiagnosticCode::TypeMismatch,
                "x[1..3]",
                String::from("a slice is an array, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x[1..3]",
                String::from(
                    "this slice is an array of `Word[32]`, but `Word[8]^2` is required here"
                )
            ),
            (
                DiagnosticCode::NotAnArray,
                "w",
                String::from("only an array can be sliced, but this has type `Word[32]`")
            ),
            (
                DiagnosticCode::NotAnArray,
                "p",
                String::from("only an array can be sliced, but this has type `(Int, Int)`")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "x[1..3]",
                String::from("this slice has 2 elements, but `Word[8]^3` has 3")
            ),
            (
                DiagnosticCode::TypeMismatch,
                "x with [0..2] = \"ab\"",
                String::from("an update gives an array, but `Word[8]` is required here")
            ),
            (
                DiagnosticCode::NotAnArray,
                "w",
                String::from("only an array can be updated, but this has type `Word[32]`")
            ),
            (
                DiagnosticCode::NotAnArray,
                "p",
                String::from("only an array can be updated, but this has type `(Int, Int)`")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "[1, 2, 3]",
                String::from("this array has 3 elements, but `Word[8]^2` has 2")
            ),
            (
                DiagnosticCode::ArrayLengthMismatch,
                "\"ab\"",
                String::from("this byte string holds 2 bytes, but `Word[8]^3` has 3")
            ),
        ]
    );
    let notes = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.notes().first().cloned().unwrap_or_default())
        .collect::<Vec<_>>();
    assert_eq!(
        notes[0],
        "one element is selected by an index, such as `x[0]`"
    );
    assert_eq!(
        notes[1],
        "a slice is an array of the elements of the array it is taken from"
    );
    assert_eq!(
        notes[7],
        "a tuple with elements replaced is written anew, such as `(v, p.1)`"
    );
}

#[test]
fn byte_string_and_slice_events_and_core_nodes_follow_the_normative_accounting() {
    // `f(x: Word[8]^4) -> Word[8]^4 { x }` takes 10 analysis events and 5
    // Core nodes (15 events in all). A byte string of n bytes in place of
    // `x` takes 1 + n events and one node; `x ++ x` adds its operator's
    // event and the second read, 2 events and 2 nodes; a slice adds its
    // own event, the events of its written bounds (an integer literal
    // takes 3), and a node for each bound, written or not, and for
    // itself; a slice update likewise, with its value.
    let cases = [
        ("Word[8]^3", "\"abc\"", 18, 5),
        ("Word[8]^3", "hex\"00 01 02\"", 18, 5),
        ("Word[8]^8", "x ++ x", 19, 7),
        ("Word[8]^2", "x[1..3]", 25, 8),
        ("Word[8]^2", "x[..2]", 22, 8),
        ("Word[8]^2", "x[2..]", 22, 8),
        ("Word[8]^4", "x with [1..3] = x[..2]", 34, 12),
    ];
    for (result, body, events, nodes) in cases {
        let fixture = module(&format!(
            "  spec f(x: Word[8]^4) -> {result} {{ {body} }}\n"
        ));
        let exact = fixture.analyze_with(Limits {
            events,
            nodes,
            ..Limits::DEFAULT
        });
        assert_eq!(exact.diagnostics, [], "{body}");
        assert!(exact.core.is_some(), "{body}");
        for (limits, label) in [
            (
                Limits {
                    events: events - 1,
                    nodes,
                    ..Limits::DEFAULT
                },
                "semantic event budget exhausted",
            ),
            (
                Limits {
                    events,
                    nodes: nodes - 1,
                    ..Limits::DEFAULT
                },
                "typed Core node budget exhausted",
            ),
        ] {
            let first = fixture.analyze_with(limits);
            assert_eq!(first, fixture.analyze_with(limits));
            assert!(first.core.is_none(), "{body}");
            assert_eq!(first.diagnostics.len(), 1, "{body}");
            assert_eq!(
                first.diagnostics[0].code(),
                DiagnosticCode::SemanticResourceLimit
            );
            assert_eq!(first.diagnostics[0].label(), label, "{body}");
        }
    }
}

#[test]
fn slice_bound_storage_failures_return_no_partial_core() {
    for (members, responsible, label) in [
        (
            "  spec f(x: Word[8]^4) -> Word[8]^2 { x[..2] }\n",
            "..2",
            "slice bound storage allocation failed",
        ),
        (
            "  spec f(x: Word[8]^4) -> Word[8]^4 { x with [1..3] = \"ab\" }\n",
            "1..3",
            "slice bound storage allocation failed",
        ),
    ] {
        let fixture = module(members);
        let analyze_with_failure = || {
            let mut analyzer = Analyzer::new(fixture.source(), &fixture.ast, Limits::DEFAULT);
            analyzer.reserve_range_limbs = |_, _| false;
            analyzer.run()
        };
        let first = analyze_with_failure();
        assert_eq!(first, analyze_with_failure());
        assert!(first.core().is_none());
        assert_eq!(first.diagnostics().len(), 1, "{members}");
        let diagnostic = &first.diagnostics()[0];
        assert_eq!(diagnostic.code(), DiagnosticCode::SemanticResourceLimit);
        assert_eq!(
            fixture.source().slice(diagnostic.primary_span()),
            Some(responsible)
        );
        assert_eq!(diagnostic.label(), label);
    }
}

#[test]
fn rejects_foreign_spans_in_byte_strings_joins_and_slices() {
    let text = "edition 2026; module values { \
                spec value(x: Word[8]^4) -> Word[8]^6 { \
                (x with [0..2] = x[2..]) ++ \"ab\" } }\n";
    let first = Fixture::new(text);
    let second = Fixture::new(text);
    fn join_of(ast: &mut SyntaxTree) -> &mut BinaryExpression {
        let ExpressionKind::Binary(binary) = &mut typed_body_mut(ast).expression.kind else {
            unreachable!();
        };
        binary
    }
    fn update_of(ast: &mut SyntaxTree) -> &mut SliceUpdateExpression {
        let ExpressionKind::Parenthesized(inner) = &mut join_of(ast).left.kind else {
            unreachable!();
        };
        let ExpressionKind::SliceUpdate(update) = &mut inner.kind else {
            unreachable!();
        };
        update
    }
    fn slice_of(ast: &mut SyntaxTree) -> &mut SliceExpression {
        let ExpressionKind::Slice(slice) = &mut update_of(ast).value.kind else {
            unreachable!();
        };
        slice
    }
    let mut foreign_ast = second.ast.clone();
    let foreign_join = join_of(&mut foreign_ast).clone();
    let foreign_update = update_of(&mut foreign_ast).clone();
    let foreign_slice = slice_of(&mut foreign_ast).clone();
    let foreign_start = foreign_update.range.start.clone().unwrap();
    type Mutation<'a> = Box<dyn Fn(&mut SyntaxTree) + 'a>;
    let mutations: Vec<Mutation<'_>> = vec![
        Box::new(|ast| join_of(ast).operator_span = foreign_join.operator_span),
        Box::new(|ast| join_of(ast).right.span = foreign_join.right.span),
        Box::new(|ast| update_of(ast).keyword_span = foreign_update.keyword_span),
        Box::new(|ast| update_of(ast).range.span = foreign_update.range.span),
        Box::new(|ast| update_of(ast).range.dots_span = foreign_update.range.dots_span),
        Box::new(|ast| {
            update_of(ast).range.start.as_mut().unwrap().span = foreign_start.span;
        }),
        Box::new(|ast| update_of(ast).value.span = foreign_update.value.span),
        Box::new(|ast| slice_of(ast).range.span = foreign_slice.range.span),
        Box::new(|ast| slice_of(ast).range.dots_span = foreign_slice.range.dots_span),
        Box::new(|ast| slice_of(ast).base.span = foreign_slice.base.span),
    ];
    assert!(analyze(first.source(), &first.ast).core.is_some());
    for (case_index, mutate) in mutations.iter().enumerate() {
        let mut ast = first.ast.clone();
        mutate(&mut ast);
        let result = analyze(first.source(), &ast);
        assert_eq!(result, analyze(first.source(), &ast), "case {case_index}");
        assert!(result.core.is_none(), "case {case_index}");
        assert_eq!(result.diagnostics.len(), 1, "case {case_index}");
        assert_eq!(
            result.diagnostics[0].code(),
            DiagnosticCode::InvalidSemanticInput,
            "case {case_index}"
        );
    }
}
