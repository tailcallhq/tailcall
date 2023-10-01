use std::fs;
use anyhow::Result;
use clap::Parser;
use super::command::{Cli, Command};
use crate::blueprint::Blueprint;
use crate::cli::fmt::Fmt;
use crate::config::Config;
use crate::http::start_server;
use crate::print_schema;

struct StartCommand {
    file_path: String,
}

struct CheckCommand {
    file_path: String,
    n_plus_one_queries: bool,
    schema: bool,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Start { file_path } => {
            let start_command = StartCommand { file_path };
            handle_start_command(start_command).await
        }
        Command::Check { file_path, n_plus_one_queries, schema } => {
            let check_command = CheckCommand { file_path, n_plus_one_queries, schema };
            handle_check_command(check_command)
        }
    }
}

async fn handle_start_command(command: StartCommand) -> Result<()> {
    start_server(&command.file_path).await?;
    Ok(())
}

fn handle_check_command(command: CheckCommand) -> Result<()> {
    let server_sdl = fs::read_to_string(&command.file_path).expect("Failed to read file");
    let config = Config::from_sdl(&server_sdl)?;
    let blueprint = blueprint_from_sdl(&server_sdl)?;
    display_details(&config, blueprint, &command)
}

fn blueprint_from_sdl(sdl: &str) -> Result<Blueprint> {
    let config = Config::from_sdl(sdl)?;
    Ok(Blueprint::try_from(&config)?)
}

fn display_details(config: &Config, blueprint: Blueprint, command: &CheckCommand) -> Result<()> {
    Fmt::display(Fmt::success(&"No errors found".to_string()));
    let seq = vec![Fmt::n_plus_one_data(command.n_plus_one_queries, config)];
    Fmt::display(Fmt::table(seq));

    if command.schema {
        Fmt::display(Fmt::heading(&"GraphQL Schema:\n".to_string()));
        let sdl = blueprint.to_schema(&config.server);
        Fmt::display(print_schema::print_schema(sdl));
    }
    Ok(())
}
