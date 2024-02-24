use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use crate::app_context::AppContext;
use crate::blueprint::Http;
use crate::builder::TailcallExecutor;
use crate::runtime::TargetRuntime;
use crate::TailcallBuilder;

pub struct ServerConfig {
    pub tailcall_executor: TailcallExecutor,
}

impl ServerConfig {
    pub async fn new(
        tailcall_builder: TailcallBuilder,
        target_runtime: TargetRuntime,
    ) -> anyhow::Result<Self> {
        let blueprint = tailcall_builder.get_blueprint(&target_runtime).await?;
        let rt = crate::cli::runtime::init(&blueprint.upstream, blueprint.server.script.clone());
        let app_ctx = Arc::new(AppContext::new(blueprint, rt));
        let tailcall_executor = tailcall_builder.build_with_app_context(app_ctx);

        Ok(Self { tailcall_executor })
    }

    pub fn addr(&self) -> SocketAddr {
        (
            self.tailcall_executor.app_ctx.blueprint.server.hostname,
            self.tailcall_executor.app_ctx.blueprint.server.port,
        )
            .into()
    }

    pub fn http_version(&self) -> String {
        match self.tailcall_executor.app_ctx.blueprint.server.http {
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
        self.tailcall_executor
            .app_ctx
            .blueprint
            .server
            .enable_graphiql
    }
}
