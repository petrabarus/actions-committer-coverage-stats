#[cfg(test)]
mod tests {
    use github_action_committer_coverage_stats::github::*;
    use github_action_committer_coverage_stats::analysis::*;

    #[ignore = "This test requires a valid token"]
    #[test]
    fn test_githubclient_print_summary_to_pr() {
        let client = GitHubClient::new(
            "https://api.github.com",
            "petrabarus/committer-coverage-summary",
            "",
        );

        let mut summary = CommitterCoverageSummary::new();
        summary.add_user_stat(CommitterCoverageUserStat::new(
            "petrabarus",
            "user@example.com",
            100,
            50,
        ));

        let min_threshold = 80.0;
        let res = client.print_summary_to_pr(1, &summary, min_threshold);
        println!("{:?}", res);
        assert!(res.is_ok());
    }
}
