//! Static ranges: proving every index in range before a program runs, from
//! the ranges of literals, loop indices, words, and their operators.

use super::*;

/// The least and greatest values an index can take, as exact integers.
pub(super) type IndexRange = (ExactInteger, ExactInteger);

/// The least and greatest values of a word expression. Bounds are kept in
/// `u128` so that no bound computation of two 64-bit words overflows.
pub(super) type WordRange = (u128, u128);

/// Returns the least value of the form 2^k - 1 that is at least `value`.
pub(super) fn all_ones(value: u128) -> u128 {
    u128::MAX.checked_shr(value.leading_zeros()).unwrap_or(0)
}

/// Returns the range of `left operator right` for the ranges of its
/// operands, computed exactly, or `None` when storage cannot be reserved.
pub(super) fn combine_ranges(
    operator: BinaryOperator,
    (left_low, left_high): &IndexRange,
    (right_low, right_high): &IndexRange,
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<IndexRange> {
    match operator {
        BinaryOperator::Add => Some((
            left_low.add(right_low, reserve_limbs)?,
            left_high.add(right_high, reserve_limbs)?,
        )),
        BinaryOperator::Subtract => Some((
            left_low.subtract(right_high, reserve_limbs)?,
            left_high.subtract(right_low, reserve_limbs)?,
        )),
        BinaryOperator::Multiply => {
            let mut products = [
                left_low.multiply(right_low, reserve_limbs)?,
                left_low.multiply(right_high, reserve_limbs)?,
                left_high.multiply(right_low, reserve_limbs)?,
                left_high.multiply(right_high, reserve_limbs)?,
            ];
            products.sort_unstable_by(ExactInteger::compare);
            let [low, _, _, high] = products;
            Some((low, high))
        }
        _ => None,
    }
}

/// Writes an exact integer in decimal, or returns `None` when storage
/// cannot be reserved.
pub(super) fn render_exact(value: &ExactInteger) -> Option<String> {
    let mut text = String::new();
    fmt::write(&mut text, format_args!("{value}")).ok()?;
    Some(text)
}

pub(super) fn reserve_range_limbs(limbs: &mut Vec<u32>, count: usize) -> bool {
    limbs.try_reserve_exact(count).is_ok()
}

/// Returns the range of `left / right` or `left % right` under the total
/// Euclidean rules of S3f, for exact operand ranges, or `None` when storage
/// cannot be reserved. Positive, zero, and negative divisors are considered
/// separately and their ranges joined.
pub(super) fn divide_ranges(
    operator: BinaryOperator,
    left: &IndexRange,
    (divisor_low, divisor_high): &IndexRange,
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<IndexRange> {
    let one = ExactInteger::from_u64(1, reserve_limbs)?;
    let minus_one = one.try_clone_with_reservation(reserve_limbs)?.negated();
    let zero = ExactInteger::from_u64(0, reserve_limbs)?;
    let copy = |value: &ExactInteger| value.try_clone_with_reservation(reserve_limbs);
    let mut parts: [Option<IndexRange>; 3] = [None, None, None];
    if divisor_high.compare(&one) != Ordering::Less {
        let lowest = if divisor_low.compare(&one) == Ordering::Less {
            copy(&one)?
        } else {
            copy(divisor_low)?
        };
        parts[0] = Some(positive_divisor_range(
            operator,
            left,
            (&lowest, divisor_high),
            reserve_limbs,
        )?);
    }
    if divisor_low.compare(&zero) != Ordering::Greater
        && divisor_high.compare(&zero) != Ordering::Less
    {
        // x / 0 = 0 and x % 0 = x.
        parts[1] = Some(if operator == BinaryOperator::Divide {
            (copy(&zero)?, copy(&zero)?)
        } else {
            (copy(&left.0)?, copy(&left.1)?)
        });
    }
    if divisor_low.compare(&minus_one) != Ordering::Greater {
        // For d < 0, x / d = -(x / -d) and x % d = x % -d.
        let nearest = if divisor_high.compare(&minus_one) == Ordering::Greater {
            copy(&one)?
        } else {
            copy(divisor_high)?.negated()
        };
        let farthest = copy(divisor_low)?.negated();
        let (low, high) =
            positive_divisor_range(operator, left, (&nearest, &farthest), reserve_limbs)?;
        parts[2] = Some(if operator == BinaryOperator::Divide {
            (high.negated(), low.negated())
        } else {
            (low, high)
        });
    }
    parts
        .into_iter()
        .flatten()
        .reduce(|(low, high), (part_low, part_high)| {
            (
                if part_low.compare(&low) == Ordering::Less {
                    part_low
                } else {
                    low
                },
                if part_high.compare(&high) == Ordering::Greater {
                    part_high
                } else {
                    high
                },
            )
        })
}

/// Returns the range of `left / d` or `left % d` over divisors d from
/// `divisor_low` through `divisor_high`, where `1 <= divisor_low`, or `None`
/// when storage cannot be reserved.
pub(super) fn positive_divisor_range(
    operator: BinaryOperator,
    (low, high): &IndexRange,
    (divisor_low, divisor_high): (&ExactInteger, &ExactInteger),
    reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
) -> Option<IndexRange> {
    let divide =
        |value: &ExactInteger, divisor: &ExactInteger| value.divide_euclid(divisor, reserve_limbs);
    if operator == BinaryOperator::Divide {
        // The quotient grows with x, and for a fixed x it moves toward zero
        // as d grows, so its extremes are at the corners.
        let mut corners = [
            divide(low, divisor_low)?.0,
            divide(low, divisor_high)?.0,
            divide(high, divisor_low)?.0,
            divide(high, divisor_high)?.0,
        ];
        corners.sort_unstable_by(ExactInteger::compare);
        let [least, _, _, greatest] = corners;
        return Some((least, greatest));
    }
    // With one divisor and one quotient, the remainder grows with x.
    if divisor_low.compare(divisor_high) == Ordering::Equal {
        let (low_quotient, low_remainder) = divide(low, divisor_low)?;
        let (high_quotient, high_remainder) = divide(high, divisor_low)?;
        if low_quotient.compare(&high_quotient) == Ordering::Equal {
            return Some((low_remainder, high_remainder));
        }
    }
    // Otherwise 0 <= r < d, and r <= x when x is not negative.
    let one = ExactInteger::from_u64(1, reserve_limbs)?;
    let largest = divisor_high.subtract(&one, reserve_limbs)?;
    let greatest = if !low.is_negative() && high.compare(&largest) == Ordering::Less {
        high.try_clone_with_reservation(reserve_limbs)?
    } else {
        largest
    };
    Some((ExactInteger::from_u64(0, reserve_limbs)?, greatest))
}

pub(super) fn word_maximum(ty: &CoreType) -> Option<u64> {
    match ty {
        CoreType::Int
        | CoreType::Bool
        | CoreType::Mod(_)
        | CoreType::Array(_)
        | CoreType::Tuple(_) => None,
        CoreType::Word8 => Some(u64::from(u8::MAX)),
        CoreType::Word16 => Some(u64::from(u16::MAX)),
        CoreType::Word32 => Some(u64::from(u32::MAX)),
        CoreType::Word64 => Some(u64::MAX),
    }
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Checks an index expression, then proves it in range for `array`.
    ///
    /// The index has the type of its first typed leaf when that is a word
    /// type, and is an `Int` otherwise. A word index ranges over its type,
    /// narrowed by its operators, and is converted to its unsigned `Int`
    /// value in Core; an `Int` index takes its range from its literals, loop
    /// indices, and converted words.
    pub(super) fn check_static_index(
        &mut self,
        index: &'ast Expression,
        array: ArrayType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let word = first_typed_leaf(index)
            .and_then(|leaf| self.leaf_type(leaf, context, scope))
            .filter(|ty| word_maximum(ty).is_some());
        let index_type = word.clone().unwrap_or(CoreType::Int);
        if !self.check_expression(index, &index_type, context, scope, output) {
            return false;
        }
        let length = array.length();
        let range = match word.as_ref().and_then(word_maximum) {
            Some(maximum) => {
                let range = self.word_range(index, u128::from(maximum), context, scope);
                Ok(self.exact_word_range(index.span, range))
            }
            None => self.static_range(index, context, scope),
        };
        let range = match range {
            Ok(range) => range,
            Err(span) => {
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::NonStaticIndex,
                            "an `Int` index may use only integer literals, loop indices, and \
                             words converted with `as Int`",
                            span,
                        )
                        .with_label("this `Int` has no bound")
                        .with_note(STATIC_INDEX_NOTE),
                    );
                }
                return false;
            }
        };
        let in_range = range.as_ref().is_some_and(|(low, high)| {
            !low.is_negative() && high.to_i64().is_some_and(|high| high < i64::from(length))
        });
        if !in_range && self.begin_report(index.span) {
            let highest = length.saturating_sub(1);
            let array = CoreType::Array(array);
            let message = match &range {
                Some((low, high)) => match (render_exact(low), render_exact(high)) {
                    (Some(low_text), Some(_)) if low.compare(high) == Ordering::Equal => {
                        format!("index {low_text} is out of range for `{array}`")
                    }
                    (Some(low_text), Some(high_text)) => format!(
                        "this index runs from {low_text} through {high_text}, out of range \
                         for `{array}`"
                    ),
                    _ => format!("this index is out of range for `{array}`"),
                },
                None => format!(
                    "a bound of this index's range exceeds the {}-significant-bit limit of \
                     `Int`",
                    self.limits.integer_bits
                ),
            };
            self.diagnostics.push(
                Diagnostic::error(DiagnosticCode::IndexOutOfRange, message, index.span)
                    .with_label(format!("indices run from 0 through {highest}"))
                    .with_note(
                        "every value an index can take, over every loop index and word in it, \
                         must select an element",
                    ),
            );
        }
        match word {
            Some(from) if in_range => self.push_node(
                output,
                index.span,
                CoreType::Int,
                CoreNodeKind::Convert { from },
            ),
            _ => in_range,
        }
    }

    /// Returns the least and greatest values of a well-typed `Int` index
    /// built from literals, loop indices, words converted with `as Int`,
    /// parentheses, negation, `+`, `-`, `*`, `/`, `%`, and conditionals,
    /// computed exactly, or `None` when a bound's magnitude exceeds the
    /// integer limit or storage cannot be reserved (which is reported as a
    /// resource limit). Anything else is returned as the span of the first
    /// such part.
    ///
    /// Parser-established expression height bounds this recursion.
    pub(super) fn static_range(
        &mut self,
        index: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Result<Option<IndexRange>, Span> {
        let range = match &index.kind {
            ExpressionKind::Literal(literal) => self.range_literal(literal).and_then(|value| {
                let copy = value.try_clone_with_reservation(self.reserve_range_limbs)?;
                Some((value, copy))
            }),
            ExpressionKind::Parenthesized(inner) => {
                return self.static_range(inner, context, scope);
            }
            ExpressionKind::Name(name) => match context.resolve(&name.text) {
                // A size is a constant of the instance.
                NameResolution::Size(value) => {
                    ExactInteger::from_u64(u64::from(value), self.reserve_range_limbs).zip(
                        ExactInteger::from_u64(u64::from(value), self.reserve_range_limbs),
                    )
                }
                NameResolution::LoopIndex(position) => {
                    let scope = context.loop_scopes.get(position).ok_or(name.span)?;
                    let last = scope.end.checked_sub(1).ok_or(name.span)?;
                    ExactInteger::from_u64(u64::from(scope.start), self.reserve_range_limbs).zip(
                        ExactInteger::from_u64(u64::from(last), self.reserve_range_limbs),
                    )
                }
                // A position ranges over every integer in its declared range.
                NameResolution::Position { low, high, .. } => {
                    let last = high.checked_sub(1).ok_or(name.span)?;
                    ExactInteger::from_u64(u64::from(low), self.reserve_range_limbs).zip(
                        ExactInteger::from_u64(u64::from(last), self.reserve_range_limbs),
                    )
                }
                _ => return Err(name.span),
            },
            ExpressionKind::Unary(unary) if unary.operator == UnaryOperator::Negate => {
                return Ok(self
                    .static_range(&unary.operand, context, scope)?
                    .map(|(low, high)| (high.negated(), low.negated())));
            }
            ExpressionKind::Binary(binary)
                if matches!(
                    binary.operator,
                    BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply
                ) =>
            {
                let left = self.static_range(&binary.left, context, scope)?;
                let right = self.static_range(&binary.right, context, scope)?;
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                combine_ranges(binary.operator, &left, &right, self.reserve_range_limbs)
            }
            ExpressionKind::Binary(binary) if binary.operator.is_division() => {
                let left = self.static_range(&binary.left, context, scope)?;
                let right = self.static_range(&binary.right, context, scope)?;
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                divide_ranges(binary.operator, &left, &right, self.reserve_range_limbs)
            }
            ExpressionKind::Conversion(conversion) => {
                let from = first_typed_leaf(&conversion.operand)
                    .and_then(|leaf| self.leaf_type(leaf, context, scope));
                match from {
                    Some(CoreType::Int) => {
                        return self.static_range(&conversion.operand, context, scope);
                    }
                    // A residue converts to its least residue, 0 through m - 1.
                    Some(CoreType::Mod(modulus)) => {
                        let reserve = self.reserve_range_limbs;
                        let high = modulus
                            .to_exact(reserve)
                            .zip(ExactInteger::from_u64(1, reserve));
                        ExactInteger::from_u64(0, reserve)
                            .zip(high.and_then(|(modulus, one)| modulus.subtract(&one, reserve)))
                    }
                    Some(from) => {
                        let Some(maximum) = word_maximum(&from) else {
                            return Err(index.span);
                        };
                        let range = self.word_range(
                            &conversion.operand,
                            u128::from(maximum),
                            context,
                            scope,
                        );
                        return Ok(self.exact_word_range(index.span, range));
                    }
                    None => return Err(index.span),
                }
            }
            ExpressionKind::Conditional(conditional) => {
                let mut joined: Option<IndexRange> = None;
                for value in conditional
                    .arms
                    .iter()
                    .map(|arm| &arm.value)
                    .chain(std::iter::once(&conditional.otherwise))
                {
                    let Some((low, high)) = self.static_range(value, context, scope)? else {
                        return Ok(None);
                    };
                    joined = Some(match joined {
                        None => (low, high),
                        Some((least, greatest)) => (
                            if low.compare(&least) == Ordering::Less {
                                low
                            } else {
                                least
                            },
                            if high.compare(&greatest) == Ordering::Greater {
                                high
                            } else {
                                greatest
                            },
                        ),
                    });
                }
                joined
            }
            _ => return Err(index.span),
        };
        let Some(range) = range else {
            self.resource_limit(index.span, "index range storage allocation failed");
            return Ok(None);
        };
        let bits = self.limits.integer_bits;
        Ok((range.0.magnitude_bits() <= bits && range.1.magnitude_bits() <= bits).then_some(range))
    }

    /// Decodes an index literal exactly, without events or diagnostics; the
    /// literal was already checked as an `Int`. Returns `None` when storage
    /// cannot be reserved.
    pub(super) fn range_literal(&self, literal: &IntegerLiteral) -> Option<ExactInteger> {
        let spelling = self.source.slice(literal.magnitude_span)?;
        let (radix, digits) = if let Some(digits) = spelling
            .strip_prefix("0b")
            .or_else(|| spelling.strip_prefix("0B"))
        {
            (2, digits)
        } else if let Some(digits) = spelling
            .strip_prefix("0x")
            .or_else(|| spelling.strip_prefix("0X"))
        {
            (16, digits)
        } else {
            (10, spelling)
        };
        let mut magnitude = Magnitude::zero();
        for character in digits.chars().filter(|character| *character != '_') {
            let digit = character.to_digit(radix)?;
            if !magnitude.multiply_add_with_reservation(radix, digit, self.reserve_magnitude_limb) {
                return None;
            }
        }
        Some(ExactInteger::new(literal.negative, magnitude))
    }

    /// Returns the least and greatest values of a well-typed word expression
    /// whose type's greatest value is `maximum`. Every word lies in its type,
    /// and an operator narrows that where its result provably does not wrap:
    /// `&`, `|`, `^`, `~`, `/`, `%`, shifts by a literal amount, and `+`,
    /// `-`, and `*` whose bounds stay within the type. Conditionals join
    /// their branches, and a conversion from a narrower word keeps its range.
    /// Everything else ranges over the whole type. No bound is ever more
    /// than `maximum`.
    ///
    /// Parser-established expression height bounds this recursion.
    pub(super) fn word_range(
        &self,
        expression: &Expression,
        maximum: u128,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> WordRange {
        let whole = (0, maximum);
        match &expression.kind {
            ExpressionKind::Literal(literal) if !literal.negative => self
                .literal_value(literal)
                .filter(|value| *value <= maximum)
                .map_or(whole, |value| (value, value)),
            ExpressionKind::Parenthesized(inner) => self.word_range(inner, maximum, context, scope),
            ExpressionKind::Unary(unary) if unary.operator == UnaryOperator::Complement => {
                let (low, high) = self.word_range(&unary.operand, maximum, context, scope);
                (maximum.saturating_sub(high), maximum.saturating_sub(low))
            }
            ExpressionKind::Binary(binary) if binary.operator.is_shift_or_rotation() => {
                let (low, high) = self.word_range(&binary.left, maximum, context, scope);
                let amount = match &binary.right.kind {
                    ExpressionKind::Literal(literal) if !literal.negative => self
                        .literal_value(literal)
                        .and_then(|amount| u32::try_from(amount).ok()),
                    _ => None,
                };
                match (binary.operator, amount) {
                    (BinaryOperator::ShiftRight, Some(amount)) => (
                        low.checked_shr(amount).unwrap_or(0),
                        high.checked_shr(amount).unwrap_or(0),
                    ),
                    (BinaryOperator::ShiftLeft, Some(amount)) => {
                        match (low.checked_shl(amount), high.checked_shl(amount)) {
                            (Some(least), Some(greatest))
                                if greatest.checked_shr(amount) == Some(high)
                                    && greatest <= maximum =>
                            {
                                (least, greatest)
                            }
                            _ => whole,
                        }
                    }
                    _ => whole,
                }
            }
            ExpressionKind::Binary(binary) => {
                let (left_low, left_high) = self.word_range(&binary.left, maximum, context, scope);
                let (right_low, right_high) =
                    self.word_range(&binary.right, maximum, context, scope);
                let within = |low: Option<u128>, high: Option<u128>| match (low, high) {
                    (Some(low), Some(high)) if high <= maximum => (low, high),
                    _ => whole,
                };
                match binary.operator {
                    BinaryOperator::And => (0, left_high.min(right_high)),
                    BinaryOperator::Or => {
                        (left_low.max(right_low), all_ones(left_high.max(right_high)))
                    }
                    BinaryOperator::Xor => (0, all_ones(left_high.max(right_high))),
                    BinaryOperator::Add => within(
                        left_low.checked_add(right_low),
                        left_high.checked_add(right_high),
                    ),
                    BinaryOperator::Subtract if left_low >= right_high => within(
                        left_low.checked_sub(right_high),
                        left_high.checked_sub(right_low),
                    ),
                    BinaryOperator::Multiply => within(
                        left_low.checked_mul(right_low),
                        left_high.checked_mul(right_high),
                    ),
                    // x / 0 = 0, and a quotient never exceeds its dividend.
                    BinaryOperator::Divide => match (
                        left_low.checked_div(right_high),
                        left_high.checked_div(right_low),
                    ) {
                        (Some(low), Some(high)) => (low, high),
                        _ => (0, left_high),
                    },
                    // x % 0 = x, and otherwise 0 <= x % d <= min(x, d - 1).
                    BinaryOperator::Remainder if right_low > 0 && left_high < right_low => {
                        (left_low, left_high)
                    }
                    BinaryOperator::Remainder if right_low > 0 => {
                        (0, left_high.min(right_high.saturating_sub(1)))
                    }
                    BinaryOperator::Remainder => (0, left_high),
                    _ => whole,
                }
            }
            ExpressionKind::Conditional(conditional) => conditional
                .arms
                .iter()
                .map(|arm| &arm.value)
                .chain(std::iter::once(&conditional.otherwise))
                .map(|value| self.word_range(value, maximum, context, scope))
                .reduce(|(low, high), (least, greatest)| (low.min(least), high.max(greatest)))
                .unwrap_or(whole),
            ExpressionKind::Conversion(conversion) => {
                let from = first_typed_leaf(&conversion.operand)
                    .and_then(|leaf| self.leaf_type(leaf, context, scope));
                // A residue converts to its least residue, 0 through m - 1.
                let range = match from.as_ref().and_then(CoreType::modulus) {
                    Some(modulus) => modulus
                        .to_u64()
                        .and_then(|modulus| modulus.checked_sub(1))
                        .map(|high| (0, u128::from(high))),
                    None => from.as_ref().and_then(word_maximum).map(|from| {
                        self.word_range(&conversion.operand, u128::from(from), context, scope)
                    }),
                };
                range.filter(|(_, high)| *high <= maximum).unwrap_or(whole)
            }
            _ => whole,
        }
    }

    /// Converts a word range to exact integers, reporting a failed
    /// reservation as a resource limit at `span` and giving `None`.
    pub(super) fn exact_word_range(
        &mut self,
        span: Span,
        (low, high): WordRange,
    ) -> Option<IndexRange> {
        let exact = |value: u128| {
            u64::try_from(value)
                .ok()
                .and_then(|value| ExactInteger::from_u64(value, self.reserve_range_limbs))
        };
        let range = exact(low).zip(exact(high));
        if range.is_none() {
            self.resource_limit(span, "index range storage allocation failed");
        }
        range
    }

    /// Decodes a literal's magnitude without allocating, or gives `None`
    /// when it exceeds 128 bits.
    pub(super) fn literal_value(&self, literal: &IntegerLiteral) -> Option<u128> {
        let spelling = self.source.slice(literal.magnitude_span)?;
        let (radix, digits) = if let Some(digits) = spelling
            .strip_prefix("0b")
            .or_else(|| spelling.strip_prefix("0B"))
        {
            (2, digits)
        } else if let Some(digits) = spelling
            .strip_prefix("0x")
            .or_else(|| spelling.strip_prefix("0X"))
        {
            (16, digits)
        } else {
            (10, spelling)
        };
        digits
            .chars()
            .filter(|character| *character != '_')
            .try_fold(0_u128, |value, character| {
                value
                    .checked_mul(u128::from(radix))?
                    .checked_add(u128::from(character.to_digit(radix)?))
            })
    }
}
