#![allow(clippy::too_many_arguments)]

use std::hash::{Hash, Hasher};

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::HeaderValue;
use tailcall_hasher::TailcallHasher;

use crate::core::config::{GraphQLOperationType, KeyValue};
use crate::core::has_headers::HasHeaders;
use crate::core::helpers::headers::MustacheHeaders;
use crate::core::http::Method::POST;
use crate::core::ir::{CacheKey, GraphQLOperationContext, IoId};
use crate::core::mustache::Mustache;
use crate::core::path::PathGraphql;

/// RequestTemplate for GraphQL requests (See RequestTemplate documentation)
#[derive(Setters, Debug, Clone)]
pub struct RequestTemplate {
    // TODO: should be Mustache as for other templates
    pub url: String,
    pub operation_type: GraphQLOperationType,
    pub operation_name: String,
    pub operation_arguments: Option<Vec<(String, Mustache)>>,
    pub headers: MustacheHeaders,
}

impl RequestTemplate {
    fn create_headers<C: PathGraphql>(&self, ctx: &C) -> HeaderMap {
        let mut header_map = HeaderMap::new();

        for (k, v) in &self.headers {
            if let Ok(header_value) = HeaderValue::from_str(&v.render_graphql(ctx)) {
                header_map.insert(k, header_value);
            }
        }

        header_map
    }

    fn set_headers<C: PathGraphql + HasHeaders>(
        &self,
        mut req: reqwest::Request,
        ctx: &C,
    ) -> reqwest::Request {
        let headers = req.headers_mut();
        let config_headers = self.create_headers(ctx);

        if !config_headers.is_empty() {
            headers.extend(config_headers);
        }
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        headers.extend(ctx.headers().to_owned());
        req
    }

    pub fn to_request<C: PathGraphql + HasHeaders + GraphQLOperationContext>(
        &self,
        ctx: &C,
    ) -> anyhow::Result<reqwest::Request> {
        let mut req = reqwest::Request::new(POST.to_hyper(), url::Url::parse(self.url.as_str())?);
        req = self.set_headers(req, ctx);
        req = self.set_body(req, ctx);
        Ok(req)
    }

    fn set_body<C: PathGraphql + HasHeaders + GraphQLOperationContext>(
        &self,
        mut req: reqwest::Request,
        ctx: &C,
    ) -> reqwest::Request {
        req.body_mut()
            .replace(self.render_graphql_query(ctx).into());
        req
    }

    fn render_graphql_query<C: PathGraphql + HasHeaders + GraphQLOperationContext>(
        &self,
        ctx: &C,
    ) -> String {
        let operation_type = &self.operation_type;
        let selection_set = ctx.selection_set().unwrap_or_default();
        let operation = self
            .operation_arguments
            .as_ref()
            .map(|args| {
                args.iter()
                    .map(|(k, v)| format!(r#"{}: {}"#, k, v.render_graphql(ctx).escape_default()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .map(|args| format!("{}({})", self.operation_name, args))
            .unwrap_or(self.operation_name.clone());

        format!(r#"{{ "query": "{operation_type} {{ {operation} {selection_set} }}" }}"#)
    }

    pub fn new(
        url: String,
        operation_type: &GraphQLOperationType,
        operation_name: &str,
        args: Option<&Vec<KeyValue>>,
        headers: MustacheHeaders,
    ) -> anyhow::Result<Self> {
        let mut operation_arguments = None;

        if let Some(args) = args.as_ref() {
            operation_arguments = Some(
                args.iter()
                    .map(|kv| Ok((kv.key.to_owned(), Mustache::parse(&kv.value)?)))
                    .collect::<anyhow::Result<Vec<_>>>()?,
            );
        }

        Ok(Self {
            url,
            operation_type: operation_type.to_owned(),
            operation_name: operation_name.to_owned(),
            operation_arguments,
            headers,
        })
    }
}

impl<Ctx: PathGraphql + HasHeaders + GraphQLOperationContext> CacheKey<Ctx> for RequestTemplate {
    fn cache_key(&self, ctx: &Ctx) -> Option<IoId> {
        let mut hasher = TailcallHasher::default();
        let graphql_query = self.render_graphql_query(ctx);
        graphql_query.hash(&mut hasher);
        Some(IoId::new(hasher.finish()))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use async_graphql::Value;
    use hyper::HeaderMap;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::core::config::GraphQLOperationType;
    use crate::core::graphql::RequestTemplate;
    use crate::core::has_headers::HasHeaders;
    use crate::core::ir::{CacheKey, GraphQLOperationContext};
    use crate::core::json::JsonLike;
    use crate::core::path::PathGraphql;

    struct Context {
        pub value: Value,
        pub headers: HeaderMap,
    }

    impl PathGraphql for Context {
        fn path_graphql<T: AsRef<str>>(&self, path: &[T]) -> Option<String> {
            self.value.get_path(path).map(|v| v.to_string())
        }
    }

    impl HasHeaders for Context {
        fn headers(&self) -> &HeaderMap {
            &self.headers
        }
    }

    impl GraphQLOperationContext for Context {
        fn selection_set(&self) -> Option<String> {
            Some("{ a,b,c }".to_owned())
        }
    }

    #[test]
    fn test_query_without_args() {
        let tmpl = RequestTemplate::new(
            "http://localhost:3000".to_string(),
            &GraphQLOperationType::Query,
            "myQuery",
            None,
            vec![],
        )
        .unwrap();
        let ctx = Context {
            value: Value::from_json(json!({
              "foo": {
                "bar": "baz",
                "header": "abc"
              }
            }))
            .unwrap(),
            headers: Default::default(),
        };

        let req = tmpl.to_request(&ctx).unwrap();
        let body = req.body().unwrap().as_bytes().unwrap().to_owned();

        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            r#"{ "query": "query { myQuery { a,b,c } }" }"#
        );
    }

    #[test]
    fn test_query_with_args() {
        let tmpl = RequestTemplate::new(
            "http://localhost:3000".to_string(),
            &GraphQLOperationType::Mutation,
            "create",
            Some(
                serde_json::from_str(
                    r#"[{"key": "id", "value": "{{foo.bar}}"}, {"key": "struct", "value": "{{foo}}"}]"#,
                )
                .unwrap(),
            )
            .as_ref(),
            vec![],
        )
        .unwrap();
        let ctx = Context {
            value: Value::from_json(json!({
              "foo": {
                "bar": "baz",
                "header": "abc"
              }
            }))
            .unwrap(),
            headers: Default::default(),
        };

        let req = tmpl.to_request(&ctx).unwrap();
        let body = req.body().unwrap().as_bytes().unwrap().to_owned();

        assert_eq!(
            std::str::from_utf8(&body).unwrap(),
            r#"{ "query": "mutation { create(id: \"baz\", struct: {bar: \"baz\", header: \"abc\"}) { a,b,c } }" }"#
        );
    }

    fn create_gql_request_template_and_ctx(json: serde_json::Value) -> (RequestTemplate, Context) {
        let value = Value::from_json(json).unwrap();

        let tmpl = RequestTemplate::new(
            "http://localhost:3000".to_string(),
            &GraphQLOperationType::Mutation,
            "create",
            Some(
                serde_json::from_str(
                    r#"[{"key": "id", "value": "{{foo.bar}}"}, {"key": "struct", "value": "{{foo}}"}]"#,
                )
                    .unwrap(),
            )
                .as_ref(),
            vec![],
        )
            .unwrap();
        let ctx = Context { value, headers: Default::default() };

        (tmpl, ctx)
    }

    #[test]
    fn test_cache_key_collision() {
        let arr = [
            json!({
              "foo": {
                "bar": "baz",
                "header": "abc"
              }
            }),
            json!({
              "foo": {
                "bar": "baz",
                "header": "ab"
              }
            }),
            json!({
              "foo": {
                "bar": "ba",
                "header": "abc"
              }
            }),
            json!({
              "foo": {
                "bar": "abc",
                "header": "baz"
              }
            }),
        ];

        let cache_key_set: HashSet<_> = arr
            .iter()
            .cloned()
            .map(|value| {
                let (tmpl, ctx) = create_gql_request_template_and_ctx(value);
                tmpl.cache_key(&ctx)
            })
            .collect();

        assert_eq!(arr.len(), cache_key_set.len());
    }
}
