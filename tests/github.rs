#[cfg(test)]
mod tests {
    use github_action_committer_coverage_stats::analysis::*;
    use github_action_committer_coverage_stats::git::BlameProvider;
    use github_action_committer_coverage_stats::github::*;

    fn create_client() -> GitHubClient {
        GitHubClient::new(
            "https://api.github.com",
            "petrabarus/github-action-committer-coverage-stats",
            "",
        )
    }

    #[ignore = "This test requires a valid token"]
    #[test]
    fn test_githubclient_print_summary_to_pr() {
        let client = create_client();

        let mut summary = CommitterCoverageSummary::default();
        summary.create_user_stat_if_not_exists("user@example.com", None);
        summary.set_user_stat("user@example.com", 100, 50).expect("User does not exist");

        let min_threshold = 80.0;
        let res = client.print_summary_to_pr(1, &summary, min_threshold);
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[ignore = "This test requires a valid token"]
    #[test]
    fn test_githubclient_get_user_by_email_found() {
        let client = create_client();

        let user = client.get_user_by_email("test@example.com");

        assert!(user.is_ok());
        let user = user.unwrap();
        assert!(user.is_some());
        let user = user.unwrap();
        assert_eq!(user.username, "example");
    }

    #[ignore = "This test requires a valid token"]
    #[test]
    fn test_githubclient_get_user_by_email_not_found() {
        let client = create_client();

        let user = client.get_user_by_email("testxxxxxxxxxxx@example.com");

        assert!(user.is_ok());
        let user = user.unwrap();
        //println!("{}", user.unwrap().username);
        assert!(user.is_none());
    }

    #[ignore = "This test requires a valid token"]
    #[test]
    fn test_githubclient_get_file_blame() {
        let client = create_client();

        let file = "./Dockerfile";
        let blame = client.get_file_blame(file);
        assert!(blame.is_ok());
        let blame = blame.unwrap();
        assert_eq!(57, blame.get_lines().len());
    }
}
