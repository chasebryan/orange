//! Source ownership, stable file identities, and byte-precise spans.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SOURCE_MAP_ID: AtomicU64 = AtomicU64::new(1);

/// Maximum UTF-8 bytes accepted for one Orange source.
pub const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;

const COLUMN_CHECKPOINT_INTERVAL_BYTES: usize = 256;

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
    text: String,
    line_starts: Vec<TextOffset>,
    column_checkpoints: Vec<ColumnCheckpoint>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ColumnCheckpoint {
    offset: TextOffset,
    column: u32,
}

impl SourceFile {
    fn new(id: SourceId, name: String, text: String) -> Result<Self, SourceError> {
        if text.len() > MAX_SOURCE_BYTES {
            return Err(SourceError::TooLarge);
        }
        let byte_len = u32::try_from(text.len()).map_err(|_| SourceError::TooLarge)?;
        let mut line_starts = vec![TextOffset::new(0)];
        let mut column_checkpoints = Vec::new();
        let mut line_start = 0_usize;
        let mut last_checkpoint = 0_usize;
        let mut column = 1_u32;

        for (index, character) in text.char_indices() {
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

            if character == '\n' {
                // `byte_len` proved that every index in the source fits in u32.
                let next = index + character.len_utf8();
                line_starts.push(TextOffset::new(
                    u32::try_from(next).map_err(|_| SourceError::TooLarge)?,
                ));
                line_start = next;
                last_checkpoint = next;
                column = 1;
            } else {
                column = column.checked_add(1).ok_or(SourceError::TooLarge)?;
            }
        }

        debug_assert!(
            line_starts
                .last()
                .is_some_and(|offset| offset.0 <= byte_len)
        );
        Ok(Self {
            id,
            name,
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

    /// Returns the caller-provided display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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
        let scalar_delta =
            u32::try_from(self.text.get(scan_start..offset_usize)?.chars().count()).ok()?;

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
        let index = u32::try_from(self.files.len()).map_err(|_| SourceError::TooManyFiles)?;
        let id = SourceId {
            map: self.identity,
            index,
        };
        let file = SourceFile::new(id, name.into(), text.into())?;
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

/// A representational source-map limit was exceeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceError {
    /// A single source exceeds [`MAX_SOURCE_BYTES`].
    TooLarge,
    /// A source map already contains `u32::MAX` files.
    TooManyFiles,
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => formatter.write_str("source exceeds the 16 MiB input limit"),
            Self::TooManyFiles => {
                formatter.write_str("source map exceeds the file representation limit")
            }
        }
    }
}

impl std::error::Error for SourceError {}

#[cfg(test)]
mod tests {
    use super::*;

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
            source.line_column(TextOffset::new(5)),
            Some(LineColumn { line: 2, column: 1 })
        );
        assert_eq!(source.line_text(1), Some("aé"));
        assert_eq!(source.line_text(2), Some("β"));
        assert_eq!(source.line_text(3), Some(""));
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
