//! This file will analyze the codebase.
use git2::{Blame, BlameHunk};
use mockall::automock;
use std::{collections::BTreeMap, path::Path};

/// Provides blame.
#[automock]
pub trait BlameProvider {
    fn get_file_blame(&self, path: &str) -> Result<BlameFile, String>;
}

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
            git2::opts::set_verify_owner_validation(false).map_err(|err| {
                format!("Failed to set verify owner validation: {}", err)
            })?;
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
}

impl BlameProvider for Git {
    fn get_file_blame(&self, path: &str) -> Result<BlameFile, String> {
        // we don't use ref_name for now
        let blame = self.load_repo_blame_from_path(path)?;
        let mut blame_file = BlameFile::new_from_path(path);

        let mut line_num: u32 = 0;
        for hunk in blame.iter() {
            let res =
                self.iterate_blame_hunk(&hunk, &mut line_num, &mut blame_file);
            res?
        }
        Ok(blame_file)
    }
}

/// Supporting helper for BlameProvider trait.
impl Git {
    fn load_repo_blame_from_path(
        &self,
        path: &str,
    ) -> Result<Blame<'_>, String> {
        let path = Path::new(path);

        let blame = self
            .repo
            .blame_file(path, None)
            .map_err(|err| format!("Failed to get blame: {}", err))?;

        Ok(blame)
    }

    fn iterate_blame_hunk(
        &self,
        hunk: &BlameHunk<'_>,
        line_num: &mut u32,
        blame_file: &mut BlameFile,
    ) -> Result<(), String> {
        let commit_id = hunk.final_commit_id();

        let commit = self
            .repo
            .find_commit(commit_id)
            .map_err(|err| format!("Failed to find commit: {}", err))?;

        let author = commit.author();
        let email : Option<String> = match author.email() {
            Some(email) => Some(email.to_string()),
            None => {
                eprintln!("Failed to get email from author when iterating blame hunk");
                None
            }
        };
        let name: Option<String> = match author.name() {
            Some(name) => Some(name.to_string()),
            None => {
                eprintln!("Failed to get name from author when iterating blame hunk");
                None
            }
        };

        let num_lines = hunk.lines_in_hunk();

        for _i in 0..num_lines {
            *line_num += 1;
            blame_file.add_line(
                *line_num,
                commit_id.to_string().as_str(),
                email.clone(),
                name.clone(),
            )
        }
        Ok(())
    }
}

pub struct BlameFile {
    path: String,
    lines: BTreeMap<u32, BlameLine>,
}

impl BlameFile {
    pub fn new_from_path(path: &str) -> BlameFile {
        BlameFile {
            path: path.to_string(),
            lines: BTreeMap::new(),
        }
    }
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_lines(&self) -> &BTreeMap<u32, BlameLine> {
        &self.lines
    }

    pub fn add_line(&mut self, line: u32, commit: &str, email: Option<String>, name: Option<String>) {
        self.lines.insert(line, BlameLine::new(line, commit, email, name));
    }

    pub fn set_lines_from_vec(&mut self, lines: Vec<BlameLine>) {
        self.lines = lines.into_iter().map(|line| (line.get_line(), line)).collect();
    }
}

pub struct BlameLine {
    line: u32,
    commit: String,
    email: Option<String>,
    name: Option<String>,
}

impl BlameLine {
    pub fn new(line: u32, commit: &str, email: Option<String>, name: Option<String>) -> BlameLine {
        BlameLine {
            line,
            commit: commit.to_string(),
            email,
            name,
        }
    }
    pub fn get_line(&self) -> u32 {
        self.line
    }

    pub fn get_commit(&self) -> &str {
        &self.commit
    }

    pub fn get_email(&self) -> &Option<String> {
        &self.email
    }

    pub fn must_get_email(&self) -> String {
        self.email.clone().unwrap_or("unknown".to_string())
    }

    pub fn get_name(&self) -> &Option<String> {
        &self.name
    }

    pub fn must_get_name(&self) -> String {
        self.name.clone().unwrap_or("unknown".to_string())
    }
}

impl std::fmt::Display for BlameLine {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let email = match &self.email {
            Some(email) => email,
            None => "",
        };
        let name = match &self.name {
            Some(name) => name,
            None => "",
        };
        write!(f, "{}: {} ({} <{}>)", self.line, self.commit, name, email)
    }
}
