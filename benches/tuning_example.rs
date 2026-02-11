use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{AlgFactors, Experiment, InputFactors};

// Input Factors

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

// Algorithm Factors

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

// Experiment

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

    fn input(&mut self, input_levels: &Self::InputFactors) -> Self::Input {
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

    fn execute(&mut self, alg_variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
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
            handles.into_iter().filter_map(|h| h.join().unwrap()).next()
        })
    }

    fn expected_output(
        &self,
        _settings: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        // we simply return the expected output cached in the input
        Some(input.position)
    }

    fn validate_output(
        &self,
        _settings: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        // additional validation logic just to make sure
        // the linear search below does not affect results
        match *output {
            Some(position) => assert_eq!(input.array[position], SEARCH_VALUE),
            None => assert!(!input.array.iter().any(|x| x.as_str() == SEARCH_VALUE)),
        }
    }
}

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

    // execute a full-factorial experiment over the union of input and algorithm factors
    SearchExp.bench(c, "tuning_example", &input_levels, &alg_levels);
}

criterion_group!(benches, run);
criterion_main!(benches);
