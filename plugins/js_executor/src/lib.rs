mod conversion;

use std::time::Duration;

use async_graphql_value::ConstValue;
use conversion::ValueWrapper;
use mini_v8::{Function, MiniV8, Script};

// required when building this as dylib
// use mimalloc::MiMalloc;
// #[global_allocator]
// static GLOBAL: MiMalloc = MiMalloc;

// should have only one instance per thread
thread_local! {
  static V8: MiniV8 = MiniV8::new();
}

pub struct JsExecutor {
  func: Function,
  with_input: bool,
}

impl JsExecutor {
  pub fn new(source: &str, with_input: bool) -> JsExecutor {
    let source = if with_input {
      format!("((ctx) => {})", source)
    } else {
      format!("(() => {})", source)
    };
    let script = Script { source, timeout: Some(Duration::from_secs(2)), origin: None };
    let func: Function = V8.with(|v8| v8.eval(script).unwrap());

    JsExecutor { func, with_input }
  }

  pub fn eval(&self, input: ConstValue) -> Result<ConstValue, String> {
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
