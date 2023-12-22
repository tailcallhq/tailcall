use clap::{Parser, Subcommand};

use crate::config::Source;

const VERSION: &str = match option_env!("APP_VERSION") {
  Some(version) => version,
  _ => "0.1.0-dev",
};
const ABOUT: &str = r"
   __        _ __           ____
  / /_____ _(_) /________ _/ / /
 / __/ __ `/ / / ___/ __ `/ / /
/ /_/ /_/ / / / /__/ /_/ / / /
\__/\__,_/_/_/\___/\__,_/_/_/";

#[derive(Parser)]
#[command(name ="tailcall",author, version = VERSION, about, long_about = Some(ABOUT))]
pub struct Cli {
  #[command(subcommand)]
  pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
  /// Starts the GraphQL server on the configured port
  Start {
    /// Path for the configuration files or http(s) link to config files separated by spaces if more than one
    #[arg(required = true)]
    file_paths: Vec<String>,
  },

  /// Validate a composition spec
  Check {
    /// Path for the configuration files separated by spaces if more than one
    #[arg(required = true)]
    file_path: Vec<String>,

    /// N plus one queries
    #[arg(short, long)]
    n_plus_one_queries: bool,

    /// Display schema
    #[arg(short, long)]
    schema: bool,

    /// Path of the file to save final configuration in
    #[arg(short, long("out"))]
    out_file_path: Option<String>,

    /// Operations to check
    #[arg(short('p'), long, value_delimiter=',', num_args = 1..)]
    operations: Vec<String>,
  },

  /// Merge multiple configuration file into one
  Compose {
    /// Path for the configuration files separated by spaces if more than one
    #[arg(required = true)]
    file_path: Vec<String>,

    /// Format of the result. Accepted values: JSON|YML|GQL.
    #[clap(short, long, default_value = "gql")]
    format: Source,
  },

  /// Initialize a new project
  Init {
    // default is current directory
    #[arg(default_value = ".")]
    folder_path: String,
  },
}
