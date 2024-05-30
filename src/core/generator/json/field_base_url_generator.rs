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

#[cfg(test)]
mod test {
    use url::Url;

    use super::FieldBaseUrlGenerator;
    use crate::core::config::{Config, Field, Http, Type};
    use crate::core::generator::json::ConfigGenerator;

    #[test]
    fn should_add_base_url_for_http_fields() {
        let url = Url::parse("https://example.com").unwrap();
        let query = "Query";
        let mut field_base_url_gen = FieldBaseUrlGenerator::new(&url, query);

        let mut config = Config::default();
        let mut query_type = Type::default();
        query_type.fields.insert(
            "f1".to_string(),
            Field {
                type_of: "Int".to_string(),
                http: Some(Http { path: "/day".to_string(), ..Default::default() }),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f2".to_string(),
            Field {
                type_of: "String".to_string(),
                http: Some(Http { path: "/month".to_string(), ..Default::default() }),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f3".to_string(),
            Field {
                type_of: "String".to_string(),
                http: Some(Http { path: "/status".to_string(), ..Default::default() }),
                ..Default::default()
            },
        );
        config.types.insert("Query".to_string(), query_type);

        config = field_base_url_gen.apply(config);

        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn should_add_base_url_if_not_present() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let query = "Query";
        let mut field_base_url_gen = FieldBaseUrlGenerator::new(&url, query);

        let mut config = Config::default();
        let mut query_type = Type::default();
        query_type.fields.insert(
            "f1".to_string(),
            Field {
                type_of: "Int".to_string(),
                http: Some(Http {
                    base_url: Some("https://calender.com/api/v1/".to_string()),
                    path: "/day".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f2".to_string(),
            Field {
                type_of: "String".to_string(),
                http: Some(Http { path: "/month".to_string(), ..Default::default() }),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f3".to_string(),
            Field {
                type_of: "String".to_string(),
                http: None,
                ..Default::default()
            },
        );
        config.types.insert("Query".to_string(), query_type);

        config = field_base_url_gen.apply(config);

        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn should_not_add_base_url_when_query_not_present() {
        let url = Url::parse("https://example.com").unwrap();
        let query = "Query";
        let mut field_base_url_gen = FieldBaseUrlGenerator::new(&url, query);
        assert!(field_base_url_gen
            .apply(Default::default())
            .to_sdl()
            .is_empty());
    }
}
