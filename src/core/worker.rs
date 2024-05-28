use crate::core::Response;

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
