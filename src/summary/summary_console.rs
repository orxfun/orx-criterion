use crate::summary::progress_bar::time_progress_bar;
use crate::{Experiment, Factors};
use cli_table::{Cell, CellStruct, Color, Style, Table, format::Justify, print_stdout};
use colorize::AnsiColor;
use std::cmp::Ordering;
use thousands::Separable;

pub fn print_summary_table<E: Experiment>(
    name: &str,
    input_levels: &[E::InputFactors],
    alg_levels: &[E::AlgFactors],
    estimates: &[Vec<Option<f64>>],
    input_separator_char: &str,
    max_progress_bar_width: usize,
    progress_bar_char: &str,
) {
    let yellow = Some(Color::Rgb(255, 255, 102));
    let cmp = |a: &f64, b: &f64| match a.total_cmp(b) {
        Ordering::Equal => Ordering::Equal,
        ordering => ordering,
    };

    enum Rank {
        Best,
        Worst,
        Intermediate,
        Missing,
    }

    // header
    let mut header = vec![
        "t".cell().bold(true),
        "i".cell().bold(true),
        "a".cell().bold(true),
    ];
    for factor in <E::InputFactors as Factors>::factor_names() {
        header.push(factor.cell().bold(true));
    }
    for param in <E::AlgFactors as Factors>::factor_names() {
        header.push(param.cell().bold(true));
    }
    header.push("time (ns)".cell().bold(true).justify(Justify::Right));
    header.push("time per input".cell().bold(true));
    header.push("time overall".cell().bold(true));
    let header: Vec<_> = header
        .into_iter()
        .map(|x| x.foreground_color(yellow))
        .collect();
    let num_columns = header.len();

    // cells
    let mut rows = vec![];
    let max_time_overall = estimates
        .iter()
        .flat_map(|input_estimates| input_estimates.iter())
        .flatten()
        .copied()
        .max_by(cmp)
        .unwrap_or(0.0);
    for (i, (input_variant, input_estimates)) in input_levels.iter().zip(estimates).enumerate() {
        if i > 0 {
            rows.push({
                let dash = |len: usize, justify: Justify| {
                    input_separator_char
                        .repeat(len)
                        .cell()
                        .foreground_color(yellow)
                        .justify(justify)
                };
                let mut columns: Vec<_> = (0..3).map(|_| dash(1, Justify::Left)).collect(); // t i a
                columns.extend((0..(num_columns - 3 - 2 - 1)).map(|_| dash(5, Justify::Left))); // factors
                columns.push(dash(5, Justify::Right)); // time (ns)
                columns.extend((0..2).map(|_| dash(max_progress_bar_width, Justify::Left))); // bars
                columns
            });
        }

        let values = || input_estimates.iter().map(|x| x.unwrap_or(f64::MAX));
        let min = values().min_by(cmp).unwrap_or(f64::MAX);
        let max = values().max_by(cmp).unwrap_or(f64::MIN);
        let max_time_per_input = input_estimates
            .iter()
            .flatten()
            .copied()
            .max_by(cmp)
            .unwrap_or(0.0);
        let rank_of = |estimate: &Option<f64>| match estimate {
            Some(x) => match ((min - x).abs(), (max - x).abs()) {
                (dif_min, _) if dif_min < 1e-5 => Rank::Best,
                (_, dif_max) if dif_max < 1e-5 => Rank::Worst,
                (_, _) => Rank::Intermediate,
            },
            None => Rank::Missing,
        };
        let cell_of = |rank: &Rank, cell: CellStruct| match rank {
            Rank::Best => cell.bold(true).foreground_color(Some(Color::Green)),
            Rank::Worst => cell.bold(true).foreground_color(Some(Color::Red)),
            Rank::Intermediate => cell,
            Rank::Missing => cell.foreground_color(Some(Color::Rgb(50, 50, 50))),
        };

        let progress_bar = |estimate, max_estimate| {
            time_progress_bar(
                estimate,
                max_estimate,
                max_progress_bar_width,
                progress_bar_char,
            )
        };

        let input_factor_levels = input_variant.factor_levels();
        for (a, (alg_variant, estimate)) in alg_levels.iter().zip(input_estimates).enumerate() {
            let t = i * alg_levels.len() + a;
            let alg_factor_levels = alg_variant.factor_levels();
            let rank = rank_of(estimate);
            let time_bar_per_input = progress_bar(*estimate, max_time_per_input);
            let time_bar_overall = progress_bar(*estimate, max_time_overall);
            let estimate = estimate
                .map(|x| (x.round() as i128).separate_with_commas())
                .unwrap_or("NA".to_string());
            let mut columns = vec![
                cell_of(&rank, (t + 1).cell()),
                cell_of(&rank, (i + 1).cell()),
                cell_of(&rank, (a + 1).cell()),
            ];

            for x in input_factor_levels.iter().chain(&alg_factor_levels) {
                columns.push(cell_of(&rank, x.cell()));
            }
            columns.push(cell_of(&rank, estimate.cell().justify(Justify::Right)));
            columns.push(cell_of(&rank, time_bar_per_input.cell()));
            columns.push(cell_of(&rank, time_bar_overall.cell()));

            rows.push(columns);
        }
    }

    let table = rows.table().title(header);
    let log = format!("\n# {name}");
    println!("{}", log.bold().yellow());
    print_stdout(table).expect("Failed to print the summary table");
}
