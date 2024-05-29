use url::Url;

use crate::core::config::Config;

use super::ConfigGenerator;

pub struct UpstreamGenerator {
    url: Url,
}

impl UpstreamGenerator {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

impl ConfigGenerator for UpstreamGenerator {
    fn apply(&mut self, mut config: Config) -> Config {
        let base_url = match self.url.host_str() {
            Some(host) => match self.url.port() {
                Some(port) => format!("{}://{}:{}", self.url.scheme(), host, port),
                None => format!("{}://{}", self.url.scheme(), host),
            },
            None => self.url.to_string(),
        };

        config.upstream.base_url = Some(base_url);
        config
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use crate::core::generator::json::ConfigGenerator;

    use super::UpstreamGenerator;


    #[test]
    fn test_apply_with_host_and_port() {
        let url = Url::parse("http://example.com:8080").unwrap();
        let mut generator = UpstreamGenerator::new(url);
        let updated_config = generator.apply(Default::default());
        assert_eq!(
            updated_config.upstream.base_url,
            Some("http://example.com:8080".to_string())
        );
    }

    #[test]
    fn test_apply_with_host_without_port() {
        let url = Url::parse("http://example.com").unwrap();
        let mut generator = UpstreamGenerator::new(url);
        let updated_config = generator.apply(Default::default());

        assert_eq!(
            updated_config.upstream.base_url,
            Some("http://example.com".to_string())
        );
    }

    #[test]
    fn test_apply_with_https_scheme() {
        let url = Url::parse("https://example.com").unwrap();
        let mut generator = UpstreamGenerator::new(url);
        let updated_config = generator.apply(Default::default());

        assert_eq!(
            updated_config.upstream.base_url,
            Some("https://example.com".to_string())
        );
    }
}
