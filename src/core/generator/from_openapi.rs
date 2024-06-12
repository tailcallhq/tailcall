use oas3::Spec;

use crate::core::config::Config;

#[derive(Default)]
pub struct OpenApiToConfigConverter {
    #[allow(unused)]
    spec: Spec,
    config: Config,
}

impl OpenApiToConfigConverter {
    pub fn new(query: impl AsRef<str>, spec_str: impl AsRef<str>) -> anyhow::Result<Self> {
        let spec = oas3::from_reader(spec_str.as_ref().as_bytes())?;
        let config = Config::default().query(query.as_ref());
        Ok(Self { config, spec })
    }

    pub fn convert(self) -> Config {
        self.config
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
