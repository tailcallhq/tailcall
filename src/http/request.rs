use std::str::FromStr;
use reqwest::{Method, Request as Req, Url};
use reqwest::header::{HeaderName, HeaderValue};
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
        reqwest_request.headers_mut().append(HeaderName::from_str(&name).unwrap(),HeaderValue::from_str(&value).unwrap());
    }
    reqwest_request
}

pub fn to_worker(req: reqwest::Request) -> worker::Request {
    let method = match req.method() {
        &Method::POST => WorkerMethod::Post,
        _ => WorkerMethod::Get
    };
    let url = req.url().as_str();
    let mut worker_request = WorkerRequest::new(url,method).unwrap();
    for (name, value) in req.headers().iter() {
        worker_request
            .headers_mut().unwrap()
            .append(name.as_str(), value.to_str().unwrap()).unwrap();
    }
    worker_request
}
