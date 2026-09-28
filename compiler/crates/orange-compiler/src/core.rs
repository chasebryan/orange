//! Typed, source-mapped Core for the Orange 2026 semantic fragment.

use std::cmp::Ordering;
use std::fmt;

use crate::parser::{BinaryOperator, UnaryOperator};
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

/// One typed specification function.
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
    /// Parameter types in declaration order.
    pub(crate) parameters: Vec<CoreType>,
    /// Statically checked result type.
    pub(crate) result_type: CoreType,
    /// Statically checked body.
    pub(crate) body: CoreExpression,
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

    /// Returns parameter types in declaration order.
    #[must_use]
    pub fn parameters(&self) -> &[CoreType] {
        &self.parameters
    }

    /// Returns the statically checked result type.
    #[must_use]
    pub const fn result_type(&self) -> CoreType {
        self.result_type
    }

    /// Returns the statically checked body.
    #[must_use]
    pub const fn body(&self) -> &CoreExpression {
        &self.body
    }
}

/// A typed expression tree stored in postorder.
///
/// Every node follows the nodes of its operands, so the final node is the
/// root and evaluation needs only a value stack. The representation is
/// private to this compiler and has no canonical encoding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreExpression {
    /// Nodes in postorder; the last node is the root.
    pub(crate) nodes: Vec<CoreNode>,
}

impl CoreExpression {
    /// Returns nodes in postorder; the last node is the root.
    #[must_use]
    pub fn nodes(&self) -> &[CoreNode] {
        &self.nodes
    }

    /// Returns the root node.
    #[must_use]
    pub fn root(&self) -> Option<&CoreNode> {
        self.nodes.last()
    }

    /// Returns the value of an expression that is exactly one literal.
    #[must_use]
    pub fn literal(&self) -> Option<&CoreValue> {
        match self.nodes.as_slice() {
            [
                CoreNode {
                    kind: CoreNodeKind::Literal(value),
                    ..
                },
            ] => Some(value),
            _ => None,
        }
    }
}

/// One typed Core expression node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreNode {
    /// Source extent of the expression this node represents.
    pub(crate) span: Span,
    /// Statically checked type of the node's value.
    pub(crate) ty: CoreType,
    /// Node operation.
    pub(crate) kind: CoreNodeKind,
}

impl CoreNode {
    /// Returns the source extent of the expression this node represents.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the statically checked type of the node's value.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        self.ty
    }

    /// Returns the node operation.
    #[must_use]
    pub const fn kind(&self) -> &CoreNodeKind {
        &self.kind
    }
}

/// The operation of one Core node.
///
/// Operands are the values of the immediately preceding complete subtrees,
/// in source order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreNodeKind {
    /// A checked literal value.
    Literal(CoreValue),
    /// The value of the parameter at this zero-based index.
    Parameter(u32),
    /// A call of a function with this many argument subtrees.
    Call {
        /// The called function.
        function: CoreFunctionId,
        /// The number of argument subtrees.
        arguments: u32,
    },
    /// A prefix operator applied to one operand subtree.
    Unary(UnaryOperator),
    /// An arithmetic or bitwise operator applied to two operand subtrees.
    Binary(BinaryOperator),
    /// A shift or rotation of one operand subtree by a literal amount.
    Shift {
        /// The shift or rotation operator.
        operator: BinaryOperator,
        /// The amount, less than the operand's word width.
        amount: u32,
    },
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
    /// An element of the integers modulo 2^8.
    Word8 => "Word[8]",
    /// An element of the integers modulo 2^16.
    Word16 => "Word[16]",
    /// An element of the integers modulo 2^32.
    Word32 => "Word[32]",
    /// An element of the integers modulo 2^64.
    Word64 => "Word[64]",
}

impl CoreType {
    /// Returns the width of a word type, or `None` for `Int`.
    #[must_use]
    pub const fn word_bits(self) -> Option<u32> {
        match self {
            Self::Int => None,
            Self::Word8 => Some(8),
            Self::Word16 => Some(16),
            Self::Word32 => Some(32),
            Self::Word64 => Some(64),
        }
    }

    /// Returns the word type of exactly this width, if it is admitted.
    #[must_use]
    pub const fn word_of_width(bits: u32) -> Option<Self> {
        match bits {
            8 => Some(Self::Word8),
            16 => Some(Self::Word16),
            32 => Some(Self::Word32),
            64 => Some(Self::Word64),
            _ => None,
        }
    }
}

/// Values admitted by the typed expression fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreValue {
    /// An exact mathematical integer.
    Int(ExactInteger),
    /// An element of the integers modulo 2^8.
    Word8(u8),
    /// An element of the integers modulo 2^16.
    Word16(u16),
    /// An element of the integers modulo 2^32.
    Word32(u32),
    /// An element of the integers modulo 2^64.
    Word64(u64),
}

impl CoreValue {
    /// Returns this value's static Core type.
    #[must_use]
    pub const fn ty(&self) -> CoreType {
        match self {
            Self::Int(_) => CoreType::Int,
            Self::Word8(_) => CoreType::Word8,
            Self::Word16(_) => CoreType::Word16,
            Self::Word32(_) => CoreType::Word32,
            Self::Word64(_) => CoreType::Word64,
        }
    }

    /// Returns the word of type `ty` whose value is `value` reduced modulo
    /// its width, or `None` when `ty` is `Int`.
    pub(crate) fn word_from_u64(ty: CoreType, value: u64) -> Option<Self> {
        let [b0, b1, b2, b3, b4, b5, b6, b7] = value.to_le_bytes();
        match ty {
            CoreType::Int => None,
            CoreType::Word8 => Some(Self::Word8(b0)),
            CoreType::Word16 => Some(Self::Word16(u16::from_le_bytes([b0, b1]))),
            CoreType::Word32 => Some(Self::Word32(u32::from_le_bytes([b0, b1, b2, b3]))),
            CoreType::Word64 => Some(Self::Word64(u64::from_le_bytes([
                b0, b1, b2, b3, b4, b5, b6, b7,
            ]))),
        }
    }

    /// Returns a word's value as an unsigned integer, or `None` for `Int`.
    pub(crate) fn word_as_u64(&self) -> Option<u64> {
        match self {
            Self::Int(_) => None,
            Self::Word8(value) => Some(u64::from(*value)),
            Self::Word16(value) => Some(u64::from(*value)),
            Self::Word32(value) => Some(u64::from(*value)),
            Self::Word64(value) => Some(*value),
        }
    }
}

impl fmt::Display for CoreValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => value.fmt(formatter),
            Self::Word8(value) => write!(formatter, "0x{value:02x}"),
            Self::Word16(value) => write!(formatter, "0x{value:04x}"),
            Self::Word32(value) => write!(formatter, "0x{value:08x}"),
            Self::Word64(value) => write!(formatter, "0x{value:016x}"),
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

    /// Returns the number of base-2^32 digits in the magnitude; zero has none.
    #[must_use]
    pub fn magnitude_digits(&self) -> usize {
        self.magnitude.limbs.len()
    }

    pub(crate) fn try_clone_with_reservation(
        &self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        Some(Self {
            negative: self.negative,
            magnitude: self.magnitude.try_clone_with_reservation(reserve_limbs)?,
        })
    }

    /// Returns the exact negation.
    pub(crate) fn negated(self) -> Self {
        Self::new(!self.negative, self.magnitude)
    }

    /// Returns the exact sum, or `None` if storage cannot be reserved.
    pub(crate) fn add(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.negative == other.negative {
            let magnitude = self.magnitude.add(&other.magnitude, reserve_limbs)?;
            return Some(Self::new(self.negative, magnitude));
        }
        // Opposite signs: subtract the smaller magnitude from the larger and
        // keep the sign of the operand with the larger magnitude.
        match self.magnitude.compare(&other.magnitude) {
            Ordering::Less => {
                let magnitude = other.magnitude.subtract(&self.magnitude, reserve_limbs)?;
                Some(Self::new(other.negative, magnitude))
            }
            Ordering::Equal | Ordering::Greater => {
                let magnitude = self.magnitude.subtract(&other.magnitude, reserve_limbs)?;
                Some(Self::new(self.negative, magnitude))
            }
        }
    }

    /// Returns the exact difference, or `None` if storage cannot be reserved.
    pub(crate) fn subtract(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        let negated_other = Self {
            negative: !other.negative && !other.magnitude.is_zero(),
            magnitude: other.magnitude.try_clone_with_reservation(reserve_limbs)?,
        };
        self.add(&negated_other, reserve_limbs)
    }

    /// Returns the exact product, or `None` if storage cannot be reserved.
    pub(crate) fn multiply(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        let magnitude = self.magnitude.multiply(&other.magnitude, reserve_limbs)?;
        Some(Self::new(self.negative != other.negative, magnitude))
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

    fn compare(&self, other: &Self) -> Ordering {
        self.limbs
            .len()
            .cmp(&other.limbs.len())
            .then_with(|| self.limbs.iter().rev().cmp(other.limbs.iter().rev()))
    }

    fn add(&self, other: &Self, reserve_limbs: fn(&mut Vec<u32>, usize) -> bool) -> Option<Self> {
        let (longer, shorter) = if self.limbs.len() >= other.limbs.len() {
            (&self.limbs, &other.limbs)
        } else {
            (&other.limbs, &self.limbs)
        };
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, longer.len().checked_add(1)?) {
            return None;
        }
        let mut carry = 0_u64;
        let mut shorter_limbs = shorter.iter();
        for limb in longer {
            let addend = shorter_limbs.next().map_or(0, |value| u64::from(*value));
            // Two u32 limbs and a one-bit carry cannot overflow u64.
            let sum = u64::from(*limb).checked_add(addend)?.checked_add(carry)?;
            limbs.push(low_limb(sum)?);
            carry = sum >> u32::BITS;
        }
        if carry != 0 {
            limbs.push(low_limb(carry)?);
        }
        let mut sum = Self { limbs };
        sum.normalize();
        Some(sum)
    }

    /// Returns `self - other`; `self` must not be smaller than `other`.
    fn subtract(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.compare(other) == Ordering::Less {
            return None;
        }
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, self.limbs.len()) {
            return None;
        }
        let mut borrow = false;
        let mut other_limbs = other.limbs.iter();
        for limb in &self.limbs {
            let subtrahend = other_limbs.next().copied().unwrap_or(0);
            let (partial, first_borrow) = limb.overflowing_sub(subtrahend);
            let (difference, second_borrow) = partial.overflowing_sub(u32::from(borrow));
            limbs.push(difference);
            borrow = first_borrow || second_borrow;
        }
        if borrow {
            return None;
        }
        let mut difference = Self { limbs };
        difference.normalize();
        Some(difference)
    }

    fn multiply(
        &self,
        other: &Self,
        reserve_limbs: fn(&mut Vec<u32>, usize) -> bool,
    ) -> Option<Self> {
        if self.is_zero() || other.is_zero() {
            return Some(Self::zero());
        }
        let length = self.limbs.len().checked_add(other.limbs.len())?;
        let mut limbs = Vec::new();
        if !reserve_limbs(&mut limbs, length) {
            return None;
        }
        limbs.resize(length, 0);
        for (row, left) in self.limbs.iter().enumerate() {
            let mut carry = 0_u64;
            let mut slots = limbs.get_mut(row..)?.iter_mut();
            for right in &other.limbs {
                let slot = slots.next()?;
                // (2^32 - 1)^2 + 2 * (2^32 - 1) = 2^64 - 1, so the
                // accumulated product, slot, and carry fit in u64.
                let value = u64::from(*left)
                    .checked_mul(u64::from(*right))?
                    .checked_add(u64::from(*slot))?
                    .checked_add(carry)?;
                *slot = low_limb(value)?;
                carry = value >> u32::BITS;
            }
            // The slot after this row has not been written by earlier rows.
            *slots.next()? = low_limb(carry)?;
        }
        let mut product = Self { limbs };
        product.normalize();
        Some(product)
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

    pub(crate) fn to_u64(&self) -> Option<u64> {
        match self.limbs.as_slice() {
            [] => Some(0),
            [low] => Some(u64::from(*low)),
            [low, high] => Some((u64::from(*high) << u32::BITS) | u64::from(*low)),
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

fn low_limb(value: u64) -> Option<u32> {
    u32::try_from(value & u64::from(u32::MAX)).ok()
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
        assert_eq!(CoreValue::Word16(0).to_string(), "0x0000");
        assert_eq!(CoreValue::Word16(0xbeef).to_string(), "0xbeef");
        assert_eq!(CoreValue::Word32(1).to_string(), "0x00000001");
        assert_eq!(CoreValue::Word32(0x6a09_e667).to_string(), "0x6a09e667");
        assert_eq!(CoreValue::Word64(0).to_string(), "0x0000000000000000");
        assert_eq!(
            CoreValue::Word64(u64::MAX).to_string(),
            "0xffffffffffffffff"
        );
    }

    #[test]
    fn core_type_inventory_and_display_are_exact() {
        assert_eq!(
            CoreType::ALL,
            &[
                CoreType::Int,
                CoreType::Word8,
                CoreType::Word16,
                CoreType::Word32,
                CoreType::Word64,
            ]
        );
        assert_eq!(
            CoreType::ALL
                .iter()
                .map(|result_type| result_type.as_str())
                .collect::<Vec<_>>(),
            ["Int", "Word[8]", "Word[16]", "Word[32]", "Word[64]"]
        );
        for ty in CoreType::ALL {
            assert_eq!(
                ty.word_bits().and_then(CoreType::word_of_width),
                (*ty != CoreType::Int).then_some(*ty)
            );
        }
        assert_eq!(CoreType::word_of_width(12), None);
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
                parameters: Vec::new(),
                result_type: CoreType::Int,
                body: CoreExpression {
                    nodes: vec![CoreNode {
                        span,
                        ty: CoreType::Int,
                        kind: CoreNodeKind::Literal(CoreValue::Int(ExactInteger::new(
                            false,
                            Magnitude::zero(),
                        ))),
                    }],
                },
            },
            CoreFunction {
                id: CoreFunctionId::from_index(1).unwrap(),
                span,
                name: String::from("word"),
                name_span: span,
                parameters: vec![CoreType::Word32],
                result_type: CoreType::Word8,
                body: CoreExpression {
                    nodes: vec![CoreNode {
                        span,
                        ty: CoreType::Word8,
                        kind: CoreNodeKind::Literal(CoreValue::Word8(8)),
                    }],
                },
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
        assert_eq!(module.functions()[0].parameters(), []);
        assert_eq!(
            module.functions()[0].body().literal().map(CoreValue::ty),
            Some(CoreType::Int)
        );
        assert_eq!(module.functions()[0].body().nodes().len(), 1);
        assert_eq!(module.functions()[0].body().root().unwrap().span(), span);
        assert_eq!(
            module.functions()[0].body().root().unwrap().ty(),
            CoreType::Int
        );
        assert_eq!(module.functions()[1].id().index(), 1);
        assert_eq!(module.functions()[1].result_type(), CoreType::Word8);
        assert_eq!(module.functions()[1].parameters(), [CoreType::Word32]);
        assert_eq!(
            module.functions()[1].body().literal(),
            Some(&CoreValue::Word8(8))
        );
        assert_eq!(
            module.functions()[1].body().root().unwrap().kind(),
            &CoreNodeKind::Literal(CoreValue::Word8(8))
        );

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
                parameters: _,
                result_type: _,
                body,
            } = function;
            for node in body.nodes {
                let CoreNode { span: _, ty, kind } = node;
                match kind {
                    CoreNodeKind::Literal(
                        CoreValue::Int(_)
                        | CoreValue::Word8(_)
                        | CoreValue::Word16(_)
                        | CoreValue::Word32(_)
                        | CoreValue::Word64(_),
                    )
                    | CoreNodeKind::Parameter(_)
                    | CoreNodeKind::Call { .. }
                    | CoreNodeKind::Unary(_)
                    | CoreNodeKind::Binary(_)
                    | CoreNodeKind::Shift { .. } => {}
                }
                match ty {
                    CoreType::Int
                    | CoreType::Word8
                    | CoreType::Word16
                    | CoreType::Word32
                    | CoreType::Word64 => {}
                }
            }
        }
    }

    fn reserve(limbs: &mut Vec<u32>, count: usize) -> bool {
        limbs.try_reserve_exact(count).is_ok()
    }

    fn exact(value: i128) -> ExactInteger {
        let mut magnitude = Magnitude::zero();
        let unsigned = value.unsigned_abs();
        for shift in (0..8).rev() {
            let digit = u32::try_from((unsigned >> (16 * shift)) & 0xffff).unwrap();
            assert!(
                magnitude.multiply_add_with_reservation(1 << 16, digit, |limbs| {
                    limbs.try_reserve(1).is_ok()
                })
            );
        }
        ExactInteger::new(value < 0, magnitude)
    }

    fn exact_corpus() -> Vec<i128> {
        let mut values = vec![
            0,
            1,
            2,
            i128::from(u32::MAX) - 1,
            i128::from(u32::MAX),
            1 << 32,
            (1 << 32) + 1,
            i128::from(i64::MAX),
            (1 << 63) + 12_345,
        ];
        let mut state = 0x2545_f491_4f6c_dd1d_u64;
        for _ in 0..12 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            values.push(i128::from(state >> (state % 61)));
        }
        let negatives = values.iter().map(|value| -value).collect::<Vec<_>>();
        values.extend(negatives);
        values
    }

    #[test]
    fn exact_integer_arithmetic_matches_an_i128_reference() {
        let corpus = exact_corpus();
        for &left in &corpus {
            assert_eq!(exact(left).to_string(), left.to_string());
            assert_eq!(exact(left).negated().to_string(), (-left).to_string());
            assert_eq!(exact(left).is_negative(), left < 0);
            for &right in &corpus {
                let (a, b) = (exact(left), exact(right));
                assert_eq!(
                    a.add(&b, reserve).unwrap().to_string(),
                    (left + right).to_string(),
                    "{left} + {right}"
                );
                assert_eq!(
                    a.subtract(&b, reserve).unwrap().to_string(),
                    (left - right).to_string(),
                    "{left} - {right}"
                );
                assert_eq!(
                    a.multiply(&b, reserve).unwrap().to_string(),
                    (left * right).to_string(),
                    "{left} * {right}"
                );
                // Every zero result is the canonical, unsigned zero.
                let difference = a.subtract(&a, reserve).unwrap();
                assert!(difference.is_zero() && !difference.is_negative());
                assert_eq!(difference, exact(0));
            }
        }
    }

    #[test]
    fn multi_limb_arithmetic_is_exact() {
        let power = |bits: usize| {
            let mut value = exact(1);
            for _ in 0..bits {
                value = value.add(&value, reserve).unwrap();
            }
            value
        };
        for bits in [31, 32, 33, 63, 64, 65, 127, 128, 500, 1024] {
            let two_k = power(bits);
            let below = two_k.subtract(&exact(1), reserve).unwrap();
            let above = two_k.add(&exact(1), reserve).unwrap();
            // (2^k - 1)(2^k + 1) = 2^(2k) - 1, whose magnitude has 2k bits.
            let product = below.multiply(&above, reserve).unwrap();
            assert_eq!(product.magnitude_bits(), 2 * bits);
            assert_eq!(
                product.add(&exact(1), reserve).unwrap(),
                power(2 * bits),
                "{bits}"
            );
            assert_eq!(
                above.multiply(&below, reserve).unwrap(),
                product,
                "multiplication commutes"
            );
            assert_eq!(below.subtract(&above, reserve).unwrap().to_string(), "-2");
            let negative = below.clone().negated();
            assert_eq!(
                negative.multiply(&negative, reserve).unwrap(),
                below.multiply(&below, reserve).unwrap()
            );
        }
        assert_eq!(
            power(128).to_string(),
            "340282366920938463463374607431768211456"
        );
    }

    #[test]
    fn exact_arithmetic_storage_failures_return_none() {
        let (a, b) = (exact(1 << 40), exact(-(1 << 70)));
        let fail = |_: &mut Vec<u32>, _: usize| false;
        assert_eq!(a.add(&b, fail), None);
        assert_eq!(a.add(&a, fail), None);
        assert_eq!(a.subtract(&b, fail), None);
        assert_eq!(a.multiply(&b, fail), None);
        // Multiplying by zero allocates nothing.
        assert_eq!(a.multiply(&exact(0), fail), Some(exact(0)));
    }
}
