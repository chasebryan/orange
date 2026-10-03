//! Deterministic, syntax-aware formatting with lossless token and comment anchors.
//!
//! Formatting changes only whitespace between lexer tokens and comments. It
//! does not resolve imports or types. An iterative syntax walk distinguishes
//! infix operators, prefix operators, array types, fill separators and finite
//! type domains before the token-gap renderer chooses their whitespace.

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::edition::Edition;
use crate::lexer::{Lexed, Token, TokenKind, lex};
use crate::parser::{
    Binding, Expression, ExpressionKind, FunctionBody, FunctionDeclaration,
    MAX_SYNTAX_NODES_PER_SOURCE, ParseResult, Pattern, Size, SyntaxTree, TypeSyntax, parse,
};
use crate::source::{MAX_SOURCE_BYTES, SourceFile, SourceMap, Span, TextOffset};

/// Maximum bytes in one complete formatted source, including preserved comments.
///
/// The output fits the same source envelope as the input and is relexed and
/// reparsed before it can be returned.
pub const MAX_FORMATTED_SOURCE_BYTES: usize = 16 * 1024 * 1024;

/// Maximum syntax work items visited while constructing one formatting plan.
///
/// The parser's node envelope bounds expressions and types; the additional
/// factor admits declaration, binding and size traversal without recursion.
pub const MAX_FORMAT_EVENTS_PER_SOURCE: usize = 4 * 262_144;

const _: () = assert!(MAX_FORMATTED_SOURCE_BYTES == MAX_SOURCE_BYTES);
const _: () = assert!(MAX_FORMAT_EVENTS_PER_SOURCE == MAX_SYNTAX_NODES_PER_SOURCE * 4);

/// The complete result of formatting one source.
#[derive(Debug, Eq, PartialEq)]
pub struct FormatResult {
    formatted: Option<String>,
    diagnostics: FormatDiagnostics,
}

#[derive(Debug, Eq, PartialEq)]
enum FormatDiagnostics {
    None,
    Lexical(Lexed),
    Syntax(ParseResult),
    Formatting(Vec<Diagnostic>),
}

impl FormatResult {
    /// Returns the complete validated text, or `None` after any failure.
    #[must_use]
    pub fn formatted(&self) -> Option<&str> {
        self.formatted.as_deref()
    }

    /// Consumes the result and returns its complete validated text.
    #[must_use]
    pub fn into_formatted(self) -> Option<String> {
        self.formatted
    }

    /// Returns the original frontend errors or a formatting failure diagnostic.
    ///
    /// If diagnostic storage itself cannot be reserved this slice is empty;
    /// [`Self::has_errors`] still reports failure.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match &self.diagnostics {
            FormatDiagnostics::None => &[],
            FormatDiagnostics::Lexical(result) => result.diagnostics(),
            FormatDiagnostics::Syntax(result) => result.diagnostics(),
            FormatDiagnostics::Formatting(diagnostics) => diagnostics,
        }
    }

    /// Returns whether formatting failed, including unreportable allocation failure.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.formatted.is_none()
    }
}

/// Formats all syntax currently parsed in the selected Orange edition.
///
/// Token spellings and order, grouping, literal spellings, and complete comment
/// bytes are preserved. Each comment stays between the same adjacent tokens.
/// Whitespace outside comments uses two-space block indentation and LF line
/// endings, with one final LF. Lists and expressions are not width-wrapped.
/// Lexical and syntax errors return their existing diagnostics without text.
/// Resource or preservation failures likewise return no partial text.
#[must_use]
pub fn format_source(source: &SourceFile, edition: Edition) -> FormatResult {
    format_with_limits(source, edition, Limits::DEFAULT)
}

#[derive(Clone, Copy)]
struct Limits {
    output: usize,
    events: usize,
}

impl Limits {
    const DEFAULT: Self = Self {
        output: MAX_FORMATTED_SOURCE_BYTES,
        events: MAX_FORMAT_EVENTS_PER_SOURCE,
    };
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Failure {
    Resource,
    Inconsistent,
}

fn failed(source: &SourceFile, failure: Failure) -> FormatResult {
    let mut diagnostics = Vec::new();
    if diagnostics.try_reserve_exact(1).is_ok() {
        let (code, message, note) = match failure {
            Failure::Resource => (
                DiagnosticCode::FormatResourceLimit,
                "formatting exceeded its resource budget",
                "formatting uses bounded syntax storage and at most 16777216 output bytes",
            ),
            Failure::Inconsistent => (
                DiagnosticCode::FormattingInconsistency,
                "formatting could not preserve the parsed source",
                "no formatted source was produced; report this compiler inconsistency",
            ),
        };
        if let Some(span) = source.span(TextOffset::new(0), TextOffset::new(0)) {
            diagnostics.push(Diagnostic::error(code, message, span).with_note(note));
        }
    }
    FormatResult {
        formatted: None,
        diagnostics: FormatDiagnostics::Formatting(diagnostics),
    }
}

fn format_with_limits(source: &SourceFile, edition: Edition, limits: Limits) -> FormatResult {
    let lexed = lex(source, edition);
    if lexed.has_errors() {
        return FormatResult {
            formatted: None,
            diagnostics: FormatDiagnostics::Lexical(lexed),
        };
    }
    let parsed = parse(source, &lexed);
    let Some(tree) = parsed.ast() else {
        return FormatResult {
            formatted: None,
            diagnostics: FormatDiagnostics::Syntax(parsed),
        };
    };
    let result = Plan::build(lexed.tokens(), tree, limits.events).and_then(|plan| {
        let formatted = render(source, lexed.tokens(), &plan, limits.output)?;
        validate(
            source,
            lexed.tokens(),
            &formatted,
            edition,
            &plan,
            limits.output,
        )?;
        Ok(formatted)
    });
    match result {
        Ok(formatted) => FormatResult {
            formatted: Some(formatted),
            diagnostics: FormatDiagnostics::None,
        },
        Err(failure) => failed(source, failure),
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Role {
    infix: bool,
    prefix: bool,
    array_caret: bool,
    fill_separator: bool,
    domain: bool,
    declaration: bool,
    condition_group: bool,
    following_space: bool,
    postfix_index: bool,
}

struct Plan {
    roles: Vec<Role>,
}

enum Work<'a> {
    Expression(&'a Expression),
    Type(&'a TypeSyntax),
    Binding(&'a Binding),
    Size(&'a Size),
}

struct Planner<'a> {
    tokens: &'a [Token],
    plan: Plan,
    work: Vec<Work<'a>>,
    events: usize,
    limit: usize,
}

impl Plan {
    fn build(tokens: &[Token], tree: &SyntaxTree, limit: usize) -> Result<Self, Failure> {
        let mut roles = Vec::new();
        roles
            .try_reserve_exact(tokens.len())
            .map_err(|_| Failure::Resource)?;
        roles.resize(tokens.len(), Role::default());
        let mut planner = Planner {
            tokens,
            plan: Self { roles },
            work: Vec::new(),
            events: 0,
            limit,
        };
        planner.declaration(tree.edition.span)?;
        for declaration in &tree.module.uses {
            planner.declaration(declaration.span)?;
        }
        for declaration in &tree.module.types {
            planner.declaration(declaration.span)?;
            planner.push(Work::Type(&declaration.ty))?;
        }
        for function in &tree.module.functions {
            planner.declaration(function.span)?;
            planner.function(function)?;
        }
        for test in &tree.module.tests {
            planner.declaration(test.span)?;
            planner.function(&test.function)?;
        }
        while let Some(work) = planner.work.pop() {
            planner.visit(work)?;
        }
        Ok(planner.plan)
    }
}

impl<'a> Planner<'a> {
    fn index(&self, span: Span) -> Result<usize, Failure> {
        self.tokens
            .binary_search_by_key(&span.start(), |token| token.span.start())
            .map_err(|_| Failure::Inconsistent)
    }

    fn after(&self, span: Span) -> usize {
        self.tokens
            .partition_point(|token| token.span.start() < span.end())
    }

    fn declaration(&mut self, span: Span) -> Result<(), Failure> {
        let index = self.index(span)?;
        self.role(index)?.declaration = true;
        Ok(())
    }

    fn role(&mut self, index: usize) -> Result<&mut Role, Failure> {
        self.plan.roles.get_mut(index).ok_or(Failure::Inconsistent)
    }

    fn push(&mut self, work: Work<'a>) -> Result<(), Failure> {
        self.events = self.events.checked_add(1).ok_or(Failure::Resource)?;
        if self.events > self.limit {
            return Err(Failure::Resource);
        }
        self.work.try_reserve(1).map_err(|_| Failure::Resource)?;
        self.work.push(work);
        Ok(())
    }

    fn pattern(&mut self, pattern: &'a Pattern) -> Result<(), Failure> {
        for name in pattern.names() {
            self.push(Work::Type(&name.ty))?;
        }
        Ok(())
    }

    fn function(&mut self, function: &'a FunctionDeclaration) -> Result<(), Failure> {
        for size in &function.sizes {
            if size.is_type() {
                let start = self.index(size.start_span)?;
                let end = self.index(size.end_span)?;
                self.role(start)?.domain = true;
                self.role(end)?.domain = true;
                for ty in &size.types {
                    self.push(Work::Type(ty))?;
                }
            }
        }
        for parameter in &function.parameters {
            self.push(Work::Type(&parameter.ty))?;
        }
        if let FunctionBody::Typed(body) = &function.body {
            self.push(Work::Type(&body.result_type))?;
            for binding in &body.bindings {
                self.push(Work::Binding(binding))?;
            }
            self.push(Work::Expression(&body.expression))?;
        }
        Ok(())
    }

    fn bindings(&mut self, bindings: &'a [Binding]) -> Result<(), Failure> {
        for binding in bindings {
            self.push(Work::Binding(binding))?;
        }
        Ok(())
    }

    fn range(&mut self, range: &'a crate::parser::SliceRange) -> Result<(), Failure> {
        if let Some(start) = &range.start {
            self.push(Work::Expression(start))?;
        }
        if let Some(end) = &range.end {
            self.push(Work::Expression(end))?;
        }
        Ok(())
    }

    fn condition(&mut self, expression: &'a Expression) -> Result<(), Failure> {
        let index = self.index(expression.span)?;
        self.role(index)?.condition_group = true;
        self.push(Work::Expression(expression))
    }

    fn postfix_index(&mut self, base: Span) -> Result<(), Failure> {
        let index = self.after(base);
        if self.tokens.get(index).map(|token| token.kind) != Some(TokenKind::LeftBracket) {
            return Err(Failure::Inconsistent);
        }
        self.role(index)?.postfix_index = true;
        Ok(())
    }

    fn visit(&mut self, work: Work<'a>) -> Result<(), Failure> {
        match work {
            Work::Binding(binding) => {
                let index = self.index(binding.span)?;
                self.role(index)?.following_space = true;
                self.pattern(&binding.pattern)?;
                self.push(Work::Expression(&binding.value))?;
            }
            Work::Size(size) => {
                if let Some(expression) = &size.expression {
                    self.push(Work::Expression(expression))?;
                }
            }
            Work::Type(ty) => {
                if let Some(length) = &ty.length {
                    let index = self.index(length.span)?;
                    let caret = index.checked_sub(1).ok_or(Failure::Inconsistent)?;
                    if self.tokens.get(caret).map(|token| token.kind) != Some(TokenKind::Caret) {
                        return Err(Failure::Inconsistent);
                    }
                    self.role(caret)?.array_caret = true;
                    self.push(Work::Size(length))?;
                }
                if let Some(modulus) = &ty.modulus {
                    self.push(Work::Expression(modulus))?;
                }
                for element in &ty.elements {
                    self.push(Work::Type(element))?;
                }
            }
            Work::Expression(expression) => match &expression.kind {
                ExpressionKind::Literal(literal) => {
                    if literal.negative {
                        let index = self.index(literal.span)?;
                        self.role(index)?.prefix = true;
                    }
                }
                ExpressionKind::Name(_) | ExpressionKind::Bytes(_) => {}
                ExpressionKind::Unary(unary) => {
                    let index = self.index(unary.operator_span)?;
                    self.role(index)?.prefix = true;
                    self.push(Work::Expression(&unary.operand))?;
                }
                ExpressionKind::Binary(binary) => {
                    let index = self.index(binary.operator_span)?;
                    self.role(index)?.infix = true;
                    self.push(Work::Expression(&binary.left))?;
                    self.push(Work::Expression(&binary.right))?;
                }
                ExpressionKind::Parenthesized(inner) => self.push(Work::Expression(inner))?,
                ExpressionKind::Call(call) => {
                    for argument in &call.arguments {
                        self.push(Work::Expression(argument))?;
                    }
                    for size in call.sizes() {
                        self.push(Work::Expression(size))?;
                    }
                }
                ExpressionKind::Conversion(conversion) => {
                    self.push(Work::Expression(&conversion.operand))?;
                    self.push(Work::Type(&conversion.target))?;
                }
                ExpressionKind::Array(array) => {
                    for element in &array.elements {
                        self.push(Work::Expression(element))?;
                    }
                }
                ExpressionKind::Tuple(tuple) => {
                    for element in &tuple.elements {
                        self.push(Work::Expression(element))?;
                    }
                }
                ExpressionKind::Fill(fill) => {
                    let index = self.after(fill.element.span);
                    if self.tokens.get(index).map(|token| token.kind) != Some(TokenKind::Semicolon)
                    {
                        return Err(Failure::Inconsistent);
                    }
                    self.role(index)?.fill_separator = true;
                    self.push(Work::Expression(&fill.element))?;
                    self.push(Work::Size(&fill.length))?;
                }
                ExpressionKind::Index(index) => {
                    self.postfix_index(index.base.span)?;
                    self.push(Work::Expression(&index.base))?;
                    self.push(Work::Expression(&index.index))?;
                }
                ExpressionKind::Project(project) => self.push(Work::Expression(&project.base))?,
                ExpressionKind::Update(update) => {
                    let index = self.index(update.keyword_span)?;
                    self.role(index)?.following_space = true;
                    self.push(Work::Expression(&update.base))?;
                    self.push(Work::Expression(&update.index))?;
                    self.push(Work::Expression(&update.value))?;
                }
                ExpressionKind::Slice(slice) => {
                    self.postfix_index(slice.base.span)?;
                    self.push(Work::Expression(&slice.base))?;
                    self.range(&slice.range)?;
                }
                ExpressionKind::SliceUpdate(update) => {
                    let index = self.index(update.keyword_span)?;
                    self.role(index)?.following_space = true;
                    self.push(Work::Expression(&update.base))?;
                    self.range(&update.range)?;
                    self.push(Work::Expression(&update.value))?;
                }
                ExpressionKind::Loop(loop_expression) => {
                    let index = self.index(loop_expression.accumulator.span())?;
                    self.role(index.checked_sub(1).ok_or(Failure::Inconsistent)?)?
                        .following_space = true;
                    self.push(Work::Size(&loop_expression.start))?;
                    self.push(Work::Size(&loop_expression.end))?;
                    self.pattern(&loop_expression.accumulator)?;
                    self.push(Work::Expression(&loop_expression.init))?;
                    self.bindings(&loop_expression.step_bindings)?;
                    self.push(Work::Expression(&loop_expression.step))?;
                }
                ExpressionKind::Conditional(conditional) => {
                    for arm in &conditional.arms {
                        self.condition(&arm.condition)?;
                        self.bindings(&arm.bindings)?;
                        self.push(Work::Expression(&arm.value))?;
                    }
                    self.bindings(&conditional.otherwise_bindings)?;
                    self.push(Work::Expression(&conditional.otherwise))?;
                }
            },
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Gap {
    None,
    Space,
    Line,
    Blank,
}

fn gap(tokens: &[Token], plan: &Plan, index: usize) -> Result<Gap, Failure> {
    use TokenKind as T;
    if index == 0 {
        return Ok(Gap::None);
    }
    let current = tokens.get(index).ok_or(Failure::Inconsistent)?.kind;
    let previous_index = index.checked_sub(1).ok_or(Failure::Inconsistent)?;
    let previous = tokens
        .get(previous_index)
        .ok_or(Failure::Inconsistent)?
        .kind;
    let role = plan.roles.get(index).ok_or(Failure::Inconsistent)?;
    let prior = plan
        .roles
        .get(previous_index)
        .ok_or(Failure::Inconsistent)?;
    if current == T::Eof {
        return Ok(Gap::Line);
    }
    if current == T::RightBrace && !role.domain {
        return Ok(Gap::Line);
    }
    if previous == T::LeftBrace && !prior.domain {
        return Ok(Gap::Line);
    }
    if role.declaration {
        return Ok(Gap::Blank);
    }
    if previous == T::Semicolon && !prior.fill_separator {
        return Ok(Gap::Line);
    }
    if role.infix || prior.infix {
        return Ok(Gap::Space);
    }
    if role.array_caret || prior.array_caret || prior.prefix || role.postfix_index {
        return Ok(Gap::None);
    }
    if matches!(
        current,
        T::Comma
            | T::Semicolon
            | T::RightParen
            | T::RightBracket
            | T::Colon
            | T::Dot
            | T::DoubleColon
            | T::DotDot
    ) || matches!(
        previous,
        T::LeftParen | T::LeftBracket | T::Dot | T::DoubleColon | T::DotDot
    ) || (previous == T::LeftBrace && prior.domain)
        || (current == T::RightBrace && role.domain)
    {
        return Ok(Gap::None);
    }
    if current == T::LeftParen
        && !role.condition_group
        && !prior.following_space
        && matches!(previous, T::Identifier | T::RightBracket | T::RightParen)
    {
        return Ok(Gap::None);
    }
    if current == T::LeftBracket
        && !prior.following_space
        && matches!(
            previous,
            T::Identifier | T::RightBracket | T::RightParen | T::RightBrace
        )
    {
        return Ok(Gap::None);
    }
    if previous == T::RightBrace && !prior.domain {
        // Specific expression continuations above retain their own spacing;
        // an else branch stays attached to the closing conditional branch.
        return Ok(if current == T::Identifier {
            Gap::Space
        } else {
            Gap::Line
        });
    }
    Ok(Gap::Space)
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

    fn whitespace(&mut self, gap: Gap, indent: usize) -> Result<(), Failure> {
        if self.text.is_empty() {
            return Ok(());
        }
        match gap {
            Gap::None => {}
            Gap::Space => {
                if !self.text.ends_with([' ', '\n', '\r', '\t']) {
                    self.append(" ")?;
                }
            }
            Gap::Line | Gap::Blank => {
                let count = if gap == Gap::Blank { 2 } else { 1 };
                let existing = self
                    .text
                    .as_bytes()
                    .iter()
                    .rev()
                    .take_while(|&&byte| byte == b'\n')
                    .count();
                for _ in existing..count {
                    self.append("\n")?;
                }
            }
        }
        if self.text.ends_with('\n') {
            for _ in 0..indent {
                self.append("  ")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CommentKind {
    Line,
    Block,
}

struct Comment<'a> {
    text: &'a str,
    kind: CommentKind,
    line_before: bool,
}

/// Scans only gaps already admitted by the lexer; literals never enter here.
struct Trivia<'a> {
    text: &'a str,
    offset: usize,
    trailing_line: bool,
}

impl<'a> Trivia<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            text,
            offset: 0,
            trailing_line: false,
        }
    }

    fn next(&mut self) -> Result<Option<Comment<'a>>, Failure> {
        let text = self.text;
        let bytes = text.as_bytes();
        let mut line_before = false;
        while let Some(&byte) = bytes.get(self.offset) {
            if !matches!(byte, b' ' | b'\t' | b'\r' | b'\n') {
                break;
            }
            line_before |= matches!(byte, b'\r' | b'\n');
            self.advance(1)?;
        }
        self.trailing_line = line_before;
        let start = self.offset;
        if start == bytes.len() {
            return Ok(None);
        }
        let remaining = bytes.get(start..).ok_or(Failure::Inconsistent)?;
        let kind = if remaining.starts_with(b"//") {
            self.advance(2)?;
            while bytes
                .get(self.offset)
                .is_some_and(|byte| !matches!(byte, b'\r' | b'\n'))
            {
                self.advance(1)?;
            }
            CommentKind::Line
        } else if remaining.starts_with(b"/*") {
            self.advance(2)?;
            let mut depth = 1_usize;
            while depth != 0 {
                let remaining = bytes.get(self.offset..).ok_or(Failure::Inconsistent)?;
                if remaining.starts_with(b"/*") {
                    depth = depth.checked_add(1).ok_or(Failure::Resource)?;
                    self.advance(2)?;
                } else if remaining.starts_with(b"*/") {
                    depth = depth.checked_sub(1).ok_or(Failure::Inconsistent)?;
                    self.advance(2)?;
                } else if self.offset < bytes.len() {
                    self.advance(1)?;
                } else {
                    return Err(Failure::Inconsistent);
                }
            }
            CommentKind::Block
        } else {
            return Err(Failure::Inconsistent);
        };
        let text = self
            .text
            .get(start..self.offset)
            .ok_or(Failure::Inconsistent)?;
        Ok(Some(Comment {
            text,
            kind,
            line_before,
        }))
    }

    fn advance(&mut self, bytes: usize) -> Result<(), Failure> {
        self.offset = self
            .offset
            .checked_add(bytes)
            .ok_or(Failure::Inconsistent)?;
        Ok(())
    }
}

fn source_gap<'a>(
    source: &'a SourceFile,
    tokens: &[Token],
    index: usize,
) -> Result<&'a str, Failure> {
    let start = if index == 0 {
        TextOffset::new(0)
    } else {
        tokens
            .get(index.checked_sub(1).ok_or(Failure::Inconsistent)?)
            .ok_or(Failure::Inconsistent)?
            .span
            .end()
    };
    let end = tokens.get(index).ok_or(Failure::Inconsistent)?.span.start();
    source
        .span(start, end)
        .and_then(|span| source.slice(span))
        .ok_or(Failure::Inconsistent)
}

fn render(
    source: &SourceFile,
    tokens: &[Token],
    plan: &Plan,
    limit: usize,
) -> Result<String, Failure> {
    let mut output = Output {
        text: String::new(),
        limit,
    };
    let mut indent = 0_usize;
    for (index, token) in tokens.iter().enumerate() {
        let role = plan.roles.get(index).ok_or(Failure::Inconsistent)?;
        if token.kind == TokenKind::RightBrace && !role.domain {
            indent = indent.checked_sub(1).ok_or(Failure::Inconsistent)?;
        }
        let desired = gap(tokens, plan, index)?;
        let mut trivia = Trivia::new(source_gap(source, tokens, index)?);
        let mut comment_count = 0_usize;
        let mut force_line = false;
        let mut first_trailing_line = false;
        while let Some(comment) = trivia.next()? {
            let trailing_line =
                comment.kind == CommentKind::Line && !comment.line_before && index != 0;
            if comment_count == 0 {
                first_trailing_line = trailing_line;
            }
            let before = if trailing_line {
                Gap::Space
            } else if comment_count == 1
                && first_trailing_line
                && desired == Gap::Blank
                && comment.line_before
            {
                Gap::Blank
            } else if comment.line_before || force_line {
                if comment_count == 0 {
                    desired.max(Gap::Line)
                } else {
                    Gap::Line
                }
            } else if comment_count == 0 {
                desired.max(Gap::Space)
            } else {
                Gap::Space
            };
            output.whitespace(before, indent)?;
            output.append(comment.text)?;
            force_line = comment.kind == CommentKind::Line
                || comment.text.contains(['\r', '\n'])
                || before >= Gap::Line;
            comment_count = comment_count.checked_add(1).ok_or(Failure::Resource)?;
        }
        let after = if force_line || (comment_count != 0 && trivia.trailing_line) {
            if desired == Gap::Blank && first_trailing_line && comment_count == 1 {
                Gap::Blank
            } else {
                Gap::Line
            }
        } else if comment_count != 0 {
            desired.max(Gap::Space)
        } else {
            desired
        };
        if token.kind == TokenKind::Eof {
            // Do not indent or trim a preserved comment's own bytes.
            output.whitespace(Gap::Line, 0)?;
        } else {
            output.whitespace(after, indent)?;
            output.append(token.lexeme(source).ok_or(Failure::Inconsistent)?)?;
        }
        if token.kind == TokenKind::LeftBrace && !role.domain {
            indent = indent.checked_add(1).ok_or(Failure::Resource)?;
        }
    }
    if indent != 0 {
        return Err(Failure::Inconsistent);
    }
    Ok(output.text)
}

fn equivalent(
    source: &SourceFile,
    tokens: &[Token],
    other: &SourceFile,
    other_tokens: &[Token],
) -> Result<(), Failure> {
    if tokens.len() != other_tokens.len() {
        return Err(Failure::Inconsistent);
    }
    for (index, (original, formatted)) in tokens.iter().zip(other_tokens).enumerate() {
        if original.kind != formatted.kind || original.lexeme(source) != formatted.lexeme(other) {
            return Err(Failure::Inconsistent);
        }
        let mut before = Trivia::new(source_gap(source, tokens, index)?);
        let mut after = Trivia::new(source_gap(other, other_tokens, index)?);
        loop {
            match (before.next()?, after.next()?) {
                (None, None) => break,
                (Some(left), Some(right)) if left.kind == right.kind && left.text == right.text => {
                }
                _ => return Err(Failure::Inconsistent),
            }
        }
    }
    Ok(())
}

fn validate(
    source: &SourceFile,
    tokens: &[Token],
    formatted: &str,
    edition: Edition,
    plan: &Plan,
    limit: usize,
) -> Result<(), Failure> {
    let mut sources = SourceMap::new();
    let id = sources
        .add("<formatted>", formatted)
        .map_err(|_| Failure::Resource)?;
    let other = sources.get(id).ok_or(Failure::Inconsistent)?;
    let lexed = lex(other, edition);
    if lexed.has_errors() {
        return Err(Failure::Inconsistent);
    }
    equivalent(source, tokens, other, lexed.tokens())?;
    if parse(other, &lexed).has_errors() {
        return Err(Failure::Inconsistent);
    }
    if render(other, lexed.tokens(), plan, limit)? != formatted {
        return Err(Failure::Inconsistent);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;
    use crate::{MAX_ARRAY_ELEMENTS, MAX_EXPRESSION_HEIGHT, analyze, evaluate};

    fn result(text: &str) -> FormatResult {
        let mut sources = SourceMap::new();
        let id = sources.add("format.or", text).unwrap();
        format_source(sources.get(id).unwrap(), Edition::E2026)
    }

    fn formatted(text: &str) -> String {
        let output = result(text);
        assert!(!output.has_errors(), "{text:?}: {:?}", output.diagnostics());
        assert!(output.diagnostics().is_empty());
        let output = output.into_formatted().unwrap();
        let second = result(&output);
        assert!(
            !second.has_errors(),
            "{output:?}: {:?}",
            second.diagnostics()
        );
        assert_eq!(second.formatted(), Some(output.as_str()));
        output
    }

    #[test]
    fn canonical_blocks_and_contextual_roles() {
        assert_eq!(
            formatted("edition 2026;module m{spec x()->Int{1}}"),
            "edition 2026;\nmodule m {\n  spec x() -> Int {\n    1\n  }\n}\n"
        );
        let output = formatted(
            "edition 2026;module m{type R=Word[8]^4;spec f[T in {R,Int},n in 1..3](a:R)->R{let x:R=[-1;n];a with [0]=(~a[0])}test \"x\"{if (1==1){1==1}else{1!=2}}impl untouched(){}} ",
        );
        assert!(output.contains("type R = Word[8]^4;"));
        assert!(output.contains("f[T in {R, Int}, n in 1..3](a: R) -> R"));
        assert!(output.contains("let x: R = [-1; n];\n"));
        assert!(output.contains("a with [0] = (~a[0])"), "{output}");
        assert!(output.contains("if (1 == 1) {\n"));
        assert!(output.contains("} else {\n"));
        assert!(output.contains("\n\n  test \"x\""));
    }

    fn assert_body_layout(expression: &str, body: &str) {
        let source = format!("edition 2026;module m{{spec f()->Int{{{expression}}}}}");
        let expected =
            format!("edition 2026;\nmodule m {{\n  spec f() -> Int {{\n{body}\n  }}\n}}\n");
        assert_eq!(formatted(&source), expected);
    }

    #[test]
    fn block_operands_keep_infix_continuations_on_the_closing_line() {
        for (expression, expected) in [
            (
                "if true{1}else{2}+3",
                "    if true {\n      1\n    } else {\n      2\n    } + 3",
            ),
            (
                "3+if true{1}else{2}",
                "    3 + if true {\n      1\n    } else {\n      2\n    }",
            ),
            (
                "for i in 0..1 with s:Int=0{s+1}+3",
                "    for i in 0..1 with s: Int = 0 {\n      s + 1\n    } + 3",
            ),
            (
                "(for i in 0..1 with s:Int=0{s+1})*3",
                "    (for i in 0..1 with s: Int = 0 {\n      s + 1\n    }) * 3",
            ),
            (
                "if true{1}else{2}==3",
                "    if true {\n      1\n    } else {\n      2\n    } == 3",
            ),
            (
                "if true{[1]}else{[2]}++[3]",
                "    if true {\n      [1]\n    } else {\n      [2]\n    } ++ [3]",
            ),
        ] {
            assert_body_layout(expression, expected);
        }
    }

    #[test]
    fn block_elements_keep_commas_and_closing_delimiters_tight() {
        for (expression, expected) in [
            (
                "(if true{1}else{2},for i in 0..1 with s:Int=0{s})",
                "    (if true {\n      1\n    } else {\n      2\n    }, for i in 0..1 with s: Int = 0 {\n      s\n    })",
            ),
            (
                "g(for i in 0..1 with s:Int=0{s},if true{1}else{2})",
                "    g(for i in 0..1 with s: Int = 0 {\n      s\n    }, if true {\n      1\n    } else {\n      2\n    })",
            ),
            (
                "[if true{1}else{2},for i in 0..1 with s:Int=0{s}]",
                "    [if true {\n      1\n    } else {\n      2\n    }, for i in 0..1 with s: Int = 0 {\n      s\n    }]",
            ),
            (
                "[for i in 0..1 with s:Int=0{s};2]",
                "    [for i in 0..1 with s: Int = 0 {\n      s\n    }; 2]",
            ),
            (
                "(if true{1}else{2})",
                "    (if true {\n      1\n    } else {\n      2\n    })",
            ),
        ] {
            assert_body_layout(expression, expected);
        }
    }

    #[test]
    fn selections_of_block_valued_calls_stay_tight() {
        let source = "edition 2026;module m{spec a()->Word[8]^2{if true{[1,2]}else{[3,4]}}spec p()->(Int,Int){for i in 0..1 with s:(Int,Int)=(1,2){s}}spec f()->Int{(a()[0] as Int)+p().0}}";
        assert_eq!(
            formatted(source),
            "edition 2026;\nmodule m {\n  spec a() -> Word[8]^2 {\n    if true {\n      [1, 2]\n    } else {\n      [3, 4]\n    }\n  }\n\n  spec p() -> (Int, Int) {\n    for i in 0..1 with s: (Int, Int) = (1, 2) {\n      s\n    }\n  }\n\n  spec f() -> Int {\n    (a()[0] as Int) + p().0\n  }\n}\n"
        );
    }

    #[test]
    fn tuple_projections_keep_following_indices_and_slices_tight() {
        let source = "edition 2026;module m{spec f()->(Int,Int^2,Int^2,Int^3){let p:(Int^3,Int)=([1,2,3],4);(p.0[1],p.0[1..],p.0[..2],p.0[0..3])}}";
        assert_eq!(
            formatted(source),
            "edition 2026;\nmodule m {\n  spec f() -> (Int, Int^2, Int^2, Int^3) {\n    let p: (Int^3, Int) = ([1, 2, 3], 4);\n    (p.0[1], p.0[1..], p.0[..2], p.0[0..3])\n  }\n}\n"
        );
        let output = formatted(
            "edition 2026;module m{spec f()->(Int,Int^1){let p:(Int^2,Int)=([1,2],3);(p.0/* index anchor */[0],p.0/* slice anchor */[..1])}}",
        );
        assert_eq!(output.matches("/* index anchor */").count(), 1);
        assert_eq!(output.matches("/* slice anchor */").count(), 1);
    }

    #[test]
    fn tuple_bindings_keep_a_space_after_the_let_keyword() {
        let source = "edition 2026;module m{spec f()->Int{let(a:Int,b:Int)=(1,2);for i in 0..1 with s:Int=a{let(c:Int,d:Int)=(s,b);if true{let(e:Int,f:Int)=(c,d);e+f}else{c}}}}";
        assert_eq!(
            formatted(source),
            "edition 2026;\nmodule m {\n  spec f() -> Int {\n    let (a: Int, b: Int) = (1, 2);\n    for i in 0..1 with s: Int = a {\n      let (c: Int, d: Int) = (s, b);\n      if true {\n        let (e: Int, f: Int) = (c, d);\n        e + f\n      } else {\n        c\n      }\n    }\n  }\n}\n"
        );
    }

    #[test]
    fn brace_postfix_spacing_does_not_expand_the_parsed_grammar() {
        for (suffix, kind) in [("[0]", TokenKind::LeftBracket), (".0", TokenKind::Dot)] {
            let mut sources = SourceMap::new();
            let id = sources
                .add("postfix-tokens.or", format!("}}{suffix}"))
                .unwrap();
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::E2026);
            assert!(!lexed.has_errors());
            assert_eq!(lexed.tokens()[1].kind, kind);
            let plan = Plan {
                roles: vec![Role::default(); lexed.tokens().len()],
            };
            assert_eq!(gap(lexed.tokens(), &plan, 1), Ok(Gap::None));
        }
        for expression in [
            "if true{[1]}else{[2]}[0]",
            "for i in 0..1 with s:(Int,Int)=(1,2){s}.0",
        ] {
            let output = result(&format!(
                "edition 2026;module m{{spec f()->Int{{{expression}}}}}"
            ));
            assert!(output.has_errors());
            assert!(output.formatted().is_none());
            assert!(
                output
                    .diagnostics()
                    .iter()
                    .any(|diagnostic| diagnostic.code() == DiagnosticCode::ExpectedSyntax)
            );
        }
    }

    #[test]
    fn block_continuation_comments_keep_their_anchors_and_layout() {
        for (expression, expected) in [
            (
                "if true{1}else{2}/* arithmetic */+3",
                "    if true {\n      1\n    } else {\n      2\n    } /* arithmetic */ + 3",
            ),
            (
                "(for i in 0..1 with s:Int=0{s}/* comma */,4)",
                "    (for i in 0..1 with s: Int = 0 {\n      s\n    } /* comma */ , 4)",
            ),
            (
                "g(if true{1}else{2}/* closing */)",
                "    g(if true {\n      1\n    } else {\n      2\n    } /* closing */ )",
            ),
            (
                "[for i in 0..1 with s:Int=0{s}/* closing */]",
                "    [for i in 0..1 with s: Int = 0 {\n      s\n    } /* closing */ ]",
            ),
            (
                "if true{1}else{2}// arithmetic\n+3",
                "    if true {\n      1\n    } else {\n      2\n    } // arithmetic\n    + 3",
            ),
            (
                "a()/* index */[0]+p()/* projection */.0",
                "    a() /* index */ [0] + p() /* projection */ .0",
            ),
        ] {
            assert_body_layout(expression, expected);
        }
    }

    #[test]
    fn block_continuation_comment_arrangements_are_idempotent() {
        let text = "edition 2026;module m{spec f()->Int{let x:Int=if true{1}else{2};g(if true{1}else{2},for i in 0..1 with s:Int=0{s})+(if true{1}else{2})}spec a()->Int{[if true{1}else{2},for i in 0..1 with s:Int=0{s}]}}";
        let mut sources = SourceMap::new();
        let id = sources.add("continuation-gaps.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        for pair in lexed.tokens().windows(2) {
            if pair[0].kind != TokenKind::RightBrace {
                continue;
            }
            let offset = usize::try_from(pair[1].span.start().bytes()).unwrap();
            for comment in [
                "/* inline */",
                "\n/* standalone */\n",
                "// trailing\n",
                "/* a */ // b\n /* c */",
                "// a\n/* b */ /* c */",
                "/* nested /* inner */ */",
            ] {
                formatted(&format!("{}{comment}{}", &text[..offset], &text[offset..]));
            }
        }
    }

    #[test]
    fn declaration_comments_have_stable_separation() {
        let output = formatted(
            "edition 2026;module m{spec a(){}/* one */spec b(){}//two\n/* three */spec c(){}}",
        );
        assert!(output.contains("}\n\n  /* one */\n  spec b()"));
        assert!(output.contains("} //two\n"));
        assert!(output.contains("/* three */\n  spec c()"), "{output}");
    }

    #[test]
    fn comments_at_every_token_boundary_are_idempotent() {
        let text = "edition 2026;module m{type R=Word[8]^2;spec f[T in {R,Int}](a:R)->R{let x:R=[1;2];if (a[0]==1){a with [0]=2}else{a}}test \"x\"{1==1}impl z(){}}";
        let mut sources = SourceMap::new();
        let id = sources.add("boundaries.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        for token in lexed.tokens() {
            let offset = token.span.start().bytes() as usize;
            for comment in [
                "/*a*/",
                " /*a*/ ",
                "\n/*a*/\n",
                "//a\n",
                "\n//a\n",
                "/*a*/ //b\r\n /*c*/",
                "\n/*a*/ /*b*/\n",
                "//a\r\n//b\r\n",
                "/* a\r\n b */",
                "/*a*/\n//b\n/*c*/",
                "//a\n/*b*/\n//c\n",
                "//a\n/*b*/ /*c*/",
            ] {
                let insertion = format!("{}{comment}{}", &text[..offset], &text[offset..]);
                formatted(&insertion);
            }
        }
    }

    #[test]
    fn complete_comment_bytes_and_token_anchors_survive() {
        let source = "// header λ\r\nedition/*edition*/2026;\r\nmodule/*name*/m{/*block\r\n /* nested */\t end*/spec f/*call*/(/*param*/a:Word/*width*/[8]/*length*/^2)->Word[8]{//body\r\n a/*select*/[/*index*/0]/*tail*/}//end\r\n}//file";
        let output = formatted(source);
        assert!(output.starts_with("// header λ\nedition /*edition*/ 2026;\n"));
        assert!(output.contains("/*block\r\n /* nested */\t end*/"));
        assert!(output.contains("{ //body\n"));
        assert!(output.ends_with("} //file\n"));
        let mut sources = SourceMap::new();
        let left_id = sources.add("before.or", source).unwrap();
        let right_id = sources.add("after.or", output).unwrap();
        let left = sources.get(left_id).unwrap();
        let right = sources.get(right_id).unwrap();
        equivalent(
            left,
            lex(left, Edition::E2026).tokens(),
            right,
            lex(right, Edition::E2026).tokens(),
        )
        .unwrap();
    }

    #[test]
    fn strings_numbers_and_explicit_groups_are_opaque() {
        let output = formatted(
            r#"edition 2026;module m{spec f()->Int{(0x00_ff + (-0b0010))}spec b()->Word[8]^2{hex"00  fF"}test "escaped\" // /* title"{"// /* */ \x2f"=="// /* */ \x2f"}}"#,
        );
        assert!(output.contains("(0x00_ff + (-0b0010))"));
        assert!(output.contains(r#""// /* */ \x2f""#));
    }

    #[test]
    fn all_expression_variants_and_static_types_are_formatted() {
        let output = formatted(
            r#"edition 2026;module m{
use missing;type Row=Word[8]^4;type Matrix=Row^4;
spec identity[T in {Int,Row}](x:T)->T{x}
spec f[n in 2..4](a:Row,b:Int)->(Int,Row){
let (x:Int,y:Row)=(b,a);
let r:Mod[(1<<n)-1]=1;
let matrix:Matrix=[[0;4];4];
let z:Row=a with [0..2]=[1,2];
let p:Row=for i in 0..n with s:Row=[0;4]{let v:Word[8]=i as Word[8];s with [i]=v};
if (x>0){(identity[Int](x),z[0..])}else if (!((x==0)||(x<0))){(missing::get[n](x),([1,2]++[3,4]))}else{let v:Int=(matrix[0][0] as Int);(v,((p as big Int) as little Row))}}
spec p(x:(Int,Int))->Int{x.0}
spec b()->Word[8]^2{hex"00 ff"}spec q()->Word[8]^2{"/*"}
test "known"{(1==1)&&(!(0!=0))}impl empty(){}
}"#,
        );
        assert!(output.contains("Mod[(1 << n) - 1]"));
        assert!(output.contains("matrix[0][0] as Int"));
        assert!(output.contains("missing::get[n](x)"));
        assert!(output.contains("x.0"));
        assert!(output.contains("[1, 2] ++ [3, 4]"));
    }

    #[test]
    fn errors_are_forwarded_without_formatted_text() {
        for (source, code) in [
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
            let output = result(source);
            assert!(output.has_errors());
            assert_eq!(output.formatted(), None);
            assert!(
                output
                    .diagnostics()
                    .iter()
                    .any(|diagnostic| diagnostic.code() == code)
            );
        }
    }

    #[test]
    fn formatting_needs_neither_imports_nor_semantic_acceptance() {
        let output = formatted(
            "edition 2026;module m{use absent;type X=Word[7];spec f()->Unknown{absent::g()}}",
        );
        assert!(output.contains("use absent;"));
        assert!(output.contains("Word[7]"));
        assert!(output.contains("absent::g()"));
    }

    #[test]
    fn exact_output_and_event_limits_fail_closed() {
        let source = "edition 2026;module m{spec f()->Int{1}}";
        let expected = formatted(source);
        let mut sources = SourceMap::new();
        let id = sources.add("limit.or", source).unwrap();
        let source = sources.get(id).unwrap();
        let accepted = format_with_limits(
            source,
            Edition::E2026,
            Limits {
                output: expected.len(),
                events: 2,
            },
        );
        assert_eq!(accepted.formatted(), Some(expected.as_str()));
        for limits in [
            Limits {
                output: expected.len() - 1,
                events: 2,
            },
            Limits {
                output: expected.len(),
                events: 1,
            },
        ] {
            let rejected = format_with_limits(source, Edition::E2026, limits);
            assert!(rejected.has_errors());
            assert_eq!(rejected.formatted(), None);
            assert_eq!(
                rejected.diagnostics()[0].code(),
                DiagnosticCode::FormatResourceLimit
            );
        }
    }

    #[test]
    fn preservation_checker_rejects_token_and_comment_changes() {
        let original = "edition 2026;module m{/*a*/spec x()->Int{0x01}}";
        for changed in [
            "edition 2026;module m{/*a*/spec x()->Int{1}}",
            "edition 2026;module m{spec x()->Int{/*a*/0x01}}",
            "edition 2026;module m{/*b*/spec x()->Int{0x01}}",
            "edition 2026;module m{spec x()->Int{0x01}}",
        ] {
            let mut sources = SourceMap::new();
            let left_id = sources.add("left", original).unwrap();
            let right_id = sources.add("right", changed).unwrap();
            let left = sources.get(left_id).unwrap();
            let right = sources.get(right_id).unwrap();
            assert_eq!(
                equivalent(
                    left,
                    lex(left, Edition::E2026).tokens(),
                    right,
                    lex(right, Edition::E2026).tokens()
                ),
                Err(Failure::Inconsistent)
            );
        }
    }

    #[test]
    fn validation_rejects_noncanonical_and_inconsistent_plans() {
        let text = "edition 2026;module m{}";
        let mut sources = SourceMap::new();
        let id = sources.add("invalid-plan.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let lexed = lex(source, Edition::E2026);
        let parsed = parse(source, &lexed);
        let plan = Plan::build(
            lexed.tokens(),
            parsed.ast().unwrap(),
            MAX_FORMAT_EVENTS_PER_SOURCE,
        )
        .unwrap();
        assert_eq!(
            validate(
                source,
                lexed.tokens(),
                text,
                Edition::E2026,
                &plan,
                MAX_FORMATTED_SOURCE_BYTES
            ),
            Err(Failure::Inconsistent)
        );
        assert_eq!(
            render(
                source,
                lexed.tokens(),
                &Plan { roles: Vec::new() },
                MAX_FORMATTED_SOURCE_BYTES
            ),
            Err(Failure::Inconsistent)
        );
    }

    #[test]
    fn comment_heavy_input_uses_streamed_trivia() {
        let source = format!(
            "edition 2026;module m{{{}spec f(){{}}}}",
            "/*x*/".repeat(100_000)
        );
        let output = formatted(&source);
        assert_eq!(output.matches("/*x*/").count(), 100_000);
    }

    #[test]
    fn longest_array_formats_with_bounded_heap_storage() {
        let elements = std::iter::repeat_n("0", MAX_ARRAY_ELEMENTS)
            .collect::<Vec<_>>()
            .join(",");
        let source = format!("edition 2026;module m{{spec f()->Word[8]^65536{{[{elements}]}}}}");
        let output = formatted(&source);
        assert!(output.contains("Word[8]^65536"));
        assert_eq!(output.matches(", ").count(), MAX_ARRAY_ELEMENTS - 1);
    }

    #[test]
    fn syntax_height_and_nested_comments_fit_one_mebibyte_stack() {
        std::thread::Builder::new()
            .stack_size(1024 * 1024)
            .spawn(|| {
                let expression = std::iter::repeat_n("x", MAX_EXPRESSION_HEIGHT)
                    .collect::<Vec<_>>()
                    .join("+");
                formatted(&format!(
                    "edition 2026;module m{{spec f(x:Int)->Int{{{expression}}}}}"
                ));
                let chained = format!("x{}", "[0]".repeat(MAX_EXPRESSION_HEIGHT - 1));
                formatted(&format!(
                    "edition 2026;module m{{spec f(x:Int)->Int{{{chained}}}}}"
                ));
                let comment = format!("{}inside{}", "/*".repeat(65_536), "*/".repeat(65_536));
                formatted(&format!("{comment}edition 2026;module m{{}}"));
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn evaluation_values_and_costs_survive_formatting() {
        fn values(text: &str) -> Vec<(String, usize)> {
            let mut sources = SourceMap::new();
            let id = sources.add("eval.or", text).unwrap();
            let source = sources.get(id).unwrap();
            let lexed = lex(source, Edition::E2026);
            let parsed = parse(source, &lexed);
            let analysis = analyze(source, parsed.ast().unwrap());
            assert!(!analysis.has_errors(), "{:?}", analysis.diagnostics());
            let result = evaluate(analysis.core().unwrap());
            assert!(!result.has_errors());
            result
                .values()
                .unwrap()
                .iter()
                .map(|value| (value.value().to_string(), value.steps()))
                .collect()
        }
        let source = "edition 2026;module m{type R=Word[8]^2;type M=R^2;spec matrix()->M{[[1,2],[3,4]]}spec calc()->Int{let x:Int=for i in 0..8 with s:Int=0{s+1};if (x==8){x*7}else{0}}spec residue[b in 4..6]()->Mod[(1<<b)-1]{1+2}}";
        assert_eq!(values(source), values(&formatted(source)));
    }

    #[test]
    fn repository_parsed_sources_are_complete_and_idempotent() {
        fn visit(path: &Path, count: &mut usize) {
            let mut entries = fs::read_dir(path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for path in entries {
                if path.is_dir() {
                    visit(&path, count);
                } else if path.extension().is_some_and(|extension| extension == "or") {
                    let text = fs::read_to_string(&path).unwrap();
                    let mut sources = SourceMap::new();
                    let id = sources.add("corpus.or", text.as_str()).unwrap();
                    let source = sources.get(id).unwrap();
                    let lexed = lex(source, Edition::E2026);
                    if !lexed.has_errors() && !parse(source, &lexed).has_errors() {
                        let result = format_source(source, Edition::E2026);
                        assert!(
                            !result.has_errors(),
                            "{}: {:?}",
                            path.display(),
                            result.diagnostics()
                        );
                        let first = result.into_formatted().unwrap();
                        assert_eq!(formatted(&first), first, "{}", path.display());
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
        eprintln!("formatted and revalidated {count} repository sources");
    }
}
