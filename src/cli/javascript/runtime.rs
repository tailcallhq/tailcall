use std::cell::{OnceCell, RefCell};
use std::task::{Context, Waker};
use std::thread;

use deno_core::{
    extension, serde_v8, v8, FastString, JsRuntime, PollEventLoopOptions, RuntimeOptions,
};
use futures_util::task::noop_waker;

use super::channel::{Command, Event};
use crate::{blueprint, WorkerIO};

struct LocalRuntime {
    js_runtime: JsRuntime,
    channel: v8::Global<v8::Value>,
}

thread_local! {
    // Practically only one JS runtime is created because CHANNEL_RUNTIME is single threaded.
  static LOCAL_RUNTIME: RefCell<OnceCell<LocalRuntime>> = const { RefCell::new(OnceCell::new()) };
}

// Single threaded JS runtime, that's shared across all tokio workers.
lazy_static::lazy_static! {
    static ref CHANNEL_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .build()
        .expect("JS runtime not initialized");
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
    async fn dispatch(&self, event: Event) -> anyhow::Result<Vec<Command>> {
        let script = self.script.clone();
        CHANNEL_RUNTIME
            .spawn(async move {
                LOCAL_RUNTIME.with(move |cell| {
                    let script = script.clone();
                    cell.borrow()
                        .get_or_init(|| LocalRuntime::try_new(script).unwrap());
                });
                on_event(event)?;
                poll_event_loop()?;
                let command = get_commands();
                command
            })
            .await?
    }
}

fn on_event(event: Event) -> anyhow::Result<()> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("JS runtime not initialized"))?;
        let js_runtime = &mut runtime.js_runtime;
        let scope = &mut js_runtime.handle_scope();
        let channel = v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &runtime.channel))?;
        let event = serde_v8::to_v8(scope, event)?;
        let fn_server_emit = v8::String::new(scope, "serverEmit").unwrap();
        let fn_server_emit = channel
            .get(scope, fn_server_emit.into())
            .ok_or(anyhow::anyhow!("channel not initialized"))?;

        let fn_server_emit = v8::Local::<v8::Function>::try_from(fn_server_emit)?;
        fn_server_emit.call(scope, channel.into(), &[event]);

        Ok(())
    })
}

fn get_commands() -> anyhow::Result<Vec<Command>> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("JS runtime not initialized"))?;
        let js_runtime = &mut runtime.js_runtime;
        let scope = &mut js_runtime.handle_scope();
        let channel = v8::Local::<v8::Object>::try_from(v8::Local::new(scope, &runtime.channel))?;
        let func = v8::String::new(scope, "getMessages").unwrap();
        let func = channel
            .get(scope, func.into())
            .ok_or(anyhow::anyhow!("channel not initialized"))?;

        let func = v8::Local::<v8::Function>::try_from(func)?;
        let command = func.call(scope, channel.into(), &[]);
        match command {
            None => Ok(vec![]),
            Some(output) => Ok(serde_v8::from_v8(scope, output)?),
        }
    })
}

fn poll_event_loop() -> anyhow::Result<()> {
    LOCAL_RUNTIME.with_borrow_mut(|cell| {
        let runtime = cell
            .get_mut()
            .ok_or(anyhow::anyhow!("JS runtime not initialized"))
            .expect("JS runtime not initialized");
        let js_runtime = &mut runtime.js_runtime;
        let waker = &mut Waker::from(noop_waker());
        let mut context = Context::from_waker(waker);
        let poll_options =
            PollEventLoopOptions { wait_for_inspector: true, pump_v8_message_loop: true };

        loop {
            let poll = js_runtime.poll_event_loop(&mut context, poll_options);

            if poll.is_pending() {
                log::info!("JS Runtime polling: {:?}", thread::current().name());
            } else {
                break;
            }
        }

        Ok(())
    })
}
