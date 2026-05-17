use crate::Factors;
use crate::experiment_sealed::ExperimentSealed;
use crate::summary::summarize;
use colorize::AnsiColor;
use criterion::Criterion;
use std::fmt::Debug;

/// An experiment to analyze the impact of algorithm factors, or parameter settings, on solution time
/// over different data sets defined by input factors.
///
/// While defining an experiment, you need to:
///
/// * provide associated types of input factors, algorithm variants, input and output of a run,
/// * implement [`input`] method which defines how to create the inputs of runs, and
/// * implement [`execute`] method which defines how to compute the output depending on the algorithm variant.
///
/// This is sufficient to run the experiment.
///
/// You may optionally implement [`expected_output`] and [`validate_output`] methods to validate the outputs
/// of experimental runs per each input. The default implementations of these methods assume that outputs
/// are always valid. Benchmarks do not aim to test behavior, but it might also be useful to add another
/// validation when desired.
///
/// [`input`]: Experiment::input
/// [`execute`]: Experiment::execute
/// [`expected_output`]: Experiment::expected_output
/// [`validate_output`]: Experiment::validate_output
///
/// # Examples
///
/// Consider the minimal sorting experiment from the README.
///
/// ```ignore
/// use orx_criterion::*;
/// use criterion::{Criterion, criterion_group, criterion_main};
///
/// #[derive(Debug, Clone, Copy)]
/// enum Dist {
///     Random,
///     Desc,
/// }
///
/// // these are our input variants
/// struct InputCfg {
///     len: usize,
///     dist: Dist,
/// }
///
/// impl Factors for InputCfg {
///     fn factor_names() -> Vec<&'static str> {
///         vec!["len", "dist"]
///     }
///
///     fn factor_levels(&self) -> Vec<String> {
///         vec![self.len.to_string(), format!("{:?}", self.dist)]
///     }
/// }
///
/// // these area our algorithm variants defining different ways to compute
/// #[derive(Debug, Clone, Copy)]
/// enum Alg {
///     StdSort,
///     StdSortUnstable,
/// }
///
/// impl Factors for Alg {
///     fn factor_names() -> Vec<&'static str> {
///         vec!["alg"]
///     }
///
///     fn factor_levels(&self) -> Vec<String> {
///         vec![format!("{:?}", self)]
///     }
/// }
///
/// struct SortExp;
///
/// impl Experiment for SortExp {
///     type InputFactors = InputCfg;   // input variants
///     type AlgFactors = Alg;          // algorithm/computation variants
///     type Input = Vec<u32>;          // we will provide an array
///     type Output = Vec<u32>;         // we will receive a sorted array
///
///     // we define how to create the input of an experiment for the given input variant
///     // this method is not timed / benchmarked!
///     fn input(&mut self, levels: &Self::InputFactors) -> Self::Input {
///         match levels.dist {
///             Dist::Desc => (0..levels.len as u32).rev().collect(),
///             Dist::Random => (0..levels.len as u32).collect(),
///         }
///     }
///
///     // we define how to compute output from the input with the given algorithm variant
///     // this is the only method that is timed / benchmarked!
///     fn execute(
///         &mut self,
///         _: &Self::InputFactors,
///         alg: &Self::AlgFactors,
///         input: &Self::Input,
///     ) -> Self::Output {
///         let mut v = input.clone();
///         match alg {
///             Alg::StdSort => v.sort(),
///             Alg::StdSortUnstable => v.sort_unstable(),
///         }
///         v
///     }
///
///     // just for additional validation, will be checked once per input & algorithm
///     // also not timed / benchmarked!
///     fn expected_output(
///         &self,
///         _: &Self::InputFactors,
///         input: &Self::Input,
///     ) -> Option<Self::Output> {
///         let mut expected = input.clone();
///         expected.sort();
///         Some(expected)
///     }
/// }
///
/// // to run the experiment, we simply:
/// // - define input variants we want to test
/// // - algorithm variants to compare
/// // - and call the `bench` method of our experiment.
/// fn run(c: &mut Criterion) {
///     let input_levels = vec![
///         InputCfg {
///             len: 64,
///             dist: Dist::Random,
///         },
///         InputCfg {
///             len: 64,
///             dist: Dist::Desc,
///         },
///         InputCfg {
///             len: 1024,
///             dist: Dist::Random,
///         },
///         InputCfg {
///             len: 1024,
///             dist: Dist::Desc,
///         },
///     ];
///
///     let alg_variants = vec![Alg::StdSort, Alg::StdSortUnstable];
///
///     SortExp.bench(c, "sorting_minimal", &input_levels, &alg_variants);
/// }
///
/// criterion_group!(benches, run);
/// criterion_main!(benches);
/// ```
pub trait Experiment: Sized {
    /// Input factors of the experiment.
    /// Each instance of this type allows to create a particular input for the problem.
    type InputFactors: Factors;

    /// Algorithm factors to evaluate.
    /// Each instance of this type represents a variant of the algorithm.
    type AlgFactors: Factors;

    /// Input of the problem.
    type Input;

    /// Output of the problem.
    type Output: PartialEq + Debug;

    /// Creates the input of the problem defined by the given `input_variant`.
    ///
    /// Note that [`bench`] function will call `input` only once per input variant.
    /// Assuming that the inputs might be expensive to create or store, this approach aims to create each
    /// input only once and use it for all algorithm variants.
    ///
    /// Furthermore, the time required to create the inputs is not included in the analysis, and hence,
    /// does not affect the result of the experiment.
    ///
    /// [`bench`]: crate::Experiment::bench
    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input;

    /// Executes the algorithm or task defined by the given `alg_variant` on the `input`, and returns the
    /// output.
    ///
    /// This is the method that is being timed, benchmarked and analyzed.
    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output;

    /// Returns the expected output that the `execute` must produce for the given input factor levels and input
    /// created for these factor levels.
    ///
    /// Default implementation returns `None`, in which case, this validation is skipped.
    /// When the method is overwritten and returns `Some(X)`, output of the `execute` method must also return `X`
    /// for the given input.
    ///
    /// Note that this validation test assumes that `execute` is pure in the sense that it deterministically creates
    /// the same output every time it is called with the same input, regardless of the algorithm variant.
    /// In other words, all algorithm variants must produce the same output for a given input.
    ///
    /// We can still analyze non-deterministic algorithms with this crate.
    /// However, for such algorithms, we should not overwrite this method (it must return None),
    /// and we can use [`validate_output`] method instead.
    ///
    /// Finally note that, validation tests are executed only once per (input, algorithm) combination, the validation
    /// time is not included in the analysis, and hence, it does not impact the analysis.
    ///
    /// [`validate_output`]: crate::Experiment::validate_output
    fn expected_output(
        &self,
        _input_factors: &Self::InputFactors,
        _input: &Self::Input,
    ) -> Option<Self::Output> {
        None
    }

    /// Performs additional validation for the output created by any one of the algorithm variants for the given input.
    ///
    /// Default implementation is an empty method which does nothing.
    /// It can be overwritten to add assertions on the expected characteristics of the output.
    ///
    /// Note that, validation tests are executed only once per (input, algorithm) combination, the validation
    /// time is not included in the analysis, and hence, it does not impact the analysis.
    fn validate_output(
        &self,
        _input_factors: &Self::InputFactors,
        _input: &Self::Input,
        _output: &Self::Output,
    ) {
    }

    /// Executes the experiment using criterion (`c`) benchmarks.
    ///
    /// Each combination of `input_levels` and `alg_levels` will be benchmarked.
    ///
    /// At the end of the criterion benchmark run, summary tables will be created to enable factorial analysis.
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
                    let output = self.execute(input_variant, alg_variant, input);
                    self.validate_output(input_variant, input, &output);
                    if let Some(expected_output) = self.expected_output(input_variant, input) {
                        assert_eq!(
                            output, expected_output,
                            "Output of run is not equal to expected output. Run: {run_str}",
                        );
                    }

                    b.iter(|| self.execute(input_variant, alg_variant, input));
                });
            }
        }

        group.finish();

        summarize(self, name, input_levels, alg_levels);
    }
}
