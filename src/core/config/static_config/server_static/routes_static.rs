use derive_getters::Getters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tailcall_macros::MergeRight;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, MergeRight, JsonSchema, Getters)]
/// Used to configure the default routes of the server.
pub struct Routes {
    /// The path for the status endpoint. Defaults to `/status`.
    status: String,
    /// The path for the GraphQL endpoint. Defaults to `/graphql`.
    graphql: String,
}

impl Default for Routes {
    fn default() -> Self {
        Self { status: "/status".into(), graphql: "/graphql".into() }
    }
}

impl Routes {
    pub fn with_status<T: Into<String>>(self, status: T) -> Self {
        Self { graphql: self.graphql, status: status.into() }
    }

    pub fn with_graphql<T: Into<String>>(self, graphql: T) -> Self {
        Self { status: self.status, graphql: graphql.into() }
    }
}