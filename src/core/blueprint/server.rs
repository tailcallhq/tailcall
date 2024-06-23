use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::net::{AddrParseError, IpAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use derive_setters::Setters;
use hyper::header::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};

use super::Auth;
use crate::core::blueprint::Cors;
use crate::core::config::{self, ConfigModule, HttpVersion};
use crate::core::valid::{Valid, ValidationError, Validator};

#[derive(Clone, Debug, Setters)]
pub struct Server {
    pub enable_apollo_tracing: bool,
    pub enable_cache_control_header: bool,
    pub enable_set_cookie_header: bool,
    pub enable_introspection: bool,
    pub enable_query_validation: bool,
    pub enable_response_validation: bool,
    pub enable_batch_requests: bool,
    pub enable_showcase: bool,
    pub global_response_timeout: i64,
    pub worker: usize,
    pub port: u16,
    pub hostname: IpAddr,
    pub vars: BTreeMap<String, String>,
    pub response_headers: HeaderMap,
    pub http: Http,
    pub pipeline_flush: bool,
    pub script: Option<Script>,
    pub cors: Option<Cors>,
    pub experimental_headers: HashSet<HeaderName>,
    pub auth: Option<Auth>,
    pub dedupe: bool,
}

/// Mimic of mini_v8::Script that's wasm compatible
#[derive(Clone, Debug)]
pub struct Script {
    pub source: String,
    pub timeout: Option<Duration>,
}

#[derive(Clone, Debug)]
pub enum Http {
    HTTP1,
    HTTP2 {
        cert: Vec<CertificateDer<'static>>,
        key: Arc<PrivateKeyDer<'static>>,
    },
}

impl Default for Server {
    fn default() -> Self {
        // NOTE: Using unwrap because try_from default will never fail
        Server::try_from(ConfigModule::default()).unwrap()
    }
}

impl Server {
    pub fn get_enable_http_validation(&self) -> bool {
        self.enable_response_validation
    }
    pub fn get_enable_cache_control(&self) -> bool {
        self.enable_cache_control_header
    }

    pub fn get_enable_introspection(&self) -> bool {
        self.enable_introspection
    }

    pub fn get_enable_query_validation(&self) -> bool {
        self.enable_query_validation
    }

    pub fn get_experimental_headers(&self) -> HashSet<HeaderName> {
        self.experimental_headers.clone()
    }
}

impl TryFrom<crate::core::config::ConfigModule> for Server {
    type Error = ValidationError<String>;

    fn try_from(config_module: config::ConfigModule) -> Result<Self, Self::Error> {
        let config_server = config_module.server.clone();

        let http_server = match config_server.clone().get_version() {
            HttpVersion::HTTP2 => {
                if config_module.extensions().cert.is_empty() {
                    return Valid::fail("Certificate is required for HTTP2".to_string())
                        .to_result();
                }

                let cert = config_module.extensions().cert.clone();

                let key_file: PrivateKeyDer<'_> = config_module
                    .extensions()
                    .keys
                    .first()
                    .ok_or_else(|| ValidationError::new("Key is required for HTTP2".to_string()))?
                    .clone_key();

                let key: Arc<PrivateKeyDer<'_>> = Arc::new(key_file);

                Valid::succeed(Http::HTTP2 { cert, key })
            }
            _ => Valid::succeed(Http::HTTP1),
        };

        validate_hostname((config_server).get_hostname().to_lowercase())
            .fuse(http_server)
            .fuse(handle_response_headers(
                (config_server).get_response_headers(),
            ))
            .fuse(to_script(&config_module))
            .fuse(handle_experimental_headers(
                (config_server).get_experimental_headers(),
            ))
            .fuse(validate_cors(
                config_server
                    .headers
                    .as_ref()
                    .and_then(|headers| headers.get_cors()),
            ))
            .fuse(Auth::make(&config_module))
            .map(
                |(hostname, http, response_headers, script, experimental_headers, cors, auth)| {
                    Server {
                        enable_apollo_tracing: (config_server).enable_apollo_tracing(),
                        enable_cache_control_header: (config_server).enable_cache_control(),
                        enable_set_cookie_header: (config_server).enable_set_cookies(),
                        enable_introspection: (config_server).enable_introspection(),
                        enable_query_validation: (config_server).enable_query_validation(),
                        enable_response_validation: (config_server).enable_http_validation(),
                        enable_batch_requests: (config_server).enable_batch_requests(),
                        enable_showcase: (config_server).enable_showcase(),
                        experimental_headers,
                        global_response_timeout: (config_server).get_global_response_timeout(),
                        http,
                        worker: (config_server).get_workers(),
                        port: (config_server).get_port(),
                        hostname,
                        vars: (config_server).get_vars(),
                        pipeline_flush: (config_server).get_pipeline_flush(),
                        response_headers,
                        script,
                        cors,
                        auth,
                        dedupe: config_server.get_dedupe(),
                    }
                },
            )
            .to_result()
    }
}

fn to_script(config_module: &crate::core::config::ConfigModule) -> Valid<Option<Script>, String> {
    config_module.extensions().script.as_ref().map_or_else(
        || Valid::succeed(None),
        |script| {
            Valid::succeed(Some(Script {
                source: script.clone(),
                timeout: config_module
                    .server
                    .script
                    .clone()
                    .map_or_else(|| None, |script| script.timeout)
                    .map(Duration::from_millis),
            }))
        },
    )
}

fn validate_cors(cors: Option<config::cors::Cors>) -> Valid<Option<Cors>, String> {
    Valid::from(cors.map(|cors| cors.try_into()).transpose())
        .trace("cors")
        .trace("headers")
        .trace("@server")
        .trace("schema")
}

fn validate_hostname(hostname: String) -> Valid<IpAddr, String> {
    if hostname == "localhost" {
        Valid::succeed(IpAddr::from([127, 0, 0, 1]))
    } else {
        Valid::from(hostname.parse().map_err(|e: AddrParseError| {
            ValidationError::new(format!("Parsing failed because of {}", e))
        }))
        .trace("hostname")
        .trace("@server")
        .trace("schema")
    }
}

fn handle_response_headers(resp_headers: Vec<(String, String)>) -> Valid<HeaderMap, String> {
    Valid::from_iter(resp_headers.iter(), |(k, v)| {
        let name = Valid::from(
            HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e))),
        );
        let value = Valid::from(
            HeaderValue::from_str(v.as_str())
                .map_err(|e| ValidationError::new(format!("Parsing failed because of {}", e))),
        );
        name.zip(value)
    })
    .map(|headers| headers.into_iter().collect::<HeaderMap>())
    .trace("custom")
    .trace("headers")
    .trace("@server")
    .trace("schema")
}

fn handle_experimental_headers(headers: BTreeSet<String>) -> Valid<HashSet<HeaderName>, String> {
    Valid::from_iter(headers.iter(), |h| {
        if !h.to_lowercase().starts_with("x-") {
            Valid::fail(
                format!(
                    "Experimental headers must start with 'x-' or 'X-'. Got: '{}'",
                    h
                )
                .to_string(),
            )
        } else {
            Valid::from(HeaderName::from_str(h).map_err(|e| ValidationError::new(e.to_string())))
        }
    })
    .map(HashSet::from_iter)
    .trace("experimental")
    .trace("headers")
    .trace("@server")
    .trace("schema")
}

#[cfg(test)]
mod tests {
    use crate::core::config::ConfigModule;

    #[test]
    fn test_try_from_default() {
        let actual = super::Server::try_from(ConfigModule::default());
        assert!(actual.is_ok())
    }
}
