pub mod http_1;
pub mod http_2;
pub mod http_server;
pub mod server_config;

pub use http_server::Server;

use self::server_config::ServerConfig;
use crate::cli::command::VERSION;

fn log_launch(sc: &ServerConfig) {
    let addr = sc.addr().to_string();
    tracing::info!(
        "🚀 Tailcall launched at [{}] over {}",
        addr,
        sc.http_version()
    );

    let url = sc.graphiql_url();
    let utm_source = if VERSION.eq("0.1.0-dev") {
        "tailcall-debug"
    } else {
        "tailcall-release"
    };
    let utm_medium = "server";
    let url = format!(
        "https://tailcall.run/playground/?u={}/graphql&utm_source={}&utm_medium={}",
        url, utm_source, utm_medium
    );
    tracing::info!("🌍 Playground: {}", url);
}
