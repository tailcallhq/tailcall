use std::collections::{HashMap, HashSet};

use crate::core::config::transformer::Transform;
use crate::core::config::Config;
use crate::core::valid::Valid;

struct UrlTypeMapping {
    url_to_type_map: HashMap<String, HashSet<String>>,
}

impl UrlTypeMapping {
    fn new() -> Self {
        Self { url_to_type_map: Default::default() }
    }

    fn populate_url_type_map(&mut self, config: &Config) {
        for (type_name, type_) in config.types.iter() {
            for field_ in type_.fields.values() {
                if let Some(http_directive) = &field_.http {
                    if let Some(base_url) = &http_directive.base_url {
                        self.url_to_type_map
                            .entry(base_url.to_owned())
                            .or_default()
                            .insert(type_name.to_owned());
                    }
                }
            }
        }
    }

    fn find_common_url(&self, threshold: f32) -> Option<(String, HashSet<String>)> {
        let count_of_unique_base_urls = self.url_to_type_map.len();
        for (base_url, type_set) in &self.url_to_type_map {
            if type_set.len() >= ((count_of_unique_base_urls as f32) * threshold) as usize {
                return Some((base_url.to_owned(), type_set.to_owned()));
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
        url_type_mapping.populate_url_type_map(&config);

        if let Some((common_url, visited_type_set)) =
            url_type_mapping.find_common_url(self.threshold)
        {
            config.upstream.base_url = Some(common_url.to_owned());

            for type_name in visited_type_set {
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
