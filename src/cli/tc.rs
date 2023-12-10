use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use inquire::Confirm;
use log::Level;
use resource::resource_str;
use tokio::runtime::Builder;

use super::command::{Cli, Command};
use crate::blueprint::Blueprint;
use crate::cli::fmt::Fmt;
use crate::config::Config;
use crate::http::Server;
use crate::print_schema;

pub fn run() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Start { file_path, log_level } => {
      env_logger::Builder::new()
        .filter_level(log_level.unwrap_or(Level::Info).to_level_filter())
        .init();
      let config =
        tokio::runtime::Runtime::new()?.block_on(async { Config::from_file_or_url(file_path.iter()).await })?;
      log::info!("N + 1: {}", config.n_plus_one().len().to_string());
      let runtime = Builder::new_multi_thread()
        .worker_threads(config.server.get_workers())
        .enable_all()
        .build()?;
      let server = Server::new(config);
      runtime.block_on(server.start())?;
      Ok(())
    }
    Command::Check { file_path, n_plus_one_queries, schema } => {
      let config =
        tokio::runtime::Runtime::new()?.block_on(async { Config::from_file_or_url(file_path.iter()).await })?;
      let blueprint = Blueprint::try_from(&config);
      match blueprint {
        Ok(blueprint) => {
          display_config(&config, n_plus_one_queries);
          if schema {
            display_schema(&blueprint);
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
  let tailcallrc_path = Path::new(file_path).join(".tailcallrc.graphql");

  if let Some(parent) = tailcallrc_path.parent() {
    fs::create_dir_all(parent)?;
  }

  if tailcallrc_path.exists() {
    let overwrite = Confirm::new("A .tailcallrc.graphql file already exists. Do you want to overwrite it?")
      .with_default(false)
      .prompt()?;

    if !overwrite {
      return Ok(());
    }
  } else {
    println!(
      "{}",
      format!("Created .tailcallrc.graphql file in {}", file_path).blue()
    );
  }

  fs::write(tailcallrc_path, tailcallrc.as_ref().as_bytes())?;
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
