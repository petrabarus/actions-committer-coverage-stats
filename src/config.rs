//! This module contains the Config struct and its implementation.
use std::{collections::HashMap, env};

pub struct Config {
    values: HashMap<String, String>,
}

impl Config {
    /// Create a new Config instance from the environment variables.
    pub fn new_from_env() -> Result<Config, String> {
        let var_configs = Config::get_var_configs();

        let mut values = HashMap::new();
        for (key, (env_var, default)) in var_configs.iter() {
            let value = match env::var(env_var) {
                Ok(value) => value,
                Err(_) => match default {
                    Some(default) => default.to_string(),
                    None => return Err(format!("Missing {}", env_var)),
                },
            };
            values.insert(key.to_string(), value);
        }

        Ok(Config { values })
    }

    fn get_var_configs() -> HashMap<&'static str, (&'static str, Option<&'static str>)> {
        HashMap::from([
            (
                "github_api_url",
                ("GITHUB_API_URL", Some("https://api.github.com")),
            ),
            ("github_token", ("INPUT_GITHUB_TOKEN", None)),
            ("github_ref", ("GITHUB_REF", None)),
            ("github_repo", ("GITHUB_REPOSITORY", None)),
        ])
    }

    pub fn get_github_token(&self) -> &str {
        &self.values["github_token"]
    }

    pub fn get_github_ref(&self) -> &str {
        &self.values["github_ref"]
    }

    pub fn get_github_api_url(&self) -> &str {
        &self.values["github_api_url"]
    }

    pub fn get_github_repo(&self) -> &str {
        &self.values["github_repo"]
    }
}

#[cfg(test)]
mod tests {}
