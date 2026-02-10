use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{AlgFactors, Experiment, InputFactors};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::Positions;

// Algorithm Factors

#[derive(Debug)]
enum Direction {
    Forwards,
    Backwards,
}

struct Params {
    num_threads: usize,
    direction: Direction,
}

impl AlgFactors for Params {
    fn factor_names() -> Vec<&'static str> {
        vec!["num_threads", "direction"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.num_threads.to_string(),
            format!("{:?}", self.direction),
        ]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["n", "d"]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        let direction = match self.direction {
            Direction::Forwards => "F",
            Direction::Backwards => "B",
        };
        vec![self.num_threads.to_string(), direction.to_string()]
    }
}

// Input Factors

#[derive(Debug)]
enum ValuePosition {
    Beg,
    Mid,
    End,
}

struct Settings {
    len: usize,
    position: ValuePosition,
}

impl InputFactors for Settings {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "position"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.len.to_string(), format!("{:?}", self.position)]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["l", "p"]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        let position = match self.position {
            ValuePosition::Beg => "B",
            ValuePosition::Mid => "M",
            ValuePosition::End => "E",
        };
        vec![self.len.to_string(), position.to_string()]
    }
}

fn run(c: &mut Criterion) {
    // let data = [
    //     DataSettings(1 << 5),
    //     DataSettings(1 << 10),
    //     DataSettings(1 << 15),
    // ];
    // let variants = [
    //     SearchMethod(StoreType::None),
    //     SearchMethod(StoreType::SortedVec),
    //     SearchMethod(StoreType::HashMap),
    //     SearchMethod(StoreType::BTreeMap),
    // ];

    // TwoSumExp::bench(c, "two_sum", &data, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
