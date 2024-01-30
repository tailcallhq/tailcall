use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::blueprint::{Blueprint, Http};
use crate::cli::{init_env, init_http, init_http2_only, init_in_memory_cache};
use crate::http::AppContext;

pub struct ServerConfig {
    pub blueprint: Blueprint,
    pub app_ctx: Arc<AppContext>,
}

impl ServerConfig {
    pub fn new(blueprint: Blueprint) -> Self {
        let h_client = init_http(&blueprint.upstream, blueprint.server.script.clone());
        let h2_client = init_http2_only(&blueprint.upstream, blueprint.server.script.clone());
        let env = init_env();
        let chrono_cache = Arc::new(init_in_memory_cache());
        let server_context = Arc::new(AppContext::new(
            blueprint.clone(),
            h_client,
            h2_client,
            env,
            chrono_cache,
        ));
        Self { app_ctx: server_context, blueprint }
    }

    pub fn addr(&self) -> SocketAddr {
        (self.blueprint.server.hostname, self.blueprint.server.port).into()
    }

    pub fn http_version(&self) -> String {
        match self.blueprint.server.http {
            Http::HTTP2 { cert: _, key: _ } => "HTTP/2".to_string(),
            _ => "HTTP/1.1".to_string(),
        }
    }

    pub fn graphiql_url(&self) -> String {
        let protocol = match self.http_version().as_str() {
            "HTTP/2" => "https",
            _ => "http",
        };
        let mut addr = self.addr();

        if addr.ip().is_unspecified() {
            addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), addr.port());
        }

        format!("{}://{}", protocol, addr)
    }

    pub fn graphiql(&self) -> bool {
        self.blueprint.server.enable_graphiql
    }
}
