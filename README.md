# orx-criterion

[![orx-criterion crate](https://img.shields.io/crates/v/orx-criterion.svg)](https://crates.io/crates/orx-criterion)
[![orx-criterion crate](https://img.shields.io/crates/d/orx-criterion.svg)](https://crates.io/crates/orx-criterion)
[![orx-criterion documentation](https://docs.rs/orx-criterion/badge.svg)](https://docs.rs/orx-criterion)

Experimentation library using [criterion](https://crates.io/crates/criterion) benchmarks for analyzing alternatives or parameter tuning.

This crate is useful in the following case:

- We have a problem or a task.
- We have different ways to solve this problem, so called _algorithm variants_.
- We have different shapes of inputs to the problem that might impact the speed, so called _input variants_.
- We want to find the best algorithm variant with respect to some goal, for instance:
  - best variant for specific inputs,
  - the variant that has the best overall performance,
  - the variant that has a good balance of speed and predictability, etc.

## Tuning Example

_You may find more examples in [benches](https://github.com/orxfun/orx-criterion/tree/main/benches) folder._

Consider a simple algorithm comparison problem:

- we are given an array of integers to sort,
- we want to know which sorting algorithm performs best across different data sets.

We compare two O(n²) algorithms — insertion sort and selection sort — across arrays of different lengths and value distributions.

### Input Factors

Input to this problem might differ in two ways:

- length of the array,
- distribution of values (random, nearly-sorted, or descending).

In order to represent these input variants, we define [`Factors`](https://docs.rs/orx-criterion/latest/orx_criterion/trait.Factors.html) named as `Settings`. Each unique instance of `Settings` can create a unique input for our experimentation.

```rust
use orx_criterion::*;

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
```

Factor names and levels are used to create the unique key for each input. For instance, `Settings { len: 1024, distribution: Distribution::Random }` will have the key `len:1024_distribution:Random`. The factor names are also used as column headers of the summary tables.

Note that `factor_names_short` and `factor_levels_short` are optional. When omitted, the long key is used as the criterion directory name; this is fine as long as it stays within 64 characters.

### Algorithm Factors

The algorithm variants are the two sorting algorithms we want to compare. Because there is only one axis of variation, we can implement [`Factors`](https://docs.rs/orx-criterion/latest/orx_criterion/trait.Factors.html) directly on the algorithm enum — no wrapper struct is needed.

```rust
use orx_criterion::*;

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
```

### Experiment

Finally, we define the experiment.

We need to implement two required methods.

- `input` takes levels of input factors and produces the input to be solved by all algorithm variants of the experiment.
- `execute` takes an algorithm variant and an input, and solves the problem on the input with the given algorithm variant. The method produces and returns the output.

The experimentation will study how much time is spent by `execute`. The time spent in `input` is not measured and does not affect the results.

Optionally, we can implement `expected_output`, which returns the correct answer for a given input. The library checks that every algorithm variant produces this output and panics if they do not match. Another optional method is `validate_output`, which allows us to implement custom validation logic. This validation methods run **only once** per (input, algorithm) combination and its time is **not included** in the results.

```rust ignore
use orx_criterion::*;

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
```

### Run the Experiment (Benchmark)

We defined everything we need to run the experiment.

Finally, we will run it using the [criterion](https://crates.io/crates/criterion) crate.

#### Define the Experiment as a Criterion Benchmark

We create the benchmark file under the **benches** folder, say `benches/sorting_alg.rs`. We add all the code above to this file, then append the following lines to start the benchmark run.

```rust ignore
use criterion::{Criterion, criterion_group, criterion_main};

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
```

#### Configure Cargo.toml

In order to run this file as a benchmark, we need to add the following lines to `Cargo.toml`:

```yaml
[[bench]]
name = "sorting_alg"
harness = false
```

#### Running the Benchmark

Then, we can run the benchmark & experiment with `cargo bench --bench sorting_alg` command.

Notice that the experimentation is run by having data points (inputs) as the outer loop and algorithm variants in the inner loop. This allows to create each input only once.

### Logs

This crate will add some additional logs to default "criterion" logs containing information about the experimentation.

![logs](https://github.com/orxfun/orx-docs-img/blob/main/orx-criterion/readme_criterion_logs.jpg?raw=true)

### Summary Table - Console

Once all benchmark runs are completed, a summary table will be printed to the console, thanks to [cli-table](https://crates.io/crates/cli-table) and [colorize](https://crates.io/crates/colorize) crates.

In addition to factor levels, the table includes three index columns:

- **t** is the index of the treatment, each row will have a unique index.
- **i** is the index of the input, each input will have its unique index.
- **a** is the index of the algorithm variant, each algorithm will have its unique index.

Rows of the <span style="color:green">best</span> and the <span style="color:red">worst</span> algorithm variants for each input will be color-coded.

The following table is the result of the run of the benchmark defined in this example.

![summary-table-console](https://raw.githubusercontent.com/orxfun/orx-docs-img/refs/heads/main/orx-criterion/readme_summary_table_console.jpg)

### Summary Table - CSV

As it will be noted in the logs, a csv version of the summary table will also be created in the directory of the benchmark: `target/criterion/{bench_name}/summary_{bench_name}.csv`.

```shell
Summary table created at:
target/criterion/sorting_alg/summary_sorting_alg.csv
```

### AI Prompt

Also a draft AI prompt to summarize the results will be created at `target/criterion/{bench_name}/prompt_{bench_name}.md`, in case you find it helpful for a quick overview. The following is a response to the prompt created for this example.

![summary-table-console](https://raw.githubusercontent.com/orxfun/orx-docs-img/refs/heads/main/orx-criterion/readme_ai_summary.jpg)

## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be added or improved, please open an [issue](https://github.com/orxfun/orx-tree/issues/new) or create a PR.

If you are interested in these particular topics, there are two open issues ([17](https://github.com/orxfun/orx-criterion/issues/17) & [19](https://github.com/orxfun/orx-criterion/issues/19)), which I believe, could make the library much more useful.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
