use serde_json;
use std::error::Error;
use std::fmt::Display;
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

/// Intermediate file representation of configurations
pub(crate) mod proto {
    use serde::{Deserialize, Serialize};

    /// This is the main file type/schema
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ConfigurationPrototype {
        pub base_directory: String,
        pub aliases: Vec<PathAliasPrototype>,
        pub port: u16,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PathAliasPrototype {
        pub alias: String,
        pub real: String,
    }
}

/// The `config` module error type.
#[derive(Debug)]
pub enum ConfigurationError {
    NotFound,
    IoError(io::Error),
    SerdeError(serde_json::Error),
}

impl Display for ConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as Debug>::fmt(self, f)
    }
}

impl From<std::io::Error> for ConfigurationError {
    fn from(e: std::io::Error) -> Self {
        Self::IoError(e)
    }
}
impl From<serde_json::Error> for ConfigurationError {
    fn from(e: serde_json::Error) -> Self {
        Self::SerdeError(e)
    }
}

impl Error for ConfigurationError {}

/// Tries to find a file in a directory or its ancestors.
///
/// All ancestors of `directory` will be tested to see if `[ancestor]/file_name`
/// is a file. The first `[ancestor]/file_name` path will be returned.
fn find_file_in_ancestors(directory: &Path, file_name: &Path) -> Option<PathBuf> {
    for dir in directory.ancestors() {
        let possible_path = dir.join(file_name);
        if possible_path.is_file() {
            return Some(possible_path);
        }
    }
    None
}

/// Tries to find and read a configuration file.
///
/// The configuration file will be searched from the specified path and its
/// ancestors.
pub fn get_configuration(
    search_start: &Path,
) -> Result<proto::ConfigurationPrototype, ConfigurationError> {
    let goal = Path::new("server_configuration.json");
    let path = match find_file_in_ancestors(&search_start, &goal) {
        Some(p) => p,
        None => return Err(ConfigurationError::NotFound),
    };
    let file = File::open(path)?;

    Ok(serde_json::from_reader(file)?)
}
