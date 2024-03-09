// Required for the #[global_allocator] proc macro
#![allow(clippy::too_many_arguments)]

use std::cell::Cell;

use mimalloc::MiMalloc;
use tailcall::cli::CLIError;
use tailcall::tracing::default_tailcall_tracing;
use tracing::subscriber::DefaultGuard;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

thread_local! {
    static TRACING_GUARD: Cell<Option<DefaultGuard>> = const { Cell::new(None) };
}

fn run_blocking() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .on_thread_start(|| {
            // initialize default tracing setup for the cli execution for every thread that
            // is spawned based on https://github.com/tokio-rs/tracing/issues/593#issuecomment-589857097
            // and required due to the fact that later for tracing the global subscriber
            // will be set by `src/cli/opentelemetry.rs` and until that we need
            // to use the default tracing configuration for cli output. And
            // since `set_default` works only for current thread incorporate this
            // with tokio runtime
            let guard = tracing::subscriber::set_default(default_tailcall_tracing());

            TRACING_GUARD.set(Some(guard));
        })
        .on_thread_stop(|| {
            TRACING_GUARD.take();
        })
        .build()?;
    rt.block_on(async { tailcall::cli::run().await })
}

fn main() -> anyhow::Result<()> {
    // enable tracing subscriber for current thread until this block ends
    // that will show any logs from cli itself to the user
    // despite of @telemetry settings that
    let _guard = tracing::subscriber::set_default(default_tailcall_tracing());
    let result = run_blocking();
    match result {
        Ok(_) => {}
        Err(error) => {
            // Ensure all errors are converted to CLIErrors before being printed.
            let cli_error = match error.downcast::<CLIError>() {
                Ok(cli_error) => cli_error,
                Err(error) => {
                    let sources = error
                        .source()
                        .map(|error| vec![CLIError::new(error.to_string().as_str())])
                        .unwrap_or_default();

                    CLIError::new(&error.to_string()).caused_by(sources)
                }
            };
            tracing::error!("{}", cli_error.color(true));
            std::process::exit(exitcode::CONFIG);
        }
    }
    Ok(())
}
