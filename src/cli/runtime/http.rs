use std::time::Duration;

use anyhow::Result;
use http_cache_reqwest::{Cache, CacheMode, HttpCache, HttpCacheOptions};
use hyper::body::Bytes;
use once_cell::sync::Lazy;
use opentelemetry::metrics::Counter;
use opentelemetry::trace::SpanKind;
use opentelemetry::KeyValue;
use opentelemetry_http::HeaderInjector;
use opentelemetry_semantic_conventions::trace::{
    HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, NETWORK_PROTOCOL_VERSION, URL_FULL,
};
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use tailcall_http_cache::HttpCacheManager;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use super::HttpIO;
use crate::core::blueprint::telemetry::Telemetry;
use crate::core::blueprint::Upstream;
use crate::core::http::Response;

static HTTP_CLIENT_REQUEST_COUNT: Lazy<Counter<u64>> = Lazy::new(|| {
    let meter = opentelemetry::global::meter("http_request");

    meter
        .u64_counter("http.client.request.count")
        .with_description("Number of outgoing requests")
        .init()
});

#[derive(Default)]
struct RequestCounter {
    attributes: Option<Vec<KeyValue>>,
}

impl RequestCounter {
    fn new(enable_telemetry: bool, request: &reqwest::Request) -> Self {
        if !enable_telemetry {
            return Self::default();
        }

        let attributes = vec![
            KeyValue::new(URL_FULL, request.url().to_string()),
            KeyValue::new(HTTP_REQUEST_METHOD, request.method().to_string()),
            KeyValue::new(NETWORK_PROTOCOL_VERSION, format!("{:?}", request.version())),
        ];

        Self { attributes: Some(attributes) }
    }

    fn update(&mut self, response: &reqwest_middleware::Result<reqwest::Response>) {
        if let Some(ref mut attributes) = self.attributes {
            attributes.push(get_response_status(response));

            HTTP_CLIENT_REQUEST_COUNT.add(1, attributes);
        }
    }
}

fn get_response_status(response: &reqwest_middleware::Result<reqwest::Response>) -> KeyValue {
    let status_code = match response {
        Ok(resp) => resp.status().as_u16(),
        Err(err) => err.status().map(|code| code.as_u16()).unwrap_or(0),
    };
    KeyValue::new(HTTP_RESPONSE_STATUS_CODE, status_code as i64)
}

#[derive(Clone)]
pub struct NativeHttp {
    client: ClientWithMiddleware,
    http2_only: bool,
    enable_telemetry: bool,
}

impl Default for NativeHttp {
    fn default() -> Self {
        Self {
            client: ClientBuilder::new(Client::new()).build(),
            http2_only: false,
            enable_telemetry: false,
        }
    }
}

impl NativeHttp {
    pub fn init(upstream: &Upstream, telemetry: &Telemetry) -> Self {
        let mut builder = Client::builder()
            .tcp_keepalive(Some(Duration::from_secs(upstream.tcp_keep_alive)))
            .timeout(Duration::from_secs(upstream.timeout))
            .connect_timeout(Duration::from_secs(upstream.connect_timeout))
            .http2_keep_alive_interval(Some(Duration::from_secs(upstream.keep_alive_interval)))
            .http2_keep_alive_timeout(Duration::from_secs(upstream.keep_alive_timeout))
            .http2_keep_alive_while_idle(upstream.keep_alive_while_idle)
            .pool_idle_timeout(Some(Duration::from_secs(upstream.pool_idle_timeout)))
            .pool_max_idle_per_host(upstream.pool_max_idle_per_host)
            .user_agent(upstream.user_agent.clone());

        // Add Http2 Prior Knowledge
        if upstream.http2_only {
            builder = builder.http2_prior_knowledge();
        }

        // Add Http Proxy
        if let Some(ref proxy) = upstream.proxy {
            builder = builder.proxy(
                reqwest::Proxy::http(proxy.url.clone())
                    .expect("Failed to set proxy in http client"),
            );
        }

        let mut client = ClientBuilder::new(builder.build().expect("Failed to build client"));

        if upstream.http_cache > 0 {
            client = client.with(Cache(HttpCache {
                mode: CacheMode::Default,
                manager: HttpCacheManager::new(upstream.http_cache),
                options: HttpCacheOptions::default(),
            }))
        }
        Self {
            client: client.build(),
            http2_only: upstream.http2_only,
            enable_telemetry: telemetry.export.is_some(),
        }
    }
}

#[async_trait::async_trait]
impl HttpIO for NativeHttp {
    #[allow(clippy::blocks_in_conditions)]
    // because of the issue with tracing and clippy - https://github.com/rust-lang/rust-clippy/issues/12281
    #[tracing::instrument(
        skip_all,
        err,
        fields(
            otel.name = "upstream_request",
            otel.kind = ?SpanKind::Client,
            url.full = %request.url(),
            http.request.method = %request.method(),
            network.protocol.version = ?request.version()
        )
    )]
    async fn execute(&self, mut request: reqwest::Request) -> Result<Response<Bytes>> {
        if self.http2_only {
            *request.version_mut() = reqwest::Version::HTTP_2;
        }

        let mut req_counter = RequestCounter::new(self.enable_telemetry, &request);

        if self.enable_telemetry {
            opentelemetry::global::get_text_map_propagator(|propagator| {
                propagator.inject_context(
                    &tracing::Span::current().context(),
                    &mut HeaderInjector(request.headers_mut()),
                );
            });
        }

        tracing::info!(
            "{} {} {:?}",
            request.method(),
            request.url(),
            request.version()
        );
        tracing::debug!("request: {:?}", request);
        let response = self.client.execute(request).await;
        tracing::debug!("response: {:?}", response);

        req_counter.update(&response);

        if self.enable_telemetry {
            let status_code = get_response_status(&response);
            tracing::Span::current().set_attribute(status_code.key, status_code.value);
        }

        Ok(Response::from_reqwest(
            response?
                .error_for_status()
                .map_err(|err| err.without_url())?,
        )
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use reqwest::Method;
    use tokio;

    use super::*;
    use crate::core::http::Response;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    async fn make_request(request_url: &str, native_http: &NativeHttp) -> Response<Bytes> {
        let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
        let result = native_http.execute(request).await;
        result.unwrap()
    }

    #[tokio::test]
    async fn test_native_http_get_request_without_cache() {
        let server = start_mock_server();

        let header_serv = server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test");
            then.status(200).body("Hello");
        });

        let native_http = NativeHttp::init(&Default::default(), &Default::default());
        let port = server.port();
        // Build a GET request to the mock server
        let request_url = format!("http://localhost:{}/test", port);
        let response = make_request(&request_url, &native_http).await;

        // Assert the response is as expected
        assert_eq!(response.status, reqwest::StatusCode::OK);
        assert_eq!(response.body, Bytes::from("Hello"));
        assert!(response.headers.get("x-cache-lookup").is_none());

        let request_url = format!("http://localhost:{}/test", port);
        let response = make_request(&request_url, &native_http).await;

        // Assert the response is as expected
        assert_eq!(response.status, reqwest::StatusCode::OK);
        assert_eq!(response.body, Bytes::from("Hello"));
        assert!(response.headers.get("x-cache-lookup").is_none());

        header_serv.assert_hits(2);
    }

    #[tokio::test]
    async fn test_native_http_get_request_with_cache() {
        let server = start_mock_server();

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test-1");
            then.status(200).body("Hello");
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test-2");
            then.status(200).body("Hello");
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test-3");
            then.status(200).body("Hello");
        });

        let upstream = Upstream { http_cache: 2, ..Default::default() };
        let native_http = NativeHttp::init(&upstream, &Default::default());
        let port = server.port();

        let url1 = format!("http://localhost:{}/test-1", port);
        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");

        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "HIT");

        let url2 = format!("http://localhost:{}/test-2", port);
        let resp = make_request(&url2, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");

        let resp = make_request(&url2, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "HIT");

        // now cache is full, let's make 3rd request and cache it and evict url1.
        let url3 = format!("http://localhost:{}/test-3", port);
        let resp = make_request(&url3, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");

        let resp = make_request(&url3, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "HIT");

        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");
    }
}
