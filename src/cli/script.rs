use anyhow::Ok;
use mini_v8::{FromValue, Script, ToValues, Value};
use reqwest::{Request, Response};

use crate::{ScriptEngine, ScriptEventContext};

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

impl ScriptEngine for JSEngine {
  type Output = EventClosure;
  fn new_event_context(&self) -> anyhow::Result<impl ScriptEventContext> {
    let func = self
      .v8
      .eval(self.script.clone())
      .map_err(|e| anyhow::anyhow!("JS Evaluation Error: {}", e.to_string()))?;
    if let Value::Function(js_function) = func {
      Ok(JSScriptRequestContext { js_function })
    } else {
      Err(anyhow::anyhow!("Expected a JS Function, but got {:?}", func))
    }
  }

  fn create_closure(&self) -> anyhow::Result<Self::Output> {
    let value: Value = self
      .v8
      .eval(self.script.clone())
      .map_err(|e| anyhow::anyhow!("JS Evaluation Error: {}", e.to_string()))?;

    EventClosure::try_from(value)
  }
}

struct JSScriptRequestContext {
  js_function: mini_v8::Function,
}

impl ScriptEventContext for JSScriptRequestContext {
  type Event = Event;
  type Command = Command;
  fn evaluate(&self, event: Self::Event) -> anyhow::Result<Self::Command> {
    let command = self
      .js_function
      .call::<Self::Event, Self::Command>(event)
      .map_err(|e| anyhow::anyhow!("Evaluation Error: {}", e.to_string()))?;

    Ok(command)
  }
}
pub enum Event {
  Empty,
  Request(Request),
  Response(Response),
}

impl ToValues for Event {
  fn to_values(self, _mv8: &mini_v8::MiniV8) -> mini_v8::Result<mini_v8::Values> {
    todo!()
  }
}

pub enum Command {
  Request(Vec<Request>),
  Response(Response),
}

impl FromValue for Command {
  fn from_value(_value: Value, _mv8: &mini_v8::MiniV8) -> mini_v8::Result<Self> {
    todo!()
  }
}

#[derive(Debug)]
pub struct EventClosure {
  handler: mini_v8::Function,
}

impl EventClosure {
  pub fn on_event<A: ToValues>(&self, args: A) -> anyhow::Result<Value> {
    self
      .handler
      .call::<A, Value>(args)
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

  use pretty_assertions::assert_eq;

  use crate::{cli::script::JSEngine, ScriptEngine};

  #[test]
  fn test_shared_context() {
    let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}");
    let closure = engine.create_closure().unwrap();
    let actual = closure.on_event(()).unwrap().as_number().unwrap();
    let expected = 1.0;
    assert_eq!(actual, expected);
    let actual = closure.on_event(()).unwrap().as_number().unwrap();
    let expected = 2.0;
    assert_eq!(actual, expected);
  }

  #[test]
  fn test_separate_context() {
    let engine = JSEngine::new("let state = 0; function onEvent() {state += 1; return state}");
    let ctx_1 = engine.create_closure().unwrap();
    let ctx_2 = engine.create_closure().unwrap();

    let actual_1 = ctx_1.on_event(()).unwrap().as_number().unwrap();
    let expected_1 = 1.0;
    assert_eq!(actual_1, expected_1);

    let actual_2 = ctx_2.on_event(()).unwrap().as_number().unwrap();
    let expected_2 = 1.0;

    assert_eq!(actual_2, expected_2);
  }
}
