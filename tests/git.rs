#[cfg(test)]
mod tests {
    use github_action_committer_coverage_stats::git::*;

    fn load_git() -> Git {
        let path = "";
        Git::new_from_path(path).unwrap()
    }

    #[ignore = "This test requires a valid git repository"]
    #[test]
    fn test_git_get_commit_hash() {
        let git = load_git();
        let commit_hash = git.get_last_commit_hash();
        assert!(commit_hash.is_ok());
        let commit_hash = commit_hash.unwrap();
        assert!(!commit_hash.is_empty());

        let blame_file = git.get_file_blame("src/lib.rs");
        assert!(blame_file.is_ok());
        let blame_file = blame_file.unwrap();

        for (line_num, line_blame) in blame_file.get_lines() {
            println!("{} {}", line_num, line_blame);
        }
    }
}
