use std::sync::Arc;
use std::time::Duration;

use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute};
use async_graphql::{Response, ServerError};
use async_graphql_value::ConstValue;
use tokio::time::timeout;

pub struct GlobalTimeout;

impl ExtensionFactory for GlobalTimeout {
  fn create(&self) -> Arc<dyn Extension> {
    Arc::new(GlobalTimeoutExtension)
  }
}

struct GlobalTimeoutExtension;

#[async_trait::async_trait]
impl Extension for GlobalTimeoutExtension {
  async fn execute(&self, ctx: &ExtensionContext<'_>, operation_name: Option<&str>, next: NextExecute<'_>) -> Response {
    let future = next.run(ctx, operation_name);
    if let ConstValue::Number(number) = ctx.data_unchecked::<ConstValue>() {
      let timeout_duration = number.as_u64().unwrap_or(0);
      if timeout_duration > 0 {
        let result = timeout(Duration::from_millis(timeout_duration), future).await;
        match result {
          Ok(result) => result,
          Err(_) => {
            let mut response = Response::new(ConstValue::Null);
            response.errors = vec![ServerError::new("Global timeout".to_string(), None)];
            response
          }
        }
      } else {
        future.await
      }
    } else {
      future.await
    }
  }
}
