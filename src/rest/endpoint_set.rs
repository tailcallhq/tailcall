use async_graphql::Variables;
use reqwest::Request;

use super::endpoint::{self, Endpoint};
use crate::async_graphql_hyper::GraphQLRequest;

#[derive(Default)]
struct EndpointSet {
    endpoints: Vec<super::endpoint::Endpoint>,
}

impl From<Endpoint> for EndpointSet {
    fn from(endpoint: Endpoint) -> Self {
        let mut set = EndpointSet::default();
        set.add_endpoint(endpoint);
        set
    }
}

impl EndpointSet {
    pub fn add_endpoint(&mut self, endpoint: endpoint::Endpoint) {
        self.endpoints.push(endpoint);
    }

    pub fn try_new(operations: &str) -> anyhow::Result<EndpointSet> {
        let mut set = EndpointSet::default();

        for endpoint in endpoint::Endpoint::try_new(operations)? {
            set.add_endpoint(endpoint);
        }

        Ok(set)
    }

    pub fn extend(mut self, other: EndpointSet) -> EndpointSet {
        self.endpoints.extend(other.endpoints);
        self
    }

    pub fn matches(&self, request: &Request) -> Option<Variables> {
        self.endpoints.iter().find_map(|e| e.matches(request))
    }

    pub fn eval(&self, request: &Request) -> anyhow::Result<Option<GraphQLRequest>> {
        for endpoint in &self.endpoints {
            if let Some(request) = endpoint.eval(request)? {
                return Ok(Some(request));
            }
        }

        Ok(None)
    }
}
