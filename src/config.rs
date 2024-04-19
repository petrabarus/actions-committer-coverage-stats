//! This module contains the Config struct and its implementation.
use std::env;

pub struct Config {
    /// This contains coverage files that will be parsed.
    coverage_files: Vec<String>,

    /// The minimum threshold for the coverage percentage.
    /// User with coverage percentage below this threshold will be considered as failing.
    min_threshold: f32,

    /// Whether to use the GitHub API to get the blame information.
    /// If false, the blame information will be read from the git repository.
    use_github_api_for_blame: bool,

    /// The workspace directory where the project is located.
    workspace: String,

    // see: https://docs.github.com/en/actions/learn-github-actions/variables
    // GITHUB_REF is in the format "refs/heads/branch-name"
    github_api_url: String,
    github_token: String,
    github_ref: String,
    github_ref_name: String,
    github_repo: String,
    github_event_name: String,
    github_head_ref: String,
}

impl Config {
    /// Create a new Config instance from the environment variables.
    pub fn new_from_env() -> Result<Config, String> {
        // Parse the action inputs
        let coverage_files =
            env::var("INPUT_FILES").unwrap_or("coverage.xml".to_string());
        let coverage_files = parse_files(&coverage_files);

        let github_token = env::var("INPUT_GITHUB_TOKEN")
            .map_err(|_| "github_token is not set")?;

        let workspace =
            env::var("INPUT_WORKSPACE").map_err(|_| "workspace is not set")?;
        let min_threshold = env::var("INPUT_MIN_THRESHOLD")
            .unwrap_or("80".to_string())
            .parse::<f32>()
            .map_err(|_| "min_threshold is not a valid number")?;
        //("INPUT_USE_GITHUB_API_FOR_BLAME: {}", env::var("INPUT_USE_GITHUB_API_FOR_BLAME").unwrap_or("false".to_string()));
        let use_github_api_for_blame = env::var("INPUT_USE_GITHUB_API_FOR_BLAME")
            .unwrap_or("false".to_string())
            .parse::<bool>()
            .map_err(|_| "use_github_api_for_blame is not a valid boolean")?;
        
        // Parse the GitHub environment variables.
        let github_ref =
            env::var("GITHUB_REF").map_err(|_| "GITHUB_REF is not set")?;
        let github_ref_name = env::var("GITHUB_REF_NAME")
            .map_err(|_| "GITHUB_REF_NAME is not set")?;
        let github_repo = env::var("GITHUB_REPOSITORY")
            .map_err(|_| "GITHUB_REPOSITORY is not set")?;
        let github_api_url = env::var("GITHUB_API_URL")
            .unwrap_or("https://api.github.com".to_string());
        let github_event_name = env::var("GITHUB_EVENT_NAME")
            .map_err(|_| "GITHUB_EVENT_NAME is not set")?;
        let github_head_ref = env::var("GITHUB_HEAD_REF")
            .unwrap_or("".to_string());

        Ok(Config {
            coverage_files,
            min_threshold,
            workspace,
            use_github_api_for_blame,
            github_api_url,
            github_token,
            github_ref,
            github_ref_name,
            github_repo,
            github_event_name,
            github_head_ref,
        })
    }

    pub fn get_files(&self) -> &Vec<String> {
        &self.coverage_files
    }

    pub fn get_min_threshold(&self) -> f32 {
        self.min_threshold
    }

    pub fn get_workspace(&self) -> &str {
        &self.workspace
    }

    pub fn get_use_github_api_for_blame(&self) -> bool {
        self.use_github_api_for_blame
    }

    pub fn get_github_token(&self) -> &str {
        &self.github_token
    }

    pub fn get_github_ref(&self) -> &str {
        &self.github_ref
    }

    pub fn get_github_ref_name(&self) -> &str {
        &self.github_ref_name
    }

    pub fn get_github_event_name(&self) -> &str {
        &self.github_event_name
    }

    pub fn get_github_head_ref(&self) -> &str {
        &self.github_head_ref
    }

    pub fn get_github_api_url(&self) -> &str {
        &self.github_api_url
    }

    pub fn get_github_repo(&self) -> &str {
        &self.github_repo
    }
}

fn parse_files(files: &str) -> Vec<String> {
    files.split(',').map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_files() {
        let files = "file1,file2,file3";
        let expected = vec![
            "file1".to_string(),
            "file2".to_string(),
            "file3".to_string(),
        ];
        assert_eq!(parse_files(files), expected);
    }
}
