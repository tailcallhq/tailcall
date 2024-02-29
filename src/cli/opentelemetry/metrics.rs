use anyhow::{anyhow, Result};

use crate::runtime::TargetRuntime;

fn cache_metrics(runtime: &TargetRuntime) -> Result<()> {
    let meter = opentelemetry::global::meter("cache");
    let cache = runtime.cache.clone();
    let counter = meter
        .f64_observable_counter("hit_rate")
        .with_description("Cache hit rate ratio")
        .init();

    meter.register_callback(&[counter.as_any()], move |observer| {
        if let Some(hit_rate) = cache.hit_rate() {
            observer.observe_f64(&counter, hit_rate, &[]);
        }
    })?;

    Ok(())
}

fn process_resources_metrics() -> Result<()> {
    let meter = opentelemetry::global::meter("process-resources");

    opentelemetry_system_metrics::init_process_observer(meter)
        .map_err(|err| anyhow!(err))
}

pub fn init_metrics(runtime: &TargetRuntime) -> Result<()> {
    cache_metrics(runtime)?;
    process_resources_metrics()?;

    Ok(())
}
