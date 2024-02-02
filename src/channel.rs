use std::io::Read;

use hyper::body::Bytes;
use reqwest::Request;

use crate::http::{Response};

pub struct Message {
    pub message: MessageContent,
    pub id: Option<u64>,
}

pub enum MessageContent {
    Request(Request),
    Response(Response<Bytes>),
}

impl Message {
    pub fn to_v8(self, v8: &mini_v8::MiniV8) -> mini_v8::Result<mini_v8::Value> {
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

    pub fn from_v8(_value: mini_v8::Value) -> anyhow::Result<Self> {
        todo!()
    }
}
