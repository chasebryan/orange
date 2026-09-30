//! Bytes: byte strings, the concatenation of arrays, and slices whose
//! bounds are proved in range and a fixed length apart before a program
//! runs.

use super::*;

/// The note of a character a byte string cannot hold.
const BYTE_STRING_NOTE: &str = "a byte string's characters are its bytes, so each is printable \
     ASCII, from ` ` through `~`; write any other byte as an escape, or in a hex string joined \
     with `++`";

/// The note of a slice update whose base or result is not an array.
const SLICE_UPDATE_NOTE: &str = "`x with [a..b] = v` is the array `x` with its elements from \
     index a up to b replaced by those of v";

/// The note of a slice whose bounds are not a fixed, positive distance
/// apart.
const SLICE_LENGTH_NOTE: &str = "a slice `x[a..b]` holds the b - a elements from index a, and b - a \
     must be the same positive number at every step, as in `x[16 * i..16 * i + 16]`";

/// The note of a slice bound that is not built from literals and loop
/// indices.
const STATIC_SLICE_NOTE: &str = "a slice's position never depends on data: its bounds are built \
     from integer literals and loop indices with `+`, `-`, and `*` by a constant";

/// A byte string that cannot be decoded, and why.
pub(super) enum ByteStringError {
    /// A character that is neither printable ASCII nor an escape, with its
    /// exact extent.
    Unprintable(Span, char),
    /// The number of bytes, which is 0 or more than [`MAX_ARRAY_LENGTH`];
    /// a count above the limit is reported as the limit plus one.
    Length(u32),
    /// Storage for the bytes could not be reserved.
    Storage,
}

/// Decodes the byte string spelled `spelling`, which the lexer accepted:
/// `"..."` with S2 escapes, or `hex"..."` with hex digit pairs and spaces.
/// `start` is the offset of the spelling in `source`, for the extent of an
/// unprintable character.
pub(super) fn decode_byte_string(
    spelling: &str,
    start: TextOffset,
    hex: bool,
    source: &SourceFile,
) -> Result<Vec<u8>, ByteStringError> {
    let quoted = if hex {
        spelling.strip_prefix("hex")
    } else {
        Some(spelling)
    };
    let contents = quoted
        .and_then(|quoted| quoted.strip_prefix('"'))
        .and_then(|quoted| quoted.strip_suffix('"'))
        .ok_or(ByteStringError::Storage)?;
    // The opening quote, and `hex` before it, precede the contents.
    let prefix = spelling
        .len()
        .checked_sub(contents.len())
        .and_then(|prefix| prefix.checked_sub(1))
        .ok_or(ByteStringError::Storage)?;
    let limit = usize::try_from(MAX_ARRAY_LENGTH).map_err(|_| ByteStringError::Storage)?;
    let mut bytes = Vec::new();
    if bytes.try_reserve_exact(limit).is_err() {
        return Err(ByteStringError::Storage);
    }
    // Decoding stops at the byte past the limit, so a long spelling is
    // never scanned to its end.
    let over = Err(ByteStringError::Length(MAX_ARRAY_LENGTH.saturating_add(1)));
    let push = |byte: u8, bytes: &mut Vec<u8>| {
        if bytes.len() < limit {
            bytes.push(byte);
            true
        } else {
            false
        }
    };
    if hex {
        let mut high: Option<char> = None;
        for digit in contents.chars().filter(char::is_ascii_hexdigit) {
            match high.take() {
                Some(high) => {
                    if !push(hex_byte(high, digit)?, &mut bytes) {
                        return over;
                    }
                }
                None => high = Some(digit),
            }
        }
    } else {
        let mut characters = contents.char_indices();
        while let Some((offset, character)) = characters.next() {
            let byte = if character == '\\' {
                match characters.next().map(|(_, escaped)| escaped) {
                    Some('n') => b'\n',
                    Some('r') => b'\r',
                    Some('t') => b'\t',
                    Some('0') => 0,
                    Some('x') => {
                        let high = characters.next().map(|(_, digit)| digit);
                        let low = characters.next().map(|(_, digit)| digit);
                        let (Some(high), Some(low)) = (high, low) else {
                            return Err(ByteStringError::Storage);
                        };
                        hex_byte(high, low)?
                    }
                    Some('"') => b'"',
                    Some('\\') => b'\\',
                    _ => return Err(ByteStringError::Storage),
                }
            } else if let Some(byte) = u8::try_from(character)
                .ok()
                .filter(|byte| (b' '..=b'~').contains(byte))
            {
                byte
            } else {
                let at = |offset: usize| {
                    u32::try_from(offset)
                        .ok()
                        .and_then(|offset| start.bytes().checked_add(offset))
                        .map(TextOffset::new)
                };
                let character_start = prefix.checked_add(offset);
                let character_end =
                    character_start.and_then(|start| start.checked_add(character.len_utf8()));
                let span = character_start
                    .and_then(at)
                    .zip(character_end.and_then(at))
                    .and_then(|(start, end)| source.span(start, end))
                    .ok_or(ByteStringError::Storage)?;
                return Err(ByteStringError::Unprintable(span, character));
            };
            if !push(byte, &mut bytes) {
                return over;
            }
        }
    }
    if bytes.is_empty() {
        return Err(ByteStringError::Length(0));
    }
    Ok(bytes)
}

/// Returns the byte of two hex digits.
fn hex_byte(high: char, low: char) -> Result<u8, ByteStringError> {
    let (Some(high), Some(low)) = (high.to_digit(16), low.to_digit(16)) else {
        return Err(ByteStringError::Storage);
    };
    high.checked_mul(16)
        .and_then(|high| high.checked_add(low))
        .and_then(|byte| u8::try_from(byte).ok())
        .ok_or(ByteStringError::Storage)
}

/// An `Int` expression built from integer literals and loop indices with
/// `+`, `-`, and `*` by a constant: a constant plus a sum of loop indices,
/// each with a nonzero coefficient.
pub(super) struct Affine {
    constant: ExactInteger,
    /// Loop-scope positions and their nonzero coefficients, by position.
    terms: Vec<(usize, ExactInteger)>,
}

impl Affine {
    fn constant(value: ExactInteger) -> Self {
        Self {
            constant: value,
            terms: Vec::new(),
        }
    }

    fn negated(self) -> Self {
        Self {
            constant: self.constant.negated(),
            terms: self
                .terms
                .into_iter()
                .map(|(position, coefficient)| (position, coefficient.negated()))
                .collect(),
        }
    }

    /// Returns `self + other`, or `self - other` when `subtract`, or `None`
    /// when storage cannot be reserved.
    fn combine(
        &self,
        other: &Self,
        subtract: bool,
        reserve: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        let apply = |left: &ExactInteger, right: &ExactInteger| {
            if subtract {
                left.subtract(right, reserve)
            } else {
                left.add(right, reserve)
            }
        };
        let zero = ExactInteger::from_u64(0, reserve)?;
        let mut terms = Vec::new();
        terms
            .try_reserve_exact(self.terms.len().checked_add(other.terms.len())?)
            .ok()?;
        let (mut left, mut right) = (self.terms.iter().peekable(), other.terms.iter().peekable());
        loop {
            let (position, coefficient) = match (left.peek(), right.peek()) {
                (None, None) => break,
                (Some((position, coefficient)), None) => {
                    left.next();
                    (*position, coefficient.try_clone_with_reservation(reserve)?)
                }
                (None, Some((position, coefficient))) => {
                    right.next();
                    (*position, apply(&zero, coefficient)?)
                }
                (
                    Some((left_position, left_coefficient)),
                    Some((right_position, right_coefficient)),
                ) => match left_position.cmp(right_position) {
                    Ordering::Less => {
                        left.next();
                        (
                            *left_position,
                            left_coefficient.try_clone_with_reservation(reserve)?,
                        )
                    }
                    Ordering::Greater => {
                        right.next();
                        (*right_position, apply(&zero, right_coefficient)?)
                    }
                    Ordering::Equal => {
                        left.next();
                        right.next();
                        (*left_position, apply(left_coefficient, right_coefficient)?)
                    }
                },
            };
            if !coefficient.is_zero() {
                terms.push((position, coefficient));
            }
        }
        Some(Self {
            constant: apply(&self.constant, &other.constant)?,
            terms,
        })
    }

    /// Returns `self * factor`, or `None` when storage cannot be reserved.
    fn scaled(
        &self,
        factor: &ExactInteger,
        reserve: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        let mut terms = Vec::new();
        if !factor.is_zero() {
            terms.try_reserve_exact(self.terms.len()).ok()?;
            for (position, coefficient) in &self.terms {
                terms.push((*position, coefficient.multiply(factor, reserve)?));
            }
        }
        Some(Self {
            constant: self.constant.multiply(factor, reserve)?,
            terms,
        })
    }

    /// Returns whether every coefficient and the constant have at most
    /// `bits` significant bits.
    fn within(&self, bits: usize) -> bool {
        self.constant.magnitude_bits() <= bits
            && self
                .terms
                .iter()
                .all(|(_, coefficient)| coefficient.magnitude_bits() <= bits)
    }
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Checks a byte string against `expected`, which must be `Word[8]^n`
    /// for its n bytes. A byte string is one literal node.
    pub(super) fn check_bytes(
        &mut self,
        expression: &'ast Expression,
        bytes: ByteString,
        expected: &CoreType,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some(decoded) = self.decode_bytes(expression.span, bytes.hex) else {
            return false;
        };
        // One event per byte, after the literal's own.
        for _ in &decoded {
            if !self.event(expression.span) {
                return false;
            }
        }
        let count = u32::try_from(decoded.len()).unwrap_or(u32::MAX);
        let fits = expected
            .as_array()
            .filter(|array| array.element() == CoreType::Word8);
        let Some(array) = fits.filter(|array| array.length() == count) else {
            self.report_byte_string_type(expression.span, count, expected);
            return false;
        };
        let mut elements = Vec::new();
        if elements.try_reserve_exact(decoded.len()).is_err() {
            self.resource_limit(expression.span, "byte string storage allocation failed");
            return false;
        }
        elements.extend(decoded.into_iter().map(CoreValue::Word8));
        let Some(value) = CoreArray::new(array, elements) else {
            self.resource_limit(expression.span, "byte string storage allocation failed");
            return false;
        };
        self.push_node(
            output,
            expression.span,
            expected.clone(),
            CoreNodeKind::Literal(CoreValue::Array(value)),
        )
    }

    /// Decodes a byte string, or reports why it has no value.
    fn decode_bytes(&mut self, span: Span, hex: bool) -> Option<Vec<u8>> {
        let spelling = self.source.slice(span)?;
        match decode_byte_string(spelling, span.start(), hex, self.source) {
            Ok(bytes) => Some(bytes),
            Err(ByteStringError::Unprintable(at, character)) => {
                if self.begin_report(at) {
                    let mut hex = String::new();
                    let mut buffer = [0; 4];
                    for byte in character.encode_utf8(&mut buffer).bytes() {
                        let separator = if hex.is_empty() { "" } else { " " };
                        let _ = fmt::write(&mut hex, format_args!("{separator}{byte:02x}"));
                    }
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnprintableByteString,
                            format!(
                                "U+{:04X} is not a printable ASCII character",
                                u32::from(character)
                            ),
                            at,
                        )
                        .with_label(if character.is_ascii() {
                            format!("its byte is written `hex\"{hex}\"`")
                        } else {
                            format!("its UTF-8 bytes are written `hex\"{hex}\"`")
                        })
                        .with_note(BYTE_STRING_NOTE),
                    );
                }
                None
            }
            Err(ByteStringError::Length(count)) => {
                if self.begin_report(span) {
                    let message = if count == 0 {
                        String::from("a byte string holds at least one byte")
                    } else {
                        format!("a byte string holds at most {MAX_ARRAY_LENGTH} bytes")
                    };
                    self.diagnostics.push(
                        Diagnostic::error(DiagnosticCode::UnsupportedArrayLength, message, span)
                            .with_label(if count == 0 {
                                String::from("this string is empty")
                            } else {
                                format!("this string holds more than {MAX_ARRAY_LENGTH}")
                            })
                            .with_note(format!(
                                "a byte string is an array `Word[8]^n` of 1 through \
                                 {MAX_ARRAY_LENGTH} bytes; join longer runs with `++`"
                            )),
                    );
                }
                None
            }
            Err(ByteStringError::Storage) => {
                self.resource_limit(span, "byte string storage allocation failed");
                None
            }
        }
    }

    #[cold]
    #[inline(never)]
    fn report_byte_string_type(&mut self, span: Span, count: u32, expected: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        let same_element = expected
            .as_array()
            .is_some_and(|array| array.element() == CoreType::Word8);
        let diagnostic = match expected.as_array() {
            Some(array) if same_element => Diagnostic::error(
                DiagnosticCode::ArrayLengthMismatch,
                format!(
                    "this byte string holds {count} {}, but `{expected}` has {}",
                    if count == 1 { "byte" } else { "bytes" },
                    array.length()
                ),
                span,
            )
            .with_label(format!("expected {} bytes", array.length())),
            _ => Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!(
                    "this byte string has type `Word[8]^{count}`, but `{expected}` is required here"
                ),
                span,
            )
            .with_label(format!("expected `{expected}`")),
        };
        self.diagnostics
            .push(diagnostic.with_note("a byte string is the array `Word[8]^n` of its n bytes"));
    }

    /// Returns the length of an array expression without reporting: from
    /// the elements of an array literal, the length of a fill, the bytes of
    /// a byte string, the operands of `++`, or the type of the first typed
    /// leaf, and for a conditional with no such leaf, from the first branch
    /// with no bindings whose value has a length. Returns `None` for an
    /// expression that is not an array.
    ///
    /// Parser-established expression height bounds this recursion.
    pub(super) fn array_length_of(
        &self,
        expression: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<u32> {
        match &expression.kind {
            ExpressionKind::Parenthesized(inner) => self.array_length_of(inner, context, scope),
            ExpressionKind::Array(array) => u32::try_from(array.elements.len()).ok(),
            ExpressionKind::Fill(fill) => array_length(self.source, fill.length_span)
                .filter(|length| (1..=MAX_ARRAY_LENGTH).contains(length)),
            ExpressionKind::Bytes(bytes) => {
                let spelling = self.source.slice(expression.span)?;
                let decoded =
                    decode_byte_string(spelling, expression.span.start(), bytes.hex, self.source)
                        .ok()?;
                u32::try_from(decoded.len()).ok()
            }
            ExpressionKind::Binary(binary) if binary.operator.is_concatenation() => self
                .array_length_of(&binary.left, context, scope)?
                .checked_add(self.array_length_of(&binary.right, context, scope)?),
            ExpressionKind::Update(update) => self.array_length_of(&update.base, context, scope),
            ExpressionKind::SliceUpdate(update) => {
                self.array_length_of(&update.base, context, scope)
            }
            ExpressionKind::Conditional(conditional) => {
                self.leaf_length(expression, context, scope).or_else(|| {
                    // A branch's bindings are not in scope here, so only a
                    // branch without them is read.
                    let otherwise = conditional
                        .otherwise_bindings
                        .is_empty()
                        .then_some(&conditional.otherwise);
                    conditional
                        .arms
                        .iter()
                        .filter(|arm| arm.bindings.is_empty())
                        .map(|arm| &arm.value)
                        .chain(otherwise)
                        .find_map(|value| self.array_length_of(value, context, scope))
                })
            }
            _ => self.leaf_length(expression, context, scope),
        }
    }

    /// Returns the length of the array type of an expression's first typed
    /// leaf, when it has one.
    fn leaf_length(
        &self,
        expression: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<u32> {
        first_typed_leaf(expression)
            .and_then(|leaf| self.leaf_type(leaf, context, scope))?
            .as_array()
            .map(ArrayType::length)
    }

    /// Returns the element type of an array expression without reporting,
    /// from its first operand that has one.
    ///
    /// Parser-established expression height bounds this recursion.
    fn array_element_of(
        &self,
        expression: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<CoreType> {
        match &expression.kind {
            ExpressionKind::Parenthesized(inner) => self.array_element_of(inner, context, scope),
            ExpressionKind::Bytes(_) => Some(CoreType::Word8),
            ExpressionKind::Binary(binary) if binary.operator.is_concatenation() => self
                .array_element_of(&binary.left, context, scope)
                .or_else(|| self.array_element_of(&binary.right, context, scope)),
            _ => first_typed_leaf(expression)
                .and_then(|leaf| self.leaf_type(leaf, context, scope))?
                .as_array()
                .map(ArrayType::element),
        }
    }

    /// Returns the type of `left ++ right` without reporting, when the
    /// lengths of both operands and the element type of one are known.
    pub(super) fn concatenation_type(
        &self,
        binary: &BinaryExpression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<CoreType> {
        let left = self.array_length_of(&binary.left, context, scope)?;
        let right = self.array_length_of(&binary.right, context, scope)?;
        let element = self
            .array_element_of(&binary.left, context, scope)
            .or_else(|| self.array_element_of(&binary.right, context, scope))?;
        ArrayType::new(&element, left.checked_add(right)?).map(CoreType::Array)
    }

    /// Checks `left ++ right` against `expected`, which must be an array
    /// type whose length is the sum of the operands' lengths.
    ///
    /// `++` associates to the left, so the joins of a chain `a ++ b ++ c`
    /// nest in their left operands as deeply as an expression's height
    /// allows. The check walks that spine in a loop rather than through
    /// [`Self::check_expression`], with the effect of checking each join
    /// recursively: every join is validated before its left operand, the
    /// innermost left operand is checked first, and each join's right
    /// operand is checked and its node pushed on the way back out. Only the
    /// operands recurse, so a chain of any length takes one frame.
    pub(super) fn check_concatenation(
        &mut self,
        expression: &'ast Expression,
        binary: &'ast BinaryExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        // Down the spine: validate each join, and check the left operand of
        // the innermost one that validates.
        let mut depth = 0_usize;
        let mut level = binary;
        let mut level_type = expected.clone();
        let (mut checked, mut outermost_open) = loop {
            let Some(left_type) = self.validate_join(level, &level_type, context, scope) else {
                // This join is rejected: it has no value, and the joins
                // around it finish with that.
                match depth.checked_sub(1) {
                    Some(outer) => break (false, outer),
                    None => return false,
                }
            };
            match &level.left.kind {
                ExpressionKind::Binary(inner) if inner.operator.is_concatenation() => {
                    if !self.event(inner.operator_span) {
                        break (false, depth);
                    }
                    depth = depth.saturating_add(1);
                    level = inner;
                    level_type = left_type;
                }
                _ => {
                    let left =
                        self.check_expression(&level.left, &left_type, context, scope, output);
                    break (left, depth);
                }
            }
        };
        // Back up the spine: finish each join with its left operand's result.
        loop {
            checked = self.finish_join(
                (expression, binary, outermost_open),
                expected,
                checked,
                context,
                scope,
                output,
            );
            match outermost_open.checked_sub(1) {
                Some(outer) => outermost_open = outer,
                None => return checked,
            }
        }
    }

    /// Validates one join of a `++` chain against `expected` before its
    /// operands are checked, and returns the type its left operand is
    /// checked against: the array of the left operand's length, or all of
    /// `expected` when that length is not found without checking.
    fn validate_join(
        &mut self,
        binary: &'ast BinaryExpression,
        expected: &CoreType,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<CoreType> {
        let Some(array) = expected.as_array() else {
            self.report_join_not_wanted(binary.operator_span, expected);
            return None;
        };
        let length = array.length();
        let Some(left) = self.array_length_of(&binary.left, context, scope) else {
            if let Some(ty) = self.known_non_array(&binary.left, context, scope) {
                self.report_joined_non_array(binary.left.span, &ty);
                return None;
            }
            return Some(expected.clone());
        };
        let right = self.array_length_of(&binary.right, context, scope);
        if right.is_none()
            && let Some(ty) = self.known_non_array(&binary.right, context, scope)
        {
            self.report_joined_non_array(binary.right.span, &ty);
            return None;
        }
        let fits = match right {
            Some(right) => left.checked_add(right) == Some(length),
            None => left < length,
        };
        if !fits {
            match right {
                Some(right) => {
                    self.report_join_length(binary.operator_span, (left, right), expected);
                }
                None => self.report_left_fills_join(binary.operator_span, left, expected),
            }
            return None;
        }
        let element = array.element();
        let right_type = length
            .checked_sub(left)
            .and_then(|right| ArrayType::new(&element, right));
        right_type?;
        ArrayType::new(&element, left).map(CoreType::Array)
    }

    /// Finishes the join `depth` levels down the left spine of the chain
    /// `outermost` checked against `expected`, whose left operand checked
    /// as `left_checked`: checks its right operand and pushes its node, or,
    /// when its left operand's length is not found without checking,
    /// reports a left operand that took the whole array.
    fn finish_join(
        &mut self,
        (outermost, binary, depth): (&'ast Expression, &'ast BinaryExpression, usize),
        expected: &CoreType,
        left_checked: bool,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some((expression, level)) = join_at(outermost, binary, depth) else {
            return false;
        };
        let Some(array) = expected.as_array() else {
            return false;
        };
        // A join's type is the array of its found length; a join with no
        // found length lies on the outer part of the spine, where every
        // join has all of `expected`.
        let whole = array.length();
        let length = if depth == 0 {
            whole
        } else {
            self.array_length_of(expression, context, scope)
                .unwrap_or(whole)
        };
        let Some(left) = self.array_length_of(&level.left, context, scope) else {
            if left_checked && !self.halted {
                // The left operand has no length of its own but checks as
                // the whole array, which leaves nothing for the right.
                self.report_left_fills_join(level.operator_span, whole, expected);
            }
            return false;
        };
        if self.halted {
            return false;
        }
        let element = array.element();
        let (Some(ty), Some(right_type)) = (
            ArrayType::new(&element, length),
            length
                .checked_sub(left)
                .and_then(|right| ArrayType::new(&element, right)),
        ) else {
            return false;
        };
        let right_checked = self.check_expression(
            &level.right,
            &CoreType::Array(right_type),
            context,
            scope,
            output,
        );
        left_checked
            && right_checked
            && self.push_node(
                output,
                expression.span,
                CoreType::Array(ty),
                CoreNodeKind::Concat,
            )
    }

    /// Returns the type of an operand of `++` that is found without
    /// reporting and is not an array.
    fn known_non_array(
        &self,
        operand: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<CoreType> {
        first_typed_leaf(operand)
            .and_then(|leaf| self.leaf_type(leaf, context, scope))
            .filter(|ty| ty.as_array().is_none())
    }

    #[cold]
    #[inline(never)]
    fn report_join_not_wanted(&mut self, span: Span, expected: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("`++` joins arrays, but `{expected}` is required here"),
                span,
            )
            .with_label(format!("`{expected}` is not an array type"))
            .with_note(CONCATENATION_NOTE),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_join_length(&mut self, span: Span, (left, right): (u32, u32), expected: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        let length = expected.as_array().map_or(0, ArrayType::length);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::ArrayLengthMismatch,
                format!(
                    "`++` joins {left} and {right} elements, {} in all, but `{expected}` has \
                     {length}",
                    u64::from(left).saturating_add(u64::from(right))
                ),
                span,
            )
            .with_label(format!("expected {length} elements in all"))
            .with_note(CONCATENATION_NOTE),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_joined_non_array(&mut self, span: Span, ty: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::NotAnArray,
                format!("only arrays can be joined, but this has type `{ty}`"),
                span,
            )
            .with_label(format!("`{ty}` is not an array"))
            .with_note(CONCATENATION_NOTE),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_left_fills_join(&mut self, span: Span, left: u32, expected: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        let length = expected.as_array().map_or(0, ArrayType::length);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::ArrayLengthMismatch,
                format!(
                    "the left operand of `++` has {left} elements, leaving none of the {length} \
                     of `{expected}` for the right"
                ),
                span,
            )
            .with_label(format!("expected {length} elements in all"))
            .with_note(CONCATENATION_NOTE),
        );
    }

    /// Checks `base[start..end]` against `expected`, which must be an array
    /// of the base's element type and of the slice's length.
    pub(super) fn check_slice(
        &mut self,
        expression: &'ast Expression,
        slice: &'ast SliceExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some(base_type) = self.leaf_type(&slice.base, context, scope) else {
            // The base's own check reports why it has no type.
            self.check_expression(&slice.base, expected, context, scope, output);
            return false;
        };
        let Some(array) = base_type.as_array() else {
            self.report_not_sliceable(slice.base.span, &base_type);
            self.check_expression(&slice.base, &base_type, context, scope, output);
            return false;
        };
        let element = array.element();
        let target = expected
            .as_array()
            .filter(|target| target.element() == element);
        if target.is_none() {
            self.report_slice_element(expression.span, &element, expected);
        }
        let base = self.check_expression(&slice.base, &base_type, context, scope, output);
        if self.halted {
            return false;
        }
        let Some(length) = self.check_slice_range(&slice.range, array, context, scope, output)
        else {
            return false;
        };
        let Some(target) = target else {
            return false;
        };
        if length != target.length() {
            self.report_slice_length_mismatch(expression.span, length, expected);
            return false;
        }
        base && self.push_node(
            output,
            expression.span,
            expected.clone(),
            CoreNodeKind::Slice,
        )
    }

    #[cold]
    #[inline(never)]
    fn report_slice_element(&mut self, span: Span, element: &CoreType, expected: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("this slice is an array of `{element}`, but `{expected}` is required here"),
                span,
            )
            .with_label(format!("expected `{expected}`"))
            .with_note("a slice is an array of the elements of the array it is taken from"),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_slice_length_mismatch(&mut self, span: Span, length: u32, expected: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        let expected_length = expected.as_array().map_or(0, ArrayType::length);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::ArrayLengthMismatch,
                format!(
                    "this slice has {length} {}, but `{expected}` has {expected_length}",
                    if length == 1 { "element" } else { "elements" },
                ),
                span,
            )
            .with_label(format!("expected {expected_length} elements"))
            .with_note(SLICE_LENGTH_NOTE),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_not_sliceable(&mut self, span: Span, base_type: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        let tuple = base_type.as_tuple().is_some();
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::NotAnArray,
                format!("only an array can be sliced, but this has type `{base_type}`"),
                span,
            )
            .with_label(if tuple {
                format!("`{base_type}` is a tuple, not an array")
            } else {
                format!("`{base_type}` has no elements")
            })
            .with_note("a slice `x[a..b]` is taken from a value of type `T^n`"),
        );
    }

    /// Checks `base with [start..end] = value` against `expected`, which
    /// must be the base's array type.
    pub(super) fn check_slice_update(
        &mut self,
        expression: &'ast Expression,
        update: &'ast SliceUpdateExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let base_type =
            first_typed_leaf(&update.base).and_then(|leaf| self.leaf_type(leaf, context, scope));
        if let Some(base_type) = base_type
            && base_type.as_array().is_none()
        {
            self.report_not_updatable(update.base.span, &base_type);
            self.check_expression(&update.base, &base_type, context, scope, output);
            return false;
        }
        let Some(array) = expected.as_array() else {
            self.report_update_not_wanted(expression.span, expected);
            return false;
        };
        let base = self.check_expression(&update.base, expected, context, scope, output);
        if self.halted {
            return false;
        }
        let Some(length) = self.check_slice_range(&update.range, array, context, scope, output)
        else {
            return false;
        };
        let Some(value_type) = ArrayType::new(&array.element(), length) else {
            return false;
        };
        let value = self.check_expression(
            &update.value,
            &CoreType::Array(value_type),
            context,
            scope,
            output,
        );
        base && value
            && self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::SliceUpdate,
            )
    }

    #[cold]
    #[inline(never)]
    fn report_update_not_wanted(&mut self, span: Span, expected: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeMismatch,
                format!("an update gives an array, but `{expected}` is required here"),
                span,
            )
            .with_label(format!("expected `{expected}`"))
            .with_note(SLICE_UPDATE_NOTE),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_not_updatable(&mut self, span: Span, base_type: &CoreType) {
        if !self.begin_report(span) {
            return;
        }
        let tuple = base_type.as_tuple().is_some();
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::NotAnArray,
                format!("only an array can be updated, but this has type `{base_type}`"),
                span,
            )
            .with_label(if tuple {
                format!("`{base_type}` is a tuple, not an array")
            } else {
                format!("`{base_type}` has no elements")
            })
            .with_note(if tuple {
                "a tuple with elements replaced is written anew, such as `(v, p.1)`"
            } else {
                SLICE_UPDATE_NOTE
            }),
        );
    }

    /// Checks the bounds of a slice of `array` and pushes their Core: the
    /// start, or 0, and then the end, or the array's length. Returns the
    /// slice's length once the bounds are proved static, a fixed positive
    /// distance apart, and within the array at every step.
    fn check_slice_range(
        &mut self,
        range: &'ast SliceRange,
        array: ArrayType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> Option<u32> {
        let length = array.length();
        let start = match &range.start {
            Some(start) => self.check_expression(start, &CoreType::Int, context, scope, output),
            None => self.push_int(output, range.dots_span, 0),
        };
        if self.halted {
            return None;
        }
        let end = match &range.end {
            Some(end) => self.check_expression(end, &CoreType::Int, context, scope, output),
            None => self.push_int(output, range.dots_span, u64::from(length)),
        };
        if !(start && end) {
            return None;
        }
        let start = self.bound_form((range.start.as_ref(), range.span), 0, context)?;
        let end = self.bound_form((range.end.as_ref(), range.span), u64::from(length), context)?;
        let reserve = self.reserve_range_limbs;
        let Some(difference) = end.combine(&start, true, reserve) else {
            self.resource_limit(range.span, "slice bound storage allocation failed");
            return None;
        };
        if !difference.terms.is_empty() || difference.constant.to_i64().is_none_or(|l| l < 1) {
            self.report_slice_length(range, &difference);
            return None;
        }
        let Some(((low, _), (_, high))) = self
            .affine_range(&start, context)
            .zip(self.affine_range(&end, context))
        else {
            self.resource_limit(range.span, "slice bound storage allocation failed");
            return None;
        };
        let in_range =
            !low.is_negative() && high.to_i64().is_some_and(|high| high <= i64::from(length));
        if !in_range {
            self.report_slice_range(range, (&low, &high), array);
            return None;
        }
        difference
            .constant
            .to_i64()
            .and_then(|length| u32::try_from(length).ok())
    }

    /// Pushes the `Int` literal `value`, which stands for an omitted bound.
    fn push_int(&mut self, output: &mut BodyOutput<'_>, span: Span, value: u64) -> bool {
        let Some(value) = ExactInteger::from_u64(value, self.reserve_range_limbs) else {
            self.resource_limit(span, "slice bound storage allocation failed");
            return false;
        };
        self.push_node(
            output,
            span,
            CoreType::Int,
            CoreNodeKind::Literal(CoreValue::Int(value)),
        )
    }

    /// Returns the affine form of a checked slice bound, or of `omitted`
    /// when the bound is not written, or reports why it has none.
    fn bound_form(
        &mut self,
        (bound, range_span): (Option<&Expression>, Span),
        omitted: u64,
        context: &BodyContext<'ast>,
    ) -> Option<Affine> {
        let Some(bound) = bound else {
            let Some(value) = ExactInteger::from_u64(omitted, self.reserve_range_limbs) else {
                self.resource_limit(range_span, "slice bound storage allocation failed");
                return None;
            };
            return Some(Affine::constant(value));
        };
        match self.affine_form(bound, context) {
            Ok(Some(form)) if form.within(self.limits.integer_bits) => Some(form),
            Ok(Some(_)) => {
                if self.begin_report(bound.span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::IndexOutOfRange,
                            format!(
                                "a part of this bound exceeds the {}-significant-bit limit of \
                                 `Int`",
                                self.limits.integer_bits
                            ),
                            bound.span,
                        )
                        .with_label("this bound has no representable range")
                        .with_note(STATIC_SLICE_NOTE),
                    );
                }
                None
            }
            Ok(None) => {
                self.resource_limit(bound.span, "slice bound storage allocation failed");
                None
            }
            Err((span, product)) => {
                if self.begin_report(span) {
                    let (message, label) = if product {
                        (
                            "a slice's bound may multiply a loop index only by a constant",
                            "both operands of this `*` use a loop index",
                        )
                    } else {
                        (
                            "a slice's bounds may use only integer literals and loop indices",
                            "this is neither",
                        )
                    };
                    self.diagnostics.push(
                        Diagnostic::error(DiagnosticCode::NonStaticIndex, message, span)
                            .with_label(label)
                            .with_note(STATIC_SLICE_NOTE),
                    );
                }
                None
            }
        }
    }

    /// Returns the affine form of a well-typed `Int` slice bound: integer
    /// literals, loop indices, parentheses, negation, `+`, `-`, and `*` of
    /// which one operand uses no loop index. The first other part is
    /// returned as its span, with whether it is a product of two loop
    /// indices; `Ok(None)` means storage could not be reserved.
    ///
    /// Parser-established expression height bounds this recursion.
    pub(super) fn affine_form(
        &self,
        bound: &Expression,
        context: &BodyContext<'ast>,
    ) -> Result<Option<Affine>, (Span, bool)> {
        let reserve = self.reserve_range_limbs;
        Ok(match &bound.kind {
            ExpressionKind::Literal(literal) => self.range_literal(literal).map(Affine::constant),
            ExpressionKind::Parenthesized(inner) => return self.affine_form(inner, context),
            ExpressionKind::Name(name) => match context.resolve(&name.text) {
                NameResolution::LoopIndex(position) => ExactInteger::from_u64(0, reserve)
                    .zip(ExactInteger::from_u64(1, reserve))
                    .map(|(constant, one)| Affine {
                        constant,
                        terms: vec![(position, one)],
                    }),
                _ => return Err((name.span, false)),
            },
            ExpressionKind::Unary(unary) if unary.operator == UnaryOperator::Negate => self
                .affine_form(&unary.operand, context)?
                .map(Affine::negated),
            ExpressionKind::Binary(binary)
                if matches!(
                    binary.operator,
                    BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply
                ) =>
            {
                let left = self.affine_form(&binary.left, context)?;
                let right = self.affine_form(&binary.right, context)?;
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                match binary.operator {
                    BinaryOperator::Multiply if left.terms.is_empty() => {
                        right.scaled(&left.constant, reserve)
                    }
                    BinaryOperator::Multiply if right.terms.is_empty() => {
                        left.scaled(&right.constant, reserve)
                    }
                    BinaryOperator::Multiply => return Err((binary.operator_span, true)),
                    operator => left.combine(&right, operator == BinaryOperator::Subtract, reserve),
                }
            }
            _ => return Err((bound.span, false)),
        })
    }

    /// Returns the length of a slice of an array of `base_length` elements
    /// without reporting, when its bounds are static and a fixed positive
    /// distance apart. An omitted start is 0 and an omitted end the base's
    /// length.
    pub(super) fn slice_length(
        &self,
        range: &SliceRange,
        base_length: u32,
        context: &BodyContext<'ast>,
    ) -> Option<u32> {
        let reserve = self.reserve_range_limbs;
        let end = match &range.end {
            Some(end) => self.affine_form(end, context).ok()??,
            None => Affine::constant(ExactInteger::from_u64(u64::from(base_length), reserve)?),
        };
        let start = match &range.start {
            Some(start) => self.affine_form(start, context).ok()??,
            None => Affine::constant(ExactInteger::from_u64(0, reserve)?),
        };
        let difference = end.combine(&start, true, reserve)?;
        if !difference.terms.is_empty() {
            return None;
        }
        u32::try_from(difference.constant.to_i64()?)
            .ok()
            .filter(|length| (1..=MAX_ARRAY_LENGTH).contains(length))
    }

    /// Returns the least and greatest values of an affine form over the
    /// ranges of its loop indices, or `None` when storage cannot be
    /// reserved.
    fn affine_range(&self, form: &Affine, context: &BodyContext<'ast>) -> Option<IndexRange> {
        let reserve = self.reserve_range_limbs;
        let mut low = form.constant.try_clone_with_reservation(reserve)?;
        let mut high = form.constant.try_clone_with_reservation(reserve)?;
        for (position, coefficient) in &form.terms {
            let scope = context.loop_scopes.get(*position)?;
            let first = ExactInteger::from_u64(u64::from(scope.start), reserve)?;
            let last = ExactInteger::from_u64(u64::from(scope.end.checked_sub(1)?), reserve)?;
            let at_first = coefficient.multiply(&first, reserve)?;
            let at_last = coefficient.multiply(&last, reserve)?;
            let (least, greatest) = if at_first.compare(&at_last) == Ordering::Greater {
                (at_last, at_first)
            } else {
                (at_first, at_last)
            };
            low = low.add(&least, reserve)?;
            high = high.add(&greatest, reserve)?;
        }
        Some((low, high))
    }

    #[cold]
    #[inline(never)]
    fn report_slice_length(&mut self, range: &SliceRange, difference: &Affine) {
        if !self.begin_report(range.span) {
            return;
        }
        let (message, label) = if difference.terms.is_empty() {
            match difference.constant.to_i64() {
                Some(0) => (
                    String::from("this slice is empty: its bounds are equal"),
                    String::from("a slice holds at least one element"),
                ),
                Some(length) if length < 0 => (
                    format!(
                        "this slice ends {} {} before it starts",
                        length.unsigned_abs(),
                        if length == -1 { "element" } else { "elements" }
                    ),
                    String::from("a slice holds at least one element"),
                ),
                _ => (
                    String::from("this slice's bounds are too far apart"),
                    String::from("a slice holds at least one element"),
                ),
            }
        } else {
            (
                String::from("the length of this slice changes from step to step"),
                String::from("its bounds must differ by the same number at every step"),
            )
        };
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::SliceLength, message, range.span)
                .with_label(label)
                .with_note(SLICE_LENGTH_NOTE),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_slice_range(
        &mut self,
        range: &SliceRange,
        (low, high): (&ExactInteger, &ExactInteger),
        array: ArrayType,
    ) {
        if !self.begin_report(range.span) {
            return;
        }
        let highest = array.length().saturating_sub(1);
        let array = CoreType::Array(array);
        let last = ExactInteger::from_u64(1, self.reserve_range_limbs)
            .and_then(|one| high.subtract(&one, self.reserve_range_limbs));
        let message = match (render_exact(low), last.as_ref().and_then(render_exact)) {
            (Some(low), Some(last)) => format!(
                "this slice reaches elements {low} through {last}, out of range for `{array}`"
            ),
            _ => format!("this slice is out of range for `{array}`"),
        };
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::IndexOutOfRange, message, range.span)
                .with_label(format!("indices run from 0 through {highest}"))
                .with_note(
                    "every element a slice can take, over every loop index in its bounds, must \
                     be an element of the array",
                ),
        );
    }
}

/// Returns the join `depth` levels down the left spine of the join
/// `expression`, whose operator expression is `binary`, with its own
/// expression.
fn join_at<'ast>(
    expression: &'ast Expression,
    binary: &'ast BinaryExpression,
    depth: usize,
) -> Option<(&'ast Expression, &'ast BinaryExpression)> {
    let mut level = (expression, binary);
    for _ in 0..depth {
        let ExpressionKind::Binary(inner) = &level.1.left.kind else {
            return None;
        };
        if !inner.operator.is_concatenation() {
            return None;
        }
        level = (&level.1.left, inner);
    }
    Some(level)
}

/// The note of every concatenation diagnostic.
const CONCATENATION_NOTE: &str = "`a ++ b` is the array of the elements of a followed by those of \
     b, of one element type";
