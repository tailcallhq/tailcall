/// Holds necessary information for request execution.
pub struct RequestWrapper<Body> {
    request: reqwest::Request,
    deserialized_body: Option<Body>,
}

impl<Body> RequestWrapper<Body> {
    pub fn new(request: reqwest::Request) -> Self {
        Self { request, deserialized_body: None }
    }

    pub fn with_deserialzied_body(self, deserialized_body: Option<Body>) -> Self {
        Self { deserialized_body, ..self }
    }

    pub fn request(&self) -> &reqwest::Request {
        &self.request
    }

    pub fn request_mut(&mut self) -> &mut reqwest::Request {
        &mut self.request
    }

    pub fn deserialized_body(&self) -> Option<&Body> {
        self.deserialized_body.as_ref()
    }

    pub fn into_request(self) -> reqwest::Request {
        self.request
    }

    pub fn into_deserialized_body(self) -> Option<Body> {
        self.deserialized_body
    }

    pub fn into_parts(self) -> (reqwest::Request, Option<Body>) {
        (self.request, self.deserialized_body)
    }
}
