mod conversion;

use std::time::Duration;

use async_graphql_value::ConstValue;
use conversion::ValueWrapper;
use js_executor_interface::JsExecutor;
use mimalloc::MiMalloc;
use mini_v8::{Function, MiniV8, Script};
use once_cell::sync::Lazy;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const V8: Lazy<MiniV8> = Lazy::new(|| MiniV8::new());

// TODO: may it become better?
// pub static _TYPE_CHECK: JsExecutor = eval;

struct JsPlugin {
  func: Function,
  with_input: bool,
}

impl JsExecutor for JsPlugin {
  fn eval(&self, input: ConstValue) -> Result<ConstValue, String> {
    let result: mini_v8::Result<ValueWrapper> = if self.with_input {
      self.func.call((ValueWrapper::from(input),))
    } else {
      self.func.call(())
    };

    match result {
      Ok(v) => Ok(v.into()),
      Err(err) => {
        log::warn!("Error while executing js: {err}");
        Ok(ConstValue::Null)
      }
    }
  }
}

// TODO: add type validation
#[no_mangle]
pub fn create_executor(source: &str, with_input: bool) -> Box<dyn JsExecutor> {
  let source = if with_input {
    format!("((ctx) => {})", source)
  } else {
    format!("(() => {})", source)
  };
  let script = Script { source: source, timeout: Some(Duration::from_secs(2)), origin: None };
  let func: Function = V8.eval(script).unwrap();

  Box::new(JsPlugin { func, with_input })
}
