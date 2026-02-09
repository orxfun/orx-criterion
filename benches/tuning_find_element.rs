use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Data, Experiment, Variant};
use orx_parallel::{ParIter, Parallelizable};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

// data

const SEARCH_PHRASE: &str = "rust";

struct DataSettings {
    len: usize,
    position: usize,
}

impl Data for DataSettings {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "position"]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["l", "p"]
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

#[derive(Debug, Clone, Copy)]
enum Approach {
    Find,
    Any,
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

    fn param_names_short() -> Vec<&'static str> {
        vec!["lib", "nt", "ch", "app"]
    }

    fn param_values(&self) -> Vec<String> {
        vec![
            format!("{:?}", self.par_lib),
            self.num_threads.to_string(),
            self.chunk_size.to_string(),
            format!("{:?}", self.approach),
        ]
    }

    fn param_values_short(&self) -> Vec<String> {
        vec![
            match self.par_lib {
                ParLib::OrxParallel => "X",
                ParLib::Rayon => "R",
            }
            .to_string(),
            self.num_threads.to_string(),
            self.chunk_size.to_string(),
            match self.approach {
                Approach::Find => "F",
                Approach::Any => "A",
            }
            .to_string(),
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
                true => format!("__{SEARCH_PHRASE}"),
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
                Approach::Find => input
                    .par()
                    .num_threads(variant.num_threads)
                    .map(|x| &x[2..])
                    .find(|x| *x == SEARCH_PHRASE)
                    .is_some(),
                Approach::Any => input
                    .par()
                    .num_threads(variant.num_threads)
                    .map(|x| &x[2..])
                    .any(|x| *x == SEARCH_PHRASE),
            },
            ParLib::Rayon => {
                fn create_pool(num_threads: usize) -> rayon::ThreadPool {
                    rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build()
                        .unwrap()
                }
                create_pool(variant.num_threads).install(|| match variant.approach {
                    Approach::Find => input
                        .par_iter()
                        .map(|x| &x[2..])
                        .find_any(|x| *x == SEARCH_PHRASE)
                        .is_some(),
                    Approach::Any => input
                        .par_iter()
                        .map(|x| &x[2..])
                        .any(|x| x == SEARCH_PHRASE),
                })
            }
        }
    }
}

fn run(c: &mut Criterion) {
    // data
    let new_data = |len, position| DataSettings { len, position };
    let data = [
        new_data(1 << 10, 1 << 9),
        new_data(1 << 10, 1 << 10),
        new_data(1 << 15, 1 << 14),
        new_data(1 << 15, 1 << 15),
        new_data(1 << 20, 1 << 16),
        new_data(1 << 20, 1 << 20),
    ];

    // variants

    let num_threads = || [1, 2, 4, 8, 16];
    let chunk_size = || [1, 1 << 6, 1 << 10];
    let approach = || [Approach::Find, Approach::Any];

    let orx_variants = num_threads().into_iter().flat_map(|num_threads| {
        chunk_size().into_iter().flat_map(move |chunk_size| {
            approach().into_iter().map(move |approach| SearchAlg {
                par_lib: ParLib::OrxParallel,
                num_threads,
                chunk_size,
                approach,
            })
        })
    });

    let rayon_variants = num_threads().into_iter().flat_map(|num_threads| {
        approach().into_iter().map(move |approach| SearchAlg {
            par_lib: ParLib::Rayon,
            num_threads,
            chunk_size: 1,
            approach,
        })
    });

    let variants: Vec<_> = orx_variants.chain(rayon_variants).collect();

    // experiment

    TuneFindElements::bench(c, "tuning_find_element", &data, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
