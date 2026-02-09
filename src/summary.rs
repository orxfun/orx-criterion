use crate::{Data, Experiment, Variant};
use cli_table::{Cell, CellStruct, Color, Style, Table, format::Justify, print_stdout};
use colorize::AnsiColor;
use std::fs::File;
use std::io::{Read, Write};
use std::{cmp::Ordering, path::PathBuf};

fn collect_point_estimates<E: Experiment>(
    name: &str,
    data: &[E::Data],
    variants: &[E::Variant],
) -> Vec<Vec<Option<f64>>> {
    data.iter()
        .map(|datum| {
            variants
                .iter()
                .map(|variant| {
                    let execution_path = E::run_estimates_path(name, datum, variant);
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

pub fn summarize<E: Experiment>(name: &str, data: &[E::Data], variants: &[E::Variant]) {
    let estimates = collect_point_estimates::<E>(name, data, variants);

    create_summary_csv::<E>(name, data, variants, &estimates)
        .expect("Failed to create csv summary");

    let log = format!(
        "\nSummary table created at:\n{}\n",
        E::summary_csv_path(name).to_str().unwrap()
    );
    println!("{}", log.italic());

    print_summary_table::<E>(name, data, variants, &estimates);

    create_ai_prompt_to_analyze::<E>(name, data, variants).expect("Failed to create ai prompt");
}

fn create_summary_csv<E: Experiment>(
    name: &str,
    data: &[E::Data],
    variants: &[E::Variant],
    estimates: &[Vec<Option<f64>>],
) -> std::io::Result<()> {
    let path = E::summary_csv_path(name);
    let mut file = File::create(path)?;

    // title
    let mut row = vec!["t", "d", "v"];
    row.extend_from_slice(&<E::Data as Data>::factor_names());
    row.extend_from_slice(&<E::Variant as Variant>::param_names());
    row.push("Time (ns)");
    file.write(row.join(",").as_bytes())?;
    file.write(b"\n")?;

    // rows
    for (d, (datum, datum_estimates)) in data.iter().zip(estimates).enumerate() {
        let factor_values = datum.factor_values();
        for (v, (variant, estimate)) in variants.iter().zip(datum_estimates).enumerate() {
            let t = d * variants.len() + v;
            let param_values = variant.param_values();
            let mut row = vec![
                (t + 1).to_string(),
                (d + 1).to_string(),
                (v + 1).to_string(),
            ];
            row.extend(factor_values.iter().map(|x| x.to_string()));
            row.extend_from_slice(&param_values);
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
    data: &[E::Data],
    variants: &[E::Variant],
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
        "d".cell().bold(true),
        "v".cell().bold(true),
    ];
    for factor in <E::Data as Data>::factor_names() {
        title.push(factor.cell().bold(true));
    }
    for param in <E::Variant as Variant>::param_names() {
        title.push(param.cell().bold(true));
    }
    title.push("Time (ns)".cell().bold(true).justify(Justify::Right));

    // cells
    let mut rows = vec![];
    for (d, (datum, datum_estimates)) in data.iter().zip(estimates).enumerate() {
        let values = || datum_estimates.iter().map(|x| x.unwrap_or(f64::MAX));
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

        let factor_values = datum.factor_values();
        for (v, (variant, estimate)) in variants.iter().zip(datum_estimates).enumerate() {
            let t = d * variants.len() + v;
            let param_values = variant.param_values();
            let rank = rank_of(estimate);
            let estimate = estimate
                .map(|x| format!("{x:.0}"))
                .unwrap_or("NA".to_string());
            let mut columns = vec![
                cell_of(&rank, (t + 1).cell()),
                cell_of(&rank, (d + 1).cell()),
                cell_of(&rank, (v + 1).cell()),
            ];

            for x in factor_values.iter().chain(&param_values) {
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
    data: &[E::Data],
    variants: &[E::Variant],
) -> std::io::Result<()> {
    let path = E::ai_prompt_path(name);
    let mut file = File::create(path)?;

    let summary_path = E::summary_csv_path(name);
    let num_data = data.len();
    let factor_names = <E::Data as Data>::factor_names().join(", ");
    let num_variants = variants.len();
    let param_names = <E::Variant as Variant>::param_names().join(", ");
    let num_treatments = num_data * num_variants;

    let prompt = format!(
        r"
The file at '{summary_path:?}' is the output of a full-factorial experiment for the '{name}' benchmark.

The experiment is applied on {num_data} data sets.
Each data set is defined by combination of values of factors '{factor_names}'.
Each data set, or combination, gets a unique index specified in column 'd'.

Problem of each data set is solved by {num_variants} variants.
Each variant is defined by combination of values of parameters '{param_names}'.
Each data set, or combination, gets a unique index specified in column 'v'.

In total, there exist {num_treatments} treatments as a unique combination of data settings and variant parameters.
Each treatment gets a unique index specified in column 't'.

The objective is to solve the problem as fast as possible.
In other words, we want to minimize elapsed time.
We are searching the best values of the parameters, or best variant, that would perform the best across different data sets.

Please analyze the output of the experiment and provide insights.
    "
    );

    file.write(prompt.as_bytes())?;
    Ok(())
}
