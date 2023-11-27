use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use criterion::{criterion_group, criterion_main, Criterion};
use tailcall::benchmark::{run_data_loader_benchmark, MockHttpClient};

fn benchmark_data_loader(c: &mut Criterion) {
  c.bench_function("test_data_loader", |b| {
    b.iter(|| {
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
