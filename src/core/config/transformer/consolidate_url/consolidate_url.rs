use std::collections::HashSet;

use super::max_value_map::MaxValueMap;
use crate::core::config::{Config, Resolver};
use crate::core::transform::Transform;
use tailcall_valid::Valid;

struct UrlTypeMapping {
    /// maintains the url to it's frequency mapping.
    url_to_frequency_map: MaxValueMap<String, u32>,
    /// maintains the types that contains the base_url in it's fields.
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
            for field in type_.fields.values() {
                if let Some(Resolver::Http(http)) = &field.resolver {
                    if let Some(base_url) = &http.base_url {
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
            if *frequency >= (self.url_to_frequency_map.len() as f32 * threshold).ceil() as u32 {
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
        Self { threshold }
    }

    fn generate_base_url(&self, mut config: Config) -> Config {
        let mut url_type_mapping = UrlTypeMapping::new();
        url_type_mapping.populate_url_frequency_map(&config);

        if let Some(common_url) = url_type_mapping.find_common_url(self.threshold) {
            config.upstream.base_url = Some(common_url.to_owned());

            for type_name in url_type_mapping.visited_type_set {
                if let Some(type_) = config.types.get_mut(&type_name) {
                    for field in type_.fields.values_mut() {
                        if let Some(Resolver::Http(http)) = &mut field.resolver {
                            if let Some(base_url) = &http.base_url {
                                if base_url == &common_url {
                                    http.base_url = None;
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

    pub fn is_enabled(threshold: f32) -> bool {
        threshold > 0.0
    }
}

impl Transform for ConsolidateURL {
    type Value = Config;
    type Error = String;
    fn transform(&self, config: Config) -> Valid<Config, String> {
        let config = self.generate_base_url(config);
        Valid::succeed(config)
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use tailcall_fixtures::configs;

    use super::*;
    use crate::core::config::Config;
    use crate::core::transform::Transform;
    use tailcall_valid::Validator;

    fn read_fixture(path: &str) -> String {
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn should_generate_correct_upstream_when_multiple_base_urls_present() {
        let config = Config::from_sdl(read_fixture(configs::MULTI_URL_CONFIG).as_str())
            .to_result()
            .unwrap();

        let transformed_config = ConsolidateURL::new(0.5)
            .transform(config)
            .to_result()
            .unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());
    }

    #[test]
    fn should_not_generate_upstream_when_threshold_is_not_matched() {
        let config = Config::from_sdl(read_fixture(configs::MULTI_URL_CONFIG).as_str())
            .to_result()
            .unwrap();

        let transformed_config = ConsolidateURL::new(0.9)
            .transform(config)
            .to_result()
            .unwrap();
        insta::assert_snapshot!(transformed_config.to_sdl());
    }
}
