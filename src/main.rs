// This is the main entry point of the program.
use committer_coverage_stats::{config, coverage, github};

fn print_summary_to_pr_if_available(
    gh: &github::GitHubClient, 
    github_ref: &str, 
    summary: &coverage::CommitterCoverageSummary,
    min_threshold: f32,
) {
    let pull_request_number = match github::parse_pr_number_from_ref(github_ref) {
        Some(pr) => pr,
        None => {
            println!("Not a pull request, skipping summary");
            return;
        }
    };

    let pr = gh.print_summary_to_pr(
        pull_request_number, 
        summary,
        min_threshold,
    );
    if let Err(err) = pr {
        panic!("Failed to print summary to pull request: {}", err);
    }
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

    coverage::load_coverage_files();
    github::load_committers();
    let summary = coverage::calculate_coverage_summary();

    print_summary_to_pr_if_available(
        &gh, 
        config.get_github_ref(), 
        &summary,
        config.get_min_threshold(),
    );

    println!("Success!");
}
