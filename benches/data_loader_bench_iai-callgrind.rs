use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use tailcall::benchmark::{run_data_loader_benchmark, MockHttpClient};

// IAI-callgrind benchmark for the data loader
#[library_benchmark]
async fn benchmark_data_loader() {
  // Create a mock HTTP client for the benchmark
  let client = Arc::new(MockHttpClient { request_count: Arc::new(AtomicUsize::new(0)) });

  // Run the data loader benchmark asynchronously
  run_data_loader_benchmark(client).await;
}

// Define the benchmark group for IAI-callgrind
library_benchmark_group!(name = data_loader; benchmarks = benchmark_data_loader);
main!(library_benchmark_groups = data_loader);
