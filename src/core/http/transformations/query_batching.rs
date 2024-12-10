use std::convert::Infallible;

use reqwest::Request;
use tailcall_valid::Valid;

use crate::core::http::DataLoaderRequest;
use crate::core::Transform;

pub struct QueryBatching<'a> {
    dl_requests: &'a [&'a DataLoaderRequest],
    group_by: Option<&'a str>,
}

impl<'a> QueryBatching<'a> {
    pub fn new(dl_requests: &'a [&'a DataLoaderRequest], group_by: Option<&'a str>) -> Self {
        QueryBatching { dl_requests, group_by }
    }
}

impl Transform for QueryBatching<'_> {
    type Value = Request;
    type Error = Infallible;
    fn transform(&self, mut base_request: Self::Value) -> Valid<Self::Value, Self::Error> {
        // Merge query params in the request
        for key in self.dl_requests.iter() {
            let request = key.to_request();
            let url = request.url();
            let pairs: Vec<_> = if let Some(group_by_key) = self.group_by {
                url.query_pairs()
                    .filter(|(key, _)| group_by_key.eq(&key.to_string()))
                    .collect()
            } else {
                url.query_pairs().collect()
            };

            if !pairs.is_empty() {
                // if pair's are empty then don't extend the query params else it ends
                // up appending '?' to the url.
                base_request.url_mut().query_pairs_mut().extend_pairs(pairs);
            }
        }
        Valid::succeed(base_request)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use http::Method;
    use reqwest::Url;
    use tailcall_valid::Validator;

    use super::*;

    fn create_base_request() -> Request {
        Request::new(Method::GET, "http://example.com".parse().unwrap())
    }

    fn create_request_with_params(params: &[(&str, &str)]) -> DataLoaderRequest {
        let mut url = Url::parse("http://example.com").unwrap();
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, value) in params {
                query_pairs.append_pair(key, value);
            }
        }
        let request = Request::new(Method::GET, url);
        DataLoaderRequest::new(request, Default::default())
    }

    fn get_query_params(request: &Request) -> HashMap<String, String> {
        request
            .url()
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_empty_requests() {
        let requests: Vec<&DataLoaderRequest> = vec![];
        let base_request = create_base_request();

        let result = QueryBatching::new(&requests, None)
            .transform(base_request)
            .to_result()
            .unwrap();

        assert!(result.url().query().is_none());
    }

    #[test]
    fn test_single_request_no_grouping() {
        let req = create_request_with_params(&[("id", "1"), ("name", "test")]);
        let requests = vec![&req];
        let base_request = create_base_request();

        let result = QueryBatching::new(&requests, None)
            .transform(base_request)
            .to_result()
            .unwrap();

        let params = get_query_params(&result);
        assert_eq!(params.len(), 2);
        assert_eq!(params.get("id").unwrap(), "1");
        assert_eq!(params.get("name").unwrap(), "test");
    }

    #[test]
    fn test_multiple_requests_with_grouping() {
        let req1 = create_request_with_params(&[("user_id", "1"), ("extra", "data1")]);
        let req2 = create_request_with_params(&[("user_id", "2"), ("extra", "data2")]);
        let requests = vec![&req1, &req2];
        let base_request = create_base_request();

        let result = QueryBatching::new(&requests, Some("user_id"))
            .transform(base_request)
            .to_result()
            .unwrap();

        let params = get_query_params(&result);
        assert!(params.contains_key("user_id"));
        assert!(!params.contains_key("extra"));

        // URL should contain both user_ids
        let url = result.url().to_string();
        assert!(url.contains("user_id=1"));
        assert!(url.contains("user_id=2"));
    }

    #[test]
    fn test_multiple_requests_no_grouping() {
        let req1 = create_request_with_params(&[("param1", "value1"), ("shared", "a")]);
        let req2 = create_request_with_params(&[("param2", "value2"), ("shared", "b")]);
        let requests = vec![&req1, &req2];
        let base_request = create_base_request();

        let result = QueryBatching::new(&requests, None)
            .transform(base_request)
            .to_result()
            .unwrap();

        let params = get_query_params(&result);
        assert_eq!(params.get("param1").unwrap(), "value1");
        assert_eq!(params.get("param2").unwrap(), "value2");
        assert_eq!(params.get("shared").unwrap(), "b");
    }

    #[test]
    fn test_requests_with_empty_params() {
        let req1 = create_request_with_params(&[("id", "1")]);
        let req2 = create_request_with_params(&[]);
        let req3 = create_request_with_params(&[("id", "3")]);
        let requests = vec![&req1, &req2, &req3];
        let base_request = create_base_request();

        let result = QueryBatching::new(&requests, Some("id"))
            .transform(base_request)
            .to_result()
            .unwrap();

        let url = result.url().to_string();
        assert!(url.contains("id=1"));
        assert!(url.contains("id=3"));
    }

    #[test]
    fn test_special_characters() {
        let req1 = create_request_with_params(&[("query", "hello world"), ("tag", "a+b")]);
        let req2 = create_request_with_params(&[("query", "foo&bar"), ("tag", "c%20d")]);
        let requests = vec![&req1, &req2];
        let base_request = create_base_request();

        let result = QueryBatching::new(&requests, None)
            .transform(base_request)
            .to_result()
            .unwrap();

        let params = get_query_params(&result);
        // Verify URL encoding is preserved
        assert!(params.values().any(|v| v.contains(" ") || v.contains("&")));
    }

    #[test]
    fn test_group_by_with_missing_key() {
        let req1 = create_request_with_params(&[("id", "1"), ("data", "test")]);
        let req2 = create_request_with_params(&[("other", "2"), ("data", "test2")]);
        let requests = vec![&req1, &req2];
        let base_request = create_base_request();

        let result = QueryBatching::new(&requests, Some("missing_key"))
            .transform(base_request)
            .to_result()
            .unwrap();

        // Should have no query parameters since grouped key doesn't exist
        assert!(result.url().query().is_none());
    }
}
