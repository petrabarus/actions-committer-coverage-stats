//! This file contains the GitHub API client and its implementation.
//use curl::easy::{Easy, List};

use json::object;
use super::coverage;

/// This struct represents the GitHub API client.
pub struct GitHubClient {
    api_url: String,
    repo: String,
    token: String,
}

const USER_AGENT: &str = "petrabarus/committer-coverage-summary";

impl GitHubClient {
    pub fn new(api_url: &str, repo: &str, token: &str) -> GitHubClient {
        GitHubClient {
            api_url: api_url.to_string(),
            repo: repo.to_string(),
            token: token.to_string(),
        }
    }

    fn create_pr_comment_url(&self, pull_request_number: u32) -> String {
        format!(
            "{}/repos/{}/issues/{}/comments",
            self.api_url, self.repo, pull_request_number
        )
    }

    pub fn post_comment(&self, pull_request_number: u32, body: &str) -> Result<(), String> {
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
                status => {
                    return Err(format!(
                        "Failed to send request: {}",
                        status.canonical_reason().unwrap_or("Unknown")
                    ))
                }
            },
            Err(err) => return Err(format!("Failed to send request: {}", err)),
        }
    }

    pub fn print_summary_to_pr(
        &self,
        pull_request_number: u32,
        summary: &coverage::CommitterCoverageSummary,
        min_threshold: f32,
    ) -> Result<(), String> {
        let body = GitHubClient::create_summary_content(summary, min_threshold);
        self.post_comment(pull_request_number, &body)
    }

    fn create_summary_content(summary: &coverage::CommitterCoverageSummary, min_threshold: f32) -> String{
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

        body.push_str("\n");
        body.push_str("⭐ [github-action-committer-coverage-stats](https://github.com/petrabarus/github-action-committer-coverage-stats)");

        body
    }
}

pub fn load_committers() {
    println!("TODO: load committers");
}


pub fn parse_pr_number_from_ref(github_ref: &str) -> Option<u32> {
    let parts: Vec<&str> = github_ref.split('/').collect();
    if parts.len() == 4 && parts[0] == "refs" && parts[1] == "pull" {
        parts[2].parse().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_parse_pull_request_number_from_ref() {
        assert_eq!(
            super::parse_pr_number_from_ref("refs/pull/123/merge"),
            Some(123)
        );
        assert_eq!(
            super::parse_pr_number_from_ref("refs/heads/main"), 
            None
        );
    }
}
