use chat_server::api;
use chat_server::config;
use hyper::server::Server;
use hyper::service::make_service_fn;
use std::convert::Infallible;
use tower::service_fn;

#[tokio::main]
async fn main() {
    chat_server::init_log(false);

    let config = config::get_configuration_from_current_directory()
        .expect("Could not load server configuration");

    let socket = config.socket();

    let service = make_service_fn(|_| async { Ok::<_, Infallible>(service_fn(api::echo)) });

    let server = Server::bind(&socket).serve(service);

    if let Err(e) = server.await {
        eprintln!("Error: {}", e);
    }
}
