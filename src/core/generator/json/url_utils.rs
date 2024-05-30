use url::Url;

pub fn extract_base_url(url: &Url) -> Option<String> {
    match url.host_str() {
        Some(host) => match url.port() {
            Some(port) => Some(format!("{}://{}:{}", url.scheme(), host, port)),
            None => Some(format!("{}://{}", url.scheme(), host)),
        },
        None => None,
    }
}

#[cfg(test)]
mod test {
    use url::Url;

    use super::*;

    #[test]
    fn test_extract_base_url_with_port() {
        let url = Url::parse("http://example.com:8080/path/to/resource").unwrap();
        assert_eq!(
            extract_base_url(&url),
            Some("http://example.com:8080".to_string())
        );
    }

    #[test]
    fn test_extract_base_url_without_port() {
        let url = Url::parse("https://subdomain.example.org").unwrap();
        assert_eq!(
            extract_base_url(&url),
            Some("https://subdomain.example.org".to_string())
        );
    }

    #[test]
    fn test_extract_base_url_with_ip_address() {
        let url = Url::parse("http://192.168.1.1:8080/path/to/resource").unwrap();
        assert_eq!(
            extract_base_url(&url),
            Some("http://192.168.1.1:8080".to_string())
        );
    }
}
