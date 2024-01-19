use super::{JsPluginExecutorInterface, JsPluginWrapperInterface};
use crate::lambda::EvaluationError;

#[derive(Clone, Debug)]
pub struct JsPluginExecutor {
  source: String,
}

impl JsPluginExecutorInterface for JsPluginExecutor {
  fn source(&self) -> &str {
    &self.source
  }

  async fn call(&self, _input: async_graphql_value::ConstValue) -> anyhow::Result<async_graphql_value::ConstValue> {
    Err(EvaluationError::JSException("JS execution is disabled".to_string()).into())
  }
}

pub struct JsPluginWrapper;

impl JsPluginWrapperInterface<JsPluginExecutor> for JsPluginWrapper {
  fn try_new() -> anyhow::Result<Self> {
    Ok(Self)
  }

  fn start(self) -> anyhow::Result<()> {
    Ok(())
  }

  fn create_executor(&self, source: String, _with_input: bool) -> JsPluginExecutor {
    JsPluginExecutor { source }
  }
}
