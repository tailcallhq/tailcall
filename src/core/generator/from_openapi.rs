use std::collections::{BTreeMap, VecDeque};

use convert_case::{Case, Casing};
use oas3::{Schema, Spec};

use crate::core::config::{Config, Enum, Field, Http, RootSchema, Type, Union, Upstream};
use crate::core::http::Method;

#[allow(unused)]
#[derive(Default)]
pub struct OpenApiToConfigConverter {
    pub query: String,
    pub spec: Spec,
    pub inline_types: VecDeque<Schema>,
    pub inline_types_frozen: bool,
    pub inline_types_other: VecDeque<Schema>,
    pub types: BTreeMap<String, Type>,
    pub unions: BTreeMap<String, Union>,
    pub enums: BTreeMap<String, Enum>,
}

impl OpenApiToConfigConverter {
    pub fn new(query: impl AsRef<str>, spec_str: impl AsRef<str>) -> anyhow::Result<Self> {
        let spec = oas3::from_reader(spec_str.as_ref().as_bytes())?;
        Ok(Self {
            query: query.as_ref().to_string(),
            spec,
            ..Default::default()
        })
    }

    pub fn create_types(&mut self) {
        let mut fields = BTreeMap::new();

        for (path, path_item) in self.spec.paths.clone().into_iter() {
            let (method, operation) = [
                (Method::GET, path_item.get),
                (Method::HEAD, path_item.head),
                (Method::OPTIONS, path_item.options),
                (Method::TRACE, path_item.trace),
                (Method::PUT, path_item.put),
                (Method::POST, path_item.post),
                (Method::DELETE, path_item.delete),
                (Method::PATCH, path_item.patch),
            ]
            .into_iter()
            .filter_map(|(method, operation)| operation.map(|operation| (method, operation)))
            .next()
            .unwrap();

            let Ok(response) = operation
                .responses
                .first_key_value()
                .map(|(_, v)| v)
                .unwrap()
                .resolve(&self.spec)
            else {
                continue;
            };

            let Some(_output_type) = response
                .content
                .first_key_value()
                .map(|(_, v)| v)
                .cloned()
                .and_then(|v| v.schema)
            else {
                continue;
            };

            let field = Field {
                http: Some(Http { path, method, ..Default::default() }),
                doc: operation.description,
                ..Default::default()
            };

            fields.insert(operation.operation_id.unwrap().to_case(Case::Camel), field);
        }

        self.types
            .insert(self.query.clone(), Type { fields, ..Default::default() });
    }

    pub fn convert(mut self) -> Config {
        self.create_types();
        let config = Config {
            server: Default::default(),
            upstream: Upstream {
                base_url: self.spec.servers.first().cloned().map(|server| server.url),
                ..Default::default()
            },
            schema: RootSchema {
                query: self.types.get(&self.query).map(|_| self.query),
                ..Default::default()
            },
            types: self.types,
            unions: self.unions,
            enums: self.enums,
            ..Default::default()
        };
        config
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    #[test]
    fn test_openapi_apis_guru() {
        let apis_guru = config_from_openapi_spec("apis-guru.yml");
        insta::assert_snapshot!(apis_guru);
    }

    #[test]
    fn test_openapi_jsonplaceholder() {
        let jsonplaceholder = config_from_openapi_spec("jsonplaceholder.yml");
        insta::assert_snapshot!(jsonplaceholder);
    }

    #[test]
    fn test_openapi_spotify() {
        let spotify = config_from_openapi_spec("spotify.yml");
        insta::assert_snapshot!(spotify);
    }

    fn config_from_openapi_spec(filename: &str) -> String {
        let spec_path = Path::new("src")
            .join("core")
            .join("generator")
            .join("openapi")
            .join(filename);

        let content = fs::read_to_string(spec_path).unwrap();
        OpenApiToConfigConverter::new("Query", content.as_str())
            .unwrap()
            .convert()
            .to_sdl()
    }
}
