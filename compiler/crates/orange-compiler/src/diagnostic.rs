//! Stable, source-aware compiler diagnostics.

use std::fmt::Write as _;

use crate::source::{SourceFile, SourceMap, Span};

/// A stable diagnostic identifier.
///
/// Codes are part of the user-facing compiler interface. Existing meanings
/// must not be silently reused when new compiler phases are added.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiagnosticCode {
    /// A character has no lexical meaning in the selected edition.
    UnexpectedCharacter,
    /// A nested block comment reached the end of the file.
    UnterminatedBlockComment,
    /// A quoted string reached a line ending or the end of the file.
    UnterminatedString,
    /// A string contains an unsupported or incomplete escape.
    InvalidEscape,
    /// An integer has invalid digits or separators for its base.
    MalformedInteger,
    /// A source exceeds the deterministic non-trivia token budget.
    LexicalTokenLimit,
    /// Further lexical errors were suppressed after the reporting budget.
    TooManyLexicalErrors,
    /// The lexer could not retain its bounded token-stream representation.
    LexicalResourceLimit,
    /// A token required by the active grammar production was not present.
    ExpectedSyntax,
    /// The source edition declaration is not exactly `edition 2026;`.
    UnsupportedSourceEdition,
    /// A module member is not a `spec` or `impl` function declaration.
    ExpectedFunctionDeclaration,
    /// Syntax follows the single module allowed in one source file.
    TrailingSyntax,
    /// Further parser errors were suppressed after the reporting budget.
    TooManySyntaxErrors,
    /// A deterministic parser resource budget was exhausted.
    ParserResourceLimit,
    /// Parsing received lexer output owned by another source.
    InvalidParserInput,
    /// A second function declaration conflicts in its declaration namespace.
    DuplicateFunction,
    /// A typed literal body appeared on a function kind without semantics.
    UnsupportedTypedFunction,
    /// A typed literal names a type outside the admitted semantic fragment.
    UnsupportedType,
    /// A `Word` type uses a width other than the admitted exact width.
    UnsupportedWordWidth,
    /// An exact integer magnitude exceeds the semantic representation budget.
    IntegerMagnitudeLimit,
    /// A fixed-width word literal is negative.
    NegativeWordLiteral,
    /// A fixed-width word literal is outside its admitted unsigned range.
    WordLiteralOutOfRange,
    /// Further semantic errors were suppressed after the reporting budget.
    TooManySemanticErrors,
    /// A deterministic semantic-analysis resource budget was exhausted.
    SemanticResourceLimit,
    /// Semantic analysis received a syntax tree owned by another source.
    InvalidSemanticInput,
    /// A deterministic reference-evaluation resource budget was exhausted.
    EvaluationResourceLimit,
}

impl DiagnosticCode {
    /// Returns the permanent printable code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnexpectedCharacter => "ORC0001",
            Self::UnterminatedBlockComment => "ORC0002",
            Self::UnterminatedString => "ORC0003",
            Self::InvalidEscape => "ORC0004",
            Self::MalformedInteger => "ORC0005",
            Self::LexicalTokenLimit => "ORC0006",
            Self::TooManyLexicalErrors => "ORC0007",
            Self::LexicalResourceLimit => "ORC0008",
            Self::ExpectedSyntax => "ORC0101",
            Self::UnsupportedSourceEdition => "ORC0102",
            Self::ExpectedFunctionDeclaration => "ORC0103",
            Self::TrailingSyntax => "ORC0104",
            Self::TooManySyntaxErrors => "ORC0105",
            Self::ParserResourceLimit => "ORC0106",
            Self::InvalidParserInput => "ORC0107",
            Self::DuplicateFunction => "ORC0201",
            Self::UnsupportedTypedFunction => "ORC0202",
            Self::UnsupportedType => "ORC0203",
            Self::UnsupportedWordWidth => "ORC0204",
            Self::IntegerMagnitudeLimit => "ORC0205",
            Self::NegativeWordLiteral => "ORC0206",
            Self::WordLiteralOutOfRange => "ORC0207",
            Self::TooManySemanticErrors => "ORC0208",
            Self::SemanticResourceLimit => "ORC0209",
            Self::InvalidSemanticInput => "ORC0210",
            Self::EvaluationResourceLimit => "ORC0301",
        }
    }
}

/// The impact of a compiler diagnostic.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Severity {
    /// Compilation cannot proceed successfully.
    Error,
}

impl Severity {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
        }
    }
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
/// Rendering uses no terminal color or locale-sensitive formatting.
#[must_use]
pub fn render_diagnostics(sources: &SourceMap, diagnostics: &[Diagnostic]) -> String {
    let mut ordered: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| OrderedDiagnostic::new(sources, diagnostic))
        .collect();
    ordered.sort_by(|left, right| compare_diagnostics(sources, left, right));

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
        secondary_spans.sort_by(|left, right| compare_secondary_span(sources, left, right));
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
    let _ = writeln!(
        output,
        "{}[{}]: {}",
        diagnostic.severity.as_str(),
        diagnostic.code.as_str(),
        sanitize_inline(&diagnostic.message)
    );

    let Some(source) = sources.get(diagnostic.primary_span.source()) else {
        let _ = writeln!(
            output,
            " --> <unknown>:{}..{}",
            diagnostic.primary_span.start().bytes(),
            diagnostic.primary_span.end().bytes()
        );
        render_secondary_spans(output, sources, secondary_spans);
        render_notes(output, diagnostic);
        return;
    };
    let Some(location) = source.line_column(diagnostic.primary_span.start()) else {
        let _ = writeln!(
            output,
            " --> {}:{}..{}",
            sanitize_source_name(source),
            diagnostic.primary_span.start().bytes(),
            diagnostic.primary_span.end().bytes()
        );
        render_secondary_spans(output, sources, secondary_spans);
        render_notes(output, diagnostic);
        return;
    };

    let _ = writeln!(
        output,
        " --> {}:{}:{}",
        sanitize_source_name(source),
        location.line,
        location.column
    );
    let Some(line) = source.line_text(location.line) else {
        render_secondary_spans(output, sources, secondary_spans);
        render_notes(output, diagnostic);
        return;
    };

    let gutter_width = location.line.to_string().len();
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
        "{space:>width$} | {indent}{carets}",
        space = "",
        width = gutter_width,
        indent = " ".repeat(caret_offset),
        carets = "^".repeat(caret_width)
    );
    if !diagnostic.label.is_empty() {
        let _ = write!(output, " {}", sanitize_inline(&diagnostic.label));
    }
    output.push('\n');
    render_secondary_spans(output, sources, secondary_spans);
    render_notes(output, diagnostic);
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
        let _ = writeln!(
            output,
            " ::: <unknown>:{}..{} {}",
            secondary.span.start().bytes(),
            secondary.span.end().bytes(),
            sanitize_inline(&secondary.label)
        );
        return;
    };
    let Some(location) = source.line_column(secondary.span.start()) else {
        let _ = writeln!(
            output,
            " ::: {}:{}..{} {}",
            sanitize_source_name(source),
            secondary.span.start().bytes(),
            secondary.span.end().bytes(),
            sanitize_inline(&secondary.label)
        );
        return;
    };
    let _ = writeln!(
        output,
        " ::: {}:{}:{}",
        sanitize_source_name(source),
        location.line,
        location.column
    );
    let Some(line) = source.line_text(location.line) else {
        return;
    };
    let gutter_width = location.line.to_string().len();
    let (excerpt, caret_offset, caret_width) =
        render_excerpt(source, secondary.span, location.line, line);
    let _ = writeln!(
        output,
        "{space:>width$} |",
        space = "",
        width = gutter_width
    );
    let _ = writeln!(output, "{} | {excerpt}", location.line);
    let _ = writeln!(
        output,
        "{space:>width$} | {indent}{marks} {}",
        sanitize_inline(&secondary.label),
        space = "",
        width = gutter_width,
        indent = " ".repeat(caret_offset),
        marks = "-".repeat(caret_width),
    );
}

fn render_notes(output: &mut String, diagnostic: &Diagnostic) {
    for note in &diagnostic.notes {
        let _ = writeln!(output, "  = note: {}", sanitize_inline(note));
    }
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

    let mut window_start = relative_start;
    for (index, _) in line[..relative_start]
        .char_indices()
        .rev()
        .take(CONTEXT_BEFORE)
    {
        window_start = index;
    }
    let mut window_end = relative_start;
    for (index, character) in line[relative_start..].char_indices().take(CONTEXT_AFTER) {
        window_end = relative_start + index + character.len_utf8();
    }

    let left_truncated = window_start != 0;
    let right_truncated = window_end != line.len();
    let mut excerpt = String::new();
    if left_truncated {
        excerpt.push_str("... ");
    }
    excerpt.push_str(&sanitize_inline(&line[window_start..window_end]));
    if right_truncated {
        excerpt.push_str(" ...");
    }

    let caret_offset =
        usize::from(left_truncated) * 4 + sanitized_width(&line[window_start..relative_start]);
    let relative_end = usize::try_from(span.end().bytes().saturating_sub(line_start.bytes()))
        .unwrap_or(line.len())
        .min(window_end)
        .min(line.len());
    let caret_width =
        if span.is_empty() || relative_end < relative_start || !line.is_char_boundary(relative_end)
        {
            1
        } else {
            sanitized_width(&line[relative_start..relative_end]).max(1)
        };
    (excerpt, caret_offset, caret_width)
}

fn sanitize_inline(text: &str) -> String {
    let mut sanitized = String::new();
    for character in text.chars() {
        if character == '\\' || (!character.is_ascii_graphic() && character != ' ') {
            sanitized.extend(character.escape_default());
        } else {
            sanitized.push(character);
        }
    }
    sanitized
}

fn sanitize_source_name(source: &SourceFile) -> String {
    if source.name_is_rendered() {
        source.name().to_owned()
    } else {
        sanitize_inline(source.name())
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
        let sanitized = sanitize_inline(source);

        assert_eq!(sanitized, r"\t\\    \u{e9}");
        assert_eq!(sanitized_width(source), sanitized.chars().count());
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
    fn renders_repeated_diagnostics_near_a_long_line_deterministically() {
        const PREFIX_BYTES: usize = 1024 * 1024;

        let mut text = "a".repeat(PREFIX_BYTES);
        text.push_str("@#");
        let mut sources = SourceMap::new();
        let id = sources.add("long.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let at = u32::try_from(PREFIX_BYTES).unwrap();
        let hash = at + 1;
        let first = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "unexpected character '@'",
            source
                .span(TextOffset::new(at), TextOffset::new(hash))
                .unwrap(),
        );
        let second = Diagnostic::error(
            DiagnosticCode::UnexpectedCharacter,
            "unexpected character '#'",
            source
                .span(TextOffset::new(hash), TextOffset::new(hash + 1))
                .unwrap(),
        );

        let diagnostics = [second, first];
        let first_render = render_diagnostics(&sources, &diagnostics);
        let second_render = render_diagnostics(&sources, &diagnostics);

        assert_eq!(first_render, second_render);
        assert!(first_render.contains(&format!(" --> long.or:1:{}", at + 1)));
        assert!(first_render.contains(&format!(" --> long.or:1:{}", hash + 1)));
        assert!(first_render.len() < 1024);
    }
}
