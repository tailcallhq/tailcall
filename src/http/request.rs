use std::str::FromStr;

use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Method, Url};
use worker::{Method as WorkerMethod, Request as WorkerRequest};

pub fn to_reqwest(req: worker::Request) -> reqwest::Request {
  let method = match req.method() {
    WorkerMethod::Post => Method::POST,
    _ => Method::GET,
  };
  let url = req.url().unwrap().to_string();
  let url = Url::parse(&url).unwrap();
  let mut reqwest_request = reqwest::Request::new(method, url);
  for (name, value) in req.headers() {
    reqwest_request.headers_mut().append(
      HeaderName::from_str(&name).unwrap(),
      HeaderValue::from_str(&value).unwrap(),
    );
  }
  reqwest_request
}

pub fn convert_method(worker_method: &Method) -> worker::Method {
  let method_str = &*worker_method.to_string().to_uppercase();

  match method_str {
    "GET" => Ok(worker::Method::Get),
    "POST" => Ok(worker::Method::Post),
    "PUT" => Ok(worker::Method::Put),
    "DELETE" => Ok(worker::Method::Delete),
    "HEAD" => Ok(worker::Method::Head),
    "OPTIONS" => Ok(worker::Method::Options),
    "PATCH" => Ok(worker::Method::Patch),
    "CONNECT" => Ok(worker::Method::Connect),
    "TRACE" => Ok(worker::Method::Trace),
    _ => Err("Unsupported HTTP method"),
  }
  .unwrap()
}

pub fn to_worker(req: &reqwest::Request) -> worker::Request {
  let method = match req.method() {
    &Method::POST => WorkerMethod::Post,
    _ => WorkerMethod::Get,
  };
  let url = req.url().as_str();

  /*for (name, value) in req.headers().iter() {
    worker_request
      .head()
      .unwrap()
      .append(name.as_str(), value.to_str().unwrap())
      .unwrap();
  }*/
  WorkerRequest::new(url, method).unwrap()
}
