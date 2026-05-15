use crate::experiment_sealed::ExperimentSealed;
use crate::{Experiment, Factors};
use cli_table::{Cell, CellStruct, Color, Style, Table, format::Justify, print_stdout};
use colorize::AnsiColor;
use std::fs::File;
use std::io::{Read, Write};
use std::{cmp::Ordering, path::PathBuf};
use thousands::Separable;

const MAX_PROGRESS_BAR_WIDTH: usize = 20;
const PROGRESS_BAR_CHAR: &'static str = "█";
const INPUT_SEPRATOR_CHAR: &'static str = "━";

fn time_progress_bar(estimate: Option<f64>, max_estimate: f64) -> String {
    match estimate {
        Some(value) if max_estimate > 0.0 => {
            let ratio = (value / max_estimate).clamp(0.0, 1.0);
            let mut width = (ratio * MAX_PROGRESS_BAR_WIDTH as f64).round() as usize;
            if value > 0.0 && width == 0 {
                width = 1;
            }
            PROGRESS_BAR_CHAR.repeat(width)
        }
        _ => String::new(),
    }
}

fn collect_point_estimates<E: Experiment>(
    exp: &E,
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
) -> Vec<Vec<Option<f64>>> {
    input_levels
        .iter()
        .map(|input_variant| {
            alg_levels
                .iter()
                .map(|alg_variant| {
                    let execution_path = exp.run_estimates_path(name, input_variant, alg_variant);
                    get_slope_point_estimate(&execution_path)
                })
                .collect()
        })
        .collect()
}

fn get_slope_point_estimate(path: &PathBuf) -> Option<f64> {
    let mut file = File::open(path).ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;

    let field_slope_null = "\"slope\":null";
    let is_slope_null = contents.contains(field_slope_null);

    let field = match is_slope_null {
        true => "\"mean\"",
        false => "\"slope\"",
    };
    let position = contents.find(field)?;
    let begin = position + field.len();
    let slice = &contents[begin..];

    let field_estimate = "\"point_estimate\":";
    let position = slice.find(field_estimate)?;
    let begin = position + field_estimate.len();
    let slice = &slice[begin..];

    let comma = ",";
    let position = slice.find(comma)?;
    let slice = &slice[0..position];

    slice.parse().ok()
}

pub fn summarize<E: Experiment>(
    exp: &E,
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
) {
    let estimates = collect_point_estimates(exp, name, input_levels, alg_levels);

    create_summary_csv(exp, name, input_levels, alg_levels, &estimates)
        .expect("Failed to create csv summary");

    let log = format!(
        "\nSummary table created at:\n{:?}\n",
        exp.summary_csv_path(name)
    );
    println!("{}", log.italic());

    print_summary_table::<E>(name, input_levels, alg_levels, &estimates);

    create_ai_prompt_to_analyze(exp, name, input_levels, alg_levels)
        .expect("Failed to create ai prompt");
    let log = format!(
        "\nA draft AI prompt to analyze the summary table is created at:\n{:?}\n",
        exp.ai_prompt_path(name)
    );
    println!("{}", log.italic());
}

fn create_summary_csv<E: Experiment>(
    exp: &E,
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
    estimates: &[Vec<Option<f64>>],
) -> std::io::Result<()> {
    let path = exp.summary_csv_path(name);
    let mut file = File::create(path)?;

    // title
    let mut row = vec!["t", "i", "a"];
    row.extend_from_slice(&<E::InputFactors as Factors>::factor_names());
    row.extend_from_slice(&<E::AlgFactors as Factors>::factor_names());
    row.push("time (ns)");
    file.write_all(row.join(",").as_bytes())?;
    file.write_all(b"\n")?;

    // rows
    for (i, (input_variant, input_estimates)) in input_levels.iter().zip(estimates).enumerate() {
        let input_factor_levels = input_variant.factor_levels();
        for (a, (alg_variant, estimate)) in alg_levels.iter().zip(input_estimates).enumerate() {
            let t = i * alg_levels.len() + a;
            let alg_factor_levels = alg_variant.factor_levels();
            let mut row = vec![
                (t + 1).to_string(),
                (i + 1).to_string(),
                (a + 1).to_string(),
            ];
            row.extend(input_factor_levels.iter().map(|x| x.to_string()));
            row.extend_from_slice(&alg_factor_levels);
            let estimate = estimate
                .map(|x| format!("{x:.0}"))
                .unwrap_or("NA".to_string());
            row.push(estimate);
            file.write_all(row.join(",").as_bytes())?;
            file.write_all(b"\n")?;
        }
    }
    Ok(())
}

fn print_summary_table<E: Experiment>(
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
    estimates: &[Vec<Option<f64>>],
) {
    let cmp = |a: &f64, b: &f64| match a.total_cmp(b) {
        Ordering::Equal => Ordering::Equal,
        ordering => ordering,
    };
    enum Rank {
        Best,
        Worst,
        Intermediate,
        Missing,
    }

    // header
    let mut header = vec![
        "t".cell().bold(true),
        "i".cell().bold(true),
        "a".cell().bold(true),
    ];
    for factor in <E::InputFactors as Factors>::factor_names() {
        header.push(factor.cell().bold(true));
    }
    for param in <E::AlgFactors as Factors>::factor_names() {
        header.push(param.cell().bold(true));
    }
    header.push("time (ns)".cell().bold(true).justify(Justify::Right));
    header.push("time per input".cell().bold(true));
    header.push("time overall".cell().bold(true));
    let num_columns = header.len();

    // cells
    let mut rows = vec![];
    let max_time_overall = estimates
        .iter()
        .flat_map(|input_estimates| input_estimates.iter())
        .flatten()
        .copied()
        .max_by(cmp)
        .unwrap_or(0.0);
    for (i, (input_variant, input_estimates)) in input_levels.iter().zip(estimates).enumerate() {
        if i > 0 {
            rows.push({
                let dash = |len: usize, justify: Justify| {
                    INPUT_SEPRATOR_CHAR
                        .repeat(len)
                        .cell()
                        .foreground_color(Some(Color::Rgb(255, 255, 102)))
                        .justify(justify)
                };
                let mut columns: Vec<_> = (0..3).map(|_| dash(1, Justify::Left)).collect(); // t i a
                columns.extend((0..(num_columns - 3 - 2 - 1)).map(|_| dash(3, Justify::Left))); // factors
                columns.push(dash(3, Justify::Right)); // time (ns)
                columns.extend((0..2).map(|_| dash(MAX_PROGRESS_BAR_WIDTH, Justify::Left))); // bars
                columns
            });
        }

        let values = || input_estimates.iter().map(|x| x.unwrap_or(f64::MAX));
        let min = values().min_by(cmp).unwrap_or(f64::MAX);
        let max = values().max_by(cmp).unwrap_or(f64::MIN);
        let max_time_per_input = input_estimates
            .iter()
            .flatten()
            .copied()
            .max_by(cmp)
            .unwrap_or(0.0);
        let rank_of = |estimate: &Option<f64>| match estimate {
            Some(x) => {
                if (min - x).abs() < 1e-5 {
                    Rank::Best
                } else if (max - x).abs() < 1e-5 {
                    Rank::Worst
                } else {
                    Rank::Intermediate
                }
            }
            None => Rank::Missing,
        };
        let cell_of = |rank: &Rank, cell: CellStruct| match rank {
            Rank::Best => cell.bold(true).foreground_color(Some(Color::Green)),
            Rank::Worst => cell.bold(true).foreground_color(Some(Color::Red)),
            Rank::Intermediate => cell,
            Rank::Missing => cell.foreground_color(Some(Color::Rgb(50, 50, 50))),
        };

        let input_factor_levels = input_variant.factor_levels();
        for (a, (alg_variant, estimate)) in alg_levels.iter().zip(input_estimates).enumerate() {
            let t = i * alg_levels.len() + a;
            let alg_factor_levels = alg_variant.factor_levels();
            let rank = rank_of(estimate);
            let time_bar_per_input = time_progress_bar(*estimate, max_time_per_input);
            let time_bar_overall = time_progress_bar(*estimate, max_time_overall);
            let estimate = estimate
                .map(|x| (x.round() as i128).separate_with_commas())
                .unwrap_or("NA".to_string());
            let mut columns = vec![
                cell_of(&rank, (t + 1).cell()),
                cell_of(&rank, (i + 1).cell()),
                cell_of(&rank, (a + 1).cell()),
            ];

            for x in input_factor_levels.iter().chain(&alg_factor_levels) {
                columns.push(cell_of(&rank, x.cell()));
            }
            columns.push(cell_of(&rank, estimate.cell().justify(Justify::Right)));
            columns.push(cell_of(&rank, time_bar_per_input.cell()));
            columns.push(cell_of(&rank, time_bar_overall.cell()));

            rows.push(columns);
        }
    }

    let table = rows.table().title(header);
    let log = format!("\n# {name}");
    println!("{}", log.bold().yellow());
    print_stdout(table).expect("Failed to print the summary table");
}

pub fn create_ai_prompt_to_analyze<E: Experiment>(
    exp: &E,
    name: &str,
    data: &[E::InputFactors],
    variants: &[E::AlgFactors],
) -> std::io::Result<()> {
    let path = exp.ai_prompt_path(name);
    let mut file = File::create(path)?;

    let summary_path = exp.summary_csv_path(name);
    let benchmark_path = exp.benchmark_file_path(name);
    let num_inputs = data.len();
    let input_factor_names = <E::InputFactors as Factors>::factor_names().join(", ");
    let num_variants = variants.len();
    let alg_factor_names = <E::AlgFactors as Factors>::factor_names().join(", ");
    let num_treatments = num_inputs * num_variants;

    let prompt = format!(
        r"
The file at '{summary_path:?}' contains the summary CSV for the '{name}' benchmark.

Each row is one treatment and includes:
- 't': treatment index
- 'i': input-data combination index
- 'a': algorithm-variant index
- input factor columns: {input_factor_names}
- algorithm factor columns: {alg_factor_names}
- 'time (ns)': execution time in nanoseconds (lower is better)

There are {num_inputs} input-data combinations, {num_variants} algorithm variants, and {num_treatments} total treatments.

Times are criterion point estimates, so treat them as benchmark summaries (not single raw run noise).

The benchmark source is at '{benchmark_path:?}'. It defines the input/algorithm factor types that explain the meaning of factor names and levels.

Provide a quick user-facing summary with:
1. the best overall algorithm variant or parameter setting,
2. whether the best choice is consistent across different inputs or depends on input characteristics,
3. the most important factor effects or tradeoffs,
4. notable interactions between input factors and algorithm factors,
5. a concise practical recommendation.

Keep the response concise and evidence-based. Prefer 4-8 bullet points.
Avoid unsupported causal claims; if confidence is low, state assumptions explicitly.
    "
    );

    file.write_all(prompt.as_bytes())?;
    Ok(())
}
