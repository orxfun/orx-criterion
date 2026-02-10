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

In order to represent these input variants, we define [`InputFactors`](https://docs.rs/orx-criterion/latest/orx_criterion/trait.InputFactors.html) named as `Settings`. Each unique setting instance can create a unique input for our experimentation. We will later add the interesting settings to the experimentation.

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
