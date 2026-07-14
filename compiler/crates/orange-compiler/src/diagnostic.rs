//! Stable, source-aware compiler diagnostics.

use std::fmt::{self, Write as _};

use crate::source::{SourceFile, SourceMap, Span};

macro_rules! define_diagnostic_codes {
    ($(#[$variant_doc:meta] $variant:ident => $code:literal,)+) => {
        /// A stable diagnostic identifier.
        ///
        /// Codes are part of the user-facing compiler interface. Existing meanings
        /// must not be silently reused when new compiler phases are added.
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub enum DiagnosticCode {
            $(#[$variant_doc] $variant,)+
        }

        impl DiagnosticCode {
            /// Returns the permanent printable code.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $code,)+
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl fmt::Display for DiagnosticCode {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

define_diagnostic_codes! {
    /// A character has no lexical meaning in the selected edition.
    UnexpectedCharacter => "ORC0001",
    /// A nested block comment reached the end of the file.
    UnterminatedBlockComment => "ORC0002",
    /// A quoted string reached a line ending or the end of the file.
    UnterminatedString => "ORC0003",
    /// A string contains an unsupported or incomplete escape.
    InvalidEscape => "ORC0004",
    /// An integer has invalid digits or separators for its base.
    MalformedInteger => "ORC0005",
    /// A source exceeds the deterministic non-trivia token budget.
    LexicalTokenLimit => "ORC0006",
    /// Further lexical errors were suppressed after the reporting budget.
    TooManyLexicalErrors => "ORC0007",
    /// The lexer could not retain its bounded token-stream representation.
    LexicalResourceLimit => "ORC0008",
    /// A token required by the active grammar production was not present.
    ExpectedSyntax => "ORC0101",
    /// The source edition declaration is not exactly `edition 2026;`.
    UnsupportedSourceEdition => "ORC0102",
    /// A module member is not a `spec` or `impl` function declaration.
    ExpectedFunctionDeclaration => "ORC0103",
    /// Syntax follows the single module allowed in one source file.
    TrailingSyntax => "ORC0104",
    /// Further parser errors were suppressed after the reporting budget.
    TooManySyntaxErrors => "ORC0105",
    /// A deterministic parser resource budget was exhausted.
    ParserResourceLimit => "ORC0106",
    /// Parsing received lexer output owned by another source.
    InvalidParserInput => "ORC0107",
    /// A second function declaration conflicts in its declaration namespace.
    DuplicateFunction => "ORC0201",
    /// A typed literal body appeared on a function kind without semantics.
    UnsupportedTypedFunction => "ORC0202",
    /// A typed literal names a type outside the admitted semantic fragment.
    UnsupportedType => "ORC0203",
    /// A `Word` type uses a width other than the admitted exact width.
    UnsupportedWordWidth => "ORC0204",
    /// An exact integer magnitude exceeds the semantic representation budget.
    IntegerMagnitudeLimit => "ORC0205",
    /// A fixed-width word literal is negative.
    NegativeWordLiteral => "ORC0206",
    /// A fixed-width word literal is outside its admitted unsigned range.
    WordLiteralOutOfRange => "ORC0207",
    /// Further semantic errors were suppressed after the reporting budget.
    TooManySemanticErrors => "ORC0208",
    /// A deterministic semantic-analysis resource budget was exhausted.
    SemanticResourceLimit => "ORC0209",
    /// Semantic analysis received a syntax tree owned by another source.
    InvalidSemanticInput => "ORC0210",
    /// A deterministic reference-evaluation resource budget was exhausted.
    EvaluationResourceLimit => "ORC0301",
}

macro_rules! define_severities {
    ($($(#[$variant_doc:meta])* $variant:ident => $name:literal,)+) => {
        /// The impact of a compiler diagnostic.
        #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
        pub enum Severity {
            $($(#[$variant_doc])* $variant,)+
        }

        impl Severity {
            /// Returns the stable lowercase severity name.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl fmt::Display for Severity {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    };
}

define_severities! {
    /// Compilation cannot proceed successfully.
    Error => "error",
}

/// A labeled secondary source span attached to a diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondarySpan {
    span: Span,
    label: String,
}

impl SecondarySpan {
    /// Returns the secondary source span.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the concise label rendered beside the secondary underline.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

/// A compiler diagnostic with one primary and zero or more secondary spans.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    severity: Severity,
    code: DiagnosticCode,
    message: String,
    primary_span: Span,
    label: String,
    secondary_spans: Vec<SecondarySpan>,
    notes: Vec<String>,
}

impl Diagnostic {
    /// Creates an error at `primary_span`.
    #[must_use]
    pub fn error(code: DiagnosticCode, message: impl Into<String>, primary_span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            primary_span,
            label: String::new(),
            secondary_spans: Vec::new(),
            notes: Vec::new(),
        }
    }

    /// Sets the concise label shown beside the primary underline.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Appends a labeled secondary source span.
    #[must_use]
    pub fn with_secondary_span(mut self, span: Span, label: impl Into<String>) -> Self {
        self.secondary_spans.push(SecondarySpan {
            span,
            label: label.into(),
        });
        self
    }

    /// Appends a deterministic explanatory note.
    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Returns the severity.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    /// Returns the stable code.
    #[must_use]
    pub const fn code(&self) -> DiagnosticCode {
        self.code
    }

    /// Returns the primary human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the primary source span.
    #[must_use]
    pub const fn primary_span(&self) -> Span {
        self.primary_span
    }

    /// Returns the concise label rendered beside the primary underline.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the structured secondary source spans in construction order.
    #[must_use]
    pub fn secondary_spans(&self) -> &[SecondarySpan] {
        &self.secondary_spans
    }

    /// Returns explanatory notes in construction order.
    #[must_use]
    pub fn notes(&self) -> &[String] {
        &self.notes
    }
}

/// Renders diagnostics in a canonical total order over their rendered fields.
///
/// The returned text is empty for no diagnostics and otherwise ends in one
/// newline. Sources owned by `sources` retain their map-local insertion order;
/// spans from every other source map share the rendered `<unknown>` identity.
/// When no source excerpt is available, a nonempty primary label follows the
/// byte range so the fallback representation does not discard information.
/// Rendering uses no terminal color or locale-sensitive formatting.
#[must_use]
pub fn render_diagnostics(sources: &SourceMap, diagnostics: &[Diagnostic]) -> String {
    let mut ordered: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| OrderedDiagnostic::new(sources, diagnostic))
        .collect();
    // Equal keys contain every rendered field, so equal diagnostics are byte-identical.
    ordered.sort_unstable_by(|left, right| compare_diagnostics(sources, left, right));

    let mut rendered = String::new();
    for (index, ordered_diagnostic) in ordered.into_iter().enumerate() {
        if index != 0 {
            rendered.push('\n');
        }
        render_one(
            &mut rendered,
            sources,
            ordered_diagnostic.diagnostic,
            &ordered_diagnostic.secondary_spans,
        );
    }
    rendered
}

struct OrderedDiagnostic<'a> {
    diagnostic: &'a Diagnostic,
    secondary_spans: Vec<&'a SecondarySpan>,
}

impl<'a> OrderedDiagnostic<'a> {
    fn new(sources: &SourceMap, diagnostic: &'a Diagnostic) -> Self {
        let mut secondary_spans: Vec<_> = diagnostic.secondary_spans.iter().collect();
        // Equal keys render identically, so an in-place unstable sort preserves output.
        secondary_spans
            .sort_unstable_by(|left, right| compare_secondary_span(sources, left, right));
        Self {
            diagnostic,
            secondary_spans,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum RenderedSourceIdentity {
    Known(u32),
    Unknown,
}

fn rendered_span_key(sources: &SourceMap, span: Span) -> (RenderedSourceIdentity, u32, u32) {
    let identity = sources
        .get(span.source())
        .map_or(RenderedSourceIdentity::Unknown, |source| {
            RenderedSourceIdentity::Known(source.id().index())
        });
    (identity, span.start().bytes(), span.end().bytes())
}

fn compare_diagnostics(
    sources: &SourceMap,
    left: &OrderedDiagnostic<'_>,
    right: &OrderedDiagnostic<'_>,
) -> std::cmp::Ordering {
    let left_diagnostic = left.diagnostic;
    let right_diagnostic = right.diagnostic;
    rendered_span_key(sources, left_diagnostic.primary_span)
        .cmp(&rendered_span_key(sources, right_diagnostic.primary_span))
        .then_with(|| left_diagnostic.severity.cmp(&right_diagnostic.severity))
        .then_with(|| left_diagnostic.code.cmp(&right_diagnostic.code))
        .then_with(|| left_diagnostic.message.cmp(&right_diagnostic.message))
        .then_with(|| left_diagnostic.label.cmp(&right_diagnostic.label))
        .then_with(|| left_diagnostic.notes.cmp(&right_diagnostic.notes))
        .then_with(|| {
            compare_ordered_secondary_spans(sources, &left.secondary_spans, &right.secondary_spans)
        })
}

fn render_one(
    output: &mut String,
    sources: &SourceMap,
    diagnostic: &Diagnostic,
    secondary_spans: &[&SecondarySpan],
) {
    let _ = write!(
        output,
        "{}[{}]: ",
        diagnostic.severity.as_str(),
        diagnostic.code.as_str()
    );
    push_sanitized_inline(output, &diagnostic.message);
    output.push('\n');

    let Some(source) = sources.get(diagnostic.primary_span.source()) else {
        let _ = write!(
            output,
            " --> <unknown>:{}..{}",
            diagnostic.primary_span.start().bytes(),
            diagnostic.primary_span.end().bytes()
        );
        finish_fallback_primary(output, diagnostic);
        render_secondary_spans(output, sources, secondary_spans);
        render_notes(output, diagnostic);
        return;
    };
    let Some(location) = source.line_column(diagnostic.primary_span.start()) else {
        output.push_str(" --> ");
        push_sanitized_source_name(output, source);
        let _ = write!(
            output,
            ":{}..{}",
            diagnostic.primary_span.start().bytes(),
            diagnostic.primary_span.end().bytes()
        );
        finish_fallback_primary(output, diagnostic);
        render_secondary_spans(output, sources, secondary_spans);
        render_notes(output, diagnostic);
        return;
    };

    output.push_str(" --> ");
    push_sanitized_source_name(output, source);
    let _ = writeln!(output, ":{}:{}", location.line, location.column);
    let Some(line) = source.line_text(location.line) else {
        render_secondary_spans(output, sources, secondary_spans);
        render_notes(output, diagnostic);
        return;
    };

    let gutter_width = decimal_width(location.line);
    let (excerpt, caret_offset, caret_width) =
        render_excerpt(source, diagnostic.primary_span, location.line, line);
    let _ = writeln!(
        output,
        "{space:>width$} |",
        space = "",
        width = gutter_width
    );
    let _ = writeln!(output, "{} | {excerpt}", location.line);
    let _ = write!(
        output,
        "{space:>width$} | ",
        space = "",
        width = gutter_width
    );
    push_repeated(output, ' ', caret_offset);
    push_repeated(output, '^', caret_width);
    if !diagnostic.label.is_empty() {
        output.push(' ');
        push_sanitized_inline(output, &diagnostic.label);
    }
    output.push('\n');
    render_secondary_spans(output, sources, secondary_spans);
    render_notes(output, diagnostic);
}

fn finish_fallback_primary(output: &mut String, diagnostic: &Diagnostic) {
    if !diagnostic.label.is_empty() {
        output.push(' ');
        push_sanitized_inline(output, &diagnostic.label);
    }
    output.push('\n');
}

fn compare_secondary_span(
    sources: &SourceMap,
    left: &SecondarySpan,
    right: &SecondarySpan,
) -> std::cmp::Ordering {
    rendered_span_key(sources, left.span)
        .cmp(&rendered_span_key(sources, right.span))
        .then_with(|| left.label.cmp(&right.label))
}

fn compare_ordered_secondary_spans(
    sources: &SourceMap,
    left: &[&SecondarySpan],
    right: &[&SecondarySpan],
) -> std::cmp::Ordering {
    left.iter()
        .zip(right)
        .map(|(left, right)| compare_secondary_span(sources, left, right))
        .find(|ordering| !ordering.is_eq())
        .unwrap_or_else(|| left.len().cmp(&right.len()))
}

fn render_secondary_spans(
    output: &mut String,
    sources: &SourceMap,
    secondary_spans: &[&SecondarySpan],
) {
    for secondary in secondary_spans {
        render_secondary_span(output, sources, secondary);
    }
}

fn render_secondary_span(output: &mut String, sources: &SourceMap, secondary: &SecondarySpan) {
    let Some(source) = sources.get(secondary.span.source()) else {
        let _ = write!(
            output,
            " ::: <unknown>:{}..{} ",
            secondary.span.start().bytes(),
            secondary.span.end().bytes()
        );
        push_sanitized_inline(output, &secondary.label);
        output.push('\n');
        return;
    };
    let Some(location) = source.line_column(secondary.span.start()) else {
        output.push_str(" ::: ");
        push_sanitized_source_name(output, source);
        let _ = write!(
            output,
            ":{}..{} ",
            secondary.span.start().bytes(),
            secondary.span.end().bytes()
        );
        push_sanitized_inline(output, &secondary.label);
        output.push('\n');
        return;
    };
    output.push_str(" ::: ");
    push_sanitized_source_name(output, source);
    let _ = writeln!(output, ":{}:{}", location.line, location.column);
    let Some(line) = source.line_text(location.line) else {
        return;
    };
    let gutter_width = decimal_width(location.line);
    let (excerpt, caret_offset, caret_width) =
        render_excerpt(source, secondary.span, location.line, line);
    let _ = writeln!(
        output,
        "{space:>width$} |",
        space = "",
        width = gutter_width
    );
    let _ = writeln!(output, "{} | {excerpt}", location.line);
    let _ = write!(
        output,
        "{space:>width$} | ",
        space = "",
        width = gutter_width
    );
    push_repeated(output, ' ', caret_offset);
    push_repeated(output, '-', caret_width);
    output.push(' ');
    push_sanitized_inline(output, &secondary.label);
    output.push('\n');
}

fn render_notes(output: &mut String, diagnostic: &Diagnostic) {
    for note in &diagnostic.notes {
        output.push_str("  = note: ");
        push_sanitized_inline(output, note);
        output.push('\n');
    }
}

fn decimal_width(value: u32) -> usize {
    value
        .checked_ilog10()
        .and_then(|logarithm| usize::try_from(logarithm).ok())
        .unwrap_or(0)
        .saturating_add(1)
}

fn push_repeated(output: &mut String, character: char, count: usize) {
    output.extend(std::iter::repeat_n(character, count));
}

fn render_excerpt(
    source: &SourceFile,
    span: Span,
    one_based_line: u32,
    line: &str,
) -> (String, usize, usize) {
    const CONTEXT_BEFORE: usize = 40;
    const CONTEXT_AFTER: usize = 80;

    let Some(line_start) = source.line_start(one_based_line) else {
        return (String::new(), 0, 1);
    };
    let relative_start = usize::try_from(span.start().bytes().saturating_sub(line_start.bytes()))
        .unwrap_or(line.len())
        .min(line.len());
    if !line.is_char_boundary(relative_start) {
        return (String::new(), 0, 1);
    }

    let Some(before_start) = line.get(..relative_start) else {
        return (String::new(), 0, 1);
    };
    let mut window_start = relative_start;
    for (index, _) in before_start.char_indices().rev().take(CONTEXT_BEFORE) {
        window_start = index;
    }
    let Some(after_start) = line.get(relative_start..) else {
        return (String::new(), 0, 1);
    };
    let mut window_end = relative_start;
    for (index, character) in after_start.char_indices().take(CONTEXT_AFTER) {
        let Some(end) = relative_start
            .checked_add(index)
            .and_then(|end| end.checked_add(character.len_utf8()))
        else {
            return (String::new(), 0, 1);
        };
        window_end = end;
    }

    let left_truncated = window_start != 0;
    let right_truncated = window_end != line.len();
    let Some(window) = line.get(window_start..window_end) else {
        return (String::new(), 0, 1);
    };
    let mut excerpt = String::new();
    if left_truncated {
        excerpt.push_str("... ");
    }
    push_sanitized_inline(&mut excerpt, window);
    if right_truncated {
        excerpt.push_str(" ...");
    }

    let Some(before_caret) = line.get(window_start..relative_start) else {
        return (String::new(), 0, 1);
    };
    let Some(caret_offset) = usize::from(left_truncated)
        .checked_mul(4)
        .and_then(|ellipsis_width| ellipsis_width.checked_add(sanitized_width(before_caret)))
    else {
        return (String::new(), 0, 1);
    };
    let relative_end = usize::try_from(span.end().bytes().saturating_sub(line_start.bytes()))
        .unwrap_or(line.len())
        .min(window_end)
        .min(line.len());
    let caret_width =
        if span.is_empty() || relative_end < relative_start || !line.is_char_boundary(relative_end)
        {
            1
        } else {
            line.get(relative_start..relative_end)
                .map_or(1, sanitized_width)
                .max(1)
        };
    (excerpt, caret_offset, caret_width)
}

fn push_sanitized_inline(output: &mut String, text: &str) {
    for character in text.chars() {
        if character == '\\' || (!character.is_ascii_graphic() && character != ' ') {
            output.extend(character.escape_default());
        } else {
            output.push(character);
        }
    }
}

fn push_sanitized_source_name(output: &mut String, source: &SourceFile) {
    if source.name_is_rendered() {
        output.push_str(source.name());
    } else {
        push_sanitized_inline(output, source.name());
    }
}

fn sanitized_width(text: &str) -> usize {
    text.chars()
        .map(|character| {
            if character == '\\' || (!character.is_ascii_graphic() && character != ' ') {
                character.escape_default().count()
            } else {
                1
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{RenderedSourceName, TextOffset};

    #[test]
    fn decimal_gutter_width_covers_the_full_line_domain() {
        assert_eq!(decimal_width(0), 1);
        assert_eq!(decimal_width(1), 1);
        assert_eq!(decimal_width(9), 1);
        assert_eq!(decimal_width(10), 2);
        assert_eq!(decimal_width(99), 2);
        assert_eq!(decimal_width(100), 3);
        assert_eq!(decimal_width(u32::MAX), 10);
    }

    #[test]
    fn diagnostic_code_inventory_is_exact_ordered_and_unique() {
        let actual = DiagnosticCode::ALL
            .iter()
            .map(|code| code.as_str())
            .collect::<Vec<_>>();
        let expected = [
            "ORC0001", "ORC0002", "ORC0003", "ORC0004", "ORC0005", "ORC0006", "ORC0007", "ORC0008",
            "ORC0101", "ORC0102", "ORC0103", "ORC0104", "ORC0105", "ORC0106", "ORC0107", "ORC0201",
            "ORC0202", "ORC0203", "ORC0204", "ORC0205", "ORC0206", "ORC0207", "ORC0208", "ORC0209",
            "ORC0210", "ORC0301",
        ];

        assert_eq!(actual, expected);
        assert!(actual.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(actual.iter().all(|code| {
            code.len() == 7
                && code.starts_with("ORC")
                && code[3..].bytes().all(|byte| byte.is_ascii_digit())
        }));
        assert!(
            DiagnosticCode::ALL
                .iter()
                .all(|code| code.to_string() == code.as_str())
        );
    }

    #[test]
    fn severity_inventory_and_display_are_exact() {
        assert_eq!(Severity::ALL, &[Severity::Error]);
        assert_eq!(Severity::Error.as_str(), "error");
        assert_eq!(Severity::Error.to_string(), "error");
    }

    #[test]
    fn renders_tabs_and_unicode_columns_deterministically() {
        let mut sources = SourceMap::new();
        let id = sources.add("sample.or", "\tlet é@\n").unwrap();
        let source = sources.get(id).unwrap();
        let span = source.span(TextOffset::new(7), TextOffset::new(8)).unwrap();
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "unexpected character '@'",
            span,
        )
        .with_label("character is not part of Orange 2026")
        .with_note("identifiers are ASCII in this pre-alpha edition");

        assert_eq!(
            render_diagnostics(&sources, &[diagnostic]),
            format!(
                concat!(
                    "error[ORC0001]: unexpected character '@'\n",
                    " --> sample.or:1:7\n",
                    "  |\n",
                    "1 | \\tlet \\u{{e9}}@\n",
                    "  | {caret}^ character is not part of Orange 2026\n",
                    "  = note: identifiers are ASCII in this pre-alpha edition\n",
                ),
                caret = " ".repeat(12),
            )
        );
    }

    #[test]
    fn inline_encoding_distinguishes_tabs_spaces_backslashes_and_unicode() {
        let source = "\t\\    é";
        let mut sanitized = String::new();
        push_sanitized_inline(&mut sanitized, source);

        assert_eq!(sanitized, r"\t\\    \u{e9}");
        assert_eq!(sanitized_width(source), sanitized.chars().count());
    }

    #[test]
    fn inline_encoding_preserves_text_that_needs_no_escaping() {
        let source = "ordinary ASCII text";
        let mut sanitized = String::new();
        push_sanitized_inline(&mut sanitized, source);

        assert_eq!(sanitized, source);
    }

    #[test]
    fn every_diagnostic_text_field_is_ascii_safe() {
        let mut hostile = (u8::MIN..=u8::MAX).map(char::from).collect::<String>();
        hostile.push('\u{202e}');
        hostile.push('🟠');
        let mut sources = SourceMap::new();
        let id = sources.add("fields.or", "x").unwrap();
        let source = sources.get(id).unwrap();
        let span = source.span(TextOffset::new(0), TextOffset::new(1)).unwrap();
        let diagnostic =
            Diagnostic::error(DiagnosticCode::UnexpectedCharacter, hostile.clone(), span)
                .with_label(hostile.clone())
                .with_secondary_span(span, hostile.clone())
                .with_note(hostile);

        let rendered = render_diagnostics(&sources, &[diagnostic]);

        assert!(rendered.is_ascii());
        assert!(
            !rendered
                .bytes()
                .any(|byte| byte.is_ascii_control() && byte != b'\n')
        );
        for escaped in [
            r"\u{0}",
            r"\n",
            r"\\",
            r"\u{7f}",
            r"\u{80}",
            r"\u{202e}",
            r"\u{1f7e0}",
        ] {
            assert!(rendered.contains(escaped), "missing {escaped:?}");
        }
    }

    #[test]
    fn escapes_non_ascii_source_text_with_matching_caret_width() {
        let mut sources = SourceMap::new();
        let text = "a\u{200b}🟠@\n";
        let id = sources.add("sample.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let at = u32::try_from(text.find('@').unwrap()).unwrap();
        let span = source
            .span(TextOffset::new(at), TextOffset::new(at + 1))
            .unwrap();
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "unexpected character '@'",
            span,
        );

        let rendered = render_diagnostics(&sources, &[diagnostic]);
        assert!(rendered.is_ascii());
        assert!(rendered.contains("1 | a\\u{200b}\\u{1f7e0}@\n"));
        assert!(rendered.contains(&format!("  | {}^\n", " ".repeat(18))));
    }

    #[test]
    fn renders_a_zero_width_span_on_the_empty_final_line() {
        let mut sources = SourceMap::new();
        let id = sources.add("eof.or", "first\r\n").unwrap();
        let source = sources.get(id).unwrap();
        let eof = source.byte_len();
        let span = source.span(eof, eof).unwrap();
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "unexpected end of input",
            span,
        )
        .with_label("expected a declaration");

        assert_eq!(
            render_diagnostics(&sources, &[diagnostic]),
            concat!(
                "error[ORC0001]: unexpected end of input\n",
                " --> eof.or:2:1\n",
                "  |\n",
                "2 | \n",
                "  | ^ expected a declaration\n",
            )
        );
    }

    #[test]
    fn truncated_escaped_excerpts_keep_the_caret_aligned() {
        let text = format!("{}é{}@\n", "a".repeat(5), "b".repeat(39));
        let mut sources = SourceMap::new();
        let id = sources.add("window.or", text.clone()).unwrap();
        let source = sources.get(id).unwrap();
        let at = u32::try_from(text.find('@').unwrap()).unwrap();
        let span = source
            .span(TextOffset::new(at), TextOffset::new(at + 1))
            .unwrap();
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "unexpected character '@'",
            span,
        );
        let rendered = render_diagnostics(&sources, &[diagnostic]);

        assert!(rendered.contains(&format!("1 | ... \\u{{e9}}{}@\n", "b".repeat(39))));
        assert!(rendered.contains(&format!("  | {}^\n", " ".repeat(49))));
    }

    #[test]
    fn inconsistent_excerpt_line_views_fail_closed() {
        let mut sources = SourceMap::new();
        let id = sources.add("mismatch.or", "ab").unwrap();
        let source = sources.get(id).unwrap();
        let span = source.span(TextOffset::new(1), TextOffset::new(2)).unwrap();

        assert_eq!(render_excerpt(source, span, 1, "é"), (String::new(), 0, 1));
        assert_eq!(render_excerpt(source, span, 2, "ab"), (String::new(), 0, 1));
    }

    #[test]
    fn short_excerpt_geometry_matches_every_valid_span_boundary() {
        fn encoded(text: &str) -> String {
            let mut output = String::new();
            for character in text.chars() {
                if character == '\\' || (!character.is_ascii_graphic() && character != ' ') {
                    output.extend(character.escape_default());
                } else {
                    output.push(character);
                }
            }
            output
        }

        let text = "aé\\\t🟠\r\nb\nc\rd";
        let mut sources = SourceMap::new();
        let id = sources.add("geometry.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let boundaries = (0..=text.len())
            .filter(|&offset| text.is_char_boundary(offset))
            .collect::<Vec<_>>();

        for &start in &boundaries {
            for &end in boundaries.iter().filter(|&&end| end >= start) {
                let span = source
                    .span(
                        TextOffset::new(u32::try_from(start).unwrap()),
                        TextOffset::new(u32::try_from(end).unwrap()),
                    )
                    .unwrap();
                let location = source.line_column(span.start()).unwrap();
                let line = source.line_text(location.line).unwrap();
                let line_start =
                    usize::try_from(source.line_start(location.line).unwrap().bytes()).unwrap();
                let relative_start = start.saturating_sub(line_start).min(line.len());
                let relative_end = end.saturating_sub(line_start).min(line.len());
                let expected_excerpt = encoded(line);
                let expected_offset = encoded(&line[..relative_start]).len();
                let expected_width = if start == end || relative_end < relative_start {
                    1
                } else {
                    encoded(&line[relative_start..relative_end]).len().max(1)
                };

                assert_eq!(
                    render_excerpt(source, span, location.line, line),
                    (expected_excerpt, expected_offset, expected_width),
                    "unexpected geometry for {start}..{end}",
                );
            }
        }
    }

    #[test]
    fn source_name_controls_are_escaped_without_terminal_injection() {
        let mut sources = SourceMap::new();
        let control_id = sources.add("line\nbreak.or", "@").unwrap();
        let control = sources.get(control_id).unwrap();
        let control_diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "control name",
            control
                .span(TextOffset::new(0), TextOffset::new(1))
                .unwrap(),
        );

        let control_rendered = render_diagnostics(&sources, &[control_diagnostic]);
        assert!(control_rendered.contains(" --> line\\nbreak.or:1:1\n"));
        assert!(!control_rendered.contains(" --> line\nbreak.or"));
    }

    #[test]
    fn raw_source_name_encoding_is_injective() {
        let mut sources = SourceMap::new();
        let ids = ["é.or", r"\u{e9}.or", "line\nbreak.or", r"line\nbreak.or"]
            .map(|name| sources.add(name, "@").unwrap());
        let rendered = ids.map(|id| {
            let source = sources.get(id).unwrap();
            let diagnostic = Diagnostic::error(
                DiagnosticCode::UnexpectedCharacter,
                "unexpected character",
                source.span(TextOffset::new(0), TextOffset::new(1)).unwrap(),
            );
            render_diagnostics(&sources, &[diagnostic])
        });

        assert!(rendered[0].contains(r" --> \u{e9}.or:1:1"));
        assert!(rendered[1].contains(r" --> \\u{e9}.or:1:1"));
        assert!(rendered[2].contains(r" --> line\nbreak.or:1:1"));
        assert!(rendered[3].contains(r" --> line\\nbreak.or:1:1"));
        assert_ne!(rendered[0], rendered[1]);
        assert_ne!(rendered[2], rendered[3]);
    }

    #[test]
    fn canonical_source_names_are_preserved_exactly_once() {
        let mut sources = SourceMap::new();
        let ids = ["é.or", r"\u{e9}.or", "line\nbreak.or", r"line\nbreak.or"].map(|name| {
            sources
                .add_with_rendered_name(RenderedSourceName::from_text(name), "@")
                .unwrap()
        });
        let rendered = ids.map(|id| {
            let source = sources.get(id).unwrap();
            let mut encoded = String::new();
            push_sanitized_source_name(&mut encoded, source);
            assert_eq!(encoded, source.name());
            let diagnostic = Diagnostic::error(
                DiagnosticCode::UnexpectedCharacter,
                "unexpected character",
                source.span(TextOffset::new(0), TextOffset::new(1)).unwrap(),
            );
            render_diagnostics(&sources, &[diagnostic])
        });

        assert!(rendered[0].contains(r" --> \u{e9}.or:1:1"));
        assert!(rendered[1].contains(r" --> \\u{e9}.or:1:1"));
        assert!(rendered[2].contains(r" --> line\nbreak.or:1:1"));
        assert!(rendered[3].contains(r" --> line\\nbreak.or:1:1"));
        assert_ne!(rendered[0], rendered[1]);
        assert_ne!(rendered[2], rendered[3]);
    }

    #[test]
    fn sorts_diagnostics_by_source_position_then_code() {
        let mut sources = SourceMap::new();
        let id = sources.add("sample.or", "@#").unwrap();
        let source = sources.get(id).unwrap();
        let first_span = source.span(TextOffset::new(0), TextOffset::new(1)).unwrap();
        let second_span = source.span(TextOffset::new(1), TextOffset::new(2)).unwrap();
        let later = Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "later", second_span);
        let earlier = Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "earlier", first_span);

        let output = render_diagnostics(&sources, &[later, earlier]);
        assert!(output.find("earlier").unwrap() < output.find("later").unwrap());
    }

    #[test]
    fn canonical_rendering_is_invariant_under_diagnostic_and_secondary_permutations() {
        fn for_each_permutation(length: usize, mut check: impl FnMut(&[usize])) {
            fn visit(position: usize, indices: &mut [usize], check: &mut dyn FnMut(&[usize])) {
                if position == indices.len() {
                    check(indices);
                    return;
                }
                for candidate in position..indices.len() {
                    indices.swap(position, candidate);
                    visit(position + 1, indices, check);
                    indices.swap(position, candidate);
                }
            }

            let mut indices = (0..length).collect::<Vec<_>>();
            visit(0, &mut indices, &mut check);
        }

        let mut sources = SourceMap::new();
        let id = sources.add("sample.or", "@#x").unwrap();
        let source = sources.get(id).unwrap();
        let first = source.span(TextOffset::new(0), TextOffset::new(1)).unwrap();
        let second = source.span(TextOffset::new(1), TextOffset::new(2)).unwrap();
        let third = source.span(TextOffset::new(2), TextOffset::new(3)).unwrap();
        let diagnostics = [
            Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "zeta", first),
            Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "alpha", first)
                .with_label("zeta label"),
            Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "alpha", first)
                .with_label("alpha label")
                .with_note("zeta note"),
            Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "alpha", first)
                .with_label("alpha label")
                .with_note("alpha note")
                .with_secondary_span(second, "related"),
            Diagnostic::error(DiagnosticCode::MalformedInteger, "alpha", first),
            Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "alpha", second),
        ];
        let expected = render_diagnostics(&sources, &diagnostics);

        for_each_permutation(diagnostics.len(), |order| {
            let permuted = order
                .iter()
                .map(|&index| diagnostics[index].clone())
                .collect::<Vec<_>>();
            assert_eq!(render_diagnostics(&sources, &permuted), expected);
        });

        let secondary_spans = [
            (first, "zeta related"),
            (second, "alpha related"),
            (third, "middle related"),
        ];
        let mut secondary_expected = None;
        for_each_permutation(secondary_spans.len(), |order| {
            let mut diagnostic =
                Diagnostic::error(DiagnosticCode::DuplicateFunction, "duplicate", third);
            for &index in order {
                let (span, label) = secondary_spans[index];
                diagnostic = diagnostic.with_secondary_span(span, label);
            }
            let rendered = render_diagnostics(&sources, &[diagnostic]);
            if let Some(expected) = &secondary_expected {
                assert_eq!(&rendered, expected);
            } else {
                secondary_expected = Some(rendered);
            }
        });
    }

    #[test]
    fn foreign_source_creation_order_does_not_affect_rendering() {
        fn foreign_span(name: &str) -> Span {
            let mut sources = SourceMap::new();
            let id = sources.add(name, "@").unwrap();
            sources
                .get(id)
                .unwrap()
                .span(TextOffset::new(0), TextOffset::new(1))
                .unwrap()
        }

        fn render(alpha_first: bool) -> String {
            let rendering_sources = SourceMap::new();
            let (alpha, beta) = if alpha_first {
                (foreign_span("alpha.or"), foreign_span("beta.or"))
            } else {
                let beta = foreign_span("beta.or");
                let alpha = foreign_span("alpha.or");
                (alpha, beta)
            };
            let alpha_diagnostic = Diagnostic::error(
                DiagnosticCode::UnexpectedCharacter,
                "alpha diagnostic",
                alpha,
            )
            .with_label("alpha label")
            .with_secondary_span(beta, "beta related")
            .with_secondary_span(alpha, "alpha related");
            let beta_diagnostic =
                Diagnostic::error(DiagnosticCode::UnexpectedCharacter, "beta diagnostic", beta)
                    .with_label("beta label")
                    .with_secondary_span(alpha, "alpha related")
                    .with_secondary_span(beta, "beta related");

            render_diagnostics(&rendering_sources, &[beta_diagnostic, alpha_diagnostic])
        }

        let alpha_first = render(true);
        let beta_first = render(false);

        assert_eq!(alpha_first, beta_first);
        assert!(
            alpha_first.find("alpha diagnostic").unwrap()
                < alpha_first.find("beta diagnostic").unwrap()
        );
        let first_diagnostic = alpha_first.split_once("\n\n").unwrap().0;
        assert!(
            first_diagnostic.find("alpha related").unwrap()
                < first_diagnostic.find("beta related").unwrap()
        );
    }

    #[test]
    fn foreign_primary_spans_preserve_labels_and_notes_in_fallback_rendering() {
        let mut foreign_sources = SourceMap::new();
        let foreign_id = foreign_sources.add("foreign.or", "@").unwrap();
        let foreign = foreign_sources.get(foreign_id).unwrap();
        let span = foreign
            .span(TextOffset::new(0), TextOffset::new(1))
            .unwrap();
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "foreign diagnostic",
            span,
        )
        .with_label("primary\nlabel")
        .with_note("fallback note");

        assert_eq!(
            render_diagnostics(&SourceMap::new(), &[diagnostic]),
            concat!(
                "error[ORC0001]: foreign diagnostic\n",
                " --> <unknown>:0..1 primary\\nlabel\n",
                "  = note: fallback note\n",
            )
        );
    }

    #[test]
    fn foreign_secondary_fallback_preserves_labels_and_canonical_order() {
        let mut sources = SourceMap::new();
        let local_id = sources.add("local.or", "@#").unwrap();
        let local = sources.get(local_id).unwrap();
        let primary = local.span(TextOffset::new(0), TextOffset::new(1)).unwrap();
        let related = local.span(TextOffset::new(1), TextOffset::new(2)).unwrap();

        let mut foreign_sources = SourceMap::new();
        let foreign_id = foreign_sources.add("foreign.or", "x").unwrap();
        let foreign = foreign_sources
            .get(foreign_id)
            .unwrap()
            .span(TextOffset::new(0), TextOffset::new(1))
            .unwrap();
        let diagnostic = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "mixed secondary spans",
            primary,
        )
        .with_label("primary")
        .with_secondary_span(foreign, "foreign\nlabel")
        .with_secondary_span(related, "local label")
        .with_note("final note");

        assert_eq!(
            render_diagnostics(&sources, &[diagnostic]),
            concat!(
                "error[ORC0001]: mixed secondary spans\n",
                " --> local.or:1:1\n",
                "  |\n",
                "1 | @#\n",
                "  | ^ primary\n",
                " ::: local.or:1:2\n",
                "  |\n",
                "1 | @#\n",
                "  |  - local label\n",
                " ::: <unknown>:0..1 foreign\\nlabel\n",
                "  = note: final note\n",
            )
        );
    }

    #[test]
    fn renders_structured_secondary_spans_deterministically() {
        let mut sources = SourceMap::new();
        let id = sources
            .add("sample.or", "spec first\nspec second\n")
            .unwrap();
        let source = sources.get(id).unwrap();
        let first = source
            .span(TextOffset::new(5), TextOffset::new(10))
            .unwrap();
        let second = source
            .span(TextOffset::new(16), TextOffset::new(22))
            .unwrap();
        let diagnostic = Diagnostic::error(
            DiagnosticCode::DuplicateFunction,
            "duplicate spec function `same`",
            second,
        )
        .with_label("duplicate declaration")
        .with_secondary_span(first, "first declaration is here");

        assert_eq!(
            render_diagnostics(&sources, &[diagnostic]),
            concat!(
                "error[ORC0201]: duplicate spec function `same`\n",
                " --> sample.or:2:6\n",
                "  |\n",
                "2 | spec second\n",
                "  |      ^^^^^^ duplicate declaration\n",
                " ::: sample.or:1:6\n",
                "  |\n",
                "1 | spec first\n",
                "  |      ----- first declaration is here\n",
            )
        );
    }

    #[test]
    fn bounded_excerpts_prevent_diagnostic_count_times_line_length_amplification() {
        const PREFIX_BYTES: usize = 1024 * 1024;

        let mut text = "a".repeat(PREFIX_BYTES);
        text.push_str("@#");
        let mut sources = SourceMap::new();
        let id = sources.add("long.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let at = u32::try_from(PREFIX_BYTES).unwrap();
        let hash = at + 1;
        let at_span = source
            .span(TextOffset::new(at), TextOffset::new(hash))
            .unwrap();
        let hash_span = source
            .span(TextOffset::new(hash), TextOffset::new(hash + 1))
            .unwrap();
        let diagnostics: Vec<_> = (0..100)
            .map(|index| {
                if index % 2 == 0 {
                    Diagnostic::error(
                        DiagnosticCode::UnexpectedCharacter,
                        "unexpected character '#'",
                        hash_span,
                    )
                } else {
                    Diagnostic::error(
                        DiagnosticCode::UnexpectedCharacter,
                        "unexpected character '@'",
                        at_span,
                    )
                }
            })
            .collect();
        let first_render = render_diagnostics(&sources, &diagnostics);
        let second_render = render_diagnostics(&sources, &diagnostics);

        assert_eq!(first_render, second_render);
        assert_eq!(first_render.matches("error[ORC0001]").count(), 100);
        assert!(first_render.contains(&format!(" --> long.or:1:{}", at + 1)));
        assert!(first_render.contains(&format!(" --> long.or:1:{}", hash + 1)));
        assert!(first_render.len() < 64 * 1024);
    }
}
