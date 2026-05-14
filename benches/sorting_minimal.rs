use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::*;

#[derive(Debug, Clone, Copy)]
enum Dist {
    Random,
    Desc,
}

struct InputCfg {
    len: usize,
    dist: Dist,
}

impl Factors for InputCfg {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "dist"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.len.to_string(), format!("{:?}", self.dist)]
    }
}

#[derive(Debug, Clone, Copy)]
enum Alg {
    StdSort,
    StdSortUnstable,
}

impl Factors for Alg {
    fn factor_names() -> Vec<&'static str> {
        vec!["alg"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("{:?}", self)]
    }
}

struct SortExp;

impl Experiment for SortExp {
    type InputFactors = InputCfg;
    type AlgFactors = Alg;
    type Input = Vec<u32>;
    type Output = Vec<u32>;

    fn input(&mut self, levels: &Self::InputFactors) -> Self::Input {
        match levels.dist {
            Dist::Desc => (0..levels.len as u32).rev().collect(),
            Dist::Random => (0..levels.len as u32).collect(),
        }
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let mut v = input.clone();
        match alg {
            Alg::StdSort => v.sort(),
            Alg::StdSortUnstable => v.sort_unstable(),
        }
        v
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let mut expected = input.clone();
        expected.sort();
        Some(expected)
    }
}

fn run(c: &mut Criterion) {
    let input_levels = vec![
        InputCfg {
            len: 64,
            dist: Dist::Random,
        },
        InputCfg {
            len: 64,
            dist: Dist::Desc,
        },
        InputCfg {
            len: 1024,
            dist: Dist::Random,
        },
        InputCfg {
            len: 1024,
            dist: Dist::Desc,
        },
    ];

    let alg_levels = vec![Alg::StdSort, Alg::StdSortUnstable];

    SortExp.bench(c, "sorting_minimal", &input_levels, &alg_levels);
}

criterion_group!(benches, run);
criterion_main!(benches);
