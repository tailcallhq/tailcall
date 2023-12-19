use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use async_graphql::async_trait;
use async_graphql::futures_util::future::join_all;

use super::protobuf::ProtobufOperation;
use super::request::execute_operation_request;
use crate::config::Batch;
use crate::data_loader::{DataLoader, Loader};
use crate::http::{DataLoaderRequest, HttpClient, Response};

#[derive(Clone)]
pub struct GrpcDataLoader {
  client: Arc<dyn HttpClient>,
  operation: ProtobufOperation,
}

impl GrpcDataLoader {
  pub fn new(client: Arc<dyn HttpClient>, operation: ProtobufOperation) -> Self {
    GrpcDataLoader { client, operation }
  }

  pub fn to_data_loader(self, batch: Batch) -> DataLoader<DataLoaderRequest, GrpcDataLoader> {
    DataLoader::new(self, tokio::spawn)
      .delay(Duration::from_millis(batch.delay as u64))
      .max_batch_size(batch.max_size)
  }

  async fn load_dedupe_only(&self, keys: &[DataLoaderRequest]) -> anyhow::Result<HashMap<DataLoaderRequest, Response>> {
    let results = keys.iter().map(|key| async {
      let result = execute_operation_request(self.client.deref(), &self.operation, key.to_request()).await;

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

  async fn load_with_batch(&self, _keys: &[DataLoaderRequest]) -> anyhow::Result<HashMap<DataLoaderRequest, Response>> {
    todo!()
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

    self.load_dedupe_only(keys).await.map_err(Arc::new)
  }
}
