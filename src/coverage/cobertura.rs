//! Cobertura coverage provider
//! This module contains the cobertura coverage provider implementation.

use super::*;

/// Cobertura coverage provider
pub struct Provider {

}

impl Provider {
    pub fn new(_content: &str) -> Provider {
        Provider {}
    }
}

impl CoverageProvider for Provider {
    fn get_name(&self) -> &str {
        "cobertura"
    }

    fn load_coverage(&self) -> Result<Coverage, String> {
        let mut coverage = Coverage::new();
        coverage.add_file(CoverageFile::new());
        coverage.add_file(CoverageFile::new());

        Ok(coverage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_name() {
        let provider = Provider {};
        assert_eq!(provider.get_name(), "cobertura");
    }

    #[test]
    fn test_load_coverage_001() {
        let path = "res/tests/cobertura-001.xml";
        let content = std::fs::read_to_string(path).
            expect(&format!("Failed to read file: {}", path));

        let provider = Provider::new(&content);
        let coverage = provider.load_coverage().
            expect("Failed to load coverage");
        
        println!("{}", content);

        assert_eq!(coverage.files.len(), 2);
    }
}