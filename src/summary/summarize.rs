use crate::Experiment;
use crate::experiment_sealed::ExperimentSealed;
use crate::summary::{read_from_criterion, summary_ai_prompt, summary_console, summary_csv};
use colorize::AnsiColor;

const MAX_PROGRESS_BAR_WIDTH: usize = 20;
const PROGRESS_BAR_CHAR: &str = "█";
const INPUT_SEPRATOR_CHAR: &str = "━";

fn print_summary_table<E: Experiment>(
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
    estimates: &[Vec<Option<f64>>],
) {
    summary_console::print_summary_table::<E>(
        name,
        input_levels,
        alg_levels,
        estimates,
        INPUT_SEPRATOR_CHAR,
        MAX_PROGRESS_BAR_WIDTH,
        PROGRESS_BAR_CHAR,
    );
}

fn log(s: String) {
    println!("{}", s.italic());
}

pub fn summarize<E: Experiment>(
    exp: &E,
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
) {
    let estimates =
        read_from_criterion::collect_point_estimates(exp, name, input_levels, alg_levels);

    // csv
    summary_csv::create_summary_csv(exp, name, input_levels, alg_levels, &estimates)
        .expect("Failed to create csv summary");
    let summary_csv_path = exp.summary_csv_path(name);
    let msg = format!("\nSummary table created at:\n{summary_csv_path:?}\n");
    log(msg);

    // console
    print_summary_table::<E>(name, input_levels, alg_levels, &estimates);

    // prompt
    summary_ai_prompt::create_ai_prompt_to_analyze(exp, name, input_levels, alg_levels)
        .expect("Failed to create ai prompt");
    let ai_prompt_path = exp.ai_prompt_path(name);
    let msg = format!(
        "\nA draft AI prompt to analyze the summary table is created at:\n{:?}\n",
        ai_prompt_path
    );
    log(msg);
}
