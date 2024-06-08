use std::collections::HashSet;

use crate::core::config::transformer::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

use super::max_value_map::MaxValueMap;

struct UrlTypeMapping {
    url_to_frequency_map: MaxValueMap<String, u32>,
    visited_type_set: HashSet<String>,
}

impl UrlTypeMapping {
    fn new() -> Self {
        Self {
            url_to_frequency_map: Default::default(),
            visited_type_set: Default::default(),
        }
    }

    /// Populates the URL type mapping based on the given configuration.
    fn populate_url_frequency_map(&mut self, config: &Config) {
        for (type_name, type_) in config.types.iter() {
            for field_ in type_.fields.values() {
                if let Some(http_directive) = &field_.http {
                    if let Some(base_url) = &http_directive.base_url {
                        self.url_to_frequency_map.increment(base_url.to_owned(), 1);
                        self.visited_type_set.insert(type_name.to_owned());
                    }
                }
            }
        }
    }

    /// Finds the most common URL that meets the threshold.
    fn find_common_url(&self, threshold: f32) -> Option<String> {
        if let Some((common_url, frequency)) = self.url_to_frequency_map.get_max_pair() {
            if *frequency >= (self.url_to_frequency_map.len() as f32 * threshold) as u32 {
                return Some(common_url.to_owned());
            }
        }
        None
    }
}

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
        let mut url_type_mapping = UrlTypeMapping::new();
        url_type_mapping.populate_url_frequency_map(&config);

        if let Some(common_url) = url_type_mapping.find_common_url(self.threshold) {
            config.upstream.base_url = Some(common_url.to_owned());

            for type_name in url_type_mapping.visited_type_set {
                if let Some(type_) = config.types.get_mut(&type_name) {
                    for field_ in type_.fields.values_mut() {
                        if let Some(htto_directive) = &mut field_.http {
                            if let Some(base_url) = htto_directive.base_url.to_owned() {
                                if base_url == common_url {
                                    htto_directive.base_url = None;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            tracing::warn!(
                "Threshold matching base url not found, transformation cannot be performed."
            );
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
