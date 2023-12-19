#!/usr/bin/env cargo-script

//! ```cargo
//! [dependencies]
//! prettytable-rs = "^0.10"
//! serde_json = "1.0"
//! serde = { version = "1", features = ["derive"] }
//! ```

use std::{env, fs};

use prettytable::{row, Table};
use serde::Deserialize;
use serde_json::from_str;

#[derive(Deserialize)]
pub struct Benchmark {
  pub id: String,
  pub typical: Metric,
  pub mean: Metric,
  pub median: Metric,
}

#[derive(Deserialize)]
pub struct Metric {
  pub estimate: f64,
  pub lower_bound: f64,
  pub upper_bound: f64,
  pub unit: String,
}

impl Metric {
  fn round(&self) -> Metric {
    Metric {
      estimate: self.estimate.round(),
      lower_bound: self.lower_bound.round(),
      upper_bound: self.upper_bound.round(),
      unit: self.unit.clone(),
    }
  }
}

pub fn parse_json(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
  // Read the JSON file content
  let content = fs::read_to_string(file_path)?;

  // Deserialize JSON content into a vector of Benchmarks
  let benchmarks: Vec<Benchmark> = content.lines().filter_map(|line| from_str(line).ok()).collect();

  // Print benchmarks in a tabular form
  print_table(&benchmarks);

  Ok(())
}

fn print_table(benchmarks: &[Benchmark]) {
  // Create a table
  let mut table = Table::new();

  // Add rows for each benchmark and its metrics
  for benchmark in benchmarks {
    // Round off the numbers
    let rounded_typical = benchmark.typical.round();
    let rounded_mean = benchmark.mean.round();
    let rounded_median = benchmark.median.round();

    // Format the benchmark name
    let benchmark_name = if benchmark.id.starts_with(':') {
      format!(":{}", &benchmark.id[1..])
    } else {
      benchmark.id.clone()
    };

    table.add_row(row!["Benchmark", benchmark_name,]);
    table.add_row(row!["Metric", "estimate", "lower_bound", "upper_bound", "unit",]);
    // Add additional rows for typical, median, and mean metrics
    table.add_row(row![
      "Typical",
      format!("{:.2}", rounded_typical.estimate),
      format!("{:.2}", rounded_typical.lower_bound),
      format!("{:.2}", rounded_typical.upper_bound),
      &rounded_typical.unit,
    ]);
    table.add_row(row![
      "Mean",
      format!("{:.2}", rounded_mean.estimate),
      format!("{:.2}", rounded_mean.lower_bound),
      format!("{:.2}", rounded_mean.upper_bound),
      &rounded_mean.unit,
    ]);
    table.add_row(row![
      "Median",
      format!("{:.2}", rounded_median.estimate),
      format!("{:.2}", rounded_median.lower_bound),
      format!("{:.2}", rounded_median.upper_bound),
      &rounded_median.unit,
    ]);

    // Add a separator row between benchmarks
    table.add_row(row!["", "", "", ""]);
  }

  // Print the table
  table.printstd();
}

fn main() {
  // Retrieve command-line arguments
  let args: Vec<String> = env::args().collect();

  // Check if a file name argument is provided
  if args.len() != 2 {
    eprintln!("Usage: {} <file_path>", args[0]);
    std::process::exit(1);
  }

  // Extract the file name from command-line arguments
  let file_path = &args[1];

  // Attempt to parse the JSON file and print the table
  match parse_json(file_path) {
    Ok(_) => println!("Table printed successfully."),
    Err(err) => eprintln!("Error: {}", err),
  }
}
