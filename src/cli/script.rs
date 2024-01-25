use std::cell::RefCell;

use async_std::task::block_on;
use lazy_static::lazy_static;
use mini_v8::{MiniV8, Value, Values};

use crate::channel::{Command, Event, SerdeV8};
use crate::ScriptIO;

thread_local! {

  static CLOSURE: RefCell<anyhow::Result<mini_v8::Value>> = const { RefCell::new(Ok(mini_v8::Value::Null))};
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
      return function(event) {{
        return JSON.stringify(onEvent(JSON.parse(event)));
      }}
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
            let closure: anyhow::Result<mini_v8::Function> = Self::init(v8, script);
            if let Err(e) = &closure {
              log::error!("JS Initialization Failure: {}", e.to_string());
            };
            let _ = CLOSURE.replace(closure.map(mini_v8::Value::Function));
          })
        })
        .await
        .unwrap()
    });

    Self {}
  }

  fn init(v8: &MiniV8, script: String) -> anyhow::Result<mini_v8::Function> {
    let value: mini_v8::Value = v8
      .eval(create_closure(script.as_str()))
      .map_err(|e| anyhow::anyhow!(e.to_string()))?;
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
        let command = CLOSURE.with_borrow(|closure| {
          let v8 = V8.with_borrow(|x| x.clone());
          on_event_impl(&v8, closure, event)
        });

        if let Err(e) = &command {
          log::error!("JS Runtime Failure:{:?}", e);
        }

        command
      })
      .await?
  }
}

fn on_event_impl<'a>(
  v8: &'a MiniV8,
  closure: &'a anyhow::Result<mini_v8::Value>,
  event: Event,
) -> anyhow::Result<Command> {
  log::info!("on_event: {:?}", event);
  let err = &anyhow::anyhow!("expected an 'onEvent' function");
  let on_event = closure
    .as_ref()
    .and_then(|a| a.as_function().ok_or(err))
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

  let args = serde_json::to_value(event.clone())?
    .to_v8(v8)
    .map_err(|e| anyhow::anyhow!("Event encoding failure: {}", e.to_string()))?;

  let value = on_event
    .call(Values::from_vec(vec![args]))
    .map_err(|e| anyhow::anyhow!("Function invocation failure: {}", e.to_string()))?;

  match value {
    Value::Undefined => {
      if let Some(req) = event.request() {
        Ok(Command::Continue(req.clone()))
      } else {
        anyhow::bail!("Event not handled: {:?}", event)
      }
    }
    _ => {
      let serde_value = serde_json::Value::from_v8(value)?;
      let command = serde_json::from_value::<Command>(serde_value)?;
      Ok(command)
    }
  }
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
