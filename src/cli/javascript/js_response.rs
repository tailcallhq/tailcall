use std::collections::BTreeMap;

use hyper::body::Bytes;
use rquickjs::{FromJs, IntoJs};

use super::create_header_map;
use crate::core::http::Response;

#[derive(Debug)]
pub struct JsResponse(Response<String>);

impl JsResponse {
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

impl<'js> IntoJs<'js> for JsResponse {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let object = rquickjs::Object::new(ctx.clone())?;
        object.set("status", self.status())?;
        object.set("headers", self.headers())?;
        object.set("body", self.body())?;
        Ok(object.into_value())
    }
}

impl<'js> FromJs<'js> for JsResponse {
    fn from_js(_: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let object = value.as_object().ok_or_else(|| rquickjs::Error::FromJs {
            from: value.type_name(),
            to: "rquickjs::Object",
            message: Some("unable to cast JS Value as object".to_string()),
        })?;
        let status = object.get::<&str, u16>("status")?;
        let headers = object.get::<&str, BTreeMap<String, String>>("headers")?;
        let body = object.get::<&str, Option<String>>("body")?;
        let response = Response {
            status: reqwest::StatusCode::from_u16(status).map_err(|_| rquickjs::Error::FromJs {
                from: "u16",
                to: "reqwest::StatusCode",
                message: Some("invalid status code".to_string()),
            })?,
            headers: create_header_map(headers).map_err(|e| rquickjs::Error::FromJs {
                from: "BTreeMap<String, String>",
                to: "reqwest::header::HeaderMap",
                message: Some(e.to_string()),
            })?,
            body: body.unwrap_or_default(),
        };
        Ok(JsResponse(response))
    }
}

impl TryFrom<JsResponse> for Response<Bytes> {
    type Error = anyhow::Error;

    fn try_from(res: JsResponse) -> Result<Self, Self::Error> {
        let res = res.0;
        Ok(Response {
            status: res.status,
            headers: res.headers,
            body: Bytes::from(res.body.as_bytes().to_vec()),
        })
    }
}

impl TryFrom<Response<Bytes>> for JsResponse {
    type Error = anyhow::Error;

    fn try_from(res: Response<Bytes>) -> Result<Self, Self::Error> {
        let body = String::from_utf8_lossy(res.body.as_ref()).to_string();
        Ok(JsResponse(Response {
            status: res.status,
            headers: res.headers,
            body,
        }))
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use anyhow::Result;
    use headers::{HeaderName, HeaderValue};
    use hyper::body::Bytes;
    use pretty_assertions::assert_eq;
    use reqwest::header::HeaderMap;
    use rquickjs::{Context, FromJs, IntoJs, Runtime};

    use super::JsResponse;

    fn create_test_response() -> Result<JsResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        let response = crate::core::http::Response {
            status: reqwest::StatusCode::OK,
            headers,
            body: Bytes::from("Hello, World!"),
        };
        let js_response: Result<JsResponse> = response.try_into();
        js_response
    }

    #[test]
    fn test_to_js_response() {
        let js_response = create_test_response();
        assert!(js_response.is_ok());
        let js_response = js_response.unwrap();
        assert_eq!(js_response.status(), 200);
        assert_eq!(
            js_response.headers().get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(js_response.body(), Some("Hello, World!".into()));
    }

    #[test]
    fn test_from_js_response() {
        let js_response = create_test_response().unwrap();
        let response: Result<crate::core::http::Response<Bytes>> = js_response.try_into();
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.status, reqwest::StatusCode::OK);
        assert_eq!(
            response.headers.get("content-type").unwrap(),
            "application/json"
        );
        assert_eq!(response.body, Bytes::from("Hello, World!"));
    }

    #[test]
    fn test_unusual_headers() {
        let body = "a";
        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-unusual-header"),
            HeaderValue::from_str("🚀").unwrap(),
        );
        let response = crate::core::http::Response {
            status: reqwest::StatusCode::OK,
            headers,
            body: body.into(),
        };
        let js_response = JsResponse(response);

        let response: Result<crate::core::http::Response<Bytes>, _> = js_response.try_into();
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.headers.get("x-unusual-header").unwrap(), "🚀");
        assert_eq!(response.body, Bytes::from(body));
    }

    #[test]
    fn test_response_into_js() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let value = create_test_response().unwrap().into_js(&ctx).unwrap();
            let object = value.as_object().unwrap();

            let status = object.get::<&str, u16>("status").unwrap();
            let headers = object
                .get::<&str, BTreeMap<String, String>>("headers")
                .unwrap();
            let body = object.get::<&str, Option<String>>("body").unwrap();

            assert_eq!(status, reqwest::StatusCode::OK);
            assert_eq!(body, Some("Hello, World!".to_owned()));
            assert!(headers.contains_key("content-type"));
            assert_eq!(
                headers.get("content-type"),
                Some(&"application/json".to_owned())
            );
        });
    }

    #[test]
    fn test_response_from_js() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let js_response = create_test_response().unwrap().into_js(&ctx).unwrap();
            let response = JsResponse::from_js(&ctx, js_response).unwrap();

            assert_eq!(response.status(), reqwest::StatusCode::OK.as_u16());
            assert_eq!(response.body(), Some("Hello, World!".to_owned()));
            assert_eq!(
                response.headers().get("content-type"),
                Some(&"application/json".to_owned())
            );
        });
    }
}
