use std::collections::BTreeMap;
use std::fmt::Display;
use std::str::FromStr;

use headers::HeaderValue;
use reqwest::header::HeaderName;
use reqwest::Request;
use rquickjs::{FromJs, IntoJs};
use serde::{Deserialize, Serialize};
use tailcall::is_default;

#[derive(Debug)]
pub struct JsRequest(reqwest::Request);

impl JsRequest {
    fn uri(&self) -> Uri {
        self.0.url().into()
    }

    fn method(&self) -> String {
        self.0.method().to_string()
    }

    fn headers(&self) -> anyhow::Result<BTreeMap<String, String>> {
        let headers = self.0.headers();
        let mut map = BTreeMap::new();
        for (k, v) in headers.iter() {
            map.insert(k.to_string(), v.to_str()?.to_string());
        }
        Ok(map)
    }

    fn body(&self) -> Option<String> {
        if let Some(body) = self.0.body() {
            let bytes = body.as_bytes()?;
            Some(String::from_utf8_lossy(bytes).to_string())
        } else {
            None
        }
    }
}

impl TryFrom<&reqwest::Request> for JsRequest {
    type Error = anyhow::Error;

    fn try_from(value: &Request) -> Result<Self, Self::Error> {
        let request = value
            .try_clone()
            .ok_or(anyhow::anyhow!("unable to clone request"))?;
        Ok(JsRequest(request))
    }
}

impl From<JsRequest> for reqwest::Request {
    fn from(val: JsRequest) -> Self {
        val.0
    }
}

impl<'js> IntoJs<'js> for JsRequest {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let object = rquickjs::Object::new(ctx.clone())?;
        object.set("uri", self.uri())?;
        object.set("method", self.method())?;
        object.set(
            "headers",
            self.headers().map_err(|e| rquickjs::Error::FromJs {
                from: "HeaderMap",
                to: "BTreeMap<String, String>",
                message: Some(e.to_string()),
            })?,
        )?;
        object.set("body", self.body())?;
        Ok(object.into_value())
    }
}

impl<'js> FromJs<'js> for JsRequest {
    fn from_js(_: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let object = value.as_object().ok_or(rquickjs::Error::FromJs {
            from: value.type_name(),
            to: "rquickjs::Object",
            message: Some("unable to cast JS Value as object".to_string()),
        })?;
        let uri = object.get::<&str, Uri>("uri")?;
        let method = object.get::<&str, String>("method")?;
        let headers = object.get::<&str, BTreeMap<String, String>>("headers")?;
        let body = object.get::<&str, Option<String>>("body")?;
        let mut request = reqwest::Request::new(
            reqwest::Method::from_bytes(method.as_bytes()).map_err(|e| {
                rquickjs::Error::FromJs {
                    from: "string",
                    to: "Method",
                    message: Some(e.to_string()),
                }
            })?,
            uri.to_string()
                .parse()
                .map_err(|_| rquickjs::Error::FromJs {
                    from: "string",
                    to: "Url",
                    message: Some("unable to parse URL".to_string()),
                })?,
        );
        for (k, v) in headers {
            request.headers_mut().insert(
                HeaderName::from_str(&k).map_err(|e| rquickjs::Error::FromJs {
                    from: "string",
                    to: "HeaderName",
                    message: Some(e.to_string()),
                })?,
                HeaderValue::from_str(v.as_str()).map_err(|e| rquickjs::Error::FromJs {
                    from: "string",
                    to: "reqwest::header::HeaderValue",
                    message: Some(e.to_string()),
                })?,
            );
        }
        if let Some(body) = body {
            let _ = request.body_mut().insert(reqwest::Body::from(body));
        }
        Ok(JsRequest(request))
    }
}

#[derive(Serialize, Deserialize, Default, Debug, PartialEq, Eq)]
pub enum Scheme {
    #[default]
    Http,
    Https,
}

impl<'js> IntoJs<'js> for Scheme {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        match self {
            Scheme::Http => Ok(rquickjs::String::from_str(ctx.clone(), "http")?.into_value()),
            Scheme::Https => Ok(rquickjs::String::from_str(ctx.clone(), "https")?.into_value()),
        }
    }
}

impl<'js> FromJs<'js> for Scheme {
    fn from_js(_: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let as_string = value.as_string().ok_or(rquickjs::Error::FromJs {
            from: value.type_name(),
            to: "rquickjs::String",
            message: Some("unable to cast JS Value as string".to_string()),
        })?;

        let rs_string = as_string.to_string()?;
        if rs_string == "https" {
            Ok(Scheme::Https)
        } else if rs_string == "http" {
            Ok(Scheme::Http)
        } else {
            Err(rquickjs::Error::FromJs {
                from: "string",
                to: "tailcall::cli::javascript::js_request::Scheme",
                message: Some("scheme must be `http` or `https`".to_string()),
            })
        }
    }
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

impl<'js> IntoJs<'js> for Uri {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let object = rquickjs::Object::new(ctx.clone())?;
        object.set("path", self.path)?;
        object.set("query", self.query)?;
        object.set("scheme", self.scheme)?;
        object.set("host", self.host)?;
        object.set("port", self.port)?;
        Ok(object.into_value())
    }
}

impl<'js> FromJs<'js> for Uri {
    fn from_js(_: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let object = value.as_object().ok_or(rquickjs::Error::FromJs {
            from: value.type_name(),
            to: "rquickjs::Object",
            message: Some("unable to cast JS Value as object".to_string()),
        })?;
        let path = object.get::<&str, String>("path")?;
        let query = object.get::<&str, BTreeMap<String, String>>("query")?;
        let scheme = object.get::<&str, Scheme>("scheme")?;
        let host = object.get::<&str, Option<String>>("host")?;
        let port = object.get::<&str, Option<u16>>("port")?;

        Ok(Uri { path, query, scheme, host, port })
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

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rquickjs::{Context, Runtime};

    use super::*;

    #[test]
    fn test_reqwest_request_to_js_request() {
        let mut reqwest_request =
            reqwest::Request::new(reqwest::Method::GET, "http://example.com/".parse().unwrap());
        let _ = reqwest_request
            .body_mut()
            .insert(reqwest::Body::from("Hello, World!"));
        let js_request: JsRequest = (&reqwest_request).try_into().unwrap();
        assert_eq!(js_request.method(), "GET");
        assert_eq!(js_request.uri().to_string(), "http://example.com/");
        let body_out = js_request.body();
        assert_eq!(body_out, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_js_request_into_js() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let mut headers = BTreeMap::new();
            headers.insert("content-type".to_string(), "application/json".to_string());

            let mut request =
                Request::new(reqwest::Method::GET, "http://example.com/".parse().unwrap());
            let _ = request
                .body_mut()
                .insert(reqwest::Body::from("Hello, World!"));
            request.headers_mut().insert(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_str("application/json").unwrap(),
            );

            let js_request: JsRequest = (&request).try_into().unwrap();
            let value = js_request.into_js(&ctx).unwrap();
            let object = value.as_object().unwrap();

            let uri = object.get::<&str, Uri>("uri").unwrap();
            let method = object.get::<&str, String>("method").unwrap();
            let body = object.get::<&str, Option<String>>("body").unwrap();
            let js_headers = object
                .get::<&str, BTreeMap<String, String>>("headers")
                .unwrap();

            assert_eq!(uri.to_string(), "http://example.com/");
            assert_eq!(method, "GET");
            assert_eq!(body, Some("Hello, World!".to_string()));
            assert_eq!(
                js_headers.get("content-type"),
                Some(&"application/json".to_string())
            );
        });
    }

    #[test]
    fn test_js_request_from_js() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let mut headers = BTreeMap::new();
            headers.insert("content-type".to_string(), "application/json".to_string());

            let mut request =
                Request::new(reqwest::Method::GET, "http://example.com/".parse().unwrap());
            let _ = request
                .body_mut()
                .insert(reqwest::Body::from("Hello, World!"));
            request.headers_mut().insert(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_str("application/json").unwrap(),
            );

            let js_request: JsRequest = (&request).try_into().unwrap();
            let value = js_request.into_js(&ctx).unwrap();

            let js_request = JsRequest::from_js(&ctx, value).unwrap();

            assert_eq!(js_request.uri().to_string(), "http://example.com/");
            assert_eq!(js_request.method(), "GET");
            assert_eq!(js_request.body(), Some("Hello, World!".to_string()));
            assert_eq!(
                js_request.headers().unwrap().get("content-type"),
                Some(&"application/json".to_string())
            );
        });
    }
}
