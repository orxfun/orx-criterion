use crate::{Treatment, Variant};
use criterion::Criterion;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub trait Experiment<const T: usize, const V: usize> {
    type Treatment: Treatment<T>;

    type Variant: Variant<V>;

    type Input;

    type Output: PartialEq + Debug;

    fn execution_to_string(treatment: &Self::Treatment, variant: &Self::Variant) -> String {
        format!("{}/{}", treatment.to_string(), variant.to_string())
    }

    fn execution_estimates_path(
        bench_name: &str,
        treatment: &Self::Treatment,
        variant: &Self::Variant,
    ) -> PathBuf {
        let execution_path = Self::execution_to_string(treatment, variant)
            .replace("/", "_")
            .replace(":", "_");
        [
            "target",
            "criterion",
            bench_name,
            &execution_path,
            "new",
            "estimates.json",
        ]
        .iter()
        .collect()
    }

    fn input(treatment: &Self::Treatment) -> Self::Input;

    fn expected_output(_: &Self::Input) -> Option<Self::Output> {
        None
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output;

    fn bench(
        c: &mut Criterion,
        name: &str,
        treatments: &[Self::Treatment],
        variants: &[Self::Variant],
    ) {
        let num_runs = treatments.len() * variants.len();
        println!(
            "\n\n    # {name} benchmarks with {} treatments and {} variants => {} executions\n",
            treatments.len(),
            variants.len(),
            num_runs
        );

        let mut group = c.benchmark_group(name);
        let mut names_paths = vec![];
        for (t, treatment) in treatments.iter().enumerate() {
            println!(
                "\n\n    ## Treatment [{} / {}]: {}",
                t + 1,
                treatments.len(),
                treatment.to_string()
            );

            let input = Self::input(treatment);
            for variant in variants {
                let execution_name = Self::execution_to_string(treatment, variant);
                let execution_path = Self::execution_estimates_path(name, treatment, variant);
                names_paths.push((execution_name.clone(), execution_path));

                group.bench_with_input(&execution_name, &input, |b, input| {
                    if let Some(expected_output) = Self::expected_output(input) {
                        let output = Self::execute(variant, input);
                        assert_eq!(
                            output, expected_output,
                            "Output of execution '{execution_name}' is not equal to expected output."
                        );
                    }

                    b.iter(|| Self::execute(variant, input));
                });
            }
        }

        group.finish();

        for (name, path) in &names_paths {
            let x = get_slope_point_estimate(path);
            println!("xxxxxxxxxxxxxxxxxxx => {x:?}");
            // println!("path = {path:?}");
            // let mut file = File::open(path).unwrap();
            // let mut contents = String::new();
            // file.read_to_string(&mut contents).unwrap();

            // let x = contents.find("\"slope\"");
            // println!("xxxxxxxxxxxxx = {x:?}");

            // println!("\n\n\n{name}\n{contents}\n\n\n");
        }
    }
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
