use crate::experiment_sealed::ExperimentSealed;
use crate::{Experiment, Factors};
use std::fs::File;
use std::io::Write;

pub fn create_summary_csv<E: Experiment>(
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
