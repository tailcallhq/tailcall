use std::convert::Infallible;

use reqwest::Request;
use tailcall_valid::Valid;

use crate::core::http::DataLoaderRequest;
use crate::core::Transform;

pub struct BodyBatching<'a> {
    dl_requests: &'a [&'a DataLoaderRequest],
}

impl<'a> BodyBatching<'a> {
    pub fn new(dl_requests: &'a [&'a DataLoaderRequest]) -> Self {
        BodyBatching { dl_requests }
    }
}

impl Transform for BodyBatching<'_> {
    type Value = Request;
    type Error = Infallible;

    // This function is used to batch the body of the requests.
    // working of this function is as follows:
    // 1. It takes the list of requests and extracts the body from each request.
    // 2. It then clubs all the extracted bodies into list format. like [body1,
    //    body2, body3]
    // 3. It does this all manually to avoid extra serialization cost.
    fn transform(&self, mut base_request: Self::Value) -> Valid<Self::Value, Self::Error> {
        let mut request_bodies = Vec::with_capacity(self.dl_requests.len());

        for req in self.dl_requests {
            if let Some(body) = req.body().and_then(|b| b.as_bytes()) {
                request_bodies.push(body);
            }
        }

        if !request_bodies.is_empty() {
            if cfg!(debug_assertions) {
                // sort the body to make it consistent for testing env.
                request_bodies.sort();
            }

            // construct serialization manually.
            let merged_body = request_bodies.iter().fold(
                Vec::with_capacity(
                    request_bodies.iter().map(|i| i.len()).sum::<usize>() + request_bodies.len(),
                ),
                |mut acc, item| {
                    if !acc.is_empty() {
                        // add ',' to separate the body from each other.
                        acc.extend_from_slice(b",");
                    }
                    acc.extend_from_slice(item);
                    acc
                },
            );

            // add list brackets to the serialized body.
            let mut serialized_body = Vec::with_capacity(merged_body.len() + 2);
            serialized_body.extend_from_slice(b"[");
            serialized_body.extend_from_slice(&merged_body);
            serialized_body.extend_from_slice(b"]");
            base_request.body_mut().replace(serialized_body.into());
        }

        Valid::succeed(base_request)
    }
}

#[cfg(test)]
mod tests {
    use http::Method;
    use reqwest::Request;
    use serde_json::json;
    use tailcall_valid::Validator;

    use super::*;
    use crate::core::http::DataLoaderRequest;

    fn create_request(body: Option<serde_json::Value>) -> DataLoaderRequest {
        let mut request = create_base_request();
        if let Some(body) = body {
            let bytes_body = serde_json::to_vec(&body).unwrap();
            request.body_mut().replace(reqwest::Body::from(bytes_body));
        }

        DataLoaderRequest::new(request, Default::default())
    }

    fn create_base_request() -> Request {
        Request::new(Method::POST, "http://example.com".parse().unwrap())
    }

    #[test]
    fn test_empty_requests() {
        let requests: Vec<&DataLoaderRequest> = vec![];
        let base_request = create_base_request();

        let result = BodyBatching::new(&requests)
            .transform(base_request)
            .to_result()
            .unwrap();

        assert!(result.body().is_none());
    }

    #[test]
    fn test_single_request() {
        let req = create_request(Some(json!({"id": 1})));
        let requests = vec![&req];
        let base_request = create_base_request();

        let request = BodyBatching::new(&requests)
            .transform(base_request)
            .to_result()
            .unwrap();

        let bytes = request
            .body()
            .and_then(|b| b.as_bytes())
            .unwrap_or_default();
        let body_str = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body_str, r#"[{"id":1}]"#);
    }

    #[test]
    fn test_multiple_requests() {
        let req1 = create_request(Some(json!({"id": 1})));
        let req2 = create_request(Some(json!({"id": 2})));
        let requests = vec![&req1, &req2];
        let base_request = create_base_request();

        let result = BodyBatching::new(&requests)
            .transform(base_request)
            .to_result()
            .unwrap();

        let body = result.body().and_then(|b| b.as_bytes()).unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, r#"[{"id":1},{"id":2}]"#);
    }

    #[test]
    fn test_requests_with_empty_bodies() {
        let req1 = create_request(Some(json!({"id": 1})));
        let req2 = create_request(None);
        let req3 = create_request(Some(json!({"id": 3})));
        let requests = vec![&req1, &req2, &req3];
        let base_request = create_base_request();

        let result = BodyBatching::new(&requests)
            .transform(base_request)
            .to_result()
            .unwrap();

        let body_bytes = result
            .body()
            .and_then(|b| b.as_bytes())
            .expect("Body should be present");
        let parsed: Vec<serde_json::Value> = serde_json::from_slice(body_bytes).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[1]["id"], 3);
    }

    #[test]
    #[cfg(test)]
    fn test_body_sorting_in_test_env() {
        let req1 = create_request(Some(json!({
            "id": 2,
            "value": "second"
        })));
        let req2 = create_request(Some(json!({
            "id": 1,
            "value": "first"
        })));
        let requests = vec![&req1, &req2];
        let base_request = create_base_request();

        let result = BodyBatching::new(&requests)
            .transform(base_request)
            .to_result()
            .unwrap();

        let body_bytes = result
            .body()
            .and_then(|b| b.as_bytes())
            .expect("Body should be present");
        let parsed: Vec<serde_json::Value> = serde_json::from_slice(body_bytes).unwrap();

        // Verify sorting by comparing the serialized form
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[0]["value"], "first");
        assert_eq!(parsed[1]["id"], 2);
        assert_eq!(parsed[1]["value"], "second");
    }

    #[test]
    fn test_complex_json_bodies() {
        let req1 = create_request(Some(json!({
            "id": 1,
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"}
            },
            "tags": ["a", "b", "c"]
        })));
        let req2 = create_request(Some(json!({
            "id": 2,
            "nested": {
                "array": [4, 5, 6],
                "object": {"key": "another"}
            },
            "tags": ["x", "y", "z"]
        })));
        let requests = vec![&req1, &req2];
        let base_request = create_base_request();

        let result = BodyBatching::new(&requests)
            .transform(base_request)
            .to_result()
            .unwrap();

        let body_bytes = result
            .body()
            .and_then(|b| b.as_bytes())
            .expect("Body should be present");
        let parsed: Vec<serde_json::Value> = serde_json::from_slice(body_bytes).unwrap();

        // Verify structure and content of both objects
        assert_eq!(parsed.len(), 2);

        // First object
        assert_eq!(parsed[0]["id"], 1);
        assert_eq!(parsed[0]["nested"]["array"], json!([1, 2, 3]));
        assert_eq!(parsed[0]["nested"]["object"]["key"], "value");
        assert_eq!(parsed[0]["tags"], json!(["a", "b", "c"]));

        // Second object
        assert_eq!(parsed[1]["id"], 2);
        assert_eq!(parsed[1]["nested"]["array"], json!([4, 5, 6]));
        assert_eq!(parsed[1]["nested"]["object"]["key"], "another");
        assert_eq!(parsed[1]["tags"], json!(["x", "y", "z"]));
    }
}
