/// Factors defining an experimentation input.
///
/// Each input setting can be uniquely determined by the combination of its factor values.
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
///   They will be used to create the input that will be used in the experimentation.
///   Similarly, [`factor_levels_short`] can optionally be implemented.
///
/// Note that four of the methods (`factor_names`, `factor_levels`, and short versions) must return vectors of the same
/// length with elements matching in order.
///
/// For demonstration benchmarks, please see the [benches](https://github.com/orxfun/orx-parallel/blob/main/benches) folder.
///
/// [`factor_names`]: InputFactors::factor_names
/// [`factor_names_short`]: InputFactors::factor_names_short
/// [`factor_levels`]: InputFactors::factor_levels
/// [`factor_levels_short`]: InputFactors::factor_levels_short
///
/// # Examples
///
/// Consider for instance a problem where we will search an element within an array.
///
/// The input to this problem is determined by the length of the array and position of the element
/// that we will search for.
///
/// In this case, `"len"` and `"position"` would be the factor names.
///
/// And combination of values of these factor would determine the input to the experimentation.
///
/// ```
/// use orx_criterion::*;
///
/// #[derive(Debug)]
/// enum ValuePosition {
///     Beg,
///     Mid,
///     End,
/// }
///
/// struct Settings {
///     len: usize,
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
/// }
///
/// let settings = Settings {
///     len: 1024,
///     position: ValuePosition::Mid,
/// };
///
/// assert_eq!(settings.key_long(), "len:1024_position:Mid");
/// assert_eq!(settings.key_short(), "len:1024_position:Mid");
/// ```
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
/// enum ValuePosition {
///     Beg,
///     Mid,
///     End,
/// }
///
/// struct Settings {
///     len: usize,
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
///             ValuePosition::Beg => "B",
///             ValuePosition::Mid => "M",
///             ValuePosition::End => "E",
///         };
///         vec![self.len.to_string(), position.to_string()]
///     }
/// }
///
/// let settings = Settings {
///     len: 1024,
///     position: ValuePosition::Mid,
/// };
///
/// assert_eq!(settings.key_long(), "len:1024_position:Mid");
/// assert_eq!(settings.key_short(), "l:1024_p:M");
/// ```
pub trait InputFactors {
    /// Names (long) of settings of the input.
    ///
    /// The long factor names are used:
    ///
    /// * in criterion benchmark run logs, and
    /// * as column headers of summary tables.
    ///
    /// Further, unless [`factor_names_short`] is explicitly implemented,
    /// they are used to create the unique keys of inputs.
    ///
    /// [`factor_names_short`]: AlgFactors::factor_names_short
    fn factor_names() -> Vec<&'static str>;

    /// String representation of values (long) of setting values (levels) of the
    /// input.
    fn factor_levels(&self) -> Vec<String>;

    /// Short names of settings of the input.
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

    /// String representation of values (short) of setting values (levels) of the
    /// input.
    fn factor_levels_short(&self) -> Vec<String> {
        self.factor_levels()
    }

    /// Key of the input created by joining results of `factor_names` and `factor_levels`.
    ///
    /// It uniquely identifies the input.
    fn key_long(&self) -> String {
        join(&Self::factor_names(), &self.factor_levels())
    }

    /// Short key of the input created by joining results of `factor_names_short` and `factor_levels_short`.
    ///
    /// It uniquely identifies the input.
    fn key_short(&self) -> String {
        join(&Self::factor_names_short(), &self.factor_levels_short())
    }
}

pub(super) fn join(names: &[&'static str], values: &[String]) -> String {
    debug_assert_eq!(names.len(), values.len());
    match names.len() {
        0 => Default::default(),
        1 => format!("{}:{}", names[0], values[0]),
        n => {
            let mut s = String::new();
            s.push_str(&format!("{}:{}", names[0], values[0]));
            for i in 1..n {
                s.push_str(&format!("_{}:{}", names[i], values[i]));
            }
            s
        }
    }
}
