use crate::data::join;

/// A variant to execute a common task.
///
/// Each variant can be uniquely determined by the combination of its parameter values.
/// 
/// These parameters might have categorical or ordinal values.
/// Consider for instance Dijkstra's algorithm to solve the single-source shortest path problem.
/// 
/// * we may solve 
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
