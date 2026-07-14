//! Source ownership, stable file identities, and byte-precise spans.

use std::ffi::OsStr;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SOURCE_MAP_ID: AtomicU64 = AtomicU64::new(1);

/// Maximum UTF-8 bytes accepted for one Orange source.
pub const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;

const COLUMN_CHECKPOINT_INTERVAL_BYTES: usize = 256;

/// An injective, ASCII-safe display encoding of an operating-system source name.
///
/// Normal UTF-8 library names should be passed directly to [`SourceMap::add`].
/// This wrapper lets command-line clients preserve non-UTF-8 path identity
/// without asking diagnostic rendering to interpret an already escaped string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderedSourceName(String);

impl RenderedSourceName {
    /// Encodes a UTF-8 source name with Rust's stable default character escapes.
    #[must_use]
    pub fn from_text(name: &str) -> Self {
        Self(name.chars().flat_map(char::escape_default).collect())
    }

    /// Encodes the target representation of an operating-system source name.
    ///
    /// Non-UTF-8 identity follows [`OsStr::as_encoded_bytes`] for the current
    /// target and pinned toolchain; it is not a cross-platform path encoding.
    #[must_use]
    pub fn from_os_str(name: &OsStr) -> Self {
        if let Some(name) = name.to_str() {
            return Self::from_text(name);
        }
        Self(
            name.as_encoded_bytes()
                .iter()
                .copied()
                .flat_map(std::ascii::escape_default)
                .map(char::from)
                .collect(),
        )
    }

    /// Returns the canonical ASCII-safe display representation.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RenderedSourceName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// A stable, insertion-ordered source identity within a [`SourceMap`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId {
    map: u64,
    index: u32,
}

impl SourceId {
    /// Returns the zero-based numeric identity.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.index
    }
}

/// A UTF-8 byte offset within a source file.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TextOffset(u32);

impl TextOffset {
    /// Creates an offset from its stable byte representation.
    #[must_use]
    pub const fn new(bytes: u32) -> Self {
        Self(bytes)
    }

    /// Returns the zero-based UTF-8 byte offset.
    #[must_use]
    pub const fn bytes(self) -> u32 {
        self.0
    }
}

/// A half-open UTF-8 byte range tied to exactly one source file.
///
/// Callers construct spans through [`SourceFile::span`], which validates source
/// bounds, ordering, and UTF-8 boundaries. The unchecked representation
/// constructor is intentionally not part of the public API.
///
/// ```compile_fail
/// use orange_compiler::{SourceMap, Span, TextOffset};
///
/// let mut sources = SourceMap::new();
/// let source = sources.add("example.or", "é").unwrap();
/// let _invalid = Span::new(source, TextOffset::new(0), TextOffset::new(1));
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    source: SourceId,
    start: TextOffset,
    end: TextOffset,
}

impl Span {
    #[must_use]
    const fn new(source: SourceId, start: TextOffset, end: TextOffset) -> Option<Self> {
        if start.0 <= end.0 {
            Some(Self { source, start, end })
        } else {
            None
        }
    }

    /// Returns the source containing this span.
    #[must_use]
    pub const fn source(self) -> SourceId {
        self.source
    }

    /// Returns the inclusive start offset.
    #[must_use]
    pub const fn start(self) -> TextOffset {
        self.start
    }

    /// Returns the exclusive end offset.
    #[must_use]
    pub const fn end(self) -> TextOffset {
        self.end
    }

    /// Returns the span length in UTF-8 bytes.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.end.0 - self.start.0
    }

    /// Returns whether this span has zero length.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start.0 == self.end.0
    }
}

/// A one-based source location intended for human-facing diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineColumn {
    /// One-based line number.
    pub line: u32,
    /// One-based Unicode-scalar column number.
    pub column: u32,
}

/// An owned UTF-8 source and its line index.
#[derive(Clone, Debug)]
pub struct SourceFile {
    id: SourceId,
    name: String,
    name_is_rendered: bool,
    text: String,
    line_starts: Vec<TextOffset>,
    column_checkpoints: Vec<ColumnCheckpoint>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ColumnCheckpoint {
    offset: TextOffset,
    column: u32,
}

fn logical_line_ending_count(bytes: &[u8]) -> Result<usize, SourceError> {
    let mut ending_count = 0_usize;
    let mut previous_was_cr = false;

    for byte in bytes {
        match *byte {
            b'\r' => {
                ending_count = ending_count
                    .checked_add(1)
                    .ok_or(SourceError::IndexAllocationFailed)?;
                previous_was_cr = true;
            }
            b'\n' => {
                if !previous_was_cr {
                    ending_count = ending_count
                        .checked_add(1)
                        .ok_or(SourceError::IndexAllocationFailed)?;
                }
                previous_was_cr = false;
            }
            _ => previous_was_cr = false,
        }
    }

    Ok(ending_count)
}

impl SourceFile {
    fn new(
        id: SourceId,
        name: String,
        name_is_rendered: bool,
        text: String,
    ) -> Result<Self, SourceError> {
        if text.len() > MAX_SOURCE_BYTES {
            return Err(SourceError::TooLarge);
        }
        let byte_len = u32::try_from(text.len()).map_err(|_| SourceError::TooLarge)?;
        let line_count = logical_line_ending_count(text.as_bytes())?
            .checked_add(1)
            .ok_or(SourceError::IndexAllocationFailed)?;
        let mut line_starts = Vec::new();
        line_starts
            .try_reserve_exact(line_count)
            .map_err(|_| SourceError::IndexAllocationFailed)?;
        line_starts.push(TextOffset::new(0));
        let mut column_checkpoints = Vec::new();
        let mut line_start = 0_usize;
        let mut last_checkpoint = 0_usize;
        let mut column = 1_u32;

        let mut index = 0_usize;
        while index < text.len() {
            if index > line_start
                && index.saturating_sub(last_checkpoint) >= COLUMN_CHECKPOINT_INTERVAL_BYTES
            {
                column_checkpoints.push(ColumnCheckpoint {
                    offset: TextOffset::new(
                        u32::try_from(index).map_err(|_| SourceError::TooLarge)?,
                    ),
                    column,
                });
                last_checkpoint = index;
            }

            let byte = text.as_bytes()[index];
            if byte == b'\r' || byte == b'\n' {
                // CRLF is one logical ending; bare CR and LF are endings too.
                let ending_width = if byte == b'\r'
                    && text.as_bytes().get(index.saturating_add(1)) == Some(&b'\n')
                {
                    2
                } else {
                    1
                };
                let next = index
                    .checked_add(ending_width)
                    .ok_or(SourceError::TooLarge)?;
                line_starts.push(TextOffset::new(
                    u32::try_from(next).map_err(|_| SourceError::TooLarge)?,
                ));
                line_start = next;
                last_checkpoint = next;
                column = 1;
                index = next;
            } else {
                let character = text[index..].chars().next().ok_or(SourceError::TooLarge)?;
                column = column.checked_add(1).ok_or(SourceError::TooLarge)?;
                index += character.len_utf8();
            }
        }

        debug_assert!(
            line_starts
                .last()
                .is_some_and(|offset| offset.0 <= byte_len)
        );
        debug_assert_eq!(line_starts.len(), line_count);
        Ok(Self {
            id,
            name,
            name_is_rendered,
            text,
            line_starts,
            column_checkpoints,
        })
    }

    /// Returns this source's stable identity.
    #[must_use]
    pub const fn id(&self) -> SourceId {
        self.id
    }

    /// Returns the caller-provided name representation.
    ///
    /// [`SourceMap::add`] preserves the raw UTF-8 name. A source inserted with
    /// [`SourceMap::add_with_rendered_name`] instead returns its canonical
    /// ASCII-safe representation here.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) const fn name_is_rendered(&self) -> bool {
        self.name_is_rendered
    }

    /// Returns the complete UTF-8 source text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the source length in UTF-8 bytes.
    #[must_use]
    pub fn byte_len(&self) -> TextOffset {
        // Construction rejects sources that do not fit in u32.
        TextOffset::new(u32::try_from(self.text.len()).unwrap_or(u32::MAX))
    }

    /// Creates a span after checking bounds, ordering, and UTF-8 boundaries.
    #[must_use]
    pub fn span(&self, start: TextOffset, end: TextOffset) -> Option<Span> {
        let start_usize = usize::try_from(start.0).ok()?;
        let end_usize = usize::try_from(end.0).ok()?;
        if start.0 <= end.0
            && end_usize <= self.text.len()
            && self.text.is_char_boundary(start_usize)
            && self.text.is_char_boundary(end_usize)
        {
            Span::new(self.id, start, end)
        } else {
            None
        }
    }

    /// Returns the text covered by a valid span from this source.
    #[must_use]
    pub fn slice(&self, span: Span) -> Option<&str> {
        if span.source != self.id {
            return None;
        }
        let start = usize::try_from(span.start.0).ok()?;
        let end = usize::try_from(span.end.0).ok()?;
        self.text.get(start..end)
    }

    /// Converts a valid UTF-8 byte offset to a one-based line and column.
    #[must_use]
    pub fn line_column(&self, offset: TextOffset) -> Option<LineColumn> {
        let offset_usize = usize::try_from(offset.0).ok()?;
        if offset_usize > self.text.len() || !self.text.is_char_boundary(offset_usize) {
            return None;
        }

        let line_index = match self
            .line_starts
            .binary_search_by_key(&offset.0, |candidate| candidate.0)
        {
            Ok(index) => index,
            Err(next_index) => next_index.saturating_sub(1),
        };
        let line_start = self.line_starts[line_index];
        let checkpoint_count = self
            .column_checkpoints
            .partition_point(|checkpoint| checkpoint.offset.0 <= offset.0);
        let checkpoint = checkpoint_count
            .checked_sub(1)
            .and_then(|index| self.column_checkpoints.get(index))
            .filter(|checkpoint| checkpoint.offset.0 >= line_start.0);
        let (scan_start, base_column) = checkpoint.map_or((line_start, 1_u32), |checkpoint| {
            (checkpoint.offset, checkpoint.column)
        });
        let scan_start = usize::try_from(scan_start.0).ok()?;
        let scalar_delta = u32::try_from(
            self.text
                .get(scan_start..offset_usize)?
                .chars()
                .filter(|character| !matches!(character, '\r' | '\n'))
                .count(),
        )
        .ok()?;

        Some(LineColumn {
            line: u32::try_from(line_index + 1).ok()?,
            column: base_column.checked_add(scalar_delta)?,
        })
    }

    /// Returns one line without its trailing newline or carriage return.
    #[must_use]
    pub fn line_text(&self, one_based_line: u32) -> Option<&str> {
        let line_index = usize::try_from(one_based_line.checked_sub(1)?).ok()?;
        let start = usize::try_from(self.line_starts.get(line_index)?.0).ok()?;
        let raw_end = self
            .line_starts
            .get(line_index + 1)
            .map_or(self.text.len(), |offset| {
                usize::try_from(offset.0).unwrap_or(self.text.len())
            });
        let mut end = raw_end;
        if end > start && self.text.as_bytes().get(end - 1) == Some(&b'\n') {
            end -= 1;
        }
        if end > start && self.text.as_bytes().get(end - 1) == Some(&b'\r') {
            end -= 1;
        }
        self.text.get(start..end)
    }

    pub(crate) fn line_start(&self, one_based_line: u32) -> Option<TextOffset> {
        let line_index = usize::try_from(one_based_line.checked_sub(1)?).ok()?;
        self.line_starts.get(line_index).copied()
    }

    pub(crate) fn lexer_span(&self, start: usize, end: usize) -> Span {
        debug_assert!(start <= end);
        debug_assert!(end <= self.text.len());
        debug_assert!(self.text.is_char_boundary(start));
        debug_assert!(self.text.is_char_boundary(end));

        // The source constructor established the representational bound.
        let start = TextOffset::new(u32::try_from(start).unwrap_or(u32::MAX));
        let end = TextOffset::new(u32::try_from(end).unwrap_or(u32::MAX));
        Span {
            source: self.id,
            start,
            end,
        }
    }
}

/// An insertion-ordered collection of owned source files.
#[derive(Debug)]
pub struct SourceMap {
    identity: u64,
    files: Vec<SourceFile>,
}

impl SourceMap {
    /// Creates an empty source map.
    #[must_use]
    pub fn new() -> Self {
        let identity = NEXT_SOURCE_MAP_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(identity, 0, "source-map identity space exhausted");
        Self {
            identity,
            files: Vec::new(),
        }
    }

    /// Adds a UTF-8 source and returns its stable insertion-ordered identity.
    pub fn add(
        &mut self,
        name: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<SourceId, SourceError> {
        self.add_inner(name.into(), false, text.into())
    }

    /// Adds a source with an already canonical [`RenderedSourceName`].
    ///
    /// Diagnostics preserve this representation exactly instead of escaping it
    /// a second time.
    pub fn add_with_rendered_name(
        &mut self,
        name: RenderedSourceName,
        text: impl Into<String>,
    ) -> Result<SourceId, SourceError> {
        self.add_inner(name.0, true, text.into())
    }

    fn add_inner(
        &mut self,
        name: String,
        name_is_rendered: bool,
        text: String,
    ) -> Result<SourceId, SourceError> {
        let index = u32::try_from(self.files.len()).map_err(|_| SourceError::TooManyFiles)?;
        let id = SourceId {
            map: self.identity,
            index,
        };
        let file = SourceFile::new(id, name, name_is_rendered, text)?;
        self.files.push(file);
        Ok(id)
    }

    /// Looks up a source by identity.
    #[must_use]
    pub fn get(&self, id: SourceId) -> Option<&SourceFile> {
        if id.map != self.identity {
            return None;
        }
        self.files.get(usize::try_from(id.index).ok()?)
    }

    /// Returns the number of sources in insertion order.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns whether no sources have been inserted.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Iterates over sources in stable insertion order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &SourceFile> {
        self.files.iter()
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

/// A source could not be represented in a source map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceError {
    /// A single source exceeds [`MAX_SOURCE_BYTES`].
    TooLarge,
    /// A source map already contains `u32::MAX` files.
    TooManyFiles,
    /// Memory for a source's derived indexing data could not be reserved.
    IndexAllocationFailed,
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => formatter.write_str("source exceeds the 16 MiB input limit"),
            Self::TooManyFiles => {
                formatter.write_str("source map exceeds the file representation limit")
            }
            Self::IndexAllocationFailed => {
                formatter.write_str("could not allocate source indexing data")
            }
        }
    }
}

impl std::error::Error for SourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendered_source_names_distinguish_text_from_literal_escapes() {
        let unicode = RenderedSourceName::from_text("é.or");
        let literal = RenderedSourceName::from_text(r"\u{e9}.or");

        assert_eq!(unicode.as_str(), r"\u{e9}.or");
        assert_eq!(literal.as_str(), r"\\u{e9}.or");
        assert_ne!(unicode, literal);
    }

    #[test]
    fn source_ids_follow_insertion_order() {
        let mut sources = SourceMap::new();
        let first = sources.add("first.or", "one").unwrap();
        let second = sources.add("second.or", "two").unwrap();

        assert_eq!(first.index(), 0);
        assert_eq!(second.index(), 1);
        assert_eq!(sources.get(second).unwrap().name(), "second.or");
    }

    #[test]
    fn maps_utf8_offsets_across_crlf_lines() {
        let mut sources = SourceMap::new();
        let id = sources.add("utf8.or", "aé\r\nβ\n").unwrap();
        let source = sources.get(id).unwrap();

        assert_eq!(
            source.line_column(TextOffset::new(3)),
            Some(LineColumn { line: 1, column: 3 })
        );
        assert_eq!(
            source.line_column(TextOffset::new(4)),
            Some(LineColumn { line: 1, column: 3 })
        );
        assert_eq!(
            source.line_column(TextOffset::new(5)),
            Some(LineColumn { line: 2, column: 1 })
        );
        assert_eq!(source.line_text(1), Some("aé"));
        assert_eq!(source.line_text(2), Some("β"));
        assert_eq!(source.line_text(3), Some(""));
    }

    #[test]
    fn treats_lf_crlf_and_bare_cr_as_logical_line_endings() {
        let mut sources = SourceMap::new();
        let id = sources.add("endings.or", "a\rb\r\nc\nd").unwrap();
        let source = sources.get(id).unwrap();

        assert_eq!(source.line_starts.len(), 4);
        assert_eq!(source.line_starts.capacity(), 4);
        assert_eq!(source.line_text(1), Some("a"));
        assert_eq!(source.line_text(2), Some("b"));
        assert_eq!(source.line_text(3), Some("c"));
        assert_eq!(source.line_text(4), Some("d"));
        assert_eq!(
            source.line_column(TextOffset::new(2)),
            Some(LineColumn { line: 2, column: 1 })
        );
        assert_eq!(
            source.line_column(TextOffset::new(5)),
            Some(LineColumn { line: 3, column: 1 })
        );
        assert_eq!(
            source.line_column(TextOffset::new(7)),
            Some(LineColumn { line: 4, column: 1 })
        );
    }

    #[test]
    fn reserves_the_finished_line_index_just_above_a_power_of_two() {
        const ENDING_COUNT: usize = 1 << 16;

        let mut sources = SourceMap::new();
        let id = sources
            .add("newline-heavy.or", "\n".repeat(ENDING_COUNT))
            .unwrap();
        let source = sources.get(id).unwrap();
        let expected_line_count = ENDING_COUNT + 1;

        assert_eq!(source.line_starts.len(), expected_line_count);
        assert_eq!(source.line_starts.capacity(), expected_line_count);
        assert_eq!(
            source.line_column(TextOffset::new(
                u32::try_from(ENDING_COUNT).expect("test input fits in a text offset")
            )),
            Some(LineColumn {
                line: u32::try_from(expected_line_count).expect("test line count fits in u32"),
                column: 1,
            })
        );
        assert_eq!(
            source.line_text(
                u32::try_from(expected_line_count).expect("test line count fits in u32")
            ),
            Some("")
        );
    }

    #[test]
    fn checkpoints_preserve_unicode_columns_across_crlf_lines() {
        let mut text = "a".repeat(255);
        text.push('é');
        text.push('β');
        text.push_str("\r\n");
        text.push_str(&"b".repeat(256));
        text.push('é');

        let mut sources = SourceMap::new();
        let id = sources.add("checkpoint.or", text).unwrap();
        let source = sources.get(id).unwrap();

        assert_eq!(
            source.line_column(TextOffset::new(257)),
            Some(LineColumn {
                line: 1,
                column: 257,
            })
        );
        assert_eq!(
            source.line_column(TextOffset::new(259)),
            Some(LineColumn {
                line: 1,
                column: 258,
            })
        );
        assert_eq!(
            source.line_column(TextOffset::new(261)),
            Some(LineColumn { line: 2, column: 1 })
        );
        assert_eq!(
            source.line_column(TextOffset::new(517)),
            Some(LineColumn {
                line: 2,
                column: 257,
            })
        );
        assert_eq!(
            source.line_column(TextOffset::new(519)),
            Some(LineColumn {
                line: 2,
                column: 258,
            })
        );
        assert_eq!(source.line_text(1).unwrap().chars().last(), Some('β'));
        assert_eq!(source.line_text(2).unwrap().chars().last(), Some('é'));
    }

    #[test]
    fn long_lines_use_sparse_bounded_column_checkpoints() {
        let text = "a".repeat(1024 * 1024);
        let mut sources = SourceMap::new();
        let id = sources.add("long.or", text).unwrap();
        let source = sources.get(id).unwrap();
        let end = source.byte_len();

        for _ in 0..100 {
            assert_eq!(
                source.line_column(end),
                Some(LineColumn {
                    line: 1,
                    column: end.bytes() + 1,
                })
            );
        }

        let checkpoint = source.column_checkpoints.last().unwrap();
        assert!(
            end.bytes().saturating_sub(checkpoint.offset.bytes())
                <= u32::try_from(COLUMN_CHECKPOINT_INTERVAL_BYTES).unwrap()
        );
        assert!(
            source.column_checkpoints.len()
                <= source.text().len() / COLUMN_CHECKPOINT_INTERVAL_BYTES
        );
    }

    #[test]
    fn enforces_the_public_source_byte_limit() {
        let mut sources = SourceMap::new();
        let accepted = sources
            .add("maximum.or", "a".repeat(MAX_SOURCE_BYTES))
            .unwrap();
        assert_eq!(
            sources.get(accepted).unwrap().byte_len().bytes(),
            u32::try_from(MAX_SOURCE_BYTES).unwrap()
        );

        assert_eq!(
            sources.add("oversized.or", "a".repeat(MAX_SOURCE_BYTES + 1)),
            Err(SourceError::TooLarge)
        );
        assert_eq!(
            SourceError::TooLarge.to_string(),
            "source exceeds the 16 MiB input limit"
        );
    }

    #[test]
    fn rejects_spans_that_split_utf8_or_cross_sources() {
        let mut sources = SourceMap::new();
        let first = sources.add("first.or", "é").unwrap();
        let second = sources.add("second.or", "ok").unwrap();
        let source = sources.get(first).unwrap();

        assert!(
            source
                .span(TextOffset::new(0), TextOffset::new(1))
                .is_none()
        );
        let foreign = sources
            .get(second)
            .unwrap()
            .span(TextOffset::new(0), TextOffset::new(1))
            .unwrap();
        assert_eq!(source.slice(foreign), None);
    }

    #[test]
    fn source_ids_cannot_alias_across_source_maps() {
        let mut first_map = SourceMap::new();
        let first_id = first_map.add("same-index.or", "first").unwrap();
        let first_span = first_map
            .get(first_id)
            .unwrap()
            .span(TextOffset::new(0), TextOffset::new(5))
            .unwrap();

        let mut second_map = SourceMap::new();
        let second_id = second_map.add("same-index.or", "other").unwrap();
        let second_source = second_map.get(second_id).unwrap();

        assert_eq!(first_id.index(), second_id.index());
        assert_ne!(first_id, second_id);
        assert!(second_map.get(first_id).is_none());
        assert_eq!(second_source.slice(first_span), None);
    }
}
