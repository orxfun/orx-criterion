use crate::{Experiment, Treatment, Variant};
use cli_table::{Cell, CellStruct, Color, Style, Table, format::Justify, print_stdout};
use std::fs::File;
use std::io::{Read, Write};
use std::{cmp::Ordering, path::PathBuf};

pub fn summarize<E: Experiment>(name: &str, treatments: &[E::Treatment], variants: &[E::Variant]) {
    let estimates = collect_point_estimates::<E>(name, treatments, variants);
    create_summary_csv::<E>(name, treatments, variants, &estimates)
        .expect("Failed to create csv summary");
    println!(
        "\nSummary table created at:\n{}\n",
        E::summary_csv_path(name).to_str().unwrap()
    );

    print_summary_table::<E>(name, treatments, variants, &estimates);
}

fn create_summary_csv<E: Experiment>(
    name: &str,
    treatments: &[E::Treatment],
    variants: &[E::Variant],
    estimates: &[Vec<Option<f64>>],
) -> std::io::Result<()> {
    let path = E::summary_csv_path(name);
    let mut file = File::create(path)?;

    // title
    let mut row = vec![];
    row.extend_from_slice(&<E::Treatment as Treatment>::factor_names());
    row.extend_from_slice(&<E::Variant as Variant>::param_names());
    row.push("Time (ns)");
    file.write(row.join(",").as_bytes())?;
    file.write(b"\n")?;

    // rows
    for (treatment, treatment_estimates) in treatments.iter().zip(estimates) {
        let factor_values = treatment.factor_values();
        for (variant, estimate) in variants.iter().zip(treatment_estimates) {
            let param_values = variant.param_values();
            let mut row = vec![];
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
    treatments: &[E::Treatment],
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
    let mut title = vec![];
    for factor in <E::Treatment as Treatment>::factor_names() {
        title.push(factor.cell().bold(true));
    }
    for param in <E::Variant as Variant>::param_names() {
        title.push(param.cell().bold(true));
    }
    title.push("Time (ns)".cell().bold(true).justify(Justify::Right));

    // cells
    let mut rows = vec![];
    for (treatment, treatment_estimates) in treatments.iter().zip(estimates) {
        let values = || treatment_estimates.iter().map(|x| x.unwrap_or(f64::MAX));
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

        let factor_values = treatment.factor_values();
        for (variant, estimate) in variants.iter().zip(treatment_estimates) {
            let mut columns = vec![];
            let param_values = variant.param_values();
            let rank = rank_of(estimate);
            let estimate = estimate
                .map(|x| format!("{x:.0}"))
                .unwrap_or("NA".to_string());

            for x in factor_values.iter().chain(&param_values) {
                columns.push(cell_of(&rank, x.cell()));
            }
            columns.push(cell_of(&rank, estimate.cell().justify(Justify::Right)));

            rows.push(columns);
        }
    }

    let table = rows.table().title(title);
    println!("# {name}");
    print_stdout(table).expect("Failed to print the summary table");
}

fn collect_point_estimates<E: Experiment>(
    name: &str,
    treatments: &[E::Treatment],
    variants: &[E::Variant],
) -> Vec<Vec<Option<f64>>> {
    treatments
        .iter()
        .map(|treatment| {
            variants
                .iter()
                .map(|variant| {
                    let execution_path = E::execution_estimates_path(name, treatment, variant);
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

    let field_slope = "\"slope\"";
    let position = contents.find(field_slope)?;
    let begin = position + field_slope.len();
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
