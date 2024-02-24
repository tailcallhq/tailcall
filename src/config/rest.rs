use async_graphql_value::ConstValue;
use serde::{Deserialize, Serialize};

use crate::blueprint::OperationQuery;
use crate::http::Method;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RestApis(Vec<(Rest, String)>);

impl RestApis {
    pub fn create_operations(&self) -> Vec<OperationQuery> {
        self.0
            .iter()
            .map(|(k, v)| {
                let variables = ConstValue::Object(
                    k.variables()
                        .map(|var| (async_graphql::Name::new(var), ConstValue::Null))
                        .collect(),
                );
                let variables = async_graphql::Variables::from_value(variables);
                OperationQuery::new_with_variables(v.into(), "".into(), variables)
            })
            .collect()
    }
}

impl RestApis {
    pub fn merge_right(mut self, other: Self) -> Self {
        self.0.extend(other.0);
        self
    }

    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn insert(&mut self, rest: Rest, query: impl Into<String>) {
        self.0.push((rest, query.into()));
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Rest, String)> {
        self.0.iter()
    }
}

#[derive(
    Clone, Debug, PartialEq, Deserialize, PartialOrd, Ord, Serialize, Eq, schemars::JsonSchema,
)]
/// The @rest operator creates a rest api for the operation it is applied to
#[serde(rename_all = "camelCase")]
pub struct Rest {
    /// Specifies the path for the rest api, relative to the base url.
    pub path: String,
    /// Specifies the HTTP Method for the rest api
    #[serde(default)]
    pub method: Method,
}

impl Rest {
    pub fn variables(&self) -> impl Iterator<Item = &str> {
        self.path.split('/').filter_map(|s| s.strip_prefix('$'))
    }
}
