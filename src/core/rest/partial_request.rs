use async_graphql::parser::types::ExecutableDocument;
use async_graphql::{Name, Variables};
use async_graphql_value::ConstValue;

use super::path::Path;
use crate::core::async_graphql_hyper::GraphQLRequest;
use crate::core::http::Request;

/// A partial GraphQLRequest that contains a parsed executable GraphQL document.
#[derive(Debug)]
pub struct PartialRequest<'a> {
    pub body: Option<&'a String>,
    pub doc: &'a ExecutableDocument,
    pub variables: Variables,
    pub path: &'a Path,
}

impl<'a> PartialRequest<'a> {
    pub fn into_request(self, request: Request) -> anyhow::Result<GraphQLRequest> {
        let mut variables = self.variables;
        if let Some(key) = self.body {
            let bytes = request.body;
            let body: ConstValue = serde_json::from_slice(&bytes)?;
            variables.insert(Name::new(key), body);
        }

        let mut req = async_graphql::Request::new("").variables(variables);
        req.set_parsed_query(self.doc.clone());

        Ok(GraphQLRequest(req))
    }
}
