use anyhow::Result;
use hyper::Response;
use once_cell::sync::Lazy;
use opentelemetry::metrics::Counter;
use opentelemetry::propagation::Extractor;
use opentelemetry::KeyValue;
use opentelemetry_semantic_conventions::trace::{
    HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE, URL_PATH,
};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::core::blueprint::telemetry::Telemetry;
use crate::core::http::Request;
use crate::core::Body;

static HTTP_SERVER_REQUEST_COUNT: Lazy<Counter<u64>> = Lazy::new(|| {
    let meter = opentelemetry::global::meter("http_request");

    meter
        .u64_counter("http.server.request.count")
        .with_description("Number of incoming request handled")
        .init()
});

#[derive(Default)]
pub struct RequestCounter {
    attributes: Option<Vec<KeyValue>>,
}

impl RequestCounter {
    pub fn new(telemetry: &Telemetry, req: &Request) -> Self {
        if telemetry.export.is_none() {
            return Self::default();
        }

        let observable_headers = &telemetry.request_headers;
        let headers = &req.headers;
        let mut attributes = Vec::with_capacity(observable_headers.len() + 3);

        attributes.push(KeyValue::new(URL_PATH, req.uri.path().to_string()));
        attributes.push(KeyValue::new(HTTP_REQUEST_METHOD, req.method.to_string()));

        for name in observable_headers {
            if let Some(value) = headers.get(name) {
                attributes.push(KeyValue::new(
                    format!("http.request.header.{}", name),
                    format!("{:?}", value),
                ));
            }
        }

        Self { attributes: Some(attributes) }
    }

    pub fn set_http_route(&mut self, route: &str) {
        if let Some(ref mut attributes) = self.attributes {
            attributes.push(KeyValue::new(HTTP_ROUTE, route.to_string()));
        }
    }

    pub fn update(self, response: &Result<Response<Body>>) {
        if let Some(mut attributes) = self.attributes {
            if let Ok(response) = response {
                attributes.push(get_response_status_code(response))
            }
            HTTP_SERVER_REQUEST_COUNT.add(1, &attributes);
        }
    }
}

pub fn get_response_status_code(response: &Response<Body>) -> KeyValue {
    KeyValue::new(HTTP_RESPONSE_STATUS_CODE, response.status().as_u16() as i64)
}

// older version of telemetry don't support new hyper
pub struct HeaderExtractor<'a>(pub &'a hyper::header::HeaderMap);
impl<'a> Extractor for HeaderExtractor<'a> {
    /// Get a value for a key from the HeaderMap.  If the value is not valid
    /// ASCII, returns None.
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|value| value.to_str().ok())
    }

    /// Collect all the keys from the HeaderMap.
    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|value| value.as_str())
            .collect::<Vec<_>>()
    }
}

pub fn propagate_context(req: &Request) {
    let context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(&req.headers))
    });

    tracing::Span::current().set_parent(context);
}
