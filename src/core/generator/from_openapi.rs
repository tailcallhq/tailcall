use std::collections::BTreeMap;

use oas3::Spec;

use crate::core::config::{Config, Enum, RootSchema, Type, Union, Upstream};

#[derive(Default)]
pub struct OpenApiToConfigConverter {
    pub query: String,
    pub spec: Spec,
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

    pub fn convert(self) -> Config {
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
