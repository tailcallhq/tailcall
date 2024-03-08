

use super::endpoint::{self, Endpoint, PartialRequest};


type Request = hyper::Request<hyper::Body>;
#[derive(Default, Clone, Debug)]
pub struct EndpointSet {
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

    pub fn matches(&self, request: &Request) -> Option<PartialRequest> {
        self.endpoints.iter().find_map(|e| e.matches(request))
    }
}
