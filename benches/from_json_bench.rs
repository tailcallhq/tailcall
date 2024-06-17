use criterion::Criterion;
use hyper::Method;
use serde_json::Value;
use tailcall::cli::runtime::NativeHttp;
use tailcall::core::generator::{Generator, Input};
use tailcall::core::HttpIO;

pub fn benchmark_from_json_method(c: &mut Criterion) {
    let tokio_runtime = tokio::runtime::Runtime::new().unwrap();

    let native_http = NativeHttp::init(&Default::default(), &Default::default());
    let request_url = String::from("http://jsonplaceholder.typicode.com/users");

    let mut reqs = Vec::with_capacity(1);
    tokio_runtime.block_on(async {
        let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
        let result = native_http.execute(request).await.unwrap();
        let body: Value = serde_json::from_slice(&result.body).unwrap();
        reqs.push(body);
    });

    let cfg_gen_reqs =
        vec![Input::Json { url: request_url.parse().unwrap(), response: reqs[0].clone() }];

    let config_generator = Generator::new()
        .with_inputs(cfg_gen_reqs)
        .with_type_name_prefix("T")
        .with_field_name_prefix("f")
        .with_operation_name("Query");
    

    c.bench_function("from_json_bench", |b| {
        b.iter(|| {
            let _ = config_generator.generate();
        });
    });
}
