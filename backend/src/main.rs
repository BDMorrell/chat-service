use chat_backend;
use log;
use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use warp::{self, filters, Filter};

#[tokio::main]
async fn main() {
    chat_backend::init_log(false);

    let config = chat_backend::config::get_configuration(
        &env::current_dir().expect("Could not find current working directory!"),
    )
    .expect("Could not load configuration file!");

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), config.get_port());

    log::info!("Configuration: {:?}", config);

    let filter = warp::filters::log::log(name);

    let api = warp::path!("api" / String).map(|path| format!("{:?}", path));
    let static_files = config.make_static_filter();

    let filter = filter.or(api).or(static_files);

    let server = warp::serve(filter).run(socket);
    server.await
}