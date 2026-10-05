//! Functions over every word width, with sizes checked for every value.
//!
//! A parameter `W: Word` stands for `Word[8]`, `Word[16]`, `Word[32]`, and
//! `Word[64]`. The function is proved once at each width, at the corners of
//! its size ranges. Those corners decide an affine use of the sizes. Only a
//! specialization a call names is lowered into Core.

use std::cell::Cell;

use super::*;

/// The word widths a `W: Word` parameter stands for, narrowest first.
pub(super) const WORD_WIDTHS: [u32; 4] = [8, 16, 32, 64];

/// One specialization reserved for a call.
#[derive(Clone, Copy)]
pub(super) struct SpecKey {
    pub(super) function: usize,
    pub(super) values: [u32; MAX_SIZES_PER_FUNCTION],
    pub(super) len: usize,
}

/// A specialization's identity and types, shared with later callers.
pub(super) struct StoredSpec {
    pub(super) values: [u32; MAX_SIZES_PER_FUNCTION],
    pub(super) len: usize,
    pub(super) id: CoreFunctionId,
    pub(super) signature: InstanceSignature,
}

const AFFINE_INDEX_NOTE: &str = "an index checked for every value of a size is affine in those \
     sizes: literals, loop indices, and sizes with `+`, `-`, and `*`, where a product has one \
     factor that names no size; `/` and `%` do not name a size";

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Proves `function` at every word width and every corner of its sizes.
    /// The Core of those corners is discarded.
    pub(super) fn prove_universal(
        &mut self,
        function: &'ast FunctionDeclaration,
        body: &'ast TypedBody,
        signature: &Signature<'ast>,
        scope: &ModuleScope<'_, 'ast>,
        edges: &mut Vec<CallEdge>,
    ) -> bool {
        let Some(ranges) = signature.ranges else {
            self.resource_limit(function.span, "semantic size table is inconsistent");
            return false;
        };
        let Some(corners) = corners(&function.sizes, ranges) else {
            self.resource_limit(function.span, "semantic size table is inconsistent");
            return false;
        };
        let Some(word_at) = function.sizes.iter().position(SizeParameter::is_word) else {
            self.resource_limit(function.span, "semantic size table is inconsistent");
            return false;
        };
        self.checking_universal = true;
        self.proving = true;
        for (width_index, _) in WORD_WIDTHS.iter().enumerate() {
            for corner in &corners {
                let mut values = *corner;
                let Some(slot) = values.get_mut(word_at) else {
                    self.resource_limit(function.span, "semantic size table is inconsistent");
                    self.checking_universal = false;
                    self.proving = false;
                    return false;
                };
                let Some(index) = u32::try_from(width_index).ok() else {
                    self.checking_universal = false;
                    self.proving = false;
                    return false;
                };
                *slot = index;
                let instance = Instance {
                    parameters: &function.sizes,
                    values,
                };
                self.enter_instance(instance, &signature.listed);
                let Some(id) = CoreFunctionId::from_index(0) else {
                    self.checking_universal = false;
                    self.proving = false;
                    return false;
                };
                let context = fresh_context(id, instance, function, body);
                let reported = self.diagnostics.len();
                let pending = self.analyze_typed_function(function, body, context, scope, edges);
                if self.halted {
                    self.checking_universal = false;
                    self.proving = false;
                    return false;
                }
                if self.diagnostics.len() > reported || pending.is_none() {
                    self.name_instance(function, instance, reported);
                    self.checking_universal = false;
                    self.proving = false;
                    self.enter_instance(Instance::NONE, &[]);
                    return self.diagnostics.len() == reported && !self.halted;
                }
            }
        }
        self.checking_universal = false;
        self.proving = false;
        self.enter_instance(Instance::NONE, &[]);
        true
    }

    /// Lowers each specialization reserved so far, and any a lowered body
    /// requests, into `pending` in identity order.
    pub(super) fn lower_specializations(
        &mut self,
        signatures: &[Option<Signature<'ast>>],
        scope: &ModuleScope<'_, 'ast>,
        pending: &mut Vec<PendingFunction>,
        edges: &mut Vec<CallEdge>,
    ) {
        let mut cursor = pending.len();
        self.checking_universal = true;
        while cursor < self.spec_keys.len() {
            if self.halted || !self.diagnostics.is_empty() {
                break;
            }
            let Some(key) = self.spec_keys.get(cursor).copied() else {
                break;
            };
            cursor = cursor.saturating_add(1);
            let Some(function) = self.ast.module.functions.get(key.function) else {
                self.resource_limit(
                    self.ast.module.span,
                    "semantic specialization table is inconsistent",
                );
                break;
            };
            let Some(signature) = signatures.get(key.function).and_then(Option::as_ref) else {
                self.resource_limit(function.span, "semantic signature table is inconsistent");
                break;
            };
            let id = signature
                .specializations
                .borrow()
                .iter()
                .find_map(|stored| {
                    (stored.len == key.len && same_prefix(&stored.values, &key.values, key.len))
                        .then_some(stored.id)
                });
            let Some(id) = id else {
                self.resource_limit(
                    function.span,
                    "semantic specialization table is inconsistent",
                );
                break;
            };
            let FunctionBody::Typed(body) = &function.body else {
                self.resource_limit(
                    function.span,
                    "semantic specialization table is inconsistent",
                );
                break;
            };
            let instance = Instance {
                parameters: &function.sizes,
                values: key.values,
            };
            self.enter_instance(instance, &signature.listed);
            let context = fresh_context(id, instance, function, body);
            let reported = self.diagnostics.len();
            let checked = self.analyze_typed_function(function, body, context, scope, edges);
            if self.halted {
                break;
            }
            if self.diagnostics.len() > reported {
                self.name_instance(function, instance, reported);
                break;
            }
            let Some(checked) = checked else {
                self.resource_limit(function.span, "semantic specialization lowering failed");
                break;
            };
            if (self.reserve_pending_function_slot)(pending) {
                pending.push(checked);
            } else {
                self.resource_limit(
                    function.span,
                    "semantic analysis could not allocate pending function storage",
                );
                break;
            }
        }
        self.checking_universal = false;
        self.enter_instance(Instance::NONE, &[]);
    }

    /// Returns the specialization a call names, reserving it when this
    /// module declares the function.
    pub(super) fn called_universal(
        &mut self,
        expression: &Expression,
        call: &CallExpression,
        signature: &Signature<'ast>,
    ) -> Option<(CoreFunctionId, InstanceSignature)> {
        let values = self.universal_values(expression, call, signature)?;
        let types = self.instance_types(signature, &values)?;
        if self.proving {
            return Some((CoreFunctionId::from_index(0)?, types));
        }
        let len = signature.sizes.len();
        if let Some(found) = signature
            .specializations
            .borrow()
            .iter()
            .find(|stored| stored.len == len && same_prefix(&stored.values, &values, len))
        {
            return Some((found.id, found.signature.clone()));
        }
        let function = self.local_function(signature);
        let Some(function) = function else {
            self.report_not_for_every(
                expression.span,
                "this specialization is not part of the module that declares the function",
                "a call from another module uses a specialization that module builds",
                "call the function from the module that declares it, at these widths and sizes",
            );
            return None;
        };
        if self.spec_keys.len() >= MAX_INSTANCES_PER_FUNCTION {
            let message =
                format!("a module builds at most {MAX_INSTANCES_PER_FUNCTION} specializations");
            self.report_not_for_every(
                expression.span,
                &message,
                "too many specializations",
                "each call that names a new width and new sizes builds one specialization",
            );
            return None;
        }
        let index = self.spec_keys.len();
        let id = self
            .id_offset
            .checked_add(self.preassigned)
            .and_then(|base| base.checked_add(index))
            .and_then(CoreFunctionId::from_index)?;
        if self.spec_keys.try_reserve(1).is_err() {
            self.resource_limit(expression.span, "specialization storage allocation failed");
            return None;
        }
        self.spec_keys.push(SpecKey {
            function,
            values,
            len,
        });
        let stored = StoredSpec {
            values,
            len,
            id,
            signature: types.clone(),
        };
        if signature
            .specializations
            .borrow_mut()
            .try_reserve(1)
            .is_err()
        {
            self.resource_limit(expression.span, "specialization storage allocation failed");
            return None;
        }
        signature.specializations.borrow_mut().push(stored);
        Some((id, types))
    }

    /// Resolves a universal call's result type without reserving anything.
    pub(super) fn universal_result(
        &self,
        call: &CallExpression,
        signature: &Signature<'ast>,
    ) -> Option<CoreType> {
        let values = self.silent_universal_values(call, signature)?;
        self.instantiate(signature.result, signature, &values)
    }

    fn universal_values(
        &mut self,
        expression: &Expression,
        call: &CallExpression,
        signature: &Signature<'ast>,
    ) -> Option<[u32; MAX_SIZES_PER_FUNCTION]> {
        let ranges = signature.ranges?;
        if call.sizes().len() != ranges.count() {
            self.report_size_count(expression.span, call, signature.sizes);
            return None;
        }
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        for (position, entry) in call.sizes().iter().enumerate() {
            let parameter = signature.sizes.get(position)?;
            let slot = values.get_mut(position)?;
            if parameter.is_word() || parameter.is_type() {
                *slot = self.called_type(entry, &call.callee, signature, position)?;
                continue;
            }
            let value = self.size_value(entry)?;
            let (start, end) = ranges.range(position)?;
            match value
                .to_i64()
                .and_then(|value| u32::try_from(value).ok())
                .filter(|value| (start..end).contains(value))
            {
                Some(value) => *slot = value,
                None => {
                    self.report_size_outside(
                        entry.span,
                        &call.callee,
                        parameter,
                        (start, end),
                        &value,
                    );
                    return None;
                }
            }
        }
        Some(values)
    }

    fn silent_universal_values(
        &self,
        call: &CallExpression,
        signature: &Signature<'ast>,
    ) -> Option<[u32; MAX_SIZES_PER_FUNCTION]> {
        let ranges = signature.ranges?;
        if call.sizes().len() != ranges.count() {
            return None;
        }
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        for (position, entry) in call.sizes().iter().enumerate() {
            let parameter = signature.sizes.get(position)?;
            let slot = values.get_mut(position)?;
            if parameter.is_word() || parameter.is_type() {
                let TypeArgument::Type(ty) = self.type_argument(entry) else {
                    return None;
                };
                let listed = signature.listed.get(position)?;
                *slot = u32::try_from(
                    listed
                        .iter()
                        .position(|candidate| candidate.as_ref() == Some(&ty))?,
                )
                .ok()?;
            } else {
                let (start, end) = ranges.range(position)?;
                let value = self
                    .types
                    .sizes
                    .value(self.source, entry)
                    .ok()?
                    .to_i64()
                    .and_then(|value| u32::try_from(value).ok())
                    .filter(|value| (start..end).contains(value))?;
                *slot = value;
            }
        }
        Some(values)
    }

    fn instance_types(
        &self,
        signature: &Signature<'ast>,
        values: &[u32; MAX_SIZES_PER_FUNCTION],
    ) -> Option<InstanceSignature> {
        let mut parameters = Vec::new();
        if parameters
            .try_reserve_exact(signature.parameters.len())
            .is_err()
        {
            return None;
        }
        for parameter in signature.parameters {
            parameters.push(self.instantiate(&parameter.ty, signature, values));
        }
        let result_type = self.instantiate(signature.result, signature, values);
        Some(InstanceSignature {
            parameters,
            result_type,
        })
    }

    fn instantiate(
        &self,
        syntax: &TypeSyntax,
        signature: &Signature<'ast>,
        values: &[u32; MAX_SIZES_PER_FUNCTION],
    ) -> Option<CoreType> {
        if syntax.is_tuple() {
            let mut elements = Vec::new();
            if elements.try_reserve_exact(syntax.elements().len()).is_err() {
                return None;
            }
            for element in syntax.elements() {
                elements.push(self.instantiate(element, signature, values)?);
            }
            return TupleType::new(&elements).map(CoreType::Tuple);
        }
        let scalar = self.instantiate_scalar(syntax, signature, values)?;
        match syntax.length() {
            None => Some(scalar),
            Some(length) => {
                let count = self.instantiate_length(length, signature, values)?;
                ArrayType::new(&scalar, count).map(CoreType::Array)
            }
        }
    }

    fn instantiate_scalar(
        &self,
        syntax: &TypeSyntax,
        signature: &Signature<'ast>,
        values: &[u32; MAX_SIZES_PER_FUNCTION],
    ) -> Option<CoreType> {
        match (syntax.name().text.as_str(), syntax.width_span()) {
            ("Int", None) => Some(CoreType::Int),
            ("Bool", None) => Some(CoreType::Bool),
            ("Word", Some(span)) => {
                let width = match self.source.slice(span) {
                    Some("8") => Some(8),
                    Some("16") => Some(16),
                    Some("32") => Some(32),
                    Some("64") => Some(64),
                    _ => None,
                }?;
                CoreType::word_of_width(width)
            }
            ("Mod", None) => {
                let expression = syntax.modulus()?;
                let resolved = self.types.modulus(expression)?;
                if resolved.dependent {
                    return None;
                }
                resolved.modulus.map(CoreType::Mod)
            }
            (name, None) => {
                if let Some(position) = signature.sizes.iter().position(|parameter| {
                    (parameter.is_word() || parameter.is_type()) && parameter.name.text == name
                }) {
                    let index = usize::try_from(*values.get(position)?).ok()?;
                    return signature.listed.get(position)?.get(index)?.clone();
                }
                self.types.name(name)?.ty.clone()
            }
            _ => None,
        }
    }

    fn instantiate_length(
        &self,
        size: &Size,
        signature: &Signature<'ast>,
        values: &[u32; MAX_SIZES_PER_FUNCTION],
    ) -> Option<u32> {
        let Some(expression) = size.expression() else {
            return array_length(self.source, size.span())
                .filter(|length| (1..=MAX_ARRAY_LENGTH).contains(length));
        };
        let scope = SizeScope {
            instance: Instance {
                parameters: signature.sizes,
                values: *values,
            },
            types: [const { None }; MAX_SIZES_PER_FUNCTION],
            bits: self.limits.integer_bits,
            reserve: self.reserve_range_limbs,
            reserve_limb: self.reserve_magnitude_limb,
            evaluated: Cell::new(0),
            affine: false,
        };
        let length = scope
            .value(self.source, expression)
            .ok()?
            .to_i64()
            .and_then(|value| u32::try_from(value).ok())?;
        (1..=MAX_ARRAY_LENGTH).contains(&length).then_some(length)
    }

    fn local_function(&self, signature: &Signature<'ast>) -> Option<usize> {
        self.ast.module.functions.iter().position(|function| {
            !function.sizes.is_empty()
                && function.sizes.len() == signature.sizes.len()
                && std::ptr::eq(function.sizes.as_ptr(), signature.sizes.as_ptr())
        })
    }
}

fn same_prefix(left: &[u32], right: &[u32], len: usize) -> bool {
    (0..len).all(|index| left.get(index) == right.get(index))
}

fn fresh_context<'ast>(
    id: CoreFunctionId,
    instance: Instance<'ast>,
    function: &'ast FunctionDeclaration,
    body: &'ast TypedBody,
) -> BodyContext<'ast> {
    BodyContext {
        id,
        instance,
        name: &function.name,
        parameters: &function.parameters,
        parameter_types: Vec::new(),
        bindings: &body.bindings,
        binding_types: Vec::new(),
        loop_scopes: Vec::new(),
        blocks: Vec::new(),
        finished_blocks: Vec::new(),
        loops: Vec::new(),
        conditionals: Vec::new(),
    }
}

/// The corners of the size parameters: each size at its first value or its
/// last. A word parameter is left at zero for the caller to fill.
fn corners(
    parameters: &[SizeParameter],
    ranges: SizeRanges,
) -> Option<Vec<[u32; MAX_SIZES_PER_FUNCTION]>> {
    let positions: Vec<usize> = parameters
        .iter()
        .enumerate()
        .filter(|(_, parameter)| parameter.is_size())
        .map(|(position, _)| position)
        .collect();
    let bits = u32::try_from(positions.len()).ok()?;
    let combinations = 1_usize.checked_shl(bits)?;
    let mut corners = Vec::new();
    if corners.try_reserve_exact(combinations).is_err() {
        return None;
    }
    for mask in 0..combinations {
        let mut values = [0; MAX_SIZES_PER_FUNCTION];
        for (bit, position) in positions.iter().copied().enumerate() {
            let (start, end) = ranges.range(position)?;
            let last = end.checked_sub(1)?;
            let flag = 1_usize.checked_shl(u32::try_from(bit).ok()?)?;
            let value = if (mask & flag) == 0 { start } else { last };
            *values.get_mut(position)? = value;
        }
        corners.push(values);
    }
    Some(corners)
}

/// The degree of `index` in the size parameters: `Some(0)` or `Some(1)`
/// when it is affine, or `None` when a product names two sizes or `/` or
/// `%` names a size.
///
/// Parser-established expression height bounds this recursion.
pub(super) fn size_degree(index: &Expression, context: &BodyContext<'_>) -> Option<u8> {
    match &index.kind {
        ExpressionKind::Literal(_) => Some(0),
        ExpressionKind::Name(name) => match context.resolve(&name.text) {
            NameResolution::Size(_) => Some(1),
            _ => Some(0),
        },
        ExpressionKind::Parenthesized(inner) => size_degree(inner, context),
        ExpressionKind::Unary(unary) if unary.operator == UnaryOperator::Negate => {
            size_degree(&unary.operand, context)
        }
        ExpressionKind::Binary(binary)
            if matches!(
                binary.operator,
                BinaryOperator::Add | BinaryOperator::Subtract
            ) =>
        {
            let left = size_degree(&binary.left, context)?;
            let right = size_degree(&binary.right, context)?;
            Some(left.max(right))
        }
        ExpressionKind::Binary(binary) if binary.operator == BinaryOperator::Multiply => {
            let left = size_degree(&binary.left, context)?;
            let right = size_degree(&binary.right, context)?;
            (left == 0 || right == 0).then_some(left.max(right))
        }
        ExpressionKind::Binary(binary) if binary.operator.is_division() => {
            let left = size_degree(&binary.left, context)?;
            let right = size_degree(&binary.right, context)?;
            (left == 0 && right == 0).then_some(0)
        }
        ExpressionKind::Conversion(conversion) => size_degree(&conversion.operand, context),
        ExpressionKind::Conditional(conditional) => {
            let mut degree = 0_u8;
            for value in conditional
                .arms
                .iter()
                .map(|arm| &arm.value)
                .chain(std::iter::once(&conditional.otherwise))
            {
                degree = degree.max(size_degree(value, context)?);
            }
            Some(degree)
        }
        _ => Some(0),
    }
}

pub(super) const fn affine_index_note() -> &'static str {
    AFFINE_INDEX_NOTE
}
