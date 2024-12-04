use tailcall_valid::Valid;

use crate::core::config::{Config, Grpc, Http, Resolver, ResolverSet};
use crate::core::Transform;

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
        let headers = grpc.headers;
        let batch_key = grpc.batch_key;
        let dedupe = grpc.dedupe;
        let select = grpc.select;
        let on_response_body = grpc.on_response_body;

        Self {
            url: new_url,
            body: body.map(|b| b.to_string()),
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
