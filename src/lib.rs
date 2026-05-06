use std::process::Command;

const ANSI_RESET: &str = "\x1b[0m";
const ANSI_FG_BLACK: &str = "\x1b[38;5;0m";
const ANSI_FG_WHITE: &str = "\x1b[38;5;15m";
const ANSI_BG_BLUE: &str = "\x1b[48;5;24m";
const ANSI_BG_PURPLE: &str = "\x1b[48;5;60m";
const ANSI_BG_GREEN: &str = "\x1b[48;5;34m";
const ANSI_BG_YELLOW: &str = "\x1b[48;5;220m";
const ANSI_BG_RED: &str = "\x1b[48;5;196m";

pub fn format_status_line(json: &serde_json::Value) -> Option<String> {
    let segments = status_segments(json);

    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" "))
    }
}

fn status_segments(json: &serde_json::Value) -> Vec<String> {
    let mut segments = Vec::new();

    if let Some(branch) = active_branch(json) {
        segments.push(format_branch_segment(&branch));
    }

    add_percentage_segment(
        &mut segments,
        json,
        "ctx",
        &["context_window", "used_percentage"],
    );
    add_percentage_segment(
        &mut segments,
        json,
        "5h",
        &["rate_limits", "five_hour", "used_percentage"],
    );
    add_percentage_segment(
        &mut segments,
        json,
        "7d",
        &["rate_limits", "seven_day", "used_percentage"],
    );

    if let (Some(model_name), Some(effort_level)) = (
        string_at(json, &["model", "display_name"]),
        string_at(json, &["effort", "level"]),
    ) {
        segments.push(format_model_segment(model_name, effort_level));
    }

    segments
}

fn add_percentage_segment(
    segments: &mut Vec<String>,
    json: &serde_json::Value,
    label: &str,
    path: &[&str],
) {
    if let Some(value) = percentage_at(json, path) {
        segments.push(format_percentage_segment(label, value));
    }
}

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

fn format_model_segment(model_name: &str, effort_level: &str) -> String {
    format!("{ANSI_BG_BLUE}{ANSI_FG_WHITE} {model_name}|{effort_level} {ANSI_RESET}")
}

fn format_branch_segment(branch: &str) -> String {
    format!("{ANSI_BG_PURPLE}{ANSI_FG_WHITE} {branch} {ANSI_RESET}")
}

fn percentage_at(json: &serde_json::Value, path: &[&str]) -> Option<f64> {
    path.iter()
        .try_fold(json, |value, key| value.get(*key))
        .and_then(|value| value.as_f64())
}

fn string_at<'a>(json: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    path.iter()
        .try_fold(json, |value, key| value.get(*key))
        .and_then(|value| value.as_str())
}

fn git_branch(cwd: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", cwd, "branch", "--show-current"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if branch.is_empty() {
        None
    } else {
        Some(branch)
    }
}

fn active_branch_with<F>(json: &serde_json::Value, git_branch: F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    string_at(json, &["worktree", "branch"])
        .filter(|branch| !branch.is_empty())
        .map(str::to_string)
        .or_else(|| string_at(json, &["cwd"]).and_then(&git_branch))
        .or_else(|| string_at(json, &["workspace", "current_dir"]).and_then(git_branch))
}

fn active_branch(json: &serde_json::Value) -> Option<String> {
    active_branch_with(json, git_branch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_percentage_segments_with_threshold_colors() {
        assert_eq!(
            format_percentage_segment("ctx", 50.0),
            "\x1b[48;5;34m\x1b[38;5;0m ctx 50% \x1b[0m"
        );
        assert_eq!(
            format_percentage_segment("ctx", 50.1),
            "\x1b[48;5;220m\x1b[38;5;0m ctx 51% \x1b[0m"
        );
        assert_eq!(
            format_percentage_segment("ctx", 80.0),
            "\x1b[48;5;220m\x1b[38;5;0m ctx 80% \x1b[0m"
        );
        assert_eq!(
            format_percentage_segment("ctx", 80.1),
            "\x1b[48;5;196m\x1b[38;5;0m ctx 81% \x1b[0m"
        );
    }

    #[test]
    fn formats_model_and_branch_segments() {
        assert_eq!(
            format_model_segment("Opus", "high"),
            "\x1b[48;5;24m\x1b[38;5;15m Opus|high \x1b[0m"
        );
        assert_eq!(
            format_branch_segment("main"),
            "\x1b[48;5;60m\x1b[38;5;15m main \x1b[0m"
        );
    }

    #[test]
    fn reads_nested_percentages_and_strings() {
        let status = json!({
            "context_window": { "used_percentage": 42.5 },
            "model": { "display_name": "Sonnet" }
        });

        assert_eq!(
            percentage_at(&status, &["context_window", "used_percentage"]),
            Some(42.5)
        );
        assert_eq!(
            string_at(&status, &["model", "display_name"]),
            Some("Sonnet")
        );
        assert_eq!(percentage_at(&status, &["missing"]), None);
        assert_eq!(
            string_at(&status, &["context_window", "used_percentage"]),
            None
        );
    }

    #[test]
    fn builds_status_line_from_json_fields() {
        let status = json!({
            "worktree": { "branch": "feature" },
            "context_window": { "used_percentage": 50.1 },
            "rate_limits": {
                "five_hour": { "used_percentage": 23.5 },
                "seven_day": { "used_percentage": 80.1 }
            },
            "model": { "display_name": "Opus" },
            "effort": { "level": "high" }
        });

        assert_eq!(
            format_status_line(&status),
            Some(
                [
                    "\x1b[48;5;60m\x1b[38;5;15m feature \x1b[0m",
                    "\x1b[48;5;220m\x1b[38;5;0m ctx 51% \x1b[0m",
                    "\x1b[48;5;34m\x1b[38;5;0m 5h 24% \x1b[0m",
                    "\x1b[48;5;196m\x1b[38;5;0m 7d 81% \x1b[0m",
                    "\x1b[48;5;24m\x1b[38;5;15m Opus|high \x1b[0m",
                ]
                .join(" ")
            )
        );
    }

    #[test]
    fn returns_none_when_no_segments_can_be_built() {
        assert_eq!(format_status_line(&json!({})), None);
    }

    #[test]
    fn prefers_worktree_branch_over_git_fallback() {
        let status = json!({
            "cwd": "/repo",
            "worktree": { "branch": "worktree-feature" }
        });

        let branch = active_branch_with(&status, |_| Some("main".to_string()));

        assert_eq!(branch, Some("worktree-feature".to_string()));
    }

    #[test]
    fn falls_back_to_cwd_git_branch() {
        let status = json!({
            "cwd": "/repo",
            "workspace": { "current_dir": "/workspace" }
        });

        let branch = active_branch_with(&status, |cwd| {
            assert_eq!(cwd, "/repo");
            Some("main".to_string())
        });

        assert_eq!(branch, Some("main".to_string()));
    }

    #[test]
    fn falls_back_to_workspace_current_dir_when_cwd_git_fails() {
        let status = json!({
            "cwd": "/not-a-repo",
            "workspace": { "current_dir": "/repo" }
        });

        let branch = active_branch_with(&status, |cwd| match cwd {
            "/not-a-repo" => None,
            "/repo" => Some("develop".to_string()),
            _ => panic!("unexpected cwd: {cwd}"),
        });

        assert_eq!(branch, Some("develop".to_string()));
    }
}
