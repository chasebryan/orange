//! Sizes: the size parameters of a sized `spec`, the instances they give,
//! and the value of each size within one instance.

use super::*;
use crate::parser::MAX_PARAMETERS_PER_FUNCTION;

/// Most instances of one function: the product of the lengths of its size
/// parameters' ranges.
pub(super) const MAX_INSTANCES_PER_FUNCTION: usize = 256;

pub(super) const SIZE_NOTE: &str = "a size is fixed in each instance of its function: it is \
     built from integer literals and the function's size parameters with `+`, `-`, `*`, `/`, \
     `%`, and parentheses";

pub(super) const SIZE_RANGE_NOTE: &str = "a size parameter `n in a..b` takes each value from a \
     up to, but not including, b, with a < b <= 65536, and a function has at most 256 instances";

/// The instance of a function being checked: its size parameters and the
/// value of each. A function without size parameters has one instance,
/// with none.
#[derive(Clone, Copy)]
pub(super) struct Instance<'ast> {
    pub(super) parameters: &'ast [SizeParameter],
    pub(super) values: [u32; MAX_SIZES_PER_FUNCTION],
}

impl<'ast> Instance<'ast> {
    pub(super) const NONE: Self = Self {
        parameters: &[],
        values: [0; MAX_SIZES_PER_FUNCTION],
    };

    /// Returns the size parameter named `name` and its value, if the
    /// instance has one.
    pub(super) fn find(&self, name: &str) -> Option<(&'ast SizeParameter, u32)> {
        self.parameters
            .iter()
            .zip(self.values)
            .find(|(parameter, _)| parameter.name.text == name)
    }

    /// Returns the values of the instance's sizes, in declaration order.
    pub(super) fn values(&self) -> &[u32] {
        self.values.get(..self.parameters.len()).unwrap_or_default()
    }

    /// Returns the instance's name as a call writes it, `f[2]` or `f[1, 3]`,
    /// or the function's name alone when it has no sizes.
    pub(super) fn label(&self, name: &str) -> String {
        let mut label = identifier_spelling_for_diagnostic(name).to_string();
        if !self.parameters.is_empty() {
            label.push('[');
            for (position, value) in self.values().iter().enumerate() {
                if position != 0 {
                    label.push_str(", ");
                }
                label.push_str(&value.to_string());
            }
            label.push(']');
        }
        label
    }
}

/// Splits an integer literal's spelling into its radix and its digits.
pub(super) fn split_radix(spelling: &str) -> (u32, &str) {
    if let Some(digits) = spelling
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
    }
}

/// Decodes a size parameter's bound, written as any integer literal, or
/// returns `None` when it exceeds [`MAX_LOOP_BOUND`].
pub(super) fn size_bound(source: &SourceFile, span: Span) -> Option<u32> {
    let (radix, digits) = split_radix(source.slice(span)?);
    let mut value = 0_u32;
    for character in digits.chars() {
        if character == '_' {
            continue;
        }
        value = value
            .checked_mul(radix)?
            .checked_add(character.to_digit(radix)?)?;
        if value > MAX_LOOP_BOUND {
            return None;
        }
    }
    Some(value)
}

/// The ranges of a function's size parameters, each `(a, b)` for
/// `n in a..b`, and the number of instances they give.
#[derive(Clone, Copy)]
pub(super) struct SizeRanges {
    ranges: [(u32, u32); MAX_SIZES_PER_FUNCTION],
    count: usize,
    instances: usize,
}

impl SizeRanges {
    /// Returns the ranges of `function`'s size parameters when each is a
    /// nonempty range within 0 through 65536 and together they give at
    /// most [`MAX_INSTANCES_PER_FUNCTION`] instances. A function without
    /// size parameters has one instance.
    pub(super) fn of(source: &SourceFile, function: &FunctionDeclaration) -> Option<Self> {
        let mut ranges = [(0, 0); MAX_SIZES_PER_FUNCTION];
        if function.sizes.len() > MAX_SIZES_PER_FUNCTION {
            return None;
        }
        let mut instances = 1_usize;
        for (slot, size) in ranges.iter_mut().zip(&function.sizes) {
            let start = size_bound(source, size.start_span)?;
            let end = size_bound(source, size.end_span)?;
            let width = usize::try_from(end.checked_sub(start)?).ok()?;
            instances = instances
                .checked_mul(width)
                .filter(|count| (1..=MAX_INSTANCES_PER_FUNCTION).contains(count))?;
            *slot = (start, end);
        }
        Some(Self {
            ranges,
            count: function.sizes.len(),
            instances,
        })
    }

    /// Returns the number of instances.
    pub(super) const fn instances(&self) -> usize {
        self.instances
    }

    /// Returns the number of size parameters.
    pub(super) const fn count(&self) -> usize {
        self.count
    }

    /// Returns the range of the size parameter at `position`.
    pub(super) fn range(&self, position: usize) -> Option<(u32, u32)> {
        self.ranges
            .get(..self.count)
            .and_then(|ranges| ranges.get(position))
            .copied()
    }

    /// Returns the instance at `index` in ascending order of its values,
    /// the first size changing slowest.
    pub(super) fn instance<'ast>(
        &self,
        parameters: &'ast [SizeParameter],
        index: usize,
    ) -> Option<Instance<'ast>> {
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        let mut rest = index;
        for position in (0..self.count).rev() {
            let (start, end) = self.range(position)?;
            let width = usize::try_from(end.checked_sub(start)?).ok()?;
            let offset = u32::try_from(rest.checked_rem(width)?).ok()?;
            rest = rest.checked_div(width)?;
            *values.get_mut(position)? = start.checked_add(offset)?;
        }
        (rest == 0).then_some(Instance { parameters, values })
    }

    /// Returns the index of the instance whose sizes have `values`, which
    /// are within their ranges.
    pub(super) fn index(&self, values: &[u32]) -> Option<usize> {
        if values.len() != self.count {
            return None;
        }
        let mut index = 0_usize;
        for (position, value) in values.iter().enumerate() {
            let (start, end) = self.range(position)?;
            if !(start..end).contains(value) {
                return None;
            }
            let width = usize::try_from(end.checked_sub(start)?).ok()?;
            let offset = usize::try_from(value.checked_sub(start)?).ok()?;
            index = index.checked_mul(width)?.checked_add(offset)?;
        }
        Some(index)
    }
}

/// The number of Core function identities a function takes: one for each
/// instance of a typed `spec`, none for a function without a typed body or
/// for a `spec` whose size parameters are malformed.
pub(super) fn instance_count(source: &SourceFile, function: &FunctionDeclaration) -> usize {
    match (&function.body, function.kind) {
        (FunctionBody::Typed(_), FunctionKind::Spec) => {
            SizeRanges::of(source, function).map_or(0, |ranges| ranges.instances())
        }
        _ => 0,
    }
}

/// Why a size has no value.
#[derive(Clone, Copy)]
pub(super) enum SizeFault {
    /// This part is neither an integer literal, a size parameter, nor an
    /// operation on them.
    NotStatic(Span),
    /// This part's value exceeds the significant-bit limit of `Int`.
    TooLarge(Span),
    /// Storage could not be reserved at this part.
    Storage(Span),
}

/// The sizes of the instance being checked and how their values are
/// computed. Every part of a size evaluated is counted, and the analyzer
/// charges one semantic event for each.
pub(super) struct SizeScope<'ast> {
    pub(super) instance: Instance<'ast>,
    pub(super) bits: usize,
    pub(super) reserve: fn(&mut Vec<u32>, usize) -> bool,
    pub(super) reserve_limb: fn(&mut Vec<u32>) -> bool,
    pub(super) evaluated: Cell<usize>,
}

impl SizeScope<'_> {
    /// Returns the exact value of a size expression: integer literals and
    /// the instance's sizes with `+`, `-`, `*`, `/`, `%`, prefix `-`, and
    /// parentheses, where `/` and `%` are Euclidean and total, as for `Int`.
    /// A part that is not static is reported before a part too large,
    /// wherever each stands.
    ///
    /// Parser-established expression height bounds this recursion.
    pub(super) fn value(
        &self,
        source: &SourceFile,
        expression: &Expression,
    ) -> Result<ExactInteger, SizeFault> {
        self.evaluated.set(self.evaluated.get().saturating_add(1));
        let storage = SizeFault::Storage(expression.span);
        let value = match &expression.kind {
            ExpressionKind::Literal(literal) => self.literal(source, literal)?,
            ExpressionKind::Name(name) => match self.instance.find(&name.text) {
                Some((_, value)) => {
                    ExactInteger::from_u64(u64::from(value), self.reserve).ok_or(storage)?
                }
                None => return Err(SizeFault::NotStatic(name.span)),
            },
            ExpressionKind::Parenthesized(inner) => return self.value(source, inner),
            ExpressionKind::Unary(unary) if unary.operator == UnaryOperator::Negate => {
                self.value(source, &unary.operand)?.negated()
            }
            ExpressionKind::Binary(binary)
                if matches!(
                    binary.operator,
                    BinaryOperator::Add
                        | BinaryOperator::Subtract
                        | BinaryOperator::Multiply
                        | BinaryOperator::Divide
                        | BinaryOperator::Remainder
                ) =>
            {
                let (left, right) = match (
                    self.value(source, &binary.left),
                    self.value(source, &binary.right),
                ) {
                    (Ok(left), Ok(right)) => (left, right),
                    (Err(SizeFault::TooLarge(_)), Err(fault))
                    | (Err(fault), _)
                    | (_, Err(fault)) => {
                        return Err(fault);
                    }
                };
                let reserve = self.reserve;
                match binary.operator {
                    BinaryOperator::Add => left.add(&right, reserve),
                    BinaryOperator::Subtract => left.subtract(&right, reserve),
                    BinaryOperator::Multiply => left.multiply(&right, reserve),
                    // As for `Int`, x / 0 is 0 and x % 0 is x.
                    BinaryOperator::Divide if right.is_zero() => ExactInteger::from_u64(0, reserve),
                    BinaryOperator::Divide => left
                        .divide_euclid(&right, reserve)
                        .map(|(quotient, _)| quotient),
                    _ if right.is_zero() => Some(left),
                    _ => left
                        .divide_euclid(&right, reserve)
                        .map(|(_, remainder)| remainder),
                }
                .ok_or(storage)?
            }
            _ => return Err(SizeFault::NotStatic(expression.span)),
        };
        if value.magnitude_bits() > self.bits {
            return Err(SizeFault::TooLarge(expression.span));
        }
        Ok(value)
    }

    /// Decodes an integer literal of a size, whose magnitude may have at
    /// most the significant bits of `Int`.
    fn literal(
        &self,
        source: &SourceFile,
        literal: &IntegerLiteral,
    ) -> Result<ExactInteger, SizeFault> {
        let storage = SizeFault::Storage(literal.span);
        let spelling = source.slice(literal.magnitude_span).ok_or(storage)?;
        let (radix, digits) = split_radix(spelling);
        let mut magnitude = Magnitude::zero();
        for character in digits.chars() {
            if character == '_' {
                continue;
            }
            let digit = character
                .to_digit(radix)
                .ok_or(SizeFault::Storage(literal.span))?;
            if !magnitude.multiply_add_with_reservation(radix, digit, self.reserve_limb) {
                return Err(SizeFault::Storage(literal.span));
            }
            if magnitude.bit_len() > self.bits {
                return Err(SizeFault::TooLarge(literal.span));
            }
        }
        Ok(ExactInteger::new(literal.negative, magnitude))
    }

    /// Classifies an array length: an integer token as S3d writes it, or
    /// a size expression whose value must be an admitted length.
    pub(super) fn array_length(&self, source: &SourceFile, size: &Size) -> Length {
        let Some(expression) = size.expression() else {
            return array_length(source, size.span)
                .filter(|length| (1..=MAX_ARRAY_LENGTH).contains(length))
                .map_or(Length::Literal, Length::Admitted);
        };
        match self.value(source, expression) {
            Ok(value) => value
                .to_i64()
                .and_then(|value| u32::try_from(value).ok())
                .filter(|length| (1..=MAX_ARRAY_LENGTH).contains(length))
                .map_or_else(|| Length::Value(value), Length::Admitted),
            Err(fault) => Length::Fault(fault),
        }
    }
}

/// What an array length stands for.
pub(super) enum Length {
    /// An admitted length, from 1 through 256.
    Admitted(u32),
    /// An integer token that is not a decimal length from 1 through 256.
    Literal,
    /// A size expression whose value is not an admitted length.
    Value(ExactInteger),
    /// A size expression with no value.
    Fault(SizeFault),
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Enters the instance whose types and sizes are resolved next, with the
    /// analyzer's limits and reservations.
    pub(super) fn enter_instance(&mut self, instance: Instance<'ast>) {
        // Sizes evaluated without a report, such as a fill's length read to
        // infer a join's length, are charged before the scope changes.
        self.charge_size_events(self.ast.module.span);
        self.types.sizes = SizeScope {
            instance,
            bits: self.limits.integer_bits,
            reserve: self.reserve_range_limbs,
            reserve_limb: self.reserve_magnitude_limb,
            evaluated: Cell::new(0),
        };
    }

    /// Charges one semantic event for each part of a size evaluated since
    /// the last charge.
    pub(super) fn charge_size_events(&mut self, span: Span) -> bool {
        let evaluated = self.types.sizes.evaluated.take();
        (0..evaluated).all(|_| self.event(span))
    }

    /// Returns the value of a size expression in the instance being checked,
    /// or reports why it has none.
    pub(super) fn size_value(&mut self, expression: &Expression) -> Option<ExactInteger> {
        let value = self.types.sizes.value(self.source, expression);
        if !self.charge_size_events(expression.span) {
            return None;
        }
        value.map_err(|fault| self.report_size_fault(fault)).ok()
    }

    #[cold]
    #[inline(never)]
    pub(super) fn report_size_fault(&mut self, fault: SizeFault) {
        match fault {
            SizeFault::NotStatic(span) => {
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::NonStaticSize,
                            "a size may use only integer literals and size parameters",
                            span,
                        )
                        .with_label("this is neither")
                        .with_note(SIZE_NOTE),
                    );
                }
            }
            SizeFault::TooLarge(span) => {
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::IntegerMagnitudeLimit,
                            format!(
                                "integer magnitude exceeds the {}-significant-bit limit",
                                self.limits.integer_bits
                            ),
                            span,
                        )
                        .with_label("this part of the size is too large")
                        .with_note("the value is rejected rather than truncated or approximated"),
                    );
                }
            }
            SizeFault::Storage(span) => self.resource_limit(span, "size storage allocation failed"),
        }
    }

    /// Checks a function's size parameters: each range nonempty and within
    /// 0 through 65536, at most 256 instances in all, and every name
    /// distinct from the other sizes' and the parameters'. Returns whether
    /// the function's instances can be checked.
    pub(super) fn check_size_parameters(&mut self, function: &'ast FunctionDeclaration) -> bool {
        let mut valid = true;
        let mut instances = 1_usize;
        for (index, size) in function.sizes.iter().enumerate() {
            // One event for the size's range and one for its name.
            if !self.event(size.span) || !self.event(size.name.span) {
                return false;
            }
            let start = size_bound(self.source, size.start_span);
            let end = size_bound(self.source, size.end_span);
            match (start, end) {
                (Some(start), Some(end)) if start < end => {
                    let width = usize::try_from(end.abs_diff(start)).unwrap_or(usize::MAX);
                    instances = instances.saturating_mul(width);
                }
                (Some(start), Some(end)) => {
                    valid = false;
                    self.report_size_range(
                        size.end_span,
                        format!("the size range {start}..{end} is empty"),
                        "a size takes at least one value",
                    );
                }
                (start, _) => {
                    valid = false;
                    self.report_size_range(
                        if start.is_none() {
                            size.start_span
                        } else {
                            size.end_span
                        },
                        format!("a size's bound must be at most {MAX_LOOP_BOUND}"),
                        "bound too large",
                    );
                }
            }
            let earlier = function.sizes.get(..index).and_then(|earlier| {
                earlier
                    .iter()
                    .find(|other| other.name.text == size.name.text)
            });
            if let Some(earlier) = earlier {
                valid = false;
                self.report_repeated_size(
                    &size.name,
                    earlier.name.span,
                    "first size parameter is here",
                );
            }
        }
        for parameter in &function.parameters {
            if let Some(size) = function
                .sizes
                .iter()
                .find(|size| size.name.text == parameter.name.text)
            {
                valid = false;
                self.report_repeated_size(
                    &parameter.name,
                    size.name.span,
                    "the size parameter is here",
                );
            }
        }
        if valid && instances > MAX_INSTANCES_PER_FUNCTION {
            valid = false;
            if let (Some(first), Some(last)) = (function.sizes.first(), function.sizes.last()) {
                let span = self
                    .source
                    .span(first.span.start(), last.span.end())
                    .unwrap_or(first.span);
                let name = identifier_spelling_for_diagnostic(&function.name.text);
                self.report_size_range(
                    span,
                    format!(
                        "`{name}` has {instances} instances, but a function has at most \
                         {MAX_INSTANCES_PER_FUNCTION}"
                    ),
                    "too many instances",
                );
            }
        }
        valid
    }

    #[cold]
    #[inline(never)]
    fn report_size_range(&mut self, span: Span, message: String, label: &str) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(DiagnosticCode::SizeRange, message, span)
                    .with_label(label)
                    .with_note(SIZE_RANGE_NOTE),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_repeated_size(&mut self, name: &Identifier, earlier: Span, label: &str) {
        if self.begin_report(name.span) {
            let spelling = identifier_spelling_for_diagnostic(&name.text);
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::DuplicateParameter,
                    format!("duplicate parameter `{spelling}`"),
                    name.span,
                )
                .with_label("this name is already a size parameter")
                .with_secondary_span(earlier, label)
                .with_note(
                    "size parameters and parameters share one namespace, and each name is unique",
                ),
            );
        }
    }

    /// Names the instance of a sized function in each ordinary diagnostic
    /// reported since `reported` diagnostics were retained.
    pub(super) fn name_instance(
        &mut self,
        function: &FunctionDeclaration,
        instance: Instance<'ast>,
        reported: usize,
    ) {
        if function.sizes.is_empty() {
            return;
        }
        let label = instance.label(&function.name.text);
        let name = identifier_spelling_for_diagnostic(&function.name.text);
        for diagnostic in self.diagnostics.iter_mut().skip(reported) {
            if !matches!(
                diagnostic.code(),
                DiagnosticCode::TooManySemanticErrors | DiagnosticCode::SemanticResourceLimit
            ) {
                diagnostic.add_note(format!(
                    "in the instance `{label}`, the first of `{name}` in error: a sized \
                     function is checked once for each value of its sizes"
                ));
            }
        }
    }

    /// Returns the instance a call names and its signature: the call gives
    /// one size for each of the callee's size parameters, and each size's
    /// value, computed in the caller's instance, lies in its range. Reports
    /// why otherwise. A callee whose sizes are malformed was reported at its
    /// declaration and is not reported again.
    pub(super) fn called_instance<'signature>(
        &mut self,
        expression: &Expression,
        call: &CallExpression,
        signature: &'signature Signature<'_>,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<(CoreFunctionId, &'signature InstanceSignature)> {
        let ranges = signature.ranges?;
        if call.sizes().is_empty() && ranges.count() != 0 {
            // Every instance takes the same number of arguments; a call that
            // gives another number is reported as such by its caller.
            let first = signature.instances.first()?;
            if first.parameters.len() != call.arguments.len() {
                return Some((signature.instance_id(0)?, first));
            }
            return match self.fitting_instance(call, signature, context, scope) {
                Ok(index) => Some((
                    signature.instance_id(index)?,
                    signature.instances.get(index)?,
                )),
                Err(fitting) => {
                    self.report_unfitted_call(
                        expression.span,
                        call,
                        signature,
                        fitting,
                        context,
                        scope,
                    );
                    None
                }
            };
        }
        if call.sizes().len() != ranges.count() {
            self.report_size_count(expression.span, call, ranges.count());
            return None;
        }
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        for ((position, size), slot) in call.sizes().iter().enumerate().zip(&mut values) {
            let value = self.size_value(size)?;
            let (start, end) = ranges.range(position)?;
            match value
                .to_i64()
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| (start..end).contains(value))
            {
                Some(value) => *slot = value,
                None => {
                    let parameter = signature.sizes.get(position)?;
                    self.report_size_outside(
                        size.span,
                        &call.callee,
                        parameter,
                        (start, end),
                        &value,
                    );
                    return None;
                }
            }
        }
        let index = ranges.index(values.get(..ranges.count())?)?;
        Some((
            signature.instance_id(index)?,
            signature.instances.get(index)?,
        ))
    }

    /// Returns the index of the one instance of a sized function whose array
    /// parameters have the lengths of a call's arguments, for a call that
    /// writes no sizes. An argument whose length is not known without
    /// reporting fits every instance. Otherwise returns the indices of the
    /// first two instances that fit, or none, without reporting.
    fn fitting_instance(
        &self,
        call: &CallExpression,
        signature: &Signature<'_>,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Result<usize, [Option<usize>; 2]> {
        let mut lengths = [None; MAX_PARAMETERS_PER_FUNCTION];
        for ((length, argument), parameter) in lengths
            .iter_mut()
            .zip(&call.arguments)
            .zip(&signature.instances.first().ok_or([None; 2])?.parameters)
        {
            if parameter.as_ref().and_then(CoreType::as_array).is_some() {
                *length = self.array_length_of(argument, context, scope);
            }
        }
        let mut fitting = [None; 2];
        let mut found = 0_usize;
        for (index, instance) in signature.instances.iter().enumerate() {
            let fits = instance
                .parameters
                .iter()
                .zip(lengths)
                .all(|(parameter, length)| {
                    match (parameter.as_ref().and_then(CoreType::as_array), length) {
                        (Some(array), Some(length)) => array.length() == length,
                        _ => true,
                    }
                });
            if fits {
                if let Some(slot) = fitting.get_mut(found) {
                    *slot = Some(index);
                }
                found = found.saturating_add(1);
                if found > 1 {
                    break;
                }
            }
        }
        match fitting {
            [Some(index), None] => Ok(index),
            _ => Err(fitting),
        }
    }

    /// Reports a call without sizes whose arguments' lengths fit no instance
    /// of the callee, or more than one.
    #[cold]
    #[inline(never)]
    fn report_unfitted_call(
        &mut self,
        span: Span,
        call: &CallExpression,
        signature: &Signature<'_>,
        fitting: [Option<usize>; 2],
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) {
        let Some(ranges) = signature.ranges else {
            return;
        };
        if !self.begin_report(span) {
            return;
        }
        let name = identifier_spelling_for_diagnostic(&call.callee.text);
        let domain = signature
            .sizes
            .iter()
            .enumerate()
            .filter_map(|(position, size)| {
                let (start, end) = ranges.range(position)?;
                Some(format!(
                    "`{}` in {start}..{end}",
                    identifier_spelling_for_diagnostic(&size.name.text)
                ))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let diagnostic = if let [Some(first), Some(second)] = fitting {
            let label = |index: usize| {
                ranges
                    .instance(signature.sizes, index)
                    .map_or_else(String::new, |instance| instance.label(&call.callee.text))
            };
            Diagnostic::error(
                DiagnosticCode::SizeCount,
                format!(
                    "this call fits more than one instance of `{name}`, among them `{}` and `{}`",
                    label(first),
                    label(second)
                ),
                span,
            )
            .with_label("write the sizes in brackets")
        } else {
            let lengths = call
                .arguments
                .iter()
                .filter_map(|argument| self.array_length_of(argument, context, scope))
                .map(|length| length.to_string())
                .collect::<Vec<_>>();
            let label = match lengths.as_slice() {
                [] => String::from("no instance fits these arguments"),
                [length] => format!("an array of length {length} is given"),
                _ => format!("arrays of lengths {} are given", lengths.join(", ")),
            };
            Diagnostic::error(
                DiagnosticCode::SizeRange,
                format!("no instance of `{name}` takes arguments of these lengths"),
                span,
            )
            .with_label(label)
            .with_note(format!("`{name}` is defined for {domain}"))
        };
        self.diagnostics.push(diagnostic.with_note(
            "a call that writes no sizes calls the one instance of its function whose array \
             parameters have the lengths of its arguments; any other call writes its sizes in \
             brackets, as in `absorb[2](p)`",
        ));
    }

    #[cold]
    #[inline(never)]
    fn report_size_count(&mut self, span: Span, call: &CallExpression, declared: usize) {
        if !self.begin_report(span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(&call.callee.text);
        let given = call.sizes().len();
        let plural = |count: usize| if count == 1 { "size" } else { "sizes" };
        let message = match (declared, given) {
            (0, _) => format!(
                "`{spelling}` has no size parameters, but this call gives {given} {}",
                plural(given)
            ),
            (_, 0) => format!(
                "`{spelling}` takes {declared} {}, but this call gives none",
                plural(declared)
            ),
            _ => format!(
                "`{spelling}` takes {declared} {}, but this call gives {given}",
                plural(declared)
            ),
        };
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::SizeCount, message, span)
                .with_label("wrong number of sizes")
                .with_note(
                    "a sized function is called with one value for each of its sizes, in brackets \
                     before its arguments, as in `sha256[2](m)`; a function without sizes is \
                     called without brackets",
                ),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_size_outside(
        &mut self,
        span: Span,
        callee: &Identifier,
        parameter: &SizeParameter,
        (start, end): (u32, u32),
        value: &ExactInteger,
    ) {
        if !self.begin_report(span) {
            return;
        }
        let function = identifier_spelling_for_diagnostic(&callee.text);
        let size = identifier_spelling_for_diagnostic(&parameter.name.text);
        let label = value.to_i64().map_or_else(
            || String::from("this size is far outside that range"),
            |value| format!("this size is {value}"),
        );
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::SizeRange,
                format!("`{function}` is defined for `{size}` in {start}..{end}"),
                span,
            )
            .with_label(label)
            .with_note(SIZE_RANGE_NOTE),
        );
    }

    /// Returns the array length written as `size`, or reports why it is
    /// not one.
    pub(super) fn checked_length(&mut self, size: &Size) -> Option<u32> {
        let length = self.types.sizes.array_length(self.source, size);
        if !self.charge_size_events(size.span) {
            return None;
        }
        match length {
            Length::Admitted(length) => Some(length),
            Length::Literal => {
                self.report_unsupported_array_length(size.span);
                None
            }
            Length::Value(value) => {
                self.report_array_length_value(size.span, &value);
                None
            }
            Length::Fault(fault) => {
                self.report_size_fault(fault);
                None
            }
        }
    }

    /// Returns the signature of the instance a call names, without
    /// reporting, or `None` when its sizes do not name one.
    pub(super) fn silent_instance<'signature>(
        &self,
        call: &CallExpression,
        signature: &'signature Signature<'_>,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<&'signature InstanceSignature> {
        let ranges = signature.ranges?;
        if call.sizes().is_empty() && ranges.count() != 0 {
            let index = self
                .fitting_instance(call, signature, context, scope)
                .ok()?;
            return signature.instances.get(index);
        }
        if call.sizes().len() != ranges.count() {
            return None;
        }
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        for (size, slot) in call.sizes().iter().zip(&mut values) {
            *slot = self
                .types
                .sizes
                .value(self.source, size)
                .ok()?
                .to_i64()
                .and_then(|value| u32::try_from(value).ok())?;
        }
        signature
            .instances
            .get(ranges.index(values.get(..ranges.count())?)?)
    }
}
