use crate::{Treatment, Variant};

pub trait Experiment<const T: usize, const V: usize> {
    type Treatment: Treatment<T>;

    type Variant: Variant<V>;

    type Input;

    type Output;

    fn execution_to_string(treatment: &Self::Treatment, variant: &Self::Variant) -> String {
        format!("{}__{}", treatment.to_string(), variant.to_string())
    }

    fn input(treatment: &Self::Treatment) -> Self::Input;

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output;
}
