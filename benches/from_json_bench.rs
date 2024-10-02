use criterion::Criterion;
use http::Method;
use serde_json::Value;
use tailcall::cli::runtime::NativeHttp;
use tailcall::core::generator::{Generator, Input};
use tailcall::core::http::Method as HTTPMethod;
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

    let cfg_gen_reqs = vec![Input::Json {
        url: request_url.parse().unwrap(),
        method: HTTPMethod::GET,
        req_body: serde_json::Value::Null,
        res_body: reqs[0].clone(),
        field_name: "f1".to_string(),
        is_mutation: false,
        headers: None,
    }];

    let config_generator = Generator::default().inputs(cfg_gen_reqs);

    c.bench_function("from_json_bench", |b| {
        b.iter(|| {
            let _ = config_generator.generate(false);
        });
    });
}
