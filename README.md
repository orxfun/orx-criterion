# orx-criterion

[![orx-criterion crate](https://img.shields.io/crates/v/orx-criterion.svg)](https://crates.io/crates/orx-criterion)
[![orx-criterion crate](https://img.shields.io/crates/d/orx-criterion.svg)](https://crates.io/crates/orx-criterion)
[![orx-criterion documentation](https://docs.rs/orx-criterion/badge.svg)](https://docs.rs/orx-criterion)

Additional [criterion](https://crates.io/crates/criterion) benchmarking utilities to enable parameter tuning for speed.

Please see the example below for demonstration.

## Tuning Example

Consider a very simple algorithm that we want to tune:

- we are given an input array and a target value to search,
- we are expected to locate its position within the array if it exists.

We want to tune our algorithm so that we locate the element as fast as possible across different data sets.

### Input Factors

Input to this problem might differ in two ways:

- length of the array,
- position of the value that we search for.

In order to represent these input variants, we define [`InputFactors`](https://docs.rs/orx-criterion/latest/orx_criterion/trait.InputFactors.html) named as `Settings`. Each unique setting instance can create a unique input for our experimentation. We will later add to experiment the settings that are interesting for our use case.

```rust
use orx_criterion::*;

/// Position of the target value in the input array.
#[derive(Debug, Clone, Copy)]
enum ValuePosition {
    /// The target value is located in the middle of the array.
    Mid,
    /// The target value does not exist in the array.
    None,
}

/// Settings to define input of the search problem.
struct Settings {
    /// Length of the input array.
    len: usize,
    /// Position of the target value inside the input array.
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
            ValuePosition::Mid => "M",
            ValuePosition::None => "X",
        };
        vec![self.len.to_string(), position.to_string()]
    }
}
```

Factor names and levels are used to create unique key of each input. For instance, the input created by `Settings { len: 1024, position: ValuePosition::Mid }` will have the key `len:1024_position:Mid`. Further, the factor names will be used as column headers of summary tables.

Treatment keys are also used as directory names by "criterion" to store the results. In order to keep the directory names sufficiently short (within 64 characters), we can optionally implement the short versions of names and levels. The short key to be used as directory name for the above example would then be `l:1024_p:M`.

## Algorithm Factors

We want to solve this problem by a linear search. Additionally, we want to consider the parallelized variants.

In order to represent these algorithm variants, we define [`AlgFactors`](https://docs.rs/orx-criterion/latest/orx_criterion/trait.AlgFactors.html) named as `Params`. Each unique setting instance determines the way that our algorithm will execute. We will later add to experiment the algorithm variants that we want to evaluate.

```rust
use orx_criterion::*;

/// Defines the direction of the search for the target value.
#[derive(Debug, Clone, Copy)]
enum Direction {
    /// The array will be search from beginning to the end.
    Forwards,
    /// The array will be search from end to the beginning.
    Backwards,
}

/// Parameters defining the search algorithm.
struct Params {
    /// Number of threads to use for the search.
    num_threads: usize,
    /// Direction of search by each thread.
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
```

## Experiment

Finally, we define the experiment.

We need to implement two required methods.

- `input` takes levels of input factors and produces the input to be solved by all algorithm variants of the experiment.
- `execute` takes an algorithm variant and an input, and solves the problem on the input with the given algorithm variant. The method produces and returns the output.

The experimentation will study how much time is spent by the `execute`. Time of the `input` creation is not important for the experimentation. We aim to find the best algorithm factor levels to minimize the execution time.

Optionally, we can implement validation methods:

- `expected_output` takes input levels and created input and returns the expected output. This value will be compared to the value that the `execute` method actually generates, and panics if they do not match. Importantly note that:
  - all algorithm variants must produce exactly the same output for the same input, and
  - an algorithm variant must always produce the same output for the same input.

  We can still investigate randomized algorithms. In such cases; however, we cannot use the `expected_output` validation. We can then simply return `None`, or not implement the method at all since the default implementation returns none.

  In the example below, we return the expected output that is cached inside the input while creating it. Another common way is to execute a well-tested method to compute the expected output, which will then be compared against new variants that are being evaluated.

- `validate_output` takes input levels, created input together with the produced output, and performs custom validation logic on them. Similarly, the default implementation is an empty function which does nothing.

Note that both of the validation methods are executed **only once** per (input, algorithm) combination and the time spent for validation is **not included** in the results. Therefore, it is okay to implement detailed, long-running validation methods when we need them to make sure of correctness of the results.

```rust ignore
use orx_criterion::*;

/// Value to search for.
const SEARCH_VALUE: &str = "criterion";

struct Input {
    array: Vec<String>,
    position: Option<usize>, // to be used for validation
}

/// Experiment to carry out factorial analysis for searching a target value
/// within an array.
struct SearchExp;

impl Experiment for SearchExp {
    type InputFactors = Settings;

    type AlgFactors = Params;

    type Input = Input;

    type Output = Option<usize>;

    fn input(input_levels: &Self::InputFactors) -> Self::Input {
        // we create an array with the given length, without the search value
        let mut array: Vec<_> = (0..input_levels.len).map(|i| i.to_string()).collect();

        // we decide on index of the search value depending on the position setting
        let index = match input_levels.position {
            ValuePosition::Mid => input_levels.len / 2,
            ValuePosition::None => input_levels.len,
        };

        // we place the search value at the index
        let position = match array.get_mut(index) {
            Some(element) => {
                *element = SEARCH_VALUE.to_string();
                Some(index)
            }
            None => None,
        };

        Input { array, position }
    }

    fn execute(alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
        // notice that how we compute the output is determined by
        // values of `alg_variant` fields.

        let chunk_size = input.array.len() / alg_variant.num_threads;
        let chunks: Vec<_> = input.array.chunks(chunk_size).collect();

        std::thread::scope(|s| {
            let mut handles = vec![];
            let mut begin = 0;
            for chunk in chunks {
                handles.push(s.spawn(move || {
                    let mut iter = chunk.iter();

                    match alg_variant.direction {
                        Direction::Forwards => iter
                            .position(|x| x.as_str() == SEARCH_VALUE)
                            .map(|x| begin + x),
                        Direction::Backwards => iter
                            .rev()
                            .position(|x| x.as_str() == SEARCH_VALUE)
                            .map(|x| begin + (chunk.len() - 1 - x)),
                    }
                }));
                begin += chunk.len();
            }

            // get the result from threads in the form of Some(position), if any
            handles
                .into_iter()
                .map(|h| h.join().unwrap())
                .filter_map(|x| x)
                .next()
        })
    }

    fn expected_output(_input_levels: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        // we simply return the expected output cached in the input
        Some(input.position)
    }

    fn validate_output(_input_levels: &Self::InputFactors, input: &Self::Input, output: &Self::Output) {
        // additional validation logic just to make sure
        // the linear search below does not affect results
        match *output {
            Some(position) => assert_eq!(input.array[position], SEARCH_VALUE),
            None => assert!(!input.array.iter().any(|x| x.as_str() == SEARCH_VALUE)),
        }
    }
}
```

## Run the Experiment (Benchmark)

We defined everything we need to run the experiment.

Finally, we will run it using the [criterion](https://crates.io/crates/criterion) crate.

### Define the Experiment as a Criterion Benchmark

We create the benchmark file under the **benches** folder, say `benches/tuning_example.rs`. We add all the code above to this file.

Finally, we add the following lines that will allow us to start the benchmark run.

We can start our benchmark with `SearchExp::bench(c, "tuning_example", &input_levels, &alg_levels)` call, which will create a benchmark run for each (input, algorithm) combination.

```rust ignore
use criterion::{Criterion, criterion_group, criterion_main};

fn run(c: &mut Criterion) {
    // input levels that we are interested in
    let lengths = [1 << 10, 1 << 24];
    let positions = [ValuePosition::Mid, ValuePosition::None];
    let input_levels: Vec<_> = lengths
        .into_iter()
        .flat_map(|len| {
            positions
                .iter()
                .copied()
                .map(move |position| Settings { len, position })
        })
        .collect();

    // algorithm variants that we want to evaluate
    let num_threads = [1, 16];
    let directions = [Direction::Forwards, Direction::Backwards];
    let alg_levels: Vec<_> = num_threads
        .into_iter()
        .flat_map(|num_threads| {
            directions.iter().copied().map(move |direction| Params {
                num_threads,
                direction,
            })
        })
        .collect();

    // execute benchmarks for each (input, algorithm) combination
    SearchExp::bench(c, "tuning_example", &input_levels, &alg_levels);
}

criterion_group!(benches, run);
criterion_main!(benches);
```

### Configure Cargo.toml

In order to run this file as a benchmark, we need to add the following lines to `Cargo.toml`:

```yaml
[[bench]]
name = "tuning_example"
harness = false
```

### Running the Benchmark

Then, we can run the benchmark & experiment with `cargo bench` command.

Notice that the experimentation is run by having data points (inputs) as the outer loop and algorithm variants in the inner loop. This allows to create each input only once.

You may also notice both the long and short keys of each treatment, such as:

- len:1024_position:Mid/num_threads:1_direction:Forwards
- l:1024_p:M/n:1_d:F

## Logs

This crate will add some additional logs to default "criterion" logs containing information about the experimentation.

```shell
# tuning_example benchmarks with 4 data points and 4 variants => 16 treatments


## Data point [1/4]: len:1024_position:Mid

### [1/16 || 1/4]: len:1024_position:Mid/num_threads:1_direction:Forwards
tuning_example/l:1024_p:M/n:1_d:F
                        time:   [135.56 µs 140.41 µs 145.96 µs]
                        change: [−3.3333% +0.8703% +5.5680%] (p = 0.69 > 0.05)
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

### [2/16 || 2/4]: len:1024_position:Mid/num_threads:1_direction:Backwards
tuning_example/l:1024_p:M/n:1_d:B
                        time:   [131.06 µs 136.04 µs 141.49 µs]
                        change: [−16.743% −13.143% −9.4026%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

### [3/16 || 3/4]: len:1024_position:Mid/num_threads:16_direction:Forwards
Benchmarking tuning_example/l:1024_p:M/n:16_d:F: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.7s, enable flat sampling, or reduce sample count to 60.
tuning_example/l:1024_p:M/n:16_d:F
                        time:   [958.02 µs 995.59 µs 1.0376 ms]
                        change: [−33.388% −30.257% −26.639%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

### [4/16 || 4/4]: len:1024_position:Mid/num_threads:16_direction:Backwards
Benchmarking tuning_example/l:1024_p:M/n:16_d:B: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.8s, enable flat sampling, or reduce sample count to 60.
tuning_example/l:1024_p:M/n:16_d:B
                        time:   [1.2515 ms 1.3025 ms 1.3544 ms]
                        change: [−21.130% −16.332% −11.060%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild





## Data point [2/4]: len:1024_position:None

### [5/16 || 1/4]: len:1024_position:None/num_threads:1_direction:Forwards
tuning_example/l:1024_p:X/n:1_d:F
                        time:   [138.13 µs 144.15 µs 150.80 µs]
                        change: [−27.490% −23.275% −18.948%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

...
```

## Summary Table
