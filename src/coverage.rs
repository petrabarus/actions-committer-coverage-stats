///! This module contains the coverage analysis for the project.

/// Represents the coverage statistics for a single committer.
pub struct CommitterCoverageUserStat {
    username: String,
    email: String,
    lines: u32,
    covered: u32,
}

impl CommitterCoverageUserStat {
    pub fn new(username: &str, email: &str, lines: u32, covered: u32) -> CommitterCoverageUserStat {
        CommitterCoverageUserStat {
            username: username.to_string(),
            email: email.to_string(),
            lines,
            covered,
        }
    }
    
    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }

    pub fn get_lines(&self) -> u32 {
        self.lines
    }

    pub fn get_covered(&self) -> u32 {
        self.covered
    }

    pub fn get_percent_covered(&self) -> f32 {
        match self.lines {
            0 => 0.0,
            _ => self.covered as f32 / self.lines as f32 * 100.0,
        }
    }
}

/// Represents the summary of the coverage for all committers.
/// This will be printed to the pull request as a comment.
pub struct CommitterCoverageSummary {
    lines: u32,
    covered: u32,
    percent_covered: f32,
    user_stats: Vec<CommitterCoverageUserStat>,
}

impl CommitterCoverageSummary {
    pub fn new() -> CommitterCoverageSummary {
        CommitterCoverageSummary {
            lines: 0,
            covered: 0,
            percent_covered: 0.0,
            user_stats: Vec::new(),
        }
    }

    pub fn add_user_stat(&mut self, user_stat: CommitterCoverageUserStat) {
        self.user_stats.push(user_stat);
    }

    pub fn calculate_summary(&mut self) {
        self.lines = self.user_stats.iter().map(|s| s.lines).sum();
        self.covered = self.user_stats.iter().map(|s| s.covered).sum();
        self.percent_covered = self.covered as f32 / self.lines as f32 * 100.0;
    }

    pub fn get_user_stats(&self) -> &Vec<CommitterCoverageUserStat> {
        &self.user_stats
    }
}

pub fn calculate_coverage_summary() -> CommitterCoverageSummary {
    let mut summary = CommitterCoverageSummary::new();

    // TODO: Remove this dummy data
    let user_stat = CommitterCoverageUserStat::new(
        "testing",
        "testing@example.com",
        100,
        50,
    );
    summary.add_user_stat(user_stat);

    summary
}

pub fn load_coverage_files() {
    println!("TODO: load coverage files");
}

#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn test_calculate_coverage_user_stat_percent_covered() {
        let user_stat = CommitterCoverageUserStat::new(
            "user",
            "user@example.com",
            100,
            50,
        );
        assert_eq!(user_stat.get_percent_covered(), 50.0);

        let user_stat = CommitterCoverageUserStat::new(
            "user2",
            "user2@example.com",
            0,
            0,
        );
        assert_eq!(user_stat.get_percent_covered(), 0.0);
    }
}
