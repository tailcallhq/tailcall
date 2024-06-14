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

pub fn from_openapi_spec(query: impl AsRef<str>, spec_str: impl AsRef<str>) -> anyhow::Result<Config> {
    OpenApiToConfigConverter::new(query, spec_str)
        .map(|converter| converter.convert())
}