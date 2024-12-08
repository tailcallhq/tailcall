#![allow(clippy::too_many_arguments)]

use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use derive_setters::Setters;
use http::header::{HeaderMap, HeaderValue};
use tailcall_hasher::TailcallHasher;
use tracing::info;

use crate::core::config::{GraphQLOperationType, KeyValue};
use crate::core::has_headers::HasHeaders;
use crate::core::helpers::headers::MustacheHeaders;
use crate::core::http::Method::POST;
use crate::core::ir::model::{CacheKey, IoId};
use crate::core::ir::{GraphQLOperationContext, RelatedFields};
use crate::core::mustache::Mustache;
use crate::core::path::{PathGraphql, PathString};

/// Represents a GraphQL selection that can either be resolved or unresolved.
#[derive(Debug, Clone)]
pub enum Selection {
    /// A selection with a resolved string value.
    Resolved(String),
    /// A selection that contains a Mustache template to be resolved later.
    UnResolved(Mustache),
}

impl Selection {
    /// Resolves the `Unresolved` variant using the provided `PathString`.
    pub fn resolve(self, p: &impl PathString) -> Selection {
        match self {
            Selection::UnResolved(template) => Selection::Resolved(template.render(p)),
            resolved => resolved,
        }
    }
}

impl From<Mustache> for Selection {
    fn from(value: Mustache) -> Self {
        match value.is_const() {
            true => Selection::Resolved(value.to_string()),
            false => Selection::UnResolved(value),
        }
    }
}

/// RequestTemplate for GraphQL requests (See RequestTemplate documentation)
#[derive(Setters, Debug, Clone)]
pub struct RequestTemplate {
    // TODO: should be Mustache as for other templates
    pub url: String,
    pub operation_type: GraphQLOperationType,
    pub operation_name: String,
    pub operation_arguments: Option<Vec<(String, Mustache)>>,
    pub headers: MustacheHeaders,
    pub related_fields: RelatedFields,
    pub selection: Option<Selection>,
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

        let selection_set = match &self.selection {
            Some(Selection::Resolved(s)) => Cow::Borrowed(s),
            Some(Selection::UnResolved(u)) => Cow::Owned(u.to_string()),
            None => Cow::Owned(ctx.selection_set(&self.related_fields).unwrap_or_default()),
        };

        let mut operation = Cow::Borrowed(&self.operation_name);

        if let Some(args) = &self.operation_arguments {
            let args = args
                .iter()
                .filter_map(|(k, v)| {
                    let value = v.render_graphql(ctx);
                    if value.is_empty() {
                        None
                    } else {
                        Some(format!(r#"{}: {}"#, k, value.escape_default()))
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            if !args.is_empty() {
                let operation = operation.to_mut();

                operation.push('(');
                operation.push_str(&args);
                operation.push(')');
            }
        }

        if let Some(directives) = ctx.directives() {
            if !directives.is_empty() {
                let operation = operation.to_mut();

                operation.push(' ');
                operation.push_str(&directives.escape_default().to_string());
            }
        }

        let query =
            format!(r#"{{ "query": "{operation_type} {{ {operation} {selection_set} }}" }}"#);
        info!("Query {} ", query);
        query
    }

    pub fn new(
        url: String,
        operation_type: &GraphQLOperationType,
        operation_name: &str,
        args: Option<&Vec<KeyValue>>,
        headers: MustacheHeaders,
        related_fields: RelatedFields,
    ) -> anyhow::Result<Self> {
        let mut operation_arguments = None;

        if let Some(args) = args.as_ref() {
            operation_arguments = Some(
                args.iter()
                    .map(|kv| Ok((kv.key.to_owned(), Mustache::parse(&kv.value))))
                    .collect::<anyhow::Result<Vec<_>>>()?,
            );
        }

        Ok(Self {
            url,
            operation_type: operation_type.to_owned(),
            operation_name: operation_name.to_owned(),
            operation_arguments,
            headers,
            related_fields,
            selection: None,
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
    use http::header::HeaderMap;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    use crate::core::config::GraphQLOperationType;
    use crate::core::graphql::request_template::RelatedFields;
    use crate::core::graphql::RequestTemplate;
    use crate::core::has_headers::HasHeaders;
    use crate::core::ir::model::CacheKey;
    use crate::core::ir::GraphQLOperationContext;
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
        fn selection_set(&self, _: &RelatedFields) -> Option<String> {
            Some("{ a,b,c }".to_owned())
        }

        fn directives(&self) -> Option<String> {
            None
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
            RelatedFields::default(),
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
            RelatedFields::default(),
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
            RelatedFields::default(),
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
