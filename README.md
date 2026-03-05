# jira-cli

`jira-cli` is a Jira command runner for agent and CI workflows.

Targets Jira Server / Data Center via REST API v2 with:

- subcommands (`get-issue`, `search`, `create-issue`, `update-issue`, `transition`, `link-issue`, `add-comment`, `list-comments`, `config`)
- JSON output by default with a stable envelope
- PAT-based auth via env var reference (never plaintext)
- Dual-platform skill launcher support (macOS arm64, Linux x86_64)

## Quick usage

```bash
scripts/jira-cli --project-root /path/to/repo config validate
```

```bash
scripts/jira-cli --project-root /path/to/repo get-issue PROJ-123
```

```bash
scripts/jira-cli --project-root /path/to/repo search --jql "project = PROJ AND status = 'In Progress'" --max-results 10
```

## Installing the skill

### 1. Copy the skill to your Claude skills directory

```bash
cp -r skills/jira-cli ~/.claude/skills/jira-cli
```

The skill will be available to Claude Code in all projects immediately.

### 2. Set up a project

In any project where you want to use jira-cli, copy the launcher script and binary:

```bash
mkdir -p scripts/bin
cp skills/jira-cli/scripts/jira-cli scripts/jira-cli
cp skills/jira-cli/scripts/bin/jira-cli-darwin-arm64 scripts/bin/   # macOS arm64
cp skills/jira-cli/scripts/bin/jira-cli-linux-x86_64 scripts/bin/   # Linux x86_64
chmod +x scripts/jira-cli scripts/bin/jira-cli-*
```

### 3. Create config

```bash
mkdir -p .agents/jira-cli
cp skills/jira-cli/references/jira.toml.example .agents/jira-cli/jira.toml
```

Edit `.agents/jira-cli/jira.toml`:

```toml
config_version = 1
base_url = "https://jira.mycompany.com"
token_env = "JIRA_API_TOKEN"
request_timeout_ms = 30000
```

### 4. Set your Personal Access Token

Create `.agents/jira-cli/.env` (never commit this):

```bash
echo 'JIRA_API_TOKEN=your-pat-here' > .agents/jira-cli/.env
```

Add to `.gitignore`:

```
.agents/jira-cli/.env
```

### 5. Validate

```bash
scripts/jira-cli --project-root . config validate
```

See [skills/jira-cli/references/SETUP.md](skills/jira-cli/references/SETUP.md) for the full setup guide including how to generate a PAT and troubleshooting steps.

## Agent safety

- Agents must use `scripts/jira-cli` for all Jira interactions.
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
