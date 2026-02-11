use crate::experiment_sealed::ExperimentSealed;
use crate::summary::summarize;
use crate::{AlgFactors, InputFactors};
use colorize::AnsiColor;
use criterion::Criterion;
use std::fmt::Debug;

/// An experiment to analyze the impact of algorithm factors, or parameter settings, on solution time
/// over different data sets defined by input factors.
pub trait Experiment: Sized {
    /// Input factors of the experiment.
    /// Each instance of this type allows to create a particular input for the problem.
    type InputFactors: InputFactors;

    /// Algorithm factors to evaluate.
    /// Each instance of this type represents a variant of the algorithm.
    type AlgFactors: AlgFactors;

    /// Input of the problem.
    type Input;

    /// Output of the problem.
    type Output: PartialEq + Debug;

    /// Creates the input of the problem defined by the given `input_variant`.
    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input;

    fn execute(&mut self, alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output;

    fn expected_output(&self, _: &Self::InputFactors, _: &Self::Input) -> Option<Self::Output> {
        None
    }

    fn validate_output(&self, _: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}

    fn bench(
        &mut self,
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

            let input = self.input(input_variant);
            for (a, alg_variant) in alg_levels.iter().enumerate() {
                let a = a + 1;
                let idx = (i - 1) * num_a + a;
                let run_str = self.run_key_long(input_variant, alg_variant);
                let log = format!("\n### [{idx}/{num_t} || {a}/{num_a}]: {run_str}");
                println!("{}", log.green());

                let execution_name = self.run_key_short(input_variant, alg_variant);

                group.bench_with_input(&execution_name, &input, |b, input| {
                    let output = self.execute(alg_variant, input);
                    self.validate_output(input_variant, input, &output);
                    if let Some(expected_output) = self.expected_output(input_variant, input) {
                        assert_eq!(
                            output, expected_output,
                            "Output of run is not equal to expected output. Run: {run_str}",
                        );
                    }

                    b.iter(|| self.execute(alg_variant, input));
                });
            }
        }

        group.finish();

        summarize(self, name, input_levels, alg_levels);
    }
}
