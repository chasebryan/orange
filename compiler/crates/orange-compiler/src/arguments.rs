//! Bounded decoding of canonical, concretely typed host arguments.
//!
//! This local argument frame is not a solver, model, proof or evidence format.
//! Expected checked types supply every word width, modulus and aggregate shape.

use crate::core::{CoreArray, CoreTuple, CoreType, CoreValue, ExactInteger, Magnitude};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::parser::MAX_PARAMETERS_PER_FUNCTION;
use crate::source::{SourceFile, TextOffset};

/// Maximum scalar and aggregate values retained across one argument vector.
pub const MAX_ARGUMENT_VALUE_NODES: usize = 16 * 262_144;
/// Maximum retained base-2^32 magnitude limbs across one argument vector.
pub const MAX_ARGUMENT_INTEGER_LIMBS: usize = 4 * 1_048_576;
/// Maximum byte scans, value transitions and integer limb work in one decode.
pub const MAX_ARGUMENT_DECODE_WORK: usize = 64 * 1_048_576;

/// A complete vector decoded against exact concrete types.
///
/// Storage cannot be replaced or partially constructed outside this module.
///
/// ```compile_fail
/// use orange_compiler::DecodedArguments;
///
/// fn replace_complete_arguments(arguments: &mut DecodedArguments) {
///     arguments.values.clear();
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct DecodedArguments {
    values: Vec<CoreValue>,
}

impl DecodedArguments {
    /// Returns every decoded argument in parameter order.
    #[must_use]
    pub fn values(&self) -> &[CoreValue] {
        &self.values
    }

    /// Consumes this complete vector and returns its typed values.
    #[must_use]
    pub fn into_values(self) -> Vec<CoreValue> {
        self.values
    }
}

/// The complete result of decoding one local argument source.
///
/// ```compile_fail
/// use orange_compiler::ArgumentDecodeResult;
///
/// fn replace_decoding_result(result: &mut ArgumentDecodeResult) {
///     result.arguments = None;
/// }
/// ```
#[derive(Debug, Eq, PartialEq)]
pub struct ArgumentDecodeResult {
    arguments: Option<DecodedArguments>,
    diagnostics: Vec<Diagnostic>,
}

impl ArgumentDecodeResult {
    /// Returns the complete vector, or `None` after any failure.
    #[must_use]
    pub const fn arguments(&self) -> Option<&DecodedArguments> {
        self.arguments.as_ref()
    }

    /// Consumes this result and returns its complete vector, if produced.
    #[must_use]
    pub fn into_arguments(self) -> Option<DecodedArguments> {
        self.arguments
    }

    /// Returns the first decoding error at its original input byte span.
    ///
    /// If diagnostic storage cannot be reserved, this slice is empty while
    /// [`Self::has_errors`] remains true. Partial arguments are never returned.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether decoding failed, including unreportable allocation failure.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.arguments.is_none()
    }
}

/// Decodes one canonical argument vector against concrete expected types.
///
/// The input is `[VALUE, VALUE]`, or `[]`, with at most one final LF. Values
/// use [`CoreValue`]'s display spelling: signed decimal integers, `true` or
/// `false`, lowercase fixed-width hexadecimal words, decimal least residues,
/// bracketed arrays/rows and parenthesized tuples. Separators are exactly
/// comma-space. No expressions, comments, coercions or residue reduction occur.
/// Input is bounded by [`crate::source::MAX_SOURCE_BYTES`]; additional public
/// node, magnitude-storage and work budgets apply across the whole vector.
#[must_use]
pub fn decode_arguments(source: &SourceFile, expected: &[CoreType]) -> ArgumentDecodeResult {
    decode_with_limits(source, expected, Limits::DEFAULT, Reservations::DEFAULT)
}

#[derive(Clone, Copy)]
struct Limits {
    nodes: usize,
    limbs: usize,
    work: usize,
}

impl Limits {
    const DEFAULT: Self = Self {
        nodes: MAX_ARGUMENT_VALUE_NODES,
        limbs: MAX_ARGUMENT_INTEGER_LIMBS,
        work: MAX_ARGUMENT_DECODE_WORK,
    };
}

#[derive(Clone, Copy)]
struct Reservations {
    values: fn(&mut Vec<CoreValue>, usize) -> bool,
    limb: fn(&mut Vec<u32>) -> bool,
}

impl Reservations {
    const DEFAULT: Self = Self {
        values: |values, count| values.try_reserve_exact(count).is_ok(),
        limb: |limbs| limbs.try_reserve_exact(1).is_ok(),
    };
}

#[derive(Clone, Copy)]
enum FailureKind {
    Noncanonical,
    Mismatch,
    Resource,
}

struct Failure {
    kind: FailureKind,
    position: usize,
    argument: Option<usize>,
    message: &'static str,
}

struct Decoder<'source> {
    source: &'source SourceFile,
    position: usize,
    argument: Option<usize>,
    budget: Limits,
    reservations: Reservations,
}

impl Decoder<'_> {
    fn failure(&self, kind: FailureKind, message: &'static str) -> Failure {
        Failure {
            kind,
            position: self.position,
            argument: self.argument,
            message,
        }
    }

    fn spend(&mut self, count: usize) -> Result<(), Failure> {
        self.budget.work = self.budget.work.checked_sub(count).ok_or_else(|| {
            self.failure(
                FailureKind::Resource,
                "argument decoding exceeded its work budget",
            )
        })?;
        Ok(())
    }

    fn peek(&mut self) -> Result<Option<u8>, Failure> {
        self.spend(1)?;
        Ok(self.source.text().as_bytes().get(self.position).copied())
    }

    fn advance(&mut self) -> Result<(), Failure> {
        self.position = self.position.checked_add(1).ok_or_else(|| {
            self.failure(FailureKind::Resource, "argument byte offset overflowed")
        })?;
        Ok(())
    }

    fn take(&mut self, byte: u8) -> Result<(), Failure> {
        if self.peek()? != Some(byte) {
            return Err(self.failure(
                FailureKind::Noncanonical,
                "argument spelling is not canonical",
            ));
        }
        self.advance()
    }

    fn node(&mut self) -> Result<(), Failure> {
        self.spend(1)?;
        self.budget.nodes = self.budget.nodes.checked_sub(1).ok_or_else(|| {
            self.failure(
                FailureKind::Resource,
                "argument decoding exceeded its value-node budget",
            )
        })?;
        Ok(())
    }

    fn reserve_values(&self, count: usize) -> Result<Vec<CoreValue>, Failure> {
        let mut values = Vec::new();
        if !(self.reservations.values)(&mut values, count) {
            return Err(self.failure(
                FailureKind::Resource,
                "argument value storage could not be reserved",
            ));
        }
        Ok(values)
    }
}

enum FrameKind<'types> {
    Arguments(&'types [CoreType]),
    Array(crate::core::ArrayType),
    Tuple(crate::core::TupleType),
}

struct Frame<'types> {
    kind: FrameKind<'types>,
    values: Vec<CoreValue>,
}

impl Frame<'_> {
    fn count(&self) -> Option<usize> {
        match &self.kind {
            FrameKind::Arguments(types) => Some(types.len()),
            FrameKind::Array(array) => usize::try_from(array.length()).ok(),
            FrameKind::Tuple(tuple) => Some(tuple.elements().len()),
        }
    }

    fn closing(&self) -> u8 {
        if matches!(self.kind, FrameKind::Tuple(_)) {
            b')'
        } else {
            b']'
        }
    }

    fn next_type(&self) -> Option<CoreType> {
        match &self.kind {
            FrameKind::Arguments(types) => types.get(self.values.len()).cloned(),
            FrameKind::Array(array) => Some(array.element()),
            FrameKind::Tuple(tuple) => tuple.elements().get(self.values.len()).cloned(),
        }
    }
}

fn decode_with_limits(
    source: &SourceFile,
    expected: &[CoreType],
    limits: Limits,
    reservations: Reservations,
) -> ArgumentDecodeResult {
    let mut decoder = Decoder {
        source,
        position: 0,
        argument: None,
        budget: limits,
        reservations,
    };
    let result = decode(&mut decoder, expected);
    match result {
        Ok(values) => ArgumentDecodeResult {
            arguments: Some(DecodedArguments { values }),
            diagnostics: Vec::new(),
        },
        Err(failure) => failed(source, failure),
    }
}

fn failed(source: &SourceFile, failure: Failure) -> ArgumentDecodeResult {
    let mut diagnostics = Vec::new();
    if diagnostics.try_reserve_exact(1).is_ok() {
        let code = match failure.kind {
            FailureKind::Noncanonical => DiagnosticCode::NoncanonicalArgumentValue,
            FailureKind::Mismatch => DiagnosticCode::ArgumentValueMismatch,
            FailureKind::Resource => DiagnosticCode::ArgumentDecodeResourceLimit,
        };
        let end = source
            .text()
            .get(failure.position..)
            .and_then(|tail| tail.chars().next())
            .and_then(|character| failure.position.checked_add(character.len_utf8()))
            .unwrap_or(failure.position);
        if let Some(span) = u32::try_from(failure.position)
            .ok()
            .zip(u32::try_from(end).ok())
            .and_then(|(start, end)| source.span(TextOffset::new(start), TextOffset::new(end)))
        {
            let mut diagnostic = Diagnostic::error(code, failure.message, span)
                .with_note("no argument vector was produced; values require exact canonical spelling and concrete types");
            if let Some(argument) = failure.argument {
                diagnostic.add_note(format!("argument index {argument} (zero-based)"));
            }
            diagnostics.push(diagnostic);
        }
    }
    ArgumentDecodeResult {
        arguments: None,
        diagnostics,
    }
}

fn scalar_array_nodes(ty: &CoreType) -> Option<usize> {
    match ty {
        CoreType::Array(array) => {
            let leaves = usize::try_from(array.scalar_length()).ok()?;
            let rows = if matches!(array.element(), CoreType::Array(_)) {
                usize::try_from(array.length()).ok()?
            } else {
                0
            };
            leaves.checked_add(rows)?.checked_add(1)
        }
        CoreType::Tuple(_) => None,
        _ => Some(1),
    }
}

fn required_nodes(decoder: &mut Decoder<'_>, types: &[CoreType]) -> Result<usize, Failure> {
    let mut nodes = 0_usize;
    for ty in types {
        decoder.spend(1)?;
        let count = if let CoreType::Tuple(tuple) = ty {
            let mut count = 1_usize;
            for element in tuple.elements() {
                decoder.spend(1)?;
                count = scalar_array_nodes(element)
                    .and_then(|nodes| count.checked_add(nodes))
                    .ok_or_else(|| {
                        decoder.failure(FailureKind::Resource, "argument shape count overflowed")
                    })?;
            }
            count
        } else {
            scalar_array_nodes(ty).ok_or_else(|| {
                decoder.failure(FailureKind::Resource, "argument shape count overflowed")
            })?
        };
        nodes = nodes.checked_add(count).ok_or_else(|| {
            decoder.failure(FailureKind::Resource, "argument shape count overflowed")
        })?;
    }
    Ok(nodes)
}

fn decode(decoder: &mut Decoder<'_>, expected: &[CoreType]) -> Result<Vec<CoreValue>, Failure> {
    if expected.len() > MAX_PARAMETERS_PER_FUNCTION
        || required_nodes(decoder, expected)? > decoder.budget.nodes
    {
        return Err(decoder.failure(
            FailureKind::Resource,
            "argument types exceed the value-node or parameter budget",
        ));
    }
    decoder.take(b'[')?;
    let mut frames = Vec::new();
    frames.try_reserve_exact(4).map_err(|_| {
        decoder.failure(
            FailureKind::Resource,
            "argument frame storage could not be reserved",
        )
    })?;
    frames.push(Frame {
        kind: FrameKind::Arguments(expected),
        values: decoder.reserve_values(expected.len())?,
    });
    loop {
        decoder.spend(1)?;
        let frame = frames
            .last()
            .ok_or_else(|| decoder.failure(FailureKind::Resource, "argument frame is missing"))?;
        let count = frame.count().ok_or_else(|| {
            decoder.failure(FailureKind::Resource, "argument shape is unrepresentable")
        })?;
        let closing = frame.closing();
        let next = decoder.peek()?;
        if frame.values.len() == count {
            if next.is_some_and(|byte| {
                byte == b','
                    || (count == 0
                        && (byte == b'['
                            || byte == b'('
                            || byte == b'-'
                            || byte.is_ascii_alphanumeric()))
            }) {
                return Err(decoder.failure(
                    FailureKind::Mismatch,
                    "argument vector or aggregate has too many elements",
                ));
            }
            decoder.take(closing)?;
            let frame = frames.pop().ok_or_else(|| {
                decoder.failure(FailureKind::Resource, "argument frame is missing")
            })?;
            decoder.spend(count)?;
            let value = match frame.kind {
                FrameKind::Arguments(_) => {
                    decoder.argument = None;
                    if decoder.peek()? == Some(b'\n') {
                        decoder.advance()?;
                    }
                    if decoder.peek()?.is_some() {
                        return Err(decoder.failure(
                            FailureKind::Noncanonical,
                            "syntax follows the canonical argument vector",
                        ));
                    }
                    return Ok(frame.values);
                }
                FrameKind::Array(array) => {
                    CoreArray::new(array, frame.values).map(CoreValue::Array)
                }
                FrameKind::Tuple(tuple) => {
                    CoreTuple::new(tuple, frame.values).map(CoreValue::Tuple)
                }
            }
            .ok_or_else(|| {
                decoder.failure(
                    FailureKind::Mismatch,
                    "argument aggregate does not match its concrete type",
                )
            })?;
            let parent = frames.last_mut().ok_or_else(|| {
                decoder.failure(FailureKind::Resource, "argument parent frame is missing")
            })?;
            parent.values.push(value);
            continue;
        }
        if matches!(frame.kind, FrameKind::Arguments(_)) {
            decoder.argument = Some(frame.values.len());
        }
        if next == Some(closing) {
            return Err(decoder.failure(
                FailureKind::Mismatch,
                "argument vector or aggregate has too few elements",
            ));
        }
        if !frame.values.is_empty() {
            decoder.take(b',')?;
            decoder.take(b' ')?;
        }
        let ty = frame.next_type().ok_or_else(|| {
            decoder.failure(FailureKind::Resource, "argument element type is missing")
        })?;
        decoder.node()?;
        let kind = match &ty {
            CoreType::Array(array) => Some((
                FrameKind::Array(*array),
                b'[',
                usize::try_from(array.length()).ok(),
            )),
            CoreType::Tuple(tuple) => Some((
                FrameKind::Tuple(tuple.clone()),
                b'(',
                Some(tuple.elements().len()),
            )),
            _ => None,
        };
        if let Some((kind, opening, count)) = kind {
            if decoder.peek()?.is_some_and(|byte| {
                byte != opening
                    && (byte == b'['
                        || byte == b'('
                        || byte == b'-'
                        || byte.is_ascii_alphanumeric())
            }) {
                return Err(decoder.failure(
                    FailureKind::Mismatch,
                    "argument aggregate has the wrong shape",
                ));
            }
            decoder.take(opening)?;
            let count = count.ok_or_else(|| {
                decoder.failure(
                    FailureKind::Resource,
                    "argument element count is unrepresentable",
                )
            })?;
            if frames.len() >= 4 {
                return Err(decoder.failure(
                    FailureKind::Resource,
                    "argument aggregate depth exceeds checked Core types",
                ));
            }
            frames.push(Frame {
                kind,
                values: decoder.reserve_values(count)?,
            });
        } else {
            if matches!(decoder.peek()?, Some(b'[' | b'(')) {
                return Err(decoder.failure(
                    FailureKind::Mismatch,
                    "argument aggregate has the wrong shape",
                ));
            }
            let value = scalar(decoder, &ty)?;
            frames
                .last_mut()
                .ok_or_else(|| decoder.failure(FailureKind::Resource, "argument frame is missing"))?
                .values
                .push(value);
        }
    }
}

fn scalar(decoder: &mut Decoder<'_>, ty: &CoreType) -> Result<CoreValue, Failure> {
    match ty {
        CoreType::Bool => {
            let (spelling, value) = if decoder.peek()? == Some(b't') {
                (b"true".as_slice(), true)
            } else {
                (b"false".as_slice(), false)
            };
            for byte in spelling {
                decoder.take(*byte)?;
            }
            Ok(CoreValue::Bool(value))
        }
        CoreType::Word8 | CoreType::Word16 | CoreType::Word32 | CoreType::Word64 => {
            decoder.take(b'0')?;
            decoder.take(b'x')?;
            let digits = ty
                .word_bits()
                .and_then(|bits| bits.checked_div(4))
                .ok_or_else(|| {
                    decoder.failure(FailureKind::Resource, "word width is unavailable")
                })?;
            let mut value = 0_u64;
            for _ in 0..digits {
                let digit = match decoder.peek()? {
                    Some(byte @ b'0'..=b'9') => {
                        u64::from(byte.checked_sub(b'0').ok_or_else(|| {
                            decoder.failure(FailureKind::Resource, "word digit is unrepresentable")
                        })?)
                    }
                    Some(byte @ b'a'..=b'f') => u64::from(
                        byte.checked_sub(b'a')
                            .and_then(|digit| digit.checked_add(10))
                            .ok_or_else(|| {
                                decoder
                                    .failure(FailureKind::Resource, "word digit is unrepresentable")
                            })?,
                    ),
                    _ => {
                        return Err(decoder.failure(
                            FailureKind::Noncanonical,
                            "word requires its exact width in lowercase hexadecimal",
                        ));
                    }
                };
                value = value
                    .checked_mul(16)
                    .and_then(|value| value.checked_add(digit))
                    .ok_or_else(|| {
                        decoder.failure(
                            FailureKind::Resource,
                            "word value overflowed its exact width",
                        )
                    })?;
                decoder.advance()?;
            }
            if decoder.peek()?.is_some_and(|byte| byte.is_ascii_hexdigit()) {
                return Err(decoder.failure(
                    FailureKind::Noncanonical,
                    "word has more digits than its exact width",
                ));
            }
            CoreValue::word_from_u64(ty, value)
                .ok_or_else(|| decoder.failure(FailureKind::Resource, "word type is unavailable"))
        }
        CoreType::Int | CoreType::Mod(_) => {
            let integer = decimal(decoder)?;
            if let CoreType::Mod(modulus) = ty {
                decoder.spend(
                    integer
                        .magnitude_digits()
                        .checked_add(modulus.bits().div_ceil(32))
                        .ok_or_else(|| {
                            decoder.failure(
                                FailureKind::Resource,
                                "residue comparison work overflowed",
                            )
                        })?,
                )?;
                crate::core::Residue::new(*modulus, integer)
                    .map(CoreValue::Mod)
                    .ok_or_else(|| {
                        decoder.failure(
                            FailureKind::Mismatch,
                            "residue is outside the expected modulus domain",
                        )
                    })
            } else {
                Ok(CoreValue::Int(integer))
            }
        }
        CoreType::Array(_) | CoreType::Tuple(_) => {
            Err(decoder.failure(FailureKind::Mismatch, "a scalar was required"))
        }
    }
}

fn decimal(decoder: &mut Decoder<'_>) -> Result<ExactInteger, Failure> {
    let negative = decoder.peek()? == Some(b'-');
    if negative {
        decoder.advance()?;
    }
    let first = decoder.peek()?;
    if first == Some(b'0') {
        decoder.advance()?;
        if negative || decoder.peek()?.is_some_and(|byte| byte.is_ascii_digit()) {
            return Err(decoder.failure(
                FailureKind::Noncanonical,
                "decimal integers forbid negative zero and leading zeroes",
            ));
        }
        return Ok(ExactInteger::new(false, Magnitude::zero()));
    }
    if !first.is_some_and(|byte| matches!(byte, b'1'..=b'9')) {
        return Err(decoder.failure(
            FailureKind::Noncanonical,
            "integer requires canonical decimal digits",
        ));
    }
    let mut magnitude = Magnitude::zero();
    loop {
        let mut chunk = 0_u32;
        let mut digits = 0_u32;
        while digits < 9 {
            let Some(byte @ b'0'..=b'9') = decoder.peek()? else {
                break;
            };
            chunk = chunk
                .checked_mul(10)
                .and_then(|value| value.checked_add(u32::from(byte.checked_sub(b'0')?)))
                .ok_or_else(|| {
                    decoder.failure(FailureKind::Resource, "decimal chunk overflowed")
                })?;
            digits = digits.checked_add(1).ok_or_else(|| {
                decoder.failure(FailureKind::Resource, "decimal chunk length overflowed")
            })?;
            decoder.advance()?;
        }
        if digits == 0 {
            break;
        }
        let limb_count = magnitude.bit_len().div_ceil(32);
        decoder.spend(
            limb_count
                .checked_mul(2)
                .and_then(|work| work.checked_add(2))
                .ok_or_else(|| {
                    decoder.failure(FailureKind::Resource, "integer conversion work overflowed")
                })?,
        )?;
        let multiplier = 10_u32.checked_pow(digits).ok_or_else(|| {
            decoder.failure(FailureKind::Resource, "decimal chunk multiplier overflowed")
        })?;
        let mut reservation_failed = false;
        let reserve = decoder.reservations.limb;
        let converted = magnitude.multiply_add_with_reservation(multiplier, chunk, |limbs| {
            let Some(remaining) = decoder.budget.limbs.checked_sub(1) else {
                reservation_failed = true;
                return false;
            };
            decoder.budget.limbs = remaining;
            if !reserve(limbs) {
                reservation_failed = true;
                return false;
            }
            true
        });
        if !converted || reservation_failed {
            return Err(decoder.failure(
                FailureKind::Resource,
                "argument integer magnitude storage could not be reserved",
            ));
        }
        if magnitude.bit_len() > crate::core::MAX_EXACT_INTEGER_BITS {
            return Err(decoder.failure(
                FailureKind::Resource,
                "integer exceeds the 16384-significant-bit limit",
            ));
        }
        if digits < 9 {
            break;
        }
    }
    Ok(ExactInteger::new(negative, magnitude))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ArrayType, Modulus, Residue, TupleType};
    use crate::source::{MAX_SOURCE_BYTES, SourceMap};

    fn with_source<T>(text: impl Into<String>, run: impl FnOnce(&SourceFile) -> T) -> T {
        let mut sources = SourceMap::new();
        let id = sources.add("arguments.values", text.into()).unwrap();
        run(sources.get(id).unwrap())
    }

    fn decoded(text: &str, types: &[CoreType]) -> DecodedArguments {
        with_source(text, |source| {
            let result = decode_arguments(source, types);
            assert_eq!(result.diagnostics(), [], "{text}");
            result.into_arguments().unwrap()
        })
    }

    fn error(text: &str, types: &[CoreType], code: DiagnosticCode) {
        with_source(text, |source| {
            let result = decode_arguments(source, types);
            assert!(result.has_errors(), "{text}");
            assert!(result.arguments().is_none());
            assert_eq!(result.diagnostics().len(), 1, "{text}");
            assert_eq!(result.diagnostics()[0].code(), code, "{text}");
            assert!(
                source
                    .slice(result.diagnostics()[0].primary_span())
                    .is_some()
            );
            assert!(result.into_arguments().is_none());
        });
    }

    fn modulus(value: u64) -> Modulus {
        let exact =
            ExactInteger::from_u64(value, |limbs, count| limbs.try_reserve_exact(count).is_ok())
                .unwrap();
        Modulus::new(&exact).unwrap()
    }

    fn vector(values: &[CoreValue]) -> String {
        let rendered: Vec<_> = values.iter().map(ToString::to_string).collect();
        format!("[{}]", rendered.join(", "))
    }

    #[test]
    fn production_argument_budgets_and_all_scalar_display_spellings() {
        assert_eq!(MAX_ARGUMENT_VALUE_NODES, 4_194_304);
        assert_eq!(MAX_ARGUMENT_INTEGER_LIMBS, 4_194_304);
        assert_eq!(MAX_ARGUMENT_DECODE_WORK, 67_108_864);
        let types = [
            CoreType::Int,
            CoreType::Int,
            CoreType::Bool,
            CoreType::Bool,
            CoreType::Word8,
            CoreType::Word16,
            CoreType::Word32,
            CoreType::Word64,
            CoreType::Mod(modulus(257)),
        ];
        let input = "[-123456789012345678901234567890, 0, true, false, 0x00, 0x00ff, 0x89abcdef, 0xffffffffffffffff, 256]";
        let values = decoded(input, &types).into_values();
        assert_eq!(vector(&values), input);
        assert_eq!(values.iter().map(CoreValue::ty).collect::<Vec<_>>(), types);
        assert_eq!(decoded("[]", &[]).values(), []);
        assert_eq!(decoded("[]\n", &[]).values(), []);
        assert_eq!(
            vector(decoded(&format!("{input}\n"), &types).values()),
            input
        );
    }

    #[test]
    fn aggregate_rows_and_tuple_fields_keep_exact_shapes_and_moduli() {
        let array = ArrayType::new(&CoreType::Word16, 2).unwrap();
        let matrix = ArrayType::new(&CoreType::Array(array), 2).unwrap();
        let residues = ArrayType::new(&CoreType::Mod(modulus(7)), 3).unwrap();
        let tuple = TupleType::new(&[
            CoreType::Bool,
            CoreType::Array(matrix),
            CoreType::Array(residues),
            CoreType::Int,
        ])
        .unwrap();
        let types = [CoreType::Tuple(tuple.clone()), CoreType::Array(array)];
        let input =
            "[(true, [[0x0000, 0xffff], [0x0010, 0x0001]], [0, 6, 3], -123), [0x0001, 0x0002]]";
        let arguments = decoded(input, &types);
        assert_eq!(vector(arguments.values()), input);
        let CoreValue::Tuple(value) = &arguments.values()[0] else {
            panic!()
        };
        assert_eq!(value.ty(), &tuple);
        let CoreValue::Array(rows) = &value.elements()[1] else {
            panic!()
        };
        assert_eq!(rows.ty(), matrix);
        assert_eq!(rows.elements().len(), 2);
        assert!(
            rows.elements()
                .iter()
                .all(|row| row.ty() == CoreType::Array(array))
        );
        let CoreValue::Array(residues) = &value.elements()[2] else {
            panic!()
        };
        assert!(
            residues
                .elements()
                .iter()
                .all(|value| value.ty() == CoreType::Mod(modulus(7)))
        );
    }

    #[test]
    fn rejects_noncanonical_partial_and_trailing_values_without_artifacts() {
        let ints = [CoreType::Int];
        for input in [
            "", "[1]\n\n", "[1]\r\n", " [1]", "[1] ", "[1]\n ", "[1]x", "[+1]", "[-0]", "[00]",
            "[01]", "[0x01]", "[1_0]", "[1.0]", "[1", "[", "[/*x*/1]", "[1 + 1]",
        ] {
            error(input, &ints, DiagnosticCode::NoncanonicalArgumentValue);
        }
        for input in [
            "[True]",
            "[False]",
            "[1]",
            "[true ]",
            "[tru]",
            "[true/*x*/]",
        ] {
            error(
                input,
                &[CoreType::Bool],
                DiagnosticCode::NoncanonicalArgumentValue,
            );
        }
        for input in [
            "[0xff]",
            "[0X00ff]",
            "[0x00FF]",
            "[0x000ff]",
            "[255]",
            "[-0x00ff]",
            "[0x00_f]",
        ] {
            error(
                input,
                &[CoreType::Word16],
                DiagnosticCode::NoncanonicalArgumentValue,
            );
        }
        for input in ["[1,2]", "[1,  2]", "[1 , 2]", "[1,\n2]"] {
            error(
                input,
                &[CoreType::Int, CoreType::Int],
                DiagnosticCode::NoncanonicalArgumentValue,
            );
        }
    }

    #[test]
    fn rejects_argument_counts_aggregate_shapes_and_residue_domains() {
        error(
            "[]",
            &[CoreType::Int],
            DiagnosticCode::ArgumentValueMismatch,
        );
        error(
            "[1, 2]",
            &[CoreType::Int],
            DiagnosticCode::ArgumentValueMismatch,
        );
        error("[1]", &[], DiagnosticCode::ArgumentValueMismatch);
        error(
            "[1,]",
            &[CoreType::Int],
            DiagnosticCode::ArgumentValueMismatch,
        );
        error(
            "[1, ]",
            &[CoreType::Int],
            DiagnosticCode::ArgumentValueMismatch,
        );
        let row = ArrayType::new(&CoreType::Int, 2).unwrap();
        let matrix = ArrayType::new(&CoreType::Array(row), 2).unwrap();
        for input in ["[[]]", "[[1]]", "[[1, 2, 3]]", "[(1, 2)]"] {
            error(
                input,
                &[CoreType::Array(row)],
                DiagnosticCode::ArgumentValueMismatch,
            );
        }
        for input in [
            "[[1, 2]]",
            "[[[1, 2]]]",
            "[[[1, 2], [3]]]",
            "[[[1, 2], [3, 4], [5, 6]]]",
        ] {
            error(
                input,
                &[CoreType::Array(matrix)],
                DiagnosticCode::ArgumentValueMismatch,
            );
        }
        let tuple = TupleType::new(&[CoreType::Int, CoreType::Bool]).unwrap();
        for input in ["[[1, true]]", "[(1)]", "[(1, true, false)]"] {
            error(
                input,
                &[CoreType::Tuple(tuple.clone())],
                DiagnosticCode::ArgumentValueMismatch,
            );
        }
        for input in ["[-1]", "[7]", "[8]", "[18446744073709551616]"] {
            error(
                input,
                &[CoreType::Mod(modulus(7))],
                DiagnosticCode::ArgumentValueMismatch,
            );
        }
        let arguments = decoded("[6]", &[CoreType::Mod(modulus(7))]);
        let CoreValue::Mod(residue) = &arguments.values()[0] else {
            panic!()
        };
        assert_eq!(residue.modulus(), modulus(7));
        assert_ne!(arguments.values()[0].ty(), CoreType::Mod(modulus(11)));
        assert!(Residue::new(modulus(7), residue.value().clone()).is_some());
    }

    #[test]
    fn every_word_width_rejects_truncation_and_matches_all_boundary_values() {
        for (ty, low, high) in [
            (CoreType::Word8, "0x00", "0xff"),
            (CoreType::Word16, "0x0000", "0xffff"),
            (CoreType::Word32, "0x00000000", "0xffffffff"),
            (CoreType::Word64, "0x0000000000000000", "0xffffffffffffffff"),
        ] {
            for spelling in [low, high] {
                let input = format!("[{spelling}]");
                assert_eq!(
                    vector(decoded(&input, std::slice::from_ref(&ty)).values()),
                    input
                );
            }
            error(
                &format!("[{high}0]"),
                std::slice::from_ref(&ty),
                DiagnosticCode::NoncanonicalArgumentValue,
            );
        }
    }

    #[test]
    fn largest_521_bit_modulus_accepts_m_minus_one_and_rejects_m() {
        let reserve = |limbs: &mut Vec<u32>, count| limbs.try_reserve_exact(count).is_ok();
        let power = ExactInteger::power_of_two(521, reserve).unwrap();
        let one = ExactInteger::from_u64(1, reserve).unwrap();
        let modulus_value = power.subtract(&one, reserve).unwrap();
        let max_modulus = Modulus::new(&modulus_value).unwrap();
        assert_eq!(max_modulus.bits(), 521);
        let maximum_residue = modulus_value.subtract(&one, reserve).unwrap();
        let input = format!("[{maximum_residue}]");
        let arguments = decoded(&input, &[CoreType::Mod(max_modulus)]);
        assert_eq!(vector(arguments.values()), input);
        error(
            &format!("[{modulus_value}]"),
            &[CoreType::Mod(max_modulus)],
            DiagnosticCode::ArgumentValueMismatch,
        );
        let neighboring_modulus = Modulus::new(&maximum_residue).unwrap();
        error(
            &input,
            &[CoreType::Mod(neighboring_modulus)],
            DiagnosticCode::ArgumentValueMismatch,
        );
        let common_value = maximum_residue.subtract(&one, reserve).unwrap();
        let common_input = format!("[{common_value}]");
        let a = decoded(&common_input, &[CoreType::Mod(max_modulus)]);
        let b = decoded(&common_input, &[CoreType::Mod(neighboring_modulus)]);
        assert_eq!(vector(a.values()), vector(b.values()));
        assert_ne!(a.values(), b.values());
    }

    #[test]
    fn exact_source_byte_envelope_decodes_and_one_extra_framing_byte_is_rejected() {
        let mut types = Vec::new();
        let mut input = String::with_capacity(MAX_SOURCE_BYTES);
        input.push_str("[(");
        for field in 0..13 {
            if field != 0 {
                input.push_str(", ");
            }
            let length = if field == 12 { 52_427 } else { 65_536 };
            types.push(CoreType::Array(
                ArrayType::new(&CoreType::Word64, length).unwrap(),
            ));
            input.push('[');
            for element in 0..length {
                if element != 0 {
                    input.push_str(", ");
                }
                input.push_str("0x0000000000000000");
            }
            input.push(']');
        }
        types.push(CoreType::Int);
        input.push_str(", 123456)]");
        assert_eq!(input.len(), MAX_SOURCE_BYTES);
        let mut sources = SourceMap::new();
        assert_eq!(
            sources.add("too-large.values", format!("{input}\n")),
            Err(crate::source::SourceError::TooLarge)
        );
        let expected = [CoreType::Tuple(TupleType::new(&types).unwrap())];
        with_source(input, |source| {
            let result = decode_arguments(source, &expected);
            assert_eq!(result.diagnostics(), []);
            let values = result.into_arguments().unwrap().into_values();
            assert_eq!(values.len(), 1);
            assert_eq!(values[0].ty(), expected[0]);
        });
    }

    #[test]
    fn decimal_chunks_cross_limb_boundaries_and_exact_integer_limit() {
        for spelling in [
            "1",
            "999999999",
            "1000000000",
            "1000000001",
            "4294967295",
            "4294967296",
            "18446744073709551615",
            "18446744073709551616",
            "-1000000000000000000000000001",
        ] {
            let input = format!("[{spelling}]");
            assert_eq!(vector(decoded(&input, &[CoreType::Int]).values()), input);
        }
        let maximum = crate::core::MAX_EXACT_INTEGER_BITS;
        let exact = ExactInteger::power_of_two(maximum - 1, |limbs, count| {
            limbs.try_reserve_exact(count).is_ok()
        })
        .unwrap();
        let input = format!("[{exact}]");
        let arguments = decoded(&input, &[CoreType::Int]);
        assert_eq!(arguments.values(), &[CoreValue::Int(exact)]);
        let mut carry = 0_u8;
        let mut digits: Vec<_> = arguments.values()[0]
            .to_string()
            .bytes()
            .rev()
            .map(|byte| {
                let digit = (byte - b'0') * 2 + carry;
                carry = digit / 10;
                b'0' + digit % 10
            })
            .collect();
        if carry != 0 {
            digits.push(b'0' + carry);
        }
        digits.reverse();
        let over = String::from_utf8(digits).unwrap();
        error(
            &format!("[{over}]"),
            &[CoreType::Int],
            DiagnosticCode::ArgumentDecodeResourceLimit,
        );
    }

    #[test]
    fn original_byte_spans_are_safe_for_unicode_and_eof_failures() {
        for (input, slice) in [("[1, é]", "é"), ("[1, 🟠]", "🟠"), ("[1, ", "")] {
            with_source(input, |source| {
                let result = decode_arguments(source, &[CoreType::Int, CoreType::Int]);
                assert_eq!(
                    result.diagnostics()[0].code(),
                    DiagnosticCode::NoncanonicalArgumentValue
                );
                assert_eq!(
                    source.slice(result.diagnostics()[0].primary_span()),
                    Some(slice)
                );
                assert!(
                    result.diagnostics()[0]
                        .notes()
                        .iter()
                        .any(|note| note == "argument index 1 (zero-based)")
                );
            });
        }
    }

    #[test]
    fn exact_node_limb_and_work_budgets_fail_closed() {
        let row = ArrayType::new(&CoreType::Int, 2).unwrap();
        let types = [CoreType::Array(row)];
        with_source("[[1, 4294967296]]", |source| {
            for (nodes, limbs, accepted) in [(3, 3, true), (2, 3, false), (3, 2, false)] {
                let result = decode_with_limits(
                    source,
                    &types,
                    Limits {
                        nodes,
                        limbs,
                        work: 1000,
                    },
                    Reservations::DEFAULT,
                );
                assert_eq!(!result.has_errors(), accepted);
                if !accepted {
                    assert_eq!(
                        result.diagnostics()[0].code(),
                        DiagnosticCode::ArgumentDecodeResourceLimit
                    );
                }
            }
            let mut minimum = 0;
            for work in 0..1000 {
                if !decode_with_limits(
                    source,
                    &types,
                    Limits {
                        nodes: 3,
                        limbs: 3,
                        work,
                    },
                    Reservations::DEFAULT,
                )
                .has_errors()
                {
                    minimum = work;
                    break;
                }
            }
            assert!(minimum > source.text().len());
            assert!(
                !decode_with_limits(
                    source,
                    &types,
                    Limits {
                        nodes: 3,
                        limbs: 3,
                        work: minimum
                    },
                    Reservations::DEFAULT
                )
                .has_errors()
            );
            let result = decode_with_limits(
                source,
                &types,
                Limits {
                    nodes: 3,
                    limbs: 3,
                    work: minimum - 1,
                },
                Reservations::DEFAULT,
            );
            assert!(result.has_errors());
            assert_eq!(
                result.diagnostics()[0].code(),
                DiagnosticCode::ArgumentDecodeResourceLimit
            );
        });
    }

    #[test]
    fn reservation_failures_and_excess_parameters_produce_no_partial_vector() {
        with_source("[1, 2]", |source| {
            for reservations in [
                Reservations {
                    values: |_, _| false,
                    ..Reservations::DEFAULT
                },
                Reservations {
                    limb: |_| false,
                    ..Reservations::DEFAULT
                },
            ] {
                let result = decode_with_limits(
                    source,
                    &[CoreType::Int, CoreType::Int],
                    Limits::DEFAULT,
                    reservations,
                );
                assert!(result.arguments().is_none());
                assert_eq!(
                    result.diagnostics()[0].code(),
                    DiagnosticCode::ArgumentDecodeResourceLimit
                );
            }
        });
        error(
            "[]",
            &vec![CoreType::Int; MAX_PARAMETERS_PER_FUNCTION + 1],
            DiagnosticCode::ArgumentDecodeResourceLimit,
        );
    }

    #[test]
    fn maximum_individual_tuple_matrix_shape_is_admitted_and_preserved() {
        let row = ArrayType::new(&CoreType::Int, 1).unwrap();
        let matrix = ArrayType::new(&CoreType::Array(row), crate::core::MAX_ARRAY_LENGTH).unwrap();
        let tuple = TupleType::new(&vec![CoreType::Array(matrix); 16]).unwrap();
        let mut input = String::from("[(");
        for field in 0..16 {
            if field != 0 {
                input.push_str(", ");
            }
            input.push('[');
            for element in 0..matrix.length() {
                if element != 0 {
                    input.push_str(", ");
                }
                input.push_str("[0]");
            }
            input.push(']');
        }
        input.push_str(")]");
        assert!(input.len() < MAX_SOURCE_BYTES);
        with_source(input, |source| {
            let result = decode_arguments(source, &[CoreType::Tuple(tuple.clone())]);
            assert_eq!(result.diagnostics(), []);
            let CoreValue::Tuple(value) = &result.arguments().unwrap().values()[0] else {
                panic!()
            };
            assert_eq!(value.ty(), &tuple);
            assert_eq!(value.elements().len(), 16);
            for field in value.elements() {
                let CoreValue::Array(rows) = field else {
                    panic!()
                };
                assert_eq!(rows.ty(), matrix);
                assert_eq!(rows.elements().len(), 65_536);
                for row in rows.elements() {
                    let CoreValue::Array(row) = row else { panic!() };
                    assert_eq!(
                        row.elements(),
                        &[CoreValue::Int(ExactInteger::new(false, Magnitude::zero()))]
                    );
                }
            }
        });
    }

    #[test]
    fn aggregate_node_budget_is_shared_across_arguments() {
        let row = ArrayType::new(&CoreType::Bool, 1).unwrap();
        let matrix = ArrayType::new(&CoreType::Array(row), crate::core::MAX_ARRAY_LENGTH).unwrap();
        let tuple = TupleType::new(&vec![CoreType::Array(matrix); 16]).unwrap();
        error(
            "[]",
            &[CoreType::Tuple(tuple.clone()), CoreType::Tuple(tuple)],
            DiagnosticCode::ArgumentDecodeResourceLimit,
        );
    }

    #[test]
    fn deep_hostile_inputs_and_all_aggregate_forms_fit_in_one_mebibyte_stack() {
        std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(|| {
                let hostile = format!("[{}0{}]", "[".repeat(100_000), "]".repeat(100_000));
                error(
                    &hostile,
                    &[CoreType::Int],
                    DiagnosticCode::ArgumentValueMismatch,
                );
                let row = ArrayType::new(&CoreType::Mod(modulus(11)), 2).unwrap();
                let matrix = ArrayType::new(&CoreType::Array(row), 2).unwrap();
                let tuple = TupleType::new(&[CoreType::Array(matrix), CoreType::Bool]).unwrap();
                let input = "[([[0, 10], [1, 9]], true)]";
                assert_eq!(
                    vector(decoded(input, &[CoreType::Tuple(tuple)]).values()),
                    input
                );
                error(
                    &format!("[{}]", "9".repeat(100_000)),
                    &[CoreType::Int],
                    DiagnosticCode::ArgumentDecodeResourceLimit,
                );
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
