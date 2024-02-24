use serde::{Deserialize, Serialize};
#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, schemars::JsonSchema,
)]
pub enum Method {
    #[default]
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
    OPTIONS,
    CONNECT,
    TRACE,
}

impl Method {
    pub fn into_reqwest(self) -> reqwest::Method {
        match self {
            Method::GET => reqwest::Method::GET,
            Method::POST => reqwest::Method::POST,
            Method::PUT => reqwest::Method::PUT,
            Method::PATCH => reqwest::Method::PATCH,
            Method::DELETE => reqwest::Method::DELETE,
            Method::HEAD => reqwest::Method::HEAD,
            Method::OPTIONS => reqwest::Method::OPTIONS,
            Method::CONNECT => reqwest::Method::CONNECT,
            Method::TRACE => reqwest::Method::TRACE,
        }
    }
}

#[cfg(test)]
mod test_method {
    use crate::http::Method;

    #[test]
    fn test() {
        assert_eq!(reqwest::Method::GET, Method::GET.into_reqwest());
        assert_eq!(reqwest::Method::POST, Method::POST.into_reqwest());
        assert_eq!(reqwest::Method::PUT, Method::PUT.into_reqwest());

        assert_eq!(reqwest::Method::PATCH, Method::PATCH.into_reqwest());
        assert_eq!(reqwest::Method::DELETE, Method::DELETE.into_reqwest());
        assert_eq!(reqwest::Method::HEAD, Method::HEAD.into_reqwest());

        assert_eq!(reqwest::Method::OPTIONS, Method::OPTIONS.into_reqwest());
        assert_eq!(reqwest::Method::CONNECT, Method::CONNECT.into_reqwest());
        assert_eq!(reqwest::Method::TRACE, Method::TRACE.into_reqwest());
    }
}
