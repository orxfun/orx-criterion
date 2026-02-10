use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{AlgFactors, Data, Experiment};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

// Variant

#[derive(Debug)]
enum Direction {
    Forwards,
    Backwards,
}

struct AlgParams {
    num_threads: usize,
    direction: Direction,
}

impl AlgFactors for AlgParams {
    fn factor_names() -> Vec<&'static str> {
        vec!["num_threads", "direction"]
    }

    fn factor_values(&self) -> Vec<String> {
        vec![
            self.num_threads.to_string(),
            format!("{:?}", self.direction),
        ]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["n", "d"]
    }

    fn factor_values_short(&self) -> Vec<String> {
        let direction = match self.direction {
            Direction::Forwards => "F",
            Direction::Backwards => "B",
        };
        vec![self.num_threads.to_string(), direction.to_string()]
    }
}

// Instance

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
