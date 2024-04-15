//! Cobertura coverage provider
//! This module contains the cobertura coverage provider implementation.

use super::{CoverageFileIteratorResult, CoverageProvider, FileCoverage};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::collections::HashMap;

type XmlReader = quick_xml::reader::Reader<std::io::BufReader<std::fs::File>>;

/// Cobertura coverage provider
pub struct Provider {
    path: String,
}

impl Provider {
    pub fn load_from_file(path: &str) -> Result<Provider, String> {
        Ok(Provider {
            path: path.to_string(),
        })
    }
}

impl CoverageProvider for Provider {
    fn get_name(&self) -> &str {
        "cobertura"
    }

    fn iter_files(&self) -> CoverageFileIteratorResult {
        let iter = CoverageFileIterator::new(&self.path)
            .map_err(|e| format!("Failed to create iterator: {}", e))?;
        Ok(Box::new(iter))
    }
}

pub struct CoverageFileIterator {
    reader: XmlReader,
    level: u32,
}

enum ReadEventReturn {
    Return,
    Continue,
    End,
}

impl CoverageFileIterator {
    pub fn new(path: &str) -> Result<CoverageFileIterator, String> {
        let mut reader = Reader::from_file(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        reader.trim_text(true);
        reader.expand_empty_elements(true);
        Ok(CoverageFileIterator { reader, level: 0 })
    }
}

/// Implementation of helper functions for the Iterator trait for CoverageFileIterator
impl CoverageFileIterator {
    /// Read the next event into the buffer and update the coverage file.
    /// This will return the coverage file if the end of the class tag is reached.
    /// Otherwise, it will return None.
    fn read_event_into(
        &mut self,
        buf: &mut Vec<u8>,
        coverage_file: &mut FileCoverage,
    ) -> ReadEventReturn {
        match self.reader.read_event_into(buf) {
            Err(e) => {
                let pos = self.reader.buffer_position();
                eprintln!("Error at position {}: {:?}", pos, e);
                ReadEventReturn::End
            }
            Ok(Event::Start(e)) => {
                self.inc_level();
                self.match_start_tag(&e, coverage_file)
            }
            Ok(Event::End(e)) => {
                let res = self.match_end_tag(&e, coverage_file);
                self.dec_level();
                res
            }
            Ok(Event::Text(e)) => {
                println!("{:?}", e.unescape().unwrap());
                ReadEventReturn::Continue
            }
            Ok(Event::Eof) => ReadEventReturn::End,
            _ => ReadEventReturn::Continue,
        }
    }

    fn reset_and_empty_file_coverage(&mut self) -> Option<FileCoverage> {
        self.level = 0;
        None
    }

    fn inc_level(&mut self) {
        self.level += 1;
    }

    fn dec_level(&mut self) {
        self.level -= 1;
    }

    fn match_start_tag(
        &self,
        e: &quick_xml::events::BytesStart,
        coverage_file: &mut FileCoverage,
    ) -> ReadEventReturn {
        let tag_name = get_start_tag_name(e);
        let attr = get_attributes(e);
        match (tag_name.as_str(), self.level) {
            ("class", 5) => {
                coverage_file.reset();
                if let Some(filename) = attr.get("filename") {
                    coverage_file.path = filename.to_string();
                    ReadEventReturn::Continue
                } else {
                    eprintln!("No filename attribute found");
                    ReadEventReturn::End
                }
            }
            ("line", 6..=8) => {
                let number = attr.get("number");
                let hits = attr.get("hits");
                if let (Some(number), Some(hits)) = (number, hits) {
                    let number = number.parse::<u32>().unwrap_or(0);
                    if number == 0 {
                        return ReadEventReturn::Continue;
                    }
                    let hits = hits.parse::<u32>().unwrap_or(0);
                    let is_covered = hits > 0;
                    coverage_file.add_line(number, is_covered);
                }
                ReadEventReturn::Continue
            }
            _ => ReadEventReturn::Continue,
        }
    }

    fn match_end_tag(
        &self,
        e: &quick_xml::events::BytesEnd,
        _coverage_file: &mut FileCoverage,
    ) -> ReadEventReturn {
        let tag_name = get_end_tag_name(e);
        match (tag_name.as_str(), self.level) {
            ("class", 5) => ReadEventReturn::Return,
            _ => ReadEventReturn::Continue,
        }
    }
}

impl Iterator for CoverageFileIterator {
    type Item = FileCoverage;

    /// Read the next coverage file from the cobertura file.
    /// This will return the coverage file if the end of the class tag is reached.
    /// Otherwise, it will return reset the state and return None.
    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        let mut coverage_file: FileCoverage = FileCoverage::default();
        loop {
            match self.read_event_into(&mut buf, &mut coverage_file) {
                ReadEventReturn::Return => {
                    return Some(coverage_file);
                }
                ReadEventReturn::End => {
                    if self.level > 0 {
                        eprintln!("Unexpected end of file");
                    }
                    return self.reset_and_empty_file_coverage();
                }
                ReadEventReturn::Continue => {}
            }
        }
    }
}

fn get_attributes(
    e: &quick_xml::events::BytesStart,
) -> HashMap<String, String> {
    e.attributes()
        .map(|a| {
            let a = a.unwrap();
            let key = a.key.as_ref();
            let key = std::str::from_utf8(key).unwrap_or("").to_string();
            let value = a.value.as_ref();
            let value = std::str::from_utf8(value).unwrap_or("").to_string();
            (key, value)
        })
        .collect()
}

fn get_start_tag_name(e: &quick_xml::events::BytesStart) -> String {
    std::str::from_utf8(e.name().as_ref())
        .unwrap_or("")
        .to_string()
}

fn get_end_tag_name(e: &quick_xml::events::BytesEnd) -> String {
    std::str::from_utf8(e.name().as_ref())
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_name() {
        let provider = Provider {
            path: "test".to_string(),
        };
        assert_eq!(provider.get_name(), "cobertura");
    }

    #[test]
    fn test_coveragefileiterator_test_reader_cobertura_001() {
        let path = "res/tests/cobertura-001.xml";
        let iter =
            CoverageFileIterator::new(path).expect("Failed to create iterator");
        let files: Vec<FileCoverage> = iter.into_iter().collect();
        assert_eq!(files.len(), 4);
    }

    #[test]
    fn test_coveragefileiterator_test_reader_cobertura_002() {
        let path = "res/tests/cobertura-002.xml";
        let iter =
            CoverageFileIterator::new(path).expect("Failed to create iterator");
        let files: Vec<FileCoverage> = iter.into_iter().collect();
        assert_eq!(files.len(), 4);
    }

    #[test]
    fn test_coveragefileiterator_test_reader_cobertura_003() {
        let path = "res/tests/cobertura-003.xml";
        let iter =
            CoverageFileIterator::new(path).expect("Failed to create iterator");
        let files: Vec<FileCoverage> = iter.into_iter().collect();
        assert_eq!(files.len(), 2083);
    }
}
