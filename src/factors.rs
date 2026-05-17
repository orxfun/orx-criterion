/// Factors that are used to define an input variant and an algorithm variant of the experiment.
///
/// Each variant setting can be uniquely determined by the combination of its factor values.
///
/// These parameters might have categorical or ordinal values.
///
/// # Required Methods
///
/// We must implement two methods: [`factor_names`] and [`factor_levels`]:
///
/// * `factor_names` contains the names of parameters.
///   They will be used as in output table and within logs.
///   One can optionally implement [`factor_names_short`] to provide shorter versions of the names
///   (see the corresponding example below).
///
/// * `factor_levels` contains values of the parameters of an instance of the variant.
///   They will be used to create the variant that will be used in the experimentation.
///   Similarly, [`factor_levels_short`] can optionally be implemented.
///
/// Note that four of the methods (`factor_names`, `factor_levels`, and short versions) must return vectors of the same
/// length with elements matching in order.
///
/// For demonstration benchmarks, please see the [benches](https://github.com/orxfun/orx-criterion/blob/main/benches) folder.
///
/// [`factor_names`]: Factors::factor_names
/// [`factor_names_short`]: Factors::factor_names_short
/// [`factor_levels`]: Factors::factor_levels
/// [`factor_levels_short`]: Factors::factor_levels_short
///
/// # Examples
///
/// Consider the sorting experiment from the README.
/// The input to this problem is determined by the length of the array and the distribution
/// used to generate it.
///
/// In this case, `"len"` and `"dist"` are the factor names, and their combined values
/// uniquely define an input variant.
///
/// Notice that we also implement `Factors` for `Alg` to define algorithm variants.
///
/// ```
/// use orx_criterion::*;
///
/// #[derive(Debug, Clone, Copy)]
/// enum Dist {
///     Random,
///     Desc,
/// }
///
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
/// let cfg = InputCfg { len: 64, dist: Dist::Random };
///
/// assert_eq!(cfg.key_long(), "len:64_dist:Random");
/// assert_eq!(cfg.key_short(), "len:64_dist:Random");
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
/// #[derive(Debug, Clone, Copy)]
/// enum Dist {
///     Random,
///     Desc,
/// }
///
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
///
///     fn factor_names_short() -> Vec<&'static str> {
///         vec!["l", "d"]
///     }
///
///     fn factor_levels_short(&self) -> Vec<String> {
///         let dist = match self.dist {
///             Dist::Random => "Rnd",
///             Dist::Desc => "Dsc",
///         };
///         vec![self.len.to_string(), dist.to_string()]
///     }
/// }
///
/// let cfg = InputCfg { len: 64, dist: Dist::Random };
///
/// assert_eq!(cfg.key_long(), "len:64_dist:Random");
/// assert_eq!(cfg.key_short(), "l:64_d:Rnd");
/// ```
pub trait Factors {
    /// Factor names.
    ///
    /// The long factor names are used:
    ///
    /// * in criterion benchmark run logs, and
    /// * as column headers of summary tables.
    ///
    /// Further, unless [`factor_names_short`] is explicitly implemented,
    /// they are used to create the unique keys of variants.
    ///
    /// [`factor_names_short`]: Factors::factor_names_short
    fn factor_names() -> Vec<&'static str>;

    /// String representation of values of factors levels of the variant.
    fn factor_levels(&self) -> Vec<String>;

    /// Shortened versions of the factor names.
    ///
    /// Default implementation returns the result of [`factor_names`].
    ///
    /// The short versions are implemented to shorten the keys which is necessary
    /// when working with very long keys (exceeding 64 characters).
    ///
    /// [`factor_names`]: Factors::factor_names
    fn factor_names_short() -> Vec<&'static str> {
        Self::factor_names()
    }

    /// Shortened string representation of values of factors levels of the variant.
    fn factor_levels_short(&self) -> Vec<String> {
        self.factor_levels()
    }

    /// Key of the variant created by joining results of `factor_names` and `factor_levels`.
    ///
    /// It uniquely identifies the variant.
    fn key_long(&self) -> String {
        join(&Self::factor_names(), &self.factor_levels())
    }

    /// Short key of the variant created by joining results of `factor_names_short` and `factor_levels_short`.
    ///
    /// It uniquely identifies the variant.
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
