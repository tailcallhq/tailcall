use anyhow::Result;

use crate::cli::server::server_config::ServerConfig;

fn cache_metrics(server_config: &ServerConfig) -> Result<()> {
  let meter = opentelemetry::global::meter("ChronoCache");
  let cache = server_config.app_ctx.runtime.cache.clone();
  let counter = meter
    .f64_observable_counter("hit_rate")
    .with_description("ChronoCache hit rate ratio")
    .init();

  meter.register_callback(&[counter.as_any()], move |observer| {
    if let Some(hit_rate) = cache.hit_rate() {
      observer.observe_f64(&counter, hit_rate, &[]);
    }
  })?;

  Ok(())
}

pub fn init_metrics(server_config: &ServerConfig) -> Result<()> {
  cache_metrics(server_config)
}
