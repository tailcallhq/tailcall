use std::cell::RefCell;

use async_std::task::block_on;
use lazy_static::lazy_static;
use mini_v8::{MiniV8, Value};

use crate::{JSValue, ScriptIO};

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
            let closure: Value = v8.eval(create_closure(script.as_str())).unwrap();
            CLOSURE.replace(closure);
            Self {}
          })
        })
        .await
        .unwrap()
    })
  }
}

#[async_trait::async_trait]
impl<Event: JSValue, Command: JSValue> ScriptIO<Event, Command> for JSEngine {
  async fn on_event(&self, event: Event) -> anyhow::Result<Command> {
    let command = RUNTIME
      .spawn(async move {
        CLOSURE.with_borrow(|closure| {
          let command: Command = Command::from_value(
            closure
              .as_function()
              .expect("expected function")
              .call(event.to_values())
              .expect("failed to call function"),
          );
          command
        })
      })
      .await?;
    Ok(command)
  }
}

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;
  use serial_test::serial;

  use crate::cli::script::JSEngine;
  use crate::ScriptIO;

  #[serial]
  #[tokio::test]
  async fn test_call_once() {
    let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}".into());
    let actual = ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
    let expected = 1.0;
    assert_eq!(actual, expected);
  }

  #[serial]
  #[tokio::test]
  async fn test_call_many() {
    let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}".into());
    ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
    ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
    ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
    let actual = ScriptIO::<(), f64>::on_event(&engine, ()).await.unwrap();
    let expected = 4.0;
    assert_eq!(actual, expected);
  }
}
