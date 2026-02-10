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
```
