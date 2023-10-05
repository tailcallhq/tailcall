#![allow(clippy::too_many_arguments)]

use std::fs;
use std::io::{self};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use inquire::Confirm;
use resource::resource_str;

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
        Err(e) => Err(e),
      }
    }
    Command::Init { file_path } => Ok(init(&file_path).await?),
  }
}

pub async fn init(file_path: &str) -> Result<()> {
  let tailcallrc = resource_str!("assets/.tailcallrc.graphql");

  let ans = Confirm::new("Do you want to add a file to the project?")
    .with_default(false)
    .prompt();

  match ans {
    Ok(true) => {
      println!("{}", "Enter the file name:".yellow());
      let mut file_name = String::new();
      io::stdin().read_line(&mut file_name)?;
      file_name = format!("{}.graphql", file_name.trim());

      let confirm = Confirm::new(&format!("Do you want to create the file {}?", file_name))
        .with_default(false)
        .prompt();

      match confirm {
        Ok(true) => {
          fs::write(format!("{}/{}", file_path, &file_name), "")?;

          let graphqlrc = format!(
            r#"schema:
- "./{}"
- "./.tailcallrc.graphql"#,
            file_name
          );
          fs::write(format!("{}/.graphqlrc.yml", file_path), graphqlrc)?;
        }
        Ok(false) => (),
        Err(_) => (),
      }
    }
    Ok(false) => (),
    Err(_) => (),
  }

  fs::write(
    format!("{}/.tailcallrc.graphql", file_path),
    tailcallrc.as_ref().as_bytes(),
  )?;
  Ok(())
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
