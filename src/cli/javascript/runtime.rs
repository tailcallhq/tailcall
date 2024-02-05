use std::cell::{OnceCell, RefCell};
use std::thread;

use deno_core::{v8, FastString, JsRuntime};

use super::channel::{Message, MessageContent};
use crate::{blueprint, WorkerIO};

struct LocalRuntime {
    value: v8::Global<v8::Value>,
    js_runtime: JsRuntime,
}

thread_local! {
  static LOCAL_RUNTIME: RefCell<OnceCell<LocalRuntime>> = const { RefCell::new(OnceCell::new()) };
}

impl LocalRuntime {
    fn try_new(script: blueprint::Script) -> anyhow::Result<Self> {
        let source = create_closure(script.source.as_str());
        let mut js_runtime = JsRuntime::new(Default::default());
        let value = js_runtime.execute_script("<anon>", FastString::from(source))?;
        log::debug!("JS Runtime created: {:?}", thread::current().name());
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
        Self { script }
    }
}

#[async_trait::async_trait]
impl WorkerIO<Message, Message> for Runtime {
    fn dispatch(&self, event: Message) -> anyhow::Result<Message> {
        log::debug!("event: {:?}", event);
        let command = LOCAL_RUNTIME.with(move |cell| {
            let script = self.script.clone();
            // TODO: use `get_or_try_init`. Currently it is an unstable feature
            cell.borrow()
                .get_or_init(|| LocalRuntime::try_new(script).expect("JS runtime not initialized"));
            on_event(event)
        });
        command
    }
}

fn on_event(message: Message) -> anyhow::Result<Message> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let local_runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("JS runtime not initialized"))?;
        let scope = &mut local_runtime.js_runtime.handle_scope();
        let value = &local_runtime.value;
        let local_value = v8::Local::new(scope, value);
        let closure: v8::Local<v8::Function> = local_value.try_into()?;
        let input = serde_v8::to_v8(scope, message)?;
        log::debug!("js input: {:?}", input);
        let null_ctx = v8::null(scope);

        let output = closure.call(scope, null_ctx.into(), &[input]);
        match output {
            None => Ok(Message { message: MessageContent::Empty, id: None }),
            Some(output) => Ok(serde_v8::from_v8(scope, output)?),
        }
    })
}
