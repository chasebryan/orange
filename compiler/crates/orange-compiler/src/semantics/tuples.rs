//! Tuples: tuple types, tuple literals, the selection of an element by its
//! position, and the tuple patterns of bindings and accumulators.

use super::*;

/// The note of a tuple type whose element is a tuple.
const TUPLE_ELEMENT_NOTE: &str = "a tuple's elements are `Int`, `Bool`, words, residues, and \
     arrays of them";

/// Returns the type of a pattern without reporting: the declared type of a
/// name, or the tuple of a tuple pattern's declared types.
pub(super) fn silent_pattern_type(
    source: &SourceFile,
    table: &TypeTable<'_>,
    pattern: &Pattern,
) -> Option<CoreType> {
    match pattern {
        Pattern::Name(typed) => silent_type(source, table, &typed.ty),
        Pattern::Tuple(tuple) => {
            let mut elements = Vec::new();
            elements.try_reserve_exact(tuple.elements.len()).ok()?;
            for typed in &tuple.elements {
                elements.push(silent_type(source, table, &typed.ty)?);
            }
            TupleType::new(&elements).map(CoreType::Tuple)
        }
    }
}

/// Returns the extent that Core records as a pattern's name: the name, or
/// the whole tuple pattern.
pub(super) fn pattern_name_span(pattern: &Pattern) -> Span {
    match pattern {
        Pattern::Name(typed) => typed.name.span,
        Pattern::Tuple(tuple) => tuple.span,
    }
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Resolves the type of a binding's or an accumulator's pattern: the
    /// declared type of a name, or the tuple of a tuple pattern's declared
    /// types, each resolved and reported in order.
    ///
    /// Returns `None` when any of them does not resolve.
    pub(super) fn analyze_pattern_type(
        &mut self,
        pattern: &Pattern,
        role: &str,
    ) -> Option<CoreType> {
        let tuple = match pattern {
            Pattern::Name(typed) => return self.analyze_type(&typed.ty, role),
            Pattern::Tuple(tuple) => tuple,
        };
        // One event for the pattern.
        if !self.event(tuple.span) {
            return None;
        }
        let mut elements = Vec::new();
        if elements.try_reserve_exact(tuple.elements.len()).is_err() {
            self.resource_limit(tuple.span, "tuple type storage allocation failed");
            return None;
        }
        let mut resolved = true;
        for typed in &tuple.elements {
            match self.analyze_type(&typed.ty, role) {
                Some(ty) => elements.push(ty),
                None => resolved = false,
            }
            if self.halted {
                return None;
            }
        }
        if !resolved {
            return None;
        }
        self.tuple_of(tuple.span, &elements, |index| {
            tuple.elements.get(index).map(|typed| &typed.ty)
        })
    }

    /// Resolves a tuple type `(T0, T1, ...)`: each element type in order,
    /// and then the tuple of them.
    pub(super) fn analyze_tuple_type(
        &mut self,
        syntax: &TypeSyntax,
        role: &str,
    ) -> Option<CoreType> {
        // One event for the tuple type.
        if !self.event(syntax.span) {
            return None;
        }
        let mut elements = Vec::new();
        if elements.try_reserve_exact(syntax.elements.len()).is_err() {
            self.resource_limit(syntax.span, "tuple type storage allocation failed");
            return None;
        }
        let mut resolved = true;
        for element in &syntax.elements {
            match self.analyze_type(element, role) {
                Some(ty) => elements.push(ty),
                None => resolved = false,
            }
            if self.halted {
                return None;
            }
        }
        if !resolved {
            return None;
        }
        self.tuple_of(syntax.span, &elements, |index| syntax.elements.get(index))
    }

    /// Returns the tuple type of resolved element types, or reports the
    /// first element that is itself a tuple, written as `syntax(index)`.
    fn tuple_of<'syntax>(
        &mut self,
        span: Span,
        elements: &[CoreType],
        syntax: impl Fn(usize) -> Option<&'syntax TypeSyntax>,
    ) -> Option<CoreType> {
        if let Some((index, element)) = elements
            .iter()
            .enumerate()
            .find(|(_, element)| element.as_tuple().is_some())
        {
            let element_syntax = syntax(index);
            let element_span = element_syntax.map_or(span, |syntax| syntax.span);
            if self.begin_report(element_span) {
                // A tuple type written out, as a pattern's name may have,
                // is shown as its type; a declared name as its name.
                let name = element_syntax
                    .filter(|syntax| syntax.elements.is_empty())
                    .map(|syntax| identifier_spelling_for_diagnostic(&syntax.name.text));
                let message = match name {
                    Some(name) => {
                        format!("`{name}` is a tuple type, so this is a tuple of tuples")
                    }
                    None => format!("`{element}` is a tuple type, so this is a tuple of tuples"),
                };
                self.diagnostics.push(
                    Diagnostic::error(DiagnosticCode::UnsupportedType, message, element_span)
                        .with_label("a tuple holds no tuple")
                        .with_note(TUPLE_ELEMENT_NOTE),
                );
            }
            return None;
        }
        let Some(tuple) = TupleType::new(elements) else {
            self.resource_limit(span, "tuple type exceeds its element limits");
            return None;
        };
        Some(CoreType::Tuple(tuple))
    }

    /// Returns the name Core records for a pattern: the name itself, or the
    /// names of a tuple pattern in parentheses, as in `(sum, carry)`.
    pub(super) fn pattern_core_name(&mut self, pattern: &Pattern) -> Option<String> {
        let tuple = match pattern {
            Pattern::Name(typed) => {
                return self.copy_core_name(&typed.name.text, typed.name.span);
            }
            Pattern::Tuple(tuple) => tuple,
        };
        let length = tuple
            .elements
            .iter()
            .map(|typed| typed.name.text.len().saturating_add(2))
            .fold(2_usize, usize::saturating_add);
        let mut name = String::new();
        if !(self.reserve_core_name)(&mut name, length) {
            self.resource_limit(tuple.span, "typed Core name storage allocation failed");
            return None;
        }
        name.push('(');
        for (index, typed) in tuple.elements.iter().enumerate() {
            if index != 0 {
                name.push_str(", ");
            }
            name.push_str(&typed.name.text);
        }
        name.push(')');
        Some(name)
    }

    /// Checks a tuple literal `(e0, e1, ...)` against `expected`, which must
    /// be a tuple type with as many elements; each element is checked
    /// against its element type, in order.
    pub(super) fn check_tuple(
        &mut self,
        expression: &'ast Expression,
        tuple: &'ast TupleExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let supplied = tuple.elements.len();
        let Some(tuple_type) = expected.as_tuple() else {
            if self.begin_report(expression.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!("a tuple cannot have type `{expected}`"),
                        expression.span,
                    )
                    .with_label(format!("expected `{expected}`"))
                    .with_note("a tuple is written where a tuple type `(T, U, ...)` is required"),
                );
            }
            return false;
        };
        let length = tuple_type.elements().len();
        if length != supplied {
            if self.begin_report(expression.span) {
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::TypeMismatch,
                        format!(
                            "this tuple has {supplied} elements, but `{expected}` has {length}"
                        ),
                        expression.span,
                    )
                    .with_label(format!("expected {length} elements"))
                    .with_note("a tuple lists every element of its type exactly once, in order"),
                );
            }
            return false;
        }
        let mut elements_checked = true;
        for (element, ty) in tuple.elements.iter().zip(tuple_type.elements()) {
            elements_checked &= self.check_expression(element, ty, context, scope, output);
            if self.halted {
                return false;
            }
        }
        let Ok(elements) = u32::try_from(supplied) else {
            self.resource_limit(
                expression.span,
                "tuple element count exceeds the u32 representation limit",
            );
            return false;
        };
        elements_checked
            && self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::Tuple { elements },
            )
    }

    /// Checks `base.k` against `expected`.
    ///
    /// The base's type is found without reporting, as an indexed array's
    /// is. It must be a tuple with an element at position `k`, and that
    /// element's type must be `expected`; the base is then checked against
    /// its own type, so errors inside it are still reported.
    pub(super) fn check_project(
        &mut self,
        expression: &'ast Expression,
        project: &'ast ProjectExpression,
        expected: &CoreType,
        context: &mut BodyContext<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        output: &mut BodyOutput<'_>,
    ) -> bool {
        let Some(base_type) = self.leaf_type(&project.base, context, scope) else {
            // The base's own check reports why it has no type and stops
            // before comparing with the type passed here.
            self.check_untyped(&project.base, expected, context, scope, output);
            return false;
        };
        let Some(tuple_type) = base_type.as_tuple() else {
            self.report_not_a_tuple(&project.base, &base_type);
            self.check_expression(&project.base, &base_type, context, scope, output);
            return false;
        };
        let position = project.position;
        let Some(element) = tuple_type.element(position).cloned() else {
            let last = tuple_type.len().saturating_sub(1);
            if self.begin_report(project.position_span) {
                // The position as written: one too large for 32 bits is
                // kept as the largest, which no tuple has.
                let written = self.source.slice(project.position_span).unwrap_or_default();
                self.diagnostics.push(
                    Diagnostic::error(
                        DiagnosticCode::IndexOutOfRange,
                        format!("`{base_type}` has no element {written}"),
                        project.position_span,
                    )
                    .with_label(format!("its elements are numbered 0 through {last}"))
                    .with_note("a tuple's elements are counted from zero"),
                );
            }
            self.check_expression(&project.base, &base_type, context, scope, output);
            return false;
        };
        let element_matches = element == *expected;
        if !element_matches && self.begin_report(expression.span) {
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TypeMismatch,
                    format!("this element has type `{element}`, but `{expected}` is required here"),
                    expression.span,
                )
                .with_label(format!("expected `{expected}`"))
                .with_note("Orange has no implicit conversions between types"),
            );
        }
        let base = self.check_expression(&project.base, &base_type, context, scope, output);
        base && element_matches
            && self.push_node(
                output,
                expression.span,
                expected.clone(),
                CoreNodeKind::Project { index: position },
            )
    }

    #[cold]
    #[inline(never)]
    fn report_not_a_tuple(&mut self, base: &Expression, base_type: &CoreType) {
        if !self.begin_report(base.span) {
            return;
        }
        let note = if base_type.as_array().is_some() {
            "an array's element is selected by an index, such as `x[0]`"
        } else {
            "`.k` selects element k of a value of a tuple type `(T, U, ...)`"
        };
        self.diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::NotATuple,
                format!("only a tuple has elements selected by position, but this has type `{base_type}`"),
                base.span,
            )
            .with_label(format!("`{base_type}` is not a tuple"))
            .with_note(note),
        );
    }
}
