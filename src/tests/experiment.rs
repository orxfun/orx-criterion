use crate::{Experiment, Treatment, Variant};

pub struct MyTreat1(usize);

impl Treatment for MyTreat1 {
    fn factor_names() -> Vec<&'static str> {
        vec!["width"]
    }

    fn factor_values(&self) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

pub struct MyVariant2 {
    len: usize,
    sort: bool,
}

impl Variant for MyVariant2 {
    fn param_names() -> Vec<&'static str> {
        vec!["len", "sort"]
    }

    fn param_values(&self) -> Vec<String> {
        vec![self.len.to_string(), self.sort.to_string()]
    }
}

pub struct MyExperiment;

impl Experiment for MyExperiment {
    type Treatment = MyTreat1;

    type Variant = MyVariant2;

    type Input = Vec<usize>;

    type Output = Vec<usize>;

    fn input(treatment: &Self::Treatment) -> Self::Input {
        (0..treatment.0).collect()
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        let mut output = input.clone();
        if variant.sort {
            for _ in 0..variant.len {
                for i in 1..output.len() {
                    output.swap(i, i - 1);
                }
            }
        }
        output
    }
}

#[test]
fn basic_experiment() {
    let treatments = [MyTreat1(2), MyTreat1(5)];
    let variants = [
        MyVariant2 {
            len: 1001,
            sort: false,
        },
        MyVariant2 {
            len: 1001,
            sort: true,
        },
    ];

    let mut outputs = vec![];
    let mut names = vec![];
    for treatment in &treatments {
        let input = MyExperiment::input(treatment);
        for variant in &variants {
            names.push(MyExperiment::execution_to_string(treatment, variant));
            outputs.push(MyExperiment::execute(variant, &input));
        }
    }

    assert_eq!(
        outputs,
        [
            vec![0, 1],
            vec![1, 0],
            vec![0, 1, 2, 3, 4],
            vec![1, 2, 3, 4, 0],
        ]
    );

    assert_eq!(
        names,
        [
            "width:2/len:1001_sort:false",
            "width:2/len:1001_sort:true",
            "width:5/len:1001_sort:false",
            "width:5/len:1001_sort:true"
        ]
    );
}

#[test]
fn experiment_bench() {
    let treatments = [MyTreat1(2), MyTreat1(5)];
    let variants = [
        MyVariant2 {
            len: 1001,
            sort: false,
        },
        MyVariant2 {
            len: 1001,
            sort: true,
        },
    ];

    let mut outputs = vec![];
    let mut names = vec![];
    for treatment in &treatments {
        let input = MyExperiment::input(treatment);
        for variant in &variants {
            names.push(MyExperiment::execution_to_string(treatment, variant));
            outputs.push(MyExperiment::execute(variant, &input));
        }
    }

    assert_eq!(
        outputs,
        [
            vec![0, 1],
            vec![1, 0],
            vec![0, 1, 2, 3, 4],
            vec![1, 2, 3, 4, 0],
        ]
    );

    assert_eq!(
        names,
        [
            "width:2/len:1001_sort:false",
            "width:2/len:1001_sort:true",
            "width:5/len:1001_sort:false",
            "width:5/len:1001_sort:true"
        ]
    );
}
