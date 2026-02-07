use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Treatment, Variant};

pub struct MyTreat1(usize);

impl Treatment<1> for MyTreat1 {
    fn factor_names() -> [&'static str; 1] {
        ["width"]
    }

    fn factor_values(&self) -> [String; 1] {
        [self.0.to_string()]
    }
}

pub struct MyVariant2 {
    repeat: usize,
    sort: bool,
}

impl Variant<2> for MyVariant2 {
    fn param_names() -> [&'static str; 2] {
        ["repeat", "sort"]
    }

    fn param_values(&self) -> [String; 2] {
        [self.repeat.to_string(), self.sort.to_string()]
    }
}

pub struct MyExperiment;

impl Experiment<1, 2> for MyExperiment {
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
            for _ in 0..variant.repeat {
                for i in 1..output.len() {
                    output.swap(i, i - 1);
                }
            }
        }
        output
    }
}

fn run(c: &mut Criterion) {
    let treatments = [MyTreat1(1 << 10), MyTreat1(1 << 20)];
    let variants = [
        MyVariant2 {
            repeat: 42,
            sort: false,
        },
        MyVariant2 {
            repeat: 42,
            sort: true,
        },
    ];

    MyExperiment::bench(c, "my_test_bench", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
