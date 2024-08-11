use std::borrow::Cow;

pub struct Adapter {}

impl Adapter {
    pub fn config(key: Option<Cow<str>>) -> genai::adapter::AdapterConfig {
        let mut config = genai::adapter::AdapterConfig::default();
        if let Some(key) = key {
            config = config.with_auth_env_name(key);
        }
        config
    }
}
