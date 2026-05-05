use std::io::{self, Read};

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_FG_BLACK: &str = "\x1b[38;5;0m";
const ANSI_BG_GREEN: &str = "\x1b[48;5;34m";
const ANSI_BG_YELLOW: &str = "\x1b[48;5;220m";
const ANSI_BG_RED: &str = "\x1b[48;5;196m";

fn format_percentage_segment(label: &str, value: f64) -> String {
    let bg_color = if value <= 50.0 {
        ANSI_BG_GREEN
    } else if value <= 80.0 {
        ANSI_BG_YELLOW
    } else {
        ANSI_BG_RED
    };

    format!(
        "{bg_color}{ANSI_FG_BLACK} {label} {}% {ANSI_RESET}",
        value.ceil()
    )
}

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&input) else {
        return;
    };

    let mut segments = Vec::new();

    if let Some(value) = json
        .get("context_window")
        .and_then(|context_window| context_window.get("used_percentage"))
        .and_then(|used_percentage| used_percentage.as_f64())
    {
        segments.push(format_percentage_segment("ctx", value));
    }

    if let Some(value) = json
        .get("rate_limits")
        .and_then(|rate_limits| rate_limits.get("five_hour"))
        .and_then(|five_hour| five_hour.get("used_percentage"))
        .and_then(|used_percentage| used_percentage.as_f64())
    {
        segments.push(format_percentage_segment("5h", value));
    }

    if let Some(value) = json
        .get("rate_limits")
        .and_then(|rate_limits| rate_limits.get("seven_day"))
        .and_then(|seven_day| seven_day.get("used_percentage"))
        .and_then(|used_percentage| used_percentage.as_f64())
    {
        segments.push(format_percentage_segment("7d", value));
    }

    if !segments.is_empty() {
        println!("{}", segments.join(" "));
    }
}
