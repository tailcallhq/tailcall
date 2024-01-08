use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use crate::config::Upstream;
use crate::http::HttpClientOptions;

pub fn make_client(_: &Upstream, _: HttpClientOptions) -> ClientWithMiddleware {
    let builder = Client::builder();
    let client = ClientBuilder::new(builder.build().expect("Failed to build client"));
    client.build()
}