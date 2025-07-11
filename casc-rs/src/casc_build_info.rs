use crate::error::CascError;
use crate::utility::dsv_file::DSVFile;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents build information loaded from a CASC `.build.info` file.
#[derive(Debug)]
pub struct CascBuildInfo {
    variables: HashMap<String, Variable>,
}

/// Represents a variable entry in the build info.
#[derive(Debug)]
pub(crate) struct Variable {
    pub(crate) name: String,
    pub(crate) var_type: String,
    pub(crate) value: String,
}

impl Variable {
    /// Creates a new `Variable` with the given name, type, and value.
    pub(crate) fn new(name: String, var_type: String, value: String) -> Self {
        Variable {
            name,
            var_type,
            value,
        }
    }
}

impl CascBuildInfo {
    /// Creates a new, empty `CascBuildInfo`.
    pub(crate) fn new() -> Self {
        CascBuildInfo {
            variables: HashMap::new(),
        }
    }

    /// Loads build info from the specified file and returns a new instance.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the `.build.info` file.
    pub(crate) fn with_file(file_name: &PathBuf) -> Result<Self, CascError> {
        let mut instance = CascBuildInfo::new();
        instance.load(file_name)?;
        Ok(instance)
    }

    /// Retrieves the value of a variable by name, or returns the provided default value if not found.
    ///
    /// # Arguments
    ///
    /// * `var_name` - The name of the variable to retrieve.
    /// * `default_value` - The value to return if the variable is not found.
    pub(crate) fn get(&self, var_name: &str, default_value: &str) -> String {
        if let Some(var) = self.variables.get(var_name) {
            var.value.clone()
        } else {
            default_value.to_string()
        }
    }

    /// Loads build info variables from the specified file into this instance.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the `.build.info` file.
    pub(crate) fn load<P: AsRef<Path>>(&mut self, file_name: P) -> Result<(), CascError> {
        let dsv = DSVFile::from_file(file_name, "|", Some("#"))?;
        let rows = dsv.rows;
        if rows.len() < 2 {
            return Err(CascError::FileCorrupted("Not enough rows".into()));
        }
        let header = &rows[0];
        let data = &rows[1];
        if header.len() != data.len() {
            return Err(CascError::FileCorrupted(
                "Header/data length mismatch".to_string(),
            ));
        }
        for (info, value) in header.iter().zip(data.iter()) {
            let split: Vec<&str> = info.split('!').collect();
            if split.len() < 2 {
                return Err(CascError::FileCorrupted("Header format invalid".into()));
            }
            let var = Variable {
                name: split[0].to_string(),
                var_type: split[1].to_string(),
                value: value.clone(),
            };
            self.variables.insert(split[0].to_string(), var);
        }
        Ok(())
    }
}
