pub mod http_1;
pub mod http_2;
pub mod http_server;
pub mod playground;
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

    let gql_slug = sc.app_ctx.blueprint.server.routes.graphql();

    let graphiql_url = sc.graphiql_url() + gql_slug;
    let url = playground::build_url(&graphiql_url);
    tracing::info!("🌍 Playground: {}", url);
}
