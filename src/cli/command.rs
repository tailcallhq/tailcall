use clap::{Parser, Subcommand};
use strum_macros::Display;
use tailcall_version::VERSION;

use crate::core::{config, generator};

const ABOUT: &str = r"
   __        _ __           ____
  / /_____ _(_) /________ _/ / /
 / __/ __ `/ / / ___/ __ `/ / /
/ /_/ /_/ / / / /__/ /_/ / / /
\__/\__,_/_/_/\___/\__,_/_/_/";

#[derive(Parser)]
#[command(name = "tailcall", author, version = VERSION.as_str(), about, long_about = Some(ABOUT))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Display)]
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
        format: Option<config::Source>,
    },

    /// Initialize a new project
    Init {
        // default is current directory
        #[arg(default_value = ".")]
        folder_path: String,
    },

    /// Generates a Tailcall Configuration from one or more source files.
    Gen {
        /// Path of the source files separated by spaces if more than one
        #[arg(required = true)]
        paths: Vec<String>,

        /// Format of the input file
        #[clap(short, long)]
        input: generator::Source,

        /// Format of the output file
        #[clap(short, long)]
        output: Option<config::Source>,

        /// Root query name
        #[arg(default_value = "Query")]
        #[clap(short, long)]
        query: String,
    },
}
