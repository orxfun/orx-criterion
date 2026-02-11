use crate::{AlgFactors, Experiment, InputFactors};

pub struct MyData(usize);

impl InputFactors for MyData {
    fn factor_names() -> Vec<&'static str> {
        vec!["width"]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["w"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.0.to_string()]
    }
}

pub struct MyVariant {
    len: usize,
    sort: bool,
}

impl AlgFactors for MyVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "sort"]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["l", "s"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.len.to_string(), self.sort.to_string()]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        vec![
            self.len.to_string(),
            match self.sort {
                true => "T",
                false => "F",
            }
            .to_string(),
        ]
    }
}

pub struct MyExperiment;

impl Experiment for MyExperiment {
    type InputFactors = MyData;

    type AlgFactors = MyVariant;

    type Input = Vec<usize>;

    type Output = Vec<usize>;

    fn input(&mut self, data: &Self::InputFactors) -> Self::Input {
        (0..data.0).collect()
    }

    fn execute(&mut self, variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
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
    let mut exp = MyExperiment;
    let data = [MyData(2), MyData(5)];
    let variants = [
        MyVariant {
            len: 1001,
            sort: false,
        },
        MyVariant {
            len: 1001,
            sort: true,
        },
    ];

    let mut outputs = vec![];
    let mut names = vec![];
    let mut names_short = vec![];
    for datum in &data {
        let input = exp.input(datum);
        for variant in &variants {
            names.push(exp.run_key_long(datum, variant));
            names_short.push(exp.run_key_short(datum, variant));
            outputs.push(exp.execute(variant, &input));
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

    assert_eq!(
        names_short,
        [
            "w:2/l:1001_s:F",
            "w:2/l:1001_s:T",
            "w:5/l:1001_s:F",
            "w:5/l:1001_s:T"
        ]
    );
}
