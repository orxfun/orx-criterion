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
        let num_d = data.len();
        let num_v = variants.len();
        let num_t = data.len() * variants.len();

        println!(
            "\n\n\n# {name} benchmarks with {num_d} data points and {num_v} variants => {num_t} treatments",
        );

        let mut group = c.benchmark_group(name);
        for (d, datum) in data.iter().enumerate() {
            let datum_str = datum.to_str_long();
            let d = d + 1;
            println!("\n\n\n\n\n## Data [{d}/{num_d}]: {datum_str}");

            let input = Self::input(datum);
            for (v, variant) in variants.iter().enumerate() {
                let v = v + 1;
                let idx = (d - 1) * num_v + v;
                let run_str = Self::run_key_long(datum, variant);
                println!("\n### [{idx}/{num_t} || {v}/{num_v}]: {run_str}");

                let execution_name = Self::run_key_short(datum, variant);

                group.bench_with_input(&execution_name, &input, |b, input| {
                    if let Some(expected_output) = Self::expected_output(input) {
                        let output = Self::execute(variant, input);
                        assert_eq!(
                            output, expected_output,
                            "Output of run is not equal to expected output. Run: {run_str}",
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
