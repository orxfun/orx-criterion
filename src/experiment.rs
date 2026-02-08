use crate::summary::summarize;
use crate::{Treatment, Variant};
use criterion::Criterion;
use std::fmt::Debug;
use std::path::PathBuf;

pub trait Experiment<const T: usize, const V: usize>: Sized {
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

    fn summary_csv_path(bench_name: &str) -> PathBuf {
        [
            "target",
            "criterion",
            bench_name,
            &format!("summary_{bench_name}.csv"),
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

        summarize::<_, _, Self>(name, treatments, variants);
    }
}
