#[cfg(feature = "unsafe-js")]
mod miniv8;
#[cfg(not(feature = "unsafe-js"))]
mod stub;

use std::fmt::Debug;
use std::future::Future;

use anyhow::Result;
use async_graphql_value::ConstValue;
#[cfg(feature = "unsafe-js")]
pub use miniv8::{JsPluginExecutor, JsPluginWrapper};
#[cfg(not(feature = "unsafe-js"))]
pub use stub::{JsPluginExecutor, JsPluginWrapper};

pub trait JsPluginExecutorInterface: Clone + Debug {
  fn source(&self) -> &str;
  fn call(&self, input: ConstValue) -> impl Future<Output = Result<ConstValue>>;
}

pub trait JsPluginWrapperInterface<E: JsPluginExecutorInterface>: Sized {
  fn try_new() -> Result<Self>;

  fn start(self) -> Result<()>;

  fn create_executor(&self, source: String, with_input: bool) -> E;
}
