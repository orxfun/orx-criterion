use crate::treatment::to_string;

pub trait Variant<const N: usize> {
    fn param_names() -> [&'static str; N];

    fn param_values(&self) -> [String; N];

    fn to_string(&self) -> String {
        to_string(&Self::param_names(), &self.param_values())
    }
}
