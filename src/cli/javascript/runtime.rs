use std::cell::OnceCell;
use std::sync::Arc;
use std::thread;

use mini_v8::{MiniV8, Script};

use crate::blueprint::{self};
use crate::channel::{Command, Event};
use crate::cli::javascript::serde_v8::SerdeV8;
use crate::ScriptIO;

thread_local! {
  static LOCAL_RUNTIME: OnceCell<anyhow::Result<LocalRuntime>> = const { OnceCell::new() };
}

#[derive(Clone)]
struct LocalRuntime {
    v8: MiniV8,
    closure: mini_v8::Value,
}

impl LocalRuntime {
    fn new(script: Arc<blueprint::Script>) -> anyhow::Result<Self> {
        let v8 = MiniV8::new();
        let _ = super::shim::init(&v8);
        let script = Script {
            source: create_closure(script.source.as_str()),
            timeout: script.timeout,
            ..Default::default()
        };

        let value: mini_v8::Value = v8
            .eval(script)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let function = value
            .as_function()
            .ok_or_else(|| anyhow::anyhow!("expected an 'onEvent' function"))?;

        let closure = mini_v8::Value::Function(function.clone());

        log::debug!("mini_v8: {:?}", thread::current().name());
        Ok(Self { v8, closure })
    }
}

pub struct Runtime {
    script: Arc<blueprint::Script>,
}

fn create_closure(script: &str) -> String {
    format!("(function() {{{} return onEvent}})();", script)
}
impl Runtime {
    pub fn new(script: blueprint::Script) -> Self {
        Self { script: Arc::new(script) }
    }
}

#[async_trait::async_trait]
impl ScriptIO<Event, Command> for Runtime {
    async fn on_event(&self, event: Event) -> anyhow::Result<Command> {
        let script = self.script.clone();
        let serde_event = serde_json::to_value(event.clone())?;
        let serde_command = LOCAL_RUNTIME.with(|cell| {
            let rtm = cell
                .get_or_init(move || LocalRuntime::new(script.clone()))
                .as_ref()
                .unwrap();
            on_event_impl(rtm, serde_event)
        })?;

        match serde_command {
            serde_json::Value::Null => {
                if let Some(req) = event.request() {
                    Ok(Command::Continue(req))
                } else {
                    Err(anyhow::anyhow!("expected a request"))
                }
            }
            _ => {
                let command: Command = serde_json::from_value(serde_command)?;
                Ok(command)
            }
        }
    }
}

fn on_event_impl(
    rtm: &LocalRuntime,
    serde_event: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let closure = &rtm.closure;
    let v8 = &rtm.v8;
    log::debug!("event: {:?}", serde_event);
    let err = &anyhow::anyhow!("expected an 'onEvent' function");
    let on_event = closure
        .as_function()
        .ok_or(err)
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    let args = serde_event.to_v8(v8)?;
    let mini_command = on_event
        .call::<mini_v8::Values, mini_v8::Value>(mini_v8::Values::from_vec(vec![args]))
        .map_err(|e| anyhow::anyhow!("Function invocation failure: {}", e.to_string()))?;
    let serde_command = serde_json::Value::from_v8(&mini_command)?;
    log::debug!("command: {:?}", serde_command);
    Ok(serde_command)
}
