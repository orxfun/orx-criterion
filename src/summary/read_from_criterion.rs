use crate::Experiment;
use crate::experiment_sealed::ExperimentSealed;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn collect_point_estimates<E: Experiment>(
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
