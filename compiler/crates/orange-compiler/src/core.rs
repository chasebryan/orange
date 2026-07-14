//! Typed, source-mapped Core for the first Orange semantic fragment.

use std::fmt;

use crate::source::Span;

pub(crate) const MAX_EXACT_INTEGER_BITS: usize = 16_384;
const BINARY_LIMB_BITS: usize = 32;
const MAX_BINARY_LIMBS: usize = MAX_EXACT_INTEGER_BITS.div_ceil(BINARY_LIMB_BITS);
// One base-1,000,000,000 limb carries more than 27 binary bits. Using the
// weaker 27-bit bound keeps this capacity an integer-only, auditable upper
// bound rather than relying on floating-point logarithms.
const MAX_DECIMAL_LIMBS: usize = MAX_EXACT_INTEGER_BITS.div_ceil(27);

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

macro_rules! define_core_types {
    ($($(#[$variant_doc:meta])* $variant:ident => $name:literal,)+) => {
        /// Types admitted by the first typed expression fragment.
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub enum CoreType {
            $($(#[$variant_doc])* $variant,)+
        }

        impl CoreType {
            /// Returns the stable printable Core type name.
            #[must_use]
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name,)+
                }
            }

            #[cfg(test)]
            const ALL: &'static [Self] = &[$(Self::$variant,)+];
        }

        impl fmt::Display for CoreType {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }
    }
}

define_core_types! {
    /// An exact, signed mathematical integer.
    Int => "Int",
    /// An unsigned 8-bit word.
    Word8 => "Word[8]",
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

    pub(crate) fn try_clone_with_reservation(
        &self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        match self {
            Self::Int(value) => value
                .try_clone_with_reservation(reserve_limbs)
                .map(Self::Int),
            Self::Word8(value) => Some(Self::Word8(*value)),
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
    pub(crate) const fn new(negative: bool, magnitude: Magnitude) -> Self {
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
    pub const fn is_zero(&self) -> bool {
        self.magnitude.is_zero()
    }

    /// Returns the number of significant magnitude bits; zero has zero bits.
    #[must_use]
    pub fn magnitude_bits(&self) -> usize {
        self.magnitude.bit_len()
    }

    fn try_clone_with_reservation(
        &self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        Some(Self {
            negative: self.negative,
            magnitude: self.magnitude.try_clone_with_reservation(reserve_limbs)?,
        })
    }
}

impl fmt::Display for ExactInteger {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let decimal = self.magnitude.decimal_limbs().ok_or(fmt::Error)?;
        if self.negative {
            formatter.write_str("-")?;
        }
        decimal.write(formatter)
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

    pub(crate) fn multiply_add_with_reservation(
        &mut self,
        multiplier: u32,
        addend: u32,
        reserve_limb: impl FnOnce(&mut Vec<u32>) -> bool,
    ) -> bool {
        // Determine whether the result needs another limb before modifying the
        // existing representation. A failed growth reservation must leave a
        // reusable magnitude unchanged rather than containing a partial
        // multiply-add result.
        let mut carry = u64::from(addend);
        for limb in &self.limbs {
            let Some(value) = u64::from(*limb)
                .checked_mul(u64::from(multiplier))
                .and_then(|product| product.checked_add(carry))
            else {
                // The u32 operand domain proves this unreachable. Returning
                // failure still keeps a compiler artifact from being exposed
                // if that representation invariant ever changes.
                return false;
            };
            carry = value >> u32::BITS;
        }
        if carry != 0 && !reserve_limb(&mut self.limbs) {
            return false;
        }

        carry = u64::from(addend);
        for limb in &mut self.limbs {
            let Some(value) = u64::from(*limb)
                .checked_mul(u64::from(multiplier))
                .and_then(|product| product.checked_add(carry))
            else {
                return false;
            };
            // A binary limb is the low half of this exact two-limb value.
            let Ok(low_limb) = u32::try_from(value & u64::from(u32::MAX)) else {
                return false;
            };
            *limb = low_limb;
            carry = value >> u32::BITS;
        }
        if carry != 0 {
            // The maximum product plus carry is
            // u32::MAX * (u32::MAX + 1), so its high half fits one limb.
            let Ok(carry_limb) = u32::try_from(carry) else {
                return false;
            };
            self.limbs.push(carry_limb);
        }
        self.normalize();
        true
    }

    pub(crate) const fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }

    fn try_clone_with_reservation(
        &self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.limbs.is_empty() {
            return Some(Self::zero());
        }
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, self.limbs.len()) {
            return None;
        }
        limbs.extend_from_slice(&self.limbs);
        Some(Self { limbs })
    }

    pub(crate) fn bit_len(&self) -> usize {
        self.limbs.last().map_or(0, |most_significant| {
            self.limbs
                .len()
                .checked_sub(1)
                .and_then(|full_limbs| full_limbs.checked_mul(BINARY_LIMB_BITS))
                .and_then(|full_bits| {
                    u32::BITS
                        .checked_sub(most_significant.leading_zeros())
                        .and_then(|remaining_bits| usize::try_from(remaining_bits).ok())
                        .and_then(|remaining_bits| full_bits.checked_add(remaining_bits))
                })
                .unwrap_or(usize::MAX)
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

    fn decimal_limbs(&self) -> Option<DecimalLimbs> {
        const DECIMAL_LIMB: u64 = 1_000_000_000;

        if self.is_zero() {
            return Some(DecimalLimbs::zero());
        }
        let mut binary = [0_u32; MAX_BINARY_LIMBS];
        binary
            .get_mut(..self.limbs.len())?
            .copy_from_slice(&self.limbs);
        let mut binary_len = self.limbs.len();
        let mut decimal = DecimalLimbs::zero();
        while binary_len != 0 {
            let mut remainder = 0_u64;
            for limb in binary.get_mut(..binary_len)?.iter_mut().rev() {
                let dividend = (remainder << u32::BITS) | u64::from(*limb);
                *limb = u32::try_from(dividend / DECIMAL_LIMB).ok()?;
                remainder = dividend % DECIMAL_LIMB;
            }
            while let Some(last_index) = binary_len.checked_sub(1) {
                if binary.get(last_index).copied() != Some(0) {
                    break;
                }
                binary_len = last_index;
            }
            decimal.push(u32::try_from(remainder).ok()?)?;
        }

        Some(decimal)
    }
}

struct DecimalLimbs {
    limbs: [u32; MAX_DECIMAL_LIMBS],
    len: usize,
}

impl DecimalLimbs {
    const fn zero() -> Self {
        Self {
            limbs: [0; MAX_DECIMAL_LIMBS],
            len: 0,
        }
    }

    fn push(&mut self, limb: u32) -> Option<()> {
        let next_len = self.len.checked_add(1)?;
        *self.limbs.get_mut(self.len)? = limb;
        self.len = next_len;
        Some(())
    }

    fn write(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.len == 0 {
            return formatter.write_str("0");
        }
        let (most_significant, remaining) = self
            .limbs
            .get(..self.len)
            .and_then(<[u32]>::split_last)
            .ok_or(fmt::Error)?;
        write!(formatter, "{most_significant}")?;
        for limb in remaining.iter().rev() {
            write!(formatter, "{limb:09}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{SourceMap, TextOffset};

    #[test]
    fn build_profiles_preserve_debug_assertions_and_overflow_checks() {
        const { assert!(cfg!(debug_assertions)) };
        let overflow = std::panic::catch_unwind(|| {
            std::hint::black_box(u32::MAX) + std::hint::black_box(1_u32)
        });
        assert!(overflow.is_err());
    }

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
            assert!(magnitude.multiply_add_with_reservation(10, digit, |limbs| {
                limbs.try_reserve(1).is_ok()
            }));
        }
        let value = ExactInteger::new(true, magnitude);
        assert_eq!(value.to_string(), "-1234567890123456789");
    }

    #[test]
    fn magnitude_multiply_add_accepts_the_full_u32_domain_without_overflow() {
        let mut magnitude = Magnitude::zero();
        let reserve = |limbs: &mut Vec<u32>| limbs.try_reserve(1).is_ok();

        assert!(magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, reserve));
        assert_eq!(magnitude.limbs, [u32::MAX]);

        assert!(magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, reserve));
        assert_eq!(magnitude.limbs, [0, u32::MAX]);

        assert!(magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, reserve));
        assert_eq!(magnitude.limbs, [u32::MAX, 1, u32::MAX - 1]);
    }

    #[test]
    fn magnitude_limb_reservation_failure_preserves_existing_state() {
        let mut magnitude = Magnitude {
            limbs: vec![u32::MAX],
        };
        let before = magnitude.clone();

        assert!(!magnitude.multiply_add_with_reservation(u32::MAX, u32::MAX, |_| false));
        assert_eq!(magnitude, before);
    }

    #[test]
    fn maximum_exact_integer_uses_the_fixed_decimal_scratch_bound() {
        let mut magnitude = Magnitude::zero();
        for _ in 0..MAX_EXACT_INTEGER_BITS {
            assert!(
                magnitude
                    .multiply_add_with_reservation(2, 1, |limbs| { limbs.try_reserve(1).is_ok() })
            );
        }

        let decimal = magnitude.decimal_limbs().unwrap();
        assert!(decimal.len <= MAX_DECIMAL_LIMBS);
        assert_eq!(magnitude.bit_len(), MAX_EXACT_INTEGER_BITS);
    }

    #[test]
    fn zero_decimal_formatting_uses_no_decimal_limbs() {
        let magnitude = Magnitude::zero();

        assert_eq!(magnitude.decimal_limbs().unwrap().len, 0);
    }

    #[test]
    fn oversized_internal_magnitude_returns_a_formatting_error_without_output() {
        let value = ExactInteger::new(
            true,
            Magnitude {
                limbs: vec![1; MAX_BINARY_LIMBS + 1],
            },
        );
        let mut output = String::new();

        let result = std::fmt::write(&mut output, format_args!("{value}"));

        assert_eq!(result, Err(fmt::Error));
        assert!(output.is_empty());
    }

    #[test]
    fn decimal_limb_capacity_failure_preserves_existing_state() {
        let mut decimal = DecimalLimbs::zero();
        for limb in 0..MAX_DECIMAL_LIMBS {
            assert_eq!(decimal.push(u32::try_from(limb).unwrap()), Some(()));
        }
        let before = decimal.limbs;

        assert_eq!(decimal.push(u32::MAX), None);
        assert_eq!(decimal.len, MAX_DECIMAL_LIMBS);
        assert_eq!(decimal.limbs, before);
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
    fn core_type_inventory_and_display_are_exact() {
        assert_eq!(CoreType::ALL, &[CoreType::Int, CoreType::Word8]);
        assert_eq!(
            CoreType::ALL
                .iter()
                .map(|result_type| result_type.as_str())
                .collect::<Vec<_>>(),
            ["Int", "Word[8]"]
        );
        assert!(
            CoreType::ALL
                .iter()
                .all(|result_type| result_type.to_string() == result_type.as_str())
        );
    }

    #[test]
    fn core_function_id_accepts_the_full_u32_domain() {
        assert_eq!(CoreFunctionId::from_index(0).unwrap().index(), 0);
        let maximum = usize::try_from(u32::MAX).unwrap();
        assert_eq!(
            CoreFunctionId::from_index(maximum).unwrap().index(),
            u32::MAX
        );

        #[cfg(target_pointer_width = "64")]
        assert!(CoreFunctionId::from_index(maximum.checked_add(1).unwrap()).is_none());
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

        let CoreModule {
            span: _,
            name: _,
            functions,
        } = module;
        for function in functions {
            let CoreFunction {
                id: _,
                span: _,
                name: _,
                name_span: _,
                value,
            } = function;
            match value {
                CoreValue::Int(_) | CoreValue::Word8(_) => {}
            }
        }
        for result_type in [CoreType::Int, CoreType::Word8] {
            match result_type {
                CoreType::Int | CoreType::Word8 => {}
            }
        }
    }
}
