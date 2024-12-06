/// Holds necessary information for request execution.
pub struct DynamicRequest<Value> {
    request: reqwest::Request,
    body_key: Option<Value>,
}

impl<Value> DynamicRequest<Value> {
    pub fn new(request: reqwest::Request) -> Self {
        Self { request, body_key: None }
    }

    pub fn with_body_key(self, body_key: Option<Value>) -> Self {
        Self { body_key, ..self }
    }

    pub fn request(&self) -> &reqwest::Request {
        &self.request
    }

    pub fn request_mut(&mut self) -> &mut reqwest::Request {
        &mut self.request
    }

    pub fn body_key(&self) -> Option<&Value> {
        self.body_key.as_ref()
    }

    pub fn into_request(self) -> reqwest::Request {
        self.request
    }

    pub fn into_body_key(self) -> Option<Value> {
        self.body_key
    }

    pub fn into_parts(self) -> (reqwest::Request, Option<Value>) {
        (self.request, self.body_key)
    }
}
