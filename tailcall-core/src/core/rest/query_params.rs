use std::collections::BTreeMap;

use async_graphql::{Name, Variables};

use super::type_map::TypeMap;
use super::typed_variables::TypedVariable;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct QueryParams {
    params: Vec<(String, TypedVariable)>,
}

impl From<Vec<(&str, TypedVariable)>> for QueryParams {
    fn from(value: Vec<(&str, TypedVariable)>) -> Self {
        Self {
            params: value.into_iter().map(|(k, v)| (k.to_string(), v)).collect(),
        }
    }
}

impl QueryParams {
    pub fn try_from_map(q: &TypeMap, map: BTreeMap<String, String>) -> anyhow::Result<Self> {
        let mut params = Vec::new();
        for (k, v) in map {
            let t = TypedVariable::try_from(
                q.get(&k)
                    .ok_or(anyhow::anyhow!("undefined query param: {}", k))?,
                &v,
            )?;
            params.push((k, t));
        }
        Ok(Self { params })
    }

    pub fn matches(&self, query_params: BTreeMap<String, String>) -> Option<Variables> {
        let mut variables = Variables::default();
        for (key, t_var) in &self.params {
            if let Some(query_param) = query_params.get(key) {
                let value = t_var.to_value(query_param).ok()?;
                variables.insert(Name::new(t_var.name()), value);
            }
        }
        Some(variables)
    }
}
