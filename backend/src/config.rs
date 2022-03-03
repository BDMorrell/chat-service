use serde_json;
use std::error::Error;
use std::fmt::Display;
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use warp::{
    filters::{self, BoxedFilter},
    Filter, Reply,
};

const SERVER_CONFIGURATION_FILENAME: &str = "server_configuration.json";

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
        pub display: String,
        pub local: String,
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
pub fn get_configuration(search_start: &Path) -> Result<Configuration, ConfigurationError> {
    let goal = Path::new(SERVER_CONFIGURATION_FILENAME);
    let path = match find_file_in_ancestors(&search_start, &goal) {
        Some(p) => p,
        None => return Err(ConfigurationError::NotFound),
    };
    let path = path.canonicalize()?;

    let file = File::open(path)?;

    let raw_config = serde_json::from_reader(file)?;

    Ok(Configuration {
        proto: raw_config,
        proto_path: path.parent(),
    })
}

#[derive(Debug)]
pub struct Configuration {
    proto: proto::ConfigurationPrototype,
    /// Where the "server_configuration.json" file was found
    proto_path: PathBuf,
}

impl Configuration {
    pub fn get_port(&self) -> u16 {
        self.proto.port
    }

    /// Makes a static file server filter.
    ///
    /// Aliases take priority over files with the same name.
    pub fn make_static_filter(&self) -> BoxedFilter<(impl Reply,)> {
        let mut result = filters::any::any();
        let local_directory: PathBuf = {
            let path = self.proto_path.join(self.proto.base_directory);
            if let Ok(canon) = path.canonicalize() {
                canon
            } else {
                path
            }
        }.to_path_buf();

        for alias in self.proto.aliases {
            let mut display: String = String::from(alias.display.trim_start_matches("/"));
            let local = Path::new(&alias.local);
            let rule = filters::path::path(&display);
            let file_filter = filters::fs::file(&local);
            result = result.or(rule).and(file_filter);
        }

        let directory_filter = filters::fs::dir(local_directory);

        result = result.or(directory_filter); // lowest priority

        result
    }
}
