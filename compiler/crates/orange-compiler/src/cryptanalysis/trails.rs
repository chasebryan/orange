//! Bounds on the trails of a substitution-permutation network.
//!
//! A [`Network`] is the round of a key-alternating substitution-permutation
//! network on n <= 128 bits: k = n / s copies of one s-bit S-box side by
//! side, word c holding bits c s through c s + s - 1, then a linear layer M
//! over GF(2). A round key added between rounds changes no difference and
//! no mask, so it does not appear.
//!
//! A differential trail over r rounds is a sequence of differences
//! a_1 -> b_1, a_2 -> b_2, ..., a_r -> b_r with a_1 != 0, a_(i+1) = M b_i,
//! and `DDT(a_i[c], b_i[c]) != 0` at every word c. A linear trail is a
//! sequence of masks with `W(a_i[c], b_i[c]) != 0` at every word c and
//! a_(i+1) = (M^-1)^T b_i, since a . (M z) = (M^T a) . z. An S-box is
//! active in round i when its word of a_i is nonzero. One S-box's
//! transition has weight s - log2 DDT(a, b), for a probability of
//! 2^-weight, or s - log2 |W(a, b)|, for a correlation of plus or minus
//! 2^-weight, and the weight of a trail is the sum over its active S-boxes.
//!
//! For each number of rounds r this module finds, by complete search, the
//! least number of active S-boxes and the least weight of any r-round
//! trail. Both bound single trails. They say nothing of a differential or
//! a linear hull, which collects many trails, nor of the security of any
//! cipher.

use super::linear::LinearMap;
use super::{AnalysisError, BitFunction, Computed, filled, size};

/// Most rounds for which trails are bounded.
pub const MAX_TRAIL_ROUNDS: u32 = 32;

/// Most steps one search for the least costs of trails may take. A step of
/// a search does more work than an elementary operation of the other
/// analyses, so its limit is lower than [`super::MAX_ANALYSIS_OPERATIONS`].
pub const MAX_TRAIL_STEPS: u64 = 1 << 28;

/// Widest S-box of a network.
pub const MAX_SBOX_BITS: u32 = 8;

/// Most S-boxes of a network: 128 bits of 2-bit S-boxes.
const MAX_SBOXES: usize = 64;

/// One round of a substitution-permutation network, with the tables its
/// trail searches need.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Network {
    sbox_bits: u32,
    sboxes: u32,
    bits: u32,
    /// DDT(a, b) at index a 2^s + b.
    differences: Vec<u32>,
    /// |W(a, b)| at index a 2^s + b.
    correlations: Vec<u32>,
    /// M applied to each value v placed at each word c, at index c 2^s + v.
    forward: Vec<u128>,
    /// (M^-1)^T, the map of masks, likewise.
    backward: Vec<u128>,
    /// The differential branch number of M over s-bit words, or 0 when it
    /// is not computed.
    differential_branch: u32,
    /// The linear branch number of M over s-bit words, or 0 when it is not
    /// computed.
    linear_branch: u32,
    /// Bit j of row i is set when output bit i of a round depends on input
    /// bit j.
    dependence: Vec<u128>,
}

/// The least cost of a trail over each number of rounds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrailBounds {
    /// Entry r - 1: the least number of active S-boxes in an r-round trail.
    pub active: Vec<Computed<u32>>,
    /// Entry r - 1: the least weight of an r-round trail. `None` when some
    /// nonzero entry of the S-box's table is not a power of two, so that
    /// weights would not be integers.
    pub weight: Option<Vec<Computed<u32>>>,
}

impl Network {
    /// Returns the round of `sbox` on each s-bit word of `layer`'s bits,
    /// then `layer`, or `None` unless the S-box is a permutation of
    /// 2 <= s <= [`MAX_SBOX_BITS`] bits, s divides the layer's width, and
    /// the layer is invertible. The layer's own word width is not used.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when the tables cannot be
    /// reserved.
    pub fn new(sbox: &BitFunction, layer: &LinearMap) -> Result<Option<Self>, AnalysisError> {
        let sbox_bits = sbox.input_bits();
        let bits = layer.bits();
        if !(2..=MAX_SBOX_BITS).contains(&sbox_bits)
            || !sbox.is_permutation()
            || !bits.is_multiple_of(sbox_bits)
        {
            return Ok(None);
        }
        let Some(inverse) = layer.inverse()? else {
            return Ok(None);
        };
        let differences = sbox.difference_table()?;
        let walsh = sbox.linear_table()?;
        let mut correlations = filled(0_u32, walsh.len())?;
        for (magnitude, entry) in correlations.iter_mut().zip(&walsh) {
            *magnitude = entry.unsigned_abs().saturating_mul(2);
        }
        let (differential_branch, linear_branch) =
            match LinearMap::new(bits, sbox_bits, layer.columns()) {
                Some(grouped) => {
                    let (differential, linear) = grouped.branch_numbers()?;
                    (
                        differential.done().copied().unwrap_or(0),
                        linear.done().copied().unwrap_or(0),
                    )
                }
                None => (0, 0),
            };
        Ok(Some(Self {
            sbox_bits,
            sboxes: bits.checked_div(sbox_bits).unwrap_or(0),
            bits,
            differences,
            correlations,
            forward: word_images(layer, sbox_bits)?,
            backward: word_images(&inverse.transpose()?, sbox_bits)?,
            differential_branch,
            linear_branch,
            dependence: dependence(sbox, layer)?,
        }))
    }

    /// Returns s, the bits of the S-box.
    #[must_use]
    pub const fn sbox_bits(&self) -> u32 {
        self.sbox_bits
    }

    /// Returns k, the number of S-boxes in a round.
    #[must_use]
    pub const fn sboxes(&self) -> u32 {
        self.sboxes
    }

    /// Returns n, the bits of the state.
    #[must_use]
    pub const fn bits(&self) -> u32 {
        self.bits
    }

    /// Returns the least number of active S-boxes and the least weight of a
    /// differential trail over each number of rounds from 1 through
    /// `rounds`, at most [`MAX_TRAIL_ROUNDS`].
    ///
    /// Each search stops once it has taken more than [`MAX_TRAIL_STEPS`]
    /// steps; that round and every later one is then
    /// [`Computed::TooCostly`] with the steps taken.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when working storage cannot be
    /// reserved.
    pub fn differential(&self, rounds: u32) -> Result<TrailBounds, AnalysisError> {
        self.bounds(
            &self.differences,
            &self.forward,
            self.differential_branch,
            rounds,
            MAX_TRAIL_STEPS,
        )
    }

    /// Returns the least number of active S-boxes and the least weight of a
    /// linear trail over each number of rounds from 1 through `rounds`, as
    /// [`Network::differential`] does for differences.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when working storage cannot be
    /// reserved.
    pub fn linear(&self, rounds: u32) -> Result<TrailBounds, AnalysisError> {
        self.bounds(
            &self.correlations,
            &self.backward,
            self.linear_branch,
            rounds,
            MAX_TRAIL_STEPS,
        )
    }

    /// Returns the least number of rounds after which every output bit
    /// depends on every input bit, or `None` when no number does.
    ///
    /// Output bit i of a round depends on input bit j when some S-box
    /// output bit p that M adds into bit i is one whose value changes with
    /// bit j at some input. Over r rounds the dependence is the r-th
    /// Boolean power D^r of that n x n relation. No row or column of D is
    /// empty, since the S-box is a permutation and M is invertible, so once
    /// a power is all ones every later one is too; and if any power is, one
    /// at most (n - 1)^2 + 1 is, by Wielandt's bound. The least such power
    /// is found by repeated squaring and a binary search.
    ///
    /// # Errors
    ///
    /// Returns [`AnalysisError::Allocation`] when working storage cannot be
    /// reserved.
    pub fn full_diffusion(&self) -> Result<Option<u32>, AnalysisError> {
        let full = low_mask(self.bits);
        let is_full = |power: &[u128]| power.iter().all(|row| *row == full);
        let wielandt = self
            .bits
            .saturating_sub(1)
            .saturating_mul(self.bits.saturating_sub(1))
            .saturating_add(1);
        // 2^levels is the least power of two at least Wielandt's bound.
        let levels = u32::BITS.saturating_sub(wielandt.saturating_sub(1).leading_zeros());
        let mut powers = Vec::new();
        powers
            .try_reserve_exact(usize::try_from(levels).unwrap_or(0).saturating_add(1))
            .map_err(|_| AnalysisError::Allocation)?;
        powers.push(self.dependence.clone());
        for _ in 0..levels {
            let square = match powers.last() {
                Some(power) => product(power, power)?,
                None => return Err(AnalysisError::Allocation),
            };
            powers.push(square);
        }
        if !powers.last().is_some_and(|power| is_full(power)) {
            return Ok(None);
        }
        // `reached` is D^rounds, the largest power found not all ones.
        let mut reached: Option<Vec<u128>> = None;
        let mut rounds = 0_u32;
        for (level, power) in powers
            .iter()
            .enumerate()
            .take(powers.len().saturating_sub(1))
            .rev()
        {
            let candidate = match &reached {
                Some(reached) => product(reached, power)?,
                None => power.clone(),
            };
            if !is_full(&candidate) {
                reached = Some(candidate);
                let step = u32::try_from(level)
                    .ok()
                    .and_then(|level| 1_u32.checked_shl(level))
                    .unwrap_or(0);
                rounds = rounds.saturating_add(step);
            }
        }
        Ok(Some(rounds.saturating_add(1)))
    }

    /// Returns the bounds for the transitions of `table`, with `images` the
    /// map from one round's S-box outputs to the next round's inputs.
    fn bounds(
        &self,
        table: &[u32],
        images: &[u128],
        branch: u32,
        rounds: u32,
        budget: u64,
    ) -> Result<TrailBounds, AnalysisError> {
        let rounds = rounds.min(MAX_TRAIL_ROUNDS);
        let search = |transitions: &Transitions| {
            least_costs(
                Search {
                    transitions,
                    images,
                    sbox_bits: self.sbox_bits,
                    sboxes: usize::try_from(self.sboxes).unwrap_or(0),
                    branch,
                    bounds: Vec::new(),
                    rounds: 0,
                    target: 0,
                    steps: 0,
                    budget,
                },
                rounds,
            )
        };
        let active = match Transitions::new(table, self.sbox_bits, false)? {
            Some(transitions) => search(&transitions)?,
            None => return Err(AnalysisError::Allocation),
        };
        let weight = match Transitions::new(table, self.sbox_bits, true)? {
            Some(transitions) => Some(search(&transitions)?),
            None => None,
        };
        Ok(TrailBounds { active, weight })
    }
}

/// The transitions of one S-box and their costs.
struct Transitions {
    /// The transitions out of a are `entries[starts[a]..starts[a + 1]]`,
    /// each (cost, b), cheapest first.
    starts: Vec<usize>,
    entries: Vec<(u32, usize)>,
    /// The cost of the cheapest transition out of each value, 0 for 0.
    least_out: Vec<u32>,
    /// Each b != 0 with the cost of the cheapest transition into it,
    /// cheapest first.
    into: Vec<(u32, usize)>,
    /// The cost of the cheapest transition of all.
    least: u32,
}

impl Transitions {
    /// Returns the transitions a -> b with a != 0 and a nonzero entry of
    /// `table`, each costing 1, or with `weighted` costing s - log2 of its
    /// entry; `None` with `weighted` when some entry is not a power of two.
    fn new(table: &[u32], sbox_bits: u32, weighted: bool) -> Result<Option<Self>, AnalysisError> {
        let values = size(sbox_bits);
        let count = table
            .iter()
            .skip(values)
            .filter(|entry| **entry != 0)
            .count();
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(count)
            .map_err(|_| AnalysisError::Allocation)?;
        let mut starts = filled(0_usize, values.saturating_add(1))?;
        let mut least_out = filled(0_u32, values)?;
        let mut least_in = filled(u32::MAX, values)?;
        for (a, row) in table.chunks_exact(values).enumerate().skip(1) {
            let first = entries.len();
            if let Some(start) = starts.get_mut(a) {
                *start = first;
            }
            for (b, &entry) in row.iter().enumerate() {
                if entry == 0 {
                    continue;
                }
                let cost = if !weighted {
                    1
                } else if entry.is_power_of_two() {
                    sbox_bits.saturating_sub(entry.trailing_zeros())
                } else {
                    return Ok(None);
                };
                entries.push((cost, b));
                if let Some(least) = least_in.get_mut(b) {
                    *least = (*least).min(cost);
                }
            }
            let row = entries.get_mut(first..).unwrap_or_default();
            row.sort_unstable();
            if let (Some(least), Some((cost, _))) = (least_out.get_mut(a), row.first()) {
                *least = *cost;
            }
        }
        if let Some(end) = starts.last_mut() {
            *end = entries.len();
        }
        let mut into = Vec::new();
        into.try_reserve_exact(values)
            .map_err(|_| AnalysisError::Allocation)?;
        for (b, &cost) in least_in.iter().enumerate().skip(1) {
            if cost != u32::MAX {
                into.push((cost, b));
            }
        }
        into.sort_unstable();
        let least = least_out.iter().skip(1).copied().min().unwrap_or(0);
        Ok(Some(Self {
            starts,
            entries,
            least_out,
            into,
            least,
        }))
    }

    /// Returns the transitions out of a, cheapest first.
    fn row(&self, a: usize) -> &[(u32, usize)] {
        match (self.starts.get(a), self.starts.get(a.saturating_add(1))) {
            (Some(&start), Some(&end)) => self.entries.get(start..end).unwrap_or_default(),
            _ => &[],
        }
    }

    /// Returns the cost of the cheapest transition out of a.
    fn least_out(&self, a: usize) -> u32 {
        self.least_out.get(a).copied().unwrap_or(0)
    }
}

/// The search ran past its budget.
struct OverBudget;

/// A search for a trail of `rounds` rounds costing at most `target`.
struct Search<'a> {
    transitions: &'a Transitions,
    images: &'a [u128],
    sbox_bits: u32,
    sboxes: usize,
    /// The branch number of the map between rounds, or 0.
    branch: u32,
    /// Entry i, for i < `rounds`: the least cost of an i-round trail.
    bounds: Vec<u32>,
    rounds: usize,
    target: u32,
    steps: u64,
    budget: u64,
}

/// Returns the least cost of a trail over 1 through `rounds` rounds, as
/// Matsui's search for the best trail finds them.
///
/// The least cost of one round, B(1), is the cheapest transition. For
/// r >= 2 the search tries each target T from B(r - 1) + B(1) upward, since
/// an r-round trail is an (r - 1)-round trail followed by one more round
/// with a nonzero input, and stops at the first T that some trail meets.
/// Under a target it builds trails round by round:
///
/// - round 1 is chosen by its output b_1 != 0, one active word at a time,
///   at the cost of the cheapest transition into each word; since the
///   S-box is a permutation, every nonzero word has one;
/// - each middle round tries the transitions of each active word of its
///   input, cheapest first;
/// - the last round costs the cheapest transition out of each active word.
///
/// A partial trail is abandoned once its cost, the cheapest transitions of
/// its round's remaining active words, and the least cost of the rounds
/// after it exceed T. The rounds after round i cost at least B(r - i), and
/// at least B(1) for each active word of round i + 1 plus B(r - i - 1);
/// round i + 1 has at least max(1, β - w) active words when round i has w,
/// for the branch number β of the map between rounds. While round 1 is
/// still being chosen, each further word costs at least B(1) and lowers
/// that bound by at most B(1), so abandoning the partial choice is sound.
///
/// Every step counts toward the budget. Once a search exceeds it, that
/// round and every later one is reported as not computed.
fn least_costs(mut search: Search<'_>, rounds: u32) -> Result<Vec<Computed<u32>>, AnalysisError> {
    let rounds = usize::try_from(rounds).map_err(|_| AnalysisError::Allocation)?;
    let mut costs = Vec::new();
    costs
        .try_reserve_exact(rounds)
        .map_err(|_| AnalysisError::Allocation)?;
    if rounds == 0 {
        return Ok(costs);
    }
    let least = search.transitions.least;
    costs.push(Computed::Done(least));
    search
        .bounds
        .try_reserve_exact(rounds)
        .map_err(|_| AnalysisError::Allocation)?;
    search.bounds.push(0);
    search.bounds.push(least);
    for length in 2..=rounds {
        search.rounds = length;
        search.target = search.bound(length.saturating_sub(1)).saturating_add(least);
        loop {
            match search.find() {
                Ok(true) => {
                    costs.push(Computed::Done(search.target));
                    search.bounds.push(search.target);
                    break;
                }
                Ok(false) => search.target = search.target.saturating_add(1),
                Err(OverBudget) => {
                    while costs.len() < rounds {
                        costs.push(Computed::TooCostly(search.steps));
                    }
                    return Ok(costs);
                }
            }
        }
    }
    Ok(costs)
}

impl Search<'_> {
    /// Returns whether some trail of `rounds` rounds costs at most
    /// `target`.
    fn find(&mut self) -> Result<bool, OverBudget> {
        self.tick()?;
        self.start(0, 0, 0, 0)
    }

    /// Chooses the output of round 1 one active word at a time: a value at
    /// a word at or after `from`, after `chosen` words costing `spent`
    /// whose image is `image`.
    fn start(
        &mut self,
        from: usize,
        chosen: u32,
        spent: u32,
        image: u128,
    ) -> Result<bool, OverBudget> {
        let transitions = self.transitions;
        let chosen = chosen.saturating_add(1);
        let after = self.after(self.rounds.saturating_sub(1), chosen);
        for word in from..self.sboxes {
            for &(cost, value) in &transitions.into {
                let total = spent.saturating_add(cost);
                if total.saturating_add(after) > self.target {
                    break;
                }
                self.tick()?;
                let next = image ^ self.image(word, value);
                if self.round(2, next, total)?
                    || self.start(word.saturating_add(1), chosen, total, next)?
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Continues a trail whose earlier rounds cost `spent` with round
    /// `round`, whose input is `input`.
    fn round(&mut self, round: usize, input: u128, spent: u32) -> Result<bool, OverBudget> {
        self.tick()?;
        let mask = low_mask(self.sbox_bits);
        let mut active = [(0_usize, 0_usize); MAX_SBOXES];
        let mut count = 0_usize;
        let mut here = 0_u32;
        for word in 0..self.sboxes {
            let shift = u32::try_from(word)
                .ok()
                .and_then(|word| word.checked_mul(self.sbox_bits))
                .unwrap_or(u32::MAX);
            let value = input.checked_shr(shift).unwrap_or(0) & mask;
            if value != 0 {
                let value = usize::try_from(value).unwrap_or(0);
                if let Some(slot) = active.get_mut(count) {
                    *slot = (word, value);
                }
                count = count.saturating_add(1);
                here = here.saturating_add(self.transitions.least_out(value));
            }
        }
        let total = spent.saturating_add(here);
        if round >= self.rounds {
            return Ok(total <= self.target);
        }
        let after = self.after(
            self.rounds.saturating_sub(round),
            u32::try_from(count).unwrap_or(u32::MAX),
        );
        if total.saturating_add(after) > self.target {
            return Ok(false);
        }
        let active = active.get(..count).unwrap_or_default();
        // suffix[i]: the cheapest transitions of active words i onward.
        let mut suffix = [0_u32; MAX_SBOXES + 1];
        for (index, &(_, value)) in active.iter().enumerate().rev() {
            let rest = suffix
                .get(index.saturating_add(1))
                .copied()
                .unwrap_or(0)
                .saturating_add(self.transitions.least_out(value));
            if let Some(entry) = suffix.get_mut(index) {
                *entry = rest;
            }
        }
        self.choose(
            &Round {
                round,
                active,
                suffix: &suffix,
                after,
            },
            0,
            spent,
            0,
        )
    }

    /// Chooses the output of active word `index` of a middle round onward,
    /// the round so far costing `spent` with outputs whose image is
    /// `image`.
    fn choose(
        &mut self,
        round: &Round<'_>,
        index: usize,
        spent: u32,
        image: u128,
    ) -> Result<bool, OverBudget> {
        let Some(&(word, value)) = round.active.get(index) else {
            return self.round(round.round.saturating_add(1), image, spent);
        };
        let transitions = self.transitions;
        let rest = round
            .suffix
            .get(index.saturating_add(1))
            .copied()
            .unwrap_or(0)
            .saturating_add(round.after);
        for &(cost, output) in transitions.row(value) {
            let total = spent.saturating_add(cost);
            if total.saturating_add(rest) > self.target {
                break;
            }
            self.tick()?;
            if self.choose(
                round,
                index.saturating_add(1),
                total,
                image ^ self.image(word, output),
            )? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns a lower bound on the cost of `rounds` further rounds when
    /// the round before them has `active` active words.
    fn after(&self, rounds: usize, active: u32) -> u32 {
        if rounds == 0 {
            return 0;
        }
        let next = self.branch.saturating_sub(active).max(1);
        let spread = self
            .transitions
            .least
            .saturating_mul(next)
            .saturating_add(self.bound(rounds.saturating_sub(1)));
        self.bound(rounds).max(spread)
    }

    /// Returns the least cost of a trail of `rounds` rounds, for rounds
    /// already searched.
    fn bound(&self, rounds: usize) -> u32 {
        self.bounds.get(rounds).copied().unwrap_or(0)
    }

    /// Returns the image of `value` placed at `word`.
    fn image(&self, word: usize, value: usize) -> u128 {
        let index = word
            .checked_shl(self.sbox_bits)
            .unwrap_or(usize::MAX)
            .saturating_add(value);
        self.images.get(index).copied().unwrap_or(0)
    }

    fn tick(&mut self) -> Result<(), OverBudget> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.budget {
            Err(OverBudget)
        } else {
            Ok(())
        }
    }
}

/// A middle round being chosen: its active words (word, value), the
/// cheapest transitions of each suffix of them, and the bound on the
/// rounds after it.
struct Round<'a> {
    round: usize,
    active: &'a [(usize, usize)],
    suffix: &'a [u32],
    after: u32,
}

/// Returns, for each word c and each value v, the image under `map` of v
/// placed at word c, at index c 2^s + v.
fn word_images(map: &LinearMap, sbox_bits: u32) -> Result<Vec<u128>, AnalysisError> {
    let values = size(sbox_bits);
    let words = usize::try_from(map.bits().checked_div(sbox_bits).unwrap_or(0))
        .map_err(|_| AnalysisError::Allocation)?;
    let mut images = filled(0_u128, values.saturating_mul(words))?;
    for (word, chunk) in images.chunks_exact_mut(values).enumerate() {
        let shift = u32::try_from(word)
            .ok()
            .and_then(|word| word.checked_mul(sbox_bits))
            .unwrap_or(u32::MAX);
        for (value, image) in chunk.iter_mut().enumerate() {
            let placed = u128::try_from(value)
                .ok()
                .and_then(|value| value.checked_shl(shift))
                .unwrap_or(0);
            *image = map.apply(placed);
        }
    }
    Ok(images)
}

/// Returns the rows of the dependence relation D of one round: bit j of
/// row i is set when output bit i depends on input bit j.
fn dependence(sbox: &BitFunction, layer: &LinearMap) -> Result<Vec<u128>, AnalysisError> {
    let sbox_bits = sbox.input_bits();
    let values = sbox.values();
    // on[t]: the S-box input bits whose flip changes output bit t at some
    // input.
    let mut on = [0_u128; 8];
    for (x, &y) in values.iter().enumerate() {
        for input in 0..sbox_bits {
            let flipped = 1_usize
                .checked_shl(input)
                .and_then(|flip| values.get(x ^ flip))
                .copied()
                .unwrap_or(y);
            let changed = y ^ flipped;
            for (output, bits) in on.iter_mut().enumerate() {
                if changed
                    .checked_shr(u32::try_from(output).unwrap_or(u32::MAX))
                    .unwrap_or(0)
                    & 1
                    == 1
                {
                    *bits |= 1_u128.checked_shl(input).unwrap_or(0);
                }
            }
        }
    }
    let rows = layer.rows()?;
    let mut relation = filled(0_u128, rows.len())?;
    for (row, dependence) in rows.iter().zip(relation.iter_mut()) {
        let mut rest = *row;
        while rest != 0 {
            let bit = rest.trailing_zeros();
            let within = bit.checked_rem(sbox_bits).unwrap_or(0);
            let bits = on
                .get(usize::try_from(within).unwrap_or(usize::MAX))
                .copied()
                .unwrap_or(0);
            *dependence |= bits.checked_shl(bit.saturating_sub(within)).unwrap_or(0);
            rest &= rest.wrapping_sub(1);
        }
    }
    Ok(relation)
}

/// Returns the Boolean product of two relations given by rows: row i of
/// the product is the union of the rows of `right` that row i of `left`
/// selects.
fn product(left: &[u128], right: &[u128]) -> Result<Vec<u128>, AnalysisError> {
    let mut rows = filled(0_u128, left.len())?;
    for (row, selector) in rows.iter_mut().zip(left) {
        let mut rest = *selector;
        while rest != 0 {
            let index = usize::try_from(rest.trailing_zeros()).unwrap_or(usize::MAX);
            *row |= right.get(index).copied().unwrap_or(0);
            rest &= rest.wrapping_sub(1);
        }
    }
    Ok(rows)
}

/// Returns the low `bits` bits set.
fn low_mask(bits: u32) -> u128 {
    1_u128
        .checked_shl(bits)
        .map_or(u128::MAX, |power| power.wrapping_sub(1))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::arithmetic_side_effects,
        clippy::as_conversions,
        clippy::indexing_slicing,
        clippy::unwrap_used
    )]

    use super::*;

    const PRESENT: [u32; 16] = [
        0xc, 0x5, 0x6, 0xb, 0x9, 0x0, 0xa, 0xd, 0x3, 0xe, 0xf, 0x8, 0x4, 0x7, 0x1, 0x2,
    ];

    /// PRESENT's pLayer: bit i moves to bit 16 i mod 63, and bit 63 stays.
    fn player() -> LinearMap {
        let columns: Vec<u128> = (0..64_u32)
            .map(|i| 1_u128 << if i == 63 { 63 } else { (16 * i) % 63 })
            .collect();
        LinearMap::new(64, 1, &columns).unwrap()
    }

    fn done(costs: &[Computed<u32>]) -> Vec<u32> {
        costs.iter().map(|cost| *cost.done().unwrap()).collect()
    }

    #[test]
    fn present_matches_its_published_bounds() {
        let sbox = BitFunction::new(4, 4, &PRESENT).unwrap();
        let network = Network::new(&sbox, &player()).unwrap().unwrap();
        assert_eq!(
            (network.sbox_bits(), network.sboxes(), network.bits()),
            (4, 16, 64)
        );
        let differential = network.differential(4).unwrap();
        assert_eq!(done(&differential.active), [1, 2, 4, 6]);
        assert_eq!(done(&differential.weight.unwrap()), [2, 4, 8, 12]);
        assert_eq!(network.full_diffusion().unwrap(), Some(3));
    }

    fn generator(seed: u64) -> impl FnMut() -> u64 {
        let mut state = seed;
        move || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        }
    }

    fn random_permutation(bits: u32, next: &mut impl FnMut() -> u64) -> Vec<u32> {
        let mut values: Vec<u32> = (0..1_u32 << bits).collect();
        for i in (1..values.len()).rev() {
            let j = (next() % (i as u64 + 1)) as usize;
            values.swap(i, j);
        }
        values
    }

    fn random_invertible(bits: u32, next: &mut impl FnMut() -> u64) -> LinearMap {
        loop {
            let columns: Vec<u128> = (0..bits)
                .map(|_| u128::from(next()) & ((1_u128 << bits) - 1))
                .collect();
            let map = LinearMap::new(bits, 1, &columns).unwrap();
            if map.inverse().unwrap().is_some() {
                return map;
            }
        }
    }

    /// The least cost of an r-round trail for each r, by dynamic
    /// programming over every state: cost_r(a) is the least cost of an
    /// r-round trail with input a.
    fn naive(
        table: &[u32],
        sbox_bits: u32,
        map: &dyn Fn(u128) -> u128,
        bits: u32,
        rounds: usize,
        cost: &dyn Fn(u32) -> u32,
    ) -> Vec<u32> {
        let states = 1_usize << bits;
        let words = bits / sbox_bits;
        let per = 1_usize << sbox_bits;
        let word = |x: usize, c: u32| (x >> (c * sbox_bits)) & (per - 1);
        let step = |a: usize, b: usize| -> Option<u32> {
            let mut total = 0;
            for c in 0..words {
                let (x, y) = (word(a, c), word(b, c));
                if x == 0 && y == 0 {
                    continue;
                }
                let entry = table[x * per + y];
                if x == 0 || entry == 0 {
                    return None;
                }
                total += cost(entry);
            }
            Some(total)
        };
        let mut previous: Vec<u32> = vec![0; states];
        let mut least = Vec::new();
        for round in 0..rounds {
            let mut current = vec![u32::MAX; states];
            for (a, least) in current.iter_mut().enumerate().skip(1) {
                for b in 1..states {
                    if let Some(here) = step(a, b) {
                        let rest = if round == 0 {
                            0
                        } else {
                            previous[map(b as u128) as usize]
                        };
                        *least = (*least).min(here.saturating_add(rest));
                    }
                }
            }
            least.push(*current.iter().skip(1).min().unwrap());
            previous = current;
        }
        least
    }

    /// Bounds with the table and the map between rounds that produced them.
    type Case<'a> = (TrailBounds, &'a Vec<u32>, &'a dyn Fn(u128) -> u128);

    #[test]
    fn searches_agree_with_dynamic_programming() {
        let mut next = generator(0x5eed_7a11);
        for &(sbox_bits, bits, rounds, trials) in &[
            (2, 8, 5, 4),
            (4, 8, 5, 4),
            (3, 9, 4, 3),
            (2, 10, 3, 2),
            (5, 10, 3, 2),
        ] {
            for trial in 0..trials {
                let values = if trial == 0 && sbox_bits == 4 {
                    PRESENT.to_vec()
                } else {
                    random_permutation(sbox_bits, &mut next)
                };
                let sbox = BitFunction::new(sbox_bits, sbox_bits, &values).unwrap();
                let layer = random_invertible(bits, &mut next);
                let network = Network::new(&sbox, &layer).unwrap().unwrap();
                let inverse = layer.inverse().unwrap().unwrap();
                let masks = inverse.transpose().unwrap();
                let differences = sbox.difference_table().unwrap();
                let correlations: Vec<u32> = sbox
                    .linear_table()
                    .unwrap()
                    .iter()
                    .map(|entry| entry.unsigned_abs() * 2)
                    .collect();
                let weight = |entry: u32| sbox_bits - entry.trailing_zeros();
                let cases: [Case<'_>; 2] = [
                    (
                        network.differential(rounds as u32).unwrap(),
                        &differences,
                        &|x| layer.apply(x),
                    ),
                    (
                        network.linear(rounds as u32).unwrap(),
                        &correlations,
                        &|x| masks.apply(x),
                    ),
                ];
                for (bounds, table, map) in cases {
                    assert_eq!(
                        done(&bounds.active),
                        naive(table, sbox_bits, map, bits, rounds, &|_| 1),
                        "active, s = {sbox_bits}, n = {bits}, trial {trial}"
                    );
                    let integral = table
                        .iter()
                        .skip(1 << sbox_bits)
                        .all(|entry| *entry == 0 || entry.is_power_of_two());
                    match bounds.weight {
                        Some(weights) => {
                            assert!(integral);
                            assert_eq!(
                                done(&weights),
                                naive(table, sbox_bits, map, bits, rounds, &weight),
                                "weight, s = {sbox_bits}, n = {bits}, trial {trial}"
                            );
                        }
                        None => assert!(!integral),
                    }
                }
            }
        }
    }

    /// The S-box of Heys's tutorial network, whose difference table has
    /// entries of 6.
    const HEYS: [u32; 16] = [
        0xe, 0x4, 0xd, 0x1, 0x2, 0xf, 0xb, 0x8, 0x3, 0xa, 0x6, 0xc, 0x5, 0x9, 0x0, 0x7,
    ];

    fn permutation(bits: u32, place: impl Fn(u32) -> u32) -> LinearMap {
        let columns: Vec<u128> = (0..bits).map(|i| 1_u128 << place(i)).collect();
        LinearMap::new(bits, 1, &columns).unwrap()
    }

    #[test]
    fn a_search_past_its_budget_stops_that_round_and_every_later_one() {
        let sbox = BitFunction::new(4, 4, &PRESENT).unwrap();
        let network = Network::new(&sbox, &player()).unwrap().unwrap();
        let bounds = network
            .bounds(&network.differences, &network.forward, 2, 5, 3_000)
            .unwrap();
        assert_eq!(bounds.active.len(), 5);
        let first = bounds
            .active
            .iter()
            .position(|cost| cost.done().is_none())
            .unwrap();
        assert!(first >= 2, "{bounds:?}");
        assert_eq!(done(&bounds.active[..first]), [1, 2, 4, 6][..first]);
        for cost in &bounds.active[first..] {
            assert!(matches!(cost, Computed::TooCostly(steps) if *steps == 3_001));
        }
        let unlimited = network
            .bounds(&network.differences, &network.forward, 2, 4, u64::MAX)
            .unwrap();
        assert_eq!(done(&unlimited.active), [1, 2, 4, 6]);
        assert!(network.differential(0).unwrap().active.is_empty());
        let separate = Network::new(&sbox, &permutation(12, |i| i))
            .unwrap()
            .unwrap();
        let clamped = separate.linear(40).unwrap();
        assert_eq!(done(&clamped.active), (1..=32).collect::<Vec<_>>());
    }

    #[test]
    fn weights_are_integers_only_for_tables_of_powers_of_two() {
        let sbox = BitFunction::new(4, 4, &HEYS).unwrap();
        let transpose = permutation(16, |i| (i % 4) * 4 + i / 4);
        let network = Network::new(&sbox, &transpose).unwrap().unwrap();
        let differential = network.differential(4).unwrap();
        assert_eq!(differential.weight, None);
        assert_eq!(done(&differential.active), [1, 2, 4, 6]);
        let linear = network.linear(4).unwrap();
        assert_eq!(linear.weight, None);
        assert_eq!(done(&linear.active), [1, 2, 3, 4]);
        assert_eq!(network.full_diffusion().unwrap(), Some(2));
    }

    #[test]
    fn networks_need_a_small_permutation_a_whole_number_of_words_and_an_invertible_layer() {
        let present = BitFunction::new(4, 4, &PRESENT).unwrap();
        let identity = |bits| permutation(bits, |i| i);
        assert!(Network::new(&present, &identity(12)).unwrap().is_some());
        assert!(Network::new(&present, &identity(10)).unwrap().is_none());
        let narrowed: Vec<u32> = PRESENT.iter().map(|value| value & 7).collect();
        let collision = BitFunction::new(4, 4, &narrowed).unwrap();
        assert!(Network::new(&collision, &identity(8)).unwrap().is_none());
        let wide = BitFunction::new(4, 3, &narrowed).unwrap();
        assert!(Network::new(&wide, &identity(8)).unwrap().is_none());
        let one = BitFunction::new(1, 1, &[1, 0]).unwrap();
        assert!(Network::new(&one, &identity(8)).unwrap().is_none());
        let nine: Vec<u32> = (0..512).collect();
        let nine = BitFunction::new(9, 9, &nine).unwrap();
        assert!(Network::new(&nine, &identity(18)).unwrap().is_none());
        let eight: Vec<u32> = (0..256).rev().collect();
        let eight = BitFunction::new(8, 8, &eight).unwrap();
        assert!(Network::new(&eight, &identity(128)).unwrap().is_some());
        let mut columns: Vec<u128> = (0..8).map(|i| 1_u128 << i).collect();
        columns[7] = 1;
        let singular = LinearMap::new(8, 1, &columns).unwrap();
        assert!(Network::new(&present, &singular).unwrap().is_none());
    }

    /// The least r with every output bit depending on every input bit, by
    /// following the dependence of each input bit round by round.
    fn naive_diffusion(network: &Network) -> Option<u32> {
        let n = network.bits;
        let full = low_mask(n);
        let limit = (n - 1) * (n - 1) + 1;
        let mut reached: Vec<u128> = (0..n).map(|j| 1_u128 << j).collect();
        for rounds in 1..=limit {
            for set in &mut reached {
                let mut next = 0;
                for (i, row) in network.dependence.iter().enumerate() {
                    if row & *set != 0 {
                        next |= 1_u128 << i;
                    }
                }
                *set = next;
            }
            if reached.iter().all(|set| *set == full) {
                return Some(rounds);
            }
        }
        None
    }

    #[test]
    fn full_diffusion_follows_each_input_bit() {
        let mut next = generator(0xd1ff_0510);
        for &(sbox_bits, bits) in &[(2, 8), (3, 9), (4, 16), (4, 24), (8, 32)] {
            for _ in 0..6 {
                let values = random_permutation(sbox_bits, &mut next);
                let sbox = BitFunction::new(sbox_bits, sbox_bits, &values).unwrap();
                let layer = random_invertible(bits, &mut next);
                let network = Network::new(&sbox, &layer).unwrap().unwrap();
                assert_eq!(network.full_diffusion().unwrap(), naive_diffusion(&network));
            }
        }
        let present = BitFunction::new(4, 4, &PRESENT).unwrap();
        let rotate = permutation(16, |i| (i + 4) % 16);
        let network = Network::new(&present, &rotate).unwrap().unwrap();
        assert_eq!(network.full_diffusion().unwrap(), None);
        let single = Network::new(&present, &permutation(4, |i| i))
            .unwrap()
            .unwrap();
        assert_eq!(network.full_diffusion().unwrap(), naive_diffusion(&network));
        assert_eq!(single.full_diffusion().unwrap(), Some(1));
        // A shift of each S-box's bits by one word and one bit, whose
        // dependence needs many rounds to fill every bit.
        let slow = Network::new(&present, &permutation(64, |i| (i + 5) % 64))
            .unwrap()
            .unwrap();
        assert_eq!(slow.full_diffusion().unwrap(), naive_diffusion(&slow));
    }
}
