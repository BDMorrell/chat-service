use log;
use pretty_env_logger;

pub mod config;

/// Initalize the logging system.
pub fn init_log(is_test: bool) {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Debug)
        .is_test(is_test)
        .init();
}
