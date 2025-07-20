use crate::error::CascError;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

/// A struct to hold a Delimiter Separated Value File (DSV)
#[derive(Debug)]
pub(crate) struct DSVFile {
    /// The delimiter string
    delimiter: String,
    /// The comment indicator string
    comment: Option<String>,
    /// The rows within the DSV file
    pub(crate) rows: Vec<Vec<String>>,
}

impl DSVFile {
    /// Initializes a new instance of the DSVFile with default delimiter (",")
    pub(crate) fn new() -> Self {
        Self {
            delimiter: ",".to_string(),
            comment: None,
            rows: Vec::new(),
        }
    }

    /// Initializes a new instance with a given delimiter
    pub(crate) fn with_delimiter(delimiter: &str) -> Self {
        Self {
            delimiter: delimiter.to_string(),
            comment: None,
            rows: Vec::new(),
        }
    }

    /// Initializes a new instance with a given file, delimiter, and optional comment string
    pub(crate) fn from_file<P: AsRef<Path>>(
        file: P,
        delimiter: &str,
        comment: Option<&str>,
    ) -> Result<Self, CascError> {
        let file = File::open(file)?;
        let mut dsv = Self {
            delimiter: delimiter.to_string(),
            comment: comment.map(|s| s.to_string()),
            rows: Vec::new(),
        };
        dsv.load(file)?;
        Ok(dsv)
    }

    /// Loads DSV data from a reader (e.g., File, BufReader, etc.)
    pub(crate) fn load<R: Read>(&mut self, reader: R) -> Result<(), CascError> {
        let buffered = BufReader::new(reader);
        let supports_commenting = self.comment.as_deref().is_some_and(|c| !c.is_empty());

        for line in buffered.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            if supports_commenting {
                if let Some(ref comment) = self.comment {
                    if line.starts_with(comment) {
                        continue;
                    }
                }
            }
            let row: Vec<String> = line.split(&self.delimiter).map(|s| s.to_string()).collect();
            self.rows.push(row);
        }
        Ok(())
    }

    /// Gets the header row, if any (first row)
    pub(crate) fn header(&self) -> Option<&Vec<String>> {
        self.rows.first()
    }
}
