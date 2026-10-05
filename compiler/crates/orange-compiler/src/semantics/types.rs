//! Declared types, moduli, and literals: resolving what a written type
//! means and decoding integer literals into exact values.

use super::*;

/// The outcome of classifying a parsed type without reporting.
pub(super) enum TypeClass {
    Resolved(CoreType),
    MissingWordWidth,
    UnsupportedWordWidth(Span),
    UnsupportedArrayLength(Span),
    /// A length written with sizes whose value in this instance is not
    /// from 1 through [`MAX_ARRAY_LENGTH`], at the length's span.
    ArrayLengthValue(Span, ExactInteger),
    /// A length written with sizes that has no value.
    Size(SizeFault),
    /// A size-dependent modulus has no admitted value in this instance.
    Modulus(ModulusFault),
    MissingModulus,
    /// A declared name whose type already has the most array dimensions,
    /// followed by the span of `^LENGTH`.
    ArrayOfArrays(Span),
    /// An admitted outer length and the scalars of each element, whose
    /// product is too large.
    ArrayShape(Span, u32, u32),
    /// A declared name whose type is a tuple, followed by the span of
    /// `^LENGTH`.
    ArrayOfTuples(Span),
    /// A modulus or declared name whose own check already failed.
    Unresolved,
    /// A modulus the module's table does not hold.
    Unindexed,
    Unsupported,
}

/// The types a module fixes before its functions are checked: the value of
/// every modulus it writes, and the names of its `type` declarations; and
/// the sizes of the instance whose types are being resolved.
pub(super) struct TypeTable<'ast> {
    /// Each modulus by the extent of its expression, sorted by that key.
    pub(super) moduli: Vec<ResolvedModulus>,
    /// Declared names in declaration order, each unique.
    pub(super) names: Vec<DeclaredType<'ast>>,
    /// The sizes of the instance being checked, or none outside a sized
    /// function.
    pub(super) sizes: SizeScope<'ast>,
}

pub(super) struct ResolvedModulus {
    pub(super) key: (u32, u32),
    /// The modulus, or `None` when its expression was reported.
    pub(super) modulus: Option<Modulus>,
    /// Its value is computed in each concrete instance rather than once
    /// for the module. `modulus` is then `None`, without a prior report.
    pub(super) dependent: bool,
}

/// Why a modulus has no value within a concrete function instance.
pub(super) enum ModulusFault {
    NotConstant(Span),
    ShiftAmount(Span),
    Value(Span, ExactInteger),
    TooLarge(Span),
    Storage(Span),
}

pub(super) struct DeclaredType<'ast> {
    pub(super) name: &'ast str,
    pub(super) span: Span,
    /// The declared type, or `None` when its declaration was reported.
    pub(super) ty: Option<CoreType>,
}

/// One `types` declaration after its entries have been checked.
pub(super) struct ResolvedTypeList<'ast> {
    pub(super) name: &'ast str,
    pub(super) span: Span,
    /// Every entry resolved, the entries are distinct, and there are at
    /// most [`MAX_INSTANCES_PER_FUNCTION`] of them.
    pub(super) valid: bool,
}

impl TypeTable<'_> {
    pub(super) const fn new() -> Self {
        Self {
            moduli: Vec::new(),
            names: Vec::new(),
            sizes: SizeScope {
                instance: Instance::NONE,
                types: [const { None }; MAX_SIZES_PER_FUNCTION],
                bits: MAX_INTEGER_BITS,
                reserve: reserve_range_limbs,
                reserve_limb: reserve_magnitude_limb,
                evaluated: Cell::new(0),
            },
        }
    }

    /// Returns the entry of the modulus written as `expression`.
    pub(super) fn modulus(&self, expression: &Expression) -> Option<&ResolvedModulus> {
        let key = span_key(expression.span);
        let position = self
            .moduli
            .binary_search_by_key(&key, |entry| entry.key)
            .ok()?;
        self.moduli.get(position)
    }

    /// Returns the declaration of `name`, if the module declares it.
    pub(super) fn name(&self, name: &str) -> Option<&DeclaredType<'_>> {
        self.names.iter().find(|declared| declared.name == name)
    }

    /// Whether this written type computes a modulus in each instance.
    pub(super) fn modulus_depends_on_sizes(&self, syntax: &TypeSyntax) -> bool {
        syntax
            .modulus()
            .and_then(|expression| self.modulus(expression))
            .is_some_and(|entry| entry.dependent)
            || syntax
                .elements
                .iter()
                .any(|element| self.modulus_depends_on_sizes(element))
    }
}

pub(super) fn span_key(span: Span) -> (u32, u32) {
    (span.start().bytes(), span.end().bytes())
}

pub(super) fn classify_type(
    source: &SourceFile,
    table: &TypeTable<'_>,
    syntax: &TypeSyntax,
) -> TypeClass {
    if syntax.is_tuple() {
        return classify_tuple_type(source, table, syntax);
    }
    let scalar = classify_scalar_type(source, table, syntax);
    match (scalar, syntax.length.as_ref()) {
        (TypeClass::Resolved(CoreType::Array(row)), Some(length))
            if row.dimensions() >= MAX_ARRAY_DIMENSIONS =>
        {
            TypeClass::ArrayOfArrays(length.span)
        }
        (TypeClass::Resolved(CoreType::Tuple(_)), Some(length)) => {
            TypeClass::ArrayOfTuples(length.span)
        }
        (TypeClass::Resolved(element), Some(length)) => {
            match table.sizes.array_length(source, length) {
                Length::Admitted(count) => match ArrayType::new(&element, count) {
                    Some(array) => TypeClass::Resolved(CoreType::Array(array)),
                    None => element
                        .as_array()
                        .map_or(TypeClass::UnsupportedArrayLength(length.span), |row| {
                            TypeClass::ArrayShape(length.span, row.scalar_length(), count)
                        }),
                },
                Length::Literal => TypeClass::UnsupportedArrayLength(length.span),
                Length::Value(value) => TypeClass::ArrayLengthValue(length.span, value),
                Length::Fault(fault) => TypeClass::Size(fault),
            }
        }
        (scalar, _) => scalar,
    }
}

/// Classifies a tuple type: the class of its first element that does not
/// resolve, or the tuple of its element types.
fn classify_tuple_type(
    source: &SourceFile,
    table: &TypeTable<'_>,
    syntax: &TypeSyntax,
) -> TypeClass {
    let mut elements = Vec::new();
    if elements.try_reserve_exact(syntax.elements.len()).is_err() {
        return TypeClass::Unresolved;
    }
    for element in &syntax.elements {
        match classify_type(source, table, element) {
            TypeClass::Resolved(ty) => elements.push(ty),
            other => return other,
        }
    }
    TupleType::new(&elements).map_or(TypeClass::Unsupported, |tuple| {
        TypeClass::Resolved(CoreType::Tuple(tuple))
    })
}

/// The most digits an admitted array length has: 65,536 has five.
const ARRAY_LENGTH_DIGITS: usize = 5;
const _: () = assert!(MAX_ARRAY_LENGTH >= 10_000 && MAX_ARRAY_LENGTH < 100_000);

/// Decodes an array length written as a decimal integer with no leading
/// zero and no underscore, as word widths are written.
pub(super) fn array_length(source: &SourceFile, span: Span) -> Option<u32> {
    let spelling = source.slice(span)?;
    let canonical = !spelling.is_empty()
        && !spelling.starts_with('0')
        && spelling.bytes().all(|byte| byte.is_ascii_digit());
    if !canonical || spelling.len() > ARRAY_LENGTH_DIGITS {
        return None;
    }
    spelling.parse().ok()
}

/// Classifies a type without its array length. A declared name stands for
/// its whole type, which may be an array type.
pub(super) fn classify_scalar_type(
    source: &SourceFile,
    table: &TypeTable<'_>,
    syntax: &TypeSyntax,
) -> TypeClass {
    if let Some(expression) = syntax.modulus() {
        return match table.modulus(expression) {
            Some(ResolvedModulus {
                dependent: true, ..
            }) => table
                .sizes
                .modulus(source, expression)
                .map_or_else(TypeClass::Modulus, |modulus| {
                    TypeClass::Resolved(CoreType::Mod(modulus))
                }),
            Some(ResolvedModulus {
                modulus: Some(modulus),
                ..
            }) => TypeClass::Resolved(CoreType::Mod(*modulus)),
            Some(_) => TypeClass::Unresolved,
            None => TypeClass::Unindexed,
        };
    }
    match (syntax.name.text.as_str(), syntax.width_span) {
        ("Int", None) => TypeClass::Resolved(CoreType::Int),
        ("Bool", None) => TypeClass::Resolved(CoreType::Bool),
        ("Word", Some(width_span)) => {
            let width = match source.slice(width_span) {
                Some("8") => Some(8),
                Some("16") => Some(16),
                Some("32") => Some(32),
                Some("64") => Some(64),
                _ => None,
            };
            width
                .and_then(CoreType::word_of_width)
                .map_or(TypeClass::UnsupportedWordWidth(width_span), |ty| {
                    TypeClass::Resolved(ty)
                })
        }
        ("Word", None) => TypeClass::MissingWordWidth,
        ("Mod", None) => TypeClass::MissingModulus,
        // A type parameter stands for its type in the instance.
        (name, None) if table.sizes.instance.find_type(name).is_some() => table
            .sizes
            .instance
            .find_type(name)
            .and_then(|position| table.sizes.types.get(position))
            .cloned()
            .flatten()
            .map_or(TypeClass::Unresolved, TypeClass::Resolved),
        (name, None) => table.name(name).map_or(TypeClass::Unsupported, |declared| {
            declared
                .ty
                .clone()
                .map_or(TypeClass::Unresolved, TypeClass::Resolved)
        }),
        _ => TypeClass::Unsupported,
    }
}

pub(super) fn silent_type(
    source: &SourceFile,
    table: &TypeTable<'_>,
    syntax: &TypeSyntax,
) -> Option<CoreType> {
    match classify_type(source, table, syntax) {
        TypeClass::Resolved(ty) => Some(ty),
        _ => None,
    }
}

/// Whether a modulus expression mentions one of the current sizes. This
/// recognizes names inside even an unsupported arithmetic operator so
/// that its eventual report identifies the first concrete instance.
/// Parser-established expression height bounds this recursion.
fn references_size(expression: &Expression, instance: Instance<'_>) -> bool {
    match &expression.kind {
        ExpressionKind::Name(name) => instance.find(&name.text).is_some(),
        ExpressionKind::Parenthesized(inner) => references_size(inner, instance),
        ExpressionKind::Unary(unary) => references_size(&unary.operand, instance),
        ExpressionKind::Binary(binary) => {
            references_size(&binary.left, instance) || references_size(&binary.right, instance)
        }
        _ => false,
    }
}

/// The same value-specific label for module constants and instance moduli.
fn invalid_modulus_label(value: &ExactInteger) -> String {
    if value.magnitude_bits() > MAX_MODULUS_BITS && !value.is_negative() {
        format!("this modulus has {} bits", value.magnitude_bits())
    } else if value.magnitude_bits() <= 64 {
        format!("this modulus is {value}")
    } else {
        String::from("this modulus is negative")
    }
}

impl SizeScope<'_> {
    /// Computes a concrete modulus with precisely the modulus-expression
    /// operators. Size arithmetic's division, remainder and prefix
    /// negation are not added to this separate static expression domain.
    pub(super) fn modulus(
        &self,
        source: &SourceFile,
        expression: &Expression,
    ) -> Result<Modulus, ModulusFault> {
        let value = self.modulus_value(source, expression)?;
        Modulus::new(&value).ok_or(ModulusFault::Value(expression.span, value))
    }

    /// Each visited part and each significant literal digit is charged as
    /// an event, as for module-constant moduli. Values remain exact within
    /// the same integer and fallible storage limits.
    /// Parser-established expression height bounds this recursion.
    fn modulus_value(
        &self,
        source: &SourceFile,
        expression: &Expression,
    ) -> Result<ExactInteger, ModulusFault> {
        self.evaluated.set(self.evaluated.get().saturating_add(1));
        let storage = ModulusFault::Storage(expression.span);
        let reserve = self.reserve;
        let value = match &expression.kind {
            ExpressionKind::Literal(literal) => self.modulus_literal(source, literal)?,
            ExpressionKind::Name(name) => match self.instance.find(&name.text) {
                Some((_, value)) => {
                    ExactInteger::from_u64(u64::from(value), reserve).ok_or(storage)?
                }
                None => return Err(ModulusFault::NotConstant(name.span)),
            },
            ExpressionKind::Parenthesized(inner) => return self.modulus_value(source, inner),
            ExpressionKind::Binary(binary) if binary.operator == BinaryOperator::ShiftLeft => {
                let left = self.modulus_value(source, &binary.left)?;
                let right = self.modulus_value(source, &binary.right)?;
                let amount = right
                    .to_i64()
                    .and_then(|amount| usize::try_from(amount).ok())
                    .filter(|amount| *amount <= self.bits)
                    .ok_or(ModulusFault::ShiftAmount(binary.right.span))?;
                ExactInteger::power_of_two(amount, reserve)
                    .and_then(|power| left.multiply(&power, reserve))
                    .ok_or(storage)?
            }
            ExpressionKind::Binary(binary)
                if matches!(
                    binary.operator,
                    BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply
                ) =>
            {
                let left = self.modulus_value(source, &binary.left)?;
                let right = self.modulus_value(source, &binary.right)?;
                match binary.operator {
                    BinaryOperator::Add => left.add(&right, reserve),
                    BinaryOperator::Subtract => left.subtract(&right, reserve),
                    _ => left.multiply(&right, reserve),
                }
                .ok_or(storage)?
            }
            _ => return Err(ModulusFault::NotConstant(expression.span)),
        };
        if value.magnitude_bits() > self.bits {
            return Err(ModulusFault::TooLarge(expression.span));
        }
        Ok(value)
    }

    fn modulus_literal(
        &self,
        source: &SourceFile,
        literal: &IntegerLiteral,
    ) -> Result<ExactInteger, ModulusFault> {
        let storage = || ModulusFault::Storage(literal.span);
        let spelling = source.slice(literal.magnitude_span).ok_or_else(storage)?;
        self.evaluated.set(self.evaluated.get().saturating_add(1));
        let (radix, digits) = split_radix(spelling);
        let mut magnitude = Magnitude::zero();
        let mut significant = false;
        for character in digits.chars().filter(|character| *character != '_') {
            let digit = character.to_digit(radix).ok_or_else(storage)?;
            significant |= digit != 0;
            if significant {
                self.evaluated.set(self.evaluated.get().saturating_add(1));
            }
            if !magnitude.multiply_add_with_reservation(radix, digit, self.reserve_limb) {
                return Err(storage());
            }
            if magnitude.bit_len() > self.bits {
                return Err(ModulusFault::TooLarge(literal.magnitude_span));
            }
        }
        Ok(ExactInteger::new(literal.negative, magnitude))
    }
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Indexes every written modulus before types are resolved. A modulus
    /// without size parameters is evaluated once in source order; one
    /// using a function's sizes is evaluated in each concrete instance.
    pub(super) fn resolve_moduli(&mut self) {
        let module = &self.ast.module;
        for declaration in &module.types {
            self.resolve_modulus(&declaration.ty);
        }
        for declaration in &module.type_lists {
            for ty in &declaration.types {
                self.resolve_modulus(ty);
            }
        }
        for function in &module.functions {
            let (FunctionBody::Typed(body), FunctionKind::Spec) = (&function.body, function.kind)
            else {
                continue;
            };
            for ty in function.sizes.iter().flat_map(|size| &size.types) {
                self.resolve_modulus(ty);
            }
            // Lists are independent of every instance. Only signatures
            // and bodies may use the function's finite size parameters.
            self.enter_instance(
                Instance {
                    parameters: &function.sizes,
                    values: [0; MAX_SIZES_PER_FUNCTION],
                },
                &[],
            );
            for parameter in &function.parameters {
                self.resolve_modulus(&parameter.ty);
            }
            self.resolve_modulus(&body.result_type);
            for binding in &body.bindings {
                self.resolve_pattern_moduli(&binding.pattern);
                self.resolve_moduli_within(&binding.value);
            }
            self.resolve_moduli_within(&body.expression);
            self.enter_instance(Instance::NONE, &[]);
        }
        // A test's moduli are resolved only where its body is checked.
        if self.tests {
            for test in &module.tests {
                let Some(body) = test.body() else {
                    continue;
                };
                self.resolve_block_moduli(&body.bindings, &body.expression);
            }
        }
        self.types.moduli.sort_unstable_by_key(|entry| entry.key);
    }

    /// Evaluates the moduli of the conversions and loops of an expression.
    ///
    /// Parser-established expression height bounds this recursion.
    pub(super) fn resolve_moduli_within(&mut self, expression: &Expression) {
        if self.halted {
            return;
        }
        match &expression.kind {
            ExpressionKind::Literal(_) | ExpressionKind::Name(_) | ExpressionKind::Bytes(_) => {}
            ExpressionKind::Call(call) => {
                for argument in &call.arguments {
                    self.resolve_moduli_within(argument);
                }
            }
            ExpressionKind::Slice(slice) => {
                self.resolve_moduli_within(&slice.base);
                self.resolve_range_moduli(&slice.range);
            }
            ExpressionKind::SliceUpdate(update) => {
                self.resolve_moduli_within(&update.base);
                self.resolve_range_moduli(&update.range);
                self.resolve_moduli_within(&update.value);
            }
            ExpressionKind::Unary(unary) => self.resolve_moduli_within(&unary.operand),
            ExpressionKind::Binary(binary) => {
                self.resolve_moduli_within(&binary.left);
                self.resolve_moduli_within(&binary.right);
            }
            ExpressionKind::Parenthesized(inner) => self.resolve_moduli_within(inner),
            ExpressionKind::Conversion(conversion) => {
                self.resolve_moduli_within(&conversion.operand);
                self.resolve_modulus(&conversion.target);
            }
            ExpressionKind::Array(array) => {
                for element in &array.elements {
                    self.resolve_moduli_within(element);
                }
            }
            ExpressionKind::Fill(fill) => self.resolve_moduli_within(&fill.element),
            ExpressionKind::Tuple(tuple) => {
                for element in &tuple.elements {
                    self.resolve_moduli_within(element);
                }
            }
            ExpressionKind::Project(project) => self.resolve_moduli_within(&project.base),
            ExpressionKind::Index(index) => {
                self.resolve_moduli_within(&index.base);
                self.resolve_moduli_within(&index.index);
            }
            ExpressionKind::Update(update) => {
                self.resolve_moduli_within(&update.base);
                self.resolve_moduli_within(&update.index);
                for index in &update.path {
                    self.resolve_moduli_within(index);
                }
                self.resolve_moduli_within(&update.value);
            }
            ExpressionKind::Loop(r#loop) => {
                self.resolve_pattern_moduli(&r#loop.accumulator);
                self.resolve_moduli_within(&r#loop.init);
                self.resolve_block_moduli(&r#loop.step_bindings, &r#loop.step);
            }
            ExpressionKind::Conditional(conditional) => {
                for arm in &conditional.arms {
                    self.resolve_moduli_within(&arm.condition);
                    self.resolve_block_moduli(&arm.bindings, &arm.value);
                }
                self.resolve_block_moduli(&conditional.otherwise_bindings, &conditional.otherwise);
            }
        }
    }

    /// Evaluates the moduli within a slice's bounds, in source order.
    pub(super) fn resolve_range_moduli(&mut self, range: &SliceRange) {
        for bound in range.start.iter().chain(&range.end) {
            self.resolve_moduli_within(bound);
        }
    }

    /// Evaluates the moduli of a block's binding types and values and of
    /// its value, in source order.
    pub(super) fn resolve_block_moduli(&mut self, bindings: &[Binding], value: &Expression) {
        for binding in bindings {
            self.resolve_pattern_moduli(&binding.pattern);
            self.resolve_moduli_within(&binding.value);
        }
        self.resolve_moduli_within(value);
    }

    /// Evaluates the moduli of a pattern's declared types, in order.
    pub(super) fn resolve_pattern_moduli(&mut self, pattern: &Pattern) {
        for typed in pattern.names() {
            self.resolve_modulus(&typed.ty);
        }
    }

    /// Evaluates the modulus of `Mod[...]`, if `syntax` has one, and enters
    /// it in the module's table, then the moduli written within it.
    pub(super) fn resolve_modulus(&mut self, syntax: &TypeSyntax) {
        // A tuple type's elements are never tuples, so this recursion is one
        // level deep.
        for element in &syntax.elements {
            self.resolve_modulus(element);
        }
        let Some(expression) = syntax.modulus() else {
            return;
        };
        if self.halted {
            return;
        }
        let dependent = references_size(expression, self.types.sizes.instance);
        let modulus = if dependent {
            None
        } else {
            self.constant(expression)
                .and_then(|value| self.checked_modulus(expression.span, &value))
        };
        if self.halted {
            return;
        }
        if self.types.moduli.try_reserve(1).is_err() {
            self.resource_limit(expression.span, "semantic modulus table allocation failed");
            return;
        }
        self.types.moduli.push(ResolvedModulus {
            key: span_key(expression.span),
            modulus,
            dependent,
        });
        // A modulus that is not a constant may still hold a conversion or a
        // loop whose type has a modulus of its own; that one is evaluated too.
        self.resolve_moduli_within(expression);
    }

    /// Evaluates a constant: integer literals combined with `+`, `-`, `*`,
    /// `<<`, and parentheses, with every value within the exact-integer
    /// limit. Anything else is reported, and gives `None`.
    ///
    /// Parser-established expression height bounds this recursion.
    pub(super) fn constant(&mut self, expression: &Expression) -> Option<ExactInteger> {
        if !self.event(expression.span) {
            return None;
        }
        let reserve = self.reserve_range_limbs;
        let value = match &expression.kind {
            ExpressionKind::Literal(literal) => {
                let magnitude = self.parse_magnitude(literal, self.limits.integer_bits)?;
                Some(ExactInteger::new(literal.negative, magnitude))
            }
            ExpressionKind::Parenthesized(inner) => return self.constant(inner),
            ExpressionKind::Binary(binary) if binary.operator == BinaryOperator::ShiftLeft => {
                let value = self.constant(&binary.left)?;
                let amount = self.constant(&binary.right)?;
                let bits = self.limits.integer_bits;
                let Some(amount) = amount
                    .to_i64()
                    .and_then(|amount| usize::try_from(amount).ok())
                    .filter(|amount| *amount <= bits)
                else {
                    self.report_invalid_modulus(
                        binary.right.span,
                        format!("a shift amount in a modulus is from 0 through {bits}"),
                    );
                    return None;
                };
                ExactInteger::power_of_two(amount, reserve)
                    .and_then(|power| value.multiply(&power, reserve))
            }
            ExpressionKind::Binary(binary)
                if matches!(
                    binary.operator,
                    BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply
                ) =>
            {
                let left = self.constant(&binary.left)?;
                let right = self.constant(&binary.right)?;
                match binary.operator {
                    BinaryOperator::Add => left.add(&right, reserve),
                    BinaryOperator::Subtract => left.subtract(&right, reserve),
                    _ => left.multiply(&right, reserve),
                }
            }
            _ => {
                self.report_invalid_modulus(
                    expression.span,
                    String::from("not a constant integer expression"),
                );
                return None;
            }
        };
        let Some(value) = value else {
            self.resource_limit(expression.span, "exact integer storage allocation failed");
            return None;
        };
        if value.magnitude_bits() > self.limits.integer_bits {
            if self.begin_report(expression.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::IntegerMagnitudeLimit,
                        format!(
                            "integer magnitude exceeds the {}-significant-bit limit",
                            self.limits.integer_bits
                        ),
                        expression.span,
                    )
                    .with_label("this value of the modulus is too large")
                    .with_note("the value is rejected rather than truncated or approximated"),
                );
            }
            return None;
        }
        Some(value)
    }

    /// Returns `value` as a modulus, or reports why it is not one.
    pub(super) fn checked_modulus(&mut self, span: Span, value: &ExactInteger) -> Option<Modulus> {
        if let Some(modulus) = Modulus::new(value) {
            return Some(modulus);
        }
        self.report_invalid_modulus(span, invalid_modulus_label(value));
        None
    }

    #[cold]
    #[inline(never)]
    pub(super) fn report_invalid_modulus(&mut self, span: Span, label: String) {
        self.report_invalid_modulus_with_note(span, label, MODULUS_NOTE);
    }

    #[cold]
    #[inline(never)]
    fn report_invalid_modulus_with_note(&mut self, span: Span, label: String, note: &str) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::InvalidModulus,
                    format!("a modulus must be a constant from 2 through 2^{MAX_MODULUS_BITS} - 1"),
                    span,
                )
                .with_label(label)
                .with_note(note),
            );
        }
    }

    /// Reports a deferred modulus's first fault in the current instance.
    #[cold]
    #[inline(never)]
    pub(super) fn report_modulus_fault(&mut self, fault: ModulusFault) {
        match fault {
            ModulusFault::NotConstant(span) => {
                self.report_invalid_modulus_with_note(
                    span,
                    String::from("not a constant integer expression"),
                    STATIC_MODULUS_NOTE,
                );
            }
            ModulusFault::ShiftAmount(span) => self.report_invalid_modulus_with_note(
                span,
                format!(
                    "a shift amount in a modulus is from 0 through {}",
                    self.limits.integer_bits
                ),
                STATIC_MODULUS_NOTE,
            ),
            ModulusFault::Value(span, value) => {
                self.report_invalid_modulus_with_note(
                    span,
                    invalid_modulus_label(&value),
                    STATIC_MODULUS_NOTE,
                );
            }
            ModulusFault::TooLarge(span) => {
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
                        .with_label("this value of the modulus is too large")
                        .with_note("the value is rejected rather than truncated or approximated"),
                    );
                }
            }
            ModulusFault::Storage(span) => {
                self.resource_limit(span, "exact integer storage allocation failed");
            }
        }
    }

    /// Resolves the module's `type` and `types` declarations in source order.
    /// Each `type` names a type for the whole module, written with the
    /// built-in types and the names declared before it. Each `types` names
    /// a list of those types for several functions.
    pub(super) fn analyze_type_declarations(&mut self) {
        let aliases = self.ast.module.types.len();
        let lists = self.ast.module.type_lists.len();
        if self.types.names.try_reserve_exact(aliases).is_err()
            || self.lists.try_reserve_exact(lists).is_err()
        {
            self.resource_limit(
                self.ast.module.span,
                "semantic type name table allocation failed",
            );
            return;
        }
        let mut alias_index = 0;
        let mut list_index = 0;
        while !self.halted {
            let alias = self.ast.module.types.get(alias_index);
            let list = self.ast.module.type_lists.get(list_index);
            match (alias, list) {
                (Some(alias), Some(list)) if alias.span.start() <= list.span.start() => {
                    self.analyze_one_type_declaration(alias);
                    alias_index = alias_index.saturating_add(1);
                }
                (Some(alias), None) => {
                    self.analyze_one_type_declaration(alias);
                    alias_index = alias_index.saturating_add(1);
                }
                (_, Some(list)) => {
                    self.analyze_one_type_list(list);
                    list_index = list_index.saturating_add(1);
                }
                (None, None) => break,
            }
        }
    }

    fn analyze_one_type_declaration(&mut self, declaration: &'ast crate::parser::TypeDeclaration) {
        // One event for the name's lookup.
        if !self.event(declaration.name.span) {
            return;
        }
        let name = declaration.name.text.as_str();
        let span = declaration.name.span;
        let earlier_type = self.types.name(name).map(|declared| declared.span);
        let earlier_list = self
            .lists
            .iter()
            .find(|list| list.name == name)
            .map(|list| list.span);
        if BUILT_IN_TYPE_NAMES.contains(&name) {
            if self.begin_report(span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateTypeName,
                        format!("`{name}` is a built-in type"),
                        span,
                    )
                    .with_label("a `type` declaration cannot name a built-in type")
                    .with_note("the built-in types are `Int`, `Bool`, `Word[n]`, and `Mod[m]`"),
                );
            }
        } else if let Some(first) = earlier_type.or(earlier_list)
            && self.begin_report(span)
        {
            let name = identifier_spelling_for_diagnostic(name);
            let (code, note) = if earlier_type.is_some() {
                (
                    DiagnosticCode::DuplicateTypeName,
                    "each `type` declaration of a module names a different type",
                )
            } else {
                (
                    DiagnosticCode::TypeList,
                    "each `type` or `types` declaration of a module has a name of its own",
                )
            };
            self.diagnostics.push(
                Diagnostic::error(code, format!("duplicate type name `{name}`"), span)
                    .with_label("this declaration repeats a type name")
                    .with_secondary_span(first, "first declaration is here")
                    .with_note(note),
            );
        }
        let ty = self.analyze_type(&declaration.ty, "declared type");
        if self.halted {
            return;
        }
        if earlier_type.is_none() && earlier_list.is_none() && !BUILT_IN_TYPE_NAMES.contains(&name)
        {
            self.types.names.push(DeclaredType { name, span, ty });
        }
    }

    fn analyze_one_type_list(&mut self, declaration: &'ast crate::parser::TypeListDeclaration) {
        if !self.event(declaration.name.span) {
            return;
        }
        let name = declaration.name.text.as_str();
        let span = declaration.name.span;
        let earlier_type = self.types.name(name).map(|declared| declared.span);
        let earlier_list = self
            .lists
            .iter()
            .find(|list| list.name == name)
            .map(|list| list.span);
        let name_ok = if BUILT_IN_TYPE_NAMES.contains(&name) {
            self.report_type_list_builtin(span, name);
            false
        } else if let Some(first) = earlier_type.or(earlier_list) {
            self.report_type_list_duplicate_name(span, name, first);
            false
        } else {
            true
        };
        let mut resolved: Vec<Option<CoreType>> = Vec::new();
        if resolved.try_reserve_exact(declaration.types.len()).is_err() {
            self.resource_limit(declaration.span, "type list storage allocation failed");
            return;
        }
        for ty in &declaration.types {
            let checked = self.analyze_type(ty, "listed type");
            if self.halted {
                return;
            }
            if let Some(checked) = &checked
                && let Some(first) = resolved
                    .iter()
                    .position(|earlier| earlier.as_ref() == Some(checked))
                    .and_then(|position| declaration.types.get(position))
            {
                self.report_type_list_duplicate(name, first.span, ty.span, checked);
            }
            resolved.push(checked);
        }
        let distinct = distinct_types(&resolved);
        let length_ok = declaration.types.len() <= MAX_INSTANCES_PER_FUNCTION;
        if name_ok && distinct && !length_ok {
            self.report_type_list_length(declaration);
        }
        if name_ok {
            self.lists.push(ResolvedTypeList {
                name,
                span,
                valid: distinct && length_ok,
            });
        }
    }

    #[cold]
    #[inline(never)]
    fn report_type_list_builtin(&mut self, span: Span, name: &str) {
        if !self.begin_report(span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(name);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeList,
                format!("`{spelling}` is a built-in type"),
                span,
            )
            .with_label("a `types` declaration cannot name a built-in type")
            .with_note("the built-in types are `Int`, `Bool`, `Word[n]`, and `Mod[m]`"),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_type_list_duplicate_name(&mut self, span: Span, name: &str, earlier: Span) {
        if !self.begin_report(span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(name);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeList,
                format!("duplicate type name `{spelling}`"),
                span,
            )
            .with_label("this declaration repeats a type name")
            .with_secondary_span(earlier, "first declaration is here")
            .with_note("each `type` or `types` declaration of a module has a name of its own"),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_type_list_duplicate(&mut self, name: &str, first: Span, again: Span, ty: &CoreType) {
        if !self.begin_report(again) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(name);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeParameter,
                format!("`{spelling}` lists the type `{ty}` twice"),
                again,
            )
            .with_label("this is the same type as an earlier one")
            .with_secondary_span(first, "first listed here")
            .with_note(
                "a type list names each type once, so that each instance has a type of its own",
            ),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_type_list_length(&mut self, declaration: &crate::parser::TypeListDeclaration) {
        if !self.begin_report(declaration.span) {
            return;
        }
        let spelling = identifier_spelling_for_diagnostic(&declaration.name.text);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeList,
                format!(
                    "`{spelling}` lists {} types, but a type list has at most \
                     {MAX_INSTANCES_PER_FUNCTION}",
                    declaration.types.len()
                ),
                declaration.span,
            )
            .with_label("too many types")
            .with_note(
                "a function has at most 256 instances, and a type parameter has one instance for \
                 each type its list names",
            ),
        );
    }

    #[cold]
    #[inline(never)]
    fn report_type_list_used_as_type(&mut self, syntax: &TypeSyntax) {
        if !self.begin_report(syntax.span) {
            return;
        }
        let name = identifier_spelling_for_diagnostic(&syntax.name.text);
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::TypeList,
                format!("`{name}` names a list of types, not a type"),
                syntax.span,
            )
            .with_label("a `types` declaration is not a type")
            .with_note(
                "name one type the list contains, or a type parameter that stands for one of \
                 them",
            ),
        );
    }

    pub(super) fn analyze_type(&mut self, syntax: &TypeSyntax, role: &str) -> Option<CoreType> {
        if syntax.is_tuple() {
            return self.analyze_tuple_type(syntax, role);
        }
        // The identifier and optional width are distinct parsed-type
        // components; a modulus was evaluated with the module's types.
        if !self.event(syntax.name.span) {
            return None;
        }
        if let Some(width_span) = syntax.width_span
            && !self.event(width_span)
        {
            return None;
        }
        if let Some(length) = &syntax.length
            && !self.event(length.span)
        {
            return None;
        }
        let class = classify_type(self.source, &self.types, syntax);
        if !self.charge_size_events(syntax.span) {
            return None;
        }
        match class {
            TypeClass::Resolved(ty) => Some(ty),
            TypeClass::Modulus(fault) => {
                self.report_modulus_fault(fault);
                None
            }
            TypeClass::Unresolved => None,
            TypeClass::Unindexed => {
                self.resource_limit(syntax.span, "semantic modulus table is inconsistent");
                None
            }
            TypeClass::MissingModulus => {
                let span = syntax.name.span;
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::InvalidModulus,
                            "`Mod` requires a modulus",
                            span,
                        )
                        .with_label("missing modulus")
                        .with_note(MODULUS_NOTE),
                    );
                }
                None
            }
            TypeClass::ArrayOfArrays(length_span) => {
                if self.begin_report(syntax.span) {
                    let name = identifier_spelling_for_diagnostic(&syntax.name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedType,
                            format!("`{name}` already has {MAX_ARRAY_DIMENSIONS} array dimensions"),
                            syntax.span,
                        )
                        .with_label(format!(
                            "arrays have at most {MAX_ARRAY_DIMENSIONS} dimensions"
                        ))
                        .with_secondary_span(length_span, "this length would add a fifth dimension")
                        .with_note(
                            "a row holds scalars, and each `^LENGTH` after a named array type \
                             adds a dimension of its rows",
                        ),
                    );
                }
                None
            }
            TypeClass::ArrayShape(length_span, columns, rows) => {
                if self.begin_report(syntax.span) {
                    let cells = u64::from(columns).saturating_mul(u64::from(rows));
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedArrayLength,
                            format!(
                                "an array shape has {cells} scalar elements, exceeding {MAX_ARRAY_LENGTH}"
                            ),
                            syntax.span,
                        )
                        .with_label("array shape exceeds the scalar element limit")
                        .with_secondary_span(length_span, "outer axis length")
                        .with_note(format!(
                            "every axis is positive and the product of the axes is at most \
                             {MAX_ARRAY_LENGTH}"
                        )),
                    );
                }
                None
            }
            TypeClass::ArrayOfTuples(length_span) => {
                if self.begin_report(syntax.span) {
                    let name = identifier_spelling_for_diagnostic(&syntax.name.text);
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedType,
                            format!("`{name}` is a tuple type, so this is an array of tuples"),
                            syntax.span,
                        )
                        .with_label("arrays of tuples are not part of Orange 2026")
                        .with_secondary_span(
                            length_span,
                            "this length would make each element a tuple",
                        )
                        .with_note("an array's elements are `Int`, `Bool`, words, or residues"),
                    );
                }
                None
            }
            TypeClass::UnsupportedArrayLength(length_span) => {
                self.report_unsupported_array_length(length_span);
                None
            }
            TypeClass::ArrayLengthValue(length_span, value) => {
                self.report_array_length_value(length_span, &value);
                None
            }
            TypeClass::Size(fault) => {
                self.report_size_fault(fault);
                None
            }
            TypeClass::UnsupportedWordWidth(width_span) => {
                if self.begin_report(width_span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedWordWidth,
                            "`Word` width must be exactly 8, 16, 32, or 64",
                            width_span,
                        )
                        .with_label("unsupported word width")
                        .with_note("word widths do not coerce, truncate, or wrap"),
                    );
                }
                None
            }
            TypeClass::MissingWordWidth => {
                let span = syntax.name.span;
                if self.begin_report(span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedWordWidth,
                            "`Word` requires an exact width of 8, 16, 32, or 64",
                            span,
                        )
                        .with_label("missing word width")
                        .with_note("write the width in decimal, as in `Word[32]`"),
                    );
                }
                None
            }
            TypeClass::Unsupported => {
                if self
                    .ast
                    .module
                    .type_lists
                    .iter()
                    .any(|list| list.name.text == syntax.name.text)
                {
                    self.report_type_list_used_as_type(syntax);
                    return None;
                }
                if self.begin_report(syntax.span) {
                    let declared_later = syntax.width_span.is_none()
                        && self
                            .ast
                            .module
                            .types
                            .iter()
                            .any(|declaration| declaration.name.text == syntax.name.text);
                    let name = identifier_spelling_for_diagnostic(&syntax.name.text);
                    let note = if declared_later {
                        format!(
                            "`{name}` is declared by a later `type` declaration; a `type` \
                             declaration uses only the names declared before it"
                        )
                    } else {
                        String::from(
                            "types are resolved contextually and never inferred by spelling \
                             similarity",
                        )
                    };
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::UnsupportedType,
                            format!("unsupported {role} `{name}`"),
                            syntax.span,
                        )
                        .with_label(format!("the admitted types are {ADMITTED_TYPES}"))
                        .with_note(note),
                    );
                }
                None
            }
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn report_unsupported_array_length(&mut self, span: Span) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedArrayLength,
                    format!(
                        "an array length must be a decimal integer from 1 through \
                         {MAX_ARRAY_LENGTH}"
                    ),
                    span,
                )
                .with_label("unsupported array length")
                .with_note(
                    "write the length in decimal without leading zeros, as in `Word[32]^16`",
                ),
            );
        }
    }

    #[cold]
    #[inline(never)]
    pub(super) fn report_array_length_value(&mut self, span: Span, value: &ExactInteger) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::UnsupportedArrayLength,
                    value.to_i64().map_or_else(
                        || format!("this array length is far outside 1 through {MAX_ARRAY_LENGTH}"),
                        |value| {
                            format!(
                                "this array length is {value}, but an array has 1 through \
                                 {MAX_ARRAY_LENGTH} elements"
                            )
                        },
                    ),
                    span,
                )
                .with_label("unsupported array length in this instance")
                .with_note(format!(
                    "a length written with sizes is computed in each instance of its function, \
                     and every instance's lengths are from 1 through {MAX_ARRAY_LENGTH}"
                )),
            );
        }
    }

    pub(super) fn analyze_literal(
        &mut self,
        expected: &CoreType,
        literal: &IntegerLiteral,
    ) -> Option<CoreValue> {
        // One literal event precedes shared exact-magnitude decoding. Word sign
        // and range classification follows only after the magnitude is valid.
        if !self.event(literal.span) {
            return None;
        }
        let magnitude = self.parse_magnitude(literal, self.limits.integer_bits)?;
        if let Some(modulus) = expected.modulus() {
            return self.residue_literal(expected, modulus, literal, magnitude);
        }
        let Some(maximum) = word_maximum(expected) else {
            return Some(CoreValue::Int(ExactInteger::new(
                literal.negative,
                magnitude,
            )));
        };
        if literal.negative {
            if self.begin_report(literal.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::NegativeWordLiteral,
                        format!("`{expected}` literals cannot be negative"),
                        literal.span,
                    )
                    .with_label(format!(
                        "negative value is outside the range 0 through {maximum}"
                    ))
                    .with_note("fixed-width words do not wrap or coerce negative integers"),
                );
            }
            return None;
        }
        let value = magnitude.to_u64().filter(|value| *value <= maximum);
        if let Some(value) = value.and_then(|value| CoreValue::word_from_u64(expected, value)) {
            Some(value)
        } else {
            if self.begin_report(literal.magnitude_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::WordLiteralOutOfRange,
                        format!("literal is outside the range of `{expected}`"),
                        literal.magnitude_span,
                    )
                    .with_label(format!("expected a value from 0 through {maximum}"))
                    .with_note("fixed-width words do not truncate or wrap out-of-range integers"),
                );
            }
            None
        }
    }

    /// Gives the residue of a literal of `Mod[m]`, whose magnitude n is less
    /// than m: n itself, or m - n when the literal is negative.
    pub(super) fn residue_literal(
        &mut self,
        expected: &CoreType,
        modulus: Modulus,
        literal: &IntegerLiteral,
        magnitude: Magnitude,
    ) -> Option<CoreValue> {
        let reserve = self.reserve_range_limbs;
        let value = ExactInteger::new(false, magnitude);
        if !modulus.contains(&value) {
            if self.begin_report(literal.magnitude_span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::WordLiteralOutOfRange,
                        format!("literal is outside the range of `{expected}`"),
                        literal.magnitude_span,
                    )
                    .with_label("the literal's magnitude is not less than the modulus")
                    .with_note(
                        "a literal of `Mod[m]` has a magnitude n less than m, and `-n` stands \
                         for m - n; residues do not reduce out-of-range literals",
                    ),
                );
            }
            return None;
        }
        let residue = if literal.negative && !value.is_zero() {
            modulus
                .to_exact(reserve)
                .and_then(|modulus| modulus.subtract(&value, reserve))
        } else {
            Some(value)
        };
        let residue = residue.and_then(|residue| Residue::new(modulus, residue));
        if residue.is_none() {
            self.resource_limit(literal.span, "exact integer storage allocation failed");
        }
        residue.map(CoreValue::Mod)
    }

    pub(super) fn parse_magnitude(
        &mut self,
        literal: &IntegerLiteral,
        bit_limit: usize,
    ) -> Option<Magnitude> {
        let Some(spelling) = self.source.slice(literal.magnitude_span) else {
            self.resource_limit(
                literal.span,
                "integer literal span does not belong to the analyzed source",
            );
            return None;
        };
        // Prefix inspection is one event whether the decimal default or an
        // explicit binary/hexadecimal prefix is selected.
        if !self.event(literal.magnitude_span) {
            return None;
        }
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
        let mut significant = false;
        for character in digits.chars() {
            if character == '_' {
                continue;
            }
            let Some(digit) = character.to_digit(radix) else {
                self.resource_limit(
                    literal.magnitude_span,
                    "semantic analysis received a malformed integer AST",
                );
                return None;
            };
            significant |= digit != 0;
            if significant && !self.event(literal.magnitude_span) {
                return None;
            }
            if !magnitude.multiply_add_with_reservation(radix, digit, self.reserve_magnitude_limb) {
                self.resource_limit(
                    literal.magnitude_span,
                    "exact integer magnitude storage allocation failed",
                );
                return None;
            }
            if magnitude.bit_len() > bit_limit {
                if self.begin_report(literal.magnitude_span) {
                    self.diagnostics.push(
                        Diagnostic::error(
                            DiagnosticCode::IntegerMagnitudeLimit,
                            format!(
                                "integer magnitude exceeds the {}-significant-bit limit",
                                self.limits.integer_bits
                            ),
                            literal.magnitude_span,
                        )
                        .with_label("exact integer is too large for this semantic fragment")
                        .with_note("the literal is rejected rather than truncated or approximated"),
                    );
                }
                return None;
            }
        }
        Some(magnitude)
    }
}
