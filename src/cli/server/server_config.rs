use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use async_graphql_extension_apollo_tracing::ApolloTracing;

use crate::blueprint::telemetry::TelemetryExporter;
use crate::blueprint::{Blueprint, Http};
use crate::cli::runtime::init;
use crate::http::AppContext;
use crate::rest::EndpointSet;
use crate::schema_extension::SchemaExtension;

pub struct ServerConfig {
    pub blueprint: Blueprint,
    pub app_ctx: Arc<AppContext>,
}

impl ServerConfig {
    pub async fn new(blueprint: Blueprint, endpoints: EndpointSet) -> anyhow::Result<Self> {
        let mut rt = init(&blueprint.upstream, blueprint.server.script.clone());

        let mut extensions = vec![];

        if let Some(TelemetryExporter::Apollo(apollo)) = blueprint.opentelemetry.export.as_ref() {
            let (graph_id, variant) = apollo.graph_ref.split_once('@').unwrap();
            extensions.push(SchemaExtension::new(ApolloTracing::new(
                apollo.api_key.clone(),
                apollo.platform.clone(),
                graph_id.to_string(),
                variant.to_string(),
                apollo.version.clone(),
            )));
        }
        rt.add_extensions(extensions);

        let app_context = Arc::new(AppContext::new(blueprint.clone(), rt, endpoints.clone()));
        endpoints.validate(app_context.as_ref()).await?;

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

    pub fn graphiql(&self) -> bool {
        self.blueprint.server.enable_graphiql
    }
}
