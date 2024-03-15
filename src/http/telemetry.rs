use anyhow::Result;
use hyper::{Body, Request, Response};
use once_cell::sync::Lazy;
use opentelemetry::metrics::Counter;
use opentelemetry::KeyValue;
use opentelemetry_semantic_conventions::trace::{
    HTTP_REQUEST_METHOD, HTTP_RESPONSE_STATUS_CODE, HTTP_ROUTE, URL_PATH,
};

use crate::blueprint::telemetry::Telemetry;

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
    pub fn new(telemetry: &Telemetry, req: &Request<Body>) -> Self {
        if telemetry.export.is_none() {
            return Self::default();
        }

        let observable_headers = &telemetry.request_headers;
        let headers = req.headers();
        let mut attributes = Vec::with_capacity(observable_headers.len() + 3);

        attributes.push(KeyValue::new(URL_PATH, req.uri().path().to_string()));
        attributes.push(KeyValue::new(HTTP_REQUEST_METHOD, req.method().to_string()));

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
