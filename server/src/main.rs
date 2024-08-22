use axum::{http::StatusCode, Router};
use chat_service::ChatServiceState;
use chatroom::{Chatroom, IncomingMessage};
use server::config;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::TRACE.into())
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

    let mut chatroom = Chatroom::new();
    // preload a message to remove an edge case.
    // TODO: try not preloading a message.
    chatroom.add(
        IncomingMessage {
            sender: "<system>".into(),
            body: "Welcome to this chat room!".into(),
        }
        .into(),
    );
    let chat_service_state = ChatServiceState::with_room(chatroom);
    let chat_service_router = chat_service::router().with_state(chat_service_state);
    let api = Router::new()
        .nest("/chat", chat_service_router)
        .fallback(just_not_found);
    let router = Router::new()
        .nest("/api", api)
        .nest_service("/", ServeDir::new(&config.static_dir))
        .layer(TraceLayer::new_for_http());

    info!("Listening for connections on {}.", config.socket);

    axum::serve(listener, router)
        .await
        .expect("Main loop server error.");
}

async fn just_not_found() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "")
}
