use crate::Experiment;
use crate::experiment_sealed::ExperimentSealed;
use crate::summary::{read_from_criterion, summary_ai_prompt, summary_console, summary_csv};
use colorize::AnsiColor;

const MAX_PROGRESS_BAR_WIDTH: usize = 20;
const PROGRESS_BAR_CHAR: &'static str = "█";
const INPUT_SEPRATOR_CHAR: &'static str = "━";

pub fn summarize<E: Experiment>(
    exp: &E,
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
) {
    let estimates =
        read_from_criterion::collect_point_estimates(exp, name, input_levels, alg_levels);

    summary_csv::create_summary_csv(exp, name, input_levels, alg_levels, &estimates)
        .expect("Failed to create csv summary");

    let log = format!(
        "\nSummary table created at:\n{:?}\n",
        exp.summary_csv_path(name)
    );
    println!("{}", log.italic());

    summary_console::print_summary_table::<E>(
        name,
        input_levels,
        alg_levels,
        &estimates,
        INPUT_SEPRATOR_CHAR,
        MAX_PROGRESS_BAR_WIDTH,
        PROGRESS_BAR_CHAR,
    );

    summary_ai_prompt::create_ai_prompt_to_analyze(exp, name, input_levels, alg_levels)
        .expect("Failed to create ai prompt");
    let log = format!(
        "\nA draft AI prompt to analyze the summary table is created at:\n{:?}\n",
        exp.ai_prompt_path(name)
    );
    println!("{}", log.italic());
}
