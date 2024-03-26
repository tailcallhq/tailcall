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
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    tracing_subscriber::fmt()
        .with_writer(
            // To avoid trace events in the browser from showing their JS backtrace
            tracing_subscriber_wasm::MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::INFO),
        )
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
}

fn to_anyhow<T: std::fmt::Display>(e: T) -> anyhow::Error {
    anyhow!("{}", e)
}
