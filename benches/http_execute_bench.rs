use criterion::Criterion;
use hyper::Method;
use tailcall::cli::runtime::NativeHttp;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::HttpIO;

pub fn benchmark_http_execute_method(c: &mut Criterion) {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

    let mut blueprint = Blueprint::default();
    blueprint.upstream.http_cache = true; // allow http caching for bench test.
    let native_http = NativeHttp::init(&blueprint.upstream, &blueprint.telemetry);
    let request_url = String::from("http://jsonplaceholder.typicode.com/users");

    tokio_runtime.block_on(async {
        // cache the 1st request in order the evaluate the perf of underlying cache.
        let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
        let _result = native_http.execute(request).await;
    });

    c.bench_function("test_http_execute_method", |b| {
        b.iter(|| {
            tokio_runtime.block_on(async {
                let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
                let _result = native_http.execute(request).await;
            })
        });
    });
}
