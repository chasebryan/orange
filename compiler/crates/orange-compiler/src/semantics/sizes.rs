//! Sizes and types: the size and type parameters of a `spec`, the
//! instances they give, and the value of each size and the type of each
//! type parameter within one instance.

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

/// The note of a call that gives a type parameter a type it does not list.
const TYPE_PARAMETER_CALL_NOTE: &str = "a call gives each type parameter one of the types it \
     lists, in brackets, as in `pow[F](x, e)`";

/// The note of an entry in a call's brackets that is not a type.
const TYPE_ARGUMENT_NOTE: &str = "a type in a call's brackets is `Int`, `Bool`, `Word[n]`, an \
     array of one of them such as `Word[8]^4`, the name of a `type` declaration, or a type \
     parameter of the calling function";

/// What a call's entry in brackets gives a type parameter.
enum TypeArgument {
    /// A type.
    Type(CoreType),
    /// A type that did not resolve, which was reported where it is written.
    Unresolved,
    /// Something other than a type.
    NotAType,
}

/// The instance of a function being checked: its size and type parameters
/// and the value of each, which for a type parameter is the position of its
/// type in the parameter's list. A function without size or type parameters
/// has one instance, with none.
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
    /// instance has one. A type parameter is not a size.
    pub(super) fn find(&self, name: &str) -> Option<(&'ast SizeParameter, u32)> {
        self.parameters
            .iter()
            .zip(self.values)
            .find(|(parameter, _)| !parameter.is_type() && parameter.name.text == name)
    }

    /// Returns the position among the brackets' parameters of the type
    /// parameter named `name`, if the instance has one.
    pub(super) fn find_type(&self, name: &str) -> Option<usize> {
        self.parameters
            .iter()
            .position(|parameter| parameter.is_type() && parameter.name.text == name)
    }

    /// Returns the values of the instance's sizes, in declaration order.
    pub(super) fn values(&self) -> &[u32] {
        self.values.get(..self.parameters.len()).unwrap_or_default()
    }

    /// Returns the instance's name as a call writes it, `f[2]`, `f[1, 3]`,
    /// or `f[F]`, or the function's name alone when it has no sizes or
    /// types. `spellings` holds each parameter's listed types as written.
    pub(super) fn label(&self, name: &str, spellings: &[Vec<String>]) -> String {
        let mut label = identifier_spelling_for_diagnostic(name).to_string();
        label.push_str(&self.suffix(spellings));
        label
    }

    /// Returns the instance's sizes and types in brackets as a call writes
    /// them, `[2]` or `[1, F]`, or an empty string when the function has
    /// none.
    pub(super) fn suffix(&self, spellings: &[Vec<String>]) -> String {
        let mut suffix = String::new();
        if !self.parameters.is_empty() {
            suffix.push('[');
            for (position, (parameter, value)) in
                self.parameters.iter().zip(self.values()).enumerate()
            {
                if position != 0 {
                    suffix.push_str(", ");
                }
                if parameter.is_type() {
                    let spelling = spellings
                        .get(position)
                        .and_then(|list| list.get(usize::try_from(*value).ok()?));
                    suffix.push_str(spelling.map_or("?", String::as_str));
                } else {
                    suffix.push_str(&value.to_string());
                }
            }
            suffix.push(']');
        }
        suffix
    }
}

/// Returns each listed type of `parameters` as written, with every run of
/// whitespace written as one space, or an empty list for a size parameter.
pub(super) fn type_spellings(source: &SourceFile, parameters: &[SizeParameter]) -> Vec<Vec<String>> {
    parameters
        .iter()
        .map(|parameter| {
            parameter
                .types
                .iter()
                .map(|ty| {
                    source
                        .slice(ty.span)
                        .map_or_else(String::new, |text| {
                            text.split_whitespace().collect::<Vec<_>>().join(" ")
                        })
                })
                .collect()
        })
        .collect()
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
/// `n in a..b`, and `(0, k)` for a type parameter that lists k types, and
/// the number of instances they give.
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
            let (start, end) = if size.is_type() {
                (0, u32::try_from(size.types.len()).ok()?)
            } else {
                (
                    size_bound(source, size.start_span)?,
                    size_bound(source, size.end_span)?,
                )
            };
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
    /// The type of each type parameter in the instance, by its position
    /// among the brackets' parameters, or `None` for a size or a type that
    /// did not resolve.
    pub(super) types: [Option<CoreType>; MAX_SIZES_PER_FUNCTION],
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
                    // A part that is not static is reported before a part
                    // too large, and otherwise the leftmost fault is.
                    (Err(SizeFault::TooLarge(_)), Err(fault @ SizeFault::NotStatic(_)))
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
    /// analyzer's limits and reservations. Each type parameter stands for
    /// its listed type at the instance's position, resolved outside every
    /// instance, as the module's own types are.
    pub(super) fn enter_instance(&mut self, instance: Instance<'ast>) {
        // Sizes evaluated without a report, such as a fill's length read to
        // infer a join's length, are charged before the scope changes.
        self.charge_size_events(self.ast.module.span);
        self.types.sizes = self.size_scope(Instance::NONE, [const { None }; MAX_SIZES_PER_FUNCTION]);
        let mut types = [const { None }; MAX_SIZES_PER_FUNCTION];
        for ((slot, parameter), value) in types
            .iter_mut()
            .zip(instance.parameters)
            .zip(instance.values)
        {
            if parameter.is_type() {
                *slot = usize::try_from(value)
                    .ok()
                    .and_then(|position| parameter.types.get(position))
                    .and_then(|ty| silent_type(self.source, &self.types, ty));
            }
        }
        self.charge_size_events(self.ast.module.span);
        self.types.sizes = self.size_scope(instance, types);
    }

    fn size_scope(
        &self,
        instance: Instance<'ast>,
        types: [Option<CoreType>; MAX_SIZES_PER_FUNCTION],
    ) -> SizeScope<'ast> {
        SizeScope {
            instance,
            types,
            bits: self.limits.integer_bits,
            reserve: self.reserve_range_limbs,
            reserve_limb: self.reserve_magnitude_limb,
            evaluated: Cell::new(0),
        }
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

    /// Checks a function's size and type parameters: each range nonempty
    /// and within 0 through 65536, each type parameter's types resolved and
    /// distinct and its name no other type's, at most 256 instances in all,
    /// and every name distinct from the other parameters' in brackets and
    /// each size's from the parameters'. Returns whether the function's
    /// instances can be checked.
    pub(super) fn check_size_parameters(&mut self, function: &'ast FunctionDeclaration) -> bool {
        let mut valid = true;
        let mut instances = 1_usize;
        for (index, size) in function.sizes.iter().enumerate() {
            // One event for the size's range and one for its name.
            if !self.event(size.span) || !self.event(size.name.span) {
                return false;
            }
            let earlier = function.sizes.get(..index).and_then(|earlier| {
                earlier
                    .iter()
                    .find(|other| other.name.text == size.name.text)
            });
            if size.is_type() {
                valid &= self.check_type_parameter(size);
                if self.halted {
                    return false;
                }
                instances = instances.saturating_mul(size.types.len());
                if let Some(earlier) = earlier {
                    valid = false;
                    self.report_repeated_bracket(&size.name, earlier.name.span);
                }
                continue;
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
            if let Some(earlier) = earlier {
                valid = false;
                if earlier.is_type() {
                    self.report_repeated_bracket(&size.name, earlier.name.span);
                } else {
                    self.report_repeated_size(
                        &size.name,
                        earlier.name.span,
                        "first size parameter is here",
                    );
                }
            }
        }
        // A type parameter names a type, so it shares no namespace with the
        // parameters, which name values.
        for parameter in &function.parameters {
            if let Some(size) = function
                .sizes
                .iter()
                .find(|size| !size.is_type() && size.name.text == parameter.name.text)
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
                let message = format!(
                    "`{name}` has {instances} instances, but a function has at most \
                     {MAX_INSTANCES_PER_FUNCTION}"
                );
                if function.sizes.iter().any(SizeParameter::is_type) {
                    self.report_instance_count(span, message);
                } else {
                    self.report_size_range(span, message, "too many instances");
                }
            }
        }
        valid
    }

    /// Checks one type parameter `K in {F, L}`: its name names no built-in
    /// or declared type, and its listed types resolve, outside every
    /// instance, to distinct types. Reports each fault and returns whether
    /// there was none.
    fn check_type_parameter(&mut self, parameter: &'ast SizeParameter) -> bool {
        let mut valid = true;
        let name = parameter.name.text.as_str();
        let declared = self.types.name(name).map(|declared| declared.span);
        if BUILT_IN_TYPE_NAMES.contains(&name) || declared.is_some() {
            valid = false;
            self.report_type_parameter_name(&parameter.name, declared);
        }
        let mut resolved: Vec<Option<CoreType>> = Vec::new();
        if resolved.try_reserve_exact(parameter.types.len()).is_err() {
            self.resource_limit(parameter.span, "type parameter storage allocation failed");
            return false;
        }
        for ty in &parameter.types {
            let checked = self.analyze_type(ty, "listed type");
            if self.halted {
                return false;
            }
            valid &= checked.is_some();
            if let Some(checked) = &checked
                && let Some(first) = resolved
                    .iter()
                    .position(|earlier| earlier.as_ref() == Some(checked))
                    .and_then(|position| parameter.types.get(position))
            {
                valid = false;
                self.report_listed_twice(parameter, first.span, ty.span, checked);
            }
            resolved.push(checked);
        }
        valid
    }

    #[cold]
    #[inline(never)]
    fn report_type_parameter_name(&mut self, name: &Identifier, declared: Option<Span>) {
        if !self.begin_report(name.span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(&name.text);
        let diagnostic = match declared {
            Some(first) => Diagnostic::error(
                DiagnosticCode::DuplicateTypeName,
                format!("duplicate type name `{spelling}`"),
                name.span,
            )
            .with_label("this type parameter repeats a declared type's name")
            .with_secondary_span(first, "the `type` declaration is here"),
            None => Diagnostic::error(
                DiagnosticCode::DuplicateTypeName,
                format!("`{spelling}` is a built-in type"),
                name.span,
            )
            .with_label("a type parameter cannot name a built-in type"),
        };
        self.diagnostics.push(diagnostic.with_note(
            "a type parameter names a type of its own, so its name is not a built-in type's or a \
             `type` declaration's",
        ));
    }

    #[cold]
    #[inline(never)]
    fn report_listed_twice(
        &mut self,
        parameter: &SizeParameter,
        first: Span,
        again: Span,
        ty: &CoreType,
    ) {
        if !self.begin_report(again) {
            return;
        }
        let name = identifier_spelling_for_diagnostic(&parameter.name.text);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeParameter,
                format!("`{name}` lists the type `{ty}` twice"),
                again,
            )
            .with_label("this is the same type as an earlier one")
            .with_secondary_span(first, "first listed here")
            .with_note(
                "a type parameter lists each type once, so that each instance has a type of its \
                 own",
            ),
        );
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
    fn report_instance_count(&mut self, span: Span, message: String) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(DiagnosticCode::SizeRange, message, span)
                    .with_label("too many instances")
                    .with_note(
                        "a function has one instance for each combination of its sizes' values \
                         and its type parameters' types, at most 256 in all",
                    ),
            );
        }
    }

    #[cold]
    #[inline(never)]
    fn report_repeated_bracket(&mut self, name: &Identifier, earlier: Span) {
        if self.begin_report(name.span) {
            let spelling = identifier_spelling_for_diagnostic(&name.text);
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::DuplicateParameter,
                    format!("duplicate parameter `{spelling}`"),
                    name.span,
                )
                .with_label("this name is already a parameter in brackets")
                .with_secondary_span(earlier, "first parameter in brackets is here")
                .with_note("each size and type parameter in a function's brackets has a name of its own"),
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
        let label = instance.label(
            &function.name.text,
            &type_spellings(self.source, &function.sizes),
        );
        let name = identifier_spelling_for_diagnostic(&function.name.text);
        let typed = function.sizes.iter().any(SizeParameter::is_type);
        let sized = function.sizes.iter().any(|size| !size.is_type());
        let rule = match (sized, typed) {
            (true, false) => "a sized function is checked once for each value of its sizes",
            (false, _) => "a function is checked once for each type of its type parameters",
            (true, true) => {
                "a function is checked once for each value of its sizes and each type of its \
                 type parameters"
            }
        };
        for diagnostic in self.diagnostics.iter_mut().skip(reported) {
            if !matches!(
                diagnostic.code(),
                DiagnosticCode::TooManySemanticErrors | DiagnosticCode::SemanticResourceLimit
            ) {
                diagnostic.add_note(format!(
                    "in the instance `{label}`, the first of `{name}` in error: {rule}"
                ));
            }
        }
    }

    /// Returns the instance a call names and its signature: the call gives
    /// one entry in brackets for each of the callee's size and type
    /// parameters, each size's value, computed in the caller's instance,
    /// lies in its range, and each type is one its parameter lists. A call
    /// that writes no brackets names the one instance that fits its
    /// arguments, and for a callee with type parameters the place of the
    /// call, whose type is `expected`. Reports why otherwise. A callee whose
    /// sizes or types are malformed was reported at its declaration and is
    /// not reported again.
    pub(super) fn called_instance<'signature>(
        &mut self,
        expression: &Expression,
        call: &CallExpression,
        signature: &'signature Signature<'_>,
        expected: &CoreType,
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
            return match self.fitting_instance(call, signature, Some(expected), context, scope) {
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
            self.report_size_count(expression.span, call, signature.sizes);
            return None;
        }
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        for ((position, size), slot) in call.sizes().iter().enumerate().zip(&mut values) {
            let parameter = signature.sizes.get(position)?;
            if parameter.is_type() {
                *slot = self.called_type(size, &call.callee, signature, position)?;
                continue;
            }
            let value = self.size_value(size)?;
            let (start, end) = ranges.range(position)?;
            match value
                .to_i64()
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| (start..end).contains(value))
            {
                Some(value) => *slot = value,
                None => {
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

    /// Returns the position, in the list of the type parameter at
    /// `position`, of the type a call writes for it in brackets, or reports
    /// why the entry is not one of the listed types.
    fn called_type(
        &mut self,
        entry: &Expression,
        callee: &Identifier,
        signature: &Signature<'_>,
        position: usize,
    ) -> Option<u32> {
        if !self.event(entry.span) {
            return None;
        }
        let parameter = signature.sizes.get(position)?;
        let listed = signature.listed.get(position)?;
        match self.type_argument(entry) {
            TypeArgument::Type(ty) => {
                // A listed type that did not resolve was reported at the
                // callee's declaration.
                if listed.iter().any(Option::is_none) {
                    return None;
                }
                match listed
                    .iter()
                    .position(|candidate| candidate.as_ref() == Some(&ty))
                {
                    Some(index) => u32::try_from(index).ok(),
                    None => {
                        let spellings = signature.spellings.get(position)?;
                        self.report_type_outside(entry.span, callee, parameter, spellings);
                        None
                    }
                }
            }
            TypeArgument::Unresolved => None,
            TypeArgument::NotAType => {
                self.report_not_a_type(entry.span, callee, parameter);
                None
            }
        }
    }

    /// Returns the type a call's entry in brackets names, without reporting:
    /// `Int`, `Bool`, `Word[n]`, an array of one of them, a `type`
    /// declaration's name, or a type parameter of the instance being checked.
    ///
    /// Parser-established expression height bounds this recursion.
    fn type_argument(&self, entry: &Expression) -> TypeArgument {
        let resolved =
            |ty: Option<CoreType>| ty.map_or(TypeArgument::Unresolved, TypeArgument::Type);
        match &entry.kind {
            ExpressionKind::Name(name) => {
                let text = name.text.as_str();
                if let Some(position) = self.types.sizes.instance.find_type(text) {
                    return resolved(self.types.sizes.types.get(position).cloned().flatten());
                }
                match text {
                    "Int" => TypeArgument::Type(CoreType::Int),
                    "Bool" => TypeArgument::Type(CoreType::Bool),
                    _ => self
                        .types
                        .name(text)
                        .map_or(TypeArgument::NotAType, |declared| {
                            resolved(declared.ty.clone())
                        }),
                }
            }
            ExpressionKind::Index(index) => match (&index.base.kind, &index.index.kind) {
                (ExpressionKind::Name(base), ExpressionKind::Literal(_)) if base.text == "Word" => {
                    let width = match self.source.slice(index.index.span) {
                        Some("8") => Some(8),
                        Some("16") => Some(16),
                        Some("32") => Some(32),
                        Some("64") => Some(64),
                        _ => None,
                    };
                    width
                        .and_then(CoreType::word_of_width)
                        .map_or(TypeArgument::NotAType, TypeArgument::Type)
                }
                _ => TypeArgument::NotAType,
            },
            // An array of a word, `Int`, `Bool`, or residue type, `T^n`.
            ExpressionKind::Binary(binary)
                if matches!(binary.operator, BinaryOperator::Xor)
                    && matches!(binary.right.kind, ExpressionKind::Literal(_)) =>
            {
                match self.type_argument(&binary.left) {
                    TypeArgument::Type(element) if element.is_scalar() => {
                        size_bound(self.source, binary.right.span)
                            .and_then(|length| ArrayType::new(&element, length))
                            .map_or(TypeArgument::NotAType, |array| {
                                TypeArgument::Type(CoreType::Array(array))
                            })
                    }
                    TypeArgument::Unresolved => TypeArgument::Unresolved,
                    TypeArgument::Type(_) | TypeArgument::NotAType => TypeArgument::NotAType,
                }
            }
            _ => TypeArgument::NotAType,
        }
    }

    /// Checks a leaf whose type is not known without its place, where that
    /// check reports why. A call that writes no brackets and fits several
    /// instances of a function with type parameters is reported as such
    /// here: its place does not choose among them, since the place's type is
    /// what the leaf's type was needed for.
    pub(super) fn check_untyped(
        &mut self,
        leaf: &'ast Expression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) {
        if let ExpressionKind::Call(call) = &leaf.kind
            && call.sizes().is_empty()
            && let Some((declarations, signatures)) = scope.tables_for(call)
            && let Some(entry) =
                first_declaration(declarations, FunctionKind::Spec, &call.callee.text)
            && let Some(Some(signature)) = signatures.get(entry.source_index)
            && signature.sizes.iter().any(SizeParameter::is_type)
            && signature
                .instances
                .first()
                .is_some_and(|first| first.parameters.len() == call.arguments.len())
            && let Err(fitting @ [Some(_), Some(_)]) =
                self.fitting_instance(call, signature, None, context, scope)
        {
            self.report_unfitted_call(leaf.span, call, signature, fitting, context, scope);
            return;
        }
        self.check_expression(leaf, expected, context, scope, output);
    }

    /// Returns the type of a call's argument without reporting: the type of
    /// its first typed leaf, an array literal's or a fill's from its
    /// elements and its length, or `None` for an argument whose type is
    /// not known without its place.
    fn argument_type(
        &self,
        argument: &Expression,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Option<CoreType> {
        let leaf = first_typed_leaf(argument)?;
        match &leaf.kind {
            ExpressionKind::Array(_) | ExpressionKind::Fill(_) => {
                self.literal_array_type(leaf, context, scope)
            }
            ExpressionKind::Tuple(_) => None,
            _ => self.leaf_type(leaf, context, scope),
        }
    }

    /// Returns the index of the one instance that fits a call that writes
    /// no brackets, or the indices of the first two instances that fit, or
    /// none, without reporting.
    ///
    /// For a function with sizes only, an instance fits when its array
    /// parameters have the lengths of the call's arguments. For a function
    /// with type parameters, it fits when its parameters have the types of
    /// the call's arguments, and among several that do, the one whose
    /// result has the type `expected`, when given, is called. An argument
    /// whose length or type is not known without reporting fits every
    /// instance.
    fn fitting_instance(
        &self,
        call: &CallExpression,
        signature: &Signature<'_>,
        expected: Option<&CoreType>,
        context: &BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
    ) -> Result<usize, [Option<usize>; 2]> {
        let typed = signature.sizes.iter().any(SizeParameter::is_type);
        let first = signature.instances.first().ok_or([None; 2])?;
        let mut lengths = [None; MAX_PARAMETERS_PER_FUNCTION];
        for ((length, argument), parameter) in
            lengths.iter_mut().zip(&call.arguments).zip(&first.parameters)
        {
            // A type parameter may make a parameter an array in one
            // instance and not in another.
            if typed || parameter.as_ref().and_then(CoreType::as_array).is_some() {
                *length = self.array_length_of(argument, context, scope);
            }
        }
        // Types are kept on the heap: this frame is on the stack of every
        // silently typed call nested in an argument.
        let mut types = Vec::new();
        if typed && types.try_reserve_exact(call.arguments.len()).is_ok() {
            types.extend(
                call.arguments
                    .iter()
                    .map(|argument| self.argument_type(argument, context, scope)),
            );
        }
        let mut fitting = [None; 2];
        let mut matching = [None; 2];
        let (mut found, mut matched) = (0_usize, 0_usize);
        for (index, instance) in signature.instances.iter().enumerate() {
            let fits =
                instance
                    .parameters
                    .iter()
                    .enumerate()
                    .all(|(position, parameter)| {
                        let length = lengths.get(position).copied().flatten();
                        match (parameter, types.get(position).and_then(Option::as_ref)) {
                            (Some(parameter), Some(given)) => parameter == given,
                            (Some(parameter), None) => match (parameter.as_array(), length) {
                                (Some(array), Some(length)) => array.length() == length,
                                _ => true,
                            },
                            (None, _) => true,
                        }
                    });
            if !fits {
                continue;
            }
            if let Some(slot) = fitting.get_mut(found) {
                *slot = Some(index);
            }
            found = found.saturating_add(1);
            let gives = match (typed.then_some(expected).flatten(), &instance.result_type) {
                (Some(expected), Some(result)) => expected == result,
                _ => true,
            };
            if gives {
                if let Some(slot) = matching.get_mut(matched) {
                    *slot = Some(index);
                }
                matched = matched.saturating_add(1);
            }
            // Without an expected type, two fitting instances decide.
            if found > 1 && (!typed || expected.is_none()) {
                break;
            }
        }
        match (fitting, matching) {
            ([Some(index), None], _) | (_, [Some(index), None]) => Ok(index),
            (_, [Some(_), Some(_)]) => Err(matching),
            _ => Err(fitting),
        }
    }

    /// Reports a call without brackets whose arguments fit no instance of
    /// the callee, or more than one.
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
        let typed = signature.sizes.iter().any(SizeParameter::is_type);
        let sized = signature.sizes.iter().any(|size| !size.is_type());
        let domain = signature
            .sizes
            .iter()
            .enumerate()
            .filter_map(|(position, size)| {
                let parameter = identifier_spelling_for_diagnostic(&size.name.text);
                if size.is_type() {
                    let listed = signature.spellings.get(position)?.join(", ");
                    return Some(format!("`{parameter}` in {{{listed}}}"));
                }
                let (start, end) = ranges.range(position)?;
                Some(format!("`{parameter}` in {start}..{end}"))
            })
            .collect::<Vec<_>>()
            .join(", ");
        let diagnostic = if let [Some(first), Some(second)] = fitting {
            let label = |index: usize| {
                ranges
                    .instance(signature.sizes, index)
                    .map_or_else(String::new, |instance| {
                        instance.label(&call.callee.text, &signature.spellings)
                    })
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
            .with_label(match (sized, typed) {
                (_, false) => "write the sizes in brackets",
                (false, true) => "write the types in brackets",
                (true, true) => "write the sizes and types in brackets",
            })
        } else if typed {
            let types = call
                .arguments
                .iter()
                .filter_map(|argument| self.argument_type(argument, context, scope))
                .map(|ty| format!("`{ty}`"))
                .collect::<Vec<_>>();
            let label = match types.as_slice() {
                [] => String::from("no instance fits these arguments"),
                [ty] => format!("an argument of type {ty} is given"),
                _ => format!("arguments of types {} are given", types.join(", ")),
            };
            Diagnostic::error(
                DiagnosticCode::TypeParameter,
                format!("no instance of `{name}` takes arguments of these types"),
                span,
            )
            .with_label(label)
            .with_note(format!("`{name}` is defined for {domain}"))
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
        self.diagnostics.push(diagnostic.with_note(if typed {
            "a call that writes no brackets calls the one instance of its function whose \
             parameters have its arguments' types, and among several, the one whose result has \
             the type its place expects; any other call writes its types in brackets, as in \
             `pow[F](x, e)`"
        } else {
            "a call that writes no sizes calls the one instance of its function whose array \
             parameters have the lengths of its arguments; any other call writes its sizes in \
             brackets, as in `absorb[2](p)`"
        }));
    }

    #[cold]
    #[inline(never)]
    fn report_size_count(&mut self, span: Span, call: &CallExpression, declared: &[SizeParameter]) {
        if !self.begin_report(span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(&call.callee.text);
        let given = call.sizes().len();
        let types = declared.iter().filter(|size| size.is_type()).count();
        let sizes = declared.len().saturating_sub(types);
        let plural = |count: usize| if count == 1 { "size" } else { "sizes" };
        let (message, note) = if types == 0 {
            let message = match (sizes, given) {
                (0, _) => format!(
                    "`{spelling}` has no size parameters, but this call gives {given} {}",
                    plural(given)
                ),
                (_, 0) => format!(
                    "`{spelling}` takes {sizes} {}, but this call gives none",
                    plural(sizes)
                ),
                _ => format!(
                    "`{spelling}` takes {sizes} {}, but this call gives {given}",
                    plural(sizes)
                ),
            };
            (
                message,
                "a sized function is called with one value for each of its sizes, in brackets \
                 before its arguments, as in `sha256[2](m)`; a function without sizes is called \
                 without brackets",
            )
        } else {
            let taken = match (sizes, types) {
                (0, 1) => String::from("1 type"),
                (0, _) => format!("{types} types"),
                _ => format!(
                    "{sizes} {} and {types} {}",
                    plural(sizes),
                    if types == 1 { "type" } else { "types" }
                ),
            };
            (
                format!("`{spelling}` takes {taken} in brackets, but this call gives {given}"),
                "a function with type parameters is called with one entry in brackets for each \
                 of its sizes and types, in order, as in `pow[F](x, e)`, or without brackets when \
                 its arguments choose one instance",
            )
        };
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::SizeCount, message, span)
                .with_label(if types == 0 {
                    "wrong number of sizes"
                } else {
                    "wrong number of entries in brackets"
                })
                .with_note(note),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_type_outside(
        &mut self,
        span: Span,
        callee: &Identifier,
        parameter: &SizeParameter,
        spellings: &[String],
    ) {
        if !self.begin_report(span) {
            return;
        }
        let function = identifier_spelling_for_diagnostic(&callee.text);
        let name = identifier_spelling_for_diagnostic(&parameter.name.text);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeParameter,
                format!(
                    "`{function}` is defined for `{name}` in {{{}}}",
                    spellings.join(", ")
                ),
                span,
            )
            .with_label("this type is not listed")
            .with_note(TYPE_PARAMETER_CALL_NOTE),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_not_a_type(&mut self, span: Span, callee: &Identifier, parameter: &SizeParameter) {
        if !self.begin_report(span) {
            return;
        }
        let function = identifier_spelling_for_diagnostic(&callee.text);
        let name = identifier_spelling_for_diagnostic(&parameter.name.text);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeParameter,
                format!("`{function}` takes a type for `{name}` here"),
                span,
            )
            .with_label("this is not a type")
            .with_note(TYPE_ARGUMENT_NOTE),
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
    /// reporting, or `None` when its brackets do not name one.
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
                .fitting_instance(call, signature, None, context, scope)
                .ok()?;
            return signature.instances.get(index);
        }
        if call.sizes().len() != ranges.count() {
            return None;
        }
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        for ((position, size), slot) in call.sizes().iter().enumerate().zip(&mut values) {
            *slot = if signature.sizes.get(position)?.is_type() {
                let TypeArgument::Type(ty) = self.type_argument(size) else {
                    return None;
                };
                let listed = signature.listed.get(position)?;
                u32::try_from(
                    listed
                        .iter()
                        .position(|candidate| candidate.as_ref() == Some(&ty))?,
                )
                .ok()?
            } else {
                self.types
                    .sizes
                    .value(self.source, size)
                    .ok()?
                    .to_i64()
                    .and_then(|value| u32::try_from(value).ok())?
            };
        }
        signature
            .instances
            .get(ranges.index(values.get(..ranges.count())?)?)
    }
}
