use std::collections::HashSet;

use super::MaxValueMap;
use crate::core::config::transformer::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

pub struct ConsolidateURL {
    threshold: f32,
}

impl ConsolidateURL {
    pub fn new(threshold: f32) -> Self {
        let mut validated_thresh = threshold;
        if !(0.0..=1.0).contains(&threshold) {
            validated_thresh = 1.0;
            tracing::warn!(
                "Invalid threshold value ({:.2}), reverting to default threshold ({:.2}). allowed range is [0.0 - 1.0] inclusive",
                threshold,
                validated_thresh
            );
        }
        Self { threshold: validated_thresh }
    }

    fn generate_base_url(&self, mut config: Config) -> Config {
        // maintain the base_url frequency map to determine the most common base url.
        let mut url_frequency_map = MaxValueMap::new();
        let mut type_with_base_url_fields = HashSet::new();

        for (type_name, type_) in config.types.iter() {
            for field_ in type_.fields.values() {
                if let Some(http_directive) = &field_.http {
                    if let Some(base_url) = &http_directive.base_url {
                        url_frequency_map.increment(base_url.to_owned(), 1);
                        type_with_base_url_fields.insert(type_name.to_owned());
                    }
                }
            }
        }

        // If there is a most common base URL, update the config.
        if let Some((most_common_base_url, most_common_base_url_frequency)) =
            url_frequency_map.get_max_pair()
        {
            let should_perform_consolidation = most_common_base_url_frequency.to_owned() as f32
                >= (url_frequency_map.len() as f32 * self.threshold);

            if !should_perform_consolidation {
                tracing::warn!(
                    "Threshold matching base url not found, transformation cannot be performed."
                );
                return config;
            }

            config.upstream.base_url = Some(most_common_base_url.to_owned());

            // Remove the base URL from the HTTP directives in the relevant types.
            for type_name in type_with_base_url_fields {
                if let Some(type_) = config.types.get_mut(&type_name) {
                    for field_ in type_.fields.values_mut() {
                        if let Some(http_directive) = &mut field_.http {
                            if let Some(base_url) = http_directive.base_url.clone() {
                                if &base_url == most_common_base_url {
                                    http_directive.base_url = None;
                                }
                            }
                        }
                    }
                }
            }
        }

        config
    }
}

impl Transform for ConsolidateURL {
    fn transform(&self, config: Config) -> Valid<Config, String> {
        let config = self.generate_base_url(config);
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use anyhow::Ok;

    use super::*;
    use crate::core::config::transformer::Transform;
    use crate::core::config::Config;
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

        let transformed_config = ConsolidateURL::new(0.5).transform(config).to_result()?;
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

        let transformed_config = ConsolidateURL::new(0.5).transform(config).to_result()?;
        insta::assert_snapshot!(transformed_config.to_sdl());

        Ok(())
    }
}
