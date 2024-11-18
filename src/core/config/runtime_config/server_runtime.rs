use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::net::{AddrParseError, IpAddr};
use std::str::FromStr;
use std::time::Duration;

use derive_setters::Setters;
use http::header::{HeaderMap, HeaderName, HeaderValue};
use rustls_pki_types::CertificateDer;
use tailcall_valid::{Valid, ValidationError, Validator};

use crate::core::blueprint;
use crate::core::config::cors_static::CorsStatic;
use crate::core::config::{self, ConfigModule, HttpVersion, PrivateKey, Routes};

pub mod auth_runtime;
pub use auth_runtime::*;
pub mod cors_runtime;
pub use cors_runtime::*;

#[derive(Clone, Debug, Setters)]
pub struct ServerRuntime {
    pub enable_jit: bool,
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
    pub http: HttpVersionRuntime,
    pub pipeline_flush: bool,
    pub script: Option<ScriptRuntime>,
    pub cors: Option<CorsRuntime>,
    pub experimental_headers: HashSet<HeaderName>,
    pub auth: Option<AuthRuntime>,
    pub routes: Routes,
}

/// Mimic of mini_v8::Script that's wasm compatible
#[derive(Clone, Debug)]
pub struct ScriptRuntime {
    pub source: String,
    pub timeout: Option<Duration>,
}

#[derive(Clone, Debug)]
pub enum HttpVersionRuntime {
    HTTP1,
    HTTP2 {
        cert: Vec<CertificateDer<'static>>,
        key: PrivateKey,
    },
}

impl Default for ServerRuntime {
    fn default() -> Self {
        // NOTE: Using unwrap because try_from default will never fail
        ServerRuntime::try_from(ConfigModule::default()).unwrap()
    }
}

impl ServerRuntime {
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

impl TryFrom<crate::core::config::ConfigModule> for ServerRuntime {
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

                let key = config_module
                    .extensions()
                    .keys
                    .first()
                    .ok_or_else(|| ValidationError::new("Key is required for HTTP2".to_string()))?
                    .clone();

                Valid::succeed(HttpVersionRuntime::HTTP2 { cert, key })
            }
            _ => Valid::succeed(HttpVersionRuntime::HTTP1),
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
            .fuse(AuthRuntime::make(&config_module))
            .map(
                |(hostname, http, response_headers, script, experimental_headers, cors, auth)| {
                    ServerRuntime {
                        enable_jit: (config_server).enable_jit(),
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
                        routes: config_server.get_routes(),
                    }
                },
            )
            .to_result()
    }
}

impl From<blueprint::Server> for ServerRuntime {
    fn from(server: blueprint::Server) -> Self {
        Self {
            enable_jit: server.enable_jit,
            enable_apollo_tracing: server.enable_apollo_tracing,
            enable_cache_control_header: server.enable_cache_control_header,
            enable_set_cookie_header: server.enable_set_cookie_header,
            enable_introspection: server.enable_introspection,
            enable_query_validation: server.enable_query_validation,
            enable_response_validation: server.enable_response_validation,
            enable_batch_requests: server.enable_batch_requests,
            enable_showcase: server.enable_showcase,
            global_response_timeout: server.global_response_timeout,
            worker: server.worker,
            port: server.port,
            hostname: server.hostname,
            vars: server.vars,
            response_headers: server.response_headers,
            http: server.http,
            pipeline_flush: server.pipeline_flush,
            script: server.script,
            cors: server.cors,
            experimental_headers: server.experimental_headers,
            auth: server.auth,
            routes: server.routes,
        }
    }
}

fn to_script(
    config_module: &crate::core::config::ConfigModule,
) -> Valid<Option<ScriptRuntime>, String> {
    config_module.extensions().script.as_ref().map_or_else(
        || Valid::succeed(None),
        |script| {
            Valid::succeed(Some(ScriptRuntime {
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

fn validate_cors(cors: Option<CorsStatic>) -> Valid<Option<CorsRuntime>, String> {
    Valid::from(cors.map(|cors| cors.try_into()).transpose())
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
        let actual = super::ServerRuntime::try_from(ConfigModule::default());
        assert!(actual.is_ok())
    }
}
