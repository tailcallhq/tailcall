use url::Url;

pub fn extract_base_url(url: &Url) -> Option<String> {
    match url.host_str() {
        Some(host) => match url.port() {
            Some(port) => Some(format!("{}://{}:{}", url.scheme(), host, port)),
            None => Some(format!("{}://{}", url.scheme(), host)),
        },
        None => None
    }
}
