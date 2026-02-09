use crate::data::join;

/// A variant to execute a common task.
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
///   * One can optionally implement [`param_names_short`] to provide shorter versions of the names.
///     Short names are used to create keys of treatments.
///     It is important to provide them when there exist many data and variants because criterion crate limits each treatment key to 64 characters.
///
/// * `param_values` contains the values of the parameters of an instance of the variant.
///   They will be used to determine the variant to solve the problem.
///   * One can optionally implement [`param_values_short`] to provide shorter versions of the names due to key length limitation mentioned above.
///
/// [`param_names`]: Variant::param_names
/// [`param_names_short`]: Variant::param_names_short
/// [`param_values`]: Variant::param_values
/// [`param_values_short`]: Variant::param_values_short
///
/// # Examples - Categorical
///
/// Consider for instance the [two-sum problem](https://leetcode.com/problems/two-sum/description/).
/// As the storage, we may use a `HashMap` or `BTreeMap` for constant time lookups;
/// or a sorted `Vec` for logarithmic time lookups.
/// The algorithm can be parameterized over storage type as demonstrated in [`two_sum.rs`](https://github.com/orxfun/orx-parallel/blob/main/benches/two_sum.rs).
///
/// ```
/// use orx_criterion::*;
///
/// #[derive(Debug)]
/// enum StoreType {
///     None,
///     SortedVec,
///     HashMap,
///     BTreeMap,
/// }
///
/// struct SearchMethod(StoreType);
///
/// impl Variant for SearchMethod {
///     fn param_names() -> Vec<&'static str> {
///         vec!["store-type"]
///     }
///
///     fn param_values(&self) -> Vec<String> {
///         vec![format!("{:?}", self.0)]
///     }
/// }
/// ```
///
/// # Examples - Ordinal
///
/// A Dijkstra's shortest-path algorithm implementation might use a d-ary heap.
/// We can investigate the impact of `d` on the algorithm performance, as demonstrated in [`shortest_path.rs`](https://github.com/orxfun/orx-parallel/blob/main/benches/shortest_path.rs).
///
/// ```
/// use orx_criterion::*;
///
/// struct HeapWidth(usize);
///
/// impl Variant for HeapWidth {
///     fn param_names() -> Vec<&'static str> {
///         vec!["heap-width"]
///     }
///
///     fn param_values(&self) -> Vec<String> {
///         vec![self.0.to_string()]
///     }
/// }
/// ```
///
/// # Examples - Mixed
///
/// We might try to tune a collection of categorical and ordinal parameters at the same time.
/// Such an example is provided in [`find_element.rs`](https://github.com/orxfun/orx-parallel/blob/main/benches/find_element.rs) benchmark.
///
/// ```
/// use orx_criterion::*;
///
/// #[derive(Debug)]
/// enum ParLib {
///     Rayon,
///     OrxParallel,
/// }
///
/// #[derive(Debug, Clone, Copy)]
/// enum Approach {
///     Find,
///     Any,
/// }
///
/// struct SearchAlg {
///     par_lib: ParLib,
///     num_threads: usize,
///     chunk_size: usize,
///     approach: Approach,
/// }
///
/// impl Variant for SearchAlg {
///     fn param_names() -> Vec<&'static str> {
///         vec!["par_lib", "num_threads", "chunk_size", "approach"]
///     }
///
///     fn param_names_short() -> Vec<&'static str> {
///         vec!["lib", "nt", "ch", "app"]
///     }
///
///     fn param_values(&self) -> Vec<String> {
///         vec![
///             format!("{:?}", self.par_lib),
///             self.num_threads.to_string(),
///             self.chunk_size.to_string(),
///             format!("{:?}", self.approach),
///         ]
///     }
///
///     fn param_values_short(&self) -> Vec<String> {
///         vec![
///             match self.par_lib {
///                 ParLib::OrxParallel => "X",
///                 ParLib::Rayon => "R",
///             }
///             .to_string(),
///             self.num_threads.to_string(),
///             self.chunk_size.to_string(),
///             match self.approach {
///                 Approach::Find => "F",
///                 Approach::Any => "A",
///             }
///             .to_string(),
///         ]
///     }
/// }
/// ```
pub trait Variant {
    fn param_names() -> Vec<&'static str>;

    fn param_values(&self) -> Vec<String>;

    fn param_names_short() -> Vec<&'static str> {
        Self::param_names()
    }

    fn param_values_short(&self) -> Vec<String> {
        self.param_values()
    }

    fn to_str_long(&self) -> String {
        join(&Self::param_names(), &self.param_values())
    }

    fn to_str_short(&self) -> String {
        join(&Self::param_names_short(), &self.param_values_short())
    }
}
