use axum::Router;
use chat_server::config;
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
    let config = config::default_load_config().expect("Could not load config file!");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not start runtime!")
        .block_on(async_main(config))
}

async fn async_main(config: config::Config) {
    let listener = tokio::net::TcpListener::bind(config.socket)
        .await
        .expect("Could not start the socket listener.");

    let router = Router::new().nest_service("/", ServeDir::new(&config.static_dir));

    info!("Listening for connections on {}.", config.socket);

    axum::serve(listener, router)
        .await
        .expect("Main loop server error.");
}
