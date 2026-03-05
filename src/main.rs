use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

mod client;
mod types;

use client::JiraClient;

const CONFIG_DIR_NAME: &str = "jira-cli";
const AGENT_DIR_NAME: &str = ".agents";
const VERSION: &str = "0.1.0";

// ── Error types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum ExitKind {
    Cli = 2,
    Config = 3,
    #[allow(dead_code)]
    Api = 4,
    Runtime = 5,
}

#[derive(Debug)]
pub struct AppError {
    exit: ExitKind,
    code: &'static str,
    message: String,
}

impl AppError {
    fn cli(code: &'static str, message: impl Into<String>) -> Self {
        Self { exit: ExitKind::Cli, code, message: message.into() }
    }

    fn config(code: &'static str, message: impl Into<String>) -> Self {
        Self { exit: ExitKind::Config, code, message: message.into() }
    }

    pub fn runtime(code: &'static str, message: impl Into<String>) -> Self {
        Self { exit: ExitKind::Runtime, code, message: message.into() }
    }
}

// ── Config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct Config {
    #[allow(dead_code)]
    config_version: Option<u32>,
    base_url: String,
    token_env: String,
    request_timeout_ms: Option<u64>,
}

// ── Output envelopes ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct SuccessEnvelope {
    version: &'static str,
    ok: bool,
    command: String,
    data: Value,
    meta: ResponseMeta,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope {
    version: &'static str,
    ok: bool,
    command: String,
    error: ErrorPayload,
}

#[derive(Debug, Serialize)]
struct ErrorPayload {
    code: String,
    message: String,
}

#[derive(Debug, Default, Serialize)]
struct ResponseMeta {
    duration_ms: u128,
}

// ── CLI definition ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lower")]
enum OutputFormat {
    Json,
    Text,
}

#[derive(Debug, Parser)]
#[command(name = "jira-cli", version, about = "Jira CLI for agent and CI workflows")]
struct Cli {
    #[arg(long, global = true)]
    project_root: Option<PathBuf>,
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
    #[arg(long, global = true)]
    output: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Fetch a single issue by key
    GetIssue(GetIssueArgs),
    /// Search issues with JQL
    Search(SearchArgs),
    /// Create a new issue
    CreateIssue(CreateIssueArgs),
    /// Update an existing issue
    UpdateIssue(UpdateIssueArgs),
    /// Transition an issue to a new status
    Transition(TransitionArgs),
    /// List available transitions for an issue
    ListTransitions(IssueKeyArgs),
    /// Link two issues
    LinkIssue(LinkIssueArgs),
    /// List all available issue link types
    ListLinkTypes,
    /// Add a comment to an issue
    AddComment(AddCommentArgs),
    /// List comments on an issue
    ListComments(IssueKeyArgs),
    /// Config subcommands
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
}

#[derive(Debug, Args)]
struct IssueKeyArgs {
    /// Issue key, e.g. PROJ-123
    key: String,
}

#[derive(Debug, Args)]
struct GetIssueArgs {
    /// Issue key, e.g. PROJ-123
    key: String,
    /// Comma-separated fields to include (default: all)
    #[arg(long)]
    fields: Option<String>,
}

#[derive(Debug, Args)]
struct SearchArgs {
    /// JQL query string
    #[arg(long)]
    jql: String,
    /// Maximum number of results
    #[arg(long, default_value_t = 50)]
    max_results: u32,
    /// Fields to include (comma-separated)
    #[arg(long)]
    fields: Option<String>,
}

#[derive(Debug, Args)]
struct CreateIssueArgs {
    /// Project key
    #[arg(long)]
    project: String,
    /// Issue type (e.g. Bug, Task, Story)
    #[arg(long = "type")]
    issue_type: String,
    /// Issue title / summary
    #[arg(long)]
    title: String,
    /// Issue description
    #[arg(long)]
    description: Option<String>,
    /// Assignee username
    #[arg(long)]
    assignee: Option<String>,
    /// Priority name (e.g. High, Medium, Low)
    #[arg(long)]
    priority: Option<String>,
}

#[derive(Debug, Args)]
struct UpdateIssueArgs {
    /// Issue key to update
    key: String,
    /// New summary / title
    #[arg(long)]
    title: Option<String>,
    /// New description
    #[arg(long)]
    description: Option<String>,
    /// New assignee username
    #[arg(long)]
    assignee: Option<String>,
    /// New priority name
    #[arg(long)]
    priority: Option<String>,
}

#[derive(Debug, Args)]
struct TransitionArgs {
    /// Issue key to transition
    key: String,
    /// Transition ID (numeric)
    #[arg(long, conflicts_with = "status_name")]
    transition_id: Option<String>,
    /// Target status name (resolved by listing transitions first)
    #[arg(long)]
    status_name: Option<String>,
}

#[derive(Debug, Args)]
struct LinkIssueArgs {
    /// Source issue key
    key: String,
    /// Link type name (e.g. "blocks", "is blocked by", "relates to")
    #[arg(long)]
    link_type: String,
    /// Target issue key
    #[arg(long)]
    target: String,
}

#[derive(Debug, Args)]
struct AddCommentArgs {
    /// Issue key
    key: String,
    /// Comment body text
    #[arg(long)]
    body: String,
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    /// Validate config and connectivity (calls GET /rest/api/2/myself)
    Validate,
}

// ── Entry point ────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    let project_root = cli
        .project_root
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let command_name = command_name(&cli.command);
    let start = Instant::now();

    let result = run(&cli, &project_root, &command_name, start);

    match result {
        Ok(data) => {
            let duration_ms = start.elapsed().as_millis();
            let envelope = SuccessEnvelope {
                version: VERSION,
                ok: true,
                command: command_name.clone(),
                data,
                meta: ResponseMeta { duration_ms },
            };
            emit_output(&envelope, cli.format, cli.output.as_deref(), &command_name, true);
        }
        Err(err) => {
            let envelope = ErrorEnvelope {
                version: VERSION,
                ok: false,
                command: command_name,
                error: ErrorPayload {
                    code: err.code.to_string(),
                    message: err.message.clone(),
                },
            };
            let exit_code = err.exit as i32;
            emit_output(&envelope, cli.format, cli.output.as_deref(), "", false);
            std::process::exit(exit_code);
        }
    }
}

fn command_name(cmd: &Commands) -> String {
    match cmd {
        Commands::GetIssue(_) => "get-issue",
        Commands::Search(_) => "search",
        Commands::CreateIssue(_) => "create-issue",
        Commands::UpdateIssue(_) => "update-issue",
        Commands::Transition(_) => "transition",
        Commands::ListTransitions(_) => "list-transitions",
        Commands::LinkIssue(_) => "link-issue",
        Commands::ListLinkTypes => "list-link-types",
        Commands::AddComment(_) => "add-comment",
        Commands::ListComments(_) => "list-comments",
        Commands::Config { command: ConfigCommand::Validate } => "config validate",
    }
    .to_string()
}

fn run(
    cli: &Cli,
    project_root: &Path,
    command_name: &str,
    _start: Instant,
) -> Result<Value, AppError> {
    // Load config & create client (all commands need it)
    let config = load_config(project_root)?;
    let token = std::env::var(&config.token_env).map_err(|_| {
        AppError::config(
            "TOKEN_ENV_MISSING",
            format!(
                "Environment variable '{}' not set (referenced by token_env in config)",
                config.token_env
            ),
        )
    })?;

    if token.trim().is_empty() {
        return Err(AppError::config("TOKEN_EMPTY", format!(
            "Environment variable '{}' is set but empty", config.token_env
        )));
    }

    let timeout_ms = config.request_timeout_ms.unwrap_or(30_000);
    let client = JiraClient::new(&config.base_url, &token, timeout_ms)?;

    match &cli.command {
        Commands::Config { command: ConfigCommand::Validate } => cmd_config_validate(&client),

        Commands::GetIssue(args) => cmd_get_issue(&client, args),
        Commands::Search(args) => cmd_search(&client, args),
        Commands::CreateIssue(args) => cmd_create_issue(&client, args),
        Commands::UpdateIssue(args) => cmd_update_issue(&client, args),
        Commands::Transition(args) => cmd_transition(&client, args),
        Commands::ListTransitions(args) => cmd_list_transitions(&client, args),
        Commands::LinkIssue(args) => cmd_link_issue(&client, args),
        Commands::ListLinkTypes => cmd_list_link_types(&client),
        Commands::AddComment(args) => cmd_add_comment(&client, args),
        Commands::ListComments(args) => cmd_list_comments(&client, args),
    }
    .map_err(|e| {
        // Annotate error with command context
        AppError { message: format!("[{}] {}", command_name, e.message), ..e }
    })
}

// ── Config loading ─────────────────────────────────────────────────────────

fn load_config(project_root: &Path) -> Result<Config, AppError> {
    // Load .env files before reading config
    load_env_files(project_root);

    let config_path = project_root
        .join(AGENT_DIR_NAME)
        .join(CONFIG_DIR_NAME)
        .join("jira.toml");

    if !config_path.exists() {
        return Err(AppError::config(
            "CONFIG_NOT_FOUND",
            format!(
                "Config file not found: {}\nCreate it at .agents/jira-cli/jira.toml",
                config_path.display()
            ),
        ));
    }

    let raw = fs::read_to_string(&config_path).map_err(|e| {
        AppError::config("CONFIG_UNREADABLE", format!("Cannot read {}: {}", config_path.display(), e))
    })?;

    let config: Config = toml::from_str(&raw).map_err(|e| {
        AppError::config("CONFIG_PARSE_FAILED", format!("Invalid TOML in {}: {}", config_path.display(), e))
    })?;

    if config.base_url.trim().is_empty() {
        return Err(AppError::config("CONFIG_INVALID", "base_url must not be empty"));
    }
    if config.token_env.trim().is_empty() {
        return Err(AppError::config("CONFIG_INVALID", "token_env must not be empty"));
    }

    Ok(config)
}

fn load_env_files(project_root: &Path) {
    // Load .agents/jira-cli/.env first, then fall back to root .env
    let agent_env = project_root.join(AGENT_DIR_NAME).join(CONFIG_DIR_NAME).join(".env");
    let root_env = project_root.join(".env");

    // dotenvy loads without overriding already-set env vars
    let _ = dotenvy::from_path(&agent_env);
    let _ = dotenvy::from_path(&root_env);
}

// ── Command implementations ────────────────────────────────────────────────

fn cmd_config_validate(client: &JiraClient) -> Result<Value, AppError> {
    let myself: types::Myself = client.get("myself")?;
    Ok(json!({
        "connected": true,
        "user": myself,
    }))
}

fn cmd_get_issue(client: &JiraClient, args: &GetIssueArgs) -> Result<Value, AppError> {
    let path = if let Some(fields) = &args.fields {
        format!("issue/{}?fields={}", args.key, fields)
    } else {
        format!("issue/{}", args.key)
    };
    let issue: types::Issue = client.get(&path)?;
    Ok(serde_json::to_value(issue).unwrap_or(json!({})))
}

fn cmd_search(client: &JiraClient, args: &SearchArgs) -> Result<Value, AppError> {
    let mut params: Vec<(&str, String)> = vec![
        ("jql", args.jql.clone()),
        ("maxResults", args.max_results.to_string()),
    ];
    if let Some(fields) = &args.fields {
        params.push(("fields", fields.clone()));
    }
    let results: types::SearchResults = client.get_with_params("search", &params)?;
    Ok(serde_json::to_value(results).unwrap_or(json!({})))
}

fn cmd_create_issue(client: &JiraClient, args: &CreateIssueArgs) -> Result<Value, AppError> {
    let mut fields = json!({
        "project": { "key": args.project },
        "issuetype": { "name": args.issue_type },
        "summary": args.title,
    });

    if let Some(desc) = &args.description {
        fields["description"] = json!(desc);
    }
    if let Some(assignee) = &args.assignee {
        fields["assignee"] = json!({ "name": assignee });
    }
    if let Some(priority) = &args.priority {
        fields["priority"] = json!({ "name": priority });
    }

    let body = json!({ "fields": fields });
    let result: Value = client.post("issue", &body)?;
    Ok(result)
}

fn cmd_update_issue(client: &JiraClient, args: &UpdateIssueArgs) -> Result<Value, AppError> {
    let mut update_fields = serde_json::Map::new();

    if let Some(title) = &args.title {
        update_fields.insert("summary".into(), json!([{"set": title}]));
    }
    if let Some(desc) = &args.description {
        update_fields.insert("description".into(), json!([{"set": desc}]));
    }
    if let Some(assignee) = &args.assignee {
        update_fields.insert("assignee".into(), json!([{"set": {"name": assignee}}]));
    }
    if let Some(priority) = &args.priority {
        update_fields.insert("priority".into(), json!([{"set": {"name": priority}}]));
    }

    if update_fields.is_empty() {
        return Err(AppError::cli(
            "NO_FIELDS_TO_UPDATE",
            "Provide at least one of --title, --description, --assignee, --priority",
        ));
    }

    let body = json!({ "update": update_fields });
    let path = format!("issue/{}", args.key);
    client.put_no_body(&path, &body)?;
    Ok(json!({ "updated": true, "key": args.key }))
}

fn cmd_transition(client: &JiraClient, args: &TransitionArgs) -> Result<Value, AppError> {
    let transition_id = if let Some(id) = &args.transition_id {
        id.clone()
    } else if let Some(name) = &args.status_name {
        // Resolve by listing transitions
        let list_args = IssueKeyArgs { key: args.key.clone() };
        let trans_resp: types::TransitionsResponse =
            client.get(&format!("issue/{}/transitions", args.key))?;
        let name_lower = name.to_lowercase();
        trans_resp
            .transitions
            .iter()
            .find(|t| t.name.to_lowercase() == name_lower)
            .map(|t| t.id.clone())
            .ok_or_else(|| {
                AppError::cli(
                    "TRANSITION_NOT_FOUND",
                    format!(
                        "No transition named '{}' found. Use list-transitions {} to see available transitions.",
                        name, list_args.key
                    ),
                )
            })?
    } else {
        return Err(AppError::cli(
            "TRANSITION_MISSING",
            "Provide --transition-id or --status-name",
        ));
    };

    let body = json!({ "transition": { "id": transition_id } });
    let path = format!("issue/{}/transitions", args.key);
    client.post_no_body(&path, &body)?;
    Ok(json!({ "transitioned": true, "key": args.key, "transition_id": transition_id }))
}

fn cmd_list_transitions(client: &JiraClient, args: &IssueKeyArgs) -> Result<Value, AppError> {
    let resp: types::TransitionsResponse = client.get(&format!("issue/{}/transitions", args.key))?;
    Ok(serde_json::to_value(resp).unwrap_or(json!({})))
}

fn cmd_link_issue(client: &JiraClient, args: &LinkIssueArgs) -> Result<Value, AppError> {
    let body = json!({
        "type": { "name": args.link_type },
        "inwardIssue": { "key": args.key },
        "outwardIssue": { "key": args.target },
    });
    client.post_no_body("issueLink", &body)?;
    Ok(json!({ "linked": true, "from": args.key, "to": args.target, "type": args.link_type }))
}

fn cmd_list_link_types(client: &JiraClient) -> Result<Value, AppError> {
    let resp: types::LinkTypesResponse = client.get("issueLinkType")?;
    Ok(serde_json::to_value(resp).unwrap_or(json!({})))
}

fn cmd_add_comment(client: &JiraClient, args: &AddCommentArgs) -> Result<Value, AppError> {
    let body = json!({ "body": args.body });
    let path = format!("issue/{}/comment", args.key);
    let result: Value = client.post(&path, &body)?;
    Ok(result)
}

fn cmd_list_comments(client: &JiraClient, args: &IssueKeyArgs) -> Result<Value, AppError> {
    let resp: types::CommentPage = client.get(&format!("issue/{}/comment", args.key))?;
    Ok(serde_json::to_value(resp).unwrap_or(json!({})))
}

// ── Output ─────────────────────────────────────────────────────────────────

fn emit_output<T: Serialize>(
    envelope: &T,
    format: OutputFormat,
    output_path: Option<&Path>,
    _command: &str,
    ok: bool,
) {
    let content = match format {
        OutputFormat::Json => serde_json::to_string_pretty(envelope)
            .unwrap_or_else(|e| format!("{{\"error\": \"serialization failed: {e}\"}}"))
            + "\n",
        OutputFormat::Text => {
            // For text mode, pretty-print JSON (good enough for agent use)
            serde_json::to_string_pretty(envelope)
                .unwrap_or_else(|e| format!("error: serialization failed: {e}"))
                + "\n"
        }
    };

    if let Some(path) = output_path {
        if let Err(e) = fs::write(path, &content) {
            eprintln!("Warning: failed to write output to {}: {}", path.display(), e);
        }
        return;
    }

    if ok {
        print!("{content}");
    } else {
        eprint!("{content}");
    }
}
