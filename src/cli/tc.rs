#![allow(clippy::too_many_arguments)]

use std::fs;

use anyhow::Result;
use async_graphql::futures_util::future::join_all;
use clap::Parser;
use inquire::Confirm;
use log::Level;
use resource::resource_str;
use stripmargin::StripMargin;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use super::command::{Cli, Command};
use crate::blueprint::Blueprint;
use crate::cli::fmt::Fmt;
use crate::config::Config;
use crate::http::start_server;
use crate::print_schema;

pub async fn run() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Start { file_path, log_level } => {
      env_logger::Builder::new()
        .filter_level(log_level.unwrap_or(Level::Info).to_level_filter())
        .init();
      let config = from_files(&file_path).await?;
      start_server(config).await?;
      Ok(())
    }
    Command::Check { file_path, n_plus_one_queries, schema } => {
      let config = from_files(&file_path).await?;
      let blueprint = Ok(Blueprint::try_from(&config)?);
      match blueprint {
        Ok(blueprint) => {
          display_details(&config, blueprint, &n_plus_one_queries, &schema)?;
          Ok(())
        }
        Err(e) => Err(e),
      }
    }
    Command::Init { file_path } => Ok(init(&file_path).await?),
  }
}

async fn from_files(file_paths: &Vec<String>) -> Result<Config> {
  let mut config = Config::default();
  let futures: Vec<_> = file_paths
    .iter()
    .map(|file_path| async move {
      let mut f = File::open(file_path).await?;
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer).await?;

      let server_sdl = String::from_utf8(buffer)?;
      Ok(Config::from_sdl(&server_sdl)?)
    })
    .collect();

  for res in join_all(futures).await {
    match res {
      Ok(conf) => config = config.clone().merge_right(&conf),
      Err(e) => return Err(e), // handle error
    }
  }

  Ok(config)
}

pub async fn init(file_path: &str) -> Result<()> {
  let tailcallrc = resource_str!("examples/.tailcallrc.graphql");

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
