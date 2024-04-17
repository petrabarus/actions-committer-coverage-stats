//! This module contains the committer coverage analysis.
use super::{
    coverage::CoverageProvider,
    git::{BlameLine, BlameProvider},
};
use std::collections::{BTreeMap, HashMap};

/// Represents the summary of the coverage for all committers.
/// This will be printed to the pull request as a comment.
#[derive(Clone, Default)]
pub struct CommitterCoverageSummary {
    lines: u32,
    covered: u32,
    percent_covered: f32,
    user_stats: HashMap<String, CommitterCoverageUserStat>,
}

impl CommitterCoverageSummary {
    /// This will reset the user stats to 0 lines and 0 covered and returns,
    /// the previous value if exists, otherwise None.
    pub fn reset_user(
        &mut self,
        email: &str,
    ) -> Option<CommitterCoverageUserStat> {
        let new_val = CommitterCoverageUserStat::new(email, 0, 0);
        self.user_stats.insert(email.to_string(), new_val)
    }

    /// This function increments the line coverage for a user.
    pub fn incr_user_line_cover(&mut self, email: &str, covered: bool) {
        self.lines += 1;
        let covered = if covered { 1 } else { 0 };
        self.covered += covered;
        self.percent_covered = self.covered as f32 / self.lines as f32 * 100.0;

        if !self.user_stats.contains_key(email) {
            self.reset_user(email);
        }

        let stat = self.user_stats.get_mut(email).unwrap();
        stat.lines += 1;
        stat.covered += covered;
        stat.percent_covered = stat.covered as f32 / stat.lines as f32 * 100.0;
    }

    pub fn get_user_stats(
        &self,
    ) -> &HashMap<String, CommitterCoverageUserStat> {
        &self.user_stats
    }

    pub fn set_user_stat(&mut self, email: &str, lines: u32, covered: u32) {
        if !self.user_stats.contains_key(email) {
            self.reset_user(email);
        }

        let stat = self.user_stats.get_mut(email).unwrap();
        stat.lines = lines;
        stat.covered = covered;
    }

    pub fn get_lines(&self) -> u32 {
        self.lines
    }

    pub fn get_covered(&self) -> u32 {
        self.covered
    }

    pub fn get_percent_covered(&self) -> f32 {
        self.percent_covered
    }
}

impl CommitterCoverageSummary {
    pub fn from_coverage_file_and_blame<
        A: CoverageProvider,
        B: BlameProvider,
    >(
        coverage: &A,
        blame: &B,
    ) -> Result<CommitterCoverageSummary, String> {
        let file_iter = coverage
            .iter_files()
            .map_err(|e| format!("Failed to get coverage files: {}", e))?;

        let mut summary: CommitterCoverageSummary =
            CommitterCoverageSummary::default();

        // loop through all files in coverage
        for file in file_iter.into_iter() {
            let path = file.get_path();
            let blame_file = blame.get_file_blame(path);
            // .map_err(|e| format!("Failed to get blame file: {}", e))?;

            // Handle is blame file error.
            if let Err(e) = blame_file {
                // Skipping if the file is not in the git tree but
                // is in the coverage report. This may be a generated file
                // or just ignored by git.
                // TODO: Add input option to ignore files.
                if e.contains("not exist in the given tree") {
                    eprintln!("File not found in blame: {}. Skipping...", path);
                    continue;
                } else {
                    return Err(format!("Failed to get blame file: {}", e));
                }
            }

            let blame_file = blame_file.unwrap();
            CommitterCoverageSummary::calculate_by_lines(
                file.get_lines(),
                blame_file.get_lines(),
                &mut summary,
            )
        }

        Ok(summary)
    }

    fn calculate_by_lines(
        coverage_lines: &BTreeMap<u32, bool>,
        blame_lines: &BTreeMap<u32, BlameLine>,
        summary: &mut CommitterCoverageSummary,
    ) {
        for (line_num, covered) in coverage_lines.iter() {
            let blame_line = blame_lines.get(line_num);
            if blame_line.is_none() {
                continue;
            }
            let blame_line = blame_line.unwrap();
            let key = CommitterCoverageSummary::get_key_from_blame_line(blame_line);
            summary.incr_user_line_cover(key, *covered);
        }
    }

    fn get_key_from_blame_line(blame_line: &BlameLine) -> &str {
        if let Some(email) = blame_line.get_email() {
            return email;
        }
        if let Some(name) = blame_line.get_name() {
            return name;
        }
        "unknown"
    }
}

/// Represents the coverage statistics for a single committer.
#[derive(Clone, Default)]
pub struct CommitterCoverageUserStat {
    // The user id of the committer. If the email is available, it should be used.
    // Otherwise, the name should be used.
    user_id: String,
    lines: u32,
    covered: u32,
    percent_covered: f32,
}

impl CommitterCoverageUserStat {
    pub fn new(
        user_id: &str,
        lines: u32,
        covered: u32,
    ) -> CommitterCoverageUserStat {
        let percent_covered = match lines {
            0 => 0.0,
            _ => covered as f32 / lines as f32 * 100.0,
        };
        CommitterCoverageUserStat {
            user_id: user_id.to_string(),
            lines,
            covered,
            percent_covered,
        }
    }

    pub fn get_email(&self) -> &str {
        &self.user_id
    }

    pub fn get_lines(&self) -> u32 {
        self.lines
    }

    pub fn get_covered(&self) -> u32 {
        self.covered
    }

    pub fn get_percent_covered(&self) -> f32 {
        self.percent_covered
    }
}

pub fn load_coverage_files() {
    println!("TODO: load coverage files");
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_committer_coverage_user_stat_percent_covered() {
        let user_stat =
            CommitterCoverageUserStat::new("user@example.com", 100, 50);
        assert_eq!(user_stat.get_percent_covered(), 50.0);

        let user_stat =
            CommitterCoverageUserStat::new("user2@example.com", 0, 0);
        assert_eq!(user_stat.get_percent_covered(), 0.0);
    }

    #[test]
    fn test_committer_coverage_summary_set_user_stat() {
        let mut summary = CommitterCoverageSummary::default();

        let email = "user@example.com";
        summary.reset_user(email);
        summary.set_user_stat(email, 10, 5);
        let stats = summary.get_user_stats();
        let user_stat = stats.get(email).unwrap();

        assert_eq!(10, user_stat.get_lines());
        assert_eq!(5, user_stat.get_covered());
    }

    #[test]
    fn test_calculate_by_lines() {
        let mut summary = CommitterCoverageSummary::default();
        let coverage_lines =
            vec![(1, true), (2, false), (3, true), (4, false), (5, true)]
                .into_iter()
                .collect();

        let blame_lines: BTreeMap<u32, BlameLine> = vec![1, 2, 3, 4, 5].iter()
        .map(|i| {
            (*i, BlameLine::new(
                *i,
                format!("commit{}", i).as_str(),
                Some(format!("user{}", i)),
                Some(format!("user{}", i)),
            ))
        }).into_iter()
        .collect();

        CommitterCoverageSummary::calculate_by_lines(
            &coverage_lines,
            &blame_lines,
            &mut summary,
        );

        assert_eq!(5, summary.get_lines());
        assert_eq!(3, summary.get_covered());
        assert_eq!(5, summary.get_user_stats().len());
    }
}
