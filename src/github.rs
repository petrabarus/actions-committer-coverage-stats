//! This file contains the GitHub API client and its implementation.
//use curl::easy::{Easy, List};

use std::collections::HashMap;

use super::analysis;
use json::object;

enum GitHubUserCacheRecord {
    Some(GithubUser),
    None,
}
/// This struct represents the GitHub API client.
pub struct GitHubClient {
    api_url: String,
    repo: String,
    token: String,
    user_cache: HashMap<String, GitHubUserCacheRecord>,
}

const USER_AGENT: &str = "testuser/committer-coverage-summary";

impl GitHubClient {
    pub fn new(api_url: &str, repo: &str, token: &str) -> GitHubClient {
        let user_cache = HashMap::new();
        GitHubClient {
            api_url: api_url.to_string(),
            repo: repo.to_string(),
            token: token.to_string(),
            user_cache,
        }
    }

    pub fn print_summary_to_pr(
        &self,
        pull_request_number: u32,
        summary: &analysis::CommitterCoverageSummary,
        min_threshold: f32,
    ) -> Result<(), String> {
        let body = create_summary_content(summary, min_threshold);
        self.request_post_issue_comment(pull_request_number, &body)
    }

    fn request_post_issue_comment(
        &self,
        pull_request_number: u32,
        body: &str,
    ) -> Result<(), String> {
        let url = self.create_pr_comment_url(pull_request_number);

        let data = object! {
            "body" => body,
        };
        let data = data.dump();

        let client = reqwest::blocking::Client::new();
        let result = client
            .post(url)
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/json")
            .bearer_auth(&self.token)
            .body(data)
            .send();

        match result {
            Ok(result) => match result.status() {
                reqwest::StatusCode::CREATED => Ok(()),
                status => Err(format!(
                    "Failed to send request: {}",
                    status.canonical_reason().unwrap_or("Unknown")
                )),
            },
            Err(err) => Err(format!("Failed to send request: {}", err)),
        }
    }

    fn create_pr_comment_url(&self, pull_request_number: u32) -> String {
        format!(
            "{}/repos/{}/issues/{}/comments",
            self.api_url, self.repo, pull_request_number
        )
    }

    /// Get a user by email.
    /// This will check the cache first before making a request to the GitHub API.
    /// If the user is not found, it will return None.
    /// If there is error in the request, it will return an error message.
    pub fn get_user_by_email(
        &mut self,
        email: &str,
    ) -> Result<Option<GithubUser>, String> {
        if let Some(user) = self.user_cache.get(email) {
            return match user {
                GitHubUserCacheRecord::Some(user) => Ok(Some(user.clone())),
                // previously searched and not found so we avoid to call the API again
                GitHubUserCacheRecord::None => Ok(None),
            };
        }

        let user = self.request_search_user_by_email(email).map_err(|err| {
            format!("Failed to search user by email: {}", err)
        })?;

        self.cache_user(email, &user);

        Ok(user)
    }

    fn request_search_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<GithubUser>, String> {
        let url = format!("{}/search/users?q={}", self.api_url, email);
        let client = reqwest::blocking::Client::new();
        let result = client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/json")
            .bearer_auth(&self.token)
            .send();

        match result {
            Ok(result) => match result.status() {
                reqwest::StatusCode::OK => {
                    let response = result.text().map_err(|err| {
                        format!("Failed to read response: {}", err)
                    })?;
                    GitHubClient::parse_user_from_search_response(&response)
                }
                status => Err(format!(
                    "Failed to send request: {}",
                    status.canonical_reason().unwrap_or("Unknown")
                )),
            },
            Err(err) => Err(format!("Failed to send request: {}", err)),
        }
    }

    fn cache_user(&mut self, email: &str, user: &Option<GithubUser>) {
        let record = match user {
            Some(user) => GitHubUserCacheRecord::Some(user.clone()),
            None => GitHubUserCacheRecord::None,
        };

        self.user_cache.insert(email.to_string(), record);
    }

    fn parse_user_from_search_response(
        response: &str,
    ) -> Result<Option<GithubUser>, String> {
        let json = json::parse(response);
        if let Err(err) = json {
            return Err(format!("Failed to parse JSON: {}", err));
        }

        let json = json.unwrap();

        if json["total_count"].is_null() {
            return Err("Invalid JSON response".to_string());
        }
        if json["total_count"].as_u32().unwrap() == 0 {
            return Ok(None);
        }

        let items = json["items"].clone();
        if items.is_array() && !items.is_empty() {
            let item = &items[0];
            let username = item["login"].to_string();
            let avatar_url = item["avatar_url"].to_string();
            let url = item["url"].to_string();

            Ok(Some(GithubUser {
                username,
                avatar_url,
                url,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Clone)]
pub struct GithubUser {
    pub username: String,
    pub avatar_url: String,
    pub url: String,
}

pub fn parse_pr_number_from_ref(github_ref: &str) -> Option<u32> {
    let parts: Vec<&str> = github_ref.split('/').collect();
    if parts.len() == 4 && parts[0] == "refs" && parts[1] == "pull" {
        parts[2].parse().ok()
    } else {
        None
    }
}

/// Create the comment body for the pull request.
fn create_summary_content(
    summary: &analysis::CommitterCoverageSummary,
    min_threshold: f32,
) -> String {
    let mut body = String::new();
    body.push_str("# Committer Coverage Report\n");
    body.push_str(&format!(
        "Total coverage: {} / {} ({:.2}%)\n\n",
        summary.get_covered(),
        summary.get_lines(),
        summary.get_percent_covered()
    ));

    body.push_str("| **user** | **lines** | **covered** | **% covered** |\n");
    body.push_str("|------|-------:|---------:|-----------|\n");

    let mut sorted_user_stats = summary.get_user_stats().clone();
    sorted_user_stats.sort_by(|a, b| {
        let a = a.get_percent_covered();
        let b = b.get_percent_covered();
        b.partial_cmp(&a).unwrap()
    });

    for user_stat in sorted_user_stats {
        let percent_covered = user_stat.get_percent_covered();
        let status = if percent_covered >= min_threshold {
            "✅"
        } else {
            "❌"
        };

        body.push_str(&format!(
            "| {} | {} | {} | {:.2} {} |\n",
            user_stat.get_username(),
            user_stat.get_lines(),
            user_stat.get_covered(),
            user_stat.get_percent_covered(),
            status
        ));
    }

    body.push_str("\n⭐ [github-action-committer-coverage-stats](https://github.com/testuser/github-action-committer-coverage-stats)");

    body
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_pull_request_number_from_ref() {
        assert_eq!(parse_pr_number_from_ref("refs/pull/123/merge"), Some(123));
        assert_eq!(parse_pr_number_from_ref("refs/heads/main"), None);
    }

    #[test]
    fn test_githubclient_parse_user_from_search_response_success() {
        let response = r#"
        {
            "total_count": 1,
            "incomplete_results": false,
            "items": [
              {
                "login": "testuser",
                "id": 1234567890,
                "node_id": "MDQ6VXNlcjUyMzI4OQ==",
                "avatar_url": "https://avatars.githubusercontent.com/u/1234567890?v=4",
                "gravatar_id": "",
                "url": "https://api.github.com/users/testuser",
                "html_url": "https://github.com/testuser",
                "followers_url": "https://api.github.com/users/testuser/followers",
                "following_url": "https://api.github.com/users/testuser/following{/other_user}",
                "gists_url": "https://api.github.com/users/testuser/gists{/gist_id}",
                "starred_url": "https://api.github.com/users/testuser/starred{/owner}{/repo}",
                "subscriptions_url": "https://api.github.com/users/testuser/subscriptions",
                "organizations_url": "https://api.github.com/users/testuser/orgs",
                "repos_url": "https://api.github.com/users/testuser/repos",
                "events_url": "https://api.github.com/users/testuser/events{/privacy}",
                "received_events_url": "https://api.github.com/users/testuser/received_events",
                "type": "User",
                "site_admin": false,
                "score": 1.0
              }
            ]
          }
        "#;

        let user = GitHubClient::parse_user_from_search_response(response);
        assert!(user.is_ok());
        let user = user.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(
            user.avatar_url,
            "https://avatars.githubusercontent.com/u/1234567890?v=4"
        );
        assert_eq!(user.url, "https://api.github.com/users/testuser");
    }

    #[test]
    fn test_githubclient_parse_user_from_search_response_empty() {
        let response = r#"
        {
            "total_count": 0,
            "incomplete_results": false,
            "items": []
          }
        "#;

        let user = GitHubClient::parse_user_from_search_response(response);
        assert!(user.is_ok());
        let user = user.unwrap();
        assert!(user.is_none());
    }
}
