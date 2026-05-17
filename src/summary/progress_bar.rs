pub fn time_progress_bar(
    estimate: Option<f64>,
    max_estimate: f64,
    max_progress_bar_width: usize,
    progress_bar_char: &str,
) -> String {
    match estimate {
        Some(value) if max_estimate > 0.0 => {
            let ratio = (value / max_estimate).clamp(0.0, 1.0);
            let mut width = (ratio * max_progress_bar_width as f64).round() as usize;
            if value > 0.0 && width == 0 {
                width = 1;
            }
            progress_bar_char.repeat(width)
        }
        _ => String::new(),
    }
}
