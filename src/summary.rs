use crate::Experiment;
use cli_table::{Table, format::Justify};
use std::{fs::File, io::Read, path::PathBuf};

pub fn print_summary_table<const T: usize, const V: usize, E: Experiment<T, V>>(
    name: &str,
    treatments: &[E::Treatment],
    variants: &[E::Variant],
) {
    let estimates = collect_point_estimates::<_, _, E>(name, treatments, variants);
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
