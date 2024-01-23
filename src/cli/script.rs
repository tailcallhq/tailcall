use anyhow::Ok;
use mini_v8::{FromValue, Script, ToValues, Value};

use crate::{EventHandler, ScriptEngine};

pub struct JSEngine {
  v8: mini_v8::MiniV8,
  script: Script,
}

impl JSEngine {
  fn create_closure(script: &str) -> String {
    format!(
      r#"
      (function() {{
        {}
        return {{onEvent}}
      }})    
    "#,
      script
    )
  }
  pub fn new(script: &str) -> Self {
    let v8 = mini_v8::MiniV8::new();
    let script = Self::create_closure(script);
    let mut script = Script::from(script);
    script.timeout = Some(std::time::Duration::from_millis(100));
    Self { v8, script }
  }
}

impl<Event: ToValues, Command: FromValue> ScriptEngine<Event, Command> for JSEngine {
  fn event_handler(&self) -> anyhow::Result<impl EventHandler<Event, Command>> {
    let value: Value = self
      .v8
      .eval(self.script.clone())
      .map_err(|e| anyhow::anyhow!("JS Evaluation Error: {}", e.to_string()))?;

    EventClosure::try_from(value)
  }
}

#[derive(Debug)]
pub struct EventClosure {
  handler: mini_v8::Function,
}

impl<Event: ToValues, Command: FromValue> EventHandler<Event, Command> for EventClosure {
  fn on_event(&self, event: Event) -> anyhow::Result<Command> {
    self
      .handler
      .call(event)
      .map_err(|e| anyhow::anyhow!("Evaluation Error: {}", e.to_string()))
  }
}

// TODO: use serde to decode the value
impl TryFrom<Value> for EventClosure {
  type Error = anyhow::Error;

  fn try_from(value: Value) -> Result<Self, Self::Error> {
    let closure = value.as_function().ok_or(anyhow::anyhow!("not a function"))?;
    let on_event_value = closure
      .call::<(), Value>(())
      .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let on_event_value: Value = on_event_value
      .as_object()
      .ok_or(anyhow::anyhow!("expected object"))?
      .get("onEvent")
      .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let on_event = on_event_value.as_function().ok_or(anyhow::anyhow!("not a function"))?;
    Ok(Self { handler: on_event.clone() })
  }
}

#[cfg(test)]
mod tests {
  use mini_v8::{Value, Values};
  use pretty_assertions::assert_eq;

  use crate::cli::script::JSEngine;
  use crate::{EventHandler, ScriptEngine};

  #[test]
  fn test_shared_context() {
    let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}");
    let ctx = engine.event_handler().unwrap();

    // TODO: not idiomatic Rust
    let actual = EventHandler::<Values, Value>::on_event(&ctx, Values::new())
      .unwrap()
      .as_number()
      .unwrap();
    let expected = 1.0;
    assert_eq!(actual, expected);
    let actual = ctx.on_event(Values::new()).unwrap().as_number().unwrap();
    let expected = 2.0;
    assert_eq!(actual, expected);
  }

  #[test]
  fn test_separate_context() {
    let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}");
    let ctx_1 = engine.event_handler().unwrap();
    let ctx_2 = engine.event_handler().unwrap();

    let actual_1 = EventHandler::<Values, Value>::on_event(&ctx_1, Values::new())
      .unwrap()
      .as_number()
      .unwrap();
    let expected_1 = 1.0;
    assert_eq!(actual_1, expected_1);

    let actual_2 = EventHandler::<Values, Value>::on_event(&ctx_2, Values::new())
      .unwrap()
      .as_number()
      .unwrap();
    let expected_2 = 1.0;

    assert_eq!(actual_2, expected_2);
  }
}
