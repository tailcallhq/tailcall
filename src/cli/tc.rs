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
          let query_details =
            QueryDetails { q_details_n_plus_one_queries: n_plus_one_queries, q_details_schema: schema };

          let display_config = DisplayDetailsConfig::new(config, blueprint, query_details);
          display_details(display_config)?; // Changed this line

          Ok(())
        }
        Err(e) => Err(e),
      }
    }
  }
}

pub fn blueprint_from_sdl(sdl: &str) -> Result<Blueprint> {
  let config = Config::from_sdl(sdl)?;
  Ok(Blueprint::try_from(&config)?)
}

pub struct DisplayDetailsConfig {
  config: Config,
  blueprint: Blueprint,
  n_plus_one_queries: bool,
  schema: bool,
}

pub struct QueryDetails {
  q_details_n_plus_one_queries: bool,
  q_details_schema: bool,
}

impl DisplayDetailsConfig {
  pub fn new(config: Config, blueprint: Blueprint, query_details: QueryDetails) -> Self {
    Self {
      config,
      blueprint,
      n_plus_one_queries: query_details.q_details_n_plus_one_queries,
      schema: query_details.q_details_schema,
    }
  }
}

pub fn display_details(display_config: DisplayDetailsConfig) -> Result<()> {
  Fmt::display(Fmt::success(&"No errors found".to_string()));
  let seq: Vec<(String, String)> = vec![Fmt::n_plus_one_data(
    display_config.n_plus_one_queries,
    &display_config.config,
  )];
  Fmt::display(Fmt::table(seq));

  if display_config.schema {
    Fmt::display(Fmt::heading(&"GraphQL Schema:\n".to_string()));
    let sdl = display_config.blueprint.to_schema(&display_config.config.server);
    Fmt::display(print_schema::print_schema(sdl));
  }
  Ok(())
}
