use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use benchmark::{run_data_loader_benchmark, MockHttpClient};

// Benchmark function for the data loader
fn benchmark_data_loader(c: &mut Criterion) {
  c.bench_function("test_data_loader", |b| {
    b.iter(|| {
      // Spawn a new Tokio runtime and run the data loader benchmark asynchronously
      tokio::runtime::Runtime::new().unwrap().spawn(async {
        let client = Arc::new(MockHttpClient { request_count: Arc::new(AtomicUsize::new(0)) });
        run_data_loader_benchmark(client).await;
      });
    })
  });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_data_loader
}
criterion_main!(benches);
