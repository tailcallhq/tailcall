// Required for the #[global_allocator] proc macro
#![allow(clippy::too_many_arguments)]

use mimalloc::MiMalloc;
use tailcall::cli::CLIError;
use tracing_subscriber::Registry;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn run_blocking() -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { tailcall::cli::run().await })
}

use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;

fn main() -> anyhow::Result<()> {
    let subscriber = Registry::default()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber).unwrap();

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
            eprintln!("{}", cli_error.color(true));
            std::process::exit(exitcode::CONFIG);
        }
    }
    Ok(())
}
