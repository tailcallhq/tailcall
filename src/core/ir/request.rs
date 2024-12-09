/// Holds necessary information for request execution.
pub struct DynamicRequest<Value> {
    request: reqwest::Request,
    /// used for request body batching.
    batching_value: Option<Value>,
}

impl<Value> DynamicRequest<Value> {
    pub fn new(request: reqwest::Request) -> Self {
        Self { request, batching_value: None }
    }

    pub fn with_batching_value(self, body_key: Option<Value>) -> Self {
        Self { batching_value: body_key, ..self }
    }

    pub fn request(&self) -> &reqwest::Request {
        &self.request
    }

    pub fn request_mut(&mut self) -> &mut reqwest::Request {
        &mut self.request
    }

    pub fn body_key(&self) -> Option<&Value> {
        self.batching_value.as_ref()
    }

    pub fn into_request(self) -> reqwest::Request {
        self.request
    }

    pub fn into_body_key(self) -> Option<Value> {
        self.batching_value
    }

    pub fn into_parts(self) -> (reqwest::Request, Option<Value>) {
        (self.request, self.batching_value)
    }
}
