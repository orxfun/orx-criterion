use crate::experiment_sealed::ExperimentSealed;
use crate::{Experiment, Factors};
use std::fs::File;
use std::io::Write;

pub fn create_ai_prompt_to_analyze<E: Experiment>(
    exp: &E,
    name: &str,
    data: &[E::InputFactors],
    variants: &[E::AlgFactors],
) -> std::io::Result<()> {
    let path = exp.ai_prompt_path(name);
    let mut file = File::create(path)?;

    let summary_path = exp.summary_csv_path(name);
    let benchmark_path = exp.benchmark_file_path(name);
    let num_inputs = data.len();
    let input_factor_names = <E::InputFactors as Factors>::factor_names().join(", ");
    let num_variants = variants.len();
    let alg_factor_names = <E::AlgFactors as Factors>::factor_names().join(", ");
    let num_treatments = num_inputs * num_variants;

    let prompt = format!(
        r"
The file at '{summary_path:?}' contains the summary CSV for the '{name}' benchmark.

Each row is one treatment and includes:
- 't': treatment index
- 'i': input-data combination index
- 'a': algorithm-variant index
- input factor columns: {input_factor_names}
- algorithm factor columns: {alg_factor_names}
- 'time (ns)': execution time in nanoseconds (lower is better)

There are {num_inputs} input-data combinations, {num_variants} algorithm variants, and {num_treatments} total treatments.

Times are criterion point estimates, so treat them as benchmark summaries (not single raw run noise).

The benchmark source is at '{benchmark_path:?}'. It defines the input/algorithm factor types that explain the meaning of factor names and levels.

Provide a quick user-facing summary with:
1. the best overall algorithm variant or parameter setting,
2. whether the best choice is consistent across different inputs or depends on input characteristics,
3. the most important factor effects or tradeoffs,
4. notable interactions between input factors and algorithm factors,
5. a concise practical recommendation.

Keep the response concise and evidence-based. Prefer 4-8 bullet points.
Avoid unsupported causal claims; if confidence is low, state assumptions explicitly.
    "
    );

    file.write_all(prompt.as_bytes())?;
    Ok(())
}
