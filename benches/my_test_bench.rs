use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{AlgFactors, InputFactors, Experiment};

struct SortData(usize, usize);

impl InputFactors for SortData {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "position"]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["l", "p"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.0.to_string(), self.1.to_string()]
    }
}

#[derive(Debug)]
enum SearchMethod {
    Linear,
    LinearBackwards,
    Binary,
}

impl AlgFactors for SearchMethod {
    fn factor_names() -> Vec<&'static str> {
        vec!["search"]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["s"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("{self:?}")]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        vec![
            match self {
                Self::Linear => "lin",
                Self::LinearBackwards => "lin-bwd",
                Self::Binary => "bin",
            }
            .to_string(),
        ]
    }
}

struct SearchExperiment;

impl Experiment for SearchExperiment {
    type Data = SortData;

    type Variant = SearchMethod;

    type Input = (Vec<usize>, usize);

    type Output = Option<usize>;

    fn input(datum: &Self::Data) -> Self::Input {
        let vec: Vec<_> = (0..(100 * datum.0)).collect();
        let value = *vec.get(100 * datum.1).unwrap_or(&usize::MAX);
        (vec, value)
    }

    fn expected_output(_: &Self::Data, input: &Self::Input) -> Option<Self::Output> {
        let (vec, value) = input;
        Some(vec.iter().position(|x| x == value))
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        let (vec, value) = input;
        match variant {
            SearchMethod::Linear => vec.iter().position(|x| x == value),
            SearchMethod::LinearBackwards => vec
                .iter()
                .rev()
                .position(|x| x == value)
                .map(|x| vec.len() - x - 1),
            SearchMethod::Binary => vec.as_slice().binary_search(value).ok(),
        }
    }
}

fn run(c: &mut Criterion) {
    let data = [
        SortData(1 << 5, 1 << 10),
        SortData(1 << 10, 1 << 9),
        SortData(1 << 20, 1 << 21),
    ];
    let variants = [
        SearchMethod::Linear,
        SearchMethod::LinearBackwards,
        SearchMethod::Binary,
    ];

    SearchExperiment::bench(c, "my_test_bench", &data, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
