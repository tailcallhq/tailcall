use anyhow::Result;
use mimalloc::MiMalloc;
use tailcall::cli::CLIError;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<()> {
    let result = tailcall::cli::run().await;
    match result {
        Ok(_) => {}
        Err(error) => {
            // Ensure all errors are converted to CLIErrors before being printed.
            let cli_error = error.downcast::<CLIError>()?;
            eprintln!("{}", cli_error.color(true));
            std::process::exit(1);
        }
    }
    Ok(())
}
