use colored::*;

use crate::core::config::{Config, QueryPath};

pub struct Fmt<Out> {
    out: Out,
}

impl<Out: std::io::Write> Fmt<Out> {
    pub fn new(out: Out) -> Self {
        Self { out }
    }
    pub fn heading(&mut self, heading: &str) -> std::io::Result<()> {
        let heading = format!("{}", heading.bold());
        self.append(&heading)
    }

    fn meta(&mut self, meta: &String) -> std::io::Result<()> {
        let val = format!("{}", meta.yellow());
        self.append(&val)
    }

    pub fn append(&mut self, s: &str) -> std::io::Result<()> {
        writeln!(self.out, "{}\n", s)
    }

    /// This function is usually used with BufWriter
    /// It flushes the output and returns the lock
    pub fn display(mut self) -> std::io::Result<Out> {
        self.out.flush()?;
        Ok(self.out)
    }

    /// This function is usually used in replacement of println!()
    /// It flushes the output and drops the lock
    pub fn display_and_drop(mut self) -> std::io::Result<()> {
        self.out.flush()?;
        Ok(())
    }

    fn format_n_plus_one_queries(&mut self, n_plus_one_info: QueryPath) -> std::io::Result<()> {
        self.meta(&n_plus_one_info.to_string())
    }

    pub fn log_n_plus_one(&mut self, show_npo: bool, config: &Config) -> std::io::Result<()> {
        let n_plus_one_info = config.n_plus_one();
        self.append(format!("N + 1 detected: {}", n_plus_one_info.size()).as_str())?;

        if show_npo {
            self.format_n_plus_one_queries(n_plus_one_info)?;
        }
        Ok(())
    }
}
