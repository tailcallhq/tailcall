use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use deno_core::v8::{self, Function, Global, Local, Object, Value};
use deno_core::{FastString, JsRuntime, PollEventLoopOptions, RuntimeOptions};
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio::sync::oneshot;
use tokio::task::spawn_local;
use tokio::time::{timeout_at, Instant};

use super::{JsResponse, JsRequest};
use super::channel::Message;
use crate::blueprint;
use crate::cli::javascript::extensions::{console, fetch, timer_promises};

pub type WorkerMessage = (oneshot::Sender<Message>, Message);

pub struct Worker {
    js_runtime: JsRuntime,
    function: Global<Function>,
}

// TODO: remove unwraps and handle errors
impl Worker {
    pub async fn new(script: blueprint::Script, http_sender: mpsc::UnboundedSender<(oneshot::Sender<JsResponse>, JsRequest)>) -> anyhow::Result<Self> {
        let mut js_runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![
                console::init_ops_and_esm(),
                timer_promises::init_ops_and_esm(),
                fetch::init_ops_and_esm(),
            ],
            ..Default::default()
        });

        js_runtime.op_state().borrow_mut().put(http_sender);

        let value = {
            let value = js_runtime.lazy_load_es_module_from_code(
                "file:///anon.js",
                FastString::from(script.source),
            )?;
            let scope = &mut js_runtime.handle_scope();
            let namespace = Local::new(scope, value);
            let namespace: Local<Object> = namespace.try_into()?;
            let key = v8::String::new(scope, "onEvent").unwrap();
            let value = namespace
                .get(scope, key.into())
                .ok_or(anyhow!("onEvent not found"))?;
            let value: Local<Function> = value.try_into()?;
            Global::new(scope, value)
        };
        log::debug!("JS Runtime created: {:?}", thread::current().name());
        Ok(Self { function: value, js_runtime })
    }

    pub async fn listen(mut self, mut receiver: UnboundedReceiver<WorkerMessage>) -> Result<()> {
        let (tx, mut rx) =
            mpsc::unbounded_channel::<(oneshot::Sender<Message>, Result<Global<Value>>)>();
        let mut has_tasks = false;
        loop {
            tokio::select! {
                biased;
                Some((send_response, request)) = receiver.recv() => {
                    has_tasks = true;
                    self.handle(request, send_response, tx.clone()).unwrap();
                },
                Some((send, value)) = rx.recv() => {
                    let scope = &mut self.js_runtime.handle_scope();
                    let value = Local::new(scope, value.unwrap());
                    let message: Message = serde_v8::from_v8(scope, value).unwrap();

                    let _ = send.send(message);
                },
                _ = self.js_runtime.run_event_loop(PollEventLoopOptions::default()), if has_tasks => {
                    has_tasks = false;
                }

            }
        }
    }

    pub fn handle(
        &mut self,
        message: Message,
        response: oneshot::Sender<Message>,
        // TODO: use type aliases
        sender: mpsc::UnboundedSender<(oneshot::Sender<Message>, Result<Global<Value>>)>,
    ) -> Result<()> {
        let message = {
            let mut scope = self.js_runtime.handle_scope();
            let message = serde_v8::to_v8(&mut scope, message)?;

            Global::new(&mut scope, message)
        };

        let call = self.js_runtime.call_with_args(&self.function, &[message]);
        // TODO: specify from config
        let call = timeout_at(Instant::now() + Duration::from_millis(1000), call);

        spawn_local(async move {
            let result = match call.await {
                Ok(result) => result,
                Err(_) => Err(anyhow!("Script timeout")),
            };

            sender.send((response, result)).unwrap()
        });

        Ok(())
    }
}
