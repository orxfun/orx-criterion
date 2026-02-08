use crate::treatment::to_string;

pub trait Variant {
    fn param_names() -> Vec<&'static str>;

    fn param_values(&self) -> Vec<String>;

    fn param_names_short() -> Vec<&'static str> {
        Self::param_names()
    }

    fn param_values_short(&self) -> Vec<String> {
        self.param_values()
    }

    fn to_string(&self) -> String {
        to_string(&Self::param_names(), &self.param_values())
    }
}
