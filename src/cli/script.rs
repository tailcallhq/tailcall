use mini_v8::{FromValue, Script, ToValues, Value};
use reqwest::{Request, Response};

use crate::{ScriptEngine, ScriptRequestContext};

pub struct JSEngine {
  v8: mini_v8::MiniV8,
  script: Script,
}

impl JSEngine {
  pub fn new(script: &str) -> Self {
    let v8 = mini_v8::MiniV8::new();
    // TODO: add timeout
    // TODO: add closure
    let script = Script::from(script);
    Self { v8, script }
  }
}

impl ScriptEngine for JSEngine {
  fn new_request_context(&self) -> anyhow::Result<impl ScriptRequestContext> {
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
}

struct JSScriptRequestContext {
  js_function: mini_v8::Function,
}

impl ScriptRequestContext for JSScriptRequestContext {
  type Event = Event;
  type Command = Command;
  fn execute(&self, event: Self::Event) -> anyhow::Result<Self::Command> {
    let command = self
      .js_function
      .call::<Self::Event, Self::Command>(event)
      .map_err(|e| anyhow::anyhow!("Evaluation Error: {}", e.to_string()))?;

    Ok(command)
  }
}
pub enum Event {
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
