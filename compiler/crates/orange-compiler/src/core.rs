//! Typed, source-mapped Core for the first Orange semantic fragment.

use std::fmt;

use crate::source::Span;

/// A successfully analyzed Orange module.
///
/// Core storage is read-only outside this crate so callers cannot reorder
/// functions, duplicate identities, or replace a checked value.
///
/// ```compile_fail
/// use orange_compiler::CoreModule;
///
/// fn discard_checked_functions(core: &mut CoreModule) {
///     core.functions.clear();
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreModule {
    /// Full source extent of the module declaration.
    pub(crate) span: Span,
    /// Exact ASCII module name.
    pub(crate) name: String,
    /// Typed functions in deterministic source order.
    pub(crate) functions: Vec<CoreFunction>,
}

impl CoreModule {
    /// Returns the full source extent of the module declaration.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact ASCII module name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns typed functions in deterministic source order.
    #[must_use]
    pub fn functions(&self) -> &[CoreFunction] {
        &self.functions
    }
}

/// A dense, source-ordered identity within one [`CoreModule`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CoreFunctionId(u32);

impl CoreFunctionId {
    pub(crate) fn from_index(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }

    /// Returns the zero-based source-order index.
    #[must_use]
    pub const fn index(self) -> u32 {
        self.0
    }
}

/// One typed, zero-argument specification function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreFunction {
    /// Dense source-order identity.
    pub(crate) id: CoreFunctionId,
    /// Full source extent of the function declaration.
    pub(crate) span: Span,
    /// Exact ASCII function name.
    pub(crate) name: String,
    /// Source extent of the function name.
    pub(crate) name_span: Span,
    /// Statically checked literal result.
    pub(crate) value: CoreValue,
}

impl CoreFunction {
    /// Returns the dense source-order identity.
    #[must_use]
    pub const fn id(&self) -> CoreFunctionId {
        self.id
    }

    /// Returns the full source extent of the function declaration.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the exact ASCII function name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the source extent of the function name.
    #[must_use]
    pub const fn name_span(&self) -> Span {
        self.name_span
    }

    /// Returns the statically checked result type.
    #[must_use]
    pub const fn result_type(&self) -> CoreType {
        self.value.ty()
    }

    /// Returns the statically checked literal result.
    #[must_use]
    pub const fn value(&self) -> &CoreValue {
        &self.value
    }
}

/// Types admitted by the first typed expression fragment.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CoreType {
    /// An exact, signed mathematical integer.
    Int,
    /// An unsigned 8-bit word.
    Word8,
}

impl fmt::Display for CoreType {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Int => "Int",
            Self::Word8 => "Word[8]",
        })
    }
}

/// Values admitted by the first typed expression fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreValue {
    /// An exact mathematical integer.
    Int(ExactInteger),
    /// An unsigned 8-bit word.
    Word8(u8),
}

impl CoreValue {
    /// Returns this value's static Core type.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        match self {
            Self::Int(_) => CoreType::Int,
            Self::Word8(_) => CoreType::Word8,
        }
    }
}

impl fmt::Display for CoreValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => value.fmt(formatter),
            Self::Word8(value) => write!(formatter, "0x{value:02x}"),
        }
    }
}

/// A signed integer with an arbitrary-precision, dependency-free magnitude.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactInteger {
    negative: bool,
    magnitude: Magnitude,
}

impl ExactInteger {
    pub(crate) fn new(negative: bool, magnitude: Magnitude) -> Self {
        Self {
            negative: negative && !magnitude.is_zero(),
            magnitude,
        }
    }

    /// Returns whether this is strictly less than zero.
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.negative
    }

    /// Returns whether this is zero.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.magnitude.is_zero()
    }

    /// Returns the number of significant magnitude bits; zero has zero bits.
    #[must_use]
    pub fn magnitude_bits(&self) -> usize {
        self.magnitude.bit_len()
    }
}

impl fmt::Display for ExactInteger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.negative {
            formatter.write_str("-")?;
        }
        formatter.write_str(&self.magnitude.to_decimal())
    }
}

/// An unsigned arbitrary-precision integer stored in little-endian binary limbs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Magnitude {
    limbs: Vec<u32>,
}

impl Magnitude {
    pub(crate) const fn zero() -> Self {
        Self { limbs: Vec::new() }
    }

    pub(crate) fn multiply_add(&mut self, multiplier: u32, addend: u32) {
        let mut carry = u64::from(addend);
        for limb in &mut self.limbs {
            let value = u64::from(*limb) * u64::from(multiplier) + carry;
            *limb = value as u32;
            carry = value >> u32::BITS;
        }
        if carry != 0 {
            self.limbs.push(carry as u32);
        }
        self.normalize();
    }

    pub(crate) fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    pub(crate) fn bit_len(&self) -> usize {
        self.limbs.last().map_or(0, |most_significant| {
            (self.limbs.len() - 1) * u32::BITS as usize
                + (u32::BITS - most_significant.leading_zeros()) as usize
        })
    }

    pub(crate) fn to_u8(&self) -> Option<u8> {
        match self.limbs.as_slice() {
            [] => Some(0),
            [value] => u8::try_from(*value).ok(),
            _ => None,
        }
    }

    fn normalize(&mut self) {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
    }

    fn to_decimal(&self) -> String {
        const DECIMAL_LIMB: u64 = 1_000_000_000;

        if self.is_zero() {
            return String::from("0");
        }
        let mut binary = self.limbs.clone();
        let mut decimal = Vec::new();
        while !binary.is_empty() {
            let mut remainder = 0_u64;
            for limb in binary.iter_mut().rev() {
                let dividend = (remainder << u32::BITS) | u64::from(*limb);
                *limb = u32::try_from(dividend / DECIMAL_LIMB)
                    .expect("division quotient fits in one binary limb");
                remainder = dividend % DECIMAL_LIMB;
            }
            while binary.last() == Some(&0) {
                binary.pop();
            }
            decimal.push(u32::try_from(remainder).expect("decimal limb fits in u32"));
        }

        let mut output = decimal.pop().unwrap_or(0).to_string();
        for limb in decimal.iter().rev() {
            use fmt::Write as _;
            let _ = write!(output, "{limb:09}");
        }
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{SourceMap, TextOffset};

    fn test_span() -> Span {
        let mut sources = SourceMap::new();
        let id = sources.add("core.or", "x").unwrap();
        sources
            .get(id)
            .unwrap()
            .span(TextOffset::new(0), TextOffset::new(1))
            .unwrap()
    }

    #[test]
    fn exact_integer_decimal_formatting_does_not_collapse_large_values() {
        let mut magnitude = Magnitude::zero();
        for digit in [1, 2, 3, 4, 5, 6, 7, 8, 9, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9] {
            magnitude.multiply_add(10, digit);
        }
        let value = ExactInteger::new(true, magnitude);
        assert_eq!(value.to_string(), "-1234567890123456789");
    }

    #[test]
    fn negative_zero_is_canonical_zero() {
        let value = ExactInteger::new(true, Magnitude::zero());
        assert!(!value.is_negative());
        assert!(value.is_zero());
        assert_eq!(value.to_string(), "0");
    }

    #[test]
    fn word_display_is_fixed_width_lowercase_hexadecimal() {
        assert_eq!(CoreValue::Word8(0).to_string(), "0x00");
        assert_eq!(CoreValue::Word8(10).to_string(), "0x0a");
        assert_eq!(CoreValue::Word8(255).to_string(), "0xff");
    }

    #[test]
    fn core_accessors_preserve_source_order_and_derive_value_types() {
        let span = test_span();
        let functions = vec![
            CoreFunction {
                id: CoreFunctionId::from_index(0).unwrap(),
                span,
                name: String::from("integer"),
                name_span: span,
                value: CoreValue::Int(ExactInteger::new(false, Magnitude::zero())),
            },
            CoreFunction {
                id: CoreFunctionId::from_index(1).unwrap(),
                span,
                name: String::from("word"),
                name_span: span,
                value: CoreValue::Word8(8),
            },
        ];
        let module = CoreModule {
            span,
            name: String::from("values"),
            functions,
        };

        assert_eq!(module.span(), span);
        assert_eq!(module.name(), "values");
        assert_eq!(module.functions()[0].id().index(), 0);
        assert_eq!(module.functions()[0].span(), span);
        assert_eq!(module.functions()[0].name(), "integer");
        assert_eq!(module.functions()[0].name_span(), span);
        assert_eq!(module.functions()[0].result_type(), CoreType::Int);
        assert_eq!(module.functions()[0].value().ty(), CoreType::Int);
        assert_eq!(module.functions()[1].id().index(), 1);
        assert_eq!(module.functions()[1].result_type(), CoreType::Word8);
        assert_eq!(module.functions()[1].value(), &CoreValue::Word8(8));
    }
}
