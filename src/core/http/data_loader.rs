use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;
use tailcall_valid::Validator;

use super::transformations::{BodyBatching, QueryBatching};
use crate::core::config::group_by::GroupBy;
use crate::core::config::Batch;
use crate::core::data_loader::{DataLoader, Loader};
use crate::core::http::{DataLoaderRequest, Response};
use crate::core::json::JsonLike;
use crate::core::runtime::TargetRuntime;
use crate::core::transform::TransformerOps;
use crate::core::Transform;

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
            if cfg!(debug_assertions) {
                // Sort keys to build consistent URLs only in Testing environment.
                dl_requests.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));
            }

            if let Some(base_dl_request) = dl_requests.first().as_mut() {
                let base_request = if base_dl_request.method() == http::Method::GET {
                    QueryBatching::new(
                        &dl_requests.iter().skip(1).collect::<Vec<_>>(),
                        Some(group_by.key()),
                    )
                    .transform(base_dl_request.to_request())
                    .to_result()
                    .map_err(|e| anyhow::anyhow!(e))?
                } else {
                    QueryBatching::new(&dl_requests.iter().skip(1).collect::<Vec<_>>(), None)
                        .pipe(BodyBatching::new(&dl_requests.iter().collect::<Vec<_>>()))
                        .transform(base_dl_request.to_request())
                        .to_result()
                        .map_err(|e| anyhow::anyhow!(e))?
                };

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
                    for dl_req in dl_requests.into_iter() {
                        let body_key = dl_req.batching_value().ok_or(anyhow::anyhow!(
                            "Unable to find batching value in the body for data loader request {}",
                            dl_req.url().as_str()
                        ))?;
                        let extracted_value = data_extractor(&response_map, body_key);
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
