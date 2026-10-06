//! Exact cryptanalysis of one checked function: `orangec analyze`.
//!
//! The reference evaluator computes the selected function at every input of
//! the analyzed domain, and [`orange_compiler::cryptanalysis`] computes each
//! property from the resulting table. The contract is
//! `docs/CRYPTANALYSIS_2026.md`.

use std::fmt::{self, Write as _};

use orange_compiler::cryptanalysis::{Cycles, Monomial, Summary};
use orange_compiler::{
    AnalysisError, BitFunction, Computed, CoreFunction, CoreModule, CoreType, CoreValue, Evaluator,
    MAX_ANALYSIS_BITS, MAX_ANALYSIS_OPERATIONS, SourceMap, render_diagnostics,
};

use super::{
    CliDiagnosticCode, CompilerCommand, MAX_STANDARD_ERROR_BYTES, MAX_STANDARD_OUTPUT_BYTES,
    Options, PreparedReport, render_cli_error, with_budget_hint, write_numeric_function,
};

/// Width of the label column of a report.
const LABEL_WIDTH: usize = 26;

/// What `analyze` analyzes and prints.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct Analysis {
    /// The selected function, `MODULE::NAME`.
    pub(crate) function: String,
    /// Its complete numeric instance.
    pub(crate) instance: Vec<u32>,
    /// Input bits, and output bits when they differ from the default.
    pub(crate) bits: Option<(u32, Option<u32>)>,
    /// The one complete table to print instead of the summary.
    pub(crate) table: Option<Table>,
}

/// A complete table `analyze --table` prints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Table {
    Values,
    Ddt,
    Lat,
    Bct,
    Anf,
}

impl Table {
    pub(crate) fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "values" => Self::Values,
            "ddt" => Self::Ddt,
            "lat" => Self::Lat,
            "bct" => Self::Bct,
            "anf" => Self::Anf,
            _ => return None,
        })
    }
}

/// Reads `--bits N` or `--bits N,M`: canonical decimals from 1 through
/// [`MAX_ANALYSIS_BITS`].
pub(crate) fn parse_bits(text: &str) -> Result<(u32, Option<u32>), String> {
    let invalid = || {
        format!(
            "option `--bits` takes N or N,M, input and output bits from 1 through {MAX_ANALYSIS_BITS}"
        )
    };
    let one = |item: &str| {
        if item.is_empty()
            || item.starts_with('0')
            || !item.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        item.parse::<u32>()
            .ok()
            .filter(|bits| (1..=MAX_ANALYSIS_BITS).contains(bits))
    };
    match text.split_once(',') {
        Some((input, output)) => Ok((
            one(input).ok_or_else(invalid)?,
            Some(one(output).ok_or_else(invalid)?),
        )),
        None => Ok((one(text).ok_or_else(invalid)?, None)),
    }
}

/// Evaluates the selected function at every input and renders its summary,
/// or the one table asked for, before any byte is written.
pub(crate) fn prepare(
    options: &Options,
    core: &CoreModule,
    sources: &SourceMap,
) -> Result<PreparedReport, String> {
    let missing = || {
        render_cli_error(
            CliDiagnosticCode::MissingPhaseArtifact,
            "analysis returned no complete artifact",
            "this is an internal compiler or resource failure",
        )
    };
    let analysis = options.analysis.as_ref().ok_or_else(missing)?;
    let selected = select(core, analysis)?;
    let (input_bits, output_bits) = domain(selected, analysis.bits)?;
    let steps = options.evaluation.steps;

    let mut evaluator = Evaluator::new(core).ok_or_else(missing)?;
    let inputs = 1_u32.checked_shl(input_bits).ok_or_else(missing)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(usize::try_from(inputs).map_err(|_| missing())?)
        .map_err(|_| missing())?;
    let mut total_steps = 0_usize;
    let mut largest_call = 0_usize;
    let parameter = selected.parameters().first().ok_or_else(missing)?;
    for x in 0..inputs {
        let argument = word(parameter, x).ok_or_else(missing)?;
        let result = evaluator
            .call(selected, &[argument], steps)
            .ok_or_else(missing)?;
        total_steps = total_steps.saturating_add(result.steps());
        largest_call = largest_call.max(result.steps());
        let Some(value) = result.value() else {
            if result.diagnostics().is_empty() {
                return Err(missing());
            }
            let at = format!("the analysis stopped at input 0x{}", hex(x, input_bits));
            let hinted = with_budget_hint(result.diagnostics(), steps, CompilerCommand::Analyze)
                .unwrap_or_else(|| result.diagnostics().to_vec());
            let noted = hinted
                .into_iter()
                .map(|diagnostic| diagnostic.with_note(at.clone()))
                .collect::<Vec<_>>();
            return Err(render_diagnostics(sources, &noted));
        };
        let value = bits_of(value).ok_or_else(missing)?;
        if value.checked_shr(output_bits).unwrap_or(0) != 0 {
            let mut name = String::new();
            write_numeric_function(&mut name, selected).map_err(|_| missing())?;
            return Err(render_cli_error(
                CliDiagnosticCode::AnalysisDomain,
                format_args!(
                    "`{name}` at input 0x{} is {value:#x}, which has more than {output_bits} output {}",
                    hex(x, input_bits),
                    noun(output_bits, "bit", "bits"),
                ),
                "`--bits N,M` analyzes N input bits and M output bits",
            ));
        }
        values.push(u32::try_from(value).map_err(|_| missing())?);
    }
    let function = BitFunction::new(input_bits, output_bits, &values).ok_or_else(missing)?;

    let mut output = Text::new(MAX_STANDARD_OUTPUT_BYTES);
    let rendered = match analysis.table {
        None => {
            let summary = function
                .summary()
                .map_err(|error| failure(error, &missing))?;
            write_summary(&mut output, selected, &function, &summary)
        }
        Some(Table::Values) => write_values(&mut output, &function),
        Some(Table::Ddt) => {
            let table = function
                .difference_table()
                .map_err(|error| failure(error, &missing))?;
            write_grid(&mut output, &function, &table, function.output_bits())
        }
        Some(Table::Lat) => {
            let table = function
                .linear_table()
                .map_err(|error| failure(error, &missing))?;
            write_grid(&mut output, &function, &table, function.output_bits())
        }
        Some(Table::Bct) => {
            let table = function
                .boomerang_table()
                .map_err(|error| failure(error, &missing))?;
            write_grid(&mut output, &function, &table, function.input_bits())
        }
        Some(Table::Anf) => {
            let forms = function
                .normal_form()
                .map_err(|error| failure(error, &missing))?;
            write_forms(&mut output, &forms)
        }
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
                ": {inputs} {}, {total_steps} {}",
                noun(inputs, "call", "calls"),
                noun(total_steps, "step", "steps")
            )?;
            writeln!(statistics, "largest call: {largest_call} of {steps} steps")
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

/// Returns the one function `analysis` selects.
fn select<'core>(
    core: &'core CoreModule,
    analysis: &Analysis,
) -> Result<&'core CoreFunction, String> {
    let (module, name) = analysis.function.split_once("::").unwrap_or_default();
    let mut matches = core.functions().iter().filter(|function| {
        function.module() == module
            && function.name() == name
            && function.sizes() == analysis.instance
            && function.title().is_none()
    });
    let selected = matches.next().ok_or_else(|| {
        render_cli_error(
            CliDiagnosticCode::EntryPoint,
            format_args!(
                "no function `{}` has the selected numeric instance",
                analysis.function
            ),
            "`--function` names a linked module's function; `--instance` supplies every size value and zero-based type-domain index, in declaration order",
        )
    })?;
    if matches.next().is_some() {
        return Err(render_cli_error(
            CliDiagnosticCode::EntryPoint,
            "the analysis function selector is ambiguous",
            "select one exact module, function, and complete numeric instance",
        ));
    }
    Ok(selected)
}

/// Returns the input and output bits analyzed: the selected `--bits`, or
/// the widths of the function's types.
fn domain(function: &CoreFunction, bits: Option<(u32, Option<u32>)>) -> Result<(u32, u32), String> {
    let signature = || {
        render_cli_error(
            CliDiagnosticCode::EntryPoint,
            "analysis requires one word parameter and a word or Bool result",
            "select a function such as `spec sbox(x: Word[8]) -> Word[8]`",
        )
    };
    let [parameter] = function.parameters() else {
        return Err(signature());
    };
    let input_width = width(parameter).ok_or_else(signature)?;
    let result = function.result_type();
    let output_width = if result == CoreType::Bool {
        1
    } else {
        width(&result).ok_or_else(signature)?
    };
    let (input_bits, output_bits) = match bits {
        None => (input_width, output_width),
        Some((input, None)) => (input, if result == CoreType::Bool { 1 } else { input }),
        Some((input, Some(output))) => (input, output),
    };
    if input_bits > input_width || output_bits > output_width {
        return Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!(
                "`--bits` selects {input_bits} input and {output_bits} output {}, wider than `{parameter} -> {result}`",
                noun(output_bits, "bit", "bits")
            ),
            "the analyzed bits are the low bits of the parameter and result types",
        ));
    }
    if input_bits > MAX_ANALYSIS_BITS || output_bits > MAX_ANALYSIS_BITS {
        return Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!(
                "a {input_bits}-bit to {output_bits}-bit function is too wide to analyze completely"
            ),
            format_args!(
                "`--bits N,M` selects the low N input and M output bits, at most {MAX_ANALYSIS_BITS} each"
            ),
        ));
    }
    Ok((input_bits, output_bits))
}

fn width(ty: &CoreType) -> Option<u32> {
    Some(match ty {
        CoreType::Word8 => 8,
        CoreType::Word16 => 16,
        CoreType::Word32 => 32,
        CoreType::Word64 => 64,
        _ => return None,
    })
}

/// Returns x as a value of the word type `ty`.
fn word(ty: &CoreType, x: u32) -> Option<CoreValue> {
    Some(match ty {
        CoreType::Word8 => CoreValue::Word8(u8::try_from(x).ok()?),
        CoreType::Word16 => CoreValue::Word16(u16::try_from(x).ok()?),
        CoreType::Word32 => CoreValue::Word32(x),
        CoreType::Word64 => CoreValue::Word64(u64::from(x)),
        _ => return None,
    })
}

fn bits_of(value: &CoreValue) -> Option<u64> {
    Some(match value {
        CoreValue::Bool(value) => u64::from(*value),
        CoreValue::Word8(value) => u64::from(*value),
        CoreValue::Word16(value) => u64::from(*value),
        CoreValue::Word32(value) => u64::from(*value),
        CoreValue::Word64(value) => *value,
        _ => return None,
    })
}

fn failure(error: AnalysisError, missing: &impl Fn() -> String) -> String {
    match error {
        AnalysisError::Allocation => missing(),
        AnalysisError::TableTooLarge => render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            "complete tables are printed for at most 10 input and 10 output bits",
            "the summary, without `--table`, covers up to 16 bits",
        ),
        AnalysisError::NotPermutation => render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            "the boomerang connectivity table needs a permutation",
            "the analyzed function is not a bijection of its input bits",
        ),
    }
}

fn write_summary(
    output: &mut Text,
    selected: &CoreFunction,
    function: &BitFunction,
    summary: &Summary,
) -> fmt::Result {
    let n = function.input_bits();
    let m = function.output_bits();
    write_numeric_function(output, selected)?;
    writeln!(
        output,
        "  {n} {} to {m} {}\n",
        noun(n, "bit", "bits"),
        noun(m, "bit", "bits")
    )?;
    let inputs = function.values().len();
    let outputs = 1_usize.checked_shl(m).unwrap_or(0);

    if n == m {
        let bijective = summary.cycles.is_some();
        line(output, "bijective", format_args!("{}", yes(bijective)))?;
        if !bijective {
            line(
                output,
                "image",
                format_args!("{} of {outputs} values", summary.image),
            )?;
        }
    } else if let Some(balanced) = summary.balanced {
        line(output, "balanced", format_args!("{}", yes(balanced)))?;
    } else {
        line(
            output,
            "injective",
            format_args!("{}", yes(summary.image == inputs)),
        )?;
    }
    if let Some(weight) = summary.weight {
        line(output, "weight", format_args!("{weight} of {inputs}"))?;
    }
    if let Some(fixed_points) = summary.fixed_points {
        line(output, "fixed points", format_args!("{fixed_points}"))?;
    }
    if let Some(cycles) = &summary.cycles {
        line(output, "cycle type", format_args!("{}", CycleType(cycles)))?;
    }

    output.write_str("\n")?;
    match &summary.differential {
        Computed::Done(differential) => {
            line(
                output,
                "differential uniformity",
                format_args!(
                    "{} (probability {}, {} {})",
                    differential.uniformity,
                    Fraction(differential.uniformity, n),
                    differential.reached,
                    noun(differential.reached, "pair", "pairs")
                ),
            )?;
            line(
                output,
                "differential spectrum",
                format_args!("{}", Spectrum(&differential.spectrum)),
            )?;
            if m != 1 {
                line(
                    output,
                    "differential branch",
                    format_args!("{}", differential.branch_number),
                )?;
            }
            if let Some(indicator) = differential.absolute_indicator {
                line(output, "absolute indicator", format_args!("{indicator}"))?;
            }
        }
        Computed::TooCostly(operations) => skipped(output, "differential uniformity", *operations)?,
    }

    output.write_str("\n")?;
    match &summary.linear {
        Computed::Done(linear) => {
            let half = inputs.checked_div(2).unwrap_or(0);
            let nonlinearity = u32::try_from(half)
                .unwrap_or(u32::MAX)
                .saturating_sub(linear.linearity.checked_div(2).unwrap_or(0));
            line(
                output,
                "linearity",
                format_args!(
                    "{} (nonlinearity {nonlinearity}, correlation {})",
                    linear.linearity,
                    Fraction(linear.linearity, n)
                ),
            )?;
            line(
                output,
                "walsh spectrum",
                format_args!("{}", Spectrum(&linear.spectrum)),
            )?;
            if m == 1 {
                line(
                    output,
                    "correlation immunity",
                    format_args!("{}", linear.correlation_immunity),
                )?;
            } else {
                line(
                    output,
                    "linear branch",
                    format_args!("{}", linear.branch_number),
                )?;
            }
        }
        Computed::TooCostly(operations) => skipped(output, "linearity", *operations)?,
    }

    output.write_str("\n")?;
    match &summary.algebraic {
        Computed::Done(algebraic) => {
            if m == 1 {
                line(
                    output,
                    "algebraic degree",
                    format_args!("{}", algebraic.degree),
                )?;
            } else if algebraic.minimum_degree == algebraic.degree {
                line(
                    output,
                    "algebraic degree",
                    format_args!("{} (every component)", algebraic.degree),
                )?;
            } else {
                line(
                    output,
                    "algebraic degree",
                    format_args!(
                        "{} (components {} to {})",
                        algebraic.degree, algebraic.minimum_degree, algebraic.degree
                    ),
                )?;
            }
            if let Some(inverse) = algebraic.inverse_degree {
                line(output, "inverse degree", format_args!("{inverse}"))?;
            }
        }
        Computed::TooCostly(operations) => skipped(output, "algebraic degree", *operations)?,
    }
    match &summary.equations {
        Computed::Done(equations) => line(
            output,
            "quadratic equations",
            format_args!(
                "{} ({} bi-affine)",
                equations.quadratic, equations.bi_affine
            ),
        )?,
        Computed::TooCostly(operations) => skipped(output, "quadratic equations", *operations)?,
    }
    match &summary.boomerang_uniformity {
        Some(Computed::Done(uniformity)) => {
            line(output, "boomerang uniformity", format_args!("{uniformity}"))?;
        }
        Some(Computed::TooCostly(operations)) => {
            skipped(output, "boomerang uniformity", *operations)?;
        }
        None => {}
    }
    Ok(())
}

fn line(output: &mut Text, label: &str, value: fmt::Arguments<'_>) -> fmt::Result {
    writeln!(output, "{label:<LABEL_WIDTH$}{value}")
}

fn skipped(output: &mut Text, label: &str, operations: u64) -> fmt::Result {
    let exponent = u64::BITS.saturating_sub(operations.saturating_sub(1).leading_zeros());
    line(
        output,
        label,
        format_args!(
            "not computed: about 2^{exponent} operations, over the limit of 2^{}",
            MAX_ANALYSIS_OPERATIONS.trailing_zeros()
        ),
    )
}

const fn yes(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn noun<T: PartialEq + From<u8>>(count: T, one: &'static str, many: &'static str) -> &'static str {
    if count == T::from(1) { one } else { many }
}

/// A count over 2^n in lowest terms: a power of two when it is one, and
/// otherwise an odd numerator over a power of two.
struct Fraction(u32, u32);

impl fmt::Display for Fraction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self(count, bits) = *self;
        let twos = count.trailing_zeros().min(bits);
        let numerator = count.checked_shr(twos).unwrap_or(0);
        let exponent = bits.saturating_sub(twos);
        match (numerator, exponent) {
            (_, 0) => write!(formatter, "{numerator}"),
            (1, _) => write!(formatter, "2^-{exponent}"),
            _ => write!(formatter, "{numerator}/2^{exponent}"),
        }
    }
}

/// Most distinct values a spectrum lists before it is summarized.
const MAX_LISTED_SPECTRUM: usize = 16;

/// Values with their multiplicities, `value: count`, ascending; a spectrum
/// of more than [`MAX_LISTED_SPECTRUM`] values gives only their number and
/// range.
struct Spectrum<'a>(&'a [(u32, u64)]);

impl fmt::Display for Spectrum<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let ([(least, _), ..], [.., (most, _)]) = (self.0, self.0)
            && self.0.len() > MAX_LISTED_SPECTRUM
        {
            return write!(
                formatter,
                "{} distinct values from {least} to {most}",
                self.0.len()
            );
        }
        for (index, (value, count)) in self.0.iter().enumerate() {
            if index != 0 {
                formatter.write_str("  ")?;
            }
            write!(formatter, "{value}: {count}")?;
        }
        Ok(())
    }
}

/// Cycle lengths, descending, a repeated length written length^count.
struct CycleType<'a>(&'a Cycles);

impl fmt::Display for CycleType<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, (length, count)) in self.0.lengths.iter().enumerate() {
            if index != 0 {
                formatter.write_str(" ")?;
            }
            if *count == 1 {
                write!(formatter, "{length}")?;
            } else {
                write!(formatter, "{length}^{count}")?;
            }
        }
        Ok(())
    }
}

/// Returns x in hexadecimal with enough digits for `bits` bits.
fn hex(x: impl Into<u64>, bits: u32) -> String {
    let digits = usize::try_from(bits.div_ceil(4)).unwrap_or(1);
    format!("{:0digits$x}", x.into())
}

/// Prints the value at each input, sixteen to a line, each line headed by
/// its first input.
fn write_values(output: &mut Text, function: &BitFunction) -> fmt::Result {
    for (row, chunk) in function.values().chunks(16).enumerate() {
        let first = row.saturating_mul(16);
        write!(
            output,
            "{}",
            hex(u64::try_from(first).unwrap_or(0), function.input_bits())
        )?;
        for value in chunk {
            write!(
                output,
                " {}",
                hex(u64::try_from(*value).unwrap_or(0), function.output_bits())
            )?;
        }
        output.write_str("\n")?;
    }
    Ok(())
}

/// Prints a table of 2^n rows and 2^`column_bits` columns, row a and column
/// b at index a * 2^column_bits + b, with the rows and columns headed by a
/// and b in hexadecimal.
fn write_grid<T: fmt::Display>(
    output: &mut Text,
    function: &BitFunction,
    table: &[T],
    column_bits: u32,
) -> fmt::Result {
    let columns = 1_usize.checked_shl(column_bits).unwrap_or(1);
    let row_label = usize::try_from(function.input_bits().div_ceil(4)).unwrap_or(1);
    let column_label = usize::try_from(column_bits.div_ceil(4)).unwrap_or(1);
    let cell = table
        .iter()
        .map(|entry| DisplayWidth::of(entry))
        .max()
        .unwrap_or(1)
        .max(column_label);
    write!(output, "{:row_label$}", "")?;
    for b in 0..columns {
        write!(
            output,
            " {:>cell$}",
            hex(u64::try_from(b).unwrap_or(0), column_bits)
        )?;
    }
    output.write_str("\n")?;
    for (a, row) in table.chunks(columns).enumerate() {
        write!(
            output,
            "{}",
            hex(u64::try_from(a).unwrap_or(0), function.input_bits())
        )?;
        for entry in row {
            write!(output, " {entry:>cell$}")?;
        }
        output.write_str("\n")?;
    }
    Ok(())
}

/// Counts the characters a value displays as.
struct DisplayWidth(usize);

impl DisplayWidth {
    fn of(value: &impl fmt::Display) -> usize {
        let mut width = Self(0);
        let _ = write!(width, "{value}");
        width.0
    }
}

impl fmt::Write for DisplayWidth {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 = self.0.saturating_add(text.chars().count());
        Ok(())
    }
}

/// Prints `yj = ...` for each output bit, monomials joined by ` + `.
fn write_forms(output: &mut Text, forms: &[Vec<Monomial>]) -> fmt::Result {
    for (bit, form) in forms.iter().enumerate() {
        write!(output, "y{bit} =")?;
        if form.is_empty() {
            output.write_str(" 0")?;
        }
        for (index, monomial) in form.iter().enumerate() {
            output.write_str(if index == 0 { " " } else { " + " })?;
            if *monomial == 0 {
                output.write_str("1")?;
            }
            for variable in 0..usize::BITS {
                if monomial.checked_shr(variable).unwrap_or(0) & 1 == 1 {
                    write!(output, "x{variable}")?;
                }
            }
        }
        output.write_str("\n")?;
    }
    Ok(())
}

/// Output text built in full before any byte is written, within a limit.
struct Text {
    text: String,
    limit: usize,
    exceeded: bool,
}

impl Text {
    const fn new(limit: usize) -> Self {
        Self {
            text: String::new(),
            limit,
            exceeded: false,
        }
    }

    fn failure(&self) -> String {
        if self.exceeded {
            render_cli_error(
                CliDiagnosticCode::OutputTooLarge,
                "analysis report exceeds the output limit",
                "orangec writes at most 64 MiB per output stream per invocation",
            )
        } else {
            render_cli_error(
                CliDiagnosticCode::MissingPhaseArtifact,
                "could not allocate the complete analysis report",
                "this is an internal compiler or resource failure",
            )
        }
    }
}

impl fmt::Write for Text {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self
            .text
            .len()
            .checked_add(text.len())
            .is_none_or(|length| length > self.limit)
        {
            self.exceeded = true;
            return Err(fmt::Error);
        }
        self.text.try_reserve(text.len()).map_err(|_| fmt::Error)?;
        self.text.push_str(text);
        Ok(())
    }
}
