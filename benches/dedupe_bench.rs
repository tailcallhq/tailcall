use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use criterion::Criterion;
use futures_util::future::join_all;
use tailcall::core::data_loader::DedupeResult;
use tokio::runtime::Builder;

pub fn benchmark_dedupe(c: &mut Criterion) {
    c.bench_function("dedupe concurrent access with thread pool", |b| {
        b.iter(|| {
            // Create a Tokio runtime with a thread pool of 5 threads
            let rt = Builder::new_multi_thread()
                .worker_threads(5)
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                let cache = Arc::new(DedupeResult::<u64, String, ()>::new(false));
                let key = 1;
                let counter = Arc::new(AtomicUsize::new(0));
                let mut handles = Vec::new();

                for _ in 0..10000000 {
                    let cache = cache.clone();
                    let counter = counter.clone();
                    let handle = tokio::spawn(async move {
                        
                        cache
                            .dedupe(&key, || Box::pin(compute_value(counter)))
                            .await
                    });
                    handles.push(handle);
                }

                let results = join_all(handles).await;
                let all_ok = results.into_iter().all(|r| r.unwrap().is_ok());

                assert!(all_ok);
            });
        });
    });
}

async fn compute_value(counter: Arc<AtomicUsize>) -> Result<String, ()> {
    counter.fetch_add(1, Ordering::SeqCst);
    tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
    Ok(format!("value_{}", counter.load(Ordering::SeqCst)))
}
