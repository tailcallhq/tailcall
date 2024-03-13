use super::endpoint::Endpoint;
use super::partial_request::PartialRequest;
use super::Request;

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
}
