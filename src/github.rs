//! This file contains the GitHub API client and its implementation.
//use curl::easy::{Easy, List};

use email_address::EmailAddress;
use reqwest::{blocking::Client, StatusCode};
use std::collections::HashMap;

use crate::{
    analysis::CommitterCoverageUserStat,
    git::{BlameFile, BlameLine, BlameProvider},
};

use super::analysis;
use json::object;

enum GitHubUserCacheRecord {
    Some(GithubUser),
    None,
}
/// This struct represents the GitHub API client.
pub struct GitHubClient {
    // default token
    token: String,
    api_url: String,
    repo: String,
    user_cache: HashMap<String, GitHubUserCacheRecord>,
}

const USER_AGENT: &str = "petrabarus/committer-coverage-summary";

impl GitHubClient {
    pub fn new(
        api_url: &str,
        repo: &str,
        token: &str,
        //
    ) -> GitHubClient {
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
        let body = self.create_summary_content(summary, min_threshold);
        self.request_post_issue_comment(pull_request_number, &body)
    }

    fn create_sync_client(&self) -> Client {
        Client::new()
    }

    fn create_sync_post_client(
        &self,
        url: &str,
    ) -> reqwest::blocking::RequestBuilder {
        let req = self.create_sync_client().post(url);
        let req = self.add_basic_headers_to_request(req);
        self.add_bearer_token_to_request(req, &self.token)
    }

    fn create_sync_get_client(
        &self,
        url: &str,
    ) -> reqwest::blocking::RequestBuilder {
        let req = self.create_sync_client().get(url);
        let req = self.add_basic_headers_to_request(req);
        self.add_bearer_token_to_request(req, &self.token)
    }

    fn add_basic_headers_to_request(
        &self,
        request: reqwest::blocking::RequestBuilder,
    ) -> reqwest::blocking::RequestBuilder {
        request
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/json")
    }

    fn add_bearer_token_to_request(
        &self,
        request: reqwest::blocking::RequestBuilder,
        token: &str,
    ) -> reqwest::blocking::RequestBuilder {
        request.bearer_auth(token)
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

        let req = self
            .create_sync_post_client(&url)
            .body(data);
        let result = req.send();

        match result {
            Ok(result) => match result.status() {
                StatusCode::CREATED => Ok(()),
                status => Err(format!(
                    "Failed to send request: {}",
                    status.canonical_reason().unwrap_or("Unknown Status")
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
        &self,
        email: &str,
    ) -> Result<Option<GithubUser>, String> {
        let user = self.request_search_user_by_email(email).map_err(|err| {
            format!("Failed to search user by email: {}", err)
        })?;

        //self.cache_user(email, &user);

        Ok(user)
    }

    fn request_search_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<GithubUser>, String> {
        let url = format!("{}/search/users?q={}", self.api_url, email);

        let req = self.create_sync_get_client(&url);
        let result = req.send();

        match result {
            Ok(result) => match result.status() {
                StatusCode::OK => {
                    let response = result.text().map_err(|err| {
                        format!("Failed to read response: {}", err)
                    })?;
                    GitHubClient::parse_user_from_search_response(&response)
                }
                status => Err(format!(
                    "Failed to send request: {}",
                    status.canonical_reason().unwrap_or("Unknown Status")
                )),
            },
            Err(err) => Err(format!("Failed to send request: {}", err)),
        }
    }

    pub fn store_cache_user(&mut self, email: &str, user: &Option<GithubUser>) {
        let record = match user {
            Some(user) => GitHubUserCacheRecord::Some(user.clone()),
            None => GitHubUserCacheRecord::None,
        };

        self.user_cache.insert(email.to_string(), record);
    }

    pub fn get_cached_user(&self, email: &str) -> Option<GithubUser> {
        match self.user_cache.get(email) {
            Some(record) => match record {
                GitHubUserCacheRecord::Some(user) => Some(user.clone()),
                GitHubUserCacheRecord::None => None,
            },
            None => None,
        }
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
            let url = item["html_url"].to_string();

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

/// Implementation for summary content, since it's so long.
impl GitHubClient {
    pub fn create_summary_content(
        &self,
        summary: &analysis::CommitterCoverageSummary,
        min_threshold: f32,
    ) -> String {
        let mut content = String::new();
        let header = self.create_summary_content_header(summary, min_threshold);
        content.push_str(header.as_str());

        let table = self.create_summary_content_table(summary, min_threshold);
        content.push_str(table.as_str());

        let footer = "\n⭐ [github-action-committer-coverage-stats](https://github.com/petrabarus/github-action-committer-coverage-stats)";
        content.push_str(footer);

        content
    }

    fn create_summary_content_header(
        &self,
        summary: &analysis::CommitterCoverageSummary,
        _min_threshold: f32,
    ) -> String {
        let mut header = String::new();
        header.push_str("# Committer Coverage Report\n");
        header.push_str(&format!(
            "Total coverage: {} / {} ({:.2}%)\n\n",
            summary.get_covered(),
            summary.get_lines(),
            summary.get_percent_covered()
        ));
        header
    }

    fn create_summary_content_table(
        &self,
        summary: &analysis::CommitterCoverageSummary,
        min_threshold: f32,
    ) -> String {
        let mut table = String::new();
        let table_header =
            "|  | **User** | **Lines** | **Covered** | **% Covered** |
|--|------|-------:|---------:|-----------|
";

        table.push_str(table_header);

        let mut sorted_user_stats: Vec<CommitterCoverageUserStat> =
            summary.get_user_stats().values().cloned().collect();

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

            let user =
                self.create_summary_content_table_row_user_display(&user_stat);

            table.push_str(&format!(
                "| {} | {} | {} | {:.2} {} |\n",
                user,
                user_stat.get_lines(),
                user_stat.get_covered(),
                user_stat.get_percent_covered(),
                status
            ));
        }

        table
    }

    fn create_summary_content_table_row_user_display(
        &self,
        user_stat: &CommitterCoverageUserStat,
    ) -> String {
        let email = user_stat.get_email();
        let name = user_stat.get_name();

        if !EmailAddress::is_valid(email) {
            eprintln!("Invalid email: {}", email);
            return self.create_unknown_user_display(name);
        }

        match self.get_user_by_email(email) {
            Ok(user) => match user {
                Some(user) => self.create_user_display(
                    user.username.as_str(),
                    user.url.as_str(),
                    user.avatar_url.as_str(),
                ),
                None => {
                    eprintln!("Received None user when creating summary table");
                    self.create_unknown_user_display(name)
                }
            },
            Err(err) => {
                eprintln!("Failed to get user by email, got error when creating summary table: {}", err);
                self.create_unknown_user_display(name)
            }
        }
    }

    fn create_unknown_user_display(&self, name: &Option<String>) -> String {
        self.create_user_display(
            self.must_get_name(name).as_str(),
            "https://github.com", 
            "https://avatars.githubusercontent.com/u/1234567890?v=4"
        )
    }

    fn create_user_display(&self, name: &str, url: &str, avatar_url: &str) -> String {
        format!(
            "<a href=\"{}\"><img src=\"{}\" width=\"20\"/></a> | {}",
            url, avatar_url, name
        )
    }

    fn must_get_name(&self, name: &Option<String>) -> String {
        if let Some(name) = name {
            name.to_string()
        } else {
            eprintln!("Email and name are both invalid");
            "unknown".to_string()
        }
    }
}

#[derive(Clone)]
pub struct GithubUser {
    pub username: String,
    pub avatar_url: String,
    pub url: String,
}

/// Parse the pull request number from the GitHub ref.
/// ```
/// let pr_number = github::parse_pr_number_from_ref("123/merge");
/// assert_eq!(pr_number, Some(123));
/// ```
pub fn parse_pr_number_from_ref(github_ref: &str) -> Option<u32> {
    let parts = github_ref.split_once('/');
    match parts {
        Some((pr, _)) => pr.parse::<u32>().ok(),
        None => None,
    }
}

impl BlameProvider for GitHubClient {
    fn get_file_blame(
        &self,
        path: &str
    ) -> Result<BlameFile, String> {
        //eprintln!("Requesting blame for file: {}", path);
        let json_result =
            self.request_graphql_blame(path).map_err(|err| {
                format!("Failed to request blame for file {}: {}", path, err)
            })?;

        let mut blame_file = BlameFile::new_from_path(path);
        let vec = GitHubClient::parse_blame_lines_from_graphql_blame_result(
            json_result.as_str(),
        )
        .map_err(|err| {
            format!(
                "Failed to parse blame lines from GraphQL response: {}",
                err
            )
        })?;

        blame_file.set_lines_from_vec(vec);

        Ok(blame_file)
    }
}

/// Implementation for GitHubClient for the BlameProvider trait.
impl GitHubClient {
    // fn request_graphql_default_branch(&self) -> String {
    //     "main".to_string()
    // }

    /// Request blame information from GitHub API using GraphQL.
    /// This will return the response as a string.
    /// If there is an error, it will return an error message.
    fn request_graphql_blame(
        &self,
        path: &str,
    ) -> Result<String, String> {
        let (repo_owner, repo_name) = self.repo.split_once('/').unwrap();
        let graphql_query = format!(
            "
query {{
  repository(
      owner:\"{}\", 
      name:\"{}\"
  ) {{
      object(expression: \"{}\") {{
        ... on Commit {{
          blame(path: \"{}\") {{
              ranges {{
              startingLine,
              endingLine,
              commit {{
                oid,
                author {{
                  name,
                  email
                }}
              }}
            }}
          }}
        }}
      }}
    }}
}}
",
            repo_owner, repo_name, "main", path
        );
        let mut data = json::JsonValue::new_object();
        data["query"] = graphql_query.into();
        let data = data.dump();
        //eprintln!("data: {}", data);

        let graphql_url = format!("{}/graphql", self.api_url);

        let req = self
            .create_sync_post_client(&graphql_url)
            .body(data);
        let result = req.send();

        match result {
            Ok(result) => match result.status() {
                StatusCode::OK => {
                    let response = result.text().map_err(|err| {
                        format!("Failed to read response: {}", err)
                    })?;
                    Ok(response)
                }
                status => Err(format!(
                    "Failed to send request: {}",
                    status.canonical_reason().unwrap_or("Unknown Status")
                )),
            },
            Err(err) => Err(format!("Failed to send request: {}", err)),
        }
    }

    fn parse_blame_lines_from_graphql_blame_result(
        response: &str,
    ) -> Result<Vec<BlameLine>, String> {
        let json = json::parse(response);
        if let Err(err) = json {
            return Err(format!("Failed to parse JSON: {}", err));
        }
        let json = json.unwrap();

        let error = &json["data"]["errors"];
        if !error.is_null() {
            return Err(format!(
                "Failed to get blame lines from GraphQL response: {}",
                error.dump()
            ));
        }

        // get value of data.repository.object.blame.ranges and then iterate
        let blame_ranges =
            &json["data"]["repository"]["object"]["blame"]["ranges"];
        if !blame_ranges.is_array() {
            return Err(format!("Invalid JSON response, got {}", json.dump()));
        }
        let blame_ranges = blame_ranges.members();

        let mut vec = Vec::new();

        for range in blame_ranges {
            let starting_line = range["startingLine"].as_u32().unwrap();
            let ending_line = range["endingLine"].as_u32().unwrap();
            let commit = range["commit"]["oid"].as_str().unwrap();
            let author_name =
                range["commit"]["author"]["name"].as_str().unwrap();
            let email = range["commit"]["author"]["email"].as_str().unwrap();

            // iterate from starting_line to ending_line
            for line_num in starting_line..=ending_line {
                let line = BlameLine::new(
                    line_num,
                    commit,
                    Some(email.to_string()),
                    Some(author_name.to_string()),
                );
                //eprintln!("line: {}", line);
                vec.push(line);
            }
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_pull_request_number_from_ref() {
        assert_eq!(parse_pr_number_from_ref("715/merge"), Some(715));
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
        assert_eq!(user.url, "https://github.com/testuser");
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

    #[test]
    fn test_githubclient_parse_blame_lines_from_api_graphql_blame_response_should_return_correct_value(
    ) {
        let response = r#"
        {
            "data": {
              "repository": {
                "object": {
                  "blame": {
                    "ranges": [
                      {
                        "startingLine": 1,
                        "endingLine": 5,
                        "commit": {
                          "oid": "8d5445550b1948b914853fc7f210ff3622ee0c18",
                          "author": {
                            "name": "User 1",
                            "email": "user1@example.com"
                          }
                        }
                      },
                      {
                        "startingLine": 6,
                        "endingLine": 6,
                        "commit": {
                          "oid": "5d2595a1368702ac796582016b764dedceabde85",
                          "author": {
                            "name": "User 2",
                            "email": "user2@example.com"
                          }
                        }
                      },
                      {
                        "startingLine": 7,
                        "endingLine": 57,
                        "commit": {
                          "oid": "8d5445550b1948b914853fc7f210ff3622ee0c18",
                          "author": {
                            "name": "User 3",
                            "email": "user3@example.com"
                          }
                        }
                      }
                    ]
                  }
                }
              }
            }
          }
        "#;

        let result =
            GitHubClient::parse_blame_lines_from_graphql_blame_result(response);
        assert!(result.is_ok());
        let vec = result.unwrap();

        assert_eq!(57, vec.len());
        let line_57 = &vec[56];
        assert_eq!(
            "8d5445550b1948b914853fc7f210ff3622ee0c18",
            line_57.get_commit()
        );
        assert_eq!("user3@example.com", line_57.get_email().clone().unwrap());
    }

    #[test]
    pub fn test_githubclient_parse_blame_lines_from_api_graphql_blame_response_should_return_error_when_invalid_json(
    ) {
        let response = r#"
        {
            "data": {

            }
        }
"#;

        let result =
            GitHubClient::parse_blame_lines_from_graphql_blame_result(response);
        assert!(result.is_err());
    }
}
