# jira-cli Setup Guide

## 1. Create config directory

Inside your project, create the config directory:

```bash
mkdir -p .agents/jira-cli
```

## 2. Create `jira.toml`

Copy the example and edit it:

```bash
cp path/to/skills/jira-cli/references/jira.toml.example .agents/jira-cli/jira.toml
```

Edit `.agents/jira-cli/jira.toml`:

```toml
config_version = 1
base_url = "https://jira.mycompany.com"
token_env = "JIRA_API_TOKEN"
request_timeout_ms = 30000
```

## 3. Create a Personal Access Token (PAT)

In Jira Server / Data Center:

1. Log in to Jira.
2. Go to your profile → **Personal Access Tokens** (or **Account Settings** → **Security**).
3. Click **Create token**, give it a name (e.g. "jira-cli agent"), and copy the token value.

## 4. Set the token environment variable

Create `.agents/jira-cli/.env` (never commit this file):

```bash
JIRA_API_TOKEN=your-personal-access-token-here
```

Or set it in your shell / CI environment:

```bash
export JIRA_API_TOKEN=your-personal-access-token-here
```

Add `.agents/jira-cli/.env` to your `.gitignore`.

## 5. Validate the connection

```bash
scripts/jira-cli --project-root . config validate
```

Expected output:

```json
{
  "version": "0.1.0",
  "ok": true,
  "command": "config validate",
  "data": {
    "connected": true,
    "user": {
      "name": "jsmith",
      "displayName": "Jane Smith",
      "emailAddress": "jsmith@mycompany.com"
    }
  },
  "meta": { "duration_ms": 123 }
}
```

## 6. Gitignore recommendations

Add to `.gitignore`:

```
.agents/jira-cli/.env
.env
```

## Troubleshooting

| Error code | Meaning | Fix |
|---|---|---|
| `CONFIG_NOT_FOUND` | `.agents/jira-cli/jira.toml` missing | Create the config file |
| `TOKEN_ENV_MISSING` | Env var not set | Set `JIRA_API_TOKEN` |
| `JIRA_API_ERROR` HTTP 401 | Invalid or expired token | Regenerate PAT in Jira |
| `JIRA_API_ERROR` HTTP 403 | Insufficient permissions | Check Jira project permissions |
| `JIRA_API_ERROR` HTTP 404 | Issue not found | Verify the issue key |
