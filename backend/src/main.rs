use chat_backend;
use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use warp::{self, Filter};
use log;

#[tokio::main]
async fn main() {
    chat_backend::init_log(false);

    let config = chat_backend::config::get_configuration(
        &env::current_dir().expect("Could not find current working directory!"),
    ).expect("Could not load configuration file!");

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), config.port);

    log::info!("Configuration: {:?}", config);

    let api = warp::path!("api" / String).map(|path| format!("{:?}", path));

    let server = warp::serve(api).run(socket);
    server.await
}
