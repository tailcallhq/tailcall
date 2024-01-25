use std::cell::RefCell;

use async_std::task::block_on;
use lazy_static::lazy_static;
use mini_v8::{MiniV8, Value, Values};

use crate::channel::{Command, Event};
use crate::cli::javascript::serde_v8::SerdeV8;
use crate::ScriptIO;

thread_local! {
  static CLOSURE: RefCell<anyhow::Result<mini_v8::Value>> = const { RefCell::new(Ok(mini_v8::Value::Null))};
  static V8: RefCell<MiniV8> = RefCell::new(MiniV8::new());
}

lazy_static! {
  static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(1)
    .thread_name("mini-v8")
    .build()
    .unwrap();
}

pub struct Runtime {}

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
impl Runtime {
  pub fn new(script: String) -> Self {
    block_on(async {
      TOKIO_RUNTIME
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
    let _ = create_console(v8);

    let value: mini_v8::Value = v8
      .eval(create_closure(script.as_str()))
      .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    let function = value
      .as_function()
      .ok_or_else(|| anyhow::anyhow!("expected an 'onEvent' function"))?;
    Ok(function.clone())
  }
}

fn create_console(v8: &MiniV8) -> anyhow::Result<()> {
  let console = v8.create_object();
  console
    .set(
      "log",
      v8.create_function(|invocation| {
        let args = invocation
          .args
          .iter()
          .map(|v| serde_json::Value::from_v8(v).unwrap().to_string())
          .collect::<Vec<_>>()
          .join(",");
        log::info!("JS: {}", args);
        Ok(Value::Undefined)
      }),
    )
    .unwrap();

  v8.global().set("console", console).unwrap();
  Ok(())
}

#[async_trait::async_trait]
impl ScriptIO<Event, Command> for Runtime {
  async fn on_event(&self, event: Event) -> anyhow::Result<Command> {
    TOKIO_RUNTIME
      .spawn(async move {
        let command = CLOSURE.with_borrow(|closure| {
          let v8 = V8.with_borrow(|x| x.clone());
          on_event_impl(&v8, closure, event)
        });

        if let Err(e) = &command {
          log::error!("JS Runtime Failure: {:?}", e);
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
  log::debug!("event: {:?}", event);
  let err = &anyhow::anyhow!("expected an 'onEvent' function");
  let on_event = closure
    .as_ref()
    .and_then(|a| a.as_function().ok_or(err))
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

  let args = event.clone().to_v8(v8)?;
  log::debug!("event args: {:?}", args);
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
      let command = Command::from_v8(&value)?;
      Ok(command)
    }
  }
}
