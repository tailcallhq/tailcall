#![allow(clippy::too_many_arguments)]

use std::fs;

use anyhow::Result;
use clap::Parser;

use super::command::{Cli, Command};
use crate::blueprint::Blueprint;
use crate::cli::fmt::Fmt;
use crate::config::Config;
use crate::http::start_server;
use crate::print_schema;

pub async fn run() -> Result<()> {
  let cli = Cli::parse();
  match cli.command {
    Command::Start { file_path } => {
      start_server(&file_path).await?;
      Ok(())
    }
    Command::Check { file_path, n_plus_one_queries, schema } => {
      let server_sdl = fs::read_to_string(file_path).expect("Failed to read file");
      let config = Config::from_sdl(&server_sdl)?;
      let blueprint = blueprint_from_sdl(&server_sdl);
      match blueprint {
        Ok(blueprint) => {
          display_details(&config, blueprint, &n_plus_one_queries, &schema)?;
          Ok(())
        }
        Err(e) => {
          let err_str = format!("{:?}", e);
          let formatted_err = Fmt::error(&err_str);
          Fmt::display(formatted_err);
          std::process::exit(exitcode::CONFIG);
        }
      }
    }
  }
}

pub fn blueprint_from_sdl(sdl: &str) -> Result<Blueprint> {
  let config = Config::from_sdl(sdl)?;
  Ok(Blueprint::try_from(&config)?)
}

pub fn display_details(config: &Config, blueprint: Blueprint, n_plus_one_queries: &bool, schema: &bool) -> Result<()> {
  Fmt::display(Fmt::success(&"No errors found".to_string()));
  let seq = vec![Fmt::n_plus_one_data(*n_plus_one_queries, config)];
  Fmt::display(Fmt::table(seq));

  if *schema {
    Fmt::display(Fmt::heading(&"GraphQL Schema:\n".to_string()));
    let sdl = blueprint.to_schema(&config.server);
    Fmt::display(print_schema::print_schema(sdl));
  }
  Ok(())
}
