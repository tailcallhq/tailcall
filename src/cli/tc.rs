#![allow(clippy::too_many_arguments)]

use std::fs;
use std::io::{self};

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
        Err(e) => Err(e),
      }
    }
    Command::Init => init().await,
  }
}

pub async fn init() -> Result<()> {
  let tailcallrc = r"directive @server(
    allowedHeaders: [String]
    baseURL: String
    enableApolloTracing: Boolean
    enableCacheControlHeader: Boolean
    enableGraphiql: String
    enableHttpCache: Boolean
    enableIntrospection: Boolean
    enableQueryValidation: Boolean
    enableResponseValidation: Boolean
    globalResponseTimeout: Int
    port: Int
    proxy: Proxy
    vars: [KeyValue]
  ) on SCHEMA
  directive @http(
    path: String!
    method: Method = GET
    query: [KeyValue]
    body: String
    baseURL: String
    headers: [KeyValue]
  ) on FIELD_DEFINITION
  directive @inline(path: [String]!) on FIELD_DEFINITION
  directive @modify(omit: Boolean, name: String) on FIELD_DEFINITION
  directive @batch(path: [String]!, key: String!) on FIELD_DEFINITION
  
  enum Method {
    GET
    POST
    PUT
    DELETE
    PATCH
    HEAD
    OPTIONS
  }
  
  input Proxy {
    url: String
  }
  
  input KeyValue {
    key: String!
    value: String!
  }";

  fs::write(".tailcallrc.graphql", tailcallrc)?;

  println!("Do you want to add a file to the project? (yes/no)");
  let mut input = String::new();
  io::stdin().read_line(&mut input)?;

  if input.trim() == "yes" {
    println!("Enter the file name:");
    let mut file_name = String::new();
    io::stdin().read_line(&mut file_name)?;
    file_name = file_name.trim().to_string();
    fs::write(&file_name, "")?;

    let graphqlrc = format!(
      r#"schema:
- "./{}.graphql"
- "./.tailcallrc.graphql"#,
      file_name
    );
    fs::write(".graphqlrc.yml", graphqlrc)?;
  }

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
