use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Treatment, Variant};

struct Treat(usize, usize);

impl Treatment<2> for Treat {
    fn factor_names() -> [&'static str; 2] {
        ["len", "position"]
    }

    fn factor_values(&self) -> [String; 2] {
        [self.0.to_string(), self.1.to_string()]
    }
}

enum SearchMethod {
    Linear,
    Binary,
}

impl Variant<1> for SearchMethod {
    fn param_names() -> [&'static str; 1] {
        ["search"]
    }

    fn param_values(&self) -> [String; 1] {
        [match self {
            Self::Linear => "lin",
            Self::Binary => "bin",
        }
        .to_string()]
    }
}

struct SearchExperiment;

impl Experiment<2, 1> for SearchExperiment {
    type Treatment = Treat;

    type Variant = SearchMethod;

    type Input = (Vec<usize>, usize);

    type Output = Option<usize>;

    fn input(treatment: &Self::Treatment) -> Self::Input {
        let vec: Vec<_> = (0..(100 * treatment.0)).collect();
        let value = *vec.get(100 * treatment.1).unwrap_or(&usize::MAX);
        (vec, value)
    }

    fn expected_output(input: &Self::Input) -> Option<Self::Output> {
        let (vec, value) = input;
        Some(vec.iter().position(|x| x == value))
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        let (vec, value) = input;
        match variant {
            SearchMethod::Linear => vec.iter().position(|x| x == value),
            SearchMethod::Binary => vec.as_slice().binary_search(value).ok(),
        }
    }
}

fn run(c: &mut Criterion) {
    let treatments = [
        Treat(1 << 10, 1 << 9),
        // Treat(1 << 20, 1 << 19),
        // Treat(1 << 20, 1 << 21),
    ];
    let variants = [SearchMethod::Linear, SearchMethod::Binary];

    SearchExperiment::bench(c, "my_test_bench", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
