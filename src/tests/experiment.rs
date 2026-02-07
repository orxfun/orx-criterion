use crate::{Experiment, Treatment, Variant};

pub struct MyTreat1(usize);

impl Treatment<1> for MyTreat1 {
    fn factor_names() -> [&'static str; 1] {
        ["width"]
    }

    fn factor_values(&self) -> [String; 1] {
        [self.0.to_string()]
    }
}

pub struct MyVariant3 {
    len: usize,
    sort: bool,
    split: char,
}

impl Variant<3> for MyVariant3 {
    fn param_names() -> [&'static str; 3] {
        ["len", "sort", "split"]
    }

    fn param_values(&self) -> [String; 3] {
        [
            self.len.to_string(),
            self.sort.to_string(),
            self.split.to_string(),
        ]
    }
}

pub struct MyExperiment;

impl Experiment<1, 3> for MyExperiment {
    type Treatment = MyTreat1;

    type Variant = MyVariant3;

    type Input = Vec<usize>;

    type Output = Vec<usize>;

    fn input(treatment: &Self::Treatment) -> Self::Input {
        (0..treatment.0).collect()
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        let mut output = input.clone();
        if variant.sort {
            for i in 1..variant.len {
                output.swap(i, i - 1);
            }
        }
        output
    }
}
