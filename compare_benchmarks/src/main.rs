use std::env;

use compare_benchmarks::parse_json;

fn main() {
  // Get command-line arguments
  let args: Vec<String> = env::args().collect();

  // Check if the correct number of arguments is provided
  if args.len() != 2 {
    eprintln!("Usage: {} <file_path>", args[0]);
    std::process::exit(1);
  }

  // Extract the file path from the command-line arguments
  let file_path = &args[1];

  // Parse and print the JSON content in a table
  if let Err(err) = parse_json(file_path) {
    eprintln!("Error parsing {}: {}", file_path, err);
  }
}
