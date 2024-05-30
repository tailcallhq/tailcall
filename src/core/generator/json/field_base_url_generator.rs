use url::Url;

use super::ConfigGenerator;
use crate::core::config::Config;

pub struct FieldBaseUrlGenerator<'a> {
    url: &'a Url,
    query: &'a str,
}

impl<'a> FieldBaseUrlGenerator<'a> {
    pub fn new(url: &'a Url, query: &'a str) -> Self {
        Self { url, query }
    }
}

impl ConfigGenerator for FieldBaseUrlGenerator<'_> {
    fn apply(&mut self, mut config: Config) -> Config {
        let base_url = match self.url.host_str() {
            Some(host) => match self.url.port() {
                Some(port) => format!("{}://{}:{}", self.url.scheme(), host, port),
                None => format!("{}://{}", self.url.scheme(), host),
            },
            None => return config,
        };

        if let Some(query_type) = config.types.get_mut(self.query) {
            for field in query_type.fields.values_mut() {
                field.http = match field.http.clone() {
                    Some(mut http) => {
                        if http.base_url.is_none() {
                            http.base_url = Some(base_url.clone());
                        }
                        Some(http)
                    }
                    None => None,
                }
            }
        }

        config
    }
}
