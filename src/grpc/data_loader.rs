use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;
use async_graphql_value::ConstValue;
use prost_reflect::DynamicMessage;

use crate::config::{Batch, GrpcBatchOperation};
use crate::data_loader::{DataLoader, Loader};
use crate::http::{HttpClient, Response};
use crate::json::JsonLike;

use super::data_loader_request::DataLoaderRequest;
use super::protobuf::{get_field_value_as_str, ProtobufOperation};
use super::request::execute_grpc_request;

#[derive(Clone)]
pub struct GrpcDataLoader {
  pub(crate) client: Arc<dyn HttpClient>,
  pub(crate) operation: ProtobufOperation,
  pub(crate) batch: Option<GrpcBatchOperation>,
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
        Err(error) => Err(error.into()),
      };

      // TODO: do we have to clone keys here? join_all seems like returns the results in passed order
      return (key.clone(), result);
    });

    let results = join_all(results).await;

    #[allow(clippy::mutable_key_type)]
    let mut hashmap = HashMap::new();
    for (key, value) in results {
      hashmap.insert(key, value?);
    }

    Ok(hashmap)
  }

  async fn load_with_batch(
    &self,
    batch: &GrpcBatchOperation,
    keys: &[DataLoaderRequest],
  ) -> Result<HashMap<DataLoaderRequest, Response>> {
    let inputs = keys
      .iter()
      .map(|key| key.to_message())
      .collect::<Result<Vec<DynamicMessage>>>()?;
    let multiple_request = batch.operation.convert_multiple_messages(&inputs)?;

    let mut first_request = keys[0].clone().to_request()?;

    // TODO: move url management to execute_grpc_request?
    let url = first_request.url_mut();
    url.set_path(&format!(
      "{}/{}",
      batch.operation.service_name(),
      batch.operation.name()
    ));
    first_request.body_mut().replace(multiple_request.into());

    let response = execute_grpc_request(self.client.deref(), &batch.operation, first_request).await?;

    let path = &batch.group_by.path();
    let response_body = response.body.group_by(path);

    let mut result = HashMap::new();

    for (key, input) in keys.iter().zip(inputs) {
      let id = get_field_value_as_str(input, batch.group_by.key())?;
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
    // TODO: don't use dataloader for grpc inside mutations
    if let Some(batch) = &self.batch {
      self.load_with_batch(batch, keys).await.map_err(|e| Arc::new(e))
    } else {
      self.load_dedupe_only(keys).await.map_err(|e| Arc::new(e))
    }
  }
}
