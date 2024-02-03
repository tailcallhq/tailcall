use anyhow::Result;
use derive_setters::Setters;
use hyper::header::CONTENT_TYPE;
use hyper::{HeaderMap, Method};
use reqwest::header::HeaderValue;
use url::Url;

use super::request::create_grpc_request;
use crate::config::GraphQLOperationType;
use crate::grpc::protobuf::ProtobufOperation;
use crate::has_headers::HasHeaders;
use crate::helpers::headers::MustacheHeaders;
use crate::mustache::Mustache;
use crate::path::PathString;

static GRPC_MIME_TYPE: HeaderValue = HeaderValue::from_static("application/grpc");

#[derive(Setters, Debug, Clone)]
pub struct RequestTemplate {
    pub url: Mustache,
    pub headers: MustacheHeaders,
    pub body: Option<Mustache>,
    pub operation: ProtobufOperation,
    pub operation_type: GraphQLOperationType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedRequestTemplate {
    pub url: Url,
    pub headers: HeaderMap,
    pub body: String,
    pub operation: ProtobufOperation,
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
        let mut req = reqwest::Request::new(Method::POST, self.url.clone());
        req.headers_mut().extend(self.headers.clone());

        Ok(create_grpc_request(
            self.url.clone(),
            self.headers.clone(),
            self.operation.convert_input(self.body.as_str())?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::path::PathBuf;

    use derive_setters::Setters;
    use hyper::header::{HeaderName, HeaderValue};
    use hyper::{HeaderMap, Method};
    use pretty_assertions::assert_eq;

    use super::RequestTemplate;
    use crate::blueprint::Upstream;
    use crate::cli::init_runtime;
    use crate::config::reader::ConfigReader;
    use crate::config::{Config, Field, GraphQLOperationType, Grpc, Type};
    use crate::grpc::protobuf::{ProtobufOperation, ProtobufSet};
    use crate::mustache::Mustache;

    async fn get_protobuf_op() -> ProtobufOperation {
        let root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut test_file = root_dir.join(file!());

        test_file.pop();
        test_file.push("tests");
        test_file.push("greetings.proto");

        let runtime = init_runtime(&Upstream::default(), None);
        let reader = ConfigReader::init(runtime);
        let mut config = Config::default();
        let grpc = Grpc {
            proto_path: test_file.to_str().unwrap().to_string(),
            ..Default::default()
        };
        config.types.insert(
            "foo".to_string(),
            Type::default().fields(vec![("bar", Field::default().grpc(grpc))]),
        );

        let protobuf_set = ProtobufSet::from_proto_file(
            &reader
                .resolve(config)
                .await
                .unwrap()
                .extensions
                .grpc_file_descriptor,
        )
        .unwrap();

        let service = protobuf_set.find_service("Greeter").unwrap();

        service.find_operation("SayHello").unwrap()
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

    impl crate::path::PathString for Context {
        fn path_string<T: AsRef<str>>(&self, parts: &[T]) -> Option<Cow<'_, str>> {
            self.value.path_string(parts)
        }
    }

    impl crate::has_headers::HasHeaders for Context {
        fn headers(&self) -> &HeaderMap {
            &self.headers
        }
    }

    #[tokio::test]
    async fn request_with_empty_body() {
        let tmpl = RequestTemplate {
            url: Mustache::parse("http://localhost:3000/").unwrap(),
            headers: vec![(
                HeaderName::from_static("test-header"),
                Mustache::parse("value").unwrap(),
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
            url: Mustache::parse("http://localhost:3000/").unwrap(),
            headers: vec![],
            operation: get_protobuf_op().await,
            body: Some(Mustache::parse(r#"{ "name": "test" }"#).unwrap()),
            operation_type: GraphQLOperationType::Query,
        };
        let ctx = Context::default();
        let rendered = tmpl.render(&ctx).unwrap();
        let req = rendered.to_request().unwrap();

        if let Some(body) = req.body() {
            assert_eq!(body.as_bytes(), Some(b"\0\0\0\0\x06\n\x04test".as_ref()))
        }
    }
}
