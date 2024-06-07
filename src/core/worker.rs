use std::collections::BTreeMap;
use std::fmt::Display;

use hyper::body::Bytes;
use reqwest::Request;
use serde::{Deserialize, Serialize};

use crate::core::{is_default, Response};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub enum Scheme {
    #[default]
    Http,
    Https,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Uri {
    pub path: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub query: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub scheme: Scheme,
    #[serde(default, skip_serializing_if = "is_default")]
    pub host: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub port: Option<u16>,
}

#[derive(Debug)]
pub struct WorkerResponse(pub Response<String>);

#[derive(Debug)]
pub struct WorkerRequest(pub reqwest::Request);

#[derive(Debug)]
pub enum Event {
    Request(WorkerRequest),
}

#[derive(Debug)]
pub enum Command {
    Request(WorkerRequest),
    Response(WorkerResponse),
}

impl WorkerResponse {
    pub fn status(&self) -> u16 {
        self.0.status.as_u16()
    }

    pub fn headers(&self) -> BTreeMap<String, String> {
        let mut headers = BTreeMap::new();
        for (key, value) in self.0.headers.iter() {
            headers.insert(key.to_string(), value.to_str().unwrap().to_string());
        }
        headers
    }

    pub fn body(&self) -> Option<String> {
        let b = self.0.body.as_bytes();
        Some(String::from_utf8_lossy(b).to_string())
    }
}

impl TryFrom<WorkerResponse> for Response<Bytes> {
    type Error = anyhow::Error;

    fn try_from(res: WorkerResponse) -> Result<Self, Self::Error> {
        let res = res.0;
        Ok(Response {
            status: res.status,
            headers: res.headers,
            body: Bytes::from(res.body.as_bytes().to_vec()),
        })
    }
}

impl TryFrom<WorkerResponse> for Response<async_graphql::Value> {
    type Error = anyhow::Error;

    fn try_from(res: WorkerResponse) -> Result<Self, Self::Error> {
        let body: async_graphql::Value = match res.body() {
            Some(body) => serde_json::from_str(&body)?,
            None => async_graphql::Value::Null,
        };

        Ok(Response { status: res.0.status, headers: res.0.headers, body })
    }
}

impl TryFrom<Response<Bytes>> for WorkerResponse {
    type Error = anyhow::Error;

    fn try_from(res: Response<Bytes>) -> Result<Self, Self::Error> {
        let body = String::from_utf8_lossy(res.body.as_ref()).to_string();
        Ok(WorkerResponse(Response {
            status: res.status,
            headers: res.headers,
            body,
        }))
    }
}

impl TryFrom<Response<async_graphql::Value>> for WorkerResponse {
    type Error = anyhow::Error;

    fn try_from(res: Response<async_graphql::Value>) -> Result<Self, Self::Error> {
        let body = serde_json::to_string(&res.body)?;
        Ok(WorkerResponse(Response {
            status: res.status,
            headers: res.headers,
            body,
        }))
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

        write!(f, "{}://{}{}{}", scheme, host, port, path)?;

        if !query.is_empty() {
            write!(f, "?{}", query)?;
        }

        Ok(())
    }
}

impl WorkerRequest {
    pub fn uri(&self) -> Uri {
        self.0.url().into()
    }

    pub fn method(&self) -> String {
        self.0.method().to_string()
    }

    pub fn headers(&self) -> anyhow::Result<BTreeMap<String, String>> {
        let headers = self.0.headers();
        let mut map = BTreeMap::new();
        for (k, v) in headers.iter() {
            map.insert(k.to_string(), v.to_str()?.to_string());
        }
        Ok(map)
    }

    pub fn body(&self) -> Option<String> {
        if let Some(body) = self.0.body() {
            let bytes = body.as_bytes()?;
            Some(String::from_utf8_lossy(bytes).to_string())
        } else {
            None
        }
    }
}

impl TryFrom<&reqwest::Request> for WorkerRequest {
    type Error = anyhow::Error;

    fn try_from(value: &Request) -> Result<Self, Self::Error> {
        let request = value
            .try_clone()
            .ok_or(anyhow::anyhow!("unable to clone request"))?;
        Ok(WorkerRequest(request))
    }
}

impl From<WorkerRequest> for reqwest::Request {
    fn from(val: WorkerRequest) -> Self {
        val.0
    }
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
