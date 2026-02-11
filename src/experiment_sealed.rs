use crate::{AlgFactors, Experimentation, InputFactors};
use std::path::PathBuf;

pub trait ExperimentSealed: Experimentation {
    /// Long key of the treatment, or run, for the input defined by the `input_variant` and algorithm
    /// defined by the `algorithm_variant`.
    fn run_key_long(
        &self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
    ) -> String {
        format!("{}/{}", input_variant.key_long(), alg_variant.key_long())
    }

    /// Short key of the treatment, or run, for the input defined by the `input_variant` and algorithm
    /// defined by the `algorithm_variant`.
    fn run_key_short(
        &self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
    ) -> String {
        format!("{}/{}", input_variant.key_short(), alg_variant.key_short())
    }

    /// Path of the "estimates.json" file that criterion will create when the benchmark is created,
    /// for the particular treatment defined by the given `input_variant` and `alg_variant`.
    fn run_estimates_path(
        &self,
        bench_name: &str,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
    ) -> PathBuf {
        let execution_path = self
            .run_key_short(input_variant, alg_variant)
            .replace("/", "_")
            .replace(":", "_");
        [
            "target",
            "criterion",
            bench_name,
            &execution_path,
            "new",
            "estimates.json",
        ]
        .iter()
        .collect()
    }

    /// Path of the benchmark file including this experiment.
    fn benchmark_file_path(&self, bench_name: &str) -> PathBuf {
        ["benches", &format!("{bench_name}.rs")].iter().collect()
    }

    /// Path of the csv file containing the summary table that will be created at the end of the
    /// benchmark execution.
    fn summary_csv_path(&self, bench_name: &str) -> PathBuf {
        [
            "target",
            "criterion",
            bench_name,
            &format!("summary_{bench_name}.csv"),
        ]
        .iter()
        .collect()
    }

    /// Path of the markdown file containing a draft AI prompt to analyze the summary file which
    /// will also be created at the end of the benchmark execution.
    fn ai_prompt_path(&self, bench_name: &str) -> PathBuf {
        [
            "target",
            "criterion",
            bench_name,
            &format!("prompt_{bench_name}.md"),
        ]
        .iter()
        .collect()
    }
}

impl<X: Experimentation> ExperimentSealed for X {}
