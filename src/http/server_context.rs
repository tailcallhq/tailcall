use async_graphql::dynamic;
use derive_setters::Setters;

use crate::{blueprint::Blueprint, config::Server, http::HttpClient};

#[derive(Clone, Setters)]
pub struct ServerContext {
    pub schema: dynamic::Schema,
    pub client: HttpClient,
    pub server: Server,
}

impl ServerContext {
    pub fn new(blueprint: Blueprint, server: Server) -> Self {
        let enable_http_cache = server.enable_http_cache();
        let enable_cache_control = server.enable_cache_control();
        let proxy = server.proxy.clone();
        ServerContext {
            schema: blueprint.to_schema(&server),
            client: HttpClient::new(enable_http_cache, proxy, enable_cache_control),
            server,
        }
    }
}
