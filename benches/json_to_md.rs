#!/usr/bin/env cargo-script

//! ```cargo
//! [dependencies]
//! serde_json = "1.0"
//! serde = { version = "1", features = ["derive"] }
//! ```

use std::{env, fs};

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

fn format_value(value: f64, unit: &str) -> String {
  match unit {
    "ns" if value >= 1000.0 => format!("{:.2} μs", value / 1000.0),
    "μs" if value >= 1000.0 => format!("{:.2} ms", value / 1000.0),
    "ms" if value >= 1000.0 => format!("{:.2} s", value / 1000.0),
    _ => format!("{:.2} {}", value, unit),
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
  // Create a table header
  let mut markdown_table = String::new();
  markdown_table.push_str("| Benchmark | Estimate | Lower Bound | Upper Bound |\n");
  markdown_table.push_str("| --- | --- | --- | --- |\n");

  // Add rows for each benchmark and its typical metric
  for benchmark in benchmarks {
    // Round off the numbers for the typical metric
    let rounded_typical = benchmark.typical.round();

    // Format the benchmark name
    let benchmark_name = if benchmark.id.starts_with(':') {
      format!(":{}", &benchmark.id[1..])
    } else {
      benchmark.id.clone()
    };

    // Format the typical metric values using the format_value function
    let formatted_estimate = format_value(rounded_typical.estimate, &rounded_typical.unit);
    let formatted_lower_bound = format_value(rounded_typical.lower_bound, &rounded_typical.unit);
    let formatted_upper_bound = format_value(rounded_typical.upper_bound, &rounded_typical.unit);

    // Add row for typical metric without the unit column
    markdown_table.push_str(&format!(
      "| {} | {} | {} | {} |\n",
      benchmark_name, formatted_estimate, formatted_lower_bound, formatted_upper_bound,
    ));
  }

  // Get the output file path from the command-line arguments
  let args: Vec<String> = env::args().collect();
  if args.len() != 3 {
    eprintln!("Usage: {} <input_file_path> <output_file_path>", args[0]);
    std::process::exit(1);
  }
  let _output_file_path = &args[2];

  // Write the Markdown table to the file
  fs::write(_output_file_path, markdown_table).expect("Failed to write Markdown table to file");
  println!("Markdown table (Typical values) written to {}", _output_file_path);
}

fn main() {
  // Retrieve command-line arguments
  let args: Vec<String> = env::args().collect();

  // Check if two file name arguments are provided
  if args.len() != 3 {
    eprintln!("Usage: {} <input_file_path> <output_file_path>", args[0]);
    std::process::exit(1);
  }

  // Extract the input and output file names from command-line arguments
  let input_file_path = &args[1];

  // Attempt to parse the JSON file and print the table
  match parse_json(input_file_path) {
    Ok(_) => println!("Table printed successfully."),
    Err(err) => {
      eprintln!("Error: {}", err);
      std::process::exit(1);
    }
  }
}
