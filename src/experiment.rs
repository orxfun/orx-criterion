use crate::experiment_sealed::ExperimentSealed;
use crate::summary::summarize;
use crate::{AlgFactors, InputFactors};
use colorize::AnsiColor;
use criterion::Criterion;
use std::fmt::Debug;

/// An experiment to analyze the impact of algorithm factors, or parameter settings, on solution time
/// over different data sets defined by input factors.
///
/// # Examples
///
/// Consider the example algorithm defined in [`AlgFactors`] to find an element on an array, where we
/// want to experiment over inputs defined by the example in [`InputFactors`].
///
/// We can finally define our experiment using these input and algorithm factors.
///
/// ```
/// use orx_criterion::*;
///
/// // Input Factors
///
/// /// Position of the target value in the input array.
/// #[derive(Debug, Clone, Copy)]
/// enum ValuePosition {
///     /// The target value is located in the middle of the array.
///     Mid,
///     /// The target value does not exist in the array.
///     None,
/// }
///
/// /// Settings to define input of the search problem.
/// struct Settings {
///     /// Length of the input array.
///     len: usize,
///     /// Position of the target value inside the input array.
///     position: ValuePosition,
/// }
///
/// impl InputFactors for Settings {
///     fn factor_names() -> Vec<&'static str> {
///         vec!["len", "position"]
///     }
///
///     fn factor_levels(&self) -> Vec<String> {
///         vec![self.len.to_string(), format!("{:?}", self.position)]
///     }
///
///     fn factor_names_short() -> Vec<&'static str> {
///         vec!["l", "p"]
///     }
///
///     fn factor_levels_short(&self) -> Vec<String> {
///         let position = match self.position {
///             ValuePosition::Mid => "M",
///             ValuePosition::None => "X",
///         };
///         vec![self.len.to_string(), position.to_string()]
///     }
/// }
///
/// // Algorithm Factors
///
/// /// Defines the direction of the search for the target value.
/// #[derive(Debug, Clone, Copy)]
/// enum Direction {
///     /// The array will be search from beginning to the end.
///     Forwards,
///     /// The array will be search from end to the beginning.
///     Backwards,
/// }
///
/// /// Parameters defining the search algorithm.
/// struct Params {
///     /// Number of threads to use for the search.
///     num_threads: usize,
///     /// Direction of search by each thread.
///     direction: Direction,
/// }
///
/// impl AlgFactors for Params {
///     fn factor_names() -> Vec<&'static str> {
///         vec!["num_threads", "direction"]
///     }
///
///     fn factor_levels(&self) -> Vec<String> {
///         vec![
///             self.num_threads.to_string(),
///             format!("{:?}", self.direction),
///         ]
///     }
///
///     fn factor_names_short() -> Vec<&'static str> {
///         vec!["n", "d"]
///     }
///
///     fn factor_levels_short(&self) -> Vec<String> {
///         let direction = match self.direction {
///             Direction::Forwards => "F",
///             Direction::Backwards => "B",
///         };
///         vec![self.num_threads.to_string(), direction.to_string()]
///     }
/// }
///
/// // Experiment
///
/// /// Value to search for.
/// const SEARCH_VALUE: &str = "criterion";
///
/// struct Input {
///     array: Vec<String>,
///     position: Option<usize>, // to be used for validation
/// }
///
/// /// Experiment to carry out factorial analysis for searching a target value
/// /// within an array.
/// struct SearchExp;
///
/// impl Experiment for SearchExp {
///     type InputFactors = Settings;
///
///     type AlgFactors = Params;
///
///     type Input = Input;
///
///     type Output = Option<usize>;
///
///     fn input(&mut self, input_levels: &Self::InputFactors) -> Self::Input {
///         // we create an array with the given length, without the search value
///         let mut array: Vec<_> = (0..input_levels.len).map(|i| i.to_string()).collect();
///
///         // we decide on index of the search value depending on the position setting
///         let index = match input_levels.position {
///             ValuePosition::Mid => input_levels.len / 2,
///             ValuePosition::None => input_levels.len,
///         };
///
///         // we place the search value at the index
///         let position = match array.get_mut(index) {
///             Some(element) => {
///                 *element = SEARCH_VALUE.to_string();
///                 Some(index)
///             }
///             None => None,
///         };
///
///         Input { array, position }
///     }
///
///     fn execute(
///         &mut self,
///         alg_variant: &Self::AlgFactors,
///         input: &Self::Input,
///     ) -> Self::Output {
///         // notice that how we compute the output is determined by
///         // values of `alg_variant` fields.
///
///         let chunk_size = input.array.len() / alg_variant.num_threads;
///         let chunks: Vec<_> = input.array.chunks(chunk_size).collect();
///
///         std::thread::scope(|s| {
///             let mut handles = vec![];
///             let mut begin = 0;
///             for chunk in chunks {
///                 handles.push(s.spawn(move || {
///                     let mut iter = chunk.iter();
///
///                     match alg_variant.direction {
///                         Direction::Forwards => iter
///                             .position(|x| x.as_str() == SEARCH_VALUE)
///                             .map(|x| begin + x),
///                         Direction::Backwards => iter
///                             .rev()
///                             .position(|x| x.as_str() == SEARCH_VALUE)
///                             .map(|x| begin + (chunk.len() - 1 - x)),
///                     }
///                 }));
///                 begin += chunk.len();
///             }
///
///             // get the result from threads in the form of Some(position), if any
///             handles
///                 .into_iter()
///                 .map(|h| h.join().unwrap())
///                 .filter_map(|x| x)
///                 .next()
///         })
///     }
///
///     fn expected_output(
///         &self,
///         _settings: &Self::InputFactors,
///         input: &Self::Input,
///     ) -> Option<Self::Output> {
///         // we simply return the expected output cached in the input
///         Some(input.position)
///     }
///
///     fn validate_output(
///         &self,
///         _settings: &Self::InputFactors,
///         input: &Self::Input,
///         output: &Self::Output,
///     ) {
///         // additional validation logic just to make sure
///         // the linear search below does not affect results
///         match *output {
///             Some(position) => assert_eq!(input.array[position], SEARCH_VALUE),
///             None => assert!(!input.array.iter().any(|x| x.as_str() == SEARCH_VALUE)),
///         }
///     }
/// }
///
/// // demonstration of experimentation methods
///
/// let mut exp = SearchExp;
///
/// let input_variant = Settings {
///     len: 4,
///     position: ValuePosition::Mid,
/// };
/// let alg_variant = Params {
///     num_threads: 4,
///     direction: Direction::Backwards,
/// };
///
/// let input = exp.input(&input_variant);
/// assert_eq!(input.array, ["0", "1", "criterion", "3"]);
/// assert_eq!(input.position, Some(2));
///
/// let expected_output = exp.expected_output(&input_variant, &input);
/// assert_eq!(expected_output, Some(Some(2)));
///
/// let output = exp.execute(&alg_variant, &input);
/// assert_eq!(output, Some(2));
/// exp.validate_output(&input_variant, &input, &output);
/// ```
///
/// The example above demonstrates the behavior of the trait methods.
/// However, we would actually only use the [`bench`] function which internally makes use of
/// abovementioned methods.
/// The usage of the experiment is demonstrated below.
///
/// [`bench`]: crate::Experiment::bench
///
/// ```ignore
/// // input levels that we are interested in
/// let lengths = [1 << 10, 1 << 24];
/// let positions = [ValuePosition::Mid, ValuePosition::None];
/// let input_levels: Vec<_> = lengths
///     .into_iter()
///     .flat_map(|len| {
///         positions
///             .iter()
///             .copied()
///             .map(move |position| Settings { len, position })
///     })
///     .collect();
///
/// // algorithm variants that we want to evaluate
/// let num_threads = [1, 16];
/// let directions = [Direction::Forwards, Direction::Backwards];
/// let alg_levels: Vec<_> = num_threads
///     .into_iter()
///     .flat_map(|num_threads| {
///         directions.iter().copied().map(move |direction| Params {
///             num_threads,
///             direction,
///         })
///     })
///     .collect();
///
/// // execute a full-factorial experiment over the union of input and algorithm factors
/// SearchExp.bench(c, "tuning_example", &input_levels, &alg_levels);
/// ```
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
    /// This is the method that is being analyzed in this experiment.
    fn execute(&mut self, alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output;

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
    /// However, for such algorithms, we should not overwrite this method (it must return None).
    /// On the other hand, we can still use more flexible [`validate_output`] method if needed.
    ///
    /// Finally note that, validation tests are executed only once per (input, algorithm) combination, the validation
    /// time is not included in the analysis, and hence, it does not impact the analysis.
    ///
    /// [`validate_output`]: crate::Experiment::validate_output
    fn expected_output(&self, _: &Self::InputFactors, _: &Self::Input) -> Option<Self::Output> {
        None
    }

    /// Performs additional validation for the output created by any one of the algorithm variants for the given input.
    ///
    /// Default implementation is an empty method which does nothing.
    /// It can be overwritten to add assertions on the expected characteristics of the output.
    ///
    /// Note that, validation tests are executed only once per (input, algorithm) combination, the validation
    /// time is not included in the analysis, and hence, it does not impact the analysis.
    fn validate_output(&self, _: &Self::InputFactors, _: &Self::Input, _: &Self::Output) {}

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
