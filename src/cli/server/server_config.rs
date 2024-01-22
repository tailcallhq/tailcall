use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::Result;

use crate::blueprint::{Blueprint, Http};
use crate::cli::env::EnvNative;
use crate::cli::http::NativeHttp;
use crate::cli::{init_chrono_cache, init_env, init_http, init_http2_only};
use crate::http::AppContext;

pub struct ServerConfig {
  pub blueprint: Blueprint,
  pub server_context: Arc<AppContext<NativeHttp, EnvNative>>,
}

impl ServerConfig {
  pub fn try_new(blueprint: Blueprint) -> Result<Self> {
    let h_client = Arc::new(init_http(&blueprint.upstream));
    let h2_client = Arc::new(init_http2_only(&blueprint.upstream));
    let env = init_env();
    let chrono_cache = init_chrono_cache();
    let server_context = Arc::new(AppContext::try_new(
      blueprint.clone(),
      h_client,
      h2_client,
      Arc::new(env),
      Arc::new(chrono_cache),
    )?);
    Ok(Self { server_context, blueprint })
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
