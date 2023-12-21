use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;

use super::data_loader_request::DataLoaderRequest;
use super::protobuf::ProtobufOperation;
use super::request::execute_grpc_request;
use crate::config::group_by::GroupBy;
use crate::config::Batch;
use crate::data_loader::{DataLoader, Loader};
use crate::grpc::request::create_grpc_request;
use crate::http::{HttpClient, Response};
use crate::json::JsonLike;

#[derive(Clone)]
pub struct GrpcDataLoader {
  pub(crate) client: Arc<dyn HttpClient>,
  pub(crate) operation: ProtobufOperation,
  pub(crate) group_by: Option<GroupBy>,
}

impl GrpcDataLoader {
  pub fn to_data_loader(self, batch: Batch) -> DataLoader<DataLoaderRequest, GrpcDataLoader> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(batch.delay as u64))
      .max_batch_size(batch.max_size)
  }

  async fn load_dedupe_only(&self, keys: &[DataLoaderRequest]) -> anyhow::Result<HashMap<DataLoaderRequest, Response>> {
    let results = keys.iter().map(|key| async {
      let result = match key.to_request() {
        Ok(req) => execute_grpc_request(self.client.deref(), &self.operation, req).await,
        Err(error) => Err(error),
      };

      // TODO: do we have to clone keys here? join_all seems like returns the results in passed order
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

  async fn load_with_group_by(
    &self,
    group_by: &GroupBy,
    keys: &[DataLoaderRequest],
  ) -> Result<HashMap<DataLoaderRequest, Response>> {
    let inputs = keys.iter().map(|key| key.template.body.as_str());
    let (multiple_body, grouped_keys) = self.operation.convert_multiple_inputs(inputs, group_by.key())?;

    let first_request = keys[0].clone();
    let multiple_request = create_grpc_request(
      first_request.template.url,
      first_request.template.headers,
      multiple_body,
    );

    let response = execute_grpc_request(self.client.deref(), &self.operation, multiple_request).await?;

    let path = &group_by.path();
    let response_body = response.body.group_by(path);

    let mut result = HashMap::new();

    for (key, id) in keys.iter().zip(grouped_keys) {
      let res = response.clone().body(
        response_body
          .get(&id)
          .and_then(|a| a.first().cloned().cloned())
          .unwrap_or(ConstValue::Null),
      );

      result.insert(key.clone(), res);
    }

    Ok(result)
  }
}

#[async_trait::async_trait]
impl Loader<DataLoaderRequest> for GrpcDataLoader {
  type Value = Response;
  type Error = Arc<anyhow::Error>;

  async fn load(
    &self,
    keys: &[DataLoaderRequest],
  ) -> async_graphql::Result<HashMap<DataLoaderRequest, Self::Value>, Self::Error> {
    if let Some(group_by) = &self.group_by {
      self.load_with_group_by(group_by, keys).await.map_err(Arc::new)
    } else {
      self.load_dedupe_only(keys).await.map_err(Arc::new)
    }
  }
}
