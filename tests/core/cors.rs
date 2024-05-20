#[cfg(test)]
mod integration_tests {
    use super::*;
    use hyper::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_HEADERS};
    use hyper::{Body, Request, Response, StatusCode};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_cors_for_tailcall_run() {
        let app_ctx = Arc::new(AppContext::default());
        let mut req_counter = RequestCounter::new(&app_ctx.blueprint.telemetry, &Request::default());

        // It simulates a request from "https://tailcall.run"
        let req = Request::builder()
            .method("OPTIONS")
            .header("Origin", "https://tailcall.run")
            .body(Body::empty())
            .unwrap();

        let response = handle_origin_tailcall::<GraphQLRequestLike>(req, app_ctx, &mut req_counter).await.unwrap();

        // It checks the CORS headers in the response
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN), Some(&HeaderValue::from_static("https://tailcall.run")));
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_METHODS), Some(&HeaderValue::from_static("GET, POST, OPTIONS")));
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_HEADERS), Some(&HeaderValue::from_static("*")));
    }
}