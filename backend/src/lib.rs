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
