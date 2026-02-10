use crate::summary::summarize;
use crate::{AlgFactors, InputFactors};
use colorize::AnsiColor;
use criterion::Criterion;
use std::fmt::Debug;
use std::path::PathBuf;

pub trait Experiment: Sized {
    type InputFactors: InputFactors;

    type AlgFactors: AlgFactors;

    type Input;

    type Output: PartialEq + Debug;

    fn run_key_long(input_variant: &Self::InputFactors, alg_variant: &Self::AlgFactors) -> String {
        format!("{}/{}", input_variant.key_long(), alg_variant.key_long())
    }

    fn run_key_short(input_variant: &Self::InputFactors, alg_variant: &Self::AlgFactors) -> String {
        format!("{}/{}", input_variant.key_short(), alg_variant.key_short())
    }

    fn run_estimates_path(
        bench_name: &str,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
    ) -> PathBuf {
        let execution_path = Self::run_key_short(input_variant, alg_variant)
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

    fn ai_prompt_path(bench_name: &str) -> PathBuf {
        [
            "target",
            "criterion",
            bench_name,
            &format!("prompt_{bench_name}.md"),
        ]
        .iter()
        .collect()
    }

    fn input(data: &Self::InputFactors) -> Self::Input;

    fn expected_output(_: &Self::InputFactors, _: &Self::Input) -> Option<Self::Output> {
        None
    }

    fn validate_output(_: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}

    fn execute(alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output;

    fn bench(
        c: &mut Criterion,
        name: &str,
        input_levels: &[Self::InputFactors],
        alg_levels: &[Self::AlgFactors],
    ) {
        let num_i = input_levels.len();
        let num_a = alg_levels.len();
        let num_t = input_levels.len() * alg_levels.len();

        let log = format!(
            "\n\n\n# {name} benchmarks with {num_i} data points and {num_a} variants => {num_t} treatments"
        );
        println!("{}", log.bold().underlined());

        let mut group = c.benchmark_group(name);
        for (i, input_variant) in input_levels.iter().enumerate() {
            let datum_str = input_variant.key_long();
            let i = i + 1;
            let log = format!("\n\n\n\n\n## Data point [{i}/{num_i}]: {datum_str}");
            println!("{}", log.yellow().bold());

            let input = Self::input(input_variant);
            for (a, alg_variant) in alg_levels.iter().enumerate() {
                let a = a + 1;
                let idx = (i - 1) * num_a + a;
                let run_str = Self::run_key_long(input_variant, alg_variant);
                let log = format!("\n### [{idx}/{num_t} || {a}/{num_a}]: {run_str}");
                println!("{}", log.green());

                let execution_name = Self::run_key_short(input_variant, alg_variant);

                group.bench_with_input(&execution_name, &input, |b, input| {
                    let output = Self::execute(alg_variant, input);
                    Self::validate_output(input_variant, input, &output);
                    if let Some(expected_output) = Self::expected_output(input_variant, input) {
                        assert_eq!(
                            output, expected_output,
                            "Output of run is not equal to expected output. Run: {run_str}",
                        );
                    }

                    b.iter(|| Self::execute(alg_variant, input));
                });
            }
        }

        group.finish();

        summarize::<Self>(name, input_levels, alg_levels);
    }
}
