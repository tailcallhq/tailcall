use std::panic;

use anyhow::anyhow;

mod cache;
mod env;
mod file;
pub mod handle;
mod http;
mod runtime;

#[worker::event(fetch)]
async fn fetch(
    req: worker::Request,
    env: worker::Env,
    ctx: worker::Context,
) -> anyhow::Result<worker::Response> {
    let result = handle::fetch(req, env, ctx).await;

    match result {
        Ok(response) => Ok(response),
        Err(message) => {
            log::error!("ServerError: {}", message.to_string());
            worker::Response::error(message.to_string(), 500).map_err(to_anyhow)
        }
    }
}

#[worker::event(start)]
fn start() {
    // Initialize Logger
    wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
    anyhow!("{}", e)
}
