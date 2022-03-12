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
        pub port: u16,
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
fn find_file_in_ancestors(directory: &Path, file_name: &Path) -> Option<(PathBuf, PathBuf)> {
    for dir in directory.ancestors() {
        let possible_path = dir.join(file_name);
        if possible_path.is_file() {
            return Some((dir.to_path_buf(), possible_path));
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
    let (directory, configuration_path) = find_file_in_ancestors(&search_start, &goal)
        .ok_or(ConfigurationError::NotFound)?;

    let file = File::open(&configuration_path)?;

    let raw_config = serde_json::from_reader(file)?;

    Ok(Configuration {
        proto: raw_config,
        config_directory: directory,
        config_path: configuration_path,
    })
}

#[derive(Debug)]
pub struct Configuration {
    proto: proto::ConfigurationPrototype,
    /// Where the "server_configuration.json" file was found
    config_directory: PathBuf,
    config_path: PathBuf,
}

impl Configuration {
    pub fn get_port(&self) -> u16 {
        self.proto.port
    }

    /// Makes a static file server filter.
    ///
    /// Aliases take priority over files with the same name.
    pub fn make_static_filter(&self) -> BoxedFilter<(impl Reply,)> {
        let local_directory: PathBuf = {
            let path = self.config_directory.join(&self.proto.base_directory);
            if let Ok(canon) = path.canonicalize() {
                canon
            } else {
                path
            }
        };

        let method_filter = filters::method::get().or(filters::method::head()).unify();
        let directory_filter = filters::fs::dir(local_directory);

        method_filter.and(directory_filter).boxed()
    }
}
