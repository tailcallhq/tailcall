use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use oas3::spec::ObjectOrReference;
use oas3::Spec;

use crate::core::config::{Config, Field, Http, Type};
use crate::core::http::Method;

#[derive(Default)]
pub struct OpenApiToConfigConverter {
    #[allow(unused)]
    spec: Spec,
    config: Config,
}

fn name_from_ref_path<T>(obj_or_ref: &ObjectOrReference<T>) -> Option<String> {
    match obj_or_ref {
        ObjectOrReference::Ref { ref_path } => {
            ref_path.split('/').last().map(|a| a.to_case(Case::Pascal))
        }
        ObjectOrReference::Object(_) => None,
    }
}

impl OpenApiToConfigConverter {
    pub fn new(query: impl AsRef<str>, spec_str: impl AsRef<str>) -> anyhow::Result<Self> {
        let spec = oas3::from_reader(spec_str.as_ref().as_bytes())?;
        let config = Config::default().query(query.as_ref());
        Ok(Self { config, spec })
    }

    pub fn define_queries(mut self) -> Option<Self> {
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
            .next()?;

            let Ok(response) = operation
                .responses
                .first_key_value()
                .map(|(_, v)| v)?
                .resolve(&self.spec)
            else {
                continue;
            };

            let Some(output_type) = response
                .content
                .first_key_value()
                .map(|(_, v)| v)
                .cloned()
                .and_then(|v| v.schema)
            else {
                continue;
            };

            if let Some(type_of) = name_from_ref_path(&output_type) {
                let field = Field {
                    type_of,
                    http: Some(Http { path, method, ..Default::default() }),
                    doc: operation.description,
                    ..Default::default()
                };

                fields.insert(operation.operation_id.unwrap().to_case(Case::Camel), field);
            }
        }

        if let Some(query) = self.config.schema.query.as_ref() {
            self.config
                .types
                .insert(query.to_string(), Type { fields, ..Default::default() });
        }

        Some(self)
    }

    pub fn convert(mut self) -> Option<Config> {
        self = self.define_queries()?;
        Some(self.config)
    }
}

pub fn from_openapi_spec(query: impl AsRef<str>, spec_str: impl AsRef<str>) -> anyhow::Result<Config> {
    let config = OpenApiToConfigConverter::new(query, spec_str)
        .map(|converter| converter.convert())?;

    config.ok_or(anyhow::anyhow!("Unable to convert config"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    #[test]
    fn test_openapi_apis_guru() {
        let apis_guru = config_from_openapi_spec("apis-guru.yml").unwrap();
        insta::assert_snapshot!(apis_guru);
    }

    #[test]
    fn test_openapi_jsonplaceholder() {
        let jsonplaceholder = config_from_openapi_spec("jsonplaceholder.yml").unwrap();
        insta::assert_snapshot!(jsonplaceholder);
    }

    #[test]
    fn test_openapi_spotify() {
        let spotify = config_from_openapi_spec("spotify.yml").unwrap();
        insta::assert_snapshot!(spotify);
    }

    fn config_from_openapi_spec(filename: &str) -> Option<String> {
        let spec_path = Path::new("src")
            .join("core")
            .join("generator")
            .join("openapi")
            .join(filename);

        let content = fs::read_to_string(spec_path).ok()?;
        from_openapi_spec("Query", content).map(ToString::to_string).ok()
    }
}