use tailcall_valid::Valid;

use crate::core::{
    config::{Config, Grpc, Http, Resolver, ResolverSet},
    Transform,
};

pub struct ConnectRPC;

impl Transform for ConnectRPC {
    type Value = Config;
    type Error = String;

    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        for (_, type_) in config.types.iter_mut() {
            for (_, field_) in type_.fields.iter_mut() {
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
        let url = grpc.url.clone();
        let body = grpc.body.clone();
        // remove the last
        // method: package.service.method
        // remove the method from the end.
        let parts = grpc.method.split(".").collect::<Vec<_>>();
        let method = parts[..parts.len() - 1].join(".").to_string();
        let endpoint = parts[parts.len() - 1].to_string();

        let new_url = format!("{}/{}/{}", url, method, endpoint);
        let headers = grpc.headers.clone();
        let batch_key = grpc.batch_key.clone();
        let dedupe = grpc.dedupe.clone();
        let select = grpc.select.clone();
        let on_response_body = grpc.on_response_body.clone();

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
