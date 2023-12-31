use clap::{Parser, Subcommand};

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

#[derive(Clone)]
pub enum FormatOption {
  Yml,
  Gql,
  Json,
}

impl std::str::FromStr for FormatOption {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "json" => Ok(FormatOption::Json),
      "yml" | "yaml" => Ok(FormatOption::Yml),
      "graphql" | "gql" => Ok(FormatOption::Gql),
      _ => Err(format!("Unsupported format: {}", s)),
    }
  }
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

    /// Display the generated blueprint on the console
    #[arg(short, long)]
    blueprint: bool,
  },

  /// Merge multiple configuration file into one
  Compose {
    /// Path for the configuration files separated by spaces if more than one
    #[arg(required = true)]
    file_path: Vec<String>,

    /// Format of the result. Accepted values: JSON|YML|GQL.
    #[clap(short, long, default_value = "gql")]
    format: FormatOption,
  },

  /// Initialize a new project
  Init { file_path: String },
}
