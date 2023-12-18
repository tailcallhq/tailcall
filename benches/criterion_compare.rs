#!/usr/bin/env cargo-script

//! ```cargo
//! [dependencies]
//! serde_json = "1.0"
//! serde = { version = "1", features = ["derive"] }
//! ```

use std::{env, fs};

use serde::Deserialize;
use serde_json::from_str;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Benchmark {
  id: String,
  typical: Typical,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Typical {
  estimate: f64,
  unit: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  // Get command-line arguments
  let args: Vec<String> = env::args().collect();

  // Check if two file paths are provided
  if args.len() != 3 {
    eprintln!("Usage: {} <old_file_path> <new_file_path>", args[0]);
    std::process::exit(1);
  }

  // Extract file paths from command-line arguments
  let old_file_path = &args[1];
  let new_file_path = &args[2];

  let old_content = fs::read_to_string(old_file_path)?;
  let old_benchmarks: Vec<Benchmark> = old_content.lines().filter_map(|line| from_str(line).ok()).collect();

  let new_content = fs::read_to_string(new_file_path)?;
  let new_benchmarks: Vec<Benchmark> = new_content.lines().filter_map(|line| from_str(line).ok()).collect();

  if old_benchmarks.len() != new_benchmarks.len() {
    return Err("Mismatch in the number of benchmarks between old and new files".into());
  }

  // Specify the output file path for Markdown
  let markdown_output_file_path = "benches/benchmark.md";

  // Generate the comparison table in markdown and write it to the output file
  let comparison_table_markdown = generate_comparison_table_markdown(&old_benchmarks, &new_benchmarks)?;
  fs::write(markdown_output_file_path, comparison_table_markdown)?;

  // Check for benchmarks exceeding the 10% change threshold
  let benchmarks_exceeding_threshold: Vec<_> = old_benchmarks
    .iter()
    .zip(new_benchmarks.iter())
    .filter_map(|(old, new)| {
      let percentage_change = calculate_percentage_change(old.typical.estimate, new.typical.estimate);
      if percentage_change.abs() > 10.0 {
        Some(old.id.clone())
      } else {
        None
      }
    })
    .collect();

  // If there are benchmarks exceeding the threshold, print their names
  if !benchmarks_exceeding_threshold.is_empty() {
    let exceeding_benchmarks_str = benchmarks_exceeding_threshold.join(", ");
    println!(
      "Benchmarks exceeding the 10% change threshold: {}",
      exceeding_benchmarks_str
    );
  }

  Ok(())
}

fn generate_comparison_table_markdown(
  old_benchmarks: &[Benchmark],
  new_benchmarks: &[Benchmark],
) -> Result<String, Box<dyn std::error::Error>> {
  let mut comparison_table = String::new();

  comparison_table.push_str("| Benchmark | Base | Change | Percentage Change |\n");
  comparison_table.push_str("|-----------|------|--------|-------------------|\n");

  for (old, new) in old_benchmarks.iter().zip(new_benchmarks) {
    let old_estimate = format_value(old.typical.estimate, &old.typical.unit);
    let new_estimate = format_value(new.typical.estimate, &new.typical.unit);
    let percentage_change = calculate_percentage_change(old.typical.estimate, new.typical.estimate);

    // Modify the formatting to display converted numbers
    comparison_table.push_str(&format!(
      "| {} | {} | {} | {:.2}% |\n",
      old.id, old_estimate, new_estimate, percentage_change
    ));
  }

  Ok(comparison_table)
}

fn format_value(value: f64, unit: &str) -> String {
  match unit {
    "ns" if value >= 1000.0 => format!("{:.2} μs", value / 1000.0),
    "μs" if value >= 1000.0 => format!("{:.2} ms", value / 1000.0),
    "ms" if value >= 1000.0 => format!("{:.2} s", value / 1000.0),
    _ => format!("{:.2} {}", value, unit),
  }
}

fn calculate_percentage_change(old_value: f64, new_value: f64) -> f64 {
  if old_value == 0.0 {
    0.0
  } else {
    ((new_value - old_value) / old_value) * 100.0
  }
}
