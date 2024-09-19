#[derive(Default, Clone, Debug)]
/// User can configure the hooks on directive
/// for the requests.
pub struct JsHooks {
    pub on_request: Option<String>,
    pub on_response: Option<String>,
}

impl JsHooks {
    pub fn new(
        on_request: Option<String>,
        on_response: Option<String>,
    ) -> Result<Self, &'static str> {
        if on_request.is_none() && on_response.is_none() {
            Err("At least one of on_request or on_response must be present")
        } else {
            Ok(JsHooks { on_request, on_response })
        }
    }
}
