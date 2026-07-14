//! Source ownership, stable file identities, and byte-precise spans.

use std::borrow::Cow;
use std::ffi::OsStr;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_SOURCE_MAP_ID: AtomicU64 = AtomicU64::new(1);

/// Maximum UTF-8 bytes accepted for one Orange source.
pub const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;

const COLUMN_CHECKPOINT_INTERVAL_BYTES: usize = 256;

fn allocate_source_map_identity(counter: &AtomicU64) -> Option<u64> {
    counter
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |identity| {
            (identity != 0).then(|| identity.wrapping_add(1))
        })
        .ok()
}

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

    /// Fallibly encodes a UTF-8 source name with stable default character escapes.
    ///
    /// # Errors
    ///
    /// Returns [`SourceError::SourceAllocationFailed`] when the complete encoded
    /// representation cannot be reserved before any characters are copied.
    pub fn try_from_text(name: &str) -> Result<Self, SourceError> {
        Self::try_from_characters_with_reservation(name.chars(), |rendered, bytes| {
            rendered.try_reserve_exact(bytes).is_ok()
        })
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

    /// Fallibly encodes the target representation of an operating-system name.
    ///
    /// Non-UTF-8 identity follows [`OsStr::as_encoded_bytes`] for the current
    /// target and pinned toolchain, matching [`Self::from_os_str`].
    ///
    /// # Errors
    ///
    /// Returns [`SourceError::SourceAllocationFailed`] when the complete encoded
    /// representation cannot be reserved before any characters are copied.
    pub fn try_from_os_str(name: &OsStr) -> Result<Self, SourceError> {
        if let Some(name) = name.to_str() {
            return Self::try_from_text(name);
        }
        Self::try_from_encoded_characters_with_reservation(
            name.as_encoded_bytes()
                .iter()
                .copied()
                .flat_map(std::ascii::escape_default)
                .map(char::from),
            |rendered, bytes| rendered.try_reserve_exact(bytes).is_ok(),
        )
    }

    fn try_from_characters_with_reservation(
        characters: impl Clone + Iterator<Item = char>,
        mut reserve: impl FnMut(&mut String, usize) -> bool,
    ) -> Result<Self, SourceError> {
        let encoded_bytes = characters
            .clone()
            .try_fold(0_usize, |bytes, character| {
                bytes.checked_add(character.escape_default().count())
            })
            .ok_or(SourceError::SourceAllocationFailed)?;
        let mut rendered = String::new();
        if !reserve(&mut rendered, encoded_bytes) {
            return Err(SourceError::SourceAllocationFailed);
        }
        rendered.extend(characters.flat_map(char::escape_default));
        Ok(Self(rendered))
    }

    fn try_from_encoded_characters_with_reservation(
        characters: impl Clone + Iterator<Item = char>,
        mut reserve: impl FnMut(&mut String, usize) -> bool,
    ) -> Result<Self, SourceError> {
        let encoded_bytes = characters
            .clone()
            .try_fold(0_usize, |bytes, character| {
                bytes.checked_add(character.len_utf8())
            })
            .ok_or(SourceError::SourceAllocationFailed)?;
        let mut rendered = String::new();
        if !reserve(&mut rendered, encoded_bytes) {
            return Err(SourceError::SourceAllocationFailed);
        }
        rendered.extend(characters);
        Ok(Self(rendered))
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
///
/// Ordering follows insertion order for IDs owned by the same map. Ordering
/// between different maps is process-local and must not be persisted.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SourceId {
    map: u64,
    index: u32,
}

impl SourceId {
    /// Returns the zero-based insertion index within the owning source map.
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
        self.end.0.saturating_sub(self.start.0)
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
        Self::new_with_checkpoint_reservation(id, name, name_is_rendered, text, |checkpoints| {
            checkpoints.try_reserve(1).is_ok()
        })
    }

    fn new_with_checkpoint_reservation(
        id: SourceId,
        name: String,
        name_is_rendered: bool,
        text: String,
        mut reserve_checkpoint: impl FnMut(&mut Vec<ColumnCheckpoint>) -> bool,
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
                if !reserve_checkpoint(&mut column_checkpoints) {
                    return Err(SourceError::IndexAllocationFailed);
                }
                column_checkpoints.push(ColumnCheckpoint {
                    offset: TextOffset::new(
                        u32::try_from(index).map_err(|_| SourceError::TooLarge)?,
                    ),
                    column,
                });
                last_checkpoint = index;
            }

            let Some(&byte) = text.as_bytes().get(index) else {
                return Err(SourceError::TooLarge);
            };
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
                let character = text
                    .get(index..)
                    .and_then(|remaining| remaining.chars().next())
                    .ok_or(SourceError::TooLarge)?;
                column = column.checked_add(1).ok_or(SourceError::TooLarge)?;
                index = index
                    .checked_add(character.len_utf8())
                    .ok_or(SourceError::TooLarge)?;
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
    ///
    /// Columns count Unicode scalar values, not bytes. Line-ending bytes are
    /// column-neutral: an offset on either byte of CRLF has the end column of
    /// the preceding logical line, while the offset immediately after the
    /// ending is column one of the next line. A trailing line ending therefore
    /// creates an addressable empty final line at end of file.
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
        let line_start = *self.line_starts.get(line_index)?;
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
            line: u32::try_from(line_index.checked_add(1)?).ok()?,
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
            .get(line_index.checked_add(1)?)
            .map_or(self.text.len(), |offset| {
                usize::try_from(offset.0).unwrap_or(self.text.len())
            });
        let mut end = raw_end;
        if end > start {
            let previous = end.checked_sub(1)?;
            if self.text.as_bytes().get(previous) == Some(&b'\n') {
                end = previous;
            }
        }
        if end > start {
            let previous = end.checked_sub(1)?;
            if self.text.as_bytes().get(previous) == Some(&b'\r') {
                end = previous;
            }
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

fn source_index(file_count: usize) -> Result<u32, SourceError> {
    u32::try_from(file_count).map_err(|_| SourceError::TooManyFiles)
}

impl SourceMap {
    /// Creates an empty source map.
    ///
    /// # Panics
    ///
    /// Panics after all nonzero 64-bit source-map identities have been issued.
    /// Exhaustion is sticky, so catching the panic cannot make a later map
    /// alias an earlier identity.
    #[must_use]
    #[allow(
        clippy::expect_used,
        reason = "this documented convenience constructor panics only after u64 identity exhaustion"
    )]
    pub fn new() -> Self {
        Self::try_new().expect("source-map identity space exhausted")
    }

    /// Tries to create an empty source map without aliasing an earlier map.
    ///
    /// Identity exhaustion is sticky: after this returns
    /// [`SourceError::IdentitySpaceExhausted`], every later construction
    /// attempt returns the same error instead of wrapping to a prior identity.
    ///
    /// # Errors
    ///
    /// Returns [`SourceError::IdentitySpaceExhausted`] after every nonzero
    /// 64-bit source-map identity has been issued.
    pub fn try_new() -> Result<Self, SourceError> {
        Self::try_new_with_identity_allocator(&NEXT_SOURCE_MAP_ID)
    }

    fn try_new_with_identity_allocator(counter: &AtomicU64) -> Result<Self, SourceError> {
        let identity =
            allocate_source_map_identity(counter).ok_or(SourceError::IdentitySpaceExhausted)?;
        Ok(Self {
            identity,
            files: Vec::new(),
        })
    }

    /// Adds a UTF-8 source and returns its stable insertion-ordered identity.
    ///
    /// # Errors
    ///
    /// Returns [`SourceError::TooLarge`] when `text` exceeds
    /// [`MAX_SOURCE_BYTES`], [`SourceError::TooManyFiles`] when every `u32`
    /// source index is occupied, [`SourceError::SourceAllocationFailed`] when a
    /// borrowed name or source cannot be copied, or
    /// [`SourceError::IndexAllocationFailed`] when source-map or source-derived
    /// indexing storage cannot be reserved.
    pub fn add<'name, 'text>(
        &mut self,
        name: impl Into<Cow<'name, str>>,
        text: impl Into<Cow<'text, str>>,
    ) -> Result<SourceId, SourceError> {
        self.add_inner(name.into(), false, text.into())
    }

    /// Adds a source with an already canonical [`RenderedSourceName`].
    ///
    /// Diagnostics preserve this representation exactly instead of escaping it
    /// a second time.
    ///
    /// # Errors
    ///
    /// Returns [`SourceError::TooLarge`] when `text` exceeds
    /// [`MAX_SOURCE_BYTES`], [`SourceError::TooManyFiles`] when every `u32`
    /// source index is occupied, [`SourceError::SourceAllocationFailed`] when a
    /// borrowed source cannot be copied, or [`SourceError::IndexAllocationFailed`]
    /// when source-map or source-derived indexing storage cannot be reserved.
    pub fn add_with_rendered_name<'text>(
        &mut self,
        name: RenderedSourceName,
        text: impl Into<Cow<'text, str>>,
    ) -> Result<SourceId, SourceError> {
        self.add_inner(Cow::Owned(name.0), true, text.into())
    }

    fn add_inner<'name, 'text>(
        &mut self,
        name: Cow<'name, str>,
        name_is_rendered: bool,
        text: Cow<'text, str>,
    ) -> Result<SourceId, SourceError> {
        self.add_inner_with_string_reservation(name, name_is_rendered, text, |string, bytes| {
            string.try_reserve_exact(bytes).is_ok()
        })
    }

    fn add_inner_with_string_reservation<'name, 'text>(
        &mut self,
        name: Cow<'name, str>,
        name_is_rendered: bool,
        text: Cow<'text, str>,
        mut reserve_string: impl FnMut(&mut String, usize) -> bool,
    ) -> Result<SourceId, SourceError> {
        if text.len() > MAX_SOURCE_BYTES {
            return Err(SourceError::TooLarge);
        }
        let index = source_index(self.files.len())?;
        let id = SourceId {
            map: self.identity,
            index,
        };
        self.files
            .try_reserve(1)
            .map_err(|_| SourceError::IndexAllocationFailed)?;
        let name = own_source_string(name, &mut reserve_string)?;
        let text = own_source_string(text, &mut reserve_string)?;
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
    pub const fn len(&self) -> usize {
        self.files.len()
    }

    /// Returns whether no sources have been inserted.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Iterates over sources in stable insertion order.
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &SourceFile> {
        self.files.iter()
    }
}

fn own_source_string(
    value: Cow<'_, str>,
    reserve_string: &mut impl FnMut(&mut String, usize) -> bool,
) -> Result<String, SourceError> {
    match value {
        Cow::Owned(value) => Ok(value),
        Cow::Borrowed(value) => {
            let mut owned = String::new();
            if !reserve_string(&mut owned, value.len()) {
                return Err(SourceError::SourceAllocationFailed);
            }
            owned.push_str(value);
            Ok(owned)
        }
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

macro_rules! define_source_errors {
    ($(#[$variant_doc:meta] $variant:ident => $message:literal,)+) => {
        /// A source could not be represented in a source map.
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum SourceError {
            $(#[$variant_doc] $variant,)+
        }

        impl SourceError {
            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl fmt::Display for SourceError {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(match self {
                    $(Self::$variant => $message,)+
                })
            }
        }
    };
}

define_source_errors! {
    /// Every nonzero 64-bit source-map identity has already been issued.
    IdentitySpaceExhausted => "source-map identity space is exhausted",
    /// A single source exceeds [`MAX_SOURCE_BYTES`].
    TooLarge => "source exceeds the 16 MiB input limit",
    /// Every source index representable by `u32` is already occupied.
    TooManyFiles => "source map exceeds the file representation limit",
    /// Memory for an owned source name or source text could not be reserved.
    SourceAllocationFailed => "could not allocate owned source data",
    /// Memory for a source-map slot or source-derived indexing data could not be reserved.
    IndexAllocationFailed => "could not allocate source indexing data",
}

impl std::error::Error for SourceError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_error_inventory_and_messages_are_exact() {
        assert_eq!(
            SourceError::ALL,
            &[
                SourceError::IdentitySpaceExhausted,
                SourceError::TooLarge,
                SourceError::TooManyFiles,
                SourceError::SourceAllocationFailed,
                SourceError::IndexAllocationFailed,
            ]
        );
        assert_eq!(
            SourceError::ALL
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            [
                "source-map identity space is exhausted",
                "source exceeds the 16 MiB input limit",
                "source map exceeds the file representation limit",
                "could not allocate owned source data",
                "could not allocate source indexing data",
            ]
        );
    }

    #[test]
    fn source_map_identity_exhaustion_is_sticky_and_cannot_alias() {
        let counter = AtomicU64::new(u64::MAX);

        assert!(SourceMap::try_new_with_identity_allocator(&counter).is_ok());
        assert_eq!(counter.load(Ordering::Relaxed), 0);
        assert!(matches!(
            SourceMap::try_new_with_identity_allocator(&counter),
            Err(SourceError::IdentitySpaceExhausted)
        ));
        assert!(matches!(
            SourceMap::try_new_with_identity_allocator(&counter),
            Err(SourceError::IdentitySpaceExhausted)
        ));
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn concurrent_identity_exhaustion_issues_the_final_identity_exactly_once() {
        const THREADS: usize = 16;

        let counter = AtomicU64::new(u64::MAX);
        let barrier = std::sync::Barrier::new(THREADS);
        let outcomes = std::thread::scope(|scope| {
            let handles = (0..THREADS)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        SourceMap::try_new_with_identity_allocator(&counter)
                    })
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });

        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, Err(SourceError::IdentitySpaceExhausted)))
                .count(),
            THREADS - 1
        );
        assert_eq!(counter.load(Ordering::Relaxed), 0);
        assert!(matches!(
            SourceMap::try_new_with_identity_allocator(&counter),
            Err(SourceError::IdentitySpaceExhausted)
        ));
    }

    #[test]
    fn source_map_identities_do_not_alias_under_concurrent_construction() {
        const THREADS: usize = 8;
        const MAPS_PER_THREAD: usize = 64;

        let handles = (0..THREADS)
            .map(|_| {
                std::thread::spawn(|| {
                    (0..MAPS_PER_THREAD)
                        .map(|_| {
                            let mut sources = SourceMap::new();
                            sources.add("concurrent.or", "").unwrap()
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();

        let mut ids = handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), THREADS * MAPS_PER_THREAD);
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), THREADS * MAPS_PER_THREAD);
        assert!(ids.iter().all(|id| id.index() == 0));
    }

    #[test]
    fn rendered_source_names_distinguish_text_from_literal_escapes() {
        let unicode = RenderedSourceName::from_text("é.or");
        let literal = RenderedSourceName::from_text(r"\u{e9}.or");

        assert_eq!(unicode.as_str(), r"\u{e9}.or");
        assert_eq!(literal.as_str(), r"\\u{e9}.or");
        assert_ne!(unicode, literal);
    }

    #[test]
    fn fallible_text_name_encoding_matches_the_full_byte_scalar_corpus() {
        let mut name = (u8::MIN..=u8::MAX).map(char::from).collect::<String>();
        name.push('\u{202e}');
        name.push('🟠');

        let infallible = RenderedSourceName::from_text(&name);
        let fallible = RenderedSourceName::try_from_text(&name).unwrap();

        assert_eq!(fallible, infallible);
        assert!(fallible.as_str().is_ascii());
        assert!(
            !fallible
                .as_str()
                .bytes()
                .any(|byte| byte.is_ascii_control())
        );
    }

    #[cfg(unix)]
    #[test]
    fn invalid_single_unix_bytes_have_unique_noncolliding_names() {
        use std::os::unix::ffi::OsStringExt as _;

        let mut observed = std::collections::BTreeSet::new();
        for byte in 0x80_u8..=u8::MAX {
            let name = std::ffi::OsString::from_vec(vec![byte]);
            assert!(name.to_str().is_none());
            let infallible = RenderedSourceName::from_os_str(&name);
            let fallible = RenderedSourceName::try_from_os_str(&name).unwrap();
            let expected = format!(r"\x{byte:02x}");

            assert_eq!(fallible, infallible);
            assert_eq!(fallible.as_str(), expected);
            assert_ne!(
                fallible,
                RenderedSourceName::try_from_text(&expected).unwrap()
            );
            assert!(observed.insert(expected));
        }
        assert_eq!(observed.len(), 128);
    }

    #[test]
    fn fallible_rendered_source_names_reserve_the_complete_encoding_once() {
        let requested = std::cell::Cell::new(None);
        let encoded = RenderedSourceName::try_from_characters_with_reservation(
            "é.or".chars(),
            |rendered, bytes| {
                requested.set(Some(bytes));
                rendered.try_reserve_exact(bytes).is_ok()
            },
        )
        .unwrap();

        assert_eq!(requested.get(), Some(r"\u{e9}.or".len()));
        assert_eq!(encoded, RenderedSourceName::from_text("é.or"));
        assert_eq!(
            RenderedSourceName::try_from_text(r"\u{e9}.or").unwrap(),
            RenderedSourceName::from_text(r"\u{e9}.or")
        );
        let already_encoded = RenderedSourceName::try_from_encoded_characters_with_reservation(
            r"\x80.or".chars(),
            |rendered, bytes| rendered.try_reserve_exact(bytes).is_ok(),
        )
        .unwrap();
        assert_eq!(already_encoded.as_str(), r"\x80.or");
    }

    #[test]
    fn rendered_source_name_reservation_failure_returns_no_partial_name() {
        assert!(matches!(
            RenderedSourceName::try_from_characters_with_reservation("é.or".chars(), |_, _| false),
            Err(SourceError::SourceAllocationFailed)
        ));
        assert!(matches!(
            RenderedSourceName::try_from_encoded_characters_with_reservation(
                r"\x80.or".chars(),
                |_, _| false,
            ),
            Err(SourceError::SourceAllocationFailed)
        ));
    }

    #[test]
    fn source_ids_follow_insertion_order() {
        let mut sources = SourceMap::new();
        let first = sources.add("first.or", "one").unwrap();
        let second = sources.add("second.or", "two").unwrap();

        assert_eq!(first.index(), 0);
        assert_eq!(second.index(), 1);
        assert!(first < second);
        assert_eq!(sources.get(second).unwrap().name(), "second.or");
    }

    #[test]
    fn source_index_accepts_the_full_u32_domain() {
        assert_eq!(source_index(u32::MAX as usize), Ok(u32::MAX));

        #[cfg(target_pointer_width = "64")]
        assert_eq!(
            source_index(u32::MAX as usize + 1),
            Err(SourceError::TooManyFiles)
        );
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
    fn line_ending_interiors_and_trailing_eof_have_stable_positions() {
        let mut sources = SourceMap::new();
        let id = sources.add("positions.or", "a\r\nb\rc\n").unwrap();
        let source = sources.get(id).unwrap();
        let expected = [
            (1, 1),
            (1, 2),
            (1, 2),
            (2, 1),
            (2, 2),
            (3, 1),
            (3, 2),
            (4, 1),
        ];

        for (offset, (line, column)) in expected.into_iter().enumerate() {
            assert_eq!(
                source.line_column(TextOffset::new(u32::try_from(offset).unwrap())),
                Some(LineColumn { line, column }),
                "unexpected position for byte offset {offset}",
            );
        }
        assert_eq!(source.line_text(1), Some("a"));
        assert_eq!(source.line_text(2), Some("b"));
        assert_eq!(source.line_text(3), Some("c"));
        assert_eq!(source.line_text(4), Some(""));
        assert_eq!(source.line_text(5), None);
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
    fn generated_line_index_corpus_matches_a_direct_reference_model() {
        fn reference_position(text: &str, offset: usize) -> LineColumn {
            let mut index = 0_usize;
            let mut line = 1_u32;
            let mut column = 1_u32;

            while index < offset {
                let byte = text.as_bytes()[index];
                if byte == b'\r' {
                    let ending_width = if text.as_bytes().get(index + 1) == Some(&b'\n') {
                        2
                    } else {
                        1
                    };
                    if index + ending_width > offset {
                        break;
                    }
                    index += ending_width;
                    line += 1;
                    column = 1;
                } else if byte == b'\n' {
                    index += 1;
                    line += 1;
                    column = 1;
                } else {
                    let character = text[index..].chars().next().unwrap();
                    index += character.len_utf8();
                    column += 1;
                }
            }

            LineColumn { line, column }
        }

        fn assert_positions(text: String) {
            let mut sources = SourceMap::new();
            let id = sources.add("positions.or", text.clone()).unwrap();
            let source = sources.get(id).unwrap();

            for offset in 0..=text.len() {
                let offset_value = TextOffset::new(u32::try_from(offset).unwrap());
                if text.is_char_boundary(offset) {
                    assert_eq!(
                        source.line_column(offset_value),
                        Some(reference_position(&text, offset)),
                        "unexpected position for {text:?} at byte {offset}",
                    );
                } else {
                    assert_eq!(
                        source.line_column(offset_value),
                        None,
                        "accepted UTF-8 interior for {text:?} at byte {offset}",
                    );
                }
            }

            let mut expected_lines = Vec::new();
            let mut line_start = 0_usize;
            let mut index = 0_usize;
            while index < text.len() {
                let byte = text.as_bytes()[index];
                if byte == b'\r' || byte == b'\n' {
                    expected_lines.push((line_start, &text[line_start..index]));
                    index += if byte == b'\r' && text.as_bytes().get(index + 1) == Some(&b'\n') {
                        2
                    } else {
                        1
                    };
                    line_start = index;
                } else {
                    index += text[index..].chars().next().unwrap().len_utf8();
                }
            }
            expected_lines.push((line_start, &text[line_start..]));

            for (line_index, (expected_start, expected_text)) in
                expected_lines.into_iter().enumerate()
            {
                let line = u32::try_from(line_index + 1).unwrap();
                assert_eq!(source.line_text(line), Some(expected_text), "{text:?}");
                assert_eq!(
                    source.line_start(line),
                    Some(TextOffset::new(u32::try_from(expected_start).unwrap())),
                    "{text:?}"
                );
            }
            let missing_line = u32::try_from(source.line_starts.len() + 1).unwrap();
            assert_eq!(source.line_text(missing_line), None, "{text:?}");
            assert_eq!(source.line_start(missing_line), None, "{text:?}");
        }

        const ATOMS: [&str; 5] = ["a", "é", "🟠", "\r", "\n"];
        const CORPUS_SIZE: usize = 15_625;
        for encoded in 0..CORPUS_SIZE {
            let mut remaining = encoded;
            let mut text = String::new();
            for _ in 0..6 {
                text.push_str(ATOMS[remaining % ATOMS.len()]);
                remaining /= ATOMS.len();
            }
            assert_positions(text);
        }

        for prefix_bytes in 254..=258 {
            for ending in ["\r", "\n", "\r\n"] {
                for scalar in ["é", "🟠"] {
                    let text = format!(
                        "{}{scalar}{ending}{}{scalar}",
                        "a".repeat(prefix_bytes),
                        "b".repeat(258)
                    );
                    assert_positions(text);
                }
            }
        }
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
    fn checkpoint_reservation_failure_rejects_the_source_without_partial_state() {
        let id = SourceId { map: 1, index: 0 };
        let result = SourceFile::new_with_checkpoint_reservation(
            id,
            String::from("checkpoint.or"),
            false,
            "a".repeat(COLUMN_CHECKPOINT_INTERVAL_BYTES + 1),
            |_| false,
        );

        assert!(matches!(result, Err(SourceError::IndexAllocationFailed)));
    }

    #[test]
    fn failed_source_construction_does_not_consume_an_insertion_identity() {
        let mut sources = SourceMap::new();
        let first = sources.add("first.or", "first").unwrap();

        assert_eq!(
            sources.add("oversized.or", "a".repeat(MAX_SOURCE_BYTES + 1)),
            Err(SourceError::TooLarge)
        );
        assert_eq!(sources.len(), 1);

        let second = sources.add("second.or", "second").unwrap();
        assert_eq!(first.index(), 0);
        assert_eq!(second.index(), 1);
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn borrowed_source_copy_failures_preserve_empty_map_and_first_index() {
        for failing_reservation in [1_usize, 2] {
            let mut sources = SourceMap::new();
            let mut reservation = 0_usize;
            let result = sources.add_inner_with_string_reservation(
                Cow::Borrowed("borrowed.or"),
                false,
                Cow::Borrowed("source"),
                |string, bytes| {
                    reservation = reservation.saturating_add(1);
                    reservation != failing_reservation && string.try_reserve_exact(bytes).is_ok()
                },
            );

            assert_eq!(result, Err(SourceError::SourceAllocationFailed));
            assert!(sources.is_empty());
            let first = sources.add("first.or", "first").unwrap();
            assert_eq!(first.index(), 0);
        }
    }

    #[test]
    fn owned_source_strings_move_without_copy_reservations() {
        let mut sources = SourceMap::new();
        let name = String::from("owned.or");
        let text = String::from("owned source");
        let name_pointer = name.as_ptr();
        let text_pointer = text.as_ptr();
        let id = sources
            .add_inner_with_string_reservation(Cow::Owned(name), false, Cow::Owned(text), |_, _| {
                false
            })
            .unwrap();

        let source = sources.get(id).unwrap();
        assert_eq!(source.name(), "owned.or");
        assert_eq!(source.text(), "owned source");
        assert_eq!(source.name().as_ptr(), name_pointer);
        assert_eq!(source.text().as_ptr(), text_pointer);
    }

    #[test]
    fn rendered_name_path_is_atomic_and_moves_owned_data_without_copying() {
        let mut sources = SourceMap::new();
        let rejected_name = RenderedSourceName::from_text("rejected.or");
        let result = sources.add_inner_with_string_reservation(
            Cow::Owned(rejected_name.0),
            true,
            Cow::Borrowed("borrowed source"),
            |_, _| false,
        );

        assert_eq!(result, Err(SourceError::SourceAllocationFailed));
        assert!(sources.is_empty());

        let name = RenderedSourceName::from_text("é.or");
        let text = String::from("owned source");
        let name_pointer = name.0.as_ptr();
        let text_pointer = text.as_ptr();
        let id = sources
            .add_inner_with_string_reservation(
                Cow::Owned(name.0),
                true,
                Cow::Owned(text),
                |_, _| false,
            )
            .unwrap();

        assert_eq!(id.index(), 0);
        let source = sources.get(id).unwrap();
        assert_eq!(source.name(), r"\u{e9}.or");
        assert!(source.name_is_rendered());
        assert_eq!(source.name().as_ptr(), name_pointer);
        assert_eq!(source.text().as_ptr(), text_pointer);
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
