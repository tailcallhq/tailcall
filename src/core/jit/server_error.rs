use std::{
    collections::BTreeMap,
    fmt::{Debug, Display, Formatter},
};

use async_graphql::ErrorExtensions;
use serde::{Deserialize, Serialize};

use super::{PathSegment, Pos, Positioned};

/// An error in a GraphQL server.
#[derive(Clone, Serialize, Deserialize)]
pub struct ServerError {
    /// An explanatory message of the error.
    pub message: String,
    /// Where the error occurred.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub locations: Vec<Pos>,
    /// If the error occurred in a resolver, the path to the error.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub path: Vec<PathSegment>,
    /// Extensions to the error.
    #[serde(skip_serializing_if = "error_extensions_is_empty", default)]
    pub extensions: Option<ErrorExtensionValues>,
}

impl From<Positioned<super::Error>> for ServerError {
    fn from(value: Positioned<super::Error>) -> Self {
        let inner_value = value.value;
        let position = value.pos;

        // TODO: fix the extensions.
        let _ext = inner_value.extend().extensions;
        let server_error = ServerError::new(inner_value.to_string(), Some(position));

        server_error
    }
}

impl ServerError {
    /// Create a new server error with the message.
    pub fn new(message: impl Into<String>, pos: Option<Pos>) -> Self {
        Self {
            message: message.into(),
            locations: pos.map(|pos| vec![pos]).unwrap_or_default(),
            path: Vec::new(),
            extensions: None,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub fn with_path(self, path: Vec<PathSegment>) -> Self {
        Self { path, ..self }
    }
}

impl Display for ServerError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<ServerError> for Vec<ServerError> {
    fn from(single: ServerError) -> Self {
        vec![single]
    }
}

impl From<async_graphql::parser::Error> for ServerError {
    fn from(e: async_graphql::parser::Error) -> Self {
        Self {
            message: e.to_string(),
            locations: e.positions().into_iter().map(|p| p.into()).collect(),
            path: Vec::new(),
            extensions: None,
        }
    }
}

impl Debug for ServerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerError")
            .field("message", &self.message)
            .field("locations", &self.locations)
            .field("path", &self.path)
            .field("extensions", &self.extensions)
            .finish()
    }
}

impl PartialEq for ServerError {
    fn eq(&self, other: &Self) -> bool {
        self.message.eq(&other.message)
            && self.locations.eq(&other.locations)
            && self.path.eq(&other.path)
            && self.extensions.eq(&other.extensions)
    }
}

fn error_extensions_is_empty(values: &Option<ErrorExtensionValues>) -> bool {
    values.as_ref().map_or(true, |values| values.0.is_empty())
}

/// Extensions to the error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct ErrorExtensionValues(BTreeMap<String, async_graphql::Value>);

impl ErrorExtensionValues {
    /// Set an extension value.
    pub fn set(&mut self, name: impl AsRef<str>, value: impl Into<async_graphql::Value>) {
        self.0.insert(name.as_ref().to_string(), value.into());
    }

    /// Unset an extension value.
    pub fn unset(&mut self, name: impl AsRef<str>) {
        self.0.remove(name.as_ref());
    }

    /// Get an extension value.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&async_graphql::Value> {
        self.0.get(name.as_ref())
    }
}
