use std::sync::Arc;

use hyper::body::Bytes;
use rquickjs::FromJs;

use super::{JsRequest, JsResponse};
use crate::core::http::Response;
use crate::core::{HttpIO, WorkerIO};

#[derive(Debug)]
pub enum Event {
    Request(JsRequest),
}

#[derive(Debug)]
pub enum Command {
    Request(JsRequest),
    Response(JsResponse),
}

impl<'js> FromJs<'js> for Command {
    fn from_js(ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
        let object = value.as_object().ok_or(rquickjs::Error::FromJs {
            from: value.type_name(),
            to: "rquickjs::Object",
            message: Some("unable to cast JS Value as object".to_string()),
        })?;

        if object.contains_key("request")? {
            Ok(Command::Request(JsRequest::from_js(
                ctx,
                object.get("request")?,
            )?))
        } else if object.contains_key("response")? {
            Ok(Command::Response(JsResponse::from_js(
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

pub struct RequestFilter {
    worker: Arc<dyn WorkerIO<Event, Command>>,
    client: Arc<dyn HttpIO>,
}

impl RequestFilter {
    pub fn new(
        client: Arc<impl HttpIO + Send + Sync + 'static>,
        worker: Arc<impl WorkerIO<Event, Command>>,
    ) -> Self {
        Self { worker, client }
    }

    #[async_recursion::async_recursion]
    async fn on_request(&self, mut request: reqwest::Request) -> anyhow::Result<Response<Bytes>> {
        let js_request = JsRequest::try_from(&request)?;
        let event = Event::Request(js_request);
        let command = self.worker.call("onRequest".to_string(), event).await?;
        match command {
            Some(command) => match command {
                Command::Request(js_request) => {
                    let response = self.client.execute(js_request.into()).await?;
                    Ok(response)
                }
                Command::Response(js_response) => {
                    // Check if the response is a redirect
                    if (js_response.status() == 301 || js_response.status() == 302)
                        && js_response.headers().contains_key("location")
                    {
                        request
                            .url_mut()
                            .set_path(js_response.headers()["location"].as_str());
                        self.on_request(request).await
                    } else {
                        Ok(js_response.try_into()?)
                    }
                }
            },
            None => self.client.execute(request).await,
        }
    }
}

#[async_trait::async_trait]
impl HttpIO for RequestFilter {
    async fn execute(
        &self,
        request: reqwest::Request,
    ) -> anyhow::Result<Response<hyper::body::Bytes>> {
        self.on_request(request).await
    }
}

#[cfg(test)]
mod tests {
    use hyper::body::Bytes;
    use rquickjs::{Context, FromJs, IntoJs, Object, Runtime, String as JsString};

    use crate::cli::javascript::request_filter::Command;
    use crate::cli::javascript::{JsRequest, JsResponse};
    use crate::core::http::Response;

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
            let js_request: JsRequest = (&request).try_into().unwrap();
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
            let js_response = JsResponse::try_from(Response {
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
}
