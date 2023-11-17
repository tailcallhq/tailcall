use std::fs;

use anyhow::Result;
use clap::Parser;
// use inquire::Confirm;
use log::Level;
use resource::resource_str;
use stripmargin::StripMargin;

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
      let config = Config::from_file_or_url(file_path.iter()).await?;
      log::info!("N + 1: {}", config.n_plus_one().len().to_string());
      start_server(config).await?;
      Ok(())
    }
    Command::Check { file_path, n_plus_one_queries, schema } => {
      let config = Config::from_file_or_url(file_path.iter()).await?;
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
    Command::Init { file_path } => Ok(init(&file_path).await?),
  }
}

pub async fn init(file_path: &str) -> Result<()> {
  let tailcallrc: resource::Resource<str> = resource_str!("examples/.tailcallrc.graphql");
  let ans = dialoguer::Confirm::new()
    .with_prompt("Do you want to add a file to the project?")
    .wait_for_newline(true)
    .interact()
    .unwrap();

  if ans {
    let file_name = inquire::Text::new("Enter the file name:")
      .with_default(".graphql")
      .prompt()
      .unwrap_or_else(|_| String::from(".graphql"));
    // let file_name = dialoguer::Input::new()
//       .with_initial_text(".graphql")
//       .with_prompt("Enter the file name:")
//       .interact()
//       .unwrap_or_else(|_| String::from(".graphql"));

    let file_name = format!("{}.graphql", file_name.strip_suffix(".graphql").unwrap_or(&file_name));

    let confirm = inquire::Confirm::new(&format!("Do you want to create the file {}?", file_name))
      .with_default(false)
      .prompt();
//     let confirm = dialoguer::Confirm::new()
//       .with_prompt(&format!("Do you want to create the file {}?", file_name))
//       .interact()
//       .unwrap();

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
    

  fs::write(
    format!("{}/.tailcallrc.graphql", file_path),
    tailcallrc.as_ref().as_bytes(),
  )?;
  Ok(())
}

// pub async fn init(file_path: &str) -> Result<()> {
//   let tailcallrc: resource::Resource<str> = resource_str!("examples/.tailcallrc.graphql");

//   let ans = Confirm::new("Do you want to add a file to the project?")
//     .with_default(false)
//     .prompt();

//   match ans {
//     Ok(true) => {
//       let file_name = inquire::Text::new("Enter the file name:")
//         .with_default(".graphql")
//         .prompt()
//         .unwrap_or_else(|_| String::from(".graphql"));

//       let file_name = format!("{}.graphql", file_name.strip_suffix(".graphql").unwrap_or(&file_name));

//       let confirm = Confirm::new(&format!("Do you want to create the file {}?", file_name))
//         .with_default(false)
//         .prompt();

//       match confirm {
//         Ok(true) => {
//           fs::write(format!("{}/{}", file_path, &file_name), "")?;

//           let graphqlrc = format!(
//             r#"|schema:
//                |- './{}'
//                |- './.tailcallrc.graphql'
//           "#,
//             &file_name
//           )
//           .strip_margin();
//           fs::write(format!("{}/.graphqlrc.yml", file_path), graphqlrc)?;
//         }
//         Ok(false) => (),
//         Err(e) => return Err(e.into()),
//       }
//     }
//     Ok(false) => (),
//     Err(e) => return Err(e.into()),
//   }

//   fs::write(
//     format!("{}/.tailcallrc.graphql", file_path),
//     tailcallrc.as_ref().as_bytes(),
//   )?;
//   Ok(())
// }

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
