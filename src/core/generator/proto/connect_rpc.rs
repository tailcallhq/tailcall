use std::fmt::Display;

use tailcall_valid::Valid;

use crate::core::config::{Config, Grpc, Http, Resolver, ResolverSet};
use crate::core::Transform;

const HEADER_CONNECT_PROTOCOL_VERSION: &str = "Connect-Protocol-Version";

enum ConnectProtocolVersion {
    V1,
}

impl Display for ConnectProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConnectProtocolVersion::V1 => "1".to_string(),
            }
        )
    }
}

pub struct ConnectRPC;

impl Transform for ConnectRPC {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        for type_ in config.types.values_mut() {
            for field_ in type_.fields.values_mut() {
                let new_resolvers = field_
                    .resolvers
                    .0
                    .iter()
                    .map(|resolver| match resolver {
                        Resolver::Grpc(grpc) => Resolver::Http(Http::from(grpc.clone())),
                        other => other.clone(),
                    })
                    .collect();

                field_.resolvers = ResolverSet(new_resolvers);
            }
        }

        Valid::succeed(config)
    }
}

impl From<Grpc> for Http {
    fn from(grpc: Grpc) -> Self {
        let url = grpc.url;
        let body = grpc.body.or_else(|| {
            // if body isn't present while transforming the resolver, we need to provide an
            // empty object.
            Some(serde_json::Value::Object(serde_json::Map::new()))
        });

        // remove the last
        // method: package.service.method
        // remove the method from the end.
        let parts = grpc.method.split(".").collect::<Vec<_>>();
        let method = parts[..parts.len() - 1].join(".").to_string();
        let endpoint = parts[parts.len() - 1].to_string();

        let new_url = format!("{}/{}/{}", url, method, endpoint);
        let mut headers = grpc.headers;
        headers.push(crate::core::config::KeyValue {
            key: HEADER_CONNECT_PROTOCOL_VERSION.to_string(),
            value: ConnectProtocolVersion::V1.to_string(),
        });

        let batch_key = grpc.batch_key;
        let dedupe = grpc.dedupe;
        let select = grpc.select;
        let on_response_body = grpc.on_response_body;

        Self {
            url: new_url,
            body,
            method: crate::core::http::Method::POST,
            headers,
            batch_key,
            dedupe,
            select,
            on_response_body,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::*;
    use crate::core::config::KeyValue;

    #[test]
    fn test_grpc_to_http_basic_conversion() {
        let grpc = Grpc {
            url: "http://localhost:8080".to_string(),
            method: "package.service.method".to_string(),
            body: Some(json!({"key": "value"})),
            headers: Default::default(),
            batch_key: Default::default(),
            dedupe: Default::default(),
            select: Default::default(),
            on_response_body: Default::default(),
        };

        let http = Http::from(grpc);

        assert_eq!(http.url, "http://localhost:8080/package.service/method");
        assert_eq!(http.method, crate::core::http::Method::POST);
        assert_eq!(http.body, Some(json!({"key": "value"})));
    }

    #[test]
    fn test_grpc_to_http_empty_body() {
        let grpc = Grpc {
            url: "http://localhost:8080".to_string(),
            method: "package.service.method".to_string(),
            body: Default::default(),
            headers: Default::default(),
            batch_key: Default::default(),
            dedupe: Default::default(),
            select: Default::default(),
            on_response_body: Default::default(),
        };

        let http = Http::from(grpc);

        assert_eq!(http.body, Some(json!({})));
    }

    #[test]
    fn test_grpc_to_http_with_headers() {
        let grpc = Grpc {
            url: "http://localhost:8080".to_string(),
            method: "a.b.c".to_string(),
            body: None,
            headers: vec![KeyValue { key: "X-Foo".to_string(), value: "bar".to_string() }],
            batch_key: Default::default(),
            dedupe: Default::default(),
            select: Default::default(),
            on_response_body: Default::default(),
        };

        let http = Http::from(grpc);

        assert_eq!(http.url, "http://localhost:8080/a.b/c");
        assert_eq!(
            http.headers
                .iter()
                .find(|h| h.key == "X-Foo")
                .unwrap()
                .value,
            "bar".to_string()
        );
        assert_eq!(
            http.headers
                .iter()
                .find(|h| h.key == "Connect-Protocol-Version")
                .unwrap()
                .value,
            "1".to_string()
        );
        assert_eq!(http.body, Some(json!({})));
    }

    #[test]
    fn test_grpc_to_http_all_fields() {
        let grpc = Grpc {
            url: "http://localhost:8080".to_string(),
            method: "package.service.method".to_string(),
            body: Some(json!({"key": "value"})),
            headers: vec![KeyValue { key: "X-Foo".to_string(), value: "bar".to_string() }],
            batch_key: vec!["batch_key_value".to_string()],
            dedupe: Some(true),
            select: Some(Value::String("select_value".to_string())),
            on_response_body: Some("on_response_body_value".to_string()),
        };

        let http = Http::from(grpc);

        assert_eq!(http.url, "http://localhost:8080/package.service/method");
        assert_eq!(http.method, crate::core::http::Method::POST);
        assert_eq!(http.body, Some(json!({"key": "value"})));
        assert_eq!(
            http.headers
                .iter()
                .find(|h| h.key == "X-Foo")
                .unwrap()
                .value,
            "bar".to_string()
        );
        assert_eq!(http.batch_key, vec!["batch_key_value".to_string()]);
        assert_eq!(http.dedupe, Some(true));
        assert_eq!(http.select, Some(Value::String("select_value".to_string())));
        assert_eq!(
            http.on_response_body,
            Some("on_response_body_value".to_string())
        );
    }
}
