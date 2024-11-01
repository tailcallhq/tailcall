use miette::IntoDiagnostic;

use crate::core::runtime::TargetRuntime;

fn cache_metrics(runtime: &TargetRuntime) -> miette::Result<()> {
    let meter = opentelemetry::global::meter("cache");
    let cache = runtime.cache.clone();
    let counter = meter
        .f64_observable_gauge("cache.hit_rate")
        .with_description("Cache hit rate ratio")
        .init();

    meter
        .register_callback(&[counter.as_any()], move |observer| {
            if let Some(hit_rate) = cache.hit_rate() {
                observer.observe_f64(&counter, hit_rate, &[]);
            }
        })
        .into_diagnostic()?;

    Ok(())
}

fn process_resources_metrics() -> miette::Result<()> {
    let meter = opentelemetry::global::meter("process-resources");

    Ok(opentelemetry_system_metrics::init_process_observer(meter)
        .map_err(|err| miette::diagnostic!("{}", err))?)
}

pub fn init_metrics(runtime: &TargetRuntime) -> miette::Result<()> {
    cache_metrics(runtime)?;
    process_resources_metrics()?;

    Ok(())
}
