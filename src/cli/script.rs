use std::cell::RefCell;

use async_std::task::block_on;
use lazy_static::lazy_static;
use mini_v8::{FromValue, MiniV8, ToValues, Value};

use crate::{Command, Event, ScriptIO};

thread_local! {
  static CLOSURE: RefCell<mini_v8::Value> = const { RefCell::new(mini_v8::Value::Null)};
  static V8: RefCell<MiniV8> = RefCell::new(MiniV8::new());
}

lazy_static! {
  static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(1)
    .thread_name("mini-v8")
    .build()
    .unwrap();
}

pub struct JSEngine {}

fn create_closure(script: &str) -> String {
  format!(
    r#"
    (function() {{
      {}
      return onEvent
    }})();
  "#,
    script
  )
}
impl JSEngine {
  pub fn new(script: String) -> Self {
    block_on(async {
      RUNTIME
        .spawn(async move {
          V8.with_borrow_mut(|v8| {
            let closure: mini_v8::Function = Self::init(v8, script).unwrap();
            CLOSURE.replace(Value::Function(closure));
            Self {}
          })
        })
        .await
        .unwrap()
    })
  }

  fn init(v8: &MiniV8, script: String) -> anyhow::Result<mini_v8::Function> {
    let value: mini_v8::Value = v8
      .eval(create_closure(script.as_str()))
      .map_err(|e| anyhow::anyhow!("failed to validate script: {}", e.to_string().replace("mini_v8::", "")))?;
    let function = value
      .as_function()
      .ok_or_else(|| anyhow::anyhow!("expected an 'onEvent' function"))?;
    Ok(function.clone())
  }
}

#[async_trait::async_trait]
impl ScriptIO<Event, Command> for JSEngine {
  async fn on_event(&self, event: Event) -> anyhow::Result<Command> {
    RUNTIME
      .spawn(async move {
        let v8 = V8.with_borrow(|x| x.clone());
        let closure = CLOSURE.with_borrow(|x| x.clone());
        let command = on_event_impl(&v8, closure.as_function().unwrap(), event);
        log::info!("JSEngine::on_event");
        command
      })
      .await?
  }
}

fn on_event_impl(v8: &MiniV8, closure: &mini_v8::Function, event: Event) -> anyhow::Result<Command> {
  let args = event
    .to_values(v8)
    .map_err(|e| anyhow::anyhow!("Event encoding failure: {}", e.to_string()))?;

  let value = closure
    .call(args)
    .map_err(|e| anyhow::anyhow!("Function invocation failure: {}", e.to_string()))?;
  let command =
    Command::from_value(value, v8).map_err(|e| anyhow::anyhow!("Command decoding failure: {}", e.to_string()))?;
  Ok(command)
}

// #[cfg(test)]
// mod tests {
//   use pretty_assertions::assert_eq;
//   use serial_test::serial;

//   use crate::cli::script::JSEngine;
//   use crate::ScriptIO;

//   #[serial]
//   #[tokio::test]
//   async fn test_call_once() {
//     let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}".into());
//     let actual = ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
//     let expected = 1.0;
//     assert_eq!(actual, expected);
//   }

//   #[serial]
//   #[tokio::test]
//   async fn test_call_many() {
//     let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}".into());
//     ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
//     ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
//     ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
//     let actual = ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
//     let expected = 4.0;
//     assert_eq!(actual, expected);
//   }
// }
