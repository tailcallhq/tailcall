use super::HttpClient;

#[allow(dead_code)]
pub struct Memoize {
    client: HttpClient,
    cache: moka::sync::Cache<String, String>,
}

impl Memoize {
    #[allow(dead_code)]
    pub fn new(client: HttpClient) -> Self {
        Self { client, cache: moka::sync::Cache::new(u64::MAX) }
    }
}
