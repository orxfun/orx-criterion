use crate::input_factors::join;

/// Factors defining an algorithm variant.
///
/// Each variant can be uniquely determined by the combination of its factor values.
///
/// These parameters might have categorical or ordinal values.
///
/// # Required Methods
///
/// We must implement two methods: [`factor_names`] and [`factor_levels`]:
///
/// * `factor_names` contains the names of parameters.
///   They will be used as column names of output table, or within logs.
///   One can optionally implement [`factor_names_short`] to provide shorter versions of the names
///   (please see the corresponding example below).
///
/// * `factor_levels` contains the values of the parameters of an instance of the variant.
///   They will be used to determine the variant to solve the problem.
///   Similarly, [`factor_levels_short`] can optionally be implemented.
///
/// Note that four of the methods (`factor_names`, `factor_levels`, and short versions) must return vectors of the same
/// length with elements matching in order.
///
/// For demonstration benchmarks, please see the [benches](https://github.com/orxfun/orx-parallel/blob/main/benches) folder.
///
/// [`factor_names`]: AlgFactors::factor_names
/// [`factor_names_short`]: AlgFactors::factor_names_short
/// [`factor_levels`]: AlgFactors::factor_levels
/// [`factor_levels_short`]: AlgFactors::factor_levels_short
///
/// # Examples
///
/// Consider for instance a parallelized algorithm which can be executed using
/// different number threads.
///
/// Further assume that we can search forwards or backwards.
///
/// In this case, `"num_threads"` and `"direction"` would be the factor names.
///
/// And combination of values of these factor would determine how the algorithm would execute.
///
/// ```
/// use orx_criterion::*;
///
/// #[derive(Debug)]
/// enum Direction {
///     Forwards,
///     Backwards,
/// }
///
/// struct Params {
///     num_threads: usize,
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
/// }
///
/// let alg_params = Params {
///     num_threads: 1,
///     direction: Direction::Forwards,
/// };
///
/// assert_eq!(alg_params.key_long(), "num_threads:1_direction:Forwards");
/// assert_eq!(
///     alg_params.key_short(),
///     "num_threads:1_direction:Forwards"
/// );
/// ```
///
/// Importantly note that, `factor_levels` must be implemented in a way that each combination
/// leads to a **unique key** by the [`key_long`] call.
///
/// This is often correct by default conversion to string.
///
/// Further notice that [`key_long`] and [`key_short`] returns the same key since we have not
/// implemented the optional shorter versions for this example.
///
/// [`key_long`]: AlgFactors::key_long
/// [`key_short`]: AlgFactors::key_short
///
/// # Examples - Optional Short Names and Values
///
/// In some cases, we need a short version of the unique key.
/// This is due to the fact that criterion limits the result folder names (practically the keys) to 64 characters.
/// The short names and values are used to create the short keys to be used as the folder names,
/// while reports and summaries will still be created by the long and human-friendly names.
///
/// It is important to make sure that short keys still uniquely define a combination of the variant,
/// as demonstrated in the following example.
///
/// ```
/// use orx_criterion::*;
///
/// #[derive(Debug)]
/// enum Direction {
///     Forwards,
///     Backwards,
/// }
///
/// struct Params {
///     num_threads: usize,
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
/// let alg_params = Params {
///     num_threads: 4,
///     direction: Direction::Backwards,
/// };
///
/// assert_eq!(alg_params.key_long(), "num_threads:4_direction:Backwards");
/// assert_eq!(alg_params.key_short(), "n:4_d:B");
/// ```
pub trait AlgFactors {
    /// Names (long) of parameters of the algorithm variant.
    ///
    /// The long parameter names are used:
    ///
    /// * in criterion benchmark run logs, and
    /// * as column headers of summary tables.
    ///
    /// Further, unless [`factor_names_short`] is explicitly implemented,
    /// they are used to create the unique keys of algorithm variants.
    ///
    /// [`factor_names_short`]: AlgFactors::factor_names_short
    fn factor_names() -> Vec<&'static str>;

    /// String representation of values (long) of parameter values (levels) of the
    /// algorithm variant.
    fn factor_levels(&self) -> Vec<String>;

    /// Short names of parameters of the algorithm variant.
    ///
    /// Default implementation returns the result of [`factor_names`].
    ///
    /// The short versions are implemented to shorten the keys which is necessary
    /// when working with very long keys (exceeding 64 characters).
    ///
    /// [`factor_names`]: AlgFactors::factor_names
    fn factor_names_short() -> Vec<&'static str> {
        Self::factor_names()
    }

    /// String representation of values (short) of parameter values (levels) of the
    /// algorithm variant.
    fn factor_levels_short(&self) -> Vec<String> {
        self.factor_levels()
    }

    /// Key of the algorithm variant created by joining results of `factor_names` and `factor_levels`.
    ///
    /// It uniquely identifies the algorithm variant.
    fn key_long(&self) -> String {
        join(&Self::factor_names(), &self.factor_levels())
    }

    /// Short key of the algorithm variant created by joining results of `factor_names_short` and `factor_levels_short`.
    ///
    /// It uniquely identifies the algorithm variant.
    fn key_short(&self) -> String {
        join(&Self::factor_names_short(), &self.factor_levels_short())
    }
}
