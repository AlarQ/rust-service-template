use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

pub struct GitHubClient {
    client: reqwest::Client,
    token: String,
    api_base: String,
}

#[derive(Serialize, Debug)]
struct CreateRepoRequest {
    name: String,
    description: Option<String>,
    private: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    auto_init: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct CreateRepoResponse {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub private: bool,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct GitHubError {
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    errors: Option<Vec<serde_json::Value>>,
}

impl GitHubClient {
    pub fn new(token: impl Into<String>) -> Result<Self> {
        let token = token.into();
        if token.is_empty() {
            anyhow::bail!("GitHub token cannot be empty");
        }

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rust-service-cli/1.0"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            token,
            api_base: "https://api.github.com".to_string(),
        })
    }

    pub async fn create_repository(
        &self,
        name: &str,
        description: Option<&str>,
        private: bool,
        owner: &str,
    ) -> Result<CreateRepoResponse> {
        let url = if owner.contains('/') {
            let parts: Vec<&str> = owner.split('/').collect();
            let org = parts[0];
            format!("{}/orgs/{}/repos", self.api_base, org)
        } else {
            format!("{}/user/repos", self.api_base)
        };

        let request_body = CreateRepoRequest {
            name: name.to_string(),
            description: description.map(String::from),
            private,
            auto_init: Some(false),
        };

        let response = self
            .client
            .post(&url)
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token))
                    .context("Invalid GitHub token format")?,
            )
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        let status = response.status();

        if status.is_success() {
            let repo: CreateRepoResponse = response
                .json()
                .await
                .context("Failed to parse GitHub API response")?;
            Ok(repo)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error: GitHubError = serde_json::from_str(&error_text).unwrap_or(GitHubError {
                message: error_text,
                errors: None,
            });

            anyhow::bail!("GitHub API error ({}): {}", status.as_u16(), error.message);
        }
    }

    pub async fn get_authenticated_user(&self) -> Result<serde_json::Value> {
        let url = format!("{}/user", self.api_base);

        let response = self
            .client
            .get(&url)
            .header(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", self.token))
                    .context("Invalid GitHub token format")?,
            )
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .context("Failed to send request to GitHub API")?;

        if response.status().is_success() {
            let user: serde_json::Value = response
                .json()
                .await
                .context("Failed to parse GitHub API response")?;
            Ok(user)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("GitHub API error ({}): {}", status.as_u16(), error_text);
        }
    }
}

pub fn get_github_token() -> Result<String> {
    std::env::var("GITHUB_TOKEN").context(
        "GITHUB_TOKEN environment variable not set. Please set it to your GitHub personal access token."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_client_creation() {
        let client = GitHubClient::new("test_token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_github_client_empty_token() {
        let client = GitHubClient::new("");
        assert!(client.is_err());
    }
}
