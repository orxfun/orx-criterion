use crate::data::join;

/// An algorithm variant to execute a common task.
///
/// Each variant can be uniquely determined by the combination of its parameter values.
///
/// These parameters might have categorical or ordinal values.
///
/// # Required Methods
///
/// We must implement two methods: [`param_names`] and [`param_values`]:
///
/// * `param_names` contains the names of parameters.
///   They will be used as column names of output table, or within logs.
///   One can optionally implement [`param_names_short`] to provide shorter versions of the names
///   (please see the corresponding example below).
///
/// * `param_values` contains the values of the parameters of an instance of the variant.
///   They will be used to determine the variant to solve the problem.
///   Similarly, [`param_values_short`] can optionally be implemented.
///
/// For demonstration benchmarks, please see the [benches](https://github.com/orxfun/orx-parallel/blob/main/benches) folder.
///
/// [`param_names`]: Variant::param_names
/// [`param_names_short`]: Variant::param_names_short
/// [`param_values`]: Variant::param_values
/// [`param_values_short`]: Variant::param_values_short
///
/// # Examples
///
/// Consider for instance a parallelized algorithm which can be executed using
/// different number threads.
///
/// Further assume that we can search forwards or backwards.
///
/// In this case, `"num_threads"` and `"direction"` would be the parameter names.
///
/// And combination of values of these parameters would determine how the algorithm would execute.
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
/// struct AlgParams {
///     num_threads: usize,
///     direction: Direction,
/// }
///
/// impl Variant for AlgParams {
///     fn param_names() -> Vec<&'static str> {
///         vec!["num_threads", "direction"]
///     }
///
///     fn param_values(&self) -> Vec<String> {
///         vec![
///             self.num_threads.to_string(),
///             format!("{:?}", self.direction),
///         ]
///     }
/// }
///
/// let alg_params = AlgParams {
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
/// Importantly note that, `param_values` must be implemented in a way that each combination
/// leads to a **unique key** by the [`key_long`] call.
///
/// This is often correct by default conversion to string.
///
/// Further notice that [`key_long`] and [`key_short`] returns the same key since we have not
/// implemented the optional shorter versions for this example.
///
/// [`key_long`]: Variant::key_long
/// [`key_short`]: Variant::key_short
///
/// # Examples - Optional Short Names and Values
///
/// In order to shorten the
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
/// struct AlgParams {
///     num_threads: usize,
///     direction: Direction,
/// }
///
/// impl Variant for AlgParams {
///     fn param_names() -> Vec<&'static str> {
///         vec!["num_threads", "direction"]
///     }
///
///     fn param_values(&self) -> Vec<String> {
///         vec![
///             self.num_threads.to_string(),
///             format!("{:?}", self.direction),
///         ]
///     }
///
///     fn param_names_short() -> Vec<&'static str> {
///         vec!["n", "d"]
///     }
///
///     fn param_values_short(&self) -> Vec<String> {
///         let direction = match self.direction {
///             Direction::Forwards => "F",
///             Direction::Backwards => "B",
///         };
///         vec![self.num_threads.to_string(), direction.to_string()]
///     }
/// }
///
/// let alg_params = AlgParams {
///     num_threads: 4,
///     direction: Direction::Backwards,
/// };
///
/// assert_eq!(alg_params.key_long(), "num_threads:4_direction:Backwards");
/// assert_eq!(alg_params.key_short(), "n:4_d:B");
/// ```
pub trait Variant {
    /// Names (long) of parameters of the algorithm variant.
    ///
    /// The long parameter names are used:
    ///
    /// * in criterion benchmark run logs, and
    /// * as column headers of summary tables.
    ///
    /// Unless, [`param_names_short`] is explicitly implemented,
    /// they will also be used in the unique key of the benchmark run.
    /// Otherwise, short names will be used in the key due to 64 character limit.
    ///
    /// [`param_names_short`]: Variant::param_names_short
    fn param_names() -> Vec<&'static str>;

    ///
    fn param_values(&self) -> Vec<String>;

    fn param_names_short() -> Vec<&'static str> {
        Self::param_names()
    }

    fn param_values_short(&self) -> Vec<String> {
        self.param_values()
    }

    fn key_long(&self) -> String {
        join(&Self::param_names(), &self.param_values())
    }

    fn key_short(&self) -> String {
        join(&Self::param_names_short(), &self.param_values_short())
    }
}
