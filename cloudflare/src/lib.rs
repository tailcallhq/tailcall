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
            tracing::error!("ServerError: {}", message.to_string());
            worker::Response::error(message.to_string(), 500).map_err(to_anyhow)
        }
    }
}

#[worker::event(start)]
fn start() {
    // Initialize Logger
    let config = tracing_wasm::WASMLayerConfigBuilder::new()
        .set_max_level(tracing::Level::INFO)
        .build();
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    tracing_wasm::set_as_global_default_with_config(config)
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
    anyhow!("{}", e)
}
