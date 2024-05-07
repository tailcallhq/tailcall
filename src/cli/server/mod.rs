pub mod http_1;
pub mod http_2;
pub mod http_server;
pub mod server_config;

pub use http_server::Server;

use self::server_config::ServerConfig;

fn log_launch(sc: &ServerConfig) {
    let addr = sc.addr().to_string();
    tracing::info!(
        "🚀 Tailcall launched at [{}] over {}",
        addr,
        sc.http_version()
    );

    let url = sc.graphiql_url();
    let url = format!("https://tailcall.run/playground/?u={}/graphql", url);
    tracing::info!("🌍 Playground: {}", url);
}
