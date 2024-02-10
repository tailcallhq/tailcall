use std::cell::{OnceCell, RefCell};
use std::thread;

use deno_core::{extension, serde_v8, v8, FastString, JsRuntime, RuntimeOptions};

use super::channel::{Command, Event};
use crate::{blueprint, WorkerIO};

struct LocalRuntime {
    js_runtime: JsRuntime,
    channel: v8::Global<v8::Value>,
}

thread_local! {
  static LOCAL_RUNTIME: RefCell<OnceCell<LocalRuntime>> = const { RefCell::new(OnceCell::new()) };
}

impl LocalRuntime {
    fn try_new(script: blueprint::Script) -> anyhow::Result<Self> {
        let source = script.source;
        extension!(console, js = ["src/cli/javascript/shim/console.js",]);
        extension!(channel, js = ["src/cli/javascript/shim/channel.js",]);
        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![console::init_ops_and_esm(), channel::init_ops_and_esm()],
            ..Default::default()
        });
        let channel =
            js_runtime.execute_script("<anon>", FastString::from_static("globalThis.channel"))?;
        js_runtime.execute_script("<anon>", FastString::from(source))?;
        log::debug!("JS Runtime created: {:?}", thread::current().name());

        Ok(Self { js_runtime, channel })
    }
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
impl WorkerIO<Event, Command> for Runtime {
    fn dispatch(&self, event: Event) -> anyhow::Result<Vec<Command>> {
        log::debug!("event: {:?}", event);
        let command = LOCAL_RUNTIME.with(move |cell| {
            let script = self.script.clone();
            cell.borrow()
                .get_or_init(|| LocalRuntime::try_new(script).expect("JS runtime not initialized"));
            on_event(event)
        });
        command
    }
}

fn on_event(event: Event) -> anyhow::Result<Vec<Command>> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("JS runtime not initialized"))?;
        let scope = &mut runtime.js_runtime.handle_scope();
        let channel = v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &runtime.channel))?;
        let server_emit_key = v8::String::new(scope, "serverEmit").unwrap();

        let function = channel
            .get(scope, server_emit_key.into())
            .ok_or(anyhow::anyhow!("channel not initialized"))?;

        let function = v8::Local::<v8::Function>::try_from(function)?;
        let event = serde_v8::to_v8(scope, event)?;
        let command = function.call(scope, channel.into(), &[event]);

        match command {
            None => Ok(vec![]),
            Some(output) => Ok(serde_v8::from_v8(scope, output)?),
        }
    })
}
