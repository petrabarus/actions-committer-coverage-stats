// This is the main entry point of the program.
use github_action_committer_coverage_stats::{
    analysis::CommitterCoverageSummary, config::Config, coverage::Coverage,
    git::Git, github, github::GitHubClient,
};

fn print_summary_to_pr(
    gh: &GitHubClient,
    github_ref: &str,
    summary: &CommitterCoverageSummary,
    min_threshold: f32,
) -> Result<(), String> {
    let pull_request_number = match github::parse_pr_number_from_ref(github_ref)
    {
        Some(pr) => pr,
        None => {
            return Err(format!(
                "Failed to parse pull request number from ref: {}",
                github_ref
            ));
        }
    };

    gh.print_summary_to_pr(pull_request_number, summary, min_threshold)
}

fn load_coverage_file(files: &[String]) -> Result<Coverage, String> {
    // just one file for now.
    // if empty, return an error
    if files.is_empty() {
        return Err("No coverage files specified".to_string());
    }
    Coverage::new_from_path(files[0].as_str())
}

fn calculate_summary_from_git_or_github_api(
    coverage: &Coverage,
    use_github_api_for_blame: bool,
    git: &Git,
    gh: &GitHubClient,
) -> Result<CommitterCoverageSummary, String> {
    if use_github_api_for_blame {
        CommitterCoverageSummary::from_coverage_file_and_blame(coverage, gh)
    } else {
        CommitterCoverageSummary::from_coverage_file_and_blame(coverage, git)
    }
}

fn main() {
    // panic if the config cannot be loaded
    let config = match Config::new_from_env() {
        Ok(config) => config,
        Err(err) => panic!("Problem loading config: {}", err),
    };

    let gh = GitHubClient::new(
        config.get_github_api_url(),
        config.get_github_repo(),
        config.get_github_token(),
    );

    let coverage = load_coverage_file(config.get_files())
        .expect("Failed to load coverage file");

    let git = Git::new_from_path(config.get_workspace())
        .expect("Failed to load git repository");

    let summary = calculate_summary_from_git_or_github_api(
        &coverage,
        config.get_use_github_api_for_blame(),
        &git,
        &gh,
    )
    .expect("Failed to generate summary");

    if config.get_github_event_name() == "pull_request" {
        println!("Printing summary to Pull Request");
        print_summary_to_pr(
            &gh,
            config.get_github_ref_name(),
            &summary,
            config.get_min_threshold(),
        )
        .expect("Failed to print summary to PR");
    } else {
        eprintln!("Event {} is not a Pull Request", config.get_github_event_name());
    }

    println!("Success!");
}
