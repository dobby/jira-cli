use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── Issue ──────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Issue {
    pub id: String,
    pub key: String,
    pub fields: IssueFields,
    #[serde(rename = "self")]
    pub self_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IssueFields {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub status: Option<StatusField>,
    pub issuetype: Option<IssueTypeField>,
    pub assignee: Option<UserField>,
    pub reporter: Option<UserField>,
    pub priority: Option<PriorityField>,
    pub project: Option<ProjectField>,
    pub comment: Option<CommentPage>,
    pub labels: Option<Vec<String>>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub issuelinks: Option<Vec<IssueLink>>,
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatusField {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IssueTypeField {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserField {
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PriorityField {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectField {
    pub id: Option<String>,
    pub key: Option<String>,
    pub name: Option<String>,
}

// ── Comments ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct CommentPage {
    pub comments: Vec<Comment>,
    #[serde(rename = "startAt")]
    pub start_at: Option<u32>,
    #[serde(rename = "maxResults")]
    pub max_results: Option<u32>,
    pub total: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Comment {
    pub id: Option<String>,
    pub body: Option<String>,
    pub author: Option<UserField>,
    pub created: Option<String>,
    pub updated: Option<String>,
}

// ── Transitions ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct TransitionsResponse {
    pub transitions: Vec<Transition>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transition {
    pub id: String,
    pub name: String,
    pub to: Option<StatusField>,
}

// ── Issue Links ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct IssueLink {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub link_type: Option<LinkType>,
    #[serde(rename = "inwardIssue")]
    pub inward_issue: Option<LinkedIssueRef>,
    #[serde(rename = "outwardIssue")]
    pub outward_issue: Option<LinkedIssueRef>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LinkedIssueRef {
    pub id: Option<String>,
    pub key: Option<String>,
    pub fields: Option<LinkedIssueFields>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LinkedIssueFields {
    pub summary: Option<String>,
    pub status: Option<StatusField>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LinkTypesResponse {
    #[serde(rename = "issueLinkTypes")]
    pub issue_link_types: Vec<LinkType>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LinkType {
    pub id: Option<String>,
    pub name: Option<String>,
    pub inward: Option<String>,
    pub outward: Option<String>,
}

// ── Search ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchResults {
    #[serde(rename = "startAt")]
    pub start_at: u32,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    pub total: u32,
    pub issues: Vec<Issue>,
}

// ── Myself ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Myself {
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
}
