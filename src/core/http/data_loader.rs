use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;
use reqwest::Request;

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

fn get_key<'a, T: JsonLike<'a> + Display>(value: &'a T, path: &[String]) -> anyhow::Result<String> {
    value
        .get_path(path)
        .map(|k| k.to_string())
        .ok_or_else(|| anyhow::anyhow!("Unable to find key {} in body", path.join(".")))
}

/// This function is used to batch the body of the requests.
/// working of this function is as follows:
/// 1. It takes the list of requests and extracts the body from each request.
/// 2. It then clubs all the extracted bodies into list format. like [body1,
///    body2, body3]
/// 3. It does this all manually to avoid extra serialization cost.
fn batch_request_body(mut base_request: Request, requests: &[DataLoaderRequest]) -> Request {
    let mut request_bodies = Vec::with_capacity(requests.len());

    if base_request.method() == reqwest::Method::GET {
        // in case of GET method do nothing and return the base request.
        return base_request;
    }

    for req in requests {
        if let Some(body) = req.body().and_then(|b| b.as_bytes()) {
            request_bodies.push(body);
        }
    }

    if !request_bodies.is_empty() {
        if cfg!(feature = "integration_test") || cfg!(test) {
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

    base_request
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
            if cfg!(feature = "integration_test") || cfg!(test) {
                // Sort keys to build consistent URLs only in Testing environment.
                dl_requests.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));
            }

            if let Some(base_dl_request) = dl_requests.first().as_mut() {
                // Create base request
                let mut base_request =
                    batch_request_body(base_dl_request.to_request(), &dl_requests);

                // Merge query params in the request
                for key in dl_requests.iter().skip(1) {
                    let request = key.to_request();
                    let url = request.url();
                    let pairs: Vec<_> = url
                        .query_pairs()
                        .filter(|(key, _)| group_by.key().eq(&key.to_string()))
                        .collect();
                    if !pairs.is_empty() {
                        // if pair's are empty then don't extend the query params else it ends
                        // up appending '?' to the url.
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
                if base_dl_request.method() == reqwest::Method::GET {
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
                    let path = group_by.body_path();
                    for dl_req in dl_requests.into_iter() {
                        // retrive the key from body
                        let request_body = dl_req.body_value().ok_or(anyhow::anyhow!(
                            "Unable to find body in request {}",
                            dl_req.url().as_str()
                        ))?;
                        let extracted_value =
                            data_extractor(&response_map, &get_key(request_body, path)?);
                        let res = res.clone().body(extracted_value);
                        hashmap.insert(dl_req.clone(), res);
                    }
                }

                Ok(hashmap)
            } else {
                let error_message = "This is definitely a bug in http data loaders, please report it to the maintainers.";
                Err(anyhow::anyhow!(error_message).into())
            }
        } else {
            let results = keys.iter().map(|key| async {
                let result = self.runtime.http.execute(key.to_request()).await;
                (key.clone(), result)
            });

            let results = join_all(results).await;

            #[allow(clippy::mutable_key_type)]
            let mut hashmap = HashMap::with_capacity(results.len());
            for (key, value) in results {
                hashmap.insert(key, value?.to_json()?);
            }

            Ok(hashmap)
        }
    }
}
