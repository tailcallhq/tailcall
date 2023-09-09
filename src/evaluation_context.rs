use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use async_graphql::dataloader::{DataLoader, HashMapCache};
use async_graphql::dynamic::ResolverContext;
#[allow(unused_imports)]
use async_graphql::InputType;
use derive_setters::Setters;
use serde_json::Value;

use crate::config::Server;
use crate::http::{HttpClient, HttpDataLoader};

#[derive(Clone, Setters)]
#[setters(strip_option)]
pub struct EvaluationContext<'a> {
    pub variables: HashMap<usize, Value>,
    pub data_loader: &'a DataLoader<HttpDataLoader, HashMapCache>,
    pub context: Option<&'a ResolverContext<'a>>,
    pub env: HashMap<String, Value>,
    pub headers: BTreeMap<String, String>,
    pub timeout: Duration,
    pub server: &'a Server,
    pub client: &'a HttpClient,
}

impl<'a> EvaluationContext<'a> {
    pub fn set(mut self, id: usize, value: Value) -> Self {
        self.variables.insert(id, value);
        self
    }

    pub fn get(&self, id: &usize) -> Option<&Value> {
        self.variables.get(id)
    }

    pub fn new(
        data_loader: &'a DataLoader<HttpDataLoader, HashMapCache>,
        client: &'a HttpClient,
        server: &'a Server,
    ) -> EvaluationContext<'a> {
        Self {
            variables: HashMap::new(),
            data_loader,
            context: None,
            timeout: Duration::from_millis(5),
            env: HashMap::new(),
            headers: data_loader.loader().clone().get_headers().clone(),
            server,
            client,
        }
    }

    pub fn args(&self) -> Option<async_graphql::Value> {
        let ctx = self.context?;

        Some(async_graphql::Value::Object(ctx.args.as_index_map().clone()))
    }

    pub fn path_value(&'a self, path: &'a Vec<String>) -> Option<&'a async_graphql::Value> {
        get_path_value(self.value()?, path)
    }

    pub fn value(&self) -> Option<&'a async_graphql::Value> {
        let ctx = self.context?;
        ctx.parent_value.as_value()
    }

    pub fn get_headers(&self) -> BTreeMap<String, String> {
        self.headers.clone()
    }
}

pub fn get_path_value<'a>(input: &'a async_graphql::Value, path: &'a Vec<String>) -> Option<&'a async_graphql::Value> {
    let mut value = Some(input);
    for name in path {
        match value {
            Some(async_graphql::Value::Object(map)) => {
                value = map.get(&async_graphql::Name::new(name));
            }

            Some(async_graphql::Value::List(list)) => {
                value = list.get(name.parse::<usize>().ok()?);
            }
            _ => return None,
        }
    }

    value
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::evaluation_context::get_path_value;

    #[test]
    fn test_path_value() {
        let json = json!(
        {
            "a": {
                "b": {
                    "c": "d"
                }
            }
        });

        let async_value = async_graphql::Value::from_json(json).unwrap();

        let path = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = get_path_value(&async_value, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &async_graphql::Value::String("d".to_string()));
    }

    #[test]
    fn test_path_not_found() {
        let json = json!(
        {
            "a": {
                "b": "c"
            }
        });

        let async_value = async_graphql::Value::from_json(json).unwrap();

        let path = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = get_path_value(&async_value, &path);
        assert!(result.is_none());
    }

    #[test]
    fn test_numeric_path() {
        let json = json!(
        {
            "a": [{
                "b": "c"
            }]
        });

        let async_value = async_graphql::Value::from_json(json).unwrap();

        let path = vec!["a".to_string(), "0".to_string(), "b".to_string()];
        let result = get_path_value(&async_value, &path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), &async_graphql::Value::String("c".to_string()));
    }
}
