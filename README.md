# orx-criterion

[![orx-criterion crate](https://img.shields.io/crates/v/orx-criterion.svg)](https://crates.io/crates/orx-criterion)
[![orx-criterion crate](https://img.shields.io/crates/d/orx-criterion.svg)](https://crates.io/crates/orx-criterion)
[![orx-criterion documentation](https://docs.rs/orx-criterion/badge.svg)](https://docs.rs/orx-criterion)

orx-criterion extends [criterion](https://crates.io/crates/criterion) with an experimentation model so you can compare algorithm variants and parameter choices across many input shapes, then get a summary.

This crate is most useful when:

- you have multiple ways to solve the same task,
- performance depends on input shape,
- you want one benchmark run that reports results across the full matrix of inputs and algorithm variants.

## Quick Start for Criterion Users

If you already use Criterion, your workflow stays the same.

1. Add dependency:

```toml
[dependencies]
orx-criterion = "1"
criterion = { version = "0.8", default-features = false }
```

2. In your bench file:

- define input factors by implementing `Factors`,
- define algorithm factors by implementing `Factors`,
- implement `Experiment` with `input` and `execute`,
- call `.bench(c, "bench_name", &input_levels, &alg_levels)`.

3. Keep standard Criterion wiring:

```rust ignore
criterion_group!(benches, run);
criterion_main!(benches);
```

4. Ensure bench is configured with `harness = false` in Cargo.toml.

5. Run as usual:

```bash
cargo bench --bench sorting_alg
```

## Criterion Mental Model

| Criterion concept | orx-criterion concept |
|---|---|
| Parameterized benchmark inputs | `InputFactors` |
| Alternative implementations or tunable settings | `AlgFactors` |
| Setup code you do not want timed | `input(...)` |
| Timed benchmark body | `execute(...)` |
| Benchmark identifier/path | `bench_name` argument in `.bench(...)` |
| Normal Criterion run and reports | unchanged |

What this crate does not change:

- you still run with `cargo bench`,
- you still use `criterion_group!` and `criterion_main!`,
- Criterion output still goes under `target/criterion`.

## Minimal Complete Example

The example below shows the smallest useful pattern.

```rust ignore
use orx_criterion::*;
use criterion::{Criterion, criterion_group, criterion_main};

#[derive(Debug, Clone, Copy)]
enum Dist {
    Random,
    Desc,
}

struct InputCfg {
    len: usize,
    dist: Dist,
}

impl Factors for InputCfg {
    fn factor_names() -> Vec<&'static str> {
        vec!["len", "dist"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.len.to_string(), format!("{:?}", self.dist)]
    }
}

#[derive(Debug, Clone, Copy)]
enum Alg {
    StdSort,
    StdSortUnstable,
}

impl Factors for Alg {
    fn factor_names() -> Vec<&'static str> {
        vec!["alg"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![format!("{:?}", self)]
    }
}

struct SortExp;

impl Experiment for SortExp {
    type InputFactors = InputCfg;
    type AlgFactors = Alg;
    type Input = Vec<u32>;
    type Output = Vec<u32>;

    fn input(&mut self, levels: &Self::InputFactors) -> Self::Input {
        match levels.dist {
            Dist::Desc => (0..levels.len as u32).rev().collect(),
            Dist::Random => (0..levels.len as u32).collect(),
        }
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        let mut v = input.clone();
        match alg {
            Alg::StdSort => v.sort(),
            Alg::StdSortUnstable => v.sort_unstable(),
        }
        v
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        let mut expected = input.clone();
        expected.sort();
        Some(expected)
    }
}

fn run(c: &mut Criterion) {
    let input_levels = vec![
        InputCfg { len: 64, dist: Dist::Random },
        InputCfg { len: 64, dist: Dist::Desc },
        InputCfg { len: 1024, dist: Dist::Random },
        InputCfg { len: 1024, dist: Dist::Desc },
    ];

    let alg_levels = vec![Alg::StdSort, Alg::StdSortUnstable];

    SortExp.bench(c, "sorting_minimal", &input_levels, &alg_levels);
}

criterion_group!(benches, run);
criterion_main!(benches);
```

Add this to Cargo.toml:

```toml
[[bench]]
name = "sorting_minimal"
harness = false
```

A more complete example (with richer input distributions and hand-written algorithms) is available in [benches/sorting_alg.rs](https://github.com/orxfun/orx-criterion/blob/main/benches/sorting_alg.rs).

## Output Artifacts

During a benchmark run, in addition to normal Criterion logs, orx-criterion produces:

1. Console summary table
- includes factor columns and timing summary per treatment,
- highlights best and worst algorithm variants for each input.

2. CSV summary
- path: `target/criterion/{bench_name}/summary_{bench_name}.csv`,
- useful for post-processing, plotting, or dashboards.

3. AI prompt draft
- path: `target/criterion/{bench_name}/prompt_{bench_name}.md`,
- optional helper for quick narrative summaries.

Example screenshots:

![logs](https://github.com/orxfun/orx-docs-img/blob/main/orx-criterion/readme_criterion_logs.jpg?raw=true)

![summary-table-console](https://raw.githubusercontent.com/orxfun/orx-docs-img/refs/heads/main/orx-criterion/readme_summary_table_console.jpg)

![summary-ai](https://raw.githubusercontent.com/orxfun/orx-docs-img/refs/heads/main/orx-criterion/readme_ai_summary.jpg)

## Common Mistakes and Tips

1. Forgetting `harness = false`
- Symptom: benchmark does not run as Criterion bench.

2. Very long factor keys
- Criterion folder names are practically limited to 64 chars.
- Implement `factor_names_short` and `factor_levels_short` when needed.

3. Expensive setup inside `execute`
- Put input construction into `input(...)` so setup time is not benchmarked.

4. Assuming validation affects timing
- `expected_output` and `validate_output` checks run once per (input, algorithm), not per sample, and validation time is not benchmarked.

5. Not keeping factor vectors aligned
- `factor_names*` and `factor_levels*` must match in length and order.

## API Docs

- `Experiment`: https://docs.rs/orx-criterion/latest/orx_criterion/trait.Experiment.html
- `Factors`: https://docs.rs/orx-criterion/latest/orx_criterion/trait.Factors.html

## Contributing

Contributions are welcome. If you notice an issue or have an improvement idea, please open an issue or submit a PR.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
