use anyhow::Context;
use async_graphql::parser::types::ExecutableDocument;
use async_graphql::{Name, Variables};
use async_graphql_value::ConstValue;
use http_body_util::BodyExt;

use super::path::Path;
use super::Request;
use crate::core::async_graphql_hyper::GraphQLRequest;

/// A partial GraphQLRequest that contains a parsed executable GraphQL document.
#[derive(Debug)]
pub struct PartialRequest<'a> {
    pub body: Option<&'a String>,
    pub doc: &'a ExecutableDocument,
    pub variables: Variables,
    pub path: &'a Path,
}

impl<'a> PartialRequest<'a> {
    pub async fn into_request(self, request: Request) -> anyhow::Result<GraphQLRequest> {
        let mut variables = self.variables;
        if let Some(key) = self.body {
            let bytes = request
                .into_body()
                .frame()
                .await
                .context("Failed to read request body")??
                .into_data()
                .map_err(|e| anyhow::anyhow!("{e:?}"))?;
            let body: ConstValue = serde_json::from_slice(&bytes)?;
            variables.insert(Name::new(key), body);
        }

        let mut req = async_graphql::Request::new("").variables(variables);
        req.set_parsed_query(self.doc.clone());

        Ok(GraphQLRequest(req))
    }
}
