# jira-cli

Rust CLI for Jira Server/Data Center. Wraps the Jira REST API v2 behind a stable JSON envelope for agent and CI use.

## Build

```bash
cargo build --release
```

## Project structure

- `src/main.rs` — CLI entrypoint, config loading, command dispatch, output envelope
- `src/client.rs` — `JiraClient` (reqwest blocking, Bearer auth)
- `src/types.rs` — Serde structs for Jira API responses
- `skills/jira-cli/` — agent skill (SKILL.md, launcher script, binaries, references)

## Release binaries

After `cargo build --release`, copy the binary into the skill:

```bash
cp target/release/jira-cli skills/jira-cli/scripts/bin/jira-cli-darwin-arm64   # macOS arm64
cp target/release/jira-cli skills/jira-cli/scripts/bin/jira-cli-linux-x86_64   # Linux x86_64
```

## Config location (for testing)

`.agents/jira-cli/jira.toml` + `.agents/jira-cli/.env` relative to project root.
