//! Webserver for a chat web-app.
//!
//! The library crate is designed only to help organize code for its binary
//! crate twin.

pub mod config;

/// Initalize the logging system.
///
/// # Panics
/// This function is to be called only once. Specifically, this function will
/// internally call [`log::set_logger`]. For more information, refer to the
/// documentation for [`log::set_logger`].
pub fn init_log(is_test: bool) {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(is_test)
        .init();
}

pub mod warp_server {
    use super::*;
    use std::net::{IpAddr, Ipv6Addr, SocketAddr};
    use warp::filters::{self, BoxedFilter};
    use warp::http::{header::HeaderMap, Method};
    use warp::hyper::body::Bytes;
    use warp::path::FullPath;
    use warp::{self, Filter, Reply};
    pub async fn serve_warp() {
        let config = config::get_configuration_from_current_directory()
            .expect("Could not load configuration.");

        let socket = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), config.get_port());

        log::info!("Configuration: {:?}", config);

        let server = make_filtered_server(config);
        server.run(socket).await
    }

    pub fn make_filtered_server(
        config: config::Configuration,
    ) -> warp::Server<impl Filter<Extract = impl Reply> + Clone + Send + Sync + 'static> {
        let api = make_api();
        let static_files = config.make_static_file_filter();
        let routes = api.or(static_files);
        warp::serve(routes)
    }

    pub fn make_api() -> BoxedFilter<(impl Reply,)> {
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
}
