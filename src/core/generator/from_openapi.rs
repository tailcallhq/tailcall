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
