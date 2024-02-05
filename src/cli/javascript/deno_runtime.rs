use std::cell::{OnceCell, RefCell};

use deno_core::{v8, FastString, JsRuntime};

use crate::{blueprint, WorkerIO};

use super::channel::Message;

struct LocalRuntime {
    value: v8::Global<v8::Value>,
    js_runtime: JsRuntime,
}

thread_local! {
  static LOCAL_RUNTIME: OnceCell<LocalRuntime> = const { OnceCell::new() };
  static JS_RUNTIME : RefCell<JsRuntime> = RefCell::new(JsRuntime::new(Default::default()));
}

impl LocalRuntime {
    fn try_new(script: blueprint::Script) -> anyhow::Result<Self> {
        let source = create_closure(script.source.as_str());
        let mut js_runtime = JsRuntime::new(Default::default());
        let value = js_runtime.execute_script("<anon>", FastString::from(source))?;
        Ok(Self { value, js_runtime })
    }
}

fn create_closure(script: &str) -> String {
    format!("(function() {{{} return onEvent}})();", script)
}

pub struct Runtime {
    script: blueprint::Script,
}

impl Runtime {
    pub fn new(script: blueprint::Script) -> Self {
        Self { script: script }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Message, Message> for Runtime {
    async fn dispatch(&self, event: Message) -> anyhow::Result<Message> {
        LOCAL_RUNTIME.with(move |local_runtime_cell| {
            let script = self.script.clone();
            let local_runtime =
                local_runtime_cell.get_or_init(|| LocalRuntime::try_new(script).unwrap());
            on_event_impl(local_runtime, event)
        })
    }
}

fn on_event_impl(rtm: &LocalRuntime, event: Message) -> anyhow::Result<Message> {
    todo!()
}
