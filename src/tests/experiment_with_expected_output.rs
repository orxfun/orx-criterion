use crate::experiment_sealed::ExperimentSealed;
use crate::{AlgFactors, Experiment, InputFactors};

struct MyData(usize, usize);

impl InputFactors for MyData {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "position"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.0.to_string(), self.1.to_string()]
    }
}

enum SearchMethod {
    Linear,
    Binary,
}

impl AlgFactors for SearchMethod {
    fn factor_names() -> Vec<&'static str> {
        vec!["search"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::Linear => "lin",
                Self::Binary => "bin",
            }
            .to_string(),
        ]
    }
}

struct SearchExperiment;

impl Experiment for SearchExperiment {
    type InputFactors = MyData;

    type AlgFactors = SearchMethod;

    type Input = (Vec<usize>, usize);

    type Output = Option<usize>;

    fn input(&mut self, datum: &Self::InputFactors) -> Self::Input {
        let vec: Vec<_> = (0..datum.0).collect();
        let value = *vec.get(datum.1).unwrap_or(&usize::MAX);
        (vec, value)
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let (vec, value) = input;
        Some(vec.iter().position(|x| x == value))
    }

    fn execute(&mut self, variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
        let (vec, value) = input;
        match variant {
            SearchMethod::Linear => vec.iter().position(|x| x == value),
            SearchMethod::Binary => vec.as_slice().binary_search(value).ok(),
        }
    }
}

#[test]
fn basic_experiment_with_expected_output() {
    let mut exp = SearchExperiment;
    let data = [MyData(4, 2), MyData(4, 5)];
    let variants = [SearchMethod::Linear, SearchMethod::Binary];

    let mut outputs = vec![];
    let mut names = vec![];
    for datum in &data {
        let input = exp.input(datum);
        for variant in &variants {
            names.push(exp.run_key_long(datum, variant));
            let output = exp.execute(variant, &input);
            if let Some(expected_output) = exp.expected_output(datum, &input) {
                assert_eq!(output, expected_output);
            }
            outputs.push(output);
        }
    }

    assert_eq!(outputs, vec![Some(2), Some(2), None, None]);

    assert_eq!(
        names,
        [
            "len:4_position:2/search:lin",
            "len:4_position:2/search:bin",
            "len:4_position:5/search:lin",
            "len:4_position:5/search:bin"
        ]
    );
}
