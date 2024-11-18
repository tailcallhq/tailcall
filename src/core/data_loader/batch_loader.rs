use std::collections::HashMap;

use anyhow::Ok;
use async_graphql_value::ConstValue;
use futures_util::future::join_all;
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

pub struct BatchLoader {
    runtime: TargetRuntime,
    max_batch_size: usize,
    group_by: GroupBy,
    body: fn(&HashMap<String, Vec<&ConstValue>>, &str) -> ConstValue,
}

impl BatchLoader {
    pub fn new(
        runtime: TargetRuntime,
        group_by: GroupBy,
        is_list: bool,
        max_batch_size: usize,
    ) -> Self {
        Self {
            runtime,
            group_by,
            max_batch_size,
            body: if is_list {
                get_body_value_list
            } else {
                get_body_value_single
            },
        }
    }

    pub async fn load_batch(
        &self,
        request: Request,
    ) -> async_graphql::Result<Response<ConstValue>, anyhow::Error> {
        // query parameters that are part of the group by
        let dynamic_query_pairs = request
            .url()
            .query_pairs()
            .filter(|(k, _)| self.group_by.key().eq(&k.to_string()))
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<_>>();

        let unique_query_pairs = dynamic_query_pairs
            .clone()
            .into_iter()
            .collect::<IndexSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        // query parameters that are not part of the group by
        let static_query_pairs = request
            .url()
            .query_pairs()
            .filter(|(k, _)| !self.group_by.key().eq(&k.to_string()))
            .collect::<Vec<_>>();

        let mut requests = vec![];
        let batch_size = self.max_batch_size - static_query_pairs.len();

        // Check if the number of query parameters exceeds the maximum batch size
        if dynamic_query_pairs.len() <= batch_size {
            requests.push(request);
        } else {
            // Split the query parameters into chunks based on max_batch_size
            let batches = unique_query_pairs.chunks(batch_size);
            for batch in batches {
                // Build a new set of query parameters for the current batch
                let mut new_request = request.try_clone().unwrap_or_else(|| {
                    // fail safe, clone it manually.
                    let mut req =
                        reqwest::Request::new(request.method().clone(), request.url().clone());
                    req.headers_mut().extend(request.headers().clone());
                    req
                });
                new_request.url_mut().query_pairs_mut().clear();
                new_request
                    .url_mut()
                    .query_pairs_mut()
                    .extend_pairs(static_query_pairs.clone());
                new_request.url_mut().query_pairs_mut().extend_pairs(batch);
                requests.push(new_request);
            }
        }

        // Execute all batched requests concurrently
        let results = join_all(requests.into_iter().map(|req| self.load_one(req)))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, anyhow::Error>>()?;

        let merged_results: HashMap<String, Response<ConstValue>> =
            results.into_iter().flatten().collect();

        let mut final_result: Vec<ConstValue> = vec![];
        let mut response = None;
        for (_, v) in dynamic_query_pairs {
            if let Some(res) = merged_results.get(&v) {
                final_result.push(res.body.clone());
                response = Some(res.clone());
            }
        }

        let merged_response = ConstValue::List(final_result);
        Ok(response.unwrap().body(merged_response))
    }

    async fn load_one(
        &self,
        request: Request,
    ) -> async_graphql::Result<HashMap<String, Response<ConstValue>>, anyhow::Error> {
        let query_set = request
            .url()
            .query_pairs()
            .filter(|(k, _)| k.eq(&self.group_by.key()))
            .map(|(_, v)| v.to_string())
            .collect::<Vec<_>>();

        let response = self
            .runtime
            .http
            .execute(request)
            .await?
            .to_json::<ConstValue>()?;

        let response_map = response.body.group_by(&self.group_by.path());
        let mut map = HashMap::new();
        for id in query_set {
            let body = (self.body)(&response_map, &id);
            let res = response.clone().body(body);
            map.insert(id, res);
        }

        Ok(map)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_batch_loader() {
        let url = "http://jsonplaceholder.typicode.com/users?static=12&id=1&id=1&id=1&id=1&id=1&id=1&id=1&id=1&id=1&id=1&id=2&id=2&id=2&id=2&id=2&id=2&id=2&id=2&id=2&id=2&id=3&id=3&id=3&id=3&id=3&id=3&id=3&id=3&id=3&id=3&id=4&id=4&id=4&id=4&id=4&id=4&id=4&id=4&id=4&id=4&id=5&id=5&id=5&id=5&id=5&id=5&id=5&id=5&id=5&id=5&id=6&id=6&id=6&id=6&id=6&id=6&id=6&id=6&id=6&id=6&id=7&id=7&id=7&id=7&id=7&id=7&id=7&id=7&id=7&id=7&id=8&id=8&id=8&id=8&id=8&id=8&id=8&id=8&id=8&id=8&id=9&id=9&id=9&id=9&id=9&id=9&id=9&id=9&id=9&id=9&id=10&id=10&id=10&id=10&id=10&id=10&id=10&id=10&id=10&id=10";
        let rt = crate::core::runtime::test::init(None);
        let batch_loader = BatchLoader::new(
            rt,
            GroupBy::new(vec!["id".into()], Some("id".into())),
            false,
            10,
        );

        let request = Request::new(reqwest::Method::GET, url.parse().unwrap());
        let result = batch_loader.load_batch(request).await.unwrap();
        println!("[Finder]: result: {:#?}", result);
    }
}
