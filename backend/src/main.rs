use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::Path;
use warp::{self, filters, Filter};

#[tokio::main]
async fn main() {
    chat_backend::init_log(false);

    let config = chat_backend::config::Configuration::from_directory_or_ancestors(
        &env::current_dir().expect("Could not find current working directory!"),
        Path::new(chat_backend::config::DEFAULT_CONFIGURATION_FILENAME),
    )
    .expect("Could not load configuration file!");

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), config.get_port());

    log::info!("Configuration: {:?}", config);

    let api = filters::path::path("api");
    let api_reply = api.map(|| "This is api.\n");
    let static_files = config.make_static_filter();

    let filter = api_reply.or(static_files);

    let server = warp::serve(filter).run(socket);
    server.await
}
