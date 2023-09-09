use std::time::Duration;
use criterion::{Criterion, criterion_group, criterion_main, SamplingMode, Throughput};
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

    let dl = tailcall::http::HttpDataLoader::default().to_async_data_loader();
    let server = Server {
        enable_http_cache: Some(true),
        ..Server::default()
    };

    let ctx = tailcall::evaluation_context::EvaluationContext::new(&dl).server(server);
    let endpoint = Endpoint::new(InetAddress::new("jsonplaceholder.typicode.com".to_string(), 80))
        .path(Path::new(vec![Segment::literal("posts".to_string())]));

    let expression = Lambda::from(()).to_endpoint(endpoint).expression;


    group.sampling_mode(SamplingMode::Auto);
    group.measurement_time(Duration::from_secs(10));
    group.throughput(Throughput::Elements(1));
    group.bench_function("test-endpoint", move |b| b.iter(|| {
        rt.block_on(test_endpoint(expression.clone(), &ctx))
    }));
    group.finish();
}

criterion_group!(benches, endpoint_benchmark);
criterion_main!(benches);