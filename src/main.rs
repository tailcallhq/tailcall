// Required for the #[global_allocator] proc macro
#![allow(clippy::too_many_arguments)]

use mimalloc::MiMalloc;
use reqwest::{Request, Response};
use tailcall::{cli::CLIError, Engine, ScriptRequestContext, ScriptServerContext};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn run_blocking() -> anyhow::Result<()> {
  let rt = tokio::runtime::Runtime::new()?;
  rt.block_on(async { tailcall::cli::run().await })
}

fn main() -> anyhow::Result<()> {
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

// struct MiniV8 {}
// impl MiniV8 {
//   fn new() -> Self {
//     todo!()
//   }
// }

// impl ScriptServerContext for MiniV8 {
//   fn new_request_context(&self) -> anyhow::Result<impl tailcall::ScriptRequestContext> {
//     Ok(V8ScriptRequestContext {})
//   }
// }

// struct V8ScriptRequestContext {}
// impl ScriptRequestContext for V8ScriptRequestContext {
//   type Event = Event;

//   type Command = Command;

//   #[must_use]
//   #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
//   fn execute<'life0, 'async_trait>(
//     &'life0 self,
//     event: Self::Event,
//   ) -> ::core::pin::Pin<
//     Box<dyn ::core::future::Future<Output = anyhow::Result<Self::Command>> + ::core::marker::Send + 'async_trait>,
//   >
//   where
//     'life0: 'async_trait,
//     Self: 'async_trait,
//   {
//     todo!()
//   }
// }

// impl Engine for MiniV8 {
//   async fn load<'a>(&'a self, script: &'a str) -> anyhow::Result<impl ScriptServerContext> {
//     Ok(V8ScriptRequestContext {})
//   }
// }

// enum Event {
//   Empty,
//   Request(Request),
//   Response(Response),
// }

// enum Command {
//   Request(Vec<Request>),
//   Response(Response),
// }

// async fn test_execution() -> anyhow::Result<()> {
//   let script = "2 + 2";

//   // Done once at server start time.
//   let v8 = MiniV8::new();
//   let server_context = v8.load(script).await?;

//   // Create request context
//   let request_context = server_context.new_request_context()?;

//   let event = Event::Empty;
//   let _ = request_context.execute(event).await?;

//   Ok(())
// }
