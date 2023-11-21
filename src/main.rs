// Required for the #[global_allocator] proc macro
#![allow(clippy::too_many_arguments)]

use anyhow::Result;
use mimalloc::MiMalloc;
use tailcall::cli::CLIError;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> Result<()> {
  let result = tailcall::cli::run();
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
