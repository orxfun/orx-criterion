use crate::{Experiment, Treatment, Variant};

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
        let vec: Vec<_> = (0..treatment.0).collect();
        let value = *vec.get(treatment.1).unwrap_or(&usize::MAX);
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

#[test]
fn basic_experiment_with_expected_output() {
    let treatments = [Treat(4, 2), Treat(4, 5)];
    let variants = [SearchMethod::Linear, SearchMethod::Binary];

    let mut outputs = vec![];
    let mut names = vec![];
    for treatment in &treatments {
        let input = SearchExperiment::input(treatment);
        for variant in &variants {
            names.push(SearchExperiment::execution_to_string(treatment, variant));
            let output = SearchExperiment::execute(variant, &input);
            if let Some(expected_output) = SearchExperiment::expected_output(&input) {
                assert_eq!(output, expected_output);
            }
            outputs.push(output);
        }
    }

    assert_eq!(outputs, vec![Some(2), Some(2), None, None]);

    assert_eq!(
        names,
        [
            "len:4_position:2__search:lin",
            "len:4_position:2__search:bin",
            "len:4_position:5__search:lin",
            "len:4_position:5__search:bin"
        ]
    );
}
