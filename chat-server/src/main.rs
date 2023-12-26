use std::fs;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use axum::Router;
use serde::Deserialize;
use tower_http::services::ServeDir;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .try_init()
        .expect("Could not start logger!");
    let config = default_load_config().expect("Could not load config file!");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not start runtime!")
        .block_on(async_main(config))
}

async fn async_main(config: Config) {
    let listener = tokio::net::TcpListener::bind(config.socket)
        .await
        .expect("Could not start the socket listener.");

    let router = Router::new().nest_service("/", ServeDir::new(&config.static_dir));

    info!("Listening for connections on {}.", config.socket);

    axum::serve(listener, router)
        .await
        .expect("Main loop server error.");
}

pub fn default_load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_name = Path::new("./chat-server.conf.toml");
    let config_body = fs::read_to_string(config_name)?;
    let mut config: Config = toml::from_str(&config_body)?;
    config.static_dir = canonicalize_and_verify_directory(&config.static_dir)?;
    Ok(config)
}

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

#[derive(Debug, Deserialize)]
pub struct Config {
    pub static_dir: PathBuf,
    pub socket: SocketAddr,
}
