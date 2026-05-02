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

    fn factor_names_short() -> Vec<&'static str> {
        vec!["l", "d"]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        let distribution = match self.distribution {
            Distribution::Random => "R",
            Distribution::NearlySorted => "N",
            Distribution::Descending => "D",
        };
        vec![self.len.to_string(), distribution.to_string()]
    }
}

// Algorithm Factors

/// Base-case sorting algorithm applied to small sub-arrays.
#[derive(Debug, Clone, Copy)]
enum BaseSort {
    /// Insertion sort: fast for small or nearly-sorted slices.
    Insertion,
    /// Selection sort: fixed number of writes regardless of input order.
    Selection,
}

/// Parameters defining the merge sort algorithm.
struct Params {
    /// When sub-array length falls at or below this value, switch to `base_sort`.
    cutoff: usize,
    /// Base-case algorithm used for small sub-arrays.
    base_sort: BaseSort,
}

impl Factors for Params {
    fn factor_names() -> Vec<&'static str> {
        vec!["cutoff", "base_sort"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.cutoff.to_string(), format!("{:?}", self.base_sort)]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["c", "b"]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        let base_sort = match self.base_sort {
            BaseSort::Insertion => "I",
            BaseSort::Selection => "S",
        };
        vec![self.cutoff.to_string(), base_sort.to_string()]
    }
}

// Helpers

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

fn merge_sort(arr: &mut [u32], cutoff: usize, base_sort: BaseSort) {
    if arr.len() <= cutoff {
        match base_sort {
            BaseSort::Insertion => insertion_sort(arr),
            BaseSort::Selection => selection_sort(arr),
        }
        return;
    }
    let mid = arr.len() / 2;
    merge_sort(&mut arr[..mid], cutoff, base_sort);
    merge_sort(&mut arr[mid..], cutoff, base_sort);
    // merge the two sorted halves via a temporary buffer
    let buf: Vec<u32> = arr.to_vec();
    let (left, right) = buf.split_at(mid);
    let mut i = 0;
    let mut j = 0;
    for k in 0..arr.len() {
        arr[k] = if j >= right.len() || (i < left.len() && left[i] <= right[j]) {
            let v = left[i];
            i += 1;
            v
        } else {
            let v = right[j];
            j += 1;
            v
        };
    }
}

/// Simple deterministic Fisher-Yates shuffle (LCG-based, seeded by length).
fn shuffle(data: &mut [u32]) {
    let mut state = data.len() as u64;
    for i in (1..data.len()).rev() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (state >> 33) as usize % (i + 1);
        data.swap(i, j);
    }
}

// Experiment

struct Input {
    data: Vec<u32>,
}

/// Experiment to tune a merge sort implementation over arrays with different
/// distributions by varying the cutoff threshold and base-case algorithm.
struct SortExp;

impl Experiment for SortExp {
    type InputFactors = Settings;

    type AlgFactors = Params;

    type Input = Input;

    type Output = Vec<u32>;

    fn input(&mut self, input_levels: &Self::InputFactors) -> Self::Input {
        let mut data: Vec<u32> = (0..input_levels.len as u32).collect();

        match input_levels.distribution {
            Distribution::Random => shuffle(&mut data),
            Distribution::NearlySorted => {
                // apply a small number of random swaps to an otherwise sorted array
                let swaps = (input_levels.len as f64).sqrt() as usize;
                let mut state = input_levels.len as u64;
                for _ in 0..swaps {
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let i = (state >> 33) as usize % input_levels.len;
                    state = state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let j = (state >> 33) as usize % input_levels.len;
                    data.swap(i, j);
                }
            }
            Distribution::Descending => data.reverse(),
        }

        Input { data }
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        // the output is determined by `alg_variant` cutoff and base_sort fields
        let mut data = input.data.clone();
        merge_sort(&mut data, alg_variant.cutoff, alg_variant.base_sort);
        data
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        // compute a reference sorted array to validate against
        let mut sorted = input.data.clone();
        sorted.sort();
        Some(sorted)
    }

    fn validate_output(&self, _: &Self::InputFactors, _input: &Self::Input, output: &Self::Output) {
        // verify the output is non-decreasing
        assert!(output.windows(2).all(|w| w[0] <= w[1]));
    }
}

fn run(c: &mut Criterion) {
    // input levels that we are interested in
    let lengths = [1 << 10, 1 << 16];
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
    let cutoffs = [1, 8, 32, 128];
    let base_sorts = [BaseSort::Insertion, BaseSort::Selection];
    let alg_levels: Vec<_> = cutoffs
        .into_iter()
        .flat_map(|cutoff| {
            base_sorts
                .iter()
                .copied()
                .map(move |base_sort| Params { cutoff, base_sort })
        })
        .collect();

    // execute a factorial experiment over the union of input and algorithm factors
    SortExp.bench(c, "sorting_alg", &input_levels, &alg_levels);
}

criterion_group!(benches, run);
criterion_main!(benches);
