use convert_case::{Case, Casing};
use url::Url;

use super::url_utils::extract_base_url;
use crate::core::config::{Config, GraphQLOperationType, Resolver};
use crate::core::transform::Transform;
use crate::core::valid::Valid;

pub struct FieldBaseUrlGenerator<'a> {
    url: &'a Url,
    operation_type: &'a GraphQLOperationType,
}

impl<'a> FieldBaseUrlGenerator<'a> {
    pub fn new(url: &'a Url, operation_type: &'a GraphQLOperationType) -> Self {
        Self { url, operation_type }
    }

    fn update_base_urls(&self, config: &mut Config, operation_name: &str, base_url: &str) {
        if let Some(query_type) = config.types.get_mut(operation_name) {
            for field in query_type.fields.values_mut() {
                if let Some(Resolver::Http(http)) = &mut field.resolver {
                    if http.base_url.is_none() {
                        http.base_url = Some(base_url.to_owned())
                    }
                }
            }
        }
    }
}

impl Transform for FieldBaseUrlGenerator<'_> {
    type Value = Config;
    type Error = String;
    fn transform(&self, mut config: Self::Value) -> Valid<Self::Value, Self::Error> {
        let base_url = match extract_base_url(self.url) {
            Some(base_url) => base_url,
            None => {
                return Valid::fail(format!("failed to extract the host url from {} ", self.url))
            }
        };
        let op_name = self.operation_type.to_string().to_case(Case::Pascal);
        self.update_base_urls(&mut config, &op_name, &base_url);

        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::FieldBaseUrlGenerator;
    use crate::core::config::{Config, Field, GraphQLOperationType, Http, Resolver, Type};
    use crate::core::transform::Transform;
    use crate::core::valid::Validator;

    #[test]
    fn should_add_base_url_for_http_fields() {
        let url = Url::parse("https://example.com").unwrap();
        let field_base_url_gen = FieldBaseUrlGenerator::new(&url, &GraphQLOperationType::Query);

        let mut config = Config::default();
        let mut query_type = Type::default();
        query_type.fields.insert(
            "f1".to_string(),
            Field {
                type_of: "Int".to_string().into(),
                resolver: Some(Resolver::Http(Http {
                    path: "/day".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f2".to_string(),
            Field {
                type_of: "String".to_string().into(),
                resolver: Some(Resolver::Http(Http {
                    path: "/month".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f3".to_string(),
            Field {
                type_of: "String".to_string().into(),
                resolver: Some(Resolver::Http(Http {
                    path: "/status".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        config.types.insert("Query".to_string(), query_type);

        config = field_base_url_gen.transform(config).to_result().unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn should_add_base_url_if_not_present() {
        let url = Url::parse("http://localhost:8080").unwrap();
        let field_base_url_gen = FieldBaseUrlGenerator::new(&url, &GraphQLOperationType::Query);

        let mut config = Config::default();
        let mut query_type = Type::default();
        query_type.fields.insert(
            "f1".to_string(),
            Field {
                type_of: "Int".to_string().into(),
                resolver: Some(Resolver::Http(Http {
                    base_url: Some("https://calender.com/api/v1/".to_string()),
                    path: "/day".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f2".to_string(),
            Field {
                type_of: "String".to_string().into(),
                resolver: Some(Resolver::Http(Http {
                    path: "/month".to_string(),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
        query_type.fields.insert(
            "f3".to_string(),
            Field {
                type_of: "String".to_string().into(),
                resolver: None,
                ..Default::default()
            },
        );
        config.types.insert("Query".to_string(), query_type);

        config = field_base_url_gen.transform(config).to_result().unwrap();

        insta::assert_snapshot!(config.to_sdl());
    }

    #[test]
    fn should_not_add_base_url_when_query_not_present() {
        let url = Url::parse("https://example.com").unwrap();
        let field_base_url_gen = FieldBaseUrlGenerator::new(&url, &GraphQLOperationType::Query);
        assert!(field_base_url_gen
            .transform(Default::default())
            .to_result()
            .unwrap()
            .to_sdl()
            .is_empty());
    }
}
