use crate::{Treatment, Variant};
use criterion::Criterion;

pub trait Experiment<const T: usize, const V: usize> {
    type Treatment: Treatment<T>;

    type Variant: Variant<V>;

    type Input;

    type Output;

    fn execution_to_string(treatment: &Self::Treatment, variant: &Self::Variant) -> String {
        format!("{}__{}", treatment.to_string(), variant.to_string())
    }

    fn input(treatment: &Self::Treatment) -> Self::Input;

    fn expected_output(_: &Self::Input) -> Option<Self::Output> {
        None
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output;

    fn bench(
        c: &mut Criterion,
        name: &str,
        treatments: &[Self::Treatment],
        variants: &[Self::Variant],
    ) {
        let mut group = c.benchmark_group(name);

        for treatment in treatments {
            let input = Self::input(treatment);
            for variant in variants {
                let execution_name = Self::execution_to_string(treatment, variant);

                group.bench_with_input(execution_name, &input, |b, input| {
                    // let output = Self::execute(variant, input);
                    b.iter(|| Self::execute(variant, input));
                });
            }
        }

        group.finish();
    }
}
