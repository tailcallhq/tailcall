/// Holds necessary information for request execution.
pub struct RequestWrapper<Body> {
    request: reqwest::Request,
    deserialized_body: Body,
}

impl<Body> RequestWrapper<Body> {
    pub fn new(request: reqwest::Request, body: Body) -> Self {
        Self { request, deserialized_body: body }
    }

    pub fn request(&self) -> &reqwest::Request {
        &self.request
    }

    pub fn request_mut(&mut self) -> &mut reqwest::Request {
        &mut self.request
    }

    pub fn deserialized_body(&self) -> &Body {
        &self.deserialized_body
    }

    pub fn into_request(self) -> reqwest::Request {
        self.request
    }

    pub fn into_deserialized_body(self) -> Body {
        self.deserialized_body
    }

    pub fn into_parts(self) -> (reqwest::Request, Body) {
        (self.request, self.deserialized_body)
    }
}
