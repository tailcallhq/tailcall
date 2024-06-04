use lazy_static::lazy_static;
use sysinfo::{System, SystemExt};

lazy_static! {
    static ref TOTAL_MEMORY: u64 = {
        let mut sys = System::new();
        sys.refresh_all();
        sys.get_total_memory() * 1024
    };
}

pub(crate) fn get_memory_by_percentage(percentage: String) -> u64 {
    let total_memory: u64 = *TOTAL_MEMORY;
    let clamped_percentage: f64 = percentage
        .parse()
        .map(|p| if p > 100.0 { 100.0 } else { p })
        .unwrap_or(0.0);
    let memory_by_percentage = (total_memory as f64 * clamped_percentage / 100.0).round() as u64;
    std::cmp::min(memory_by_percentage, u64::MAX)
}

#[cfg(test)]
mod tests {
    use hyper::body::Bytes;
    use reqwest::Method;
    use tokio;

    use super::*;
    use crate::cli::runtime::NativeHttp;
    use crate::core::blueprint::Upstream;
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

        let percentage = "100.0";
        let total_memory: u64 = get_memory_by_percentage(percentage.to_string());
        // Calculate the percentage needed for total_memory to be 1243(size of two
        // entries)
        let target_memory: u64 = 1243;
        let percentage: f64 = (target_memory as f64 / total_memory as f64) * 100.0;
        let percentage_str = percentage.to_string();

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test-1");
            then.status(200).body("Hello");
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test-2");
            then.status(200).body("Hello");
        });

        server.mock(|when, then| {
            when.method(httpmock::Method::GET).path("/test-3");
            then.status(200).body("Hello");
        });
        // 616 is the body length without "Hello" 616+5(size of hello in bytes) = 621 ,
        // 1242 = 621*2
        let upstream = Upstream {
            http_cache_percentage: Some(percentage_str),
            ..Default::default()
        };
        let native_http = NativeHttp::init(&upstream, &Default::default());
        let port = server.port();

        let url1 = format!("http://localhost:{}/test-1", port);
        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");

        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "HIT");

        let url2 = format!("http://localhost:{}/test-2", port);
        let resp = make_request(&url2, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");

        let resp = make_request(&url2, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "HIT");

        // now cache is full, let's make 3rd request and cache it and evict url1.
        let url3 = format!("http://localhost:{}/test-3", port);
        let resp = make_request(&url3, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");

        let resp = make_request(&url3, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "HIT");

        let resp = make_request(&url1, &native_http).await;
        assert_eq!(resp.headers.get("x-cache-lookup").unwrap(), "MISS");
    }
}
