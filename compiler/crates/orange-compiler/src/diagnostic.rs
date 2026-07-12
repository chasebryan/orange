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
            Self::ExpectedSyntax => "ORC0101",
            Self::UnsupportedSourceEdition => "ORC0102",
            Self::ExpectedFunctionDeclaration => "ORC0103",
            Self::TrailingSyntax => "ORC0104",
            Self::TooManySyntaxErrors => "ORC0105",
            Self::ParserResourceLimit => "ORC0106",
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

/// A compiler diagnostic with one primary source span.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    severity: Severity,
    code: DiagnosticCode,
    message: String,
    primary_span: Span,
    label: String,
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
            notes: Vec::new(),
        }
    }

    /// Sets the concise label shown beside the primary underline.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
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
}

/// Renders diagnostics in a canonical total order over their rendered fields.
///
/// The returned text is empty for no diagnostics and otherwise ends in one
/// newline. Rendering uses no terminal color or locale-sensitive formatting.
#[must_use]
pub fn render_diagnostics(sources: &SourceMap, diagnostics: &[Diagnostic]) -> String {
    let mut ordered: Vec<_> = diagnostics.iter().collect();
    ordered.sort_by(|left, right| diagnostic_key(left).cmp(&diagnostic_key(right)));

    let mut rendered = String::new();
    for (index, diagnostic) in ordered.into_iter().enumerate() {
        if index != 0 {
            rendered.push('\n');
        }
        render_one(&mut rendered, sources, diagnostic);
    }
    rendered
}

fn diagnostic_key(
    diagnostic: &Diagnostic,
) -> (
    crate::source::SourceId,
    u32,
    u32,
    Severity,
    DiagnosticCode,
    &str,
    &str,
    &[String],
) {
    (
        diagnostic.primary_span.source(),
        diagnostic.primary_span.start().bytes(),
        diagnostic.primary_span.end().bytes(),
        diagnostic.severity,
        diagnostic.code,
        &diagnostic.message,
        &diagnostic.label,
        &diagnostic.notes,
    )
}

fn render_one(output: &mut String, sources: &SourceMap, diagnostic: &Diagnostic) {
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
        render_notes(output, diagnostic);
        return;
    };
    let Some(location) = source.line_column(diagnostic.primary_span.start()) else {
        let _ = writeln!(
            output,
            " --> {}:{}..{}",
            sanitize_inline(source.name()),
            diagnostic.primary_span.start().bytes(),
            diagnostic.primary_span.end().bytes()
        );
        render_notes(output, diagnostic);
        return;
    };

    let _ = writeln!(
        output,
        " --> {}:{}:{}",
        sanitize_inline(source.name()),
        location.line,
        location.column
    );
    let Some(line) = source.line_text(location.line) else {
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
    render_notes(output, diagnostic);
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
        if character == '\t' {
            sanitized.push_str("    ");
        } else if character.is_control() || is_bidi_control(character) {
            sanitized.extend(character.escape_default());
        } else {
            sanitized.push(character);
        }
    }
    sanitized
}

fn sanitized_width(text: &str) -> usize {
    text.chars()
        .map(|character| {
            if character == '\t' {
                4
            } else if character.is_control() || is_bidi_control(character) {
                character.escape_default().count()
            } else {
                1
            }
        })
        .sum()
}

const fn is_bidi_control(character: char) -> bool {
    matches!(
        character,
        '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::TextOffset;

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
            concat!(
                "error[ORC0001]: unexpected character '@'\n",
                " --> sample.or:1:7\n",
                "  |\n",
                "1 |     let é@\n",
                "  |          ^ character is not part of Orange 2026\n",
                "  = note: identifiers are ASCII in this pre-alpha edition\n",
            )
        );
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
