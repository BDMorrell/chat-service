use std::fs;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub fn canonicalize_and_verify_directory(path: &Path) -> Result<PathBuf, std::io::Error> {
    let directory = path.canonicalize()?;
    if !(directory.is_dir()) {
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("{} is not a directory", directory.display()),
        ))
    } else {
        Ok(directory)
    }
}

pub fn default_load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_name = Path::new("./chat-server.conf.toml");
    let config_body = fs::read_to_string(config_name)?;
    let mut config: Config = toml::from_str(&config_body)?;
    config.static_dir = canonicalize_and_verify_directory(&config.static_dir)?;
    Ok(config)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub static_dir: PathBuf,
    pub socket: SocketAddr,
}
