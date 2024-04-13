//! This module contains the coverage analysis for the project.
use std::vec;

mod cobertura;

/// Represents the coverage provider that can load the coverage statistics from a file.
trait CoverageProvider {
    fn get_name(&self) -> &str;
    fn load_coverage(&self) -> Result<Coverage, String>;
}

/// Represents the coverage statistics for a single project or coverage file.
/// This will be used to calculate the overall coverage for the project.
/// This contains list of files with their coverage stats.
#[derive(Clone)]
pub struct Coverage {
    files: Vec<CoverageFile>,
}

impl Coverage {
    /// Creates a new coverage object from the coverage file in the given path.
    /// It will return an error if the file cannot be read or parsed.
    pub fn new_from_path(path: &str) -> Result<Coverage, String> {
        // read the file
        let content = std::fs::read_to_string(path)
            .map_err(|err| format!("Failed to read coverage file: {}", err))?;

        let provider = Coverage::create_provider_from_content(&content)
            .map_err(|err| format!("Failed to create coverage provider: {}", err))?;
    
        let coverage = provider.load_coverage()
            .map_err(|err| format!("Failed to load coverage files: {}", err))?;

        Ok(coverage)
    }

    pub fn new() -> Coverage {
        Coverage { 
            files: vec![] 
        }
    }

    pub fn add_file(&mut self, file: CoverageFile) {
        self.files.push(file)
    }

    pub fn get_file_count(&self) -> usize {
        self.files.len()
    }
    
    fn create_provider_from_content(content: &str) -> Result<Box<dyn CoverageProvider>, String> {
        let provider = "cobertura";

        match provider {
            "cobertura" => {
                let provider = cobertura::Provider::new(content);
                Ok(Box::new(provider))
            }
            _ => Err(format!("Unknown coverage provider: {}", provider)),
        }
    }
}

/// Represents the coverage statistics for a single file.
/// This contains the file path and the coverage stats for that file.
#[derive(Clone, Default)]
pub struct CoverageFile {
    path: String,
}

impl CoverageFile {
    pub fn get_path(&self) -> &str {
        &self.path
    }
}

pub fn load_coverage_files() {
    println!("Loading coverage files...");
}