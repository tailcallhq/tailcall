use super::HttpClient;

pub struct Memoize {
    client: HttpClient,
    cache: moka::sync::Cache<String, String>,
}

impl Memoize {
    pub fn new(client: HttpClient) -> Self {
        Self { client, cache: moka::sync::Cache::new(u64::MAX) }
    }
}
