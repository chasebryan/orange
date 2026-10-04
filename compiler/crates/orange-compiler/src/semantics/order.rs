//! Byte order: conversions that pack words into words of another width or
//! into a number, and unpack them again, in the order `big` or `little`
//! names.

use super::*;

/// The note of a conversion in a byte order between types it does not
/// convert.
const ORDER_NOTE: &str = "`as big` and `as little` convert a word or an array of words to words \
     of another width, to `Int`, or to `Mod[m]`, and back";

/// What one side of a conversion in a byte order is.
enum Side {
    /// Words: the width of each and their number.
    Words(u32, u32),
    /// `Int` or `Mod[m]`: the number that words spell.
    Number,
    /// Anything else, which no byte order converts.
    Other,
}

impl Side {
    fn of(ty: &CoreType) -> Self {
        match (ty.words(), ty) {
            (Some((bits, count)), _) => Self::Words(bits, count),
            (None, CoreType::Int | CoreType::Mod(_)) => Self::Number,
            (None, _) => Self::Other,
        }
    }
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Checks `operand as big Target` or `operand as little Target` against
    /// `expected`.
    ///
    /// The target is resolved and compared with `expected` first, as for
    /// every conversion. The operand's own type is the type of its first
    /// typed leaf; an array literal or a fill takes its elements' type and
    /// its length. One side is words, a word or an array of words, and the
    /// other is words of the same number of bits, `Int`, or `Mod[m]`.
    pub(super) fn check_packing(
        &mut self,
        expression: &'ast Expression,
        conversion: &'ast ConversionExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some((order, order_span)) = conversion.order else {
            return false;
        };
        // One event for the order's word, as for every other token.
        if !self.event(order_span) {
            return false;
        }
        let target = self.analyze_type(&conversion.target, "conversion type");
        if self.halted {
            return false;
        }
        let target_matches = match target.clone() {
            Some(target) if target != *expected => {
                self.report_conversion_mismatch(conversion.target.span, &target, expected);
                false
            }
            Some(_) => true,
            None => false,
        };
        let Some(leaf) = first_typed_leaf(&conversion.operand) else {
            self.report_untyped_conversion(&conversion.operand);
            return false;
        };
        let from = match &leaf.kind {
            ExpressionKind::Tuple(_) => {
                self.report_unpacked(conversion.keyword_span, order, "a tuple");
                return false;
            }
            ExpressionKind::Array(_) | ExpressionKind::Fill(_) => {
                let Some(from) = self.literal_array_type(leaf, context, scope) else {
                    self.report_untyped_conversion(&conversion.operand);
                    return false;
                };
                from
            }
            _ => {
                let Some(from) = self.leaf_type(leaf, context, scope) else {
                    // The leaf's own check reports why it has no type. That
                    // check stops before comparing with the type passed here.
                    self.check_untyped(leaf, expected, context, scope, output);
                    return false;
                };
                from
            }
        };
        // Like every undefined operator, a rejected conversion stops here:
        // its operand is checked only far enough to find its type.
        if matches!(Side::of(&from), Side::Other) {
            self.report_unpacked(conversion.keyword_span, order, &format!("`{from}`"));
            return false;
        }
        if let Some(target) = &target
            && !self.check_sides(conversion, order, &from, target)
        {
            return false;
        }
        let operand = self.check_expression(&conversion.operand, &from, context, scope, output);
        operand
            && target_matches
            && self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::Pack { from, order },
            )
    }

    /// Checks that a byte order converts `from` to `target`: words to words
    /// of the same number of bits, or words to or from a number.
    fn check_sides(
        &mut self,
        conversion: &ConversionExpression,
        order: ByteOrder,
        from: &CoreType,
        target: &CoreType,
    ) -> bool {
        match (Side::of(from), Side::of(target)) {
            (_, Side::Other) => {
                self.report_unpacking(conversion.target.span, order, target);
                false
            }
            (Side::Number, Side::Number) => {
                self.report_no_words(conversion, order, from, target);
                false
            }
            (Side::Words(from_bits, from_count), Side::Words(bits, count)) => {
                let width =
                    |bits: u32, count: u32| u64::from(bits).saturating_mul(u64::from(count));
                let (from_width, width) = (width(from_bits, from_count), width(bits, count));
                if from_width == width {
                    true
                } else {
                    self.report_packed_width(conversion, (from, from_width), (target, width));
                    false
                }
            }
            (Side::Words(..) | Side::Number | Side::Other, _) => true,
        }
    }

    /// Returns the type of an array literal or a fill as the operand of a
    /// conversion, without reporting: its elements' type, from the first
    /// element with a known type, and its length. Nested literals preserve
    /// their row type; the rank and scalar-product bounds are enforced by
    /// the array constructor at every level.
    pub(super) fn literal_array_type(
        &self,
        leaf: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<CoreType> {
        let element_type = |element| {
            let typed = first_typed_leaf(element)?;
            match &typed.kind {
                ExpressionKind::Array(_) | ExpressionKind::Fill(_) => {
                    self.literal_array_type(typed, context, scope)
                }
                _ => self.leaf_type(typed, context, scope),
            }
        };
        let element = match &leaf.kind {
            ExpressionKind::Array(array) => array.elements.iter().find_map(element_type),
            ExpressionKind::Fill(fill) => element_type(&fill.element),
            _ => None,
        }?;
        let length = self.array_length_of(leaf, context, scope)?;
        ArrayType::new(&element, length).map(CoreType::Array)
    }

    // The reports of a conversion in a byte order are out of line, so that
    // the frame of `check_packing`, which nested conversions stack, stays
    // small.
    #[cold]
    #[inline(never)]
    fn report_unpacked(&mut self, span: Span, order: ByteOrder, operand: &str) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    format!("`as {}` does not convert {operand}", order.as_str()),
                    span,
                )
                .with_label("a byte order packs and unpacks words")
                .with_note(ORDER_NOTE),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_unpacking(&mut self, span: Span, order: ByteOrder, target: &CoreType) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    format!("`as {}` does not convert to `{target}`", order.as_str()),
                    span,
                )
                .with_label("a byte order packs and unpacks words")
                .with_note(ORDER_NOTE),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_no_words(
        &mut self,
        conversion: &ConversionExpression,
        order: ByteOrder,
        from: &CoreType,
        target: &CoreType,
    ) {
        let span = conversion.order_span().unwrap_or(conversion.keyword_span);
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedOperator,
                    format!(
                        "`{}` orders words, but this converts `{from}` to `{target}`",
                        order.as_str()
                    ),
                    span,
                )
                .with_label("neither side is a word or an array of words")
                .with_note(format!(
                    "a number converts to another without a byte order, as `x as {target}`"
                )),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_packed_width(
        &mut self,
        conversion: &ConversionExpression,
        (from, from_width): (&CoreType, u64),
        (target, width): (&CoreType, u64),
    ) {
        let span = conversion.target.span;
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::PackedWidth,
                    format!("`{from}` and `{target}` have different widths"),
                    span,
                )
                .with_label(format!("`{target}` has {width} bits"))
                .with_secondary_span(
                    conversion.operand.span,
                    format!("`{from}` has {from_width} bits"),
                )
                .with_note(
                    "a byte order keeps every bit of the words it converts, so words convert \
                     only to words of the same number of bits",
                ),
            );
        }
    }
}
