use derive_setters::Setters;
use hyper::header;
use hyper::header::{HeaderName, HeaderValue};
use hyper::http::request::Parts;

use crate::config;

#[derive(Clone, Debug, Setters)]
pub struct CorsParams {
    pub allow_credentials: Option<bool>,
    pub allow_headers: ConstOrMirror,
    pub allow_methods: ConstOrMirror,
    pub allow_origin: Vec<HeaderValue>,
    pub allow_private_network: bool,
    pub expose_headers: Option<HeaderValue>,
    pub max_age: Option<HeaderValue>,
    pub vary: Vec<HeaderValue>,
}

impl CorsParams {
    pub fn allow_origin_to_header(
        &self,
        origin: Option<&HeaderValue>,
    ) -> Option<(HeaderName, HeaderValue)> {
        Some((
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            origin.filter(|o| self.allow_origin.contains(o))?.clone(),
        ))
    }

    pub fn allow_credentials_to_header(&self) -> Option<(HeaderName, HeaderValue)> {
        self.allow_credentials?.then(|| {
            (
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            )
        })
    }

    pub fn allow_private_network_to_header(
        &self,
        parts: &Parts,
    ) -> Option<(HeaderName, HeaderValue)> {
        #[allow(clippy::declare_interior_mutable_const)]
        const REQUEST_PRIVATE_NETWORK: HeaderName =
            HeaderName::from_static("access-control-request-private-network");

        #[allow(clippy::declare_interior_mutable_const)]
        const ALLOW_PRIVATE_NETWORK: HeaderName =
            HeaderName::from_static("access-control-allow-private-network");

        #[allow(clippy::declare_interior_mutable_const)]
        const TRUE: HeaderValue = HeaderValue::from_static("true");

        if !self.allow_private_network {
            return None;
        }

        // Access-Control-Allow-Private-Network is only relevant if the request
        // has the Access-Control-Request-Private-Network header set, else skip
        #[allow(clippy::borrow_interior_mutable_const)]
        if parts.headers.get(REQUEST_PRIVATE_NETWORK) != Some(&TRUE) {
            return None;
        }

        self.allow_private_network
            .then_some((ALLOW_PRIVATE_NETWORK, TRUE))
    }

    pub fn allow_methods_to_header(&self, parts: &Parts) -> Option<(HeaderName, HeaderValue)> {
        self.const_or_mirror_to_header(&self.allow_methods, parts)
    }

    pub fn allow_headers_to_header(&self, parts: &Parts) -> Option<(HeaderName, HeaderValue)> {
        self.const_or_mirror_to_header(&self.allow_headers, parts)
    }

    pub fn const_or_mirror_to_header(
        &self,
        const_or_mirror: &ConstOrMirror,
        parts: &Parts,
    ) -> Option<(HeaderName, HeaderValue)> {
        let allow_methods = match &const_or_mirror {
            ConstOrMirror::Const(v) => v.clone()?,
            ConstOrMirror::MirrorRequest => parts
                .headers
                .get(header::ACCESS_CONTROL_REQUEST_METHOD)?
                .clone(),
        };

        Some((header::ACCESS_CONTROL_ALLOW_METHODS, allow_methods))
    }

    pub fn max_age_to_header(&self) -> Option<(HeaderName, HeaderValue)> {
        Some((
            header::ACCESS_CONTROL_MAX_AGE,
            self.max_age.as_ref()?.clone(),
        ))
    }

    pub fn expose_headers_to_header(&self) -> Option<(HeaderName, HeaderValue)> {
        Some((
            header::ACCESS_CONTROL_EXPOSE_HEADERS,
            self.expose_headers.as_ref()?.clone(),
        ))
    }
}

impl TryFrom<config::cors_params::CorsParams> for CorsParams {
    type Error = anyhow::Error;

    fn try_from(value: config::cors_params::CorsParams) -> anyhow::Result<Self> {
        Ok(CorsParams {
            allow_credentials: value.allow_credentials,
            allow_headers: value.allow_headers.try_into()?,
            allow_methods: value.allow_methods.try_into()?,
            allow_origin: value
                .allow_origin
                .iter()
                .map(|val| Ok(val.parse()?))
                .collect::<anyhow::Result<_>>()?,
            allow_private_network: false,
            expose_headers: value.expose_headers.map(|val| val.parse()).transpose()?,
            max_age: value.max_age.map(|val| val.into()),
            vary: value
                .vary
                .iter()
                .map(|val| Ok(val.parse()?))
                .collect::<anyhow::Result<_>>()?,
        })
    }
}

#[derive(Clone, Debug)]
pub enum ConstOrMirror {
    Const(Option<HeaderValue>),
    MirrorRequest,
}

impl TryFrom<config::cors_params::ConstOrMirror> for ConstOrMirror {
    type Error = anyhow::Error;

    fn try_from(value: config::cors_params::ConstOrMirror) -> anyhow::Result<Self> {
        Ok(match value {
            config::cors_params::ConstOrMirror::Const(val) => {
                ConstOrMirror::Const(val.map(|val| val.parse()).transpose()?)
            }
            config::cors_params::ConstOrMirror::MirrorRequest => ConstOrMirror::MirrorRequest,
        })
    }
}
