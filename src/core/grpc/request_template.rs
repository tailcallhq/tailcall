use std::hash::{Hash, Hasher};

use anyhow::Result;
use derive_setters::Setters;
use http::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use tailcall_hasher::TailcallHasher;
use url::Url;

use super::request::create_grpc_request;
use crate::core::config::GraphQLOperationType;
use crate::core::grpc::protobuf::ProtobufOperation;
use crate::core::has_headers::HasHeaders;
use crate::core::helpers::headers::MustacheHeaders;
use crate::core::ir::model::{CacheKey, IoId};
use crate::core::mustache::Mustache;
use crate::core::path::PathString;

static GRPC_MIME_TYPE: HeaderValue = HeaderValue::from_static("application/grpc");

#[derive(Setters, Debug, Clone)]
pub struct RequestTemplate {
    pub url: Mustache,
    pub headers: MustacheHeaders,
    pub body: Option<RequestBody>,
    pub operation: ProtobufOperation,
    pub operation_type: GraphQLOperationType,
}

#[derive(Default, Debug, Clone, PartialEq, Setters)]
pub struct RequestBody {
    pub mustache: Option<Mustache>,
    pub value: String,
}

impl RequestBody {
    pub fn render<C: PathString>(&self, ctx: &C) -> String {
        if let Some(mustache) = &self.mustache {
            mustache.render(ctx)
        } else {
            self.value.to_string()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedRequestTemplate {
    pub url: Url,
    pub headers: HeaderMap,
    pub body: String,
    pub operation: ProtobufOperation,
}

impl Hash for RenderedRequestTemplate {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.body.hash(state);
    }
}

impl RequestTemplate {
    fn create_url<C: PathString>(&self, ctx: &C) -> Result<Url> {
        let url = url::Url::parse(self.url.render(ctx).as_str())?;

        Ok(url)
    }

    fn create_headers<C: PathString>(&self, ctx: &C) -> HeaderMap {
        let mut header_map = HeaderMap::new();

        header_map.insert(CONTENT_TYPE, GRPC_MIME_TYPE.to_owned());

        for (k, v) in &self.headers {
            if let Ok(header_value) = HeaderValue::from_str(&v.render(ctx)) {
                header_map.insert(k, header_value);
            }
        }

        header_map
    }

    pub fn render<C: PathString + HasHeaders>(&self, ctx: &C) -> Result<RenderedRequestTemplate> {
        let url = self.create_url(ctx)?;
        let headers = self.render_headers(ctx);
        let body = self.render_body(ctx);
        Ok(RenderedRequestTemplate { url, headers, body, operation: self.operation.clone() })
    }

    fn render_body<C: PathString + HasHeaders>(&self, ctx: &C) -> String {
        if let Some(body) = &self.body {
            body.render(ctx)
        } else {
            "{}".to_owned()
        }
    }

    fn render_headers<C: PathString + HasHeaders>(&self, ctx: &C) -> HeaderMap {
        let mut req_headers = HeaderMap::new();

        let headers = self.create_headers(ctx);
        if !headers.is_empty() {
            req_headers.extend(headers);
        }

        req_headers.extend(ctx.headers().to_owned());

        req_headers
    }
}

impl RenderedRequestTemplate {
    pub fn to_request(&self) -> Result<reqwest::Request> {
        Ok(create_grpc_request(
            self.url.clone(),
            self.headers.clone(),
            self.operation.convert_input(self.body.as_str())?,
        ))
    }
}

impl<Ctx: PathString + HasHeaders> CacheKey<Ctx> for RequestTemplate {
    fn cache_key(&self, ctx: &Ctx) -> Option<IoId> {
        let mut hasher = TailcallHasher::default();
        let rendered_req = self.render(ctx).unwrap();
        rendered_req.hash(&mut hasher);
        Some(IoId::new(hasher.finish()))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashSet;

    use derive_setters::Setters;
    use http::header::{HeaderMap, HeaderName, HeaderValue};
    use http::Method;
    use pretty_assertions::assert_eq;
    use tailcall_fixtures::protobuf;

    use super::{RequestBody, RequestTemplate};
    use crate::core::blueprint::GrpcMethod;
    use crate::core::config::reader::ConfigReader;
    use crate::core::config::{
        Config, Field, GraphQLOperationType, Grpc, Link, LinkType, Resolver, Type,
    };
    use crate::core::grpc::protobuf::{ProtobufOperation, ProtobufSet};
    use crate::core::ir::model::CacheKey;
    use crate::core::mustache::Mustache;

    async fn get_protobuf_op() -> ProtobufOperation {
        let test_file = protobuf::GREETINGS;

        let id = "greetings".to_string();

        let runtime = crate::core::runtime::test::init(None);
        let reader = ConfigReader::init(runtime);
        let mut config = Config::default().links(vec![Link {
            id: Some(id.clone()),
            src: test_file.to_string(),
            type_of: LinkType::Protobuf,
            headers: None,
            meta: None,
            proto_paths: None,
        }]);
        let method = GrpcMethod {
            package: id.to_string(),
            service: "a".to_string(),
            name: "b".to_string(),
        };
        let grpc = Grpc { method: method.to_string(), ..Default::default() };
        config.types.insert(
            "foo".to_string(),
            Type::default().fields(vec![(
                "bar",
                Field::default().resolvers(Resolver::Grpc(grpc).into()),
            )]),
        );

        let protobuf_set = ProtobufSet::from_proto_file(
            reader
                .resolve(config, None)
                .await
                .unwrap()
                .extensions()
                .get_file_descriptor_set(),
        )
        .unwrap();

        let method = GrpcMethod::try_from("greetings.Greeter.SayHello").unwrap();
        let service = protobuf_set.find_service(&method).unwrap();

        service.find_operation(&method).unwrap()
    }

    #[derive(Setters)]
    struct Context {
        pub value: serde_json::Value,
        pub headers: HeaderMap,
    }

    impl Default for Context {
        fn default() -> Self {
            Self { value: serde_json::Value::Null, headers: HeaderMap::new() }
        }
    }

    impl crate::core::path::PathString for Context {
        fn path_string<'a, T: AsRef<str>>(&'a self, parts: &'a [T]) -> Option<Cow<'a, str>> {
            self.value.path_string(parts)
        }
    }

    impl crate::core::has_headers::HasHeaders for Context {
        fn headers(&self) -> &HeaderMap {
            &self.headers
        }
    }

    #[tokio::test]
    async fn request_with_empty_body() {
        let tmpl = RequestTemplate {
            url: Mustache::parse("http://localhost:3000/"),
            headers: vec![(
                HeaderName::from_static("test-header"),
                Mustache::parse("value"),
            )],
            operation: get_protobuf_op().await,
            body: None,
            operation_type: GraphQLOperationType::Query,
        };
        let ctx = Context::default();
        let rendered = tmpl.render(&ctx).unwrap();
        let req = rendered.to_request().unwrap();

        assert_eq!(req.url().as_str(), "http://localhost:3000/");
        assert_eq!(req.method(), Method::POST);
        assert_eq!(
            req.headers(),
            &HeaderMap::from_iter([
                (
                    HeaderName::from_static("test-header"),
                    HeaderValue::from_static("value")
                ),
                (
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("application/grpc")
                )
            ])
        );

        if let Some(body) = req.body() {
            assert_eq!(body.as_bytes(), Some(b"\0\0\0\0\0".as_ref()))
        }
    }

    #[tokio::test]
    async fn request_with_body() {
        let tmpl = RequestTemplate {
            url: Mustache::parse("http://localhost:3000/"),
            headers: vec![],
            operation: get_protobuf_op().await,
            body: Some(RequestBody {
                mustache: Some(Mustache::parse(r#"{ "name": "test" }"#)),
                value: Default::default(),
            }),
            operation_type: GraphQLOperationType::Query,
        };
        let ctx = Context::default();
        let rendered = tmpl.render(&ctx).unwrap();
        let req = rendered.to_request().unwrap();

        if let Some(body) = req.body() {
            assert_eq!(body.as_bytes(), Some(b"\0\0\0\0\x06\n\x04test".as_ref()))
        }
    }

    async fn request_template_with_body(body_str: &str) -> RequestTemplate {
        RequestTemplate {
            url: Mustache::parse("http://localhost:3000/"),
            headers: vec![],
            operation: get_protobuf_op().await,
            body: Some(RequestBody {
                mustache: Some(Mustache::parse(body_str)),
                value: Default::default(),
            }),
            operation_type: GraphQLOperationType::Query,
        }
    }

    #[tokio::test]
    async fn test_grpc_cache_key_collision() {
        let arr = [
            r#"{ "name": "test" }"#,
            r#"{ "name": "test1" }"#,
            r#"{ "name1": "test" }"#,
            r#"{ "name1": "test1" }"#,
        ];

        let ctx = Context::default();
        let tmpl_set: HashSet<_> =
            futures_util::future::join_all(arr.iter().cloned().zip(std::iter::repeat(&ctx)).map(
                |(body_str, ctx)| async {
                    let tmpl = request_template_with_body(body_str).await;
                    tmpl.cache_key(ctx)
                },
            ))
            .await
            .into_iter()
            .collect();

        assert_eq!(arr.len(), tmpl_set.len());
    }
}
