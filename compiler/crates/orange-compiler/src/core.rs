//! Typed, source-mapped Core for the first Orange semantic fragment.

use std::fmt;

use crate::source::Span;

/// A successfully analyzed Orange module.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreModule {
    /// Full source extent of the module declaration.
    pub span: Span,
    /// Exact ASCII module name.
    pub name: String,
    /// Typed functions in deterministic source order.
    pub functions: Vec<CoreFunction>,
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
    pub id: CoreFunctionId,
    /// Full source extent of the function declaration.
    pub span: Span,
    /// Exact ASCII function name.
    pub name: String,
    /// Source extent of the function name.
    pub name_span: Span,
    /// Statically checked result type.
    pub result_type: CoreType,
    /// Statically checked literal result.
    pub value: CoreValue,
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
}
