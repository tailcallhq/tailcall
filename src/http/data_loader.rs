use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;

use crate::config::group_by::GroupBy;
use crate::config::Batch;
use crate::data_loader::{DataLoader, Loader};
use crate::http::{DataLoaderRequest, HttpClient, Response};
use crate::json::JsonLike;

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
  pub client: Arc<dyn HttpClient>,
  pub group_by: Option<GroupBy>,
  pub body: fn(&HashMap<String, Vec<&ConstValue>>, &str) -> ConstValue,
}
impl HttpDataLoader {
  pub fn new(client: Arc<dyn HttpClient>, group_by: Option<GroupBy>, is_list: bool) -> Self {
    HttpDataLoader {
      client,
      group_by,
      body: if is_list {
        get_body_value_list
      } else {
        get_body_value_single
      },
    }
  }

  pub fn to_data_loader(self, batch: Batch) -> DataLoader<DataLoaderRequest, HttpDataLoader> {
    DataLoader::new(self)
      .delay(Duration::from_millis(batch.delay as u64))
      .max_batch_size(batch.max_size)
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
      let mut keys = keys.to_vec();
      keys.sort_by(|a, b| a.to_request().url().cmp(b.to_request().url()));

      let mut request = keys[0].to_request();
      let first_url = request.url_mut();

      for key in &keys[1..] {
        let request = key.to_request();
        let url = request.url();
        first_url.query_pairs_mut().extend_pairs(url.query_pairs());
      }

      let res = self.client.execute(request, None).await?;
      #[allow(clippy::mutable_key_type)]
      let mut hashmap: HashMap<DataLoaderRequest, Response> = HashMap::with_capacity(keys.len());
      let path = &group_by.path();
      let body_value = res.body.group_by(path);

      for key in &keys {
        let req = key.to_request();
        let query_set: std::collections::HashMap<_, _> = req.url().query_pairs().collect();
        let id = query_set
          .get(group_by.key())
          .ok_or(anyhow::anyhow!("Unable to find key {} in query params", group_by.key()))?;
        hashmap.insert(key.clone(), res.clone().body((self.body)(&body_value, id)));
      }
      Ok(hashmap)
    } else {
      let results = keys.iter().map(|key| async {
        let result = self.client.execute(key.to_request(), None).await;
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
}
