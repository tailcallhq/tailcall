use std::collections::HashMap;

use anyhow::Ok;
use async_graphql_value::ConstValue;
use indexmap::IndexSet;
use reqwest::Request;

use crate::core::config::group_by::GroupBy;
use crate::core::http::Response;
use crate::core::json::JsonLike;
use crate::core::runtime::TargetRuntime;

fn get_body_value_single(body_value: &HashMap<String, Vec<&ConstValue>>, id: &str) -> ConstValue {
    body_value
        .get(id)
        .and_then(|a| a.first().cloned().cloned())
        .unwrap_or(ConstValue::Null)
}

fn get_body_value_list(body_value: &HashMap<String, Vec<&ConstValue>>, id: &str) -> ConstValue {
    ConstValue::List(
        body_value
            .get(id)
            .unwrap_or(&Vec::new())
            .iter()
            .map(|&o| o.to_owned())
            .collect::<Vec<_>>(),
    )
}

/// `HttpMerge` executes HTTP requests immediately (with no batching delay) and
/// processes the response based on a `group_by` configuration.
pub struct HttpMerge {
    runtime: TargetRuntime,
}

impl HttpMerge {
    pub fn new(runtime: TargetRuntime) -> Self {
        Self { runtime }
    }

    pub async fn load(
        &self,
        group_by: &GroupBy,
        is_list: &bool,
        request: Request,
    ) -> async_graphql::Result<Response<ConstValue>, anyhow::Error> {
        let query_pairs = request
            .url()
            .query_pairs()
            .filter(|(k, _)| group_by.key().eq(&k.to_string()))
            .map(|(_, v)| (v.to_string()))
            .collect::<Vec<_>>();
        let request = RequestWrapper::from(request).request();
        let (response_map, response) = self.execute(group_by, is_list, request).await?;

        let mut final_result: Vec<ConstValue> = vec![];
        for v in query_pairs {
            if let Some(res) = response_map.get(&v) {
                final_result.push(res.body.clone());
            }
        }

        let merged_response = if final_result.len() > 1 {
            ConstValue::List(final_result)
        } else {
            final_result.remove(0)
        };
        Ok(response.body(merged_response))
    }

    async fn execute(
        &self,
        group_by: &GroupBy,
        is_list: &bool,
        request: Request,
    ) -> async_graphql::Result<
        (HashMap<String, Response<ConstValue>>, Response<ConstValue>),
        anyhow::Error,
    > {
        let body = if *is_list {
            get_body_value_list
        } else {
            get_body_value_single
        };
        let query_set = request
            .url()
            .query_pairs()
            .filter(|(k, _)| k.eq(&group_by.key()))
            .map(|(_, v)| v.to_string())
            .collect::<Vec<_>>();

        let response = self
            .runtime
            .http
            .execute(request)
            .await?
            .to_json::<ConstValue>()?;

        let response_map = response.body.group_by(&group_by.path());
        let mut map = HashMap::new();
        for id in query_set {
            let body = (body)(&response_map, &id);
            let res = response.clone().body(body);
            map.insert(id, res);
        }

        Ok((map, response))
    }
}

struct RequestWrapper {
    request: Request,
}

impl From<Request> for RequestWrapper {
    fn from(request: Request) -> Self {
        Self { request }
    }
}

impl RequestWrapper {
    /// removes the duplicate query parameters and retains the original order.
    pub fn request(mut self) -> Request {
        // retain the original order of query parameters
        // note: indexset is slow in some scenarios, if this becomes a bottleneck,
        // think about alternative approach.
        let original_query_param_order = self
            .request
            .url()
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<IndexSet<_>>();

        self.request.url_mut().query_pairs_mut().clear();
        for (key, value) in original_query_param_order {
            if value.is_empty() {
                self.request
                    .url_mut()
                    .query_pairs_mut()
                    .append_key_only(key.as_str());
            } else {
                self.request
                    .url_mut()
                    .query_pairs_mut()
                    .append_pair(key.as_str(), value.as_str());
            }
        }

        self.request
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_request_wrapper() {
        let url = "http://jsonplaceholder.typicode.com/users?static=12&id=1&id=1&id=1&id=2&id=2&id=3&id=3&id=3&id=4&id=4&id=4&id=4&id=5&id=5&id=5&id=6&id=6&id=7&id=7&id=7&id=8&id=8&id=8&id=8&id=8&id=9&id=9&id=10&id=10&id=10&id=10&id=10";
        let request_wrapper: RequestWrapper =
            Request::new(reqwest::Method::GET, url.parse().unwrap()).into();
        let request = request_wrapper.request();
        assert_eq!(
            request.url().query(),
            Some("static=12&id=1&id=2&id=3&id=4&id=5&id=6&id=7&id=8&id=9&id=10")
        );
    }

    #[test]
    fn test_request_wrapper_key_without_value() {
        let url = "http://jsonplaceholder.typicode.com/users?static=12&id";
        let request_wrapper: RequestWrapper =
            Request::new(reqwest::Method::GET, url.parse().unwrap()).into();
        let request = request_wrapper.request();
        assert_eq!(request.url().query(), Some("static=12&id"));
    }
}
