#![allow(clippy::module_inception)]
pub mod http_1;
pub mod http_2;
mod log_and_launch_browser;
pub mod server;

pub use server::Server;
