use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Data, Experiment, Variant};
use orx_parallel::{ParIter, Parallelizable};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

// data

struct DataSettings {
    len: usize,
    position: usize,
}

impl Data for DataSettings {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "position"]
    }

    fn factor_values(&self) -> Vec<String> {
        vec![self.len.to_string(), self.position.to_string()]
    }
}

// variants

#[derive(Debug)]
enum ParLib {
    Rayon,
    OrxParallel,
}

#[derive(Debug)]
enum Approach {
    MapFind,
    MapAny,
}

struct SearchAlg {
    par_lib: ParLib,
    num_threads: usize,
    chunk_size: usize,
    approach: Approach,
}

impl Variant for SearchAlg {
    fn param_names() -> Vec<&'static str> {
        vec!["par_lib", "num_threads", "chunk_size", "approach"]
    }

    fn param_values(&self) -> Vec<String> {
        vec![
            format!("{:?}", self.par_lib),
            self.num_threads.to_string(),
            self.chunk_size.to_string(),
            format!("{:?}", self.approach),
        ]
    }
}

// experiment

struct TuneFindElements;

impl Experiment for TuneFindElements {
    type Data = DataSettings;

    type Variant = SearchAlg;

    type Input = Vec<String>;

    type Output = bool;

    fn input(data: &Self::Data) -> Self::Input {
        (0..data.len)
            .map(|i| match i == data.position {
                true => "__rust".to_string(),
                false => format!("__{i}"),
            })
            .collect()
    }

    fn expected_output(data: &Self::Data, _: &Self::Input) -> Option<Self::Output> {
        Some(data.position < data.len)
    }

    fn execute(variant: &Self::Variant, input: &Self::Input) -> Self::Output {
        match variant.par_lib {
            ParLib::OrxParallel => match variant.approach {
                Approach::MapFind => input
                    .par()
                    .num_threads(variant.num_threads)
                    .map(|x| &x[2..])
                    .find(|x| *x == "rust")
                    .is_some(),
                Approach::MapAny => input
                    .par()
                    .num_threads(variant.num_threads)
                    .map(|x| &x[2..])
                    .any(|x| *x == "rust"),
            },
            ParLib::Rayon => {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(variant.num_threads)
                    .build_global()
                    .unwrap();
                match variant.approach {
                    Approach::MapFind => input
                        .par_iter()
                        .map(|x| &x[2..])
                        .find_any(|x| *x == "rust")
                        .is_some(),
                    Approach::MapAny => input.par_iter().map(|x| &x[2..]).any(|x| x == "rust"),
                }
            }
        }
    }
}

fn run(c: &mut Criterion) {
    // let data = [
    //     SortData(1 << 5, 1 << 10),
    //     SortData(1 << 10, 1 << 9),
    //     SortData(1 << 20, 1 << 21),
    // ];
    // let variants = [
    //     SearchMethod::Linear,
    //     SearchMethod::LinearBackwards,
    //     SearchMethod::Binary,
    // ];

    // SearchExperiment::bench(c, "tuning_find_element", &data, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
