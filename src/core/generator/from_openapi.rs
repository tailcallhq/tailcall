use oas3::{OpenApiV3Spec, Spec};

use crate::core::config::Config;

#[derive(Default)]
pub struct OpenApiToConfigConverter {
    #[allow(unused)]
    spec: Spec,
    config: Config,
}

impl OpenApiToConfigConverter {
    pub fn new(spec: OpenApiV3Spec) -> anyhow::Result<Self> {
        let config = Config::default();
        Ok(Self { config, spec })
    }

    pub fn convert(self) -> Config {
        self.config
    }
}

pub fn from_openapi_spec(spec: OpenApiV3Spec) -> anyhow::Result<Config> {
    OpenApiToConfigConverter::new(spec).map(|converter| converter.convert())
}

#[cfg(test)]
mod tests {
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
            .join("tests")
            .join("fixtures")
            .join("openapi")
            .join(filename);

        let spec = oas3::from_path(spec_path).unwrap();
        from_openapi_spec(spec).ok().map(|config| config.to_sdl())
    }
}
