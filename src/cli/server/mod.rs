pub mod http_1;
pub mod http_2;
pub mod server;
pub mod server_config;

pub use server::Server;

use self::server_config::ServerConfig;

fn log_launch_and_open_browser(sc: &ServerConfig) {
  let addr = sc.addr().to_string();
  log::info!("🚀 Tailcall launched at [{}] over {}", addr, sc.http_version());
  if sc.graphiql() {
    let url = sc.graphiql_url();
    log::info!("🌍 Playground: {}", url);

    let _ = webbrowser::open(url.as_str());
  }
}
