#![allow(clippy::too_many_arguments)]

use std::collections::BTreeMap;

use derive_setters::Setters;
use hyper::HeaderMap;
use reqwest::header::{HeaderName, HeaderValue};

use crate::config::{GraphQLOperationType, KeyValues};
use crate::has_headers::HasHeaders;
use crate::http::Method::POST;
use crate::lambda::{GraphQLOperationContext, SelectionSetFilterData, UrlToObjFieldsMap};
use crate::mustache::{Mustache, Segment};
use crate::path::PathGraphql;

/// RequestTemplate for GraphQL requests (See RequestTemplate documentation)
/// TODO: add benchmarks for this
#[derive(Setters, Debug, Clone)]
pub struct GraphqlRequestTemplate {
  pub url: String,
  pub operation_type: GraphQLOperationType,
  pub operation_name: Option<String>,
  pub operation_arguments: Option<Vec<(String, Mustache)>>,
  pub headers: Vec<(HeaderName, Mustache)>,
  pub field_type: String,
  pub parent_type_name: String,
  pub field_name: String,
  pub filter_selection_set: bool,
  pub url_obj_fields: UrlToObjFieldsMap,
  pub url_obj_ids: BTreeMap<String, BTreeMap<String, Vec<String>>>,
}

impl GraphqlRequestTemplate {
  fn create_headers<C: PathGraphql>(&self, ctx: &C) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    for (k, v) in &self.headers {
      if let Ok(header_value) = HeaderValue::from_str(&v.render_graphql(ctx)) {
        header_map.insert(k, header_value);
      }
    }

    header_map
  }

  fn set_headers<C: PathGraphql + HasHeaders>(&self, mut req: reqwest::Request, ctx: &C) -> reqwest::Request {
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
    let obj_fields = match self.url_obj_fields.get(&self.url) {
      Some(obj_fields) => obj_fields.clone(),
      None => BTreeMap::new(),
    };
    let selection_set = ctx
      .selection_set(
        Some(SelectionSetFilterData {
          obj_name_to_fields_map: obj_fields,
          obj_name: self.field_type.clone(),
          url: self.url.clone(),
          url_obj_name_to_ids_map: self.url_obj_ids.clone(),
        }),
        self.filter_selection_set,
      )
      .unwrap_or_default();

    if self.operation_name.is_some() {
      let operation_type = &self.operation_type;

      let operation_name = self.operation_name.clone().unwrap_or_default();
      let operation = self
        .operation_arguments
        .as_ref()
        .map(|args| {
          args
            .iter()
            .map(|(k, v)| format!(r#"{}: {}"#, k, v.render_graphql(ctx).escape_default()))
            .collect::<Vec<_>>()
            .join(", ")
        })
        .map(|args| format!("{}({})", operation_name, args))
        .unwrap_or(operation_name);

      let graphql_query = format!(r#"{{ "query": "{operation_type} {{ {operation} {selection_set} }}" }}"#);
      req.body_mut().replace(graphql_query.into());
      req
    } else {
      // _entities query
      let typename = self.parent_type_name.clone();
      let typename_esc = self.parent_type_name.escape_default();
      let field_name = self.field_name.clone();
      let id = self
        .operation_arguments
        .as_ref()
        .map(|args| {
          args
            .first() // TODO verify that args are present if name is not specified.
            .unwrap_or(&("".to_string(), Mustache::from(vec![Segment::Literal("".to_string())])))
            .0
            .clone()
        })
        .unwrap_or_default();
      let arg_map = self.operation_arguments.as_ref().map_or_else(BTreeMap::new, |args| {
        BTreeMap::from_iter(args.iter().map(|(k, v)| (k, v.render_graphql(ctx))))
      });
      let id_value = arg_map
        .get(&id)
        .map(String::as_str)
        .unwrap_or_default()
        .escape_default();
      let operation = self
        .operation_arguments
        .as_ref()
        .and_then(|args| if args.len() > 1 { Some(&args[1..]) } else { None })
        .map(|args| {
          args
            .iter()
            .map(|(k, v)| format!(r#"{}: {}"#, k, v.render_graphql(ctx).escape_default()))
            .collect::<Vec<_>>()
            .join(", ")
        })
        .map(|args| format!("{}({})", field_name, args))
        .unwrap_or(field_name);

      let graphql_query = format!(
        r#"{{ "query": "query {{ _entities(representations: [ {{ __typename: {typename_esc}, {id}: {id_value} }} ]) {{ ... on {typename} {{ {operation} {selection_set} }} }} }}" }}"#,
      );
      req.body_mut().replace(graphql_query.into());
      req
    }
  }

  pub fn new(
    url: String,
    operation_type: &GraphQLOperationType,
    operation_name: &Option<String>,
    args: Option<&KeyValues>,
    headers: HeaderMap<HeaderValue>,
    field_type: String,
    parent_type_name: String,
    field_name: String,
    filter_selection_set: bool,
  ) -> anyhow::Result<Self> {
    let mut operation_arguments = None;

    if let Some(args) = args.as_ref() {
      operation_arguments = Some(
        args
          .iter()
          .map(|(k, v)| Ok((k.to_owned(), Mustache::parse(v)?)))
          .collect::<anyhow::Result<Vec<_>>>()?,
      );
    }

    let headers = headers
      .iter()
      .map(|(k, v)| Ok((k.clone(), Mustache::parse(v.to_str()?)?)))
      .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(Self {
      url,
      operation_type: operation_type.to_owned(),
      operation_name: operation_name.clone(),
      operation_arguments,
      headers,
      field_type,
      parent_type_name,
      field_name,
      filter_selection_set,
      url_obj_fields: BTreeMap::new(),
      url_obj_ids: BTreeMap::new(),
    })
  }

  pub fn is_entities_query(&self) -> bool {
    self.operation_name.is_none()
  }
}

#[cfg(test)]
mod tests {
  use async_graphql::Value;
  use hyper::HeaderMap;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  use crate::config::GraphQLOperationType;
  use crate::graphql_request_template::GraphqlRequestTemplate;
  use crate::has_headers::HasHeaders;
  use crate::json::JsonLike;
  use crate::lambda::{GraphQLOperationContext, SelectionSetFilterData};
  use crate::path::PathGraphql;

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
    fn selection_set(
      &self,
      _selection_set_filter: Option<SelectionSetFilterData>,
      _filter_selection_set: bool,
    ) -> Option<String> {
      Some("{ a,b,c }".to_owned())
    }
  }

  #[test]
  fn test_query_without_args() {
    let tmpl = GraphqlRequestTemplate::new(
      "http://localhost:3000".to_string(),
      &GraphQLOperationType::Query,
      &Some("myQuery".to_string()),
      None,
      HeaderMap::new(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      false,
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
    let tmpl = GraphqlRequestTemplate::new(
      "http://localhost:3000".to_string(),
      &GraphQLOperationType::Mutation,
      &Some("create".to_string()),
      Some(
        serde_json::from_str(r#"[{"key": "id", "value": "{{foo.bar}}"}, {"key": "struct", "value": "{{foo}}"}]"#)
          .unwrap(),
      )
      .as_ref(),
      HeaderMap::new(),
      "".to_string(),
      "".to_string(),
      "".to_string(),
      false,
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
      r#"{ "query": "mutation { create(id: \"baz\", struct: {bar: \"baz\",header: \"abc\"}) { a,b,c } }" }"#
    );
  }
}
