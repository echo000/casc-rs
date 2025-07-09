use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::{Path, PathBuf};

/// Represents the configuration for a CASC storage, containing variables parsed from config files.
#[derive(Debug)]
pub struct CascConfig {
    variables: HashMap<String, Variable>,
}

/// Represents a variable in the CASC configuration, with a name and a list of values.
#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Variable {
    /// The name of the variable.
    pub name: String,
    /// The values associated with the variable.
    pub values: Vec<String>,
}

impl Variable {
    /// Creates a new `Variable` with the given name and values.
    pub fn new(name: String, values: Vec<String>) -> Self {
        Variable { name, values }
    }
}

impl CascConfig {
    /// Creates a new, empty `CascConfig`.
    pub fn new() -> Self {
        CascConfig {
            variables: HashMap::new(),
        }
    }

    /// Retrieves a variable by name, if it exists.
    pub fn get(&self, var_name: &str) -> Option<&Variable> {
        self.variables.get(var_name)
    }

    /// Loads configuration variables from a file.
    ///
    /// # Arguments
    ///
    /// * `file_name` - The path to the configuration file.
    pub fn load<P: AsRef<Path>>(&mut self, file_name: P) -> Result<(), Error> {
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Ignore empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Split the line into key-value parts
            if let Some((name, value)) = line.split_once('=') {
                let name = name.trim().to_string();
                let values: Vec<String> = value.split_whitespace().map(|v| v.to_string()).collect();

                let variable = Variable::new(name, values);
                self.variables.insert(variable.name.clone(), variable);
            }
        }

        Ok(())
    }
}
