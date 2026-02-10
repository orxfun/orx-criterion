use crate::{AlgFactors, Experiment, InputFactors};
use cli_table::{Cell, CellStruct, Color, Style, Table, format::Justify, print_stdout};
use colorize::AnsiColor;
use std::fs::File;
use std::io::{Read, Write};
use std::{cmp::Ordering, path::PathBuf};

fn collect_point_estimates<E: Experiment>(
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
                    let execution_path = E::run_estimates_path(name, input_variant, alg_variant);
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
    let is_slope_null = contents.find(field_slope_null).is_some();

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

pub fn summarize<E: Experiment>(name: &str, input_levels: &[E::InputFactors], alg_levels: &[E::AlgFactors]) {
    let estimates = collect_point_estimates::<E>(name, input_levels, alg_levels);

    create_summary_csv::<E>(name, input_levels, alg_levels, &estimates)
        .expect("Failed to create csv summary");

    let log = format!(
        "\nSummary table created at:\n{}\n",
        E::summary_csv_path(name).to_str().unwrap()
    );
    println!("{}", log.italic());

    print_summary_table::<E>(name, input_levels, alg_levels, &estimates);

    create_ai_prompt_to_analyze::<E>(name, input_levels, alg_levels)
        .expect("Failed to create ai prompt");
    let log = format!(
        "\nA draft AI prompt to analyze the summary table is created at:\n{}\n",
        E::ai_prompt_path(name).to_str().unwrap()
    );
    println!("{}", log.italic());
}

fn create_summary_csv<E: Experiment>(
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
    estimates: &[Vec<Option<f64>>],
) -> std::io::Result<()> {
    let path = E::summary_csv_path(name);
    let mut file = File::create(path)?;

    // title
    let mut row = vec!["t", "i", "a"];
    row.extend_from_slice(&<E::InputFactors as InputFactors>::factor_names());
    row.extend_from_slice(&<E::AlgFactors as AlgFactors>::factor_names());
    row.push("Time (ns)");
    file.write(row.join(",").as_bytes())?;
    file.write(b"\n")?;

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
            file.write(row.join(",").as_bytes())?;
            file.write(b"\n")?;
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
    let cmp = |a: &f64, b: &f64| match a < b {
        true => Ordering::Less,
        false => Ordering::Greater,
    };
    enum Rank {
        Best,
        Worst,
        Intermediate,
        Missing,
    }

    // title
    let mut title = vec![
        "t".cell().bold(true),
        "i".cell().bold(true),
        "a".cell().bold(true),
    ];
    for factor in <E::InputFactors as InputFactors>::factor_names() {
        title.push(factor.cell().bold(true));
    }
    for param in <E::AlgFactors as AlgFactors>::factor_names() {
        title.push(param.cell().bold(true));
    }
    title.push("Time (ns)".cell().bold(true).justify(Justify::Right));

    // cells
    let mut rows = vec![];
    for (i, (input_variant, input_estimates)) in input_levels.iter().zip(estimates).enumerate() {
        let values = || input_estimates.iter().map(|x| x.unwrap_or(f64::MAX));
        let min = values().min_by(cmp).unwrap_or(f64::MAX);
        let max = values().max_by(cmp).unwrap_or(f64::MIN);
        let rank_of = |estimate: &Option<f64>| match estimate {
            Some(x) => {
                if (min - x).abs() < 1e-5 {
                    return Rank::Best;
                } else if (max - x).abs() < 1e-5 {
                    return Rank::Worst;
                } else {
                    return Rank::Intermediate;
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
            let estimate = estimate
                .map(|x| format!("{x:.0}"))
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

            rows.push(columns);
        }
    }

    let table = rows.table().title(title);
    let log = format!("\n# {name}");
    println!("{}", log.bold().yellow());
    print_stdout(table).expect("Failed to print the summary table");
}

pub fn create_ai_prompt_to_analyze<E: Experiment>(
    name: &str,
    data: &[E::InputFactors],
    variants: &[E::AlgFactors],
) -> std::io::Result<()> {
    let path = E::ai_prompt_path(name);
    let mut file = File::create(path)?;

    let summary_path = E::summary_csv_path(name);
    let num_inputs = data.len();
    let input_factor_names = <E::InputFactors as InputFactors>::factor_names().join(", ");
    let num_variants = variants.len();
    let alg_factor_names = <E::AlgFactors as AlgFactors>::factor_names().join(", ");
    let num_treatments = num_inputs * num_variants;

    let prompt = format!(
        r"
The file at '{summary_path:?}' is the output of a full-factorial experiment for the '{name}' benchmark.

The experiment is applied on {num_inputs} data sets.
Each data set is defined by combination of values of factors '{input_factor_names}'.
Each data set, or combination, gets a unique index specified in column 'i'.

Problem of each data set is solved by {num_variants} algorithm variants.
Each variant is defined by combination of values of parameters '{alg_factor_names}'.
Each algorithm variant gets a unique index specified in column 'a'.

In total, there exist {num_treatments} treatments as unique combinations of input data settings and algorithm variant parameters.
Each treatment gets a unique index specified in column 't'.

The response variable is the time.
Although we have a single value per treatment, these values are obtained by the 'criterion' crate which runs sufficiently large number of repetitions to obtain these point estimates.

The objective is to solve the problem as fast as possible.
In other words, we want to minimize elapsed time.
We are searching the best values of the parameters, or best variant, that would perform the best across different data sets.

Please analyze the output of the experiment and provide insights.
    "
    );

    file.write(prompt.as_bytes())?;
    Ok(())
}
