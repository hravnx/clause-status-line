use std::io::{self, Read};

fn main() {
    let mut input = String::new();
    if io::stdin().read_to_string(&mut input).is_err() {
        return;
    }

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&input) else {
        return;
    };

    if let Some(status_line) = claude_status_line::format_status_line(&json) {
        println!("{status_line}");
    }
}
