use derive_setters::Setters;
use hyper::header;
use hyper::header::{HeaderName, HeaderValue};
use hyper::http::request::Parts;

use crate::config;
use crate::valid::ValidationError;

#[derive(Clone, Debug, Setters)]
pub struct CorsParams {
    pub allow_credentials: bool,
    pub allow_headers: Option<HeaderValue>,
    pub allow_methods: Option<HeaderValue>,
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
        let allow_origin = origin.filter(|o| self.allow_origin.contains(o))?.clone();
        Some((header::ACCESS_CONTROL_ALLOW_ORIGIN, allow_origin))
    }

    pub fn allow_credentials_to_header(&self) -> Option<(HeaderName, HeaderValue)> {
        self.allow_credentials.then(|| {
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

    pub fn allow_methods_to_header(&self) -> Option<(HeaderName, HeaderValue)> {
        Some((
            header::ACCESS_CONTROL_ALLOW_METHODS,
            self.allow_methods.clone()?,
        ))
    }

    pub fn allow_headers_to_header(&self) -> Option<(HeaderName, HeaderValue)> {
        Some((
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            self.allow_headers.clone()?,
        ))
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

    pub fn vary_to_header(&self) -> Option<(HeaderName, HeaderValue)> {
        let values = &self.vary;
        let mut res = values.first()?.as_bytes().to_owned();
        for val in &values[1..] {
            res.extend_from_slice(b", ");
            res.extend_from_slice(val.as_bytes());
        }

        let header_val = HeaderValue::from_bytes(&res).ok()?;
        Some((header::VARY, header_val))
    }

    #[allow(clippy::borrow_interior_mutable_const)]
    pub fn expose_headers_is_wildcard(&self) -> bool {
        matches!(&self.expose_headers, Some(v) if v == WILDCARD)
    }
}

impl TryFrom<config::cors_params::CorsParams> for CorsParams {
    type Error = ValidationError<String>;

    fn try_from(value: config::cors_params::CorsParams) -> Result<Self, ValidationError<String>> {
        Ok(try_from_inner(value)?)
    }
}

fn try_from_inner(value: config::cors_params::CorsParams) -> Result<CorsParams, anyhow::Error> {
    Ok(CorsParams {
        allow_credentials: value.allow_credentials,
        allow_headers: value
            .allow_headers
            .map(|list| list.join(", ").parse())
            .transpose()?,
        allow_methods: value
            .allow_methods
            .map(|list| list.join(", ").parse())
            .transpose()?,
        allow_origin: value
            .allow_origin
            .into_iter()
            .map(|val| Ok(val.parse()?))
            .collect::<anyhow::Result<_>>()?,
        allow_private_network: false,
        expose_headers: Some(value.expose_headers.join(", ").parse()?),
        max_age: value.max_age.map(|val| val.into()),
        vary: value
            .vary
            .iter()
            .map(|val| Ok(val.parse()?))
            .collect::<anyhow::Result<_>>()?,
    })
}

#[allow(clippy::declare_interior_mutable_const)]
const WILDCARD: HeaderValue = HeaderValue::from_static("*");

#[allow(clippy::borrow_interior_mutable_const)]
pub fn is_wildcard(header_value: &HeaderValue) -> bool {
    header_value == WILDCARD
}
