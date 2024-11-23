use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;

use crate::core::config::group_by::GroupBy;
use crate::core::config::Batch;
use crate::core::data_loader::{DataLoader, Loader};
use crate::core::http::{DataLoaderRequest, Response};
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

#[derive(Clone)]
pub struct HttpDataLoader {
    pub runtime: TargetRuntime,
    pub group_by: Option<GroupBy>,
    is_list: bool,
}
impl HttpDataLoader {
    pub fn new(runtime: TargetRuntime, group_by: Option<GroupBy>, is_list: bool) -> Self {
        HttpDataLoader { runtime, group_by, is_list }
    }

    pub fn to_data_loader(self, batch: Batch) -> DataLoader<DataLoaderRequest, HttpDataLoader> {
        DataLoader::new(self)
            .delay(Duration::from_millis(batch.delay as u64))
            .max_batch_size(batch.max_size.unwrap_or_default())
    }
}

fn get_key<'a>(value: &'a serde_json::Value, path: &[String]) -> anyhow::Result<&'a str> {
    value
        .get_path(path)
        .and_then(|k| k.as_str())
        .ok_or(anyhow::anyhow!(
            "Unable to find key {} in body",
            path.join(" ")
        ))
}

#[async_trait::async_trait]
impl Loader<DataLoaderRequest> for HttpDataLoader {
    type Value = Response<async_graphql::Value>;
    type Error = Arc<anyhow::Error>;

    async fn load(
        &self,
        keys: &[DataLoaderRequest],
    ) -> async_graphql::Result<HashMap<DataLoaderRequest, Self::Value>, Self::Error> {
        if let Some(group_by) = &self.group_by {
            let query_name = group_by.key();
            let mut dl_requests = keys.to_vec();

            // Sort keys to build consistent URLs
            if cfg!(feature = "integration_test") || cfg!(test) {
                dl_requests.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));
            }

            // Create base request
            let mut base_request = dl_requests[0].to_request();
            // TODO: add the body as is in the DalaLoaderRequest.
            let mut body_mapping = HashMap::with_capacity(dl_requests.len());

            if dl_requests[0].method() == reqwest::Method::POST {
                // run only for POST requests.
                let mut arr = Vec::with_capacity(dl_requests.len());
                for req in dl_requests.iter() {
                    if let Some(body) = req.body().and_then(|b| b.as_bytes()) {
                        let value = serde_json::from_slice::<serde_json::Value>(body)
                            .map_err(|e| anyhow::anyhow!("Unable to deserialize body: {}", e))?;
                        body_mapping.insert(req, value);
                        arr.push(body);
                    }
                }

                if !arr.is_empty() {
                    if cfg!(feature = "integration_test") || cfg!(test) {
                        arr.sort();
                    }

                    // construct serialization manually.
                    let arr = arr.iter().fold(
                        Vec::with_capacity(arr.iter().map(|i| i.len()).sum::<usize>() + arr.len()),
                        |mut acc, item| {
                            if !acc.is_empty() {
                                acc.extend_from_slice(b",");
                            }
                            acc.extend_from_slice(item);
                            acc
                        },
                    );

                    // add list brackets to the serialized body.
                    let mut serialized_arr = Vec::with_capacity(arr.len() + 2);
                    serialized_arr.extend_from_slice(b"[");
                    serialized_arr.extend_from_slice(&arr);
                    serialized_arr.extend_from_slice(b"]");
                    base_request.body_mut().replace(serialized_arr.into());
                }
            }

            // Merge query params in the request
            for key in &dl_requests[1..] {
                let request = key.to_request();
                let url = request.url();
                let pairs: Vec<_> = url
                    .query_pairs()
                    .filter(|(key, _)| group_by.key().eq(&key.to_string()))
                    .collect();
                if !pairs.is_empty() {
                    base_request.url_mut().query_pairs_mut().extend_pairs(pairs);
                }
            }

            // Dispatch request
            let res = self
                .runtime
                .http
                .execute(base_request)
                .await?
                .to_json::<ConstValue>()?;

            // Create a response HashMap
            #[allow(clippy::mutable_key_type)]
            let mut hashmap = HashMap::with_capacity(dl_requests.len());

            // Parse the response body and group it by batchKey
            let path = &group_by.path();

            // ResponseMap contains the response body grouped by the batchKey
            let response_map = res.body.group_by(path);

            // depending on graphql type, it will extract the data out of the response.
            let data_extractor = if self.is_list {
                get_body_value_list
            } else {
                get_body_value_single
            };

            // For each request and insert its corresponding value
            if dl_requests[0].method() == reqwest::Method::GET {
                for dl_req in dl_requests.iter() {
                    let url = dl_req.url();
                    let query_set: HashMap<_, _> = url.query_pairs().collect();
                    let id = query_set.get(query_name).ok_or(anyhow::anyhow!(
                        "Unable to find key {} in query params",
                        query_name
                    ))?;

                    // Clone the response and set the body
                    let body = data_extractor(&response_map, id);
                    let res = res.clone().body(body);

                    hashmap.insert(dl_req.clone(), res);
                }
            } else {
                let path = group_by.body_key();

                for (dl_req, body) in body_mapping.into_iter() {
                    // retrive the key from body
                    let extracted_value = data_extractor(&response_map, get_key(&body, &path)?);
                    let res = res.clone().body(extracted_value);
                    hashmap.insert(dl_req.clone(), res);
                }
            }

            Ok(hashmap)
        } else {
            let results = keys.iter().map(|key| async {
                let result = self.runtime.http.execute(key.to_request()).await;
                (key.clone(), result)
            });

            let results = join_all(results).await;

            #[allow(clippy::mutable_key_type)]
            let mut hashmap = HashMap::new();
            for (key, value) in results {
                hashmap.insert(key, value?.to_json()?);
            }

            Ok(hashmap)
        }
    }
}
