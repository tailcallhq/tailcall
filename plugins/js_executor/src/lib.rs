mod conversion;

use std::time::Duration;

use async_graphql_value::ConstValue;
use mini_v8::{Function, MiniV8, Script};
use once_cell::sync::Lazy;

use js_executor_interface::JsExecutor;

use conversion::ValueWrapper;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const V8: Lazy<MiniV8> = Lazy::new(|| MiniV8::new());

// TODO: may it become better?
// pub static _TYPE_CHECK: JsExecutor = eval;

struct JsPlugin {
  func: Function,
}

impl JsExecutor for JsPlugin {
  fn eval(&self, input: ConstValue) -> Result<ConstValue, String> {
    let result: mini_v8::Result<ValueWrapper> = self.func.call((ValueWrapper::from(input),));

    match result {
      Ok(v) => Ok(v.into()),
      Err(err) => {
        log::warn!("Error while executing js: {err}");
        Ok(ConstValue::Null)
      }
    }
  }
}

#[no_mangle]
pub fn create_executor(source: &str) -> Box<dyn JsExecutor> {
  let source = format!(
    "((ctx) => {})",
    source,
  );
  let script = Script { source: source, timeout: Some(Duration::from_secs(2)), origin: None };
  let func: Function = V8.eval(script).unwrap();

  Box::new(JsPlugin { func })
}
