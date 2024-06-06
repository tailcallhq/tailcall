use std::collections::HashSet;

use crate::core::{
    config::{transformer::Transform, Config},
    valid::Valid,
};

struct UpstreamBaseUrlGenerator;

impl UpstreamBaseUrlGenerator {
    fn generate_base_url(&self, mut config: Config) -> Config {
        let mut base_url_set = HashSet::new();

        let operation_types = [
            &config.schema.query,
            &config.schema.mutation,
            &config.schema.subscription,
        ];

        for operation_type in operation_types.iter().filter_map(|op| op.as_deref()) {
            if let Some(type_) = config.types.get(operation_type) {
                for field in type_.fields.values() {
                    if let Some(http_directive) = &field.http {
                        if let Some(base_url) = &http_directive.base_url {
                            base_url_set.insert(base_url.to_owned());
                        }
                    }
                }
            }
        }

        if base_url_set.len() == 1 {
            if let Some(base_url) = base_url_set.iter().next() {
                config.upstream.base_url = Some(base_url.to_owned());

                for operation_type in operation_types.iter().filter_map(|op| op.as_deref()) {
                    if let Some(type_) = config.types.get_mut(operation_type) {
                        for field in type_.fields.values_mut() {
                            if let Some(http_directive) = &mut field.http {
                                http_directive.base_url = None;
                            }
                        }
                    }
                }
            }
        }

        config
    }
}

impl Transform for UpstreamBaseUrlGenerator {
    fn transform(&self, config: Config) -> Valid<Config, String> {
        let config = self.generate_base_url(config);
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Ok;

    use crate::core::config::transformer::Transform;
    use crate::core::config::Config;
    use crate::core::generator::json::upstream_base_url_generator::UpstreamBaseUrlGenerator;
    use crate::core::valid::Validator;

    #[test]
    fn should_generate_upstream_base_url_when_all_http_directive_has_same_base_url(
    ) -> anyhow::Result<()> {
        let config = Config::from_sdl(
            r#"
            schema @server @upstream {
            query: Query
          }
          
          type Query {
            f1: [Int] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/users")
            f2: [Int] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/post")
            f3: [Int] @http(baseURL: "https://jsonplaceholder.typicode.com", path: "/todos")
          }
          
          "#,
        )
        .to_result()?;

        let transformed_config = UpstreamBaseUrlGenerator.transform(config).to_result()?;
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }

    #[test]
    fn should_not_generate_upstream_base_url_when_all_http_directive_has_same_base_url(
    ) -> anyhow::Result<()> {
        let config = Config::from_sdl(
            r#"schema @server @upstream {
            query: Query
          }
          
          type Query {
            f1: [Int] @http(baseURL: "https://jsonplaceholder-1.typicode.com", path: "/users")
            f2: [Int] @http(baseURL: "https://jsonplaceholder-2.typicode.com", path: "/post")
            f3: [Int] @http(baseURL: "https://jsonplaceholder-3.typicode.com", path: "/todos")
          }
 
          "#,
        )
        .to_result()?;

        let transformed_config = UpstreamBaseUrlGenerator.transform(config).to_result()?;
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }
}
