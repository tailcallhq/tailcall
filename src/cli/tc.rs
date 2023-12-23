use std::fs;
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use inquire::Confirm;
use log::Level;
use stripmargin::StripMargin;
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
    Command::Start { file_paths, log_level } => {
      env_logger::Builder::new()
        .filter_level(log_level.unwrap_or(Level::Info).to_level_filter())
        .init();
      let config = tokio::runtime::Runtime::new()?.block_on(async { Config::from_iter(file_paths.iter()).await })?;
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
      let config = tokio::runtime::Runtime::new()?.block_on(async { Config::from_iter(file_path.iter()).await })?;
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
  let tailcall_rc = ".tailcallrc.graphql";
  let graphql_rc = ".graphqlrc.yml";  
  let tailcallrc_data = include_str!("../../examples/.tailcallrc.graphql");
  let graphql_rc_exists = fs::metadata(format!("{}/{}", file_path, graphql_rc)).is_ok();
  let path = Path::new(file_path).join(tailcall_rc);

  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }

  if !graphql_rc_exists {
    fs::write(format!("{}/{}", file_path, graphql_rc), "")?;

    let graphqlrc = r#"|schema:
         |- './.tailcallrc.graphql'
    "#
    .strip_margin();

    fs::write(format!("{}/.graphqlrc.yml", file_path), graphqlrc)?;
    Fmt::display(Fmt::success(&format!("Created file .graphqlrc.yml in {}", file_path)));
  }

  let tailcall_exists = fs::metadata(format!("{}/{}", file_path, tailcall_rc)).is_ok();

  if tailcall_exists {
    let confirm = Confirm::new(&format!("Do you want to overwrite the file {}?", tailcall_rc))
      .with_default(false)
      .prompt();

    match confirm {
      Ok(true) => fs::write(format!("{}/{}", file_path, tailcall_rc), tailcallrc_data)?,
      Ok(false) => (),
      Err(e) => return Err(e.into()),
    };
  } else {
    fs::write(format!("{}/{}", file_path, tailcall_rc), tailcallrc_data)?;
    Fmt::display(Fmt::success(&format!("Created file .tailcallrc.graphql in {}", file_path)));
  }

  let graphqlrc_path = format!("{}/.graphqlrc.yml", file_path);
  let graphqlrc = fs::read_to_string(&graphqlrc_path)?;

  if !graphqlrc.contains(tailcall_rc) {
    let confirm = Confirm::new(&format!("Do you want to add {} to the schema?", tailcall_rc))
      .with_default(false)
      .prompt();

    match confirm {
      Ok(true) => {
        let mut schema_line = graphqlrc
          .lines()
          .find(|line| line.contains("schema:"))
          .unwrap()
          .to_string();

        schema_line.push_str("\n  - './.tailcallrc.graphql'");

        let updated = graphqlrc.replace("schema:", &schema_line);

        fs::write(graphqlrc_path, updated)?;
      }
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
