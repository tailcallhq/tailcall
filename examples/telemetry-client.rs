use anyhow::{anyhow, Result};
use hyper::body::{Body, HttpBody};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Client, HeaderMap, Method, Response};
use once_cell::sync::Lazy;
use opentelemetry::trace::{SpanKind, TraceContextExt, TraceError, Tracer};
use opentelemetry::{global, Context, KeyValue};
use opentelemetry_http::HeaderInjector;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::{runtime, Resource};
use tonic::metadata::MetadataMap;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::default().merge(&Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "tailcall-client-example",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            "test",
        ),
    ]))
});

fn init_tracer() -> Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    static TELEMETRY_URL: &str = "https://api.honeycomb.io:443";
    let headers = HeaderMap::from_iter([(
        HeaderName::from_static("x-honeycomb-team"),
        HeaderValue::from_str(&std::env::var("HONEYCOMB_API_KEY")?)?,
    )]);

    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(TELEMETRY_URL)
        .with_metadata(MetadataMap::from_headers(headers));

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(RESOURCE.clone()))
        .install_batch(runtime::Tokio)?
        .provider()
        .ok_or(TraceError::Other(
            anyhow!("Failed to instantiate OTLP provider").into(),
        ))?;

    global::set_tracer_provider(provider);

    Ok(())
}

async fn send_request(
    url: &str,
    method: Method,
    body_content: &str,
    span_name: &str,
) -> std::result::Result<Response<Body>, Box<dyn std::error::Error + Send + Sync + 'static>> {
    let client = Client::new();
    let tracer = global::tracer("example/client");
    let span = tracer
        .span_builder(String::from(span_name))
        .with_kind(SpanKind::Client)
        .start(&tracer);
    let cx = Context::current_with_span(span);

    let mut req = hyper::Request::builder().uri(url).method(method);
    global::get_text_map_propagator(|propagator| {
        propagator.inject_context(&cx, &mut HeaderInjector(req.headers_mut().unwrap()))
    });
    let res = client
        .request(req.body(Body::from(String::from(body_content)))?)
        .await?;

    cx.span().add_event(
        "Got response!".to_string(),
        vec![KeyValue::new("status", res.status().to_string())],
    );

    Ok(res)
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    init_tracer()?;

    let response = send_request(
        "http://127.0.0.1:8000/graphql",
        Method::POST,
        r#"{ "query": "{ user(id: 1) { name } news { news { title } } }" }"#,
        "client_graphql_request",
    )
    .await?;

    let buf = response.into_body().collect().await?.to_bytes();
    println!("Got response: {:?}", buf);

    global::shutdown_tracer_provider();

    Ok(())
}
