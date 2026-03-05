use reqwest::blocking::{Client, Response};
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

use crate::AppError;

pub struct JiraClient {
    base_url: String,
    http: Client,
}

impl JiraClient {
    pub fn new(base_url: &str, token: &str, timeout_ms: u64) -> Result<Self, AppError> {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", token);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&auth_value)
                .map_err(|e| AppError::runtime("AUTH_HEADER_INVALID", e.to_string()))?,
        );
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let http = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_millis(timeout_ms))
            .build()
            .map_err(|e| AppError::runtime("HTTP_CLIENT_BUILD_FAILED", e.to_string()))?;

        let base_url = base_url.trim_end_matches('/').to_string();
        Ok(Self { base_url, http })
    }

    fn api_url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        format!("{}/rest/api/2/{}", self.base_url, path)
    }

    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, AppError> {
        let url = self.api_url(path);
        let resp = self
            .http
            .get(&url)
            .send()
            .map_err(|e| AppError::runtime("HTTP_GET_FAILED", e.to_string()))?;
        self.parse_response(resp)
    }

    pub fn get_with_params<T: DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<T, AppError> {
        let url = self.api_url(path);
        let resp = self
            .http
            .get(&url)
            .query(params)
            .send()
            .map_err(|e| AppError::runtime("HTTP_GET_FAILED", e.to_string()))?;
        self.parse_response(resp)
    }

    pub fn post<T: DeserializeOwned>(&self, path: &str, body: &Value) -> Result<T, AppError> {
        let url = self.api_url(path);
        let resp = self
            .http
            .post(&url)
            .json(body)
            .send()
            .map_err(|e| AppError::runtime("HTTP_POST_FAILED", e.to_string()))?;
        self.parse_response(resp)
    }

    pub fn post_no_body(&self, path: &str, body: &Value) -> Result<(), AppError> {
        let url = self.api_url(path);
        let resp = self
            .http
            .post(&url)
            .json(body)
            .send()
            .map_err(|e| AppError::runtime("HTTP_POST_FAILED", e.to_string()))?;
        self.check_status(resp)
    }

    pub fn put_no_body(&self, path: &str, body: &Value) -> Result<(), AppError> {
        let url = self.api_url(path);
        let resp = self
            .http
            .put(&url)
            .json(body)
            .send()
            .map_err(|e| AppError::runtime("HTTP_PUT_FAILED", e.to_string()))?;
        self.check_status(resp)
    }

    fn parse_response<T: DeserializeOwned>(&self, resp: Response) -> Result<T, AppError> {
        let status = resp.status();
        let body = resp
            .text()
            .map_err(|e| AppError::runtime("HTTP_READ_BODY_FAILED", e.to_string()))?;

        if !status.is_success() {
            return Err(self.api_error(status.as_u16(), &body));
        }

        serde_json::from_str(&body).map_err(|e| {
            AppError::runtime(
                "JSON_PARSE_FAILED",
                format!("Failed to parse response: {e}\nBody: {body}"),
            )
        })
    }

    fn check_status(&self, resp: Response) -> Result<(), AppError> {
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().unwrap_or_default();
            return Err(self.api_error(status.as_u16(), &body));
        }
        Ok(())
    }

    fn api_error(&self, status: u16, body: &str) -> AppError {
        // Try to extract Jira error messages
        let message = if let Ok(v) = serde_json::from_str::<Value>(body) {
            if let Some(msgs) = v.get("errorMessages").and_then(|m| m.as_array()) {
                let joined: Vec<String> = msgs
                    .iter()
                    .filter_map(|m| m.as_str().map(String::from))
                    .collect();
                if !joined.is_empty() {
                    format!("HTTP {status}: {}", joined.join("; "))
                } else {
                    format!("HTTP {status}: {body}")
                }
            } else {
                format!("HTTP {status}: {body}")
            }
        } else {
            format!("HTTP {status}: {body}")
        };
        AppError::runtime("JIRA_API_ERROR", message)
    }
}
