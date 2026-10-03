//! Bounded, offline declaration references for parsed Orange source.
//!
//! The generator reads source-owned syntax spans, never resolves imports or
//! types, and renders all source-derived strings as inert HTML text. It keeps
//! declarations in their written order and includes the complete source display.

use std::fmt::{self, Write as _};

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::edition::Edition;
use crate::lexer::{Lexed, Token, TokenKind, lex};
use crate::parser::{
    FunctionBody, FunctionDeclaration, MAX_SYNTAX_NODES_PER_SOURCE, ParseResult, SyntaxTree, parse,
};
use crate::source::{MAX_SOURCE_BYTES, SourceFile, Span, TextOffset};

/// Maximum bytes in one complete standalone declaration-reference document.
pub const MAX_DOCUMENTATION_HTML_BYTES: usize = 16 * 1024 * 1024;

/// Maximum declaration planning and rendering events for one source.
///
/// Each declaration consumes one planning event, one navigation event and one
/// section event. Source escaping is bounded separately by source/output bytes.
pub const MAX_DOCUMENTATION_EVENTS_PER_SOURCE: usize = 4 * 262_144;

const _: () = assert!(MAX_DOCUMENTATION_HTML_BYTES == MAX_SOURCE_BYTES);
const _: () = assert!(MAX_DOCUMENTATION_EVENTS_PER_SOURCE == MAX_SYNTAX_NODES_PER_SOURCE * 4);

/// The complete result of generating a parsed-source declaration reference.
#[derive(Debug, Eq, PartialEq)]
pub struct DocumentationResult {
    html: Option<String>,
    diagnostics: DocumentationDiagnostics,
}

#[derive(Debug, Eq, PartialEq)]
enum DocumentationDiagnostics {
    None,
    Lexical(Lexed),
    Syntax(ParseResult),
    Generation(Vec<Diagnostic>),
}

impl DocumentationResult {
    /// Returns the complete standalone HTML, or `None` after any failure.
    #[must_use]
    pub fn html(&self) -> Option<&str> {
        self.html.as_deref()
    }

    /// Consumes the result and returns the complete standalone HTML.
    #[must_use]
    pub fn into_html(self) -> Option<String> {
        self.html
    }

    /// Returns original frontend errors or a documentation failure diagnostic.
    ///
    /// If diagnostic storage cannot be reserved, this slice is empty while
    /// [`Self::has_errors`] still reports failure. Partial HTML is never returned.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match &self.diagnostics {
            DocumentationDiagnostics::None => &[],
            DocumentationDiagnostics::Lexical(result) => result.diagnostics(),
            DocumentationDiagnostics::Syntax(result) => result.diagnostics(),
            DocumentationDiagnostics::Generation(diagnostics) => diagnostics,
        }
    }

    /// Returns whether generation failed, including unreportable allocation failure.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.html.is_none()
    }
}

/// Generates a deterministic, standalone offline reference for parsed syntax.
///
/// Entries describe the written module, imports, aliases, functions and tests
/// in source order, with unique ordinal anchors and input source locations.
/// Function/test headers omit bodies; a full escaped source display retains
/// every comment. The HTML contains no filename, ambient path, clock, scripts,
/// styles or external assets, and performs no import loading, semantic analysis
/// or evaluation. It establishes no proof or test outcome.
///
/// Source text escapes HTML metacharacters. Controls other than CR/LF/tab and
/// a fixed set of Unicode display-direction/separator controls appear as
/// visible ASCII escapes. The source display is not a byte-recovery format.
#[must_use]
pub fn document_source(source: &SourceFile, edition: Edition) -> DocumentationResult {
    document_with_limits(source, edition, Limits::DEFAULT)
}

#[derive(Clone, Copy)]
struct Limits {
    output: usize,
    events: usize,
}

impl Limits {
    const DEFAULT: Self = Self {
        output: MAX_DOCUMENTATION_HTML_BYTES,
        events: MAX_DOCUMENTATION_EVENTS_PER_SOURCE,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Failure {
    Resource,
    Inconsistent,
}

fn failed(source: &SourceFile, failure: Failure) -> DocumentationResult {
    let mut diagnostics = Vec::new();
    if diagnostics.try_reserve_exact(1).is_ok() {
        let (code, message, note) = match failure {
            Failure::Resource => (
                DiagnosticCode::DocumentationResourceLimit,
                "documentation exceeded its resource budget",
                "documentation uses bounded declaration storage and at most 16777216 HTML bytes",
            ),
            Failure::Inconsistent => (
                DiagnosticCode::DocumentationInconsistency,
                "documentation could not preserve the parsed declarations",
                "no HTML was produced; report this compiler inconsistency",
            ),
        };
        if let Some(span) = source.span(TextOffset::new(0), TextOffset::new(0)) {
            diagnostics.push(Diagnostic::error(code, message, span).with_note(note));
        }
    }
    DocumentationResult {
        html: None,
        diagnostics: DocumentationDiagnostics::Generation(diagnostics),
    }
}

fn document_with_limits(
    source: &SourceFile,
    edition: Edition,
    limits: Limits,
) -> DocumentationResult {
    let lexed = lex(source, edition);
    if lexed.has_errors() {
        return DocumentationResult {
            html: None,
            diagnostics: DocumentationDiagnostics::Lexical(lexed),
        };
    }
    let parsed = parse(source, &lexed);
    let Some(tree) = parsed.ast() else {
        return DocumentationResult {
            html: None,
            diagnostics: DocumentationDiagnostics::Syntax(parsed),
        };
    };
    let mut budget = Budget {
        remaining: limits.events,
    };
    let result = entries(source, lexed.tokens(), tree, &mut budget)
        .and_then(|entries| render(source, &entries, &mut budget, limits.output));
    match result {
        Ok(html) => DocumentationResult {
            html: Some(html),
            diagnostics: DocumentationDiagnostics::None,
        },
        Err(failure) => failed(source, failure),
    }
}

struct Budget {
    remaining: usize,
}

impl Budget {
    fn event(&mut self) -> Result<(), Failure> {
        self.remaining = self.remaining.checked_sub(1).ok_or(Failure::Resource)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Module,
    Import,
    Alias,
    Spec,
    Impl,
    Test,
}

impl Kind {
    const fn keyword(self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Import => "use",
            Self::Alias => "type",
            Self::Spec => "spec",
            Self::Impl => "impl",
            Self::Test => "test",
        }
    }

    const fn description(self, legacy: bool) -> &'static str {
        match self {
            Self::Module => "Module declaration.",
            Self::Import => "Import declaration; the imported module is not loaded.",
            Self::Alias => "Type alias declaration; the written type is not resolved.",
            Self::Spec | Self::Impl if legacy => "Legacy empty body; syntax only.",
            Self::Spec => "Spec declaration; body omitted.",
            Self::Impl => "Impl declaration; body omitted.",
            Self::Test => "Test declaration; body omitted and test not run.",
        }
    }
}

struct Entry<'a> {
    kind: Kind,
    name: &'a str,
    span: Span,
    header: Span,
    legacy: bool,
}

fn header(source: &SourceFile, declaration: Span, end: TextOffset) -> Result<Span, Failure> {
    if declaration.source() != source.id() || end > declaration.end() {
        return Err(Failure::Inconsistent);
    }
    source
        .span(declaration.start(), end)
        .ok_or(Failure::Inconsistent)
}

fn opening_after(tokens: &[Token], end: TextOffset) -> Result<(), Failure> {
    let index = tokens.partition_point(|token| token.span.start() < end);
    if tokens.get(index).map(|token| token.kind) == Some(TokenKind::LeftBrace) {
        Ok(())
    } else {
        Err(Failure::Inconsistent)
    }
}

fn function_entry<'a>(
    source: &SourceFile,
    tokens: &[Token],
    function: &'a FunctionDeclaration,
) -> Result<Entry<'a>, Failure> {
    let (end, legacy) = match &function.body {
        FunctionBody::Typed(body) => {
            opening_after(tokens, body.result_type.span.end())?;
            (body.result_type.span.end(), false)
        }
        FunctionBody::Empty => {
            let after = tokens.partition_point(|token| token.span.start() < function.span.end());
            let close = tokens.get(after.checked_sub(1).ok_or(Failure::Inconsistent)?);
            let open = tokens.get(after.checked_sub(2).ok_or(Failure::Inconsistent)?);
            match (open, close) {
                (Some(open), Some(close))
                    if open.kind == TokenKind::LeftBrace && close.kind == TokenKind::RightBrace =>
                {
                    (open.span.start(), true)
                }
                _ => return Err(Failure::Inconsistent),
            }
        }
    };
    Ok(Entry {
        kind: match function.kind {
            crate::parser::FunctionKind::Spec => Kind::Spec,
            crate::parser::FunctionKind::Impl => Kind::Impl,
        },
        name: function.name.text(),
        span: function.span,
        header: header(source, function.span, end)?,
        legacy,
    })
}

fn add_entry<'a>(
    entries: &mut Vec<Entry<'a>>,
    entry: Entry<'a>,
    budget: &mut Budget,
) -> Result<(), Failure> {
    budget.event()?;
    if entries
        .last()
        .is_some_and(|previous| previous.span.end() > entry.span.start())
    {
        // The module encloses its members; only its header participates here.
        if entries.len() != 1 || entries.last().map(|entry| entry.kind) != Some(Kind::Module) {
            return Err(Failure::Inconsistent);
        }
    }
    entries.try_reserve(1).map_err(|_| Failure::Resource)?;
    entries.push(entry);
    Ok(())
}

fn entries<'a>(
    source: &'a SourceFile,
    tokens: &[Token],
    tree: &'a SyntaxTree,
    budget: &mut Budget,
) -> Result<Vec<Entry<'a>>, Failure> {
    let module = &tree.module;
    opening_after(tokens, module.name.span.end())?;
    let mut entries = Vec::new();
    add_entry(
        &mut entries,
        Entry {
            kind: Kind::Module,
            name: module.name.text(),
            span: module.span,
            header: header(source, module.span, module.name.span.end())?,
            legacy: false,
        },
        budget,
    )?;
    for import in &module.uses {
        add_entry(
            &mut entries,
            Entry {
                kind: Kind::Import,
                name: import.name.text(),
                span: import.span,
                header: import.span,
                legacy: false,
            },
            budget,
        )?;
    }
    for alias in &module.types {
        add_entry(
            &mut entries,
            Entry {
                kind: Kind::Alias,
                name: alias.name.text(),
                span: alias.span,
                header: alias.span,
                legacy: false,
            },
            budget,
        )?;
    }
    let mut functions = module.functions.iter().peekable();
    let mut tests = module.tests.iter().peekable();
    loop {
        match (functions.peek(), tests.peek()) {
            (Some(function), Some(test)) if function.span.start() < test.span.start() => {
                let function = functions.next().ok_or(Failure::Inconsistent)?;
                add_entry(
                    &mut entries,
                    function_entry(source, tokens, function)?,
                    budget,
                )?;
            }
            (_, Some(_)) => {
                let test = tests.next().ok_or(Failure::Inconsistent)?;
                opening_after(tokens, test.title.span.end())?;
                add_entry(
                    &mut entries,
                    Entry {
                        kind: Kind::Test,
                        name: source.slice(test.title.span).ok_or(Failure::Inconsistent)?,
                        span: test.span,
                        header: header(source, test.span, test.title.span.end())?,
                        legacy: false,
                    },
                    budget,
                )?;
            }
            (Some(_), None) => {
                let function = functions.next().ok_or(Failure::Inconsistent)?;
                add_entry(
                    &mut entries,
                    function_entry(source, tokens, function)?,
                    budget,
                )?;
            }
            (None, None) => break,
        }
    }
    Ok(entries)
}

struct Output {
    text: String,
    limit: usize,
}

impl Output {
    fn append(&mut self, text: &str) -> Result<(), Failure> {
        let length = self
            .text
            .len()
            .checked_add(text.len())
            .ok_or(Failure::Resource)?;
        if length > self.limit {
            return Err(Failure::Resource);
        }
        self.text
            .try_reserve(text.len())
            .map_err(|_| Failure::Resource)?;
        self.text.push_str(text);
        Ok(())
    }

    fn escaped(&mut self, text: &str) -> Result<(), Failure> {
        for character in text.chars() {
            match character {
                '&' => self.append("&amp;")?,
                '<' => self.append("&lt;")?,
                '>' => self.append("&gt;")?,
                '"' => self.append("&quot;")?,
                '\'' => self.append("&#39;")?,
                character if visible_escape(character) => {
                    write!(self, "\\u{{{:x}}}", u32::from(character))
                        .map_err(|_| Failure::Resource)?;
                }
                character => {
                    let mut bytes = [0_u8; 4];
                    self.append(character.encode_utf8(&mut bytes))?;
                }
            }
        }
        Ok(())
    }

    fn label(&mut self, entry: &Entry<'_>) -> Result<(), Failure> {
        self.append(entry.kind.keyword())?;
        self.append(" ")?;
        self.escaped(entry.name)
    }
}

impl fmt::Write for Output {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.append(text).map_err(|_| fmt::Error)
    }
}

fn visible_escape(character: char) -> bool {
    (character.is_control() && !matches!(character, '\r' | '\n' | '\t'))
        || matches!(character,
            '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{2028}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

const PROLOGUE: &str = "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'\">\n<title>Orange 2026 declaration reference</title>\n</head>\n<body>\n<main>\n<h1>Orange 2026 declaration reference</h1>\n<p>This reference describes parsed source syntax. Imports and types are not checked, tests are not run, and no proof or assurance is established.</p>\n<nav aria-label=\"Declarations\">\n<ul>\n";

fn render(
    source: &SourceFile,
    entries: &[Entry<'_>],
    budget: &mut Budget,
    limit: usize,
) -> Result<String, Failure> {
    let mut output = Output {
        text: String::new(),
        limit,
    };
    output.append(PROLOGUE)?;
    for (index, entry) in entries.iter().enumerate() {
        budget.event()?;
        let ordinal = index.checked_add(1).ok_or(Failure::Resource)?;
        write!(&mut output, "<li><a href=\"#declaration-{ordinal}\">")
            .map_err(|_| Failure::Resource)?;
        output.label(entry)?;
        output.append("</a></li>\n")?;
    }
    output.append(
        "<li><a href=\"#source\">Complete source</a></li>\n</ul>\n</nav>\n<h2>Declarations</h2>\n",
    )?;
    for (index, entry) in entries.iter().enumerate() {
        budget.event()?;
        let ordinal = index.checked_add(1).ok_or(Failure::Resource)?;
        let location = source
            .line_column(entry.span.start())
            .ok_or(Failure::Inconsistent)?;
        write!(&mut output, "<section id=\"declaration-{ordinal}\">\n<h3>")
            .map_err(|_| Failure::Resource)?;
        output.label(entry)?;
        write!(
            &mut output,
            "</h3>\n<p>Line {}, column {}. ",
            location.line, location.column
        )
        .map_err(|_| Failure::Resource)?;
        output.append(entry.kind.description(entry.legacy))?;
        output.append("</p>\n<pre><code>")?;
        output.escaped(source.slice(entry.header).ok_or(Failure::Inconsistent)?)?;
        output.append("</code></pre>\n</section>\n")?;
    }
    output.append("<h2>Complete source</h2>\n<p>Source display; not a byte-recovery or proof-identity format. Control characters are shown as visible ASCII escapes.</p>\n<pre id=\"source\"><code>")?;
    output.escaped(source.text())?;
    output.append("</code></pre>\n</main>\n</body>\n</html>\n")?;
    Ok(output.text)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::{MAX_EXPRESSION_HEIGHT, SourceMap};

    fn result(name: &str, text: &str) -> DocumentationResult {
        let mut sources = SourceMap::new();
        let id = sources.add(name, text).unwrap();
        document_source(sources.get(id).unwrap(), Edition::E2026)
    }

    fn html(text: &str) -> String {
        let output = result("input.or", text);
        assert!(!output.has_errors(), "{text:?}: {:?}", output.diagnostics());
        assert!(output.diagnostics().is_empty());
        let output = output.into_html().unwrap();
        assert_eq!(
            result("unrelated/filename.or", text).html(),
            Some(output.as_str())
        );
        output
    }

    fn listing(document: &str) -> &str {
        document
            .split_once("<pre id=\"source\"><code>")
            .unwrap()
            .1
            .split_once("</code></pre>")
            .unwrap()
            .0
    }

    #[test]
    fn minimal_document_is_exact_and_deterministic() {
        let output = html("edition 2026;module m{}");
        assert_eq!(
            output,
            "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; base-uri 'none'; form-action 'none'\">\n<title>Orange 2026 declaration reference</title>\n</head>\n<body>\n<main>\n<h1>Orange 2026 declaration reference</h1>\n<p>This reference describes parsed source syntax. Imports and types are not checked, tests are not run, and no proof or assurance is established.</p>\n<nav aria-label=\"Declarations\">\n<ul>\n<li><a href=\"#declaration-1\">module m</a></li>\n<li><a href=\"#source\">Complete source</a></li>\n</ul>\n</nav>\n<h2>Declarations</h2>\n<section id=\"declaration-1\">\n<h3>module m</h3>\n<p>Line 1, column 14. Module declaration.</p>\n<pre><code>module m</code></pre>\n</section>\n<h2>Complete source</h2>\n<p>Source display; not a byte-recovery or proof-identity format. Control characters are shown as visible ASCII escapes.</p>\n<pre id=\"source\"><code>edition 2026;module m{}</code></pre>\n</main>\n</body>\n</html>\n"
        );
    }

    #[test]
    fn declaration_order_headers_domains_and_legacy_comments_are_preserved() {
        let text = "edition 2026;module m{use missing;type Row=Word[8]^4;type Matrix=Row^4;spec choose[T in {Row,Int},n in 2..4](a:T)->T{a}test \"between\"{1==1}impl old(){/* legacy comment */}spec modulus[b in 4..6](x:Mod[(1<<b)-1])->Mod[(1<<b)-1]{x}spec old_spec(){// empty\n}test \"last\"{false}}";
        let output = html(text);
        let labels = [
            "module m",
            "use missing",
            "type Row",
            "type Matrix",
            "spec choose",
            "test &quot;between&quot;",
            "impl old",
            "spec modulus",
            "spec old_spec",
            "test &quot;last&quot;",
        ];
        let mut rest = output.as_str();
        for label in labels {
            rest = rest.split_once(&format!("<h3>{label}</h3>")).unwrap().1;
        }
        assert!(output.contains("<code>spec choose[T in {Row,Int},n in 2..4](a:T)-&gt;T</code>"));
        assert!(output.contains(
            "<code>spec modulus[b in 4..6](x:Mod[(1&lt;&lt;b)-1])-&gt;Mod[(1&lt;&lt;b)-1]</code>"
        ));
        assert!(output.contains("<code>impl old()</code>"));
        assert!(output.contains("<code>spec old_spec()</code>"));
        assert_eq!(output.matches("Legacy empty body; syntax only.").count(), 2);
        assert_eq!(output.matches("/* legacy comment */").count(), 1);
        assert_eq!(output.matches("// empty").count(), 1);
        assert!(output.contains("Test declaration; body omitted and test not run."));
    }

    #[test]
    fn hostile_markup_and_names_remain_inert_text() {
        let text = "/* <script src=\"https://bad.invalid\">&' </code></pre><img onerror='x'> */edition 2026;module verified{spec verified()->Word[8]^4{\"<>&'\"}test \"<script>verified & 'title'\"{true}}";
        let output = html(text);
        assert!(!output.contains("<script"));
        assert!(!output.contains("<img"));
        assert!(!output.contains("<style"));
        assert!(output.contains("&lt;script src=&quot;https://bad.invalid&quot;&gt;&amp;&#39;"));
        assert!(output.contains("&lt;/code&gt;&lt;/pre&gt;&lt;img onerror=&#39;x&#39;&gt;"));
        assert!(output.contains("<h3>module verified</h3>"));
        assert!(!output.contains("class=\"verified\""));
        assert_eq!(
            listing(&output),
            "/* &lt;script src=&quot;https://bad.invalid&quot;&gt;&amp;&#39; &lt;/code&gt;&lt;/pre&gt;&lt;img onerror=&#39;x&#39;&gt; */edition 2026;module verified{spec verified()-&gt;Word[8]^4{&quot;&lt;&gt;&amp;&#39;&quot;}test &quot;&lt;script&gt;verified &amp; &#39;title&#39;&quot;{true}}"
        );
    }

    #[test]
    fn controls_are_visible_without_normalizing_other_source_text() {
        let controls = [
            '\0', '\u{1}', '\u{7f}', '\u{85}', '\u{61c}', '\u{200e}', '\u{200f}', '\u{2028}',
            '\u{2029}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}',
            '\u{2067}', '\u{2068}', '\u{2069}',
        ];
        let mut comment = String::from("/* λ\r\n\t");
        comment.extend(controls);
        comment.push_str(" */");
        let text = format!("{comment}edition 2026;module m{{}}");
        let output = html(&text);
        let source = listing(&output);
        assert!(source.starts_with("/* λ\r\n\t\\u{0}\\u{1}\\u{7f}\\u{85}\\u{61c}"));
        for control in controls {
            assert!(!output.contains(control));
            assert!(source.contains(&format!("\\u{{{:x}}}", u32::from(control))));
        }
        assert!(source.ends_with(" */edition 2026;module m{}"));
    }

    #[test]
    fn duplicate_names_have_unique_source_order_anchors() {
        let output = html(
            "edition 2026;module m{spec duplicate(){}test \"duplicate\"{true}impl duplicate(){}test \"duplicate\"{false}spec duplicate(){}}",
        );
        assert_eq!(output.matches("<section id=\"declaration-").count(), 6);
        for ordinal in 1..=6 {
            assert_eq!(
                output
                    .matches(&format!("id=\"declaration-{ordinal}\""))
                    .count(),
                1
            );
            assert_eq!(
                output
                    .matches(&format!("href=\"#declaration-{ordinal}\""))
                    .count(),
                1
            );
        }
        assert_eq!(output.matches("<h3>spec duplicate</h3>").count(), 2);
        assert_eq!(
            output
                .matches("<h3>test &quot;duplicate&quot;</h3>")
                .count(),
            2
        );
    }

    #[test]
    fn locations_use_input_unicode_and_crlf_columns() {
        let output = html("edition 2026;\r\n/*λ*/ module m {\r\n  spec f()->Int {999999}\r\n}");
        assert!(output.contains("Line 2, column 7. Module declaration."));
        assert!(output.contains("Line 3, column 3. Spec declaration; body omitted."));
        assert_eq!(output.matches("999999").count(), 1);
        assert!(listing(&output).contains("\r\n/*λ*/ module"));
    }

    #[test]
    fn parsed_sources_need_no_imports_type_acceptance_or_test_execution() {
        let output = html(
            "edition 2026;module m{use absent;type Invalid=Word[7];spec f()->Unknown{absent::function()}test \"fails\"{false}}",
        );
        assert!(output.contains("type Invalid=Word[7];"));
        assert!(output.contains("spec f()-&gt;Unknown"));
        assert!(output.contains("test not run"));
        assert!(!output.contains("test passed"));
    }

    #[test]
    fn frontend_errors_return_original_diagnostics_and_no_html() {
        for (text, code) in [
            (
                "edition 2026;module m{@}",
                DiagnosticCode::UnexpectedCharacter,
            ),
            (
                "edition 2026;module m{/*",
                DiagnosticCode::UnterminatedBlockComment,
            ),
            (
                "edition 2027;module m{}",
                DiagnosticCode::UnsupportedSourceEdition,
            ),
            (
                "edition 2026;module m{spec f()->Int{}}",
                DiagnosticCode::ExpectedSyntax,
            ),
        ] {
            let output = result("errors.or", text);
            assert!(output.has_errors());
            assert_eq!(output.html(), None);
            assert!(
                output
                    .diagnostics()
                    .iter()
                    .any(|diagnostic| diagnostic.code() == code)
            );
        }
    }

    #[test]
    fn exact_output_and_work_limits_return_no_partial_document() {
        let text = "edition 2026;module m{}";
        let expected = html(text);
        let mut sources = SourceMap::new();
        let id = sources.add("budget.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let accepted = document_with_limits(
            source,
            Edition::E2026,
            Limits {
                output: expected.len(),
                events: 3,
            },
        );
        assert_eq!(accepted.html(), Some(expected.as_str()));
        for limits in [
            Limits {
                output: expected.len() - 1,
                events: 3,
            },
            Limits {
                output: expected.len(),
                events: 2,
            },
        ] {
            let output = document_with_limits(source, Edition::E2026, limits);
            assert!(output.has_errors());
            assert_eq!(output.html(), None);
            assert_eq!(
                output.diagnostics()[0].code(),
                DiagnosticCode::DocumentationResourceLimit
            );
        }
    }

    #[test]
    fn stale_header_spans_and_legacy_tokens_fail_closed() {
        let text = "edition 2026;module m{spec f(){}}";
        let mut sources = SourceMap::new();
        let id = sources.add("original.or", text).unwrap();
        let other_id = sources.add("foreign.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let foreign = sources.get(other_id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let parsed = parse(source, &lexed);
        let function = &parsed.ast().unwrap().module.functions[0];
        assert_eq!(
            header(
                source,
                foreign
                    .span(TextOffset::new(0), TextOffset::new(1))
                    .unwrap(),
                TextOffset::new(1)
            ),
            Err(Failure::Inconsistent)
        );
        assert!(matches!(
            function_entry(source, &[], function),
            Err(Failure::Inconsistent)
        ));
        let bad = Entry {
            kind: Kind::Spec,
            name: "f",
            span: function.span,
            header: foreign
                .span(TextOffset::new(0), TextOffset::new(1))
                .unwrap(),
            legacy: true,
        };
        assert_eq!(
            render(
                source,
                &[bad],
                &mut Budget { remaining: 2 },
                MAX_DOCUMENTATION_HTML_BYTES
            ),
            Err(Failure::Inconsistent)
        );
    }

    #[test]
    fn maximum_expression_height_and_nested_comment_fit_one_mebibyte_stack() {
        std::thread::Builder::new()
            .stack_size(1024 * 1024)
            .spawn(|| {
                let expression = format!("x{}", "[0]".repeat(MAX_EXPRESSION_HEIGHT - 1));
                html(&format!(
                    "edition 2026;module m{{spec f(x:Int)->Int{{{expression}}}}}"
                ));
                let comment = format!("{}inside{}", "/*".repeat(65_536), "*/".repeat(65_536));
                html(&format!("{comment}edition 2026;module m{{}}"));
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn repository_parsed_sources_generate_complete_references() {
        fn visit(path: &Path, count: &mut usize) {
            let mut children = fs::read_dir(path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            children.sort();
            for path in children {
                if path.is_dir() {
                    visit(&path, count);
                } else if path.extension().is_some_and(|extension| extension == "or") {
                    let text = fs::read_to_string(&path).unwrap();
                    let mut sources = SourceMap::new();
                    let id = sources.add("corpus.or", text.as_str()).unwrap();
                    let source = sources.get(id).unwrap();
                    let lexed = lex(source, Edition::E2026);
                    if !lexed.has_errors() && !parse(source, &lexed).has_errors() {
                        let output = document_source(source, Edition::E2026);
                        assert!(
                            !output.has_errors(),
                            "{}: {:?}",
                            path.display(),
                            output.diagnostics()
                        );
                        let output = output.into_html().unwrap();
                        assert!(output.ends_with("</html>\n"));
                        assert!(output.contains("<pre id=\"source\"><code>"));
                        *count += 1;
                    }
                }
            }
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mut count = 0;
        visit(&root.join("algorithms"), &mut count);
        visit(&root.join("compiler/fixtures"), &mut count);
        assert!(count >= 100, "only {count} parseable sources inspected");
        eprintln!("documented {count} repository sources");
    }
}
