use lazy_static::lazy_static;
use sysinfo::{System, SystemExt};

lazy_static! {
    static ref TOTAL_MEMORY: u64 = {
        let mut sys = System::new();
        sys.refresh_all();
        sys.get_total_memory() * 1024
    };
}

pub fn get_memory_by_percentage(percentage: u64) -> u64 {
    let total_memory: u64 = *TOTAL_MEMORY;
    let clamped_percentage = std::cmp::min(percentage, 100);
    let memory_by_percentage =
        ((total_memory as f64) * (clamped_percentage as f64) / 100.0).ceil() as u64;
    std::cmp::min(memory_by_percentage, u64::MAX)
}

#[cfg(test)]
mod tests {

    use hyper::body::Bytes;
    use reqwest::Method;
    use tokio;

    use crate::cli::runtime::NativeHttp;
    use crate::core::blueprint::Upstream;
    use crate::core::config::HttpCache;
    use crate::core::http::Response;
    use crate::core::HttpIO;

    fn start_mock_server() -> httpmock::MockServer {
        httpmock::MockServer::start()
    }

    async fn make_request(request_url: &str, native_http: &NativeHttp) -> Response<Bytes> {
        let request = reqwest::Request::new(Method::GET, request_url.parse().unwrap());
        let result = native_http.execute(request).await;
        result.unwrap()
    }

    #[tokio::test]
    async fn test_native_http_get_request_with_cache_percentage_based() {
        let server = start_mock_server();

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test-1");
            then.status(200).body("Hello");
        });
        //1% of total memory will be allocated for cache as max size, if goes beyond
        // max size lru policy will be applied.

        let http_cache = HttpCache { enable: true, size: 1 };
        let upstream = Upstream { http_cache: Some(http_cache), ..Default::default() };
        let native_http = NativeHttp::init(&upstream, &Default::default());
        let port = server.port();

        let url1 = format!("http://localhost:{}/test-1", port);
        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");

        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "HIT");
    }
}
