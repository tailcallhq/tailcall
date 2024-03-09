use async_graphql::{Name, Variables};
use async_graphql_value::ConstValue;

use crate::async_graphql_hyper::GraphQLRequest;

type Request = hyper::Request<hyper::Body>;

#[derive(Debug, PartialEq)]
pub struct PartialRequest<'a> {
    pub body: Option<&'a String>,
    pub graphql_query: &'a String,
    pub variables: Variables,
}

impl<'a> PartialRequest<'a> {
    pub async fn into_request(self, request: Request) -> anyhow::Result<GraphQLRequest> {
        let mut variables = self.variables;
        if let Some(key) = self.body {
            let bytes = hyper::body::to_bytes(request.into_body()).await?;
            let body: ConstValue = serde_json::from_slice(&bytes)?;
            variables.insert(Name::new(key), body);
        }

        Ok(GraphQLRequest(
            async_graphql::Request::new(self.graphql_query).variables(variables),
        ))
    }
}
