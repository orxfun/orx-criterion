use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{AlgFactors, Experiment, InputFactors};
use orx_parallel::{ParIter, ParallelizableCollection};
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
    None,
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
            ValuePosition::None => "X",
        };
        vec![self.len.to_string(), position.to_string()]
    }
}

// Experiment

const SEARCH_VALUE: &str = "criterion";

struct Input {
    array: Vec<String>,
    exists: bool,
}

struct SearchExp;

impl Experiment for SearchExp {
    type Data = Settings;

    type Variant = Params;

    type Input = Input;

    type Output = bool;

    fn input(data: &Self::Data) -> Self::Input {
        let mut array: Vec<_> = (0..data.len).map(|i| i.to_string()).collect();
        let position = match data.position {
            ValuePosition::Beg => data.len / 5,
            ValuePosition::Mid => data.len / 2,
            ValuePosition::End => 4 * data.len / 5,
            ValuePosition::None => data.len,
        };
        // we place the search value at the position
        if let Some(element) = array.get_mut(position) {
            *element = SEARCH_VALUE.to_string();
        }
        let exists = position < array.len();
        Input { array, exists }
    }

    fn execute(alg_variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        match alg_variant.direction {
            Direction::Forwards => input
                .array
                .par()
                .num_threads(alg_variant.num_threads)
                .find(|x| x.as_str() == SEARCH_VALUE)
                .is_some(),
            Direction::Backwards => {
                let chunk_size = input.array.len() / alg_variant.num_threads;
                let chunks: Vec<_> = input.array.chunks(chunk_size).collect();
                std::thread::scope(|s| {
                    let mut handles = vec![];
                    for chunk in chunks {
                        handles.push(s.spawn(|| {
                            chunk
                                .iter()
                                .position(|x| x.as_str() == SEARCH_VALUE)
                                .is_some()
                        }));
                    }
                    handles.into_iter().map(|h| h.join().unwrap()).any(|x| x)
                })
            }
        }
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
