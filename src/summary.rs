use crate::{Experiment, Treatment, Variant};
use cli_table::{Cell, Style, Table, format::Justify, print_stdout};
use std::{fs::File, io::Read, path::PathBuf};

pub fn print_summary_table<const T: usize, const V: usize, E: Experiment<T, V>>(
    name: &str,
    treatments: &[E::Treatment],
    variants: &[E::Variant],
) {
    let all_estimates = collect_point_estimates::<_, _, E>(name, treatments, variants);
    println!("all_estimates = {all_estimates:?}");

    // title
    let mut title = vec![];
    for factor in <E::Treatment as Treatment<_>>::factor_names() {
        title.push(factor.cell().bold(true));
    }
    for param in <E::Variant as Variant<_>>::param_names() {
        title.push(param.cell().bold(true));
    }
    title.push("Time (ns)".cell().bold(true).justify(Justify::Right));

    // cells
    let mut rows = vec![];
    for (treatment, estimates) in treatments.iter().zip(&all_estimates) {
        println!("estimates = {estimates:?}");
        let factor_values = treatment.factor_values();
        for (variant, estimate) in variants.iter().zip(estimates) {
            let mut columns = vec![];
            let param_values = variant.param_values();
            let estimate = estimate.map(|x| x.to_string()).unwrap_or("NA".to_string());

            for x in factor_values.iter().chain(&param_values) {
                columns.push(x.cell());
            }

            columns.push(estimate.cell().justify(Justify::Right));
            rows.push(columns);
        }
    }

    let table = rows.table().title(title);
    print_stdout(table).expect("Failed to print the summary table");
}

fn collect_point_estimates<const T: usize, const V: usize, E: Experiment<T, V>>(
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
