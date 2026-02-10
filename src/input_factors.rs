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
    fn factor_names() -> Vec<&'static str>;

    fn factor_levels(&self) -> Vec<String>;

    fn factor_names_short() -> Vec<&'static str> {
        Self::factor_names()
    }

    fn factor_levels_short(&self) -> Vec<String> {
        self.factor_levels()
    }

    fn key_long(&self) -> String {
        join(&Self::factor_names(), &self.factor_levels())
    }

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
