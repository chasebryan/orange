//! `orangec analyze --linear`: the matrix over GF(2) of a linear layer, read
//! from the function's values at 0 and at each single bit, and its exact
//! properties.
//!
//! A function of n bits is affine exactly when F(x) = F(0) + M x for the
//! matrix M whose column j is F(e_j) + F(0). The layer is checked against
//! that matrix at every input when n is at most [`MAX_EXHAUSTIVE_BITS`], and
//! otherwise at every input of at most two set bits, which shows exactly
//! that the algebraic normal form of F has no term of degree 2.

use std::fmt::{self, Write as _};

use orange_compiler::cryptanalysis::linear::{Fields, LayerSummary};
use orange_compiler::{
    ArrayType, Computed, CoreArray, CoreFunction, CoreModule, CoreType, CoreValue, Evaluator,
    LinearMap, MAX_LAYER_BITS, SourceMap, render_diagnostics,
};

use super::{Analysis, LABEL_WIDTH, Table, Text, hex, line, noun, skipped, yes};
use crate::{
    CliDiagnosticCode, CompilerCommand, MAX_STANDARD_ERROR_BYTES, MAX_STANDARD_OUTPUT_BYTES,
    Options, PreparedReport, render_cli_error, with_budget_hint, write_numeric_function,
};

/// Widest layer checked against its matrix at every input.
const MAX_EXHAUSTIVE_BITS: u32 = 16;

/// Word width of a single-word layer when `--word` is not given.
const DEFAULT_WORD_BITS: u32 = 8;

/// The type of a layer's parameter and result: one word, or a
/// one-dimensional array of words, element i holding bits i w through
/// i w + w - 1.
struct Shape {
    /// The array type, or `None` for a single word.
    array: Option<ArrayType>,
    /// The word type of the parameter or of each element.
    element: CoreType,
    /// The bits of `element`.
    element_bits: u32,
    /// All the bits, n.
    bits: u32,
}

impl Shape {
    /// Returns the shape of `function`, which must take one word or array of
    /// words and return a value of the same type.
    fn of(function: &CoreFunction) -> Result<Self, String> {
        let signature = || {
            render_cli_error(
                CliDiagnosticCode::EntryPoint,
                "linear analysis requires one parameter of a word or array type and a result of the same type",
                "select a function such as `spec mix(a: Word[8]^4) -> Word[8]^4`",
            )
        };
        let [parameter] = function.parameters() else {
            return Err(signature());
        };
        if function.result_type() != *parameter {
            return Err(signature());
        }
        let (array, element) = match parameter {
            CoreType::Array(array) if array.dimensions() == 1 => (Some(*array), array.element()),
            _ => (None, parameter.clone()),
        };
        let element_bits = element.word_bits().ok_or_else(signature)?;
        let count = array.map_or(1, ArrayType::length);
        let bits = element_bits.saturating_mul(count);
        if bits > MAX_LAYER_BITS {
            return Err(render_cli_error(
                CliDiagnosticCode::AnalysisDomain,
                format_args!("`{parameter}` has {bits} bits, too wide for a linear layer"),
                format_args!("a linear layer is analyzed over at most {MAX_LAYER_BITS} bits"),
            ));
        }
        Ok(Self {
            array,
            element,
            element_bits,
            bits,
        })
    }

    /// Returns x as a value of the layer's type.
    fn value(&self, x: u128) -> Option<CoreValue> {
        let Some(array) = self.array else {
            return word(&self.element, x);
        };
        let mask = low_mask(self.element_bits);
        let mut elements = Vec::new();
        elements
            .try_reserve_exact(usize::try_from(array.length()).ok()?)
            .ok()?;
        for index in 0..array.length() {
            let shift = index.checked_mul(self.element_bits)?;
            elements.push(word(
                &self.element,
                x.checked_shr(shift).unwrap_or(0) & mask,
            )?);
        }
        Some(CoreValue::Array(CoreArray::new(array, elements)?))
    }

    /// Writes x as a value of the layer's type: a word in hexadecimal, or
    /// the words of an array in index order.
    fn render(&self, x: u128) -> String {
        let Some(array) = self.array else {
            return format!("0x{}", hex(x, self.bits));
        };
        let mask = low_mask(self.element_bits);
        let mut text = String::from("[");
        for index in 0..array.length() {
            if index != 0 {
                text.push_str(", ");
            }
            let element = index
                .checked_mul(self.element_bits)
                .and_then(|shift| x.checked_shr(shift))
                .unwrap_or(0);
            let _ = write!(text, "0x{}", hex(element & mask, self.element_bits));
        }
        text.push(']');
        text
    }

    /// Returns the bits of a value of the layer's type.
    fn bits_of(&self, value: &CoreValue) -> Option<u128> {
        let CoreValue::Array(array) = value else {
            return word_bits(value);
        };
        let mut bits = 0_u128;
        for (index, element) in array.elements().iter().enumerate() {
            let shift = u32::try_from(index).ok()?.checked_mul(self.element_bits)?;
            bits |= word_bits(element)?.checked_shl(shift)?;
        }
        Some(bits)
    }
}

fn word(ty: &CoreType, x: u128) -> Option<CoreValue> {
    Some(match ty {
        CoreType::Word8 => CoreValue::Word8(u8::try_from(x).ok()?),
        CoreType::Word16 => CoreValue::Word16(u16::try_from(x).ok()?),
        CoreType::Word32 => CoreValue::Word32(u32::try_from(x).ok()?),
        CoreType::Word64 => CoreValue::Word64(u64::try_from(x).ok()?),
        _ => return None,
    })
}

fn word_bits(value: &CoreValue) -> Option<u128> {
    Some(match value {
        CoreValue::Word8(value) => u128::from(*value),
        CoreValue::Word16(value) => u128::from(*value),
        CoreValue::Word32(value) => u128::from(*value),
        CoreValue::Word64(value) => u128::from(*value),
        _ => return None,
    })
}

fn low_mask(bits: u32) -> u128 {
    1_u128
        .checked_shl(bits)
        .map_or(u128::MAX, |power| power.wrapping_sub(1))
}

fn unit(bit: u32) -> u128 {
    1_u128.checked_shl(bit).unwrap_or(0)
}

/// Calls the layer one input at a time, each call with the whole budget,
/// and counts the calls and their steps.
struct Calls<'core, 'a> {
    evaluator: Evaluator<'core>,
    function: &'core CoreFunction,
    shape: &'a Shape,
    sources: &'a SourceMap,
    steps: usize,
    calls: u64,
    total_steps: usize,
    largest_call: usize,
}

impl Calls<'_, '_> {
    fn call(&mut self, x: u128, missing: &impl Fn() -> String) -> Result<u128, String> {
        let argument = self.shape.value(x).ok_or_else(missing)?;
        let result = self
            .evaluator
            .call(self.function, &[argument], self.steps)
            .ok_or_else(missing)?;
        self.calls = self.calls.saturating_add(1);
        self.total_steps = self.total_steps.saturating_add(result.steps());
        self.largest_call = self.largest_call.max(result.steps());
        let Some(value) = result.value() else {
            if result.diagnostics().is_empty() {
                return Err(missing());
            }
            let at = format!("the analysis stopped at input {}", self.shape.render(x));
            let hinted =
                with_budget_hint(result.diagnostics(), self.steps, CompilerCommand::Analyze)
                    .unwrap_or_else(|| result.diagnostics().to_vec());
            let noted = hinted
                .into_iter()
                .map(|diagnostic| diagnostic.with_note(at.clone()))
                .collect::<Vec<_>>();
            return Err(render_diagnostics(self.sources, &noted));
        };
        self.shape.bits_of(value).ok_or_else(missing)
    }
}

/// Which inputs the layer was checked at.
enum Checked {
    /// Every input.
    Every(u128),
    /// Every input of at most two set bits.
    Pairs(u128),
}

/// Reads the matrix of the selected layer from its values, checks the layer
/// against it, and renders its summary, or its matrix, before any byte is
/// written.
pub(super) fn prepare<'core>(
    options: &Options,
    core: &'core CoreModule,
    sources: &SourceMap,
    analysis: &Analysis,
    selected: &'core CoreFunction,
) -> Result<PreparedReport, String> {
    let missing = || {
        render_cli_error(
            CliDiagnosticCode::MissingPhaseArtifact,
            "analysis returned no complete artifact",
            "this is an internal compiler or resource failure",
        )
    };
    let shape = Shape::of(selected)?;
    let n = shape.bits;
    let word_bits = match (analysis.word, shape.array) {
        (Some(word_bits), _) => word_bits,
        (None, Some(_)) => shape.element_bits,
        (None, None) => DEFAULT_WORD_BITS,
    };
    if word_bits > n || !n.is_multiple_of(word_bits) {
        return Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!(
                "words of {word_bits} {} do not divide the {n} bits of `{}`",
                noun(word_bits, "bit", "bits"),
                selected.result_type()
            ),
            "`--word W` groups the bits of a layer into words of W bits, a power of two dividing its width",
        ));
    }

    let mut calls = Calls {
        evaluator: Evaluator::new(core).ok_or_else(missing)?,
        function: selected,
        shape: &shape,
        sources,
        steps: options.evaluation.steps,
        calls: 0,
        total_steps: 0,
        largest_call: 0,
    };
    let constant = calls.call(0, &missing)?;
    let mut columns = Vec::new();
    columns
        .try_reserve_exact(usize::try_from(n).map_err(|_| missing())?)
        .map_err(|_| missing())?;
    for bit in 0..n {
        columns.push(calls.call(unit(bit), &missing)? ^ constant);
    }
    let map = LinearMap::new(n, word_bits, &columns).ok_or_else(missing)?;
    let checked = check(&mut calls, &map, constant, &missing)?;

    let mut output = Text::new(MAX_STANDARD_OUTPUT_BYTES);
    let rendered = match analysis.table {
        None => {
            let summary = map.summary().map_err(|_| missing())?;
            let fixed = map
                .affine_fixed_dimension(constant)
                .map_err(|_| missing())?;
            write_summary(
                &mut output,
                selected,
                &map,
                &Layer {
                    constant,
                    checked,
                    fixed,
                    summary: &summary,
                },
            )
        }
        Some(Table::Matrix) => {
            let rows = map.rows().map_err(|_| missing())?;
            write_matrix(&mut output, &map, &rows)
        }
        Some(_) => return Err(missing()),
    };
    if rendered.is_err() {
        return Err(output.failure());
    }

    let mut statistics = Text::new(MAX_STANDARD_ERROR_BYTES);
    if options.evaluation.stats {
        let rendered = (|| {
            write_numeric_function(&mut statistics, selected)?;
            writeln!(
                statistics,
                ": {} {}, {} {}",
                calls.calls,
                noun(calls.calls, "call", "calls"),
                calls.total_steps,
                noun(calls.total_steps, "step", "steps")
            )?;
            writeln!(
                statistics,
                "largest call: {} of {} steps",
                calls.largest_call, calls.steps
            )
        })();
        if rendered.is_err() {
            return Err(statistics.failure());
        }
    }
    Ok(PreparedReport {
        output: output.text,
        statistics: statistics.text,
    })
}

/// Checks that the layer equals x -> M x + `constant` at every input of at
/// most [`MAX_EXHAUSTIVE_BITS`] bits, or else at every input of two set
/// bits, in increasing order; inputs of fewer bits define M.
fn check(
    calls: &mut Calls<'_, '_>,
    map: &LinearMap,
    constant: u128,
    missing: &impl Fn() -> String,
) -> Result<Checked, String> {
    let n = map.bits();
    let verify = |calls: &mut Calls<'_, '_>, x: u128| -> Result<(), String> {
        let value = calls.call(x, missing)?;
        let expected = map.apply(x) ^ constant;
        if value == expected {
            return Ok(());
        }
        let mut name = String::new();
        write_numeric_function(&mut name, calls.function).map_err(|_| missing())?;
        Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!(
                "`{name}` is not affine over GF(2): at input {} it is {}, but its values at 0 and at single bits give {}",
                calls.shape.render(x),
                calls.shape.render(value),
                calls.shape.render(expected)
            ),
            "`--linear` analyzes a map x -> M x + c; analyze an S-box without `--linear`",
        ))
    };
    if n <= MAX_EXHAUSTIVE_BITS {
        let inputs = unit(n);
        for x in 1..inputs {
            if x.count_ones() >= 2 {
                verify(calls, x)?;
            }
        }
        return Ok(Checked::Every(inputs));
    }
    for high in 1..n {
        for low in 0..high {
            verify(calls, unit(high) | unit(low))?;
        }
    }
    let n = u128::from(n);
    let pairs = n
        .saturating_mul(n.saturating_sub(1))
        .checked_div(2)
        .unwrap_or(0);
    Ok(Checked::Pairs(pairs.saturating_add(n).saturating_add(1)))
}

/// What the summary reports beside the matrix's own properties.
struct Layer<'a> {
    constant: u128,
    checked: Checked,
    fixed: Option<u32>,
    summary: &'a LayerSummary,
}

fn write_summary(
    output: &mut Text,
    selected: &CoreFunction,
    map: &LinearMap,
    layer: &Layer<'_>,
) -> fmt::Result {
    let n = map.bits();
    let w = map.word_bits();
    let k = map.words();
    let summary = layer.summary;
    write_numeric_function(output, selected)?;
    writeln!(
        output,
        "  {n} {} as {k} {} of {w} {}\n",
        noun(n, "bit", "bits"),
        noun(k, "word", "words"),
        noun(w, "bit", "bits")
    )?;

    match layer.checked {
        Checked::Every(inputs) => line(output, "checked", format_args!("all {inputs} inputs"))?,
        Checked::Pairs(inputs) => line(
            output,
            "checked",
            format_args!(
                "the {inputs} inputs of at most 2 bits: no term of degree 2, higher degrees unchecked"
            ),
        )?,
    }
    if layer.constant == 0 {
        line(output, "form", format_args!("linear"))?;
    } else {
        line(
            output,
            "form",
            format_args!("affine, constant 0x{}", hex(layer.constant, n)),
        )?;
    }
    line(
        output,
        "rank",
        format_args!(
            "{} of {n}, {}",
            summary.rank,
            if summary.rank == n {
                "invertible"
            } else {
                "singular"
            }
        ),
    )?;
    match layer.fixed {
        None => line(output, "fixed points", format_args!("none"))?,
        Some(0) => line(output, "fixed points", format_args!("1"))?,
        Some(dimension) => line(output, "fixed points", format_args!("2^{dimension}"))?,
    }
    line(
        output,
        "involution",
        format_args!(
            "{}",
            yes(summary.involution && map.apply(layer.constant) == layer.constant)
        ),
    )?;
    line(
        output,
        "xor count, row by row",
        format_args!("{}", summary.xor_count),
    )?;

    output.write_str("\n")?;
    let most = k.saturating_add(1);
    for (label, branch) in [
        ("differential branch", &summary.differential_branch),
        ("linear branch", &summary.linear_branch),
    ] {
        match branch {
            Computed::Done(branch) if *branch == most => {
                line(
                    output,
                    label,
                    format_args!("{branch} of at most {most} (MDS)"),
                )?;
            }
            Computed::Done(branch) => {
                line(output, label, format_args!("{branch} of at most {most}"))?;
            }
            Computed::TooCostly(operations) => skipped(output, label, *operations)?,
        }
    }

    match &summary.fields {
        Some(fields) => {
            output.write_str("\n")?;
            write_fields(output, w, k, fields)
        }
        None if w > 1 => {
            output.write_str("\n")?;
            line(
                output,
                "field",
                format_args!("not searched for words of more than 8 bits"),
            )
        }
        None => Ok(()),
    }
}

/// Prints the fields GF(2^w) whose products the blocks are, and the k x k
/// matrix of those products.
fn write_fields(output: &mut Text, w: u32, k: u32, fields: &Fields) -> fmt::Result {
    let Some((first, rest)) = fields.moduli.split_first() else {
        return line(
            output,
            "field",
            format_args!("none: some block is not a product in any GF(2^{w})"),
        );
    };
    if fields.entries.iter().all(|entry| *entry <= 1) {
        line(
            output,
            "field",
            format_args!("every GF(2^{w}): each block is 0 or 1"),
        )?;
    } else if rest.is_empty() {
        line(output, "field", format_args!("GF(2^{w}) modulo {first:#x}"))?;
    } else {
        let mut each = format!("{first:#x}");
        for modulus in rest {
            write!(each, ", {modulus:#x}")?;
        }
        line(
            output,
            "field",
            format_args!("GF(2^{w}) modulo each of {each}"),
        )?;
    }
    let columns = usize::try_from(k).unwrap_or(1).max(1);
    for (row, entries) in fields.entries.chunks(columns).enumerate() {
        if row == 0 {
            write!(output, "{:<LABEL_WIDTH$}", "field matrix")?;
        } else {
            write!(output, "{:LABEL_WIDTH$}", "")?;
        }
        for (column, entry) in entries.iter().enumerate() {
            if column != 0 {
                output.write_str(" ")?;
            }
            write!(output, "{}", hex(*entry, w))?;
        }
        output.write_str("\n")?;
    }
    Ok(())
}

/// Prints the matrix one row per output bit, `yi` and then the coefficient
/// of each input bit x0, x1, ... as 0 or 1, a space between words.
fn write_matrix(output: &mut Text, map: &LinearMap, rows: &[u128]) -> fmt::Result {
    let n = map.bits();
    let w = map.word_bits().max(1);
    let label = n.saturating_sub(1).to_string().len().saturating_add(1);
    for (i, row) in rows.iter().enumerate() {
        write!(output, "{:<label$}", format!("y{i}"))?;
        for j in 0..n {
            if j.is_multiple_of(w) {
                output.write_str(" ")?;
            }
            output.write_str(if row & unit(j) == 0 { "0" } else { "1" })?;
        }
        output.write_str("\n")?;
    }
    Ok(())
}
