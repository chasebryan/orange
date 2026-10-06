//! Exact properties of a linear layer: an n x n matrix over GF(2).
//!
//! A [`LinearMap`] is given by its columns: column j is the image of the
//! input whose only set bit is bit j, so the map sends x to the sum of the
//! columns of the bits set in x. Its n <= 128 bits are grouped into
//! k = n / w words of w bits, word c holding bits c w through c w + w - 1,
//! and the branch numbers count nonzero words. Every number is exact for
//! the matrix. Whether a program computes that matrix at every input is
//! the caller's to establish; [`LinearMap`] knows only the matrix.

use super::{AnalysisError, Computed, MAX_ANALYSIS_OPERATIONS, filled};

/// Most bits of an analyzed linear layer.
pub const MAX_LAYER_BITS: u32 = 128;

/// Widest word for which branch numbers are computed; each word position
/// keeps a table of its 2^w images.
const MAX_BRANCH_WORD_BITS: u32 = 16;

/// Widest word for which the fields GF(2^w) of the blocks are searched.
const MAX_FIELD_WORD_BITS: u32 = 8;

/// A linear map from n bits to n bits, grouped into words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinearMap {
    bits: u32,
    word_bits: u32,
    columns: Vec<u128>,
}

/// The fields GF(2^w) over which every w x w block of a matrix is
/// multiplication by a constant, and the matrix of those constants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fields {
    /// Every modulus, an irreducible polynomial of degree w with bit i the
    /// coefficient of x^i, in ascending order.
    pub moduli: Vec<u32>,
    /// The k x k constants over the first modulus, row r and column c at
    /// index r k + c: the image of the word 1 at input word c, read at
    /// output word r.
    pub entries: Vec<u32>,
}

/// The exact properties of a linear layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayerSummary {
    /// The rank of the matrix over GF(2); n exactly when it is invertible.
    pub rank: u32,
    /// The dimension of the space of fixed points, {x : M x = x}.
    pub fixed_dimension: u32,
    /// Whether M M is the identity.
    pub involution: bool,
    /// The XOR gates of the rows taken one at a time: the sum over rows of
    /// one less than the row's weight, a zero row costing nothing.
    pub xor_count: u64,
    /// The least wt(x) + wt(M x) over x != 0, counting nonzero words.
    pub differential_branch: Computed<u32>,
    /// The least wt(b) + wt(M^T b) over b != 0: the input and output masks
    /// of a nonzero correlation.
    pub linear_branch: Computed<u32>,
    /// The fields of the blocks, for words of 2 through 8 bits; `None` for
    /// other widths.
    pub fields: Option<Fields>,
}

impl LinearMap {
    /// Returns the map of `bits` bits in words of `word_bits` bits whose
    /// column j is `columns[j]`, or `None` unless 1 <= bits <=
    /// [`MAX_LAYER_BITS`], `word_bits` is a power of two dividing `bits`,
    /// there are exactly `bits` columns, and each is below 2^bits.
    #[must_use]
    pub fn new(bits: u32, word_bits: u32, columns: &[u128]) -> Option<Self> {
        if !(1..=MAX_LAYER_BITS).contains(&bits)
            || !word_bits.is_power_of_two()
            || !bits.is_multiple_of(word_bits)
            || usize::try_from(bits).ok() != Some(columns.len())
        {
            return None;
        }
        let mask = low_mask(bits);
        if columns.iter().any(|column| column & !mask != 0) {
            return None;
        }
        let mut stored = Vec::new();
        stored.try_reserve_exact(columns.len()).ok()?;
        stored.extend_from_slice(columns);
        Some(Self {
            bits,
            word_bits,
            columns: stored,
        })
    }

    /// Returns n, the number of bits.
    #[must_use]
    pub const fn bits(&self) -> u32 {
        self.bits
    }

    /// Returns w, the number of bits of a word.
    #[must_use]
    pub const fn word_bits(&self) -> u32 {
        self.word_bits
    }

    /// Returns k = n / w, the number of words.
    #[must_use]
    pub const fn words(&self) -> u32 {
        match self.bits.checked_div(self.word_bits) {
            Some(words) => words,
            None => 0,
        }
    }

    /// Returns the columns: column j is the image of bit j.
    #[must_use]
    pub fn columns(&self) -> &[u128] {
        &self.columns
    }

    /// Returns M x, the sum of the columns of the bits set in x.
    #[must_use]
    pub fn apply(&self, x: u128) -> u128 {
        let mut rest = x & low_mask(self.bits);
        let mut image = 0;
        while rest != 0 {
            let column = usize::try_from(rest.trailing_zeros())
                .ok()
                .and_then(|bit| self.columns.get(bit));
            image ^= column.copied().unwrap_or(0);
            rest &= rest.wrapping_sub(1);
        }
        image
    }

    /// Returns the rows: bit j of row i is the coefficient of input bit j
    /// in output bit i.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when the rows cannot be
    /// reserved.
    pub fn rows(&self) -> Result<Vec<u128>, AnalysisError> {
        let mut rows = filled(0_u128, self.columns.len())?;
        for (j, column) in self.columns.iter().enumerate() {
            let input = unit(j);
            for (i, row) in rows.iter_mut().enumerate() {
                if column & unit(i) != 0 {
                    *row |= input;
                }
            }
        }
        Ok(rows)
    }

    /// Returns the transpose, the map from output masks to input masks:
    /// b . (M x) = (M^T b) . x for every x and b.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when the columns cannot be
    /// reserved.
    pub fn transpose(&self) -> Result<Self, AnalysisError> {
        Ok(Self {
            bits: self.bits,
            word_bits: self.word_bits,
            columns: self.rows()?,
        })
    }

    /// Returns d when the affine map x -> M x + `constant` has exactly 2^d
    /// fixed points, and `None` when it has none: the solutions of
    /// (M + I) x = `constant`.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when the system cannot be
    /// reserved.
    pub fn affine_fixed_dimension(&self, constant: u128) -> Result<Option<u32>, AnalysisError> {
        let mut shifted = filled(0_u128, self.columns.len().saturating_add(1))?;
        for (j, (sum, column)) in shifted.iter_mut().zip(&self.columns).enumerate() {
            *sum = column ^ unit(j);
        }
        let rank = gf2_rank(shifted.get(..self.columns.len()).unwrap_or_default());
        if let Some(last) = shifted.last_mut() {
            *last = constant;
        }
        Ok((gf2_rank(&shifted) == rank).then(|| self.bits.saturating_sub(rank)))
    }

    /// Computes every property, each branch number only when its search is
    /// within [`MAX_ANALYSIS_OPERATIONS`].
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when working storage cannot be
    /// reserved.
    pub fn summary(&self) -> Result<LayerSummary, AnalysisError> {
        let rank = gf2_rank(&self.columns);
        let invertible = rank == self.bits;
        let mut shifted = filled(0_u128, self.columns.len())?;
        for (j, (sum, column)) in shifted.iter_mut().zip(&self.columns).enumerate() {
            *sum = column ^ unit(j);
        }
        let fixed_dimension = self.bits.saturating_sub(gf2_rank(&shifted));
        let involution = self
            .columns
            .iter()
            .enumerate()
            .all(|(j, column)| self.apply(*column) == unit(j));
        let xor_count = self
            .rows()?
            .iter()
            .map(|row| u64::from(row.count_ones().saturating_sub(1)))
            .fold(0_u64, u64::saturating_add);
        let inverse = if invertible {
            Some(self.inverse()?)
        } else {
            None
        };
        let differential_branch = self.branch(inverse.as_ref())?;
        let transpose_inverse = match &inverse {
            Some(inverse) => Some(inverse.transpose()?),
            None => None,
        };
        let linear_branch = self.transpose()?.branch(transpose_inverse.as_ref())?;
        let fields = if (2..=MAX_FIELD_WORD_BITS).contains(&self.word_bits) {
            Some(self.fields()?)
        } else {
            None
        };
        Ok(LayerSummary {
            rank,
            fixed_dimension,
            involution,
            xor_count,
            differential_branch,
            linear_branch,
            fields,
        })
    }

    /// Returns the least wt(x) + wt(M x) over x != 0, searching inputs in
    /// order of their weight t.
    ///
    /// For a singular matrix the search stops once t alone reaches the best
    /// sum. For an invertible one each pair (x, y = M x) is also found from
    /// y, by searching M^-1 at the same weights: after weights 1 through t
    /// on both sides, every pair not yet seen has more than t nonzero words
    /// in x and in y, so the search stops once 2 (t + 1) reaches the best
    /// sum. An MDS layer of k words is thereby settled at weight k / 2
    /// rather than k - 1.
    ///
    /// The cost counted is k 2^w per side for the tables of word images and
    /// C(k, t) (2^w - 1)^t per side for the inputs of each weight t.
    fn branch(&self, inverse: Option<&Self>) -> Result<Computed<u32>, AnalysisError> {
        let words = self.words();
        let per_word = 1_u64.checked_shl(self.word_bits).unwrap_or(u64::MAX);
        let values = per_word.saturating_sub(1);
        let sides = if inverse.is_some() { 2 } else { 1 };
        let mut best = u32::MAX;
        // Each side first tabulates the image of every value at every word.
        let mut operations = per_word
            .saturating_mul(u64::from(words))
            .saturating_mul(sides);
        let mut forward = None;
        let mut backward = None;
        for weight in 1..=words {
            let unseen = if inverse.is_some() {
                weight.saturating_mul(2)
            } else {
                weight
            };
            if unseen >= best {
                break;
            }
            let inputs = choose(words, weight)
                .saturating_mul(saturating_power(values, weight))
                .saturating_mul(sides);
            operations = operations.saturating_add(inputs);
            if self.word_bits > MAX_BRANCH_WORD_BITS || operations > MAX_ANALYSIS_OPERATIONS {
                return Ok(Computed::TooCostly(operations));
            }
            let images = match &forward {
                Some(images) => images,
                None => forward.insert(self.word_images()?),
            };
            best = best.min(self.least_at_weight(images, weight)?);
            if let Some(inverse) = inverse {
                let images = match &backward {
                    Some(images) => images,
                    None => backward.insert(inverse.word_images()?),
                };
                best = best.min(inverse.least_at_weight(images, weight)?);
            }
        }
        Ok(Computed::Done(best))
    }

    /// Returns the inverse of an invertible matrix, by Gauss-Jordan
    /// elimination of its rows beside the identity.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when storage cannot be
    /// reserved; a singular matrix keeps the identity's rows where its own
    /// have no pivot, which the caller never asks of it.
    fn inverse(&self) -> Result<Self, AnalysisError> {
        let mut rows = self.rows()?;
        let mut inverse = filled(0_u128, rows.len())?;
        for (i, row) in inverse.iter_mut().enumerate() {
            *row = unit(i);
        }
        for column in 0..rows.len() {
            let bit = unit(column);
            let Some(pivot) = (column..rows.len())
                .find(|&row| rows.get(row).is_some_and(|value| value & bit != 0))
            else {
                continue;
            };
            rows.swap(column, pivot);
            inverse.swap(column, pivot);
            let (pivot_row, pivot_inverse) = (
                rows.get(column).copied().unwrap_or(0),
                inverse.get(column).copied().unwrap_or(0),
            );
            for (row, (value, inverted)) in rows.iter_mut().zip(inverse.iter_mut()).enumerate() {
                if row != column && *value & bit != 0 {
                    *value ^= pivot_row;
                    *inverted ^= pivot_inverse;
                }
            }
        }
        // `inverse` holds the rows of M^-1; as columns they are (M^-1)^T.
        Self {
            bits: self.bits,
            word_bits: self.word_bits,
            columns: inverse,
        }
        .transpose()
    }

    /// Returns, for each word position c and each value v of a word, the
    /// image of v placed at word c, at index c 2^w + v.
    fn word_images(&self) -> Result<Vec<u128>, AnalysisError> {
        let per_word = 1_usize
            .checked_shl(self.word_bits)
            .ok_or(AnalysisError::Allocation)?;
        let words = usize::try_from(self.words()).map_err(|_| AnalysisError::Allocation)?;
        let mut images = filled(0_u128, per_word.saturating_mul(words))?;
        for (word, chunk) in images.chunks_exact_mut(per_word).enumerate() {
            let first = word.saturating_mul(usize::try_from(self.word_bits).unwrap_or(0));
            for (value, image) in chunk.iter_mut().enumerate() {
                let placed = u128::try_from(value)
                    .ok()
                    .and_then(|value| value.checked_shl(u32::try_from(first).ok()?))
                    .unwrap_or(0);
                *image = self.apply(placed);
            }
        }
        Ok(images)
    }

    /// Returns the least weight + wt(M x) over inputs x with exactly
    /// `weight` nonzero words. The first weight - 1 nonzero words step like
    /// an odometer; the last runs through its table directly.
    fn least_at_weight(&self, images: &[u128], weight: u32) -> Result<u32, AnalysisError> {
        let words = usize::try_from(self.words()).map_err(|_| AnalysisError::Allocation)?;
        let chosen = usize::try_from(weight).map_err(|_| AnalysisError::Allocation)?;
        let leading = chosen.saturating_sub(1);
        let per_word = 1_usize
            .checked_shl(self.word_bits)
            .ok_or(AnalysisError::Allocation)?;
        let starts = word_starts(self.bits, self.word_bits);
        // positions[i] is the word of the i-th nonzero input word, values[i]
        // the value of each but the last, and sums[i + 1] the image of the
        // first i + 1 of them.
        let mut positions = filled(0_usize, chosen)?;
        for (i, position) in positions.iter_mut().enumerate() {
            *position = i;
        }
        let mut values = filled(1_usize, leading)?;
        let mut sums = filled(0_u128, chosen)?;
        let mut least = u32::MAX;
        loop {
            refresh(&mut sums, images, &positions, &values, per_word, 0);
            let last = positions.last().copied().unwrap_or(0);
            let first = last.saturating_mul(per_word);
            let row = images
                .get(first.saturating_add(1)..first.saturating_add(per_word))
                .unwrap_or_default();
            loop {
                let prefix = sums.last().copied().unwrap_or(0);
                for image in row {
                    least = least.min(word_weight(prefix ^ image, self.word_bits, starts));
                }
                let Some(changed) = advance(&mut values, per_word) else {
                    break;
                };
                refresh(&mut sums, images, &positions, &values, per_word, changed);
            }
            if !next_combination(&mut positions, words) {
                return Ok(weight.saturating_add(least));
            }
            values.fill(1);
        }
    }

    /// Returns the fields over which every w x w block is multiplication
    /// by a constant.
    fn fields(&self) -> Result<Fields, AnalysisError> {
        let width = self.word_bits;
        let words = usize::try_from(self.words()).map_err(|_| AnalysisError::Allocation)?;
        let word = usize::try_from(width).map_err(|_| AnalysisError::Allocation)?;
        let mut moduli = Vec::new();
        for modulus in irreducible(width) {
            let commutes = (0..words).all(|row| {
                (0..words).all(|column| {
                    let block = |value: u32| self.block_image(row, column, value);
                    (0..word).all(|bit| {
                        let basis = u32::try_from(bit)
                            .ok()
                            .and_then(|bit| 1_u32.checked_shl(bit))
                            .unwrap_or(0);
                        block(times_x(basis, modulus, width))
                            == times_x(block(basis), modulus, width)
                    })
                })
            });
            if commutes {
                moduli
                    .try_reserve(1)
                    .map_err(|_| AnalysisError::Allocation)?;
                moduli.push(modulus);
            }
        }
        let mut entries = Vec::new();
        if !moduli.is_empty() {
            entries
                .try_reserve_exact(words.saturating_mul(words))
                .map_err(|_| AnalysisError::Allocation)?;
            for row in 0..words {
                for column in 0..words {
                    entries.push(self.block_image(row, column, 1));
                }
            }
        }
        Ok(Fields { moduli, entries })
    }

    /// Returns word `row` of the image of `value` placed at word `column`.
    fn block_image(&self, row: usize, column: usize, value: u32) -> u32 {
        let width = usize::try_from(self.word_bits).unwrap_or(0);
        let shift = |word: usize| u32::try_from(word.saturating_mul(width)).unwrap_or(u32::MAX);
        let placed = u128::from(value).checked_shl(shift(column)).unwrap_or(0);
        let image = self.apply(placed).checked_shr(shift(row)).unwrap_or(0);
        u32::try_from(image & low_mask(self.word_bits)).unwrap_or(0)
    }
}

/// Returns the rank over GF(2) of a set of vectors.
fn gf2_rank(vectors: &[u128]) -> u32 {
    // basis[h] holds the reduced vector whose highest set bit is h.
    let mut basis = [0_u128; 128];
    let mut rank = 0_u32;
    for &vector in vectors {
        let mut rest = vector;
        while rest != 0 {
            let highest =
                usize::try_from(127_u32.saturating_sub(rest.leading_zeros())).unwrap_or(0);
            let Some(slot) = basis.get_mut(highest) else {
                break;
            };
            if *slot == 0 {
                *slot = rest;
                rank = rank.saturating_add(1);
                break;
            }
            rest ^= *slot;
        }
    }
    rank
}

/// Returns the value with only bit `bit` set, or 0 beyond bit 127.
fn unit(bit: usize) -> u128 {
    u32::try_from(bit)
        .ok()
        .and_then(|bit| 1_u128.checked_shl(bit))
        .unwrap_or(0)
}

/// Returns the low `bits` bits set.
fn low_mask(bits: u32) -> u128 {
    1_u128
        .checked_shl(bits)
        .map_or(u128::MAX, |power| power.wrapping_sub(1))
}

/// Returns the lowest bit of every word set.
fn word_starts(bits: u32, word_bits: u32) -> u128 {
    let mut starts = 0_u128;
    let mut bit = 0_u32;
    while bit < bits {
        starts |= 1_u128.checked_shl(bit).unwrap_or(0);
        bit = bit.saturating_add(word_bits);
    }
    starts
}

/// Returns the number of nonzero words of `value`. Folding each word onto
/// its lowest bit needs log2 w shifts, since w is a power of two.
fn word_weight(value: u128, word_bits: u32, starts: u128) -> u32 {
    let mut folded = value;
    let mut shift = 1_u32;
    while shift < word_bits {
        folded |= folded.checked_shr(shift).unwrap_or(0);
        shift = shift.saturating_mul(2);
    }
    (folded & starts).count_ones()
}

/// Recomputes sums[i + 1] = sums[i] ^ image of (positions[i], values[i])
/// for every word i from `from` that has a value.
fn refresh(
    sums: &mut [u128],
    images: &[u128],
    positions: &[usize],
    values: &[usize],
    per_word: usize,
    from: usize,
) {
    for i in from..values.len() {
        let previous = sums.get(i).copied().unwrap_or(0);
        let image = positions
            .get(i)
            .zip(values.get(i))
            .and_then(|(position, value)| {
                images.get(position.saturating_mul(per_word).saturating_add(*value))
            })
            .copied()
            .unwrap_or(0);
        if let Some(sum) = sums.get_mut(i.saturating_add(1)) {
            *sum = previous ^ image;
        }
    }
}

/// Steps the nonzero word values like an odometer, the last fastest, and
/// returns the first index that changed, or `None` after the last.
fn advance(values: &mut [usize], per_word: usize) -> Option<usize> {
    for (i, value) in values.iter_mut().enumerate().rev() {
        if value.saturating_add(1) < per_word {
            *value = value.saturating_add(1);
            return Some(i);
        }
        *value = 1;
    }
    None
}

/// Steps `positions`, increasing word positions out of `words`, to the
/// next combination in lexicographic order; `false` after the last.
fn next_combination(positions: &mut [usize], words: usize) -> bool {
    let chosen = positions.len();
    let Some(mut i) = chosen.checked_sub(1) else {
        return false;
    };
    loop {
        // Position i can reach at most words - chosen + i.
        let limit = words.saturating_sub(chosen).saturating_add(i);
        if positions.get(i).is_some_and(|position| *position < limit) {
            let start = positions.get(i).copied().unwrap_or(0).saturating_add(1);
            for (offset, position) in positions.iter_mut().skip(i).enumerate() {
                *position = start.saturating_add(offset);
            }
            return true;
        }
        let Some(previous) = i.checked_sub(1) else {
            return false;
        };
        i = previous;
    }
}

/// Returns C(n, k), saturating.
fn choose(n: u32, k: u32) -> u64 {
    let mut result = 1_u64;
    for i in 0..k {
        result = result
            .saturating_mul(u64::from(n.saturating_sub(i)))
            .checked_div(u64::from(i.saturating_add(1)))
            .unwrap_or(u64::MAX);
    }
    result
}

/// Returns base^exponent, saturating.
fn saturating_power(base: u64, exponent: u32) -> u64 {
    (0..exponent).fold(1_u64, |power, _| power.saturating_mul(base))
}

/// Returns x v modulo `modulus`, a polynomial of degree `width`.
fn times_x(value: u32, modulus: u32, width: u32) -> u32 {
    let doubled = value.checked_shl(1).unwrap_or(0);
    if doubled.checked_shr(width).unwrap_or(0) & 1 == 1 {
        doubled ^ modulus
    } else {
        doubled
    }
}

/// Returns every irreducible polynomial of degree `width` over GF(2), in
/// ascending order.
fn irreducible(width: u32) -> impl Iterator<Item = u32> {
    let first = 1_u32.checked_shl(width).unwrap_or(0);
    let end = first.checked_shl(1).unwrap_or(0);
    (first..end).filter(move |&candidate| {
        let half = width.checked_div(2).unwrap_or(0);
        let low = 2_u32;
        let high = 1_u32.checked_shl(half.saturating_add(1)).unwrap_or(0);
        !(low..high).any(|divisor| remainder(candidate, divisor) == 0)
    })
}

/// Returns `dividend` modulo `divisor` as polynomials over GF(2).
fn remainder(dividend: u32, divisor: u32) -> u32 {
    let divisor_degree = 31_u32.saturating_sub(divisor.leading_zeros());
    let mut rest = dividend;
    while rest != 0 {
        let degree = 31_u32.saturating_sub(rest.leading_zeros());
        if degree < divisor_degree {
            break;
        }
        rest ^= divisor
            .checked_shl(degree.saturating_sub(divisor_degree))
            .unwrap_or(0);
    }
    rest
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AES MixColumns over GF(2^8) mod x^8 + x^4 + x^3 + x + 1, FIPS 197
    /// section 5.1.3, on four bytes with byte i in bits 8i through 8i + 7.
    fn aes_mix_columns() -> LinearMap {
        let xtime = |a: u32| times_x(a, 0x11b, 8);
        let times = |a: u32, c: u32| match c {
            1 => a,
            2 => xtime(a),
            _ => xtime(a) ^ a,
        };
        let rows = [[2, 3, 1, 1], [1, 2, 3, 1], [1, 1, 2, 3], [3, 1, 1, 2]];
        let mut columns = Vec::new();
        for j in 0..32 {
            let word = j / 8;
            let value = 1_u32 << (j % 8);
            let mut image = 0_u128;
            for (r, row) in rows.iter().enumerate() {
                image |= u128::from(times(value, row[word])) << (8 * r);
            }
            columns.push(image);
        }
        LinearMap::new(32, 8, &columns).unwrap()
    }

    fn from_function(bits: u32, word_bits: u32, function: impl Fn(u128) -> u128) -> LinearMap {
        let columns: Vec<u128> = (0..bits).map(|j| function(1_u128 << j)).collect();
        LinearMap::new(bits, word_bits, &columns).unwrap()
    }

    fn naive_word_weight(value: u128, bits: u32, word_bits: u32) -> u32 {
        let mask = (1_u128 << word_bits) - 1;
        (0..bits / word_bits)
            .filter(|word| (value >> (word * word_bits)) & mask != 0)
            .count() as u32
    }

    /// The branch number by visiting every nonzero input.
    fn naive_branch(map: &LinearMap) -> u32 {
        (1..1_u128 << map.bits())
            .map(|x| {
                naive_word_weight(x, map.bits(), map.word_bits())
                    + naive_word_weight(map.apply(x), map.bits(), map.word_bits())
            })
            .min()
            .unwrap()
    }

    fn generator(seed: u64) -> impl FnMut() -> u64 {
        let mut state = seed;
        move || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            state >> 11
        }
    }

    #[test]
    fn construction_checks_widths_and_columns() {
        assert!(LinearMap::new(0, 1, &[]).is_none());
        assert!(LinearMap::new(129, 1, &[0; 129]).is_none());
        assert!(LinearMap::new(12, 3, &[0; 12]).is_none());
        assert!(LinearMap::new(8, 16, &[0; 8]).is_none());
        assert!(LinearMap::new(8, 4, &[0; 7]).is_none());
        assert!(LinearMap::new(8, 4, &[0x100, 0, 0, 0, 0, 0, 0, 0]).is_none());
        let identity: Vec<u128> = (0..128).map(|j| 1_u128 << j).collect();
        let map = LinearMap::new(128, 8, &identity).unwrap();
        assert_eq!((map.bits(), map.word_bits(), map.words()), (128, 8, 16));
        assert_eq!(map.apply(u128::MAX), u128::MAX);
        assert_eq!(map.transpose().unwrap(), map);
    }

    #[test]
    fn aes_mix_columns_is_mds_over_its_field() {
        let map = aes_mix_columns();
        // FIPS 197 does not list a MixColumns example; this column is the
        // one the Rijndael proposal's test values and many references use.
        let column = u128::from(u32::from_le_bytes([0xdb, 0x13, 0x53, 0x45]));
        assert_eq!(
            map.apply(column),
            u128::from(u32::from_le_bytes([0x8e, 0x4d, 0xa1, 0xbc]))
        );
        let summary = map.summary().unwrap();
        assert_eq!(summary.rank, 32);
        assert_eq!(summary.differential_branch, Computed::Done(5));
        assert_eq!(summary.linear_branch, Computed::Done(5));
        assert!(!summary.involution);
        let fields = summary.fields.unwrap();
        assert_eq!(fields.moduli, [0x11b]);
        assert_eq!(
            fields.entries,
            [2, 3, 1, 1, 1, 2, 3, 1, 1, 1, 2, 3, 3, 1, 1, 2]
        );
    }

    #[test]
    fn branch_numbers_match_every_input_on_small_maps() {
        let mut next = generator(7);
        for (bits, word_bits) in [
            (4, 1),
            (6, 2),
            (8, 2),
            (8, 4),
            (12, 4),
            (12, 2),
            (16, 4),
            (16, 8),
        ] {
            for _ in 0..12 {
                let mut columns: Vec<u128> = (0..bits)
                    .map(|_| u128::from(next()) & ((1_u128 << bits) - 1))
                    .collect();
                // Some sparse and some singular maps as well as dense ones.
                if next().is_multiple_of(3) {
                    for column in &mut columns {
                        *column &= u128::from(next()) & u128::from(next());
                    }
                }
                let map = LinearMap::new(bits, word_bits, &columns).unwrap();
                let summary = map.summary().unwrap();
                assert_eq!(
                    summary.differential_branch,
                    Computed::Done(naive_branch(&map)),
                    "{bits} bits in words of {word_bits}: {columns:x?}"
                );
                let transpose = map.transpose().unwrap();
                assert_eq!(
                    summary.linear_branch,
                    Computed::Done(naive_branch(&transpose))
                );
                // b . (M x) = (M^T b) . x for every x and b.
                for _ in 0..64 {
                    let x = u128::from(next()) & ((1_u128 << bits) - 1);
                    let b = u128::from(next()) & ((1_u128 << bits) - 1);
                    assert_eq!(
                        (b & map.apply(x)).count_ones() % 2,
                        (transpose.apply(b) & x).count_ones() % 2
                    );
                }
                let images: Vec<u128> = (0..1_u128 << bits).map(|x| map.apply(x)).collect();
                let distinct = {
                    let mut sorted = images.clone();
                    sorted.sort_unstable();
                    sorted.dedup();
                    sorted.len()
                };
                assert_eq!(1_usize << summary.rank, distinct);
                if summary.rank == bits {
                    let inverse = map.inverse().unwrap();
                    assert!((0..1_u128 << bits).all(|x| inverse.apply(map.apply(x)) == x));
                }
                let fixed = images
                    .iter()
                    .enumerate()
                    .filter(|(x, image)| *x as u128 == **image)
                    .count();
                assert_eq!(1_usize << summary.fixed_dimension, fixed);
                assert_eq!(
                    summary.involution,
                    (0..1_u128 << bits).all(|x| map.apply(map.apply(x)) == x)
                );
            }
        }
    }

    #[test]
    fn bit_permutations_and_binary_matrices() {
        // The PRESENT player: bit i moves to 16 i mod 63, bit 63 stays.
        let player = from_function(64, 4, |x| {
            (0..64)
                .filter(|i| (x >> i) & 1 == 1)
                .map(|i| 1_u128 << if i == 63 { 63 } else { (16 * i) % 63 })
                .fold(0, |sum, bit| sum | bit)
        });
        let summary = player.summary().unwrap();
        assert_eq!(summary.rank, 64);
        assert_eq!(summary.differential_branch, Computed::Done(2));
        assert_eq!(summary.linear_branch, Computed::Done(2));
        // Its cycles: bits 0, 21, 42 and 63 fixed, the other 60 in 20
        // cycles of three.
        assert_eq!(summary.fixed_dimension, 24);
        assert_eq!(summary.xor_count, 0);
        assert!(!summary.involution);
        // A block that moves one bit is singular, so no field describes it.
        assert!(
            summary
                .fields
                .is_some_and(|fields| fields.moduli.is_empty())
        );

        // Midori's MixColumn, circ(0, 1, 1, 1) on four 4-bit cells: an
        // involution of branch number 4 whose entries are 0 and 1, so every
        // field of 16 elements describes it.
        let midori = from_function(16, 4, |x| {
            let cell = |i: u32| (x >> (4 * (i % 4))) & 0xf;
            (0..4)
                .map(|i| (cell(i + 1) ^ cell(i + 2) ^ cell(i + 3)) << (4 * i))
                .fold(0, |sum, cell| sum | cell)
        });
        let summary = midori.summary().unwrap();
        assert!(summary.involution);
        assert_eq!(summary.differential_branch, Computed::Done(4));
        assert_eq!(summary.linear_branch, Computed::Done(4));
        assert_eq!(summary.xor_count, 32);
        let fields = summary.fields.unwrap();
        assert_eq!(fields.moduli, [0x13, 0x19, 0x1f]);
        assert_eq!(
            fields.entries,
            [0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0]
        );
    }

    #[test]
    fn affine_fixed_points_solve_the_shifted_system() {
        let mut next = generator(11);
        for _ in 0..40 {
            let bits = 6;
            let columns: Vec<u128> = (0..bits).map(|_| u128::from(next()) & 0x3f).collect();
            let map = LinearMap::new(bits, 2, &columns).unwrap();
            let constant = u128::from(next()) & 0x3f;
            let fixed = (0..64_u128)
                .filter(|&x| map.apply(x) ^ constant == x)
                .count();
            match map.affine_fixed_dimension(constant).unwrap() {
                Some(dimension) => assert_eq!(fixed, 1 << dimension),
                None => assert_eq!(fixed, 0),
            }
        }
    }

    #[test]
    fn singular_and_costly_maps() {
        let zero = LinearMap::new(16, 8, &[0; 16]).unwrap();
        let summary = zero.summary().unwrap();
        assert_eq!(summary.rank, 0);
        assert_eq!(summary.fixed_dimension, 0);
        assert_eq!(summary.differential_branch, Computed::Done(1));
        assert_eq!(summary.xor_count, 0);
        // Over 32-bit words the search needs 2^32 - 1 inputs of one word
        // per position, more than the limit allows for two words.
        let identity: Vec<u128> = (0..64).map(|j| 1_u128 << j).collect();
        let wide = LinearMap::new(64, 32, &identity).unwrap();
        assert!(matches!(
            wide.summary().unwrap().differential_branch,
            Computed::TooCostly(_)
        ));
        // A 16-byte identity needs only inputs of one word.
        let bytes =
            LinearMap::new(128, 8, &(0..128).map(|j| 1_u128 << j).collect::<Vec<_>>()).unwrap();
        assert_eq!(
            bytes.summary().unwrap().differential_branch,
            Computed::Done(2)
        );
    }

    #[test]
    fn helpers_count_and_enumerate_exactly() {
        assert_eq!(choose(16, 3), 560);
        assert_eq!(choose(128, 2), 8128);
        assert_eq!(choose(4, 5), 0);
        assert_eq!(saturating_power(255, 3), 16_581_375);
        assert_eq!(irreducible(2).collect::<Vec<_>>(), [0x7]);
        assert_eq!(irreducible(4).collect::<Vec<_>>(), [0x13, 0x19, 0x1f]);
        assert_eq!(irreducible(8).count(), 30);
        assert!(irreducible(8).any(|modulus| modulus == 0x11b));
        assert_eq!(word_weight(0x0100_0001, 8, word_starts(32, 8)), 2);
        assert_eq!(word_weight(0x8000_0080, 8, word_starts(32, 8)), 2);
        let mut positions = vec![0, 1];
        let mut seen = vec![positions.clone()];
        while next_combination(&mut positions, 4) {
            seen.push(positions.clone());
        }
        assert_eq!(seen, [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]]);
    }
}
