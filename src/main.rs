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
    let coverages: Vec<Coverage> = files
        .iter()
        .map(|file| {
            let coverage = Coverage::new_from_path(file);
            // print reason for failure if any
            if let Err(err) = &coverage {
                println!("Failed to load coverage file {}: {}", file, err);
            }
            coverage
        })
        .filter_map(Result::ok)
        .collect();

    // just return the first coverage file for now
    if let Some(coverage) = coverages.first() {
        Ok(coverage.clone())
    } else {
        Err("No coverage files loaded".to_string())
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

    let coverage = load_coverage_file(config.get_files())
        .expect("Failed to load coverage file");

    let git = git::Git::new_from_path(config.get_workspace())
        .expect("Failed to load git repository");

    // DEBUG: print the last commit hash
    let last_commit = git.get_last_commit_hash()
        .expect("Failed to get last commit hash");
    println!("Last commit hash: {}", last_commit);

    // DEBUG: print the blame file for src/main.rs
    let blame_file = git.get_blame_file("src/main.rs")
        .expect("Failed to get blame file");
    println!("Blame file: {}", blame_file.get_path());
    for line in blame_file.get_lines() {
        println!("{}", line);
    }

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
