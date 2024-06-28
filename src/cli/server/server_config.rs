use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use async_graphql_extension_apollo_tracing::ApolloTracing;

use crate::cli::runtime::init;
use crate::core::app_context::AppContext;
use crate::core::blueprint::telemetry::TelemetryExporter;
use crate::core::blueprint::{Blueprint, Http};
use crate::core::rest::{EndpointSet, Unchecked};
use crate::core::schema_extension::SchemaExtension;

pub struct ServerConfig {
    pub blueprint: Blueprint,
    pub app_ctx: Arc<AppContext>,
}

impl ServerConfig {
    pub async fn new(
        blueprint: Blueprint,
        endpoints: EndpointSet<Unchecked>,
    ) -> anyhow::Result<Self> {
        let mut rt = init(&blueprint);

        let mut extensions = vec![];

        if let Some(TelemetryExporter::Apollo(apollo)) = blueprint.telemetry.export.as_ref() {
            let (graph_id, variant) = apollo.graph_ref.split_once('@').unwrap();
            extensions.push(SchemaExtension::new(ApolloTracing::new(
                apollo.api_key.clone(),
                apollo.platform.clone().unwrap_or_default(),
                graph_id.to_string(),
                variant.to_string(),
                apollo.version.clone().unwrap_or_default(),
            )));
        }
        rt.add_extensions(extensions);

        let endpoints = endpoints.into_checked(&blueprint, rt.clone()).await?;
        let app_context = Arc::new(AppContext::new(blueprint.clone(), rt, endpoints));

        Ok(Self { app_ctx: app_context, blueprint })
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
}
