use crate::treatment::to_string;

pub trait Variant {
    fn param_names() -> Vec<&'static str>;

    fn param_values(&self) -> Vec<String>;

    fn to_string(&self) -> String {
        to_string(&Self::param_names(), &self.param_values())
    }
}
