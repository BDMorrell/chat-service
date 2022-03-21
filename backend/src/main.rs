use std::env;
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::path::Path;
use tokio::runtime;
use warp::filters::{self, BoxedFilter};
use warp::http::{header::HeaderMap, Method};
use warp::hyper::body::Bytes;
use warp::path::FullPath;
use warp::{self, Filter, Reply};

fn main() {
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(serve_warp());
}

async fn serve_warp() {
    chat_backend::init_log(false);

    let config = chat_backend::config::Configuration::from_directory_or_ancestors(
        &env::current_dir().expect("Could not find current working directory!"),
        Path::new(chat_backend::config::DEFAULT_CONFIGURATION_FILENAME),
    )
    .expect("Could not load configuration file!");

    let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), config.get_port());

    log::info!("Configuration: {:?}", config);

    let server = make_filtered_server(config);
    server.run(socket).await
}

fn make_filtered_server(
    config: chat_backend::config::Configuration,
) -> warp::Server<impl Filter<Extract = impl Reply> + Clone + Send + Sync + 'static> {
    let api = make_api();
    let static_files = config.make_static_file_filter();
    let routes = api.or(static_files);
    warp::serve(routes)
}

fn make_api() -> BoxedFilter<(impl Reply,)> {
    let hello = filters::path::path("hello")
        .and(filters::method::get())
        .map(|| "Hello, world!");
    // const ECHO_BODY_LENGTH_LIMIT: u64 = 1024 * 8;
    let echo = filters::path::path("echo")
        // .and(filters::header::optional("name"))
        // .and(filters::body::content_length_limit(ECHO_BODY_LENGTH_LIMIT))
        .and(filters::body::bytes())
        .and(filters::method::method())
        .and(filters::path::full())
        .and(filters::header::headers_cloned())
        .map(
            move |body: Bytes, method: Method, path: FullPath, headers: HeaderMap| {
                let first_line = format!("{} {}", method, path.as_str());
                let mut heads = String::new();
                for (name, value) in headers.into_iter() {
                    let name = match name {
                        Some(header) => String::from(header.as_str()),
                        None => String::from(""),
                    };
                    let header_string =
                        format!("{}: {}\n", name, String::from_utf8_lossy(value.as_bytes()));
                    heads.push_str(&header_string);
                }
                format!(
                    "{}\n{}\n{}",
                    first_line,
                    heads,
                    String::from_utf8_lossy(&body)
                )
            },
        )
        .with(filters::reply::header("Content-Type", "text/plain"));

    filters::path::path("api").and(echo.or(hello)).boxed()
}
