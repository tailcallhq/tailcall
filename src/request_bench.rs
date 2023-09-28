use criterion::{black_box, criterion_group, criterion_main, Criterion};
use reqwest::Url;

use myapp::request_template::RequestTemplate; 

fn bench_request(c: &mut Criterion) {

  let tmpl = RequestTemplate::new("http://localhost:3000/{{id}}").unwrap();

  let mut group = c.benchmark_group("request");

  group.bench_function("baseline", |b| {
    let ctx = serde_json::json!({"id": 1});
    b.iter(|| tmpl.to_request(black_box(&ctx)).unwrap())
  });

  // Cache the template evaluation
  group.bench_function("cache eval", |b| {
    let ctx = serde_json::json!({"id": 1});
    let url = tmpl.eval_url(&ctx).unwrap();
    let headers = tmpl.eval_headers(&ctx);
    let body = tmpl.eval_body(&ctx);

    b.iter(|| {
      let mut req = reqwest::Request::new(reqwest::Method::GET, url.clone());
      req.headers_mut().extend(headers.clone());
      req.body_mut().replace(body.clone());
      req
    })
  });

  group.finish();
}

criterion_group!(benches, bench_request);
criterion_main!(benches);