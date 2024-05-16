#[derive(Default, Clone, Debug)]
/// User can configure the filter/interceptor
/// for the http requests.
pub struct HttpFilter {
    pub on_request: Option<String>,
}
