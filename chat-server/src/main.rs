use std::fs;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use axum_extra::{headers, TypedHeader};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::level_filters::LevelFilter;
use tracing::{event, info, warn};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
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

    let notifyAllTx = broadcast::Sender::new(2048);

    let router = Router::new()
        .route(
            "/api/ws",
            get(|ws: WebSocketUpgrade| -> impl IntoResponse {}),
        )
        .nest_service("/", ServeDir::new(&config.static_dir))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    info!("Listening for connections on {}.", config.socket);

    axum::serve(listener, router)
        .await
        .expect("Main loop server error.");
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    Text(String),
}

#[derive(Debug)]
pub struct WebsocketSharedState {
    pub sender: broadcast::Sender<Event>,
}

#[derive(Debug)]
pub struct SharedStateWebsocketHandler {
    state: Arc<WebsocketSharedState>,
}

impl SharedStateWebsocketHandler {
    pub fn new(state: Arc<WebsocketSharedState>) -> Self {
        Self(state)
    }

    fn handle_connection(self: &mut Self, ws: WebSocketUpgrade) -> impl IntoResponse {
        let new_sender = self.sender.clone();
        ws.protocols("custom")
            .on_failed_upgrade(|error| {
                warn!("Websocket failed to upgrade: {}", error);
            })
            .on_upgrade(move |socket| {
                let tx = new_sender;
                let rx = tx.subscribe();
                let (sender, receiver) = socket.split();
                tokio::spawn(handle_writing(sender, rx));
                tokio::spawn(handle_reading(receiver, tx));
            })
    }

    async fn handle_writing(sender: SplitStream<WebSocket>, event_source: broadcast::Receiver<Event>) {
        while Some(event) = await event_source.next() {
            write!(sender, event);
        }
    }

    async fn handle_reading(reader: SplitSink<WebSocket, Message>, event_target: broadcast::Sender<Event>) {
        while Some(text) = await reader.next() {
            match Json::from_bytes(text) {
                Some(message) => event_target.send(message),
                Err(e) => warn!("Bad event from <anonymous>: {}", e),
            }
        }
    }
}
