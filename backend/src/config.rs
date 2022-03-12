//! Utilities for configuration files.
//!
//! To load and use configurations, see [`Configuration`]. All errors will be of
//! type [`ConfigurationError`].
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


/// The default filename for the configuration file.
const SERVER_CONFIGURATION_FILENAME: &str = "server_configuration.json";

/// File 'schemas' for the configuration file
pub(crate) mod proto {
    use serde::{Deserialize, Serialize};

    /// This is the main file 'schema'
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ConfigurationPrototype {
        /// Where the static content to should be found at.
        pub base_directory: String,
        /// Which port the server is to listen from.
        pub port: u16,
    }
}

/// The list of errors that may come from trying to use the configuration file.
#[derive(Debug)]
pub enum ConfigurationError {
    /// The configuration file could not be found.
    NotFound,
    /// For errors that are from [`io`].
    IoError(io::Error),
    /// For errors that are from [`serde`], or packages that implement file
    /// formats.
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
    let (directory, configuration_path) =
        find_file_in_ancestors(search_start, goal).ok_or(ConfigurationError::NotFound)?;

    let file = File::open(&configuration_path)?;

    let raw_config = serde_json::from_reader(file)?;

    Ok(Configuration {
        proto: raw_config,
        config_directory: directory,
        config_path: configuration_path,
    })
}

/// Configuration information holder.
#[derive(Debug)]
pub struct Configuration {
    /// The parsed contents of the configuration file.
    proto: proto::ConfigurationPrototype,
    /// Where the configuration file was found.
    ///
    /// See [`Self::get_configuration_directory`].
    config_directory: PathBuf,
    /// Path to the configuration file.
    config_path: PathBuf,
}

impl Configuration {
    /// Returns the port to be used.
    pub fn get_port(&self) -> u16 {
        self.proto.port
    }

    /// Returns the directory where the configuration file was found.
    ///
    /// This is especially useful for decoding relative file paths that were in
    /// the configuration file.
    pub fn get_configuration_directory(&self) -> &Path {
        self.config_directory.as_path()
    }

    /// Returns the [`Path`] to the configuration file.
    pub fn get_configuration_file_path(&self) -> &Path {
        self.config_path.as_path()
    }

    /// Makes a static file server filter.
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
