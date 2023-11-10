use std::time::Duration;

use mini_v8::{MiniV8, Script};

use js_executor_interface::JsExecutor;

use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub static _TYPE_CHECK: JsExecutor = eval;

#[no_mangle]
pub fn eval(source: &str, input: &str) -> Result<String, String> {
  let source = format!(
    "JSON.stringify(((ctx) => {})({}))",
    source,
    input
  );
  let v8 = MiniV8::new();

  // TODO: fix options
  let script = Script { source: source.clone(), timeout: Some(Duration::from_secs(2)), origin: None };
  // make function initially then just call it
  v8.eval(script).map_err(|err| err.to_string())
}
