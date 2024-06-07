use std::collections::BTreeMap;
use std::str::FromStr;

use headers::HeaderValue;
use reqwest::header::HeaderName;
use rquickjs::{FromJs, IntoJs};

use super::create_header_map;
use crate::core::http::Response;
use crate::core::worker::*;

impl<'js> FromJs<'js> for Command {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let object = value.as_object().ok_or(rquickjs::Error::FromJs {
            from: value.type_name(),
            to: "rquickjs::Object",
            message: Some("unable to cast JS Value as object".to_string()),
        })?;

        if object.contains_key("request")? {
            Ok(Command::Request(WorkerRequest::from_js(
                ctx,
                object.get("request")?,
            )?))
        } else if object.contains_key("response")? {
            Ok(Command::Response(WorkerResponse::from_js(
                ctx,
                object.get("response")?,
            )?))
        } else {
            Err(rquickjs::Error::FromJs {
                from: "object",
                to: "tailcall::cli::javascript::request_filter::Command",
                message: Some("object must contain either request or response".to_string()),
            })
        }
    }
}

impl<'js> IntoJs<'js> for WorkerRequest {
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

impl<'js> FromJs<'js> for WorkerRequest {
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
        Ok(WorkerRequest(request))
    }
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

impl<'js> IntoJs<'js> for WorkerResponse {
    fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        let object = rquickjs::Object::new(ctx.clone())?;
        object.set("status", self.status())?;
        object.set("headers", self.headers())?;
        object.set("body", self.body())?;
        Ok(object.into_value())
    }
}

impl<'js> FromJs<'js> for WorkerResponse {
    fn from_js(_: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let object = value.as_object().ok_or(rquickjs::Error::FromJs {
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
        Ok(WorkerResponse(response))
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
    use reqwest::Request;
    use rquickjs::{Context, FromJs, IntoJs, Object, Runtime, String as JsString};

    use super::*;
    use crate::core::http::Response;
    use crate::core::worker::{Command, WorkerRequest, WorkerResponse};

    fn create_test_response() -> Result<WorkerResponse> {
        let mut headers = HeaderMap::new();
        headers.insert("content-type", "application/json".parse().unwrap());
        let response = crate::core::http::Response {
            status: reqwest::StatusCode::OK,
            headers,
            body: Bytes::from("Hello, World!"),
        };
        let js_response: Result<WorkerResponse> = response.try_into();
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
        let js_response = WorkerResponse(response);

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
            let response = WorkerResponse::from_js(&ctx, js_response).unwrap();

            assert_eq!(response.status(), reqwest::StatusCode::OK.as_u16());
            assert_eq!(response.body(), Some("Hello, World!".to_owned()));
            assert_eq!(
                response.headers().get("content-type"),
                Some(&"application/json".to_owned())
            );
        });
    }

    #[test]
    fn test_command_from_invalid_object() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let value = JsString::from_str(ctx.clone(), "invalid")
                .unwrap()
                .into_value();
            assert!(Command::from_js(&ctx, value).is_err());
        });
    }

    #[test]
    fn test_command_from_request() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let request =
                reqwest::Request::new(reqwest::Method::GET, "http://example.com/".parse().unwrap());
            let js_request: WorkerRequest = (&request).try_into().unwrap();
            let value = Object::new(ctx.clone()).unwrap();
            value.set("request", js_request.into_js(&ctx)).unwrap();
            assert!(Command::from_js(&ctx, value.into_value()).is_ok());
        });
    }

    #[test]
    fn test_command_from_response() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let js_response = WorkerResponse::try_from(Response {
                status: reqwest::StatusCode::OK,
                headers: reqwest::header::HeaderMap::default(),
                body: Bytes::new(),
            })
            .unwrap();
            let value = Object::new(ctx.clone()).unwrap();
            value.set("response", js_response).unwrap();
            assert!(Command::from_js(&ctx, value.into_value()).is_ok());
        });
    }

    #[test]
    fn test_command_from_arbitrary_object() {
        let runtime = Runtime::new().unwrap();
        let context = Context::base(&runtime).unwrap();
        context.with(|ctx| {
            let value = Object::new(ctx.clone()).unwrap();
            assert!(Command::from_js(&ctx, value.into_value()).is_err());
        });
    }

    #[test]
    fn test_reqwest_request_to_js_request() {
        let mut reqwest_request =
            reqwest::Request::new(reqwest::Method::GET, "http://example.com/".parse().unwrap());
        let _ = reqwest_request
            .body_mut()
            .insert(reqwest::Body::from("Hello, World!"));
        let js_request: WorkerRequest = (&reqwest_request).try_into().unwrap();
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

            let js_request: WorkerRequest = (&request).try_into().unwrap();
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

            let js_request: WorkerRequest = (&request).try_into().unwrap();
            let value = js_request.into_js(&ctx).unwrap();

            let js_request = WorkerRequest::from_js(&ctx, value).unwrap();

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
