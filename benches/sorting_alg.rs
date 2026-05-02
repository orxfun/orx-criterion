use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};

// Input Factors

/// Distribution of values in the input array.
#[derive(Debug, Clone, Copy)]
enum Distribution {
    /// Randomly shuffled integers.
    Random,
    /// Array is sorted with a small number of random swaps applied.
    NearlySorted,
    /// Array is sorted in descending order.
    Descending,
}

/// Settings to define the input of the sorting problem.
struct Settings {
    /// Length of the input array.
    len: usize,
    /// Distribution of values in the array.
    distribution: Distribution,
}

impl Factors for Settings {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "distribution"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.len.to_string(), format!("{:?}", self.distribution)]
    }
}

// Algorithm Factors

/// Sorting algorithm to benchmark.
#[derive(Debug, Clone, Copy)]
enum Algorithm {
    /// Insertion sort: fast for small or nearly-sorted slices.
    Insertion,
    /// Selection sort: fixed number of writes regardless of input order.
    Selection,
}

impl Factors for Algorithm {
    fn factor_names() -> Vec<&'static str> {
        vec!["algorithm"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("{:?}", self)]
    }
}

// Helpers

fn shuffle(data: &mut [u32]) {
    let n = data.len();
    for i in 0..n {
        data.swap(i, (i * 7 + 13) % n);
    }
}

fn insertion_sort(arr: &mut [u32]) {
    for i in 1..arr.len() {
        let key = arr[i];
        let mut j = i;
        while j > 0 && arr[j - 1] > key {
            arr[j] = arr[j - 1];
            j -= 1;
        }
        arr[j] = key;
    }
}

fn selection_sort(arr: &mut [u32]) {
    for i in 0..arr.len() {
        let min_idx = (i..arr.len()).min_by_key(|&k| arr[k]).unwrap();
        arr.swap(i, min_idx);
    }
}

// Experiment

/// Experiment to compare insertion sort and selection sort over arrays with
/// different lengths and value distributions.
struct SortExp;

impl Experiment for SortExp {
    type InputFactors = Settings;

    type AlgFactors = Algorithm;

    type Input = Vec<u32>;

    type Output = Vec<u32>;

    fn input(&mut self, input_levels: &Self::InputFactors) -> Self::Input {
        let mut data: Vec<u32> = (0..input_levels.len as u32).collect();

        match input_levels.distribution {
            Distribution::Random => shuffle(&mut data),
            Distribution::NearlySorted => {
                // swap a small number of adjacent pairs
                let swaps = (input_levels.len as f64).sqrt() as usize;
                for i in (0..swaps).filter(|i| i + 1 < input_levels.len) {
                    data.swap(i, i + 1);
                }
            }
            Distribution::Descending => data.reverse(),
        }

        data
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let mut data = input.clone();
        match alg_variant {
            Algorithm::Insertion => insertion_sort(&mut data),
            Algorithm::Selection => selection_sort(&mut data),
        }
        data
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let mut sorted = input.clone();
        sorted.sort();
        Some(sorted)
    }
}

fn run(c: &mut Criterion) {
    // input levels that we are interested in
    let lengths = [1 << 6, 1 << 10];
    let distributions = [
        Distribution::Random,
        Distribution::NearlySorted,
        Distribution::Descending,
    ];
    let input_levels: Vec<_> = lengths
        .into_iter()
        .flat_map(|len| {
            distributions
                .iter()
                .copied()
                .map(move |distribution| Settings { len, distribution })
        })
        .collect();

    // algorithm variants that we want to evaluate
    let alg_levels = [Algorithm::Insertion, Algorithm::Selection];

    // execute a factorial experiment over the union of input and algorithm factors
    SortExp.bench(c, "sorting_alg", &input_levels, &alg_levels);
}

criterion_group!(benches, run);
criterion_main!(benches);
