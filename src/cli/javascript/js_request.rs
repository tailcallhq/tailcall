use std::collections::BTreeMap;
use std::fmt::Display;

use hyper::body::Bytes;
use serde::{Deserialize, Serialize};

use super::create_header_map;
use crate::is_default;
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JsRequest {
    uri: Uri,
    method: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    body: Option<Bytes>,
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub enum Scheme {
    #[default]
    Http,
    Https,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Uri {
    path: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    query: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    scheme: Scheme,
    #[serde(default, skip_serializing_if = "is_default")]
    host: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    port: Option<u16>,
}

impl From<&reqwest::Url> for Uri {
    fn from(value: &reqwest::Url) -> Self {
        Self {
            path: value.path().to_string(),
            query: value.query_pairs().into_owned().collect(),
            scheme: match value.scheme() {
                "https" => Scheme::Https,
                _ => Scheme::Http,
            },
            host: value.host_str().map(|u| u.to_string()),
            port: value.port(),
        }
    }
}

impl Display for Uri {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let host = self.host.as_deref().unwrap_or("localhost");
        let port = self.port.map(|p| format!(":{}", p)).unwrap_or_default();
        let scheme = match self.scheme {
            Scheme::Https => "https",
            _ => "http",
        };
        let path = self.path.as_str();
        let query = self
            .query
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<String>>()
            .join("&");

        if !query.is_empty() {
            write!(f, "{}://{}:{}{}?{}", scheme, host, port, path, query)
        } else {
            write!(f, "{}://{}{}{}", scheme, host, port, path)
        }
    }
}

impl TryInto<reqwest::Request> for JsRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<reqwest::Request, Self::Error> {
        let mut request = reqwest::Request::new(
            reqwest::Method::from_bytes(self.method.as_bytes())?,
            self.uri.to_string().parse()?,
        );
        let headers = create_header_map(self.headers)?;
        request.headers_mut().extend(headers);
        if let Some(bytes) = self.body {
            let _ = request.body_mut().insert(reqwest::Body::from(bytes));
        }

        Ok(request)
    }
}

impl TryFrom<&reqwest::Request> for JsRequest {
    type Error = anyhow::Error;

    fn try_from(req: &reqwest::Request) -> Result<Self, Self::Error> {
        let url = Uri::from(req.url());
        let method = req.method().as_str().to_string();
        let headers = req
            .headers()
            .iter()
            .map(|(key, value)| {
                (
                    key.to_string(),
                    value.to_str().unwrap_or_default().to_string(),
                )
            })
            .collect::<BTreeMap<String, String>>();

        // NOTE: We don't pass body to worker for performance reasons
        Ok(JsRequest { uri: url, method, headers, body: None })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    impl Uri {
        pub fn parse(input: &str) -> anyhow::Result<Self> {
            Ok(Self::from(&reqwest::Url::parse(input)?))
        }
    }

    #[test]
    fn test_js_request_to_reqwest_request() {
        let body = "Hello, World!";
        let mut headers = BTreeMap::new();
        headers.insert("x-unusual-header".to_string(), "🚀".to_string());

        let js_request = JsRequest {
            uri: Uri::parse("http://example.com/").unwrap(),
            method: "GET".to_string(),
            headers,
            body: Some(Bytes::from(body)),
        };
        let reqwest_request: reqwest::Request = js_request.try_into().unwrap();
        assert_eq!(reqwest_request.method(), reqwest::Method::GET);
        assert_eq!(reqwest_request.url().as_str(), "http://example.com/");
        assert_eq!(
            reqwest_request.headers().get("x-unusual-header").unwrap(),
            "🚀"
        );
        let body_out = reqwest_request
            .body()
            .as_ref()
            .and_then(|body| body.as_bytes())
            .map(|a| String::from_utf8_lossy(a).to_string());
        assert_eq!(body_out, Some(body.to_string()));
    }

    #[test]
    fn test_reqwest_request_to_js_request() {
        let mut reqwest_request =
            reqwest::Request::new(reqwest::Method::GET, "http://example.com/".parse().unwrap());
        let _ = reqwest_request
            .body_mut()
            .insert(reqwest::Body::from("Hello, World!"));
        let js_request: JsRequest = (&reqwest_request).try_into().unwrap();
        assert_eq!(js_request.method, "GET");
        assert_eq!(js_request.uri.to_string(), "http://example.com/");
        let body_out = js_request.body;
        assert_eq!(body_out, None);
    }

    #[test]
    fn test_from_http_url_without_port() {
        let uri = Uri::parse("http://example.com/path?query=value").unwrap();
        assert_eq!(uri.scheme, Scheme::Http);
        assert_eq!(uri.host.unwrap(), "example.com");
        assert_eq!(uri.path, "/path");
        assert_eq!(uri.query.len(), 1);
        assert_eq!(uri.query.get("query").unwrap(), "value");
        assert_eq!(uri.port, None);
    }

    #[test]
    fn test_from_https_url_without_query() {
        let uri = Uri::parse("https://example.com:8085/path").unwrap();
        assert_eq!(uri.scheme, Scheme::Https);
        assert_eq!(uri.host.unwrap(), "example.com");
        assert_eq!(uri.path, "/path");
        assert_eq!(uri.query.len(), 0);
        assert_eq!(uri.port, Some(8085));
    }
}
