use colored::*;

use crate::core::config::{Config, QueryPath};

pub struct Fmt {}

impl Fmt {
    pub fn heading(heading: &str) -> String {
        format!("{}", heading.bold())
    }

    pub fn meta(meta: &str) -> String {
        format!("{}", meta.yellow())
    }

    pub fn display(s: String) {
        println!("{}", s);
    }

    pub fn format_n_plus_one_queries(n_plus_one_info: QueryPath) -> String {
        Fmt::meta(&n_plus_one_info.to_string())
    }

    pub fn log_n_plus_one(show_npo: bool, config: &Config) {
        let n_plus_one_info = config.n_plus_one();
        let mut message = format!("N + 1 detected: {}", n_plus_one_info.size());

        if show_npo {
            message.push('\n');
            message.push_str(&Fmt::format_n_plus_one_queries(n_plus_one_info));
        }

        tracing::info!("{}", message);
    }
}
