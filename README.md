# jira-cli

`jira-cli` is a Jira command runner for agent and CI workflows.

Targets Jira Server / Data Center via REST API v2 with:

- subcommands (`get-issue`, `search`, `create-issue`, `update-issue`, `transition`, `link-issue`, `add-comment`, `list-comments`, `config`)
- JSON output by default with a stable envelope
- PAT-based auth via env var reference (never plaintext)
- Dual-platform skill launcher support (macOS arm64, Linux x86_64)

## Install

```bash
npx skills add dobby/jira-cli
```

Then configure your project (see [skills/jira-cli/references/SETUP.md](skills/jira-cli/references/SETUP.md)):

```bash
mkdir -p .agents/jira-cli
cp skills/jira-cli/references/jira.toml.example .agents/jira-cli/jira.toml
# edit jira.toml: set base_url to your Jira instance
echo 'JIRA_API_TOKEN=your-pat-here' > .agents/jira-cli/.env
```

Validate:

```bash
skills/jira-cli/scripts/jira-cli --project-root . config validate
```

## Quick usage

```bash
skills/jira-cli/scripts/jira-cli --project-root /path/to/repo get-issue PROJ-123
```

```bash
skills/jira-cli/scripts/jira-cli --project-root /path/to/repo search --jql "project = PROJ AND status = 'In Progress'" --max-results 10
```

```bash
skills/jira-cli/scripts/jira-cli --project-root /path/to/repo create-issue --project PROJ --type Bug --title "Login fails on Safari" --priority High
```

```bash
skills/jira-cli/scripts/jira-cli --project-root /path/to/repo transition PROJ-123 --status-name "In Progress"
```

## Agent safety

- Agents must use `skills/jira-cli/scripts/jira-cli` for all Jira interactions.
- Agents must not call the Jira API directly (curl, fetch, etc.).
- Agents must not read `.agents/jira-cli/jira.toml` or `.env` files directly.

## Output envelope

All commands return a stable JSON envelope:

```json
{
  "version": "0.1.0",
  "ok": true,
  "command": "get-issue",
  "data": { ... },
  "meta": { "duration_ms": 42 }
}
```

## Repository layout

- `src/main.rs` — Rust CLI implementation
- `src/client.rs` — Jira REST API HTTP client
- `src/types.rs` — Serde structs for Jira API responses
- `skills/jira-cli/SKILL.md` — skill instructions for agents
- `skills/jira-cli/scripts/jira-cli` — platform launcher script
- `skills/jira-cli/scripts/bin/` — prebuilt binaries
- `skills/jira-cli/references/jira.toml.example` — starter config
- `skills/jira-cli/references/SETUP.md` — setup and usage guide
- `.github/workflows/build-release.yml` — CI + release pipeline
