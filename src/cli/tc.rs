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
use crate::http::Server;
use crate::print_schema;

pub fn run() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Command::Start { file_paths, log_level } => {
      env_logger::Builder::new()
        .filter_level(log_level.unwrap_or(Level::Info).to_level_filter())
        .init();
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
  let folder_exists = fs::metadata(file_path).is_ok();

  if !folder_exists {
    let confirm = Confirm::new(&format!("Do you want to create the folder {}?", file_path))
      .with_default(false)
      .prompt();

    match confirm {
      Ok(true) => fs::create_dir(file_path)?,
      Ok(false) => (),
      Err(e) => return Err(e.into()),
    };
  }

  let tailcallrc: resource::Resource<str> = resource_str!("examples/.tailcallrc.graphql");

  let file_name = ".tailcallrc.graphql";
  let yml_file_name = ".graphqlrc.yml";
  let yml_exists = fs::metadata(format!("{}/{}", file_path, yml_file_name)).is_ok();

  if yml_exists {
    // what should we do in this case?
  } else {
    fs::write(format!("{}/{}", file_path, yml_file_name), "")?;

    let graphqlrc = 
      r#"|schema:
         |- './.tailcallrc.graphql'
    "#
    .strip_margin();

    fs::write(format!("{}/.graphqlrc.yml", file_path), graphqlrc)?;
  }

  let tailcall_exists = fs::metadata(format!("{}/{}", file_path, file_name)).is_ok();

  if tailcall_exists {
    // confirm overwrite
    let confirm = Confirm::new(&format!("Do you want to overwrite the file {}?", file_name))
      .with_default(false)
      .prompt();

    match confirm {
      Ok(true) => fs::write(format!("{}/{}", file_path, file_name), tailcallrc.as_ref().as_bytes())?,
      Ok(false) => (),
      Err(e) => return Err(e.into()),
    };
  } else {
    fs::write(format!("{}/{}", file_path, file_name), tailcallrc.as_ref().as_bytes())?;
  }

  let graphqlrc_path = format!("{}/.graphqlrc.yml", file_path);
  let graphqlrc = fs::read_to_string(&graphqlrc_path)?;

  if !graphqlrc.contains(file_name) {
    let confirm = Confirm::new(&format!("Do you want to add {} to the schema?", file_name))
      .with_default(false)
      .prompt();

    match confirm {
      Ok(true) => {
        // add - './.tailcallrc.graphql' on line below schema keyword
        // optionally, file can be "schema: 'some-schema.graphql'"
        // in this case, we need to keep the first line as it is and add the second line
        let mut lines = graphqlrc.lines();
        let index = lines.position(|line| line.contains("schema:"));

        println!("lines: {:?}", lines);

        let mut updated = String::new();

        if let Some(index) = index {
          println!("here!");
          let mut schema_line = lines.nth(index).unwrap().to_string();
          println!("schema_line: {}", schema_line);
          schema_line.push_str("\n  - './.tailcallrc.graphql'");
          updated = graphqlrc.replace("schema:", &schema_line);
        } else {
          updated = graphqlrc.replace("schema:", "schema:\n  - './.tailcallrc.graphql'");
        }

        fs::write(graphqlrc_path, updated)?;
      },
      Ok(false) => (),
      Err(e) => return Err(e.into()),
    }
  }

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
