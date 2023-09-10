use criterion::{criterion_group, criterion_main, Criterion, SamplingMode, Throughput};
use std::time::Duration;
use tailcall::config::Server;

use tailcall::endpoint::Endpoint;
use tailcall::evaluation_context::EvaluationContext;
use tailcall::expression::Expression;
use tailcall::inet_address::InetAddress;
use tailcall::lambda::Lambda;
use tailcall::path::{Path, Segment};

pub async fn test_endpoint<'a>(expression: Expression, ctx: &'a EvaluationContext<'a>) -> () {
    expression.eval(&ctx).await.unwrap();
}

fn endpoint_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("lambda-benchmark-group");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let server = Server { enable_join_cache: Some(true), enable_http_cache: Some(true), ..Server::default() };
    let client = tailcall::http::HttpClient::new(true, None, false);
    let endpoint = Endpoint::new(InetAddress::new("jsonplaceholder.typicode.com".to_string(), 80))
        .path(Path::new(vec![Segment::literal("posts".to_string())]));

    let expression = Lambda::from(()).to_endpoint(endpoint).expression;

    group.sampling_mode(SamplingMode::Auto);
    group.sampling_mode(SamplingMode::Auto);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(1));
    group.bench_function("test-endpoint", move |b| {
        b.iter(|| {
            let data_loader = tailcall::http::HttpDataLoader::new(client.clone())
                .to_async_data_loader()
                .delay(Duration::from_secs(0))
                .max_batch_size(0);
            rt.block_on(test_endpoint(
                expression.clone(),
                &tailcall::evaluation_context::EvaluationContext::new(&data_loader, &client, &server).server(&server),
            ))
        })
    });
    group.finish();
}

criterion_group!(benches, endpoint_benchmark);
criterion_main!(benches);
