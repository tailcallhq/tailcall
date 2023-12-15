use reqwest::{Method, Request as Req, Url};
use reqwest::header::HeaderValue;
use worker::{Method as WorkerMethod, Request as WorkerRequest};

pub fn to_reqwest(req: worker::Request) -> reqwest::Request {
    let method = match req.method() {
        WorkerMethod::Post => reqwest::Method::POST,
        _ => reqwest::Method::GET,
    };
    let url = req.url().to_string();
    let url = Url::parse(&url)?;
    let mut reqwest_request = reqwest::Request::new(method, url);
    for (name, value) in req.headers() {
        reqwest_request.headers_mut().insert(name.clone(),HeaderValue::from(value.clone()));
    }
    reqwest_request
}

pub fn to_worker(req: reqwest::Request) -> worker::Request {
    let method = match req.method() {
        &Method::POST => WorkerMethod::Post,
        _ => WorkerMethod::Get
    };
    let url = req.url().as_str();
    let worker_request = WorkerRequest::new(url,method);
    for (name, value) in req.headers().iter() {
        worker_request
            .headers_mut()
            .append(name, value.to_str().unwrap().to_owned());
    }
    worker_request.unwrap()
}
