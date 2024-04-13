//! This file will analyze the codebase.
use std::path::Path;
pub struct Git {
    path: String,
    repo: git2::Repository,
}

impl Git {
    /// Opens an existing git repository from the given path.
    /// By default, the program will pass the $GITHUB_WORKSPACE environment variable.
    /// Since the owner of the repository is not verified, we need to disable the owner validation.
    /// 
    /// In git we usually use `git --global safe.directory /path/to/repo` to set the safe directory.
    /// This is to prevent the user from accidentally running git commands in the wrong directory.
    /// 
    /// Since the $GITHUB_WORKSPACE is not a safe directory, we need to disable the owner validation.
    /// This will call git2::opts::set_verify_owner_validation(false) to disable the owner validation.
    /// 
    /// This returns an error if the repository cannot be opened.
    pub fn new_from_path(path: &str) -> Result<Git, String> {
        unsafe {
            git2::opts::set_verify_owner_validation(false)
                .map_err(|err| format!("Failed to set verify owner validation: {}", err))?;
        }

        let repo = git2::Repository::open(path)
            .map_err(|err| format!("Failed to open git repository: {}", err))?;
        Ok(Git {
            path: path.to_string(),
            repo,
        })
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_last_commit_hash(&self) -> Result<String, String> {
        let obj = self
            .repo
            .head()
            .map_err(|err| format!("Failed to get head: {}", err))?
            .resolve()
            .map_err(|err| format!("Failed to resolve head: {}", err))?
            .peel_to_commit()
            .map_err(|err| format!("Failed to peel to commit: {}", err))?;

        Ok(obj.id().to_string())
    }

    pub fn get_blame_file(&self, path: &str) -> Result<BlameFile, String> {
        let path = Path::new(path);

        let blame = self
            .repo
            .blame_file(path, None)
            .map_err(|err| format!("Failed to get blame: {}", err))?;

        let mut blame_file = BlameFile {
            path: path.to_string_lossy().to_string(),
            lines: vec![],
        };
        let mut line = 0;
        for hunk in blame.iter() {
            let commit_id = hunk.final_commit_id();
            let num_lines = hunk.lines_in_hunk();
            let commit = self
                .repo
                .find_commit(commit_id)
                .map_err(|err| format!("Failed to find commit: {}", err))?;

            let author = commit.author();
            let email = author.email().unwrap_or("unknown");
            for _i in 0..num_lines {
                line += 1;
                let blame_line = BlameLine {
                    line,
                    commit: commit_id.to_string(),
                    email: email.to_string(),
                };
                blame_file.lines.push(blame_line);
            }
        }

        Ok(blame_file)
    }
}

pub struct BlameFile {
    path: String,
    lines: Vec<BlameLine>,
}

impl BlameFile {
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_lines(&self) -> &Vec<BlameLine> {
        &self.lines
    }
}

pub struct BlameLine {
    line: u32,
    commit: String,
    email: String,
}

impl BlameLine {
    pub fn get_line(&self) -> u32 {
        self.line
    }

    pub fn get_commit(&self) -> &str {
        &self.commit
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }
}

impl std::fmt::Display for BlameLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {} <{}>", self.line, self.commit, self.email)
    }
}
