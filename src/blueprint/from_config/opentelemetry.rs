use hyper::header::HeaderValue;
use hyper::HeaderMap;
use url::Url;

use super::init_context::InitContext;
use super::TryFoldConfig;
use crate::blueprint::opentelemetry::{Opentelemetry, OpentelemetryExporter, OpentelemetryInner, OtlpExporter};
use crate::config::{self, Config, KeyValues};
use crate::directive::DirectiveCodec;
use crate::helpers::headers::to_headervec;
use crate::mustache::Mustache;
use crate::try_fold::TryFold;
use crate::valid::{Valid, ValidationError};

fn to_url(url: &str, init_context: &InitContext) -> Valid<Url, String> {
  Valid::from(Mustache::parse(url).map_err(|e| ValidationError::new(e.to_string())))
    .and_then(|url| Valid::from(Url::parse(&url.render(init_context)).map_err(|e| ValidationError::new(e.to_string()))))
    .trace("url")
}

fn to_headers(headers: &KeyValues, init_context: &InitContext) -> Valid<HeaderMap, String> {
  to_headervec(headers)
    .and_then(|headers| {
      Valid::from_iter(headers, |(k, v)| {
        Valid::from(HeaderValue::from_str(&v.render(init_context)).map_err(|e| ValidationError::new(e.to_string())))
          .map(|v| (k, v))
      })
    })
    .map(HeaderMap::from_iter)
    .trace("headers")
}

pub fn to_opentelemetry<'a>() -> TryFold<'a, Config, Opentelemetry, String> {
  TryFoldConfig::<Opentelemetry>::new(|config, up| {
    if let Some(opentelemetry) = config.opentelemetry.0.as_ref() {
      let init_context = InitContext::from(&config.server);

      let export = match &opentelemetry.export {
        config::OpentelemetryExporter::Stdout(config) => Valid::succeed(OpentelemetryExporter::Stdout(config.clone())),
        config::OpentelemetryExporter::Otlp(config) => to_url(&config.url, &init_context)
          .zip(to_headers(&config.headers, &init_context))
          .map(|(url, headers)| OpentelemetryExporter::Otlp(OtlpExporter { url, headers }))
          .trace("otlp"),
      };

      export
        .map(|export| Opentelemetry(Some(OpentelemetryInner { export })))
        .trace(config::Opentelemetry::trace_name().as_str())
    } else {
      Valid::succeed(up)
    }
  })
}
