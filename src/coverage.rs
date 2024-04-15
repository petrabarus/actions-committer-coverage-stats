//! This module contains the coverage analysis for the project.

use std::collections::BTreeMap;

mod cobertura;

type CoverageFileIteratorResult = Result<Box<dyn Iterator<Item = FileCoverage>>, String>;
/// Represents the coverage provider that can load the coverage statistics from a file.
pub trait CoverageProvider {
    fn get_name(&self) -> &str;
    fn iter_files(&self) -> CoverageFileIteratorResult;
}

pub struct Coverage {
    path: String,
    provider: Option<Box<dyn CoverageProvider>>,
}

impl Coverage {
    pub fn new_from_path(path: &str) -> Result<Coverage, String> {
        let provider = cobertura::Provider::load_from_file(path)
            .map_err(|e| format!("Failed to load coverage file: {}", e))?;
        Ok(Coverage {
            path: path.to_string(),
            provider: Some(Box::new(provider)),
        })
    }

    pub fn get_path(&self) -> &str {
        &self.path
    }
}

impl CoverageProvider for Coverage {
    fn get_name(&self) -> &str {
        match &self.provider {
            None => "unknown",
            Some(provider) => {
                provider.get_name()
            }
        }
    }

    fn iter_files(&self) -> CoverageFileIteratorResult {
        match &self.provider {
            None => Err("No provider".to_string()),
            Some(provider) => provider.iter_files(),
        }
    }
}

pub struct FileCoverage {
    path: String,
    line_cover: BTreeMap<u32, bool>,
}

impl Default for FileCoverage {
    fn default() -> Self {
        FileCoverage {
            path: "".to_string(),
            line_cover: BTreeMap::new(),
        }
    }
}

impl FileCoverage {
    pub fn get_path(&self) -> &str {
        &self.path
    }

    pub fn get_lines(&self) -> Vec<FileCoverageLine> {
        self.line_cover.iter().map(|(line, covered)| {
            FileCoverageLine {
                line: *line,
                covered: *covered,
            }
        }).collect()
    }

    pub fn add_line(&mut self, line_number: u32, covered: bool) {
        self.line_cover.insert(line_number, covered);
    }

    pub fn reset(&mut self) {
        self.path.clear();
        self.line_cover.clear()
    }
}

pub struct FileCoverageLine {
    line: u32,
    covered: bool,
}

impl FileCoverageLine {
    pub fn get_line(&self) -> u32 {
        self.line
    }

    pub fn is_covered(&self) -> bool {
        self.covered
    }
}