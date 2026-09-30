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
    /// from 1 through 256, at the length's span.
    ArrayLengthValue(Span, ExactInteger),
    /// A length written with sizes that has no value.
    Size(SizeFault),
    MissingModulus,
    /// A declared name whose element type is already an array, followed by
    /// the span of `^LENGTH`.
    ArrayOfArrays(Span),
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
}

pub(super) struct DeclaredType<'ast> {
    pub(super) name: &'ast str,
    pub(super) span: Span,
    /// The declared type, or `None` when its declaration was reported.
    pub(super) ty: Option<CoreType>,
}

impl TypeTable<'_> {
    pub(super) const fn new() -> Self {
        Self {
            moduli: Vec::new(),
            names: Vec::new(),
            sizes: SizeScope {
                instance: Instance::NONE,
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
        (TypeClass::Resolved(CoreType::Array(_)), Some(length)) => {
            TypeClass::ArrayOfArrays(length.span)
        }
        (TypeClass::Resolved(CoreType::Tuple(_)), Some(length)) => {
            TypeClass::ArrayOfTuples(length.span)
        }
        (TypeClass::Resolved(element), Some(length)) => {
            match table.sizes.array_length(source, length) {
                Length::Admitted(count) => ArrayType::new(&element, count)
                    .map_or(TypeClass::UnsupportedArrayLength(length.span), |array| {
                        TypeClass::Resolved(CoreType::Array(array))
                    }),
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

/// Decodes an array length written as a decimal integer with no leading
/// zero and no underscore, as word widths are written.
pub(super) fn array_length(source: &SourceFile, span: Span) -> Option<u32> {
    let spelling = source.slice(span)?;
    let canonical = !spelling.is_empty()
        && !spelling.starts_with('0')
        && spelling.bytes().all(|byte| byte.is_ascii_digit());
    if !canonical || spelling.len() > 3 {
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

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Evaluates every modulus written in the module's `type` declarations
    /// and typed `spec` functions, once each and in source order, before any
    /// type is resolved.
    pub(super) fn resolve_moduli(&mut self) {
        let module = &self.ast.module;
        for declaration in &module.types {
            self.resolve_modulus(&declaration.ty);
        }
        for function in &module.functions {
            let (FunctionBody::Typed(body), FunctionKind::Spec) = (&function.body, function.kind)
            else {
                continue;
            };
            for parameter in &function.parameters {
                self.resolve_modulus(&parameter.ty);
            }
            self.resolve_modulus(&body.result_type);
            for binding in &body.bindings {
                self.resolve_pattern_moduli(&binding.pattern);
                self.resolve_moduli_within(&binding.value);
            }
            self.resolve_moduli_within(&body.expression);
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
        let modulus = self
            .constant(expression)
            .and_then(|value| self.checked_modulus(expression.span, &value));
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
        let label = if value.magnitude_bits() > MAX_MODULUS_BITS && !value.is_negative() {
            format!("this modulus has {} bits", value.magnitude_bits())
        } else if value.magnitude_bits() <= 64 {
            format!("this modulus is {value}")
        } else {
            String::from("this modulus is negative")
        };
        self.report_invalid_modulus(span, label);
        None
    }

    #[cold]
    #[inline(never)]
    pub(super) fn report_invalid_modulus(&mut self, span: Span, label: String) {
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::InvalidModulus,
                    format!("a modulus must be a constant from 2 through 2^{MAX_MODULUS_BITS} - 1"),
                    span,
                )
                .with_label(label)
                .with_note(MODULUS_NOTE),
            );
        }
    }

    /// Resolves the module's `type` declarations in order. Each names a type
    /// for the whole module, written with the built-in types and the names
    /// declared before it.
    pub(super) fn analyze_type_declarations(&mut self) {
        let declarations = &self.ast.module.types;
        if self
            .types
            .names
            .try_reserve_exact(declarations.len())
            .is_err()
        {
            self.resource_limit(
                self.ast.module.span,
                "semantic type name table allocation failed",
            );
            return;
        }
        for declaration in declarations {
            // One event for the name's lookup.
            if !self.event(declaration.name.span) {
                return;
            }
            let name = declaration.name.text.as_str();
            let span = declaration.name.span;
            let earlier = self.types.name(name).map(|declared| declared.span);
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
            } else if let Some(first) = earlier
                && self.begin_report(span)
            {
                let name = identifier_spelling_for_diagnostic(name);
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::DuplicateTypeName,
                        format!("duplicate type name `{name}`"),
                        span,
                    )
                    .with_label("this declaration repeats a type name")
                    .with_secondary_span(first, "first declaration is here")
                    .with_note("each `type` declaration of a module names a different type"),
                );
            }
            let ty = self.analyze_type(&declaration.ty, "declared type");
            if self.halted {
                return;
            }
            if earlier.is_none() && !BUILT_IN_TYPE_NAMES.contains(&name) {
                self.types.names.push(DeclaredType { name, span, ty });
            }
        }
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
                            format!("`{name}` is an array type, so this is an array of arrays"),
                            syntax.span,
                        )
                        .with_label("arrays of arrays are not part of Orange 2026")
                        .with_secondary_span(
                            length_span,
                            "this length would make each element an array",
                        )
                        .with_note("an array's elements are `Int`, `Bool`, words, or residues"),
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
                        || {
                            format!(
                                "this array length is far outside 1 through {MAX_ARRAY_LENGTH}"
                            )
                        },
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
                .with_note(
                    "a length written with sizes is computed in each instance of its function, \
                     and every instance's lengths are from 1 through 256",
                ),
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
