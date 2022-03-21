use tokio::runtime;

fn main() {
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(chat_backend::warp_server::serve_warp());
}
