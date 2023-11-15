use std::collections::HashMap;
use std::sync::Arc;

use async_graphql::dataloader::{DataLoader, NoCache};
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;

use super::http_list_data_loader::HttpListDataLoader;
use super::http_object_data_loader::HttpObjectDataLoader;
use crate::config::group_by::GroupBy;
use crate::http::{DataLoaderRequest, HttpClient, Response};
use crate::json::JsonLike;

pub enum DataLoaderType {
  Object(DataLoader<HttpObjectDataLoader, NoCache>),
  List(DataLoader<HttpListDataLoader, NoCache>),
}

pub struct LoaderOptions {
  pub batched: Option<GroupBy>,
  pub client: Arc<dyn HttpClient>,
}

pub async fn get_result_map(
  loader_options: LoaderOptions,
  keys: &[DataLoaderRequest],
  get_body_value: fn(HashMap<String, Vec<&ConstValue>>, &str) -> ConstValue,
) -> Result<HashMap<DataLoaderRequest, Response>, anyhow::Error> {
  if let Some(group_by) = loader_options.batched.clone() {
    let mut keys = keys.to_vec();
    keys.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));

    let mut request = keys[0].to_request();
    let first_url = request.url_mut();

    for key in &keys[1..] {
      let request = key.to_request();
      let url = request.url();
      first_url.query_pairs_mut().extend_pairs(url.query_pairs());
    }

    let res = loader_options.client.execute(request).await?;

    #[allow(clippy::mutable_key_type)]
    let mut hashmap: HashMap<DataLoaderRequest, Response> = HashMap::with_capacity(keys.len());
    let path = &group_by.path();
    let body_value = res.body.group_by(path);

    for key in keys {
      let req = key.to_request();
      let query_set: std::collections::HashMap<_, _> = req.url().query_pairs().collect();
      let id = query_set
        .get(group_by.key())
        .ok_or(anyhow::anyhow!("Unable to find key {} in query params", group_by.key()))?;
      hashmap.insert(key.clone(), res.clone().body(get_body_value(body_value.clone(), id)));
    }

    Ok(hashmap)
  } else {
    let results = keys.iter().map(|key| async {
      let result = loader_options.client.execute(key.to_request()).await;
      (key.clone(), result)
    });

    let results = join_all(results).await;

    #[allow(clippy::mutable_key_type)]
    let mut hashmap = HashMap::new();
    for (key, value) in results {
      hashmap.insert(key, value?);
    }

    Ok(hashmap)
  }
}
