//! Exact cryptanalytic properties of a function between small bit spaces.
//!
//! A [`BitFunction`] is a table: the value of a function F from n-bit inputs
//! to m-bit outputs at each of its 2^n inputs, with 1 <= n, m <= 16. From
//! the table alone this module computes, by complete enumeration, the
//! properties a cryptanalyst first asks of an S-box or a Boolean function:
//! its difference distribution, its Walsh spectrum, its algebraic normal
//! form, its boomerang connectivity, the implicit quadratic equations its
//! graph satisfies, and its cycle structure. Every number is exact. No
//! property is sampled, estimated or proved by any other means, and none is
//! a claim about the security of a cipher that uses the function.
//!
//! Bit i of an input or output is the bit of weight 2^i. For masks a and b,
//! a.x is the parity of the bits of x selected by a.

use std::fmt;

pub mod linear;
pub mod trails;

/// Most input or output bits of an analyzed function.
pub const MAX_ANALYSIS_BITS: u32 = 16;

/// Most elementary operations one property may cost before it is reported
/// as not computed rather than computed.
pub const MAX_ANALYSIS_OPERATIONS: u64 = 1 << 32;

/// Most input or output bits for which complete tables are produced.
pub const MAX_TABLE_BITS: u32 = 10;

/// Words of the bitmap that marks the values of a 2^16-input function.
const PERMUTATION_WORDS: usize = 1 << (MAX_ANALYSIS_BITS - 6);

/// A function from n-bit inputs to m-bit outputs, given by its value at
/// every input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BitFunction {
    input_bits: u32,
    output_bits: u32,
    values: Vec<usize>,
}

/// A property that was computed, or the cost that kept it from being
/// computed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Computed<T> {
    /// The exact value of the property.
    Done(T),
    /// The property would cost this many elementary operations, more than
    /// [`MAX_ANALYSIS_OPERATIONS`].
    TooCostly(u64),
}

impl<T> Computed<T> {
    /// Returns the computed value, if there is one.
    #[must_use]
    pub const fn done(&self) -> Option<&T> {
        match self {
            Self::Done(value) => Some(value),
            Self::TooCostly(_) => None,
        }
    }
}

/// How a function's output changes with a difference in its input.
///
/// DDT(a, b) is the number of inputs x with F(x) ^ F(x ^ a) = b. Every
/// figure here ranges over the nonzero input differences a and every
/// output difference b.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Differential {
    /// The largest DDT(a, b): the differential uniformity.
    pub uniformity: u32,
    /// How many pairs (a, b) reach the uniformity.
    pub reached: u64,
    /// Each DDT value that occurs, ascending, with how many pairs have it.
    pub spectrum: Vec<(u32, u64)>,
    /// The least wt(a) + wt(b) over the pairs with DDT(a, b) > 0.
    pub branch_number: u32,
    /// For a Boolean function, the largest |2^n - 2 DDT(a, 1)|: the
    /// absolute indicator. `None` when the output has more than one bit.
    pub absolute_indicator: Option<u32>,
}

/// How well linear functions of the input approximate linear functions of
/// the output.
///
/// The Walsh coefficient W(a, b) is the sum over x of
/// (-1)^(b.F(x) ^ a.x). Every figure here ranges over the nonzero output
/// masks b and every input mask a.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Linear {
    /// The largest |W(a, b)|: the linearity. The nonlinearity is
    /// 2^(n - 1) - linearity / 2.
    pub linearity: u32,
    /// Each |W(a, b)| that occurs, ascending, with how many pairs have it.
    pub spectrum: Vec<(u32, u64)>,
    /// The least wt(a) + wt(b) over the pairs with W(a, b) != 0.
    pub branch_number: u32,
    /// The largest t such that W(a, b) = 0 whenever 1 <= wt(a) <= t: the
    /// order of correlation immunity, n when no such a has W(a, b) != 0.
    pub correlation_immunity: u32,
}

/// The algebraic degrees of a function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Algebraic {
    /// The largest degree of a coordinate function: the algebraic degree.
    pub degree: u32,
    /// The least degree of a component function b.F with b != 0.
    pub minimum_degree: u32,
    /// The algebraic degree of the inverse, for a permutation.
    pub inverse_degree: Option<u32>,
}

/// The implicit equations the graph {(x, F(x))} satisfies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Equations {
    /// The dimension of the space of polynomials of degree at most 2 in
    /// the input and output bits that vanish at every (x, F(x)).
    pub quadratic: usize,
    /// The dimension of the subspace of bi-affine polynomials: those whose
    /// quadratic terms each multiply an input bit by an output bit.
    pub bi_affine: usize,
}

/// The cycle structure of a permutation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cycles {
    /// Each cycle length that occurs, descending, with how many cycles have
    /// it. Fixed points are the cycles of length 1.
    pub lengths: Vec<(usize, usize)>,
}

/// Every summary property of one function.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Summary {
    /// The number of distinct output values.
    pub image: usize,
    /// Whether every output value occurs 2^(n - m) times; `None` when the
    /// output has more bits than the input.
    pub balanced: Option<bool>,
    /// The number of inputs x with F(x) = x; `None` unless n = m.
    pub fixed_points: Option<usize>,
    /// The cycle structure, for a permutation.
    pub cycles: Option<Cycles>,
    /// The number of inputs x with F(x) = 1, for a Boolean function.
    pub weight: Option<usize>,
    /// The difference distribution.
    pub differential: Computed<Differential>,
    /// The Walsh spectrum.
    pub linear: Computed<Linear>,
    /// The algebraic degrees.
    pub algebraic: Computed<Algebraic>,
    /// The implicit quadratic equations.
    pub equations: Computed<Equations>,
    /// The largest BCT(a, b) with a, b != 0, for a permutation.
    pub boomerang_uniformity: Option<Computed<u32>>,
}

/// A failure to complete an analysis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisError {
    /// Working storage could not be reserved.
    Allocation,
    /// A complete table was asked of a function with more than
    /// [`MAX_TABLE_BITS`] input or output bits.
    TableTooLarge,
    /// A table that exists only for permutations was asked of a function
    /// that is not one.
    NotPermutation,
}

impl fmt::Display for AnalysisError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Allocation => "analysis storage could not be reserved",
            Self::TableTooLarge => "complete tables are limited to 10 input and 10 output bits",
            Self::NotPermutation => "the boomerang connectivity table needs a permutation",
        })
    }
}

/// One monomial of an algebraic normal form: the product of the input bits
/// selected by a mask, the constant 1 for the empty mask.
pub type Monomial = usize;

impl BitFunction {
    /// Returns the function from `input_bits` to `output_bits` bits whose
    /// value at x is `values[x]`, or `None` unless both widths lie in
    /// 1..=[`MAX_ANALYSIS_BITS`], there are exactly 2^input_bits values,
    /// and each is below 2^output_bits.
    #[must_use]
    pub fn new(input_bits: u32, output_bits: u32, values: &[u32]) -> Option<Self> {
        if !(1..=MAX_ANALYSIS_BITS).contains(&input_bits)
            || !(1..=MAX_ANALYSIS_BITS).contains(&output_bits)
            || values.len() != size(input_bits)
        {
            return None;
        }
        let outputs = size(output_bits);
        let mut stored = Vec::new();
        stored.try_reserve_exact(values.len()).ok()?;
        for &value in values {
            let value = usize::try_from(value)
                .ok()
                .filter(|value| *value < outputs)?;
            stored.push(value);
        }
        Some(Self {
            input_bits,
            output_bits,
            values: stored,
        })
    }

    /// Returns the number of input bits n.
    #[must_use]
    pub const fn input_bits(&self) -> u32 {
        self.input_bits
    }

    /// Returns the number of output bits m.
    #[must_use]
    pub const fn output_bits(&self) -> u32 {
        self.output_bits
    }

    /// Returns the value at each input, in input order.
    #[must_use]
    pub fn values(&self) -> &[usize] {
        &self.values
    }

    /// Returns whether the function is a permutation of its n-bit space.
    /// The answer needs no allocation: values seen are marked in a fixed
    /// bitmap of 2^16 bits.
    #[must_use]
    pub fn is_permutation(&self) -> bool {
        if self.input_bits != self.output_bits {
            return false;
        }
        let mut seen = [0_u64; PERMUTATION_WORDS];
        for &value in &self.values {
            if bit(&seen, value) {
                return false;
            }
            set_bit(&mut seen, value);
        }
        true
    }

    /// Computes every summary property whose cost is within
    /// [`MAX_ANALYSIS_OPERATIONS`].
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when working storage cannot be
    /// reserved.
    pub fn summary(&self) -> Result<Summary, AnalysisError> {
        let n = u64::from(self.input_bits);
        let m = u64::from(self.output_bits);
        let inputs = 1_u64.checked_shl(self.input_bits).unwrap_or(u64::MAX);
        let outputs = 1_u64.checked_shl(self.output_bits).unwrap_or(u64::MAX);
        let inverse = self.inverse_values()?;

        let mut counts = filled(0_usize, size(self.output_bits))?;
        for &value in &self.values {
            if let Some(count) = counts.get_mut(value) {
                *count = count.saturating_add(1);
            }
        }
        let image = counts.iter().filter(|count| **count != 0).count();
        let balanced = (self.input_bits >= self.output_bits).then(|| {
            let share = size(self.input_bits.saturating_sub(self.output_bits));
            counts.iter().all(|count| *count == share)
        });
        let fixed_points = (self.input_bits == self.output_bits).then(|| {
            self.values
                .iter()
                .enumerate()
                .filter(|(x, value)| x == *value)
                .count()
        });
        let weight = (self.output_bits == 1).then(|| counts.get(1).copied().unwrap_or(0));
        let cycles = match &inverse {
            Some(_) => Some(self.cycles()?),
            None => None,
        };

        let differential = within(inputs.saturating_mul(inputs), || self.differential())?;
        let linear = within(
            outputs
                .saturating_mul(inputs)
                .saturating_mul(n.saturating_add(1)),
            || self.linear(),
        )?;
        // Each component meets at most n + 1 weight sets of 2^n / 64 words;
        // each coordinate and inverse coordinate costs one transform.
        let algebraic = within(
            outputs
                .saturating_mul(inputs.div_ceil(64))
                .saturating_mul(n.saturating_add(1))
                .saturating_add(m.saturating_mul(2).saturating_mul(inputs).saturating_mul(n)),
            || self.algebraic(inverse.as_deref()),
        )?;
        let variables = n.saturating_add(m);
        let monomials = variables
            .saturating_mul(variables.saturating_sub(1))
            .div_ceil(2)
            .saturating_add(variables)
            .saturating_add(1);
        let equations = within(
            monomials
                .saturating_mul(monomials)
                .saturating_mul(inputs.div_ceil(64)),
            || self.equations(),
        )?;
        let boomerang_uniformity = match &inverse {
            Some(inverse) => Some(within(
                inputs.saturating_mul(inputs).saturating_mul(inputs),
                || self.boomerang_uniformity(inverse),
            )?),
            None => None,
        };

        Ok(Summary {
            image,
            balanced,
            fixed_points,
            cycles,
            weight,
            differential,
            linear,
            algebraic,
            equations,
            boomerang_uniformity,
        })
    }

    /// Returns the difference distribution table, row a and column b at
    /// index a * 2^m + b.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::TableTooLarge`] beyond [`MAX_TABLE_BITS`],
    /// or [`AnalysisError::Allocation`].
    pub fn difference_table(&self) -> Result<Vec<u32>, AnalysisError> {
        self.check_table_size()?;
        let outputs = size(self.output_bits);
        let mut table = filled(0_u32, self.values.len().saturating_mul(outputs))?;
        for (a, row) in table.chunks_exact_mut(outputs).enumerate() {
            difference_row(&self.values, a, row);
        }
        Ok(table)
    }

    /// Returns the linear approximation table: at index a * 2^m + b, the
    /// number of inputs x with a.x = b.F(x) less 2^(n - 1), which is
    /// W(a, b) / 2.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::TableTooLarge`] beyond [`MAX_TABLE_BITS`],
    /// or [`AnalysisError::Allocation`].
    pub fn linear_table(&self) -> Result<Vec<i32>, AnalysisError> {
        self.check_table_size()?;
        let outputs = size(self.output_bits);
        let mut table = filled(0_i32, self.values.len().saturating_mul(outputs))?;
        let mut column = filled(0_i32, self.values.len())?;
        for b in 0..outputs {
            self.walsh_column(b, &mut column);
            for (row, coefficient) in table.chunks_exact_mut(outputs).zip(&column) {
                if let Some(entry) = row.get_mut(b) {
                    *entry = coefficient.checked_div(2).unwrap_or(0);
                }
            }
        }
        Ok(table)
    }

    /// Returns the boomerang connectivity table of a permutation: at index
    /// a * 2^n + b, the number of inputs x with
    /// F^-1(F(x) ^ b) ^ F^-1(F(x ^ a) ^ b) = a.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::NotPermutation`] unless the function is a
    /// permutation, [`AnalysisError::TableTooLarge`] beyond
    /// [`MAX_TABLE_BITS`], or [`AnalysisError::Allocation`].
    pub fn boomerang_table(&self) -> Result<Vec<u32>, AnalysisError> {
        self.check_table_size()?;
        let inverse = self
            .inverse_values()?
            .ok_or(AnalysisError::NotPermutation)?;
        let inputs = self.values.len();
        let mut table = filled(0_u32, inputs.saturating_mul(inputs))?;
        for (a, row) in table.chunks_exact_mut(inputs).enumerate() {
            boomerang_row(&self.values, &inverse, a, row);
        }
        Ok(table)
    }

    /// Returns the algebraic normal form of each output bit, lowest first:
    /// the monomials whose sum is that bit, in ascending degree and, within
    /// a degree, ascending mask.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::TableTooLarge`] beyond [`MAX_TABLE_BITS`],
    /// or [`AnalysisError::Allocation`].
    pub fn normal_form(&self) -> Result<Vec<Vec<Monomial>>, AnalysisError> {
        self.check_table_size()?;
        let order = monomial_order(self.input_bits)?;
        let mut forms = Vec::new();
        forms
            .try_reserve_exact(
                usize::try_from(self.output_bits).map_err(|_| AnalysisError::Allocation)?,
            )
            .map_err(|_| AnalysisError::Allocation)?;
        let mut coefficients = filled(0_u8, self.values.len())?;
        for bit in 0..self.output_bits {
            for (coefficient, value) in coefficients.iter_mut().zip(&self.values) {
                *coefficient = u8::from(value.checked_shr(bit).unwrap_or(0) & 1 == 1);
            }
            mobius(&mut coefficients);
            let mut form = Vec::new();
            let terms = coefficients.iter().filter(|c| **c != 0).count();
            form.try_reserve_exact(terms)
                .map_err(|_| AnalysisError::Allocation)?;
            form.extend(
                order
                    .iter()
                    .copied()
                    .filter(|monomial| coefficients.get(*monomial) == Some(&1)),
            );
            forms.push(form);
        }
        Ok(forms)
    }

    fn check_table_size(&self) -> Result<(), AnalysisError> {
        if self.input_bits > MAX_TABLE_BITS || self.output_bits > MAX_TABLE_BITS {
            Err(AnalysisError::TableTooLarge)
        } else {
            Ok(())
        }
    }

    fn value_at(&self, x: usize) -> usize {
        self.values.get(x).copied().unwrap_or(0)
    }

    /// Returns the inverse table when the function is a permutation of an
    /// n-bit space, and `None` when it is not.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when the table of a permutation
    /// cannot be reserved; that failure is never read as "not a permutation".
    fn inverse_values(&self) -> Result<Option<Vec<usize>>, AnalysisError> {
        if !self.is_permutation() {
            return Ok(None);
        }
        let mut inverse = filled(0_usize, self.values.len())?;
        for (x, &value) in self.values.iter().enumerate() {
            if let Some(slot) = inverse.get_mut(value) {
                *slot = x;
            }
        }
        Ok(Some(inverse))
    }

    fn cycles(&self) -> Result<Cycles, AnalysisError> {
        let mut seen = filled(false, self.values.len())?;
        let mut tally = filled(0_usize, self.values.len().saturating_add(1))?;
        for start in 0..self.values.len() {
            let mut length = 0_usize;
            let mut x = start;
            while let Some(visited) = seen.get_mut(x) {
                if *visited {
                    break;
                }
                *visited = true;
                length = length.saturating_add(1);
                x = self.value_at(x);
            }
            if length != 0
                && let Some(count) = tally.get_mut(length)
            {
                *count = count.saturating_add(1);
            }
        }
        let mut lengths = Vec::new();
        let distinct = tally.iter().filter(|count| **count != 0).count();
        lengths
            .try_reserve_exact(distinct)
            .map_err(|_| AnalysisError::Allocation)?;
        lengths.extend(
            tally
                .iter()
                .enumerate()
                .rev()
                .filter(|(_, count)| **count != 0)
                .map(|(length, count)| (length, *count)),
        );
        Ok(Cycles { lengths })
    }

    fn differential(&self) -> Result<Differential, AnalysisError> {
        let inputs = self.values.len();
        let mut row = filled(0_u32, size(self.output_bits))?;
        let mut histogram = filled(0_u64, inputs.saturating_add(1))?;
        let mut uniformity = 0_u32;
        let mut reached = 0_u64;
        let mut branch_number = u32::MAX;
        let mut absolute_indicator = 0_u32;
        let full = u32::try_from(inputs).map_err(|_| AnalysisError::Allocation)?;
        let outputs = u64::try_from(row.len()).map_err(|_| AnalysisError::Allocation)?;
        let mut zeros = 0_u64;
        for a in 1..inputs {
            difference_row(&self.values, a, &mut row);
            // Most entries of a wide function's row are zero; they are
            // counted at once, and only the others are visited.
            let mut nonzero = 0_u64;
            for (b, &count) in row.iter().enumerate() {
                if count == 0 {
                    continue;
                }
                nonzero = nonzero.saturating_add(1);
                if let Some(slot) = usize::try_from(count)
                    .ok()
                    .and_then(|count| histogram.get_mut(count))
                {
                    *slot = slot.saturating_add(1);
                }
                match count.cmp(&uniformity) {
                    std::cmp::Ordering::Greater => {
                        uniformity = count;
                        reached = 1;
                    }
                    std::cmp::Ordering::Equal => reached = reached.saturating_add(1),
                    std::cmp::Ordering::Less => {}
                }
                branch_number = branch_number.min(a.count_ones().saturating_add(b.count_ones()));
            }
            zeros = zeros.saturating_add(outputs.saturating_sub(nonzero));
            if self.output_bits == 1 {
                let changes = row.get(1).copied().unwrap_or(0);
                let correlation = full.abs_diff(changes.saturating_mul(2));
                absolute_indicator = absolute_indicator.max(correlation);
            }
        }
        if let Some(slot) = histogram.first_mut() {
            *slot = zeros;
        }
        Ok(Differential {
            uniformity,
            reached,
            spectrum: nonzero_entries(&histogram)?,
            branch_number,
            absolute_indicator: (self.output_bits == 1).then_some(absolute_indicator),
        })
    }

    /// Fills `column` with W(a, b) for every input mask a.
    fn walsh_column(&self, b: usize, column: &mut [i32]) {
        for (entry, &value) in column.iter_mut().zip(&self.values) {
            *entry = if (b & value).count_ones().is_multiple_of(2) {
                1
            } else {
                -1
            };
        }
        walsh_hadamard(column);
    }

    fn linear(&self) -> Result<Linear, AnalysisError> {
        let inputs = self.values.len();
        let mut column = filled(0_i32, inputs)?;
        let mut histogram = filled(0_u64, inputs.saturating_add(1))?;
        let mut linearity = 0_u32;
        let mut branch_number = u32::MAX;
        let mut correlated = self.input_bits.saturating_add(1);
        for b in 1..size(self.output_bits) {
            self.walsh_column(b, &mut column);
            for (a, coefficient) in column.iter().enumerate() {
                let magnitude = coefficient.unsigned_abs();
                linearity = linearity.max(magnitude);
                if let Some(slot) = usize::try_from(magnitude)
                    .ok()
                    .and_then(|magnitude| histogram.get_mut(magnitude))
                {
                    *slot = slot.saturating_add(1);
                }
                if magnitude != 0 {
                    branch_number =
                        branch_number.min(a.count_ones().saturating_add(b.count_ones()));
                    if a != 0 {
                        correlated = correlated.min(a.count_ones());
                    }
                }
            }
        }
        Ok(Linear {
            linearity,
            spectrum: nonzero_entries(&histogram)?,
            branch_number,
            correlation_immunity: correlated.saturating_sub(1),
        })
    }

    fn algebraic(&self, inverse: Option<&[usize]>) -> Result<Algebraic, AnalysisError> {
        let (degree, minimum_degree) = degrees(self.input_bits, self.output_bits, &self.values)?;
        let inverse_degree = match inverse {
            Some(inverse) => Some(degrees(self.input_bits, self.output_bits, inverse)?.0),
            None => None,
        };
        Ok(Algebraic {
            degree,
            minimum_degree,
            inverse_degree,
        })
    }

    fn equations(&self) -> Result<Equations, AnalysisError> {
        let words = self.values.len().div_ceil(64);
        let input_bits = usize::try_from(self.input_bits).map_err(|_| AnalysisError::Allocation)?;
        let output_bits =
            usize::try_from(self.output_bits).map_err(|_| AnalysisError::Allocation)?;
        let variables = input_bits.saturating_add(output_bits);
        // The column of each variable: bit x is input bit i of x, or output
        // bit j of F(x).
        let mut columns = Vec::new();
        columns
            .try_reserve_exact(variables)
            .map_err(|_| AnalysisError::Allocation)?;
        for variable in 0..variables {
            let mut column = filled(0_u64, words)?;
            for (x, &value) in self.values.iter().enumerate() {
                let bit = if variable < input_bits {
                    x.checked_shr(u32::try_from(variable).unwrap_or(u32::MAX))
                } else {
                    value.checked_shr(
                        u32::try_from(variable.saturating_sub(input_bits)).unwrap_or(u32::MAX),
                    )
                };
                if bit.unwrap_or(0) & 1 == 1 {
                    set_bit(&mut column, x);
                }
            }
            columns.push(column);
        }

        let mut ones = filled(0_u64, words)?;
        for x in 0..self.values.len() {
            set_bit(&mut ones, x);
        }
        let mut bi_affine = Basis::new(words);
        let mut quadratic = Basis::new(words);
        let mut bi_affine_monomials = 0_usize;
        let mut quadratic_monomials = 0_usize;
        let mut product = filled(0_u64, words)?;

        // Monomials of degree at most 1 belong to both spaces.
        for column in std::iter::once(&ones).chain(&columns) {
            bi_affine.insert(column)?;
            quadratic.insert(column)?;
            bi_affine_monomials = bi_affine_monomials.saturating_add(1);
            quadratic_monomials = quadratic_monomials.saturating_add(1);
        }
        for (first, left) in columns.iter().enumerate() {
            for (second, right) in columns.iter().enumerate().skip(first.saturating_add(1)) {
                for ((target, left), right) in product.iter_mut().zip(left).zip(right) {
                    *target = left & right;
                }
                quadratic.insert(&product)?;
                quadratic_monomials = quadratic_monomials.saturating_add(1);
                if first < input_bits && second >= input_bits {
                    bi_affine.insert(&product)?;
                    bi_affine_monomials = bi_affine_monomials.saturating_add(1);
                }
            }
        }
        Ok(Equations {
            quadratic: quadratic_monomials.saturating_sub(quadratic.rank()),
            bi_affine: bi_affine_monomials.saturating_sub(bi_affine.rank()),
        })
    }

    fn boomerang_uniformity(&self, inverse: &[usize]) -> Result<u32, AnalysisError> {
        let inputs = self.values.len();
        let mut row = filled(0_u32, inputs)?;
        let mut uniformity = 0_u32;
        for a in 1..inputs {
            boomerang_row(&self.values, inverse, a, &mut row);
            uniformity = row
                .iter()
                .skip(1)
                .fold(uniformity, |most, count| most.max(*count));
        }
        Ok(uniformity)
    }
}

/// Fills `row` with DDT(a, b) for every b.
///
/// The inputs x and x ^ a give the same difference, so only the x without
/// the highest bit of a are visited, each counting for both.
fn difference_row(values: &[usize], a: usize, row: &mut [u32]) {
    row.fill(0);
    if a == 0 {
        if let Some(count) = row.first_mut() {
            *count = u32::try_from(values.len()).unwrap_or(u32::MAX);
        }
        return;
    }
    let high = 1_usize
        .checked_shl(
            usize::BITS
                .saturating_sub(1)
                .saturating_sub(a.leading_zeros()),
        )
        .unwrap_or(0);
    let low = a ^ high;
    for block in values.chunks_exact(high.saturating_mul(2)) {
        let (first, second) = block.split_at(high);
        for (i, &value) in first.iter().enumerate() {
            if let Some(&other) = second.get(i ^ low)
                && let Some(count) = row.get_mut(value ^ other)
            {
                *count = count.saturating_add(2);
            }
        }
    }
}

/// Fills `row` with BCT(a, b) for every b.
fn boomerang_row(values: &[usize], inverse: &[usize], a: usize, row: &mut [u32]) {
    row.fill(0);
    let inverse_at = |y: usize| inverse.get(y).copied().unwrap_or(0);
    for (x, &first) in values.iter().enumerate() {
        let second = values.get(x ^ a).copied().unwrap_or(0);
        for (b, count) in row.iter_mut().enumerate() {
            if inverse_at(first ^ b) ^ inverse_at(second ^ b) == a {
                *count = count.saturating_add(1);
            }
        }
    }
}

/// Returns the algebraic degree of `values`, read as `output_bits` Boolean
/// functions of `input_bits` bits, and the least degree of a nonzero
/// combination of them.
fn degrees(
    input_bits: u32,
    output_bits: u32,
    values: &[usize],
) -> Result<(u32, u32), AnalysisError> {
    let inputs = values.len();
    let words = inputs.div_ceil(64);
    // Bitsets of the monomials of each weight, so that a form's degree is
    // the largest weight whose set it meets.
    let mut by_weight = Vec::new();
    by_weight
        .try_reserve_exact(
            usize::try_from(input_bits)
                .map_err(|_| AnalysisError::Allocation)?
                .saturating_add(1),
        )
        .map_err(|_| AnalysisError::Allocation)?;
    for _ in 0..=input_bits {
        by_weight.push(filled(0_u64, words)?);
    }
    for x in 0..inputs {
        if let Some(set) = usize::try_from(x.count_ones())
            .ok()
            .and_then(|weight| by_weight.get_mut(weight))
        {
            set_bit(set, x);
        }
    }

    let mut coordinates = Vec::new();
    coordinates
        .try_reserve_exact(usize::try_from(output_bits).map_err(|_| AnalysisError::Allocation)?)
        .map_err(|_| AnalysisError::Allocation)?;
    let mut coefficients = filled(0_u8, inputs)?;
    let mut degree = 0_u32;
    for bit in 0..output_bits {
        for (coefficient, value) in coefficients.iter_mut().zip(values) {
            *coefficient = u8::from(value.checked_shr(bit).unwrap_or(0) & 1 == 1);
        }
        mobius(&mut coefficients);
        let mut form = filled(0_u64, words)?;
        for (x, coefficient) in coefficients.iter().enumerate() {
            if *coefficient != 0 {
                set_bit(&mut form, x);
            }
        }
        degree = degree.max(form_degree(&form, &by_weight, input_bits).unwrap_or(0));
        coordinates.push(form);
    }

    // Visit every nonzero combination b.F in Gray-code order, so that each
    // differs from the one before by one coordinate.
    let mut component = filled(0_u64, words)?;
    let mut minimum_degree = u32::MAX;
    for step in 1..size(output_bits) {
        let changed = usize::try_from(step.trailing_zeros()).unwrap_or(0);
        if let Some(coordinate) = coordinates.get(changed) {
            for (word, coordinate) in component.iter_mut().zip(coordinate) {
                *word ^= coordinate;
            }
        }
        // A zero component has no monomial; its degree is reported as 0.
        let component_degree = form_degree(&component, &by_weight, degree).unwrap_or(0);
        minimum_degree = minimum_degree.min(component_degree);
    }
    Ok((degree, minimum_degree))
}

/// Returns the largest weight at most `highest` whose monomial set meets
/// `form`, or `None` for the zero form.
fn form_degree(form: &[u64], by_weight: &[Vec<u64>], highest: u32) -> Option<u32> {
    (0..=highest).rev().find(|weight| {
        usize::try_from(*weight)
            .ok()
            .and_then(|weight| by_weight.get(weight))
            .is_some_and(|set| set.iter().zip(form).any(|(set, form)| set & form != 0))
    })
}

/// Returns every mask of `input_bits` bits in ascending weight and, within
/// a weight, ascending value.
fn monomial_order(input_bits: u32) -> Result<Vec<Monomial>, AnalysisError> {
    let inputs = size(input_bits);
    let mut order = Vec::new();
    order
        .try_reserve_exact(inputs)
        .map_err(|_| AnalysisError::Allocation)?;
    for weight in 0..=input_bits {
        order.extend((0..inputs).filter(|mask| mask.count_ones() == weight));
    }
    Ok(order)
}

/// Replaces truth-table bits by algebraic-normal-form coefficients, in
/// place: the binary Mobius transform.
fn mobius(coefficients: &mut [u8]) {
    let mut half = 1_usize;
    while half < coefficients.len() {
        for block in coefficients.chunks_exact_mut(half.saturating_mul(2)) {
            let (low, high) = block.split_at_mut(half);
            for (high, low) in high.iter_mut().zip(low.iter()) {
                *high ^= *low;
            }
        }
        half = half.saturating_mul(2);
    }
}

/// Replaces signs (-1)^f(x) by Walsh coefficients, in place: the fast
/// Walsh-Hadamard transform.
fn walsh_hadamard(column: &mut [i32]) {
    let mut half = 1_usize;
    while half < column.len() {
        for block in column.chunks_exact_mut(half.saturating_mul(2)) {
            let (low, high) = block.split_at_mut(half);
            for (low, high) in low.iter_mut().zip(high.iter_mut()) {
                let (sum, difference) = (low.wrapping_add(*high), low.wrapping_sub(*high));
                *low = sum;
                *high = difference;
            }
        }
        half = half.saturating_mul(2);
    }
}

/// A basis of a space of bit vectors, kept in echelon form: each vector has
/// a pivot bit that every later vector lacks.
struct Basis {
    words: usize,
    vectors: Vec<(usize, Vec<u64>)>,
}

impl Basis {
    const fn new(words: usize) -> Self {
        Self {
            words,
            vectors: Vec::new(),
        }
    }

    const fn rank(&self) -> usize {
        self.vectors.len()
    }

    /// Adds `vector` to the span, growing the basis when it is independent.
    fn insert(&mut self, vector: &[u64]) -> Result<(), AnalysisError> {
        let mut reduced = filled(0_u64, self.words)?;
        reduced.copy_from_slice(vector.get(..self.words).ok_or(AnalysisError::Allocation)?);
        for (pivot, basis) in &self.vectors {
            if bit(&reduced, *pivot) {
                for (word, basis) in reduced.iter_mut().zip(basis) {
                    *word ^= basis;
                }
            }
        }
        let pivot = reduced.iter().enumerate().find_map(|(index, word)| {
            (*word != 0).then(|| {
                index
                    .saturating_mul(64)
                    .saturating_add(usize::try_from(word.trailing_zeros()).unwrap_or(0))
            })
        });
        if let Some(pivot) = pivot {
            self.vectors
                .try_reserve(1)
                .map_err(|_| AnalysisError::Allocation)?;
            self.vectors.push((pivot, reduced));
        }
        Ok(())
    }
}

fn bit(words: &[u64], index: usize) -> bool {
    words.get(index / 64).is_some_and(|word| {
        word.checked_shr(u32::try_from(index % 64).unwrap_or(0))
            .unwrap_or(0)
            & 1
            == 1
    })
}

fn set_bit(words: &mut [u64], index: usize) {
    if let Some(word) = words.get_mut(index / 64) {
        *word |= 1_u64
            .checked_shl(u32::try_from(index % 64).unwrap_or(0))
            .unwrap_or(0);
    }
}

/// Returns 2^bits for bits at most [`MAX_ANALYSIS_BITS`].
fn size(bits: u32) -> usize {
    1_usize.checked_shl(bits).unwrap_or(0)
}

fn filled<T: Clone>(value: T, length: usize) -> Result<Vec<T>, AnalysisError> {
    let mut vector = Vec::new();
    vector
        .try_reserve_exact(length)
        .map_err(|_| AnalysisError::Allocation)?;
    vector.resize(length, value);
    Ok(vector)
}

/// Returns `compute()` when `operations` is within the limit.
fn within<T>(
    operations: u64,
    compute: impl FnOnce() -> Result<T, AnalysisError>,
) -> Result<Computed<T>, AnalysisError> {
    if operations > MAX_ANALYSIS_OPERATIONS {
        Ok(Computed::TooCostly(operations))
    } else {
        compute().map(Computed::Done)
    }
}

/// Returns the (index, count) pairs of a histogram with a nonzero count.
fn nonzero_entries(histogram: &[u64]) -> Result<Vec<(u32, u64)>, AnalysisError> {
    let mut entries = Vec::new();
    let distinct = histogram.iter().filter(|count| **count != 0).count();
    entries
        .try_reserve_exact(distinct)
        .map_err(|_| AnalysisError::Allocation)?;
    for (index, &count) in histogram.iter().enumerate() {
        if count != 0 {
            entries.push((
                u32::try_from(index).map_err(|_| AnalysisError::Allocation)?,
                count,
            ));
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRESENT: [u32; 16] = [
        0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd, 0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2,
    ];

    fn parity(x: usize) -> u32 {
        x.count_ones() % 2
    }

    /// A deterministic table of `2^n` values below `2^m`.
    fn table(n: u32, m: u32, seed: u64) -> Vec<u32> {
        let mut state = seed;
        (0..1_u32 << n)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                u32::try_from(state >> 40).unwrap() & ((1 << m) - 1)
            })
            .collect()
    }

    /// A deterministic permutation of `2^n` values.
    fn permutation(n: u32, seed: u64) -> Vec<u32> {
        let mut values = (0..1_u32 << n).collect::<Vec<_>>();
        let keys = table(n, 16, seed);
        values.sort_by_key(|value| (keys[*value as usize], *value));
        values
    }

    fn aes_sbox() -> Vec<u32> {
        let times = |mut a: u32, mut b: u32| {
            let mut product = 0;
            while b != 0 {
                if b & 1 == 1 {
                    product ^= a;
                }
                a = (a << 1) ^ if a & 0x80 != 0 { 0x11b } else { 0 };
                b >>= 1;
            }
            product
        };
        (0..256)
            .map(|x| {
                let inverse = (0..256).find(|y| times(x, *y) == 1).unwrap_or(0);
                let mut b = inverse ^ 0x63;
                for shift in 1..5 {
                    b ^= ((inverse << shift) | (inverse >> (8 - shift))) & 0xff;
                }
                b
            })
            .collect()
    }

    fn naive_ddt(values: &[u32], n: u32, m: u32) -> Vec<u32> {
        let (inputs, outputs) = (1_usize << n, 1_usize << m);
        let mut ddt = vec![0; inputs * outputs];
        for a in 0..inputs {
            for x in 0..inputs {
                ddt[a * outputs + (values[x] ^ values[x ^ a]) as usize] += 1;
            }
        }
        ddt
    }

    fn naive_walsh(values: &[u32], n: u32, a: usize, b: usize) -> i32 {
        (0..1_usize << n)
            .map(|x| {
                if parity(b & values[x] as usize) == parity(a & x) {
                    1
                } else {
                    -1
                }
            })
            .sum()
    }

    fn naive_degree(values: &[u32], n: u32, mask: usize) -> Option<u32> {
        // The coefficient of monomial u is the sum of f(x) over x within u.
        (0..1_usize << n)
            .filter(|u| {
                (0..1_usize << n)
                    .filter(|x| x & u == *x)
                    .map(|x| parity(mask & values[x] as usize))
                    .sum::<u32>()
                    % 2
                    == 1
            })
            .map(|u| u.count_ones())
            .max()
    }

    /// The rank over GF(2) of rows of at most 128 bits.
    fn rank(mut rows: Vec<u128>) -> usize {
        let mut rank = 0;
        for column in 0..128 {
            let Some(pivot) = (rank..rows.len()).find(|row| rows[*row] >> column & 1 == 1) else {
                continue;
            };
            rows.swap(rank, pivot);
            for row in 0..rows.len() {
                if row != rank && rows[row] >> column & 1 == 1 {
                    rows[row] ^= rows[rank];
                }
            }
            rank += 1;
        }
        rank
    }

    /// Counts vanishing quadratic and bi-affine polynomials from the rank
    /// of the points-by-monomials matrix, row by row.
    fn naive_equations(values: &[u32], n: u32, m: u32) -> (usize, usize) {
        let variables = (n + m) as usize;
        let bit = |x: usize, v: usize| -> u128 {
            if v < n as usize {
                (x >> v & 1) as u128
            } else {
                (values[x] >> (v - n as usize) & 1) as u128
            }
        };
        let mut quadratic_rows = Vec::new();
        let mut bi_affine_rows = Vec::new();
        for x in 0..1_usize << n {
            let mut quadratic = 1_u128;
            let mut bi_affine = 1_u128;
            let mut position = 1;
            let mut affine_position = 1;
            for v in 0..variables {
                quadratic |= bit(x, v) << position;
                bi_affine |= bit(x, v) << affine_position;
                position += 1;
                affine_position += 1;
            }
            for v in 0..variables {
                for w in v + 1..variables {
                    let product = bit(x, v) & bit(x, w);
                    quadratic |= product << position;
                    position += 1;
                    if v < n as usize && w >= n as usize {
                        bi_affine |= product << affine_position;
                        affine_position += 1;
                    }
                }
            }
            quadratic_rows.push(quadratic);
            bi_affine_rows.push(bi_affine);
        }
        let quadratic_monomials = 1 + variables + variables * (variables - 1) / 2;
        let bi_affine_monomials = 1 + variables + (n * m) as usize;
        (
            quadratic_monomials - rank(quadratic_rows),
            bi_affine_monomials - rank(bi_affine_rows),
        )
    }

    fn summary(values: &[u32], n: u32, m: u32) -> Summary {
        BitFunction::new(n, m, values).unwrap().summary().unwrap()
    }

    #[test]
    fn construction_checks_widths_lengths_and_values() {
        assert!(BitFunction::new(0, 1, &[0]).is_none());
        assert!(BitFunction::new(1, 0, &[0, 0]).is_none());
        assert!(BitFunction::new(17, 1, &[]).is_none());
        assert!(BitFunction::new(1, 17, &[0, 0]).is_none());
        assert!(BitFunction::new(2, 2, &[0, 1, 2]).is_none());
        assert!(BitFunction::new(2, 2, &[0, 1, 2, 4]).is_none());
        let function = BitFunction::new(2, 2, &[0, 1, 2, 3]).unwrap();
        assert_eq!(function.values(), &[0, 1, 2, 3]);
        assert!(function.is_permutation());
        assert!(
            !BitFunction::new(2, 2, &[0, 1, 2, 2])
                .unwrap()
                .is_permutation()
        );
        assert!(
            !BitFunction::new(2, 1, &[0, 1, 1, 0])
                .unwrap()
                .is_permutation()
        );
    }

    #[test]
    fn present_sbox_matches_its_published_properties() {
        let result = summary(&PRESENT, 4, 4);
        assert_eq!(result.image, 16);
        assert_eq!(result.balanced, Some(true));
        assert_eq!(result.fixed_points, Some(0));
        assert_eq!(
            result.cycles,
            Some(Cycles {
                lengths: vec![(7, 1), (4, 1), (3, 1), (2, 1)]
            })
        );
        let differential = result.differential.done().unwrap();
        assert_eq!(differential.uniformity, 4);
        assert_eq!(differential.reached, 24);
        assert_eq!(differential.spectrum, vec![(0, 144), (2, 72), (4, 24)]);
        assert_eq!(differential.branch_number, 3);
        let linear = result.linear.done().unwrap();
        assert_eq!(linear.linearity, 8);
        assert_eq!(linear.spectrum, vec![(0, 108), (4, 96), (8, 36)]);
        assert_eq!(linear.branch_number, 2);
        let algebraic = result.algebraic.done().unwrap();
        assert_eq!(
            (
                algebraic.degree,
                algebraic.minimum_degree,
                algebraic.inverse_degree
            ),
            (3, 2, Some(3))
        );
        assert_eq!(
            result.equations.done(),
            Some(&Equations {
                quadratic: 21,
                bi_affine: 9
            })
        );
        assert_eq!(result.boomerang_uniformity, Some(Computed::Done(16)));
    }

    #[test]
    fn aes_sbox_matches_its_published_properties() {
        let values = aes_sbox();
        assert_eq!(&values[..4], &[0x63, 0x7c, 0x77, 0x7b]);
        let result = summary(&values, 8, 8);
        assert_eq!(result.fixed_points, Some(0));
        assert_eq!(
            result.cycles.unwrap().lengths,
            vec![(87, 1), (81, 1), (59, 1), (27, 1), (2, 1)]
        );
        let differential = result.differential.done().unwrap();
        assert_eq!(
            differential.spectrum,
            vec![(0, 32_895), (2, 32_130), (4, 255)]
        );
        let linear = result.linear.done().unwrap();
        assert_eq!(linear.linearity, 32);
        let algebraic = result.algebraic.done().unwrap();
        assert_eq!(
            (
                algebraic.degree,
                algebraic.minimum_degree,
                algebraic.inverse_degree
            ),
            (7, 7, Some(7))
        );
        assert_eq!(
            result.equations.done(),
            Some(&Equations {
                quadratic: 39,
                bi_affine: 23
            })
        );
        assert_eq!(result.boomerang_uniformity, Some(Computed::Done(6)));
    }

    #[test]
    fn summaries_agree_with_definitions_on_many_functions() {
        let shapes = [
            (1, 1),
            (2, 3),
            (3, 1),
            (3, 3),
            (4, 2),
            (4, 4),
            (5, 3),
            (5, 5),
            (6, 4),
        ];
        for (seed, (n, m)) in shapes.into_iter().enumerate() {
            for variant in 0..3_u64 {
                let values = if n == m && variant == 0 {
                    permutation(n, seed as u64)
                } else {
                    table(n, m, seed as u64 * 7 + variant)
                };
                check_against_definitions(&values, n, m);
            }
        }
    }

    fn check_against_definitions(values: &[u32], n: u32, m: u32) {
        let (inputs, outputs) = (1_usize << n, 1_usize << m);
        let function = BitFunction::new(n, m, values).unwrap();
        let result = function.summary().unwrap();

        let ddt = naive_ddt(values, n, m);
        assert_eq!(function.difference_table().unwrap(), ddt);
        let rows = &ddt[outputs..];
        let differential = result.differential.done().unwrap();
        assert_eq!(differential.uniformity, *rows.iter().max().unwrap());
        let mut spectrum = std::collections::BTreeMap::new();
        for count in rows {
            *spectrum.entry(*count).or_insert(0_u64) += 1;
        }
        assert_eq!(
            differential.spectrum,
            spectrum.into_iter().collect::<Vec<_>>()
        );
        let branch = (1..inputs)
            .flat_map(|a| (0..outputs).map(move |b| (a, b)))
            .filter(|(a, b)| ddt[a * outputs + b] != 0)
            .map(|(a, b)| a.count_ones() + b.count_ones())
            .min()
            .unwrap();
        assert_eq!(differential.branch_number, branch);

        let lat = function.linear_table().unwrap();
        let mut linearity = 0;
        let mut correlated = n + 1;
        for a in 0..inputs {
            for b in 0..outputs {
                let walsh = naive_walsh(values, n, a, b);
                assert_eq!(lat[a * outputs + b] * 2, walsh);
                let ones = (0..inputs)
                    .filter(|x| parity(a & x) == parity(b & values[*x] as usize))
                    .count();
                assert_eq!(lat[a * outputs + b], ones as i32 - (inputs / 2) as i32);
                if b != 0 {
                    linearity = linearity.max(walsh.unsigned_abs());
                    if a != 0 && walsh != 0 {
                        correlated = correlated.min(a.count_ones());
                    }
                }
            }
        }
        let linear = result.linear.done().unwrap();
        assert_eq!(linear.linearity, linearity);
        assert_eq!(linear.correlation_immunity, correlated - 1);

        let degrees = (1..outputs)
            .map(|mask| naive_degree(values, n, mask).unwrap_or(0))
            .collect::<Vec<_>>();
        let coordinates = (0..m)
            .map(|bit| naive_degree(values, n, 1 << bit).unwrap_or(0))
            .max()
            .unwrap();
        let algebraic = result.algebraic.done().unwrap();
        assert_eq!(algebraic.degree, coordinates);
        assert_eq!(algebraic.degree, *degrees.iter().max().unwrap());
        assert_eq!(algebraic.minimum_degree, *degrees.iter().min().unwrap());

        // The normal form, evaluated at every input, gives the table back.
        let forms = function.normal_form().unwrap();
        for (x, value) in values.iter().enumerate() {
            let evaluated = forms.iter().enumerate().fold(0, |sum, (bit, form)| {
                let ones = form
                    .iter()
                    .filter(|monomial| x & **monomial == **monomial)
                    .count();
                sum | ((ones as u32 % 2) << bit)
            });
            assert_eq!(evaluated, *value);
        }
        for form in &forms {
            assert!(
                form.windows(2).all(|pair| {
                    (pair[0].count_ones(), pair[0]) < (pair[1].count_ones(), pair[1])
                })
            );
        }

        let (quadratic, bi_affine) = naive_equations(values, n, m);
        assert_eq!(
            result.equations.done(),
            Some(&Equations {
                quadratic,
                bi_affine
            })
        );

        let image = values
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        assert_eq!(result.image, image);
        assert_eq!(function.is_permutation(), n == m && image == inputs);
        if function.is_permutation() {
            let mut inverse = vec![0; inputs];
            for (x, value) in values.iter().enumerate() {
                inverse[*value as usize] = x;
            }
            let bct = function.boomerang_table().unwrap();
            let mut uniformity = 0;
            for a in 0..inputs {
                for b in 0..inputs {
                    let count = (0..inputs)
                        .filter(|x| {
                            inverse[values[*x] as usize ^ b] ^ inverse[values[x ^ a] as usize ^ b]
                                == a
                        })
                        .count() as u32;
                    assert_eq!(bct[a * inputs + b], count);
                    if a != 0 && b != 0 {
                        uniformity = uniformity.max(count);
                    }
                }
            }
            assert_eq!(
                result.boomerang_uniformity,
                Some(Computed::Done(uniformity))
            );
            let cycles = result.cycles.unwrap();
            assert_eq!(
                cycles
                    .lengths
                    .iter()
                    .map(|(length, count)| length * count)
                    .sum::<usize>(),
                inputs
            );
            assert_eq!(
                result.fixed_points,
                Some(
                    cycles
                        .lengths
                        .iter()
                        .find(|(l, _)| *l == 1)
                        .map_or(0, |(_, c)| *c)
                )
            );
        } else {
            assert_eq!(
                function.boomerang_table(),
                Err(AnalysisError::NotPermutation)
            );
            assert_eq!(result.boomerang_uniformity, None);
            assert_eq!(result.cycles, None);
        }
        if m == 1 {
            let indicator = (1..inputs)
                .map(|a| {
                    (0..inputs)
                        .map(|x| {
                            if values[x] == values[x ^ a] {
                                1_i32
                            } else {
                                -1
                            }
                        })
                        .sum::<i32>()
                        .unsigned_abs()
                })
                .max()
                .unwrap();
            assert_eq!(differential.absolute_indicator, Some(indicator));
            assert_eq!(
                result.weight,
                Some(values.iter().filter(|v| **v == 1).count())
            );
        } else {
            assert_eq!(differential.absolute_indicator, None);
            assert_eq!(result.weight, None);
        }
    }

    #[test]
    fn affine_and_constant_functions_reach_the_extremes() {
        // x -> x ^ 5 is affine: every difference passes with certainty and
        // every component is a linear function of the input.
        let affine = (0..16).map(|x| x ^ 5).collect::<Vec<_>>();
        let result = summary(&affine, 4, 4);
        assert_eq!(result.differential.done().unwrap().uniformity, 16);
        assert_eq!(result.linear.done().unwrap().linearity, 16);
        let algebraic = result.algebraic.done().unwrap();
        assert_eq!((algebraic.degree, algebraic.minimum_degree), (1, 1));
        assert_eq!(
            result.cycles.unwrap().lengths,
            vec![(2, 8)],
            "x ^ 5 pairs every input with another"
        );

        let constant = vec![3; 8];
        let result = summary(&constant, 3, 2);
        assert_eq!(result.image, 1);
        assert_eq!(result.balanced, Some(false));
        let algebraic = result.algebraic.done().unwrap();
        assert_eq!((algebraic.degree, algebraic.minimum_degree), (0, 0));
        assert_eq!(result.linear.done().unwrap().correlation_immunity, 3);
        assert_eq!(
            function_forms(&constant, 3, 2),
            vec![vec![0], vec![0]],
            "both output bits are the constant 1"
        );
    }

    fn function_forms(values: &[u32], n: u32, m: u32) -> Vec<Vec<Monomial>> {
        BitFunction::new(n, m, values)
            .unwrap()
            .normal_form()
            .unwrap()
    }

    #[test]
    fn wide_functions_skip_costly_properties_and_refuse_tables() {
        let values = table(16, 16, 11);
        let function = BitFunction::new(16, 16, &values).unwrap();
        assert_eq!(
            function.difference_table(),
            Err(AnalysisError::TableTooLarge)
        );
        assert_eq!(function.linear_table(), Err(AnalysisError::TableTooLarge));
        assert_eq!(function.normal_form(), Err(AnalysisError::TableTooLarge));
        assert!(
            BitFunction::new(11, 1, &table(11, 1, 3))
                .unwrap()
                .linear_table()
                .is_err()
        );

        // A property over the limit is reported with its cost, uncomputed.
        assert_eq!(
            within(1 << 36, || -> Result<(), AnalysisError> { unreachable!() }),
            Ok(Computed::TooCostly(1 << 36))
        );
        assert_eq!(within(1 << 32, || Ok(7)), Ok(Computed::Done(7)));
    }

    #[test]
    fn difference_rows_count_both_members_of_each_pair() {
        let values = PRESENT.iter().map(|v| *v as usize).collect::<Vec<_>>();
        let mut row = vec![0; 16];
        for a in 0..16 {
            difference_row(&values, a, &mut row);
            assert_eq!(row.iter().sum::<u32>(), 16);
            let naive = naive_ddt(&PRESENT, 4, 4);
            assert_eq!(row, naive[a * 16..a * 16 + 16]);
        }
    }

    #[test]
    fn analysis_errors_read_as_sentences() {
        assert_eq!(
            AnalysisError::TableTooLarge.to_string(),
            "complete tables are limited to 10 input and 10 output bits"
        );
        assert_eq!(
            AnalysisError::NotPermutation.to_string(),
            "the boomerang connectivity table needs a permutation"
        );
        assert_eq!(
            AnalysisError::Allocation.to_string(),
            "analysis storage could not be reserved"
        );
    }
}
