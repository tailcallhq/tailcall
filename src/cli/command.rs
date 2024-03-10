use clap::{Parser, Subcommand};

use crate::config::Source;

pub const VERSION: &str = match option_env!("APP_VERSION") {
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
        /// Path for the configuration files or http(s) link to config files
        /// separated by spaces if more than one
        #[arg(required = true)]
        file_paths: Vec<String>,
    },

    /// Validate a composition spec
    Check {
        /// Path for the configuration files separated by spaces if more than
        /// one
        #[arg(required = true)]
        file_paths: Vec<String>,

        /// N plus one queries
        #[arg(short, long)]
        n_plus_one_queries: bool,

        /// Display schema
        #[arg(short, long)]
        schema: bool,

        /// Prints the input config in the provided format.
        #[clap(short, long)]
        format: Option<Source>,
    },

    /// Initialize a new project
    Init {
        // default is current directory
        #[arg(default_value = ".")]
        folder_path: String,
    },
}
