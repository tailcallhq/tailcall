use criterion::{black_box, criterion_group, criterion_main, Criterion};
use reqwest::Url;

fn bench_request(c: &mut Criterion) {
  let tmpl = RequestTemplate::new("http://localhost:3000/{{id}}").unwrap();

  let mut group = c.benchmark_group("request");

  group.bench_function("baseline", |b| {
    let ctx = serde_json::json!({"id": 1});
    b.iter(|| tmpl.to_request(black_box(&ctx)).unwrap())
  });

  // Optimize by re-using url between runs
  group.bench_function("reuse url", |b| {
    let url = Url::parse("http://localhost:3000/1").unwrap();
    let ctx = serde_json::json!({"id": 1});
    b.iter(|| {
      let mut req = reqwest::Request::new(reqwest::Method::GET, url.clone());
      tmpl.eval_headers(black_box(&ctx)).extend(req.headers_mut());
      req.body_mut().replace(tmpl.eval_body(black_box(&ctx)));
      req
    })
  });

  group.finish();
}

criterion_group!(benches, bench_request);
criterion_main!(benches);