use std::{env, fs};

use anyhow::Result;
use clap::Parser;
use env_logger::Env;
use inquire::Confirm;
use resource::resource_str;
use stripmargin::StripMargin;
use tokio::runtime::Builder;

use super::command::{Cli, Command};
use crate::blueprint::Blueprint;
use crate::cli::fmt::Fmt;
use crate::cli::CLIError;
use crate::config::Config;
use crate::http::Server;
use crate::print_schema;

pub fn run() -> Result<()> {
  let cli = Cli::parse();

  logger_init();

  match cli.command {
    Command::Start { file_paths } => {
      let config =
        tokio::runtime::Runtime::new()?.block_on(async { Config::read_from_files(file_paths.iter()).await })?;
      log::info!("N + 1: {}", config.n_plus_one().len().to_string());
      let runtime = Builder::new_multi_thread()
        .worker_threads(config.server.get_workers())
        .enable_all()
        .build()?;
      let server = Server::new(config);
      runtime.block_on(server.start())?;
      Ok(())
    }
    Command::Check { file_path, n_plus_one_queries, schema, out_file_path } => {
      let config =
        tokio::runtime::Runtime::new()?.block_on(async { Config::read_from_files(file_path.iter()).await })?;
      let blueprint = Blueprint::try_from(&config).map_err(CLIError::from);
      match blueprint {
        Ok(blueprint) => {
          display_config(&config, n_plus_one_queries);
          if schema {
            display_schema(&blueprint);
          }
          if let Some(out_file) = out_file_path {
            tokio::runtime::Runtime::new()?.block_on(async { config.write_file(&out_file).await })?;
            Fmt::display(Fmt::success(
              &format!("Schema has been written to {}", out_file).to_string(),
            ));
          }
          Ok(())
        }
        Err(e) => Err(e.into()),
      }
    }
    Command::Init { file_path } => Ok(tokio::runtime::Runtime::new()?.block_on(async { init(&file_path).await })?),
  }
}

pub async fn init(file_path: &str) -> Result<()> {
  let tailcallrc: resource::Resource<str> = resource_str!("examples/.tailcallrc.graphql");

  let answer = Confirm::new("Do you want to add a file to the project?")
    .with_default(false)
    .prompt();

  match answer {
    Ok(true) => {
      let file_name = inquire::Text::new("Enter the file name:")
        .with_default(".graphql")
        .prompt()
        .unwrap_or_else(|_| String::from(".graphql"));

      let file_name = format!("{}.graphql", file_name.strip_suffix(".graphql").unwrap_or(&file_name));

      let confirm = Confirm::new(&format!("Do you want to create the file {}?", file_name))
        .with_default(false)
        .prompt();

      match confirm {
        Ok(true) => {
          fs::write(format!("{}/{}", file_path, &file_name), "")?;

          let graphqlrc = format!(
            r#"|schema:
               |- './{}'
               |- './.tailcallrc.graphql'
          "#,
            &file_name
          )
          .strip_margin();
          fs::write(format!("{}/.graphqlrc.yml", file_path), graphqlrc)?;
        }
        Ok(false) => (),
        Err(e) => return Err(e.into()),
      }
    }
    Ok(false) => (),
    Err(e) => return Err(e.into()),
  }

  fs::write(
    format!("{}/.tailcallrc.graphql", file_path),
    tailcallrc.as_ref().as_bytes(),
  )?;
  Ok(())
}

pub fn display_schema(blueprint: &Blueprint) {
  Fmt::display(Fmt::heading(&"GraphQL Schema:\n".to_string()));
  let sdl = blueprint.to_schema();
  Fmt::display(print_schema::print_schema(sdl));
}

fn display_config(config: &Config, n_plus_one_queries: bool) {
  Fmt::display(Fmt::success(&"No errors found".to_string()));
  let seq = vec![Fmt::n_plus_one_data(n_plus_one_queries, config)];
  Fmt::display(Fmt::table(seq));
}

// initialize logger
fn logger_init() {
  // set the log level
  const LONG_ENV_FILTER_VAR_NAME: &str = "TAILCALL_LOG_LEVEL";
  const SHORT_ENV_FILTER_VAR_NAME: &str = "TC_LOG_LEVEL";

  // Select which env variable to use for the log level filter. This is because filter_or doesn't allow picking between multiple env_var for the filter value
  let filter_env_name = env::var(LONG_ENV_FILTER_VAR_NAME)
    .map(|_| LONG_ENV_FILTER_VAR_NAME)
    .unwrap_or_else(|_| SHORT_ENV_FILTER_VAR_NAME);

  // use the log level from the env if there is one, otherwise use the default.
  let env = Env::new().filter_or(filter_env_name, "info");

  env_logger::Builder::from_env(env).init();
}
