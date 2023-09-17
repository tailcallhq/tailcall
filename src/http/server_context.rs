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
        ServerContext { schema: blueprint.to_schema(&server), client: HttpClient::new(server.clone()), server }
    }
}
