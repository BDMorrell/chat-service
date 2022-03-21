use tokio::runtime;

fn main() {
    chat_backend::init_log(false);
    let configuration = chat_backend::config::get_configuration_from_current_directory();
    runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(chat_backend::warp_server::serve_warp());
}
