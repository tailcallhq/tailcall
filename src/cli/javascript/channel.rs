use std::io::Read;
use std::str::FromStr;

use hyper::body::Bytes;
use hyper::HeaderMap;
use mini_v8::{Object, Value};
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Body, Request};

use crate::http::Response;

pub struct Message {
    pub message: MessageContent,
    pub id: Option<u64>,
}

pub enum MessageContent {
    Request(Request),
    Response(Response<Bytes>),
}

impl Message {
    pub fn to_mv8(self, v8: &mini_v8::MiniV8) -> mini_v8::Result<mini_v8::Value> {
        let v8 = v8.clone();
        match self.message {
            MessageContent::Request(request) => {
                let obj = v8.clone().create_object();
                let req_url = request.url().clone();
                let req_headers = request.headers().clone();

                obj.set("type", v8.clone().create_string("request"))?;

                if let Some(id) = self.id {
                    obj.set("id", mini_v8::Value::Number(id as f64))?;
                }

                obj.set(
                    "method",
                    v8.clone().create_string(request.method().as_str()),
                )?;

                obj.set(
                    "url",
                    v8.clone().create_function({
                        let v8 = v8.clone();
                        move |_| {
                            let uri = req_url.clone().to_string();
                            Ok(v8.clone().create_string(uri.as_str()))
                        }
                    }),
                )?;

                obj.set("headers", {
                    let headers = v8.clone().create_object();
                    headers.set(
                        "get",
                        v8.clone().create_function({
                            let v8 = v8.clone();

                            move |inv| {
                                let key = inv.args.get(0);
                                let key = key.as_string();
                                if let Some(key) = key {
                                    let value =
                                        req_headers.get(key.to_string()).unwrap().to_str().unwrap();
                                    Ok(mini_v8::Value::String(v8.create_string(value)))
                                } else {
                                    Ok(mini_v8::Value::Null)
                                }
                            }
                        }),
                    )?;

                    headers
                })?;

                obj.set(
                    "body",
                    v8.clone().create_function({
                        let v8 = v8.clone();
                        move |_| {
                            let bytes = request.body().and_then(|body| body.as_bytes());
                            if let Some(bytes) = bytes {
                                let bytes_array = v8.create_array();
                                for byte in bytes {
                                    bytes_array.push(mini_v8::Value::Number(*byte as f64))?;
                                }
                                Ok(mini_v8::Value::Array(bytes_array))
                            } else {
                                Ok(mini_v8::Value::Null)
                            }
                        }
                    }),
                )?;
                Ok(mini_v8::Value::Object(obj))
            }
            MessageContent::Response(response) => {
                let obj = v8.clone().create_object();
                let res_status = response.status;
                let res_headers = response.headers.clone();

                obj.set("type", v8.clone().create_string("response"))?;

                if let Some(id) = self.id {
                    obj.set("id", mini_v8::Value::Number(id as f64))?;
                }

                obj.set("status", mini_v8::Value::Number(res_status.as_u16() as f64))?;

                obj.set("headers", {
                    let headers = v8.clone().create_object();
                    headers.set(
                        "get",
                        v8.clone().create_function({
                            let v8 = v8.clone();

                            move |inv| {
                                let key = inv.args.get(0);
                                let key = key.as_string();
                                if let Some(key) = key {
                                    let value =
                                        res_headers.get(key.to_string()).unwrap().to_str().unwrap();
                                    Ok(mini_v8::Value::String(v8.create_string(value)))
                                } else {
                                    Ok(mini_v8::Value::Null)
                                }
                            }
                        }),
                    )?;

                    headers
                })?;

                obj.set(
                    "body",
                    v8.clone().create_function({
                        let v8 = v8.clone();
                        move |_| {
                            let bytes = response.body.bytes();
                            let bytes_array = v8.create_array();
                            for byte in bytes {
                                let byte = byte.unwrap(); // FIXME: remove unwrap
                                bytes_array.push(mini_v8::Value::Number(byte as f64))?;
                            }
                            Ok(mini_v8::Value::Array(bytes_array))
                        }
                    }),
                )?;
                Ok(mini_v8::Value::Object(obj))
            }
        }
    }

    pub fn from_mv8(value: mini_v8::Value) -> anyhow::Result<Self> {
        let wrapper = value
            .as_object()
            .ok_or(anyhow::anyhow!("expected an object"))?;
        let id = wrapper
            .get::<&str, mini_v8::Value>("id")
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let id = id.as_number().map(|n| n as u64);
        let message = wrapper
            .get::<&str, mini_v8::Value>("message")
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        let message = message
            .as_object()
            .ok_or(anyhow::anyhow!("expected an object"))?;
        if let Ok(request) = message.get::<&str, mini_v8::Value>("request") {
            let request = Self::request_from_v8(request)?;
            Ok(Message { message: MessageContent::Request(request), id })
        } else if let Ok(response) = message.get::<&str, mini_v8::Value>("response") {
            let response = Self::response_from_v8(response)?;
            Ok(Message { message: MessageContent::Response(response), id })
        } else {
            Err(anyhow::anyhow!("expected a request or response"))
        }
    }
    fn response_from_v8(value: mini_v8::Value) -> anyhow::Result<Response<Bytes>> {
        let response = value
            .as_object()
            .ok_or(anyhow::anyhow!("expected an object"))?;
        let status_value = response
            .get::<&str, mini_v8::Value>("status")
            .map_err(|e| anyhow::anyhow!(format!("error getting status: {}", e.to_string())))?;
        let status = status_value
            .as_number()
            .ok_or(anyhow::anyhow!("expected a number"))?;
        let status = reqwest::StatusCode::from_u16(status as u16)?;
        let header_map = Self::headers_from_v8(response)?;
        let body_bytes = Self::body_from_v8(response)?;
        let response = Response { status, headers: header_map, body: Bytes::from(body_bytes) };
        Ok(response)
    }

    fn headers_from_v8(obj: &Object) -> anyhow::Result<HeaderMap> {
        let headers_value = obj
            .get::<&str, mini_v8::Value>("headers")
            .map_err(|e| anyhow::anyhow!(format!("error getting headers: {}", e.to_string())))?;
        let headers = headers_value
            .as_object()
            .ok_or(anyhow::anyhow!("expected an object"))?;
        let mut header_map = reqwest::header::HeaderMap::new();
        if let Ok(mut headers) = headers.clone().properties::<String, String>(false) {
            while let Some(Ok((key, value))) = headers.next() {
                header_map.insert(
                    HeaderName::from_str(key.clone().as_str())?,
                    HeaderValue::from_str(value.as_str())?,
                );
            }
        }
        Ok(header_map)
    }
    fn request_from_v8(request: Value) -> anyhow::Result<Request> {
        let request = request
            .as_object()
            .ok_or(anyhow::anyhow!("expected an object"))?;
        let method_value = request
            .get::<&str, mini_v8::Value>("method")
            .map_err(|e| anyhow::anyhow!(format!("error getting method: {}", e.to_string())))?;
        let method = method_value
            .as_string()
            .ok_or(anyhow::anyhow!("expected a string"))?;
        let method = reqwest::Method::from_str(method.to_string().as_str())?;
        let url_value = request
            .get::<&str, mini_v8::Value>("url")
            .map_err(|e| anyhow::anyhow!(format!("error getting url: {}", e.to_string())))?;
        let url = url_value
            .as_string()
            .ok_or(anyhow::anyhow!("expected a string"))?
            .to_string();
        let url = reqwest::Url::from_str(url.as_str())?;
        let header_map = Self::headers_from_v8(request)?;

        let body_bytes = Self::body_from_v8(request)?;
        let mut request = Request::new(method, url);
        request.headers_mut().extend(header_map);
        request.body_mut().replace(Body::from(body_bytes));
        Ok(request)
    }

    fn body_from_v8(obj: &Object) -> anyhow::Result<Vec<u8>> {
        let body_value = obj
            .get::<&str, mini_v8::Value>("body")
            .map_err(|e| anyhow::anyhow!(format!("error getting body: {}", e.to_string())))?;
        let body = body_value
            .as_string()
            .ok_or(anyhow::anyhow!("expected an array"))?;
        let body_bytes = body.to_string();
        let body = body_bytes.as_bytes();
        Ok(body.to_vec())
    }
}
