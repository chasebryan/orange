//! `orangec analyze --layer`: bounds on the trails of a substitution-
//! permutation network whose round is the selected S-box on every word of
//! the layer's bits, then the layer.
//!
//! The layer is read and checked as `--linear` reads it. The S-box must be
//! a permutation of 2 to 8 bits, its width must divide the layer's, and the
//! layer must be invertible.

use std::fmt::{self, Write as _};

use orange_compiler::{
    Computed, CoreFunction, CoreModule, MAX_SBOX_BITS, MAX_TRAIL_STEPS, Network, SourceMap,
    TrailBounds,
};

use super::linear::{self, Checked, Shape};
use super::{LABEL_WIDTH, Tabulated, Text, Trails, line, noun};
use crate::{
    CliDiagnosticCode, MAX_STANDARD_ERROR_BYTES, MAX_STANDARD_OUTPUT_BYTES, Options,
    PreparedReport, render_cli_error, write_numeric_function,
};

/// Width of the rounds column of a trail table.
const ROUNDS_WIDTH: usize = 8;

/// Width of the active S-boxes column of a trail table.
const ACTIVE_WIDTH: usize = 16;

/// Builds the network of the tabulated S-box and the selected layer, and
/// renders its full diffusion and trail bounds before any byte is written.
pub(super) fn prepare(
    options: &Options,
    core: &CoreModule,
    sources: &SourceMap,
    selected: &CoreFunction,
    tabulated: &Tabulated,
    trails: &Trails,
) -> Result<PreparedReport, String> {
    let missing = || {
        render_cli_error(
            CliDiagnosticCode::MissingPhaseArtifact,
            "analysis returned no complete artifact",
            "this is an internal compiler or resource failure",
        )
    };
    let sbox = &tabulated.function;
    let s = sbox.input_bits();
    let mut name = String::new();
    write_numeric_function(&mut name, selected).map_err(|_| missing())?;
    if !(2..=MAX_SBOX_BITS).contains(&s) {
        return Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!(
                "an S-box of {s} {} is outside the 2 to {MAX_SBOX_BITS} bits of a trail search",
                noun(s, "bit", "bits")
            ),
            "`--bits N` selects the low N bits of the S-box's parameter and result",
        ));
    }
    if sbox.output_bits() != s || !sbox.is_permutation() {
        return Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!("`{name}` is not a permutation of {s} bits"),
            "a trail search needs an invertible S-box; `--bits N` selects its low N bits",
        ));
    }

    let layer = select_layer(core, &trails.layer)?;
    let shape = Shape::of(layer)?;
    let n = shape.bits;
    if !n.is_multiple_of(s) {
        return Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!(
                "the {n} bits of `{}` are not a whole number of {s}-bit S-boxes",
                layer.result_type()
            ),
            "a round applies the S-box to each word of the layer's bits, word c holding bits c s through c s + s - 1",
        ));
    }
    let read = linear::read(
        options,
        core,
        sources,
        layer,
        &shape,
        1,
        "`--layer` selects the linear layer of a round, a map x -> M x + c",
    )?;
    let mut layer_name = String::new();
    write_numeric_function(&mut layer_name, layer).map_err(|_| missing())?;
    let Some(network) = Network::new(sbox, &read.map).map_err(|_| missing())? else {
        return Err(render_cli_error(
            CliDiagnosticCode::AnalysisDomain,
            format_args!("`{layer_name}` is not invertible over GF(2)"),
            "a round's linear layer must be a bijection; `--linear` reports its rank",
        ));
    };
    let diffusion = network.full_diffusion().map_err(|_| missing())?;
    let differential = network.differential(trails.rounds).map_err(|_| missing())?;
    let linear = network.linear(trails.rounds).map_err(|_| missing())?;

    let mut output = Text::new(MAX_STANDARD_OUTPUT_BYTES);
    let rendered = (|| {
        line(
            &mut output,
            "round",
            format_args!(
                "{name} on {} {} of {s} bits, then {layer_name}",
                network.sboxes(),
                noun(network.sboxes(), "word", "words")
            ),
        )?;
        match read.checked {
            Checked::Every(inputs) => line(
                &mut output,
                "layer checked",
                format_args!("all {inputs} inputs"),
            )?,
            Checked::Pairs(inputs) => line(
                &mut output,
                "layer checked",
                format_args!(
                    "the {inputs} inputs of at most 2 bits: no term of degree 2, higher degrees unchecked"
                ),
            )?,
        }
        match diffusion {
            Some(rounds) => line(
                &mut output,
                "full diffusion",
                format_args!("{rounds} {}", noun(rounds, "round", "rounds")),
            )?,
            None => line(
                &mut output,
                "full diffusion",
                format_args!(
                    "never: some output bit depends on some input bit after no number of rounds"
                ),
            )?,
        }
        output.write_str("\n")?;
        write_trails(
            &mut output,
            "differential trails",
            "probability 2^-w",
            "some DDT entry is not a power of two",
            &differential,
        )?;
        output.write_str("\n")?;
        write_trails(
            &mut output,
            "linear trails",
            "correlation 2^-w in magnitude",
            "some |W(a, b)| is not a power of two",
            &linear,
        )
    })();
    if rendered.is_err() {
        return Err(output.failure());
    }

    let mut statistics = Text::new(MAX_STANDARD_ERROR_BYTES);
    if options.evaluation.stats
        && (tabulated
            .write_statistics(&mut statistics, selected)
            .is_err()
            || read.write_statistics(&mut statistics, layer).is_err())
    {
        return Err(statistics.failure());
    }
    Ok(PreparedReport {
        output: output.text,
        statistics: statistics.text,
    })
}

/// Returns the one function without size parameters that `--layer` names.
fn select_layer<'core>(
    core: &'core CoreModule,
    layer: &str,
) -> Result<&'core CoreFunction, String> {
    let (module, name) = layer.split_once("::").unwrap_or_default();
    let mut matches = core.functions().iter().filter(|function| {
        function.module() == module
            && function.name() == name
            && function.sizes().is_empty()
            && function.title().is_none()
    });
    let selected = matches.next().ok_or_else(|| {
        render_cli_error(
            CliDiagnosticCode::EntryPoint,
            format_args!("no function `{layer}` without size or type parameters"),
            "`--layer` names a linked module's function with no size or type parameters",
        )
    })?;
    if matches.next().is_some() {
        return Err(render_cli_error(
            CliDiagnosticCode::EntryPoint,
            "the layer function selector is ambiguous",
            "select one exact module and function",
        ));
    }
    Ok(selected)
}

/// Prints the least number of active S-boxes and least weight of a trail
/// for each number of rounds, `-` where a search passed its limit.
fn write_trails(
    output: &mut Text,
    label: &str,
    weight: &str,
    unweighted: &str,
    bounds: &TrailBounds,
) -> fmt::Result {
    match &bounds.weight {
        Some(_) => line(
            output,
            label,
            format_args!("a trail of weight w has {weight}"),
        )?,
        None => line(
            output,
            label,
            format_args!("weights not computed: {unweighted}"),
        )?,
    }
    write!(output, "{:<ROUNDS_WIDTH$}", "rounds")?;
    if bounds.weight.is_some() {
        writeln!(output, "{:<ACTIVE_WIDTH$}least weight", "active S-boxes")?;
    } else {
        writeln!(output, "active S-boxes")?;
    }
    let mut stopped = false;
    for (index, active) in bounds.active.iter().enumerate() {
        let rounds = index.saturating_add(1);
        let active = Cell(active);
        stopped |= active.stopped();
        match bounds
            .weight
            .as_ref()
            .and_then(|weights| weights.get(index))
        {
            Some(weight) => {
                let weight = Cell(weight);
                stopped |= weight.stopped();
                let active = active.to_string();
                writeln!(
                    output,
                    "{rounds:<ROUNDS_WIDTH$}{active:<ACTIVE_WIDTH$}{weight}"
                )?;
            }
            None => writeln!(output, "{rounds:<ROUNDS_WIDTH$}{active}")?,
        }
    }
    if stopped {
        writeln!(
            output,
            "{:<LABEL_WIDTH$}a search passed its limit of 2^{} steps",
            "-",
            MAX_TRAIL_STEPS.trailing_zeros()
        )?;
    }
    Ok(())
}

/// A bound, or `-` for one not computed.
struct Cell<'a>(&'a Computed<u32>);

impl Cell<'_> {
    const fn stopped(&self) -> bool {
        matches!(self.0, Computed::TooCostly(_))
    }
}

impl fmt::Display for Cell<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Computed::Done(value) => write!(formatter, "{value}"),
            Computed::TooCostly(_) => formatter.write_str("-"),
        }
    }
}
