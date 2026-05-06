# claude-status-line

A small Rust status-line formatter for Claude Code.

It reads Claude Code status JSON from stdin and prints compact ANSI-colored
segments for:

- model and effort level
- worktree branch
- context window usage
- 5-hour and 7-day rate limit usage

Percentage segments are rounded up and colored by usage:

- green: up to 50%
- yellow: up to 80%
- red: above 80%

## Usage

```bash
claude-status-line < tests/fixtures/schema.json
```

Example output includes ANSI styling and segments like:

```text
worktree-my-feature ctx 81% 5h 24% 7d 42% Opus|high
```

## Development

```bash
cargo test
cargo run --quiet < tests/fixtures/schema.json
```

## License

MIT
