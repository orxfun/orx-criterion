use crate::summary::summarize;
use crate::{Data, Variant};
use criterion::Criterion;
use std::fmt::Debug;
use std::path::PathBuf;

pub trait Experiment: Sized {
    type Data: Data;

    type Variant: Variant;

    type Input;

    type Output: PartialEq + Debug;

    fn run_key_long(data: &Self::Data, variant: &Self::Variant) -> String {
        format!("{}/{}", data.to_str_long(), variant.to_str_long())
    }

    fn run_key_short(data: &Self::Data, variant: &Self::Variant) -> String {
        format!("{}/{}", data.to_str_short(), variant.to_str_short())
    }

    fn run_estimates_path(bench_name: &str, data: &Self::Data, variant: &Self::Variant) -> PathBuf {
        let execution_path = Self::run_key_short(data, variant)
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

    fn input(data: &Self::Data) -> Self::Input;

    fn expected_output(_: &Self::Input) -> Option<Self::Output> {
        None
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output;

    fn bench(c: &mut Criterion, name: &str, data: &[Self::Data], variants: &[Self::Variant]) {
        let num_runs = data.len() * variants.len();
        println!(
            "\n\n    # {name} benchmarks with {} data points and {} variants => {} treatments\n",
            data.len(),
            variants.len(),
            num_runs
        );

        let mut group = c.benchmark_group(name);
        for (t, datum) in data.iter().enumerate() {
            println!(
                "\n\n    ## Data [{} / {}]: {}",
                t + 1,
                data.len(),
                datum.to_str_long()
            );

            let input = Self::input(datum);
            for variant in variants {
                let execution_name = Self::run_key_short(datum, variant);

                group.bench_with_input(&execution_name, &input, |b, input| {
                    if let Some(expected_output) = Self::expected_output(input) {
                        let output = Self::execute(variant, input);
                        assert_eq!(
                            output,
                            expected_output,
                            "Output of run is not equal to expected output. Run: {}",
                            Self::run_key_long(datum, variant)
                        );
                    }

                    b.iter(|| Self::execute(variant, input));
                });
            }
        }

        group.finish();

        summarize::<Self>(name, data, variants);
    }
}
