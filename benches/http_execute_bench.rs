use criterion::Criterion;
use hyper::Method;
use tailcall::{Blueprint, HttpIO, NativeHttp};

pub fn benchmark_http_execute_method(c: &mut Criterion) {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

    let mut blueprint = Blueprint::default();
    blueprint.upstream.http_cache = true; // allow http caching for bench test.
    let native_http = NativeHttp::init(&blueprint.upstream, &blueprint.telemetry);

    tokio_runtime.block_on(async {
        // cache the 1st request in order the evaluate the perf of underlying cache.

        let request_url = format!("http://jsonplaceholder.typicode.com/users");
        let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
        let _result = native_http.execute(request).await;
    });

    c.bench_function("test_http_execute_method", |b| {
        b.iter(|| {
            tokio_runtime.block_on(async {
                for _ in 0..100 {
                    let request_url = format!("http://jsonplaceholder.typicode.com/users");
                    let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
                    let _result = native_http.execute(request).await;
                }
            })
        });
    });
}
