use std::fs;

use anyhow::Result;
use clap::Parser;
use inquire::Confirm;
use log::Level;
use resource::resource_str;
use stripmargin::StripMargin;
use tokio::runtime::Builder;

use super::command::{Cli, Command};
use crate::blueprint::Blueprint;
use crate::cli::fmt::Fmt;
use crate::cli::CLIError;
use crate::config::Config;
use crate::http::start_server;
use crate::print_schema;

pub fn run() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Start { file_path, log_level } => {
      env_logger::Builder::new()
        .filter_level(log_level.unwrap_or(Level::Info).to_level_filter())
        .init();
      match parse_blueprint(file_path, false) {
        Ok(blueprint) => {
          let runtime = Builder::new_multi_thread()
            .worker_threads(blueprint.server.worker) // if 0 should be num_cpus::get()
            .enable_all()
            .build()?;
          runtime.block_on(start_server(blueprint))?;
          Ok(())
        }
        Err(e) => Err(e),
      }
    }
    Command::Check { file_path, n_plus_one_queries, schema } => match parse_blueprint(file_path, n_plus_one_queries) {
      Ok(blueprint) => {
        if schema {
          display_schema(&blueprint);
        }
        Ok(())
      }
      Err(e) => Err(e),
    },
    Command::Init { file_path } => Ok(tokio::runtime::Runtime::new()?.block_on(async { init(&file_path).await })?),
  }
}

fn parse_blueprint(file_path: Vec<String>, n_plus_one_queries: bool) -> Result<Blueprint> {
  let config = tokio::runtime::Runtime::new()?.block_on(async { Config::from_file_or_url(file_path.iter()).await })?;
  display_config(&config, n_plus_one_queries);
  Blueprint::try_from(&config).map_err(|e| CLIError::new(&e.to_string()).into())
}

pub async fn init(file_path: &str) -> Result<()> {
  let tailcallrc: resource::Resource<str> = resource_str!("examples/.tailcallrc.graphql");

  let ans = Confirm::new("Do you want to add a file to the project?")
    .with_default(false)
    .prompt();

  match ans {
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
