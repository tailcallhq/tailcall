use criterion::Criterion;
use hyper::Method;
use serde_json::Value;
use tailcall::cli::runtime::NativeHttp;
use tailcall::core::generator::{from_json, ConfigGenerationRequest};
use tailcall::core::HttpIO;

pub fn benchmark_from_json_method(c: &mut Criterion) {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

    let native_http = NativeHttp::init(&Default::default(), &Default::default());
    let request_url = String::from("http://jsonplaceholder.typicode.com/users");

    let mut reqs = Vec::with_capacity(1);
    tokio_runtime.block_on(async {
        // cache the 1st request in order the evaluate the perf of underlying cache.
        let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
        let result = native_http.execute(request).await.unwrap();
        let body: Value = serde_json::from_slice(&result.body).unwrap();
        reqs.push(body);
    });

    let cfg_gen_reqs = [ConfigGenerationRequest::new(&request_url, &reqs[0])];

    c.bench_function("from_json_bench", |b| {
        b.iter(|| {
            let _ = from_json(&cfg_gen_reqs);
        });
    });
}
