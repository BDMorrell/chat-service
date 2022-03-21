//! Utilities for configuration files.
//!
//! To load and use configurations, see [`Configuration`]. All errors will be of
//! type [`ConfigurationError`].
use serde_json;
use std::env;
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

pub fn get_configuration_from_current_directory() -> Result<Configuration, ConfigurationError> {
    let current_directory = env::current_dir()?;
    Configuration::from_directory_or_ancestors(
        &current_directory,
        Path::new(DEFAULT_CONFIGURATION_FILENAME),
    )
}

/// The default filename for the configuration file.
pub const DEFAULT_CONFIGURATION_FILENAME: &str = "server_configuration.json";

/// File 'schemas' for the configuration file
mod proto {
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
    /// The specified configuration path is invalid.
    InvalidPath,
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
    //! This impl is for utilities for finding and loading a configuration file.

    /// Try to find and load a configuration from a specified directory.
    pub fn from_directory_or_ancestors(
        directory: &Path,
        file_name: &Path,
    ) -> Result<Configuration, ConfigurationError> {
        let configuration_file_path: PathBuf = Self::find_configuration_file(directory, file_name)
            .ok_or(ConfigurationError::NotFound)?;
        Self::from_file(&configuration_file_path)
    }

    /// Create a [`Configuration`] from a specified file.
    pub fn from_file(file_name: &Path) -> Result<Configuration, ConfigurationError> {
        let config_path = file_name.to_path_buf();
        let config_directory = file_name
            .parent()
            .ok_or(ConfigurationError::InvalidPath)?
            .to_path_buf();

        let file = File::open(config_path.clone())?;

        let proto = serde_json::from_reader(file)?;

        Ok(Configuration {
            proto,
            config_directory,
            config_path,
        })
    }

    /// Find the configuration file by looking in a directory and its ancestors.
    ///
    /// The configuration file path returned is in the nearest ancestor of the
    /// specified directory.
    ///
    /// # Note
    /// The ancestry of the given directory is based on [`Path::ancestors`].
    fn find_configuration_file(directory: &Path, file_name: &Path) -> Option<PathBuf> {
        for dir in directory.ancestors() {
            let possible_path = dir.join(file_name);
            if possible_path.is_file() {
                return Some(possible_path);
            }
        }
        None
    }
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

    /// Returns the path to the static file directory.
    ///
    /// The result has been canonicalized, if possible.
    pub fn get_static_file_directory(&self) -> PathBuf {
        let path = self.config_directory.join(&self.proto.base_directory);
        path.canonicalize().ok().unwrap_or(path)
    }

    /// Makes a static file server filter.
    pub fn make_static_file_filter(&self) -> BoxedFilter<(impl Reply,)> {
        filters::fs::dir(self.get_static_file_directory()).boxed()
    }
}
