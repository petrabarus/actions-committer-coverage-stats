// This is the main entry point of the program.
use github_action_committer_coverage_stats::{
    analysis, config, coverage::Coverage, git, github,
};

fn print_summary_to_pr_if_available(
    gh: &github::GitHubClient,
    github_ref: &str,
    summary: &analysis::CommitterCoverageSummary,
    min_threshold: f32,
) {
    let pull_request_number = match github::parse_pr_number_from_ref(github_ref)
    {
        Some(pr) => pr,
        None => {
            println!("Not a pull request, skipping summary");
            return;
        }
    };

    let pr =
        gh.print_summary_to_pr(pull_request_number, summary, min_threshold);
    if let Err(err) = pr {
        panic!("Failed to print summary to pull request: {}", err);
    }
}

fn load_coverage_file(files: &[String]) -> Result<Coverage, String> {
    // just one file for now.
    // if empty, return an error
    if files.is_empty() {
        return Err("No coverage files specified".to_string());
    }
    Coverage::new_from_path(files[0].as_str())
}

fn main() {
    // panic if the config cannot be loaded
    let config = match config::Config::new_from_env() {
        Ok(config) => config,
        Err(err) => panic!("Problem loading config: {}", err),
    };

    let gh = github::GitHubClient::new(
        config.get_github_api_url(),
        config.get_github_repo(),
        config.get_github_token(),
    );

    let coverage = load_coverage_file(config.get_files())
        .expect("Failed to load coverage file");

    let git = git::Git::new_from_path(config.get_workspace())
        .expect("Failed to load git repository");

    let summary =
        analysis::calculate_committers_coverage_summary(&git, &coverage);

    print_summary_to_pr_if_available(
        &gh,
        config.get_github_ref(),
        &summary,
        config.get_min_threshold(),
    );

    println!("Success!");
}
