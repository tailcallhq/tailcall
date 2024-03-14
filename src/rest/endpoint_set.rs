use std::sync::{Arc, Mutex};

use super::endpoint::Endpoint;
use super::partial_request::PartialRequest;
use super::Request;
use crate::blueprint::Blueprint;
use crate::http::RequestContext;
use crate::rest::operation::OperationQuery;
use crate::runtime::TargetRuntime;
use crate::valid::Validator;

/// Collection of endpoints
#[derive(Default, Clone, Debug)]
pub struct EndpointSet {
    endpoints: Vec<Endpoint>,
}

impl Iterator for EndpointSet {
    type Item = Endpoint;

    fn next(&mut self) -> Option<Self::Item> {
        self.endpoints.pop()
    }
}

pub struct EndpointSetIter<'a> {
    inner: std::slice::Iter<'a, Endpoint>,
}

impl<'a> IntoIterator for &'a EndpointSet {
    type Item = &'a Endpoint;
    type IntoIter = EndpointSetIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        EndpointSetIter { inner: self.endpoints.iter() }
    }
}
impl<'a> Iterator for EndpointSetIter<'a> {
    type Item = &'a Endpoint;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl From<Endpoint> for EndpointSet {
    fn from(endpoint: Endpoint) -> Self {
        let mut set = EndpointSet::default();
        set.add_endpoint(endpoint);
        set
    }
}

impl EndpointSet {
    pub fn add_endpoint(&mut self, endpoint: Endpoint) {
        self.endpoints.push(endpoint);
    }

    pub fn try_new(operations: &str) -> anyhow::Result<EndpointSet> {
        let mut set = EndpointSet::default();

        for endpoint in Endpoint::try_new(operations)? {
            set.add_endpoint(endpoint);
        }

        Ok(set)
    }

    pub fn extend(&mut self, other: EndpointSet) {
        self.endpoints.extend(other.endpoints);
    }

    pub fn matches(&self, request: &Request) -> Option<PartialRequest> {
        self.endpoints.iter().find_map(|e| e.matches(request))
    }
    pub async fn validate(
        &self,
        blueprint: &Blueprint,
        runtime: TargetRuntime,
    ) -> anyhow::Result<()> {
        let mut operations = vec![];
        let req_ctx = Arc::new(RequestContext {
            server: Default::default(),
            upstream: Default::default(),
            req_headers: Default::default(),
            experimental_headers: Default::default(),
            cookie_headers: None,
            http_data_loaders: Arc::new(vec![]),
            gql_data_loaders: Arc::new(vec![]),
            grpc_data_loaders: Arc::new(vec![]),
            min_max_age: Arc::new(Mutex::new(None)),
            cache_public: Arc::new(Mutex::new(None)),
            runtime,
        });

        for endpoint in self {
            let req = endpoint.clone().into_request();
            let operation_qry = OperationQuery::new(req, String::new(), req_ctx.clone())?; // TODO fix trace
            operations.push(operation_qry);
        }
        super::operation::validate_operations(blueprint, operations)
            .await
            .to_result()?;
        Ok(())
    }
}
