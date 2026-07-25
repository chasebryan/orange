//! Indexed conformance evidence for the accepted Orange 2026 base S2 slice.

use std::collections::{BTreeMap, BTreeSet};

use orange_compiler::{
    DiagnosticCode, Edition, FunctionKind, Lexed, ParseResult, SourceMap, TokenKind, lex, parse,
};

const LANGUAGE_SPECIFICATION: &str = include_str!("../../../../docs/LANGUAGE_2026.md");
const S2_CONFORMANCE_SOURCE: &str = include_str!("s2_conformance.rs");
const WORKSPACE_MANIFEST: &str = include_str!("../../../Cargo.toml");
const COMPILER_MANIFEST: &str = include_str!("../../orange-compiler/Cargo.toml");
const ORANGEC_MANIFEST: &str = include_str!("../Cargo.toml");
const COMPILER_LIB_SOURCE: &str = include_str!("../../orange-compiler/src/lib.rs");
const SOURCE_SOURCE: &str = include_str!("../../orange-compiler/src/source.rs");
const LEXER_SOURCE: &str = include_str!("../../orange-compiler/src/lexer.rs");
const PARSER_SOURCE: &str = include_str!("../../orange-compiler/src/parser.rs");
const CLI_TEST_SOURCE: &str = include_str!("cli.rs");

const EXPECTED_WORKSPACE_MANIFEST: &str = r#"[workspace]
members = [
  "crates/orange-compiler",
  "crates/orangec",
]
resolver = "2"

[workspace.package]
version = "0.0.1"
edition = "2024"
rust-version = "1.96.1"
publish = false

[workspace.lints.rust]
missing_docs = "deny"
unsafe_code = "forbid"

[workspace.lints.clippy]
all = "deny"

[profile.release]
debug-assertions = true
overflow-checks = true
"#;
const EXPECTED_COMPILER_MANIFEST: &str = r#"[package]
name = "orange-compiler"
description = "Permanent compiler foundations for the Orange language"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[lints]
workspace = true
"#;
const EXPECTED_ORANGEC_MANIFEST: &str = r#"[package]
name = "orangec"
description = "Command-line frontend for the Orange compiler"
version.workspace = true
edition.workspace = true
rust-version.workspace = true
publish.workspace = true

[dependencies]
orange-compiler = { path = "../orange-compiler" }

[lints]
workspace = true
"#;

const CONFORMANCE_EVIDENCE: u16 = 1 << 0;
const CLI_EVIDENCE: u16 = 1 << 1;
const SOURCE_UNIT_EVIDENCE: u16 = 1 << 2;
const LEXER_UNIT_EVIDENCE: u16 = 1 << 3;
const PARSER_UNIT_EVIDENCE: u16 = 1 << 4;
const INJECTED_RESOURCE_EVIDENCE: u16 = 1 << 5;

#[derive(Clone, Copy)]
struct RuleRequirement {
    id: &'static str,
    layers: u16,
}

const RULES: [RuleRequirement; 13] = [
    RuleRequirement {
        id: "S2-SOURCE-01",
        layers: SOURCE_UNIT_EVIDENCE | CLI_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-TRIVIA-01",
        layers: CONFORMANCE_EVIDENCE | LEXER_UNIT_EVIDENCE | PARSER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-NAME-01",
        layers: LEXER_UNIT_EVIDENCE | PARSER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-INTEGER-01",
        layers: CONFORMANCE_EVIDENCE | LEXER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-STRING-01",
        layers: CONFORMANCE_EVIDENCE | LEXER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-PUNCT-01",
        layers: CONFORMANCE_EVIDENCE | LEXER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-LEX-RESOURCE-01",
        layers: LEXER_UNIT_EVIDENCE | INJECTED_RESOURCE_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-GRAMMAR-01",
        layers: CONFORMANCE_EVIDENCE | PARSER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-AST-01",
        layers: PARSER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-PARSE-DIAG-01",
        layers: CONFORMANCE_EVIDENCE | PARSER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-PARSE-RESOURCE-01",
        layers: PARSER_UNIT_EVIDENCE | INJECTED_RESOURCE_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-PHASE-01",
        layers: CONFORMANCE_EVIDENCE | CLI_EVIDENCE | PARSER_UNIT_EVIDENCE,
    },
    RuleRequirement {
        id: "S2-DETERMINISM-01",
        layers: CONFORMANCE_EVIDENCE | LEXER_UNIT_EVIDENCE | PARSER_UNIT_EVIDENCE,
    },
];

#[derive(Clone, Copy)]
struct TestEvidence {
    source_path: &'static str,
    test: &'static str,
    rules: &'static [&'static str],
}

const TEST_EVIDENCE: &[TestEvidence] = &[
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s2_conformance.rs",
        test: "s2_ascii_trivia_uppercase_radices_and_base_grammar_are_repeatable",
        rules: &[
            "S2-TRIVIA-01",
            "S2-INTEGER-01",
            "S2-GRAMMAR-01",
            "S2-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s2_conformance.rs",
        test: "s2_supported_string_escape_set_is_exact_and_repeatable",
        rules: &["S2-STRING-01", "S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s2_conformance.rs",
        test: "s2_punctuation_longest_matches_are_exact_and_repeatable",
        rules: &["S2-PUNCT-01", "S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s2_conformance.rs",
        test: "s2_base_grammar_diagnostics_are_exact_and_repeatable",
        rules: &[
            "S2-GRAMMAR-01",
            "S2-NAME-01",
            "S2-PARSE-DIAG-01",
            "S2-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s2_conformance.rs",
        test: "s2_lexical_failure_skips_parsing_repeatably",
        rules: &["S2-PHASE-01", "S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/s2_conformance.rs",
        test: "s2_same_kind_duplicate_names_remain_distinct_syntax_nodes",
        rules: &[
            "S2-NAME-01",
            "S2-GRAMMAR-01",
            "S2-AST-01",
            "S2-DETERMINISM-01",
        ],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/source.rs",
        test: "maps_utf8_offsets_across_crlf_lines",
        rules: &["S2-SOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/source.rs",
        test: "treats_lf_crlf_and_bare_cr_as_logical_line_endings",
        rules: &["S2-SOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/source.rs",
        test: "checkpoints_preserve_unicode_columns_across_crlf_lines",
        rules: &["S2-SOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/source.rs",
        test: "enforces_the_public_source_byte_limit",
        rules: &["S2-SOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/source.rs",
        test: "rejects_spans_that_split_utf8_or_cross_sources",
        rules: &["S2-SOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "ignores_line_comments_and_nested_block_comments",
        rules: &["S2-TRIVIA-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "uses_only_ascii_whitespace_and_stops_comments_at_all_line_endings",
        rules: &["S2-TRIVIA-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "reserved_and_punctuation_spellings_are_exact",
        rules: &["S2-NAME-01", "S2-PUNCT-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "rejects_non_ascii_identifiers_one_scalar_at_a_time",
        rules: &["S2-NAME-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "validates_integer_bases_and_separator_placement",
        rules: &["S2-INTEGER-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "reports_invalid_escapes_and_recovers_at_the_closing_quote",
        rules: &["S2-STRING-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "unterminated_string_stops_before_every_logical_line_ending",
        rules: &["S2-STRING-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "bounds_the_token_stream_and_reports_the_limit_once",
        rules: &["S2-LEX-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "exact_token_boundary_preserves_the_mandatory_eof_slot",
        rules: &["S2-LEX-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "ordinary_token_reservation_failure_discards_partial_tokens_and_preserves_eof",
        rules: &["S2-LEX-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "diagnostic_vector_reservation_failure_is_an_allocation_free_lexical_failure",
        rules: &["S2-LEX-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "token_limit_survives_the_ordinary_diagnostic_budget",
        rules: &["S2-LEX-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "bounds_lexical_diagnostics_and_emits_one_suppression_record",
        rules: &["S2-LEX-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/lexer.rs",
        test: "malformed_corpus_is_deterministic_and_preserves_valid_spans",
        rules: &["S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "function_kind_inventory_and_spellings_are_exact",
        rules: &["S2-GRAMMAR-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "accepts_duplicate_function_names_as_syntax_in_source_order",
        rules: &["S2-NAME-01", "S2-AST-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "accepts_an_empty_module",
        rules: &["S2-GRAMMAR-01", "S2-AST-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "accepts_lf_crlf_and_bare_cr_as_logical_line_endings",
        rules: &["S2-TRIVIA-01", "S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "rejects_nonexact_edition_and_trailing_syntax_with_stable_codes",
        rules: &["S2-PARSE-DIAG-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "unicode_is_not_whitespace_and_does_not_destabilize_parsing",
        rules: &["S2-TRIVIA-01", "S2-PHASE-01", "S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "caps_syntax_diagnostics_with_one_suppression_record",
        rules: &["S2-PARSE-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "bounds_recovery_delimiter_depth",
        rules: &["S2-PARSE-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "diagnostic_vector_reservation_failure_returns_no_ast_or_diagnostics",
        rules: &["S2-PARSE-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "module_function_reservation_failure_returns_no_partial_ast",
        rules: &["S2-PARSE-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "enforces_internal_event_and_node_limits_without_large_inputs",
        rules: &["S2-PARSE-RESOURCE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orange-compiler/src/parser.rs",
        test: "parser_is_repeatable_and_malformed_corpus_never_panics",
        rules: &["S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "accepts_the_minimal_program_from_standard_input_repeatably",
        rules: &["S2-GRAMMAR-01", "S2-DETERMINISM-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "rejects_non_utf8_source_before_lexing",
        rules: &["S2-SOURCE-01", "S2-PHASE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "rejects_an_oversized_file_without_reading_it",
        rules: &["S2-SOURCE-01", "S2-PHASE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "check_reports_a_stable_source_diagnostic_and_failure_status",
        rules: &["S2-SOURCE-01", "S2-PHASE-01"],
    },
    TestEvidence {
        source_path: "compiler/crates/orangec/tests/cli.rs",
        test: "check_reports_an_exact_repeatable_parser_diagnostic",
        rules: &["S2-PARSE-DIAG-01", "S2-DETERMINISM-01"],
    },
];

fn required_evidence_label(rule: &str) -> &'static str {
    match rule {
        "S2-SOURCE-01" => "Source unit and CLI",
        "S2-TRIVIA-01" | "S2-DETERMINISM-01" => "Conformance, lexer, and parser unit",
        "S2-NAME-01" => "Lexer and parser unit",
        "S2-INTEGER-01" | "S2-STRING-01" | "S2-PUNCT-01" => "Conformance and lexer unit",
        "S2-LEX-RESOURCE-01" => "Lexer and injected-resource unit",
        "S2-GRAMMAR-01" | "S2-PARSE-DIAG-01" => "Conformance and parser unit",
        "S2-AST-01" => "Parser unit",
        "S2-PARSE-RESOURCE-01" => "Parser and injected-resource unit",
        "S2-PHASE-01" => "Conformance, CLI, and parser unit",
        _ => panic!("unknown S2 rule {rule}"),
    }
}

fn required_layers_for_label(label: &str) -> u16 {
    match label {
        "Source unit and CLI" => SOURCE_UNIT_EVIDENCE | CLI_EVIDENCE,
        "Conformance, lexer, and parser unit" => {
            CONFORMANCE_EVIDENCE | LEXER_UNIT_EVIDENCE | PARSER_UNIT_EVIDENCE
        }
        "Lexer and parser unit" => LEXER_UNIT_EVIDENCE | PARSER_UNIT_EVIDENCE,
        "Conformance and lexer unit" => CONFORMANCE_EVIDENCE | LEXER_UNIT_EVIDENCE,
        "Lexer and injected-resource unit" => LEXER_UNIT_EVIDENCE | INJECTED_RESOURCE_EVIDENCE,
        "Conformance and parser unit" => CONFORMANCE_EVIDENCE | PARSER_UNIT_EVIDENCE,
        "Parser unit" => PARSER_UNIT_EVIDENCE,
        "Parser and injected-resource unit" => PARSER_UNIT_EVIDENCE | INJECTED_RESOURCE_EVIDENCE,
        "Conformance, CLI, and parser unit" => {
            CONFORMANCE_EVIDENCE | CLI_EVIDENCE | PARSER_UNIT_EVIDENCE
        }
        _ => panic!("unknown S2 evidence label {label}"),
    }
}

fn documented_rules() -> Vec<(&'static str, &'static str)> {
    LANGUAGE_SPECIFICATION
        .lines()
        .filter_map(|line| {
            let fields = line.split('|').map(str::trim).collect::<Vec<_>>();
            if fields.len() != 5 {
                return None;
            }
            let id = fields[1].strip_prefix('`')?.strip_suffix('`')?;
            id.starts_with("S2-").then_some((id, fields[3]))
        })
        .collect()
}

fn evidence_source(source_path: &str) -> Option<&'static str> {
    match source_path {
        "compiler/crates/orange-compiler/src/source.rs" => Some(SOURCE_SOURCE),
        "compiler/crates/orange-compiler/src/lexer.rs" => Some(LEXER_SOURCE),
        "compiler/crates/orange-compiler/src/parser.rs" => Some(PARSER_SOURCE),
        "compiler/crates/orangec/tests/cli.rs" => Some(CLI_TEST_SOURCE),
        "compiler/crates/orangec/tests/s2_conformance.rs" => Some(S2_CONFORMANCE_SOURCE),
        _ => None,
    }
}

fn evidence_test_indentation(source_path: &str) -> &'static str {
    match source_path {
        "compiler/crates/orangec/tests/cli.rs"
        | "compiler/crates/orangec/tests/s2_conformance.rs" => "",
        _ => "    ",
    }
}

fn rust_whitespace_len(source: &str, offset: usize) -> Option<usize> {
    let character = source.get(offset..)?.chars().next()?;
    matches!(
        character,
        '\u{0009}'
            | '\u{000a}'
            | '\u{000b}'
            | '\u{000c}'
            | '\u{000d}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{2028}'
            | '\u{2029}'
    )
    .then_some(character.len_utf8())
}

fn raw_string_start(bytes: &[u8], offset: usize) -> Option<(usize, usize)> {
    let r_offset = match (bytes.get(offset), bytes.get(offset + 1)) {
        (Some(b'r'), _) => offset,
        (Some(b'b' | b'c'), Some(b'r')) => offset + 1,
        _ => return None,
    };
    let mut cursor = r_offset + 1;
    while bytes.get(cursor) == Some(&b'#') {
        cursor += 1;
    }
    (bytes.get(cursor) == Some(&b'"')).then_some((cursor - r_offset - 1, cursor + 1))
}

fn char_literal_start(source: &str, offset: usize) -> bool {
    let tail = &source[offset + 1..];
    if tail.starts_with('\\') {
        return true;
    }
    let Some(character) = tail.chars().next() else {
        return false;
    };
    tail[character.len_utf8()..].starts_with('\'')
}

fn rust_code_brace_stack_at(source: &str, offset: usize) -> Option<Vec<usize>> {
    let bytes = source.as_bytes();
    if offset > bytes.len() || !source.is_char_boundary(offset) {
        return None;
    }

    let mut cursor = 0;
    let mut delimiter_stack = Vec::new();
    let mut block_comment_depth = 0_usize;
    let mut line_comment = false;
    let mut string = false;
    let mut string_escape = false;
    let mut character = false;
    let mut character_escape = false;
    let mut raw_string_hashes = None;

    while cursor < offset {
        if line_comment {
            if bytes[cursor] == b'\n' || bytes[cursor] == b'\r' {
                line_comment = false;
            }
            cursor += 1;
            continue;
        }
        if block_comment_depth != 0 {
            if bytes.get(cursor..cursor + 2) == Some(b"/*") {
                block_comment_depth += 1;
                cursor += 2;
            } else if bytes.get(cursor..cursor + 2) == Some(b"*/") {
                block_comment_depth -= 1;
                cursor += 2;
            } else {
                cursor += 1;
            }
            continue;
        }
        if let Some(hashes) = raw_string_hashes {
            let terminator_end = cursor + 1 + hashes;
            if bytes[cursor] == b'"'
                && terminator_end <= bytes.len()
                && bytes[cursor + 1..terminator_end]
                    .iter()
                    .all(|byte| *byte == b'#')
            {
                raw_string_hashes = None;
                cursor += hashes + 1;
            } else {
                cursor += 1;
            }
            continue;
        }
        if string {
            if string_escape {
                string_escape = false;
            } else if bytes[cursor] == b'\\' {
                string_escape = true;
            } else if bytes[cursor] == b'"' {
                string = false;
            }
            cursor += 1;
            continue;
        }
        if character {
            if character_escape {
                character_escape = false;
            } else if bytes[cursor] == b'\\' {
                character_escape = true;
            } else if bytes[cursor] == b'\'' {
                character = false;
            }
            cursor += 1;
            continue;
        }

        if bytes.get(cursor..cursor + 2) == Some(b"//") {
            line_comment = true;
            cursor += 2;
        } else if bytes.get(cursor..cursor + 2) == Some(b"/*") {
            block_comment_depth = 1;
            cursor += 2;
        } else if let Some((hashes, after_opening)) = raw_string_start(bytes, cursor) {
            raw_string_hashes = Some(hashes);
            cursor = after_opening;
        } else if bytes[cursor] == b'"' {
            string = true;
            cursor += 1;
        } else if bytes[cursor] == b'\'' && char_literal_start(source, cursor) {
            character = true;
            cursor += 1;
        } else {
            match bytes[cursor] {
                b'{' | b'(' | b'[' => delimiter_stack.push((bytes[cursor], cursor)),
                b'}' | b')' | b']' => {
                    let expected = match bytes[cursor] {
                        b'}' => b'{',
                        b')' => b'(',
                        b']' => b'[',
                        _ => unreachable!(),
                    };
                    let (opening, _) = delimiter_stack.pop()?;
                    if opening != expected {
                        return None;
                    }
                }
                _ => {}
            }
            cursor += 1;
        }
    }

    if line_comment
        || block_comment_depth != 0
        || raw_string_hashes.is_some()
        || string
        || character
        || delimiter_stack
            .iter()
            .any(|(delimiter, _)| *delimiter != b'{')
    {
        return None;
    }
    Some(
        delimiter_stack
            .into_iter()
            .map(|(_, offset)| offset)
            .collect(),
    )
}

fn last_code_construct_is_outer_attribute(source: &str) -> Option<bool> {
    let bytes = source.as_bytes();
    let mut cursor = 0;
    let mut square_attribute_stack = Vec::new();
    let mut last_code = None;
    let mut last_construct_is_attribute = false;
    let mut block_comment_depth = 0_usize;
    let mut line_comment = false;
    let mut string = false;
    let mut string_escape = false;
    let mut character = false;
    let mut character_escape = false;
    let mut raw_string_hashes = None;

    while cursor < bytes.len() {
        if line_comment {
            if bytes[cursor] == b'\n' || bytes[cursor] == b'\r' {
                line_comment = false;
            }
            cursor += 1;
            continue;
        }
        if block_comment_depth != 0 {
            if bytes.get(cursor..cursor + 2) == Some(b"/*") {
                block_comment_depth += 1;
                cursor += 2;
            } else if bytes.get(cursor..cursor + 2) == Some(b"*/") {
                block_comment_depth -= 1;
                cursor += 2;
            } else {
                cursor += 1;
            }
            continue;
        }
        if let Some(hashes) = raw_string_hashes {
            let terminator_end = cursor + 1 + hashes;
            if bytes[cursor] == b'"'
                && terminator_end <= bytes.len()
                && bytes[cursor + 1..terminator_end]
                    .iter()
                    .all(|byte| *byte == b'#')
            {
                raw_string_hashes = None;
                last_code = Some(terminator_end - 1);
                last_construct_is_attribute = false;
                cursor = terminator_end;
            } else {
                cursor += 1;
            }
            continue;
        }
        if string {
            if string_escape {
                string_escape = false;
            } else if bytes[cursor] == b'\\' {
                string_escape = true;
            } else if bytes[cursor] == b'"' {
                string = false;
                last_code = Some(cursor);
            }
            cursor += 1;
            continue;
        }
        if character {
            if character_escape {
                character_escape = false;
            } else if bytes[cursor] == b'\\' {
                character_escape = true;
            } else if bytes[cursor] == b'\'' {
                character = false;
                last_code = Some(cursor);
            }
            cursor += 1;
            continue;
        }

        if let Some(length) = rust_whitespace_len(source, cursor) {
            cursor += length;
        } else if bytes.get(cursor..cursor + 2) == Some(b"//") {
            line_comment = true;
            cursor += 2;
        } else if bytes.get(cursor..cursor + 2) == Some(b"/*") {
            block_comment_depth = 1;
            cursor += 2;
        } else if let Some((hashes, after_opening)) = raw_string_start(bytes, cursor) {
            raw_string_hashes = Some(hashes);
            last_code = Some(after_opening - 1);
            last_construct_is_attribute = false;
            cursor = after_opening;
        } else if bytes[cursor] == b'"' {
            string = true;
            last_code = Some(cursor);
            last_construct_is_attribute = false;
            cursor += 1;
        } else if bytes[cursor] == b'\'' && char_literal_start(source, cursor) {
            character = true;
            last_code = Some(cursor);
            last_construct_is_attribute = false;
            cursor += 1;
        } else {
            match bytes[cursor] {
                b'[' => {
                    square_attribute_stack
                        .push(last_code.is_some_and(|offset| bytes[offset] == b'#'));
                    last_construct_is_attribute = false;
                }
                b']' => {
                    last_construct_is_attribute = square_attribute_stack.pop()?;
                }
                _ => last_construct_is_attribute = false,
            }
            last_code = Some(cursor);
            cursor += 1;
        }
    }

    (!line_comment
        && block_comment_depth == 0
        && raw_string_hashes.is_none()
        && !string
        && !character
        && square_attribute_stack.is_empty())
    .then_some(last_construct_is_attribute)
}

fn rust_code_offset_after_trivia(source: &str, mut cursor: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if cursor > bytes.len() || !source.is_char_boundary(cursor) {
        return None;
    }

    loop {
        while let Some(length) = rust_whitespace_len(source, cursor) {
            cursor += length;
        }
        if bytes.get(cursor..cursor + 2) == Some(b"//") {
            cursor += 2;
            while bytes
                .get(cursor)
                .is_some_and(|byte| *byte != b'\n' && *byte != b'\r')
            {
                cursor += 1;
            }
            continue;
        }
        if bytes.get(cursor..cursor + 2) == Some(b"/*") {
            let mut depth = 1_usize;
            cursor += 2;
            while depth != 0 {
                if bytes.get(cursor..cursor + 2) == Some(b"/*") {
                    depth += 1;
                    cursor += 2;
                } else if bytes.get(cursor..cursor + 2) == Some(b"*/") {
                    depth -= 1;
                    cursor += 2;
                } else if cursor == bytes.len() {
                    return None;
                } else {
                    cursor += 1;
                }
            }
            continue;
        }
        return Some(cursor);
    }
}

fn starts_with_inner_attribute(source: &str, offset: usize) -> Option<bool> {
    let bytes = source.as_bytes();
    let hash = rust_code_offset_after_trivia(source, offset)?;
    if bytes.get(hash) != Some(&b'#') {
        return Some(false);
    }
    let bang = rust_code_offset_after_trivia(source, hash + 1)?;
    if bytes.get(bang) != Some(&b'!') {
        return Some(false);
    }
    let opening = rust_code_offset_after_trivia(source, bang + 1)?;
    Some(bytes.get(opening) == Some(&b'['))
}

fn crate_starts_with_inner_attribute(source: &str) -> Option<bool> {
    let bytes = source.as_bytes();
    let offset = if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
        3
    } else {
        0
    };
    if starts_with_inner_attribute(source, offset)? {
        return Some(true);
    }

    let first = rust_code_offset_after_trivia(source, offset)?;
    if bytes.get(first..first + 2) != Some(b"#!") {
        return Some(false);
    }
    let after_shebang = bytes[first..]
        .iter()
        .position(|byte| *byte == b'\n' || *byte == b'\r')
        .map_or(bytes.len(), |line_offset| first + line_offset);
    starts_with_inner_attribute(source, after_shebang)
}

fn expected_evidence_test_brace_stack(source: &str, source_path: &str) -> Option<Vec<usize>> {
    if crate_starts_with_inner_attribute(source)? {
        return None;
    }
    match source_path {
        "compiler/crates/orangec/tests/cli.rs"
        | "compiler/crates/orangec/tests/s2_conformance.rs" => Some(Vec::new()),
        _ => {
            const TEST_MODULE: &str = "#[cfg(test)]\nmod tests {";
            let modules = source.match_indices(TEST_MODULE).collect::<Vec<_>>();
            let [(module_offset, _)] = modules.as_slice() else {
                return None;
            };
            if last_code_construct_is_outer_attribute(&source[..*module_offset]) != Some(false) {
                return None;
            }
            let opening = *module_offset + TEST_MODULE.len() - 1;
            if starts_with_inner_attribute(source, opening + 1)? {
                return None;
            }
            (rust_code_brace_stack_at(source, opening) == Some(Vec::new())).then_some(vec![opening])
        }
    }
}

fn cargo_test_harness_contract_is_exact(
    workspace_manifest: &str,
    compiler_manifest: &str,
    orangec_manifest: &str,
) -> bool {
    workspace_manifest == EXPECTED_WORKSPACE_MANIFEST
        && compiler_manifest == EXPECTED_COMPILER_MANIFEST
        && orangec_manifest == EXPECTED_ORANGEC_MANIFEST
}

fn compiler_unit_harness_is_unconditional(source: &str) -> bool {
    if crate_starts_with_inner_attribute(source) != Some(false) {
        return false;
    }
    ["pub mod lexer;", "pub mod parser;", "pub mod source;"]
        .into_iter()
        .all(|declaration| {
            let offsets = source
                .match_indices(declaration)
                .map(|(offset, _)| offset)
                .collect::<Vec<_>>();
            let [offset] = offsets.as_slice() else {
                return false;
            };
            rust_code_brace_stack_at(source, *offset) == Some(Vec::new())
                && last_code_construct_is_outer_attribute(&source[..*offset]) == Some(false)
        })
}

fn exact_test_declaration(source_path: &str, test: &str) -> String {
    let indentation = evidence_test_indentation(source_path);
    format!("{indentation}#[test]\n{indentation}fn {test}(")
}

fn evidence_layers(source_path: &str, test: &str) -> u16 {
    let broad = match source_path {
        "compiler/crates/orange-compiler/src/source.rs" => SOURCE_UNIT_EVIDENCE,
        "compiler/crates/orange-compiler/src/lexer.rs" => LEXER_UNIT_EVIDENCE,
        "compiler/crates/orange-compiler/src/parser.rs" => PARSER_UNIT_EVIDENCE,
        "compiler/crates/orangec/tests/cli.rs" => CLI_EVIDENCE,
        "compiler/crates/orangec/tests/s2_conformance.rs" => CONFORMANCE_EVIDENCE,
        _ => 0,
    };
    let injected = match (source_path, test) {
        (
            "compiler/crates/orange-compiler/src/lexer.rs",
            "ordinary_token_reservation_failure_discards_partial_tokens_and_preserves_eof"
            | "diagnostic_vector_reservation_failure_is_an_allocation_free_lexical_failure",
        )
        | (
            "compiler/crates/orange-compiler/src/parser.rs",
            "diagnostic_vector_reservation_failure_returns_no_ast_or_diagnostics"
            | "module_function_reservation_failure_returns_no_partial_ast"
            | "enforces_internal_event_and_node_limits_without_large_inputs",
        ) => INJECTED_RESOURCE_EVIDENCE,
        _ => 0,
    };
    broad | injected
}

fn layer_names(layers: u16) -> Vec<&'static str> {
    [
        (CONFORMANCE_EVIDENCE, "conformance"),
        (CLI_EVIDENCE, "CLI"),
        (SOURCE_UNIT_EVIDENCE, "source unit"),
        (LEXER_UNIT_EVIDENCE, "lexer unit"),
        (PARSER_UNIT_EVIDENCE, "parser unit"),
        (INJECTED_RESOURCE_EVIDENCE, "injected resource"),
    ]
    .into_iter()
    .filter_map(|(layer, name)| (layers & layer != 0).then_some(name))
    .collect()
}

fn lex_text(text: &str) -> (SourceMap, Lexed) {
    let mut sources = SourceMap::new();
    let id = sources.add("s2.or", text).unwrap();
    let lexed = lex(sources.get(id).unwrap(), Edition::E2026);
    (sources, lexed)
}

fn parse_text(text: &str) -> (SourceMap, Lexed, ParseResult) {
    let mut sources = SourceMap::new();
    let id = sources.add("s2.or", text).unwrap();
    let source = sources.get(id).unwrap();
    let lexed = lex(source, Edition::E2026);
    let parsed = parse(source, &lexed);
    (sources, lexed, parsed)
}

#[test]
fn s2_named_evidence_scanner_rejects_noncode_and_nested_lookalikes() {
    const UNIT_PATH: &str = "compiler/crates/orange-compiler/src/source.rs";
    const ROOT_PATH: &str = "compiler/crates/orangec/tests/cli.rs";

    for whitespace in [
        '\u{0009}', '\u{000a}', '\u{000b}', '\u{000c}', '\u{000d}', '\u{0020}', '\u{0085}',
        '\u{200e}', '\u{200f}', '\u{2028}', '\u{2029}',
    ] {
        let encoded = whitespace.to_string();
        assert_eq!(
            rust_whitespace_len(&encoded, 0),
            Some(whitespace.len_utf8())
        );
    }
    assert_eq!(rust_whitespace_len("\u{00a0}", 0), None);

    let root = "#[test]\nfn real() {}\n";
    assert_eq!(rust_code_brace_stack_at(root, 0), Some(Vec::new()));
    assert_eq!(
        expected_evidence_test_brace_stack(root, ROOT_PATH),
        Some(Vec::new())
    );

    let unit = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn real() {}\n}\n";
    let unit_test = unit.find("    #[test]").unwrap();
    assert_eq!(
        rust_code_brace_stack_at(unit, unit_test),
        expected_evidence_test_brace_stack(unit, UNIT_PATH)
    );

    for noncode in [
        "/* #[test]\nfn fake() {} */\n",
        "// #[test]\n// fn fake() {}\n",
        "const LOOKALIKE: &str = \"#[test]\\nfn fake() {}\";\n",
    ] {
        let offset = noncode.find("#[test]").unwrap();
        assert_eq!(rust_code_brace_stack_at(noncode, offset), None);
    }

    let nested = "fn wrapper() {\n    #[test]\n    fn fake() {}\n}\n";
    let nested_test = nested.find("    #[test]").unwrap();
    assert_ne!(
        rust_code_brace_stack_at(nested, nested_test),
        Some(Vec::new())
    );

    for (opening, closing) in [('(', ')'), ('[', ']')] {
        let macro_wrapped = format!("discard!{opening}\n#[test]\nfn fake() {{}}\n{closing};\n");
        let wrapped_test = macro_wrapped.find("#[test]").unwrap();
        assert_eq!(rust_code_brace_stack_at(&macro_wrapped, wrapped_test), None);
    }

    for attribute in [
        "#[cfg(any())]",
        "#[ignore]",
        "# [cfg(any())]",
        "#\u{200e}[cfg(any())]",
        "# /* nested /* separator */ comment */ [ignore]",
    ] {
        let controlled =
            format!("fn before() {{\n}}\n{attribute}\n#[test]\nfn controlled() {{}}\n");
        let test = controlled.find("#[test]").unwrap();
        assert_eq!(
            last_code_construct_is_outer_attribute(&controlled[..test]),
            Some(true)
        );
    }

    let alternate_modules = concat!(
        "#[cfg(test)]\nmod first {}\n",
        "#[cfg(test)]\nmod second {}\n",
    );
    assert_eq!(
        expected_evidence_test_brace_stack(alternate_modules, UNIT_PATH),
        None
    );

    for additional_attribute in [
        "#[cfg(any())]\n",
        "#[cfg(any())]\n// separator\n",
        "#[cfg(any())]\n/* separator */\n",
        "# [cfg(any())]\n/* nested /* separator */ comment */\n",
    ] {
        let additionally_controlled_module = format!(
            "{additional_attribute}#[cfg(test)]\nmod tests {{\n    #[test]\n    fn fake() {{}}\n}}\n"
        );
        assert_eq!(
            expected_evidence_test_brace_stack(&additionally_controlled_module, UNIT_PATH),
            None
        );
    }

    for inner_attribute in [
        "#![cfg(any())]\n",
        "#![cfg_attr(test, cfg(any()))]\n",
        "#\u{200e}!\u{200f}[cfg(any())]\n",
        "# /* separator */ ! /* nested /* separator */ comment */ [cfg(any())]\n",
    ] {
        let disabled_unit_module = format!(
            "#[cfg(test)]\nmod tests {{\n{inner_attribute}    fn helper() {{}}\n    #[test]\n    fn fake() {{}}\n}}\n"
        );
        assert_eq!(
            expected_evidence_test_brace_stack(&disabled_unit_module, UNIT_PATH),
            None
        );

        let disabled_integration_target =
            format!("{inner_attribute}fn helper() {{}}\n#[test]\nfn fake() {{}}\n");
        assert_eq!(
            expected_evidence_test_brace_stack(&disabled_integration_target, ROOT_PATH),
            None
        );
    }

    let shebang_disabled_integration_target =
        "#!/usr/bin/env false\n#![cfg(any())]\n#[test]\nfn fake() {}\n";
    assert_eq!(
        expected_evidence_test_brace_stack(shebang_disabled_integration_target, ROOT_PATH),
        None
    );

    assert!(cargo_test_harness_contract_is_exact(
        EXPECTED_WORKSPACE_MANIFEST,
        EXPECTED_COMPILER_MANIFEST,
        EXPECTED_ORANGEC_MANIFEST
    ));
    let compiler_tests_disabled = format!("{EXPECTED_COMPILER_MANIFEST}\n[lib]\ntest = false\n");
    assert!(!cargo_test_harness_contract_is_exact(
        EXPECTED_WORKSPACE_MANIFEST,
        &compiler_tests_disabled,
        EXPECTED_ORANGEC_MANIFEST
    ));
    let orangec_autotests_disabled =
        EXPECTED_ORANGEC_MANIFEST.replacen("[package]\n", "[package]\nautotests = false\n", 1);
    assert!(!cargo_test_harness_contract_is_exact(
        EXPECTED_WORKSPACE_MANIFEST,
        EXPECTED_COMPILER_MANIFEST,
        &orangec_autotests_disabled
    ));
    let workspace_member_removed =
        EXPECTED_WORKSPACE_MANIFEST.replace("  \"crates/orangec\",\n", "");
    assert!(!cargo_test_harness_contract_is_exact(
        &workspace_member_removed,
        EXPECTED_COMPILER_MANIFEST,
        EXPECTED_ORANGEC_MANIFEST
    ));

    let compiler_root = "pub mod lexer;\npub mod parser;\npub mod source;\n";
    assert!(compiler_unit_harness_is_unconditional(compiler_root));
    assert!(!compiler_unit_harness_is_unconditional(&format!(
        "#![cfg(not(test))]\n{compiler_root}"
    )));
    for module in ["lexer", "parser", "source"] {
        let declaration = format!("pub mod {module};");
        let controlled = compiler_root.replace(
            &declaration,
            &format!("#\u{200e}[cfg(not(test))]\n{declaration}"),
        );
        assert!(!compiler_unit_harness_is_unconditional(&controlled));
    }

    let ordinary_preceding_item = concat!(
        "const PREVIOUS: [u8; 1] = [0];\n",
        "// separator\n",
        "#[cfg(test)]\n",
        "mod tests {\n",
        "    #[test]\n",
        "    fn real() {}\n",
        "}\n",
    );
    assert!(expected_evidence_test_brace_stack(ordinary_preceding_item, UNIT_PATH).is_some());
}

#[test]
fn s2_rule_index_is_exact_and_covered() {
    assert!(
        cargo_test_harness_contract_is_exact(
            WORKSPACE_MANIFEST,
            COMPILER_MANIFEST,
            ORANGEC_MANIFEST
        ),
        "Cargo workspace and test-target activation contract changed"
    );
    assert!(
        compiler_unit_harness_is_unconditional(COMPILER_LIB_SOURCE),
        "mapped compiler unit modules are not unconditionally registered"
    );

    let documented = documented_rules();
    let documented_ids = documented.iter().map(|(id, _)| *id).collect::<Vec<_>>();
    let required_ids = RULES.iter().map(|rule| rule.id).collect::<Vec<_>>();
    assert_eq!(
        required_ids.iter().copied().collect::<BTreeSet<_>>().len(),
        required_ids.len(),
        "duplicate required S2 rule"
    );
    assert_eq!(documented_ids, required_ids);
    assert_eq!(
        documented_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        documented_ids.len(),
        "duplicate documented S2 rule"
    );

    for (id, label) in documented {
        assert_eq!(label, required_evidence_label(id), "{id} evidence label");
        let requirement = RULES.iter().find(|rule| rule.id == id).unwrap();
        assert_eq!(
            requirement.layers,
            required_layers_for_label(label),
            "{id} evidence-layer capability mask"
        );
    }

    let known_rules = required_ids.iter().copied().collect::<BTreeSet<_>>();
    let mut observed_layers = BTreeMap::new();
    let mut named_tests = BTreeSet::new();
    for evidence in TEST_EVIDENCE {
        assert!(
            named_tests.insert((evidence.source_path, evidence.test)),
            "duplicate S2 evidence mapping for {}::{}",
            evidence.source_path,
            evidence.test
        );
        assert!(!evidence.rules.is_empty(), "unbound S2 evidence");
        assert_eq!(
            evidence
                .rules
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
                .len(),
            evidence.rules.len(),
            "duplicate rule on {}::{}",
            evidence.source_path,
            evidence.test
        );

        let source = evidence_source(evidence.source_path)
            .unwrap_or_else(|| panic!("unknown evidence source {}", evidence.source_path));
        let expected_stack = expected_evidence_test_brace_stack(source, evidence.source_path)
            .unwrap_or_else(|| panic!("invalid test module in {}", evidence.source_path));
        let declaration = exact_test_declaration(evidence.source_path, evidence.test);
        let offsets = source
            .match_indices(&declaration)
            .map(|(offset, _)| offset)
            .collect::<Vec<_>>();
        assert_eq!(
            offsets.len(),
            1,
            "expected one unconditional test declaration for {}::{}",
            evidence.source_path,
            evidence.test
        );
        assert_eq!(
            rust_code_brace_stack_at(source, offsets[0]),
            Some(expected_stack),
            "{}::{} is outside the expected test harness",
            evidence.source_path,
            evidence.test
        );
        assert_eq!(
            last_code_construct_is_outer_attribute(&source[..offsets[0]]),
            Some(false),
            "{}::{} has an additional controlling attribute or malformed prefix",
            evidence.source_path,
            evidence.test
        );

        let layers = evidence_layers(evidence.source_path, evidence.test);
        assert_ne!(layers, 0, "unclassified S2 evidence");
        for rule in evidence.rules {
            assert!(known_rules.contains(rule), "unknown evidence rule {rule}");
            observed_layers
                .entry(*rule)
                .and_modify(|observed| *observed |= layers)
                .or_insert(layers);
        }
    }

    for rule in RULES {
        let observed = observed_layers.get(rule.id).copied().unwrap_or_default();
        assert_eq!(
            observed & rule.layers,
            rule.layers,
            "{} missing evidence layers {:?}; observed {:?}",
            rule.id,
            layer_names(rule.layers & !observed),
            layer_names(observed)
        );
    }
}

#[test]
fn s2_ascii_trivia_uppercase_radices_and_base_grammar_are_repeatable() {
    let integer_text = "\t0B1010\r\n0XCA_FE\r1_000\n";
    let (integer_sources, first_integers) = lex_text(integer_text);
    let integer_source = integer_sources.iter().next().unwrap();
    let second_integers = lex(integer_source, Edition::E2026);
    assert_eq!(first_integers, second_integers);
    assert_eq!(first_integers.diagnostics(), []);
    assert_eq!(
        first_integers
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>(),
        [
            TokenKind::Integer,
            TokenKind::Integer,
            TokenKind::Integer,
            TokenKind::Eof,
        ]
    );
    assert_eq!(
        first_integers
            .tokens()
            .iter()
            .map(|token| token.lexeme(integer_source).unwrap())
            .collect::<Vec<_>>(),
        ["0B1010", "0XCA_FE", "1_000", ""]
    );

    let base_text = concat!(
        "\tedition\t2026;\r\n",
        "module\tDemo\t{\r",
        "/* outer /* inner */ done */\n",
        "spec\tone\t(\t)\t{\t}\r\n",
        "impl two() {}\n",
        "}\t",
    );
    let (sources, lexed, first) = parse_text(base_text);
    let source = sources.iter().next().unwrap();
    let second = parse(source, &lexed);
    assert_eq!(first, second);
    assert_eq!(lexed.diagnostics(), []);
    assert_eq!(first.diagnostics(), []);
    let ast = first.ast().unwrap();
    let functions = ast.module().functions();
    assert_eq!(ast.module().name().text(), "Demo");
    assert_eq!(functions.len(), 2);
    assert_eq!(functions[0].kind(), FunctionKind::Spec);
    assert_eq!(functions[1].kind(), FunctionKind::Impl);
    assert_eq!(
        source.slice(functions[0].span()),
        Some("spec\tone\t(\t)\t{\t}")
    );
}

#[test]
fn s2_supported_string_escape_set_is_exact_and_repeatable() {
    let valid = concat!(
        "\"", "\\\"", "\\\\", "\\n", "\\r", "\\t", "\\0", "\\x00", "\\xAf", "\"",
    );
    let (sources, first) = lex_text(valid);
    let source = sources.iter().next().unwrap();
    let second = lex(source, Edition::E2026);
    assert_eq!(first, second);
    assert_eq!(first.diagnostics(), []);
    assert_eq!(
        first
            .tokens()
            .iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>(),
        [TokenKind::String, TokenKind::Eof]
    );
    assert_eq!(first.tokens()[0].lexeme(source), Some(valid));

    for (escape, responsible) in [
        (r"\q", r"\q"),
        (r"\u", r"\u"),
        (r"\x", r"\x"),
        (r"\x0", r"\x0"),
    ] {
        let text = format!("\"{escape}\"");
        let (invalid_sources, first_invalid) = lex_text(&text);
        let invalid_source = invalid_sources.iter().next().unwrap();
        let second_invalid = lex(invalid_source, Edition::E2026);
        assert_eq!(first_invalid, second_invalid, "{escape}");
        assert_eq!(first_invalid.diagnostics().len(), 1, "{escape}");
        assert_eq!(
            first_invalid.diagnostics()[0].code(),
            DiagnosticCode::InvalidEscape,
            "{escape}"
        );
        assert_eq!(
            invalid_source.slice(first_invalid.diagnostics()[0].primary_span()),
            Some(responsible),
            "{escape}"
        );
    }
}

#[test]
fn s2_punctuation_longest_matches_are_exact_and_repeatable() {
    const CASES: &[(&str, &[(&str, TokenKind)])] = &[
        ("...", &[("..", TokenKind::DotDot), (".", TokenKind::Dot)]),
        (
            ":::",
            &[("::", TokenKind::DoubleColon), (":", TokenKind::Colon)],
        ),
        (
            "&&&",
            &[("&&", TokenKind::AmpAmp), ("&", TokenKind::Ampersand)],
        ),
        (
            "|||",
            &[("||", TokenKind::PipePipe), ("|", TokenKind::Pipe)],
        ),
        (
            "===",
            &[("==", TokenKind::EqualEqual), ("=", TokenKind::Equal)],
        ),
        (
            "!==",
            &[("!=", TokenKind::BangEqual), ("=", TokenKind::Equal)],
        ),
        (
            "<==",
            &[("<=", TokenKind::LessEqual), ("=", TokenKind::Equal)],
        ),
        (
            ">==",
            &[(">=", TokenKind::GreaterEqual), ("=", TokenKind::Equal)],
        ),
        ("->-", &[("->", TokenKind::Arrow), ("-", TokenKind::Minus)]),
        (
            "=>=",
            &[("=>", TokenKind::FatArrow), ("=", TokenKind::Equal)],
        ),
    ];
    let text = CASES
        .iter()
        .map(|(source, _)| *source)
        .collect::<Vec<_>>()
        .join(" ");
    let expected = CASES
        .iter()
        .flat_map(|(_, tokens)| tokens.iter().copied())
        .collect::<Vec<_>>();
    let (sources, first) = lex_text(&text);
    let source = sources.iter().next().unwrap();
    let second = lex(source, Edition::E2026);
    assert_eq!(first, second);
    assert_eq!(first.diagnostics(), []);
    assert_eq!(first.tokens().len(), expected.len() + 1);
    for (token, (spelling, kind)) in first.tokens().iter().zip(expected) {
        assert_eq!(token.kind, kind, "{spelling}");
        assert_eq!(token.lexeme(source), Some(spelling), "{spelling}");
    }
    assert_eq!(first.tokens().last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn s2_base_grammar_diagnostics_are_exact_and_repeatable() {
    #[derive(Clone, Copy)]
    struct ExpectedDiagnostic {
        code: DiagnosticCode,
        start: u32,
        end: u32,
        responsible: &'static str,
    }

    struct Case {
        context: &'static str,
        text: &'static str,
        diagnostics: &'static [ExpectedDiagnostic],
    }

    const CASES: &[Case] = &[
        Case {
            context: "missing edition keyword",
            text: "2026; module m {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 0,
                end: 4,
                responsible: "2026",
            }],
        },
        Case {
            context: "missing edition value",
            text: "edition; module m {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 7,
                end: 8,
                responsible: ";",
            }],
        },
        Case {
            context: "missing edition semicolon",
            text: "edition 2026 module m {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 13,
                end: 19,
                responsible: "module",
            }],
        },
        Case {
            context: "missing module keyword",
            text: "edition 2026; m {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 14,
                end: 15,
                responsible: "m",
            }],
        },
        Case {
            context: "missing module name",
            text: "edition 2026; module {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 21,
                end: 22,
                responsible: "{",
            }],
        },
        Case {
            context: "missing module opening brace",
            text: "edition 2026; module m spec f() {} }",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 23,
                end: 27,
                responsible: "spec",
            }],
        },
        Case {
            context: "missing function kind",
            text: "edition 2026; module m { f() {} }",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedFunctionDeclaration,
                start: 25,
                end: 26,
                responsible: "f",
            }],
        },
        Case {
            context: "missing function name",
            text: "edition 2026; module m { spec () {} }",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 30,
                end: 31,
                responsible: "(",
            }],
        },
        Case {
            context: "missing left parenthesis",
            text: "edition 2026; module m { spec f) {} }",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 31,
                end: 32,
                responsible: ")",
            }],
        },
        Case {
            context: "missing right parenthesis",
            text: "edition 2026; module m { spec f( {} }",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 33,
                end: 34,
                responsible: "{",
            }],
        },
        Case {
            context: "missing empty body",
            text: "edition 2026; module m { spec f() } }",
            diagnostics: &[
                ExpectedDiagnostic {
                    code: DiagnosticCode::ExpectedSyntax,
                    start: 34,
                    end: 35,
                    responsible: "}",
                },
                ExpectedDiagnostic {
                    code: DiagnosticCode::TrailingSyntax,
                    start: 36,
                    end: 37,
                    responsible: "}",
                },
            ],
        },
        Case {
            context: "nonempty legacy body",
            text: "edition 2026; module m { spec f() { x } }",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 36,
                end: 37,
                responsible: "x",
            }],
        },
        Case {
            context: "unterminated function and module",
            text: "edition 2026; module m { spec f() {",
            diagnostics: &[
                ExpectedDiagnostic {
                    code: DiagnosticCode::ExpectedSyntax,
                    start: 35,
                    end: 35,
                    responsible: "",
                },
                ExpectedDiagnostic {
                    code: DiagnosticCode::ExpectedSyntax,
                    start: 35,
                    end: 35,
                    responsible: "",
                },
            ],
        },
        Case {
            context: "unterminated module after a complete function",
            text: "edition 2026; module m { spec f() {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 36,
                end: 36,
                responsible: "",
            }],
        },
        Case {
            context: "nonexact edition",
            text: "edition 02026; module m {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::UnsupportedSourceEdition,
                start: 8,
                end: 13,
                responsible: "02026",
            }],
        },
        Case {
            context: "trailing module",
            text: "edition 2026; module m {} module n {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::TrailingSyntax,
                start: 26,
                end: 32,
                responsible: "module",
            }],
        },
        Case {
            context: "reserved module name",
            text: "edition 2026; module game {}",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 21,
                end: 25,
                responsible: "game",
            }],
        },
        Case {
            context: "reserved function name",
            text: "edition 2026; module m { spec proof() {} }",
            diagnostics: &[ExpectedDiagnostic {
                code: DiagnosticCode::ExpectedSyntax,
                start: 30,
                end: 35,
                responsible: "proof",
            }],
        },
    ];

    for case in CASES {
        let (sources, lexed, first) = parse_text(case.text);
        let source = sources.iter().next().unwrap();
        let second = parse(source, &lexed);
        assert_eq!(lexed.diagnostics(), [], "{}", case.context);
        assert_eq!(first, second, "{}", case.context);
        assert!(first.has_errors(), "{}", case.context);
        assert!(first.ast().is_none(), "{}", case.context);
        assert_eq!(
            first.diagnostics().len(),
            case.diagnostics.len(),
            "{}",
            case.context
        );
        for (diagnostic, expected) in first.diagnostics().iter().zip(case.diagnostics) {
            assert_eq!(diagnostic.code(), expected.code, "{}", case.context);
            assert_eq!(
                diagnostic.primary_span().start().bytes(),
                expected.start,
                "{}",
                case.context
            );
            assert_eq!(
                diagnostic.primary_span().end().bytes(),
                expected.end,
                "{}",
                case.context
            );
            assert_eq!(
                source.slice(diagnostic.primary_span()),
                Some(expected.responsible),
                "{}",
                case.context
            );
        }
    }
}

#[test]
fn s2_lexical_failure_skips_parsing_repeatably() {
    let text = "@ edition 2026; module broken { spec f( {}";
    let (sources, first_lexed) = lex_text(text);
    let source = sources.iter().next().unwrap();
    let second_lexed = lex(source, Edition::E2026);
    assert_eq!(first_lexed, second_lexed);
    assert_eq!(first_lexed.diagnostics().len(), 1);
    assert_eq!(
        first_lexed.diagnostics()[0].code(),
        DiagnosticCode::UnexpectedCharacter
    );

    let first = parse(source, &first_lexed);
    let second = parse(source, &second_lexed);
    assert_eq!(first, second);
    assert!(first.has_errors());
    assert!(first.ast().is_none());
    assert_eq!(first.diagnostics(), []);
}

#[test]
fn s2_same_kind_duplicate_names_remain_distinct_syntax_nodes() {
    let text = concat!(
        "edition 2026; module duplicates { ",
        "spec same() {} ",
        "spec same() {} ",
        "}",
    );
    let (sources, lexed, first) = parse_text(text);
    let source = sources.iter().next().unwrap();
    let second = parse(source, &lexed);

    assert_eq!(lexed.diagnostics(), []);
    assert_eq!(first, second);
    assert_eq!(first.diagnostics(), []);
    let functions = first.ast().unwrap().module().functions();
    assert_eq!(functions.len(), 2);
    assert_eq!(functions[0].kind(), FunctionKind::Spec);
    assert_eq!(functions[1].kind(), FunctionKind::Spec);
    assert_eq!(functions[0].name().text(), "same");
    assert_eq!(functions[1].name().text(), "same");
    assert_ne!(functions[0].name().span(), functions[1].name().span());
    assert_eq!(source.slice(functions[0].span()), Some("spec same() {}"));
    assert_eq!(source.slice(functions[1].span()), Some("spec same() {}"));
}
